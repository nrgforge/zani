use ratatui::style::{Modifier, Style};

use crate::palette::Palette;

/// Style information for a single character in a markdown line.
#[derive(Debug, Clone, PartialEq)]
pub struct CharStyle {
    /// Whether this character is a syntax character (should be dimmed).
    pub is_syntax: bool,
    /// Text modifier (bold, italic, none).
    pub modifier: Modifier,
    /// Whether this is a heading's text content.
    pub is_heading: bool,
    /// Whether this is inside a code block or inline code.
    pub is_code: bool,
}

impl Default for CharStyle {
    fn default() -> Self {
        Self {
            is_syntax: false,
            modifier: Modifier::empty(),
            is_heading: false,
            is_code: false,
        }
    }
}

impl CharStyle {
    /// Resolve this char style into a ratatui Style using the given Palette.
    pub fn resolve(&self, palette: &Palette) -> Style {
        let fg = if self.is_syntax {
            palette.dimmed_foreground
        } else if self.is_heading {
            palette.accent_heading
        } else if self.is_code {
            palette.accent_code
        } else {
            palette.foreground
        };

        Style::default()
            .fg(fg)
            .bg(palette.background)
            .add_modifier(self.modifier)
    }
}

/// Check if a line is a fenced code block delimiter (starts with ```).
pub fn is_fence_line(line: &str) -> bool {
    line.trim_start().starts_with("```")
}

/// Parse a single line of markdown and return per-character style information.
/// This is a render-time operation — it does not modify the text.
///
/// When `in_code_block` is true, all characters are styled as code and
/// heading/bold/italic parsing is skipped. Fence lines (starting with ```)
/// get `is_syntax + is_code` on all characters.
pub fn style_line(line: &str) -> Vec<CharStyle> {
    style_line_with_context(line, false)
}

/// Parse a line with code block context.
pub fn style_line_with_context(line: &str, in_code_block: bool) -> Vec<CharStyle> {
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut styles = vec![CharStyle::default(); len];

    // Fenced code block handling
    if is_fence_line(line) {
        // Fence lines: all chars are syntax + code
        for s in &mut styles {
            s.is_syntax = true;
            s.is_code = true;
        }
        return styles;
    }
    if in_code_block {
        // Inside a fenced block: all chars are code, skip other parsing
        for s in &mut styles {
            s.is_code = true;
        }
        return styles;
    }

    // Heading detection: line starts with # followed by space
    if let Some(heading_level) = detect_heading(&chars) {
        // Mark the # and space as syntax
        let prefix_len = heading_level + 1; // # chars + space
        for i in 0..prefix_len.min(len) {
            styles[i].is_syntax = true;
            styles[i].is_heading = true;
            styles[i].modifier = Modifier::BOLD;
        }
        // Mark the rest as heading text
        for i in prefix_len..len {
            styles[i].is_heading = true;
            styles[i].modifier = Modifier::BOLD;
        }
        return styles;
    }

    // Inline formatting
    let mut i = 0;
    while i < len {
        // Bold: **text**
        if i + 1 < len && chars[i] == '*' && chars[i + 1] == '*' {
            if let Some(end) = find_closing(&chars, i + 2, &['*', '*']) {
                // Opening **
                styles[i].is_syntax = true;
                styles[i].modifier = Modifier::BOLD;
                styles[i + 1].is_syntax = true;
                styles[i + 1].modifier = Modifier::BOLD;
                // Content
                for j in (i + 2)..end {
                    styles[j].modifier = Modifier::BOLD;
                }
                // Closing **
                styles[end].is_syntax = true;
                styles[end].modifier = Modifier::BOLD;
                styles[end + 1].is_syntax = true;
                styles[end + 1].modifier = Modifier::BOLD;
                i = end + 2;
                continue;
            }
        }

        // Italic: *text* (but not **)
        if chars[i] == '*' && (i + 1 >= len || chars[i + 1] != '*') {
            if let Some(end) = find_closing_single(&chars, i + 1, '*') {
                styles[i].is_syntax = true;
                styles[i].modifier = Modifier::ITALIC;
                for j in (i + 1)..end {
                    styles[j].modifier = Modifier::ITALIC;
                }
                styles[end].is_syntax = true;
                styles[end].modifier = Modifier::ITALIC;
                i = end + 1;
                continue;
            }
        }

        // Inline code: `text`
        if chars[i] == '`' {
            if let Some(end) = find_closing_single(&chars, i + 1, '`') {
                styles[i].is_syntax = true;
                styles[i].is_code = true;
                for j in (i + 1)..end {
                    styles[j].is_code = true;
                }
                styles[end].is_syntax = true;
                styles[end].is_code = true;
                i = end + 1;
                continue;
            }
        }

        i += 1;
    }

    styles
}

/// Detect heading level from line start. Returns None if not a heading.
fn detect_heading(chars: &[char]) -> Option<usize> {
    let mut level = 0;
    for &c in chars {
        if c == '#' {
            level += 1;
        } else if c == ' ' && level > 0 {
            return Some(level);
        } else {
            return None;
        }
    }
    None
}

