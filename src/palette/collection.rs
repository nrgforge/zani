//! Palette collection: the 40 curated palettes named after PNW flora (ADR-015).
//! This file is separated from the palette logic so palette data is easy to
//! alter and review independently.
//!
//! Organization: 8 Affective Categories × 5 palettes each = 40 total.
//! Within each category, palettes are spread across OKLCH hue space
//! for maximum perceptual diversity.

use ratatui::style::Color;
use super::{AffectiveCategory, Palette, oklch_hue};

/// Returns all built-in palettes.
pub(super) fn all_palettes() -> Vec<Palette> {
    vec![
        // Dark — Warm
        default_palette(),
        chinquapin(),
        red_cedar(),
        chanterelle(),
        madrone(),
        // Dark — Cool
        sitka(),
        oakmoss(),
        oregon_grape(),
        elderberry(),
        witchs_hair(),
        // Dark — Vivid
        fly_agaric(),
        lobaria(),
        jack_o_lantern(),
        violet_cort(),
        salal(),
        // Dark — Muted
        usnea(),
        alder(),
        douglas_fir(),
        sagebrush(),
        map_lichen(),
        // Light — Warm
        oatgrass(),
        oregon_sunshine(),
        white_oak(),
        ponderosa(),
        balsamroot(),
        // Light — Cool
        cascade_aster(),
        pearly_everlasting(),
        partridgefoot(),
        lupine(),
        phlox(),
        // Light — Vivid
        paintbrush(),
        columbine(),
        tiger_lily(),
        farewell(),
        camas(),
        // Light — Muted
        sword_fern(),
        oceanspray(),
        goatsbeard(),
        fringecup(),
        reindeer_lichen(),
    ]
}

// ============================================================
// Dark — Warm: intimate, sheltering
// ============================================================

/// Default palette. Smooth bark from mahogany to cinnamon.
pub(super) fn default_palette() -> Palette {
    Palette {
        name: "Manzanita",
        provenance: "Arctostaphylos — smooth bark from mahogany to cinnamon to deep red, thriving in dry fire-adapted chaparral and oak savannah from southern Oregon through California.",
        foreground: Color::Rgb(225, 215, 200),
        background: Color::Rgb(42, 30, 28),
        dimmed_foreground: Color::Rgb(110, 100, 90),
        accent_heading: Color::Rgb(210, 145, 110),
        accent_emphasis: Color::Rgb(200, 192, 178),
        accent_link: Color::Rgb(155, 185, 165),
        accent_code: Color::Rgb(180, 170, 155),
        category: AffectiveCategory::DarkWarm,
        sort_key: oklch_hue(42, 30, 28),
        color_256: None,
    }
}

/// Evergreen broadleaf with dense golden leaf scales.
fn chinquapin() -> Palette {
    Palette {
        name: "Chinquapin",
        provenance: "Chrysolepis chrysophylla — evergreen broadleaf of the Siskiyou and Coast ranges, leaf undersides covered in dense golden scales on dry rocky slopes.",
        foreground: Color::Rgb(225, 218, 200),
        background: Color::Rgb(40, 32, 22),
        dimmed_foreground: Color::Rgb(108, 103, 88),
        accent_heading: Color::Rgb(205, 178, 110),
        accent_emphasis: Color::Rgb(200, 195, 180),
        accent_link: Color::Rgb(148, 178, 175),
        accent_code: Color::Rgb(178, 172, 155),
        category: AffectiveCategory::DarkWarm,
        sort_key: oklch_hue(40, 32, 22),
        color_256: None,
    }
}

/// Heartwood in warm amber to cinnamon brown.
fn red_cedar() -> Palette {
    Palette {
        name: "Red Cedar",
        provenance: "Thuja plicata — heartwood in warm amber to cinnamon brown, aromatic and rot-resistant, dominant in moist lowland forests from Alaska through the Oregon Cascades.",
        foreground: Color::Rgb(222, 215, 200),
        background: Color::Rgb(45, 28, 20),
        dimmed_foreground: Color::Rgb(108, 102, 90),
        accent_heading: Color::Rgb(200, 162, 105),
        accent_emphasis: Color::Rgb(198, 192, 178),
        accent_link: Color::Rgb(150, 180, 168),
        accent_code: Color::Rgb(178, 170, 155),
        category: AffectiveCategory::DarkWarm,
        sort_key: oklch_hue(45, 28, 20),
        color_256: None,
    }
}

