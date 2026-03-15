# Product Discovery: Zani

*2026-03-14*

## Stakeholder Map

### Direct Stakeholders

- **The Writer** — a person who wants to sit down and write prose with as little friction as possible. Comfortable in a terminal. Values local-first tools, markdown, and composability with Unix workflows. May be a novelist, essayist, academic, or anyone who writes extended prose and wants the environment to support rather than interrupt their creative state.

- **The Builder** — the developer-writer who maintains Zani. Also a direct user — builds the tool they want to use. Dual stakeholder: product decisions are informed by firsthand writing experience, not abstracted user research. This is a strength (authentic domain knowledge) and a risk (blind spots around assumptions that feel obvious to the builder but aren't universal).

### Indirect Stakeholders

- **The Reader** — the eventual audience for what the writer produces. Affected by whether the writing environment supports sustained creative quality, but never interacts with Zani directly.

- **Terminal Emulator Developers** — their capabilities and escape code support constrain what Zani can render. Zani degrades gracefully across terminal profiles but cannot control the font, window chrome, or color accuracy of the host terminal.

- **The Broader Ecosystem** — Zani is envisioned as one tool in a suite of writer-focused applications sharing the philosophy "tools that help you be more generative rather than generate for you." Other tools in this ecosystem (Plexus for knowledge graphs, llm-orc for model orchestration) are complementary — they handle analysis and organization, Zani handles the writing itself.

## Jobs and Mental Models

### The Writer

**Jobs:**
- Drop into focused writing immediately — open a file and start writing, with no setup, no configuration screens, no decisions required
- Stay in generative mode — the environment should keep me writing forward, not re-reading and revising
- Feel like the writing matters — the environment should feel beautiful and considered, not utilitarian
- Match the mood of my project — different writing calls for different aesthetic registers (dark and contemplative vs. bright and energetic)
- Trust that my work is safe — saving happens automatically, the file is plain markdown on disk, git handles versioning
- Compose with my existing tools — tmux for session management, git for version control, shell scripts for export workflows

**Mental Model:**
There is nothing stopping you from writing. No chrome, no decisions, no setup. But beyond the absence of friction, the environment actively creates a feeling conducive to writing — typewriter mode centering the cursor, sentence focus dimming everything but the current thought, a palette that puts you in a pleasing colorspace. The feeling is: you *want* to write.

The writer thinks of Zani as a room they walk into. The room has a mood (the palette), the lighting keeps their attention on the current thought (focus dimming), and the room is clean — no clutter on the walls, no notifications, no menus. When they need to adjust something (change the palette, toggle focus mode), they reach for a control that's hidden until needed. The text they're writing is the only thing that matters; everything else is environment.

The word "palette" activates the art-making metaphor — the writer's text is the paint, the palette is the medium. This is deliberate: the writer is the artist, not the tool.

### The Builder

**Jobs:**
- Ground every design choice in research or direct experience — no "vibes-based" decisions
- Maintain the integrity of the curated collection — every palette must earn its place through species congruence, WCAG compliance, and chroma target conformance
- Keep the tool fast — sub-millisecond response, zero GC pauses, the tool must never be the bottleneck
- Build something that could extend to a broader ecosystem — the methodology (affective essentialism) should transfer, the architecture should compose

**Mental Model:**
The builder thinks of Zani as an instrument — a carefully tuned device where every parameter has a measurable rationale. The 40 palettes are not a theme gallery; they are calibrated mood instruments backed by color science. The naming register is not decoration; it is a constitutive part of the instrument through the Label-Feedback Hypothesis. Building Zani is an act of research-driven craft, not feature accumulation.

## Value Tensions

- **Curation vs. personalization.** The writer might want "their own" palette. Zani provides 40 curated options instead. The tension resolves through alternative curated registers (e.g., celestial bodies) that conform to the same affective essentialism standards — extending the collection without opening it to arbitrary user-created themes. But the boundary between "curated extension" and "customization" is undefined.

- **Invisibility vs. discoverability.** The tool disappears by default (Invariant 1). But a new user needs to discover that focus modes exist, that palettes can be browsed, that local config binds palettes to projects. How does a writer learn what's available without chrome that teaches them? The tool that disappears is also the tool that's hard to learn. Two candidate solutions: (a) show a help dialog on first launch that can be disabled in settings, or (b) launch into the Settings Layer on first run so the writer sees what's available, then disable automatic settings-on-launch. Both preserve Invariant 1 for returning users while solving cold-start discovery.

- **Terminal-native vs. accessible.** The target writer is terminal-comfortable. But "writers who want local-first, frictionless writing" is broader than "people who already use terminals." The distribution and onboarding story determines whether Zani reaches writers who would love it but don't yet live in terminals.

- **Single document vs. writing workflow.** Zani handles one document at a time — that's the focused writing constraint. But writing workflows involve multiple documents (notes, outlines, drafts, references). The answer is "use tmux" — but this pushes workflow orchestration outside the tool, which may feel fragmented to a writer who doesn't already use tmux.

- **Vim-first vs. approachable.** Vim bindings are first-class. Non-vim editing is also supported. But the default experience — which mode does a new user land in? — determines whether the tool feels welcoming or intimidating. The tension is between honoring the power-user identity and being approachable.

## Assumption Inversions

| Assumption | Inverted Form | Implications |
|------------|--------------|-------------|
| The writer is terminal-comfortable | What if the ideal user *would* love a terminal writing app but doesn't know terminals yet? | Distribution and onboarding become product-critical, not afterthoughts. A wrapped distribution (Electron shell? dedicated terminal window with pre-configured font?) might be the bridge. |
| Color priming meaningfully affects writing | What if the effect is real but too small to matter in practice? | The palette system is still valuable as aesthetic curation (beautiful beats ugly), but the "mood instrument" framing would be overstated. The tool still works; the story changes. (Domain model Open Question 7 acknowledges this.) |
| Writers want the tool to disappear | What if some writers want ambient information — word count, session timer, progress toward a goal? | The current design hides everything. A writer who wants a visible word count has no option short of summoning the Settings Layer. Lightweight ambient signals might not violate Invariant 1 if they're subtle enough. |
| 40 palettes is enough | What if the curated set feels limiting? What if writers want palettes from different naming registers? | Alternative registers (celestial bodies, geological formations, etc.) are architecturally supported — the palette system is register-agnostic. But no mechanism exists to install or switch between registers. |
| Autosave is always preferred | What if a writer wants to abandon changes? | Currently no "revert to last saved" because saving is automatic and continuous. The writer's escape hatch is git (revert to last commit), but this requires git knowledge. |
| Pull-only integrations are sufficient | What if a writer wants real-time feedback — a gentle nudge when a sentence is getting too long, or when passive voice creeps in? | Invariant 8 prohibits background processing during writing. The assumption is that any background analysis would be a distraction. But "ambient, non-interruptive" feedback might be possible without violating the spirit of the invariant. |
| Single document at a time is correct | What if the writer needs to glance at notes while writing? | tmux is the answer, but it requires the writer to manage their own layout. A "reference pane" within Zani would violate the single-document constraint but might serve a real workflow need. |

## Product Vocabulary

| User Term | Stakeholder | Context | Notes |
|-----------|-------------|---------|-------|
| Palette | Writer | "I want to switch my palette" / "This palette feels right for this project" | Not "theme" — palette activates the art-making metaphor. The writer's text is the paint. |
| Focus mode | Writer | "Turn on focus mode" / "I want sentence focus" | The writer thinks in terms of on/off and variants (sentence, paragraph), not opacity factors or dimming mechanics. |
| Typewriter mode | Writer | "I want typewriter mode on" | The writer thinks of this as a single toggle. They don't distinguish "scroll mode" from "focus mode" — the system separates these concerns, but the user may not. |
| Distraction-free | Writer | "I want a distraction-free writing experience" | The overall promise. Not a feature — the absence of features. The writer measures success by what they don't notice. |
| Drop in | Writer | "I want to drop into writing" | The speed of getting from intent to prose. No setup, no boot time, no decisions. Open file, start writing. |
| Beautiful | Writer | "I want the writing experience to feel beautiful" | Not decorative — environmental. The writer means the typography, color, and spacing create a space they want to be in. |
| Local-first | Writer | "I want my writing to be local" / "No cloud, no subscription" | Files on disk, plain markdown, git-friendly. The writer owns their data. |
| Room | Writer (mental model) | "Walking into" a palette | The metaphor for the palette as environment — you're inside it, not looking at it. |
| Mood | Writer | "This palette has the right mood for my novel" | The affective state the palette primes. Not "color scheme" — the writer talks about mood, not hue. |
| Register | Builder | "A naming register of Cascadian flora" | Builder vocabulary. The writer might say "the plant names" or "the nature theme." The register concept is system-level. |

## Product Debt

| Assumption | Baked Into | Actual User Need | Gap Type | Resolution |
|------------|-----------|-----------------|----------|------------|
| Distribution is not a product concern yet | No install mechanism, no packaging, no onboarding | Writers who would love the tool need to find it, install it, and learn it | Missing workflow | Open — needs research spike on distribution (Homebrew formula, cargo install, wrapped distribution) |
| The writer already knows vim (or can toggle to non-vim) | Default editing mode unspecified in scenarios | A writer who opens Zani cold needs to know which mode they're in and how to start typing | Missing onboarding | Default to insert/non-vim mode with discoverable vim activation? Or clear mode indicator? |
| The writer knows tmux for multi-document workflows | Single-document architecture, no built-in session management | Writers who work across multiple files need guidance on how to set up their environment | Missing workflow | Documentation, or a `zani --session` launcher that opens a pre-configured tmux layout |
| Provenance descriptions surface somewhere useful | Provenance exists in code but display location undefined (Open Question 9) | The writer might want to know why a palette is named "Oregon Sunshine" — the species connection adds meaning | Missing UI surface | Palette Browser detail view, or hover/info action in the browser |
| Alternative naming registers are architecturally possible | Palette system is register-agnostic per ADR-015 | A writer who exhausts the Cascadian flora register might want palettes from a different naming world | Over-abstraction (built for one register, claims extensibility) | Validate that the architecture actually supports multiple registers before claiming it |
