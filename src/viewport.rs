use crate::animation::AnimationManager;
use crate::buffer::Buffer;
use crate::scroll_mode::ScrollMode;
use crate::wrap::{self, VisualLine};

/// Scroll state and visual-line computation.
pub struct Viewport {
    pub scroll_offset: usize,
    pub scroll_display: f64,
    pub typewriter_vertical_offset: u16,
    pub scroll_mode: ScrollMode,
    pub column_width: u16,
    /// Cache for visual_lines: (buffer_version, column_width, result).
    visual_lines_cache: Option<(u64, u16, Vec<VisualLine>)>,
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
            scroll_display: 0.0,
            typewriter_vertical_offset: 0,
            scroll_mode: ScrollMode::Edge,
            column_width: 60,
            visual_lines_cache: None,
        }
    }

    /// Compute visual lines for the current buffer and column width.
    /// Returns cached result when buffer and column width haven't changed.
    pub fn visual_lines(&mut self, buffer: &Buffer) -> Vec<VisualLine> {
        let ver = buffer.version();
        let cw = self.column_width;
        if let Some((cv, ccw, ref cached)) = self.visual_lines_cache
            && cv == ver && ccw == cw
        {
            return cached.clone();
        }
        let result = wrap::visual_lines_for_buffer(buffer, self.column_width);
        self.visual_lines_cache = Some((ver, cw, result.clone()));
        result
    }

    /// Adjust scroll_offset so the cursor stays visible within the given height.
    pub fn ensure_cursor_visible(
        &mut self,
        cursor_line: usize,
        cursor_col: usize,
        visual_lines: &[VisualLine],
        visible_height: u16,
        animations: &mut AnimationManager,
    ) {
        let height = visible_height as usize;
        if height == 0 {
            return;
        }

        let old_offset = self.scroll_offset;

        // Find the cursor's visual line position
        let mut cursor_vl = 0;
        let mut found = false;
        for (vl_index, vl) in visual_lines.iter().enumerate() {
            if vl.logical_line == cursor_line
                && cursor_col >= vl.char_start
                && cursor_col < vl.char_end
            {
                cursor_vl = vl_index;
                found = true;
                break;
            }
            // Handle cursor at end of a visual line
            if vl.logical_line == cursor_line && cursor_col == vl.char_end {
                cursor_vl = vl_index;
            }
        }

        let _ = found;

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

        if self.scroll_offset != old_offset {
            let from = self.scroll_display;
            let to = self.scroll_offset as f64;
            if (from - to).abs() > 0.01 {
                use crate::animation::{Easing, TransitionKind};
                animations.start(
                    TransitionKind::Scroll { from, to },
                    std::time::Duration::from_millis(150),
                    Easing::EaseOut,
                );
            }
        }
    }

    /// Sync scroll_display with animation state.
    pub fn sync_scroll(&mut self, animations: &AnimationManager) {
        if let Some(progress) = animations.scroll_progress() {
            if let Some((from, to)) = animations.scroll_values() {
                self.scroll_display = from + (to - from) * progress;
            }
        } else {
            self.scroll_display = self.scroll_offset as f64;
        }
    }
}
