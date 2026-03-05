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
        foreground: ratatui::style::Color::Rgb(200, 200, 210),
        background: ratatui::style::Color::Rgb(30, 30, 40),
        dimmed_foreground: ratatui::style::Color::Rgb(90, 90, 100),
        accent_heading: ratatui::style::Color::Rgb(130, 170, 200),
        accent_emphasis: ratatui::style::Color::Rgb(180, 180, 190),
        accent_link: ratatui::style::Color::Rgb(140, 170, 180),
        accent_code: ratatui::style::Color::Rgb(160, 160, 170),
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
