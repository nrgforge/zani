# ADR-008: Composable Dimming with Scroll/Focus Separation

**Status:** Accepted

## Context

Focus Mode previously bundled three concerns into one enum: sentence dimming, paragraph dimming, and typewriter scroll+dimming. A "gradient" concept (vertical distance-based dimming) was entangled with focus mode selection, making behaviors non-composable.

The animation system stored `(from_line, to_line)` and recomputed distances from both endpoints to blend. When the user pressed Enter rapidly, each new animation recomputed a "from" state that didn't match the current visual state, causing the old text to briefly brighten before dimming again (flicker).

## Decision

### 1. Separate Scroll Mode from Focus Mode (Invariants 12, 13)

Two orthogonal axes replace the single `FocusMode` enum:

- **ScrollMode**: Edge (viewport adjusts at edges) | Typewriter (cursor stays centered). Purely viewport behavior. Zero dimming contribution.
- **FocusMode**: Off | Sentence | Paragraph. Purely dimming behavior.

Typewriter is removed from `FocusMode` and becomes a `ScrollMode` variant.

### 2. Opacity-based dimming via composable layers (Invariants 4, 15)

Each character's dimming is expressed as an `f64` opacity factor in `[0.0, 1.0]`:
- 1.0 = full brightness (active region)
- 0.0 = fully background-colored

Dimming is built from independent layers that compose by multiplication:

**Paragraph layer** — per-line opacity based on distance from the active paragraph:
- Active paragraph: 1.0
- 1–3 lines away: 0.6
- 4–6 lines away: 0.35
- 7+ lines away: 0.2

**Sentence layer** — per-character opacity within the document:
- Active sentence chars: 1.0
- Everything else: 0.6

The Focus Mode setting controls which layers are active:
- **Off**: no layers (all text at 1.0)
- **Paragraph**: paragraph layer only
- **Sentence**: paragraph layer + sentence layer

Sentence mode always includes paragraph dimming. This means surrounding paragraphs fade by distance even when the writer is focused at the sentence level. The final per-character opacity is the product of all active layers.

The renderer applies the final opacity:

```
final_fg = interpolate(base_fg, background, 1.0 - opacity)
```

No integer distance abstraction. The layers directly produce the values the renderer needs.

### 3. Chase-based animation (Invariant 14)

Each layer tracks animated opacity per logical line (or per character range for the sentence layer):

```rust
struct LineOpacity {
    current: f64,          // what's on screen right now
    target: f64,           // what the layer says it should be
    start_value: f64,      // current at the moment target last changed
    start_time: Instant,
    fade_config: FadeConfig,
}
```

When the target changes, the system records `start_value = current` and `start_time = now`, then picks the appropriate FadeConfig based on direction (brightening or dimming). Each frame, it interpolates from `start_value` toward `target` using `elapsed / duration` plus the easing curve.

If interrupted (target changes again mid-transition), the new animation starts from wherever `current` actually is. No "from" state is recomputed. This guarantees no visual discontinuity.

Each layer independently manages its own animation state and FadeConfig. The paragraph layer might fade out over 1800ms while the sentence layer transitions in 150ms.

### 4. Configurable fade timing (Fade Config)

Each dimming source specifies separate fade configs for brightening and dimming:

```rust
struct FadeConfig {
    duration: Duration,
    easing: Easing,
}
```

Example: fade-in 150ms EaseOut, fade-out 1800ms EaseOut. The system picks the appropriate config based on whether the character is brightening or dimming.

### 5. Rendering pipeline

```
1. Each active layer computes target_opacity per line (or per char range)
2. Each layer's LineOpacity tracker chases target independently
3. Final opacity = product of all active layers' animated opacities
4. Resolve markdown style → base_fg color
5. Apply dimming: final_fg = interpolate(base_fg, background, 1.0 - final_opacity)
6. Map through color profile (TrueColor passthrough, 256 nearest, Basic DIM)
7. Apply selection/find highlighting on top (overrides dimming)
```

For Basic color profile (Invariant 11): if `final_opacity < 1.0`, apply the DIM modifier.

## Consequences

**Positive:**
- Flicker on rapid Enter is eliminated by construction (Invariant 14).
- Scroll and focus are independently selectable — any combination works.
- New dimming sources compose naturally via multiplication.
- Fade-in and fade-out durations are independently tunable per source.
- The rendering pipeline is simpler: opacity in, color out. No distance-to-factor lookup.

**Negative:**
- Requires tracking per-line (or per-character-range) animated opacity state. More state than the previous stateless distance calculation.
- Existing tests for `line_distance` and integer-distance-based blending need rewriting.

**Neutral:**
- The `apply_dimming` function signature changes from `(color, palette, distance: usize)` to `(color, palette, opacity: f64)`. Same interpolation math, different input domain.
- ADR-004's rendering mechanism (color interpolation, not terminal opacity) is unchanged. This ADR governs the abstraction above it.
