//! Palette collection: the 40 curated palettes named after PNW flora (ADR-015).
//! This file is separated from the palette logic so palette data is easy to
//! alter and review independently.

use ratatui::style::Color;
use super::{AffectiveCategory, Palette, oklch_hue};

/// Returns all built-in palettes.
pub(super) fn all_palettes() -> Vec<Palette> {
    vec![
        default_palette(),
        hearthstone(),
        inkwell(),
        moonstone(),
        neon_noir(),
        aurora(),
        parchment(),
        manuscript(),
        glacier(),
        daybreak(),
    ]
}

/// Returns the default palette (warm dark).
pub(super) fn default_palette() -> Palette {
    Palette {
        name: "Ember",
        provenance: "",
        foreground: Color::Rgb(220, 215, 205),
        background: Color::Rgb(40, 38, 35),
        dimmed_foreground: Color::Rgb(100, 97, 92),
        accent_heading: Color::Rgb(200, 170, 130),
        accent_emphasis: Color::Rgb(190, 185, 175),
        accent_link: Color::Rgb(150, 180, 170),
        accent_code: Color::Rgb(170, 165, 155),
        category: AffectiveCategory::DarkWarm,
        sort_key: oklch_hue(40, 38, 35),
        color_256: None,
    }
}

/// Cool dark palette — deep navy with silver text.
fn inkwell() -> Palette {
    Palette {
        name: "Inkwell",
        provenance: "",
        foreground: Color::Rgb(205, 210, 220),
        background: Color::Rgb(30, 32, 40),
        dimmed_foreground: Color::Rgb(90, 93, 100),
        accent_heading: Color::Rgb(140, 170, 210),
        accent_emphasis: Color::Rgb(180, 185, 195),
        accent_link: Color::Rgb(130, 185, 175),
        accent_code: Color::Rgb(160, 165, 175),
        category: AffectiveCategory::DarkCool,
        sort_key: oklch_hue(30, 32, 40),
        color_256: None,
    }
}

/// Warm light palette — cream paper with dark ink.
fn parchment() -> Palette {
    Palette {
        name: "Parchment",
        provenance: "",
        foreground: Color::Rgb(60, 50, 40),
        background: Color::Rgb(240, 230, 215),
        dimmed_foreground: Color::Rgb(170, 163, 152),
        accent_heading: Color::Rgb(120, 75, 40),
        accent_emphasis: Color::Rgb(70, 60, 50),
        accent_link: Color::Rgb(60, 95, 60),
        accent_code: Color::Rgb(100, 90, 80),
        category: AffectiveCategory::LightWarm,
        sort_key: oklch_hue(240, 230, 215),
        color_256: None,
    }
}

/// Dark warm palette — deep mahogany with amber headings.
fn hearthstone() -> Palette {
    Palette {
        name: "Hearthstone",
        provenance: "",
        foreground: Color::Rgb(230, 215, 195),
        background: Color::Rgb(35, 28, 24),
        dimmed_foreground: Color::Rgb(110, 100, 90),
        accent_heading: Color::Rgb(215, 155, 95),
        accent_emphasis: Color::Rgb(205, 195, 180),
        accent_link: Color::Rgb(155, 175, 140),
        accent_code: Color::Rgb(185, 170, 150),
        category: AffectiveCategory::DarkWarm,
        sort_key: oklch_hue(35, 28, 24),
        color_256: None,
    }
}

/// Cool dark palette — deep blue-purple with lavender headings.
fn moonstone() -> Palette {
    Palette {
        name: "Moonstone",
        provenance: "",
        foreground: Color::Rgb(210, 215, 225),
        background: Color::Rgb(30, 28, 38),
        dimmed_foreground: Color::Rgb(95, 97, 105),
        accent_heading: Color::Rgb(155, 145, 200),
        accent_emphasis: Color::Rgb(190, 195, 210),
        accent_link: Color::Rgb(135, 180, 185),
        accent_code: Color::Rgb(170, 170, 185),
        category: AffectiveCategory::DarkCool,
        sort_key: oklch_hue(30, 28, 38),
        color_256: None,
    }
}

/// Vivid dark palette — high-saturation magenta and cyan on deep dark.
fn neon_noir() -> Palette {
    Palette {
        name: "Neon Noir",
        provenance: "",
        foreground: Color::Rgb(225, 225, 235),
        background: Color::Rgb(22, 22, 28),
        dimmed_foreground: Color::Rgb(90, 90, 100),
        accent_heading: Color::Rgb(235, 110, 200),
        accent_emphasis: Color::Rgb(210, 210, 225),
        accent_link: Color::Rgb(90, 215, 215),
        accent_code: Color::Rgb(185, 185, 200),
        category: AffectiveCategory::DarkVivid,
        sort_key: oklch_hue(22, 22, 28),
        color_256: None,
    }
}

/// Vivid dark palette — vivid green and blue accents on dark green-tinged base.
fn aurora() -> Palette {
    Palette {
        name: "Aurora",
        provenance: "",
        foreground: Color::Rgb(220, 230, 225),
        background: Color::Rgb(20, 28, 25),
        dimmed_foreground: Color::Rgb(85, 100, 92),
        accent_heading: Color::Rgb(100, 220, 160),
        accent_emphasis: Color::Rgb(200, 215, 205),
        accent_link: Color::Rgb(130, 190, 220),
        accent_code: Color::Rgb(175, 195, 185),
        category: AffectiveCategory::DarkVivid,
        sort_key: oklch_hue(20, 28, 25),
        color_256: None,
    }
}

/// Warm light palette — golden-warm paper with sienna headings.
fn manuscript() -> Palette {
    Palette {
        name: "Manuscript",
        provenance: "",
        foreground: Color::Rgb(55, 45, 35),
        background: Color::Rgb(235, 225, 205),
        dimmed_foreground: Color::Rgb(165, 158, 145),
        accent_heading: Color::Rgb(130, 80, 25),
        accent_emphasis: Color::Rgb(65, 55, 45),
        accent_link: Color::Rgb(50, 90, 80),
        accent_code: Color::Rgb(95, 80, 65),
        category: AffectiveCategory::LightWarm,
        sort_key: oklch_hue(235, 225, 205),
        color_256: None,
    }
}

/// Cool light palette — blue-gray paper with steel blue headings.
fn glacier() -> Palette {
    Palette {
        name: "Glacier",
        provenance: "",
        foreground: Color::Rgb(40, 45, 55),
        background: Color::Rgb(225, 230, 240),
        dimmed_foreground: Color::Rgb(150, 155, 165),
        accent_heading: Color::Rgb(55, 85, 130),
        accent_emphasis: Color::Rgb(50, 55, 65),
        accent_link: Color::Rgb(45, 100, 110),
        accent_code: Color::Rgb(75, 80, 95),
        category: AffectiveCategory::LightCool,
        sort_key: oklch_hue(225, 230, 240),
        color_256: None,
    }
}

/// Vivid light palette — bright with vivid crimson and teal accents.
fn daybreak() -> Palette {
    Palette {
        name: "Daybreak",
        provenance: "",
        foreground: Color::Rgb(45, 40, 35),
        background: Color::Rgb(240, 235, 230),
        dimmed_foreground: Color::Rgb(160, 155, 148),
        accent_heading: Color::Rgb(170, 50, 70),
        accent_emphasis: Color::Rgb(55, 50, 45),
        accent_link: Color::Rgb(25, 105, 115),
        accent_code: Color::Rgb(85, 75, 65),
        category: AffectiveCategory::LightVivid,
        sort_key: oklch_hue(240, 235, 230),
        color_256: None,
    }
}
