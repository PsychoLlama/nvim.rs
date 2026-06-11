// fuzzy_shim.c: C accessor functions for the fuzzy matching Rust crate.
//
// These thin wrappers allow the Rust fuzzy crate to access typval, list, dict,
// and callback internals without pulling in the entire C type system.

#include <stdbool.h>
#include <stdint.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/fuzzy.h"
#include "nvim/fuzzy_shim.h"
#include "nvim/mbyte.h"
#include "nvim/memory.h"
#include "nvim/message.h"

// ---------------------------------------------------------------------------
// typval_T accessors (argvars[]/rettv)
// ---------------------------------------------------------------------------

/// Return v_type of argvars[idx]. Returns VAR_UNKNOWN if tv is NULL.
int nvim_fuzzy_argvars_type(const void *argvars, int idx)
{
  if (argvars == NULL) {
    return VAR_UNKNOWN;
  }
  return (int)((const typval_T *)argvars)[idx].v_type;
}

/// Return vval.v_list of argvars[idx], or NULL if not a list.
void *nvim_fuzzy_argvars_list(const void *argvars, int idx)
{
  if (argvars == NULL) {
    return NULL;
  }
  const typval_T *tv = &((const typval_T *)argvars)[idx];
  return tv->v_type == VAR_LIST ? tv->vval.v_list : NULL;
}

/// Return vval.v_dict of argvars[idx], or NULL if not a dict.
void *nvim_fuzzy_argvars_dict(const void *argvars, int idx)
{
  if (argvars == NULL) {
    return NULL;
  }
  const typval_T *tv = &((const typval_T *)argvars)[idx];
  return tv->v_type == VAR_DICT ? tv->vval.v_dict : NULL;
}

/// Return tv_get_string(&argvars[idx]), or NULL if argvars is NULL.
const char *nvim_fuzzy_argvars_string(const void *argvars, int idx)
{
  if (argvars == NULL) {
    return NULL;
  }
  return tv_get_string(&((const typval_T *)argvars)[idx]);
}

/// Return vval.v_string of argvars[idx], or NULL if not VAR_STRING.
const char *nvim_fuzzy_argvars_vstring(const void *argvars, int idx)
{
  if (argvars == NULL) {
    return NULL;
  }
  const typval_T *tv = &((const typval_T *)argvars)[idx];
  return tv->v_type == VAR_STRING ? tv->vval.v_string : NULL;
}

/// Check that argvars[idx] is a non-null dict.  Returns FAIL on error (also
/// emits semsg), OK on success.
int nvim_fuzzy_check_nonnull_dict_arg(const void *argvars, int idx)
{
  return tv_check_for_nonnull_dict_arg((const typval_T *)argvars, idx);
}

/// Emit the "not a list" error for matchfuzzy/matchfuzzypos.
void nvim_fuzzy_semsg_listarg(bool retmatchpos)
{
  semsg(_(e_listarg), retmatchpos ? "matchfuzzypos()" : "matchfuzzy()");
}

/// Emit the "invalid argument" error with the string representation of argvars[idx].
void nvim_fuzzy_semsg_invarg2(const void *argvars, int idx)
{
  semsg(_(e_invarg2), tv_get_string(&((const typval_T *)argvars)[idx]));
}

/// Emit e_invargNval for key + stringified tv.
void nvim_fuzzy_semsg_invarg_nval(const char *argname, const void *di_tv)
{
  semsg(_(e_invargNval), argname, tv_get_string((const typval_T *)di_tv));
}

/// Emit e_invargval for name.
void nvim_fuzzy_semsg_invargval(const char *name)
{
  semsg(_(e_invargval), name);
}

// ---------------------------------------------------------------------------
// tv_list_alloc_ret wrapper (allocates list and sets rettv)
// ---------------------------------------------------------------------------

/// Allocate a list and set rettv to point to it.  Returns the list pointer.
/// count: 3 for matchfuzzypos, -1 for kListLenUnknown for matchfuzzy.
void *nvim_fuzzy_tv_list_alloc_ret(void *rettv, int count)
{
  return (void *)tv_list_alloc_ret((typval_T *)rettv, (ptrdiff_t)count);
}

// ---------------------------------------------------------------------------
// list_T accessors
// ---------------------------------------------------------------------------

/// Return tv_list_len(l).
int nvim_fuzzy_list_len(const void *l)
{
  return l == NULL ? 0 : tv_list_len((const list_T *)l);
}

/// Return the first list item, or NULL.
void *nvim_fuzzy_list_first(const void *l)
{
  return l == NULL ? NULL : tv_list_first((const list_T *)l);
}

