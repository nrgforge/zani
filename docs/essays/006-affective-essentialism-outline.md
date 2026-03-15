# Affective Essentialism: From Color Themes to Calibrated Instruments
*Outline — 2026-03-09*

## Central Question

How does the seemingly mundane task of choosing color palettes for a writing application lead to a transferable empirical methodology for grounding aesthetic design in measurable properties of authentic source material?

## Volta

A person sits down to pick color themes for a writing app — one of the most "vibes-based" tasks in software — and discovers that doing it rigorously produces a transferable methodology for any domain where affect is the design goal and the medium is measurable. The question changes from "what colors" to "how do you distill the essence of something into an affective state based on its properties."

## Structure: Braided

Two threads interweave throughout:

**Thread 1 (Specific):** The journey of building Zani's 40-palette collection — from theme-picker frustration through flora naming, provenance bias, essence modes, chromatic inversion, to enforced chroma targets. Concrete, with code, with the matrix bug, with Oregon Sunshine moving categories.

**Thread 2 (General):** The methodology crystallizing — affective essentialism — and its transferability. The lighting agent as origin, Zani as the site of crystallization. The four-step pattern, its distinctive Step 4, and the structural conditions for applicability across domains.

## Bigger Frame

Tools that help people be more generative rather than generate for them. The current moment is dominated by tools that produce output *for* people. This project — and the lighting agent before it — is about tools that *set the conditions* for human creativity. Affective essentialism is the methodology for building such tools.

---

## Opening: The Theme-Picker Problem

- The experience everyone knows: scrolling Reddit threads for "best theme," cycling through Gruvbox/Solarized/Railscasts, the overwhelming choice set with no principled basis for selection
- Contrast ratios are terrible on many community themes. Where do they even come from? Someone liked the vibes
- **Thread 2 enters:** Prior work on an interactive lighting agent (masters research) used El-Nasr's work on simulated illumination as emotional modulator — gesture pipeline translating to aesthetic lighting choices for different affective states. The same structure: input → palette → affect
- The question that started Zani's color work: can this be done rigorously for a writing environment?

### Key reference
> "El-Nasr's most theoretically significant contribution is the concept of *emotional affordances* — an extension of Gibson's affordance theory (1966) arguing that color and lighting function as subconsciously operative environmental stimuli that afford specific emotional responses without requiring deliberate attention."
> — Essay 002, §Color as Emotional Affordance
> **Note:** The term "emotional affordances" should be cited cautiously; verify it appears verbatim in El-Nasr's work. The CHI 2005 entry (El-Nasr, Zupko, & Miron) is an extended abstract.

---

## The Mood-Instrument Inversion

- Essay 002's key finding: color operates below conscious attention. Pathway is color → affect → cognition, not color → aesthetics → preference
- Saturation is the strongest single predictor of physiological arousal, ahead of brightness — though both contribute, along with their interactions (Wilms & Oberfeld 2018). The blue-for-creativity claim (Mehta & Zhu 2009) failed replication (Steele 2014)
- **Medium transfer note:** The cited color science research studied physical environments. But during focused writing, the screen background fills a comparable proportion of the visual field (~90%) — the perceptual conditions are analogous to sitting in a tinted room
- The reframe: palettes are instruments, not decorations. The word "palette" itself activates the art-making metaphor — the writer's text is the paint
- WCAG AA constraints leave mood expression *unconstrained* — lightness relationships are the only restriction; hue and saturation are free parameters

### Key references
> "Saturation was the primary driver of physiological arousal, with roughly twice the impact of brightness (regression: arousal ~ -0.31 x brightness + 0.60 x saturation). Skin conductance responses confirmed this: saturated colors produce higher arousal regardless of hue."
> — Essay 002, summarizing Wilms & Oberfeld, *Psychological Research*, 2018

> "Steidle and Werth (2013) conducted six experiments showing that dim lighting (~150 lux) outperformed standard office lighting for divergent thinking tasks. The mechanism: darkness activates a felt sense of freedom from social constraints."
> — Essay 002, §Environment as Co-Author
> **Citation correction:** The dim-lighting finding is Steidle, A. & Werth, L. (2013), "Freedom from constraints: Darkness and dim illumination promote creativity," *Journal of Environmental Psychology*, 33, 67–80. The original essay misattributed this to Xu & Labroo (2014), which is a different paper about bright light amplifying emotional intensity.

---

## Names Complete the Instrument

- The initial assumption: evocative names distort perception, undermining authenticity
- Lupyan's Label-Feedback Hypothesis (2012): labels modulate visual perception at a neurological level, in milliseconds, below conscious attention
- Miller & Kahn (2005): "unexpected descriptive" names produce strongest positive response when color is visible alongside — the "search for meaning" that resolves pleasurably
- The Pantone counterexample: deliberately neutral naming as a valid different choice for a different purpose (industrial color matching vs. affective design)
- Paint-store experience (Sunset, Liveable Green) as personal confirmation
- **Thread 2:** Naming isn't post-hoc labeling — it's a constitutive part of the instrument. A palette without a well-chosen name is an instrument missing a string

