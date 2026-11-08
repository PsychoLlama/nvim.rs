//! The lines that have scrolled off the top of a `:terminal`.
//!
//! The emulator's screen holds only as many rows as the window is tall.
//! Everything above that lives here, oldest last, capped by `'scrollback'`.
//! vterm hands rows over as they scroll away ([`scrollback_push`]) and asks for
//! them back when the screen grows or leaves the alternate screen
//! ([`scrollback_pop`]); the refresh path then mirrors what is stored into the
//! buffer's lines.
//!
//! Two numbers connect this to the buffer. `pending` counts rows pushed but
//! not yet appended to the buffer, and `deleted` counts rows evicted over
//! the terminal's whole life — buffer line numbers are only meaningful
//! relative to it, which is why [`super::row_to_linenr`] exists.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::change::{appended_lines_buf, deleted_lines_buf};
use crate::extmark::kExtmarkUndo;
use crate::grid::{MAX_SCHAR_SIZE, schar_put_scratch};
use crate::mark::kMarkAdjustTerm;
use crate::mark::mark_adjust_buf;
use crate::memline::{ml_append_buf_terminated, ml_delete_buf};
use crate::narrow::as_index;
use crate::narrow::count_as_int;
use crate::narrow::len_as_int;
use crate::types::{LineNr, OptInt};
use crate::vterm::screen::ZERO_SCREEN_CELL;
use crate::vterm::types::{VTermPos, VTermScreenCell};
use crate::winlayer::Buf;
use core::ffi::c_int;

use super::refresh::invalidate_terminal;
use super::{SB_MAX, Term};
use crate::pos::MAXLNUM;

/// A cell holding nothing. vterm reports an empty cell as a zero `schar`;
/// the width still has to be 1 or the row scan below would not advance.
fn blank_cell() -> VTermScreenCell {
    VTermScreenCell {
        width: 1,
        ..ZERO_SCREEN_CELL
    }
}

/// Size the scrollback from `'scrollback'`, if it has not been sized yet.
///
/// Returns false only when there is no buffer to read the option from,
/// which happens if the buffer was wiped while the terminal still lives.
pub(crate) fn term_may_alloc_scrollback(mut term: Term, buffer: Option<Buf>) -> bool {
    if term.sb.is_sized() {
        return true;
    }
    let Some(buf) = buffer.or_else(|| super::term_buf(term)) else {
        return false;
    };
    term.sb.set_capacity(scrollback_limit(buf));
    true
}

/// `'scrollback'` as a row count. The option's "unlimited" spelling is a
/// negative value, which stands for a cap large enough never to be reached.
fn scrollback_limit(mut buffer: Buf) -> usize {
    if buffer.b_p_scbk < 1 {
        buffer.b_p_scbk = OptInt::from(SB_MAX);
    }
    usize::try_from(buffer.b_p_scbk).expect("'scrollback' is positive here")
}

/// A row scrolled off the top of the screen.
pub(crate) fn scrollback_push(mut term: Term, row: &[VTermScreenCell]) -> c_int {
    if !term_may_alloc_scrollback(term, None) {
        return 0;
    }
    term.sb.push(row);
    if !term.synchronized_output {
        invalidate_terminal(term, None);
    }
    1
}

/// The screen grew: give back the row most recently scrolled off, into
/// `row`.
pub(crate) fn scrollback_pop(mut term: Term, row: &mut [VTermScreenCell]) -> c_int {
    let mut old_height = term.old_height;
    if !term.sb.pop(row, &mut old_height) {
        return 0;
    }
    term.old_height = old_height;
    if !term.synchronized_output {
        invalidate_terminal(term, None);
    }
    1
}

/// The child asked for the scrollback to go.
pub(crate) fn scrollback_clear(mut term: Term) -> c_int {
    // On the alternate screen the scrollback belongs to the screen
    // underneath, which is about to be restored; clearing it there would
    // lose history the program never touched.
    if term.in_altscreen || !term.sb.is_sized() || term.sb.is_empty() {
        return 1;
    }
    term.sb.clear();
    invalidate_terminal(term, None);
    1
}

