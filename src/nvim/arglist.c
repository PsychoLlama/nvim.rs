// arglist.c: functions for dealing with the argument list

#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <string.h>

#include "auto/config.h"
#include "nvim/arglist.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/errors.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/window.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds2.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_getln.h"
#include "nvim/fileio.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/macros_defs.h"
#include "nvim/mark.h"
#include "nvim/memline_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/normal.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/os/input.h"
#include "nvim/path.h"
#include "nvim/pos_defs.h"
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
#include "nvim/types_defs.h"
#include "nvim/undo.h"
#include "nvim/version.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

/// State used by the :all command to open all the files in the argument list in
/// separate windows.
typedef struct {
  alist_T *alist;     ///< argument list to be used
  int had_tab;
  bool keep_tabs;
  bool forceit;

  bool use_firstwin;  ///< use first window for arglist
  uint8_t *opened;    ///< Array of weight for which args are open:
                      ///<  0: not opened
                      ///<  1: opened in other tab
                      ///<  2: opened in curtab
                      ///<  3: opened in curtab and curwin
  int opened_len;     ///< length of opened[]
  win_T *new_curwin;
  tabpage_T *new_curtab;
} arg_all_state_T;

#include "arglist.c.generated.h"

// Rust FFI declarations (window wrappers removed)
extern win_T *rs_lastwin_nofloating(void);
extern int rs_tabpage_index(tabpage_T *ftp);
extern int rs_valid_tabpage(tabpage_T *tpc);
extern int rs_win_valid(win_T *win);

extern int rs_magic_isset(void);

static const char e_window_layout_changed_unexpectedly[]
  = N_("E249: Window layout changed unexpectedly");

enum {
  AL_SET = 1,
  AL_ADD = 2,
  AL_DEL = 3,
};

// Static assertions for Rust FFI constant synchronization
_Static_assert(AL_SET == 1, "AL_SET mismatch");
_Static_assert(AL_ADD == 2, "AL_ADD mismatch");
_Static_assert(AL_DEL == 3, "AL_DEL mismatch");
_Static_assert(OK == 1, "OK mismatch");
_Static_assert(FAIL == 0, "FAIL mismatch");
_Static_assert(NUL == 0, "NUL mismatch");
_Static_assert(BLN_CURBUF == 1, "BLN_CURBUF mismatch");
_Static_assert(BLN_LISTED == 2, "BLN_LISTED mismatch");
_Static_assert(EW_DIR == 0x01, "EW_DIR mismatch");
_Static_assert(EW_FILE == 0x02, "EW_FILE mismatch");
_Static_assert(EW_NOTFOUND == 0x04, "EW_NOTFOUND mismatch");
_Static_assert(EW_ADDSLASH == 0x08, "EW_ADDSLASH mismatch");
_Static_assert(EW_NOERROR == 0x200, "EW_NOERROR mismatch");
_Static_assert(EW_NOTWILD == 0x400, "EW_NOTWILD mismatch");
_Static_assert(RE_MAGIC == 1, "RE_MAGIC mismatch");
_Static_assert(kEqualFiles == 1, "kEqualFiles mismatch");
_Static_assert(CCGD_AW == 1, "CCGD_AW mismatch");
_Static_assert(CCGD_MULTWIN == 2, "CCGD_MULTWIN mismatch");
_Static_assert(CCGD_FORCEIT == 4, "CCGD_FORCEIT mismatch");
_Static_assert(CCGD_EXCMD == 16, "CCGD_EXCMD mismatch");
_Static_assert(ECMD_LAST == -1, "ECMD_LAST mismatch");
_Static_assert(ECMD_HIDE == 0x01, "ECMD_HIDE mismatch");
_Static_assert(ECMD_OLDBUF == 0x04, "ECMD_OLDBUF mismatch");
_Static_assert(ECMD_FORCEIT == 0x08, "ECMD_FORCEIT mismatch");
_Static_assert(ECMD_ONE == 1, "ECMD_ONE mismatch");
_Static_assert(CMD_args == 7, "CMD_args mismatch");
_Static_assert(CMD_argglobal == 13, "CMD_argglobal mismatch");
_Static_assert(CMD_arglocal == 14, "CMD_arglocal mismatch");
_Static_assert(CMD_argdo == 10, "CMD_argdo mismatch");
_Static_assert(CMD_snext == 413, "CMD_snext mismatch");
_Static_assert(CMD_drop == 130, "CMD_drop mismatch");
_Static_assert(WSP_ROOM == 0x01, "WSP_ROOM mismatch");
_Static_assert(WSP_BELOW == 0x40, "WSP_BELOW mismatch");
_Static_assert(VAR_UNKNOWN == 0, "VAR_UNKNOWN mismatch");
_Static_assert(VAR_NUMBER == 1, "VAR_NUMBER mismatch");
_Static_assert(VAR_STRING == 2, "VAR_STRING mismatch");
_Static_assert(ML_EMPTY == 0x01, "ML_EMPTY mismatch");

