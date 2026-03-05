use crossterm::event::KeyCode;

use crate::editing_mode::EditingMode;
use crate::focus_mode::FocusMode;
use crate::scroll_mode::ScrollMode;

/// A selectable item in the Settings Layer.
/// Defines the logical meaning of each row, replacing magic indices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsItem {
    /// An editing mode choice (Vim or Standard).
    EditingMode(EditingMode),
    /// A palette choice (index into Palette::all()).
    Palette(usize),
    /// A focus mode choice.
    FocusMode(FocusMode),
    /// A scroll mode choice.
    ScrollMode(ScrollMode),
    /// Column width (adjusted via Left/Right, not Enter).
    ColumnWidth,
    /// File row (opens inline rename on Enter).
    File,
}

const ALL_ITEMS: [SettingsItem; 12] = [
    SettingsItem::EditingMode(EditingMode::Vim),
    SettingsItem::EditingMode(EditingMode::Standard),
    SettingsItem::Palette(0),
    SettingsItem::Palette(1),
    SettingsItem::Palette(2),
    SettingsItem::FocusMode(FocusMode::Off),
    SettingsItem::FocusMode(FocusMode::Sentence),
    SettingsItem::FocusMode(FocusMode::Paragraph),
    SettingsItem::ScrollMode(ScrollMode::Edge),
    SettingsItem::ScrollMode(ScrollMode::Typewriter),
    SettingsItem::ColumnWidth,
    SettingsItem::File,
];

impl SettingsItem {
    /// Returns the ordered list of all selectable settings items.
    pub fn all() -> &'static [SettingsItem] {
        &ALL_ITEMS
    }

    /// Look up the item at a given cursor index.
    pub fn at(index: usize) -> Option<SettingsItem> {
        Self::all().get(index).copied()
    }
}

/// State for the Settings Layer overlay.
pub struct SettingsState {
    pub visible: bool,
    pub cursor: usize,
}

impl SettingsState {
    pub fn new() -> Self {
        Self {
            visible: false,
            cursor: 0,
        }
    }

    /// Move the settings cursor up (wrapping).
    pub fn nav_up(&mut self) {
        let count = SettingsItem::all().len();
        if self.cursor == 0 {
            self.cursor = count - 1;
        } else {
            self.cursor -= 1;
        }
    }

    /// Move the settings cursor down (wrapping).
    pub fn nav_down(&mut self) {
        let count = SettingsItem::all().len();
        self.cursor = (self.cursor + 1) % count;
    }

    /// Dismiss the Settings Layer.
    pub fn dismiss(&mut self) {
        self.visible = false;
    }
}

impl Default for SettingsState {
    fn default() -> Self {
        Self::new()
    }
}

/// State for the inline rename overlay.
pub struct RenameState {
    pub active: bool,
    pub buf: String,
    pub cursor: usize,
}

impl RenameState {
    pub fn new() -> Self {
        Self {
            active: false,
            buf: String::new(),
            cursor: 0,
        }
    }

    /// Move rename cursor left.
    pub fn cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move rename cursor right.
    pub fn cursor_right(&mut self) {
        if self.cursor < self.buf.chars().count() {
            self.cursor += 1;
        }
    }

    /// Open inline rename: seed buffer with current filename, cursor at end.
    pub fn open(&mut self, file_path: Option<&std::path::Path>) {
        let name = file_path
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        self.buf = name;
        self.cursor = self.buf.chars().count();
        self.active = true;
    }

    /// Insert a character at cursor position (filters out `/`).
    pub fn insert(&mut self, ch: char) {
        if ch == '/' {
            return;
        }
        let byte_idx = char_to_byte_index(&self.buf, self.cursor);
        self.buf.insert(byte_idx, ch);
        self.cursor += 1;
    }

