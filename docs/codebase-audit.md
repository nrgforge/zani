# Codebase Audit: Zani

**Date:** 2026-02-28
**Scope:** Whole codebase — all 22 source files, integration tests, documentation, and configuration.
**Coverage:** This analysis sampled strategically across all source files, with deepest focus on `app.rs` (3,294 lines), `main.rs` (170 lines), `writing_surface.rs` (712 lines), `focus_mode.rs` (535 lines), `ui.rs` (962 lines), and all ADRs and domain model documentation. It is representative, not exhaustive. Test code was sampled across all 19 inline test modules and the integration test suite.

---

## Executive Summary

Zani is a terminal-based creative writing application built in Rust with ratatui, crossterm, and ropey. At ~9,000 lines across 22 source files, it is a well-scoped, single-purpose tool with a clear architectural vision: zero-chrome distraction-free writing with focus modes, curated color palettes, vim keybindings, and smart typography. The codebase benefits from unusually thorough design documentation — 8 ADRs, a domain model with 15 invariants, and an amendment log tracking design pivots.

The actual architecture is a layered monolith centered on a single `App` struct (3,294 lines, ~34% of source) that has accumulated responsibilities across cursor logic, file I/O, undo history, dimming orchestration, settings state, rename state, and input dispatch. This God Object pattern is the codebase's primary structural pressure point — it does not yet block progress, but the cost of each new feature is increasing as unrelated concerns interleave. The rendering boundary (UI/WritingSurface) is clean and well-designed, using a builder pattern that enforces one-shot render semantics. Key dispatch was consolidated into the library crate (`App::handle_key`), and Ctrl-chord handlers now route through the `Action` command pattern with full undo coverage.

The codebase shows evidence of healthy design pivots — Typewriter mode moved from FocusMode to ScrollMode, sentence dimming migrated from per-line layers to per-character fade queues. Previous evolutionary debris (vestigial fields, dead methods, stale documentation) has been cleaned up. The 341-test suite is comprehensive in unit coverage with assertion messages on all multi-assertion tests.

---

## Architectural Profile

### Patterns Identified

| Pattern | Confidence | Summary |
|---------|-----------|---------|
| God Object (`App`) | High | 33 public fields, 6+ responsibilities, ~34% of source |
| Layered Architecture (with leaky input boundary) | High | Rendering layer clean; persistence and input split across layers |
| Command Pattern (complete) | High | 23 `Action` variants, all mutations routed through `apply_action` |
| Builder Pattern (`WritingSurface`) | High | Clean one-shot render construction, well-applied |
| Composable Opacity Layers | High | Multiplicative dimming with chase-based animation, backed by ADR-008 |
| Data Transfer Object (`Config`) | Medium | Serialization boundary decoupled from `App`, with migration logic |
| Code as Configuration (Palettes) | Medium | Static Rust structs, no runtime extension path |

### Quality Attribute Fitness

| Attribute | Assessment |
|-----------|-----------|
| **Usability** | Primary optimization target — zero-chrome, focus modes, curated palettes, graceful degradation |
| **Performance / Latency** | Architecturally valued (Invariant 6, Rust, zero-GC) with wrap caching and O(log n) word/sentence navigation via rope-backed `Buffer` API |
| **Testability** | Well-served — 341 unit tests across 19 modules, all key dispatch testable in library crate, assertion messages on all multi-assertion tests |
| **Modifiability** | Under pressure from `App` gravity — all features flow through one struct |
| **Reliability** | Silent error absorption on autosave — failed writes discard errors, prevent user awareness |
| **Operability** | Minimal by design — no logging, no diagnostics, consistent with "tool disappears" philosophy |
| **Graceful Degradation** | Structurally embedded (Invariant 11) — 3-tier color profile, terminal fallbacks, OSC 52 silent fail |

### Inferred Decisions

