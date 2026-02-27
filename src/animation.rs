/// Easing curve selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Easing {
    EaseOut,
    EaseInOut,
}

/// Cubic ease-out: 1 - (1-t)^3
pub fn ease_out(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    1.0 - inv * inv * inv
}

/// Cubic ease-in-out: 3t^2 - 2t^3
pub fn ease_in_out(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    3.0 * t * t - 2.0 * t * t * t
}

impl Easing {
    pub fn apply(self, t: f64) -> f64 {
        match self {
            Easing::EaseOut => ease_out(t),
            Easing::EaseInOut => ease_in_out(t),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Task 1: Easing function tests ===

    #[test]
    fn ease_out_at_zero_is_zero() {
        assert_eq!(ease_out(0.0), 0.0);
    }

    #[test]
    fn ease_out_at_one_is_one() {
        assert!((ease_out(1.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ease_out_is_monotonic() {
        let mut prev = ease_out(0.0);
        for i in 1..=100 {
            let t = i as f64 / 100.0;
            let curr = ease_out(t);
            assert!(curr >= prev, "ease_out not monotonic at t={t}: {curr} < {prev}");
            prev = curr;
        }
    }

    #[test]
    fn ease_in_out_at_zero_is_zero() {
        assert_eq!(ease_in_out(0.0), 0.0);
    }

    #[test]
    fn ease_in_out_at_one_is_one() {
        assert!((ease_in_out(1.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ease_in_out_at_half_is_half() {
        assert!((ease_in_out(0.5) - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn easing_apply_dispatches_correctly() {
        let t = 0.3;
        assert_eq!(Easing::EaseOut.apply(t), ease_out(t));
        assert_eq!(Easing::EaseInOut.apply(t), ease_in_out(t));
    }
}
