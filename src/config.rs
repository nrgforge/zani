use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::editing_mode::EditingMode;
use crate::focus_mode::FocusMode;
use crate::palette::Palette;
use crate::scroll_mode::ScrollMode;

/// Where the active config was resolved from (ADR-011).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigSource {
    /// Built-in defaults only.
    Default,
    /// Global config file (~/.config/zani/config.toml).
    Global,
    /// Local config file (.zani.toml in a project directory).
    Local,
}

/// Per-project config overrides from `.zani.toml` (ADR-011).
/// All fields are optional — unspecified fields fall through to
/// global config, then defaults.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocalConfig {
    pub palette: Option<String>,
    pub focus_mode: Option<String>,
    pub column_width: Option<u16>,
    pub editing_mode: Option<String>,
    pub scroll_mode: Option<String>,
}

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

    /// Load config with local override resolution (ADR-011).
    /// Walks up from `file_path` looking for `.zani.toml`. If found,
    /// its fields override the global config. Returns the resolved
    /// config and its source.
    pub fn load_for_path(file_path: &Path) -> (Self, ConfigSource) {
        let global = Self::load();

        // Walk up from file's parent directory looking for .zani.toml
        let start = if file_path.is_dir() {
            file_path.to_path_buf()
        } else {
            file_path.parent().map(|p| p.to_path_buf()).unwrap_or_default()
        };

        let mut dir = Some(start.as_path());
        while let Some(d) = dir {
            let local_path = d.join(".zani.toml");
            if let Ok(content) = std::fs::read_to_string(&local_path) {
                if let Ok(local) = toml::from_str::<LocalConfig>(&content) {
                    let mut config = global;
                    config.merge_local(&local);
                    return (config, ConfigSource::Local);
                }
            }
            dir = d.parent();
        }

        let source = if Self::path().map_or(false, |p| p.exists()) {
            ConfigSource::Global
        } else {
            ConfigSource::Default
        };
        (global, source)
    }

    /// Apply local config overrides to this config.
    fn merge_local(&mut self, local: &LocalConfig) {
        if let Some(ref p) = local.palette {
            self.palette = p.clone();
        }
        if let Some(ref fm) = local.focus_mode {
            self.focus_mode = match fm.as_str() {
                "sentence" => FocusMode::Sentence,
                "paragraph" => FocusMode::Paragraph,
                _ => FocusMode::Off,
            };
        }
        if let Some(cw) = local.column_width {
            self.column_width = cw.clamp(20, 120);
        }
        if let Some(ref em) = local.editing_mode {
            self.editing_mode = match em.as_str() {
                "standard" => EditingMode::Standard,
                _ => EditingMode::Vim,
            };
        }
        if let Some(ref sm) = local.scroll_mode {
            self.scroll_mode = match sm.as_str() {
                "typewriter" => ScrollMode::Typewriter,
                _ => ScrollMode::Edge,
            };
        }
    }

    /// Write a `.zani.toml` file binding a palette to a project directory (ADR-011).
    pub fn bind_to_project(dir: &Path, palette_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let local = LocalConfig {
            palette: Some(palette_name.to_string()),
            ..LocalConfig::default()
        };
        let content = toml::to_string_pretty(&local)?;
        let path = dir.join(".zani.toml");
        std::fs::write(path, content)?;
        Ok(())
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
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn default_config_values() {
        let config = Config::default();
        assert_eq!(config.palette, "Ember", "default palette should be Ember");
        assert_eq!(config.focus_mode, FocusMode::Off, "default focus mode should be Off");
        assert_eq!(config.column_width, 60, "default column width should be 60");
        assert_eq!(config.editing_mode, EditingMode::Vim, "default editing mode should be Vim");
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
        assert_eq!(config.palette, "Parchment", "palette should match specified value");
        assert_eq!(config.focus_mode, FocusMode::Off, "missing focus_mode should default to Off");
        assert_eq!(config.column_width, 60, "missing column_width should default to 60");
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

    // === Acceptance tests: Local Config (ADR-011) ===

    #[test]
    fn local_config_overrides_global_palette() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join(".zani.toml"),
            r#"palette = "Inkwell""#,
        ).unwrap();

        let file = dir.path().join("document.md");
        fs::write(&file, "test").unwrap();

        let (config, source) = Config::load_for_path(&file);
        assert_eq!(config.palette, "Inkwell", "Local config should override palette");
        assert_eq!(source, ConfigSource::Local);
    }

    #[test]
    fn walk_up_search_finds_nearest_local_config() {
        let dir = TempDir::new().unwrap();
        // .zani.toml at project root
        fs::write(
            dir.path().join(".zani.toml"),
            r#"palette = "Parchment""#,
        ).unwrap();
        // Subdirectory with no .zani.toml
        let sub = dir.path().join("chapters");
        fs::create_dir(&sub).unwrap();
        let file = sub.join("chapter1.md");
        fs::write(&file, "test").unwrap();

        let (config, source) = Config::load_for_path(&file);
        assert_eq!(config.palette, "Parchment", "Walk-up should find parent's .zani.toml");
        assert_eq!(source, ConfigSource::Local);
    }

    #[test]
    fn partial_local_config_merges_with_global() {
        let dir = TempDir::new().unwrap();
        // Local config with only palette
        fs::write(
            dir.path().join(".zani.toml"),
            r#"palette = "Inkwell""#,
        ).unwrap();
        let file = dir.path().join("doc.md");
        fs::write(&file, "test").unwrap();

        let global = Config::load();
        let (config, _) = Config::load_for_path(&file);
        assert_eq!(config.palette, "Inkwell", "Palette from local");
        // Unspecified fields should come from global config
        assert_eq!(config.column_width, global.column_width, "Unspecified column_width from global");
        assert_eq!(config.focus_mode, global.focus_mode, "Unspecified focus_mode from global");
        assert_eq!(config.editing_mode, global.editing_mode, "Unspecified editing_mode from global");
        assert_eq!(config.scroll_mode, global.scroll_mode, "Unspecified scroll_mode from global");
    }

    #[test]
    fn no_local_config_falls_through() {
        let dir = TempDir::new().unwrap();
        // No .zani.toml anywhere
        let file = dir.path().join("doc.md");
        fs::write(&file, "test").unwrap();

        let global = Config::load();
        let (config, _) = Config::load_for_path(&file);
        assert_eq!(config.palette, global.palette, "Should fall through to global/default");
    }

    #[test]
    fn bind_writes_zani_toml() {
        let dir = TempDir::new().unwrap();
        Config::bind_to_project(dir.path(), "Neon Noir").unwrap();

        let content = fs::read_to_string(dir.path().join(".zani.toml")).unwrap();
        let local: LocalConfig = toml::from_str(&content).unwrap();
        assert_eq!(local.palette, Some("Neon Noir".to_string()));
    }

    #[test]
    fn bind_then_load_round_trip() {
        let dir = TempDir::new().unwrap();
        Config::bind_to_project(dir.path(), "Inkwell").unwrap();

        let file = dir.path().join("doc.md");
        fs::write(&file, "test").unwrap();

        let (config, source) = Config::load_for_path(&file);
        assert_eq!(config.palette, "Inkwell", "load_for_path should read the bound palette");
        assert_eq!(source, ConfigSource::Local);
    }
}
