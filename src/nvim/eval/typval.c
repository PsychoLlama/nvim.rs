#include <assert.h>
#include <lauxlib.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <uv.h>

#include "nvim/ascii_defs.h"
#include "nvim/assert_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/encode.h"
#include "nvim/eval/executor.h"
#include "nvim/eval/gc.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/typval_encode.h"
#include "nvim/eval/userfunc.h"
#include "nvim/eval/vars.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/hashtab.h"
#include "nvim/hashtab_defs.h"
#include "nvim/lib/queue_defs.h"
#include "nvim/lua/executor.h"
#include "nvim/macros_defs.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memory.h"
#include "nvim/memory_defs.h"
#include "nvim/message.h"
#include "nvim/os/input.h"
#include "nvim/pos_defs.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"


// Rust FFI functions
extern bool rs_func_equal(typval_T *tv1, typval_T *tv2, bool ic);
extern bool rs_callback_from_typval(Callback *callback, const typval_T *arg);
extern char *rs_partial_name(partial_T *pt);

// Rust typval functions (migrated in Phase 2)
extern bool tv_list_equal(list_T *l1, list_T *l2, bool ic);
extern bool tv2bool(const typval_T *tv);


// DictListType enum deleted (Phase 8): tv_dict2list migrated to Rust

#include "eval/typval.c.generated.h"

static const char e_variable_nested_too_deep_for_unlock[]
  = N_("E743: Variable nested too deep for (un)lock");
static const char e_using_invalid_value_as_string[]
  = N_("E908: Using an invalid value as a String");
static const char e_string_required_for_argument_nr[]
  = N_("E1174: String required for argument %d");
static const char e_non_empty_string_required_for_argument_nr[]
  = N_("E1175: Non-empty string required for argument %d");
static const char e_dict_required_for_argument_nr[]
  = N_("E1206: Dictionary required for argument %d");
static const char e_number_required_for_argument_nr[]
  = N_("E1210: Number required for argument %d");
static const char e_list_required_for_argument_nr[]
  = N_("E1211: List required for argument %d");
static const char e_bool_required_for_argument_nr[]
  = N_("E1212: Bool required for argument %d");
static const char e_float_or_number_required_for_argument_nr[]
  = N_("E1219: Float or Number required for argument %d");
static const char e_string_or_number_required_for_argument_nr[]
  = N_("E1220: String or Number required for argument %d");
static const char e_string_or_list_required_for_argument_nr[]
  = N_("E1222: String or List required for argument %d");
static const char e_string_list_or_dict_required_for_argument_nr[]
  = N_("E1225: String, List or Dictionary required for argument %d");
static const char e_list_or_blob_required_for_argument_nr[]
  = N_("E1226: List or Blob required for argument %d");
static const char e_blob_required_for_argument_nr[]
  = N_("E1238: Blob required for argument %d");
static const char e_invalid_value_for_blob_nr[]
  = N_("E1239: Invalid value for blob: %d");
static const char e_string_list_or_blob_required_for_argument_nr[]
  = N_("E1252: String, List or Blob required for argument %d");
static const char e_string_or_function_required_for_argument_nr[]
  = N_("E1256: String or function required for argument %d");
static const char e_non_null_dict_required_for_argument_nr[]
  = N_("E1297: Non-NULL Dictionary required for argument %d");

bool tv_in_free_unref_items = false;

// TODO(ZyX-I): Remove DICT_MAXNEST, make users be non-recursive instead

#define DICT_MAXNEST 100

const char *const tv_empty_string = "";

//{{{1 Lists
//{{{2 List item

// tv_list_item_alloc deleted (Phase 8): replaced by nvim_list_item_alloc C accessor

// tv_list_item_remove, tv_list_watch_add, tv_list_watch_remove, tv_list_watch_fix
// migrated to Rust (Phase 5)

//{{{2 Alloc/free

// tv_list_alloc migrated to Rust (Phase 5)

// tv_list_init_static10 migrated to Rust (Phase 3)

// tv_list_init_static migrated to Rust (Phase 5)

// tv_list_free_contents, tv_list_free_list, tv_list_free, tv_list_unref
// migrated to Rust (Phase 5)

//{{{2 Add/remove

// tv_list_drop_items, tv_list_remove_items, tv_list_move_items
// migrated to Rust (Phase 5)

// tv_list_insert, tv_list_insert_tv migrated to Rust (Phase 5)

// tv_list_append, tv_list_append_tv migrated to Rust (Phase 5)

/// Like tv_list_append_tv(), but tv is moved to a list
///
/// This means that it is no longer valid to use contents of the typval_T after
/// function exits. A pointer is returned to the allocated typval which can be used
typval_T *tv_list_append_owned_tv(list_T *const l, typval_T tv)
  FUNC_ATTR_NONNULL_ALL
{
  listitem_T *const li = nvim_list_item_alloc();
  *TV_LIST_ITEM_TV(li) = tv;
  tv_list_append(l, li);
  return TV_LIST_ITEM_TV(li);
}

// tv_list_append_list, tv_list_append_dict, tv_list_append_string,
// tv_list_append_allocated_string, tv_list_append_number migrated to Rust (Phase 5)

/// Append a typval by pointer (accessor for Rust decode module, Phase 1).
/// Copies *tv into a new list item; caller must not use tv after this.
void nvim_tv_list_append_typval_ptr(list_T *l, typval_T *tv)
  FUNC_ATTR_NONNULL_ALL
{
  tv_list_append_owned_tv(l, *tv);
}

/// Append VAR_UNKNOWN to list and return pointer to the new item's typval
/// (accessor for Rust mpack decoder, Phase 3).
typval_T *nvim_tv_list_append_unknown_and_get(list_T *l)
  FUNC_ATTR_NONNULL_ALL
{
  return tv_list_append_owned_tv(l, (typval_T) { .v_type = VAR_UNKNOWN });
}

//{{{2 Operations on the whole list

/// Make a copy of list
///
/// @param[in]  conv  If non-NULL, then all internal strings will be converted.
///                   Only used when `deep` is true.
/// @param[in]  orig  Original list to copy.
/// @param[in]  deep  If false, then shallow copy will be done.
/// @param[in]  copyID  See var_item_copy().
///
/// @return Copied list. May be NULL in case original list is NULL or some
///         failure happens. The refcount of the new list is set to 1.
// tv_list_copy migrated to Rust (Phase 6e)

// tv_list_check_range_index_one, tv_list_check_range_index_two migrated to Rust (Phase 6c)

/// Assign values from list "src" into a range of "dest".
/// "idx1_arg" is the index of the first item in "dest" to be replaced.
/// "idx2" is the index of last item to be replaced, but when "empty_idx2" is
/// true then replace all items after "idx1".
/// "op" is the operator, normally "=" but can be "+=" and the like.
/// "varname" is used for error messages.
/// Returns OK or FAIL.
// tv_list_assign_range migrated to Rust (Phase 6d)

// tv_list_flatten migrated to Rust (Phase 6c)

// tv_list2items migrated to Rust (Phase 6e)

// tv_string2items migrated to Rust (Phase 6e)

// tv_list_extend, tv_list_concat migrated to Rust (Phase 6)

// tv_list_slice, tv_list_slice_or_index migrated to Rust (Phase 6c)

// list_join_inner, tv_list_join migrated to Rust (Phase 3)

// f_join migrated to Rust (Phase 6)
// f_list2str migrated to Rust (Phase 6)

// tv_list_remove migrated to Rust (Phase 6c)

// sortinfo_T, ListSortItem, ListSorter, sortinfo, item_compare, item_compare2,
// do_sort, do_uniq, parse_sort_uniq_args, do_sort_uniq, f_sort, f_uniq
// migrated to Rust (Phase 6)

// tv_list_equal, tv_list_find_nr, tv_list_find_str migrated to Rust (Phase 2)

//{{{2 Indexing/searching

// tv_list_find_index deleted (dead code, Phase 6e)

//{{{1 Dictionaries
//{{{2 Dictionary watchers

// tv_callback_equal, callback_free, callback_put, callback_copy, callback_to_string
// migrated to Rust (Phase 1)

