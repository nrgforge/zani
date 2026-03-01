use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyModifiers};

use crate::animation::AnimationManager;
use crate::buffer::Buffer;
use crate::clipboard;
use crate::config::Config;
use crate::color_profile::ColorProfile;
use crate::draft_name;
use crate::editing_mode::EditingMode;
use crate::find::FindState;
use crate::focus_mode::{self, DimLayer, FadeConfig, FocusMode, LineOpacity, paragraph_target_opacities};
use crate::palette::Palette;
use crate::scroll_mode::ScrollMode;
use crate::smart_typography;
use crate::undo::UndoHistory;
use crate::vim_bindings::{self, Action, CursorShape, Direction, Mode};
use crate::wrap::{self, VisualLine};

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

/// State for the inline rename overlay.
pub struct RenameState {
    pub active: bool,
    pub buf: String,
    pub cursor: usize,
}

impl RenameState {
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
}

/// State for the Settings Layer overlay.
pub struct SettingsState {
    pub visible: bool,
    pub cursor: usize,
}

/// Application state.
///
/// Cursor, scroll, and selection fields live here rather than in a separate
/// struct because 15+ methods read and write them together with buffer state.
/// Extracting them would add indirection without reducing coupling.
pub struct App {
    pub buffer: Buffer,
    pub palette: Palette,
    pub focus_mode: FocusMode,
    pub scroll_mode: ScrollMode,
    pub editing_mode: EditingMode,
    pub color_profile: ColorProfile,
    pub vim_mode: Mode,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll_offset: usize,
    pub scroll_display: f64,
    pub column_width: u16,
    pub settings: SettingsState,
    pub should_quit: bool,
    pub file_path: Option<PathBuf>,
    pub is_scratch: bool,
    pub dirty: bool,
    pub last_save: Option<Instant>,
    pub autosave_interval: Duration,
    /// Last save error message, surfaced in the settings layer.
    pub save_error: Option<String>,
    pub rename: RenameState,
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
    /// Undo/redo history.
    pub undo_history: UndoHistory,
    /// Find overlay state (None when find is not active).
    pub find_state: Option<FindState>,
    pub animations: AnimationManager,
    pub paragraph_dim: DimLayer,
    /// Full bounds of the last known sentence (to detect genuine changes).
    last_sentence_bounds: Option<(usize, usize)>,
    /// Queue of sentences fading out, each with its own animation.
    /// (char_start, char_end, animated opacity).
    sentence_fades: Vec<(usize, usize, LineOpacity)>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            palette: Palette::default_palette(),
            focus_mode: FocusMode::Off,
            scroll_mode: ScrollMode::Edge,
            editing_mode: EditingMode::default(),
            color_profile: ColorProfile::TrueColor,
            vim_mode: Mode::Normal,
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
            scroll_display: 0.0,
            column_width: 60,
            settings: SettingsState { visible: false, cursor: 0 },
            should_quit: false,
            file_path: None,
            is_scratch: false,
            dirty: false,
            last_save: None,
            autosave_interval: Duration::from_secs(3),
            save_error: None,
            rename: RenameState { active: false, buf: String::new(), cursor: 0 },
            pending_normal_key: None,
            typewriter_vertical_offset: 0,
            selection_anchor: None,
            yank_register: None,
            undo_history: UndoHistory::new(),
            find_state: None,
            animations: AnimationManager::new(),
            paragraph_dim: DimLayer::new(
                FadeConfig { duration: Duration::from_millis(150), easing: crate::animation::Easing::EaseOut },
                FadeConfig { duration: Duration::from_millis(1800), easing: crate::animation::Easing::EaseOut },
            ),
            last_sentence_bounds: None,
            sentence_fades: Vec::new(),
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

    /// The cursor shape based on current editing and vim mode.
    pub fn cursor_shape(&self) -> CursorShape {
        if self.editing_mode == EditingMode::Standard {
            return CursorShape::Bar;
        }
        self.vim_mode.cursor_shape()
    }

    /// Toggle the Settings Layer visibility.
    pub fn toggle_settings(&mut self) {
        self.settings.visible = !self.settings.visible;
        if self.settings.visible {
            use crate::animation::{Easing, TransitionKind};
            self.animations.start(
                TransitionKind::OverlayOpacity { appearing: true },
                Duration::from_millis(150),
                Easing::EaseOut,
            );
            // Find the index of the active palette in the full settings item list
            let items = SettingsItem::all();
            let target = SettingsItem::Palette(self.active_palette_index());
            self.settings.cursor = items.iter().position(|i| *i == target).unwrap_or(0);
        }
    }

    /// Switch to a different Palette.
    pub fn set_palette(&mut self, palette: Palette) {
        self.palette = palette;
    }

    /// Dismiss the Settings Layer.
    pub fn dismiss_settings(&mut self) {
        self.settings.visible = false;
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
        if self.settings.cursor == 0 {
            self.settings.cursor = count - 1;
        } else {
            self.settings.cursor -= 1;
        }
    }

    /// Move the settings cursor down (wrapping).
    pub fn settings_nav_down(&mut self) {
        let count = SettingsItem::all().len();
        self.settings.cursor = (self.settings.cursor + 1) % count;
    }

