// drawline_ffi.c: C accessor functions used by the Rust drawline crate.
//
// These wrap functions and globals that are not yet accessible from Rust
// via the existing shim infrastructure.

#include <stdbool.h>
#include <stdint.h>

#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/cursor_shape.h"
#include "nvim/decoration.h"
#include "nvim/drawline.h"
#include "nvim/globals.h"
#include "nvim/highlight.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memline.h"
#include "nvim/option_vars.h"
#include "nvim/plines.h"
#include "nvim/pos_defs.h"
#include "nvim/spell.h"
#include "nvim/state.h"
#include "nvim/syntax.h"
#include "nvim/terminal.h"
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

// ============================================================================
// Phase 1 accessors: needed by rs_c_win_line_pre_loop
// ============================================================================

/// Return true if virtual editing is active for the given window.
bool nvim_win_virtual_active_drawline(win_T *wp) { return virtual_active(wp); }

/// Return wp->w_buffer as opaque pointer.
void *nvim_win_get_w_buffer_drawline(win_T *wp) { return wp->w_buffer; }

/// Return true if wp->w_buffer->terminal is non-NULL (drawline variant).
bool nvim_win_buf_has_terminal_drawline(win_T *wp) { return wp->w_buffer->terminal != NULL; }

/// Get buf_meta_total(wp->w_buffer, kMTMetaInline) > 0.
bool nvim_win_buf_meta_total_inline(win_T *wp)
{
  return buf_meta_total(wp->w_buffer, kMTMetaInline) > 0;
}

/// Get highlight_attr[hlf] for a given highlight group index.
int nvim_get_highlight_attr_hlf(int hlf) { return highlight_attr[hlf]; }

/// Wrap terminal_get_line_attributes for a window's buffer.
void nvim_win_terminal_get_line_attrs(win_T *wp, int lnum, int *term_attrs)
{
  terminal_get_line_attributes(wp->w_buffer->terminal, wp, lnum, term_attrs);
}

/// Get wp->w_p_lcs_chars.multispace[pos], or NUL if NULL or out of range.
uint32_t nvim_win_lcs_multispace_char_at(win_T *wp, int pos)
{
  const schar_T *ms = wp->w_p_lcs_chars.multispace;
  if (ms == NULL) {
    return 0;
  }
  return (uint32_t)ms[pos];
}

/// Get wp->w_p_lcs_chars.leadmultispace[pos], or NUL if NULL or out of range.
uint32_t nvim_win_lcs_leadmultispace_char_at(win_T *wp, int pos)
{
  const schar_T *lms = wp->w_p_lcs_chars.leadmultispace;
  if (lms == NULL) {
    return 0;
  }
  return (uint32_t)lms[pos];
}

/// Get length (number of schar_T elements before NUL) of lcs_chars.multispace.
int nvim_win_lcs_multispace_len(win_T *wp)
{
  const schar_T *ms = wp->w_p_lcs_chars.multispace;
  if (ms == NULL) {
    return 0;
  }
  int n = 0;
  while (ms[n] != 0) {
    n++;
  }
  return n;
}

/// Get length (number of schar_T elements before NUL) of lcs_chars.leadmultispace.
int nvim_win_lcs_leadmultispace_len(win_T *wp)
{
  const schar_T *lms = wp->w_p_lcs_chars.leadmultispace;
  if (lms == NULL) {
    return 0;
  }
  int n = 0;
  while (lms[n] != 0) {
    n++;
  }
  return n;
}

/// Return true if wp->w_p_stc != NUL.
bool nvim_win_get_w_p_stc_is_set(win_T *wp) { return *wp->w_p_stc != NUL; }

/// Result struct for advance_to_start_vcol: carries outputs from the
/// charsize iteration loop back to Rust.
typedef struct {
  int ptr_offset;       ///< byte offset of ptr into the line buffer
  int vcol;             ///< updated vcol after advancing
  bool in_multispace;   ///< whether current char is in multispace run
  int multispace_pos;   ///< current position in multispace sequence
  int skip_cells;       ///< number of cells to skip (start_vcol - vcol - head)
  int fromcol;          ///< updated wlv->fromcol
  bool need_showbreak;  ///< whether showbreak is needed
} AdvanceToStartVcolResult;