/// Render row `row` of the screen into the terminal's line buffer.
///
/// Negative rows come from the scrollback, counting back from -1. Cells
/// vterm reports as empty become spaces, and trailing spaces are dropped by
/// terminating at the last cell that actually held something.
pub(crate) fn fetch_row(mut term: Term, row: c_int, end_col: c_int) {
    // Worst case is one maximum-length grapheme cluster per column, plus
    // the terminator. C sized this buffer once, at 8191 bytes, which a wide
    // enough terminal full of clusters overruns.
    let needed = as_index(end_col.max(0)) * MAX_SCHAR_SIZE as usize + 1;
    if term.textbuf.len() < needed {
        term.textbuf.resize(needed, 0);
    }
    // Out of the terminal for the walk: reading a cell does not touch it.
    // The two arms are the same walk, specialised: which half of
    // `fetch_cell` runs is decided once per row rather than once per cell,
    // and a screen row is found once rather than for each of its cells.
    let mut text = core::mem::take(&mut term.textbuf);
    let line_len = if row >= 0 {
        let screen = term.vt.screen_row(row);
        render_row(&mut text, end_col, |col, cell| {
            if let Some(screen) = &screen {
                screen.export(col, cell);
            }
            true
        })
    } else {
        render_row(&mut text, end_col, |col, cell| {
            fetch_cell(term, row, col, cell)
        })
    };
    text[line_len] = b'\0';
    term.textbuf = text;
}

/// [`fetch_row`]'s walk over the cells `cell_at` reads, into `text`:
/// answers the length of the line up to its last cell with content.
#[inline(always)]
fn render_row(
    text: &mut [u8],
    end_col: c_int,
    mut cell_at: impl FnMut(c_int, &mut VTermScreenCell) -> bool,
) -> usize {
    let mut at = 0usize;
    let mut line_len = 0usize;
    let mut col = 0;
    while col < end_col {
        let mut cell = blank_cell();
        cell_at(col, &mut cell);
        if cell.schar != 0 {
            // The bytes past the glyph are overwritten by the next cell or
            // lie past the terminator.
            at += schar_put_scratch(text, at, cell.schar);
            line_len = at;
        } else {
            // Written but not counted, so that trailing blanks are dropped
            // by terminating at the last cell with content.
            text[at] = b' ';
            at += 1;
        }
        col += c_int::from(cell.width);
    }
    line_len
}

/// One cell of the screen or of the scrollback.
///
/// Returns false for a scrollback cell past the end of a row that was
/// stored while the terminal was narrower; the cell is left blank.
pub(crate) fn fetch_cell(term: Term, row: c_int, col: c_int, cell: &mut VTermScreenCell) -> bool {
    if row >= 0 {
        term.vt.get_cell(VTermPos { row, col }, cell);
        return true;
    }
    let stored = term
        .sb
        .row(as_index(-row - 1))
        .and_then(|stored| stored.get(as_index(col)));
    match stored {
        Some(found) => {
            *cell = *found;
            true
        }
        None => {
            *cell = blank_cell();
            false
        }
    }
}

/// Bring the scrollback back within `'scrollback'` after the option changed
/// or rows were pushed.
///
/// Trimming deletes from the top of the buffer, so the marks that pointed
/// into those lines have to move with them.
pub(crate) fn adjust_scrollback(mut term: Term, buffer: Buf) {
    let limit = scrollback_limit(buffer);
    assert!(
        term.sb.pending() == 0,
        "scrollback trimmed while rows were still owed to the buffer"
    );
    if limit < term.sb.len() {
        let diff = term.sb.len() - limit;
        for _ in 0..diff {
            let _ = ml_delete_buf(buffer, LineNr::ONE, false);
            term.sb.drop_oldest();
        }
        let diff = len_as_int(diff);
        mark_adjust_term(buffer, LineNr::new(diff), -diff);
        // Report what the deletion took away.
        deleted_lines_buf(buffer, LineNr::ONE, diff);
    }
    term.sb.set_capacity(limit);
}

/// `mark_adjust_buf` as this module always calls it: a deletion at the top
/// of a terminal buffer, running to the end of it.
fn mark_adjust_term(buffer: Buf, line2: LineNr, amount: i32) {
    let (end, after) = (MAXLNUM.get(), true);
    let (mode, op) = (kMarkAdjustTerm, kExtmarkUndo);
    mark_adjust_buf(buffer, LineNr::ONE, line2, end, amount, after, mode, op);
}