    /// Apply the currently selected settings item.
    pub fn settings_apply(&mut self) {
        let Some(item) = SettingsItem::at(self.settings.cursor) else {
            return;
        };
        match item {
            SettingsItem::EditingMode(mode) => {
                self.editing_mode = mode;
                match mode {
                    EditingMode::Standard => {
                        self.vim_mode = Mode::Insert;
                        self.pending_normal_key = None;
                    }
                    EditingMode::Vim => {
                        self.vim_mode = Mode::Normal;
                    }
                }
            }
            SettingsItem::Palette(idx) => {
                if let Some(p) = Palette::all().into_iter().nth(idx) {
                    if p.name != self.palette.name {
                        use crate::animation::{Easing, TransitionKind};
                        use std::time::Duration;
                        self.animations.start(
                            TransitionKind::Palette {
                                from: Box::new(self.palette.clone()),
                                to: Box::new(p.clone()),
                            },
                            Duration::from_millis(300),
                            Easing::EaseInOut,
                        );
                    }
                    self.palette = p;
                }
            }
            SettingsItem::FocusMode(mode) => self.focus_mode = mode,
            SettingsItem::ScrollMode(mode) => self.scroll_mode = mode,
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
        self.rename.buf = name;
        self.rename.cursor = self.rename.buf.chars().count();
        self.rename.active = true;
    }

    /// Insert a character at cursor position (filters out `/`).
    pub fn rename_insert(&mut self, ch: char) {
        if ch == '/' {
            return;
        }
        let byte_idx = char_to_byte_index(&self.rename.buf, self.rename.cursor);
        self.rename.buf.insert(byte_idx, ch);
        self.rename.cursor += 1;
    }

    /// Delete the character before the cursor.
    pub fn rename_backspace(&mut self) {
        if self.rename.cursor == 0 {
            return;
        }
        self.rename.cursor -= 1;
        let byte_idx = char_to_byte_index(&self.rename.buf, self.rename.cursor);
        // Find the byte length of the char at this position
        let ch = self.rename.buf[byte_idx..].chars().next().unwrap();
        self.rename.buf.replace_range(byte_idx..byte_idx + ch.len_utf8(), "");
    }

    /// Cancel rename, clearing state.
    pub fn rename_cancel(&mut self) {
        self.rename.active = false;
        self.rename.buf.clear();
        self.rename.cursor = 0;
    }

    /// Confirm rename: rename on disk, update file_path, clear scratch flag.
    /// Empty name is treated as cancel.
    pub fn rename_confirm(&mut self) {
        if self.rename.buf.trim().is_empty() {
            self.rename_cancel();
            return;
        }

        if let Some(old_path) = &self.file_path {
            let new_path = old_path.with_file_name(&self.rename.buf);

            // Only attempt fs::rename if old file exists on disk
            if old_path.exists()
                && std::fs::rename(old_path, &new_path).is_err()
            {
                // Stay in rename mode so user can retry or Esc
                return;
            }

            self.file_path = Some(new_path);
            if self.is_scratch {
                self.is_scratch = false;
            }
        }

        self.rename.active = false;
        self.rename.buf.clear();
        self.rename.cursor = 0;
    }

    /// Persist current settings to config file (best-effort, errors silently ignored).
    fn save_config(&self) {
        let config = Config {
            palette: self.palette.name.to_string(),
            focus_mode: self.focus_mode,
            column_width: self.column_width,
            editing_mode: self.editing_mode,
            scroll_mode: self.scroll_mode,
        };
        let _ = config.save();
    }

    /// Handle a key press event. This is the main input dispatch entry point.
    pub fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        // Ctrl combinations — checked first, independent of vim mode
        if modifiers.contains(KeyModifiers::CONTROL) {
            match code {
                KeyCode::Char('c') => {
                    // Copy selection if any, otherwise no-op
                    if let Some(text) = self.selected_text() {
                        clipboard::write_osc52(&text);
                        self.yank_register = Some(text);
                        self.selection_anchor = None;
                        if self.vim_mode == Mode::Visual {
                            self.vim_mode = Mode::Normal;
                        }
                    }
                }
                KeyCode::Char('x') => {
                    // Cut: copy selection then delete it
                    if let Some(text) = self.selected_text() {
                        clipboard::write_osc52(&text);
                        self.yank_register = Some(text);
                        self.delete_selection_silent();
                        self.selection_anchor = None;
                        if self.vim_mode == Mode::Visual {
                            self.vim_mode = Mode::Normal;
                        }
                    }
                }
                KeyCode::Char('v') => {
                    // Paste from yank register at cursor
                    if let Some(text) = self.yank_register.clone() {
                        // In Standard mode with selection, replace selection first
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
                        // Advance cursor past inserted text
                        let char_count = text.chars().count();
                        self.set_cursor_from_char_index(idx + char_count);
                        self.dirty = true;
                    }
                }
                KeyCode::Char('a') => {
                    // Select all
                    let total_chars = self.buffer.len_chars();
                    self.selection_anchor = Some((0, 0));
                    if total_chars > 0 {
                        self.set_cursor_from_char_index(total_chars.saturating_sub(1));
                    }
                    if self.editing_mode == EditingMode::Vim {
                        self.vim_mode = Mode::Visual;
                    }
                }
                KeyCode::Char('q') => {
                    self.should_quit = true;
                }
                KeyCode::Char('p') => {
                    self.toggle_settings();
                }
                KeyCode::Char('s') => {
                    self.autosave();
                }
                KeyCode::Char('f') => {
                    if self.find_state.is_none() {
                        self.find_state = Some(FindState::new(
                            self.cursor_line,
                            self.cursor_col,
                        ));
                        self.animations.start(
                            crate::animation::TransitionKind::OverlayOpacity { appearing: true },
                            Duration::from_millis(150),
                            crate::animation::Easing::EaseOut,
                        );
                    }
                }
                KeyCode::Char('z') => {
                    self.apply_action(Action::Undo);
                }
                KeyCode::Char('y') => {
                    self.apply_action(Action::Redo);
                }
                _ => {}
            }
            return;
        }

        // Find overlay — swallow all keys when active
        if let Some(ref mut find) = self.find_state {
            match code {
                KeyCode::Esc => {
                    // Cancel: restore cursor to pre-search position
                    let (line, col) = find.saved_cursor;
                    self.cursor_line = line;
                    self.cursor_col = col;
                    self.find_state = None;
                }
                KeyCode::Enter => {
                    // Jump to current match and close find
                    if let Some((line, col)) = find.current_match_pos() {
                        self.cursor_line = line;
                        self.cursor_col = col;
                    }
                    self.find_state = None;
                }
                KeyCode::Backspace => {
                    find.backspace();
                    find.search(&self.buffer);
                    // Jump cursor to first match for live preview
                    if let Some((line, col)) = find.current_match_pos() {
                        self.cursor_line = line;
                        self.cursor_col = col;
                    }
                }
                KeyCode::Up => {
                    find.prev_match();
                    if let Some((line, col)) = find.current_match_pos() {
                        self.cursor_line = line;
                        self.cursor_col = col;
                    }
                }
                KeyCode::Down => {
                    find.next_match();
                    if let Some((line, col)) = find.current_match_pos() {
                        self.cursor_line = line;
                        self.cursor_col = col;
                    }
                }
                KeyCode::Char(c) => {
                    find.insert_char(c);
                    find.search(&self.buffer);
                    // Jump cursor to first match for live preview
                    if let Some((line, col)) = find.current_match_pos() {
                        self.cursor_line = line;
                        self.cursor_col = col;
                    }
                }
                _ => {}
            }
            return;
        }

        // Inline rename — swallow all keys when active
        if self.rename.active {
            match code {
                KeyCode::Esc => self.rename_cancel(),
                KeyCode::Enter => self.rename_confirm(),
                KeyCode::Backspace => self.rename_backspace(),
                KeyCode::Left => self.rename.cursor_left(),
                KeyCode::Right => self.rename.cursor_right(),
                KeyCode::Char(c) => self.rename_insert(c),
                _ => {}
            }
            return;
        }

        // Settings Layer navigation — swallow all keys when open
        if self.settings.visible {
            match code {
                KeyCode::Esc => self.dismiss_settings(),
                KeyCode::Up | KeyCode::Char('k') => self.settings_nav_up(),
                KeyCode::Down | KeyCode::Char('j') => self.settings_nav_down(),
                KeyCode::Enter => {
                    self.settings_apply();
                    self.save_config();
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    if SettingsItem::at(self.settings.cursor) == Some(SettingsItem::ColumnWidth) {
                        self.settings_adjust_column(-1);
                        self.save_config();
                    }
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    if SettingsItem::at(self.settings.cursor) == Some(SettingsItem::ColumnWidth) {
                        self.settings_adjust_column(1);
                        self.save_config();
                    }
                }
                _ => {} // swallow all other keys
            }
            return;
        }

        let is_standard = self.editing_mode == EditingMode::Standard;

        // Shift+Arrow/Home/End extends selection
        if modifiers.contains(KeyModifiers::SHIFT) {
            match code {
                KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down
                | KeyCode::Home | KeyCode::End => {
                    self.extend_selection(code);
                    return;
                }
                _ => {}
            }
        }

        match code {
            KeyCode::Esc => self.handle_escape(),
            KeyCode::Char(c) => self.handle_char(c),
            KeyCode::Backspace => {
                if is_standard || self.vim_mode == Mode::Insert {
                    // In Standard mode with selection, delete selection instead
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
                self.apply_action(Action::MoveCursor(Direction::Left));
            }
            KeyCode::Right => {
                if is_standard {
                    self.selection_anchor = None;
                }
                self.apply_action(Action::MoveCursor(Direction::Right));
            }
            KeyCode::Up => {
                if is_standard {
                    self.selection_anchor = None;
                }
                self.apply_action(Action::MoveCursor(Direction::Up));
            }
            KeyCode::Down => {
                if is_standard {
                    self.selection_anchor = None;
                }
                self.apply_action(Action::MoveCursor(Direction::Down));
            }
            _ => {}
        }
    }

    /// Process a character key input.
    pub fn handle_char(&mut self, ch: char) {
        // Standard mode: always insert characters directly
        if self.editing_mode == EditingMode::Standard {
            if ch.is_control() {
                return;
            }
            // Selection replaces on type
            if self.selection_anchor.is_some() {
                self.delete_selection_silent();
                self.selection_anchor = None;
            }
            self.insert_char(ch);
            return;
        }

        let action = match self.vim_mode {
            Mode::Normal => {
                // Handle multi-key sequences
                if let Some(pending) = self.pending_normal_key.take() {
                    match (pending, ch) {
                        ('g', 'g') => {
                            self.apply_action(Action::GotoFirstLine);
                            return;
                        }
                        ('d', 'd') => {
                            self.apply_action(Action::DeleteLine);
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
                            self.apply_action(Action::GotoFirstLine);
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
        if self.settings.visible {
            self.dismiss_settings();
        } else if self.editing_mode == EditingMode::Standard {
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
                        // Crossed a line boundary — commit group
                        self.undo_history.commit_group();
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
                    let deleted = self.buffer.slice_to_string(idx, idx + 1);
                    self.undo_history.record_delete(idx, &deleted);
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
                    let start_idx = self.buffer.line_to_char(sl) + sc;
                    let end_idx = (self.buffer.line_to_char(el) + ec + 1).min(self.buffer.len_chars());
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
                        let idx = idx.min(self.buffer.len_chars());
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
            Action::Undo => {
                self.undo_history.commit_group();
                if let Some(ops) = self.undo_history.undo() {
                    // Apply inverse operations in reverse order
                    for op in ops.iter().rev() {
                        match op {
                            crate::undo::Operation::Insert { pos, text } => {
                                // Undo an insert = delete
                                let end = pos + text.chars().count();
                                self.buffer.remove(*pos, end);
                            }
                            crate::undo::Operation::Delete { pos, text } => {
                                // Undo a delete = insert
                                self.buffer.insert(*pos, text);
                            }
                        }
                    }
                    // Position cursor at the location of the first operation
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
                    // Re-apply operations in original order
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
                    // Position cursor after the last operation
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

    fn insert_char(&mut self, ch: char) {
        let idx = self.cursor_char_index();

        // Check for smart typography transformation
        let preceding = self.preceding_text(idx);
        if let Some(edit) = smart_typography::transform(ch, &preceding) {
            // Delete preceding characters
            if edit.delete_before > 0 {
                let start = idx.saturating_sub(edit.delete_before);
                let deleted = self.buffer.slice_to_string(start, idx);
                self.undo_history.record_delete(start, &deleted);
                self.buffer.remove(start, idx);
                self.cursor_col -= edit.delete_before;
            }
            // Insert replacement
            let new_idx = self.cursor_char_index();
            self.undo_history.record_insert(new_idx, edit.insert);
            self.buffer.insert(new_idx, edit.insert);
            self.cursor_col += edit.insert.chars().count();
        } else {
            let s = ch.to_string();
            self.undo_history.record_insert(idx, &s);
            self.buffer.insert(idx, &s);
            self.cursor_col += 1;
        }
        self.dirty = true;

        // Commit undo group on whitespace (each word is one undo unit)
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

    /// Calculate the character index in the buffer for the current cursor position.
    pub fn cursor_char_index(&self) -> usize {
        self.line_start_char_index() + self.cursor_col
    }

    /// Character index of the start of the current line.
    fn line_start_char_index(&self) -> usize {
        self.buffer.line_to_char(self.cursor_line)
    }

    /// Number of visible characters on the given line (excludes trailing newline).
    fn line_content_len(&self, line: usize) -> usize {
        let slice = self.buffer.line(line);
        let len = slice.len_chars();
        if len > 0 && slice.char(len - 1) == '\n' {
            len - 1
        } else {
            len
        }
    }

    /// Find the visual line index containing (cursor_line, cursor_col).
    fn find_cursor_visual_index(&self, visual_lines: &[VisualLine]) -> Option<usize> {
        for (i, vl) in visual_lines.iter().enumerate() {
            if vl.logical_line == self.cursor_line
                && self.cursor_col >= vl.char_start
                && self.cursor_col < vl.char_end
            {
                return Some(i);
            }
        }
        // Fallback: cursor at end of line or on empty line
        visual_lines
            .iter()
            .rposition(|vl| vl.logical_line == self.cursor_line)
    }

    fn move_cursor(&mut self, dir: vim_bindings::Direction) {
        match dir {
            vim_bindings::Direction::Left => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                }
            }
            vim_bindings::Direction::Right => {
                let max_col = self.line_content_len(self.cursor_line);
                if self.cursor_col < max_col {
                    self.cursor_col += 1;
                }
            }
            vim_bindings::Direction::Up | vim_bindings::Direction::Down => {
                let visual_lines = self.visual_lines();
                let Some(cur_idx) = self.find_cursor_visual_index(&visual_lines) else {
                    return;
                };

                let target_idx = if dir == vim_bindings::Direction::Up {
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
        }
    }

    /// Move cursor forward to the start of the next word (whitespace-delimited).
    fn word_forward(&mut self) {
        let mut idx = self.cursor_char_index();
        let len = self.buffer.len_chars();

        // Skip current non-whitespace
        while idx < len && !self.buffer.char_at(idx).is_whitespace() {
            idx += 1;
        }
        // Skip whitespace
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

        // Skip whitespace backward
        while idx > 0 && self.buffer.char_at(idx).is_whitespace() {
            idx -= 1;
        }
        // Skip non-whitespace backward to find word start
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

        // Skip whitespace
        while idx < len && self.buffer.char_at(idx).is_whitespace() {
            idx += 1;
        }
        // Move to end of word
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

        // Yank deleted line to register and clipboard
        let deleted = self.buffer.slice_to_string(line_start, line_start + line_len);
        self.yank_register = Some(deleted.clone());
        clipboard::write_osc52(&deleted);

        self.undo_history.commit_group();
        self.undo_history.record_delete(line_start, &deleted);
        self.undo_history.commit_group();

        self.buffer.remove(line_start, line_start + line_len);
        self.dirty = true;

        // Adjust cursor if we deleted the last line
        if self.cursor_line >= self.buffer.len_lines() {
            self.cursor_line = self.buffer.len_lines().saturating_sub(1);
        }
        self.clamp_cursor_col();
    }

    /// Extend selection by moving the cursor while keeping (or setting) the anchor.
    /// Used for Shift+Arrow selection in both modes.
    pub fn extend_selection(&mut self, code: crossterm::event::KeyCode) {
        use crossterm::event::KeyCode;

        // Set anchor if not already selecting
        if self.selection_anchor.is_none() {
            self.selection_anchor = Some((self.cursor_line, self.cursor_col));
        }

        // Move cursor
        match code {
            KeyCode::Left => self.move_cursor(vim_bindings::Direction::Left),
            KeyCode::Right => self.move_cursor(vim_bindings::Direction::Right),
            KeyCode::Up => self.move_cursor(vim_bindings::Direction::Up),
            KeyCode::Down => self.move_cursor(vim_bindings::Direction::Down),
            KeyCode::Home => self.cursor_col = 0,
            KeyCode::End => {
                self.cursor_col = self.line_content_len(self.cursor_line);
            }
            _ => {}
        }

        // In Vim mode, switch to Visual if not already
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

    fn clamp_cursor_col(&mut self) {
        let max_col = self.line_content_len(self.cursor_line);
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
        let start_idx = self.buffer.line_to_char(sl) + sc;
        // Include the character at end_col
        let end_idx = self.buffer.line_to_char(el) + ec + 1;
        let end_idx = end_idx.min(self.buffer.len_chars());
        if start_idx >= end_idx {
            return None;
        }
        Some(self.buffer.slice_to_string(start_idx, end_idx))
    }

    /// Find the sentence boundaries containing the cursor.
    /// Returns (start, end) as absolute char indices into the buffer.
    pub fn sentence_bounds(&self) -> Option<(usize, usize)> {
        focus_mode::sentence_bounds_in_buffer(&self.buffer, self.cursor_char_index())
    }

    /// Recompute dimming layer targets based on current focus mode and cursor position.
    pub fn update_dim_layers(&mut self) {
        let line_count = self.buffer.len_lines();
        match self.focus_mode {
            FocusMode::Off => {
                let targets = vec![1.0; line_count];
                self.paragraph_dim.update_targets(&targets);
                self.last_sentence_bounds = None;
                self.sentence_fades.clear();
            }
            FocusMode::Paragraph => {
                let targets = paragraph_target_opacities(
                    line_count,
                    self.paragraph_bounds(),
                );
                self.paragraph_dim.update_targets(&targets);
                self.last_sentence_bounds = None;
                self.sentence_fades.clear();
            }
            FocusMode::Sentence => {
                let targets = paragraph_target_opacities(
                    line_count,
                    self.paragraph_bounds(),
                );
                self.paragraph_dim.update_targets(&targets);

                let current_bounds = self.sentence_bounds();
                let current_start = current_bounds.map(|(s, _)| s);
                let last_start = self.last_sentence_bounds.map(|(s, _)| s);

                // Detect genuine sentence change (start index changed).
                if current_start != last_start {
                    // Check if we're returning to any currently-fading sentence
                    let returning_idx = current_bounds.and_then(|(cs, _)| {
                        self.sentence_fades.iter().position(|(fs, _, _)| *fs == cs)
                    });

                    if let Some(idx) = returning_idx {
                        // Reverse that entry: fade back in (current → 1.0, 150ms)
                        self.sentence_fades[idx].2.set_target(
                            1.0,
                            FadeConfig {
                                duration: Duration::from_millis(150),
                                easing: crate::animation::Easing::EaseOut,
                            },
                        );
                    } else if let Some((old_start, old_end)) = self.last_sentence_bounds {
                        // Push a new fade for the sentence we just left
                        let mut opacity = LineOpacity::new(1.0);
                        opacity.set_target(
                            0.6,
                            FadeConfig {
                                duration: Duration::from_millis(1800),
                                easing: crate::animation::Easing::EaseOut,
                            },
                        );
                        self.sentence_fades.push((old_start, old_end, opacity));
                    }
                }
                self.last_sentence_bounds = current_bounds;

                // Prune completed fades
                self.sentence_fades.retain(|(_, _, o)| o.is_animating());

                // Sentence dimming is handled per-character by the renderer
                // via sentence_fades, not by a line-level DimLayer.
            }
        }
    }

    /// Snapshot of all in-flight sentence fades: (char_start, char_end, current_opacity).
    pub fn sentence_fade_snapshot(&self) -> Vec<(usize, usize, f64)> {
        self.sentence_fades
            .iter()
            .map(|(s, e, o)| (*s, *e, o.current_opacity()))
            .collect()
    }

    /// Compute final per-line opacities for the renderer.
    pub fn line_opacities(&self) -> Vec<f64> {
        let line_count = self.buffer.len_lines();
        (0..line_count)
            .map(|i| self.paragraph_dim.opacity(i))
            .collect()
    }

    /// Whether any dimming layer is still animating.
    pub fn dim_animating(&self) -> bool {
        self.paragraph_dim.is_animating()
            || self.sentence_fades.iter().any(|(_, _, o)| o.is_animating())
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

        let old_offset = self.scroll_offset;

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

        if self.scroll_mode == ScrollMode::Typewriter {
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

        if self.scroll_offset != old_offset {
            let from = self.scroll_display;
            let to = self.scroll_offset as f64;
            if (from - to).abs() > 0.01 {
                use crate::animation::{Easing, TransitionKind};
                self.animations.start(
                    TransitionKind::Scroll { from, to },
                    std::time::Duration::from_millis(150),
                    Easing::EaseOut,
                );
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
            let content = self.buffer.to_string();
            match std::fs::write(path, &content) {
                Ok(()) => {
                    self.dirty = false;
                    self.last_save = Some(Instant::now());
                    self.save_error = None;
                    return true;
                }
                Err(e) => {
                    self.save_error = Some(e.to_string());
                }
            }
        }
        false
    }

    /// Returns the effective palette, accounting for any active crossfade animation.
    pub fn effective_palette(&self) -> Palette {
        if let Some((progress, from, _to)) = self.animations.palette_progress() {
            use crate::palette::interpolate;
            Palette {
                name: self.palette.name,
                foreground: interpolate(&from.foreground, &self.palette.foreground, progress),
                background: interpolate(&from.background, &self.palette.background, progress),
                dimmed_foreground: interpolate(
                    &from.dimmed_foreground,
                    &self.palette.dimmed_foreground,
                    progress,
                ),
                accent_heading: interpolate(
                    &from.accent_heading,
                    &self.palette.accent_heading,
                    progress,
                ),
                accent_emphasis: interpolate(
                    &from.accent_emphasis,
                    &self.palette.accent_emphasis,
                    progress,
                ),
                accent_link: interpolate(&from.accent_link, &self.palette.accent_link, progress),
                accent_code: interpolate(&from.accent_code, &self.palette.accent_code, progress),
            }
        } else {
            self.palette.clone()
        }
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
    use crate::editing_mode::EditingMode;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // === Unit test: SettingsItem::all() matches expected count ===

    #[test]
    fn settings_item_count_matches_expected() {
        // 2 editing modes + 3 palettes + 3 focus modes + 2 scroll modes + 1 column width + 1 file = 12
        assert_eq!(SettingsItem::all().len(), 12);
    }

    // === Acceptance test: Default state has no visible Chrome ===

    #[test]
    fn default_state_has_no_chrome() {
        let app = App::new();
        assert!(!app.settings.visible);
    }

    // === Acceptance test: Settings Layer is summoned by hotkey ===

    #[test]
    fn toggle_settings_makes_chrome_visible() {
        let mut app = App::new();
        app.toggle_settings();
        assert!(app.settings.visible);
    }

    // === Acceptance test: Settings Layer is dismissed ===

    #[test]
    fn dismiss_settings_hides_chrome() {
        let mut app = App::new();
        app.toggle_settings();
        assert!(app.settings.visible);
        app.dismiss_settings();
        assert!(!app.settings.visible);
    }

    #[test]
    fn escape_dismisses_settings() {
        let mut app = App::new();
        app.toggle_settings();
        app.handle_escape();
        assert!(!app.settings.visible);
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
        assert_eq!(app.buffer.to_string(), "ac\n");
        assert!(app.dirty);
    }

    #[test]
    fn o_opens_line_below() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("first\nsecond\n");
        app.cursor_line = 0;
        app.handle_char('o');
        assert_eq!(app.vim_mode, Mode::Insert, "o should enter insert mode");
        assert_eq!(app.cursor_line, 1, "o should move cursor to new line below");
        assert_eq!(app.cursor_col, 0, "o should place cursor at column 0");
        assert_eq!(app.buffer.len_lines(), 4, "o should insert a new blank line");
    }

    #[test]
    fn big_o_opens_line_above() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("first\nsecond\n");
        app.cursor_line = 1;
        app.handle_char('O');
        assert_eq!(app.vim_mode, Mode::Insert, "O should enter insert mode");
        assert_eq!(app.cursor_line, 1, "O should keep cursor on inserted blank line");
        assert_eq!(app.cursor_col, 0, "O should place cursor at column 0");
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
        let text = app.buffer.to_string();
        assert!(!text.contains("second"), "Line should be deleted, got: {}", text);
        assert!(app.dirty);
    }

    #[test]
    fn unknown_second_key_is_harmless() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("hello\n");
        app.cursor_col = 2;
        let before = app.buffer.to_string();
        app.handle_char('g');
        app.handle_char('z'); // unknown
        assert_eq!(app.buffer.to_string(), before);
        assert_eq!(app.cursor_col, 2); // unchanged
    }

    // === Acceptance test: Typewriter mode centering ===

    #[test]
    fn typewriter_mode_centers_cursor() {
        let mut app = App::new();
        // Create a buffer with 20 lines
        let text = (0..20).map(|i| format!("Line {}\n", i)).collect::<String>();
        app.buffer = Buffer::from_text(&text);
        app.scroll_mode = ScrollMode::Typewriter;
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
        app.scroll_mode = ScrollMode::Typewriter;
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
        let text = app.buffer.to_string();
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
        assert_eq!(app.cursor_col, 2, "h should move cursor left");
        app.handle_char('l');
        assert_eq!(app.cursor_col, 3, "l should move cursor right");
        app.handle_char('k');
        assert_eq!(app.cursor_line, 0, "k should move cursor up");
        app.handle_char('j');
        assert_eq!(app.cursor_line, 1, "j should move cursor down");
    }

    // === Settings Layer navigation ===

    #[test]
    fn toggle_settings_sets_cursor_to_active_palette() {
        let mut app = App::new();
        app.palette = Palette::inkwell();
        app.toggle_settings();
        assert_eq!(app.settings.cursor, 3); // Inkwell is at position 3 (2 editing modes + Ember + Inkwell)
    }

    #[test]
    fn settings_nav_down_wraps() {
        let mut app = App::new();
        app.settings.cursor = 11;
        app.settings_nav_down();
        assert_eq!(app.settings.cursor, 0, "nav down from last item should wrap to 0");
    }

    #[test]
    fn settings_nav_up_wraps() {
        let mut app = App::new();
        app.settings.cursor = 0;
        app.settings_nav_up();
        assert_eq!(app.settings.cursor, 11, "nav up from 0 should wrap to last item");
    }

    #[test]
    fn settings_nav_down_increments() {
        let mut app = App::new();
        app.settings.cursor = 2;
        app.settings_nav_down();
        assert_eq!(app.settings.cursor, 3);
    }

    #[test]
    fn settings_nav_up_decrements() {
        let mut app = App::new();
        app.settings.cursor = 5;
        app.settings_nav_up();
        assert_eq!(app.settings.cursor, 4);
    }

    #[test]
    fn settings_apply_palette() {
        let mut app = App::new();
        app.settings.cursor = 3; // Inkwell (2 editing modes + Ember=2, Inkwell=3)
        app.settings_apply();
        assert_eq!(app.palette.name, "Inkwell");
    }

    #[test]
    fn settings_apply_focus_mode() {
        let mut app = App::new();
        app.settings.cursor = 6; // Sentence
        app.settings_apply();
        assert_eq!(app.focus_mode, FocusMode::Sentence, "cursor 6 should select Sentence focus");

        app.settings.cursor = 7; // Paragraph
        app.settings_apply();
        assert_eq!(app.focus_mode, FocusMode::Paragraph, "cursor 7 should select Paragraph focus");
    }

    #[test]
    fn settings_apply_scroll_mode() {
        let mut app = App::new();
        app.settings.cursor = 9; // ScrollMode::Typewriter
        app.settings_apply();
        assert_eq!(app.scroll_mode, ScrollMode::Typewriter, "cursor 9 should select Typewriter scroll");

        app.settings.cursor = 8; // ScrollMode::Edge
        app.settings_apply();
        assert_eq!(app.scroll_mode, ScrollMode::Edge, "cursor 8 should select Edge scroll");
    }

    #[test]
    fn settings_apply_column_is_noop() {
        let mut app = App::new();
        app.settings.cursor = 10; // ColumnWidth
        let before = app.column_width;
        app.settings_apply();
        assert_eq!(app.column_width, before, "ColumnWidth row should not change width on Enter");
    }

    #[test]
    fn settings_adjust_column_increases() {
        let mut app = App::new();
        assert_eq!(app.column_width, 60, "default column width should be 60");
        app.settings_adjust_column(5);
        assert_eq!(app.column_width, 65, "adjusting +5 should increase to 65");
    }

    #[test]
    fn settings_adjust_column_clamps_low() {
        let mut app = App::new();
        app.column_width = 22;
        app.settings_adjust_column(-5);
        assert_eq!(app.column_width, 20, "column width should clamp at 20");
    }

    #[test]
    fn settings_adjust_column_clamps_high() {
        let mut app = App::new();
        app.column_width = 118;
        app.settings_adjust_column(5);
        assert_eq!(app.column_width, 120, "column width should clamp at 120");
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
        assert!(app.rename.active);
        assert_eq!(app.rename.buf, "draft.md");
        assert_eq!(app.rename.cursor, 8); // "draft.md".len()
    }

    #[test]
    fn rename_insert_adds_char_at_cursor() {
        let mut app = App::new();
        app.rename.active = true;
        app.rename.buf = "ab".to_string();
        app.rename.cursor = 1;
        app.rename_insert('X');
        assert_eq!(app.rename.buf, "aXb");
        assert_eq!(app.rename.cursor, 2);
    }

    #[test]
    fn rename_insert_filters_slash() {
        let mut app = App::new();
        app.rename.active = true;
        app.rename.buf = "ab".to_string();
        app.rename.cursor = 1;
        app.rename_insert('/');
        assert_eq!(app.rename.buf, "ab");
        assert_eq!(app.rename.cursor, 1);
    }

    #[test]
    fn rename_backspace_deletes_before_cursor() {
        let mut app = App::new();
        app.rename.active = true;
        app.rename.buf = "abc".to_string();
        app.rename.cursor = 2;
        app.rename_backspace();
        assert_eq!(app.rename.buf, "ac");
        assert_eq!(app.rename.cursor, 1);
    }

    #[test]
    fn rename_backspace_at_start_is_noop() {
        let mut app = App::new();
        app.rename.active = true;
        app.rename.buf = "abc".to_string();
        app.rename.cursor = 0;
        app.rename_backspace();
        assert_eq!(app.rename.buf, "abc");
        assert_eq!(app.rename.cursor, 0);
    }

    #[test]
    fn rename_cursor_left_right() {
        let mut app = App::new();
        app.rename.active = true;
        app.rename.buf = "abc".to_string();
        app.rename.cursor = 1;

        app.rename.cursor_left();
        assert_eq!(app.rename.cursor, 0);

        app.rename.cursor_left(); // at start, stays 0
        assert_eq!(app.rename.cursor, 0);

        app.rename.cursor_right();
        assert_eq!(app.rename.cursor, 1);

        app.rename.cursor = 3; // at end
        app.rename.cursor_right(); // stays at end
        assert_eq!(app.rename.cursor, 3);
    }

    #[test]
    fn rename_cancel_clears_state() {
        let mut app = App::new();
        app.file_path = Some(PathBuf::from("/tmp/draft.md"));
        app.rename_open();
        assert!(app.rename.active);

        app.rename_cancel();
        assert!(!app.rename.active);
        assert!(app.rename.buf.is_empty());
        assert_eq!(app.rename.cursor, 0);
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
        app.rename.buf = "new.md".to_string();
        app.rename.cursor = 6;
        app.rename_confirm();

        assert!(!app.rename.active);
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
        app.rename.buf = "".to_string();
        app.rename.cursor = 0;
        app.rename_confirm();

        assert!(!app.rename.active);
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
        app.rename.buf = "real.md".to_string();
        app.rename.cursor = 7;
        app.rename_confirm();

        assert!(!app.is_scratch);
    }

    #[test]
    fn settings_apply_file_opens_rename() {
        let mut app = App::new();
        app.file_path = Some(PathBuf::from("/tmp/draft.md"));
        app.settings.cursor = 11; // File
        app.settings_apply();
        assert!(app.rename.active);
        assert_eq!(app.rename.buf, "draft.md");
    }

    #[test]
    fn rename_confirm_unsaved_scratch_updates_path_without_fs_rename() {
        // File doesn't exist on disk — should just update path
        let mut app = App::new();
        app.file_path = Some(PathBuf::from("/nonexistent/dir/scratch.md"));
        app.is_scratch = true;
        app.rename_open();
        app.rename.buf = "real.md".to_string();
        app.rename.cursor = 7;
        app.rename_confirm();

        assert!(!app.rename.active);
        assert_eq!(
            app.file_path,
            Some(PathBuf::from("/nonexistent/dir/real.md"))
        );
        assert!(!app.is_scratch);
    }

    // === Editing mode tests ===

    #[test]
    fn settings_apply_switches_to_standard_mode() {
        let mut app = App::new();
        app.settings.cursor = 1; // Standard
        app.settings_apply();
        assert_eq!(app.editing_mode, EditingMode::Standard);
        assert_eq!(app.vim_mode, Mode::Insert);
    }

    #[test]
    fn settings_apply_switches_to_vim_mode() {
        let mut app = App::new();
        app.editing_mode = EditingMode::Standard;
        app.vim_mode = Mode::Insert;
        app.settings.cursor = 0; // Vim
        app.settings_apply();
        assert_eq!(app.editing_mode, EditingMode::Vim);
        assert_eq!(app.vim_mode, Mode::Normal);
    }

    #[test]
    fn switching_to_standard_clears_pending_normal_key() {
        let mut app = App::new();
        app.pending_normal_key = Some('g');
        app.settings.cursor = 1; // Standard
        app.settings_apply();
        assert_eq!(app.pending_normal_key, None);
    }

    // === Standard mode tests ===

    #[test]
    fn standard_mode_typing_inserts_directly() {
        let mut app = App::new();
        app.editing_mode = EditingMode::Standard;
        app.vim_mode = Mode::Insert;
        app.buffer = Buffer::from_text("hello\n");
        app.cursor_col = 5;
        app.handle_char('!');
        assert_eq!(app.buffer.to_string(), "hello!\n");
    }

    #[test]
    fn standard_mode_escape_clears_selection_not_mode() {
        let mut app = App::new();
        app.editing_mode = EditingMode::Standard;
        app.vim_mode = Mode::Insert;
        app.selection_anchor = Some((0, 2));
        app.handle_escape();
        assert_eq!(app.selection_anchor, None);
        assert_eq!(app.vim_mode, Mode::Insert); // stays in Insert
    }

    #[test]
    fn standard_mode_cursor_is_always_bar() {
        let mut app = App::new();
        app.editing_mode = EditingMode::Standard;
        app.vim_mode = Mode::Insert;
        assert_eq!(app.cursor_shape(), CursorShape::Bar);
    }

    #[test]
    fn standard_mode_typing_replaces_selection() {
        let mut app = App::new();
        app.editing_mode = EditingMode::Standard;
        app.vim_mode = Mode::Insert;
        app.buffer = Buffer::from_text("hello world\n");
        // Select "hello" (0,0) to (0,4)
        app.selection_anchor = Some((0, 0));
        app.cursor_col = 4;
        app.handle_char('X');
        assert_eq!(app.buffer.to_string(), "X world\n");
        assert_eq!(app.selection_anchor, None);
    }

    #[test]
    fn standard_mode_q_inserts_q_not_quit() {
        let mut app = App::new();
        app.editing_mode = EditingMode::Standard;
        app.vim_mode = Mode::Insert;
        app.buffer = Buffer::from_text("\n");
        app.cursor_col = 0;
        app.handle_char('q');
        assert!(!app.should_quit);
        assert!(app.buffer.to_string().contains('q'));
    }

    // === Mode leakage prevention tests ===

    #[test]
    fn standard_mode_vim_mode_stays_insert() {
        let mut app = App::new();
        app.editing_mode = EditingMode::Standard;
        app.vim_mode = Mode::Insert;
        // SwitchMode should be ignored in Standard mode
        app.apply_action(Action::SwitchMode(Mode::Normal));
        assert_eq!(app.vim_mode, Mode::Insert);
    }

    #[test]
    fn standard_mode_escape_keeps_insert() {
        let mut app = App::new();
        app.editing_mode = EditingMode::Standard;
        app.vim_mode = Mode::Insert;
        app.handle_escape();
        assert_eq!(app.vim_mode, Mode::Insert);
    }

    #[test]
    fn standard_mode_vim_keys_insert_literally() {
        let mut app = App::new();
        app.editing_mode = EditingMode::Standard;
        app.vim_mode = Mode::Insert;
        app.buffer = Buffer::from_text("\n");
        app.cursor_col = 0;
        // 'i' should insert 'i', not switch to Insert mode
        app.handle_char('i');
        assert!(app.buffer.to_string().contains('i'));
        assert_eq!(app.vim_mode, Mode::Insert);
    }

    // === Full round-trip test ===

    #[test]
    fn standard_mode_full_round_trip() {
        let mut app = App::new();
        // Switch to Standard mode
        app.settings.cursor = 1;
        app.settings_apply();
        assert_eq!(app.editing_mode, EditingMode::Standard);
        assert_eq!(app.vim_mode, Mode::Insert);

        // Type text
        app.buffer = Buffer::from_text("\n");
        app.cursor_col = 0;
        app.handle_char('h');
        app.handle_char('i');
        assert_eq!(app.buffer.to_string(), "hi\n");

        // Select with shift+arrow (via extend_selection)
        use crossterm::event::KeyCode;
        app.cursor_col = 0;
        app.extend_selection(KeyCode::Right);
        app.extend_selection(KeyCode::Right);
        assert_eq!(app.selection_anchor, Some((0, 0)));
        assert_eq!(app.cursor_col, 2);

        // Copy (simulating Ctrl+C behavior)
        if let Some(text) = app.selected_text() {
            app.yank_register = Some(text);
            app.selection_anchor = None;
        }
        assert!(app.yank_register.is_some());

        // Paste "hi" specifically to test the flow
        app.yank_register = Some("hi".to_string());
        app.cursor_col = 2;
        if let Some(text) = app.yank_register.clone() {
            let idx = app.cursor_char_index();
            app.buffer.insert(idx, &text);
            app.set_cursor_from_char_index(idx + text.chars().count());
            app.dirty = true;
        }
        assert_eq!(app.buffer.to_string(), "hihi\n");

        // Switch back to Vim mode
        app.settings.cursor = 0;
        app.settings_apply();
        assert_eq!(app.editing_mode, EditingMode::Vim);
        assert_eq!(app.vim_mode, Mode::Normal);
    }

    // === Find tests ===

    #[test]
    fn find_state_opens_and_closes() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("hello world\n");
        assert!(app.find_state.is_none());
        app.find_state = Some(crate::find::FindState::new(0, 0));
        assert!(app.find_state.is_some());
        app.find_state = None;
        assert!(app.find_state.is_none());
    }

    #[test]
    fn find_escape_restores_cursor() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("hello world\n");
        app.cursor_line = 0;
        app.cursor_col = 3;
        let fs = crate::find::FindState::new(0, 3);
        app.find_state = Some(fs);
        // Simulate moving cursor to a match
        app.cursor_col = 6;
        // Cancel find — should restore
        let saved = app.find_state.as_ref().unwrap().saved_cursor;
        app.cursor_line = saved.0;
        app.cursor_col = saved.1;
        app.find_state = None;
        assert_eq!(app.cursor_col, 3);
    }

    // === Undo/Redo integration tests ===

    #[test]
    fn undo_restores_previous_state() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("hello\n");
        app.vim_mode = Mode::Insert;
        app.cursor_col = 5;
        app.handle_char('!');
        app.undo_history.commit_group();
        assert_eq!(app.buffer.to_string(), "hello!\n");
        app.apply_action(Action::Undo);
        assert_eq!(app.buffer.to_string(), "hello\n");
    }

    #[test]
    fn undo_then_redo_restores_change() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("hello\n");
        app.vim_mode = Mode::Insert;
        app.cursor_col = 5;
        app.handle_char('!');
        app.undo_history.commit_group();
        app.apply_action(Action::Undo);
        assert_eq!(app.buffer.to_string(), "hello\n");
        app.apply_action(Action::Redo);
        assert_eq!(app.buffer.to_string(), "hello!\n");
    }

    #[test]
    fn multiple_undos_walk_back_through_history() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("\n");
        app.vim_mode = Mode::Insert;
        app.cursor_col = 0;
        // Type "a " (space commits group)
        app.handle_char('a');
        app.handle_char(' ');
        // Type "b " (space commits group)
        app.handle_char('b');
        app.handle_char(' ');
        app.undo_history.commit_group();
        assert_eq!(app.buffer.to_string(), "a b \n");
        // Undo "b "
        app.apply_action(Action::Undo);
        assert_eq!(app.buffer.to_string(), "a \n");
        // Undo "a "
        app.apply_action(Action::Undo);
        assert_eq!(app.buffer.to_string(), "\n");
    }

    #[test]
    fn redo_cleared_on_new_edit_after_undo() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("\n");
        app.vim_mode = Mode::Insert;
        app.cursor_col = 0;
        app.handle_char('a');
        app.undo_history.commit_group();
        app.apply_action(Action::Undo);
        // New edit
        app.cursor_col = 0;
        app.handle_char('b');
        app.undo_history.commit_group();
        // Redo should not bring back 'a'
        app.apply_action(Action::Redo);
        // Buffer should still be "b\n" — redo is no-op
        assert_eq!(app.buffer.to_string(), "b\n");
    }

    #[test]
    fn delete_then_undo_restores_text() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("abc\n");
        app.vim_mode = Mode::Insert;
        app.cursor_col = 3;
        app.apply_action(Action::DeleteBack);
        app.undo_history.commit_group();
        assert_eq!(app.buffer.to_string(), "ab\n");
        app.apply_action(Action::Undo);
        assert_eq!(app.buffer.to_string(), "abc\n");
    }

    #[test]
    fn empty_undo_redo_are_noops() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("hello\n");
        let before = app.buffer.to_string();
        app.apply_action(Action::Undo);
        assert_eq!(app.buffer.to_string(), before);
        app.apply_action(Action::Redo);
        assert_eq!(app.buffer.to_string(), before);
    }

    // === Shift+Arrow selection tests ===

    #[test]
    fn shift_right_sets_anchor_and_extends() {
        use crossterm::event::KeyCode;
        let mut app = App::new();
        app.editing_mode = EditingMode::Standard;
        app.vim_mode = Mode::Insert;
        app.buffer = Buffer::from_text("hello\n");
        app.cursor_col = 0;
        app.extend_selection(KeyCode::Right);
        assert_eq!(app.selection_anchor, Some((0, 0)));
        assert_eq!(app.cursor_col, 1);
    }

    #[test]
    fn shift_left_extends_backward() {
        use crossterm::event::KeyCode;
        let mut app = App::new();
        app.editing_mode = EditingMode::Standard;
        app.vim_mode = Mode::Insert;
        app.buffer = Buffer::from_text("hello\n");
        app.cursor_col = 3;
        app.extend_selection(KeyCode::Left);
        assert_eq!(app.selection_anchor, Some((0, 3)));
        assert_eq!(app.cursor_col, 2);
    }

    #[test]
    fn shift_down_extends_to_next_line() {
        use crossterm::event::KeyCode;
        let mut app = App::new();
        app.editing_mode = EditingMode::Standard;
        app.vim_mode = Mode::Insert;
        app.buffer = Buffer::from_text("hello\nworld\n");
        app.cursor_line = 0;
        app.cursor_col = 2;
        app.extend_selection(KeyCode::Down);
        assert_eq!(app.selection_anchor, Some((0, 2)));
        assert_eq!(app.cursor_line, 1);
    }

    #[test]
    fn shift_home_selects_to_line_start() {
        use crossterm::event::KeyCode;
        let mut app = App::new();
        app.editing_mode = EditingMode::Standard;
        app.vim_mode = Mode::Insert;
        app.buffer = Buffer::from_text("hello\n");
        app.cursor_col = 3;
        app.extend_selection(KeyCode::Home);
        assert_eq!(app.selection_anchor, Some((0, 3)));
        assert_eq!(app.cursor_col, 0);
    }

    #[test]
    fn shift_end_selects_to_line_end() {
        use crossterm::event::KeyCode;
        let mut app = App::new();
        app.editing_mode = EditingMode::Standard;
        app.vim_mode = Mode::Insert;
        app.buffer = Buffer::from_text("hello\n");
        app.cursor_col = 0;
        app.extend_selection(KeyCode::End);
        assert_eq!(app.selection_anchor, Some((0, 0)));
        assert_eq!(app.cursor_col, 5); // past 'o', end of visible content
    }

    #[test]
    fn multiple_shift_arrows_accumulate_selection() {
        use crossterm::event::KeyCode;
        let mut app = App::new();
        app.editing_mode = EditingMode::Standard;
        app.vim_mode = Mode::Insert;
        app.buffer = Buffer::from_text("hello\n");
        app.cursor_col = 0;
        app.extend_selection(KeyCode::Right);
        app.extend_selection(KeyCode::Right);
        app.extend_selection(KeyCode::Right);
        assert_eq!(app.selection_anchor, Some((0, 0)));
        assert_eq!(app.cursor_col, 3);
    }

    #[test]
    fn shift_arrow_in_vim_enters_visual() {
        use crossterm::event::KeyCode;
        let mut app = App::new();
        app.editing_mode = EditingMode::Vim;
        app.vim_mode = Mode::Normal;
        app.buffer = Buffer::from_text("hello\n");
        app.cursor_col = 0;
        app.extend_selection(KeyCode::Right);
        assert_eq!(app.vim_mode, Mode::Visual);
        assert_eq!(app.selection_anchor, Some((0, 0)));
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
        assert_eq!(app.buffer.to_string(), " world\n");
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
        assert_eq!(app.buffer.to_string(), "aXYb\n");
        assert!(app.dirty);
    }

    #[test]
    fn big_p_inserts_before_cursor_charwise() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("ab\n");
        app.cursor_col = 1;
        app.yank_register = Some("XY".to_string());
        app.handle_char('P');
        assert_eq!(app.buffer.to_string(), "aXYb\n");
    }

    #[test]
    fn p_multiline_inserts_on_next_line() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("first\nsecond\n");
        app.cursor_line = 0;
        app.yank_register = Some("new\n".to_string());
        app.handle_char('p');
        let text = app.buffer.to_string();
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
        let text = app.buffer.to_string();
        assert_eq!(text, "first\nnew\nsecond\n");
    }

    #[test]
    fn empty_register_paste_is_noop() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("hello\n");
        app.yank_register = None;
        let before = app.buffer.to_string();
        app.handle_char('p');
        assert_eq!(app.buffer.to_string(), before);
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
        assert!(!app.buffer.to_string().contains("second"));
    }

    // === Ctrl key operation tests ===

    #[test]
    fn delete_selection_silent_removes_without_yanking() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("hello world\n");
        app.selection_anchor = Some((0, 0));
        app.cursor_col = 4;
        app.yank_register = None;
        app.delete_selection_silent();
        assert_eq!(app.buffer.to_string(), " world\n");
        assert_eq!(app.yank_register, None);
        assert!(app.dirty);
    }

