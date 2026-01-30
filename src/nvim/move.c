// move.c: Functions for moving the cursor and scrolling text.
//
// There are two ways to move the cursor:
// 1. Move the cursor directly, the text is scrolled to keep the cursor in the
//    window.
// 2. Scroll the text, the cursor is moved into the text visible in the
//    window.
// The 'scrolloff' option makes this a bit complicated.

#include <assert.h>
#include <limits.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#include "nvim/ascii_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/cursor.h"
#include "nvim/decoration.h"
#include "nvim/diff.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/errors.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/window.h"
#include "nvim/fold.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/grid_defs.h"
#include "nvim/macros_defs.h"
#include "nvim/mark_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/message.h"
#include "nvim/mouse.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/normal_defs.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/plines.h"
#include "nvim/popupmenu.h"
#include "nvim/pos_defs.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"
#include "nvim/winfloat.h"

typedef struct {
  linenr_T lnum;                // line number
  int fill;                     // filler lines
  int height;                   // height of added line
} lineoff_T;

#include "move.c.generated.h"

// Rust implementations
extern int rs_win_col_off(win_T *wp);
extern int rs_win_col_off2(win_T *wp);
extern int rs_sms_marker_overlap(win_T *wp, int extra2);
extern int rs_adjust_plines_for_skipcol(win_T *wp);
extern int rs_skipcol_from_plines(win_T *wp, int plines_off);
extern int rs_scrolljump_value(win_T *wp);
extern int rs_check_top_offset(win_T *wp);
extern void rs_reset_skipcol(win_T *wp);
extern void rs_compute_wcol(win_T *wp);
extern void rs_check_topfill(win_T *wp, int down);
extern int rs_scrolldown(win_T *wp, linenr_T line_count, int byfold);
extern int rs_scrollup(win_T *wp, linenr_T line_count, int byfold);
extern void rs_scroll_redraw(int up, linenr_T count);
extern void rs_scroll_cursor_halfway(win_T *wp, int atend, int prefer_above);
extern void rs_scroll_cursor_top(win_T *wp, int min_scroll, int always);
extern void rs_scroll_cursor_bot(win_T *wp, int min_scroll, int set_topbot);
extern void rs_set_empty_rows(win_T *wp, int used);
extern void rs_scrolldown_clamp(void);
extern void rs_scrollup_clamp(void);
extern void rs_cursor_correct_sms(win_T *wp);
extern void rs_adjust_skipcol(void);
extern void rs_set_topline(win_T *wp, linenr_T lnum);
extern void rs_set_valid_virtcol(win_T *wp, colnr_T vcol);
extern void rs_cursor_correct(win_T *wp);
extern int rs_get_scroll_overlap(int dir);
extern int rs_scroll_with_sms(int dir, int count, int *curscount);
extern int rs_pagescroll(int dir, int count, int half);
extern void rs_redraw_for_cursorline(win_T *wp);
extern void rs_redraw_for_cursorcolumn(win_T *wp);
extern int rs_plines_correct_topline(win_T *wp, linenr_T lnum, linenr_T *nextp,
                                     int limit_winheight, int *foldedp);
extern void rs_validate_virtcol(win_T *wp);
extern void rs_validate_cheight(win_T *wp);
extern void rs_validate_botline(win_T *wp);
extern void rs_validate_cursor(win_T *wp);
extern void rs_update_curswant(void);
extern void rs_update_curswant_force(void);
extern void rs_comp_botline(win_T *wp);
extern void rs_topline_back_winheight(win_T *wp, lineoff_T *lp, int winheight);
extern void rs_topline_back(win_T *wp, lineoff_T *lp);
extern void rs_botline_forw(win_T *wp, lineoff_T *lp);
extern int rs_virtcol2col(win_T *wp, linenr_T lnum, int vcol);
extern void rs_textpos2screenpos(win_T *wp, pos_T *pos, int *rowp, int *scolp,
                                  int *ccolp, int *ecolp, int local);
extern void rs_do_check_cursorbind(void);
extern void rs_curs_rows(win_T *wp);
extern void rs_curs_columns(win_T *wp, int may_scroll);

// Accessor for global scrolljump option
OptInt nvim_get_p_sj(void)
{
  return p_sj;
}

/// Get the number of screen lines skipped with "wp->w_skipcol".
static int adjust_plines_for_skipcol(win_T *wp)
{
  return rs_adjust_plines_for_skipcol(wp);
}

