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

#include "window_shim.h.generated.h"