/// Pacific golden chanterelle with warm apricot caps.
fn chanterelle() -> Palette {
    Palette {
        name: "Chanterelle",
        provenance: "Cantharellus formosus — Pacific golden chanterelle with warm apricot-orange caps, mycorrhizal with Douglas fir in Coast Range and Cascade forests.",
        foreground: Color::Rgb(228, 218, 200),
        background: Color::Rgb(34, 38, 20),
        dimmed_foreground: Color::Rgb(112, 102, 88),
        accent_heading: Color::Rgb(218, 158, 88),
        accent_emphasis: Color::Rgb(202, 195, 178),
        accent_link: Color::Rgb(148, 182, 172),
        accent_code: Color::Rgb(182, 172, 155),
        category: AffectiveCategory::DarkWarm,
        sort_key: oklch_hue(34, 38, 20),
        color_256: None,
    }
}

/// Bark peels to reveal smooth terra-cotta wood.
fn madrone() -> Palette {
    Palette {
        name: "Madrone",
        provenance: "Arbutus menziesii — bark peels to reveal smooth terra-cotta wood beneath, thriving in dry fire-adapted oak savannahs of the Rogue Valley and on well-drained slopes to British Columbia.",
        foreground: Color::Rgb(222, 212, 205),
        background: Color::Rgb(48, 26, 34),
        dimmed_foreground: Color::Rgb(108, 100, 96),
        accent_heading: Color::Rgb(198, 140, 118),
        accent_emphasis: Color::Rgb(198, 190, 182),
        accent_link: Color::Rgb(150, 180, 172),
        accent_code: Color::Rgb(178, 168, 158),
        category: AffectiveCategory::DarkWarm,
        sort_key: oklch_hue(48, 26, 34),
        color_256: None,
    }
}

// ============================================================
// Dark — Cool: intellectual, deep
// ============================================================

/// Massive coastal conifer with dense blue-green needles.
fn sitka() -> Palette {
    Palette {
        name: "Sitka",
        provenance: "Picea sitchensis — massive coastal conifer with dense blue-green needles and furrowed bark, dominating fog-belt forests from Alaska to the northern Oregon coast.",
        foreground: Color::Rgb(208, 218, 222),
        background: Color::Rgb(24, 32, 36),
        dimmed_foreground: Color::Rgb(88, 98, 102),
        accent_heading: Color::Rgb(120, 178, 195),
        accent_emphasis: Color::Rgb(188, 198, 202),
        accent_link: Color::Rgb(180, 165, 142),
        accent_code: Color::Rgb(165, 175, 178),
        category: AffectiveCategory::DarkCool,
        sort_key: oklch_hue(24, 32, 36),
        color_256: None,
    }
}

/// Fruticose lichen draping Oregon white oak branches.
fn oakmoss() -> Palette {
    Palette {
        name: "Oakmoss",
        provenance: "Evernia prunastri — fruticose lichen draping Oregon white oak branches in the Willamette and Rogue valleys, thallus muted teal-green above and pale beneath.",
        foreground: Color::Rgb(208, 218, 215),
        background: Color::Rgb(26, 34, 32),
        dimmed_foreground: Color::Rgb(90, 100, 98),
        accent_heading: Color::Rgb(125, 180, 168),
        accent_emphasis: Color::Rgb(185, 198, 195),
        accent_link: Color::Rgb(178, 162, 145),
        accent_code: Color::Rgb(165, 175, 172),
        category: AffectiveCategory::DarkCool,
        sort_key: oklch_hue(26, 34, 32),
        color_256: None,
    }
}

/// Clusters of blue-black berries with heavy bloom.
fn oregon_grape() -> Palette {
    Palette {
        name: "Oregon Grape",
        provenance: "Mahonia aquifolium — state flower of Oregon, clusters of blue-black berries with heavy bloom on holly-like evergreen shrubs in dry open woodland and forest edges.",
        foreground: Color::Rgb(210, 212, 225),
        background: Color::Rgb(28, 28, 40),
        dimmed_foreground: Color::Rgb(92, 94, 108),
        accent_heading: Color::Rgb(138, 148, 210),
        accent_emphasis: Color::Rgb(188, 190, 205),
        accent_link: Color::Rgb(172, 178, 155),
        accent_code: Color::Rgb(168, 170, 185),
        category: AffectiveCategory::DarkCool,
        sort_key: oklch_hue(28, 28, 40),
        color_256: None,
    }
}

/// Powder-blue berries with a waxy bloom.
fn elderberry() -> Palette {
    Palette {
        name: "Elderberry",
        provenance: "Sambucus cerulea — deciduous shrub producing heavy clusters of powder-blue berries with waxy bloom along stream banks and roadsides through the Rogue and Umpqua valleys.",
        foreground: Color::Rgb(215, 212, 228),
        background: Color::Rgb(32, 28, 40),
        dimmed_foreground: Color::Rgb(98, 94, 110),
        accent_heading: Color::Rgb(155, 142, 202),
        accent_emphasis: Color::Rgb(195, 192, 208),
        accent_link: Color::Rgb(148, 180, 170),
        accent_code: Color::Rgb(170, 168, 188),
        category: AffectiveCategory::DarkCool,
        sort_key: oklch_hue(32, 28, 40),
        color_256: None,
    }
}

