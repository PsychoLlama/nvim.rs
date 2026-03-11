#pragma once

#include <stdbool.h>

#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/ex_eval_defs.h"  // IWYU pragma: keep

#include "ex_eval.h.generated.h"

// Functions implemented in Rust (src/nvim-rs/ex_eval/)
bool aborting(void);
bool should_abort(int retcode);
bool aborted_in_try(void);
void update_force_abort(void);