// tv_dict_watcher_add, tv_dict_watcher_remove, tv_dict_watcher_notify
// migrated to Rust (Phase 7); declared in typval.h

// tv_dict_watcher_matches migrated to Rust (Phase 5)
// tv_dict_watcher_free migrated to Rust (Phase 5)
// (declarations now in typval.h)

//{{{2 Dictionary item
// tv_dict_item_alloc_len, tv_dict_item_alloc, tv_dict_item_free,
// tv_dict_item_copy, tv_dict_item_remove migrated to Rust (Phase 3)

//{{{2 Alloc/free

// tv_dict_alloc migrated to Rust (Phase 3)

// tv_dict_free_contents, tv_dict_free_dict, tv_dict_free, tv_dict_unref migrated to Rust (Phase 4)

//{{{2 Indexing/searching

// tv_dict_find migrated to Rust (Phase 3)

// tv_dict_has_key: migrated to Rust (nvim-rs/typval)
// tv_dict_get_tv: migrated to Rust (nvim-rs/typval)
// tv_dict_get_number: migrated to Rust (nvim-rs/typval)
// tv_dict_get_number_def: migrated to Rust (nvim-rs/typval)
// tv_dict_get_bool: migrated to Rust (nvim-rs/typval)

// tv_dict_to_env migrated to Rust (Phase 6j)

// tv_dict_get_string migrated to Rust (Phase 6i)

// tv_dict_get_string_buf: migrated to Rust (nvim-rs/typval)
// tv_dict_get_string_buf_chk: migrated to Rust (nvim-rs/typval)

// tv_dict_get_callback migrated to Rust (Phase 5)
extern bool tv_dict_get_callback(dict_T *d, const char *key, ptrdiff_t key_len, Callback *result);

// tv_dict_wrong_func_name migrated to Rust (Phase 5)
extern int tv_dict_wrong_func_name(dict_T *d, typval_T *tv, const char *name);

//{{{2 dict_add*
// tv_dict_add, tv_dict_add_list, tv_dict_add_tv, tv_dict_add_dict,
// tv_dict_add_nr, tv_dict_add_float, tv_dict_add_bool, tv_dict_add_str,
// tv_dict_add_str_len, tv_dict_add_allocated_str, tv_dict_add_func
// migrated to Rust (Phase 3)

//{{{2 Operations on the whole dict

// tv_dict_clear deleted (dead code, Phase 6h)

// tv_dict_extend, tv_dict_equal, tv_dict_copy migrated to Rust (Phase 4)

/// Set all existing keys in "dict" as read-only.
///
// tv_dict_set_keys_readonly migrated to Rust (Phase 6h)

//{{{1 Blobs
//{{{2 Alloc/free
// tv_blob_alloc, tv_blob_free, tv_blob_unref migrated to Rust (Phase 2)

//{{{2 Operations on the whole blob
// tv_blob_slice_or_index, tv_blob_check_index, tv_blob_check_range,
// tv_blob_set_range, tv_blob_set_append, tv_blob_remove migrated to Rust (Phase 1)

// f_blob2list, f_list2blob migrated to Rust (Phase 2)

//{{{1 Generic typval operations
//{{{2 Init/alloc/clear
//{{{3 Alloc

// tv_list_alloc_ret migrated to Rust (Phase 5)

// tv_dict_alloc_lock, tv_dict_alloc_ret migrated to Rust (Phase 3)

// tv_dict2list migrated to Rust (Phase 8)

// f_items migrated to Rust (Phase 6e)

// f_keys, f_values migrated to Rust (Phase 6g; Phase 8: dict case inlined into Rust)

// f_has_key migrated to Rust (Phase 6)

// tv_dict_remove migrated to Rust (Phase 6f)

// tv_blob_alloc_ret, tv_blob_copy migrated to Rust (Phase 2)

// tv_clear, encode-nothing framework migrated to Rust (Phase 2)

// tv_free, tv_copy migrated to Rust (Phase 2 of typval.c migration)

//{{{2 Locks

// tv_item_lock migrated to Rust (Phase 3 of typval.c migration)

// tv_islocked: migrated to Rust (nvim-rs/typval)

// tv_check_lock, value_check_lock, value_check_lock_impl migrated to Rust (Phase 3 / nvim-rs/typval)

// tv_equal migrated to Rust (Phase 2 of typval.c migration)

//{{{2 Get

// tv_get_number, tv_get_number_chk, tv_get_bool, tv_get_bool_chk migrated to Rust (Phase 1)
// tv_get_lnum, tv_get_lnum_buf migrated to Rust (Phase 1)

// tv_get_float migrated to Rust (nvim-rs/typval); declared in typval.h

// Rust FFI accessor functions for error message reporting
// These wrap semsg() calls since semsg is variadic and hard to call from Rust.

/// Get a pointer to args[idx] in a typval array.
/// Used by Rust to safely index into typval arrays.
const void *nvim_typval_array_get(const typval_T *args, int idx) { return &args[idx]; }

// nvim_typval_error_* wrappers deleted (Phase 9): Rust calls emsg/semsg directly
// via typval_err_* inline helpers in nvim-rs/typval/src/lib.rs.

// tv_get_string_buf_chk, tv_get_string_chk, tv_get_string, tv_get_string_buf
// migrated to Rust (Phase 1)

// tv2bool migrated to Rust (Phase 2)

// Rust accessor functions for opaque typval_T handle pattern

/// Get the v_type field from a typval (accessor for Rust).
int nvim_tv_get_type(const typval_T *tv) { return (int)tv->v_type; }

/// Get the v_number field from a typval (accessor for Rust).
int64_t nvim_tv_get_number(const typval_T *tv) { return tv->vval.v_number; }

/// Get the v_bool field from a typval (accessor for Rust).
int nvim_tv_get_bool(const typval_T *tv) { return (int)tv->vval.v_bool; }

/// Get the v_float field from a typval (accessor for Rust).
double nvim_tv_get_float(const typval_T *tv) { return tv->vval.v_float; }

/// Get the v_string field from a typval (accessor for Rust).
const char *nvim_tv_get_string_ptr(const typval_T *tv) { return tv->vval.v_string; }

/// Check if v_list is NULL (accessor for Rust).
int nvim_tv_list_is_null(const typval_T *tv) { return tv->vval.v_list == NULL; }

/// Check if v_dict is NULL (accessor for Rust).
int nvim_tv_dict_is_null(const typval_T *tv) { return tv->vval.v_dict == NULL; }

/// Check if v_blob is NULL (accessor for Rust).
int nvim_tv_blob_is_null(const typval_T *tv) { return tv->vval.v_blob == NULL; }

/// Check if v_partial is NULL (accessor for Rust).
int nvim_tv_partial_is_null(const typval_T *tv) { return tv->vval.v_partial == NULL; }

/// Get the v_list pointer from a typval (accessor for Rust).
list_T *nvim_tv_get_list(const typval_T *tv) { return tv->vval.v_list; }

/// Get the v_dict pointer from a typval (accessor for Rust).
dict_T *nvim_tv_get_dict(const typval_T *tv) { return tv->vval.v_dict; }

/// Get the v_blob pointer from a typval (accessor for Rust).
blob_T *nvim_tv_get_blob(const typval_T *tv) { return tv->vval.v_blob; }

// Typval setter functions for Rust

/// Set v_type to VAR_NUMBER and v_number (setter for Rust).
void nvim_tv_set_number(typval_T *tv, int64_t n)
{
  tv->v_type = VAR_NUMBER;
  tv->vval.v_number = (varnumber_T)n;
}

/// Set v_type to VAR_FLOAT and v_float (setter for Rust).
void nvim_tv_set_float(typval_T *tv, double f)
{
  tv->v_type = VAR_FLOAT;
  tv->vval.v_float = f;
}

/// Get number from typval with error checking (accessor for Rust).
/// This wrapper calls tv_get_number_chk and updates the error pointer.
int64_t nvim_tv_get_number_chk(const typval_T *tv, bool *error) { return tv_get_number_chk(tv, error); }

