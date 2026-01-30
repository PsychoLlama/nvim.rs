#include <assert.h>
#include <inttypes.h>
#include <stdbool.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/assert_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/change.h"
#include "nvim/cursor.h"
#include "nvim/drawscreen.h"
#include "nvim/fold.h"
#include "nvim/globals.h"
#include "nvim/macros_defs.h"
#include "nvim/mark.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/move.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/plines.h"
#include "nvim/pos_defs.h"
#include "nvim/state.h"
#include "nvim/state_defs.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"

#include "cursor.c.generated.h"

// =============================================================================
// Rust Function Declarations
// =============================================================================

extern int rs_gchar_cursor(void);
extern int rs_getviscol(void);
extern int rs_getviscol2(colnr_T col, colnr_T coladd);
extern int rs_getvpos(win_T *wp, pos_T *pos, colnr_T wcol);
extern int rs_coladvance(win_T *wp, colnr_T wcol);
extern const char *rs_get_cursor_line_ptr(void);
extern const char *rs_get_cursor_pos_ptr(void);
extern colnr_T rs_get_cursor_line_len(void);
extern colnr_T rs_get_cursor_pos_len(void);
extern int rs_char_before_cursor(void);
extern void rs_adjust_cursor_col(void);
extern void rs_check_pos(buf_T *buf, pos_T *pos);
extern void rs_check_cursor_lnum(win_T *win);
extern void rs_check_cursor_col(win_T *win);
extern void rs_check_cursor(win_T *wp);
extern void rs_check_visual_pos(void);
extern int rs_inc_cursor(void);
extern int rs_dec_cursor(void);
extern void rs_pchar_cursor(char c);
extern linenr_T rs_get_cursor_rel_lnum(win_T *wp, linenr_T lnum);
extern bool rs_set_leftcol(colnr_T leftcol);

// =============================================================================
// Screen Column Functions
// =============================================================================

/// @return  the screen position of the cursor.
int getviscol(void)
{
  return rs_getviscol();
}

/// @return the screen position of character col with a coladd in the cursor line.
int getviscol2(colnr_T col, colnr_T coladd)
{
  return rs_getviscol2(col, coladd);
}

/// Go to column "wcol", and add/insert white space as necessary to get the
/// cursor in that column.
/// The caller must have saved the cursor line for undo!
int coladvance_force(colnr_T wcol)
{
  int rc = coladvance2(curwin, &curwin->w_cursor, true, false, wcol);

  if (wcol == MAXCOL) {
    curwin->w_valid &= ~VALID_VIRTCOL;
  } else {
    // Virtcol is valid
    set_valid_virtcol(curwin, wcol);
  }
  return rc;
}

/// Try to advance the Cursor to the specified screen column.
/// If virtual editing: fine tune the cursor position.
/// Note that all virtual positions off the end of a line should share
/// a curwin->w_cursor.col value (n.b. this is equal to strlen(line)),
/// beginning at coladd 0.
///
/// @return  OK if desired column is reached, FAIL if not
int coladvance(win_T *wp, colnr_T wcol)
{
  return rs_coladvance(wp, wcol);
}

