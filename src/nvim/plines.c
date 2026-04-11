// plines.c: calculate the vertical and horizontal size of text in a window

#include <limits.h>
#include <stdbool.h>
#include <stdint.h>
#include <string.h>

#include "nvim/api/extmark.h"
#include "nvim/ascii_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/decoration.h"
#include "nvim/decoration_defs.h"
#include "nvim/diff.h"
#include "nvim/fold.h"
#include "nvim/globals.h"
#include "nvim/indent.h"
#include "nvim/macros_defs.h"
#include "nvim/mark_defs.h"
#include "nvim/marktree.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memline.h"
#include "nvim/move.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/plines.h"
#include "nvim/pos_defs.h"
#include "nvim/state.h"
#include "nvim/state_defs.h"
#include "nvim/types_defs.h"

#include "plines.c.generated.h"

// Rust implementations
extern const char *rs_get_showbreak_value(win_T *win);
extern CharSize rs_charsize_fast(win_T *wp, const char *cur, int use_tabstop, int vcol, int32_t cur_char);
extern int rs_linesize_fast(win_T *wp, int use_tabstop, const char *line, int vcol_arg, int len);
extern void rs_getvcol(void *csarg, const char *line, int end_col, int cstype,
                       int pos_lnum, int pos_coladd,
                       int *start_out, int *cursor_out, int *end_out, int *pos_col_out);
extern int rs_plines_win_nofold(void *csarg, int cstype, int first_char);
extern int rs_plines_win_col(void *csarg, const char *line, int column, int cstype, int fill_lines);
extern void rs_getvvcol(win_T *wp, pos_T pos, colnr_T *start, colnr_T *cursor, colnr_T *end);
extern void rs_getvcols(win_T *wp, pos_T pos1, pos_T pos2, colnr_T *left, colnr_T *right);

// Filter for inline virtual text marks
static const uint32_t inline_filter[kMTMetaCount] = {[kMTMetaInline] = kMTFilterSelect };

// CharsizeArg accessor functions for Rust

/// Get the window handle from CharsizeArg.
win_T *nvim_csarg_get_win(CharsizeArg *csarg) { return csarg->win; }

/// Get the line pointer from CharsizeArg.
char *nvim_csarg_get_line(CharsizeArg *csarg) { return csarg->line; }

/// Get the virt_row from CharsizeArg.
int nvim_csarg_get_virt_row(CharsizeArg *csarg) { return csarg->virt_row; }

/// Get use_tabstop from CharsizeArg.
int nvim_csarg_get_use_tabstop(CharsizeArg *csarg) { return csarg->use_tabstop ? 1 : 0; }

/// Get max_head_vcol from CharsizeArg.
int nvim_csarg_get_max_head_vcol(CharsizeArg *csarg) { return csarg->max_head_vcol; }

/// Get indent_width from CharsizeArg.
int nvim_csarg_get_indent_width(CharsizeArg *csarg) { return csarg->indent_width; }

/// Set indent_width in CharsizeArg.
void nvim_csarg_set_indent_width(CharsizeArg *csarg, int value) { csarg->indent_width = value; }

/// Get cur_text_width_left from CharsizeArg.
int nvim_csarg_get_cur_text_width_left(CharsizeArg *csarg) { return csarg->cur_text_width_left; }

/// Set cur_text_width_left in CharsizeArg.
void nvim_csarg_set_cur_text_width_left(CharsizeArg *csarg, int value) { csarg->cur_text_width_left = value; }

/// Get cur_text_width_right from CharsizeArg.
int nvim_csarg_get_cur_text_width_right(CharsizeArg *csarg) { return csarg->cur_text_width_right; }

/// Set cur_text_width_right in CharsizeArg.
void nvim_csarg_set_cur_text_width_right(CharsizeArg *csarg, int value) { csarg->cur_text_width_right = value; }

// Marktree iterator accessor functions for Rust

/// Get the current mark's row position.
int nvim_csarg_itr_current_row(CharsizeArg *csarg)
{
  MTKey mark = rs_marktree_itr_current(csarg->iter);
  return mark.pos.row;
}

/// Get the current mark's column position.
int nvim_csarg_itr_current_col(CharsizeArg *csarg)
{
  MTKey mark = rs_marktree_itr_current(csarg->iter);
  return mark.pos.col;
}

