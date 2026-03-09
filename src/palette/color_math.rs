//! Color science functions: OKLCH, CIELAB, CIEDE2000.
//! Pure math operating on numeric inputs — no ratatui dependency.

/// Linearize an sRGB channel value (inverse gamma).
pub(super) fn linearize(value: f64) -> f64 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

/// Compute the OKLCH hue angle (in degrees) for an sRGB color.
/// Used as the Perceptual Sort Order key within an Affective Category (ADR-009).
pub(crate) fn oklch_hue(r: u8, g: u8, b: u8) -> f64 {
    let lr = linearize(r as f64 / 255.0);
    let lg = linearize(g as f64 / 255.0);
    let lb = linearize(b as f64 / 255.0);

    let l = 0.4122214708 * lr + 0.5363325363 * lg + 0.0514459929 * lb;
    let m = 0.2119034982 * lr + 0.6806995451 * lg + 0.1073969566 * lb;
    let s = 0.0883024619 * lr + 0.2817188376 * lg + 0.6299787005 * lb;

    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    let _ok_l = 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_;
    let ok_a = 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_;
    let ok_b = 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_;

    let hue_rad = ok_b.atan2(ok_a);
    let hue_deg = hue_rad.to_degrees();
    if hue_deg < 0.0 { hue_deg + 360.0 } else { hue_deg }
}

/// Convert sRGB to CIELAB (D65 illuminant).
/// Returns (L*, a*, b*) where L* is lightness [0, 100].
pub(super) fn srgb_to_lab(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let lr = linearize(r as f64 / 255.0);
    let lg = linearize(g as f64 / 255.0);
    let lb = linearize(b as f64 / 255.0);

    // linear RGB → XYZ (sRGB D65 matrix, IEC 61966-2-1)
    let x = 0.4124564 * lr + 0.3575761 * lg + 0.1804375 * lb;
    let y = 0.2126729 * lr + 0.7151522 * lg + 0.0721750 * lb;
    let z = 0.0193339 * lr + 0.1191920 * lg + 0.9503041 * lb;

    // D65 reference white
    let xn = 0.95047;
    let yn = 1.0;
    let zn = 1.08883;

    let fx = lab_f(x / xn);
    let fy = lab_f(y / yn);
    let fz = lab_f(z / zn);

    let l_star = 116.0 * fy - 16.0;
    let a_star = 500.0 * (fx - fy);
    let b_star = 200.0 * (fy - fz);

    (l_star, a_star, b_star)
}

/// CIELAB transfer function.
fn lab_f(t: f64) -> f64 {
    let delta: f64 = 6.0 / 29.0;
    if t > delta * delta * delta {
        t.cbrt()
    } else {
        t / (3.0 * delta * delta) + 4.0 / 29.0
    }
}

/// Compute CIEDE2000 color difference between two CIELAB colors.
/// ISO/CIE 11664-6:2014.
pub(super) fn delta_e_2000(
    l1: f64, a1: f64, b1: f64,
    l2: f64, a2: f64, b2: f64,
) -> f64 {
    let c1 = (a1 * a1 + b1 * b1).sqrt();
    let c2 = (a2 * a2 + b2 * b2).sqrt();
    let c_avg = (c1 + c2) / 2.0;

    let c_avg_7 = c_avg.powi(7);
    let twenty_five_7: f64 = 25.0_f64.powi(7);
    let g = 0.5 * (1.0 - (c_avg_7 / (c_avg_7 + twenty_five_7)).sqrt());

    let a1p = a1 * (1.0 + g);
    let a2p = a2 * (1.0 + g);

    let c1p = (a1p * a1p + b1 * b1).sqrt();
    let c2p = (a2p * a2p + b2 * b2).sqrt();

    let h1p = atan2_pos(b1, a1p);
    let h2p = atan2_pos(b2, a2p);

    // ΔL', ΔC', ΔH'
    let dl = l2 - l1;
    let dc = c2p - c1p;

    let dh_angle = if c1p * c2p == 0.0 {
        0.0
    } else {
        let diff = h2p - h1p;
        if diff.abs() <= 180.0 {
            diff
        } else if diff > 180.0 {
            diff - 360.0
        } else {
            diff + 360.0
        }
    };
    let dh = 2.0 * (c1p * c2p).sqrt() * (dh_angle / 2.0).to_radians().sin();

    // Weighting functions
    let l_avg = (l1 + l2) / 2.0;
    let c_avg_p = (c1p + c2p) / 2.0;

    let h_avg = if c1p * c2p == 0.0 {
        h1p + h2p
    } else if (h1p - h2p).abs() <= 180.0 {
        (h1p + h2p) / 2.0
    } else if h1p + h2p < 360.0 {
        (h1p + h2p + 360.0) / 2.0
    } else {
        (h1p + h2p - 360.0) / 2.0
    };

    let t = 1.0
        - 0.17 * (h_avg - 30.0).to_radians().cos()
        + 0.24 * (2.0 * h_avg).to_radians().cos()
        + 0.32 * (3.0 * h_avg + 6.0).to_radians().cos()
        - 0.20 * (4.0 * h_avg - 63.0).to_radians().cos();

    let l_avg_50_sq = (l_avg - 50.0).powi(2);
    let sl = 1.0 + 0.015 * l_avg_50_sq / (20.0 + l_avg_50_sq).sqrt();
    let sc = 1.0 + 0.045 * c_avg_p;
    let sh = 1.0 + 0.015 * c_avg_p * t;

    let c_avg_p_7 = c_avg_p.powi(7);
    let rc = 2.0 * (c_avg_p_7 / (c_avg_p_7 + twenty_five_7)).sqrt();
    let d_theta = 30.0 * (-((h_avg - 275.0) / 25.0).powi(2)).exp();
    let rt = -(d_theta * 2.0).to_radians().sin() * rc;

    let term_l = dl / sl;
    let term_c = dc / sc;
    let term_h = dh / sh;

    (term_l * term_l + term_c * term_c + term_h * term_h + rt * term_c * term_h).sqrt()
}

/// atan2 returning degrees in [0, 360).
fn atan2_pos(y: f64, x: f64) -> f64 {
    if x == 0.0 && y == 0.0 {
        0.0
    } else {
        let h = y.atan2(x).to_degrees();
        if h < 0.0 { h + 360.0 } else { h }
    }
}
