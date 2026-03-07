# Research Log: Color Palettes as Creative Environment

## Question 1: Does the color environment a writer works in affect creative output — mood, tone, psychological arousal — in ways that are generative rather than merely decorative?

**Method:** Web search across three domains — El-Nasr's game lighting/arousal research, color psychology and creativity, environmental conditions and creative writing

### Thread A: El-Nasr — Color, Lighting, and Psychological Arousal

Magy Seif El-Nasr's research program (2003–2011) established that simulated illumination functions as a direct modulator of emotional state, not merely an aesthetic choice. The core contributions:

**Five tension-modulation patterns** (derived from 30+ film analyses, Game Studies 2007):

| Pattern | Effect |
|---------|--------|
| Low→High contrast escalation | Increases arousal |
| Low→High saturation/warmth | Increases arousal |
| High→Low contrast release | Reduces arousal |
| High→Low saturation/warmth release | Reduces arousal |
| Sustained high contrast or saturation | Continuous arousal elevation |

**Experimental validation** (CHI 2005, 100+ participants): Dynamic lighting in an FPS mod — where saturation and warmth increased with danger level — successfully induced contextually appropriate emotions. Notably, experienced FPS players who had trained themselves toward emotional detachment reported feeling *disturbed* by the arousal induction — direct evidence that color environment was priming emotional states the players were not voluntarily entering.

**Emotional affordances framework** (2011): El-Nasr extended Gibson's affordance theory to argue that color and lighting function as *emotional affordances* — subconsciously operative environmental stimuli that afford specific emotional responses without requiring deliberate attention. This is the theoretical bridge to our design question: if a palette can function as an emotional affordance, it can prime the writer's affective state.

**Bidirectional mapping** (DigitalBeing, 2006): The same color-arousal mapping that allows color to induce arousal (games) also allows arousal to be represented as color (dance visualization). This mutual legibility suggests the mapping reflects a consistent perceptual-physiological mechanism, not arbitrary convention.

**Knez & Niedenthal controlled experiment** (Cyberpsychology & Behavior, 2008): 38 participants in warm-lit vs. cool-lit Half-Life 2 mazes. Warm lighting produced significantly greater enthusiasm, energy, and faster task completion. Crucially, *affect mediated the performance improvement* — color changed emotional state, which changed performance. The pathway is: color → mood → behavior.

### Thread B: Color Psychology and Creative Performance

**Mehta & Zhu (Science, 2009):** The headline finding — blue backgrounds doubled creative output, red boosted detail work by 31% — has **not survived replication**. Steele (2014) failed to replicate with 3x the sample size. The directional trend has partial support (blue somewhat associated with creative performance, red with detail), but effect sizes appear much smaller than originally reported. **Do not treat the 2x claim as reliable.**

**Saturation is the primary arousal driver, not hue.** Wilms & Oberfeld (Psychological Research, 2018) found saturation had roughly twice the impact of brightness on arousal (regression: arousal ≈ −0.31(brightness) + 0.60(saturation)). Saturated colors produce higher skin conductance responses regardless of hue. This complicates the "blue = creative, red = focused" narrative — a highly saturated blue and red may both elevate arousal similarly.

**Green shows the most consistent creativity-specific effect.** Lichtenfeld et al. (PSPB, 2012) found green exposure enhanced both quantity and quality of creative ideas across four experiments, even when controlling for brightness and saturation. Proposed mechanism: green as a "growth cue" activating mastery-approach orientation. No prominent replication failure, though the broader field has replication issues.

**Color-in-context theory** (Elliot & Maier, 2012): Color effects depend on psychological context. Red in an achievement/evaluation context impairs performance; red in a playful/romantic context may enhance it. Implication: the *framing* of the color environment matters as much as the colors themselves.

### Thread C: Environmental Conditions and Creative Writing

**Dim lighting promotes divergent thinking.** Xu & Labroo (J Environmental Psychology, 2014): Six experiments showed dim lighting (~150 lux) outperformed standard office lighting for creative tasks. Mechanism: darkness activates a felt sense of freedom from social constraints. Boundary condition: bright light was better for careful reasoning and proofreading.

**Mood congruence and creative processing.** Fredrickson's broaden-and-build theory (1998, 2001): Positive affect broadens the thought-action repertoire, expanding associative search and cognitive flexibility. Negative emotions narrow toward specific problem-focused responses. For creative writing, an environment generating approach-oriented affect supports generative ideation. The mood congruence principle also suggests writers working on emotionally toned content may benefit from mood-congruent environments that prime associated concepts and memories.

