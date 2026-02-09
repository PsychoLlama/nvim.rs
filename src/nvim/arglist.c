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
int nvim_al_magic_isset(void) { return magic_isset(); }
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
void ex_args(exarg_T *eap)
{
  if (eap->cmdidx != CMD_args) {
    if (check_arglist_locked() == FAIL) {
      return;
    }
    alist_unlink(ALIST(curwin));
    if (eap->cmdidx == CMD_argglobal) {
      ALIST(curwin) = &global_alist;
    } else {     // eap->cmdidx == CMD_arglocal
      alist_new();
    }
  }

  // ":args file ..": define new argument list, handle like ":next"
  // Also for ":argslocal file .." and ":argsglobal file ..".
  if (*eap->arg != NUL) {
    if (check_arglist_locked() == FAIL) {
      return;
    }
    ex_next(eap);
    return;
  }

  // ":args": list arguments.
  if (eap->cmdidx == CMD_args) {
    if (ARGCOUNT <= 0) {
      return;  // empty argument list
    }

    char **items = xmalloc(sizeof(char *) * (size_t)ARGCOUNT);

    // Overwrite the command, for a short list there is no scrolling
    // required and no wait_return().
    gotocmdline(true);

    for (int i = 0; i < ARGCOUNT; i++) {
      items[i] = alist_name(&ARGLIST[i]);
    }
    list_in_columns(items, ARGCOUNT, curwin->w_arg_idx);
    xfree(items);

    return;
  }

  // ":argslocal": make a local copy of the global argument list.
  if (eap->cmdidx == CMD_arglocal) {
    garray_T *gap = &curwin->w_alist->al_ga;

    ga_grow(gap, GARGCOUNT);

    for (int i = 0; i < GARGCOUNT; i++) {
      if (GARGLIST[i].ae_fname != NULL) {
        AARGLIST(curwin->w_alist)[gap->ga_len].ae_fname = xstrdup(GARGLIST[i].ae_fname);
        AARGLIST(curwin->w_alist)[gap->ga_len].ae_fnum = GARGLIST[i].ae_fnum;
        gap->ga_len++;
      }
    }
  }
}

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
void ex_argedit(exarg_T *eap)
{
  int i = eap->addr_count ? (int)eap->line2 : curwin->w_arg_idx + 1;
  // Whether curbuf will be reused, curbuf->b_ffname will be set.
  bool curbuf_is_reusable = curbuf_reusable();

  if (do_arglist(eap->arg, AL_ADD, i, true) == FAIL) {
    return;
  }
  maketitle();

  if (curwin->w_arg_idx == 0
      && (curbuf->b_ml.ml_flags & ML_EMPTY)
      && (curbuf->b_ffname == NULL || curbuf_is_reusable)) {
    i = 0;
  }
  // Edit the argument.
  if (i < ARGCOUNT) {
    do_argfile(eap, i);
  }
}

/// ":argadd"
void ex_argadd(exarg_T *eap)
{
  do_arglist(eap->arg, AL_ADD,
             eap->addr_count > 0 ? (int)eap->line2 : curwin->w_arg_idx + 1,
             false);
  maketitle();
}

/// ":argdelete"
void ex_argdelete(exarg_T *eap)
{
  if (check_arglist_locked() == FAIL) {
    return;
  }

  if (eap->addr_count > 0 || *eap->arg == NUL) {
    // ":argdel" works like ":.argdel"
    if (eap->addr_count == 0) {
      if (curwin->w_arg_idx >= ARGCOUNT) {
        emsg(_("E610: No argument to delete"));
        return;
      }
      eap->line1 = eap->line2 = curwin->w_arg_idx + 1;
    } else if (eap->line2 > ARGCOUNT) {
      // ":1,4argdel": Delete all arguments in the range.
      eap->line2 = ARGCOUNT;
    }
    linenr_T n = eap->line2 - eap->line1 + 1;
    if (*eap->arg != NUL) {
      // Can't have both a range and an argument.
      emsg(_(e_invarg));
    } else if (n <= 0) {
      // Don't give an error for ":%argdel" if the list is empty.
      if (eap->line1 != 1 || eap->line2 != 0) {
        emsg(_(e_invrange));
      }
    } else {
      for (linenr_T i = eap->line1; i <= eap->line2; i++) {
        xfree(ARGLIST[i - 1].ae_fname);
      }
      memmove(ARGLIST + eap->line1 - 1, ARGLIST + eap->line2,
              (size_t)(ARGCOUNT - eap->line2) * sizeof(aentry_T));
      ALIST(curwin)->al_ga.ga_len -= (int)n;
      if (curwin->w_arg_idx >= eap->line2) {
        curwin->w_arg_idx -= (int)n;
      } else if (curwin->w_arg_idx > eap->line1) {
        curwin->w_arg_idx = (int)eap->line1;
      }
      if (ARGCOUNT == 0) {
        curwin->w_arg_idx = 0;
      } else if (curwin->w_arg_idx >= ARGCOUNT) {
        curwin->w_arg_idx = ARGCOUNT - 1;
      }
    }
  } else {
    do_arglist(eap->arg, AL_DEL, 0, false);
  }
  maketitle();
}

