use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::editing_mode::EditingMode;
use crate::focus_mode::FocusMode;
use crate::palette::Palette;
use crate::scroll_mode::ScrollMode;

/// Persisted user preferences, loaded from and saved to config.toml.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    /// Name of the active palette (matched against Palette::all()).
    #[serde(default = "default_palette_name")]
    pub palette: String,
    /// Active focus mode.
    #[serde(default, with = "focus_mode_serde")]
    pub focus_mode: FocusMode,
    /// Prose column width.
    #[serde(default = "default_column_width")]
    pub column_width: u16,
    /// Editing mode (vim or standard).
    #[serde(default)]
    pub editing_mode: EditingMode,
    /// Scroll mode (edge or typewriter).
    #[serde(default, with = "scroll_mode_serde")]
    pub scroll_mode: ScrollMode,
}

fn default_palette_name() -> String {
    Palette::default_palette().name.to_string()
}

fn default_column_width() -> u16 {
    60
}

impl Default for Config {
    fn default() -> Self {
        Self {
            palette: default_palette_name(),
            focus_mode: FocusMode::Off,
            column_width: default_column_width(),
            editing_mode: EditingMode::default(),
            scroll_mode: ScrollMode::Edge,
        }
    }
}

impl Config {
    /// Resolve the palette name to a Palette, falling back to default.
    pub fn resolve_palette(&self) -> Palette {
        Palette::all()
            .into_iter()
            .find(|p| p.name == self.palette)
            .unwrap_or_else(Palette::default_palette)
    }

    /// Config file path: $HOME/.config/zani/config.toml
    pub fn path() -> Option<PathBuf> {
        std::env::var("HOME")
            .ok()
            .map(|home| PathBuf::from(home).join(".config/zani/config.toml"))
    }

    /// Load config from disk. Returns default if file doesn't exist or is invalid.
    /// Clamps column_width to 20–120 to enforce Invariant 5.
    pub fn load() -> Self {
        let mut config: Config = Self::path()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .and_then(|content| toml::from_str(&content).ok())
            .unwrap_or_default();
        config.column_width = config.column_width.clamp(20, 120);
        config
    }

    /// Save config to disk. Creates parent directories as needed.
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::path().ok_or("could not determine config path")?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// Serde support for FocusMode as a lowercase string.
mod focus_mode_serde {
    use super::FocusMode;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(mode: &FocusMode, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match mode {
            FocusMode::Off => "off",
            FocusMode::Sentence => "sentence",
            FocusMode::Paragraph => "paragraph",
        };
        serializer.serialize_str(s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<FocusMode, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "off" => Ok(FocusMode::Off),
            "sentence" => Ok(FocusMode::Sentence),
            "paragraph" => Ok(FocusMode::Paragraph),
            "typewriter" => Ok(FocusMode::Off),
            _ => Ok(FocusMode::Off),
        }
    }
}

/// Serde support for ScrollMode as a lowercase string.
mod scroll_mode_serde {
    use super::ScrollMode;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(mode: &ScrollMode, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match mode {
            ScrollMode::Edge => "edge",
            ScrollMode::Typewriter => "typewriter",
        };
        serializer.serialize_str(s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ScrollMode, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "edge" => Ok(ScrollMode::Edge),
            "typewriter" => Ok(ScrollMode::Typewriter),
            _ => Ok(ScrollMode::Edge),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editing_mode::EditingMode;

    #[test]
    fn default_config_values() {
        let config = Config::default();
        assert_eq!(config.palette, "Ember");
        assert_eq!(config.focus_mode, FocusMode::Off);
        assert_eq!(config.column_width, 60);
        assert_eq!(config.editing_mode, EditingMode::Vim);
    }

    #[test]
    fn round_trip_serialization() {
        let config = Config {
            palette: "Inkwell".to_string(),
            focus_mode: FocusMode::Paragraph,
            column_width: 72,
            editing_mode: EditingMode::Standard,
            scroll_mode: ScrollMode::Typewriter,
        };
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let loaded: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(config, loaded);
    }

    #[test]
    fn deserialize_with_missing_fields_uses_defaults() {
        let toml_str = r#"palette = "Parchment""#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.palette, "Parchment");
        assert_eq!(config.focus_mode, FocusMode::Off);
        assert_eq!(config.column_width, 60);
    }

    #[test]
    fn empty_toml_gives_defaults() {
        let config: Config = toml::from_str("").unwrap();
        assert_eq!(config, Config::default());
    }

    #[test]
    fn missing_editing_mode_defaults_to_vim() {
        let toml_str = r#"palette = "Ember""#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.editing_mode, EditingMode::Vim);
    }

    #[test]
    fn editing_mode_round_trip() {
        let config = Config {
            editing_mode: EditingMode::Standard,
            ..Config::default()
        };
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let loaded: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(loaded.editing_mode, EditingMode::Standard);
    }

    #[test]
    fn resolve_palette_finds_known_palette() {
        let config = Config {
            palette: "Inkwell".to_string(),
            ..Config::default()
        };
        let palette = config.resolve_palette();
        assert_eq!(palette.name, "Inkwell");
    }

    #[test]
    fn resolve_palette_falls_back_on_unknown() {
        let config = Config {
            palette: "NonExistent".to_string(),
            ..Config::default()
        };
        let palette = config.resolve_palette();
        assert_eq!(palette.name, Palette::default_palette().name);
    }

    #[test]
    fn legacy_typewriter_focus_mode_maps_to_off() {
        let toml_str = r#"focus_mode = "typewriter""#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.focus_mode, FocusMode::Off);
    }

    #[test]
    fn column_width_clamped_on_deserialize() {
        let too_low: Config = toml::from_str("column_width = 5").unwrap();
        let mut clamped_low = too_low;
        clamped_low.column_width = clamped_low.column_width.clamp(20, 120);
        assert_eq!(clamped_low.column_width, 20);

        let too_high: Config = toml::from_str("column_width = 200").unwrap();
        let mut clamped_high = too_high;
        clamped_high.column_width = clamped_high.column_width.clamp(20, 120);
        assert_eq!(clamped_high.column_width, 120);
    }

    #[test]
    fn scroll_mode_round_trip() {
        let config = Config {
            scroll_mode: ScrollMode::Typewriter,
            ..Config::default()
        };
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let loaded: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(loaded.scroll_mode, ScrollMode::Typewriter);
    }
}
