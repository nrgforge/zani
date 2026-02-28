# Composable Dimming Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the integer-distance dimming system with opacity-based composable dimming layers, separate scroll mode from focus mode, and eliminate the animation flicker bug.

**Architecture:** Two orthogonal axes (ScrollMode for viewport, FocusMode for dimming). Dimming expressed as f64 opacity per line, animated via chase-from-current-state. Layers compose by multiplication. See ADR-008.

**Tech Stack:** Rust, ratatui, crossterm. No new dependencies.

---

### Task 1: Add ScrollMode enum

**Files:**
- Create: `src/scroll_mode.rs`
- Modify: `src/lib.rs`

**Step 1: Write the failing test**

In `src/scroll_mode.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScrollMode {
    #[default]
    Edge,
    Typewriter,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_edge() {
        assert_eq!(ScrollMode::default(), ScrollMode::Edge);
    }

    #[test]
    fn scroll_mode_cycles() {
        assert_eq!(ScrollMode::Edge.next(), ScrollMode::Typewriter);
        assert_eq!(ScrollMode::Typewriter.next(), ScrollMode::Edge);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib scroll_mode`
Expected: FAIL — `next()` method not implemented yet.

**Step 3: Write minimal implementation**

Add the `next()` method:

```rust
impl ScrollMode {
    pub fn next(self) -> Self {
        match self {
            Self::Edge => Self::Typewriter,
            Self::Typewriter => Self::Edge,
        }
    }
}
```

Add to `src/lib.rs`:

```rust
pub mod scroll_mode;
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib scroll_mode`
Expected: PASS

**Step 5: Commit**

```bash
git add src/scroll_mode.rs src/lib.rs
git commit -m "feat: add ScrollMode enum (Edge, Typewriter)"
```

---

### Task 2: Add FadeConfig and LineOpacity types

**Files:**
- Modify: `src/focus_mode.rs`

**Step 1: Write the failing test**

Add at the bottom of `src/focus_mode.rs` (inside the `tests` module):

```rust
#[test]
fn fade_config_default_values() {
    let config = FadeConfig::default();
    assert!(config.duration.as_millis() > 0);
}

#[test]
fn line_opacity_at_target_returns_target() {
    let lo = LineOpacity::new(0.6);
    // A freshly created LineOpacity is already at its target
    assert!((lo.current_opacity() - 0.6).abs() < f64::EPSILON);
}

#[test]
fn line_opacity_chases_target() {
    use std::time::{Duration, Instant};
    let mut lo = LineOpacity::new(0.0);
    let config = FadeConfig { duration: Duration::from_millis(100), easing: crate::animation::Easing::EaseOut };
    lo.set_target(1.0, config);
    // Immediately after setting target, current is still 0.0
    // but after enough time passes, it should reach 1.0
    lo.start_time = Some(Instant::now() - Duration::from_millis(200));
    assert!((lo.current_opacity() - 1.0).abs() < f64::EPSILON);
}

#[test]
fn line_opacity_interruption_starts_from_current() {
    use std::time::{Duration, Instant};
    let mut lo = LineOpacity::new(0.0);
    let config = FadeConfig { duration: Duration::from_millis(1000), easing: crate::animation::Easing::EaseOut };
    lo.set_target(1.0, config.clone());
    // Simulate being halfway through (set start_time in the past)
    lo.start_time = Some(Instant::now() - Duration::from_millis(500));
    let mid_opacity = lo.current_opacity();
    assert!(mid_opacity > 0.0 && mid_opacity < 1.0, "Should be mid-fade: {}", mid_opacity);

    // Now interrupt: change target back to 0.0
    lo.set_target(0.0, config);
    // start_value should be captured from current visual state
    assert!((lo.start_value - mid_opacity).abs() < 0.01, "start_value should capture current: {} vs {}", lo.start_value, mid_opacity);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib focus_mode::tests::fade_config`
Expected: FAIL — types don't exist yet.

**Step 3: Write minimal implementation**

Add to `src/focus_mode.rs` (above the tests module):

