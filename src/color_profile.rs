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
    /// Convert an RGB color to this profile's output format.
    /// TrueColor passes through, Color256 maps to nearest index,
    /// Basic passes through (crossterm handles the mapping).
    pub fn map_color(self, color: ratatui::style::Color) -> ratatui::style::Color {
        match self {
            Self::TrueColor => color,
            Self::Color256 => {
                if let ratatui::style::Color::Rgb(r, g, b) = color {
                    ratatui::style::Color::Indexed(nearest_256_color(r, g, b))
                } else {
                    color
                }
            }
            Self::Basic => color,
        }
    }

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

/// Map an RGB color to the nearest 256-color index.
/// Uses the 6x6x6 color cube (indices 16–231) for best coverage.
pub fn nearest_256_color(r: u8, g: u8, b: u8) -> u8 {
    // The 6x6x6 color cube maps values to: 0, 95, 135, 175, 215, 255
    let cube = [0u8, 95, 135, 175, 215, 255];

    fn nearest_cube_index(val: u8) -> u8 {
        let cube = [0u8, 95, 135, 175, 215, 255];
        let mut best = 0;
        let mut best_dist = (val as i16 - cube[0] as i16).unsigned_abs();
        for (i, &c) in cube.iter().enumerate().skip(1) {
            let dist = (val as i16 - c as i16).unsigned_abs();
            if dist < best_dist {
                best = i;
                best_dist = dist;
            }
        }
        best as u8
    }

    let ri = nearest_cube_index(r);
    let gi = nearest_cube_index(g);
    let bi = nearest_cube_index(b);

    // Check if a grayscale ramp entry (indices 232–255) is closer
    // Grayscale ramp: 232 + i maps to 8 + 10*i for i in 0..24
    let cube_color_r = cube[ri as usize];
    let cube_color_g = cube[gi as usize];
    let cube_color_b = cube[bi as usize];
    let cube_dist = (r as i32 - cube_color_r as i32).pow(2)
        + (g as i32 - cube_color_g as i32).pow(2)
        + (b as i32 - cube_color_b as i32).pow(2);

    let mut best_gray = 0u8;
    let mut best_gray_dist = i32::MAX;
    for i in 0..24u8 {
        let gray_val = 8 + 10 * i as u16;
        let dist = (r as i32 - gray_val as i32).pow(2)
            + (g as i32 - gray_val as i32).pow(2)
            + (b as i32 - gray_val as i32).pow(2);
        if dist < best_gray_dist {
            best_gray = i;
            best_gray_dist = dist;
        }
    }

    if best_gray_dist < cube_dist {
        232 + best_gray
    } else {
        16 + 36 * ri + 6 * gi + bi
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

    // === Tests for nearest_256_color ===

    #[test]
    fn nearest_256_pure_black() {
        // Pure black (0,0,0) → cube index 16 (0,0,0 in 6x6x6 cube)
        // or grayscale 232 (value 8) — cube is exact match at 0, so cube wins
        let idx = nearest_256_color(0, 0, 0);
        assert_eq!(idx, 16); // 16 + 36*0 + 6*0 + 0
    }

    #[test]
    fn nearest_256_pure_white() {
        // Pure white (255,255,255) → cube index 231 (5,5,5 in 6x6x6)
        let idx = nearest_256_color(255, 255, 255);
        assert_eq!(idx, 231); // 16 + 36*5 + 6*5 + 5
    }

    #[test]
    fn nearest_256_mid_gray() {
        // Mid gray (128,128,128) — should map to grayscale ramp entry
        // Grayscale 128 is closest to 232+12=244 (value 8+10*12=128, exact)
        let idx = nearest_256_color(128, 128, 128);
        assert_eq!(idx, 244); // 232 + 12
    }

    #[test]
    fn nearest_256_red() {
        // Pure red (255,0,0) → cube (5,0,0) = 16 + 180 = 196
        let idx = nearest_256_color(255, 0, 0);
        assert_eq!(idx, 196); // 16 + 36*5 + 0 + 0
    }

    // === Test: map_color ===

    #[test]
    fn truecolor_passes_rgb_through() {
        use ratatui::style::Color;
        let color = Color::Rgb(100, 200, 50);
        assert_eq!(ColorProfile::TrueColor.map_color(color), color);
    }

    #[test]
    fn color256_maps_rgb_to_indexed() {
        use ratatui::style::Color;
        let color = Color::Rgb(255, 0, 0);
        let mapped = ColorProfile::Color256.map_color(color);
        assert_eq!(mapped, Color::Indexed(196));
    }
}
