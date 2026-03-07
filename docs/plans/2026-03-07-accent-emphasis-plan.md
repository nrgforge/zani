# Accent Emphasis as Signature Species Color — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Wire the dormant `accent_emphasis` palette field into bold/italic text rendering and browser swatches, then retune all 40 values to each species' signature botanical color.

**Architecture:** Three structural changes (markdown resolve, browser swatches, settings swatches) followed by a palette-by-palette color retune. TDD for the structural changes; the retune is validated by existing WCAG AA and hue diversity tests.

**Tech Stack:** Rust, ratatui, WCAG 2.0 contrast validation

---

### Task 1: Wire accent_emphasis into bold/italic text rendering

**Files:**
- Modify: `src/markdown_styling.rs:32-51` (CharStyle::resolve)
- Test: `src/markdown_styling.rs` (existing test module)

**Step 1: Write the failing test**

Add to the test module in `src/markdown_styling.rs`:

```rust
#[test]
fn resolve_bold_uses_accent_emphasis() {
    let palette = Palette::default_palette();
    let s = CharStyle { modifier: Modifier::BOLD, ..Default::default() };
    let style = s.resolve(&palette);
    assert_eq!(style.fg.unwrap(), palette.accent_emphasis);
}

#[test]
fn resolve_italic_uses_accent_emphasis() {
    let palette = Palette::default_palette();
    let s = CharStyle { modifier: Modifier::ITALIC, ..Default::default() };
    let style = s.resolve(&palette);
    assert_eq!(style.fg.unwrap(), palette.accent_emphasis);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test resolve_bold_uses resolve_italic_uses -- --nocapture`
Expected: FAIL — both assert `accent_emphasis` but resolve currently returns `foreground`.

**Step 3: Implement the change**

In `CharStyle::resolve()`, add a check for bold/italic modifier after the existing accent checks but before the `foreground` fallback. Change the else branch:

```rust
pub fn resolve(&self, palette: &Palette) -> Style {
    let fg = if self.is_syntax {
        palette.dimmed_foreground
    } else if self.is_heading {
        palette.accent_heading
    } else if self.is_link_text {
        palette.accent_link
    } else if self.is_code {
        palette.accent_code
    } else if self.modifier.contains(Modifier::BOLD) || self.modifier.contains(Modifier::ITALIC) {
        palette.accent_emphasis
    } else {
        palette.foreground
    };

    Style::default()
        .fg(fg)
        .bg(palette.background)
        .add_modifier(self.modifier)
}
```

**Step 4: Run all tests**

Run: `cargo test`
Expected: All pass. The existing `resolve_plain_uses_foreground` test uses `CharStyle::default()` which has `Modifier::empty()`, so it still hits the `foreground` branch.

**Step 5: Commit**

```
git add src/markdown_styling.rs
git commit -m "feat: use accent_emphasis for bold/italic text color"
```

---

### Task 2: Pack browser swatches tight, add accent_emphasis as 4th swatch

**Files:**
- Modify: `src/ui.rs` — `draw_palette_browser` function (swatch rendering block)
- Test: `src/ui.rs` (existing test module)

**Step 1: Write the failing test**

Add to the test module in `src/ui.rs`:

```rust
#[test]
fn palette_browser_shows_emphasis_swatch() {
    let mut app = App::new();
    app.toggle_settings();
    app.settings.cursor = crate::settings::SettingsItem::all()
        .iter()
        .position(|i| *i == SettingsItem::Palette)
        .unwrap();
    app.settings_apply();

    let buf = render_app(&mut app, 80, 30);

    // Find the row containing "Manzanita" and look for accent_emphasis bg color
    let default = Palette::default_palette();
    let area = buf.area;
    for y in area.top()..area.bottom() {
        let mut row_text = String::new();
        for x in area.left()..area.right() {
            row_text.push_str(buf[(x, y)].symbol());
        }
        if row_text.contains("Manzanita") {
            let mut found_emphasis = false;
            for x in area.left()..area.right() {
                if buf[(x, y)].bg == default.accent_emphasis {
                    found_emphasis = true;
                    break;
                }
            }
            assert!(found_emphasis, "Browser swatch row should include accent_emphasis color");
            return;
        }
    }
    panic!("Could not find Manzanita row");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test palette_browser_shows_emphasis_swatch`
Expected: FAIL — current swatches only show background, foreground, accent_heading.

**Step 3: Implement the change**

In `draw_palette_browser`, replace the swatch block (around line 802-814):

```rust
            // Right-align swatches: packed tight, no spacers
            // Swatch block: 4 * "  " = 8 chars
            let swatch_width: usize = 8;
            let inner_width = (overlay_width as usize).saturating_sub(2);
            let name_width = inner_width.saturating_sub(swatch_width);
            let padded = format!("{:<width$}", text, width = name_width);

            let mut spans = vec![Span::styled(padded, style)];
            for color in [p.background, p.foreground, p.accent_heading, p.accent_emphasis] {
                spans.push(Span::styled("  ", Style::default().bg(color)));
            }
            lines.push(Line::from(spans));
```

**Step 4: Run all tests**

