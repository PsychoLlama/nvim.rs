#pragma once

#include <assert.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>

#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/gettext_defs.h"
#include "nvim/hashtab.h"
#include "nvim/lib/queue_defs.h"
#include "nvim/macros_defs.h"
#include "nvim/mbyte_defs.h"  // IWYU pragma: keep
#include "nvim/message.h"
#include "nvim/types_defs.h"

#include "eval/typval.h.inline.generated.h"

// In a hashtab item "hi_key" points to "di_key" in a dictitem.
// This avoids adding a pointer to the hashtab item.

/// Convert a hashitem pointer to a dictitem pointer
#define TV_DICT_HI2DI(hi) \
  ((dictitem_T *)((hi)->hi_key - offsetof(dictitem_T, di_key)))

/// Increase reference count for a given list
///
/// Does nothing for NULL lists.
///
/// @param[in,out]  l  List to modify.
static inline void tv_list_ref(list_T *const l)
  FUNC_ATTR_ALWAYS_INLINE
{
  if (l == NULL) {
    return;
  }
  l->lv_refcount++;
}

/// Set a list as the return value.  Increments the reference count.
///
/// @param[out]  tv  Object to receive the list
/// @param[in,out]  l  List to pass to the object
static inline void tv_list_set_ret(typval_T *const tv, list_T *const l)
  FUNC_ATTR_ALWAYS_INLINE FUNC_ATTR_NONNULL_ARG(1)
{
  tv->v_type = VAR_LIST;
  tv->vval.v_list = l;
  tv_list_ref(l);
}

/// Get list lock status
///
/// Returns VAR_FIXED for NULL lists.
///
/// @param[in]  l  List to check.
static inline VarLockStatus tv_list_locked(const list_T *const l)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  if (l == NULL) {
    return VAR_FIXED;
  }
  return l->lv_lock;
}

/// Set list lock status
///
/// May only “set” VAR_FIXED for NULL lists.
///
/// @param[out]  l  List to modify.
/// @param[in]  lock  New lock status.
static inline void tv_list_set_lock(list_T *const l, const VarLockStatus lock)
{
  if (l == NULL) {
    assert(lock == VAR_FIXED);
    return;
  }
  l->lv_lock = lock;
}

/// Set list copyID
///
/// Does not expect NULL list, be careful.
///
/// @param[out]  l  List to modify.
/// @param[in]  copyid  New copyID.
static inline void tv_list_set_copyid(list_T *const l, const int copyid)
  FUNC_ATTR_NONNULL_ALL
{
  l->lv_copyID = copyid;
}

/// Get the number of items in a list
///
/// @param[in]  l  List to check.
static inline int tv_list_len(const list_T *const l)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  if (l == NULL) {
    return 0;
  }
  return l->lv_len;
}

/// Get list copyID
///
/// Does not expect NULL list, be careful.
///
/// @param[in]  l  List to check.
static inline int tv_list_copyid(const list_T *const l)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
  return l->lv_copyID;
}

/// Get latest list copy
///
/// Gets lv_copylist field assigned by tv_list_copy() earlier.
///
/// Does not expect NULL list, be careful.
///
/// @param[in]  l  List to check.
static inline list_T *tv_list_latest_copy(const list_T *const l)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
  return l->lv_copylist;
}

/// Normalize index: that is, return either -1 or non-negative index
///
/// @param[in]  l  List to index. Used to get length.
/// @param[in]  n  List index, possibly negative.
///
/// @return -1 or list index in range [0, tv_list_len(l)).
static inline int tv_list_uidx(const list_T *const l, int n)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  // Negative index is relative to the end.
  if (n < 0) {
    n += tv_list_len(l);
  }

  // Check for index out of range.
  if (n < 0 || n >= tv_list_len(l)) {
    return -1;
  }
  return n;
}

/// Check whether list has watchers
///
/// E.g. is referenced by a :for loop.
///
/// @param[in]  l  List to check.
///
/// @return true if there are watchers, false otherwise.
static inline bool tv_list_has_watchers(const list_T *const l)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  return l && l->lv_watch;
}

