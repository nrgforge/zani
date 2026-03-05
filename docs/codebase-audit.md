# Codebase Audit: Zani

**Date:** 2026-03-04 (updated post-extraction)
**Prior audits:** 2026-03-03, 2026-03-02 (post-extraction series)
**Scope:** Whole codebase — all 27 source modules, integration tests, allocation benchmark, documentation, and configuration.
**Coverage:** This analysis sampled strategically across all source files via ten independent analytical lenses operating across three levels (Macro, Meso, Micro). Deepest focus on `app.rs` (1,776 lines), `editor.rs`, `ui.rs`, `writing_surface.rs`, `focus_mode.rs`, `animation.rs`, `persistence.rs`, `main.rs`, `settings.rs`, `config.rs`, all test modules, and documentation. Not covered: build artifacts, `.worktrees/` content, git object internals.

## Executive Summary

Zani is a ~10,500-line Rust terminal writing application built on ratatui, crossterm, and ropey. It implements a distraction-free creative writing environment with curated color palettes, sentence/paragraph focus dimming, vim modal editing, markdown-as-native-format, and smooth animations — all rendered through a custom `WritingSurface` widget that bypasses ratatui's built-in paragraph rendering for per-character styling control.

Since the prior audit (2026-03-03), a focused series of changes has resolved 13 of 15 findings. The highest-convergence finding — `column_width` duplicated between Editor and Viewport — was eliminated by removing the field from Editor entirely. `ui::draw()` was decoupled from `App` via a new `DrawContext` struct, completing the pattern started by `SettingsViewModel`. The `OverlayOpacity` shared animation slot was split into `SettingsOverlay`, `FindOverlay`, and `ScratchQuitOverlay`. The autosave `load_error` guard was moved into `autosave()` itself, closing the Ctrl+S bypass. `SettingsItem::all()` was converted to a `const` array. The `hjkl_moves_cursor` eager test was split into four. `alloc_bench` now asserts zero allocations for all three focus modes. The animation decision rule was documented. The `--inline` feature ghost was removed and `ZANI_WINDOW` now serves as an active re-spawn guard.

Two new features were added: (1) **external file change detection** with mtime tracking, auto-reload for clean buffers, and a conflict bar for dirty buffers; (2) **scratch quit prompt** giving users a Save/Rename/Discard choice when quitting a dirty scratch buffer. Both features use the new `DrawContext` and `ScratchQuitOverlay` animation infrastructure. A clipboard test isolation fix (`#[cfg(not(test))]` on `read_clipboard`) resolved 7 pre-existing test failures caused by system clipboard pollution.

A subsequent domain-logic extraction series removed 7 methods from App, moving them to the subsystems that own the relevant state: `Editor::reset_to_content()`, `Editor::can_vim_navigate()`, `Editor::set_editing_mode()`, `Palette::index_in_all()`, `Viewport::adjust_column_width()`, and `ScratchQuitState` (extracted to `settings.rs` with its own `handle_key` state machine). A coordinator invariant doc comment was added to App. App dropped from 1,839 to 1,776 lines. Three bug fixes followed: scratch quit now checks `buffer.has_content()` (not `dirty`), the dialog shows the generated filename, and rename renders as a standalone overlay outside settings.

The remaining open findings are: (1) documentation drift — README test count stale (says 354, actual is 380); (2) structural concerns — focus mode shotgun surgery (4 modules), WritingSurface builder speculative generality, integration test composition path gap; (3) product design decision — no user-visible save error indicator on the writing surface.

The codebase's strengths — domain modeling discipline, WCAG AA as a hard invariant, the pure vim state machine, version-keyed caching, the custom WritingSurface, the `App::tick()` frame contract, and the coordinator invariant — remain intact and should be protected.

## Architectural Profile

### Patterns Identified

