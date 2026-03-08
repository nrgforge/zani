mod collection;

use ratatui::style::Color;

/// Mood-based grouping of palettes along two axes:
/// brightness (Dark, Light) and character (Warm, Cool, Vivid, Muted).
/// The organizing taxonomy for the Palette Browser (ADR-009, ADR-014).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AffectiveCategory {
    DarkWarm,
    DarkCool,
    DarkVivid,
    DarkMuted,
    LightWarm,
    LightCool,
    LightVivid,
    LightMuted,
}

impl AffectiveCategory {
    /// Returns all categories in display order.
    pub fn all() -> &'static [AffectiveCategory] {
        &[
            AffectiveCategory::DarkWarm,
            AffectiveCategory::DarkCool,
            AffectiveCategory::DarkVivid,
            AffectiveCategory::DarkMuted,
            AffectiveCategory::LightWarm,
            AffectiveCategory::LightCool,
            AffectiveCategory::LightVivid,
            AffectiveCategory::LightMuted,
        ]
    }

    /// Human-readable label for display.
    pub fn label(&self) -> &'static str {
        match self {
            AffectiveCategory::DarkWarm => "Dark — Warm",
            AffectiveCategory::DarkCool => "Dark — Cool",
            AffectiveCategory::DarkVivid => "Dark — Vivid",
            AffectiveCategory::DarkMuted => "Dark — Muted",
            AffectiveCategory::LightWarm => "Light — Warm",
            AffectiveCategory::LightCool => "Light — Cool",
            AffectiveCategory::LightVivid => "Light — Vivid",
            AffectiveCategory::LightMuted => "Light — Muted",
        }
    }

    /// Returns true if this is a Dark brightness category.
    pub fn is_dark(&self) -> bool {
        matches!(self, AffectiveCategory::DarkWarm | AffectiveCategory::DarkCool | AffectiveCategory::DarkVivid | AffectiveCategory::DarkMuted)
    }
}

/// Hand-tuned 256-color alternate values for a Palette (ADR-012).
/// RGB values chosen so that `nearest_256_color` maps them to specific
/// 6x6x6 cube entries that preserve mood character.
#[derive(Debug, Clone, Copy)]
pub struct Color256Overrides {
    pub foreground: Color,
    pub background: Color,
    pub dimmed_foreground: Color,
    pub accent_heading: Color,
    pub accent_emphasis: Color,
    pub accent_link: Color,
    pub accent_code: Color,
}

/// A named, curated color system defining foreground, background,
/// dimming endpoints, and accent colors for the Writing Surface.
/// Designed as a mood instrument — priming a specific affective state
/// rather than serving as decoration. Named after a Cascadia bioregion
/// species (ADR-015, ADR-016). Belongs to an Affective Category.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub name: &'static str,
    /// Botanically accurate one-line note: scientific name, appearance,
    /// and Cascadia bioregion ecological context (ADR-015, ADR-016).
    pub provenance: &'static str,
    pub foreground: Color,
    pub background: Color,
    pub dimmed_foreground: Color,
    pub accent_heading: Color,
    pub accent_emphasis: Color,
    pub accent_link: Color,
    pub accent_code: Color,
    /// Mood-based grouping (ADR-009, ADR-014).
    pub category: AffectiveCategory,
    /// OKLCH hue angle for Perceptual Sort Order within category (ADR-009).
    pub sort_key: f64,
    /// Optional hand-tuned 256-color alternate values (ADR-012).
    pub color_256: Option<Color256Overrides>,
}

impl Palette {
    /// Returns the default palette (Manzanita — Dark Warm).
    pub fn default_palette() -> Self {
        collection::default_palette()
    }

    /// Look up a palette by name. Returns the default palette if not found.
    pub fn by_name(name: &str) -> Self {
        Self::all()
            .into_iter()
            .find(|p| p.name == name)
            .unwrap_or_else(Self::default_palette)
    }

