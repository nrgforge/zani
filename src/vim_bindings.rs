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
    /// Move cursor right (clamped), then switch to Insert mode.
    AppendMode,
    /// Move to end of line, then switch to Insert mode.
    AppendEndOfLine,
    /// Insert a character at the cursor.
    InsertChar(char),
    /// Insert a newline at the cursor.
    InsertNewline,
    /// Delete the character before the cursor.
    DeleteBack,
    /// Delete the character under the cursor.
    DeleteChar,
    /// Move cursor in a direction.
    MoveCursor(Direction),
    /// Move to start of line (column 0).
    LineStart,
    /// Move to end of line.
    LineEnd,
    /// Move to first line of file.
    GotoFirstLine,
    /// Move to last line of file.
    GotoLastLine,
    /// Delete the entire current line.
    DeleteLine,
    /// Word forward (next word start).
    WordForward,
    /// Word backward (previous word start).
    WordBackward,
    /// End of word (next word end).
    WordEnd,
    /// Open line below cursor and enter Insert mode.
    OpenLineBelow,
    /// Open line above cursor and enter Insert mode.
    OpenLineAbove,
    /// Enter Visual mode (set selection anchor).
    EnterVisual,
    /// Yank (copy) the current selection to register.
    Yank,
    /// Delete the current selection (yanks first, vim convention).
    DeleteSelection,
    /// Paste register content after cursor.
    PasteAfter,
    /// Paste register content before cursor.
    PasteBefore,
    /// Paste register content at cursor (replacing selection in Standard mode).
    PasteAtCursor,
    /// Undo the last change.
    Undo,
    /// Redo the last undone change.
    Redo,
    /// Select all text in the buffer.
    SelectAll,
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
        'a' => Action::AppendMode,
        'A' => Action::AppendEndOfLine,
        'h' => Action::MoveCursor(Direction::Left),
        'l' => Action::MoveCursor(Direction::Right),
        'j' => Action::MoveCursor(Direction::Down),
        'k' => Action::MoveCursor(Direction::Up),
        'w' => Action::WordForward,
        'b' => Action::WordBackward,
        'e' => Action::WordEnd,
        '0' => Action::LineStart,
        '$' => Action::LineEnd,
        'G' => Action::GotoLastLine,
        'x' => Action::DeleteChar,
        'o' => Action::OpenLineBelow,
        'O' => Action::OpenLineAbove,
        'v' => Action::EnterVisual,
        'p' => Action::PasteAfter,
        'P' => Action::PasteBefore,
        _ => Action::None,
    }
}

/// Process a key event in Visual mode.
/// Shares movement keys with Normal, plus yank and delete.
pub fn handle_visual(ch: char) -> Action {
    match ch {
        'h' => Action::MoveCursor(Direction::Left),
        'l' => Action::MoveCursor(Direction::Right),
        'j' => Action::MoveCursor(Direction::Down),
        'k' => Action::MoveCursor(Direction::Up),
        'w' => Action::WordForward,
        'b' => Action::WordBackward,
        'e' => Action::WordEnd,
        '0' => Action::LineStart,
        '$' => Action::LineEnd,
        'G' => Action::GotoLastLine,
        'y' => Action::Yank,
        'd' => Action::DeleteSelection,
        _ => Action::None,
    }
}

/// Process a Normal mode key with optional pending multi-key state.
/// Returns the action to perform and the new pending key (if any).
pub fn handle_normal_with_pending(ch: char, pending: Option<char>) -> (Action, Option<char>) {
    if let Some(p) = pending {
        match (p, ch) {
            ('g', 'g') => (Action::GotoFirstLine, None),
            ('d', 'd') => (Action::DeleteLine, None),
            _ => (Action::None, None),
        }
    } else if ch == 'g' || ch == 'd' {
        (Action::None, Some(ch))
    } else {
        (handle_normal(ch), None)
    }
}