/// Pendant lichen hanging in dark curtains from old-growth conifers.
fn witchs_hair() -> Palette {
    Palette {
        name: "Witch's Hair",
        provenance: "Alectoria sarmentosa — pendant lichen hanging in dark curtains from conifer branches in old-growth forests of the Cascades and Coast Range, thallus olive-black to deep greenish-brown.",
        foreground: Color::Rgb(212, 218, 208),
        background: Color::Rgb(28, 32, 26),
        dimmed_foreground: Color::Rgb(92, 100, 88),
        accent_heading: Color::Rgb(142, 178, 132),
        accent_emphasis: Color::Rgb(190, 198, 188),
        accent_link: Color::Rgb(168, 160, 185),
        accent_code: Color::Rgb(168, 175, 165),
        category: AffectiveCategory::DarkCool,
        sort_key: oklch_hue(28, 32, 26),
        color_256: None,
    }
}

// ============================================================
// Dark — Vivid: electric, energetic
// ============================================================

/// Iconic red-capped mushroom with white warts.
fn fly_agaric() -> Palette {
    Palette {
        name: "Fly Agaric",
        provenance: "Amanita muscaria — iconic red-capped mushroom with white warts, mycorrhizal with birch, pine, and spruce, fruiting in fall across Pacific Northwest forests from sea level to subalpine.",
        foreground: Color::Rgb(230, 222, 220),
        background: Color::Rgb(28, 18, 18),
        dimmed_foreground: Color::Rgb(98, 88, 86),
        accent_heading: Color::Rgb(230, 90, 80),
        accent_emphasis: Color::Rgb(210, 205, 200),
        accent_link: Color::Rgb(95, 200, 210),
        accent_code: Color::Rgb(188, 182, 178),
        category: AffectiveCategory::DarkVivid,
        sort_key: oklch_hue(28, 18, 18),
        color_256: None,
    }
}

/// Large foliose lichen, saturated bright green when wet.
fn lobaria() -> Palette {
    Palette {
        name: "Lobaria",
        provenance: "Lobaria pulmonaria — large foliose lichen with ridged lettuce-like thallus in saturated bright green when wet, an old-growth indicator on hardwoods and conifers in moist forests.",
        foreground: Color::Rgb(218, 230, 220),
        background: Color::Rgb(18, 26, 20),
        dimmed_foreground: Color::Rgb(85, 100, 88),
        accent_heading: Color::Rgb(85, 220, 130),
        accent_emphasis: Color::Rgb(200, 215, 205),
        accent_link: Color::Rgb(180, 150, 215),
        accent_code: Color::Rgb(172, 188, 178),
        category: AffectiveCategory::DarkVivid,
        sort_key: oklch_hue(18, 26, 20),
        color_256: None,
    }
}

/// Bioluminescent orange mushroom, gills glow faintly in darkness.
fn jack_o_lantern() -> Palette {
    Palette {
        name: "Jack-o'-Lantern",
        provenance: "Omphalotus olearius — bioluminescent orange mushroom growing in dense clusters at the base of hardwoods, gills glow faintly in darkness, occasional in Oregon's Coast Range and Siskiyou foothills.",
        foreground: Color::Rgb(228, 222, 215),
        background: Color::Rgb(26, 20, 16),
        dimmed_foreground: Color::Rgb(98, 88, 82),
        accent_heading: Color::Rgb(235, 158, 50),
        accent_emphasis: Color::Rgb(210, 205, 198),
        accent_link: Color::Rgb(100, 188, 218),
        accent_code: Color::Rgb(185, 180, 172),
        category: AffectiveCategory::DarkVivid,
        sort_key: oklch_hue(26, 20, 16),
        color_256: None,
    }
}

/// Entirely deep violet mushroom — cap, gills, and stipe.
fn violet_cort() -> Palette {
    Palette {
        name: "Violet Cort",
        provenance: "Cortinarius violaceus — entirely deep violet mushroom, cap, gills, and stipe, mycorrhizal with conifers in mature forests of the Cascades and Olympics, fruiting in fall.",
        foreground: Color::Rgb(222, 218, 232),
        background: Color::Rgb(24, 18, 30),
        dimmed_foreground: Color::Rgb(90, 86, 102),
        accent_heading: Color::Rgb(185, 110, 232),
        accent_emphasis: Color::Rgb(205, 200, 218),
        accent_link: Color::Rgb(118, 210, 172),
        accent_code: Color::Rgb(180, 175, 195),
        category: AffectiveCategory::DarkVivid,
        sort_key: oklch_hue(24, 18, 30),
        color_256: None,
    }
}

