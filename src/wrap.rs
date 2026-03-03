/// A visual line produced by soft-wrapping a logical line.
/// Stores the byte range within the original line and the character offsets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualLine {
    /// Index of the logical line this visual line belongs to.
    pub logical_line: usize,
    /// Character offset within the logical line where this visual line starts.
    pub char_start: usize,
    /// Character offset within the logical line where this visual line ends (exclusive).
    pub char_end: usize,
}

/// Soft-wrap a line of text to fit within `width` characters.
/// Breaks at word boundaries (spaces) when possible.
/// Returns a list of VisualLines representing the wrapped output.
pub fn wrap_line(text: &str, width: usize, logical_line: usize) -> Vec<VisualLine> {
    if text.is_empty() || text == "\n" {
        return vec![VisualLine {
            logical_line,
            char_start: 0,
            char_end: 0,
        }];
    }

    // Strip trailing newline for wrapping purposes
    let text = text.trim_end_matches('\n');
    if text.is_empty() {
        return vec![VisualLine {
            logical_line,
            char_start: 0,
            char_end: 0,
        }];
    }

    let chars: Vec<char> = text.chars().collect();
    let total = chars.len();
    let mut lines = Vec::new();
    let mut pos = 0;

    while pos < total {
        let remaining = total - pos;
        if remaining <= width {
            lines.push(VisualLine {
                logical_line,
                char_start: pos,
                char_end: total,
            });
            break;
        }

        // Look for the last space within the width
        let end = pos + width;
        let mut break_at = None;
        for i in (pos..end).rev() {
            if chars[i] == ' ' {
                break_at = Some(i);
                break;
            }
        }

        match break_at {
            Some(space_idx) => {
                lines.push(VisualLine {
                    logical_line,
                    char_start: pos,
                    char_end: space_idx,
                });
                // Skip the space
                pos = space_idx + 1;
            }
            None => {
                // No space found — hard break at width
                lines.push(VisualLine {
                    logical_line,
                    char_start: pos,
                    char_end: end,
                });
                pos = end;
            }
        }
    }

    lines
}

/// Wrap all lines from a buffer, returning a flat list of visual lines.
pub fn wrap_all(lines: &[String], width: usize) -> Vec<VisualLine> {
    lines
        .iter()
        .enumerate()
        .flat_map(|(idx, line)| wrap_line(line, width, idx))
        .collect()
}

/// Compute visual lines for a Buffer (rope-backed text).
/// Shared between WritingSurface rendering and cursor visibility scrolling.
pub fn visual_lines_for_buffer(buffer: &crate::buffer::Buffer, column_width: u16) -> Vec<VisualLine> {
    let mut all = Vec::new();
    let mut line_buf = String::new();
    for i in 0..buffer.len_lines() {
        line_buf.clear();
        use std::fmt::Write;
        let _ = write!(line_buf, "{}", buffer.line(i));
        let wrapped = wrap_line(&line_buf, column_width as usize, i);
        all.extend(wrapped);
    }
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Acceptance test: Text wraps at prose-width column ===

    #[test]
    fn wraps_at_specified_width_without_breaking_mid_word() {
        let text = "The quick brown fox jumps over the lazy dog and keeps on running forever";
        let lines = wrap_line(text, 30, 0);

        // Every visual line should be <= 30 chars
        for vl in &lines {
            let len = vl.char_end - vl.char_start;
            assert!(len <= 30, "Line too long: {} chars", len);
        }

        // No word should be split (unless a single word exceeds width)
        let chars: Vec<char> = text.chars().collect();
        for vl in &lines {
            if vl.char_start > 0 {
                // The character before this line's start should be a space
                // (we skip spaces at break points)
                // Actually, the previous line ends before the space, and we skip it
                // So char_start should be right after a space in the original
                assert!(
                    vl.char_start == 0 || chars[vl.char_start - 1] == ' ',
                    "Line doesn't start after a word boundary"
                );
            }
        }
    }

    #[test]
    fn wraps_at_60_char_column() {
        let text = "This is a longer paragraph that should definitely wrap when we constrain it to sixty characters wide because it has many words in it.";
        let lines = wrap_line(text, 60, 0);
        assert!(lines.len() > 1, "Should wrap to multiple lines");
        for vl in &lines {
            let len = vl.char_end - vl.char_start;
            assert!(len <= 60, "Line too long: {} chars", len);
        }
    }

    // === Unit tests ===

    #[test]
    fn short_line_no_wrap() {
        let lines = wrap_line("Hello world", 60, 0);
        assert_eq!(lines.len(), 1, "short line should produce 1 visual line");
        assert_eq!(lines[0].char_start, 0, "char_start should be 0");
        assert_eq!(lines[0].char_end, 11, "char_end should be 11");
    }

    #[test]
    fn empty_line() {
        let lines = wrap_line("", 60, 0);
        assert_eq!(lines.len(), 1, "empty line should produce 1 visual line");
        assert_eq!(lines[0].char_start, 0, "char_start should be 0");
        assert_eq!(lines[0].char_end, 0, "char_end should be 0");
    }

    #[test]
    fn newline_only() {
        let lines = wrap_line("\n", 60, 0);
        assert_eq!(lines.len(), 1, "newline-only should produce 1 visual line");
        assert_eq!(lines[0].char_start, 0, "char_start should be 0");
        assert_eq!(lines[0].char_end, 0, "char_end should be 0");
    }

    #[test]
    fn long_word_hard_breaks() {
        let text = "supercalifragilisticexpialidocious";
        let lines = wrap_line(text, 10, 0);
        assert!(lines.len() > 1);
        // First line should be exactly 10 chars (hard break)
        assert_eq!(lines[0].char_end - lines[0].char_start, 10);
    }

    #[test]
    fn preserves_logical_line_index() {
        let lines = wrap_line("short", 60, 5);
        assert_eq!(lines[0].logical_line, 5);
    }

    #[test]
    fn wrap_all_multiple_lines() {
        let input = vec![
            "First line\n".to_string(),
            "Second line that is quite a bit longer and should wrap at the boundary\n".to_string(),
            "Third\n".to_string(),
        ];
        let visual = wrap_all(&input, 30);
        // First and third lines fit in 30 chars, second wraps
        assert!(visual.len() > 3, "wrapping long line should produce more than 3 visual lines");
        assert_eq!(visual[0].logical_line, 0, "first visual line should be logical line 0");
        assert_eq!(visual.last().unwrap().logical_line, 2, "last visual line should be logical line 2");
    }

    #[test]
    fn trailing_newline_stripped_for_wrapping() {
        let lines = wrap_line("Hello world\n", 60, 0);
        assert_eq!(lines.len(), 1, "trailing newline should not add a visual line");
        assert_eq!(lines[0].char_end, 11, "char_end should exclude trailing newline");
    }
}
