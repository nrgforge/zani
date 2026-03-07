# Designing the Palette Collection
*2026-03-06*

## Abstract

This essay investigates three questions about Zani's palette collection: how many palettes to offer, what affective dimensions to organize them by, and how to name them. Research into color-emotion factor models (Ou et al., Kobayashi, Valdez & Mehrabian, Jonauskaite & Mohr 2025) reveals that the current three-value character axis (Warm, Cool, Vivid) is missing the low-saturation pole — "Muted" — which every major factor model identifies as a distinct affective register. Choice architecture research (Chernev et al. 2015, Sharma 2023, Mogilner et al. 2008) supports approximately five palettes per category across eight categories (~40 total), well within the comfort zone for a categorized, hedonic, keyboard-navigated browser. Psycholinguistic research on color naming (Lupyan 2012, Miller & Kahn 2005, Chou 2020) establishes that palette names are part of the mood instrument itself — labels modulate perception — and that a naming register drawn from Pacific Northwest flora provides traceable provenance, register consistency, and moderate schema congruence with affective categories, grounding evocative naming in curation rather than arbitrary vibes.

## The Missing Pole

Essay 002 established that saturation is the primary arousal lever and warmth drives approach-oriented positive affect. The current affective category taxonomy organizes palettes along two axes — brightness (Dark, Light) and character (Warm, Cool, Vivid) — producing six categories. This taxonomy emerged from the Wilms & Oberfeld (2018) findings and the El-Nasr emotional affordances framework, and it captures the space well in two of three dimensions.

The gap becomes visible when examining the three independent factor-analytic programs that have converged on the structure of color-emotion space:

| Program | Axis 1 | Axis 2 | Axis 3 |
|---------|--------|--------|--------|
| Ou et al. (2004) | Activity (chroma) | Weight (lightness) | Heat (hue) |
| Kobayashi (1981) | Clear/Grayish (chroma) | Soft/Hard (value) | Warm/Cool (hue) |
| Valdez & Mehrabian (1994) | Arousal (saturation) | Pleasure (brightness) | Dominance (brightness + saturation) |

All three identify a bipolar chroma/saturation axis. The current taxonomy captures the high pole — "Vivid" corresponds to Ou's "Active," Kobayashi's "Clear," and Valdez & Mehrabian's high-arousal register. But there is no category for the low pole: the muted, desaturated, grayish register that Kobayashi identifies with distinct image-word associations (*natural, refined, mature, quiet, nostalgic, subdued*) and that Suk & Irtel (2010) found drives emotional response more strongly than hue variation alone.

The Jonauskaite & Mohr (2025) systematic review — 132 studies, 42,266 participants, 64 countries — confirms this structure. Desaturated colors occupy a distinct position in the valence-arousal-power space: low arousal, low power, contemplative. This is not the absence of vividness but its own affective register, suitable for journaling, reflective writing, quiet literary fiction.

Adding "Muted" as a fourth character value produces eight categories:

| | Warm | Cool | Vivid | Muted |
|---|---|---|---|---|
| **Dark** | Intimate, sheltering | Intellectual, deep | Electric, energetic | Contemplative, subdued |
| **Light** | Gentle, morning | Crisp, alpine | Bright, solar | Airy, soft |

The weight/dominance dimension, which Valdez & Mehrabian identify as a third factor, does not require its own axis. It emerges from the combination of existing axes — Dark + Vivid produces maximum dominance; Light + Muted produces minimum — making it implicitly available without adding taxonomic complexity.

**Invariant tension:** The domain model currently defines Affective Category as "brightness (Dark, Light) and character (Warm, Cool, Vivid)" — three character values. Adding Muted requires amending this definition to four character values. ADR-009 anticipated this: "Adding a new category axis (e.g., 'Muted') requires a code change, not configuration."

## How Many Palettes

The instinct of approximately five palettes per category finds strong support in choice architecture research, though the answer depends on understanding why. The headline "paradox of choice" findings are more nuanced than popular accounts suggest.

Iyengar & Lepper (2000) found a 10x purchase rate difference between 6-option and 24-option jam displays. But Scheibehenne et al. (2010), meta-analyzing 63 conditions across 50 studies, found the mean effect size was virtually zero — high variance between studies masked any consistent direction. Chernev, Bockenholt, & Goodman (2015), with a larger and more methodologically sophisticated meta-analysis (99 observations, N = 7,202), resolved the contradiction: choice overload is real but moderated by four factors. When the choice set is easy to compare, the decision context is relaxed, the chooser has some preference clarity, and the goal is finding the right option rather than minimizing effort, overload is unlikely.

