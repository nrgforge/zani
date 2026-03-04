use zani::buffer::Buffer;
use zani::focus_mode;
use zani::markdown_styling;
use zani::palette::Palette;

/// Integration test: Writing Surface applies both Focus Dimming
/// and Markdown Styling in one render pass.
///
/// Verifies that text in the Active Region has markdown formatting
/// at full foreground color, text outside has Focus-dimmed foreground,
/// and syntax characters are further dimmed. Both dimming layers
/// compose correctly.
#[test]
fn focus_dimming_and_markdown_colors_compose() {
    let palette = Palette::default_palette();
    let text = "## Heading\n\nSome **bold** text in the body paragraph.\n\nAnother paragraph here.";
    let buffer = Buffer::from_text(text);
    let line_count = buffer.len_lines();

    // Active paragraph is just line 2
    let opacities = focus_mode::paragraph_target_opacities(line_count, Some((2, 2)));

    // For each line, compute both layers
    for line_idx in 0..line_count {
        let line_text = buffer.line(line_idx).to_string();
        let md_styles = markdown_styling::style_line_with_context(&line_text, false);
        let opacity = opacities[line_idx];
        let focus_color = focus_mode::apply_dimming_with_opacity(&palette.foreground, &palette, opacity);

        for ms in md_styles.iter() {
            let md_resolved = ms.resolve(&palette);

            // If syntax: should be dimmed from markdown styling
            if ms.is_syntax {
                assert_eq!(
                    md_resolved.fg.unwrap(),
                    palette.dimmed_foreground,
                    "Syntax char should use dimmed_foreground"
                );
            }

            // If in active region (opacity 1.0): markdown styling at full color
            if opacity >= 1.0 && !ms.is_syntax && !ms.is_heading && !ms.is_code {
                assert_eq!(
                    md_resolved.fg.unwrap(),
                    palette.foreground,
                    "Active region plain text should be full foreground"
                );
            }

            // Focus dimming color should differ from foreground for non-active regions
            if opacity < 1.0 {
                assert_ne!(
                    focus_color, palette.foreground,
                    "Non-active region should have dimmed focus color"
                );
            }

            // Composed color: for a non-syntax, non-active character,
            // we'd apply both markdown resolved color AND focus dimming.
            // The composed result should not exceed the palette's color range.
            if opacity < 1.0 && !ms.is_syntax {
                // The final color would be the focus-dimmed version of the
                // markdown resolved foreground. Both are interpolations toward
                // background, so the composed result stays within range.
                let base_fg = md_resolved.fg.unwrap();
                let composed = focus_mode::apply_dimming_with_opacity(&base_fg, &palette, opacity);
                // Verify composition produces an RGB color
                assert!(matches!(composed, ratatui::style::Color::Rgb(_, _, _)));
            }
        }
    }
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
