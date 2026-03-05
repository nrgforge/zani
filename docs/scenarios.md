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
