use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Clear, Paragraph};

use crate::app::App;
use crate::settings::SettingsItem;
use crate::editing_mode::EditingMode;
use crate::focus_mode::FocusMode;
use crate::palette::Palette;
use crate::scroll_mode::ScrollMode;
use crate::vim_bindings::Mode;
use crate::wrap::VisualLine;
use crate::writing_surface::WritingSurface;

/// Render the application state to a frame.
pub fn draw(frame: &mut ratatui::Frame, app: &App, visual_lines: &[VisualLine], sentence_bounds: Option<(usize, usize)>) {
    let area = frame.area();
    if area.height < 1 {
        return; // terminal too small
    }

    // Full area for the Writing Surface — no Chrome by default (Invariant 1)
    let surface_area = area;

    // Compute the effective palette (mid-crossfade interpolation when animating).
    let effective = app.effective_palette();

    // Compute find match ranges for the writing surface
    let (find_ranges, find_current) = if let Some(ref fs) = app.find_state {
        (fs.match_ranges(), if fs.matches.is_empty() { None } else { Some(fs.current_match) })
    } else {
        (Vec::new(), None)
    };

    // Build Writing Surface
    let surface = WritingSurface::new(&app.editor.buffer, &effective)
        .column_width(app.viewport.column_width)
        .scroll_offset(app.viewport.scroll_display.round() as usize)
        .cursor(app.editor.cursor_line, app.editor.cursor_col)
        .focus_mode(app.dimming.focus_mode)
        .sentence_bounds(sentence_bounds)
        .sentence_fades(app.dimming.sentence_fade_snapshot())
        .color_profile(app.color_profile)
        .vertical_offset(app.viewport.typewriter_vertical_offset)
        .selection(app.editor.selection_range())
        .find_matches(find_ranges, find_current)
        .line_opacities(app.dimming.line_opacities())
        .precomputed_visual_lines(visual_lines)
        .code_block_state(app.render_cache.code_block_state())
        .line_char_offsets(app.render_cache.line_char_offsets())
        .md_styles(app.render_cache.md_styles());

    // Compute cursor position before render consumes the surface
    let cursor_pos = surface.cursor_visual_position(visual_lines);
    let x_offset = surface.center_offset(surface_area.width);

    // Render surface
    frame.render_widget(surface, surface_area);

    // Settings Layer overlay (Invariant 1: only visible when summoned)
    if app.settings.visible {
        draw_settings_layer(frame, app, area);
    }

    // Find overlay bar at top of screen
    if let Some(ref fs) = app.find_state {
        let find_opacity = app.animations.overlay_progress().unwrap_or(1.0);
        draw_find_bar(frame, fs, &effective, area, find_opacity);
    }

    // Position cursor
    if let Some(ref fs) = app.find_state {
        // Place cursor in the find bar
        let find_prefix_len = 6u16; // "Find: "
        let cursor_x = area.x + find_prefix_len + fs.cursor as u16;
        frame.set_cursor_position((cursor_x, area.y));
    } else if let Some((vl_idx, col)) = cursor_pos {
        let screen_row = vl_idx.saturating_sub(app.viewport.scroll_display.round() as usize);
        if screen_row < surface_area.height as usize {
            let x = surface_area.x + x_offset + col;
            let y = surface_area.y + app.viewport.typewriter_vertical_offset + screen_row as u16;
            frame.set_cursor_position((x, y));
        }
    }
}

