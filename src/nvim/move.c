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

// Rust implementations (functions not yet given #[export_name])
extern void rs_compute_wcol(win_T *wp);
extern int rs_plines_correct_topline(win_T *wp, linenr_T lnum, linenr_T *nextp,
                                     int limit_winheight, int *foldedp);
extern void rs_textpos2screenpos(win_T *wp, pos_T *pos, int *rowp, int *scolp,
                                  int *ccolp, int *ecolp, int local);

// Accessor for global scrolljump option
OptInt nvim_get_p_sj(void) { return p_sj; }
void nvim_set_p_sj(OptInt val) { p_sj = val; }

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

// Validate w_wcol and w_virtcol only.
void validate_cursor_col(win_T *wp)
{
  validate_virtcol(wp);
  rs_compute_wcol(wp);
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

// C Wrappers for Rust FFI

/// Wrapper for cursor_correct() (accessor for Rust).
void nvim_cursor_correct(win_T *wp) { cursor_correct(wp); }

/// Wrapper for validate_cursor() with window parameter (accessor for Rust).
void nvim_validate_cursor_win(win_T *wp) { validate_cursor(wp); }

/// Wrapper for validate_virtcol() (accessor for Rust).
void nvim_validate_virtcol(win_T *wp) { validate_virtcol(wp); }

/// Wrapper for validate_cheight() (accessor for Rust).
void nvim_validate_cheight(win_T *wp) { validate_cheight(wp); }

/// Wrapper for check_topfill() (accessor for Rust).
void nvim_check_topfill(win_T *wp, int down) { check_topfill(wp, down); }

/// Wrapper for invalidate_botline() (accessor for Rust).
void nvim_invalidate_botline(win_T *wp) { invalidate_botline(wp); }

/// Wrapper for win_col_off() (accessor for Rust).
int nvim_win_col_off(win_T *wp) { return win_col_off(wp); }

/// Wrapper for win_col_off2() (accessor for Rust).
int nvim_win_col_off2(win_T *wp) { return win_col_off2(wp); }

/// Wrapper for cursor_correct_sms() (accessor for Rust).
void nvim_cursor_correct_sms(win_T *wp) { cursor_correct_sms(wp); }

/// Wrapper for validate_botline() (accessor for Rust).
void nvim_validate_botline(win_T *wp) { validate_botline(wp); }

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
linenr_T nvim_curbuf_line_count(void) { return curbuf->b_ml.ml_line_count; }

/// Wrapper for redraw_for_cursorcolumn() (accessor for Rust).
void nvim_redraw_for_cursorcolumn(win_T *wp) { redraw_for_cursorcolumn(wp); }

/// Wrapper for curs_columns() (accessor for Rust).
void nvim_curs_columns(win_T *wp, int may_scroll) { curs_columns(wp, may_scroll); }

/// Wrapper for update_topline() (accessor for Rust).
void nvim_update_topline(win_T *wp) { update_topline(wp); }

// C Wrapper Functions for pagescroll() Rust Migration

/// Wrapper for cursor_down_inner() (accessor for Rust).
void nvim_cursor_down_inner(win_T *wp, int n, int skip_conceal) { cursor_down_inner(wp, n, skip_conceal != 0); }

/// Wrapper for cursor_up_inner() (accessor for Rust).
void nvim_cursor_up_inner(win_T *wp, linenr_T n, int skip_conceal) { cursor_up_inner(wp, n, skip_conceal != 0); }

/// Wrapper for rs_nv_screengo() (accessor for Rust).
/// Returns 1 on success, 0 on failure.
extern bool rs_nv_screengo(oparg_T *oap, int dir, int dist, bool skip_conceal);
int nvim_nv_screengo(int dir, int dist, int skip_conceal)
{
  oparg_T oa = { 0 };
  return rs_nv_screengo(&oa, dir, dist, skip_conceal != 0) ? 1 : 0;
}

/// Wrapper for beginline() (accessor for Rust).
void nvim_beginline_flags(int flags) { beginline(flags); }

/// Wrapper for beep_flush() (accessor for Rust).
void nvim_beep_flush_wrapper(void) { beep_flush(); }

/// Wrapper for rs_nv_g_home_m_cmd() (accessor for Rust move crate).
extern void rs_nv_g_home_m_cmd(cmdarg_T *cap);
void nvim_nv_g_home_m_cmd(void)
{
  oparg_T oa = { 0 };
  cmdarg_T ca = { 0 };
  ca.oap = &oa;
  rs_nv_g_home_m_cmd(&ca);
}

/// Check if ONE_WINDOW macro is true (accessor for Rust).
int nvim_one_window(void) { return ONE_WINDOW ? 1 : 0; }

/// Get p_sol option value (accessor for Rust).
int nvim_get_p_sol(void) { return p_sol ? 1 : 0; }


/// Get w_p_scr (scroll option) value (accessor for Rust).
OptInt nvim_win_get_p_scr(win_T *wp) { return wp ? wp->w_p_scr : 0; }

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
int nvim_plines_m_win(win_T *wp, linenr_T first, linenr_T last, int max) { return plines_m_win(wp, first, last, max); }

/// Get skip_update_topline flag (accessor for Rust).
int nvim_get_skip_update_topline(void) { return skip_update_topline ? 1 : 0; }

/// Set skip_update_topline flag (accessor for Rust).
void nvim_set_skip_update_topline(int val) { skip_update_topline = val != 0; }

/// Wrapper for changed_cline_bef_curs (accessor for Rust).
void nvim_changed_cline_bef_curs(win_T *wp) { changed_cline_bef_curs(wp); }
