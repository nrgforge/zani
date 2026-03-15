# Roadmap: Zani

**Generated:** 2026-03-14
**Derived from:** System Design v1.0, ADRs 001–018

---

## What Is Already Built

The following are complete and covered by 448 passing tests. They are listed here
to make the boundary between built and remaining explicit.

- Writing Surface (custom ratatui render, soft-wrap, centered column)
- Editor (ropey buffer, cursor, undo/redo)
- Focus Mode (sentence and paragraph dimming with animated chase values)
- Scroll Mode (Edge and Typewriter)
- Markdown Styling (bold, italic, headings, links, code — per-character)
- Smart Typography (curly quotes, em dash, ellipsis)
- Vim Bindings (modal key handling)
- Palette system (40 curated PNW flora palettes, 8 affective categories,
  OKLCH hue sorting, CIEDE2000 validation, Chroma Targets, ADR-009 through ADR-018)
- Palette Browser state machine (`palette_browser.rs`) and UI (`draw_palette_browser`)
- Settings Layer state machine (`settings.rs`) and UI (`draw_settings_layer`)
- Config module: `Config`, `LocalConfig`, `ConfigSource`, `load_for_path`,
  `bind_to_project`, provenance-aware persistence (ADR-011, ADR-013)
- Color Profile detection and hybrid 256-color degradation (ADR-012)
- Autosave: `Persistence`, `should_autosave`, `autosave` (3-second idle trigger)
- Writing Window: `spawn_command` for Ghostty, Kitty, WezTerm, Alacritty, iTerm2

---

## Work Packages

### WP-A: Settings Layer and Palette Browser — Integration and UI Polish

**Objective:** Wire the Settings Layer and Palette Browser through `app.rs` and `ui.rs`
so the full cycle (summon → navigate → select palette → crossfade → dismiss)
works end-to-end and matches the scenarios in ADR-010 and ADR-013. The state machines
and draw functions exist; the integration seams need closing.

**Changes:**
- `app.rs`: verify all `handle_settings_key` and `handle_palette_browser_key` paths
  match ADR-013 provenance rules (Local vs. Global save-on-change vs. save-on-quit)
- `ui.rs`: verify `draw_settings_layer` renders the Config row with correct affordance
  text ("Config: project" vs. "Config: global [enter]") based on `config_source`
- `ui.rs`: verify `draw_palette_browser` shows `(project)` / `(global)` marker on
  active palette per `config_source` (code present; needs scenario coverage)
- Acceptance tests: `settings → browser → apply → crossfade` and
  `file open → local config resolution → palette switch`

**Scenarios covered:**
- Feature: Chrome and Settings Layer — all scenarios
- Feature: Palette Browser (ADR-010) — all scenarios
- Feature: Config Persistence Provenance (ADR-013) — all scenarios
- Feature: Local Config (ADR-011) — "Opening a file in a different project triggers
  palette crossfade"

**Dependencies:** None — all modules exist. This is integration closure.

---

### WP-B: Writing Window — Multiplexer Detection and `--window` Flag

**Objective:** Complete the Writing Window launch path so `zani --window` works in
supported terminals and degrades gracefully everywhere else, including inside tmux/screen.
`spawn_command` is built; the multiplexer guard and the `main.rs` flag wiring are not.

**Changes:**
- `writing_window.rs`: add `is_multiplexer()` detection (`TMUX`, `STY` env vars) and
  return `Terminal::Unknown` (or a dedicated variant) when inside a multiplexer
- `main.rs`: parse `--window` flag; call `spawn_command` when set and not already in
  a Writing Window (`ZANI_WINDOW=1`); fall through to inline otherwise
- `writing_window.rs`: test: multiplexer env vars suppress spawn

**Scenarios covered:**
- Feature: Writing Window — "Zani spawns a Writing Window with --window flag"
- Feature: Writing Window — "Zani does not re-spawn when already in a Writing Window"
- Feature: Writing Window — "Inline Mode inside terminal multiplexer" `[Planned]`
- Feature: Writing Window — "Unknown terminal falls back to Inline Mode"

**Dependencies:** None. Open choice relative to all other WPs.

---

### WP-C: Flora Reference Document

