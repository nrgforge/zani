use std::time::Duration;

use crate::animation::{Easing, FadeConfig};
use crate::buffer::Buffer;
use crate::focus_mode::{self, DimLayer, FocusMode, LineOpacity, fill_paragraph_target_opacities};

/// Cached sentence bounds, keyed on (buffer_version, cursor_pos).
struct SentenceBoundsCache {
    key: (u64, usize),
    bounds: Option<(usize, usize)>,
}

/// Focus mode and dimming animation state.
///
/// Groups the dimming concern: which mode is active, the paragraph-level
/// DimLayer, and the per-sentence fade queue. Data flows one way —
/// buffer content and cursor position are inputs; dimming state is output.
pub struct DimmingState {
    pub focus_mode: FocusMode,
    pub paragraph_dim: DimLayer,
    last_sentence_bounds: Option<(usize, usize)>,
    sentence_fades: Vec<(usize, usize, LineOpacity)>,
    /// Reusable buffer for paragraph target opacities (avoids alloc per frame).
    paragraph_targets_buf: Vec<f64>,
    /// Pre-populated output buffers, reused across frames.
    line_opacities_buf: Vec<f64>,
    sentence_fades_buf: Vec<(usize, usize, f64)>,
    /// Number of sentence fades currently animating (avoids per-entry syscall in dim_animating).
    sentence_animating_count: usize,
    /// True when output buffers are valid and no animations are running.
    settled: bool,
    /// Last inputs for change detection (enables early return when settled).
    last_line_count: usize,
    last_focus_mode: FocusMode,
    last_paragraph_bounds: Option<(usize, usize)>,
    /// Cached sentence bounds computation.
    sentence_cache: Option<SentenceBoundsCache>,
}

impl DimmingState {
    pub fn new() -> Self {
        Self {
            focus_mode: FocusMode::Off,
            paragraph_dim: DimLayer::new(
                FadeConfig { duration: Duration::from_millis(150), easing: Easing::EaseOut },
                FadeConfig { duration: Duration::from_millis(1800), easing: Easing::EaseOut },
            ),
            last_sentence_bounds: None,
            sentence_fades: Vec::new(),
            sentence_animating_count: 0,
            paragraph_targets_buf: Vec::new(),
            line_opacities_buf: Vec::new(),
            sentence_fades_buf: Vec::new(),
            settled: false,
            last_line_count: 0,
            last_focus_mode: FocusMode::Off,
            last_paragraph_bounds: None,
            sentence_cache: None,
        }
    }

    /// Pre-populated sentence fades for the renderer.
    pub fn sentence_fade_snapshot(&self) -> &[(usize, usize, f64)] {
        &self.sentence_fades_buf
    }

    /// Whether any dimming layer is still animating.
    pub fn dim_animating(&self) -> bool {
        self.paragraph_dim.is_animating()
            || self.sentence_animating_count > 0
    }

    /// Compute sentence bounds with caching.
    fn sentence_bounds_cached(&mut self, buffer: &Buffer, cursor_pos: usize) -> Option<(usize, usize)> {
        let key = (buffer.version(), cursor_pos);
        if let Some(ref cache) = self.sentence_cache {
            if cache.key == key {
                return cache.bounds;
            }
        }
        let bounds = focus_mode::sentence_bounds_in_buffer(buffer, cursor_pos);
        self.sentence_cache = Some(SentenceBoundsCache { key, bounds });
        bounds
    }

    /// The most recently computed sentence bounds.
    pub fn sentence_bounds(&self) -> Option<(usize, usize)> {
        self.sentence_cache.as_ref().and_then(|c| c.bounds)
    }

