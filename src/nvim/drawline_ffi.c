// drawline_ffi.c: C accessor functions used by the Rust drawline crate.
//
// These wrap functions and globals that are not yet accessible from Rust
// via the existing shim infrastructure.

#include <stdbool.h>
#include <stdint.h>

#include "nvim/buffer_defs.h"
#include "nvim/cursor_shape.h"
#include "nvim/drawline.h"
#include "nvim/globals.h"
#include "nvim/memline.h"
#include "nvim/option_vars.h"
#include "nvim/pos_defs.h"
#include "nvim/state.h"
#include "nvim/syntax.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"

#include "drawline_ffi.c.generated.h"

/// Wrapper for cursor_is_block_during_visual, callable from Rust.
/// Returns 1 if the cursor should appear as a block during visual mode, else 0.
int nvim_cursor_is_block_during_visual(int exclusive)
{
  return cursor_is_block_during_visual(exclusive != 0) ? 1 : 0;
}

/// Return the character at the given buffer position.
/// Wraps gchar_pos() with individual lnum/col/coladd arguments.
int nvim_gchar_pos_lnum_col(int32_t lnum, int32_t col, int32_t coladd)
{
  pos_T pos = { .lnum = (linenr_T)lnum, .col = (colnr_T)col, .coladd = (colnr_T)coladd };
  return gchar_pos(&pos);
}

/// Get spellvars_T->spv_has_spell.
bool nvim_spv_get_has_spell(spellvars_T *spv) { return spv->spv_has_spell; }

/// Get spellvars_T->spv_checked_lnum.
int nvim_spv_get_checked_lnum(spellvars_T *spv) { return (int)spv->spv_checked_lnum; }

/// Get spellvars_T->spv_checked_col.
int nvim_spv_get_checked_col(spellvars_T *spv) { return spv->spv_checked_col; }

/// Get spellvars_T->spv_capcol_lnum.
int nvim_spv_get_capcol_lnum(spellvars_T *spv) { return (int)spv->spv_capcol_lnum; }

/// Get spellvars_T->spv_cap_col.
int nvim_spv_get_cap_col(spellvars_T *spv) { return spv->spv_cap_col; }

/// Set spellvars_T->spv_cap_col.
void nvim_spv_set_cap_col(spellvars_T *spv, int val) { spv->spv_cap_col = val; }

/// Set spellvars_T->spv_checked_lnum.
void nvim_spv_set_checked_lnum(spellvars_T *spv, int val)
{
  spv->spv_checked_lnum = (linenr_T)val;
}

/// Set spellvars_T->spv_capcol_lnum.
void nvim_spv_set_capcol_lnum(spellvars_T *spv, int val)
{
  spv->spv_capcol_lnum = (linenr_T)val;
}

/// Get did_emsg global.
bool nvim_get_did_emsg(void) { return did_emsg; }

/// Set did_emsg to a bool value.
void nvim_set_did_emsg(bool val) { did_emsg = val; }

/// Set did_emsg to an int cast as bool (used to restore saved value).
void nvim_set_did_emsg_int(bool val) { did_emsg = val; }

/// Get wp->w_s->b_syn_error.
bool nvim_win_get_syn_error(win_T *wp) { return wp->w_s->b_syn_error; }

/// Get wp->w_s->b_syn_slow.
bool nvim_win_get_syn_slow(win_T *wp) { return wp->w_s->b_syn_slow; }

/// Set wp->w_s->b_syn_error.
void nvim_win_set_syn_error(win_T *wp, bool val) { wp->w_s->b_syn_error = val; }

/// Get wp->w_p_fdt (pointer to foldtext string, NUL-terminated).
const char *nvim_win_get_p_fdt(win_T *wp) { return wp->w_p_fdt; }

/// Get wp->w_p_cc_cols (colorcolumn columns array, or NULL).
int *nvim_win_get_p_cc_cols(win_T *wp) { return wp->w_p_cc_cols; }