/// Advance ptr through the line until vcol >= start_vcol, tracking multispace
/// state. This wraps the charsize-iteration loop (original C lines 462-530)
/// which uses CharsizeArg/CSType/StrCharInfo/utfc_next that are C-only.
///
/// @param wp          window
/// @param lnum        line number
/// @param line        pointer to the buffer line (ml_get_buf result)
/// @param start_vcol  target vcol to advance to
/// @param wlv_vcol    current wlv->vcol
/// @param wlv_tocol   current wlv->tocol
/// @param wlv_fromcol current wlv->fromcol
/// @param has_fold    whether there is a fold active
/// @param p_list      whether 'list' is enabled
/// @param p_wrap      whether 'wrap' is enabled
/// @param leadcol     lead column computed from lcs_lead/leadmultispace
/// @param in_ms       current in_multispace value
/// @param ms_pos      current multispace_pos value
/// @return AdvanceToStartVcolResult
AdvanceToStartVcolResult nvim_c_advance_to_start_vcol(win_T *wp, linenr_T lnum, char *line,  // NOLINT(readability-non-const-parameter)
                                                      int start_vcol, int wlv_vcol,
                                                      int wlv_tocol, int wlv_fromcol,
                                                      bool has_fold, bool p_list, bool p_wrap,
                                                      int leadcol,
                                                      bool in_ms, int ms_pos)
{
  AdvanceToStartVcolResult out = {
    .ptr_offset = 0,
    .vcol = wlv_vcol,
    .in_multispace = in_ms,
    .multispace_pos = ms_pos,
    .skip_cells = 0,
    .fromcol = wlv_fromcol,
    .need_showbreak = false,
  };

  char *ptr = line;
  char *prev_ptr = ptr;
  CharSize cs = { 0 };

  CharsizeArg csarg;
  CSType cstype = init_charsize_arg(&csarg, wp, lnum, line);
  csarg.max_head_vcol = start_vcol;
  int vcol = wlv_vcol;
  StrCharInfo ci = utf_ptr2StrCharInfo(ptr);

  while (vcol < start_vcol) {
    cs = win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &csarg);
    vcol += cs.width;
    prev_ptr = ci.ptr;
    if (*prev_ptr == NUL) {
      break;
    }
    ci = utfc_next(ci);
    if (p_list) {
      out.in_multispace = *prev_ptr == ' ' && (*ci.ptr == ' '
                                               || (prev_ptr > line && prev_ptr[-1] == ' '));
      if (!out.in_multispace) {
        out.multispace_pos = 0;
      } else if (ci.ptr >= line + leadcol
                 && wp->w_p_lcs_chars.multispace != NULL) {
        out.multispace_pos++;
        if (wp->w_p_lcs_chars.multispace[out.multispace_pos] == NUL) {
          out.multispace_pos = 0;
        }
      } else if (ci.ptr < line + leadcol
                 && wp->w_p_lcs_chars.leadmultispace != NULL) {
        out.multispace_pos++;
        if (wp->w_p_lcs_chars.leadmultispace[out.multispace_pos] == NUL) {
          out.multispace_pos = 0;
        }
      }
    }
  }
  out.vcol = vcol;
  int charsize = cs.width;
  int head = cs.head;
  ptr = ci.ptr;

  // When 'cuc', 'colorcolumn', 'virtualedit', visual mode active, or
  // drawing a fold, the end of the line may be before the start of the
  // displayed part.
  if (vcol < start_vcol && (wp->w_p_cuc
                            || wp->w_p_cc_cols
                            || virtual_active(wp)
                            || (VIsual_active && wp->w_buffer == curwin->w_buffer)
                            || has_fold)) {
    out.vcol = start_vcol;
    vcol = start_vcol;
  }

  // Handle a character that's not completely on the screen.
  if (vcol > start_vcol) {
    out.vcol -= charsize;
    vcol = out.vcol;
    ptr = prev_ptr;
  }

  if (start_vcol > vcol) {
    out.skip_cells = start_vcol - vcol - head;
  }

  // Adjust inverted text relative to start of screen
  if (wlv_tocol <= vcol) {
    out.fromcol = 0;
  } else if (wlv_fromcol >= 0 && wlv_fromcol < vcol) {
    out.fromcol = vcol;
  }

  if (p_wrap) {
    out.need_showbreak = true;
  }

  out.ptr_offset = (int)(ptr - line);
  return out;
}

