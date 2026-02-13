// cmdhist.c: Functions for the history of the command-line.

#include <assert.h>
#include <limits.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/cmdhist.h"
#include "nvim/errors.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_getln.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/macros_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/option_vars.h"
#include "nvim/os/time.h"
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"

#include "cmdhist.c.generated.h"

extern HistoryType rs_hist_char2type(int c);
extern int rs_get_hislen(void);
extern HistoryType rs_get_histtype(const char *name, size_t len, int return_default);

_Static_assert(sizeof(histentry_T) == 40,
               "sizeof(histentry_T) changed - update Rust HistoryEntry in cmdline/src/history.rs");
_Static_assert(HIST_COUNT == 5, "HIST_COUNT changed - update Rust HIST_COUNT");
_Static_assert(CMOD_KEEPPATTERNS == 0x1000, "CMOD_KEEPPATTERNS changed - update Rust constant");
_Static_assert(RE_MAGIC == 1, "RE_MAGIC changed - update Rust constant");
_Static_assert(RE_STRING == 2, "RE_STRING changed - update Rust constant");

static histentry_T *(history[HIST_COUNT]) = { NULL, NULL, NULL, NULL, NULL };
static int hisidx[HIST_COUNT] = { -1, -1, -1, -1, -1 };  ///< lastused entry
/// identifying (unique) number of newest history entry
static int hisnum[HIST_COUNT] = { 0, 0, 0, 0, 0 };
static int hislen = 0;  ///< actual length of history tables

/// C accessor for hislen static variable.
int nvim_get_hislen(void)
{
  return hislen;
}

/// Return the length of the history tables
int get_hislen(void)
{
  return rs_get_hislen();
}

/// Return a pointer to a specified history table
histentry_T *get_histentry(int hist_type)
{
  return history[hist_type];
}

void set_histentry(int hist_type, histentry_T *entry)
{
  history[hist_type] = entry;
}

int *get_hisidx(int hist_type)
{
  return &hisidx[hist_type];
}

int *get_hisnum(int hist_type)
{
  return &hisnum[hist_type];
}

// =============================================================================
// histentry_T field accessors for Rust
// =============================================================================

int nvim_cmdhist_he_get_hisnum(histentry_T *he)
{
  return he->hisnum;
}

void nvim_cmdhist_he_set_hisnum(histentry_T *he, int val)
{
  he->hisnum = val;
}

char *nvim_cmdhist_he_get_hisstr(histentry_T *he)
{
  return he->hisstr;
}

void nvim_cmdhist_he_set_hisstr(histentry_T *he, char *val)
{
  he->hisstr = val;
}

size_t nvim_cmdhist_he_get_hisstrlen(histentry_T *he)
{
  return he->hisstrlen;
}

void nvim_cmdhist_he_set_hisstrlen(histentry_T *he, size_t val)
{
  he->hisstrlen = val;
}

uint64_t nvim_cmdhist_he_get_timestamp(histentry_T *he)
{
  return he->timestamp;
}

void nvim_cmdhist_he_set_timestamp(histentry_T *he, uint64_t val)
{
  he->timestamp = val;
}

void *nvim_cmdhist_he_get_additional_data(histentry_T *he)
{
  return he->additional_data;
}

void nvim_cmdhist_he_set_additional_data(histentry_T *he, void *val)
{
  he->additional_data = (AdditionalData *)val;
}

void nvim_cmdhist_he_clear(histentry_T *he)
{
  CLEAR_POINTER(he);
}

void nvim_cmdhist_he_copy(histentry_T *dst, histentry_T *src)
{
  *dst = *src;
}

histentry_T *nvim_cmdhist_he_at(histentry_T *base, int idx)
{
  return &base[idx];
}

// -- Memory wrappers --

void nvim_cmdhist_xfree(void *ptr)
{
  xfree(ptr);
}

void *nvim_cmdhist_xmalloc(size_t size)
{
  return xmalloc(size);
}

char *nvim_cmdhist_xstrnsave(const char *s, size_t len)
{
  return xstrnsave(s, len);
}

// -- String wrappers --

int nvim_cmdhist_strnicmp(const char *s1, const char *s2, size_t n)
{
  return STRNICMP(s1, s2, n);
}

char *nvim_cmdhist_vim_strchr(const char *s, int c)
{
  return vim_strchr(s, c);
}

// -- Global accessors --

int nvim_cmdhist_get_cmdline_firstc(void)
{
  return get_cmdline_firstc();
}

// -- Array ops --

void nvim_cmdhist_memset_entries(histentry_T *dst, int count)
{
  memset(dst, 0, (size_t)count * sizeof(histentry_T));
}

