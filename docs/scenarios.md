# Behavior Scenarios

## Feature: Writing Surface Rendering

### Scenario: Text wraps at prose-width column
**Given** a Document with a paragraph longer than 60 characters
**When** the Writing Surface renders the Buffer
**Then** the text wraps at approximately 60 characters without breaking mid-word
**And** the wrapped text is centered horizontally in the terminal

### Scenario: Writing Surface renders directly to ratatui cell buffer
**Given** a Document with text content
**When** the Writing Surface renders
**Then** characters are written to ratatui's cell buffer without using the Paragraph widget
**And** each cell has the correct character, foreground color, and text attributes

### Scenario: Cursor positioning accounts for soft-wrapped lines
**Given** a Document with a paragraph that wraps to 3 visual lines
**When** the cursor is on the second visual line (middle of the paragraph)
**Then** the cursor appears at the correct row and column on screen
**And** arrow key movement follows visual lines, not logical lines

### Scenario: Scroll position accounts for wrapped lines
**Given** a Document long enough to exceed the terminal height
**When** the writer scrolls down by one line
**Then** the Writing Surface advances by one visual line (not one paragraph)
**And** the scroll position is accurate regardless of how many lines each paragraph wraps to

---

## Feature: Focus Mode

### Scenario: Sentence Focus Mode dims surrounding text
**Given** Focus Mode is set to Sentence
**When** the cursor is in the middle of a sentence
**Then** that sentence renders at the Palette's full foreground color
**And** all other text renders with foreground color interpolated toward the background color

### Scenario: Paragraph Focus Mode dims surrounding text
**Given** Focus Mode is set to Paragraph
**When** the cursor is in a paragraph
**Then** that paragraph renders at the Palette's full foreground color
**And** adjacent paragraphs render at a dimmed foreground color
**And** paragraphs further away render at a more dimmed foreground color

### Scenario: Focus Mode off shows all text at full brightness
**Given** Focus Mode is Off
**When** the Writing Surface renders
**Then** all text renders at the Palette's full foreground color
**And** no Dimming is applied

### Scenario: Focus Mode toggle
**Given** the writer is in any Focus Mode (or Off)
**When** the writer invokes the Focus Mode toggle
**Then** the Focus Mode cycles to the next variant (Off → Sentence → Paragraph → Off)
**And** the Writing Surface re-renders immediately with the new Dimming

---

## Feature: Markdown Styling

### Scenario: Bold text is styled with visible syntax characters
**Given** a Document containing `**bold text**`
**When** the Writing Surface renders that line
**Then** `bold text` renders with the bold text attribute at full foreground color
**And** the `**` characters render with the bold text attribute at dimmed foreground color

### Scenario: Italic text is styled with visible syntax characters
**Given** a Document containing `*italic text*`
**When** the Writing Surface renders that line
**Then** `italic text` renders with the italic text attribute at full foreground color
**And** the `*` characters render with the italic text attribute at dimmed foreground color

### Scenario: Headings are styled with visible hash marks
**Given** a Document containing `## Section Title`
**When** the Writing Surface renders that line
**Then** `Section Title` renders in bold with a distinct accent color from the Palette
**And** the `## ` prefix renders at dimmed foreground color

### Scenario: Code blocks are styled as source text
**Given** a Document containing a fenced code block (` ```language ... ``` `)
**When** the Writing Surface renders the block
**Then** the content renders as plain styled text (no execution, no diagram rendering)
**And** the fence markers render at dimmed foreground color

### Scenario: Markdown syntax is never removed from the Buffer
**Given** a Document containing any markdown syntax
**When** the Writing Surface applies Markdown Styling
**Then** the Buffer's content is unchanged — every character the writer typed is preserved
**And** the writer can place the cursor on any syntax character

---

## Feature: Palette

### Scenario: Default palette uses no pure black or white
**Given** Zani starts with the default Palette
**When** the Writing Surface renders any text
**Then** no cell has a foreground color of `rgb(0, 0, 0)` or `rgb(255, 255, 255)`
**And** no cell has a background color of `rgb(0, 0, 0)` or `rgb(255, 255, 255)`

