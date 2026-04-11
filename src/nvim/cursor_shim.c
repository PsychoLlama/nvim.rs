// cursor_shim.c: C accessor wrappers shared between the Rust cursor crate and
// other Rust crates (move, change, window, mouse, edit, memline, drawline).
//
// After cursor.c was deleted, these functions were extracted here because
// multiple Rust crates declare and call them via FFI.
//
// Functions that are cursor-crate-only have been inlined directly in
// src/nvim-rs/cursor/src/lib.rs using extern globals (curwin, curbuf, VIsual)
// and direct C function calls.

#include <stdbool.h>

#include "nvim/buffer_defs.h"
#include "nvim/cursor.h"
#include "nvim/globals.h"
#include "nvim/memline.h"
#include "nvim/plines.h"
#include "nvim/pos_defs.h"
#include "nvim/types_defs.h"

#include "cursor_shim.c.generated.h"

// Safety assertion: the Rust CharsizeArgBuf is 320 bytes and must be >=
// sizeof(CharsizeArg). If this fires, increase the buffer size in lib.rs.
_Static_assert(sizeof(CharsizeArg) <= 320,
               "CharsizeArg size exceeds Rust CharsizeArgBuf (320 bytes) - "
               "update cursor crate");

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

/// Get window cursor pointer (wp->w_cursor).
pos_T *nvim_win_get_cursor_ptr(win_T *wp) { return &wp->w_cursor; }

/// Get length from cursor position to end of line (for Rust).
colnr_T nvim_cursor_get_pos_len(void)
{
  return ml_get_buf_len(curbuf, curwin->w_cursor.lnum) - curwin->w_cursor.col;
}

/// Set current window cursor column (for Rust).
void nvim_curwin_set_cursor_col(colnr_T col) { curwin->w_cursor.col = col; }

/// Set window cursor coladd (for Rust).
void nvim_win_set_cursor_coladd(win_T *wp, colnr_T coladd) { wp->w_cursor.coladd = coladd; }
