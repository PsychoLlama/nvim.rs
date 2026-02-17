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
extern int rs_charsize_nowrap(buf_T *buf, const char *cur, int use_tabstop, int vcol, int32_t cur_char);
extern int rs_win_may_fill(win_T *wp);
extern int rs_in_win_border(win_T *wp, int vcol);
extern int rs_win_chartabsize(win_T *wp, const char *p, int col);
extern CharSize rs_charsize_fast(win_T *wp, const char *cur, int use_tabstop, int vcol, int32_t cur_char);
extern int rs_linesize_fast(win_T *wp, int use_tabstop, const char *line, int vcol_arg, int len);
extern CharSize rs_charsize_regular(void *csarg, const char *cur, int vcol, int32_t cur_char);
extern int rs_linesize_regular(void *csarg, int vcol_arg, int len);
extern void rs_getvcol(void *csarg, const char *line, int end_col, int cstype,
                       int pos_lnum, int pos_coladd,
                       int *start_out, int *cursor_out, int *end_out, int *pos_col_out);
extern int rs_plines_win_nofold(void *csarg, int cstype, int first_char);
extern int rs_plines_win_col(void *csarg, const char *line, int column, int cstype, int fill_lines);
extern int rs_linetabsize(win_T *wp, linenr_T lnum);
extern int rs_linetabsize_eol(win_T *wp, linenr_T lnum);
extern int rs_linetabsize_col(int startvcol, char *s);
extern colnr_T rs_getvcol_nolist(pos_T pos);
extern void rs_getvvcol(win_T *wp, pos_T pos, colnr_T *start, colnr_T *cursor, colnr_T *end);
extern void rs_getvcols(win_T *wp, pos_T pos1, pos_T pos2, colnr_T *left, colnr_T *right);
extern int64_t rs_win_text_height(win_T *wp, linenr_T start_lnum, int64_t start_vcol,
                                  linenr_T *end_lnum, int64_t *end_vcol, int64_t *fill,
                                  int64_t max);
extern int rs_diff_check_fill(win_T *wp, linenr_T lnum);
extern int rs_diffopt_filler(void);

// Filter for inline virtual text marks
static const uint32_t inline_filter[kMTMetaCount] = {[kMTMetaInline] = kMTFilterSelect };

// ============================================================================
// CharsizeArg accessor functions for Rust
// ============================================================================

/// Get the window handle from CharsizeArg.
win_T *nvim_csarg_get_win(CharsizeArg *csarg)
{
  return csarg->win;
}

/// Get the line pointer from CharsizeArg.
char *nvim_csarg_get_line(CharsizeArg *csarg)
{
  return csarg->line;
}

/// Get the virt_row from CharsizeArg.
int nvim_csarg_get_virt_row(CharsizeArg *csarg)
{
  return csarg->virt_row;
}

/// Get use_tabstop from CharsizeArg.
int nvim_csarg_get_use_tabstop(CharsizeArg *csarg)
{
  return csarg->use_tabstop ? 1 : 0;
}

/// Get max_head_vcol from CharsizeArg.
int nvim_csarg_get_max_head_vcol(CharsizeArg *csarg)
{
  return csarg->max_head_vcol;
}

/// Get indent_width from CharsizeArg.
int nvim_csarg_get_indent_width(CharsizeArg *csarg)
{
  return csarg->indent_width;
}

/// Set indent_width in CharsizeArg.
void nvim_csarg_set_indent_width(CharsizeArg *csarg, int value)
{
  csarg->indent_width = value;
}

/// Get cur_text_width_left from CharsizeArg.
int nvim_csarg_get_cur_text_width_left(CharsizeArg *csarg)
{
  return csarg->cur_text_width_left;
}

/// Set cur_text_width_left in CharsizeArg.
void nvim_csarg_set_cur_text_width_left(CharsizeArg *csarg, int value)
{
  csarg->cur_text_width_left = value;
}

/// Get cur_text_width_right from CharsizeArg.
int nvim_csarg_get_cur_text_width_right(CharsizeArg *csarg)
{
  return csarg->cur_text_width_right;
}

/// Set cur_text_width_right in CharsizeArg.
void nvim_csarg_set_cur_text_width_right(CharsizeArg *csarg, int value)
{
  csarg->cur_text_width_right = value;
}

// ============================================================================
// Marktree iterator accessor functions for Rust
// ============================================================================

/// Get the current mark's row position.
int nvim_csarg_itr_current_row(CharsizeArg *csarg)
{
  MTKey mark = marktree_itr_current(csarg->iter);
  return mark.pos.row;
}

