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

_Static_assert(sizeof(pos_T) == 12, "pos_T size changed - update Rust CursorPos in cursor crate");

// inc_cursor and dec_cursor are now exported directly from Rust
// (src/nvim-rs/cursor/src/lib.rs via #[unsafe(export_name = "..."])).

// Rust Accessor Functions

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

/// Wrapper for virtual_active() callable from Rust.
bool nvim_virtual_active_win(win_T *wp) { return virtual_active(wp); }

// nvim_gchar_cursor is already defined in normal.c

/// Get curwin pointer for Rust.
win_T *nvim_cursor_get_curwin(void) { return curwin; }

/// Get cursor position pointer for current window (curwin->w_cursor).
pos_T *nvim_cursor_get_curwin_cursor(void) { return &curwin->w_cursor; }

/// Check if character at position is TAB.
bool nvim_char_at_pos_is_tab(win_T *wp, pos_T *pos) { return *(ml_get_buf(wp->w_buffer, pos->lnum) + pos->col) == TAB; }

/// Clear VALID_VIRTCOL flag for window.
void nvim_win_clear_valid_virtcol(win_T *wp) { wp->w_valid &= ~VALID_VIRTCOL; }

/// Get window cursor pointer (wp->w_cursor).
pos_T *nvim_win_get_cursor_ptr(win_T *wp) { return &wp->w_cursor; }

/// Get pointer to cursor line (for Rust).
const char *nvim_cursor_get_line_ptr(void) { return ml_get_buf(curbuf, curwin->w_cursor.lnum); }

/// Get mutable pointer to cursor line (for Rust).
char *nvim_cursor_get_line_ptr_mut(void) { return ml_get_buf_mut(curbuf, curwin->w_cursor.lnum); }

/// Get pointer to cursor position in line (for Rust).
const char *nvim_cursor_get_pos_ptr(void) { return ml_get_buf(curbuf, curwin->w_cursor.lnum) + curwin->w_cursor.col; }

/// Get length of cursor line (for Rust).
colnr_T nvim_cursor_get_line_len(void) { return ml_get_buf_len(curbuf, curwin->w_cursor.lnum); }

/// Get length from cursor position to end of line (for Rust).
colnr_T nvim_cursor_get_pos_len(void) { return ml_get_buf_len(curbuf, curwin->w_cursor.lnum) - curwin->w_cursor.col; }

/// Set current window cursor column (for Rust).
void nvim_curwin_set_cursor_col(colnr_T col) { curwin->w_cursor.col = col; }

// Check Validation Accessor Functions (for Rust migration)

/// Get line count from buffer (for Rust).
int64_t nvim_buf_get_ml_line_count_i64(buf_T *buf) { return buf->b_ml.ml_line_count; }

/// Get line length from buffer (for Rust).
colnr_T nvim_buf_get_line_len_pos(buf_T *buf, linenr_T lnum) { return ml_get_buf_len(buf, lnum); }

/// Get VIsual position pointer (for Rust).
pos_T *nvim_get_visual_pos(void) { return &VIsual; }

/// Set VIsual position (for Rust).
void nvim_set_visual_pos(linenr_T lnum, colnr_T col, colnr_T coladd)
{
  VIsual.lnum = lnum;
  VIsual.col = col;
  VIsual.coladd = coladd;
}

/// Get curbuf pointer (for Rust).
buf_T *nvim_cursor_get_curbuf(void) { return curbuf; }

/// Get buffer from window (for check_cursor_lnum).
buf_T *nvim_win_get_buffer_ptr(win_T *wp) { return wp->w_buffer; }

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
void nvim_win_set_cursor_coladd(win_T *wp, colnr_T coladd) { wp->w_cursor.coladd = coladd; }

/// Get getvcol start and end columns (for check_cursor_col virtualedit).
void nvim_get_vcol_range(win_T *wp, pos_T *pos, colnr_T *start, colnr_T *end) { getvcol(wp, pos, start, NULL, end); }


