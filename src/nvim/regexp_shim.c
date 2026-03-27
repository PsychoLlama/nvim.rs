// regexp_shim.c -- FFI bridge between Neovim's C core and the Rust regexp engine.
//
// All regexp logic (matching, compilation, parsing, substitution) lives in
// src/nvim-rs/regexp/src/lib.rs. This file contains only:
//   - Struct/type definitions (regexec_T, bt_regprog_T, nfa_regprog_T, etc.)
//   - Static global variables (rex, regstack, parser state, etc.)
//   - Accessor functions that let Rust read/write C struct fields and globals
//   - Error message helpers wrapping emsg()/semsg() with gettext
//   - Engine vtable definitions (bt_regengine, nfa_regengine)
//   - Thin wrappers for public API functions that need C type casts

#include <assert.h>
#include <stdbool.h>
#include <stddef.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/userfunc.h"
#include "nvim/globals.h"
#include "nvim/keycodes.h"
#include "nvim/macros_defs.h"
#include "nvim/mark.h"
#include "nvim/mark_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/option_vars.h"
#include "nvim/plines.h"
#include "nvim/pos_defs.h"
#include "nvim/profile.h"
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
#include "nvim/strings.h"

extern char *rs_partial_name(partial_T *pt);

/// Structure returned by vim_regcomp() to pass on to vim_regexec().
/// This is the general structure. For the actual matcher, two specific
/// structures are used. See code below.
struct regprog {
  regengine_T *engine;
  unsigned regflags;
  unsigned re_engine;  ///< Automatic, backtracking or NFA engine.
  unsigned re_flags;   ///< Second argument for vim_regcomp().
  bool re_in_use;      ///< prog is being executed
};

/// Structure used by the back track matcher.
/// These fields are only to be used in regexp.c!
/// See regexp.c for an explanation.
typedef struct {
  // These four members implement regprog_T.
  regengine_T *engine;
  unsigned regflags;
  unsigned re_engine;
  unsigned re_flags;
  bool re_in_use;

  int regstart;
  uint8_t reganch;
  uint8_t *regmust;
  int regmlen;
  uint8_t reghasz;
  uint8_t program[];
} bt_regprog_T;

struct regengine {
  /// bt_regcomp or nfa_regcomp
  regprog_T *(*regcomp)(uint8_t *, int);
  /// bt_regfree or nfa_regfree
  void (*regfree)(regprog_T *);
  /// bt_regexec_nl or nfa_regexec_nl
  int (*regexec_nl)(regmatch_T *, uint8_t *, colnr_T, bool);
  /// bt_regexec_mult or nfa_regexec_mult
  int (*regexec_multi)(regmmatch_T *, win_T *, buf_T *, linenr_T, colnr_T, proftime_T *, int *);
};

// flags for regflags
#define RF_ICASE    1   // ignore case
#define RF_NOICASE  2   // don't ignore case
#define RF_ICOMBINE 8   // ignore combining characters

#include "regexp_shim.c.generated.h"

typedef struct {
  regmatch_T *reg_match;
  regmmatch_T *reg_mmatch;

  uint8_t **reg_startp;
  uint8_t **reg_endp;
  lpos_T *reg_startpos;
  lpos_T *reg_endpos;

  win_T *reg_win;
  buf_T *reg_buf;
  linenr_T reg_firstlnum;
  linenr_T reg_maxline;
  bool reg_line_lbr;  // "\n" in string is line break

  // The current match-position is remembered with these variables:
  linenr_T lnum;  ///< line number, relative to first line
  uint8_t *line;   ///< start of current line
  uint8_t *input;  ///< current input, points into "line"

  int need_clear_subexpr;   ///< subexpressions still need to be cleared
  int need_clear_zsubexpr;  ///< extmatch subexpressions still need to be
                            ///< cleared

  // Internal copy of 'ignorecase'.  It is set at each call to vim_regexec().
  // Normally it gets the value of "rm_ic" or "rmm_ic", but when the pattern
  // contains '\c' or '\C' the value is overruled.
  bool reg_ic;

  // Similar to "reg_ic", but only for 'combining' characters.  Set with \Z
  // flag in the regexp.  Defaults to false, always.
  bool reg_icombine;

  bool reg_nobreak;

  // Copy of "rmm_maxcol": maximum column to search for a match.  Zero when
  // there is no maximum.
  colnr_T reg_maxcol;

  // State for the NFA engine regexec.
  int nfa_has_zend;     ///< NFA regexp \ze operator encountered.
  int nfa_has_backref;  ///< NFA regexp \1 .. \9 encountered.
  int nfa_nsubexpr;     ///< Number of sub expressions actually being used
                        ///< during execution. 1 if only the whole match
                        ///< (subexpr 0) is used.
  // listid is global, so that it increases on recursive calls to
  // nfa_regmatch(), which means we don't have to clear the lastlist field of
  // all the states.
  int nfa_listid;
  int nfa_alt_listid;

  int nfa_has_zsubexpr;  ///< NFA regexp has \z( ), set zsubexpr.
} regexec_T;