void nvim_cmdhist_memcpy_entries(histentry_T *dst, histentry_T *src, int count)
{
  memcpy(dst, src, (size_t)count * sizeof(histentry_T));
}

// -- Sizeof --

size_t nvim_cmdhist_sizeof_histentry(void)
{
  return sizeof(histentry_T);
}

// =============================================================================
// Phase 2: History Modification Accessors
// =============================================================================

int64_t nvim_cmdhist_get_p_hi(void)
{
  return p_hi;
}

int nvim_cmdhist_get_maptick(void)
{
  return maptick;
}

uint64_t nvim_cmdhist_os_time(void)
{
  return os_time();
}

int nvim_cmdhist_get_cmdmod_cmod_flags(void)
{
  return (int)cmdmod.cmod_flags;
}

void nvim_cmdhist_set_hislen(int val)
{
  hislen = val;
}

int nvim_cmdhist_strcmp(const char *s1, const char *s2)
{
  return strcmp(s1, s2);
}

extern void rs_init_history(void);
extern void rs_add_to_history(int histype, const char *new_entry, size_t new_entrylen, int in_map,
                              int sep);
extern int rs_clr_history(int histype);
extern int nvim_cmdhist_get_last_maptick(void);
extern void nvim_cmdhist_set_last_maptick(int val);

// =============================================================================
// Phase 3: Regexp Wrappers
// =============================================================================

void *nvim_cmdhist_regcomp(const char *str, int flags)
{
  regmatch_T *rm = xmalloc(sizeof(regmatch_T));
  rm->regprog = vim_regcomp((char *)str, flags);
  if (rm->regprog == NULL) {
    xfree(rm);
    return NULL;
  }
  rm->rm_ic = false;  // always match case
  return rm;
}

int nvim_cmdhist_regexec(void *rm, const char *str)
{
  return vim_regexec((regmatch_T *)rm, (char *)str, 0);
}

void nvim_cmdhist_regfree(void *rm)
{
  regmatch_T *r = (regmatch_T *)rm;
  vim_regfree(r->regprog);
  xfree(r);
}

extern int rs_del_history_entry(int histype, const char *str);
extern int rs_del_history_idx(int histype, int idx);

/// Translate a history character to the associated type number
HistoryType hist_char2type(const int c)
  FUNC_ATTR_CONST FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_hist_char2type(c);
}

/// Table of history names.
/// These names are used in :history and various hist...() functions.
/// It is sufficient to give the significant prefix of a history name.
static char *(history_names[]) = {
  "cmd",
  "search",
  "expr",
  "input",
  "debug",
  NULL
};

/// Function given to ExpandGeneric() to obtain the possible first
/// arguments of the ":history command.
char *get_history_arg(expand_T *xp, int idx)
{
  const char *short_names = ":=@>?/";
  const int short_names_count = (int)strlen(short_names);
  const int history_name_count = ARRAY_SIZE(history_names) - 1;

  if (idx < short_names_count) {
    xp->xp_buf[0] = short_names[idx];
    xp->xp_buf[1] = NUL;
    return xp->xp_buf;
  }
  if (idx < short_names_count + history_name_count) {
    return history_names[idx - short_names_count];
  }
  if (idx == short_names_count + history_name_count) {
    return "all";
  }
  return NULL;
}

/// Initialize command line history.
void init_history(void)
{
  rs_init_history();
}

/// Convert history name to its HIST_ equivalent
static HistoryType get_histtype(const char *const name, const size_t len, const bool return_default)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_get_histtype(name, len, return_default);
}

/// Add the given string to the given history.
void add_to_history(int histype, const char *new_entry, size_t new_entrylen, bool in_map, int sep)
{
  rs_add_to_history(histype, new_entry, new_entrylen, in_map, sep);
}

/// Get identifier of newest history entry.
///
/// @param histype  may be one of the HIST_ values.
static int get_history_idx(int histype)
{
  if (hislen == 0 || histype < 0 || histype >= HIST_COUNT
      || hisidx[histype] < 0) {
    return -1;
  }

  return history[histype][hisidx[histype]].hisnum;
}

