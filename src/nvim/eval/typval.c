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


/// Type for tv_dict2list() function
typedef enum {
  kDict2ListKeys,    ///< List dictionary keys.
  kDict2ListValues,  ///< List dictionary values.
  kDict2ListItems,   ///< List dictionary contents: [keys, values].
} DictListType;

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

/// Allocate a list item
///
/// @return [allocated] new list item.
static listitem_T *tv_list_item_alloc(void)
  FUNC_ATTR_NONNULL_RET FUNC_ATTR_MALLOC
{
  return xmalloc(sizeof(listitem_T));
}

// tv_list_item_remove, tv_list_watch_add, tv_list_watch_remove, tv_list_watch_fix
// migrated to Rust (Phase 5)

//{{{2 Alloc/free

// tv_list_alloc migrated to Rust (Phase 5)

/// Initialize a static list with 10 items
///
/// @param[out]  sl  Static list to initialize.
void tv_list_init_static10(staticList10_T *const sl)
  FUNC_ATTR_NONNULL_ALL
{
#define SL_SIZE ARRAY_SIZE(sl->sl_items)
  list_T *const l = &sl->sl_list;

  CLEAR_POINTER(sl);
  l->lv_first = &sl->sl_items[0];
  l->lv_last = &sl->sl_items[SL_SIZE - 1];
  l->lv_refcount = DO_NOT_FREE_CNT;
  tv_list_set_lock(l, VAR_FIXED);
  sl->sl_list.lv_len = 10;

  sl->sl_items[0].li_prev = NULL;
  sl->sl_items[0].li_next = &sl->sl_items[1];
  sl->sl_items[SL_SIZE - 1].li_prev = &sl->sl_items[SL_SIZE - 2];
  sl->sl_items[SL_SIZE - 1].li_next = NULL;

  for (size_t i = 1; i < SL_SIZE - 1; i++) {
    listitem_T *const li = &sl->sl_items[i];
    li->li_prev = li - 1;
    li->li_next = li + 1;
  }
#undef SL_SIZE
}

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
  listitem_T *const li = tv_list_item_alloc();
  *TV_LIST_ITEM_TV(li) = tv;
  tv_list_append(l, li);
  return TV_LIST_ITEM_TV(li);
}

// tv_list_append_list, tv_list_append_dict, tv_list_append_string,
// tv_list_append_allocated_string, tv_list_append_number migrated to Rust (Phase 5)

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

typedef struct {
  char *s;
  char *tofree;
} Join;

/// Join list into a string, helper function
///
/// @param[out]  gap  Garray where result will be saved.
/// @param[in]  l  List to join.
/// @param[in]  sep  Used separator.
/// @param[in]  join_gap  Garray to keep each list item string.
///
/// @return OK in case of success, FAIL otherwise.
static int list_join_inner(garray_T *const gap, list_T *const l, const char *const sep,
                           garray_T *const join_gap)
  FUNC_ATTR_NONNULL_ALL
{
  size_t sumlen = 0;
  bool first = true;

  // Stringify each item in the list.
  TV_LIST_ITER(l, item, {
    if (got_int) {
      break;
    }
    char *s;
    size_t len;
    s = encode_tv2echo(TV_LIST_ITEM_TV(item), &len);
    if (s == NULL) {
      return FAIL;
    }

    sumlen += len;

    Join *const p = GA_APPEND_VIA_PTR(Join, join_gap);
    p->tofree = p->s = s;

    line_breakcheck();
  });

  // Allocate result buffer with its total size, avoid re-allocation and
  // multiple copy operations.  Add 2 for a tailing ']' and NUL.
  if (join_gap->ga_len >= 2) {
    sumlen += strlen(sep) * (size_t)(join_gap->ga_len - 1);
  }
  ga_grow(gap, (int)sumlen + 2);

  for (int i = 0; i < join_gap->ga_len && !got_int; i++) {
    if (first) {
      first = false;
    } else {
      ga_concat(gap, sep);
    }
    const Join *const p = ((const Join *)join_gap->ga_data) + i;

    if (p->s != NULL) {
      ga_concat(gap, p->s);
    }
    line_breakcheck();
  }

  return OK;
}

/// Join list into a string using given separator
///
/// @param[out]  gap  Garray where result will be saved.
/// @param[in]  l  Joined list.
/// @param[in]  sep  Separator.
///
/// @return OK in case of success, FAIL otherwise.
int tv_list_join(garray_T *const gap, list_T *const l, const char *const sep)
  FUNC_ATTR_NONNULL_ARG(1)
{
  if (!tv_list_len(l)) {
    return OK;
  }

  garray_T join_ga;
  int retval;

  ga_init(&join_ga, (int)sizeof(Join), tv_list_len(l));
  retval = list_join_inner(gap, l, sep, &join_ga);

#define FREE_JOIN_TOFREE(join) xfree((join)->tofree)
  GA_DEEP_CLEAR(&join_ga, Join, FREE_JOIN_TOFREE);
#undef FREE_JOIN_TOFREE

  return retval;
}

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

// tv_dict_watcher_free migrated to Rust (Phase 5)
extern void tv_dict_watcher_free(DictWatcher *watcher);

