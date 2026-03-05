use std::fmt;

use ropey::Rope;

/// The in-memory representation of a Document's text,
/// managed as a rope data structure.
#[derive(Debug, Clone)]
pub struct Buffer {
    rope: Rope,
    version: u64,
}

impl fmt::Display for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for chunk in self.rope.chunks() {
            f.write_str(chunk)?;
        }
        Ok(())
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            rope: Rope::new(),
            version: 0,
        }
    }

    pub fn from_text(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
            version: 0,
        }
    }

    /// Monotonically increasing version, incremented on each mutation.
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Extract a slice of the buffer as a String.
    pub fn slice_to_string(&self, start: usize, end: usize) -> String {
        self.rope.slice(start..end).to_string()
    }

    /// Character index of the start of the given line (0-indexed).
    pub fn line_to_char(&self, line_idx: usize) -> usize {
        self.rope.line_to_char(line_idx)
    }

    /// Which line a character index falls on.
    pub fn char_to_line(&self, char_idx: usize) -> usize {
        self.rope.char_to_line(char_idx)
    }

    /// Iterate characters starting from a character index.
    pub fn chars_at(&self, char_idx: usize) -> ropey::iter::Chars<'_> {
        self.rope.chars_at(char_idx)
    }

    /// Get the character at a specific index.
    pub fn char_at(&self, char_idx: usize) -> char {
        self.rope.char(char_idx)
    }

    /// Total number of characters in the buffer.
    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    /// Whether the buffer contains any non-whitespace text.
    pub fn has_content(&self) -> bool {
        self.rope.chars().any(|c| !c.is_whitespace())
    }

    /// Total number of lines (newline-delimited) in the buffer.
    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }

    /// Get the text of a specific line (0-indexed).
    pub fn line(&self, idx: usize) -> ropey::RopeSlice<'_> {
        self.rope.line(idx)
    }

    /// Insert text at a character offset.
    pub fn insert(&mut self, char_idx: usize, text: &str) {
        self.rope.insert(char_idx, text);
        self.version += 1;
    }

    /// Remove a range of characters.
    pub fn remove(&mut self, start: usize, end: usize) {
        self.rope.remove(start..end);
        self.version += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_buffer() {
        let buf = Buffer::new();
        assert_eq!(buf.len_chars(), 0, "empty buffer should have 0 chars");
        assert_eq!(buf.len_lines(), 1, "empty buffer should have 1 line (ropey convention)");
    }

    #[test]
    fn from_text() {
        let buf = Buffer::from_text("hello\nworld");
        assert_eq!(buf.len_lines(), 2, "two-line text should have 2 lines");
        assert_eq!(buf.line(0).to_string(), "hello\n", "first line should include newline");
        assert_eq!(buf.line(1).to_string(), "world", "second line should be 'world'");
    }

    #[test]
    fn insert_and_remove() {
        let mut buf = Buffer::from_text("hello world");
        buf.insert(5, " beautiful");
        assert_eq!(buf.to_string(), "hello beautiful world", "insert should add text at position");
        buf.remove(5, 15);
        assert_eq!(buf.to_string(), "hello world", "remove should restore original text");
    }

    #[test]
    fn version_increments_on_insert() {
        let mut buf = Buffer::new();
        let v0 = buf.version();
        buf.insert(0, "hello");
        assert_eq!(buf.version(), v0 + 1, "version should increment after insert");
    }

    #[test]
    fn version_increments_on_remove() {
        let mut buf = Buffer::from_text("hello");
        let v0 = buf.version();
        buf.remove(0, 3);
        assert_eq!(buf.version(), v0 + 1, "version should increment after remove");
    }
}
