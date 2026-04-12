#include <assert.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>

#include "klib/kvec.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/api/vim.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/buffer_defs.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/grid_defs.h"
#include "nvim/macros_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/mouse.h"
#include "nvim/move.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/optionstr.h"
#include "nvim/pos_defs.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_defs.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"
#include "nvim/winfloat.h"

#include "winfloat.c.generated.h"
extern bool rs_parse_winhl_opt(const char *winhl, win_T *wp);
extern int rs_win_valid(win_T *win);
extern int rs_tabpage_win_valid(tabpage_T *tp, win_T *win);

// Rust FFI declarations (window wrappers removed)
extern void rs_win_append(win_T *after, win_T *wp, tabpage_T *tp);
extern void rs_win_remove(win_T *wp, tabpage_T *tp);
extern win_T *rs_lastwin_nofloating(void);
extern int rs_win_comp_pos(void);
extern tabpage_T *rs_win_find_tabpage(win_T *win);
extern void rs_win_remove_status_line(win_T *wp, int add_hsep);

// win_new_float: migrated to Rust (winfloat crate, Phase 6).
extern win_T *rs_win_new_float(win_T *wp, bool last, WinConfig *fconfig, Error *err);
win_T *win_new_float(win_T *wp, bool last, WinConfig fconfig, Error *err)
{
  return rs_win_new_float(wp, last, &fconfig, err);
}

// win_set_minimal_style: migrated to Rust (winfloat crate, Phase 4).

// win_border_height and win_border_width exported directly from Rust.

// win_config_float: migrated to Rust (winfloat crate, Phase 5).
extern void rs_win_config_float(win_T *wp, WinConfig *fconfig);
void win_config_float(win_T *wp, WinConfig fconfig)
{
  rs_win_config_float(wp, &fconfig);
}

// float_zindex_cmp, win_float_remove: migrated to Rust (winfloat crate, Phase 3).

// win_check_anchored_floats, win_float_update_statusline, win_float_anchor_laststatus,
// win_reconfig_floats: migrated to Rust (winfloat crate, Phase 1).

/// Return true if "win" is floating window in the current tab page.
///
// win_float_valid exported directly from Rust (returns int; callers cast to bool as needed).

// win_float_find_preview, win_float_find_altwin: migrated to Rust (winfloat crate, Phase 2).

// handle_error_and_cleanup: inlined into win_float_create Rust implementation (Phase 6).

// win_float_create: migrated to Rust (winfloat crate, Phase 6).
// win_float_create exported directly from Rust with same symbol name.