```rust
use std::time::{Duration, Instant};
use crate::animation::Easing;

/// Configuration for how a dimming transition animates.
#[derive(Debug, Clone)]
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

/// Animated opacity for a single logical line within a dimming layer.
/// Chases `target` from `start_value` over the configured duration.
/// Interruption-safe: calling `set_target` captures the current visual
/// state as the new start_value (Invariant 14).
#[derive(Debug, Clone)]
pub struct LineOpacity {
    pub target: f64,
    pub start_value: f64,
    pub start_time: Option<Instant>,
    fade_config: FadeConfig,
}

impl LineOpacity {
    /// Create a new LineOpacity already at the given value (no animation).
    pub fn new(value: f64) -> Self {
        Self {
            target: value,
            start_value: value,
            start_time: None,
            fade_config: FadeConfig::default(),
        }
    }

    /// Set a new target opacity. Captures the current visual state as
    /// the animation start point. No discontinuity possible.
    pub fn set_target(&mut self, new_target: f64, config: FadeConfig) {
        if (new_target - self.target).abs() < f64::EPSILON {
            return;
        }
        self.start_value = self.current_opacity();
        self.target = new_target;
        self.fade_config = config;
        self.start_time = Some(Instant::now());
    }

    /// The current visual opacity, accounting for animation progress.
    pub fn current_opacity(&self) -> f64 {
        let Some(start) = self.start_time else {
            return self.target; // No animation in flight
        };
        let elapsed = start.elapsed().as_secs_f64();
        let total = self.fade_config.duration.as_secs_f64();
        if total <= 0.0 || elapsed >= total {
            return self.target;
        }
        let t = self.fade_config.easing.apply(elapsed / total);
        self.start_value + (self.target - self.start_value) * t
    }

    /// Whether this line's animation is still in flight.
    pub fn is_animating(&self) -> bool {
        self.start_time
            .map(|s| s.elapsed() < self.fade_config.duration)
            .unwrap_or(false)
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --lib focus_mode`
Expected: ALL PASS

**Step 5: Commit**

```bash
git add src/focus_mode.rs
git commit -m "feat: add FadeConfig and LineOpacity types for chase-based dimming"
```

---

### Task 3: Add DimLayer to manage per-line animated opacities

**Files:**
- Modify: `src/focus_mode.rs`

**Step 1: Write the failing test**

Add to the `tests` module in `src/focus_mode.rs`:

```rust
#[test]
fn dim_layer_computes_paragraph_opacities() {
    let mut layer = DimLayer::new(FadeConfig::default(), FadeConfig::default());
    // 5 lines, paragraph spans lines 1-3, cursor on line 2
    let targets = paragraph_target_opacities(5, Some((1, 3)));
    layer.update_targets(&targets);
    assert!((layer.opacity(0) - 0.6).abs() < 0.01, "Line 0 should be dimmed");
    assert!((layer.opacity(1) - 1.0).abs() < f64::EPSILON, "Line 1 in paragraph");
    assert!((layer.opacity(2) - 1.0).abs() < f64::EPSILON, "Line 2 in paragraph");
    assert!((layer.opacity(3) - 1.0).abs() < f64::EPSILON, "Line 3 in paragraph");
    assert!((layer.opacity(4) - 0.6).abs() < 0.01, "Line 4 should be dimmed");
}

#[test]
fn dim_layer_is_animating_after_target_change() {
    let mut layer = DimLayer::new(FadeConfig::default(), FadeConfig::default());
    let targets_a = vec![1.0, 1.0, 0.6];
    layer.update_targets(&targets_a);
    // Initially at target — not animating
    assert!(!layer.is_animating());

    // Change targets
    let targets_b = vec![0.6, 1.0, 1.0];
    layer.update_targets(&targets_b);
    assert!(layer.is_animating());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib focus_mode::tests::dim_layer`
Expected: FAIL — `DimLayer` and `paragraph_target_opacities` don't exist.

**Step 3: Write minimal implementation**

Add to `src/focus_mode.rs`:

```rust
/// Compute target opacities for the paragraph dimming layer.
/// `line_count` is the total number of logical lines.
/// `paragraph_bounds` is (start_line, end_line) inclusive.
pub fn paragraph_target_opacities(
    line_count: usize,
    paragraph_bounds: Option<(usize, usize)>,
) -> Vec<f64> {
    let mut targets = vec![0.2; line_count];
    let Some((para_start, para_end)) = paragraph_bounds else {
        // No paragraph bounds — everything at 1.0 (no dimming)
        return vec![1.0; line_count];
    };
    for i in para_start..=para_end.min(line_count.saturating_sub(1)) {
        targets[i] = 1.0;
    }
    // Tier by distance from paragraph bounds
    for i in 0..line_count {
        if i >= para_start && i <= para_end {
            continue;
        }
        let dist = if i < para_start {
            para_start - i
        } else {
            i - para_end
        };
        targets[i] = match dist {
            1..=3 => 0.6,
            4..=6 => 0.35,
            _ => 0.2,
        };
    }
    targets
}

/// A dimming layer that manages animated per-line opacities.
/// Each layer independently tracks and animates its lines.
#[derive(Debug, Clone)]
pub struct DimLayer {
    lines: Vec<LineOpacity>,
    fade_in: FadeConfig,
    fade_out: FadeConfig,
}

impl DimLayer {
    pub fn new(fade_in: FadeConfig, fade_out: FadeConfig) -> Self {
        Self {
            lines: Vec::new(),
            fade_in,
            fade_out,
        }
    }

    /// Update target opacities for all lines. Resizes internal state if needed.
    pub fn update_targets(&mut self, targets: &[f64]) {
        // Grow/shrink to match line count
        while self.lines.len() < targets.len() {
            self.lines.push(LineOpacity::new(targets[self.lines.len()]));
        }
        self.lines.truncate(targets.len());

        for (line, &target) in self.lines.iter_mut().zip(targets.iter()) {
            if (line.target - target).abs() > f64::EPSILON {
                let config = if target > line.current_opacity() {
                    self.fade_in.clone()
                } else {
                    self.fade_out.clone()
                };
                line.set_target(target, config);
            }
        }
    }

    /// Get the current animated opacity for a logical line.
    pub fn opacity(&self, line: usize) -> f64 {
        self.lines.get(line).map(|l| l.current_opacity()).unwrap_or(1.0)
    }

    /// Whether any line in this layer is still animating.
    pub fn is_animating(&self) -> bool {
        self.lines.iter().any(|l| l.is_animating())
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --lib focus_mode`
Expected: ALL PASS

**Step 5: Commit**

```bash
git add src/focus_mode.rs
git commit -m "feat: add DimLayer with paragraph target opacity computation"
```

---

### Task 4: Change apply_dimming to accept f64 opacity instead of usize distance

**Files:**
- Modify: `src/focus_mode.rs`

This is a structure change. We rename the old function and add the new signature. No callers change yet.

**Step 1: Write the failing test**

Add to `tests` in `src/focus_mode.rs`:

```rust
#[test]
fn apply_dimming_opacity_one_returns_base_color() {
    let palette = Palette::default_palette();
    let color = apply_dimming_with_opacity(&palette.foreground, &palette, 1.0);
    assert_eq!(color, palette.foreground);
}

#[test]
fn apply_dimming_opacity_zero_returns_background() {
    let palette = Palette::default_palette();
    let color = apply_dimming_with_opacity(&palette.foreground, &palette, 0.0);
    assert_eq!(color, palette.background);
}

#[test]
fn apply_dimming_opacity_half_is_midpoint() {
    use ratatui::style::Color;
    let palette = Palette::default_palette();
    let color = apply_dimming_with_opacity(&palette.foreground, &palette, 0.5);
    // Should be roughly halfway between foreground and background
    if let (Color::Rgb(fr, fg, fb), Color::Rgb(br, bg, bb), Color::Rgb(mr, mg, mb)) =
        (palette.foreground, palette.background, color)
    {
        let expected_r = ((fr as f64 + br as f64) / 2.0).round() as u8;
        assert!((mr as i16 - expected_r as i16).unsigned_abs() <= 1);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib focus_mode::tests::apply_dimming_opacity`
Expected: FAIL — function doesn't exist.

**Step 3: Write minimal implementation**

