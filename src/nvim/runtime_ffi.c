/// runtime_ffi.c: C accessor wrappers for the Rust runtime crate (nvim-runtime).
///
/// These thin wrappers provide a stable C ABI for Rust code to call into
/// Neovim's C internals.  Each function is called from one or more Rust
/// modules in src/nvim-rs/runtime/.

#include <assert.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>

#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/debugger.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/getchar.h"
#include "nvim/lua/executor.h"
#include "nvim/memline.h"
#include "nvim/usercmd.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/vars.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/userfunc.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_eval.h"
#include "nvim/ex_eval_defs.h"
#include "nvim/mbyte_defs.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/globals.h"
#include "nvim/hashtab.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/os/input.h"
#include "nvim/os/fs.h"
#include "nvim/os/os.h"
#include "nvim/option_vars.h"
#include "nvim/mbyte.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/path.h"
#include "nvim/pos_defs.h"
#include "nvim/profile.h"
#include "nvim/regexp.h"
#include "nvim/runtime.h"
#include "nvim/runtime_defs.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"

#include "runtime_ffi.c.generated.h"

extern int rs_get_copyID(void);

_Static_assert(ETYPE_TOP == 0, "ETYPE_TOP must be 0");
_Static_assert(ETYPE_SCRIPT == 1, "ETYPE_SCRIPT must be 1");
_Static_assert(ETYPE_UFUNC == 2, "ETYPE_UFUNC must be 2");
_Static_assert(ETYPE_AUCMD == 3, "ETYPE_AUCMD must be 3");
_Static_assert(ETYPE_MODELINE == 4, "ETYPE_MODELINE must be 4");
_Static_assert(ETYPE_EXCEPT == 5, "ETYPE_EXCEPT must be 5");
_Static_assert(ETYPE_ARGS == 6, "ETYPE_ARGS must be 6");
_Static_assert(ETYPE_ENV == 7, "ETYPE_ENV must be 7");
_Static_assert(ETYPE_INTERNAL == 8, "ETYPE_INTERNAL must be 8");
_Static_assert(ETYPE_SPELL == 9, "ETYPE_SPELL must be 9");

_Static_assert(ESTACK_NONE == 0, "ESTACK_NONE must be 0");
_Static_assert(ESTACK_SFILE == 1, "ESTACK_SFILE must be 1");
_Static_assert(ESTACK_STACK == 2, "ESTACK_STACK must be 2");
_Static_assert(ESTACK_SCRIPT == 3, "ESTACK_SCRIPT must be 3");

_Static_assert(DIP_ALL == 0x01, "DIP_ALL must be 0x01");
_Static_assert(DIP_DIR == 0x02, "DIP_DIR must be 0x02");
_Static_assert(DIP_ERR == 0x04, "DIP_ERR must be 0x04");
_Static_assert(DIP_START == 0x08, "DIP_START must be 0x08");
_Static_assert(DIP_OPT == 0x10, "DIP_OPT must be 0x10");
_Static_assert(DIP_NORTP == 0x20, "DIP_NORTP must be 0x20");
_Static_assert(DIP_NOAFTER == 0x40, "DIP_NOAFTER must be 0x40");
_Static_assert(DIP_AFTER == 0x80, "DIP_AFTER must be 0x80");
_Static_assert(DIP_DIRFILE == 0x200, "DIP_DIRFILE must be 0x200");

_Static_assert(DOSO_NONE == 0, "DOSO_NONE must be 0");
_Static_assert(DOSO_VIMRC == 1, "DOSO_VIMRC must be 1");

_Static_assert(FAIL == 0, "FAIL must be 0");
_Static_assert(OK == 1, "OK must be 1");

// Phase 2: validate sctx_T layout for Rust repr(C) mirror (globals.rs)
_Static_assert(sizeof(sctx_T) == 24, "sctx_T must be 24 bytes");
_Static_assert(offsetof(sctx_T, sc_sid) == 0, "sctx_T.sc_sid offset must be 0");
_Static_assert(offsetof(sctx_T, sc_seq) == 4, "sctx_T.sc_seq offset must be 4");
_Static_assert(offsetof(sctx_T, sc_lnum) == 8, "sctx_T.sc_lnum offset must be 8");
_Static_assert(offsetof(sctx_T, sc_chan) == 16, "sctx_T.sc_chan offset must be 16");

// Phase 1: validate constants inlined into Rust (constants.rs)
_Static_assert(IOSIZE == 1025, "IOSIZE must be 1025");
_Static_assert(MAXPATHL == 4096, "MAXPATHL must be 4096");
_Static_assert(PROF_YES == 1, "PROF_YES must be 1");
_Static_assert(DOSO_VIMRC == 1, "DOSO_VIMRC must be 1");
_Static_assert(SID_STR == -10, "SID_STR must be -10");
_Static_assert(DOCMD_VERBOSE == 0x01, "DOCMD_VERBOSE must be 0x01");
_Static_assert(DOCMD_NOWAIT == 0x02, "DOCMD_NOWAIT must be 0x02");
_Static_assert(DOCMD_REPEAT == 0x04, "DOCMD_REPEAT must be 0x04");
_Static_assert(CSTP_FINISH == 32, "CSTP_FINISH must be 32");
_Static_assert(EW_DIR == 0x01, "EW_DIR must be 0x01");
_Static_assert(EW_FILE == 0x02, "EW_FILE must be 0x02");
_Static_assert(EW_NOBREAK == 0x40000, "EW_NOBREAK must be 0x40000");
_Static_assert(CPO_CONCAT == 'C', "CPO_CONCAT must be 'C'");
_Static_assert(CONV_NONE == 0, "CONV_NONE must be 0");

// Phase 3: validate estack_T layout mirrored in Rust (globals.rs EstackT)
_Static_assert(sizeof(estack_T) == 32, "estack_T size must be 32");
_Static_assert(offsetof(estack_T, es_lnum) == 0, "estack_T.es_lnum must be at offset 0");
_Static_assert(offsetof(estack_T, es_name) == 8, "estack_T.es_name must be at offset 8");
_Static_assert(offsetof(estack_T, es_type) == 16, "estack_T.es_type must be at offset 16");
_Static_assert(offsetof(estack_T, es_info) == 24, "estack_T.es_info must be at offset 24");