**Warm dim + cool bright are both effective; the mismatch kills.** A Building Simulation study found that 300 lux at 3000K (warm dim) and 2000 lux at 6000K (cool bright) both produced high positive mood, while "warm bright" and "cool dim" did not. The interaction matters more than either variable alone.

**Writers deliberately design environments.** Diana Fuss (Princeton, 2004) documented how literary writers shaped spaces to match creative needs — most strikingly, Proust's cork-lined, light-blocked room for writing about sensory memory. Will Self plasters workspace walls with maps and notes from the current project. This is practitioner-level evidence that environment-as-prime is intuitively understood by creative professionals.

**4E cognition framework.** The dominant theoretical lens treats creativity as constituted through brain-body-environment interactions. Creative spaces offer specific "action possibilities" (affordances) that shape what ideas emerge. The environment is not backdrop but co-author.

### Synthesis and Implications

**What the evidence supports:**

1. **Color environment primes emotional state** — this is well-established across El-Nasr's game studies, Knez & Niedenthal's controlled experiments, and the broader environmental psychology literature. The pathway is color → affect → cognition/behavior.

2. **Saturation and warmth are the primary levers** — not specific hues. High saturation elevates arousal; warm colors produce approach-oriented positive affect; cool desaturated colors produce calm/reflective states. The clean "blue = creative" claim is unreliable.

3. **The priming is subconscious** — El-Nasr's "emotional affordances" framework explicitly claims that color-based emotional priming operates below conscious attention. This means a palette could shift a writer's affective state without them deliberately attending to it.

4. **Dim environments favor generative work; bright environments favor revision** — Xu & Labroo's finding maps directly onto writing phases. A dark-mode palette with warm tones would support drafting; switching to a lighter, cooler palette for editing is theoretically grounded.

5. **Mood congruence is plausible but not directly tested for writing** — the idea that a cyberpunk neon palette could prime a noir/futuristic creative register is theoretically grounded in mood-congruent memory research and environmental priming, but no controlled study has tested "genre-matched writing environments."

**What remains speculative:**

- Whether genre-specific palettes (noir, pastoral, etc.) produce measurably different creative output
- Whether the effect size is large enough to matter in practice vs. being a comfort/preference choice
- Whether sustained exposure (hours of writing) amplifies or habituates the priming effect

**What this means for Zani's palette design:**

The evidence supports designing palettes as *mood instruments*, not just aesthetic choices. A "Neon Noir" palette with high-saturation accent colors against a deep dark background would, per the research, elevate arousal and prime approach-oriented affect differently than a "Morning Pages" palette with warm, desaturated, gentle tones. The mechanism is real even if the magnitude is uncertain.

The categorization system should reflect this: palettes organized by the affective state they prime (energizing vs. calming, warm vs. cool, high-arousal vs. contemplative), not just by visual similarity.

---

## Question 2: Given WCAG AA (4.5:1), no pure black/white, and the mood-instrument framing, what is the actual design space for zani palettes?

