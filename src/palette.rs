use ratatui::style::Color;

/// Mood-based grouping of palettes along two axes:
/// brightness (Dark, Light) and character (Warm, Cool, Vivid).
/// The organizing taxonomy for the Palette Browser (ADR-009).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AffectiveCategory {
    DarkWarm,
    DarkCool,
    DarkVivid,
    LightWarm,
    LightCool,
    LightVivid,
}

impl AffectiveCategory {
    /// Returns all categories in display order.
    pub fn all() -> &'static [AffectiveCategory] {
        &[
            AffectiveCategory::DarkWarm,
            AffectiveCategory::DarkCool,
            AffectiveCategory::DarkVivid,
            AffectiveCategory::LightWarm,
            AffectiveCategory::LightCool,
            AffectiveCategory::LightVivid,
        ]
    }

    /// Human-readable label for display.
    pub fn label(&self) -> &'static str {
        match self {
            AffectiveCategory::DarkWarm => "Dark — Warm",
            AffectiveCategory::DarkCool => "Dark — Cool",
            AffectiveCategory::DarkVivid => "Dark — Vivid",
            AffectiveCategory::LightWarm => "Light — Warm",
            AffectiveCategory::LightCool => "Light — Cool",
            AffectiveCategory::LightVivid => "Light — Vivid",
        }
    }

    /// Returns true if this is a Dark brightness category.
    pub fn is_dark(&self) -> bool {
        matches!(self, AffectiveCategory::DarkWarm | AffectiveCategory::DarkCool | AffectiveCategory::DarkVivid)
    }
}

