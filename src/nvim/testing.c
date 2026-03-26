// testing.c: Support for tests

#include <inttypes.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/encode.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_docmd.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/hashtab.h"
#include "nvim/hashtab_defs.h"
#include "nvim/macros_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/os/fs.h"
#include "nvim/runtime.h"
#include "nvim/runtime_defs.h"
#include "nvim/strings.h"
#include "nvim/testing.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"

#include "testing.c.generated.h"

// Rust code hard-codes sizeof(typval_T) for pointer arithmetic on argvar arrays.
// If this changes, update TYPVAL_SIZE in:
//   src/nvim-rs/testing/src/viml_assert.rs
//   src/nvim-rs/eval/src/funcs/dispatch.rs
_Static_assert(sizeof(typval_T) == 16,
               "sizeof(typval_T) changed - update Rust TYPVAL_SIZE");

// Constants used by Rust code in viml_assert.rs
_Static_assert(VAR_LIST == 4, "VAR_LIST changed - update Rust");
_Static_assert(VV_ERRMSG == 3, "VV_ERRMSG changed - update Rust");
_Static_assert(FAIL == 0, "FAIL changed - update Rust");

// C accessor functions for Rust

/// Get the current sourcing line number for error messages.
linenr_T nvim_testing_get_sourcing_lnum(void)
{
  if (HAVE_SOURCING_INFO) {
    return SOURCING_LNUM;
  }
  return 0;
}

/// Set the return value to a number.
void nvim_testing_set_rettv_number(typval_T *rettv, varnumber_T val)
{
  rettv->v_type = VAR_NUMBER;
  rettv->vval.v_number = val;
}

/// Get the v_type field of a typval.
int nvim_testing_tv_get_type(const typval_T *tv)
{
  if (tv == NULL) {
    return VAR_UNKNOWN;
  }
  return (int)tv->v_type;
}

/// Check if a string typval is empty (NULL or empty string).
int nvim_testing_tv_string_is_empty(const typval_T *tv)
{
  if (tv == NULL || tv->v_type != VAR_STRING) {
    return 0;
  }
  return tv->vval.v_string == NULL || *tv->vval.v_string == NUL;
}

/// Get the bool value from a typval (for VAR_BOOL type).
int nvim_testing_tv_get_bool_value(const typval_T *tv)
{
  if (tv == NULL || tv->v_type != VAR_BOOL) {
    return -1;
  }
  return (int)tv->vval.v_bool;
}

/// Check if a typval is a float type.
int nvim_testing_tv_is_float(const typval_T *tv) { return tv != NULL && tv->v_type == VAR_FLOAT; }

// C accessor functions for f_assert_fails (Rust)

/// Get the list length from a typval that contains a list.
int nvim_testing_tv_list_len(const typval_T *tv)
{
  if (tv == NULL || tv->v_type != VAR_LIST || tv->vval.v_list == NULL) {
    return 0;
  }
  return tv_list_len(tv->vval.v_list);
}

/// Get the first listitem from a typval that contains a list.
listitem_T *nvim_testing_tv_list_first(const typval_T *tv)
{
  if (tv == NULL || tv->v_type != VAR_LIST || tv->vval.v_list == NULL) {
    return NULL;
  }
  return tv_list_first(tv->vval.v_list);
}

/// Get the last listitem from a typval that contains a list.
listitem_T *nvim_testing_tv_list_last(const typval_T *tv)
{
  if (tv == NULL || tv->v_type != VAR_LIST || tv->vval.v_list == NULL) {
    return NULL;
  }
  return tv_list_last(tv->vval.v_list);
}

/// Create a temporary NUMBER typval on the stack and return a pointer to it.
/// WARNING: The returned pointer is only valid until the next call to this function.
static typval_T _assert_fails_tmp_tv;
typval_T *nvim_testing_make_number_tv(long val)
{
  _assert_fails_tmp_tv.v_type = VAR_NUMBER;
  _assert_fails_tmp_tv.vval.v_number = val;
  return &_assert_fails_tmp_tv;
}

/// Create a temporary STRING typval on the stack and return a pointer to it.
/// WARNING: The returned pointer is only valid until the next call to this function.
typval_T *nvim_testing_make_string_tv(char *val)
{
  _assert_fails_tmp_tv.v_type = VAR_STRING;
  _assert_fails_tmp_tv.vval.v_string = val;
  return &_assert_fails_tmp_tv;
}

/// Get vval.v_number from a typval.
varnumber_T nvim_testing_tv_get_number(const typval_T *tv)
{
  if (tv == NULL) {
    return 0;
  }
  return tv->vval.v_number;
}

/// Get vval.v_string from a typval (borrowed).
const char *nvim_testing_tv_get_vstring(const typval_T *tv)
{
  if (tv == NULL || tv->v_type != VAR_STRING) {
    return NULL;
  }
  return tv->vval.v_string;
}