/// Calculate history index from a number:
///
/// @param num      > 0: seen as identifying number of a history entry
///                 < 0: relative position in history wrt newest entry
/// @param histype  may be one of the HIST_ values.
static int calc_hist_idx(int histype, int num)
{
  int i;

  if (hislen == 0 || histype < 0 || histype >= HIST_COUNT
      || (i = hisidx[histype]) < 0 || num == 0) {
    return -1;
  }

  histentry_T *hist = history[histype];
  if (num > 0) {
    bool wrapped = false;
    while (hist[i].hisnum > num) {
      if (--i < 0) {
        if (wrapped) {
          break;
        }
        i += hislen;
        wrapped = true;
      }
    }
    if (i >= 0 && hist[i].hisnum == num && hist[i].hisstr != NULL) {
      return i;
    }
  } else if (-num <= hislen) {
    i += num + 1;
    if (i < 0) {
      i += hislen;
    }
    if (hist[i].hisstr != NULL) {
      return i;
    }
  }
  return -1;
}

/// Clear all entries in a history
int clr_history(const int histype)
{
  return rs_clr_history(histype);
}

/// Remove all entries matching {str} from a history.
static int del_history_entry(int histype, char *str)
{
  return rs_del_history_entry(histype, str);
}

/// Remove an indexed entry from a history.
static int del_history_idx(int histype, int idx)
{
  return rs_del_history_idx(histype, idx);
}

/// "histadd()" function
void f_histadd(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rettv->vval.v_number = false;
  if (check_secure()) {
    return;
  }
  const char *str = tv_get_string_chk(&argvars[0]);  // NULL on type error
  HistoryType histype = str != NULL ? get_histtype(str, strlen(str), false) : HIST_INVALID;
  if (histype == HIST_INVALID) {
    return;
  }

  char buf[NUMBUFLEN];
  str = tv_get_string_buf(&argvars[1], buf);
  if (*str == NUL) {
    return;
  }

  init_history();
  add_to_history(histype, str, strlen(str), false, NUL);
  rettv->vval.v_number = true;
}

/// "histdel()" function
void f_histdel(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  int n;
  const char *const str = tv_get_string_chk(&argvars[0]);  // NULL on type error
  if (str == NULL) {
    n = 0;
  } else if (argvars[1].v_type == VAR_UNKNOWN) {
    // only one argument: clear entire history
    n = clr_history(get_histtype(str, strlen(str), false));
  } else if (argvars[1].v_type == VAR_NUMBER) {
    // index given: remove that entry
    n = del_history_idx(get_histtype(str, strlen(str), false),
                        (int)tv_get_number(&argvars[1]));
  } else {
    // string given: remove all matching entries
    char buf[NUMBUFLEN];
    n = del_history_entry(get_histtype(str, strlen(str), false),
                          (char *)tv_get_string_buf(&argvars[1], buf));
  }
  rettv->vval.v_number = n;
}

/// "histget()" function
void f_histget(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  const char *const str = tv_get_string_chk(&argvars[0]);  // NULL on type error
  if (str == NULL) {
    rettv->vval.v_string = NULL;
  } else {
    int idx;
    HistoryType type = get_histtype(str, strlen(str), false);
    if (argvars[1].v_type == VAR_UNKNOWN) {
      idx = get_history_idx(type);
    } else {
      idx = (int)tv_get_number_chk(&argvars[1], NULL);  // -1 on type error
    }
    idx = calc_hist_idx(type, idx);
    if (idx < 0) {
      rettv->vval.v_string = xstrnsave("", 0);
    } else {
      rettv->vval.v_string = xstrnsave(history[type][idx].hisstr,
                                       history[type][idx].hisstrlen);
    }
  }
  rettv->v_type = VAR_STRING;
}

/// "histnr()" function
void f_histnr(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  const char *const histname = tv_get_string_chk(&argvars[0]);
  HistoryType i = histname == NULL
                  ? HIST_INVALID
                  : get_histtype(histname, strlen(histname), false);
  if (i != HIST_INVALID) {
    rettv->vval.v_number = get_history_idx(i);
  } else {
    rettv->vval.v_number = HIST_INVALID;
  }
}

