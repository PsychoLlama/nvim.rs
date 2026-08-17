//! The viewport arithmetic (`src/move/arith.rs`).
//!
//! `update_topline()` and `curs_columns()` are the two ends of "which part of
//! the buffer does this window show, and where in it is the cursor?". Between
//! their window reads sit a handful of decisions that are pure integer work:
//! how far `'scrolljump'` moves, when the cursor is far enough out to be
//! re-centred rather than scrolled to, where a virtual column lands once
//! `'wrap'` and `w_skipcol` have had their say, and which way `'sidescroll'`
//! pushes `w_leftcol`. Those are what this file drives.
//!
//! Each expectation is derived from `v0.12.4`'s `src/nvim/move.c` rather than
//! from the port: the C branch each case is aiming at is named in its comment.
//! Nothing here needs an editor — no window, no buffer, no screen — which is
//! also what lets Miri run the lot.

use std::ffi::c_int;

use c2rust_neovim::r#move::arith::{
    cursor_screen_col, fit_scrolloff_cols, fit_skipcol_to_window, marker_overlap,
    recentre_threshold, scrolljump_lines, scrolloff_cols, sidescroll_leftcol,
    skipcol_for_tall_line, skipcol_from_plines, skipcol_line_back, skipcol_showing_last,
    skipped_plines, sms_cursor_row, sms_fixup_count_back, sms_fixup_count_forw, top_skipped_plines,
    visible_sms_col, wrap_cursor_cell, wrap_rowoff,
};
use c2rust_neovim::types::colnr_T;

// ------------------------------------------------------------ skipped_plines
//
// `adjust_plines_for_skipcol()`:
//
//   if (wp->w_skipcol == 0) return 0;
//   int width = wp->w_view_width - win_col_off(wp);
//   int w2 = width + win_col_off2(wp);
//   if (wp->w_skipcol >= width && w2 > 0) return (wp->w_skipcol - width) / w2 + 1;
//   return 0;
//
// A window whose first screen line holds 10 cells of text and whose later
// ones hold 8 (a 'number' column that 'cpoptions' does not indent past).

const W1: c_int = 10;
const W2: c_int = 8;

#[test]
fn nothing_is_skipped_without_skipcol() {
    assert_eq!(skipped_plines(0, W1, W2), 0);
}

#[test]
fn a_skipcol_inside_the_first_screen_line_hides_no_whole_line() {
    // `w_skipcol < width`: the first arm's guard fails.
    assert_eq!(skipped_plines(1, W1, W2), 0);
    assert_eq!(skipped_plines(W1 - 1, W1, W2), 0);
}

#[test]
fn a_skipcol_at_the_wrap_hides_the_first_screen_line() {
    assert_eq!(skipped_plines(W1, W1, W2), 1);
    assert_eq!(skipped_plines(W1 + W2 - 1, W1, W2), 1);
    assert_eq!(skipped_plines(W1 + W2, W1, W2), 2);
    assert_eq!(skipped_plines(W1 + 3 * W2, W1, W2), 4);
}

#[test]
fn a_window_with_no_room_on_its_later_lines_skips_nothing() {
    // `w2 > 0` guards the division.
    assert_eq!(skipped_plines(100, W1, 0), 0);
    assert_eq!(skipped_plines(100, W1, -4), 0);
}

// -------------------------------------------------------- skipcol_from_plines
//
// `skipcol_from_plines()`, the inverse of the above. Round-tripping the two
// is the property that matters: a skipcol built to hide N lines hides N.

#[test]
fn hiding_no_screen_line_needs_no_skipcol() {
    assert_eq!(skipcol_from_plines(0, W1, W2), 0);
    // Negative offsets never reach either arm.
    assert_eq!(skipcol_from_plines(-3, W1, W2), 0);
}

#[test]
fn hiding_screen_lines_costs_the_first_width_then_the_rest() {
    assert_eq!(skipcol_from_plines(1, W1, W2), W1);
    assert_eq!(skipcol_from_plines(2, W1, W2), W1 + W2);
    assert_eq!(skipcol_from_plines(5, W1, W2), W1 + 4 * W2);
}