**Objective:** Author `docs/flora-reference.md` — a durable reference corpus with
one entry per active species in the palette collection. Each entry must satisfy the
field requirements in ADR-017: common name, scientific name, Signature Colors with
source citations, Essence Mode, geographic range, and palette assignment rationale.
This document is the ground-truth source for provenance bias audits (Open Question 11).

**Changes:**
- Create `docs/flora-reference.md` with 40 species entries (one per palette)
- Entries must be grounded in external sources first (field guides, herbaria), then
  compared against existing Provenance Descriptions — not the reverse
- Retired species (Columbine, Balsamroot, Reindeer Lichen) must not appear

**Scenarios covered:**
- Feature: Flora Reference (ADR-017) — all scenarios
- Integration Scenarios (ADR-016 + ADR-017) — provenance consistency check

**Dependencies:** WP-A (implied logic) — having the palette names and categories
stable first reduces churn in the reference. The palette collection is already
stable (40 species, ADR-015 complete), so this dependency is weak. Open choice
if the palette collection is confirmed frozen.

---

### WP-D: Field Guide

**Objective:** Author `docs/references/field-guide.md` — the writer-facing field
guide that explains how to use Zani: focus modes, palette browsing, local config,
writing window, and vim bindings. This is a human document, not a spec. It is the
primary onboarding artifact for cold-start discovery (Open Question 16).

**Changes:**
- Create `docs/references/field-guide.md`
- Cover: launch modes, Settings Layer (summon/dismiss), Palette Browser navigation,
  Focus Mode (sentence/paragraph/off), Scroll Mode, Local Config binding,
  Writing Window, Vim mode activation, Smart Typography

**Scenarios covered:**
- Feature: Writing Window — inline mode, --window flag, fallback behavior
- Feature: Chrome and Settings Layer — cold-start discoverability
- Addresses Open Question 16 (cold-start discoverability) partially

**Dependencies:** WP-A (implied logic) — Settings Layer and Palette Browser should
be integration-complete before documenting them. WP-B (implied logic) — Writing
Window behavior should be final before documenting the `--window` flag.

---

### WP-E: External Integrations (Pull-Only)

**Objective:** Wire Plexus ingest and llm-orc ensemble invocation as opt-in, pull-only
operations triggered by explicit hotkey. No automatic or background processing.
Results must be presented without modifying the Buffer.

**Changes:**
- `app.rs`: add hotkey bindings for Ingest and Ensemble actions (keys TBD)
- New module or extension of `app.rs`: call out to Plexus ingest pipeline with current
  buffer content (or selection)
- New module or extension of `app.rs`: invoke llm-orc ensemble; display results in
  a transient overlay without writing to buffer
- `ui.rs`: add overlay render for integration results (follows existing overlay pattern)

**Scenarios covered:**
- Feature: External Integrations (Pull-Only) — "Plexus ingest is triggered on demand"
  `[Planned]`
- Feature: External Integrations (Pull-Only) — "llm-orc ensemble is invoked on demand"
  `[Planned]`

**Dependencies:**
- WP-A (hard dependency) — Settings Layer must be integration-complete before adding
  new overlay surfaces that follow the same chrome conventions
- Plexus and llm-orc must be available in the environment (external dependency,
  not code dependency)

**Open choice:** The result presentation surface (transient overlay vs. side panel vs.
status line) is undesigned. Builder must choose before implementing `ui.rs` changes.

---

### WP-F: Distribution and Onboarding

**Objective:** Make Zani installable by writers who are not already in this repository.
Determine the distribution channel (Homebrew formula, `cargo install`, or wrapped
distribution) and implement the minimum viable install path. Address cold-start
discoverability (Open Question 16) with either a first-launch help dialog or
automatic Settings Layer on first run.

**Changes:**
- `Cargo.toml` / release pipeline: confirm binary name, version, and metadata for
  `cargo install` or crate publication
- `main.rs`: detect first run (no config files exist, no file argument) and implement
  chosen cold-start behavior (help dialog or Settings Layer auto-open)
- Homebrew formula or packaging script (if chosen channel is Homebrew)
- `docs/references/field-guide.md` must exist before this WP ships (writers need
  something to read)

**Scenarios covered:**
- Open Question 16 (cold-start discoverability)
- Open Question 18 (default editing mode — non-vim default for broadest audience)
- Open Question 19 (distribution and onboarding)