/// Get first list item
///
/// @param[in]  l  List to get item from.
///
/// @return List item or NULL in case of an empty list.
static inline listitem_T *tv_list_first(const list_T *const l)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  if (l == NULL) {
    return NULL;
  }
  return l->lv_first;
}

/// Get last list item
///
/// @param[in]  l  List to get item from.
///
/// @return List item or NULL in case of an empty list.
static inline listitem_T *tv_list_last(const list_T *const l)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  if (l == NULL) {
    return NULL;
  }
  return l->lv_last;
}

/// Set a dictionary as the return value
///
/// @param[out]  tv  Object to receive the dictionary
/// @param[in,out]  d  Dictionary to pass to the object
static inline void tv_dict_set_ret(typval_T *const tv, dict_T *const d)
  FUNC_ATTR_ALWAYS_INLINE FUNC_ATTR_NONNULL_ARG(1)
{
  tv->v_type = VAR_DICT;
  tv->vval.v_dict = d;
  if (d != NULL) {
    d->dv_refcount++;
  }
}

/// Get the number of items in a Dictionary
///
/// @param[in]  d  Dictionary to check.
static inline long tv_dict_len(const dict_T *const d)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  if (d == NULL) {
    return 0;
  }
  return (long)d->dv_hashtab.ht_used;
}

/// Check if dictionary is watched
///
/// @param[in]  d  Dictionary to check.
///
/// @return true if there is at least one watcher.
static inline bool tv_dict_is_watched(const dict_T *const d)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  return d && !QUEUE_EMPTY(&d->watchers);
}

/// Set a blob as the return value.
///
/// Increments the reference count.
///
/// @param[out]  tv  Object to receive the blob.
/// @param[in,out]  b  Blob to pass to the object.
static inline void tv_blob_set_ret(typval_T *const tv, blob_T *const b)
  FUNC_ATTR_ALWAYS_INLINE FUNC_ATTR_NONNULL_ARG(1)
{
  tv->v_type = VAR_BLOB;
  tv->vval.v_blob = b;
  if (b != NULL) {
    b->bv_refcount++;
  }
}

/// Get the length of the data in the blob, in bytes.
///
/// @param[in]  b  Blob to check.
static inline int tv_blob_len(const blob_T *const b)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  if (b == NULL) {
    return 0;
  }
  return b->bv_ga.ga_len;
}

/// Get the byte at index `idx` in the blob.
///
/// @param[in]  b  Blob to index. Cannot be NULL.
/// @param[in]  idx  Index in a blob. Must be valid.
///
/// @return Byte value at the given index.
static inline uint8_t tv_blob_get(const blob_T *const b, int idx)
  FUNC_ATTR_ALWAYS_INLINE FUNC_ATTR_NONNULL_ALL FUNC_ATTR_WARN_UNUSED_RESULT
{
  return ((uint8_t *)b->bv_ga.ga_data)[idx];
}

/// Store the byte `c` at index `idx` in the blob.
///
/// @param[in]  b  Blob to index. Cannot be NULL.
/// @param[in]  idx  Index in a blob. Must be valid.
/// @param[in]  c  Value to store.
static inline void tv_blob_set(blob_T *const blob, int idx, uint8_t c)
  FUNC_ATTR_ALWAYS_INLINE FUNC_ATTR_NONNULL_ALL
{
  ((uint8_t *)blob->bv_ga.ga_data)[idx] = c;
}

/// Initialize Vimscript object
///
/// Initializes to unlocked VAR_UNKNOWN object.
///
/// @param[out]  tv  Object to initialize.
static inline void tv_init(typval_T *const tv)
{
  if (tv != NULL) {
    memset(tv, 0, sizeof(*tv));
  }
}

/// Empty string
///
/// Needed for hack which allows not allocating empty string and still not
/// crashing when freeing it.
extern const char *const tv_empty_string;

/// Specifies that free_unref_items() function has (not) been entered
extern bool tv_in_free_unref_items;

