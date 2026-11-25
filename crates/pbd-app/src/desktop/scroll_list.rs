//! A list shown through a window of whole rows (`townsfolk` design 38b):
//! which rows show, the wheel's whole-row steps, the scroll bar's thumb and
//! its drag, and the least scroll that shows a row. Only the maths: the
//! panel that uses it builds the rows in [`ListView::window`] and draws the
//! bar with [`bar`]. Every length is native px.

use bevy::prelude::*;
use pbd_app::ui_kit::{self, KitPiece, Set, UiKit};

/// The scroll bar's width, native px: the stone `slider-track` on end.
pub const BAR_W: f32 = 8.0;
/// The track's inset round the thumb, native px.
pub const BAR_INSET: f32 = 2.0;
/// The thumb's width, native px (`townsfolk` task 12.2): 6 inside the 8 px
/// track, a px in from each side, so it can be caught at 1x (it was 4).
pub const THUMB_W: f32 = 6.0;
/// The shortest thumb, native px, however long the list.
pub const MIN_THUMB: f32 = 8.0;

/// A list of `rows` seen through a window of `shown`, from row `first`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ListView {
    pub rows: usize,
    pub shown: usize,
    pub first: usize,
}

impl ListView {
    /// A view, its first row clamped so the window is never short at the
    /// bottom.
    pub fn new(rows: usize, shown: usize, first: usize) -> Self {
        let mut view = Self { rows, shown, first };
        view.first = first.min(view.max_first());
        view
    }

    /// The last row that can be first.
    pub fn max_first(&self) -> usize {
        self.rows.saturating_sub(self.shown)
    }

    /// Whether the rows overflow the window: the bar shows only then.
    pub fn overflows(&self) -> bool {
        self.rows > self.shown
    }

    /// The rows shown, as indices.
    pub fn window(&self) -> std::ops::Range<usize> {
        self.first..(self.first + self.shown).min(self.rows)
    }

    /// Scrolled to `first`, clamped.
    pub fn at(&self, first: usize) -> Self {
        Self::new(self.rows, self.shown, first)
    }

    /// The wheel turned `notches` (up positive, as Bevy reads it): one row a
    /// notch, the list moving down the rows as the wheel turns toward you.
    pub fn wheel(&self, notches: i32) -> Self {
        let first = self.first as i64 - notches as i64;
        self.at(first.max(0) as usize)
    }

    /// A press on the track above (`down` false) or below the thumb: a
    /// window's worth of rows.
    pub fn page(&self, down: bool) -> Self {
        if down {
            self.at(self.first + self.shown)
        } else {
            self.at(self.first.saturating_sub(self.shown))
        }
    }

    /// The least scroll that shows `row`: a hidden row above becomes the
    /// first, one below the last; a shown row leaves it where it is. For
    /// keyboard or controller focus moving onto a hidden row, which the
    /// talk panels do not have yet (design 38b), so only its test calls it.
    #[allow(dead_code)]
    pub fn reveal(&self, row: usize) -> Self {
        if row < self.first {
            self.at(row)
        } else if row >= self.first + self.shown {
            self.at(row + 1 - self.shown)
        } else {
            *self
        }
    }

    /// The thumb in a track `track` long: its top and its length. The
    /// length is the shown share of the list (never under [`MIN_THUMB`]),
    /// the top the first row's share of the rows that can be first.
    pub fn thumb(&self, track: f32) -> (f32, f32) {
        if !self.overflows() || self.rows == 0 {
            return (0.0, track);
        }
        let len = (track * self.shown as f32 / self.rows as f32)
            .max(MIN_THUMB)
            .min(track);
        let top = (track - len) * self.first as f32 / self.max_first() as f32;
        (top, len)
    }

    /// The thumb dragged so its top is at `top` in a track `track` long:
    /// the nearest whole row.
    pub fn drag(&self, track: f32, top: f32) -> Self {
        let (_, len) = self.thumb(track);
        let travel = track - len;
        if travel <= 0.0 {
            return *self;
        }
        let share = (top / travel).clamp(0.0, 1.0);
        self.at((share * self.max_first() as f32).round() as usize)
    }
}

/// Whole notches from wheel motion: a line unit is a notch; pixels are
/// gathered in `carry` until a row's pitch (`pitch`) of them has turned.
pub fn notches(y: f32, pixels: bool, pitch: f32, carry: &mut f32) -> i32 {
    if !pixels {
        return y.round() as i32;
    }
    *carry += y / pitch;
    let whole = carry.trunc();
    *carry -= whole;
    whole as i32
}

