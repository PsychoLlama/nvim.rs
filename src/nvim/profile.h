#pragma once

#include <stdbool.h>  // IWYU pragma: keep
#include <stdint.h>  // IWYU pragma: keep
#include <time.h>

#include "nvim/cmdexpand_defs.h"  // IWYU pragma: keep
#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/runtime_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

#define TIME_MSG(s) do { \
  if (time_fd != NULL) time_msg(s, NULL); \
} while (0)

// Rust-implemented profile functions (exported via #[export_name])
// Phase 1: arithmetic and comparison
extern proftime_T profile_zero(void);
extern proftime_T profile_divide(proftime_T tm, int count);
extern proftime_T profile_add(proftime_T tm1, proftime_T tm2);
extern proftime_T profile_sub(proftime_T tm1, proftime_T tm2);
extern proftime_T profile_self(proftime_T self_time, proftime_T total, proftime_T children);
extern bool profile_equal(proftime_T tm1, proftime_T tm2);
extern int64_t profile_signed(proftime_T tm);
extern int profile_cmp(proftime_T tm1, proftime_T tm2);
extern proftime_T profile_get_wait(void);
extern proftime_T profile_sub_wait(proftime_T tm, proftime_T tma);
// Phase 2: timing, state, command
extern proftime_T profile_start(void);
extern proftime_T profile_end(proftime_T tm);
extern const char *profile_msg(proftime_T tm);
extern proftime_T profile_setlimit(int64_t msec);
extern bool profile_passed_limit(proftime_T tm);
extern void profile_set_wait(proftime_T wait);
extern void profile_reset(void);
extern void profile_dump(void);
extern void ex_profile(exarg_T *eap);
extern char *get_profile_name(expand_T *xp, int idx);
extern void set_context_in_profile_cmd(expand_T *xp, const char *arg);
// Phase 3: input, function line, script line, child, startup
extern void profile_init(scriptitem_T *si);
extern void prof_input_start(void);
extern void prof_input_end(void);
extern bool prof_def_func(void);
extern void func_line_start(void *cookie);
extern void func_line_exec(void *cookie);
extern void func_line_end(void *cookie);
extern void func_do_profile(ufunc_T *fp);
extern void prof_child_enter(proftime_T *tm);
extern void prof_child_exit(proftime_T *tm);
extern void script_line_start(void);
extern void script_line_exec(void);
extern void script_line_end(void);
extern void script_prof_save(proftime_T *tm);
extern void script_prof_restore(const proftime_T *tm);
extern void time_push(proftime_T *rel, proftime_T *start);
extern void time_pop(proftime_T tp);
extern void time_start(const char *message);
extern void time_msg(const char *mesg, const proftime_T *start);
extern void time_init(const char *fname, const char *proc_name);
extern void time_finish(void);

#include "profile.h.generated.h"
