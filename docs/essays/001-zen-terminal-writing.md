# Zani: A Terminal Writing App

*2026-02-26*

## The Problem

There is no purpose-built terminal writing app with thoughtful visual design.

WordGrinder — the only dedicated terminal word processor, built by an author who needed a tool to write a novel — is focused and fast, but uses its own file format and hasn't prioritized typography or aesthetics. Neovim with zen-mode.nvim provides focusing and dimming, but you're still in a development editor. The features exist, but the cohesive writing experience doesn't. Emacs writeroom-mode is similar. In the GUI world, iA Writer has executed well on distraction-free writing, but it's a closed graphical app — no composability, no scripting, no integration with external tools.

The gap: no terminal app exists that is purpose-built for creative writing, uses markdown natively, has considered typography and visual design within terminal constraints, and provides vim-style editing without being vim.

## Design Principles from Research

The word "zen" gets used loosely. Here's what the research actually says about the conditions that support sustained creative writing.

**Flow states.** Csikszentmihalyi's research found that writing is one of the activities most naturally conducive to flow — it has rules, requires skill, and provides immediate feedback. Flow breaks when the writer notices the tool: a lag, a visual distraction, unexpected behavior. The practical implication is that a writing tool should be as invisible as possible during use.

**Cognitive load.** Every visible UI element imposes processing cost. The brain evaluates and dismisses each one, consuming resources that could go to the writing itself. Whitespace reduces this load. Oliver Reichenstein (iA Writer's creator): *"In iA Writer, you have no choice but to think and write."*

**Typography.** Research consensus puts optimal line length at 55-66 characters. Increasing line spacing to 120% improves reading accuracy by 20% and reduces eye strain by 30%. Monospace fonts are slower to read than proportional fonts, but iA Writer argues this is appropriate for drafting — writing should be deliberate, and monospace communicates "work in progress."

**Color and contrast.** Pure black (#000) on pure white (#FFF) causes halation — a glow effect that strains eyes, worse for the ~33% of people with astigmatism. Research supports soft dark backgrounds with warm off-white text, maintaining at least a 4.5:1 contrast ratio (WCAG).

**Focus modes.** Sentence and paragraph dimming — where surrounding text fades while the active text stays bright — is implemented in iA Writer, Calmly Writer, Focused, and others. The mechanism: dimming reduces the temptation to re-read and revise, keeping the writer in generative mode. Typewriter mode (cursor stays centered, text scrolls around it) largely eliminates scrolling UX issues.

These findings translate into concrete design decisions:

1. Zero visible chrome by default. Text, cursor, empty space.
2. Centered ~60-character column with 120%+ line spacing.
3. Curated color palettes — no pure black or white, WCAG AA contrast ratio. The default is warm and muted.
4. Sentence, paragraph, and typewriter focus modes using per-character color interpolation.
5. Monospace rendering. The terminal's grid is the right default for drafting.
6. Minimal app-layer latency. The tool must respond faster than conscious perception.
7. Settings hidden until summoned. The default state is writing.

## Why Rust

The language choice follows from the latency requirement.

Dan Luu's research on terminal latency shows people perceive delays down to 2ms. Modern terminal stacks already introduce 100-200ms from keypress to screen update. The app layer needs to add as little as possible.

Rust's zero-GC runtime means predictable, consistent latency on every keypress. In benchmarks, Rust TUI apps (ratatui) use 30-40% less memory and 15% less CPU than Go equivalents (Bubbletea). The important metric isn't peak throughput — it's the absence of GC-induced pauses during sustained typing.

The Rust TUI ecosystem is mature:

- **Ratatui** — immediate-mode rendering, sub-millisecond frame times, intelligent double-buffering. The dominant Rust TUI framework.
- **Crossterm** — cross-platform terminal backend (Windows, macOS, Linux). The recommended default.
- **Ropey** — rope data structure for text buffers. Handles 100MB+ documents, 1.8M insertions/sec, cheap cloning via shared data.
- **tui-textarea** — existing text editor widget with basic vim emulation. Useful as reference, though Zani will need its own implementation.

Go's Charm ecosystem (Bubbletea, Lipgloss) has better out-of-the-box aesthetics. Lipgloss provides CSS-like declarative styling that ratatui lacks. But this is a library gap, not a language gap — Zani can adopt Charm's design patterns (declarative styles, adaptive color profiles, graceful degradation) in Rust. Zig is worth watching — Ghostty and libvaxis demonstrate strong terminal capabilities — but the text editing ecosystem isn't there yet.

**Technical risk:** Ratatui's Paragraph widget has known issues with soft-wrapped text and scroll positioning (open issues #293, #2342). For a prose app, this is the most critical rendering path. The plan: use ratatui for the App Shell and layout, but build a custom Writing Surface rendering directly to ratatui's cell buffer with Ropey managing the text. This gives full control over wrapping and scrolling behavior.

## Terminal-Native Affordances

A terminal writing app has structural advantages over a GUI app like iA Writer.

**Unix composability.** Terminal apps participate in the Unix tool ecosystem: pipes, scripts, shell aliases, git hooks. `cat notes.md | zani` works. Scripted export workflows work. A GUI app is self-contained; a terminal app composes with everything else.

**Integration with existing tools.** Zani can integrate with Plexus (a knowledge graph engine) via its ingest pipeline — on save or on demand, writing feeds into the knowledge graph and semantics are extracted automatically. It can invoke llm-orc ensembles (coordinated specialized models) for on-demand analysis: style checks, fact checks, structural critique. These are pull interactions, triggered by the user, not running in the background. They don't interrupt writing.

**Session persistence.** Through tmux or zellij, writing sessions survive SSH disconnections, machine switches, and network interruptions. Write on any machine with a terminal.

**Git integration.** Markdown files on disk are naturally git-friendly — branching, diffing, and collaboration use standard git workflows. The scope of Zani's git awareness (what it does automatically vs. what the writer controls) requires further research.

**Distribution.** Rust compiles to static binaries. No app store, no subscription, no runtime. Every computer has a terminal.

## Typography in the Terminal

Terminal typography is more constrained than GUI typography, but the constraints are narrower than they appear.

**What the app controls:** Per-character foreground and background color (true color, 16.7M colors), text attributes (bold, italic, dim, underline, strikethrough), cursor shape, layout and centering, and character substitution (smart quotes, em dashes, ellipses).

Focus dimming is implemented as color interpolation: active text at full brightness (e.g., `rgb(220, 215, 205)`), surrounding text faded toward the background color (e.g., `rgb(100, 97, 92)` over a `rgb(40, 38, 35)` background). This works on any terminal with true color support, which includes virtually all modern terminals except macOS Terminal.app (which gets a 256-color approximation).

**What the app cannot control:** The font. Terminal emulators set the font, not applications. No modern terminal supports changing font family via escape codes at runtime.

**How Zani handles it:** iA Writer's fonts (Mono, Duo, Quattro) are free and MIT-licensed, available as Nerd Font patches. iA Writer Duo is their duospace design — 150% width on m, M, w, W — which preserves monospace rhythm while improving text flow.

To avoid requiring users to reconfigure their terminal before writing, Zani launches a dedicated terminal window with writing-optimized settings. Every major terminal supports CLI config overrides:

```
ghostty --font-family="iA Writer Duo" --font-size=16 -e zani draft.md
kitty -o font_family="iA Writer Duo" -o font_size=16 zani draft.md
alacritty -o font.normal.family="iA Writer Duo" -e zani draft.md
```

Zani detects the current terminal emulator, spawns a new window with the configured font, line height, and color settings, and runs inside it. The user's development terminal is unchanged. `zani --inline` skips window creation for SSH, tmux, or pre-configured terminals.

Line height is also terminal-configured (Kitty: `modify_font cell_height 120%`, WezTerm: `line_height = 1.2`, Ghostty: `adjust-cell-height = 20`). The dedicated window approach handles this too.

On Kitty specifically, the text sizing protocol allows rendering text at different sizes — headings can be genuinely larger. Other terminals fall back to bold + color differentiation for headings.

## Summary

Zani is a terminal writing app built in Rust. It provides a distraction-free markdown writing environment with focus modes, curated color palettes, vim keybindings, and autosave. It launches in a dedicated terminal window with writing-optimized typography. It integrates with external tools (knowledge graphs, LLM orchestration) via on-demand, user-initiated interactions. It ships as a single static binary.

The core technical decisions: Rust for predictable latency, ratatui + crossterm for the application shell, ropey for the text buffer, a custom Writing Surface rendering directly to ratatui's cell buffer, and the dedicated-window approach for font control. The core design decisions come from research on flow states, cognitive load, typography, and color science — not aesthetic preference.