### Scenario: All palette color pairs meet WCAG AA
**Given** any built-in Palette
**When** the foreground and background colors are measured
**Then** every foreground/background pair has a contrast ratio of at least 4.5:1

### Scenario: Dimming interpolation works against the active palette
**Given** a Palette with a specific background color
**When** Focus Mode is active and Dimming is applied
**Then** dimmed text foreground colors interpolate toward that Palette's background color (not a hardcoded value)

### Scenario: Writer switches palette via Settings Layer
**Given** the writer is in a writing session with Palette A
**And** the Settings Layer is visible
**When** the writer selects a different Palette from the Palette list
**Then** the Writing Surface re-renders with the selected Palette's colors
**And** all Dimming interpolation uses the selected Palette's endpoints
**And** the Settings Layer reflects the newly active Palette

---

## Feature: Chrome and Settings Layer

### Scenario: Default state has no visible Chrome
**Given** Zani launches and opens a Document
**When** the initial render completes
**Then** the only visible elements are the Document text, the cursor, and empty space
**And** no status bar, mode indicator, line numbers, file name, or word count is visible

### Scenario: Settings Layer is summoned by hotkey
**Given** the writer is in the default chromeless writing state
**When** the writer presses the Settings Layer hotkey
**Then** the Settings Layer appears as an overlay on the Writing Surface
**And** the overlay lists the current Palette name, Focus Mode, and column width
**And** the Writing Surface remains visible behind the overlay

### Scenario: Settings Layer shows Palette selection
**Given** the Settings Layer is visible
**When** the writer views the Palette section
**Then** the available Palette names are listed
**And** the currently active Palette is indicated

### Scenario: Settings Layer shows Focus Mode selection
**Given** the Settings Layer is visible
**When** the writer views the Focus Mode section
**Then** the Focus Mode options (Off, Sentence, Paragraph) are listed
**And** the currently active Focus Mode is indicated

### Scenario: Settings Layer is dismissed
**Given** the Settings Layer is visible
**When** the writer presses the dismiss hotkey (or Escape)
**Then** the Settings Layer disappears
**And** the writing state returns to chromeless default

### Scenario: Settings Layer shows status information
**Given** the Settings Layer is visible
**When** the writer views the Settings Layer
**Then** the current vim mode, file name, and dirty state are visible
**And** this information is only visible while the Settings Layer is summoned

---

## Feature: Writing Window

### Scenario: Zani spawns a Writing Window with --window flag
**Given** the writer is in a supported terminal (Ghostty, Kitty, WezTerm, Alacritty, iTerm2)
**When** the writer runs `zani --window document.md`
**Then** a new terminal window opens with writing-optimized settings (font, size, line height)
**And** Zani runs inside that window with `ZANI_WINDOW=1` set
**And** the original terminal is unchanged

### Scenario: Default launch runs inline
**Given** the writer runs `zani document.md` without the `--window` flag
**When** Zani starts
**Then** it runs inside the current terminal without spawning a new window

### Scenario: Zani does not re-spawn when already in a Writing Window
**Given** `ZANI_WINDOW=1` is set in the environment
**When** Zani starts with `--window`
**Then** it runs inline without spawning another window

### Scenario: Inline Mode on --inline flag `[Superseded by ADR-007]`
**Given** the writer runs `zani --inline document.md`
**When** Zani starts
**Then** it runs inside the current terminal without spawning a new window

> Inline is the default behavior; `--inline` flag removed.

### Scenario: Inline Mode inside terminal multiplexer `[Planned]`
**Given** the writer is inside tmux or screen
**When** the writer runs `zani document.md`
**Then** Zani runs inline in the current pane
**And** no Writing Window is spawned regardless of flags

### Scenario: Unknown terminal falls back to Inline Mode
**Given** the writer is in a terminal Zani cannot identify
**When** the writer runs `zani --window document.md`
**Then** Zani runs inline in the current terminal
**And** no error is shown

