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


/// struct storing information about current sort
typedef struct {
  int item_compare_ic;
  bool item_compare_lc;
  bool item_compare_numeric;
  bool item_compare_numbers;
  bool item_compare_float;
  const char *item_compare_func;
  partial_T *item_compare_partial;
  dict_T *item_compare_selfdict;
  bool item_compare_func_err;
} sortinfo_T;

/// Structure representing one list item, used for sort array.
typedef struct {
  listitem_T *item;  ///< Sorted list item.
  int idx;  ///< Sorted list item index.
} ListSortItem;

typedef int (*ListSorter)(const void *, const void *);

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
/// @warning Allocated item is not initialized, do not forget to initialize it
///          and specifically set lv_lock.
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

/// "join()" function
void f_join(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  if (argvars[0].v_type != VAR_LIST) {
    emsg(_(e_listreq));
    return;
  }
  const char *const sep = (argvars[1].v_type == VAR_UNKNOWN
                           ? " "
                           : tv_get_string_chk(&argvars[1]));

  rettv->v_type = VAR_STRING;

  if (sep != NULL) {
    garray_T ga;
    ga_init(&ga, (int)sizeof(char), 80);
    tv_list_join(&ga, argvars[0].vval.v_list, sep);
    ga_append(&ga, NUL);
    rettv->vval.v_string = ga.ga_data;
  } else {
    rettv->vval.v_string = NULL;
  }
}

/// "list2str()" function
void f_list2str(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  garray_T ga;

  rettv->v_type = VAR_STRING;
  rettv->vval.v_string = NULL;
  if (argvars[0].v_type != VAR_LIST) {
    emsg(_(e_invarg));
    return;
  }

  list_T *const l = argvars[0].vval.v_list;
  if (l == NULL) {
    return;  // empty list results in empty string
  }

  ga_init(&ga, 1, 80);
  char buf[MB_MAXBYTES + 1];

  TV_LIST_ITER_CONST(l, li, {
    buf[utf_char2bytes((int)tv_get_number(TV_LIST_ITEM_TV(li)), buf)] = NUL;
    ga_concat(&ga, buf);
  });
  ga_append(&ga, NUL);

  rettv->vval.v_string = ga.ga_data;
}

// tv_list_remove migrated to Rust (Phase 6c)

static sortinfo_T *sortinfo = NULL;

#define ITEM_COMPARE_FAIL 999

/// Compare functions for f_sort() and f_uniq() below.
static int item_compare(const void *s1, const void *s2, bool keep_zero)
{
  ListSortItem *const si1 = (ListSortItem *)s1;
  ListSortItem *const si2 = (ListSortItem *)s2;

  typval_T *const tv1 = TV_LIST_ITEM_TV(si1->item);
  typval_T *const tv2 = TV_LIST_ITEM_TV(si2->item);

  int res;

  if (sortinfo->item_compare_numbers) {
    const varnumber_T v1 = tv_get_number(tv1);
    const varnumber_T v2 = tv_get_number(tv2);

    res = v1 == v2 ? 0 : v1 > v2 ? 1 : -1;
    goto item_compare_end;
  }

  if (sortinfo->item_compare_float) {
    const float_T v1 = tv_get_float(tv1);
    const float_T v2 = tv_get_float(tv2);

    res = v1 == v2 ? 0 : v1 > v2 ? 1 : -1;
    goto item_compare_end;
  }

  char *tofree1 = NULL;
  char *tofree2 = NULL;
  char *p1;
  char *p2;

  // encode_tv2string() puts quotes around a string and allocates memory.  Don't
  // do that for string variables. Use a single quote when comparing with
  // a non-string to do what the docs promise.
  if (tv1->v_type == VAR_STRING) {
    if (tv2->v_type != VAR_STRING || sortinfo->item_compare_numeric) {
      p1 = "'";
    } else {
      p1 = tv1->vval.v_string;
    }
  } else {
    tofree1 = p1 = encode_tv2string(tv1, NULL);
  }
  if (tv2->v_type == VAR_STRING) {
    if (tv1->v_type != VAR_STRING || sortinfo->item_compare_numeric) {
      p2 = "'";
    } else {
      p2 = tv2->vval.v_string;
    }
  } else {
    tofree2 = p2 = encode_tv2string(tv2, NULL);
  }
  if (p1 == NULL) {
    p1 = "";
  }
  if (p2 == NULL) {
    p2 = "";
  }
  if (!sortinfo->item_compare_numeric) {
    if (sortinfo->item_compare_lc) {
      res = strcoll(p1, p2);
    } else {
      res = sortinfo->item_compare_ic ? STRICMP(p1, p2) : strcmp(p1, p2);
    }
  } else {
    double n1 = strtod(p1, &p1);
    double n2 = strtod(p2, &p2);
    res = n1 == n2 ? 0 : n1 > n2 ? 1 : -1;
  }

  xfree(tofree1);
  xfree(tofree2);

item_compare_end:
  // When the result would be zero, compare the item indexes.  Makes the
  // sort stable.
  if (res == 0 && !keep_zero) {
    // WARNING: When using uniq si1 and si2 are actually listitem_T **, no
    // indexes are there.
    res = si1->idx > si2->idx ? 1 : -1;
  }
  return res;
}

