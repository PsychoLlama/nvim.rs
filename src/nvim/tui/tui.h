#pragma once

#include "nvim/highlight_defs.h"  // IWYU pragma: keep
#include "nvim/tui/tui_defs.h"  // IWYU pragma: keep
#include "nvim/ugrid.h"  // IWYU pragma: keep
#include "nvim/ui_defs.h"  // IWYU pragma: keep

// Functions implemented in Rust (nvim-tui crate)
extern bool tui_is_stopped(TUIData *tui);

#include "tui/tui.h.generated.h"
