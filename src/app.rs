use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{KeyCode, KeyModifiers};

use crate::animation::AnimationManager;
use crate::config::Config;
use crate::color_profile::ColorProfile;
use crate::dimming::DimmingState;
use crate::editing_mode::EditingMode;
use crate::editor::Editor;
use crate::find::FindState;
use crate::palette::Palette;
use crate::persistence::Persistence;
use crate::settings::{RenameState, SettingsItem, SettingsState};
use crate::vim_bindings::{Action, Mode};
use crate::viewport::Viewport;
use crate::writing_surface::RenderCache;

/// Thin coordinator that owns subsystems and routes input between them.
pub struct App {
    pub editor: Editor,
    pub viewport: Viewport,
    pub palette: Palette,
    pub dimming: DimmingState,
    pub color_profile: ColorProfile,
    pub settings: SettingsState,
    pub should_quit: bool,
    pub persistence: Persistence,
    pub rename: RenameState,
    /// Find overlay state (None when find is not active).
    pub find_state: Option<FindState>,
    pub animations: AnimationManager,
    pub render_cache: RenderCache,
    /// Whether the next frame requires a full redraw.
    pub needs_redraw: bool,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            editor: Editor::new(),
            viewport: Viewport::new(),
            palette: Palette::default_palette(),
            dimming: DimmingState::new(),
            color_profile: ColorProfile::TrueColor,
            settings: SettingsState::new(),
            should_quit: false,
            persistence: Persistence::new(),
            rename: RenameState::new(),
            find_state: None,
            animations: AnimationManager::new(),
            render_cache: RenderCache::new(),
            needs_redraw: true,
        }
    }

    pub fn with_file(mut self, path: PathBuf, content: &str) -> Self {
        self.persistence.with_file(path, &mut self.editor.buffer, content);
        self
    }

    pub fn with_scratch_name(mut self) -> Self {
        self.persistence.with_scratch_name();
        self
    }

    /// Build an App fully configured from persisted settings.
    pub fn from_config(config: &Config, color_profile: ColorProfile, file_path: Option<PathBuf>) -> Self {
        let mut app = Self::new();
        app.color_profile = color_profile;
        app.palette = config.resolve_palette();
        app.dimming.focus_mode = config.focus_mode;
        app.viewport.scroll_mode = config.scroll_mode;
        app.viewport.column_width = config.column_width;
        app.editor.editing_mode = config.editing_mode;
        if config.editing_mode == EditingMode::Standard {
            app.editor.vim_mode = Mode::Insert;
        }
        if let Some(ref path) = file_path {
            let content = std::fs::read_to_string(path).unwrap_or_default();
            app = app.with_file(path.clone(), &content);
        } else {
            app = app.with_scratch_name();
        }
        app
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

    /// Find the current palette's position in Palette::all().
    pub fn active_palette_index(&self) -> usize {
        Palette::all()
            .iter()
            .position(|p| p.name == self.palette.name)
            .unwrap_or(0)
    }

    /// Apply the currently selected settings item.
    pub fn settings_apply(&mut self) {
        let Some(item) = SettingsItem::at(self.settings.cursor) else {
            return;
        };
        match item {
            SettingsItem::EditingMode(mode) => {
                self.editor.editing_mode = mode;
                match mode {
                    EditingMode::Standard => {
                        self.editor.vim_mode = Mode::Insert;
                        self.editor.pending_normal_key = None;
                    }
                    EditingMode::Vim => {
                        self.editor.vim_mode = Mode::Normal;
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
                                from: Box::new(self.palette),
                                to: Box::new(p),
                            },
                            Duration::from_millis(300),
                            Easing::EaseInOut,
                        );
                    }
                    self.palette = p;
                }
            }
            SettingsItem::FocusMode(mode) => self.dimming.focus_mode = mode,
            SettingsItem::ScrollMode(mode) => self.viewport.scroll_mode = mode,
            SettingsItem::ColumnWidth => {} // adjusted via Left/Right, not Enter
            SettingsItem::File => self.rename.open(self.persistence.file_path.as_deref()),
        }
    }

    /// Adjust column width by delta, clamped to 20–120.
    pub fn settings_adjust_column(&mut self, delta: i16) {
        let new = self.viewport.column_width as i16 + delta;
        self.viewport.column_width = new.clamp(20, 120) as u16;
    }

    /// Persist current settings to config file (best-effort, errors silently ignored).
    fn save_config(&self) {
        let config = Config {
            palette: self.palette.name.to_string(),
            focus_mode: self.dimming.focus_mode,
            column_width: self.viewport.column_width,
            editing_mode: self.editor.editing_mode,
            scroll_mode: self.viewport.scroll_mode,
        };
        let _ = config.save();
    }

    /// Handle a key press event. This is the main input dispatch entry point.
    pub fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        self.needs_redraw = true;

        // Ctrl combinations — checked first, independent of vim mode
        if modifiers.contains(KeyModifiers::CONTROL) {
            self.handle_ctrl_key(code);
            return;
        }

        // Find overlay — swallow all keys when active
        if self.find_state.is_some() {
            self.handle_find_key(code);
            return;
        }

        // Inline rename — swallow all keys when active
        if self.rename.active {
            self.handle_rename_key(code);
            return;
        }

        // Settings Layer navigation — swallow all keys when open
        if self.settings.visible {
            self.handle_settings_key(code);
            return;
        }

        // Intercept Up/Down to use cached visual lines (O(1) instead of O(N))
        if self.try_handle_vertical_move(code, modifiers) {
            return;
        }

        // Route to editor for text editing keys
        if self.editor.handle_key(code, modifiers) {
            self.should_quit = true;
        }
    }

    /// Handle vertical cursor movement using the viewport's cached visual lines.
    /// Returns true if the key was handled.
    fn try_handle_vertical_move(&mut self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        use crate::vim_bindings::Direction;

        let dir = match code {
            KeyCode::Up => Direction::Up,
            KeyCode::Down => Direction::Down,
            KeyCode::Char('k') if self.can_vim_navigate() => Direction::Up,
            KeyCode::Char('j') if self.can_vim_navigate() => Direction::Down,
            _ => return false,
        };

        let visual_lines = self.viewport.visual_lines(&self.editor.buffer);

        if modifiers.contains(KeyModifiers::SHIFT) {
            self.editor.extend_selection_visual(dir, &visual_lines);
        } else {
            // Clear selection for standard mode arrow keys
            if self.editor.editing_mode == EditingMode::Standard {
                self.editor.selection_anchor = None;
            }
            self.editor.move_cursor_visual(dir, &visual_lines);
        }
        true
    }

    /// Whether the current mode allows vim navigation keys (j/k in Normal/Visual).
    fn can_vim_navigate(&self) -> bool {
        self.editor.editing_mode == EditingMode::Vim
            && matches!(self.editor.vim_mode, Mode::Normal | Mode::Visual)
    }

    /// Handle Ctrl+key combinations.
    fn handle_ctrl_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('c') => {
                self.editor.apply_action(Action::Yank);
            }
            KeyCode::Char('x') => {
                self.editor.apply_action(Action::DeleteSelection);
            }
            KeyCode::Char('v') => {
                self.editor.apply_action(Action::PasteAtCursor);
            }
            KeyCode::Char('a') => {
                let total_chars = self.editor.buffer.len_chars();
                self.editor.selection_anchor = Some((0, 0));
                if total_chars > 0 {
                    self.editor.set_cursor_from_char_index(total_chars.saturating_sub(1));
                }
                if self.editor.editing_mode == EditingMode::Vim {
                    self.editor.vim_mode = Mode::Visual;
                }
            }
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Char('p') => {
                self.toggle_settings();
            }
            KeyCode::Char('s') => {
                self.persistence.autosave(&self.editor.buffer, &mut self.editor.dirty);
            }
            KeyCode::Char('f') => {
                if self.find_state.is_none() {
                    self.find_state = Some(FindState::new(
                        self.editor.cursor_line,
                        self.editor.cursor_col,
                    ));
                    self.animations.start(
                        crate::animation::TransitionKind::OverlayOpacity { appearing: true },
                        Duration::from_millis(150),
                        crate::animation::Easing::EaseOut,
                    );
                }
            }
            KeyCode::Char('z') => {
                self.editor.apply_action(Action::Undo);
            }
            KeyCode::Char('y') => {
                self.editor.apply_action(Action::Redo);
            }
            _ => {}
        }
    }

    /// Handle key input while the find overlay is active.
    fn handle_find_key(&mut self, code: KeyCode) {
        let find = self.find_state.as_mut().unwrap();
        match code {
            KeyCode::Esc => {
                let (line, col) = find.saved_cursor;
                self.editor.cursor_line = line;
                self.editor.cursor_col = col;
                self.find_state = None;
            }
            KeyCode::Enter => {
                self.jump_to_find_match();
                self.find_state = None;
            }
            KeyCode::Backspace => {
                find.backspace();
                find.search(&self.editor.buffer);
                self.jump_to_find_match();
            }
            KeyCode::Up => {
                find.prev_match();
                self.jump_to_find_match();
            }
            KeyCode::Down => {
                find.next_match();
                self.jump_to_find_match();
            }
            KeyCode::Char(c) => {
                find.insert_char(c);
                find.search(&self.editor.buffer);
                self.jump_to_find_match();
            }
            _ => {}
        }
    }

    /// Move cursor to the current find match position, if any.
    fn jump_to_find_match(&mut self) {
        if let Some(find) = &self.find_state
            && let Some((line, col)) = find.current_match_pos()
        {
            self.editor.cursor_line = line;
            self.editor.cursor_col = col;
        }
    }

    /// Handle key input while the inline rename is active.
    fn handle_rename_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => self.rename.cancel(),
            KeyCode::Enter => self.rename.confirm(&mut self.persistence.file_path, &mut self.persistence.is_scratch),
            KeyCode::Backspace => self.rename.backspace(),
            KeyCode::Left => self.rename.cursor_left(),
            KeyCode::Right => self.rename.cursor_right(),
            KeyCode::Char(c) => self.rename.insert(c),
            _ => {}
        }
    }

    /// Handle key input while the Settings Layer is open.
    fn handle_settings_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => self.settings.dismiss(),
            KeyCode::Up | KeyCode::Char('k') => self.settings.nav_up(),
            KeyCode::Down | KeyCode::Char('j') => self.settings.nav_down(),
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
            self.palette
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Buffer;
    use crate::editing_mode::EditingMode;
    use crate::focus_mode::FocusMode;
    use crate::scroll_mode::ScrollMode;
    use crate::settings::SettingsItem;
    use crate::vim_bindings::{self, CursorShape};
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
        app.settings.dismiss();
        assert!(!app.settings.visible);
    }

    #[test]
    fn escape_dismisses_settings() {
        let mut app = App::new();
        app.toggle_settings();
        app.handle_key(KeyCode::Esc, KeyModifiers::NONE);
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
        app.editor.vim_mode = Mode::Insert;
        app.editor.handle_char('!');
        assert!(app.editor.dirty);

        let saved = app.persistence.autosave(&app.editor.buffer, &mut app.editor.dirty);
        assert!(saved);
        assert!(!app.editor.dirty);

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
        assert!(!app.persistence.autosave(&app.editor.buffer, &mut app.editor.dirty));
    }

    // === Acceptance test: Vim mode switch ===

    #[test]
    fn i_enters_insert_and_escape_returns_to_normal() {
        let mut app = App::new();
        assert_eq!(app.editor.vim_mode, Mode::Normal, "should start in Normal mode");
        app.editor.handle_char('i');
        assert_eq!(app.editor.vim_mode, Mode::Insert, "i should switch to Insert mode");
        app.editor.handle_escape();
        assert_eq!(app.editor.vim_mode, Mode::Normal, "Escape should return to Normal mode");
    }

    // === Acceptance test: Vim navigation motions ===

    #[test]
    fn w_moves_to_next_word() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("hello world\n");
        app.editor.cursor_col = 0;
        app.editor.handle_char('w');
        assert_eq!(app.editor.cursor_col, 6); // "world"
    }

    #[test]
    fn b_moves_to_previous_word() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("hello world\n");
        app.editor.cursor_col = 8;
        app.editor.handle_char('b');
        assert_eq!(app.editor.cursor_col, 6); // start of "world"
    }

    #[test]
    fn e_moves_to_end_of_word() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("hello world\n");
        app.editor.cursor_col = 0;
        app.editor.handle_char('e');
        assert_eq!(app.editor.cursor_col, 4); // 'o' in "hello"
    }

    #[test]
    fn zero_moves_to_line_start() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("hello world\n");
        app.editor.cursor_col = 5;
        app.editor.handle_char('0');
        assert_eq!(app.editor.cursor_col, 0);
    }

    #[test]
    fn dollar_moves_to_line_end() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("hello world\n");
        app.editor.cursor_col = 0;
        app.editor.handle_char('$');
        assert_eq!(app.editor.cursor_col, 10); // last char before newline
    }

    #[test]
    fn g_moves_to_last_line() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("line one\nline two\nline three\n");
        app.editor.cursor_line = 0;
        app.editor.handle_char('G');
        // Last line is the empty line after trailing newline (line 3)
        assert_eq!(app.editor.cursor_line, app.editor.buffer.len_lines() - 1);
    }

    #[test]
    fn x_deletes_char_under_cursor() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("abc\n");
        app.editor.cursor_col = 1;
        app.editor.handle_char('x');
        assert_eq!(app.editor.buffer.to_string(), "ac\n");
        assert!(app.editor.dirty);
    }

    #[test]
    fn o_opens_line_below() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("first\nsecond\n");
        app.editor.cursor_line = 0;
        app.editor.handle_char('o');
        assert_eq!(app.editor.vim_mode, Mode::Insert, "o should enter insert mode");
        assert_eq!(app.editor.cursor_line, 1, "o should move cursor to new line below");
        assert_eq!(app.editor.cursor_col, 0, "o should place cursor at column 0");
        assert_eq!(app.editor.buffer.len_lines(), 4, "o should insert a new blank line");
    }

    #[test]
    fn big_o_opens_line_above() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("first\nsecond\n");
        app.editor.cursor_line = 1;
        app.editor.handle_char('O');
        assert_eq!(app.editor.vim_mode, Mode::Insert, "O should enter insert mode");
        assert_eq!(app.editor.cursor_line, 1, "O should keep cursor on inserted blank line");
        assert_eq!(app.editor.cursor_col, 0, "O should place cursor at column 0");
        assert_eq!(app.editor.buffer.len_lines(), 4);
    }

    #[test]
    fn big_a_moves_to_end_and_enters_insert() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("hello\n");
        app.editor.cursor_col = 0;
        app.editor.handle_char('A');
        assert_eq!(app.editor.vim_mode, Mode::Insert, "A should enter Insert mode");
        // line has 6 chars (h,e,l,l,o,\n), max_col = 5
        assert_eq!(app.editor.cursor_col, 5, "A should move cursor to end of line");
    }

    // === Acceptance test: Multi-key sequences ===

    #[test]
    fn gg_goes_to_top() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("line one\nline two\nline three\n");
        app.editor.cursor_line = 2;
        app.editor.cursor_col = 3;
        app.editor.handle_char('g');
        app.editor.handle_char('g');
        assert_eq!(app.editor.cursor_line, 0, "gg should move cursor to first line");
        assert_eq!(app.editor.cursor_col, 0, "gg should move cursor to column 0");
    }

    #[test]
    fn dd_deletes_line() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("first\nsecond\nthird\n");
        app.editor.cursor_line = 1;
        app.editor.handle_char('d');
        app.editor.handle_char('d');
        let text = app.editor.buffer.to_string();
        assert!(!text.contains("second"), "Line should be deleted, got: {}", text);
        assert!(app.editor.dirty);
    }

    #[test]
    fn unknown_second_key_is_harmless() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("hello\n");
        app.editor.cursor_col = 2;
        let before = app.editor.buffer.to_string();
        app.editor.handle_char('g');
        app.editor.handle_char('z'); // unknown
        assert_eq!(app.editor.buffer.to_string(), before, "buffer should be unchanged after unknown sequence");
        assert_eq!(app.editor.cursor_col, 2, "cursor should be unchanged after unknown sequence");
    }

    // === Acceptance test: Typewriter mode centering ===

    #[test]
    fn typewriter_mode_centers_cursor() {
        let mut app = App::new();
        // Create a buffer with 20 lines
        let text = (0..20).map(|i| format!("Line {}\n", i)).collect::<String>();
        app.editor.buffer = Buffer::from_text(&text);
        app.viewport.scroll_mode = ScrollMode::Typewriter;
        app.editor.cursor_line = 10;
        app.editor.cursor_col = 0;

        let visual_lines = app.viewport.visual_lines(&app.editor.buffer);
        app.viewport.ensure_cursor_visible(app.editor.cursor_line, app.editor.cursor_col, &visual_lines, 10, &mut app.animations); // height 10

        // Cursor at visual line 10, height 10 → scroll_offset = 10 - 5 = 5
        assert_eq!(app.viewport.scroll_offset, 5, "scroll should center cursor in viewport");
        assert_eq!(app.viewport.typewriter_vertical_offset, 0, "no vertical offset needed when content fills above");
    }

    #[test]
    fn typewriter_mode_at_top_uses_vertical_offset() {
        let mut app = App::new();
        let text = (0..20).map(|i| format!("Line {}\n", i)).collect::<String>();
        app.editor.buffer = Buffer::from_text(&text);
        app.viewport.scroll_mode = ScrollMode::Typewriter;
        app.editor.cursor_line = 1;
        app.editor.cursor_col = 0;

        let visual_lines = app.viewport.visual_lines(&app.editor.buffer);
        app.viewport.ensure_cursor_visible(app.editor.cursor_line, app.editor.cursor_col, &visual_lines, 10, &mut app.animations);

        // Cursor at visual line 1, center = 5
        // Not enough content above → scroll stays 0, vertical offset pushes down
        assert_eq!(app.viewport.scroll_offset, 0, "scroll should stay at 0 when near top");
        assert_eq!(app.viewport.typewriter_vertical_offset, 4, "vertical offset should push content down to center cursor");
    }

    // === Acceptance test: Vim append mode ===

    #[test]
    fn a_enters_insert_with_cursor_one_right() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("hello\n");
        app.editor.cursor_line = 0;
        app.editor.cursor_col = 2; // on 'l'
        app.editor.handle_char('a');
        assert_eq!(app.editor.vim_mode, Mode::Insert, "a should enter Insert mode");
        assert_eq!(app.editor.cursor_col, 3, "a should advance cursor one position right");
    }

    #[test]
    fn a_at_end_of_line_enters_insert_at_end() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("hi\n");
        app.editor.cursor_line = 0;
        app.editor.cursor_col = 1; // on 'i', which is the last char before newline
        app.editor.handle_char('a');
        assert_eq!(app.editor.vim_mode, Mode::Insert, "a should enter Insert mode");
        // max_col = len_chars - 1 = 2 (h, i, \n → 3 chars, max=2)
        // cursor was at 1, < 2, so moves to 2
        assert_eq!(app.editor.cursor_col, 2, "a at end should move cursor to line end");
    }

    // === Unit test: Smart typography in insert mode ===

    #[test]
    fn smart_quotes_applied_during_insert() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("He said ");
        app.editor.cursor_line = 0;
        app.editor.cursor_col = 8;
        app.editor.vim_mode = Mode::Insert;

        app.editor.handle_char('"');
        let text = app.editor.buffer.to_string();
        assert!(text.contains('\u{201C}'), "Should have opening curly quote, got: {}", text);
    }

    // === Unit test: Cursor movement ===

    #[test]
    fn hjkl_moves_cursor() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("line one\nline two\nline three");
        app.editor.cursor_line = 1;
        app.editor.cursor_col = 3;

        app.editor.handle_char('h');
        assert_eq!(app.editor.cursor_col, 2, "h should move cursor left");
        app.editor.handle_char('l');
        assert_eq!(app.editor.cursor_col, 3, "l should move cursor right");
        app.editor.handle_char('k');
        assert_eq!(app.editor.cursor_line, 0, "k should move cursor up");
        app.editor.handle_char('j');
        assert_eq!(app.editor.cursor_line, 1, "j should move cursor down");
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
        app.settings.nav_down();
        assert_eq!(app.settings.cursor, 0, "nav down from last item should wrap to 0");
    }

    #[test]
    fn settings_nav_up_wraps() {
        let mut app = App::new();
        app.settings.cursor = 0;
        app.settings.nav_up();
        assert_eq!(app.settings.cursor, 11, "nav up from 0 should wrap to last item");
    }

    #[test]
    fn settings_nav_down_increments() {
        let mut app = App::new();
        app.settings.cursor = 2;
        app.settings.nav_down();
        assert_eq!(app.settings.cursor, 3);
    }

    #[test]
    fn settings_nav_up_decrements() {
        let mut app = App::new();
        app.settings.cursor = 5;
        app.settings.nav_up();
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
        assert_eq!(app.dimming.focus_mode, FocusMode::Sentence, "cursor 6 should select Sentence focus");

        app.settings.cursor = 7; // Paragraph
        app.settings_apply();
        assert_eq!(app.dimming.focus_mode, FocusMode::Paragraph, "cursor 7 should select Paragraph focus");
    }

    #[test]
    fn settings_apply_scroll_mode() {
        let mut app = App::new();
        app.settings.cursor = 9; // ScrollMode::Typewriter
        app.settings_apply();
        assert_eq!(app.viewport.scroll_mode, ScrollMode::Typewriter, "cursor 9 should select Typewriter scroll");

        app.settings.cursor = 8; // ScrollMode::Edge
        app.settings_apply();
        assert_eq!(app.viewport.scroll_mode, ScrollMode::Edge, "cursor 8 should select Edge scroll");
    }

    #[test]
    fn settings_apply_column_is_noop() {
        let mut app = App::new();
        app.settings.cursor = 10; // ColumnWidth
        let before = app.viewport.column_width;
        app.settings_apply();
        assert_eq!(app.viewport.column_width, before, "ColumnWidth row should not change width on Enter");
    }

    #[test]
    fn settings_adjust_column_increases() {
        let mut app = App::new();
        assert_eq!(app.viewport.column_width, 60, "default column width should be 60");
        app.settings_adjust_column(5);
        assert_eq!(app.viewport.column_width, 65, "adjusting +5 should increase to 65");
    }

    #[test]
    fn settings_adjust_column_clamps_low() {
        let mut app = App::new();
        app.viewport.column_width = 22;
        app.settings_adjust_column(-5);
        assert_eq!(app.viewport.column_width, 20, "column width should clamp at 20");
    }

    #[test]
    fn settings_adjust_column_clamps_high() {
        let mut app = App::new();
        app.viewport.column_width = 118;
        app.settings_adjust_column(5);
        assert_eq!(app.viewport.column_width, 120, "column width should clamp at 120");
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
        assert!(app.persistence.file_path.is_some());
        assert!(app.persistence.is_scratch);
    }

    #[test]
    fn scratch_enables_autosave() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = App::new();
        let name = crate::draft_name::generate();
        app.persistence.file_path = Some(dir.path().join(&name));
        app.persistence.is_scratch = true;
        app.editor.vim_mode = Mode::Insert;
        app.editor.handle_char('x');
        assert!(app.editor.dirty);
        let saved = app.persistence.autosave(&app.editor.buffer, &mut app.editor.dirty);
        assert!(saved, "scratch buffer should autosave");
    }

    #[test]
    fn explicit_file_is_not_scratch() {
        let tmp = NamedTempFile::new().unwrap();
        let app = App::new().with_file(tmp.path().to_path_buf(), "hello");
        assert!(!app.persistence.is_scratch);
        assert!(app.persistence.file_path.is_some());
    }

    // === Inline rename ===

    #[test]
    fn rename_open_seeds_buffer_with_filename() {
        let mut app = App::new();
        app.persistence.file_path = Some(PathBuf::from("/tmp/draft.md"));
        app.rename.open(app.persistence.file_path.as_deref());
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
        app.rename.insert('X');
        assert_eq!(app.rename.buf, "aXb");
        assert_eq!(app.rename.cursor, 2);
    }

    #[test]
    fn rename_insert_filters_slash() {
        let mut app = App::new();
        app.rename.active = true;
        app.rename.buf = "ab".to_string();
        app.rename.cursor = 1;
        app.rename.insert('/');
        assert_eq!(app.rename.buf, "ab");
        assert_eq!(app.rename.cursor, 1);
    }

    #[test]
    fn rename_backspace_deletes_before_cursor() {
        let mut app = App::new();
        app.rename.active = true;
        app.rename.buf = "abc".to_string();
        app.rename.cursor = 2;
        app.rename.backspace();
        assert_eq!(app.rename.buf, "ac");
        assert_eq!(app.rename.cursor, 1);
    }

    #[test]
    fn rename_backspace_at_start_is_noop() {
        let mut app = App::new();
        app.rename.active = true;
        app.rename.buf = "abc".to_string();
        app.rename.cursor = 0;
        app.rename.backspace();
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
        app.persistence.file_path = Some(PathBuf::from("/tmp/draft.md"));
        app.rename.open(app.persistence.file_path.as_deref());
        assert!(app.rename.active);

        app.rename.cancel();
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
        app.persistence.file_path = Some(old_path.clone());
        app.rename.open(app.persistence.file_path.as_deref());
        // Clear buffer and type new name
        app.rename.buf = "new.md".to_string();
        app.rename.cursor = 6;
        app.rename.confirm(&mut app.persistence.file_path, &mut app.persistence.is_scratch);

        assert!(!app.rename.active);
        let new_path = dir.path().join("new.md");
        assert_eq!(app.persistence.file_path, Some(new_path.clone()));
        assert!(new_path.exists());
        assert!(!old_path.exists());
    }

    #[test]
    fn rename_confirm_empty_name_cancels() {
        let mut app = App::new();
        app.persistence.file_path = Some(PathBuf::from("/tmp/draft.md"));
        app.rename.open(app.persistence.file_path.as_deref());
        app.rename.buf = "".to_string();
        app.rename.cursor = 0;
        app.rename.confirm(&mut app.persistence.file_path, &mut app.persistence.is_scratch);

        assert!(!app.rename.active);
        // file_path unchanged
        assert_eq!(app.persistence.file_path, Some(PathBuf::from("/tmp/draft.md")));
    }

    #[test]
    fn rename_confirm_clears_scratch_flag() {
        let dir = tempfile::tempdir().unwrap();
        let old_path = dir.path().join("scratch.md");
        std::fs::write(&old_path, "").unwrap();

        let mut app = App::new();
        app.persistence.file_path = Some(old_path);
        app.persistence.is_scratch = true;
        app.rename.open(app.persistence.file_path.as_deref());
        app.rename.buf = "real.md".to_string();
        app.rename.cursor = 7;
        app.rename.confirm(&mut app.persistence.file_path, &mut app.persistence.is_scratch);

        assert!(!app.persistence.is_scratch);
    }

    #[test]
    fn settings_apply_file_opens_rename() {
        let mut app = App::new();
        app.persistence.file_path = Some(PathBuf::from("/tmp/draft.md"));
        app.settings.cursor = 11; // File
        app.settings_apply();
        assert!(app.rename.active);
        assert_eq!(app.rename.buf, "draft.md");
    }

    #[test]
    fn rename_confirm_unsaved_scratch_updates_path_without_fs_rename() {
        // File doesn't exist on disk — should just update path
        let mut app = App::new();
        app.persistence.file_path = Some(PathBuf::from("/nonexistent/dir/scratch.md"));
        app.persistence.is_scratch = true;
        app.rename.open(app.persistence.file_path.as_deref());
        app.rename.buf = "real.md".to_string();
        app.rename.cursor = 7;
        app.rename.confirm(&mut app.persistence.file_path, &mut app.persistence.is_scratch);

        assert!(!app.rename.active);
        assert_eq!(
            app.persistence.file_path,
            Some(PathBuf::from("/nonexistent/dir/real.md"))
        );
        assert!(!app.persistence.is_scratch);
    }

    // === Editing mode tests ===

    #[test]
    fn settings_apply_switches_to_standard_mode() {
        let mut app = App::new();
        app.settings.cursor = 1; // Standard
        app.settings_apply();
        assert_eq!(app.editor.editing_mode, EditingMode::Standard);
        assert_eq!(app.editor.vim_mode, Mode::Insert);
    }

    #[test]
    fn settings_apply_switches_to_vim_mode() {
        let mut app = App::new();
        app.editor.editing_mode = EditingMode::Standard;
        app.editor.vim_mode = Mode::Insert;
        app.settings.cursor = 0; // Vim
        app.settings_apply();
        assert_eq!(app.editor.editing_mode, EditingMode::Vim);
        assert_eq!(app.editor.vim_mode, Mode::Normal);
    }

    #[test]
    fn switching_to_standard_clears_pending_normal_key() {
        let mut app = App::new();
        app.editor.pending_normal_key = Some('g');
        app.settings.cursor = 1; // Standard
        app.settings_apply();
        assert_eq!(app.editor.pending_normal_key, None);
    }

    // === Standard mode tests ===

    #[test]
    fn standard_mode_typing_inserts_directly() {
        let mut app = App::new();
        app.editor.editing_mode = EditingMode::Standard;
        app.editor.vim_mode = Mode::Insert;
        app.editor.buffer = Buffer::from_text("hello\n");
        app.editor.cursor_col = 5;
        app.editor.handle_char('!');
        assert_eq!(app.editor.buffer.to_string(), "hello!\n");
    }

    #[test]
    fn standard_mode_escape_clears_selection_not_mode() {
        let mut app = App::new();
        app.editor.editing_mode = EditingMode::Standard;
        app.editor.vim_mode = Mode::Insert;
        app.editor.selection_anchor = Some((0, 2));
        app.editor.handle_escape();
        assert_eq!(app.editor.selection_anchor, None);
        assert_eq!(app.editor.vim_mode, Mode::Insert); // stays in Insert
    }

    #[test]
    fn standard_mode_cursor_is_always_bar() {
        let mut app = App::new();
        app.editor.editing_mode = EditingMode::Standard;
        app.editor.vim_mode = Mode::Insert;
        assert_eq!(app.editor.cursor_shape(), CursorShape::Bar);
    }

    #[test]
    fn standard_mode_typing_replaces_selection() {
        let mut app = App::new();
        app.editor.editing_mode = EditingMode::Standard;
        app.editor.vim_mode = Mode::Insert;
        app.editor.buffer = Buffer::from_text("hello world\n");
        // Select "hello" (0,0) to (0,4)
        app.editor.selection_anchor = Some((0, 0));
        app.editor.cursor_col = 4;
        app.editor.handle_char('X');
        assert_eq!(app.editor.buffer.to_string(), "X world\n");
        assert_eq!(app.editor.selection_anchor, None);
    }

    #[test]
    fn standard_mode_q_inserts_q_not_quit() {
        let mut app = App::new();
        app.editor.editing_mode = EditingMode::Standard;
        app.editor.vim_mode = Mode::Insert;
        app.editor.buffer = Buffer::from_text("\n");
        app.editor.cursor_col = 0;
        app.editor.handle_char('q');
        assert!(!app.should_quit);
        assert!(app.editor.buffer.to_string().contains('q'));
    }

    // === Mode leakage prevention tests ===

    #[test]
    fn standard_mode_vim_mode_stays_insert() {
        let mut app = App::new();
        app.editor.editing_mode = EditingMode::Standard;
        app.editor.vim_mode = Mode::Insert;
        // SwitchMode should be ignored in Standard mode
        app.editor.apply_action(Action::SwitchMode(Mode::Normal));
        assert_eq!(app.editor.vim_mode, Mode::Insert);
    }

    #[test]
    fn standard_mode_escape_keeps_insert() {
        let mut app = App::new();
        app.editor.editing_mode = EditingMode::Standard;
        app.editor.vim_mode = Mode::Insert;
        app.editor.handle_escape();
        assert_eq!(app.editor.vim_mode, Mode::Insert);
    }

    #[test]
    fn standard_mode_vim_keys_insert_literally() {
        let mut app = App::new();
        app.editor.editing_mode = EditingMode::Standard;
        app.editor.vim_mode = Mode::Insert;
        app.editor.buffer = Buffer::from_text("\n");
        app.editor.cursor_col = 0;
        // 'i' should insert 'i', not switch to Insert mode
        app.editor.handle_char('i');
        assert!(app.editor.buffer.to_string().contains('i'));
        assert_eq!(app.editor.vim_mode, Mode::Insert);
    }

    // === Full round-trip test ===

    #[test]
    fn standard_mode_full_round_trip() {
        let mut app = App::new();
        // Switch to Standard mode
        app.settings.cursor = 1;
        app.settings_apply();
        assert_eq!(app.editor.editing_mode, EditingMode::Standard);
        assert_eq!(app.editor.vim_mode, Mode::Insert);

        // Type text
        app.editor.buffer = Buffer::from_text("\n");
        app.editor.cursor_col = 0;
        app.editor.handle_char('h');
        app.editor.handle_char('i');
        assert_eq!(app.editor.buffer.to_string(), "hi\n");

        // Select with shift+arrow (via extend_selection)
        use crossterm::event::KeyCode;
        app.editor.cursor_col = 0;
        app.editor.extend_selection(KeyCode::Right);
        app.editor.extend_selection(KeyCode::Right);
        assert_eq!(app.editor.selection_anchor, Some((0, 0)));
        assert_eq!(app.editor.cursor_col, 2);

        // Copy (simulating Ctrl+C behavior)
        if let Some(text) = app.editor.selected_text() {
            app.editor.yank_register = Some(text);
            app.editor.selection_anchor = None;
        }
        assert!(app.editor.yank_register.is_some());

        // Paste "hi" specifically to test the flow
        app.editor.yank_register = Some("hi".to_string());
        app.editor.cursor_col = 2;
        if let Some(text) = app.editor.yank_register.clone() {
            let idx = app.editor.cursor_char_index();
            app.editor.buffer.insert(idx, &text);
            app.editor.set_cursor_from_char_index(idx + text.chars().count());
            app.editor.dirty = true;
        }
        assert_eq!(app.editor.buffer.to_string(), "hihi\n");

        // Switch back to Vim mode
        app.settings.cursor = 0;
        app.settings_apply();
        assert_eq!(app.editor.editing_mode, EditingMode::Vim);
        assert_eq!(app.editor.vim_mode, Mode::Normal);
    }

    // === Find tests ===

    #[test]
    fn find_state_opens_and_closes() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("hello world\n");
        assert!(app.find_state.is_none());
        app.find_state = Some(crate::find::FindState::new(0, 0));
        assert!(app.find_state.is_some());
        app.find_state = None;
        assert!(app.find_state.is_none());
    }

    #[test]
    fn find_escape_restores_cursor() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("hello world\n");
        app.editor.cursor_line = 0;
        app.editor.cursor_col = 3;
        let fs = crate::find::FindState::new(0, 3);
        app.find_state = Some(fs);
        // Simulate moving cursor to a match
        app.editor.cursor_col = 6;
        // Cancel find — should restore
        let saved = app.find_state.as_ref().unwrap().saved_cursor;
        app.editor.cursor_line = saved.0;
        app.editor.cursor_col = saved.1;
        app.find_state = None;
        assert_eq!(app.editor.cursor_col, 3);
    }

    // === Undo/Redo integration tests ===

    #[test]
    fn undo_restores_previous_state() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("hello\n");
        app.editor.vim_mode = Mode::Insert;
        app.editor.cursor_col = 5;
        app.editor.handle_char('!');
        app.editor.undo_history.commit_group();
        assert_eq!(app.editor.buffer.to_string(), "hello!\n");
        app.editor.apply_action(Action::Undo);
        assert_eq!(app.editor.buffer.to_string(), "hello\n");
    }

    #[test]
    fn undo_then_redo_restores_change() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("hello\n");
        app.editor.vim_mode = Mode::Insert;
        app.editor.cursor_col = 5;
        app.editor.handle_char('!');
        app.editor.undo_history.commit_group();
        app.editor.apply_action(Action::Undo);
        assert_eq!(app.editor.buffer.to_string(), "hello\n");
        app.editor.apply_action(Action::Redo);
        assert_eq!(app.editor.buffer.to_string(), "hello!\n");
    }

    #[test]
    fn multiple_undos_walk_back_through_history() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("\n");
        app.editor.vim_mode = Mode::Insert;
        app.editor.cursor_col = 0;
        // Type "a " (space commits group)
        app.editor.handle_char('a');
        app.editor.handle_char(' ');
        // Type "b " (space commits group)
        app.editor.handle_char('b');
        app.editor.handle_char(' ');
        app.editor.undo_history.commit_group();
        assert_eq!(app.editor.buffer.to_string(), "a b \n");
        // Undo "b "
        app.editor.apply_action(Action::Undo);
        assert_eq!(app.editor.buffer.to_string(), "a \n");
        // Undo "a "
        app.editor.apply_action(Action::Undo);
        assert_eq!(app.editor.buffer.to_string(), "\n");
    }

    #[test]
    fn redo_cleared_on_new_edit_after_undo() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("\n");
        app.editor.vim_mode = Mode::Insert;
        app.editor.cursor_col = 0;
        app.editor.handle_char('a');
        app.editor.undo_history.commit_group();
        app.editor.apply_action(Action::Undo);
        // New edit
        app.editor.cursor_col = 0;
        app.editor.handle_char('b');
        app.editor.undo_history.commit_group();
        // Redo should not bring back 'a'
        app.editor.apply_action(Action::Redo);
        // Buffer should still be "b\n" — redo is no-op
        assert_eq!(app.editor.buffer.to_string(), "b\n");
    }

    #[test]
    fn delete_then_undo_restores_text() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("abc\n");
        app.editor.vim_mode = Mode::Insert;
        app.editor.cursor_col = 3;
        app.editor.apply_action(Action::DeleteBack);
        app.editor.undo_history.commit_group();
        assert_eq!(app.editor.buffer.to_string(), "ab\n");
        app.editor.apply_action(Action::Undo);
        assert_eq!(app.editor.buffer.to_string(), "abc\n");
    }

    #[test]
    fn empty_undo_redo_are_noops() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("hello\n");
        let before = app.editor.buffer.to_string();
        app.editor.apply_action(Action::Undo);
        assert_eq!(app.editor.buffer.to_string(), before);
        app.editor.apply_action(Action::Redo);
        assert_eq!(app.editor.buffer.to_string(), before);
    }

    // === Shift+Arrow selection tests ===

    #[test]
    fn shift_right_sets_anchor_and_extends() {
        use crossterm::event::KeyCode;
        let mut app = App::new();
        app.editor.editing_mode = EditingMode::Standard;
        app.editor.vim_mode = Mode::Insert;
        app.editor.buffer = Buffer::from_text("hello\n");
        app.editor.cursor_col = 0;
        app.editor.extend_selection(KeyCode::Right);
        assert_eq!(app.editor.selection_anchor, Some((0, 0)));
        assert_eq!(app.editor.cursor_col, 1);
    }

    #[test]
    fn shift_left_extends_backward() {
        use crossterm::event::KeyCode;
        let mut app = App::new();
        app.editor.editing_mode = EditingMode::Standard;
        app.editor.vim_mode = Mode::Insert;
        app.editor.buffer = Buffer::from_text("hello\n");
        app.editor.cursor_col = 3;
        app.editor.extend_selection(KeyCode::Left);
        assert_eq!(app.editor.selection_anchor, Some((0, 3)));
        assert_eq!(app.editor.cursor_col, 2);
    }

    #[test]
    fn shift_down_extends_to_next_line() {
        use crossterm::event::KeyCode;
        let mut app = App::new();
        app.editor.editing_mode = EditingMode::Standard;
        app.editor.vim_mode = Mode::Insert;
        app.editor.buffer = Buffer::from_text("hello\nworld\n");
        app.editor.cursor_line = 0;
        app.editor.cursor_col = 2;
        app.editor.extend_selection(KeyCode::Down);
        assert_eq!(app.editor.selection_anchor, Some((0, 2)));
        assert_eq!(app.editor.cursor_line, 1);
    }

    #[test]
    fn shift_home_selects_to_line_start() {
        use crossterm::event::KeyCode;
        let mut app = App::new();
        app.editor.editing_mode = EditingMode::Standard;
        app.editor.vim_mode = Mode::Insert;
        app.editor.buffer = Buffer::from_text("hello\n");
        app.editor.cursor_col = 3;
        app.editor.extend_selection(KeyCode::Home);
        assert_eq!(app.editor.selection_anchor, Some((0, 3)));
        assert_eq!(app.editor.cursor_col, 0);
    }

    #[test]
    fn shift_end_selects_to_line_end() {
        use crossterm::event::KeyCode;
        let mut app = App::new();
        app.editor.editing_mode = EditingMode::Standard;
        app.editor.vim_mode = Mode::Insert;
        app.editor.buffer = Buffer::from_text("hello\n");
        app.editor.cursor_col = 0;
        app.editor.extend_selection(KeyCode::End);
        assert_eq!(app.editor.selection_anchor, Some((0, 0)));
        assert_eq!(app.editor.cursor_col, 5); // past 'o', end of visible content
    }

    #[test]
    fn multiple_shift_arrows_accumulate_selection() {
        use crossterm::event::KeyCode;
        let mut app = App::new();
        app.editor.editing_mode = EditingMode::Standard;
        app.editor.vim_mode = Mode::Insert;
        app.editor.buffer = Buffer::from_text("hello\n");
        app.editor.cursor_col = 0;
        app.editor.extend_selection(KeyCode::Right);
        app.editor.extend_selection(KeyCode::Right);
        app.editor.extend_selection(KeyCode::Right);
        assert_eq!(app.editor.selection_anchor, Some((0, 0)));
        assert_eq!(app.editor.cursor_col, 3);
    }

    #[test]
    fn shift_arrow_in_vim_enters_visual() {
        use crossterm::event::KeyCode;
        let mut app = App::new();
        app.editor.editing_mode = EditingMode::Vim;
        app.editor.vim_mode = Mode::Normal;
        app.editor.buffer = Buffer::from_text("hello\n");
        app.editor.cursor_col = 0;
        app.editor.extend_selection(KeyCode::Right);
        assert_eq!(app.editor.vim_mode, Mode::Visual);
        assert_eq!(app.editor.selection_anchor, Some((0, 0)));
    }

    // === Visual mode tests ===

    #[test]
    fn v_enters_visual_with_correct_anchor() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("hello world\n");
        app.editor.cursor_col = 3;
        app.editor.handle_char('v');
        assert_eq!(app.editor.vim_mode, Mode::Visual);
        assert_eq!(app.editor.selection_anchor, Some((0, 3)));
    }

    #[test]
    fn movement_in_visual_preserves_anchor() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("hello world\n");
        app.editor.cursor_col = 3;
        app.editor.handle_char('v');
        app.editor.handle_char('l'); // move right
        app.editor.handle_char('l');
        assert_eq!(app.editor.vim_mode, Mode::Visual);
        assert_eq!(app.editor.selection_anchor, Some((0, 3))); // anchor unchanged
        assert_eq!(app.editor.cursor_col, 5); // cursor moved
    }

    #[test]
    fn escape_clears_selection_and_returns_to_normal() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("hello\n");
        app.editor.handle_char('v');
        assert_eq!(app.editor.vim_mode, Mode::Visual);
        app.editor.handle_escape();
        assert_eq!(app.editor.vim_mode, Mode::Normal);
        assert_eq!(app.editor.selection_anchor, None);
    }

    #[test]
    fn y_yanks_correct_text_to_register() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("hello world\n");
        app.editor.cursor_col = 0;
        app.editor.handle_char('v'); // anchor at (0,0)
        app.editor.handle_char('l');
        app.editor.handle_char('l');
        app.editor.handle_char('l');
        app.editor.handle_char('l'); // cursor at (0,4) = 'o'
        app.editor.handle_char('y');
        assert_eq!(app.editor.vim_mode, Mode::Normal);
        assert_eq!(app.editor.yank_register, Some("hello".to_string()));
        assert_eq!(app.editor.selection_anchor, None);
    }

    #[test]
    fn d_deletes_selection_and_yanks_to_register() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("hello world\n");
        app.editor.cursor_col = 0;
        app.editor.handle_char('v');
        app.editor.handle_char('l');
        app.editor.handle_char('l');
        app.editor.handle_char('l');
        app.editor.handle_char('l'); // select "hello"
        app.editor.handle_char('d');
        assert_eq!(app.editor.vim_mode, Mode::Normal);
        assert_eq!(app.editor.yank_register, Some("hello".to_string()));
        assert_eq!(app.editor.buffer.to_string(), " world\n");
        assert!(app.editor.dirty);
    }

    #[test]
    fn selection_range_normalizes_when_anchor_after_cursor() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("hello world\n");
        app.editor.cursor_col = 8;
        app.editor.handle_char('v'); // anchor at (0,8)
        app.editor.handle_char('h');
        app.editor.handle_char('h');
        app.editor.handle_char('h'); // cursor at (0,5)
        let range = app.editor.selection_range().unwrap();
        assert_eq!(range, (0, 5, 0, 8)); // normalized: start < end
    }

    #[test]
    fn selected_text_works_multiline() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("hello\nworld\n");
        app.editor.cursor_col = 3;
        app.editor.handle_char('v'); // anchor at (0,3)
        app.editor.handle_char('j'); // move down to line 1
        app.editor.handle_char('l'); // cursor at (1,4)
        let text = app.editor.selected_text().unwrap();
        assert_eq!(text, "lo\nworld");
    }

    #[test]
    fn gg_works_in_visual_mode() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("first\nsecond\nthird\n");
        app.editor.cursor_line = 2;
        app.editor.handle_char('v');
        app.editor.handle_char('g');
        app.editor.handle_char('g');
        assert_eq!(app.editor.cursor_line, 0);
        assert_eq!(app.editor.vim_mode, Mode::Visual); // stays in Visual
    }

    #[test]
    fn q_does_not_quit_in_visual_mode() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("hello\n");
        app.editor.handle_char('v');
        app.editor.handle_char('q'); // should be a no-op, not quit
        assert!(!app.should_quit);
        assert_eq!(app.editor.vim_mode, Mode::Visual);
    }

    // === Paste tests ===

    #[test]
    fn p_inserts_register_content_after_cursor_charwise() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("ab\n");
        app.editor.cursor_col = 0;
        app.editor.yank_register = Some("XY".to_string());
        app.editor.handle_char('p');
        assert_eq!(app.editor.buffer.to_string(), "aXYb\n");
        assert!(app.editor.dirty);
    }

    #[test]
    fn big_p_inserts_before_cursor_charwise() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("ab\n");
        app.editor.cursor_col = 1;
        app.editor.yank_register = Some("XY".to_string());
        app.editor.handle_char('P');
        assert_eq!(app.editor.buffer.to_string(), "aXYb\n");
    }

    #[test]
    fn p_multiline_inserts_on_next_line() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("first\nsecond\n");
        app.editor.cursor_line = 0;
        app.editor.yank_register = Some("new\n".to_string());
        app.editor.handle_char('p');
        let text = app.editor.buffer.to_string();
        assert_eq!(text, "first\nnew\nsecond\n");
        assert_eq!(app.editor.cursor_line, 1);
    }

    #[test]
    fn big_p_multiline_inserts_on_current_line() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("first\nsecond\n");
        app.editor.cursor_line = 1;
        app.editor.yank_register = Some("new\n".to_string());
        app.editor.handle_char('P');
        let text = app.editor.buffer.to_string();
        assert_eq!(text, "first\nnew\nsecond\n");
    }

    #[test]
    fn empty_register_paste_is_noop() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("hello\n");
        app.editor.yank_register = None;
        let before = app.editor.buffer.to_string();
        app.editor.handle_char('p');
        assert_eq!(app.editor.buffer.to_string(), before);
        assert!(!app.editor.dirty);
    }

    #[test]
    fn dd_populates_register_with_deleted_line() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("first\nsecond\nthird\n");
        app.editor.cursor_line = 1;
        app.editor.handle_char('d');
        app.editor.handle_char('d');
        assert_eq!(app.editor.yank_register, Some("second\n".to_string()));
        assert!(!app.editor.buffer.to_string().contains("second"));
    }

    // === Ctrl key operation tests ===

    #[test]
    fn delete_selection_silent_removes_without_yanking() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("hello world\n");
        app.editor.selection_anchor = Some((0, 0));
        app.editor.cursor_col = 4;
        app.editor.yank_register = None;
        app.editor.delete_selection_silent();
        assert_eq!(app.editor.buffer.to_string(), " world\n");
        assert_eq!(app.editor.yank_register, None);
        assert!(app.editor.dirty);
    }

    #[test]
    fn select_all_sets_anchor_and_moves_cursor_to_end() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("hello\nworld\n");
        // Simulate Ctrl+A behavior
        let total_chars = app.editor.buffer.len_chars();
        app.editor.selection_anchor = Some((0, 0));
        app.editor.set_cursor_from_char_index(total_chars.saturating_sub(1));
        assert_eq!(app.editor.selection_anchor, Some((0, 0)));
        assert!(app.editor.cursor_line > 0 || app.editor.cursor_col > 0);
    }

    #[test]
    fn scroll_animation_starts_on_scroll_change() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text(&"line\n".repeat(50));
        app.viewport.scroll_display = 0.0;
        app.viewport.scroll_offset = 0;
        let visual_lines = app.viewport.visual_lines(&app.editor.buffer);
        app.editor.cursor_line = 30;
        app.editor.cursor_col = 0;
        app.viewport.ensure_cursor_visible(app.editor.cursor_line, app.editor.cursor_col, &visual_lines, 20, &mut app.animations);
        assert!(app.viewport.scroll_offset > 0);
        assert!(app.animations.is_active());
        assert!(app.animations.scroll_progress().is_some());
    }

    #[test]
    fn yank_then_paste_round_trip() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("hello world\n");
        app.editor.cursor_col = 0;
        // Select "hello"
        app.editor.handle_char('v');
        app.editor.handle_char('l');
        app.editor.handle_char('l');
        app.editor.handle_char('l');
        app.editor.handle_char('l');
        app.editor.handle_char('y');
        // Move to end of "world"
        app.editor.handle_char('$');
        // Paste after
        app.editor.handle_char('p');
        let text = app.editor.buffer.to_string();
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
        app.settings.dismiss();
        // No overlay animation started on dismiss
        assert!(app.animations.overlay_progress().is_none());
    }

    // === Task 7: DimLayer wired into App ===

    #[test]
    fn dim_layers_produce_opacities() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("Line 1\n\nLine 3\nLine 4");
        app.dimming.focus_mode = FocusMode::Paragraph;
        app.editor.cursor_line = 0;
        app.dimming.update(app.editor.buffer.len_lines(), app.editor.paragraph_bounds(), app.editor.sentence_bounds());
        let opacities = app.dimming.line_opacities();
        assert_eq!(opacities.len(), 4);
        assert!((opacities[0] - 1.0).abs() < f64::EPSILON, "Cursor line should be bright");
        assert!(opacities[2] < 1.0, "Other paragraph should be dimmed");
    }

    #[test]
    fn sentence_fade_animates_on_sentence_change() {
        let mut app = App::new();
        // Two sentences on separate lines
        app.editor.buffer = Buffer::from_text("First sentence.\nSecond sentence.");
        app.dimming.focus_mode = FocusMode::Sentence;
        app.editor.cursor_line = 0;
        app.editor.cursor_col = 0;

        app.dimming.update(app.editor.buffer.len_lines(), app.editor.paragraph_bounds(), app.editor.sentence_bounds());

        // No fades in progress initially
        assert!(app.dimming.sentence_fades_is_empty());

        // Move cursor to line 1 (second sentence)
        app.editor.cursor_line = 1;
        app.editor.cursor_col = 0;
        app.dimming.update(app.editor.buffer.len_lines(), app.editor.paragraph_bounds(), app.editor.sentence_bounds());

        // One fade should be queued for the old sentence
        assert_eq!(app.dimming.sentence_fades_len(), 1, "should have one fading sentence");
        let snap = app.dimming.sentence_fade_snapshot();
        assert!(snap[0].2 > 0.9, "Opacity should be near 1.0 right after change");
        assert!(app.dimming.dim_animating(), "should be animating");
    }

    #[test]
    fn focus_off_all_bright() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("Line 1\nLine 2\nLine 3");
        app.dimming.focus_mode = FocusMode::Off;
        app.dimming.update(app.editor.buffer.len_lines(), app.editor.paragraph_bounds(), app.editor.sentence_bounds());
        let opacities = app.dimming.line_opacities();
        for (i, &o) in opacities.iter().enumerate() {
            assert!((o - 1.0).abs() < f64::EPSILON, "Line {} should be bright in Off mode", i);
        }
    }

    // === Sentence fade queue regression tests ===

    #[test]
    fn rapid_typing_after_period_preserves_fade() {
        let mut app = App::new();
        app.dimming.focus_mode = FocusMode::Sentence;

        // Cursor in "Hello world."
        app.editor.buffer = Buffer::from_text("Hello world.");
        app.editor.cursor_line = 0;
        app.editor.cursor_col = 5;
        app.dimming.update(app.editor.buffer.len_lines(), app.editor.paragraph_bounds(), app.editor.sentence_bounds());
        assert!(app.dimming.sentence_fades_is_empty(), "No fades initially");

        // Simulate typing space after period: "Hello world. "
        app.editor.buffer = Buffer::from_text("Hello world. ");
        app.editor.cursor_col = 12;
        app.dimming.update(app.editor.buffer.len_lines(), app.editor.paragraph_bounds(), app.editor.sentence_bounds());
        assert_eq!(app.dimming.sentence_fades_len(), 1, "One fade after leaving sentence");
        let original_start = app.dimming.sentence_fade_start(0);

        // Simulate typing 'T': "Hello world. T"
        app.editor.buffer = Buffer::from_text("Hello world. T");
        app.editor.cursor_col = 13;
        app.dimming.update(app.editor.buffer.len_lines(), app.editor.paragraph_bounds(), app.editor.sentence_bounds());

        // The original "Hello world." fade must survive the second sentence change
        assert!(
            app.dimming.sentence_fade_has_start(original_start),
            "Original sentence fade must survive rapid typing"
        );
    }

    #[test]
    fn multiple_sentences_fade_independently() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("First.\n\nSecond.\n\nThird.");
        app.dimming.focus_mode = FocusMode::Sentence;

        // Start in first sentence
        app.editor.cursor_line = 0;
        app.editor.cursor_col = 0;
        app.dimming.update(app.editor.buffer.len_lines(), app.editor.paragraph_bounds(), app.editor.sentence_bounds());

        // Move to second sentence
        app.editor.cursor_line = 2;
        app.editor.cursor_col = 0;
        app.dimming.update(app.editor.buffer.len_lines(), app.editor.paragraph_bounds(), app.editor.sentence_bounds());
        assert_eq!(app.dimming.sentence_fades_len(), 1);

        // Move to third sentence
        app.editor.cursor_line = 4;
        app.editor.cursor_col = 0;
        app.dimming.update(app.editor.buffer.len_lines(), app.editor.paragraph_bounds(), app.editor.sentence_bounds());
        assert_eq!(app.dimming.sentence_fades_len(), 2, "Two sentences should be fading simultaneously");

        // Both should have high opacity (just started or recently started)
        let snap = app.dimming.sentence_fade_snapshot();
        assert!(snap[0].2 > 0.5, "First fade should still be in progress");
        assert!(snap[1].2 > 0.9, "Second fade should have just started");
    }

    #[test]
    fn returning_to_fading_sentence_reverses_fade() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("First.\n\nSecond.");
        app.dimming.focus_mode = FocusMode::Sentence;

        app.editor.cursor_line = 0;
        app.editor.cursor_col = 0;
        app.dimming.update(app.editor.buffer.len_lines(), app.editor.paragraph_bounds(), app.editor.sentence_bounds());

        // Move to second — first starts fading toward 0.6
        app.editor.cursor_line = 2;
        app.editor.cursor_col = 0;
        app.dimming.update(app.editor.buffer.len_lines(), app.editor.paragraph_bounds(), app.editor.sentence_bounds());
        assert_eq!(app.dimming.sentence_fades_len(), 1);
        assert!((app.dimming.sentence_fade_target(0) - 0.6).abs() < f64::EPSILON);

        // Return to first — should reverse that entry toward 1.0
        app.editor.cursor_line = 0;
        app.editor.cursor_col = 0;
        app.dimming.update(app.editor.buffer.len_lines(), app.editor.paragraph_bounds(), app.editor.sentence_bounds());
        assert!((app.dimming.sentence_fade_target(0) - 1.0).abs() < f64::EPSILON, "Should reverse to 1.0");
    }

    #[test]
    fn completed_fades_are_pruned() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("First.\n\nSecond.");
        app.dimming.focus_mode = FocusMode::Sentence;

        app.editor.cursor_line = 0;
        app.editor.cursor_col = 0;
        app.dimming.update(app.editor.buffer.len_lines(), app.editor.paragraph_bounds(), app.editor.sentence_bounds());

        app.editor.cursor_line = 2;
        app.editor.cursor_col = 0;
        app.dimming.update(app.editor.buffer.len_lines(), app.editor.paragraph_bounds(), app.editor.sentence_bounds());
        assert_eq!(app.dimming.sentence_fades_len(), 1);

        // Backdate the animation past its 1800ms duration
        app.dimming.backdate_sentence_fade(0, Duration::from_millis(2000));

        // Next update should prune the completed entry
        app.dimming.update(app.editor.buffer.len_lines(), app.editor.paragraph_bounds(), app.editor.sentence_bounds());
        assert!(app.dimming.sentence_fades_is_empty(), "Completed fade should be pruned");
    }

    // === Cursor navigation tests ===

    #[test]
    fn cursor_right_reaches_end_of_last_line_without_newline() {
        let mut app = App::new();
        app.editor.editing_mode = EditingMode::Standard;
        app.editor.vim_mode = Mode::Insert;
        app.editor.buffer = Buffer::from_text("hello");
        app.editor.cursor_line = 0;
        app.editor.cursor_col = 0;

        // Press right 5 times to reach past the last character
        for _ in 0..5 {
            app.editor.move_cursor(vim_bindings::Direction::Right);
        }

        // In standard/insert mode, cursor should be at position 5 (after 'o')
        assert_eq!(app.editor.cursor_col, 5, "Cursor should be past the last character");
    }

    #[test]
    fn line_end_reaches_end_of_line() {
        let mut app = App::new();
        app.editor.editing_mode = EditingMode::Standard;
        app.editor.vim_mode = Mode::Insert;
        app.editor.buffer = Buffer::from_text("hello\nworld");
        app.editor.cursor_line = 0;
        app.editor.cursor_col = 0;

        // LineEnd should go to position after last visible char
        app.editor.apply_action(Action::LineEnd);
        assert_eq!(app.editor.cursor_col, 5, "End should place cursor after 'o' on line with newline");

        // Same for last line (no newline)
        app.editor.cursor_line = 1;
        app.editor.cursor_col = 0;
        app.editor.apply_action(Action::LineEnd);
        assert_eq!(app.editor.cursor_col, 5, "End should place cursor after 'd' on last line");
    }

    #[test]
    fn clamp_cursor_col_allows_end_of_line() {
        let mut app = App::new();
        app.editor.editing_mode = EditingMode::Standard;
        app.editor.vim_mode = Mode::Insert;
        app.editor.buffer = Buffer::from_text("hello");
        app.editor.cursor_line = 0;
        app.editor.cursor_col = 5; // past last char

        app.editor.clamp_cursor_col();
        assert_eq!(app.editor.cursor_col, 5, "Clamp should allow cursor past last char in insert mode");
    }

    #[test]
    fn mode_switch_clears_fade_queue() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("First.\n\nSecond.");
        app.dimming.focus_mode = FocusMode::Sentence;

        app.editor.cursor_line = 0;
        app.editor.cursor_col = 0;
        app.dimming.update(app.editor.buffer.len_lines(), app.editor.paragraph_bounds(), app.editor.sentence_bounds());

        app.editor.cursor_line = 2;
        app.editor.cursor_col = 0;
        app.dimming.update(app.editor.buffer.len_lines(), app.editor.paragraph_bounds(), app.editor.sentence_bounds());
        assert!(!app.dimming.sentence_fades_is_empty());

        // Switch to Off
        app.dimming.focus_mode = FocusMode::Off;
        app.dimming.update(app.editor.buffer.len_lines(), app.editor.paragraph_bounds(), app.editor.sentence_bounds());
        assert!(app.dimming.sentence_fades_is_empty(), "Off mode should clear all fades");
    }

    // === Visual-line cursor navigation tests ===

    /// Helper: create an App with given text and column_width, cursor at (0, 0).
    fn app_with_wrap(text: &str, width: u16) -> App {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text(text);
        app.viewport.column_width = width;
        app.editor.cursor_line = 0;
        app.editor.cursor_col = 0;
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
        app.editor.cursor_col = 0; // on vl0

        app.editor.move_cursor_with_width(vim_bindings::Direction::Down, app.viewport.column_width);
        // Should move to vl1, same logical line
        assert_eq!(app.editor.cursor_line, 0, "stays on same logical line");
        assert_eq!(app.editor.cursor_col, 6, "moved to start of next visual line");
    }

    #[test]
    fn visual_nav_up_within_wrapped_line() {
        let mut app = app_with_wrap("hello world foo", 6);
        // Start on second visual line
        app.editor.cursor_col = 6; // vl1 start

        app.editor.move_cursor_with_width(vim_bindings::Direction::Up, app.viewport.column_width);
        assert_eq!(app.editor.cursor_line, 0, "stays on same logical line");
        assert_eq!(app.editor.cursor_col, 0, "moved to start of first visual line");
    }

    #[test]
    fn visual_nav_down_crosses_logical_line() {
        let mut app = app_with_wrap("short\nother", 60);
        // Width 60 means no wrapping; two visual lines, one per logical line
        app.editor.cursor_col = 2;

        app.editor.move_cursor_with_width(vim_bindings::Direction::Down, app.viewport.column_width);
        assert_eq!(app.editor.cursor_line, 1, "moved to next logical line");
        assert_eq!(app.editor.cursor_col, 2, "preserved visual column");
    }

    #[test]
    fn visual_nav_clamps_col_on_shorter_target() {
        let mut app = app_with_wrap("longline\nhi", 60);
        app.editor.cursor_col = 7; // near end of "longline"

        app.editor.move_cursor_with_width(vim_bindings::Direction::Down, app.viewport.column_width);
        assert_eq!(app.editor.cursor_line, 1);
        // "hi" has length 2, so col should clamp to 2
        assert_eq!(app.editor.cursor_col, 2, "clamped to end of shorter line");
    }

    #[test]
    fn visual_nav_up_on_first_line_is_noop() {
        let mut app = app_with_wrap("hello\nworld", 60);
        app.editor.cursor_col = 3;

        app.editor.move_cursor_with_width(vim_bindings::Direction::Up, app.viewport.column_width);
        assert_eq!(app.editor.cursor_line, 0);
        assert_eq!(app.editor.cursor_col, 3, "cursor unchanged");
    }

    #[test]
    fn visual_nav_down_on_last_line_is_noop() {
        let mut app = app_with_wrap("hello\nworld", 60);
        app.editor.cursor_line = 1;
        app.editor.cursor_col = 3;

        app.editor.move_cursor_with_width(vim_bindings::Direction::Down, app.viewport.column_width);
        assert_eq!(app.editor.cursor_line, 1);
        assert_eq!(app.editor.cursor_col, 3, "cursor unchanged");
    }

    #[test]
    fn visual_nav_through_empty_line() {
        let mut app = app_with_wrap("above\n\nbelow", 60);
        app.editor.cursor_col = 3;

        // Down from "above" -> empty line
        app.editor.move_cursor_with_width(vim_bindings::Direction::Down, app.viewport.column_width);
        assert_eq!(app.editor.cursor_line, 1);
        assert_eq!(app.editor.cursor_col, 0, "empty line clamps to 0");

        // Down from empty line -> "below"
        app.editor.move_cursor_with_width(vim_bindings::Direction::Down, app.viewport.column_width);
        assert_eq!(app.editor.cursor_line, 2);
        assert_eq!(app.editor.cursor_col, 0, "visual col was 0 from empty line");
    }

    #[test]
    fn visual_nav_traverses_all_visual_lines_of_wrapped_paragraph() {
        // "hello world foo" at width 6 produces 3 visual lines, all logical line 0
        let mut app = app_with_wrap("hello world foo", 6);

        // Collect cursor positions going down
        let mut positions = vec![(app.editor.cursor_line, app.editor.cursor_col)];
        for _ in 0..5 {
            let prev = (app.editor.cursor_line, app.editor.cursor_col);
            app.editor.move_cursor_with_width(vim_bindings::Direction::Down, app.viewport.column_width);
            let curr = (app.editor.cursor_line, app.editor.cursor_col);
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
            app.editor.move_cursor_with_width(vim_bindings::Direction::Up, app.viewport.column_width);
        }
        assert_eq!(app.editor.cursor_col, 0, "back at start");
    }

    // === Undo recording tests for Ctrl-chord actions ===

    #[test]
    fn ctrl_x_records_undo() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("hello world");
        // Select "hello" (chars 0..4)
        app.editor.selection_anchor = Some((0, 0));
        app.editor.cursor_line = 0;
        app.editor.cursor_col = 4;
        app.editor.vim_mode = Mode::Visual;
        // Cut via Ctrl+X (now routed through DeleteSelection)
        app.handle_key(KeyCode::Char('x'), KeyModifiers::CONTROL);
        assert_eq!(app.editor.buffer.to_string(), " world", "selection should be deleted");
        // Undo should restore
        app.editor.apply_action(Action::Undo);
        assert_eq!(app.editor.buffer.to_string(), "hello world", "undo should restore deleted text");
    }

    #[test]
    fn ctrl_v_paste_at_cursor_records_undo() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("world");
        app.editor.cursor_line = 0;
        app.editor.cursor_col = 0;
        app.editor.yank_register = Some("hello ".to_string());
        // Paste via Ctrl+V
        app.handle_key(KeyCode::Char('v'), KeyModifiers::CONTROL);
        assert_eq!(app.editor.buffer.to_string(), "hello world", "text should be pasted");
        // Undo should restore
        app.editor.apply_action(Action::Undo);
        assert_eq!(app.editor.buffer.to_string(), "world", "undo should remove pasted text");
    }

    #[test]
    fn paste_after_records_undo() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("hello");
        app.editor.cursor_line = 0;
        app.editor.cursor_col = 4;
        app.editor.yank_register = Some(" world".to_string());
        app.editor.apply_action(Action::PasteAfter);
        assert_eq!(app.editor.buffer.to_string(), "hello world", "text should be pasted after cursor");
        app.editor.apply_action(Action::Undo);
        assert_eq!(app.editor.buffer.to_string(), "hello", "undo should remove pasted text");
    }
}