static int item_compare_keeping_zero(const void *s1, const void *s2) { return item_compare(s1, s2, true); }

static int item_compare_not_keeping_zero(const void *s1, const void *s2) { return item_compare(s1, s2, false); }

static int item_compare2(const void *s1, const void *s2, bool keep_zero)
{
  typval_T rettv;
  typval_T argv[3];
  const char *func_name;
  partial_T *partial = sortinfo->item_compare_partial;

  // shortcut after failure in previous call; compare all items equal
  if (sortinfo->item_compare_func_err) {
    return 0;
  }

  ListSortItem *si1 = (ListSortItem *)s1;
  ListSortItem *si2 = (ListSortItem *)s2;

  if (partial == NULL) {
    func_name = sortinfo->item_compare_func;
  } else {
    func_name = rs_partial_name(partial);
  }

  // Copy the values.  This is needed to be able to set v_lock to VAR_FIXED
  // in the copy without changing the original list items.
  tv_copy(TV_LIST_ITEM_TV(si1->item), &argv[0]);
  tv_copy(TV_LIST_ITEM_TV(si2->item), &argv[1]);

  rettv.v_type = VAR_UNKNOWN;  // tv_clear() uses this
  funcexe_T funcexe = FUNCEXE_INIT;
  funcexe.fe_evaluate = true;
  funcexe.fe_partial = partial;
  funcexe.fe_selfdict = sortinfo->item_compare_selfdict;
  int res = call_func(func_name, -1, &rettv, 2, argv, &funcexe);
  tv_clear(&argv[0]);
  tv_clear(&argv[1]);

  if (res == FAIL) {
    // XXX: ITEM_COMPARE_FAIL is unused
    res = ITEM_COMPARE_FAIL;
    sortinfo->item_compare_func_err = true;
  } else {
    res = (int)tv_get_number_chk(&rettv, &sortinfo->item_compare_func_err);
    if (res > 0) {
      res = 1;
    } else if (res < 0) {
      res = -1;
    }
  }
  if (sortinfo->item_compare_func_err) {
    res = ITEM_COMPARE_FAIL;  // return value has wrong type
  }
  tv_clear(&rettv);

  // When the result would be zero, compare the pointers themselves.  Makes
  // the sort stable.
  if (res == 0 && !keep_zero) {
    // WARNING: When using uniq si1 and si2 are actually listitem_T **, no
    // indexes are there.
    res = si1->idx > si2->idx ? 1 : -1;
  }

  return res;
}

static int item_compare2_keeping_zero(const void *s1, const void *s2) { return item_compare2(s1, s2, true); }

static int item_compare2_not_keeping_zero(const void *s1, const void *s2) { return item_compare2(s1, s2, false); }

/// sort() List "l"
static void do_sort(list_T *l, sortinfo_T *info)
{
  const int len = tv_list_len(l);

  // Make an array with each entry pointing to an item in the List.
  ListSortItem *ptrs = xmalloc((size_t)((unsigned)len * sizeof(ListSortItem)));

  // f_sort(): ptrs will be the list to sort
  int i = 0;
  TV_LIST_ITER(l, li, {
    ptrs[i].item = li;
    ptrs[i].idx = i;
    i++;
  });

  info->item_compare_func_err = false;
  ListSorter item_compare_func = ((info->item_compare_func == NULL
                                   && info->item_compare_partial == NULL)
                                  ? item_compare_not_keeping_zero
                                  : item_compare2_not_keeping_zero);

  // Sort the array with item pointers.
  qsort(ptrs, (size_t)len, sizeof(ListSortItem), item_compare_func);
  if (!info->item_compare_func_err) {
    // Clear the list and append the items in the sorted order.
    l->lv_first = NULL;
    l->lv_last = NULL;
    l->lv_idx_item = NULL;
    l->lv_len = 0;
    for (i = 0; i < len; i++) {
      tv_list_append(l, ptrs[i].item);
    }
  }
  if (info->item_compare_func_err) {
    emsg(_("E702: Sort compare function failed"));
  }

  xfree(ptrs);
}

/// uniq() List "l"
static void do_uniq(list_T *l, sortinfo_T *info)
{
  const int len = tv_list_len(l);

  // Make an array with each entry pointing to an item in the List.
  ListSortItem *ptrs = xmalloc((size_t)((unsigned)len * sizeof(ListSortItem)));

  // f_uniq(): ptrs will be a stack of items to remove.

  info->item_compare_func_err = false;
  ListSorter item_compare_func = ((info->item_compare_func == NULL
                                   && info->item_compare_partial == NULL)
                                  ? item_compare_keeping_zero
                                  : item_compare2_keeping_zero);

  for (listitem_T *li = TV_LIST_ITEM_NEXT(l, tv_list_first(l)); li != NULL;) {
    listitem_T *const prev_li = TV_LIST_ITEM_PREV(l, li);
    if (item_compare_func(&prev_li, &li) == 0) {
      li = tv_list_item_remove(l, li);
    } else {
      li = TV_LIST_ITEM_NEXT(l, li);
    }
    if (info->item_compare_func_err) {
      emsg(_("E882: Uniq compare function failed"));
      break;
    }
  }

  xfree(ptrs);
}

