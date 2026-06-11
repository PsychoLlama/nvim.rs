#pragma once

#include "nvim/buffer_defs.h"  // IWYU pragma: keep
#include "nvim/cmdexpand_defs.h"  // IWYU pragma: keep
#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/option_defs.h"  // IWYU pragma: keep
#include "nvim/regexp_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

#include "option_shim.h.generated.h"

// Hand-written prototypes for functions with function-pointer parameters.
void nvim_callback_for_all_tab_windows(void (*callback)(win_T *));
void nvim_for_all_buffers(void (*callback)(buf_T *));
void nvim_for_all_windows_in_curtab(void (*callback)(win_T *, void *), void *ud);
