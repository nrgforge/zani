# ADR-010: Palette Browser Sub-Panel

**Status:** Proposed

## Context

The current Settings Layer lists palettes as inline rows (`SettingsItem::Palette(0)`, `Palette(1)`, `Palette(2)`) in a static array of 12 items. With a larger collection organized by Affective Category (ADR-009), inline rows would dominate the settings panel. The domain model defines Palette Browser as a dedicated sub-panel of the Settings Layer, governed by Invariant 1 (hidden by default, summoned on demand).

## Decision

Replace inline palette rows in the Settings Layer with a single "Palette: [current name]" row. Selecting that row opens the Palette Browser — a dedicated sub-panel showing Palettes grouped by Affective Category with Perceptual Sort Order, color swatches, and optional name/category filtering.

Applying a Palette from the browser triggers the existing 300ms crossfade animation. The browser indicates when a Local Config override is active (per ADR-011): "Neon Noir (project)" versus "Ember (global)".

The browser is dismissed via Esc (returning to the Settings Layer) or by the same key that Summons settings (closing everything).

**Rejected alternatives:**

- **Keep inline rows, add more:** Does not scale. 15+ palette rows would dominate the 12-item settings panel.
- **Modal popup dialog:** Breaks the Settings Layer interaction pattern (overlay, not dialog).
- **Command palette / search-only:** Loses the categorized browsing experience. Search is a supplement, not a replacement.

## Consequences

**Positive:**
- The Settings Layer stays uncluttered — one row for palette, not N rows.
- Full browsing experience with categories, swatches, and filtering. Scales to any collection size.
- The existing crossfade animation generalizes without modification (`Palette::blend` works for any two palettes).

**Negative:**
- New sub-panel state machine: open, navigate by category, navigate within category, filter, apply, dismiss. More implementation complexity than extending the flat list.
- The `SettingsItem::Palette(usize)` variant and the static `ALL_ITEMS` array need restructuring.

**Neutral:**
- The Palette Browser is a sub-panel of the Settings Layer, not a separate overlay. It shares the same dismissal behavior (Invariant 1).