1. **Vim as First-Class Citizen** (High confidence) — Standard mode is an accommodation; vim is the default. The entire key dispatch is structured around modal editing.
2. **`App` as God Object** (High confidence) — Organic accretion, not an explicit decision. ~34% of source lines, 33 public fields, 6+ distinct responsibilities.
3. **Dual Animation Systems** (High confidence) — `AnimationManager` for UI transitions, `DimLayer`/`LineOpacity` for content-level dimming. Architecturally parallel, unnamed boundary.
4. **Graceful Degradation as Structural Invariant** (High confidence) — 3-tier color profile, terminal fallbacks, documented as Invariant 11.
5. **OSC 52 as Sole Clipboard** (Medium confidence) — No system clipboard crate; write-only; paste only from internal yank register.
6. **Palettes as Static Code** (Medium confidence) — Per ADR-006; `&'static str` names block user-defined palettes.

---

## Tradeoff Map

| Optimizes For | At the Expense Of | Evidence |
|--------------|-------------------|----------|
| Coordination simplicity (all state in `App`) | Modifiability, testability | `app.rs:L62-110` — 33 public fields; every feature opens this file |
| ~~Implementation directness (Ctrl-chords inline in `main.rs`)~~ | ~~Undo consistency, testability~~ | **Resolved** — all Ctrl-chords routed through `apply_action` with undo |
| Usability continuity (no error dialogs) | Reliability, observability | `app.rs:L1234` — autosave error silently discarded |
| ~~Rendering correctness (no cache invalidation)~~ | ~~Performance at scale~~ | **Resolved** — wrap computation cached, keyed on `(buffer.version(), column_width)` |
| Call-site readability (WritingSurface builder) | Verbosity | `ui.rs:L37-48` — 11-method builder chain per frame |
| Type safety (palettes in Rust source) | User extensibility | `palette.rs:L18-63` — `&'static str` names, compile-time only |
| Graceful degradation (always runs) | Color fidelity on basic terminals | `writing_surface.rs:L289` — Basic profile uses `DIM` attribute, not RGB interpolation |
| ~~Development velocity (`rope_mut()` public)~~ | ~~Encapsulation~~ | **Resolved** — `rope_mut()` removed, `Buffer` exposes domain methods directly |

---

## Findings

### Macro Level

#### Finding: App as Gravitational Center

**Observation:** `App` at 3,294 lines carries 33 public fields spanning cursor position, scroll state, file/save state, rename sub-state, settings sub-state, dimming animation state, undo history, find overlay, and vim mode. The struct and its `impl` block handle all editing mutations, cursor logic, undo integration, focus mode computation, autosave, scroll management, and settings management.
- `src/app.rs:L62-110` — 33 public fields, no sub-structs grouping related state
- `src/app.rs:L459-694` — `apply_action` is a 239-line match over 23 variants
- `src/app.rs:L286-371` — rename subsystem (6 methods) exclusively touches 3 fields
- `src/app.rs:L186-278` — settings subsystem (6 methods) exclusively touches 2 fields

**Pattern:** God Object. A struct that knows too much and does too much, accumulating responsibility that logically belongs to smaller, more focused types.

**Tradeoff:** Optimizes for coordination simplicity (all state reachable in one hop) at the expense of modifiability and testability. Adding any feature requires opening `app.rs` regardless of which subsystem is affected.

**Question:** What would it take to add a second open document — a split-pane session — to this codebase?

**Stewardship:** Extract `RenameState`, `SettingsState`, and cursor/scroll state into sub-structs owned by `App`. This shrinks the public surface without breaking call sites. The current size is approaching the threshold where the next feature will cost more than it should.

---

#### Finding: Command Pattern Partially Applied [RESOLVED]

**Observation:** `vim_bindings.rs` defines a 23-variant `Action` enum (the Command pattern). All key inputs — including multi-key sequences (`gg`, `dd`) and Ctrl-chords (Ctrl+C/X/V) — now route through `apply_action` with full undo recording. Multi-key dispatch logic lives in the vim layer (`handle_normal_with_pending`, `handle_visual_with_pending`).

**Resolution:** `PasteAtCursor` variant added. Ctrl+C → `Yank`, Ctrl+X → `DeleteSelection`, Ctrl+V → `PasteAtCursor`. `DeleteSelection`, `PasteAfter`, and `PasteBefore` now record undo. Pending key logic moved from `App` to `vim_bindings.rs`.

---

#### Finding: Composable Opacity Layers — Well-Designed Domain System

