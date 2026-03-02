use crate::editing_mode::EditingMode;
use crate::focus_mode::FocusMode;
use crate::palette::Palette;
use crate::scroll_mode::ScrollMode;

/// A selectable item in the Settings Layer.
/// Defines the logical meaning of each row, replacing magic indices.
#[derive(Debug, Clone, PartialEq, Eq)]
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

impl SettingsItem {
    /// Returns the ordered list of all selectable settings items.
    pub fn all() -> Vec<SettingsItem> {
        let mut items = Vec::new();
        items.push(SettingsItem::EditingMode(EditingMode::Vim));
        items.push(SettingsItem::EditingMode(EditingMode::Standard));
        for i in 0..Palette::all().len() {
            items.push(SettingsItem::Palette(i));
        }
        items.push(SettingsItem::FocusMode(FocusMode::Off));
        items.push(SettingsItem::FocusMode(FocusMode::Sentence));
        items.push(SettingsItem::FocusMode(FocusMode::Paragraph));
        items.push(SettingsItem::ScrollMode(ScrollMode::Edge));
        items.push(SettingsItem::ScrollMode(ScrollMode::Typewriter));
        items.push(SettingsItem::ColumnWidth);
        items.push(SettingsItem::File);
        items
    }

    /// Look up the item at a given cursor index.
    pub fn at(index: usize) -> Option<SettingsItem> {
        Self::all().into_iter().nth(index)
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

/// Convert a char index to a byte index in a UTF-8 string.
fn char_to_byte_index(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}
