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

// Rust implementations of profile functions
extern proftime_T rs_profile_zero(void);
extern proftime_T rs_profile_divide(proftime_T tm, int count);
extern proftime_T rs_profile_add(proftime_T tm1, proftime_T tm2);
extern proftime_T rs_profile_sub(proftime_T tm1, proftime_T tm2);
extern proftime_T rs_profile_self(proftime_T self, proftime_T total, proftime_T children);
extern bool rs_profile_equal(proftime_T tm1, proftime_T tm2);
extern int64_t rs_profile_signed(proftime_T tm);
extern int rs_profile_cmp(proftime_T tm1, proftime_T tm2);
extern proftime_T rs_profile_get_wait(void);
extern proftime_T rs_profile_sub_wait(proftime_T tm, proftime_T tma);
// Phase 1: timing wrappers
extern proftime_T rs_profile_start(void);
extern proftime_T rs_profile_end(proftime_T tm);
extern const char *rs_profile_msg(proftime_T tm);
extern proftime_T rs_profile_setlimit(int64_t msec);
extern bool rs_profile_passed_limit(proftime_T tm);
extern void rs_profile_set_wait(proftime_T wait);
extern proftime_T rs_profile_get_wait_time(void);
// Phase 2: input/wait profiling
extern void rs_prof_input_start(void);
extern void rs_prof_input_end(void);
extern bool rs_prof_def_func(void);
// Phase 3: function line profiling
extern void rs_func_line_start(void *cookie);
extern void rs_func_line_exec(void *cookie);
extern void rs_func_line_end(void *cookie);
extern void rs_func_do_profile(ufunc_T *fp);
// Phase 4: script line profiling
extern void rs_script_line_start(void);
extern void rs_script_line_exec(void);
extern void rs_script_line_end(void);
extern void rs_script_prof_save(proftime_T *tm);
extern void rs_script_prof_restore(const proftime_T *tm);
// Phase 5: child profiling
extern void rs_prof_child_enter(proftime_T *tm);
extern void rs_prof_child_exit(proftime_T *tm);
// Phase 6: startup timing
extern void rs_time_push(proftime_T *rel, proftime_T *start);
extern void rs_time_pop(proftime_T tp);
extern void rs_time_start(const char *message);
extern void rs_time_msg(const char *mesg, const proftime_T *start);
extern void rs_time_init(const char *fname, const char *proc_name);
extern void rs_time_finish(void);

/// Struct used in sn_prl_ga for every line of a script.
typedef struct {
  int snp_count;                ///< nr of times line was executed
  proftime_T sn_prl_total;      ///< time spent in a line + children
  proftime_T sn_prl_self;       ///< time spent in a line itself
} sn_prl_T;

#define PRL_ITEM(si, idx)     (((sn_prl_T *)(si)->sn_prl_ga.ga_data)[(idx)])

static char *startuptime_buf = NULL;  // --startuptime buffer

/// Gets the current time.
///
/// @return the current time
proftime_T profile_start(void) FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_profile_start();
}

/// Computes the time elapsed.
///
/// @return Elapsed time from `tm` until now.
proftime_T profile_end(proftime_T tm) FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_profile_end(tm);
}

/// Gets a string representing time `tm`.
///
/// @warning Do not modify or free this string, not multithread-safe.
///
/// @param tm Time
/// @return Static string representing `tm` in the form "seconds.microseconds".
const char *profile_msg(proftime_T tm) FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_profile_msg(tm);
}

/// Gets the time `msec` into the future.
///
/// @param msec milliseconds, the maximum number of milliseconds is
///             (2^63 / 10^6) - 1 = 9.223372e+12.
/// @return if msec > 0, returns the time msec past now. Otherwise returns
///         the zero time.
proftime_T profile_setlimit(int64_t msec) FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_profile_setlimit(msec);
}

/// Checks if current time has passed `tm`.
///
/// @return true if the current time is past `tm`, false if not or if the
///         timer was not set.
bool profile_passed_limit(proftime_T tm) FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_profile_passed_limit(tm);
}