Palette selection in Zani scores well on all four moderators. Palettes share a common visual structure (easy to compare). There is no time pressure (relaxed context). The affective categories provide a preference scaffold even for users who lack strong prior opinions. And the task is experiential — finding a palette that *feels right* — not effort-minimizing.

Two additional findings shape the answer:

**Categorization provides its own relief.** Mogilner, Rudnick, & Iyengar (2008) found that the mere presence of categories increases both perceived variety and choice satisfaction, even when the categories carry no informational content. Meaningful categories — which Zani's affective groupings are — produce an even stronger effect. Sharma (2023) found that the "category ratio" (items per category label) matters more than total count, with chunking literature suggesting 3–7 items per group as the range where cognitive load is minimal.

**Hedonic choices are more tolerant.** Dhar & Wertenbroch (2000) found that experiential/hedonic choices tolerate larger assortments than utilitarian ones. Consumers with hedonic motivation actively seek larger choice sets because the browsing itself is part of the experience. Palette selection is fundamentally hedonic — a writer scrolling through palettes is engaging in aesthetic exploration, not checking a box.

The convergent recommendation: **approximately five palettes per category, across eight categories, producing a collection of roughly forty palettes.** Within-category counts of 3–7 are the sweet spot. Going below three risks under-stimulation and reduced perceived variety. Going above eight risks within-category overload, particularly for users with low preference certainty. The total of ~40 is conservative for a categorized hedonic browser — the research suggests a collection could go moderately larger without harm, but the constraint of hand-curation (each palette must be a carefully tuned mood instrument satisfying Invariant 3) provides a natural ceiling.

Yan et al. (2015) found that excessive categorization itself becomes a source of overload. Eight categories sits comfortably within the researched range; further fragmentation (splitting Muted into Muted-Warm and Muted-Cool, for instance) would risk over-categorization without clear benefit.

## Names as Part of the Instrument

The most surprising research finding concerns naming. The conventional view treats palette names as labels applied after the design work is done — nice to have, occasionally clever, fundamentally decorative. The psycholinguistic evidence says otherwise.

Lupyan's Label-Feedback Hypothesis (2012, 2013) demonstrates that linguistic labels actively modulate visual perception. Hearing or reading a label activates visual features corresponding to the labeled category, changing what a person sees at a perceptual level — not merely how they think about it. The effect is rapid (milliseconds), pervasive, and operates below conscious attention. Winawer et al. (2007) showed that language creates perceptual boundaries between colors: Russian speakers, whose language makes an obligatory distinction between light blue (*goluboy*) and dark blue (*siniy*), discriminate blues across this boundary faster than English speakers.

For a writing tool whose palettes are designed as mood instruments, this means the name is not separate from the instrument. When a writer selects "Ember" and begins work, the label is priming their visual system alongside the colors themselves. Two palettes with different names will feel more distinct than the same two palettes with similar names, even if the actual color distances are identical.

Skorinko et al. (2006) confirmed what anyone who has chosen paint colors already suspects: identical swatches rated with evocative names ("mocha") produce significantly more favorable evaluations than the same swatches with generic names ("brown"). This is not self-deception. It is how human cognition processes color in the presence of language.

### Evocative but Not Arbitrary

Miller & Kahn (2005) constructed a taxonomy of color names that clarifies which kind of evocation works:

| Type | Example | Performance |
|------|---------|-------------|
| Common | "Dark green" | Baseline |
| Common descriptive | "Pine green" | Good |
| Unexpected descriptive | "Kermit green" | Best when color is visible alongside name |
| Ambiguous | "Friendly green" | Requires cognitive effort; collapses under load |

The "unexpected descriptive" category — atypical but resolvable — produces the strongest positive response when the user can see the color alongside the name. The mechanism is a "search for meaning" that, when resolved, generates satisfaction. Seeing a warm reddish-brown palette labeled "Madrone" triggers a brief cognitive connection (madrone trees have smooth reddish-brown bark) whose resolution is pleasurable.