// Deleted: nvim_exestack_ga_grow, nvim_exestack_get_entry, nvim_exestack_get_next_slot,
//          nvim_exestack_inc_len, nvim_exestack_dec_len, nvim_exestack_has_data,
//          nvim_estack_get_lnum, nvim_estack_set_lnum, nvim_estack_get_name,
//          nvim_estack_set_name, nvim_estack_get_type, nvim_estack_set_type,
//          nvim_estack_set_entry, nvim_estack_get_info_ufunc, nvim_estack_set_info_ufunc,
//          nvim_estack_get_info_aucmd — replaced by direct EstackT field access in Rust.

const char *nvim_ufunc_get_name(ufunc_T *fp) { return fp->uf_name; }

const char *nvim_ufunc_get_name_exp(ufunc_T *fp) { return fp->uf_name_exp; }

int nvim_ufunc_get_script_ctx_sid(ufunc_T *fp) { return fp->uf_script_ctx.sc_sid; }

linenr_T nvim_ufunc_get_script_ctx_lnum(ufunc_T *fp) { return fp->uf_script_ctx.sc_lnum; }

uint64_t nvim_ufunc_get_script_ctx_chan(ufunc_T *fp) { return fp->uf_script_ctx.sc_chan; }

int nvim_aucmd_get_script_ctx_sid(AutoPatCmd *apc) { return apc->script_ctx.sc_sid; }

linenr_T nvim_aucmd_get_script_ctx_lnum(AutoPatCmd *apc) { return apc->script_ctx.sc_lnum; }

linenr_T nvim_get_sourcing_lnum_direct(void) { return SOURCING_LNUM; }

/// Set SOURCING_LNUM.
void nvim_rt_set_sourcing_lnum(int lnum) { SOURCING_LNUM = (linenr_T)lnum; }

/// Format a stack entry with line number: "type_name name[lnum]dots"
int nvim_estack_format_entry(char *buf, size_t buflen,
                             const char *type_name, const char *name,
                             linenr_T lnum, const char *dots)
{
  if (lnum == 0) {
    return vim_snprintf(buf, buflen, "%s%s%s", type_name, name, dots);
  }
  return vim_snprintf(buf, buflen, "%s%s[%" PRIdLINENR "]%s",
                      type_name, name, lnum, dots);
}

// Phase 4: validate scriptitem_T layout mirrored in Rust (globals.rs ScriptitemT)
_Static_assert(sizeof(scriptitem_T) == 128, "scriptitem_T size must be 128");
_Static_assert(offsetof(scriptitem_T, sn_vars) == 0, "scriptitem_T.sn_vars offset");
_Static_assert(offsetof(scriptitem_T, sn_name) == 8, "scriptitem_T.sn_name offset");
_Static_assert(offsetof(scriptitem_T, sn_lua) == 16, "scriptitem_T.sn_lua offset");
_Static_assert(offsetof(scriptitem_T, sn_prof_on) == 17, "scriptitem_T.sn_prof_on offset");
_Static_assert(offsetof(scriptitem_T, sn_pr_force) == 18, "scriptitem_T.sn_pr_force offset");
_Static_assert(offsetof(scriptitem_T, sn_pr_child) == 24, "scriptitem_T.sn_pr_child offset");
_Static_assert(offsetof(scriptitem_T, sn_pr_nest) == 32, "scriptitem_T.sn_pr_nest offset");
_Static_assert(offsetof(scriptitem_T, sn_pr_count) == 36, "scriptitem_T.sn_pr_count offset");
_Static_assert(offsetof(scriptitem_T, sn_pr_total) == 40, "scriptitem_T.sn_pr_total offset");
_Static_assert(offsetof(scriptitem_T, sn_pr_children) == 64, "scriptitem_T.sn_pr_children offset");
_Static_assert(offsetof(scriptitem_T, sn_prl_ga) == 72, "scriptitem_T.sn_prl_ga offset");
_Static_assert(offsetof(scriptitem_T, sn_prl_start) == 96, "scriptitem_T.sn_prl_start offset");
_Static_assert(offsetof(scriptitem_T, sn_prl_idx) == 120, "scriptitem_T.sn_prl_idx offset");
_Static_assert(offsetof(scriptitem_T, sn_prl_execed) == 124, "scriptitem_T.sn_prl_execed offset");

// Deleted: nvim_script_items_get_len, nvim_script_item_get, nvim_scriptitem_get_name,
//          nvim_scriptitem_is_lua, nvim_scriptitem_get_prof_on — replaced by direct
//          ScriptitemT field access in Rust.

int nvim_estack_get_sctx_sid(estack_T *entry)
{
  if (entry->es_type == ETYPE_SCRIPT || entry->es_type == ETYPE_MODELINE) {
    return entry->es_info.sctx ? entry->es_info.sctx->sc_sid : 0;
  }
  return 0;
}

/// For a ufunc/aucmd entry, get the SID of the defining script context.
int nvim_estack_get_def_ctx_sid(estack_T *entry)
{
  if (entry->es_type == ETYPE_UFUNC) {
    return entry->es_info.ufunc->uf_script_ctx.sc_sid;
  } else if (entry->es_type == ETYPE_AUCMD) {
    return entry->es_info.aucmd->script_ctx.sc_sid;
  }
  return 0;
}

/// For a ufunc/aucmd entry, get the name of the defining script.
char *nvim_estack_get_def_script_name(estack_T *entry)
{
  int sid = 0;
  if (entry->es_type == ETYPE_UFUNC) {
    sid = entry->es_info.ufunc->uf_script_ctx.sc_sid;
  } else if (entry->es_type == ETYPE_AUCMD) {
    sid = entry->es_info.aucmd->script_ctx.sc_sid;
  }
  if (sid > 0 && SCRIPT_ID_VALID(sid)) {
    return xstrdup(SCRIPT_ITEM(sid)->sn_name);
  }
  return NULL;
}