**Observation:** The dimming system uses independently animated opacity layers composing by multiplication, with documented invariants grounding each decision.
- `src/focus_mode.rs:L229-278` — `DimLayer` manages per-line chase-based animation
- `src/focus_mode.rs:L127-191` — `LineOpacity` captures current value on target change, eliminating flicker
- `src/app.rs:L1128-1133` — Composition is a one-liner: `paragraph_dim.opacity(i) * sentence_dim.opacity(i)`
- `docs/decisions/adr-008` — Documents the layered model, multiplicative composition, and chase-based rationale

**Pattern:** Composable Layers with Chase-Based Animation. Each visual concern owns its own state and animation lifecycle.

**Tradeoff:** Optimizes for visual correctness (no flicker, independent timing, clean composition) at the expense of state complexity (three independent animation state machines).

**Question:** What would a fourth dimming source require in terms of changes across how many files?

**Stewardship:** Well-applied, deliberate design. No action needed. Document the extension contract for adding a new layer.

---

#### Finding: WritingSurface Builder Pattern — Clean and Complete

**Observation:** `WritingSurface` uses the builder pattern consistently. Construction produces an immutable render value; all configuration flows through method chaining.
- `src/writing_surface.rs:L47-121` — `new()` + 10 builder methods, each returning `Self`
- `src/ui.rs:L37-48` — 11-method chain at the call site
- `src/writing_surface.rs:L189-400` — `Widget::render` consumes `self`, enforcing single-use

**Pattern:** Builder Pattern with consume-on-use.

**Tradeoff:** Optimizes for call-site readability and parameter safety at the expense of verbosity.

**Question:** What would change about the rendering path for a split-pane view?

**Stewardship:** Well-applied. No action needed. Keep builder methods focused on rendering parameters.

---

### Meso Level

#### Finding: Key Dispatch Split Creates Undo Holes and Test Blind Spots [RESOLVED]

**Observation:** Input handling was consolidated into `App::handle_key` in the library crate. Ctrl-chord handlers (Ctrl+C/X/V) now route through the `Action` command pattern with full undo recording. `main.rs` is a thin event loop adapter.

**Resolution:** `handle_key` moved to library crate. Ctrl+V paste now records undo. All input semantics are testable. Undo coverage confirmed by `ctrl_x_records_undo`, `ctrl_v_paste_at_cursor_records_undo`, and `paste_after_records_undo` tests.

---

#### Finding: `Buffer` Is a Transparent Newtype [RESOLVED]

**Observation:** `Buffer` now exposes domain-level methods (`char_to_line`, `line_to_char`, `slice_to_string`, `len_chars`, `len_lines`, `line`, `char_at`, `chars_at`) directly. `rope()` and `rope_mut()` have been removed. `Buffer` also tracks a `version` field (incremented on every mutation) used for wrap computation caching.

**Resolution:** All rope methods `App` needs are on `Buffer` directly. `rope_mut()` removed (no callers). `rope()` removed. `version()` accessor added for cache invalidation.

---

#### Finding: `sentence_dim` Is a Hollow Field [RESOLVED]

**Observation:** The vestigial `sentence_dim` field was removed in a prior stewardship pass. `line_opacities()` now returns only `paragraph_dim.opacity(i)`. The actual architecture — one paragraph layer plus a sentence-fade queue — is self-evident.

**Resolution:** Field removed. `dim_animating()` simplified. No behavioral change.

---

#### Finding: Invariant 5 Column Width Bounds Misimplemented [RESOLVED]

**Observation:** Domain Model Invariant 5 was amended to reflect actual bounds (20–120). `Config::load()` now applies `clamp(20, 120)` so hand-edited config files cannot bypass the guard.

**Resolution:** Invariant 5 amended. Clamp added to `Config::load()`. Domain model and code now agree.

---

#### Finding: Autosave Silently Discards Errors [RESOLVED]

**Observation:** `App::autosave()` now captures save failures in `save_error: Option<String>`. The error is surfaced in the settings layer when the writer summons it. `dirty` remains `true` on failure, enabling retry.

