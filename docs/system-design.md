# System Design: Zani

**Version:** 1.0
**Status:** Current
**Last amended:** 2026-03-06

## Architectural Drivers

| Driver | Type | Provenance |
|--------|------|------------|
| Sub-millisecond keystroke latency | Quality Attribute | Invariant 6; ADR-001 (Rust, zero-GC) |
| Graceful degradation across terminal color profiles | Quality Attribute | Invariant 11; ADR-012 |
| Hidden-by-default chrome | Quality Attribute | Invariant 1; ADR-010 |
| WCAG AA accessibility floor | Constraint | Invariant 3; ADR-006 |
| Rust + ratatui + crossterm + ropey | Constraint | ADR-001 |
| Single-user terminal application | Scale | Essay 001 |
| Immediate-mode rendering | Constraint | ADR-002 |
| Markdown as native format | Constraint | Invariant 7 |
| Dimming via color interpolation | Constraint | Invariant 4; ADR-004, ADR-008 |
| Scroll/Focus orthogonality | Constraint | Invariants 12-13; ADR-008 |

## Module Decomposition

### Module: palette
**Purpose:** Defines named, curated color systems with affective categorization, validation, and degradation support.
**Provenance:** Invariant 3; ADR-006; ADR-009; ADR-012; Essay 002 §Constraint Space, §Degradation
**Owns:** Palette, AffectiveCategory, PerceptualSortOrder, PaletteError, interpolation, validation
**Depends on:** (ratatui Color only)
**Depended on by:** app, config, color_profile, animation, writing_surface, ui, palette_browser

### Module: palette_browser
**Purpose:** State machine for browsing palettes by Affective Category with navigation, filtering, and selection.
**Provenance:** ADR-010; Essay 002 §Browsing; Reflection 002 §Palette Browser as Visual Navigation
**Owns:** PaletteBrowserState, Browse action state transitions
**Depends on:** palette
**Depended on by:** app, ui

### Module: config
**Purpose:** Persisted user preferences with local-then-global-then-default resolution.
**Provenance:** ADR-011; Essay 002 §Browsing; Invariant 5 (column width clamping)
**Owns:** Config, ConfigResolution, LocalConfig, Bind action (writing .zani.toml)
**Depends on:** palette, editing_mode, focus_mode, scroll_mode
**Depended on by:** app, main

### Module: color_profile
**Purpose:** Detects terminal color capability and maps colors to the available profile, using hand-tuned values when present.
**Provenance:** Invariant 11; ADR-012; Essay 002 §Degradation
**Owns:** ColorProfile, Detect action, Degrade action (hand-tuned check + automatic fallback), nearest_256_color
**Depends on:** palette (for hand-tuned alternate access)
**Depended on by:** app, writing_surface, ui

### Module: settings
**Purpose:** Settings Layer item definitions and overlay navigation state.
**Provenance:** Invariant 1; ADR-010; Essay 002 §Browsing
**Owns:** SettingsItem, SettingsState, RenameState, ScratchQuitState, Summon/Dismiss state transitions
**Depends on:** editing_mode, focus_mode, scroll_mode
**Depended on by:** app, ui

### Module: app
**Purpose:** Thin coordinator that owns subsystems and routes input between them.
**Provenance:** Invariant 2 (writing is the only default action); all ADRs (integration point)
**Owns:** Input dispatch, tick loop, config persistence, subsystem wiring
**Depends on:** editor, viewport, palette, palette_browser, dimming, color_profile, settings, persistence, animation, find, writing_surface, config
**Depended on by:** main, ui

### Module: editor
**Purpose:** Text editing core: buffer, cursor, undo, selection, and vim state.
**Provenance:** Invariant 7 (markdown native format); Invariant 10 (markdown always editable)
**Owns:** Editor, cursor management, Write action, undo/redo
**Depends on:** buffer, clipboard, editing_mode, smart_typography, undo, vim_bindings, wrap
**Depended on by:** app

### Module: dimming
**Purpose:** Opacity management for focus dimming with animated per-line chase values.
**Provenance:** Invariants 4, 14, 15; ADR-004; ADR-008
**Owns:** DimmingState, Dim action, paragraph/sentence opacity computation
**Depends on:** buffer, focus_mode, animation
**Depended on by:** app

### Module: animation
**Purpose:** Animation primitives: chase-to-target values and discrete transition management.
**Provenance:** Invariant 14 (chase-based animation); ADR-008
**Owns:** AnimatedValue, FadeConfig, AnimationManager, Transition, TransitionKind
**Depends on:** palette
**Depended on by:** app, dimming