/// @param addspaces  change the text to achieve our goal? only for wp=curwin!
/// @param finetune  change char offset for the exact column
/// @param wcol_arg  column to move to (can be negative)
static int coladvance2(win_T *wp, pos_T *pos, bool addspaces, bool finetune, colnr_T wcol_arg)
{
  assert(wp == curwin || !addspaces);
  colnr_T wcol = wcol_arg;
  int idx;
  colnr_T col = 0;
  int head = 0;

  int one_more = (State & MODE_INSERT)
                 || (State & MODE_TERMINAL)
                 || restart_edit != NUL
                 || (VIsual_active && *p_sel != 'o')
                 || ((get_ve_flags(wp) & kOptVeFlagOnemore) && wcol < MAXCOL);

  char *line = ml_get_buf(wp->w_buffer, pos->lnum);
  int linelen = ml_get_buf_len(wp->w_buffer, pos->lnum);

  if (wcol >= MAXCOL) {
    idx = linelen - 1 + one_more;
    col = wcol;

    if ((addspaces || finetune) && !VIsual_active) {
      wp->w_curswant = linetabsize(wp, pos->lnum) + one_more;
      if (wp->w_curswant > 0) {
        wp->w_curswant--;
      }
    }
  } else {
    int width = wp->w_view_width - win_col_off(wp);
    int csize = 0;

    if (finetune
        && wp->w_p_wrap
        && wp->w_view_width != 0
        && wcol >= (colnr_T)width
        && width > 0) {
      csize = linetabsize_eol(wp, pos->lnum);
      if (csize > 0) {
        csize--;
      }

      if (wcol / width > (colnr_T)csize / width
          && ((State & MODE_INSERT) == 0 || (int)wcol > csize + 1)) {
        // In case of line wrapping don't move the cursor beyond the
        // right screen edge.  In Insert mode allow going just beyond
        // the last character (like what happens when typing and
        // reaching the right window edge).
        wcol = (csize / width + 1) * width - 1;
      }
    }

    CharsizeArg csarg;
    CSType cstype = init_charsize_arg(&csarg, wp, pos->lnum, line);
    StrCharInfo ci = utf_ptr2StrCharInfo(line);
    col = 0;
    while (col <= wcol && *ci.ptr != NUL) {
      CharSize cs = win_charsize(cstype, col, ci.ptr, ci.chr.value, &csarg);
      csize = cs.width;
      head = cs.head;
      col += cs.width;
      ci = utfc_next(ci);
    }
    idx = (int)(ci.ptr - line);

    // Handle all the special cases.  The virtual_active() check
    // is needed to ensure that a virtual position off the end of
    // a line has the correct indexing.  The one_more comparison
    // replaces an explicit add of one_more later on.
    if (col > wcol || (!virtual_active(wp) && one_more == 0)) {
      idx -= 1;
      // Don't count the chars from 'showbreak'.
      csize -= head;
      col -= csize;
    }

    if (virtual_active(wp)
        && addspaces
        && wcol >= 0
        && ((col != wcol && col != wcol + 1) || csize > 1)) {
      // 'virtualedit' is set: The difference between wcol and col is filled with spaces.

      if (line[idx] == NUL) {
        // Append spaces
        int correct = wcol - col;
        size_t newline_size;
        STRICT_ADD(idx, correct, &newline_size, size_t);
        char *newline = xmallocz(newline_size);
        memcpy(newline, line, (size_t)idx);
        memset(newline + idx, ' ', (size_t)correct);

        ml_replace(pos->lnum, newline, false);
        inserted_bytes(pos->lnum, (colnr_T)idx, 0, correct);
        idx += correct;
        col = wcol;
      } else {
        // Break a tab
        int correct = wcol - col - csize + 1;             // negative!!
        char *newline;

        if (-correct > csize) {
          return FAIL;
        }

        size_t n;
        STRICT_ADD(linelen - 1, csize, &n, size_t);
        newline = xmallocz(n);
        // Copy first idx chars
        memcpy(newline, line, (size_t)idx);
        // Replace idx'th char with csize spaces
        memset(newline + idx, ' ', (size_t)csize);
        // Copy the rest of the line
        STRICT_SUB(linelen, idx, &n, size_t);
        STRICT_SUB(n, 1, &n, size_t);
        memcpy(newline + idx + csize, line + idx + 1, n);

        ml_replace(pos->lnum, newline, false);
        inserted_bytes(pos->lnum, idx, 1, csize);
        idx += (csize - 1 + correct);
        col += correct;
      }
    }
  }

  pos->col = MAX(idx, 0);
  pos->coladd = 0;

  if (finetune) {
    if (wcol == MAXCOL) {
      // The width of the last character is used to set coladd.
      if (!one_more) {
        colnr_T scol, ecol;

        getvcol(wp, pos, &scol, NULL, &ecol);
        pos->coladd = ecol - scol;
      }
    } else {
      int b = (int)wcol - (int)col;

      // The difference between wcol and col is used to set coladd.
      if (b > 0 && b < (MAXCOL - 2 * wp->w_view_width)) {
        pos->coladd = b;
      }

      col += b;
    }
  }

  // Prevent from moving onto a trail byte.
  mark_mb_adjustpos(wp->w_buffer, pos);

  if (wcol < 0 || col < wcol) {
    return FAIL;
  }
  return OK;
}

/// Return in "pos" the position of the cursor advanced to screen column "wcol".
///
/// @return  OK if desired column is reached, FAIL if not
int getvpos(win_T *wp, pos_T *pos, colnr_T wcol)
{
  return rs_getvpos(wp, pos, wcol);
}

/// Increment the cursor position.  See inc() for return values.
int inc_cursor(void)
{
  return rs_inc_cursor();
}

