# Field Guide: Zani

**Generated:** 2026-03-14
**Derived from:** System Design v1.0, current implementation

---

## How to Use This Guide

This guide maps the domain vocabulary used in design documents, ADRs, and code
comments to the actual types, fields, and functions that implement them. Each
module entry describes what it owns, where it lives, and what crosses its
boundaries. Use it when navigating unfamiliar code, reviewing a change, or
deciding where new logic belongs.

---

## Module: palette

**Implementation state:** Complete
**Code location:** `src/palette/mod.rs`, `src/palette/collection.rs`, `src/palette/color_math.rs`
**Stability:** Settled

### Domain Concepts in Code

| Concept | Code Manifestation | Location |
|---------|-------------------|----------|
| Palette | `struct Palette` — name, provenance, 7 color fields, category, sort_key, optional 256-color overrides | `palette/mod.rs:94` |
| Affective Category | `enum AffectiveCategory` — 8 variants: DarkWarm, DarkCool, DarkVivid, DarkMuted, LightWarm, LightCool, LightVivid, LightMuted | `palette/mod.rs:12` |
| Perceptual Sort Order | `Palette.sort_key: f64` — OKLCH hue angle of the background color | `palette/mod.rs:109` |
| Chroma Target | `AffectiveCategory::chroma_target() -> (f64, f64)` — ΔE2000 range (min, max) for backgrounds | `palette/mod.rs:60` |
| 256-color overrides | `struct Color256Overrides` — parallel set of 7 hand-tuned colors for 6×6×6 cube mapping | `palette/mod.rs:78` |
| Palette collection | `fn all_palettes() -> Vec<Palette>` — 40 palettes, 5 per Affective Category | `palette/collection.rs:16` |
| Default palette | Manzanita (Dark Warm); `Palette::default_palette()` | `palette/mod.rs:116` |
| Palette blend / crossfade | `Palette::blend(from, to, progress)` — linear RGB interpolation across all 7 color fields | `palette/mod.rs:129` |
| Color by name lookup | `Palette::by_name(name)` — returns default palette on miss | `palette/mod.rs:121` |
| Category grouping | `Palette::all_by_category()` — returns groups sorted by sort_key within each category | `palette/mod.rs:158` |
| Palette validation | `Palette::validate()` — enforces no pure black (#000000) or pure white (#FFFFFF) (Invariant 3) | `palette/mod.rs:186` |
| OKLCH hue | `fn oklch_hue(r, g, b) -> f64` — converts sRGB to OKLCH hue angle in degrees [0, 360) | `palette/color_math.rs:15` |
| CIELAB conversion | `fn srgb_to_lab(r, g, b) -> (L*, a*, b*)` — sRGB to CIE L*a*b* under D65 | `palette/color_math.rs:39` |
| CIEDE2000 | `fn delta_e_2000(l1,a1,b1, l2,a2,b2) -> f64` — perceptual color difference per ISO 11664-6 | `palette/color_math.rs:77` |
| Color interpolation | `fn interpolate(a, b, t)` (pub(super)) — used by `Palette::blend` | `palette/mod.rs` (private impl) |
| PNW species naming | `Palette.name: &'static str` and `Palette.provenance: &'static str` — botanical name + ecological note | `palette/mod.rs:95-98` |

### Design Rationale

Palettes are mood instruments rather than color themes. Each palette primes an
affective state — the eight-category taxonomy (Dark/Light × Warm/Cool/Vivid/Muted)
makes that intent explicit and navigable. All 40 palettes are named after
Cascadia bioregion flora (ADR-015, ADR-016) with botanically accurate provenance
strings, encoding the place-based character of the project.

The constraint that backgrounds must fall within each category's ΔE2000 chroma
target (Invariant 18) is enforced at design time via the `chroma_target()` lookup,
not at runtime. WCAG AA contrast (4.5:1) for accents against backgrounds is an
author-enforced invariant across all 40 palettes.

Color science lives in `color_math.rs` (pure math, no ratatui dependency) and is
kept separate from palette data in `collection.rs` so the data file can be read
and reviewed without decoding the math.

### Key Integration Points

- `color_profile` — `ColorProfile::degrade_palette()` reads `color_256_overrides()` to produce a degraded palette for 256-color terminals
- `animation` — `AnimationManager` and `Palette::blend()` drive crossfade transitions
- `markdown_styling` — `CharStyle::resolve()` reads all 7 palette color fields to produce per-character styles
- `config` — `Config.palette: String` names the active palette; resolved via `Palette::by_name()`
- `palette_browser` — browses via `Palette::all_by_category()`

---

## Module: palette_browser

**Implementation state:** Complete
**Code location:** `src/palette_browser.rs`
**Stability:** Settled

### Domain Concepts in Code

| Concept | Code Manifestation | Location |
|---------|-------------------|----------|
| Browser state machine | `struct PaletteBrowserState` — open flag, groups, category_idx, palette_idx, active_palette | `palette_browser.rs:6` |
| Category navigation | `category_idx: usize` into `groups: Vec<(AffectiveCategory, Vec<Palette>)>` | `palette_browser.rs:9-11` |
| Palette navigation | `palette_idx: usize` within the focused category | `palette_browser.rs:12` |
| Active palette marker | `active_palette: String` — name of the currently applied palette for marking in UI | `palette_browser.rs:15` |
| Open action | `PaletteBrowserState::open(active_palette_name)` — populates groups, positions cursor on current palette | `palette_browser.rs:30` |
| Close action | `PaletteBrowserState::close()` | `palette_browser.rs:52` |
| Focused palette | `focused_palette() -> Option<Palette>` — the palette the cursor is on | `palette_browser.rs:57` |

### Design Rationale

The palette browser is a state machine owned by `app` and rendered by `ui`.
It is entered from the Settings Layer's Palette row and exits on Enter (apply)
or Escape (cancel). Navigation is two-dimensional: left/right steps through
categories, up/down steps through palettes within a category. The affective
taxonomy is the primary navigation axis, not alphabetical order.

### Key Integration Points

- `palette` — groups and navigation data come from `Palette::all_by_category()`
- `app` — owns `PaletteBrowserState`; routes input to it when open; extracts `focused_palette()` for live preview
- `ui` — `draw_palette_browser()` reads `PaletteBrowserState` fields directly

---

## Module: config

**Implementation state:** Complete
**Code location:** `src/config.rs`
**Stability:** Settled

### Domain Concepts in Code

| Concept | Code Manifestation | Location |
|---------|-------------------|----------|
| Config | `struct Config` — palette name, focus_mode, column_width (20–120), editing_mode, scroll_mode | `config.rs:35` |
| Config resolution | `enum ConfigSource` — Default, Global, Local | `config.rs:12` |
| Local config | `struct LocalConfig` — all-Optional fields from `.zani.toml` | `config.rs:25` |
| Global config path | `~/.config/zani/config.toml` via `Config::path()` | `config.rs:80` |
| Local config discovery | `Config::load_for_path()` — walks up from file's directory looking for `.zani.toml` | `config.rs:102` |
| Column width clamping | `column_width.clamp(20, 120)` enforced in `Config::load()` (Invariant 5) | `config.rs:93` |
| Local override merge | `Config::merge_local()` — overwrites only the fields present in LocalConfig | `config.rs:134` |
| Default palette | `default_palette_name()` returns `"Manzanita"` | `config.rs:53` |
| Default column width | `default_column_width()` returns `60` | `config.rs:57` |

### Design Rationale

Config follows a local-then-global-then-default resolution hierarchy (ADR-011).
All fields are serde-serialized as TOML. The column width invariant (20–120 chars)
is clamped on load, not on set, to keep the setting round-trippable. Local config
(`.zani.toml`) stores only overrides — unset fields fall through to global config,
not to defaults — so project configs stay minimal.

The `ConfigSource` enum is threaded through `App` so the Settings Layer UI can
display "project" vs "global" for the Config row item (ADR-013).

### Key Integration Points

- `app` — creates `App::from_config_with_source()`, calls `save_config_on_quit()`
- `main` — top-level config load and `ConfigSource` propagation
- `palette` — `Config::resolve_palette()` calls `Palette::by_name()`
- `settings` — `SettingsItem::Config` row reflects `ConfigSource`

---

## Module: color_profile

**Implementation state:** Complete
**Code location:** `src/color_profile.rs`
**Stability:** Settled

### Domain Concepts in Code

| Concept | Code Manifestation | Location |
|---------|-------------------|----------|
| Color capability | `enum ColorProfile` — TrueColor, Color256, Basic | `color_profile.rs:6` |
| Capability detection | `ColorProfile::detect()` — reads `COLORTERM`, `TERM` env vars | `color_profile.rs:34` |
| Color mapping | `ColorProfile::map_color(color)` — passthrough for TrueColor, `nearest_256_color()` for Color256 | `color_profile.rs:19` |
| Palette degradation | `ColorProfile::degrade_palette(palette)` — applies hand-tuned 256-color overrides if present, else passthrough | `color_profile.rs:45` |
| 256-color mapping | `fn nearest_256_color(r, g, b) -> u8` — maps RGB to nearest xterm-256 index | `color_profile.rs` (private fn) |

### Design Rationale

Terminal color capability is detected once at startup and threaded through the
render pipeline (Invariant 11, ADR-012). Degradation happens at two levels:
palette-level (hand-tuned 256-color overrides replace the full palette before any
rendering occurs) and per-color (individual colors go through `map_color` during
rendering). This two-level strategy means curators can tune 256-color appearance
per-palette while the automatic fallback still works for palettes without overrides.

### Key Integration Points

- `main` — calls `ColorProfile::detect()` before constructing `App`
- `app` — stores `ColorProfile`, passes it to `writing_surface` and `ui`
- `palette` — `degrade_palette()` calls `palette.color_256_overrides()`
- `writing_surface` — uses `ColorProfile::map_color()` per character during render

---

## Module: settings

**Implementation state:** Complete
**Code location:** `src/settings.rs`
**Stability:** Settled

### Domain Concepts in Code

| Concept | Code Manifestation | Location |
|---------|-------------------|----------|
| Settings item | `enum SettingsItem` — EditingMode(EditingMode), Palette, FocusMode(FocusMode), ScrollMode(ScrollMode), ColumnWidth, File, Config | `settings.rs:10` |
| Item ordering | `ALL_ITEMS: [SettingsItem; 11]` — fixed display order | `settings.rs:27` |
| Settings layer state | `struct SettingsState` — visible flag, cursor position | `settings.rs:54` |
| Rename state | `struct RenameState` — active flag, input buffer string, cursor position | `settings.rs` |
| Scratch quit state | `struct ScratchQuitState` — active flag, selected action | `settings.rs` |
| Scratch quit action | `enum ScratchQuitAction` — Save, Discard, Cancel | `settings.rs` |
| Summon / dismiss | `SettingsState::visible` toggled via `App::handle_key` routing | `app.rs` |

### Design Rationale

The Settings Layer is a summonable overlay — it disappears by default (Invariant 1).
`SettingsItem` replaces magic indices with typed variants so the Settings Layer
cursor position has semantic meaning. Each item variant carries the mode it
represents (e.g., `SettingsItem::FocusMode(FocusMode::Sentence)`) so the UI can
render it correctly without a separate mapping table.

`RenameState` and `ScratchQuitState` are co-located here because they are
sub-states of the Settings Layer interaction, not independent features.

### Key Integration Points

- `app` — owns `SettingsState`, `RenameState`, `ScratchQuitState`; routes key events to them
- `ui` — `draw_settings_layer()` reads `SettingsState` and `SettingsItem::all()`
- `editing_mode`, `focus_mode`, `scroll_mode` — enum values carried as SettingsItem variants

---

## Module: app

**Implementation state:** Complete
**Code location:** `src/app.rs`
**Stability:** In flux (active integration point)

### Domain Concepts in Code

| Concept | Code Manifestation | Location |
|---------|-------------------|----------|
| App shell | `struct App` — owns all subsystems; no domain logic | `app.rs:43` |
| Per-frame output | `struct TickOutput` — visual_lines, sentence_bounds; produced by `App::tick()`, consumed by `ui::draw()` | `app.rs:28` |
| Subsystem wiring | `App::from_config_with_source()` — constructs and connects all subsystems | `app.rs` |
| Input dispatch | `App::handle_key(code, modifiers)` — routes to active overlay or editor | `app.rs` |
| Tick loop | `App::tick(width, height) -> Option<TickOutput>` — advances animations, rebuilds render cache, returns frame data | `app.rs` |
| Animation gate | `App::any_animation_active() -> bool` — controls poll timeout (16ms vs 250ms) | `app.rs` |
| Autosave trigger | `App::should_autosave()`, `App::autosave()` — delegates to `Persistence` | `app.rs` |
| External change detection | `App::check_external_change()` — delegates to `Persistence`; sets `external_change_pending` | `app.rs` |
| Config persistence | `App::save_config_on_quit()` — writes updated config to disk on exit | `app.rs` |
| Local config path | `App.local_config_path: Option<PathBuf>` — path to found `.zani.toml`, for write-back | `app.rs:53` |
| Redraw flag | `App.needs_redraw: bool` — set on resize, forces full re-render next tick | `app.rs:62` |
| Cursor shape | `App::cursor_shape()` — delegates to `Editor::cursor_shape()` | `app.rs` |
| Quit coordination | `App.should_quit`, `App::should_quit()` — checked in the main event loop | `app.rs:54` |
| Scratch quit flow | `App.scratch_quit: ScratchQuitState`, `App.pending_quit_after_rename: bool` | `app.rs:65-67` |

### Design Rationale

App is a coordinator, not an owner of domain logic (see coordinator invariant in
the struct doc comment). When adding a method here, the test is: "Does this read
or write state from only one subsystem?" If yes, it belongs on that subsystem
instead. This discipline keeps `app.rs` from accumulating business logic.

`TickOutput` is the seam between `app` and `ui` — App computes what the frame
needs during `tick()`, and `ui::draw()` consumes it without touching App's
internals directly beyond the `DrawContext`.

### Key Integration Points

- `main` — constructs App, calls `tick()` and `handle_key()` in the event loop
- `ui` — constructs `DrawContext` from App's public fields and `TickOutput`
- All subsystems — `App` owns them as fields and delegates to them

---

## Module: editor

**Implementation state:** Complete
**Code location:** `src/editor.rs`
**Stability:** Settled

### Domain Concepts in Code

| Concept | Code Manifestation | Location |
|---------|-------------------|----------|
| Editor | `struct Editor` — buffer, cursor_line, cursor_col, vim_mode, editing_mode, selection, yank_register, undo_history, dirty flag | `editor.rs:19` |
| Cursor position | `Editor.cursor_line: usize`, `Editor.cursor_col: usize` — logical line and column | `editor.rs:21-22` |
| Vim mode | `Editor.vim_mode: Mode` — Normal, Insert, Visual | `editor.rs:23` |
| Editing mode | `Editor.editing_mode: EditingMode` — Vim or Standard | `editor.rs:24` |
| Selection anchor | `Editor.selection_anchor: Option<(usize, usize)>` — set in Visual mode | `editor.rs:26` |
| Yank register | `Editor.yank_register: Option<String>` — vim yank/paste buffer | `editor.rs:27` |
| Dirty flag | `Editor.dirty: bool` — true when buffer has unsaved changes | `editor.rs:29` |
| Paragraph bounds cache | `ParagraphBoundsCache` — (buffer_version, cursor_line) keyed cache | `editor.rs:13` |
| Cursor shape | `Editor::cursor_shape() -> CursorShape` — Bar for Insert/Standard, Block for Normal/Visual | `editor.rs:57` |

### Design Rationale

Editor owns the text editing concerns: buffer, cursor, undo, selection, and vim
state. It deliberately does not own the viewport (scroll position) — that belongs
to `Viewport`. The dirty flag lives on Editor because it is a property of whether
the buffer has been written since last save, which is an editor-level concern.

Smart typography transformations are applied in Editor at insertion time by
delegating to `smart_typography::transform()`.

### Key Integration Points

- `buffer` — `Editor.buffer: Buffer` is the actual text storage
- `vim_bindings` — `Action` enum returned by key dispatch; `Mode`, `Direction`, `CursorShape` types
- `undo` — `Editor.undo_history: UndoHistory`; operations recorded and replayed here
- `clipboard` — `Editor` calls `clipboard::read_clipboard()` for paste
- `wrap` — `Editor` delegates wrap queries for visual-line cursor movement
- `app` — `App` owns `Editor` and delegates all text input to it

---

## Module: dimming

**Implementation state:** Complete
**Code location:** `src/dimming.rs`
**Stability:** Settled

### Domain Concepts in Code

| Concept | Code Manifestation | Location |
|---------|-------------------|----------|
| Dimming state | `struct DimmingState` — focus_mode, paragraph_dim, sentence fades, output buffers | `dimming.rs:18` |
| Paragraph dim layer | `DimmingState.paragraph_dim: DimLayer` — N animated values, one per logical line | `dimming.rs:20` |
| Sentence fades | `sentence_fades: Vec<(usize, usize, LineOpacity)>` — (start, end) char range + animated opacity | `dimming.rs:22` |
| Sentence bounds cache | `SentenceBoundsCache` — (buffer_version, cursor_pos) keyed | `dimming.rs:8` |
| Settled flag | `DimmingState.settled: bool` — true when no animations are running and inputs haven't changed | `dimming.rs:31` |
| Output buffers | `line_opacities_buf`, `sentence_fades_buf` — pre-allocated, reused each frame | `dimming.rs:26-27` |
| Opacity constants | `OPACITY_NEAR` (0.6), `OPACITY_MID` (0.35), `OPACITY_FAR` (0.2) — paragraph distance bands | `focus_mode.rs:10-14` |
| Apply dimming | `focus_mode::apply_dimming_with_opacity(base_fg, palette, opacity)` — lerps fg toward bg | `focus_mode.rs:31` |

### Design Rationale

Dimming is separated from `focus_mode` (which defines the logic of which text is
active) because the animated state — the per-line chase values, the settled
detection, the output buffer reuse — is a runtime concern that doesn't belong in
a pure enum module. Data flows one way: buffer content and cursor position are
inputs; dimming state is output.

The settled flag enables an early-return optimization: when focus mode is off and
no animations are running, `DimmingState::update()` skips all computation.

FadeConfig for paragraph transitions:
- Brighten (cursor enters paragraph): 150ms EaseOut
- Dim (cursor leaves paragraph): 1800ms EaseOut

### Key Integration Points

- `focus_mode` — `FocusMode` enum, `DimLayer`, `LineOpacity` type alias, opacity constants, `sentence_bounds_in_buffer()`
- `animation` — `AnimatedValue` (via `LineOpacity`), `FadeConfig`, `Easing`
- `buffer` — buffer content and version used as cache keys
- `app` — owns `DimmingState`; passes line opacities to `WritingSurface` via `TickOutput`

---

## Module: animation

**Implementation state:** Complete
**Code location:** `src/animation.rs`
**Stability:** Settled

### Domain Concepts in Code

| Concept | Code Manifestation | Location |
|---------|-------------------|----------|
| Animated value | `struct AnimatedValue` — target, start_value, start_time, fade_config; chase-to-target semantics | `animation.rs:47` |
| Fade config | `struct FadeConfig` — duration + easing; configures an `AnimatedValue` | `animation.rs:29` |
| Easing | `enum Easing` — EaseOut, EaseInOut; `fn ease_out()`, `fn ease_in_out()` | `animation.rs:110` |
| Animation manager | `struct AnimationManager` — owns 0–1 active `Transition` per `TransitionKind` | `animation.rs` |
| Transition | `struct Transition` / `enum TransitionKind` — typed wrapper around an `AnimatedValue` | `animation.rs:138` |
| TransitionKind | `Palette { from, to }`, `OverlayFade` — the two discrete transition types | `animation.rs:138` |
| DimLayer | `struct DimLayer` (in `focus_mode.rs`) — N `AnimatedValue`s, one per line; fade-in and fade-out configs | `focus_mode.rs` |
| Chase semantics | `AnimatedValue::set_target()` captures current visual value as start — no visual discontinuity when interrupted (Invariant 14) | `animation.rs:69` |

### Design Rationale

The module comment explains the decision rule: `AnimationManager` for global,
discrete transitions that run once and are pruned (palette crossfades, overlay
fade-in); `AnimatedValue`/`DimLayer` for per-line persistent values (dimming) that
pre-allocate and zero-alloc at steady state.

Chase-to-target semantics guarantee no visual discontinuity: if a transition is
interrupted mid-flight, the new animation starts from the current visual value
(not the original start), so it never jumps.

### Key Integration Points

- `dimming` — `DimLayer` and `LineOpacity` (= `AnimatedValue`) live in `focus_mode.rs` but build on `AnimatedValue`
- `app` — owns `AnimationManager`; calls `any_animation_active()` to drive 60fps poll
- `palette` — `Palette::blend()` is the interpolation primitive used by `AnimationManager` for crossfades

---

## Module: writing_surface

**Implementation state:** Complete
**Code location:** `src/writing_surface.rs`
**Stability:** In flux (rendering details evolve)

### Domain Concepts in Code

| Concept | Code Manifestation | Location |
|---------|-------------------|----------|
| Writing surface | `struct WritingSurface` — builder pattern; implements `ratatui::widgets::Widget` | `writing_surface.rs` |
| Render cache | `struct RenderCache` — per-logical-line code_block_state, line_char_offsets, md_styles, line_texts, line_chars | `writing_surface.rs:35` |
| Cache versioning | `RenderCache.version: u64` — compared against `buffer.version()` to detect staleness | `writing_surface.rs:37` |
| Fence detection | `fn is_fence_from_rope_slice(slice)` — zero-allocation code block delimiter check | `writing_surface.rs:13` |
| Code block state | `RenderCache.code_block_state: Vec<bool>` — per-line flag, true if inside a fenced block | `writing_surface.rs:38` |
| Markdown styles | `RenderCache.md_styles: Vec<Vec<CharStyle>>` — per-character style for every line | `writing_surface.rs:39` |
| Center offset | `WritingSurface::center_offset(width)` — horizontal centering given terminal width | `writing_surface.rs` |
| Cursor visual position | `WritingSurface::cursor_visual_position(visual_lines)` — maps logical (line, col) to visual line index | `writing_surface.rs` |

### Design Rationale

`WritingSurface` is the custom text viewport (ADR-002). It receives pre-computed
data from `RenderCache` and per-frame state from `App` (scroll offset, cursor,
focus mode, line opacities, find match ranges, selection). Per-character styling
is computed once per buffer version in `RenderCache::refresh()` and reused across
frames.

`RenderCache` reuses `Vec` capacity — inner vecs are cleared and extended, not
reallocated — to achieve zero steady-state heap allocation during normal writing.

### Key Integration Points

- `buffer` — `RenderCache::refresh()` iterates buffer lines; version used for cache invalidation
- `markdown_styling` — `style_line_with_context()` called per line during cache refresh
- `color_profile` — `map_color()` applied to each character's foreground and background
- `palette` — `CharStyle::resolve()` called with the effective palette
- `wrap` — visual line data consumed by the surface renderer
- `focus_mode` — `apply_dimming_with_opacity()` applied per character using line opacities
- `app` — owns `RenderCache`; `ui` constructs `WritingSurface` from it

---

## Module: ui

**Implementation state:** Complete
**Code location:** `src/ui.rs`
**Stability:** In flux (overlay rendering evolves)

### Domain Concepts in Code

| Concept | Code Manifestation | Location |
|---------|-------------------|----------|
| Draw entry point | `fn draw(frame, ctx)` — composes all layers: surface, settings, palette browser, find bar, conflict bar, scratch quit, rename | `ui.rs:21` |
| Draw context | `struct DrawContext` — assembled from `App` fields and `TickOutput`; all draw functions receive it | `ui.rs` |
| Settings layer draw | `fn draw_settings_layer(frame, vm, palette, area)` | `ui.rs` |
| Palette browser draw | `fn draw_palette_browser(frame, ctx, area)` | `ui.rs` |
| Find bar draw | `fn draw_find_bar(frame, fs, palette, area, opacity)` | `ui.rs:113` |
| Conflict bar draw | `fn draw_conflict_bar(frame, palette, area)` | `ui.rs` |
| Scratch quit draw | `fn draw_scratch_quit_overlay(...)` | `ui.rs` |
| Rename draw | `fn draw_rename_overlay(frame, palette, area, buf, cursor)` | `ui.rs` |
| Chrome-free default | `surface_area = area` — writing surface occupies the full terminal (Invariant 1) | `ui.rs:28` |
| Cursor placement | `frame.set_cursor_position()` — positioned after render based on visual line computation | `ui.rs:102` |

### Design Rationale

`ui` contains only draw functions. It has no state of its own. `DrawContext`
assembles a flat view of what the UI needs from `App` — this keeps draw functions
pure and avoids App coupling in rendering logic.

The layering order is: Writing Surface → Settings Layer → Palette Browser →
Find Bar → Conflict Bar → Scratch Quit → Rename overlay. Each layer draws on top
of the previous; overlays use `ratatui::widgets::Clear` before drawing.

### Key Integration Points

- `app` — `DrawContext::new(app, visual_lines, sentence_bounds)` reads app fields
- `writing_surface` — `WritingSurface` widget constructed and rendered here
- `palette_browser` — `draw_palette_browser` reads `PaletteBrowserState`
- `settings` — `draw_settings_layer` reads `SettingsState` and `SettingsItem::all()`
- All other display modules — consumed read-only through `DrawContext`

---

## Module: viewport

**Implementation state:** Complete
**Code location:** `src/viewport.rs`
**Stability:** Settled

### Domain Concepts in Code

| Concept | Code Manifestation | Location |
|---------|-------------------|----------|
| Viewport | `struct Viewport` — scroll_offset, typewriter_vertical_offset, scroll_mode, column_width, effective_column_width, visual_lines cache | `viewport.rs:8` |
| Scroll offset | `Viewport.scroll_offset: usize` — first visible visual line index | `viewport.rs:9` |
| Typewriter vertical offset | `Viewport.typewriter_vertical_offset: u16` — padding above cursor when buffer is short (Invariant 13) | `viewport.rs:10` |
| Effective column width | `Viewport.effective_column_width: u16` — column_width clamped to terminal width; set per tick | `viewport.rs:15` |
| Visual lines cache | `visual_lines_cache: Option<(buffer_version, column_width, Rc<[VisualLine]>)>` | `viewport.rs:17` |
| Cursor visibility | `Viewport::ensure_cursor_visible()` — adjusts scroll_offset for Edge or Typewriter mode | `viewport.rs:55` |
| Column width adjustment | `Viewport::adjust_column_width(delta)` — clamped to 20–120 (Invariant 5) | `viewport.rs:103` |
| Visual lines computation | `Viewport::visual_lines(buffer) -> Rc<[VisualLine]>` — cached; O(1) on hit | `viewport.rs:41` |

### Design Rationale

Viewport owns the question of which part of the document is visible and at what
column width. It does not own the text or the cursor position — those belong to
Editor. The visual lines cache uses `Rc` for cheap sharing across `TickOutput` and
the draw call without copying.

Typewriter mode (Invariant 13) keeps the cursor centered: when `scroll_mode` is
`Typewriter`, `ensure_cursor_visible` sets `scroll_offset = cursor_vl - center` and
falls back to `typewriter_vertical_offset` padding for buffers shorter than the
screen height.

### Key Integration Points

- `buffer` — version field used as cache key
- `wrap` — `visual_lines_for_buffer()` called on cache miss
- `scroll_mode` — `ScrollMode` enum determines Edge vs Typewriter behavior
- `app` — owns `Viewport`; sets `effective_column_width` each tick; passes scroll data to `ui`

---

## Module: persistence

**Implementation state:** Complete
**Code location:** `src/persistence.rs`
**Stability:** Settled

### Domain Concepts in Code

| Concept | Code Manifestation | Location |
|---------|-------------------|----------|
| Persistence | `struct Persistence` — file_path, is_scratch, save_error, load_error, last_save, autosave_interval, last_known_mtime | `persistence.rs:8` |
| Scratch buffer | `Persistence.is_scratch: bool` — true when no file was specified at launch | `persistence.rs:11` |
| Autosave interval | `autosave_interval: Duration` — 3 seconds default | `persistence.rs:14` |
| Autosave trigger | `Persistence::should_autosave(dirty)` — guards on dirty flag, elapsed time, and no load error | `persistence.rs:34` |
| Autosave action | `Persistence::autosave(buffer, dirty)` — writes to `file_path`, records mtime | `persistence.rs:45` |
| External change detection | `last_known_mtime: Option<SystemTime>` — compared on poll; drives conflict bar | `persistence.rs:15` |
| Draft filename | `draft_name::generate()` — called by persistence when naming a scratch buffer | `draft_name.rs` |

### Design Rationale

Persistence owns file identity and I/O. It is not responsible for serialization
format — files are always plain UTF-8 markdown (Invariant 7). Autosave is
deliberately conservative: it refuses to save when a `load_error` is set (which
would overwrite a file that was only partially loaded).

The mtime-based external change detection is intentionally lightweight — it compares
`SystemTime` from `fs::metadata`, which is sufficient for detecting race conditions
between zani and external editors.

### Key Integration Points

- `buffer` — receives `&Buffer` for content to write; `dirty` flag lives on `Editor`
- `draft_name` — `generate()` supplies the default filename for scratch buffers
- `app` — owns `Persistence`; calls `should_autosave()` and `autosave()` in the event loop

---

## Module: buffer

**Implementation state:** Complete
**Code location:** `src/buffer.rs`
**Stability:** Settled (stable API)

### Domain Concepts in Code

| Concept | Code Manifestation | Location |
|---------|-------------------|----------|
| Buffer | `struct Buffer` — wraps `ropey::Rope` + monotonic `version: u64` | `buffer.rs:8` |
| Version | `Buffer.version: u64` — incremented on every insert or remove; used as cache key throughout | `buffer.rs:11` |
| Construction | `Buffer::new()`, `Buffer::from_text(text)` | `buffer.rs:29,37` |
| Mutation | `Buffer::insert(char_idx, text)`, `Buffer::remove(start, end)` — both increment version | `buffer.rs:94,100` |
| Line access | `Buffer::line(idx) -> RopeSlice`, `Buffer::len_lines()` | `buffer.rs:89,84` |
| Char access | `Buffer::char_at(idx)`, `Buffer::chars_at(idx)`, `Buffer::len_chars()` | `buffer.rs:69,64,74` |
| Index conversion | `Buffer::line_to_char(line_idx)`, `Buffer::char_to_line(char_idx)` | `buffer.rs:54,60` |
| Content check | `Buffer::has_content()` — true if any non-whitespace chars | `buffer.rs:79` |

### Design Rationale

Buffer is a thin, stable wrapper around `ropey::Rope` that adds version tracking.
The version field is the central cache invalidation primitive for the entire render
pipeline: `Viewport`, `RenderCache`, and `DimmingState` all key their caches on
`buffer.version()`. All mutation routes through `insert` and `remove` to guarantee
the version is always current.

### Key Integration Points

- `ropey` crate — underlying rope data structure; O(log n) insert/remove/access
- Used by nearly every module as the source of truth for document text

---

## Leaf Modules

These modules have no domain actions of their own and expose stable, simple
interfaces. They are dependencies of the modules above.

---

### Module: focus_mode

**Code location:** `src/focus_mode.rs`
**Stability:** Settled

| Concept | Code Manifestation | Location |
|---------|-------------------|----------|
| Focus mode | `enum FocusMode` — Off, Sentence, Paragraph | `focus_mode.rs:18` |
| LineOpacity | `type LineOpacity = AnimatedValue` — per-line animated opacity value | `focus_mode.rs:7` |
| DimLayer | `struct DimLayer` — N `LineOpacity` values, fade-in/fade-out `FadeConfig`; lives here, used by `dimming` | `focus_mode.rs` |
| Sentence boundaries | `fn sentence_bounds_in_buffer(buffer, cursor_idx) -> Option<(usize, usize)>` — char-index range | `focus_mode.rs:43` |
| Opacity application | `fn apply_dimming_with_opacity(base_fg, palette, opacity)` — lerps toward background | `focus_mode.rs:31` |
| Paragraph targets | `fn fill_paragraph_target_opacities(...)` — fills buffer with NEAR/MID/FAR/1.0 targets | `focus_mode.rs` |

---

### Module: scroll_mode

**Code location:** `src/scroll_mode.rs`
**Stability:** Settled

| Concept | Code Manifestation | Location |
|---------|-------------------|----------|
| Scroll mode | `enum ScrollMode` — Edge (default), Typewriter | `scroll_mode.rs:3` |

---

### Module: editing_mode

**Code location:** `src/editing_mode.rs`
**Stability:** Settled

| Concept | Code Manifestation | Location |
|---------|-------------------|----------|
| Editing mode | `enum EditingMode` — Vim (default), Standard | `editing_mode.rs:6` |

---

### Module: vim_bindings

**Code location:** `src/vim_bindings.rs`
**Stability:** Settled

| Concept | Code Manifestation | Location |
|---------|-------------------|----------|
| Vim mode | `enum Mode` — Normal, Insert, Visual | `vim_bindings.rs:3` |
| Cursor shape | `enum CursorShape` — Bar (Insert), Block (Normal/Visual) | `vim_bindings.rs:11` |
| Action | `enum Action` — SwitchMode, InsertChar, InsertNewline, DeleteBack, DeleteChar, MoveCursor, WordForward, WordBackward, DeleteLine, Yank, Paste, Undo, Redo, etc. | `vim_bindings.rs:30` |
| Direction | `enum Direction` — Up, Down, Left, Right | `vim_bindings.rs` |
| Key dispatch | `fn process_key(mode, key, modifiers) -> Option<Action>` | `vim_bindings.rs` |

---

### Module: markdown_styling

**Code location:** `src/markdown_styling.rs`
**Stability:** Settled

| Concept | Code Manifestation | Location |
|---------|-------------------|----------|
| Char style | `struct CharStyle` — is_syntax, modifier (Modifier), is_heading, is_code, is_link_text | `markdown_styling.rs:7` |
| Style resolution | `CharStyle::resolve(palette) -> Style` — maps is_* flags to palette accent fields | `markdown_styling.rs:34` |
| Line styling | `fn style_line_with_context(line, in_code_block) -> Vec<CharStyle>` | `markdown_styling.rs` |
| Fence detection | `fn is_fence_line(line) -> bool` | `markdown_styling.rs:57` |
| accent_emphasis mapping | Bold and italic modifier bits map to `palette.accent_emphasis` in `CharStyle::resolve()` | `markdown_styling.rs:43` |

---

### Module: wrap

**Code location:** `src/wrap.rs`
**Stability:** Settled

| Concept | Code Manifestation | Location |
|---------|-------------------|----------|
| Visual line | `struct VisualLine` — logical_line, char_start, char_end | `wrap.rs:4` |
| Line wrapping | `fn wrap_line(text, width, logical_line) -> Vec<VisualLine>` — word-boundary soft wrap | `wrap.rs:16` |
| Buffer wrapping | `fn visual_lines_for_buffer(buffer, width) -> Vec<VisualLine>` | `wrap.rs` |

---

### Module: smart_typography

**Code location:** `src/smart_typography.rs`
**Stability:** Settled

| Concept | Code Manifestation | Location |
|---------|-------------------|----------|
| Smart edit | `struct SmartEdit` — delete_before count, insert string | `smart_typography.rs:18` |
| Transform | `fn transform(ch, preceding) -> Option<SmartEdit>` — converts `"`, `'`, `-`, `.` at insertion time | `smart_typography.rs:6` |
| Rules | Double quotes → " ", single quotes → ' ', `--` → —, `...` → … | `smart_typography.rs` |

---

### Module: clipboard

**Code location:** `src/clipboard.rs`
**Stability:** Settled

| Concept | Code Manifestation | Location |
|---------|-------------------|----------|
| Clipboard read | `fn read_clipboard() -> Option<String>` — dispatches to pbpaste / wl-paste / xclip | `clipboard.rs:29` |
| Clipboard write | `fn write_clipboard(text)` — dispatches to pbcopy / wl-copy / xclip | `clipboard.rs` |
| Platform detection | `fn clipboard_read_command(is_macos, env_fn)` — separated for testability | `clipboard.rs:6` |

---

### Module: undo

**Code location:** `src/undo.rs`
**Stability:** Settled

| Concept | Code Manifestation | Location |
|---------|-------------------|----------|
| Operation | `enum Operation` — Insert { pos, text }, Delete { pos, text } | `undo.rs:3` |
| Undo history | `struct UndoHistory` — undo_stack, redo_stack, current_group | `undo.rs:16` |
| Group commit | `UndoHistory::commit_group()` — seals current group onto undo stack; clears redo | `undo.rs:45` |
| Undo / redo | `undo()` / `redo()` — pop group, push to opposite stack, return operations | `undo.rs:56,65` |
| Group accumulation | Operations accumulate in `current_group` until committed at whitespace/newline/mode switch | `undo.rs:18` |

---

### Module: find

**Code location:** `src/find.rs`
**Stability:** Settled

| Concept | Code Manifestation | Location |
|---------|-------------------|----------|
| Find state | `struct FindState` — query, cursor, matches, current_match, saved_cursor, match_ranges_cache | `find.rs:4` |
| Search | `FindState::search(buffer)` — case-insensitive substring search, populates match list | `find.rs:32` |
| Match ranges | `FindState::match_ranges() -> &[(line, start_col, end_col)]` — for WritingSurface highlight | `find.rs` |
| Cursor restore | `saved_cursor: (usize, usize)` — restored on Escape (cancel) | `find.rs:14` |

---

### Module: draft_name

**Code location:** `src/draft_name.rs`
**Stability:** Settled

| Concept | Code Manifestation | Location |
|---------|-------------------|----------|
| Draft name generation | `fn generate() -> String` — `YYYY-MM-DD-HHMM-adjective-plant.md` | `draft_name.rs:27` |
| Vocabulary | `ADJECTIVES` (40 words) + `PLANTS` (40 PNW species) | `draft_name.rs:4,15` |
| UTC timestamp | `fn utc_timestamp() -> String` — computed without chrono via Hinnant's algorithm | `draft_name.rs:36` |

---

### Module: writing_window

**Code location:** `src/writing_window.rs`
**Stability:** Settled

| Concept | Code Manifestation | Location |
|---------|-------------------|----------|
| Terminal detection | `enum Terminal` — Ghostty, Kitty, WezTerm, Alacritty, ITerm2, Unknown | `writing_window.rs:3` |
| Detect terminal | `fn detect_terminal(env_fn) -> Terminal` — reads GHOSTTY_RESOURCES_DIR, KITTY_PID, WEZTERM_EXECUTABLE, TERM_PROGRAM | `writing_window.rs:35` |
| Window config | `struct WindowConfig` — font_family ("PT Mono"), font_size (24), title ("Zani"), padding | `writing_window.rs:14` |
| Spawn command | `fn spawn_command(terminal, config, binary, args) -> Option<Vec<String>>` — builds terminal-specific launch command | `writing_window.rs:59` |
| ZANI_WINDOW guard | `std::env::var("ZANI_WINDOW")` — prevents recursive window spawning | `main.rs:28` |

---

## Responsibility Matrix Summary

| Domain Concept | Owning Type | File |
|---------------|-------------|------|
| Document text | `Buffer` | `buffer.rs` |
| Palette | `Palette` | `palette/mod.rs` |
| Affective Category | `AffectiveCategory` | `palette/mod.rs` |
| Color math (OKLCH, CIEDE2000) | `color_math` functions | `palette/color_math.rs` |
| Palette collection (40 palettes) | `fn all_palettes()` | `palette/collection.rs` |
| Color capability | `ColorProfile` | `color_profile.rs` |
| User preferences | `Config` / `LocalConfig` | `config.rs` |
| Focus mode state | `FocusMode` | `focus_mode.rs` |
| Dimming animation | `DimmingState` / `DimLayer` | `dimming.rs`, `focus_mode.rs` |
| Animation primitives | `AnimatedValue` / `AnimationManager` | `animation.rs` |
| Scroll mode | `ScrollMode` | `scroll_mode.rs` |
| Editing mode | `EditingMode` | `editing_mode.rs` |
| Vim modal state | `Mode` / `Action` | `vim_bindings.rs` |
| Cursor shape | `CursorShape` | `vim_bindings.rs` |
| Editor (cursor + text) | `Editor` | `editor.rs` |
| Undo/redo | `UndoHistory` / `Operation` | `undo.rs` |
| Scroll / column state | `Viewport` | `viewport.rs` |
| Per-character markdown style | `CharStyle` | `markdown_styling.rs` |
| Soft-wrap geometry | `VisualLine` / `wrap_line()` | `wrap.rs` |
| Render cache | `RenderCache` | `writing_surface.rs` |
| Writing surface widget | `WritingSurface` | `writing_surface.rs` |
| Palette browser navigation | `PaletteBrowserState` | `palette_browser.rs` |
| Settings layer | `SettingsState` / `SettingsItem` | `settings.rs` |
| File I/O / autosave | `Persistence` | `persistence.rs` |
| Find overlay | `FindState` | `find.rs` |
| Draft naming | `fn generate()` | `draft_name.rs` |
| Smart typography | `fn transform()` / `SmartEdit` | `smart_typography.rs` |
| Clipboard I/O | `read_clipboard()` / `write_clipboard()` | `clipboard.rs` |
| Window spawning | `writing_window` functions | `writing_window.rs` |
| App coordination | `App` / `TickOutput` | `app.rs` |
| Frame rendering | `fn draw()` / `DrawContext` | `ui.rs` |
| Event loop + terminal init | `fn main()` / `fn run()` | `main.rs` |