/// Gets the zero time.
///
/// @return the zero time
proftime_T profile_zero(void) FUNC_ATTR_CONST
{
  return rs_profile_zero();
}

/// Divides time `tm` by `count`.
///
/// @return 0 if count <= 0, otherwise tm / count
proftime_T profile_divide(proftime_T tm, int count) FUNC_ATTR_CONST
{
  return rs_profile_divide(tm, count);
}

/// Adds time `tm2` to `tm1`.
///
/// @return `tm1` + `tm2`
proftime_T profile_add(proftime_T tm1, proftime_T tm2) FUNC_ATTR_CONST
{
  return rs_profile_add(tm1, tm2);
}

/// Subtracts time `tm2` from `tm1`.
///
/// Unsigned overflow (wraparound) occurs if `tm2` is greater than `tm1`.
/// Use `profile_signed()` to get the signed integer value.
///
/// @see profile_signed
///
/// @return `tm1` - `tm2`
proftime_T profile_sub(proftime_T tm1, proftime_T tm2) FUNC_ATTR_CONST
{
  return rs_profile_sub(tm1, tm2);
}

/// Adds the `self` time from the total time and the `children` time.
///
/// @return if `total` <= `children`, then self, otherwise `self` + `total` -
///         `children`
proftime_T profile_self(proftime_T self, proftime_T total, proftime_T children)
  FUNC_ATTR_CONST
{
  return rs_profile_self(self, total, children);
}

/// Gets the current waittime.
///
/// @return the current waittime
static proftime_T profile_get_wait(void) FUNC_ATTR_PURE
{
  return rs_profile_get_wait();
}

/// Sets the current waittime.
void profile_set_wait(proftime_T wait)
{
  rs_profile_set_wait(wait);
}

/// Subtracts the passed waittime since `tm`.
///
/// @return `tma` - (waittime - `tm`)
proftime_T profile_sub_wait(proftime_T tm, proftime_T tma) FUNC_ATTR_PURE
{
  return rs_profile_sub_wait(tm, tma);
}

/// Checks if time `tm1` is equal to `tm2`.
///
/// @return true if `tm1` == `tm2`
static bool profile_equal(proftime_T tm1, proftime_T tm2) FUNC_ATTR_CONST
{
  return rs_profile_equal(tm1, tm2);
}

/// Converts time duration `tm` (`profile_sub` result) to a signed integer.
///
/// @return signed representation of the given time value
int64_t profile_signed(proftime_T tm)
  FUNC_ATTR_CONST
{
  return rs_profile_signed(tm);
}

/// Compares profiling times.
///
/// Times `tm1` and `tm2` must be less than 150 years apart.
///
/// @return <0: `tm2` < `tm1`
///          0: `tm2` == `tm1`
///         >0: `tm2` > `tm1`
int profile_cmp(proftime_T tm1, proftime_T tm2) FUNC_ATTR_CONST
{
  return rs_profile_cmp(tm1, tm2);
}

static char *profile_fname = NULL;

/// Reset all profiling information.
void profile_reset(void)
{
  // Reset sourced files.
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

  // Reset functions.
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

  XFREE_CLEAR(profile_fname);
}

/// ":profile cmd args"
void ex_profile(exarg_T *eap)
{
  static proftime_T pause_time;

  char *e = skiptowhite(eap->arg);
  int len = (int)(e - eap->arg);
  e = skipwhite(e);

  if (len == 5 && strncmp(eap->arg, "start", 5) == 0 && *e != NUL) {
    xfree(profile_fname);
    profile_fname = expand_env_save_opt(e, true);
    do_profiling = PROF_YES;
    profile_set_wait(profile_zero());
    set_vim_var_nr(VV_PROFILING, 1);
  } else if (do_profiling == PROF_NONE) {
    emsg(_("E750: First use \":profile start {fname}\""));
  } else if (strcmp(eap->arg, "stop") == 0) {
    profile_dump();
    do_profiling = PROF_NONE;
    set_vim_var_nr(VV_PROFILING, 0);
    profile_reset();
  } else if (strcmp(eap->arg, "pause") == 0) {
    if (do_profiling == PROF_YES) {
      pause_time = profile_start();
    }
    do_profiling = PROF_PAUSED;
  } else if (strcmp(eap->arg, "continue") == 0) {
    if (do_profiling == PROF_PAUSED) {
      pause_time = profile_end(pause_time);
      profile_set_wait(profile_add(profile_get_wait(), pause_time));
    }
    do_profiling = PROF_YES;
  } else if (strcmp(eap->arg, "dump") == 0) {
    profile_dump();
  } else {
    // The rest is similar to ":breakadd".
    ex_breakadd(eap);
  }
}

