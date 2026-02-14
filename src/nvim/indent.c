#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/assert_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/change.h"
#include "nvim/charset.h"
#include "nvim/cursor.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/extmark.h"
#include "nvim/extmark_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/indent.h"
#include "nvim/indent_c.h"
#include "nvim/mark_defs.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/ops.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/os/input.h"
#include "nvim/plines.h"
#include "nvim/pos_defs.h"
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
#include "nvim/search.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/textformat.h"
#include "nvim/types_defs.h"
#include "nvim/undo.h"
#include "nvim/vim_defs.h"

#include "indent.c.generated.h"

extern int rs_tabstop_padding(int col, int64_t ts_arg, const int *vts);
extern int rs_indent_size_ts(const char *ptr, int64_t ts, const int *vts);
extern int rs_indent_size_no_ts(const char *ptr);
extern bool rs_set_indent(int size, int flags);
extern bool rs_copy_indent(int size, const char *src);
extern int rs_get_breakindent_win(win_T *wp, const char *line);
extern void rs_ins_try_si(int c);
extern void rs_op_reindent(oparg_T *oap, Indenter how);
extern int rs_tabstop_at(int col, int64_t ts, const int *vts, bool left);
extern int rs_tabstop_start(int col, int ts, const int *vts);
extern bool rs_tabstop_eq(const int *ts1, const int *ts2);
extern int rs_tabstop_count(const int *ts);
extern int rs_tabstop_first(const int *ts);

typedef struct {
  int ntabs;
  int nspcs;
} TabstopFromtoResult;
extern TabstopFromtoResult rs_tabstop_fromto(int start_col, int end_col, int ts, const int *vts);
extern int rs_get_sw_value_col(buf_T *buf, int col, bool left);
extern bool rs_may_do_si(void);

// Phase 138: Indentation helper functions
// Character classification
extern int rs_indent_is_space(char c);
extern int rs_indent_is_tab(char c);
extern int rs_indent_is_white(char c);
extern int rs_indent_is_eol(char c);
// Indent flags
extern int rs_indent_flag_set(void);
extern int rs_indent_flag_inc(void);
extern int rs_indent_flag_dec(void);
extern int rs_indent_is_set(int action);
extern int rs_indent_is_inc(int action);
extern int rs_indent_is_dec(int action);
// Shiftround modes
extern int rs_sr_round(void);
extern int rs_sr_left(void);
extern int rs_sr_right(void);
extern int rs_sr_is_round(int mode);
extern int rs_sr_is_left(int mode);
extern int rs_sr_is_right(int mode);
// Indent calculations
extern int rs_indent_round(int indent, int sw);
extern int rs_indent_floor(int indent, int sw);
extern int rs_indent_ceil(int indent, int sw);
extern int rs_indent_shift(int cur_indent, int sw, int count, int round);
extern int rs_indent_on_boundary(int indent, int sw);
// Whitespace counting
extern int rs_count_leading_white(const char *ptr);
extern int rs_count_leading_spaces(const char *ptr);
extern int rs_count_leading_tabs(const char *ptr);
extern int rs_line_is_blank(const char *ptr);
extern int rs_line_no_indent(const char *ptr);
// Column calculations
extern int rs_col_add_spaces(int col, int spaces);
extern int rs_col_add_tab(int col, int ts);
extern int rs_tabs_to_spaces(int ntabs, int ts);
extern int rs_spaces_to_tabs(int spaces, int ts);
extern int rs_spaces_after_tabs(int spaces, int ts);
// Expandtab helpers
extern int rs_use_expandtab(int expandtab);
extern int rs_indent_chars_needed(int indent, int ts, int expandtab);
// Default values
extern int rs_default_ts(void);
extern int rs_default_sw(void);
extern int rs_default_sts(void);
extern int rs_normalize_ts(int ts);
extern int rs_normalize_sw(int sw, int ts);

// Phase: Getters (indent/getters.rs)
extern int rs_get_sw_value(buf_T *buf);
extern int rs_get_sw_value_indent(buf_T *buf, bool left);
extern int rs_get_sts_value(void);
extern int rs_get_indent(void);
extern int rs_get_indent_lnum(linenr_T lnum);
extern int rs_get_indent_buf(buf_T *buf, linenr_T lnum);
extern bool rs_inindent(int extra);
extern int *rs_tabstop_copy(const int *oldts);

