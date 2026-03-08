# Research Log: Palette Collection Design

## Question 1: What affective dimensions of color have robust empirical support beyond Warm/Cool/Vivid?

**Method:** Web search — color psychology meta-analyses, factor analysis models, systematic reviews

**Findings:**

Three independent research programs converge on a three-dimensional model of color-emotion space:

| Program | Dimension 1 | Dimension 2 | Dimension 3 |
|---------|------------|------------|------------|
| Ou et al. (2004) | Colour Activity (chroma) | Colour Weight (lightness) | Colour Heat (hue) |
| Kobayashi (1981) | Clear/Grayish (chroma) | Soft/Hard (value) | Warm/Cool (hue) |
| Valdez & Mehrabian (1994) | Arousal (saturation-dominant) | Pleasure (brightness-dominant) | Dominance (brightness+saturation) |

The Jonauskaite & Mohr (2025) systematic review (132 studies, 42,266 participants, 64 countries) confirms three reliable dimensions:
- **Valence** — driven by lightness (light = positive, dark = negative)
- **Arousal** — driven by saturation (high = aroused, low = calm)
- **Power/Potency** — driven by lightness inversely + saturation

**The gap in Zani's taxonomy:** Every model identifies a bipolar Activity/Chroma axis. Zani's "Vivid" captures the high end (high saturation = active, aroused, energetic). But there is no category for the low end — **muted, desaturated, grayish colors**. This is not merely "not vivid." It is a distinct affective register:
- Kobayashi explicitly separates "Grayish" as a pole with its own image-word associations: natural, refined, mature, quiet, nostalgic, subdued
- Suk & Irtel (2010): "Emotional responses to color vary more strongly with regard to tone than to hue categories"
- Desaturated colors = low arousal, low power, contemplative — suitable for journaling, reflective writing, quiet literary fiction

A secondary question: achromatic/neutral palettes. Grey and near-neutral colors constitute a distinct affective register (Jonauskaite: grey = sadness, boredom, disappointment in isolated patches; but in context, light grays = soothing, medium grays = formal/restrained). However, this is arguably the extreme end of "Muted" rather than a separate dimension.

Weight/Dominance is real but already implicitly captured by Dark × Vivid (high dominance) and Light × Warm (low dominance). No new axis needed.

**Implications:**
- Add "Muted" as a fourth character value alongside Warm, Cool, Vivid
- This produces 8 categories (2 brightness × 4 character), filling the missing pole of the Activity/Chroma axis
- Achromatic palettes can live under DarkMuted/LightMuted rather than warranting their own axis
- The Weight/Dominance dimension is covered by existing axis combinations

**Key sources:**
- Ou et al. (2004) — Color Research and Application
- Kobayashi (1981) — Color Research and Application
- Valdez & Mehrabian (1994) — Journal of Experimental Psychology: General
- Palmer & Schloss (2010) — PNAS
- Suk & Irtel (2010) — Color Research and Application
- Jonauskaite & Mohr (2025) — Psychonomic Bulletin & Review (systematic review, 132 studies)
- Wilms & Oberfeld (2018) — Psychological Research

---

## Question 2: What does choice architecture research say about the optimal number of options in a constrained selection UI?

**Method:** Web search — choice overload meta-analyses, categorized choice sets, hedonic vs. utilitarian choices

**Findings:**

### The choice overload landscape

The Iyengar & Lepper (2000) jam study found 30% purchase rate with 6 options vs. 3% with 24. But Scheibehenne et al. (2010) meta-analysis of 63 conditions found the mean effect size was virtually zero (d ≈ 0.02) — high variance between studies masks the effect.

Chernev et al. (2015) resolved this with a larger meta-analysis (99 observations, N = 7,202): overload IS real but moderated by four factors:

| Moderator | Overload more likely | Overload less likely |
|-----------|---------------------|---------------------|
| Choice set complexity | Options dissimilar, non-alignable | Options share structure, easy to compare |
| Decision task difficulty | Time pressure, complex info | Relaxed, simple display |
| Preference uncertainty | Chooser lacks prior preferences | Chooser knows what they want |
| Decision goal | Effort-minimizing | Accuracy-maximizing |

