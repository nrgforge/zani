use ratatui::buffer::Buffer as RatatuiBuffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Widget;

use crate::buffer::Buffer;
use crate::color_profile::ColorProfile;
use crate::focus_mode::{self, FocusMode};
use crate::markdown_styling;
use crate::palette::Palette;
use crate::wrap::{self, VisualLine};

/// The custom text viewport where prose is rendered.
/// Handles soft-wrapping, scroll positioning, and per-character styling.
/// Renders directly to ratatui's cell buffer, bypassing the Paragraph widget.
pub struct WritingSurface<'a> {
    buffer: &'a Buffer,
    palette: &'a Palette,
    /// Column width for prose wrapping (Invariant 5: ~60 chars).
    column_width: u16,
    /// Current scroll offset in visual lines.
    scroll_offset: usize,
    /// Cursor position as (logical_line, char_offset_in_line).
    cursor: (usize, usize),
    /// Current Focus Mode variant.
    focus_mode: FocusMode,
    /// Sentence bounds (start, end) as absolute char indices for sentence focus mode.
    sentence_bounds: Option<(usize, usize)>,
    /// Sentences currently fading out: (char_start, char_end, current_opacity).
    sentence_fades: &'a [(usize, usize, f64)],
    /// Terminal color capability for rendering.
    color_profile: ColorProfile,
    /// Vertical offset (rows from top) to start rendering content.
    /// Used by Typewriter mode to keep the cursor vertically centered
    /// even when there isn't enough content above to scroll.
    vertical_offset: u16,
    /// Visual mode selection range: (start_line, start_col, end_line, end_col).
    selection: Option<(usize, usize, usize, usize)>,
    /// Find match ranges: (line, start_col, end_col) for highlighting.
    find_matches: Vec<(usize, usize, usize)>,
    /// The current (active) find match index, if any.
    find_current: Option<usize>,
    /// Pre-computed opacity for each logical line (from DimLayer).
    line_opacities: &'a [f64],
    /// Pre-computed visual lines (soft-wrapped).
    precomputed_visual_lines: Option<&'a [VisualLine]>,
}

impl<'a> WritingSurface<'a> {
    pub fn new(buffer: &'a Buffer, palette: &'a Palette) -> Self {
        Self {
            buffer,
            palette,
            column_width: 60,
            scroll_offset: 0,
            cursor: (0, 0),
            focus_mode: FocusMode::Off,
            sentence_bounds: None,
            sentence_fades: &[],
            color_profile: ColorProfile::TrueColor,
            vertical_offset: 0,
            selection: None,
            find_matches: Vec::new(),
            find_current: None,
            line_opacities: &[],
            precomputed_visual_lines: None,
        }
    }

    pub fn column_width(mut self, width: u16) -> Self {
        self.column_width = width;
        self
    }

    pub fn scroll_offset(mut self, offset: usize) -> Self {
        self.scroll_offset = offset;
        self
    }

    pub fn cursor(mut self, line: usize, col: usize) -> Self {
        self.cursor = (line, col);
        self
    }

    pub fn focus_mode(mut self, mode: FocusMode) -> Self {
        self.focus_mode = mode;
        self
    }

    pub fn sentence_bounds(mut self, bounds: Option<(usize, usize)>) -> Self {
        self.sentence_bounds = bounds;
        self
    }

    pub fn sentence_fades(mut self, fades: &'a [(usize, usize, f64)]) -> Self {
        self.sentence_fades = fades;
        self
    }

    pub fn color_profile(mut self, profile: ColorProfile) -> Self {
        self.color_profile = profile;
        self
    }

    pub fn vertical_offset(mut self, offset: u16) -> Self {
        self.vertical_offset = offset;
        self
    }

    pub fn selection(mut self, sel: Option<(usize, usize, usize, usize)>) -> Self {
        self.selection = sel;
        self
    }

    pub fn find_matches(mut self, matches: Vec<(usize, usize, usize)>, current: Option<usize>) -> Self {
        self.find_matches = matches;
        self.find_current = current;
        self
    }

    pub fn line_opacities(mut self, opacities: &'a [f64]) -> Self {
        self.line_opacities = opacities;
        self
    }

    pub fn precomputed_visual_lines(mut self, lines: &'a [VisualLine]) -> Self {
        self.precomputed_visual_lines = Some(lines);
        self
    }

