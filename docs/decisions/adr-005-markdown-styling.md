# ADR-005: Styled Markdown with Dimmed Syntax Characters

**Status:** Accepted

## Context

Zani's native format is markdown (Invariant 7). The Writing Surface must render markdown in a way that is visually pleasant for sustained writing while keeping the source fully editable.

Three approaches were considered:
1. **Raw markdown** — show syntax characters with no visual treatment. Functional but visually noisy.
2. **Styled markdown** — show syntax characters but apply formatting. Characters like `**`, `*`, `#` are visible but dimmed; the text they modify receives formatting (bold, italic, color).
3. **Concealed markdown** — hide syntax characters, show only rendered output. Breaks editability — the writer can't see where formatting boundaries are, and cursor positioning becomes ambiguous.

Invariant 10 states: "Markdown is always editable. Markdown Styling is a render-time visual layer. The Buffer contains the raw markdown exactly as typed. Syntax characters are never hidden or removed — they are dimmed."

## Decision

Option 2. The Writing Surface applies Markdown Styling at render time:
- Syntax characters (`#`, `**`, `*`, `` ` ``, `>`, `-`, etc.) are rendered with Dimming — their foreground color interpolated toward the background, making them visually recede.
- Text modified by syntax receives formatting: `**bold**` renders the word in bold at full brightness, `*italic*` renders in italic, headings receive bold + a distinct color.
- Code blocks (fenced with `` ``` ``) are styled as source text with no further rendering (mermaid, C4, etc. are shown as source).
- The Buffer is never modified by Markdown Styling. What the writer typed is what the Buffer contains.

## Consequences

**Positive:**
- The writer sees the markdown source and can edit it directly — cursor placement is never ambiguous.
- Visual hierarchy (headings, emphasis, code) is clear at a glance without needing a preview pane.
- Reuses the Dimming mechanism from Focus Mode (ADR-004) for syntax character styling.
- No distinction between "edit mode" and "preview mode" — there is only writing.

**Negative:**
- Syntax characters are still visible, which is slightly noisier than a concealed approach.
- Requires a markdown parser that can identify syntax boundaries for per-character styling. This parser must be fast enough to run on every render frame within the latency budget (Invariant 6).

**Neutral:**
- Headings on Kitty terminals can use the text sizing protocol for genuinely larger text. Other terminals fall back to bold + color differentiation. This is graceful degradation per Invariant 11, not a separate decision.