    /// Delete the character before the cursor.
    pub fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.cursor -= 1;
        let byte_idx = char_to_byte_index(&self.buf, self.cursor);
        // Find the byte length of the char at this position
        let ch = self.buf[byte_idx..].chars().next().unwrap();
        self.buf.replace_range(byte_idx..byte_idx + ch.len_utf8(), "");
    }

    /// Cancel rename, clearing state.
    pub fn cancel(&mut self) {
        self.active = false;
        self.buf.clear();
        self.cursor = 0;
    }

    /// Confirm rename: rename on disk, update file_path, clear scratch flag.
    /// Empty name is treated as cancel.
    pub fn confirm(
        &mut self,
        file_path: &mut Option<std::path::PathBuf>,
        is_scratch: &mut bool,
    ) {
        if self.buf.trim().is_empty() {
            self.cancel();
            return;
        }

        if let Some(old_path) = file_path {
            let new_path = old_path.with_file_name(&self.buf);

            // Only attempt fs::rename if old file exists on disk
            if old_path.exists()
                && std::fs::rename(old_path, &new_path).is_err()
            {
                // Stay in rename mode so user can retry or Esc
                return;
            }

            *file_path = Some(new_path);
            if *is_scratch {
                *is_scratch = false;
            }
        }

        self.active = false;
        self.buf.clear();
        self.cursor = 0;
    }
}

impl Default for RenameState {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of handling a key in the scratch quit prompt.
pub enum ScratchQuitAction {
    /// Key consumed, no state change beyond navigation.
    None,
    /// Prompt dismissed (Esc).
    Close,
    /// User chose an option: 0=Save, 1=Rename, 2=Discard.
    Choose(u8),
}

/// Scratch quit prompt state: Save / Rename / Discard.
pub struct ScratchQuitState {
    pub active: bool,
    /// Selected choice: 0=Save, 1=Rename, 2=Discard.
    pub selected: u8,
}

impl ScratchQuitState {
    pub fn new() -> Self {
        Self { active: false, selected: 0 }
    }

    pub fn open(&mut self) {
        self.active = true;
        self.selected = 0;
    }

    pub fn handle_key(&mut self, code: KeyCode) -> ScratchQuitAction {
        match code {
            KeyCode::Left | KeyCode::Char('h') | KeyCode::Up | KeyCode::Char('k') => {
                self.selected = if self.selected == 0 { 2 } else { self.selected - 1 };
                ScratchQuitAction::None
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Down | KeyCode::Char('j') => {
                self.selected = if self.selected == 2 { 0 } else { self.selected + 1 };
                ScratchQuitAction::None
            }
            KeyCode::Esc => {
                self.active = false;
                ScratchQuitAction::Close
            }
            KeyCode::Enter => ScratchQuitAction::Choose(self.selected),
            KeyCode::Char('s') | KeyCode::Char('S') => ScratchQuitAction::Choose(0),
            KeyCode::Char('r') | KeyCode::Char('R') => ScratchQuitAction::Choose(1),
            KeyCode::Char('d') | KeyCode::Char('D') => ScratchQuitAction::Choose(2),
            _ => ScratchQuitAction::None,
        }
    }
}

impl Default for ScratchQuitState {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a char index to a byte index in a UTF-8 string.
fn char_to_byte_index(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    // === ScratchQuitState ===

    #[test]
    fn scratch_quit_nav_wraps_forward() {
        let mut state = ScratchQuitState::new();
        state.open();
        state.selected = 2;
        state.handle_key(KeyCode::Right);
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn scratch_quit_nav_wraps_backward() {
        let mut state = ScratchQuitState::new();
        state.open();
        state.selected = 0;
        state.handle_key(KeyCode::Left);
        assert_eq!(state.selected, 2);
    }

    #[test]
    fn scratch_quit_esc_deactivates() {
        let mut state = ScratchQuitState::new();
        state.open();
        assert!(state.active);
        let action = state.handle_key(KeyCode::Esc);
        assert!(!state.active);
        assert!(matches!(action, ScratchQuitAction::Close));
    }

    #[test]
    fn scratch_quit_enter_returns_selected() {
        let mut state = ScratchQuitState::new();
        state.open();
        state.selected = 1;
        let action = state.handle_key(KeyCode::Enter);
        assert!(matches!(action, ScratchQuitAction::Choose(1)));
    }

    #[test]
    fn scratch_quit_hotkeys() {
        let mut state = ScratchQuitState::new();
        state.open();
        assert!(matches!(state.handle_key(KeyCode::Char('s')), ScratchQuitAction::Choose(0)));
        assert!(matches!(state.handle_key(KeyCode::Char('R')), ScratchQuitAction::Choose(1)));
        assert!(matches!(state.handle_key(KeyCode::Char('d')), ScratchQuitAction::Choose(2)));
    }
}
