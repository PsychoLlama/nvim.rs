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
  int n = plines_win_full(wp, lnum, nextp, foldedp, true, false);
  if (lnum == wp->w_topline) {
    n -= adjust_plines_for_skipcol(wp);
  }
  if (limit_winheight && n > wp->w_view_height) {
    return wp->w_view_height;
  }
  return n;
}

// Compute wp->w_botline for the current wp->w_topline.  Can be called after
// wp->w_topline changed.
static void comp_botline(win_T *wp)
{
  linenr_T lnum;
  int done;

  // If w_cline_row is valid, start there.
  // Otherwise have to start at w_topline.
  check_cursor_moved(wp);
  if (wp->w_valid & VALID_CROW) {
    lnum = wp->w_cursor.lnum;
    done = wp->w_cline_row;
  } else {
    lnum = wp->w_topline;
    done = 0;
  }

  for (; lnum <= wp->w_buffer->b_ml.ml_line_count; lnum++) {
    linenr_T last = lnum;
    bool folded;
    int n = plines_correct_topline(wp, lnum, &last, true, &folded);
    if (lnum <= wp->w_cursor.lnum && last >= wp->w_cursor.lnum) {
      wp->w_cline_row = done;
      wp->w_cline_height = n;
      wp->w_cline_folded = folded;
      redraw_for_cursorline(wp);
      wp->w_valid |= (VALID_CROW|VALID_CHEIGHT);
    }
    if (done + n > wp->w_view_height) {
      break;
    }
    done += n;
    lnum = last;
  }

  // wp->w_botline is the line that is just below the window
  wp->w_botline = lnum;
  wp->w_valid |= VALID_BOTLINE|VALID_BOTLINE_AP;
  wp->w_viewport_invalid = true;

  set_empty_rows(wp, done);

  win_check_anchored_floats(wp);
}

/// Redraw when w_cline_row changes and 'relativenumber' or 'cursorline' is set.
/// Also when concealing is on and 'concealcursor' is not active.
static void redraw_for_cursorline(win_T *wp)
  FUNC_ATTR_NONNULL_ALL
{
  if ((wp->w_valid & VALID_CROW) == 0 && !pum_visible()
      && (wp->w_p_rnu || win_cursorline_standout(wp))) {
    // win_line() will redraw the number column and cursorline only.
    redraw_later(wp, UPD_VALID);
  }
}