### Key reference
> "The 'unexpected descriptive' category — atypical but resolvable — produces the strongest positive response when the user can see the color alongside the name. The mechanism is a 'search for meaning' that, when resolved, generates satisfaction."
> — Essay 003, §Names as Part of the Instrument, summarizing Miller, E. G. & Kahn, B. E. (2005), "Shades of Meaning," *Journal of Consumer Research*, 32(1), 86–92

> "Lupyan's Label-Feedback Hypothesis (2012, 2013) demonstrates that linguistic labels actively modulate visual perception. Hearing or reading a label activates visual features corresponding to the labeled category, changing what a person sees at a perceptual level — not merely how they think about it."
> — Essay 003, §Names as Part of the Instrument
> Lupyan, G. (2012), *Frontiers in Psychology*, 3, 54

---

## The Flora Register and Its Failure Mode

- PNW flora as naming register: traceable provenance, register consistency, connection to Zani's own name (Manzanita)
- Cohesion across names matters (Farrow & Ball vs. iTerm2 chaos) — the register *is* the signal of curation
- **But:** provenance descriptions drifted to rationalize palette colors rather than document species reality. "Warm amber-gold" for a bright yellow flower. Descriptions were written backward — from palette to plant, not plant to palette
- Personal botanical knowledge as the check: growing Oregon Sunshine, living among manzanita in the Rogue Valley
- **Thread 2 enters:** Domain expertise is irreplaceable. The person building the instrument needs authentic knowledge of the source material

### Key reference
> "The naming register imposes a productive constraint on palette design. A palette cannot be added to the collection unless a PNW plant exists whose color associations are moderately congruent with the palette's affect."
> — Essay 003, §The Naming Constraint as Quality Control

---

## Essence Modes: How Source Material Connects to Composition

