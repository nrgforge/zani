/// The terminal's color capability, detected at startup.
/// Rendering degrades gracefully from TrueColor down to Basic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorProfile {
    /// 24-bit RGB (16.7M colors). Full Palette and Dimming support.
    TrueColor,
    /// 256-color palette. Dimming approximated with fewer gradient steps.
    Color256,
    /// 16 basic ANSI colors. Focus Mode uses the `dim` attribute.
    Basic,
}

impl ColorProfile {
    /// Detect the terminal's color capability from environment variables.
    pub fn detect() -> Self {
        Self::detect_from_env(std::env::var("COLORTERM").ok().as_deref())
    }

    /// Detect from an explicit COLORTERM value (testable without env mutation).
    pub fn detect_from_env(colorterm: Option<&str>) -> Self {
        match colorterm {
            Some("truecolor") | Some("24bit") => Self::TrueColor,
            _ => {
                // Heuristic: most modern terminals support 256 colors.
                // Basic ANSI is the conservative fallback if we can't tell.
                // In practice, terminals that set TERM to *-256color support 256.
                if let Ok(term) = std::env::var("TERM") {
                    if term.contains("256color") {
                        return Self::Color256;
                    }
                }
                Self::Basic
            }
        }
    }

    /// Detect from explicit COLORTERM and TERM values (fully testable).
    pub fn detect_from(colorterm: Option<&str>, term: Option<&str>) -> Self {
        match colorterm {
            Some("truecolor") | Some("24bit") => Self::TrueColor,
            _ => match term {
                Some(t) if t.contains("256color") => Self::Color256,
                _ => Self::Basic,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Acceptance tests from scenarios.md § Color Profile Detection ===

    #[test]
    fn true_color_detected_from_colorterm_truecolor() {
        let profile = ColorProfile::detect_from(Some("truecolor"), None);
        assert_eq!(profile, ColorProfile::TrueColor);
    }

    #[test]
    fn true_color_detected_from_colorterm_24bit() {
        let profile = ColorProfile::detect_from(Some("24bit"), None);
        assert_eq!(profile, ColorProfile::TrueColor);
    }

    #[test]
    fn color_256_detected_from_term() {
        let profile = ColorProfile::detect_from(None, Some("xterm-256color"));
        assert_eq!(profile, ColorProfile::Color256);
    }

    #[test]
    fn basic_when_no_color_indicators() {
        let profile = ColorProfile::detect_from(None, Some("xterm"));
        assert_eq!(profile, ColorProfile::Basic);
    }

    #[test]
    fn basic_when_no_env_vars() {
        let profile = ColorProfile::detect_from(None, None);
        assert_eq!(profile, ColorProfile::Basic);
    }

    #[test]
    fn colorterm_takes_precedence_over_term() {
        // Even if TERM says 256color, COLORTERM=truecolor wins
        let profile = ColorProfile::detect_from(Some("truecolor"), Some("xterm-256color"));
        assert_eq!(profile, ColorProfile::TrueColor);
    }
}