/// Parse the optional arguments to sort() and uniq() and return the values in "info".
static int parse_sort_uniq_args(typval_T *argvars, sortinfo_T *info)
{
  info->item_compare_ic = false;
  info->item_compare_lc = false;
  info->item_compare_numeric = false;
  info->item_compare_numbers = false;
  info->item_compare_float = false;
  info->item_compare_func = NULL;
  info->item_compare_partial = NULL;
  info->item_compare_selfdict = NULL;

  if (argvars[1].v_type == VAR_UNKNOWN) {
    return OK;
  }

  // optional second argument: {func}
  if (argvars[1].v_type == VAR_FUNC) {
    info->item_compare_func = argvars[1].vval.v_string;
  } else if (argvars[1].v_type == VAR_PARTIAL) {
    info->item_compare_partial = argvars[1].vval.v_partial;
  } else {
    bool error = false;
    int nr = (int)tv_get_number_chk(&argvars[1], &error);
    if (error) {
      return FAIL;  // type error; errmsg already given
    }
    if (nr == 1) {
      info->item_compare_ic = true;
    } else if (argvars[1].v_type != VAR_NUMBER) {
      info->item_compare_func = tv_get_string(&argvars[1]);
    } else if (nr != 0) {
      emsg(_(e_invarg));
      return FAIL;
    }
    if (info->item_compare_func != NULL) {
      if (*info->item_compare_func == NUL) {
        // empty string means default sort
        info->item_compare_func = NULL;
      } else if (strcmp(info->item_compare_func, "n") == 0) {
        info->item_compare_func = NULL;
        info->item_compare_numeric = true;
      } else if (strcmp(info->item_compare_func, "N") == 0) {
        info->item_compare_func = NULL;
        info->item_compare_numbers = true;
      } else if (strcmp(info->item_compare_func, "f") == 0) {
        info->item_compare_func = NULL;
        info->item_compare_float = true;
      } else if (strcmp(info->item_compare_func, "i") == 0) {
        info->item_compare_func = NULL;
        info->item_compare_ic = true;
      } else if (strcmp(info->item_compare_func, "l") == 0) {
        info->item_compare_func = NULL;
        info->item_compare_lc = true;
      }
    }
  }

  if (argvars[2].v_type != VAR_UNKNOWN) {
    // optional third argument: {dict}
    if (tv_check_for_dict_arg(argvars, 2) == FAIL) {
      return FAIL;
    }
    info->item_compare_selfdict = argvars[2].vval.v_dict;
  }

  return OK;
}

/// "sort()" or "uniq()" function
static void do_sort_uniq(typval_T *argvars, typval_T *rettv, bool sort)
{
  if (argvars[0].v_type != VAR_LIST) {
    semsg(_(e_listarg), sort ? "sort()" : "uniq()");
    return;
  }

  // Pointer to current info struct used in compare function. Save and restore
  // the current one for nested calls.
  sortinfo_T info;
  sortinfo_T *old_sortinfo = sortinfo;
  sortinfo = &info;

  const char *const arg_errmsg = (sort ? N_("sort() argument") : N_("uniq() argument"));
  list_T *const l = argvars[0].vval.v_list;
  if (value_check_lock(tv_list_locked(l), arg_errmsg, TV_TRANSLATE)) {
    goto theend;
  }
  tv_list_set_ret(rettv, l);

  const int len = tv_list_len(l);
  if (len <= 1) {
    goto theend;  // short list sorts pretty quickly
  }
  if (parse_sort_uniq_args(argvars, &info) == FAIL) {
    goto theend;
  }

  if (sort) {
    do_sort(l, &info);
  } else {
    do_uniq(l, &info);
  }

theend:
  sortinfo = old_sortinfo;
}

/// "sort({list})" function
void f_sort(typval_T *argvars, typval_T *rettv, EvalFuncData fptr) { do_sort_uniq(argvars, rettv, true); }

/// "uniq({list})" function
void f_uniq(typval_T *argvars, typval_T *rettv, EvalFuncData fptr) { do_sort_uniq(argvars, rettv, false); }

// tv_list_equal, tv_list_find_nr, tv_list_find_str migrated to Rust (Phase 2)

//{{{2 Indexing/searching

// tv_list_find_index deleted (dead code, Phase 6e)

//{{{1 Dictionaries
//{{{2 Dictionary watchers

/// Perform all necessary cleanup for a `DictWatcher` instance
///
/// @param  watcher  Watcher to free.
static void tv_dict_watcher_free(DictWatcher *watcher)
  FUNC_ATTR_NONNULL_ALL
{
  callback_free(&watcher->callback);
  xfree(watcher->key_pattern);
  xfree(watcher);
}

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