/// Check if the current mark is invalid.
int nvim_csarg_itr_mark_invalid(CharsizeArg *csarg)
{
  MTKey mark = rs_marktree_itr_current(csarg->iter);
  return mt_invalid(mark) ? 1 : 0;
}

/// Check if the current mark has right gravity.
int nvim_csarg_itr_mark_right(CharsizeArg *csarg)
{
  MTKey mark = rs_marktree_itr_current(csarg->iter);
  return mt_right(mark) ? 1 : 0;
}

/// Check if the current mark's namespace is visible in the window.
int nvim_csarg_itr_ns_in_win(CharsizeArg *csarg)
{
  MTKey mark = rs_marktree_itr_current(csarg->iter);
  return ns_in_win(mark.ns, csarg->win) ? 1 : 0;
}

/// Get the virtual text width from the current mark (summed for all inline virt texts).
/// Returns the left and right widths added to the output parameters.
void nvim_csarg_itr_get_virt_text_widths(CharsizeArg *csarg, int *left_width, int *right_width)
{
  MTKey mark = rs_marktree_itr_current(csarg->iter);
  *left_width = 0;
  *right_width = 0;

  if (mt_invalid(mark) || !ns_in_win(mark.ns, csarg->win)) {
    return;
  }

  DecorInline decor = mt_decor(mark);
  DecorVirtText *vt = decor.ext ? decor.data.ext.vt : NULL;
  while (vt) {
    if (!(vt->flags & kVTIsLines) && vt->pos == kVPosInline) {
      if (mt_right(mark)) {
        *right_width += vt->width;
      } else {
        *left_width += vt->width;
      }
    }
    vt = vt->next;
  }
}

/// Advance the iterator to the next inline virtual text mark.
void nvim_csarg_itr_next(CharsizeArg *csarg)
{
  rs_marktree_itr_next_filter(csarg->win->w_buffer->b_marktree, csarg->iter,
                           csarg->virt_row + 1, 0, inline_filter);
}

// Additional accessor functions for charsize_regular

/// Get the cursor offset for virtual text.
int nvim_virt_text_cursor_off(CharsizeArg *csarg, int on_NUL)
{
  int off = 0;
  if (!on_NUL || !(State & MODE_NORMAL)) {
    off += csarg->cur_text_width_left;
  }
  if (!on_NUL && (State & MODE_NORMAL)) {
    off += csarg->cur_text_width_right;
  }
  return off;
}

/// Get breakindent for a window/line.
int nvim_get_breakindent_win(win_T *wp, char *line) { return get_breakindent_win(wp, line); }

/// Check if character is in 'breakat'.
int nvim_vim_isbreak(int c) { return breakat_flags[(uint8_t)c] ? 1 : 0; }

/// Get the 'linebreak' option.
int nvim_win_get_p_lbr(win_T *wp) { return wp->w_p_lbr ? 1 : 0; }

// Note: nvim_win_get_p_bri is defined in window.c

/// Get the 'listchars' eol character.
int nvim_win_get_lcs_eol(win_T *wp) { return wp->w_p_lcs_chars.eol; }

/// Get filler lines for a window at a given line number (FFI wrapper).
int nvim_win_get_fill(win_T *wp, linenr_T lnum) { return win_get_fill(wp, lnum); }

/// Get screen lines for a buffer line without filler lines (FFI wrapper).
/// @param winheight When non-zero, limit to window height.
int nvim_plines_win_nofill(win_T *wp, linenr_T lnum, int winheight)
{
  return plines_win_nofill(wp, lnum, winheight != 0);
}

// Note: nvim_win_get_p_list is defined in window.c

// Visual mode and virtual editing accessors for getvcol

/// Check if virtual editing is active for a window.
int nvim_win_virtual_active(win_T *wp) { return virtual_active(wp) ? 1 : 0; }

/// Get the VIsual_active global.
int nvim_get_VIsual_active(void) { return VIsual_active ? 1 : 0; }

/// Get the VIsual position (lnum).
int nvim_get_VIsual_lnum(void) { return VIsual.lnum; }

/// Get the VIsual position (col).
int nvim_get_VIsual_col(void) { return VIsual.col; }

/// Get the VIsual position (coladd).
int nvim_get_VIsual_coladd(void) { return VIsual.coladd; }

// Note: nvim_get_p_sel_first is defined in cursor_shape.c

// Character iteration accessors for linesize_regular