    #[test]
    fn select_all_sets_anchor_and_moves_cursor_to_end() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("hello\nworld\n");
        // Simulate Ctrl+A behavior
        let total_chars = app.buffer.len_chars();
        app.selection_anchor = Some((0, 0));
        app.set_cursor_from_char_index(total_chars.saturating_sub(1));
        assert_eq!(app.selection_anchor, Some((0, 0)));
        assert!(app.cursor_line > 0 || app.cursor_col > 0);
    }

    #[test]
    fn scroll_animation_starts_on_scroll_change() {
        let mut app = App::new();
        app.buffer = Buffer::from_text(&"line\n".repeat(50));
        app.scroll_display = 0.0;
        app.scroll_offset = 0;
        let visual_lines = app.visual_lines();
        app.cursor_line = 30;
        app.cursor_col = 0;
        app.ensure_cursor_visible(&visual_lines, 20);
        assert!(app.scroll_offset > 0);
        assert!(app.animations.is_active());
        assert!(app.animations.scroll_progress().is_some());
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
        let text = app.buffer.to_string();
        assert!(text.contains("hello"), "Yanked text should be pasted");
    }

    // === Palette crossfade tests ===

    #[test]
    fn effective_palette_returns_current_when_no_animation() {
        let app = App::new();
        let eff = app.effective_palette();
        assert_eq!(eff.name, app.palette.name);
        assert_eq!(eff.foreground, app.palette.foreground);
    }

    #[test]
    fn palette_animation_starts_on_switch() {
        let mut app = App::new();
        app.toggle_settings();
        app.settings.cursor = 3; // Inkwell palette index
        app.settings_apply();
        assert_eq!(app.palette.name, "Inkwell");
        assert!(app.animations.is_active());
    }

    // === Overlay fade animation tests ===

    #[test]
    fn overlay_animation_starts_on_settings_open() {
        let mut app = App::new();
        app.toggle_settings();
        assert!(app.animations.overlay_progress().is_some());
    }

    #[test]
    fn overlay_no_animation_on_settings_close() {
        let mut app = App::new();
        app.toggle_settings();
        // Clear the opening animation
        app.animations.transitions.clear();
        app.dismiss_settings();
        // No overlay animation started on dismiss
        assert!(app.animations.overlay_progress().is_none());
    }

    // === Task 7: DimLayer wired into App ===

    #[test]
    fn dim_layers_produce_opacities() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("Line 1\n\nLine 3\nLine 4");
        app.focus_mode = FocusMode::Paragraph;
        app.cursor_line = 0;
        app.update_dim_layers();
        let opacities = app.line_opacities();
        assert_eq!(opacities.len(), 4);
        assert!((opacities[0] - 1.0).abs() < f64::EPSILON, "Cursor line should be bright");
        assert!(opacities[2] < 1.0, "Other paragraph should be dimmed");
    }

    #[test]
    fn sentence_fade_animates_on_sentence_change() {
        let mut app = App::new();
        // Two sentences on separate lines
        app.buffer = Buffer::from_text("First sentence.\nSecond sentence.");
        app.focus_mode = FocusMode::Sentence;
        app.cursor_line = 0;
        app.cursor_col = 0;

        app.update_dim_layers();

        // No fades in progress initially
        assert!(app.sentence_fades.is_empty());

        // Move cursor to line 1 (second sentence)
        app.cursor_line = 1;
        app.cursor_col = 0;
        app.update_dim_layers();

        // One fade should be queued for the old sentence
        assert_eq!(app.sentence_fades.len(), 1, "should have one fading sentence");
        let snap = app.sentence_fade_snapshot();
        assert!(snap[0].2 > 0.9, "Opacity should be near 1.0 right after change");
        assert!(app.dim_animating(), "should be animating");
    }

    #[test]
    fn focus_off_all_bright() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("Line 1\nLine 2\nLine 3");
        app.focus_mode = FocusMode::Off;
        app.update_dim_layers();
        let opacities = app.line_opacities();
        for (i, &o) in opacities.iter().enumerate() {
            assert!((o - 1.0).abs() < f64::EPSILON, "Line {} should be bright in Off mode", i);
        }
    }

    // === Sentence fade queue regression tests ===

    #[test]
    fn rapid_typing_after_period_preserves_fade() {
        let mut app = App::new();
        app.focus_mode = FocusMode::Sentence;

        // Cursor in "Hello world."
        app.buffer = Buffer::from_text("Hello world.");
        app.cursor_line = 0;
        app.cursor_col = 5;
        app.update_dim_layers();
        assert!(app.sentence_fades.is_empty(), "No fades initially");

        // Simulate typing space after period: "Hello world. "
        app.buffer = Buffer::from_text("Hello world. ");
        app.cursor_col = 12;
        app.update_dim_layers();
        assert_eq!(app.sentence_fades.len(), 1, "One fade after leaving sentence");
        let original_start = app.sentence_fades[0].0;

        // Simulate typing 'T': "Hello world. T"
        app.buffer = Buffer::from_text("Hello world. T");
        app.cursor_col = 13;
        app.update_dim_layers();

        // The original "Hello world." fade must survive the second sentence change
        assert!(
            app.sentence_fades.iter().any(|(s, _, _)| *s == original_start),
            "Original sentence fade must survive rapid typing"
        );
    }

    #[test]
    fn multiple_sentences_fade_independently() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("First.\n\nSecond.\n\nThird.");
        app.focus_mode = FocusMode::Sentence;

        // Start in first sentence
        app.cursor_line = 0;
        app.cursor_col = 0;
        app.update_dim_layers();

        // Move to second sentence
        app.cursor_line = 2;
        app.cursor_col = 0;
        app.update_dim_layers();
        assert_eq!(app.sentence_fades.len(), 1);

        // Move to third sentence
        app.cursor_line = 4;
        app.cursor_col = 0;
        app.update_dim_layers();
        assert_eq!(app.sentence_fades.len(), 2, "Two sentences should be fading simultaneously");

        // Both should have high opacity (just started or recently started)
        let snap = app.sentence_fade_snapshot();
        assert!(snap[0].2 > 0.5, "First fade should still be in progress");
        assert!(snap[1].2 > 0.9, "Second fade should have just started");
    }

    #[test]
    fn returning_to_fading_sentence_reverses_fade() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("First.\n\nSecond.");
        app.focus_mode = FocusMode::Sentence;

        app.cursor_line = 0;
        app.cursor_col = 0;
        app.update_dim_layers();

        // Move to second — first starts fading toward 0.6
        app.cursor_line = 2;
        app.cursor_col = 0;
        app.update_dim_layers();
        assert_eq!(app.sentence_fades.len(), 1);
        assert!((app.sentence_fades[0].2.target - 0.6).abs() < f64::EPSILON);

        // Return to first — should reverse that entry toward 1.0
        app.cursor_line = 0;
        app.cursor_col = 0;
        app.update_dim_layers();
        assert!((app.sentence_fades[0].2.target - 1.0).abs() < f64::EPSILON, "Should reverse to 1.0");
    }

    #[test]
    fn completed_fades_are_pruned() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("First.\n\nSecond.");
        app.focus_mode = FocusMode::Sentence;

        app.cursor_line = 0;
        app.cursor_col = 0;
        app.update_dim_layers();

        app.cursor_line = 2;
        app.cursor_col = 0;
        app.update_dim_layers();
        assert_eq!(app.sentence_fades.len(), 1);

        // Backdate the animation past its 1800ms duration
        app.sentence_fades[0].2.start_time =
            Some(Instant::now() - Duration::from_millis(2000));

        // Next update should prune the completed entry
        app.update_dim_layers();
        assert!(app.sentence_fades.is_empty(), "Completed fade should be pruned");
    }

    // === Cursor navigation tests ===

    #[test]
    fn cursor_right_reaches_end_of_last_line_without_newline() {
        let mut app = App::new();
        app.editing_mode = EditingMode::Standard;
        app.vim_mode = Mode::Insert;
        app.buffer = Buffer::from_text("hello");
        app.cursor_line = 0;
        app.cursor_col = 0;

        // Press right 5 times to reach past the last character
        for _ in 0..5 {
            app.move_cursor(vim_bindings::Direction::Right);
        }

        // In standard/insert mode, cursor should be at position 5 (after 'o')
        assert_eq!(app.cursor_col, 5, "Cursor should be past the last character");
    }

    #[test]
    fn line_end_reaches_end_of_line() {
        let mut app = App::new();
        app.editing_mode = EditingMode::Standard;
        app.vim_mode = Mode::Insert;
        app.buffer = Buffer::from_text("hello\nworld");
        app.cursor_line = 0;
        app.cursor_col = 0;

        // LineEnd should go to position after last visible char
        app.apply_action(Action::LineEnd);
        assert_eq!(app.cursor_col, 5, "End should place cursor after 'o' on line with newline");

        // Same for last line (no newline)
        app.cursor_line = 1;
        app.cursor_col = 0;
        app.apply_action(Action::LineEnd);
        assert_eq!(app.cursor_col, 5, "End should place cursor after 'd' on last line");
    }

    #[test]
    fn clamp_cursor_col_allows_end_of_line() {
        let mut app = App::new();
        app.editing_mode = EditingMode::Standard;
        app.vim_mode = Mode::Insert;
        app.buffer = Buffer::from_text("hello");
        app.cursor_line = 0;
        app.cursor_col = 5; // past last char

        app.clamp_cursor_col();
        assert_eq!(app.cursor_col, 5, "Clamp should allow cursor past last char in insert mode");
    }

    #[test]
    fn mode_switch_clears_fade_queue() {
        let mut app = App::new();
        app.buffer = Buffer::from_text("First.\n\nSecond.");
        app.focus_mode = FocusMode::Sentence;

        app.cursor_line = 0;
        app.cursor_col = 0;
        app.update_dim_layers();

        app.cursor_line = 2;
        app.cursor_col = 0;
        app.update_dim_layers();
        assert!(!app.sentence_fades.is_empty());

        // Switch to Off
        app.focus_mode = FocusMode::Off;
        app.update_dim_layers();
        assert!(app.sentence_fades.is_empty(), "Off mode should clear all fades");
    }

    // === Visual-line cursor navigation tests ===

    /// Helper: create an App with given text and column_width, cursor at (0, 0).
    fn app_with_wrap(text: &str, width: u16) -> App {
        let mut app = App::new();
        app.buffer = Buffer::from_text(text);
        app.column_width = width;
        app.cursor_line = 0;
        app.cursor_col = 0;
        app
    }

    #[test]
    fn visual_nav_down_within_wrapped_line() {
        // "abcde fghij" at width 6 wraps to:
        //   vl0: logical 0, chars 0..6  ("abcde ")  — actually let's use a clearer example
        // "hello world foo" at width 6:
        //   vl0: logical 0, 0..6 "hello "
        //   vl1: logical 0, 6..12 "world "
        //   vl2: logical 0, 12..15 "foo"
        let mut app = app_with_wrap("hello world foo", 6);
        app.cursor_col = 0; // on vl0

        app.move_cursor(vim_bindings::Direction::Down);
        // Should move to vl1, same logical line
        assert_eq!(app.cursor_line, 0, "stays on same logical line");
        assert_eq!(app.cursor_col, 6, "moved to start of next visual line");
    }

    #[test]
    fn visual_nav_up_within_wrapped_line() {
        let mut app = app_with_wrap("hello world foo", 6);
        // Start on second visual line
        app.cursor_col = 6; // vl1 start

        app.move_cursor(vim_bindings::Direction::Up);
        assert_eq!(app.cursor_line, 0, "stays on same logical line");
        assert_eq!(app.cursor_col, 0, "moved to start of first visual line");
    }

    #[test]
    fn visual_nav_down_crosses_logical_line() {
        let mut app = app_with_wrap("short\nother", 60);
        // Width 60 means no wrapping; two visual lines, one per logical line
        app.cursor_col = 2;

        app.move_cursor(vim_bindings::Direction::Down);
        assert_eq!(app.cursor_line, 1, "moved to next logical line");
        assert_eq!(app.cursor_col, 2, "preserved visual column");
    }

    #[test]
    fn visual_nav_clamps_col_on_shorter_target() {
        let mut app = app_with_wrap("longline\nhi", 60);
        app.cursor_col = 7; // near end of "longline"

        app.move_cursor(vim_bindings::Direction::Down);
        assert_eq!(app.cursor_line, 1);
        // "hi" has length 2, so col should clamp to 2
        assert_eq!(app.cursor_col, 2, "clamped to end of shorter line");
    }

    #[test]
    fn visual_nav_up_on_first_line_is_noop() {
        let mut app = app_with_wrap("hello\nworld", 60);
        app.cursor_col = 3;

        app.move_cursor(vim_bindings::Direction::Up);
        assert_eq!(app.cursor_line, 0);
        assert_eq!(app.cursor_col, 3, "cursor unchanged");
    }

    #[test]
    fn visual_nav_down_on_last_line_is_noop() {
        let mut app = app_with_wrap("hello\nworld", 60);
        app.cursor_line = 1;
        app.cursor_col = 3;

        app.move_cursor(vim_bindings::Direction::Down);
        assert_eq!(app.cursor_line, 1);
        assert_eq!(app.cursor_col, 3, "cursor unchanged");
    }

    #[test]
    fn visual_nav_through_empty_line() {
        let mut app = app_with_wrap("above\n\nbelow", 60);
        app.cursor_col = 3;

        // Down from "above" -> empty line
        app.move_cursor(vim_bindings::Direction::Down);
        assert_eq!(app.cursor_line, 1);
        assert_eq!(app.cursor_col, 0, "empty line clamps to 0");

        // Down from empty line -> "below"
        app.move_cursor(vim_bindings::Direction::Down);
        assert_eq!(app.cursor_line, 2);
        assert_eq!(app.cursor_col, 0, "visual col was 0 from empty line");
    }

    #[test]
    fn visual_nav_traverses_all_visual_lines_of_wrapped_paragraph() {
        // "hello world foo" at width 6 produces 3 visual lines, all logical line 0
        let mut app = app_with_wrap("hello world foo", 6);

        // Collect cursor positions going down
        let mut positions = vec![(app.cursor_line, app.cursor_col)];
        for _ in 0..5 {
            let prev = (app.cursor_line, app.cursor_col);
            app.move_cursor(vim_bindings::Direction::Down);
            let curr = (app.cursor_line, app.cursor_col);
            if curr == prev {
                break; // hit bottom
            }
            positions.push(curr);
        }

        // Should have visited 3 visual lines
        assert_eq!(positions.len(), 3, "should visit all 3 visual lines");
        // All on same logical line
        assert!(positions.iter().all(|(l, _)| *l == 0), "all on logical line 0");
        // Columns should be ascending
        assert!(positions[0].1 < positions[1].1);
        assert!(positions[1].1 < positions[2].1);

        // Now go back up through all of them
        for _ in 0..2 {
            app.move_cursor(vim_bindings::Direction::Up);
        }
        assert_eq!(app.cursor_col, 0, "back at start");
    }
}
