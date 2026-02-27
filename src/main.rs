use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal;
use ratatui::Terminal;

use zani::app::{App, SettingsItem};
use zani::color_profile::ColorProfile;
use zani::config::Config;
use zani::vim_bindings::{Action, CursorShape, Direction, Mode};
use zani::writing_window;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse CLI arguments
    let args: Vec<String> = std::env::args().collect();
    let window_flag = args.iter().any(|a| a == "--window");
    let file_path: Option<PathBuf> = args
        .iter()
        .skip(1) // skip binary name
        .filter(|a| !a.starts_with('-'))
        .last()
        .map(PathBuf::from);

    // Writing Window: only spawn a dedicated window when explicitly requested
    if window_flag {
        let env_fn = |key: &str| -> Option<String> { std::env::var(key).ok() };
        let detected = writing_window::detect_terminal(&env_fn);
        let config = writing_window::WindowConfig::default();

        let binary = std::env::current_exe()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "zani".to_string());

        let file_str: Option<String> = file_path.as_ref().map(|p| p.display().to_string());
        let zani_args: Vec<&str> = file_str.iter().map(|s| s.as_str()).collect();

        if let Some(cmd) =
            writing_window::spawn_command(&detected, &config, &binary, &zani_args)
        {
            match std::process::Command::new(&cmd[0])
                .args(&cmd[1..])
                .env("ZANI_WINDOW", "1")
                .spawn()
            {
                Ok(_) => std::process::exit(0),
                Err(_) => {
                    eprintln!("Failed to open Writing Window, running inline.");
                }
            }
        } else {
            eprintln!("Unknown terminal, running inline.");
        }
    }

    // Detect terminal color capability
    let color_profile = ColorProfile::detect();

    // Load persisted config
    let config = Config::load();

    // Create application state
    let mut app = App::new();
    app.color_profile = color_profile;
    app.set_palette(config.resolve_palette());
    app.focus_mode = config.focus_mode;
    app.column_width = config.column_width;
    app.editing_mode = config.editing_mode;
    if app.editing_mode == zani::editing_mode::EditingMode::Standard {
        app.vim_mode = zani::vim_bindings::Mode::Insert;
    }
    if let Some(ref path) = file_path {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        app = app.with_file(path.clone(), &content);
    } else {
        app = app.with_scratch_name();
    }

    // Initialize terminal
    terminal::enable_raw_mode()?;
    crossterm::execute!(
        io::stdout(),
        terminal::EnterAlternateScreen,
        event::EnableMouseCapture
    )?;
    let backend = ratatui::backend::CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    // Run event loop (separated so cleanup always runs)
    let result = run(&mut terminal, &mut app);

    // Restore terminal
    terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        terminal::LeaveAlternateScreen,
        event::DisableMouseCapture,
        crossterm::cursor::SetCursorStyle::DefaultUserShape
    )?;
    terminal.show_cursor()?;

    result
}

fn run(
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        // Adjust scroll to keep cursor visible
        let size = terminal.size()?;
        let surface_height = size.height; // full height — no Chrome by default
        let visual_lines = app.visual_lines();
        app.ensure_cursor_visible(&visual_lines, surface_height);

        // Draw
        terminal.draw(|frame| {
            zani::ui::draw(frame, app);
        })?;
        app.animations.tick();

        // Update smooth scroll display value
        if let Some(progress) = app.animations.scroll_progress() {
            if let Some((from, to)) = app.animations.scroll_values() {
                app.scroll_display = from + (to - from) * progress;
            }
        } else {
            app.scroll_display = app.scroll_offset as f64;
        }

        // Set cursor shape based on vim mode
        let cursor_style = match app.cursor_shape() {
            CursorShape::Bar => crossterm::cursor::SetCursorStyle::BlinkingBar,
            CursorShape::Block => crossterm::cursor::SetCursorStyle::SteadyBlock,
        };
        crossterm::execute!(terminal.backend_mut(), cursor_style)?;

        // Poll for input: 16ms when animating (≈60fps), 250ms otherwise
        let line_before_input = app.cursor_line;
        let poll_timeout = if app.animations.is_active() {
            Duration::from_millis(16)
        } else {
            Duration::from_millis(250)
        };
        if event::poll(poll_timeout)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    handle_key(app, key.code, key.modifiers);
                }
                _ => {}
            }
        }

        if app.cursor_line != line_before_input
            && app.focus_mode != zani::focus_mode::FocusMode::Off
        {
            app.animations.start(
                zani::animation::TransitionKind::FocusDimming {
                    from_line: line_before_input,
                    to_line: app.cursor_line,
                },
                Duration::from_millis(150),
                zani::animation::Easing::EaseOut,
            );
        }

        // Autosave on idle
        if app.should_autosave() {
            app.autosave();
        }

        if app.should_quit {
            app.autosave();
            break;
        }
    }

    Ok(())
}

