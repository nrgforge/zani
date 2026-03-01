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
}

/// Apply dimming to a foreground color using an opacity factor (0.0–1.0).
/// opacity=1.0 returns base_fg unchanged. opacity=0.0 returns background.
/// Intermediate values interpolate linearly toward the background.
pub fn apply_dimming_with_opacity(base_fg: &Color, palette: &Palette, opacity: f64) -> Color {
    if opacity >= 1.0 {
        return *base_fg;
    }
    if opacity <= 0.0 {
        return palette.background;
    }
    palette::interpolate(base_fg, &palette.background, 1.0 - opacity)
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
            start += 1; // start after the empty line
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

/// Find sentence boundaries using Buffer's O(log n) char access instead of
/// allocating a full String + Vec<char>.
pub fn sentence_bounds_in_buffer(buffer: &crate::buffer::Buffer, cursor_idx: usize) -> Option<(usize, usize)> {
    let len = buffer.len_chars();
    if len == 0 {
        return None;
    }

    let cursor_idx = cursor_idx.min(len.saturating_sub(1));

    // Find sentence start: scan backward from cursor
    let mut start = cursor_idx;
    while start > 0 {
        let prev = start - 1;
        let prev_ch = buffer.char_at(prev);
        let start_ch = buffer.char_at(start);
        // Hard boundary: double newline (empty line)
        if prev_ch == '\n' && start < len && start_ch == '\n' {
            start += 1;
            break;
        }
        // Sentence boundary: [.!?] followed by whitespace
        if prev > 0 && is_sentence_end(buffer.char_at(prev - 1)) && prev_ch.is_whitespace() {
            start = prev;
            while start < cursor_idx && buffer.char_at(start).is_whitespace() && buffer.char_at(start) != '\n' {
                start += 1;
            }
            break;
        }
        if is_sentence_end(prev_ch) && start_ch.is_whitespace() {
            while start < cursor_idx && buffer.char_at(start).is_whitespace() && buffer.char_at(start) != '\n' {
                start += 1;
            }
            break;
        }
        start = prev;
    }

    // Find sentence end: scan forward from cursor
    let mut end = cursor_idx;
    while end < len {
        let end_ch = buffer.char_at(end);
        if is_sentence_end(end_ch) {
            end += 1;
            break;
        }
        if end_ch == '\n' && end + 1 < len && buffer.char_at(end + 1) == '\n' {
            break;
        }
        end += 1;
    }

    Some((start, end))
}

fn is_sentence_end(ch: char) -> bool {
    matches!(ch, '.' | '!' | '?')
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

/// Compute target opacities for the paragraph dimming layer.
/// `line_count` is the total number of logical lines.
/// `paragraph_bounds` is (start_line, end_line) inclusive.
pub fn paragraph_target_opacities(
    line_count: usize,
    paragraph_bounds: Option<(usize, usize)>,
) -> Vec<f64> {
    if line_count == 0 {
        return Vec::new();
    }
    let Some((para_start, para_end)) = paragraph_bounds else {
        return vec![1.0; line_count];
    };
    let mut targets = Vec::with_capacity(line_count);
    for i in 0..line_count {
        if i >= para_start && i <= para_end {
            targets.push(1.0);
        } else {
            let dist = if i < para_start {
                para_start - i
            } else {
                i - para_end
            };
            targets.push(match dist {
                1..=3 => 0.6,
                4..=6 => 0.35,
                _ => 0.2,
            });
        }
    }
    targets
}

/// Manages a vector of `LineOpacity` values. Each layer independently
/// tracks and animates per-line opacities.
#[derive(Debug, Clone)]
pub struct DimLayer {
    lines: Vec<LineOpacity>,
    fade_in: FadeConfig,
    fade_out: FadeConfig,
}

impl DimLayer {
    /// Create a DimLayer with empty lines and the given fade configurations.
    pub fn new(fade_in: FadeConfig, fade_out: FadeConfig) -> Self {
        Self {
            lines: Vec::new(),
            fade_in,
            fade_out,
        }
    }

    /// Resize the internal vec to match targets length.
    /// For new lines, create LineOpacity already at the target value (no animation).
    /// For existing lines where the target changed, start a chase animation using
    /// fade_in if brightening, fade_out if dimming.
    pub fn update_targets(&mut self, targets: &[f64]) {
        // Grow if needed
        while self.lines.len() < targets.len() {
            let idx = self.lines.len();
            self.lines.push(LineOpacity::new(targets[idx]));
        }
        // Shrink if needed
        self.lines.truncate(targets.len());
        // Update existing lines
        for (i, &target) in targets.iter().enumerate() {
            let current = self.lines[i].current_opacity();
            let config = if target > current {
                self.fade_in.clone()
            } else {
                self.fade_out.clone()
            };
            self.lines[i].set_target(target, config);
        }
    }

    /// Return the current animated opacity for a line, or 1.0 if out of bounds.
    pub fn opacity(&self, line: usize) -> f64 {
        self.lines.get(line).map_or(1.0, |lo| lo.current_opacity())
    }

    /// True if any line is still animating.
    pub fn is_animating(&self) -> bool {
        self.lines.iter().any(|lo| lo.is_animating())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

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

    // === Acceptance test: Focus Mode toggle ===

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

    // === Task 3: paragraph_target_opacities and DimLayer tests ===

    #[test]
    fn paragraph_target_opacities_active_paragraph_is_bright() {
        let targets = paragraph_target_opacities(5, Some((1, 3)));
        assert!((targets[1] - 1.0).abs() < f64::EPSILON);
        assert!((targets[2] - 1.0).abs() < f64::EPSILON);
        assert!((targets[3] - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn paragraph_target_opacities_outside_is_dimmed() {
        let targets = paragraph_target_opacities(5, Some((1, 3)));
        assert!((targets[0] - 0.6).abs() < 0.01);
        assert!((targets[4] - 0.6).abs() < 0.01);
    }

    #[test]
    fn paragraph_target_opacities_no_bounds_all_bright() {
        let targets = paragraph_target_opacities(5, None);
        for t in &targets {
            assert!((t - 1.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn paragraph_target_opacities_far_lines_more_dimmed() {
        // 20 lines, paragraph at 10-12
        let targets = paragraph_target_opacities(20, Some((10, 12)));
        // Line 0 is 10 lines away — should be 0.2
        assert!((targets[0] - 0.2).abs() < 0.01);
        // Line 8 is 2 lines away — should be 0.6
        assert!((targets[8] - 0.6).abs() < 0.01);
        // Line 5 is 5 lines away — should be 0.35
        assert!((targets[5] - 0.35).abs() < 0.01);
    }

    #[test]
    fn dim_layer_computes_paragraph_opacities() {
        let mut layer = DimLayer::new(FadeConfig::default(), FadeConfig::default());
        let targets = paragraph_target_opacities(5, Some((1, 3)));
        layer.update_targets(&targets);
        assert!((layer.opacity(0) - 0.6).abs() < 0.01);
        assert!((layer.opacity(1) - 1.0).abs() < f64::EPSILON);
        assert!((layer.opacity(2) - 1.0).abs() < f64::EPSILON);
        assert!((layer.opacity(3) - 1.0).abs() < f64::EPSILON);
        assert!((layer.opacity(4) - 0.6).abs() < 0.01);
    }

    #[test]
    fn dim_layer_is_animating_after_target_change() {
        let mut layer = DimLayer::new(FadeConfig::default(), FadeConfig::default());
        let targets_a = vec![1.0, 1.0, 0.6];
        layer.update_targets(&targets_a);
        assert!(!layer.is_animating(), "Should not animate when first initialized");

        let targets_b = vec![0.6, 1.0, 1.0];
        layer.update_targets(&targets_b);
        assert!(layer.is_animating(), "Should animate after target changes");
    }

    #[test]
    fn dim_layer_out_of_bounds_returns_one() {
        let layer = DimLayer::new(FadeConfig::default(), FadeConfig::default());
        assert!((layer.opacity(999) - 1.0).abs() < f64::EPSILON);
    }

    // === Task 4: apply_dimming_with_opacity tests ===

    #[test]
    fn apply_dimming_opacity_one_returns_base_color() {
        let palette = Palette::default_palette();
        let color = apply_dimming_with_opacity(&palette.foreground, &palette, 1.0);
        assert_eq!(color, palette.foreground);
    }

    #[test]
    fn apply_dimming_opacity_zero_returns_background() {
        let palette = Palette::default_palette();
        let color = apply_dimming_with_opacity(&palette.foreground, &palette, 0.0);
        assert_eq!(color, palette.background);
    }

    #[test]
    fn apply_dimming_opacity_half_is_midpoint() {
        use ratatui::style::Color;
        let palette = Palette::default_palette();
        let color = apply_dimming_with_opacity(&palette.foreground, &palette, 0.5);
        if let (Color::Rgb(fr, _fg, _fb), Color::Rgb(br, _bg, _bb), Color::Rgb(mr, _mg, _mb)) =
            (palette.foreground, palette.background, color)
        {
            let expected_r = ((fr as f64 + br as f64) / 2.0).round() as u8;
            assert!((mr as i16 - expected_r as i16).unsigned_abs() <= 1,
                "Red channel: expected ~{expected_r}, got {mr}");
        }
    }

    #[test]
    fn apply_dimming_opacity_respects_accent_colors() {
        let palette = Palette::default_palette();
        let dimmed = apply_dimming_with_opacity(&palette.accent_heading, &palette, 0.6);
        assert_ne!(dimmed, palette.accent_heading, "Should be dimmed");
        assert_ne!(dimmed, palette.background, "Should not be fully dimmed");
    }
}
