use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Paragraph;
use ratatui::Terminal;

use zani::app::App;
use zani::focus_mode::FocusMode;
use zani::vim_bindings::{Action, CursorShape, Direction, Mode};
use zani::writing_surface::WritingSurface;
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
        let surface_height = size.height.saturating_sub(1); // reserve status line
        app.ensure_cursor_visible(surface_height);

        // Draw
        terminal.draw(|frame| {
            draw(frame, app);
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

fn draw(frame: &mut ratatui::Frame, app: &App) {
    let area = frame.area();
    if area.height < 2 {
        return; // terminal too small
    }

    let surface_area = Rect::new(area.x, area.y, area.width, area.height.saturating_sub(1));
    let status_area = Rect::new(area.x, area.bottom().saturating_sub(1), area.width, 1);

    // Build Writing Surface
    let surface = WritingSurface::new(&app.buffer, &app.palette)
        .column_width(app.column_width)
        .scroll_offset(app.scroll_offset)
        .cursor(app.cursor_line, app.cursor_col)
        .focus_mode(app.focus_mode)
        .active_line(app.cursor_line)
        .paragraph_bounds(app.paragraph_bounds());

    // Compute cursor position before render consumes the surface
    let visual_lines = surface.visual_lines();
    let cursor_pos = surface.cursor_visual_position(&visual_lines);
    let x_offset = surface.center_offset(surface_area.width);

    // Render surface
    frame.render_widget(surface, surface_area);

    // Render status line
    let mode_str = match app.vim_mode {
        Mode::Normal => "NORMAL",
        Mode::Insert => "INSERT",
        Mode::Visual => "VISUAL",
    };

    let file_str = app
        .file_path
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("[scratch]");

    let dirty_str = if app.dirty { " [+]" } else { "" };

    let focus_str = match app.focus_mode {
        FocusMode::Off => "",
        FocusMode::Sentence => " | SENTENCE",
        FocusMode::Paragraph => " | PARAGRAPH",
        FocusMode::Typewriter => " | TYPEWRITER",
    };

    let status = format!(" {} | {}{}{}", mode_str, file_str, dirty_str, focus_str);
    let status_style = Style::default()
        .fg(app.palette.dimmed_foreground)
        .bg(app.palette.background);
    let status_widget = Paragraph::new(status).style(status_style);
    frame.render_widget(status_widget, status_area);

    // Position cursor
    if let Some((vl_idx, col)) = cursor_pos {
        let screen_row = vl_idx.saturating_sub(app.scroll_offset);
        if screen_row < surface_area.height as usize {
            let x = surface_area.x + x_offset + col;
            let y = surface_area.y + screen_row as u16;
            frame.set_cursor_position((x, y));
        }
    }
}
