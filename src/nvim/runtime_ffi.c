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

#include "nvim/autocmd_defs.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/vars.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/userfunc.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_eval_defs.h"
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

/// Grow the execution stack garray by n entries.
void nvim_exestack_ga_grow(int n)
{
  ga_grow(&exestack, n);
}

/// Get an execution stack entry by index (returns pointer as opaque handle).
estack_T *nvim_exestack_get_entry(int idx)
{
  assert(idx >= 0 && idx < exestack.ga_len);
  return &((estack_T *)exestack.ga_data)[idx];
}

/// Get a pointer to the next unused slot in the exestack (at ga_len).
estack_T *nvim_exestack_get_next_slot(void)
{
  return &((estack_T *)exestack.ga_data)[exestack.ga_len];
}

/// Increment the exestack ga_len.
void nvim_exestack_inc_len(void)
{
  exestack.ga_len++;
}

/// Decrement the exestack ga_len (if > 1).
void nvim_exestack_dec_len(void)
{
  if (exestack.ga_len > 1) {
    exestack.ga_len--;
  }
}

/// Check if the execution stack has data.
bool nvim_exestack_has_data(void)
{
  return exestack.ga_data != NULL && exestack.ga_len > 0;
}

linenr_T nvim_estack_get_lnum(estack_T *entry)
{
  return entry->es_lnum;
}

void nvim_estack_set_lnum(estack_T *entry, linenr_T lnum)
{
  entry->es_lnum = lnum;
}

const char *nvim_estack_get_name(estack_T *entry)
{
  return entry->es_name;
}

void nvim_estack_set_name(estack_T *entry, char *name)
{
  entry->es_name = name;
}

int nvim_estack_get_type(estack_T *entry)
{
  return (int)entry->es_type;
}

void nvim_estack_set_type(estack_T *entry, int type)
{
  entry->es_type = (etype_T)type;
}

/// Set all fields of an estack entry at once.
void nvim_estack_set_entry(estack_T *entry, int type, char *name, linenr_T lnum)
{
  entry->es_type = (etype_T)type;
  entry->es_name = name;
  entry->es_lnum = lnum;
  entry->es_info.ufunc = NULL;
}

/// Get the ufunc from an estack entry's union.
ufunc_T *nvim_estack_get_info_ufunc(estack_T *entry)
{
  return entry->es_info.ufunc;
}

/// Set the ufunc in an estack entry's union.
void nvim_estack_set_info_ufunc(estack_T *entry, ufunc_T *ufunc)
{
  entry->es_info.ufunc = ufunc;
}

/// Get the aucmd from an estack entry's union.
AutoPatCmd *nvim_estack_get_info_aucmd(estack_T *entry)
{
  return entry->es_info.aucmd;
}

/// Get the name of a ufunc.
const char *nvim_ufunc_get_name(ufunc_T *fp)
{
  return fp->uf_name;
}

/// Get the expanded name of a ufunc (may be NULL).
const char *nvim_ufunc_get_name_exp(ufunc_T *fp)
{
  return fp->uf_name_exp;
}

/// Get the script context SID of a ufunc.
int nvim_ufunc_get_script_ctx_sid(ufunc_T *fp)
{
  return fp->uf_script_ctx.sc_sid;
}

/// Get the script context lnum of a ufunc.
linenr_T nvim_ufunc_get_script_ctx_lnum(ufunc_T *fp)
{
  return fp->uf_script_ctx.sc_lnum;
}

/// Get the script context SID of an aucmd.
int nvim_aucmd_get_script_ctx_sid(AutoPatCmd *apc)
{
  return apc->script_ctx.sc_sid;
}

/// Get the script context lnum of an aucmd.
linenr_T nvim_aucmd_get_script_ctx_lnum(AutoPatCmd *apc)
{
  return apc->script_ctx.sc_lnum;
}

