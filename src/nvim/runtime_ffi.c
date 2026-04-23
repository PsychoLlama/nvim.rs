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

// Phase 4: validate EVENT_SOURCE* constants inlined into Rust (constants.rs)
_Static_assert(EVENT_SOURCECMD == 101, "EVENT_SOURCECMD must be 101");
_Static_assert(EVENT_SOURCEPOST == 102, "EVENT_SOURCEPOST must be 102");
_Static_assert(EVENT_SOURCEPRE == 103, "EVENT_SOURCEPRE must be 103");

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

// Deleted: nvim_rt_dict_alloc_lock — Rust uses tv_dict_alloc_lock(VAR_FIXED=2) directly via link_name.
// Deleted: nvim_rt_list_alloc — Rust uses tv_list_alloc directly via link_name.
// Deleted: nvim_rt_dict_add_func — Rust uses tv_dict_add_func with hardcoded "funcref" key via link_name.

// Deleted: nvim_rt_dict_add_event — Rust uses tv_dict_add_str directly via link_name.
// Deleted: nvim_rt_dict_add_lnum — Rust uses tv_dict_add_nr directly via link_name.
// Deleted: nvim_rt_dict_add_filepath — Rust uses tv_dict_add_str directly via link_name.
// Deleted: nvim_rt_list_append_dict — Rust uses tv_list_append_dict directly via link_name.

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

// Deleted: nvim_script_items_ga_grow, nvim_script_items_inc_len,
//          nvim_script_items_set_item, nvim_xcalloc_scriptitem,
//          nvim_scriptitem_set_name, nvim_scriptitem_set_prof_on — replaced by
//          direct ScriptitemT/script_items field access in Rust.

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

// Deleted: nvim_rt_do_exedit — Rust calls do_exedit(eap, NULL) directly via link_name.

// Deleted: nvim_rt_emsg_invarg — Rust calls emsg(gettext(e_invarg)) directly.

// Deleted: nvim_rt_get_namebuff — Rust imports NameBuff directly as extern static.
// Deleted: nvim_rt_get_iobuff — Rust imports IObuff directly as extern static.

/// Call home_replace(NULL, name, buf, len, true).
void nvim_rt_home_replace(const char *name, char *buf, size_t len) { home_replace(NULL, name, buf, len, true); }

// Deleted: nvim_rt_format_script_entry — reimplemented in Rust (script.rs).
// Deleted: nvim_rt_msg_putchar_nl — Rust calls msg_putchar('\n') directly via link_name.
// Deleted: nvim_rt_msg_outtrans — Rust calls msg_outtrans(msg, 0, false) directly via link_name.

/// Allocate a list and set it as the return value.

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

// Deleted: nvim_rt_p2_dict_alloc — Rust uses tv_dict_alloc directly via link_name.
// Deleted: nvim_rt_dict_add_str — Rust uses tv_dict_add_str directly via link_name.
// Deleted: nvim_rt_dict_add_nr — Rust uses tv_dict_add_nr directly via link_name.

/// Add a bool to a dict.
void nvim_rt_dict_add_bool(dict_T *d, const char *key, size_t keylen,
                           bool val) { tv_dict_add_bool(d, key, keylen, val ? kBoolVarTrue : kBoolVarFalse); }

/// Append a dict to a list.

dict_T *nvim_rt_copy_script_vars(int sid)
{
  scriptitem_T *si = SCRIPT_ITEM(sid);
  if (si->sn_vars == NULL) {
    return tv_dict_alloc();
  }
  return tv_dict_copy(NULL, &si->sn_vars->sv_dict, true, rs_get_copyID());
}

// Deleted: nvim_rt_dict_add_dict — Rust uses tv_dict_add_dict directly via link_name.
// Deleted: nvim_rt_dict_add_list — Rust uses tv_dict_add_list directly via link_name.

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


// Deleted: nvim_rt_get_appname — Rust calls get_appname(false) directly via link_name.
// Deleted: nvim_rt_stdpaths_get_xdg_var — Rust calls stdpaths_get_xdg_var directly via link_name.
// Deleted: nvim_rt_get_default_lib_dir — Rust imports default_lib_dir directly as extern static.


// Deleted: nvim_rt_append_path — Rust uses append_path directly via link_name.