/// Return how many lines "lnum" will take on the screen, taking into account
/// whether it is the first line, whether w_skipcol is non-zero and limiting to
/// the window height.
int plines_correct_topline(win_T *wp, linenr_T lnum, linenr_T *nextp, bool limit_winheight,
                           bool *foldedp)
{
  int folded = 0;
  int result = rs_plines_correct_topline(wp, lnum, nextp, limit_winheight ? 1 : 0, &folded);
  if (foldedp) {
    *foldedp = (folded != 0);
  }
  return result;
}

// Compute wp->w_botline for the current wp->w_topline.  Can be called after
// wp->w_topline changed.
static void comp_botline(win_T *wp)
{
  rs_comp_botline(wp);
}

/// Redraw when w_cline_row changes and 'relativenumber' or 'cursorline' is set.
/// Also when concealing is on and 'concealcursor' is not active.
static void redraw_for_cursorline(win_T *wp)
  FUNC_ATTR_NONNULL_ALL
{
  rs_redraw_for_cursorline(wp);
}

/// Redraw when 'concealcursor' is active, or when w_virtcol changes and:
/// - 'cursorcolumn' is set, or
/// - 'cursorlineopt' contains "screenline", or
/// - Visual mode is active.
static void redraw_for_cursorcolumn(win_T *wp)
  FUNC_ATTR_NONNULL_ALL
{
  rs_redraw_for_cursorcolumn(wp);
}

/// Set wp->w_virtcol to a value ("vcol") that is already valid.
/// Handles redrawing if wp->w_virtcol was previously invalid.
void set_valid_virtcol(win_T *wp, colnr_T vcol)
{
  rs_set_valid_virtcol(wp, vcol);
}

/// Calculates how much the 'listchars' "precedes" or 'smoothscroll' "<<<"
/// marker overlaps with buffer text for window "wp".
/// Parameter "extra2" should be the padding on the 2nd line, not the first
/// line. When "extra2" is -1 calculate the padding.
/// Returns the number of columns of overlap with buffer text, excluding the
/// extra padding on the ledge.
int sms_marker_overlap(win_T *wp, int extra2)
{
  return rs_sms_marker_overlap(wp, extra2);
}

/// Calculates the skipcol offset for window "wp" given how many
/// physical lines we want to scroll down.
static int skipcol_from_plines(win_T *wp, int plines_off)
{
  return rs_skipcol_from_plines(wp, plines_off);
}

/// Set wp->w_skipcol to zero and redraw later if needed.
static void reset_skipcol(win_T *wp)
{
  rs_reset_skipcol(wp);
}

// Rust implementation
extern void rs_update_topline(win_T *wp);

// Update wp->w_topline to move the cursor onto the screen.
void update_topline(win_T *wp)
{
  rs_update_topline(wp);
}

/// Return the scrolljump value to use for the window "wp".
/// When 'scrolljump' is positive use it as-is.
/// When 'scrolljump' is negative use it as a percentage of the window height.
static int scrolljump_value(win_T *wp)
{
  return rs_scrolljump_value(wp);
}

/// Return true when there are not 'scrolloff' lines above the cursor for window "wp".
static bool check_top_offset(win_T *wp)
{
  return rs_check_top_offset(wp) != 0;
}

/// Update w_curswant.
void update_curswant_force(void)
{
  rs_update_curswant_force();
}

/// Update w_curswant if w_set_curswant is set.
void update_curswant(void)
{
  rs_update_curswant();
}

// Rust implementation of check_cursor_moved
extern void rs_check_cursor_moved(win_T *wp);

// Check if the cursor has moved.  Set the w_valid flag accordingly.
void check_cursor_moved(win_T *wp)
{
  rs_check_cursor_moved(wp);
}

// Rust implementations of window setting functions
extern void rs_changed_window_setting(win_T *wp);
extern void rs_changed_window_setting_all(void);

// Call this function when some window settings have changed, which require
// the cursor position, botline and topline to be recomputed and the window to
// be redrawn.  E.g, when changing the 'wrap' option or folding.
void changed_window_setting(win_T *wp)
{
  rs_changed_window_setting(wp);
}

/// Call changed_window_setting() for every window.
void changed_window_setting_all(void)
{
  rs_changed_window_setting_all();
}

// Set wp->w_topline to a certain number.
void set_topline(win_T *wp, linenr_T lnum)
{
  rs_set_topline(wp, lnum);
}