// Phase: Checks (indent/checks.rs)
extern bool rs_preprocs_left(void);
extern bool rs_use_indentexpr_for_lisp(void);
extern int rs_lisp_match(const char *p);

// Phase: Tabstop parsing (indent/lib.rs)
extern bool rs_tabstop_set(const char *var, colnr_T **array);

// Phase: Breakindent option parsing (indent/lib.rs)
extern bool rs_briopt_check(const char *briopt, win_T *wp);

// Phase: Error helpers (indent/lib.rs)
extern void rs_emsg_text_too_long(void);

/// Set the integer values corresponding to the string setting of 'vartabstop'.
/// "array" will be set, caller must free it if needed.
///
/// @return  false for an error.
bool tabstop_set(char *var, colnr_T **array)
{
  return rs_tabstop_set(var, array);
}

/// Calculate the number of screen spaces a tab will occupy.
/// If "vts" is set then the tab widths are taken from that array,
/// otherwise the value of ts is used.
int tabstop_padding(colnr_T col, OptInt ts_arg, const colnr_T *vts)
  FUNC_ATTR_PURE
{
  return rs_tabstop_padding(col, ts_arg, vts);
}

/// Find the size of the tab that covers a particular column.
///
/// If this is being called as part of a shift operation, col is not the cursor
/// column but is the column number to the left of the first non-whitespace
/// character in the line.  If the shift is to the left (left == true), then
/// return the size of the tab interval to the left of the column.
int tabstop_at(colnr_T col, OptInt ts, const colnr_T *vts, bool left)
{
  return rs_tabstop_at(col, ts, vts, left);
}

/// Find the column on which a tab starts.
colnr_T tabstop_start(colnr_T col, int ts, colnr_T *vts)
{
  return rs_tabstop_start(col, ts, vts);
}

/// Find the number of tabs and spaces necessary to get from one column
/// to another.
void tabstop_fromto(colnr_T start_col, colnr_T end_col, int ts_arg, const colnr_T *vts, int *ntabs,
                    int *nspcs)
{
  // Resolve ts_arg if it's 0 (meaning use buffer's tabstop)
  int ts = ts_arg == 0 ? (int)curbuf->b_p_ts : ts_arg;
  assert(ts != 0);  // suppress clang "Division by zero"

  TabstopFromtoResult result = rs_tabstop_fromto(start_col, end_col, ts, vts);
  *ntabs = result.ntabs;
  *nspcs = result.nspcs;
}

/// See if two tabstop arrays contain the same values.
static bool tabstop_eq(const colnr_T *ts1, const colnr_T *ts2)
{
  return rs_tabstop_eq(ts1, ts2);
}

/// Copy a tabstop array, allocating space for the new array.
int *tabstop_copy(const int *oldts)
{
  return rs_tabstop_copy(oldts);
}

/// Return a count of the number of tabstops.
int tabstop_count(colnr_T *ts)
{
  return rs_tabstop_count(ts);
}

/// Return the first tabstop, or 8 if there are no tabstops defined.
int tabstop_first(colnr_T *ts)
{
  return rs_tabstop_first(ts);
}

/// Return the effective shiftwidth value for current buffer, using the
/// 'tabstop' value when 'shiftwidth' is zero.
int get_sw_value(buf_T *buf)
{
  return rs_get_sw_value(buf);
}

/// Idem, using "pos".
static int get_sw_value_pos(buf_T *buf, pos_T *pos, bool left)
{
  pos_T save_cursor = curwin->w_cursor;

  curwin->w_cursor = *pos;
  int sw_value = get_sw_value_col(buf, get_nolist_virtcol(), left);
  curwin->w_cursor = save_cursor;
  return sw_value;
}

/// Idem, using the first non-black in the current line.
int get_sw_value_indent(buf_T *buf, bool left)
{
  return rs_get_sw_value_indent(buf, left);
}

/// Idem, using virtual column "col".
int get_sw_value_col(buf_T *buf, colnr_T col, bool left)
{
  return rs_get_sw_value_col(buf, col, left);
}

/// Return the effective softtabstop value for the current buffer,
/// using the shiftwidth  value when 'softtabstop' is negative.
int get_sts_value(void)
{
  return rs_get_sts_value();
}

/// Count the size (in window cells) of the indent in the current line.
int get_indent(void)
{
  return rs_get_indent();
}

