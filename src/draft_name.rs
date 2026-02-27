use rand::Rng;
use std::time::SystemTime;

const ADJECTIVES: &[&str] = &[
    "bright", "calm", "clear", "cool", "crisp",
    "dark", "deep", "dry", "dusk", "faint",
    "first", "fresh", "full", "gold", "green",
    "half", "high", "hush", "last", "late",
    "light", "long", "lost", "low", "mild",
    "mist", "near", "new", "old", "pale",
    "peak", "pure", "quiet", "rare", "red",
    "shy", "slow", "soft", "still", "warm",
];

const PLANTS: &[&str] = &[
    "alder", "aster", "azalea", "balm", "birch",
    "brome", "camas", "cedar", "clover", "columbine",
    "daisy", "elder", "fawn", "fern", "flax",
    "hazel", "heath", "hemlock", "heron", "iris",
    "kinnikinnick", "larkspur", "lichen", "lily", "lupine",
    "madrone", "maple", "moss", "nettle", "oak",
    "orchid", "pine", "poppy", "reed", "sage",
    "sedge", "sorrel", "spruce", "sumac", "yarrow",
];

/// Generate a draft filename: `YYYY-MM-DD-HHMM-adjective-plant.md`
pub fn generate() -> String {
    let mut rng = rand::thread_rng();
    let adj = ADJECTIVES[rng.gen_range(0..ADJECTIVES.len())];
    let plant = PLANTS[rng.gen_range(0..PLANTS.len())];
    let ts = utc_timestamp();
    format!("{}-{}-{}.md", ts, adj, plant)
}

/// Format current UTC time as `YYYY-MM-DD-HHMM` without chrono.
fn utc_timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let day_secs = secs % 86400;
    let hour = day_secs / 3600;
    let minute = (day_secs % 3600) / 60;

    let days = (secs / 86400) as i64;
    let (year, month, day) = days_to_civil(days + 719468);

    format!("{:04}-{:02}-{:02}-{:02}{:02}", year, month, day, hour, minute)
}

/// Convert a day count to (year, month, day) using Hinnant's algorithm.
/// Input: days since epoch 0000-03-01 (civil day number + 719468).
fn days_to_civil(z: i64) -> (i64, u32, u32) {
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32; // day of era [0, 146096]
    let yoe =
        (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // year of era [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day of year [0, 365]
    let mp = (5 * doy + 2) / 153; // month index [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // day [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // month [1, 12]
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_matches_expected_format() {
        let name = generate();
        // e.g. "2026-02-26-1430-quiet-camas.md"
        assert!(name.ends_with(".md"), "should end with .md: {}", name);

        let parts: Vec<&str> = name.trim_end_matches(".md").splitn(6, '-').collect();
        assert_eq!(parts.len(), 6, "should have 6 dash-separated parts: {:?}", parts);

        // Year is 4 digits
        assert_eq!(parts[0].len(), 4, "year should be 4 digits: {}", parts[0]);
        // Month is 2 digits
        assert_eq!(parts[1].len(), 2, "month should be 2 digits: {}", parts[1]);
        // Day is 2 digits
        assert_eq!(parts[2].len(), 2, "day should be 2 digits: {}", parts[2]);
        // HHMM is 4 digits
        assert_eq!(parts[3].len(), 4, "HHMM should be 4 digits: {}", parts[3]);
        // Adjective exists
        assert!(ADJECTIVES.contains(&parts[4]), "adjective should be from list: {}", parts[4]);
        // Plant exists
        assert!(PLANTS.contains(&parts[5]), "plant should be from list: {}", parts[5]);
    }

    #[test]
    fn epoch_converts_to_1970_01_01() {
        let (y, m, d) = days_to_civil(719468);
        assert_eq!((y, m, d), (1970, 1, 1));
    }

    #[test]
    fn known_date_converts_correctly() {
        // 2026-02-26 = day 20,510 since epoch (1970-01-01)
        // 20510 + 719468 = 739978
        let (y, m, d) = days_to_civil(739978);
        assert_eq!((y, m, d), (2026, 2, 26));
    }

    #[test]
    fn two_names_are_almost_certainly_different() {
        let a = generate();
        let b = generate();
        // With 1600+ combinations this should practically never collide
        // (same timestamp is fine, different adj-plant)
        assert_ne!(a, b, "two generated names should differ");
    }
}