/// This flag is set whenever the argument list is being changed and calling a
/// function that might trigger an autocommand.
static bool arglist_locked = false;

// =============================================================================
// C accessor functions for Rust FFI
// =============================================================================

// -- Globals --
int nvim_al_get_arglist_locked(void) { return arglist_locked; }
void nvim_al_set_arglist_locked(int val) { arglist_locked = val; }
alist_T *nvim_al_get_global_alist(void) { return &global_alist; }
int nvim_al_get_arg_had_last(void) { return arg_had_last; }
void nvim_al_set_arg_had_last(int val) { arg_had_last = val; }
int nvim_al_get_max_alist_id(void) { return max_alist_id; }
int nvim_al_inc_max_alist_id(void) { return ++max_alist_id; }
win_T *nvim_al_get_curwin(void) { return curwin; }
buf_T *nvim_al_get_curbuf(void) { return curbuf; }
tabpage_T *nvim_al_get_curtab(void) { return curtab; }
int nvim_al_get_got_int(void) { return got_int; }

// -- Macros --
int nvim_al_ARGCOUNT(void) { return ARGCOUNT; }
aentry_T *nvim_al_ARGLIST(void) { return ARGLIST; }
int nvim_al_GARGCOUNT(void) { return GARGCOUNT; }
aentry_T *nvim_al_GARGLIST(void) { return GARGLIST; }
alist_T *nvim_al_ALIST_curwin(void) { return ALIST(curwin); }
int nvim_al_WARGCOUNT(win_T *wp) { return WARGCOUNT(wp); }
aentry_T *nvim_al_WARGLIST(win_T *wp) { return WARGLIST(wp); }
aentry_T *nvim_al_AARGLIST(alist_T *al, int i) { return &AARGLIST(al)[i]; }

// -- alist_T fields --
garray_T *nvim_al_ga_ptr(alist_T *al) { return &al->al_ga; }
int nvim_al_get_refcount(alist_T *al) { return al->al_refcount; }
void nvim_al_set_refcount(alist_T *al, int val) { al->al_refcount = val; }
void nvim_al_inc_refcount(alist_T *al) { al->al_refcount++; }
int nvim_al_dec_refcount(alist_T *al) { return --al->al_refcount; }
int nvim_al_get_id(alist_T *al) { return al->id; }
void nvim_al_set_id(alist_T *al, int val) { al->id = val; }

// -- aentry_T fields --
char *nvim_al_ae_get_fname(aentry_T *ae) { return ae->ae_fname; }
void nvim_al_ae_set_fname(aentry_T *ae, char *fname) { ae->ae_fname = fname; }
int nvim_al_ae_get_fnum(aentry_T *ae) { return ae->ae_fnum; }
void nvim_al_ae_set_fnum(aentry_T *ae, int fnum) { ae->ae_fnum = fnum; }