#[test]
fn the_two_skipcol_helpers_round_trip() {
    for lines in 0..8 {
        let skipcol = skipcol_from_plines(lines, W1, W2);
        assert_eq!(skipped_plines(skipcol, W1, W2), lines, "lines={lines}");
    }
}

// ------------------------------------------------------------ marker_overlap
//
// `sms_marker_overlap()`:
//
//   if (*get_showbreak_value(wp) != NUL) return 0;
//   if (wp->w_p_list && wp->w_p_lcs_chars.prec) return 1;
//   return extra2 > 3 ? 0 : 3 - extra2;

#[test]
fn showbreak_draws_no_marker_over_text() {
    assert_eq!(marker_overlap(0, true, false), 0);
    // 'showbreak' wins over 'listchars' "precedes".
    assert_eq!(marker_overlap(0, true, true), 0);
}

#[test]
fn the_precedes_character_overlaps_one_column() {
    assert_eq!(marker_overlap(0, false, true), 1);
    assert_eq!(marker_overlap(9, false, true), 1);
}

#[test]
fn the_smoothscroll_marker_is_three_columns_minus_the_padding() {
    assert_eq!(marker_overlap(0, false, false), 3);
    assert_eq!(marker_overlap(1, false, false), 2);
    assert_eq!(marker_overlap(3, false, false), 0);
    // Padding wider than the marker leaves no overlap at all — and does not
    // go negative.
    assert_eq!(marker_overlap(4, false, false), 0);
    assert_eq!(marker_overlap(80, false, false), 0);
}

// ---------------------------------------------------------- scrolljump_lines
//
// `scrolljump_value()`:
//
//   int result = p_sj >= 0 ? (int)p_sj : (wp->w_view_height * (int)(-p_sj)) / 100;

#[test]
fn a_positive_scrolljump_is_a_line_count() {
    assert_eq!(scrolljump_lines(0, 24), 0);
    assert_eq!(scrolljump_lines(1, 24), 1);
    assert_eq!(scrolljump_lines(5, 24), 5);
    // The window height does not cap it here; `scroll_cursor_*` does.
    assert_eq!(scrolljump_lines(100, 24), 100);
}

#[test]
fn a_negative_scrolljump_is_a_percentage_of_the_window() {
    assert_eq!(scrolljump_lines(-50, 24), 12);
    assert_eq!(scrolljump_lines(-100, 24), 24);
    // Integer division truncates, as the C's does.
    assert_eq!(scrolljump_lines(-50, 25), 12);
    assert_eq!(scrolljump_lines(-1, 24), 0);
}

// ------------------------------------------------------- recentre_threshold
//
//   int halfheight = wp->w_view_height / 2 - 1;
//   if (halfheight < 2) halfheight = 2;

#[test]
fn the_recentre_threshold_is_half_the_window() {
    assert_eq!(recentre_threshold(24), 11);
    assert_eq!(recentre_threshold(25), 11);
    assert_eq!(recentre_threshold(10), 4);
}

#[test]
fn a_short_window_still_scrolls_two_lines_before_recentring() {
    // 6/2-1 == 2 is the last height the floor does not bite at.
    assert_eq!(recentre_threshold(6), 2);
    assert_eq!(recentre_threshold(5), 2);
    assert_eq!(recentre_threshold(1), 2);
    assert_eq!(recentre_threshold(0), 2);
}

// -------------------------------------------------- wrap_rowoff / screen col
//
// `validate_cursor_col()`:
//
//   col = wp->w_virtcol + off;
//   if (wp->w_p_wrap && col >= wp->w_view_width && width > 0)
//     col -= ((col - wp->w_view_width) / width + 1) * width;
//   if (col > wp->w_leftcol) col -= wp->w_leftcol; else col = 0;
//
// A window 12 columns wide with a 2-column 'number' gutter, so `width` (the
// text width of a *later* screen line, `view_width - off + col_off2`) is 10.

const VIEW: c_int = 12;
const OFF: colnr_T = 2;
const WIDTH: c_int = 10;

#[test]
fn a_column_on_the_first_screen_line_only_gains_the_gutter() {
    assert_eq!(cursor_screen_col(0, OFF, VIEW, WIDTH, true, 0), 2);
    assert_eq!(cursor_screen_col(9, OFF, VIEW, WIDTH, true, 0), 11);
}