/// Dense evergreen shrub; berries ripen to deep saturated blue-purple.
fn salal() -> Palette {
    Palette {
        name: "Salal",
        provenance: "Gaultheria shallon — dense evergreen understory shrub from Alaska to northern California, berries ripen to deep blue-purple so saturated they appear nearly black.",
        foreground: Color::Rgb(220, 222, 235),
        background: Color::Rgb(20, 20, 32),
        dimmed_foreground: Color::Rgb(88, 88, 105),
        accent_heading: Color::Rgb(138, 118, 235),
        accent_emphasis: Color::Rgb(200, 205, 222),
        accent_link: Color::Rgb(212, 172, 100),
        accent_code: Color::Rgb(178, 180, 198),
        category: AffectiveCategory::DarkVivid,
        sort_key: oklch_hue(20, 20, 32),
        color_256: None,
    }
}

// ============================================================
// Dark — Muted: contemplative, subdued
// ============================================================

/// Pendant gray-green lichen on conifer branches.
fn usnea() -> Palette {
    Palette {
        name: "Usnea",
        provenance: "Usnea — pendant gray-green lichen on conifer branches throughout Pacific Northwest forests, sensitive to air quality, thallus silvery-green with a distinctive central cord.",
        foreground: Color::Rgb(210, 214, 212),
        background: Color::Rgb(36, 38, 38),
        dimmed_foreground: Color::Rgb(100, 104, 102),
        accent_heading: Color::Rgb(158, 175, 168),
        accent_emphasis: Color::Rgb(190, 194, 192),
        accent_link: Color::Rgb(175, 162, 155),
        accent_code: Color::Rgb(170, 174, 172),
        category: AffectiveCategory::DarkMuted,
        sort_key: oklch_hue(36, 38, 38),
        color_256: None,
    }
}

/// Smooth pale gray bark mottled with white lichen.
fn alder() -> Palette {
    Palette {
        name: "Alder",
        provenance: "Alnus rubra — smooth pale gray bark mottled with white lichen patches, pioneer species colonizing disturbed riparian areas throughout western Oregon and Washington.",
        foreground: Color::Rgb(215, 212, 208),
        background: Color::Rgb(38, 36, 34),
        dimmed_foreground: Color::Rgb(104, 100, 96),
        accent_heading: Color::Rgb(172, 160, 148),
        accent_emphasis: Color::Rgb(195, 192, 188),
        accent_link: Color::Rgb(150, 168, 175),
        accent_code: Color::Rgb(172, 168, 162),
        category: AffectiveCategory::DarkMuted,
        sort_key: oklch_hue(38, 36, 34),
        color_256: None,
    }
}

/// Deeply furrowed bark in muted gray-brown on old growth.
fn douglas_fir() -> Palette {
    Palette {
        name: "Douglas Fir",
        provenance: "Pseudotsuga menziesii — deeply furrowed bark on old growth develops thick corky ridges in muted gray-brown, the dominant timber species from British Columbia through the Siskiyous.",
        foreground: Color::Rgb(212, 210, 205),
        background: Color::Rgb(36, 34, 30),
        dimmed_foreground: Color::Rgb(102, 98, 92),
        accent_heading: Color::Rgb(170, 158, 138),
        accent_emphasis: Color::Rgb(192, 190, 185),
        accent_link: Color::Rgb(148, 170, 172),
        accent_code: Color::Rgb(170, 168, 160),
        category: AffectiveCategory::DarkMuted,
        sort_key: oklch_hue(36, 34, 30),
        color_256: None,
    }
}

/// Aromatic shrub with silver-green foliage and shredding gray bark.
fn sagebrush() -> Palette {
    Palette {
        name: "Sagebrush",
        provenance: "Artemisia tridentata — aromatic shrub with silver-green foliage and shredding gray bark, dominant in high desert east of the Cascades from British Columbia through central and southern Oregon.",
        foreground: Color::Rgb(210, 215, 210),
        background: Color::Rgb(34, 38, 34),
        dimmed_foreground: Color::Rgb(98, 105, 98),
        accent_heading: Color::Rgb(152, 172, 148),
        accent_emphasis: Color::Rgb(190, 196, 190),
        accent_link: Color::Rgb(170, 158, 172),
        accent_code: Color::Rgb(168, 174, 168),
        category: AffectiveCategory::DarkMuted,
        sort_key: oklch_hue(34, 38, 34),
        color_256: None,
    }
}