| Pattern | Confidence | Evidence |
|---------|------------|----------|
| **Coordinator-Subsystem** (App routes input to extracted subsystems) | High | `app.rs` coordinator invariant doc comment; 11 subsystems; 7 domain methods extracted to subsystems; `tick()` returns `TickOutput` |
| **Immediate-Mode Game Loop** (tick → draw → poll → handle → repeat) | High | `main.rs:L92-145`; `App::tick()` returns `Option<TickOutput>` gating draw |
| **Command Pattern (partial)** via `Action` enum | High | `vim_bindings.rs` pure `char → Action`; `editor.rs` `apply_action`; Standard mode partially bypasses |
| **Builder Pattern with De Facto Required Arguments** on WritingSurface | Medium | 14 builder methods; single call site always invokes all 14; `Option<>` fields never `None` in production |
| **Demand-Driven Caching** via `Buffer::version()` | High | Monotonic `u64` drives `RenderCache`, visual line cache, sentence bounds cache invalidation |
| **Parallel Domain-Specific Animation Systems** (documented) | High | `AnimationManager` for palette/overlay; `AnimatedValue`/`DimLayer` for per-line dimming; decision rule in `animation.rs` module docs |
| **ViewModel Extraction** (DrawContext + SettingsViewModel) | High | `draw()` takes `&DrawContext` (not `&App`); `draw_settings_layer` takes `SettingsViewModel`; full decoupling achieved |

### Quality Attribute Fitness

| Attribute | Status |
|-----------|--------|
| **Performance** | Prioritized by design (Rust, zero-GC, visual line cache, zero-allocation enforced via `alloc_bench.rs` across all focus modes). WritingSurface find-match O(matches) per character. `SettingsItem::all()` now returns `&'static [SettingsItem]`. |
| **Usability** | Core strength — focus modes, curated palettes, WCAG AA enforcement, zero chrome, typewriter scrolling. New: scratch quit prompt (Save/Rename/Discard), external change conflict bar (Reload/Keep mine). |
| **Modifiability** | Significantly improved — `DrawContext` decouples `draw()` from App internals; `column_width` duplication eliminated; animation decision rule documented; 7 domain methods extracted from App to subsystems; coordinator invariant doc comment defines boundary rule. App at 1,776 lines (down from 1,839). |
| **Testability** | Strong — 380 unit tests, 3 integration, 1 alloc bench. Clipboard tests isolated via `#[cfg(not(test))]`. `hjkl` split into 4 tests. Autosave guard tested at both layers. Config round-trip tests cover all 5 fields. ScratchQuitState has 5 dedicated tests. |
| **Reliability** | Improved — `autosave()` now checks `load_error` directly (no bypass path). External file changes detected via mtime. Save errors visible in settings overlay. Scratch buffers get explicit quit prompt. |
| **Operability** | Gap — no logging infrastructure; error handling uses three incompatible strategies (silent discard, field storage, propagation). |
| **Accessibility** | Strong for active text (WCAG AA enforced in tests via `Palette::validate`); intentionally relaxed for dimmed text; interpolated dimming colors not validated. |

### Inferred Decisions