/// Decrement the line pointer 'p' crossing line boundaries as necessary.
///
/// @return  1 when crossing a line, -1 when at start of file, 0 otherwise.
int dec_cursor(void)
{
  return rs_dec_cursor();
}

/// Get the line number relative to the current cursor position, i.e. the
/// difference between line number and cursor position. Only look for lines that
/// can be visible, folded lines don't count.
///
/// @param lnum line number to get the result for
linenr_T get_cursor_rel_lnum(win_T *wp, linenr_T lnum)
{
  return rs_get_cursor_rel_lnum(wp, lnum);
}

/// Make sure "pos.lnum" and "pos.col" are valid in "buf".
/// This allows for the col to be on the NUL byte.
void check_pos(buf_T *buf, pos_T *pos)
{
  rs_check_pos(buf, pos);
}

/// Make sure curwin->w_cursor.lnum is valid.
void check_cursor_lnum(win_T *win)
{
  rs_check_cursor_lnum(win);
}

/// Make sure win->w_cursor.col is valid. Special handling of insert-mode.
/// @see mb_check_adjust_col
void check_cursor_col(win_T *win)
{
  rs_check_cursor_col(win);
}

/// Make sure curwin->w_cursor in on a valid character
void check_cursor(win_T *wp)
{
  rs_check_cursor(wp);
}

/// Check if VIsual position is valid, correct it if not.
/// Can be called when in Visual mode and a change has been made.
void check_visual_pos(void)
{
  rs_check_visual_pos();
}

/// Make sure curwin->w_cursor is not on the NUL at the end of the line.
/// Allow it when in Visual mode and 'selection' is not "old".
void adjust_cursor_col(void)
{
  rs_adjust_cursor_col();
}

/// Set "curwin->w_leftcol" to "leftcol".
/// Adjust the cursor position if needed.
///
/// @return  true if the cursor was moved.
bool set_leftcol(colnr_T leftcol)
{
  return rs_set_leftcol(leftcol);
}

int gchar_cursor(void)
{
  return rs_gchar_cursor();
}

/// Return the character immediately before the cursor.
int char_before_cursor(void)
{
  return rs_char_before_cursor();
}

/// Write a character at the current cursor position.
/// It is directly written into the block.
void pchar_cursor(char c)
{
  rs_pchar_cursor(c);
}

/// @return  pointer to cursor line.
char *get_cursor_line_ptr(void)
{
  return (char *)rs_get_cursor_line_ptr();
}

/// @return  pointer to cursor position.
char *get_cursor_pos_ptr(void)
{
  return (char *)rs_get_cursor_pos_ptr();
}

/// @return  length (excluding the NUL) of the cursor line.
colnr_T get_cursor_line_len(void)
{
  return rs_get_cursor_line_len();
}

/// @return  length (excluding the NUL) of the cursor position.
colnr_T get_cursor_pos_len(void)
{
  return rs_get_cursor_pos_len();
}

// =============================================================================
// Rust Accessor Functions
// =============================================================================

/// Wrapper for getvvcol() callable from Rust.
/// Returns the start column in `scol`, cursor column in `ccol`, end column in `ecol`.
/// Pass NULL for any column you don't need.
void nvim_getvvcol(win_T *wp, pos_T *pos, colnr_T *scol, colnr_T *ccol, colnr_T *ecol)
{
  getvvcol(wp, pos, scol, ccol, ecol);
}

/// Wrapper for getvcol() callable from Rust.
/// Returns the start column in `scol`, cursor column in `ccol`, end column in `ecol`.
/// Pass NULL for any column you don't need.
void nvim_getvcol(win_T *wp, pos_T *pos, colnr_T *scol, colnr_T *ccol, colnr_T *ecol)
{
  getvcol(wp, pos, scol, ccol, ecol);
}

/// Wrapper for set_valid_virtcol() callable from Rust.
void nvim_set_valid_virtcol(win_T *wp, colnr_T vcol)
{
  set_valid_virtcol(wp, vcol);
}

/// Wrapper for virtual_active() callable from Rust.
bool nvim_virtual_active_win(win_T *wp)
{
  return virtual_active(wp);
}

// nvim_gchar_cursor is already defined in normal.c

/// Get curwin pointer for Rust.
win_T *nvim_cursor_get_curwin(void)
{
  return curwin;
}

