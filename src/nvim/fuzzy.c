// fuzzy.c: fuzzy matching algorithm and related functions
//
// Portions of this file are adapted from fzy (https://github.com/jhawthorn/fzy)
// Original code:
//   Copyright (c) 2014 John Hawthorn
//   Licensed under the MIT License.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

#include <assert.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/charset.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/fuzzy.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/globals.h"
#include "nvim/insexpand.h"
#include "nvim/macros_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"

// Note: score_t and related constants moved to Rust (src/nvim-rs/fuzzy/)

typedef struct {
  int idx;  ///< used for stable sort
  listitem_T *item;
  int score;
  list_T *lmatchpos;
  char *pat;
  char *itemstr;
  bool itemstr_allocated;
  int startpos;
} fuzzyItem_T;

_Static_assert(sizeof(pos_T) == 12, "pos_T size must match Rust PosT");
_Static_assert(sizeof(garray_T) == 24, "garray_T size must match Rust GArray");
_Static_assert(sizeof(fuzmatch_str_T) == 24, "fuzmatch_str_T size must match Rust FuzmatchStr");
_Static_assert(offsetof(fuzmatch_str_T, idx) == 0, "fuzmatch_str_T.idx offset");
_Static_assert(offsetof(fuzmatch_str_T, str) == 8, "fuzmatch_str_T.str offset");
_Static_assert(offsetof(fuzmatch_str_T, score) == 16, "fuzmatch_str_T.score offset");

#include "fuzzy.c.generated.h"

// Rust FFI declarations
// fuzzy_match, fuzzy_match_str, fuzzy_match_str_in_line, search_for_fuzzy_match,
// fuzmatch_str_free, fuzzymatches_to_strmatches exported directly from Rust.
extern garray_T *rs_fuzzy_match_str_with_pos(const char *str, const char *pat);

/// Sort the fuzzy matches in the descending order of the match score.
/// For items with same score, retain the order using the index (stable sort)
static int fuzzy_match_item_compare(const void *const s1, const void *const s2)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_PURE
{
  const int v1 = ((const fuzzyItem_T *)s1)->score;
  const int v2 = ((const fuzzyItem_T *)s2)->score;

  if (v1 == v2) {
    const char *const pat = ((const fuzzyItem_T *)s1)->pat;
    const size_t patlen = strlen(pat);
    int startpos = ((const fuzzyItem_T *)s1)->startpos;
    const bool exact_match1 = startpos >= 0
                              && strncmp(pat, ((fuzzyItem_T *)s1)->itemstr + startpos, patlen) == 0;
    startpos = ((const fuzzyItem_T *)s2)->startpos;
    const bool exact_match2 = startpos >= 0
                              && strncmp(pat, ((fuzzyItem_T *)s2)->itemstr + startpos, patlen) == 0;

    if (exact_match1 == exact_match2) {
      const int idx1 = ((const fuzzyItem_T *)s1)->idx;
      const int idx2 = ((const fuzzyItem_T *)s2)->idx;
      return idx1 == idx2 ? 0 : idx1 > idx2 ? 1 : -1;
    } else if (exact_match2) {
      return 1;
    }
    return -1;
  } else {
    return v1 > v2 ? -1 : 1;
  }
}