/// Function given to ExpandGeneric() to obtain the possible arguments of the
/// argedit and argdelete commands.
char *get_arglist_name(expand_T *xp FUNC_ATTR_UNUSED, int idx)
{
  return rs_get_arglist_name(xp, idx);
}

/// Get the file name for an argument list entry.
char *alist_name(aentry_T *aep) { return rs_alist_name(aep); }

/// Close all the windows containing files which are not in the argument list.
/// Used by the ":all" command.
static void arg_all_close_unused_windows(arg_all_state_T *aall)
{
  win_T *old_curwin = curwin;
  tabpage_T *old_curtab = curtab;

  if (aall->had_tab > 0) {
    goto_tabpage_tp(first_tabpage, true, true);
  }

  // moving tabpages around in an autocommand may cause an endless loop
  tabpage_move_disallowed++;
  while (true) {
    win_T *wpnext = NULL;
    tabpage_T *tpnext = curtab->tp_next;
    // Try to close floating windows first
    for (win_T *wp = lastwin->w_floating ? lastwin : firstwin; wp != NULL; wp = wpnext) {
      int i;
      wpnext = wp->w_floating
               ? wp->w_prev->w_floating ? wp->w_prev : firstwin
               : (wp->w_next == NULL || wp->w_next->w_floating) ? NULL : wp->w_next;
      buf_T *buf = wp->w_buffer;
      if (buf->b_ffname == NULL
          || (!aall->keep_tabs
              && (buf->b_nwindows > 1 || wp->w_width != Columns
                  || (wp->w_floating && !is_aucmd_win(wp))))) {
        i = aall->opened_len;
      } else {
        // check if the buffer in this window is in the arglist
        for (i = 0; i < aall->opened_len; i++) {
          if (i < aall->alist->al_ga.ga_len
              && (AARGLIST(aall->alist)[i].ae_fnum == buf->b_fnum
                  || path_full_compare(alist_name(&AARGLIST(aall->alist)[i]),
                                       buf->b_ffname,
                                       true, true) & kEqualFiles)) {
            int weight = 1;

            if (old_curtab == curtab) {
              weight++;
              if (old_curwin == wp) {
                weight++;
              }
            }

            if (weight > (int)aall->opened[i]) {
              aall->opened[i] = (uint8_t)weight;
              if (i == 0) {
                if (aall->new_curwin != NULL) {
                  aall->new_curwin->w_arg_idx = aall->opened_len;
                }
                aall->new_curwin = wp;
                aall->new_curtab = curtab;
              }
            } else if (aall->keep_tabs) {
              i = aall->opened_len;
            }

            if (wp->w_alist != aall->alist) {
              // Use the current argument list for all windows
              // containing a file from it.
              alist_unlink(wp->w_alist);
              wp->w_alist = aall->alist;
              wp->w_alist->al_refcount++;
            }
            break;
          }
        }
      }
      wp->w_arg_idx = i;

      if (i == aall->opened_len && !aall->keep_tabs) {  // close this window
        if (buf_hide(buf) || aall->forceit || buf->b_nwindows > 1
            || !bufIsChanged(buf)) {
          // If the buffer was changed, and we would like to hide it, try autowriting.
          if (!buf_hide(buf) && buf->b_nwindows <= 1 && bufIsChanged(buf)) {
            bufref_T bufref;
            set_bufref(&bufref, buf);
            autowrite(buf, false);
            // Check if autocommands removed the window.
            if (!win_valid(wp) || !bufref_valid(&bufref)) {
              wpnext = lastwin->w_floating ? lastwin : firstwin;  // Start all over...
              continue;
            }
          }
          // don't close last window
          if (ONE_WINDOW
              && (first_tabpage->tp_next == NULL || !aall->had_tab)) {
            aall->use_firstwin = true;
          } else {
            win_close(wp, !buf_hide(buf) && !bufIsChanged(buf), false);
            // check if autocommands removed the next window
            if (!win_valid(wpnext)) {
              // start all over...
              wpnext = lastwin->w_floating ? lastwin : firstwin;
            }
          }
        }
      }
    }

    // Without the ":tab" modifier only do the current tab page.
    if (aall->had_tab == 0 || tpnext == NULL) {
      break;
    }

    // check if autocommands removed the next tab page
    if (!valid_tabpage(tpnext)) {
      tpnext = first_tabpage;           // start all over...
    }
    goto_tabpage_tp(tpnext, true, true);
  }
  tabpage_move_disallowed--;
}