/// Get cursor position pointer for current window (curwin->w_cursor).
pos_T *nvim_cursor_get_curwin_cursor(void)
{
  return &curwin->w_cursor;
}

/// Core getvpos implementation (calls coladvance2 directly).
/// This is used by both getvpos() and rs_getvpos().
int nvim_getvpos(win_T *wp, pos_T *pos, colnr_T wcol)
{
  return coladvance2(wp, pos, false, virtual_active(wp), wcol);
}

/// Check if character at position is TAB.
bool nvim_char_at_pos_is_tab(win_T *wp, pos_T *pos)
{
  return *(ml_get_buf(wp->w_buffer, pos->lnum) + pos->col) == TAB;
}

/// Clear VALID_VIRTCOL flag for window.
void nvim_win_clear_valid_virtcol(win_T *wp)
{
  wp->w_valid &= ~VALID_VIRTCOL;
}

/// Get window cursor pointer (wp->w_cursor).
pos_T *nvim_win_get_cursor_ptr(win_T *wp)
{
  return &wp->w_cursor;
}

/// Get pointer to cursor line (for Rust).
const char *nvim_cursor_get_line_ptr(void)
{
  return ml_get_buf(curbuf, curwin->w_cursor.lnum);
}

/// Get mutable pointer to cursor line (for Rust).
char *nvim_cursor_get_line_ptr_mut(void)
{
  return ml_get_buf_mut(curbuf, curwin->w_cursor.lnum);
}

/// Get pointer to cursor position in line (for Rust).
const char *nvim_cursor_get_pos_ptr(void)
{
  return ml_get_buf(curbuf, curwin->w_cursor.lnum) + curwin->w_cursor.col;
}

/// Get length of cursor line (for Rust).
colnr_T nvim_cursor_get_line_len(void)
{
  return ml_get_buf_len(curbuf, curwin->w_cursor.lnum);
}

/// Get length from cursor position to end of line (for Rust).
colnr_T nvim_cursor_get_pos_len(void)
{
  return ml_get_buf_len(curbuf, curwin->w_cursor.lnum) - curwin->w_cursor.col;
}

/// Set current window cursor column (for Rust).
void nvim_curwin_set_cursor_col(colnr_T col)
{
  curwin->w_cursor.col = col;
}

// =============================================================================
// Check Validation Accessor Functions (for Rust migration)
// =============================================================================

/// Get line count from buffer (for Rust).
int64_t nvim_buf_get_ml_line_count_i64(buf_T *buf)
{
  return buf->b_ml.ml_line_count;
}

/// Get line length from buffer (for Rust).
colnr_T nvim_buf_get_line_len_pos(buf_T *buf, linenr_T lnum)
{
  return ml_get_buf_len(buf, lnum);
}

/// Get VIsual position pointer (for Rust).
pos_T *nvim_get_visual_pos(void)
{
  return &VIsual;
}

/// Set VIsual position (for Rust).
void nvim_set_visual_pos(linenr_T lnum, colnr_T col, colnr_T coladd)
{
  VIsual.lnum = lnum;
  VIsual.col = col;
  VIsual.coladd = coladd;
}

/// Get curbuf pointer (for Rust).
buf_T *nvim_cursor_get_curbuf(void)
{
  return curbuf;
}

/// Get buffer from window (for check_cursor_lnum).
buf_T *nvim_win_get_buffer_ptr(win_T *wp)
{
  return wp->w_buffer;
}

/// Check if line is folded at end of buffer (for check_cursor_lnum).
/// Returns the first line of the fold if found, or 0 if not folded.
linenr_T nvim_check_folding_at_end(win_T *win)
{
  linenr_T first_lnum;
  buf_T *buf = win->w_buffer;
  if (hasFolding(win, buf->b_ml.ml_line_count, &first_lnum, NULL)) {
    return first_lnum;
  }
  return 0;
}

/// Set window cursor coladd (for Rust).
void nvim_win_set_cursor_coladd(win_T *wp, colnr_T coladd)
{
  wp->w_cursor.coladd = coladd;
}

/// Wrapper for mark_mb_adjustpos (for Rust).
void nvim_mark_mb_adjustpos(buf_T *buf, pos_T *lp)
{
  mark_mb_adjustpos(buf, lp);
}

/// Get getvcol start and end columns (for check_cursor_col virtualedit).
void nvim_get_vcol_range(win_T *wp, pos_T *pos, colnr_T *start, colnr_T *end)
{
  getvcol(wp, pos, start, NULL, end);
}
