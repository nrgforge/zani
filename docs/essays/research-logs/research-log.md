# Research Log: Species-Color Validation

## Question 1: Do the 40 palette color profiles holistically match the botanical colors of their namesake species?

**Method:** Internal audit — compared each palette's RGB color values (interpreted as plain color descriptions) against the species' provenance description (which documents the real botanical appearance). Scored each on a 4-point scale: A (strong match), B (reasonable match), C (weak match), D (mismatch).

**Findings:**

### Score distribution

- **A (Strong match):** 28 palettes (70%)
- **B (Reasonable match):** 10 palettes (25%)
- **C (Weak match):** 2 palettes (5%)
- **D (Mismatch):** 0 palettes (0%)

### Full audit

| # | Palette | Category | Score | Notes |
|---|---------|----------|-------|-------|
| 1 | Manzanita | Dark Warm | A | Dark reddish-brown bg + warm peach/salmon accents = mahogany-to-cinnamon bark |
| 2 | Chinquapin | Dark Warm | A | Dark olive-brown bg + warm gold accents = golden leaf-scale undersides |
| 3 | Red Cedar | Dark Warm | A | Dark reddish-brown bg + warm amber accents = amber-to-cinnamon heartwood |
| 4 | Chanterelle | Dark Warm | A | Warm orange-gold heading + golden yellow emphasis = apricot-orange caps |
| 5 | Madrone | Dark Warm | A | Terra-cotta heading + rust emphasis = peeling bark signature |
| 6 | Sitka | Dark Cool | A | Dark blue-gray bg + steel blue heading = fog-belt blue-green needles |
| 7 | Oakmoss | Dark Cool | A | Teal-green heading + emphasis = muted teal-green thallus |
| 8 | Oregon Grape | Dark Cool | A | Navy bg (berries) + periwinkle heading (bloom) + golden emphasis (flowers) + sage link (foliage) — captures all three signature colors |
| 9 | Elderberry | Dark Cool | A | Dark purple bg + lavender heading + periwinkle emphasis = powder-blue berries with waxy bloom |
| 10 | Witch's Hair | Dark Cool | B | Right color family (olive-green) but palette is brighter/lighter than the actual olive-black to dark greenish-brown thallus |
| 11 | Fly Agaric | Dark Vivid | A | Dark red bg + double vivid red accents = unmistakable red cap |
| 12 | Lobaria | Dark Vivid | A | Double vivid green accents = saturated bright green when wet |
| 13 | Jack-o'-Lantern | Dark Vivid | A | Vivid orange heading + chartreuse emphasis (bioluminescent glow) |
| 14 | Violet Cort | Dark Vivid | A | Three purple slots from dark to vivid = entirely deep violet mushroom |
| 15 | Salal | Dark Vivid | A | Dark blue-purple bg + vivid blue-violet accents = near-black saturated berries |
| 16 | Usnea | Dark Muted | A | Gray-green-neutral throughout = silvery-green pendant lichen |
| 17 | Alder | Dark Muted | B | Warm gray tones match pale gray bark; "white lichen patches" not specifically captured |
| 18 | Douglas Fir | Dark Muted | B | Warm brown-gray bg + warm tan heading match bark; teal-green emphasis is needles (provenance focuses on bark) |
| 19 | Sagebrush | Dark Muted | A | Sage green heading (literally sage) + gray-green throughout = silver-green foliage |
| 20 | Map Lichen | Dark Muted | A | Olive-gold heading + chartreuse-gold emphasis = chartreuse-yellow patches on rock |
| 21 | Oatgrass | Light Warm | A | Warm straw/cream bg + warm brown-gold accents = golden straw bunchgrass |
| 22 | Oregon Sunshine | Light Warm | **C** | **Bright yellow flowers missing; pale mint-green bg contradicts dry rocky habitat and yellow blooms** |
| 23 | White Oak | Light Warm | B | Warm peach/buff bg is slightly too peachy vs. described light gray to buff bark |
| 24 | Ponderosa | Light Warm | A | Warm butterscotch bg + warm amber accents = butterscotch and vanilla bark plates |
| 25 | Balsamroot | Light Warm | **C** | **Same problem as Oregon Sunshine: mint-green bg, amber-gold flowers rendered as muted olive tones** |
| 26 | Cascade Aster | Light Cool | A | Pale lavender bg + deep purple accents = lavender to pale violet ray flowers |
| 27 | Pearly Everlasting | Light Cool | B | Pale cool gray-blue bg captures pearly quality; steel blue heading is a slight stretch from white/cream bracts |
| 28 | Partridgefoot | Light Cool | B | Cool gray-green bg + teal-green accents match foliage; creamy racemes not represented |
| 29 | Lupine | Light Cool | A | Pale blue-lavender bg + deep blue-violet accents = blue to blue-violet flower spires |
| 30 | Phlox | Light Cool | B | Right pink color family but heading/emphasis more saturated than described pale pink to near-white |
| 31 | Paintbrush | Light Vivid | B | Vivid red heading nails scarlet bracts; green emphasis is complementary contrast rather than botanical |
| 32 | Columbine | Light Vivid | B | Strong red-orange heading; yellow petals rendered as olive-gold rather than true yellow |
| 33 | Tiger Lily | Light Vivid | A | Vivid burnt orange heading + dark maroon-brown emphasis = orange tepals spotted in maroon |
| 34 | Farewell | Light Vivid | A | Double vivid magenta-pink accents = vivid pink to magenta flowers |
| 35 | Camas | Light Vivid | A | Double vivid blue-violet accents = bright blue-violet star-shaped flowers |
| 36 | Sword Fern | Light Muted | A | Pale cool green bg + double medium green accents = evergreen fronds |
| 37 | Oceanspray | Light Muted | A | Cream bg (fresh flowers) + warm brown accents (dried parchment) = both states |
| 38 | Goatsbeard | Light Muted | B | Captures quiet woodland character but almost too neutral; white plume quality could be stronger |
| 39 | Fringecup | Light Muted | A | Pale warm pink bg + muted rose accents = flowers aging from white to soft pink |
| 40 | Reindeer Lichen | Light Muted | A | Pale cool blue-gray bg + muted green accents = silvery-green to gray cushions |