#[test]
fn a_wrapped_column_comes_back_onto_its_own_screen_line() {
    // virtcol 10 -> col 12, one whole screen line past the edge.
    assert_eq!(wrap_rowoff(12, VIEW, WIDTH), 1);
    assert_eq!(cursor_screen_col(10, OFF, VIEW, WIDTH, true, 0), 2);
    assert_eq!(cursor_screen_col(19, OFF, VIEW, WIDTH, true, 0), 11);
    assert_eq!(wrap_rowoff(22, VIEW, WIDTH), 2);
    assert_eq!(cursor_screen_col(20, OFF, VIEW, WIDTH, true, 0), 2);
}

#[test]
fn without_wrap_the_column_keeps_running_off_the_right() {
    assert_eq!(cursor_screen_col(30, OFF, VIEW, WIDTH, false, 0), 32);
    // A window with no room for a second screen line cannot wrap either.
    assert_eq!(cursor_screen_col(30, OFF, VIEW, 0, true, 0), 32);
}

#[test]
fn horizontal_scrolling_shifts_the_column_and_clamps_at_zero() {
    assert_eq!(cursor_screen_col(30, OFF, VIEW, WIDTH, false, 10), 22);
    // `col > leftcol` is strict: a column at the left edge answers 0.
    assert_eq!(cursor_screen_col(30, OFF, VIEW, WIDTH, false, 32), 0);
    assert_eq!(cursor_screen_col(30, OFF, VIEW, WIDTH, false, 40), 0);
}

// -------------------------------------------------------- wrap_cursor_cell
//
// `curs_columns()`'s 'wrap' arm. `w_skipcol` is deducted in whole multiples of
// width2 so that the wrapping formula below it still works.

#[test]
fn a_cursor_on_a_line_that_is_not_scrolled_only_wraps() {
    // wcol 25 in a 12-wide window with width2 10: two screen lines down.
    let (wcol, wrow, subbed) = wrap_cursor_cell(25, 4, 0, true, W1, WIDTH, VIEW);
    assert_eq!((wcol, wrow), (5, 6));
    assert!(!subbed);
}

#[test]
fn skipcol_is_only_deducted_on_the_top_line() {
    // The same column on a line that is not `w_topline`.
    let (wcol, wrow, subbed) = wrap_cursor_cell(25, 4, 10, false, W1, WIDTH, VIEW);
    assert_eq!((wcol, wrow), (5, 6));
    assert!(!subbed);
}

#[test]
fn a_skipcol_inside_the_first_screen_line_costs_one_width2() {
    // `w_skipcol <= width1`, so exactly one multiple comes off.
    let (wcol, wrow, subbed) = wrap_cursor_cell(25, 0, 10, true, W1, WIDTH, VIEW);
    assert_eq!((wcol, wrow), (5, 1));
    assert!(subbed);
}

#[test]
fn a_larger_skipcol_costs_as_many_multiples_as_it_hides() {
    // `w_skipcol > width1`: ((25 - 10) / 10 + 1) == 2 multiples.
    let (wcol, wrow, subbed) = wrap_cursor_cell(60, 0, 25, true, W1, WIDTH, VIEW);
    assert_eq!((wcol, wrow), (10, 3));
    assert!(subbed);
}

#[test]
fn a_cursor_before_the_skipped_columns_is_left_alone() {
    // `w_wcol >= w_skipcol` guards the deduction.
    let (wcol, wrow, subbed) = wrap_cursor_cell(5, 2, 10, true, W1, WIDTH, VIEW);
    assert_eq!((wcol, wrow), (5, 2));
    assert!(!subbed);
}

// ------------------------------------------------------- sidescroll_leftcol
//
// `curs_columns()`'s no-wrap arm. A 40-column window, cursor character at
// virtual columns 50..50, currently scrolled to column 0.

const SIDE_VIEW: c_int = 40;

fn scroll(start: colnr_T, end: colnr_T, leftcol: colnr_T, siso: i64, ss: i64) -> Option<c_int> {
    // `wcol` is the cursor's screen column with `extra` still in it, which is
    // how `curs_columns()` has it at this point; `extra` is the gutter.
    let extra = 0;
    sidescroll_leftcol(
        start, end, leftcol, start, extra, SIDE_VIEW, SIDE_VIEW, siso, ss,
    )
}