/// Render the find bar at the top of the screen.
fn draw_find_bar(
    frame: &mut ratatui::Frame,
    fs: &crate::find::FindState,
    palette: &Palette,
    area: Rect,
    opacity: f64,
) {
    let bar_area = Rect::new(area.x, area.y, area.width, 1);
    frame.render_widget(Clear, bar_area);

    let effective_fg = crate::palette::interpolate(&palette.background, &palette.foreground, opacity);
    let effective_dim = crate::palette::interpolate(&palette.background, &palette.dimmed_foreground, opacity);

    let bar_style = Style::default()
        .fg(effective_fg)
        .bg(palette.background);

    let prefix = "Find: ";
    let match_info = if fs.query.is_empty() {
        String::new()
    } else if fs.matches.is_empty() {
        " [no matches]".to_string()
    } else {
        format!(" [{}/{}]", fs.current_match + 1, fs.matches.len())
    };

    let mut spans = vec![
        Span::styled(prefix, bar_style),
        Span::styled(fs.query.clone(), bar_style),
        Span::styled(
            match_info,
            Style::default()
                .fg(effective_dim)
                .bg(palette.background),
        ),
    ];

    // Pad the rest of the bar
    let used: usize = prefix.len() + fs.query.len() + spans[2].content.len();
    if area.width as usize > used {
        spans.push(Span::styled(
            " ".repeat(area.width as usize - used),
            bar_style,
        ));
    }

    let line = Line::from(spans);
    let paragraph = Paragraph::new(Text::from(vec![line])).style(bar_style);
    frame.render_widget(paragraph, bar_area);
}

/// A row in the settings overlay, optionally selectable.
struct SettingsRow {
    text: String,
    cursor_index: Option<usize>,
    /// Optional color swatches (bg colors for 2-char blocks) appended after text.
    swatches: Vec<ratatui::style::Color>,
    /// Whether this row is a section subheading (rendered dimmed).
    is_heading: bool,
}