Run: `cargo test`
Expected: All pass. The existing `palette_rows_have_color_swatches` test checks for bg/fg/accent swatches on the Settings row (not browser), so it's unaffected.

**Step 5: Commit**

```
git add src/ui.rs
git commit -m "feat: pack browser swatches tight, add accent_emphasis as 4th swatch"
```

---

### Task 3: Update settings layer swatches to match

**Files:**
- Modify: `src/ui.rs` — settings swatch vec (line ~562-564) and swatch rendering (line ~712-720)

**Step 1: Update the swatch color list**

In `draw_settings_layer`, change the swatches for the Palette row:

```rust
        let swatches = match item {
            SettingsItem::Palette => {
                vec![palette.background, palette.foreground, palette.accent_heading, palette.accent_emphasis]
            }
            _ => vec![],
        };
```

**Step 2: Pack the swatch rendering tight (remove spacers)**

In the swatch rendering block (around line 712-720), change:

```rust
            } else {
                let mut spans = vec![Span::styled(row.text.clone(), style)];
                for color in &row.swatches {
                    spans.push(Span::styled(
                        "  ",
                        Style::default().bg(*color),
                    ));
                }
                Line::from(spans)
            }
```

**Step 3: Run all tests**

Run: `cargo test`
Expected: All pass. The `palette_rows_have_color_swatches` test checks for bg, fg, and accent_heading swatches — all still present.

**Step 4: Commit**

```
git add src/ui.rs
git commit -m "refactor: pack settings swatches tight, add accent_emphasis"
```

---

### Task 4: Retune all 40 accent_emphasis values to signature botanical colors

**Files:**
- Modify: `src/palette/collection.rs` — all 40 palette definitions

**Context:** Each palette's `accent_emphasis` should become the single most visually distinctive color of the named species. The color must pass WCAG AA (4.5:1 contrast ratio against the palette's background). Use the provenance description to identify the characteristic feature and its color.

**Color reference for each palette:**

Dark — Warm:
- Manzanita: sage-green (powdery leaves)
- Chinquapin: golden (leaf scale undersides)
- Red Cedar: amber-gold (aromatic heartwood)
- Chanterelle: egg-yolk pale gold (interior flesh)
- Madrone: terra-cotta red (peeled bark)

Dark — Cool:
- Sitka: silvery blue-green (dense needles)
- Oakmoss: teal-green (lichen thallus)
- Oregon Grape: golden-yellow (flowers)
- Elderberry: powder-blue (waxy berry bloom)
- Witches Hair: olive-green (pendant lichen)

Dark — Vivid:
- Fly Agaric: scarlet red (cap color)
- Lungwort: bright green (wet thallus)
- Jack O'Lantern: bioluminescent orange-green (glowing gills)
- Violet Cort: deep violet (entire mushroom)
- Salal: dark blue-purple (ripe berries)

Dark — Muted:
- Old Man's Beard: silvery sage-green (lichen thallus)
- Red Alder: pale gray-green (bark lichen)
- Douglas Fir: blue-green (needle color)
- Sagebrush: silver-sage (foliage)
- Map Lichen: chartreuse-yellow (thallus patches)

Light — Warm:
- Oatgrass: golden straw (midsummer color)
- Oregon Sunshine: bright yellow (flower heads)
- White Oak: pale buff (fissured bark)
- Ponderosa: butterscotch (bark plates)
- Balsamroot: amber-gold (sunflower blooms)

Light — Cool:
- Cascade Aster: lavender (ray flowers)
- Pearly Everlasting: papery white (bracts)
- Partridgefoot: creamy white (racemes)
- Lupine: blue-violet (flower spires)
- Phlox: pale pink (cushion flowers)

Light — Vivid:
- Paintbrush: green (calyx beneath red bracts — red already in heading)
- Columbine: yellow (inner petals — red in heading)
- Tiger Lily: maroon (tepal spots)
- Farewell: vivid pink-magenta (satiny petals)
- Camas: blue-violet (star flowers)

Light — Muted:
- Sword Fern: deep green (leathery fronds)
- Oceanspray: parchment tan (dried flower panicles)
- Goatsbeard: creamy white (airy plumes)
- Fringecup: soft pink (aging bell flowers)
- Reindeer Lichen: silvery pale green (cushion growth)

**Step 1: Update all 40 accent_emphasis values**

Replace each `accent_emphasis` color with the signature botanical color. Each must pass `palette.validate()` (WCAG AA 4.5:1 against background).

**Step 2: Run all tests**

Run: `cargo test`
Expected: All 430+ tests pass, including `all_palettes_satisfy_invariant_3` (WCAG AA).

**Step 3: Commit**

```
git add src/palette/collection.rs
git commit -m "feat: retune accent_emphasis to signature botanical colors for all 40 palettes"
```

---

### Task 5: Verify end-to-end

**Step 1: Run full test suite**

Run: `cargo test`
Expected: All tests pass.

**Step 2: Visual check**

Run: `cargo run` and open the palette browser. Verify:
- 4th swatch appears for each palette
- Swatches are packed tight (no gaps between colors)
- Bold text in the writing surface uses the signature color
- Provenance footer still renders correctly

**Step 3: Final commit if any fixups needed**