/// Get float from typval with error checking (accessor for Rust).
/// Returns true on success and stores result in *ret.
/// Returns false and sets *ret to 0.0 on error.
bool nvim_tv_get_float_chk(const typval_T *tv, double *ret)
{
  float_T f;
  bool ok = tv_get_float_chk(tv, &f);
  *ret = ok ? f : 0.0;
  return ok;
}

// String accessor functions for Rust

/// Get string from typval with type conversion (accessor for Rust).
/// Uses a static buffer for conversions, so result may be overwritten by next call.
/// @param tv  The typval to get the string from.
/// @param out_len  If non-NULL, receives the string length.
/// @return Pointer to the string (may be static buffer).
const char *nvim_tv_get_string(const typval_T *tv, size_t *out_len)
{
  const char *s = tv_get_string(tv);
  if (out_len != NULL) {
    *out_len = strlen(s);
  }
  return s;
}

/// Get string from typval with error checking (accessor for Rust).
/// @param tv  The typval to get the string from.
/// @param out_len  If non-NULL, receives the string length.
/// @return Pointer to the string, or NULL on error.
const char *nvim_tv_get_string_chk(const typval_T *tv, size_t *out_len)
{
  const char *s = tv_get_string_chk(tv);
  if (s == NULL) {
    if (out_len != NULL) {
      *out_len = 0;
    }
    return NULL;
  }
  if (out_len != NULL) {
    *out_len = strlen(s);
  }
  return s;
}

/// Set typval to a string (takes ownership, setter for Rust).
/// @param tv  The typval to set.
/// @param s   The string to set (NULL-terminated, will be owned by typval).
void nvim_tv_set_string(typval_T *tv, char *s)
{
  tv->v_type = VAR_STRING;
  tv->vval.v_string = s;
}

/// Set typval to a copy of a string (setter for Rust).
/// @param tv   The typval to set.
/// @param s    The string to copy (NULL-terminated).
/// @param len  Length of the string (or -1 to use strlen).
void nvim_tv_set_string_copy(typval_T *tv, const char *s, int len)
{
  tv->v_type = VAR_STRING;
  if (s == NULL) {
    tv->vval.v_string = NULL;
  } else if (len < 0) {
    tv->vval.v_string = xstrdup(s);
  } else {
    tv->vval.v_string = xmemdupz(s, (size_t)len);
  }
}

/// Allocate a string of given size and set it as typval value (accessor for Rust).
/// @param tv   The typval to set.
/// @param len  Length of the string to allocate (not including NUL terminator).
/// @return Pointer to the allocated string buffer, or NULL on failure.
char *nvim_tv_alloc_string(typval_T *tv, size_t len)
{
  tv->v_type = VAR_STRING;
  tv->vval.v_string = xmalloc(len + 1);
  tv->vval.v_string[len] = NUL;
  return tv->vval.v_string;
}

// List accessor functions for Rust

/// Allocate a new list item (accessor for Rust, wraps static tv_list_item_alloc).
listitem_T *nvim_list_item_alloc(void) { return xmalloc(sizeof(listitem_T)); }

/// Increment lv_len on a list (accessor for Rust).
void nvim_list_inc_len(list_T *l) { l->lv_len++; }

/// Decrement lv_len on a list (accessor for Rust).
void nvim_list_dec_len(list_T *l) { l->lv_len--; }

/// Set lv_len on a list (accessor for Rust).
void nvim_list_set_len(list_T *l, int len) { l->lv_len = len; }

// nvim_tv_clear deleted (Phase 2): tv_clear migrated to Rust

/// Free a list item (xfree wrapper for Rust).
void nvim_list_item_free(listitem_T *li) { xfree(li); }

/// Append wrapper: tv_list_append (self-contained for Rust, avoids circular call).
void nvim_tv_list_append_item(list_T *l, listitem_T *item)
{
  if (l->lv_last == NULL) {
    l->lv_first = item;
    l->lv_last = item;
    item->li_prev = NULL;
  } else {
    l->lv_last->li_next = item;
    item->li_prev = l->lv_last;
    l->lv_last = item;
  }
  l->lv_len++;
  item->li_next = NULL;
}

/// Get lv_len from a list (accessor for Rust).
int nvim_list_get_len(const list_T *l) { return l->lv_len; }

/// Get lv_lock from a list (accessor for Rust).
int nvim_list_get_lock(const list_T *l) { return (int)l->lv_lock; }

/// Check if lv_watch is non-NULL (accessor for Rust).
int nvim_list_has_watchers(const list_T *l) { return l->lv_watch != NULL; }

/// Get lv_first from a list (accessor for Rust).
listitem_T *nvim_list_get_first(const list_T *l) { return l->lv_first; }

/// Get lv_last from a list (accessor for Rust).
listitem_T *nvim_list_get_last(const list_T *l) { return l->lv_last; }

// Dict accessor functions for Rust

/// Get dv_hashtab.ht_used from a dict (accessor for Rust).
size_t nvim_dict_get_ht_used(const dict_T *d) { return d->dv_hashtab.ht_used; }

/// Get dv_lock from a dict (accessor for Rust).
int nvim_dict_get_lock(const dict_T *d) { return (int)d->dv_lock; }

/// Check if dict has watchers (accessor for Rust).
int nvim_dict_has_watchers(const dict_T *d) { return !QUEUE_EMPTY(&d->watchers); }

/// Get dict length (number of items) (accessor for Rust).
int nvim_dict_get_len(const dict_T *d)
{
  if (d == NULL) {
    return 0;
  }
  return (int)d->dv_hashtab.ht_used;
}

/// Get dv_copyID from a dict (accessor for Rust).
int nvim_dict_get_copyid(const dict_T *d) { return d->dv_copyID; }

/// Get dv_used_next from a dict (accessor for Rust).
dict_T *nvim_dict_get_used_next(const dict_T *d) { return d->dv_used_next; }

// nvim_dictitem_get_tv: defined in vars.c

/// Get di_key from a dictitem as a C string (accessor for Rust).
const char *nvim_dictitem_get_key(const dictitem_T *di) { return di->di_key; }

// nvim_dict_find migrated to Rust (Phase 3 of f39b5673);
// uses nvim_dict_hash_find / nvim_dict_hash_find_len / nvim_hashitem_is_empty accessors.

/// Get string representation of a typval into buf (accessor for Rust).
/// Returns NULL on type error, empty string for empty string.
const char *nvim_tv_get_string_buf(const typval_T *tv, char *buf) { return tv_get_string_buf(tv, buf); }

/// Get string or NULL on type error (accessor for Rust).
const char *nvim_tv_get_string_buf_chk(const typval_T *tv, char *buf) { return tv_get_string_buf_chk(tv, buf); }

/// Get number from typval (accessor for Rust).
int64_t nvim_tv_get_number_simple(const typval_T *tv) { return (int64_t)tv_get_number(tv); }

/// Get bool from typval (accessor for Rust).
int nvim_tv_get_bool_simple(const typval_T *tv) { return (int)tv_get_bool(tv); }

/// Get lv_used_next from a list (accessor for Rust).
list_T *nvim_list_get_used_next(const list_T *l) { return l->lv_used_next; }

/// Set tv_in_free_unref_items global (accessor for Rust).
void nvim_set_tv_in_free_unref_items(int val) { tv_in_free_unref_items = (bool)val; }

// Blob accessor functions for Rust

/// Get blob length from typval (accessor for Rust).
int nvim_tv_blob_len(const typval_T *tv)
{
  if (tv == NULL || tv->v_type != VAR_BLOB || tv->vval.v_blob == NULL) {
    return 0;
  }
  return tv_blob_len(tv->vval.v_blob);
}

/// Get bv_ga.ga_len from a blob (accessor for Rust).
int nvim_blob_get_len(const blob_T *b) { return b->bv_ga.ga_len; }

/// Get bv_lock from a blob (accessor for Rust).
int nvim_blob_get_lock(const blob_T *b) { return (int)b->bv_lock; }

/// Get a byte from blob at index (accessor for Rust).
uint8_t nvim_blob_get_byte(const blob_T *b, int idx) { return ((uint8_t *)b->bv_ga.ga_data)[idx]; }

