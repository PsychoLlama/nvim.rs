#pragma once

#include "nvim/buffer_defs.h"  // IWYU pragma: keep
#include "nvim/mark_defs.h"  // IWYU pragma: keep
#include "nvim/normal_defs.h"  // IWYU pragma: keep
#include "nvim/pos_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

#include "normal_shim.h.generated.h"

// Hand-written prototype: grammar skips unsigned int return types.
unsigned int nvim_get_ve_flags(void);
