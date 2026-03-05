use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use crossterm::event::{KeyCode, KeyModifiers};

use crate::animation::AnimationManager;
use crate::buffer::Buffer;
use crate::config::Config;
use crate::color_profile::ColorProfile;
use crate::dimming::DimmingState;
use crate::editing_mode::EditingMode;
use crate::editor::Editor;
use crate::find::FindState;
use crate::focus_mode::FocusMode;
use crate::markdown_styling::CharStyle;
use crate::palette::Palette;
use crate::persistence::Persistence;
use crate::scroll_mode::ScrollMode;
use crate::settings::{RenameState, ScratchQuitAction, ScratchQuitState, SettingsItem, SettingsState};
use crate::vim_bindings::{Action, CursorShape, Mode};
use crate::viewport::Viewport;
use crate::wrap::VisualLine;
use crate::writing_surface::RenderCache;

/// Per-frame state update output, consumed by the draw call.
pub struct TickOutput {
    pub visual_lines: Rc<[VisualLine]>,
    pub sentence_bounds: Option<(usize, usize)>,
}

/// Thin coordinator that owns subsystems and routes input between them.
///
/// ## Coordinator invariant
///
/// App must contain only routing (pure delegation) and coordination
/// (orchestrating multiple subsystems). Domain logic — business rules,
/// calculations, state machine transitions — belongs in the subsystem
/// that owns the relevant state. When adding a method to App, ask:
/// "Does this read/write state from only one subsystem?" If yes, it
/// belongs on that subsystem, not here.
pub struct App {
    pub(crate) editor: Editor,
    pub(crate) viewport: Viewport,
    pub(crate) palette: Palette,
    pub(crate) dimming: DimmingState,
    pub(crate) color_profile: ColorProfile,
    pub(crate) settings: SettingsState,
    should_quit: bool,
    pub(crate) persistence: Persistence,
    pub(crate) rename: RenameState,
    /// Find overlay state (None when find is not active).
    pub(crate) find_state: Option<FindState>,
    pub(crate) animations: AnimationManager,
    pub(crate) render_cache: RenderCache,
    /// Whether the next frame requires a full redraw.
    pub(crate) needs_redraw: bool,
    /// True when the file was modified externally while the buffer is dirty.
    external_change_pending: bool,
    scratch_quit: ScratchQuitState,
    /// Quit after the rename completes (set when scratch quit → Rename).
    pending_quit_after_rename: bool,
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
            external_change_pending: false,
            scratch_quit: ScratchQuitState::new(),
            pending_quit_after_rename: false,
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
        app.editor.set_editing_mode(config.editing_mode);
        if let Some(ref path) = file_path {
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    app = app.with_file(path.clone(), &content);
                }
                Err(e) => {
                    app.persistence.load_error = Some(e.to_string());
                    app = app.with_file(path.clone(), "");
                }
            }
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
                TransitionKind::SettingsOverlay,
                Duration::from_millis(150),
                Easing::EaseOut,
            );
            // Find the index of the active palette in the full settings item list
            let items = SettingsItem::all();
            let target = SettingsItem::Palette(self.palette.index_in_all());
            self.settings.cursor = items.iter().position(|i| *i == target).unwrap_or(0);
        }
    }

    /// Switch to a different Palette.
    pub fn set_palette(&mut self, palette: Palette) {
        debug_assert!(palette.validate().is_ok(), "Palette {:?} failed validation", palette.name);
        self.palette = palette;
    }

    /// Apply the currently selected settings item.
    pub fn settings_apply(&mut self) {
        let Some(item) = SettingsItem::at(self.settings.cursor) else {
            return;
        };
        match item {
            SettingsItem::EditingMode(mode) => {
                self.editor.set_editing_mode(mode);
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

        // Conflict bar — swallow all keys except Ctrl (handled above)
        if self.external_change_pending {
            match code {
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    self.external_change_pending = false;
                    self.reload_from_disk();
                }
                KeyCode::Char('k') | KeyCode::Char('K') => {
                    self.external_change_pending = false;
                    self.persistence.record_mtime();
                }
                _ => {} // swallow other keys
            }
            return;
        }

        // Scratch quit prompt — swallow all keys when active
        if self.scratch_quit.active {
            self.handle_scratch_quit_key(code);
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
        if self.editor.handle_key(code, modifiers, self.viewport.effective_column_width) {
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
            KeyCode::Char('k') if self.editor.can_vim_navigate() => Direction::Up,
            KeyCode::Char('j') if self.editor.can_vim_navigate() => Direction::Down,
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
                self.editor.apply_action(Action::SelectAll);
            }
            KeyCode::Char('q') => {
                if self.persistence.is_scratch && self.editor.buffer.has_content() {
                    self.scratch_quit.open();
                    self.animations.start(
                        crate::animation::TransitionKind::ScratchQuitOverlay,
                        Duration::from_millis(150),
                        crate::animation::Easing::EaseOut,
                    );
                } else if self.persistence.is_scratch {
                    // Empty scratch: delete draft file, quit silently
                    if let Some(ref path) = self.persistence.file_path {
                        let _ = std::fs::remove_file(path);
                    }
                    self.should_quit = true;
                } else {
                    self.should_quit = true;
                }
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
                        crate::animation::TransitionKind::FindOverlay,
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
            KeyCode::Esc => {
                self.rename.cancel();
                self.pending_quit_after_rename = false;
            }
            KeyCode::Enter => {
                self.rename.confirm(&mut self.persistence.file_path, &mut self.persistence.is_scratch);
                if !self.rename.active && self.pending_quit_after_rename {
                    self.pending_quit_after_rename = false;
                    self.persistence.autosave(&self.editor.buffer, &mut self.editor.dirty);
                    self.should_quit = true;
                }
            }
            KeyCode::Backspace => self.rename.backspace(),
            KeyCode::Left => self.rename.cursor_left(),
            KeyCode::Right => self.rename.cursor_right(),
            KeyCode::Char(c) => self.rename.insert(c),
            _ => {}
        }
    }

    /// Handle key input while the scratch quit prompt is active.
    fn handle_scratch_quit_key(&mut self, code: KeyCode) {
        match self.scratch_quit.handle_key(code) {
            ScratchQuitAction::None | ScratchQuitAction::Close => {}
            ScratchQuitAction::Choose(choice) => self.apply_scratch_quit_choice(choice),
        }
    }

    /// Execute a scratch quit choice: 0=Save, 1=Rename, 2=Discard.
    fn apply_scratch_quit_choice(&mut self, choice: u8) {
        self.scratch_quit.active = false;
        match choice {
            0 => {
                // Save: autosave to the generated scratch name and quit
                self.persistence.autosave(&self.editor.buffer, &mut self.editor.dirty);
                self.should_quit = true;
            }
            1 => {
                // Rename: open rename prompt, quit after confirm
                self.rename.open(self.persistence.file_path.as_deref());
                self.pending_quit_after_rename = true;
            }
            2 => {
                // Discard: delete the draft file and quit
                if let Some(ref path) = self.persistence.file_path {
                    let _ = std::fs::remove_file(path);
                }
                self.should_quit = true;
            }
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
                    self.viewport.adjust_column_width(-1);
                    self.save_config();
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if SettingsItem::at(self.settings.cursor) == Some(SettingsItem::ColumnWidth) {
                    self.viewport.adjust_column_width(1);
                    self.save_config();
                }
            }
            _ => {} // swallow all other keys
        }
    }

    /// Run one frame of state updates: visual lines, scroll, dimming, render cache, animations.
    /// Returns None when no redraw is needed.
    pub fn tick(&mut self, surface_width: u16, surface_height: u16) -> Option<TickOutput> {
        let should_draw = self.needs_redraw || self.any_animation_active();

        if !should_draw {
            return None;
        }

        // Clamp column width to available terminal width so text wraps
        // instead of clipping when the window is narrower than column_width.
        self.viewport.effective_column_width = self.viewport.column_width.min(surface_width);
        let visual_lines = self.viewport.visual_lines(&self.editor.buffer);
        self.viewport.ensure_cursor_visible(
            self.editor.cursor_line,
            self.editor.cursor_col,
            &visual_lines,
            surface_height,
        );

        let pb = self.editor.paragraph_bounds_cached();
        self.dimming.update(
            &self.editor.buffer,
            self.editor.cursor_char_index(),
            self.editor.buffer.len_lines(),
            pb,
        );
        let sb = self.dimming.sentence_bounds();

        self.render_cache.refresh(&self.editor.buffer);
        self.animations.tick();
        self.needs_redraw = false;

        Some(TickOutput { visual_lines, sentence_bounds: sb })
    }

    /// Whether any animation subsystem is still active (palette/overlay transitions or dimming).
    pub fn any_animation_active(&self) -> bool {
        self.animations.is_active() || self.dimming.dim_animating()
    }

    // --- Public accessors for external callers (main.rs, alloc_bench.rs) ---

    pub fn should_quit(&self) -> bool { self.should_quit }
    pub fn mark_needs_redraw(&mut self) { self.needs_redraw = true; }
    pub fn cursor_shape(&self) -> CursorShape { self.editor.cursor_shape() }

    pub fn should_autosave(&self) -> bool {
        if self.external_change_pending {
            return false;
        }
        self.persistence.should_autosave(self.editor.dirty)
    }

    pub fn autosave(&mut self) {
        self.persistence.autosave(&self.editor.buffer, &mut self.editor.dirty);
    }

    /// Check if the file was modified externally. Auto-reloads clean buffers;
    /// sets conflict flag for dirty ones.
    pub fn check_external_change(&mut self) {
        if !self.persistence.mtime_changed() {
            return;
        }
        if !self.editor.dirty {
            self.reload_from_disk();
        } else {
            self.external_change_pending = true;
            self.needs_redraw = true;
        }
    }

    /// Replace the buffer with the file's current contents on disk.
    fn reload_from_disk(&mut self) {
        let Some(ref path) = self.persistence.file_path else {
            return;
        };
        match std::fs::read_to_string(path) {
            Ok(content) => {
                self.editor.reset_to_content(&content);
                self.persistence.record_mtime();
                self.needs_redraw = true;
            }
            Err(e) => {
                self.persistence.load_error = Some(e.to_string());
            }
        }
    }

    // --- Read-only accessors for draw callers (ui.rs, alloc_bench.rs) ---

    pub fn buffer(&self) -> &Buffer { &self.editor.buffer }
    pub fn palette(&self) -> Palette { self.palette }
    pub fn color_profile(&self) -> ColorProfile { self.color_profile }
    pub fn focus_mode(&self) -> FocusMode { self.dimming.focus_mode }
    pub fn column_width(&self) -> u16 { self.viewport.column_width }
    pub fn scroll_offset(&self) -> usize { self.viewport.scroll_offset }
    pub fn scroll_mode(&self) -> ScrollMode { self.viewport.scroll_mode }
    pub fn typewriter_vertical_offset(&self) -> u16 { self.viewport.typewriter_vertical_offset }
    pub fn cursor_position(&self) -> (usize, usize) { (self.editor.cursor_line, self.editor.cursor_col) }
    pub fn editing_mode(&self) -> EditingMode { self.editor.editing_mode }
    pub fn vim_mode(&self) -> Mode { self.editor.vim_mode }
    pub fn is_dirty(&self) -> bool { self.editor.dirty }
    pub fn selection_range(&self) -> Option<(usize, usize, usize, usize)> { self.editor.selection_range() }

    pub fn find_state(&self) -> Option<&FindState> { self.find_state.as_ref() }
    pub fn settings_visible(&self) -> bool { self.settings.visible }
    pub fn settings_cursor(&self) -> usize { self.settings.cursor }
    pub fn settings_overlay_progress(&self) -> Option<f64> { self.animations.settings_overlay_progress() }
    pub fn find_overlay_progress(&self) -> Option<f64> { self.animations.find_overlay_progress() }

    pub fn external_change_pending(&self) -> bool { self.external_change_pending }
    pub fn scratch_quit_active(&self) -> bool { self.scratch_quit.active }
    pub fn scratch_quit_selected(&self) -> u8 { self.scratch_quit.selected }
    pub fn scratch_quit_overlay_progress(&self) -> Option<f64> { self.animations.scratch_quit_overlay_progress() }
    pub fn file_path(&self) -> Option<&Path> { self.persistence.file_path.as_deref() }
    pub fn save_error(&self) -> Option<&str> { self.persistence.save_error.as_deref() }
    pub fn load_error(&self) -> Option<&str> { self.persistence.load_error.as_deref() }

    pub fn rename_active(&self) -> bool { self.rename.active }
    pub fn rename_buf(&self) -> &str { &self.rename.buf }
    pub fn rename_cursor(&self) -> usize { self.rename.cursor }

    pub fn sentence_fade_snapshot(&self) -> &[(usize, usize, f64)] { self.dimming.sentence_fade_snapshot() }
    pub fn paragraph_line_opacities(&self) -> &[f64] { self.dimming.paragraph_line_opacities() }

    pub fn code_block_state(&self) -> &[bool] { self.render_cache.code_block_state() }
    pub fn line_char_offsets(&self) -> &[usize] { self.render_cache.line_char_offsets() }
    pub fn md_styles(&self) -> &[Vec<CharStyle>] { self.render_cache.md_styles() }
    pub fn line_texts(&self) -> &[String] { self.render_cache.line_texts() }
    pub fn line_chars(&self) -> &[Vec<char>] { self.render_cache.line_chars() }

    // --- Setup methods for external callers (tests, benchmarks) ---

    pub fn set_buffer(&mut self, buffer: Buffer) { self.editor.buffer = buffer; }
    pub fn set_focus_mode(&mut self, mode: FocusMode) { self.dimming.focus_mode = mode; }
    pub fn set_cursor(&mut self, line: usize, col: usize) {
        self.editor.cursor_line = line;
        self.editor.cursor_col = col;
    }

    /// Returns the effective palette, accounting for any active crossfade animation.
    pub fn effective_palette(&self) -> Palette {
        if let Some((progress, from, _to)) = self.animations.palette_progress() {
            Palette::blend(from, &self.palette, progress)
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
    use crate::focus_mode::{self, FocusMode};
    use crate::scroll_mode::ScrollMode;
    use crate::settings::SettingsItem;
    use crate::vim_bindings;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Compute sentence bounds for the current cursor without caching (test helper).
    fn sentence_bounds(app: &App) -> Option<(usize, usize)> {
        focus_mode::sentence_bounds_in_buffer(&app.editor.buffer, app.editor.cursor_char_index())
    }

    /// Look up the cursor index for a given SettingsItem.
    fn item_pos(target: SettingsItem) -> usize {
        SettingsItem::all()
            .iter()
            .position(|i| *i == target)
            .unwrap_or_else(|| panic!("SettingsItem {:?} not found", target))
    }

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
        app.viewport.ensure_cursor_visible(app.editor.cursor_line, app.editor.cursor_col, &visual_lines, 10); // height 10

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
        app.viewport.ensure_cursor_visible(app.editor.cursor_line, app.editor.cursor_col, &visual_lines, 10);

        // Cursor at visual line 1, center = 5
        // Not enough content above → scroll stays 0, vertical offset pushes down
        assert_eq!(app.viewport.scroll_offset, 0, "scroll should stay at 0 when near top");
        assert_eq!(app.viewport.typewriter_vertical_offset, 4, "vertical offset should push content down to center cursor");
    }

    // === Settings Layer navigation ===

    #[test]
    fn toggle_settings_sets_cursor_to_active_palette() {
        let mut app = App::new();
        app.palette = Palette::inkwell();
        app.toggle_settings();
        assert_eq!(app.settings.cursor, item_pos(SettingsItem::Palette(1)));
    }

    #[test]
    fn settings_nav_down_wraps() {
        let mut app = App::new();
        app.settings.cursor = SettingsItem::all().len() - 1;
        app.settings.nav_down();
        assert_eq!(app.settings.cursor, item_pos(SettingsItem::EditingMode(EditingMode::Vim)), "nav down from last item should wrap to first");
    }

    #[test]
    fn settings_nav_up_wraps() {
        let mut app = App::new();
        app.settings.cursor = item_pos(SettingsItem::EditingMode(EditingMode::Vim));
        app.settings.nav_up();
        assert_eq!(app.settings.cursor, SettingsItem::all().len() - 1, "nav up from 0 should wrap to last item");
    }

    #[test]
    fn settings_nav_down_increments() {
        let mut app = App::new();
        app.settings.cursor = item_pos(SettingsItem::Palette(0));
        app.settings.nav_down();
        assert_eq!(app.settings.cursor, item_pos(SettingsItem::Palette(1)));
    }

    #[test]
    fn settings_nav_up_decrements() {
        let mut app = App::new();
        app.settings.cursor = item_pos(SettingsItem::FocusMode(FocusMode::Off));
        app.settings.nav_up();
        assert_eq!(app.settings.cursor, item_pos(SettingsItem::Palette(2)));
    }

    #[test]
    fn settings_apply_palette() {
        let mut app = App::new();
        app.settings.cursor = item_pos(SettingsItem::Palette(1)); // Inkwell
        app.settings_apply();
        assert_eq!(app.palette.name, "Inkwell");
    }

    #[test]
    fn settings_apply_focus_mode() {
        let mut app = App::new();
        app.settings.cursor = item_pos(SettingsItem::FocusMode(FocusMode::Sentence));
        app.settings_apply();
        assert_eq!(app.dimming.focus_mode, FocusMode::Sentence);

        app.settings.cursor = item_pos(SettingsItem::FocusMode(FocusMode::Paragraph));
        app.settings_apply();
        assert_eq!(app.dimming.focus_mode, FocusMode::Paragraph);
    }

    #[test]
    fn settings_apply_scroll_mode() {
        let mut app = App::new();
        app.settings.cursor = item_pos(SettingsItem::ScrollMode(ScrollMode::Typewriter));
        app.settings_apply();
        assert_eq!(app.viewport.scroll_mode, ScrollMode::Typewriter);

        app.settings.cursor = item_pos(SettingsItem::ScrollMode(ScrollMode::Edge));
        app.settings_apply();
        assert_eq!(app.viewport.scroll_mode, ScrollMode::Edge);
    }

    #[test]
    fn settings_apply_column_is_noop() {
        let mut app = App::new();
        app.settings.cursor = item_pos(SettingsItem::ColumnWidth);
        let before = app.viewport.column_width;
        app.settings_apply();
        assert_eq!(app.viewport.column_width, before, "ColumnWidth row should not change width on Enter");
    }

    #[test]
    fn palette_index_in_all_default_is_zero() {
        let app = App::new();
        assert_eq!(app.palette.index_in_all(), 0);
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
        app.settings.cursor = item_pos(SettingsItem::File);
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
        app.settings.cursor = item_pos(SettingsItem::EditingMode(EditingMode::Standard));
        app.settings_apply();
        assert_eq!(app.editor.editing_mode, EditingMode::Standard);
        assert_eq!(app.editor.vim_mode, Mode::Insert);
    }

    #[test]
    fn settings_apply_switches_to_vim_mode() {
        let mut app = App::new();
        app.editor.editing_mode = EditingMode::Standard;
        app.editor.vim_mode = Mode::Insert;
        app.settings.cursor = item_pos(SettingsItem::EditingMode(EditingMode::Vim));
        app.settings_apply();
        assert_eq!(app.editor.editing_mode, EditingMode::Vim);
        assert_eq!(app.editor.vim_mode, Mode::Normal);
    }

    #[test]
    fn switching_to_standard_clears_pending_normal_key() {
        let mut app = App::new();
        app.editor.pending_normal_key = Some('g');
        app.settings.cursor = item_pos(SettingsItem::EditingMode(EditingMode::Standard));
        app.settings_apply();
        assert_eq!(app.editor.pending_normal_key, None);
    }

    // === Full round-trip test ===

    #[test]
    fn standard_mode_full_round_trip() {
        let mut app = App::new();
        // Switch to Standard mode
        app.settings.cursor = item_pos(SettingsItem::EditingMode(EditingMode::Standard));
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
        app.editor.extend_selection(KeyCode::Right, 60);
        app.editor.extend_selection(KeyCode::Right, 60);
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
        app.settings.cursor = item_pos(SettingsItem::EditingMode(EditingMode::Vim));
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

    // === Shift+Arrow selection tests ===

    #[test]
    fn shift_down_extends_to_next_line() {
        use crossterm::event::KeyCode;
        let mut app = App::new();
        app.editor.editing_mode = EditingMode::Standard;
        app.editor.vim_mode = Mode::Insert;
        app.editor.buffer = Buffer::from_text("hello\nworld\n");
        app.editor.cursor_line = 0;
        app.editor.cursor_col = 2;
        app.editor.extend_selection(KeyCode::Down, 60);
        assert_eq!(app.editor.selection_anchor, Some((0, 2)));
        assert_eq!(app.editor.cursor_line, 1);
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
        app.settings.cursor = item_pos(SettingsItem::Palette(1)); // Inkwell
        app.settings_apply();
        assert_eq!(app.palette.name, "Inkwell");
        assert!(app.animations.is_active());
    }

    // === Overlay fade animation tests ===

    #[test]
    fn overlay_animation_starts_on_settings_open() {
        let mut app = App::new();
        app.toggle_settings();
        assert!(app.animations.settings_overlay_progress().is_some());
    }

    #[test]
    fn overlay_no_animation_on_settings_close() {
        let mut app = App::new();
        app.toggle_settings();
        // Clear the opening animation
        app.animations.transitions.clear();
        app.settings.dismiss();
        // No overlay animation started on dismiss
        assert!(app.animations.settings_overlay_progress().is_none());
    }

    // === Task 7: DimLayer wired into App ===

    #[test]
    fn dim_layers_produce_opacities() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("Line 1\n\nLine 3\nLine 4");
        app.dimming.focus_mode = FocusMode::Paragraph;
        app.editor.cursor_line = 0;
        app.dimming.update(&app.editor.buffer, app.editor.cursor_char_index(), app.editor.buffer.len_lines(), app.editor.paragraph_bounds());
        let opacities = app.dimming.paragraph_line_opacities();
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

        app.dimming.update(&app.editor.buffer, app.editor.cursor_char_index(), app.editor.buffer.len_lines(), app.editor.paragraph_bounds());

        // No fades in progress initially
        assert!(app.dimming.sentence_fades_is_empty());

        // Move cursor to line 1 (second sentence)
        app.editor.cursor_line = 1;
        app.editor.cursor_col = 0;
        app.dimming.update(&app.editor.buffer, app.editor.cursor_char_index(), app.editor.buffer.len_lines(), app.editor.paragraph_bounds());

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
        app.dimming.update(&app.editor.buffer, app.editor.cursor_char_index(), app.editor.buffer.len_lines(), app.editor.paragraph_bounds());
        let opacities = app.dimming.paragraph_line_opacities();
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
        app.dimming.update(&app.editor.buffer, app.editor.cursor_char_index(), app.editor.buffer.len_lines(), app.editor.paragraph_bounds());
        assert!(app.dimming.sentence_fades_is_empty(), "No fades initially");

        // Simulate typing space after period: "Hello world. "
        app.editor.buffer = Buffer::from_text("Hello world. ");
        app.editor.cursor_col = 12;
        app.dimming.update(&app.editor.buffer, app.editor.cursor_char_index(), app.editor.buffer.len_lines(), app.editor.paragraph_bounds());
        assert_eq!(app.dimming.sentence_fades_len(), 1, "One fade after leaving sentence");
        let original_start = app.dimming.sentence_fade_start(0);

        // Simulate typing 'T': "Hello world. T"
        app.editor.buffer = Buffer::from_text("Hello world. T");
        app.editor.cursor_col = 13;
        app.dimming.update(&app.editor.buffer, app.editor.cursor_char_index(), app.editor.buffer.len_lines(), app.editor.paragraph_bounds());

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
        app.dimming.update(&app.editor.buffer, app.editor.cursor_char_index(), app.editor.buffer.len_lines(), app.editor.paragraph_bounds());

        // Move to second sentence
        app.editor.cursor_line = 2;
        app.editor.cursor_col = 0;
        app.dimming.update(&app.editor.buffer, app.editor.cursor_char_index(), app.editor.buffer.len_lines(), app.editor.paragraph_bounds());
        assert_eq!(app.dimming.sentence_fades_len(), 1);

        // Move to third sentence
        app.editor.cursor_line = 4;
        app.editor.cursor_col = 0;
        app.dimming.update(&app.editor.buffer, app.editor.cursor_char_index(), app.editor.buffer.len_lines(), app.editor.paragraph_bounds());
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
        app.dimming.update(&app.editor.buffer, app.editor.cursor_char_index(), app.editor.buffer.len_lines(), app.editor.paragraph_bounds());

        // Move to second — first starts fading toward 0.6
        app.editor.cursor_line = 2;
        app.editor.cursor_col = 0;
        app.dimming.update(&app.editor.buffer, app.editor.cursor_char_index(), app.editor.buffer.len_lines(), app.editor.paragraph_bounds());
        assert_eq!(app.dimming.sentence_fades_len(), 1);
        assert!((app.dimming.sentence_fade_target(0) - 0.6).abs() < f64::EPSILON);

        // Return to first — should reverse that entry toward 1.0
        app.editor.cursor_line = 0;
        app.editor.cursor_col = 0;
        app.dimming.update(&app.editor.buffer, app.editor.cursor_char_index(), app.editor.buffer.len_lines(), app.editor.paragraph_bounds());
        assert!((app.dimming.sentence_fade_target(0) - 1.0).abs() < f64::EPSILON, "Should reverse to 1.0");
    }

    #[test]
    fn completed_fades_are_pruned() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("First.\n\nSecond.");
        app.dimming.focus_mode = FocusMode::Sentence;

        app.editor.cursor_line = 0;
        app.editor.cursor_col = 0;
        app.dimming.update(&app.editor.buffer, app.editor.cursor_char_index(), app.editor.buffer.len_lines(), app.editor.paragraph_bounds());

        app.editor.cursor_line = 2;
        app.editor.cursor_col = 0;
        app.dimming.update(&app.editor.buffer, app.editor.cursor_char_index(), app.editor.buffer.len_lines(), app.editor.paragraph_bounds());
        assert_eq!(app.dimming.sentence_fades_len(), 1);

        // Backdate the animation past its 1800ms duration
        app.dimming.backdate_sentence_fade(0, Duration::from_millis(2000));

        // Next update should prune the completed entry
        app.dimming.update(&app.editor.buffer, app.editor.cursor_char_index(), app.editor.buffer.len_lines(), app.editor.paragraph_bounds());
        assert!(app.dimming.sentence_fades_is_empty(), "Completed fade should be pruned");
    }

    #[test]
    fn mode_switch_clears_fade_queue() {
        let mut app = App::new();
        app.editor.buffer = Buffer::from_text("First.\n\nSecond.");
        app.dimming.focus_mode = FocusMode::Sentence;

        app.editor.cursor_line = 0;
        app.editor.cursor_col = 0;
        app.dimming.update(&app.editor.buffer, app.editor.cursor_char_index(), app.editor.buffer.len_lines(), app.editor.paragraph_bounds());

        app.editor.cursor_line = 2;
        app.editor.cursor_col = 0;
        app.dimming.update(&app.editor.buffer, app.editor.cursor_char_index(), app.editor.buffer.len_lines(), app.editor.paragraph_bounds());
        assert!(!app.dimming.sentence_fades_is_empty());

        // Switch to Off
        app.dimming.focus_mode = FocusMode::Off;
        app.dimming.update(&app.editor.buffer, app.editor.cursor_char_index(), app.editor.buffer.len_lines(), app.editor.paragraph_bounds());
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

    // === Config round-trip through App ===

    #[test]
    fn from_config_round_trip() {
        use crate::config::Config;
        let config = Config {
            palette: "Inkwell".to_string(),
            focus_mode: FocusMode::Paragraph,
            column_width: 80,
            editing_mode: EditingMode::Standard,
            scroll_mode: ScrollMode::Edge,
        };
        let app = App::from_config(&config, ColorProfile::TrueColor, None);
        assert_eq!(app.palette.name, "Inkwell");
        assert_eq!(app.dimming.focus_mode, FocusMode::Paragraph);
        assert_eq!(app.viewport.column_width, 80);
        assert_eq!(app.editor.editing_mode, EditingMode::Standard);
        assert_eq!(app.viewport.scroll_mode, ScrollMode::Edge);
    }

    #[test]
    fn save_config_round_trip() {
        use crate::config::Config;
        let original = Config {
            palette: "Parchment".to_string(),
            focus_mode: FocusMode::Sentence,
            column_width: 72,
            editing_mode: EditingMode::Standard,
            scroll_mode: ScrollMode::Typewriter,
        };
        let app = App::from_config(&original, ColorProfile::TrueColor, None);
        let recovered = Config {
            palette: app.palette().name.to_string(),
            focus_mode: app.focus_mode(),
            column_width: app.column_width(),
            editing_mode: app.editing_mode(),
            scroll_mode: app.scroll_mode(),
        };
        assert_eq!(recovered, original);
    }

    // === External change detection ===

    #[test]
    fn external_change_clean_auto_reloads() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.md");
        std::fs::write(&path, "original").unwrap();

        let mut app = App::new().with_file(path.clone(), "original");
        assert!(!app.editor.dirty);

        // External write
        std::thread::sleep(std::time::Duration::from_millis(50));
        std::fs::write(&path, "changed").unwrap();

        app.check_external_change();
        assert_eq!(app.editor.buffer.to_string(), "changed");
        assert!(!app.external_change_pending);
    }

    #[test]
    fn external_change_dirty_sets_conflict() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.md");
        std::fs::write(&path, "original").unwrap();

        let mut app = App::new().with_file(path.clone(), "original");
        app.editor.dirty = true;

        // External write
        std::thread::sleep(std::time::Duration::from_millis(50));
        std::fs::write(&path, "changed").unwrap();

        app.check_external_change();
        assert!(app.external_change_pending);
        assert_eq!(app.editor.buffer.to_string(), "original", "buffer should not change");
    }

    #[test]
    fn autosave_suppressed_during_conflict() {
        let mut app = App::new();
        app.external_change_pending = true;
        app.editor.dirty = true;
        assert!(!app.should_autosave());
    }

    #[test]
    fn conflict_reload_clears_flag() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.md");
        std::fs::write(&path, "original").unwrap();

        let mut app = App::new().with_file(path.clone(), "original");
        app.editor.dirty = true;
        app.external_change_pending = true;

        // Write new content for reload
        std::thread::sleep(std::time::Duration::from_millis(50));
        std::fs::write(&path, "reloaded").unwrap();

        app.handle_key(KeyCode::Char('r'), KeyModifiers::NONE);
        assert!(!app.external_change_pending);
        assert_eq!(app.editor.buffer.to_string(), "reloaded");
    }

    #[test]
    fn conflict_keep_resumes_autosave() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.md");
        std::fs::write(&path, "original").unwrap();

        let mut app = App::new().with_file(path, "original");
        app.editor.dirty = true;
        app.external_change_pending = true;

        app.handle_key(KeyCode::Char('k'), KeyModifiers::NONE);
        assert!(!app.external_change_pending);
        // Autosave should no longer be suppressed
        assert!(app.should_autosave());
    }

    // === Scratch quit prompt ===

    #[test]
    fn scratch_empty_quits_silently() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = App::new();
        app.persistence.file_path = Some(dir.path().join("draft.md"));
        app.persistence.is_scratch = true;
        // Buffer is empty (default) — no content worth saving

        app.handle_key(KeyCode::Char('q'), KeyModifiers::CONTROL);
        assert!(app.should_quit);
        assert!(!app.scratch_quit.active);
    }

    #[test]
    fn scratch_with_content_opens_prompt() {
        let mut app = App::new();
        app.persistence.is_scratch = true;
        app.editor.buffer = Buffer::from_text("some writing\n");

        app.handle_key(KeyCode::Char('q'), KeyModifiers::CONTROL);
        assert!(!app.should_quit);
        assert!(app.scratch_quit.active);
    }

    #[test]
    fn scratch_autosaved_with_content_still_opens_prompt() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = App::new();
        app.persistence.file_path = Some(dir.path().join("draft.md"));
        app.persistence.is_scratch = true;
        app.editor.buffer = Buffer::from_text("some writing\n");
        // Simulate autosave having cleared dirty
        app.editor.dirty = false;

        app.handle_key(KeyCode::Char('q'), KeyModifiers::CONTROL);
        assert!(!app.should_quit, "should not quit silently when buffer has content");
        assert!(app.scratch_quit.active, "should show save/discard prompt");
    }

    #[test]
    fn scratch_save_choice_quits() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = App::new();
        app.persistence.file_path = Some(dir.path().join("draft.md"));
        app.persistence.is_scratch = true;
        app.editor.dirty = true;
        app.editor.buffer = Buffer::from_text("content");

        app.apply_scratch_quit_choice(0); // Save
        assert!(app.should_quit);
    }

    #[test]
    fn scratch_discard_choice_quits() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("draft.md");
        std::fs::write(&path, "content").unwrap();

        let mut app = App::new();
        app.persistence.file_path = Some(path.clone());
        app.persistence.is_scratch = true;
        app.editor.dirty = true;

        app.apply_scratch_quit_choice(2); // Discard
        assert!(app.should_quit);
        assert!(!path.exists(), "draft file should be deleted");
    }

    #[test]
    fn non_scratch_quit_unchanged() {
        let tmp = NamedTempFile::new().unwrap();
        let mut app = App::new().with_file(tmp.path().to_path_buf(), "hello");
        assert!(!app.persistence.is_scratch);

        app.handle_key(KeyCode::Char('q'), KeyModifiers::CONTROL);
        assert!(app.should_quit);
        assert!(!app.scratch_quit.active);
    }
}
