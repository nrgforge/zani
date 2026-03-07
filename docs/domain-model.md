# Domain Model: Zani

## Concepts (Nouns)

| Term | Definition | Related Terms |
|------|-----------|---------------|
| **Document** | A markdown file opened for editing. The unit of work in Zani. | Buffer, Draft |
| **Buffer** | The in-memory representation of a Document's text, managed as a rope data structure (Ropey). Supports efficient insertion, deletion, and cloning. | Document |
| **Writing Surface** | The custom text viewport where prose is rendered. Handles soft-wrapping, scroll positioning, and per-character styling. Built on ratatui's cell buffer, bypassing the Paragraph widget. | App Shell, Focus Mode |
| **App Shell** | The ratatui application frame that manages layout, input routing, and the event loop. Contains the Writing Surface but does not render prose directly. | Writing Surface |
| **Focus Mode** | A visual mode that dims text outside the active region to keep the writer in generative mode. Two variants: Sentence, Paragraph. Orthogonal to Scroll Mode. | Dimming, Active Region, Scroll Mode |
| **Active Region** | The sentence or paragraph currently at full brightness during a Focus Mode. Everything outside it is dimmed. | Focus Mode, Dimming |
| **Dimming** | Per-character color interpolation from full foreground brightness toward the background color, creating a visual fade. Internally expressed as an opacity factor (0.0–1.0) per character, which governs the interpolation amount. The mechanism behind Focus Mode and Markdown Styling. | Focus Mode, Palette, Color Profile |
| **Scroll Mode** | How the viewport follows the cursor. Two variants: Edge (scroll when cursor nears edges) and Typewriter (cursor stays vertically centered). Orthogonal to Focus Mode. | Focus Mode, Writing Surface |
| **Typewriter Mode** | A Scroll Mode variant where the cursor stays vertically centered and text scrolls around it, eliminating manual scrolling. Does not contribute dimming. | Scroll Mode |
| **Fade Config** | A pairing of duration and easing curve that governs how a dimming transition animates. Each dimming source specifies separate configs for fade-in (brightening) and fade-out (dimming). | Dimming, Active Region |
| **Palette** | A named, curated color system defining foreground, background, dimming endpoints, and accent colors. Designed as a mood instrument — priming a specific affective state rather than serving as decoration. Belongs to an Affective Category. May include hand-tuned 256-color alternate values for graceful degradation. All palettes satisfy Invariant 3. | Affective Category, Color Profile, Perceptual Sort Order |
| **Affective Category** | A mood-based grouping of palettes along two axes: brightness (Dark, Light) and character (Warm, Cool, Vivid). Examples: "Dark — Warm", "Dark — Vivid", "Light — Cool". The organizing taxonomy for the Palette Browser. | Palette, Palette Browser |
| **Perceptual Sort Order** | Ordering of palettes within an Affective Category by OKLCH hue angle, so adjacent palettes are perceptual neighbors. Produces a smooth browsing experience rather than jarring color jumps. | Affective Category, Palette Browser |
| **Color Profile** | The terminal's color capability: True Color (24-bit), 256-color, or basic ANSI. Detected at startup; rendering degrades gracefully. | Palette |
| **Chrome** | Any visible UI element that is not the writer's text: status bars, line numbers, file names, word counts. Hidden by default; summoned on demand. | Settings Layer |
| **Settings Layer** | The hidden interface for configuration, brought up by hotkey. Invisible during writing. Contains a single "Palette" row that opens the Palette Browser. | Chrome, Palette Browser |
| **Palette Browser** | A dedicated sub-panel within the Settings Layer for browsing, filtering, and selecting palettes. Organizes palettes by Affective Category with Perceptual Sort Order within each category. Replaces inline palette rows when the collection exceeds a few entries. | Settings Layer, Affective Category, Perceptual Sort Order |
| **Local Config** | A `.zani.toml` file in a project directory that overrides global configuration for files opened from that directory or below it. Enables per-project palette binding. Resolved by walking up from the opened file's location. | Config Resolution, Palette |
| **Config Resolution** | The lookup order for settings: Local Config (`.zani.toml`, walk-up from file) → global config (`~/.config/zani/config.toml`) → built-in default. When the resolved palette differs from the currently active one, a crossfade animation handles the transition. | Local Config, Palette |
| **Writing Window** | A dedicated terminal window spawned by Zani with writing-optimized settings (font, line height, colors). Separate from the user's development terminal. | Inline Mode |
| **Inline Mode** | Running Zani inside the current terminal without spawning a Writing Window. This is the default behavior. Used for SSH, tmux, or pre-configured terminals. | Writing Window |
| **Markdown Styling** | Inline rendering of markdown: syntax characters (`#`, `**`, `*`, etc.) are visible but dimmed, while the text they modify receives formatting (bold, italic, color). The writer always sees and edits the raw markdown; styling is a visual layer, not a transformation. | Writing Surface, Dimming |
| **Smart Typography** | Automatic conversion of ASCII characters to typographic equivalents: straight quotes to curly quotes, `--` to em dash, `...` to ellipsis. | Writing Surface |
| **Vim Bindings** | Built-in modal editing with normal, insert, and visual modes. Bar cursor in insert mode, block cursor in normal mode. Not a plugin. | |
| **Autosave** | Automatic saving of the Document to disk on a regular cadence or on pause. The writer never manually saves. | |
| **Git Integration** | Zani's awareness of and interaction with git. Scope and behavior TBD — requires a research spike. Zani should not make assumptions about the writer's git workflow. | Autosave, Document |

