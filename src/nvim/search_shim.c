// search_shim.c: C accessor functions and remaining logic for search module

#include <assert.h>
#include <inttypes.h>
#include <limits.h>
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
#include "nvim/file_search.h"
#include "nvim/fileio.h"
#include "nvim/fold.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/indent_c.h"
#include "nvim/insexpand.h"
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
#include "nvim/move.h"
#include "nvim/path.h"
#include "nvim/plines.h"
#include "nvim/profile.h"
#include "nvim/regexp.h"
#include "nvim/search.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/tag.h"
#include "nvim/ui.h"
#include "nvim/window.h"

#include "search_shim.c.generated.h"
extern int rs_win_valid(win_T *win);
static const char e_search_hit_top_without_match_for_str[] = N_("E384: Search hit TOP without match for: %s");
static const char e_search_hit_bottom_without_match_for_str[] = N_("E385: Search hit BOTTOM without match for: %s");
extern int rs_magic_isset(void);
extern int rs_is_search_forward(void);
extern int rs_compl_status_adding(void);
extern int rs_compl_status_sol(void);
extern int rs_ins_compl_len(void);
extern int rs_ins_compl_interrupted(void);
extern char *rs_find_word_start(char *ptr);
extern char *rs_find_word_end(char *ptr);
extern void rs_searchcount_compute(int pos_lnum, int pos_col, int pos_coladd, int maxcount, int timeout, bool recompute, const char *pattern, searchstat_T *stat);
char *nvim_search_skipwhite_ml_get(linenr_T lnum) { return skipwhite(ml_get(lnum)); }
void nvim_call_set_vv_searchforward(void) { set_vim_var_nr(VV_SEARCHFORWARD, rs_is_search_forward()); }
typedef struct { FILE *fp; char *name; linenr_T lnum; int matched; } SearchedFile;
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

static char *get_line_and_copy(linenr_T lnum, char *buf) { xstrlcpy(buf, ml_get(lnum), LSIZE); return buf; }

void find_pattern_in_path(char *ptr, Direction dir, size_t len, bool whole, bool skip_comments,
                          int type, int count, int action, linenr_T start_lnum, linenr_T end_lnum,
                          bool forceit, bool silent)
{
  FpipInitResult init = nvim_fpip_init(ptr, dir, len, whole ? 1 : 0, skip_comments ? 1 : 0,
                                       type, count, action, start_lnum, end_lnum,
                                       forceit ? 1 : 0, silent ? 1 : 0);
  if (!init.ok || init.handle == NULL) { if (init.handle != NULL) { nvim_fpip_cleanup(init.handle); } return; }
  nvim_fpip_run(init.handle);
  nvim_fpip_cleanup(init.handle);
}

/// Opaque state for the find_pattern_in_path operation.
typedef struct {
  SearchedFile *files;
  int max_path_depth;
  int old_files;
  int depth;
  int depth_displayed;
  int match_count;
  Direction dir;
  char *ptr;
  size_t len;
  bool whole;
  bool skip_comments;
  int type;
  int count;
  int action;
  linenr_T start_lnum;
  linenr_T end_lnum;
  bool forceit;
  bool silent;
  char *file_line;
  char *curr_fname;
  char *prev_fname;
  regmatch_T regmatch;
  regmatch_T incl_regmatch;
  regmatch_T def_regmatch;
  char *inc_opt;
  bool did_show;
  bool found;
  linenr_T lnum;
  int l_g_do_tagpreview;
} FpipState;