/// Count the size (in window cells) of the indent in line "lnum".
int get_indent_lnum(linenr_T lnum)
{
  return rs_get_indent_lnum(lnum);
}

/// Count the size (in window cells) of the indent in line "lnum" of buffer "buf".
int get_indent_buf(buf_T *buf, linenr_T lnum)
{
  return rs_get_indent_buf(buf, lnum);
}

/// Compute the size of the indent (in window cells) in line "ptr",
/// without tabstops (count tab as ^I or <09>).
int indent_size_no_ts(char const *ptr)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_PURE
{
  return rs_indent_size_no_ts(ptr);
}

/// Compute the size of the indent (in window cells) in line "ptr",
/// using tabstops
int indent_size_ts(char const *ptr, OptInt ts, colnr_T *vts)
  FUNC_ATTR_NONNULL_ARG(1) FUNC_ATTR_PURE
{
  return rs_indent_size_ts(ptr, ts, vts);
}

/// Set the indent of the current line.
/// Leaves the cursor on the first non-blank in the line.
/// Caller must take care of undo.
/// "flags":
///  SIN_CHANGED:    call changed_bytes() if the line was changed.
///  SIN_INSERT: insert the indent in front of the line.
///  SIN_UNDO:   save line for undo before changing it.
///  SIN_NOMARK: don't move extmarks (because just after ml_append or something)
///  @param size measured in spaces
///
/// @return  true if the line was changed.
bool set_indent(int size, int flags)
{
  return rs_set_indent(size, flags);
}

// Return the indent of the current line after a number.  Return -1 if no
// number was found.  Used for 'n' in 'formatoptions': numbered list.
// Since a pattern is used it can actually handle more than numbers.
int get_number_indent(linenr_T lnum)
{
  colnr_T col;
  pos_T pos;
  regmatch_T regmatch;
  int lead_len = 0;  // Length of comment leader.

  if (lnum > curbuf->b_ml.ml_line_count) {
    return -1;
  }
  pos.lnum = 0;

  // In format_lines() (i.e. not insert mode), fo+=q is needed too...
  if ((State & MODE_INSERT) || has_format_option(FO_Q_COMS)) {
    lead_len = get_leader_len(ml_get(lnum), NULL, false, true);
  }
  regmatch.regprog = vim_regcomp(curbuf->b_p_flp, RE_MAGIC);

  if (regmatch.regprog != NULL) {
    regmatch.rm_ic = false;

    // vim_regexec() expects a pointer to a line.  This lets us
    // start matching for the flp beyond any comment leader...
    if (vim_regexec(&regmatch, ml_get(lnum) + lead_len, 0)) {
      pos.lnum = lnum;
      pos.col = (colnr_T)(*regmatch.endp - ml_get(lnum));
      pos.coladd = 0;
    }
    vim_regfree(regmatch.regprog);
  }

  if ((pos.lnum == 0) || (*ml_get_pos(&pos) == NUL)) {
    return -1;
  }
  getvcol(curwin, &pos, &col, NULL, NULL);
  return (int)col;
}

/// Check "briopt" as 'breakindentopt' and update the members of "wp".
/// This is called when 'breakindentopt' is changed and when a window is
/// initialized
///
/// @param briopt  when NULL: use "wp->w_p_briopt"
/// @param wp      when NULL: only check "briopt"
///
/// @return  FAIL for failure, OK otherwise.
bool briopt_check(char *briopt, win_T *wp)
{
  return rs_briopt_check(briopt, wp);
}

// Return appropriate space number for breakindent, taking influencing
// parameters into account. Window must be specified, since it is not
// necessarily always the current one.
int get_breakindent_win(win_T *wp, char *line)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_get_breakindent_win(wp, line);
}

/// Get breakindent for window and line number (accessor for Rust FFI).
/// Combines get_breakindent_win with ml_get_buf for convenience.
int nvim_get_breakindent_win_lnum(win_T *wp, linenr_T lnum)
{
  return get_breakindent_win(wp, ml_get_buf(wp->w_buffer, lnum));
}

// When extra == 0: Return true if the cursor is before or on the first
// non-blank in the line.
// When extra == 1: Return true if the cursor is before the first non-blank in
// the line.
bool inindent(int extra)
{
  return rs_inindent(extra);
}

/// Handle reindenting a block of lines.
void op_reindent(oparg_T *oap, Indenter how)
{
  rs_op_reindent(oap, how);
}

