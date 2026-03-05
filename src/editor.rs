use crossterm::event::{KeyCode, KeyModifiers};

use crate::buffer::Buffer;
use crate::clipboard;
use crate::editing_mode::EditingMode;
use crate::smart_typography;
use crate::undo::UndoHistory;
use crate::vim_bindings::{self, Action, CursorShape, Direction, Mode};
use crate::wrap::{self, VisualLine};

/// Cached paragraph bounds, keyed on (buffer_version, cursor_line).
/// Horizontal cursor movement does not invalidate paragraph bounds.
struct ParagraphBoundsCache {
    key: (u64, usize), // (buffer_version, cursor_line)
    bounds: Option<(usize, usize)>,
}

/// Text editor core: buffer, cursor, undo, selection, and vim state.
pub struct Editor {
    pub buffer: Buffer,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub vim_mode: Mode,
    pub editing_mode: EditingMode,
    pub pending_normal_key: Option<char>,
    pub selection_anchor: Option<(usize, usize)>,
    pub yank_register: Option<String>,
    pub undo_history: UndoHistory,
    pub dirty: bool,
    paragraph_cache: Option<ParagraphBoundsCache>,
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

impl Editor {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            cursor_line: 0,
            cursor_col: 0,
            vim_mode: Mode::Normal,
            editing_mode: EditingMode::default(),
            pending_normal_key: None,
            selection_anchor: None,
            yank_register: None,
            undo_history: UndoHistory::new(),
            dirty: false,
            paragraph_cache: None,
        }
    }

    /// The cursor shape based on current editing and vim mode.
    pub fn cursor_shape(&self) -> CursorShape {
        if self.editing_mode == EditingMode::Standard {
            return CursorShape::Bar;
        }
        self.vim_mode.cursor_shape()
    }

    /// Handle a key press for editor-level input (not overlays or Ctrl combos).
    /// Returns true if the app should quit.
    pub fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers, column_width: u16) -> bool {
        let is_standard = self.editing_mode == EditingMode::Standard;

        // Shift+Arrow/Home/End extends selection
        if modifiers.contains(KeyModifiers::SHIFT) {
            match code {
                KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down
                | KeyCode::Home | KeyCode::End => {
                    self.extend_selection(code, column_width);
                    return false;
                }
                _ => {}
            }
        }

        match code {
            KeyCode::Esc => { self.handle_escape(); return false; }
            KeyCode::Char(c) => return self.handle_char(c),
            KeyCode::Backspace => {
                if is_standard || self.vim_mode == Mode::Insert {
                    if is_standard && self.selection_anchor.is_some() {
                        self.delete_selection_silent();
                        self.selection_anchor = None;
                    } else {
                        self.apply_action(Action::DeleteBack);
                    }
                }
            }
            KeyCode::Enter => {
                if is_standard || self.vim_mode == Mode::Insert {
                    if is_standard && self.selection_anchor.is_some() {
                        self.delete_selection_silent();
                        self.selection_anchor = None;
                    }
                    self.apply_action(Action::InsertNewline);
                }
            }
            KeyCode::Delete => {
                self.apply_action(Action::DeleteChar);
            }
            KeyCode::Home => {
                if is_standard {
                    self.selection_anchor = None;
                }
                self.apply_action(Action::LineStart);
            }
            KeyCode::End => {
                if is_standard {
                    self.selection_anchor = None;
                }
                self.apply_action(Action::LineEnd);
            }
            KeyCode::Left => {
                if is_standard {
                    self.selection_anchor = None;
                }
                self.move_cursor_with_width(Direction::Left, column_width);
            }
            KeyCode::Right => {
                if is_standard {
                    self.selection_anchor = None;
                }
                self.move_cursor_with_width(Direction::Right, column_width);
            }
            KeyCode::Up => {
                if is_standard {
                    self.selection_anchor = None;
                }
                self.move_cursor_with_width(Direction::Up, column_width);
            }
            KeyCode::Down => {
                if is_standard {
                    self.selection_anchor = None;
                }
                self.move_cursor_with_width(Direction::Down, column_width);
            }
            _ => {}
        }
        false
    }

    /// Process a character key input. Returns true if the app should quit.
    pub fn handle_char(&mut self, ch: char) -> bool {
        // Standard mode: always insert characters directly
        if self.editing_mode == EditingMode::Standard {
            if ch.is_control() {
                return false;
            }
            // Selection replaces on type
            if self.selection_anchor.is_some() {
                self.delete_selection_silent();
                self.selection_anchor = None;
            }
            self.insert_char(ch);
            return false;
        }

        let action = match self.vim_mode {
            Mode::Normal => {
                if ch == 'q' {
                    return true;
                }
                let pending = self.pending_normal_key.take();
                let (action, new_pending) = vim_bindings::handle_normal_with_pending(ch, pending);
                self.pending_normal_key = new_pending;
                action
            }
            Mode::Visual => {
                let pending = self.pending_normal_key.take();
                let (action, new_pending) = vim_bindings::handle_visual_with_pending(ch, pending);
                self.pending_normal_key = new_pending;
                action
            }
            Mode::Insert => vim_bindings::handle_insert(ch),
        };

        self.apply_action(action);
        false
    }

    /// Process Escape key.
    pub fn handle_escape(&mut self) {
        if self.editing_mode == EditingMode::Standard {
            // In Standard mode, Escape just clears selection
            self.selection_anchor = None;
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
                // In Standard mode, vim_mode must stay Insert
                if self.editing_mode != EditingMode::Standard {
                    self.vim_mode = mode;
                }
            }
            Action::AppendMode => {
                let line_len = self.buffer.line(self.cursor_line).len_chars();
                if self.cursor_col < line_len {
                    self.cursor_col += 1;
                }
                self.vim_mode = Mode::Insert;
            }
            Action::AppendEndOfLine => {
                self.cursor_col = self.line_content_len(self.cursor_line);
                self.vim_mode = Mode::Insert;
            }
            Action::InsertChar(ch) => {
                self.insert_char(ch);
            }
            Action::InsertNewline => {
                let idx = self.cursor_char_index();
                self.undo_history.commit_group();
                self.undo_history.record_insert(idx, "\n");
                self.buffer.insert(idx, "\n");
                self.cursor_line += 1;
                self.cursor_col = 0;
                self.dirty = true;
                self.undo_history.commit_group();
            }
            Action::DeleteBack => {
                let idx = self.cursor_char_index();
                if idx > 0 {
                    let deleted = self.buffer.slice_to_string(idx - 1, idx);
                    self.undo_history.record_delete(idx - 1, &deleted);
                    self.buffer.remove(idx - 1, idx);
                    if self.cursor_col > 0 {
                        self.cursor_col -= 1;
                    } else if self.cursor_line > 0 {
                        self.cursor_line -= 1;
                        self.cursor_col = self.buffer.line(self.cursor_line).len_chars().saturating_sub(1);
                        self.undo_history.commit_group();
                    }
                    self.dirty = true;
                }
            }
            Action::DeleteChar => {
                let idx = self.cursor_char_index();
                let line_len = self.buffer.line(self.cursor_line).len_chars();
                let content_len = if line_len > 0 { line_len - 1 } else { 0 };
                if self.cursor_col < content_len {
                    let deleted = self.buffer.slice_to_string(idx, idx + 1);
                    self.undo_history.record_delete(idx, &deleted);
                    self.buffer.remove(idx, idx + 1);
                    self.dirty = true;
                    self.clamp_cursor_col();
                }
            }
            Action::MoveCursor(dir) => {
                // Left/Right ignore column_width. Up/Down use 60 as a default,
                // but this path is unreachable through App for vertical moves:
                // App::try_handle_vertical_move intercepts j/k and uses
                // move_cursor_visual with the real column width.
                self.move_cursor_with_width(dir, 60);
            }
            Action::LineStart => {
                self.cursor_col = 0;
            }
            Action::LineEnd => {
                let content_len = self.line_content_len(self.cursor_line);
                self.cursor_col = if self.vim_mode == Mode::Normal {
                    content_len.saturating_sub(1)
                } else {
                    content_len
                };
            }
            Action::GotoFirstLine => {
                self.cursor_line = 0;
                self.cursor_col = 0;
            }
            Action::GotoLastLine => {
                self.cursor_line = self.buffer.len_lines().saturating_sub(1);
                self.clamp_cursor_col();
            }
            Action::DeleteLine => {
                self.delete_current_line();
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
                let line_len = self.buffer.line(self.cursor_line).len_chars();
                let line_end_idx = self.line_start_char_index() + line_len.saturating_sub(1);
                self.undo_history.commit_group();
                self.undo_history.record_insert(line_end_idx, "\n");
                self.buffer.insert(line_end_idx, "\n");
                self.cursor_line += 1;
                self.cursor_col = 0;
                self.dirty = true;
                self.undo_history.commit_group();
                self.vim_mode = Mode::Insert;
            }
            Action::OpenLineAbove => {
                let line_start = self.line_start_char_index();
                self.undo_history.commit_group();
                self.undo_history.record_insert(line_start, "\n");
                self.buffer.insert(line_start, "\n");
                self.cursor_col = 0;
                self.dirty = true;
                self.undo_history.commit_group();
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
                    if let Some(text) = self.selected_text() {
                        clipboard::write_osc52(&text);
                        self.yank_register = Some(text);
                    }
                    let start_idx = self.buffer.line_to_char(sl) + sc;
                    let end_idx = (self.buffer.line_to_char(el) + ec + 1).min(self.buffer.len_chars());
                    let deleted = self.buffer.slice_to_string(start_idx, end_idx);
                    self.undo_history.commit_group();
                    self.undo_history.record_delete(start_idx, &deleted);
                    self.undo_history.commit_group();
                    self.buffer.remove(start_idx, end_idx);
                    self.dirty = true;
                    self.cursor_line = sl;
                    self.cursor_col = sc;
                    self.clamp_cursor_col();
                }
                self.selection_anchor = None;
                self.vim_mode = Mode::Normal;
            }
            Action::PasteAfter => {
                if let Some(text) = self.resolve_paste_text() {
                    self.undo_history.commit_group();
                    if text.contains('\n') {
                        let line_len = self.buffer.line(self.cursor_line).len_chars();
                        let insert_idx = self.line_start_char_index() + line_len;
                        self.undo_history.record_insert(insert_idx, &text);
                        self.buffer.insert(insert_idx, &text);
                        self.cursor_line += 1;
                        self.cursor_col = 0;
                    } else {
                        let idx = self.cursor_char_index() + 1;
                        let idx = idx.min(self.buffer.len_chars());
                        self.undo_history.record_insert(idx, &text);
                        self.buffer.insert(idx, &text);
                        self.cursor_col += text.chars().count();
                    }
                    self.undo_history.commit_group();
                    self.dirty = true;
                }
            }
            Action::PasteBefore => {
                if let Some(text) = self.resolve_paste_text() {
                    self.undo_history.commit_group();
                    if text.contains('\n') {
                        let insert_idx = self.line_start_char_index();
                        self.undo_history.record_insert(insert_idx, &text);
                        self.buffer.insert(insert_idx, &text);
                        self.cursor_col = 0;
                    } else {
                        let idx = self.cursor_char_index();
                        self.undo_history.record_insert(idx, &text);
                        self.buffer.insert(idx, &text);
                        self.cursor_col += text.chars().count().saturating_sub(1);
                    }
                    self.undo_history.commit_group();
                    self.dirty = true;
                }
            }
            Action::PasteAtCursor => {
                if let Some(text) = self.resolve_paste_text() {
                    if self.editing_mode == EditingMode::Standard
                        && self.selection_anchor.is_some()
                    {
                        self.delete_selection_silent();
                        self.selection_anchor = None;
                    }
                    let idx = self.cursor_char_index();
                    self.undo_history.commit_group();
                    self.undo_history.record_insert(idx, &text);
                    self.buffer.insert(idx, &text);
                    self.undo_history.commit_group();
                    let char_count = text.chars().count();
                    self.set_cursor_from_char_index(idx + char_count);
                    self.dirty = true;
                }
            }
            Action::SelectAll => {
                let total_chars = self.buffer.len_chars();
                self.selection_anchor = Some((0, 0));
                if total_chars > 0 {
                    self.set_cursor_from_char_index(total_chars.saturating_sub(1));
                }
                if self.editing_mode == EditingMode::Vim {
                    self.vim_mode = Mode::Visual;
                }
            }
            Action::Undo => {
                self.undo_history.commit_group();
                if let Some(ops) = self.undo_history.undo() {
                    for op in ops.iter().rev() {
                        match op {
                            crate::undo::Operation::Insert { pos, text } => {
                                let end = pos + text.chars().count();
                                self.buffer.remove(*pos, end);
                            }
                            crate::undo::Operation::Delete { pos, text } => {
                                self.buffer.insert(*pos, text);
                            }
                        }
                    }
                    if let Some(op) = ops.first() {
                        let pos = match op {
                            crate::undo::Operation::Insert { pos, .. } => *pos,
                            crate::undo::Operation::Delete { pos, .. } => *pos,
                        };
                        self.set_cursor_from_char_index(pos);
                    }
                    self.dirty = true;
                }
            }
            Action::Redo => {
                if let Some(ops) = self.undo_history.redo() {
                    for op in &ops {
                        match op {
                            crate::undo::Operation::Insert { pos, text } => {
                                self.buffer.insert(*pos, text);
                            }
                            crate::undo::Operation::Delete { pos, text } => {
                                let end = pos + text.chars().count();
                                self.buffer.remove(*pos, end);
                            }
                        }
                    }
                    if let Some(op) = ops.last() {
                        let pos = match op {
                            crate::undo::Operation::Insert { pos, text } => {
                                pos + text.chars().count()
                            }
                            crate::undo::Operation::Delete { pos, .. } => *pos,
                        };
                        self.set_cursor_from_char_index(pos);
                    }
                    self.dirty = true;
                }
            }
            Action::None => {}
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        let idx = self.cursor_char_index();

        // Only compute preceding text for chars that smart typography acts on.
        // This avoids a String allocation for ~99% of keystrokes.
        if matches!(ch, '"' | '\'' | '-' | '.') {
            let preceding = self.preceding_text(idx);
            if let Some(edit) = smart_typography::transform(ch, &preceding) {
                if edit.delete_before > 0 {
                    let start = idx.saturating_sub(edit.delete_before);
                    let deleted = self.buffer.slice_to_string(start, idx);
                    self.undo_history.record_delete(start, &deleted);
                    self.buffer.remove(start, idx);
                    self.cursor_col -= edit.delete_before;
                }
                let new_idx = self.cursor_char_index();
                self.undo_history.record_insert(new_idx, edit.insert);
                self.buffer.insert(new_idx, edit.insert);
                self.cursor_col += edit.insert.chars().count();
                self.dirty = true;
                if ch == ' ' || ch == '\t' {
                    self.undo_history.commit_group();
                }
                return;
            }
        }

        let s = ch.to_string();
        self.undo_history.record_insert(idx, &s);
        self.buffer.insert(idx, &s);
        self.cursor_col += 1;
        self.dirty = true;

        if ch == ' ' || ch == '\t' {
            self.undo_history.commit_group();
        }
    }

    /// Get the text preceding the cursor position (on the current line).
    fn preceding_text(&self, char_idx: usize) -> String {
        let line_start = self.line_start_char_index();
        if char_idx <= line_start {
            return String::new();
        }
        self.buffer.slice_to_string(line_start, char_idx)
    }

    /// Try system clipboard first, fall back to yank register.
    /// Syncs clipboard content into yank_register so subsequent pastes use it.
    fn resolve_paste_text(&mut self) -> Option<String> {
        #[cfg(not(test))]
        if let Some(text) = clipboard::read_clipboard() {
            self.yank_register = Some(text.clone());
            return Some(text);
        }
        self.yank_register.clone()
    }

    /// Calculate the character index in the buffer for the current cursor position.
    pub fn cursor_char_index(&self) -> usize {
        self.line_start_char_index() + self.cursor_col
    }

    /// Character index of the start of the current line.
    pub fn line_start_char_index(&self) -> usize {
        self.buffer.line_to_char(self.cursor_line)
    }

    /// Number of visible characters on the given line (excludes trailing newline).
    pub fn line_content_len(&self, line: usize) -> usize {
        let slice = self.buffer.line(line);
        let len = slice.len_chars();
        if len > 0 && slice.char(len - 1) == '\n' {
            len - 1
        } else {
            len
        }
    }

    /// Find the visual line index containing (cursor_line, cursor_col).
    pub fn find_cursor_visual_index(&self, visual_lines: &[VisualLine]) -> Option<usize> {
        for (i, vl) in visual_lines.iter().enumerate() {
            if vl.logical_line == self.cursor_line
                && self.cursor_col >= vl.char_start
                && self.cursor_col < vl.char_end
            {
                return Some(i);
            }
        }
        visual_lines
            .iter()
            .rposition(|vl| vl.logical_line == self.cursor_line)
    }

    /// Move cursor in the given direction using the specified column width for wrapping.
    pub fn move_cursor_with_width(&mut self, dir: Direction, column_width: u16) {
        match dir {
            Direction::Left => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                }
            }
            Direction::Right => {
                let max_col = self.line_content_len(self.cursor_line);
                if self.cursor_col < max_col {
                    self.cursor_col += 1;
                }
            }
            Direction::Up | Direction::Down => {
                let visual_lines = wrap::visual_lines_for_buffer(&self.buffer, column_width);
                self.move_cursor_visual(dir, &visual_lines);
            }
        }
    }

    /// Move cursor up/down using pre-computed visual lines.
    pub fn move_cursor_visual(&mut self, dir: Direction, visual_lines: &[VisualLine]) {
        let Some(cur_idx) = self.find_cursor_visual_index(visual_lines) else {
            return;
        };

        let target_idx = if dir == Direction::Up {
            if cur_idx == 0 {
                return;
            }
            cur_idx - 1
        } else {
            if cur_idx + 1 >= visual_lines.len() {
                return;
            }
            cur_idx + 1
        };

        let cur_vl = &visual_lines[cur_idx];
        let target_vl = &visual_lines[target_idx];
        let visual_col = self.cursor_col - cur_vl.char_start;
        let target_len = target_vl.char_end - target_vl.char_start;

        self.cursor_line = target_vl.logical_line;
        self.cursor_col = target_vl.char_start + visual_col.min(target_len);
    }

    /// Move cursor forward to the start of the next word.
    fn word_forward(&mut self) {
        let mut idx = self.cursor_char_index();
        let len = self.buffer.len_chars();

        while idx < len && !self.buffer.char_at(idx).is_whitespace() {
            idx += 1;
        }
        while idx < len && self.buffer.char_at(idx).is_whitespace() {
            idx += 1;
        }

        self.set_cursor_from_char_index(idx.min(len.saturating_sub(1)));
    }

    /// Move cursor backward to the start of the previous word.
    fn word_backward(&mut self) {
        let mut idx = self.cursor_char_index();

        if idx == 0 {
            return;
        }
        idx -= 1;

        while idx > 0 && self.buffer.char_at(idx).is_whitespace() {
            idx -= 1;
        }
        while idx > 0 && !self.buffer.char_at(idx - 1).is_whitespace() {
            idx -= 1;
        }

        self.set_cursor_from_char_index(idx);
    }

    /// Move cursor forward to the end of the current/next word.
    fn word_end(&mut self) {
        let mut idx = self.cursor_char_index();
        let len = self.buffer.len_chars();

        if idx + 1 >= len {
            return;
        }
        idx += 1;

        while idx < len && self.buffer.char_at(idx).is_whitespace() {
            idx += 1;
        }
        while idx + 1 < len && !self.buffer.char_at(idx + 1).is_whitespace() {
            idx += 1;
        }

        self.set_cursor_from_char_index(idx.min(len.saturating_sub(1)));
    }

    /// Set cursor position from an absolute char index in the buffer.
    pub fn set_cursor_from_char_index(&mut self, char_idx: usize) {
        let total_chars = self.buffer.len_chars();
        let idx = char_idx.min(total_chars.saturating_sub(1));
        self.cursor_line = self.buffer.char_to_line(idx);
        let line_start = self.buffer.line_to_char(self.cursor_line);
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

        let deleted = self.buffer.slice_to_string(line_start, line_start + line_len);
        self.yank_register = Some(deleted.clone());
        clipboard::write_osc52(&deleted);

        self.undo_history.commit_group();
        self.undo_history.record_delete(line_start, &deleted);
        self.undo_history.commit_group();

        self.buffer.remove(line_start, line_start + line_len);
        self.dirty = true;

        if self.cursor_line >= self.buffer.len_lines() {
            self.cursor_line = self.buffer.len_lines().saturating_sub(1);
        }
        self.clamp_cursor_col();
    }

    /// Extend selection by moving the cursor while keeping (or setting) the anchor.
    pub fn extend_selection(&mut self, code: KeyCode, column_width: u16) {
        if self.selection_anchor.is_none() {
            self.selection_anchor = Some((self.cursor_line, self.cursor_col));
        }

        match code {
            KeyCode::Left => self.move_cursor_with_width(Direction::Left, column_width),
            KeyCode::Right => self.move_cursor_with_width(Direction::Right, column_width),
            KeyCode::Up => self.move_cursor_with_width(Direction::Up, column_width),
            KeyCode::Down => self.move_cursor_with_width(Direction::Down, column_width),
            KeyCode::Home => self.cursor_col = 0,
            KeyCode::End => {
                self.cursor_col = self.line_content_len(self.cursor_line);
            }
            _ => {}
        }

        if self.editing_mode == EditingMode::Vim && self.vim_mode != Mode::Visual {
            self.vim_mode = Mode::Visual;
        }
    }

    /// Extend selection using pre-computed visual lines for Up/Down movement.
    pub fn extend_selection_visual(&mut self, dir: Direction, visual_lines: &[VisualLine]) {
        if self.selection_anchor.is_none() {
            self.selection_anchor = Some((self.cursor_line, self.cursor_col));
        }
        self.move_cursor_visual(dir, visual_lines);
        if self.editing_mode == EditingMode::Vim && self.vim_mode != Mode::Visual {
            self.vim_mode = Mode::Visual;
        }
    }

    /// Delete the current selection without yanking (for Standard mode replace-on-type).
    pub fn delete_selection_silent(&mut self) {
        if let Some((sl, sc, el, ec)) = self.selection_range() {
            let start_idx = self.buffer.line_to_char(sl) + sc;
            let end_idx = (self.buffer.line_to_char(el) + ec + 1).min(self.buffer.len_chars());
            let deleted = self.buffer.slice_to_string(start_idx, end_idx);
            self.undo_history.commit_group();
            self.undo_history.record_delete(start_idx, &deleted);
            self.undo_history.commit_group();
            self.buffer.remove(start_idx, end_idx);
            self.dirty = true;
            self.cursor_line = sl;
            self.cursor_col = sc;
            self.clamp_cursor_col();
        }
    }

    pub fn clamp_cursor_col(&mut self) {
        let max_col = self.line_content_len(self.cursor_line);
        if self.cursor_col > max_col {
            self.cursor_col = max_col;
        }
    }

    /// Find the paragraph bounds (start_line, end_line) containing the cursor.
    /// Uses rope slice iteration to avoid String allocation.
    pub fn paragraph_bounds(&self) -> Option<(usize, usize)> {
        let total = self.buffer.len_lines();
        if total == 0 {
            return None;
        }

        let mut start = self.cursor_line;
        while start > 0 {
            if self.buffer.line(start - 1).chars().all(|c| c.is_whitespace()) {
                break;
            }
            start -= 1;
        }

        let mut end = self.cursor_line;
        while end + 1 < total {
            if self.buffer.line(end + 1).chars().all(|c| c.is_whitespace()) {
                break;
            }
            end += 1;
        }

        Some((start, end))
    }

    /// Ensure paragraph bounds cache is fresh. Only depends on buffer version and cursor line.
    fn ensure_paragraph_cached(&mut self) {
        let key = (self.buffer.version(), self.cursor_line);
        if let Some(ref cache) = self.paragraph_cache {
            if cache.key == key {
                return;
            }
        }
        let bounds = self.paragraph_bounds();
        self.paragraph_cache = Some(ParagraphBoundsCache { key, bounds });
    }

    /// Cached paragraph bounds — recomputes only when cursor line or buffer changes.
    pub fn paragraph_bounds_cached(&mut self) -> Option<(usize, usize)> {
        self.ensure_paragraph_cached();
        self.paragraph_cache.as_ref().unwrap().bounds
    }

    /// Returns the normalized selection range (start_line, start_col, end_line, end_col).
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
        let start_idx = self.buffer.line_to_char(sl) + sc;
        let end_idx = self.buffer.line_to_char(el) + ec + 1;
        let end_idx = end_idx.min(self.buffer.len_chars());
        if start_idx >= end_idx {
            return None;
        }
        Some(self.buffer.slice_to_string(start_idx, end_idx))
    }

    /// Replace the buffer with new content and reset all editing state.
    pub fn reset_to_content(&mut self, content: &str) {
        self.buffer = Buffer::from_text(content);
        self.dirty = false;
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.undo_history = UndoHistory::new();
        self.selection_anchor = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editing_mode::EditingMode;
    use crate::vim_bindings::{Action, Mode};

    // === Vim mode switch ===

    #[test]
    fn i_enters_insert_and_escape_returns_to_normal() {
        let mut editor = Editor::new();
        assert_eq!(editor.vim_mode, Mode::Normal);
        editor.handle_char('i');
        assert_eq!(editor.vim_mode, Mode::Insert);
        editor.handle_escape();
        assert_eq!(editor.vim_mode, Mode::Normal);
    }

    // === Vim navigation motions ===

    #[test]
    fn w_moves_to_next_word() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("hello world\n");
        editor.cursor_col = 0;
        editor.handle_char('w');
        assert_eq!(editor.cursor_col, 6);
    }

    #[test]
    fn b_moves_to_previous_word() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("hello world\n");
        editor.cursor_col = 8;
        editor.handle_char('b');
        assert_eq!(editor.cursor_col, 6);
    }

    #[test]
    fn e_moves_to_end_of_word() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("hello world\n");
        editor.cursor_col = 0;
        editor.handle_char('e');
        assert_eq!(editor.cursor_col, 4);
    }

    #[test]
    fn zero_moves_to_line_start() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("hello world\n");
        editor.cursor_col = 5;
        editor.handle_char('0');
        assert_eq!(editor.cursor_col, 0);
    }

    #[test]
    fn dollar_moves_to_line_end() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("hello world\n");
        editor.cursor_col = 0;
        editor.handle_char('$');
        assert_eq!(editor.cursor_col, 10);
    }

    #[test]
    fn g_moves_to_last_line() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("line one\nline two\nline three\n");
        editor.cursor_line = 0;
        editor.handle_char('G');
        assert_eq!(editor.cursor_line, editor.buffer.len_lines() - 1);
    }

    #[test]
    fn x_deletes_char_under_cursor() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("abc\n");
        editor.cursor_col = 1;
        editor.handle_char('x');
        assert_eq!(editor.buffer.to_string(), "ac\n");
        assert!(editor.dirty);
    }

    #[test]
    fn o_opens_line_below() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("first\nsecond\n");
        editor.cursor_line = 0;
        editor.handle_char('o');
        assert_eq!(editor.vim_mode, Mode::Insert);
        assert_eq!(editor.cursor_line, 1);
        assert_eq!(editor.cursor_col, 0);
        assert_eq!(editor.buffer.len_lines(), 4);
    }

    #[test]
    fn big_o_opens_line_above() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("first\nsecond\n");
        editor.cursor_line = 1;
        editor.handle_char('O');
        assert_eq!(editor.vim_mode, Mode::Insert);
        assert_eq!(editor.cursor_line, 1);
        assert_eq!(editor.cursor_col, 0);
        assert_eq!(editor.buffer.len_lines(), 4);
    }

    #[test]
    fn big_a_moves_to_end_and_enters_insert() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("hello\n");
        editor.cursor_col = 0;
        editor.handle_char('A');
        assert_eq!(editor.vim_mode, Mode::Insert);
        assert_eq!(editor.cursor_col, 5);
    }

    // === Multi-key sequences ===

    #[test]
    fn gg_goes_to_top() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("line one\nline two\nline three\n");
        editor.cursor_line = 2;
        editor.cursor_col = 3;
        editor.handle_char('g');
        editor.handle_char('g');
        assert_eq!(editor.cursor_line, 0);
        assert_eq!(editor.cursor_col, 0);
    }

    #[test]
    fn dd_deletes_line() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("first\nsecond\nthird\n");
        editor.cursor_line = 1;
        editor.handle_char('d');
        editor.handle_char('d');
        assert!(!editor.buffer.to_string().contains("second"));
        assert!(editor.dirty);
    }

    #[test]
    fn unknown_second_key_is_harmless() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("hello\n");
        editor.cursor_col = 2;
        let before = editor.buffer.to_string();
        editor.handle_char('g');
        editor.handle_char('z');
        assert_eq!(editor.buffer.to_string(), before);
        assert_eq!(editor.cursor_col, 2);
    }

    // === Vim append mode ===

    #[test]
    fn a_enters_insert_with_cursor_one_right() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("hello\n");
        editor.cursor_line = 0;
        editor.cursor_col = 2;
        editor.handle_char('a');
        assert_eq!(editor.vim_mode, Mode::Insert);
        assert_eq!(editor.cursor_col, 3);
    }

    #[test]
    fn a_at_end_of_line_enters_insert_at_end() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("hi\n");
        editor.cursor_line = 0;
        editor.cursor_col = 1;
        editor.handle_char('a');
        assert_eq!(editor.vim_mode, Mode::Insert);
        assert_eq!(editor.cursor_col, 2);
    }

    // === Smart typography ===

    #[test]
    fn smart_quotes_applied_during_insert() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("He said ");
        editor.cursor_line = 0;
        editor.cursor_col = 8;
        editor.vim_mode = Mode::Insert;
        editor.handle_char('"');
        let text = editor.buffer.to_string();
        assert!(text.contains('\u{201C}'), "Should have opening curly quote, got: {}", text);
    }

    // === Cursor movement ===

    #[test]
    fn h_moves_cursor_left() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("line one\nline two\nline three");
        editor.cursor_line = 1;
        editor.cursor_col = 3;
        editor.handle_char('h');
        assert_eq!(editor.cursor_col, 2, "h should move cursor left by one column");
    }

    #[test]
    fn l_moves_cursor_right() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("line one\nline two\nline three");
        editor.cursor_line = 1;
        editor.cursor_col = 3;
        editor.handle_char('l');
        assert_eq!(editor.cursor_col, 4, "l should move cursor right by one column");
    }

    #[test]
    fn k_moves_cursor_up() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("line one\nline two\nline three");
        editor.cursor_line = 1;
        editor.cursor_col = 3;
        editor.handle_char('k');
        assert_eq!(editor.cursor_line, 0, "k should move cursor up by one line");
    }

    #[test]
    fn j_moves_cursor_down() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("line one\nline two\nline three");
        editor.cursor_line = 1;
        editor.cursor_col = 3;
        editor.handle_char('j');
        assert_eq!(editor.cursor_line, 2, "j should move cursor down by one line");
    }

    // === Standard mode tests ===

    #[test]
    fn standard_mode_typing_inserts_directly() {
        let mut editor = Editor::new();
        editor.editing_mode = EditingMode::Standard;
        editor.vim_mode = Mode::Insert;
        editor.buffer = Buffer::from_text("hello\n");
        editor.cursor_col = 5;
        editor.handle_char('!');
        assert_eq!(editor.buffer.to_string(), "hello!\n");
    }

    #[test]
    fn standard_mode_escape_clears_selection_not_mode() {
        let mut editor = Editor::new();
        editor.editing_mode = EditingMode::Standard;
        editor.vim_mode = Mode::Insert;
        editor.selection_anchor = Some((0, 2));
        editor.handle_escape();
        assert_eq!(editor.selection_anchor, None);
        assert_eq!(editor.vim_mode, Mode::Insert);
    }

    #[test]
    fn standard_mode_cursor_is_always_bar() {
        let mut editor = Editor::new();
        editor.editing_mode = EditingMode::Standard;
        editor.vim_mode = Mode::Insert;
        assert_eq!(editor.cursor_shape(), CursorShape::Bar);
    }

    #[test]
    fn standard_mode_typing_replaces_selection() {
        let mut editor = Editor::new();
        editor.editing_mode = EditingMode::Standard;
        editor.vim_mode = Mode::Insert;
        editor.buffer = Buffer::from_text("hello world\n");
        editor.selection_anchor = Some((0, 0));
        editor.cursor_col = 4;
        editor.handle_char('X');
        assert_eq!(editor.buffer.to_string(), "X world\n");
        assert_eq!(editor.selection_anchor, None);
    }

    #[test]
    fn standard_mode_q_inserts_q_not_quit() {
        let mut editor = Editor::new();
        editor.editing_mode = EditingMode::Standard;
        editor.vim_mode = Mode::Insert;
        editor.buffer = Buffer::from_text("\n");
        editor.cursor_col = 0;
        let quit = editor.handle_char('q');
        assert!(!quit);
        assert!(editor.buffer.to_string().contains('q'));
    }

    // === Mode leakage prevention ===

    #[test]
    fn standard_mode_vim_mode_stays_insert() {
        let mut editor = Editor::new();
        editor.editing_mode = EditingMode::Standard;
        editor.vim_mode = Mode::Insert;
        editor.apply_action(Action::SwitchMode(Mode::Normal));
        assert_eq!(editor.vim_mode, Mode::Insert);
    }

    #[test]
    fn standard_mode_escape_keeps_insert() {
        let mut editor = Editor::new();
        editor.editing_mode = EditingMode::Standard;
        editor.vim_mode = Mode::Insert;
        editor.handle_escape();
        assert_eq!(editor.vim_mode, Mode::Insert);
    }

    #[test]
    fn standard_mode_vim_keys_insert_literally() {
        let mut editor = Editor::new();
        editor.editing_mode = EditingMode::Standard;
        editor.vim_mode = Mode::Insert;
        editor.buffer = Buffer::from_text("\n");
        editor.cursor_col = 0;
        editor.handle_char('i');
        assert!(editor.buffer.to_string().contains('i'));
        assert_eq!(editor.vim_mode, Mode::Insert);
    }

    // === Undo/Redo ===

    #[test]
    fn undo_restores_previous_state() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("hello\n");
        editor.vim_mode = Mode::Insert;
        editor.cursor_col = 5;
        editor.handle_char('!');
        editor.undo_history.commit_group();
        assert_eq!(editor.buffer.to_string(), "hello!\n");
        editor.apply_action(Action::Undo);
        assert_eq!(editor.buffer.to_string(), "hello\n");
    }

    #[test]
    fn undo_then_redo_restores_change() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("hello\n");
        editor.vim_mode = Mode::Insert;
        editor.cursor_col = 5;
        editor.handle_char('!');
        editor.undo_history.commit_group();
        editor.apply_action(Action::Undo);
        assert_eq!(editor.buffer.to_string(), "hello\n");
        editor.apply_action(Action::Redo);
        assert_eq!(editor.buffer.to_string(), "hello!\n");
    }

    #[test]
    fn multiple_undos_walk_back_through_history() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("\n");
        editor.vim_mode = Mode::Insert;
        editor.cursor_col = 0;
        editor.handle_char('a');
        editor.handle_char(' ');
        editor.handle_char('b');
        editor.handle_char(' ');
        editor.undo_history.commit_group();
        assert_eq!(editor.buffer.to_string(), "a b \n");
        editor.apply_action(Action::Undo);
        assert_eq!(editor.buffer.to_string(), "a \n");
        editor.apply_action(Action::Undo);
        assert_eq!(editor.buffer.to_string(), "\n");
    }

    #[test]
    fn redo_cleared_on_new_edit_after_undo() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("\n");
        editor.vim_mode = Mode::Insert;
        editor.cursor_col = 0;
        editor.handle_char('a');
        editor.undo_history.commit_group();
        editor.apply_action(Action::Undo);
        editor.cursor_col = 0;
        editor.handle_char('b');
        editor.undo_history.commit_group();
        editor.apply_action(Action::Redo);
        assert_eq!(editor.buffer.to_string(), "b\n");
    }

    #[test]
    fn delete_then_undo_restores_text() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("abc\n");
        editor.vim_mode = Mode::Insert;
        editor.cursor_col = 3;
        editor.apply_action(Action::DeleteBack);
        editor.undo_history.commit_group();
        assert_eq!(editor.buffer.to_string(), "ab\n");
        editor.apply_action(Action::Undo);
        assert_eq!(editor.buffer.to_string(), "abc\n");
    }

    #[test]
    fn empty_undo_redo_are_noops() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("hello\n");
        let before = editor.buffer.to_string();
        editor.apply_action(Action::Undo);
        assert_eq!(editor.buffer.to_string(), before);
        editor.apply_action(Action::Redo);
        assert_eq!(editor.buffer.to_string(), before);
    }

    // === Shift+Arrow selection ===

    #[test]
    fn shift_right_sets_anchor_and_extends() {
        let mut editor = Editor::new();
        editor.editing_mode = EditingMode::Standard;
        editor.vim_mode = Mode::Insert;
        editor.buffer = Buffer::from_text("hello\n");
        editor.cursor_col = 0;
        editor.extend_selection(KeyCode::Right, 60);
        assert_eq!(editor.selection_anchor, Some((0, 0)));
        assert_eq!(editor.cursor_col, 1);
    }

    #[test]
    fn shift_left_extends_backward() {
        let mut editor = Editor::new();
        editor.editing_mode = EditingMode::Standard;
        editor.vim_mode = Mode::Insert;
        editor.buffer = Buffer::from_text("hello\n");
        editor.cursor_col = 3;
        editor.extend_selection(KeyCode::Left, 60);
        assert_eq!(editor.selection_anchor, Some((0, 3)));
        assert_eq!(editor.cursor_col, 2);
    }

    #[test]
    fn shift_home_selects_to_line_start() {
        let mut editor = Editor::new();
        editor.editing_mode = EditingMode::Standard;
        editor.vim_mode = Mode::Insert;
        editor.buffer = Buffer::from_text("hello\n");
        editor.cursor_col = 3;
        editor.extend_selection(KeyCode::Home, 60);
        assert_eq!(editor.selection_anchor, Some((0, 3)));
        assert_eq!(editor.cursor_col, 0);
    }

    #[test]
    fn shift_end_selects_to_line_end() {
        let mut editor = Editor::new();
        editor.editing_mode = EditingMode::Standard;
        editor.vim_mode = Mode::Insert;
        editor.buffer = Buffer::from_text("hello\n");
        editor.cursor_col = 0;
        editor.extend_selection(KeyCode::End, 60);
        assert_eq!(editor.selection_anchor, Some((0, 0)));
        assert_eq!(editor.cursor_col, 5);
    }

    #[test]
    fn multiple_shift_arrows_accumulate_selection() {
        let mut editor = Editor::new();
        editor.editing_mode = EditingMode::Standard;
        editor.vim_mode = Mode::Insert;
        editor.buffer = Buffer::from_text("hello\n");
        editor.cursor_col = 0;
        editor.extend_selection(KeyCode::Right, 60);
        editor.extend_selection(KeyCode::Right, 60);
        editor.extend_selection(KeyCode::Right, 60);
        assert_eq!(editor.selection_anchor, Some((0, 0)));
        assert_eq!(editor.cursor_col, 3);
    }

    #[test]
    fn shift_arrow_in_vim_enters_visual() {
        let mut editor = Editor::new();
        editor.editing_mode = EditingMode::Vim;
        editor.vim_mode = Mode::Normal;
        editor.buffer = Buffer::from_text("hello\n");
        editor.cursor_col = 0;
        editor.extend_selection(KeyCode::Right, 60);
        assert_eq!(editor.vim_mode, Mode::Visual);
        assert_eq!(editor.selection_anchor, Some((0, 0)));
    }

    // === Visual mode ===

    #[test]
    fn v_enters_visual_with_correct_anchor() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("hello world\n");
        editor.cursor_col = 3;
        editor.handle_char('v');
        assert_eq!(editor.vim_mode, Mode::Visual);
        assert_eq!(editor.selection_anchor, Some((0, 3)));
    }

    #[test]
    fn movement_in_visual_preserves_anchor() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("hello world\n");
        editor.cursor_col = 3;
        editor.handle_char('v');
        editor.handle_char('l');
        editor.handle_char('l');
        assert_eq!(editor.vim_mode, Mode::Visual);
        assert_eq!(editor.selection_anchor, Some((0, 3)));
        assert_eq!(editor.cursor_col, 5);
    }

    #[test]
    fn escape_clears_selection_and_returns_to_normal() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("hello\n");
        editor.handle_char('v');
        assert_eq!(editor.vim_mode, Mode::Visual);
        editor.handle_escape();
        assert_eq!(editor.vim_mode, Mode::Normal);
        assert_eq!(editor.selection_anchor, None);
    }

    #[test]
    fn y_yanks_correct_text_to_register() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("hello world\n");
        editor.cursor_col = 0;
        editor.handle_char('v');
        editor.handle_char('l');
        editor.handle_char('l');
        editor.handle_char('l');
        editor.handle_char('l');
        editor.handle_char('y');
        assert_eq!(editor.vim_mode, Mode::Normal);
        assert_eq!(editor.yank_register, Some("hello".to_string()));
        assert_eq!(editor.selection_anchor, None);
    }

    #[test]
    fn d_deletes_selection_and_yanks_to_register() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("hello world\n");
        editor.cursor_col = 0;
        editor.handle_char('v');
        editor.handle_char('l');
        editor.handle_char('l');
        editor.handle_char('l');
        editor.handle_char('l');
        editor.handle_char('d');
        assert_eq!(editor.vim_mode, Mode::Normal);
        assert_eq!(editor.yank_register, Some("hello".to_string()));
        assert_eq!(editor.buffer.to_string(), " world\n");
        assert!(editor.dirty);
    }

    #[test]
    fn selection_range_normalizes_when_anchor_after_cursor() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("hello world\n");
        editor.cursor_col = 8;
        editor.handle_char('v');
        editor.handle_char('h');
        editor.handle_char('h');
        editor.handle_char('h');
        let range = editor.selection_range().unwrap();
        assert_eq!(range, (0, 5, 0, 8));
    }

    #[test]
    fn selected_text_works_multiline() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("hello\nworld\n");
        editor.cursor_col = 3;
        editor.handle_char('v');
        editor.handle_char('j');
        editor.handle_char('l');
        let text = editor.selected_text().unwrap();
        assert_eq!(text, "lo\nworld");
    }

    #[test]
    fn gg_works_in_visual_mode() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("first\nsecond\nthird\n");
        editor.cursor_line = 2;
        editor.handle_char('v');
        editor.handle_char('g');
        editor.handle_char('g');
        assert_eq!(editor.cursor_line, 0);
        assert_eq!(editor.vim_mode, Mode::Visual);
    }

    #[test]
    fn q_does_not_quit_in_visual_mode() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("hello\n");
        editor.handle_char('v');
        let quit = editor.handle_char('q');
        assert!(!quit);
        assert_eq!(editor.vim_mode, Mode::Visual);
    }

    // === Paste tests ===

    #[test]
    fn p_inserts_register_content_after_cursor_charwise() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("ab\n");
        editor.cursor_col = 0;
        editor.yank_register = Some("XY".to_string());
        editor.handle_char('p');
        assert_eq!(editor.buffer.to_string(), "aXYb\n");
        assert!(editor.dirty);
    }

    #[test]
    fn big_p_inserts_before_cursor_charwise() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("ab\n");
        editor.cursor_col = 1;
        editor.yank_register = Some("XY".to_string());
        editor.handle_char('P');
        assert_eq!(editor.buffer.to_string(), "aXYb\n");
    }

    #[test]
    fn p_multiline_inserts_on_next_line() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("first\nsecond\n");
        editor.cursor_line = 0;
        editor.yank_register = Some("new\n".to_string());
        editor.handle_char('p');
        let text = editor.buffer.to_string();
        assert_eq!(text, "first\nnew\nsecond\n");
        assert_eq!(editor.cursor_line, 1);
    }

    #[test]
    fn big_p_multiline_inserts_on_current_line() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("first\nsecond\n");
        editor.cursor_line = 1;
        editor.yank_register = Some("new\n".to_string());
        editor.handle_char('P');
        let text = editor.buffer.to_string();
        assert_eq!(text, "first\nnew\nsecond\n");
    }

    #[test]
    fn empty_register_paste_is_noop() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("hello\n");
        editor.yank_register = None;
        let before = editor.buffer.to_string();
        editor.handle_char('p');
        assert_eq!(editor.buffer.to_string(), before);
        assert!(!editor.dirty);
    }

    #[test]
    fn dd_populates_register_with_deleted_line() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("first\nsecond\nthird\n");
        editor.cursor_line = 1;
        editor.handle_char('d');
        editor.handle_char('d');
        assert_eq!(editor.yank_register, Some("second\n".to_string()));
        assert!(!editor.buffer.to_string().contains("second"));
    }

    // === Ctrl-chord undo tests ===

    #[test]
    fn delete_selection_silent_removes_without_yanking() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("hello world\n");
        editor.selection_anchor = Some((0, 0));
        editor.cursor_col = 4;
        editor.yank_register = None;
        editor.delete_selection_silent();
        assert_eq!(editor.buffer.to_string(), " world\n");
        assert_eq!(editor.yank_register, None);
        assert!(editor.dirty);
    }

    #[test]
    fn select_all_sets_anchor_and_moves_cursor_to_end() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("hello\nworld\n");
        let total_chars = editor.buffer.len_chars();
        editor.selection_anchor = Some((0, 0));
        editor.set_cursor_from_char_index(total_chars.saturating_sub(1));
        assert_eq!(editor.selection_anchor, Some((0, 0)));
        assert!(editor.cursor_line > 0 || editor.cursor_col > 0);
    }

    #[test]
    fn yank_then_paste_round_trip() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("hello world\n");
        editor.cursor_col = 0;
        editor.handle_char('v');
        editor.handle_char('l');
        editor.handle_char('l');
        editor.handle_char('l');
        editor.handle_char('l');
        editor.handle_char('y');
        editor.handle_char('$');
        editor.handle_char('p');
        let text = editor.buffer.to_string();
        assert!(text.contains("hello"));
    }

    #[test]
    fn paste_after_records_undo() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("hello");
        editor.cursor_line = 0;
        editor.cursor_col = 4;
        editor.yank_register = Some(" world".to_string());
        editor.apply_action(Action::PasteAfter);
        assert_eq!(editor.buffer.to_string(), "hello world");
        editor.apply_action(Action::Undo);
        assert_eq!(editor.buffer.to_string(), "hello");
    }

    // === Cursor navigation ===

    #[test]
    fn cursor_right_reaches_end_of_last_line_without_newline() {
        let mut editor = Editor::new();
        editor.editing_mode = EditingMode::Standard;
        editor.vim_mode = Mode::Insert;
        editor.buffer = Buffer::from_text("hello");
        editor.cursor_line = 0;
        editor.cursor_col = 0;
        for _ in 0..5 {
            editor.move_cursor_with_width(Direction::Right, 60);
        }
        assert_eq!(editor.cursor_col, 5);
    }

    #[test]
    fn line_end_reaches_end_of_line() {
        let mut editor = Editor::new();
        editor.editing_mode = EditingMode::Standard;
        editor.vim_mode = Mode::Insert;
        editor.buffer = Buffer::from_text("hello\nworld");
        editor.cursor_line = 0;
        editor.cursor_col = 0;
        editor.apply_action(Action::LineEnd);
        assert_eq!(editor.cursor_col, 5);

        editor.cursor_line = 1;
        editor.cursor_col = 0;
        editor.apply_action(Action::LineEnd);
        assert_eq!(editor.cursor_col, 5);
    }

    #[test]
    fn clamp_cursor_col_allows_end_of_line() {
        let mut editor = Editor::new();
        editor.editing_mode = EditingMode::Standard;
        editor.vim_mode = Mode::Insert;
        editor.buffer = Buffer::from_text("hello");
        editor.cursor_line = 0;
        editor.cursor_col = 5;
        editor.clamp_cursor_col();
        assert_eq!(editor.cursor_col, 5);
    }

    // === Bounds cache ===

    #[test]
    fn bounds_cache_returns_same_result_on_second_call() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("First paragraph.\n\nSecond paragraph.\n");
        editor.cursor_line = 0;
        editor.cursor_col = 5;

        let pb1 = editor.paragraph_bounds_cached();

        // Second call should hit cache (same result)
        let pb2 = editor.paragraph_bounds_cached();

        assert_eq!(pb1, pb2);
    }

    #[test]
    fn bounds_cache_invalidates_on_cursor_move() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("First paragraph.\n\nSecond paragraph.\n");
        editor.cursor_line = 0;
        editor.cursor_col = 0;

        let pb1 = editor.paragraph_bounds_cached();
        assert_eq!(pb1, Some((0, 0)));

        // Move cursor to second paragraph
        editor.cursor_line = 2;
        let pb2 = editor.paragraph_bounds_cached();
        assert_eq!(pb2, Some((2, 2)));
    }

    #[test]
    fn bounds_cache_invalidates_on_buffer_edit() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("One.\n\nTwo.\n");
        editor.cursor_line = 2;
        editor.cursor_col = 0;

        let pb1 = editor.paragraph_bounds_cached();
        assert_eq!(pb1, Some((2, 2)));

        // Insert a line that merges the paragraphs
        editor.buffer.remove(4, 5); // remove the blank line's newline

        let pb2 = editor.paragraph_bounds_cached();
        // After merging, paragraph should now span more lines
        assert_ne!(pb1, pb2);
    }

    // === o/O undo regression ===

    #[test]
    fn o_then_undo_restores_original() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("first\nsecond\n");
        editor.cursor_line = 0;
        let original = editor.buffer.to_string();
        editor.handle_char('o');
        assert_ne!(editor.buffer.to_string(), original);
        editor.apply_action(Action::Undo);
        assert_eq!(editor.buffer.to_string(), original);
    }

    #[test]
    fn big_o_then_undo_restores_original() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("first\nsecond\n");
        editor.cursor_line = 1;
        let original = editor.buffer.to_string();
        editor.handle_char('O');
        assert_ne!(editor.buffer.to_string(), original);
        editor.apply_action(Action::Undo);
        assert_eq!(editor.buffer.to_string(), original);
    }

    // === reset_to_content ===

    #[test]
    fn reset_to_content_clears_state() {
        let mut editor = Editor::new();
        editor.buffer = Buffer::from_text("old text\n");
        editor.cursor_line = 0;
        editor.cursor_col = 4;
        editor.vim_mode = Mode::Insert;
        editor.insert_char('x'); // populate undo history
        editor.dirty = true;
        editor.selection_anchor = Some((0, 2));

        editor.reset_to_content("new content\n");

        assert_eq!(editor.buffer.to_string(), "new content\n");
        assert!(!editor.dirty);
        assert_eq!(editor.cursor_line, 0);
        assert_eq!(editor.cursor_col, 0);
        assert_eq!(editor.selection_anchor, None);
    }
}
