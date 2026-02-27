# Animation & Easing System

*2026-02-26*

## Goal

Make every visual state change in Zani interpolated — no hard snaps. Cursor movement glides the viewport, focus dimming crossfades between lines, palette switches blend smoothly, and overlays fade in/out.

## Architecture: Centralized AnimationManager

A single `AnimationManager` struct in `App` tracks all active transitions. The event loop queries it to decide poll timeout (16ms during animation, 250ms idle). At render time, draw code queries the manager for current interpolation progress.

No external dependencies. Easing math is two cubic functions.

### Core Types (`src/animation.rs`)

**Easing** — two curves:
- `EaseOut`: `1 - (1-t)^3` — fast start, gentle landing. For cursor/focus/scroll (~150ms).
- `EaseInOut`: `3t^2 - 2t^3` — smooth acceleration/deceleration. For palette/overlay (~300ms).

**Transition** — one active animation:
- `start: Instant` — when it began
- `duration: Duration` — target length
- `easing: Easing` — which curve
- `kind: TransitionKind` — what's animated

**TransitionKind** — enum:
- `FocusDimming { from_line, to_line }` — crossfade focus between cursor lines
- `Scroll { from_offset: f64, to_offset: f64 }` — smooth viewport scroll
- `Palette { from: Palette, to: Palette }` — crossfade all palette colors
- `OverlayOpacity { appearing: bool }` — settings/find bar fade

**AnimationManager** — holds `Vec<Transition>`:
- `start(kind, duration, easing)` — add transition, replaces existing of same kind
- `progress(kind) -> Option<f64>` — eased progress 0.0–1.0, None if not animating
- `is_active() -> bool` — any transitions running?
- `tick()` — prune completed transitions

New transitions of the same kind replace old ones, picking up from current interpolated state.

## Event Loop

Poll timeout becomes dynamic:

```
if app.animations.is_active() { 16ms } else { 250ms }
```

After each draw, `animations.tick()` prunes completed transitions. No threads, no async.

## Animated Properties

### Smooth Scroll

Change `scroll_offset` from `usize` to `f64`. When `ensure_cursor_visible()` needs to scroll, start a `Scroll` transition instead of instant assignment. WritingSurface renders fractional offset by shifting the first visible line's y position.

Duration: ~150ms, ease-out.

### Focus Dimming Crossfade

When cursor moves to a new line, start a `FocusDimming` transition. During animation, each line's focus distance is blended between distance-from-old-cursor and distance-from-new-cursor using eased progress. At t=0 the old line is bright; at t=1 the new line is bright.

Duration: ~150ms, ease-out.

### Palette Crossfade

When palette changes in settings, store old palette and start a `Palette` transition. Every color access during render interpolates: `palette::interpolate(old.color, new.color, t)`. At completion, drop old palette reference.

Duration: ~300ms, ease-in-out.

### Overlay Fade

Settings overlay and find bar get an `OverlayOpacity` transition on open/close. Colors interpolate from transparent (matching underlying content) to final values on appear, reversed on dismiss.

Duration: ~150ms, ease-out for appear, ease-in-out for dismiss.

## Existing Infrastructure

- `palette::interpolate(color1, color2, t: f64)` — already implemented
- WritingSurface recomputes all cell styles each frame (immediate mode)
- All colors stored as `Color::Rgb(u8, u8, u8)` — smooth interpolation with final u8 rounding
- `Instant`-based timing already used for autosave

## Constraints

- Ratatui is immediate-mode: animation state lives in `App`, not on widgets
- u8 rounding is imperceptible for color transitions (1-unit RGB steps)
- 16ms poll during animation means ~60fps; at typical input rate, often faster
- Animations must handle interruption gracefully (new transition replaces old from current state)