void *nvim_regexp_get_rex_ptr(void);  // returns regexec_T*
void *nvim_regexp_get_rsm_ptr(void);  // returns regsubmatch_T* equivalent
bool *nvim_regexp_get_can_f_submatch_ptr(void);
#define REX_PTR ((regexec_T *)nvim_regexp_get_rex_ptr())
#define RSM_PTR ((regsubmatch_T *)nvim_regexp_get_rsm_ptr())

void nvim_regexp_set_rc_did_emsg(int v) { rc_did_emsg = (bool)v; }

typedef struct {
  regmatch_T *sm_match;
  regmmatch_T *sm_mmatch;
  linenr_T sm_firstlnum;
  linenr_T sm_maxline;
  int sm_line_lbr;
} regsubmatch_T;

#define REG_MULTI       (REX_PTR->reg_match == NULL)

int64_t *nvim_regexp_get_rex_reg_buf_chartab(void) { return REX_PTR->reg_buf->b_chartab; }
int nvim_regexp_get_got_int(void) { return got_int; }
void *nvim_regexp_get_re_extmatch_out(void) { return (void *)re_extmatch_out; }
void nvim_regexp_set_re_extmatch_out(void *em) { re_extmatch_out = (reg_extmatch_T *)em; }
void nvim_regexp_set_re_extmatch_out_match(int i, uint8_t *v) { re_extmatch_out->matches[i] = v; }
int nvim_regexp_visual_quick_check(void) { return (REX_PTR->reg_buf == curbuf && VIsual.lnum != 0 && REG_MULTI) ? 1 : 0; }

void *nvim_regexp_get_visual_area(int32_t *top_lnum, int32_t *top_col,
                                  int32_t *bot_lnum, int32_t *bot_col,
                                  int *mode, int32_t *curswant_out)
{
  pos_T top, bot;
  win_T *wp = REX_PTR->reg_win == NULL ? curwin : REX_PTR->reg_win;
  int vmode;
  colnr_T curswant;

  if (VIsual_active) {
    if (lt(VIsual, wp->w_cursor)) {
      top = VIsual;
      bot = wp->w_cursor;
    } else {
      top = wp->w_cursor;
      bot = VIsual;
    }
    vmode = VIsual_mode;
    curswant = wp->w_curswant;
  } else {
    if (lt(curbuf->b_visual.vi_start, curbuf->b_visual.vi_end)) {
      top = curbuf->b_visual.vi_start;
      bot = curbuf->b_visual.vi_end;
    } else {
      top = curbuf->b_visual.vi_end;
      bot = curbuf->b_visual.vi_start;
    }
    // a substitute command may have removed some lines
    if (bot.lnum > curbuf->b_ml.ml_line_count) {
      bot.lnum = curbuf->b_ml.ml_line_count;
    }
    vmode = curbuf->b_visual.vi_mode;
    curswant = curbuf->b_visual.vi_curswant;
  }

  *top_lnum = (int32_t)top.lnum;
  *top_col = (int32_t)top.col;
  *bot_lnum = (int32_t)bot.lnum;
  *bot_col = (int32_t)bot.col;
  *mode = vmode;
  *curswant_out = (int32_t)curswant;
  return (void *)wp;
}

int nvim_regexp_get_p_sel_char(void) { return *p_sel; }
void nvim_regexp_call_getvvcol(void *wp, int32_t lnum, int32_t col, int32_t *start_out, int32_t *end_out)
{
  pos_T pos = { .lnum = (linenr_T)lnum, .col = (colnr_T)col, .coladd = 0 };
  colnr_T s, e;
  getvvcol((win_T *)wp, &pos, &s, NULL, &e);
  *start_out = (int32_t)s; *end_out = (int32_t)e;
}

int32_t nvim_regexp_call_win_linetabsize(void *wp, int32_t lnum, const char *line, int32_t col) { return (int32_t)win_linetabsize((win_T *)wp, (linenr_T)lnum, (char *)line, (colnr_T)col); }

