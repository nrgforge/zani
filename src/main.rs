use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};
use crossterm::terminal;
use ratatui::Terminal;

use zani::app::App;
use zani::color_profile::ColorProfile;
use zani::config::Config;
use zani::vim_bindings::CursorShape;
use zani::writing_window;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse CLI arguments
    let args: Vec<String> = std::env::args().collect();
    let window_flag = args.iter().any(|a| a == "--window");
    let file_path: Option<PathBuf> = args
        .iter()
        .skip(1) // skip binary name
        .filter(|a| !a.starts_with('-'))
        .next_back()
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
    let mut app = App::from_config(&config, color_profile, file_path);

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
    let mut last_cursor_shape: Option<CursorShape> = None;
    loop {
        if let Some(out) = app.tick(terminal.size()?.height) {
            terminal.draw(|frame| {
                zani::ui::draw(frame, app, &out.visual_lines, out.sentence_bounds);
            })?;

            // Set cursor shape based on vim mode (only emit when changed)
            let shape = app.cursor_shape();
            if last_cursor_shape != Some(shape) {
                last_cursor_shape = Some(shape);
                let cursor_style = match shape {
                    CursorShape::Bar => crossterm::cursor::SetCursorStyle::BlinkingBar,
                    CursorShape::Block => crossterm::cursor::SetCursorStyle::SteadyBlock,
                };
                crossterm::execute!(terminal.backend_mut(), cursor_style)?;
            }
        }

        // Poll for input: 16ms when animating (≈60fps), 250ms otherwise
        let poll_timeout = if app.any_animation_active() {
            Duration::from_millis(16)
        } else {
            Duration::from_millis(250)
        };
        if event::poll(poll_timeout)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    app.handle_key(key.code, key.modifiers);
                }
                Event::Resize(_, _) => {
                    app.mark_needs_redraw();
                }
                _ => {}
            }
        }

        // Autosave on idle
        if app.should_autosave() {
            app.autosave();
        }

        if app.should_quit() {
            app.autosave();
            break;
        }
    }

    Ok(())
}