/// Process a Visual mode key with optional pending multi-key state.
/// Returns the action to perform and the new pending key (if any).
pub fn handle_visual_with_pending(ch: char, pending: Option<char>) -> (Action, Option<char>) {
    if let Some(p) = pending {
        match (p, ch) {
            ('g', 'g') => (Action::GotoFirstLine, None),
            _ => (Action::None, None),
        }
    } else if ch == 'g' {
        (Action::None, Some(ch))
    } else {
        (handle_visual(ch), None)
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
    fn a_in_normal_returns_append_mode() {
        let action = handle_normal('a');
        assert_eq!(action, Action::AppendMode);
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
        assert_eq!(handle_normal('h'), Action::MoveCursor(Direction::Left), "h should move left");
        assert_eq!(handle_normal('j'), Action::MoveCursor(Direction::Down), "j should move down");
        assert_eq!(handle_normal('k'), Action::MoveCursor(Direction::Up), "k should move up");
        assert_eq!(handle_normal('l'), Action::MoveCursor(Direction::Right), "l should move right");
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

    #[test]
    fn visual_mode_movement_keys() {
        assert_eq!(handle_visual('h'), Action::MoveCursor(Direction::Left), "h should move left");
        assert_eq!(handle_visual('j'), Action::MoveCursor(Direction::Down), "j should move down");
        assert_eq!(handle_visual('k'), Action::MoveCursor(Direction::Up), "k should move up");
        assert_eq!(handle_visual('l'), Action::MoveCursor(Direction::Right), "l should move right");
        assert_eq!(handle_visual('w'), Action::WordForward, "w should move word forward");
        assert_eq!(handle_visual('b'), Action::WordBackward, "b should move word backward");
        assert_eq!(handle_visual('e'), Action::WordEnd, "e should move to word end");
        assert_eq!(handle_visual('0'), Action::LineStart, "0 should move to line start");
        assert_eq!(handle_visual('$'), Action::LineEnd, "$ should move to line end");
        assert_eq!(handle_visual('G'), Action::GotoLastLine, "G should go to last line");
    }

    #[test]
    fn visual_mode_yank_and_delete() {
        assert_eq!(handle_visual('y'), Action::Yank);
        assert_eq!(handle_visual('d'), Action::DeleteSelection);
    }

    #[test]
    fn v_in_normal_enters_visual() {
        assert_eq!(handle_normal('v'), Action::EnterVisual);
    }

    #[test]
    fn p_in_normal_returns_paste_after() {
        assert_eq!(handle_normal('p'), Action::PasteAfter);
    }

    #[test]
    fn big_p_in_normal_returns_paste_before() {
        assert_eq!(handle_normal('P'), Action::PasteBefore);
    }

    // === handle_normal_with_pending tests ===

    #[test]
    fn gg_returns_goto_first_line() {
        let (action, pending) = handle_normal_with_pending('g', Some('g'));
        assert_eq!(action, Action::GotoFirstLine, "gg should produce GotoFirstLine");
        assert_eq!(pending, None, "pending should be cleared after gg");
    }

    #[test]
    fn dd_returns_delete_line() {
        let (action, pending) = handle_normal_with_pending('d', Some('d'));
        assert_eq!(action, Action::DeleteLine, "dd should produce DeleteLine");
        assert_eq!(pending, None, "pending should be cleared after dd");
    }

    #[test]
    fn g_alone_sets_pending() {
        let (action, pending) = handle_normal_with_pending('g', None);
        assert_eq!(action, Action::None, "first g should produce no action");
        assert_eq!(pending, Some('g'), "first g should set pending");
    }

    #[test]
    fn unknown_second_key_clears_pending() {
        let (action, pending) = handle_normal_with_pending('z', Some('g'));
        assert_eq!(action, Action::None, "unknown second key should produce no action");
        assert_eq!(pending, None, "unknown second key should clear pending");
    }

    #[test]
    fn visual_gg_returns_goto_first_line() {
        let (action, pending) = handle_visual_with_pending('g', Some('g'));
        assert_eq!(action, Action::GotoFirstLine, "visual gg should produce GotoFirstLine");
        assert_eq!(pending, None, "pending should be cleared after visual gg");
    }
}