/// Command line expansion for :profile.
static enum {
  PEXP_SUBCMD,          ///< expand :profile sub-commands
  PEXP_FUNC,  ///< expand :profile func {funcname}
} pexpand_what;

static char *pexpand_cmds[] = {
  "continue",
  "dump",
  "file",
  "func",
  "pause",
  "start",
  "stop",
  NULL
};

/// Function given to ExpandGeneric() to obtain the profile command
/// specific expansion.
char *get_profile_name(expand_T *xp, int idx)
  FUNC_ATTR_PURE
{
  switch (pexpand_what) {
  case PEXP_SUBCMD:
    return pexpand_cmds[idx];
  default:
    return NULL;
  }
}

/// Handle command line completion for :profile command.
void set_context_in_profile_cmd(expand_T *xp, const char *arg)
{
  // Default: expand subcommands.
  xp->xp_context = EXPAND_PROFILE;
  pexpand_what = PEXP_SUBCMD;
  xp->xp_pattern = (char *)arg;

  char *const end_subcmd = skiptowhite(arg);
  if (*end_subcmd == NUL) {
    return;
  }

  if ((end_subcmd - arg == 5 && strncmp(arg, "start", 5) == 0)
      || (end_subcmd - arg == 4 && strncmp(arg, "file", 4) == 0)) {
    xp->xp_context = EXPAND_FILES;
    xp->xp_pattern = skipwhite(end_subcmd);
    return;
  } else if (end_subcmd - arg == 4 && strncmp(arg, "func", 4) == 0) {
    xp->xp_context = EXPAND_USER_FUNC;
    xp->xp_pattern = skipwhite(end_subcmd);
    return;
  }

  xp->xp_context = EXPAND_NOTHING;
}

/// Called when starting to wait for the user to type a character.
void prof_input_start(void)
{
  rs_prof_input_start();
}

/// Called when finished waiting for the user to type a character.
void prof_input_end(void)
{
  rs_prof_input_end();
}

/// @return  true when a function defined in the current script should be
///          profiled.
bool prof_def_func(void)
  FUNC_ATTR_PURE
{
  return rs_prof_def_func();
}

/// Print the count and times for one function or function line.
///
/// @param prefer_self  when equal print only self time
static void prof_func_line(FILE *fd, int count, const proftime_T *total, const proftime_T *self,
                           bool prefer_self)
{
  if (count > 0) {
    fprintf(fd, "%5d ", count);
    if (prefer_self && profile_equal(*total, *self)) {
      fprintf(fd, "           ");
    } else {
      fprintf(fd, "%s ", profile_msg(*total));
    }
    if (!prefer_self && profile_equal(*total, *self)) {
      fprintf(fd, "           ");
    } else {
      fprintf(fd, "%s ", profile_msg(*self));
    }
  } else {
    fprintf(fd, "                            ");
  }
}

/// @param prefer_self  when equal print only self time
static void prof_sort_list(FILE *fd, ufunc_T **sorttab, int st_len, char *title, bool prefer_self)
{
  fprintf(fd, "FUNCTIONS SORTED ON %s TIME\n", title);
  fprintf(fd, "count  total (s)   self (s)  function\n");
  for (int i = 0; i < 20 && i < st_len; i++) {
    ufunc_T *fp = sorttab[i];
    prof_func_line(fd, fp->uf_tm_count, &fp->uf_tm_total, &fp->uf_tm_self,
                   prefer_self);
    if ((uint8_t)fp->uf_name[0] == K_SPECIAL) {
      fprintf(fd, " <SNR>%s()\n", fp->uf_name + 3);
    } else {
      fprintf(fd, " %s()\n", fp->uf_name);
    }
  }
  fprintf(fd, "\n");
}