- Oregon Sunshine as the clearest case: no yellow in a palette named after a species defined by joyful yellow flowers. The naming register promised congruence; the implementation broke it
- The recognition: the species' *dominant feature* needed to drive the palette's *dominant slot* (the background, 90% of screen area) — not be relegated to accent punctuation
- Three modes emerged from direct experience with the plants:
  - **Feature essence:** One botanical feature dominates (Ponderosa's bark, Oregon Sunshine's flower)
  - **Throughline essence:** Year-round constant as background, seasonal variation as accents (Oregon Grape's evergreen foliage + spring flowers + fall berries)
  - **Place essence:** Atmospheric impression of the habitat (Sitka = fog-belt coast, Sagebrush = high desert east of the Cascades)
- **Thread 2:** Essence modes are a novel framework not from the literature — they emerged from the builder's direct experience with the source material. This is the bridge between authentic knowledge and compositional decisions

### Key reference
> "Composition determines essence. A species' signature colors are not interchangeable across palette slots. The 7-slot palette structure is not a bag of colors — it is a composition where position determines experience."
> — Reflection 004, §Composition determines essence

---

## The Chromatic Inversion: Taxonomy Correct, Implementation Unexpressed

- The spike that changed the question: measured all 40 backgrounds, found Light Warm was more vivid than Light Vivid. The hierarchy was inverted at the top
- Three Light Vivid backgrounds were perceptually indistinguishable from Light Muted (Farewell ΔE 4.38, Camas ΔE 3.47 — same band as Muted palettes)
- The bottom worked: Muted was correctly restrained. The failure was at the top — Vivid wasn't vivid enough
- The distinction lived entirely in accent colors occupying 5-10% of the visual field. The dominant experience — the room the writer sits in for hours — wasn't expressing the taxonomy
- **Thread 2:** Intent without measurement is indistinguishable from vibes. The categories were researched, the taxonomy was sound, but without empirical verification against the perceptual experience, the implementation silently drifted

### Key references
> "Light Warm backgrounds (mean ΔE2000 12.73 from neutral) register as more vivid than Light Vivid (mean ΔE2000 7.14) — the hierarchy is inverted."
> — Essay 005, §Empirical Baseline

> "The vivid/muted distinction is carried entirely by accent colors. But accents are typographic punctuation — headings, emphasis, links, code spans — occupying perhaps 5–10% of the visual field. The dominant experience, the one the writer sits inside for hours, is the background."
> — Essay 005, §Where the Distinction Actually Lives

### ΔE2000 perceptual thresholds (from color science literature)
| ΔE2000 | Perception |
|--------|-----------|
| < 1.0 | Imperceptible |
| 1.0–2.5 | Barely tinted |
| 2.5–5.0 | Noticeably tinted |
| 5.0–10.0 | Clearly colored |
| > 10.0 | Vivid — strongly colored |

---

## Building the Measurement, Finding It Broken

- Implementing CIEDE2000 (ISO/CIE 11664-6:2014) from scratch in Rust to measure palette conformance
- TDD: acceptance tests written to fail, palette retuning to make them pass
- The matrix bug: a single coefficient in the sRGB→XYZ conversion (0.2591332 instead of the IEC 61966-2-1 standard 0.1191920) inflating every measurement
- **Thread 2:** The measuring instrument had to be fixed before the palettes could be fixed. Perception caught the error — Light Muted backgrounds with 1-point RGB spreads shouldn't register ΔE 6.85. Domain knowledge checked the math
- Balancing three competing constraints across 40 palettes: chroma targets, WCAG AA contrast, 15° hue diversity. Each adjustment could violate another constraint

---

## Affective Essentialism: The Methodology

- The named pattern that crystallized from the journey:
  1. **Define the intended affect** — what affective state should this produce?
  2. **Identify measurable properties of the medium** that correlate with that state
  3. **Set target ranges per category** — evidence-based, with perceptual thresholds
  4. **Validate against authentic source material** — domain expertise confirms the measurement, not the other way around
- **What distinguishes this from generic evidence-based design:** In standard design empiricism, hitting your metrics means you're done. In affective essentialism, hitting your metrics means you've passed one gate — you still need domain expertise to confirm the result is authentic. Step 4 is the novel move. A chroma audit tells you Farewell's background is in the wrong perceptual band. Only someone who knows the species can tell you whether the fix honors the plant. A palette can be in the right ΔE2000 band and still not feel like Oregon Sunshine
- The key constraint: measurement confirms but does not replace domain knowledge. Because affect operates below conscious attention, purely numeric compliance is insufficient — the instrument must also be authentic to the source material that grounds it
- **Thread 1 closes:** The 40 palettes now pass 448 tests, backgrounds conform to enforced ΔE2000 ranges, every species' essence drives its dominant slot

---

## Transferability: The Lighting Agent and Beyond

- **The lighting agent as origin, not validation.** The masters project — gesture classification → lighting palette → affective state — was the *seed* of this thinking. The methodology didn't exist yet; it crystallized in the Zani work, then mapped back to the lighting agent retroactively. They share the same four-step structure with different media. The retroactive mapping confirms structural alignment; it is not an independent replication
- **Why the pattern should generalize.** Three structural properties make a domain amenable to affective essentialism: (1) the medium operates below conscious attention, making vibes-based design insufficient; (2) the medium is measurable, making empiricism possible; (3) authentic source material exists that can validate the measurement. These properties are shared by other affective design domains — sound design has perceptual metrics (loudness, spectral centroid, roughness), typography has measurable properties (x-height ratios, stroke contrast, spacing) — though prospective application in these domains remains to be demonstrated
- **The bigger frame:** Tools that prime human creativity rather than replace it. The lighting agent sets conditions through light. Zani sets conditions through color. Both trust the human to do the creative work — they just tune the environment
- The current moment: most tool-building energy goes toward generating output *for* people. Affective essentialism is a methodology for the other kind of tool — the kind that makes the human more generative

---

## Closing Implication

- The question that changed: from "what color themes should we have" to "how do you distill the essence of something into an affective state based on its properties"
- The first question has a flat answer (a list). The second has transferable implications
- What vibe-coding can't produce: the provenance bias was invisible to automated checking. The chromatic inversion was invisible without empirical measurement. The essence modes came from standing in a Sitka forest, growing Oregon Sunshine, living among manzanita. Every step required either domain expertise or empirical rigor or both. These failure modes — plausible-but-backward rationalization, intent-without-measurement drift, composition without authentic knowledge — are structural features of affective design, not peculiarities of color palettes. Any domain where the medium operates below conscious attention will reproduce them
- The methodology emerged from the process. It was not designed in advance. The RDD cycle — research → model → decide → build, with epistemic gates at every step — created the space for a genuinely novel framework to crystallize from what looked like a mundane design task

---

## Citation Corrections (from audit)

These corrections apply to citations as they appear in the project's research essays. The outline uses corrected versions above.

| Original Citation | Correction |
|---|---|
| Xu & Labroo, *J. Environmental Psychology*, 2014 (dim lighting) | The dim-lighting/creativity finding is **Steidle & Werth (2013)**, *J. Environmental Psychology*, 33, 67–80. Xu & Labroo (2014) is about bright light and emotional intensity, published in *J. Consumer Psychology*. |
| Glaveanu et al., *Frontiers in Psychology*, 2019 (4E cognition) | The 2019 *Frontiers in Psychology* paper on 4E/creativity is sole-authored by **Malinin, L. H.** (2019), not Glaveanu. |
| El-Nasr "emotional affordances" | The term may not appear verbatim in the CHI 2005 extended abstract (El-Nasr, Zupko, & Miron). Cite cautiously or verify against the full text of El-Nasr's broader research program (2003–2011). |
| Wilms & Oberfeld — "saturation as sole primary driver" | The paper reports saturation, brightness, and their interactions all drive arousal. Saturation is the strongest single factor but the "sole primary driver" framing overstates the finding. **Corrected in outline body** to "strongest single predictor." |