/// Allocate a locked dict.
dict_T *nvim_rt_dict_alloc_lock(void) { return tv_dict_alloc_lock(VAR_FIXED); }

/// Allocate a list with a known length.
list_T *nvim_rt_list_alloc(int count) { return tv_list_alloc(count); }

/// Add a funcref to a dict.
void nvim_rt_dict_add_func(dict_T *d, ufunc_T *fp) { tv_dict_add_func(d, S_LEN("funcref"), fp); }

/// Add an "event" string to a dict.
void nvim_rt_dict_add_event(dict_T *d, const char *event) { tv_dict_add_str(d, S_LEN("event"), event); }

/// Add a "lnum" number to a dict.
void nvim_rt_dict_add_lnum(dict_T *d, linenr_T lnum) { tv_dict_add_nr(d, S_LEN("lnum"), lnum); }

/// Add a "filepath" string to a dict.
void nvim_rt_dict_add_filepath(dict_T *d, const char *filepath) { tv_dict_add_str(d, S_LEN("filepath"), filepath); }

/// Append a dict typval to a list.
void nvim_rt_list_append_dict(list_T *l, dict_T *d)
{
  typval_T tv = {
    .v_type = VAR_DICT,
    .v_lock = VAR_LOCKED,
    .vval.v_dict = d,
  };
  tv_list_append_tv(l, &tv);
}

void nvim_rt_list_set_ret(void *rettv, list_T *l) { tv_list_set_ret((typval_T *)rettv, l); }

/// Call get_scriptname for a ufunc's script context.
const char *nvim_ufunc_get_scriptname(ufunc_T *fp)
{
  sctx_T sctx = fp->uf_script_ctx;
  if (sctx.sc_sid > 0) {
    return get_scriptname(sctx, NULL);
  }
  return "";
}

/// Call get_scriptname for an aucmd's script context.
const char *nvim_aucmd_get_scriptname(AutoPatCmd *apc)
{
  sctx_T sctx = apc->script_ctx;
  if (sctx.sc_sid > 0) {
    return get_scriptname(sctx, NULL);
  }
  return "";
}

_Static_assert(SID_MODELINE == -1, "SID_MODELINE");
_Static_assert(SID_CMDARG == -2, "SID_CMDARG");
_Static_assert(SID_CARG == -3, "SID_CARG");
_Static_assert(SID_ENV == -4, "SID_ENV");
_Static_assert(SID_ERROR == -5, "SID_ERROR");
_Static_assert(SID_NONE == -6, "SID_NONE");
_Static_assert(SID_WINLAYOUT == -7, "SID_WINLAYOUT");
_Static_assert(SID_LUA == -8, "SID_LUA");
_Static_assert(SID_API_CLIENT == -9, "SID_API_CLIENT");
_Static_assert(SID_STR == -10, "SID_STR");

/// Allocate script-local variables for a script.
void nvim_new_script_vars(int sid) { new_script_vars(sid); }

// Deleted: nvim_script_items_ga_grow, nvim_script_items_inc_len,
//          nvim_script_items_set_item, nvim_xcalloc_scriptitem,
//          nvim_scriptitem_set_name, nvim_scriptitem_set_prof_on — replaced by
//          direct ScriptitemT/script_items field access in Rust.

/// Compare two filenames (path_fnamecmp wrapper).
int nvim_rt_path_fnamecmp(const char *a, const char *b) { return path_fnamecmp(a, b); }

/// Full implementation of get_scriptname, callable from Rust.
char *nvim_rt_get_scriptname(int sc_sid, uint64_t sc_chan, bool *should_free)
{
  sctx_T ctx = { .sc_sid = sc_sid, .sc_chan = sc_chan };
  return get_scriptname(ctx, should_free);
}

int nvim_rt_exarg_get_addr_count(void *eap) { return ((exarg_T *)eap)->addr_count; }

linenr_T nvim_rt_exarg_get_line2(void *eap) { return ((exarg_T *)eap)->line2; }

char *nvim_rt_exarg_get_arg(void *eap) { return ((exarg_T *)eap)->arg; }

void nvim_rt_exarg_set_arg(void *eap, char *arg) { ((exarg_T *)eap)->arg = arg; }

bool nvim_exarg_arg_is_nul(void *eap) { return *((exarg_T *)eap)->arg == NUL; }

/// Call expand_env.
void nvim_rt_expand_env(char *src, char *dst, int dstlen) { expand_env(src, dst, dstlen); }

/// Call do_exedit.
void nvim_rt_do_exedit(void *eap) { do_exedit((exarg_T *)eap, NULL); }

void nvim_rt_emsg_invarg(void) { emsg(_(e_invarg)); }

char *nvim_rt_get_namebuff(void) { return NameBuff; }

char *nvim_rt_get_iobuff(void) { return IObuff; }

/// Call home_replace(NULL, name, buf, len, true).
void nvim_rt_home_replace(const char *name, char *buf, size_t len) { home_replace(NULL, name, buf, len, true); }

void nvim_rt_format_script_entry(int i, const char *namebuff)
{
  vim_snprintf(IObuff, (size_t)IOSIZE, "%3d: %s", i, namebuff);
}

bool nvim_rt_message_filtered(const char *msg) { return message_filtered(msg); }

/// Output a newline.
void nvim_rt_msg_putchar_nl(void) { msg_putchar('\n'); }

/// Output a translated string.
void nvim_rt_msg_outtrans(const char *msg) { msg_outtrans(msg, 0, false); }

void nvim_rt_line_breakcheck(void) { line_breakcheck(); }

/// Allocate a list and set it as the return value.
void nvim_rt_list_alloc_ret(void *rettv, int count) { tv_list_alloc_ret((typval_T *)rettv, count); }

bool nvim_rt_check_for_opt_dict_arg(void *argvars) { return tv_check_for_opt_dict_arg((typval_T *)argvars, 0) != FAIL; }

list_T *nvim_rt_get_rettv_list(void *rettv) { return ((typval_T *)rettv)->vval.v_list; }

