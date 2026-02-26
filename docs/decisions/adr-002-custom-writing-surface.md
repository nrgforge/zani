# ADR-002: Custom Writing Surface on Ratatui's Cell Buffer

**Status:** Proposed

## Context

Ratatui's Paragraph widget has known issues with soft-wrapped text and scroll positioning (open issues #293, #2342). Wrapped lines remain a single Line object internally, causing scroll position inaccuracies. The reflow module is described as "fairly convoluted and difficult to understand."

For Zani, soft-wrapping and scrolling are the most critical rendering path — a prose writing app lives and dies by how it handles wrapped text. Invariant 5 requires text to wrap at approximately 60 characters, centered in the terminal. Invariant 6 requires sub-1ms app-layer latency per keystroke. The Writing Surface must also support per-character Dimming (Invariant 4) and Markdown Styling (Invariant 10).

Three options were considered:
1. Use ratatui's Paragraph widget and work around its limitations.
2. Contribute fixes upstream to ratatui.
3. Use ratatui for the App Shell but build a custom Writing Surface that renders directly to ratatui's cell buffer.

## Decision

Option 3. The App Shell uses ratatui for layout, input routing, and the event loop. The Writing Surface is a custom component that:
- Reads text from the Buffer (Ropey).
- Performs its own soft-wrapping to the prose-width column.
- Calculates scroll position accounting for wrapped lines.
- Applies Markdown Styling (bold, italic, color, dimmed syntax characters).
- Applies Focus Mode Dimming (per-character color interpolation).
- Renders directly to ratatui's cell buffer.

## Relationship to ADR-001

This decision does not bypass ratatui — it bypasses one widget (Paragraph) while continuing to use ratatui's core rendering architecture. The render path is:

```
Ropey (Buffer) → Writing Surface (wrapping/styling) → ratatui cell buffer → crossterm → terminal
```

Ratatui's performance characteristics — double-buffering, diff-based terminal updates, sub-millisecond frame times — live in the cell buffer and crossterm layers, which the Writing Surface uses fully. Skipping Paragraph removes a layer of abstraction (Text/Line/Span conversion and Paragraph's reflow logic), not adds one. This is a maintenance cost, not a performance cost.

## Consequences

**Positive:**
- Full control over wrapping, scrolling, and per-character styling — the three capabilities most critical to the writing experience.
- No dependency on ratatui's text wrapping bugs being fixed upstream.
- Wrapping, styling, and dimming can be integrated in a single render pass.

**Negative:**
- More code to write and maintain than using a built-in widget.
- Must handle edge cases (Unicode grapheme clusters, wide characters, line break opportunities) that Paragraph already handles.

**Neutral:**
- Ratatui is still used for everything outside the Writing Surface (layout, chrome, settings layer).
- Ropey handles the text storage and manipulation; the Writing Surface handles presentation.