/// Compare function for total time sorting.
static int prof_total_cmp(const void *s1, const void *s2)
{
  ufunc_T *p1 = *(ufunc_T **)s1;
  ufunc_T *p2 = *(ufunc_T **)s2;
  return profile_cmp(p1->uf_tm_total, p2->uf_tm_total);
}

/// Compare function for self time sorting.
static int prof_self_cmp(const void *s1, const void *s2)
{
  ufunc_T *p1 = *(ufunc_T **)s1;
  ufunc_T *p2 = *(ufunc_T **)s2;
  return profile_cmp(p1->uf_tm_self, p2->uf_tm_self);
}

/// Start profiling function "fp".
void func_do_profile(ufunc_T *fp)
{
  rs_func_do_profile(fp);
}

/// Prepare profiling for entering a child or something else that is not
/// counted for the script/function itself.
/// Should always be called in pair with prof_child_exit().
///
/// @param tm  place to store waittime
void prof_child_enter(proftime_T *tm)
{
  rs_prof_child_enter(tm);
}

/// Take care of time spent in a child.
/// Should always be called after prof_child_enter().
///
/// @param tm  where waittime was stored
void prof_child_exit(proftime_T *tm)
{
  rs_prof_child_exit(tm);
}

/// Called when starting to read a function line.
/// "sourcing_lnum" must be correct!
/// When skipping lines it may not actually be executed, but we won't find out
/// until later and we need to store the time now.
void func_line_start(void *cookie)
{
  rs_func_line_start(cookie);
}

/// Called when actually executing a function line.
void func_line_exec(void *cookie)
{
  rs_func_line_exec(cookie);
}

/// Called when done with a function line.
void func_line_end(void *cookie)
{
  rs_func_line_end(cookie);
}

/// Dump the profiling results for all functions in file "fd".
static void func_dump_profile(FILE *fd)
{
  hashtab_T *const functbl = func_tbl_get();
  int st_len = 0;

  int todo = (int)functbl->ht_used;
  if (todo == 0) {
    return;         // nothing to dump
  }

  ufunc_T **sorttab = xmalloc(sizeof(ufunc_T *) * (size_t)todo);

  for (hashitem_T *hi = functbl->ht_array; todo > 0; hi++) {
    if (!HASHITEM_EMPTY(hi)) {
      todo--;
      ufunc_T *fp = HI2UF(hi);
      if (fp->uf_prof_initialized) {
        sorttab[st_len++] = fp;

        if ((uint8_t)fp->uf_name[0] == K_SPECIAL) {
          fprintf(fd, "FUNCTION  <SNR>%s()\n", fp->uf_name + 3);
        } else {
          fprintf(fd, "FUNCTION  %s()\n", fp->uf_name);
        }
        if (fp->uf_script_ctx.sc_sid != 0) {
          bool should_free;
          char *p = get_scriptname(fp->uf_script_ctx, &should_free);
          fprintf(fd, "    Defined: %s:%" PRIdLINENR "\n",
                  p, fp->uf_script_ctx.sc_lnum);
          if (should_free) {
            xfree(p);
          }
        }
        if (fp->uf_tm_count == 1) {
          fprintf(fd, "Called 1 time\n");
        } else {
          fprintf(fd, "Called %d times\n", fp->uf_tm_count);
        }
        fprintf(fd, "Total time: %s\n", profile_msg(fp->uf_tm_total));
        fprintf(fd, " Self time: %s\n", profile_msg(fp->uf_tm_self));
        fprintf(fd, "\n");
        fprintf(fd, "count  total (s)   self (s)\n");

        for (int i = 0; i < fp->uf_lines.ga_len; i++) {
          if (FUNCLINE(fp, i) == NULL) {
            continue;
          }
          prof_func_line(fd, fp->uf_tml_count[i],
                         &fp->uf_tml_total[i], &fp->uf_tml_self[i], true);
          fprintf(fd, "%s\n", FUNCLINE(fp, i));
        }
        fprintf(fd, "\n");
      }
    }
  }

  if (st_len > 0) {
    qsort((void *)sorttab, (size_t)st_len, sizeof(ufunc_T *),
          prof_total_cmp);
    prof_sort_list(fd, sorttab, st_len, "TOTAL", false);
    qsort((void *)sorttab, (size_t)st_len, sizeof(ufunc_T *),
          prof_self_cmp);
    prof_sort_list(fd, sorttab, st_len, "SELF", true);
  }

  xfree(sorttab);
}

