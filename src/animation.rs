//! Animation primitives shared across subsystems.
//!
//! `AnimatedValue` is the shared primitive: a single f64 that chases a target
//! using configurable easing and duration (Invariant 14: no visual discontinuity
//! when interrupted mid-animation).
//!
//! `AnimationManager` owns overlay/palette transitions (0-1 per kind, pruned on
//! completion, returns normalized progress). `DimLayer` (in focus_mode.rs) owns
//! N animated values (one per line, never pruned, uses chase semantics). Both
//! build on `AnimatedValue` as the underlying primitive.
//!
//! ## Decision rule: which subsystem to use
//!
//! - **`AnimationManager`** — global, discrete transitions with a start and end.
//!   Use for palette crossfades, overlay opacity fade-in, or any effect that runs
//!   once to completion then is pruned. Finite count (0-1 per `TransitionKind`).
//!
//! - **`AnimatedValue` / `DimLayer`** — per-line, chase-to-target values that
//!   persist across frames. Use for paragraph and sentence dimming where each line
//!   independently tracks a target opacity. Pre-allocated, zero steady-state allocs.
//!   All dimming uses opacity-based color interpolation, not distance-based.

use std::time::{Duration, Instant};

use crate::palette::Palette;

/// Configuration pairing duration and easing curve for animated transitions.
#[derive(Debug, Clone, Copy)]
pub struct FadeConfig {
    pub duration: Duration,
    pub easing: Easing,
}

impl Default for FadeConfig {
    fn default() -> Self {
        Self {
            duration: Duration::from_millis(150),
            easing: Easing::EaseOut,
        }
    }
}

/// A single f64 value that animates toward a target using chase semantics.
/// Interrupting mid-animation starts a new transition from the current visual value,
/// guaranteeing smooth, discontinuity-free transitions (Invariant 14).
#[derive(Debug, Clone)]
pub struct AnimatedValue {
    pub target: f64,
    pub start_value: f64,
    pub start_time: Option<Instant>,
    fade_config: FadeConfig,
}

impl AnimatedValue {
    /// Create an AnimatedValue already at `value` with no animation in flight.
    pub fn new(value: f64) -> Self {
        Self {
            target: value,
            start_value: value,
            start_time: None,
            fade_config: FadeConfig::default(),
        }
    }

    /// Set a new target. Captures the current visual state as `start_value`
    /// so the animation chases from the current position.
    /// No-ops if the target hasn't changed (within epsilon).
    /// Returns true if a new animation was started.
    pub fn set_target(&mut self, new_target: f64, config: FadeConfig) -> bool {
        if (new_target - self.target).abs() < f64::EPSILON {
            return false;
        }
        self.start_value = self.current();
        self.target = new_target;
        self.start_time = Some(Instant::now());
        self.fade_config = config;
        true
    }

    /// Returns the current visual value accounting for animation progress.
    /// Returns `target` if no animation is in flight or the animation is complete.
    pub fn current(&self) -> f64 {
        let start_time = match self.start_time {
            Some(t) => t,
            None => return self.target,
        };

        let total = self.fade_config.duration.as_secs_f64();
        if total <= 0.0 {
            return self.target;
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        let t = (elapsed / total).min(1.0);
        let eased = self.fade_config.easing.apply(t);
        self.start_value + (self.target - self.start_value) * eased
    }

    /// Returns true if an animation is still in flight.
    pub fn is_animating(&self) -> bool {
        match self.start_time {
            None => false,
            Some(t) => t.elapsed() < self.fade_config.duration,
        }
    }
}

/// Easing curve selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Easing {
    EaseOut,
    EaseInOut,
}

/// Cubic ease-out: 1 - (1-t)^3
pub fn ease_out(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    1.0 - inv * inv * inv
}

/// Cubic ease-in-out: 3t^2 - 2t^3
pub fn ease_in_out(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    3.0 * t * t - 2.0 * t * t * t
}

