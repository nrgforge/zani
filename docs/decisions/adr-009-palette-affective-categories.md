# ADR-009: Palettes Organized by Affective Category

**Status:** Proposed — amended by ADR-014 (character axis expanded from 3 to 4 values)

## Context

Essay 002 established that palette selection is a mood-instrument function — the subconscious pathway is color → affect → cognition (El-Nasr emotional affordances, Wilms & Oberfeld saturation findings). The existing flat list of three palettes (ADR-006) provides no structure for a larger collection. The domain model defines Affective Category as a two-axis taxonomy (brightness x character) and Perceptual Sort Order as OKLCH hue-angle ordering within each category.

Saturation is the primary arousal lever, not hue (Wilms & Oberfeld 2018). Warmth drives approach-oriented positive affect. These two axes — saturation/energy and warmth/coolness — map onto the character dimension of the taxonomy.

## Decision

Organize the Palette collection by Affective Category. Each Palette belongs to exactly one category. Categories follow a two-axis taxonomy:

- **Brightness axis:** Dark, Light
- **Character axis:** Warm, Cool, Vivid

> **Amended by ADR-014:** Character axis expanded to (Warm, Cool, Vivid, Muted), producing eight categories instead of the six listed below.

Producing categories such as "Dark — Warm", "Dark — Cool", "Dark — Vivid", "Light — Warm", "Light — Cool".

Within each Affective Category, Palettes are ordered by Perceptual Sort Order — OKLCH hue angle as the primary sort key — so adjacent entries are perceptual neighbors. Browsing a category feels like a smooth color gradient, not random jumps.

**Rejected alternatives:**

- **Alphabetical sort:** No perceptual coherence. Adjacent palettes may be visually unrelated.
- **Free-form tags:** Overcomplicated for a curated collection. Tags invite proliferation; a fixed taxonomy constrains it.
- **Flat list with no categorization:** Does not scale beyond approximately five palettes.

## Consequences

**Positive:**
- Browsing is intentional (mood selection) rather than trial-and-error. The writer chooses the affective register, not just the aesthetic.
- Adjacent palettes share visual character, so preview transitions during Browse are smooth.
- The taxonomy maps directly onto the research: saturation/warmth as the reliable affective levers.

**Negative:**
- Each new Palette must be assigned to a category. If a palette spans categories (e.g., moderate saturation that is neither "Warm" nor "Vivid"), the assignment is a judgment call.

**Neutral:**
- OKLCH hue angle is used only for sort-order computation, not stored as Palette data. Palettes remain stored as RGB values.
- The category taxonomy is fixed at the code level. Adding a new category axis (e.g., "Muted") requires a code change, not configuration.