/// Crustose lichen forming chartreuse-yellow patches on exposed rock.
fn map_lichen() -> Palette {
    Palette {
        name: "Map Lichen",
        provenance: "Rhizocarpon geographicum — crustose lichen forming chartreuse-yellow patches bordered by black prothallus lines on exposed rock, common on basalt and granite throughout the Cascades and Siskiyous.",
        foreground: Color::Rgb(214, 214, 205),
        background: Color::Rgb(36, 36, 30),
        dimmed_foreground: Color::Rgb(102, 102, 92),
        accent_heading: Color::Rgb(175, 172, 130),
        accent_emphasis: Color::Rgb(194, 194, 185),
        accent_link: Color::Rgb(155, 162, 178),
        accent_code: Color::Rgb(172, 172, 162),
        category: AffectiveCategory::DarkMuted,
        sort_key: oklch_hue(36, 36, 30),
        color_256: None,
    }
}

// ============================================================
// Light — Warm: gentle, morning
// ============================================================

/// Native bunchgrass turning golden straw by midsummer.
fn oatgrass() -> Palette {
    Palette {
        name: "Oatgrass",
        provenance: "Danthonia californica — native bunchgrass of dry prairies and oak savannah, turning golden straw by midsummer, characteristic of remnant grasslands in the Rogue and Willamette valleys.",
        foreground: Color::Rgb(55, 48, 35),
        background: Color::Rgb(245, 240, 210),
        dimmed_foreground: Color::Rgb(168, 162, 148),
        accent_heading: Color::Rgb(125, 85, 30),
        accent_emphasis: Color::Rgb(65, 58, 45),
        accent_link: Color::Rgb(45, 95, 75),
        accent_code: Color::Rgb(98, 88, 68),
        category: AffectiveCategory::LightWarm,
        sort_key: oklch_hue(245, 240, 210),
        color_256: None,
    }
}

/// Woolly perennial with bright yellow daisy-like flower heads.
fn oregon_sunshine() -> Palette {
    Palette {
        name: "Oregon Sunshine",
        provenance: "Eriophyllum lanatum — woolly perennial with bright yellow daisy-like flower heads on dry rocky slopes and road cuts from the Siskiyous through the Columbia Gorge, blooming May through July.",
        foreground: Color::Rgb(52, 45, 32),
        background: Color::Rgb(230, 242, 205),
        dimmed_foreground: Color::Rgb(170, 164, 148),
        accent_heading: Color::Rgb(135, 90, 15),
        accent_emphasis: Color::Rgb(62, 55, 42),
        accent_link: Color::Rgb(35, 88, 82),
        accent_code: Color::Rgb(92, 82, 62),
        category: AffectiveCategory::LightWarm,
        sort_key: oklch_hue(230, 242, 205),
        color_256: None,
    }
}

/// Deciduous hardwood with pale fissured bark in light gray to buff tones.
fn white_oak() -> Palette {
    Palette {
        name: "White Oak",
        provenance: "Quercus garryana — deciduous hardwood with pale fissured bark in light gray to buff, defining the oak savannah ecosystem of the Rogue, Umpqua, and Willamette valleys.",
        foreground: Color::Rgb(50, 45, 38),
        background: Color::Rgb(245, 225, 215),
        dimmed_foreground: Color::Rgb(164, 160, 150),
        accent_heading: Color::Rgb(115, 80, 48),
        accent_emphasis: Color::Rgb(60, 55, 48),
        accent_link: Color::Rgb(48, 90, 68),
        accent_code: Color::Rgb(90, 82, 72),
        category: AffectiveCategory::LightWarm,
        sort_key: oklch_hue(245, 225, 215),
        color_256: None,
    }
}

/// Mature bark in warm butterscotch plates with vanilla scent.
fn ponderosa() -> Palette {
    Palette {
        name: "Ponderosa",
        provenance: "Pinus ponderosa — mature bark breaks into warm butterscotch and vanilla plates with faint vanilla scent, widespread on dry slopes from British Columbia through the eastern Siskiyous.",
        foreground: Color::Rgb(52, 42, 30),
        background: Color::Rgb(250, 225, 200),
        dimmed_foreground: Color::Rgb(168, 160, 142),
        accent_heading: Color::Rgb(130, 78, 22),
        accent_emphasis: Color::Rgb(62, 52, 40),
        accent_link: Color::Rgb(40, 92, 78),
        accent_code: Color::Rgb(92, 80, 58),
        category: AffectiveCategory::LightWarm,
        sort_key: oklch_hue(250, 225, 200),
        color_256: None,
    }
}

