#pragma once

#include "nvim/api/private/defs.h"  // IWYU pragma: keep
#include "nvim/buffer_defs.h"  // IWYU pragma: keep
#include "nvim/macros_defs.h"

/// NW -> 0
/// NE -> kFloatAnchorEast
/// SW -> kFloatAnchorSouth
/// SE -> kFloatAnchorSouth | kFloatAnchorEast
EXTERN const char *const float_anchor_str[] INIT( = { "NW", "NE", "SW", "SE" });

// These functions are exported directly from Rust (nvim-plines / nvim-window crates).
extern int win_border_height(win_T *wp);
extern int win_border_width(win_T *wp);
extern int win_float_valid(win_T *win);

#include "winfloat.h.generated.h"
