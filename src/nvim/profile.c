#include <assert.h>
#include <math.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <uv.h>

#include "nvim/ascii_defs.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/debugger.h"
#include "nvim/errors.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/userfunc.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/fileio.h"
#include "nvim/garray.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/hashtab.h"
#include "nvim/hashtab_defs.h"
#include "nvim/keycodes.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/os/fs.h"
#include "nvim/os/os.h"
#include "nvim/os/time.h"
#include "nvim/pos_defs.h"
#include "nvim/profile.h"
#include "nvim/runtime.h"
#include "nvim/types_defs.h"

#include "profile.c.generated.h"

/// Struct used in sn_prl_ga for every line of a script.
typedef struct {
  int snp_count;                ///< nr of times line was executed
  proftime_T sn_prl_total;      ///< time spent in a line + children
  proftime_T sn_prl_self;       ///< time spent in a line itself
} sn_prl_T;

#define PRL_ITEM(si, idx)     (((sn_prl_T *)(si)->sn_prl_ga.ga_data)[(idx)])

static char *startuptime_buf = NULL;  // --startuptime buffer

// Rust implementations replace these C functions.
// Declarations for the Rust functions called by the wrappers below.
extern void rs_func_dump_profile(FILE *fd);
extern void rs_script_dump_profile(FILE *fd);
// rs_profile_init is declared in runtime_ffi.c and called via nvim_rt_profile_init.

// Collect profiled ufunc_T pointers for Rust dump functions.
// Returns a malloc'd array of pointers; caller must free it.
// *out_count is set to the number of entries.
void nvim_profile_collect_profiled_funcs(void ***out_array, int *out_count)
{
  hashtab_T *const functbl = func_tbl_get();
  int todo = (int)functbl->ht_used;
  if (todo == 0) {
    *out_array = NULL;
    *out_count = 0;
    return;
  }

  void **arr = xmalloc(sizeof(void *) * (size_t)todo);
  int n = 0;
  for (hashitem_T *hi = functbl->ht_array; todo > 0; hi++) {
    if (!HASHITEM_EMPTY(hi)) {
      todo--;
      ufunc_T *fp = HI2UF(hi);
      if (fp->uf_prof_initialized) {
        arr[n++] = fp;
      }
    }
  }
  *out_array = arr;
  *out_count = n;
}

// scriptitem_T dump accessors
int nvim_si_get_pr_count(int sid) { return SCRIPT_ITEM(sid)->sn_pr_count; }

proftime_T nvim_si_get_pr_total(int sid) { return SCRIPT_ITEM(sid)->sn_pr_total; }

proftime_T nvim_si_get_pr_self(int sid) { return SCRIPT_ITEM(sid)->sn_pr_self; }

const char *nvim_si_get_name(int sid) { return SCRIPT_ITEM(sid)->sn_name; }

FILE *nvim_si_fopen(int sid) { return os_fopen(SCRIPT_ITEM(sid)->sn_name, "r"); }

// vim_fgets wrapper for Rust
int nvim_profile_vim_fgets(char *buf, int size, FILE *fd) { return vim_fgets(buf, size, fd); }

// IObuff/IOSIZE access
char *nvim_profile_get_iobuff(void) { return IObuff; }

int nvim_profile_get_iosize(void) { return IOSIZE; }

// xfree for Rust-allocated strings (get_scriptname may return alloc'd)
void nvim_profile_xfree_ptr(void *ptr) { xfree(ptr); }

// profile_init setters for scriptitem fields (SID-based, used by script_line.rs)
void nvim_si_set_pr_count(int sid, int val) { SCRIPT_ITEM(sid)->sn_pr_count = val; }

void nvim_si_set_pr_total(int sid, proftime_T val) { SCRIPT_ITEM(sid)->sn_pr_total = val; }

void nvim_si_set_pr_self(int sid, proftime_T val) { SCRIPT_ITEM(sid)->sn_pr_self = val; }

void nvim_si_set_prof_on(int sid, int val) { SCRIPT_ITEM(sid)->sn_prof_on = (val != 0); }

void nvim_si_ga_init_prl(int sid) { ga_init(&SCRIPT_ITEM(sid)->sn_prl_ga, sizeof(sn_prl_T), 100); }

void nvim_si_set_prl_idx_val(int sid, int val) { SCRIPT_ITEM(sid)->sn_prl_idx = val; }

void nvim_si_set_pr_nest_val(int sid, int val) { SCRIPT_ITEM(sid)->sn_pr_nest = val; }

// profile_init: pointer-based setters (scriptitem_T*) for rs_profile_init
void nvim_si_ptr_set_pr_count(scriptitem_T *si, int val) { si->sn_pr_count = val; }

void nvim_si_ptr_set_pr_total(scriptitem_T *si, proftime_T val) { si->sn_pr_total = val; }

void nvim_si_ptr_set_pr_self(scriptitem_T *si, proftime_T val) { si->sn_pr_self = val; }

void nvim_si_ptr_ga_init_prl(scriptitem_T *si) { ga_init(&si->sn_prl_ga, sizeof(sn_prl_T), 100); }