/// Mirror everything the scrollback gained or lost into the buffer's lines.
///
/// Reading is paused for the duration: appending lines can run autocommands,
/// and more terminal output arriving in the middle would be appended at the
/// wrong place.
pub(crate) fn refresh_scrollback(mut term: Term, buffer: Buf) {
    let read_pause = term.opts.read_pause_cb.expect("non-null function pointer");
    read_pause(true, term.opts.owner);

    // Rows evicted since the last refresh are gone from the buffer's top;
    // move the marks that were pointing at them.
    let mut deleted =
        LineNr::new(len_as_int(term.sb.deleted() - term.old_sb_deleted)).min(buffer.line_count());
    mark_adjust_term(buffer, deleted, -deleted.get());
    term.old_sb_deleted = term.sb.deleted();

    let mut old_height = term.old_height;
    let (height, width) = term.size();

    while deleted > LineNr::ZERO && buffer.line_count() > LineNr::new(old_height) {
        let _ = ml_delete_buf(buffer, LineNr::ONE, false);
        deleted_lines_buf(buffer, LineNr::ONE, 1);
        deleted -= 1;
    }
    old_height = old_height.min(buffer.line_count().get());

    // Each owed row is appended just above the rows that make up the
    // screen, which sit at the end of the buffer.
    while term.sb.pending() > 0 {
        fetch_row(term, -term.sb.pending(), width);
        let at = LineNr::new(buffer.line_count().get() - old_height);
        let text = crate::cstr::in_bytes(&term.textbuf).to_bytes_with_nul();
        let _ = ml_append_buf_terminated(buffer, at, text, false);
        appended_lines_buf(buffer, at, 1);
        term.sb.mark_mirrored();
    }

    // Anything past the scrollback plus one screen is stale.
    let max_line_count = LineNr::new(count_as_int(term.sb.len()) + height);
    while buffer.line_count() > max_line_count {
        let last = buffer.line_count();
        let _ = ml_delete_buf(buffer, last, false);
        deleted_lines_buf(buffer, buffer.line_count(), 1);
    }

    adjust_scrollback(term, buffer);
    read_pause(false, term.opts.owner);
}