/// Get the current mark's column position.
int nvim_csarg_itr_current_col(CharsizeArg *csarg)
{
  MTKey mark = marktree_itr_current(csarg->iter);
  return mark.pos.col;
}

/// Check if the current mark is invalid.
int nvim_csarg_itr_mark_invalid(CharsizeArg *csarg)
{
  MTKey mark = marktree_itr_current(csarg->iter);
  return mt_invalid(mark) ? 1 : 0;
}

/// Check if the current mark has right gravity.
int nvim_csarg_itr_mark_right(CharsizeArg *csarg)
{
  MTKey mark = marktree_itr_current(csarg->iter);
  return mt_right(mark) ? 1 : 0;
}

/// Check if the current mark's namespace is visible in the window.
int nvim_csarg_itr_ns_in_win(CharsizeArg *csarg)
{
  MTKey mark = marktree_itr_current(csarg->iter);
  return ns_in_win(mark.ns, csarg->win) ? 1 : 0;
}

/// Get the virtual text width from the current mark (summed for all inline virt texts).
/// Returns the left and right widths added to the output parameters.
void nvim_csarg_itr_get_virt_text_widths(CharsizeArg *csarg, int *left_width, int *right_width)
{
  MTKey mark = marktree_itr_current(csarg->iter);
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
  marktree_itr_next_filter(csarg->win->w_buffer->b_marktree, csarg->iter,
                           csarg->virt_row + 1, 0, inline_filter);
}

// ============================================================================
// Additional accessor functions for charsize_regular
// ============================================================================

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
int nvim_get_breakindent_win(win_T *wp, char *line)
{
  return get_breakindent_win(wp, line);
}

/// Check if character is in 'breakat'.
int nvim_vim_isbreak(int c)
{
  return breakat_flags[(uint8_t)c] ? 1 : 0;
}

/// Get the 'linebreak' option.
int nvim_win_get_p_lbr(win_T *wp)
{
  return wp->w_p_lbr ? 1 : 0;
}

// Note: nvim_win_get_p_bri is defined in window.c

/// Get the 'listchars' eol character.
int nvim_win_get_lcs_eol(win_T *wp)
{
  return wp->w_p_lcs_chars.eol;
}

/// Get the 'listchars' tab3 character (used for tabs).
int nvim_win_get_lcs_tab3(win_T *wp)
{
  return wp->w_p_lcs_chars.tab3;
}

/// Get filler lines for a window at a given line number (FFI wrapper).
int nvim_win_get_fill(win_T *wp, linenr_T lnum)
{
  return win_get_fill(wp, lnum);
}

/// Get screen lines for a buffer line without filler lines (FFI wrapper).
/// @param winheight When non-zero, limit to window height.
int nvim_plines_win_nofill(win_T *wp, linenr_T lnum, int winheight)
{
  return plines_win_nofill(wp, lnum, winheight != 0);
}

// Note: nvim_win_get_p_list is defined in window.c

// ============================================================================
// Visual mode and virtual editing accessors for getvcol
// ============================================================================

/// Check if virtual editing is active for a window.
int nvim_win_virtual_active(win_T *wp)
{
  return virtual_active(wp) ? 1 : 0;
}

/// Get the VIsual_active global.
int nvim_get_VIsual_active(void)
{
  return VIsual_active ? 1 : 0;
}

/// Get the VIsual position (lnum).
int nvim_get_VIsual_lnum(void)
{
  return VIsual.lnum;
}

/// Get the VIsual position (col).
int nvim_get_VIsual_col(void)
{
  return VIsual.col;
}

/// Get the VIsual position (coladd).
int nvim_get_VIsual_coladd(void)
{
  return VIsual.coladd;
}

// Note: nvim_get_p_sel_first is defined in cursor_shape.c

// ============================================================================
// Character iteration accessors for linesize_regular
// ============================================================================

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

/// Return the number of cells the first char in "p" will take on the screen,
/// taking into account the size of a tab.
/// Also see getvcol()
///
/// @param p
/// @param col
///
/// @return Number of cells.
///
/// @see charsize_nowrap()
int win_chartabsize(win_T *wp, char *p, colnr_T col)
{
  return rs_win_chartabsize(wp, p, (int)col);
}

/// Like linetabsize_str(), but "s" starts at virtual column "startvcol".
///
/// @param startvcol
/// @param s
///
/// @return Number of cells the string will take on the screen.
int linetabsize_col(int startvcol, char *s)
{
  return rs_linetabsize_col(startvcol, s);
}

/// Return the number of cells line "lnum" of window "wp" will take on the
/// screen, taking into account the size of a tab and inline virtual text.
/// Doesn't count the size of 'listchars' "eol".
int linetabsize(win_T *wp, linenr_T lnum)
{
  return rs_linetabsize(wp, lnum);
}

