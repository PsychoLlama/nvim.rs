#pragma once

#include "nvim/buffer_defs.h"  // IWYU pragma: keep
#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/garray_defs.h"  // IWYU pragma: keep
#include "nvim/pos_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

#include "eval/funcs_shim.h.generated.h"

// Hand-written prototype: grammar skips function-pointer params.
int nvim_readdir_core(garray_T *gap, const char *path, void *context,
                      varnumber_T (*checkitem)(void *context, const char *name));
