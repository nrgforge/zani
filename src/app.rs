use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::buffer::Buffer;
use crate::clipboard;
use crate::color_profile::ColorProfile;
use crate::draft_name;
use crate::focus_mode::{self, FocusMode};
use crate::palette::Palette;
use crate::smart_typography;
use crate::vim_bindings::{self, Action, CursorShape, Mode};
use crate::wrap::{self, VisualLine};

/// A selectable item in the Settings Layer.
/// Defines the logical meaning of each row, replacing magic indices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsItem {
    /// A palette choice (index into Palette::all()).
    Palette(usize),
    /// A focus mode choice.
    FocusMode(FocusMode),
    /// Column width (adjusted via Left/Right, not Enter).
    ColumnWidth,
    /// File row (opens inline rename on Enter).
    File,
}

impl SettingsItem {
    /// Returns the ordered list of all selectable settings items.
    pub fn all() -> Vec<SettingsItem> {
        let mut items = Vec::new();
        for i in 0..Palette::all().len() {
            items.push(SettingsItem::Palette(i));
        }
        items.push(SettingsItem::FocusMode(FocusMode::Off));
        items.push(SettingsItem::FocusMode(FocusMode::Sentence));
        items.push(SettingsItem::FocusMode(FocusMode::Paragraph));
        items.push(SettingsItem::FocusMode(FocusMode::Typewriter));
        items.push(SettingsItem::ColumnWidth);
        items.push(SettingsItem::File);
        items
    }

    /// Look up the item at a given cursor index.
    pub fn at(index: usize) -> Option<SettingsItem> {
        Self::all().into_iter().nth(index)
    }
}