/// Set a byte in blob at index (accessor for Rust).
void nvim_blob_set_byte(blob_T *b, int idx, uint8_t c) { ((uint8_t *)b->bv_ga.ga_data)[idx] = c; }

/// Get the raw data pointer from a blob's garray (accessor for Rust).
uint8_t *nvim_blob_get_ga_data(blob_T *b) { return (uint8_t *)b->bv_ga.ga_data; }

/// Set ga_len in a blob's garray (accessor for Rust).
void nvim_blob_set_ga_len(blob_T *b, int len) { b->bv_ga.ga_len = len; }

/// Call ga_grow on a blob's garray (accessor for Rust).
void nvim_blob_ga_grow(blob_T *b, int n) { ga_grow(&b->bv_ga, n); }

/// Set v_type=VAR_BLOB and vval.v_blob (accessor for Rust).
/// Also increments the blob's refcount.
void nvim_tv_set_blob(typval_T *tv, blob_T *b) { tv_blob_set_ret(tv, b); }

// value_check_lock_impl and nvim_value_check_lock_translated: migrated to Rust (Phase 3)

/// Get translated error string for "Value is locked" (accessor for Rust).
const char *nvim_gettext_value_locked(void) { return _(N_("E741: Value is locked: %.*s")); }
/// Get translated error string for "Cannot change value" (accessor for Rust).
const char *nvim_gettext_value_fixed(void) { return _(N_("E742: Cannot change value of %.*s")); }
/// Get translated "Unknown" string (accessor for Rust).
const char *nvim_gettext_unknown(void) { return _("Unknown"); }

// Phase 9: translated string accessors for static-local error strings.
// These are needed because the static strings are defined with N_() and translated at runtime
// via _(). They must be accessed through C so gettext can do the lookup.
const char *nvim_gettext_e_string_required_for_argument_nr(void) { return _(e_string_required_for_argument_nr); }
const char *nvim_gettext_e_nonempty_string_required_for_argument_nr(void) { return _(e_non_empty_string_required_for_argument_nr); }
const char *nvim_gettext_e_number_required_for_argument_nr(void) { return _(e_number_required_for_argument_nr); }
const char *nvim_gettext_e_float_or_number_required_for_argument_nr(void) { return _(e_float_or_number_required_for_argument_nr); }
const char *nvim_gettext_e_bool_required_for_argument_nr(void) { return _(e_bool_required_for_argument_nr); }
const char *nvim_gettext_e_blob_required_for_argument_nr(void) { return _(e_blob_required_for_argument_nr); }
const char *nvim_gettext_e_list_required_for_argument_nr(void) { return _(e_list_required_for_argument_nr); }
const char *nvim_gettext_e_dict_required_for_argument_nr(void) { return _(e_dict_required_for_argument_nr); }
const char *nvim_gettext_e_nonnull_dict_required_for_argument_nr(void) { return _(e_non_null_dict_required_for_argument_nr); }
const char *nvim_gettext_e_string_or_number_required_for_argument_nr(void) { return _(e_string_or_number_required_for_argument_nr); }
const char *nvim_gettext_e_string_or_list_required_for_argument_nr(void) { return _(e_string_or_list_required_for_argument_nr); }
const char *nvim_gettext_e_string_list_or_blob_required_for_argument_nr(void) { return _(e_string_list_or_blob_required_for_argument_nr); }
const char *nvim_gettext_e_string_list_or_dict_required_for_argument_nr(void) { return _(e_string_list_or_dict_required_for_argument_nr); }
const char *nvim_gettext_e_string_or_func_required_for_argument_nr(void) { return _(e_string_or_function_required_for_argument_nr); }
const char *nvim_gettext_e_list_or_blob_required_for_argument_nr(void) { return _(e_list_or_blob_required_for_argument_nr); }
const char *nvim_gettext_e_using_invalid_value_as_string(void) { return _(e_using_invalid_value_as_string); }
const char *nvim_gettext_e_variable_nested_too_deep_for_unlock(void) { return _(e_variable_nested_too_deep_for_unlock); }
const char *nvim_gettext_e_invalid_value_for_blob_nr(void) { return _(e_invalid_value_for_blob_nr); }

// nvim_semsg_blobidx, nvim_emsg_blob_wrong_bytes, nvim_emsg_float_* deleted (Phase 9):
// Rust calls emsg/semsg directly via typval_err_* inline helpers.

/// Thin C stub for nvim_value_check_lock: calls the Rust value_check_lock.
/// Required because other crates (eval_exec, vars) still call this by name.
bool nvim_value_check_lock(VarLockStatus lock, const char *name, size_t name_len)
{
  return value_check_lock((int)lock, name, name_len);
}

/// Set lv_lock on a list (accessor for Rust tv_item_lock).
void nvim_list_set_lock(list_T *l, int lock) { l->lv_lock = (VarLockStatus)lock; }

/// Set bv_lock on a blob (accessor for Rust tv_item_lock).
void nvim_blob_set_lock(blob_T *b, int lock) { b->bv_lock = (VarLockStatus)lock; }

/// Get v_lock from a typval (accessor for Rust).
int nvim_tv_get_v_lock(const typval_T *tv) { return (int)tv->v_lock; }

// Listitem accessor functions for Rust

/// Get li_next from a listitem (accessor for Rust).
listitem_T *nvim_listitem_get_next(const listitem_T *li) { return li->li_next; }

/// Get li_prev from a listitem (accessor for Rust).
listitem_T *nvim_listitem_get_prev(const listitem_T *li) { return li->li_prev; }

/// Get pointer to li_tv from a listitem (accessor for Rust).
typval_T *nvim_listitem_get_tv(listitem_T *li) { return &li->li_tv; }

// List cache accessor functions for Rust (for tv_list_find optimization)

/// Get lv_idx from a list (accessor for Rust).
int nvim_list_get_idx(const list_T *l) { return l->lv_idx; }

/// Get lv_idx_item from a list (accessor for Rust).
listitem_T *nvim_list_get_idx_item(const list_T *l) { return l->lv_idx_item; }

/// Set lv_idx on a list (accessor for Rust).
void nvim_list_set_idx(list_T *l, int idx) { l->lv_idx = idx; }

/// Set lv_idx_item on a list (accessor for Rust).
void nvim_list_set_idx_item(list_T *l, listitem_T *item) { l->lv_idx_item = item; }

/// Get lv_copyID from a list (accessor for Rust).
int nvim_list_get_copyid(const list_T *l) { return l->lv_copyID; }

/// Get lv_copylist from a list (accessor for Rust).
list_T *nvim_list_get_copylist(const list_T *l) { return l->lv_copylist; }

/// Set lv_copyID on a list (accessor for Rust).
void nvim_list_set_copyid(list_T *l, int copyid) { l->lv_copyID = copyid; }

/// Set lv_copylist on a list (accessor for Rust).
void nvim_list_set_copylist(list_T *l, list_T *copy) { l->lv_copylist = copy; }

/// Set lv_first on a list (accessor for Rust).
void nvim_list_set_first(list_T *l, listitem_T *item) { l->lv_first = item; }

/// Set lv_last on a list (accessor for Rust).
void nvim_list_set_last(list_T *l, listitem_T *item) { l->lv_last = item; }

/// Set li_next on a listitem (accessor for Rust).
void nvim_listitem_set_next(listitem_T *li, listitem_T *next) { li->li_next = next; }

/// Set li_prev on a listitem (accessor for Rust).
void nvim_listitem_set_prev(listitem_T *li, listitem_T *prev) { li->li_prev = prev; }

/// Index into a typval_T array (accessor for Rust).
typval_T *nvim_tv_idx(typval_T *argvars, int i) { return &argvars[i]; }

// Phase 1 accessor helpers for Rust get functions

/// Format a VimL number into buf (snprintf wrapper for Rust).
void nvim_format_number(int64_t n, char *buf, int buflen)
{
  snprintf(buf, (size_t)buflen, "%" PRId64, n);
}