Add to `src/focus_mode.rs`:

```rust
/// Apply dimming to a foreground color using an opacity factor (0.0–1.0).
/// opacity=1.0 returns base_fg unchanged. opacity=0.0 returns background.
/// Intermediate values interpolate linearly.
pub fn apply_dimming_with_opacity(base_fg: &Color, palette: &Palette, opacity: f64) -> Color {
    if opacity >= 1.0 {
        return *base_fg;
    }
    if opacity <= 0.0 {
        return palette.background;
    }
    palette::interpolate(base_fg, &palette.background, 1.0 - opacity)
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --lib focus_mode`
Expected: ALL PASS

**Step 5: Commit**

```bash
git add src/focus_mode.rs
git commit -m "feat: add apply_dimming_with_opacity for f64-based dimming"
```

---

### Task 5: Remove Typewriter from FocusMode enum

**Files:**
- Modify: `src/focus_mode.rs`
- Modify: `src/config.rs`
- Modify: `src/app.rs` (lines ~46, ~1013)
- Modify: `src/ui.rs` (lines ~190-195)
- Modify: `src/main.rs` (lines ~155-166)

This is a structure change with wide impact. We remove `FocusMode::Typewriter` and update all references. The scroll behavior that was inside `ensure_cursor_visible` stays — it just uses the new `ScrollMode` instead.

**Step 1: Remove Typewriter from FocusMode**

In `src/focus_mode.rs`, remove the `Typewriter` variant and update `next()`:

```rust
pub enum FocusMode {
    #[default]
    Off,
    Sentence,
    Paragraph,
}

impl FocusMode {
    pub fn next(self) -> Self {
        match self {
            Self::Off => Self::Sentence,
            Self::Sentence => Self::Paragraph,
            Self::Paragraph => Self::Off,
        }
    }
}
```

Remove the `FocusMode::Typewriter` arm from `line_distance()`. Remove the Typewriter test cases.

**Step 2: Update config.rs**

In `focus_mode_serde`, remove the `"typewriter"` serialization/deserialization arms. Map `"typewriter"` in existing configs to `FocusMode::Off` for backwards compatibility.

Add `scroll_mode` field to `Config`:

```rust
use crate::scroll_mode::ScrollMode;

pub struct Config {
    // ... existing fields ...
    #[serde(default, with = "scroll_mode_serde")]
    pub scroll_mode: ScrollMode,
}
```

Add `scroll_mode_serde` module (same pattern as `focus_mode_serde`).

**Step 3: Update app.rs**

Add `scroll_mode: ScrollMode` field to `App`. In `ensure_cursor_visible`, change `self.focus_mode == FocusMode::Typewriter` to `self.scroll_mode == ScrollMode::Typewriter`.

Remove `FocusMode::Typewriter` from `SettingsItem::all()`. Add `SettingsItem::ScrollMode(ScrollMode)` variants.

**Step 4: Update ui.rs**

Remove the `FocusMode::Typewriter` arm in settings rendering. Add `SettingsItem::ScrollMode` rendering.

**Step 5: Update main.rs**

Remove the `TransitionKind::FocusDimming` animation trigger (lines 155-166). Load `scroll_mode` from config. Set `app.scroll_mode`.

**Step 6: Run full test suite**

Run: `cargo test`
Expected: ALL PASS (some old tests will need updating — Typewriter-specific focus tests become ScrollMode tests or get removed).

**Step 7: Commit**

```bash
git add src/focus_mode.rs src/scroll_mode.rs src/config.rs src/app.rs src/ui.rs src/main.rs src/lib.rs
git commit -m "refactor: separate ScrollMode from FocusMode, remove Typewriter dimming"
```

---

### Task 6: Remove old distance-based dimming from WritingSurface

**Files:**
- Modify: `src/writing_surface.rs`
- Modify: `src/animation.rs`

This is a structure change: remove the old `line_distance`, `focus_anim`, and `TransitionKind::FocusDimming` code.

**Step 1: Remove from animation.rs**

Remove `TransitionKind::FocusDimming` variant. Remove `focus_progress()` from `AnimationManager`.

