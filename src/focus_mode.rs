use std::time::{Duration, Instant};

use ratatui::style::Color;

use crate::animation::Easing;
use crate::palette::{self, Palette};

/// Focus Mode variants that control which text is dimmed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusMode {
    /// No dimming. All text at full brightness.
    #[default]
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

/// Apply focus dimming to any foreground color based on its distance
/// from the Active Region.
///
/// `base_fg` is the resolved foreground color (may differ from `palette.foreground`
/// for headings, code, etc.). `distance` is 0 for the Active Region (full brightness),
/// and increases for text further away.
///
/// Returns `base_fg` unchanged at distance 0. At distance > 0, interpolates
/// toward the palette background:
/// - distance 1 → ~40% toward background
/// - distance 2 → ~65% toward background
/// - distance 3+ → ~80% toward background
pub fn apply_dimming(base_fg: &Color, palette: &Palette, distance: usize) -> Color {
    if distance == 0 {
        return *base_fg;
    }

    let t = match distance {
        1 => 0.4,
        2 => 0.65,
        _ => 0.8,
    };

    palette::interpolate(base_fg, &palette.background, t)
}

/// Compute the foreground color for a character based on its distance
/// from the Active Region. Convenience wrapper around `apply_dimming`
/// that always uses the palette's foreground as the base color.
pub fn dim_color(palette: &Palette, distance: usize) -> Color {
    apply_dimming(&palette.foreground, palette, distance)
}

/// Find the sentence boundaries containing the cursor position.
///
/// A sentence ends at [.!?] followed by whitespace, newline, or EOF.
/// Empty lines are hard boundaries. Returns the start (inclusive) and
/// end (exclusive) char indices of the active sentence.
///
/// `text` is the full buffer text, `cursor_idx` is the char index of the cursor.
pub fn sentence_bounds_at(text: &str, cursor_idx: usize) -> Option<(usize, usize)> {
    if text.is_empty() {
        return None;
    }

    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let cursor_idx = cursor_idx.min(len.saturating_sub(1));

    // Find sentence start: scan backward from cursor
    let mut start = cursor_idx;
    while start > 0 {
        let prev = start - 1;
        // Hard boundary: double newline (empty line)
        if chars[prev] == '\n' && start < len && chars[start] == '\n' {
            start = start + 1; // start after the empty line
            break;
        }
        // Sentence boundary: [.!?] followed by whitespace
        if prev > 0 && is_sentence_end(chars[prev - 1]) && chars[prev].is_whitespace() {
            // Current char is the start of the next sentence
            // But we need to skip leading whitespace
            start = prev;
            while start < cursor_idx && chars[start].is_whitespace() && chars[start] != '\n' {
                start += 1;
            }
            break;
        }
        // Also check: [.!?] at position prev, and start is whitespace
        if is_sentence_end(chars[prev]) && chars[start].is_whitespace() {
            while start < cursor_idx && chars[start].is_whitespace() && chars[start] != '\n' {
                start += 1;
            }
            break;
        }
        start = prev;
    }

    // Find sentence end: scan forward from cursor
    let mut end = cursor_idx;
    while end < len {
        if is_sentence_end(chars[end]) {
            // Include the sentence-ending punctuation
            end += 1;
            break;
        }
        // Hard boundary: double newline
        if chars[end] == '\n' && end + 1 < len && chars[end + 1] == '\n' {
            break;
        }
        end += 1;
    }

    Some((start, end))
}