// -- garray_T ops --
int nvim_al_ga_get_len(garray_T *ga) { return ga->ga_len; }
void nvim_al_ga_set_len(garray_T *ga, int len) { ga->ga_len = len; }
void *nvim_al_ga_get_data(garray_T *ga) { return ga->ga_data; }
void nvim_al_ga_init_aentry(garray_T *ga) { ga_init(ga, (int)sizeof(aentry_T), 5); }
void nvim_al_ga_grow(garray_T *ga, int n) { ga_grow(ga, n); }
void nvim_al_ga_clear(garray_T *ga) { ga_clear(ga); }

// -- curwin fields --
int nvim_al_win_get_arg_idx(win_T *wp) { return wp->w_arg_idx; }
void nvim_al_win_set_arg_idx(win_T *wp, int idx) { wp->w_arg_idx = idx; }
alist_T *nvim_al_win_get_alist(win_T *wp) { return wp->w_alist; }
void nvim_al_win_set_alist(win_T *wp, alist_T *al) { wp->w_alist = al; }
void nvim_al_win_set_locked(win_T *wp, int val) { wp->w_locked = val; }

// -- Phase 2 extra accessors --
void nvim_al_emsg_arglist_locked(void) { emsg(_(e_cannot_change_arglist_recursively)); }
void nvim_al_xfree(void *ptr) { xfree(ptr); }
void *nvim_al_xmalloc(size_t size) { return xmalloc(size); }
char *nvim_al_xstrdup(const char *s) { return xstrdup(s); }
void nvim_al_deep_clear_aentry(alist_T *al)
{
#define FREE_AENTRY_FNAME(arg) xfree((arg)->ae_fname)
  GA_DEEP_CLEAR(&al->al_ga, aentry_T, FREE_AENTRY_FNAME);
#undef FREE_AENTRY_FNAME
}
int nvim_al_buflist_add(const char *fname, int flags)
{
  return buflist_add((char *)fname, flags);
}
void nvim_al_buf_set_name(int fnum, const char *name)
{
  buf_set_name(fnum, (char *)name);
}
void nvim_al_os_breakcheck(void) { os_breakcheck(); }

// -- Phase 3 extra accessors --
int nvim_al_rem_backslash(const char *p) { return rem_backslash(p); }
int nvim_al_ascii_isspace(int c) { return ascii_isspace(c); }
char *nvim_al_skipwhite(const char *p) { return skipwhite(p); }
int nvim_al_expand_wildcards(int num_pat, char **pat, int *num_files, char ***files, int flags)
{
  return expand_wildcards(num_pat, pat, num_files, files, flags);
}
int nvim_al_gen_expand_wildcards(int num_pat, char **pat, int *num_files, char ***files, int flags)
{
  return gen_expand_wildcards(num_pat, pat, num_files, files, flags);
}
void nvim_al_ga_init_charptr(garray_T *ga) { ga_init(ga, (int)sizeof(char *), 20); }
void nvim_al_ga_append_charptr(garray_T *ga, char *ptr) { GA_APPEND(char *, ga, ptr); }
garray_T *nvim_al_alloc_garray(void) { return xcalloc(1, sizeof(garray_T)); }
void nvim_al_free_garray(garray_T *ga) { xfree(ga); }

alist_T *nvim_al_alloc_alist(void)
{
  return xmalloc(sizeof(alist_T));
}

// -- Phase 4 extra accessors --

// Callback-based iteration over FOR_ALL_TAB_WINDOWS
// callback(wp, userdata) is called for each window; if it returns non-zero, stop.
void nvim_al_foreach_tab_window(int (*callback)(win_T *wp, void *ud), void *ud)
{
  FOR_ALL_TAB_WINDOWS(tp, wp) {
    if (callback(wp, ud)) {
      return;
    }
  }
}