#[test]
fn a_cursor_comfortably_on_screen_does_not_scroll() {
    assert_eq!(scroll(10, 10, 0, 0, 1), None);
    // Right at the last column: `off_right` is 0, which is not > 0.
    assert_eq!(scroll(39, 39, 0, 0, 1), None);
}

#[test]
fn a_cursor_past_the_right_edge_scrolls_to_the_middle_when_sidescroll_is_zero() {
    // p_ss == 0: `new_leftcol = w_wcol - extra - width1 / 2`.
    assert_eq!(scroll(50, 50, 0, 0, 0), Some(30));
}

#[test]
fn a_small_sidescroll_step_moves_by_the_step() {
    // diff == off_right == 11, under width1/2 (20), and `off_right >=
    // off_left` fails because the cursor is 50 columns past `w_leftcol`
    // while only 11 past the right edge. So the step arm runs and the
    // window scrolls by exactly the gap.
    assert_eq!(scroll(50, 50, 0, 0, 1), Some(11));
}

#[test]
fn a_cursor_off_the_left_scrolls_by_the_sidescroll_step() {
    // off_left is negative and off_right is not positive, so `off_right >=
    // off_left` fails and the step arm runs: leftcol - max(diff, p_ss).
    assert_eq!(scroll(18, 18, 20, 0, 1), Some(18));
    // A step wider than the gap is used as-is.
    assert_eq!(scroll(19, 19, 20, 0, 5), Some(15));
}

#[test]
fn a_gap_of_half_a_window_centres_instead_of_stepping() {
    // diff >= width1 / 2 takes the centring arm even off the left.
    assert_eq!(scroll(0, 0, 30, 0, 1), Some(0));
}

#[test]
fn sidescrolloff_widens_the_margin_the_cursor_must_keep() {
    // Without 'sidescrolloff' column 39 is fine; with 5 it is not.
    assert_eq!(scroll(39, 39, 0, 0, 1), None);
    assert!(scroll(39, 39, 0, 5, 1).is_some());
}

#[test]
fn the_answer_never_scrolls_past_the_first_column() {
    assert_eq!(scroll(0, 0, 5, 0, 1), Some(0));
    assert_eq!(scroll(0, 0, 1, 3, 1), Some(0));
}

// ---------------------------------------------- skipcol_for_tall_line & fit
//
// `curs_columns()`'s tall-line arm: a single buffer line taller than the
// window, where `w_skipcol` decides which part of it is shown. The C's
// "extra" is a two-bit answer — 1: fewer than 'scrolloff' lines above,
// 2: fewer below, 3: both.
//
// A 5-line window whose text is 10 cells wide.

const TALL_H: c_int = 5;

#[test]
fn without_scrolloff_a_cursor_below_the_window_lowers_the_skipcol() {
    // so == 0, cursor on display line 8 of a 12-line-tall buffer line:
    // extra == 2, so the skipcol rises until the cursor's line is last.
    let skipcol = skipcol_for_tall_line(0, 85, 0, W1, WIDTH, TALL_H, 8, 12);
    assert_eq!(skipcol, 40);
}

#[test]
fn a_cursor_above_the_skipped_part_raises_the_skipcol_back() {
    // extra == 1: `w_skipcol + so*width2 > w_virtcol` with so == 0 needs
    // skipcol > virtcol, i.e. the cursor is above what is shown.
    let skipcol = skipcol_for_tall_line(40, 5, 0, W1, WIDTH, TALL_H, 0, 12);
    assert_eq!(skipcol, 0);
}

#[test]
fn a_window_too_short_for_scrolloff_centres_the_cursor() {
    // `w_view_height <= so * 2` takes the centring arm whatever "extra" says.
    let skipcol = skipcol_for_tall_line(0, 85, 3, W1, WIDTH, TALL_H, 2, 12);
    // n = 85/10 = 8, minus half the window (2) = 6, capped at
    // plines - height + 1 = 11 - 5 + 1 = 7 -> 6; width1 + 5*width2.
    assert_eq!(skipcol, W1 + 5 * WIDTH);
}

