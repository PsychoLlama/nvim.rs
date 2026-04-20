// search_shim.c: C accessor functions and remaining logic for search module

#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/change.h"
#include "nvim/charset.h"
#include "nvim/cmdhist.h"
#include "nvim/cursor.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_getln.h"
#include "nvim/fold.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/indent_c.h"
#include "nvim/macros_defs.h"
#include "nvim/mark.h"
#include "nvim/mark_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/os/fs.h"
#include "nvim/os/input.h"
#include "nvim/os/time.h"
#include "nvim/plines.h"
#include "nvim/profile.h"
#include "nvim/regexp.h"
#include "nvim/search.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/ui.h"

#include "search_shim.c.generated.h"
extern int rs_win_valid(win_T *win);
static const char e_search_hit_top_without_match_for_str[] = N_("E384: Search hit TOP without match for: %s");
static const char e_search_hit_bottom_without_match_for_str[] = N_("E385: Search hit BOTTOM without match for: %s");
extern int rs_magic_isset(void);
extern int rs_is_search_forward(void);
extern void rs_searchcount_compute(int pos_lnum, int pos_col, int pos_coladd, int maxcount, int timeout, bool recompute, const char *pattern, searchstat_T *stat);
char *nvim_search_skipwhite_ml_get(linenr_T lnum) { return skipwhite(ml_get(lnum)); }
void nvim_call_set_vv_searchforward(void) { set_vim_var_nr(VV_SEARCHFORWARD, rs_is_search_forward()); }
int nvim_get_search_match_lines(void) { return (int)search_match_lines; }
int nvim_get_search_match_endcol(void) { return (int)search_match_endcol; }
void nvim_set_search_match_lines(int val) { search_match_lines = (linenr_T)val; }
void nvim_set_search_match_endcol(int val) { search_match_endcol = (colnr_T)val; }
int nvim_get_p_is(void) { return p_is ? 1 : 0; }
void nvim_set_highlight_match(int value) { highlight_match = value != 0; }
linenr_T nvim_get_search_first_line(void) { return search_first_line; }
void nvim_set_search_first_line(linenr_T value) { search_first_line = value; }
linenr_T nvim_get_search_last_line(void) { return search_last_line; }
void nvim_set_search_last_line(linenr_T value) { search_last_line = value; }
int nvim_get_no_hlsearch(void) { return no_hlsearch ? 1 : 0; }
int nvim_get_no_smartcase(void) { return no_smartcase ? 1 : 0; }
void nvim_set_no_smartcase(int val) { no_smartcase = val != 0; }
int nvim_get_p_scs(void) { return p_scs ? 1 : 0; }
int nvim_get_p_ws(void) { return p_ws ? 1 : 0; }
int nvim_get_p_hls(void) { return p_hls ? 1 : 0; }
void nvim_showmatch_display_cursor(int match_lnum, int match_col, int match_coladd)
{
  OptInt *so = curwin->w_p_so >= 0 ? &curwin->w_p_so : &p_so;
  OptInt *siso = curwin->w_p_siso >= 0 ? &curwin->w_p_siso : &p_siso;
  pos_T mpos = { match_lnum, match_col, match_coladd };
  pos_T save_cursor = curwin->w_cursor;
  OptInt save_so = *so, save_siso = *siso;
  if (dollar_vcol >= 0 && dollar_vcol == curwin->w_virtcol) { dollar_vcol = -1; }
  curwin->w_virtcol++;
  colnr_T save_dollar_vcol = dollar_vcol;
  int save_state = State;
  State = MODE_SHOWMATCH; ui_cursor_shape();
  curwin->w_cursor = mpos; *so = 0; *siso = 0;
  show_cursor_info_later(false); update_screen(); setcursor(); ui_flush();
  dollar_vcol = save_dollar_vcol;
  if (vim_strchr(p_cpo, CPO_SHOWMATCH) != NULL) {
    os_delay((uint64_t)p_mat * 100 + 8, true);
  } else if (!char_avail()) {
    os_delay((uint64_t)p_mat * 100 + 9, false);
  }
  curwin->w_cursor = save_cursor; *so = save_so; *siso = save_siso;
  State = save_state; ui_cursor_shape();
}

