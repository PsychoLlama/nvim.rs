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
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/move.h"
#include "nvim/option_vars.h"
#include "nvim/pos_defs.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
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