#[test]
fn a_cursor_already_framed_leaves_the_skipcol_alone() {
    // Neither "extra" bit set: skipcol <= virtcol and the cursor is far
    // enough from the bottom.
    let skipcol = skipcol_for_tall_line(20, 25, 0, W1, WIDTH, TALL_H, 1, 12);
    assert_eq!(skipcol, 20);
}

#[test]
fn the_row_moves_with_the_skipcol_that_was_chosen() {
    // No skipcol was deducted from `w_wcol`, so the whole new skipcol comes
    // off the row: 40/10 == 4 screen lines.
    let (skipcol, wrow, scrolled) = fit_skipcol_to_window(40, 0, 6, false, WIDTH, TALL_H);
    assert_eq!((skipcol, wrow), (40, 2));
    // The window's contents moved four lines up.
    assert_eq!(scrolled, -4);
}

#[test]
fn only_the_change_in_skipcol_comes_off_a_row_that_already_had_it_deducted() {
    let (skipcol, wrow, scrolled) = fit_skipcol_to_window(40, 20, 6, true, WIDTH, TALL_H);
    assert_eq!((skipcol, wrow), (40, 4));
    assert_eq!(scrolled, -2);
}

#[test]
fn a_row_still_below_the_window_pushes_the_skipcol_further() {
    // wrow 9 with no skipcol deducted: 9 - 5 + 1 == 5 lines too far.
    let (skipcol, wrow, scrolled) = fit_skipcol_to_window(0, 0, 9, false, WIDTH, TALL_H);
    assert_eq!((skipcol, wrow), (50, 4));
    assert_eq!(scrolled, -5);
}

#[test]
fn a_skipcol_that_shrank_scrolls_the_window_the_other_way() {
    let (skipcol, wrow, scrolled) = fit_skipcol_to_window(10, 40, 4, false, WIDTH, TALL_H);
    assert_eq!((skipcol, wrow), (10, 3));
    assert_eq!(scrolled, 3);
}

// ---------------------------------------------------------------------------
// The 'smoothscroll' arithmetic, which works in the two *text* widths rather
// than in screen lines. `win_col_off2()` is never negative, so a later screen
// line is never narrower than the first: these use a window whose first line
// holds 10 cells and whose later ones hold 14, the shape a reused 'number'
// column gives ('cpoptions' containing "n").

const S1: c_int = 10;
const S2: c_int = 14;

// ---------------------------------------------------- top_skipped_plines
//
// `scroll_cursor_bot()`'s "a similar formula is used in curs_columns()":
//
//   if (wp->w_skipcol > width1) skip_lines += (wp->w_skipcol - width1) / width2 + 1;
//   else if (wp->w_skipcol > 0) skip_lines = 1;
//
// The similarity is not equality, which is why this is tested next to
// `skipped_plines`.

#[test]
fn any_skipcol_at_all_hides_the_first_screen_line_here() {
    assert_eq!(top_skipped_plines(0, S1, S2), 0);
    assert_eq!(top_skipped_plines(1, S1, S2), 1);
    assert_eq!(top_skipped_plines(S1, S1, S2), 1);
}

#[test]
fn the_two_skip_counts_disagree_below_the_first_wrap() {
    // `curs_columns()` still shows the first screen line, so it counts none;
    // `scroll_cursor_bot()` has to scroll past the clipped part of it.
    for skipcol in 1..S1 {
        assert_eq!(skipped_plines(skipcol, S1, S2), 0, "skipcol={skipcol}");
        assert_eq!(top_skipped_plines(skipcol, S1, S2), 1, "skipcol={skipcol}");
    }
}

#[test]
fn past_the_first_wrap_the_two_skip_counts_agree() {
    for skipcol in [S1, S1 + 1, S1 + S2, S1 + 3 * S2, S1 + 3 * S2 + 1] {
        assert_eq!(
            top_skipped_plines(skipcol, S1, S2),
            skipped_plines(skipcol, S1, S2),
            "skipcol={skipcol}"
        );
    }
}

// -------------------------------------------------------- scrolloff_cols
//
// `cursor_correct_sms()` and `adjust_skipcol()`:
//
//   int64_t so_cols = so == 0 ? 0 : width1 + (so - 1) * width2;