/// Get the SOURCING_LNUM (lnum of the top exestack entry).
linenr_T nvim_get_sourcing_lnum_direct(void)
{
  return SOURCING_LNUM;
}

/// Duplicate a string using xstrdup.
char *nvim_runtime_xstrdup(const char *s)
{
  return xstrdup(s);
}

/// Format a stack entry with line number: "type_name name[lnum]dots"
/// Returns the number of bytes written.
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

/// Get the script_items garray length.
int nvim_script_items_get_len(void)
{
  return script_items.ga_len;
}

/// Get a script item by 1-based ID. Returns NULL if invalid.
scriptitem_T *nvim_script_item_get(int id)
{
  if (id <= 0 || id > script_items.ga_len) {
    return NULL;
  }
  return SCRIPT_ITEM(id);
}

/// Get a script item's name.
const char *nvim_scriptitem_get_name(scriptitem_T *si)
{
  return si->sn_name;
}

/// Check if a script item is Lua.
bool nvim_scriptitem_is_lua(scriptitem_T *si)
{
  return si->sn_lua;
}

/// Check if profiling is enabled for a script item.
bool nvim_scriptitem_get_prof_on(scriptitem_T *si)
{
  return si->sn_prof_on;
}

/// Get the SID from an estack entry's sctx (for Script/Modeline types).
int nvim_estack_get_sctx_sid(estack_T *entry)
{
  if (entry->es_type == ETYPE_SCRIPT || entry->es_type == ETYPE_MODELINE) {
    return entry->es_info.sctx ? entry->es_info.sctx->sc_sid : 0;
  }
  return 0;
}

/// For a ufunc/aucmd entry, get the SID of the defining script context.
/// Returns the SID, or 0 if not available.
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
/// Returns an xstrdup'd string or NULL.
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
dict_T *nvim_rt_dict_alloc_lock(void)
{
  return tv_dict_alloc_lock(VAR_FIXED);
}

/// Allocate a list with a known length.
list_T *nvim_rt_list_alloc(int count)
{
  return tv_list_alloc(count);
}

/// Add a funcref to a dict.
void nvim_rt_dict_add_func(dict_T *d, ufunc_T *fp)
{
  tv_dict_add_func(d, S_LEN("funcref"), fp);
}

/// Add an "event" string to a dict.
void nvim_rt_dict_add_event(dict_T *d, const char *event)
{
  tv_dict_add_str(d, S_LEN("event"), event);
}

/// Add a "lnum" number to a dict.
void nvim_rt_dict_add_lnum(dict_T *d, linenr_T lnum)
{
  tv_dict_add_nr(d, S_LEN("lnum"), lnum);
}

/// Add a "filepath" string to a dict.
void nvim_rt_dict_add_filepath(dict_T *d, const char *filepath)
{
  tv_dict_add_str(d, S_LEN("filepath"), filepath);
}

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

/// Set rettv to a list.
void nvim_rt_list_set_ret(void *rettv, list_T *l)
{
  tv_list_set_ret((typval_T *)rettv, l);
}

/// Call get_scriptname for a ufunc's script context.
/// Returns the script name (may be empty string for invalid SID).
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

/// Grow script_items garray by n entries.
void nvim_script_items_ga_grow(int n)
{
  ga_grow(&script_items, n);
}

/// Increment script_items ga_len.
void nvim_script_items_inc_len(void)
{
  script_items.ga_len++;
}

/// Set a script item at 1-based index.
void nvim_script_items_set_item(int id, scriptitem_T *si)
{
  SCRIPT_ITEM(id) = si;
}

/// Allocate a new scriptitem_T (xcalloc).
scriptitem_T *nvim_xcalloc_scriptitem(void)
{
  return xcalloc(1, sizeof(scriptitem_T));
}

/// Allocate script-local variables for a script.
void nvim_new_script_vars(int sid)
{
  new_script_vars(sid);
}

/// Set the sn_name field of a script item.
void nvim_scriptitem_set_name(scriptitem_T *si, char *name)
{
  si->sn_name = name;
}

