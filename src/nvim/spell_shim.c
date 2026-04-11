// spell_shim.c: C accessor wrappers for the Rust spell crate.
//
// These functions provide access to C internals (DecorState, syntax, window
// fields) that cannot be accessed directly from Rust FFI.

#include <stdbool.h>

#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/decoration.h"
#include "nvim/decoration_provider.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/globals.h"
#include "nvim/memline.h"
#include "nvim/message.h"
#include "nvim/option_vars.h"
#include "nvim/pos_defs.h"
#include "nvim/spell.h"
#include "nvim/spell_defs.h"
#include "nvim/syntax.h"
#include "nvim/syntax_bridge.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"

#include "spell_shim.c.generated.h"

// =============================================================================
// Decoration state helpers for spell navigation (rs_spell_move_to)
// =============================================================================

/// Return sizeof(DecorState) so Rust can allocate enough space.
size_t nvim_spell_decor_state_size(void)
{
  return sizeof(DecorState);
}

/// Reset decor_state for spell navigation. Saves the current decor_state into
/// the provided buffer and initializes a fresh decor_state for spell nav.
/// The saved state must be restored with nvim_spell_restore_decor_state().
///
/// @param wp  The window being navigated.
/// @param saved_out  Caller-allocated storage (sizeof(DecorState)) for saving.
void nvim_spell_nav_start(win_T *wp, void *saved_out)
{
  *(DecorState *)saved_out = decor_state;
  decor_state = (DecorState){ 0 };
  decor_redraw_reset(wp, &decor_state);
}

/// Free the current (temporary) decor_state and restore the saved one.
///
/// @param saved  Pointer to the previously saved DecorState.
void nvim_spell_restore_decor_state(void *saved)
{
  decor_state_free(&decor_state);
  decor_state = *(DecorState *)saved;
}

/// Check decoration spell state at a column.
///
/// Returns: 1 (kTrue), 0 (kFalse), or -1 (kNone).
int nvim_spell_nav_decor_col(win_T *wp, int lnum, int *decor_lnum, int col)
{
  if (*decor_lnum != lnum) {
    decor_providers_invoke_spell(wp, lnum - 1, col, lnum - 1, -1);
    decor_redraw_line(wp, lnum - 1, &decor_state);
    *decor_lnum = lnum;
  }
  decor_redraw_col(wp, col, 0, false, &decor_state);
  switch (decor_state.spell) {
  case kTrue:  return 1;
  case kFalse: return 0;
  default:     return -1;
  }
}

// =============================================================================
// Syntax helpers for spell navigation
// =============================================================================

/// Check if syntax is present in the window.
bool nvim_spell_syntax_present(win_T *wp)
{
  return syntax_present(wp);
}

/// Check if syntax allows spell checking at a position.
bool nvim_spell_can_syn_spell(win_T *wp, int lnum, int col)
{
  bool can_spell;
  syn_get_id(wp, lnum, col, false, &can_spell, false);
  return can_spell;
}

// =============================================================================
// Misc accessors for spell navigation
// =============================================================================

/// Get the number of whitespace columns at the start of a line.
int nvim_spell_getwhitecols(const char *p)
{
  return (int)getwhitecols(p);
}

/// Get the line count of a window's buffer.
int nvim_spell_win_ml_line_count(win_T *wp)
{
  return wp->w_buffer->b_ml.ml_line_count;
}

/// Get line content from a window's buffer.
char *nvim_spell_ml_get_buf_win(win_T *wp, int lnum)
{
  return ml_get_buf(wp->w_buffer, (linenr_T)lnum);
}

/// Get line length from a window's buffer.
int nvim_spell_ml_get_buf_len_win(win_T *wp, int lnum)
{
  return ml_get_buf_len(wp->w_buffer, (linenr_T)lnum);
}

/// Check if the 'noplainbuffer' spelloptions flag is set.
bool nvim_spell_win_noplainbuffer(win_T *wp)
{
  return (wp->w_s->b_p_spo_flags & kOptSpoFlagNoplainbuffer) != 0;
}

/// Give the "search hit TOP, continuing at BOTTOM" or vice-versa warning.
/// @param forward  true => forward search (show "TOP...BOTTOM"), false => backward.
void nvim_spell_give_wrap_warning(bool forward)
{
  give_warning(_(forward ? bot_top_msg : top_bot_msg), true);
}
