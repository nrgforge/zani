use std::rc::Rc;

use crate::buffer::Buffer;
use crate::scroll_mode::ScrollMode;
use crate::wrap::{self, VisualLine};

/// Scroll state and visual-line computation.
pub struct Viewport {
    pub scroll_offset: usize,
    pub typewriter_vertical_offset: u16,
    pub scroll_mode: ScrollMode,
    pub column_width: u16,
    /// Cache for visual_lines: (buffer_version, column_width, result).
    visual_lines_cache: Option<(u64, u16, Rc<[VisualLine]>)>,
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new()
    }
}

impl Viewport {
    pub fn new() -> Self {
        Self {
            scroll_offset: 0,
            typewriter_vertical_offset: 0,
            scroll_mode: ScrollMode::Edge,
            column_width: 60,
            visual_lines_cache: None,
        }
    }

    /// Compute visual lines for the current buffer and column width.
    /// Returns cached result when buffer and column width haven't changed.
    /// Rc::clone is O(1) — no heap allocation on cache hit.
    pub fn visual_lines(&mut self, buffer: &Buffer) -> Rc<[VisualLine]> {
        let ver = buffer.version();
        let cw = self.column_width;
        if let Some((cv, ccw, ref cached)) = self.visual_lines_cache {
            if cv == ver && ccw == cw {
                return Rc::clone(cached);
            }
        }
        let result: Rc<[VisualLine]> = wrap::visual_lines_for_buffer(buffer, self.column_width).into();
        self.visual_lines_cache = Some((ver, cw, Rc::clone(&result)));
        result
    }

    /// Adjust scroll_offset so the cursor stays visible within the given height.
    pub fn ensure_cursor_visible(
        &mut self,
        cursor_line: usize,
        cursor_col: usize,
        visual_lines: &[VisualLine],
        visible_height: u16,
    ) {
        let height = visible_height as usize;
        if height == 0 {
            return;
        }

        // Find the cursor's visual line position
        let mut cursor_vl = 0;
        for (vl_index, vl) in visual_lines.iter().enumerate() {
            if vl.logical_line == cursor_line
                && cursor_col >= vl.char_start
                && cursor_col < vl.char_end
            {
                cursor_vl = vl_index;
                break;
            }
            // Handle cursor at end of a visual line
            if vl.logical_line == cursor_line && cursor_col == vl.char_end {
                cursor_vl = vl_index;
            }
        }

        if self.scroll_mode == ScrollMode::Typewriter {
            let center = height / 2;
            if cursor_vl >= center {
                self.scroll_offset = cursor_vl - center;
                self.typewriter_vertical_offset = 0;
            } else {
                self.scroll_offset = 0;
                self.typewriter_vertical_offset = (center - cursor_vl) as u16;
            }
        } else {
            self.typewriter_vertical_offset = 0;
            if cursor_vl < self.scroll_offset {
                self.scroll_offset = cursor_vl;
            } else if cursor_vl >= self.scroll_offset + height {
                self.scroll_offset = cursor_vl - height + 1;
            }
        }
    }
}
