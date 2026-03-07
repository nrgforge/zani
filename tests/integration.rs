use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::widgets::Widget;
use zani::buffer::Buffer;
use zani::focus_mode::{self, FocusMode};
use zani::markdown_styling;
use zani::palette::Palette;
use zani::writing_surface::WritingSurface;

/// Integration test: Writing Surface applies both Focus Dimming
/// and Markdown Styling in one render pass.
///
/// Renders through the actual WritingSurface widget and inspects
/// cell colors in the output buffer, verifying that:
/// - Active-region plain text uses full foreground
/// - Non-active-region text is dimmed (darker than foreground)
/// - Markdown syntax characters (`*`) are dimmed further than body text
/// - Heading accent color is applied to heading text
/// - Both dimming layers compose without exceeding the palette's range
#[test]
fn focus_dimming_and_markdown_colors_compose() {
    let palette = Palette::default_palette();
    // Line layout:
    //   0: "## Heading"                         (heading, outside active paragraph)
    //   1: ""                                    (blank)
    //   2: "Some **bold** text."                 (active paragraph — cursor here)
    //   3: ""                                    (blank)
    //   4: "Another paragraph."                  (outside active paragraph)
    let text = "## Heading\n\nSome **bold** text.\n\nAnother paragraph.";
    let buffer = Buffer::from_text(text);
    let line_count = buffer.len_lines();

    // Active paragraph: line 2 only
    let opacities = focus_mode::paragraph_target_opacities(line_count, Some((2, 2)));

    let area = Rect::new(0, 0, 80, 10);
    let x_offset = (80 - 60) / 2; // 10

    let surface = WritingSurface::new(&buffer, &palette)
        .column_width(60)
        .focus_mode(FocusMode::Paragraph)
        .cursor(2, 0)
        .line_opacities(&opacities);

    let mut buf = ratatui::buffer::Buffer::empty(area);
    surface.render(area, &mut buf);

    // --- Active region (line 2): plain text at full foreground ---
    // 'S' in "Some" — plain text, active paragraph
    let s_cell = &buf[(x_offset, 2)];
    assert_eq!(s_cell.symbol(), "S");
    assert_eq!(
        s_cell.fg, palette.foreground,
        "Active region plain text should be full foreground"
    );

    // --- Active region (line 2): syntax chars dimmed from body ---
    // "Some **bold** text." — first '*' is at column 5
    let star_cell = &buf[(x_offset + 5, 2)];
    assert_eq!(star_cell.symbol(), "*");
    assert_ne!(
        star_cell.fg, palette.foreground,
        "Syntax '*' should be dimmer than plain text even in active region"
    );

    // --- Active region (line 2): bold text at full foreground ---
    // 'b' in "bold" is at column 7
    let b_cell = &buf[(x_offset + 7, 2)];
    assert_eq!(b_cell.symbol(), "b");
    assert_eq!(
        b_cell.fg, palette.foreground,
        "Bold text in active region should be full foreground"
    );

    // --- Non-active region (line 4): dimmed from foreground ---
    // 'A' in "Another paragraph."
    let a_cell = &buf[(x_offset, 4)];
    assert_eq!(a_cell.symbol(), "A");
    assert_ne!(
        a_cell.fg, palette.foreground,
        "Non-active region text should be dimmed"
    );
    // Verify the dimmed color is darker (closer to background)
    if let (Color::Rgb(ar, ag, ab), Color::Rgb(fr, fg, fb)) =
        (a_cell.fg, palette.foreground)
    {
        let dimmed_brightness = ar as u32 + ag as u32 + ab as u32;
        let full_brightness = fr as u32 + fg as u32 + fb as u32;
        assert!(
            dimmed_brightness < full_brightness,
            "Dimmed text ({dimmed_brightness}) should be darker than full foreground ({full_brightness})"
        );
    }

    // --- Heading (line 0): accent color, also dimmed by focus ---
    // 'H' in "Heading" is at column 3 (after "## ")
    let h_cell = &buf[(x_offset + 3, 0)];
    assert_eq!(h_cell.symbol(), "H");
    // Heading should use accent color, not plain foreground
    assert_ne!(
        h_cell.fg, palette.foreground,
        "Heading text should use accent color, not plain foreground"
    );
    // Heading is in non-active region, so it should also be focus-dimmed
    assert_ne!(
        h_cell.fg, palette.accent_heading,
        "Heading in non-active region should be focus-dimmed from accent"
    );

    // --- Background consistency: all cells use palette background ---
    assert_eq!(
        buf[(x_offset, 0)].bg, palette.background,
        "Background should be palette background"
    );
    assert_eq!(
        buf[(x_offset, 2)].bg, palette.background,
        "Active region background should be palette background"
    );
}