### Module: writing_surface
**Purpose:** Custom text viewport rendering with per-character styling.
**Provenance:** ADR-002 (custom writing surface); Invariants 4, 5, 10
**Owns:** WritingSurface, RenderCache, Render action, Wrap action delegation
**Depends on:** buffer, color_profile, focus_mode, markdown_styling, palette, wrap
**Depended on by:** app, ui

### Module: ui
**Purpose:** Draw functions that compose subsystem state into terminal frames.
**Provenance:** Invariant 1 (tool disappears); all visual ADRs
**Owns:** DrawContext, draw(), overlay rendering (settings, find, conflict, scratch quit, rename, palette browser)
**Depends on:** app, buffer, color_profile, editing_mode, find, focus_mode, markdown_styling, palette, scroll_mode, settings, vim_bindings, wrap, writing_surface, palette_browser
**Depended on by:** main

### Module: viewport
**Purpose:** Scroll position management and visual line computation.
**Provenance:** Invariants 12-13; ADR-008
**Owns:** Viewport, Scroll action, visual line caching, column width management
**Depends on:** buffer, scroll_mode, wrap
**Depended on by:** app

### Module: persistence
**Purpose:** Autosave, file path management, and external change detection.
**Provenance:** Invariant 7 (markdown native format)
**Owns:** Persistence, Autosave action, scratch buffer management
**Depends on:** buffer
**Depended on by:** app

### Leaf Modules (no domain action, stable interfaces)

| Module | Purpose |
|--------|---------|
| buffer | In-memory rope text representation (Ropey wrapper) |
| focus_mode | FocusMode enum and sentence boundary parsing |
| scroll_mode | ScrollMode enum (Edge, Typewriter) |
| editing_mode | EditingMode enum (Vim, Standard) |
| vim_bindings | Vim modal key handling and action dispatch |
| markdown_styling | Per-character markdown style computation |
| smart_typography | ASCII to typographic character conversion |
| wrap | Soft-wrapping computation |
| clipboard | OSC52 clipboard integration |
| undo | Undo/redo operation history |
| find | Find overlay state and search logic |
| draft_name | Draft filename generation |
| writing_window | Terminal detection and dedicated window spawning |

## Responsibility Matrix

| Domain Concept/Action | Owning Module | Provenance |
|----------------------|---------------|------------|
| Document | persistence | Invariant 7 |
| Buffer | buffer | ADR-001 |
| Writing Surface | writing_surface | ADR-002 |
| App Shell | app | Invariant 2 |
| Focus Mode | focus_mode | ADR-004; ADR-008 |
| Active Region | focus_mode | ADR-004 |
| Dimming | dimming | Invariant 4; ADR-008 |
| Scroll Mode | scroll_mode | ADR-008; Invariant 12 |
| Typewriter Mode | viewport | Invariant 13 |
| Fade Config | animation | ADR-008 |
| **Palette** | **palette** | **ADR-006; ADR-009** |
| **Affective Category** | **palette** | **ADR-009; Essay 002 §Browsing** |
| **Perceptual Sort Order** | **palette** | **ADR-009; Essay 002 §Browsing** |
| **Color Profile** | **color_profile** | **Invariant 11; ADR-012** |
| Chrome | ui | Invariant 1 |
| **Settings Layer** | **settings** | **Invariant 1; ADR-010** |
| **Palette Browser** | **palette_browser** | **ADR-010; Essay 002 §Browsing** |
| **Local Config** | **config** | **ADR-011; Essay 002 §Browsing** |
| **Config Resolution** | **config** | **ADR-011** |
| Writing Window | writing_window | ADR-007; Invariant 9 |
| Inline Mode | main | ADR-007 |
| Markdown Styling | markdown_styling | ADR-005; Invariant 10 |
| Smart Typography | smart_typography | Essay 001 |
| Vim Bindings | vim_bindings | Essay 001 |
| Autosave | persistence | Invariant 7 |
| Write | editor | Invariant 2 |
| Focus (toggle) | app → dimming | ADR-004 |
| Dim | dimming | Invariant 4; ADR-008 |
| Launch | writing_window | ADR-007 |
| Render | writing_surface | ADR-002 |
| Wrap | writing_surface → wrap | ADR-002 |
| Scroll | viewport | ADR-008 |
| Autosave (action) | persistence | Invariant 7 |
| Summon | app → settings | Invariant 1 |
| Dismiss | settings | Invariant 1 |
| **Detect** | **color_profile** | **Invariant 11** |
| **Degrade** | **color_profile** | **Invariant 11; ADR-012** |
| **Browse** | **palette_browser** | **ADR-010** |
| **Bind** | **config** | **ADR-011** |
| **Resolve** | **config** | **ADR-011** |

