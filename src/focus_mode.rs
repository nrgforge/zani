use ratatui::style::Color;

use crate::palette::{self, Palette};

/// Focus Mode variants that control which text is dimmed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusMode {
    /// No dimming. All text at full brightness.
    Off,
    /// Current sentence at full brightness, everything else dimmed.
    Sentence,
    /// Current paragraph at full brightness, nearby paragraphs partially dimmed.
    Paragraph,
    /// Current line stays centered, surrounding text dimmed.
    Typewriter,
}

impl FocusMode {
    /// Cycle to the next variant: Off → Sentence → Paragraph → Typewriter → Off.
    pub fn next(self) -> Self {
        match self {
            Self::Off => Self::Sentence,
            Self::Sentence => Self::Paragraph,
            Self::Paragraph => Self::Typewriter,
            Self::Typewriter => Self::Off,
        }
    }
}

/// Compute the foreground color for a character based on its distance
/// from the Active Region.
///
/// `distance` is 0 for the Active Region (full brightness),
/// and increases for text further away. The meaning of distance
/// depends on the FocusMode variant:
/// - Sentence: 0 = active sentence, 1+ = everything else
/// - Paragraph: 0 = active paragraph, 1 = adjacent, 2+ = further
/// - Typewriter: 0 = active line, increases by visual line distance
pub fn dim_color(palette: &Palette, distance: usize) -> Color {
    if distance == 0 {
        return palette.foreground;
    }

    // Interpolate toward background. Closer text is brighter.
    // distance 1 → ~40% toward background
    // distance 2 → ~65% toward background
    // distance 3+ → ~80% toward background (near the dimmed_foreground)
    let t = match distance {
        1 => 0.4,
        2 => 0.65,
        _ => 0.8,
    };

    palette::interpolate(&palette.foreground, &palette.background, t)
}

/// Determine the distance of a given logical line from the active region,
/// based on the current FocusMode and cursor position.
///
/// For Sentence mode, `active_line` is the line containing the active sentence.
/// For Paragraph mode, `active_line` is the line within the active paragraph.
/// For Typewriter mode, distance is measured in visual lines from center.
///
/// Returns 0 for the active region, > 0 for surrounding text.
pub fn line_distance(
    mode: FocusMode,
    logical_line: usize,
    active_logical_line: usize,
    paragraph_bounds: Option<(usize, usize)>,
) -> usize {
    match mode {
        FocusMode::Off => 0,
        FocusMode::Sentence => {
            // Simplified: sentence focus dims everything not on the cursor's line.
            // A more refined implementation would parse sentence boundaries.
            if logical_line == active_logical_line {
                0
            } else {
                1
            }
        }
        FocusMode::Paragraph => {
            if let Some((para_start, para_end)) = paragraph_bounds {
                if logical_line >= para_start && logical_line <= para_end {
                    0
                } else {
                    // Distance in paragraphs — approximate by line distance from bounds
                    let dist_from_start = para_start.saturating_sub(logical_line);
                    let dist_from_end = logical_line.saturating_sub(para_end);
                    let line_dist = dist_from_start.max(dist_from_end);
                    // Rough: every ~3 lines counts as another paragraph distance step
                    (line_dist / 3).max(1)
                }
            } else {
                if logical_line == active_logical_line { 0 } else { 1 }
            }
        }
        FocusMode::Typewriter => {
            // Distance is measured in lines from the active line
            let diff = if logical_line > active_logical_line {
                logical_line - active_logical_line
            } else {
                active_logical_line - logical_line
            };
            diff
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Acceptance test: Focus Mode off shows all text at full brightness ===

    #[test]
    fn off_mode_returns_zero_distance_for_all_lines() {
        for line in 0..10 {
            assert_eq!(line_distance(FocusMode::Off, line, 5, None), 0);
        }
    }

    #[test]
    fn off_mode_dim_color_is_full_foreground() {
        let palette = Palette::default_palette();
        let color = dim_color(&palette, 0);
        assert_eq!(color, palette.foreground);
    }

    // === Acceptance test: Sentence Focus Mode dims surrounding text ===

    #[test]
    fn sentence_mode_active_line_is_zero_distance() {
        assert_eq!(line_distance(FocusMode::Sentence, 5, 5, None), 0);
    }

    #[test]
    fn sentence_mode_other_lines_are_nonzero_distance() {
        assert!(line_distance(FocusMode::Sentence, 3, 5, None) > 0);
        assert!(line_distance(FocusMode::Sentence, 7, 5, None) > 0);
    }

    #[test]
    fn sentence_mode_dimmed_color_differs_from_foreground() {
        let palette = Palette::default_palette();
        let active = dim_color(&palette, 0);
        let dimmed = dim_color(&palette, 1);
        assert_ne!(active, dimmed);
    }

    // === Acceptance test: Paragraph Focus Mode dims surrounding text ===

    #[test]
    fn paragraph_mode_active_paragraph_is_zero_distance() {
        // Paragraph spans lines 3-5
        assert_eq!(line_distance(FocusMode::Paragraph, 3, 4, Some((3, 5))), 0);
        assert_eq!(line_distance(FocusMode::Paragraph, 4, 4, Some((3, 5))), 0);
        assert_eq!(line_distance(FocusMode::Paragraph, 5, 4, Some((3, 5))), 0);
    }

    #[test]
    fn paragraph_mode_adjacent_text_is_dimmed() {
        // Line 1 is outside paragraph 3-5
        let dist = line_distance(FocusMode::Paragraph, 1, 4, Some((3, 5)));
        assert!(dist > 0);
    }

    #[test]
    fn paragraph_mode_further_text_is_more_dimmed() {
        let palette = Palette::default_palette();
        let near = dim_color(&palette, 1);
        let far = dim_color(&palette, 3);
        // "more dimmed" = closer to background
        // We can check that far is interpolated further toward background
        // by checking it differs from near
        assert_ne!(near, far);
    }

    // === Acceptance test: Typewriter Mode ===

    #[test]
    fn typewriter_mode_distance_increases_from_active_line() {
        assert_eq!(line_distance(FocusMode::Typewriter, 5, 5, None), 0);
        assert_eq!(line_distance(FocusMode::Typewriter, 4, 5, None), 1);
        assert_eq!(line_distance(FocusMode::Typewriter, 6, 5, None), 1);
        assert_eq!(line_distance(FocusMode::Typewriter, 3, 5, None), 2);
        assert_eq!(line_distance(FocusMode::Typewriter, 8, 5, None), 3);
    }

    // === Acceptance test: Focus Mode toggle ===

    #[test]
    fn focus_mode_cycles() {
        let mode = FocusMode::Off;
        let mode = mode.next();
        assert_eq!(mode, FocusMode::Sentence);
        let mode = mode.next();
        assert_eq!(mode, FocusMode::Paragraph);
        let mode = mode.next();
        assert_eq!(mode, FocusMode::Typewriter);
        let mode = mode.next();
        assert_eq!(mode, FocusMode::Off);
    }
}