/// Add watcher to a dictionary
///
/// @param[in]  dict  Dictionary to add watcher to.
/// @param[in]  key_pattern  Pattern to watch for.
/// @param[in]  key_pattern_len  Key pattern length.
/// @param  callback  Function to be called on events.
void tv_dict_watcher_add(dict_T *const dict, const char *const key_pattern,
                         const size_t key_pattern_len, Callback callback)
  FUNC_ATTR_NONNULL_ARG(2)
{
  if (dict == NULL) {
    return;
  }
  DictWatcher *const watcher = xmalloc(sizeof(DictWatcher));
  watcher->key_pattern = xmemdupz(key_pattern, key_pattern_len);
  watcher->key_pattern_len = key_pattern_len;
  watcher->callback = callback;
  watcher->busy = false;
  watcher->needs_free = false;
  QUEUE_INSERT_TAIL(&dict->watchers, &watcher->node);
}

// tv_callback_equal, callback_free, callback_put, callback_copy, callback_to_string
// migrated to Rust (Phase 1)

/// Remove watcher from a dictionary
///
/// @param  dict  Dictionary to remove watcher from.
/// @param[in]  key_pattern  Pattern to remove watcher for.
/// @param[in]  key_pattern_len  Pattern length.
/// @param  callback  Callback to remove watcher for.
///
/// @return True on success, false if relevant watcher was not found.
bool tv_dict_watcher_remove(dict_T *const dict, const char *const key_pattern,
                            const size_t key_pattern_len, Callback callback)
  FUNC_ATTR_NONNULL_ARG(2)
{
  if (dict == NULL) {
    return false;
  }

  QUEUE *w = NULL;
  DictWatcher *watcher = NULL;
  bool matched = false;
  bool queue_is_busy = false;
  QUEUE_FOREACH(w, &dict->watchers, {
    watcher = tv_dict_watcher_node_data(w);
    if (watcher->busy) {
      queue_is_busy = true;
    }
    if (tv_callback_equal(&watcher->callback, &callback)
        && watcher->key_pattern_len == key_pattern_len
        && memcmp(watcher->key_pattern, key_pattern, key_pattern_len) == 0) {
      matched = true;
      break;
    }
  })

  if (!matched) {
    return false;
  }

  if (queue_is_busy) {
    watcher->needs_free = true;
  } else {
    QUEUE_REMOVE(w);
    tv_dict_watcher_free(watcher);
  }
  return true;
}

// tv_dict_watcher_matches migrated to Rust (Phase 5)
extern bool tv_dict_watcher_matches(DictWatcher *watcher, const char *key);

/// Send a change notification to all dictionary watchers that match given key
///
/// @param[in]  dict  Dictionary which was modified.
/// @param[in]  key  Key which was modified.
/// @param[in]  newtv  New key value.
/// @param[in]  oldtv  Old key value.
void tv_dict_watcher_notify(dict_T *const dict, const char *const key, typval_T *const newtv,
                            typval_T *const oldtv)
  FUNC_ATTR_NONNULL_ARG(1, 2)
{
  typval_T argv[3];

  argv[0].v_type = VAR_DICT;
  argv[0].v_lock = VAR_UNLOCKED;
  argv[0].vval.v_dict = dict;
  argv[1].v_type = VAR_STRING;
  argv[1].v_lock = VAR_UNLOCKED;
  argv[1].vval.v_string = xstrdup(key);
  argv[2].v_type = VAR_DICT;
  argv[2].v_lock = VAR_UNLOCKED;
  argv[2].vval.v_dict = tv_dict_alloc();
  argv[2].vval.v_dict->dv_refcount++;

  if (newtv) {
    dictitem_T *const v = tv_dict_item_alloc_len(S_LEN("new"));
    tv_copy(newtv, &v->di_tv);
    tv_dict_add(argv[2].vval.v_dict, v);
  }

  if (oldtv && oldtv->v_type != VAR_UNKNOWN) {
    dictitem_T *const v = tv_dict_item_alloc_len(S_LEN("old"));
    tv_copy(oldtv, &v->di_tv);
    tv_dict_add(argv[2].vval.v_dict, v);
  }

  typval_T rettv;

  bool any_needs_free = false;
  dict->dv_refcount++;
  QUEUE *w;
  QUEUE_FOREACH(w, &dict->watchers, {
    DictWatcher *watcher = tv_dict_watcher_node_data(w);
    if (!watcher->busy && tv_dict_watcher_matches(watcher, key)) {
      rettv = TV_INITIAL_VALUE;
      watcher->busy = true;
      callback_call(&watcher->callback, 3, argv, &rettv);
      watcher->busy = false;
      tv_clear(&rettv);
      if (watcher->needs_free) {
        any_needs_free = true;
      }
    }
  })
  if (any_needs_free) {
    QUEUE_FOREACH(w, &dict->watchers, {
      DictWatcher *watcher = tv_dict_watcher_node_data(w);
      if (watcher->needs_free) {
        QUEUE_REMOVE(w);
        tv_dict_watcher_free(watcher);
      }
    })
  }
  tv_dict_unref(dict);

  for (size_t i = 1; i < ARRAY_SIZE(argv); i++) {
    tv_clear(argv + i);
  }
}

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