void f_searchcount(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  pos_T pos = curwin->w_cursor;
  char *pattern = NULL;
  int maxcount = (int)p_msc;
  int timeout = SEARCH_STAT_DEF_TIMEOUT;
  bool recompute = true;
  searchstat_T stat;
  tv_dict_alloc_ret(rettv);
  if (argvars[0].v_type != VAR_UNKNOWN) {
    bool error = false;
    if (tv_check_for_nonnull_dict_arg(argvars, 0) == FAIL) { return; }
    dict_T *dict = argvars[0].vval.v_dict;
    dictitem_T *di = tv_dict_find(dict, "timeout", -1);
    if (di != NULL) { timeout = (int)tv_get_number_chk(&di->di_tv, &error); if (error) { return; } }
    di = tv_dict_find(dict, "maxcount", -1);
    if (di != NULL) { maxcount = (int)tv_get_number_chk(&di->di_tv, &error); if (error) { return; } }
    di = tv_dict_find(dict, "recompute", -1);
    if (di != NULL) { recompute = tv_get_number_chk(&di->di_tv, &error); if (error) { return; } }
    di = tv_dict_find(dict, "pattern", -1);
    if (di != NULL) { pattern = (char *)tv_get_string_chk(&di->di_tv); if (pattern == NULL) { return; } }
    di = tv_dict_find(dict, "pos", -1);
    if (di != NULL) {
      if (di->di_tv.v_type != VAR_LIST) { semsg(_(e_invarg2), "pos"); return; }
      if (tv_list_len(di->di_tv.vval.v_list) != 3) { semsg(_(e_invarg2), "List format should be [lnum, col, off]"); return; }
      listitem_T *li = tv_list_find(di->di_tv.vval.v_list, 0);
      if (li != NULL) { pos.lnum = (linenr_T)tv_get_number_chk(TV_LIST_ITEM_TV(li), &error); if (error) { return; } }
      li = tv_list_find(di->di_tv.vval.v_list, 1);
      if (li != NULL) { pos.col = (colnr_T)tv_get_number_chk(TV_LIST_ITEM_TV(li), &error) - 1; if (error) { return; } }
      li = tv_list_find(di->di_tv.vval.v_list, 2);
      if (li != NULL) { pos.coladd = (colnr_T)tv_get_number_chk(TV_LIST_ITEM_TV(li), &error); if (error) { return; } }
    }
  }
  rs_searchcount_compute(pos.lnum, pos.col, pos.coladd, maxcount, timeout, recompute, pattern, &stat);
  tv_dict_add_nr(rettv->vval.v_dict, S_LEN("current"), stat.cur);
  tv_dict_add_nr(rettv->vval.v_dict, S_LEN("total"), stat.cnt);
  tv_dict_add_nr(rettv->vval.v_dict, S_LEN("exact_match"), stat.exact_match);
  tv_dict_add_nr(rettv->vval.v_dict, S_LEN("incomplete"), stat.incomplete);
  tv_dict_add_nr(rettv->vval.v_dict, S_LEN("maxcount"), stat.last_maxcount);
}

