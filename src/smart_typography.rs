/// Apply smart typography transformations to a character being inserted,
/// given the text preceding it (for context).
///
/// Returns the replacement string if a transformation applies,
/// or None if the character should be inserted as-is.
pub fn transform(ch: char, preceding: &str) -> Option<SmartEdit> {
    match ch {
        '"' => Some(smart_double_quote(preceding)),
        '\'' => smart_single_quote(preceding),
        '-' => smart_dash(preceding),
        '.' => smart_ellipsis(preceding),
        _ => None,
    }
}

/// A smart typography edit: replace some characters before the cursor
/// and insert new text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmartEdit {
    /// Number of characters before the insertion point to remove.
    pub delete_before: usize,
    /// Text to insert in their place.
    pub insert: &'static str,
}

fn smart_double_quote(preceding: &str) -> SmartEdit {
    // Opening quote if preceded by nothing, whitespace, or opening punctuation
    let is_opening = preceding.is_empty()
        || preceding.ends_with(|c: char| c.is_whitespace() || "([{".contains(c));

    SmartEdit {
        delete_before: 0,
        insert: if is_opening { "\u{201C}" } else { "\u{201D}" }, // " or "
    }
}

fn smart_single_quote(preceding: &str) -> Option<SmartEdit> {
    // Opening quote if preceded by nothing, whitespace, or opening punctuation
    let is_opening = preceding.is_empty()
        || preceding.ends_with(|c: char| c.is_whitespace() || "([{".contains(c));

    Some(SmartEdit {
        delete_before: 0,
        insert: if is_opening { "\u{2018}" } else { "\u{2019}" }, // ' or '
    })
}

fn smart_dash(preceding: &str) -> Option<SmartEdit> {
    // Two hyphens → em dash
    if preceding.ends_with('-') {
        Some(SmartEdit {
            delete_before: 1, // remove the preceding hyphen
            insert: "\u{2014}", // —
        })
    } else {
        None
    }
}

fn smart_ellipsis(preceding: &str) -> Option<SmartEdit> {
    // Three periods → ellipsis
    if preceding.ends_with("..") {
        Some(SmartEdit {
            delete_before: 2, // remove the two preceding periods
            insert: "\u{2026}", // …
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Acceptance test: Straight double quotes convert to curly quotes ===

    #[test]
    fn opening_double_quote_after_space() {
        let edit = transform('"', "He said ").unwrap();
        assert_eq!(edit.delete_before, 0);
        assert_eq!(edit.insert, "\u{201C}"); // "
    }

    #[test]
    fn closing_double_quote_after_word() {
        let edit = transform('"', "hello").unwrap();
        assert_eq!(edit.delete_before, 0);
        assert_eq!(edit.insert, "\u{201D}"); // "
    }

    #[test]
    fn opening_double_quote_at_start() {
        let edit = transform('"', "").unwrap();
        assert_eq!(edit.insert, "\u{201C}");
    }

    // === Acceptance test: Double hyphen converts to em dash ===

    #[test]
    fn double_hyphen_becomes_em_dash() {
        let edit = transform('-', "word-").unwrap();
        assert_eq!(edit.delete_before, 1);
        assert_eq!(edit.insert, "\u{2014}"); // —
    }

    #[test]
    fn single_hyphen_passes_through() {
        let edit = transform('-', "word");
        assert!(edit.is_none());
    }

    // === Acceptance test: Triple period converts to ellipsis ===

    #[test]
    fn triple_period_becomes_ellipsis() {
        let edit = transform('.', "wait..").unwrap();
        assert_eq!(edit.delete_before, 2);
        assert_eq!(edit.insert, "\u{2026}"); // …
    }

    #[test]
    fn single_period_passes_through() {
        let edit = transform('.', "word");
        assert!(edit.is_none());
    }

    #[test]
    fn double_period_passes_through() {
        let edit = transform('.', "word.");
        assert!(edit.is_none());
    }

    // === Unit tests: single quotes ===

    #[test]
    fn opening_single_quote_after_space() {
        let edit = transform('\'', "it ").unwrap();
        assert_eq!(edit.insert, "\u{2018}"); // '
    }

    #[test]
    fn closing_single_quote_after_word() {
        let edit = transform('\'', "don").unwrap();
        assert_eq!(edit.insert, "\u{2019}"); // '  (also serves as apostrophe)
    }
}
