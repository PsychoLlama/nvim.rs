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


_Static_assert(sizeof(histentry_T) == 40,
               "sizeof(histentry_T) changed - update Rust HistoryEntry in cmdline/src/history.rs");
_Static_assert(HIST_COUNT == 5, "HIST_COUNT changed - update Rust HIST_COUNT");
_Static_assert(CMOD_KEEPPATTERNS == 0x1000, "CMOD_KEEPPATTERNS changed - update Rust constant");
_Static_assert(RE_MAGIC == 1, "RE_MAGIC changed - update Rust constant");
_Static_assert(RE_STRING == 2, "RE_STRING changed - update Rust constant");

// history[], hisidx[], hisnum[], hislen, nvim_get_hislen(), get_histentry(),
// set_histentry(), get_hisidx(), get_hisnum() are now Rust statics/functions
// in src/nvim-rs/cmdhist/src/state.rs.

// histentry_T field accessors for Rust

int nvim_cmdhist_he_get_hisnum(histentry_T *he) { return he->hisnum; }

void nvim_cmdhist_he_set_hisnum(histentry_T *he, int val) { he->hisnum = val; }

char *nvim_cmdhist_he_get_hisstr(histentry_T *he) { return he->hisstr; }

void nvim_cmdhist_he_set_hisstr(histentry_T *he, char *val) { he->hisstr = val; }

size_t nvim_cmdhist_he_get_hisstrlen(histentry_T *he) { return he->hisstrlen; }

void nvim_cmdhist_he_set_hisstrlen(histentry_T *he, size_t val) { he->hisstrlen = val; }

uint64_t nvim_cmdhist_he_get_timestamp(histentry_T *he) { return he->timestamp; }

void nvim_cmdhist_he_set_timestamp(histentry_T *he, uint64_t val) { he->timestamp = val; }

void *nvim_cmdhist_he_get_additional_data(histentry_T *he) { return he->additional_data; }

void nvim_cmdhist_he_set_additional_data(histentry_T *he, void *val) { he->additional_data = (AdditionalData *)val; }

void nvim_cmdhist_he_clear(histentry_T *he) { CLEAR_POINTER(he); }

void nvim_cmdhist_he_copy(histentry_T *dst, histentry_T *src) { *dst = *src; }

histentry_T *nvim_cmdhist_he_at(histentry_T *base, int idx) { return &base[idx]; }

// -- Memory wrappers --

void nvim_cmdhist_xfree(void *ptr) { xfree(ptr); }

void *nvim_cmdhist_xmalloc(size_t size) { return xmalloc(size); }

char *nvim_cmdhist_xstrnsave(const char *s, size_t len) { return xstrnsave(s, len); }

// -- String wrappers --

int nvim_cmdhist_strnicmp(const char *s1, const char *s2, size_t n) { return STRNICMP(s1, s2, n); }

char *nvim_cmdhist_vim_strchr(const char *s, int c) { return vim_strchr(s, c); }

// -- Global accessors --

int nvim_cmdhist_get_cmdline_firstc(void) { return get_cmdline_firstc(); }

// -- Array ops --

void nvim_cmdhist_memset_entries(histentry_T *dst, int count) { memset(dst, 0, (size_t)count * sizeof(histentry_T)); }

void nvim_cmdhist_memcpy_entries(histentry_T *dst, histentry_T *src, int count)
{
  memcpy(dst, src, (size_t)count * sizeof(histentry_T));
}

// -- Sizeof --

size_t nvim_cmdhist_sizeof_histentry(void) { return sizeof(histentry_T); }

// Phase 2: History Modification Accessors

int64_t nvim_cmdhist_get_p_hi(void) { return p_hi; }

int nvim_cmdhist_get_maptick(void) { return maptick; }

uint64_t nvim_cmdhist_os_time(void) { return os_time(); }

int nvim_cmdhist_get_cmdmod_cmod_flags(void) { return (int)cmdmod.cmod_flags; }

// nvim_cmdhist_set_hislen() is now in src/nvim-rs/cmdhist/src/state.rs.

int nvim_cmdhist_strcmp(const char *s1, const char *s2) { return strcmp(s1, s2); }

extern int nvim_cmdhist_get_last_maptick(void);
extern void nvim_cmdhist_set_last_maptick(int val);

// Phase 3: Regexp Wrappers

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

int nvim_cmdhist_regexec(void *rm, const char *str) { return vim_regexec((regmatch_T *)rm, (char *)str, 0); }