/// Set the sn_prof_on field.
void nvim_scriptitem_set_prof_on(scriptitem_T *si, bool val)
{
  si->sn_prof_on = val;
}

/// Compare two filenames (path_fnamecmp wrapper).
int nvim_rt_path_fnamecmp(const char *a, const char *b)
{
  return path_fnamecmp(a, b);
}

/// Full implementation of get_scriptname, callable from Rust.
char *nvim_rt_get_scriptname(int sc_sid, uint64_t sc_chan, bool *should_free)
{
  sctx_T ctx = { .sc_sid = sc_sid, .sc_chan = sc_chan };
  return get_scriptname(ctx, should_free);
}

/// Get exarg_T addr_count.
int nvim_rt_exarg_get_addr_count(void *eap)
{
  return ((exarg_T *)eap)->addr_count;
}

/// Get exarg_T line2.
linenr_T nvim_rt_exarg_get_line2(void *eap)
{
  return ((exarg_T *)eap)->line2;
}

/// Get exarg_T arg pointer.
char *nvim_rt_exarg_get_arg(void *eap)
{
  return ((exarg_T *)eap)->arg;
}

/// Set exarg_T arg pointer.
void nvim_rt_exarg_set_arg(void *eap, char *arg)
{
  ((exarg_T *)eap)->arg = arg;
}

/// Check if the first byte of exarg_T arg is NUL.
bool nvim_exarg_arg_is_nul(void *eap)
{
  return *((exarg_T *)eap)->arg == NUL;
}

/// Call expand_env.
void nvim_rt_expand_env(char *src, char *dst, int dstlen)
{
  expand_env(src, dst, dstlen);
}

/// Call do_exedit.
void nvim_rt_do_exedit(void *eap)
{
  do_exedit((exarg_T *)eap, NULL);
}

/// Emit e_invarg error.
void nvim_rt_emsg_invarg(void)
{
  emsg(_(e_invarg));
}

/// Get the got_int flag.
bool nvim_rt_got_int(void)
{
  return got_int;
}

/// Get a pointer to NameBuff.
char *nvim_rt_get_namebuff(void)
{
  return NameBuff;
}

/// Get the MAXPATHL constant.
int nvim_rt_maxpathl(void)
{
  return MAXPATHL;
}

/// Get a pointer to IObuff.
char *nvim_rt_get_iobuff(void)
{
  return IObuff;
}

/// Get the IOSIZE constant.
int nvim_rt_iosize(void)
{
  return IOSIZE;
}

/// Call home_replace(NULL, name, buf, len, true).
void nvim_rt_home_replace(const char *name, char *buf, size_t len)
{
  home_replace(NULL, name, buf, len, true);
}

/// Format script entry: vim_snprintf(IObuff, IOSIZE, "%3d: %s", i, namebuff)
void nvim_rt_format_script_entry(int i, const char *namebuff)
{
  vim_snprintf(IObuff, (size_t)IOSIZE, "%3d: %s", i, namebuff);
}

/// Check message_filtered.
bool nvim_rt_message_filtered(const char *msg)
{
  return message_filtered(msg);
}

/// Output a newline.
void nvim_rt_msg_putchar_nl(void)
{
  msg_putchar('\n');
}

/// Output a translated string.
void nvim_rt_msg_outtrans(const char *msg)
{
  msg_outtrans(msg, 0, false);
}

/// Check for line break.
void nvim_rt_line_breakcheck(void)
{
  line_breakcheck();
}

/// Allocate a list and set it as the return value.
void nvim_rt_list_alloc_ret(void *rettv, int count)
{
  tv_list_alloc_ret((typval_T *)rettv, count);
}

/// Check for optional dict argument. Returns true if OK.
bool nvim_rt_check_for_opt_dict_arg(void *argvars)
{
  return tv_check_for_opt_dict_arg((typval_T *)argvars, 0) != FAIL;
}

