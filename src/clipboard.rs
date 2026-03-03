use std::io::Write;
use std::process::Command;

/// Determine the clipboard-read command for the current platform.
/// Separated for testability (platform flag + env detection injected).
fn clipboard_read_command(
    is_macos: bool,
    env_fn: &dyn Fn(&str) -> Option<String>,
) -> Option<Vec<String>> {
    if is_macos {
        return Some(vec!["pbpaste".into()]);
    }
    if env_fn("WAYLAND_DISPLAY").is_some() {
        return Some(vec!["wl-paste".into(), "--no-newline".into()]);
    }
    if env_fn("DISPLAY").is_some() {
        return Some(vec![
            "xclip".into(),
            "-selection".into(),
            "clipboard".into(),
            "-o".into(),
        ]);
    }
    None
}

/// Read text from the system clipboard via subprocess.
/// Returns None on any failure (missing tool, empty clipboard, etc.).
pub fn read_clipboard() -> Option<String> {
    let cmd = clipboard_read_command(cfg!(target_os = "macos"), &|k| std::env::var(k).ok())?;
    let output = Command::new(&cmd[0])
        .args(&cmd[1..])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let trimmed = text.trim_end_matches('\n').to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

/// Write text to the system clipboard via OSC 52 escape sequence.
/// This works in terminals that support OSC 52 (most modern terminals).
/// Silently fails on write error (graceful degradation).
pub fn write_osc52(text: &str) {
    let encoded = base64_encode(text.as_bytes());
    let sequence = format!("\x1b]52;c;{}\x07", encoded);
    let _ = std::io::stdout().write_all(sequence.as_bytes());
    let _ = std::io::stdout().flush();
}

/// Minimal base64 encoder — no external dependency needed.
fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    let chunks = input.chunks(3);

    for chunk in chunks {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };

        let triple = (b0 << 16) | (b1 << 8) | b2;

        out.push(TABLE[((triple >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((triple >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            out.push(TABLE[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }

        if chunk.len() > 2 {
            out.push(TABLE[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_encodes_hello() {
        assert_eq!(base64_encode(b"Hello"), "SGVsbG8=");
    }

    #[test]
    fn base64_encodes_empty() {
        assert_eq!(base64_encode(b""), "");
    }

    #[test]
    fn base64_encodes_single_byte() {
        assert_eq!(base64_encode(b"A"), "QQ==");
    }

    #[test]
    fn base64_encodes_two_bytes() {
        assert_eq!(base64_encode(b"AB"), "QUI=");
    }

    #[test]
    fn base64_encodes_three_bytes() {
        assert_eq!(base64_encode(b"ABC"), "QUJD");
    }

    #[test]
    fn base64_encodes_multiline_text() {
        let text = "hello\nworld";
        let encoded = base64_encode(text.as_bytes());
        assert_eq!(encoded, "aGVsbG8Kd29ybGQ=");
    }

    #[test]
    fn osc52_sequence_has_correct_format() {
        // We can't easily capture stdout in a unit test, but we can verify
        // the base64 encoding that feeds into it.
        let encoded = base64_encode(b"test");
        let sequence = format!("\x1b]52;c;{}\x07", encoded);
        assert!(sequence.starts_with("\x1b]52;c;"));
        assert!(sequence.ends_with("\x07"));
        assert!(sequence.contains("dGVzdA=="));
    }

    // === Clipboard read command detection ===

    #[test]
    fn clipboard_command_macos() {
        let cmd = clipboard_read_command(true, &|_| None);
        assert_eq!(cmd, Some(vec!["pbpaste".to_string()]));
    }

    #[test]
    fn clipboard_command_wayland() {
        let cmd = clipboard_read_command(false, &|k| {
            if k == "WAYLAND_DISPLAY" { Some("wayland-0".into()) } else { None }
        });
        assert_eq!(cmd, Some(vec!["wl-paste".to_string(), "--no-newline".to_string()]));
    }

    #[test]
    fn clipboard_command_x11() {
        let cmd = clipboard_read_command(false, &|k| {
            if k == "DISPLAY" { Some(":0".into()) } else { None }
        });
        assert_eq!(cmd, Some(vec![
            "xclip".to_string(), "-selection".to_string(),
            "clipboard".to_string(), "-o".to_string(),
        ]));
    }

    #[test]
    fn clipboard_command_unknown() {
        let cmd = clipboard_read_command(false, &|_| None);
        assert_eq!(cmd, None);
    }
}