**Step 2: Remove from writing_surface.rs**

Remove the `focus_anim` field from `WritingSurface`. Remove `focus_animation()` builder method. Remove the `line_distance` blending logic from the render method (lines 252-270). Remove the old `distance` computation in the per-character loop (lines 283-289).

Replace with a new field: `line_opacities: &'a [f64]` — the pre-computed animated opacities passed in from App.

**Step 3: Update render to use opacity**

The per-character rendering loop becomes:

```rust
let line_opacity = self.line_opacities
    .get(vl.logical_line)
    .copied()
    .unwrap_or(1.0);

// Per-character sentence mask (if sentence mode)
let char_opacity = if use_sentence_dimming {
    let abs_idx = abs_line_start + char_idx;
    let (s_start, s_end) = self.sentence_bounds.unwrap();
    let sentence_mask = if abs_idx >= s_start && abs_idx < s_end { 1.0 } else { 0.6 };
    line_opacity * sentence_mask
} else {
    line_opacity
};

// Apply dimming
if char_opacity < 1.0 {
    match self.color_profile {
        ColorProfile::Basic => resolved.add_modifier(Modifier::DIM),
        _ => {
            let base_fg = resolved.fg.unwrap_or(self.palette.foreground);
            let dimmed = focus_mode::apply_dimming_with_opacity(&base_fg, self.palette, char_opacity);
            resolved.fg(self.color_profile.map_color(dimmed))
        }
    }
} else {
    let fg = resolved.fg.unwrap_or(self.palette.foreground);
    resolved.fg(self.color_profile.map_color(fg))
}
```

**Step 4: Update tests**

Update existing WritingSurface tests to pass `line_opacities` instead of `focus_anim`. The `focus_animation_blends_distances` test becomes an opacity-based test.

**Step 5: Run full test suite**

Run: `cargo test`
Expected: ALL PASS

**Step 6: Commit**

```bash
git add src/writing_surface.rs src/animation.rs
git commit -m "refactor: replace distance-based dimming with opacity-based rendering"
```

---

### Task 7: Wire DimLayer into App and the render pipeline

**Files:**
- Modify: `src/app.rs`
- Modify: `src/ui.rs`
- Modify: `src/main.rs`

**Step 1: Add DimLayer fields to App**

In `src/app.rs`, add:

```rust
use crate::focus_mode::{DimLayer, FadeConfig};

pub struct App {
    // ... existing fields ...
    pub paragraph_dim: DimLayer,
    pub sentence_dim: DimLayer,
}
```

Initialize in `App::new()` with appropriate FadeConfigs:
- Paragraph layer: fade-in 150ms EaseOut, fade-out 1800ms EaseOut
- Sentence layer: fade-in 150ms EaseOut, fade-out 150ms EaseOut

**Step 2: Add update method to App**

```rust
impl App {
    pub fn update_dim_layers(&mut self) {
        let line_count = self.buffer.len_lines();
        match self.focus_mode {
            FocusMode::Off => {
                // All lines at 1.0
                let targets = vec![1.0; line_count];
                self.paragraph_dim.update_targets(&targets);
                self.sentence_dim.update_targets(&targets);
            }
            FocusMode::Paragraph => {
                let targets = focus_mode::paragraph_target_opacities(
                    line_count,
                    self.paragraph_bounds(),
                );
                self.paragraph_dim.update_targets(&targets);
                self.sentence_dim.update_targets(&vec![1.0; line_count]);
            }
            FocusMode::Sentence => {
                let targets = focus_mode::paragraph_target_opacities(
                    line_count,
                    self.paragraph_bounds(),
                );
                self.paragraph_dim.update_targets(&targets);
                // Sentence layer targets are all 1.0 at line level;
                // per-character masking handles sentence boundaries in the renderer
                self.sentence_dim.update_targets(&vec![1.0; line_count]);
            }
        }
    }

    /// Compute final per-line opacities for the renderer.
    pub fn line_opacities(&self) -> Vec<f64> {
        let line_count = self.buffer.len_lines();
        (0..line_count)
            .map(|i| self.paragraph_dim.opacity(i) * self.sentence_dim.opacity(i))
            .collect()
    }

    /// Whether any dim layer is still animating.
    pub fn dim_animating(&self) -> bool {
        self.paragraph_dim.is_animating() || self.sentence_dim.is_animating()
    }
}
```

