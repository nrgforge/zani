# Research Log: Zani — Terminal Writing App

## Question 1: What terminal-based writing apps exist today, what do they do well, and what gaps remain for a "zen writing experience"?

**Method:** Web search, HN comment analysis, design philosophy research

### Findings

#### The Landscape

**Dedicated terminal writing apps are nearly nonexistent.** The field is dominated by:

1. **WordGrinder** — The only real dedicated terminal word processor. Gets out of your way, fast, minimal. But: uses its own file format (not plain text/markdown), monospace-only, limited export, aesthetically bare. HN commenters note frustration with "reading and writing large volumes of prose in monospaced fonts."

2. **Neovim/Vim with plugins** — The most common approach. zen-mode.nvim, Goyo, vim-pencil, twilight.nvim. Can be configured into a decent writing environment, but requires significant setup and — critically — you're still in your development editor. Mode-switching between "dev Neovim" and "writing Neovim" is a pain point (confirmed by user).

3. **Emacs with modes** — writeroom-mode, olivetti, etc. Same problem as Neovim — it's a dev tool wearing a writing costume.

4. **GUI distraction-free apps** — iA Writer, Ulysses, FocusWriter, Cold Turkey Writer. These nail the zen experience but are all graphical. iA Writer in particular is the gold standard for design philosophy.

#### What Writers Love (from GUI apps)

- **iA Writer's typography philosophy**: Monospace communicates "work in progress" (psychologically appropriate for drafting). Their "duospace" concept gives 150% width to m/M/w/W — keeps monospace benefits but flows better. Variable fonts adjust weight by size/device/background.
- **Focus Mode**: Dim everything except current sentence/paragraph. iA Writer offers Sentence/Paragraph/Typewriter modes.
- **The tool disappears**: No toolbars, no menus, no chrome. Just text.
- **Markdown as native format**: Plain text, portable, version-controllable.

#### What Writers Hate