/// Like linetabsize(), but counts the size of 'listchars' "eol".
int linetabsize_eol(win_T *wp, linenr_T lnum)
{
  return rs_linetabsize_eol(wp, lnum);
}

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
    if (marktree_itr_get_filter(wp->w_buffer->b_marktree, lnum - 1, 0, lnum, 0,
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

/// Get the number of cells taken up on the screen for the given arguments.
/// "csarg->cur_text_width_left" and "csarg->cur_text_width_right" are set
/// to the extra size for inline virtual text.
///
/// When "csarg->max_head_vcol" is positive, only count in "head" the size
/// of 'showbreak'/'breakindent' before "csarg->max_head_vcol".
/// When "csarg->max_head_vcol" is negative, only count in "head" the size
/// of 'showbreak'/'breakindent' before where cursor should be placed.
CharSize charsize_regular(CharsizeArg *csarg, char *const cur, colnr_T const vcol,
                          int32_t const cur_char)
{
  return rs_charsize_regular(csarg, cur, (int)vcol, cur_char);
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

/// Get the number of cells taken up on the screen at given virtual column.
///
/// @see win_chartabsize()
int charsize_nowrap(buf_T *buf, const char *cur, bool use_tabstop, colnr_T vcol, int32_t cur_char)
{
  return rs_charsize_nowrap(buf, cur, use_tabstop, (int)vcol, cur_char);
}

/// Check that virtual column "vcol" is in the rightmost column of window "wp".
///
/// @param  wp    window
/// @param  vcol  column number
static bool in_win_border(win_T *wp, colnr_T vcol)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ARG(1)
{
  return rs_in_win_border(wp, (int)vcol);
}

/// Calculate virtual column until the given "len".
///
/// @param csarg    Argument to charsize functions.
/// @param vcol_arg Starting virtual column.
/// @param len      First byte of the end character, or MAXCOL.
///
/// @return virtual column before the character at "len",
///         or full size of the line if "len" is MAXCOL.
int linesize_regular(CharsizeArg *const csarg, int vcol_arg, colnr_T const len)
{
  return rs_linesize_regular(csarg, vcol_arg, (int)len);
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

/// Get virtual cursor column in the current window, pretending 'list' is off.
///
/// @param posp
///
/// @return The virtual cursor column.
colnr_T getvcol_nolist(pos_T *posp)
{
  return rs_getvcol_nolist(*posp);
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

/// Check if there may be filler lines anywhere in window "wp".
bool win_may_fill(win_T *wp)
{
  return rs_win_may_fill(wp);
}

/// Return the number of filler lines above "lnum".
///
/// @param wp
/// @param lnum
///
/// @return Number of filler lines above lnum
int win_get_fill(win_T *wp, linenr_T lnum)
{
  int virt_lines = decor_virt_lines(wp, lnum - 1, lnum, NULL, NULL, true);

  // be quick when there are no filler lines
  if (rs_diffopt_filler()) {
    int n = rs_diff_check_fill(wp, lnum);

    if (n > 0) {
      return virt_lines + n;
    }
  }
  return virt_lines;
}

/// Return the number of window lines occupied by buffer line "lnum".
/// Includes any filler lines.
///
/// @param limit_winheight  when true limit to window height
int plines_win(win_T *wp, linenr_T lnum, bool limit_winheight)
{
  // Check for filler lines above this buffer line.
  return plines_win_nofill(wp, lnum, limit_winheight) + win_get_fill(wp, lnum);
}

/// Return the number of window lines occupied by buffer line "lnum".
/// Does not include filler lines.
///
/// @param limit_winheight  when true limit to window height
int plines_win_nofill(win_T *wp, linenr_T lnum, bool limit_winheight)
{
  if (decor_conceal_line(wp, lnum - 1, false)) {
    return 0;
  }

  if (!wp->w_p_wrap) {
    return 1;
  }

  if (wp->w_view_width == 0) {
    return 1;
  }

  // Folded lines are handled just like an empty line.
  if (lineFolded(wp, lnum)) {
    return 1;
  }

  const int lines = plines_win_nofold(wp, lnum);
  if (limit_winheight && lines > wp->w_view_height) {
    return wp->w_view_height;
  }
  return lines;
}

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

/// Get the number of screen lines buffer line "lnum" will take in window "wp".
/// This takes care of both folds and topfill.
///
/// XXX: Because of topfill, this only makes sense when lnum >= wp->w_topline.
///
/// @param[in]  wp               window the line is in
/// @param[in]  lnum             line number
/// @param[out] nextp            if not NULL, the last line of a fold
/// @param[out] foldedp          if not NULL, whether lnum is on a fold
/// @param[in]  cache            whether to use the window's cache for folds
/// @param[in]  limit_winheight  when true limit to window height
///
/// @return the total number of screen lines
int plines_win_full(win_T *wp, linenr_T lnum, linenr_T *const nextp, bool *const foldedp,
                    const bool cache, const bool limit_winheight)
{
  bool folded = hasFoldingWin(wp, lnum, &lnum, nextp, cache, NULL);
  if (foldedp != NULL) {
    *foldedp = folded;
  }

  int filler_lines = lnum == wp->w_topline ? wp->w_topfill : win_get_fill(wp, lnum);

  if (decor_conceal_line(wp, lnum - 1, false)) {
    return filler_lines;
  }

  return (folded ? 1 : plines_win_nofill(wp, lnum, limit_winheight)) + filler_lines;
}

/// Return number of window lines a physical line range will occupy in window "wp".
/// Takes into account folding, 'wrap', topfill and filler lines beyond the end of the buffer.
///
/// XXX: Because of topfill, this only makes sense when first >= wp->w_topline.
///
/// @param first  first line number
/// @param last   last line number
/// @param max    number of lines to limit the height to
///
/// @see win_text_height
int plines_m_win(win_T *wp, linenr_T first, linenr_T last, int max)
{
  int count = 0;

  while (first <= last && count < max) {
    linenr_T next = first;
    count += plines_win_full(wp, first, &next, NULL, false, false);
    first = next + 1;
  }
  if (first == wp->w_buffer->b_ml.ml_line_count + 1) {
    count += win_get_fill(wp, first);
  }
  return MIN(max, count);
}

/// Return total number of physical and filler lines in a physical line range.
/// Doesn't treat a fold as a single line or consider a wrapped line multiple lines,
/// unlike plines_m_win() or win_text_height().
///
/// Mainly used for calculating scrolling offsets.
int plines_m_win_fill(win_T *wp, linenr_T first, linenr_T last)
{
  int count = last - first + 1 + decor_virt_lines(wp, first - 1, last, NULL, NULL, false);

  if (rs_diffopt_filler()) {
    for (int lnum = first; lnum <= last; lnum++) {
      // Note: this also considers folds (no filler lines inside folds).
      int n = rs_diff_check_fill(wp, lnum);
      count += MAX(n, 0);
    }
  }

  return MAX(count, 0);
}

/// Get the number of screen lines a range of text will take in window "wp".
///
/// @param[in] start_lnum    Starting line number, 1-based inclusive.
/// @param[in] start_vcol    >= 0: Starting virtual column index on "start_lnum",
///                                0-based inclusive, rounded down to full screen lines.
///                          < 0:  Count a full "start_lnum", including filler lines above.
/// @param[in,out] end_lnum  Ending line number, 1-based inclusive. Set to last line for
///                          which the height is calculated (smaller if "max" is reached).
/// @param[in,out] end_vcol  >= 0: Ending virtual column index on "end_lnum",
///                                0-based exclusive, rounded up to full screen lines.
///                          < 0:  Count a full "end_lnum", not including filler lines below.
///                          Set to the number of columns in "end_lnum" to reach "max".
/// @param[in] max           Don't calculate the height for lines beyond the line where "max"
///                          height is reached.
/// @param[out] fill         If not NULL, set to the number of filler lines in the range.
int64_t win_text_height(win_T *const wp, const linenr_T start_lnum, const int64_t start_vcol,
                        linenr_T *const end_lnum, int64_t *const end_vcol, int64_t *const fill,
                        int64_t const max)
{
  return rs_win_text_height(wp, start_lnum, start_vcol, end_lnum, end_vcol, fill, max);
}

// =============================================================================
// C Wrappers for Rust FFI
// =============================================================================

/// Wrapper for linetabsize_eol() (accessor for Rust).
int nvim_linetabsize_eol(win_T *wp, linenr_T lnum)
{
  return linetabsize_eol(wp, lnum);
}

/// Wrapper for plines_win() (accessor for Rust).
int nvim_plines_win(win_T *wp, linenr_T lnum, int limit)
{
  return plines_win(wp, lnum, limit != 0);
}

/// Wrapper for win_may_fill() (accessor for Rust).
int nvim_win_may_fill(win_T *wp)
{
  return win_may_fill(wp) ? 1 : 0;
}

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
void nvim_win_set_p_list(win_T *wp, int val)
{
  wp->w_p_list = val != 0;
}

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