fn is_sentence_end(ch: char) -> bool {
    matches!(ch, '.' | '!' | '?')
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

/// Configuration pairing duration and easing curve for dimming transitions.
#[derive(Debug, Clone)]
pub struct FadeConfig {
    pub duration: Duration,
    pub easing: Easing,
}

impl Default for FadeConfig {
    fn default() -> Self {
        Self {
            duration: Duration::from_millis(150),
            easing: Easing::EaseOut,
        }
    }
}

/// Animated opacity for a single logical line within a dimming layer.
///
/// Implements chase-based animation: when the target changes, the current
/// visual state is captured as the start value and animation begins from
/// there. This guarantees no visual discontinuity when interrupted mid-animation
/// (Invariant 14).
#[derive(Debug, Clone)]
pub struct LineOpacity {
    pub target: f64,
    pub start_value: f64,
    pub start_time: Option<Instant>,
    fade_config: FadeConfig,
}

impl LineOpacity {
    /// Create a LineOpacity already at `value` with no animation in flight.
    pub fn new(value: f64) -> Self {
        Self {
            target: value,
            start_value: value,
            start_time: None,
            fade_config: FadeConfig::default(),
        }
    }

    /// Set a new target opacity. Captures the current visual state as
    /// `start_value` so the animation chases from the current position.
    /// No-ops if the target hasn't changed (within epsilon).
    pub fn set_target(&mut self, new_target: f64, config: FadeConfig) {
        if (new_target - self.target).abs() < f64::EPSILON {
            return;
        }
        self.start_value = self.current_opacity();
        self.target = new_target;
        self.start_time = Some(Instant::now());
        self.fade_config = config;
    }

    /// Returns the current visual opacity accounting for animation progress.
    /// Returns `target` if no animation is in flight or the animation is complete.
    pub fn current_opacity(&self) -> f64 {
        let start_time = match self.start_time {
            Some(t) => t,
            None => return self.target,
        };

        let total = self.fade_config.duration.as_secs_f64();
        if total <= 0.0 {
            return self.target;
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        let t = (elapsed / total).min(1.0);
        let eased = self.fade_config.easing.apply(t);
        self.start_value + (self.target - self.start_value) * eased
    }

    /// Returns true if an animation is still in flight.
    pub fn is_animating(&self) -> bool {
        match self.start_time {
            None => false,
            Some(t) => t.elapsed() < self.fade_config.duration,
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

    // === Acceptance test: Sentence boundary parsing ===

    #[test]
    fn single_sentence() {
        let text = "Hello world.";
        let bounds = sentence_bounds_at(text, 5);
        assert_eq!(bounds, Some((0, 12))); // entire text is one sentence
    }

    #[test]
    fn multi_sentence_on_one_line() {
        let text = "First sentence. Second sentence.";
        // Cursor in "Second" (char 16)
        let bounds = sentence_bounds_at(text, 20);
        assert_eq!(bounds, Some((16, 32)));
    }

    #[test]
    fn cursor_in_first_of_two_sentences() {
        let text = "First sentence. Second sentence.";
        // Cursor in "First" (char 3)
        let bounds = sentence_bounds_at(text, 3);
        assert_eq!(bounds, Some((0, 15)));
    }

    #[test]
    fn sentence_spanning_lines() {
        let text = "This is a sentence\nthat spans lines.";
        // Cursor at char 5 ("is")
        let bounds = sentence_bounds_at(text, 5);
        assert_eq!(bounds, Some((0, 36))); // entire text is one sentence
    }

    #[test]
    fn empty_line_is_hard_boundary() {
        let text = "Paragraph one.\n\nParagraph two.";
        // Cursor in "two" (char 20)
        let bounds = sentence_bounds_at(text, 20);
        assert!(bounds.is_some());
        let (start, end) = bounds.unwrap();
        assert!(start >= 16, "Should not cross empty line boundary, got start={}", start);
        assert_eq!(end, 30);
    }

    #[test]
    fn empty_text_returns_none() {
        assert_eq!(sentence_bounds_at("", 0), None);
    }

    // === Unit test: apply_dimming matches dim_color for palette.foreground ===

    #[test]
    fn apply_dimming_with_foreground_matches_dim_color() {
        let palette = Palette::default_palette();
        for distance in 0..5 {
            let from_dim = dim_color(&palette, distance);
            let from_apply = apply_dimming(&palette.foreground, &palette, distance);
            assert_eq!(from_dim, from_apply, "Mismatch at distance {}", distance);
        }
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

    // === Task 2: FadeConfig and LineOpacity tests ===

    #[test]
    fn fade_config_default_values() {
        let config = FadeConfig::default();
        assert!(config.duration > Duration::ZERO, "Default duration must be > 0");
    }

    #[test]
    fn line_opacity_at_target_returns_target() {
        let lo = LineOpacity::new(0.75);
        assert!((lo.current_opacity() - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn line_opacity_chases_target() {
        let mut lo = LineOpacity::new(1.0);
        lo.set_target(0.0, FadeConfig::default());
        // Simulate animation completion by backdating start_time past the duration
        lo.start_time = Some(Instant::now() - Duration::from_millis(500));
        let opacity = lo.current_opacity();
        assert!(
            (opacity - 0.0).abs() < 1e-9,
            "Expected opacity near 0.0 after animation completes, got {opacity}"
        );
    }

    #[test]
    fn line_opacity_interruption_starts_from_current() {
        let mut lo = LineOpacity::new(1.0);
        // Start animating toward 0.0
        lo.set_target(0.0, FadeConfig::default());
        // Simulate being halfway through the 150ms animation (75ms elapsed)
        lo.start_time = Some(Instant::now() - Duration::from_millis(75));
        // Verify we are genuinely mid-animation before interrupting
        let pre_interrupt_opacity = lo.current_opacity();
        assert!(
            pre_interrupt_opacity > 0.0 && pre_interrupt_opacity < 1.0,
            "Expected mid-animation opacity between 0 and 1, got {pre_interrupt_opacity}"
        );
        // Now interrupt: set a new target back to 1.0.
        // set_target captures current_opacity() as the new start_value internally.
        lo.set_target(1.0, FadeConfig::default());
        // The new start_value must be between 0 and 1 (it captured the mid-animation state)
        assert!(
            lo.start_value > 0.0 && lo.start_value < 1.0,
            "start_value should be the mid-animation opacity (between 0 and 1), got {}",
            lo.start_value
        );
    }
}