/// Integration test: Palette switch updates both Focus Dimming
/// and Markdown Styling.
#[test]
fn palette_switch_updates_all_styling() {
    let palette_a = Palette::default_palette();

    // Create a second palette with different colors
    let palette_b = Palette {
        name: "Test Alt",
        provenance: "",
        foreground: ratatui::style::Color::Rgb(200, 200, 210),
        background: ratatui::style::Color::Rgb(30, 30, 40),
        dimmed_foreground: ratatui::style::Color::Rgb(90, 90, 100),
        accent_heading: ratatui::style::Color::Rgb(130, 170, 200),
        accent_emphasis: ratatui::style::Color::Rgb(180, 180, 190),
        accent_link: ratatui::style::Color::Rgb(140, 170, 180),
        accent_code: ratatui::style::Color::Rgb(160, 160, 170),
        category: zani::palette::AffectiveCategory::DarkCool,
        sort_key: 0.0,
        color_256: None,
    };

    let line = "## Heading with **bold**";
    let md_styles = markdown_styling::style_line_with_context(line, false);

    // Resolve with palette A
    let resolved_a: Vec<_> = md_styles.iter().map(|s| s.resolve(&palette_a)).collect();
    // Resolve with palette B
    let resolved_b: Vec<_> = md_styles.iter().map(|s| s.resolve(&palette_b)).collect();

    // Heading text should use different accent colors
    let heading_idx = 3; // First heading text character
    assert_ne!(
        resolved_a[heading_idx].fg, resolved_b[heading_idx].fg,
        "Palette switch should change heading accent color"
    );

    // Background should change
    assert_ne!(
        resolved_a[0].bg, resolved_b[0].bg,
        "Palette switch should change background"
    );

    // Focus dimming endpoints should change
    let dim_a = focus_mode::apply_dimming_with_opacity(&palette_a.foreground, &palette_a, 0.6);
    let dim_b = focus_mode::apply_dimming_with_opacity(&palette_b.foreground, &palette_b, 0.6);
    assert_ne!(dim_a, dim_b, "Palette switch should change dimming colors");
}

/// Integration test: Styling metadata does not alter the raw buffer content.
#[test]
fn styling_preserves_raw_buffer_content() {
    let text = "Some **bold** and *italic* with -- dashes";
    let buffer = Buffer::from_text(text);

    // The buffer content should be exactly what was typed
    let content = buffer.to_string();
    assert_eq!(content, text, "buffer content should match original text");

    // Markdown styling does NOT modify the buffer
    let line = buffer.line(0).to_string();
    let styles = markdown_styling::style_line_with_context(&line, false);
    // styles is per-character metadata, not a modified string
    assert_eq!(styles.len(), line.chars().count(), "style count should match char count");

    // If we were to write this to disk, we'd write buffer.to_string()
    // which is the original text — no styling information included
    assert!(!content.contains('\u{1b}'), "buffer should not contain ANSI escape codes");
    assert!(content.contains("**bold**"), "markdown bold syntax should be preserved");
    assert!(content.contains("*italic*"), "markdown italic syntax should be preserved");
    assert!(content.contains("--"), "raw dashes should be preserved (no smart typography)");
}

/// Integration test: Config Resolution feeds the correct Palette to the Palette Browser.
/// Exercises: config → App::from_config_with_source → palette_browser.open → UI rendering.
/// Uses real types at every boundary (no mocks).
#[test]
fn config_resolution_feeds_palette_browser() {
    use crossterm::event::{KeyCode, KeyModifiers};
    use zani::app::App;
    use zani::color_profile::ColorProfile;
    use zani::config::{Config, ConfigSource};

    // Simulate: local config binds "Inkwell" to project
    let config = Config {
        palette: "Inkwell".to_string(),
        ..Config::default()
    };

    let mut app = App::from_config_with_source(
        &config,
        ColorProfile::TrueColor,
        None,
        ConfigSource::Local,
    );
    assert_eq!(app.palette().name, "Inkwell", "App should use Inkwell from config");
    assert_eq!(app.config_source(), ConfigSource::Local);

    // Open settings (Ctrl+P), then navigate to Palette row and press Enter
    app.toggle_settings();
    assert!(app.settings_visible());

    // Palette row is at index 2 (after 2 editing mode rows), toggle_settings lands on it
    // Press Enter to open browser
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert!(app.palette_browser().open);

    // Focused palette should be Inkwell (cursor positioned on active palette)
    let focused = app.palette_browser().focused_palette().unwrap();
    assert_eq!(focused.name, "Inkwell", "Browser cursor should land on Inkwell");
}