FpipInitResult nvim_fpip_init(const char *ptr, int dir, size_t len,
                              int whole, int skip_comments,
                              int type, int count, int action,
                              linenr_T start_lnum, linenr_T end_lnum,
                              int forceit, int silent)
{
  FpipState *st = xcalloc(1, sizeof(FpipState));
  st->max_path_depth = 50; st->match_count = 1;
  st->ptr = (char *)ptr; st->dir = (Direction)dir; st->len = len;
  st->whole = whole != 0; st->skip_comments = skip_comments != 0;
  st->type = type; st->count = count; st->action = action;
  st->start_lnum = start_lnum; st->end_lnum = end_lnum;
  st->forceit = forceit != 0; st->silent = silent != 0;
  st->curr_fname = curbuf->b_fname; st->prev_fname = NULL;
  st->did_show = false; st->found = false; st->l_g_do_tagpreview = g_do_tagpreview;
  st->depth = -1; st->depth_displayed = -1;
  st->regmatch.regprog = NULL; st->incl_regmatch.regprog = NULL; st->def_regmatch.regprog = NULL;
  st->file_line = xmalloc(LSIZE);
  if (type != CHECK_PATH && type != FIND_DEFINE
      && !rs_compl_status_sol()) {
    size_t patsize = len + 5;
    char *pat = xmalloc(patsize);
    assert(len <= INT_MAX);
    snprintf(pat, patsize, st->whole ? "\\<%.*s\\>" : "%.*s", (int)len, ptr);
    st->regmatch.rm_ic = ignorecase(pat);
    st->regmatch.regprog = vim_regcomp(pat, rs_magic_isset() ? RE_MAGIC : 0);
    xfree(pat);
    if (st->regmatch.regprog == NULL) {
      return (FpipInitResult){ st, 0 };
    }
  }
  st->inc_opt = (*curbuf->b_p_inc == NUL) ? p_inc : curbuf->b_p_inc;
  if (*st->inc_opt != NUL) {
    st->incl_regmatch.regprog = vim_regcomp(st->inc_opt, rs_magic_isset() ? RE_MAGIC : 0);
    if (st->incl_regmatch.regprog == NULL) { return (FpipInitResult){ st, 0 }; }
    st->incl_regmatch.rm_ic = false;
  }
  if (type == FIND_DEFINE && (*curbuf->b_p_def != NUL || *p_def != NUL)) {
    st->def_regmatch.regprog = vim_regcomp(*curbuf->b_p_def == NUL ? p_def : curbuf->b_p_def, rs_magic_isset() ? RE_MAGIC : 0);
    if (st->def_regmatch.regprog == NULL) { return (FpipInitResult){ st, 0 }; }
    st->def_regmatch.rm_ic = false;
  }
  st->files = xcalloc((size_t)st->max_path_depth, sizeof(SearchedFile));
  st->old_files = st->max_path_depth; st->end_lnum = MIN(st->end_lnum, curbuf->b_ml.ml_line_count);
  st->lnum = MIN(st->start_lnum, st->end_lnum); return (FpipInitResult){ st, 1 };
}

