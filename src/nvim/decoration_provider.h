#pragma once

#include <stdbool.h>

#include "nvim/decoration_defs.h"  // IWYU pragma: keep
#include "nvim/macros_defs.h"
#include "nvim/types_defs.h"  // IWYU pragma: keep

/// Result of a range provider invocation (returned from C to Rust).
typedef struct {
  bool ok;        ///< false = error, true = success
  bool stop_win;  ///< true if provider returned false (stop this window)
  bool has_skip;  ///< true if provider returned a skip coordinate
  int skip_row;   ///< skip row (valid when has_skip)
  int skip_col;   ///< skip col (valid when has_skip)
} DecorRangeInvokeResult;

#include "decoration_provider.h.generated.h"
