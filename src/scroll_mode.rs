/// Controls how the viewport tracks the cursor while writing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScrollMode {
    /// Scroll only when the cursor reaches the edge of the viewport.
    #[default]
    Edge,
    /// Keep the cursor centered vertically at all times (typewriter style).
    Typewriter,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_edge() {
        assert_eq!(ScrollMode::default(), ScrollMode::Edge);
    }
}