/// :history command - print a history
void ex_history(exarg_T *eap)
{
  int histype1 = HIST_CMD;
  int histype2 = HIST_CMD;
  int hisidx1 = 1;
  int hisidx2 = -1;
  char *end;
  char *arg = eap->arg;

  if (hislen == 0) {
    msg(_("'history' option is zero"), 0);
    return;
  }

  if (!(ascii_isdigit(*arg) || *arg == '-' || *arg == ',')) {
    end = arg;
    while (ASCII_ISALPHA(*end)
           || vim_strchr(":=@>/?", (uint8_t)(*end)) != NULL) {
      end++;
    }
    histype1 = get_histtype(arg, (size_t)(end - arg), false);
    if (histype1 == HIST_INVALID) {
      if (STRNICMP(arg, "all", end - arg) == 0) {
        histype1 = 0;
        histype2 = HIST_COUNT - 1;
      } else {
        semsg(_(e_trailing_arg), arg);
        return;
      }
    } else {
      histype2 = histype1;
    }
  } else {
    end = arg;
  }
  if (!get_list_range(&end, &hisidx1, &hisidx2) || *end != NUL) {
    if (*end != NUL) {
      semsg(_(e_trailing_arg), end);
    } else {
      semsg(_(e_val_too_large), arg);
    }
    return;
  }

  for (; !got_int && histype1 <= histype2; histype1++) {
    assert(history_names[histype1] != NULL);
    vim_snprintf(IObuff, IOSIZE, "\n      #  %s history", history_names[histype1]);
    msg_puts_title(IObuff);
    int idx = hisidx[histype1];
    histentry_T *hist = history[histype1];
    int j = hisidx1;
    int k = hisidx2;
    if (j < 0) {
      j = (-j > hislen) ? 0 : hist[(hislen + j + idx + 1) % hislen].hisnum;
    }
    if (k < 0) {
      k = (-k > hislen) ? 0 : hist[(hislen + k + idx + 1) % hislen].hisnum;
    }
    if (idx >= 0 && j <= k) {
      for (int i = idx + 1; !got_int; i++) {
        if (i == hislen) {
          i = 0;
        }
        if (hist[i].hisstr != NULL
            && hist[i].hisnum >= j && hist[i].hisnum <= k
            && !message_filtered(hist[i].hisstr)) {
          msg_putchar('\n');
          int len = snprintf(IObuff, IOSIZE,
                             "%c%6d  ", i == idx ? '>' : ' ', hist[i].hisnum);
          if (vim_strsize(hist[i].hisstr) > Columns - 10) {
            trunc_string(hist[i].hisstr, IObuff + len, Columns - 10, IOSIZE - len);
          } else {
            xstrlcpy(IObuff + len, hist[i].hisstr, (size_t)(IOSIZE - len));
          }
          msg_outtrans(IObuff, 0, false);
        }
        if (i == idx) {
          break;
        }
      }
    }
  }
}

/// Iterate over history items
///
/// @warning No history-editing functions must be run while iteration is in
///          progress.
///
/// @param[in]   iter          Pointer to the last history entry.
/// @param[in]   history_type  Type of the history (HIST_*). Ignored if iter
///                            parameter is not NULL.
/// @param[in]   zero          If true then zero (but not free) returned items.
///
///                            @warning When using this parameter user is
///                                     responsible for calling clr_history()
///                                     itself after iteration is over. If
///                                     clr_history() is not called behaviour is
///                                     undefined. No functions that work with
///                                     history must be called during iteration
///                                     in this case.
/// @param[out]  hist          Next history entry.
///
/// @return Pointer used in next iteration or NULL to indicate that iteration
///         was finished.
const void *hist_iter(const void *const iter, const uint8_t history_type, const bool zero,
                      histentry_T *const hist)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ARG(4)
{
  *hist = (histentry_T) {
    .hisstr = NULL
  };
  if (hisidx[history_type] == -1) {
    return NULL;
  }
  histentry_T *const hstart = &(history[history_type][0]);
  histentry_T *const hlast = &(history[history_type][hisidx[history_type]]);
  const histentry_T *const hend = &(history[history_type][hislen - 1]);
  histentry_T *hiter;
  if (iter == NULL) {
    histentry_T *hfirst = hlast;
    do {
      hfirst++;
      if (hfirst > hend) {
        hfirst = hstart;
      }
      if (hfirst->hisstr != NULL) {
        break;
      }
    } while (hfirst != hlast);
    hiter = hfirst;
  } else {
    hiter = (histentry_T *)iter;
  }
  if (hiter == NULL) {
    return NULL;
  }
  *hist = *hiter;
  if (zero) {
    CLEAR_POINTER(hiter);
  }
  if (hiter == hlast) {
    return NULL;
  }
  hiter++;
  return (const void *)((hiter > hend) ? hstart : hiter);
}

/// Get array of history items
///
/// @param[in]   history_type  Type of the history to get array for.
/// @param[out]  new_hisidx    Location where last index in the new array should
///                            be saved.
/// @param[out]  new_hisnum    Location where last history number in the new
///                            history should be saved.
///
/// @return Pointer to the array or NULL.
histentry_T *hist_get_array(const uint8_t history_type, int **const new_hisidx,
                            int **const new_hisnum)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
  init_history();
  *new_hisidx = &(hisidx[history_type]);
  *new_hisnum = &(hisnum[history_type]);
  return history[history_type];
}