    /// Interpolate between two palettes. `progress` ranges from 0.0 (`from`) to 1.0 (`to`).
    pub fn blend(from: &Palette, to: &Palette, progress: f64) -> Palette {
        Palette {
            name: to.name,
            provenance: to.provenance,
            foreground: interpolate(&from.foreground, &to.foreground, progress),
            background: interpolate(&from.background, &to.background, progress),
            dimmed_foreground: interpolate(&from.dimmed_foreground, &to.dimmed_foreground, progress),
            accent_heading: interpolate(&from.accent_heading, &to.accent_heading, progress),
            accent_emphasis: interpolate(&from.accent_emphasis, &to.accent_emphasis, progress),
            accent_link: interpolate(&from.accent_link, &to.accent_link, progress),
            accent_code: interpolate(&from.accent_code, &to.accent_code, progress),
            category: to.category,
            sort_key: to.sort_key,
            color_256: to.color_256,
        }
    }

    /// Returns the 256-color overrides if present.
    pub fn color_256_overrides(&self) -> Option<&Color256Overrides> {
        self.color_256.as_ref()
    }

    /// Returns all built-in palettes.
    pub fn all() -> Vec<Self> {
        collection::all_palettes()
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

        // Validate 256-color overrides if present (ADR-012, Invariant 3).
        if let Some(ref ov) = self.color_256 {
            let ov_colors = [
                ("256:foreground", &ov.foreground),
                ("256:background", &ov.background),
                ("256:dimmed_foreground", &ov.dimmed_foreground),
                ("256:accent_heading", &ov.accent_heading),
                ("256:accent_emphasis", &ov.accent_emphasis),
                ("256:accent_link", &ov.accent_link),
                ("256:accent_code", &ov.accent_code),
            ];

            for (name, color) in ov_colors {
                if is_pure_black(color) {
                    return Err(PaletteError::PureBlack(name.to_string()));
                }
                if is_pure_white(color) {
                    return Err(PaletteError::PureWhite(name.to_string()));
                }
            }

            let ov_pairs = [
                ("256:foreground/background", &ov.foreground, &ov.background),
                ("256:accent_heading/background", &ov.accent_heading, &ov.background),
                ("256:accent_emphasis/background", &ov.accent_emphasis, &ov.background),
                ("256:accent_link/background", &ov.accent_link, &ov.background),
                ("256:accent_code/background", &ov.accent_code, &ov.background),
            ];

            for (name, fg, bg) in ov_pairs {
                let ratio = contrast_ratio(fg, bg);
                if ratio < 4.5 {
                    return Err(PaletteError::InsufficientContrast {
                        pair: name.to_string(),
                        ratio,
                    });
                }
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
pub(crate) fn oklch_hue(r: u8, g: u8, b: u8) -> f64 {
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

        // Check all four character types exist within each brightness level (ADR-014)
        let dark_labels: Vec<&str> = all_cats.iter().filter(|c| c.is_dark()).map(|c| c.label()).collect();
        assert!(dark_labels.iter().any(|l| l.contains("Warm")));
        assert!(dark_labels.iter().any(|l| l.contains("Cool")));
        assert!(dark_labels.iter().any(|l| l.contains("Vivid")));
        assert!(dark_labels.iter().any(|l| l.contains("Muted")));

        let light_labels: Vec<&str> = all_cats.iter().filter(|c| !c.is_dark()).map(|c| c.label()).collect();
        assert!(light_labels.iter().any(|l| l.contains("Warm")));
        assert!(light_labels.iter().any(|l| l.contains("Cool")));
        assert!(light_labels.iter().any(|l| l.contains("Vivid")));
        assert!(light_labels.iter().any(|l| l.contains("Muted")));
    }

    // === Acceptance tests: Muted Affective Category (ADR-014) ===

    #[test]
    fn dark_muted_and_light_muted_categories_exist() {
        let all_cats = AffectiveCategory::all();
        assert!(all_cats.contains(&AffectiveCategory::DarkMuted));
        assert!(all_cats.contains(&AffectiveCategory::LightMuted));
        assert_eq!(all_cats.len(), 8);
    }

    #[test]
    fn dark_muted_is_classified_as_dark() {
        assert!(AffectiveCategory::DarkMuted.is_dark());
        assert!(!AffectiveCategory::LightMuted.is_dark());
    }

    #[test]
    fn muted_categories_appear_after_vivid_in_display_order() {
        let all_cats = AffectiveCategory::all();
        let dark_vivid_pos = all_cats.iter().position(|c| *c == AffectiveCategory::DarkVivid).unwrap();
        let dark_muted_pos = all_cats.iter().position(|c| *c == AffectiveCategory::DarkMuted).unwrap();
        let light_vivid_pos = all_cats.iter().position(|c| *c == AffectiveCategory::LightVivid).unwrap();
        let light_muted_pos = all_cats.iter().position(|c| *c == AffectiveCategory::LightMuted).unwrap();
        assert!(dark_muted_pos > dark_vivid_pos, "DarkMuted should appear after DarkVivid");
        assert!(light_muted_pos > light_vivid_pos, "LightMuted should appear after LightVivid");
    }

    #[test]
    fn muted_categories_display_correct_labels() {
        assert_eq!(AffectiveCategory::DarkMuted.label(), "Dark — Muted");
        assert_eq!(AffectiveCategory::LightMuted.label(), "Light — Muted");
    }

    // === Unit tests for the validation logic ===

    /// Helper to build a test palette with required fields.
    fn test_palette(
        name: &'static str,
        fg: Color, bg: Color, dimmed: Color,
        heading: Color, emphasis: Color, link: Color, code: Color,
    ) -> Palette {
        Palette {
            name, provenance: "",
            foreground: fg, background: bg, dimmed_foreground: dimmed,
            accent_heading: heading, accent_emphasis: emphasis,
            accent_link: link, accent_code: code,
            category: AffectiveCategory::DarkWarm,
            sort_key: 0.0,
            color_256: None,
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

    // === Acceptance tests: 256-Color Validation (ADR-012, Invariant 3) ===

    #[test]
    fn validate_rejects_256_overrides_with_insufficient_contrast() {
        let mut palette = Palette::default_palette();
        palette.color_256 = Some(Color256Overrides {
            foreground: Color::Rgb(42, 40, 37), // too close to background
            background: Color::Rgb(38, 38, 38),
            dimmed_foreground: Color::Rgb(95, 95, 95),
            accent_heading: Color::Rgb(215, 175, 135),
            accent_emphasis: Color::Rgb(175, 175, 175),
            accent_link: Color::Rgb(135, 175, 175),
            accent_code: Color::Rgb(175, 175, 135),
        });
        assert!(matches!(
            palette.validate(),
            Err(PaletteError::InsufficientContrast { .. })
        ));
    }

    #[test]
    fn validate_rejects_256_overrides_with_pure_black() {
        let mut palette = Palette::default_palette();
        palette.color_256 = Some(Color256Overrides {
            foreground: Color::Rgb(215, 215, 215),
            background: Color::Rgb(0, 0, 0), // pure black
            dimmed_foreground: Color::Rgb(95, 95, 95),
            accent_heading: Color::Rgb(215, 175, 135),
            accent_emphasis: Color::Rgb(175, 175, 175),
            accent_link: Color::Rgb(135, 175, 175),
            accent_code: Color::Rgb(175, 175, 135),
        });
        assert!(matches!(palette.validate(), Err(PaletteError::PureBlack(_))));
    }

    #[test]
    fn validate_accepts_valid_256_overrides() {
        let mut palette = Palette::default_palette();
        palette.color_256 = Some(Color256Overrides {
            foreground: Color::Rgb(215, 215, 215),
            background: Color::Rgb(38, 38, 38),
            dimmed_foreground: Color::Rgb(95, 95, 95),
            accent_heading: Color::Rgb(215, 175, 135),
            accent_emphasis: Color::Rgb(175, 175, 175),
            accent_link: Color::Rgb(135, 175, 175),
            accent_code: Color::Rgb(175, 175, 135),
        });
        assert!(palette.validate().is_ok(), "Valid 256-color overrides should pass validation");
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
        let sitka = Palette::by_name("Sitka");
        let idx = sitka.index_in_all();
        assert!(idx > 0, "Sitka should not be at index 0 (that's the default)");
    }

    #[test]
    fn index_in_all_unknown_returns_zero() {
        let mut custom = Palette::default_palette();
        custom.name = "Unknown";
        assert_eq!(custom.index_in_all(), 0);
    }

    // === Acceptance tests: PNW Flora Naming Register (ADR-015) ===

    #[test]
    fn default_palette_is_manzanita() {
        let p = Palette::default_palette();
        assert_eq!(p.name, "Manzanita");
        assert_eq!(p.category, AffectiveCategory::DarkWarm);
    }

    #[test]
    fn no_old_prototype_names_remain() {
        let old_names = [
            "Ember", "Hearthstone", "Inkwell", "Moonstone", "Neon Noir",
            "Aurora", "Parchment", "Manuscript", "Glacier", "Daybreak",
            "Balsamroot", "Reindeer Lichen", "Columbine",
        ];
        for palette in Palette::all() {
            assert!(
                !old_names.contains(&palette.name),
                "Palette '{}' uses a retired prototype name",
                palette.name
            );
        }
    }

    #[test]
    fn collection_contains_exactly_40_palettes() {
        assert_eq!(Palette::all().len(), 40);
    }

    #[test]
    fn each_category_contains_exactly_5_palettes() {
        let grouped = Palette::all_by_category();
        for (cat, palettes) in &grouped {
            assert_eq!(
                palettes.len(), 5,
                "Category {:?} has {} palettes, expected 5",
                cat, palettes.len()
            );
        }
        assert_eq!(grouped.len(), 8, "Should have 8 affective categories");
    }

    #[test]
    fn every_palette_has_provenance_description() {
        for palette in Palette::all() {
            assert!(
                !palette.provenance.is_empty(),
                "Palette '{}' has empty provenance",
                palette.name
            );
            // Provenance should reference a scientific name (contains em dash or
            // genus-species pattern). We check for the em dash separator which
            // all our provenance descriptions use.
            assert!(
                palette.provenance.contains('—'),
                "Palette '{}' provenance should include scientific name with em dash separator",
                palette.name
            );
        }
    }

    #[test]
    fn sibling_palettes_have_diverse_hue_angles() {
        let grouped = Palette::all_by_category();
        let mut violations = Vec::new();
        for (cat, palettes) in &grouped {
            let hues: Vec<f64> = palettes.iter().map(|p| p.sort_key).collect();
            for i in 0..hues.len() {
                for j in (i + 1)..hues.len() {
                    let diff = (hues[i] - hues[j]).abs();
                    let min_diff = diff.min(360.0 - diff);
                    if min_diff < 15.0 {
                        violations.push(format!(
                            "{:?}: '{}' ({:.1}°) and '{}' ({:.1}°) = {:.1}° apart",
                            cat, palettes[i].name, hues[i], palettes[j].name, hues[j], min_diff
                        ));
                    }
                }
            }
        }
        assert!(
            violations.is_empty(),
            "Hue diversity violations:\n{}",
            violations.join("\n")
        );
    }

    // === Acceptance tests: Species-Color Validation Remediation (ADR-016) ===

    #[test]
    fn bracken_replaces_oregon_sunshine_in_light_warm() {
        let all = Palette::all();
        let bracken = all.iter().find(|p| p.name == "Bracken");
        assert!(bracken.is_some(), "Bracken should exist in the collection");
        let b = bracken.unwrap();
        assert_eq!(b.category, AffectiveCategory::LightWarm);
        assert!(
            b.provenance.contains("Pteridium aquilinum"),
            "Bracken provenance should reference Pteridium aquilinum"
        );
    }

    #[test]
    fn licorice_fern_replaces_balsamroot_in_light_warm() {
        let all = Palette::all();
        let fern = all.iter().find(|p| p.name == "Licorice Fern");
        assert!(fern.is_some(), "Licorice Fern should exist in the collection");
        let f = fern.unwrap();
        assert_eq!(f.category, AffectiveCategory::LightWarm);
        assert!(
            f.provenance.contains("Polypodium glycyrrhiza"),
            "Licorice Fern provenance should reference Polypodium glycyrrhiza"
        );
    }

    #[test]
    fn silver_fir_replaces_reindeer_lichen_in_light_muted() {
        let all = Palette::all();
        let fir = all.iter().find(|p| p.name == "Silver Fir");
        assert!(fir.is_some(), "Silver Fir should exist in the collection");
        let f = fir.unwrap();
        assert_eq!(f.category, AffectiveCategory::LightMuted);
        assert!(
            f.provenance.contains("Abies amabilis"),
            "Silver Fir provenance should reference Abies amabilis"
        );
    }

    #[test]
    fn oregon_sunshine_in_light_vivid_with_yellow_hue() {
        let all = Palette::all();
        let sunshine = all.iter().find(|p| p.name == "Oregon Sunshine");
        assert!(
            sunshine.is_some(),
            "Oregon Sunshine should exist in the collection"
        );
        let s = sunshine.unwrap();
        assert_eq!(s.category, AffectiveCategory::LightVivid);
        assert!(
            s.provenance.contains("Eriophyllum lanatum"),
            "Oregon Sunshine provenance should reference Eriophyllum lanatum"
        );
        // Background hue should be in the yellow range (roughly 70°–120° OKLCH)
        assert!(
            s.sort_key > 70.0 && s.sort_key < 120.0,
            "Oregon Sunshine background hue {:.1}° should be in yellow range",
            s.sort_key
        );
    }

    #[test]
    fn columbine_retired_from_collection() {
        let all = Palette::all();
        assert!(
            !all.iter().any(|p| p.name == "Columbine"),
            "Columbine should not be in the collection after retirement"
        );
    }

    #[test]
    fn jack_o_lantern_provenance_uses_olivascens() {
        let all = Palette::all();
        let jol = all.iter().find(|p| p.name == "Jack-o'-Lantern").unwrap();
        assert!(
            jol.provenance.contains("olivascens"),
            "Jack-o'-Lantern should reference O. olivascens, not O. olearius"
        );
        assert!(
            !jol.provenance.contains("olearius"),
            "Jack-o'-Lantern should not reference O. olearius"
        );
    }

    #[test]
    fn corrected_provenances_match_botanical_sources() {
        let all = Palette::all();

        let oakmoss = all.iter().find(|p| p.name == "Oakmoss").unwrap();
        assert!(
            !oakmoss.provenance.contains("teal"),
            "Oakmoss provenance should not say 'teal' (real color is gray-green to olive)"
        );

        let witchs_hair = all.iter().find(|p| p.name == "Witch's Hair").unwrap();
        assert!(
            !witchs_hair.provenance.contains("olive-black"),
            "Witch's Hair provenance should not say 'olive-black' (real color is pale yellow-green)"
        );

        let partridgefoot = all.iter().find(|p| p.name == "Partridgefoot").unwrap();
        assert!(
            !partridgefoot.provenance.contains("gray-green"),
            "Partridgefoot provenance should not say 'gray-green' (USDA says glossy green)"
        );

        let lupine = all.iter().find(|p| p.name == "Lupine").unwrap();
        assert!(
            !lupine.provenance.contains("silvery"),
            "Lupine provenance should not say 'silvery' (L. latifolius is not silvery)"
        );
    }
}