### Optimal numbers from experimental data

| Study | Peak/recommended range | Domain |
|-------|----------------------|--------|
| Shah & Wolford (2007) | Peak at 10 (uncategorized) | Consumer goods (pens) |
| Reutskaja & Hogarth (2009) | 10-15 (Western Europeans) | Gift boxes |
| Hick's Law | Logarithmic cost — diminishing marginal burden up to ~15 | Reaction time |

### Categorization provides relief

Mogilner, Rudnick, & Iyengar (2008): The mere presence of categories — even meaningless ones — increases perceived variety AND satisfaction. Meaningful categories (like affective categories) are even better. Effect is strongest for domain novices.

Sharma (2023): The "category ratio" (items per category label) matters more than total count. Optimal ratio draws on chunking literature: ~3-7 per group.

Yan et al. (2015): Too many categories itself becomes overload. Optimal number of categories exists; excessive fragmentation hurts.

Besedes et al. (2015): Tournament-style sequential architecture (evaluate subsets, compare winners) improves decision quality. Zani's category → palette two-level navigation mirrors this.

### Hedonic choices are more tolerant

Dhar & Wertenbroch (2000): Hedonic choices tolerate larger assortments. Consumers with hedonic motivation seek larger choice sets; browsing itself is part of the experience. Palette selection is fundamentally hedonic.

### Real-world theme ecosystems

- VS Code: 5,000+ in marketplace, ~15 built-in, "best of" lists feature 15-25
- iTerm2-Color-Schemes: 450+ uncategorized — acknowledged as overwhelming
- macOS: 2 (Light/Dark) — extreme minimal
- Slack: ~10 presets

**Implications:**
- ~5 palettes per category across ~6-8 categories (~30-40 total) is well-supported
- Zani's two-level navigation means users never face the full set at once
- Within-category count of 3-7 is the sweet spot (Sharma category ratio + chunking)
- Hedonic browsing raises the tolerance ceiling above the jam-study numbers
- Going above 8-10 per category risks within-category overload
- Going below 3 per category risks under-stimulation and reduced perceived variety
- Cap categories at ~6-8 to avoid over-categorization (Yan et al.)

---

## Question 3: How do color names influence perception and preference, and what naming strategy avoids "just vibes" while preserving evocative power?

**Method:** Web search — psycholinguistic research on color naming, paint/cosmetics industry naming practices, software theme naming patterns, schema congruence theory

**Findings:**

### Names genuinely change perception

Lupyan's Label-Feedback Hypothesis (Frontiers in Psychology, 2012; PNAS, 2013): Linguistic labels do not merely describe perception — they actively modulate it. Hearing or reading a label activates visual features corresponding to the labeled category, changing what you see at a perceptual level, not just a semantic one.

Winawer et al. (PNAS, 2007): Russian speakers, whose language obligatorily distinguishes light blue (goluboy) from dark blue (siniy), discriminate blues across this boundary faster than English speakers. Language creates perceptual boundaries.

Skorinko et al. (Psychology & Marketing, 2006): Identical color swatches with "fancy" names (e.g., "mocha") rated significantly more favorably than with generic names (e.g., "brown"). The user's observation about paint names influencing perception more than the color itself is experimentally validated.

### The Miller & Kahn taxonomy of color names (2005)

Four name types, tested experimentally (Journal of Consumer Research):

| Type | Definition | Example | Performance |
|------|-----------|---------|-------------|
| Common | Typical, unspecific | Dark green | Baseline |
| Common Descriptive | Typical, specific | Pine green | Good |
| Unexpected Descriptive | Atypical, resolvable | Kermit green | Best (when color is visible) |
| Ambiguous | Atypical, unresolvable | Friendly green | Best without distraction, collapses under cognitive load |

The "unexpected descriptive" sweet spot: names that trigger a "search for meaning" which, when resolved (user sees palette alongside name), produces positive affect. The resolution itself is pleasurable.

