// textformat.c: text formatting functions

#include <stdbool.h>
#include <stdint.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/change.h"
#include "nvim/charset.h"
#include "nvim/cursor.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/eval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/getchar.h"
#include "nvim/globals.h"
#include "nvim/indent.h"
#include "nvim/indent_c.h"
#include "nvim/macros_defs.h"
#include "nvim/mark.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/ops.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/os/input.h"
#include "nvim/pos_defs.h"
#include "nvim/search.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/textformat.h"
#include "nvim/textobject.h"
#include "nvim/ui.h"
#include "nvim/undo.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

#include "textformat.c.generated.h"

// Rust implementations
extern int rs_win_fdccol_count(win_T *wp);

/// C accessor for curbuf->b_p_fo (formatoptions).
char *nvim_get_curbuf_b_p_fo(void)
{
  return curbuf->b_p_fo;
}

// =============================================================================
// C Accessor Functions for Rust
// =============================================================================

/// Get line content at lnum (accessor for Rust).
char *nvim_textfmt_ml_get(linenr_T lnum)
{
  return ml_get(lnum);
}

/// Get line length at lnum (accessor for Rust).
colnr_T nvim_textfmt_ml_get_len(linenr_T lnum)
{
  return ml_get_len(lnum);
}

/// Get comment leader length (accessor for Rust).
int nvim_textfmt_get_leader_len(char *line, char **flags, bool backward, bool include_space)
{
  return get_leader_len(line, flags, backward, include_space);
}

/// Check if line starts a paragraph/section (accessor for Rust).
bool nvim_textfmt_startPS(linenr_T lnum, int para, bool both)
{
  return startPS(lnum, para, both);
}

/// Get number indent for a line (accessor for Rust).
int nvim_textfmt_get_number_indent(linenr_T lnum)
{
  return get_number_indent(lnum);
}

/// Get curbuf->b_p_tw (textwidth option, accessor for Rust).
int nvim_textfmt_get_curbuf_b_p_tw(void)
{
  return (int)curbuf->b_p_tw;
}

/// Get curbuf->b_p_wm (wrapmargin option, accessor for Rust).
int nvim_textfmt_get_curbuf_b_p_wm(void)
{
  return (int)curbuf->b_p_wm;
}

/// Get curwin->w_view_width (accessor for Rust).
int nvim_textfmt_get_curwin_w_view_width(void)
{
  return curwin->w_view_width;
}

/// Get curbuf pointer (accessor for Rust).
void *nvim_textfmt_get_curbuf(void)
{
  return curbuf;
}

/// Get cmdwin_buf pointer (accessor for Rust).
void *nvim_textfmt_get_cmdwin_buf(void)
{
  return cmdwin_buf;
}

/// Get curwin pointer (accessor for Rust).
void *nvim_textfmt_get_curwin(void)
{
  return curwin;
}


/// Get curwin->w_scwidth (sign column width, accessor for Rust).
int nvim_textfmt_get_curwin_w_scwidth(void)
{
  return curwin->w_scwidth;
}

/// Get curwin->w_p_nu (number option, accessor for Rust).
bool nvim_textfmt_get_curwin_w_p_nu(void)
{
  return curwin->w_p_nu;
}

/// Get curwin->w_p_rnu (relativenumber option, accessor for Rust).
bool nvim_textfmt_get_curwin_w_p_rnu(void)
{
  return curwin->w_p_rnu;
}

// =============================================================================
// C Accessor Functions for Format Operators (Phase T3)
// =============================================================================

/// Get oap->cursor_start.lnum (accessor for Rust).
linenr_T nvim_textfmt_oap_get_cursor_start_lnum(oparg_T *oap)
{
  return oap->cursor_start.lnum;
}

/// Get oap->cursor_start.col (accessor for Rust).
colnr_T nvim_textfmt_oap_get_cursor_start_col(oparg_T *oap)
{
  return oap->cursor_start.col;
}

/// Get oap->start.lnum (accessor for Rust).
linenr_T nvim_textfmt_oap_get_start_lnum(oparg_T *oap)
{
  return oap->start.lnum;
}

/// Get oap->start.col (accessor for Rust).
colnr_T nvim_textfmt_oap_get_start_col(oparg_T *oap)
{
  return oap->start.col;
}

/// Get oap->end.lnum (accessor for Rust).
linenr_T nvim_textfmt_oap_get_end_lnum(oparg_T *oap)
{
  return oap->end.lnum;
}

