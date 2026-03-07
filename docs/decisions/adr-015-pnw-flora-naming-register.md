# ADR-015: PNW Flora Naming Register and Collection Structure

**Status:** Proposed

## Context

ADR-006 established that Zani ships with a curated collection of named palettes that are "evocative — each has a name, a mood, and a distinct feel." The essay noted that the specific palette collection required its own research cycle. Essay 003 completed that research, investigating naming psychology, choice architecture, and collection structure.

Three findings drive this decision:

**Naming psychology.** Lupyan's Label-Feedback Hypothesis (2012, 2013) demonstrates that linguistic labels modulate visual perception at a neurological level — names are part of the mood instrument, not decoration. Miller & Kahn (2005) found that "unexpected descriptive" names (atypical but resolvable when seen alongside the color) produce the strongest positive response. Chou (2020) confirmed this effect is strongest when color is a secondary attribute — exactly Zani's context, where the palette sets a mood for writing.

**Register consistency signals curation.** Farrow & Ball's ~132 colors draw from a single world (English landscape, historical architecture). Community theme collections (iTerm2-Color-Schemes: 450+ themes) feel chaotic because names come from unrelated worlds. Schema congruity theory (Mandler) shows that moderate incongruence (name evokes category affect through indirect association) is optimal.

**Choice architecture.** Chernev et al. (2015) meta-analysis (N = 7,202) shows choice overload is moderated by set comparability, decision context, preference clarity, and decision goal. Zani's categorized, hedonic, keyboard-navigated browser scores well on all four. Sharma (2023) found within-category counts of 3–7 are optimal. Mogilner et al. (2008) found that meaningful categories increase satisfaction.

Zani's name derives from Manzanita (*Arctostaphylos*), a Pacific Northwest genus. Extending this botanical origin to the full collection provides the register consistency, traceable provenance, and moderate schema congruence the research identifies as optimal.

## Decision

### Naming Register

Every Palette is named after a Pacific Northwest species — vascular plants, mosses, lichens, or fungi — whose natural color associations are moderately congruent with the palette's Affective Category. This is Invariant 16 in the domain model.

Each name must satisfy the five-point curation test:
1. **Traceable** — points to a specific species with a real appearance
2. **Category-resonant** — affect matches category without restating it
3. **Register-consistent** — sounds like it belongs with the other names
4. **Uniquely evocative** — activates a distinct sensory image from siblings in its category
5. **Resolvable** — seeing the palette alongside the name, the connection clicks

Single-word names are preferred (Madrone, Salal, Cascara, Hemlock, Yarrow, Lupine). Two-word names are acceptable when the species demands it (Ghost Pipe, Sword Fern, Silver Fir).

### Provenance Descriptions

Each Palette carries a Provenance Description: a botanically accurate one-line note documenting the species' scientific name, appearance, and Pacific Northwest ecological context. Descriptions must reflect actual habitat range and species character — not romanticized sketches. They must hold up to scrutiny from someone who lives among these plants.

### Collection Structure

Eight Affective Categories (per ADR-014) × five Palettes each = 40 Palettes total. Five per category is the target, not an approximate range. The constraint of hand-curation and the Naming Register together provide the quality ceiling.

Within each category, palettes are spread across the available OKLCH hue space for maximum perceptual diversity. Siblings should feel like genuinely different instruments, not variations of one palette.

### Default Palette

The default Palette is Manzanita, tuned to evoke the species' smooth bark ranging from mahogany to cinnamon to deep red. Manzanita belongs to the Dark-Warm Affective Category. This replaces Ember as the default.

### Existing Palette Renames

All ten existing palettes are renamed to PNW species. Zani has no users; no backwards-compatibility migration is needed.

**Rejected alternatives:**

- **Preserve existing names (Ember, Inkwell, etc.) alongside botanical names:** Two naming registers in one collection destroys the register consistency that the research identifies as the primary signal of curation.
- **Use a different register (geological features, PNW places, weather phenomena):** Botanical names connect directly to the app's identity (Zani = Manzanita) and provide the richest source of color-congruent associations across all eight categories. Other registers were not evaluated but could not match this provenance depth.
- **Generated palettes instead of hand-curated:** The Naming Register constraint requires a corresponding species for each palette, which prevents algorithmic generation. Hand-curation is also required for Invariant 3 compliance, 256-color hand-tuning (ADR-012), and the mood-instrument quality standard.
- **Variable count per category (3–7):** Five per category provides elegant uniformity and a concrete design target. The Naming Register has sufficient candidates for all eight categories at this density.

## Consequences

**Positive:**
- Every palette name is traceable, register-consistent, and resolvable — eliminating the "just vibes" problem of uncurated theme collections.
- The PNW flora register connects the collection to the app's identity (Zani = Manzanita) and provides a narrative dimension — browsing a category is like walking through a PNW habitat.
- The Naming Register acts as quality control: no palette without a corresponding species, preventing unbounded collection growth.
- Provenance Descriptions add educational value — users learn about PNW flora through their writing tool.
- 40 palettes across 8 categories gives the Palette Browser ~5 entries per group, well within the research-supported sweet spot.

**Negative:**
- 30 new palettes must be designed, named, color-tuned, and validated against Invariant 3.
- Each Provenance Description requires real botanical research — approximate or romanticized descriptions undermine the curation promise.
- The Naming Register constrains future palette additions — a palette cannot be added without a species fit.
- The `Palette` struct needs a `provenance` field (or equivalent) to carry descriptions.

**Neutral:**
- ADR-006 is amended: its "curated collection of named palettes" now has a specific naming register (PNW flora), a specific size (40), and a specific default (Manzanita). ADR-006's core decision (curated, research-informed, evocative) is unchanged — the new ADR operationalizes it.
- The Provenance Description display location in the UI is deferred (domain model Open Question 9). The data structure should carry descriptions regardless of where they surface.