**Dependencies:**
- WP-A (hard dependency) — Settings Layer must be polished before it can serve as
  the first-run onboarding surface
- WP-D (implied logic) — Field Guide should exist before distribution so writers
  have documentation to reference
- Open Question 18 (default editing mode) must be resolved before first-run behavior
  is finalized: the cold-start experience depends on which mode the writer lands in

---

## Dependency Graph

```
WP-A (Settings/Browser integration)
  └── WP-E depends on WP-A [hard]
  └── WP-F depends on WP-A [hard]

WP-B (Writing Window / multiplexer)
  └── WP-D depends on WP-B [implied]
  └── WP-F independent of WP-B [open choice]

WP-C (Flora Reference)
  └── independent [open choice — palette collection is frozen]

WP-D (Field Guide)
  └── WP-D depends on WP-A [implied]
  └── WP-D depends on WP-B [implied]
  └── WP-F depends on WP-D [implied]

WP-E (External Integrations)
  └── WP-E depends on WP-A [hard]

WP-F (Distribution / Onboarding)
  └── WP-F depends on WP-A [hard]
  └── WP-F depends on WP-D [implied]
```

Flattened sequencing by hard dependencies only:

```
WP-A  ──►  WP-E
WP-A  ──►  WP-F

WP-B  (independent — can run in parallel with WP-A)
WP-C  (independent — can run in parallel with everything)
WP-D  (implied after WP-A and WP-B)
```

---

## Transition States

Each state below is a coherent, shippable intermediate where the tool works and no
half-finished surface is exposed.

**State 1: Integration complete, no distribution**
WP-A and WP-B are done. The full Settings Layer → Palette Browser → crossfade cycle
works. `zani --window` spawns correctly or falls back. Config provenance (Local/Global)
persists correctly. Flora Reference and Field Guide do not exist yet. Integrations not
wired. Usable by a developer who can clone the repo.

**State 2: Documented**
WP-C and WP-D are done. The Flora Reference provides a citable record for provenance
audits. The Field Guide documents the tool for a writer who has never opened it.
Still no distribution mechanism — a developer still needs to `cargo build`.

**State 3: Distributable**
WP-F is done. Writers who are not developers can install Zani. Cold-start discoverability
is solved. External integrations (WP-E) remain optional and do not block distribution.

**State 4: Fully integrated**
WP-E is done. Plexus ingest and llm-orc ensemble invocation are available on hotkey.
This state has no additional prerequisites beyond WP-A.

---

## Open Decision Points

The following require a builder decision before work can proceed. They are not design
questions that more planning resolves — they require either a spike or a directional
choice.

**1. Cold-start behavior (blocks WP-F)**
Two candidates: (a) help dialog on first launch, dismissible and disableable in settings;
(b) Settings Layer auto-opens on first run. The choice determines what `main.rs` detects
and what the Field Guide says about getting started. Directional lean from product
discovery: option (b) uses an existing surface without adding new chrome.

**2. Default editing mode (blocks WP-F)**
Non-vim is the recommended default for the broadest audience (product discovery).
The builder (a vim user) wants deeper vim motion support. These can coexist — the
question is what a writer encounters when they type their first character with no config.
Decision needed before implementing first-run behavior.

**3. Integration result surface (blocks WP-E)**
Plexus ingest and llm-orc results need a display surface that does not modify the buffer
and follows the "tool disappears" invariant. Options: transient overlay (like find),
a dedicated side panel, or results written to a separate file. The choice determines
how much new UI surface WP-E requires and how it interacts with the overlay stack.

**4. Provenance bias audit method (deferred to WP-C)**
Open Question 11 documents that existing provenance descriptions may rationalize palette
colors rather than document the species. WP-C (Flora Reference) is the instrument for
auditing this — entries must be written from external sources first. The audit methodology
(A/B/C/D scoring from ADR-016) is already defined; the trigger for running it on all 40
entries should be decided before WP-C work begins.

**5. Distribution channel (blocks WP-F)**
Three candidates: Homebrew formula, `cargo install` (requires crates.io publication),
wrapped distribution (shell around a binary with pre-configured terminal). The target
audience question ("writers who already use terminals" vs. "writers who would love a
terminal app") determines whether the channel is `cargo install` (developer-accessible)
or something with lower friction. A spike on Homebrew formula complexity vs. crate
publication is the fastest way to resolve this.