Chou (2020) extended this finding with a critical moderator: when color is a *secondary* attribute — setting a mood rather than being the primary decision factor — atypical names outperform descriptive ones. Palette selection in a writing app is precisely this context. The writer is choosing an atmosphere for their work, not evaluating a paint chip. "Ember" outperforms "Dark Warm Orange" because the naming register matches the task register.

### What Separates Curated from Chaotic

The paint industry has solved the problem of naming hundreds of colors. Farrow & Ball maintains ~132 colors, each with a name traceable to a specific origin — Stiffkey Blue from Norfolk beach mud, Dead Salmon from an 1805 painting invoice, Dimpse from a Southwest England dialect word for twilight. Critically, every name draws from the same world: English landscape, historical architecture, natural observation. The consistency of the naming register is itself the signal of curation.

By contrast, community-contributed theme collections (iTerm2-Color-Schemes lists 450+ themes with names ranging from "Blazer" to "Japanesque" to "CLRS") feel chaotic not because individual names are bad, but because the names come from unrelated worlds. There is no discernible mind behind the collection.

Schema congruity theory (Mandler; Meyers-Levy & Tybout 1989) provides the formal framework. Fully congruent names ("Warm Darkness" for a dark warm palette) are comfortable but forgettable — no search for meaning is triggered. Extremely incongruent names ("Peppermint" for a dark warm palette) produce confusion. The optimal zone is **moderate incongruence**: the name evokes the category's character through indirect association, producing a resolvable surprise. "Ember" for a dark warm palette works because embers are warm and dark — but the word activates a rich sensory image (glowing coals, fireplace, late night) rather than restating the category label.

The affective category system amplifies this effect. Because the category label ("Dark — Warm") handles the taxonomic communication, individual names are freed from descriptive duty entirely. The category says *what*; the name says *how it feels*.

## Pacific Northwest Flora as Naming Register

Zani's name derives from Manzanita (*Arctostaphylos*), a genus of Pacific Northwest shrubs and trees known for smooth, warm-toned bark in shades of mahogany, cinnamon, and deep red. Extending this botanical origin to the full palette collection — naming every palette after a PNW plant — provides the register consistency, traceable provenance, and moderate schema congruence that the research identifies as optimal.

The Pacific Northwest is one of the most botanically diverse regions in North America, spanning coastal rainforest, alpine meadow, old-growth canopy, sagebrush steppe, and high desert. This ecological range maps well to the eight affective categories:

**Dark-Warm** draws from trees with warm bark — Madrone (cinnamon exfoliating bark), Cascara (reddish-brown, mottled), Ponderosa (golden-brown plates that smell of vanilla in heat). The PNW is rich in warm-barked species, making this the most abundant category.