/// Start profiling a script.
void profile_init(scriptitem_T *si)
{
  si->sn_pr_count = 0;
  si->sn_pr_total = profile_zero();
  si->sn_pr_self = profile_zero();

  ga_init(&si->sn_prl_ga, sizeof(sn_prl_T), 100);
  si->sn_prl_idx = -1;
  si->sn_prof_on = true;
  si->sn_pr_nest = 0;
}

/// Save time when starting to invoke another script or function.
///
/// @param tm  place to store wait time
void script_prof_save(proftime_T *tm)
{
  rs_script_prof_save(tm);
}

/// Count time spent in children after invoking another script or function.
void script_prof_restore(const proftime_T *tm)
{
  rs_script_prof_restore(tm);
}

/// Dump the profiling results for all scripts in file "fd".
static void script_dump_profile(FILE *fd)
{
  sn_prl_T *pp;

  for (int id = 1; id <= script_items.ga_len; id++) {
    scriptitem_T *si = SCRIPT_ITEM(id);
    if (si->sn_prof_on) {
      fprintf(fd, "SCRIPT  %s\n", si->sn_name);
      if (si->sn_pr_count == 1) {
        fprintf(fd, "Sourced 1 time\n");
      } else {
        fprintf(fd, "Sourced %d times\n", si->sn_pr_count);
      }
      fprintf(fd, "Total time: %s\n", profile_msg(si->sn_pr_total));
      fprintf(fd, " Self time: %s\n", profile_msg(si->sn_pr_self));
      fprintf(fd, "\n");
      fprintf(fd, "count  total (s)   self (s)\n");

      FILE *sfd = os_fopen(si->sn_name, "r");
      if (sfd == NULL) {
        fprintf(fd, "Cannot open file!\n");
      } else {
        // Keep going till the end of file, so that trailing
        // continuation lines are listed.
        for (int i = 0;; i++) {
          if (vim_fgets(IObuff, IOSIZE, sfd)) {
            break;
          }
          // When a line has been truncated, append NL, taking care
          // of multi-byte characters .
          if (IObuff[IOSIZE - 2] != NUL && IObuff[IOSIZE - 2] != NL) {
            int n = IOSIZE - 2;

            // Move to the first byte of this char.
            // utf_head_off() doesn't work, because it checks
            // for a truncated character.
            while (n > 0 && (IObuff[n] & 0xc0) == 0x80) {
              n--;
            }

            IObuff[n] = NL;
            IObuff[n + 1] = NUL;
          }
          if (i < si->sn_prl_ga.ga_len
              && (pp = &PRL_ITEM(si, i))->snp_count > 0) {
            fprintf(fd, "%5d ", pp->snp_count);
            if (profile_equal(pp->sn_prl_total, pp->sn_prl_self)) {
              fprintf(fd, "           ");
            } else {
              fprintf(fd, "%s ", profile_msg(pp->sn_prl_total));
            }
            fprintf(fd, "%s ", profile_msg(pp->sn_prl_self));
          } else {
            fprintf(fd, "                            ");
          }
          fprintf(fd, "%s", IObuff);
        }
        fclose(sfd);
      }
      fprintf(fd, "\n");
    }
  }
}