/// Turn a dictionary into a list
///
/// @param[in] argvars Arguments to items(). The first argument is check for being
///                    a dictionary, will give an error if not.
/// @param[out] rettv  Location where result will be saved.
/// @param[in] what    What to save in rettv.
static void tv_dict2list(typval_T *const argvars, typval_T *const rettv, const DictListType what)
{
  if ((what == kDict2ListItems
       ? tv_check_for_string_or_list_or_dict_arg(argvars, 0)
       : tv_check_for_dict_arg(argvars, 0)) == FAIL) {
    tv_list_alloc_ret(rettv, 0);
    return;
  }

  dict_T *d = argvars[0].vval.v_dict;
  tv_list_alloc_ret(rettv, tv_dict_len(d));
  if (d == NULL) {
    // NULL dict behaves like an empty dict
    return;
  }

  TV_DICT_ITER(d, di, {
    typval_T tv_item = { .v_lock = VAR_UNLOCKED };

    switch (what) {
      case kDict2ListKeys:
        tv_item.v_type = VAR_STRING;
        tv_item.vval.v_string = xstrdup(di->di_key);
        break;
      case kDict2ListValues:
        tv_copy(&di->di_tv, &tv_item);
        break;
      case kDict2ListItems: {
        // items()
        list_T *const sub_l = tv_list_alloc(2);
        tv_item.v_type = VAR_LIST;
        tv_item.vval.v_list = sub_l;
        tv_list_ref(sub_l);
        tv_list_append_string(sub_l, di->di_key, -1);
        tv_list_append_tv(sub_l, &di->di_tv);
        break;
      }
    }

    tv_list_append_owned_tv(rettv->vval.v_list, tv_item);
  });
}

// f_items migrated to Rust (Phase 6e)

// f_keys, f_values migrated to Rust (Phase 6g)

// f_has_key migrated to Rust (Phase 6)

// tv_dict_remove migrated to Rust (Phase 6f)

// tv_blob_alloc_ret, tv_blob_copy migrated to Rust (Phase 2)

//{{{3 Clear
#define TYPVAL_ENCODE_ALLOW_SPECIALS false
#define TYPVAL_ENCODE_CHECK_BEFORE

#define TYPVAL_ENCODE_CONV_NIL(tv) \
  do { \
    (tv)->vval.v_special = kSpecialVarNull; \
    (tv)->v_lock = VAR_UNLOCKED; \
  } while (0)

#define TYPVAL_ENCODE_CONV_BOOL(tv, num) \
  do { \
    (tv)->vval.v_bool = kBoolVarFalse; \
    (tv)->v_lock = VAR_UNLOCKED; \
  } while (0)

#define TYPVAL_ENCODE_CONV_NUMBER(tv, num) \
  do { \
    (void)(num); \
    (tv)->vval.v_number = 0; \
    (tv)->v_lock = VAR_UNLOCKED; \
  } while (0)

#define TYPVAL_ENCODE_CONV_UNSIGNED_NUMBER(tv, num)

#define TYPVAL_ENCODE_CONV_FLOAT(tv, flt) \
  do { \
    (tv)->vval.v_float = 0; \
    (tv)->v_lock = VAR_UNLOCKED; \
  } while (0)

#define TYPVAL_ENCODE_CONV_STRING(tv, buf, len) \
  do { \
    xfree(buf); \
    (tv)->vval.v_string = NULL; \
    (tv)->v_lock = VAR_UNLOCKED; \
  } while (0)

#define TYPVAL_ENCODE_CONV_STR_STRING(tv, buf, len)

#define TYPVAL_ENCODE_CONV_EXT_STRING(tv, buf, len, type)

#define TYPVAL_ENCODE_CONV_BLOB(tv, blob, len) \
  do { \
    tv_blob_unref((tv)->vval.v_blob); \
    (tv)->vval.v_blob = NULL; \
    (tv)->v_lock = VAR_UNLOCKED; \
  } while (0)

static inline int _nothing_conv_func_start(typval_T *const tv, char *const fun)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_ALWAYS_INLINE FUNC_ATTR_NONNULL_ARG(1)
{
  tv->v_lock = VAR_UNLOCKED;
  if (tv->v_type == VAR_PARTIAL) {
    partial_T *const pt_ = tv->vval.v_partial;
    if (pt_ != NULL && pt_->pt_refcount > 1) {
      pt_->pt_refcount--;
      tv->vval.v_partial = NULL;
      return OK;
    }
  } else {
    func_unref(fun);
    if (fun != tv_empty_string) {
      xfree(fun);
    }
    tv->vval.v_string = NULL;
  }
  return NOTDONE;
}
#define TYPVAL_ENCODE_CONV_FUNC_START(tv, fun) \
  do { \
    if (_nothing_conv_func_start(tv, fun) != NOTDONE) { \
      return OK; \
    } \
  } while (0)

#define TYPVAL_ENCODE_CONV_FUNC_BEFORE_ARGS(tv, len)
#define TYPVAL_ENCODE_CONV_FUNC_BEFORE_SELF(tv, len)

static inline void _nothing_conv_func_end(typval_T *const tv, const int copyID)
  FUNC_ATTR_ALWAYS_INLINE FUNC_ATTR_NONNULL_ALL
{
  if (tv->v_type == VAR_PARTIAL) {
    partial_T *const pt = tv->vval.v_partial;
    if (pt == NULL) {
      return;
    }
    // Dictionary should already be freed by the time.
    // If it was not freed then it is a part of the reference cycle.
    assert(pt->pt_dict == NULL || pt->pt_dict->dv_copyID == copyID);
    pt->pt_dict = NULL;
    // As well as all arguments.
    pt->pt_argc = 0;
    assert(pt->pt_refcount <= 1);
    partial_unref(pt);
    tv->vval.v_partial = NULL;
    assert(tv->v_lock == VAR_UNLOCKED);
  }
}
#define TYPVAL_ENCODE_CONV_FUNC_END(tv) _nothing_conv_func_end(tv, copyID)