_Static_assert(EW_DIR == 0x01, "EW_DIR must be 0x01");
_Static_assert(EW_FILE == 0x02, "EW_FILE must be 0x02");

bool nvim_rt_pkg_exarg_get_forceit(void *eap) { return ((exarg_T *)eap)->forceit; }

/// Call vim_snprintf.
int nvim_rt_pkg_snprintf(char *buf, size_t len, const char *fmt, const char *arg)
{
  return vim_snprintf(buf, len, fmt, arg);
}

/// Call eval_to_number.
int64_t nvim_rt_pkg_eval_to_number(char *expr) { return (int64_t)eval_to_number(expr, false); }

/// Call do_cmdline_cmd.

// Deleted: nvim_rt_pkg_time_msg — Rust checks time_fd and calls time_msg directly.

_Static_assert(EXPAND_RUNTIME == 51, "EXPAND_RUNTIME must be 51");

void nvim_rt_cmd_expand_set_context(void *xp, int context, const char *pattern)
{
  ((expand_T *)xp)->xp_context = context;
  ((expand_T *)xp)->xp_pattern = (char *)pattern;
}

void nvim_rt_set_runtimepath(const char *new_rtp)
{
  set_option_value_give_err(kOptRuntimepath, CSTR_AS_OPTVAL(new_rtp), 0);
}

// =============================================================================
// Phase 2: Accessors for ex_finish / ex_scriptencoding / source_finished
// =============================================================================

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

// Deleted: nvim_rt_enc_canonize — Rust uses enc_canonize directly via link_name.

/// Setup encoding conversion.

// Deleted: nvim_rt_get_p_enc — Rust imports p_enc directly as extern static.

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

// Deleted: nvim_rt_report_make_pending_finish — Rust calls report_make_pending with CSTP_FINISH constant directly.

// Deleted: nvim_rt_emsg_scriptencoding_outside — Rust calls emsg(gettext(...)) directly.
// Deleted: nvim_rt_emsg_finish_outside — Rust calls emsg(gettext(...)) directly.

// =============================================================================
// Phase 4: Accessors for do_source_ext and related functions
// =============================================================================

// Deleted: nvim_rt_expand_env_save — Rust uses expand_env_save directly via link_name.
// Deleted: nvim_rt_src_path_tail — Rust uses path_tail directly via link_name.

// Deleted: nvim_rt_apply_autocmds — Rust uses apply_autocmds directly via link_name.

// Deleted: nvim_rt_EVENT_SOURCECMD/PRE/POST — Rust uses constants directly from constants.rs.

/// vimrc_found wrapper.

/// fclose wrapper.

// Deleted: nvim_rt_smsg_cannot_source — Rust calls smsg(0, gettext(...), fname) directly.
// Deleted: nvim_rt_smsg_could_not_source — Rust calls smsg directly.
// Deleted: nvim_rt_smsg_could_not_source_lnum — Rust calls smsg directly.
// Deleted: nvim_rt_smsg_sourcing — Rust calls smsg directly.
// Deleted: nvim_rt_smsg_sourcing_lnum — Rust calls smsg directly.
// Deleted: nvim_rt_smsg_finished_sourcing — Rust calls smsg directly.
// Deleted: nvim_rt_smsg_continuing_in — Rust calls smsg directly.

/// verbose_enter/leave wrappers.

/// Get SOURCING_NAME (can be NULL).
const char *nvim_rt_get_sourcing_name(void) { return HAVE_SOURCING_INFO ? SOURCING_NAME : NULL; }

/// Get SOURCING_LNUM.
int nvim_rt_get_sourcing_lnum(void) { return SOURCING_LNUM; }

// Deleted: nvim_rt_get_time_fd — Rust imports time_fd directly as extern static.

// Deleted: nvim_rt_time_msg_iobuff — reimplemented in Rust (dosource.rs).

/// prof_child_enter/exit wrappers.

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

/// profile_init wrapper.

// Deleted: nvim_rt_profile_start — Rust uses profile_start directly via link_name.
// Deleted: nvim_rt_profile_zero — Rust uses profile_zero directly via link_name.

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

// Deleted: nvim_rt_emsg_interr — Rust calls emsg(gettext(e_interr)) directly.

