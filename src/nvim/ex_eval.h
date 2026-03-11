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
void exception_state_save(exception_state_T *estate);
void exception_state_restore(exception_state_T *estate);
void exception_state_clear(void);
bool has_loop_cmd(char *p);
void ex_endfunction(exarg_T *eap);
void free_global_msglist(void);
void report_make_pending(int pending, void *value);