/// Iterate over a list
///
/// @param  modifier  Modifier: expected to be const or nothing, volatile should
///                   also work if you have any uses for the volatile list.
/// @param[in]  l  List to iterate over.
/// @param  li  Name of the variable with current listitem_T entry.
/// @param  code  Cycle body.
#define TV_LIST_ITER_MOD(modifier, l, li, code) \
  do { \
    modifier list_T *const l_ = (l); \
    if (l_ != NULL) { \
      for (modifier listitem_T *li = l_->lv_first; \
           li != NULL; li = li->li_next) { \
        code \
      } \
    } \
  } while (0)

/// Iterate over a list
///
/// To be used when you need to modify list or values you iterate over, use
/// #TV_LIST_ITER_CONST if you don’t.
///
/// @param[in]  l  List to iterate over.
/// @param  li  Name of the variable with current listitem_T entry.
/// @param  code  Cycle body.
#define TV_LIST_ITER(l, li, code) \
  TV_LIST_ITER_MOD( , l, li, code)

/// Iterate over a list
///
/// To be used when you don’t need to modify list or values you iterate over,
/// use #TV_LIST_ITER if you do.
///
/// @param[in]  l  List to iterate over.
/// @param  li  Name of the variable with current listitem_T entry.
/// @param  code  Cycle body.
#define TV_LIST_ITER_CONST(l, li, code) \
  TV_LIST_ITER_MOD(const, l, li, code)

// Below macros are macros to avoid duplicating code for functionally identical
// const and non-const function variants.

/// Get typval_T out of list item
///
/// @param[in]  li  List item to get typval_T from, must not be NULL.
///
/// @return Pointer to typval_T.
#define TV_LIST_ITEM_TV(li) (&(li)->li_tv)

/// Get next list item given the current one
///
/// @param[in]  l  List to get item from.
/// @param[in]  li  List item to get typval_T from.
///
/// @return Pointer to the next item or NULL.
#define TV_LIST_ITEM_NEXT(l, li) ((li)->li_next)

/// Get previous list item given the current one
///
/// @param[in]  l  List to get item from.
/// @param[in]  li  List item to get typval_T from.
///
/// @return Pointer to the previous item or NULL.
#define TV_LIST_ITEM_PREV(l, li) ((li)->li_prev)
// List argument is not used currently, but it is a must for lists implemented
// as a pair (size(in list), array) without terminator - basically for lists on
// top of kvec.

/// Iterate over a dictionary
///
/// @param[in]  d  Dictionary to iterate over.
/// @param  di  Name of the variable with current dictitem_T entry.
/// @param  code  Cycle body.
#define TV_DICT_ITER(d, di, code) \
  HASHTAB_ITER(&(d)->dv_hashtab, di##hi_, { \
    { \
      dictitem_T *const di = TV_DICT_HI2DI(di##hi_); \
      { \
        code \
      } \
    } \
  })

/// Get the float value
///
/// Raises an error if object is not number or floating-point.
///
/// @param[in]  tv  Vimscript object to get value from.
/// @param[out]  ret_f  Location where resulting float is stored.
///
/// @return true in case of success, false if tv is not a number or float.
static inline bool tv_get_float_chk(const typval_T *const tv, float_T *const ret_f)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_WARN_UNUSED_RESULT
{
  if (tv->v_type == VAR_FLOAT) {
    *ret_f = tv->vval.v_float;
    return true;
  }
  if (tv->v_type == VAR_NUMBER) {
    *ret_f = (float_T)tv->vval.v_number;
    return true;
  }
  semsg("%s", _("E808: Number or Float required"));
  return false;
}

/// Compute the `DictWatcher` address from a QUEUE node.
///
/// This only exists for .asan-blacklist (ASAN doesn't handle QUEUE_DATA pointer
/// arithmetic).
static inline DictWatcher *tv_dict_watcher_node_data(QUEUE *q)
  FUNC_ATTR_ALWAYS_INLINE FUNC_ATTR_NONNULL_ALL FUNC_ATTR_NONNULL_RET
    FUNC_ATTR_NO_SANITIZE_ADDRESS FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  return QUEUE_DATA(q, DictWatcher, node);
}