---

## Feature: Autosave

### Scenario: Document is saved automatically
**Given** the writer has made changes to the Document
**When** the writer pauses typing (no keystrokes for a configured interval)
**Then** the Buffer is written to disk at the Document's file path
**And** no save confirmation or dialog is shown

### Scenario: Autosave does not disrupt writing
**Given** the writer is actively typing
**When** an Autosave triggers
**Then** there is no visible indication that a save occurred (unless Chrome is summoned)
**And** keystroke latency is unaffected

---

## Feature: Smart Typography

### Scenario: Straight double quotes convert to curly quotes
**Given** the writer types a straight double quote `"`
**When** the character is inserted into the Buffer
**Then** it is replaced with the appropriate curly quote (`"` or `"`) based on context (opening or closing)

### Scenario: Double hyphen converts to em dash
**Given** the writer types `--`
**When** the second hyphen is inserted
**Then** the two hyphens are replaced with an em dash `—` in the Buffer

### Scenario: Triple period converts to ellipsis
**Given** the writer types `...`
**When** the third period is inserted
**Then** the three periods are replaced with an ellipsis character `…` in the Buffer

---

## Feature: Vim Bindings

### Scenario: Insert mode uses bar cursor
**Given** Zani is in insert mode
**When** the terminal renders the cursor
**Then** the cursor shape is a vertical bar

### Scenario: Normal mode uses block cursor
**Given** Zani is in normal mode
**When** the terminal renders the cursor
**Then** the cursor shape is a block

### Scenario: Mode switch between normal and insert
**Given** Zani is in normal mode
**When** the writer presses `i`
**Then** Zani enters insert mode
**And** the cursor changes to a bar
**And** keystrokes insert text into the Buffer

---

## Feature: Color Profile Detection

### Scenario: True Color terminal gets full color rendering
**Given** the terminal supports True Color (`COLORTERM=truecolor` or `COLORTERM=24bit`)
**When** Zani starts
**Then** all Dimming uses 24-bit RGB interpolation
**And** the full Palette is rendered as specified

### Scenario: 256-color terminal gets approximated rendering
**Given** the terminal supports only 256 colors
**When** Zani starts
**Then** Dimming approximates the gradient using the nearest 256-color values
**And** the Palette colors are mapped to the nearest available colors

### Scenario: Basic terminal gets minimal dimming
**Given** the terminal supports only basic ANSI colors
**When** Zani starts
**Then** Focus Mode Dimming uses the ANSI `dim` attribute instead of color interpolation
**And** the Palette maps to the 16 basic ANSI colors

---

## Feature: External Integrations (Pull-Only)

### Scenario: Plexus ingest is triggered on demand `[Planned]`
**Given** the writer has text in the Document
**When** the writer invokes the Ingest action via hotkey
**Then** the Document content (or selection) is sent to Plexus via its ingest pipeline
**And** no ingest occurs without explicit invocation

### Scenario: llm-orc ensemble is invoked on demand `[Planned]`
**Given** the writer has text in the Document
**When** the writer invokes an ensemble via hotkey
**Then** the selected ensemble runs against the Document content (or selection)
**And** results are presented without modifying the Buffer
**And** no ensemble runs without explicit invocation

---

## Integration Scenarios