/// Get the list from rettv.
list_T *nvim_rt_get_rettv_list(void *rettv)
{
  return ((typval_T *)rettv)->vval.v_list;
}

/// Check if first arg is a dict.
bool nvim_rt_argvars_is_dict(void *argvars)
{
  return ((typval_T *)argvars)[0].v_type == VAR_DICT;
}

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

/// Get "name" pattern string from dict in argvars. Returns allocated string or NULL.
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

/// Free a compiled regex.
void nvim_rt_vim_regfree(void *regmatch)
{
  if (regmatch != NULL) {
    regmatch_T *rm = (regmatch_T *)regmatch;
    vim_regfree(rm->regprog);
    xfree(rm);
  }
}

/// Allocate a dict.
dict_T *nvim_rt_p2_dict_alloc(void)
{
  return tv_dict_alloc();
}

/// Add a string to a dict.
void nvim_rt_dict_add_str(dict_T *d, const char *key, size_t keylen,
                          const char *val)
{
  tv_dict_add_str(d, key, keylen, val);
}

/// Add a number to a dict.
void nvim_rt_dict_add_nr(dict_T *d, const char *key, size_t keylen,
                         int64_t nr)
{
  tv_dict_add_nr(d, key, keylen, (varnumber_T)nr);
}

/// Add a bool to a dict.
void nvim_rt_dict_add_bool(dict_T *d, const char *key, size_t keylen,
                           bool val)
{
  tv_dict_add_bool(d, key, keylen, val ? kBoolVarTrue : kBoolVarFalse);
}

/// Append a dict to a list.
void nvim_rt_p2_tv_list_append_dict(list_T *l, dict_T *d)
{
  tv_list_append_dict(l, d);
}

/// Copy script variables dict. Returns new dict (empty if sn_vars is NULL).
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
                           dict_T *val)
{
  tv_dict_add_dict(d, key, keylen, val);
}

/// Add a list to a dict.
void nvim_rt_dict_add_list(dict_T *d, const char *key, size_t keylen,
                           list_T *val)
{
  tv_dict_add_list(d, key, keylen, val);
}

#if defined(BACKSLASH_IN_FILENAME)
/// Adjust slashes in a filename (Windows only).
void nvim_rt_slash_adjust(char *name)
{
  slash_adjust(name);
}
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
/// Returns NULL when done, or next iterator position.
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

/// Check if position is after a path separator.
bool nvim_rt_after_pathsep(const char *b, const char *s)
{
  return after_pathsep(b, s);
}

/// Count occurrences of a character in a buffer.
size_t nvim_rt_memcnt(const void *s, int c, size_t n)
{
  return memcnt(s, c, n);
}

/// Get the application name.
const char *nvim_rt_get_appname(void)
{
  return get_appname(false);
}

/// Get an XDG path variable. Returns allocated string.
char *nvim_rt_stdpaths_get_xdg_var(int type)
{
  return stdpaths_get_xdg_var((XDGVarType)type);
}

/// Get environment variable value. Returns allocated string.
char *nvim_rt_vim_getenv(const char *name)
{
  return vim_getenv(name);
}

/// Check if path is a directory.
bool nvim_rt_os_isdir(const char *name)
{
  return os_isdir(name);
}

/// Get default_lib_dir global.
const char *nvim_rt_get_default_lib_dir(void)
{
  return default_lib_dir;
}

/// Get prefix from exe path.
void nvim_rt_vim_get_prefix_from_exepath(char *buf)
{
  vim_get_prefix_from_exepath(buf);
}

/// Append path component.
int nvim_rt_append_path(char *path, const char *to_append, size_t max_len)
{
  return append_path(path, to_append, max_len);
}

/// Check if character is a path separator.
bool nvim_rt_vim_ispathsep(int c)
{
  return vim_ispathsep(c);
}

