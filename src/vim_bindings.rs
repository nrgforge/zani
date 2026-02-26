/// Vim editing mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
}

/// Cursor shape corresponding to the current mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorShape {
    /// Vertical bar — used in Insert mode.
    Bar,
    /// Solid block — used in Normal and Visual modes.
    Block,
}

impl Mode {
    /// The cursor shape for this mode.
    pub fn cursor_shape(self) -> CursorShape {
        match self {
            Mode::Insert => CursorShape::Bar,
            Mode::Normal | Mode::Visual => CursorShape::Block,
        }
    }
}

/// Result of processing a key in the current mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Switch to a different mode.
    SwitchMode(Mode),
    /// Insert a character at the cursor.
    InsertChar(char),
    /// Insert a newline at the cursor.
    InsertNewline,
    /// Delete the character before the cursor.
    DeleteBack,
    /// Move cursor in a direction.
    MoveCursor(Direction),
    /// No action (key not handled).
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

/// Process a key event in Normal mode.
pub fn handle_normal(ch: char) -> Action {
    match ch {
        'i' => Action::SwitchMode(Mode::Insert),
        'a' => Action::SwitchMode(Mode::Insert), // TODO: move cursor right first
        'h' => Action::MoveCursor(Direction::Left),
        'l' => Action::MoveCursor(Direction::Right),
        'j' => Action::MoveCursor(Direction::Down),
        'k' => Action::MoveCursor(Direction::Up),
        _ => Action::None,
    }
}

/// Process a key event in Insert mode.
/// Returns None for Escape (caller handles mode switch).
pub fn handle_insert(ch: char) -> Action {
    match ch {
        '\x1b' => Action::SwitchMode(Mode::Normal), // Escape
        '\n' | '\r' => Action::InsertNewline,
        '\x7f' | '\x08' => Action::DeleteBack, // Backspace
        c if !c.is_control() => Action::InsertChar(c),
        _ => Action::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Acceptance test: Insert mode uses bar cursor ===

    #[test]
    fn insert_mode_has_bar_cursor() {
        assert_eq!(Mode::Insert.cursor_shape(), CursorShape::Bar);
    }

    // === Acceptance test: Normal mode uses block cursor ===

    #[test]
    fn normal_mode_has_block_cursor() {
        assert_eq!(Mode::Normal.cursor_shape(), CursorShape::Block);
    }

    // === Acceptance test: Mode switch between normal and insert ===

    #[test]
    fn i_in_normal_switches_to_insert() {
        let action = handle_normal('i');
        assert_eq!(action, Action::SwitchMode(Mode::Insert));
    }

    #[test]
    fn escape_in_insert_switches_to_normal() {
        let action = handle_insert('\x1b');
        assert_eq!(action, Action::SwitchMode(Mode::Normal));
    }

    #[test]
    fn typing_in_insert_produces_insert_char() {
        let action = handle_insert('x');
        assert_eq!(action, Action::InsertChar('x'));
    }

    // === Unit tests: movement ===

    #[test]
    fn hjkl_movement_in_normal() {
        assert_eq!(handle_normal('h'), Action::MoveCursor(Direction::Left));
        assert_eq!(handle_normal('j'), Action::MoveCursor(Direction::Down));
        assert_eq!(handle_normal('k'), Action::MoveCursor(Direction::Up));
        assert_eq!(handle_normal('l'), Action::MoveCursor(Direction::Right));
    }

    #[test]
    fn unbound_key_in_normal_is_none() {
        assert_eq!(handle_normal('z'), Action::None);
    }

    #[test]
    fn enter_in_insert_is_newline() {
        assert_eq!(handle_insert('\n'), Action::InsertNewline);
    }

    #[test]
    fn backspace_in_insert_is_delete_back() {
        assert_eq!(handle_insert('\x7f'), Action::DeleteBack);
    }

    #[test]
    fn visual_mode_has_block_cursor() {
        assert_eq!(Mode::Visual.cursor_shape(), CursorShape::Block);
    }
}
