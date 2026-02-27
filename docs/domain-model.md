# Domain Model: Zani

## Concepts (Nouns)

| Term | Definition | Related Terms |
|------|-----------|---------------|
| **Document** | A markdown file opened for editing. The unit of work in Zani. | Buffer, Draft |
| **Buffer** | The in-memory representation of a Document's text, managed as a rope data structure (Ropey). Supports efficient insertion, deletion, and cloning. | Document |
| **Writing Surface** | The custom text viewport where prose is rendered. Handles soft-wrapping, scroll positioning, and per-character styling. Built on ratatui's cell buffer, bypassing the Paragraph widget. | App Shell, Focus Mode |
| **App Shell** | The ratatui application frame that manages layout, input routing, and the event loop. Contains the Writing Surface but does not render prose directly. | Writing Surface |
| **Focus Mode** | A visual mode that dims text outside the active region to keep the writer in generative mode. Three variants: Sentence, Paragraph, Typewriter. | Dimming, Active Region |
| **Active Region** | The sentence, paragraph, or line currently at full brightness during a Focus Mode. Everything outside it is dimmed. | Focus Mode, Dimming |
| **Dimming** | Per-character color interpolation from full foreground brightness toward the background color, creating a visual fade. The mechanism behind Focus Mode and Markdown Styling. | Focus Mode, Palette, Color Profile |
| **Typewriter Mode** | A Focus Mode variant where the cursor stays vertically centered and text scrolls around it, eliminating manual scrolling. | Focus Mode |
| **Palette** | A named, curated color system defining foreground, background, dimming endpoints, and accent colors. Each palette has a mood and character (e.g., Campfire, Manila, Legal Pad). All palettes satisfy Invariant 3. The writer selects from the collection. | Color Profile |
| **Color Profile** | The terminal's color capability: True Color (24-bit), 256-color, or basic ANSI. Detected at startup; rendering degrades gracefully. | Palette |
| **Chrome** | Any visible UI element that is not the writer's text: status bars, line numbers, file names, word counts. Hidden by default; summoned on demand. | Settings Layer |
| **Settings Layer** | The hidden interface for configuration, brought up by hotkey. Invisible during writing. | Chrome |
| **Writing Window** | A dedicated terminal window spawned by Zani with writing-optimized settings (font, line height, colors). Separate from the user's development terminal. | Inline Mode |
| **Inline Mode** | Running Zani inside the current terminal without spawning a Writing Window. Activated via `--inline` flag, used for SSH, tmux, or pre-configured terminals. | Writing Window |
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
| Plugin | (n/a) | Zani does not have a plugin system. Features are built in. |
| Opacity / Transparency | Dimming | Terminals don't support true opacity. Dimming is the actual mechanism (color interpolation). |

## Actions (Verbs)

| Action | Actor | Subject | Description |
|--------|-------|---------|-------------|
| **Write** | Writer | Document | The primary act: producing prose. Everything else serves this. |
| **Focus** | Writer | Focus Mode | Toggle or switch between Focus Mode variants (Sentence, Paragraph, Typewriter, Off). |
| **Dim** | Writing Surface | Text | Apply color interpolation to text outside the Active Region, fading it toward the background. |
| **Launch** | Zani | Writing Window | Detect the terminal emulator and spawn a dedicated Writing Window with writing-optimized settings. |
| **Render** | Writing Surface | Buffer | Transform the Buffer's text into styled, wrapped, positioned characters on the terminal screen. |
| **Wrap** | Writing Surface | Text | Soft-wrap prose to fit the centered column (~60 characters). Custom implementation, not ratatui's Paragraph. |
| **Scroll** | Writing Surface | Text | Move text relative to the viewport. In Typewriter Mode, text moves around a fixed cursor position. |
| **Autosave** | Zani | Document | Persist the Buffer to disk automatically on a cadence or pause. |
| **Summon** | Writer | Settings Layer / Chrome | Bring up hidden UI elements via hotkey. The inverse of the default hidden state. |
| **Dismiss** | Writer | Settings Layer / Chrome | Hide summoned UI elements, returning to the bare writing state. |
| **Detect** | Zani | Color Profile / Terminal | Identify terminal emulator and color capabilities at startup. |
| **Degrade** | Zani | Palette | Fall back from True Color to 256-color to basic ANSI based on the detected Color Profile. |
| **Ingest** | Writer (via Zani) | Document / Selection | Send writing to Plexus for knowledge graph ingestion. On-save or on-demand, not real-time. Semantics are extracted automatically by Plexus. |
| **Invoke Ensemble** | Writer (via Zani) | Document / Selection | Trigger an llm-orc ensemble against the current text for analysis, critique, or research. A pull interaction. |