Bold entries are new or changed for ADR-009 through ADR-012.

## Dependency Graph

### Directed Edges

```
main → app, config, color_profile, writing_window
app → editor, viewport, palette, palette_browser, dimming, color_profile, settings, persistence, animation, find, config
ui → app, writing_surface, palette, palette_browser, settings, color_profile, config, find, focus_mode, editing_mode, scroll_mode, vim_bindings, markdown_styling, buffer, wrap
writing_surface → buffer, palette, color_profile, focus_mode, markdown_styling, wrap
editor → buffer, clipboard, editing_mode, smart_typography, undo, vim_bindings, wrap
dimming → buffer, focus_mode, animation
animation → palette
viewport → buffer, scroll_mode, wrap
config → palette, editing_mode, focus_mode, scroll_mode
palette_browser → palette
color_profile → palette
persistence → buffer
find → buffer
```

### Layering Rules

1. **Leaf modules** (buffer, focus_mode, scroll_mode, editing_mode, vim_bindings, markdown_styling, smart_typography, wrap, clipboard, undo, draft_name) depend on nothing within zani.
2. **Domain modules** (palette, animation, dimming, viewport, persistence, find, writing_surface, config, settings, palette_browser, color_profile, editor) depend on leaf modules and each other in a DAG — no cycles.
3. **Coordinator** (app) depends on domain modules. Domain modules never import app.
4. **Presentation** (ui, main) depend on app and domain modules.
5. **palette** is foundational to the color system — depended on by animation, config, color_profile, writing_surface, palette_browser, ui, app.
6. **No cycles.** Verified: color_profile depends on palette (for hand-tuned access); palette does NOT depend on color_profile.

### New dependency: color_profile → palette

ADR-012 introduces a dependency from color_profile to palette: the Degrade action needs to check whether a Palette has hand-tuned 256-color values before falling back to automatic mapping. This is a read-only data dependency (color_profile reads palette overrides, does not modify palette state). This is the preferred direction — color_profile applies the profile to palette data, not the other way around.

## Integration Contracts

### app → palette_browser
**Protocol:** Direct method calls. App owns a `PaletteBrowserState` and routes key events to it when open.
**Shared types:** `PaletteBrowserState` (owned by app), `Palette` and `AffectiveCategory` (from palette module).
**Error handling:** Browser navigation cannot fail — all operations are in-memory state transitions.
**Owned by:** palette_browser defines the state machine; app orchestrates open/close and palette application.

### palette_browser → palette
**Protocol:** Read-only queries. Browser reads `Palette::all_by_category()` and `AffectiveCategory` variants.
**Shared types:** `Palette`, `AffectiveCategory`, `Vec<Palette>`.
**Error handling:** None — palette collection is static.
**Owned by:** palette defines the data; palette_browser consumes it.

### config → palette (extended)
**Protocol:** `Config::resolve_palette()` looks up palette by name from `Palette::all()`. Unchanged.
**Shared types:** `Palette`, palette name (String).
**Error handling:** Unknown palette name falls back to default. Unchanged.
**Owned by:** palette defines the collection; config resolves names to palettes.

### config (new: local resolution)
**Protocol:** `Config::load_for_path(path: &Path)` walks up from the given path, checking each ancestor for `.zani.toml`. If found, deserializes and merges with global config. `Config::bind_to_project(path: &Path, config: &Config)` writes a `.zani.toml`.
**Shared types:** `Config` (same struct, all fields `Option<>` in local variant for partial override).
**Error handling:** Walk-up filesystem errors (permission denied, symlink loops) are logged and skipped — resolution continues to the next ancestor. `.zani.toml` parse errors fall through to global config silently.
**Owned by:** config module.

### color_profile → palette (new dependency)
**Protocol:** `ColorProfile::degrade_palette(palette: &Palette) -> DegradedPalette` or `Palette::degraded_colors(profile: ColorProfile) -> [Color; N]`. The Degrade action checks `palette.color_256_overrides()` first; if `Some`, uses those; otherwise runs `nearest_256_color` on each color.
**Shared types:** `Palette`, `Color`, optional 256-color alternate values.
**Error handling:** If hand-tuned values fail validation, fall back to automatic mapping.
**Owned by:** palette defines the override data; color_profile implements the Degrade logic.