/// Get oap->line_count (accessor for Rust).
linenr_T nvim_textfmt_oap_get_line_count(oparg_T *oap)
{
  return oap->line_count;
}

/// Get oap->is_VIsual (accessor for Rust).
bool nvim_textfmt_oap_get_is_VIsual(oparg_T *oap)
{
  return oap->is_VIsual;
}

/// Get oap->end_adjusted (accessor for Rust).
bool nvim_textfmt_oap_get_end_adjusted(oparg_T *oap)
{
  return oap->end_adjusted;
}

/// Set curwin->w_cursor (accessor for Rust).
void nvim_textfmt_set_curwin_cursor(linenr_T lnum, colnr_T col)
{
  curwin->w_cursor.lnum = lnum;
  curwin->w_cursor.col = col;
}

/// Get curwin->w_cursor.lnum (accessor for Rust).
linenr_T nvim_textfmt_get_curwin_cursor_lnum(void)
{
  return curwin->w_cursor.lnum;
}

/// Call u_save (accessor for Rust).
int nvim_textfmt_u_save(linenr_T top, linenr_T bot)
{
  return u_save(top, bot);
}

/// Call redraw_curbuf_later (accessor for Rust).
void nvim_textfmt_redraw_curbuf_later(int type)
{
  redraw_curbuf_later(type);
}

/// Get cmdmod lockmarks flag (accessor for Rust).
bool nvim_textfmt_get_cmdmod_lockmarks(void)
{
  return (cmdmod.cmod_flags & CMOD_LOCKMARKS) != 0;
}

/// Set curbuf->b_op_start (accessor for Rust).
void nvim_textfmt_set_curbuf_op_start(linenr_T lnum, colnr_T col)
{
  curbuf->b_op_start.lnum = lnum;
  curbuf->b_op_start.col = col;
}

/// Set curbuf->b_op_end (accessor for Rust).
void nvim_textfmt_set_curbuf_op_end(linenr_T lnum, colnr_T col)
{
  curbuf->b_op_end.lnum = lnum;
  curbuf->b_op_end.col = col;
}

/// Set saved_cursor (accessor for Rust).
void nvim_textfmt_set_saved_cursor(linenr_T lnum, colnr_T col)
{
  saved_cursor.lnum = lnum;
  saved_cursor.col = col;
}

/// Get saved_cursor.lnum (accessor for Rust).
linenr_T nvim_textfmt_get_saved_cursor_lnum(void)
{
  return saved_cursor.lnum;
}

/// Clear saved_cursor (accessor for Rust).
void nvim_textfmt_clear_saved_cursor(void)
{
  saved_cursor.lnum = 0;
}

/// Call beginline (accessor for Rust).
void nvim_textfmt_beginline(int flags)
{
  beginline(flags);
}

/// Call check_cursor (accessor for Rust).
void nvim_textfmt_check_cursor(void *win)
{
  check_cursor((win_T *)win);
}

/// Get curbuf->b_ml.ml_line_count (accessor for Rust).
linenr_T nvim_textfmt_get_ml_line_count(void)
{
  return curbuf->b_ml.ml_line_count;
}

/// Call msgmore (accessor for Rust).
void nvim_textfmt_msgmore(linenr_T n)
{
  msgmore(n);
}

/// Adjust visual windows after line count change (accessor for Rust).
void nvim_textfmt_adjust_visual_windows(linenr_T old_line_count)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (wp->w_old_cursor_lnum != 0) {
      if (wp->w_old_cursor_lnum > wp->w_old_visual_lnum) {
        wp->w_old_cursor_lnum += old_line_count;
      } else {
        wp->w_old_visual_lnum += old_line_count;
      }
    }
  }
}

// =============================================================================
// C Accessor Functions for fex_format (Phase fex)
// =============================================================================

/// Check if 'formatexpr' was set insecurely (accessor for Rust).
bool nvim_textfmt_fex_was_set_insecurely(void)
{
  return was_set_insecurely(curwin, kOptFormatexpr, OPT_LOCAL);
}

/// Set v:lnum (accessor for Rust).
void nvim_textfmt_set_vv_lnum(int64_t val)
{
  set_vim_var_nr(VV_LNUM, (varnumber_T)val);
}

/// Set v:count (accessor for Rust).
void nvim_textfmt_set_vv_count(int64_t val)
{
  set_vim_var_nr(VV_COUNT, (varnumber_T)val);
}