1. **Rust + zero-GC for physiology, not benchmarks** — Language chosen to meet a sensory latency invariant (Dan Luu's perceptual research), not throughput needs. Go explicitly rejected for GC pauses during sustained typing. (High confidence)
2. **Custom WritingSurface as first-class architectural layer** — Bypasses ratatui's Paragraph for domain-specific per-character rendering with dimming, markdown styling, and find-match composition. (High confidence)
3. **Vim as assumed user, Standard mode as accommodation** — Vim is `#[default]`, implemented first; Standard mode retrofitted. Settings layer uses hjkl unconditionally regardless of editing mode. (High confidence)
4. **Composable dimming via flat data structures** — Two implementations explored; simpler flat `Vec<LineOpacity>` won over trait-based `LayerStack`. (High confidence)
5. **OSC 52 clipboard write + subprocess clipboard read** — Writes via escape sequence (universal, zero-dependency); reads via platform-specific subprocesses with fallback to yank register. (High confidence)
6. **Coordinator extraction via incremental refactoring** — Two extraction series: the first established `tick()` and SettingsViewModel; the second extracted 7 domain methods to subsystems and added a coordinator invariant doc comment. App dropped from ~3,300 to 1,776 lines. (High confidence)
7. **Two animation systems for different scheduling semantics** — `AnimatedValue` provides chase-with-interruption for N simultaneous line opacities; `AnimationManager` provides discrete-with-prune for 2-3 global transitions (palette crossfade, overlay fade, scroll). Different domains, shared primitive. (Medium confidence — appears conscious but undocumented)

## Tradeoff Map

| Optimizes For | At the Expense Of | Evidence |
|--------------|-------------------|----------|
| Perceived latency (zero-GC, sub-1ms target) | Ecosystem convenience | Rust 2024 edition; 6 runtime deps; custom WritingSurface |
| Rendering correctness (stateless per-frame) | Performance at scale | O(n) markdown styling per frame; WritingSurface per-character loop |
| Testability (pub(crate) fields, isolated subsystems) | Encapsulation | App subsystem fields pub(crate); DrawContext extracts at boundary |
| Distraction-free UX (minimal error surfacing) | Operability & data safety | Save errors visible in settings overlay; no persistent status bar |
| Development velocity (incremental refactoring) | API stability | Coordinator boundary stabilizing; DrawContext + SettingsViewModel complete |
| Feature locality (dimming types in focus_mode) | Conceptual integrity | Dual animation system (now documented); focus dimming spans 4 modules |
| Zero-allocation on render path | Code complexity | WritingSurface fallback paths; RenderCache; DimmingState pre-allocated buffers |

## Findings

### Macro Level

#### Finding: App — Coordinator Label vs. Reality *(SIGNIFICANTLY IMPROVED)*

**Update (2026-03-04):** A 7-commit extraction series removed domain logic methods from App and placed them in the subsystems that own the relevant state: `Editor::reset_to_content()`, `Editor::can_vim_navigate()`, `Editor::set_editing_mode()`, `Palette::index_in_all()`, `Viewport::adjust_column_width()`, and `ScratchQuitState` (full state machine extracted to `settings.rs`). A coordinator invariant doc comment was added to App defining the boundary rule: "Does this read/write state from only one subsystem? If yes, it belongs on that subsystem." App dropped from 1,839 to 1,776 lines.

**Remaining:** App still contains inline palette interpolation (`effective_palette`) and animation-start logic embedded in `settings_apply`. These are the most displaced concerns — both orchestrate multiple subsystems, making them borderline coordination vs. domain logic. `settings_apply` dispatches per-item logic including animation start, mode mutation, and delegation.

**Pattern:** God Object in Extraction — actively being addressed. The coordinator invariant comment now provides a decision rule for future developers.

**Tradeoff:** Optimizes for co-location of state at the expense of modifiability. The extraction series materially reduced the problem; the remaining displaced logic is harder to extract because it genuinely coordinates multiple subsystems.

**Question:** When the next feature touches `effective_palette` or `settings_apply`, does the coordinator invariant help the developer decide where the new code belongs?

**Stewardship:** The extraction series and coordinator invariant are the right approach. Continue extracting when the next feature in either area makes the current arrangement costly. The remaining `effective_palette` and `settings_apply` logic are the next candidates.

---

#### Finding: Two Parallel Animation Systems *(RESOLVED)*

**Resolution (2026-03-04):** Decision rule now documented in `animation.rs` module-level doc comment (lines 12-21). The `ScratchQuitOverlay` variant was successfully added to `AnimationManager` following the documented rule, validating the decision.

**Original observation:** `AnimationManager` maintains discrete overlay/palette transitions; `DimLayer` maintains per-line chase animations. Both build on `AnimatedValue` but serve different scheduling semantics. The decision rule was undocumented.

**Stewardship:** Resolved. The documented decision rule guided the `ScratchQuitOverlay` addition correctly. No further action needed.

---

#### Finding: Rendering Split — draw() Takes &App While WritingSurface Has Explicit Interface *(RESOLVED)*

**Resolution (2026-03-04):** `DrawContext` struct introduced in `ui.rs`. `draw()` now takes `&DrawContext` instead of `&App`. The constructor extracts all rendering data through App's public accessor methods. New features (conflict bar, scratch quit overlay) were built directly on `DrawContext`, validating the pattern.

**Original observation:** `ui::draw()` took `&App` and accessed `pub(crate)` subsystem fields directly, bypassing the accessor facade.

**Stewardship:** Resolved. Future rendering features should add fields to `DrawContext` and populate them in the constructor.

---

#### Finding: Configuration Scatter/Gather Without Full Round-Trip Verification *(RESOLVED)*

**Resolution (2026-03-04):** Both round-trip tests (`from_config_round_trip` and `save_config_round_trip`) now verify all 5 config fields: palette name, focus_mode, column_width, editing_mode, and scroll_mode. The `save_config_round_trip` test uses `assert_eq!(recovered, original)` to catch any scatter/gather asymmetry mechanically. The prior audit's claim that only 2 fields were tested was incorrect.

**Remaining concern:** Adding a new persisted setting still requires changes in four locations (Config struct, from_config, save_config, settings UI). The round-trip tests will catch asymmetry between from_config and save_config, but won't catch a missing field in the UI.

**Stewardship:** Resolved. The round-trip tests provide the mechanical safety net recommended by the prior audit.

---

#### Finding: Clipboard Asymmetry Resolved but Subprocess Read Has No Error Surfacing

**Observation:** Clipboard write uses OSC 52 (escape sequence, universal). Clipboard read now uses platform subprocesses (`pbpaste`, `wl-paste`, `xclip`) with fallback to yank register. Read errors are silently discarded — a subprocess failure falls back to the yank register with no signal.
- `src/clipboard.rs` — write via OSC52, read via subprocess with fallback
- `src/app.rs` — Ctrl+V reads from clipboard; no error path

**Pattern:** Silent Fallback — errors in the external-process clipboard path are masked by the internal fallback, meaning the user cannot distinguish "pasted from system clipboard" from "pasted from yank register because clipboard read failed."

**Tradeoff:** Optimizes for graceful degradation at the expense of user awareness. Appropriate for clipboard (low-stakes), but the pattern should not be replicated for file operations.

**Question:** What signal does a user receive when `pbpaste` is not installed and they attempt Ctrl+V?

**Stewardship:** This is an acceptable tradeoff for clipboard operations. No action needed. The existing design correctly prioritizes functionality over error messaging for this non-critical path.

### Meso Level

#### Finding: `column_width` Duplicated Between Editor and Viewport (5-Lens Convergence) *(RESOLVED)*

**Resolution (2026-03-04):** `column_width` removed from `Editor` entirely. The field now lives exclusively in `Viewport`. App passes it through at call sites. The 5-lens convergence — the strongest finding of the prior audit — is fully resolved.

**Original observation:** `column_width` was stored independently in `Editor::column_width` and `Viewport::column_width` with manual synchronization at two call sites.

**Stewardship:** Resolved. The single-source-of-truth pattern should be maintained for any future state that might be duplicated across subsystems.

---

#### Finding: `OverlayOpacity` Shared Between Settings and Find Overlays *(RESOLVED)*

**Resolution (2026-03-04):** `TransitionKind::OverlayOpacity` split into three distinct variants: `SettingsOverlay`, `FindOverlay`, `ScratchQuitOverlay`. Each has its own accessor method (`settings_overlay_progress()`, `find_overlay_progress()`, `scratch_quit_overlay_progress()`). A test (`settings_and_find_overlays_coexist`) verifies independent coexistence.

**Original observation:** A single `OverlayOpacity` variant was shared by Settings and Find overlays with accidental coupling.

**Stewardship:** Resolved. Future overlays should add a new `TransitionKind` variant following this pattern.

---

#### Finding: `ui::draw()` Bypasses App's Accessor Facade *(RESOLVED)*

**Resolution (2026-03-04):** `DrawContext` struct introduced. `draw()` now takes `&DrawContext` instead of `&App`. The constructor assembles all rendering data through App's public accessor methods. This completes the pattern started by `SettingsViewModel`.

**Stewardship:** Resolved. See Macro-level "Rendering Split" finding for details.

---

#### Finding: Autosave Guard Has Reachability Gap *(RESOLVED)*

**Resolution (2026-03-04):** `load_error` check moved into `autosave()` itself (persistence.rs:L46). Both `should_autosave()` and `autosave()` now independently guard on `load_error.is_some()`. Test `autosave_refuses_when_load_error_set` verifies the file is not overwritten when `load_error` is set.

**Original observation:** `should_autosave` checked `load_error` but `autosave` did not, allowing the Ctrl+S path to bypass the guard.

**Stewardship:** Resolved. The "move guards into the guarded function" practice should be applied to future safety-critical paths.

---

#### Finding: Silent Error Handling Risks Data Loss

**Observation:** Three incompatible error strategies coexist: (1) `save_config` discards errors with `let _ = config.save()` (app.rs:L214); (2) autosave captures errors into `save_error: Option<String>`, visible only in settings overlay; (3) terminal setup propagates errors via `?`. A user whose document fails to save continuously will not know unless they open the settings panel.
- `src/app.rs:L214` — `let _ = config.save()` silently discards config write failure
- `src/persistence.rs:L57-59` — save error stored in field but only surfaced in settings overlay
- `src/main.rs:L67-89` — terminal setup propagates errors via `?`

**Pattern:** Silent Failure — the most consequential silence is autosave failure: a user can write for 30 minutes without knowing their work is not being persisted.

**Tradeoff:** Optimizes for visual minimalism and the distraction-free philosophy at the expense of reliability. No error dialogs, no popups, no status bars — but also no signal when data is at risk.

**Question:** If a user writes for 30 minutes in a session where autosave is silently failing due to a filesystem permission error, what is the first signal they receive?

**Stewardship:** This is a genuine tension between product philosophy and reliability. A minimal improvement preserving the zen aesthetic: a brief, auto-dismissing indicator in a corner of the writing surface when `save_error` is set, visible for a few seconds without requiring user action. The implementation is straightforward; the product decision requires deliberate choice.

---

#### Finding: Focus Mode Distributed Across Four Modules

**Observation:** Focus dimming spans `focus_mode.rs` (sentence parsing, opacity calculation, DimLayer, color math), `dimming.rs` (DimmingState orchestration), `app.rs` (sentence bounds caching, settings apply), and `writing_surface.rs` (per-character opacity application). Changing focus dimming semantics requires reading and editing at least three files.
- `src/focus_mode.rs:L40-268` — sentence parsing, DimLayer, `apply_dimming_with_opacity`
- `src/dimming.rs:L46-102` — `DimmingState::update()` orchestrates layers
- `src/writing_surface.rs:L277-296` — renderer applies per-character opacity
- `src/app.rs:L571-599` — sentence bounds caching in `tick()`

**Pattern:** Shotgun Surgery — any change to focus dimming semantics requires edits to at least three files.

**Tradeoff:** Optimizes for separation of computation from rendering at the expense of feature cohesion.

**Question:** What would a developer need to read to understand the complete semantics of sentence focus mode?

**Stewardship:** Push all opacity-computation logic into `DimmingState`, so `WritingSurface` only applies pre-computed opacities. This consolidates the feature's logic while preserving the rendering boundary.

---

#### Finding: Documentation Drift Persists *(RESOLVED)*

**Resolution (2026-03-04):** All items addressed:
- `--inline` flag removed from code and scenarios
- `ZANI_WINDOW` re-spawn guard implemented
- ADR-004 amended for opacity-based approach
- README architecture section updated with animation subsystem
- README test count updated (354 → 380)
- Plexus/llm-orc domain model entries already marked "_(Planned, not yet implemented.)_"
- tmux scenario already marked `[Planned]`

**Stewardship:** Resolved. Keep domain model "planned" markers current as features are implemented or dropped.

### Micro Level

#### Finding: WritingSurface Builder Has 11 "Optional" Fields Always Provided

**Observation:** WritingSurface defines 17 fields, of which 11 are typed as `Option<&'a ...>` or initialized to defaults in `new()`. The single call site in ui.rs:L38-55 always populates every field. The fallback paths inside `render()` (L412-444) that handle absent data compute the same values inline with heap allocation — the very allocation the cache was designed to prevent. These fallback paths are only exercised by test helpers.
- `src/writing_surface.rs:L185-195` — six fields declared `Option<&'a ...>`
- `src/writing_surface.rs:L412-444` — fallback paths that allocate `Vec` when precomputed data is absent
- `src/ui.rs:L38-55` — the only production call site, always provides all precomputed data

**Pattern:** Speculative Generality — the builder was designed with fallback paths for callers that don't have precomputed data. There is one caller and it always has the data. The fallback paths are dead in production, alive in tests.

**Tradeoff:** Optimizes for test isolation (tests can construct a surface without `RenderCache`) at the expense of clarity. The `Option<>` wrappers communicate "this may not be present" when the actual invariant is "this is always present."

**Question:** If a new developer reads `WritingSurface::new()` and sees `precomputed_visual_lines` defaulting to `None`, what assumption will they make about whether they need to provide it?

**Stewardship:** Add a comment at each `None => { ... }` branch stating: "This path is only reached from test helpers; production always provides precomputed data." Alternatively, collapse `Option<>` fields to required fields and update test helpers to supply minimal data.

---

#### Finding: `SettingsItem::all()` Allocates Vec on Every Keystroke *(RESOLVED)*

**Resolution (2026-03-04):** `SettingsItem::all()` now returns `&'static [SettingsItem]` backed by a `const ALL_ITEMS: [SettingsItem; 12]` array. Zero allocation on every call.

**Original observation:** `all()` returned a fresh `Vec<SettingsItem>` on every invocation, allocating 3-5 times per keypress during settings navigation.

**Stewardship:** Resolved.

---

#### Finding: `--inline` Flag and `ZANI_WINDOW` Guard Are Feature Ghosts *(MOSTLY RESOLVED)*

**Resolution (2026-03-04):** `--inline` flag removed entirely. `ZANI_WINDOW` now serves as an active re-spawn guard — main.rs checks `std::env::var("ZANI_WINDOW").is_err()` as a gate in the `--window` block, preventing unbounded recursion.

**Remaining:** The re-spawn guard is not unit tested. Testing it is awkward since it's in `main()` rather than a library function.

**Stewardship:** The `--inline` ghost is fully resolved. The re-spawn guard test is low priority — the guard is a single `if` condition in `main()` and the risk of regression is minimal.

---

#### Finding: `Persistence::is_scratch` Flag Has No Behavioral Consequence *(RESOLVED)*

**Resolution (2026-03-04):** `is_scratch` now drives four distinct behaviors: (1) Ctrl+Q on scratch with content opens the scratch quit prompt (Save/Rename/Discard) showing the generated filename; (2) Ctrl+Q on empty scratch silently deletes the draft file and exits; (3) non-scratch Ctrl+Q exits normally; (4) Rename from scratch quit prompt renders a standalone rename overlay outside the settings panel. The prompt checks `buffer.has_content()` (not `dirty`) so autosaved drafts still trigger the dialog. Tests: `scratch_with_content_opens_prompt`, `scratch_empty_quits_silently`, `scratch_autosaved_with_content_still_opens_prompt`, `scratch_save_choice_quits`, `scratch_discard_choice_quits`, `non_scratch_quit_unchanged`.

**Original observation:** `is_scratch` was set on construction and cleared after rename but had no behavioral effect on quit or save.

**Stewardship:** Resolved. The flag now has full lifecycle significance. The content-based check (not dirty-based) correctly handles the autosave interaction.

---

#### Finding: `column_width` Sync Tested at Init, Not at Mutation *(RESOLVED)*

**Resolution (2026-03-04):** `column_width` removed from Editor entirely (see `column_width` duplication finding). There is no longer a sync to test — Viewport is the single source of truth.

**Stewardship:** Resolved by eliminating the root cause.

---

#### Finding: Autosave Guard Tested via `should_autosave`, Not via `autosave` *(RESOLVED)*

**Resolution (2026-03-04):** Both code and test fixed. `autosave()` now checks `load_error` directly (persistence.rs:L46). Test `autosave_refuses_when_load_error_set` calls `autosave()` with `load_error` set and verifies the original file content is preserved.

**Stewardship:** Resolved.

---

#### Finding: `alloc_bench.rs` Only Asserts Paragraph Mode *(RESOLVED)*

**Resolution (2026-03-04):** All three focus modes (Off, Paragraph, Sentence) now have zero-allocation assertions. Three independent `assert!` calls verify `off_per_frame == 0`, `paragraph_per_frame == 0`, `sentence_per_frame == 0`.

**Stewardship:** Resolved.

---

#### Finding: `hjkl_moves_cursor` Is an Eager Test with Assertion Roulette *(RESOLVED)*

**Resolution (2026-03-04):** Split into four independent tests: `h_moves_cursor_left`, `l_moves_cursor_right`, `k_moves_cursor_up`, `j_moves_cursor_down`. Each tests exactly one key with its own setup and assertion, following the pattern established by `w`, `b`, `e` tests.

**Stewardship:** Resolved.

---

#### Finding: Integration Test Does Not Verify Actual Composition Path

**Observation:** `focus_dimming_and_markdown_styling_compose` manually calls `apply_dimming_with_opacity` and asserts the result `matches!(Color::Rgb(_, _, _))`. The function signature already guarantees an RGB return for RGB input — the assertion cannot meaningfully fail. The actual composition in `WritingSurface::render` (where dimming opacity is applied on top of markdown-resolved color) is not tested end-to-end.
- `tests/integration.rs:L62-70` — assertion checks type shape, not color values
- `src/writing_surface.rs:L625-937` — WritingSurface tests exist but none assert composed color values

**Pattern:** Test-code correspondence gap — the test name and comment promise integration verification but the test verifies helper arithmetic already covered by unit tests.

**Tradeoff:** Optimizes for test isolation at the expense of integration confidence.

**Question:** If `WritingSurface` were modified to apply focus dimming before markdown resolution (reversing composition order), would the test suite catch it?

**Stewardship:** Either rename the test to match what it actually verifies, or extend it to render a real frame and assert specific cell colors in the output buffer.

### Multi-Lens Observations

#### Convergence: `column_width` Duplication (5+ lenses) *(RESOLVED)*

The strongest convergent finding of the prior audit. Eliminated by removing `column_width` from Editor entirely.

#### Convergence: Dual Animation System (4 lenses) *(RESOLVED)*

Decision rule documented in `animation.rs` module docs. The `ScratchQuitOverlay` was added following the documented rule, validating the approach.

#### Convergence: Silent Error Handling (3 lenses) *(PARTIALLY RESOLVED)*

The autosave guard bypass is fixed (`autosave()` now checks `load_error`). Save errors are visible in settings overlay. Scratch quit prompt uses content-based check to avoid autosave masking the dialog. The broader concern — no persistent status bar or auto-dismissing error indicator on the writing surface — remains a product design choice.

#### Convergence: `ui::draw()` Bypassing Accessor Facade (3 lenses) *(RESOLVED)*

`DrawContext` struct fully decouples `draw()` from App internals.

#### Convergence: Feature Ghosts in Writing Window (3 lenses) *(RESOLVED)*

`--inline` removed. `ZANI_WINDOW` now serves as active re-spawn guard.

#### Convergence: Test-Code Correspondence Gaps (3 lenses) *(RESOLVED)*

`column_width` duplication eliminated (no sync to test). `alloc_bench` asserts all three modes. `autosave_refuses_when_load_error_set` tests the mutation directly. `hjkl` split into four tests.

## Stewardship Guide

### What to Protect

1. **The domain modeling discipline.** Named invariants, ADRs with amendment logs, a formal domain model — this practice is rare and valuable. Preserve it even as individual documents are corrected.

2. **WCAG AA as a hard invariant.** `Palette::validate()` enforced in tests is a model for how invariants should work. The approach — physiology research → design constraint → automated enforcement — is worth replicating.

3. **Vim bindings as a pure state machine.** `vim_bindings.rs` is the best-structured module: pure functions, no side effects, independently testable. It should serve as the template for future input handling.

4. **The version-keyed caching system.** `Buffer::version()` driving `RenderCache`, visual line cache, and sentence bounds cache is well-engineered. The invalidation key is correct and minimal.

5. **The extraction series approach.** One structural change per commit, no behavior changes, each obviously correct. This is the model for future structural improvements.

6. **`App::tick()` and `TickOutput`.** The recently extracted tick method consolidates per-frame state updates and gates draw via `Option<TickOutput>`. This is the right architectural direction — preserve this boundary.

7. **WritingSurface's explicit builder interface.** Despite the de-facto required arguments, the builder correctly insulates the renderer from App's internals.

8. **DrawContext + SettingsViewModel decoupling.** `draw()` now takes `&DrawContext`, completing the pattern started by SettingsViewModel. This boundary should be maintained — all new rendering state flows through DrawContext.

9. **Mtime-based external change detection.** The `Persistence` mtime tracking, auto-reload for clean buffers, and conflict bar for dirty buffers establish a solid file lifecycle model. The pattern of silently handling the common case (clean reload) while prompting for the ambiguous case (dirty conflict) is the right UX tradeoff.

10. **Scratch quit prompt lifecycle.** The Save/Rename/Discard flow for scratch buffers, including `pending_quit_after_rename` for deferred quit, content-based activation (not dirty-based), and standalone rename overlay, is a well-structured state machine that should serve as a template for future modal interactions.

11. **Coordinator invariant doc comment.** The decision rule on App — "Does this read/write state from only one subsystem? If yes, it belongs on that subsystem" — should be consulted when adding new methods to App.

### What to Improve (Prioritized)

*Most findings from prior audits are resolved. The domain-logic extraction series addressed App decomposition. Config round-trip tests already cover all fields. Remaining items:*

1. ~~**Fix remaining documentation drift**~~ — Resolved. README test count updated. Plexus/llm-orc and tmux entries were already marked as planned.

2. **Add user-visible save error indicator** — Save errors are currently only visible in the settings overlay. Consider a brief, auto-dismissing indicator on the writing surface when `save_error` is set. *(Finding: Silent Error Handling — product design decision)*

3. **Consolidate focus mode logic** — Focus dimming spans 4 modules (focus_mode.rs, dimming.rs, app.rs, writing_surface.rs). Push all opacity-computation logic into `DimmingState` so WritingSurface only applies pre-computed opacities. *(Finding: Focus Mode Distributed)*

4. **Address WritingSurface builder speculative generality** — 11 `Option<>` fields are always provided in production. Either collapse to required fields or add comments at each `None =>` fallback branch. *(Finding: WritingSurface Builder)*

5. **Strengthen integration test** — `focus_dimming_and_markdown_styling_compose` checks type shape, not color values. Either rename to match what it verifies, or extend to assert specific cell colors. *(Finding: Integration Test Composition Path)*

### Ongoing Practices

- **Add new rendering state to DrawContext.** The scratch quit overlay and conflict bar were built on DrawContext from the start. Future overlays/bars should follow this pattern.
- **Add new overlay animations as separate TransitionKind variants.** The `ScratchQuitOverlay` pattern — new variant + new accessor method + test for coexistence — is the template.
- **Move guards into the guarded function.** If a precondition matters, check it where the mutation happens, not in a separate "should I?" method. (Established by `autosave()` fix.)
- **Keep domain model current.** Mark unimplemented entries. Update when modules move or rename.
- **Isolate external dependencies in tests.** Use `#[cfg(not(test))]` to gate system calls (clipboard, filesystem) that make tests non-deterministic. (Established by `resolve_paste_text` fix.)
- **Make fields private as invariants are identified.** Each invariant can be sealed one at a time by narrowing visibility.
- **Assert what matters, not what's guaranteed.** Review new tests for assertions that verify type-system guarantees or test-setup values.