void nvim_cmdhist_regfree(void *rm)
{
  regmatch_T *r = (regmatch_T *)rm;
  vim_regfree(r->regprog);
  xfree(r);
}


// Phase 4: VimL Typval Wrappers

_Static_assert(VAR_UNKNOWN == 0, "VAR_UNKNOWN changed - update Rust constant");
_Static_assert(VAR_NUMBER == 1, "VAR_NUMBER changed - update Rust constant");
_Static_assert(VAR_STRING == 2, "VAR_STRING changed - update Rust constant");
_Static_assert(NUMBUFLEN == 65, "NUMBUFLEN changed - update Rust constant");

const char *nvim_cmdhist_tv_get_string_chk(typval_T *tv) { return tv_get_string_chk(tv); }

const char *nvim_cmdhist_tv_get_string_buf(typval_T *tv, char *buf) { return tv_get_string_buf(tv, buf); }

int64_t nvim_cmdhist_tv_get_number(typval_T *tv) { return tv_get_number(tv); }

int64_t nvim_cmdhist_tv_get_number_chk(typval_T *tv, void *error) { return tv_get_number_chk(tv, (bool *)error); }

int nvim_cmdhist_tv_get_type(typval_T *tv) { return tv->v_type; }

typval_T *nvim_cmdhist_tv_idx(typval_T *tv, int idx) { return &tv[idx]; }

void nvim_cmdhist_rettv_set_number(typval_T *rettv, int64_t val) { rettv->vval.v_number = val; }

void nvim_cmdhist_rettv_set_string(typval_T *rettv, char *s) { rettv->vval.v_string = s; }

void nvim_cmdhist_rettv_set_type(typval_T *rettv, int typ) { rettv->v_type = typ; }


size_t nvim_cmdhist_strlen(const char *s) { return strlen(s); }

// Phase 5: Ex Command Accessors

_Static_assert(IOSIZE == 1025, "IOSIZE changed - update Rust constant");

void nvim_cmdhist_msg_puts_title(const char *buf) { msg_puts_title(buf); }

void nvim_cmdhist_msg_putchar(int c) { msg_putchar(c); }

void nvim_cmdhist_msg_outtrans(const char *buf) { msg_outtrans(buf, 0, false); }

int nvim_cmdhist_message_filtered(const char *s) { return message_filtered(s); }

int nvim_cmdhist_get_Columns(void) { return Columns; }

int nvim_cmdhist_get_got_int(void) { return got_int; }

int nvim_cmdhist_vim_strsize(const char *s) { return vim_strsize(s); }

void nvim_cmdhist_trunc_string(const char *s, char *buf, int len, int buflen)
{
  trunc_string((char *)s, buf, len, buflen);
}

void nvim_cmdhist_xstrlcpy(char *dst, const char *src, size_t n) { xstrlcpy(dst, src, n); }

char *nvim_cmdhist_format_hist_header(const char *name)
{
  vim_snprintf(IObuff, IOSIZE, "\n      #  %s history", name);
  return IObuff;
}

int nvim_cmdhist_format_hist_entry(int is_current, int hisnum_val)
{
  return snprintf(IObuff, IOSIZE, "%c%6d  ", is_current ? '>' : ' ', hisnum_val);
}

char *nvim_cmdhist_get_IObuff(void) { return IObuff; }

int nvim_cmdhist_get_IOSIZE(void) { return IOSIZE; }

int nvim_cmdhist_get_list_range(char **end, int *val1, int *val2) { return get_list_range(end, val1, val2); }

void nvim_cmdhist_semsg_trailing_arg(const char *s) { semsg(_(e_trailing_arg), s); }

void nvim_cmdhist_semsg_val_too_large(const char *s) { semsg(_(e_val_too_large), s); }

void nvim_cmdhist_msg_history_zero(void) { msg(_("'history' option is zero"), 0); }

char *nvim_cmdhist_eap_get_arg(exarg_T *eap) { return eap->arg; }

void nvim_cmdhist_xp_buf_set(expand_T *xp, int idx, char c) { xp->xp_buf[idx] = c; }

char *nvim_cmdhist_xp_buf_ptr(expand_T *xp) { return xp->xp_buf; }

int nvim_cmdhist_ascii_isdigit(int c) { return ascii_isdigit(c); }

int nvim_cmdhist_ascii_isalpha(int c) { return ASCII_ISALPHA(c); }

/// Convert a pointer within a history array to an index.
int nvim_cmdhist_ptr_to_idx(histentry_T *base, histentry_T *ptr) { return (int)(ptr - base); }