/// Set v:char (accessor for Rust).
void nvim_textfmt_set_vv_char(int c)
{
  set_vim_var_char(c);
}

/// Clear v:char (accessor for Rust).
void nvim_textfmt_clear_vv_char(void)
{
  set_vim_var_string(VV_CHAR, NULL, -1);
}

/// Get curbuf->b_p_fex (accessor for Rust).
char *nvim_textfmt_get_curbuf_b_p_fex(void)
{
  return curbuf->b_p_fex;
}

static sctx_T fex_saved_sctx;

/// Save current_sctx and set to curbuf's formatexpr script context (accessor for Rust).
void nvim_textfmt_fex_save_sctx(void)
{
  fex_saved_sctx = current_sctx;
  current_sctx = curbuf->b_p_script_ctx[kBufOptFormatexpr];
}

/// Restore current_sctx after fex evaluation (accessor for Rust).
void nvim_textfmt_fex_restore_sctx(void)
{
  current_sctx = fex_saved_sctx;
}

/// Increment sandbox counter (accessor for Rust).
void nvim_textfmt_sandbox_inc(void)
{
  sandbox++;
}

/// Decrement sandbox counter (accessor for Rust).
void nvim_textfmt_sandbox_dec(void)
{
  sandbox--;
}

/// Evaluate expression and return number (accessor for Rust).
int nvim_textfmt_eval_to_number(char *expr)
{
  return (int)eval_to_number(expr, true);
}

// =============================================================================
// C Accessor Functions for format_lines (Phase 2)
// =============================================================================

_Static_assert(INSCHAR_FORMAT == 1, "");
_Static_assert(INSCHAR_DO_COM == 2, "");
_Static_assert(INSCHAR_NO_FEX == 8, "");
_Static_assert(INSCHAR_COM_LIST == 16, "");
_Static_assert(MODE_NORMAL == 0x01, "");
_Static_assert(MODE_INSERT == 0x10, "");
_Static_assert(SIN_CHANGED == 1, "");

/// Delete bytes at cursor (accessor for Rust).
void nvim_textfmt_del_bytes(int count, bool fixpos, bool use_delcombine)
{
  del_bytes(count, fixpos, use_delcombine);
}

/// Call do_join (accessor for Rust).
int nvim_textfmt_do_join(int count, bool insert_space, bool save_undo, bool use_fo, bool setmark)
{
  return do_join((size_t)count, insert_space, save_undo, use_fo, setmark);
}

/// Check if line is empty (accessor for Rust).
bool nvim_textfmt_lineempty(linenr_T lnum)
{
  return *ml_get(lnum) == NUL;
}

/// Get p_smd (accessor for Rust).
int nvim_textfmt_get_p_smd(void)
{
  return p_smd;
}

/// Set p_smd (accessor for Rust).
void nvim_textfmt_set_p_smd(int val)
{
  p_smd = val;
}

/// Get get_c_indent (accessor for Rust).
int nvim_textfmt_get_c_indent(void)
{
  return get_c_indent();
}

/// Get get_expr_indent (accessor for Rust).
int nvim_textfmt_get_expr_indent(void)
{
  return get_expr_indent();
}

// =============================================================================
// C Accessor Functions for internal_format (Phase 3)
// =============================================================================

_Static_assert(OPENLINE_DELSPACES == 0x01, "");
_Static_assert(OPENLINE_DO_COM == 0x02, "");
_Static_assert(OPENLINE_KEEPTRAIL == 0x04, "");
_Static_assert(OPENLINE_MARKFIX == 0x08, "");
_Static_assert(OPENLINE_COM_LIST == 0x10, "");
_Static_assert(OPENLINE_FORMAT == 0x20, "");
_Static_assert(FORWARD == 1, "");
_Static_assert(VREPLACE_FLAG == 0x200, "");
_Static_assert(UPD_VALID == 10, "");
_Static_assert(INDENT_SET == 1, "");

/// Call pchar_cursor (accessor for Rust).
void nvim_textfmt_pchar_cursor(int c)
{
  pchar_cursor((char)c);
}

/// Call undisplay_dollar (accessor for Rust).
void nvim_textfmt_undisplay_dollar(void)
{
  undisplay_dollar();
}

/// Call backspace_until_column (accessor for Rust).
void nvim_textfmt_backspace_until_column(int col)
{
  backspace_until_column(col);
}

