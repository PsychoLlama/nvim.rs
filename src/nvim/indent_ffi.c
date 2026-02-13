/// indent_ffi.c: C accessor wrappers for the Rust indent crate (nvim-indent).
///
/// These thin wrappers provide a stable C ABI for Rust code to call into
/// Neovim's C internals for indentation operations.

#include <assert.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/change.h"
#include "nvim/charset.h"
#include "nvim/cursor.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/extmark.h"
#include "nvim/extmark_defs.h"
#include "nvim/globals.h"
#include "nvim/indent.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/move.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/plines.h"
#include "nvim/pos_defs.h"
#include "nvim/regexp.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/textformat.h"
#include "nvim/types_defs.h"
#include "nvim/undo.h"
#include "nvim/vim_defs.h"

#include "indent_ffi.c.generated.h"

// =============================================================================
// Static assertions for constants used in Rust code
// =============================================================================

_Static_assert(SIN_CHANGED == 1, "SIN_CHANGED must be 1");
_Static_assert(SIN_INSERT == 2, "SIN_INSERT must be 2");
_Static_assert(SIN_UNDO == 4, "SIN_UNDO must be 4");
_Static_assert(SIN_NOMARK == 8, "SIN_NOMARK must be 8");

// =============================================================================
// Phase 1: set_indent() accessors
// =============================================================================

bool nvim_curbuf_get_p_et(void) { return curbuf->b_p_et; }

int nvim_u_savesub_curline(void) { return u_savesub(curwin->w_cursor.lnum); }

linenr_T nvim_get_saved_cursor_lnum(void) { return saved_cursor.lnum; }
colnr_T nvim_get_saved_cursor_col(void) { return saved_cursor.col; }
void nvim_set_saved_cursor_col(colnr_T val) { saved_cursor.col = val; }

// =============================================================================
// Phase 3: get_breakindent_win() accessors
// =============================================================================

// These accessors already exist in other files (window.c, move.c, message.c, option.c):
//   nvim_win_get_view_width, nvim_win_get_buffer, nvim_win_get_p_list,
//   nvim_win_get_lcs_tab1, nvim_win_get_briopt_sbr, nvim_win_get_briopt_list,
//   nvim_win_col_off, nvim_win_col_off2, nvim_vim_strsize

int nvim_win_get_briopt_shift(win_T *wp) { return wp->w_briopt_shift; }
int nvim_win_get_briopt_min(win_T *wp) { return wp->w_briopt_min; }
int nvim_win_get_briopt_vcol(win_T *wp) { return wp->w_briopt_vcol; }

int nvim_buf_get_b_fnum(buf_T *buf) { return buf->b_fnum; }
int64_t nvim_indent_buf_get_changedtick(buf_T *buf) { return buf_get_changedtick(buf); }
const char *nvim_get_flp_value(buf_T *buf) { return get_flp_value(buf); }

unsigned nvim_get_dy_flags_uhex(void) { return dy_flags & kOptDyFlagUhex; }

const char *nvim_indent_get_showbreak_value(win_T *wp) { return get_showbreak_value(wp); }

/// Higher-level regex helper for breakindent: match formatlistpat against line.
///
/// Returns 1 if match found, 0 if no match. On match with list < 0, computes
/// the width of the matched text and stores it in *out_width.
/// On match with list > 0, *out_width is not modified.
int nvim_breakindent_flp_match(win_T *wp, const char *pat, const char *line,
                               int briopt_list, int *out_width)
{
  regmatch_T regmatch = {
    .regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING + RE_AUTO + RE_STRICT),
  };
  if (regmatch.regprog == NULL) {
    return 0;
  }
  regmatch.rm_ic = false;
  int matched = 0;
  if (vim_regexec(&regmatch, line, 0)) {
    matched = 1;
    if (briopt_list < 0) {
      // Compute width of matched text
      char *ptr = *regmatch.startp;
      char *end_ptr = *regmatch.endp;
      int indent = 0;
      while (ptr < end_ptr) {
        indent += win_chartabsize(wp, ptr, indent);
        MB_PTR_ADV(ptr);
      }
      *out_width = indent;
    }
  }
  vim_regfree(regmatch.regprog);
  return matched;
}