/// Check whether two callbacks are equal
///
/// @param[in]  cb1  First callback to check.
/// @param[in]  cb2  Second callback to check.
///
/// @return True if they are equal, false otherwise.
bool tv_callback_equal(const Callback *cb1, const Callback *cb2)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_WARN_UNUSED_RESULT
{
  if (cb1->type != cb2->type) {
    return false;
  }
  switch (cb1->type) {
  case kCallbackFuncref:
    return strcmp(cb1->data.funcref, cb2->data.funcref) == 0;
  case kCallbackPartial:
    // FIXME: this is inconsistent with tv_equal but is needed for precision
    // maybe change dictwatcheradd to return a watcher id instead?
    return cb1->data.partial == cb2->data.partial;
  case kCallbackLua:
    return cb1->data.luaref == cb2->data.luaref;
  case kCallbackNone:
    return true;
  }
  abort();
  return false;
}

/// Unref/free callback
void callback_free(Callback *callback)
  FUNC_ATTR_NONNULL_ALL
{
  switch (callback->type) {
  case kCallbackFuncref:
    func_unref(callback->data.funcref);
    xfree(callback->data.funcref);
    break;
  case kCallbackPartial:
    partial_unref(callback->data.partial);
    break;
  case kCallbackLua:
    NLUA_CLEAR_REF(callback->data.luaref);
    break;
  case kCallbackNone:
    break;
  }
  callback->type = kCallbackNone;
  callback->data.funcref = NULL;
}

/// Copy a callback into a typval_T.
void callback_put(Callback *cb, typval_T *tv)
  FUNC_ATTR_NONNULL_ALL
{
  switch (cb->type) {
  case kCallbackPartial:
    tv->v_type = VAR_PARTIAL;
    tv->vval.v_partial = cb->data.partial;
    cb->data.partial->pt_refcount++;
    break;
  case kCallbackFuncref:
    tv->v_type = VAR_FUNC;
    tv->vval.v_string = xstrdup(cb->data.funcref);
    func_ref(cb->data.funcref);
    break;
  case kCallbackLua:
  // TODO(tjdevries): Unified Callback.
  // At this point this isn't possible, but it'd be nice to put
  // these handled more neatly in one place.
  // So instead, we just do the default and put nil
  default:
    tv->v_type = VAR_SPECIAL;
    tv->vval.v_special = kSpecialVarNull;
    break;
  }
}

// Copy callback from "src" to "dest", incrementing the refcounts.
void callback_copy(Callback *dest, Callback *src)
  FUNC_ATTR_NONNULL_ALL
{
  dest->type = src->type;
  switch (src->type) {
  case kCallbackPartial:
    dest->data.partial = src->data.partial;
    dest->data.partial->pt_refcount++;
    break;
  case kCallbackFuncref:
    dest->data.funcref = xstrdup(src->data.funcref);
    func_ref(src->data.funcref);
    break;
  case kCallbackLua:
    dest->data.luaref = api_new_luaref(src->data.luaref);
    break;
  default:
    dest->data.funcref = NULL;
    break;
  }
}

/// Generate a string description of a callback
char *callback_to_string(Callback *cb, Arena *arena)
{
  if (cb->type == kCallbackLua) {
    return nlua_funcref_str(cb->data.luaref, arena);
  }

  const size_t msglen = 100;
  char *msg = xmallocz(msglen);

  switch (cb->type) {
  case kCallbackFuncref:
    // TODO(tjdevries): Is this enough space for this?
    snprintf(msg, msglen, "<vim function: %s>", cb->data.funcref);
    break;
  case kCallbackPartial:
    snprintf(msg, msglen, "<vim partial: %s>", cb->data.partial->pt_name);
    break;
  default:
    *msg = NUL;
    break;
  }
  return msg;
}

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

/// Test if `key` matches with with `watcher->key_pattern`
///
/// @param[in]  watcher  Watcher to check key pattern from.
/// @param[in]  key  Key to check.
///
/// @return true if key matches, false otherwise.
static bool tv_dict_watcher_matches(DictWatcher *watcher, const char *const key)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_PURE
{
  // For now only allow very simple globbing in key patterns: a '*' at the end
  // of the string means it should match everything up to the '*' instead of the
  // whole string.
  const size_t len = watcher->key_pattern_len;
  if (len && watcher->key_pattern[len - 1] == '*') {
    return strncmp(key, watcher->key_pattern, len - 1) == 0;
  }
  return strcmp(key, watcher->key_pattern) == 0;
}

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

/// Free items contained in a dictionary
///
/// @param[in,out]  d  Dictionary to clear.
void tv_dict_free_contents(dict_T *const d)
  FUNC_ATTR_NONNULL_ALL
{
  // Lock the hashtab, we don't want it to resize while freeing items.
  hash_lock(&d->dv_hashtab);
  assert(d->dv_hashtab.ht_locked > 0);
  HASHTAB_ITER(&d->dv_hashtab, hi, {
    // Remove the item before deleting it, just in case there is
    // something recursive causing trouble.
    dictitem_T *const di = TV_DICT_HI2DI(hi);
    hash_remove(&d->dv_hashtab, hi);
    tv_dict_item_free(di);
  });

  while (!QUEUE_EMPTY(&d->watchers)) {
    QUEUE *w = QUEUE_HEAD(&d->watchers);
    QUEUE_REMOVE(w);
    DictWatcher *watcher = tv_dict_watcher_node_data(w);
    tv_dict_watcher_free(watcher);
  }

  hash_clear(&d->dv_hashtab);
  d->dv_hashtab.ht_locked--;
  hash_init(&d->dv_hashtab);
}

