use ropey::Rope;

/// The in-memory representation of a Document's text,
/// managed as a rope data structure.
#[derive(Debug, Clone)]
pub struct Buffer {
    rope: Rope,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            rope: Rope::new(),
        }
    }

    pub fn from_text(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
        }
    }

    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    pub fn rope_mut(&mut self) -> &mut Rope {
        &mut self.rope
    }

    /// Total number of characters in the buffer.
    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
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
    }

    /// Remove a range of characters.
    pub fn remove(&mut self, start: usize, end: usize) {
        self.rope.remove(start..end);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_buffer() {
        let buf = Buffer::new();
        assert_eq!(buf.len_chars(), 0);
        assert_eq!(buf.len_lines(), 1); // Ropey counts 1 line for empty
    }

    #[test]
    fn from_text() {
        let buf = Buffer::from_text("hello\nworld");
        assert_eq!(buf.len_lines(), 2);
        assert_eq!(buf.line(0).to_string(), "hello\n");
        assert_eq!(buf.line(1).to_string(), "world");
    }

    #[test]
    fn insert_and_remove() {
        let mut buf = Buffer::from_text("hello world");
        buf.insert(5, " beautiful");
        assert_eq!(buf.rope().to_string(), "hello beautiful world");
        buf.remove(5, 15);
        assert_eq!(buf.rope().to_string(), "hello world");
    }
}
