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

#include "profile.h.generated.h"