#define TYPVAL_ENCODE_CONV_EMPTY_LIST(tv) \
  do { \
    tv_list_unref((tv)->vval.v_list); \
    (tv)->vval.v_list = NULL; \
    (tv)->v_lock = VAR_UNLOCKED; \
  } while (0)

static inline void _nothing_conv_empty_dict(typval_T *const tv, dict_T **const dictp)
  FUNC_ATTR_ALWAYS_INLINE FUNC_ATTR_NONNULL_ARG(2)
{
  tv_dict_unref(*dictp);
  *dictp = NULL;
  if (tv != NULL) {
    tv->v_lock = VAR_UNLOCKED;
  }
}
#define TYPVAL_ENCODE_CONV_EMPTY_DICT(tv, dict) \
  do { \
    assert((void *)&(dict) != (void *)&TYPVAL_ENCODE_NODICT_VAR); \
    _nothing_conv_empty_dict(tv, ((dict_T **)&(dict))); \
  } while (0)

static inline int _nothing_conv_real_list_after_start(typval_T *const tv,
                                                      MPConvStackVal *const mpsv)
  FUNC_ATTR_ALWAYS_INLINE FUNC_ATTR_WARN_UNUSED_RESULT
{
  assert(tv != NULL);
  tv->v_lock = VAR_UNLOCKED;
  if (tv->vval.v_list->lv_refcount > 1) {
    tv->vval.v_list->lv_refcount--;
    tv->vval.v_list = NULL;
    mpsv->data.l.li = NULL;
    return OK;
  }
  return NOTDONE;
}
#define TYPVAL_ENCODE_CONV_LIST_START(tv, len)

#define TYPVAL_ENCODE_CONV_REAL_LIST_AFTER_START(tv, mpsv) \
  do { \
    if (_nothing_conv_real_list_after_start(tv, &(mpsv)) != NOTDONE) { \
      goto typval_encode_stop_converting_one_item; \
    } \
  } while (0)

#define TYPVAL_ENCODE_CONV_LIST_BETWEEN_ITEMS(tv)

static inline void _nothing_conv_list_end(typval_T *const tv)
  FUNC_ATTR_ALWAYS_INLINE
{
  if (tv == NULL) {
    return;
  }
  assert(tv->v_type == VAR_LIST);
  list_T *const list = tv->vval.v_list;
  tv_list_unref(list);
  tv->vval.v_list = NULL;
}
#define TYPVAL_ENCODE_CONV_LIST_END(tv) _nothing_conv_list_end(tv)

static inline int _nothing_conv_real_dict_after_start(typval_T *const tv, dict_T **const dictp,
                                                      const void *const nodictvar,
                                                      MPConvStackVal *const mpsv)
  FUNC_ATTR_ALWAYS_INLINE FUNC_ATTR_WARN_UNUSED_RESULT
{
  if (tv != NULL) {
    tv->v_lock = VAR_UNLOCKED;
  }
  if ((const void *)dictp != nodictvar && (*dictp)->dv_refcount > 1) {
    (*dictp)->dv_refcount--;
    *dictp = NULL;
    mpsv->data.d.todo = 0;
    return OK;
  }
  return NOTDONE;
}
#define TYPVAL_ENCODE_CONV_DICT_START(tv, dict, len)

#define TYPVAL_ENCODE_CONV_REAL_DICT_AFTER_START(tv, dict, mpsv) \
  do { \
    if (_nothing_conv_real_dict_after_start(tv, (dict_T **)&(dict), \
                                            (void *)&TYPVAL_ENCODE_NODICT_VAR, &(mpsv)) \
        != NOTDONE) { \
      goto typval_encode_stop_converting_one_item; \
    } \
  } while (0)

#define TYPVAL_ENCODE_SPECIAL_DICT_KEY_CHECK(tv, dict)
#define TYPVAL_ENCODE_CONV_DICT_AFTER_KEY(tv, dict)
#define TYPVAL_ENCODE_CONV_DICT_BETWEEN_ITEMS(tv, dict)

static inline void _nothing_conv_dict_end(typval_T *const tv, dict_T **const dictp,
                                          const void *const nodictvar)
  FUNC_ATTR_ALWAYS_INLINE
{
  if ((const void *)dictp != nodictvar) {
    tv_dict_unref(*dictp);
    *dictp = NULL;
  }
}
#define TYPVAL_ENCODE_CONV_DICT_END(tv, dict) \
  _nothing_conv_dict_end(tv, (dict_T **)&(dict), \
                         (void *)&TYPVAL_ENCODE_NODICT_VAR)

#define TYPVAL_ENCODE_CONV_RECURSE(val, conv_type)

#define TYPVAL_ENCODE_SCOPE static
#define TYPVAL_ENCODE_NAME nothing
#define TYPVAL_ENCODE_FIRST_ARG_TYPE const void *const
#define TYPVAL_ENCODE_FIRST_ARG_NAME ignored
#include "nvim/eval/typval_encode.c.h"

#undef TYPVAL_ENCODE_SCOPE
#undef TYPVAL_ENCODE_NAME
#undef TYPVAL_ENCODE_FIRST_ARG_TYPE
#undef TYPVAL_ENCODE_FIRST_ARG_NAME