#[test]
fn no_scrolloff_asks_for_no_columns() {
    assert_eq!(scrolloff_cols(0, S1, S2), 0);
}

#[test]
fn scrolloff_costs_the_first_screen_line_then_the_rest() {
    assert_eq!(scrolloff_cols(1, S1, S2), S1 as i64);
    assert_eq!(scrolloff_cols(2, S1, S2), (S1 + S2) as i64);
    assert_eq!(scrolloff_cols(4, S1, S2), (S1 + 3 * S2) as i64);
}

#[test]
fn a_huge_scrolloff_stays_in_range_of_an_int64() {
    // 'scrolloff' is an OptInt; the C widens before multiplying and so does
    // this, which is what keeps `set so=100000000` from overflowing an int.
    assert_eq!(
        scrolloff_cols(100_000_000, S1, S2),
        S1 as i64 + 99_999_999 * S2 as i64
    );
}

// ----------------------------------------------------- fit_scrolloff_cols
//
//   while (so_cols > size && so_cols - width2 >= width1 && width1 > 0) so_cols -= width2;
//   if (so_cols >= width1 && so_cols > size) so_cols -= width1;

#[test]
fn a_line_wide_enough_keeps_all_its_scrolloff_columns() {
    assert_eq!(fit_scrolloff_cols(26, 100, S1, S2), 26);
}

#[test]
fn a_short_line_gives_up_scrolloff_a_screen_line_at_a_time() {
    // Four screen lines of context against a line only 5 cells wide: the loop
    // takes width2 off while a whole first line still fits, then the trailing
    // step takes the first line itself.
    assert_eq!(fit_scrolloff_cols(52, 5, S1, S2), 0);
    // A wider line keeps the context that fits in it.
    assert_eq!(fit_scrolloff_cols(52, 30, S1, S2), 24);
}

#[test]
fn a_window_with_no_text_width_cannot_lose_whole_screen_lines() {
    // `width1 > 0` guards the loop; the trailing `if` still applies, and with
    // a zero width1 it takes nothing off.
    assert_eq!(fit_scrolloff_cols(30, 5, 0, S2), 30);
}

// -------------------------------------------------------- visible_sms_col
//
// `cursor_correct_sms()`'s band walk: step the cursor's column by whole
// screen lines until it lies inside `top..bot`.

#[test]
fn a_column_inside_the_band_does_not_move() {
    assert_eq!(visible_sms_col(30, 20, 60, S1, S2), 30);
}

#[test]
fn a_column_above_the_band_climbs_by_whole_screen_lines() {
    // Below width1 it first gains the whole first screen line, then later
    // ones until it is inside.
    assert_eq!(visible_sms_col(2, 20, 60, S1, S2), 26);
    assert_eq!(visible_sms_col(18, 20, 60, S1, S2), 32);
}

#[test]
fn a_column_below_the_band_drops_by_whole_screen_lines() {
    assert_eq!(visible_sms_col(60, 20, 60, S1, S2), 46);
    assert_eq!(visible_sms_col(75, 20, 60, S1, S2), 47);
}

#[test]
fn a_window_with_no_later_width_takes_at_most_the_first_step() {
    assert_eq!(visible_sms_col(2, 20, 60, S1, 0), 2 + S1);
    assert_eq!(visible_sms_col(75, 20, 60, S1, 0), 75);
}

// ------------------------------------------------------ skipcol_line_back
//
//   if (skipcol >= width1 + width2) skipcol -= width2; else skipcol -= width1;

#[test]
fn scrolling_back_past_the_first_wrap_costs_a_later_width() {
    assert_eq!(skipcol_line_back(S1 + S2, S1, S2), S1);
    assert_eq!(skipcol_line_back(S1 + 3 * S2, S1, S2), S1 + 2 * S2);
}

#[test]
fn scrolling_back_onto_the_first_screen_line_costs_the_first_width() {
    assert_eq!(skipcol_line_back(S1, S1, S2), 0);
    assert_eq!(skipcol_line_back(S1 + S2 - 1, S1, S2), S2 - 1);
}

// ---------------------------------------------------- skipcol_showing_last
//
// `scrolldown()`'s 'smoothscroll' arm, which puts the *last* screen line of
// the new top line at the top of the window.