**Step 3: Call update_dim_layers from the event loop**

In `src/main.rs`, after `handle_key`:

```rust
app.update_dim_layers();
```

Update the `is_active` check for poll timeout:

```rust
let poll_timeout = if app.animations.is_active() || app.dim_animating() {
    Duration::from_millis(16)
} else {
    Duration::from_millis(250)
};
```

Remove the old `FocusDimming` animation trigger entirely (the DimLayer handles it internally).

**Step 4: Pass opacities to WritingSurface in ui.rs**

```rust
let line_opacities = app.line_opacities();
let surface = WritingSurface::new(&app.buffer, &effective)
    // ... existing builder calls ...
    .line_opacities(&line_opacities);
    // Remove: .focus_animation(...)
```

**Step 5: Run full test suite**

Run: `cargo test`
Expected: ALL PASS

**Step 6: Commit**

```bash
git add src/app.rs src/ui.rs src/main.rs
git commit -m "feat: wire DimLayer into App and render pipeline"
```

---

### Task 8: Remove old distance-based functions

**Files:**
- Modify: `src/focus_mode.rs`

Structure cleanup: remove `line_distance()`, `apply_dimming()`, `dim_color()`, and their tests now that nothing calls them.

**Step 1: Remove dead code**

Remove:
- `apply_dimming()` (replaced by `apply_dimming_with_opacity()`)
- `dim_color()` (convenience wrapper for the old function)
- `line_distance()` (replaced by `paragraph_target_opacities()` and `DimLayer`)
- All tests that reference the removed functions

**Step 2: Run full test suite**

Run: `cargo test`
Expected: ALL PASS

**Step 3: Commit**

```bash
git add src/focus_mode.rs
git commit -m "refactor: remove deprecated distance-based dimming functions"
```

---

### Task 9: Update config serialization for backward compatibility

**Files:**
- Modify: `src/config.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn legacy_typewriter_focus_mode_maps_to_off() {
    let toml_str = r#"focus_mode = "typewriter""#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(config.focus_mode, FocusMode::Off);
}

#[test]
fn scroll_mode_round_trip() {
    let config = Config {
        scroll_mode: ScrollMode::Typewriter,
        ..Config::default()
    };
    let toml_str = toml::to_string_pretty(&config).unwrap();
    let loaded: Config = toml::from_str(&toml_str).unwrap();
    assert_eq!(loaded.scroll_mode, ScrollMode::Typewriter);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib config`
Expected: FAIL

**Step 3: Implement**

Update `focus_mode_serde::deserialize` to map `"typewriter"` to `FocusMode::Off`. Add `scroll_mode_serde` module. Update the `Config` struct and `Default` impl.

**Step 4: Run tests**

Run: `cargo test --lib config`
Expected: ALL PASS

**Step 5: Commit**

```bash
git add src/config.rs
git commit -m "feat: add scroll_mode to config, map legacy typewriter to Off"
```

---

### Task 10: Final polish and cleanup

**Files:**
- All modified files

**Step 1: Run full test suite**

Run: `cargo test`
Expected: ALL PASS

**Step 2: Run clippy**

Run: `cargo clippy -- -D warnings`
Expected: No warnings

**Step 3: Manual smoke test**

Run: `cargo run`

Verify:
- Focus Off: all text bright, no dimming
- Focus Paragraph: active paragraph bright, others fade with distance
- Focus Sentence: active sentence bright, same paragraph slightly dimmed, other paragraphs fade with distance
- Rapid Enter in Sentence mode: no flicker — old text smoothly fades out
- Settings layer shows Focus (Off/Sentence/Paragraph) and Scroll (Edge/Typewriter) as separate sections
- Typewriter scroll centers cursor, Edge scroll follows at edges
- Both scroll modes work with all focus modes

**Step 4: Commit any final fixes**

```bash
git add -A
git commit -m "chore: final polish for composable dimming redesign"
```
