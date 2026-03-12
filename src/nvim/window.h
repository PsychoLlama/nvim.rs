#pragma once

#include <stdbool.h>

#include "nvim/buffer_defs.h"  // IWYU pragma: keep
#include "nvim/garray_defs.h"  // IWYU pragma: keep
#include "nvim/macros_defs.h"
#include "nvim/option_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

/// arguments for win_split()
enum {
  WSP_ROOM    = 0x01,   ///< require enough room
  WSP_VERT    = 0x02,   ///< split/equalize vertically
  WSP_HOR     = 0x04,   ///< equalize horizontally
  WSP_TOP     = 0x08,   ///< window at top-left of shell
  WSP_BOT     = 0x10,   ///< window at bottom-right of shell
  WSP_HELP    = 0x20,   ///< creating the help window
  WSP_BELOW   = 0x40,   ///< put new window below/right
  WSP_ABOVE   = 0x80,   ///< put new window above/left
  WSP_NEWLOC  = 0x100,  ///< don't copy location list
  WSP_NOENTER = 0x200,  ///< don't enter the new window
};

enum {
  MIN_COLUMNS = 12,   ///< minimal columns for screen
  MIN_LINES   = 2,    ///< minimal lines for screen
  STATUS_HEIGHT = 1,  ///< height of a status line under a window
};

enum {
  /// Lowest number used for window ID. Cannot have this many windows per tab.
  LOWEST_WIN_ID = 1000,
};

EXTERN int tabpage_move_disallowed INIT( = 0);  ///< moving tabpages around disallowed

// Bulk snapshot structs used by window_shim accessor functions.
// Must be defined before window_shim.h.generated.h which references them.
typedef struct {
  int topline;
  int topfill;
  int leftcol;
  int skipcol;
  int width;
  int height;
} WinSnapshot;

typedef struct {
  int32_t topline;
  int32_t botline;
  int32_t topfill;
  int32_t skipcol;
} WinViewportSnapshot;

// Declarations for Rust-exported window functions (Phase 15).
// These replace the auto-generated declarations that were lost when the
// corresponding C thin wrappers were deleted.

// Phase 15.1: Core window operations
win_T *prevwin_curwin(void);
int check_split_disallowed(const win_T *wp);
int win_split(int size, int flags);
win_T *win_split_ins(int size, int flags, win_T *new_wp, int dir, frame_T *to_flatten);
void win_init(win_T *newp, win_T *oldp, int flags);
int win_splitmove(win_T *wp, int size, int flags);
void win_move_after(win_T *win1, win_T *win2);
void leaving_window(win_T *win);
void entering_window(win_T *win);
void win_init_empty(win_T *wp);
void curwin_init(void);
win_T *winframe_remove(win_T *win, int *dirp, tabpage_T *tp, frame_T **unflat_altfr);
win_T *winframe_find_altwin(win_T *win, int *dirp, tabpage_T *tp, frame_T **altfr);
void winframe_restore(win_T *wp, int dir, frame_T *unflat_altfr);
void unuse_tabpage(tabpage_T *tp);
void use_tabpage(tabpage_T *tp);
void win_alloc_first(void);
void win_alloc_aucmd_win(int idx);
void win_init_size(void);
void free_tabpage(tabpage_T *tp);
void win_goto(win_T *wp);
void free_wininfo(WinInfo *wip, buf_T *bp);
void win_free(win_T *wp, tabpage_T *tp);
void win_comp_scroll(win_T *wp);
void restore_snapshot(int idx, int close_curwin);

// Phase 15.2: Tabpage and navigation
int win_new_tabpage(int after, char *filename);
int make_tabpages(int maxcount);
void close_tabpage(tabpage_T *tp);
void goto_tabpage(int n);
void goto_tabpage_win(tabpage_T *tp, win_T *wp);
void tabpage_move(int nr);
void win_fix_current_dir(void);
win_T *buf_jump_open_win(buf_T *buf);
win_T *buf_jump_open_tab(buf_T *buf);
win_T *swbuf_goto_win_with_buf(buf_T *buf);
void command_height(void);

// Phase 15.3: Screen/scroll/UI
void win_new_screensize(void);
void win_new_screen_rows(void);
void win_new_screen_cols(void);
void snapshot_windows_scroll_size(void);
void may_make_initial_scroll_size_snapshot(void);
void may_trigger_win_scrolled_resized(void);
void scroll_to_fraction(win_T *wp, int prev_height);
const char *check_colorcolumn(char *cc, win_T *wp);
void close_others(int message, int forceit);
void ui_ext_win_viewport(win_T *wp);

// Phase 17+: Rust-exported functions (via #[export_name])
void frame_new_height(frame_T *topfrp, int height, bool topfirst, bool wfh, bool set_ch);
void win_set_inner_size(win_T *wp, bool valid_cursor);
void win_fix_scroll(bool resize);
void set_winbar(bool make_room);
void close_windows(buf_T *buf, bool keep_curwin);
void win_ui_flush(bool validate);
bool can_close_in_cmdwin(win_T *win, Error *err);
bool check_can_set_curbuf_disabled(void);
bool check_can_set_curbuf_forceit(int forceit);
bool check_split_disallowed_err(const win_T *wp, Error *err);
void ui_ext_win_position(win_T *wp, bool validate);
void win_free_all(void);
bool goto_tabpage_lastused(void);

#include "window_shim.h.generated.h"
