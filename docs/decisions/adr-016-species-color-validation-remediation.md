# ADR-016: Species-Color Validation Remediation

**Status:** Proposed

## Context

ADR-015 established the Naming Register — every Palette is named after a Cascadia bioregion species with moderate color congruence. Essay 004 validated all 40 palette-species assignments against authoritative botanical databases (Oregon Flora Project, USDA PLANTS, USFS Silvics/FEIS, Burke Herbarium, McCune & Geiser macrolichen keys, regional field guides). Invariant 16 (strengthened in the domain model after Essay 004) now requires Signature Colors to be verified against external sources and composition to be driven by the Species Essence.

The audit scored 28 A (strong match) and 12 B (reasonable match), with zero C or D scores. However, two structural problems emerged:

**Missing iconic feature.** Oregon Sunshine (#22, Light Warm) and Balsamroot (#25, Light Warm) are both bright golden-yellow wildflowers whose palettes render mint-green backgrounds with muted amber-brown accents. The backgrounds connect to real foliage colors, but both species' defining visual — documented as bright golden-yellow flowers by every source consulted — is suppressed. The species names prime expectations the palettes cannot satisfy (Invariant 16's resolvability requirement; label-feedback research from Essay 003).

**Geographic/color mismatch.** Reindeer Lichen (#40, Light Muted) is circumboreal and uncommon in the Cascadia bioregion per McCune & Geiser (2009). Its palette overemphasizes green relative to the real silvery-gray species.

**Hue redundancy.** Moving Oregon Sunshine to Light Vivid (where its vivid yellow fills the one missing major hue) would create a 6th palette in that category. Columbine's orange-red sits between Paintbrush's red and Tiger Lily's orange — the most crowded hue region.

**Provenance errors.** Six Provenance Descriptions contain factual inaccuracies discovered through source comparison.

## Decision

### Species replacements

Three palettes receive new species names. In each case the palette colors are unchanged — the problem is the species name, not the colors.

1. **Balsamroot (#25, Light Warm) → Licorice Fern.** *Polypodium glycyrrhiza* — bright green fronds match the Rgb(218, 242, 210) background; golden-brown to cinnamon-brown sori map to both warm accent colors; "licorice" evokes warm, sweet, amber-brown. Quintessentially Cascadian: the defining epiphytic fern, southern Alaska through coastal California.

2. **Reindeer Lichen (#40, Light Muted) → Silver Fir.** *Abies amabilis* (Pacific Silver Fir) — silvery-white needle undersides match the Rgb(225, 232, 238) background; dark green needle tops match the muted green accents; deep purple mature cones match the muted purple link Rgb(80, 72, 90). The only candidate matching all three of the palette's color families. Mid-elevation Cascadia tree, SE Alaska through Oregon.

3. **Oregon Sunshine's vacated Light Warm slot (#22) → Bracken.** *Pteridium aquilinum* — characteristically light green fronds match the Rgb(230, 242, 205) background; straw-brown stipes map to the amber heading; amber-gold autumn tones match the emphasis. "Bracken" is itself used as a color name meaning golden-brown. Extremely widespread in Cascadia.

### Category move

**Oregon Sunshine moves from Light Warm to Light Vivid** with a redesigned palette. The species' defining feature — bright golden-yellow daisies (OSU Extension, Portland Nursery, USDA Fact Sheet) — is high-chroma saturated color, definitionally "vivid." Yellow is the one major hue absent from Light Vivid. The redesigned palette requires new colors: vivid golden-yellow accents on a light background, satisfying WCAG AA 4.5:1 contrast and 15° OKLCH hue diversity against Light Vivid neighbors.

### Retirement

**Columbine retired from Light Vivid** to maintain five palettes per category. Columbine scored A in validation — it is retired for hue-diversity optimization, not quality. Its orange-red heading Rgb(170, 62, 25) sits between Paintbrush's red Rgb(175, 40, 48) and Tiger Lily's orange Rgb(168, 78, 10).

Without Columbine, Light Vivid becomes: Paintbrush (red), Tiger Lily (orange), Oregon Sunshine (yellow), Farewell (magenta-pink), Camas (blue-violet) — distributed cleanly around the hue wheel.

### Species name correction

**Jack-o'-Lantern (#13):** Correct *Omphalotus olearius* (European) to *Omphalotus olivascens* (western North American) in the Provenance Description. No color or name change.

### Provenance text corrections

Six Provenance Descriptions are corrected to match external source documentation:

| # | Species | Error | Correction |
|---|---------|-------|------------|
| 7 | Oakmoss | Says "muted teal-green" | Real thallus is gray-green to olive, not teal |
| 10 | Witch's Hair | Says "olive-black to deep greenish-brown" | *A. sarmentosa* is pale yellow-green to straw; may confuse with *Bryoria* |
| 13 | Jack-o'-Lantern | Uses *O. olearius* | Western species is *O. olivascens* |
| 28 | Partridgefoot | Says "gray-green" foliage | USDA describes foliage as "glossy green" |
| 29 | Lupine | Claims "silvery sheen from fine hairs" | *L. latifolius* is not silvery (unlike *L. argenteus*) |
| 40 | ~~Reindeer Lichen~~ Silver Fir | Replace entire provenance | New species, new provenance |

**Rejected alternatives:**

- **Relax scoring criteria to accept B-scores as sufficient:** The B-scores are technically within "moderate congruence," but the label-feedback research (Essay 003) shows that names create expectations. Oregon Sunshine priming vivid yellow and delivering mint green creates cognitive dissonance, not resonance. The Naming Register's value depends on the name-color connection being resolvable, not merely defensible.
- **Replace Oregon Sunshine and Balsamroot with non-yellow species in Light Warm:** This was the original Q1 approach and remains sound for Balsamroot. But Oregon Sunshine has personal significance to the user and is one of the few species whose Signature Colors naturally fill a gap in another category. Retaining the species in a better category preserves both the personal connection and the collection quality.
- **Keep Columbine and allow Light Vivid to have 6 palettes:** Six is within the 3–7 range from Essay 003 research. However, with four palettes in the red-to-orange quarter (Paintbrush, Columbine, Tiger Lily, Oregon Sunshine) the hue diversity would be weak at the warm end. Dropping the most crowded entry produces a cleaner distribution.
- **Keep Reindeer Lichen with corrected provenance:** The color mismatch (green-overemphasized vs real silver-gray) and geographic weakness (uncommon in Cascadia per McCune & Geiser) together make this the weakest fit in the collection. Silver Fir matches all three color families and is genuinely Cascadian.

## Consequences

**Positive:**
- Projected score distribution improves from 28 A / 12 B to 32 A / 8 B.
- All geographic borderline flags resolved (Manzanita confirmed by direct observation, Jack-o'-Lantern kept with defensible range, Reindeer Lichen replaced by Silver Fir).
- Oregon Sunshine's vivid yellow fills the one missing major hue in Light Vivid.
- Six provenance errors corrected, strengthening the collection's botanical grounding.
- Collection stays at 40 palettes, 5 per category.

**Negative:**
- Oregon Sunshine requires a completely new palette design — new RGB values satisfying WCAG AA contrast, 15° OKLCH hue diversity, and the species' vivid golden-yellow Signature Colors.
- Columbine is a quality palette (scored A) being removed for structural reasons. If the collection expands in the future, Columbine could return if hue space permits.
- Three provenance descriptions must be rewritten from scratch (Bracken, Licorice Fern, Silver Fir).

**Neutral:**
- ADR-015 is amended: its "Five per category is the target" is maintained — the net effect is still 8 × 5 = 40.
- The Palette struct's `name` and `provenance` fields change for 4 palettes (Bracken, Licorice Fern, Silver Fir, Oregon Sunshine). The Columbine palette is removed. Oregon Sunshine gains entirely new color values.
- Existing test `collection_contains_exactly_40_palettes()` remains valid. Test `each_category_contains_exactly_5_palettes()` remains valid.
- Retired species (Columbine, Balsamroot, Reindeer Lichen) should be documented in the Flora Reference with retirement rationale and conditions for return. Reindeer Lichen in particular is an evocative name that could return if a future palette's colors match its real silver-gray appearance.

**Follow-on goal: all A-scores.** This ADR addresses the 4 weakest palettes (structural problems: wrong category, geographic issues, missing iconic features). The remaining 8 B-scores are addressable through color tuning (5 palettes with hue/tone drift) and provenance/composition fixes (3 palettes). This follow-on work applies the same principle — ground palettes in botanical truth — and should be tackled as a subsequent implementation cycle after the structural changes land, with each color adjustment re-verified against WCAG AA contrast and 15° OKLCH hue diversity constraints.
