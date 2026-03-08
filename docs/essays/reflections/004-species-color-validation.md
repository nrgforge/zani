# Reflections: Species-Color Validation

## The naming register has a cost

The naming register from Essay 003 was framed as a productive constraint — a quality gate that prevents arbitrary palette expansion. This validation revealed the constraint's other edge: it imposes an ongoing curation burden. Every palette must satisfy three competing requirements simultaneously:

1. **Botanical truth** — the species' real colors, grounded in external sources
2. **Technical constraints** — WCAG AA contrast, 15° OKLCH hue diversity, within-category distinctness
3. **Affective register** — the category's mood (gentle morning, bright solar, etc.)

When all three align, the palette is effortless — Ponderosa's butterscotch bark, Camas's blue-violet spikes, Fly Agaric's red cap. When they conflict, something gives. The question is what.

## What gives, and the order matters

The validation found that the original design process sometimes sacrificed botanical truth to satisfy technical constraints. Oregon Sunshine lost its characteristic yellow. Balsamroot's golden flowers became muted amber-brown. Both cases followed the same pattern: the 15° hue diversity requirement pushed backgrounds green to maintain separation from neighbors, and the accent colors drifted to match.

The user's insight: the constraint-satisfaction process has a cost, and it's not evenly distributed. Palettes whose species have distinctive, high-chroma signature colors (vivid yellow, bright orange) are more vulnerable to drift than palettes whose species have broad, muted color profiles (bark, lichen, foliage). The solution isn't to relax the constraints — it's to choose species whose real colors already sit where the constraints need them to be.

## The label-feedback loop is real and bidirectional

The Lupyan label-feedback research (Essay 003) established that names prime visual perception — a palette labeled "Ponderosa" activates warm butterscotch associations before the colors even register. The validation revealed the failure mode: when the primed expectation contradicts the actual colors, the label creates cognitive dissonance rather than resonance. Oregon Sunshine primes bright yellow; the palette delivers mint green. The "search for meaning" that Miller & Kahn (2005) identified as pleasurable becomes an unresolvable mismatch.

The fix for Oregon Sunshine — moving it to Light Vivid where the palette can authentically represent the species — demonstrates the principle: the name and the palette must agree. When they can't agree in a given category, the species either needs a different category or needs to be replaced by a species whose real colors match.

## "Well curated" means ongoing verification

The user flagged both Manzanita and Oregon Sunshine from personal experience — having grown Oregon Sunshine and living among manzanita in the Rogue Valley. This local knowledge caught errors that database searches might miss (Manzanita's confirmed geographic presence) and surfaced problems that the original design process papered over (Oregon Sunshine's missing yellow).

The implication: curation is not a one-time design exercise. The 40-palette collection will need periodic validation as palettes are tuned, constraints evolve, and new species are considered. The methodology from this validation — external source comparison with A/B/C/D scoring — provides a repeatable framework for future audits.

## Composition determines essence

A species' signature colors are not interchangeable across palette slots. The 7-slot palette structure (foreground, background, dimmed, heading, emphasis, link, code) is not a bag of colors — it is a composition where position determines experience. The background dominates the visual field (~90% of screen area in a writing app). The accent slots provide punctuation. The same species colors arranged differently produce fundamentally different affective experiences.

Oregon Sunshine has two signature colors: bright golden-yellow (flowers) and gray-green (woolly foliage). Composed as "green background + amber accents," the palette reads as a leafy meadow — gentle and muted. Composed as "warm near-white background + vivid yellow accents," the palette reads as sunshine itself — bright and solar. The species is the same; the colors are drawn from the same botanical source; but the composition determines which part of the species the writer is "inside."

The implication for palette design: identifying a species' signature colors is necessary but not sufficient. The designer must also decide which color represents the species' *essence* — its most characteristic visual feature — and that color should drive the palette's dominant slot (usually the background or the primary accent). For Oregon Sunshine, the essence is the flower — the sunshine. For Ponderosa, the essence is the bark. For Sword Fern, the essence is the frond. The essence determines the composition; the composition determines the category.

This principle explains why some species fit naturally in one category but not others. Oregon Sunshine's essence (vivid yellow flower) makes it a natural Light Vivid palette when the yellow drives the accent. Placing it in Light Warm required suppressing the essence to serve the category, which broke the naming register's congruence promise.

## Three modes of essence

Conversation with the user (who grows Oregon Grape and Oregon Sunshine, and associates Sitka with the Oregon coast) revealed that "essence" is not a single concept. Species connect to their palettes through at least three different compositional modes:

**Feature essence.** One botanical feature dominates the species' identity. The background or primary accent is built around that feature. Examples: Ponderosa (bark — the butterscotch jigsaw plates are the entire identity), Oregon Sunshine (flower — the sunshine is in the name), Fly Agaric (cap — the red cap IS the organism in cultural imagination), Sword Fern (frond — there is nothing else).

**Throughline essence.** The species has varied visual interest across seasons, but one element persists as the constant while others provide seasonal punctuation. The background represents the throughline; accents represent the seasonal variation. Example: Oregon Grape — the holly-like evergreen foliage is the year-round constant, while little yellow flowers (spring) and purple berries (fall) are seasonal accents. The palette captures this: dark background with golden emphasis (flowers) and periwinkle heading (berries) as the changing details against the foliage constant.

**Place essence.** The species evokes a habitat or landscape more than a single botanical feature. The background represents the atmosphere — the light, the air, the feeling of being in that place — which may not correspond directly to any part of the organism. Example: Sitka — the dark blue-gray background is not precisely "needle color" or "bark color" but the experience of standing in a fog-belt Sitka spruce forest on the Oregon coast. Sagebrush works similarly: the palette evokes the high desert east of the Cascades as much as the plant itself.

Each mode composes differently and suggests different design priorities:
- Feature essence: identify the one defining color, build the palette around it
- Throughline essence: identify the constant, use it for the background; use accents for the seasonal cycle
- Place essence: identify the atmospheric impression, use it for the background; use accents for the organism's actual botanical colors

This taxonomy is descriptive, not prescriptive — a single species may blend modes. Madrone has feature essence (the peeling bark) but also place essence (dry oak savannah hillsides). The value is in making the compositional rationale explicit and traceable, so future palette design decisions can articulate *why* a particular color occupies a particular slot.

## Open question: provenance bias

The provenance descriptions were found to contain subtle bias toward palette colors rather than real species colors. Balsamroot's provenance says "warm amber-gold" when field guides say "bright yellow." Witch's Hair says "olive-black" when the real species is pale yellow-green. This suggests the provenance texts were written (or drifted) to rationalize the palette rather than document the species. Future provenance writing should be grounded in external sources first, before the palette colors are known.
