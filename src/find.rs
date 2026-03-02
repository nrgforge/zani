use crate::buffer::Buffer;

/// State for the find overlay.
pub struct FindState {
    /// The search query string.
    pub query: String,
    /// Cursor position within the query string.
    pub cursor: usize,
    /// All match positions as (line, col) of each match start.
    pub matches: Vec<(usize, usize)>,
    /// Index into `matches` for the current highlighted match.
    pub current_match: usize,
    /// Cursor position before find was opened (for cancel restore).
    pub saved_cursor: (usize, usize),
    /// Cached match ranges (line, start_col, end_col), populated by search().
    match_ranges_cache: Vec<(usize, usize, usize)>,
}

impl FindState {
    pub fn new(cursor_line: usize, cursor_col: usize) -> Self {
        Self {
            query: String::new(),
            cursor: 0,
            matches: Vec::new(),
            current_match: 0,
            saved_cursor: (cursor_line, cursor_col),
            match_ranges_cache: Vec::new(),
        }
    }

    /// Search the buffer for all occurrences of the query.
    pub fn search(&mut self, buffer: &Buffer) {
        self.matches.clear();
        self.current_match = 0;

        if self.query.is_empty() {
            self.match_ranges_cache.clear();
            return;
        }

        let query_lower = self.query.to_lowercase();
        for line_idx in 0..buffer.len_lines() {
            let line_text = buffer.line(line_idx).to_string();
            let line_lower = line_text.to_lowercase();
            let mut search_start = 0;
            while let Some(byte_pos) = line_lower[search_start..].find(&query_lower) {
                // Convert byte position to char position
                let char_col = line_lower[..search_start + byte_pos].chars().count();
                self.matches.push((line_idx, char_col));
                search_start += byte_pos + query_lower.len();
            }
        }

        // Populate match_ranges_cache
        let query_len = self.query.chars().count();
        self.match_ranges_cache.clear();
        self.match_ranges_cache.reserve(self.matches.len().saturating_sub(self.match_ranges_cache.capacity()));
        for &(line, col) in &self.matches {
            self.match_ranges_cache.push((line, col, col + query_len));
        }
    }

    /// Jump to the next match after the current cursor position.
    pub fn next_match(&mut self) {
        if !self.matches.is_empty() {
            self.current_match = (self.current_match + 1) % self.matches.len();
        }
    }

    /// Jump to the previous match.
    pub fn prev_match(&mut self) {
        if !self.matches.is_empty() {
            if self.current_match == 0 {
                self.current_match = self.matches.len() - 1;
            } else {
                self.current_match -= 1;
            }
        }
    }

    /// Get the position of the current match, if any.
    pub fn current_match_pos(&self) -> Option<(usize, usize)> {
        self.matches.get(self.current_match).copied()
    }

    /// Insert a character into the query at the cursor position.
    pub fn insert_char(&mut self, ch: char) {
        let byte_idx = self.query.char_indices()
            .nth(self.cursor)
            .map(|(i, _)| i)
            .unwrap_or(self.query.len());
        self.query.insert(byte_idx, ch);
        self.cursor += 1;
    }

    /// Delete the character before the cursor in the query.
    pub fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.cursor -= 1;
        let byte_idx = self.query.char_indices()
            .nth(self.cursor)
            .map(|(i, _)| i)
            .unwrap_or(self.query.len());
        let ch = self.query[byte_idx..].chars().next().unwrap();
        self.query.replace_range(byte_idx..byte_idx + ch.len_utf8(), "");
    }

    /// Return cached match ranges for highlighting: (line, start_col, end_col).
    pub fn match_ranges(&self) -> &[(usize, usize, usize)] {
        &self.match_ranges_cache
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_finds_nothing() {
        let buffer = Buffer::from_text("hello world\n");
        let mut fs = FindState::new(0, 0);
        fs.search(&buffer);
        assert!(fs.matches.is_empty());
    }

    #[test]
    fn finds_single_occurrence() {
        let buffer = Buffer::from_text("hello world\n");
        let mut fs = FindState::new(0, 0);
        fs.query = "world".to_string();
        fs.search(&buffer);
        assert_eq!(fs.matches.len(), 1);
        assert_eq!(fs.matches[0], (0, 6));
    }

    #[test]
    fn finds_multiple_occurrences() {
        let buffer = Buffer::from_text("the cat sat on the mat\n");
        let mut fs = FindState::new(0, 0);
        fs.query = "the".to_string();
        fs.search(&buffer);
        assert_eq!(fs.matches.len(), 2);
        assert_eq!(fs.matches[0], (0, 0));
        assert_eq!(fs.matches[1], (0, 15));
    }

    #[test]
    fn case_insensitive_search() {
        let buffer = Buffer::from_text("Hello HELLO hello\n");
        let mut fs = FindState::new(0, 0);
        fs.query = "hello".to_string();
        fs.search(&buffer);
        assert_eq!(fs.matches.len(), 3);
    }

    #[test]
    fn finds_across_lines() {
        let buffer = Buffer::from_text("first line\nsecond line\nthird line\n");
        let mut fs = FindState::new(0, 0);
        fs.query = "line".to_string();
        fs.search(&buffer);
        assert_eq!(fs.matches.len(), 3);
        assert_eq!(fs.matches[0].0, 0);
        assert_eq!(fs.matches[1].0, 1);
        assert_eq!(fs.matches[2].0, 2);
    }

    #[test]
    fn next_match_cycles() {
        let buffer = Buffer::from_text("aaa\n");
        let mut fs = FindState::new(0, 0);
        fs.query = "a".to_string();
        fs.search(&buffer);
        assert_eq!(fs.matches.len(), 3);
        assert_eq!(fs.current_match, 0);
        fs.next_match();
        assert_eq!(fs.current_match, 1);
        fs.next_match();
        assert_eq!(fs.current_match, 2);
        fs.next_match();
        assert_eq!(fs.current_match, 0); // wraps
    }

    #[test]
    fn prev_match_cycles() {
        let buffer = Buffer::from_text("aaa\n");
        let mut fs = FindState::new(0, 0);
        fs.query = "a".to_string();
        fs.search(&buffer);
        assert_eq!(fs.current_match, 0);
        fs.prev_match();
        assert_eq!(fs.current_match, 2); // wraps to end
    }

    #[test]
    fn insert_and_backspace_in_query() {
        let mut fs = FindState::new(0, 0);
        fs.insert_char('a');
        fs.insert_char('b');
        assert_eq!(fs.query, "ab");
        assert_eq!(fs.cursor, 2);
        fs.backspace();
        assert_eq!(fs.query, "a");
        assert_eq!(fs.cursor, 1);
    }

    #[test]
    fn match_ranges_returns_correct_ranges() {
        let buffer = Buffer::from_text("hello hello\n");
        let mut fs = FindState::new(0, 0);
        fs.query = "hello".to_string();
        fs.search(&buffer);
        let ranges = fs.match_ranges();
        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0], (0, 0, 5));
        assert_eq!(ranges[1], (0, 6, 11));
    }
}