/// Free a dictionary itself, ignoring items it contains
///
/// Ignores the reference count.
///
/// @param[in,out]  d  Dictionary to free.
void tv_dict_free_dict(dict_T *const d)
  FUNC_ATTR_NONNULL_ALL
{
  // Remove the dict from the list of dicts for garbage collection.
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

/// Free a dictionary, including all items it contains
///
/// Ignores the reference count.
///
/// @param  d  Dictionary to free.
void tv_dict_free(dict_T *const d)
  FUNC_ATTR_NONNULL_ALL
{
  if (tv_in_free_unref_items) {
    return;
  }

  tv_dict_free_contents(d);
  tv_dict_free_dict(d);
}

/// Unreference a dictionary
///
/// Decrements the reference count and frees dictionary when it becomes zero.
///
/// @param[in]  d  Dictionary to operate on.
void tv_dict_unref(dict_T *const d)
{
  if (d != NULL && --d->dv_refcount <= 0) {
    tv_dict_free(d);
  }
}

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

/// Get a function from a dictionary
///
/// @param[in]  d  Dictionary to get callback from.
/// @param[in]  key  Dictionary key.
/// @param[in]  key_len  Key length, may be -1 to use strlen().
/// @param[out]  result  The address where a pointer to the wanted callback
///                      will be left.
///
/// @return true/false on success/failure.
bool tv_dict_get_callback(dict_T *const d, const char *const key, const ptrdiff_t key_len,
                          Callback *const result)
  FUNC_ATTR_NONNULL_ARG(2, 4) FUNC_ATTR_WARN_UNUSED_RESULT
{
  result->type = kCallbackNone;

  dictitem_T *const di = tv_dict_find(d, key, key_len);

  if (di == NULL) {
    return true;
  }

  if (!tv_is_func(di->di_tv) && di->di_tv.v_type != VAR_STRING) {
    emsg(_("E6000: Argument is not a function or function name"));
    return false;
  }

  typval_T tv;
  tv_copy(&di->di_tv, &tv);
  set_selfdict(&tv, d);
  const bool res = rs_callback_from_typval(result, &tv);
  tv_clear(&tv);
  return res;
}

/// Check for adding a function to g: or l:.
/// If the name is wrong give an error message and return true.
int tv_dict_wrong_func_name(dict_T *d, typval_T *tv, const char *name)
{
  return (d == get_globvar_dict() || &d->dv_hashtab == get_funccal_local_ht())
         && tv_is_func(*tv)
         && var_wrong_func_name(name, true);
}

//{{{2 dict_add*
// tv_dict_add, tv_dict_add_list, tv_dict_add_tv, tv_dict_add_dict,
// tv_dict_add_nr, tv_dict_add_float, tv_dict_add_bool, tv_dict_add_str,
// tv_dict_add_str_len, tv_dict_add_allocated_str, tv_dict_add_func
// migrated to Rust (Phase 3)

//{{{2 Operations on the whole dict

// tv_dict_clear deleted (dead code, Phase 6h)

/// Extend dictionary with items from another dictionary
///
/// @param  d1  Dictionary to extend.
/// @param[in]  d2  Dictionary to extend with.
/// @param[in]  action  "error", "force", "move", "keep":
///                     e*, including "error": duplicate key gives an error.
///                     f*, including "force": duplicate d2 keys override d1.
///                     m*, including "move": move items instead of copying.
///                     other, including "keep": duplicate d2 keys ignored.
void tv_dict_extend(dict_T *const d1, dict_T *const d2, const char *const action)
  FUNC_ATTR_NONNULL_ALL
{
  const bool watched = tv_dict_is_watched(d1);
  const char *const arg_errmsg = _("extend() argument");
  const size_t arg_errmsg_len = strlen(arg_errmsg);

  if (*action == 'm') {
    hash_lock(&d2->dv_hashtab);  // don't rehash on hash_remove()
  }

  HASHTAB_ITER(&d2->dv_hashtab, hi2, {
    dictitem_T *const di2 = TV_DICT_HI2DI(hi2);
    dictitem_T *const di1 = tv_dict_find(d1, di2->di_key, -1);
    // Check the key to be valid when adding to any scope.
    if (d1->dv_scope != VAR_NO_SCOPE && !valid_varname(di2->di_key)) {
      break;
    }
    if (di1 == NULL) {
      if (*action == 'm') {
        // Cheap way to move a dict item from "d2" to "d1".
        // If dict_add() fails then "d2" won't be empty.
        dictitem_T *const new_di = di2;
        if (tv_dict_add(d1, new_di) == OK) {
          hash_remove(&d2->dv_hashtab, hi2);
          tv_dict_watcher_notify(d1, new_di->di_key, &new_di->di_tv, NULL);
        }
      } else {
        dictitem_T *const new_di = tv_dict_item_copy(di2);
        if (tv_dict_add(d1, new_di) == FAIL) {
          tv_dict_item_free(new_di);
        } else if (watched) {
          tv_dict_watcher_notify(d1, new_di->di_key, &new_di->di_tv, NULL);
        }
      }
    } else if (*action == 'e') {
      semsg(_("E737: Key already exists: %s"), di2->di_key);
      break;
    } else if (*action == 'f' && di2 != di1) {
      typval_T oldtv;

      if (value_check_lock(di1->di_tv.v_lock, arg_errmsg, arg_errmsg_len)
          || var_check_ro(di1->di_flags, arg_errmsg, arg_errmsg_len)) {
        break;
      }
      // Disallow replacing a builtin function.
      if (tv_dict_wrong_func_name(d1, &di2->di_tv, di2->di_key)) {
        break;
      }

      if (watched) {
        tv_copy(&di1->di_tv, &oldtv);
      }

      tv_clear(&di1->di_tv);
      tv_copy(&di2->di_tv, &di1->di_tv);

      if (watched) {
        tv_dict_watcher_notify(d1, di1->di_key, &di1->di_tv, &oldtv);
        tv_clear(&oldtv);
      }
    }
  });

  if (*action == 'm') {
    hash_unlock(&d2->dv_hashtab);
  }
}

/// Compare two dictionaries
///
/// @param[in]  d1  First dictionary.
/// @param[in]  d2  Second dictionary.
/// @param[in]  ic  True if case is to be ignored.
///
/// @return True if dictionaries are equal, false otherwise.
bool tv_dict_equal(dict_T *const d1, dict_T *const d2, const bool ic)
  FUNC_ATTR_WARN_UNUSED_RESULT
{
  if (d1 == d2) {
    return true;
  }
  if (tv_dict_len(d1) != tv_dict_len(d2)) {
    return false;
  }
  if (tv_dict_len(d1) == 0) {
    // empty and NULL dicts are considered equal
    return true;
  }
  if (d1 == NULL || d2 == NULL) {
    return false;
  }

  TV_DICT_ITER(d1, di1, {
    dictitem_T *const di2 = tv_dict_find(d2, di1->di_key, -1);
    if (di2 == NULL) {
      return false;
    }
    if (!tv_equal(&di1->di_tv, &di2->di_tv, ic)) {
      return false;
    }
  });
  return true;
}

/// Make a copy of dictionary
///
/// @param[in]  conv  If non-NULL, then all internal strings will be converted.
/// @param[in]  orig  Original dictionary to copy.
/// @param[in]  deep  If false, then shallow copy will be done.
/// @param[in]  copyID  See var_item_copy().
///
/// @return Copied dictionary. May be NULL in case original dictionary is NULL
///         or some failure happens. The refcount of the new dictionary is set
///         to 1.
dict_T *tv_dict_copy(const vimconv_T *const conv, dict_T *const orig, const bool deep,
                     const int copyID)
{
  if (orig == NULL) {
    return NULL;
  }

  dict_T *copy = tv_dict_alloc();
  if (copyID != 0) {
    orig->dv_copyID = copyID;
    orig->dv_copydict = copy;
  }
  TV_DICT_ITER(orig, di, {
    if (got_int) {
      break;
    }
    dictitem_T *new_di;
    if (conv == NULL || conv->vc_type == CONV_NONE) {
      new_di = tv_dict_item_alloc(di->di_key);
    } else {
      size_t len = strlen(di->di_key);
      char *const key = string_convert(conv, di->di_key, &len);
      if (key == NULL) {
        new_di = tv_dict_item_alloc_len(di->di_key, len);
      } else {
        new_di = tv_dict_item_alloc_len(key, len);
        xfree(key);
      }
    }
    if (deep) {
      if (var_item_copy(conv, &di->di_tv, &new_di->di_tv, deep,
                        copyID) == FAIL) {
        xfree(new_di);
        break;
      }
    } else {
      tv_copy(&di->di_tv, &new_di->di_tv);
    }
    if (tv_dict_add(copy, new_di) == FAIL) {
      tv_dict_item_free(new_di);
      break;
    }
  });

  copy->dv_refcount++;
  if (got_int) {
    tv_dict_unref(copy);
    copy = NULL;
  }

  return copy;
}

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

//{{{3 Free

/// Free allocated Vimscript object and value stored inside
///
/// @param  tv  Object to free.
void tv_free(typval_T *tv)
{
  if (tv == NULL) {
    return;
  }

  switch (tv->v_type) {
  case VAR_PARTIAL:
    partial_unref(tv->vval.v_partial);
    break;
  case VAR_FUNC:
    func_unref(tv->vval.v_string);
    FALLTHROUGH;
  case VAR_STRING:
    xfree(tv->vval.v_string);
    break;
  case VAR_BLOB:
    tv_blob_unref(tv->vval.v_blob);
    break;
  case VAR_LIST:
    tv_list_unref(tv->vval.v_list);
    break;
  case VAR_DICT:
    tv_dict_unref(tv->vval.v_dict);
    break;
  case VAR_BOOL:
  case VAR_SPECIAL:
  case VAR_NUMBER:
  case VAR_FLOAT:
  case VAR_UNKNOWN:
    break;
  }
  xfree(tv);
}

//{{{3 Copy

/// Copy typval from one location to another
///
/// When needed allocates string or increases reference count. Does not make
/// a copy of a container, but copies its reference!
///
/// It is OK for `from` and `to` to point to the same location; this is used to
/// make a copy later.
///
/// @param[in]  from  Location to copy from.
/// @param[out]  to  Location to copy to.
void tv_copy(const typval_T *const from, typval_T *const to)
{
  to->v_type = from->v_type;
  to->v_lock = VAR_UNLOCKED;
  memmove(&to->vval, &from->vval, sizeof(to->vval));
  switch (from->v_type) {
  case VAR_NUMBER:
  case VAR_FLOAT:
  case VAR_BOOL:
  case VAR_SPECIAL:
    break;
  case VAR_STRING:
  case VAR_FUNC:
    if (from->vval.v_string != NULL) {
      to->vval.v_string = xstrdup(from->vval.v_string);
      if (from->v_type == VAR_FUNC) {
        func_ref(to->vval.v_string);
      }
    }
    break;
  case VAR_PARTIAL:
    if (to->vval.v_partial != NULL) {
      to->vval.v_partial->pt_refcount++;
    }
    break;
  case VAR_BLOB:
    if (from->vval.v_blob != NULL) {
      to->vval.v_blob->bv_refcount++;
    }
    break;
  case VAR_LIST:
    tv_list_ref(to->vval.v_list);
    break;
  case VAR_DICT:
    if (from->vval.v_dict != NULL) {
      to->vval.v_dict->dv_refcount++;
    }
    break;
  case VAR_UNKNOWN:
    semsg(_(e_intern2), "tv_copy(UNKNOWN)");
    break;
  }
}

//{{{2 Locks

/// Lock or unlock an item
///
/// @param[out]  tv  Item to (un)lock.
/// @param[in]  deep  Levels to (un)lock, -1 to (un)lock everything.
/// @param[in]  lock  True if it is needed to lock an item, false to unlock.
/// @param[in]  check_refcount  If true, do not lock a list or dict with a
///                             reference count larger than 1.
void tv_item_lock(typval_T *const tv, const int deep, const bool lock, const bool check_refcount)
  FUNC_ATTR_NONNULL_ALL
{
  // TODO(ZyX-I): Make this not recursive
  static int recurse = 0;

  if (recurse >= DICT_MAXNEST) {
    emsg(_(e_variable_nested_too_deep_for_unlock));
    return;
  }
  if (deep == 0) {
    return;
  }
  recurse++;

  // lock/unlock the item itself
#define CHANGE_LOCK(lock, var) \
  do { \
    (var) = ((VarLockStatus[]) { \
      [VAR_UNLOCKED] = ((lock) ? VAR_LOCKED : VAR_UNLOCKED), \
      [VAR_LOCKED] = ((lock) ? VAR_LOCKED : VAR_UNLOCKED), \
      [VAR_FIXED] = VAR_FIXED, \
    })[var]; \
  } while (0)
  CHANGE_LOCK(lock, tv->v_lock);

  switch (tv->v_type) {
  case VAR_BLOB: {
    blob_T *const b = tv->vval.v_blob;
    if (b != NULL && !(check_refcount && b->bv_refcount > 1)) {
      CHANGE_LOCK(lock, b->bv_lock);
    }
    break;
  }
  case VAR_LIST: {
    list_T *const l = tv->vval.v_list;
    if (l != NULL && !(check_refcount && l->lv_refcount > 1)) {
      CHANGE_LOCK(lock, l->lv_lock);
      if (deep < 0 || deep > 1) {
        // Recursive: lock/unlock the items the List contains.
        TV_LIST_ITER(l, li, {
            tv_item_lock(TV_LIST_ITEM_TV(li), deep - 1, lock, check_refcount);
          });
      }
    }
    break;
  }
  case VAR_DICT: {
    dict_T *const d = tv->vval.v_dict;
    if (d != NULL && !(check_refcount && d->dv_refcount > 1)) {
      CHANGE_LOCK(lock, d->dv_lock);
      if (deep < 0 || deep > 1) {
        // recursive: lock/unlock the items the List contains
        TV_DICT_ITER(d, di, {
            tv_item_lock(&di->di_tv, deep - 1, lock, check_refcount);
          });
      }
    }
    break;
  }
  case VAR_NUMBER:
  case VAR_FLOAT:
  case VAR_STRING:
  case VAR_FUNC:
  case VAR_PARTIAL:
  case VAR_BOOL:
  case VAR_SPECIAL:
    break;
  case VAR_UNKNOWN:
    abort();
  }
#undef CHANGE_LOCK
  recurse--;
}

// tv_islocked: migrated to Rust (nvim-rs/typval)

// tv_check_lock and value_check_lock migrated to Rust (nvim-rs/typval); declared in typval.h

//{{{2 Comparison

static int tv_equal_recurse_limit;

/// Compare two Vimscript values
///
/// Like "==", but strings and numbers are different, as well as floats and
/// numbers.
///
/// @warning Too nested structures may be considered equal even if they are not.
///
/// @param[in]  tv1  First value to compare.
/// @param[in]  tv2  Second value to compare.
/// @param[in]  ic  True if case is to be ignored.
///
/// @return true if values are equal.
bool tv_equal(typval_T *const tv1, typval_T *const tv2, const bool ic)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
  // TODO(ZyX-I): Make this not recursive
  static int recursive_cnt = 0;  // Catch recursive loops.

  if (!(tv_is_func(*tv1) && tv_is_func(*tv2)) && tv1->v_type != tv2->v_type) {
    return false;
  }

  // Catch lists and dicts that have an endless loop by limiting
  // recursiveness to a limit.  We guess they are equal then.
  // A fixed limit has the problem of still taking an awful long time.
  // Reduce the limit every time running into it. That should work fine for
  // deeply linked structures that are not recursively linked and catch
  // recursiveness quickly.
  if (recursive_cnt == 0) {
    tv_equal_recurse_limit = 1000;
  }
  if (recursive_cnt >= tv_equal_recurse_limit) {
    tv_equal_recurse_limit--;
    return true;
  }

  switch (tv1->v_type) {
  case VAR_LIST: {
    recursive_cnt++;
    const bool r = tv_list_equal(tv1->vval.v_list, tv2->vval.v_list, ic);
    recursive_cnt--;
    return r;
  }
  case VAR_DICT: {
    recursive_cnt++;
    const bool r = tv_dict_equal(tv1->vval.v_dict, tv2->vval.v_dict, ic);
    recursive_cnt--;
    return r;
  }
  case VAR_PARTIAL:
  case VAR_FUNC: {
    if ((tv1->v_type == VAR_PARTIAL && tv1->vval.v_partial == NULL)
        || (tv2->v_type == VAR_PARTIAL && tv2->vval.v_partial == NULL)) {
      return false;
    }
    recursive_cnt++;
    const bool r = rs_func_equal(tv1, tv2, ic);
    recursive_cnt--;
    return r;
  }
  case VAR_BLOB:
    return tv_blob_equal(tv1->vval.v_blob, tv2->vval.v_blob);
  case VAR_NUMBER:
    return tv1->vval.v_number == tv2->vval.v_number;
  case VAR_FLOAT:
    return tv1->vval.v_float == tv2->vval.v_float;
  case VAR_STRING: {
    char buf1[NUMBUFLEN];
    char buf2[NUMBUFLEN];
    const char *s1 = tv_get_string_buf(tv1, buf1);
    const char *s2 = tv_get_string_buf(tv2, buf2);
    return mb_strcmp_ic(ic, s1, s2) == 0;
  }
  case VAR_BOOL:
    return tv1->vval.v_bool == tv2->vval.v_bool;
  case VAR_SPECIAL:
    return tv1->vval.v_special == tv2->vval.v_special;
  case VAR_UNKNOWN:
    // VAR_UNKNOWN can be the result of an invalid expression, let’s say it
    // does not equal anything, not even self.
    return false;
  }

  abort();
  return false;
}

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

