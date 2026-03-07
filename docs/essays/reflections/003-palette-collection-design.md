# Reflections: Palette Collection Design
*2026-03-06*

## Names as Instrument, Not Bias

The most significant shift in thinking was the reframing of palette names from potential bias to functional component. The initial assumption was that evocative names ("Restful," "Livable Green" in the paint analogy) distort perception in a way that undermines authenticity — the writer responds to the name rather than the color, and the mood-instrument framing loses integrity. The research inverted this: labels modulate visual perception at a neurological level (Lupyan 2012), and the "unexpected descriptive" naming pattern (Miller & Kahn 2005) produces satisfaction through resolved incongruence. The name does not distort the palette experience — it completes it. A palette without a well-chosen name is an instrument missing a string.

This reframing means palette names are not a secondary design concern to be addressed after color tuning. Name and color should be co-created, as Farrow & Ball's Joa Studholme describes her process. The PNW plant register operationalizes this: when designing a Dark-Warm palette, the designer can think simultaneously about the color space and about which warm-barked PNW tree the palette evokes. If no plant fits, the palette may not belong in the collection.

## Muted as the Missing Register, Not a Compromise

The addition of "Muted" initially felt like it might dilute the mood-instrument framing — if palettes are supposed to prime affect, does a low-arousal, low-saturation palette prime *less*? The factor-analytic evidence (Ou et al., Kobayashi, Jonauskaite & Mohr 2025) clarified that desaturated colors occupy their own affective territory, not a diminished version of saturated ones. Muted is not "less mood" — it is a specific mood: contemplative, subdued, reflective. The El-Nasr emotional affordances framework is not in conflict; it established the mechanism (color primes affect subconsciously), and the low-saturation pole is simply the calm end of the arousal lever that saturation controls.

For a writing tool, the Muted register may prove to be one of the most important. Journaling, reflective prose, and quiet literary fiction are core use cases where a contemplative atmosphere serves better than an energized one.

## Manzanita as Default

The decision that the Manzanita palette should be the default — the first palette any user encounters — creates a satisfying identity circuit: the app is named after the plant, the default palette is named after the plant, and the entire naming register connects to the same Pacific Northwest botanical world. This is not merely branding coherence; it means the user's first experience of Zani is anchored in a specific, traceable, real-world referent rather than an abstract color choice.

Manzanita the plant has smooth bark ranging from mahogany to deep red to cinnamon, with warm undertones. This positions the default palette firmly in the Dark-Warm category, which is also the most natural "first experience" register — intimate, sheltering, warm. The current default, Ember, already occupies this space. The rename from Ember to Manzanita carries the same warmth but adds provenance.

## The Collection as Ecosystem

An unexpected consequence of the PNW plant register: the palette collection becomes a kind of ecosystem model. Dark-Warm palettes are named after warm-barked trees (Madrone, Cascara, Ponderosa) — species that share habitat and ecological niche. Dark-Cool palettes are named after deep-shade forest species (Salal, Hemlock, Sitka). The categories are not arbitrary groupings; they reflect real ecological relationships. A user browsing Dark-Warm palettes is, in a sense, walking through a PNW forest looking at bark.

This gives the collection a narrative dimension that a purely aesthetic naming scheme (Ember, Inkwell, Glacier) lacks. Whether users consciously register the ecological coherence is an open question — but the research on naming suggests that even subconscious register consistency contributes to the perception of curation.

## Decisions from Gate Exchange

- **Manzanita replaces Ember.** The default palette should be tuned to evoke the actual species (*Arctostaphylos*) — smooth bark ranging from mahogany to cinnamon to deep red — rather than preserving Ember's current values. Ember was a good starting place; Manzanita is the destination.
- **Flora includes mosses, lichens, and fungi.** The naming register encompasses all PNW flora, not just vascular plants. This significantly strengthens the Dark-Muted category (lichens like Usnea, mosses, fungi like Ghost Pipe) and reflects the PNW's extraordinary diversity of non-vascular species.
- **Each palette carries a provenance description.** A one-line note per palette documenting the species, its appearance, and its PNW context (e.g., "Madrone — *Arbutus menziesii*, the smooth cinnamon-barked tree of PNW coastal hillsides"). This serves the curation test, provides flavor text for documentation, and satisfies a tertiary educational purpose — introducing users to the PNW botanical world through their writing tool.
- **The ecosystem-as-collection framing is intentional.** Browsing a category should feel like walking through a specific PNW habitat. This is not a metaphor imposed on the naming — it emerges naturally from organizing plants by the same axes (light/dark, warm/cool, vivid/muted) that describe their actual habitats.

## Open Questions for Downstream Phases

- **Maximize hue diversity within categories.** Rather than asking "how much separation is enough," the design goal is to spread palettes across the available OKLCH hue space within each category. Siblings should feel like genuinely different instruments, not variations of one palette. Perceptual Sort Order still applies for smooth browsing, but the palette *selection* should aim for maximum spread, not clustering.
- **Provenance descriptions must be botanically accurate.** Madrone (*Arbutus menziesii*) is not just a "coastal hillside tree" — it thrives in the dry, fire-adapted oak savannahs of the Rogue Valley as much as on coastal bluffs. Each provenance description requires real botanical research: actual habitat range, ecological context, and species character. The descriptions should hold up to scrutiny from someone who lives among these plants. Approximate or romanticized sketches undermine the curation promise.
- **Provenance display:** Where does the provenance description surface in the UI? Options include: only in documentation, in the Palette Browser as a subtitle under each name, or as a brief tooltip/detail view. The "tool disappears" invariant (Invariant 1) suggests it should not clutter the browsing experience, but the educational value argues for accessibility.
- **Renaming existing palettes:** The current 10 palettes (Ember, Hearthstone, Inkwell, Moonstone, Neon Noir, Aurora, Parchment, Manuscript, Glacier, Daybreak) will be renamed to PNW plants. This is a breaking change for any users who have palette names in `.zani.toml` config files. Migration strategy needed.