Chou (Psychology & Marketing, 2020) extended this: when color is a **secondary attribute** (setting mood, not the primary decision factor), atypical names outperform descriptive ones. Palette selection is exactly this context — color sets a mood for writing, not the point itself.

### What separates curated from chaotic naming

Farrow & Ball (~132 colors) vs. Benjamin Moore (~3,500): Every Farrow & Ball name has a traceable provenance — place names (Stiffkey Blue), historical references (Dead Salmon from an 1805 invoice), dialect words (Dimpse = twilight in SW England). Names are not arbitrary; they point to something real. The obscurity is a feature, inviting the search-for-meaning that Miller & Kahn identified.

**The critical differentiator is naming register consistency.** Catppuccin's coffee metaphor, Nord's Arctic metaphor, and Farrow & Ball's English-countryside register all work because they are internally consistent. Community theme grab-bags (iTerm2-Color-Schemes: 450+ themes with names from "Blazer" to "Japanesque") fail because names don't come from the same world.

### Schema congruence: the inverted-U

Mandler's schema congruity theory, applied to naming:
- Fully congruent: "Warm Darkness" for a Dark-Warm palette → comfortable but forgettable
- Moderately incongruent: "Ember" for a Dark-Warm palette → resolvable surprise, positive affect, stronger memory
- Extremely incongruent: "Peppermint" for a Dark-Warm palette → confusion, negative affect

**Optimal: names should rhyme with their category, not restate it.** "Ember" works because embers are warm and dark — but the word activates a rich sensory image rather than merely restating the category label.

### Categories free names to be expressive

When affective categories handle the structural communication ("this is dark and warm"), individual names don't need to be descriptive. The category label does the taxonomic work, freeing the name to do the evocative work. This is directly supported by categorical cognition research on reduced cognitive load.

### The "not just vibes" test

A name passes curation scrutiny if:
1. **Traceable** — points to a specific referent ("embers are the last warm glow of a dying fire")
2. **Category-resonant** — affect matches category without restating it (moderate schema congruence)
3. **Register-consistent** — sounds like it belongs with the other names in the collection
4. **Uniquely evocative** — activates a distinct sensory image from siblings in the same category
5. **Resolvable** — seeing the palette alongside the name, the connection clicks

**Implications:**
- Zani's current names (Ember, Inkwell, Glacier, etc.) already follow the optimal pattern — unexpected-descriptive, category-resonant, traceable
- At 40 palettes, the challenge is maintaining register consistency and distinct evocation within categories
- Loose thematic families within categories (hearth/fire for Dark-Warm, mineral/water for Dark-Cool) help without being rigid
- The naming register should be consistent across the whole collection — concrete-evocative drawn from natural/material/atmospheric phenomena
- Names that evoke *situations* (writing by firelight, working in a study) may be especially effective (consumption-situation imagery, European Journal of Marketing, 2024)

**Key sources:**
- Lupyan (2012) — Frontiers in Psychology; Lupyan & Ward (2013) — PNAS
- Winawer et al. (2007) — PNAS
- Skorinko et al. (2006) — Psychology & Marketing
- Miller & Kahn (2005) — Journal of Consumer Research
- Chou (2020) — Psychology & Marketing
- Mandler — Schema congruity theory (applied via Meyers-Levy & Tybout, 1989)
- Farrow & Ball naming practices — Slate (2013), Domino interview with Joa Studholme

---

## Question 4: Can Pacific Northwest plants fill all 8 affective categories with ~5 palettes each?

**Method:** Web search — PNW botany, ethnobotany, field guide cross-referencing, plant color associations by habitat

**Findings:**

The PNW plant kingdom maps well to the affective categories. The app name "Zani" derives from Manzanita, grounding the entire naming register in PNW flora. Category-by-category assessment:

### Category strength assessment