/// Large sunflower-like blooms in warm amber-gold.
fn balsamroot() -> Palette {
    Palette {
        name: "Balsamroot",
        provenance: "Balsamorhiza sagittata — large sunflower-like blooms in warm amber-gold on stout stems above arrow-shaped basal leaves, abundant on dry slopes from the Rogue Valley through eastern Washington.",
        foreground: Color::Rgb(55, 45, 30),
        background: Color::Rgb(218, 242, 210),
        dimmed_foreground: Color::Rgb(170, 162, 140),
        accent_heading: Color::Rgb(138, 88, 12),
        accent_emphasis: Color::Rgb(65, 55, 40),
        accent_link: Color::Rgb(38, 90, 82),
        accent_code: Color::Rgb(95, 82, 55),
        category: AffectiveCategory::LightWarm,
        sort_key: oklch_hue(218, 242, 210),
        color_256: None,
    }
}

// ============================================================
// Light — Cool: crisp, alpine
// ============================================================

/// Lavender to pale violet ray flowers in subalpine meadows.
fn cascade_aster() -> Palette {
    Palette {
        name: "Cascade Aster",
        provenance: "Eucephalus ledophyllus — lavender to pale violet ray flowers with yellow disk centers, blooming in subalpine meadows and open clearings throughout the Cascade Range.",
        foreground: Color::Rgb(42, 40, 52),
        background: Color::Rgb(232, 228, 240),
        dimmed_foreground: Color::Rgb(155, 152, 168),
        accent_heading: Color::Rgb(72, 55, 120),
        accent_emphasis: Color::Rgb(52, 50, 62),
        accent_link: Color::Rgb(45, 95, 90),
        accent_code: Color::Rgb(78, 75, 95),
        category: AffectiveCategory::LightCool,
        sort_key: oklch_hue(232, 228, 240),
        color_256: None,
    }
}

/// Papery white bracts persistent through winter.
fn pearly_everlasting() -> Palette {
    Palette {
        name: "Pearly Everlasting",
        provenance: "Anaphalis margaritacea — papery white bracts surrounding tiny yellow disk flowers, persistent through winter, colonizing road cuts, gravel bars, and burned slopes throughout the region.",
        foreground: Color::Rgb(40, 42, 48),
        background: Color::Rgb(232, 238, 240),
        dimmed_foreground: Color::Rgb(155, 158, 162),
        accent_heading: Color::Rgb(55, 75, 112),
        accent_emphasis: Color::Rgb(50, 52, 58),
        accent_link: Color::Rgb(60, 90, 72),
        accent_code: Color::Rgb(72, 78, 88),
        category: AffectiveCategory::LightCool,
        sort_key: oklch_hue(232, 238, 240),
        color_256: None,
    }
}

/// Low mat-forming alpine plant with finely divided fernlike leaves.
fn partridgefoot() -> Palette {
    Palette {
        name: "Partridgefoot",
        provenance: "Luetkea pectinata — low mat-forming alpine plant with finely divided fernlike leaves in cool gray-green and small creamy racemes, common in snowmelt zones in the Cascades and Olympics.",
        foreground: Color::Rgb(38, 48, 44),
        background: Color::Rgb(228, 236, 232),
        dimmed_foreground: Color::Rgb(150, 162, 158),
        accent_heading: Color::Rgb(40, 88, 78),
        accent_emphasis: Color::Rgb(48, 58, 54),
        accent_link: Color::Rgb(82, 72, 105),
        accent_code: Color::Rgb(72, 85, 80),
        category: AffectiveCategory::LightCool,
        sort_key: oklch_hue(228, 236, 232),
        color_256: None,
    }
}

/// Spires of blue to blue-violet pea flowers in subalpine meadows.
fn lupine() -> Palette {
    Palette {
        name: "Lupine",
        provenance: "Lupinus latifolius — spires of blue to blue-violet pea flowers in subalpine meadows and open slopes throughout the Cascades, palmate leaves with silvery sheen from fine hairs.",
        foreground: Color::Rgb(38, 42, 55),
        background: Color::Rgb(218, 228, 245),
        dimmed_foreground: Color::Rgb(148, 155, 168),
        accent_heading: Color::Rgb(45, 72, 130),
        accent_emphasis: Color::Rgb(48, 52, 65),
        accent_link: Color::Rgb(82, 88, 58),
        accent_code: Color::Rgb(70, 78, 100),
        category: AffectiveCategory::LightCool,
        sort_key: oklch_hue(218, 228, 245),
        color_256: None,
    }
}

/// Low cushion plant of rocky alpine ridges, pale pink to near-white flowers.
fn phlox() -> Palette {
    Palette {
        name: "Phlox",
        provenance: "Phlox diffusa — low cushion plant of rocky alpine ridges and pumice flats, covered in pale pink to near-white flowers from the Siskiyou crest through the high Cascades and Olympics.",
        foreground: Color::Rgb(48, 38, 42),
        background: Color::Rgb(238, 228, 232),
        dimmed_foreground: Color::Rgb(162, 152, 158),
        accent_heading: Color::Rgb(120, 50, 72),
        accent_emphasis: Color::Rgb(58, 48, 52),
        accent_link: Color::Rgb(40, 92, 95),
        accent_code: Color::Rgb(88, 75, 80),
        category: AffectiveCategory::LightCool,
        sort_key: oklch_hue(238, 228, 232),
        color_256: None,
    }
}