**Method:** Spike (computational exploration of RGB space under zani's constraints) + web search (WCAG palette design techniques)

### Spike Findings: The Design Space Is Wide Open

**Spike question:** "How many meaningfully distinct palettes can satisfy zani's contrast invariants?"

**Background/foreground viability:** Of 72,000 sampled dark-background/light-foreground pairs, 100% passed WCAG AA (4.5:1). 99.6% also passed AAA (7:1). The existing palettes (Ember at 10.5:1, Inkwell at 10.7:1, Parchment at 10.1:1) sit well above the minimum — there is substantial headroom.

**Accent color freedom:** Every hue at full saturation (1.0) can pass 4.5:1 against dark backgrounds. The most constrained hues are blue/violet (H=240-260), which still have hundreds of viable lightness/saturation combinations. Yellow/chartreuse (H=60-90) have the most freedom. Against the Ember background alone, 12,759 sampled accent colors pass.

**Mood category viability:** Monte Carlo sampling across six mood categories — warm contemplative, cool intellectual, high-energy neon, natural/organic, warm light, cool light — produced **100% viability** across all categories (500/500 samples each generated full valid palettes). The constraints do not meaningfully restrict mood expression.

**The key constraint insight:** WCAG constrains *lightness relationships*, not hue or saturation. Any hue at any saturation can satisfy the contrast requirement if placed at the correct lightness relative to the background. For dark backgrounds (luminance ~0.01-0.03), accent colors need to be above roughly L=0.45 in lightness. For light backgrounds (luminance ~0.80-0.95), accents need to be below roughly L=0.40. Within those bands, hue and saturation are essentially free parameters.

**What this means:** A neon cyberpunk palette with fully saturated magenta and cyan accents is just as achievable as a muted morning-pages palette with desaturated warm tones. The constraints shape where on the lightness axis colors must sit, but leave mood expression — hue character and saturation intensity — entirely unconstrained.

### Web Research: WCAG Palette Design Techniques

**The "myth" of accessibility limiting palettes** (Stéphanie Walter): Designers who find WCAG constraining are typically using HSL, where "lightness" doesn't correspond to perceived brightness. A yellow and blue at the same HSL lightness have dramatically different perceived luminance. The fix is working in perceptually uniform color spaces.

**Perceptually uniform color spaces are the key tool:**
- **OKLCH** — the current best practice. Lightness (L), Chroma (C), Hue (H) axes where equal L values produce equal perceived brightness regardless of hue. Supported in all major browsers, underlies Tailwind CSS defaults.
- **CIELAB/LCh** — used by Accessible Palette and Stripe's color system.
- **HSLuv** — HSL remapped to CIELUV so hue/saturation changes don't affect perceived brightness.

The practical result: once you fix lightness at a given palette "stop," every color at that stop across every hue will pass or fail contrast against the same backgrounds. Accessibility becomes a construction property, not a per-color verification.

**Techniques for visually distinct accessible palettes:**
1. **Tiered shade scales (100-900)** — USWDS "magic number" rule: grade difference of 50+ ensures AA compliance
2. **Hue shifting during darkening** — yellows shift toward brown rather than green for natural dark tones
3. **Non-linear chroma distribution** — extreme light/dark shades are low-chroma (neutral), mid-tones are high-chroma (vibrant)
4. **Semantic color roles** — decouple "what a color does" from "what it looks like"

**Dark theme specific findings:**
- Pure black (#000000) causes halation — use #121212 or darker grays (zani already enforces this)
- Saturated accents that work in light mode "vibrate" on dark backgrounds — reduce saturation 20-40%
- Off-white text (#E0E0E0 range) instead of pure white reduces strain (zani already does this)
- Elevation through tonal shift instead of shadows

**APCA (WCAG 3.0 draft):** The current WCAG formula overstates contrast for dark colors and doesn't account for font weight or polarity (light-on-dark vs dark-on-light). APCA addresses this with Lc (Lightness Contrast) values. Recommendation: comply with WCAG 2.1 for audit/legal, use APCA for quality validation of reading experience.

### Synthesis

The design space is effectively unconstrained for mood expression. The constraints shape *which lightness band* colors must occupy, but leave hue and saturation — the primary vehicles for mood/affect — entirely free. A palette collection organized by affective character is not just theoretically grounded (Question 1) but practically achievable within existing invariants.

The right approach for palette construction: work in OKLCH space, fix lightness values that satisfy contrast requirements, then freely explore hue and saturation for mood expression. This makes contrast compliance a *construction property* rather than something verified after the fact.

**Tension with existing invariants:** None found. The existing Invariant 3 (no pure black/white, WCAG AA minimum) is fully compatible with the expanded palette collection. No invariant changes are needed.

---

## Question 3: What happens to mood-instrument palettes under 256-color quantization?

**Method:** Spike (reproduce zani's `nearest_256_color` algorithm, test existing and hypothetical palettes)

### Spike Findings

**Spike question:** "Do carefully designed mood-instrument palettes survive the 6x6x6 color cube mapping used in zani's Color256 profile?"

**Contrast ratios survive.** Every palette tested — all three existing palettes plus six hypothetical mood palettes (Neon Noir, Forest Deep, Midnight Ocean, Sunset Draft, Morning Pages, Arctic Study) — maintained >4.5:1 contrast across all foreground/background and accent/background pairs after quantization. No failures. The luminance relationships that drive contrast are robust to the color cube snapping.

**Dark backgrounds lose hue identity.** The 6x6x6 cube has a 95-value gap between its first two stops (0 and 95), meaning all dark backgrounds with channel values below ~48 fall into the grayscale ramp. Only 10 distinct 256-color indices exist below luminance 0.03. Concrete impact:

| Palette background | True Color | 256-color |
|--------------------|------------|-----------|
| Ember (warm brown) | (40, 38, 35) | (38, 38, 38) — neutral gray |
| Inkwell (cool navy) | (30, 32, 40) | (38, 38, 38) — same gray |
| Neon Noir (purple-black) | (18, 12, 28) | (18, 18, 18) — neutral gray |
| Midnight Ocean (deep navy) | (15, 20, 35) | (28, 28, 28) — neutral gray |
| Forest Deep (dark green) | (25, 30, 22) | (28, 28, 28) — same gray |

In 256-color mode, Ember and Inkwell become the same background. Midnight Ocean, Forest Deep, and Sunset Draft all share a background. The warm/cool/chromatic character that defines mood is erased.

**High-saturation accents survive well.** Maximum hue shift across all tested accent colors was ~10°, with saturation dropping ~0.10 on average. A neon magenta stays recognizably magenta; a cyan stays cyan. The mood character carried by vivid accents is preserved.

**Low-saturation accents collapse to gray.** The gentle, muted palettes (Morning Pages, Arctic Study) lost most of their accent hue character. Links, emphasis, and code colors all mapped to the same grayscale values. In 256-color, subtlety is the first casualty.

**Dimming gradients get stepped.** A smooth 6-stop opacity gradient (1.0 → 0.0) quantizes to ~4 distinct steps. Still functional but visibly banded rather than smooth.

### Decision: Hybrid degradation strategy

Given zani's commitment to a beautiful writing experience in any terminal environment, the approach is:

1. **True Color (primary):** Palettes are authored in OKLCH, stored as exact RGB values. Full mood expression — background hue, accent saturation, smooth dimming gradients — all intact.

2. **256-color (hand-tuned for key palettes):** For the core palette collection, provide hand-tuned 256-color alternate values that maximize mood preservation within the 6x6x6 cube. This means:
   - Selecting accent colors that land on or near cube vertices to minimize hue drift
   - Accepting that dark backgrounds will be neutral gray, and compensating by ensuring accents carry more of the mood weight
   - For muted/subtle palettes, slightly increasing saturation in the 256-color variant so hue character survives quantization
   - Verifying contrast ratios hold for the tuned 256-color values independently

3. **Basic ANSI (16 colors):** Accept that mood expression is minimal. Focus on readability — ensure foreground/background contrast is clear and dimming falls back to the terminal's dim attribute (already implemented). This tier is functional, not beautiful.

This strategy keeps the automatic `nearest_256_color` mapping as the fallback for any palette, but gives the curated collection a considered 256-color presentation. The palette struct may need to accommodate per-profile color overrides — a design question for the model/architect phases.

### Implications for palette construction

When designing palettes in OKLCH:
- **Choose accents near 6x6x6 cube vertices** when possible, so the True Color and 256-color versions stay close. The cube vertices at high lightness offer strong hue coverage.
- **Lean on accent saturation, not background hue, for mood** — background hue will be lost in 256-color, but accent character survives if saturation is sufficient.
- **Test both profiles during design** — run each palette through `nearest_256_color` and verify the 256-color version still reads as the intended mood before shipping.

---

## Question 4: What are the UI and configuration considerations for palette selection?

**Method:** Design reasoning from existing codebase and research findings (no spike or web search needed)

### Current state

Palette selection lives in the settings panel as three static rows (`SettingsItem::Palette(0)`, `Palette(1)`, `Palette(2)`) in a flat `ALL_ITEMS` array. When the cursor hovers a palette row, the entire overlay previews that palette's colors via `Palette::blend()`. Applying a palette triggers a 300ms EaseOut crossfade animation. Config persists as a global `~/.config/zani/config.toml` with a `palette: String` field.

This design works well for 3 palettes. It will not scale to a larger curated collection.

### Transition smoothness

The 300ms crossfade is already implemented and handles the jarring-switch concern for the *moment of application*. Two additional considerations:

1. **Preview during browsing.** Currently, hovering a palette row previews it on the settings overlay. With a larger collection, the writer will navigate through many palettes. Each cursor movement triggering a full-overlay color change could feel hectic. Options:
   - **Preview on hover with debounce** — only blend after the cursor rests on a palette for ~200ms, avoiding rapid flickering during fast navigation
   - **Preview swatch only** — show color swatches (already implemented) without changing the overlay background until the writer presses Enter to apply
   - **Split preview** — the writing surface behind the overlay shows the current palette; the settings panel shows swatches and names. Only on apply does the surface crossfade.

   The current hover-preview behavior feels right for a small list. For a larger collection with categorization, a swatch-based browser that only applies on Enter (with the crossfade) is likely less distracting.

2. **The writing surface crossfade.** The existing animation only crossfades between two known palettes (from/to). This already works correctly — the `TransitionKind::Palette { from, to }` pattern generalizes to any pair. No changes needed to the animation system itself.

### Per-document and per-folder palette binding

The mood-instrument framing makes this natural: if a writer associates Neon Noir with their cyberpunk project and Morning Pages with their journal, the palette should follow the context. Three resolution levels, in priority order:

1. **Per-document** — a palette specified for a specific file
2. **Per-folder** — a palette specified for all files in a directory (the "project" level)
3. **Global** — the `~/.config/zani/config.toml` default

Implementation approach — **local config files:**

A `.zani.toml` file in a directory would override global config for any file opened from that directory (or below it). Walk up from the file's location until finding `.zani.toml` or reaching the filesystem root, then fall back to global config. This mirrors how `.editorconfig`, `.prettierrc`, and similar tools work — a familiar pattern for terminal-native users.

```toml
# ~/writing/cyberpunk-stories/.zani.toml
palette = "Neon Noir"

# Could also override other settings per-project:
# focus_mode = "paragraph"
# column_width = 72
```

Per-document binding could use the same mechanism with a more specific path, but is likely overengineering for v1. Per-folder covers the primary use case (project = mood) without per-file metadata.

**Behavioral considerations:**
- When opening a file, resolve the palette from local config → global config → default. If the resolved palette differs from the current one, crossfade on open.
- Switching palettes in the settings panel should offer a choice: "Apply to this project" (write `.zani.toml`) vs. "Apply globally" (write `~/.config/zani/config.toml`). This could be as simple as a second keybinding in the palette browser — Enter to apply globally, a modifier (e.g., Shift+Enter or a "pin to project" action) to write the local override.
- The settings panel should indicate when a local override is active — e.g., showing "Neon Noir (project)" vs. "Ember (global)" — so the writer knows why a particular palette is loaded.

### Browsing a larger collection

With 3 palettes, a flat list works. With 10-20+ mood-instrument palettes, the settings panel needs structure. Design considerations:

1. **Categorization by affect** — group palettes by the mood dimension from Question 1 (e.g., "Dark — Warm", "Dark — Cool", "Dark — Vivid", "Light — Warm", "Light — Cool"). This is the natural organization from the mood-instrument framing.

2. **Perceptual sort order within categories.** Within each group, palettes should be sorted by perceptual similarity so that adjacent entries are close neighbors in color space. Scrolling through the list should feel like a smooth gradient, not random jumps. This matters because the hover-preview (or swatch display) creates a visual experience as the writer browses — jarring transitions between unrelated palettes make browsing feel chaotic. The sort could use OKLCH hue as the primary axis (warm → cool rotation), with lightness as a secondary axis. This is essentially a traveling salesman problem on the palette's dominant hue, which for a small collection can be solved by hand or with a simple hue-angle sort.

3. **Filtering** — in a longer list, a quick-filter by typing a palette name or category name reduces navigation friction. This parallels the existing find overlay pattern.

4. **The settings panel may need a sub-panel.** Currently palettes are inline rows in the main settings list. A collection of 15+ palettes inline would dominate the settings panel. A dedicated palette browser — opened from a single "Palette: [current]" row in settings — gives room for categories, swatches, descriptions, and filtering without cluttering the main settings view.

### Implications for downstream phases

These questions shape the model and architect phases:

- **The Palette struct** may need a `category` or `mood` field for grouping/filtering
- **The Config system** needs local-config resolution (walk-up `.zani.toml` search)
- **The Settings UI** needs a dedicated palette browser sub-panel
- **The settings item model** changes from `Palette(usize)` indexing a flat array to something that supports categories and a scrollable list

None of these are research questions — they're design decisions that belong in the model/architect/build phases.