### settings (restructured)
**Protocol:** `SettingsItem::Palette` becomes a single variant (no index parameter) that signals "open browser". App catches this and opens PaletteBrowserState.
**Shared types:** `SettingsItem` (modified enum).
**Error handling:** None.
**Owned by:** settings defines the items; app interprets them.

## Fitness Criteria

| Criterion | Measure | Threshold | Derived From |
|-----------|---------|-----------|-------------|
| No module owns more than 8 glossary concepts/actions | Count responsibility matrix rows per module | ≤ 8 | Maintainability |
| Domain modules never import app | Grep for `use crate::app` in non-ui, non-main files | 0 matches | Layering rule |
| No dependency cycles | Topological sort of dependency graph | Acyclic | Architecture principle |
| Every Palette satisfies Invariant 3 | `palette.validate()` on all palettes including 256-color alternates | All pass | Invariant 3 |
| Config merge preserves unspecified fields | Test: local config with 1 field → all other fields from global | Pass | ADR-011 |
| Palette Browser is a sub-panel, not a separate overlay | Browser state is entered from Settings Layer, Esc returns to Settings | Pass | ADR-010; Invariant 1 |
| Color degradation checks hand-tuned before automatic | Test: palette with overrides uses them; palette without uses automatic | Pass | ADR-012 |
| Walk-up search terminates at filesystem root | Test: no `.zani.toml` anywhere → falls through to global | Pass | ADR-011 |

## Test Architecture

### Boundary Integration Tests

| Dependency Edge | Integration Test | What It Verifies |
|----------------|-----------------|------------------|
| palette_browser → palette | `browser_lists_palettes_by_category` | Real `Palette::all_by_category()` returns palettes grouped correctly; browser state iterates them |
| app → palette_browser | `settings_palette_row_opens_browser` | Selecting Palette row in settings opens `PaletteBrowserState`; Esc returns to settings |
| app → palette_browser | `browser_apply_triggers_crossfade` | Selecting a palette in browser starts a palette crossfade animation with real Palette types |
| config (local resolution) | `local_config_overrides_global` | `.zani.toml` with `palette = "Inkwell"` overrides global `palette = "Ember"` via real `Config::load_for_path()` |
| config (merge) | `partial_local_config_merges_with_global` | Local config specifying only `palette` inherits `column_width` and `focus_mode` from global config |
| config (bind) | `bind_writes_zani_toml` | `Config::bind_to_project()` writes a valid `.zani.toml` that `Config::load_for_path()` can read back |
| color_profile → palette | `degrade_uses_hand_tuned_values` | Palette with 256-color overrides: `Degrade` returns the hand-tuned colors, not automatic mapping |
| color_profile → palette | `degrade_falls_back_without_overrides` | Palette without 256-color overrides: `Degrade` returns `nearest_256_color` results |
| app → config | `file_open_resolves_local_config` | Opening a file in a project with `.zani.toml` resolves to the project's palette |

### Invariant Enforcement Tests

| Invariant | Enforcement Location | Test |
|-----------|---------------------|------|
| 1: Tool disappears | settings, ui | `default_state_has_no_chrome` (existing) |
| 3: WCAG AA, no pure B/W | palette | `all_palettes_satisfy_invariant_3` (existing, extended to cover 256-color alternates) |
| 5: Column prose-width | viewport, config | `column_width_clamped_on_deserialize` (existing) |
| 11: Graceful degradation | color_profile, palette | `degrade_uses_hand_tuned_values`, `degrade_falls_back_without_overrides` (new) |

### Test Layers

- **Unit:** Module-internal logic. Mocks are acceptable for cross-module dependencies. Examples: `PaletteBrowserState` navigation logic, `AffectiveCategory` sort ordering, config merge logic, `nearest_256_color` correctness.
- **Integration:** Real types flowing across module boundaries. No mocks at the boundary under test. Examples: browser reading real palette collection, config resolution reading real `.zani.toml` files, Degrade using real palette override data.
- **Acceptance:** End-to-end scenarios from `scenarios.md`. Full wiring through App. Examples: settings → browser → apply → crossfade, file open → config resolution → palette switch.

## Design Amendment Log

| # | Date | What Changed | Trigger | Provenance | Status |
|---|------|-------------|---------|------------|--------|
| — | — | — | — | — | — |