// Opaque regex wrappers: allocate regmatch_T on C heap, compile, execute, free.
void *nvim_al_regmatch_alloc(void)
{
  return xcalloc(1, sizeof(regmatch_T));
}
void nvim_al_regmatch_set_ic(void *rm, int ic)
{
  ((regmatch_T *)rm)->rm_ic = ic;
}
int nvim_al_regmatch_compile(void *rm, const char *pat, int re_flags)
{
  regmatch_T *rmp = (regmatch_T *)rm;
  rmp->regprog = vim_regcomp(pat, re_flags);
  return rmp->regprog != NULL;
}
int nvim_al_regmatch_exec(void *rm, const char *line)
{
  return vim_regexec((regmatch_T *)rm, line, 0);
}
void nvim_al_regmatch_free(void *rm)
{
  regmatch_T *rmp = (regmatch_T *)rm;
  vim_regfree(rmp->regprog);
  xfree(rmp);
}
void nvim_al_regmatch_free_prog(void *rm)
{
  regmatch_T *rmp = (regmatch_T *)rm;
  vim_regfree(rmp->regprog);
  rmp->regprog = NULL;
}

char *nvim_al_file_pat_to_reg_pat(const char *pat)
{
  return file_pat_to_reg_pat(pat, NULL, NULL, false);
}
int nvim_al_magic_isset(void) { return rs_magic_isset(); }
int nvim_al_get_p_fic(void) { return p_fic; }
void nvim_al_semsg_nomatch2(const char *s) { semsg(_(e_nomatch2), s); }
void nvim_al_emsg_nomatch(void) { emsg(_(e_nomatch)); }
char *nvim_al_alist_name(aentry_T *ae) { return alist_name(ae); }
void nvim_al_check_arg_idx(win_T *wp) { check_arg_idx(wp); }
char *nvim_al_curbuf_b_ffname(void) { return curbuf->b_ffname; }
char *nvim_al_curbuf_b_fname(void) { return curbuf->b_fname; }
void nvim_al_memmove_aentry(aentry_T *dst, const aentry_T *src, int count)
{
  memmove(dst, src, (size_t)count * sizeof(aentry_T));
}

// -- Phase 5 extra accessors --
buf_T *nvim_al_buflist_findnr(int fnum) { return buflist_findnr(fnum); }
char *nvim_al_buf_get_fname(buf_T *buf) { return buf == NULL ? NULL : buf->b_fname; }
char *nvim_al_buf_get_ffname(buf_T *buf) { return buf == NULL ? NULL : buf->b_ffname; }
int nvim_al_buf_get_fnum(buf_T *buf) { return buf == NULL ? 0 : buf->b_fnum; }
int nvim_al_path_full_compare(const char *s1, const char *s2, int check_name, int expand_env)
{
  return path_full_compare(s1, s2, check_name, expand_env);
}
buf_T *nvim_al_win_get_buffer(win_T *wp) { return wp->w_buffer; }
void nvim_al_win_set_arg_idx_invalid(win_T *wp, int val) { wp->w_arg_idx_invalid = val; }