| Category | Strength | Top candidates | Notes |
|----------|----------|---------------|-------|
| Dark-Warm | Very strong | Madrone, Cascara, Ponderosa, Vine Maple, Ninebark, Douglas Fir | PNW is rich in warm-barked trees |
| Dark-Cool | Strong | Salal, Hemlock, Sitka, Camas, Sword Fern, Silver Fir | Cool deep-shade forest provides many candidates |
| Dark-Vivid | Very strong | Salmonberry, Fireweed, Devil's Club, Oregon Grape, Paintbrush, Huckleberry | Saturated berries/flowers against dark forest |
| Dark-Muted | Thinnest | Ghost Pipe, Usnea, Alder, Licorice Fern, Snowberry | Fog/lichen aesthetic is hard to pin to a single plant name. Ghost Pipe and Usnea are the standouts. |
| Light-Warm | Strong | Oceanspray, Trillium, Thimbleberry, Yarrow, Rabbitbrush | Good diversity of warm-light plants |
| Light-Cool | Adequate | Snowberry, Silver Fir, Lupine, Avalanche Lily, Elderberry | Alpine zone provides options; some two-word names |
| Light-Vivid | Very strong | Fireweed, Paintbrush, Lupine, Columbine, Bunchberry, Oregon Grape | PNW wildflower meadows are vivid |
| Light-Muted | Adequate | Everlasting, Sagebrush, Yarrow, Oceanspray (dried), Prairie Sage | Sagebrush steppe and dried-meadow aesthetic |

### Key observations

**Multi-category plants are an asset, not a problem.** Several plants (Fireweed, Camas, Lupine, Oceanspray, Yarrow, Snowberry) could plausibly serve in multiple categories depending on which aspect of the plant is emphasized. This provides flexibility — the same plant in different seasons or light conditions reads differently.

**Dark-Muted is the thinnest category** but solvable. Ghost Pipe is an exceptional fit (translucent, eerie, deep-shade, turns black with age). Usnea (old man's beard lichen) captures the fog-forest aesthetic. The category benefits from leaning into the stranger, more unusual plants.

**Name length matters.** The best candidates are 1-2 syllable single words (Salal, Madrone, Cascara, Hemlock, Sitka, Yarrow). Multi-word plant names (Avalanche Lily, Pearly Everlasting, Sword Fern) can be shortened but lose specificity.

**Standout names per category:**
- Dark-Warm: Madrone (iconic warm bark), Cascara (pleasant sound), Ponderosa (vanilla-scented warm bark)
- Dark-Cool: Salal (ubiquitous, short, distinctly PNW), Sitka (cool, coastal), Hemlock (deep shade)
- Dark-Vivid: Salmonberry (magenta flowers, orange berries), Fireweed (vivid magenta, fire association)
- Dark-Muted: Ghost Pipe (translucent, contemplative), Usnea (gray-green lichen threads)
- Light-Warm: Oceanspray (cream cascades), Trillium (beloved spring wildflower), Yarrow (soft, feathery)
- Light-Cool: Snowberry (crisp white), Lupine (blue-purple, alpine)
- Light-Vivid: Paintbrush (fiery scarlet, artistic), Columbine (red-yellow, hummingbird-attracting)
- Light-Muted: Everlasting (pearly, dried, papery), Sagebrush (silvery, aromatic)

### Reference resources
- "Plants of the Pacific Northwest Coast" by Pojar & MacKinnon — definitive PNW field guide (794 species)
- "Ethnobotany of Western Washington" by Erna Gunther — classic academic ethnobotany
- "Wildflowers of the Pacific Northwest" by Turner & Gustafson — organized by habitat with color photography
- Native American Ethnobotanical Database (University of Michigan) — searchable traditional plant uses

**Implications:**
- The PNW plant register is viable for all 8 categories, with 5+ candidates per category
- Dark-Muted will need the most creative sourcing but has strong individual candidates
- Several plants provide flexibility across categories — useful when assigning names to specific palettes
- Single-word names should be prioritized (Madrone, Salal, Cascara, Hemlock, Sitka, Yarrow, Lupine, Columbine)
- The naming register connects deeply to the app's identity (Zani = Manzanita) and passes every curation test from Q3