### Scenario: Writing Surface applies both Focus Dimming and Markdown Styling in one render pass
**Given** Focus Mode is set to Paragraph and the Document contains markdown formatting
**When** the Writing Surface renders
**Then** text in the Active Region has markdown formatting (bold/italic) at full foreground color and dimmed syntax characters
**And** text outside the Active Region has markdown formatting at the Focus-dimmed foreground color and further-dimmed syntax characters
**And** both dimming layers compose correctly (syntax dimming + focus dimming do not produce colors outside the Palette's range)

### Scenario: Palette switch updates both Focus Dimming and Markdown Styling
**Given** Focus Mode is active and Markdown Styling is rendering
**When** the writer switches to a different Palette
**Then** Focus Dimming interpolation endpoints update to the new Palette's colors
**And** Markdown Styling accent colors update to the new Palette's accent colors
**And** the transition is immediate (single re-render)

### Scenario: Autosave writes the Buffer, not the styled output
**Given** the Document contains markdown with Smart Typography conversions applied
**When** Autosave triggers
**Then** the file on disk contains the Buffer's content (with smart typography characters)
**And** no Markdown Styling or Dimming information is written to disk

---

## Feature: Palette Affective Categories (ADR-009)

### Scenario: Every palette belongs to exactly one Affective Category
**Given** the complete Palette collection
**When** each Palette is inspected
**Then** every Palette has an Affective Category assignment
**And** no Palette belongs to more than one category

### Scenario: Palettes within a category are sorted by Perceptual Sort Order
**Given** an Affective Category containing multiple Palettes
**When** the Palettes are listed in their category order
**Then** each adjacent pair has OKLCH hue angles that are closer to each other than to non-adjacent pairs
**And** scrolling through the list produces a smooth perceptual gradient

### Scenario: Affective Category taxonomy covers brightness and character axes
**Given** the set of all Affective Categories
**When** the categories are enumerated
**Then** both Dark and Light brightness levels are represented
**And** Warm, Cool, Vivid, and Muted character types are represented within each brightness level
**And** the total number of categories is exactly eight

---

## Feature: Palette Browser (ADR-010)

### Scenario: Settings Layer shows a single Palette entry instead of inline rows
**Given** the Settings Layer is visible
**When** the writer views the Palette section
**Then** a single row reads "Palette: [current palette name]"
**And** no inline palette choices are listed

### Scenario: Selecting the Palette row opens the Palette Browser
**Given** the Settings Layer is visible and the cursor is on the Palette row
**When** the writer presses Enter
**Then** the Palette Browser sub-panel opens
**And** Palettes are displayed grouped by Affective Category

### Scenario: Palette Browser groups palettes by Affective Category
**Given** the Palette Browser is open
**When** the writer views the browser contents
**Then** Palettes appear under their Affective Category headings
**And** within each category, Palettes are ordered by Perceptual Sort Order

### Scenario: Applying a palette from the browser triggers crossfade
**Given** the Palette Browser is open and the writer is on a Palette different from the current one
**When** the writer selects that Palette
**Then** the Writing Surface transitions from the old Palette to the new Palette via the 300ms crossfade animation
**And** the Palette Browser reflects the newly active Palette

### Scenario: Esc from the Palette Browser returns to the Settings Layer
**Given** the Palette Browser is open
**When** the writer presses Esc
**Then** the Palette Browser closes
**And** the Settings Layer is visible with the Palette row showing the current palette name

### Scenario: Palette Browser indicates Local Config provenance
**Given** a Local Config binds "Neon Noir" to the current project
**And** the global config specifies "Ember"
**When** the Palette Browser is open
**Then** the active Palette displays as "Neon Noir (project)"
**And** a palette set only in global config would display as "[name] (global)"

---

## Feature: Local Config (ADR-011)

### Scenario: Local Config overrides global config palette
**Given** a `.zani.toml` exists in the project directory with `palette = "Inkwell"`
**And** the global config specifies `palette = "Ember"`
**When** the writer opens a file in that project directory
**Then** Config Resolution resolves to "Inkwell"
**And** the Writing Surface renders with the Inkwell Palette

### Scenario: Walk-up search finds nearest Local Config
**Given** a `.zani.toml` exists at `/projects/novel/` with `palette = "Parchment"`
**And** no `.zani.toml` exists at `/projects/novel/chapters/`
**When** the writer opens `/projects/novel/chapters/chapter1.md`
**Then** Config Resolution walks up from `chapters/` to `novel/` and finds the `.zani.toml`
**And** the resolved Palette is "Parchment"

### Scenario: Missing Local Config fields fall through to global config
**Given** a `.zani.toml` exists with only `palette = "Inkwell"` (no other fields)
**And** the global config specifies `column_width = 72` and `focus_mode = "paragraph"`
**When** Config Resolution runs
**Then** the resolved palette is "Inkwell" (from local)
**And** the resolved column_width is 72 (from global)
**And** the resolved focus_mode is Paragraph (from global)

### Scenario: No Local Config falls through to global config
**Given** no `.zani.toml` exists in any ancestor directory of the opened file
**And** the global config specifies `palette = "Ember"`
**When** Config Resolution runs
**Then** the resolved Palette is "Ember" (from global config)

### Scenario: Opening a file in a different project triggers palette crossfade
**Given** the writer has "Ember" active (from the current project's Local Config)
**When** the writer opens a file in a different project whose Local Config specifies "Neon Noir"
**Then** the Writing Surface crossfades from Ember to Neon Noir via the 300ms animation

### Scenario: Palette Browser offers Bind to project `[Superseded by ADR-013]`
**Given** the Palette Browser is open and the writer selects a new Palette
**When** the writer chooses "Bind to project"
**Then** a `.zani.toml` file is written in the nearest project directory (or the file's directory)
**And** the file contains `palette = "[selected palette name]"`

> Superseded: Bind-to-project moved from the Palette Browser to a dedicated Config row in the Settings Layer (ADR-013).

---

## Feature: Config Persistence Provenance (ADR-013)

### Scenario: Local config persists immediately on settings change
**Given** the session loaded config from a `.zani.toml` (config_source is Local)
**When** the writer changes any setting (palette, focus mode, column width, editing mode, or scroll mode)
**Then** the change is written immediately to the same `.zani.toml` that was loaded
**And** the global config file is not modified
**And** a second file opened in the same project directory will use the updated settings

### Scenario: Global config changes are held in memory until quit
**Given** the session loaded config from `~/.config/zani/config.toml` (config_source is Global)
**When** the writer changes any setting during the session
**Then** the change is held in memory only — the global config file is not modified yet
**And** on quit, the current settings are written to the global config file

### Scenario: Default config changes are held in memory until quit
**Given** no config files exist (config_source is Default)
**When** the writer changes any setting during the session
**Then** the change is held in memory only
**And** on quit, the current settings are written to `~/.config/zani/config.toml` (creating it if needed)

### Scenario: Config row shows current config scope
**Given** the Settings Layer is visible
**When** the writer views the Config row
**Then** the row displays "Config: project" when config_source is Local
**Or** the row displays "Config: global [enter]" when config_source is Global or Default

### Scenario: Config row Save to project creates Local Config
**Given** the Settings Layer is visible and config_source is Global
**And** the writer has a file open (not a scratch buffer)
**When** the writer selects the Config row and presses Enter
**Then** a `.zani.toml` is written in the file's parent directory (or the existing Local Config location if one was found by walk-up)
**And** the `.zani.toml` contains all current settings (palette, focus mode, column width, editing mode, scroll mode)
**And** config_source switches to Local
**And** subsequent settings changes persist immediately to the `.zani.toml`

### Scenario: Config row is informational when already Local
**Given** the Settings Layer is visible and config_source is Local
**When** the writer views the Config row
**Then** the row displays "Config: project" without an `[enter]` affordance
**And** pressing Enter on the Config row has no effect

### Scenario: Scratch buffer cannot Save to project
**Given** the writer is editing a scratch buffer (no file path)
**And** the Settings Layer is visible
**When** the writer views the Config row
**Then** the row displays "Config: global" without an `[enter]` affordance
**And** pressing Enter on the Config row has no effect

---

## Feature: Hybrid 256-Color Degradation (ADR-012)

### Scenario: Hand-tuned 256-color values are used when available
**Given** a Palette with hand-tuned 256-color alternate values
**And** the detected Color Profile is 256-color
**When** Degrade runs for this Palette
**Then** the hand-tuned values are used instead of automatic nearest-color mapping

### Scenario: Automatic mapping is used when no hand-tuned values exist
**Given** a Palette without hand-tuned 256-color alternate values
**And** the detected Color Profile is 256-color
**When** Degrade runs for this Palette
**Then** the automatic `nearest_256_color` mapping applies to each color

### Scenario: Hand-tuned 256-color values satisfy Invariant 3
**Given** a Palette with hand-tuned 256-color alternate values
**When** the hand-tuned foreground and background colors are measured
**Then** every foreground/background pair has a contrast ratio of at least 4.5:1
**And** no color is pure black `rgb(0, 0, 0)` or pure white `rgb(255, 255, 255)`

### Scenario: Basic ANSI focuses on readability
**Given** the detected Color Profile is basic ANSI (16 colors)
**When** Degrade runs for any Palette
**Then** colors map to the 16 ANSI colors prioritizing contrast and readability
**And** no mood-specific accent tuning is applied

---

## Integration Scenarios (ADR-009 through ADR-013)

### Scenario: Config Resolution feeds the correct Palette to the Palette Browser
**Given** a Local Config binds "Inkwell" to the current project
**When** the writer opens the Palette Browser
**Then** "Inkwell" is marked as the active Palette
**And** the Affective Category containing Inkwell is expanded or highlighted

### Scenario: Palette selected in browser persists via Local Config Bind `[Superseded by ADR-013]`
**Given** the Palette Browser is open in a project with an existing Local Config
**When** the writer selects "Neon Noir" and chooses "Bind to project"
**Then** the `.zani.toml` is updated with `palette = "Neon Noir"`
**And** subsequent file opens in this project resolve to "Neon Noir" via Config Resolution

> Superseded: Palette selection in the browser saves via write-back-to-source (ADR-013). The browser selects; the config scope determines where changes persist.

### Scenario: Palette change in browser persists to Local Config via write-back
**Given** the session loaded config from a `.zani.toml` (config_source is Local)
**And** the Palette Browser is open
**When** the writer selects "Neon Noir"
**Then** `save_config()` writes `palette = "Neon Noir"` to the same `.zani.toml`
**And** subsequent file opens in this project resolve to "Neon Noir" via Config Resolution

### Scenario: 256-color degradation applies to the resolved Palette
**Given** the detected Color Profile is 256-color
**And** Config Resolution resolves to a Palette with hand-tuned 256-color values
**When** the Writing Surface renders
**Then** the hand-tuned 256-color values are used for all Palette colors
**And** Focus Mode Dimming interpolates between the 256-color values

### Scenario: Palette validation runs on both True Color and 256-color values
**Given** a Palette with both True Color values and hand-tuned 256-color alternates
**When** `validate()` is called
**Then** both the True Color color pairs and the 256-color color pairs pass Invariant 3
**And** validation failure in either set produces a PaletteError

### Scenario: Save to project then reload round-trips all settings
**Given** a writer opens a file with global config (no `.zani.toml` exists)
**And** the writer changes palette to "Neon Noir", focus mode to Paragraph, and column width to 72
**When** the writer selects the Config row to "Save to project"
**And** Config Resolution runs again for a file in the same directory
**Then** the resolved config has palette "Neon Noir", focus mode Paragraph, and column width 72
**And** config_source is Local

---

## Feature: Muted Affective Category (ADR-014)

### Scenario: DarkMuted and LightMuted categories exist in the taxonomy
**Given** the `AffectiveCategory` enum
**When** all variants are enumerated
**Then** `DarkMuted` and `LightMuted` are present
**And** `AffectiveCategory::all()` returns exactly eight categories

### Scenario: DarkMuted is classified as dark
**Given** the `DarkMuted` Affective Category
**When** `is_dark()` is called
**Then** the result is true

### Scenario: Muted categories appear after Vivid in display order
**Given** the display order returned by `AffectiveCategory::all()`
**When** the order is inspected
**Then** `DarkMuted` appears after `DarkVivid`
**And** `LightMuted` appears after `LightVivid`

### Scenario: Muted categories display correct labels
**Given** the `DarkMuted` and `LightMuted` Affective Categories
**When** `label()` is called on each
**Then** `DarkMuted` returns "Dark — Muted"
**And** `LightMuted` returns "Light — Muted"

### Scenario: Palette Browser renders Muted category sections
**Given** the Palette Browser is open
**When** the writer views the browser contents
**Then** "Dark — Muted" and "Light — Muted" appear as category headings
**And** each heading has Palettes listed beneath it

---

## Feature: PNW Flora Naming Register (ADR-015)

### Scenario: Default palette is Manzanita
**Given** Zani starts with no config files
**When** the default Palette is loaded
**Then** the Palette name is "Manzanita"
**And** the Palette belongs to the DarkWarm Affective Category

### Scenario: Every palette is named after a Cascadia bioregion species
**Given** the complete Palette collection
**When** each Palette name is inspected
**Then** no Palette is named "Ember", "Hearthstone", "Inkwell", "Moonstone", "Neon Noir", "Aurora", "Parchment", "Manuscript", "Glacier", "Daybreak", "Balsamroot", "Reindeer Lichen", or "Columbine"
**And** every Palette name corresponds to a Cascadia bioregion species

### Scenario: Collection contains exactly 40 palettes
**Given** the complete Palette collection returned by `Palette::all()`
**When** the count is taken
**Then** the total is 40

### Scenario: Each Affective Category contains exactly 5 palettes
**Given** the Palette collection grouped by Affective Category
**When** each category group is counted
**Then** every Affective Category contains exactly 5 Palettes

### Scenario: Every palette has a Provenance Description
**Given** the complete Palette collection
**When** each Palette is inspected
**Then** every Palette has a non-empty Provenance Description
**And** every description includes a scientific name in italicized format

### Scenario: Sibling palettes have diverse hue angles
**Given** an Affective Category containing 5 Palettes
**When** the OKLCH hue angles of their backgrounds are compared
**Then** no two siblings have hue angles within 15 degrees of each other
**And** the palettes span a broad arc of the category's available hue space

### Scenario: Palette name passes the curation test
**Given** a Palette with name N and Affective Category C
**When** evaluated against the five-point curation test
**Then** N is traceable to a specific Cascadia bioregion species
**And** N's color associations are congruent with C's affect without restating the category label
**And** N is register-consistent with other names in the collection
**And** N activates a distinct sensory image from other names in category C
**And** seeing the Palette's colors alongside N, the connection is resolvable

### Scenario: Provenance Description is botanically accurate
**Given** a Palette with Provenance Description D
**When** D is evaluated
**Then** D references the species' actual habitat range (not a romanticized approximation)
**And** D includes the scientific name and a one-line ecological context

---

## Integration Scenarios (ADR-014 + ADR-015)

### Scenario: Manzanita default round-trips through config persistence
**Given** Zani starts with the default Palette (Manzanita)
**And** no config files exist (config_source is Default)
**When** the writer quits without changing any settings
**And** Zani restarts
**Then** the resolved Palette is still Manzanita

### Scenario: Palette Browser shows 8 categories with 5 palettes each
**Given** the Palette Browser is open
**When** the writer scans all category sections
**Then** eight Affective Category headings are visible
**And** each heading contains exactly 5 Palette entries

### Scenario: Config file references palette by Cascadia species name
**Given** a `.zani.toml` exists with `palette = "Salal"`
**When** Config Resolution runs
**Then** the resolved Palette is Salal
**And** the Palette belongs to the DarkVivid Affective Category

---

## Feature: Species-Color Validation Remediation (ADR-016)

### Scenario: Replaced species have correct names in collection
**Given** the complete Palette collection
**When** each Palette name in Light Warm is inspected
**Then** "Bracken" is present (replacing Oregon Sunshine's former slot)
**And** "Licorice Fern" is present (replacing Balsamroot)
**And** "Balsamroot" is not present
**And** "Oregon Sunshine" is present in Light Vivid (not Light Warm)

### Scenario: Silver Fir replaces Reindeer Lichen in Light Muted
**Given** the Palette collection filtered to LightMuted
**When** the Palette names are inspected
**Then** "Silver Fir" is present
**And** "Reindeer Lichen" is not present

### Scenario: Oregon Sunshine belongs to Light Vivid with redesigned colors
**Given** the Palette named "Oregon Sunshine"
**When** its Affective Category is inspected
**Then** the category is LightVivid
**And** at least one accent color has an OKLCH hue angle in the yellow range (70°–100°)

### Scenario: Columbine is not in the collection
**Given** the complete Palette collection
**When** `Palette::by_name("Columbine")` is called
**Then** the result is None

### Scenario: Light Vivid hue distribution is clean after reorganization
**Given** the 5 Palettes in Light Vivid after reorganization
**When** the OKLCH hue angles of their accent_heading colors are compared
**Then** no two palettes have hue angles within 15 degrees of each other
**And** the five palettes span red, orange, yellow, magenta-pink, and blue-violet

### Scenario: Jack-o'-Lantern provenance uses correct species
**Given** the Palette named "Jack-o'-Lantern"
**When** its Provenance Description is inspected
**Then** the scientific name contains "olivascens" (not "olearius")

### Scenario: Replaced palettes have botanically grounded provenance
**Given** the Palettes named "Bracken", "Licorice Fern", and "Silver Fir"
**When** each Provenance Description is inspected
**Then** each includes the correct scientific name
**And** each references the species' actual Cascadia bioregion habitat
**And** no description rationalizes the palette colors — it describes the species

### Scenario: Corrected provenance texts match external sources
**Given** the Palettes named "Oakmoss", "Witch's Hair", "Partridgefoot", and "Lupine"
**When** their Provenance Descriptions are inspected
**Then** Oakmoss does not describe its thallus as "teal"
**And** Witch's Hair does not describe its thallus as "olive-black"
**And** Partridgefoot does not describe its foliage as "gray-green"
**And** Lupine does not claim "silvery sheen"

### Scenario: Collection remains at 40 palettes after remediation
**Given** the complete Palette collection after all ADR-016 changes
**When** the count is taken
**Then** the total is 40
**And** each of the 8 Affective Categories contains exactly 5 Palettes

---

## Feature: Flora Reference (ADR-017)

### Scenario: Flora Reference documents all collection species
**Given** the Flora Reference at `docs/flora-reference.md`
**When** the species entries are counted
**Then** every species in the Palette collection has a corresponding entry
**And** no entry exists for retired species (Columbine, Balsamroot, Reindeer Lichen)

### Scenario: Flora Reference entries contain required fields
**Given** a Flora Reference entry for any species
**When** the entry is inspected
**Then** it includes the common name and scientific name
**And** it includes Signature Colors with at least one source citation
**And** it includes an Essence Mode (Feature, Throughline, or Place)
**And** it includes the geographic range within the Cascadia bioregion
**And** it includes the palette assignment (Affective Category and slot rationale)

### Scenario: Provenance Description is consistent with Flora Reference
**Given** any Palette in the collection
**When** its Provenance Description is compared to the corresponding Flora Reference entry
**Then** the scientific name matches
**And** the habitat description is consistent
**And** the color language does not contradict the documented Signature Colors

---

## Integration Scenarios (ADR-016 + ADR-017)

### Scenario: New species pass the existing validation infrastructure
**Given** the Palettes named "Bracken", "Licorice Fern", "Silver Fir", and "Oregon Sunshine"
**When** `Palette::validate()` is called on each
**Then** all pass Invariant 3 (no pure black/white, WCAG AA 4.5:1 minimum)
**And** no PaletteError is returned

### Scenario: Sibling hue diversity holds after Light Warm and Light Vivid changes
**Given** the 5 Palettes in Light Warm after replacement (Oatgrass, Bracken, White Oak, Ponderosa, Licorice Fern)
**And** the 5 Palettes in Light Vivid after reorganization (Paintbrush, Tiger Lily, Oregon Sunshine, Farewell, Camas)
**When** the OKLCH hue diversity test runs for each category
**Then** no two siblings in either category have background hue angles within 15 degrees of each other
