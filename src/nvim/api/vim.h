#pragma once

#include <stdint.h>  // IWYU pragma: keep

#include "nvim/api/keysets_defs.h"  // IWYU pragma: keep
#include "nvim/api/private/defs.h"  // IWYU pragma: keep

#include "api/vim.h.generated.h"

// Hand-written prototypes for functions with function-pointer parameters.
void rs_for_each_buf_handle(void (*cb)(int, Array *), Array *ud);
void rs_for_each_win_handle(void (*cb)(int, Array *), Array *ud);
void rs_for_each_tab_handle(void (*cb)(int, Array *), Array *ud);