/// Dump the profiling info.
void profile_dump(void)
{
  if (profile_fname == NULL) {
    return;
  }

  FILE *fd = os_fopen(profile_fname, "w");
  if (fd == NULL) {
    semsg(_(e_notopen), profile_fname);
  } else {
    script_dump_profile(fd);
    func_dump_profile(fd);
    fclose(fd);
  }
}

/// Called when starting to read a script line.
/// "sourcing_lnum" must be correct!
/// When skipping lines it may not actually be executed, but we won't find out
/// until later and we need to store the time now.
void script_line_start(void)
{
  rs_script_line_start();
}

/// Called when actually executing a function line.
void script_line_exec(void)
{
  rs_script_line_exec();
}

/// Called when done with a function line.
void script_line_end(void)
{
  rs_script_line_end();
}

/// Saves the previous time before doing something that could nest.
///
/// @param[out] rel to the time elapsed so far
/// @param[out] start the current time
void time_push(proftime_T *rel, proftime_T *start)
{
  rs_time_push(rel, start);
}

/// Computes the prev time after doing something that could nest.
///
/// @param tp the time to subtract
void time_pop(proftime_T tp)
{
  rs_time_pop(tp);
}

/// Initializes the startuptime code.
///
/// @param message the message that will be displayed
void time_start(const char *message)
{
  rs_time_start(message);
}

/// Prints out timing info.
///
/// @param mesg the message to display next to the timing information
/// @param start only for do_source: start time
void time_msg(const char *mesg, const proftime_T *start)
{
  rs_time_msg(mesg, start);
}

/// Initializes the `time_fd` stream for the --startuptime report.
///
/// @param fname startuptime report file path
/// @param proc_name name of the current Nvim process to write in the report.
void time_init(const char *fname, const char *proc_name)
{
  rs_time_init(fname, proc_name);
}

/// Flushes the startuptimes to disk for the current process
void time_finish(void)
{
  rs_time_finish();
}

// C accessor for Rust — now delegates to the Rust-owned state
proftime_T nvim_get_prof_wait_time(void)
{
  return rs_profile_get_wait_time();
}

// C accessors for Phase 2
int nvim_get_script_items_len(void)
{
  return script_items.ga_len;
}

int nvim_script_item_get_pr_force(int sid)
{
  return SCRIPT_ITEM(sid)->sn_pr_force ? 1 : 0;
}

// C accessors for Phase 4: scriptitem_T profiling fields

int nvim_si_get_prof_on(int sid)
{
  return SCRIPT_ITEM(sid)->sn_prof_on ? 1 : 0;
}

int nvim_si_get_prl_idx(int sid)
{
  return SCRIPT_ITEM(sid)->sn_prl_idx;
}

void nvim_si_set_prl_idx(int sid, int val)
{
  SCRIPT_ITEM(sid)->sn_prl_idx = val;
}

int nvim_si_get_prl_execed(int sid)
{
  return SCRIPT_ITEM(sid)->sn_prl_execed;
}

void nvim_si_set_prl_execed(int sid, int val)
{
  SCRIPT_ITEM(sid)->sn_prl_execed = val;
}

proftime_T nvim_si_get_prl_start(int sid)
{
  return SCRIPT_ITEM(sid)->sn_prl_start;
}

void nvim_si_set_prl_start(int sid, proftime_T val)
{
  SCRIPT_ITEM(sid)->sn_prl_start = val;
}

proftime_T nvim_si_get_prl_children(int sid)
{
  return SCRIPT_ITEM(sid)->sn_prl_children;
}

void nvim_si_set_prl_children(int sid, proftime_T val)
{
  SCRIPT_ITEM(sid)->sn_prl_children = val;
}

proftime_T nvim_si_get_prl_wait(int sid)
{
  return SCRIPT_ITEM(sid)->sn_prl_wait;
}

void nvim_si_set_prl_wait(int sid, proftime_T val)
{
  SCRIPT_ITEM(sid)->sn_prl_wait = val;
}