// -- Phase 6 extra accessors --
char *nvim_al_eap_get_cmd(exarg_T *eap) { return eap->cmd; }
char *nvim_al_eap_get_arg(exarg_T *eap) { return eap->arg; }
linenr_T nvim_al_eap_get_line1(exarg_T *eap) { return eap->line1; }
linenr_T nvim_al_eap_get_line2(exarg_T *eap) { return eap->line2; }
int nvim_al_eap_get_addr_count(exarg_T *eap) { return eap->addr_count; }
int nvim_al_eap_get_forceit(exarg_T *eap) { return eap->forceit; }
int nvim_al_eap_get_cmdidx(exarg_T *eap) { return (int)eap->cmdidx; }
void nvim_al_eap_set_line1(exarg_T *eap, linenr_T val) { eap->line1 = val; }
void nvim_al_eap_set_line2(exarg_T *eap, linenr_T val) { eap->line2 = val; }
int nvim_al_check_can_set_curbuf_forceit(int forceit) { return check_can_set_curbuf_forceit(forceit); }
void nvim_al_setpcmark(void) { setpcmark(); }
int nvim_al_win_split(int size, int flags) { return win_split(size, flags); }
void nvim_al_reset_binding(win_T *wp) { RESET_BINDING(wp); }
int nvim_al_buf_hide(buf_T *buf) { return buf_hide(buf); }
char *nvim_al_fix_fname(const char *fname) { return fix_fname(fname); }
int nvim_al_otherfile(const char *fname) { return otherfile((char *)fname); }
int nvim_al_check_changed(buf_T *buf, int flags) { return check_changed(buf, flags); }
int nvim_al_do_ecmd(int fnum, const char *ffname, const char *sfname, exarg_T *eap,
                    linenr_T newlnum, int flags, win_T *oldwin)
{
  return do_ecmd(fnum, (char *)ffname, (char *)sfname, eap, newlnum, flags, oldwin);
}
void nvim_al_setmark(int c) { setmark(c); }
char *nvim_al_FullName_save(const char *fname, int force) { return FullName_save(fname, force); }
int nvim_al_path_fnamecmp(const char *s1, const char *s2) { return path_fnamecmp(s1, s2); }
int nvim_al_get_cmdmod_cmod_tab(void) { return cmdmod.cmod_tab; }
void nvim_al_emsg_E163(void) { emsg(_("E163: There is only one file to edit")); }
void nvim_al_emsg_E164(void) { emsg(_("E164: Cannot go before first file")); }
void nvim_al_emsg_E165(void) { emsg(_("E165: Cannot go beyond last file")); }

// -- Phase 7 extra accessors --
void nvim_al_gotocmdline(int clr) { gotocmdline(clr); }
void nvim_al_list_in_columns(char **items, int count, int current) { list_in_columns(items, count, current); }
void nvim_al_maketitle(void) { maketitle(); }
int nvim_al_curbuf_reusable(void) { return curbuf_reusable(); }
int nvim_al_curbuf_ml_empty(void) { return (curbuf->b_ml.ml_flags & ML_EMPTY) != 0; }
void nvim_al_emsg_invarg(void) { emsg(_(e_invarg)); }
void nvim_al_emsg_invrange(void) { emsg(_(e_invrange)); }
void nvim_al_emsg_E610(void) { emsg(_("E610: No argument to delete")); }