    /// Check whether a character at (logical_line, char_col) is within the selection.
    fn is_char_selected(&self, logical_line: usize, char_col: usize) -> bool {
        let Some((sl, sc, el, ec)) = self.selection else {
            return false;
        };
        if logical_line < sl || logical_line > el {
            return false;
        }
        if sl == el {
            // Single-line selection
            return char_col >= sc && char_col <= ec;
        }
        if logical_line == sl {
            return char_col >= sc;
        }
        if logical_line == el {
            return char_col <= ec;
        }
        // Lines strictly between start and end are fully selected
        true
    }

    /// Check if a character at (line, col) is in a find match.
    /// Returns Some(true) for current match, Some(false) for other matches, None if not a match.
    fn find_match_kind(&self, logical_line: usize, char_col: usize) -> Option<bool> {
        for (i, &(line, start, end)) in self.find_matches.iter().enumerate() {
            if logical_line == line && char_col >= start && char_col < end {
                return Some(self.find_current == Some(i));
            }
        }
        None
    }

    /// Compute all visual lines from the buffer.
    fn compute_visual_lines(&self) -> Vec<VisualLine> {
        wrap::visual_lines_for_buffer(self.buffer, self.column_width)
    }

    /// Find the visual line and column for a cursor position (logical_line, char_offset).
    /// Returns (visual_line_index, column_within_visual_line).
    pub fn cursor_visual_position(&self, visual_lines: &[VisualLine]) -> Option<(usize, u16)> {
        let (line, col) = self.cursor;
        for (i, vl) in visual_lines.iter().enumerate() {
            if vl.logical_line == line && col >= vl.char_start && col <= vl.char_end {
                return Some((i, (col - vl.char_start) as u16));
            }
        }
        // If cursor is at end of line, it belongs to the last visual line of that logical line
        for (i, vl) in visual_lines.iter().enumerate().rev() {
            if vl.logical_line == line {
                return Some((i, (vl.char_end - vl.char_start) as u16));
            }
        }
        None
    }

    /// Calculate the horizontal offset to center the column in the area.
    pub fn center_offset(&self, area_width: u16) -> u16 {
        if area_width > self.column_width {
            (area_width - self.column_width) / 2
        } else {
            0
        }
    }
}