int nvim_si_get_pr_nest(int sid)
{
  return SCRIPT_ITEM(sid)->sn_pr_nest;
}

void nvim_si_set_pr_nest(int sid, int val)
{
  SCRIPT_ITEM(sid)->sn_pr_nest = val;
}

proftime_T nvim_si_get_pr_child(int sid)
{
  return SCRIPT_ITEM(sid)->sn_pr_child;
}

void nvim_si_set_pr_child(int sid, proftime_T val)
{
  SCRIPT_ITEM(sid)->sn_pr_child = val;
}

proftime_T nvim_si_get_pr_children(int sid)
{
  return SCRIPT_ITEM(sid)->sn_pr_children;
}

void nvim_si_set_pr_children(int sid, proftime_T val)
{
  SCRIPT_ITEM(sid)->sn_pr_children = val;
}

// garray_T ops for sn_prl_ga
int nvim_si_prl_ga_len(int sid)
{
  return SCRIPT_ITEM(sid)->sn_prl_ga.ga_len;
}

void nvim_si_prl_ga_set_len(int sid, int len)
{
  SCRIPT_ITEM(sid)->sn_prl_ga.ga_len = len;
}

int nvim_si_prl_ga_maxlen(int sid)
{
  return SCRIPT_ITEM(sid)->sn_prl_ga.ga_maxlen;
}

void nvim_si_prl_ga_grow(int sid, int n)
{
  ga_grow(&SCRIPT_ITEM(sid)->sn_prl_ga, n);
}

// PRL_ITEM field accessors
int nvim_si_prl_item_get_count(int sid, int idx)
{
  return PRL_ITEM(SCRIPT_ITEM(sid), idx).snp_count;
}

void nvim_si_prl_item_set_count(int sid, int idx, int val)
{
  PRL_ITEM(SCRIPT_ITEM(sid), idx).snp_count = val;
}

proftime_T nvim_si_prl_item_get_total(int sid, int idx)
{
  return PRL_ITEM(SCRIPT_ITEM(sid), idx).sn_prl_total;
}

void nvim_si_prl_item_set_total(int sid, int idx, proftime_T val)
{
  PRL_ITEM(SCRIPT_ITEM(sid), idx).sn_prl_total = val;
}

proftime_T nvim_si_prl_item_get_self(int sid, int idx)
{
  return PRL_ITEM(SCRIPT_ITEM(sid), idx).sn_prl_self;
}

void nvim_si_prl_item_set_self(int sid, int idx, proftime_T val)
{
  PRL_ITEM(SCRIPT_ITEM(sid), idx).sn_prl_self = val;
}

// C accessors for Phase 6: startup timing FILE* management

FILE *nvim_profile_get_time_fd(void)
{
  return time_fd;
}

void nvim_profile_set_time_fd(FILE *fd)
{
  time_fd = fd;
}

FILE *nvim_profile_fopen(const char *name, const char *mode)
{
  return fopen(name, mode);
}

void nvim_profile_fclose(FILE *fd)
{
  fclose(fd);
}

void nvim_profile_fputs(const char *s, FILE *fd)
{
  fputs(s, fd);
}

char *nvim_profile_xmalloc(size_t size)
{
  return xmalloc(size);
}

void nvim_profile_xfree(char *ptr)
{
  xfree(ptr);
}

int nvim_profile_setvbuf(FILE *fd, char *buf, size_t size)
{
  return setvbuf(fd, buf, _IOFBF, size);
}

void nvim_profile_fprintf_stderr(const char *s)
{
  fprintf(stderr, "%s", s);
}

char *nvim_profile_get_startuptime_buf(void)
{
  return startuptime_buf;
}

void nvim_profile_set_startuptime_buf(char *buf)
{
  startuptime_buf = buf;
}

const char *nvim_profile_gettext_e_notopen(void)
{
  return _(e_notopen);
}

const char *nvim_profile_uv_err_name(int r)
{
  return uv_err_name(r);
}