/// Redraw when 'concealcursor' is active, or when w_virtcol changes and:
/// - 'cursorcolumn' is set, or
/// - 'cursorlineopt' contains "screenline", or
/// - Visual mode is active.
static void redraw_for_cursorcolumn(win_T *wp)
  FUNC_ATTR_NONNULL_ALL
{
  // If the cursor moves horizontally when 'concealcursor' is active, then the
  // current line needs to be redrawn to calculate the correct cursor position.
  if (wp == curwin && wp->w_p_cole > 0 && conceal_cursor_line(wp)) {
    redrawWinline(wp, wp->w_cursor.lnum);
  }

  if ((wp->w_valid & VALID_VIRTCOL) || pum_visible()) {
    return;
  }

  if (wp->w_p_cuc) {
    // When 'cursorcolumn' is set need to redraw with UPD_SOME_VALID.
    redraw_later(wp, UPD_SOME_VALID);
  } else if (wp->w_p_cul && (wp->w_p_culopt_flags & kOptCuloptFlagScreenline)) {
    // When 'cursorlineopt' contains "screenline" need to redraw with UPD_VALID.
    redraw_later(wp, UPD_VALID);
  }

  // When current buffer's cursor moves in Visual mode, redraw it with UPD_INVERTED.
  if (VIsual_active && wp->w_buffer == curbuf) {
    redraw_buf_later(curbuf, UPD_INVERTED);
  }
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

// Update wp->w_topline to move the cursor onto the screen.
void update_topline(win_T *wp)
{
  bool check_botline = false;
  OptInt *so_ptr = wp->w_p_so >= 0 ? &wp->w_p_so : &p_so;
  OptInt save_so = *so_ptr;

  // Cursor is updated instead when this is true for 'splitkeep'.
  if (skip_update_topline) {
    return;
  }

  // If there is no valid screen and when the window height is zero just use
  // the cursor line.
  if (!default_grid.chars || wp->w_view_height == 0) {
    wp->w_topline = wp->w_cursor.lnum;
    wp->w_botline = wp->w_topline;
    wp->w_viewport_invalid = true;
    wp->w_scbind_pos = 1;
    return;
  }

  check_cursor_moved(wp);
  if (wp->w_valid & VALID_TOPLINE) {
    return;
  }

  // When dragging with the mouse, don't scroll that quickly
  if (mouse_dragging > 0) {
    *so_ptr = mouse_dragging - 1;
  }

  linenr_T old_topline = wp->w_topline;
  int old_topfill = wp->w_topfill;

  // If the buffer is empty, always set topline to 1.
  if (buf_is_empty(wp->w_buffer)) {  // special case - file is empty
    if (wp->w_topline != 1) {
      redraw_later(wp, UPD_NOT_VALID);
    }
    wp->w_topline = 1;
    wp->w_botline = 2;
    wp->w_skipcol = 0;
    wp->w_valid |= VALID_BOTLINE|VALID_BOTLINE_AP;
    wp->w_viewport_invalid = true;
    wp->w_scbind_pos = 1;
  } else {
    bool check_topline = false;
    // If the cursor is above or near the top of the window, scroll the window
    // to show the line the cursor is in, with 'scrolloff' context.
    if (wp->w_topline > 1 || wp->w_skipcol > 0) {
      // If the cursor is above topline, scrolling is always needed.
      // If the cursor is far below topline and there is no folding,
      // scrolling down is never needed.
      if (wp->w_cursor.lnum < wp->w_topline) {
        check_topline = true;
      } else if (check_top_offset(wp)) {
        check_topline = true;
      } else if (wp->w_skipcol > 0 && wp->w_cursor.lnum == wp->w_topline) {
        colnr_T vcol;

        // Check that the cursor position is visible.  Add columns for
        // the marker displayed in the top-left if needed.
        getvvcol(wp, &wp->w_cursor, &vcol, NULL, NULL);
        int overlap = sms_marker_overlap(wp, -1);
        if (wp->w_skipcol + overlap > vcol) {
          check_topline = true;
        }
      }
    }
    // Check if there are more filler lines than allowed.
    if (!check_topline && wp->w_topfill > win_get_fill(wp, wp->w_topline)) {
      check_topline = true;
    }

    if (check_topline) {
      int halfheight = wp->w_view_height / 2 - 1;
      if (halfheight < 2) {
        halfheight = 2;
      }
      int64_t n;
      if (win_lines_concealed(wp)) {
        // Count the number of logical lines between the cursor and
        // topline + p_so (approximation of how much will be
        // scrolled).
        n = 0;
        for (linenr_T lnum = wp->w_cursor.lnum; lnum < wp->w_topline + *so_ptr; lnum++) {
          // stop at end of file or when we know we are far off
          assert(wp->w_buffer != 0);
          if (lnum >= wp->w_buffer->b_ml.ml_line_count
              || (n += !decor_conceal_line(wp, lnum, false)) >= halfheight) {
            break;
          }
          hasFolding(wp, lnum, NULL, &lnum);
        }
      } else {
        n = wp->w_topline + *so_ptr - wp->w_cursor.lnum;
      }

      // If we weren't very close to begin with, we scroll to put the
      // cursor in the middle of the window.  Otherwise put the cursor
      // near the top of the window.
      if (n >= halfheight) {
        scroll_cursor_halfway(wp, false, false);
      } else {
        scroll_cursor_top(wp, scrolljump_value(wp), false);
        check_botline = true;
      }
    } else {
      // Make sure topline is the first line of a fold.
      hasFolding(wp, wp->w_topline, &wp->w_topline, NULL);
      check_botline = true;
    }
  }

  // If the cursor is below the bottom of the window, scroll the window
  // to put the cursor on the window.
  // When w_botline is invalid, recompute it first, to avoid a redraw later.
  // If w_botline was approximated, we might need a redraw later in a few
  // cases, but we don't want to spend (a lot of) time recomputing w_botline
  // for every small change.
  if (check_botline) {
    if (!(wp->w_valid & VALID_BOTLINE_AP)) {
      validate_botline(wp);
    }

    assert(wp->w_buffer != 0);
    if (wp->w_botline <= wp->w_buffer->b_ml.ml_line_count) {
      if (wp->w_cursor.lnum < wp->w_botline) {
        if ((wp->w_cursor.lnum >= wp->w_botline - *so_ptr || win_lines_concealed(wp))) {
          lineoff_T loff;

          // Cursor is (a few lines) above botline, check if there are
          // 'scrolloff' window lines below the cursor.  If not, need to
          // scroll.
          int n = wp->w_empty_rows;
          loff.lnum = wp->w_cursor.lnum;
          // In a fold go to its last line.
          hasFolding(wp, loff.lnum, NULL, &loff.lnum);
          loff.fill = 0;
          n += wp->w_filler_rows;
          loff.height = 0;
          while (loff.lnum < wp->w_botline
                 && (loff.lnum + 1 < wp->w_botline || loff.fill == 0)) {
            n += loff.height;
            if (n >= *so_ptr) {
              break;
            }
            botline_forw(wp, &loff);
          }
          if (n >= *so_ptr) {
            // sufficient context, no need to scroll
            check_botline = false;
          }
        } else {
          // sufficient context, no need to scroll
          check_botline = false;
        }
      }
      if (check_botline) {
        int n = 0;
        if (win_lines_concealed(wp)) {
          // Count the number of logical lines between the cursor and
          // botline - p_so (approximation of how much will be
          // scrolled).
          for (linenr_T lnum = wp->w_cursor.lnum; lnum >= wp->w_botline - *so_ptr; lnum--) {
            // stop at end of file or when we know we are far off
            if (lnum <= 0
                || (n += !decor_conceal_line(wp, lnum - 1, false)) > wp->w_view_height + 1) {
              break;
            }
            hasFolding(wp, lnum, &lnum, NULL);
          }
        } else {
          n = wp->w_cursor.lnum - wp->w_botline + 1 + (int)(*so_ptr);
        }
        if (n <= wp->w_view_height + 1) {
          scroll_cursor_bot(wp, scrolljump_value(wp), false);
        } else {
          scroll_cursor_halfway(wp, false, false);
        }
      }
    }
  }
  wp->w_valid |= VALID_TOPLINE;
  wp->w_viewport_invalid = true;
  win_check_anchored_floats(wp);

  // Need to redraw when topline changed.
  if (wp->w_topline != old_topline
      || wp->w_topfill != old_topfill) {
    dollar_vcol = -1;
    redraw_later(wp, UPD_VALID);

    // When 'smoothscroll' is not set, should reset w_skipcol.
    if (!wp->w_p_sms) {
      reset_skipcol(wp);
    } else if (wp->w_skipcol != 0) {
      redraw_later(wp, UPD_SOME_VALID);
    }

    // May need to set w_skipcol when cursor in w_topline.
    if (wp->w_cursor.lnum == wp->w_topline) {
      validate_cursor(wp);
    }
  }

  *so_ptr = save_so;
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
  validate_virtcol(curwin);
  curwin->w_curswant = curwin->w_virtcol;
  curwin->w_set_curswant = false;
}

/// Update w_curswant if w_set_curswant is set.
void update_curswant(void)
{
  if (curwin->w_set_curswant) {
    update_curswant_force();
  }
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
  if (!(wp->w_valid & VALID_BOTLINE)) {
    comp_botline(wp);
  }
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
  check_cursor_lnum(wp);
  check_cursor_moved(wp);
  if ((wp->w_valid & (VALID_WCOL|VALID_WROW)) != (VALID_WCOL|VALID_WROW)) {
    curs_columns(wp, true);
  }
}

// Compute wp->w_cline_row and wp->w_cline_height, based on the current value
// of wp->w_topline.
static void curs_rows(win_T *wp)
{
  // Check if wp->w_lines[].wl_size is invalid
  bool all_invalid = (!redrawing()
                      || wp->w_lines_valid == 0
                      || wp->w_lines[0].wl_lnum > wp->w_topline);
  int i = 0;
  wp->w_cline_row = 0;
  for (linenr_T lnum = wp->w_topline; lnum < wp->w_cursor.lnum; i++) {
    bool valid = false;
    if (!all_invalid && i < wp->w_lines_valid) {
      if (wp->w_lines[i].wl_lnum < lnum || !wp->w_lines[i].wl_valid) {
        continue;                       // skip changed or deleted lines
      }
      if (wp->w_lines[i].wl_lnum == lnum) {
        // Check for newly inserted lines below this row, in which
        // case we need to check for folded lines.
        if (!wp->w_buffer->b_mod_set
            || wp->w_lines[i].wl_lastlnum < wp->w_cursor.lnum
            || wp->w_buffer->b_mod_top
            > wp->w_lines[i].wl_lastlnum + 1) {
          valid = true;
        }
      } else if (wp->w_lines[i].wl_lnum > lnum) {
        i--;                            // hold at inserted lines
      }
    }
    if (valid && (lnum != wp->w_topline || (wp->w_skipcol == 0 && !win_may_fill(wp)))) {
      lnum = wp->w_lines[i].wl_lastlnum + 1;
      // Cursor inside folded or concealed lines, don't count this row
      if (lnum > wp->w_cursor.lnum) {
        break;
      }
      wp->w_cline_row += wp->w_lines[i].wl_size;
    } else {
      linenr_T last = lnum;
      bool folded;
      int n = plines_correct_topline(wp, lnum, &last, true, &folded);
      lnum = last + 1;
      if (lnum + decor_conceal_line(wp, lnum - 1, false) > wp->w_cursor.lnum) {
        break;
      }
      wp->w_cline_row += n;
    }
  }

  check_cursor_moved(wp);
  if (!(wp->w_valid & VALID_CHEIGHT)) {
    if (all_invalid
        || i == wp->w_lines_valid
        || (i < wp->w_lines_valid
            && (!wp->w_lines[i].wl_valid
                || wp->w_lines[i].wl_lnum != wp->w_cursor.lnum))) {
      wp->w_cline_height = plines_win_full(wp, wp->w_cursor.lnum, NULL,
                                           &wp->w_cline_folded, true, true);
    } else if (i > wp->w_lines_valid) {
      // a line that is too long to fit on the last screen line
      wp->w_cline_height = 0;
      wp->w_cline_folded = hasFolding(wp, wp->w_cursor.lnum, NULL, NULL);
    } else {
      wp->w_cline_height = wp->w_lines[i].wl_size;
      wp->w_cline_folded = wp->w_lines[i].wl_folded;
    }
  }

  redraw_for_cursorline(wp);
  wp->w_valid |= VALID_CROW|VALID_CHEIGHT;
}

// Validate wp->w_virtcol only.
void validate_virtcol(win_T *wp)
{
  check_cursor_moved(wp);

  if (wp->w_valid & VALID_VIRTCOL) {
    return;
  }

  getvvcol(wp, &wp->w_cursor, NULL, &(wp->w_virtcol), NULL);
  redraw_for_cursorcolumn(wp);
  wp->w_valid |= VALID_VIRTCOL;
}

// Validate wp->w_cline_height only.
void validate_cheight(win_T *wp)
{
  check_cursor_moved(wp);

  if (wp->w_valid & VALID_CHEIGHT) {
    return;
  }

  wp->w_cline_height = plines_win_full(wp, wp->w_cursor.lnum,
                                       NULL, &wp->w_cline_folded,
                                       true, true);
  wp->w_valid |= VALID_CHEIGHT;
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
  colnr_T startcol;
  colnr_T endcol;

  // First make sure that w_topline is valid (after moving the cursor).
  update_topline(wp);

  // Next make sure that w_cline_row is valid.
  if (!(wp->w_valid & VALID_CROW)) {
    curs_rows(wp);
  }

  // Compute the number of virtual columns.
  if (wp->w_cline_folded) {
    // In a folded line the cursor is always in the first column
    startcol = wp->w_virtcol = endcol = wp->w_leftcol;
  } else {
    getvvcol(wp, &wp->w_cursor, &startcol, &(wp->w_virtcol), &endcol);
  }

  // remove '$' from change command when cursor moves onto it
  if (startcol > dollar_vcol) {
    dollar_vcol = -1;
  }

  int extra = win_col_off(wp);
  wp->w_wcol = wp->w_virtcol + extra;
  endcol += extra;

  // Now compute w_wrow, counting screen lines from w_cline_row.
  wp->w_wrow = wp->w_cline_row;

  int n;
  int width1 = wp->w_view_width - extra;  // text width for first screen line
  int width2 = 0;                          // text width for second and later screen line
  bool did_sub_skipcol = false;
  if (width1 <= 0) {
    // No room for text, put cursor in last char of window.
    // If not wrapping, the last non-empty line.
    wp->w_wcol = wp->w_view_width - 1;
    if (wp->w_p_wrap) {
      wp->w_wrow = wp->w_view_height - 1;
    } else {
      wp->w_wrow = wp->w_view_height - 1 - wp->w_empty_rows;
    }
  } else if (wp->w_p_wrap && wp->w_view_width != 0) {
    width2 = width1 + win_col_off2(wp);

    // skip columns that are not visible
    if (wp->w_cursor.lnum == wp->w_topline
        && wp->w_skipcol > 0
        && wp->w_wcol >= wp->w_skipcol) {
      // Deduct by multiples of width2.  This allows the long line wrapping
      // formula below to correctly calculate the w_wcol value when wrapping.
      if (wp->w_skipcol <= width1) {
        wp->w_wcol -= width2;
      } else {
        wp->w_wcol -= width2 * (((wp->w_skipcol - width1) / width2) + 1);
      }

      did_sub_skipcol = true;
    }

    // long line wrapping, adjust wp->w_wrow
    if (wp->w_wcol >= wp->w_view_width) {
      // this same formula is used in validate_cursor_col()
      n = (wp->w_wcol - wp->w_view_width) / width2 + 1;
      wp->w_wcol -= n * width2;
      wp->w_wrow += n;
    }
  } else if (may_scroll
             && !wp->w_cline_folded) {
    // No line wrapping: compute wp->w_leftcol if scrolling is on and line
    // is not folded.
    // If scrolling is off, wp->w_leftcol is assumed to be 0

    // If Cursor is left of the screen, scroll rightwards.
    // If Cursor is right of the screen, scroll leftwards
    // If we get closer to the edge than 'sidescrolloff', scroll a little
    // extra
    int siso = get_sidescrolloff_value(wp);
    int off_left = startcol - wp->w_leftcol - siso;
    int off_right = endcol - wp->w_leftcol - wp->w_view_width + siso + 1;
    if (off_left < 0 || off_right > 0) {
      int diff = (off_left < 0) ? -off_left : off_right;

      // When far off or not enough room on either side, put cursor in
      // middle of window.
      int new_leftcol;
      if (p_ss == 0 || diff >= width1 / 2 || off_right >= off_left) {
        new_leftcol = wp->w_wcol - extra - width1 / 2;
      } else {
        if (diff < p_ss) {
          assert(p_ss <= INT_MAX);
          diff = (int)p_ss;
        }
        if (off_left < 0) {
          new_leftcol = wp->w_leftcol - diff;
        } else {
          new_leftcol = wp->w_leftcol + diff;
        }
      }
      new_leftcol = MAX(new_leftcol, 0);
      if (new_leftcol != (int)wp->w_leftcol) {
        wp->w_leftcol = new_leftcol;
        win_check_anchored_floats(wp);
        // screen has to be redrawn with new wp->w_leftcol
        redraw_later(wp, UPD_NOT_VALID);
      }
    }
    wp->w_wcol -= wp->w_leftcol;
  } else if (wp->w_wcol > (int)wp->w_leftcol) {
    wp->w_wcol -= wp->w_leftcol;
  } else {
    wp->w_wcol = 0;
  }

  // Skip over filler lines.  At the top use w_topfill, there
  // may be some filler lines above the window.
  if (wp->w_cursor.lnum == wp->w_topline) {
    wp->w_wrow += wp->w_topfill;
  } else {
    wp->w_wrow += win_get_fill(wp, wp->w_cursor.lnum);
  }

  int plines = 0;
  int so = get_scrolloff_value(wp);
  colnr_T prev_skipcol = wp->w_skipcol;
  if ((wp->w_wrow >= wp->w_view_height
       || ((prev_skipcol > 0
            || wp->w_wrow + so >= wp->w_view_height)
           && (plines = plines_win_nofill(wp, wp->w_cursor.lnum, false)) - 1
           >= wp->w_view_height))
      && wp->w_view_height != 0
      && wp->w_cursor.lnum == wp->w_topline
      && width2 > 0
      && wp->w_view_width != 0) {
    // Cursor past end of screen.  Happens with a single line that does
    // not fit on screen.  Find a skipcol to show the text around the
    // cursor.  Avoid scrolling all the time. compute value of "extra":
    // 1: Less than "p_so" lines above
    // 2: Less than "p_so" lines below
    // 3: both of them
    extra = 0;
    if (wp->w_skipcol + so * width2 > wp->w_virtcol) {
      extra = 1;
    }
    // Compute last display line of the buffer line that we want at the
    // bottom of the window.
    if (plines == 0) {
      plines = plines_win(wp, wp->w_cursor.lnum, false);
    }
    plines--;
    if (plines > wp->w_wrow + so) {
      assert(so <= INT_MAX);
      n = wp->w_wrow + so;
    } else {
      n = plines;
    }
    if ((colnr_T)n >= wp->w_view_height + wp->w_skipcol / width2 - so) {
      extra += 2;
    }

    if (extra == 3 || wp->w_view_height <= so * 2) {
      // not enough room for 'scrolloff', put cursor in the middle
      n = wp->w_virtcol / width2;
      if (n > wp->w_view_height / 2) {
        n -= wp->w_view_height / 2;
      } else {
        n = 0;
      }
      // don't skip more than necessary
      if (n > plines - wp->w_view_height + 1) {
        n = plines - wp->w_view_height + 1;
      }
      wp->w_skipcol = n > 0 ? width1 + (n - 1) * width2
                            : 0;
    } else if (extra == 1) {
      // less than 'scrolloff' lines above, decrease skipcol
      assert(so <= INT_MAX);
      extra = (wp->w_skipcol + so * width2 - wp->w_virtcol + width2 - 1) / width2;
      if (extra > 0) {
        if ((colnr_T)(extra * width2) > wp->w_skipcol) {
          extra = wp->w_skipcol / width2;
        }
        wp->w_skipcol -= extra * width2;
      }
    } else if (extra == 2) {
      // less than 'scrolloff' lines below, increase skipcol
      endcol = (n - wp->w_view_height + 1) * width2;
      while (endcol > wp->w_virtcol) {
        endcol -= width2;
      }
      wp->w_skipcol = MAX(wp->w_skipcol, endcol);
    }

    // adjust w_wrow for the changed w_skipcol
    if (did_sub_skipcol) {
      wp->w_wrow -= (wp->w_skipcol - prev_skipcol) / width2;
    } else {
      wp->w_wrow -= wp->w_skipcol / width2;
    }

    if (wp->w_wrow >= wp->w_view_height) {
      // small window, make sure cursor is in it
      extra = wp->w_wrow - wp->w_view_height + 1;
      wp->w_skipcol += extra * width2;
      wp->w_wrow -= extra;
    }

    // extra could be either positive or negative
    extra = (prev_skipcol - wp->w_skipcol) / width2;
    // TODO(bfredl): this is very suspicious when not called by win_update()
    // We should not randomly alter screen state outside of update_screen() :(
    if (wp->w_grid.target) {
      win_scroll_lines(wp, 0, extra);
    }
  } else if (!wp->w_p_sms) {
    wp->w_skipcol = 0;
  }
  if (prev_skipcol != wp->w_skipcol) {
    redraw_later(wp, UPD_SOME_VALID);
  }

  redraw_for_cursorcolumn(wp);

  // now w_leftcol and w_skipcol are valid, avoid check_cursor_moved()
  // thinking otherwise
  wp->w_valid_leftcol = wp->w_leftcol;
  wp->w_valid_skipcol = wp->w_skipcol;

  wp->w_valid |= VALID_WCOL|VALID_WROW|VALID_VIRTCOL;
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
  colnr_T scol = 0;
  colnr_T ccol = 0;
  colnr_T ecol = 0;
  int row = 0;
  colnr_T coloff = 0;
  bool visible_row = false;
  bool is_folded = false;

  linenr_T lnum = pos->lnum;
  if (lnum >= wp->w_topline && lnum <= wp->w_botline) {
    is_folded = hasFolding(wp, lnum, &lnum, NULL);
    row = plines_m_win(wp, wp->w_topline, lnum - 1, INT_MAX);
    // "row" should be the screen line where line "lnum" begins, which can
    // be negative if "lnum" is "w_topline" and "w_skipcol" is non-zero.
    row -= adjust_plines_for_skipcol(wp);
    // Add filler lines above this buffer line.
    row += lnum == wp->w_topline ? wp->w_topfill : win_get_fill(wp, lnum);
    visible_row = true;
  } else if (!local || lnum < wp->w_topline) {
    row = 0;
  } else {
    row = wp->w_view_height - 1;
  }

  bool existing_row = (lnum > 0 && lnum <= wp->w_buffer->b_ml.ml_line_count);

  if ((local || visible_row) && existing_row) {
    const colnr_T off = win_col_off(wp);
    if (is_folded) {
      row += (local ? 0 : wp->w_winrow + wp->w_winrow_off) + 1;
      coloff = (local ? 0 : wp->w_wincol + wp->w_wincol_off) + 1 + off;
    } else {
      assert(lnum == pos->lnum);
      getvcol(wp, pos, &scol, &ccol, &ecol);

      // similar to what is done in validate_cursor_col()
      colnr_T col = scol;
      col += off;
      int width = wp->w_view_width - off + win_col_off2(wp);

      // long line wrapping, adjust row
      if (wp->w_p_wrap && col >= (colnr_T)wp->w_view_width && width > 0) {
        // use same formula as what is used in curs_columns()
        int rowoff = visible_row ? ((col - wp->w_view_width) / width + 1) : 0;
        col -= rowoff * width;
        row += rowoff;
      }

      col -= wp->w_leftcol;

      if (col >= 0 && col < wp->w_view_width && row >= 0 && row < wp->w_view_height) {
        coloff = col - scol + (local ? 0 : wp->w_wincol + wp->w_wincol_off) + 1;
        row += (local ? 0 : wp->w_winrow + wp->w_winrow_off) + 1;
      } else {
        // character is left, right or below of the window
        scol = ccol = ecol = 0;
        if (local) {
          coloff = col < 0 ? -1 : wp->w_view_width + 1;
        } else {
          row = 0;
        }
      }
    }
  }
  *rowp = row;
  *scolp = scol + coloff;
  *ccolp = ccol + coloff;
  *ecolp = ecol + coloff;
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
  int offset = vcol2col(wp, lnum, vcol - 1, NULL);
  char *line = ml_get_buf(wp->w_buffer, lnum);
  char *p = line + offset;

  if (*p == NUL) {
    if (p == line) {  // empty line
      return 0;
    }
    // Move to the first byte of the last char.
    MB_PTR_BACK(line, p);
  }
  return (int)(p - line + 1);
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
  if (lp->fill < win_get_fill(wp, lp->lnum)) {
    // Add a filler line
    lp->fill++;
    lp->height = 1;
  } else {
    lp->lnum--;
    lp->fill = 0;
    if (lp->lnum < 1) {
      lp->height = MAXCOL;
    } else if (hasFolding(wp, lp->lnum, &lp->lnum, NULL)) {
      // Add a closed fold unless concealed.
      lp->height = !decor_conceal_line(wp, lp->lnum - 1, false);
    } else {
      lp->height = plines_win_nofill(wp, lp->lnum, winheight);
    }
  }
}

static void topline_back(win_T *wp, lineoff_T *lp)
{
  topline_back_winheight(wp, lp, true);
}

// Add one line below "lp->lnum".  This can be a filler line, a closed fold or
// a (wrapped) text line.  Uses and sets "lp->fill".
// Returns the height of the added line in "lp->height".
// Lines below the last one are incredibly high.
static void botline_forw(win_T *wp, lineoff_T *lp)
{
  if (lp->fill < win_get_fill(wp, lp->lnum + 1)) {
    // Add a filler line.
    lp->fill++;
    lp->height = 1;
  } else {
    lp->lnum++;
    lp->fill = 0;
    assert(wp->w_buffer != 0);
    if (lp->lnum > wp->w_buffer->b_ml.ml_line_count) {
      lp->height = MAXCOL;
    } else if (hasFolding(wp, lp->lnum, NULL, &lp->lnum)) {
      // Add a closed fold unless concealed.
      lp->height = !decor_conceal_line(wp, lp->lnum - 1, false);
    } else {
      lp->height = plines_win_nofill(wp, lp->lnum, true);
    }
  }
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
  int nochange = true;
  int buflen = curbuf->b_ml.ml_line_count;
  colnr_T prev_col = curwin->w_cursor.col;
  colnr_T prev_curswant = curwin->w_curswant;
  linenr_T prev_lnum = curwin->w_cursor.lnum;
  oparg_T oa = { 0 };
  cmdarg_T ca = { 0 };
  ca.oap = &oa;

  if (half) {
    // Scroll [count], 'scroll' or current window height lines.
    if (count) {
      curwin->w_p_scr = MIN(curwin->w_view_height, count);
    }
    count = MIN(curwin->w_view_height, (int)curwin->w_p_scr);

    int curscount = count;
    // Adjust count so as to not reveal end of buffer lines.
    if (dir == FORWARD
        && (curwin->w_topline + curwin->w_view_height + count > buflen
            || win_lines_concealed(curwin))) {
      int n = plines_correct_topline(curwin, curwin->w_topline, NULL, false, NULL);
      if (n - count < curwin->w_view_height && curwin->w_topline < buflen) {
        n += plines_m_win(curwin, curwin->w_topline + 1, buflen, curwin->w_view_height + count);
      }
      if (n < curwin->w_view_height + count) {
        count = n - curwin->w_view_height;
      }
    }

    // (Try to) scroll the window unless already at the end of the buffer.
    if (count > 0) {
      nochange = scroll_with_sms(dir, count, &curscount);
      curwin->w_cursor.lnum = prev_lnum;
      curwin->w_cursor.col = prev_col;
      curwin->w_curswant = prev_curswant;
    }

    // Move the cursor the same amount of screen lines, skipping over
    // concealed lines as those were not included in "curscount".
    if (curwin->w_p_wrap) {
      nv_screengo(&oa, dir, curscount, true);
    } else if (dir == FORWARD) {
      cursor_down_inner(curwin, curscount, true);
    } else {
      cursor_up_inner(curwin, curscount, true);
    }
  } else {
    // Scroll [count] times 'window' or current window height lines.
    count *= ((ONE_WINDOW && p_window > 0 && p_window < Rows - 1)
              ? MAX(1, (int)p_window - 2) : get_scroll_overlap(dir));
    nochange = scroll_with_sms(dir, count, &count);

    if (!nochange) {
      // Place cursor at top or bottom of window.
      validate_botline(curwin);
      linenr_T lnum = (dir == FORWARD ? curwin->w_topline : curwin->w_botline - 1);
      // In silent Ex mode the value of w_botline - 1 may be 0,
      // but cursor lnum needs to be at least 1.
      curwin->w_cursor.lnum = MAX(lnum, 1);
    }
  }

  if (get_scrolloff_value(curwin) > 0) {
    cursor_correct(curwin);
  }
  // Move cursor to first line of closed fold.
  foldAdjustCursor(curwin);

  nochange = nochange
             && prev_col == curwin->w_cursor.col
             && prev_lnum == curwin->w_cursor.lnum;

  // Error if both the viewport and cursor did not change.
  if (nochange) {
    beep_flush();
  } else if (!curwin->w_p_sms) {
    beginline(BL_SOL | BL_FIX);
  } else if (p_sol) {
    nv_g_home_m_cmd(&ca);
  }

  return nochange;
}

void do_check_cursorbind(void)
{
  static win_T *prev_curwin = NULL;
  static pos_T prev_cursor = { 0, 0, 0 };

  if (curwin == prev_curwin && equalpos(curwin->w_cursor, prev_cursor)) {
    return;
  }
  prev_curwin = curwin;
  prev_cursor = curwin->w_cursor;

  linenr_T line = curwin->w_cursor.lnum;
  colnr_T col = curwin->w_cursor.col;
  colnr_T coladd = curwin->w_cursor.coladd;
  colnr_T curswant = curwin->w_curswant;
  bool set_curswant = curwin->w_set_curswant;
  win_T *old_curwin = curwin;
  buf_T *old_curbuf = curbuf;
  int old_VIsual_select = VIsual_select;
  int old_VIsual_active = VIsual_active;

  // loop through the cursorbound windows
  VIsual_select = VIsual_active = false;
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    curwin = wp;
    curbuf = curwin->w_buffer;
    // skip original window and windows with 'nocursorbind'
    if (curwin != old_curwin && curwin->w_p_crb) {
      if (curwin->w_p_diff) {
        curwin->w_cursor.lnum =
          diff_get_corresponding_line(old_curbuf, line);
      } else {
        curwin->w_cursor.lnum = line;
      }
      curwin->w_cursor.col = col;
      curwin->w_cursor.coladd = coladd;
      curwin->w_curswant = curswant;
      curwin->w_set_curswant = set_curswant;

      // Make sure the cursor is in a valid position.  Temporarily set
      // "restart_edit" to allow the cursor to be beyond the EOL.
      {
        int restart_edit_save = restart_edit;
        restart_edit = true;
        check_cursor(curwin);

        // Avoid a scroll here for the cursor position, 'scrollbind' is
        // more important.
        if (!curwin->w_p_scb) {
          validate_cursor(curwin);
        }

        restart_edit = restart_edit_save;
      }
      // Correct cursor for multi-byte character.
      mb_adjust_cursor();
      redraw_later(curwin, UPD_VALID);

      // Only scroll when 'scrollbind' hasn't done this.
      if (!curwin->w_p_scb) {
        update_topline(curwin);
      }
      curwin->w_redr_status = true;
    }
  }

  // reset current-window
  VIsual_select = old_VIsual_select;
  VIsual_active = old_VIsual_active;
  curwin = old_curwin;
  curbuf = old_curbuf;
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