/// A named, curated color system defining foreground, background,
/// dimming endpoints, and accent colors for the Writing Surface.
/// Designed as a mood instrument — priming a specific affective state
/// rather than serving as decoration. Belongs to an Affective Category.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub name: &'static str,
    pub foreground: Color,
    pub background: Color,
    pub dimmed_foreground: Color,
    pub accent_heading: Color,
    pub accent_emphasis: Color,
    pub accent_link: Color,
    pub accent_code: Color,
    /// Mood-based grouping (ADR-009).
    pub category: AffectiveCategory,
    /// OKLCH hue angle for Perceptual Sort Order within category (ADR-009).
    pub sort_key: f64,
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
            category: AffectiveCategory::DarkWarm,
            sort_key: oklch_hue(40, 38, 35),
        }
    }

    /// Cool dark palette — deep navy with silver text.
    pub fn inkwell() -> Self {
        Self {
            name: "Inkwell",
            foreground: Color::Rgb(205, 210, 220),
            background: Color::Rgb(30, 32, 40),
            dimmed_foreground: Color::Rgb(90, 93, 100),
            accent_heading: Color::Rgb(140, 170, 210),
            accent_emphasis: Color::Rgb(180, 185, 195),
            accent_link: Color::Rgb(130, 185, 175),
            accent_code: Color::Rgb(160, 165, 175),
            category: AffectiveCategory::DarkCool,
            sort_key: oklch_hue(30, 32, 40),
        }
    }

    /// Warm light palette — cream paper with dark ink.
    pub fn parchment() -> Self {
        Self {
            name: "Parchment",
            foreground: Color::Rgb(60, 50, 40),
            background: Color::Rgb(240, 230, 215),
            dimmed_foreground: Color::Rgb(170, 163, 152),
            accent_heading: Color::Rgb(120, 75, 40),
            accent_emphasis: Color::Rgb(70, 60, 50),
            accent_link: Color::Rgb(60, 95, 60),
            accent_code: Color::Rgb(100, 90, 80),
            category: AffectiveCategory::LightWarm,
            sort_key: oklch_hue(240, 230, 215),
        }
    }

    /// Interpolate between two palettes. `progress` ranges from 0.0 (`from`) to 1.0 (`to`).
    pub fn blend(from: &Palette, to: &Palette, progress: f64) -> Palette {
        Palette {
            name: to.name,
            foreground: interpolate(&from.foreground, &to.foreground, progress),
            background: interpolate(&from.background, &to.background, progress),
            dimmed_foreground: interpolate(&from.dimmed_foreground, &to.dimmed_foreground, progress),
            accent_heading: interpolate(&from.accent_heading, &to.accent_heading, progress),
            accent_emphasis: interpolate(&from.accent_emphasis, &to.accent_emphasis, progress),
            accent_link: interpolate(&from.accent_link, &to.accent_link, progress),
            accent_code: interpolate(&from.accent_code, &to.accent_code, progress),
            category: to.category,
            sort_key: to.sort_key,
        }
    }

    /// Returns all built-in palettes.
    pub fn all() -> Vec<Self> {
        vec![Self::default_palette(), Self::inkwell(), Self::parchment()]
    }

    /// Returns palettes grouped by Affective Category, sorted by
    /// Perceptual Sort Order (OKLCH hue angle) within each category.
    pub fn all_by_category() -> Vec<(AffectiveCategory, Vec<Self>)> {
        let all = Self::all();
        let mut groups: Vec<(AffectiveCategory, Vec<Self>)> = Vec::new();

        for &cat in AffectiveCategory::all() {
            let mut palettes: Vec<Self> = all.iter()
                .filter(|p| p.category == cat)
                .copied()
                .collect();
            if !palettes.is_empty() {
                palettes.sort_by(|a, b| a.sort_key.partial_cmp(&b.sort_key).unwrap_or(std::cmp::Ordering::Equal));
                groups.push((cat, palettes));
            }
        }

        groups
    }

    /// Find this palette's position in `Palette::all()`.
    pub fn index_in_all(&self) -> usize {
        Self::all()
            .iter()
            .position(|p| p.name == self.name)
            .unwrap_or(0)
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

/// Compute the OKLCH hue angle (in degrees) for an sRGB color.
/// Used as the Perceptual Sort Order key within an Affective Category (ADR-009).
fn oklch_hue(r: u8, g: u8, b: u8) -> f64 {
    // sRGB → linear RGB
    let lr = linearize(r as f64 / 255.0);
    let lg = linearize(g as f64 / 255.0);
    let lb = linearize(b as f64 / 255.0);

    // linear RGB → OKLab (Björn Ottosson's method)
    let l = 0.4122214708 * lr + 0.5363325363 * lg + 0.0514459929 * lb;
    let m = 0.2119034982 * lr + 0.6806995451 * lg + 0.1073969566 * lb;
    let s = 0.0883024619 * lr + 0.2817188376 * lg + 0.6299787005 * lb;

    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    let _ok_l = 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_;
    let ok_a = 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_;
    let ok_b = 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_;

    // OKLab → OKLCH hue angle
    let hue_rad = ok_b.atan2(ok_a);
    let hue_deg = hue_rad.to_degrees();
    if hue_deg < 0.0 { hue_deg + 360.0 } else { hue_deg }
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
    fn all_palettes_satisfy_invariant_3() {
        for palette in Palette::all() {
            assert!(
                palette.validate().is_ok(),
                "Palette '{}' fails Invariant 3: {:?}",
                palette.name,
                palette.validate().err()
            );
        }
    }

    // === Acceptance tests: Affective Categories (ADR-009) ===

    #[test]
    fn every_palette_belongs_to_exactly_one_affective_category() {
        for palette in Palette::all() {
            // category is a required field — the type system ensures exactly one.
            // Verify the category is a valid variant by checking the label isn't empty.
            assert!(
                !palette.category.label().is_empty(),
                "Palette '{}' must have an Affective Category",
                palette.name
            );
        }
    }

    #[test]
    fn palettes_within_category_sorted_by_perceptual_sort_order() {
        for (cat, palettes) in Palette::all_by_category() {
            for window in palettes.windows(2) {
                assert!(
                    window[0].sort_key <= window[1].sort_key,
                    "In category {:?}, '{}' (hue {:.1}) should sort before '{}' (hue {:.1})",
                    cat, window[0].name, window[0].sort_key, window[1].name, window[1].sort_key
                );
            }
        }
    }

    #[test]
    fn affective_category_taxonomy_covers_brightness_and_character() {
        let all_cats = AffectiveCategory::all();
        let has_dark = all_cats.iter().any(|c| c.is_dark());
        let has_light = all_cats.iter().any(|c| !c.is_dark());
        assert!(has_dark, "Taxonomy must include Dark brightness categories");
        assert!(has_light, "Taxonomy must include Light brightness categories");

        // Check all three character types exist within each brightness level
        let dark_labels: Vec<&str> = all_cats.iter().filter(|c| c.is_dark()).map(|c| c.label()).collect();
        assert!(dark_labels.iter().any(|l| l.contains("Warm")));
        assert!(dark_labels.iter().any(|l| l.contains("Cool")));
        assert!(dark_labels.iter().any(|l| l.contains("Vivid")));

        let light_labels: Vec<&str> = all_cats.iter().filter(|c| !c.is_dark()).map(|c| c.label()).collect();
        assert!(light_labels.iter().any(|l| l.contains("Warm")));
        assert!(light_labels.iter().any(|l| l.contains("Cool")));
        assert!(light_labels.iter().any(|l| l.contains("Vivid")));
    }

    // === Unit tests for the validation logic ===

    /// Helper to build a test palette with required fields.
    fn test_palette(
        name: &'static str,
        fg: Color, bg: Color, dimmed: Color,
        heading: Color, emphasis: Color, link: Color, code: Color,
    ) -> Palette {
        Palette {
            name, foreground: fg, background: bg, dimmed_foreground: dimmed,
            accent_heading: heading, accent_emphasis: emphasis,
            accent_link: link, accent_code: code,
            category: AffectiveCategory::DarkWarm,
            sort_key: 0.0,
        }
    }

    #[test]
    fn rejects_pure_black_foreground() {
        let palette = test_palette(
            "bad",
            Color::Rgb(0, 0, 0), Color::Rgb(40, 38, 35), Color::Rgb(100, 97, 92),
            Color::Rgb(200, 170, 130), Color::Rgb(190, 185, 175),
            Color::Rgb(150, 180, 170), Color::Rgb(170, 165, 155),
        );
        assert!(matches!(palette.validate(), Err(PaletteError::PureBlack(_))));
    }

    #[test]
    fn rejects_pure_white_background() {
        let palette = test_palette(
            "bad",
            Color::Rgb(220, 215, 205), Color::Rgb(255, 255, 255), Color::Rgb(100, 97, 92),
            Color::Rgb(200, 170, 130), Color::Rgb(190, 185, 175),
            Color::Rgb(150, 180, 170), Color::Rgb(170, 165, 155),
        );
        assert!(matches!(palette.validate(), Err(PaletteError::PureWhite(_))));
    }

    #[test]
    fn rejects_insufficient_contrast() {
        let palette = test_palette(
            "bad",
            Color::Rgb(42, 40, 37), Color::Rgb(40, 38, 35), Color::Rgb(100, 97, 92),
            Color::Rgb(200, 170, 130), Color::Rgb(190, 185, 175),
            Color::Rgb(150, 180, 170), Color::Rgb(170, 165, 155),
        );
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
        assert_eq!(interpolate(&a, &b, -0.5), a, "negative t should clamp to first color");
        assert_eq!(interpolate(&a, &b, 1.5), b, "t > 1 should clamp to second color");
    }

    // === index_in_all ===

    #[test]
    fn index_in_all_finds_known_palette() {
        let inkwell = Palette::inkwell();
        assert_eq!(inkwell.index_in_all(), 1);
    }

    #[test]
    fn index_in_all_unknown_returns_zero() {
        let mut custom = Palette::default_palette();
        custom.name = "Unknown";
        assert_eq!(custom.index_in_all(), 0);
    }
}