void nvim_si_ptr_set_prl_idx(scriptitem_T *si, int val) { si->sn_prl_idx = val; }

void nvim_si_ptr_set_prof_on(scriptitem_T *si, bool val) { si->sn_prof_on = val; }

void nvim_si_ptr_set_pr_nest(scriptitem_T *si, int val) { si->sn_pr_nest = val; }

// fprintf wrappers for profile dump output (Rust can't call variadic C fprintf)
void nvim_profile_fprintf_str(FILE *fd, const char *s) { fputs(s, fd); }

// C accessors for Phase 2
int nvim_get_script_items_len(void) { return script_items.ga_len; }

int nvim_script_item_get_pr_force(int sid) { return SCRIPT_ITEM(sid)->sn_pr_force ? 1 : 0; }

// C accessors for Phase 4: scriptitem_T profiling fields

int nvim_si_get_prof_on(int sid) { return SCRIPT_ITEM(sid)->sn_prof_on ? 1 : 0; }

int nvim_si_get_prl_idx(int sid) { return SCRIPT_ITEM(sid)->sn_prl_idx; }

void nvim_si_set_prl_idx(int sid, int val) { SCRIPT_ITEM(sid)->sn_prl_idx = val; }

int nvim_si_get_prl_execed(int sid) { return SCRIPT_ITEM(sid)->sn_prl_execed; }

void nvim_si_set_prl_execed(int sid, int val) { SCRIPT_ITEM(sid)->sn_prl_execed = val; }

proftime_T nvim_si_get_prl_start(int sid) { return SCRIPT_ITEM(sid)->sn_prl_start; }

void nvim_si_set_prl_start(int sid, proftime_T val) { SCRIPT_ITEM(sid)->sn_prl_start = val; }

proftime_T nvim_si_get_prl_children(int sid) { return SCRIPT_ITEM(sid)->sn_prl_children; }

void nvim_si_set_prl_children(int sid, proftime_T val) { SCRIPT_ITEM(sid)->sn_prl_children = val; }

proftime_T nvim_si_get_prl_wait(int sid) { return SCRIPT_ITEM(sid)->sn_prl_wait; }

void nvim_si_set_prl_wait(int sid, proftime_T val) { SCRIPT_ITEM(sid)->sn_prl_wait = val; }

int nvim_si_get_pr_nest(int sid) { return SCRIPT_ITEM(sid)->sn_pr_nest; }

void nvim_si_set_pr_nest(int sid, int val) { SCRIPT_ITEM(sid)->sn_pr_nest = val; }

proftime_T nvim_si_get_pr_child(int sid) { return SCRIPT_ITEM(sid)->sn_pr_child; }

void nvim_si_set_pr_child(int sid, proftime_T val) { SCRIPT_ITEM(sid)->sn_pr_child = val; }

proftime_T nvim_si_get_pr_children(int sid) { return SCRIPT_ITEM(sid)->sn_pr_children; }

void nvim_si_set_pr_children(int sid, proftime_T val) { SCRIPT_ITEM(sid)->sn_pr_children = val; }

// garray_T ops for sn_prl_ga
int nvim_si_prl_ga_len(int sid) { return SCRIPT_ITEM(sid)->sn_prl_ga.ga_len; }

void nvim_si_prl_ga_set_len(int sid, int len) { SCRIPT_ITEM(sid)->sn_prl_ga.ga_len = len; }

int nvim_si_prl_ga_maxlen(int sid) { return SCRIPT_ITEM(sid)->sn_prl_ga.ga_maxlen; }

void nvim_si_prl_ga_grow(int sid, int n) { ga_grow(&SCRIPT_ITEM(sid)->sn_prl_ga, n); }

// PRL_ITEM field accessors
int nvim_si_prl_item_get_count(int sid, int idx) { return PRL_ITEM(SCRIPT_ITEM(sid), idx).snp_count; }

void nvim_si_prl_item_set_count(int sid, int idx, int val) { PRL_ITEM(SCRIPT_ITEM(sid), idx).snp_count = val; }

proftime_T nvim_si_prl_item_get_total(int sid, int idx) { return PRL_ITEM(SCRIPT_ITEM(sid), idx).sn_prl_total; }

void nvim_si_prl_item_set_total(int sid, int idx, proftime_T val)
{
  PRL_ITEM(SCRIPT_ITEM(sid), idx).sn_prl_total = val;
}

proftime_T nvim_si_prl_item_get_self(int sid, int idx) { return PRL_ITEM(SCRIPT_ITEM(sid), idx).sn_prl_self; }

void nvim_si_prl_item_set_self(int sid, int idx, proftime_T val) { PRL_ITEM(SCRIPT_ITEM(sid), idx).sn_prl_self = val; }

// C accessors for Phase 6: startup timing FILE* management

FILE *nvim_profile_get_time_fd(void) { return time_fd; }

void nvim_profile_set_time_fd(FILE *fd) { time_fd = fd; }