/// Format a float into buf using Vim's %g format (vim_snprintf wrapper for Rust).
void nvim_format_float(double f, char *buf, int buflen)
{
  vim_snprintf(buf, (size_t)buflen, "%g", f);
}

/// Return the name string for a bool variable value (0=false, 1=true).
const char *nvim_get_bool_var_name(int b) { return encode_bool_var_names[b]; }

/// Return the name string for a special variable value (0=v:null).
const char *nvim_get_special_var_name(int s) { return encode_special_var_names[s]; }

/// Call vim_str2nr with STR2NR_ALL to parse a Vim number string.
/// Writes result to *out. Returns 0 if s is NULL, parsed value otherwise.
void nvim_vim_str2nr(const char *s, int64_t *out)
{
  varnumber_T n = 0;
  if (s != NULL) {
    vim_str2nr(s, NULL, NULL, STR2NR_ALL, &n, NULL, 0, false, NULL);
  }
  *out = (int64_t)n;
}

/// Call var2fpos on a typval, return lnum, 0 if not a position.
int32_t nvim_tv_to_lnum_pos(const typval_T *tv, int *ret_fnum)
{
  pos_T *fp = var2fpos(tv, true, ret_fnum, false);
  if (fp == NULL) {
    return 0;
  }
  return (int32_t)fp->lnum;
}


// nvim_emsg_get_number_unknown deleted (Phase 9): Rust calls semsg directly.

// Phase 2 accessor helpers for Rust blob operations

/// Allocate a new empty blob_T (wrapper for Rust).
blob_T *nvim_blob_alloc_impl(void)
{
  blob_T *b = xcalloc(1, sizeof(blob_T));
  ga_init(&b->bv_ga, 1, 100);
  return b;
}

/// Free a blob_T (clear ga + xfree). Does NOT check refcount (wrapper for Rust).
void nvim_blob_free_impl(blob_T *b)
{
  ga_clear(&b->bv_ga);
  xfree(b);
}

/// Decrement blob refcount and return new value (accessor for Rust).
int nvim_blob_dec_refcount(blob_T *b) { return --b->bv_refcount; }

/// Set ga_maxlen of a blob (accessor for Rust).
void nvim_blob_set_ga_maxlen(blob_T *b, int n) { b->bv_ga.ga_maxlen = n; }

/// Duplicate ga_data of blob (xmemdup wrapper for Rust).
uint8_t *nvim_blob_xmemdup_ga_data(const blob_T *from, int len)
{
  return (uint8_t *)xmemdup(from->bv_ga.ga_data, (size_t)len);
}

/// Set ga_data of a blob (accessor for Rust).
void nvim_blob_set_ga_data(blob_T *b, uint8_t *data) { b->bv_ga.ga_data = data; }

/// tv_list_alloc_ret self-contained impl for Rust (avoids circular call).
list_T *nvim_tv_list_alloc_ret(typval_T *ret_tv, ptrdiff_t len)
{
  list_T *const l = nvim_list_alloc_impl();
  tv_list_set_ret(ret_tv, l);
  ret_tv->v_lock = VAR_UNLOCKED;
  return l;
}


/// ga_append on a blob's garray (wrapper for Rust).
void nvim_blob_ga_append(blob_T *b, uint8_t c)
{
  ga_append(&b->bv_ga, c);
}

/// ga_clear on a blob's garray (wrapper for Rust).
void nvim_blob_ga_clear_only(blob_T *b)
{
  ga_clear(&b->bv_ga);
}

// nvim_semsg_blob_invalid_value deleted (Phase 9): Rust calls semsg directly.

// Dict item accessor functions for Rust (Phase 3)
// These are self-contained to avoid circular calls with Rust exports.

/// Allocate a dict item with given key (accessor for Rust).
/// Direct implementation to avoid circular call with Rust tv_dict_item_alloc_len.
dictitem_T *nvim_dict_item_alloc_len(const char *key, size_t key_len)
{
  dictitem_T *const di = xmalloc(offsetof(dictitem_T, di_key) + key_len + 1);
  memcpy(di->di_key, key, key_len);
  di->di_key[key_len] = NUL;
  di->di_flags = DI_FLAGS_ALLOC;
  di->di_tv.v_lock = VAR_UNLOCKED;
  di->di_tv.v_type = VAR_UNKNOWN;
  return di;
}

/// Free a dict item (accessor for Rust).
/// Direct implementation to avoid circular call with Rust tv_dict_item_free.
void nvim_dict_item_free(dictitem_T *item)
{
  tv_clear(&item->di_tv);
  if (item->di_flags & DI_FLAGS_ALLOC) {
    xfree(item);
  }
}

/// Get a pointer to di_tv (accessor for Rust).
typval_T *nvim_dictitem_di_tv(dictitem_T *di) { return &di->di_tv; }

/// Add a dict item to a dict (accessor for Rust).
/// Direct implementation to avoid circular call with Rust tv_dict_add.
int nvim_dict_add_item(dict_T *d, dictitem_T *item)
{
  if (tv_dict_wrong_func_name(d, &item->di_tv, item->di_key)) {
    return FAIL;
  }
  return hash_add(&d->dv_hashtab, item->di_key);
}

/// Allocate an empty dict (accessor for Rust).
/// Self-contained to avoid circular call with Rust tv_dict_alloc.
dict_T *nvim_dict_alloc_impl(void)
{
  dict_T *const d = xcalloc(1, sizeof(dict_T));
  if (gc_first_dict != NULL) {
    gc_first_dict->dv_used_prev = d;
  }
  d->dv_used_next = gc_first_dict;
  d->dv_used_prev = NULL;
  gc_first_dict = d;
  hash_init(&d->dv_hashtab);
  d->dv_lock = VAR_UNLOCKED;
  d->dv_scope = VAR_NO_SCOPE;
  d->dv_refcount = 0;
  d->dv_copyID = 0;
  QUEUE_INIT(&d->watchers);
  d->lua_table_ref = LUA_NOREF;
  return d;
}

// Phase 4 accessors for tv_dict_free_contents / tv_dict_free_dict migration

/// Free all items in a dict's hashtab and reset it (high-level for Rust tv_dict_free_contents).
/// Does NOT free watchers; call nvim_dict_free_watchers separately.
void nvim_dict_free_hashtab_contents(dict_T *d)
{
  hash_lock(&d->dv_hashtab);
  assert(d->dv_hashtab.ht_locked > 0);
  HASHTAB_ITER(&d->dv_hashtab, hi, {
    dictitem_T *const di = TV_DICT_HI2DI(hi);
    hash_remove(&d->dv_hashtab, hi);
    tv_dict_item_free(di);
  });
  hash_clear(&d->dv_hashtab);
  d->dv_hashtab.ht_locked--;
  hash_init(&d->dv_hashtab);
}

/// Free all watchers from a dict's watcher QUEUE (high-level for Rust tv_dict_free_contents).
void nvim_dict_free_watchers(dict_T *d)
{
  while (!QUEUE_EMPTY(&d->watchers)) {
    QUEUE *w = QUEUE_HEAD(&d->watchers);
    QUEUE_REMOVE(w);
    DictWatcher *watcher = tv_dict_watcher_node_data(w);
    tv_dict_watcher_free(watcher);
  }
}

/// Remove dict from GC list, clear lua_table_ref, and xfree it (high-level for Rust tv_dict_free_dict).
void nvim_dict_gc_unlink_and_free(dict_T *d)
{
  if (d->dv_used_prev == NULL) {
    gc_first_dict = d->dv_used_next;
  } else {
    d->dv_used_prev->dv_used_next = d->dv_used_next;
  }
  if (d->dv_used_next != NULL) {
    d->dv_used_next->dv_used_prev = d->dv_used_prev;
  }
  NLUA_CLEAR_REF(d->lua_table_ref);
  xfree(d);
}

/// Get dict is_watched status (accessor for Rust tv_dict_extend).
int nvim_dict_is_watched(const dict_T *d) { return tv_dict_is_watched(d); }