void nvim_fpip_run(void *handle)
{
  FpipState *st = (FpipState *)handle;
  SearchedFile *files = st->files;
  int max_path_depth = st->max_path_depth, old_files = st->old_files;
  int depth = st->depth, depth_displayed = st->depth_displayed, match_count = st->match_count;
  char *ptr = st->ptr; size_t len = st->len;
  int type = st->type, action = st->action;
  linenr_T end_lnum = st->end_lnum, lnum = st->lnum;
  char *file_line = st->file_line, *curr_fname = st->curr_fname, *prev_fname = st->prev_fname;
  regmatch_T *regmatch = &st->regmatch, *incl_regmatch = &st->incl_regmatch, *def_regmatch = &st->def_regmatch;
  char *inc_opt = st->inc_opt;
  bool did_show = st->did_show, found = st->found;
  int l_g_do_tagpreview = st->l_g_do_tagpreview;
  char *new_fname, *p; bool define_matched; bool matched = false; int i;
  char *already = NULL, *startp = NULL; win_T *curwin_save = NULL;
  char *line = get_line_and_copy(lnum, file_line);
  while (true) {
    if (incl_regmatch->regprog != NULL
        && vim_regexec(incl_regmatch, line, 0)) {
      char *p_fname = (curr_fname == curbuf->b_fname) ? curbuf->b_ffname : curr_fname;
      if (inc_opt != NULL && strstr(inc_opt, "\\zs") != NULL) {
        new_fname = find_file_name_in_path(incl_regmatch->startp[0],
                                           (size_t)(incl_regmatch->endp[0]
                                                    - incl_regmatch->startp[0]),
                                           FNAME_EXP|FNAME_INCL|FNAME_REL,
                                           1, p_fname);
      } else {
        new_fname = file_name_in_line(incl_regmatch->endp[0], 0,
                                      FNAME_EXP|FNAME_INCL|FNAME_REL, 1, p_fname,
                                      NULL);
      }
      bool already_searched = false;
      if (new_fname != NULL) {
        for (i = 0;; i++) {
          if (i == depth + 1) {
            i = old_files;
          }
          if (i == max_path_depth) {
            break;
          }
          if (path_full_compare(new_fname, files[i].name, true,
                                true) & kEqualFiles) {
            if (type != CHECK_PATH
                && action == ACTION_SHOW_ALL && files[i].matched) {
              msg_putchar('\n');
              if (!got_int) {
                msg_home_replace(new_fname);
                msg_puts(_(" (includes previously listed match)"));
                prev_fname = NULL;
              }
            }
            XFREE_CLEAR(new_fname);
            already_searched = true;
            break;
          }
        }
      }
      if (type == CHECK_PATH && (action == ACTION_SHOW_ALL
                                 || (new_fname == NULL && !already_searched))) {
        if (did_show) {
          msg_putchar('\n');
        } else {
          gotocmdline(true);
          msg_puts_title(_("--- Included files "));
          if (action != ACTION_SHOW_ALL) {
            msg_puts_title(_("not found "));
          }
          msg_puts_title(_("in path ---\n"));
        }
        did_show = true;
        while (depth_displayed < depth && !got_int) {
          depth_displayed++;
          for (i = 0; i < depth_displayed; i++) {
            msg_puts("  ");
          }
          msg_home_replace(files[depth_displayed].name);
          msg_puts(" -->\n");
        }
        if (!got_int) {
          for (i = 0; i <= depth_displayed; i++) {
            msg_puts("  ");
          }
          if (new_fname != NULL) {
            msg_outtrans(new_fname, HLF_D, false);
          } else {
            if (inc_opt != NULL
                && strstr(inc_opt, "\\zs") != NULL) {
              p = incl_regmatch->startp[0];
              i = (int)(incl_regmatch->endp[0]
                        - incl_regmatch->startp[0]);
            } else {
              for (p = incl_regmatch->endp[0];
                   *p && !vim_isfilec((uint8_t)(*p)); p++) {}
              for (i = 0; vim_isfilec((uint8_t)p[i]); i++) {}
            }
            if (i == 0) {
              p = incl_regmatch->endp[0];
              i = (int)strlen(p);
            } else if (p > line) {
              if (p[-1] == '"' || p[-1] == '<') {
                p--;
                i++;
              }
              if (p[i] == '"' || p[i] == '>') {
                i++;
              }
            }
            char save_char = p[i];
            p[i] = NUL;
            msg_outtrans(p, HLF_D, false);
            p[i] = save_char;
          }
          if (new_fname == NULL && action == ACTION_SHOW_ALL) {
            if (already_searched) {
              msg_puts(_("  (Already listed)"));
            } else {
              msg_puts(_("  NOT FOUND"));
            }
          }
        }
      }
      if (new_fname != NULL) {
        SearchedFile *bigger;
        if (depth + 1 == old_files) {
          bigger = xmalloc((size_t)max_path_depth * 2 * sizeof(SearchedFile));
          for (i = 0; i <= depth; i++) {
            bigger[i] = files[i];
          }
          for (i = depth + 1; i < old_files + max_path_depth; i++) {
            bigger[i].fp = NULL;
            bigger[i].name = NULL;
            bigger[i].lnum = 0;
            bigger[i].matched = false;
          }
          for (i = old_files; i < max_path_depth; i++) {
            bigger[i + max_path_depth] = files[i];
          }
          old_files += max_path_depth;
          max_path_depth *= 2;
          xfree(files);
          files = bigger;
        }
        if ((files[depth + 1].fp = os_fopen(new_fname, "r")) == NULL) {
          xfree(new_fname);
        } else {
          if (++depth == old_files) {
            xfree(files[old_files].name);
            old_files++;
          }
          files[depth].name = curr_fname = new_fname;
          files[depth].lnum = 0; files[depth].matched = false;
          if (action == ACTION_EXPAND && !shortmess(SHM_COMPLETIONSCAN) && !st->silent) {
            msg_hist_off = true; vim_snprintf(IObuff, IOSIZE, _("Scanning included file: %s"), new_fname); msg_trunc(IObuff, true, HLF_R);
          } else if (p_verbose >= 5) {
            verbose_enter(); smsg(0, _("Searching included file %s"), new_fname); verbose_leave();
          }
        }
      }
    } else {
      p = line;
search_line:
      define_matched = false;
      if (def_regmatch->regprog != NULL
          && vim_regexec(def_regmatch, line, 0)) {
        p = def_regmatch->endp[0];
        while (*p && !vim_iswordc((uint8_t)(*p))) {
          p++;
        }
        define_matched = true;
      }
      if (def_regmatch->regprog == NULL || define_matched) {
        if (define_matched || rs_compl_status_sol()) {
          startp = skipwhite(p);
          matched = p_ic ? !mb_strnicmp(startp, ptr, len) : !strncmp(startp, ptr, len);
          if (matched && define_matched && st->whole && vim_iswordc((uint8_t)startp[len])) { matched = false; }
        } else if (regmatch->regprog != NULL
                   && vim_regexec(regmatch, line, (colnr_T)(p - line))) {
          matched = true;
          startp = regmatch->startp[0];
          if (st->skip_comments) {
            if ((*line != '#'
                 || strncmp(skipwhite(line + 1), "define", 6) != 0)
                && get_leader_len(line, NULL, false, true)) {
              matched = false;
            }
            p = skipwhite(line);
            if (matched
                || (p[0] == '/' && p[1] == '*') || p[0] == '*') {
              for (p = line; *p && p < startp; p++) {
                if (matched
                    && p[0] == '/'
                    && (p[1] == '*' || p[1] == '/')) {
                  matched = false;
                  if (p[1] == '/') {
                    break;
                  }
                  p++;
                } else if (!matched && p[0] == '*' && p[1] == '/') {
                  matched = true;
                  p++;
                }
              }
            }
          }
        }
      }
    }
    if (matched) {
      if (action == ACTION_EXPAND) {
        bool cont_s_ipos = false;
        if (depth == -1 && lnum == curwin->w_cursor.lnum) {
          break;
        }
        found = true;
        char *aux = p = startp;
        if (rs_compl_status_adding() && (int)strlen(p) >= rs_ins_compl_len()) {
          p += rs_ins_compl_len();
          if (vim_iswordp(p)) {
            goto exit_matched;
          }
          p = rs_find_word_start(p);
        }
        p = rs_find_word_end(p);
        i = (int)(p - aux);
        if (rs_compl_status_adding() && i == rs_ins_compl_len()) {
          strncpy(IObuff, aux, (size_t)i);  // NOLINT(runtime/printf)
          if (depth < 0) {
            if (lnum >= end_lnum) { goto exit_matched; }
            line = get_line_and_copy(++lnum, file_line);
          } else if (vim_fgets(line = file_line, LSIZE, files[depth].fp)) {
            goto exit_matched;
          }
          already = aux = p = skipwhite(line);
          p = rs_find_word_start(p);
          p = rs_find_word_end(p);
          if (p > aux) {
            if (*aux != ')' && IObuff[i - 1] != TAB) {
              if (IObuff[i - 1] != ' ') {
                IObuff[i++] = ' ';
              }
              if (p_js
                  && (IObuff[i - 2] == '.'
                      || IObuff[i - 2] == '?'
                      || IObuff[i - 2] == '!')) {
                IObuff[i++] = ' ';
              }
            }
            if (p - aux >= IOSIZE - i) {
              p = aux + IOSIZE - i - 1;
            }
            strncpy(IObuff + i, aux, (size_t)(p - aux));  // NOLINT(runtime/printf)
            i += (int)(p - aux);
            cont_s_ipos = true;
          }
          IObuff[i] = NUL;
          aux = IObuff;
          if (i == rs_ins_compl_len()) {
            goto exit_matched;
          }
        }
        const int add_r = ins_compl_add_infercase(aux, i, p_ic,
                                                  curr_fname == curbuf->b_fname
                                                  ? NULL : curr_fname,
                                                  st->dir, cont_s_ipos, 0);
        if (add_r == OK) {
          st->dir = FORWARD;
        } else if (add_r == FAIL) {
          break;
        }
      } else if (action == ACTION_SHOW_ALL) {
        found = true;
        if (!did_show) {
          gotocmdline(true);
        }
        if (curr_fname != prev_fname) {
          if (did_show) {
            msg_putchar('\n');
          }
          if (!got_int) {
            msg_home_replace(curr_fname);
          }
          prev_fname = curr_fname;
        }
        did_show = true;
        if (!got_int) {
          show_pat_in_path(line, type, true, action,
                           (depth == -1) ? NULL : files[depth].fp,
                           (depth == -1) ? &lnum : &files[depth].lnum,
                           match_count++);
        }
        for (i = 0; i <= depth; i++) { files[i].matched = true; }
      } else if (--st->count <= 0) {
        found = true;
        if (depth == -1 && lnum == curwin->w_cursor.lnum
            && l_g_do_tagpreview == 0) {
          emsg(_("E387: Match is on current line"));
        } else if (action == ACTION_SHOW) {
          show_pat_in_path(line, type, did_show, action,
                           (depth == -1) ? NULL : files[depth].fp,
                           (depth == -1) ? &lnum : &files[depth].lnum, 1);
          did_show = true;
        } else {
          if (l_g_do_tagpreview != 0) { curwin_save = curwin; prepare_tagpreview(true); }
          if (action == ACTION_SPLIT) {
            if (win_split(0, 0) == FAIL) { break; }
            RESET_BINDING(curwin);
          }
          if (depth == -1) {
            if (l_g_do_tagpreview != 0) {
              if (!rs_win_valid(curwin_save)) { break; }
              if (!GETFILE_SUCCESS(getfile(curwin_save->w_buffer->b_fnum, NULL, NULL, true, lnum, st->forceit))) { break; }
            } else { setpcmark(); }
            curwin->w_cursor.lnum = lnum; check_cursor(curwin);
          } else {
            if (!GETFILE_SUCCESS(getfile(0, files[depth].name, NULL, true, files[depth].lnum, st->forceit))) { break; }
            curwin->w_cursor.lnum = files[depth].lnum;
          }
        }
        if (action != ACTION_SHOW) { curwin->w_cursor.col = (colnr_T)(startp - line); curwin->w_set_curswant = true; }
        if (l_g_do_tagpreview != 0 && curwin != curwin_save && rs_win_valid(curwin_save)) {
          validate_cursor(curwin); redraw_later(curwin, UPD_VALID); win_enter(curwin_save, true);
        }
        break;
      }
exit_matched:
      matched = false;
      if (def_regmatch->regprog == NULL && action == ACTION_EXPAND
          && !rs_compl_status_sol() && *startp != NUL && *(startp + utfc_ptr2len(startp)) != NUL) {
        goto search_line;
      }
    }
    line_breakcheck();
    if (action == ACTION_EXPAND) { rs_ins_compl_check_keys(30, 0); }
    if (got_int || rs_ins_compl_interrupted()) { break; }
    while (depth >= 0 && !already
           && vim_fgets(line = file_line, LSIZE, files[depth].fp)) {
      fclose(files[depth].fp);
      old_files--;
      files[old_files].name = files[depth].name;
      files[old_files].matched = files[depth].matched;
      depth--;
      curr_fname = (depth == -1) ? curbuf->b_fname
                                 : files[depth].name;
      depth_displayed = MIN(depth_displayed, depth);
    }
    if (depth >= 0) {
      files[depth].lnum++;
      i = (int)strlen(line);
      if (i > 0 && line[i - 1] == '\n') {
        line[--i] = NUL;
      }
      if (i > 0 && line[i - 1] == '\r') {
        line[--i] = NUL;
      }
    } else if (!already) {
      if (++lnum > end_lnum) {
        break;
      }
      line = get_line_and_copy(lnum, file_line);
    }
    already = NULL;
  }
  for (i = 0; i <= depth; i++) { fclose(files[i].fp); xfree(files[i].name); }
  for (i = old_files; i < max_path_depth; i++) { xfree(files[i].name); }
  xfree(files);
  if (type == CHECK_PATH) {
    if (!did_show) {
      if (action != ACTION_SHOW_ALL) { msg(_("All included files were found"), 0); }
      else { msg(_("No included files"), 0); }
    }
  } else if (!found && action != ACTION_EXPAND && !st->silent) {
    if (got_int || rs_ins_compl_interrupted()) { emsg(_(e_interr)); }
    else if (type == FIND_DEFINE) { emsg(_("E388: Couldn't find definition")); }
    else { emsg(_("E389: Couldn't find pattern")); }
  }
  if (action == ACTION_SHOW || action == ACTION_SHOW_ALL) { msg_end(); }
  st->files = files; st->max_path_depth = max_path_depth; st->old_files = old_files;
  st->did_show = did_show; st->found = found;
}
void nvim_fpip_cleanup(void *handle)
{
  FpipState *st = (FpipState *)handle;
  xfree(st->file_line);
  vim_regfree(st->regmatch.regprog); vim_regfree(st->incl_regmatch.regprog); vim_regfree(st->def_regmatch.regprog);
  xfree(st);
}