void nvim_call_iemsg_restore_mismatch(void) { iemsg("restore_last_search_pattern() called more often than save_last_search_pattern()"); }
void nvim_emsg_nopresub(void) { emsg(_(e_nopresub)); }
void nvim_set_rc_did_emsg(void) { rc_did_emsg = true; }
int nvim_get_rc_did_emsg(void) { return rc_did_emsg; }
void nvim_clear_rc_did_emsg(void) { rc_did_emsg = false; }
void nvim_search_add_to_history(const char *pat, size_t patlen) { add_to_history(HIST_SEARCH, pat, patlen, true, NUL); }
int nvim_get_cmdmod_keeppatterns(void) { return (cmdmod.cmod_flags & CMOD_KEEPPATTERNS) != 0; }
int nvim_search_regcomp_compile(const char *pat, int magic, regmmatch_T *regmatch)
{
  regmatch->rmm_ic = ignorecase(pat); regmatch->rmm_maxcol = 0;
  regmatch->regprog = vim_regcomp(pat, magic ? RE_MAGIC : 0);
  return regmatch->regprog != NULL ? 1 : 0;
}
void nvim_inc_emsg_off(void) { emsg_off++; }
void nvim_dec_emsg_off(void) { emsg_off--; }
int nvim_search_regcomp(char *pat, size_t patlen, int pat_use, int options, void *regmatch_out) { return search_regcomp(pat, patlen, NULL, RE_SEARCH, pat_use, options, (regmmatch_T *)regmatch_out); }
int nvim_searchit_regexec_multi(void *regmatch, void *win, void *buf, linenr_T lnum, colnr_T col, void *tm, int *timed_out) { return vim_regexec_multi((regmmatch_T *)regmatch, (win_T *)win, (buf_T *)buf, lnum, col, (proftime_T *)tm, timed_out); }
void nvim_searchit_regfree(void *regmatch) { vim_regfree(((regmmatch_T *)regmatch)->regprog); }
int nvim_regmatch_regprog_is_null(const void *regmatch) { return ((const regmmatch_T *)regmatch)->regprog == NULL ? 1 : 0; }
colnr_T nvim_regmatch_rmm_matchcol(const void *regmatch) { return ((const regmmatch_T *)regmatch)->rmm_matchcol; }
int nvim_cpo_has_search(void) { return vim_strchr(p_cpo, CPO_SEARCH) != NULL ? 1 : 0; }
int nvim_profile_passed_limit(void *tm) { return tm != NULL && profile_passed_limit(*(proftime_T *)tm) ? 1 : 0; }
void nvim_searchit_emsg_patnotf(int p_ws_val, linenr_T lnum)
{
  char *pat = get_search_pat();
  if (p_ws_val) { semsg(_(e_patnotf2), pat); }
  else if (lnum == 0) { semsg(_(e_search_hit_top_without_match_for_str), pat); }
  else { semsg(_(e_search_hit_bottom_without_match_for_str), pat); }
}
void nvim_searchit_emsg_invalid(void) { semsg(_("E383: Invalid search string: %s"), get_search_pat()); }
void nvim_searchit_emsg_interr(void) { emsg(_(e_interr)); }
void nvim_searchit_give_warning(int dir) { give_warning(_(dir == BACKWARD ? top_bot_msg : bot_top_msg), true); }
void *nvim_regmmatch_alloc(void) { return xcalloc(1, sizeof(regmmatch_T)); }
int nvim_do_search_check_lineoff(void) { extern int rs_get_search_offset_line(int idx); return (rs_get_search_offset_line(0) && vim_strchr(p_cpo, CPO_LINEOFF) != NULL) ? 1 : 0; }
int nvim_do_search_hasFolding_fwd(linenr_T *lnum) { return hasFolding(curwin, *lnum, NULL, lnum) ? 1 : 0; }
int nvim_do_search_hasFolding_bwd(linenr_T *lnum) { return hasFolding(curwin, *lnum, lnum, NULL) ? 1 : 0; }
int nvim_fdo_has_search_flag(void) { return (fdo_flags & kOptFdoFlagSearch) != 0 ? 1 : 0; }
int nvim_hasFolding_cursor(void) { return hasFolding(curwin, curwin->w_cursor.lnum, NULL, NULL) ? 1 : 0; }
int nvim_get_curwin_cursor_coladd(void) { return (int)curwin->w_cursor.coladd; }
void nvim_do_search_hlsearch_on(int options) { if (no_hlsearch && !(options & SEARCH_KEEP)) { redraw_all_later(UPD_SOME_VALID); set_no_hlsearch(false); } }
char *nvim_do_search_skip_regexp(char *pat, int delim, char **newp) { return skip_regexp_ex(pat, delim, rs_magic_isset(), newp, NULL, NULL); }
void nvim_do_search_set_searchcmdlen(int val) { searchcmdlen = val; }
int nvim_do_search_get_searchcmdlen(void) { return searchcmdlen; }
char *nvim_search_ml_get(linenr_T lnum) { return ml_get(lnum); }
colnr_T nvim_search_ml_get_len(linenr_T lnum) { return ml_get_len(lnum); }
linenr_T nvim_search_get_line_count(void) { return curbuf->b_ml.ml_line_count; }
char *nvim_search_get_curbuf_b_p_mps(void) { return curbuf->b_p_mps; }
int nvim_search_get_curbuf_b_p_lisp(void) { return curbuf->b_p_lisp ? 1 : 0; }
int nvim_search_get_curwin_w_p_rl(void) { return curwin->w_p_rl ? 1 : 0; }
int nvim_search_check_linecomment(const char *line) { return check_linecomment(line); }
void nvim_search_set_oap_motion_type(void *oap, int motion_type) { if (oap != NULL) { ((oparg_T *)oap)->motion_type = (MotionType)motion_type; } }
const char *nvim_cap_get_nchar_composing_ptr(cmdarg_T *cap) { return cap ? cap->nchar_composing : NULL; }
void nvim_set_p_ws(int val) { p_ws = val; }
long nvim_get_p_msc(void) { return (long)p_msc; }
int nvim_curbuf_get_changedtick(void) { return (int)buf_get_changedtick(curbuf); }
void *nvim_search_get_curbuf_ptr(void) { return (void *)curbuf; }
int nvim_searchit_for_stat(int *pos_lnum, int *pos_col, int *pos_coladd,
                           int *end_lnum, int *end_col, int *end_coladd)
{
  pos_T pos = { *pos_lnum, *pos_col, *pos_coladd };
  pos_T endpos = { 0, 0, 0 };
  int retval = searchit(curwin, curbuf, &pos, &endpos, FORWARD, NULL, 0, 1, SEARCH_KEEP, RE_LAST, NULL);
  *pos_lnum = pos.lnum; *pos_col = pos.col; *pos_coladd = pos.coladd;
  *end_lnum = endpos.lnum; *end_col = endpos.col; *end_coladd = endpos.coladd;
  return retval;
}
int nvim_profile_passed_limit_val(proftime_T start) { return profile_passed_limit(start) ? 1 : 0; }
void nvim_stat_free_pat(char *pat) { xfree(pat); }
int nvim_curwin_rl_with_rlc_s(void) { return (curwin->w_p_rl && *curwin->w_p_rlc == 's') ? 1 : 0; }
void nvim_cmdline_stat_display(const char *msgbuf) { msg_hist_off = true; msg_ext_overwrite = true; msg_ext_set_kind("search_count"); give_warning(msgbuf, false); msg_hist_off = false; }
int nvim_is_pos_in_string(const char *line, int col) { return is_pos_in_string(line, (colnr_T)col); }
int nvim_is_zero_width_regcomp(const char *pat, size_t patlen, void *regmatch) { return search_regcomp((char *)pat, patlen, NULL, RE_SEARCH, RE_SEARCH, SEARCH_KEEP, (regmmatch_T *)regmatch); }
void nvim_regmatch_set_startcol(void *regmatch, int col) { ((regmmatch_T *)regmatch)->startpos[0].col = (colnr_T)col; }
int nvim_regmatch_get_startcol(const void *regmatch) { return ((const regmmatch_T *)regmatch)->startpos[0].col; }
int nvim_regmatch_get_startlnum(const void *regmatch) { return ((const regmmatch_T *)regmatch)->startpos[0].lnum; }
int nvim_regmatch_get_endcol(const void *regmatch) { return ((const regmmatch_T *)regmatch)->endpos[0].col; }
int nvim_regmatch_get_endlnum(const void *regmatch) { return ((const regmmatch_T *)regmatch)->endpos[0].lnum; }
int nvim_is_zero_width_regexec(void *regmatch, int lnum, int col) { return vim_regexec_multi((regmmatch_T *)regmatch, curwin, curbuf, (linenr_T)lnum, (colnr_T)col, NULL, NULL); }
int nvim_is_zero_width_searchit(const char *pat, size_t patlen, int dir, int flags, int *pos_lnum, int *pos_col, int *pos_coladd)
{
  pos_T pos = { *pos_lnum, *pos_col, *pos_coladd };
  int result = searchit(curwin, curbuf, &pos, NULL, (Direction)dir, (char *)pat, patlen, 1, SEARCH_KEEP + flags, RE_SEARCH, NULL);
  *pos_lnum = pos.lnum; *pos_col = pos.col; *pos_coladd = pos.coladd;
  return result;
}
int nvim_buf_ml_line_count(void *buf) { return ((buf_T *)buf)->b_ml.ml_line_count; }
const char *nvim_buf_get_line_skipwhite(void *buf, int lnum, int *skipwhite_off)
{
  char *ptr = ml_get_buf((buf_T *)buf, (linenr_T)lnum);
  char *p = skipwhite(ptr); *skipwhite_off = (int)(p - ptr); return p;
}
int nvim_mb_strcmp_ic_wrapper(int ic, const char *s1, const char *s2) { return mb_strcmp_ic((bool)ic, s1, s2); }
int nvim_mb_strnicmp_wrapper(const char *s1, const char *s2, size_t len) { return mb_strnicmp(s1, s2, len); }
int nvim_search_get_p_ic(void) { return p_ic ? 1 : 0; }
int nvim_shortmess_search(void) { return shortmess(SHM_SEARCH) ? 1 : 0; }
void nvim_give_search_wrap_warning(int at_top) { give_warning(_(at_top ? top_bot_msg : bot_top_msg), true); }
int nvim_showmatch_get_p_ri(void) { return p_ri ? 1 : 0; }
int nvim_showmatch_find_and_check(int *out_lnum, int *out_col, int *out_coladd)
{
  pos_T *lpos = findmatch(NULL, NUL);
  if (lpos == NULL) { return -1; }
  if (lpos->lnum < curwin->w_topline || lpos->lnum >= curwin->w_botline) { return 0; }
  colnr_T vcol = 0;
  if (!curwin->w_p_wrap) { getvcol(curwin, lpos, NULL, &vcol, NULL); }
  bool col_visible = curwin->w_p_wrap || (vcol >= curwin->w_leftcol && vcol < curwin->w_leftcol + curwin->w_view_width);
  if (!col_visible) { return 0; }
  *out_lnum = lpos->lnum; *out_col = lpos->col; *out_coladd = lpos->coladd;
  return 1;
}
void nvim_showmatch_beep(void) { vim_beep(kOptBoFlagShowmatch); }
colnr_T nvim_search_get_curwin_cursor_coladd(void) { return curwin->w_cursor.coladd; }
int nvim_search_incl_pos(int *lnum, int *col, int *coladd)
{
  pos_T pos = { *lnum, *col, *coladd }; int ret = incl(&pos);
  *lnum = pos.lnum; *col = pos.col; *coladd = pos.coladd; return ret;
}
int nvim_search_decl_pos(int *lnum, int *col, int *coladd)
{
  pos_T pos = { *lnum, *col, *coladd }; int ret = decl(&pos);
  *lnum = pos.lnum; *col = pos.col; *coladd = pos.coladd; return ret;
}
char *nvim_fpip_get_curbuf_fname(void) { return curbuf->b_fname; }
char *nvim_fpip_get_curbuf_ffname(void) { return curbuf->b_ffname; }
char *nvim_fpip_get_curbuf_b_p_inc(void) { return curbuf->b_p_inc; }
char *nvim_fpip_get_curbuf_b_p_def(void) { return curbuf->b_p_def; }
char *nvim_fpip_get_p_inc(void) { return p_inc; }
char *nvim_fpip_get_p_def(void) { return p_def; }
int nvim_fpip_get_p_js(void) { return p_js ? 1 : 0; }
linenr_T nvim_fpip_get_curbuf_ml_line_count(void) { return curbuf->b_ml.ml_line_count; }
linenr_T nvim_fpip_curwin_cursor_lnum(void) { return curwin->w_cursor.lnum; }
void nvim_fpip_reset_binding_curwin(void) { RESET_BINDING(curwin); }
void nvim_fpip_curwin_set_cursor_col(colnr_T col) { curwin->w_cursor.col = col; }
void nvim_fpip_curwin_set_curswant(void) { curwin->w_set_curswant = true; }
void *nvim_fpip_curwin_ptr(void) { return (void *)curwin; }
int nvim_fpip_get_buf_fnum(void *win) { return ((win_T *)win)->w_buffer->b_fnum; }
void nvim_fpip_set_curwin_cursor_lnum(linenr_T lnum) { curwin->w_cursor.lnum = lnum; }
int nvim_fpip_msg_silent(void) { return msg_silent; }
void msg_hist_off_set(int v) { msg_hist_off = v != 0; }
int nvim_search_current_searchit(int dir, int flags, int count,
                                 int *pos_lnum, int *pos_col, int *pos_coladd,
                                 int *end_lnum, int *end_col, int *end_coladd)
{
  extern const char *rs_get_last_used_pattern(void);
  extern size_t rs_get_last_used_pattern_len(void);
  const char *pat = rs_get_last_used_pattern();
  size_t patlen = rs_get_last_used_pattern_len();
  pos_T pos = { *pos_lnum, *pos_col, *pos_coladd };
  pos_T end_pos = { *end_lnum, *end_col, *end_coladd };
  int result = searchit(curwin, curbuf, &pos, &end_pos, dir ? FORWARD : BACKWARD, (char *)pat, patlen, count, SEARCH_KEEP | flags, RE_SEARCH, NULL);
  *pos_lnum = pos.lnum; *pos_col = pos.col; *pos_coladd = pos.coladd;
  *end_lnum = end_pos.lnum; *end_col = end_pos.col; *end_coladd = end_pos.coladd;
  return result;
}
