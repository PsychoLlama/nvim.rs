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

// Rust implementations (functions not yet given #[export_name])
typedef struct {
  int ntabs;
  int nspcs;
} TabstopFromtoResult;
extern TabstopFromtoResult rs_tabstop_fromto(int start_col, int end_col, int ts, const int *vts);

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

/// Get breakindent for window and line number (accessor for Rust FFI).
/// Combines get_breakindent_win with ml_get_buf for convenience.
int nvim_get_breakindent_win_lnum(win_T *wp, linenr_T lnum)
{
  return get_breakindent_win(wp, ml_get_buf(wp->w_buffer, lnum));
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