// -- Phase 8 extra accessors --
win_T *nvim_al_get_firstwin(void) { return firstwin; }
win_T *nvim_al_get_lastwin(void) { return lastwin; }
tabpage_T *nvim_al_get_first_tabpage(void) { return first_tabpage; }
void nvim_al_goto_tabpage_tp(tabpage_T *tp, int trigger_enter, int trigger_leave)
{
  goto_tabpage_tp(tp, trigger_enter, trigger_leave);
}
int nvim_al_valid_tabpage(tabpage_T *tp) { return rs_valid_tabpage(tp); }
int nvim_al_win_valid(win_T *wp) { return rs_win_valid(wp); }
void nvim_al_win_close(win_T *wp, int free_buf, int force) { win_close(wp, free_buf, force); }
void nvim_al_win_enter(win_T *wp, int undo_sync) { win_enter(wp, undo_sync); }
void nvim_al_win_move_after(win_T *wp, win_T *after) { win_move_after(wp, after); }
win_T *nvim_al_lastwin_nofloating(void) { return rs_lastwin_nofloating(); }
int nvim_al_win_is_floating(win_T *wp) { return wp->w_floating; }
win_T *nvim_al_win_get_prev(win_T *wp) { return wp->w_prev; }
win_T *nvim_al_win_get_next(win_T *wp) { return wp->w_next; }
int nvim_al_win_get_width(win_T *wp) { return wp->w_width; }
void *nvim_al_win_get_frame_parent(win_T *wp) { return wp->w_frame->fr_parent; }
int nvim_al_get_Columns(void) { return Columns; }
int nvim_al_buf_get_nwindows(buf_T *buf) { return buf->b_nwindows; }
int nvim_al_bufIsChanged(buf_T *buf) { return bufIsChanged(buf); }
int nvim_al_buf_is_empty(buf_T *buf) { return buf_is_empty(buf); }
int nvim_al_autowrite(buf_T *buf, int eap_forceit) { return autowrite(buf, eap_forceit); }
void *nvim_al_bufref_create(buf_T *buf)
{
  bufref_T *br = xcalloc(1, sizeof(bufref_T));
  set_bufref(br, buf);
  return br;
}
int nvim_al_bufref_valid(void *br) { return bufref_valid((bufref_T *)br); }
void nvim_al_bufref_destroy(void *br) { xfree(br); }
void nvim_al_set_bufref(void *br, buf_T *buf) { set_bufref((bufref_T *)br, buf); }
int nvim_al_ONE_WINDOW(void) { return ONE_WINDOW; }
int nvim_al_is_aucmd_win(win_T *wp) { return is_aucmd_win(wp); }
extern void rs_reset_VIsual_and_resel(void);
void nvim_al_reset_VIsual_and_resel(void) { rs_reset_VIsual_and_resel(); }
void *nvim_al_xcalloc(size_t count, size_t size) { return xcalloc(count, size); }
int nvim_al_tabpage_index(tabpage_T *tp) { return rs_tabpage_index(tp); }
int nvim_al_get_p_tpm(void) { return p_tpm; }
int nvim_al_get_p_ea(void) { return p_ea; }
void nvim_al_set_p_ea(int val) { p_ea = val; }
void nvim_al_set_cmdmod_cmod_tab(int val) { cmdmod.cmod_tab = val; }
int nvim_al_get_cmdwin_type(void) { return cmdwin_type; }
int nvim_al_get_autocmd_no_enter(void) { return autocmd_no_enter; }
void nvim_al_set_autocmd_no_enter(int val) { autocmd_no_enter = val; }
int nvim_al_get_autocmd_no_leave(void) { return autocmd_no_leave; }
void nvim_al_set_autocmd_no_leave(int val) { autocmd_no_leave = val; }
int nvim_al_get_tabpage_move_disallowed(void) { return tabpage_move_disallowed; }
void nvim_al_set_tabpage_move_disallowed(int val) { tabpage_move_disallowed = val; }
tabpage_T *nvim_al_tp_get_next(tabpage_T *tp) { return tp->tp_next; }
int nvim_al_buf_get_changed(buf_T *buf) { return buf->b_changed; }
void nvim_al_set_lastused_tabpage(tabpage_T *tp) { lastused_tabpage = tp; }
void nvim_al_emsg_e_cmdwin(void) { emsg(_(e_cmdwin)); }
void nvim_al_emsg_e_window_layout(void) { emsg(_(e_window_layout_changed_unexpectedly)); }

// -- Phase 9 extra accessors (VimL functions) --
int nvim_al_tv_get_type(typval_T *tv) { return (int)tv->v_type; }
int64_t nvim_al_tv_get_number(typval_T *tv) { return tv_get_number(tv); }
int64_t nvim_al_tv_get_number_chk(typval_T *tv, int *error)
{
  bool berr = false;
  int64_t result = tv_get_number_chk(tv, error != NULL ? &berr : NULL);
  if (error != NULL) {
    *error = berr;
  }
  return result;
}
void nvim_al_rettv_set_number(typval_T *rettv, int64_t val) { rettv->vval.v_number = val; }
void nvim_al_rettv_set_string(typval_T *rettv, char *s) { rettv->vval.v_string = s; }
void nvim_al_rettv_set_type(typval_T *rettv, int typ) { rettv->v_type = (VarType)typ; }
void nvim_al_tv_list_alloc_ret(typval_T *rettv, int len) { tv_list_alloc_ret(rettv, len); }
void nvim_al_tv_list_append_string(typval_T *rettv, const char *s, int64_t len)
{
  tv_list_append_string(rettv->vval.v_list, s, (ssize_t)len);
}
win_T *nvim_al_find_win_by_nr_or_id(typval_T *tv) { return find_win_by_nr_or_id(tv); }
win_T *nvim_al_find_tabwin(typval_T *tv_win, typval_T *tv_tab) { return find_tabwin(tv_win, tv_tab); }
int nvim_al_win_get_alist_id(win_T *wp) { return wp->w_alist->id; }
typval_T *nvim_al_tv_idx(typval_T *tv, int idx) { return &tv[idx]; }
aentry_T *nvim_al_ae_idx(aentry_T *ae, int idx) { return &ae[idx]; }