bool nvim_rt_argvars_is_dict(void *argvars) { return ((typval_T *)argvars)[0].v_type == VAR_DICT; }

/// Find "sid" in a dict from argvars.
/// Returns: >0 = valid sid, -1 = not found, -2 = error, -3 = invalid value.
int64_t nvim_rt_dict_find_sid(void *argvars)
{
  dict_T *dict = ((typval_T *)argvars)[0].vval.v_dict;
  dictitem_T *sid_di = tv_dict_find(dict, S_LEN("sid"));
  if (sid_di == NULL) {
    return -1;
  }
  bool error = false;
  varnumber_T sid = tv_get_number_chk(&sid_di->di_tv, &error);
  if (error) {
    return -2;
  }
  if (sid <= 0) {
    semsg(_(e_invargNval), "sid", tv_get_string(&sid_di->di_tv));
    return -3;
  }
  return sid;
}

char *nvim_rt_dict_get_name_pat(void *argvars)
{
  dict_T *dict = ((typval_T *)argvars)[0].vval.v_dict;
  return tv_dict_get_string(dict, "name", true);
}

/// Compile a regex pattern. Returns opaque handle or NULL on failure.
void *nvim_rt_vim_regcomp(const char *pat)
{
  regmatch_T *rm = xcalloc(1, sizeof(regmatch_T));
  rm->rm_ic = p_ic;
  rm->regprog = vim_regcomp((char *)pat, RE_MAGIC + RE_STRING);
  if (rm->regprog == NULL) {
    xfree(rm);
    return NULL;
  }
  return rm;
}

/// Test if a string matches a compiled regex.
bool nvim_rt_vim_regexec(void *regmatch, const char *str)
{
  return vim_regexec((regmatch_T *)regmatch, (char *)str, 0);
}

void nvim_rt_vim_regfree(void *regmatch)
{
  if (regmatch != NULL) {
    regmatch_T *rm = (regmatch_T *)regmatch;
    vim_regfree(rm->regprog);
    xfree(rm);
  }
}

/// Allocate a dict.
dict_T *nvim_rt_p2_dict_alloc(void) { return tv_dict_alloc(); }

/// Add a string to a dict.
void nvim_rt_dict_add_str(dict_T *d, const char *key, size_t keylen,
                          const char *val) { tv_dict_add_str(d, key, keylen, val); }

/// Add a number to a dict.
void nvim_rt_dict_add_nr(dict_T *d, const char *key, size_t keylen,
                         int64_t nr) { tv_dict_add_nr(d, key, keylen, (varnumber_T)nr); }

/// Add a bool to a dict.
void nvim_rt_dict_add_bool(dict_T *d, const char *key, size_t keylen,
                           bool val) { tv_dict_add_bool(d, key, keylen, val ? kBoolVarTrue : kBoolVarFalse); }

/// Append a dict to a list.
void nvim_rt_p2_tv_list_append_dict(list_T *l, dict_T *d) { tv_list_append_dict(l, d); }

dict_T *nvim_rt_copy_script_vars(int sid)
{
  scriptitem_T *si = SCRIPT_ITEM(sid);
  if (si->sn_vars == NULL) {
    return tv_dict_alloc();
  }
  return tv_dict_copy(NULL, &si->sn_vars->sv_dict, true, rs_get_copyID());
}

/// Add a dict to a dict.
void nvim_rt_dict_add_dict(dict_T *d, const char *key, size_t keylen,
                           dict_T *val) { tv_dict_add_dict(d, key, keylen, val); }

/// Add a list to a dict.
void nvim_rt_dict_add_list(dict_T *d, const char *key, size_t keylen,
                           list_T *val) { tv_dict_add_list(d, key, keylen, val); }

#if defined(BACKSLASH_IN_FILENAME)
/// Adjust slashes in a filename (Windows only).
void nvim_rt_slash_adjust(char *name) { slash_adjust(name); }
#endif

// Static assertions for XDG types
_Static_assert(kXDGNone == -1, "kXDGNone");
_Static_assert(kXDGConfigHome == 0, "kXDGConfigHome");
_Static_assert(kXDGDataHome == 1, "kXDGDataHome");
_Static_assert(kXDGCacheHome == 2, "kXDGCacheHome");
_Static_assert(kXDGStateHome == 3, "kXDGStateHome");
_Static_assert(kXDGRuntimeDir == 4, "kXDGRuntimeDir");
_Static_assert(kXDGConfigDirs == 5, "kXDGConfigDirs");
_Static_assert(kXDGDataDirs == 6, "kXDGDataDirs");

/// vim_env_iter wrapper - iterate forward through ENV_SEPCHAR-separated values.
const void *nvim_rt_vim_env_iter(const char *val, const void *iter,
                                 const char **dir, size_t *len)
{
  return vim_env_iter(ENV_SEPCHAR, val, iter, dir, len);
}

/// vim_env_iter_rev wrapper - iterate backwards.
const void *nvim_rt_vim_env_iter_rev(const char *val, const void *iter,
                                     const char **dir, size_t *len)
{
  return vim_env_iter_rev(ENV_SEPCHAR, val, iter, dir, len);
}

bool nvim_rt_after_pathsep(const char *b, const char *s) { return after_pathsep(b, s); }

/// Count occurrences of a character in a buffer.
size_t nvim_rt_memcnt(const void *s, int c, size_t n) { return memcnt(s, c, n); }

const char *nvim_rt_get_appname(void) { return get_appname(false); }

char *nvim_rt_stdpaths_get_xdg_var(int type) { return stdpaths_get_xdg_var((XDGVarType)type); }

char *nvim_rt_vim_getenv(const char *name) { return vim_getenv(name); }

bool nvim_rt_os_isdir(const char *name) { return os_isdir(name); }

const char *nvim_rt_get_default_lib_dir(void) { return default_lib_dir; }

void nvim_rt_vim_get_prefix_from_exepath(char *buf) { vim_get_prefix_from_exepath(buf); }

/// Append path component.
int nvim_rt_append_path(char *path, const char *to_append, size_t max_len)
{
  return append_path(path, to_append, max_len);
}

