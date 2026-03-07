# Design: Accent Emphasis as Signature Species Color

## Problem

Many palettes lack their most distinctive botanical color. Manzanita lacks sage-green leaves, Douglas Fir lacks needle-green, Oregon Grape lacks yellow flowers. Meanwhile, `accent_emphasis` is defined in every palette, validated for contrast, and blended during crossfades — but never rendered. Bold/italic text just uses `foreground` with a modifier.

## Decision

Give `accent_emphasis` a real job: the signature species color, visible in both body text and browser swatches.

### Changes

**1. Markdown rendering** (`markdown_styling.rs`)

`CharStyle::resolve()` uses `accent_emphasis` as fg for bold and italic text (when `modifier` contains `Bold` or `Italic`), replacing plain `foreground`.

**2. Browser swatches** (`ui.rs`)

Add `accent_emphasis` as a 4th swatch. Pack swatches tight with no spacers between them:

```
  > Manzanita (project)        ████████
                                 bg fg hd em
```

4 swatches x 2 chars = 8 chars. Name area = inner_width - 8.

**3. Palette values** (`collection.rs`)

Retune all 40 `accent_emphasis` values to be each species' most visually distinctive color — the color you'd paint first if sketching it from memory.

### Color selection principle

`accent_heading` captures the palette's mood. `accent_emphasis` captures the species' identity.

Examples:
- Manzanita: powdery sage-green (leaves)
- Douglas Fir: blue-green (needles)
- Oregon Grape: golden-yellow (flowers)
- Sitka Spruce: silvery blue-green (needles)
- Paintbrush: green (calyx — red already in heading)

### Constraints

- WCAG AA (4.5:1) required for `accent_emphasis` vs `background` — unchanged
- No pure black/white — unchanged
- Hue diversity 15deg minimum on `sort_key` (background hue) — unaffected since this constraint applies to backgrounds, not accents
