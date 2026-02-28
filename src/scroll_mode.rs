/// Controls how the viewport tracks the cursor while writing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScrollMode {
    /// Scroll only when the cursor reaches the edge of the viewport.
    #[default]
    Edge,
    /// Keep the cursor centered vertically at all times (typewriter style).
    Typewriter,
}

impl ScrollMode {
    /// Cycle to the next variant: Edge → Typewriter → Edge.
    pub fn next(self) -> Self {
        match self {
            Self::Edge => Self::Typewriter,
            Self::Typewriter => Self::Edge,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_edge() {
        assert_eq!(ScrollMode::default(), ScrollMode::Edge);
    }

    #[test]
    fn scroll_mode_cycles() {
        let mode = ScrollMode::Edge;
        let mode = mode.next();
        assert_eq!(mode, ScrollMode::Typewriter);
        let mode = mode.next();
        assert_eq!(mode, ScrollMode::Edge);
    }
}