/// Persist current settings to config file (best-effort, errors silently ignored).
fn save_config(app: &App) {
    let config = Config {
        palette: app.palette.name.to_string(),
        focus_mode: app.focus_mode,
        column_width: app.column_width,
        editing_mode: app.editing_mode,
    };
    let _ = config.save();
}

fn handle_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    // Ctrl combinations — checked first, independent of vim mode
    if modifiers.contains(KeyModifiers::CONTROL) {
        match code {
            KeyCode::Char('c') => {
                // Copy selection if any, otherwise no-op
                if let Some(text) = app.selected_text() {
                    zani::clipboard::write_osc52(&text);
                    app.yank_register = Some(text);
                    app.selection_anchor = None;
                    if app.vim_mode == Mode::Visual {
                        app.vim_mode = Mode::Normal;
                    }
                }
            }
            KeyCode::Char('x') => {
                // Cut: copy selection then delete it
                if let Some(text) = app.selected_text() {
                    zani::clipboard::write_osc52(&text);
                    app.yank_register = Some(text);
                    app.delete_selection_silent();
                    app.selection_anchor = None;
                    if app.vim_mode == Mode::Visual {
                        app.vim_mode = Mode::Normal;
                    }
                }
            }
            KeyCode::Char('v') => {
                // Paste from yank register at cursor
                if let Some(text) = app.yank_register.clone() {
                    // In Standard mode with selection, replace selection first
                    if app.editing_mode == zani::editing_mode::EditingMode::Standard
                        && app.selection_anchor.is_some()
                    {
                        app.delete_selection_silent();
                        app.selection_anchor = None;
                    }
                    let idx = app.cursor_char_index();
                    app.buffer.insert(idx, &text);
                    // Advance cursor past inserted text
                    let char_count = text.chars().count();
                    app.set_cursor_from_char_index(idx + char_count);
                    app.dirty = true;
                }
            }
            KeyCode::Char('a') => {
                // Select all
                let total_chars = app.buffer.rope().len_chars();
                app.selection_anchor = Some((0, 0));
                if total_chars > 0 {
                    app.set_cursor_from_char_index(total_chars.saturating_sub(1));
                }
                if app.editing_mode == zani::editing_mode::EditingMode::Vim {
                    app.vim_mode = Mode::Visual;
                }
            }
            KeyCode::Char('q') => {
                app.should_quit = true;
            }
            KeyCode::Char('p') => {
                app.toggle_settings();
            }
            KeyCode::Char('s') => {
                app.autosave();
            }
            KeyCode::Char('f') => {
                if app.find_state.is_none() {
                    app.find_state = Some(zani::find::FindState::new(
                        app.cursor_line,
                        app.cursor_col,
                    ));
                }
            }
            KeyCode::Char('z') => {
                app.apply_action(Action::Undo);
            }
            KeyCode::Char('y') => {
                app.apply_action(Action::Redo);
            }
            _ => {}
        }
        return;
    }

    // Find overlay — swallow all keys when active
    if let Some(ref mut find) = app.find_state {
        match code {
            KeyCode::Esc => {
                // Cancel: restore cursor to pre-search position
                let (line, col) = find.saved_cursor;
                app.cursor_line = line;
                app.cursor_col = col;
                app.find_state = None;
            }
            KeyCode::Enter => {
                // Jump to current match and close find
                if let Some((line, col)) = find.current_match_pos() {
                    app.cursor_line = line;
                    app.cursor_col = col;
                }
                app.find_state = None;
            }
            KeyCode::Backspace => {
                find.backspace();
                find.search(&app.buffer);
                // Jump cursor to first match for live preview
                if let Some((line, col)) = find.current_match_pos() {
                    app.cursor_line = line;
                    app.cursor_col = col;
                }
            }
            KeyCode::Up => {
                find.prev_match();
                if let Some((line, col)) = find.current_match_pos() {
                    app.cursor_line = line;
                    app.cursor_col = col;
                }
            }
            KeyCode::Down => {
                find.next_match();
                if let Some((line, col)) = find.current_match_pos() {
                    app.cursor_line = line;
                    app.cursor_col = col;
                }
            }
            KeyCode::Char(c) => {
                find.insert_char(c);
                find.search(&app.buffer);
                // Jump cursor to first match for live preview
                if let Some((line, col)) = find.current_match_pos() {
                    app.cursor_line = line;
                    app.cursor_col = col;
                }
            }
            _ => {}
        }
        return;
    }

    // Inline rename — swallow all keys when active
    if app.rename_active {
        match code {
            KeyCode::Esc => app.rename_cancel(),
            KeyCode::Enter => app.rename_confirm(),
            KeyCode::Backspace => app.rename_backspace(),
            KeyCode::Left => app.rename_cursor_left(),
            KeyCode::Right => app.rename_cursor_right(),
            KeyCode::Char(c) => app.rename_insert(c),
            _ => {}
        }
        return;
    }

    // Settings Layer navigation — swallow all keys when open
    if app.settings_visible {
        match code {
            KeyCode::Esc => app.dismiss_settings(),
            KeyCode::Up | KeyCode::Char('k') => app.settings_nav_up(),
            KeyCode::Down | KeyCode::Char('j') => app.settings_nav_down(),
            KeyCode::Enter => {
                app.settings_apply();
                save_config(app);
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if SettingsItem::at(app.settings_cursor) == Some(SettingsItem::ColumnWidth) {
                    app.settings_adjust_column(-1);
                    save_config(app);
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if SettingsItem::at(app.settings_cursor) == Some(SettingsItem::ColumnWidth) {
                    app.settings_adjust_column(1);
                    save_config(app);
                }
            }
            _ => {} // swallow all other keys
        }
        return;
    }

    let is_standard = app.editing_mode == zani::editing_mode::EditingMode::Standard;

    // Shift+Arrow/Home/End extends selection
    if modifiers.contains(KeyModifiers::SHIFT) {
        match code {
            KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down
            | KeyCode::Home | KeyCode::End => {
                app.extend_selection(code);
                return;
            }
            _ => {}
        }
    }

    match code {
        KeyCode::Esc => app.handle_escape(),
        KeyCode::Char(c) => app.handle_char(c),
        KeyCode::Backspace => {
            if is_standard || app.vim_mode == Mode::Insert {
                // In Standard mode with selection, delete selection instead
                if is_standard && app.selection_anchor.is_some() {
                    app.delete_selection_silent();
                    app.selection_anchor = None;
                } else {
                    app.apply_action(Action::DeleteBack);
                }
            }
        }
        KeyCode::Enter => {
            if is_standard || app.vim_mode == Mode::Insert {
                if is_standard && app.selection_anchor.is_some() {
                    app.delete_selection_silent();
                    app.selection_anchor = None;
                }
                app.apply_action(Action::InsertNewline);
            }
        }
        KeyCode::Delete => {
            app.apply_action(Action::DeleteChar);
        }
        KeyCode::Home => {
            if is_standard {
                app.selection_anchor = None;
            }
            app.apply_action(Action::LineStart);
        }
        KeyCode::End => {
            if is_standard {
                app.selection_anchor = None;
            }
            app.apply_action(Action::LineEnd);
        }
        KeyCode::Left => {
            if is_standard {
                app.selection_anchor = None;
            }
            app.apply_action(Action::MoveCursor(Direction::Left));
        }
        KeyCode::Right => {
            if is_standard {
                app.selection_anchor = None;
            }
            app.apply_action(Action::MoveCursor(Direction::Right));
        }
        KeyCode::Up => {
            if is_standard {
                app.selection_anchor = None;
            }
            app.apply_action(Action::MoveCursor(Direction::Up));
        }
        KeyCode::Down => {
            if is_standard {
                app.selection_anchor = None;
            }
            app.apply_action(Action::MoveCursor(Direction::Down));
        }
        _ => {}
    }
}