/// tv_dict_item_copy wrapper (creates a copy of a dict item, accessor for Rust).
dictitem_T *nvim_dict_item_copy_impl(const dictitem_T *di)
{
  return tv_dict_item_copy((dictitem_T *)di);  // read-only; copies the item
}

/// Allocate a dict item with given key (accessor for Rust, avoids circular call).
dictitem_T *nvim_dict_item_alloc_impl(const char *key)
{
  return tv_dict_item_alloc(key);
}

/// Check if varname is valid (accessor for Rust tv_dict_extend).
int nvim_valid_varname(const char *name) { return valid_varname(name); }

/// var_check_ro wrapper (accessor for Rust tv_dict_extend).
bool nvim_var_check_ro(int flags, const char *name, size_t name_len)
{
  return var_check_ro(flags, name, name_len);
}

/// Get di_flags from a dictitem (accessor for Rust tv_dict_extend).
int nvim_dictitem_get_flags(const dictitem_T *di) { return (int)di->di_flags; }

/// hash_lock on dict's hashtab (accessor for Rust tv_dict_extend).
void nvim_dict_hash_lock(dict_T *d) { hash_lock(&d->dv_hashtab); }

/// hash_unlock on dict's hashtab (accessor for Rust tv_dict_extend).
void nvim_dict_hash_unlock(dict_T *d) { hash_unlock(&d->dv_hashtab); }

/// hash_remove on dict's hashtab for given item (accessor for Rust tv_dict_extend).
void nvim_dict_hash_remove(dict_T *d, hashitem_T *hi) { hash_remove(&d->dv_hashtab, hi); }

/// hash_find on dict's hashtab by NUL-terminated key (accessor for Rust).
hashitem_T *nvim_dict_hash_find(const dict_T *d, const char *key)
{
  return hash_find(&d->dv_hashtab, key);
}

/// hash_find_len on dict's hashtab by key + length (accessor for Rust).
hashitem_T *nvim_dict_hash_find_len(const dict_T *d, const char *key, size_t len)
{
  return hash_find_len(&d->dv_hashtab, key, len);
}

// nvim_hashitem_is_empty: already defined in syntax_accessors.c

/// Get di_key from a dictitem as mutable (accessor for Rust).
const char *nvim_dictitem_get_key_ptr(const dictitem_T *di) { return di->di_key; }

// nvim_semsg_key_exists deleted (Phase 9): Rust calls semsg directly.

/// string_convert wrapper for tv_dict_copy (accessor for Rust).
/// Converts key using vimconv, writing new length to len_out. Returns NULL if no conversion.
char *nvim_dict_copy_key_convert(const vimconv_T *conv, const char *key, size_t *len_out)
{
  *len_out = strlen(key);
  return string_convert(conv, (char *)key, len_out);
}

/// Allocate a new dict item with given key and length (accessor for Rust tv_dict_copy).
dictitem_T *nvim_dict_item_alloc_len_impl(const char *key, size_t len)
{
  return tv_dict_item_alloc_len(key, len);
}

/// Increment dict refcount and set dv_copyID/dv_copydict (accessor for Rust tv_dict_copy).
void nvim_dict_set_copyid_and_copydict(dict_T *orig, int copyID, dict_T *copy)
{
  orig->dv_copyID = copyID;
  orig->dv_copydict = copy;
}

/// Increment dict refcount (accessor for Rust).
void nvim_dict_inc_refcount(dict_T *d) { d->dv_refcount++; }

/// Get dict refcount (accessor for Rust).
int nvim_dict_get_refcount(const dict_T *d) { return d->dv_refcount; }

/// Set v_type to VAR_DICT and vval.v_dict (accessor for Rust).
void nvim_tv_set_dict(typval_T *tv, dict_T *d)
{
  tv->v_type = VAR_DICT;
  tv->v_lock = VAR_UNLOCKED;
  tv->vval.v_dict = d;
}

/// tv_dict_alloc_ret wrapper for Rust.
/// Self-contained implementation using tv_dict_alloc_lock and tv_dict_set_ret.
void nvim_tv_dict_alloc_ret(typval_T *ret_tv)
{
  dict_T *const d = nvim_dict_alloc_impl();
  d->dv_lock = VAR_UNLOCKED;
  tv_dict_set_ret(ret_tv, d);
}

/// tv_list_ref wrapper - increment list refcount (accessor for Rust).
void nvim_list_ref(list_T *l) { tv_list_ref(l); }

/// func_ref wrapper for Rust.
void nvim_func_ref(char *name) { func_ref(name); }

// nvim_xmemdupz: already defined in shada_shim.c
// nvim_ufunc_get_name, nvim_ufunc_get_namelen: already defined in runtime_ffi.c and userfunc.c

/// tv_copy wrapper for Rust.
void nvim_tv_copy(const typval_T *from, typval_T *to) { tv_copy(from, to); }

// nvim_xstrdup: already defined in register.c

/// xstrndup wrapper for Rust.
char *nvim_xstrndup(const char *s, size_t len) { return xstrndup(s, len); }

/// Set v_lock on a typval (accessor for Rust).
void nvim_tv_set_lock(typval_T *tv, int lock) { tv->v_lock = (VarLockStatus)lock; }

/// Set v_type on a typval (accessor for Rust).
void nvim_tv_set_type(typval_T *tv, int v_type) { tv->v_type = (VarType)v_type; }

/// Set vval.v_list on a typval (accessor for Rust).
void nvim_tv_set_list(typval_T *tv, list_T *l)
{
  tv->v_type = VAR_LIST;
  tv->v_lock = VAR_UNLOCKED;
  tv->vval.v_list = l;
}

/// Set vval.v_bool on a typval (accessor for Rust).
void nvim_tv_set_bool(typval_T *tv, int val) { tv->vval.v_bool = (BoolVarValue)val; }

/// Set dv_lock on a dict (accessor for Rust).
void nvim_dict_set_lock(dict_T *d, int lock) { d->dv_lock = (VarLockStatus)lock; }

/// Get lv_refcount from a list (accessor for Rust).
int nvim_list_get_refcount(const list_T *l) { return l->lv_refcount; }

/// Decrement lv_refcount from a list and return new value (accessor for Rust).
int nvim_list_dec_refcount(list_T *l) { return --l->lv_refcount; }

// nvim_dict_remove_key migrated to Rust (Phase 3 of f39b5673).

// Phase 5 accessor functions for Rust list infrastructure migration

/// Get lv_watch from a list (accessor for Rust).
listwatch_T *nvim_list_get_watch(const list_T *l) { return l->lv_watch; }

/// Set lv_watch on a list (accessor for Rust).
void nvim_list_set_watch(list_T *l, listwatch_T *lw) { l->lv_watch = lw; }

/// Get lw_next from a listwatch (accessor for Rust).
listwatch_T *nvim_listwatch_get_next(const listwatch_T *lw) { return lw->lw_next; }

/// Set lw_next on a listwatch (accessor for Rust).
void nvim_listwatch_set_next(listwatch_T *lw, listwatch_T *next) { lw->lw_next = next; }

/// Get lw_item from a listwatch (accessor for Rust).
listitem_T *nvim_listwatch_get_item(const listwatch_T *lw) { return lw->lw_item; }

/// Set lw_item on a listwatch (accessor for Rust).
void nvim_listwatch_set_item(listwatch_T *lw, listitem_T *item) { lw->lw_item = item; }

/// Get tv_in_free_unref_items global (accessor for Rust).
int nvim_get_tv_in_free_unref_items(void) { return (int)tv_in_free_unref_items; }

/// Set lv_refcount on a list (accessor for Rust).
void nvim_list_set_refcount(list_T *l, int rc) { l->lv_refcount = rc; }

/// Initialize a static list with DO_NOT_FREE_CNT (accessor for Rust).
/// CLEAR_POINTER zeros the struct, then sets refcount.
void nvim_list_init_static_impl(list_T *l)
{
  CLEAR_POINTER(l);
  l->lv_refcount = DO_NOT_FREE_CNT;
}

/// Clear (zero) a staticList10_T struct (accessor for Rust tv_list_init_static10).
void nvim_staticlist10_clear(staticList10_T *sl) { CLEAR_POINTER(sl); }