/// Fuzzy search the string "str" in a list of "items" and return the matching
/// strings in "fmatchlist".
/// If "matchseq" is true, then for multi-word search strings, match all the
/// words in sequence.
/// If "items" is a list of strings, then search for "str" in the list.
/// If "items" is a list of dicts, then either use "key" to lookup the string
/// for each item or use "item_cb" Funcref function to get the string.
/// If "retmatchpos" is true, then return a list of positions where "str"
/// matches for each item.
static void fuzzy_match_in_list(list_T *const l, char *const str, const bool matchseq,
                                const char *const key, Callback *const item_cb,
                                const bool retmatchpos, list_T *const fmatchlist,
                                const int max_matches)
  FUNC_ATTR_NONNULL_ARG(2, 5, 7)
{
  int len = tv_list_len(l);
  if (len == 0) {
    return;
  }
  if (max_matches > 0 && len > max_matches) {
    len = max_matches;
  }

  fuzzyItem_T *const items = xcalloc((size_t)len, sizeof(fuzzyItem_T));
  int match_count = 0;
  uint32_t matches[FUZZY_MATCH_MAX_LEN];

  // For all the string items in items, get the fuzzy matching score
  TV_LIST_ITER(l, li, {
    if (max_matches > 0 && match_count >= max_matches) {
      break;
    }

    char *itemstr = NULL;
    bool itemstr_allocate = false;
    typval_T rettv;

    rettv.v_type = VAR_UNKNOWN;
    const typval_T *const tv = TV_LIST_ITEM_TV(li);
    if (tv->v_type == VAR_STRING) {  // list of strings
      itemstr = tv->vval.v_string;
    } else if (tv->v_type == VAR_DICT
               && (key != NULL || item_cb->type != kCallbackNone)) {
      // For a dict, either use the specified key to lookup the string or
      // use the specified callback function to get the string.
      if (key != NULL) {
        itemstr = tv_dict_get_string(tv->vval.v_dict, key, false);
      } else {
        typval_T argv[2];

        // Invoke the supplied callback (if any) to get the dict item
        tv->vval.v_dict->dv_refcount++;
        argv[0].v_type = VAR_DICT;
        argv[0].vval.v_dict = tv->vval.v_dict;
        argv[1].v_type = VAR_UNKNOWN;
        if (callback_call(item_cb, 1, argv, &rettv)) {
          if (rettv.v_type == VAR_STRING) {
            itemstr = rettv.vval.v_string;
            itemstr_allocate = true;
          }
        }
        tv_dict_unref(tv->vval.v_dict);
      }
    }

    int score;
    if (itemstr != NULL
        && fuzzy_match(itemstr, str, matchseq, &score, matches, FUZZY_MATCH_MAX_LEN)) {
      char *itemstr_copy = itemstr_allocate ? xstrdup(itemstr) : itemstr;
      list_T *match_positions = NULL;

      // Copy the list of matching positions in itemstr to a list, if
      // "retmatchpos" is set.
      if (retmatchpos) {
        match_positions = tv_list_alloc(kListLenMayKnow);
        // Fill position information
        int j = 0;
        const char *p = str;
        while (*p != NUL && j < FUZZY_MATCH_MAX_LEN) {
          if (!ascii_iswhite(utf_ptr2char(p)) || matchseq) {
            tv_list_append_number(match_positions, matches[j]);
            j++;
          }
          MB_PTR_ADV(p);
        }
      }
      items[match_count].idx = match_count;
      items[match_count].item = li;
      items[match_count].score = score;
      items[match_count].pat = str;
      items[match_count].startpos = (int)matches[0];
      items[match_count].itemstr = itemstr_copy;
      items[match_count].itemstr_allocated = itemstr_allocate;
      items[match_count].lmatchpos = match_positions;

      match_count++;
    }
    tv_clear(&rettv);
  });

  if (match_count > 0) {
    // Sort the list by the descending order of the match score
    qsort(items, (size_t)match_count, sizeof(fuzzyItem_T), fuzzy_match_item_compare);

    // For matchfuzzy(), return a list of matched strings.
    //          ['str1', 'str2', 'str3']
    // For matchfuzzypos(), return a list with three items.
    // The first item is a list of matched strings. The second item
    // is a list of lists where each list item is a list of matched
    // character positions. The third item is a list of matching scores.
    //      [['str1', 'str2', 'str3'], [[1, 3], [1, 3], [1, 3]]]
    list_T *retlist;
    if (retmatchpos) {
      const listitem_T *const li = tv_list_find(fmatchlist, 0);
      assert(li != NULL && TV_LIST_ITEM_TV(li)->vval.v_list != NULL);
      retlist = TV_LIST_ITEM_TV(li)->vval.v_list;
    } else {
      retlist = fmatchlist;
    }

    // Copy the matching strings to the return list
    for (int i = 0; i < match_count; i++) {
      tv_list_append_tv(retlist, TV_LIST_ITEM_TV(items[i].item));
    }

    // next copy the list of matching positions
    if (retmatchpos) {
      const listitem_T *li = tv_list_find(fmatchlist, -2);
      assert(li != NULL && TV_LIST_ITEM_TV(li)->vval.v_list != NULL);
      retlist = TV_LIST_ITEM_TV(li)->vval.v_list;

      for (int i = 0; i < match_count; i++) {
        assert(items[i].lmatchpos != NULL);
        tv_list_append_list(retlist, items[i].lmatchpos);
        items[i].lmatchpos = NULL;
      }

      // copy the matching scores
      li = tv_list_find(fmatchlist, -1);
      assert(li != NULL && TV_LIST_ITEM_TV(li)->vval.v_list != NULL);
      retlist = TV_LIST_ITEM_TV(li)->vval.v_list;
      for (int i = 0; i < match_count; i++) {
        tv_list_append_number(retlist, items[i].score);
      }
    }
  }

  for (int i = 0; i < match_count; i++) {
    if (items[i].itemstr_allocated) {
      xfree(items[i].itemstr);
    }
    assert(items[i].lmatchpos == NULL);
  }
  xfree(items);
}

