# Composable Dimming Redesign

**Date:** 2026-02-27
**ADR:** ADR-008
**Invariants:** 4 (clarified), 12–15 (new)

## Problem

Focus dimming flickers when pressing Enter rapidly in Sentence/Paragraph mode. The animation system stores `(from_line, to_line)` and recomputes distances from both, so interrupting an animation mid-flight causes a visual discontinuity. Additionally, Typewriter bundles scroll behavior with dimming, and there's no way to compose dimming from independent sources.

## Design

### Two orthogonal axes

- **ScrollMode**: Edge | Typewriter — viewport behavior only, zero dimming
- **FocusMode**: Off | Sentence | Paragraph — dimming behavior only

### Composable dimming layers

Each layer produces a per-line opacity (f64, 0.0–1.0). Layers compose by multiplication.

**Paragraph layer**: active paragraph = 1.0, tiered by distance (0.6 / 0.35 / 0.2).

**Sentence layer**: per-character mask — active sentence = 1.0, everything else = 0.6.

Focus Mode controls which layers are active:
- Off: no layers
- Paragraph: paragraph layer
- Sentence: paragraph layer + sentence layer

### Per-line animated opacity

Each layer tracks per logical line:

```rust
struct LineOpacity {
    current: f64,
    target: f64,
    start_value: f64,
    start_time: Instant,
    fade_config: FadeConfig,
}

struct FadeConfig {
    duration: Duration,
    easing: Easing,
}
```

When target changes, `start_value` = current visual state. Animation chases from there. Interruption-safe by construction.

Each layer specifies its own FadeConfig for fade-in and fade-out independently.

### Rendering pipeline

1. Active layers compute target opacities per line
2. Each layer's LineOpacity chases its target independently
3. Final opacity = product of all active layers
4. For Sentence mode: multiply by per-character sentence mask
5. Resolve markdown style → base foreground color
6. `final_fg = interpolate(base_fg, background, 1.0 - opacity)`
7. Map through color profile
8. Selection/find highlighting overrides on top

### What changes

| Component | Before | After |
|-----------|--------|-------|
| `FocusMode` enum | Off, Sentence, Paragraph, Typewriter | Off, Sentence, Paragraph |
| New `ScrollMode` enum | (didn't exist) | Edge, Typewriter |
| `line_distance()` | returns `usize` | removed; layers return `f64` opacity |
| `apply_dimming()` | takes `distance: usize` | takes `opacity: f64` |
| `TransitionKind::FocusDimming` | `{from_line, to_line}` | removed; each layer animates internally |
| Animation state | global `(progress, from, to)` | per-line `LineOpacity` per layer |
| WritingSurface | receives `focus_anim` param | receives `Vec<f64>` line opacities |
| App | `focus_mode: FocusMode` | `focus_mode: FocusMode` + `scroll_mode: ScrollMode` |
| Settings | Typewriter in focus list | Typewriter in separate scroll list |

### What stays the same

- Color interpolation math (`palette::interpolate`)
- Markdown styling pipeline
- Color profile detection and mapping
- Selection/find highlighting
- Scroll animation (separate `TransitionKind::Scroll`)
- Palette crossfade animation
- Overlay animations
- Sentence boundary parsing (`sentence_bounds_at`)
- Paragraph boundary detection