/// Advance to the next list item in list l.  Returns NULL at end.
void *nvim_fuzzy_list_item_next(const void *l, const void *li)
{
  if (l == NULL || li == NULL) {
    return NULL;
  }
  return (void *)TV_LIST_ITEM_NEXT((const list_T *)l, (const listitem_T *)li);
}

/// Return v_type of TV_LIST_ITEM_TV(li).
int nvim_fuzzy_list_item_type(const void *li)
{
  if (li == NULL) {
    return VAR_UNKNOWN;
  }
  return (int)TV_LIST_ITEM_TV((const listitem_T *)li)->v_type;
}

/// Return vval.v_string of TV_LIST_ITEM_TV(li), or NULL if not VAR_STRING.
const char *nvim_fuzzy_list_item_string(const void *li)
{
  if (li == NULL) {
    return NULL;
  }
  const typval_T *tv = TV_LIST_ITEM_TV((const listitem_T *)li);
  return tv->v_type == VAR_STRING ? tv->vval.v_string : NULL;
}

/// Return vval.v_dict of TV_LIST_ITEM_TV(li), or NULL if not VAR_DICT.
void *nvim_fuzzy_list_item_dict(const void *li)
{
  if (li == NULL) {
    return NULL;
  }
  const typval_T *tv = TV_LIST_ITEM_TV((const listitem_T *)li);
  return tv->v_type == VAR_DICT ? tv->vval.v_dict : NULL;
}

/// tv_list_find wrapper. Returns the listitem at position n (negative=from end).
void *nvim_fuzzy_list_find(void *l, int n)
{
  return l == NULL ? NULL : (void *)tv_list_find((list_T *)l, n);
}

/// Append a tv via tv_list_append_tv.
/// Appends TV_LIST_ITEM_TV(item_li) to list dst.
void nvim_fuzzy_list_append_item_tv(void *dst, const void *item_li)
{
  if (dst == NULL || item_li == NULL) {
    return;
  }
  tv_list_append_tv((list_T *)dst, TV_LIST_ITEM_TV((const listitem_T *)item_li));
}

/// Append a sublist to list dst (tv_list_append_list).
void nvim_fuzzy_list_append_list(void *dst, void *sublist)
{
  if (dst == NULL) {
    return;
  }
  tv_list_append_list((list_T *)dst, (list_T *)sublist);
}

/// Append an integer number to list dst.
void nvim_fuzzy_list_append_number(void *dst, int64_t n)
{
  if (dst == NULL) {
    return;
  }
  tv_list_append_number((list_T *)dst, (varnumber_T)n);
}

/// Allocate a new list with kListLenMayKnow.
void *nvim_fuzzy_list_alloc_mayknow(void)
{
  return (void *)tv_list_alloc(kListLenMayKnow);
}

/// Get the sub-list from a listitem that holds a list.  Returns NULL if not a list item.
void *nvim_fuzzy_listitem_get_list(const void *li)
{
  if (li == NULL) {
    return NULL;
  }
  const typval_T *tv = TV_LIST_ITEM_TV((const listitem_T *)li);
  return tv->v_type == VAR_LIST ? tv->vval.v_list : NULL;
}

// ---------------------------------------------------------------------------
// dict_T accessors
// ---------------------------------------------------------------------------

/// tv_dict_get_string wrapper.  Returns NULL if not found or not a string.
char *nvim_fuzzy_dict_get_string(const void *dict, const char *key, bool allocate)
{
  return dict == NULL ? NULL : tv_dict_get_string((const dict_T *)dict, key, allocate);
}

/// Find a dictitem by key; return pointer to di_tv, or NULL if not found.
void *nvim_fuzzy_dict_find_tv(const void *dict, const char *key)
{
  if (dict == NULL) {
    return NULL;
  }
  dictitem_T *di = tv_dict_find((const dict_T *)dict, key, -1);
  return di == NULL ? NULL : (void *)&di->di_tv;
}

/// Return v_type of dictitem found by key, or VAR_UNKNOWN if not found.
int nvim_fuzzy_dict_find_type(const void *dict, const char *key)
{
  if (dict == NULL) {
    return VAR_UNKNOWN;
  }
  const dictitem_T *di = tv_dict_find((const dict_T *)dict, key, -1);
  return di == NULL ? (int)VAR_UNKNOWN : (int)di->di_tv.v_type;
}

/// Return vval.v_string of dictitem found by key, or NULL.
const char *nvim_fuzzy_dict_find_string(const void *dict, const char *key)
{
  if (dict == NULL) {
    return NULL;
  }
  const dictitem_T *di = tv_dict_find((const dict_T *)dict, key, -1);
  if (di == NULL || di->di_tv.v_type != VAR_STRING) {
    return NULL;
  }
  return di->di_tv.vval.v_string;
}

