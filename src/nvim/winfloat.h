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

// These functions are exported from Rust (nvim-winfloat crate, Phase 1-2).
extern void win_check_anchored_floats(win_T *win);
extern void win_float_update_statusline(void);
extern void win_float_anchor_laststatus(void);
extern void win_reconfig_floats(void);
extern win_T *win_float_find_preview(void);
extern win_T *win_float_find_altwin(const win_T *win, const tabpage_T *tp);

#include "winfloat.h.generated.h"