impl Easing {
    pub fn apply(self, t: f64) -> f64 {
        match self {
            Easing::EaseOut => ease_out(t),
            Easing::EaseInOut => ease_in_out(t),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TransitionKind {
    Palette { from: Box<Palette>, to: Box<Palette> },
    OverlayOpacity,
    ScratchQuitOverlay,
}

impl TransitionKind {
    pub fn same_kind(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

#[derive(Debug, Clone)]
pub struct Transition {
    pub kind: TransitionKind,
    pub start: Instant,
    pub duration: Duration,
    pub easing: Easing,
}

impl Transition {
    pub fn new(kind: TransitionKind, duration: Duration, easing: Easing) -> Self {
        Self {
            kind,
            start: Instant::now(),
            duration,
            easing,
        }
    }

    fn linear_progress(&self) -> f64 {
        let elapsed = self.start.elapsed().as_secs_f64();
        let total = self.duration.as_secs_f64();
        if total <= 0.0 {
            1.0
        } else {
            (elapsed / total).min(1.0)
        }
    }

    pub fn progress(&self) -> f64 {
        self.easing.apply(self.linear_progress())
    }

    pub fn is_complete(&self) -> bool {
        self.start.elapsed() >= self.duration
    }
}

#[derive(Debug, Clone, Default)]
pub struct AnimationManager {
    pub transitions: Vec<Transition>,
}

impl AnimationManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start(&mut self, kind: TransitionKind, duration: Duration, easing: Easing) {
        self.transitions.retain(|t| !t.kind.same_kind(&kind));
        self.transitions.push(Transition::new(kind, duration, easing));
    }

    pub fn is_active(&self) -> bool {
        self.transitions.iter().any(|t| !t.is_complete())
    }

    pub fn tick(&mut self) {
        self.transitions.retain(|t| !t.is_complete());
    }

    pub fn palette_progress(&self) -> Option<(f64, &Palette, &Palette)> {
        self.transitions
            .iter()
            .find(|t| matches!(t.kind, TransitionKind::Palette { .. }))
            .and_then(|t| match &t.kind {
                TransitionKind::Palette { from, to } => {
                    Some((t.progress(), from.as_ref(), to.as_ref()))
                }
                _ => None,
            })
    }

    pub fn overlay_progress(&self) -> Option<f64> {
        self.transitions
            .iter()
            .find(|t| matches!(t.kind, TransitionKind::OverlayOpacity))
            .map(|t| t.progress())
    }

    pub fn scratch_quit_overlay_progress(&self) -> Option<f64> {
        self.transitions
            .iter()
            .find(|t| matches!(t.kind, TransitionKind::ScratchQuitOverlay))
            .map(|t| t.progress())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Task 1: Easing function tests ===

    #[test]
    fn ease_out_at_zero_is_zero() {
        assert_eq!(ease_out(0.0), 0.0);
    }

    #[test]
    fn ease_out_at_one_is_one() {
        assert!((ease_out(1.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ease_out_is_monotonic() {
        let mut prev = ease_out(0.0);
        for i in 1..=100 {
            let t = i as f64 / 100.0;
            let curr = ease_out(t);
            assert!(curr >= prev, "ease_out not monotonic at t={t}: {curr} < {prev}");
            prev = curr;
        }
    }

    #[test]
    fn ease_in_out_at_zero_is_zero() {
        assert_eq!(ease_in_out(0.0), 0.0);
    }

    #[test]
    fn ease_in_out_at_one_is_one() {
        assert!((ease_in_out(1.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ease_in_out_at_half_is_half() {
        assert!((ease_in_out(0.5) - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn easing_apply_dispatches_correctly() {
        let t = 0.3;
        assert_eq!(Easing::EaseOut.apply(t), ease_out(t));
        assert_eq!(Easing::EaseInOut.apply(t), ease_in_out(t));
    }

    // === Task 2: Transition tests ===

    #[test]
    fn transition_created_now_has_progress_near_zero() {
        let t = Transition::new(
            TransitionKind::OverlayOpacity,
            Duration::from_secs(1),
            Easing::EaseOut,
        );
        assert!(t.progress() < 0.1, "Expected progress near 0, got {}", t.progress());
    }

    #[test]
    fn transition_in_the_past_is_complete() {
        let past_start = Instant::now() - Duration::from_secs(5);
        let t = Transition {
            kind: TransitionKind::OverlayOpacity,
            start: past_start,
            duration: Duration::from_secs(1),
            easing: Easing::EaseOut,
        };
        assert!(t.is_complete());
    }

    #[test]
    fn transition_in_the_past_has_progress_near_one() {
        let past_start = Instant::now() - Duration::from_secs(5);
        let t = Transition {
            kind: TransitionKind::OverlayOpacity,
            start: past_start,
            duration: Duration::from_secs(1),
            easing: Easing::EaseOut,
        };
        assert!((t.progress() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn same_kind_matches_overlay_with_overlay() {
        let a = TransitionKind::OverlayOpacity;
        let b = TransitionKind::OverlayOpacity;
        assert!(a.same_kind(&b));
    }

    #[test]
    fn same_kind_does_not_match_palette_with_overlay() {
        let p = Palette::default_palette();
        let a = TransitionKind::Palette { from: Box::new(p), to: Box::new(p) };
        let b = TransitionKind::OverlayOpacity;
        assert!(!a.same_kind(&b));
    }

    // === Task 3: AnimationManager tests ===

    #[test]
    fn manager_starts_empty_and_inactive() {
        let mgr = AnimationManager::new();
        assert!(!mgr.is_active());
    }

    #[test]
    fn manager_tracks_started_transition() {
        let mut mgr = AnimationManager::new();
        mgr.start(
            TransitionKind::OverlayOpacity,
            Duration::from_secs(1),
            Easing::EaseOut,
        );
        assert!(mgr.is_active());
        assert!(mgr.overlay_progress().is_some());
    }

    #[test]
    fn manager_replaces_same_kind() {
        let mut mgr = AnimationManager::new();
        mgr.start(
            TransitionKind::OverlayOpacity,
            Duration::from_secs(1),
            Easing::EaseOut,
        );
        mgr.start(
            TransitionKind::OverlayOpacity,
            Duration::from_secs(1),
            Easing::EaseOut,
        );
        let overlay_count = mgr
            .transitions
            .iter()
            .filter(|t| matches!(t.kind, TransitionKind::OverlayOpacity))
            .count();
        assert_eq!(overlay_count, 1, "Expected 1 overlay transition after replacing, got {overlay_count}");
    }

    #[test]
    fn manager_tick_removes_completed() {
        let mut mgr = AnimationManager::new();
        mgr.transitions.push(Transition {
            kind: TransitionKind::OverlayOpacity,
            start: Instant::now() - Duration::from_secs(5),
            duration: Duration::from_secs(1),
            easing: Easing::EaseOut,
        });
        assert!(!mgr.is_active());
        assert_eq!(mgr.transitions.len(), 1);
        mgr.tick();
        assert!(mgr.transitions.is_empty(), "tick() should prune completed transitions");
    }

    #[test]
    fn multiple_animation_kinds_coexist() {
        let p = Palette::default_palette();
        let mut m = AnimationManager::new();
        m.start(
            TransitionKind::Palette { from: Box::new(p), to: Box::new(p) },
            Duration::from_millis(150),
            Easing::EaseInOut,
        );
        m.start(
            TransitionKind::OverlayOpacity,
            Duration::from_millis(150),
            Easing::EaseOut,
        );
        assert_eq!(m.transitions.len(), 2);
        assert!(m.palette_progress().is_some());
        assert!(m.overlay_progress().is_some());
    }
}
