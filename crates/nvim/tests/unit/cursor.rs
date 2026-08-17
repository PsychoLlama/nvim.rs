//! The cursor's viewport arithmetic (`src/cursor/arith.rs`).
//!
//! `coladvance2` is the editor's answer to "put the cursor at screen column
//! N": it walks the line measuring characters, overshoots by one, steps back,
//! and then — under 'virtualedit' or 'wrap' — negotiates a position the line
//! does not actually have. Those decisions are what this file drives, along
//! with `check_cursor_col`'s clamp and `get_cursor_rel_lnum`'s fold-skipping
//! count.
//!
//! Each expectation is derived from `v0.12.4`'s `src/nvim/cursor.c` rather
//! than from the port: the C branch each case is aiming at is named in its
//! comment. Nothing here needs an editor — no window, no buffer, no screen —
//! which is also what lets Miri run the lot.

use std::cell::Cell;
use std::ffi::c_int;

use c2rust_neovim::cursor::arith::{
    ColAdd, carried_coladd, checked_col, folded_line_span, gap_coladd, step_back, wrap_target_col,
};
use c2rust_neovim::pos::MAXCOL;
use c2rust_neovim::types::{colnr_T, linenr_T};

// --------------------------------------------------------------- wrap_target_col
//
// `if (wcol / width > csize / width && ((State & MODE_INSERT) == 0 || wcol >
// csize + 1)) wcol = (csize / width + 1) * width - 1;`
//
// A window 10 columns wide showing a line of 25 cells: screen rows 0-9,
// 10-19 and 20-24, so `last_vcol` is 24.

const WIDTH: c_int = 10;
const LAST: c_int = 24;

#[test]
fn a_column_on_the_line_is_left_alone() {
    // Same screen row as the line's end (row 1 vs row 2 is not "past").
    assert_eq!(wrap_target_col(15, WIDTH, LAST, false), 15);
    // The line's own last column.
    assert_eq!(wrap_target_col(24, WIDTH, LAST, false), 24);
}

#[test]
fn a_column_past_the_last_screen_row_lands_on_its_end() {
    // Row 3 does not exist, so the cursor stops at the end of row 2.
    assert_eq!(wrap_target_col(35, WIDTH, LAST, false), 29);
    // Far past: still the end of row 2, not some multiple of it.
    assert_eq!(wrap_target_col(999, WIDTH, LAST, false), 29);
}

#[test]
fn insert_mode_is_allowed_one_column_past_the_line() {
    // `wcol > csize + 1` fails at exactly one past, which is where typing at
    // the right window edge leaves the cursor.
    assert_eq!(wrap_target_col(30, WIDTH, 29, true), 30);
    assert_eq!(wrap_target_col(30, WIDTH, 29, false), 29);
    // Two past is beyond the concession.
    assert_eq!(wrap_target_col(31, WIDTH, 29, true), 29);
}

#[test]
fn an_empty_line_pulls_back_to_its_first_row() {
    // `linetabsize_eol` of an empty line is 0, so every row but the first is
    // out of reach.
    assert_eq!(wrap_target_col(15, WIDTH, 0, false), 9);
    assert_eq!(wrap_target_col(5, WIDTH, 0, false), 5);
}

// -------------------------------------------------------------------- step_back
//
// `idx -= 1; csize -= head; col -= csize;`
//
// The measuring walk always ends one character past the one it wanted, so
// every path through it undoes exactly one step. `head` is the 'showbreak' /
// 'breakindent' padding charged to the character but not owned by it.

#[test]
fn stepping_back_undoes_one_character() {
    // A 2-cell character measured at column 10 left `col` at 12.
    assert_eq!(step_back(5, 12, 2, 0), (4, 10, 2));
}

#[test]
fn showbreak_padding_is_not_the_characters_to_give_back() {
    // Three cells charged, one of them 'showbreak': the character is 2 wide
    // and the cursor goes back to where its own cells began.
    assert_eq!(step_back(5, 12, 3, 1), (4, 10, 2));
    // All padding and no character: nothing to give back but the index.
    assert_eq!(step_back(5, 12, 1, 1), (4, 12, 0));
}

// ------------------------------------------------------------------- gap_coladd
//
// `int b = wcol - col; if (b > 0 && b < (MAXCOL - 2 * wp->w_view_width))
// pos->coladd = b;`

#[test]
fn a_gap_becomes_the_virtual_offset() {
    assert_eq!(gap_coladd(5, 80), 5);
}

#[test]
fn no_gap_means_no_virtual_offset() {
    assert_eq!(gap_coladd(0, 80), 0);
    // The walk can overshoot, which is not a position to hold onto.
    assert_eq!(gap_coladd(-3, 80), 0);
}

#[test]
fn an_implausible_gap_is_refused() {
    // 'virtualedit' admits columns near MAXCOL; two screens' worth short of
    // it is where they stop being believable.
    assert_eq!(gap_coladd(MAXCOL - 160, 80), 0);
    assert_eq!(gap_coladd(MAXCOL - 161, 80), MAXCOL - 161);
}

// ------------------------------------------------------------------ checked_col
//
// check_cursor_col's clamp: `if (len == 0) col = 0; else if (col >= len) { ...
// col = len : col = len - 1 + mark_mb_adjustpos } else if (col < 0) col = 0;`