/// Initialize StrCharInfo and return the first character value.
/// Returns the character value, and sets *ptr_out to the pointer,
/// and *len_out to the byte length.
int32_t nvim_str_char_info_init(const char *line, const char **ptr_out, int *len_out)
{
  StrCharInfo ci = utf_ptr2StrCharInfo((char *)line);
  *ptr_out = ci.ptr;
  *len_out = ci.chr.len;
  return ci.chr.value;
}

/// Advance to the next character and return its value.
/// Updates *ptr_out and *len_out.
int32_t nvim_str_char_info_next(const char **ptr_out, int len, int32_t value, int *len_out)
{
  StrCharInfo cur = { .ptr = (char *)*ptr_out, .chr = { .value = value, .len = len } };
  StrCharInfo next = utfc_next(cur);
  *ptr_out = next.ptr;
  *len_out = next.chr.len;
  return next.chr.value;
}

/// Functions calculating horizontal size of text, when displayed in a window.





/// Prepare the structure passed to charsize functions.
///
/// "line" is the start of the line.
/// When "lnum" is zero do not use inline virtual text.
CSType init_charsize_arg(CharsizeArg *csarg, win_T *wp, linenr_T lnum, char *line)
{
  csarg->win = wp;
  csarg->line = line;
  csarg->max_head_vcol = 0;
  csarg->cur_text_width_left = 0;
  csarg->cur_text_width_right = 0;
  csarg->virt_row = -1;
  csarg->indent_width = INT_MIN;
  csarg->use_tabstop = !wp->w_p_list || wp->w_p_lcs_chars.tab1;

  if (lnum > 0) {
    if (rs_marktree_itr_get_filter(wp->w_buffer->b_marktree, lnum - 1, 0, lnum, 0,
                                inline_filter, csarg->iter)) {
      csarg->virt_row = lnum - 1;
    }
  }

  if (csarg->virt_row >= 0
      || (wp->w_p_wrap && (wp->w_p_lbr || wp->w_p_bri || *rs_get_showbreak_value(wp) != NUL))) {
    return kCharsizeRegular;
  } else {
    return kCharsizeFast;
  }
}


/// Like charsize_regular(), except it doesn't handle inline virtual text,
/// 'linebreak', 'breakindent' or 'showbreak'.
/// Handles normal characters, tabs and wrapping.
/// Can be used if CSType is kCharsizeFast.
///
/// @see charsize_regular
CharSize charsize_fast(CharsizeArg *csarg, const char *cur, colnr_T vcol, int32_t cur_char)
  FUNC_ATTR_PURE
{
  return rs_charsize_fast(csarg->win, cur, csarg->use_tabstop, (int)vcol, cur_char);
}




/// Like linesize_regular(), but can be used when CSType is kCharsizeFast.
///
/// @see linesize_regular
int linesize_fast(CharsizeArg const *const csarg, int vcol_arg, colnr_T const len)
{
  return rs_linesize_fast(csarg->win, csarg->use_tabstop, csarg->line, vcol_arg, (int)len);
}

/// Get virtual column number of pos.
///  start: on the first position of this character (TAB, ctrl)
/// cursor: where the cursor is on this character (first char, except for TAB)
///    end: on the last position of this character (TAB, ctrl)
///
/// This is used very often, keep it fast!
///
/// @param wp
/// @param pos
/// @param start
/// @param cursor
/// @param end
void getvcol(win_T *wp, pos_T *pos, colnr_T *start, colnr_T *cursor, colnr_T *end)
{
  char *const line = ml_get_buf(wp->w_buffer, pos->lnum);  // start of the line
  colnr_T const end_col = pos->col;

  CharsizeArg csarg;
  CSType const cstype = init_charsize_arg(&csarg, wp, pos->lnum, line);
  csarg.max_head_vcol = -1;

  int start_out = 0;
  int cursor_out = 0;
  int end_out = 0;
  int pos_col_out = pos->col;

  rs_getvcol(&csarg, line, (int)end_col, (int)cstype,
             (int)pos->lnum, (int)pos->coladd,
             start ? &start_out : NULL,
             cursor ? &cursor_out : NULL,
             end ? &end_out : NULL,
             &pos_col_out);

  // Update pos->col if it was modified (NUL case)
  pos->col = (colnr_T)pos_col_out;

  if (start != NULL) {
    *start = (colnr_T)start_out;
  }
  if (cursor != NULL) {
    *cursor = (colnr_T)cursor_out;
  }
  if (end != NULL) {
    *end = (colnr_T)end_out;
  }
}


