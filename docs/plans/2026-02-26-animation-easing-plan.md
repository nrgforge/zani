# Animation & Easing Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make every visual state change in Zani interpolated — smooth scroll, focus dimming crossfade, palette crossfade, and overlay fade.

**Architecture:** A centralized `AnimationManager` in `App` tracks active transitions. The event loop polls at 16ms during animation, 250ms idle. Easing math is two pure cubic functions. `palette::interpolate()` already exists. No external dependencies.

**Tech Stack:** Rust, ratatui, crossterm, `std::time::Instant`

---

### Task 1: Easing Functions

**Files:**
- Create: `src/animation.rs`
- Modify: `src/lib.rs:1-18`

**Step 1: Write the failing tests**

In `src/animation.rs`, add a `#[cfg(test)] mod tests` block with these tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ease_out_at_zero_is_zero() {
        assert!((ease_out(0.0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn ease_out_at_one_is_one() {
        assert!((ease_out(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn ease_out_is_monotonic() {
        let mut prev = 0.0;
        for i in 1..=10 {
            let t = i as f64 / 10.0;
            let val = ease_out(t);
            assert!(val >= prev, "ease_out should be monotonic at t={}", t);
            prev = val;
        }
    }

    #[test]
    fn ease_in_out_at_zero_is_zero() {
        assert!((ease_in_out(0.0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn ease_in_out_at_one_is_one() {
        assert!((ease_in_out(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn ease_in_out_at_half_is_half() {
        assert!((ease_in_out(0.5) - 0.5).abs() < 0.001);
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib animation -- --nocapture`
Expected: compilation error — `ease_out` and `ease_in_out` not defined.

**Step 3: Write the easing functions**

At the top of `src/animation.rs`:

```rust
use std::time::{Duration, Instant};

/// Easing curve selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Easing {
    /// Fast start, gentle landing: 1 - (1-t)^3
    EaseOut,
    /// Smooth acceleration/deceleration: 3t^2 - 2t^3
    EaseInOut,
}

/// Cubic ease-out: fast start, gentle landing.
pub fn ease_out(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    1.0 - inv * inv * inv
}

/// Cubic ease-in-out: smooth acceleration then deceleration.
pub fn ease_in_out(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    3.0 * t * t - 2.0 * t * t * t
}

impl Easing {
    /// Apply the easing curve to a linear progress value.
    pub fn apply(self, t: f64) -> f64 {
        match self {
            Easing::EaseOut => ease_out(t),
            Easing::EaseInOut => ease_in_out(t),
        }
    }
}
```

Add `pub mod animation;` to `src/lib.rs`.

**Step 4: Run tests to verify they pass**

Run: `cargo test --lib animation`
Expected: 6 tests PASS.

**Step 5: Commit**

```bash
git add src/animation.rs src/lib.rs
git commit -m "feat: add easing functions (ease-out, ease-in-out)"
```

---

### Task 2: TransitionKind and Transition

**Files:**
- Modify: `src/animation.rs`

**Step 1: Write the failing tests**

Add to `tests` module:

```rust
#[test]
fn transition_progress_before_start_is_zero() {
    // A transition that starts "now" should have progress ~0.0
    let t = Transition::new(
        TransitionKind::Scroll { from: 0.0, to: 10.0 },
        Duration::from_millis(150),
        Easing::EaseOut,
    );
    let p = t.progress();
    assert!(p < 0.1, "Progress at creation should be near 0, got {}", p);
}

#[test]
fn transition_is_complete_after_duration() {
    let start = Instant::now() - Duration::from_millis(200);
    let t = Transition {
        kind: TransitionKind::Scroll { from: 0.0, to: 10.0 },
        start,
        duration: Duration::from_millis(150),
        easing: Easing::EaseOut,
    };
    assert!(t.is_complete());
    assert!((t.progress() - 1.0).abs() < 0.001);
}

#[test]
fn transition_kind_discriminant_matches() {
    let a = TransitionKind::Scroll { from: 0.0, to: 5.0 };
    let b = TransitionKind::Scroll { from: 1.0, to: 9.0 };
    let c = TransitionKind::FocusDimming { from_line: 0, to_line: 1 };
    assert!(a.same_kind(&b));
    assert!(!a.same_kind(&c));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib animation`
Expected: compilation error — `Transition`, `TransitionKind` not defined.

**Step 3: Write the types**

Add to `src/animation.rs` above the tests:

```rust
use crate::palette::Palette;

/// What property is being animated.
#[derive(Debug, Clone)]
pub enum TransitionKind {
    /// Smooth viewport scroll between offsets.
    Scroll { from: f64, to: f64 },
    /// Focus dimming crossfade between cursor lines.
    FocusDimming { from_line: usize, to_line: usize },
    /// Palette crossfade between two palettes.
    Palette { from: Box<Palette>, to: Box<Palette> },
    /// Overlay (settings/find) fade in/out.
    OverlayOpacity { appearing: bool },
}

impl TransitionKind {
    /// Check if two kinds animate the same property (ignoring values).
    pub fn same_kind(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

/// A single in-progress animation.
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

    /// Linear progress (0.0 to 1.0), clamped.
    fn linear_progress(&self) -> f64 {
        let elapsed = self.start.elapsed().as_secs_f64();
        let total = self.duration.as_secs_f64();
        if total <= 0.0 { 1.0 } else { (elapsed / total).min(1.0) }
    }

    /// Eased progress (0.0 to 1.0).
    pub fn progress(&self) -> f64 {
        self.easing.apply(self.linear_progress())
    }

    /// Whether this transition has finished.
    pub fn is_complete(&self) -> bool {
        self.start.elapsed() >= self.duration
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --lib animation`
Expected: 9 tests PASS.

**Step 5: Commit**

```bash
git add src/animation.rs
git commit -m "feat: add Transition and TransitionKind types"
```

---

### Task 3: AnimationManager

**Files:**
- Modify: `src/animation.rs`

**Step 1: Write the failing tests**

Add to `tests` module:

```rust
#[test]
fn manager_starts_empty_and_inactive() {
    let m = AnimationManager::new();
    assert!(!m.is_active());
}

#[test]
fn manager_tracks_started_transition() {
    let mut m = AnimationManager::new();
    m.start(
        TransitionKind::Scroll { from: 0.0, to: 10.0 },
        Duration::from_millis(150),
        Easing::EaseOut,
    );
    assert!(m.is_active());
    assert!(m.scroll_progress().is_some());
}

#[test]
fn manager_replaces_same_kind() {
    let mut m = AnimationManager::new();
    m.start(
        TransitionKind::Scroll { from: 0.0, to: 5.0 },
        Duration::from_millis(150),
        Easing::EaseOut,
    );
    m.start(
        TransitionKind::Scroll { from: 5.0, to: 10.0 },
        Duration::from_millis(150),
        Easing::EaseOut,
    );
    // Should only have 1 scroll transition
    let count = m.transitions.iter().filter(|t| matches!(t.kind, TransitionKind::Scroll { .. })).count();
    assert_eq!(count, 1);
}

#[test]
fn manager_tick_removes_completed() {
    let mut m = AnimationManager::new();
    let t = Transition {
        kind: TransitionKind::Scroll { from: 0.0, to: 5.0 },
        start: Instant::now() - Duration::from_millis(200),
        duration: Duration::from_millis(100),
        easing: Easing::EaseOut,
    };
    m.transitions.push(t);
    assert!(m.is_active());
    m.tick();
    assert!(!m.is_active());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib animation`
Expected: compilation error — `AnimationManager` not defined.

**Step 3: Write AnimationManager**

Add to `src/animation.rs`:

```rust
/// Centralized manager for all active transitions.
#[derive(Debug, Clone)]
pub struct AnimationManager {
    pub transitions: Vec<Transition>,
}

impl Default for AnimationManager {
    fn default() -> Self {
        Self { transitions: Vec::new() }
    }
}

impl AnimationManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a new transition. Replaces any existing transition of the same kind.
    pub fn start(&mut self, kind: TransitionKind, duration: Duration, easing: Easing) {
        self.transitions.retain(|t| !t.kind.same_kind(&kind));
        self.transitions.push(Transition::new(kind, duration, easing));
    }

    /// Whether any transitions are active.
    pub fn is_active(&self) -> bool {
        !self.transitions.is_empty()
    }

    /// Remove completed transitions.
    pub fn tick(&mut self) {
        self.transitions.retain(|t| !t.is_complete());
    }

    /// Get scroll transition progress, if active.
    pub fn scroll_progress(&self) -> Option<f64> {
        self.transitions.iter()
            .find(|t| matches!(t.kind, TransitionKind::Scroll { .. }))
            .map(|t| t.progress())
    }

    /// Get scroll transition endpoints, if active.
    pub fn scroll_values(&self) -> Option<(f64, f64)> {
        self.transitions.iter()
            .find(|t| matches!(t.kind, TransitionKind::Scroll { .. }))
            .and_then(|t| match &t.kind {
                TransitionKind::Scroll { from, to } => Some((*from, *to)),
                _ => None,
            })
    }

    /// Get focus dimming transition progress and line info, if active.
    pub fn focus_progress(&self) -> Option<(f64, usize, usize)> {
        self.transitions.iter()
            .find(|t| matches!(t.kind, TransitionKind::FocusDimming { .. }))
            .and_then(|t| match &t.kind {
                TransitionKind::FocusDimming { from_line, to_line } => {
                    Some((t.progress(), *from_line, *to_line))
                }
                _ => None,
            })
    }

    /// Get palette transition progress and palettes, if active.
    pub fn palette_progress(&self) -> Option<(f64, &Palette, &Palette)> {
        self.transitions.iter()
            .find(|t| matches!(t.kind, TransitionKind::Palette { .. }))
            .and_then(|t| match &t.kind {
                TransitionKind::Palette { from, to } => {
                    Some((t.progress(), from.as_ref(), to.as_ref()))
                }
                _ => None,
            })
    }

    /// Get overlay opacity progress (0.0 = invisible, 1.0 = fully visible), if active.
    pub fn overlay_progress(&self) -> Option<f64> {
        self.transitions.iter()
            .find(|t| matches!(t.kind, TransitionKind::OverlayOpacity { .. }))
            .and_then(|t| match &t.kind {
                TransitionKind::OverlayOpacity { appearing } => {
                    let p = t.progress();
                    Some(if *appearing { p } else { 1.0 - p })
                }
                _ => None,
            })
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --lib animation`
Expected: 13 tests PASS.

**Step 5: Commit**

```bash
git add src/animation.rs
git commit -m "feat: add AnimationManager with transition tracking"
```

---

### Task 4: Wire AnimationManager into App and Event Loop

**Files:**
- Modify: `src/app.rs:1-128` (imports and App struct)
- Modify: `src/main.rs:106-151` (run function)

**Step 1: Add `animations` field to App**

In `src/app.rs`, add import:
```rust
use crate::animation::AnimationManager;
```

Add to `App` struct (after `find_state`):
```rust
    /// Animation manager for smooth transitions.
    pub animations: AnimationManager,
```

In `App::new()`, add:
```rust
            animations: AnimationManager::new(),
```

**Step 2: Make event loop poll timeout dynamic**

In `src/main.rs:106-151`, the `run()` function. Change line 130:

```rust
// Before:
if event::poll(Duration::from_millis(250))? {

// After:
let poll_timeout = if app.animations.is_active() {
    Duration::from_millis(16)
} else {
    Duration::from_millis(250)
};
if event::poll(poll_timeout)? {
```

After the draw call (after line 120), add:
```rust
        app.animations.tick();
```

Add import in `main.rs`:
```rust
use std::time::Instant;
```
(Only if not already present — check first.)

**Step 3: Run tests to verify nothing broke**

Run: `cargo test`
Expected: All existing tests PASS (no behavior change yet).

**Step 4: Commit**

```bash
git add src/app.rs src/main.rs
git commit -m "feat: wire AnimationManager into App and dynamic poll timeout"
```

---

### Task 5: Smooth Scroll

**Files:**
- Modify: `src/app.rs` (scroll_offset type change, ensure_cursor_visible)
- Modify: `src/writing_surface.rs` (scroll_offset builder, render)
- Modify: `src/ui.rs` (scroll_offset usage)
- Modify: `src/main.rs` (scroll_offset usage)

This is the most invasive change because `scroll_offset` changes from `usize` to `f64`. The approach:

1. Add a `scroll_target: usize` (the integer scroll destination) alongside existing `scroll_offset: usize`.
2. Add a `scroll_offset_f64: f64` for the animated value.
3. WritingSurface receives the f64 and uses it for rendering (partial first line).
4. `ensure_cursor_visible` sets the target; the event loop interpolates.

**Actually — simpler approach:** Keep `scroll_offset: usize` as the *target*. Introduce `scroll_display: f64` as the *rendered* value. The animation interpolates `scroll_display` toward `scroll_offset`. WritingSurface receives `scroll_display`.

**Step 1: Add scroll_display field and animation trigger**

In `App` struct, add:
```rust
    /// Displayed scroll offset (f64 for smooth animation).
    /// Interpolates toward `scroll_offset` when a scroll animation is active.
    pub scroll_display: f64,
```

In `App::new()`:
```rust
            scroll_display: 0.0,
```

In `ensure_cursor_visible`, after any line that sets `self.scroll_offset = X`, start a scroll animation:

```rust
// After setting self.scroll_offset:
let old_display = self.scroll_display;
let new_target = self.scroll_offset as f64;
if (old_display - new_target).abs() > 0.01 {
    use crate::animation::{TransitionKind, Easing};
    use std::time::Duration;
    self.animations.start(
        TransitionKind::Scroll { from: old_display, to: new_target },
        Duration::from_millis(150),
        Easing::EaseOut,
    );
}
```

**Step 2: Update scroll_display each frame**

In `main.rs::run()`, after `app.animations.tick()`, add:

```rust
// Update smooth scroll display value
if let Some(progress) = app.animations.scroll_progress() {
    if let Some((from, to)) = app.animations.scroll_values() {
        app.scroll_display = from + (to - from) * progress;
    }
} else {
    app.scroll_display = app.scroll_offset as f64;
}
```

**Step 3: WritingSurface uses scroll_display**

Change `WritingSurface::scroll_offset` from `usize` to `f64`:

In `src/writing_surface.rs`:
- Field: `scroll_offset: f64` (was `usize`)
- Builder: `pub fn scroll_offset(mut self, offset: f64) -> Self`
- Default: `scroll_offset: 0.0`
- In `render()`, compute the integer visible range and partial offset:
  ```rust
  let visible_start = self.scroll_offset.floor() as usize;
  let fractional = self.scroll_offset.fract();
  // fractional is 0.0-1.0 — shift y position of first line up by this amount
  ```
  For the y calculation of each line:
  ```rust
  // Before: let y = area.top() + self.vertical_offset + screen_row as u16;
  // After: skip the first line's fractional part by checking if screen_row == 0
  ```
  Actually, ratatui cells are integer positions — we can't render half-rows. The smooth effect comes from the *transition itself* moving through integer positions quickly. At 60fps and 150ms, a 5-line scroll takes ~9ms per line, which looks fluid.

  **Revised approach:** Keep `scroll_offset` as `usize` in WritingSurface. The smooth animation effect comes from the event loop updating `app.scroll_offset` through intermediate integer values as the animation progresses. This avoids fractional rendering complexity.

  So: `scroll_display` is f64 internally but we round it for the actual `scroll_offset` passed to WritingSurface:
  ```rust
  .scroll_offset(app.scroll_display.round() as usize)
  ```

  This means WritingSurface doesn't change at all. The smoothness comes from rapidly updating `scroll_display` at 60fps, rounding to the nearest integer, giving step-wise scrolling at high frame rate.

**Step 4: Update ui.rs to pass scroll_display**

In `src/ui.rs`, line 33:
```rust
// Before:
.scroll_offset(app.scroll_offset)

// After:
.scroll_offset(app.scroll_display.round() as usize)
```

And line 69 (cursor position calculation):
```rust
// Before:
let screen_row = vl_idx.saturating_sub(app.scroll_offset);

// After:
let screen_row = vl_idx.saturating_sub(app.scroll_display.round() as usize);
```

**Step 5: Write tests**

In `src/app.rs` tests:

```rust
#[test]
fn scroll_animation_starts_on_scroll_change() {
    let mut app = App::new();
    app.buffer = Buffer::from_text(&"line\n".repeat(50));
    app.scroll_display = 0.0;
    app.scroll_offset = 0;
    let visual_lines = app.visual_lines();
    app.cursor_line = 30;
    app.cursor_col = 0;
    app.ensure_cursor_visible(&visual_lines, 20);
    // scroll_offset should have changed
    assert!(app.scroll_offset > 0);
    // A scroll animation should be active
    assert!(app.animations.is_active());
}
```

**Step 6: Run tests to verify**

Run: `cargo test`
Expected: All tests PASS.

**Step 7: Commit**

```bash
git add src/app.rs src/main.rs src/ui.rs
git commit -m "feat: add smooth scroll animation"
```

---

### Task 6: Focus Dimming Crossfade

**Files:**
- Modify: `src/app.rs` (track previous cursor line, start animation)
- Modify: `src/writing_surface.rs` (accept animation progress, blend distances)
- Modify: `src/ui.rs` (pass animation progress to WritingSurface)

**Step 1: Track previous cursor line in App**

Add to `App` struct:
```rust
    /// Previous cursor line, for focus dimming crossfade.
    pub prev_cursor_line: usize,
```

In `App::new()`:
```rust
            prev_cursor_line: 0,
```

**Step 2: Start focus animation on line change**

In `src/main.rs`, in the `run()` loop, after `handle_key` is called, check if the cursor line changed:

```rust
let prev_line = app.cursor_line;
// ... (handle_key happens here in the match) ...
// After the event handling, outside the poll block:
if app.cursor_line != app.prev_cursor_line {
    use zani::animation::{TransitionKind, Easing};
    app.animations.start(
        TransitionKind::FocusDimming {
            from_line: app.prev_cursor_line,
            to_line: app.cursor_line,
        },
        Duration::from_millis(150),
        Easing::EaseOut,
    );
    app.prev_cursor_line = app.cursor_line;
}
```

Actually, better to do this inside `main.rs::run()` at the top of the loop, comparing against a saved value:

```rust
// At top of loop, before draw:
let line_before = app.cursor_line;

// ... (draw, poll, handle_key) ...

// After poll/handle block, before autosave:
if app.cursor_line != line_before && app.focus_mode != zani::focus_mode::FocusMode::Off {
    app.animations.start(
        zani::animation::TransitionKind::FocusDimming {
            from_line: line_before,
            to_line: app.cursor_line,
        },
        Duration::from_millis(150),
        zani::animation::Easing::EaseOut,
    );
}
```

**Step 3: Pass focus animation to WritingSurface**

Add to `WritingSurface`:
```rust
    /// Focus dimming animation: (progress, from_line, to_line).
    focus_anim: Option<(f64, usize, usize)>,
```

Builder method:
```rust
    pub fn focus_animation(mut self, anim: Option<(f64, usize, usize)>) -> Self {
        self.focus_anim = anim;
        self
    }
```

In `ui.rs::draw()`, when building the surface, add:
```rust
        .focus_animation(app.animations.focus_progress())
```

**Step 4: Blend focus distances in WritingSurface render**

In `WritingSurface::render()`, where `line_distance` is computed (around line 244-253), blend when animation is active:

```rust
let line_distance = if use_sentence_dimming {
    0
} else if let Some((progress, from_line, to_line)) = self.focus_anim {
    // Blend between old and new focus distances
    let old_dist = focus_mode::line_distance(
        self.focus_mode, vl.logical_line, from_line, self.paragraph_bounds,
    );
    let new_dist = focus_mode::line_distance(
        self.focus_mode, vl.logical_line, to_line, self.paragraph_bounds,
    );
    // Interpolate: at progress=0, use old; at progress=1, use new
    // Use f64 for smooth blending, then clamp
    let blended = old_dist as f64 * (1.0 - progress) + new_dist as f64 * progress;
    blended.round() as usize
} else {
    focus_mode::line_distance(
        self.focus_mode, vl.logical_line, self.active_line, self.paragraph_bounds,
    )
};
```

**Step 5: Write tests**

In `src/writing_surface.rs` tests:

```rust
#[test]
fn focus_animation_blends_distances() {
    let text = "Line 0\nLine 1\nLine 2\nLine 3\nLine 4";
    let buffer = Buffer::from_text(text);
    let palette = Palette::default_palette();
    let area = Rect::new(0, 0, 80, 5);

    // Animation midway: transitioning focus from line 0 to line 4
    // At progress=0.5, line 2 should have blended distance
    let surface = WritingSurface::new(&buffer, &palette)
        .column_width(60)
        .focus_mode(FocusMode::Typewriter)
        .active_line(4)
        .focus_animation(Some((0.5, 0, 4)));

    let mut buf = RatatuiBuffer::empty(area);
    surface.render(area, &mut buf);

    // Line 2: old distance from line 0 = 2, new distance from line 4 = 2
    // Blended = 2.0 — same dimming either way for the midpoint line
    // Line 0: old distance = 0, new distance = 4 → blended = 2.0
    // Line 4: old distance = 4, new distance = 0 → blended = 2.0
    // At midpoint, all lines converge toward distance 2 — test that rendering doesn't panic
    // (The actual visual test is manual)
}
```

**Step 6: Run tests**

Run: `cargo test`
Expected: All tests PASS.

**Step 7: Commit**

```bash
git add src/app.rs src/main.rs src/ui.rs src/writing_surface.rs
git commit -m "feat: add focus dimming crossfade animation"
```

---

### Task 7: Palette Crossfade

**Files:**
- Modify: `src/app.rs` (settings_apply triggers animation)
- Modify: `src/ui.rs` (compute effective palette for rendering)

**Step 1: Start palette animation on palette change**

In `src/app.rs`, in `settings_apply()`, where palette is set:

```rust
// Before:
SettingsItem::Palette(idx) => {
    if let Some(p) = Palette::all().into_iter().nth(idx) {
        self.palette = p;
    }
}

// After:
SettingsItem::Palette(idx) => {
    if let Some(p) = Palette::all().into_iter().nth(idx) {
        if p.name != self.palette.name {
            use crate::animation::{TransitionKind, Easing};
            use std::time::Duration;
            self.animations.start(
                TransitionKind::Palette {
                    from: Box::new(self.palette.clone()),
                    to: Box::new(p.clone()),
                },
                Duration::from_millis(300),
                Easing::EaseInOut,
            );
        }
        self.palette = p;
    }
}
```

**Step 2: Add effective_palette helper to App**

In `src/app.rs`:

```rust
/// Returns the effective palette, accounting for any active crossfade animation.
/// During a palette transition, interpolates all colors between old and new palettes.
pub fn effective_palette(&self) -> Palette {
    if let Some((progress, from, _to)) = self.animations.palette_progress() {
        use crate::palette::interpolate;
        Palette {
            name: self.palette.name,
            foreground: interpolate(&from.foreground, &self.palette.foreground, progress),
            background: interpolate(&from.background, &self.palette.background, progress),
            dimmed_foreground: interpolate(&from.dimmed_foreground, &self.palette.dimmed_foreground, progress),
            accent_heading: interpolate(&from.accent_heading, &self.palette.accent_heading, progress),
            accent_emphasis: interpolate(&from.accent_emphasis, &self.palette.accent_emphasis, progress),
            accent_link: interpolate(&from.accent_link, &self.palette.accent_link, progress),
            accent_code: interpolate(&from.accent_code, &self.palette.accent_code, progress),
        }
    } else {
        self.palette.clone()
    }
}
```

**Step 3: Use effective_palette in ui.rs**

In `src/ui.rs::draw()`, compute the effective palette and use it everywhere:

```rust
// At the top of draw(), after the area check:
let palette = app.effective_palette();
```

Then pass `&palette` to WritingSurface instead of `&app.palette`:

```rust
let surface = WritingSurface::new(&app.buffer, &palette)
```

Also use `palette` in `draw_find_bar` and `draw_settings_layer` — pass it as parameter or compute inside. Since settings layer already uses a `preview_palette`, the palette crossfade applies to the WritingSurface only.

**Step 4: Write tests**

In `src/app.rs` tests:

```rust
#[test]
fn effective_palette_returns_current_when_no_animation() {
    let app = App::new();
    let eff = app.effective_palette();
    assert_eq!(eff.name, app.palette.name);
    assert_eq!(eff.foreground, app.palette.foreground);
}

#[test]
fn palette_animation_starts_on_switch() {
    let mut app = App::new();
    app.toggle_settings();
    // Move to Inkwell and apply
    app.settings_cursor = 3; // Inkwell
    app.settings_apply();
    assert!(app.animations.is_active());
    // effective_palette should be mid-transition
    let eff = app.effective_palette();
    // At very early progress, effective should differ from final
    // (this is timing-dependent but the animation just started)
    assert_eq!(eff.name, "Inkwell"); // name is always the target
}
```

**Step 5: Run tests**

Run: `cargo test`
Expected: All tests PASS.

**Step 6: Commit**

```bash
git add src/app.rs src/ui.rs
git commit -m "feat: add palette crossfade animation"
```

---

### Task 8: Overlay Fade

**Files:**
- Modify: `src/app.rs` (toggle_settings, dismiss_settings, Ctrl+F open/close start animation)
- Modify: `src/ui.rs` (apply opacity to overlay colors)

**Step 1: Start overlay animation on open/close**

In `src/app.rs`, in `toggle_settings()`:

```rust
pub fn toggle_settings(&mut self) {
    self.settings_visible = !self.settings_visible;
    self.chrome_visible = self.settings_visible;

    use crate::animation::{TransitionKind, Easing};
    use std::time::Duration;
    self.animations.start(
        TransitionKind::OverlayOpacity { appearing: self.settings_visible },
        Duration::from_millis(150),
        Easing::EaseOut,
    );

    if self.settings_visible {
        let items = SettingsItem::all();
        let target = SettingsItem::Palette(self.active_palette_index());
        self.settings_cursor = items.iter().position(|i| *i == target).unwrap_or(0);
    }
}
```

In `dismiss_settings()`:

```rust
pub fn dismiss_settings(&mut self) {
    self.settings_visible = false;
    self.chrome_visible = false;

    use crate::animation::{TransitionKind, Easing};
    use std::time::Duration;
    self.animations.start(
        TransitionKind::OverlayOpacity { appearing: false },
        Duration::from_millis(150),
        Easing::EaseInOut,
    );
}
```

Note: For dismiss, the overlay is already hidden (`settings_visible = false`), so the fade-out won't render. To make fade-out work, we need to keep rendering the overlay while the fade-out animation is active.

Revised approach for `dismiss_settings`:
```rust
pub fn dismiss_settings(&mut self) {
    // Don't immediately hide — let the fade-out animation render
    use crate::animation::{TransitionKind, Easing};
    use std::time::Duration;
    self.animations.start(
        TransitionKind::OverlayOpacity { appearing: false },
        Duration::from_millis(150),
        Easing::EaseInOut,
    );
    // settings_visible stays true during animation; ui.rs checks overlay_progress
    // to determine actual visibility
    self.chrome_visible = false;
}
```

Actually, this adds complexity. **Simpler: only animate the appear, not the dismiss.** Dismiss is instant. Users expect instant dismiss (they pressed Escape to make it go away). The appear animation gives the "alive" feeling.

**Revised Step 1:** Only animate `toggle_settings` when opening:

```rust
pub fn toggle_settings(&mut self) {
    self.settings_visible = !self.settings_visible;
    self.chrome_visible = self.settings_visible;

    if self.settings_visible {
        use crate::animation::{TransitionKind, Easing};
        use std::time::Duration;
        self.animations.start(
            TransitionKind::OverlayOpacity { appearing: true },
            Duration::from_millis(150),
            Easing::EaseOut,
        );
        let items = SettingsItem::all();
        let target = SettingsItem::Palette(self.active_palette_index());
        self.settings_cursor = items.iter().position(|i| *i == target).unwrap_or(0);
    }
}
```

Same for find bar: animate appear only.

**Step 2: Apply opacity in ui.rs settings overlay**

In `draw_settings_layer()`, query overlay progress:

```rust
let opacity = app.animations.overlay_progress().unwrap_or(1.0);
```

Interpolate the overlay's `normal_style` colors toward the underlying background:

```rust
let effective_fg = crate::palette::interpolate(
    &preview_palette.background, &preview_palette.foreground, opacity
);
let effective_bg = preview_palette.background; // background stays same
let normal_style = Style::default().fg(effective_fg).bg(effective_bg);
```

And the border:
```rust
let effective_dim = crate::palette::interpolate(
    &preview_palette.background, &preview_palette.dimmed_foreground, opacity
);
let block = Block::bordered()
    .title(" Settings ")
    .border_style(Style::default().fg(effective_dim))
    .style(Style::default().bg(effective_bg));
```

Same pattern for `draw_find_bar()`.

**Step 3: Write tests**

```rust
#[test]
fn overlay_animation_starts_on_settings_open() {
    let mut app = App::new();
    app.toggle_settings();
    assert!(app.animations.overlay_progress().is_some());
}

#[test]
fn overlay_no_animation_on_settings_close() {
    let mut app = App::new();
    app.toggle_settings();
    app.animations.tick(); // clear opening animation manually
    app.animations.transitions.clear();
    app.dismiss_settings();
    // No overlay animation on dismiss (instant)
    assert!(app.animations.overlay_progress().is_none());
}
```

**Step 4: Run tests**

Run: `cargo test`
Expected: All tests PASS.

**Step 5: Commit**

```bash
git add src/app.rs src/ui.rs
git commit -m "feat: add overlay fade-in animation for settings and find"
```

---

### Task 9: Find Bar Fade-In

**Files:**
- Modify: `src/main.rs` (start overlay animation when Ctrl+F opens find)
- Modify: `src/ui.rs` (apply opacity to find bar)

**Step 1: Start overlay animation when find opens**

In `src/main.rs`, in the Ctrl+F handler:

```rust
KeyCode::Char('f') => {
    if app.find_state.is_none() {
        app.find_state = Some(zani::find::FindState::new(
            app.cursor_line,
            app.cursor_col,
        ));
        app.animations.start(
            zani::animation::TransitionKind::OverlayOpacity { appearing: true },
            Duration::from_millis(150),
            zani::animation::Easing::EaseOut,
        );
    }
}
```

**Step 2: Apply opacity in draw_find_bar**

In `ui.rs::draw_find_bar()`, add opacity interpolation:

```rust
let opacity = app.animations.overlay_progress().unwrap_or(1.0);
let effective_fg = crate::palette::interpolate(
    &app.palette.background, &app.palette.foreground, opacity
);
let bar_style = Style::default().fg(effective_fg).bg(app.palette.background);
```

**Step 3: Run tests**

Run: `cargo test`
Expected: All tests PASS.

**Step 4: Commit**

```bash
git add src/main.rs src/ui.rs
git commit -m "feat: add find bar fade-in animation"
```

---

### Task 10: Final Polish and Full Test Suite

**Files:**
- Modify: `src/animation.rs` (any cleanup)
- Modify: `src/main.rs` (ensure prev_cursor_line cleanup)

**Step 1: Verify all animations don't conflict**

Multiple animations can be active simultaneously (scroll + focus + overlay). The manager handles them independently. Verify this works:

```rust
#[test]
fn multiple_animation_kinds_coexist() {
    let mut m = AnimationManager::new();
    m.start(
        TransitionKind::Scroll { from: 0.0, to: 10.0 },
        Duration::from_millis(150),
        Easing::EaseOut,
    );
    m.start(
        TransitionKind::FocusDimming { from_line: 0, to_line: 5 },
        Duration::from_millis(150),
        Easing::EaseOut,
    );
    m.start(
        TransitionKind::OverlayOpacity { appearing: true },
        Duration::from_millis(150),
        Easing::EaseOut,
    );
    assert_eq!(m.transitions.len(), 3);
    assert!(m.scroll_progress().is_some());
    assert!(m.focus_progress().is_some());
    assert!(m.overlay_progress().is_some());
}
```

**Step 2: Run full test suite + clippy**

Run: `cargo test && cargo clippy`
Expected: All tests PASS, no new warnings.

**Step 3: Commit any final fixes**

```bash
git add -A
git commit -m "test: add multi-animation coexistence test"
```

---

## File Summary

| File | Tasks |
|------|-------|
| `src/animation.rs` (new) | 1, 2, 3, 10 |
| `src/lib.rs` | 1 |
| `src/app.rs` | 4, 5, 6, 7, 8 |
| `src/main.rs` | 4, 5, 6, 9 |
| `src/ui.rs` | 5, 7, 8, 9 |
| `src/writing_surface.rs` | 6 |

## Dependency Graph

```
Task 1 (easing) → Task 2 (types) → Task 3 (manager) → Task 4 (wiring)
Task 4 → Task 5 (scroll)
Task 4 → Task 6 (focus)
Task 4 → Task 7 (palette)
Task 4 → Task 8 (overlay settings)
Task 8 → Task 9 (overlay find)
All → Task 10 (polish)
```

Tasks 5, 6, 7, 8 are independent of each other (all depend on Task 4).