/// Check whether given typval_T contains a function
///
/// That is, whether it contains VAR_FUNC or VAR_PARTIAL.
///
/// @param[in]  tv  Typval to check.
///
/// @return True if it is a function or a partial, false otherwise.
static inline bool tv_is_func(const typval_T tv)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_ALWAYS_INLINE FUNC_ATTR_CONST
{
  return tv.v_type == VAR_FUNC || tv.v_type == VAR_PARTIAL;
}

/// Specify that argument needs to be translated
///
/// Used for size_t length arguments to avoid calling gettext() and strlen()
/// unless needed.
#define TV_TRANSLATE (SIZE_MAX)

/// Specify that argument is a NUL-terminated C string
///
/// Used for size_t length arguments to avoid calling strlen() unless needed.
#define TV_CSTRING (SIZE_MAX - 1)

#ifdef UNIT_TESTING
// Do not use enum constants, see commit message.
EXTERN const size_t kTVCstring INIT( = TV_CSTRING);
EXTERN const size_t kTVTranslate INIT( = TV_TRANSLATE);
#endif

// Functions implemented in Rust via #[export_name] in src/nvim-rs/typval/src/lib.rs

// List and blob operations
extern listitem_T *tv_list_find(list_T *l, int n);
extern int tv_list_idx_of_item(const list_T *l, const listitem_T *item);
extern void tv_list_reverse(list_T *l);
extern bool tv_blob_equal(const blob_T *b1, const blob_T *b2);

// Type validation
extern bool tv_check_str_or_nr(const typval_T *tv);
extern bool tv_check_num(const typval_T *tv);
extern bool tv_check_str(const typval_T *tv);

// Argument type-checking functions
extern int tv_check_for_string_arg(const typval_T *args, int idx);
extern int tv_check_for_nonempty_string_arg(const typval_T *args, int idx);
extern int tv_check_for_opt_string_arg(const typval_T *args, int idx);
extern int tv_check_for_number_arg(const typval_T *args, int idx);
extern int tv_check_for_opt_number_arg(const typval_T *args, int idx);
extern int tv_check_for_float_or_nr_arg(const typval_T *args, int idx);
extern int tv_check_for_bool_arg(const typval_T *args, int idx);
extern int tv_check_for_opt_bool_arg(const typval_T *args, int idx);
extern int tv_check_for_blob_arg(const typval_T *args, int idx);
extern int tv_check_for_list_arg(const typval_T *args, int idx);
extern int tv_check_for_dict_arg(const typval_T *args, int idx);
extern int tv_check_for_nonnull_dict_arg(const typval_T *args, int idx);
extern int tv_check_for_opt_dict_arg(const typval_T *args, int idx);
extern int tv_check_for_string_or_number_arg(const typval_T *args, int idx);
extern int tv_check_for_buffer_arg(const typval_T *args, int idx);
extern int tv_check_for_lnum_arg(const typval_T *args, int idx);
extern int tv_check_for_string_or_list_arg(const typval_T *args, int idx);
extern int tv_check_for_opt_string_or_list_arg(const typval_T *args, int idx);
extern int tv_check_for_string_or_list_or_blob_arg(const typval_T *args, int idx);
extern int tv_check_for_string_or_list_or_dict_arg(const typval_T *args, int idx);
extern int tv_check_for_string_or_func_arg(const typval_T *args, int idx);
extern int tv_check_for_list_or_blob_arg(const typval_T *args, int idx);

// Lock-checking functions (migrated to Rust)
extern bool value_check_lock(VarLockStatus lock, const char *name, size_t name_len);
extern bool tv_check_lock(const typval_T *tv, const char *name, size_t name_len);
extern bool tv_islocked(const typval_T *tv);

// Lock/unlock operations (migrated to Rust, Phase 3)
extern void tv_item_lock(typval_T *tv, int deep, bool lock, bool check_refcount);

// Float getter (migrated to Rust)
extern float_T tv_get_float(const typval_T *tv);