static void show_pat_in_path(char *line, int type, bool did_show, int action, FILE *fp,
                             linenr_T *lnum, int count)
  FUNC_ATTR_NONNULL_ARG(1, 6)
{
  if (did_show) { msg_putchar('\n'); } else if (!msg_silent) { gotocmdline(true); }
  if (got_int) { return; }
  size_t linelen = strlen(line);
  while (true) {
    char *p = line + linelen - 1;
    if (fp != NULL) {
      if (p >= line && *p == '\n') { p--; }
      if (p >= line && *p == '\r') { p--; }
      *(p + 1) = NUL;
    }
    if (action == ACTION_SHOW_ALL) {
      snprintf(IObuff, IOSIZE, "%3d: ", count); msg_puts(IObuff);
      snprintf(IObuff, IOSIZE, "%4" PRIdLINENR, *lnum);
      msg_puts_hl(IObuff, HLF_N, false); msg_puts(" ");
    }
    msg_prt_line(line, false);
    if (got_int || type != FIND_DEFINE || p < line || *p != '\\') { break; }
    if (fp != NULL) {
      if (vim_fgets(line, LSIZE, fp)) { break; }
      linelen = strlen(line); (*lnum)++;
    } else {
      if (++*lnum > curbuf->b_ml.ml_line_count) { break; }
      line = ml_get(*lnum); linelen = (size_t)ml_get_len(*lnum);
    }
    msg_putchar('\n');
  }
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
void nvim_regmmatch_free(void *rm) { xfree(rm); }
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
proftime_T nvim_profile_setlimit_ms(int timeout) { return profile_setlimit(timeout); }
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