impl Widget for WritingSurface<'_> {
    fn render(self, area: Rect, buf: &mut RatatuiBuffer) {
        let computed;
        let visual_lines: &[VisualLine] = match self.precomputed_visual_lines {
            Some(vl) => vl,
            None => {
                computed = self.compute_visual_lines();
                &computed
            }
        };
        let x_offset = self.center_offset(area.width);

        // Fill background
        let bg = self.color_profile.map_color(self.palette.background);
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf[(x, y)].set_style(Style::default().bg(bg));
                buf[(x, y)].set_char(' ');
            }
        }

        // Pre-compute per-logical-line metadata
        let mut code_block_state: Vec<bool> = Vec::with_capacity(self.buffer.len_lines());
        let mut line_char_offsets: Vec<usize> = Vec::with_capacity(self.buffer.len_lines());
        let mut in_code_block = false;
        let mut char_offset = 0;
        for i in 0..self.buffer.len_lines() {
            let line_text = self.buffer.line(i).to_string();
            line_char_offsets.push(char_offset);
            char_offset += line_text.chars().count();
            if markdown_styling::is_fence_line(&line_text) {
                code_block_state.push(false);
                in_code_block = !in_code_block;
            } else {
                code_block_state.push(in_code_block);
            }
        }

        // Sentence mode: use per-character distance
        let use_sentence_dimming = self.focus_mode == FocusMode::Sentence
            && self.sentence_bounds.is_some();

        // Render visible visual lines with per-character styling
        let visible_start = self.scroll_offset;

        // Clamp visible end to account for vertical offset
        let effective_height = (area.height as usize).saturating_sub(self.vertical_offset as usize);
        let visible_end = (self.scroll_offset + effective_height).min(visual_lines.len());

        // Reuse line data across visual lines from the same logical line
        let mut last_logical_line: Option<usize> = None;
        let mut line_text = String::new();
        let mut chars: Vec<char> = Vec::new();
        let mut md_styles: Vec<markdown_styling::CharStyle> = Vec::new();
        let mut line_opacity: f64 = 1.0;
        let mut abs_line_start: usize = 0;

        for (screen_row, vl_idx) in (visible_start..visible_end).enumerate() {
            let vl = &visual_lines[vl_idx];

            // Only recompute when the logical line changes
            if last_logical_line != Some(vl.logical_line) {
                last_logical_line = Some(vl.logical_line);

                line_text.clear();
                use std::fmt::Write;
                let _ = write!(line_text, "{}", self.buffer.line(vl.logical_line));
                chars.clear();
                chars.extend(line_text.chars());

                let line_in_code_block = code_block_state
                    .get(vl.logical_line)
                    .copied()
                    .unwrap_or(false);
                md_styles = markdown_styling::style_line_with_context(&line_text, line_in_code_block);

                line_opacity = self.line_opacities
                    .get(vl.logical_line)
                    .copied()
                    .unwrap_or(1.0);

                abs_line_start = line_char_offsets
                    .get(vl.logical_line)
                    .copied()
                    .unwrap_or(0);
            }

            let y = area.top() + self.vertical_offset + screen_row as u16;
            for (col, char_idx) in (vl.char_start..vl.char_end).enumerate() {
                let x = area.left() + x_offset + col as u16;
                if x < area.right() && char_idx < chars.len() {
                    // Per-character opacity with animated sentence transitions.
                    // Check fading sentences FIRST so fade-in animations are
                    // visible even when the chars are in the current sentence.
                    let char_opacity = if use_sentence_dimming {
                        let abs_idx = abs_line_start + char_idx;
                        let fade_hit = self.sentence_fades.iter()
                            .find(|(fs, fe, _)| abs_idx >= *fs && abs_idx < *fe)
                            .map(|(_, _, opacity)| *opacity);

                        if let Some(opacity) = fade_hit {
                            line_opacity * opacity
                        } else {
                            let (s_start, s_end) = self.sentence_bounds.unwrap();
                            let in_current = abs_idx >= s_start && abs_idx < s_end;
                            if in_current {
                                line_opacity
                            } else {
                                line_opacity * 0.6
                            }
                        }
                    } else {
                        line_opacity
                    };

                    // Resolve markdown style for this character
                    let style = if char_idx < md_styles.len() {
                        let resolved = md_styles[char_idx].resolve(self.palette);
                        if char_opacity < 1.0 {
                            match self.color_profile {
                                ColorProfile::Basic => {
                                    resolved.add_modifier(ratatui::style::Modifier::DIM)
                                }
                                _ => {
                                    let base_fg = resolved.fg.unwrap_or(self.palette.foreground);
                                    let dimmed = focus_mode::apply_dimming_with_opacity(&base_fg, self.palette, char_opacity);
                                    resolved.fg(self.color_profile.map_color(dimmed))
                                }
                            }
                        } else {
                            let fg = resolved.fg.unwrap_or(self.palette.foreground);
                            resolved.fg(self.color_profile.map_color(fg))
                        }
                    } else {
                        Style::default()
                            .fg(self.color_profile.map_color(self.palette.foreground))
                            .bg(self.palette.background)
                    };

                    // Swap fg/bg for selected characters
                    let style = if self.is_char_selected(vl.logical_line, char_idx) {
                        let fg = style.fg.unwrap_or(self.palette.foreground);
                        let bg = style.bg.unwrap_or(self.palette.background);
                        style.fg(bg).bg(fg)
                    } else {
                        style
                    };

                    // Find match highlighting
                    let style = match self.find_match_kind(vl.logical_line, char_idx) {
                        Some(true) => {
                            // Current match: inverted accent
                            style.fg(self.palette.background).bg(self.palette.accent_heading)
                        }
                        Some(false) => {
                            // Other matches: dimmed accent background
                            style.bg(self.palette.dimmed_foreground)
                        }
                        None => style,
                    };

                    buf[(x, y)].set_char(chars[char_idx]);
                    buf[(x, y)].set_style(style);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer as RatatuiBuffer;
    use ratatui::layout::Rect;
    use ratatui::style::Color;

    fn render_surface(text: &str, width: u16, area: Rect) -> RatatuiBuffer {
        let buffer = Buffer::from_text(text);
        let palette = Palette::default_palette();
        let surface = WritingSurface::new(&buffer, &palette).column_width(width);
        let mut buf = RatatuiBuffer::empty(area);
        surface.render(area, &mut buf);
        buf
    }

    fn extract_text_from_buf(buf: &RatatuiBuffer, area: Rect) -> Vec<String> {
        let mut lines = Vec::new();
        for y in area.top()..area.bottom() {
            let mut line = String::new();
            for x in area.left()..area.right() {
                line.push(buf[(x, y)].symbol().chars().next().unwrap_or(' '));
            }
            lines.push(line.trim_end().to_string());
        }
        lines
    }

    // === Acceptance test: Sentence mode per-character dimming ===

    #[test]
    fn sentence_mode_dims_outside_active_sentence() {
        let text = "First. Second.";
        let buffer = Buffer::from_text(text);
        let palette = Palette::default_palette();
        let area = Rect::new(0, 0, 80, 5);

        // Cursor in "Second" — sentence bounds should be (7, 14)
        let surface = WritingSurface::new(&buffer, &palette)
            .column_width(60)
            .focus_mode(FocusMode::Sentence)
            .line_opacities(&[1.0])
            .sentence_bounds(Some((7, 14)));

        let mut buf = RatatuiBuffer::empty(area);
        surface.render(area, &mut buf);

        let x_offset = (80 - 60) / 2; // 10

        // 'F' (char 0) is outside active sentence — should be dimmed
        let f_cell = &buf[(x_offset, 0)];
        assert_eq!(f_cell.symbol(), "F");
        assert_ne!(f_cell.fg, palette.foreground, "'F' should be dimmed (outside active sentence)");

        // 'S' (char 7) is inside active sentence — should be bright
        let s_cell = &buf[(x_offset + 7, 0)];
        assert_eq!(s_cell.symbol(), "S");
        assert_eq!(s_cell.fg, palette.foreground, "'S' should be bright (inside active sentence)");
    }

    // === Acceptance test: Text wraps at prose-width column ===

    #[test]
    fn text_wraps_at_column_width_and_is_centered() {
        let text = "The quick brown fox jumps over the lazy dog and keeps running through the forest";
        let area = Rect::new(0, 0, 80, 10);
        let buf = render_surface(text, 30, area);
        let lines = extract_text_from_buf(&buf, area);

        // Text should be centered: (80-30)/2 = 25 chars of padding
        // Find the first non-empty line
        let first_line = &lines[0];
        let content_start = first_line.find(|c: char| c != ' ');
        assert!(content_start.is_some());
        let offset = content_start.unwrap();
        assert_eq!(offset, 25, "Text should be centered with 25 char left margin");

        // Each line of content should be <= 30 chars
        for line in &lines {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                assert!(trimmed.len() <= 30, "Line too long: '{}' ({} chars)", trimmed, trimmed.len());
            }
        }
    }

    // === Acceptance test: Writing Surface renders directly to ratatui cell buffer ===

    #[test]
    fn renders_characters_to_cell_buffer_with_correct_style() {
        let text = "Hello";
        let area = Rect::new(0, 0, 80, 5);
        let buffer = Buffer::from_text(text);
        let palette = Palette::default_palette();
        let surface = WritingSurface::new(&buffer, &palette).column_width(60);
        let mut buf = RatatuiBuffer::empty(area);
        surface.render(area, &mut buf);

        // Find the 'H' character and verify its style
        let x_offset = (80 - 60) / 2; // 10
        let cell = &buf[(x_offset, 0)];
        assert_eq!(cell.symbol(), "H", "first char should be 'H'");
        assert_eq!(cell.fg, palette.foreground, "fg should match palette foreground");
        assert_eq!(cell.bg, palette.background, "bg should match palette background");
    }

    // === Acceptance test: Cursor positioning accounts for soft-wrapped lines ===

    #[test]
    fn cursor_position_on_wrapped_line() {
        let text = "The quick brown fox jumps over the lazy dog";
        let buffer = Buffer::from_text(text);
        let palette = Palette::default_palette();
        // Width 20 so "The quick brown fox" wraps
        let surface = WritingSurface::new(&buffer, &palette)
            .column_width(20)
            .cursor(0, 25); // char 25 is in "the lazy dog" part

        let visual_lines = surface.compute_visual_lines();
        let pos = surface.cursor_visual_position(&visual_lines);
        assert!(pos.is_some());
        let (vl_idx, col) = pos.unwrap();
        // Cursor should be on a visual line past the first one
        assert!(vl_idx > 0, "Cursor should be on a wrapped visual line");
        // Column should be within that visual line
        let vl = &visual_lines[vl_idx];
        assert!(col <= (vl.char_end - vl.char_start) as u16);
    }

    // === Acceptance test: Scroll position accounts for wrapped lines ===

    #[test]
    fn scroll_advances_by_visual_lines() {
        let text = "First paragraph that is long enough to wrap at twenty characters.\nSecond paragraph.";
        let area = Rect::new(0, 0, 40, 3); // Only 3 rows visible
        let buffer = Buffer::from_text(text);
        let palette = Palette::default_palette();

        // Render with scroll_offset=0
        let surface0 = WritingSurface::new(&buffer, &palette)
            .column_width(20)
            .scroll_offset(0);
        let visual_lines = surface0.compute_visual_lines();
        assert!(visual_lines.len() > 3, "Should have more visual lines than screen height");

        let mut buf0 = RatatuiBuffer::empty(area);
        surface0.render(area, &mut buf0);
        let lines0 = extract_text_from_buf(&buf0, area);

        // Render with scroll_offset=1
        let surface1 = WritingSurface::new(&buffer, &palette)
            .column_width(20)
            .scroll_offset(1);
        let mut buf1 = RatatuiBuffer::empty(area);
        surface1.render(area, &mut buf1);
        let lines1 = extract_text_from_buf(&buf1, area);

        // First visible line at offset=1 should be what was the second line at offset=0
        assert_eq!(lines1[0], lines0[1], "Scrolling by 1 should shift visual lines by 1");
    }

    // === Unit test: Background fills entire area ===

    #[test]
    fn background_fills_entire_area() {
        let area = Rect::new(0, 0, 40, 5);
        let buf = render_surface("Hi", 20, area);
        let palette = Palette::default_palette();

        // Check a cell in the margin area
        let cell = &buf[(0, 0)];
        assert_eq!(cell.bg, palette.background);
    }

    // === Unit test: Empty buffer renders without panic ===

    #[test]
    fn empty_buffer_renders() {
        let area = Rect::new(0, 0, 80, 10);
        let _buf = render_surface("", 60, area);
        // No panic = pass
    }

    // === Acceptance test: Basic profile uses DIM modifier for dimming ===

    #[test]
    fn basic_profile_uses_dim_modifier_for_focus_dimming() {
        use ratatui::style::Modifier;
        use crate::color_profile::ColorProfile;

        let text = "Hello world";
        let buffer = Buffer::from_text(text);
        let palette = Palette::default_palette();
        let area = Rect::new(0, 0, 80, 5);

        // Line opacity < 1.0 means line 0 is dimmed
        let surface = WritingSurface::new(&buffer, &palette)
            .column_width(60)
            .line_opacities(&[0.6])
            .color_profile(ColorProfile::Basic);

        let mut buf = RatatuiBuffer::empty(area);
        surface.render(area, &mut buf);

        let x_offset = (80 - 60) / 2; // 10
        let cell = &buf[(x_offset, 0)];
        assert_eq!(cell.symbol(), "H");
        assert!(
            cell.modifier.contains(Modifier::DIM),
            "Basic profile should use DIM modifier for dimmed text"
        );
    }

    // === Unit test: All color profiles render without panic ===

    #[test]
    fn all_profiles_render_without_panic() {
        use crate::color_profile::ColorProfile;

        let text = "Test text for rendering";
        let buffer = Buffer::from_text(text);
        let palette = Palette::default_palette();
        let area = Rect::new(0, 0, 80, 5);

        for profile in [ColorProfile::TrueColor, ColorProfile::Color256, ColorProfile::Basic] {
            let surface = WritingSurface::new(&buffer, &palette)
                .column_width(60)
                .line_opacities(&[0.6])
                .color_profile(profile);
            let mut buf = RatatuiBuffer::empty(area);
            surface.render(area, &mut buf);
            // No panic = pass
        }
    }

    // === Acceptance test: Different opacities produce different dimming levels ===

    #[test]
    fn opacity_based_dimming_produces_different_colors() {
        let text = "Line 0\nLine 1\nLine 2\nLine 3\nLine 4";
        let buffer = Buffer::from_text(text);
        let palette = Palette::default_palette();
        let area = Rect::new(0, 0, 80, 5);
        let x_offset = 10; // column_width 60 -> left margin = (80-60)/2 = 10

        // Render with moderate dimming (opacity 0.5)
        let moderate = WritingSurface::new(&buffer, &palette)
            .column_width(60)
            .line_opacities(&[0.5, 0.5, 0.5, 0.5, 0.5]);
        let mut buf_moderate = RatatuiBuffer::empty(area);
        moderate.render(area, &mut buf_moderate);
        let line0_moderate_fg = buf_moderate[(x_offset, 0)].fg;

        // Render with heavy dimming (opacity 0.2)
        let heavy = WritingSurface::new(&buffer, &palette)
            .column_width(60)
            .line_opacities(&[0.2, 0.2, 0.2, 0.2, 0.2]);
        let mut buf_heavy = RatatuiBuffer::empty(area);
        heavy.render(area, &mut buf_heavy);
        let line0_heavy_fg = buf_heavy[(x_offset, 0)].fg;

        // Different opacities should produce different colors
        assert_ne!(
            line0_moderate_fg, line0_heavy_fg,
            "Different opacities should produce different dimming levels"
        );

        // Verify opacity 0.5 is brighter than opacity 0.2
        if let (Color::Rgb(mr, mg, mb), Color::Rgb(hr, hg, hb)) =
            (line0_moderate_fg, line0_heavy_fg)
        {
            let moderate_brightness = mr as u32 + mg as u32 + mb as u32;
            let heavy_brightness = hr as u32 + hg as u32 + hb as u32;
            assert!(
                moderate_brightness > heavy_brightness,
                "Opacity 0.5 should be brighter than 0.2: moderate={moderate_brightness} vs heavy={heavy_brightness}"
            );
        }
    }

    // === Acceptance test: Selected text renders with swapped fg/bg ===

    #[test]
    fn selected_chars_have_swapped_fg_bg() {
        let text = "Hello world";
        let buffer = Buffer::from_text(text);
        let palette = Palette::default_palette();
        let area = Rect::new(0, 0, 80, 5);

        // Select "Hello" (chars 0-4 on line 0)
        let surface = WritingSurface::new(&buffer, &palette)
            .column_width(60)
            .selection(Some((0, 0, 0, 4)));

        let mut buf = RatatuiBuffer::empty(area);
        surface.render(area, &mut buf);

        let x_offset = (80 - 60) / 2; // 10

        // 'H' (char 0) is selected — fg/bg should be swapped
        let h_cell = &buf[(x_offset, 0)];
        assert_eq!(h_cell.symbol(), "H");
        assert_eq!(h_cell.fg, palette.background, "Selected char fg should be palette background");
        assert_eq!(h_cell.bg, palette.foreground, "Selected char bg should be palette foreground");

        // 'w' (char 6) is NOT selected — normal colors
        let w_cell = &buf[(x_offset + 6, 0)];
        assert_eq!(w_cell.symbol(), "w");
        assert_eq!(w_cell.fg, palette.foreground, "Unselected char should have normal fg");
    }

    // === Sentence fade queue regression test ===

    #[test]
    fn multiple_sentence_fades_apply_per_region_opacity() {
        // "First. Second. Third." — active sentence is "Third." (15..21)
        // Two fading regions at different opacities
        let text = "First. Second. Third.";
        let buffer = Buffer::from_text(text);
        let palette = Palette::default_palette();
        let area = Rect::new(0, 0, 80, 5);
        let x_offset = (80 - 60) / 2; // 10

        let fades = vec![(0usize, 6usize, 0.8f64), (7usize, 14usize, 0.5f64)];

        let surface = WritingSurface::new(&buffer, &palette)
            .column_width(60)
            .focus_mode(FocusMode::Sentence)
            .sentence_bounds(Some((15, 21)))
            .sentence_fades(&fades)
            .line_opacities(&[1.0]);

        let mut buf = RatatuiBuffer::empty(area);
        surface.render(area, &mut buf);

        // 'F' (char 0) — in first fade region (opacity 0.8)
        let f_cell = &buf[(x_offset, 0)];
        assert_eq!(f_cell.symbol(), "F");

        // 'S' (char 7) — in second fade region (opacity 0.5)
        let s_cell = &buf[(x_offset + 7, 0)];
        assert_eq!(s_cell.symbol(), "S");

        // 'T' (char 15) — in active sentence (full brightness)
        let t_cell = &buf[(x_offset + 15, 0)];
        assert_eq!(t_cell.symbol(), "T");
        assert_eq!(t_cell.fg, palette.foreground, "Active sentence should be full brightness");

        // Different fade opacities produce different colors
        assert_ne!(f_cell.fg, t_cell.fg, "Fading char should differ from active");
        assert_ne!(s_cell.fg, t_cell.fg, "More-faded char should differ from active");
        assert_ne!(f_cell.fg, s_cell.fg, "Different fade opacities should differ from each other");

        // Higher opacity (0.8) should be brighter than lower (0.5)
        if let (Color::Rgb(fr, fg, fb), Color::Rgb(sr, sg, sb)) = (f_cell.fg, s_cell.fg) {
            let bright_f = fr as u32 + fg as u32 + fb as u32;
            let bright_s = sr as u32 + sg as u32 + sb as u32;
            assert!(bright_f > bright_s, "Opacity 0.8 should be brighter than 0.5");
        }
    }
}
