#pragma once

#include <stdbool.h>

#include "nvim/register_defs.h"

// These functions are exported directly from Rust (nvim-clipboard crate).
extern yankreg_T *adjust_clipboard_name(int *name, bool quiet, bool writing);
extern bool get_clipboard(int name, yankreg_T **target, bool quiet);
extern void set_clipboard(int name, yankreg_T *reg);
extern void start_batch_changes(void);
extern void end_batch_changes(void);
extern int save_batch_count(void);
extern void restore_batch_count(int save_count);

#include "clipboard.h.generated.h"