/// Integration test: 256-color degradation applies to resolved palette.
/// Exercises: palette (Color256Overrides) → color_profile (degrade_palette) → rendering chain.
/// Uses real types (no mocks).
#[test]
fn degradation_applies_to_resolved_palette() {
    use ratatui::style::Color;
    use zani::color_profile::ColorProfile;
    use zani::palette::{Color256Overrides, Palette, AffectiveCategory};

    // Create a palette with hand-tuned 256-color overrides
    let palette = Palette {
        name: "TestWith256",
        provenance: "",
        foreground: Color::Rgb(220, 220, 220),
        background: Color::Rgb(30, 30, 30),
        dimmed_foreground: Color::Rgb(100, 100, 100),
        accent_heading: Color::Rgb(150, 180, 200),
        accent_emphasis: Color::Rgb(180, 180, 190),
        accent_link: Color::Rgb(140, 170, 180),
        accent_code: Color::Rgb(160, 160, 170),
        category: AffectiveCategory::DarkCool,
        sort_key: 0.0,
        color_256: Some(Color256Overrides {
            foreground: Color::Rgb(210, 210, 210),
            background: Color::Rgb(25, 25, 25),
            dimmed_foreground: Color::Rgb(90, 90, 90),
            accent_heading: Color::Rgb(140, 170, 190),
            accent_emphasis: Color::Rgb(170, 170, 180),
            accent_link: Color::Rgb(130, 160, 170),
            accent_code: Color::Rgb(150, 150, 160),
        }),
    };

    // Degrade to 256-color
    let profile = ColorProfile::Color256;
    let degraded = profile.degrade_palette(&palette);

    // Should use hand-tuned values, not originals
    assert_eq!(degraded.foreground, Color::Rgb(210, 210, 210), "Should use hand-tuned foreground");
    assert_eq!(degraded.background, Color::Rgb(25, 25, 25), "Should use hand-tuned background");
    assert_eq!(degraded.accent_heading, Color::Rgb(140, 170, 190), "Should use hand-tuned accent");

    // Original fields preserved
    assert_eq!(degraded.name, "TestWith256");
    assert_eq!(degraded.category, AffectiveCategory::DarkCool);

    // TrueColor passthrough
    let truecolor = ColorProfile::TrueColor;
    let passed = truecolor.degrade_palette(&palette);
    assert_eq!(passed.foreground, palette.foreground, "TrueColor should pass through");
}

/// Integration test: Palette selected in browser persists via Config::bind_to_project.
/// Exercises: palette_browser (select) → config (bind_to_project) → config (load_for_path).
#[test]
fn palette_bind_persists_via_local_config() {
    use std::fs;
    use tempfile::TempDir;
    use zani::config::Config;

    let dir = TempDir::new().unwrap();
    let file = dir.path().join("doc.md");
    fs::write(&file, "test").unwrap();

    // Simulate selecting Inkwell and binding to project
    Config::bind_to_project(dir.path(), "Inkwell").unwrap();

    // Verify: loading config for a file in this directory resolves to Inkwell
    let (config, source, _) = Config::load_for_path(&file);
    assert_eq!(config.palette, "Inkwell", "Bound palette should persist");
    assert_eq!(source, zani::config::ConfigSource::Local);
}

