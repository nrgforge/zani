//! Pure event-translation layer for mouse input.
//!
//! Translates crossterm `MouseEvent` into a small `MouseAction` enum that the
//! `App` coordinator routes to the editor. Multi-click detection is the only
//! state this module carries, threaded by the caller via `LastClick`.

use std::time::{Duration, Instant};

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

pub const SCROLL_LINES_PER_NOTCH: i16 = 3;
pub const MULTI_CLICK_WINDOW_MS: u64 = 400;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseAction {
    ScrollLines(i16),
    ClickAt { row: u16, col: u16, click_count: u8 },
    DragTo { row: u16, col: u16 },
    Release,
}

#[derive(Debug, Clone, Copy)]
pub struct LastClick {
    pub instant: Instant,
    pub row: u16,
    pub col: u16,
    pub count: u8,
}

pub fn translate(
    event: MouseEvent,
    last: Option<LastClick>,
    now: Instant,
) -> (Option<MouseAction>, Option<LastClick>) {
    match event.kind {
        MouseEventKind::ScrollUp => (Some(MouseAction::ScrollLines(-SCROLL_LINES_PER_NOTCH)), last),
        MouseEventKind::ScrollDown => (Some(MouseAction::ScrollLines(SCROLL_LINES_PER_NOTCH)), last),
        MouseEventKind::Down(MouseButton::Left) => {
            let window = Duration::from_millis(MULTI_CLICK_WINDOW_MS);
            let count = match last {
                Some(prev)
                    if prev.row == event.row
                        && prev.col == event.column
                        && now.duration_since(prev.instant) <= window =>
                {
                    (prev.count + 1).min(3)
                }
                _ => 1,
            };
            let next = LastClick {
                instant: now,
                row: event.row,
                col: event.column,
                count,
            };
            (
                Some(MouseAction::ClickAt {
                    row: event.row,
                    col: event.column,
                    click_count: count,
                }),
                Some(next),
            )
        }
        MouseEventKind::Drag(MouseButton::Left) => (
            Some(MouseAction::DragTo {
                row: event.row,
                col: event.column,
            }),
            last,
        ),
        MouseEventKind::Up(MouseButton::Left) => (Some(MouseAction::Release), last),
        _ => (None, last),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyModifiers, MouseEvent};
    use std::time::Duration;

    fn evt(kind: MouseEventKind, row: u16, col: u16) -> MouseEvent {
        MouseEvent {
            kind,
            row,
            column: col,
            modifiers: KeyModifiers::NONE,
        }
    }

    #[test]
    fn scroll_up_translates_to_negative_scroll() {
        let now = Instant::now();
        let (action, _) = translate(evt(MouseEventKind::ScrollUp, 0, 0), None, now);
        assert_eq!(action, Some(MouseAction::ScrollLines(-SCROLL_LINES_PER_NOTCH)));
    }

    #[test]
    fn scroll_down_translates_to_positive_scroll() {
        let now = Instant::now();
        let (action, _) = translate(evt(MouseEventKind::ScrollDown, 0, 0), None, now);
        assert_eq!(action, Some(MouseAction::ScrollLines(SCROLL_LINES_PER_NOTCH)));
    }

    #[test]
    fn first_click_has_count_1() {
        let now = Instant::now();
        let (action, last) = translate(
            evt(MouseEventKind::Down(MouseButton::Left), 5, 10),
            None,
            now,
        );
        assert_eq!(
            action,
            Some(MouseAction::ClickAt { row: 5, col: 10, click_count: 1 })
        );
        assert_eq!(last.unwrap().count, 1);
    }

    #[test]
    fn same_position_within_window_increments_count() {
        let t0 = Instant::now();
        let (_, last) = translate(
            evt(MouseEventKind::Down(MouseButton::Left), 5, 10),
            None,
            t0,
        );
        let t1 = t0 + Duration::from_millis(100);
        let (action, last) = translate(
            evt(MouseEventKind::Down(MouseButton::Left), 5, 10),
            last,
            t1,
        );
        assert_eq!(
            action,
            Some(MouseAction::ClickAt { row: 5, col: 10, click_count: 2 })
        );
        let t2 = t1 + Duration::from_millis(100);
        let (action, _) = translate(
            evt(MouseEventKind::Down(MouseButton::Left), 5, 10),
            last,
            t2,
        );
        assert_eq!(
            action,
            Some(MouseAction::ClickAt { row: 5, col: 10, click_count: 3 })
        );
    }

    #[test]
    fn different_column_resets_count() {
        let t0 = Instant::now();
        let (_, last) = translate(
            evt(MouseEventKind::Down(MouseButton::Left), 5, 10),
            None,
            t0,
        );
        let t1 = t0 + Duration::from_millis(100);
        let (action, _) = translate(
            evt(MouseEventKind::Down(MouseButton::Left), 5, 11),
            last,
            t1,
        );
        assert_eq!(
            action,
            Some(MouseAction::ClickAt { row: 5, col: 11, click_count: 1 })
        );
    }

    #[test]
    fn click_after_window_expires_resets_count() {
        let t0 = Instant::now();
        let (_, last) = translate(
            evt(MouseEventKind::Down(MouseButton::Left), 5, 10),
            None,
            t0,
        );
        let t1 = t0 + Duration::from_millis(MULTI_CLICK_WINDOW_MS + 50);
        let (action, _) = translate(
            evt(MouseEventKind::Down(MouseButton::Left), 5, 10),
            last,
            t1,
        );
        assert_eq!(
            action,
            Some(MouseAction::ClickAt { row: 5, col: 10, click_count: 1 })
        );
    }

    #[test]
    fn drag_translates_to_drag_to() {
        let now = Instant::now();
        let (action, _) = translate(
            evt(MouseEventKind::Drag(MouseButton::Left), 7, 20),
            None,
            now,
        );
        assert_eq!(action, Some(MouseAction::DragTo { row: 7, col: 20 }));
    }

    #[test]
    fn release_translates_to_release() {
        let now = Instant::now();
        let (action, _) = translate(
            evt(MouseEventKind::Up(MouseButton::Left), 7, 20),
            None,
            now,
        );
        assert_eq!(action, Some(MouseAction::Release));
    }

    #[test]
    fn right_click_is_ignored() {
        let now = Instant::now();
        let (action, _) = translate(
            evt(MouseEventKind::Down(MouseButton::Right), 0, 0),
            None,
            now,
        );
        assert_eq!(action, None);
    }
}
