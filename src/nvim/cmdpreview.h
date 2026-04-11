#pragma once

#include "nvim/types_defs.h"  // IWYU pragma: keep

// Implemented in Rust (src/nvim-rs/cmdline/src/preview.rs)
handle_T cmdpreview_get_bufnr(void);

#include "cmdpreview.h.generated.h"