/// @return  true if lines starting with '#' should be left aligned.
bool preprocs_left(void)
{
  return rs_preprocs_left();
}

/// @return  true if the conditions are OK for smart indenting.
bool may_do_si(void)
{
  return rs_may_do_si();
}

// Try to do some very smart auto-indenting.
// Used when inserting a "normal" character.
void ins_try_si(int c)
{
  rs_ins_try_si(c);
}

/// Insert an indent (for <Tab> or CTRL-T) or delete an indent (for CTRL-D).
/// Keep the cursor on the same character.
/// type == INDENT_INC   increase indent (for CTRL-T or <Tab>)
/// type == INDENT_DEC   decrease indent (for CTRL-D)
/// type == INDENT_SET   set indent to "amount"
///
/// @param round               if true, round the indent to 'shiftwidth' (only with _INC and _Dec).
/// @param call_changed_bytes  call changed_bytes()
extern void rs_change_indent(int type, int amount, int round, bool call_changed_bytes);
void change_indent(int type, int amount, int round, bool call_changed_bytes)
{
  rs_change_indent(type, amount, round, call_changed_bytes);
}

/// Copy the indent from ptr to the current line (and fill to size).
/// Leaves the cursor on the first non-blank in the line.
///
/// @return true if the line was changed.
bool copy_indent(int size, char *src)
{
  return rs_copy_indent(size, src);
}

/// Give a "resulting text too long" error and maybe set got_int.
static void emsg_text_too_long(void)
{
  rs_emsg_text_too_long();
}

