#pragma once

#include <stdio.h>  // IWYU pragma: keep

#include "nvim/decoration_defs.h"  // IWYU pragma: keep
#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/fold_defs.h"  // IWYU pragma: keep
#include "nvim/garray_defs.h"  // IWYU pragma: keep
#include "nvim/macros_defs.h"
#include "nvim/pos_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

EXTERN int disable_fold_update INIT( = 0);

/// Result struct for fold level calculations (used by Rust FFI).
typedef struct {
  int lvl;
  int lvl_next;
  int start;
  int end;
} FoldLevelResult_C;

#include "fold.h.generated.h"
