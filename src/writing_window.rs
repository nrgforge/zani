/// Detected terminal emulator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Terminal {
    Ghostty,
    Kitty,
    WezTerm,
    Alacritty,
    ITerm2,
    Unknown(String),
}

/// Configuration for the Writing Window.
#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub font_family: String,
    pub font_size: u16,
    pub title: String,
    pub padding_x: u16,
    pub padding_y: u16,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            font_family: "PT Mono".to_string(),
            font_size: 24,
            title: "Zani".to_string(),
            padding_x: 20,
            padding_y: 20,
        }
    }
}

/// Detect the terminal emulator from environment variables.
pub fn detect_terminal(env: &dyn Fn(&str) -> Option<String>) -> Terminal {
    if env("GHOSTTY_RESOURCES_DIR").is_some() {
        return Terminal::Ghostty;
    }
    if env("KITTY_PID").is_some() {
        return Terminal::Kitty;
    }
    if env("WEZTERM_EXECUTABLE").is_some() {
        return Terminal::WezTerm;
    }
    if let Some(term_program) = env("TERM_PROGRAM") {
        match term_program.as_str() {
            "iTerm.app" => return Terminal::ITerm2,
            "Alacritty" => return Terminal::Alacritty,
            _ => {}
        }
    }
    let name = env("TERM_PROGRAM").unwrap_or_else(|| "unknown".to_string());
    Terminal::Unknown(name)
}

/// Build the command to spawn a Writing Window for the detected terminal.
/// `binary` is the path to the zani executable (use `std::env::current_exe()`).
/// Returns None if the terminal is unknown (should run inline instead).
pub fn spawn_command(
    terminal: &Terminal,
    config: &WindowConfig,
    binary: &str,
    zani_args: &[&str],
) -> Option<Vec<String>> {
    let inline_args: Vec<String> = std::iter::once(binary.to_string())
        .chain(zani_args.iter().map(|s| s.to_string()))
        .collect();

    match terminal {
        Terminal::Ghostty => {
            // macOS: use the app bundle binary directly — `open -na` doesn't
            // forward config flags, and the homebrew `ghostty` shim doesn't
            // open windows.
            let ghostty_bin = if cfg!(target_os = "macos") {
                "/Applications/Ghostty.app/Contents/MacOS/ghostty".to_string()
            } else {
                "ghostty".to_string()
            };
            let mut cmd = vec![
                ghostty_bin,
                format!("--font-family={}", config.font_family),
                format!("--font-size={}", config.font_size),
                format!("--title={}", config.title),
                format!("--window-padding-x={}", config.padding_x),
                format!("--window-padding-y={}", config.padding_y),
                "--window-padding-balance=true".to_string(),
                "-e".to_string(),
            ];
            cmd.extend(inline_args);
            Some(cmd)
        }
        Terminal::Kitty => {
            let mut cmd = vec![
                "kitty".to_string(),
                "-o".to_string(),
                format!("font_family={}", config.font_family),
                "-o".to_string(),
                format!("font_size={}", config.font_size),
            ];
            cmd.extend(inline_args);
            Some(cmd)
        }
        Terminal::WezTerm => {
            let mut cmd = vec![
                "wezterm".to_string(),
                "start".to_string(),
                "--".to_string(),
            ];
            cmd.extend(inline_args);
            Some(cmd)
        }
        Terminal::Alacritty => {
            let mut cmd = vec![
                "alacritty".to_string(),
                "-o".to_string(),
                format!("font.normal.family={}", config.font_family),
                "-o".to_string(),
                format!("font.size={}", config.font_size),
                "-e".to_string(),
            ];
            cmd.extend(inline_args);
            Some(cmd)
        }
        Terminal::ITerm2 => {
            // iTerm2 uses profile-based launch; simplified here
            let mut cmd = vec![
                "open".to_string(),
                "-a".to_string(),
                "iTerm".to_string(),
                "--args".to_string(),
            ];
            cmd.extend(inline_args);
            Some(cmd)
        }
        Terminal::Unknown(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_env<'a>(vars: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |key| {
            vars.iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| v.to_string())
        }
    }

    // === Acceptance test: Zani spawns a dedicated Writing Window on supported terminal ===

    #[test]
    fn detects_ghostty() {
        let env = make_env(&[("GHOSTTY_RESOURCES_DIR", "/usr/share/ghostty")]);
        assert_eq!(detect_terminal(&env), Terminal::Ghostty);
    }

    #[test]
    fn detects_kitty() {
        let env = make_env(&[("KITTY_PID", "12345")]);
        assert_eq!(detect_terminal(&env), Terminal::Kitty);
    }

    #[test]
    fn detects_wezterm() {
        let env = make_env(&[("WEZTERM_EXECUTABLE", "/usr/bin/wezterm")]);
        assert_eq!(detect_terminal(&env), Terminal::WezTerm);
    }

    #[test]
    fn detects_alacritty() {
        let env = make_env(&[("TERM_PROGRAM", "Alacritty")]);
        assert_eq!(detect_terminal(&env), Terminal::Alacritty);
    }

    #[test]
    fn detects_iterm2() {
        let env = make_env(&[("TERM_PROGRAM", "iTerm.app")]);
        assert_eq!(detect_terminal(&env), Terminal::ITerm2);
    }

    #[test]
    fn spawn_command_includes_binary_and_font() {
        let config = WindowConfig::default();
        let cmd = spawn_command(&Terminal::Ghostty, &config, "/usr/bin/zani", &["draft.md"]).unwrap();
        assert!(!cmd.contains(&"--inline".to_string()));
        assert!(cmd.contains(&"/usr/bin/zani".to_string()));
        assert!(cmd.contains(&"draft.md".to_string()));
        assert!(cmd.iter().any(|s| s.contains("font-family")));
        if cfg!(target_os = "macos") {
            assert!(cmd[0].contains("Ghostty.app"));
        } else {
            assert_eq!(cmd[0], "ghostty");
        }
    }

    // === Acceptance test: Unknown terminal falls back to Inline Mode ===

    #[test]
    fn unknown_terminal_returns_none_spawn_command() {
        let config = WindowConfig::default();
        let cmd = spawn_command(&Terminal::Unknown("xterm".to_string()), &config, "zani", &["draft.md"]);
        assert!(cmd.is_none());
    }

    #[test]
    fn unknown_terminal_detected_from_empty_env() {
        let env = make_env(&[]);
        assert!(matches!(detect_terminal(&env), Terminal::Unknown(_)));
    }

}