// Get functions (migrated to Rust, Phase 1)
extern varnumber_T tv_get_number(const typval_T *tv);
extern varnumber_T tv_get_number_chk(const typval_T *tv, bool *ret_error);
extern varnumber_T tv_get_bool(const typval_T *tv);
extern varnumber_T tv_get_bool_chk(const typval_T *tv, bool *ret_error);
extern linenr_T tv_get_lnum(const typval_T *tv);
extern linenr_T tv_get_lnum_buf(const typval_T *tv, const buf_T *buf);
extern const char *tv_get_string_buf_chk(const typval_T *tv, char *buf);
extern const char *tv_get_string_chk(const typval_T *tv);
extern const char *tv_get_string(const typval_T *tv);
extern const char *tv_get_string_buf(const typval_T *tv, char *buf);

// List infrastructure functions (migrated to Rust, Phase 5)
extern listitem_T *tv_list_item_remove(list_T *l, listitem_T *item);
extern void tv_list_watch_add(list_T *l, listwatch_T *lw);
extern void tv_list_watch_remove(list_T *l, listwatch_T *lwrem);
extern list_T *tv_list_alloc(ptrdiff_t len);
extern void tv_list_init_static(list_T *l);
extern void tv_list_free_contents(list_T *l);
extern void tv_list_free_list(list_T *l);
extern void tv_list_free(list_T *l);
extern void tv_list_unref(list_T *l);
extern void tv_list_drop_items(list_T *l, listitem_T *item, listitem_T *item2);
extern void tv_list_remove_items(list_T *l, listitem_T *item, listitem_T *item2);
extern void tv_list_move_items(list_T *l, listitem_T *item, listitem_T *item2, list_T *tgt_l, int cnt);
extern void tv_list_insert(list_T *l, listitem_T *ni, listitem_T *item);
extern void tv_list_insert_tv(list_T *l, typval_T *tv, listitem_T *item);
extern list_T *tv_list_alloc_ret(typval_T *ret_tv, ptrdiff_t len);

