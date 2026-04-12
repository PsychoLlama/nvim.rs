// drawline_shim.c: Thin C shim for drawline functions implemented in Rust.
//
// This file provides the C-visible symbols for win_line() and
// nvim_win_extmark_push(), both of which are delegated to Rust.

#include <stdbool.h>
#include <stdint.h>

#include "nvim/buffer_defs.h"
#include "nvim/drawline.h"
#include "nvim/fold_defs.h"
#include "nvim/pos_defs.h"
#include "nvim/types_defs.h"

#include "drawline_shim.c.generated.h"

/// Rust implementation of win_line().
extern int rs_win_line(win_T *wp, linenr_T lnum, int startrow, int endrow, int col_rows,
                       bool concealed, spellvars_T *spv, foldinfo_T foldinfo);

/// Display line "lnum" of window "wp" on the screen.
/// wp->w_virtcol needs to be valid.
///
/// @param lnum         line to display
/// @param startrow     first row relative to window grid
/// @param endrow       last grid row to be redrawn
/// @param col_rows     set to the height of the line when only updating the columns,
///                     otherwise set to 0
/// @param concealed    only draw virtual lines belonging to the line above
/// @param spv          'spell' related variables kept between calls for "wp"
/// @param foldinfo     fold info for this line
///
/// @return             the number of last row the line occupies.
int win_line(win_T *wp, linenr_T lnum, int startrow, int endrow, int col_rows, bool concealed,
             spellvars_T *spv, foldinfo_T foldinfo)
{
  return rs_win_line(wp, lnum, startrow, endrow, col_rows, concealed, spv, foldinfo);
}

/// Push a window extmark into win_extmark_arr.
/// Called from Rust decoration code.
void nvim_win_extmark_push(uint64_t ns_id, uint64_t mark_id, int win_row, int win_col)
{
  WinExtmark m = { (NS)ns_id, mark_id, win_row, win_col };
  kv_push(win_extmark_arr, m);
}
