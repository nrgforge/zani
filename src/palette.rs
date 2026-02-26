use ratatui::style::Color;

/// A named, curated color system defining foreground, background,
/// dimming endpoints, and accent colors for the Writing Surface.
#[derive(Debug, Clone)]
pub struct Palette {
    pub name: &'static str,
    pub foreground: Color,
    pub background: Color,
    pub dimmed_foreground: Color,
    pub accent_heading: Color,
    pub accent_emphasis: Color,
    pub accent_link: Color,
    pub accent_code: Color,
}

impl Palette {
    /// Returns the default palette (warm dark).
    pub fn default_palette() -> Self {
        Self {
            name: "Ember",
            foreground: Color::Rgb(220, 215, 205),
            background: Color::Rgb(40, 38, 35),
            dimmed_foreground: Color::Rgb(100, 97, 92),
            accent_heading: Color::Rgb(200, 170, 130),
            accent_emphasis: Color::Rgb(190, 185, 175),
            accent_link: Color::Rgb(150, 180, 170),
            accent_code: Color::Rgb(170, 165, 155),
        }
    }

    /// Validates that this palette satisfies Invariant 3:
    /// no pure black (#000000) or pure white (#FFFFFF).
    pub fn validate(&self) -> Result<(), PaletteError> {
        let colors = [
            ("foreground", &self.foreground),
            ("background", &self.background),
            ("dimmed_foreground", &self.dimmed_foreground),
            ("accent_heading", &self.accent_heading),
            ("accent_emphasis", &self.accent_emphasis),
            ("accent_link", &self.accent_link),
            ("accent_code", &self.accent_code),
        ];

        for (name, color) in colors {
            if is_pure_black(color) {
                return Err(PaletteError::PureBlack(name.to_string()));
            }
            if is_pure_white(color) {
                return Err(PaletteError::PureWhite(name.to_string()));
            }
        }

        // WCAG AA applies to readable text colors against background.
        // dimmed_foreground is excluded — it's a dimming endpoint that
        // intentionally fades toward the background (per ADR-004).
        let pairs = [
            ("foreground/background", &self.foreground, &self.background),
            ("accent_heading/background", &self.accent_heading, &self.background),
            ("accent_emphasis/background", &self.accent_emphasis, &self.background),
            ("accent_link/background", &self.accent_link, &self.background),
            ("accent_code/background", &self.accent_code, &self.background),
        ];

        for (name, fg, bg) in pairs {
            let ratio = contrast_ratio(fg, bg);
            if ratio < 4.5 {
                return Err(PaletteError::InsufficientContrast {
                    pair: name.to_string(),
                    ratio,
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum PaletteError {
    PureBlack(String),
    PureWhite(String),
    InsufficientContrast { pair: String, ratio: f64 },
}

/// Extract RGB components from a ratatui Color.
/// Returns None for non-RGB colors.
fn rgb_components(color: &Color) -> Option<(u8, u8, u8)> {
    match color {
        Color::Rgb(r, g, b) => Some((*r, *g, *b)),
        _ => None,
    }
}

fn is_pure_black(color: &Color) -> bool {
    rgb_components(color) == Some((0, 0, 0))
}

fn is_pure_white(color: &Color) -> bool {
    rgb_components(color) == Some((255, 255, 255))
}

/// Calculate the WCAG 2.0 contrast ratio between two colors.
/// Returns a ratio >= 1.0, where 21.0 is maximum contrast.
fn contrast_ratio(color1: &Color, color2: &Color) -> f64 {
    let l1 = relative_luminance(color1);
    let l2 = relative_luminance(color2);
    let lighter = l1.max(l2);
    let darker = l1.min(l2);
    (lighter + 0.05) / (darker + 0.05)
}

/// Calculate relative luminance per WCAG 2.0.
/// https://www.w3.org/TR/WCAG20/#relativeluminancedef
fn relative_luminance(color: &Color) -> f64 {
    let (r, g, b) = rgb_components(color).unwrap_or((0, 0, 0));
    let r = linearize(r as f64 / 255.0);
    let g = linearize(g as f64 / 255.0);
    let b = linearize(b as f64 / 255.0);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Linearize an sRGB channel value.
fn linearize(value: f64) -> f64 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

/// Interpolate between two RGB colors. `t` ranges from 0.0 (color1) to 1.0 (color2).
pub fn interpolate(color1: &Color, color2: &Color, t: f64) -> Color {
    let (r1, g1, b1) = rgb_components(color1).unwrap_or((0, 0, 0));
    let (r2, g2, b2) = rgb_components(color2).unwrap_or((0, 0, 0));
    let t = t.clamp(0.0, 1.0);
    Color::Rgb(
        (r1 as f64 + (r2 as f64 - r1 as f64) * t).round() as u8,
        (g1 as f64 + (g2 as f64 - g1 as f64) * t).round() as u8,
        (b1 as f64 + (b2 as f64 - b1 as f64) * t).round() as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Acceptance test: Default palette uses no pure black or white ===
    // Scenario from scenarios.md § Palette
    #[test]
    fn default_palette_has_no_pure_black_or_white() {
        let palette = Palette::default_palette();
        assert!(palette.validate().is_ok(), "Default palette must satisfy Invariant 3");
    }

    // === Acceptance test: All palette color pairs meet WCAG AA ===
    #[test]
    fn default_palette_meets_wcag_aa() {
        let palette = Palette::default_palette();
        // validate() checks both pure black/white AND contrast ratios
        assert!(palette.validate().is_ok());
    }

    // === Unit tests for the validation logic ===

    #[test]
    fn rejects_pure_black_foreground() {
        let palette = Palette {
            name: "bad",
            foreground: Color::Rgb(0, 0, 0),
            background: Color::Rgb(40, 38, 35),
            dimmed_foreground: Color::Rgb(100, 97, 92),
            accent_heading: Color::Rgb(200, 170, 130),
            accent_emphasis: Color::Rgb(190, 185, 175),
            accent_link: Color::Rgb(150, 180, 170),
            accent_code: Color::Rgb(170, 165, 155),
        };
        assert!(matches!(palette.validate(), Err(PaletteError::PureBlack(_))));
    }

    #[test]
    fn rejects_pure_white_background() {
        let palette = Palette {
            name: "bad",
            foreground: Color::Rgb(220, 215, 205),
            background: Color::Rgb(255, 255, 255),
            dimmed_foreground: Color::Rgb(100, 97, 92),
            accent_heading: Color::Rgb(200, 170, 130),
            accent_emphasis: Color::Rgb(190, 185, 175),
            accent_link: Color::Rgb(150, 180, 170),
            accent_code: Color::Rgb(170, 165, 155),
        };
        assert!(matches!(palette.validate(), Err(PaletteError::PureWhite(_))));
    }

    #[test]
    fn rejects_insufficient_contrast() {
        let palette = Palette {
            name: "bad",
            foreground: Color::Rgb(42, 40, 37),
            background: Color::Rgb(40, 38, 35),
            dimmed_foreground: Color::Rgb(100, 97, 92),
            accent_heading: Color::Rgb(200, 170, 130),
            accent_emphasis: Color::Rgb(190, 185, 175),
            accent_link: Color::Rgb(150, 180, 170),
            accent_code: Color::Rgb(170, 165, 155),
        };
        assert!(matches!(
            palette.validate(),
            Err(PaletteError::InsufficientContrast { .. })
        ));
    }

    // === Unit tests for contrast ratio math ===

    #[test]
    fn contrast_ratio_white_on_black_is_21() {
        let white = Color::Rgb(255, 255, 255);
        let black = Color::Rgb(0, 0, 0);
        let ratio = contrast_ratio(&white, &black);
        assert!((ratio - 21.0).abs() < 0.1);
    }

    #[test]
    fn contrast_ratio_is_symmetric() {
        let a = Color::Rgb(220, 215, 205);
        let b = Color::Rgb(40, 38, 35);
        let ratio1 = contrast_ratio(&a, &b);
        let ratio2 = contrast_ratio(&b, &a);
        assert!((ratio1 - ratio2).abs() < 0.001);
    }

    #[test]
    fn contrast_ratio_same_color_is_one() {
        let c = Color::Rgb(128, 128, 128);
        let ratio = contrast_ratio(&c, &c);
        assert!((ratio - 1.0).abs() < 0.001);
    }

    // === Unit tests for interpolation ===

    #[test]
    fn interpolate_at_zero_returns_first_color() {
        let a = Color::Rgb(220, 215, 205);
        let b = Color::Rgb(40, 38, 35);
        assert_eq!(interpolate(&a, &b, 0.0), a);
    }

    #[test]
    fn interpolate_at_one_returns_second_color() {
        let a = Color::Rgb(220, 215, 205);
        let b = Color::Rgb(40, 38, 35);
        assert_eq!(interpolate(&a, &b, 1.0), b);
    }

    #[test]
    fn interpolate_at_half_returns_midpoint() {
        let a = Color::Rgb(0, 0, 0);
        let b = Color::Rgb(200, 100, 50);
        let mid = interpolate(&a, &b, 0.5);
        assert_eq!(mid, Color::Rgb(100, 50, 25));
    }

    #[test]
    fn interpolate_clamps_out_of_range() {
        let a = Color::Rgb(100, 100, 100);
        let b = Color::Rgb(200, 200, 200);
        assert_eq!(interpolate(&a, &b, -0.5), a);
        assert_eq!(interpolate(&a, &b, 1.5), b);
    }
}