#undef TYPVAL_ENCODE_ALLOW_SPECIALS
#undef TYPVAL_ENCODE_CHECK_BEFORE
#undef TYPVAL_ENCODE_CONV_NIL
#undef TYPVAL_ENCODE_CONV_BOOL
#undef TYPVAL_ENCODE_CONV_NUMBER
#undef TYPVAL_ENCODE_CONV_UNSIGNED_NUMBER
#undef TYPVAL_ENCODE_CONV_FLOAT
#undef TYPVAL_ENCODE_CONV_STRING
#undef TYPVAL_ENCODE_CONV_STR_STRING
#undef TYPVAL_ENCODE_CONV_EXT_STRING
#undef TYPVAL_ENCODE_CONV_BLOB
#undef TYPVAL_ENCODE_CONV_FUNC_START
#undef TYPVAL_ENCODE_CONV_FUNC_BEFORE_ARGS
#undef TYPVAL_ENCODE_CONV_FUNC_BEFORE_SELF
#undef TYPVAL_ENCODE_CONV_FUNC_END
#undef TYPVAL_ENCODE_CONV_EMPTY_LIST
#undef TYPVAL_ENCODE_CONV_EMPTY_DICT
#undef TYPVAL_ENCODE_CONV_LIST_START
#undef TYPVAL_ENCODE_CONV_REAL_LIST_AFTER_START
#undef TYPVAL_ENCODE_CONV_LIST_BETWEEN_ITEMS
#undef TYPVAL_ENCODE_CONV_LIST_END
#undef TYPVAL_ENCODE_CONV_DICT_START
#undef TYPVAL_ENCODE_CONV_REAL_DICT_AFTER_START
#undef TYPVAL_ENCODE_SPECIAL_DICT_KEY_CHECK
#undef TYPVAL_ENCODE_CONV_DICT_AFTER_KEY
#undef TYPVAL_ENCODE_CONV_DICT_BETWEEN_ITEMS
#undef TYPVAL_ENCODE_CONV_DICT_END
#undef TYPVAL_ENCODE_CONV_RECURSE

/// Free memory for a variable value and set the value to NULL or 0
///
/// @param[in,out]  tv  Value to free.
void tv_clear(typval_T *const tv)
{
  if (tv == NULL || tv->v_type == VAR_UNKNOWN) {
    return;
  }

  // WARNING: do not translate the string here, gettext is slow and function
  // is used *very* often. At the current state encode_vim_to_nothing() does
  // not error out and does not use the argument anywhere.
  //
  // If situation changes and this argument will be used, translate it in the
  // place where it is used.
  const int evn_ret = encode_vim_to_nothing(NULL, tv, "tv_clear() argument");
  (void)evn_ret;
  assert(evn_ret == OK);
}

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

void nvim_typval_error_string_required(int idx) { semsg(_(e_string_required_for_argument_nr), idx); }

void nvim_typval_error_nonempty_string_required(int idx) { semsg(_(e_non_empty_string_required_for_argument_nr), idx); }

void nvim_typval_error_number_required(int idx) { semsg(_(e_number_required_for_argument_nr), idx); }

void nvim_typval_error_float_or_number_required(int idx) { semsg(_(e_float_or_number_required_for_argument_nr), idx); }

void nvim_typval_error_bool_required(int idx) { semsg(_(e_bool_required_for_argument_nr), idx); }

void nvim_typval_error_blob_required(int idx) { semsg(_(e_blob_required_for_argument_nr), idx); }

void nvim_typval_error_list_required(int idx) { semsg(_(e_list_required_for_argument_nr), idx); }

void nvim_typval_error_dict_required(int idx) { semsg(_(e_dict_required_for_argument_nr), idx); }

void nvim_typval_error_nonnull_dict_required(int idx) { semsg(_(e_non_null_dict_required_for_argument_nr), idx); }

void nvim_typval_error_string_or_number_required(int idx)
{
  semsg(_(e_string_or_number_required_for_argument_nr), idx);
}

void nvim_typval_error_string_or_list_required(int idx) { semsg(_(e_string_or_list_required_for_argument_nr), idx); }

void nvim_typval_error_string_list_or_blob_required(int idx)
{
  semsg(_(e_string_list_or_blob_required_for_argument_nr), idx);
}

void nvim_typval_error_string_list_or_dict_required(int idx)
{
  semsg(_(e_string_list_or_dict_required_for_argument_nr), idx);
}

void nvim_typval_error_string_or_func_required(int idx)
{
  semsg(_(e_string_or_function_required_for_argument_nr), idx);
}

void nvim_typval_error_list_or_blob_required(int idx) { semsg(_(e_list_or_blob_required_for_argument_nr), idx); }

// tv_check_num error messages (type-specific)

void nvim_typval_error_using_funcref_as_number(void) { emsg(_("E703: Using a Funcref as a Number")); }

void nvim_typval_error_using_list_as_number(void) { emsg(_("E745: Using a List as a Number")); }

void nvim_typval_error_using_dict_as_number(void) { emsg(_("E728: Using a Dictionary as a Number")); }

void nvim_typval_error_using_float_as_number(void) { emsg(_("E805: Using a Float as a Number")); }

void nvim_typval_error_using_blob_as_number(void) { emsg(_("E974: Using a Blob as a Number")); }

void nvim_typval_error_using_invalid_as_number(void) { emsg(_("E685: using an invalid value as a Number")); }