/// Find closing double-char delimiter (e.g., **) starting from `start`.
/// Returns the index of the first char of the closing delimiter.
fn find_closing(chars: &[char], start: usize, delim: &[char; 2]) -> Option<usize> {
    let mut i = start;
    while i + 1 < chars.len() {
        if chars[i] == delim[0] && chars[i + 1] == delim[1] {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Find closing single-char delimiter starting from `start`.
fn find_closing_single(chars: &[char], start: usize, delim: char) -> Option<usize> {
    for i in start..chars.len() {
        if chars[i] == delim {
            return Some(i);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Acceptance test: Bold text is styled with visible syntax characters ===

    #[test]
    fn bold_text_has_bold_modifier_and_dimmed_syntax() {
        let styles = style_line("some **bold** text");
        // ** at positions 5,6
        assert!(styles[5].is_syntax);
        assert!(styles[6].is_syntax);
        assert!(styles[5].modifier.contains(Modifier::BOLD));
        // "bold" at positions 7,8,9,10
        assert!(!styles[7].is_syntax);
        assert!(styles[7].modifier.contains(Modifier::BOLD));
        // closing ** at 11,12
        assert!(styles[11].is_syntax);
        assert!(styles[12].is_syntax);
        // "text" is plain
        assert!(!styles[14].is_syntax);
        assert!(!styles[14].modifier.contains(Modifier::BOLD));
    }

    // === Acceptance test: Italic text is styled with visible syntax characters ===

    #[test]
    fn italic_text_has_italic_modifier_and_dimmed_syntax() {
        let styles = style_line("some *italic* text");
        // * at position 5
        assert!(styles[5].is_syntax);
        assert!(styles[5].modifier.contains(Modifier::ITALIC));
        // "italic" at positions 6-11
        assert!(!styles[6].is_syntax);
        assert!(styles[6].modifier.contains(Modifier::ITALIC));
        // closing * at 12
        assert!(styles[12].is_syntax);
        // "text" is plain
        assert!(!styles[14].modifier.contains(Modifier::ITALIC));
    }

    // === Acceptance test: Headings are styled with visible hash marks ===

    #[test]
    fn heading_has_bold_and_dimmed_hash() {
        let styles = style_line("## Section Title");
        // "## " (positions 0,1,2) are syntax
        assert!(styles[0].is_syntax);
        assert!(styles[1].is_syntax);
        assert!(styles[2].is_syntax);
        assert!(styles[0].is_heading);
        // "Section Title" (positions 3+) are heading text
        assert!(!styles[3].is_syntax);
        assert!(styles[3].is_heading);
        assert!(styles[3].modifier.contains(Modifier::BOLD));
    }

    // === Acceptance test: Code blocks styled as source text ===
    // (Fenced code blocks are multi-line — this tests inline code.
    //  Fenced block styling will be handled when we parse multi-line context.)

    #[test]
    fn inline_code_has_code_style_and_dimmed_backticks() {
        let styles = style_line("use `foo` here");
        // ` at position 4
        assert!(styles[4].is_syntax);
        assert!(styles[4].is_code);
        // "foo" at 5,6,7
        assert!(!styles[5].is_syntax);
        assert!(styles[5].is_code);
        // closing ` at 8
        assert!(styles[8].is_syntax);
        assert!(styles[8].is_code);
        // "here" is plain
        assert!(!styles[10].is_code);
    }

    // === Acceptance test: Markdown syntax is never removed from the Buffer ===

    #[test]
    fn style_line_returns_one_style_per_character() {
        let text = "**bold** and *italic*";
        let styles = style_line(text);
        assert_eq!(styles.len(), text.chars().count());
        // Every character in the original text has a style — nothing removed
    }

    // === Acceptance test: Fenced code block styling ===

    #[test]
    fn fence_line_renders_as_syntax_and_code() {
        let styles = style_line_with_context("```rust", false);
        for s in &styles {
            assert!(s.is_syntax, "Fence line chars should be syntax");
            assert!(s.is_code, "Fence line chars should be code");
        }
    }

    #[test]
    fn content_inside_fence_is_code_styled() {
        let styles = style_line_with_context("let x = 42;", true);
        for s in &styles {
            assert!(s.is_code, "Content inside fence should be code");
            assert!(!s.is_syntax, "Content inside fence should not be syntax");
        }
    }

    #[test]
    fn content_after_closing_fence_is_normal() {
        // After closing fence, in_code_block=false → normal parsing
        let styles = style_line_with_context("Just normal text", false);
        for s in &styles {
            assert!(!s.is_code);
            assert!(!s.is_syntax);
        }
    }

    #[test]
    fn heading_inside_code_block_is_not_styled_as_heading() {
        let styles = style_line_with_context("# Not a heading", true);
        for s in &styles {
            assert!(!s.is_heading, "Headings should not be detected inside code blocks");
            assert!(s.is_code, "Should be code-styled");
        }
    }

    // === Unit tests ===

    #[test]
    fn plain_text_has_no_syntax_markers() {
        let styles = style_line("just plain text");
        for s in &styles {
            assert!(!s.is_syntax);
            assert!(s.modifier.is_empty());
        }
    }

    #[test]
    fn unmatched_asterisk_is_plain() {
        let styles = style_line("a * b");
        // The * has no closing pair, so it's plain
        assert!(!styles[2].is_syntax);
    }

    #[test]
    fn heading_level_1() {
        let styles = style_line("# Title");
        assert!(styles[0].is_syntax); // #
        assert!(styles[1].is_syntax); // space
        assert!(styles[2].is_heading);
        assert!(!styles[2].is_syntax);
    }

    #[test]
    fn resolve_syntax_uses_dimmed_foreground() {
        let palette = Palette::default_palette();
        let s = CharStyle { is_syntax: true, ..Default::default() };
        let style = s.resolve(&palette);
        assert_eq!(style.fg.unwrap(), palette.dimmed_foreground);
    }

    #[test]
    fn resolve_heading_uses_accent_color() {
        let palette = Palette::default_palette();
        let s = CharStyle { is_heading: true, ..Default::default() };
        let style = s.resolve(&palette);
        assert_eq!(style.fg.unwrap(), palette.accent_heading);
    }

    #[test]
    fn resolve_plain_uses_foreground() {
        let palette = Palette::default_palette();
        let s = CharStyle::default();
        let style = s.resolve(&palette);
        assert_eq!(style.fg.unwrap(), palette.foreground);
    }
}