## Relationships

- A **Document** is represented in memory by exactly one **Buffer**
- The **Writing Surface** renders one **Buffer** at a time
- The **App Shell** contains exactly one **Writing Surface**
- A **Focus Mode** defines an **Active Region** within the visible text
- **Dimming** applies to all text outside the **Active Region**
- A **Palette** is constrained by the detected **Color Profile**
- The **Writing Window** is spawned by **Launch**; **Inline Mode** skips it
- **Autosave** persists the **Document** to disk; independent of **Git Integration**
- **Chrome** and the **Settings Layer** are hidden by default, revealed by **Summon**, hidden by **Dismiss**
- **Markdown Styling** is applied by the **Writing Surface** during **Render** — it reads syntax from the **Buffer** but does not modify it
- **Markdown Styling** uses **Dimming** on syntax characters and applies text attributes (bold, italic, color) to the modified text
- **Smart Typography** transforms characters within the **Buffer** during **Write**
- **Ingest** sends **Document** content to Plexus (external)
- **Invoke Ensemble** sends **Document** content to llm-orc (external)

## Invariants

1. **The tool disappears.** The default visual state is text, cursor, and empty space. No Chrome is visible unless explicitly summoned by the writer.

2. **Writing is the only default action.** When Zani is open, the only thing to do is write. All other interactions (settings, integrations, focus toggles) require deliberate invocation.

3. **No pure black or white. WCAG AA minimum.** The Palette never uses `#000000` or `#FFFFFF`. All foreground/background color pairs maintain at least a 4.5:1 contrast ratio (WCAG AA). Within these constraints, palettes are free to be warm, cool, vivid, subdued, or anything else.

4. **Focus dimming is color interpolation, not opacity.** Dimmed text uses per-character RGB interpolation toward the background color. This requires only True Color support, not terminal transparency.

5. **The column is prose-width.** Text wraps at approximately 60 characters, centered in the terminal. Not configurable below 45 or above 80.

6. **Latency is a UX requirement, not a performance metric.** Every keystroke must produce a visible result within the app layer's control in under 1ms. Architectural choices (Rust, zero-GC, immediate-mode rendering) serve this invariant.

7. **Markdown is the native format.** Documents are plain markdown files on disk. No proprietary format, no database, no intermediate representation. What git sees is what the writer wrote.

8. **External integrations are pull-only.** Plexus ingest and llm-orc ensemble invocations happen only when the writer explicitly requests them. Nothing runs in the background during writing.

9. **The Writing Window is opt-in.** When launched with `--window`, Zani spawns a dedicated terminal window with writing-optimized settings. Without `--window`, Zani runs inline in the current terminal. The writer's development terminal is unchanged.

10. **Markdown is always editable.** Markdown Styling is a render-time visual layer. The Buffer contains the raw markdown exactly as typed. Syntax characters are never hidden or removed — they are dimmed. The writer can always place their cursor on any character in the document.

11. **Graceful degradation, not feature gating.** If the terminal lacks True Color, Zani approximates with 256-color. If the terminal is unknown, Zani runs inline. If the recommended font isn't installed, Zani works with whatever font is present. Reduced capability, never failure.

## Amendment Log

| # | Date | Invariant | Change | Propagation |
|---|------|-----------|--------|-------------|
| 1 | 2026-02-26 | Invariant 9 | Changed from "Writing Window is the default" to "Writing Window is opt-in (`--window` flag)". Inline is now the default. | ADR-003 superseded by ADR-007. Writing Window scenarios updated. |