/// Put the submatches in "argv[argskip]" which is a list passed into
/// call_func() by vim_regsub_both().
static int fill_submatch_list(int argc FUNC_ATTR_UNUSED, typval_T *argv, int argskip, ufunc_T *fp)
  FUNC_ATTR_NONNULL_ALL
{
  typval_T *listarg = argv + argskip;

  if (!fp->uf_varargs && fp->uf_args.ga_len <= argskip) {
    // called function doesn't take a submatches argument
    return argskip;
  }

  // Relies on sl_list to be the first item in staticList10_T.
  tv_list_init_static10((staticList10_T *)listarg->vval.v_list);

  // There are always 10 list items in staticList10_T.
  listitem_T *li = tv_list_first(listarg->vval.v_list);
  for (int i = 0; i < 10; i++) {
    char *s = RSM_PTR->sm_match->startp[i];
    if (s == NULL || RSM_PTR->sm_match->endp[i] == NULL) {
      s = NULL;
    } else {
      s = xstrnsave(s, (size_t)(RSM_PTR->sm_match->endp[i] - s));
    }
    TV_LIST_ITEM_TV(li)->v_type = VAR_STRING;
    TV_LIST_ITEM_TV(li)->vval.v_string = s;
    li = TV_LIST_ITEM_NEXT(argv->vval.v_list, li);
  }
  return argskip + 1;
}

static void clear_submatch_list(staticList10_T *sl)
{
  TV_LIST_ITER(&sl->sl_list, li, {
    xfree(TV_LIST_ITEM_TV(li)->vval.v_string);
  });
}

char **nvim_regexp_get_eval_result_ptr(int i);
int *nvim_regexp_get_regsub_nesting_ptr(void);

/// Evaluate a \= substitution expression; handles VimL call_func/eval_to_string.
/// Returns strlen of the result stored in EVAL_RESULT[nested], or 0 on error.
int nvim_regexp_eval_regsub_expr(char *source, void *expr_ptr, int flags, int nested)
{
  typval_T *expr = (typval_T *)expr_ptr;
  const bool prev_can_f_submatch = (*nvim_regexp_get_can_f_submatch_ptr());
  regsubmatch_T rsm_save;

  // Access eval_result and regsub_nesting via Rust-exported pointers
  char **eval_result_p = nvim_regexp_get_eval_result_ptr(nested);
  int *regsub_nesting_p = nvim_regexp_get_regsub_nesting_ptr();

  XFREE_CLEAR(*eval_result_p);

  if (*nvim_regexp_get_can_f_submatch_ptr()) {
    rsm_save = *RSM_PTR;
  }
  *nvim_regexp_get_can_f_submatch_ptr() = true;
  RSM_PTR->sm_match = REX_PTR->reg_match;
  RSM_PTR->sm_mmatch = REX_PTR->reg_mmatch;
  RSM_PTR->sm_firstlnum = REX_PTR->reg_firstlnum;
  RSM_PTR->sm_maxline = REX_PTR->reg_maxline;
  RSM_PTR->sm_line_lbr = REX_PTR->reg_line_lbr;

  (*regsub_nesting_p)++;

  if (expr != NULL) {
    typval_T argv[2];
    typval_T rettv;
    staticList10_T matchList = TV_LIST_STATIC10_INIT;
    rettv.v_type = VAR_STRING;
    rettv.vval.v_string = NULL;
    argv[0].v_type = VAR_LIST;
    argv[0].vval.v_list = &matchList.sl_list;
    funcexe_T funcexe = FUNCEXE_INIT;
    funcexe.fe_argv_func = fill_submatch_list;
    funcexe.fe_evaluate = true;
    char *s;
    if (expr->v_type == VAR_FUNC) {
      s = expr->vval.v_string;
      call_func(s, -1, &rettv, 1, argv, &funcexe);
    } else if (expr->v_type == VAR_PARTIAL) {
      partial_T *partial = expr->vval.v_partial;

      s = rs_partial_name(partial);
      funcexe.fe_partial = partial;
      call_func(s, -1, &rettv, 1, argv, &funcexe);
    }
    if (tv_list_len(&matchList.sl_list) > 0) {
      clear_submatch_list(&matchList);
    }
    if (rettv.v_type == VAR_UNKNOWN) {
      *eval_result_p = NULL;
    } else {
      char buf[NUMBUFLEN];
      *eval_result_p = (char *)tv_get_string_buf_chk(&rettv, buf);
      if (*eval_result_p != NULL) {
        *eval_result_p = xstrdup(*eval_result_p);
      }
    }
    tv_clear(&rettv);
  } else {
    *eval_result_p = eval_to_string(source + 2, true, false);
  }
  (*regsub_nesting_p)--;

  if (*eval_result_p != NULL) {
    int had_backslash = false;

    for (char *s = *eval_result_p; *s != NUL; MB_PTR_ADV(s)) {
      if (*s == NL && !RSM_PTR->sm_line_lbr) {
        *s = CAR;
      } else if (*s == '\\' && s[1] != NUL) {
        s++;
        if (*s == NL && !RSM_PTR->sm_line_lbr) {
          *s = CAR;
        }
        had_backslash = true;
      }
    }
    if (had_backslash && (flags & REGSUB_BACKSLASH)) {
      char *s = vim_strsave_escaped(*eval_result_p, "\\");
      xfree(*eval_result_p);
      *eval_result_p = s;
    }
  }

  *nvim_regexp_get_can_f_submatch_ptr() = prev_can_f_submatch;
  if (*nvim_regexp_get_can_f_submatch_ptr()) {
    *RSM_PTR = rsm_save;
  }

  if (*eval_result_p != NULL) {
    return (int)strlen(*eval_result_p);
  }
  return 0;
}