/// Get pointer to sl->sl_items[i] (accessor for Rust tv_list_init_static10).
listitem_T *nvim_staticlist10_get_item(staticList10_T *sl, int i) { return &sl->sl_items[i]; }

/// Get pointer to sl->sl_list (accessor for Rust tv_list_init_static10).
list_T *nvim_staticlist10_get_list(staticList10_T *sl) { return &sl->sl_list; }

/// Return DO_NOT_FREE_CNT constant (accessor for Rust).
int nvim_do_not_free_cnt(void) { return DO_NOT_FREE_CNT; }

/// Self-contained list alloc (accessor for Rust).
/// Avoids circular call with Rust tv_list_alloc.
list_T *nvim_list_alloc_impl(void)
{
  list_T *const list = xcalloc(1, sizeof(list_T));
  if (gc_first_list != NULL) {
    gc_first_list->lv_used_prev = list;
  }
  list->lv_used_prev = NULL;
  list->lv_used_next = gc_first_list;
  gc_first_list = list;
  list->lua_table_ref = LUA_NOREF;
  return list;
}

/// Self-contained list free_list (accessor for Rust).
/// Avoids circular call with Rust tv_list_free_list.
void nvim_list_free_list_impl(list_T *l)
{
  if (l->lv_used_prev == NULL) {
    gc_first_list = l->lv_used_next;
  } else {
    l->lv_used_prev->lv_used_next = l->lv_used_next;
  }
  if (l->lv_used_next != NULL) {
    l->lv_used_next->lv_used_prev = l->lv_used_prev;
  }
  NLUA_CLEAR_REF(l->lua_table_ref);
  xfree(l);
}

// nvim_list_watch_fix migrated to Rust (Phase 3 of f39b5673).
// Uses nvim_listwatch_get_item / nvim_listwatch_set_item / nvim_listitem_get_next.

// nvim_list_item_clear_free migrated to Rust (Phase 3 of f39b5673).
// Uses nvim_listitem_get_tv + tv_clear + nvim_list_item_free.

/// Shallow copy of a list (tv_list_copy(NULL, l, false, 0)) for Rust.
/// Returns a new list with refcount=1, or NULL on alloc failure.
list_T *nvim_list_copy_shallow(list_T *l)
{
  return tv_list_copy(NULL, l, false, 0);
}

/// Set vval.v_list on a typval WITHOUT touching type/lock (raw field write for Rust).
/// Use only when type/lock have already been set.
void nvim_tv_set_list_vval(typval_T *tv, list_T *l)
{
  tv->vval.v_list = l;
}

// Phase 6c accessor functions for list slice/range/flatten/remove

/// Set list as return value of typval (tv_list_set_ret) for Rust.
/// Sets type to VAR_LIST, increments refcount.
void nvim_tv_list_set_ret(typval_T *tv, list_T *l)
{
  tv_list_set_ret(tv, l);
}

/// Check if got_int is set (accessor for Rust).
int nvim_got_int(void) { return got_int; }

/// Call fast_breakcheck() (accessor for Rust).
void nvim_fast_breakcheck(void) { fast_breakcheck(); }

// nvim_emsg_invrange deleted (Phase 9): Rust calls emsg directly.

/// tv_list_slice_or_index index case: copy item[n1] into rettv (for Rust).
/// Copies item TV to stack, clears rettv (freeing the list), then assigns.
void nvim_tv_list_index_into_rettv(typval_T *rettv, listitem_T *item)
{
  typval_T var1;
  tv_copy(TV_LIST_ITEM_TV(item), &var1);
  tv_clear(rettv);
  *rettv = var1;
}

/// tv_list_remove single-item case: move item's TV into rettv (for Rust).
/// This is a bitwise move (no refcount change), then frees the listitem struct.
void nvim_tv_listitem_move_to_rettv(typval_T *rettv, listitem_T *item)
{
  *rettv = *TV_LIST_ITEM_TV(item);
  xfree(item);
}

// Phase 6d: tv_list_assign_range accessors

// nvim_emsg_list_more_items, nvim_emsg_list_not_enough_items deleted (Phase 9):
// Rust calls emsg directly.

/// Get listitem v_lock field via TV_LIST_ITEM_TV (accessor for Rust).
/// Needed by tv_list_assign_range lock check.
int nvim_listitem_get_v_lock(const listitem_T *li) { return TV_LIST_ITEM_TV(li)->v_lock; }

// Phase 6e: tv_list_copy, f_items accessors
// nvim_tv_dict2list_items deleted (Phase 8): tv_dict2list migrated to Rust

// Phase 4 additional accessors

/// Set dv_refcount on a dict (accessor for Rust tv_dict_unref).
void nvim_dict_set_refcount(dict_T *d, int rc) { d->dv_refcount = rc; }

/// Free a dict item without clearing its tv (for error paths in tv_dict_copy where tv is unset).
void nvim_dict_item_free_raw(dictitem_T *di) { xfree(di); }

/// Get tv_dict_len (number of items) for a dict (accessor for Rust tv_dict_equal).
int nvim_dict_get_len_impl(const dict_T *d) { return tv_dict_len(d); }

/// Get dv_scope from a dict (accessor for Rust tv_dict_extend).
int nvim_dict_get_scope_impl(const dict_T *d) { return (int)d->dv_scope; }

// Phase 6g / Phase 8: f_keys, f_values, f_items (dict case) accessor wrappers deleted;
// tv_dict2list logic now implemented directly in Rust tv_dict2list_impl().

// Phase 6j: tv_dict_to_env accessor

// nvim_dictitem_format_env migrated to Rust (Phase 3 of f39b5673).
// Uses nvim_dictitem_get_key, nvim_dictitem_di_tv, tv_get_string, xmalloc, snprintf.

// Phase 6f: tv_dict_remove migrated to Rust; accessors in eval_shim.c

// Phase 6h: tv_dict_set_keys_readonly, tv_dict_clear accessors

/// Get ht_array pointer for a dict's hashtab (accessor for Rust hashtab iteration).
hashitem_T *nvim_dict_get_ht_array(const dict_T *d) { return d->dv_hashtab.ht_array; }

/// Get hi_key from a hashitem (accessor for Rust).
const char *nvim_hashitem_get_key(const hashitem_T *hi) { return hi->hi_key; }

/// Get the address of hash_removed sentinel (accessor for Rust).
const char *nvim_hash_removed_ptr(void) { return HI_KEY_REMOVED; }

/// Convert hashitem to dictitem via TV_DICT_HI2DI and set DI_FLAGS_RO|DI_FLAGS_FIX.
void nvim_hashitem_set_ro_fix(hashitem_T *hi) { TV_DICT_HI2DI(hi)->di_flags |= DI_FLAGS_RO | DI_FLAGS_FIX; }

/// Advance a hashitem pointer by one (sizeof(hashitem_T)) for Rust iteration.
hashitem_T *nvim_hashitem_next(hashitem_T *hi) { return hi + 1; }

/// Convert hashitem to dictitem via TV_DICT_HI2DI (accessor for Rust dict iteration).
dictitem_T *nvim_hashitem_to_dictitem(hashitem_T *hi) { return TV_DICT_HI2DI(hi); }

// Phase 1 (typval migration): C accessor wrappers for callback Lua operations.
// These allow Rust to call Lua-specific functions without depending on lua headers.

/// Clear the luaref in a Callback (NLUA_CLEAR_REF equivalent for Rust).
/// Sets the type back to kCallbackNone.
void nvim_callback_clear_luaref(Callback *cb)
{
  NLUA_CLEAR_REF(cb->data.luaref);
  cb->type = kCallbackNone;
  cb->data.funcref = NULL;
}

/// Duplicate a LuaRef (api_new_luaref wrapper for Rust).
int nvim_callback_new_luaref(int ref) { return (int)api_new_luaref((LuaRef)ref); }

/// Format a luaref callback as a string (nlua_funcref_str wrapper for Rust).
/// Returns allocated string; caller must free.
char *nvim_callback_funcref_str(int luaref, Arena *arena)
{
  return nlua_funcref_str((LuaRef)luaref, arena);
}