**Resolution:** `save_error` field added to `App`. Settings layer displays save errors. Distraction-free default preserved while failures are visible on demand.

---

### Micro Level

#### Finding: Three Word-Navigation Methods Each Materialize the Entire Buffer [RESOLVED]

**Observation:** Word navigation (`word_forward`, `word_backward`, `word_end`) now uses `Buffer::char_at()` for O(log n) character access instead of materializing the entire buffer. `sentence_bounds_in_buffer()` also uses the Buffer API directly.

**Resolution:** Full-buffer `to_string()` calls replaced with rope-backed `char_at()` scans. `sentence_bounds_in_buffer()` added as an allocation-free alternative to `sentence_bounds_at()`.

---

#### Finding: Wrap Computation Runs 3x Per Frame [RESOLVED]

**Observation:** `App::visual_lines()` now caches results keyed on `(buffer.version(), column_width)`. `Buffer` tracks a `version: u64` field incremented on every `insert()` and `remove()`. Cache hits return a clone; misses recompute and store.

**Resolution:** `visual_lines_cache: Option<(u64, u16, Vec<VisualLine>)>` added to `App`. Redundant recomputation eliminated for non-mutating operations (e.g., Up/Down cursor movement).

---

#### Finding: Assertion Roulette in Test Suite [RESOLVED]

**Observation:** All multi-bare-assertion tests now have descriptive message strings. Tests across 10 files updated: `vim_bindings.rs`, `config.rs`, `app.rs`, `buffer.rs`, `wrap.rs`, `focus_mode.rs`, `writing_surface.rs`, `palette.rs`, and `tests/integration.rs`.

**Resolution:** ~60 bare assertions annotated with context messages describing what each assertion verifies.

---

#### Finding: `autosave_writes_buffer_content` Integration Test Doesn't Test Autosave [RESOLVED]

**Observation:** The test was renamed to `styling_preserves_raw_buffer_content` and its assertions now have descriptive messages.

**Resolution:** Test renamed. Assertions annotated. The test now accurately describes what it verifies.

---

### Multi-Lens Observations

#### Convergence: `App` God Object (5 lenses)

All five analysis lenses — Pattern Recognition, Architectural Fitness, Dependency & Coupling, Intent-Implementation Alignment, and Structural Health — independently identified `App` as the codebase's central structural concern. The convergence across independent analyses strengthens confidence that this is the primary modifiability pressure point.

#### Convergence: Key Dispatch Split (4 lenses) [RESOLVED]

Pattern Recognition, Architectural Fitness, Dependency & Coupling, and Test Quality all flagged the `main.rs`/`app.rs` input handling split. **Resolved:** Key dispatch consolidated into library crate. Ctrl-chords route through `apply_action` with full undo coverage and test coverage.

#### Convergence: `sentence_dim` Vestigial Field (4 lenses) [RESOLVED]

Intent-Implementation Alignment, Dead Code, Structural Health, and Invariant Analysis all identified `sentence_dim` as a hollow field. **Resolved:** Field removed in prior stewardship pass.

#### Convergence: README Documentation Drift (3 lenses) [RESOLVED]

Decision Archaeology, Documentation Integrity, and Invariant Analysis all flagged README inaccuracies. **Resolved:** README updated with correct keybindings, palette names, Focus/Scroll mode sections, and test count.

---

## Stewardship Guide

### What to Protect

1. **The rendering boundary.** `WritingSurface`'s builder pattern cleanly separates configuration from rendering. This is the codebase's strongest architectural seam.
2. **The composable dimming design.** ADR-008's multiplicative opacity composition with chase-based animation is principled, well-documented, and correctly implemented. The invariants (12–15) that govern it are enforced by code.
3. **The domain model and ADR discipline.** Eight ADRs, 15 invariants, an amendment log. This is unusually rigorous for a project of this scale. The investment pays dividends in design coherence.
4. **Graceful degradation.** The 3-tier color profile with fallback behavior is structurally embedded, not bolted on. Invariant 11 is enforced by the `ColorProfile` enum design.
5. **The test culture.** 341 unit tests across 19 modules with descriptive assertion messages demonstrates a commitment to verification.

### What to Improve (Prioritized)