// Rust implementations of validity flag functions
extern void rs_invalidate_botline(win_T *wp);
extern void rs_approximate_botline_win(win_T *wp);
extern void rs_changed_cline_bef_curs(win_T *wp);
extern void rs_changed_line_abv_curs(void);
extern void rs_changed_line_abv_curs_win(win_T *wp);

/// Call this function when the length of the cursor line (in screen
/// characters) has changed, and the change is before the cursor.
/// If the line length changed the number of screen lines might change,
/// requiring updating w_topline.  That may also invalidate w_crow.
/// Need to take care of w_botline separately!
void changed_cline_bef_curs(win_T *wp)
{
  rs_changed_cline_bef_curs(wp);
}

// Call this function when the length of a line (in screen characters) above
// the cursor have changed.
// Need to take care of w_botline separately!
void changed_line_abv_curs(void)
{
  rs_changed_line_abv_curs();
}

void changed_line_abv_curs_win(win_T *wp)
{
  rs_changed_line_abv_curs_win(wp);
}

// Make sure the value of wp->w_botline is valid.
void validate_botline(win_T *wp)
{
  rs_validate_botline(wp);
}

// Mark wp->w_botline as invalid (because of some change in the buffer).
void invalidate_botline(win_T *wp)
{
  rs_invalidate_botline(wp);
}

void approximate_botline_win(win_T *wp)
{
  rs_approximate_botline_win(wp);
}

// Rust implementation of cursor_valid
extern int rs_cursor_valid(win_T *wp);

// Return true if wp->w_wrow and wp->w_wcol are valid.
int cursor_valid(win_T *wp)
{
  return rs_cursor_valid(wp);
}

// Validate cursor position.  Makes sure w_wrow and w_wcol are valid.
// w_topline must be valid, you may need to call update_topline() first!
void validate_cursor(win_T *wp)
{
  rs_validate_cursor(wp);
}

// Compute wp->w_cline_row and wp->w_cline_height, based on the current value
// of wp->w_topline.
static void curs_rows(win_T *wp)
{
  rs_curs_rows(wp);
}

// Validate wp->w_virtcol only.
void validate_virtcol(win_T *wp)
{
  rs_validate_virtcol(wp);
}

// Validate wp->w_cline_height only.
void validate_cheight(win_T *wp)
{
  rs_validate_cheight(wp);
}

// Validate w_wcol and w_virtcol only.
void validate_cursor_col(win_T *wp)
{
  validate_virtcol(wp);
  rs_compute_wcol(wp);
}

// Compute offset of a window, occupied by absolute or relative line number,
// fold column and sign column (these don't move when scrolling horizontally).
int win_col_off(win_T *wp)
{
  return rs_win_col_off(wp);
}

// Return the difference in column offset for the second screen line of a
// wrapped line.  It's positive if 'number' or 'relativenumber' is on and 'n'
// is in 'cpoptions'.
int win_col_off2(win_T *wp)
{
  return rs_win_col_off2(wp);
}

// Compute wp->w_wcol and wp->w_virtcol.
// Also updates wp->w_wrow and wp->w_cline_row.
// Also updates wp->w_leftcol.
// @param may_scroll when true, may scroll horizontally
void curs_columns(win_T *wp, int may_scroll)
{
  rs_curs_columns(wp, may_scroll);
}

/// Compute the screen position of text character at "pos" in window "wp"
/// The resulting values are one-based, zero when character is not visible.
///
/// @param[out] rowp screen row
/// @param[out] scolp start screen column
/// @param[out] ccolp cursor screen column
/// @param[out] ecolp end screen column
void textpos2screenpos(win_T *wp, pos_T *pos, int *rowp, int *scolp, int *ccolp, int *ecolp,
                       bool local)
{
  rs_textpos2screenpos(wp, pos, rowp, scolp, ccolp, ecolp, local ? 1 : 0);
}

/// "screenpos({winid}, {lnum}, {col})" function
void f_screenpos(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  tv_dict_alloc_ret(rettv);
  dict_T *dict = rettv->vval.v_dict;

  win_T *wp = find_win_by_nr_or_id(&argvars[0]);
  if (wp == NULL) {
    return;
  }

  pos_T pos = {
    .lnum = (linenr_T)tv_get_number(&argvars[1]),
    .col = (colnr_T)tv_get_number(&argvars[2]) - 1,
    .coladd = 0
  };
  if (pos.lnum > wp->w_buffer->b_ml.ml_line_count) {
    semsg(_(e_invalid_line_number_nr), pos.lnum);
    return;
  }
  pos.col = MAX(pos.col, 0);
  int row = 0;
  int scol = 0;
  int ccol = 0;
  int ecol = 0;
  textpos2screenpos(wp, &pos, &row, &scol, &ccol, &ecol, false);

  tv_dict_add_nr(dict, S_LEN("row"), row);
  tv_dict_add_nr(dict, S_LEN("col"), scol);
  tv_dict_add_nr(dict, S_LEN("curscol"), ccol);
  tv_dict_add_nr(dict, S_LEN("endcol"), ecol);
}

