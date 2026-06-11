#pragma once

#include "nvim/highlight_defs.h"  // IWYU pragma: keep
#include "nvim/tui/tui_defs.h"  // IWYU pragma: keep
#include "nvim/ugrid.h"  // IWYU pragma: keep
#include "nvim/ui_defs.h"  // IWYU pragma: keep

// Functions implemented in Rust (nvim-tui crate)
extern bool tui_is_stopped(TUIData *tui);

#include "tui/tui.h.generated.h"

// Hand-written prototype for function with function-pointer parameter.
void nvim_tui_set_primary_device_attr_cb(TUIData *tui, void (*cb)(TUIData *));