// ============================================================
// Light — Vivid: bright, solar
// ============================================================

/// Vivid scarlet to red-orange bracts in meadows and open slopes.
fn paintbrush() -> Palette {
    Palette {
        name: "Paintbrush",
        provenance: "Castilleja miniata — vivid scarlet to red-orange bracts surrounding inconspicuous flowers, hemiparasitic on neighboring roots, common in meadows from the Rogue Valley to subalpine.",
        foreground: Color::Rgb(45, 38, 35),
        background: Color::Rgb(248, 230, 228),
        dimmed_foreground: Color::Rgb(162, 155, 150),
        accent_heading: Color::Rgb(175, 40, 48),
        accent_emphasis: Color::Rgb(55, 48, 45),
        accent_link: Color::Rgb(15, 100, 110),
        accent_code: Color::Rgb(88, 78, 68),
        category: AffectiveCategory::LightVivid,
        sort_key: oklch_hue(248, 230, 228),
        color_256: None,
    }
}

/// Nodding flowers with red-orange sepals and yellow petals.
fn columbine() -> Palette {
    Palette {
        name: "Columbine",
        provenance: "Aquilegia formosa — nodding flowers with red-orange sepals and yellow petals, blooming in moist seeps and open woodland edges from British Columbia through the Siskiyous.",
        foreground: Color::Rgb(48, 40, 32),
        background: Color::Rgb(240, 236, 228),
        dimmed_foreground: Color::Rgb(162, 158, 148),
        accent_heading: Color::Rgb(170, 62, 25),
        accent_emphasis: Color::Rgb(58, 50, 42),
        accent_link: Color::Rgb(18, 95, 112),
        accent_code: Color::Rgb(88, 78, 62),
        category: AffectiveCategory::LightVivid,
        sort_key: oklch_hue(240, 236, 228),
        color_256: None,
    }
}

/// Nodding orange flowers with recurved tepals spotted in maroon.
fn tiger_lily() -> Palette {
    Palette {
        name: "Tiger Lily",
        provenance: "Lilium columbianum — nodding orange flowers with recurved tepals spotted in maroon, rising on slender stems above woodland meadows from British Columbia to northern California.",
        foreground: Color::Rgb(48, 42, 30),
        background: Color::Rgb(250, 230, 210),
        dimmed_foreground: Color::Rgb(165, 160, 145),
        accent_heading: Color::Rgb(168, 78, 10),
        accent_emphasis: Color::Rgb(58, 52, 40),
        accent_link: Color::Rgb(15, 90, 118),
        accent_code: Color::Rgb(90, 82, 58),
        category: AffectiveCategory::LightVivid,
        sort_key: oklch_hue(250, 230, 210),
        color_256: None,
    }
}

/// Satiny four-petaled flowers in vivid pink to magenta.
fn farewell() -> Palette {
    Palette {
        name: "Farewell",
        provenance: "Clarkia amoena — satiny four-petaled flowers in vivid pink to magenta, blooming in dry grasslands and road cuts from the Rogue Valley north as grasses begin to cure.",
        foreground: Color::Rgb(48, 38, 42),
        background: Color::Rgb(240, 232, 235),
        dimmed_foreground: Color::Rgb(162, 152, 158),
        accent_heading: Color::Rgb(168, 35, 85),
        accent_emphasis: Color::Rgb(58, 48, 52),
        accent_link: Color::Rgb(10, 100, 102),
        accent_code: Color::Rgb(88, 75, 78),
        category: AffectiveCategory::LightVivid,
        sort_key: oklch_hue(240, 232, 235),
        color_256: None,
    }
}

/// Spikes of bright blue-violet star-shaped flowers carpeting wet prairies.
fn camas() -> Palette {
    Palette {
        name: "Camas",
        provenance: "Camassia quamash — spikes of bright blue-violet star-shaped flowers carpeting wet prairies in April and May, historically a staple root crop, in remnant wet meadows of the Rogue and Willamette valleys.",
        foreground: Color::Rgb(38, 40, 52),
        background: Color::Rgb(232, 235, 242),
        dimmed_foreground: Color::Rgb(152, 155, 168),
        accent_heading: Color::Rgb(50, 55, 155),
        accent_emphasis: Color::Rgb(48, 50, 62),
        accent_link: Color::Rgb(112, 68, 30),
        accent_code: Color::Rgb(72, 75, 95),
        category: AffectiveCategory::LightVivid,
        sort_key: oklch_hue(232, 235, 242),
        color_256: None,
    }
}