/// ":retab".
void ex_retab(exarg_T *eap)
{
  bool got_tab = false;
  int num_spaces = 0;
  int start_col = 0;                   // For start of white-space string
  int64_t start_vcol = 0;                  // For start of white-space string
  char *new_line = (char *)1;  // init to non-NULL
  colnr_T *new_vts_array = NULL;
  char *new_ts_str;  // string value of tab argument

  linenr_T first_line = 0;              // first changed line
  linenr_T last_line = 0;               // last changed line
  bool is_indent_only = false;

  int save_list = curwin->w_p_list;
  curwin->w_p_list = 0;             // don't want list mode here

  char *ptr = eap->arg;
  if (strncmp(ptr, "-indentonly", 11) == 0 && ascii_iswhite_or_nul(ptr[11])) {
    is_indent_only = true;
    ptr = skipwhite(ptr + 11);
  }

  new_ts_str = ptr;
  if (!tabstop_set(ptr, &new_vts_array)) {
    return;
  }
  while (ascii_isdigit(*ptr) || *ptr == ',') {
    ptr++;
  }

  // This ensures that either new_vts_array and new_ts_str are freshly
  // allocated, or new_vts_array points to an existing array and new_ts_str
  // is null.
  if (new_vts_array == NULL) {
    new_vts_array = curbuf->b_p_vts_array;
    new_ts_str = NULL;
  } else {
    new_ts_str = xmemdupz(new_ts_str, (size_t)(ptr - new_ts_str));
  }
  for (linenr_T lnum = eap->line1; !got_int && lnum <= eap->line2; lnum++) {
    ptr = ml_get(lnum);
    int old_len = ml_get_len(lnum);
    int col = 0;
    int64_t vcol = 0;
    bool did_undo = false;  // called u_save for current line
    while (true) {
      if (ascii_iswhite(ptr[col])) {
        if (!got_tab && num_spaces == 0) {
          // First consecutive white-space
          start_vcol = vcol;
          start_col = col;
        }
        if (ptr[col] == ' ') {
          num_spaces++;
        } else {
          got_tab = true;
        }
      } else {
        if (got_tab || (eap->forceit && num_spaces > 1)) {
          // Retabulate this string of white-space

          // len is virtual length of white string
          int len = num_spaces = (int)(vcol - start_vcol);
          int num_tabs = 0;
          if (!curbuf->b_p_et) {
            int t, s;

            tabstop_fromto((colnr_T)start_vcol, (colnr_T)vcol,
                           (int)curbuf->b_p_ts, new_vts_array, &t, &s);
            num_tabs = t;
            num_spaces = s;
          }
          if (curbuf->b_p_et || got_tab
              || (num_spaces + num_tabs < len)) {
            if (did_undo == false) {
              did_undo = true;
              if (u_save((linenr_T)(lnum - 1),
                         (linenr_T)(lnum + 1)) == FAIL) {
                new_line = NULL;  // flag out-of-memory
                break;
              }
            }

            // len is actual number of white characters used
            len = num_spaces + num_tabs;
            const int new_len = old_len - col + start_col + len + 1;
            if (new_len <= 0 || new_len >= MAXCOL) {
              emsg_text_too_long();
              break;
            }
            new_line = xmalloc((size_t)new_len);

            if (start_col > 0) {
              memmove(new_line, ptr, (size_t)start_col);
            }
            memmove(new_line + start_col + len,
                    ptr + col, (size_t)old_len - (size_t)col + 1);
            ptr = new_line + start_col;
            for (col = 0; col < len; col++) {
              ptr[col] = (col < num_tabs) ? '\t' : ' ';
            }
            if (ml_replace(lnum, new_line, false) == OK) {
              // "new_line" may have been copied
              new_line = curbuf->b_ml.ml_line_ptr;
              extmark_splice_cols(curbuf, lnum - 1, 0, (colnr_T)old_len,
                                  (colnr_T)new_len - 1, kExtmarkUndo);
            }
            if (first_line == 0) {
              first_line = lnum;
            }
            last_line = lnum;
            ptr = new_line;
            old_len = new_len - 1;
            col = start_col + len;
          }
        }
        got_tab = false;
        num_spaces = 0;

        if (is_indent_only) {
          break;
        }
      }
      if (ptr[col] == NUL) {
        break;
      }
      vcol += win_chartabsize(curwin, ptr + col, (colnr_T)vcol);
      if (vcol >= MAXCOL) {
        emsg_text_too_long();
        break;
      }
      col += utfc_ptr2len(ptr + col);
    }
    if (new_line == NULL) {                 // out of memory
      break;
    }
    line_breakcheck();
  }
  if (got_int) {
    emsg(_(e_interr));
  }

  // If a single value was given then it can be considered equal to
  // either the value of 'tabstop' or the value of 'vartabstop'.
  if (tabstop_count(curbuf->b_p_vts_array) == 0
      && tabstop_count(new_vts_array) == 1
      && curbuf->b_p_ts == tabstop_first(new_vts_array)) {
    // not changed
  } else if (tabstop_count(curbuf->b_p_vts_array) > 0
             && tabstop_eq(curbuf->b_p_vts_array, new_vts_array)) {
    // not changed
  } else {
    redraw_curbuf_later(UPD_NOT_VALID);
  }
  if (first_line != 0) {
    changed_lines(curbuf, first_line, 0, last_line + 1, 0, true);
  }

  curwin->w_p_list = save_list;         // restore 'list'

  if (new_ts_str != NULL) {  // set the new tabstop
    // If 'vartabstop' is in use or if the value given to retab has more
    // than one tabstop then update 'vartabstop'.
    colnr_T *old_vts_ary = curbuf->b_p_vts_array;

    if (tabstop_count(old_vts_ary) > 0 || tabstop_count(new_vts_array) > 1) {
      set_option_direct(kOptVartabstop, CSTR_AS_OPTVAL(new_ts_str), OPT_LOCAL, 0);
      curbuf->b_p_vts_array = new_vts_array;
      xfree(old_vts_ary);
    } else {
      // 'vartabstop' wasn't in use and a single value was given to
      // retab then update 'tabstop'.
      curbuf->b_p_ts = tabstop_first(new_vts_array);
      xfree(new_vts_array);
    }
    xfree(new_ts_str);
  }
  coladvance(curwin, curwin->w_curswant);

  u_clearline(curbuf);
}

