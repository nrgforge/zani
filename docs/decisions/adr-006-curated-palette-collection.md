# ADR-006: Curated Palette Collection

**Status:** Proposed

## Context

Invariant 3 states: "No pure black or white. WCAG AA minimum. The Palette never uses #000000 or #FFFFFF. All foreground/background color pairs maintain at least a 4.5:1 contrast ratio."

The research found:
- Pure black on pure white causes halation — a glow effect that strains eyes, worse for the ~33% of people with astigmatism.
- Pure white on pure black has the same problem.
- Soft dark backgrounds with warm off-white text reduce total light emission and visual fatigue.
- Yellow/warm-tinted text is perceived as least visually fatiguing.
- In dark/low-light environments (common for writing), muted colors reduce strain.

These findings inform the *default* palette (warm, muted, dark). But the guardrails for all palettes are narrower than the default's aesthetic: no pure black/white, WCAG AA contrast. Within those guardrails, palettes can be warm or cool, subdued or vivid, evocative of any mood — campfire warmth, neo-noir neon, sepia parchment.

## Decision

Zani ships with a curated collection of named palettes. Each palette is research-informed (respects the color science constraints) but evocative — it has a name, a mood, and a distinct feel. Examples of the kind of palettes to include:

The specific palette collection requires its own research cycle (color theory, cross-terminal testing, mood design). The following are illustrative of the range, not a final list:
- Warm dark (campfire, candlelit)
- Warm light (manila, legal pad, parchment)
- Cool dark (inkwell, neo-noir)
- Vivid/expressive (neon accents, retrofuture)

Each palette provides both a dark and light variant where it makes sense (campfire is naturally dark; legal pad is naturally light; some palettes work both ways). Each palette defines:
- Background color
- Foreground color
- Dimming interpolation endpoints (per ADR-004)
- Accent colors for Markdown Styling (headings, emphasis, links, code)

All palettes satisfy Invariant 3: no `#000000` or `#FFFFFF`, all color pairs at 4.5:1+ contrast ratio (WCAG AA). The constraint is accessibility and eye health. Within that, palettes are playful, opinionated, and evocative. A palette can set a mood for the writing session.

The default palette is warm and muted dark (per the research on low-light writing environments). The writer selects a palette in configuration or via the Settings Layer.

## Consequences

**Positive:**
- Writers can match the writing environment to their mood or preference without leaving the research-informed guardrails.
- Named, evocative palettes make Zani feel considered and personal — not "dark theme #3."
- Accessibility is built into every option, not a trade-off against aesthetics.
- Each palette is a complete color system (foreground, background, dimming gradient, accents), so Focus Mode and Markdown Styling work correctly across all palettes.

**Negative:**
- Each palette requires visual testing across terminals and displays. More palettes means more testing.
- Dimming interpolation must work well against each palette's background — this needs to be validated per palette, not assumed from the math alone.
- Palette names and moods are subjective. Getting them right is design work, not just color picking.

**Neutral:**
- On 256-color terminals, palettes degrade to the nearest available colors (per Invariant 11). Character may be approximated but not exact.
- The specific palette collection will grow over time. The initial set should be small and well-tested rather than exhaustive.