/// Convert a virtual (screen) column to a character column.  The first column
/// is one.  For a multibyte character, the column number of the first byte is
/// returned.
static int virtcol2col(win_T *wp, linenr_T lnum, int vcol)
{
  return rs_virtcol2col(wp, lnum, vcol);
}

/// "virtcol2col({winid}, {lnum}, {col})" function
void f_virtcol2col(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rettv->vval.v_number = -1;

  if (tv_check_for_number_arg(argvars, 0) == FAIL
      || tv_check_for_number_arg(argvars, 1) == FAIL
      || tv_check_for_number_arg(argvars, 2) == FAIL) {
    return;
  }

  win_T *wp = find_win_by_nr_or_id(&argvars[0]);
  if (wp == NULL) {
    return;
  }

  bool error = false;
  linenr_T lnum = (linenr_T)tv_get_number_chk(&argvars[1], &error);
  if (error || lnum < 0 || lnum > wp->w_buffer->b_ml.ml_line_count) {
    return;
  }

  int screencol = (int)tv_get_number_chk(&argvars[2], &error);
  if (error || screencol < 0) {
    return;
  }

  rettv->vval.v_number = virtcol2col(wp, lnum, screencol);
}

/// Make sure the cursor is in the visible part of the topline after scrolling
/// the screen with 'smoothscroll'.
static void cursor_correct_sms(win_T *wp)
{
  rs_cursor_correct_sms(wp);
}

/// Scroll "count" lines up or down, and redraw.
void scroll_redraw(int up, linenr_T count)
{
  rs_scroll_redraw(up, count);
}

/// Scroll a window down by "line_count" logical lines.  "CTRL-Y"
///
/// @param line_count number of lines to scroll
/// @param byfold if true, count a closed fold as one line
bool scrolldown(win_T *wp, linenr_T line_count, int byfold)
{
  return rs_scrolldown(wp, line_count, byfold) != 0;
}

/// Scroll a window up by "line_count" logical lines.  "CTRL-E"
///
/// @param line_count number of lines to scroll
/// @param byfold if true, count a closed fold as one line
bool scrollup(win_T *wp, linenr_T line_count, bool byfold)
{
  return rs_scrollup(wp, line_count, byfold ? 1 : 0) != 0;
}

/// Called after changing the cursor column: make sure that curwin->w_skipcol is
/// valid for 'smoothscroll'.
void adjust_skipcol(void)
{
  rs_adjust_skipcol();
}

/// Don't end up with too many filler lines in the window.
///
/// @param down  when true scroll down when not enough space
void check_topfill(win_T *wp, bool down)
{
  rs_check_topfill(wp, down ? 1 : 0);
}

// Scroll the screen one line down, but don't do it if it would move the
// cursor off the screen.
void scrolldown_clamp(void)
{
  rs_scrolldown_clamp();
}

// Scroll the screen one line up, but don't do it if it would move the cursor
// off the screen.
void scrollup_clamp(void)
{
  rs_scrollup_clamp();
}

// Add one line above "lp->lnum".  This can be a filler line, a closed fold or
// a (wrapped) text line.  Uses and sets "lp->fill".
// Returns the height of the added line in "lp->height".
// Lines above the first one are incredibly high: MAXCOL.
// When "winheight" is true limit to window height.
static void topline_back_winheight(win_T *wp, lineoff_T *lp, int winheight)
{
  rs_topline_back_winheight(wp, lp, winheight);
}

static void topline_back(win_T *wp, lineoff_T *lp)
{
  rs_topline_back(wp, lp);
}

// Add one line below "lp->lnum".  This can be a filler line, a closed fold or
// a (wrapped) text line.  Uses and sets "lp->fill".
// Returns the height of the added line in "lp->height".
// Lines below the last one are incredibly high.
static void botline_forw(win_T *wp, lineoff_T *lp)
{
  rs_botline_forw(wp, lp);
}