// tv_check_str error messages (type-specific)

void nvim_typval_error_using_funcref_as_string(void) { emsg(_("E729: Using a Funcref as a String")); }

void nvim_typval_error_using_list_as_string(void) { emsg(_("E730: Using a List as a String")); }

void nvim_typval_error_using_dict_as_string(void) { emsg(_("E731: Using a Dictionary as a String")); }

void nvim_typval_error_using_blob_as_string(void) { emsg(_("E976: Using a Blob as a String")); }

void nvim_typval_error_using_invalid_as_string(void) { emsg(_(e_using_invalid_value_as_string)); }

// tv_check_str_or_nr error messages (type-specific)

void nvim_typval_error_str_or_nr_float(void) { emsg(_("E805: Expected a Number or a String, Float found")); }

void nvim_typval_error_str_or_nr_funcref(void) { emsg(_("E703: Expected a Number or a String, Funcref found")); }

void nvim_typval_error_str_or_nr_list(void) { emsg(_("E745: Expected a Number or a String, List found")); }

void nvim_typval_error_str_or_nr_dict(void) { emsg(_("E728: Expected a Number or a String, Dictionary found")); }

void nvim_typval_error_str_or_nr_blob(void) { emsg(_("E974: Expected a Number or a String, Blob found")); }

void nvim_typval_error_str_or_nr_bool(void) { emsg(_("E5299: Expected a Number or a String, Boolean found")); }

void nvim_typval_error_str_or_nr_special(void) { emsg(_("E5300: Expected a Number or a String")); }

void nvim_typval_error_str_or_nr_unknown(void) { semsg(_(e_intern2), "tv_check_str_or_nr(UNKNOWN)"); }

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

/// Clear a typval (tv_clear wrapper for Rust).
void nvim_tv_clear(typval_T *tv) { tv_clear(tv); }

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

/// Look up a key in a dict, returning a dictitem pointer or NULL (accessor for Rust).
/// Directly uses hash table to avoid circular dependency with Rust tv_dict_find.
dictitem_T *nvim_dict_find(const dict_T *d, const char *key, ptrdiff_t len)
{
  if (d == NULL) {
    return NULL;
  }
  hashitem_T *const hi = (len < 0
                          ? hash_find(&d->dv_hashtab, key)
                          : hash_find_len(&d->dv_hashtab, key, (size_t)len));
  if (HASHITEM_EMPTY(hi)) {
    return NULL;
  }
  return TV_DICT_HI2DI(hi);
}

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
/// Emit tv_item_lock nested too deep error (accessor for Rust).
void nvim_emsg_item_lock_nested(void) { emsg(_(e_variable_nested_too_deep_for_unlock)); }

/// Emit blob index out of range error (accessor for Rust).
void nvim_semsg_blobidx(int64_t idx) { semsg(_(e_blobidx), idx); }

/// Emit blob wrong number of bytes error (accessor for Rust).
void nvim_emsg_blob_wrong_bytes(void) { emsg(_("E972: Blob value does not have the right number of bytes")); }

/// Emit tv_get_float error for funcref (accessor for Rust).
void nvim_emsg_float_funcref(void) { emsg(_("E891: Using a Funcref as a Float")); }
/// Emit tv_get_float error for string (accessor for Rust).
void nvim_emsg_float_string(void) { emsg(_("E892: Using a String as a Float")); }
/// Emit tv_get_float error for list (accessor for Rust).
void nvim_emsg_float_list(void) { emsg(_("E893: Using a List as a Float")); }
/// Emit tv_get_float error for dict (accessor for Rust).
void nvim_emsg_float_dict(void) { emsg(_("E894: Using a Dictionary as a Float")); }
/// Emit tv_get_float error for bool (accessor for Rust).
void nvim_emsg_float_bool(void) { emsg(_("E362: Using a boolean value as a Float")); }
/// Emit tv_get_float error for special (accessor for Rust).
void nvim_emsg_float_special(void) { emsg(_("E907: Using a special value as a Float")); }
/// Emit tv_get_float error for blob (accessor for Rust).
void nvim_emsg_float_blob(void) { emsg(_("E975: Using a Blob as a Float")); }
/// Emit tv_get_float error for unknown (accessor for Rust).
void nvim_emsg_float_unknown(void) { semsg(_(e_intern2), "tv_get_float(UNKNOWN)"); }

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


/// Emit the E685 intern2 error for tv_get_number(UNKNOWN).
void nvim_emsg_get_number_unknown(void) { semsg(_(e_intern2), "tv_get_number(UNKNOWN)"); }

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

/// Get the refcount of a blob (accessor for Rust).
int nvim_blob_get_refcount(const blob_T *b) { return b->bv_refcount; }

/// Decrement blob refcount and return new value (accessor for Rust).
int nvim_blob_dec_refcount(blob_T *b) { return --b->bv_refcount; }

/// Set ga_maxlen of a blob (accessor for Rust).
void nvim_blob_set_ga_maxlen(blob_T *b, int n) { b->bv_ga.ga_maxlen = n; }

/// Get ga_maxlen of a blob (accessor for Rust).
int nvim_blob_get_ga_maxlen(const blob_T *b) { return b->bv_ga.ga_maxlen; }

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

/// Emit e_invalid_value_for_blob_nr error (accessor for Rust).
void nvim_semsg_blob_invalid_value(int64_t n)
{
  semsg(_(e_invalid_value_for_blob_nr), (int)n);
}

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

