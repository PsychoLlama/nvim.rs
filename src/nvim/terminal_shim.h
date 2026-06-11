#pragma once

#include "nvim/buffer_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

#include "terminal_shim.h.generated.h"

// Hand-written prototype for function with function-pointer parameter.
void nvim_terminal_foreach_invalidated(void (*fn)(void *term, void *ctx), void *ctx);