/// Get virtual column in virtual mode.
///
/// @param wp
/// @param pos
/// @param start
/// @param cursor
/// @param end
void getvvcol(win_T *wp, pos_T *pos, colnr_T *start, colnr_T *cursor, colnr_T *end)
{
  rs_getvvcol(wp, *pos, start, cursor, end);
}

/// Get the leftmost and rightmost virtual column of pos1 and pos2.
/// Used for Visual block mode.
///
/// @param wp
/// @param pos1
/// @param pos2
/// @param left
/// @param right
void getvcols(win_T *wp, pos_T *pos1, pos_T *pos2, colnr_T *left, colnr_T *right)
{
  rs_getvcols(wp, *pos1, *pos2, left, right);
}

/// Functions calculating vertical size of text when displayed inside a window.
/// Calls horizontal size functions defined above.


// plines_win, plines_win_nofill, win_get_fill: implemented in Rust (plines crate)

/// Get number of window lines physical line "lnum" will occupy in window "wp".
/// Does not care about folding, 'wrap' or filler lines.
int plines_win_nofold(win_T *wp, linenr_T lnum)
{
  char *s = ml_get_buf(wp->w_buffer, lnum);
  CharsizeArg csarg;
  CSType const cstype = init_charsize_arg(&csarg, wp, lnum, s);
  return rs_plines_win_nofold(&csarg, (int)cstype, (int)(uint8_t)*s);
}

/// Like plines_win(), but only reports the number of physical screen lines
/// used from the start of the line to the given column number.
int plines_win_col(win_T *wp, linenr_T lnum, long column)
{
  // Check for filler lines above this buffer line.
  // Keep this in C as it depends on decoration and diff systems.
  int fill_lines = win_get_fill(wp, lnum);

  char *line = ml_get_buf(wp->w_buffer, lnum);
  CharsizeArg csarg;
  CSType const cstype = init_charsize_arg(&csarg, wp, lnum, line);

  return rs_plines_win_col(&csarg, line, (int)column, (int)cstype, fill_lines);
}

// plines_win_full, plines_m_win, plines_m_win_fill: implemented in Rust (plines crate)


// C Wrappers for Rust FFI

/// Wrapper for linetabsize_eol() (accessor for Rust).
int nvim_linetabsize_eol(win_T *wp, linenr_T lnum) { return linetabsize_eol(wp, lnum); }

/// Wrapper for plines_win() (accessor for Rust).
int nvim_plines_win(win_T *wp, linenr_T lnum, int limit) { return plines_win(wp, lnum, limit != 0); }

/// Wrapper for plines_win_col() (accessor for Rust).
int nvim_plines_win_col(win_T *wp, linenr_T lnum, long column) { return plines_win_col(wp, lnum, column); }

/// Wrapper for win_may_fill() (accessor for Rust).
int nvim_win_may_fill(win_T *wp) { return win_may_fill(wp) ? 1 : 0; }

/// Wrapper for win_linetabsize() (accessor for Rust).
int nvim_win_linetabsize(win_T *wp, linenr_T lnum, char *line, colnr_T len)
{
  return win_linetabsize(wp, lnum, line, len);
}

/// Wrapper for getvcol() with PosT structure by value (accessor for Rust).
void nvim_getvcol_byval(win_T *wp, pos_T pos, colnr_T *start, colnr_T *cursor, colnr_T *end)
{
  getvcol(wp, &pos, start, cursor, end);
}

/// Set the p_list option for a window (accessor for Rust).
void nvim_win_set_p_list(win_T *wp, int val) { wp->w_p_list = val != 0; }

/// Wrapper for hasFolding without cache (accessor for Rust).
int nvim_hasFolding_nocache(win_T *wp, linenr_T lnum, linenr_T *firstp, linenr_T *lastp)
{
  return hasFoldingWin(wp, lnum, firstp, lastp, false, NULL) ? 1 : 0;
}

/// Wrapper for linetabsize_col calculation (accessor for Rust).
/// Uses init_charsize_arg with lnum=0 (no virtual text).
int nvim_linetabsize_col(win_T *wp, int startvcol, char *s)
{
  CharsizeArg csarg;
  CSType const cstype = init_charsize_arg(&csarg, wp, 0, s);
  if (cstype == kCharsizeFast) {
    return linesize_fast(&csarg, startvcol, MAXCOL);
  } else {
    return linesize_regular(&csarg, startvcol, MAXCOL);
  }
}
