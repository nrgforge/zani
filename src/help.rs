//! First-launch help dialog state.
//!
//! The dialog displays a single line pointing the user at the Settings layer.
//! Visibility is initialized from `Config::show_help_on_launch` and dismissed
//! by Esc or Enter. Dismissing does not flip the persistent config flag —
//! the user changes that through the Settings layer.

/// Help overlay state — visible flag only. Rendering and routing live in
/// `ui.rs` and `app.rs`; this module just tracks whether to show it.
pub struct HelpOverlay {
    pub visible: bool,
}

impl HelpOverlay {
    /// Build a new overlay. `show` is read from `Config::show_help_on_launch`.
    pub fn new(show: bool) -> Self {
        Self { visible: show }
    }

    /// Dismiss the overlay for this session.
    pub fn dismiss(&mut self) {
        self.visible = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_true_is_visible() {
        assert!(HelpOverlay::new(true).visible);
    }

    #[test]
    fn new_false_is_not_visible() {
        assert!(!HelpOverlay::new(false).visible);
    }

    #[test]
    fn dismiss_clears_visible() {
        let mut h = HelpOverlay::new(true);
        h.dismiss();
        assert!(!h.visible);
    }
}