/// Free a dict completely (accessor for Rust).
/// Delegates to C tv_dict_free (before migration).
void nvim_dict_free_impl(dict_T *d) { tv_dict_free(d); }

/// Decrement dict refcount and free if zero (accessor for Rust).
/// Delegates to C tv_dict_unref (before migration).
void nvim_dict_unref_impl(dict_T *d) { tv_dict_unref(d); }

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
  return tv_dict_item_copy(di);
}

/// Allocate a dict item with given key (accessor for Rust, avoids circular call).
dictitem_T *nvim_dict_item_alloc_impl(const char *key)
{
  return tv_dict_item_alloc(key);
}

/// Call tv_dict_watcher_notify (accessor for Rust tv_dict_extend).
void nvim_dict_watcher_notify(dict_T *d, const char *key, typval_T *newtv, typval_T *oldtv)
{
  tv_dict_watcher_notify(d, key, newtv, oldtv);
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

/// Get di_key from a dictitem as mutable (accessor for Rust).
const char *nvim_dictitem_get_key_ptr(const dictitem_T *di) { return di->di_key; }

/// Emit "E737: Key already exists" error (accessor for Rust tv_dict_extend).
void nvim_semsg_key_exists(const char *key) { semsg(_("E737: Key already exists: %s"), key); }

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

/// Get dv_used_prev from a dict (accessor for Rust).
dict_T *nvim_dict_get_used_prev(const dict_T *d) { return d->dv_used_prev; }

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

/// Get v_lock from a typval (accessor for Rust).
int nvim_tv_get_lock(const typval_T *tv) { return (int)tv->v_lock; }

/// Get dv_scope from a dict (accessor for Rust).
int nvim_dict_get_scope(const dict_T *d) { return (int)d->dv_scope; }

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

/// Remove a hash item for key from dict's hashtab (accessor for Rust).
/// Only removes from hash, does NOT free the dictitem.
void nvim_dict_remove_key(dict_T *d, const char *key)
{
  hashitem_T *hi = hash_find(&d->dv_hashtab, key);
  if (!HASHITEM_EMPTY(hi)) {
    hash_remove(&d->dv_hashtab, hi);
  }
}

// Phase 5 accessor functions for Rust list infrastructure migration

/// Get gc_first_list global (accessor for Rust).
list_T *nvim_gc_first_list_get(void) { return gc_first_list; }

/// Set gc_first_list global (accessor for Rust).
void nvim_gc_first_list_set(list_T *l) { gc_first_list = l; }

/// Set lv_used_prev on a list (accessor for Rust).
void nvim_list_set_used_prev(list_T *l, list_T *prev) { l->lv_used_prev = prev; }

/// Set lv_used_next on a list (accessor for Rust).
void nvim_list_set_used_next(list_T *l, list_T *next) { l->lv_used_next = next; }

/// Get lv_used_prev from a list (accessor for Rust).
list_T *nvim_list_get_used_prev(const list_T *l) { return l->lv_used_prev; }

/// Initialize lua_table_ref on a list to LUA_NOREF (accessor for Rust).
void nvim_list_init_lua_ref(list_T *l) { l->lua_table_ref = LUA_NOREF; }

/// Clear lua_table_ref on a list using NLUA_CLEAR_REF (accessor for Rust).
void nvim_list_clear_lua_ref(list_T *l) { NLUA_CLEAR_REF(l->lua_table_ref); }

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

/// Self-contained list watch_fix (accessor for Rust).
/// Avoids circular call: advances watchers past the removed item.
void nvim_list_watch_fix(list_T *l, const listitem_T *item)
{
  for (listwatch_T *lw = l->lv_watch; lw != NULL; lw = lw->lw_next) {
    if (lw->lw_item == item) {
      lw->lw_item = item->li_next;
    }
  }
}

/// Clear a list item's tv and free it (accessor for Rust).
void nvim_list_item_clear_free(listitem_T *li)
{
  tv_clear(&li->li_tv);
  xfree(li);
}

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

/// Emit e_invrange error (accessor for Rust).
void nvim_emsg_invrange(void) { emsg(_(e_invrange)); }

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

/// Emit "E710: List value has more items than target" (accessor for Rust).
void nvim_emsg_list_more_items(void) { emsg(_("E710: List value has more items than target")); }

/// Emit "E711: List value has not enough items" (accessor for Rust).
void nvim_emsg_list_not_enough_items(void) { emsg(_("E711: List value has not enough items")); }

/// Get listitem v_lock field via TV_LIST_ITEM_TV (accessor for Rust).
/// Needed by tv_list_assign_range lock check.
int nvim_listitem_get_v_lock(const listitem_T *li) { return TV_LIST_ITEM_TV(li)->v_lock; }

// Phase 6e: tv_list_copy, f_items accessors

/// Call tv_dict2list(argvars, rettv, kDict2ListItems) (accessor for Rust f_items dict case).
void nvim_tv_dict2list_items(typval_T *argvars, typval_T *rettv)
{
  tv_dict2list(argvars, rettv, kDict2ListItems);
}

// Phase 4 additional accessors

/// Set dv_refcount on a dict (accessor for Rust tv_dict_unref).
void nvim_dict_set_refcount(dict_T *d, int rc) { d->dv_refcount = rc; }

/// Free a dict item without clearing its tv (for error paths in tv_dict_copy where tv is unset).
void nvim_dict_item_free_raw(dictitem_T *di) { xfree(di); }

/// Get tv_dict_len (number of items) for a dict (accessor for Rust tv_dict_equal).
int nvim_dict_get_len_impl(const dict_T *d) { return tv_dict_len(d); }

/// Get dv_scope from a dict (accessor for Rust tv_dict_extend).
int nvim_dict_get_scope_impl(const dict_T *d) { return (int)d->dv_scope; }

// Phase 6g: f_keys, f_values accessors

/// Call tv_dict2list(argvars, rettv, kDict2ListKeys) (accessor for Rust f_keys).
void nvim_tv_dict2list_keys(typval_T *argvars, typval_T *rettv)
{
  tv_dict2list(argvars, rettv, kDict2ListKeys);
}

/// Call tv_dict2list(argvars, rettv, kDict2ListValues) (accessor for Rust f_values).
void nvim_tv_dict2list_values(typval_T *argvars, typval_T *rettv)
{
  tv_dict2list(argvars, rettv, kDict2ListValues);
}

// Phase 6j: tv_dict_to_env accessor

/// Format a dict item as "key=value" env string (accessor for Rust tv_dict_to_env).
/// Returns allocated string. Caller must free with xfree.
char *nvim_dictitem_format_env(const dictitem_T *di)
{
  const char *str = tv_get_string(&di->di_tv);
  size_t len = strlen(di->di_key) + strlen(str) + 2;  // "=" and NUL
  char *entry = xmalloc(len);
  snprintf(entry, len, "%s=%s", di->di_key, str);
  return entry;
}

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

// nvim_partial_get_name: already defined in eval/userfunc.c

// nvim_tv_set_partial, nvim_tv_set_special: already defined in eval/vars.c

/// Set vval.v_string (takes ownership) on a typval without changing type (accessor for Rust).
void nvim_tv_set_vstring_owned(typval_T *tv, char *s) { tv->vval.v_string = s; }

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

/// Emit E6000 error (accessor for Rust tv_dict_get_callback).
void nvim_emsg_not_func_or_funcname(void)
{
  emsg(_("E6000: Argument is not a function or function name"));
}

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

/// Emit E702 sort compare function failed (accessor for Rust do_sort).
void nvim_emsg_sort_failed(void) { emsg(_("E702: Sort compare function failed")); }

/// Emit E882 uniq compare function failed (accessor for Rust do_uniq).
void nvim_emsg_uniq_failed(void) { emsg(_("E882: Uniq compare function failed")); }

/// semsg(_(e_listarg), fname) wrapper for Rust do_sort_uniq.
void nvim_emsg_listarg(const char *fname) { semsg(_(e_listarg), fname); }

/// emsg(_(e_invarg)) wrapper for Rust (accessor for parse_sort_uniq_args, f_list2str).
void nvim_emsg_invarg(void) { emsg(_(e_invarg)); }

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

/// emsg(_(e_listreq)) wrapper for Rust f_join.
void nvim_emsg_e_listreq(void) { emsg(_(e_listreq)); }

/// Join list into a newly allocated NUL-terminated string.
/// Returns "" for empty lists, NULL if encoding any item fails.
/// Caller must xfree the result.
char *nvim_list_join_to_string(list_T *l, const char *sep)
{
  if (!tv_list_len(l)) {
    return xstrdup("");
  }

  // Stringify each item via encode_tv2echo, accumulate in a per-item array.
  typedef struct { char *s; char *tofree; } JoinItem;
  size_t nitems = (size_t)tv_list_len(l);
  JoinItem *items = xmalloc(nitems * sizeof(JoinItem));
  size_t sumlen = 0;
  size_t i = 0;
  bool failed = false;

  TV_LIST_ITER(l, item, {
    if (got_int) { break; }
    size_t len;
    char *s = encode_tv2echo(TV_LIST_ITEM_TV(item), &len);
    if (s == NULL) { failed = true; break; }
    sumlen += len;
    items[i].s = items[i].tofree = s;
    i++;
    line_breakcheck();
  });

  if (failed || got_int) {
    for (size_t j = 0; j < i; j++) { xfree(items[j].tofree); }
    xfree(items);
    return NULL;
  }

  // Build result.
  size_t seplen = strlen(sep);
  if (i >= 2) { sumlen += seplen * (i - 1); }
  char *result = xmalloc(sumlen + 1);
  char *p = result;
  for (size_t j = 0; j < i; j++) {
    if (j > 0) { memcpy(p, sep, seplen); p += seplen; }
    if (items[j].s != NULL) { size_t slen = strlen(items[j].s); memcpy(p, items[j].s, slen); p += slen; }
    xfree(items[j].tofree);
  }
  *p = NUL;
  xfree(items);
  return result;
}

/// list2str: convert list of codepoints to UTF-8 string.
/// Returns allocated string. Caller must xfree.
char *nvim_f_list2str_from_list(list_T *l)
{
  garray_T ga;
  ga_init(&ga, 1, 80);
  char buf[MB_MAXBYTES + 1];
  TV_LIST_ITER_CONST(l, li, {
    buf[utf_char2bytes((int)tv_get_number(TV_LIST_ITEM_TV(li)), buf)] = NUL;
    ga_concat(&ga, buf);
  });
  ga_append(&ga, NUL);
  return ga.ga_data;
}


