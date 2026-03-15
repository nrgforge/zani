# Domain Model: Zani

## Concepts (Nouns)

| Term | Definition | Product Origin | Related Terms |
|------|-----------|----------------|---------------|
| **Document** | A markdown file opened for editing. The unit of work in Zani. | "Local-first" — plain markdown on disk, git-friendly | Buffer, Draft |
| **Buffer** | The in-memory representation of a Document's text, managed as a rope data structure (Ropey). Supports efficient insertion, deletion, and cloning. | — | Document |
| **Writing Surface** | The custom text viewport where prose is rendered. Handles soft-wrapping, scroll positioning, and per-character styling. Built on ratatui's cell buffer, bypassing the Paragraph widget. | "Beautiful" — the rendered writing environment | App Shell, Focus Mode |
| **App Shell** | The ratatui application frame that manages layout, input routing, and the event loop. Contains the Writing Surface but does not render prose directly. | — | Writing Surface |
| **Focus Mode** | A visual mode that dims text outside the active region to keep the writer in generative mode. Two variants: Sentence, Paragraph. Orthogonal to Scroll Mode. | "Focus mode" — writer's direct term | Dimming, Active Region, Scroll Mode |
| **Active Region** | The sentence or paragraph currently at full brightness during a Focus Mode. Everything outside it is dimmed. | — | Focus Mode, Dimming |
| **Dimming** | Per-character color interpolation from full foreground brightness toward the background color, creating a visual fade. Internally expressed as an opacity factor (0.0–1.0) per character, which governs the interpolation amount. The mechanism behind Focus Mode and Markdown Styling. | — | Focus Mode, Palette, Color Profile |
| **Scroll Mode** | How the viewport follows the cursor. Two variants: Edge (scroll when cursor nears edges) and Typewriter (cursor stays vertically centered). Orthogonal to Focus Mode. | — | Focus Mode, Writing Surface |
| **Typewriter Mode** | A Scroll Mode variant where the cursor stays vertically centered and text scrolls around it, eliminating manual scrolling. Does not contribute dimming. | "Typewriter mode" — writer's direct term | Scroll Mode |
| **Fade Config** | A pairing of duration and easing curve that governs how a dimming transition animates. Each dimming source specifies separate configs for fade-in (brightening) and fade-out (dimming). | — | Dimming, Active Region |
| **Palette** | A named, curated color system defining foreground, background, dimming endpoints, and accent colors. Designed as a mood instrument — priming a specific affective state rather than serving as decoration. Named after a Cascadia bioregion species from the Naming Register, with a Provenance Description documenting the species and its color associations. The composition — which of the species' Signature Colors occupies which slot — is a deliberate design decision driven by the Species Essence. The background dominates the visual field (~90% of screen area) and determines the writer's immersive experience; accent slots provide punctuation. Belongs to an Affective Category and its background must achieve that category's Chroma Target. May include hand-tuned 256-color alternate values for graceful degradation. The default Palette is Manzanita (*Arctostaphylos*), tuned to evoke the species' smooth mahogany-to-cinnamon bark. All palettes satisfy Invariants 3 and 18. | "Palette", "Mood", "Room" — writer's core vocabulary for the color environment | Affective Category, Chroma Target, Color Profile, Perceptual Sort Order, Naming Register, Provenance Description, Species Essence, Signature Colors |
| **Affective Category** | A mood-based grouping of palettes along two axes: brightness (Dark, Light) and character (Warm, Cool, Vivid, Muted). Eight categories of approximately five palettes each (~40 total, target 3–7 per category). Examples: "Dark — Warm", "Dark — Muted", "Light — Cool". The organizing taxonomy for the Palette Browser. Siblings are spread across the available OKLCH hue space for maximum perceptual diversity. A species' Essence Mode and Signature Colors jointly determine which Affective Category a palette belongs to — the essence determines the composition, and the composition determines the category. Each category has a Chroma Target that its palette backgrounds must achieve; the character axis must produce chromatically distinguishable profiles in the dominant visual field (backgrounds), not only in accent punctuation. | "Mood" — writer says "mood," system says Affective Category | Palette, Palette Browser, Species Essence, Chroma Target |
| **Naming Register** | The constraint that all Palette names are drawn from Cascadia bioregion flora — vascular plants, mosses, lichens, and fungi. The register provides traceable provenance, register consistency, and moderate schema congruence with Affective Categories. Named after the same botanical world as the app itself (Zani = Manzanita). A palette cannot be added to the collection unless a species exists whose Signature Colors are moderately congruent with the palette's color composition. | — (builder term; writer says "the plant names") | Palette, Provenance Description, Flora Reference, Signature Colors |
| **Provenance Description** | A botanically accurate one-line note attached to each Palette, documenting the species' scientific name, its appearance, and its Cascadia ecological context. Must be grounded in external botanical sources (Oregon Flora Project, USDA PLANTS, USFS, field guides) — not rationalized from the palette's colors after the fact. Must reflect actual habitat range and species character, not romanticized sketches — the descriptions should hold up to scrutiny from someone who lives among these plants. | — | Palette, Naming Register, Flora Reference |
| **Flora Reference** | A structured reference document (`docs/flora-reference.md`) documenting each species in the collection: its Signature Colors with source citations, Essence Mode, geographic range within Cascadia, and any provenance notes. The durable botanical corpus that exists independently of code. Consulted when designing new palettes, validating existing ones, or considering species replacements. Enables ongoing refinement without losing accumulated knowledge. | — | Palette, Naming Register, Signature Colors, Essence Mode |
| **Signature Colors** | The 1–2 real documented colors that define a species' visual identity, verified against external botanical databases and field guides. Distinct from palette colors — Signature Colors describe what the species actually looks like, not what the palette renders. Example: Oregon Sunshine's Signature Colors are bright golden-yellow (flowers) and gray-green (woolly foliage). | — | Flora Reference, Species Essence, Palette |
| **Species Essence** | The compositional rationale connecting a species' Signature Colors to specific Palette slots. Identifies which visual feature of the species the palette is "about" — which color the writer is meant to be *inside*. The essence determines the composition; the composition determines the Affective Category. | — | Palette, Signature Colors, Essence Mode |
| **Essence Mode** | The strategy by which a species' Signature Colors are distributed across palette slots. Three modes: **Feature** (one dominant color drives the background or primary accent — e.g., Ponderosa's bark), **Throughline** (a year-round constant occupies the background while seasonal colors occupy accents — e.g., Oregon Grape's evergreen foliage with spring flowers and fall berries), **Place** (the atmospheric impression of the species' habitat drives the background — e.g., Sitka's fog-belt coast). A species may blend modes. Descriptive, not prescriptive — the value is in making the compositional rationale explicit and traceable. | — | Species Essence, Palette, Flora Reference |
| **Chroma Target** | A per-category specification defining the OKLCH chroma range and ΔE2000 perceptual band that a category's palette backgrounds must achieve, measured as CIEDE2000 color difference between the background and nearest-lightness neutral gray. Each target carries a perceptual keyword — "colored paper" (Vivid), "warm/cool tint" (Warm/Cool), "barely there" (Muted), "near-neutral" (Dark Muted) — describing the intended chromatic character. Dark categories use ΔE2000 ranges rather than raw chroma due to sRGB gamut constraints on dark saturated colors. Specific ranges are defined per ADR; the domain model establishes the concept and the principle that each category must have one. | — | Affective Category, Palette |
| **Perceptual Sort Order** | Ordering of palettes within an Affective Category by OKLCH hue angle, so adjacent palettes are perceptual neighbors. Produces a smooth browsing experience rather than jarring color jumps. | — | Affective Category, Palette Browser |
| **Color Profile** | The terminal's color capability: True Color (24-bit), 256-color, or basic ANSI. Detected at startup; rendering degrades gracefully. | — | Palette |
| **Chrome** | Any visible UI element that is not the writer's text: status bars, line numbers, file names, word counts. Hidden by default; summoned on demand. | "Distraction-free" — success measured by what the writer doesn't notice | Settings Layer |
| **Settings Layer** | The hidden interface for configuration, brought up by hotkey. Invisible during writing. Contains a single "Palette" row that opens the Palette Browser. | — | Chrome, Palette Browser |
| **Palette Browser** | A dedicated sub-panel within the Settings Layer for browsing, filtering, and selecting palettes. Organizes palettes by Affective Category with Perceptual Sort Order within each category. The two-level architecture (category → palette) means the writer faces ~8 category headers and ~5 palettes per group, never the full collection at once. | "Browse" — writer browses palettes by mood | Settings Layer, Affective Category, Perceptual Sort Order |
| **Local Config** | A `.zani.toml` file in a project directory that overrides global configuration for files opened from that directory or below it. Enables per-project palette binding. Resolved by walking up from the opened file's location. | "Local-first" — per-project settings on disk | Config Resolution, Palette |
| **Config Resolution** | The lookup order for settings: Local Config (`.zani.toml`, walk-up from file) → global config (`~/.config/zani/config.toml`) → built-in default. When the resolved palette differs from the currently active one, a crossfade animation handles the transition. | — | Local Config, Palette |
| **Writing Window** | A dedicated terminal window spawned by Zani with writing-optimized settings (font, line height, colors). Separate from the user's development terminal. | — | Inline Mode |
| **Inline Mode** | Running Zani inside the current terminal without spawning a Writing Window. This is the default behavior. Used for SSH, tmux, or pre-configured terminals. | — | Writing Window |
| **Markdown Styling** | Inline rendering of markdown: syntax characters (`#`, `**`, `*`, etc.) are visible but dimmed, while the text they modify receives formatting (bold, italic, color). The writer always sees and edits the raw markdown; styling is a visual layer, not a transformation. | "Beautiful" — part of the visual writing experience | Writing Surface, Dimming |
| **Smart Typography** | Automatic conversion of ASCII characters to typographic equivalents: straight quotes to curly quotes, `--` to em dash, `...` to ellipsis. | "Beautiful" — part of the visual writing experience | Writing Surface |
| **Vim Bindings** | Built-in modal editing with normal, insert, and visual modes. Bar cursor in insert mode, block cursor in normal mode. Not a plugin. First-class but not exclusive — non-vim editing is also supported. | — | |
| **Autosave** | Automatic saving of the Document to disk on a regular cadence or on pause. The writer never manually saves. | "Drop in" — trust that work is safe without thinking about saving | |
| **Git Integration** | Zani's awareness of and interaction with git. Scope and behavior TBD — requires a research spike. Zani should not make assumptions about the writer's git workflow. | "Local-first" — git handles versioning | Autosave, Document |

## Aliases (Terms to Avoid)

| Avoid | Use Instead | Reason |
|-------|-------------|--------|
| Draft | Document | "Draft" describes a stage, not the thing. A Document can be a draft or finished. |
| Viewport | Writing Surface | "Viewport" is too generic; Writing Surface captures the purpose. |
| Editor | Writing Surface or Zani | "Editor" implies code editing. Zani is a writing app. |
| Theme | Palette | "Theme" implies swappable skins. Palette is the specific color set. |
| Color Scheme | Palette | Same reasoning as "Theme" — too generic. |
| Mood / Mood Category | Affective Category | "Mood" is ambiguous (the writer's mood vs. the palette's intended effect). Affective Category is precise. |
| Plant (as register label) | Flora / Species | The Naming Register includes mosses, lichens, and fungi — "plant" is too narrow. Use "flora" for the register and "species" for individual entries. |
| PNW / Pacific Northwest (as geographic constraint) | Cascadia bioregion | The geographic scope is coastal BC through northern California, east to the Cascades crest. "Cascadia bioregion" is more precise than "Pacific Northwest," which can include areas outside the naming register's scope. |
| Color (of a species) | Signature Colors | When discussing a species' real botanical appearance, use "Signature Colors" — not "colors" or "color profile," which could be confused with the Palette's computed colors. |
| Plugin | (n/a) | Zani does not have a plugin system. Features are built in. |
| Opacity / Transparency (terminal) | Dimming | Terminals don't support true per-character transparency. Dimming uses color interpolation. Note: the internal opacity factor (0.0–1.0) is a rendering calculation, not terminal transparency. |
| Saturation | Chroma | "Saturation" means different things in different color spaces (HSL, HSV, OKLCH). OKLCH chroma is the perceptually uniform measure used for all palette color work. Use "chroma" when discussing color intensity. |

## Actions (Verbs)

| Action | Actor | Subject | Description |
|--------|-------|---------|-------------|
| **Write** | Writer | Document | The primary act: producing prose. Everything else serves this. |
| **Focus** | Writer | Focus Mode | Toggle or switch between Focus Mode variants (Sentence, Paragraph, Off). |
| **Dim** | Writing Surface | Text | Apply color interpolation to text outside the Active Region, fading it toward the background. |
| **Launch** | Zani | Writing Window | Detect the terminal emulator and spawn a dedicated Writing Window with writing-optimized settings. |
| **Render** | Writing Surface | Buffer | Transform the Buffer's text into styled, wrapped, positioned characters on the terminal screen. |
| **Wrap** | Writing Surface | Text | Soft-wrap prose to fit the centered column (~60 characters). Custom implementation, not ratatui's Paragraph. |
| **Scroll** | Writing Surface | Text | Move text relative to the viewport. In Edge mode, the viewport adjusts when the cursor nears the edges. In Typewriter mode, text moves around a fixed cursor position. Governed by Scroll Mode, independent of Focus Mode. |
| **Autosave** | Zani | Document | Persist the Buffer to disk automatically on a cadence or pause. |
| **Summon** | Writer | Settings Layer / Chrome | Bring up hidden UI elements via hotkey. The inverse of the default hidden state. |
| **Dismiss** | Writer | Settings Layer / Chrome | Hide summoned UI elements, returning to the bare writing state. |
| **Detect** | Zani | Color Profile / Terminal | Identify terminal emulator and color capabilities at startup. |
| **Degrade** | Zani | Palette | Fall back from True Color to 256-color to basic ANSI based on the detected Color Profile. For palettes with hand-tuned 256-color values, use those; otherwise fall back to automatic nearest-color mapping. |
| **Browse** | Writer | Palette Browser | Navigate the palette collection by Affective Category, with Perceptual Sort Order and optional name/category filtering. |
| **Bind** | Writer | Local Config | Associate a palette (or other settings) with a project by writing a `.zani.toml` file in the project directory. |
| **Resolve** | Zani | Config Resolution | Determine the active configuration by walking up from the opened file through Local Configs to global config to built-in default. |
| **Ingest** | Writer (via Zani) | Document / Selection | _(Planned, not yet implemented.)_ Send writing to Plexus for knowledge graph ingestion. On-save or on-demand, not real-time. Semantics are extracted automatically by Plexus. |
| **Invoke Ensemble** | Writer (via Zani) | Document / Selection | _(Planned, not yet implemented.)_ Trigger an llm-orc ensemble against the current text for analysis, critique, or research. A pull interaction. |

## Relationships

- A **Document** is represented in memory by exactly one **Buffer**
- The **Writing Surface** renders one **Buffer** at a time
- The **App Shell** contains exactly one **Writing Surface**
- A **Focus Mode** defines an **Active Region** within the visible text
- **Dimming** applies to all text outside the **Active Region**
- **Scroll Mode** and **Focus Mode** are orthogonal — neither influences the other
- **Typewriter Mode** is a **Scroll Mode** variant; it contributes zero dimming
- A **Palette** belongs to one **Affective Category**
- A **Palette** has exactly one **Provenance Description**
- A **Palette**'s name is drawn from the **Naming Register**
- The **Naming Register** constrains palette addition — no palette enters the collection without a corresponding PNW species whose color associations are moderately congruent with the palette's Affective Category
- An **Affective Category** contains many **Palettes**, sorted by **Perceptual Sort Order**
- Sibling **Palettes** within an **Affective Category** are spread across OKLCH hue space for maximum perceptual diversity
- A **Palette** is constrained by the detected **Color Profile**
- The **Writing Window** is spawned by **Launch**; **Inline Mode** skips it
- **Autosave** persists the **Document** to disk; independent of **Git Integration**
- **Chrome** and the **Settings Layer** are hidden by default, revealed by **Summon**, hidden by **Dismiss**
- The **Palette Browser** is a sub-panel of the **Settings Layer**, opened from a single "Palette" row
- A **Local Config** overrides global config for files in its directory and below
- **Config Resolution** walks up from file location through **Local Configs** to global config to default
- A **Palette** has exactly one **Species Essence** that determines which **Signature Colors** occupy which slots
- A **Species Essence** follows an **Essence Mode** (Feature, Throughline, or Place) that guides the composition
- The **Flora Reference** documents each species' **Signature Colors**, **Essence Mode**, geographic range, and source citations
- A **Palette**'s **Provenance Description** must be consistent with the **Flora Reference** — the reference is the source of truth
- The **Naming Register** is constrained by the **Flora Reference** — a species cannot enter the collection unless its **Signature Colors** are documented and verified against external sources
- An **Affective Category** has exactly one **Chroma Target** that its palette backgrounds must satisfy
- A **Palette**'s background must achieve a ΔE2000 from neutral within its **Affective Category**'s **Chroma Target** range
- **Browse** opens the **Palette Browser** from the **Settings Layer**
- **Bind** writes a **Local Config** file for the current project
- **Markdown Styling** is applied by the **Writing Surface** during **Render** — it reads syntax from the **Buffer** but does not modify it
- **Markdown Styling** uses **Dimming** on syntax characters and applies text attributes (bold, italic, color) to the modified text
- **Smart Typography** transforms characters within the **Buffer** during **Write**
- **Ingest** sends **Document** content to Plexus (external) _(planned, not yet implemented)_
- **Invoke Ensemble** sends **Document** content to llm-orc (external) _(planned, not yet implemented)_

## Invariants

1. **The tool disappears.** The default visual state is text, cursor, and empty space. No Chrome is visible unless explicitly summoned by the writer.

2. **Writing is the only default action.** When Zani is open, the only thing to do is write. All other interactions (settings, integrations, focus toggles) require deliberate invocation.

3. **No pure black or white. WCAG AA minimum.** The Palette never uses `#000000` or `#FFFFFF`. All foreground/background color pairs maintain at least a 4.5:1 contrast ratio (WCAG AA). Within these constraints, palettes are free to be warm, cool, vivid, muted, or anything else.

4. **Focus dimming is color interpolation, not terminal opacity.** Dimmed text uses per-character RGB interpolation toward the background color. Internally, each character's dimming is expressed as an opacity factor (0.0–1.0) which governs the interpolation amount. This is a rendering calculation, not a terminal transparency feature.

5. **The column is prose-width.** Text wraps at approximately 60 characters, centered in the terminal. Configurable between 20 and 120 characters.

6. **Latency is a UX requirement, not a performance metric.** Every keystroke must produce a visible result within the app layer's control in under 1ms. Architectural choices (Rust, zero-GC, immediate-mode rendering) serve this invariant.

7. **Markdown is the native format.** Documents are plain markdown files on disk. No proprietary format, no database, no intermediate representation. What git sees is what the writer wrote.

8. **External integrations are pull-only.** _(Planned, not yet implemented.)_ Plexus ingest and llm-orc ensemble invocations happen only when the writer explicitly requests them. Nothing runs in the background during writing.

9. **The Writing Window is opt-in.** When launched with `--window`, Zani spawns a dedicated terminal window with writing-optimized settings. Without `--window`, Zani runs inline in the current terminal. The writer's development terminal is unchanged.

10. **Markdown is always editable.** Markdown Styling is a render-time visual layer. The Buffer contains the raw markdown exactly as typed. Syntax characters are never hidden or removed — they are dimmed. The writer can always place their cursor on any character in the document.

11. **Graceful degradation, not feature gating.** If the terminal lacks True Color, Zani approximates with 256-color. If the terminal is unknown, Zani runs inline. If the recommended font isn't installed, Zani works with whatever font is present. Reduced capability, never failure.

12. **Scroll Mode and Focus Mode are orthogonal.** Scroll Mode (Edge, Typewriter) controls how the viewport follows the cursor. Focus Mode (Off, Sentence, Paragraph) controls which text is dimmed. Neither influences the other.

13. **Typewriter is a scroll behavior, not a dimming behavior.** Typewriter mode centers the cursor vertically. It contributes zero dimming. All dimming comes from Focus Mode.

14. **Dimming animations chase the current visual state.** When the target opacity changes (cursor moves to a new sentence/paragraph), the animation starts from whatever the character's current rendered opacity is. There is no "from" state to recompute. This guarantees interruption-safe transitions with no visual discontinuity.

15. **Dimming effects compose by multiplication.** If multiple dimming sources exist, each produces an opacity factor in [0.0, 1.0]. The final opacity is their product. This ensures independent sources can never brighten text — they can only dim further.

16. **Palette names are Cascadia flora with traceable provenance.** Every Palette is named after a Cascadia bioregion species (vascular plants, mosses, lichens, or fungi) whose Signature Colors — verified against external botanical sources and documented in the Flora Reference — are moderately congruent with the palette's color composition. Each name must be traceable to a specific species, register-consistent with the collection, uniquely evocative within its category, and resolvable when seen alongside the actual palette. The species' essence determines which Signature Color drives the palette's dominant slot; when the essence cannot be authentically represented in a given Affective Category, the species belongs in a different category or a different species should be chosen. The Provenance Description must be grounded in external sources — reflecting the species' real documented appearance, actual habitat range, and ecological context, not rationalizing the palette's colors after the fact.

17. **Composition is a deliberate design decision.** The assignment of a species' Signature Colors to specific palette slots (background, accents) is not arbitrary. The background dominates the visual field and determines the writer's immersive experience. The Species Essence — which visual feature the palette is "about" — determines which color occupies the dominant slot. The same species composed differently produces a different affective experience and may belong in a different category.

18. **Affective Categories are chromatically expressed.** Each Affective Category has a Chroma Target — a ΔE2000 range from neutral — specifying the perceptual band its backgrounds must achieve. Background chroma is the primary carrier of affective character; the character axis (Warm/Cool/Vivid/Muted) must produce distinguishable chromatic profiles in the dominant visual field, not only in accent punctuation. No individual palette's background should fall into a perceptual band below its category's intended character. For dark categories where sRGB gamut limits constrain background chroma, accent chroma provides compensatory reinforcement.

## Open Questions

1. **Save error visibility.** Autosave failures are only visible in the Settings Layer. A writer whose document fails to save silently will not know unless they open settings. What is the right signal that preserves the "tool disappears" invariant? Options: brief auto-dismissing indicator on the Writing Surface, subtle color shift in the margin, or accept the current behavior as consistent with minimal chrome.

2. **Focus mode cohesion.** Focus dimming logic spans four modules: `focus_mode.rs` (sentence parsing, opacity calculation), `dimming.rs` (orchestration), `writing_surface.rs` (per-character application), and `app.rs` (sentence bounds caching). Changing focus semantics requires edits across all four. Should all opacity computation consolidate into `DimmingState` so `WritingSurface` only applies pre-computed values?

3. **WritingSurface builder contract.** WritingSurface has 11 `Option<>` fields that are always provided by the single production call site. The fallback paths (which allocate) only run in test helpers. Should these fields become required (pushing test helpers to supply minimal data), or is the current arrangement acceptable with comments on each fallback branch?

4. **Integration test composition gap.** `focus_dimming_and_markdown_styling_compose` asserts that the result is `Color::Rgb` — a guarantee already provided by the type system for RGB inputs. It does not verify actual composed color values or rendering order. Should it render a real frame and assert specific cell colors, or be renamed to match what it actually verifies?

5. **Genre-matched palette efficacy.** The mood congruence principle (genre-matched palettes priming associated concepts and memories) is theoretically grounded but has not been directly tested in a writing context. Whether a "Neon Noir" palette measurably affects cyberpunk writing output differently than a "Morning Pages" palette remains speculative. *(Source: Reflection 002, §Intuition Confirmed and Extended)*

6. **Color priming habituation.** Whether sustained exposure to a palette (hours of writing) amplifies or habituates the affective priming effect is unknown. The research tested short exposures. *(Source: Research Log Q1, §What remains speculative)*

7. **Practical effect size.** Whether the magnitude of color-affect priming is large enough to matter in practice versus being a comfort/preference choice has not been established. The mechanism is real; the practical significance for writing output is uncertain. *(Source: Research Log Q1, §What remains speculative)*

8. **Project palette binding granularity.** Local Config currently binds a single palette name to a project. A writer might want a category preference or a shortlist of palettes for a project rather than a single locked-in choice — different sessions within the same project may call for different registers. The single-palette binding is the simplest version and enables the "immediately drop into the right palette" experience; the writer can always override in-session. Whether category-level or multi-palette binding is worth the added complexity should be evaluated after the single-palette version is in use. *(Source: Epistemic Gate, /rdd-model phase)*

9. **Provenance description display.** Where does the Provenance Description surface in the UI? Options: only in documentation, in the Palette Browser as a subtitle under each name, or as a detail view accessible from the browser. The "tool disappears" invariant (Invariant 1) argues against clutter; the educational value argues for accessibility. *(Source: Reflection 003, §Decisions from Gate Exchange)*

10. *(Resolved — Zani has no users yet; renaming palettes to PNW species carries no backwards-compatibility burden.)*

11. **Provenance bias.** Validation revealed that provenance descriptions contain subtle bias toward palette colors rather than real species colors. Balsamroot's provenance says "warm amber-gold" when field guides say "bright yellow." Witch's Hair says "olive-black" when the real species is pale yellow-green. The descriptions were written (or drifted) to rationalize the palette rather than document the species. Future provenance writing should be grounded in external sources first, before the palette colors are known. How should we systematically audit and correct existing descriptions? *(Source: Essay 004, §Provenance Errors; Reflection 004, §Open question: provenance bias)*

12. **Flora Reference maintenance cadence.** The Flora Reference is a durable corpus, but when should it be updated? Options: only when species are added/replaced, on a periodic audit cadence (e.g., annually), or continuously as new sources are found. The validation methodology (A/B/C/D scoring against external sources) provides a repeatable framework, but the trigger for running it is undefined. *(Source: Reflection 004, §"Well curated" means ongoing verification)*

13. **Essence Mode classification completeness.** Three modes are identified (Feature, Throughline, Place), but a species may blend modes (e.g., Madrone has feature essence from peeling bark and place essence from dry oak savannah hillsides). Is a primary/secondary classification needed, or is the current "descriptive, not prescriptive" framing sufficient for guiding palette design? *(Source: Reflection 004, §Three modes of essence)*

14. **Constraint-satisfaction cost distribution.** The 15° OKLCH hue diversity and WCAG AA contrast constraints are not equally costly across species. Species with distinctive, high-chroma signature colors (vivid yellow, bright orange) are more vulnerable to drift than species with broad, muted color profiles (bark, lichen, foliage). Should species selection prefer those whose real colors already sit where the constraints need them to be, or should the constraints flex? *(Source: Reflection 004, §What gives, and the order matters)*

15. *(Resolved — Chroma Target failure is signal, not exception. A species that cannot achieve its category's Chroma Target is in the wrong category or is the wrong species. Invariant 16 already prescribes the remedy: reassign or replace. Chroma Targets make the threshold measurable, turning a subjective judgment into a concrete test. There is no "accept the outlier" option. Source: Essay 005, §Invariant Alignment; Epistemic Gate, /rdd-model phase.)*

16. **Cold-start discoverability.** A new writer opening Zani for the first time encounters text, cursor, and empty space — Invariant 1 working as designed. But they may never discover focus modes, palette browsing, typewriter mode, or local config binding without external guidance. Two candidate solutions: (a) a help dialog on first launch, dismissible and disableable in settings, or (b) launching into the Settings Layer on first run so the writer sees available options, then disabling automatic settings-on-launch. Neither has been designed or validated. Bundled with Open Question 19 (distribution) — these are one problem, not two. *(Source: Product Discovery, §Value Tensions — Invisibility vs. discoverability)*

17. **Alternative naming registers.** The architecture could support additional curated palette collections beyond Cascadian flora — celestial bodies, geological formations, etc. — each conforming to the same affective essentialism standards (chroma targets, WCAG AA, hue diversity, species/source congruence). But no mechanism exists to install, switch between, or manage multiple registers. The boundary between "curated extension" (acceptable) and "user customization" (not acceptable) is undefined. *(Source: Product Discovery, §Value Tensions — Curation vs. personalization)*

18. **Default editing mode.** Vim bindings are first-class but non-vim editing is also supported. A writer who opens Zani cold needs to know which mode they're in and how to start typing. Directional lean: the default should be non-vim (insert mode) for the broadest audience, with first-class vim activation for power users. The builder (as a vim user) also wants deeper vim motion support — both can coexist if the default is non-vim with comprehensive vim available. *(Source: Product Discovery, §Value Tensions — Vim-first vs. approachable; §Product Debt; Epistemic Gate conversation)*

19. **Distribution and onboarding.** Writers who would love Zani need to find it, install it, and learn it. No install mechanism, packaging, or onboarding flow exists. Candidate distribution channels: Homebrew formula, `cargo install`, wrapped distribution. The broader question: is the target audience "writers who already use terminals" or "writers who would love a terminal writing app but don't know terminals yet"? The answer determines whether distribution is a nice-to-have or product-critical. *(Source: Product Discovery, §Assumption Inversions — terminal-comfortable assumption)*

## Amendment Log

| # | Date | Invariant | Change | Propagation |
|---|------|-----------|--------|-------------|
| 1 | 2026-02-26 | Invariant 9 | Changed from "Writing Window is the default" to "Writing Window is opt-in (`--window` flag)". Inline is now the default. | ADR-003 superseded by ADR-007. Writing Window scenarios updated. |
| 2 | 2026-02-27 | Invariant 4 | Clarified: internal opacity factor (0.0–1.0) is a rendering calculation, not terminal transparency. | ADR-004 unchanged; ADR-008 added. |
| 3 | 2026-02-27 | Invariants 12–15 | Added: Scroll/Focus orthogonality, Typewriter is scroll-only, chase-based animation, multiplicative dimming composition. | ADR-008 covers the full redesign. Focus Mode concept updated (Typewriter removed). Scroll Mode concept added. Fade Config concept added. |
| 4 | 2026-03-06 | Affective Category | Expanded character axis from (Warm, Cool, Vivid) to (Warm, Cool, Vivid, Muted). Six categories become eight. Grounded in convergent factor-analytic evidence: Ou et al. (2004), Kobayashi (1981), Valdez & Mehrabian (1994), Jonauskaite & Mohr (2025). Muted is the low-saturation pole of the Activity/Chroma axis — a distinct affective register (contemplative, subdued, reflective), not the absence of vividness. | ADR-009 (palette affective categories) needs update. `AffectiveCategory` enum in `palette.rs` needs `DarkMuted` and `LightMuted` variants. Palette Browser layout gains two additional category sections. |
| 5 | 2026-03-06 | Invariant 3 | Minor wording: added "muted" to the list of permissible palette characters ("warm, cool, vivid, muted, or anything else"). No substantive change. | None. |
| 6 | 2026-03-06 | Invariant 16 | Added: Palette names are PNW flora with traceable provenance. Establishes the Naming Register as a constitutional constraint on the collection. All existing palette names (Ember, Hearthstone, etc.) must be renamed to PNW species. | All palette definitions in `palette.rs` must be renamed. Default palette becomes Manzanita (replacing Ember). Config files referencing old names need migration (see Open Question 10). ADR-006 (curated palette collection) needs update to reflect naming register. |
| 7 | 2026-03-06 | Concepts | Added: Naming Register, Provenance Description. Amended: Palette (name constraint, provenance, Manzanita as default), Affective Category (Muted added, density guideline), Palette Browser (two-level architecture noted). Added alias: "Plant" → "Flora / Species". | New concepts are referenced by Palette; no existing concept definitions contradicted. |
| 8 | 2026-03-07 | Invariant 16 | **Strengthened.** Added requirements: Signature Colors must be verified against external botanical sources and documented in the Flora Reference. Species essence determines which Signature Color drives the dominant slot. When the essence cannot be authentically represented in a given Affective Category, the species belongs in a different category or should be replaced. Provenance must reflect real documented appearance and actual habitat, not rationalize palette colors. Scope clarified from "PNW" to "Cascadia bioregion." | ADR-015 (naming register) needs update. All 6 identified provenance errors need correction in `collection.rs`. Species replacement decisions (Bracken, Licorice Fern, Silver Fir) and Oregon Sunshine category move are direct applications of the strengthened invariant. |
| 9 | 2026-03-07 | Invariant 17 | **Added.** Composition is a deliberate design decision — background dominates the visual field, Species Essence determines which color occupies the dominant slot, same species composed differently produces a different affective experience and may belong in a different category. | New invariant. No existing documents contradict; the principle was implicit in the palette design process but not codified. |
| 10 | 2026-03-07 | Concepts | Added: Flora Reference, Signature Colors, Species Essence, Essence Mode. Amended: Palette (composition/essence language, background dominance), Naming Register (Cascadia bioregion, Signature Colors verification), Provenance Description (external source grounding), Affective Category (essence/composition relationship, "approximately five" density). Added aliases: "PNW" → "Cascadia bioregion", "Color (of a species)" → "Signature Colors". | New concepts are referenced by Palette and Naming Register. Flora Reference (`docs/flora-reference.md`) does not yet exist — creation is a downstream deliverable. |
| 11 | 2026-03-07 | Invariant 18 | **Added.** Affective Categories are chromatically expressed — each category has a Chroma Target (ΔE2000 range from neutral) that its backgrounds must achieve. Background chroma is the primary carrier of affective character. Dark categories may use accent chroma as compensatory reinforcement where sRGB gamut constrains background chroma. | New invariant. Operationalizes what Invariant 17 (background dominance) already implies. All 40 palette backgrounds must be audited against their category's Chroma Target. Specific target ranges to be defined in an ADR. |
| 12 | 2026-03-07 | Concepts | Added: Chroma Target. Amended: Affective Category (chromatic expression language, Chroma Target reference). Added alias: "Saturation" → "Chroma". Added relationships: Affective Category has one Chroma Target; Palette background must satisfy its category's Chroma Target. | Chroma Target is a new concept referenced by Affective Category and Palette. ADR-009 (palette affective categories) and all palette definitions in `collection.rs` are affected. |
| 13 | 2026-03-14 | Concepts | Added Product Origin column to Concepts table, tracing each concept to user-facing vocabulary from Product Discovery. Amended: Vim Bindings definition (added "first-class but not exclusive — non-vim editing is also supported"). Added Open Questions 16–19 from Product Discovery value tensions (cold-start discoverability, alternative naming registers, default editing mode, distribution and onboarding). | No invariants changed. Product Discovery artifact (`docs/product-discovery.md`) created as the source document. System design and ADRs may need Product Origin provenance added in subsequent conformance passes. |
