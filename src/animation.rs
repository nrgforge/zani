use std::time::{Duration, Instant};

use crate::palette::Palette;

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
    Scroll { from: f64, to: f64 },
    FocusDimming { from_line: usize, to_line: usize },
    Palette { from: Box<Palette>, to: Box<Palette> },
    OverlayOpacity { appearing: bool },
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
            TransitionKind::Scroll { from: 0.0, to: 10.0 },
            Duration::from_secs(1),
            Easing::EaseOut,
        );
        // Progress should be very small since we just created it
        assert!(t.progress() < 0.1, "Expected progress near 0, got {}", t.progress());
    }

    #[test]
    fn transition_in_the_past_is_complete() {
        let past_start = Instant::now() - Duration::from_secs(5);
        let t = Transition {
            kind: TransitionKind::Scroll { from: 0.0, to: 10.0 },
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
            kind: TransitionKind::Scroll { from: 0.0, to: 10.0 },
            start: past_start,
            duration: Duration::from_secs(1),
            easing: Easing::EaseOut,
        };
        assert!((t.progress() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn same_kind_matches_scroll_with_scroll() {
        let a = TransitionKind::Scroll { from: 0.0, to: 5.0 };
        let b = TransitionKind::Scroll { from: 3.0, to: 8.0 };
        assert!(a.same_kind(&b));
    }

    #[test]
    fn same_kind_does_not_match_scroll_with_focus_dimming() {
        let a = TransitionKind::Scroll { from: 0.0, to: 5.0 };
        let b = TransitionKind::FocusDimming { from_line: 0, to_line: 3 };
        assert!(!a.same_kind(&b));
    }
}