/// The scrollback's line accounting, on the store itself: the rows kept
/// (newest first), the rows still owed to the buffer (`pending`), the rows
/// evicted over the terminal's life (`deleted`, which buffer line numbers
/// are relative to), and what a pop does to each — plus the row renderer's
/// trailing-blank rule. The oracle for retyping these counts.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::schar_from_ascii;
    use crate::terminal::state::Scrollback;

    fn cell(c: u8) -> VTermScreenCell {
        VTermScreenCell {
            schar: schar_from_ascii(c),
            ..blank_cell()
        }
    }

    /// A row of `text`, one cell per byte; a space is an empty cell.
    fn row(text: &str) -> Vec<VTermScreenCell> {
        text.bytes()
            .map(|c| if c == b' ' { blank_cell() } else { cell(c) })
            .collect()
    }

    /// The first cell of each kept row, newest first.
    fn kept(sb: &Scrollback) -> String {
        (0..sb.len())
            .map(|i| char::from(sb.row(i).expect("a kept row")[0].schar.to_le_bytes()[0]))
            .collect()
    }

    fn sized(capacity: usize) -> Scrollback {
        let mut sb = Scrollback::default();
        assert!(!sb.is_sized(), "a new scrollback waits for 'scrollback'");
        sb.set_capacity(capacity);
        assert!(sb.is_sized());
        sb
    }

    #[test]
    fn pushed_rows_are_kept_newest_first_and_owed_to_the_buffer() {
        let mut sb = sized(3);
        sb.push(&row("a"));
        sb.push(&row("b"));
        assert_eq!(kept(&sb), "ba");
        assert_eq!((sb.len(), sb.pending(), sb.deleted()), (2, 2, 0));
    }

    #[test]
    fn a_full_scrollback_evicts_the_oldest_and_counts_it_deleted() {
        let mut sb = sized(2);
        for c in ["a", "b", "c", "d", "e"] {
            sb.push(&row(c));
        }
        assert_eq!(kept(&sb), "ed");
        // Owed rows are capped at what is kept.
        assert_eq!((sb.len(), sb.pending(), sb.deleted()), (2, 2, 3));
    }

    #[test]
    fn a_pop_cancels_an_owed_row_before_it_grows_the_screen() {
        let mut sb = sized(5);
        sb.push(&row("a"));
        sb.push(&row("b"));
        let mut screen_row = row("zz");
        let mut old_height = 10;
        assert!(sb.pop(&mut screen_row, &mut old_height));
        assert_eq!(screen_row[0].schar, schar_from_ascii(b'b'));
        assert_eq!((sb.pending(), old_height), (1, 10), "owed, so cancelled");
        sb.mark_mirrored();
        assert_eq!(sb.pending(), 0);
        assert!(sb.pop(&mut screen_row, &mut old_height));
        assert_eq!(screen_row[0].schar, schar_from_ascii(b'a'));
        assert_eq!(
            (sb.pending(), old_height),
            (0, 11),
            "mirrored, so the screen grew"
        );
        assert!(!sb.pop(&mut screen_row, &mut old_height), "nothing left");
        assert_eq!((sb.len(), old_height), (0, 11));
    }

    #[test]
    fn a_resize_after_a_refresh_gives_rows_back_one_screen_line_each() {
        // Ten rows scroll off and the refresh mirrors them all; then the
        // screen grows by three, which vterm satisfies with three pops.
        let mut sb = sized(100);
        for c in b'a'..b'k' {
            sb.push(&row(core::str::from_utf8(&[c]).expect("ascii")));
        }
        while sb.pending() > 0 {
            sb.mark_mirrored();
        }
        let (mut screen_row, mut old_height) = (row("    "), 24);
        for _ in 0..3 {
            assert!(sb.pop(&mut screen_row, &mut old_height));
        }
        assert_eq!(kept(&sb), "gfedcba");
        assert_eq!(
            (sb.len(), sb.pending(), sb.deleted(), old_height),
            (7, 0, 0, 27)
        );
    }

    #[test]
    fn a_narrower_row_pops_with_a_blank_tail_and_a_wider_one_is_cut() {
        let mut sb = sized(5);
        sb.push(&row("ab"));
        let mut screen_row = row("wxyz");
        screen_row[3].width = 2;
        let mut old_height = 0;
        assert!(sb.pop(&mut screen_row, &mut old_height));
        let cells: Vec<_> = screen_row.iter().map(|c| (c.schar, c.width)).collect();
        let (a, b) = (schar_from_ascii(b'a'), schar_from_ascii(b'b'));
        assert_eq!(cells, vec![(a, 1), (b, 1), (0, 1), (0, 1)]);

        sb.push(&row("abcd"));
        let mut narrow = row("zz");
        assert!(sb.pop(&mut narrow, &mut old_height));
        assert_eq!((narrow[0].schar, narrow[1].schar), (a, b));
    }

    #[test]
    fn an_eviction_reuses_a_row_only_of_the_same_width() {
        // Observable as the content: a reused row is overwritten whole.
        let mut sb = sized(1);
        sb.push(&row("abc"));
        sb.push(&row("xyz"));
        assert_eq!(sb.row(0).map(<[_]>::len), Some(3));
        assert_eq!(sb.row(0).expect("kept")[2].schar, schar_from_ascii(b'z'));
        sb.push(&row("pq"));
        assert_eq!(sb.row(0).map(<[_]>::len), Some(2));
        assert_eq!(sb.deleted(), 2);
    }

    #[test]
    fn clearing_counts_every_kept_row_deleted_and_forgives_the_debt() {
        let mut sb = sized(5);
        for c in ["a", "b", "c"] {
            sb.push(&row(c));
        }
        sb.clear();
        assert!(sb.is_empty() && sb.is_sized());
        assert_eq!((sb.pending(), sb.deleted()), (0, 3));
    }

    #[test]
    fn trimming_the_oldest_row_does_not_shift_the_numbering() {
        let mut sb = sized(5);
        for c in ["a", "b", "c"] {
            sb.push(&row(c));
        }
        sb.drop_oldest();
        assert_eq!(kept(&sb), "cb");
        assert_eq!((sb.pending(), sb.deleted()), (3, 0));
    }

    /// [`render_row`] over `cells`, answering the text up to the length it
    /// reports and the columns it asked for.
    fn render(cells: &[VTermScreenCell], end_col: c_int) -> (String, Vec<c_int>) {
        let mut text = vec![0u8; 64];
        let mut asked = Vec::new();
        let len = render_row(&mut text, end_col, |col, out| {
            asked.push(col);
            if let Some(found) = cells.get(as_index(col)) {
                *out = *found;
            }
            true
        });
        (
            String::from_utf8(text[..len].to_vec()).expect("ascii"),
            asked,
        )
    }

    #[test]
    fn a_rendered_row_drops_trailing_blanks_but_keeps_inner_ones() {
        assert_eq!(
            render(&row("a b  "), 5),
            ("a b".into(), vec![0, 1, 2, 3, 4])
        );
        assert_eq!(render(&row("   "), 3).0, "");
        // Past the stored row the cells are blank.
        assert_eq!(render(&row("ab"), 6).0, "ab");
    }

    #[test]
    fn a_wide_cell_advances_the_walk_by_its_width() {
        let mut cells = row("ab  c");
        cells[0].width = 2;
        let (text, asked) = render(&cells, 5);
        assert_eq!(asked, vec![0, 2, 3, 4]);
        assert_eq!(text, "a  c", "column 1 is never asked for");
    }
}