/// xmemcpyz wrapper.
void nvim_rt_xmemcpyz(void *dst, const void *src, size_t len)
{
  xmemcpyz(dst, src, len);
}

/// Get IOSIZE constant (already exists as nvim_rt_iosize, but alias for clarity).
size_t nvim_rt_get_iosize(void)
{
  return (size_t)IOSIZE;
}

_Static_assert(EW_DIR == 0x01, "EW_DIR must be 0x01");
_Static_assert(EW_FILE == 0x02, "EW_FILE must be 0x02");

/// Get the did_source_packages global.
bool nvim_rt_pkg_get_did_source_packages(void)
{
  return did_source_packages;
}

/// Set the did_source_packages global.
void nvim_rt_pkg_set_did_source_packages(bool val)
{
  did_source_packages = val;
}

/// Get the p_lpl (loadplugins) option value.
bool nvim_rt_pkg_get_p_lpl(void)
{
  return p_lpl;
}

/// Get exarg_T forceit field.
bool nvim_rt_pkg_exarg_get_forceit(void *eap)
{
  return ((exarg_T *)eap)->forceit;
}

/// Call fix_fname (returns allocated string).
char *nvim_rt_pkg_fix_fname(const char *fname)
{
  return fix_fname(fname);
}

/// Call vim_snprintf.
int nvim_rt_pkg_snprintf(char *buf, size_t len, const char *fmt, const char *arg)
{
  return vim_snprintf(buf, len, fmt, arg);
}

/// Call eval_to_number.
int64_t nvim_rt_pkg_eval_to_number(char *expr)
{
  return (int64_t)eval_to_number(expr, false);
}

/// Call do_cmdline_cmd.
void nvim_rt_pkg_do_cmdline_cmd(const char *cmd)
{
  do_cmdline_cmd(cmd);
}

/// Call TIME_MSG (time_msg if time_fd is set).
void nvim_rt_pkg_time_msg(const char *msg)
{
  TIME_MSG(msg);
}

_Static_assert(EXPAND_RUNTIME == 51, "EXPAND_RUNTIME must be 51");

/// Set xp_context and xp_pattern on an expand_T.
void nvim_rt_cmd_expand_set_context(void *xp, int context, const char *pattern)
{
  ((expand_T *)xp)->xp_context = context;
  ((expand_T *)xp)->xp_pattern = (char *)pattern;
}

/// Advance pointer by one multibyte character (MB_PTR_ADV).
int nvim_rt_utfc_ptr2len(const char *p)
{
  return utfc_ptr2len(p);
}

/// Check if character is a path separator (not colon).
bool nvim_rt_vim_ispathsep_nocolon(int c)
{
  return vim_ispathsep_nocolon(c);
}

/// Add a path separator to the end of the path if not present.
void nvim_rt_add_pathsep(char *p)
{
  add_pathsep(p);
}

/// Compare first n bytes of filenames (path-aware).
int nvim_rt_path_fnamencmp(const char *a, const char *b, size_t n)
{
  return path_fnamencmp(a, b, n);
}

/// Concatenate two path components into an allocated string.
char *nvim_rt_concat_fnames(const char *fname1, const char *fname2, bool sep)
{
  return concat_fnames((char *)fname1, fname2, sep);
}

/// try_malloc: malloc that returns NULL on failure.
void *nvim_rt_try_malloc(size_t n)
{
  return try_malloc(n);
}

/// Set the 'runtimepath' option to a new value.
void nvim_rt_set_runtimepath(const char *new_rtp)
{
  set_option_value_give_err(kOptRuntimepath, CSTR_AS_OPTVAL(new_rtp), 0);
}

/// get_past_head: skip drive letter/UNC prefix on Windows, identity on Unix.
char *nvim_rt_get_past_head(const char *path)
{
  return get_past_head((char *)path);
}

/// fix_fname: canonicalize path (resolve symlinks, etc).
char *nvim_rt_fix_fname(const char *fname)
{
  return fix_fname((char *)fname);
}