bool nvim_rt_vim_ispathsep(int c) { return vim_ispathsep(c); }

_Static_assert(EW_DIR == 0x01, "EW_DIR must be 0x01");
_Static_assert(EW_FILE == 0x02, "EW_FILE must be 0x02");

bool nvim_rt_pkg_exarg_get_forceit(void *eap) { return ((exarg_T *)eap)->forceit; }

/// Call fix_fname (returns allocated string).
char *nvim_rt_pkg_fix_fname(const char *fname) { return fix_fname(fname); }

/// Call vim_snprintf.
int nvim_rt_pkg_snprintf(char *buf, size_t len, const char *fmt, const char *arg)
{
  return vim_snprintf(buf, len, fmt, arg);
}

/// Call eval_to_number.
int64_t nvim_rt_pkg_eval_to_number(char *expr) { return (int64_t)eval_to_number(expr, false); }

/// Call do_cmdline_cmd.
void nvim_rt_pkg_do_cmdline_cmd(const char *cmd) { do_cmdline_cmd(cmd); }

/// Call TIME_MSG (time_msg if time_fd is set).
void nvim_rt_pkg_time_msg(const char *msg) { TIME_MSG(msg); }

_Static_assert(EXPAND_RUNTIME == 51, "EXPAND_RUNTIME must be 51");

void nvim_rt_cmd_expand_set_context(void *xp, int context, const char *pattern)
{
  ((expand_T *)xp)->xp_context = context;
  ((expand_T *)xp)->xp_pattern = (char *)pattern;
}

/// Advance pointer by one multibyte character (MB_PTR_ADV).
int nvim_rt_utfc_ptr2len(const char *p) { return utfc_ptr2len(p); }

bool nvim_rt_vim_ispathsep_nocolon(int c) { return vim_ispathsep_nocolon(c); }

/// Add a path separator to the end of the path if not present.
void nvim_rt_add_pathsep(char *p) { add_pathsep(p); }

/// Compare first n bytes of filenames (path-aware).
int nvim_rt_path_fnamencmp(const char *a, const char *b, size_t n) { return path_fnamencmp(a, b, n); }

/// Concatenate two path components into an allocated string.
char *nvim_rt_concat_fnames(const char *fname1, const char *fname2, bool sep)
{
  return concat_fnames((char *)fname1, fname2, sep);
}

void nvim_rt_set_runtimepath(const char *new_rtp)
{
  set_option_value_give_err(kOptRuntimepath, CSTR_AS_OPTVAL(new_rtp), 0);
}

/// get_past_head: skip drive letter/UNC prefix on Windows, identity on Unix.
char *nvim_rt_get_past_head(const char *path) { return get_past_head((char *)path); }

/// fix_fname: canonicalize path (resolve symlinks, etc).
char *nvim_rt_fix_fname(const char *fname) { return fix_fname((char *)fname); }

// =============================================================================
// Phase 2: Accessors for ex_finish / ex_scriptencoding / source_finished
// =============================================================================

/// Get eap->ea_getline as a void* (LineGetter is a function pointer).
void *nvim_rt_exarg_get_getline_fn(void *eap) { return (void *)(uintptr_t)((exarg_T *)eap)->ea_getline; }

/// Get eap->cookie.
void *nvim_rt_exarg_get_cookie(void *eap) { return ((exarg_T *)eap)->cookie; }

/// Get eap->cstack.
void *nvim_rt_exarg_get_cstack(void *eap) { return ((exarg_T *)eap)->cstack; }

/// Check if eap->ea_getline is getsourceline.
bool nvim_rt_exarg_is_sourcing(void *eap) { return getline_equal(((exarg_T *)eap)->ea_getline, ((exarg_T *)eap)->cookie, getsourceline); }

/// Get the source_cookie from eap (via getline_cookie).
void *nvim_rt_exarg_get_source_cookie(void *eap) { return getline_equal(((exarg_T *)eap)->ea_getline, ((exarg_T *)eap)->cookie, getsourceline) ? getline_cookie(((exarg_T *)eap)->ea_getline, ((exarg_T *)eap)->cookie) : NULL; }

/// Check if fgetline/cookie pair is sourcing (getline_equal getsourceline).
bool nvim_rt_getline_is_sourcing(void *fgetline, void *cookie) { return getline_equal((LineGetter)(uintptr_t)fgetline, cookie, getsourceline); }

/// Get source_cookie from fgetline/cookie pair.
void *nvim_rt_getline_get_source_cookie(void *fgetline, void *cookie) { return getline_cookie((LineGetter)(uintptr_t)fgetline, cookie); }

/// Canonicalize an encoding name.
char *nvim_rt_enc_canonize(char *enc) { return enc_canonize(enc); }

/// Setup encoding conversion.
int nvim_rt_convert_setup(void *vcp, char *from, const char *to) { return convert_setup((vimconv_T *)vcp, from, (char *)to); }

/// Get p_enc (the 'encoding' option value).
const char *nvim_rt_get_p_enc(void) { return p_enc; }

/// cleanup_conditionals wrapper.
int nvim_rt_cleanup_conditionals(void *cstack, int searched_cond, int inclusive)
{
  return cleanup_conditionals((cstack_T *)cstack, searched_cond, inclusive);
}

/// Set cs_pending at index.
void nvim_rt_cstack_set_pending(void *cstack, int idx, int val)
{
  ((cstack_T *)cstack)->cs_pending[idx] = (char)val;
}

/// report_make_pending wrapper (for CSTP_FINISH).
void nvim_rt_report_make_pending_finish(void) { report_make_pending(CSTP_FINISH, NULL); }

/// emsg for E167.
void nvim_rt_emsg_scriptencoding_outside(void) { emsg(_("E167: :scriptencoding used outside of a sourced file")); }

/// emsg for E168.
void nvim_rt_emsg_finish_outside(void) { emsg(_("E168: :finish used outside of a sourced file")); }

// =============================================================================
// Phase 4: Accessors for do_source_ext and related functions
// =============================================================================