/// Application state.
pub struct App {
    pub buffer: Buffer,
    pub palette: Palette,
    pub focus_mode: FocusMode,
    pub color_profile: ColorProfile,
    pub vim_mode: Mode,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll_offset: usize,
    pub column_width: u16,
    pub chrome_visible: bool,
    pub settings_visible: bool,
    pub settings_cursor: usize,
    pub should_quit: bool,
    pub file_path: Option<PathBuf>,
    pub is_scratch: bool,
    pub dirty: bool,
    pub last_save: Option<Instant>,
    pub autosave_interval: Duration,
    pub rename_active: bool,
    pub rename_buf: String,
    pub rename_cursor: usize,
    /// Pending first key of a multi-key Normal mode sequence (e.g., 'g' for gg, 'd' for dd).
    pub pending_normal_key: Option<char>,
    /// Vertical offset for Typewriter mode rendering. When the cursor is near
    /// the top of the document, content starts this many rows down so the cursor
    /// appears vertically centered.
    pub typewriter_vertical_offset: u16,
    /// Anchor position (line, col) where Visual mode selection started.
    pub selection_anchor: Option<(usize, usize)>,
    /// Internal yank register (session-only vim register).
    pub yank_register: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            palette: Palette::default_palette(),
            focus_mode: FocusMode::Off,
            color_profile: ColorProfile::TrueColor,
            vim_mode: Mode::Normal,
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
            column_width: 60,
            chrome_visible: false,
            settings_visible: false,
            settings_cursor: 0,
            should_quit: false,
            file_path: None,
            is_scratch: false,
            dirty: false,
            last_save: None,
            autosave_interval: Duration::from_secs(3),
            rename_active: false,
            rename_buf: String::new(),
            rename_cursor: 0,
            pending_normal_key: None,
            typewriter_vertical_offset: 0,
            selection_anchor: None,
            yank_register: None,
        }
    }

    pub fn with_file(mut self, path: PathBuf, content: &str) -> Self {
        self.buffer = Buffer::from_text(content);
        self.file_path = Some(path);
        self
    }

    pub fn with_scratch_name(mut self) -> Self {
        self.file_path = Some(PathBuf::from(draft_name::generate()));
        self.is_scratch = true;
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
        if self.settings_visible {
            self.settings_cursor = self.active_palette_index();
        }
    }

    /// Switch to a different Palette.
    pub fn set_palette(&mut self, palette: Palette) {
        self.palette = palette;
    }

    /// Dismiss the Settings Layer.
    pub fn dismiss_settings(&mut self) {
        self.settings_visible = false;
        self.chrome_visible = false;
    }

    /// Find the current palette's position in Palette::all().
    pub fn active_palette_index(&self) -> usize {
        Palette::all()
            .iter()
            .position(|p| p.name == self.palette.name)
            .unwrap_or(0)
    }

    /// Move the settings cursor up (wrapping).
    pub fn settings_nav_up(&mut self) {
        let count = SettingsItem::all().len();
        if self.settings_cursor == 0 {
            self.settings_cursor = count - 1;
        } else {
            self.settings_cursor -= 1;
        }
    }

    /// Move the settings cursor down (wrapping).
    pub fn settings_nav_down(&mut self) {
        let count = SettingsItem::all().len();
        self.settings_cursor = (self.settings_cursor + 1) % count;
    }

    /// Apply the currently selected settings item.
    pub fn settings_apply(&mut self) {
        let Some(item) = SettingsItem::at(self.settings_cursor) else {
            return;
        };
        match item {
            SettingsItem::Palette(idx) => {
                if let Some(p) = Palette::all().into_iter().nth(idx) {
                    self.palette = p;
                }
            }
            SettingsItem::FocusMode(mode) => self.focus_mode = mode,
            SettingsItem::ColumnWidth => {} // adjusted via Left/Right, not Enter
            SettingsItem::File => self.rename_open(),
        }
    }

    /// Adjust column width by delta, clamped to 20–120.
    pub fn settings_adjust_column(&mut self, delta: i16) {
        let new = self.column_width as i16 + delta;
        self.column_width = new.clamp(20, 120) as u16;
    }

    /// Open inline rename: seed buffer with current filename, cursor at end.
    pub fn rename_open(&mut self) {
        let name = self
            .file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        self.rename_buf = name;
        self.rename_cursor = self.rename_buf.chars().count();
        self.rename_active = true;
    }

    /// Insert a character at cursor position (filters out `/`).
    pub fn rename_insert(&mut self, ch: char) {
        if ch == '/' {
            return;
        }
        let byte_idx = char_to_byte_index(&self.rename_buf, self.rename_cursor);
        self.rename_buf.insert(byte_idx, ch);
        self.rename_cursor += 1;
    }

    /// Delete the character before the cursor.
    pub fn rename_backspace(&mut self) {
        if self.rename_cursor == 0 {
            return;
        }
        self.rename_cursor -= 1;
        let byte_idx = char_to_byte_index(&self.rename_buf, self.rename_cursor);
        // Find the byte length of the char at this position
        let ch = self.rename_buf[byte_idx..].chars().next().unwrap();
        self.rename_buf.replace_range(byte_idx..byte_idx + ch.len_utf8(), "");
    }

    /// Move rename cursor left.
    pub fn rename_cursor_left(&mut self) {
        if self.rename_cursor > 0 {
            self.rename_cursor -= 1;
        }
    }

    /// Move rename cursor right.
    pub fn rename_cursor_right(&mut self) {
        if self.rename_cursor < self.rename_buf.chars().count() {
            self.rename_cursor += 1;
        }
    }

    /// Cancel rename, clearing state.
    pub fn rename_cancel(&mut self) {
        self.rename_active = false;
        self.rename_buf.clear();
        self.rename_cursor = 0;
    }

    /// Confirm rename: rename on disk, update file_path, clear scratch flag.
    /// Empty name is treated as cancel.
    pub fn rename_confirm(&mut self) {
        if self.rename_buf.trim().is_empty() {
            self.rename_cancel();
            return;
        }

        if let Some(old_path) = &self.file_path {
            let new_path = old_path.with_file_name(&self.rename_buf);

            // Only attempt fs::rename if old file exists on disk
            if old_path.exists() {
                if std::fs::rename(old_path, &new_path).is_err() {
                    // Stay in rename mode so user can retry or Esc
                    return;
                }
            }

            self.file_path = Some(new_path);
            if self.is_scratch {
                self.is_scratch = false;
            }
        }

        self.rename_active = false;
        self.rename_buf.clear();
        self.rename_cursor = 0;
    }

    /// Process a character key input.
    pub fn handle_char(&mut self, ch: char) {
        let action = match self.vim_mode {
            Mode::Normal => {
                // Handle multi-key sequences
                if let Some(pending) = self.pending_normal_key.take() {
                    match (pending, ch) {
                        ('g', 'g') => {
                            self.cursor_line = 0;
                            self.cursor_col = 0;
                            return;
                        }
                        ('d', 'd') => {
                            self.delete_current_line();
                            return;
                        }
                        _ => return,
                    }
                }

                if ch == 'g' || ch == 'd' {
                    self.pending_normal_key = Some(ch);
                    return;
                }

                if ch == 'q' {
                    self.should_quit = true;
                    return;
                }
                vim_bindings::handle_normal(ch)
            }
            Mode::Visual => {
                // Handle multi-key sequences (gg works in Visual)
                if let Some(pending) = self.pending_normal_key.take() {
                    match (pending, ch) {
                        ('g', 'g') => {
                            self.cursor_line = 0;
                            self.cursor_col = 0;
                            return;
                        }
                        _ => return,
                    }
                }

                if ch == 'g' {
                    self.pending_normal_key = Some(ch);
                    return;
                }

                vim_bindings::handle_visual(ch)
            }
            Mode::Insert => vim_bindings::handle_insert(ch),
        };

        self.apply_action(action);
    }

    /// Process Escape key.
    pub fn handle_escape(&mut self) {
        if self.settings_visible {
            self.dismiss_settings();
        } else if self.vim_mode == Mode::Visual {
            self.selection_anchor = None;
            self.vim_mode = Mode::Normal;
        } else if self.vim_mode == Mode::Insert {
            self.vim_mode = Mode::Normal;
        }
    }

    pub fn apply_action(&mut self, action: Action) {
        match action {
            Action::SwitchMode(mode) => {
                self.vim_mode = mode;
            }
            Action::AppendMode => {
                // Move cursor one right of current position, then enter Insert.
                // In Insert mode the cursor can sit past the last char (append position).
                let line_len = self.buffer.line(self.cursor_line).len_chars();
                if self.cursor_col < line_len {
                    self.cursor_col += 1;
                }
                self.vim_mode = Mode::Insert;
            }
            Action::AppendEndOfLine => {
                // Move to end of line content, then enter Insert
                let line_len = self.buffer.line(self.cursor_line).len_chars();
                self.cursor_col = if line_len > 0 { line_len - 1 } else { 0 };
                self.vim_mode = Mode::Insert;
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
            Action::DeleteChar => {
                let idx = self.cursor_char_index();
                let line_len = self.buffer.line(self.cursor_line).len_chars();
                // Don't delete the trailing newline
                let content_len = if line_len > 0 { line_len - 1 } else { 0 };
                if self.cursor_col < content_len {
                    self.buffer.remove(idx, idx + 1);
                    self.dirty = true;
                    self.clamp_cursor_col();
                }
            }
            Action::MoveCursor(dir) => {
                self.move_cursor(dir);
            }
            Action::LineStart => {
                self.cursor_col = 0;
            }
            Action::LineEnd => {
                let line_len = self.buffer.line(self.cursor_line).len_chars();
                self.cursor_col = if line_len > 1 { line_len - 2 } else { 0 };
            }
            Action::GotoLastLine => {
                self.cursor_line = self.buffer.len_lines().saturating_sub(1);
                self.clamp_cursor_col();
            }
            Action::WordForward => {
                self.word_forward();
            }
            Action::WordBackward => {
                self.word_backward();
            }
            Action::WordEnd => {
                self.word_end();
            }
            Action::OpenLineBelow => {
                // Insert newline at end of current line, move cursor there
                let line_len = self.buffer.line(self.cursor_line).len_chars();
                let line_end_idx = self.line_start_char_index() + line_len.saturating_sub(1);
                self.buffer.insert(line_end_idx, "\n");
                self.cursor_line += 1;
                self.cursor_col = 0;
                self.dirty = true;
                self.vim_mode = Mode::Insert;
            }
            Action::OpenLineAbove => {
                // Insert newline at start of current line, cursor on new blank line
                let line_start = self.line_start_char_index();
                self.buffer.insert(line_start, "\n");
                // cursor_line stays the same (the new line pushed content down)
                self.cursor_col = 0;
                self.dirty = true;
                self.vim_mode = Mode::Insert;
            }
            Action::EnterVisual => {
                self.selection_anchor = Some((self.cursor_line, self.cursor_col));
                self.vim_mode = Mode::Visual;
            }
            Action::Yank => {
                if let Some(text) = self.selected_text() {
                    clipboard::write_osc52(&text);
                    self.yank_register = Some(text);
                }
                self.selection_anchor = None;
                self.vim_mode = Mode::Normal;
            }
            Action::DeleteSelection => {
                if let Some((sl, sc, el, ec)) = self.selection_range() {
                    // Yank first (vim convention)
                    if let Some(text) = self.selected_text() {
                        clipboard::write_osc52(&text);
                        self.yank_register = Some(text);
                    }
                    // Delete the selection
                    let rope = self.buffer.rope();
                    let start_idx = rope.line_to_char(sl) + sc;
                    let end_idx = (rope.line_to_char(el) + ec + 1).min(rope.len_chars());
                    self.buffer.remove(start_idx, end_idx);
                    self.dirty = true;
                    // Move cursor to selection start
                    self.cursor_line = sl;
                    self.cursor_col = sc;
                    self.clamp_cursor_col();
                }
                self.selection_anchor = None;
                self.vim_mode = Mode::Normal;
            }
            Action::PasteAfter => {
                if let Some(text) = self.yank_register.clone() {
                    if text.contains('\n') {
                        // Line-wise paste: insert on next line
                        let line_len = self.buffer.line(self.cursor_line).len_chars();
                        let insert_idx = self.line_start_char_index() + line_len;
                        self.buffer.insert(insert_idx, &text);
                        self.cursor_line += 1;
                        self.cursor_col = 0;
                    } else {
                        // Char-wise paste: insert after cursor
                        let idx = self.cursor_char_index() + 1;
                        let idx = idx.min(self.buffer.rope().len_chars());
                        self.buffer.insert(idx, &text);
                        self.cursor_col += text.chars().count();
                    }
                    self.dirty = true;
                }
            }
            Action::PasteBefore => {
                if let Some(text) = self.yank_register.clone() {
                    if text.contains('\n') {
                        // Line-wise paste: insert on current line
                        let insert_idx = self.line_start_char_index();
                        self.buffer.insert(insert_idx, &text);
                        self.cursor_col = 0;
                    } else {
                        // Char-wise paste: insert before cursor
                        let idx = self.cursor_char_index();
                        self.buffer.insert(idx, &text);
                        self.cursor_col += text.chars().count().saturating_sub(1);
                    }
                    self.dirty = true;
                }
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

    /// Move cursor forward to the start of the next word (whitespace-delimited).
    fn word_forward(&mut self) {
        let text = self.buffer.rope().to_string();
        let chars: Vec<char> = text.chars().collect();
        let mut idx = self.cursor_char_index();
        let len = chars.len();

        // Skip current non-whitespace
        while idx < len && !chars[idx].is_whitespace() {
            idx += 1;
        }
        // Skip whitespace
        while idx < len && chars[idx].is_whitespace() {
            idx += 1;
        }

        // Convert absolute index back to line/col
        self.set_cursor_from_char_index(idx.min(len.saturating_sub(1)));
    }

    /// Move cursor backward to the start of the previous word.
    fn word_backward(&mut self) {
        let text = self.buffer.rope().to_string();
        let chars: Vec<char> = text.chars().collect();
        let mut idx = self.cursor_char_index();

        if idx == 0 {
            return;
        }
        idx -= 1;

        // Skip whitespace backward
        while idx > 0 && chars[idx].is_whitespace() {
            idx -= 1;
        }
        // Skip non-whitespace backward to find word start
        while idx > 0 && !chars[idx - 1].is_whitespace() {
            idx -= 1;
        }

        self.set_cursor_from_char_index(idx);
    }

    /// Move cursor forward to the end of the current/next word.
    fn word_end(&mut self) {
        let text = self.buffer.rope().to_string();
        let chars: Vec<char> = text.chars().collect();
        let mut idx = self.cursor_char_index();
        let len = chars.len();

        if idx + 1 >= len {
            return;
        }
        idx += 1;

        // Skip whitespace
        while idx < len && chars[idx].is_whitespace() {
            idx += 1;
        }
        // Move to end of word
        while idx + 1 < len && !chars[idx + 1].is_whitespace() {
            idx += 1;
        }

        self.set_cursor_from_char_index(idx.min(len.saturating_sub(1)));
    }

    /// Set cursor position from an absolute char index in the buffer.
    fn set_cursor_from_char_index(&mut self, char_idx: usize) {
        let rope = self.buffer.rope();
        let total_chars = rope.len_chars();
        let idx = char_idx.min(total_chars.saturating_sub(1));
        self.cursor_line = rope.char_to_line(idx);
        let line_start = rope.line_to_char(self.cursor_line);
        self.cursor_col = idx - line_start;
    }

    /// Delete the entire current line (dd). Yanks to register and clipboard.
    fn delete_current_line(&mut self) {
        let total = self.buffer.len_lines();
        if total == 0 {
            return;
        }
        let line_start = self.line_start_char_index();
        let line_len = self.buffer.line(self.cursor_line).len_chars();
        if line_len == 0 {
            return;
        }

        // Yank deleted line to register and clipboard
        let deleted = self.buffer.rope().slice(line_start..line_start + line_len).to_string();
        self.yank_register = Some(deleted.clone());
        clipboard::write_osc52(&deleted);

        self.buffer.remove(line_start, line_start + line_len);
        self.dirty = true;

        // Adjust cursor if we deleted the last line
        if self.cursor_line >= self.buffer.len_lines() {
            self.cursor_line = self.buffer.len_lines().saturating_sub(1);
        }
        self.clamp_cursor_col();
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

    /// Returns the normalized selection range (start_line, start_col, end_line, end_col).
    /// Normalizes so start <= end regardless of anchor vs cursor order.
    pub fn selection_range(&self) -> Option<(usize, usize, usize, usize)> {
        let (anchor_line, anchor_col) = self.selection_anchor?;
        let (cl, cc) = (self.cursor_line, self.cursor_col);
        if (anchor_line, anchor_col) <= (cl, cc) {
            Some((anchor_line, anchor_col, cl, cc))
        } else {
            Some((cl, cc, anchor_line, anchor_col))
        }
    }

    /// Extract the selected text from the buffer using the current selection range.
    pub fn selected_text(&self) -> Option<String> {
        let (sl, sc, el, ec) = self.selection_range()?;
        let rope = self.buffer.rope();
        let start_idx = rope.line_to_char(sl) + sc;
        // Include the character at end_col
        let end_idx = rope.line_to_char(el) + ec + 1;
        let end_idx = end_idx.min(rope.len_chars());
        if start_idx >= end_idx {
            return None;
        }
        Some(rope.slice(start_idx..end_idx).to_string())
    }

    /// Find the sentence boundaries containing the cursor.
    /// Returns (start, end) as absolute char indices into the buffer.
    pub fn sentence_bounds(&self) -> Option<(usize, usize)> {
        let text = self.buffer.rope().to_string();
        let cursor_idx = self.cursor_char_index();
        focus_mode::sentence_bounds_at(&text, cursor_idx)
    }

    /// Compute visual lines for the current buffer and column width.
    pub fn visual_lines(&self) -> Vec<VisualLine> {
        wrap::visual_lines_for_buffer(&self.buffer, self.column_width)
    }

    /// Adjust scroll_offset so the cursor stays visible within the given height.
    /// Accepts pre-computed visual lines to avoid redundant computation.
    pub fn ensure_cursor_visible(&mut self, visual_lines: &[VisualLine], visible_height: u16) {
        let height = visible_height as usize;
        if height == 0 {
            return;
        }

        // Find the cursor's visual line position
        let mut cursor_vl = 0;
        let mut found = false;
        for (vl_index, vl) in visual_lines.iter().enumerate() {
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
        }

        // If not found in any range, use the last match from end-of-line check
        let _ = found;

        if self.focus_mode == FocusMode::Typewriter {
            // Typewriter mode: keep cursor centered vertically
            let center = height / 2;
            if cursor_vl >= center {
                // Enough content above — scroll so cursor lands at center
                self.scroll_offset = cursor_vl - center;
                self.typewriter_vertical_offset = 0;
            } else {
                // Near top of document — push content down so cursor is centered
                self.scroll_offset = 0;
                self.typewriter_vertical_offset = (center - cursor_vl) as u16;
            }
        } else {
            self.typewriter_vertical_offset = 0;
            // Edge-scrolling: only adjust when cursor would be off-screen
            if cursor_vl < self.scroll_offset {
                self.scroll_offset = cursor_vl;
            } else if cursor_vl >= self.scroll_offset + height {
                self.scroll_offset = cursor_vl - height + 1;
            }
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    // === Unit test: SettingsItem::all() matches expected count ===

    #[test]
    fn settings_item_count_matches_expected() {
        // 3 palettes + 4 focus modes + 1 column width + 1 file = 9
        assert_eq!(SettingsItem::all().len(), 9);
    }

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

    // === Acceptance test: Vim navigation motions ===

    #[test]
    fn w_moves_to_next_word() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("hello world\n");
        app.cursor_col = 0;
        app.handle_char('w');
        assert_eq!(app.cursor_col, 6); // "world"
    }

    #[test]
    fn b_moves_to_previous_word() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("hello world\n");
        app.cursor_col = 8;
        app.handle_char('b');
        assert_eq!(app.cursor_col, 6); // start of "world"
    }

    #[test]
    fn e_moves_to_end_of_word() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("hello world\n");
        app.cursor_col = 0;
        app.handle_char('e');
        assert_eq!(app.cursor_col, 4); // 'o' in "hello"
    }

    #[test]
    fn zero_moves_to_line_start() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("hello world\n");
        app.cursor_col = 5;
        app.handle_char('0');
        assert_eq!(app.cursor_col, 0);
    }

    #[test]
    fn dollar_moves_to_line_end() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("hello world\n");
        app.cursor_col = 0;
        app.handle_char('$');
        assert_eq!(app.cursor_col, 10); // last char before newline
    }

    #[test]
    fn g_moves_to_last_line() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("line one\nline two\nline three\n");
        app.cursor_line = 0;
        app.handle_char('G');
        // Last line is the empty line after trailing newline (line 3)
        assert_eq!(app.cursor_line, app.buffer.len_lines() - 1);
    }

    #[test]
    fn x_deletes_char_under_cursor() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("abc\n");
        app.cursor_col = 1;
        app.handle_char('x');
        assert_eq!(app.buffer.rope().to_string(), "ac\n");
        assert!(app.dirty);
    }

    #[test]
    fn o_opens_line_below() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("first\nsecond\n");
        app.cursor_line = 0;
        app.handle_char('o');
        assert_eq!(app.vim_mode, Mode::Insert);
        assert_eq!(app.cursor_line, 1);
        assert_eq!(app.cursor_col, 0);
        assert_eq!(app.buffer.len_lines(), 4); // first, new blank, second, trailing
    }

    #[test]
    fn big_o_opens_line_above() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("first\nsecond\n");
        app.cursor_line = 1;
        app.handle_char('O');
        assert_eq!(app.vim_mode, Mode::Insert);
        assert_eq!(app.cursor_line, 1); // stays on what is now the blank line
        assert_eq!(app.cursor_col, 0);
        assert_eq!(app.buffer.len_lines(), 4);
    }

    #[test]
    fn big_a_moves_to_end_and_enters_insert() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("hello\n");
        app.cursor_col = 0;
        app.handle_char('A');
        assert_eq!(app.vim_mode, Mode::Insert);
        // line has 6 chars (h,e,l,l,o,\n), max_col = 5
        assert_eq!(app.cursor_col, 5);
    }

    // === Acceptance test: Multi-key sequences ===

    #[test]
    fn gg_goes_to_top() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("line one\nline two\nline three\n");
        app.cursor_line = 2;
        app.cursor_col = 3;
        app.handle_char('g');
        app.handle_char('g');
        assert_eq!(app.cursor_line, 0);
        assert_eq!(app.cursor_col, 0);
    }

    #[test]
    fn dd_deletes_line() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("first\nsecond\nthird\n");
        app.cursor_line = 1;
        app.handle_char('d');
        app.handle_char('d');
        let text = app.buffer.rope().to_string();
        assert!(!text.contains("second"), "Line should be deleted, got: {}", text);
        assert!(app.dirty);
    }

    #[test]
    fn unknown_second_key_is_harmless() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("hello\n");
        app.cursor_col = 2;
        let before = app.buffer.rope().to_string();
        app.handle_char('g');
        app.handle_char('z'); // unknown
        assert_eq!(app.buffer.rope().to_string(), before);
        assert_eq!(app.cursor_col, 2); // unchanged
    }

    // === Acceptance test: Typewriter mode centering ===

    #[test]
    fn typewriter_mode_centers_cursor() {
        let mut app = App::new();
        // Create a buffer with 20 lines
        let text = (0..20).map(|i| format!("Line {}\n", i)).collect::<String>();
        app.buffer = Buffer::from_text(&text);
        app.focus_mode = FocusMode::Typewriter;
        app.cursor_line = 10;
        app.cursor_col = 0;

        let visual_lines = app.visual_lines();
        app.ensure_cursor_visible(&visual_lines, 10); // height 10

        // Cursor at visual line 10, height 10 → scroll_offset = 10 - 5 = 5
        assert_eq!(app.scroll_offset, 5);
        assert_eq!(app.typewriter_vertical_offset, 0);
    }

    #[test]
    fn typewriter_mode_at_top_uses_vertical_offset() {
        let mut app = App::new();
        let text = (0..20).map(|i| format!("Line {}\n", i)).collect::<String>();
        app.buffer = Buffer::from_text(&text);
        app.focus_mode = FocusMode::Typewriter;
        app.cursor_line = 1;
        app.cursor_col = 0;

        let visual_lines = app.visual_lines();
        app.ensure_cursor_visible(&visual_lines, 10);

        // Cursor at visual line 1, center = 5
        // Not enough content above → scroll stays 0, vertical offset pushes down
        assert_eq!(app.scroll_offset, 0);
        assert_eq!(app.typewriter_vertical_offset, 4); // center(5) - cursor_vl(1)
    }

    // === Acceptance test: Vim append mode ===

    #[test]
    fn a_enters_insert_with_cursor_one_right() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("hello\n");
        app.cursor_line = 0;
        app.cursor_col = 2; // on 'l'
        app.handle_char('a');
        assert_eq!(app.vim_mode, Mode::Insert);
        assert_eq!(app.cursor_col, 3); // moved right to after 'l'
    }

    #[test]
    fn a_at_end_of_line_enters_insert_at_end() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("hi\n");
        app.cursor_line = 0;
        app.cursor_col = 1; // on 'i', which is the last char before newline
        app.handle_char('a');
        assert_eq!(app.vim_mode, Mode::Insert);
        // max_col = len_chars - 1 = 2 (h, i, \n → 3 chars, max=2)
        // cursor was at 1, < 2, so moves to 2
        assert_eq!(app.cursor_col, 2);
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

    // === Settings Layer navigation ===

    #[test]
    fn toggle_settings_sets_cursor_to_active_palette() {
        let mut app = App::new();
        app.palette = Palette::inkwell();
        app.toggle_settings();
        assert_eq!(app.settings_cursor, 1); // Inkwell is index 1
    }

    #[test]
    fn settings_nav_down_wraps() {
        let mut app = App::new();
        app.settings_cursor = 8;
        app.settings_nav_down();
        assert_eq!(app.settings_cursor, 0);
    }

    #[test]
    fn settings_nav_up_wraps() {
        let mut app = App::new();
        app.settings_cursor = 0;
        app.settings_nav_up();
        assert_eq!(app.settings_cursor, 8);
    }

    #[test]
    fn settings_nav_down_increments() {
        let mut app = App::new();
        app.settings_cursor = 2;
        app.settings_nav_down();
        assert_eq!(app.settings_cursor, 3);
    }

    #[test]
    fn settings_nav_up_decrements() {
        let mut app = App::new();
        app.settings_cursor = 5;
        app.settings_nav_up();
        assert_eq!(app.settings_cursor, 4);
    }

    #[test]
    fn settings_apply_palette() {
        let mut app = App::new();
        app.settings_cursor = 1; // Inkwell
        app.settings_apply();
        assert_eq!(app.palette.name, "Inkwell");
    }

    #[test]
    fn settings_apply_focus_mode() {
        let mut app = App::new();
        app.settings_cursor = 4; // Sentence
        app.settings_apply();
        assert_eq!(app.focus_mode, FocusMode::Sentence);

        app.settings_cursor = 6; // Typewriter
        app.settings_apply();
        assert_eq!(app.focus_mode, FocusMode::Typewriter);
    }

    #[test]
    fn settings_apply_column_is_noop() {
        let mut app = App::new();
        app.settings_cursor = 7;
        let before = app.column_width;
        app.settings_apply();
        assert_eq!(app.column_width, before);
    }

    #[test]
    fn settings_adjust_column_increases() {
        let mut app = App::new();
        assert_eq!(app.column_width, 60);
        app.settings_adjust_column(5);
        assert_eq!(app.column_width, 65);
    }

    #[test]
    fn settings_adjust_column_clamps_low() {
        let mut app = App::new();
        app.column_width = 22;
        app.settings_adjust_column(-5);
        assert_eq!(app.column_width, 20);
    }

    #[test]
    fn settings_adjust_column_clamps_high() {
        let mut app = App::new();
        app.column_width = 118;
        app.settings_adjust_column(5);
        assert_eq!(app.column_width, 120);
    }

    #[test]
    fn active_palette_index_default_is_zero() {
        let app = App::new();
        assert_eq!(app.active_palette_index(), 0);
    }

    // === Scratch buffer ===

    #[test]
    fn scratch_sets_file_path() {
        let app = App::new().with_scratch_name();
        assert!(app.file_path.is_some());
        assert!(app.is_scratch);
    }

    #[test]
    fn scratch_enables_autosave() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = App::new();
        let name = crate::draft_name::generate();
        app.file_path = Some(dir.path().join(&name));
        app.is_scratch = true;
        app.vim_mode = Mode::Insert;
        app.handle_char('x');
        assert!(app.dirty);
        let saved = app.autosave();
        assert!(saved, "scratch buffer should autosave");
    }

    #[test]
    fn explicit_file_is_not_scratch() {
        let tmp = NamedTempFile::new().unwrap();
        let app = App::new().with_file(tmp.path().to_path_buf(), "hello");
        assert!(!app.is_scratch);
        assert!(app.file_path.is_some());
    }

    // === Inline rename ===

    #[test]
    fn rename_open_seeds_buffer_with_filename() {
        let mut app = App::new();
        app.file_path = Some(PathBuf::from("/tmp/draft.md"));
        app.rename_open();
        assert!(app.rename_active);
        assert_eq!(app.rename_buf, "draft.md");
        assert_eq!(app.rename_cursor, 8); // "draft.md".len()
    }

    #[test]
    fn rename_insert_adds_char_at_cursor() {
        let mut app = App::new();
        app.rename_active = true;
        app.rename_buf = "ab".to_string();
        app.rename_cursor = 1;
        app.rename_insert('X');
        assert_eq!(app.rename_buf, "aXb");
        assert_eq!(app.rename_cursor, 2);
    }

    #[test]
    fn rename_insert_filters_slash() {
        let mut app = App::new();
        app.rename_active = true;
        app.rename_buf = "ab".to_string();
        app.rename_cursor = 1;
        app.rename_insert('/');
        assert_eq!(app.rename_buf, "ab");
        assert_eq!(app.rename_cursor, 1);
    }

    #[test]
    fn rename_backspace_deletes_before_cursor() {
        let mut app = App::new();
        app.rename_active = true;
        app.rename_buf = "abc".to_string();
        app.rename_cursor = 2;
        app.rename_backspace();
        assert_eq!(app.rename_buf, "ac");
        assert_eq!(app.rename_cursor, 1);
    }

    #[test]
    fn rename_backspace_at_start_is_noop() {
        let mut app = App::new();
        app.rename_active = true;
        app.rename_buf = "abc".to_string();
        app.rename_cursor = 0;
        app.rename_backspace();
        assert_eq!(app.rename_buf, "abc");
        assert_eq!(app.rename_cursor, 0);
    }

    #[test]
    fn rename_cursor_left_right() {
        let mut app = App::new();
        app.rename_active = true;
        app.rename_buf = "abc".to_string();
        app.rename_cursor = 1;

        app.rename_cursor_left();
        assert_eq!(app.rename_cursor, 0);

        app.rename_cursor_left(); // at start, stays 0
        assert_eq!(app.rename_cursor, 0);

        app.rename_cursor_right();
        assert_eq!(app.rename_cursor, 1);

        app.rename_cursor = 3; // at end
        app.rename_cursor_right(); // stays at end
        assert_eq!(app.rename_cursor, 3);
    }

    #[test]
    fn rename_cancel_clears_state() {
        let mut app = App::new();
        app.file_path = Some(PathBuf::from("/tmp/draft.md"));
        app.rename_open();
        assert!(app.rename_active);

        app.rename_cancel();
        assert!(!app.rename_active);
        assert!(app.rename_buf.is_empty());
        assert_eq!(app.rename_cursor, 0);
    }

    #[test]
    fn rename_confirm_renames_on_disk() {
        let dir = tempfile::tempdir().unwrap();
        let old_path = dir.path().join("old.md");
        std::fs::write(&old_path, "content").unwrap();

        let mut app = App::new();
        app.file_path = Some(old_path.clone());
        app.rename_open();
        // Clear buffer and type new name
        app.rename_buf = "new.md".to_string();
        app.rename_cursor = 6;
        app.rename_confirm();

        assert!(!app.rename_active);
        let new_path = dir.path().join("new.md");
        assert_eq!(app.file_path, Some(new_path.clone()));
        assert!(new_path.exists());
        assert!(!old_path.exists());
    }

    #[test]
    fn rename_confirm_empty_name_cancels() {
        let mut app = App::new();
        app.file_path = Some(PathBuf::from("/tmp/draft.md"));
        app.rename_open();
        app.rename_buf = "".to_string();
        app.rename_cursor = 0;
        app.rename_confirm();

        assert!(!app.rename_active);
        // file_path unchanged
        assert_eq!(app.file_path, Some(PathBuf::from("/tmp/draft.md")));
    }

    #[test]
    fn rename_confirm_clears_scratch_flag() {
        let dir = tempfile::tempdir().unwrap();
        let old_path = dir.path().join("scratch.md");
        std::fs::write(&old_path, "").unwrap();

        let mut app = App::new();
        app.file_path = Some(old_path);
        app.is_scratch = true;
        app.rename_open();
        app.rename_buf = "real.md".to_string();
        app.rename_cursor = 7;
        app.rename_confirm();

        assert!(!app.is_scratch);
    }

    #[test]
    fn settings_apply_file_opens_rename() {
        let mut app = App::new();
        app.file_path = Some(PathBuf::from("/tmp/draft.md"));
        app.settings_cursor = 8;
        app.settings_apply();
        assert!(app.rename_active);
        assert_eq!(app.rename_buf, "draft.md");
    }

    #[test]
    fn rename_confirm_unsaved_scratch_updates_path_without_fs_rename() {
        // File doesn't exist on disk — should just update path
        let mut app = App::new();
        app.file_path = Some(PathBuf::from("/nonexistent/dir/scratch.md"));
        app.is_scratch = true;
        app.rename_open();
        app.rename_buf = "real.md".to_string();
        app.rename_cursor = 7;
        app.rename_confirm();

        assert!(!app.rename_active);
        assert_eq!(
            app.file_path,
            Some(PathBuf::from("/nonexistent/dir/real.md"))
        );
        assert!(!app.is_scratch);
    }

    // === Visual mode tests ===

    #[test]
    fn v_enters_visual_with_correct_anchor() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("hello world\n");
        app.cursor_col = 3;
        app.handle_char('v');
        assert_eq!(app.vim_mode, Mode::Visual);
        assert_eq!(app.selection_anchor, Some((0, 3)));
    }

    #[test]
    fn movement_in_visual_preserves_anchor() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("hello world\n");
        app.cursor_col = 3;
        app.handle_char('v');
        app.handle_char('l'); // move right
        app.handle_char('l');
        assert_eq!(app.vim_mode, Mode::Visual);
        assert_eq!(app.selection_anchor, Some((0, 3))); // anchor unchanged
        assert_eq!(app.cursor_col, 5); // cursor moved
    }

    #[test]
    fn escape_clears_selection_and_returns_to_normal() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("hello\n");
        app.handle_char('v');
        assert_eq!(app.vim_mode, Mode::Visual);
        app.handle_escape();
        assert_eq!(app.vim_mode, Mode::Normal);
        assert_eq!(app.selection_anchor, None);
    }

    #[test]
    fn y_yanks_correct_text_to_register() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("hello world\n");
        app.cursor_col = 0;
        app.handle_char('v'); // anchor at (0,0)
        app.handle_char('l');
        app.handle_char('l');
        app.handle_char('l');
        app.handle_char('l'); // cursor at (0,4) = 'o'
        app.handle_char('y');
        assert_eq!(app.vim_mode, Mode::Normal);
        assert_eq!(app.yank_register, Some("hello".to_string()));
        assert_eq!(app.selection_anchor, None);
    }

    #[test]
    fn d_deletes_selection_and_yanks_to_register() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("hello world\n");
        app.cursor_col = 0;
        app.handle_char('v');
        app.handle_char('l');
        app.handle_char('l');
        app.handle_char('l');
        app.handle_char('l'); // select "hello"
        app.handle_char('d');
        assert_eq!(app.vim_mode, Mode::Normal);
        assert_eq!(app.yank_register, Some("hello".to_string()));
        assert_eq!(app.buffer.rope().to_string(), " world\n");
        assert!(app.dirty);
    }

    #[test]
    fn selection_range_normalizes_when_anchor_after_cursor() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("hello world\n");
        app.cursor_col = 8;
        app.handle_char('v'); // anchor at (0,8)
        app.handle_char('h');
        app.handle_char('h');
        app.handle_char('h'); // cursor at (0,5)
        let range = app.selection_range().unwrap();
        assert_eq!(range, (0, 5, 0, 8)); // normalized: start < end
    }

    #[test]
    fn selected_text_works_multiline() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("hello\nworld\n");
        app.cursor_col = 3;
        app.handle_char('v'); // anchor at (0,3)
        app.handle_char('j'); // move down to line 1
        app.handle_char('l'); // cursor at (1,4)
        let text = app.selected_text().unwrap();
        assert_eq!(text, "lo\nworld");
    }

    #[test]
    fn gg_works_in_visual_mode() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("first\nsecond\nthird\n");
        app.cursor_line = 2;
        app.handle_char('v');
        app.handle_char('g');
        app.handle_char('g');
        assert_eq!(app.cursor_line, 0);
        assert_eq!(app.vim_mode, Mode::Visual); // stays in Visual
    }

    #[test]
    fn q_does_not_quit_in_visual_mode() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("hello\n");
        app.handle_char('v');
        app.handle_char('q'); // should be a no-op, not quit
        assert!(!app.should_quit);
        assert_eq!(app.vim_mode, Mode::Visual);
    }

    // === Paste tests ===

    #[test]
    fn p_inserts_register_content_after_cursor_charwise() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("ab\n");
        app.cursor_col = 0;
        app.yank_register = Some("XY".to_string());
        app.handle_char('p');
        assert_eq!(app.buffer.rope().to_string(), "aXYb\n");
        assert!(app.dirty);
    }

    #[test]
    fn big_p_inserts_before_cursor_charwise() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("ab\n");
        app.cursor_col = 1;
        app.yank_register = Some("XY".to_string());
        app.handle_char('P');
        assert_eq!(app.buffer.rope().to_string(), "aXYb\n");
    }

    #[test]
    fn p_multiline_inserts_on_next_line() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("first\nsecond\n");
        app.cursor_line = 0;
        app.yank_register = Some("new\n".to_string());
        app.handle_char('p');
        let text = app.buffer.rope().to_string();
        assert_eq!(text, "first\nnew\nsecond\n");
        assert_eq!(app.cursor_line, 1);
    }

    #[test]
    fn big_p_multiline_inserts_on_current_line() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("first\nsecond\n");
        app.cursor_line = 1;
        app.yank_register = Some("new\n".to_string());
        app.handle_char('P');
        let text = app.buffer.rope().to_string();
        assert_eq!(text, "first\nnew\nsecond\n");
    }

    #[test]
    fn empty_register_paste_is_noop() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("hello\n");
        app.yank_register = None;
        let before = app.buffer.rope().to_string();
        app.handle_char('p');
        assert_eq!(app.buffer.rope().to_string(), before);
        assert!(!app.dirty);
    }

    #[test]
    fn dd_populates_register_with_deleted_line() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("first\nsecond\nthird\n");
        app.cursor_line = 1;
        app.handle_char('d');
        app.handle_char('d');
        assert_eq!(app.yank_register, Some("second\n".to_string()));
        assert!(!app.buffer.rope().to_string().contains("second"));
    }

    #[test]
    fn yank_then_paste_round_trip() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("hello world\n");
        app.cursor_col = 0;
        // Select "hello"
        app.handle_char('v');
        app.handle_char('l');
        app.handle_char('l');
        app.handle_char('l');
        app.handle_char('l');
        app.handle_char('y');
        // Move to end of "world"
        app.handle_char('$');
        // Paste after
        app.handle_char('p');
        let text = app.buffer.rope().to_string();
        assert!(text.contains("hello"), "Yanked text should be pasted");
    }
}