// ============================================================
// Light — Muted: airy, soft
// ============================================================

/// Iconic PNW understory fern with leathery fronds.
fn sword_fern() -> Palette {
    Palette {
        name: "Sword Fern",
        provenance: "Polystichum munitum — evergreen fern with tough leathery fronds forming dense understory carpets in moist conifer forests from Alaska to southern California.",
        foreground: Color::Rgb(40, 50, 42),
        background: Color::Rgb(228, 236, 230),
        dimmed_foreground: Color::Rgb(152, 164, 155),
        accent_heading: Color::Rgb(50, 85, 55),
        accent_emphasis: Color::Rgb(50, 60, 52),
        accent_link: Color::Rgb(85, 72, 92),
        accent_code: Color::Rgb(72, 85, 75),
        category: AffectiveCategory::LightMuted,
        sort_key: oklch_hue(228, 236, 230),
        color_256: None,
    }
}

/// Arching shrub bearing cascading panicles of tiny creamy-white flowers.
fn oceanspray() -> Palette {
    Palette {
        name: "Oceanspray",
        provenance: "Holodiscus discolor — arching shrub bearing cascading panicles of tiny creamy-white flowers that dry to soft parchment brown, common on dry hillsides from the Siskiyous to British Columbia.",
        foreground: Color::Rgb(48, 45, 38),
        background: Color::Rgb(238, 236, 230),
        dimmed_foreground: Color::Rgb(162, 160, 152),
        accent_heading: Color::Rgb(100, 82, 60),
        accent_emphasis: Color::Rgb(58, 55, 48),
        accent_link: Color::Rgb(55, 85, 90),
        accent_code: Color::Rgb(85, 80, 72),
        category: AffectiveCategory::LightMuted,
        sort_key: oklch_hue(238, 236, 230),
        color_256: None,
    }
}

/// Tall woodland perennial with airy plumes of tiny white flowers.
fn goatsbeard() -> Palette {
    Palette {
        name: "Goatsbeard",
        provenance: "Aruncus dioicus — tall woodland perennial with airy plumes of tiny white flowers above compound leaves, found in moist shaded slopes and stream banks throughout western Oregon and Washington.",
        foreground: Color::Rgb(42, 44, 42),
        background: Color::Rgb(234, 240, 230),
        dimmed_foreground: Color::Rgb(158, 162, 158),
        accent_heading: Color::Rgb(62, 78, 85),
        accent_emphasis: Color::Rgb(52, 54, 52),
        accent_link: Color::Rgb(85, 75, 65),
        accent_code: Color::Rgb(75, 80, 78),
        category: AffectiveCategory::LightMuted,
        sort_key: oklch_hue(234, 240, 230),
        color_256: None,
    }
}

/// Delicate racemes of small bell-shaped flowers aging to soft pink.
fn fringecup() -> Palette {
    Palette {
        name: "Fringecup",
        provenance: "Tellima grandiflora — delicate racemes of small bell-shaped flowers opening white then aging to soft pink, above scalloped basal leaves in moist forest understory throughout the Pacific Northwest.",
        foreground: Color::Rgb(48, 40, 42),
        background: Color::Rgb(236, 230, 232),
        dimmed_foreground: Color::Rgb(162, 155, 158),
        accent_heading: Color::Rgb(108, 62, 72),
        accent_emphasis: Color::Rgb(58, 50, 52),
        accent_link: Color::Rgb(52, 85, 82),
        accent_code: Color::Rgb(85, 75, 78),
        category: AffectiveCategory::LightMuted,
        sort_key: oklch_hue(236, 230, 232),
        color_256: None,
    }
}

/// Fruticose lichen forming pale silvery-green cushions on thin soils.
fn reindeer_lichen() -> Palette {
    Palette {
        name: "Reindeer Lichen",
        provenance: "Cladonia rangiferina — fruticose lichen forming pale silvery-green to gray cushions on thin soils and rotting wood in open conifer forests, especially on drier east-slope Cascades.",
        foreground: Color::Rgb(42, 48, 42),
        background: Color::Rgb(225, 232, 238),
        dimmed_foreground: Color::Rgb(155, 162, 152),
        accent_heading: Color::Rgb(58, 82, 58),
        accent_emphasis: Color::Rgb(52, 58, 52),
        accent_link: Color::Rgb(80, 72, 90),
        accent_code: Color::Rgb(75, 82, 72),
        category: AffectiveCategory::LightMuted,
        sort_key: oklch_hue(225, 232, 238),
        color_256: None,
    }
}