// Recompute topline to put the cursor at the top of the window.
// Scroll at least "min_scroll" lines.
// If "always" is true, always set topline (for "zt").
void scroll_cursor_top(win_T *wp, int min_scroll, int always)
{
  rs_scroll_cursor_top(wp, min_scroll, always);
}

// Set w_empty_rows and w_filler_rows for window "wp", having used up "used"
// screen lines for text lines.
void set_empty_rows(win_T *wp, int used)
{
  rs_set_empty_rows(wp, used);
}

/// Recompute topline to put the cursor at the bottom of the window.
/// When scrolling scroll at least "min_scroll" lines.
/// If "set_topbot" is true, set topline and botline first (for "zb").
/// This is messy stuff!!!
void scroll_cursor_bot(win_T *wp, int min_scroll, bool set_topbot)
{
  rs_scroll_cursor_bot(wp, min_scroll, set_topbot ? 1 : 0);
}

/// Recompute topline to put the cursor halfway across the window
///
/// @param atend if true, also put the cursor halfway to the end of the file.
///
void scroll_cursor_halfway(win_T *wp, bool atend, bool prefer_above)
{
  rs_scroll_cursor_halfway(wp, atend ? 1 : 0, prefer_above ? 1 : 0);
}

// Correct the cursor position so that it is in a part of the screen at least
// 'so' lines from the top and bottom, if possible.
// If not possible, put it at the same position as scroll_cursor_halfway().
// When called topline must be valid!
void cursor_correct(win_T *wp)
{
  rs_cursor_correct(wp);
}

/// Decide how much overlap to use for page-up or page-down scrolling.
/// This is symmetric, so that doing both keeps the same lines displayed.
/// Three lines are examined:
///
///  before CTRL-F          after CTRL-F / before CTRL-B
///     etc.                    l1
///  l1 last but one line       ------------
///  l2 last text line          l2 top text line
///  -------------              l3 second text line
///  l3                            etc.
static int get_scroll_overlap(Direction dir)
{
  return rs_get_scroll_overlap((int)dir);
}

/// Scroll "count" lines with 'smoothscroll' in direction "dir". Return true
/// when scrolling happened. Adjust "curscount" for scrolling different amount
/// of lines when 'smoothscroll' is disabled.
static bool scroll_with_sms(Direction dir, int count, int *curscount)
{
  return rs_scroll_with_sms((int)dir, count, curscount) != 0;
}

/// Move screen "count" (half) pages up ("dir" is BACKWARD) or down ("dir" is
/// FORWARD) and update the screen. Handle moving the cursor and not scrolling
/// to reveal end of buffer lines for half-page scrolling with CTRL-D and CTRL-U.
///
/// @return  FAIL for failure, OK otherwise.
int pagescroll(Direction dir, int count, bool half)
{
  return rs_pagescroll((int)dir, count, half ? 1 : 0);
}

void do_check_cursorbind(void)
{
  rs_do_check_cursorbind();
}

// =============================================================================
// C Wrappers for Rust FFI
// =============================================================================

/// Wrapper for cursor_correct() (accessor for Rust).
void nvim_cursor_correct(win_T *wp)
{
  cursor_correct(wp);
}

/// Wrapper for validate_cursor() with window parameter (accessor for Rust).
void nvim_validate_cursor_win(win_T *wp)
{
  validate_cursor(wp);
}

/// Wrapper for validate_virtcol() (accessor for Rust).
void nvim_validate_virtcol(win_T *wp)
{
  validate_virtcol(wp);
}

/// Wrapper for validate_cheight() (accessor for Rust).
void nvim_validate_cheight(win_T *wp)
{
  validate_cheight(wp);
}

/// Wrapper for check_topfill() (accessor for Rust).
void nvim_check_topfill(win_T *wp, int down)
{
  check_topfill(wp, down != 0);
}

/// Wrapper for invalidate_botline() (accessor for Rust).
void nvim_invalidate_botline(win_T *wp)
{
  invalidate_botline(wp);
}

/// Wrapper for win_col_off() (accessor for Rust).
int nvim_win_col_off(win_T *wp)
{
  return win_col_off(wp);
}

/// Wrapper for win_col_off2() (accessor for Rust).
int nvim_win_col_off2(win_T *wp)
{
  return win_col_off2(wp);
}

/// Wrapper for cursor_correct_sms() (accessor for Rust).
void nvim_cursor_correct_sms(win_T *wp)
{
  cursor_correct_sms(wp);
}

/// Wrapper for validate_botline() (accessor for Rust).
void nvim_validate_botline(win_T *wp)
{
  validate_botline(wp);
}