// Callback-based iteration over FOR_ALL_WINDOWS_IN_TAB
void nvim_al_foreach_windows_in_tab(int (*callback)(win_T *wp, void *ud), tabpage_T *tp, void *ud)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, tp) {
    if (callback(wp, ud)) {
      return;
    }
  }
}

// Rust FFI declarations for Phase 2
extern int rs_check_arglist_locked(void);
extern void rs_alist_clear(alist_T *al);
extern void rs_alist_init(alist_T *al);
extern void rs_alist_unlink(alist_T *al);
extern void rs_alist_new(void);
extern void rs_alist_add(alist_T *al, char *fname, int set_fnum);
extern void rs_alist_set(alist_T *al, int count, char **files, int use_curbuf, int *fnum_list, int fnum_len);
extern void rs_alist_expand(int *fnum_list, int fnum_len);

// Rust FFI declarations for Phase 3
extern int rs_get_arglist_exp(char *str, int *fcountp, char ***fnamesp, int wig);

// Rust FFI declarations for Phase 4
extern int rs_do_arglist(char *str, int what, int after, int will_edit);
extern void rs_set_arglist(char *str);

// Rust FFI declarations for Phase 5
extern char *rs_alist_name(aentry_T *aep);
extern char *rs_get_arglist_name(void *xp, int idx);
extern bool rs_editing_arg_idx(win_T *win);
extern void rs_check_arg_idx(win_T *win);
extern char *rs_arg_all(void);

// Rust FFI declarations for Phase 6
extern void rs_ex_previous(exarg_T *eap);
extern void rs_ex_rewind(exarg_T *eap);
extern void rs_ex_last(exarg_T *eap);
extern void rs_ex_argument(exarg_T *eap);
extern void rs_do_argfile(exarg_T *eap, int argn);
extern void rs_ex_next(exarg_T *eap);
extern void rs_ex_argdedupe(void);

// Rust FFI declarations for Phase 7
extern void rs_ex_args(exarg_T *eap);
extern void rs_ex_argedit(exarg_T *eap);
extern void rs_ex_argadd(exarg_T *eap);
extern void rs_ex_argdelete(exarg_T *eap);

// Rust FFI declarations for Phase 8
extern void rs_ex_all(exarg_T *eap);

