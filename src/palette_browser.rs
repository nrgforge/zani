use crate::palette::{AffectiveCategory, Palette};

/// State machine for browsing palettes by Affective Category (ADR-010).
/// Entered from the Settings Layer's Palette row. Organizes palettes
/// by category with Perceptual Sort Order within each category.
pub struct PaletteBrowserState {
    pub open: bool,
    /// Category groups, populated from Palette::all_by_category().
    groups: Vec<(AffectiveCategory, Vec<Palette>)>,
    /// Index into groups (which category is focused).
    pub category_idx: usize,
    /// Index into the current category's palette list.
    pub palette_idx: usize,
    /// Name of the currently active palette (for marking).
    active_palette: String,
}

impl PaletteBrowserState {
    pub fn new() -> Self {
        Self {
            open: false,
            groups: Vec::new(),
            category_idx: 0,
            palette_idx: 0,
            active_palette: String::new(),
        }
    }

    /// Open the browser, populating groups from the palette collection.
    pub fn open(&mut self, active_palette_name: &str) {
        self.groups = Palette::all_by_category();
        self.active_palette = active_palette_name.to_string();
        self.open = true;

        // Position cursor on the active palette's category and index
        for (gi, (_, palettes)) in self.groups.iter().enumerate() {
            for (pi, p) in palettes.iter().enumerate() {
                if p.name == active_palette_name {
                    self.category_idx = gi;
                    self.palette_idx = pi;
                    return;
                }
            }
        }

        // Fallback: first palette in first category
        self.category_idx = 0;
        self.palette_idx = 0;
    }

    /// Close the browser.
    pub fn close(&mut self) {
        self.open = false;
    }

    /// Returns the currently focused palette, if any.
    pub fn focused_palette(&self) -> Option<Palette> {
        self.groups
            .get(self.category_idx)
            .and_then(|(_, palettes)| palettes.get(self.palette_idx))
            .copied()
    }

    /// Returns the category groups for rendering.
    pub fn groups(&self) -> &[(AffectiveCategory, Vec<Palette>)] {
        &self.groups
    }

    /// Returns the active palette name for marking.
    pub fn active_palette_name(&self) -> &str {
        &self.active_palette
    }

    /// Move cursor down: next palette in category, or first palette in next category.
    pub fn nav_down(&mut self) {
        if self.groups.is_empty() {
            return;
        }
        let cat_len = self.groups[self.category_idx].1.len();
        if self.palette_idx + 1 < cat_len {
            self.palette_idx += 1;
        } else if self.category_idx + 1 < self.groups.len() {
            self.category_idx += 1;
            self.palette_idx = 0;
        }
        // At the end of last category: stay put (no wrapping)
    }

    /// Move cursor up: previous palette in category, or last palette in previous category.
    pub fn nav_up(&mut self) {
        if self.groups.is_empty() {
            return;
        }
        if self.palette_idx > 0 {
            self.palette_idx -= 1;
        } else if self.category_idx > 0 {
            self.category_idx -= 1;
            self.palette_idx = self.groups[self.category_idx].1.len().saturating_sub(1);
        }
        // At the start of first category: stay put (no wrapping)
    }

    /// Update the active palette name after a selection is applied.
    pub fn set_active(&mut self, name: &str) {
        self.active_palette = name.to_string();
    }
}

impl Default for PaletteBrowserState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Acceptance tests: Palette Browser state machine (ADR-010) ===

    #[test]
    fn browser_groups_palettes_by_affective_category() {
        let mut browser = PaletteBrowserState::new();
        browser.open("Manzanita");

        let groups = browser.groups();
        assert!(!groups.is_empty(), "Browser should have palette groups");

        // Each group should have a category and at least one palette
        for (cat, palettes) in groups {
            assert!(!cat.label().is_empty());
            assert!(!palettes.is_empty(), "Category {:?} should have palettes", cat);
            // All palettes in a group should share the same category
            for p in palettes {
                assert_eq!(p.category, *cat, "Palette '{}' should be in category {:?}", p.name, cat);
            }
        }
    }

    #[test]
    fn browser_opens_with_cursor_on_active_palette() {
        let mut browser = PaletteBrowserState::new();
        browser.open("Sitka");

        let focused = browser.focused_palette().unwrap();
        assert_eq!(focused.name, "Sitka", "Cursor should start on active palette");
    }

    #[test]
    fn nav_down_moves_through_palettes_and_categories() {
        let mut browser = PaletteBrowserState::new();
        browser.open("Manzanita");

        browser.nav_down();
        // Should have moved — either to next palette in category or next category
        let after = browser.focused_palette().unwrap();
        assert!(!after.name.is_empty());

        // Navigate all the way through
        for _ in 0..10 {
            browser.nav_down();
        }
        // Should stop at end, not crash
        assert!(browser.focused_palette().is_some());
    }

    #[test]
    fn nav_up_moves_back() {
        let mut browser = PaletteBrowserState::new();
        browser.open("Manzanita");

        // Navigate down then back up
        let initial = browser.focused_palette().unwrap().name.to_string();
        browser.nav_down();
        browser.nav_up();
        let back = browser.focused_palette().unwrap();
        assert_eq!(back.name, initial, "Up after down should return to start");
    }

    #[test]
    fn esc_closes_browser() {
        let mut browser = PaletteBrowserState::new();
        browser.open("Manzanita");
        assert!(browser.open);

        browser.close();
        assert!(!browser.open);
    }

    #[test]
    fn set_active_updates_marker() {
        let mut browser = PaletteBrowserState::new();
        browser.open("Manzanita");
        assert_eq!(browser.active_palette_name(), "Manzanita");

        browser.set_active("Sitka");
        assert_eq!(browser.active_palette_name(), "Sitka");
    }
}