/// expand_env_save: expand environment variables and return allocated copy.
char *nvim_rt_expand_env_save(const char *fname) { return expand_env_save((char *)fname); }

/// os_isdir wrapper.
bool nvim_rt_src_os_isdir(const char *fname) { return os_isdir(fname); }

/// path_tail wrapper.
char *nvim_rt_src_path_tail(char *fname) { return path_tail(fname); }

/// apply_autocmds wrapper for source events.
bool nvim_rt_apply_autocmds(int event, const char *fname_exp, const char *fname, bool force, void *buf)
{
  return apply_autocmds((event_T)event, (char *)fname_exp, (char *)fname, force, (buf_T *)buf);
}

/// Get EVENT_SOURCECMD value.
int nvim_rt_EVENT_SOURCECMD(void) { return EVENT_SOURCECMD; }
/// Get EVENT_SOURCEPRE value.
int nvim_rt_EVENT_SOURCEPRE(void) { return EVENT_SOURCEPRE; }
/// Get EVENT_SOURCEPOST value.
int nvim_rt_EVENT_SOURCEPOST(void) { return EVENT_SOURCEPOST; }

/// aborting() wrapper.
bool nvim_rt_aborting(void) { return aborting(); }

/// vimrc_found wrapper.
void nvim_rt_vimrc_found(const char *fname_exp, const char *env) { vimrc_found((char *)fname_exp, (char *)env); }

/// fclose wrapper.
int nvim_rt_fclose(void *fp) { return fclose((FILE *)fp); }

/// smsg wrapper for do_source verbose messages.
void nvim_rt_smsg_cannot_source(const char *fname) { smsg(0, _("Cannot source a directory: \"%s\""), fname); }
void nvim_rt_smsg_could_not_source(const char *fname) { smsg(0, _("could not source \"%s\""), fname); }
void nvim_rt_smsg_could_not_source_lnum(int64_t lnum, const char *fname)
{
  smsg(0, _("line %" PRId64 ": could not source \"%s\""), lnum, fname);
}
void nvim_rt_smsg_sourcing(const char *fname) { smsg(0, _("sourcing \"%s\""), fname); }
void nvim_rt_smsg_sourcing_lnum(int64_t lnum, const char *fname)
{
  smsg(0, _("line %" PRId64 ": sourcing \"%s\""), lnum, fname);
}
void nvim_rt_smsg_finished_sourcing(const char *fname) { smsg(0, _("finished sourcing %s"), fname); }
void nvim_rt_smsg_continuing_in(const char *name) { smsg(0, _("continuing in %s"), name); }

/// verbose_enter/leave wrappers.
void nvim_rt_verbose_enter(void) { verbose_enter(); }
void nvim_rt_verbose_leave(void) { verbose_leave(); }

/// Get SOURCING_NAME (can be NULL).
const char *nvim_rt_get_sourcing_name(void) { return HAVE_SOURCING_INFO ? SOURCING_NAME : NULL; }

/// Get SOURCING_LNUM.
int nvim_rt_get_sourcing_lnum(void) { return SOURCING_LNUM; }

/// Get time_fd.
void *nvim_rt_get_time_fd(void) { return time_fd; }

/// time_push/pop/msg wrappers.
void nvim_rt_time_push(uint64_t *rel_time, uint64_t *start_time) { time_push((proftime_T *)rel_time, (proftime_T *)start_time); }
void nvim_rt_time_pop(uint64_t rel_time) { time_pop((proftime_T)rel_time); }
void nvim_rt_time_msg_iobuff(const char *fname)
{
  vim_snprintf(IObuff, (size_t)IOSIZE, "sourcing %s", fname);
  time_msg(IObuff, NULL);
}

/// prof_child_enter/exit wrappers.
void nvim_rt_prof_child_enter(uint64_t *wait_start) { prof_child_enter((proftime_T *)wait_start); }
void nvim_rt_prof_child_exit(uint64_t *wait_start) { prof_child_exit((proftime_T *)wait_start); }

/// save_funccal / restore_funccal wrappers.
void *nvim_rt_save_funccal(void)
{
  funccal_entry_T *entry = xmalloc(sizeof(funccal_entry_T));
  save_funccal(entry);
  return entry;
}
void nvim_rt_restore_funccal(void *entry) { restore_funccal(); xfree(entry); }

// Deleted: nvim_rt_script_item_get, nvim_rt_si_get_sn_prof_on, nvim_rt_si_set_sn_prof_on,
//          nvim_rt_si_get_sn_pr_force, nvim_rt_si_set_sn_pr_force, nvim_rt_si_inc_pr_count,
//          nvim_rt_si_get_pr_children, nvim_rt_si_get_sn_name, nvim_rt_si_get_sn_lua —
//          replaced by direct ScriptitemT field access in Rust.

/// has_profiling wrapper.
bool nvim_rt_has_profiling(bool file, const char *name, bool *forceit) { return has_profiling(file, name, forceit); }

/// profile_init wrapper.
void nvim_rt_profile_init(void *si) { profile_init((scriptitem_T *)si); }

/// profile_start/end/zero/sub_wait/add/self wrappers.
uint64_t nvim_rt_profile_start(void) { return (uint64_t)profile_start(); }
uint64_t nvim_rt_profile_end(uint64_t tm) { return (uint64_t)profile_end((proftime_T)tm); }
uint64_t nvim_rt_profile_zero(void) { return (uint64_t)profile_zero(); }
uint64_t nvim_rt_profile_sub_wait(uint64_t wait_start, uint64_t tm) { return (uint64_t)profile_sub_wait((proftime_T)wait_start, (proftime_T)tm); }
uint64_t nvim_rt_profile_add(uint64_t tm1, uint64_t tm2) { return (uint64_t)profile_add((proftime_T)tm1, (proftime_T)tm2); }
uint64_t nvim_rt_profile_self(uint64_t self, uint64_t total, uint64_t children) { return (uint64_t)profile_self((proftime_T)self, (proftime_T)total, (proftime_T)children); }