/// Real implementation of value_check_lock (called from Rust and from
/// nvim_value_check_lock_translated). Does NOT call the Rust symbol.
static bool value_check_lock_impl(VarLockStatus lock, const char *name, size_t name_len)
{
  const char *error_message = NULL;
  switch (lock) {
  case VAR_UNLOCKED:
    return false;
  case VAR_LOCKED:
    error_message = N_("E741: Value is locked: %.*s");
    break;
  case VAR_FIXED:
    error_message = N_("E742: Cannot change value of %.*s");
    break;
  }
  assert(error_message != NULL);

  if (name == NULL) {
    name = _("Unknown");
    name_len = strlen(name);
  } else if (name_len == TV_TRANSLATE) {
    name = _(name);
    name_len = strlen(name);
  } else if (name_len == TV_CSTRING) {
    name_len = strlen(name);
  }

  semsg(_(error_message), (int)name_len, name);
  return true;
}

/// Call value_check_lock with TV_TRANSLATE semantics (accessor for Rust).
/// Returns true if locked (and emits error), false if unlocked.
bool nvim_value_check_lock_translated(VarLockStatus lock, const char *name)
{
  return value_check_lock_impl(lock, name, TV_TRANSLATE);
}

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

/// Call value_check_lock_impl with arbitrary name and length (accessor for Rust).
/// Returns true if locked, false otherwise.
bool nvim_value_check_lock(VarLockStatus lock, const char *name, size_t name_len)
{
  return value_check_lock_impl(lock, name, name_len);
}

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
