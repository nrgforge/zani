use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal;
use ratatui::Terminal;

use zani::app::App;
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

    // Create application state
    let mut app = App::new();
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
        app.ensure_cursor_visible(surface_height);

        // Draw
        terminal.draw(|frame| {
            zani::ui::draw(frame, app);
        })?;

        // Set cursor shape based on vim mode
        let cursor_style = match app.cursor_shape() {
            CursorShape::Bar => crossterm::cursor::SetCursorStyle::BlinkingBar,
            CursorShape::Block => crossterm::cursor::SetCursorStyle::SteadyBlock,
        };
        crossterm::execute!(terminal.backend_mut(), cursor_style)?;

        // Poll for input (250ms timeout enables autosave checks)
        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    handle_key(app, key.code, key.modifiers);
                }
                _ => {}
            }
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

fn handle_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    // Ctrl combinations — checked first, independent of vim mode
    if modifiers.contains(KeyModifiers::CONTROL) {
        match code {
            KeyCode::Char('f') => {
                app.focus_mode = app.focus_mode.next();
            }
            KeyCode::Char('p') => {
                app.toggle_settings();
            }
            KeyCode::Char('s') => {
                app.autosave();
            }
            KeyCode::Char('c') | KeyCode::Char('q') => {
                app.should_quit = true;
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
            KeyCode::Enter => app.settings_apply(),
            KeyCode::Left | KeyCode::Char('h') => {
                if app.settings_cursor == 7 {
                    app.settings_adjust_column(-1);
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if app.settings_cursor == 7 {
                    app.settings_adjust_column(1);
                }
            }
            _ => {} // swallow all other keys
        }
        return;
    }

    match code {
        KeyCode::Esc => app.handle_escape(),
        KeyCode::Char(c) => app.handle_char(c),
        KeyCode::Backspace => {
            if app.vim_mode == Mode::Insert {
                app.apply_action(Action::DeleteBack);
            }
        }
        KeyCode::Enter => {
            if app.vim_mode == Mode::Insert {
                app.apply_action(Action::InsertNewline);
            }
        }
        KeyCode::Left => app.apply_action(Action::MoveCursor(Direction::Left)),
        KeyCode::Right => app.apply_action(Action::MoveCursor(Direction::Right)),
        KeyCode::Up => app.apply_action(Action::MoveCursor(Direction::Up)),
        KeyCode::Down => app.apply_action(Action::MoveCursor(Direction::Down)),
        _ => {}
    }
}