#[test]
fn an_empty_line_puts_the_cursor_at_zero() {
    assert_eq!(checked_col(7, 0, || true), (0, false));
    assert_eq!(checked_col(7, 0, || false), (0, false));
}

#[test]
fn insert_mode_may_rest_on_the_nul() {
    assert_eq!(checked_col(7, 3, || true), (3, false));
    // Exactly at the NUL is already "past the last character".
    assert_eq!(checked_col(3, 3, || true), (3, false));
}

#[test]
fn normal_mode_steps_back_onto_the_last_character() {
    // The `true` asks for the head-byte adjustment the C does with
    // `mark_mb_adjustpos` — `len - 1` can land inside a multibyte character.
    assert_eq!(checked_col(7, 3, || false), (2, true));
    assert_eq!(checked_col(3, 3, || false), (2, true));
}

#[test]
fn a_column_inside_the_line_is_kept() {
    assert_eq!(checked_col(1, 3, || unreachable!()), (1, false));
    assert_eq!(checked_col(0, 3, || unreachable!()), (0, false));
}

#[test]
fn a_negative_column_is_pulled_to_zero() {
    assert_eq!(checked_col(-1, 3, || unreachable!()), (0, false));
}

#[test]
fn the_mode_question_is_only_asked_when_it_matters() {
    // Answering it costs several option lookups, and check_cursor_col runs
    // on every motion.
    let asked = Cell::new(0);
    let ask = || {
        asked.set(asked.get() + 1);
        true
    };
    checked_col(1, 3, ask);
    assert_eq!(asked.get(), 0);
    checked_col(9, 3, ask);
    assert_eq!(asked.get(), 1);
}

// --------------------------------------------------------------- carried_coladd
//
// `if (oldcol == MAXCOL) coladd = 0; else if (cur_ve_flags == kOptVeFlagAll) {
// if (oldcoladd > col) coladd = oldcoladd - col; else coladd = 0; }`
//
// `oldcoladd` is the C's `win->w_cursor.col + win->w_cursor.coladd` taken
// before the clamp: the cursor's virtual column, not its offset.

#[test]
fn the_end_of_the_line_drops_the_virtual_offset() {
    assert_eq!(carried_coladd(MAXCOL, 9, 5, true), ColAdd::Zero);
    // MAXCOL wins over the 'virtualedit' question entirely.
    assert_eq!(carried_coladd(MAXCOL, 9, 5, false), ColAdd::Zero);
}

#[test]
fn a_partial_virtualedit_leaves_the_offset_alone() {
    // Only ve=all lets the cursor keep a position the text does not have;
    // under the other spellings the C touches nothing.
    assert_eq!(carried_coladd(3, 9, 5, false), ColAdd::Keep);
}

#[test]
fn virtualedit_all_carries_the_distance_past_the_clamp() {
    assert_eq!(carried_coladd(3, 7, 5, true), ColAdd::Carry(2));
}

#[test]
fn a_virtual_column_the_clamp_caught_up_with_is_dropped() {
    // Not a miscalculation, just nothing left to be virtual about.
    assert_eq!(carried_coladd(3, 5, 5, true), ColAdd::Zero);
    // And the overflow guard the C's comment calls "a weird number".
    assert_eq!(carried_coladd(3, 4, 5, true), ColAdd::Zero);
}

// ------------------------------------------------------------ folded_line_span
//
// `for (; from_line < to_line; from_line++, retval++) { hasFolding(wp,
// from_line, NULL, &from_line); } if (from_line > to_line) retval--;`

/// The `fold_last` of a buffer with no folds at all.
fn unfolded(lnum: linenr_T) -> linenr_T {
    lnum
}

/// The `fold_last` of a buffer whose only closed fold covers `range`.
fn folded(range: std::ops::RangeInclusive<linenr_T>) -> impl Fn(linenr_T) -> linenr_T {
    move |lnum| {
        if range.contains(&lnum) {
            *range.end()
        } else {
            lnum
        }
    }
}

#[test]
fn without_folds_the_span_is_the_line_difference() {
    assert_eq!(folded_line_span(1, 5, unfolded), 4);
    assert_eq!(folded_line_span(1, 1, unfolded), 0);
}

#[test]
fn a_closed_fold_counts_as_one_line() {
    // Lines 1, [2-4], 5 between 1 and 6: three visible steps, not five.
    assert_eq!(folded_line_span(1, 6, folded(2..=4)), 3);
}

#[test]
fn a_fold_reaching_past_the_target_loses_its_last_step() {
    // `to` is inside the fold 3-7, so the step that swallowed it was not a
    // whole line: lines 1 and 2, then the fold the target sits in.
    assert_eq!(folded_line_span(1, 5, folded(3..=7)), 2);
}

#[test]
fn a_fold_the_span_starts_in_still_counts_once() {
    assert_eq!(folded_line_span(3, 9, folded(3..=7)), 2);
}

#[test]
fn the_span_is_a_column_of_visible_rows() {
    // Every from/to pair inside one closed fold has no visible lines
    // between them at all.
    let fold = folded(2..=9);
    for from in 2..=8 as colnr_T {
        assert_eq!(folded_line_span(from, from + 1, &fold), 0);
    }
}