/// Call open_line (accessor for Rust).
bool nvim_textfmt_open_line(int dir, int flags, int indent, bool *did_do_comment)
{
  return open_line(dir, flags, indent, did_do_comment);
}

/// Set replace_offset (accessor for Rust).
void nvim_textfmt_set_replace_offset(int val)
{
  replace_offset = val;
}

/// Check utf_allow_break (accessor for Rust).
bool nvim_textfmt_utf_allow_break(int cc, int ncc)
{
  return utf_allow_break(cc, ncc);
}

/// Check utf_allow_break_before (accessor for Rust).
bool nvim_textfmt_utf_allow_break_before(int cc)
{
  return utf_allow_break_before(cc);
}

/// Get curwin->w_p_lbr (accessor for Rust).
int nvim_textfmt_get_curwin_w_p_lbr(void)
{
  return curwin->w_p_lbr;
}

/// Set curwin->w_p_lbr (accessor for Rust).
void nvim_textfmt_set_curwin_w_p_lbr(int val)
{
  curwin->w_p_lbr = val;
}

/// Get cursor position remaining length (accessor for Rust).
int nvim_textfmt_get_cursor_pos_len(void)
{
  return get_cursor_pos_len();
}

// =============================================================================
// C Accessor Functions for Auto-format (Phase T4)
// =============================================================================

/// Get curwin->w_cursor.col (accessor for Rust).
colnr_T nvim_textfmt_get_curwin_cursor_col(void)
{
  return curwin->w_cursor.col;
}

/// Call dec_cursor (accessor for Rust).
void nvim_textfmt_dec_cursor(void)
{
  dec_cursor();
}

/// Call inc_cursor (accessor for Rust).
void nvim_textfmt_inc_cursor(void)
{
  inc_cursor();
}

/// Call gchar_cursor (accessor for Rust).
int nvim_textfmt_gchar_cursor(void)
{
  return gchar_cursor();
}

/// Get pointer to cursor line content (accessor for Rust).
char *nvim_textfmt_get_cursor_line_ptr(void)
{
  return get_cursor_line_ptr();
}

/// Get length of cursor line (accessor for Rust).
colnr_T nvim_textfmt_get_cursor_line_len(void)
{
  return get_cursor_line_len();
}

/// Check WHITECHAR condition (accessor for Rust).
/// This handles the UTF-8 composing character check.
bool nvim_textfmt_whitechar(int cc)
{
  return ascii_iswhite(cc)
         && !utf_iscomposing_first(utf_ptr2char((char *)get_cursor_pos_ptr() + 1));
}

/// Get leader length without flags output (accessor for Rust).
int nvim_textfmt_get_leader_len_simple(char *line)
{
  return get_leader_len(line, NULL, false, true);
}

/// Call u_save_cursor (accessor for Rust).
int nvim_textfmt_u_save_cursor(void)
{
  return u_save_cursor();
}

/// Get saved_cursor.col (accessor for Rust).
colnr_T nvim_textfmt_get_saved_cursor_col(void)
{
  return saved_cursor.col;
}

/// Call coladvance (accessor for Rust).
void nvim_textfmt_coladvance(void *win, colnr_T col)
{
  coladvance((win_T *)win, col);
}

/// Call check_cursor_col (accessor for Rust).
void nvim_textfmt_check_cursor_col(void *win)
{
  check_cursor_col((win_T *)win);
}

/// Replace line with added trailing space (accessor for Rust).
void nvim_textfmt_ml_replace_with_space(linenr_T lnum)
{
  char *linep = get_cursor_line_ptr();
  colnr_T len = get_cursor_line_len();
  char *plinep = xstrnsave(linep, (size_t)len + 2);
  plinep[len] = ' ';
  plinep[len + 1] = NUL;
  ml_replace(lnum, plinep, false);
}

/// Call del_char (accessor for Rust).
int nvim_textfmt_del_char(bool fixpos)
{
  return del_char(fixpos);
}

static bool did_add_space = false;  ///< auto_format() added an extra space
                                    ///< under the cursor

/// Get did_add_space state (accessor for Rust).
bool nvim_textfmt_get_did_add_space(void)
{
  return did_add_space;
}

/// Set did_add_space state (accessor for Rust).
void nvim_textfmt_set_did_add_space(bool val)
{
  did_add_space = val;
}

#define WHITECHAR(cc) (ascii_iswhite(cc) \
                       && !utf_iscomposing_first(utf_ptr2char((char *)get_cursor_pos_ptr() + 1)))