/// Wrapper for plines_win_full() (accessor for Rust).
int nvim_plines_win_full(win_T *wp, linenr_T lnum, linenr_T *nextp, int *foldedp,
                         int cache, int limit_winheight)
{
  bool folded = false;
  int result = plines_win_full(wp, lnum, nextp, &folded, cache != 0, limit_winheight != 0);
  if (foldedp) {
    *foldedp = folded ? 1 : 0;
  }
  return result;
}

/// Get line count from curbuf (accessor for Rust).
linenr_T nvim_curbuf_line_count(void)
{
  return curbuf->b_ml.ml_line_count;
}

/// Wrapper for redraw_for_cursorcolumn() (accessor for Rust).
void nvim_redraw_for_cursorcolumn(win_T *wp)
{
  redraw_for_cursorcolumn(wp);
}

/// Wrapper for comp_botline() (accessor for Rust).
void nvim_comp_botline(win_T *wp)
{
  comp_botline(wp);
}

/// Wrapper for curs_columns() (accessor for Rust).
void nvim_curs_columns(win_T *wp, int may_scroll)
{
  curs_columns(wp, may_scroll);
}

/// Wrapper for update_topline() (accessor for Rust).
void nvim_update_topline(win_T *wp)
{
  update_topline(wp);
}

// =============================================================================
// C Wrapper Functions for pagescroll() Rust Migration
// =============================================================================

/// Wrapper for cursor_down_inner() (accessor for Rust).
void nvim_cursor_down_inner(win_T *wp, int n, int skip_conceal)
{
  cursor_down_inner(wp, n, skip_conceal != 0);
}

/// Wrapper for cursor_up_inner() (accessor for Rust).
void nvim_cursor_up_inner(win_T *wp, linenr_T n, int skip_conceal)
{
  cursor_up_inner(wp, n, skip_conceal != 0);
}

/// Wrapper for nv_screengo() (accessor for Rust).
/// Returns 1 on success, 0 on failure.
int nvim_nv_screengo(int dir, int dist, int skip_conceal)
{
  oparg_T oa = { 0 };
  return nv_screengo(&oa, dir, dist, skip_conceal != 0) ? 1 : 0;
}

/// Wrapper for beginline() (accessor for Rust).
void nvim_beginline_flags(int flags)
{
  beginline(flags);
}

/// Wrapper for beep_flush() (accessor for Rust).
void nvim_beep_flush_wrapper(void)
{
  beep_flush();
}

/// Wrapper for nv_g_home_m_cmd() (accessor for Rust).
void nvim_nv_g_home_m_cmd(void)
{
  oparg_T oa = { 0 };
  cmdarg_T ca = { 0 };
  ca.oap = &oa;
  nv_g_home_m_cmd(&ca);
}

/// Check if ONE_WINDOW macro is true (accessor for Rust).
int nvim_one_window(void)
{
  return ONE_WINDOW ? 1 : 0;
}

/// Get p_sol option value (accessor for Rust).
int nvim_get_p_sol(void)
{
  return p_sol ? 1 : 0;
}

/// Get Rows value (accessor for Rust).
int nvim_get_rows_val(void)
{
  return Rows;
}

/// Get w_p_scr (scroll option) value (accessor for Rust).
OptInt nvim_win_get_p_scr(win_T *wp)
{
  return wp ? wp->w_p_scr : 0;
}

/// Set w_p_scr (scroll option) value (accessor for Rust).
void nvim_win_set_p_scr(win_T *wp, OptInt val)
{
  if (wp) {
    wp->w_p_scr = val;
  }
}

/// Wrapper for plines_correct_topline() (accessor for Rust).
int nvim_plines_correct_topline(win_T *wp, linenr_T lnum, int limit_winheight)
{
  return plines_correct_topline(wp, lnum, NULL, limit_winheight != 0, NULL);
}

/// Wrapper for plines_m_win() (accessor for Rust).
int nvim_plines_m_win(win_T *wp, linenr_T first, linenr_T last, int max)
{
  return plines_m_win(wp, first, last, max);
}

/// Get skip_update_topline flag (accessor for Rust).
int nvim_get_skip_update_topline(void)
{
  return skip_update_topline ? 1 : 0;
}

/// Wrapper for changed_cline_bef_curs (accessor for Rust).
void nvim_changed_cline_bef_curs(win_T *wp)
{
  changed_cline_bef_curs(wp);
}