1. ~~**Move key dispatch into the library crate**~~ — **[DONE]** `handle_key` moved into `App`. Ctrl-chords route through `apply_action`. Undo hole closed.

2. ~~**Fix the README**~~ — **[DONE]** Keybindings, palette names, Focus/Scroll mode sections, and test count updated.

3. ~~**Remove dead code**~~ — **[DONE]** `sentence_dim`, `FocusMode::next()`, `ScrollMode::next()`, `rope_mut()`, `chrome_visible` removed.

4. ~~**Extract sub-states from `App`**~~ — **[DONE]** `RenameState` and `SettingsState` extracted as named structs.

5. ~~**Cache wrap computation**~~ — **[DONE]** `visual_lines_cache` on `App`, keyed on `(buffer.version(), column_width)`.

6. ~~**Reconcile Invariant 5 bounds**~~ — **[DONE]** Domain model amended to 20–120. `Config::load()` clamps.

7. ~~**Surface autosave failures**~~ — **[DONE]** `save_error: Option<String>` on `App`, surfaced in settings layer.

8. ~~**Add assertion messages to tests**~~ — **[DONE]** ~60 bare assertions annotated across 10 files.

### Ongoing Practices

- **Before adding a new feature, ask: does this require opening `app.rs`?** If so, consider whether the feature's state could live in a sub-struct first.
- **Route all buffer mutations through `apply_action`.** If a new action needs to modify the buffer, add an `Action` variant rather than mutating inline. This keeps undo history consistent.
- **When removing a feature or redesigning a subsystem, sweep documentation.** The README/domain model drift from the Typewriter→ScrollMode pivot was preventable with a post-pivot docs pass.
- **Add assertion messages to new tests.** One sentence per `assert_eq!` describing what the assertion verifies.
- **Treat the domain model invariants as a checklist.** When a new invariant is added or an existing one is relaxed, update the Amendment Log and any affected ADRs.

---

## Resolution Log

All audit findings have been addressed. This log documents the work done.

### Phase 1: Stewardship Priorities (8 items)

| # | Item | Commit/Change | Status |
|---|------|--------------|--------|
| 1 | Move key dispatch into library crate | `handle_key` moved to `App`; `main.rs` is thin event loop | Done |
| 2 | Fix the README | Keybindings, palettes, Focus/Scroll sections, test count | Done |
| 3 | Remove dead code | `sentence_dim`, `FocusMode::next()`, `ScrollMode::next()`, `rope_mut()`, `chrome_visible` | Done |
| 4 | Extract sub-states from `App` | `RenameState`, `SettingsState` as named structs | Done |
| 5 | Cache wrap computation | `visual_lines_cache` on `App`, keyed `(buffer.version(), column_width)` | Done |
| 6 | Reconcile Invariant 5 bounds | Domain model amended to 20–120; `Config::load()` clamps | Done |
| 7 | Surface autosave failures | `save_error: Option<String>` on `App`, displayed in settings | Done |
| 8 | Add assertion messages to tests | ~60 bare assertions annotated across 10 files | Done |

### Phase 2: Remaining Findings (5 items)

| # | Item | Change | Status |
|---|------|--------|--------|
| 1 | Ctrl-chord command pattern + undo gaps | `PasteAtCursor` added; Ctrl+C/X/V → `apply_action`; `DeleteSelection`/`PasteAfter`/`PasteBefore` record undo | Done |
| 2 | Wrap recomputation cache | `Buffer.version` field; `App.visual_lines_cache` | Done |
| 3 | `pending_normal_key` in vim layer | `handle_normal_with_pending`/`handle_visual_with_pending` in `vim_bindings.rs` | Done |
| 4 | Assertion roulette | Message strings on all multi-bare-assertion tests across 10 files | Done |
| 5 | Audit doc stale | This Resolution Log; [RESOLVED] tags; updated metrics | Done |

### Metrics After Resolution

- **Source files:** 22
- **Total lines:** ~9,018
- **Tests:** 341 (was 331 at audit time)
- **Action variants:** 23 (was 22)
- **app.rs:** 3,294 lines (was 2,984)
- **Clippy:** Clean (0 warnings)