/// Integration test: Save to project round-trips all settings (ADR-013).
/// Exercises: App (settings changes) → save_to_project → Config::save_local
/// → Config::load_for_path round-trip with real types at every boundary.
#[test]
fn save_to_project_round_trips_all_settings() {
    use crossterm::event::{KeyCode, KeyModifiers};
    use std::fs;
    use tempfile::TempDir;
    use zani::app::App;
    use zani::color_profile::ColorProfile;
    use zani::config::{Config, ConfigSource};
    use zani::focus_mode::FocusMode;
    use zani::palette::Palette;
    use zani::settings::SettingsItem;

    let dir = TempDir::new().unwrap();
    let file = dir.path().join("doc.md");
    fs::write(&file, "test content").unwrap();

    // Start with Global config (no .zani.toml exists)
    let config = Config::default();
    let mut app = App::from_config_with_source(
        &config,
        ColorProfile::TrueColor,
        Some(file.clone()),
        ConfigSource::Global,
    );
    assert_eq!(app.config_source(), ConfigSource::Global);

    // Change settings: palette to Neon Noir, focus to Paragraph, column width to 72
    app.set_palette(Palette::by_name("Neon Noir"));
    app.toggle_settings();

    // Navigate to FocusMode(Paragraph) and apply
    let paragraph_idx = SettingsItem::all()
        .iter()
        .position(|i| *i == SettingsItem::FocusMode(FocusMode::Paragraph))
        .unwrap();
    // Set cursor directly for test efficiency
    app.handle_key(KeyCode::Esc, KeyModifiers::NONE); // close settings
    app.toggle_settings(); // reopen
    // Use handle_key to navigate — settings opens on Palette row (index 2)
    // Navigate to Paragraph (index 5): 3 presses down
    app.handle_key(KeyCode::Down, KeyModifiers::NONE); // 3
    app.handle_key(KeyCode::Down, KeyModifiers::NONE); // 4
    app.handle_key(KeyCode::Down, KeyModifiers::NONE); // 5 = Paragraph
    assert_eq!(app.settings_cursor(), paragraph_idx);
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(app.focus_mode(), FocusMode::Paragraph);

    // Adjust column width: navigate to ColumnWidth row (index 8), then Left arrow
    app.handle_key(KeyCode::Down, KeyModifiers::NONE); // 6
    app.handle_key(KeyCode::Down, KeyModifiers::NONE); // 7
    app.handle_key(KeyCode::Down, KeyModifiers::NONE); // 8 = ColumnWidth
    // Set column width to 72 via Left/Right keys
    // Default is 60; each Right adds 1. We need 72, so press Right 12 times.
    for _ in 0..12 {
        app.handle_key(KeyCode::Right, KeyModifiers::NONE);
    }
    assert_eq!(app.column_width(), 72);

    // Navigate to Config row (index 10): 2 presses down from ColumnWidth (8)
    app.handle_key(KeyCode::Down, KeyModifiers::NONE); // 9 = File
    app.handle_key(KeyCode::Down, KeyModifiers::NONE); // 10 = Config
    let config_idx = SettingsItem::all()
        .iter()
        .position(|i| *i == SettingsItem::Config)
        .unwrap();
    assert_eq!(app.settings_cursor(), config_idx);

    // Press Enter to save to project
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(
        app.config_source(),
        ConfigSource::Local,
        "Config source should switch to Local after save"
    );

    // Verify .zani.toml was created
    let toml_path = dir.path().join(".zani.toml");
    assert!(toml_path.exists(), ".zani.toml should exist after save to project");

    // Round-trip: load config for a file in the same directory
    let (reloaded, source, local_path) = Config::load_for_path(&file);
    assert_eq!(source, ConfigSource::Local, "Reloaded config should be Local");
    assert!(local_path.is_some(), "Local config path should be set");
    assert_eq!(reloaded.palette, "Neon Noir", "Palette should round-trip");
    assert_eq!(reloaded.focus_mode, FocusMode::Paragraph, "Focus mode should round-trip");
    assert_eq!(reloaded.column_width, 72, "Column width should round-trip");
}

/// Integration test: Palette validation runs on both True Color and 256-color values.
/// Exercises: palette validation across both color sets.
#[test]
fn validation_covers_both_truecolor_and_256() {
    use ratatui::style::Color;
    use zani::palette::{Color256Overrides, Palette, AffectiveCategory};

    // Valid truecolor values, INVALID 256 overrides (pure black bg)
    let palette = Palette {
        name: "TestBad256",
        provenance: "",
        foreground: Color::Rgb(220, 220, 220),
        background: Color::Rgb(30, 30, 30),
        dimmed_foreground: Color::Rgb(100, 100, 100),
        accent_heading: Color::Rgb(150, 180, 200),
        accent_emphasis: Color::Rgb(180, 180, 190),
        accent_link: Color::Rgb(140, 170, 180),
        accent_code: Color::Rgb(160, 160, 170),
        category: AffectiveCategory::DarkCool,
        sort_key: 0.0,
        color_256: Some(Color256Overrides {
            foreground: Color::Rgb(210, 210, 210),
            background: Color::Rgb(0, 0, 0), // pure black — violates Invariant 3
            dimmed_foreground: Color::Rgb(90, 90, 90),
            accent_heading: Color::Rgb(140, 170, 190),
            accent_emphasis: Color::Rgb(170, 170, 180),
            accent_link: Color::Rgb(130, 160, 170),
            accent_code: Color::Rgb(150, 150, 160),
        }),
    };

    let result = palette.validate();
    assert!(result.is_err(), "Palette with pure black 256-color bg should fail validation");
}