### Systematic pattern in the C scores

Both C-scored palettes share the same problem:

1. **Oregon Sunshine** (#22) and **Balsamroot** (#25) are both Light-Warm wildflowers with bright gold/yellow blooms.
2. Both have mint-green backgrounds — `Rgb(230, 242, 205)` and `Rgb(218, 242, 210)` — rather than warm straw/gold/cream.
3. Both have their gold flower colors muted to olive-brown rather than true warm yellow.
4. The mint-green backgrounds place them visually closer to Light-Cool palettes than Light-Warm.

The likely cause: these backgrounds were shifted green to maintain the 15° OKLCH hue diversity requirement against neighboring palettes. Oatgrass (#21) occupies the warm straw hue, and Ponderosa (#24) occupies the warm peach/butterscotch hue, leaving Oregon Sunshine and Balsamroot pushed toward green to maintain separation.

This is exactly the scenario the user identified — the constraint-satisfaction process pushed colors away from botanical truth for two species whose defining trait is bright yellow/gold.

### B-score patterns

The 10 B scores cluster into recognizable patterns:

- **Tone mismatch** (3): Witch's Hair is brighter than real; Phlox heading is more saturated than real; Goatsbeard is more neutral than distinctive
- **Secondary feature gap** (3): Alder's white patches, Partridgefoot's creamy racemes, Columbine's yellow petals — all rendered as olive/muted rather than their true color
- **Mild hue drift** (2): White Oak bg is more peach than gray-buff; Pearly Everlasting heading is steel blue rather than white/cream
- **Design choice** (2): Paintbrush uses green emphasis for contrast; Douglas Fir includes needle colors when provenance focuses on bark

None of the B scores warrant species replacement — they represent reasonable artistic license within the "moderate congruence" standard.

**Implications:**

- The collection is in strong shape overall: 95% of palettes score A or B.
- Two palettes need species replacement: Oregon Sunshine (#22) and Balsamroot (#25).
- Both sit in the Light-Warm category, both have mint-green backgrounds, and both are named after bright-gold wildflowers.
- The fix is to find Cascadia species whose real botanical colors match a "pale green with warm brown/olive accents" impression.
- The replacement species should NOT be bright-yellow wildflowers — the palettes aren't yellow, so the replacement should match what the palettes actually are.
- **Limitation:** This audit compared palette colors against our own provenance descriptions — a circular reference. An external validation against authoritative botanical sources is needed to ground scores in reality.

---

## Question 2: Do the source-backed botanical colors confirm or revise the self-assessment scores?

**Method:** External validation — searched Oregon Flora Project, USDA PLANTS Database, USFS Silvics/FEIS, Burke Herbarium, Calflora, iNaturalist, and regional field guides for each species' documented colors. Also validated geographic range against the Cascadia bioregion (coastal BC through northern California, east to Cascades crest). Eight parallel research agents, one per affective category.

**Findings:**

### Source-backed score distribution

- **A (Strong match):** 28 palettes (70%)
- **B (Reasonable match):** 12 palettes (30%)
- **C (Weak match):** 0 palettes (0%)
- **D (Mismatch):** 0 palettes (0%)

### Changes from self-assessment

The source-backed validation revised 10 scores — 6 upgrades and 4 downgrades:

| # | Palette | Self → Source | Reason |
|---|---------|---------------|--------|
| 7 | Oakmoss | A → **B** | Real *Evernia prunastri* is gray-green to olive, not teal; palette's teal heading overshifted |
| 9 | Elderberry | A → **B** | Real *Sambucus caerulea* berries are powder-blue with waxy bloom; palette leans too purple |
| 15 | Salal | A → **B** | Real berries are near-black; palette's vivid blue-violet accents are brighter than life |
| 40 | Reindeer Lichen | A → **B** | Real *Cladonia rangiferina* is predominantly gray/silver-white; palette overemphasizes green |
| 18 | Douglas Fir | B → **A** | Sources confirm bark IS warm brown-gray; teal-green emphasis legitimately represents needle color |
| 22 | Oregon Sunshine | C → **B** | Sources reveal gray-green woolly foliage that connects to mint-green bg; amber accents evoke late-season tones |
| 25 | Balsamroot | C → **B** | Sources confirm silvery gray-green foliage matching bg; amber tones are muted but recognizable |
| 30 | Phlox | B → **A** | Sources confirm *P. diffusa* ranges from pale pink to bright pink-magenta; palette saturation is within natural range |
| 31 | Paintbrush | B → **A** | Sources confirm bracts ARE vivid scarlet; green emphasis represents the actual green calyx and stem |
| 32 | Columbine | B → **A** | Sources confirm red-to-orange spurs and yellow petal tips; olive-gold reads as warm secondary |

The self-assessment's two C scores (Oregon Sunshine, Balsamroot) upgraded to B because external sources documented gray-green woolly foliage that the self-assessment overlooked — the mint-green backgrounds do connect to the species, just not to their most iconic feature (bright yellow flowers). Four self-assessed A's downgraded because sources revealed the palette colors are shifted further from reality than our own provenance descriptions suggested.

**Net result:** The same 28 A's, but a different 28. The B pool expanded from 10 to 12, absorbing the 2 former C's.

### Full source-backed audit

| # | Palette | Category | Color | Geo | Key finding |
|---|---------|----------|-------|-----|-------------|
| 1 | Manzanita | Dark Warm | A | Borderline | Genus *Arctostaphylos* is Cascadian; *A. manzanita* species range is primarily Californian |
| 2 | Chinquapin | Dark Warm | A | Yes | Golden leaf-scale undersides confirmed by OregonFlora and USFS |
| 3 | Red Cedar | Dark Warm | A | Yes | Warm reddish heartwood and fibrous cinnamon bark confirmed |
| 4 | Chanterelle | Dark Warm | A | Yes | Apricot-orange to egg-yolk yellow caps confirmed by MycoMatch and field guides |
| 5 | Madrone | Dark Warm | A | Yes | Peeling cinnamon-to-terra-cotta bark extensively documented |
| 6 | Sitka | Dark Cool | A | Yes | Blue-green to silvery needles, gray-purple bark confirmed by USFS Silvics |
| 7 | Oakmoss | Dark Cool | B | Yes | Real thallus is gray-green to olive; palette's teal is overshifted. Provenance says "teal" — should be "gray-green" |
| 8 | Oregon Grape | Dark Cool | A | Yes | Navy berries, yellow flowers, sage-green foliage — all three confirmed |
| 9 | Elderberry | Dark Cool | B | Yes | Real *S. caerulea* berries are powder-blue with waxy bloom, not deep purple; palette leans too violet |
| 10 | Witch's Hair | Dark Cool | B | Yes | **Provenance error:** says "olive-black to dark greenish-brown thallus" but *Alectoria sarmentosa* is actually pale yellow-green to straw. May be confusing with *Bryoria* |
| 11 | Fly Agaric | Dark Vivid | A | Yes | Red cap with white warts confirmed by every mycological source |
| 12 | Lobaria | Dark Vivid | A | Yes | Saturated bright green when wet confirmed; brown when dry |
| 13 | Jack-o'-Lantern | Dark Vivid | A | Borderline | **Species name error:** provenance uses *Omphalotus olearius* (European). Western species is *O. olivascens*. Primarily California/Oregon but documented as far north as BC |
| 14 | Violet Cort | Dark Vivid | A | Yes | Entirely deep violet confirmed for *Cortinarius violaceus* |
| 15 | Salal | Dark Vivid | B | Yes | Real berries are near-black, darker than palette's vivid blue-violet accents suggest |
| 16 | Usnea | Dark Muted | A | Yes | Pale gray-green to yellow-green pendant lichen confirmed by McCune & Geiser |
| 17 | Alder | Dark Muted | B | Yes | Real bark is pale gray to whitish with white lichen patches; palette is warmer than actual |
| 18 | Douglas Fir | Dark Muted | A | Yes | Bark warm brown-gray confirmed; needle blue-green provides secondary color reference |
| 19 | Sagebrush | Dark Muted | A | Yes | Silver-green aromatic foliage confirmed; sage-green heading is literal |
| 20 | Map Lichen | Dark Muted | A | Yes | Chartreuse-yellow to yellow-green areoles on rock confirmed |
| 21 | Oatgrass | Light Warm | A | Yes | Golden straw by midsummer confirmed; warm bg matches dried prairie bunchgrass |
| 22 | Oregon Sunshine | Light Warm | B | Yes | Bright golden-yellow flowers (iconic) + gray-green woolly foliage (matching bg). Palette captures foliage but misses most recognizable feature |
| 23 | White Oak | Light Warm | B | Yes | Bark is light gray to silver-gray per USFS; palette's peach/buff bg is warmer. Autumn foliage and acorn tones provide secondary connection |
| 24 | Ponderosa | Light Warm | A | Yes | Butterscotch/amber bark plates extensively documented. One of the strongest matches |
| 25 | Balsamroot | Light Warm | B | Yes | Bright golden-yellow flowers + silvery gray-green foliage. Same pattern as Oregon Sunshine: bg captures foliage, accents mute the vivid yellow. **Provenance error:** says "from the Rogue Valley" but species is primarily east of Cascades |
| 26 | Cascade Aster | Light Cool | A | Yes | Lavender to pale violet ray flowers confirmed |
| 27 | Pearly Everlasting | Light Cool | B | Yes | Bracts are white to cream, not steel blue; palette heading is a stretch |
| 28 | Partridgefoot | Light Cool | B | Yes | **Provenance error:** foliage is "glossy green" per USDA, not "gray-green" as provenance claims |
| 29 | Lupine | Light Cool | A | Yes | Blue to blue-violet flower spires confirmed. **Provenance note:** claims silvery sheen but *L. latifolius* foliage is specifically NOT silvery (unlike *L. argenteus*) |
| 30 | Phlox | Light Cool | A | Yes | *P. diffusa* color range includes bright pink-magenta; palette saturation within natural range |
| 31 | Paintbrush | Light Vivid | A | Yes | Vivid scarlet bracts confirmed; green emphasis represents actual green calyx |
| 32 | Columbine | Light Vivid | A | Yes | Red-orange spurs + yellow petal tips confirmed for *Aquilegia formosa* |
| 33 | Tiger Lily | Light Vivid | A | Yes | Orange tepals with maroon spots confirmed |
| 34 | Farewell | Light Vivid | A | Yes | Vivid pink-to-magenta flowers confirmed for *Clarkia amoena* |
| 35 | Camas | Light Vivid | A | Yes | Bright blue-violet star-shaped flowers confirmed |
| 36 | Sword Fern | Light Muted | A | Yes | Evergreen fronds in deep green confirmed |
| 37 | Oceanspray | Light Muted | A | Yes | Cream fresh flowers aging to brown parchment — both states captured |
| 38 | Goatsbeard | Light Muted | B | Yes | Captures quiet woodland character; colors are atmospheric interpretation rather than directly botanical |
| 39 | Fringecup | Light Muted | A | Yes | Flowers aging from white to soft pink confirmed |
| 40 | Reindeer Lichen | Light Muted | B | Borderline | Real *Cladonia rangiferina* is predominantly silvery-gray to white, not green. Palette overemphasizes green. **Geographic:** circumboreal species, uncommon in PNW per McCune & Geiser. **Provenance error:** claims "east-slope Cascades" but species is actually rare on east side |

### Geographic flags

Three species have borderline Cascadia bioregion status:

1. **Manzanita (#1):** ~~Borderline~~ **Confirmed Cascadian.** While *A. manzanita* sensu stricto is primarily Californian, *Arctostaphylos* is well-represented throughout the Siskiyou-Klamath region and the Rogue Valley. User confirms manzanita presence in the Rogue Valley from direct observation. The genus is unambiguously Cascadian.

2. **Jack-o'-Lantern (#13):** *Omphalotus olivascens* (the correct western species) is primarily Californian/Oregonian but has been documented as far north as BC. Borderline but defensible.

3. **Reindeer Lichen (#40):** *Cladonia rangiferina* is a circumboreal species documented as uncommon in the Pacific Northwest by McCune & Geiser (2009). The provenance claim of "east-slope Cascades" contradicts source literature. The species occurs in the PNW but is not characteristic of the region.

### Provenance text errors

Source-backed validation identified six provenance descriptions with factual inaccuracies:

| # | Palette | Error | Correction |
|---|---------|-------|------------|
| 10 | Witch's Hair | Says "olive-black to dark greenish-brown thallus" | *Alectoria sarmentosa* is pale yellow-green to straw; may confuse with *Bryoria* |
| 13 | Jack-o'-Lantern | Uses *Omphalotus olearius* | Western species is *O. olivascens*; *O. olearius* is European |
| 25 | Balsamroot | Says "from the Rogue Valley" | *B. sagittata* is primarily east of the Cascades; Rogue Valley presence unconfirmed |
| 28 | Partridgefoot | Says "gray-green" foliage | USDA describes foliage as "glossy green" |
| 29 | Lupine | Claims silvery sheen | *L. latifolius* is specifically not silvery (unlike *L. argenteus*) |
| 40 | Reindeer Lichen | Says "east-slope Cascades" | Species is actually rare on east side per McCune & Geiser |

### B-score patterns (revised)

The 12 source-backed B scores cluster into four patterns:

- **Hue/tone overshifted** (4): Oakmoss teal vs real gray-green; Elderberry too purple vs powder-blue; Salal too vivid vs near-black; Reindeer Lichen too green vs silver-gray
- **Missing iconic feature** (2): Oregon Sunshine and Balsamroot both capture foliage but miss the bright yellow flowers that define the species
- **Warm/cool drift** (3): White Oak bg too peachy vs silver-gray bark; Alder too warm vs cool gray bark; Pearly Everlasting heading too blue vs white/cream bracts
- **Atmospheric interpretation** (3): Goatsbeard captures woodland mood but not direct colors; Partridgefoot teal-green is foliage-adjacent; Witch's Hair provenance itself is wrong so comparison is compromised

### Oregon Sunshine and Balsamroot: the weakest B's

The source validation upgraded these from C to B based on the foliage connection the self-assessment missed. However, they remain the weakest palettes in the collection by a clear margin:

1. Both species' most iconic visual identity is bright golden-yellow flowers — extensively documented across every source consulted.
2. Both palettes render this as muted amber-brown, which reads as "late-season dried" rather than the vivid in-bloom display people most associate with the species.
3. Both mint-green backgrounds connect to real foliage colors but position the palettes closer to Light-Cool than Light-Warm.
4. The two palettes are also similar to each other (Rgb(230,242,205) vs Rgb(218,242,210)), reducing within-category distinctiveness.

The original analysis holds: these are the strongest candidates for species replacement. The palettes themselves are fine — the problem is the species names. Both palettes read as "pale green with warm brown/olive accents," and the replacement species should match that actual impression rather than being another bright-yellow wildflower.

**Implications:**

- The collection is stronger than the self-assessment suggested: 100% of palettes score A or B against external sources.
- No palettes require replacement on color grounds alone — but Oregon Sunshine (#22) and Balsamroot (#25) are marginal B's where the species name primes expectations the palette doesn't satisfy.
- Three species have borderline geographic status (Manzanita, Jack-o'-Lantern, Reindeer Lichen).
- Six provenance descriptions contain factual errors that should be corrected regardless of species decisions.
- The "moderate congruence" standard from Invariant 16 is met by all 40 palettes, but some meet it more convincingly than others.
- **Next question:** What replacement species would strengthen the weakest B's and resolve the geographic borderline cases?

---

## Question 3: What Cascadia species authentically match the color profiles of the weakest palettes?

**Method:** Web search — researched species whose real botanical colors match the existing palette color profiles. Searched Oregon Flora Project, USDA PLANTS, USFS FEIS/Silvics, McCune & Geiser (lichens), regional field guides, and nursery databases. Three parallel agents, one per replacement slot.

**Findings:**

### Oregon Sunshine (#22) → Bracken

**Recommended replacement:** Bracken (*Pteridium aquilinum*)

The palette reads as Rgb(230,242,205) pale mint-green background with warm amber-brown and olive-gold accents — "light green field with warm brown details." Bracken maps to every element:

- **Light green fronds** — bracken's fronds are characteristically lighter green than most PNW ferns, matching the pale mint bg. Sources: USFS FEIS, WSU PNW Plants, Gardenia.net.
- **Straw-brown stipes** — described as "straw-colored to light brown" with dark brown bases. Maps to the amber-brown heading. Source: Minnesota Wildflowers, multiple field guides.
- **Amber autumn tones** — fronds turn "golden brown," "amber," "bronze" before dying back. The Woodland Trust notes "bracken" is itself used as a color name meaning golden-brown. Maps to the olive-gold emphasis.
- **Single-word name** — warm connotation, no bright-yellow expectation. In textile/color contexts, "bracken" connotes golden-brown.
- **Extremely widespread in Cascadia** — Alaska to Baja California, both sides of Cascades crest, every PNW county.
- **Projected score: A** — light green + straw-brown + amber-gold is a near-perfect three-way match.

Runner-up: Hazelnut (*Corylus cornuta* var. *californica*) — green leaves + brown nut/husk, A- score. Name primes brown, not yellow.

### Balsamroot (#25) → Licorice Fern

**Recommended replacement:** Licorice Fern (*Polypodium glycyrrhiza*)

The palette reads as Rgb(218,242,210) — slightly more saturated green than #22, with warm amber-brown and olive-gold accents. Licorice Fern maps to every element:

- **Bright green fronds** — described as "bright green" and "lush" by multiple sources (Real Gardens Grow Natives, Native Plants PNW, UW). More saturated green than bracken, matching the more saturated bg.
- **Golden-brown to cinnamon-brown sori** — newly ripened sori are "buttery yellow," aging to "rich golden-brown" and then "red-brown to cinnamon-brown." Sources: Gardenia.net, UBC Botany. Maps to both warm accent colors.
- **Reddish-brown rhizome** — the creeping root has a sweet licorice flavor and reddish-brown color.
- **"Licorice" evokes warmth** — sweet, earthy, amber-brown associations. *Glycyrrhiza* literally means "sweet root." Perfect for Light-Warm.
- **Quintessentially Cascadian** — southern Alaska through coastal BC, WA, OR to central CA. The iconic PNW epiphyte on mossy bigleaf maples. One of the most recognizable sights in PNW temperate rainforest.
- **Distinct from Bracken** — different life form (epiphyte vs ground fern), different habitat (rainforest tree trunks vs open hillsides), different visual character.
- **Projected score: A** — bright green + golden-brown sori + warm evocative name.

Runner-up: Step Moss (*Hylocomium splendens*) — olive-green body + red-brown stems, A- score. Name less evocative of warmth.

### Reindeer Lichen (#40) → Silver Fir

**Recommended replacement:** Silver Fir (*Abies amabilis*, Pacific Silver Fir)

The palette reads as Rgb(225,232,238) pale cool blue-gray background with muted dark green, medium muted green, and muted purple accents. Silver Fir is the only candidate matching all three color families:

- **Silvery-white needle undersides** — the species' defining visual trait and namesake. Two dense stomatal bands create a conspicuous silver-white appearance. Sources: OSU Landscape Plants, Conifers.org, USFS Silvics. Maps directly to the pale cool blue-gray bg.
- **Light gray smooth bark** — reinforces the silvery-gray background impression. Sources: USFS Silvics, multiple field guides.
- **Dark green needle upper surfaces** — maps to the muted dark green heading and medium muted green emphasis.
- **Deep purple mature cones** — start green, ripen to deep purple, then dark purplish-brown. Sources: OSU Landscape Plants, USFS Silvics. Maps to the muted purple link accent Rgb(80,72,90). This is the decisive advantage — no other candidate matches the purple.
- **"Silver Fir" already anticipated** — listed as an example two-word name in ADR-015 and the palette collection design essay.
- **Iconic mid-elevation Cascadia tree** — SE Alaska through coastal BC, WA, OR at 610–1830m. Defining species of the Western Cascades mid-elevation zone. Common in Olympic, North Cascades, and Mount Rainier National Parks.
- **Projected score: A** — the only candidate producing all three palette color families (silver-gray, muted green, muted purple).

Runner-up: Shield Lichen (*Parmelia sulcata*) — silvery gray-green thallus, A- score, but no purple match.

### Jack-o'-Lantern (#13) — species name fix only

No species replacement needed. The color match is A. The fix is:
- Correct *Omphalotus olearius* → *Omphalotus olivascens* in the provenance text
- *O. olearius* is the European species; *O. olivascens* is the western North American species
- Geographic range is borderline but defensible (primarily CA/OR, documented as far north as BC)

### Summary (revised after Q4)

| Slot | Current | Action | New Species | Category |
|------|---------|--------|-------------|----------|
| #22 | Oregon Sunshine (Light Warm) | Move to Light Vivid, redesign palette | **Oregon Sunshine** | Light Vivid (new, #41) |
| #22 | *(vacated slot)* | New species for Light Warm | **Bracken** | Light Warm |
| #25 | Balsamroot | Replace species | **Licorice Fern** | Light Warm |
| #40 | Reindeer Lichen | Replace species | **Silver Fir** | Light Muted |
| #13 | Jack-o'-Lantern | Fix species name only | *(keep)* | Dark Vivid |

All replacements and the Oregon Sunshine redesign are projected A scores, and all species are genuinely common in the Cascadia bioregion.

**Implications:**

- Collection grows from 40 to 41 palettes: Light Vivid gains a 6th palette (within the 3–7 acceptable range).
- Oregon Sunshine gets a palette that authentically represents its vivid golden-yellow flowers, filling the one missing major hue in Light Vivid (yellow).
- The projected score distribution moves from 28 A / 12 B to 32 A / 9 B (3 species replacements convert B→A, plus Oregon Sunshine redesign is projected A).
- All geographic borderline flags resolved (Manzanita confirmed by user, Jack-o'-Lantern kept with defensible range, Reindeer Lichen replaced by Silver Fir).
- Light Warm gains taxonomic diversity: ground fern (Bracken) + epiphytic fern (Licorice Fern) + bunchgrass (Oatgrass) + two trees (White Oak, Ponderosa).
- "Sword Fern" (Light Muted) and "Licorice Fern" (Light Warm) — two ferns in 41 palettes across different categories. Acceptable given register diversity.
- **New design work required:** Oregon Sunshine needs a new palette with vivid golden-yellow accents on a light background, satisfying WCAG AA 4.5:1 contrast and 15° OKLCH hue diversity against Light Vivid neighbors.

---

## Question 4: Does Oregon Sunshine's real botanical color profile fit better in a different affective category?

**Method:** Category analysis — compared Oregon Sunshine's documented colors (bright golden-yellow flowers, gray-green woolly foliage, dry sunny habitat) against each of the 8 affective categories, evaluating color fit, character match, hue gap coverage, and within-category distinctness.

**Findings:**

**Light Vivid is the clear best fit.** Oregon Sunshine's defining feature — bright golden-yellow daisies — is high-chroma saturated color, which is definitionally "vivid." The species' character (cheerful, sun-loving, dry-slope, full-exposure) matches "bright, solar" far better than Light Warm's "gentle, morning."

Critically, yellow is the one major hue absent from Light Vivid. The current 5 palettes cover red (Paintbrush), orange-red (Columbine), orange (Tiger Lily), magenta-pink (Farewell), and blue-violet (Camas). Golden-yellow would fill the gap, sitting well-separated in OKLCH hue space from both Tiger Lily's orange and Camas's blue-violet.

The current Light Warm placement forces the palette to suppress Oregon Sunshine's most iconic feature. The mint-green background + amber-brown accents represent the woolly foliage and late-season dried tones, but not the vivid in-bloom display that defines the species in every field guide and personal experience. User confirms personal connection to the species — grew it directly.

Other categories were evaluated and rejected:
- **Light Warm (current):** Golden-yellow overshoots the "gentle, morning" register. Competes with Balsamroot/Oatgrass.
- **Light Cool:** Warm yellow clashes with "crisp, alpine." Not an alpine species.
- **Light Muted:** Would require suppressing the flowers. Gray-green foliage is already covered by Sword Fern, Reindeer Lichen.
- **All Dark categories:** Oregon Sunshine is a sun-drenched, open-habitat species. Dark backgrounds are botanically incongruous.

**Implications:**

- Oregon Sunshine moves from Light Warm to Light Vivid with a redesigned palette.
- The vacated Light Warm slot is filled by Bracken (already identified as the best replacement for that color profile).
- Light Vivid goes from 5 to 6 palettes (within the 3–7 research-supported range).
- The Oregon Sunshine redesign requires new palette colors — a task for the implementation phase, subject to WCAG contrast and OKLCH hue diversity constraints.
