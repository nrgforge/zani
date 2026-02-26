use ratatui::buffer::Buffer as RatatuiBuffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Widget;

use crate::buffer::Buffer;
use crate::palette::Palette;
use crate::wrap::{wrap_line, VisualLine};

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
}

impl<'a> WritingSurface<'a> {
    pub fn new(buffer: &'a Buffer, palette: &'a Palette) -> Self {
        Self {
            buffer,
            palette,
            column_width: 60,
            scroll_offset: 0,
            cursor: (0, 0),
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

    /// Compute all visual lines from the buffer.
    pub fn visual_lines(&self) -> Vec<VisualLine> {
        let mut all = Vec::new();
        for i in 0..self.buffer.len_lines() {
            let line_text = self.buffer.line(i).to_string();
            let wrapped = wrap_line(&line_text, self.column_width as usize, i);
            all.extend(wrapped);
        }
        all
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
    fn center_offset(&self, area_width: u16) -> u16 {
        if area_width > self.column_width {
            (area_width - self.column_width) / 2
        } else {
            0
        }
    }
}

impl Widget for WritingSurface<'_> {
    fn render(self, area: Rect, buf: &mut RatatuiBuffer) {
        let visual_lines = self.visual_lines();
        let x_offset = self.center_offset(area.width);
        let style = Style::default()
            .fg(self.palette.foreground)
            .bg(self.palette.background);

        // Fill background
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf[(x, y)].set_style(Style::default().bg(self.palette.background));
                buf[(x, y)].set_char(' ');
            }
        }

        // Render visible visual lines
        let visible_start = self.scroll_offset;
        let visible_end = (self.scroll_offset + area.height as usize).min(visual_lines.len());

        for (screen_row, vl_idx) in (visible_start..visible_end).enumerate() {
            let vl = &visual_lines[vl_idx];
            let line_text = self.buffer.line(vl.logical_line).to_string();
            let chars: Vec<char> = line_text.chars().collect();

            let y = area.top() + screen_row as u16;
            for (col, char_idx) in (vl.char_start..vl.char_end).enumerate() {
                let x = area.left() + x_offset + col as u16;
                if x < area.right() && char_idx < chars.len() {
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
        assert_eq!(cell.symbol(), "H");
        assert_eq!(cell.fg, palette.foreground);
        assert_eq!(cell.bg, palette.background);
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

        let visual_lines = surface.visual_lines();
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
        let visual_lines = surface0.visual_lines();
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
}