- Proprietary file formats (WordGrinder's biggest complaint)
- Complexity / feature creep
- Subscription models
- Poor export options
- Grammar tools that strip voice (Grammarly complaint — relevant for what NOT to build)
- Monospace fatigue for long prose sessions

#### The Gap

There is no terminal app that:
- Is purpose-built for creative writing (not a reconfigured code editor)
- Has thoughtful typography and visual design within terminal constraints
- Uses markdown/plain text natively
- Provides a zen experience comparable to iA Writer
- Offers vim-style editing without being vim
- Has zero visible chrome during writing

### Implications

The niche is wide open. The closest thing is "Neovim with plugins," which confirms both the demand and the inadequacy of the current solution. Zani's core value proposition: **a terminal app that IS the writing mode** — not a dev tool with a writing plugin bolted on.

Key design principles to carry forward from iA Writer:
- Typography matters enormously (even within terminal constraints)
- "Work in progress" visual feel is a feature, not a limitation
- Focus modes (sentence/paragraph dimming) are beloved
- The tool should be invisible during writing
- Markdown is the right format

The vim-mode question is interesting: writers who live in the terminal want modal editing, but they don't want to configure vim. Zani should have vim keybindings built in, not as a plugin.

**User note:** Has used zen-mode.nvim — likes the focus feature, but wouldn't call it beautiful. The features are right but the aesthetic execution is lacking. This confirms the core thesis: the gap is design quality, not feature set.

---

## Question 2: What Rust TUI frameworks exist, and which is best suited for a beautiful, smooth terminal writing experience?

**Method:** Web search, documentation review, ecosystem analysis

### Findings

#### The Stack: Ratatui + Crossterm + Ropey

The Rust TUI ecosystem has converged clearly:

**Ratatui** is the dominant TUI framework. Key capabilities:
- Sub-millisecond rendering with zero-cost abstractions
- 60+ FPS even with complex layouts and real-time data updates
- Immediate-mode rendering with intelligent double-buffering (only redraws what changed)
- 30-40% less memory than Go's Bubbletea equivalent
- Rich widget ecosystem, active development
- Supports multiple backends (Crossterm, Termion, Termwiz)

**Crossterm** is the recommended backend — cross-platform (including Windows), most popular, best documented. Default choice unless targeting a specific terminal.

**Ropey** is the standard rope data structure for text editing in Rust. Handles huge texts (100MB+) with ease, 1.8M+ small incoherent insertions/sec, cheap cloning (8 bytes for initial clone via shared data). Used by zee-editor and others.

**tui-textarea** exists as a text editor widget for ratatui with basic vim emulation, undo/redo, search. Could serve as a starting point or reference, but Zani would likely outgrow it — it's designed as an embedded widget, not a full writing experience.

#### Critical Concern: Text Wrapping

Ratatui has known issues with soft-wrapping and scrolling wrapped text. The Paragraph widget wraps text, but:
- Wrapped lines are still one Line object internally, causing scroll position inaccuracies
- The reflow module is "fairly convoluted and difficult to understand"
- There's an open RFC (issue #293) on text wrapping design
- Scrolling interaction with wrapped text has open issues (#2342)

**This is the single biggest technical risk for Zani.** A prose writing app lives and dies by how it handles soft-wrapped text and scrolling. Options:
1. Build custom wrapping/scrolling on top of ratatui's raw cell buffer
2. Contribute fixes upstream
3. Use ratatui for layout/chrome but handle the text viewport ourselves

Option 3 is likely the right call — ratatui for the app shell, custom rendering for the writing surface.

#### Color and Aesthetics

- True color (24-bit / 16.7M colors) is supported by virtually all modern terminals
- **Exception: macOS Terminal.app** only supports ANSI 256 — need graceful fallback
- Color detection via COLORTERM env var ("truecolor" or "24bit")
- Subtle colors, gradients, and muted palettes are all achievable on modern terminals
- The Charm ecosystem (Go: Lipgloss, Glamour) is the aesthetic benchmark for terminal design — CSS-like styling, adaptive color profiles, automatic color coercion for limited palettes

#### Design Inspiration: Charm's Approach

Charm (Go ecosystem) has set the bar for beautiful terminal design:
- Declarative, CSS-like styling (padding, margins, borders)
- Automatic color profile detection and graceful degradation
- Glamour for markdown rendering with stylesheets
- Focus on making terminal UIs "simple and fun"

Ratatui doesn't have Lipgloss's elegance out of the box, but its immediate-mode rendering gives full control. The aesthetic layer would need to be Zani's own.

### Implications

**Architecture emerging:**
- Ratatui + Crossterm for the application shell, layout, and input handling
- Ropey for the text buffer (handles large documents, efficient edits)
- Custom text viewport rendering for the writing surface (bypassing ratatui's Paragraph widget limitations)
- Own styling/theming system inspired by Charm's design principles
- Color profile detection with graceful fallback (true color → 256 → basic)

**The text wrapping challenge is the key technical risk** that needs a spike or deeper investigation before committing to the architecture.

---

## Question 3: Is Rust the right language for a high-performance terminal writing app?

**Method:** Web search, performance benchmarking research, ecosystem comparison

### The Contenders

#### Rust (ratatui + crossterm)
- **Performance:** 30-40% less memory, 15% less CPU than Go equivalent. Zero GC. Sub-millisecond rendering.
- **Ecosystem:** Ropey (rope buffer), tui-textarea (reference widget), massive crate ecosystem
- **Aesthetics:** No Lipgloss equivalent — styling is functional but not elegant. Must build own aesthetic layer.
- **Risk:** Text wrapping issues in ratatui (solvable with custom viewport)
- **Maturity:** Very high. Production-proven in dozens of TUI apps.

#### Go (bubbletea + lipgloss)
- **Performance:** Good enough for most TUIs. But GC pauses exist.
- **Ecosystem:** Charm suite is the gold standard for beautiful terminal apps. Lipgloss (CSS-like styling), Glamour (markdown rendering), mature and cohesive.
- **Aesthetics:** Best out of the box. Lipgloss is genuinely elegant.
- **Risk:** GC pauses during typing. For a zen writing experience where every keystroke must feel instant, this is a real concern.
- **Maturity:** Very high. Charm is production-proven.

#### Zig (libvaxis / OpenTUI)
- **Performance:** C-like. Ghostty proves Zig can build exceptional terminal software.
- **Ecosystem:** libvaxis supports modern terminal extensions (Kitty keyboard, graphics, etc.). OpenTUI emerging (8.9k stars, powers OpenCode). But text editing libraries are immature.
- **Aesthetics:** Libvaxis has OSC 4 support (color palette queries) that ratatui lacks.
- **Risk:** Language still on 0.15.x, ecosystem is young, fewer libraries for text editing.
- **Maturity:** Low-medium. Exciting but a bet on the future.

#### C (notcurses)
- **Performance:** Exceptional. Multi-threaded rendering, pixel-level control, sixel/kitty graphics.
- **Ecosystem:** Most powerful rendering library. Rust bindings (notcurses-rs) exist but are v3 dev status.
- **Aesthetics:** Can do anything a terminal can do. But at C development velocity.
- **Risk:** C development velocity and safety. notcurses-rs bindings are immature.
- **Maturity:** notcurses itself is mature. Rust bindings are not.

### The Critical Insight: Typing Latency

Dan Luu's research on terminal latency (https://danluu.com/term-latency/) is essential reading:
- People perceive latency **down to 2ms**
- Modern terminals are often **100-200ms** keypress-to-screen (vs 30-50ms in 1970s/80s computers)
- Every layer adds latency: keyboard → OS → terminal → app → render → display
- The app layer should add **as close to zero** overhead as possible

For a zen writing experience, typing must feel like thought-to-screen with no perceptible gap. **Rust's zero-GC advantage is not a performance optimization — it's a UX requirement.** Go's GC pauses, even sub-millisecond, can introduce micro-stutters during sustained typing. You wouldn't notice in a dashboard app; you'd notice in a writing app where every keystroke matters.

### Decision: Rust

**Rust is the right tool for this job.** The reasoning:

1. **Latency floor:** Zero GC means the app layer contributes minimal, predictable latency. For sustained typing, this matters more than aggregate throughput.

2. **Control:** Ratatui's immediate-mode rendering gives pixel-level control over what gets drawn. When building a custom text viewport (which we'll need for proper soft-wrapping), this control is essential.

3. **Ropey:** Purpose-built rope data structure for text editors. Handles large documents with cheap cloning. No equivalent in Go.

4. **The aesthetic gap is solvable:** Go's advantage is Lipgloss, not the language. Zani can build its own styling/theming system. Charm's principles (CSS-like declarations, adaptive color profiles, graceful degradation) are design patterns to adopt, not code to import.

5. **Zig is compelling but premature:** Libvaxis's modern terminal support is interesting, and OpenTUI is worth watching. But the ecosystem isn't ready for a full writing app. Zig might be the right choice in 2-3 years.

6. **Cross-platform:** Crossterm handles Windows, macOS, Linux uniformly. Important for reach.

### What Rust Doesn't Give Us (and how to address it)

- **No Lipgloss:** Build a theming module inspired by Charm's principles — declarative style definitions, adaptive color profiles, graceful degradation per terminal capability.
- **Text wrapping weakness:** Build custom viewport renderer (already planned)
- **No OSC 4:** Could add terminal capability detection manually, or wait for ratatui to add it
- **Steeper learning curve:** Accepted tradeoff for the performance guarantee

---

## Question 4: What does research say creates a "zen" writing experience, and how does that translate to a terminal?

**Method:** Web search across cognitive science, UX research, typography studies, flow psychology, and design philosophy

### Findings

#### 1. Flow States and Writing (Csikszentmihalyi)

Writing is one of the activities most naturally conducive to flow. Csikszentmihalyi found that activities with rules, skill requirements, and clear feedback produce flow most easily — writing has all three. The conditions for flow:

- **Clear goals** — know what you're doing next (the next sentence, the next paragraph)
- **Immediate feedback** — see words appearing as you think them
- **Balance of challenge and skill** — the writing is hard enough to engage but not so hard it overwhelms
- **Sense of control** — the tool does what you expect, instantly
- **Loss of self-consciousness** — you forget the tool, the room, yourself
- **Transformation of time** — hours feel like minutes

**Design implication:** The tool must be *invisible*. Any moment the writer notices the tool — a lag, a visual distraction, an unexpected behavior — breaks flow. The goal isn't "nice features." It's *disappearance*.

#### 2. Cognitive Load and the Power of Removal

UX research consistently shows: every visible element imposes cognitive load. The brain must process and decide to ignore each one. Minimalist interfaces don't just look clean — they *free cognitive resources for the actual task*.

Key findings:
- "Less is more" isn't aesthetic preference — it's cognitive science
- Whitespace/negative space creates "breathing room" that *calms* and reduces processing load
- Strategic emptiness signals quality and creates emotional comfort
- Google's homepage is the canonical example: almost nothing visible, yet immensely powerful

Oliver Reichenstein (iA Writer creator): *"In iA Writer, you have no choice but to think and write."* His philosophy: design is thought. The typewriter as inspiration — constraint enables focus. The tool *removing options* is a feature.

**Design implication:** Zani's default state should be *almost nothing*. Text, cursor, breathing room. No status bar, no line numbers, no file name, no word count — unless specifically requested. Settings exist but are invisible until summoned.

#### 3. Typography: What the Research Says

**Monospace vs. proportional for prose:**
- Proportional fonts are objectively faster and more comfortable to read for long prose
- BUT: monospace communicates "work in progress" / "draft" — psychologically beneficial during creative writing
- iA's insight: monospace *slows you down intentionally*, which is appropriate for writing (writing should be "measured, reflected, slow")
- iA's duospace compromise: 150% width on m/M/w/W keeps monospace rhythm but flows better

**Optimal line length:**
- Research consensus: **55-66 characters per line** is optimal for reading comfort and comprehension
- 66 characters is often cited as the sweet spot
- Both very short and very long lines slow reading by disrupting eye movement patterns
- WCAG guideline: maximum 80 characters per line for accessibility

**Line spacing:**
- Increasing line spacing from 100% to 120% **improves reading accuracy by 20%** and **reduces eye strain by 30%**
- Generous line height is one of the simplest high-impact improvements

**Design implication:** Center text in a ~60-character column with generous line spacing. Use a monospace or duospace font by default (leveraging the terminal's natural monospace rendering). The "constraint" of monospace is actually a *feature* for a writing tool — it says "this is a draft, keep going."

#### 4. Color and Contrast: What Actually Reduces Strain

**Critical findings:**
- **Never pure black (#000) on pure white (#FFF)** — causes eye strain and "halation" effect (letters appear to bleed/glow)
- **Never pure white text on pure black background** — same halation problem, worse for people with astigmatism (~33% of population)
- Optimal: **soft dark gray background + warm light gray text** — 4.5:1 contrast ratio minimum (WCAG)
- In dark/low-light environments (common for writing), dark mode with muted colors reduces total light emission
- Yellow/warm-tinted text perceived as least visually fatiguing (Apple's eye protection mode)

**Design implication:** Zani's color palette should be warm and muted. Not #000/#FFF. Something like a deep warm gray background with a soft warm off-white text. Think: candlelit room, not fluorescent office. Multiple theme options, but the default should be researched and intentional.

#### 5. Focus Modes: Sentence and Paragraph Dimming

Every major zen writing app implements this, and writers love it:
- **Sentence focus:** Current sentence at full opacity, everything else dimmed (typically 30-40% opacity)
- **Paragraph focus:** Current paragraph at full opacity, surrounding paragraphs dimmed
- **Typewriter mode:** Current line stays vertically centered; text scrolls around you rather than you scrolling through text

iA Writer offers all three as toggleable modes. Calmly Writer, Focused, and others implement variations.

The psychological mechanism: dimming surrounding text reduces the temptation to re-read and edit what you've already written. It keeps attention on the *generative* act of writing, not the *editorial* act of revising. This is crucial for first drafts.

**Design implication:** All three focus modes should be core features, toggled easily. Dimming should use terminal opacity/color capabilities (true color makes this smooth). Typewriter mode is particularly interesting in a terminal — the cursor stays put, the *world* moves.

#### 6. Smooth Scrolling and Motion

Research shows smooth scrolling reduces cognitive load vs. jump scrolling — the brain can track continuous motion but is jarred by discontinuous jumps. However:
- Hard to implement in text editors (most measure scrolling in lines, not pixels)
- Can lag behind input speed if poorly implemented
- More important for *reading* than *writing* (when writing, you're mostly at the bottom/cursor position)

Typewriter mode largely eliminates the scrolling question — the text moves, you don't scroll.

**Design implication:** Typewriter mode as the primary interaction model sidesteps most scrolling UX issues. When scrolling is needed (reviewing what you've written), aim for smooth line-by-line movement rather than jarring jumps. Don't over-invest here — typewriter mode is the answer.

#### 7. The Typewriter Metaphor

The typewriter revival among writers is real and instructive:
- Writers describe typewriters as "distraction-free" — no internet, no backspace temptation, no formatting
- The physical, tactile feedback creates a sense of *making something*
- The inability to easily edit encourages forward momentum
- The "work in progress" aesthetic (visible corrections, imperfection) is psychologically freeing

**Design implication:** Zani should channel typewriter energy — forward momentum, imperfection is OK, the draft is a draft. Consider: optional "typewriter sounds"? Probably not core, but the *attitude* of the typewriter matters. The tool encourages *producing words*, not perfecting them.

#### 8. Ambient Sound (Contextual)

Research shows moderate ambient noise (~50-70dB) enhances creative work through "stochastic resonance" — slight distraction promotes abstract thinking. Complete silence can actually be *too* quiet, amplifying minor distractions.

**Design implication:** Not a core feature for v1, but interesting for future: optional ambient soundscapes. Many writers use external apps for this. Could be a differentiator later. Note for now, move on.

### Synthesis: The Zen Writing Experience Design Principles

From the research, seven principles emerge for Zani:

1. **The tool disappears.** Zero chrome by default. Text and cursor. Nothing else. Every visible element must earn its place.

2. **Generous breathing room.** Centered ~60-character column. Wide margins. 120%+ line spacing. The emptiness is the design.

3. **Warm, muted palette.** No pure black or white. Soft dark background, warm off-white text. Low total light emission. Feels like writing by candlelight, not under fluorescents.

4. **Focus modes as core feature.** Sentence, paragraph, and typewriter modes. Dimming surrounding text keeps the writer in generative mode, not editorial mode.

5. **Monospace as feature, not limitation.** The terminal's natural monospace rendering communicates "draft" — psychologically appropriate. Consider duospace-like rendering if achievable.

6. **Instant feedback, zero perceptible latency.** Every keystroke appears immediately. The flow state depends on the tool responding faster than conscious thought. This is why Rust matters.

7. **Forward momentum by design.** The tool encourages writing, not editing. Typewriter mode keeps you at the frontier. Settings are hidden. The only thing to do is write.

---

## Question 4b: What unique affordances does a terminal-native writing app have over a GUI app?

**Method:** Ecosystem exploration (plexus, llm-orc), web search on terminal composability

### Context: The User's Tool Ecosystem

The user has built two sophisticated tools that Zani could integrate with:

**Plexus** — A Rust-based knowledge graph engine with provenance tracking. All data enters through `ingest`, and semantics are extracted automatically. Tracks concepts, relationships, and *how knowledge entered the system* (source file, line, adapter, confidence). Exposes MCP tools including `ingest` (the single entry point for all data) and `evidence_trail` (query provenance for a concept).

**llm-orc (LLM Orchestra)** — A Python-based multi-agent ensemble orchestrator. Coordinates specialized models (local Ollama + cloud) to decompose complex problems. 25 MCP tools for execution, ensemble management, profile management. Can fan-out work, chain agents with dependencies, run scripts.

**Manza** — A Tauri-based markdown editor (GUI). Zani would be the terminal sibling.

### Findings: The Terminal Advantage

#### 1. Composability (What iA Writer Can Never Do)

The Unix philosophy: small tools with clean interfaces, composed via pipes and text streams. A terminal writing app is a *citizen of the ecosystem*, not a walled garden.

Concrete examples:
- `cat notes.md | zani` — edit piped input
- `zani draft.md --export` — scripted export workflows
- `zani --wordcount draft.md` — use in shell scripts
- Git hooks that trigger on save
- Integration with grep, sed, awk for batch text operations
- Shell aliases and scripts that automate writing workflows

iA Writer is a beautiful island. Zani is a node in a network.

#### 2. Knowledge Graph Integration (Plexus)

This is the killer differentiator. Because Plexus is an MCP server and Zani lives in the terminal:

- **Concept tracking across drafts:** As you write, Plexus could ingest your writing — not in real-time (that would break zen), but on save or on demand. Your writing feeds your knowledge graph, and semantics are extracted automatically.
- **Evidence trails for research writing:** Working on an essay? Query Plexus for what you know about a concept, with provenance — which sources, which files, what confidence level.
- **Cross-document awareness:** Plexus knows about your other projects, notes, and research. Zani doesn't need to replicate that — it can query it.
- **Ingest from the writing surface:** Mark a passage as significant, tag it, and it flows through ingest into the knowledge graph with full provenance (file, line, context) and automatic semantic extraction.

This isn't "AI in your writing app." It's *your knowledge infrastructure* accessible from your writing surface, on demand, invisible when you don't want it.

#### 3. LLM Orchestration (llm-orc)

Not an inline AI assistant (that breaks zen). Instead:

- **On-demand ensembles:** Hotkey to run an ensemble against your current draft — style check, fact check, structure analysis — using coordinated specialized models, not one big model
- **Research assistance:** Trigger a research ensemble that fans out across multiple models to explore a topic, returns structured analysis
- **Draft comparison:** Run an ensemble that compares two versions of a passage
- **Cost-aware:** Use local Ollama models for quick checks, cloud models for deep analysis

The key: these are *pull* interactions. You invoke them when you want them. They don't interrupt. They don't autocomplete. They wait until summoned, like settings behind a hotkey.

#### 4. Session Persistence and Ubiquity

- **tmux/zellij integration:** Start writing, detach, SSH from another machine, reattach. Your writing session persists across connections, machines, network interruptions.
- **SSH writing:** Write on any machine with a terminal. A Raspberry Pi. A headless server. A cloud VM. No app installation required beyond the binary.
- **Multiplexer panes:** Writing in one pane, reference material in another, terminal in a third. All keyboard-driven, all in the same visual context.
- **No context switch:** Terminal users already live in the terminal. Opening a GUI app is a context switch. Zani means staying in flow.

#### 5. The $50-for-Fewer-Features Inversion

iA Writer charges $50 for:
- A beautifully constrained writing environment (yes, good)
- Proprietary platform (macOS/iOS/Windows/Android)
- No composability, no scripting, no integration
- Focus mode, markdown support, typography — that's it

Zani would be:
- Free/open source
- Runs everywhere a terminal exists
- Composable with every Unix tool
- Integrated with your personal knowledge graph and AI orchestration layer
- The zen writing experience is the *starting point*, not the whole product

The insight: iA Writer's *constraint* is its product. Zani's constraint is its *surface*, beneath which lies deep integration with the user's entire thinking infrastructure.

#### 6. Git as Native Capability

In a terminal, git isn't an "integration" — it's already there:
- Autosave + auto-commit = version history for free
- Branch per project/draft
- Diff between drafts is `git diff`
- Collaboration via standard git workflows
- Conflict resolution with familiar tools

In iA Writer, git support is either absent or a bolted-on feature. In Zani, it's the filesystem.

#### 7. Everyone Has a Terminal

This is the sneaky distribution advantage. iA Writer requires:
- macOS, iOS, Windows, or Android
- $50
- An app store download

Zani requires:
- A terminal (every computer has one)
- A single binary (Rust compiles to static binaries)

The target audience — writers who are comfortable in a terminal — is niche but *deeply underserved*. And that niche is growing as developer tools and terminal culture expand.

### Synthesis: Zani's Positioning

Zani isn't "iA Writer for the terminal." It's something iA Writer *cannot be*:

**A zen writing surface that sits atop deep integration with knowledge infrastructure, AI orchestration, and the Unix tool ecosystem — while presenting nothing but text, cursor, and breathing room.**

The writing experience is the front door. Behind it:
- Your knowledge graph (Plexus) ingests what you write, extracting semantics automatically
- Your AI ensembles (llm-orc) analyze, research, and critique on demand
- Git tracks every version automatically
- Unix pipes connect it to everything else

And all of it is invisible until you want it. Principle #1: the tool disappears.

---

## Question 5: What can we actually do typographically in a terminal to make prose feel beautiful?

**Method:** Web search on terminal rendering capabilities, escape codes, font protocols, terminal emulator configuration

### Findings

#### What the App CAN Control

**1. Per-character color (true color / 24-bit RGB)**
Ratatui's Text → Line → Span hierarchy allows styling every individual character with any of 16.7 million colors. This is the foundation for:
- **Focus dimming:** Active sentence at full warm off-white, surrounding text interpolated toward background color (simulating opacity)
- **Gentle gradients:** Text can fade smoothly from full brightness to dim over several lines
- **Warm muted palette:** Precise control over exact foreground/background colors
- **Markdown syntax highlighting:** Subtle color cues for headings, emphasis, links — without being garish

**2. Text attributes (ANSI escape codes)**
Widely supported across modern terminals:
- **Bold** — for markdown `**strong**` emphasis
- **Italic** — for markdown `*emphasis*` (requires terminal + font support, very common now)
- **Dim** — built-in attribute that reduces intensity; useful for metadata, status text
- **Underline** (including curly underline on some terminals) — for links or highlights
- **Strikethrough** — for ~~deleted~~ text in markdown

**3. Unicode visual elements**
Box-drawing characters available for any decorative chrome:
- Rounded corners `╭─╮ ╰─╯` — softer, friendlier feel than sharp corners
- Light lines `─ │` — subtle separators
- Gradient fills `░▒▓█` — for progress bars or visual effects
- Em dash `—`, ellipsis `…`, smart quotes `"" ''` — typographically correct prose rendering

**4. Cursor styling**
Most modern terminals support cursor shape changes via escape codes:
- Block, underline, or bar cursor
- Blinking or steady
- Zani could use a thin bar cursor for writing (like iA Writer) vs block for vim normal mode

**5. Kitty text sizing protocol (Kitty-specific, graceful fallback)**
On Kitty terminals, apps can render text at different sizes:
- Scale factor 1-7x for headings (`# Heading` rendered larger)
- Superscripts/subscripts
- Fully backwards-compatible — terminals without support simply ignore the codes
- This is the only way to get heading size differentiation in a terminal

#### What the App CANNOT Control (But Can Recommend)

**1. The actual font**
The terminal emulator chooses the font, not the app. BUT:
- **iA Writer's fonts are free and open source** (MIT licensed, on GitHub)
- **Nerd Font patched versions exist** for iA Writer Mono, Duo, and Quattro
- Zani can ship with a "recommended setup guide" suggesting iA Writer Duo as the terminal font
- iA Writer Duo IS the duospace font — 150% width on m/M/w/W. If the user sets this as their terminal font, they get the duospace writing experience for free.

This is elegant: Zani doesn't need to *implement* duospace. It just needs to *recommend* the font that already is duospace.

**2. Line height / cell spacing**
Each terminal emulator has its own configuration:
- Kitty: `modify_font cell_height 120%` (also `cell_width`, `baseline`)
- WezTerm: `line_height = 1.2`
- Ghostty: `adjust-cell-height = 20`
- Alacritty: `offset.y` in font config

Zani can detect the terminal and suggest optimal settings, or ship config snippets.

**3. Font ligatures and OpenType features**
Terminal-dependent. Kitty, WezTerm, and iTerm2 support ligatures. Alacritty does not. Not critical for a writing app (ligatures matter more for code), but nice-to-have.

**4. Anti-aliasing, subpixel rendering, gamma**
Entirely terminal emulator domain. Kitty offers gamma curve adjustment.

#### The Focus Dimming Implementation

This is core to the zen experience and fully achievable with true color:

```
Approach: Interpolate foreground color toward background color

Active text:     rgb(220, 215, 205)  — warm off-white, full brightness
Paragraph -1:    rgb(150, 146, 139)  — ~60% toward background
Paragraph -2:    rgb(100, 97, 92)    — ~30% toward background
Background:      rgb(40, 38, 35)     — warm dark gray

Result: Active paragraph glows. Surrounding text recedes into
the background like text on a page you're not looking at.
This is pure color math — no special terminal features needed.
```

This works on ANY terminal with true color support (virtually all modern terminals except macOS Terminal.app, which gets a 256-color approximation).

#### The Typography Strategy

Zani's approach to terminal typography:

1. **Ship recommended font configs** — iA Writer Duo (Nerd Font patched) as the default recommendation, with setup guides for Kitty, WezTerm, Ghostty, Alacritty, iTerm2
2. **Ship recommended terminal settings** — line height 120%, cell width, suggested color scheme
3. **Use true color for ALL visual design** — focus dimming, warm palette, markdown highlighting
4. **Use bold/italic for markdown rendering** — `**bold**` → bold, `*italic*` → italic, subtly
5. **On Kitty: use text sizing protocol** for headings — `# Heading` rendered at 2x, graceful fallback on other terminals (use bold + color instead)
6. **Centered ~60-character column** — app controls this via layout, not terminal settings
7. **Smart typography** — auto-convert straight quotes to curly quotes, -- to em dash, ... to ellipsis
8. **Cursor as design element** — thin bar cursor in insert mode (like iA Writer), block in vim normal mode

### Implications

The critical insight: **Zani doesn't need to fight the terminal's font constraints. It works WITH them.**

The terminal's monospace grid + the user's chosen font (recommended: iA Writer Duo) + Zani's true-color visual design + generous centered column layout = a writing experience that can genuinely rival iA Writer's aesthetics.

The focus dimming implementation is the biggest aesthetic win, and it requires zero special terminal features — just true color, which is nearly universal.

The one premium feature is Kitty's text sizing protocol for true heading sizes. This is a "delight" feature on Kitty, with graceful fallback (bold + color differentiation) everywhere else.

---

## Question 5b: How can we eliminate the font-switching friction?

**Method:** Web search on terminal font control APIs, CLI arguments, and alternative rendering approaches

### The Problem

The user has Inconsolata Nerd Font in their terminal. They type `zani`. They want to immediately be in a beautiful writing environment — ideally with a writing-optimized font like iA Writer Duo — without reconfiguring their terminal first.

### The Options

#### Option 1: Launch a Dedicated Writing Window (RECOMMENDED)

Every major modern terminal supports CLI config overrides:

- **Ghostty:** `ghostty --font-family="iA Writer Duo" --font-size=16 --adjust-cell-height=20 -e zani draft.md`
- **Kitty:** `kitty -o font_family="iA Writer Duo" -o font_size=16 -o modify_font="cell_height 120%" zani draft.md`
- **WezTerm:** `wezterm --config-file ~/.config/zani/wezterm.lua start -- zani draft.md`
- **Alacritty:** `alacritty -o font.normal.family="iA Writer Duo" -o font.size=16 -e zani draft.md`

**How it works:** When you type `zani draft.md`, Zani:
1. Detects your terminal emulator
2. Spawns a NEW terminal window with writing-optimized settings (font, line height, colors, padding)
3. Runs itself inside that window
4. When you quit Zani, the window closes

Your development terminal is untouched. The writing window is a separate, clean space.

**Why this is actually better than modifying the current terminal:**
- Clean separation: writing window vs. development terminal
- No need to restore settings on exit
- The act of launching creates a distinct *space* for writing — this IS the zen transition
- Full control over font, line height, padding, colors
- Works across all modern terminals

**The UX:** `zani draft.md` → a beautiful, dedicated writing window appears → you write → you quit → back to your dev terminal. Zero config. Zero friction.

For users who WANT to run Zani inline (already configured their terminal, or over SSH), a flag: `zani --inline draft.md` skips the window launch.

#### Option 2: Modify Current Terminal In-Place (LIMITED)

**Kitty** can change font SIZE via remote control (`kitten @set-font-size`) but CANNOT change font FAMILY at runtime. The maintainer confirmed: "Currently kitty cannot change fonts after running."

No terminal supports changing font family via escape codes for the current window. OSC 50 (xterm font change) exists in theory but isn't supported by any modern terminal.

**Verdict:** Dead end for the current window.

#### Option 3: Render Text as Images via Graphics Protocol (NUCLEAR OPTION)

Kitty graphics protocol and Sixel allow rendering arbitrary pixels to the terminal. In theory, Zani could:
- Rasterize text using any font (via a library like cosmic-text or fontdue)
- Send the rendered pixels to the terminal as images
- Achieve complete font independence

**Problems:**
- Breaks text selection, copy/paste, and accessibility
- Extremely complex to implement (essentially building a text renderer)
- Only works on terminals with graphics protocol support
- Performance concerns for real-time keystroke rendering
- Cursor rendering becomes non-trivial

**Verdict:** Technically fascinating, practically insane for v1. Maybe a future experiment, but Option 1 is better in every practical dimension.

### Decision: The Dedicated Window Approach

Option 1 is the clear winner. It:
- Solves the friction problem completely (type `zani`, get writing environment)
- Works with any font, any line height, any terminal settings
- Aligns with the zen philosophy (entering a dedicated writing *space*)
- Is simple to implement (shell detection + exec)
- Degrades gracefully (--inline flag for SSH/tmux/manual config)

**Implementation sketch:**
```
zani draft.md
  ├─ Detect terminal ($TERM_PROGRAM, $KITTY_PID, $WEZTERM_EXECUTABLE, etc.)
  ├─ Am I already in a Zani window? (check env var ZANI_WINDOW=1)
  │   ├─ Yes → run inline
  │   └─ No → spawn terminal window with Zani config
  │       ├─ ghostty: ghostty --font-family=... -e zani --inline draft.md
  │       ├─ kitty: kitty -o font_family=... zani --inline draft.md
  │       ├─ wezterm: wezterm --config-file=... start -- zani --inline draft.md
  │       ├─ alacritty: alacritty -o font...=... -e zani --inline draft.md
  │       └─ unknown: run inline, suggest font config
  └─ ZANI_WINDOW=1 → rendering loop

zani --inline draft.md
  └─ Skip window launch, run in current terminal
```

Zani ships with a config file (`~/.config/zani/config.toml`) that includes:
- Preferred font (default: iA Writer Duo Nerd Font)
- Font size (default: 16)
- Line height (default: 120%)
- Color scheme
- Terminal-specific overrides

The first time you run Zani, if the recommended font isn't installed, it can offer to install it (iA Writer fonts are free/MIT licensed).
