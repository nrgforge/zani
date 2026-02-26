use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::buffer::Buffer;
use crate::focus_mode::FocusMode;
use crate::palette::Palette;
use crate::smart_typography;
use crate::vim_bindings::{self, Action, CursorShape, Mode};
use crate::wrap::wrap_line;

/// Application state.
pub struct App {
    pub buffer: Buffer,
    pub palette: Palette,
    pub focus_mode: FocusMode,
    pub vim_mode: Mode,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll_offset: usize,
    pub column_width: u16,
    pub chrome_visible: bool,
    pub settings_visible: bool,
    pub should_quit: bool,
    pub file_path: Option<PathBuf>,
    pub dirty: bool,
    pub last_save: Option<Instant>,
    pub autosave_interval: Duration,
}

impl App {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            palette: Palette::default_palette(),
            focus_mode: FocusMode::Off,
            vim_mode: Mode::Normal,
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
            column_width: 60,
            chrome_visible: false,
            settings_visible: false,
            should_quit: false,
            file_path: None,
            dirty: false,
            last_save: None,
            autosave_interval: Duration::from_secs(3),
        }
    }

    pub fn with_file(mut self, path: PathBuf, content: &str) -> Self {
        self.buffer = Buffer::from_text(content);
        self.file_path = Some(path);
        self
    }

    /// The cursor shape based on current vim mode.
    pub fn cursor_shape(&self) -> CursorShape {
        self.vim_mode.cursor_shape()
    }

    /// Toggle the Settings Layer visibility.
    pub fn toggle_settings(&mut self) {
        self.settings_visible = !self.settings_visible;
        self.chrome_visible = self.settings_visible;
    }

    /// Dismiss the Settings Layer.
    pub fn dismiss_settings(&mut self) {
        self.settings_visible = false;
        self.chrome_visible = false;
    }

    /// Process a character key input.
    pub fn handle_char(&mut self, ch: char) {
        let action = match self.vim_mode {
            Mode::Normal => {
                // Check for quit
                if ch == 'q' {
                    self.should_quit = true;
                    return;
                }
                vim_bindings::handle_normal(ch)
            }
            Mode::Insert => vim_bindings::handle_insert(ch),
            Mode::Visual => vim_bindings::handle_normal(ch),
        };

        self.apply_action(action);
    }

    /// Process Escape key.
    pub fn handle_escape(&mut self) {
        if self.settings_visible {
            self.dismiss_settings();
        } else if self.vim_mode == Mode::Insert {
            self.vim_mode = Mode::Normal;
        }
    }

    pub fn apply_action(&mut self, action: Action) {
        match action {
            Action::SwitchMode(mode) => {
                self.vim_mode = mode;
            }
            Action::InsertChar(ch) => {
                self.insert_char(ch);
            }
            Action::InsertNewline => {
                let idx = self.cursor_char_index();
                self.buffer.insert(idx, "\n");
                self.cursor_line += 1;
                self.cursor_col = 0;
                self.dirty = true;
            }
            Action::DeleteBack => {
                let idx = self.cursor_char_index();
                if idx > 0 {
                    self.buffer.remove(idx - 1, idx);
                    if self.cursor_col > 0 {
                        self.cursor_col -= 1;
                    } else if self.cursor_line > 0 {
                        self.cursor_line -= 1;
                        self.cursor_col = self.buffer.line(self.cursor_line).len_chars().saturating_sub(1);
                    }
                    self.dirty = true;
                }
            }
            Action::MoveCursor(dir) => {
                self.move_cursor(dir);
            }
            Action::None => {}
        }
    }

    fn insert_char(&mut self, ch: char) {
        let idx = self.cursor_char_index();

        // Check for smart typography transformation
        let preceding = self.preceding_text(idx);
        if let Some(edit) = smart_typography::transform(ch, &preceding) {
            // Delete preceding characters
            if edit.delete_before > 0 {
                let start = idx.saturating_sub(edit.delete_before);
                self.buffer.remove(start, idx);
                self.cursor_col -= edit.delete_before;
            }
            // Insert replacement
            let new_idx = self.cursor_char_index();
            self.buffer.insert(new_idx, edit.insert);
            self.cursor_col += edit.insert.chars().count();
        } else {
            self.buffer.insert(idx, &ch.to_string());
            self.cursor_col += 1;
        }
        self.dirty = true;
    }

    /// Get the text preceding the cursor position (on the current line).
    fn preceding_text(&self, char_idx: usize) -> String {
        let line_start = self.line_start_char_index();
        if char_idx <= line_start {
            return String::new();
        }
        let rope = self.buffer.rope();
        rope.slice(line_start..char_idx).to_string()
    }

    /// Calculate the character index in the buffer for the current cursor position.
    fn cursor_char_index(&self) -> usize {
        self.line_start_char_index() + self.cursor_col
    }

    /// Character index of the start of the current line.
    fn line_start_char_index(&self) -> usize {
        let rope = self.buffer.rope();
        rope.line_to_char(self.cursor_line)
    }

    fn move_cursor(&mut self, dir: vim_bindings::Direction) {
        match dir {
            vim_bindings::Direction::Left => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                }
            }
            vim_bindings::Direction::Right => {
                let line_len = self.buffer.line(self.cursor_line).len_chars();
                // Don't count the newline
                let max_col = if line_len > 0 { line_len - 1 } else { 0 };
                if self.cursor_col < max_col {
                    self.cursor_col += 1;
                }
            }
            vim_bindings::Direction::Up => {
                if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    self.clamp_cursor_col();
                }
            }
            vim_bindings::Direction::Down => {
                if self.cursor_line + 1 < self.buffer.len_lines() {
                    self.cursor_line += 1;
                    self.clamp_cursor_col();
                }
            }
        }
    }

    fn clamp_cursor_col(&mut self) {
        let line_len = self.buffer.line(self.cursor_line).len_chars();
        let max_col = if line_len > 0 { line_len - 1 } else { 0 };
        if self.cursor_col > max_col {
            self.cursor_col = max_col;
        }
    }

    /// Find the paragraph bounds (start_line, end_line) containing the cursor.
    /// A paragraph is a contiguous block of non-empty lines.
    pub fn paragraph_bounds(&self) -> Option<(usize, usize)> {
        let total = self.buffer.len_lines();
        if total == 0 {
            return None;
        }

        // Search backward for paragraph start (blank line or buffer start)
        let mut start = self.cursor_line;
        while start > 0 {
            let line = self.buffer.line(start - 1).to_string();
            if line.trim().is_empty() {
                break;
            }
            start -= 1;
        }

        // Search forward for paragraph end (blank line or buffer end)
        let mut end = self.cursor_line;
        while end + 1 < total {
            let line = self.buffer.line(end + 1).to_string();
            if line.trim().is_empty() {
                break;
            }
            end += 1;
        }

        Some((start, end))
    }

    /// Adjust scroll_offset so the cursor stays visible within the given height.
    pub fn ensure_cursor_visible(&mut self, visible_height: u16) {
        let height = visible_height as usize;
        if height == 0 {
            return;
        }

        // Compute visual lines to find the cursor's visual position
        let mut cursor_vl = 0;
        let mut found = false;
        let mut vl_index = 0;
        for i in 0..self.buffer.len_lines() {
            let line_text = self.buffer.line(i).to_string();
            let wrapped = wrap_line(&line_text, self.column_width as usize, i);
            for vl in &wrapped {
                if vl.logical_line == self.cursor_line
                    && self.cursor_col >= vl.char_start
                    && self.cursor_col < vl.char_end
                {
                    cursor_vl = vl_index;
                    found = true;
                    break;
                }
                // Handle cursor at end of a visual line
                if vl.logical_line == self.cursor_line && self.cursor_col == vl.char_end {
                    cursor_vl = vl_index;
                }
                vl_index += 1;
            }
            if found {
                break;
            }
        }

        if cursor_vl < self.scroll_offset {
            self.scroll_offset = cursor_vl;
        } else if cursor_vl >= self.scroll_offset + height {
            self.scroll_offset = cursor_vl - height + 1;
        }
    }

    /// Check if autosave should trigger (dirty + enough time elapsed).
    pub fn should_autosave(&self) -> bool {
        if !self.dirty {
            return false;
        }
        match self.last_save {
            Some(last) => last.elapsed() >= self.autosave_interval,
            None => true, // Never saved and dirty — save now
        }
    }

    /// Perform autosave if a file path is set. Returns true if saved.
    pub fn autosave(&mut self) -> bool {
        if !self.dirty || self.file_path.is_none() {
            return false;
        }

        if let Some(path) = &self.file_path {
            let content = self.buffer.rope().to_string();
            if std::fs::write(path, &content).is_ok() {
                self.dirty = false;
                self.last_save = Some(Instant::now());
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // === Acceptance test: Default state has no visible Chrome ===

    #[test]
    fn default_state_has_no_chrome() {
        let app = App::new();
        assert!(!app.chrome_visible);
        assert!(!app.settings_visible);
    }

    // === Acceptance test: Settings Layer is summoned by hotkey ===

    #[test]
    fn toggle_settings_makes_chrome_visible() {
        let mut app = App::new();
        app.toggle_settings();
        assert!(app.settings_visible);
        assert!(app.chrome_visible);
    }

    // === Acceptance test: Settings Layer is dismissed ===

    #[test]
    fn dismiss_settings_hides_chrome() {
        let mut app = App::new();
        app.toggle_settings();
        assert!(app.settings_visible);
        app.dismiss_settings();
        assert!(!app.settings_visible);
        assert!(!app.chrome_visible);
    }

    #[test]
    fn escape_dismisses_settings() {
        let mut app = App::new();
        app.toggle_settings();
        app.handle_escape();
        assert!(!app.settings_visible);
    }

    // === Acceptance test: Document is saved automatically ===

    #[test]
    fn autosave_writes_buffer_to_disk() {
        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, "initial").unwrap();
        let path = tmp.path().to_path_buf();

        let mut app = App::new().with_file(path.clone(), "initial");
        // Type something
        app.vim_mode = Mode::Insert;
        app.handle_char('!');
        assert!(app.dirty);

        let saved = app.autosave();
        assert!(saved);
        assert!(!app.dirty);

        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "!initial");
    }

    // === Acceptance test: Autosave does not disrupt writing ===
    // (Tested by verifying autosave is a simple fs::write, no UI interaction)

    #[test]
    fn autosave_when_not_dirty_is_noop() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();
        let mut app = App::new().with_file(path, "content");
        // Not dirty
        assert!(!app.autosave());
    }

    // === Acceptance test: Vim mode switch ===

    #[test]
    fn i_enters_insert_and_escape_returns_to_normal() {
        let mut app = App::new();
        assert_eq!(app.vim_mode, Mode::Normal);
        app.handle_char('i');
        assert_eq!(app.vim_mode, Mode::Insert);
        app.handle_escape();
        assert_eq!(app.vim_mode, Mode::Normal);
    }

    // === Unit test: Smart typography in insert mode ===

    #[test]
    fn smart_quotes_applied_during_insert() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("He said ");
        app.cursor_line = 0;
        app.cursor_col = 8;
        app.vim_mode = Mode::Insert;

        app.handle_char('"');
        let text = app.buffer.rope().to_string();
        assert!(text.contains('\u{201C}'), "Should have opening curly quote, got: {}", text);
    }

    // === Unit test: Cursor movement ===

    #[test]
    fn hjkl_moves_cursor() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("line one\nline two\nline three");
        app.cursor_line = 1;
        app.cursor_col = 3;

        app.handle_char('h');
        assert_eq!(app.cursor_col, 2);
        app.handle_char('l');
        assert_eq!(app.cursor_col, 3);
        app.handle_char('k');
        assert_eq!(app.cursor_line, 0);
        app.handle_char('j');
        assert_eq!(app.cursor_line, 1);
    }
}