## Aliases (Terms to Avoid)

| Avoid | Use Instead | Reason |
|-------|-------------|--------|
| Draft | Document | "Draft" describes a stage, not the thing. A Document can be a draft or finished. |
| Viewport | Writing Surface | "Viewport" is too generic; Writing Surface captures the purpose. |
| Editor | Writing Surface or Zani | "Editor" implies code editing. Zani is a writing app. |
| Theme | Palette | "Theme" implies swappable skins. Palette is the specific color set. |
| Color Scheme | Palette | Same reasoning as "Theme" — too generic. |
| Mood / Mood Category | Affective Category | "Mood" is ambiguous (the writer's mood vs. the palette's intended effect). Affective Category is precise. |
| Plugin | (n/a) | Zani does not have a plugin system. Features are built in. |
| Opacity / Transparency (terminal) | Dimming | Terminals don't support true per-character transparency. Dimming uses color interpolation. Note: the internal opacity factor (0.0–1.0) is a rendering calculation, not terminal transparency. |

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
- An **Affective Category** contains many **Palettes**, sorted by **Perceptual Sort Order**
- A **Palette** is constrained by the detected **Color Profile**
- The **Writing Window** is spawned by **Launch**; **Inline Mode** skips it
- **Autosave** persists the **Document** to disk; independent of **Git Integration**
- **Chrome** and the **Settings Layer** are hidden by default, revealed by **Summon**, hidden by **Dismiss**
- The **Palette Browser** is a sub-panel of the **Settings Layer**, opened from a single "Palette" row
- A **Local Config** overrides global config for files in its directory and below
- **Config Resolution** walks up from file location through **Local Configs** to global config to default
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

3. **No pure black or white. WCAG AA minimum.** The Palette never uses `#000000` or `#FFFFFF`. All foreground/background color pairs maintain at least a 4.5:1 contrast ratio (WCAG AA). Within these constraints, palettes are free to be warm, cool, vivid, subdued, or anything else.

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

## Open Questions

1. **Save error visibility.** Autosave failures are only visible in the Settings Layer. A writer whose document fails to save silently will not know unless they open settings. What is the right signal that preserves the "tool disappears" invariant? Options: brief auto-dismissing indicator on the Writing Surface, subtle color shift in the margin, or accept the current behavior as consistent with minimal chrome.

2. **Focus mode cohesion.** Focus dimming logic spans four modules: `focus_mode.rs` (sentence parsing, opacity calculation), `dimming.rs` (orchestration), `writing_surface.rs` (per-character application), and `app.rs` (sentence bounds caching). Changing focus semantics requires edits across all four. Should all opacity computation consolidate into `DimmingState` so `WritingSurface` only applies pre-computed values?

3. **WritingSurface builder contract.** WritingSurface has 11 `Option<>` fields that are always provided by the single production call site. The fallback paths (which allocate) only run in test helpers. Should these fields become required (pushing test helpers to supply minimal data), or is the current arrangement acceptable with comments on each fallback branch?

4. **Integration test composition gap.** `focus_dimming_and_markdown_styling_compose` asserts that the result is `Color::Rgb` — a guarantee already provided by the type system for RGB inputs. It does not verify actual composed color values or rendering order. Should it render a real frame and assert specific cell colors, or be renamed to match what it actually verifies?

5. **Genre-matched palette efficacy.** The mood congruence principle (genre-matched palettes priming associated concepts and memories) is theoretically grounded but has not been directly tested in a writing context. Whether a "Neon Noir" palette measurably affects cyberpunk writing output differently than a "Morning Pages" palette remains speculative. *(Source: Reflection 002, §Intuition Confirmed and Extended)*

6. **Color priming habituation.** Whether sustained exposure to a palette (hours of writing) amplifies or habituates the affective priming effect is unknown. The research tested short exposures. *(Source: Research Log Q1, §What remains speculative)*

7. **Practical effect size.** Whether the magnitude of color-affect priming is large enough to matter in practice versus being a comfort/preference choice has not been established. The mechanism is real; the practical significance for writing output is uncertain. *(Source: Research Log Q1, §What remains speculative)*

8. **Project palette binding granularity.** Local Config currently binds a single palette name to a project. A writer might want a category preference or a shortlist of palettes for a project rather than a single locked-in choice — different sessions within the same project may call for different registers. The single-palette binding is the simplest version and enables the "immediately drop into the right palette" experience; the writer can always override in-session. Whether category-level or multi-palette binding is worth the added complexity should be evaluated after the single-palette version is in use. *(Source: Epistemic Gate, /rdd-model phase)*

## Amendment Log

| # | Date | Invariant | Change | Propagation |
|---|------|-----------|--------|-------------|
| 1 | 2026-02-26 | Invariant 9 | Changed from "Writing Window is the default" to "Writing Window is opt-in (`--window` flag)". Inline is now the default. | ADR-003 superseded by ADR-007. Writing Window scenarios updated. |
| 2 | 2026-02-27 | Invariant 4 | Clarified: internal opacity factor (0.0–1.0) is a rendering calculation, not terminal transparency. | ADR-004 unchanged; ADR-008 added. |
| 3 | 2026-02-27 | Invariants 12–15 | Added: Scroll/Focus orthogonality, Typewriter is scroll-only, chase-based animation, multiplicative dimming composition. | ADR-008 covers the full redesign. Focus Mode concept updated (Typewriter removed). Scroll Mode concept added. Fade Config concept added. |