/// Render the Settings Layer overlay centered on screen.
fn draw_settings_layer(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let opacity = app.animations.overlay_progress().unwrap_or(1.0);
    let overlay_width = 48u16.min(area.width);
    let all_palettes = Palette::all();
    let items = SettingsItem::all();

    // Build rows with cursor indices, inserting blank separators between groups
    let mut rows: Vec<SettingsRow> = Vec::new();
    let mut prev_group: Option<&str> = None;

    for (cursor_idx, item) in items.iter().enumerate() {
        let group = match item {
            SettingsItem::EditingMode(_) => "Editing",
            SettingsItem::Palette(_) => "Palette",
            SettingsItem::FocusMode(_) => "Focus",
            SettingsItem::ScrollMode(_) => "Scroll",
            SettingsItem::ColumnWidth => "Document",
            SettingsItem::File => "Document",
        };

        // Insert subheading when group changes
        if prev_group != Some(group) {
            // Blank line before subheading (except first group)
            if prev_group.is_some() {
                rows.push(SettingsRow { text: String::new(), cursor_index: None, swatches: vec![], is_heading: false });
            }
            rows.push(SettingsRow { text: format!("  {}", group), cursor_index: None, swatches: vec![], is_heading: true });
            prev_group = Some(group);
        }

        let text = match item {
            SettingsItem::EditingMode(mode) => {
                let label = match mode {
                    EditingMode::Vim => "Vim",
                    EditingMode::Standard => "Standard",
                };
                let marker = if *mode == app.editor.editing_mode { ">" } else { " " };
                format!("  {} {}", marker, label)
            }
            SettingsItem::Palette(idx) => {
                let palette = &all_palettes[*idx];
                let marker = if palette.name == app.palette.name { ">" } else { " " };
                // Pad name to 14 chars so swatches align across palette rows
                format!("  {} {:<14}", marker, palette.name)
            }
            SettingsItem::FocusMode(mode) => {
                let label = match mode {
                    FocusMode::Off => "Off",
                    FocusMode::Sentence => "Sentence",
                    FocusMode::Paragraph => "Paragraph",
                };
                let marker = if *mode == app.dimming.focus_mode { ">" } else { " " };
                format!("  {} {}", marker, label)
            }
            SettingsItem::ScrollMode(mode) => {
                let label = match mode {
                    ScrollMode::Edge => "Edge",
                    ScrollMode::Typewriter => "Typewriter",
                };
                let marker = if *mode == app.viewport.scroll_mode { ">" } else { " " };
                format!("  {} {}", marker, label)
            }
            SettingsItem::ColumnWidth => {
                format!("  Column      {}", app.viewport.column_width)
            }
            SettingsItem::File => {
                let file_str = app
                    .persistence.file_path
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("[scratch]");
                let prefix = "  File        ";
                let avail = (overlay_width as usize).saturating_sub(2 + prefix.len());
                if file_str.len() <= avail {
                    format!("{}{}", prefix, file_str)
                } else {
                    let skip = file_str.len() - (avail - 2);
                    format!("{}..{}", prefix, &file_str[skip..])
                }
            }
        };

        let swatches = match item {
            SettingsItem::Palette(idx) => {
                let p = &all_palettes[*idx];
                vec![p.background, p.foreground, p.accent_heading]
            }
            _ => vec![],
        };

        rows.push(SettingsRow {
            text,
            cursor_index: Some(cursor_idx),
            swatches,
            is_heading: false,
        });
    }

    // Status information (not selectable)
    let mode_str = if app.editor.editing_mode == EditingMode::Standard {
        "STANDARD"
    } else {
        match app.editor.vim_mode {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Visual => "VISUAL",
        }
    };
    let dirty_str = if app.editor.dirty { " [+]" } else { "" };
    let error_str = if let Some(ref err) = app.persistence.save_error {
        format!("  Save failed: {}", err)
    } else {
        String::new()
    };
    rows.push(SettingsRow {
        text: format!("  {}{}{}", mode_str, dirty_str, error_str),
        cursor_index: None,
        swatches: vec![],
        is_heading: false,
    });

    // Determine preview palette: if cursor is on a palette row, preview those colors
    let preview_palette = match SettingsItem::at(app.settings.cursor) {
        Some(SettingsItem::Palette(idx)) => {
            all_palettes.get(idx).cloned().unwrap_or_else(|| app.palette.clone())
        }
        _ => app.palette.clone(),
    };

    // Interpolate colors from background toward full foreground based on opacity.
    // At opacity 1.0 (animation complete or no animation) colors are unchanged.
    let effective_fg = crate::palette::interpolate(
        &preview_palette.background, &preview_palette.foreground, opacity,
    );
    let effective_dim = crate::palette::interpolate(
        &preview_palette.background, &preview_palette.dimmed_foreground, opacity,
    );
    let effective_accent = crate::palette::interpolate(
        &preview_palette.background, &preview_palette.accent_heading, opacity,
    );

    // Styles use the preview palette so colors update as the cursor moves
    let normal_style = Style::default()
        .fg(effective_fg)
        .bg(preview_palette.background);
    let cursor_style = Style::default()
        .fg(preview_palette.background)
        .bg(effective_accent);

    // Style for the rename cursor character (inverted fg/bg)
    let rename_cursor_style = Style::default()
        .fg(preview_palette.background)
        .bg(effective_fg);

    // Convert rows to styled Lines
    let lines: Vec<Line> = rows
        .iter()
        .map(|row| {
            // Rename-active file row: multi-span with visible cursor
            let is_file_row = row.cursor_index
                .and_then(SettingsItem::at)
                .is_some_and(|item| item == SettingsItem::File);
            if app.rename.active && is_file_row {
                let prefix = "  File        ";
                let buf = &app.rename.buf;
                let cursor_pos = app.rename.cursor;
                let chars: Vec<char> = buf.chars().collect();

                // Available width for filename inside overlay (border + prefix)
                let avail = (overlay_width as usize).saturating_sub(2 + prefix.len());

                // Determine visible window that keeps the cursor in view.
                // We show up to `avail` chars, biased toward the cursor position.
                let ellipsis = "..";
                let ellipsis_len = ellipsis.len();
                let (vis_start, show_ellipsis) = if chars.len() <= avail {
                    // Entire name fits
                    (0, false)
                } else {
                    // Need to truncate. Keep cursor visible with some context after it.
                    // Reserve space for ellipsis at the start.
                    let content_avail = avail.saturating_sub(ellipsis_len);
                    // Position the window so cursor is visible
                    let ideal_start = cursor_pos.saturating_sub(content_avail / 2);
                    let start = if ideal_start + content_avail > chars.len() {
                        chars.len().saturating_sub(content_avail)
                    } else {
                        ideal_start
                    };
                    if start == 0 { (0, false) } else { (start, true) }
                };

                let vis_end = (if show_ellipsis {
                    vis_start + avail.saturating_sub(ellipsis_len)
                } else {
                    vis_start + avail
                }).min(chars.len());

                // Build the visible portion, splitting around the cursor
                let vis_cursor = cursor_pos.saturating_sub(vis_start);
                let vis_chars = &chars[vis_start..vis_end];

                let before: String = vis_chars[..vis_cursor.min(vis_chars.len())].iter().collect();
                let cursor_ch = if vis_cursor < vis_chars.len() {
                    vis_chars[vis_cursor].to_string()
                } else {
                    " ".to_string()
                };
                let after_start = (vis_cursor + 1).min(vis_chars.len());
                let after: String = vis_chars[after_start..].iter().collect();

                let mut spans = vec![Span::styled(prefix.to_string(), normal_style)];
                if show_ellipsis {
                    spans.push(Span::styled(ellipsis.to_string(), normal_style));
                }
                spans.push(Span::styled(before, normal_style));
                spans.push(Span::styled(cursor_ch, rename_cursor_style));
                spans.push(Span::styled(after, normal_style));

                return Line::from(spans);
            }

            let style = if row.cursor_index == Some(app.settings.cursor) {
                cursor_style
            } else if row.is_heading {
                Style::default()
                    .fg(effective_dim)
                    .bg(preview_palette.background)
            } else {
                normal_style
            };

            if row.swatches.is_empty() {
                Line::from(Span::styled(row.text.clone(), style))
            } else {
                // Multi-span line: label + color swatches
                let mut spans = vec![Span::styled(row.text.clone(), style)];
                spans.push(Span::styled(" ", style));
                for color in &row.swatches {
                    spans.push(Span::styled(
                        "  ",
                        Style::default().bg(*color),
                    ));
                    spans.push(Span::styled(" ", style));
                }
                Line::from(spans)
            }
        })
        .collect();

    let content_rows = lines.len();
    let overlay_height = (content_rows as u16 + 2).min(area.height);
    let x = area.x + (area.width.saturating_sub(overlay_width)) / 2;
    let y = area.y + (area.height.saturating_sub(overlay_height)) / 2;
    let overlay_area = Rect::new(x, y, overlay_width, overlay_height);

    frame.render_widget(Clear, overlay_area);

    let block = Block::bordered()
        .title(" Settings ")
        .border_style(Style::default().fg(effective_dim))
        .style(Style::default().bg(preview_palette.background));

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
    fn render_app(app: &mut App, width: u16, height: u16) -> ratatui::buffer::Buffer {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        let visual_lines = app.viewport.visual_lines(&app.editor.buffer);
        let sentence_bounds = app.editor.sentence_bounds_cached();
        app.render_cache.refresh(&app.editor.buffer);
        terminal
            .draw(|frame| {
                draw(frame, &app, &visual_lines, sentence_bounds);
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
        let mut app = App::new();
        let buf = render_app(&mut app, 80, 24);
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
        let buf = render_app(&mut app, 80, 24);
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
        let buf = render_app(&mut app, 80, 24);
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
        app.editor.buffer = crate::buffer::Buffer::from_text("The quick brown fox");
        app.toggle_settings();
        let buf = render_app(&mut app, 80, 30);
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
        let buf = render_app(&mut app, 80, 24);
        let text = extract_all_text(&buf);

        // All focus mode options should be listed
        assert!(text.contains("Off"), "Should list Off focus mode");
        assert!(text.contains("Sentence"), "Should list Sentence focus mode");
        assert!(text.contains("Paragraph"), "Should list Paragraph focus mode");

        // Scroll mode options should be listed
        assert!(text.contains("Edge"), "Should list Edge scroll mode");
        assert!(text.contains("Typewriter"), "Should list Typewriter scroll mode");

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
        app.persistence.file_path = Some(std::path::PathBuf::from("/tmp/draft.md"));
        app.toggle_settings();
        let buf = render_app(&mut app, 80, 24);
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
        app.editor.dirty = true;
        app.toggle_settings();
        let buf = render_app(&mut app, 80, 24);
        let text = extract_all_text(&buf);

        assert!(
            text.contains("[+]"),
            "Settings Layer should show dirty indicator '[+]'"
        );
    }

    #[test]
    fn settings_layer_shows_save_error() {
        let mut app = App::new();
        app.persistence.save_error = Some("Permission denied".to_string());
        app.toggle_settings();
        let buf = render_app(&mut app, 80, 24);
        let text = extract_all_text(&buf);

        assert!(
            text.contains("Save failed"),
            "Settings Layer should show save error"
        );
    }

    #[test]
    fn settings_layer_shows_standard_mode_label() {
        let mut app = App::new();
        app.editor.editing_mode = crate::editing_mode::EditingMode::Standard;
        app.editor.vim_mode = crate::vim_bindings::Mode::Insert;
        app.toggle_settings();
        let buf = render_app(&mut app, 80, 24);
        let text = extract_all_text(&buf);

        assert!(
            text.contains("STANDARD"),
            "Settings Layer should show 'STANDARD' when in Standard editing mode"
        );
        assert!(
            !text.contains("INSERT"),
            "Settings Layer should not show 'INSERT' in Standard editing mode"
        );
    }

    #[test]
    fn settings_layer_shows_editing_mode_options() {
        let mut app = App::new();
        app.toggle_settings();
        let buf = render_app(&mut app, 80, 24);
        let text = extract_all_text(&buf);

        assert!(
            text.contains("Editing"),
            "Settings Layer should show 'Editing' group heading"
        );
        assert!(
            text.contains("Vim"),
            "Settings Layer should list Vim option"
        );
        assert!(
            text.contains("Standard"),
            "Settings Layer should list Standard option"
        );
        assert!(
            text.contains("> Vim"),
            "Active editing mode 'Vim' should be indicated with '>'"
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
        let buf = render_app(&mut app, 80, 24);
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
        // Clear the fade-in animation so opacity is 1.0 (fully rendered) for color assertions
        app.animations.transitions.clear();
        let buf = render_app(&mut app, 80, 24);

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
        let buf = render_app(&mut app, 80, 24);

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

    // === File row in Settings ===

    #[test]
    fn settings_layer_shows_file_row() {
        let mut app = App::new();
        app.persistence.file_path = Some(std::path::PathBuf::from("/tmp/draft.md"));
        app.toggle_settings();
        let buf = render_app(&mut app, 80, 24);
        let text = extract_all_text(&buf);

        assert!(
            text.contains("File"),
            "Settings Layer should show File label"
        );
        assert!(
            text.contains("draft.md"),
            "File row should show the filename"
        );
    }

    #[test]
    fn settings_layer_shows_scratch_name() {
        let mut app = App::new().with_scratch_name();
        app.toggle_settings();
        let buf = render_app(&mut app, 80, 24);
        let text = extract_all_text(&buf);

        assert!(
            text.contains("File"),
            "Settings Layer should show File label for scratch"
        );
        // Scratch name ends with .md
        assert!(
            text.contains(".md"),
            "File row should show the scratch filename"
        );
    }

    // === Acceptance test: Settings Layer is dismissed ===

    #[test]
    fn dismissed_settings_layer_returns_to_chromeless() {
        let mut app = App::new();
        app.persistence.file_path = Some(std::path::PathBuf::from("/tmp/draft.md"));

        // Open settings — overlay should be visible
        app.toggle_settings();
        let buf = render_app(&mut app, 80, 24);
        let text = extract_all_text(&buf);
        assert!(text.contains("NORMAL"), "Settings Layer should show vim mode");
        assert!(text.contains("Settings"), "Settings Layer title should be visible");

        // Dismiss via Escape — overlay should disappear
        app.settings.dismiss();
        let buf = render_app(&mut app, 80, 24);
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

    // === Acceptance test: Color swatches in Settings ===

    #[test]
    fn palette_rows_have_color_swatches() {
        let mut app = App::new();
        app.toggle_settings();
        let buf = render_app(&mut app, 80, 24);

        // Find the row containing "Ember" and check for swatch bg colors
        let ember = Palette::default_palette();
        let area = buf.area;
        for y in area.top()..area.bottom() {
            let mut row_text = String::new();
            for x in area.left()..area.right() {
                row_text.push_str(buf[(x, y)].symbol());
            }
            if row_text.contains("Ember") {
                // Look for cells with the palette's background color as bg
                let mut found_bg_swatch = false;
                let mut found_fg_swatch = false;
                let mut found_accent_swatch = false;
                for x in area.left()..area.right() {
                    let cell = &buf[(x, y)];
                    if cell.bg == ember.background {
                        found_bg_swatch = true;
                    }
                    if cell.bg == ember.foreground {
                        found_fg_swatch = true;
                    }
                    if cell.bg == ember.accent_heading {
                        // Cursor row also uses accent_heading as bg,
                        // but swatch cells have space as symbol
                        if cell.symbol() == " " {
                            found_accent_swatch = true;
                        }
                    }
                }
                assert!(found_bg_swatch, "Should have swatch with palette background color");
                assert!(found_fg_swatch, "Should have swatch with palette foreground color");
                assert!(found_accent_swatch, "Should have swatch with accent heading color");
                return;
            }
        }
        panic!("Could not find 'Ember' row in rendered buffer");
    }

    // === Acceptance test: Live palette preview ===

    #[test]
    fn settings_previews_hovered_palette_colors() {
        let mut app = App::new(); // default is Ember
        app.toggle_settings();

        // Move cursor to Inkwell (next palette after Ember)
        app.settings.nav_down();
        assert_eq!(app.settings.cursor, 3); // Inkwell at index 3

        let inkwell = Palette::inkwell();
        let buf = render_app(&mut app, 80, 24);

        // The overlay border/background should use Inkwell's colors, not Ember's
        // Find the Settings title row — its border should use Inkwell's dimmed_foreground
        let area = buf.area;
        for y in area.top()..area.bottom() {
            let mut row_text = String::new();
            for x in area.left()..area.right() {
                row_text.push_str(buf[(x, y)].symbol());
            }
            if row_text.contains("Settings") {
                // Check that a border character uses the Inkwell background
                // The block style sets bg to preview_palette.background
                let border_cell = &buf[(area.left() + (area.width - 48) / 2, y)];
                assert_eq!(
                    border_cell.bg, inkwell.background,
                    "Settings overlay should preview Inkwell's background color"
                );
                return;
            }
        }
        panic!("Could not find Settings title in rendered buffer");
    }

    // === Inline rename rendering ===

    #[test]
    fn rename_mode_shows_editable_text() {
        let mut app = App::new();
        app.persistence.file_path = Some(std::path::PathBuf::from("/tmp/draft.md"));
        app.toggle_settings();
        app.settings.cursor = 11; // File
        app.rename.open(app.persistence.file_path.as_deref());

        let buf = render_app(&mut app, 80, 30);
        let text = extract_all_text(&buf);

        assert!(
            text.contains("File"),
            "Rename mode should show File label"
        );
        assert!(
            text.contains("draft.md"),
            "Rename mode should show editable filename"
        );
    }

    #[test]
    fn rename_cursor_char_has_inverted_style() {
        let mut app = App::new();
        app.persistence.file_path = Some(std::path::PathBuf::from("/tmp/abc.md"));
        app.toggle_settings();
        // Clear the fade-in animation so opacity is 1.0 (fully rendered) for color assertions
        app.animations.transitions.clear();
        app.settings.cursor = 11; // File
        app.rename.open(app.persistence.file_path.as_deref());
        // Cursor at end (position 6), so cursor char is a space
        // Move cursor to start to test on 'a'
        app.rename.cursor = 0;

        let buf = render_app(&mut app, 80, 24);

        // Find the row containing "File" and the rename text
        let area = buf.area;
        for y in area.top()..area.bottom() {
            let mut row_text = String::new();
            for x in area.left()..area.right() {
                row_text.push_str(buf[(x, y)].symbol());
            }
            if row_text.contains("File") && row_text.contains("abc.md") {
                // Find the 'a' character — it should have inverted colors
                for x in area.left()..area.right() {
                    let cell = &buf[(x, y)];
                    if cell.symbol() == "a" {
                        // Cursor char: fg=background, bg=foreground
                        assert_eq!(
                            cell.fg, app.palette.background,
                            "Cursor char fg should be palette background"
                        );
                        assert_eq!(
                            cell.bg, app.palette.foreground,
                            "Cursor char bg should be palette foreground"
                        );
                        return;
                    }
                }
                panic!("Could not find 'a' character in rename row");
            }
        }
        panic!("Could not find File row with rename text");
    }
}