/// Get indent level from 'indentexpr'.
int get_expr_indent(void)
{
  bool use_sandbox = was_set_insecurely(curwin, kOptIndentexpr, OPT_LOCAL);
  const sctx_T save_sctx = current_sctx;

  // Save and restore cursor position and curswant, in case it was changed
  // * via :normal commands.
  pos_T save_pos = curwin->w_cursor;
  colnr_T save_curswant = curwin->w_curswant;
  bool save_set_curswant = curwin->w_set_curswant;
  set_vim_var_nr(VV_LNUM, (varnumber_T)curwin->w_cursor.lnum);

  if (use_sandbox) {
    sandbox++;
  }
  textlock++;
  current_sctx = curbuf->b_p_script_ctx[kBufOptIndentexpr];

  // Need to make a copy, the 'indentexpr' option could be changed while
  // evaluating it.
  char *inde_copy = xstrdup(curbuf->b_p_inde);
  int indent = (int)eval_to_number(inde_copy, true);
  xfree(inde_copy);

  if (use_sandbox) {
    sandbox--;
  }
  textlock--;
  current_sctx = save_sctx;

  // Restore the cursor position so that 'indentexpr' doesn't need to.
  // Pretend to be in Insert mode, allow cursor past end of line for "o"
  // command.
  int save_State = State;
  State = MODE_INSERT;
  curwin->w_cursor = save_pos;
  curwin->w_curswant = save_curswant;
  curwin->w_set_curswant = save_set_curswant;
  check_cursor(curwin);
  State = save_State;

  // Reset did_throw, unless 'debug' has "throw" and inside a try/catch.
  if (did_throw && (vim_strchr(p_debug, 't') == NULL || trylevel == 0)) {
    handle_did_throw();
    did_throw = false;
  }

  // If there is an error, just keep the current indent.
  if (indent < 0) {
    indent = get_indent();
  }

  return indent;
}

// When 'p' is present in 'cpoptions, a Vi compatible method is used.
// The incompatible newer method is quite a bit better at indenting
// code in lisp-like languages than the traditional one; it's still
// mostly heuristics however -- Dirk van Deun, dirk@rave.org

// TODO(unknown):
// Findmatch() should be adapted for lisp, also to make showmatch
// work correctly: now (v5.3) it seems all C/C++ oriented:
// - it does not recognize the #\( and #\) notations as character literals
// - it doesn't know about comments starting with a semicolon
// - it incorrectly interprets '(' as a character literal
// All this messes up get_lisp_indent in some rare cases.
// Update from Sergey Khorev:
// I tried to fix the first two issues.
int get_lisp_indent(void)
{
  pos_T *pos;
  pos_T paren;
  int amount;

  pos_T realpos = curwin->w_cursor;
  curwin->w_cursor.col = 0;

  if ((pos = findmatch(NULL, '(')) == NULL) {
    pos = findmatch(NULL, '[');
  } else {
    paren = *pos;
    pos = findmatch(NULL, '[');

    if ((pos == NULL) || lt(*pos, paren)) {
      pos = &paren;
    }
  }

  if (pos != NULL) {
    // Extra trick: Take the indent of the first previous non-white
    // line that is at the same () level.
    amount = -1;
    int parencount = 0;

    while (--curwin->w_cursor.lnum >= pos->lnum) {
      if (linewhite(curwin->w_cursor.lnum)) {
        continue;
      }

      for (char *that = get_cursor_line_ptr(); *that != NUL; that++) {
        if (*that == ';') {
          while (*(that + 1) != NUL) {
            that++;
          }
          continue;
        }

        if (*that == '\\') {
          if (*(that + 1) != NUL) {
            that++;
          }
          continue;
        }

        if ((*that == '"') && (*(that + 1) != NUL)) {
          while (*++that && *that != '"') {
            // Skipping escaped characters in the string
            if (*that == '\\') {
              if (*++that == NUL) {
                break;
              }
              if (that[1] == NUL) {
                that++;
                break;
              }
            }
          }
          if (*that == NUL) {
            break;
          }
        }
        if ((*that == '(') || (*that == '[')) {
          parencount++;
        } else if ((*that == ')') || (*that == ']')) {
          parencount--;
        }
      }

      if (parencount == 0) {
        amount = get_indent();
        break;
      }
    }

    if (amount == -1) {
      curwin->w_cursor.lnum = pos->lnum;
      curwin->w_cursor.col = pos->col;
      colnr_T col = pos->col;

      char *line = get_cursor_line_ptr();

      CharsizeArg csarg;
      CSType cstype = init_charsize_arg(&csarg, curwin, pos->lnum, line);

      StrCharInfo sci = utf_ptr2StrCharInfo(line);
      amount = 0;
      while (*sci.ptr != NUL && col > 0) {
        amount += win_charsize(cstype, amount, sci.ptr, sci.chr.value, &csarg).width;
        sci = utfc_next(sci);
        col--;
      }
      char *that = sci.ptr;

      // Some keywords require "body" indenting rules (the
      // non-standard-lisp ones are Scheme special forms):
      // (let ((a 1))    instead    (let ((a 1))
      //   (...))       of       (...))
      if (((*that == '(') || (*that == '[')) && lisp_match(that + 1)) {
        amount += 2;
      } else {
        if (*that != NUL) {
          that++;
          amount++;
        }
        colnr_T firsttry = amount;

        while (ascii_iswhite(*that)) {
          amount += win_charsize(cstype, amount, that, (uint8_t)(*that), &csarg).width;
          that++;
        }

        if (*that && (*that != ';')) {
          // Not a comment line.
          // Test *that != '(' to accommodate first let/do
          // argument if it is more than one line.
          if ((*that != '(') && (*that != '[')) {
            firsttry++;
          }

          parencount = 0;

          CharInfo ci = utf_ptr2CharInfo(that);
          if (((ci.value != '"') && (ci.value != '\'') && (ci.value != '#')
               && ((ci.value < '0') || (ci.value > '9')))) {
            int quotecount = 0;
            while (*that && (!ascii_iswhite(ci.value) || quotecount || parencount)) {
              if (ci.value == '"') {
                quotecount = !quotecount;
              }
              if (((ci.value == '(') || (ci.value == '[')) && !quotecount) {
                parencount++;
              }
              if (((ci.value == ')') || (ci.value == ']')) && !quotecount) {
                parencount--;
              }
              if ((ci.value == '\\') && (*(that + 1) != NUL)) {
                amount += win_charsize(cstype, amount, that, ci.value, &csarg).width;
                StrCharInfo next_sci = utfc_next((StrCharInfo){ that, ci });
                that = next_sci.ptr;
                ci = next_sci.chr;
              }

              amount += win_charsize(cstype, amount, that, ci.value, &csarg).width;
              StrCharInfo next_sci = utfc_next((StrCharInfo){ that, ci });
              that = next_sci.ptr;
              ci = next_sci.chr;
            }
          }

          while (ascii_iswhite(*that)) {
            amount += win_charsize(cstype, amount, that, (uint8_t)(*that), &csarg).width;
            that++;
          }

          if (!*that || (*that == ';')) {
            amount = firsttry;
          }
        }
      }
    }
  } else {
    amount = 0;  // No matching '(' or '[' found, use zero indent.
  }
  curwin->w_cursor = realpos;

  return amount;
}