**Dark-Cool** draws from the deep-shade forest — Salal (leathery blue-green leaves, ubiquitous understory), Hemlock (cool dark canopy, Washington's state tree), Sitka (gray-purple bark, coastal fog belt). The cool, shaded PNW forest is the defining biome.

**Dark-Vivid** draws from high-saturation species against dark backgrounds — Salmonberry (vivid magenta flowers in deep forest; species name *spectabilis* means "spectacular"), Devil's Club (bright red berry clusters against old-growth understory), Oregon Grape (vivid yellow flowers, dusky purple berries). PNW forests are full of plants that pop against shadow.

**Dark-Muted** is the thinnest category but has exceptional individual candidates — Ghost Pipe (translucent, chlorophyll-free, turns black with age; growing on dark forest floor), Usnea (gray-green lichen threads draping from branches in fog forest), Alder (mottled ashy-gray bark colonized by white lichen). This category benefits from leaning into the stranger, less familiar species.

**Light-Warm** draws from meadow and early spring — Oceanspray (cascading white-to-cream flower plumes), Trillium (beloved spring wildflower, white aging to warm pink), Thimbleberry (large white flowers with pale yellow stamens, soft fuzzy leaves). Gentle, warm-light associations.

**Light-Cool** draws from alpine and coastal environments — Snowberry (crisp white berries persisting through frost), Lupine (blue-purple spikes in alpine meadows), Silver Fir (silvery-white needle undersides). Cool-toned, high-altitude or maritime.

**Light-Vivid** draws from wildflower meadows in full sun — Paintbrush (fiery scarlet bracts; *Castilleja*), Fireweed (vivid magenta spikes colonizing burned areas), Columbine (red-yellow bicolored flowers attracting hummingbirds). Summer at its most saturated.

**Light-Muted** draws from dried meadow and sagebrush steppe — Everlasting (pearly white papery bracts, silvery foliage; the name refers to how the flowers preserve when dried), Sagebrush (silvery leaves, aromatic, defining the eastern PNW landscape), Yarrow (soft feathery silvery-green foliage, cream flower clusters). Airy, gentle, desaturated.

Several plants serve as "multi-category assets" — their character shifts with season, light, or emphasis. Fireweed reads as Dark-Vivid (magenta against charred backgrounds) or Light-Vivid (summer meadow in full sun). Oceanspray reads as Light-Warm (fresh cream blooms) or Light-Muted (dried papery plumes). This flexibility is useful when assigning specific names to specific palettes.

### Why This Register Works

Evaluated against the five-point curation test derived from the naming research:

1. **Traceable:** Every name points to a real plant with a real appearance. "Madrone" refers to *Arbutus menziesii*, whose cinnamon bark is one of the most recognizable sights in PNW forests.

2. **Category-resonant:** Plant-color associations match category affect through indirect association. A user does not need to know what a madrone looks like to feel that the name belongs with a warm, dark palette — but a user who does know will experience the resolvable surprise that Miller & Kahn identify as optimal.

3. **Register-consistent:** All names come from the same world — Pacific Northwest botany. The collection sounds like it was assembled by a single sensibility, not accumulated from random contributions.

4. **Uniquely evocative:** Each plant activates a distinct sensory image. Within Dark-Warm, "Madrone" (smooth cinnamon bark, coastal hillside), "Cascara" (mottled brown, medicinal history), and "Ponderosa" (vanilla-scented plates, eastern sun) prime different versions of warmth.

5. **Resolvable:** Seeing a warm reddish-brown palette labeled "Madrone" clicks. Seeing a pale silvery palette labeled "Sagebrush" clicks. The connection does not require botanical expertise — the sensory associations travel with the names.

The register also connects deeply to the application's identity. Zani is named after a plant. Its palettes are named after that plant's neighbors in the same ecosystem. The coherence is not imposed — it emerges from the origin.

### The Naming Constraint as Quality Control

A naming register imposes a productive constraint on palette design. A palette cannot be added to the collection unless a PNW plant exists whose color associations are moderately congruent with the palette's affect. This prevents the collection from expanding beyond what curation can sustain. If a proposed Dark-Muted palette has no natural plant counterpart, that is a signal — either the palette is too similar to an existing one (and the plant name is already taken) or the palette sits outside the affective space the collection is designed to cover.

Pojar & MacKinnon's *Plants of the Pacific Northwest Coast* (794 species) and the broader PNW ethnobotanical literature provide a deep well of candidates. At five palettes per category, the collection requires ~40 names from a pool of hundreds of named species. The constraint is real but not restrictive.

## Recommended Collection Structure

The research converges on a specific collection design:

**Taxonomy:** 2 brightness values (Dark, Light) × 4 character values (Warm, Cool, Vivid, Muted) = 8 affective categories.

**Density:** Approximately 5 palettes per category, producing a collection of ~40 palettes. The floor is 3 per category (below which perceived variety suffers); the ceiling is ~7 (above which within-category overload risks emerge). The exact count per category need not be uniform — some categories may have 4, others 6, depending on how many distinct palettes can be designed that satisfy Invariant 3 and carry genuinely different character.

**Naming:** Every palette named after a Pacific Northwest plant whose natural color associations are moderately congruent with its affective category. Single-word names preferred (Madrone, Salal, Cascara, Hemlock, Yarrow, Lupine). Two-word names acceptable when the plant demands it (Ghost Pipe, Sword Fern, Silver Fir). Names drawn from Pojar & MacKinnon and the broader PNW ethnobotanical literature.

**Ordering:** Within each category, palettes continue to sort by OKLCH hue angle of the background (Perceptual Sort Order), producing smooth browsing from palette to palette.

**Browser interaction:** The Palette Browser's two-level architecture (category → palette) means users face ~8 category headers and ~5 palettes per group, never the full ~40 at once. This is well within the research-supported range for categorized hedonic browsing.

This structure scales the current 10-palette collection to ~40 while preserving the curated, hand-tuned character that distinguishes Zani's approach from community theme aggregations. Each palette is a mood instrument; each name is part of that instrument; and the collection is organized to make browsing itself a pleasure rather than a chore.