/// Open up to "count" windows for the files in the argument list "aall->alist".
static void arg_all_open_windows(arg_all_state_T *aall, int count)
{
  bool tab_drop_empty_window = false;

  // ":tab drop file" should re-use an empty window to avoid "--remote-tab"
  // leaving an empty tab page when executed locally.
  if (aall->keep_tabs && buf_is_empty(curbuf) && curbuf->b_nwindows == 1
      && curbuf->b_ffname == NULL && !curbuf->b_changed) {
    aall->use_firstwin = true;
    tab_drop_empty_window = true;
  }

  int split_ret = OK;

  for (int i = 0; i < count && !got_int; i++) {
    if (aall->alist == &global_alist && i == global_alist.al_ga.ga_len - 1) {
      arg_had_last = true;
    }
    if (aall->opened[i] > 0) {
      // Move the already present window to below the current window
      if (curwin->w_arg_idx != i) {
        FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
          if (wp->w_arg_idx == i) {
            if (aall->keep_tabs) {
              aall->new_curwin = wp;
              aall->new_curtab = curtab;
            } else if (wp->w_floating) {
              break;
            } else if (wp->w_frame->fr_parent != curwin->w_frame->fr_parent) {
              emsg(_(e_window_layout_changed_unexpectedly));
              i = count;
              break;
            } else {
              win_move_after(wp, curwin);
            }
            break;
          }
        }
      }
    } else if (split_ret == OK) {
      // trigger events for tab drop
      if (tab_drop_empty_window && i == count - 1) {
        autocmd_no_enter--;
      }
      if (!aall->use_firstwin) {        // split current window
        bool p_ea_save = p_ea;
        p_ea = true;                    // use space from all windows
        split_ret = win_split(0, WSP_ROOM | WSP_BELOW);
        p_ea = p_ea_save;
        if (split_ret == FAIL) {
          continue;
        }
      } else {      // first window: do autocmd for leaving this buffer
        autocmd_no_leave--;
      }

      // edit file "i"
      curwin->w_arg_idx = i;
      if (i == 0) {
        aall->new_curwin = curwin;
        aall->new_curtab = curtab;
      }
      do_ecmd(0, alist_name(&AARGLIST(aall->alist)[i]), NULL, NULL, ECMD_ONE,
              ((buf_hide(curwin->w_buffer)
                || bufIsChanged(curwin->w_buffer)) ? ECMD_HIDE : 0) + ECMD_OLDBUF,
              curwin);
      if (tab_drop_empty_window && i == count - 1) {
        autocmd_no_enter++;
      }
      if (aall->use_firstwin) {
        autocmd_no_leave++;
      }
      aall->use_firstwin = false;
    }
    os_breakcheck();

    // When ":tab" was used open a new tab for a new window repeatedly.
    if (aall->had_tab > 0 && tabpage_index(NULL) <= p_tpm) {
      cmdmod.cmod_tab = 9999;
    }
  }
}

/// do_arg_all(): Open up to 'count' windows, one for each argument.
///
/// @param forceit    hide buffers in current windows
/// @param keep_tabs  keep current tabs, for ":tab drop file"
static void do_arg_all(int count, int forceit, int keep_tabs)
{
  win_T *last_curwin;
  tabpage_T *last_curtab;
  bool prev_arglist_locked = arglist_locked;

  assert(firstwin != NULL);  // satisfy coverity

  if (cmdwin_type != 0) {
    emsg(_(e_cmdwin));
    return;
  }
  if (ARGCOUNT <= 0) {
    // Don't give an error message.  We don't want it when the ":all"
    // command is in the .vimrc.
    return;
  }
  setpcmark();

  arg_all_state_T aall = {
    .use_firstwin = false,
    .had_tab = cmdmod.cmod_tab,
    .new_curwin = NULL,
    .new_curtab = NULL,
    .forceit = forceit,
    .keep_tabs = keep_tabs,
    .opened_len = ARGCOUNT,
    .opened = xcalloc((size_t)ARGCOUNT, 1),
  };

  // Autocommands may do anything to the argument list.  Make sure it's not
  // freed while we are working here by "locking" it.  We still have to
  // watch out for its size to be changed.
  aall.alist = curwin->w_alist;
  aall.alist->al_refcount++;
  arglist_locked = true;

  tabpage_T *const new_lu_tp = curtab;

  // Stop Visual mode, the cursor and "VIsual" may very well be invalid after
  // switching to another buffer.
  reset_VIsual_and_resel();

  // Try closing all windows that are not in the argument list.
  // Also close windows that are not full width;
  // When 'hidden' or "forceit" set the buffer becomes hidden.
  // Windows that have a changed buffer and can't be hidden won't be closed.
  // When the ":tab" modifier was used do this for all tab pages.
  arg_all_close_unused_windows(&aall);

  // Open a window for files in the argument list that don't have one.
  // ARGCOUNT may change while doing this, because of autocommands.
  if (count > aall.opened_len || count <= 0) {
    count = aall.opened_len;
  }

  // Don't execute Win/Buf Enter/Leave autocommands here.
  autocmd_no_enter++;
  autocmd_no_leave++;
  last_curwin = curwin;
  last_curtab = curtab;
  // lastwin may be aucmd_win
  win_enter(lastwin_nofloating(), false);

  // Open up to "count" windows.
  arg_all_open_windows(&aall, count);

  // Remove the "lock" on the argument list.
  alist_unlink(aall.alist);
  arglist_locked = prev_arglist_locked;

  autocmd_no_enter--;

  // restore last referenced tabpage's curwin
  if (last_curtab != aall.new_curtab) {
    if (valid_tabpage(last_curtab)) {
      goto_tabpage_tp(last_curtab, true, true);
    }
    if (win_valid(last_curwin)) {
      win_enter(last_curwin, false);
    }
  }
  // to window with first arg
  if (valid_tabpage(aall.new_curtab)) {
    goto_tabpage_tp(aall.new_curtab, true, true);
  }

  // Now set the last used tabpage to where we started.
  if (valid_tabpage(new_lu_tp)) {
    lastused_tabpage = new_lu_tp;
  }

  if (win_valid(aall.new_curwin)) {
    win_enter(aall.new_curwin, false);
  }

  autocmd_no_leave--;
  xfree(aall.opened);
}