/// Fill the gap with dict diff info (keep complex diffing logic in C for now).
/// This handles the dictionary comparison and writes the encoded expected value.
void nvim_testing_fill_dict_diff(garray_T *gap, typval_T *exp_tv, typval_T *got_tv, int *omitted)
{
  *omitted = 0;

  // Only diff if both are non-NULL dicts
  if (exp_tv->v_type != VAR_DICT || got_tv->v_type != VAR_DICT
      || exp_tv->vval.v_dict == NULL || got_tv->vval.v_dict == NULL) {
    // Just encode the expected value
    char *tofree = encode_tv2string(exp_tv, NULL);
    ga_concat_shorten_esc(gap, tofree);
    xfree(tofree);
    return;
  }

  dict_T *exp_d = exp_tv->vval.v_dict;
  dict_T *got_d = got_tv->vval.v_dict;

  // Create temporary dicts to hold only differing items
  typval_T exp_diff = { .v_type = VAR_DICT, .vval.v_dict = tv_dict_alloc() };
  typval_T got_diff = { .v_type = VAR_DICT, .vval.v_dict = tv_dict_alloc() };

  // Find items in exp_d that differ from got_d
  int todo = (int)exp_d->dv_hashtab.ht_used;
  for (const hashitem_T *hi = exp_d->dv_hashtab.ht_array; todo > 0; hi++) {
    if (!HASHITEM_EMPTY(hi)) {
      dictitem_T *item2 = tv_dict_find(got_d, hi->hi_key, -1);
      if (item2 == NULL
          || !tv_equal(&TV_DICT_HI2DI(hi)->di_tv, &item2->di_tv, false)) {
        const size_t key_len = strlen(hi->hi_key);
        tv_dict_add_tv(exp_diff.vval.v_dict, hi->hi_key, key_len, &TV_DICT_HI2DI(hi)->di_tv);
        if (item2 != NULL) {
          tv_dict_add_tv(got_diff.vval.v_dict, hi->hi_key, key_len, &item2->di_tv);
        }
      } else {
        (*omitted)++;
      }
      todo--;
    }
  }

  // Find items only in got_d
  todo = (int)got_d->dv_hashtab.ht_used;
  for (const hashitem_T *hi = got_d->dv_hashtab.ht_array; todo > 0; hi++) {
    if (!HASHITEM_EMPTY(hi)) {
      dictitem_T *item2 = tv_dict_find(exp_d, hi->hi_key, -1);
      if (item2 == NULL) {
        const size_t key_len = strlen(hi->hi_key);
        tv_dict_add_tv(got_diff.vval.v_dict, hi->hi_key, key_len, &TV_DICT_HI2DI(hi)->di_tv);
      }
      todo--;
    }
  }

  // Encode and append the diff
  char *tofree = encode_tv2string(&exp_diff, NULL);
  ga_concat_shorten_esc(gap, tofree);
  xfree(tofree);

  // Store the got_diff for later use (will be encoded in Rust)
  // For now, we'll encode it here and let Rust handle the rest
  tv_clear(&exp_diff);

  // We need to pass got_diff back to Rust somehow
  // Actually, let's simplify: we'll encode got_diff here and store it in got_tv temporarily
  // This is a bit hacky but avoids major restructuring
  tv_clear(got_tv);
  *got_tv = got_diff;
}


/// Append "p[clen]" to "gap", escaping unprintable characters.
/// Changes NL to \n, CR to \r, etc.
static void ga_concat_esc(garray_T *gap, const char *p, int clen)
  FUNC_ATTR_NONNULL_ALL
{
  char buf[NUMBUFLEN];

  if (clen > 1) {
    memmove(buf, p, (size_t)clen);
    buf[clen] = NUL;
    ga_concat(gap, buf);
    return;
  }

  switch (*p) {
  case BS:
    ga_concat(gap, "\\b"); break;
  case ESC:
    ga_concat(gap, "\\e"); break;
  case FF:
    ga_concat(gap, "\\f"); break;
  case NL:
    ga_concat(gap, "\\n"); break;
  case TAB:
    ga_concat(gap, "\\t"); break;
  case CAR:
    ga_concat(gap, "\\r"); break;
  case '\\':
    ga_concat(gap, "\\\\"); break;
  default:
    if ((uint8_t)(*p) < ' ' || *p == 0x7f) {
      vim_snprintf(buf, NUMBUFLEN, "\\x%02x", *p);
      ga_concat(gap, buf);
    } else {
      ga_append(gap, (uint8_t)(*p));
    }
    break;
  }
}

/// Append "str" to "gap", escaping unprintable characters.
/// Changes NL to \n, CR to \r, etc.
static void ga_concat_shorten_esc(garray_T *gap, const char *str)
  FUNC_ATTR_NONNULL_ARG(1)
{
  char buf[NUMBUFLEN];

  if (str == NULL) {
    ga_concat(gap, "NULL");
    return;
  }

  for (const char *p = str; *p != NUL;) {
    int same_len = 1;
    const char *s = p;
    const int c = mb_cptr2char_adv(&s);
    const int clen = (int)(s - p);
    while (*s != NUL && c == utf_ptr2char(s)) {
      same_len++;
      s += clen;
    }
    if (same_len > 20) {
      ga_concat(gap, "\\[");
      ga_concat_esc(gap, p, clen);
      ga_concat(gap, " occurs ");
      vim_snprintf(buf, NUMBUFLEN, "%d", same_len);
      ga_concat(gap, buf);
      ga_concat(gap, " times]");
      p = s;
    } else {
      ga_concat_esc(gap, p, clen);
      p += clen;
    }
  }
}