/// Get partial_T refcount (accessor for Rust).
int nvim_partial_get_refcount(const partial_T *pt) { return pt->pt_refcount; }

/// Increment partial_T refcount (accessor for Rust).
void nvim_partial_inc_refcount(partial_T *pt) { pt->pt_refcount++; }

/// Decrement partial_T refcount (accessor for Rust, used by tv_clear).
void nvim_partial_dec_refcount(partial_T *pt) { pt->pt_refcount--; }

// nvim_partial_get_name: already defined in eval/userfunc.c

// nvim_tv_set_partial, nvim_tv_set_special: already defined in eval/vars.c

/// Set vval.v_string (takes ownership) on a typval without changing type (accessor for Rust).
void nvim_tv_set_vstring_owned(typval_T *tv, char *s) { tv->vval.v_string = s; }

/// Null vval.v_partial without changing type/lock (accessor for Rust, used by tv_clear).
void nvim_tv_set_vpartial_null(typval_T *tv) { tv->vval.v_partial = NULL; }

/// Null vval.v_blob without changing type/lock (accessor for Rust, used by tv_clear).
void nvim_tv_set_vblob_null(typval_T *tv) { tv->vval.v_blob = NULL; }

/// Null vval.v_list without changing type/lock (accessor for Rust, used by tv_clear).
void nvim_tv_set_vlist_null(typval_T *tv) { tv->vval.v_list = NULL; }

/// Null vval.v_dict without changing type/lock (accessor for Rust, used by tv_clear).
void nvim_tv_set_vdict_null(typval_T *tv) { tv->vval.v_dict = NULL; }

// Phase 2 (typval migration): accessors for tv_copy, tv_free, tv_equal.

// nvim_tv_get_partial: already defined in eval/vars.c

/// Get vval.v_string as mutable (accessor for Rust tv_free/tv_copy).
char *nvim_tv_get_string_mutable(typval_T *tv) { return tv->vval.v_string; }

/// Copy the vval union from one typval to another (memmove accessor for Rust tv_copy).
void nvim_tv_copy_vval(typval_T *to, const typval_T *from)
{
  memmove(&to->vval, &from->vval, sizeof(to->vval));
}

/// Get bv_refcount from a blob (accessor for Rust tv_copy blob refcount bump).
int nvim_blob_get_bv_refcount(const blob_T *b) { return b->bv_refcount; }

/// Increment bv_refcount on a blob (accessor for Rust tv_copy).
void nvim_blob_inc_refcount(blob_T *b) { b->bv_refcount++; }

/// mb_strcmp_ic wrapper for Rust (accessible by Rust for tv_equal string comparison).
int nvim_mb_strcmp_ic(bool ic, const char *s1, const char *s2) { return mb_strcmp_ic(ic, s1, s2); }

// Phase 5 (typval migration): dict helpers and tv_dict2list accessors

/// Return pointer to dv_hashtab field (accessor for Rust tv_dict_wrong_func_name).
hashtab_T *nvim_dict_get_hashtab_ptr(const dict_T *d) { return (hashtab_T *)&d->dv_hashtab; }

/// Return global var dict pointer (accessor for Rust tv_dict_wrong_func_name).
dict_T *nvim_get_globvar_dict(void) { return get_globvar_dict(); }

/// Return current funccal local hashtab (accessor for Rust tv_dict_wrong_func_name).
hashtab_T *nvim_get_funccal_local_ht(void) { return get_funccal_local_ht(); }

/// Check if typval is a func/partial type (accessor for Rust).
bool nvim_tv_is_func(const typval_T *tv) { return tv_is_func(*tv); }

/// var_wrong_func_name(name, true) wrapper for Rust.
bool nvim_var_wrong_func_name(const char *name) { return var_wrong_func_name(name, true); }

/// Get key_pattern pointer from a DictWatcher (accessor for Rust).
const char *nvim_watcher_get_key_pattern(const DictWatcher *w) { return w->key_pattern; }

/// Get key_pattern_len from a DictWatcher (accessor for Rust).
size_t nvim_watcher_get_key_pattern_len(const DictWatcher *w) { return w->key_pattern_len; }

/// Get pointer to callback field in a DictWatcher (accessor for Rust tv_dict_watcher_free).
Callback *nvim_watcher_get_callback_ptr(DictWatcher *w) { return &w->callback; }

/// Call set_selfdict(tv, d) (accessor for Rust tv_dict_get_callback).
void nvim_set_selfdict(typval_T *tv, dict_T *d) { set_selfdict(tv, d); }

// nvim_emsg_not_func_or_funcname deleted (Phase 9): Rust calls emsg directly.

/// Check typval is func or string (accessor for Rust tv_dict_get_callback).
bool nvim_tv_is_func_or_string(const typval_T *tv)
{
  return tv_is_func(*tv) || tv->v_type == VAR_STRING;
}

/// call rs_callback_from_typval for Rust callers that only have C linkage.
/// Returns true on success.
bool nvim_callback_from_typval_impl(Callback *result, typval_T *tv)
{
  return rs_callback_from_typval(result, tv);
}

// Phase 6 (typval migration): sort/uniq, join, list2str, tv_list_init_static10 accessors

// nvim_emsg_sort_failed, nvim_emsg_uniq_failed, nvim_emsg_listarg, nvim_emsg_invarg
// deleted (Phase 9): Rust calls emsg/semsg directly.

/// tv_check_for_dict_arg wrapper: returns OK (1) or FAIL (0) (accessor for Rust).
int nvim_tv_check_for_dict_arg(typval_T *argvars, int idx)
{
  return tv_check_for_dict_arg(argvars, idx);
}

/// tv_get_string_chk wrapper: returns NULL if type error (accessor for Rust parse_sort_uniq_args).
const char *nvim_tv_get_string_checked(const typval_T *tv)
{
  return tv_get_string_chk(tv);
}

// nvim_emsg_e_listreq deleted (Phase 9): Rust calls emsg directly.

// nvim_list_join_to_string, nvim_f_list2str_from_list deleted (Phase 3): inlined into Rust

// Phase 7 (typval migration): dict watcher add/remove/notify accessors

/// Get pointer to dict's watchers QUEUE head (accessor for Rust tv_dict_watcher_add/remove/notify).
QUEUE *nvim_dict_get_watchers_head(dict_T *d) { return &d->watchers; }

/// Get DictWatcher* from a QUEUE node pointer (accessor for Rust watcher iteration).
DictWatcher *nvim_watcher_node_data(QUEUE *q) { return tv_dict_watcher_node_data(q); }

/// Get busy flag from a DictWatcher (accessor for Rust).
bool nvim_watcher_get_busy(DictWatcher *w) { return w->busy; }

/// Set busy flag on a DictWatcher (accessor for Rust).
void nvim_watcher_set_busy(DictWatcher *w, bool v) { w->busy = v; }

/// Get needs_free flag from a DictWatcher (accessor for Rust).
bool nvim_watcher_get_needs_free(DictWatcher *w) { return w->needs_free; }

/// Set needs_free flag on a DictWatcher (accessor for Rust).
void nvim_watcher_set_needs_free(DictWatcher *w, bool v) { w->needs_free = v; }

/// Compare two Callback structs for equality, using raw pointers (accessor for Rust).
bool nvim_callback_equal_raw(const Callback *cb1, const Callback *cb2)
{
  return tv_callback_equal(cb1, cb2);
}

// Phase 10: lua_table_ref write accessors for nlua_pop_typval.

/// Set lua_table_ref on a list (accessor for Rust nlua_pop_typval).
void nvim_list_set_lua_table_ref(list_T *l, LuaRef ref_) { l->lua_table_ref = ref_; }

/// Set lua_table_ref on a dict (accessor for Rust nlua_pop_typval).
void nvim_dict_set_lua_table_ref(dict_T *d, LuaRef ref_) { d->lua_table_ref = ref_; }

/// Increment dv_refcount on a dict (for nlua_pop_typval dict allocation).
void nvim_dict_inc_refcount_wrapped(dict_T *d) { d->dv_refcount++; }

