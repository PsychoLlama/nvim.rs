#pragma once

#include "nvim/buffer_defs.h"  // IWYU pragma: keep
#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/garray_defs.h"  // IWYU pragma: keep
#include "nvim/pos_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

#include "session_shim.h.generated.h"

// Hand-written prototypes: grammar skips function-pointer params.
int nvim_ses_foreach_session_global(
    int (*cb)(const char *key, int var_type, const char *escaped_val,
              double float_val, int float_sign, void *ud),
    void *ud);
int nvim_ses_foreach_buffer(
    int (*cb)(buf_T *buf, bool only_save_windows, void *ud),
    bool only_save_windows,
    void *ud);