/// Do fuzzy matching. Returns the list of matched strings in "rettv".
/// If "retmatchpos" is true, also returns the matching character positions.
static void do_fuzzymatch(const typval_T *const argvars, typval_T *const rettv,
                          const bool retmatchpos)
  FUNC_ATTR_NONNULL_ALL
{
  // validate and get the arguments
  if (argvars[0].v_type != VAR_LIST || argvars[0].vval.v_list == NULL) {
    semsg(_(e_listarg), retmatchpos ? "matchfuzzypos()" : "matchfuzzy()");
    return;
  }
  if (argvars[1].v_type != VAR_STRING || argvars[1].vval.v_string == NULL) {
    semsg(_(e_invarg2), tv_get_string(&argvars[1]));
    return;
  }

  Callback cb = CALLBACK_NONE;
  const char *key = NULL;
  bool matchseq = false;
  int max_matches = 0;
  if (argvars[2].v_type != VAR_UNKNOWN) {
    if (tv_check_for_nonnull_dict_arg(argvars, 2) == FAIL) {
      return;
    }

    // To search a dict, either a callback function or a key can be
    // specified.
    dict_T *const d = argvars[2].vval.v_dict;
    const dictitem_T *di;
    if ((di = tv_dict_find(d, "key", -1)) != NULL) {
      if (di->di_tv.v_type != VAR_STRING || di->di_tv.vval.v_string == NULL
          || *di->di_tv.vval.v_string == NUL) {
        semsg(_(e_invargNval), "key", tv_get_string(&di->di_tv));
        return;
      }
      key = tv_get_string(&di->di_tv);
    } else if (!tv_dict_get_callback(d, "text_cb", -1, &cb)) {
      semsg(_(e_invargval), "text_cb");
      return;
    }

    if ((di = tv_dict_find(d, "limit", -1)) != NULL) {
      if (di->di_tv.v_type != VAR_NUMBER) {
        semsg(_(e_invargval), "limit");
        return;
      }
      max_matches = (int)tv_get_number_chk(&di->di_tv, NULL);
    }

    if (tv_dict_has_key(d, "matchseq")) {
      matchseq = true;
    }
  }

  // get the fuzzy matches
  tv_list_alloc_ret(rettv, retmatchpos ? 3 : kListLenUnknown);
  if (retmatchpos) {
    // For matchfuzzypos(), a list with three items are returned. First
    // item is a list of matching strings, the second item is a list of
    // lists with matching positions within each string and the third item
    // is the list of scores of the matches.
    tv_list_append_list(rettv->vval.v_list, tv_list_alloc(kListLenUnknown));
    tv_list_append_list(rettv->vval.v_list, tv_list_alloc(kListLenUnknown));
    tv_list_append_list(rettv->vval.v_list, tv_list_alloc(kListLenUnknown));
  }

  fuzzy_match_in_list(argvars[0].vval.v_list, (char *)tv_get_string(&argvars[1]),
                      matchseq, key, &cb, retmatchpos, rettv->vval.v_list, max_matches);

  callback_free(&cb);
}

/// "matchfuzzy()" function
void f_matchfuzzy(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  do_fuzzymatch(argvars, rettv, false);
}

/// "matchfuzzypos()" function
void f_matchfuzzypos(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  do_fuzzymatch(argvars, rettv, true);
}

/// Fuzzy match the position of string "pat" in string "str".
/// @returns a dynamic array of matching positions. If there is no match, returns NULL.
garray_T *fuzzy_match_str_with_pos(char *const str, const char *const pat)
{
  return rs_fuzzy_match_str_with_pos(str, pat);
}

// Note: The fuzzy matching algorithm implementation has been moved to Rust.
// See src/nvim-rs/fuzzy/ for the implementation.
// The algorithm is ported from https://github.com/jhawthorn/fzy.
