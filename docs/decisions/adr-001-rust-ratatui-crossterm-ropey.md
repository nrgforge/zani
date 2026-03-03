# ADR-001: Rust with Ratatui, Crossterm, and Ropey

**Status:** Accepted

## Context

Invariant 6 states: "Latency is a UX requirement, not a performance metric. Every keystroke must produce a visible result within the app layer's control in under 1ms."

Dan Luu's research shows people perceive latency down to 2ms. Modern terminal stacks already introduce 100-200ms from keypress to screen update. The app layer must add as close to zero overhead as possible. For a writing app where every keystroke matters, GC-induced micro-pauses during sustained typing are a UX problem, not just a performance one.

Four languages were evaluated: Rust, Go, Zig, and C.

- **Go (Bubbletea + Lipgloss):** Best out-of-box aesthetics via the Charm ecosystem. But GC pauses exist during sustained typing. 30-40% more memory than Rust equivalent.
- **Zig (libvaxis):** Promising (Ghostty proves Zig's terminal capability), but the text editing ecosystem is immature. Language is pre-1.0.
- **C (notcurses):** Most powerful rendering library. Rust bindings (notcurses-rs) are immature.
- **Rust (ratatui + crossterm + ropey):** Zero-GC runtime, sub-millisecond rendering, mature ecosystem for all three core concerns (app shell, terminal backend, text buffer).

## Decision

Use Rust with:
- **Ratatui** for the App Shell — immediate-mode rendering, intelligent double-buffering, 60+ FPS.
- **Crossterm** as the terminal backend — cross-platform (Windows, macOS, Linux), most popular, best documented.
- **Ropey** for the Buffer — rope data structure handling 100MB+ documents, 1.8M insertions/sec, cheap cloning via shared data.

## Consequences

**Positive:**
- Zero-GC guarantees predictable, consistent latency on every keystroke.
- Ratatui's immediate-mode rendering gives full control over what gets drawn — essential for the custom Writing Surface.
- Ropey is purpose-built for text editors. No comparably mature rope library exists in Go.
- Static binary compilation — no runtime dependencies, simple distribution.

**Negative:**
- No Lipgloss equivalent. The aesthetic/theming layer must be built by Zani.
- Steeper learning curve than Go.
- Ratatui's Paragraph widget has known soft-wrap/scroll issues (addressed in ADR-002).

**Neutral:**
- Crossterm is the recommended default backend. Termion and Termwiz are alternatives if platform-specific needs arise.
