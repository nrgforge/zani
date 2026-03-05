# ADR-004: Focus Dimming via Per-Character Color Interpolation

**Status:** Accepted (clarified 2026-02-27: see ADR-008 for animation and composition model)

## Context

Focus Mode dims text outside the Active Region to keep the writer in generative mode. Invariant 4 states: "Focus dimming is color interpolation, not opacity. Dimmed text uses per-character RGB interpolation toward the background color."

Terminals do not support true per-character opacity. The ANSI `dim` attribute exists but is coarse (one level, terminal-defined appearance). True Color (24-bit RGB) is supported by virtually all modern terminals except macOS Terminal.app.

## Decision

Dimming is implemented as per-character foreground color interpolation toward the background color. The Writing Surface calculates each character's foreground color based on its opacity level (set per-line by the active Focus Mode).

Example interpolation for the default Palette:
- Active text: `rgb(220, 215, 205)` — full brightness, warm off-white
- 1 paragraph away: `rgb(150, 146, 139)` — ~60% toward background
- 2+ paragraphs away: `rgb(100, 97, 92)` — ~30% brightness
- Background: `rgb(40, 38, 35)` — warm dark gray

The interpolation curve, number of gradient steps, and scope (sentence or paragraph) vary by the active Focus Mode variant.

## Consequences

**Positive:**
- Smooth, multi-level dimming with precise control over the gradient.
- Works on any terminal with True Color support — no special features needed.
- Same mechanism is reused for Markdown Styling (dimming syntax characters).
- The gradient is continuous, not a binary dim/bright toggle.

**Negative:**
- Requires True Color. On 256-color terminals (macOS Terminal.app), the gradient is approximated with fewer steps. On basic ANSI, dimming falls back to the `dim` attribute (per Invariant 11).
- Every visible character's color must be calculated each render frame. This is arithmetic, not I/O, so it should be fast — but it's worth profiling.

**Neutral:**
- The specific RGB values above are starting points. The Palette defines the actual colors; the interpolation math is independent of the specific values.