/// Set si profiling fields after source.
void nvim_rt_si_update_profile(void *si, uint64_t wait_start)
{
  scriptitem_T *sip = (scriptitem_T *)si;
  sip->sn_pr_start = profile_end(sip->sn_pr_start);
  sip->sn_pr_start = profile_sub_wait((proftime_T)wait_start, sip->sn_pr_start);
  sip->sn_pr_total = profile_add(sip->sn_pr_total, sip->sn_pr_start);
  sip->sn_pr_self = profile_self(sip->sn_pr_self, sip->sn_pr_start, sip->sn_pr_children);
}

/// si_set_pr_start (from profile_start()).
void nvim_rt_si_set_pr_start(void *si, uint64_t tm)
{
  ((scriptitem_T *)si)->sn_pr_start = (proftime_T)tm;
  ((scriptitem_T *)si)->sn_pr_children = profile_zero();
}

// Deleted: nvim_rt_si_set_sn_lua, nvim_rt_si_set_sn_name —
//          replaced by direct ScriptitemT field writes in Rust.

/// emsg for E_INTERR.
void nvim_rt_emsg_interr(void) { emsg(_(e_interr)); }

/// curbuf accessor.
void *nvim_rt_get_curbuf(void) { return curbuf; }

/// curbuf->b_ffname accessor.
const char *nvim_rt_curbuf_get_ffname(void) { return curbuf ? curbuf->b_ffname : NULL; }

/// curbuf->b_fnum accessor.
int nvim_rt_curbuf_get_fnum(void) { return curbuf ? curbuf->b_fnum : 0; }

/// curbuf->b_fname accessor.
const char *nvim_rt_curbuf_get_fname(void) { return curbuf ? curbuf->b_fname : NULL; }

/// curbuf->b_p_ft accessor (filetype).
const char *nvim_rt_curbuf_get_ft(void) { return curbuf ? curbuf->b_p_ft : NULL; }

/// IObuff accessor.
char *nvim_rt_src_get_iobuff(void) { return IObuff; }

/// get_scriptname wrapper (from runtime.c, used by FFI).
char *nvim_rt_src_get_scriptname(int sc_sid, uint64_t sc_chan, bool *should_free)
{
  sctx_T ctx = { .sc_sid = sc_sid, .sc_chan = sc_chan };
  return get_scriptname(ctx, should_free);
}

/// nlua_exec_file wrapper.
void nvim_rt_nlua_exec_file(const char *fname) { nlua_exec_file(fname); }

/// nlua_exec_ga wrapper (executes buflines from a garray_T*).
void nvim_rt_nlua_exec_ga(void *ga, const char *fname)
{
  nlua_exec_ga((garray_T *)ga, (char *)fname);
}

/// string_convert wrapper.
char *nvim_rt_string_convert(void *vcp, char *s, size_t *len) { return string_convert((vimconv_T *)vcp, s, len); }

/// Check BOM: firstline[0..2] == {0xef, 0xbb, 0xbf}.
bool nvim_rt_check_utf8_bom(const uint8_t *line, size_t len)
{
  return len >= 3 && line[0] == 0xef && line[1] == 0xbb && line[2] == 0xbf;
}

/// add_win_cmd_modifiers wrapper.
void nvim_rt_add_win_cmd_modifiers(char *buf, bool *multi_mods)
{
  add_win_cmd_modifiers(buf, &cmdmod, multi_mods);
}

/// os_setenv wrapper.
void nvim_rt_os_setenv(const char *name, const char *val, int overwrite) { os_setenv(name, val, overwrite); }

/// SYS_OPTWIN_FILE constant.
const char *nvim_rt_SYS_OPTWIN_FILE(void) { return SYS_OPTWIN_FILE; }

// nvim_rt_openscript deleted: openscript now exported directly from Rust (typebuf.rs, Phase 2)

/// exarg_T: get eap->nextcmd.
const char *nvim_rt_exarg_get_nextcmd(const void *eap) { return ((exarg_T *)eap)->nextcmd; }

/// exarg_T: get eap->cstack->cs_idx.
int nvim_rt_exarg_get_cstack_idx(const void *eap) { return ((exarg_T *)eap)->cstack->cs_idx; }

/// eap->forceit accessor.
bool nvim_rt_exarg_get_forceit(const void *eap) { return ((exarg_T *)eap)->forceit; }

/// eap->line1 accessor.
int nvim_rt_exarg_get_line1(const void *eap) { return (int)((exarg_T *)eap)->line1; }

/// eap->line2 accessor (already exists as nvim_rt_exarg_get_line2 for linenr_T).

/// ml_get wrapper.
const char *nvim_rt_ml_get(int lnum) { return ml_get((linenr_T)lnum); }

/// snprintf IObuff: ":{range}lua buffer=N" or ":source buffer=N".
void nvim_rt_snprintf_source_buffer_name(char *buf, int size, bool ex_lua, int fnum)
{
  if (ex_lua) {
    snprintf(buf, (size_t)size, ":{range}lua buffer=%d", fnum);
  } else {
    snprintf(buf, (size_t)size, ":source buffer=%d", fnum);
  }
}

/// ga_init wrapper for buflines.
void nvim_rt_ga_init_strptrs(void *ga) { ga_init((garray_T *)ga, (int)sizeof(char *), 100); }

/// ga_append for buflines.
void nvim_rt_ga_append_str(void *ga, char *str) { GA_APPEND(char *, (garray_T *)ga, str); }

/// skip_to_newline wrapper.
const char *nvim_rt_skip_to_newline(const char *str) { return skip_to_newline(str); }

/// xmemdupz wrapper.
char *nvim_rt_xmemdupz(const char *str, size_t len) { return xmemdupz(str, len); }

/// emsg_norange: signal "E16: Invalid range" for :source with range+file.
void nvim_rt_emsg_norange(void) { emsg(_(e_norange)); }

/// semsg for "can't open file".
void nvim_rt_semsg_notopen(const char *fname) { semsg(_(e_notopen), fname); }

/// emsg_argreq: "E471: Argument required".
void nvim_rt_emsg_argreq(void) { emsg(_(e_argreq)); }

