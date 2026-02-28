/// A single buffer operation that can be undone/redone.
#[derive(Debug, Clone)]
pub enum Operation {
    /// Text was inserted at the given char position.
    Insert { pos: usize, text: String },
    /// Text was deleted starting at the given char position.
    Delete { pos: usize, text: String },
}

/// Undo/redo history with grouped operations.
///
/// Operations are accumulated into a "current group" and sealed into the undo
/// stack at natural boundaries (whitespace insertion, newlines, mode switches).
/// Each undo step reverses an entire group.
#[derive(Default)]
pub struct UndoHistory {
    undo_stack: Vec<Vec<Operation>>,
    redo_stack: Vec<Vec<Operation>>,
    current_group: Vec<Operation>,
}

impl UndoHistory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an insert operation into the current group.
    pub fn record_insert(&mut self, pos: usize, text: &str) {
        self.current_group.push(Operation::Insert {
            pos,
            text: text.to_string(),
        });
    }

    /// Record a delete operation into the current group.
    pub fn record_delete(&mut self, pos: usize, text: &str) {
        self.current_group.push(Operation::Delete {
            pos,
            text: text.to_string(),
        });
    }

    /// Seal the current group into the undo stack and clear the redo stack.
    /// No-op if the current group is empty.
    pub fn commit_group(&mut self) {
        if !self.current_group.is_empty() {
            let group = std::mem::take(&mut self.current_group);
            self.undo_stack.push(group);
            self.redo_stack.clear();
        }
    }

    /// Pop the last group from the undo stack and return its operations
    /// (in reverse order, ready to be inverted and applied).
    /// Pushes the group onto the redo stack.
    pub fn undo(&mut self) -> Option<Vec<Operation>> {
        let group = self.undo_stack.pop()?;
        self.redo_stack.push(group.clone());
        Some(group)
    }

    /// Pop the last group from the redo stack and return its operations
    /// (in original order, ready to be re-applied).
    /// Pushes the group back onto the undo stack.
    pub fn redo(&mut self) -> Option<Vec<Operation>> {
        let group = self.redo_stack.pop()?;
        self.undo_stack.push(group.clone());
        Some(group)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_history_undo_is_none() {
        let mut h = UndoHistory::new();
        assert!(h.undo().is_none());
    }

    #[test]
    fn empty_history_redo_is_none() {
        let mut h = UndoHistory::new();
        assert!(h.redo().is_none());
    }

    #[test]
    fn commit_empty_group_is_noop() {
        let mut h = UndoHistory::new();
        h.commit_group();
        assert!(h.undo().is_none());
    }

    #[test]
    fn undo_returns_committed_group() {
        let mut h = UndoHistory::new();
        h.record_insert(0, "a");
        h.record_insert(1, "b");
        h.commit_group();
        let ops = h.undo().unwrap();
        assert_eq!(ops.len(), 2);
    }

    #[test]
    fn undo_then_redo_returns_same_group() {
        let mut h = UndoHistory::new();
        h.record_insert(0, "hello");
        h.commit_group();
        let undone = h.undo().unwrap();
        let redone = h.redo().unwrap();
        assert_eq!(undone.len(), redone.len());
    }

    #[test]
    fn new_edit_after_undo_clears_redo() {
        let mut h = UndoHistory::new();
        h.record_insert(0, "a");
        h.commit_group();
        h.undo();
        // New edit
        h.record_insert(0, "b");
        h.commit_group();
        // Redo stack should be cleared by the commit
        assert!(h.redo().is_none());
    }

    #[test]
    fn multiple_undos_walk_back() {
        let mut h = UndoHistory::new();
        h.record_insert(0, "a");
        h.commit_group();
        h.record_insert(1, "b");
        h.commit_group();
        h.record_insert(2, "c");
        h.commit_group();

        let g3 = h.undo().unwrap();
        assert_eq!(g3.len(), 1);
        let g2 = h.undo().unwrap();
        assert_eq!(g2.len(), 1);
        let g1 = h.undo().unwrap();
        assert_eq!(g1.len(), 1);
        assert!(h.undo().is_none());
    }
}
