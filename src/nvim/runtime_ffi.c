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
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/userfunc.h"
#include "nvim/ex_eval_defs.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/memory.h"
#include "nvim/pos_defs.h"
#include "nvim/runtime.h"
#include "nvim/runtime_defs.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"

#include "runtime_ffi.c.generated.h"

// =============================================================================
// Static assertions for constants used in Rust code
// =============================================================================

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

// =============================================================================
// Phase 1: Execution stack accessors
// =============================================================================

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

// --- estack_T field accessors ---

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

// --- ufunc_T field accessors ---

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

// --- AutoPatCmd field accessors ---

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

// --- estack_sfile helpers ---

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

/// Call vim_snprintf.
int nvim_runtime_snprintf(char *buf, size_t len, const char *fmt, ...)
  FUNC_ATTR_UNUSED
{
  // This is a simplified version - callers use specific formatting functions below
  return 0;
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

// --- Script item accessors ---

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

// --- estack_T source context accessors ---

/// Get the SID from an estack entry's sctx (for Script/Modeline types).
int nvim_estack_get_sctx_sid(estack_T *entry)
{
  if (entry->es_type == ETYPE_SCRIPT || entry->es_type == ETYPE_MODELINE) {
    return entry->es_info.sctx ? entry->es_info.sctx->sc_sid : 0;
  }
  return 0;
}

// --- estack_sfile: script context from entry ---

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

// =============================================================================
// Phase 1: Typval accessors for stacktrace
// =============================================================================

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