static int lisp_match(char *p)
{
  return rs_lisp_match(p);
}

/// Re-indent the current line, based on the current contents of it and the
/// surrounding lines. Fixing the cursor position seems really easy -- I'm very
/// confused what all the part that handles Control-T is doing that I'm not.
/// "get_the_indent" should be get_c_indent, get_expr_indent or get_lisp_indent.
void fixthisline(IndentGetter get_the_indent)
{
  int amount = get_the_indent();

  if (amount < 0) {
    return;
  }

  change_indent(INDENT_SET, amount, false, true);
  if (linewhite(curwin->w_cursor.lnum)) {
    did_ai = true;  // delete the indent if the line stays empty
  }
}

/// Return true if 'indentexpr' should be used for Lisp indenting.
/// Caller may want to check 'autoindent'.
bool use_indentexpr_for_lisp(void)
{
  return rs_use_indentexpr_for_lisp();
}

/// Fix indent for 'lisp' and 'cindent'.
void fix_indent(void)
{
  if (p_paste) {
    return;  // no auto-indenting when 'paste' is set
  }
  if (curbuf->b_p_lisp && curbuf->b_p_ai) {
    if (use_indentexpr_for_lisp()) {
      do_c_expr_indent();
    } else {
      fixthisline(get_lisp_indent);
    }
  } else if (cindent_on()) {
    do_c_expr_indent();
  }
}

/// "indent()" function
void f_indent(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  const linenr_T lnum = tv_get_lnum(argvars);
  if (lnum >= 1 && lnum <= curbuf->b_ml.ml_line_count) {
    rettv->vval.v_number = get_indent_lnum(lnum);
  } else {
    rettv->vval.v_number = -1;
  }
}

/// "lispindent(lnum)" function
void f_lispindent(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  const pos_T pos = curwin->w_cursor;
  const linenr_T lnum = tv_get_lnum(argvars);
  if (lnum >= 1 && lnum <= curbuf->b_ml.ml_line_count) {
    curwin->w_cursor.lnum = lnum;
    rettv->vval.v_number = get_lisp_indent();
    curwin->w_cursor = pos;
  } else {
    rettv->vval.v_number = -1;
  }
}