    /// Recompute dimming layer targets based on current focus mode and cursor position.
    /// Also populates the output buffers for line_opacities and sentence_fade_snapshot.
    /// Short-circuits when inputs haven't changed and all animations have settled.
    pub fn update(
        &mut self,
        buffer: &Buffer,
        cursor_pos: usize,
        line_count: usize,
        paragraph_bounds: Option<(usize, usize)>,
    ) {
        let sentence_bounds = self.sentence_bounds_cached(buffer, cursor_pos);
        // Early return when settled and inputs unchanged — output buffers are still valid
        if self.settled
            && line_count == self.last_line_count
            && self.focus_mode == self.last_focus_mode
            && paragraph_bounds == self.last_paragraph_bounds
            && sentence_bounds == self.last_sentence_bounds
            && !self.dim_animating()
        {
            return;
        }

        match self.focus_mode {
            FocusMode::Off => {
                self.paragraph_dim.set_all_to(1.0, line_count);
                self.last_sentence_bounds = None;
                self.sentence_fades.clear();
                self.sentence_animating_count = 0;
            }
            FocusMode::Paragraph => {
                fill_paragraph_target_opacities(&mut self.paragraph_targets_buf, line_count, paragraph_bounds);
                self.paragraph_dim.update_targets(&self.paragraph_targets_buf);
                self.last_sentence_bounds = None;
                self.sentence_fades.clear();
                self.sentence_animating_count = 0;
            }
            FocusMode::Sentence => {
                fill_paragraph_target_opacities(&mut self.paragraph_targets_buf, line_count, paragraph_bounds);
                self.paragraph_dim.update_targets(&self.paragraph_targets_buf);

                let current_start = sentence_bounds.map(|(s, _)| s);
                let last_start = self.last_sentence_bounds.map(|(s, _)| s);

                if current_start != last_start {
                    let returning_idx = sentence_bounds.and_then(|(cs, _)| {
                        self.sentence_fades.iter().position(|(fs, _, _)| *fs == cs)
                    });

                    if let Some(idx) = returning_idx {
                        if self.sentence_fades[idx].2.set_target(
                            1.0,
                            FadeConfig {
                                duration: Duration::from_millis(150),
                                easing: Easing::EaseOut,
                            },
                        ) {
                            self.sentence_animating_count += 1;
                        }
                    } else if let Some((old_start, old_end)) = self.last_sentence_bounds {
                        let mut opacity = LineOpacity::new(1.0);
                        opacity.set_target(
                            0.6,
                            FadeConfig {
                                duration: Duration::from_millis(1800),
                                easing: Easing::EaseOut,
                            },
                        );
                        self.sentence_fades.push((old_start, old_end, opacity));
                        self.sentence_animating_count += 1;
                    }
                }
                self.last_sentence_bounds = sentence_bounds;

                self.sentence_fades.retain(|(_, _, o)| o.is_animating());
                self.sentence_animating_count = self.sentence_fades.len();
            }
        }

        // Populate output buffers (reuses existing Vec capacity)
        self.line_opacities_buf.clear();
        self.line_opacities_buf.reserve(line_count.saturating_sub(self.line_opacities_buf.capacity()));
        for i in 0..line_count {
            self.line_opacities_buf.push(self.paragraph_dim.opacity(i));
        }

        self.sentence_fades_buf.clear();
        for (s, e, o) in &self.sentence_fades {
            self.sentence_fades_buf.push((*s, *e, o.current()));
        }

        // Settle: reconcile animating counts and track for next frame's early return
        self.paragraph_dim.settle();
        self.last_line_count = line_count;
        self.last_focus_mode = self.focus_mode;
        self.last_paragraph_bounds = paragraph_bounds;
        self.settled = !self.dim_animating();
    }

    /// Pre-populated per-line paragraph opacities for the renderer.
    pub fn paragraph_line_opacities(&self) -> &[f64] {
        &self.line_opacities_buf
    }
}

/// Test-only accessors for inspecting internal state.
#[cfg(test)]
impl DimmingState {
    pub fn sentence_fades_len(&self) -> usize {
        self.sentence_fades.len()
    }

    pub fn sentence_fades_is_empty(&self) -> bool {
        self.sentence_fades.is_empty()
    }

    pub fn sentence_fade_start(&self, idx: usize) -> usize {
        self.sentence_fades[idx].0
    }

    pub fn sentence_fade_has_start(&self, start: usize) -> bool {
        self.sentence_fades.iter().any(|(s, _, _)| *s == start)
    }

    pub fn sentence_fade_target(&self, idx: usize) -> f64 {
        self.sentence_fades[idx].2.target
    }

    pub fn backdate_sentence_fade(&mut self, idx: usize, duration: std::time::Duration) {
        self.sentence_fades[idx].2.start_time =
            Some(std::time::Instant::now() - duration);
    }
}

impl Default for DimmingState {
    fn default() -> Self {
        Self::new()
    }
}