#[test]
fn a_line_that_fits_needs_no_skipcol() {
    assert_eq!(skipcol_showing_last(0, S1, S2), 0);
    assert_eq!(skipcol_showing_last(S1, S1, S2), 0);
}

#[test]
fn a_wrapped_line_skips_all_but_its_last_screen_line() {
    assert_eq!(skipcol_showing_last(S1 + 1, S1, S2), S1);
    assert_eq!(skipcol_showing_last(S1 + S2, S1, S2), S1);
    assert_eq!(skipcol_showing_last(S1 + S2 + 1, S1, S2), S1 + S2);
    assert_eq!(skipcol_showing_last(S1 + 3 * S2, S1, S2), S1 + 2 * S2);
}

#[test]
fn the_skipcol_it_picks_hides_every_screen_line_but_one() {
    for size in S1 + 1..S1 + 4 * S2 {
        let skipcol = skipcol_showing_last(size, S1, S2);
        // `size` cells take this many screen lines; all but the last are hidden.
        let lines = 1 + (size - S1 + S2 - 1) / S2;
        assert_eq!(skipped_plines(skipcol, S1, S2), lines - 1, "size={size}");
    }
}

// -------------------------------------------------------- sms_cursor_row
//
// `adjust_skipcol()`'s row computation.

#[test]
fn a_cursor_on_the_first_screen_line_is_on_row_zero() {
    assert_eq!(sms_cursor_row(0, 0, 0, 100, S1, S2), 0);
    assert_eq!(sms_cursor_row(S1 - 1, 0, 0, 100, S1, S2), 0);
}

#[test]
fn each_later_screen_line_is_one_row_further_down() {
    assert_eq!(sms_cursor_row(S1, 0, 0, 100, S1, S2), 1);
    assert_eq!(sms_cursor_row(S1 + S2 + 1, 0, 0, 100, S1, S2), 2);
    assert_eq!(sms_cursor_row(S1 + 3 * S2 + 1, 0, 0, 100, S1, S2), 4);
}

#[test]
fn a_column_exactly_on_a_wrap_stays_on_the_row_above_it() {
    // Upstream's second test is `col > width2`, not `>=`, so a column landing
    // exactly at a wrap is still booked to the screen line before it. Kept.
    assert_eq!(sms_cursor_row(S1 + S2, 0, 0, 100, S1, S2), 1);
    // Only that one wrap: past it the division answers for itself again.
    assert_eq!(sms_cursor_row(S1 + 2 * S2, 0, 0, 100, S1, S2), 3);
}

#[test]
fn the_skipcol_comes_off_the_row() {
    assert_eq!(sms_cursor_row(S1 + 3 * S2, 0, S1 + S2, 100, S1, S2), 2);
}

#[test]
fn scrolloff_columns_are_wound_back_to_the_lines_own_width() {
    // A 12-cell line rounds up to S1 + S2 cells of screen, so asking for four
    // screen lines of context cannot push the row past it.
    assert_eq!(sms_cursor_row(2, (S1 + 3 * S2) as i64, 0, 12, S1, S2), 1);
}

// ------------------------------------------------------ sms_fixup_count_*
//
// `scroll_with_sms()`: how many more screen lines it takes to bring
// `w_skipcol` back to zero, in each direction.

#[test]
fn scrolling_back_counts_the_screen_lines_already_skipped() {
    assert_eq!(sms_fixup_count_back(S1 + 1, S1, S2), 1);
    assert_eq!(sms_fixup_count_back(S1 + S2, S1, S2), 1);
    assert_eq!(sms_fixup_count_back(S1 + S2 + 1, S1, S2), 2);
    assert_eq!(sms_fixup_count_back(S1 + 3 * S2, S1, S2), 3);
}

#[test]
fn scrolling_on_counts_the_screen_lines_still_to_come() {
    // A 40-cell line with the first screen line skipped has two more wraps.
    assert_eq!(sms_fixup_count_forw(S1, 40, S1, S2), 3);
    assert_eq!(sms_fixup_count_forw(S1 + S2, 40, S1, S2), 2);
    // At the very end it still asks for one, which is what makes the caller's
    // second `scroll_redraw()` clear the last partly visible line.
    assert_eq!(sms_fixup_count_forw(40, 40, S1, S2), 1);
}