/// The scroll bar of a list, a `track` long: the stone `slider-track` on
/// end, with a thumb in the rune fill the settings sliders use (the kit has
/// no stone thumb piece). Only when the list overflows; otherwise the same
/// width left empty, so the rows do not move. `track_extra` and
/// `thumb_extra` are what the panel tells them apart and presses by.
pub fn bar(
    parent: &mut ChildSpawnerCommands,
    kit: &UiKit,
    view: ListView,
    track: f32,
    held: bool,
    track_extra: impl Bundle,
    thumb_extra: impl Bundle,
) {
    if !view.overflows() {
        parent.spawn(Node {
            width: px(BAR_W),
            flex_shrink: 0.0,
            ..default()
        });
        return;
    }
    let inner = track - 2.0 * BAR_INSET;
    let (top, len) = view.thumb(inner);
    parent
        .spawn((
            Button,
            kit.nine(Set::Stone, "slider-track"),
            KitPiece(Set::Stone.key("slider-track")),
            Node {
                width: px(BAR_W),
                height: px(track),
                flex_shrink: 0.0,
                ..default()
            },
            bevy::ui::RelativeCursorPosition::default(),
            track_extra,
        ))
        .with_children(|t| {
            t.spawn((
                Button,
                Node {
                    position_type: PositionType::Absolute,
                    left: px((BAR_W - THUMB_W) / 2.0),
                    width: px(THUMB_W),
                    top: px(BAR_INSET + top),
                    height: px(len),
                    ..default()
                },
                BackgroundColor(if held { ui_kit::RUNE } else { ui_kit::RUNE_DIM }),
                thumb_extra,
            ));
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The thumb is wider than the 4 px it was, and inside its track.
    #[test]
    fn the_thumb_is_wide_enough_to_catch() {
        const { assert!(THUMB_W > 4.0 && THUMB_W < BAR_W) };
    }

    /// The window: the rows from the first, never past the end; clamped so
    /// it is never short at the bottom; no overflow, no bar.
    #[test]
    fn the_window_shows_whole_rows_and_is_clamped() {
        let v = ListView::new(12, 7, 0);
        assert!(v.overflows());
        assert_eq!(v.window(), 0..7);
        assert_eq!(v.max_first(), 5);
        assert_eq!(v.at(3).window(), 3..10);
        assert_eq!(
            v.at(9).first,
            5,
            "clamped: the window is full at the bottom"
        );
        assert_eq!(v.at(9).window(), 5..12);
        let short = ListView::new(4, 7, 3);
        assert!(!short.overflows());
        assert_eq!(short.first, 0);
        assert_eq!(short.window(), 0..4);
        assert_eq!(ListView::new(0, 7, 2).window(), 0..0);
        // A list that shrinks under a scrolled window clamps.
        assert_eq!(ListView::new(8, 7, 5).first, 1);
    }

    /// The wheel moves whole rows, a row a notch, and stops at both ends;
    /// pixel motion gathers to whole rows.
    #[test]
    fn the_wheel_scrolls_by_whole_rows() {
        let v = ListView::new(12, 7, 0);
        assert_eq!(v.wheel(-1).first, 1, "toward you: down the list");
        assert_eq!(v.wheel(-3).first, 3);
        assert_eq!(v.wheel(1).first, 0, "stops at the top");
        assert_eq!(v.wheel(-40).first, 5, "stops at the bottom");
        assert_eq!(v.at(4).wheel(2).first, 2);
        assert_eq!(v.page(true).first, 5);
        assert_eq!(v.at(5).page(false).first, 0);
        let mut carry = 0.0;
        assert_eq!(notches(-10.0, true, 26.0, &mut carry), 0);
        assert_eq!(notches(-20.0, true, 26.0, &mut carry), -1);
        assert!((carry - (-4.0 / 26.0)).abs() < 1e-5);
        assert_eq!(notches(-2.0, false, 26.0, &mut carry), -2);
    }

    /// The thumb's length is the shown share, never under the least, and
    /// its top the first row's share; a drag snaps to the nearest row and
    /// round-trips every first row.
    #[test]
    fn the_thumb_is_the_shown_share_and_drags_to_whole_rows() {
        let track = 176.0;
        let v = ListView::new(14, 7, 0);
        let (top, len) = v.thumb(track);
        assert_eq!(top, 0.0);
        assert!(
            (len - 88.0).abs() < 1e-4,
            "half the list shows: half the track"
        );
        let (top, len) = v.at(7).thumb(track);
        assert!(
            (top + len - track).abs() < 1e-4,
            "at the bottom, the thumb ends where the track does"
        );
        let (_, len) = ListView::new(1000, 7, 0).thumb(track);
        assert_eq!(len, MIN_THUMB);
        assert_eq!(ListView::new(5, 7, 0).thumb(track), (0.0, track));
        for first in 0..=v.max_first() {
            let (top, _) = v.at(first).thumb(track);
            assert_eq!(v.drag(track, top).first, first, "row {first}");
            assert_eq!(v.drag(track, top + 2.0).first, first, "snaps back");
        }
        assert_eq!(v.drag(track, -30.0).first, 0);
        assert_eq!(v.drag(track, 999.0).first, 7);
    }

    /// Revealing a row scrolls the least that shows it.
    #[test]
    fn revealing_a_row_scrolls_the_least() {
        let v = ListView::new(20, 7, 5);
        assert_eq!(v.reveal(6).first, 5, "shown: no scroll");
        assert_eq!(v.reveal(2).first, 2, "above: it becomes the first");
        assert_eq!(v.reveal(14).first, 8, "below: it becomes the last");
        assert_eq!(v.reveal(19).first, 13);
    }
}
