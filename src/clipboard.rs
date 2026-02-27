use std::io::Write;

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
}
