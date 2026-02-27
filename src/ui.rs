use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Clear, Paragraph};

use crate::app::App;
use crate::focus_mode::FocusMode;
use crate::palette::Palette;
use crate::vim_bindings::Mode;
use crate::writing_surface::WritingSurface;

/// Render the application state to a frame.
pub fn draw(frame: &mut ratatui::Frame, app: &App) {
    let area = frame.area();
    if area.height < 1 {
        return; // terminal too small
    }

    // Full area for the Writing Surface — no Chrome by default (Invariant 1)
    let surface_area = area;

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

    // Settings Layer overlay (Invariant 1: only visible when summoned)
    if app.settings_visible {
        draw_settings_layer(frame, app, area);
    }

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

/// A row in the settings overlay, optionally selectable.
struct SettingsRow {
    text: String,
    cursor_index: Option<usize>,
}

/// Render the Settings Layer overlay centered on screen.
fn draw_settings_layer(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let all_palettes = Palette::all();
    let focus_modes = [
        (FocusMode::Off, "Off"),
        (FocusMode::Sentence, "Sentence"),
        (FocusMode::Paragraph, "Paragraph"),
        (FocusMode::Typewriter, "Typewriter"),
    ];

    // Build rows with cursor indices
    let mut rows: Vec<SettingsRow> = Vec::new();

    // Blank line before palettes
    rows.push(SettingsRow { text: String::new(), cursor_index: None });

    // Palette rows (cursor indices 0–2)
    for (i, palette) in all_palettes.iter().enumerate() {
        let marker = if palette.name == app.palette.name { ">" } else { " " };
        rows.push(SettingsRow {
            text: format!("  {} {}", marker, palette.name),
            cursor_index: Some(i),
        });
    }

    // Blank line before focus modes
    rows.push(SettingsRow { text: String::new(), cursor_index: None });

    // Focus mode rows (cursor indices 3–6)
    for (i, (mode, label)) in focus_modes.iter().enumerate() {
        let marker = if *mode == app.focus_mode { ">" } else { " " };
        rows.push(SettingsRow {
            text: format!("  {} {}", marker, label),
            cursor_index: Some(3 + i),
        });
    }

    // Blank line before column width
    rows.push(SettingsRow { text: String::new(), cursor_index: None });

    // Column width row (cursor index 7)
    rows.push(SettingsRow {
        text: format!("  Column      {}", app.column_width),
        cursor_index: Some(7),
    });

    // Status information (not selectable)
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
    rows.push(SettingsRow {
        text: format!("  {} | {}{}", mode_str, file_str, dirty_str),
        cursor_index: None,
    });

    // Styles
    let normal_style = Style::default()
        .fg(app.palette.foreground)
        .bg(app.palette.background);
    let cursor_style = Style::default()
        .fg(app.palette.background)
        .bg(app.palette.accent_heading);

    // Convert rows to styled Lines
    let lines: Vec<Line> = rows
        .iter()
        .map(|row| {
            let style = if row.cursor_index == Some(app.settings_cursor) {
                cursor_style
            } else {
                normal_style
            };
            Line::from(Span::styled(row.text.clone(), style))
        })
        .collect();

    let content_rows = lines.len();
    let overlay_height = (content_rows as u16 + 2).min(area.height);
    let overlay_width = 34u16.min(area.width);
    let x = area.x + (area.width.saturating_sub(overlay_width)) / 2;
    let y = area.y + (area.height.saturating_sub(overlay_height)) / 2;
    let overlay_area = Rect::new(x, y, overlay_width, overlay_height);

    frame.render_widget(Clear, overlay_area);

    let block = Block::bordered()
        .title(" Settings ")
        .border_style(Style::default().fg(app.palette.dimmed_foreground))
        .style(Style::default().bg(app.palette.background));

    let paragraph = Paragraph::new(Text::from(lines))
        .style(normal_style)
        .block(block);

    frame.render_widget(paragraph, overlay_area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    /// Render the app to a test buffer and return it for inspection.
    fn render_app(app: &App, width: u16, height: u16) -> ratatui::buffer::Buffer {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                draw(frame, app);
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }

    /// Extract all visible text from a rendered buffer as a single string.
    fn extract_all_text(buf: &ratatui::buffer::Buffer) -> String {
        let area = buf.area;
        let mut text = String::new();
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                text.push_str(buf[(x, y)].symbol());
            }
        }
        text
    }

    // === Acceptance test: Default state has no visible Chrome ===

    #[test]
    fn default_state_renders_no_chrome() {
        let app = App::new();
        let buf = render_app(&app, 80, 24);
        let text = extract_all_text(&buf);

        assert!(
            !text.contains("NORMAL"),
            "Mode indicator 'NORMAL' should not be visible in default state"
        );
        assert!(
            !text.contains("INSERT"),
            "Mode indicator 'INSERT' should not be visible in default state"
        );
        assert!(
            !text.contains("[scratch]"),
            "Filename should not be visible in default state"
        );
    }

    // === Acceptance test: Settings Layer is summoned by hotkey ===

    #[test]
    fn settings_layer_shows_palette_focus_mode_and_column_width() {
        let mut app = App::new();
        app.toggle_settings();
        let buf = render_app(&app, 80, 24);
        let text = extract_all_text(&buf);

        // Settings Layer overlay should show current Palette name
        assert!(
            text.contains("Ember"),
            "Settings Layer should show the active Palette name 'Ember'"
        );
        // Settings Layer overlay should show current Focus Mode
        assert!(
            text.contains("Off"),
            "Settings Layer should show the active Focus Mode 'Off'"
        );
        // Settings Layer overlay should show column width
        assert!(
            text.contains("60"),
            "Settings Layer should show the column width '60'"
        );
    }

    // === Acceptance test: Settings Layer shows Palette selection ===

    #[test]
    fn settings_layer_lists_palettes_with_active_indicated() {
        let mut app = App::new(); // default palette is Ember
        app.toggle_settings();
        let buf = render_app(&app, 80, 24);
        let text = extract_all_text(&buf);

        // All built-in palette names should be listed
        for palette in crate::palette::Palette::all() {
            assert!(
                text.contains(palette.name),
                "Settings Layer should list palette '{}'",
                palette.name
            );
        }

        // Active palette should be indicated (with > marker)
        assert!(
            text.contains("> Ember"),
            "Active palette 'Ember' should be indicated with '>'"
        );
    }

    #[test]
    fn settings_layer_does_not_replace_writing_surface() {
        let mut app = App::new();
        app.buffer = crate::buffer::Buffer::from_text("The quick brown fox");
        app.toggle_settings();
        let buf = render_app(&app, 80, 24);
        let text = extract_all_text(&buf);

        assert!(
            text.contains("quick brown fox"),
            "Writing Surface text should remain visible behind Settings Layer"
        );
    }

    // === Acceptance test: Settings Layer shows Focus Mode selection ===

    #[test]
    fn settings_layer_lists_focus_mode_options_with_active_indicated() {
        let mut app = App::new(); // default focus mode is Off
        app.toggle_settings();
        let buf = render_app(&app, 80, 24);
        let text = extract_all_text(&buf);

        // All focus mode options should be listed
        assert!(text.contains("Off"), "Should list Off focus mode");
        assert!(text.contains("Sentence"), "Should list Sentence focus mode");
        assert!(text.contains("Paragraph"), "Should list Paragraph focus mode");
        assert!(text.contains("Typewriter"), "Should list Typewriter focus mode");

        // Active focus mode should be indicated
        assert!(
            text.contains("> Off"),
            "Active focus mode 'Off' should be indicated with '>'"
        );
    }

    // === Acceptance test: Settings Layer shows status information ===

    #[test]
    fn settings_layer_shows_vim_mode_and_filename() {
        let mut app = App::new();
        app.file_path = Some(std::path::PathBuf::from("/tmp/draft.md"));
        app.toggle_settings();
        let buf = render_app(&app, 80, 24);
        let text = extract_all_text(&buf);

        assert!(
            text.contains("NORMAL"),
            "Settings Layer should show vim mode 'NORMAL'"
        );
        assert!(
            text.contains("draft.md"),
            "Settings Layer should show filename 'draft.md'"
        );
    }

    #[test]
    fn settings_layer_shows_dirty_state() {
        let mut app = App::new();
        app.dirty = true;
        app.toggle_settings();
        let buf = render_app(&app, 80, 24);
        let text = extract_all_text(&buf);

        assert!(
            text.contains("[+]"),
            "Settings Layer should show dirty indicator '[+]'"
        );
    }

    // === Acceptance test: Writer switches palette via Settings Layer ===

    #[test]
    fn palette_switch_changes_rendered_colors() {
        let mut app = App::new(); // default palette is Ember
        let ember_bg = app.palette.background;

        // Switch to Inkwell palette
        app.set_palette(crate::palette::Palette::inkwell());
        app.toggle_settings();
        let buf = render_app(&app, 80, 24);
        let text = extract_all_text(&buf);

        // Settings Layer should reflect new active palette
        assert!(
            text.contains("> Inkwell"),
            "Active palette indicator should show 'Inkwell' after switch"
        );

        // The rendered buffer should use Inkwell's background color, not Ember's
        let inkwell_bg = app.palette.background;
        assert_ne!(ember_bg, inkwell_bg, "Palettes should have different backgrounds");

        // Check that a cell in the writing surface area uses the new palette background
        let cell = &buf[(0, 0)];
        assert_eq!(
            cell.bg, inkwell_bg,
            "Writing surface should render with Inkwell's background"
        );
    }

    // === Acceptance test: Settings Layer is dismissed ===

    // === Acceptance test: Settings cursor row has distinct highlight ===

    #[test]
    fn settings_cursor_row_has_inverted_background() {
        let mut app = App::new();
        app.toggle_settings(); // cursor starts at active palette index (0 = Ember)
        let buf = render_app(&app, 80, 24);

        // Find the row containing "Ember" in the overlay
        let area = buf.area;
        for y in area.top()..area.bottom() {
            let mut row_text = String::new();
            for x in area.left()..area.right() {
                row_text.push_str(buf[(x, y)].symbol());
            }
            if row_text.contains("Ember") {
                // The cursor row should have accent_heading as background
                let cell = &buf[(area.left() + 24, y)]; // inside the overlay content
                assert_eq!(
                    cell.bg, app.palette.accent_heading,
                    "Cursor row background should be accent_heading (inverted highlight)"
                );
                return;
            }
        }
        panic!("Could not find 'Ember' row in rendered buffer");
    }

    #[test]
    fn settings_cursor_non_selected_row_has_normal_background() {
        let mut app = App::new();
        app.toggle_settings(); // cursor at 0 (Ember)
        let buf = render_app(&app, 80, 24);

        // Find the row containing "Inkwell" — should NOT be highlighted
        let area = buf.area;
        for y in area.top()..area.bottom() {
            let mut row_text = String::new();
            for x in area.left()..area.right() {
                row_text.push_str(buf[(x, y)].symbol());
            }
            if row_text.contains("Inkwell") {
                let cell = &buf[(area.left() + 24, y)];
                assert_eq!(
                    cell.bg, app.palette.background,
                    "Non-cursor row background should be palette background"
                );
                return;
            }
        }
        panic!("Could not find 'Inkwell' row in rendered buffer");
    }

    // === Acceptance test: Settings Layer is dismissed ===

    #[test]
    fn dismissed_settings_layer_returns_to_chromeless() {
        let mut app = App::new();
        app.file_path = Some(std::path::PathBuf::from("/tmp/draft.md"));

        // Open settings — overlay should be visible
        app.toggle_settings();
        let buf = render_app(&app, 80, 24);
        let text = extract_all_text(&buf);
        assert!(text.contains("NORMAL"), "Settings Layer should show vim mode");
        assert!(text.contains("Settings"), "Settings Layer title should be visible");

        // Dismiss via Escape — overlay should disappear
        app.handle_escape();
        let buf = render_app(&app, 80, 24);
        let text = extract_all_text(&buf);
        assert!(
            !text.contains("NORMAL"),
            "After dismissal, vim mode should not be visible"
        );
        assert!(
            !text.contains("draft.md"),
            "After dismissal, filename should not be visible"
        );
        assert!(
            !text.contains("Settings"),
            "After dismissal, Settings title should not be visible"
        );
    }
}
