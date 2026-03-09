#pragma once

#include <stdbool.h>

#include "nvim/buffer_defs.h"
#include "nvim/fold_defs.h"
#include "nvim/macros_defs.h"
#include "nvim/pos_defs.h"

/// flags for update_screen()
/// The higher the value, the higher the priority
enum {
  UPD_VALID        = 10,  ///< buffer not changed, or changes marked with b_mod_*
  UPD_INVERTED     = 20,  ///< redisplay inverted part that changed
  UPD_INVERTED_ALL = 25,  ///< redisplay whole inverted part
  UPD_REDRAW_TOP   = 30,  ///< display first w_upd_rows screen lines
  UPD_SOME_VALID   = 35,  ///< like UPD_NOT_VALID but may scroll
  UPD_NOT_VALID    = 40,  ///< buffer needs complete redraw
  UPD_CLEAR        = 50,  ///< screen messed up, clear it
};

/// While redrawing the screen this flag is set.  It means the screen size
/// ('lines' and 'rows') must not be changed.
EXTERN bool updating_screen INIT( = false);

/// While computing a statusline and the like we do not want any w_redr_type or
/// must_redraw to be set.
EXTERN bool redraw_not_allowed INIT( = false);

/// used for 'hlsearch' highlight matching
EXTERN match_T screen_search_hl INIT( = { 0 });

/// last lnum where CurSearch was displayed
EXTERN linenr_T search_hl_has_cursor_lnum INIT( = 0);

#define W_ENDCOL(wp)   ((wp)->w_wincol + (wp)->w_width)
#define W_ENDROW(wp)   ((wp)->w_winrow + (wp)->w_height)

// Declarations for functions now exported directly from Rust (via export_name).
// These were previously thin C wrappers calling rs_* functions.
DLLEXPORT void conceal_check_cursor_line(void);
DLLEXPORT void check_screensize(void);
DLLEXPORT bool redrawing(void);
DLLEXPORT void start_search_hl(void);
DLLEXPORT void end_search_hl(void);
DLLEXPORT void setcursor(void);
DLLEXPORT void setcursor_mayforce(win_T *wp, bool force);
DLLEXPORT bool redraw_custom_title_later(void);
DLLEXPORT bool skip_showmode(void);
DLLEXPORT void unshowmode(bool force);
DLLEXPORT void comp_col(void);
DLLEXPORT void status_redraw_all(void);
DLLEXPORT void status_redraw_curbuf(void);
DLLEXPORT void status_redraw_buf(buf_T *buf);
DLLEXPORT void redraw_statuslines(void);
DLLEXPORT void win_redraw_last_status(const frame_T *frp);
DLLEXPORT bool conceal_cursor_line(const win_T *wp);
DLLEXPORT bool win_cursorline_standout(const win_T *wp);
DLLEXPORT void win_update_cursorline(win_T *wp, foldinfo_T *foldinfo);
DLLEXPORT int compute_foldcolumn(win_T *wp, int col);
DLLEXPORT int number_width(win_T *wp);
// Phase 2: redraw scheduling functions now exported from Rust
DLLEXPORT void redraw_later(win_T *wp, int type);
DLLEXPORT void redraw_all_later(int type);
DLLEXPORT void set_must_redraw(int type);
DLLEXPORT void screen_invalidate_highlights(void);
DLLEXPORT void redraw_curbuf_later(int type);
DLLEXPORT void redraw_buf_later(buf_T *buf, int type);
DLLEXPORT void redraw_buf_line_later(buf_T *buf, linenr_T line, bool force);
DLLEXPORT void redraw_win_range_later(win_T *wp, linenr_T first, linenr_T last);
DLLEXPORT void redrawWinline(win_T *wp, linenr_T lnum);
DLLEXPORT void redraw_buf_range_later(buf_T *buf, linenr_T first, linenr_T last);
DLLEXPORT void redraw_buf_status_later(buf_T *buf);

#include "drawscreen.h.generated.h"