// Deleted: nvim_rt_get_curbuf — Rust imports curbuf directly as extern static.

/// curbuf->b_ffname accessor.
const char *nvim_rt_curbuf_get_ffname(void) { return curbuf ? curbuf->b_ffname : NULL; }

/// curbuf->b_fnum accessor.
int nvim_rt_curbuf_get_fnum(void) { return curbuf ? curbuf->b_fnum : 0; }

/// curbuf->b_fname accessor.
const char *nvim_rt_curbuf_get_fname(void) { return curbuf ? curbuf->b_fname : NULL; }

/// curbuf->b_p_ft accessor (filetype).
const char *nvim_rt_curbuf_get_ft(void) { return curbuf ? curbuf->b_p_ft : NULL; }

// Deleted: nvim_rt_src_get_iobuff — Rust imports IObuff directly as extern static.

// Deleted: nvim_rt_nlua_exec_ga — Rust uses nlua_exec_ga directly via link_name.

// Deleted: nvim_rt_string_convert — Rust uses string_convert directly via link_name.

// Deleted: nvim_rt_check_utf8_bom — reimplemented in Rust (dosource.rs).

/// add_win_cmd_modifiers wrapper.
void nvim_rt_add_win_cmd_modifiers(char *buf, bool *multi_mods)
{
  add_win_cmd_modifiers(buf, &cmdmod, multi_mods);
}

/// os_setenv wrapper.

// Deleted: nvim_rt_SYS_OPTWIN_FILE — Rust uses compile-time constant SYS_OPTWIN_FILE.

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

// Deleted: nvim_rt_snprintf_source_buffer_name — reimplemented in Rust (dosource.rs).

// Deleted: nvim_rt_emsg_norange — Rust calls emsg(gettext(e_norange)) directly.
// Deleted: nvim_rt_semsg_notopen — Rust calls semsg(gettext(e_notopen), fname) directly.
// Deleted: nvim_rt_emsg_argreq — Rust calls emsg(gettext(e_argreq)) directly.

/// SOURCING_NAME check: if not NULL, return it.
const char *nvim_rt_get_sourcing_name_if_set(void)
{
  return (HAVE_SOURCING_INFO && SOURCING_NAME != NULL) ? SOURCING_NAME : NULL;
}

/// SOURCING_LNUM value.
int nvim_rt_get_sourcing_lnum_value(void) { return HAVE_SOURCING_INFO ? SOURCING_LNUM : 0; }

// Deleted: nvim_rt_snprintf_traceback — reimplemented in Rust (dosource.rs).

// Deleted: nvim_rt_STRICMP — Rust uses strcasecmp directly via link_name.

// =============================================================================
// Phase 5: Accessors for rs_do_in_path / rs_do_in_cached_path
// =============================================================================

/// path_is_after: check if a path is in an "after" directory.

// Deleted: nvim_rt_smsg_searching_prefix — Rust calls smsg(0, gettext(...), ...) directly.
// Deleted: nvim_rt_smsg_searching_in — Rust calls smsg directly.
// Deleted: nvim_rt_smsg_searching — Rust calls smsg directly.
// Deleted: nvim_rt_semsg_dirnotf — Rust calls semsg(gettext(e_dirnotf), ...) directly.
// Deleted: nvim_rt_smsg_notfound_in — Rust calls smsg directly.
// Deleted: nvim_rt_smsg_notfound_rtp — Rust calls smsg directly.
// Deleted: nvim_rt_smsg_searching_rtp — Rust calls smsg directly.

// Deleted: nvim_rt_copy_option_part — Rust uses copy_option_part directly via link_name.

// Deleted: nvim_rt_vim_strchr — Rust uses vim_strchr directly via link_name.

// Deleted: nvim_rt_get_p_cpo — Rust imports p_cpo directly as extern static.

// Deleted: nvim_rt_dbg_breakpoint — Rust uses dbg_breakpoint directly via link_name.
// Deleted: nvim_rt_dbg_find_breakpoint — Rust uses dbg_find_breakpoint directly via link_name.

/// script_line_start wrapper.

/// script_line_end wrapper.

/// Get vc_type from vimconv_T*.
int nvim_rt_conv_get_type(const void *vcp) { return ((vimconv_T *)vcp)->vc_type; }