/// do_source: trampoline to avoid circular dependency.
int nvim_rt_do_source(char *fname, bool check_other, int is_vimrc, int *ret_sid)
{
  return do_source(fname, check_other, is_vimrc, ret_sid);
}

/// SOURCING_NAME check: if not NULL, return it.
const char *nvim_rt_get_sourcing_name_if_set(void)
{
  return (HAVE_SOURCING_INFO && SOURCING_NAME != NULL) ? SOURCING_NAME : NULL;
}

/// SOURCING_LNUM value.
int nvim_rt_get_sourcing_lnum_value(void) { return HAVE_SOURCING_INFO ? SOURCING_LNUM : 0; }

/// vim_snprintf for traceback name.
void nvim_rt_snprintf_traceback(char *buf, int size, const char *traceback_name,
                                 const char *sourcing_name, int sourcing_lnum)
{
  vim_snprintf(buf, (size_t)size, "%s called at %s:%" PRId64,
               traceback_name, sourcing_name, (int64_t)sourcing_lnum);
}

/// STRICMP wrapper (case-insensitive string compare).
int nvim_rt_STRICMP(const char *a, const char *b) { return STRICMP(a, b); }

// =============================================================================
// Phase 5: Accessors for rs_do_in_path / rs_do_in_cached_path
// =============================================================================

/// path_is_after: check if a path is in an "after" directory.
bool nvim_rt_path_is_after(const char *buf, size_t buflen) { return path_is_after(buf, buflen); }

/// smsg wrapper: "Searching for %s under %s in %s".
void nvim_rt_smsg_searching_prefix(const char *name, const char *prefix, const char *path)
{
  smsg(0, _("Searching for \"%s\" under \"%s\" in \"%s\""), name, prefix, path);
}

/// smsg wrapper: "Searching for %s in %s".
void nvim_rt_smsg_searching_in(const char *name, const char *path)
{
  smsg(0, _("Searching for \"%s\" in \"%s\""), name, path);
}

/// smsg wrapper: "Searching for %s".
void nvim_rt_smsg_searching(const char *buf)
{
  smsg(0, _("Searching for \"%s\""), buf);
}

/// semsg wrapper for e_dirnotf: "not found in '%s': \"%s\"".
void nvim_rt_semsg_dirnotf(const char *basepath, const char *name)
{
  semsg(_(e_dirnotf), basepath, name);
}

/// smsg wrapper: "not found in '%s': \"%s\"".
void nvim_rt_smsg_notfound_in(const char *basepath, const char *name)
{
  smsg(0, _("not found in '%s': \"%s\""), basepath, name);
}

/// smsg wrapper: "not found in runtime path: \"%s\"".
void nvim_rt_smsg_notfound_rtp(const char *name)
{
  smsg(0, _("not found in runtime path: \"%s\""), name);
}

/// smsg wrapper: "Searching for \"%s\" in runtime path".
void nvim_rt_smsg_searching_rtp(const char *name)
{
  smsg(0, _("Searching for \"%s\" in runtime path"), name);
}

/// copy_option_part wrapper.
void nvim_rt_copy_option_part(char **option, char *buf, size_t maxlen, const char *sep_chars)
{
  copy_option_part(option, buf, maxlen, sep_chars);
}

/// vim_strchr wrapper.
char *nvim_rt_vim_strchr(const char *buf, int c) { return vim_strchr(buf, c); }

/// p_cpo accessor.
const char *nvim_rt_get_p_cpo(void) { return p_cpo; }

/// skipwhite wrapper.
char *nvim_rt_skipwhite(const char *p) { return skipwhite(p); }

/// ga_init wrapper (for growarray).
void nvim_rt_ga_init(void *ga, int itemsize, int growsize)
{
  ga_init((garray_T *)ga, itemsize, growsize);
}

/// ga_grow wrapper.
void nvim_rt_ga_grow(void *ga, int n)
{
  ga_grow((garray_T *)ga, n);
}

/// ga_concat wrapper.
void nvim_rt_ga_concat(void *ga, const char *s)
{
  ga_concat((garray_T *)ga, s);
}

/// ga_concat_len wrapper.
void nvim_rt_ga_concat_len(void *ga, const char *s, size_t len)
{
  ga_concat_len((garray_T *)ga, s, len);
}

/// ga_append wrapper (appends a single byte).
void nvim_rt_ga_append_byte(void *ga, char byte)
{
  ga_append((garray_T *)ga, byte);
}

/// ga_set_growsize wrapper.
void nvim_rt_ga_set_growsize(void *ga, int size)
{
  ga_set_growsize((garray_T *)ga, size);
}

/// ga_get_len wrapper.
int nvim_rt_ga_get_len(const void *ga) { return ((garray_T *)ga)->ga_len; }

/// ga_get_data wrapper.
void *nvim_rt_ga_get_data(const void *ga) { return ((garray_T *)ga)->ga_data; }

/// ga_get_maxlen wrapper.
int nvim_rt_ga_get_maxlen(const void *ga) { return ((garray_T *)ga)->ga_maxlen; }

/// Set ga_len.
void nvim_rt_ga_set_len(void *ga, int len) { ((garray_T *)ga)->ga_len = len; }

/// dbg_breakpoint wrapper.
void nvim_rt_dbg_breakpoint(const char *fname, int lnum)
{
  dbg_breakpoint((char *)fname, (linenr_T)lnum);
}

/// dbg_find_breakpoint wrapper.
int nvim_rt_dbg_find_breakpoint(bool file, const char *fname, int after)
{
  return (int)dbg_find_breakpoint(file, (char *)fname, (linenr_T)after);
}

/// script_line_start wrapper.
void nvim_rt_script_line_start(void) { script_line_start(); }

/// script_line_end wrapper.
void nvim_rt_script_line_end(void) { script_line_end(); }

/// Get vc_type from vimconv_T*.
int nvim_rt_conv_get_type(const void *vcp) { return ((vimconv_T *)vcp)->vc_type; }

/// skipwhite_len wrapper.
const char *nvim_rt_skipwhite_len(const char *p, size_t len) { return skipwhite_len(p, len); }