/// ":all" and ":sall".
/// Also used for ":tab drop file ..." after setting the argument list.
void ex_all(exarg_T *eap)
{
  if (eap->addr_count == 0) {
    eap->line2 = 9999;
  }
  do_arg_all((int)eap->line2, eap->forceit, eap->cmdidx == CMD_drop);
}

/// Concatenate all files in the argument list, separated by spaces, and return
/// it in one allocated string.
/// Spaces and backslashes in the file names are escaped with a backslash.
char *arg_all(void) { return rs_arg_all(); }

/// "argc([window id])" function
void f_argc(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  if (argvars[0].v_type == VAR_UNKNOWN) {
    // use the current window
    rettv->vval.v_number = ARGCOUNT;
  } else if (argvars[0].v_type == VAR_NUMBER
             && tv_get_number(&argvars[0]) == -1) {
    // use the global argument list
    rettv->vval.v_number = GARGCOUNT;
  } else {
    // use the argument list of the specified window
    win_T *wp = find_win_by_nr_or_id(&argvars[0]);
    if (wp != NULL) {
      rettv->vval.v_number = WARGCOUNT(wp);
    } else {
      rettv->vval.v_number = -1;
    }
  }
}

/// "argidx()" function
void f_argidx(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rettv->vval.v_number = curwin->w_arg_idx;
}

/// "arglistid()" function
void f_arglistid(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rettv->vval.v_number = -1;
  win_T *wp = find_tabwin(&argvars[0], &argvars[1]);
  if (wp != NULL) {
    rettv->vval.v_number = wp->w_alist->id;
  }
}

/// Get the argument list for a given window
static void get_arglist_as_rettv(aentry_T *arglist, int argcount, typval_T *rettv)
{
  tv_list_alloc_ret(rettv, argcount);
  if (arglist != NULL) {
    for (int idx = 0; idx < argcount; idx++) {
      tv_list_append_string(rettv->vval.v_list, alist_name(&arglist[idx]), -1);
    }
  }
}

/// "argv(nr)" function
void f_argv(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  aentry_T *arglist = NULL;
  int argcount = -1;

  if (argvars[0].v_type == VAR_UNKNOWN) {
    get_arglist_as_rettv(ARGLIST, ARGCOUNT, rettv);
    return;
  }

  if (argvars[1].v_type == VAR_UNKNOWN) {
    arglist = ARGLIST;
    argcount = ARGCOUNT;
  } else if (argvars[1].v_type == VAR_NUMBER
             && tv_get_number(&argvars[1]) == -1) {
    arglist = GARGLIST;
    argcount = GARGCOUNT;
  } else {
    win_T *wp = find_win_by_nr_or_id(&argvars[1]);
    if (wp != NULL) {
      // Use the argument list of the specified window
      arglist = WARGLIST(wp);
      argcount = WARGCOUNT(wp);
    }
  }

  rettv->v_type = VAR_STRING;
  rettv->vval.v_string = NULL;
  int idx = (int)tv_get_number_chk(&argvars[0], NULL);
  if (arglist != NULL && idx >= 0 && idx < argcount) {
    rettv->vval.v_string = xstrdup(alist_name(&arglist[idx]));
  } else if (idx == -1) {
    get_arglist_as_rettv(arglist, argcount, rettv);
  }
}