/// tv_get_number_chk on di->di_tv; sets *error to true on type error.
int64_t nvim_fuzzy_dict_find_number(const void *dict, const char *key, bool *error)
{
  if (dict == NULL) {
    if (error) {
      *error = true;
    }
    return 0;
  }
  const dictitem_T *di = tv_dict_find((const dict_T *)dict, key, -1);
  if (di == NULL) {
    return 0;
  }
  bool err = false;
  varnumber_T n = tv_get_number_chk(&di->di_tv, &err);
  if (error) {
    *error = err;
  }
  return (int64_t)n;
}

/// tv_dict_has_key wrapper.
bool nvim_fuzzy_dict_has_key(const void *dict, const char *key)
{
  return dict != NULL && tv_dict_has_key((const dict_T *)dict, key);
}

/// tv_dict_get_callback wrapper.
/// cb_out must point to a Callback (zero-initialised is OK = CALLBACK_NONE).
/// Returns false (and emits error) on failure.
bool nvim_fuzzy_dict_get_callback(const void *dict, const char *key, void *cb_out)
{
  if (dict == NULL || cb_out == NULL) {
    return false;
  }
  return tv_dict_get_callback((dict_T *)dict, key, -1, (Callback *)cb_out);
}

// ---------------------------------------------------------------------------
// Callback accessors
// ---------------------------------------------------------------------------

/// Allocate a zeroed Callback on the heap (caller owns and must free).
void *nvim_fuzzy_callback_alloc_none(void)
{
  Callback *cb = xcalloc(1, sizeof(Callback));
  cb->type = kCallbackNone;
  return (void *)cb;
}

/// Free a heap-allocated Callback and call callback_free to release inner refs.
void nvim_fuzzy_callback_free(void *cb)
{
  if (cb == NULL) {
    return;
  }
  callback_free((Callback *)cb);
  xfree(cb);
}

/// Return true if the Callback is kCallbackNone.
bool nvim_fuzzy_callback_is_none(const void *cb)
{
  return cb == NULL || ((const Callback *)cb)->type == kCallbackNone;
}

/// Invoke callback with a single dict argument; put result in a heap-allocated
/// typval_T.  Returns the rettv pointer on success, NULL on failure.
/// The returned typval_T must be cleared with nvim_fuzzy_tv_clear_and_free.
void *nvim_fuzzy_callback_call_dict(void *cb, void *dict)
{
  if (cb == NULL || dict == NULL) {
    return NULL;
  }
  dict_T *d = (dict_T *)dict;
  typval_T *rettv = xcalloc(1, sizeof(typval_T));
  rettv->v_type = VAR_UNKNOWN;

  d->dv_refcount++;
  typval_T argv[2];
  argv[0].v_type = VAR_DICT;
  argv[0].vval.v_dict = d;
  argv[1].v_type = VAR_UNKNOWN;

  bool ok = callback_call((Callback *)cb, 1, argv, rettv);
  tv_dict_unref(d);

  if (!ok) {
    xfree(rettv);
    return NULL;
  }
  return (void *)rettv;
}

/// Return v_type of a heap-allocated typval_T returned by nvim_fuzzy_callback_call_dict.
int nvim_fuzzy_tv_type(const void *tv)
{
  return tv == NULL ? (int)VAR_UNKNOWN : (int)((const typval_T *)tv)->v_type;
}

/// Return vval.v_string of the typval_T, or NULL if not a string.
const char *nvim_fuzzy_tv_string(const void *tv)
{
  if (tv == NULL) {
    return NULL;
  }
  const typval_T *t = (const typval_T *)tv;
  return t->v_type == VAR_STRING ? t->vval.v_string : NULL;
}

/// Clear and free a heap-allocated typval_T.
void nvim_fuzzy_tv_clear_and_free(void *tv)
{
  if (tv == NULL) {
    return;
  }
  tv_clear((typval_T *)tv);
  xfree(tv);
}

// ---------------------------------------------------------------------------
// Misc
// ---------------------------------------------------------------------------

/// ascii_iswhite check (for position list building).
bool nvim_fuzzy_ascii_iswhite(int c)
{
  return ascii_iswhite(c);
}

/// utf_ptr2char: return the Unicode codepoint at p.
int nvim_fuzzy_utf_ptr2char(const char *p)
{
  return utf_ptr2char(p);
}

/// MB_PTR_ADV: advance pointer by one UTF-8 char and return new pointer.
const char *nvim_fuzzy_mb_ptr_adv(const char *p)
{
  MB_PTR_ADV(p);
  return p;
}

/// xstrdup wrapper.
char *nvim_fuzzy_xstrdup(const char *s)
{
  return s == NULL ? NULL : xstrdup(s);
}

/// xfree wrapper for char pointers.
void nvim_fuzzy_xfree(void *p)
{
  xfree(p);
}

#include "fuzzy_shim.c.generated.h"