int nvim_regexp_get_reg_do_extmatch(void) { return reg_do_extmatch; }
int32_t nvim_regexp_get_curwin_lnum(void) { return (int32_t)curwin->w_cursor.lnum; }
int32_t nvim_regexp_get_curwin_col(void) { return (int32_t)curwin->w_cursor.col; }
int32_t nvim_regexp_get_curwin_vcol(void) { colnr_T vcol = 0; getvvcol(curwin, &curwin->w_cursor, NULL, NULL, &vcol); return (int32_t)(++vcol); }
int64_t nvim_regexp_get_p_mmp(void) { return p_mmp; }
uint8_t *nvim_regexp_get_re_extmatch_in_match(int no) { return (re_extmatch_in != NULL && re_extmatch_in->matches[no] != NULL) ? re_extmatch_in->matches[no] : NULL; }
void *nvim_regexp_get_rex_reg_win_or_curwin(void) { return (void *)(REX_PTR->reg_win == NULL ? curwin : REX_PTR->reg_win); }
int32_t nvim_regexp_get_rex_reg_win_cursor_lnum(void) { return REX_PTR->reg_win != NULL ? (int32_t)REX_PTR->reg_win->w_cursor.lnum : 0; }
int32_t nvim_regexp_get_rex_reg_win_cursor_col(void) { return REX_PTR->reg_win != NULL ? (int32_t)REX_PTR->reg_win->w_cursor.col : 0; }

void *nvim_regexp_get_curwin(void) { return (void *)curwin; }
int64_t nvim_regexp_get_win_b_p_ts(void *wp) { return (int64_t)((win_T *)wp)->w_buffer->b_p_ts; }
int32_t nvim_regexp_get_win_buf_line_count(void *wp) { return (int32_t)((win_T *)wp)->w_buffer->b_ml.ml_line_count; }
int nvim_regexp_fmark_is_set(void *fm) { return fm != NULL && ((fmark_T *)fm)->mark.lnum > 0; }
int32_t nvim_regexp_fmark_get_lnum(void *fm) { return (int32_t)((fmark_T *)fm)->mark.lnum; }
int32_t nvim_regexp_fmark_get_col(void *fm) { return (int32_t)((fmark_T *)fm)->mark.col; }
void *nvim_regexp_get_curbuf(void) { return (void *)curbuf; }
int32_t nvim_regexp_get_curbuf_ml_line_count(void) { return (int32_t)curbuf->b_ml.ml_line_count; }
int32_t nvim_regexp_get_buf_ml_line_count(void *buf) { return (int32_t)((buf_T *)buf)->b_ml.ml_line_count; }
int32_t nvim_regexp_get_p_re(void) { return (int32_t)p_re; }
void nvim_regexp_set_p_re(int32_t v) { p_re = (long)v; }
void nvim_regexp_set_reg_do_extmatch(int v) { reg_do_extmatch = v; }
int64_t nvim_regexp_get_p_verbose(void) { return p_verbose; }
size_t nvim_regexp_get_regmatch_size(void) { return sizeof(regmatch_T); }
void nvim_regexp_init_regmatch(void *buf, void *prog, int rm_ic) { regmatch_T *rmp = (regmatch_T *)buf; memset(rmp, 0, sizeof(regmatch_T)); rmp->regprog = (regprog_T *)prog; rmp->rm_ic = (bool)rm_ic; }
void nvim_regexp_set_rex_reg_buf_curbuf(void) { REX_PTR->reg_buf = curbuf; }
int nvim_regexp_get_called_emsg(void) { return called_emsg; }
void *nvim_regexp_alloc_bt_regprog(int64_t regsize_val) { bt_regprog_T *r = xmalloc(offsetof(bt_regprog_T, program) + (size_t)regsize_val); r->re_in_use = false; return r; }
