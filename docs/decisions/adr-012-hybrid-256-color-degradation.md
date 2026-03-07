# ADR-012: Hybrid 256-Color Degradation with Hand-Tuned Overrides

**Status:** Proposed

## Context

Essay 002's quantization spike found that dark backgrounds lose hue identity under 256-color mapping — the 6x6x6 cube's 0–95 gap maps all dark backgrounds with per-channel values below ~48 to the grayscale ramp. Ember's warm brown and Inkwell's cool navy become the same neutral gray. High-saturation accents survive well (max ~10 degree hue shift), but low-saturation accents collapse to gray.

The automatic `nearest_256_color` mapping (per Invariant 11) preserves contrast but erases mood character for muted palettes. The domain model specifies that a Palette may include hand-tuned 256-color alternate values for graceful degradation.

## Decision

For the curated Palette collection, key palettes receive hand-tuned 256-color alternate color values that maximize mood preservation within the 6x6x6 cube. The hand-tuned values:

- Select accent colors near cube vertices to minimize hue drift
- Slightly increase saturation for muted palettes so hue character survives quantization
- Accept that dark backgrounds will be neutral gray; compensate by loading more mood character into accents
- Are independently validated for Invariant 3 (WCAG AA contrast)

The Palette struct accommodates this with optional per-Color-Profile overrides. When Degrade runs for a 256-color Color Profile, it checks for hand-tuned values first. If present, those are used. If absent, the existing automatic `nearest_256_color` mapping applies.

Basic ANSI (16 colors) focuses solely on readability — no mood expression at that tier.

**Rejected alternatives:**

- **Automatic mapping only:** Mood character is erased for muted palettes; background hue identity is lost for all dark palettes. Functional but not considered.
- **Completely separate 256-color palette definitions:** Doubles the maintenance surface. Every palette change requires updating two definitions.
- **Skip 256-color optimization:** Contradicts Invariant 11 (graceful degradation, not feature gating) and zani's commitment to a beautiful writing experience in any terminal.

## Consequences

**Positive:**
- Curated palettes maintain recognizable mood character in 256-color terminals. The spike verified that contrast is preserved.
- Writers on constrained terminals get a considered experience, not just nearest-match approximation.
- The design principle "lean on accent saturation for mood character" (since backgrounds lose hue) produces palettes that communicate mood even when degraded.

**Negative:**
- Hand-tuned values must be tested and maintained for each palette that receives them. Doubles the color verification work for those palettes.
- A palette designer must understand 256-color cube geometry to tune effectively.

**Neutral:**
- The fallback path (automatic `nearest_256_color` mapping) is unchanged and remains the default for any palette without hand-tuned alternates.
- The Palette struct grows to accommodate optional overrides, but the runtime cost is a single check per color per render.