// Rust FFI declarations for Phase 9
extern void rs_f_argc(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void rs_f_argidx(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void rs_f_arglistid(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void rs_f_argv(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

static int check_arglist_locked(void)
{
  return rs_check_arglist_locked();
}

void alist_clear(alist_T *al) { rs_alist_clear(al); }
void alist_init(alist_T *al) { rs_alist_init(al); }
void alist_unlink(alist_T *al) { rs_alist_unlink(al); }
void alist_new(void) { rs_alist_new(); }

void alist_add(alist_T *al, char *fname, int set_fnum) { rs_alist_add(al, fname, set_fnum); }

void alist_set(alist_T *al, int count, char **files, int use_curbuf, int *fnum_list, int fnum_len)
{
  rs_alist_set(al, count, files, use_curbuf, fnum_list, fnum_len);
}

#if !defined(UNIX)
void alist_expand(int *fnum_list, int fnum_len) { rs_alist_expand(fnum_list, fnum_len); }
#endif

#if defined(BACKSLASH_IN_FILENAME)
/// Adjust slashes in file names.  Called after 'shellslash' was set.
/// No-op on Linux — only relevant on Windows.
void alist_slash_adjust(void) {}
#endif

/// Parse a list of arguments (file names), expand them and return in
/// "fnames[fcountp]".  When "wig" is true, removes files matching 'wildignore'.
///
/// @return  FAIL or OK.
int get_arglist_exp(char *str, int *fcountp, char ***fnamesp, bool wig)
{
  return rs_get_arglist_exp(str, fcountp, fnamesp, wig);
}

/// @param str
/// @param what
///         AL_SET: Redefine the argument list to 'str'.
///         AL_ADD: add files in 'str' to the argument list after "after".
///         AL_DEL: remove files in 'str' from the argument list.
/// @param after
///         0 means before first one
/// @param will_edit  will edit added argument
///
/// @return  FAIL for failure, OK otherwise.
static int do_arglist(char *str, int what, int after, bool will_edit)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_do_arglist(str, what, after, will_edit);
}

/// Redefine the argument list.
void set_arglist(char *str) { rs_set_arglist(str); }

/// @return  true if window "win" is editing the file at the current argument
///          index.
bool editing_arg_idx(win_T *win) { return rs_editing_arg_idx(win); }

/// Check if window "win" is editing the w_arg_idx file in its argument list.
void check_arg_idx(win_T *win) { rs_check_arg_idx(win); }

/// ":args", ":arglocal" and ":argglobal".
void ex_args(exarg_T *eap) { rs_ex_args(eap); }

/// ":previous", ":sprevious", ":Next" and ":sNext".
void ex_previous(exarg_T *eap) { rs_ex_previous(eap); }

/// ":rewind", ":first", ":sfirst" and ":srewind".
void ex_rewind(exarg_T *eap) { rs_ex_rewind(eap); }

/// ":last" and ":slast".
void ex_last(exarg_T *eap) { rs_ex_last(eap); }

/// ":argument" and ":sargument".
void ex_argument(exarg_T *eap) { rs_ex_argument(eap); }

/// Edit file "argn" of the argument lists.
void do_argfile(exarg_T *eap, int argn) { rs_do_argfile(eap, argn); }

/// ":next", and commands that behave like it.
void ex_next(exarg_T *eap) { rs_ex_next(eap); }

/// ":argdedupe"
void ex_argdedupe(exarg_T *eap FUNC_ATTR_UNUSED) { rs_ex_argdedupe(); }

/// ":argedit"
void ex_argedit(exarg_T *eap) { rs_ex_argedit(eap); }

/// ":argadd"
void ex_argadd(exarg_T *eap) { rs_ex_argadd(eap); }

/// ":argdelete"
void ex_argdelete(exarg_T *eap) { rs_ex_argdelete(eap); }

/// Function given to ExpandGeneric() to obtain the possible arguments of the
/// argedit and argdelete commands.
char *get_arglist_name(expand_T *xp FUNC_ATTR_UNUSED, int idx)
{
  return rs_get_arglist_name(xp, idx);
}

/// Get the file name for an argument list entry.
char *alist_name(aentry_T *aep) { return rs_alist_name(aep); }

/// ":all" and ":sall".
/// Also used for ":tab drop file ..." after setting the argument list.
void ex_all(exarg_T *eap) { rs_ex_all(eap); }

/// Concatenate all files in the argument list, separated by spaces, and return
/// it in one allocated string.
/// Spaces and backslashes in the file names are escaped with a backslash.
char *arg_all(void) { return rs_arg_all(); }

/// "argc([window id])" function
void f_argc(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rs_f_argc(argvars, rettv, fptr);
}

/// "argidx()" function
void f_argidx(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rs_f_argidx(argvars, rettv, fptr);
}

/// "arglistid()" function
void f_arglistid(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rs_f_arglistid(argvars, rettv, fptr);
}

/// "argv(nr)" function
void f_argv(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rs_f_argv(argvars, rettv, fptr);
}