FILE *nvim_profile_fopen(const char *name, const char *mode) { return fopen(name, mode); }

void nvim_profile_fclose(FILE *fd) { fclose(fd); }

void nvim_profile_fputs(const char *s, FILE *fd) { fputs(s, fd); }

char *nvim_profile_xmalloc(size_t size) { return xmalloc(size); }

void nvim_profile_xfree(char *ptr) { xfree(ptr); }

int nvim_profile_setvbuf(FILE *fd, char *buf, size_t size) { return setvbuf(fd, buf, _IOFBF, size); }

void nvim_profile_fprintf_stderr(const char *s) { fprintf(stderr, "%s", s); }

char *nvim_profile_get_startuptime_buf(void) { return startuptime_buf; }

void nvim_profile_set_startuptime_buf(char *buf) { startuptime_buf = buf; }

const char *nvim_profile_gettext_e_notopen(void) { return _(e_notopen); }

const char *nvim_profile_uv_err_name(int r) { return uv_err_name(r); }

// C accessors for Phase 7: command handling, state management, dump

int nvim_profile_get_prof_none(void) { return PROF_NONE; }

int nvim_profile_get_prof_yes(void) { return PROF_YES; }

int nvim_profile_get_prof_paused(void) { return PROF_PAUSED; }

int nvim_profile_get_do_profiling(void) { return do_profiling; }

void nvim_profile_set_do_profiling(int val) { do_profiling = val; }

void nvim_profile_set_vim_var_nr_profiling(int val) { set_vim_var_nr(VV_PROFILING, val); }

void nvim_profile_emsg_e750(void)
{
  emsg(_("E750: First use \":profile start {fname}\""));
}

void nvim_profile_semsg_notopen(const char *fname) { semsg(_(e_notopen), fname); }

void nvim_profile_ex_breakadd(exarg_T *eap) { ex_breakadd(eap); }

char *nvim_profile_expand_env_save_opt(char *src) { return expand_env_save_opt(src, true); }

char *nvim_profile_eap_get_arg(exarg_T *eap) { return eap->arg; }

char *nvim_profile_skiptowhite(const char *s) { return skiptowhite(s); }

char *nvim_profile_skipwhite(const char *s) { return skipwhite(s); }

void nvim_profile_xp_set_context(expand_T *xp, int ctx) { xp->xp_context = ctx; }

void nvim_profile_xp_set_pattern(expand_T *xp, const char *pat) { xp->xp_pattern = (char *)pat; }

int nvim_profile_get_expand_profile(void) { return EXPAND_PROFILE; }

int nvim_profile_get_expand_files(void) { return EXPAND_FILES; }

int nvim_profile_get_expand_user_func(void) { return EXPAND_USER_FUNC; }

int nvim_profile_get_expand_nothing(void) { return EXPAND_NOTHING; }

FILE *nvim_profile_os_fopen(const char *name, const char *mode) { return os_fopen(name, mode); }

void nvim_profile_reset_scripts(void)
{
  for (int id = 1; id <= script_items.ga_len; id++) {
    scriptitem_T *si = SCRIPT_ITEM(id);
    if (si->sn_prof_on) {
      si->sn_prof_on = false;
      si->sn_pr_force = false;
      si->sn_pr_child = profile_zero();
      si->sn_pr_nest = 0;
      si->sn_pr_count = 0;
      si->sn_pr_total = profile_zero();
      si->sn_pr_self = profile_zero();
      si->sn_pr_start = profile_zero();
      si->sn_pr_children = profile_zero();
      ga_clear(&si->sn_prl_ga);
      si->sn_prl_start = profile_zero();
      si->sn_prl_children = profile_zero();
      si->sn_prl_wait = profile_zero();
      si->sn_prl_idx = -1;
      si->sn_prl_execed = 0;
    }
  }
}

void nvim_profile_reset_funcs(void)
{
  hashtab_T *const functbl = func_tbl_get();
  size_t todo = functbl->ht_used;
  hashitem_T *hi = functbl->ht_array;

  for (; todo > 0; hi++) {
    if (!HASHITEM_EMPTY(hi)) {
      todo--;
      ufunc_T *uf = HI2UF(hi);
      if (uf->uf_prof_initialized) {
        uf->uf_profiling = 0;
        uf->uf_tm_count = 0;
        uf->uf_tm_total = profile_zero();
        uf->uf_tm_self = profile_zero();
        uf->uf_tm_children = profile_zero();

        for (int i = 0; i < uf->uf_lines.ga_len; i++) {
          uf->uf_tml_count[i] = 0;
          uf->uf_tml_total[i] = uf->uf_tml_self[i] = 0;
        }

        uf->uf_tml_start = profile_zero();
        uf->uf_tml_children = profile_zero();
        uf->uf_tml_wait = profile_zero();
        uf->uf_tml_idx = -1;
        uf->uf_tml_execed = 0;
      }
    }
  }
}

void nvim_profile_script_dump(FILE *fd) { rs_script_dump_profile(fd); }

void nvim_profile_func_dump(FILE *fd) { rs_func_dump_profile(fd); }