// Phase 6: list operations and VimL functions (migrated to Rust)
extern void tv_list_extend(list_T *l1, list_T *l2, listitem_T *bef);
extern int tv_list_concat(list_T *l1, list_T *l2, typval_T *tv);
extern void f_has_key(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Phase 6c: range/slice/flatten/remove (migrated to Rust)
extern listitem_T *tv_list_check_range_index_one(list_T *l, int *n1, bool quiet);
extern int tv_list_check_range_index_two(list_T *l, int *n1, const listitem_T *li1, int *n2, bool quiet);
extern int tv_list_slice_or_index(list_T *list, bool range, varnumber_T n1_arg, varnumber_T n2_arg, bool exclusive, typval_T *rettv, bool verbose);
extern void tv_list_flatten(list_T *list, listitem_T *first, int64_t maxitems, int64_t maxdepth);
extern void tv_list_remove(typval_T *argvars, typval_T *rettv, const char *arg_errmsg);

// Phase 6d: assign range (migrated to Rust)
extern int tv_list_assign_range(list_T *dest, list_T *src, int idx1_arg, int idx2, bool empty_idx2,
                                const char *op, const char *varname);

// Phase 6e: list copy, items (migrated to Rust)
extern list_T *tv_list_copy(const vimconv_T *conv, list_T *orig, bool deep, int copyID);
extern void f_items(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Phase 6f: dict remove (migrated to Rust)
extern void tv_dict_remove(typval_T *argvars, typval_T *rettv, const char *arg_errmsg);

// Phase 6g: f_keys, f_values (migrated to Rust)
extern void f_keys(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_values(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Phase 6h: tv_dict_set_keys_readonly (migrated to Rust)
extern void tv_dict_set_keys_readonly(dict_T *dict);

// Phase 6i: tv_dict_get_string (migrated to Rust)
extern char *tv_dict_get_string(const dict_T *d, const char *key, bool save);

// Phase 6j: tv_dict_to_env (migrated to Rust)
extern char **tv_dict_to_env(dict_T *denv);

// List append functions (migrated to Rust, Phase 5)
extern void tv_list_append(list_T *l, listitem_T *item);
extern void tv_list_append_tv(list_T *l, typval_T *tv);
extern void tv_list_append_list(list_T *l, list_T *itemlist);
extern void tv_list_append_dict(list_T *l, dict_T *dict);
extern void tv_list_append_string(list_T *l, const char *str, ssize_t len);
extern void tv_list_append_allocated_string(list_T *l, char *str);
extern void tv_list_append_number(list_T *l, varnumber_T n);

// Dict item and dict alloc/add functions (migrated to Rust, Phase 3)
extern dictitem_T *tv_dict_item_alloc_len(const char *key, size_t key_len);
extern dictitem_T *tv_dict_item_alloc(const char *key);
extern void tv_dict_item_free(dictitem_T *item);
extern dictitem_T *tv_dict_item_copy(dictitem_T *di);
extern void tv_dict_item_remove(dict_T *dict, dictitem_T *item);
extern dict_T *tv_dict_alloc(void);
extern dict_T *tv_dict_alloc_lock(VarLockStatus lock);
extern void tv_dict_alloc_ret(typval_T *ret_tv);
extern dictitem_T *tv_dict_find(const dict_T *d, const char *key, ptrdiff_t len);
extern int tv_dict_add(dict_T *d, dictitem_T *item);
extern int tv_dict_add_list(dict_T *d, const char *key, size_t key_len, list_T *list);
extern int tv_dict_add_tv(dict_T *d, const char *key, size_t key_len, typval_T *tv);
extern int tv_dict_add_dict(dict_T *d, const char *key, size_t key_len, dict_T *dict);
extern int tv_dict_add_nr(dict_T *d, const char *key, size_t key_len, varnumber_T nr);
extern int tv_dict_add_float(dict_T *d, const char *key, size_t key_len, float_T nr);
extern int tv_dict_add_bool(dict_T *d, const char *key, size_t key_len, BoolVarValue val);
extern int tv_dict_add_str(dict_T *d, const char *key, size_t key_len, const char *val);
extern int tv_dict_add_str_len(dict_T *d, const char *key, size_t key_len, const char *val, int len);
extern int tv_dict_add_allocated_str(dict_T *d, const char *key, size_t key_len, char *val);
extern int tv_dict_add_func(dict_T *d, const char *key, size_t key_len, ufunc_T *fp);

// Blob functions (migrated to Rust, Phase 2)
extern blob_T *tv_blob_alloc(void);
extern void tv_blob_free(blob_T *blob);
extern void tv_blob_unref(blob_T *blob);
extern blob_T *tv_blob_alloc_ret(typval_T *ret_tv);
extern void tv_blob_copy(blob_T *from, typval_T *to);
extern void f_blob2list(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_list2blob(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Dict lookup functions (migrated to Rust, Phase 4)
extern bool tv_dict_has_key(const dict_T *d, const char *key);
extern int tv_dict_get_tv(dict_T *d, const char *key, typval_T *rettv);
extern varnumber_T tv_dict_get_number(const dict_T *d, const char *key);
extern varnumber_T tv_dict_get_number_def(const dict_T *d, const char *key, int def);
extern varnumber_T tv_dict_get_bool(const dict_T *d, const char *key, int def);
extern const char *tv_dict_get_string_buf(const dict_T *d, const char *key, char *numbuf);
extern const char *tv_dict_get_string_buf_chk(const dict_T *d, const char *key, ptrdiff_t key_len,
                                              char *numbuf, const char *def);

// Callback operations (migrated to Rust, Phase 1 of typval.c migration)
extern bool tv_callback_equal(const Callback *cb1, const Callback *cb2);
extern void callback_free(Callback *callback);
extern void callback_put(Callback *cb, typval_T *tv);
extern void callback_copy(Callback *dest, Callback *src);
extern char *callback_to_string(Callback *cb, Arena *arena);

// Core typval operations (migrated to Rust, Phase 2 of typval.c migration)
extern void tv_free(typval_T *tv);
extern void tv_copy(const typval_T *from, typval_T *to);
extern bool tv_equal(typval_T *tv1, typval_T *tv2, bool ic);

#include "eval/typval.h.generated.h"
