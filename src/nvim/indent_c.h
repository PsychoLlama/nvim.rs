#pragma once

#include <stdbool.h>
#include <stdint.h>

#include "nvim/eval/typval_defs.h"
#include "nvim/pos_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

/// Result of findmatchlimit, passed across FFI boundary.
typedef struct {
  bool found;
  int lnum;
  int col;
} FindMatchResult;

#include "indent_c.h.generated.h"
