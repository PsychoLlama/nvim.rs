#include <assert.h>
#include <inttypes.h>
#include <limits.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>

#include "klib/kvec.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/arglist.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cursor.h"
#include "nvim/decoration.h"
#include "nvim/diff.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/eval/window.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds2.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_eval.h"
#include "nvim/ex_getln.h"
#include "nvim/file_search.h"
#include "nvim/fileio.h"
#include "nvim/fold.h"
#include "nvim/garray.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/grid_defs.h"
#include "nvim/hashtab.h"
#include "nvim/highlight.h"
#include "nvim/keycodes.h"
#include "nvim/macros_defs.h"
#include "nvim/main.h"
#include "nvim/map_defs.h"
#include "nvim/mapping.h"
#include "nvim/mark.h"
#include "nvim/mark_defs.h"
#include "nvim/match.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/mouse.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/os/fs.h"
#include "nvim/os/os_defs.h"
#include "nvim/path.h"
#include "nvim/plines.h"
#include "nvim/popupmenu.h"
#include "nvim/pos_defs.h"
#include "nvim/quickfix.h"
#include "nvim/search.h"
#include "nvim/state.h"
#include "nvim/state_defs.h"
#include "nvim/statusline.h"
#include "nvim/strings.h"
#include "nvim/syntax.h"
#include "nvim/tag.h"
#include "nvim/terminal.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_compositor.h"
#include "nvim/ui_defs.h"
#include "nvim/undo.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"
#include "nvim/winfloat.h"

#include "window.c.generated.h"

extern int rs_win_locked(win_T *wp);
extern int rs_win_valid(win_T *win);
extern int rs_tabpage_win_valid(tabpage_T *tp, win_T *win);
extern int rs_only_one_window(void);
extern int rs_one_window(void);
extern int rs_win_valid_any_tab(win_T *win);
extern int rs_valid_tabpage(tabpage_T *tpc);
extern int rs_one_tabpage(void);
extern int rs_one_window_in_tab(win_T *win, tabpage_T *tp);
extern int rs_last_window(win_T *win);
extern int rs_win_count(void);
extern int rs_tabpage_index(tabpage_T *ftp);
extern int rs_valid_tabpage_win(tabpage_T *tpc);
extern int rs_frame_has_win(frame_T *frp, win_T *wp);
extern int rs_frame_fixed_height(frame_T *frp);
extern int rs_frame_fixed_width(frame_T *frp);
extern int rs_is_bottom_win(win_T *wp);
extern int rs_frame_check_height(frame_T *topfrp, int height);
extern int rs_frame_check_width(frame_T *topfrp, int width);
extern win_T *rs_win_find_by_handle(int handle);
extern tabpage_T *rs_win_find_tabpage(win_T *win);
extern tabpage_T *rs_find_tabpage(int n);
extern int rs_get_last_winid(void);
extern win_T *rs_lastwin_nofloating(void);
extern win_T *rs_frame2win(frame_T *frp);
extern frame_T *rs_win_altframe(win_T *win);

// Result structure from rs_winframe_find_altwin
typedef struct {
  frame_T *altfr;
  int dir;
} WinframeResult;
extern WinframeResult rs_winframe_find_altwin(win_T *wp, frame_T *altfr_initial);

extern int rs_frame_minheight(frame_T *topfrp, win_T *next_curwin);
extern int rs_frame_minwidth(frame_T *topfrp, win_T *next_curwin);
extern int rs_win_comp_pos(void);
extern void rs_frame_comp_pos(frame_T *topfrp, int *row, int *col);
extern void rs_frame_setheight(frame_T *curfrp, int height);
extern void rs_frame_setwidth(frame_T *curfrp, int width);
extern void rs_win_setheight_win(int height, win_T *win);
extern void rs_win_setwidth_win(int width, win_T *wp);
extern void rs_frame_new_height(frame_T *topfrp, int height, int topfirst, int wfh, int set_ch);
extern void rs_frame_new_width(frame_T *topfrp, int width, int leftfirst, int wfw);
extern void rs_frame_add_height(frame_T *frp, int n);
extern void rs_frame_add_statusline(frame_T *frp);
extern void rs_frame_set_vsep(const frame_T *frp, int add);
extern void rs_frame_add_hsep(const frame_T *frp);
extern void rs_frame_fix_width(win_T *wp);
extern void rs_frame_fix_height(win_T *wp);
extern win_T *rs_win_vert_neighbor(tabpage_T *tp, win_T *wp, int up, int count);
extern win_T *rs_win_horz_neighbor(tabpage_T *tp, win_T *wp, int left, int count);
extern void rs_frame_append(frame_T *after, frame_T *frp);
extern void rs_frame_insert(frame_T *before, frame_T *frp);
extern void rs_frame_remove(frame_T *frp);
extern void rs_win_append(win_T *after, win_T *wp, tabpage_T *tp);
extern void rs_win_remove(win_T *wp, tabpage_T *tp);

// Split helper functions from Rust
extern int rs_split_max_windows(int vertical);
extern int rs_split_iteration_size(int vertical, int todo);
extern int rs_split_make_windows_flags(int vertical);

// Close validation functions from Rust
extern int rs_close_can_close_floating(void);
extern int rs_close_count_nonfloating(tabpage_T *tp);
extern int rs_close_count_total(tabpage_T *tp);

// Frame tree operations from Rust (operations.rs)
extern void rs_frame_init(frame_T *frp);
extern void rs_frame_clear_links(frame_T *frp);
extern void rs_frame_copy_size(frame_T *dest, const frame_T *src);
extern frame_T *rs_frame_first_leaf(frame_T *frp);
extern frame_T *rs_frame_last_leaf(frame_T *frp);
extern frame_T *rs_frame_next_leaf(frame_T *frp);
extern frame_T *rs_frame_prev_leaf(frame_T *frp);
extern int rs_frame_count_leaves(const frame_T *frp);
extern int rs_frame_is_valid(const frame_T *frp);
extern int rs_frame_contains_win(const frame_T *frp, win_T *wp);
extern int rs_frame_children_width(const frame_T *frp);
extern int rs_frame_children_height(const frame_T *frp);
extern int rs_frame_max_child_width(const frame_T *frp);
extern int rs_frame_max_child_height(const frame_T *frp);
extern void rs_frame_propagate_size(frame_T *frp);

// Window navigation from Rust (navigate/movement.rs)
extern win_T *rs_nav_find_in_direction(int dir);
extern win_T *rs_nav_find_left(win_T *wp);
extern win_T *rs_nav_find_right(win_T *wp);
extern win_T *rs_nav_find_above(win_T *wp);
extern win_T *rs_nav_find_below(win_T *wp);
extern win_T *rs_nav_get_next(win_T *wp, int wrap);
extern win_T *rs_nav_get_prev(win_T *wp, int wrap);
extern win_T *rs_nav_get_next_nonfloat(win_T *wp, int wrap);
extern win_T *rs_nav_get_prev_nonfloat(win_T *wp, int wrap);
extern int rs_nav_is_horizontal_dir(int dir);
extern int rs_nav_is_vertical_dir(int dir);

extern void rs_diff_clear(tabpage_T *tp);

// Phase 1: Pure calculations and thin wrappers
extern void rs_set_fraction(win_T *wp);
extern int64_t rs_win_default_scroll(win_T *wp);
extern void rs_win_setheight(int height);
extern void rs_win_setwidth(int width);
extern int rs_min_rows(tabpage_T *tp);
extern int rs_min_rows_for_all_tabpages(void);
extern void rs_win_get_tabwin(int id, int *tabnr, int *winnr);

// Phase 2: Option validation + height/width setters
extern const char *rs_did_set_winminheight(void);
extern const char *rs_did_set_winminwidth(void);
extern void rs_win_new_height(win_T *wp, int height);
extern void rs_win_new_width(win_T *wp, int width);

// Phase 3: Snapshot lifecycle
extern void rs_clear_snapshot(tabpage_T *tp, int idx);
extern void rs_make_snapshot(int idx);
extern win_T *rs_get_snapshot_curwin(int idx);
extern int rs_check_snapshot_rec(frame_T *sn, frame_T *fr);
extern win_T *rs_restore_snapshot_rec(frame_T *sn, frame_T *fr);

// Phase 4: Size save/restore + cursor line validation
extern void rs_win_size_save(garray_T *gap);
extern void rs_win_size_restore(garray_T *gap);
extern void rs_check_lnums(int do_curwin);
extern void rs_check_lnums_nested(int do_curwin);
extern void rs_reset_lnums(void);

// Phase 5: Status line management
extern void rs_last_status(int morewin);
extern void rs_win_remove_status_line(win_T *wp, int add_hsep);
extern int rs_resize_frame_for_winbar(frame_T *fr);

// win_split_ins migration: Rust orchestrator
typedef struct {
  win_T *wp;           // new window or NULL
  int do_enter;        // whether to call win_enter_ext
  int enter_flags;     // WEE_* flags for win_enter_ext
  int vertical;        // 1 if vertical split
  int saved_option;    // saved p_wiw or p_wh value
} SplitInsResult;
extern SplitInsResult rs_win_split_ins(int size, int flags, win_T *new_wp, int dir,
                                       frame_T *to_flatten);

// win_close_othertab migration: Rust helpers
typedef struct {
  int free_tp_idx;     // tabpage index if removed (0 if not)
} TabRemoveResult;
extern int rs_close_othertab_validate(win_T *win, tabpage_T *tp, int force);
extern TabRemoveResult rs_close_othertab_remove_tabpage(win_T *win, tabpage_T *tp);
extern void rs_close_othertab_leave_open(win_T *win, int did_decrement, buf_T *bufref_buf,
                                         int bufref_valid);

// win_close migration: Rust helpers
typedef struct {
  win_T *wp;           // new curwin candidate (from win_free_mem or cursor transfer)
  int close_curwin;    // 1 if curwin was the closed window
  int was_floating;    // 1 if closed window was floating
  int dir;             // direction from win_free_mem ('v' or 'h')
} WinCloseStructResult;
extern int rs_win_close_validate(win_T *win, int free_buf, int force);
extern WinCloseStructResult rs_win_close_structural(win_T *win, int help_window,
                                                     frame_T *win_frame);
extern void rs_win_close_post_layout(int was_floating, int dir, frame_T *win_frame);

// do_window migration: Rust dispatcher
extern void rs_do_window(int nchar, int Prenum, int xchar);

// Accessor functions for Rust opaque handle pattern.
// These provide safe access to win_T fields from Rust code.

/// Get the w_locked field from a window (accessor for Rust).
int nvim_win_get_locked(win_T *wp)
{
  return wp->w_locked;
}

/// Get the w_floating field from a window (accessor for Rust).
int nvim_win_get_floating(win_T *wp)
{
  return wp->w_floating;
}

/// Get the w_p_pvw (preview window) field from a window (accessor for Rust).
int nvim_win_get_pvw(win_T *wp)
{
  return wp->w_p_pvw;
}

/// Get the w_ns_hl (highlight namespace) field from a window (accessor for Rust).
int nvim_win_get_ns_hl(win_T *wp)
{
  return wp->w_ns_hl;
}

/// Get the w_hl_attr_normal field from a window (accessor for Rust).
int nvim_win_get_hl_attr_normal(win_T *wp)
{
  return wp->w_hl_attr_normal;
}

/// Get the w_hl_attr_normalnc field from a window (accessor for Rust).
int nvim_win_get_hl_attr_normalnc(win_T *wp)
{
  return wp->w_hl_attr_normalnc;
}

// Accessors for update_window_hl (Phase 17)

/// Get the w_ns_hl_active field from a window.
int nvim_win_get_ns_hl_active(win_T *wp)
{
  return wp->w_ns_hl_active;
}

/// Set the w_ns_hl_active field of a window.
void nvim_win_set_ns_hl_active(win_T *wp, int val)
{
  wp->w_ns_hl_active = val;
}

/// Get the w_ns_hl_attr pointer from a window.
int *nvim_win_get_ns_hl_attr(win_T *wp)
{
  return wp->w_ns_hl_attr;
}

/// Set the w_ns_hl_attr pointer of a window.
void nvim_win_set_ns_hl_attr(win_T *wp, int *val)
{
  wp->w_ns_hl_attr = val;
}

/// Get the w_hl_needs_update field from a window.
bool nvim_win_get_hl_needs_update(win_T *wp)
{
  return wp->w_hl_needs_update;
}

/// Set the w_hl_needs_update field of a window.
void nvim_win_set_hl_needs_update(win_T *wp, bool val)
{
  wp->w_hl_needs_update = val;
}

/// Set the w_hl_attr_normal field of a window.
void nvim_win_set_hl_attr_normal(win_T *wp, int val)
{
  wp->w_hl_attr_normal = val;
}

/// Set the w_hl_attr_normalnc field of a window.
void nvim_win_set_hl_attr_normalnc(win_T *wp, int val)
{
  wp->w_hl_attr_normalnc = val;
}

/// Get the w_config.external field from a window.
bool nvim_win_get_config_external(win_T *wp)
{
  return wp->w_config.external;
}

/// Get the w_config.border flag from a window.
bool nvim_win_get_config_border(win_T *wp)
{
  return wp->w_config.border;
}

/// Get a border highlight ID from w_config.border_hl_ids.
int nvim_win_get_config_border_hl_id(win_T *wp, int idx)
{
  return wp->w_config.border_hl_ids[idx];
}

/// Set a border attribute in w_config.border_attr.
void nvim_win_set_config_border_attr(win_T *wp, int idx, int val)
{
  wp->w_config.border_attr[idx] = val;
}

/// Set the w_config.shadow field of a window.
void nvim_win_set_config_shadow(win_T *wp, bool val)
{
  wp->w_config.shadow = val;
}

/// Get the w_config.shadow field of a window.
bool nvim_win_get_config_shadow(win_T *wp)
{
  return wp->w_config.shadow;
}

/// Get the w_p_winbl field from a window.
int nvim_win_get_p_winbl(win_T *wp)
{
  return (int)wp->w_p_winbl;
}

/// Set the w_grid_alloc.blending field of a window.
void nvim_win_set_grid_blending(win_T *wp, bool val)
{
  wp->w_grid_alloc.blending = val;
}

/// Get the w_next field from a window (accessor for Rust).
win_T *nvim_win_get_next(win_T *wp)
{
  return wp->w_next;
}

/// Get the w_prev field from a window (accessor for Rust).
win_T *nvim_win_get_prev(win_T *wp)
{
  return wp->w_prev;
}

/// Set the w_next field of a window (accessor for Rust).
void nvim_win_set_next(win_T *wp, win_T *next)
{
  wp->w_next = next;
}

/// Set the w_prev field of a window (accessor for Rust).
void nvim_win_set_prev(win_T *wp, win_T *prev)
{
  wp->w_prev = prev;
}

/// Set the firstwin global variable (accessor for Rust).
/// Also syncs curtab->tp_firstwin if curtab is not NULL.
void nvim_set_firstwin(win_T *wp)
{
  firstwin = wp;
  if (curtab != NULL) {
    curtab->tp_firstwin = wp;
  }
}

/// Set the lastwin global variable (accessor for Rust).
/// Also syncs curtab->tp_lastwin if curtab is not NULL.
void nvim_set_lastwin(win_T *wp)
{
  lastwin = wp;
  if (curtab != NULL) {
    curtab->tp_lastwin = wp;
  }
}

/// Set the tp_firstwin field of a tabpage (accessor for Rust).
void nvim_tabpage_set_firstwin(tabpage_T *tp, win_T *wp)
{
  tp->tp_firstwin = wp;
}

/// Set the tp_lastwin field of a tabpage (accessor for Rust).
void nvim_tabpage_set_lastwin(tabpage_T *tp, win_T *wp)
{
  tp->tp_lastwin = wp;
}

/// Get the tp_lastwin field from a tabpage (accessor for Rust).
win_T *nvim_tabpage_get_lastwin(tabpage_T *tp)
{
  return tp->tp_lastwin;
}

// Global state accessors for Rust.
// These provide safe access to global variables from Rust code.

/// Get the current window (accessor for Rust).
win_T *nvim_get_curwin(void)
{
  return curwin;
}

/// Get the current window's grid_alloc (accessor for Rust).
ScreenGrid *nvim_get_curwin_grid_alloc(void)
{
  return curwin ? &curwin->w_grid_alloc : NULL;
}

/// Get the first window in the current tab (accessor for Rust).
win_T *nvim_get_firstwin(void)
{
  return firstwin;
}

/// Get the last window in the current tab (accessor for Rust).
win_T *nvim_get_lastwin(void)
{
  return lastwin;
}

/// Get the current window's handle (accessor for Rust).
int nvim_get_curwin_handle(void)
{
  return curwin->handle;
}

/// Get the current window's cursor position for incsearch state (accessor for Rust).
/// Fills the provided pos struct with lnum, col, coladd.
void nvim_get_curwin_cursor_pos(void *pos)
{
  // pos is a pointer to a struct with lnum (int32), col (int), coladd (int)
  int32_t *p = (int32_t *)pos;
  p[0] = (int32_t)curwin->w_cursor.lnum;
  p[1] = (int32_t)curwin->w_cursor.col;
  p[2] = (int32_t)curwin->w_cursor.coladd;
}

/// Save current window view state (accessor for Rust).
/// Fills the provided viewstate struct.
void nvim_save_viewstate(void *vs)
{
  // vs is a pointer to a struct matching viewstate_T layout
  int32_t *p = (int32_t *)vs;
  p[0] = (int32_t)curwin->w_curswant;
  p[1] = (int32_t)curwin->w_leftcol;
  p[2] = (int32_t)curwin->w_skipcol;
  p[3] = (int32_t)curwin->w_topline;
  p[4] = (int32_t)curwin->w_topfill;
  p[5] = (int32_t)curwin->w_botline;
  p[6] = (int32_t)curwin->w_empty_rows;
}

/// Restore current window view state (accessor for Rust).
void nvim_restore_viewstate(const void *vs)
{
  const int32_t *p = (const int32_t *)vs;
  curwin->w_curswant = (colnr_T)p[0];
  curwin->w_leftcol = (colnr_T)p[1];
  curwin->w_skipcol = (colnr_T)p[2];
  curwin->w_topline = (linenr_T)p[3];
  curwin->w_topfill = (int)p[4];
  curwin->w_botline = (linenr_T)p[5];
  curwin->w_empty_rows = (int)p[6];
}

/// Set the current window's cursor position (accessor for Rust).
void nvim_set_curwin_cursor_pos(const void *pos)
{
  const int32_t *p = (const int32_t *)pos;
  curwin->w_cursor.lnum = (linenr_T)p[0];
  curwin->w_cursor.col = (colnr_T)p[1];
  curwin->w_cursor.coladd = (colnr_T)p[2];
}

/// Compare two positions for equality (accessor for Rust).
int nvim_equalpos(const void *pos1, const void *pos2)
{
  const int32_t *p1 = (const int32_t *)pos1;
  const int32_t *p2 = (const int32_t *)pos2;
  return p1[0] == p2[0] && p1[1] == p2[1] && p1[2] == p2[2];
}

/// Validate cursor position (accessor for Rust).
void nvim_validate_cursor(void)
{
  validate_cursor(curwin);
}

/// Get the current buffer (accessor for Rust).
buf_T *nvim_get_curbuf(void)
{
  return curbuf;
}

/// Get the current tabpage (accessor for Rust).
tabpage_T *nvim_get_curtab(void)
{
  return curtab;
}

/// Get the tp_firstwin field from a tabpage (accessor for Rust).
win_T *nvim_tabpage_get_firstwin(tabpage_T *tp)
{
  return tp->tp_firstwin;
}

/// Get the tp_next field from a tabpage (accessor for Rust).
tabpage_T *nvim_tabpage_get_next(tabpage_T *tp)
{
  return tp->tp_next;
}

/// Get the first tabpage (accessor for Rust).
tabpage_T *nvim_get_first_tabpage(void)
{
  return first_tabpage;
}

/// Get the last used tabpage (accessor for Rust).
tabpage_T *nvim_get_lastused_tabpage(void)
{
  return lastused_tabpage;
}

/// Get the tp_curwin field from a tabpage (accessor for Rust).
win_T *nvim_tabpage_get_curwin(tabpage_T *tp)
{
  return tp->tp_curwin;
}

/// Get the handle field from a tabpage (accessor for Rust).
int nvim_tabpage_get_handle(tabpage_T *tp)
{
  return (int)tp->handle;
}

// Frame accessors for Rust opaque handle pattern.

/// Get the fr_layout field from a frame (accessor for Rust).
/// Returns FR_LEAF (0), FR_ROW (1), or FR_COL (2).
int nvim_frame_get_layout(frame_T *frp)
{
  return frp->fr_layout;
}

/// Get the fr_win field from a frame (accessor for Rust).
/// Only valid when fr_layout == FR_LEAF.
win_T *nvim_frame_get_win(frame_T *frp)
{
  return frp->fr_win;
}

/// Get the fr_child field from a frame (accessor for Rust).
/// First child frame (for FR_ROW or FR_COL layouts).
frame_T *nvim_frame_get_child(frame_T *frp)
{
  return frp->fr_child;
}

/// Get the fr_next field from a frame (accessor for Rust).
/// Next sibling frame in parent.
frame_T *nvim_frame_get_next(frame_T *frp)
{
  return frp->fr_next;
}

/// Get the fr_parent field from a frame (accessor for Rust).
/// Parent frame in the frame tree.
frame_T *nvim_frame_get_parent(frame_T *frp)
{
  return frp->fr_parent;
}

/// Get the w_frame field from a window (accessor for Rust).
/// The window's frame in the frame tree.
frame_T *nvim_win_get_frame(win_T *wp)
{
  return wp->w_frame;
}

/// Get fr_height field from a frame (accessor for Rust).
int nvim_frame_get_height(frame_T *frp)
{
  return frp->fr_height;
}

/// Get fr_width field from a frame (accessor for Rust).
int nvim_frame_get_width(frame_T *frp)
{
  return frp->fr_width;
}

/// Get w_p_wfh (winfixheight) from a window (accessor for Rust).
int nvim_win_get_wfh(win_T *wp)
{
  return wp->w_p_wfh;
}

/// Get w_p_wfw (winfixwidth) from a window (accessor for Rust).
int nvim_win_get_wfw(win_T *wp)
{
  return wp->w_p_wfw;
}

/// Get handle field from a window (accessor for Rust).
int nvim_win_get_handle(win_T *wp)
{
  return wp->handle;
}

// Accessors for fold.c migration (Phase: fold method checks)

/// Get a character from the foldmethod string at the given index.
/// Used by Rust foldmethodIs* functions to check fold method.
char nvim_win_get_fdm_char(win_T *wp, int idx)
{
  return wp->w_p_fdm[idx];
}

/// Get the w_p_fen (foldenable) field from a window.
int nvim_win_get_p_fen(win_T *wp)
{
  return wp->w_p_fen;
}

/// Check if window's buffer has a terminal.
int nvim_win_buf_has_terminal(win_T *wp)
{
  return wp->w_buffer->terminal != NULL;
}

/// Check if window's folds growarray is empty.
int nvim_win_folds_empty(win_T *wp)
{
  return GA_EMPTY(&wp->w_folds);
}

/// Get the w_valid field from a window.
int nvim_win_get_valid(win_T *wp)
{
  return wp->w_valid;
}

/// Set the w_valid field of a window.
void nvim_win_set_valid(win_T *wp, int val)
{
  wp->w_valid = val;
}

/// Clear specific bits from the w_valid field.
void nvim_win_clear_valid_bits(win_T *wp, int bits)
{
  wp->w_valid &= ~bits;
}

/// Set the w_lines_valid field (number of valid w_lines entries).
void nvim_win_set_lines_valid(win_T *wp, int val)
{
  wp->w_lines_valid = val;
}

// Accessors for plines.c migration (Phase 3: display calculations)

/// Get the w_view_width field from a window.
int nvim_win_get_view_width(win_T *wp)
{
  return wp->w_view_width;
}

/// Get the w_view_height field from a window.
int nvim_win_get_view_height(win_T *wp)
{
  return wp->w_view_height;
}

// Note: nvim_win_get_skipcol is defined later in window.c (returns colnr_T)

/// Get the w_buffer field from a window.
buf_T *nvim_win_get_w_buffer(win_T *wp)
{
  return wp->w_buffer;
}

/// Get the fold column count for a window.
int nvim_win_fdccol_count(win_T *wp)
{
  return win_fdccol_count(wp);
}

/// Check if the given window is the current window.
int nvim_win_is_curwin(win_T *wp)
{
  return wp == curwin;
}

/// Get the w_p_rnu (relativenumber) field from a window.
int nvim_win_get_p_rnu(win_T *wp)
{
  return wp->w_p_rnu;
}

/// Get the w_p_nu (number) field from a window.
int nvim_win_get_p_nu(win_T *wp)
{
  return wp->w_p_nu;
}

/// Get the w_p_nuw (numberwidth) field from a window.
OptInt nvim_win_get_p_nuw(win_T *wp)
{
  return wp->w_p_nuw;
}

/// Get the w_p_stc (statuscolumn) field from a window.
char *nvim_win_get_p_stc(win_T *wp)
{
  return wp->w_p_stc;
}

/// Get the w_p_cocu (concealcursor) field from a window.
char *nvim_win_get_p_cocu(win_T *wp)
{
  return wp->w_p_cocu;
}

/// Get the w_minscwidth field from a window.
int nvim_win_get_minscwidth(win_T *wp)
{
  return wp->w_minscwidth;
}

/// Get the w_nrwidth_line_count field from a window.
linenr_T nvim_win_get_nrwidth_line_count(win_T *wp)
{
  return wp->w_nrwidth_line_count;
}

/// Set the w_nrwidth_line_count field of a window.
void nvim_win_set_nrwidth_line_count(win_T *wp, linenr_T val)
{
  wp->w_nrwidth_line_count = val;
}

/// Get the w_nrwidth_width field from a window.
int nvim_win_get_nrwidth_width(win_T *wp)
{
  return wp->w_nrwidth_width;
}

/// Set the w_nrwidth_width field of a window.
void nvim_win_set_nrwidth_width(win_T *wp, int val)
{
  wp->w_nrwidth_width = val;
}

/// Set the w_statuscol_line_count field of a window.
void nvim_win_set_statuscol_line_count(win_T *wp, linenr_T val)
{
  wp->w_statuscol_line_count = val;
}

/// Get the buffer line count for a window.
linenr_T nvim_win_buf_line_count(win_T *wp)
{
  return wp->w_buffer->b_ml.ml_line_count;
}

/// Get the sign text meta total for a window's buffer.
int nvim_win_buf_meta_total_signtext(win_T *wp)
{
  return buf_meta_total(wp->w_buffer, kMTMetaSignText) > 0;
}

/// Get the global p_wmw value.
OptInt nvim_get_p_wmw(void)
{
  return p_wmw;
}

/// Get the global p_wh (winheight) value.
OptInt nvim_get_p_wh(void)
{
  return p_wh;
}

/// Get the global p_wmh (winminheight) value.
OptInt nvim_get_p_wmh(void)
{
  return p_wmh;
}

/// Get the global p_wiw (winwidth) value.
OptInt nvim_get_p_wiw(void)
{
  return p_wiw;
}

/// Get the global p_sb (splitbelow) value.
int nvim_get_p_sb(void)
{
  return p_sb ? 1 : 0;
}

/// Get the global p_spr (splitright) value.
int nvim_get_p_spr(void)
{
  return p_spr ? 1 : 0;
}

/// Get the global Rows value.
int nvim_get_Rows(void)
{
  return Rows;
}

/// Get the global Columns value.
int nvim_get_Columns(void)
{
  return Columns;
}

/// Get the w_height field from a window (raw field accessor).
int nvim_win_field_height(win_T *wp)
{
  return wp->w_height;
}

/// Set the w_height field of a window (raw field accessor).
void nvim_win_field_set_height(win_T *wp, int val)
{
  wp->w_height = val;
}

/// Set the w_hsep_height field of a window.
void nvim_win_set_hsep_height(win_T *wp, int val)
{
  wp->w_hsep_height = val;
}

/// Set the w_status_height field of a window.
void nvim_win_set_status_height(win_T *wp, int val)
{
  wp->w_status_height = val;
}

/// Get the w_width field from a window (raw field accessor).
int nvim_win_field_width(win_T *wp)
{
  return wp->w_width;
}

/// Set the w_width field of a window (raw field accessor).
void nvim_win_field_set_width(win_T *wp, int val)
{
  wp->w_width = val;
}

/// Set the w_vsep_width field of a window.
void nvim_win_set_vsep_width(win_T *wp, int val)
{
  wp->w_vsep_width = val;
}

/// Set the fr_height field of a frame.
void nvim_frame_set_height(frame_T *frp, int val)
{
  frp->fr_height = val;
}

/// Set the fr_width field of a frame.
void nvim_frame_set_width(frame_T *frp, int val)
{
  frp->fr_width = val;
}

/// Wrapper for win_new_height().
void nvim_win_new_height(win_T *wp, int height)
{
  win_new_height(wp, height);
}

/// Wrapper for win_new_width().
void nvim_win_new_width(win_T *wp, int width)
{
  win_new_width(wp, width);
}

/// Wrapper for frame_new_height().
void nvim_frame_new_height(frame_T *topfrp, int height, bool topfirst, bool wfh, bool set_ch)
{
  frame_new_height(topfrp, height, topfirst, wfh, set_ch);
}

/// Wrapper for frame_new_width().
void nvim_frame_new_width(frame_T *topfrp, int width, bool leftfirst, bool wfw)
{
  frame_new_width(topfrp, width, leftfirst, wfw);
}

/// Wrapper for win_config_float().
void nvim_win_config_float(win_T *wp)
{
  win_config_float(wp, wp->w_config);
}

/// Wrapper for win_fix_scroll().
void nvim_win_fix_scroll(bool upd_topline)
{
  win_fix_scroll(upd_topline);
}

/// Wrapper for redraw_all_later().
void nvim_redraw_all_later(int type)
{
  redraw_all_later(type);
}

/// Get w_config.height from a window.
int nvim_win_get_config_height(win_T *wp)
{
  return wp->w_config.height;
}

/// Set w_config.height on a window.
void nvim_win_set_config_height(win_T *wp, int val)
{
  wp->w_config.height = val;
}

/// Get w_config.width from a window.
int nvim_win_get_config_width(win_T *wp)
{
  return wp->w_config.width;
}

/// Set w_config.width on a window.
void nvim_win_set_config_width(win_T *wp, int val)
{
  wp->w_config.width = val;
}

/// Get the global p_ch (cmdheight) value.
int64_t nvim_get_window_p_ch(void)
{
  return p_ch;
}

/// Set the global redraw_cmdline flag.
void nvim_set_redraw_cmdline(bool val)
{
  redraw_cmdline = val;
}

/// Get the w_winbar_height field from a window.
int nvim_win_get_winbar_height(win_T *wp)
{
  return wp->w_winbar_height;
}

/// Get the w_status_height field from a window.
int nvim_win_get_status_height(win_T *wp)
{
  return wp->w_status_height;
}

/// Get the global State variable.
int nvim_get_State(void)
{
  return State;
}

/// Set the global State variable.
void nvim_set_State(int val)
{
  State = val;
}

/// Get the real state (calls get_real_state()).
int nvim_get_real_state(void)
{
  return get_real_state();
}

/// Get the 'pumheight' option value.
int64_t nvim_get_p_ph(void)
{
  return p_ph;
}

/// Get the 'pumwidth' option value.
int64_t nvim_get_p_pw(void)
{
  return p_pw;
}

/// Get the 'pummaxwidth' option value.
int64_t nvim_get_p_pmw(void)
{
  return p_pmw;
}

/// Get the 'diff' option value for a window.
int nvim_win_get_p_diff(win_T *wp)
{
  return wp->w_p_diff;
}

/// Get the 'cursorbind' option value for a window.
int nvim_win_get_p_crb(win_T *wp)
{
  return wp->w_p_crb;
}

/// Set the curwin global.
void nvim_set_curwin(win_T *wp)
{
  curwin = wp;
}

/// Set the curbuf global.
void nvim_set_curbuf(buf_T *buf)
{
  curbuf = buf;
}

/// Check if there are virtual lines in the window's buffer.
int nvim_win_buf_meta_total_lines(win_T *wp)
{
  return buf_meta_total(wp->w_buffer, kMTMetaLines) > 0;
}

/// Get diffopt filler state.
int nvim_diffopt_filler(void)
{
  return diffopt_filler();
}

/// Check if window is the command-line window.
int nvim_win_is_cmdwin(win_T *wp)
{
  return wp == cmdwin_win;
}

/// Get the signcolumn width for a window.
int nvim_win_get_scwidth(win_T *wp)
{
  return wp->w_scwidth;
}

/// Get the p_cpo global option string.
char *nvim_get_p_cpo(void)
{
  return p_cpo;
}

/// Get the window's showbreak option.
char *nvim_win_get_p_sbr(win_T *wp)
{
  return wp->w_p_sbr;
}

/// Get the global showbreak option.
char *nvim_get_p_sbr(void)
{
  return p_sbr;
}

/// Get the empty string option constant.
char *nvim_get_empty_string_option(void)
{
  return empty_string_option;
}

/// Get the window's 'list' option.
int nvim_win_get_p_list(win_T *wp)
{
  return wp->w_p_list;
}

/// Get the 'listchars' precedes character for window.
uint32_t nvim_win_get_lcs_prec(win_T *wp)
{
  return wp->w_p_lcs_chars.prec;
}

/// Get the window's 'cursorline' option.
int nvim_win_get_p_cul(win_T *wp)
{
  return wp->w_p_cul;
}

/// Get the window's 'conceallevel' option.
OptInt nvim_win_get_p_cole(win_T *wp)
{
  return wp->w_p_cole;
}

/// Get the window's 'scrolloff' option.
OptInt nvim_win_get_p_so(win_T *wp)
{
  return wp->w_p_so;
}

/// Get the window's 'sidescrolloff' option.
OptInt nvim_win_get_p_siso(win_T *wp)
{
  return wp->w_p_siso;
}

/// Get the global 'scrolloff' option.
OptInt nvim_get_p_so(void)
{
  return p_so;
}

/// Get the global 'sidescrolloff' option.
OptInt nvim_get_p_siso(void)
{
  return p_siso;
}

/// Check if the buffer in window is a terminal.
int nvim_win_buf_is_terminal(win_T *wp)
{
  return wp->w_buffer->terminal != NULL;
}

/// Get the global 'laststatus' option.
OptInt nvim_get_p_ls(void)
{
  return p_ls;
}

/// Check if global 'winbar' option is empty.
int nvim_get_p_wbr_empty(void)
{
  return *p_wbr == NUL;
}

/// Get the global 'showtabline' option.
OptInt nvim_get_p_stal(void)
{
  return p_stal;
}

/// Get the global winbar height (wrapper for Rust FFI).
int nvim_global_winbar_height(void)
{
  return global_winbar_height();
}

/// Get the global statusline height (wrapper for Rust FFI).
int nvim_global_stl_height(void)
{
  return global_stl_height();
}

/// Get the global 'eadirection' option (p_ead).
const char *nvim_get_p_ead(void)
{
  return p_ead;
}

/// Check if there is more than one tabpage.
int nvim_first_tabpage_has_next(void)
{
  return first_tabpage != NULL && first_tabpage->tp_next != NULL;
}

// Accessors for drawscreen.c migration (separator drawing)

/// Get the w_winrow field from a window.
int nvim_win_get_winrow(win_T *wp)
{
  return wp->w_winrow;
}

/// Get the w_wincol field from a window.
int nvim_win_get_wincol(win_T *wp)
{
  return wp->w_wincol;
}

/// Get the w_winrow_off field from a window.
int nvim_win_get_winrow_off(win_T *wp)
{
  return wp->w_winrow_off;
}

/// Get the w_wincol_off field from a window.
int nvim_win_get_wincol_off(win_T *wp)
{
  return wp->w_wincol_off;
}

/// Set the w_winrow field of a window.
void nvim_win_set_winrow(win_T *wp, int val)
{
  wp->w_winrow = val;
}

/// Set the w_wincol field of a window.
void nvim_win_set_wincol(win_T *wp, int val)
{
  wp->w_wincol = val;
}

/// Set the w_redr_status field of a window.
void nvim_win_set_redr_status(win_T *wp, int val)
{
  wp->w_redr_status = val;
}

/// Get the w_grid_alloc field from a window.
ScreenGrid *nvim_win_get_grid_alloc(win_T *wp)
{
  return &wp->w_grid_alloc;
}

/// Get the w_config.hide field from a window.
int nvim_win_get_config_hide(win_T *wp)
{
  return wp ? wp->w_config.hide : 0;
}

/// Set the w_pos_changed field of a window.
void nvim_win_set_pos_changed(win_T *wp, int val)
{
  wp->w_pos_changed = val;
}

/// Get the w_config.relative field from a window.
int nvim_win_get_config_relative(win_T *wp)
{
  return (int)wp->w_config.relative;
}

/// Get the w_config.window field (parent window handle).
int nvim_win_get_config_window(win_T *wp)
{
  return wp ? wp->w_config.window : 0;
}

/// Get the w_config.anchor field.
int nvim_win_get_config_anchor(win_T *wp)
{
  return wp ? (int)wp->w_config.anchor : 0;
}

/// Get the w_config.zindex field.
int nvim_win_get_config_zindex(win_T *wp)
{
  return wp ? wp->w_config.zindex : 50;  // Default zindex
}

/// Get the w_config.focusable field.
int nvim_win_get_config_focusable(win_T *wp)
{
  return wp ? wp->w_config.focusable : 0;
}

/// Get the topframe global.
frame_T *nvim_get_topframe(void)
{
  return topframe;
}

/// Get the w_width field from a window (internal accessor).
int nvim_win_get_w_width(win_T *wp)
{
  return wp->w_width;
}

/// Get the w_height field from a window (internal accessor).
int nvim_win_get_w_height(win_T *wp)
{
  return wp->w_height;
}

/// Get the w_hsep_height field from a window.
int nvim_win_get_hsep_height(win_T *wp)
{
  return wp->w_hsep_height;
}

/// Get the w_vsep_width field from a window.
int nvim_win_get_vsep_width(win_T *wp)
{
  return wp->w_vsep_width;
}

/// Get the w_wcol field from a window (cursor column in window).
int nvim_win_get_wcol(win_T *wp)
{
  return wp->w_wcol;
}

/// Set the w_wcol field in a window (cursor column in window).
void nvim_win_set_wcol(win_T *wp, int val)
{
  wp->w_wcol = val;
}

/// Get the w_wrow field from a window (cursor row in window).
int nvim_win_get_wrow(win_T *wp)
{
  return wp->w_wrow;
}

/// Set the w_wrow field for a window (accessor for Rust).
void nvim_win_set_wrow(win_T *wp, int val)
{
  if (wp) {
    wp->w_wrow = val;
  }
}

/// Get the w_p_sms (smoothscroll) option for a window (accessor for Rust).
int nvim_win_get_p_sms(win_T *wp)
{
  return wp ? wp->w_p_sms : 0;
}

/// Set the w_p_sms (smoothscroll) option for a window (accessor for Rust).
void nvim_win_set_p_sms(win_T *wp, int val)
{
  if (wp) {
    wp->w_p_sms = val;
  }
}

/// Get the tp_topframe field from a tabpage (accessor for Rust).
frame_T *nvim_tabpage_get_topframe(tabpage_T *tp)
{
  return tp->tp_topframe;
}

/// Get the prevwin global (accessor for Rust).
win_T *nvim_get_prevwin(void)
{
  return prevwin;
}

/// Get W_ENDROW(wp) - the row after the window content.
int nvim_win_get_endrow(win_T *wp)
{
  return W_ENDROW(wp);
}

// Statusline accessors for Rust statusline crate
char *nvim_win_get_p_stl(win_T *wp)
{
  return wp ? wp->w_p_stl : NULL;
}

StlClickDefinition *nvim_win_get_status_click_defs(win_T *wp)
{
  return wp ? wp->w_status_click_defs : NULL;
}

size_t nvim_win_get_status_click_defs_size(win_T *wp)
{
  return wp ? wp->w_status_click_defs_size : 0;
}

void nvim_win_set_status_click_defs(win_T *wp, StlClickDefinition *cd)
{
  if (wp) {
    wp->w_status_click_defs = cd;
  }
}

void nvim_win_set_status_click_defs_size(win_T *wp, size_t sz)
{
  if (wp) {
    wp->w_status_click_defs_size = sz;
  }
}

StlClickDefinition *nvim_win_get_winbar_click_defs(win_T *wp)
{
  return wp ? wp->w_winbar_click_defs : NULL;
}

size_t nvim_win_get_winbar_click_defs_size(win_T *wp)
{
  return wp ? wp->w_winbar_click_defs_size : 0;
}

int nvim_win_get_redr_status(win_T *wp)
{
  return wp ? wp->w_redr_status : 0;
}

// Global statusline/tabline accessors
char *nvim_get_p_tal(void)
{
  return p_tal;
}

char *nvim_get_p_stl(void)
{
  return p_stl;
}

StlClickDefinition *nvim_get_tab_page_click_defs(void)
{
  return tab_page_click_defs;
}

size_t nvim_get_tab_page_click_defs_size(void)
{
  return tab_page_click_defs_size;
}

void nvim_set_tab_page_click_defs(StlClickDefinition *cd)
{
  tab_page_click_defs = cd;
}

void nvim_set_tab_page_click_defs_size(size_t sz)
{
  tab_page_click_defs_size = sz;
}

/// Get W_ENDCOL(wp) - the column after the window content.
int nvim_win_get_endcol(win_T *wp)
{
  return W_ENDCOL(wp);
}

/// Get the vertical separator character (fcs_chars.vert).
schar_T nvim_win_get_fcs_vert(win_T *wp)
{
  return wp->w_p_fcs_chars.vert;
}

/// Get the horizontal separator character (fcs_chars.horiz).
schar_T nvim_win_get_fcs_horiz(win_T *wp)
{
  return wp->w_p_fcs_chars.horiz;
}

/// Get the vertical+horizontal connector character (fcs_chars.verthoriz).
schar_T nvim_win_get_fcs_verthoriz(win_T *wp)
{
  return wp->w_p_fcs_chars.verthoriz;
}

/// Get the vertical-right connector character (fcs_chars.vertright).
schar_T nvim_win_get_fcs_vertright(win_T *wp)
{
  return wp->w_p_fcs_chars.vertright;
}

/// Get the vertical-left connector character (fcs_chars.vertleft).
schar_T nvim_win_get_fcs_vertleft(win_T *wp)
{
  return wp->w_p_fcs_chars.vertleft;
}

/// Get the horizontal-down connector character (fcs_chars.horizdown).
schar_T nvim_win_get_fcs_horizdown(win_T *wp)
{
  return wp->w_p_fcs_chars.horizdown;
}

/// Get the horizontal-up connector character (fcs_chars.horizup).
schar_T nvim_win_get_fcs_horizup(win_T *wp)
{
  return wp->w_p_fcs_chars.horizup;
}

/// Get the statusline fill character (fcs_chars.stl).
schar_T nvim_win_get_fcs_stl(win_T *wp)
{
  return wp->w_p_fcs_chars.stl;
}

/// Get the statusline fill character for non-current windows (fcs_chars.stlnc).
schar_T nvim_win_get_fcs_stlnc(win_T *wp)
{
  return wp->w_p_fcs_chars.stlnc;
}

/// Get the fold closed character (fcs_chars.foldclosed).
schar_T nvim_win_get_fcs_foldclosed(win_T *wp)
{
  return wp->w_p_fcs_chars.foldclosed;
}

/// Get the fold open character (fcs_chars.foldopen).
schar_T nvim_win_get_fcs_foldopen(win_T *wp)
{
  return wp->w_p_fcs_chars.foldopen;
}

/// Get the fold separator character (fcs_chars.foldsep).
schar_T nvim_win_get_fcs_foldsep(win_T *wp)
{
  return wp->w_p_fcs_chars.foldsep;
}

/// Get the fold inner character (fcs_chars.foldinner).
schar_T nvim_win_get_fcs_foldinner(win_T *wp)
{
  return wp->w_p_fcs_chars.foldinner;
}

/// Get the diff character (fcs_chars.diff).
schar_T nvim_win_get_fcs_diff(win_T *wp)
{
  return wp->w_p_fcs_chars.diff;
}

/// Get the listchars extends character (lcs_chars.ext).
schar_T nvim_win_get_lcs_ext(win_T *wp)
{
  return wp->w_p_lcs_chars.ext;
}

/// Get the window's wrap flags (w_p_wrap_flags).
int nvim_win_get_wrap_flags(win_T *wp)
{
  return wp->w_p_wrap_flags;
}

/// Get the window's 'wrap' option.
int nvim_win_get_p_wrap(win_T *wp)
{
  return wp->w_p_wrap;
}

/// Get the window's virtual column (w_virtcol).
colnr_T nvim_win_get_virtcol(win_T *wp)
{
  return wp->w_virtcol;
}

/// Set the window's virtual column (w_virtcol).
void nvim_win_set_virtcol(win_T *wp, colnr_T val)
{
  if (wp) {
    wp->w_virtcol = val;
  }
}

/// Get the 'cursorcolumn' option value for a window.
int nvim_win_get_p_cuc(win_T *wp)
{
  return wp->w_p_cuc;
}

/// Get the window's cursorline line number (w_cursorline).
linenr_T nvim_win_get_cursorline(win_T *wp)
{
  return wp->w_cursorline;
}

/// Get the window's cursorlineopt flags (w_p_culopt_flags).
int nvim_win_get_p_culopt_flags(win_T *wp)
{
  return wp->w_p_culopt_flags;
}

/// Get the cursor line number for the window.
linenr_T nvim_win_get_cursor_lnum(win_T *wp)
{
  return wp->w_cursor.lnum;
}

/// Get the window's topline.
linenr_T nvim_win_get_topline(win_T *wp)
{
  return wp->w_topline;
}

/// Set the window's topline.
void nvim_win_set_topline(win_T *wp, linenr_T val)
{
  wp->w_topline = val;
}

/// Get the window's topline_was_set flag.
int nvim_win_get_topline_was_set(win_T *wp)
{
  return wp ? wp->w_topline_was_set : 0;
}

/// Set the window's topline_was_set flag.
void nvim_win_set_topline_was_set(win_T *wp, int val)
{
  if (wp) {
    wp->w_topline_was_set = val != 0;
  }
}

/// Get the window's botline (line below the bottom of the window).
linenr_T nvim_win_get_botline(win_T *wp)
{
  return wp->w_botline;
}

/// Get the window's redraw type.
int nvim_win_get_redr_type(win_T *wp)
{
  return wp ? wp->w_redr_type : 0;
}

/// Set the window's redraw type.
void nvim_win_set_redr_type(win_T *wp, int val)
{
  if (wp) {
    wp->w_redr_type = val;
  }
}

/// Get the number of valid w_lines entries.
int nvim_win_get_lines_valid(win_T *wp)
{
  return wp ? wp->w_lines_valid : 0;
}

// NOTE: nvim_win_set_lines_valid already defined earlier in this file

/// Get the top line of the redraw range.
linenr_T nvim_win_get_redraw_top(win_T *wp)
{
  return wp ? wp->w_redraw_top : 0;
}

/// Set the top line of the redraw range.
void nvim_win_set_redraw_top(win_T *wp, linenr_T val)
{
  if (wp) {
    wp->w_redraw_top = val;
  }
}

/// Get the bottom line of the redraw range.
linenr_T nvim_win_get_redraw_bot(win_T *wp)
{
  return wp ? wp->w_redraw_bot : 0;
}

/// Set the bottom line of the redraw range.
void nvim_win_set_redraw_bot(win_T *wp, linenr_T val)
{
  if (wp) {
    wp->w_redraw_bot = val;
  }
}

/// Get the window's topfill (filler lines above topline).
int nvim_win_get_topfill(win_T *wp)
{
  return wp->w_topfill;
}

/// Set the window's topfill (filler lines above topline).
void nvim_win_set_topfill(win_T *wp, int val)
{
  wp->w_topfill = val;
}

/// Get the window's arg_idx (argument list index).
int nvim_win_get_arg_idx(win_T *wp)
{
  return wp->w_arg_idx;
}

/// Check if the window's arg_idx is invalid.
int nvim_win_get_arg_idx_invalid(win_T *wp)
{
  return wp->w_arg_idx_invalid;
}

/// Get the argument count for a window's argument list.
int nvim_win_argcount(win_T *wp)
{
  return WARGCOUNT(wp);
}

/// Get the window's skipcol.
colnr_T nvim_win_get_skipcol(win_T *wp)
{
  return wp->w_skipcol;
}

/// Set the window's skipcol.
void nvim_win_set_skipcol(win_T *wp, colnr_T val)
{
  wp->w_skipcol = val;
}

/// Get the cursor column.
colnr_T nvim_win_get_cursor_col(win_T *wp)
{
  return wp->w_cursor.col;
}

/// Set the cursor line number.
void nvim_win_set_cursor_lnum(win_T *wp, linenr_T lnum)
{
  wp->w_cursor.lnum = lnum;
}

/// Set the cursor column.
void nvim_win_set_cursor_col(win_T *wp, colnr_T col)
{
  wp->w_cursor.col = col;
}

/// Get the cursor coladd.
colnr_T nvim_win_get_cursor_coladd(win_T *wp)
{
  return wp->w_cursor.coladd;
}

/// Get the valid cursor line number.
linenr_T nvim_win_get_valid_cursor_lnum(win_T *wp)
{
  return wp->w_valid_cursor.lnum;
}

/// Get the valid cursor column.
colnr_T nvim_win_get_valid_cursor_col(win_T *wp)
{
  return wp->w_valid_cursor.col;
}

/// Get the valid cursor coladd.
colnr_T nvim_win_get_valid_cursor_coladd(win_T *wp)
{
  return wp->w_valid_cursor.coladd;
}

/// Set the valid cursor position (all fields).
void nvim_win_set_valid_cursor(win_T *wp, linenr_T lnum, colnr_T col, colnr_T coladd)
{
  wp->w_valid_cursor.lnum = lnum;
  wp->w_valid_cursor.col = col;
  wp->w_valid_cursor.coladd = coladd;
}

/// Set just the valid cursor column.
void nvim_win_set_valid_cursor_col(win_T *wp, colnr_T col)
{
  wp->w_valid_cursor.col = col;
}

/// Set just the valid cursor coladd.
void nvim_win_set_valid_cursor_coladd(win_T *wp, colnr_T coladd)
{
  wp->w_valid_cursor.coladd = coladd;
}

/// Get the leftcol (horizontal scroll position).
colnr_T nvim_win_get_leftcol(win_T *wp)
{
  return wp->w_leftcol;
}

/// Get the valid leftcol.
colnr_T nvim_win_get_valid_leftcol(win_T *wp)
{
  return wp->w_valid_leftcol;
}

/// Set the valid leftcol.
void nvim_win_set_valid_leftcol(win_T *wp, colnr_T val)
{
  wp->w_valid_leftcol = val;
}

/// Get the valid skipcol.
colnr_T nvim_win_get_valid_skipcol(win_T *wp)
{
  return wp->w_valid_skipcol;
}

/// Set the valid skipcol.
void nvim_win_set_valid_skipcol(win_T *wp, colnr_T val)
{
  wp->w_valid_skipcol = val;
}

/// Get the viewport_invalid flag.
int nvim_win_get_viewport_invalid(win_T *wp)
{
  return wp->w_viewport_invalid ? 1 : 0;
}

/// Set the viewport_invalid flag.
void nvim_win_set_viewport_invalid(win_T *wp, int val)
{
  wp->w_viewport_invalid = val != 0;
}

/// Get w_cline_row.
int nvim_win_get_cline_row(win_T *wp)
{
  return wp->w_cline_row;
}

/// Set w_cline_row.
void nvim_win_set_cline_row(win_T *wp, int val)
{
  wp->w_cline_row = val;
}

/// Get w_cline_height.
int nvim_win_get_cline_height(win_T *wp)
{
  return wp->w_cline_height;
}

/// Set w_cline_height.
void nvim_win_set_cline_height(win_T *wp, int val)
{
  wp->w_cline_height = val;
}

/// Get w_cline_folded.
int nvim_win_get_cline_folded(win_T *wp)
{
  return wp->w_cline_folded ? 1 : 0;
}

/// Set w_cline_folded.
void nvim_win_set_cline_folded(win_T *wp, int val)
{
  wp->w_cline_folded = val != 0;
}

/// Get w_curswant.
colnr_T nvim_win_get_curswant(win_T *wp)
{
  return wp->w_curswant;
}

/// Set w_curswant.
void nvim_win_set_curswant(win_T *wp, colnr_T val)
{
  wp->w_curswant = val;
}

/// Get w_set_curswant flag.
int nvim_win_get_set_curswant(win_T *wp)
{
  return wp->w_set_curswant ? 1 : 0;
}

/// Set w_set_curswant flag.
void nvim_win_set_set_curswant(win_T *wp, int val)
{
  wp->w_set_curswant = val != 0;
}

/// Get the window's 'breakindent' option.
int nvim_win_get_p_bri(win_T *wp)
{
  return wp->w_p_bri;
}

/// Get the window's 'rightleft' option.
int nvim_win_get_p_rl(win_T *wp)
{
  return wp->w_p_rl;
}

/// Set the window's 'rightleft' option.
void nvim_win_set_p_rl(win_T *wp, int val)
{
  if (wp) {
    wp->w_p_rl = val != 0;
  }
}

/// Get the window's 'arabic' option.
int nvim_win_get_p_arab(win_T *wp)
{
  return wp ? wp->w_p_arab : 0;
}

/// Get the window's grid view (for wlv_put_linebuf).
void *nvim_win_get_w_grid(win_T *wp)
{
  return &wp->w_grid;
}

/// Get the window's 'list' option.
int nvim_win_get_w_p_list(win_T *wp)
{
  return wp->w_p_list;
}

/// Get the window's 'listchars' tab1 character (first tab char).
uint32_t nvim_win_get_lcs_tab1(win_T *wp)
{
  return wp->w_p_lcs_chars.tab1;
}

/// Get the window's 'briopt' sbr flag.
bool nvim_win_get_briopt_sbr(win_T *wp)
{
  return wp->w_briopt_sbr;
}

/// Get the highlight attribute for a window's highlight group.
/// This wraps the inline win_hl_attr function for FFI.
int nvim_win_hl_attr(win_T *wp, int hlf)
{
  return win_hl_attr(wp, hlf);
}

/// Get the buffer associated with a window (for Rust FFI).
buf_T *nvim_win_get_buffer(win_T *wp)
{
  return wp->w_buffer;
}

/// Get a line from a window's buffer (for Rust FFI).
const char *nvim_win_ml_get_buf(win_T *wp, linenr_T lnum)
{
  return ml_get_buf(wp->w_buffer, lnum);
}

/// Check if UI has tabline extension.
int nvim_ui_has_tabline(void)
{
  return ui_has(kUITabline);
}

/// Get a specific border adjustment value for a window.
int nvim_win_get_border_adj(win_T *wp, int idx)
{
  if (idx < 0 || idx >= 4) {
    return 0;
  }
  return wp->w_border_adj[idx];
}

#define NOWIN           ((win_T *)-1)   // non-existing window

#define ROWS_AVAIL (Rows - p_ch - tabline_height() - global_stl_height())

/// Get the global ROWS_AVAIL value (Rows - p_ch - tabline_height() - global_stl_height()).
int nvim_get_rows_avail(void)
{
  return ROWS_AVAIL;
}

/// flags for win_enter_ext()
typedef enum {
  WEE_UNDO_SYNC = 0x01,
  WEE_CURWIN_INVALID = 0x02,
  WEE_TRIGGER_NEW_AUTOCMDS = 0x04,
  WEE_TRIGGER_ENTER_AUTOCMDS = 0x08,
  WEE_TRIGGER_LEAVE_AUTOCMDS = 0x10,
} wee_flags_T;

static const char e_cannot_split_window_when_closing_buffer[]
  = N_("E1159: Cannot split a window when closing the buffer");

static char *m_onlyone = N_("Already only one window");

/// When non-zero splitting a window is forbidden.  Used to avoid that nasty
/// autocommands mess up the window structure.
static int split_disallowed = 0;

// #define WIN_DEBUG
#ifdef WIN_DEBUG
/// Call this method to log the current window layout.
static void log_frame_layout(frame_T *frame)
{
  DLOG("layout %s, wi: %d, he: %d, wwi: %d, whe: %d, id: %d",
       frame->fr_layout == FR_LEAF ? "LEAF" : frame->fr_layout == FR_ROW ? "ROW" : "COL",
       frame->fr_width,
       frame->fr_height,
       frame->fr_win == NULL ? -1 : frame->fr_win->w_width,
       frame->fr_win == NULL ? -1 : frame->fr_win->w_height,
       frame->fr_win == NULL ? -1 : frame->fr_win->w_id);
  if (frame->fr_child != NULL) {
    DLOG("children");
    log_frame_layout(frame->fr_child);
    if (frame->fr_next != NULL) {
      DLOG("END of children");
    }
  }
  if (frame->fr_next != NULL) {
    log_frame_layout(frame->fr_next);
  }
}
#endif

/// Check if the current window is allowed to move to a different buffer.
///
/// @return If the window has 'winfixbuf', or this function will return false.
bool check_can_set_curbuf_disabled(void)
{
  if (curwin->w_p_wfb) {
    emsg(_(e_winfixbuf_cannot_go_to_buffer));
    return false;
  }

  return true;
}

/// Check if the current window is allowed to move to a different buffer.
///
/// @param forceit If true, do not error. If false and 'winfixbuf' is enabled, error.
///
/// @return If the window has 'winfixbuf', then forceit must be true
///     or this function will return false.
bool check_can_set_curbuf_forceit(int forceit)
{
  if (!forceit && curwin->w_p_wfb) {
    emsg(_(e_winfixbuf_cannot_go_to_buffer));
    return false;
  }

  return true;
}

/// @return the current window, unless in the cmdline window and "prevwin" is
/// set, then return "prevwin".
win_T *prevwin_curwin(void)
  FUNC_ATTR_WARN_UNUSED_RESULT
{
  // In cmdwin, the alternative buffer should be used.
  return is_in_cmdwin() && prevwin != NULL ? prevwin : curwin;
}

/// If the 'switchbuf' option contains "useopen" or "usetab", then try to jump
/// to a window containing "buf".
/// Returns the pointer to the window that was jumped to or NULL.
win_T *swbuf_goto_win_with_buf(buf_T *buf)
{
  win_T *wp = NULL;

  if (buf == NULL) {
    return wp;
  }

  // If 'switchbuf' contains "useopen": jump to first window in the current
  // tab page containing "buf" if one exists.
  if (swb_flags & kOptSwbFlagUseopen) {
    wp = buf_jump_open_win(buf);
  }

  // If 'switchbuf' contains "usetab": jump to first window in any tab page
  // containing "buf" if one exists.
  if (wp == NULL && (swb_flags & kOptSwbFlagUsetab)) {
    wp = buf_jump_open_tab(buf);
  }

  return wp;
}

// 'cmdheight' value explicitly set by the user: window commands are allowed to
// resize the topframe to values higher than this minimum, but not lower.
static OptInt min_set_ch = 1;

/// Get the min_set_ch value (minimum command line height set by user).
OptInt nvim_get_min_set_ch(void)
{
  return min_set_ch;
}

// Rust FFI declarations for drag functions
extern void rs_win_drag_status_line(win_T *dragwin, int offset);
extern void rs_win_drag_vsep_line(win_T *dragwin, int offset);

// Rust FFI declarations for equalization
extern void rs_win_equal(win_T *next_curwin, int current, int dir);

/// all CTRL-W window commands are handled here, called from normal_cmd().
///
/// @param xchar  extra char from ":wincmd gx" or NUL
void do_window(int nchar, int Prenum, int xchar)
{
  rs_do_window(nchar, Prenum, xchar);
}

static void cmd_with_count(char *cmd, char *bufp, size_t bufsize, int64_t Prenum)
{
  size_t len = xstrlcpy(bufp, cmd, bufsize);

  if (Prenum > 0 && len < bufsize) {
    vim_snprintf(bufp + len, bufsize - len, "%" PRId64, Prenum);
  }
}

void win_set_buf(win_T *win, buf_T *buf, Error *err)
  FUNC_ATTR_NONNULL_ALL
{
  const handle_T win_handle = win->handle;
  tabpage_T *tab = win_find_tabpage(win);

  // no redrawing and don't set the window title
  RedrawingDisabled++;

  switchwin_T switchwin;
  int win_result;

  TRY_WRAP(err, {
    win_result = switch_win_noblock(&switchwin, win, tab, true);
    if (win_result != FAIL) {
      const int save_acd = p_acd;
      if (!switchwin.sw_same_win) {
        // Temporarily disable 'autochdir' when setting buffer in another window.
        p_acd = false;
      }

      do_buffer(DOBUF_GOTO, DOBUF_FIRST, FORWARD, buf->b_fnum, 0);

      if (!switchwin.sw_same_win) {
        p_acd = save_acd;
      }
    }
  });
  if (win_result == FAIL && !ERROR_SET(err)) {
    api_set_error(err, kErrorTypeException, "Failed to switch to window %d", win_handle);
  }

  // If window is not current, state logic will not validate its cursor. So do it now.
  // Still needed if do_buffer returns FAIL (e.g: autocmds abort script after buffer was set).
  validate_cursor(curwin);

  restore_win_noblock(&switchwin, true);
  RedrawingDisabled--;
}

/// Return the number of fold columns to display
int win_fdccol_count(win_T *wp)
{
  const char *fdc = wp->w_p_fdc;

  // auto:<NUM>
  if (strncmp(fdc, "auto", 4) == 0) {
    const int fdccol = fdc[4] == ':' ? fdc[5] - '0' : 1;
    int needed_fdccols = getDeepestNesting(wp);
    return MIN(fdccol, needed_fdccols);
  }
  return fdc[0] - '0';
}

/// Merges two window configs, freeing replaced fields if necessary.
void merge_win_config(WinConfig *dst, const WinConfig src)
  FUNC_ATTR_NONNULL_ALL
{
  if (dst->title_chunks.items != src.title_chunks.items) {
    clear_virttext(&dst->title_chunks);
  }
  if (dst->footer_chunks.items != src.footer_chunks.items) {
    clear_virttext(&dst->footer_chunks);
  }
  *dst = src;
}

void ui_ext_win_position(win_T *wp, bool validate)
{
  wp->w_pos_changed = false;
  if (!wp->w_floating) {
    if (ui_has(kUIMultigrid)) {
      // Windows on the default grid don't necessarily have comp_col and comp_row set
      // But the rest of the calculations relies on it
      wp->w_grid_alloc.comp_col = wp->w_wincol;
      wp->w_grid_alloc.comp_row = wp->w_winrow;
    }
    ui_call_win_pos(wp->w_grid_alloc.handle, wp->handle, wp->w_winrow,
                    wp->w_wincol, wp->w_width, wp->w_height);
    return;
  }

  WinConfig c = wp->w_config;
  if (!c.external) {
    ScreenGrid *grid = &default_grid;
    Float row = c.row;
    Float col = c.col;
    if (c.relative == kFloatRelativeWindow) {
      Error dummy = ERROR_INIT;
      win_T *win = find_window_by_handle(c.window, &dummy);
      api_clear_error(&dummy);
      if (win != NULL) {
        // When a floating window is anchored to another window,
        // update the position of its anchored window first.
        if (win->w_pos_changed && win->w_grid_alloc.chars != NULL && win_valid(win)) {
          ui_ext_win_position(win, validate);
        }
        int row_off = 0;
        int col_off = 0;
        win_grid_alloc(win);
        grid = grid_adjust(&win->w_grid, &row_off, &col_off);
        row += row_off;
        col += col_off;
        if (c.bufpos.lnum >= 0) {
          int lnum = MIN(c.bufpos.lnum + 1, win->w_buffer->b_ml.ml_line_count);
          pos_T pos = { lnum, c.bufpos.col, 0 };
          int trow, tcol, tcolc, tcole;
          textpos2screenpos(win, &pos, &trow, &tcol, &tcolc, &tcole, true);
          row += trow - 1;
          col += tcol - 1;
        }
      }
    } else if (c.relative == kFloatRelativeLaststatus) {
      row += Rows - (int)p_ch - last_stl_height(false);
    } else if (c.relative == kFloatRelativeTabline) {
      row += tabline_height();
    }

    bool resort = wp->w_grid_alloc.comp_index != 0
                  && wp->w_grid_alloc.zindex != wp->w_config.zindex;
    bool raise = resort && wp->w_grid_alloc.zindex < wp->w_config.zindex;
    wp->w_grid_alloc.zindex = wp->w_config.zindex;
    if (resort) {
      ui_comp_layers_adjust(wp->w_grid_alloc.comp_index, raise);
    }
    bool valid = (wp->w_redr_type == 0 || ui_has(kUIMultigrid));
    if (!valid && !validate) {
      wp->w_pos_changed = true;
      return;
    }

    // TODO(bfredl): ideally, compositor should work like any multigrid UI
    // and use standard win_pos events.
    bool east = c.anchor & kFloatAnchorEast;
    bool south = c.anchor & kFloatAnchorSouth;

    int comp_row = (int)row - (south ? wp->w_height_outer : 0);
    int comp_col = (int)col - (east ? wp->w_width_outer : 0);
    int above_ch = wp->w_config.zindex < kZIndexMessages ? (int)p_ch : 0;
    comp_row += grid->comp_row;
    comp_col += grid->comp_col;
    comp_row = MAX(MIN(comp_row, Rows - wp->w_height_outer - above_ch), 0);
    if (!c.fixed || east) {
      comp_col = MAX(MIN(comp_col, Columns - wp->w_width_outer), 0);
    }
    wp->w_winrow = comp_row;
    wp->w_wincol = comp_col;

    if (!c.hide) {
      ui_comp_put_grid(&wp->w_grid_alloc, comp_row, comp_col,
                       wp->w_height_outer, wp->w_width_outer, valid, false);
      if (ui_has(kUIMultigrid)) {
        String anchor = cstr_as_string(float_anchor_str[c.anchor]);
        ui_call_win_float_pos(wp->w_grid_alloc.handle, wp->handle, anchor,
                              grid->handle, row, col, c.mouse,
                              wp->w_grid_alloc.zindex, (int)wp->w_grid_alloc.comp_index,
                              wp->w_winrow,
                              wp->w_wincol);
      }
      ui_check_cursor_grid(wp->w_grid_alloc.handle);
      wp->w_grid_alloc.mouse_enabled = wp->w_config.mouse;
      if (!valid) {
        wp->w_grid_alloc.valid = false;
        redraw_later(wp, UPD_NOT_VALID);
      }
    } else {
      if (ui_has(kUIMultigrid)) {
        ui_call_win_hide(wp->w_grid_alloc.handle);
      }
      ui_comp_remove_grid(&wp->w_grid_alloc);
    }
  } else {
    ui_call_win_external_pos(wp->w_grid_alloc.handle, wp->handle);
  }
}

void ui_ext_win_viewport(win_T *wp)
{
  // NOTE: The win_viewport command is delayed until the next flush when there are pending updates.
  // This ensures that the updates and the viewport are sent together.
  if ((wp == curwin || ui_has(kUIMultigrid)) && wp->w_viewport_invalid && wp->w_redr_type == 0) {
    const linenr_T line_count = wp->w_buffer->b_ml.ml_line_count;
    // Avoid ml_get errors when producing "scroll_delta".
    const linenr_T cur_topline = MIN(wp->w_topline, line_count);
    const linenr_T cur_botline = MIN(wp->w_botline, line_count);
    int64_t delta = 0;
    linenr_T last_topline = wp->w_viewport_last_topline;
    linenr_T last_botline = wp->w_viewport_last_botline;
    int last_topfill = wp->w_viewport_last_topfill;
    int64_t last_skipcol = wp->w_viewport_last_skipcol;
    if (last_topline > line_count) {
      delta -= last_topline - line_count;
      last_topline = line_count;
      last_topfill = 0;
      last_skipcol = MAXCOL;
    }
    last_botline = MIN(last_botline, line_count);
    if (cur_topline < last_topline
        || (cur_topline == last_topline && wp->w_skipcol < last_skipcol)) {
      int64_t vcole = last_skipcol;
      linenr_T lnume = last_topline;
      if (last_topline > 0 && cur_botline < last_topline) {
        // Scrolling too many lines: only give an approximate "scroll_delta".
        delta -= last_topline - cur_botline;
        lnume = cur_botline;
        vcole = 0;
      }
      delta -= win_text_height(wp, cur_topline, wp->w_skipcol, &lnume, &vcole, NULL, INT64_MAX);
    } else if (cur_topline > last_topline
               || (cur_topline == last_topline && wp->w_skipcol > last_skipcol)) {
      int64_t vcole = wp->w_skipcol;
      linenr_T lnume = cur_topline;
      if (last_botline > 0 && cur_topline > last_botline) {
        // Scrolling too many lines: only give an approximate "scroll_delta".
        delta += cur_topline - last_botline;
        lnume = last_botline;
        vcole = 0;
      }
      delta += win_text_height(wp, last_topline, last_skipcol, &lnume, &vcole, NULL, INT64_MAX);
    }
    delta += last_topfill;
    delta -= wp->w_topfill;
    linenr_T ev_botline = wp->w_botline;
    if (ev_botline == line_count + 1 && wp->w_empty_rows == 0) {
      // TODO(bfredl): The might be more cases to consider, like how does this
      // interact with incomplete final line? Diff filler lines?
      ev_botline = line_count;
    }
    ui_call_win_viewport(wp->w_grid_alloc.handle, wp->handle, wp->w_topline - 1, ev_botline,
                         wp->w_cursor.lnum - 1, wp->w_cursor.col, line_count, delta);
    wp->w_viewport_invalid = false;
    wp->w_viewport_last_topline = wp->w_topline;
    wp->w_viewport_last_botline = wp->w_botline;
    wp->w_viewport_last_topfill = wp->w_topfill;
    wp->w_viewport_last_skipcol = wp->w_skipcol;
  }
}

/// If "split_disallowed" is set, or "wp"'s buffer is closing, give an error and return FAIL.
/// Otherwise return OK.
int check_split_disallowed(const win_T *wp)
  FUNC_ATTR_NONNULL_ALL
{
  Error err = ERROR_INIT;
  const bool ok = check_split_disallowed_err(wp, &err);
  if (ERROR_SET(&err)) {
    emsg(_(err.msg));
    api_clear_error(&err);
  }
  return ok ? OK : FAIL;
}

/// Like `check_split_disallowed`, but set `err` to the (untranslated) error message on failure and
/// return false. Otherwise return true.
/// @see check_split_disallowed
bool check_split_disallowed_err(const win_T *wp, Error *err)
  FUNC_ATTR_NONNULL_ALL
{
  if (split_disallowed > 0) {
    api_set_error(err, kErrorTypeException, "E242: Can't split a window while closing another");
    return false;
  }
  if (wp->w_buffer->b_locked_split) {
    api_set_error(err, kErrorTypeException, "%s", e_cannot_split_window_when_closing_buffer);
    return false;
  }
  return true;
}

// split the current window, implements CTRL-W s and :split
//
// "size" is the height or width for the new window, 0 to use half of current
// height or width.
//
// "flags":
// WSP_ROOM: require enough room for new window
// WSP_VERT: vertical split.
// WSP_TOP:  open window at the top-left of the screen (help window).
// WSP_BOT:  open window at the bottom-right of the screen (quickfix window).
// WSP_HELP: creating the help window, keep layout snapshot
// WSP_NOENTER: do not enter the new window or trigger WinNew autocommands
//
// return FAIL for failure, OK otherwise
int win_split(int size, int flags)
{
  if (check_split_disallowed(curwin) == FAIL) {
    return FAIL;
  }

  // When the ":tab" modifier was used open a new tab page instead.
  if (may_open_tabpage() == OK) {
    return OK;
  }

  // Add flags from ":vertical", ":topleft" and ":botright".
  flags |= cmdmod.cmod_split;
  if ((flags & WSP_TOP) && (flags & WSP_BOT)) {
    emsg(_("E442: Can't split topleft and botright at the same time"));
    return FAIL;
  }

  // When creating the help window make a snapshot of the window layout.
  // Otherwise clear the snapshot, it's now invalid.
  if (flags & WSP_HELP) {
    make_snapshot(SNAP_HELP_IDX);
  } else {
    clear_snapshot(curtab, SNAP_HELP_IDX);
  }

  return win_split_ins(size, flags, NULL, 0, NULL) == NULL ? FAIL : OK;
}

/// When "new_wp" is NULL: split the current window in two.
/// When "new_wp" is not NULL: insert this window at the far
/// top/left/right/bottom.
/// When "to_flatten" is not NULL: flatten this frame before reorganising frames;
/// remains unflattened on failure.
///
/// On failure, if "new_wp" was not NULL, no changes will have been made to the
/// window layout or sizes.
/// @return  NULL for failure, or pointer to new window
win_T *win_split_ins(int size, int flags, win_T *new_wp, int dir, frame_T *to_flatten)
{
  win_T *oldwin;
  if (flags & WSP_TOP) {
    oldwin = firstwin;
  } else if (flags & WSP_BOT || curwin->w_floating) {
    oldwin = lastwin_nofloating();
  } else {
    oldwin = curwin;
  }

  SplitInsResult res = rs_win_split_ins(size, flags, new_wp, dir, to_flatten);
  if (res.wp == NULL) {
    return NULL;
  }

  if (res.do_enter) {
    win_enter_ext(res.wp, res.enter_flags);
  }
  // restore p_wiw or p_wh
  if (res.vertical) {
    p_wiw = res.saved_option;
  } else {
    p_wh = res.saved_option;
  }
  if (win_valid(oldwin)) {
    oldwin->w_pos_changed = true;
  }
  return res.wp;
}

// Initialize window "newp" from window "oldp".
// Used when splitting a window and when creating a new tab page.
// The windows will both edit the same buffer.
// WSP_NEWLOC may be specified in flags to prevent the location list from
// being copied.
void win_init(win_T *newp, win_T *oldp, int flags)
{
  newp->w_buffer = oldp->w_buffer;
  newp->w_s = &(oldp->w_buffer->b_s);
  oldp->w_buffer->b_nwindows++;
  newp->w_cursor = oldp->w_cursor;
  newp->w_valid = 0;
  newp->w_curswant = oldp->w_curswant;
  newp->w_set_curswant = oldp->w_set_curswant;
  newp->w_topline = oldp->w_topline;
  newp->w_topfill = oldp->w_topfill;
  newp->w_leftcol = oldp->w_leftcol;
  newp->w_pcmark = oldp->w_pcmark;
  newp->w_prev_pcmark = oldp->w_prev_pcmark;
  newp->w_alt_fnum = oldp->w_alt_fnum;
  newp->w_wrow = oldp->w_wrow;
  newp->w_fraction = oldp->w_fraction;
  newp->w_prev_fraction_row = oldp->w_prev_fraction_row;
  copy_jumplist(oldp, newp);
  if (flags & WSP_NEWLOC) {
    // Don't copy the location list.
    newp->w_llist = NULL;
    newp->w_llist_ref = NULL;
  } else {
    copy_loclist_stack(oldp, newp);
  }
  newp->w_localdir = (oldp->w_localdir == NULL)
                     ? NULL : xstrdup(oldp->w_localdir);
  newp->w_prevdir = (oldp->w_prevdir == NULL)
                    ? NULL : xstrdup(oldp->w_prevdir);

  if (*p_spk != 'c') {
    if (*p_spk == 't') {
      newp->w_skipcol = oldp->w_skipcol;
    }
    newp->w_botline = oldp->w_botline;
    newp->w_prev_height = oldp->w_height;
    newp->w_prev_winrow = oldp->w_winrow;
  }

  // copy tagstack and folds
  for (int i = 0; i < oldp->w_tagstacklen; i++) {
    taggy_T *tag = &newp->w_tagstack[i];
    *tag = oldp->w_tagstack[i];
    if (tag->tagname != NULL) {
      tag->tagname = xstrdup(tag->tagname);
    }
    if (tag->user_data != NULL) {
      tag->user_data = xstrdup(tag->user_data);
    }
  }
  newp->w_tagstackidx = oldp->w_tagstackidx;
  newp->w_tagstacklen = oldp->w_tagstacklen;

  // Keep same changelist position in new window.
  newp->w_changelistidx = oldp->w_changelistidx;

  copyFoldingState(oldp, newp);

  win_init_some(newp, oldp);

  newp->w_winbar_height = oldp->w_winbar_height;
}

// Initialize window "newp" from window "old".
// Only the essential things are copied.
static void win_init_some(win_T *newp, win_T *oldp)
{
  // Use the same argument list.
  newp->w_alist = oldp->w_alist;
  newp->w_alist->al_refcount++;
  newp->w_arg_idx = oldp->w_arg_idx;

  // copy options from existing window
  win_copy_options(oldp, newp);
}

/// Check if "win" is a pointer to an existing window in the current tabpage.
///
/// @param  win  window to check
bool win_valid(const win_T *win) FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_win_valid((win_T *)win) != 0;
}

/// Check if "win" is a pointer to an existing window in tabpage "tp".
///
/// @param  win  window to check
bool tabpage_win_valid(const tabpage_T *tp, const win_T *win)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_tabpage_win_valid((tabpage_T *)tp, (win_T *)win) != 0;
}

// Find window "handle" in the current tab page.
// Return NULL if not found.
win_T *win_find_by_handle(handle_T handle)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_win_find_by_handle(handle);
}

/// Check if "win" is a pointer to an existing window in any tabpage.
///
/// @param  win  window to check
bool win_valid_any_tab(win_T *win) FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_win_valid_any_tab(win) != 0;
}

// Return the number of windows.
int win_count(void)
{
  return rs_win_count();
}

/// Make "count" windows on the screen.
/// Must be called when there is just one window, filling the whole screen.
/// (excluding the command line).
///
/// @param vertical  split windows vertically if true.
///
/// @return actual number of windows on the screen.
int make_windows(int count, bool vertical)
{
  // Calculate maximum number of windows using Rust helper
  int maxcount = rs_split_max_windows(vertical);
  count = MIN(count, maxcount);

  // add status line now, otherwise first window will be too big
  if (count > 1) {
    last_status(true);
  }

  // Don't execute autocommands while creating the windows.  Must do that
  // when putting the buffers in the windows.
  block_autocmds();

  int todo;
  int flags = rs_split_make_windows_flags(vertical);

  // todo is number of windows left to create
  for (todo = count - 1; todo > 0; todo--) {
    int split_size = rs_split_iteration_size(vertical, todo);
    if (win_split(split_size, flags) == FAIL) {
      break;
    }
  }

  unblock_autocmds();

  // return actual number of windows
  return count - todo;
}

// Exchange current and next window
static void win_exchange(int Prenum)
{
  if (curwin->w_floating) {
    emsg(e_floatexchange);
    return;
  }

  if (one_window(curwin, NULL)) {
    // just one window
    beep_flush();
    return;
  }
  if (text_or_buf_locked()) {
    beep_flush();
    return;
  }

  frame_T *frp;

  // find window to exchange with
  if (Prenum) {
    frp = curwin->w_frame->fr_parent->fr_child;
    while (frp != NULL && --Prenum > 0) {
      frp = frp->fr_next;
    }
  } else if (curwin->w_frame->fr_next != NULL) {  // Swap with next
    frp = curwin->w_frame->fr_next;
  } else {  // Swap last window in row/col with previous
    frp = curwin->w_frame->fr_prev;
  }

  // We can only exchange a window with another window, not with a frame
  // containing windows.
  if (frp == NULL || frp->fr_win == NULL || frp->fr_win == curwin) {
    return;
  }
  win_T *wp = frp->fr_win;

  // 1. remove curwin from the list. Remember after which window it was in wp2
  // 2. insert curwin before wp in the list
  // if wp != wp2
  //    3. remove wp from the list
  //    4. insert wp after wp2
  // 5. exchange the status line height, winbar height, hsep height and vsep width.
  win_T *wp2 = curwin->w_prev;
  frame_T *frp2 = curwin->w_frame->fr_prev;
  if (wp->w_prev != curwin) {
    win_remove(curwin, NULL);
    frame_remove(curwin->w_frame);
    win_append(wp->w_prev, curwin, NULL);
    frame_insert(frp, curwin->w_frame);
  }
  if (wp != wp2) {
    win_remove(wp, NULL);
    frame_remove(wp->w_frame);
    win_append(wp2, wp, NULL);
    if (frp2 == NULL) {
      frame_insert(wp->w_frame->fr_parent->fr_child, wp->w_frame);
    } else {
      frame_append(frp2, wp->w_frame);
    }
  }
  int temp = curwin->w_status_height;
  curwin->w_status_height = wp->w_status_height;
  wp->w_status_height = temp;
  temp = curwin->w_vsep_width;
  curwin->w_vsep_width = wp->w_vsep_width;
  wp->w_vsep_width = temp;
  temp = curwin->w_hsep_height;
  curwin->w_hsep_height = wp->w_hsep_height;
  wp->w_hsep_height = temp;

  frame_fix_height(curwin);
  frame_fix_height(wp);
  frame_fix_width(curwin);
  frame_fix_width(wp);

  win_comp_pos();                 // recompute window positions

  if (wp->w_buffer != curbuf) {
    reset_VIsual_and_resel();
  } else if (VIsual_active) {
    wp->w_cursor = curwin->w_cursor;
  }

  win_enter(wp, true);
  redraw_later(curwin, UPD_NOT_VALID);
  redraw_later(wp, UPD_NOT_VALID);
}

// rotate windows: if upwards true the second window becomes the first one
//                 if upwards false the first window becomes the second one
static void win_rotate(bool upwards, int count)
{
  if (curwin->w_floating) {
    emsg(e_floatexchange);
    return;
  }

  if (count <= 0 || one_window(curwin, NULL)) {
    // nothing to do
    beep_flush();
    return;
  }

  // Check if all frames in this row/col have one window.
  frame_T *frp;
  FOR_ALL_FRAMES(frp, curwin->w_frame->fr_parent->fr_child) {
    if (frp->fr_win == NULL) {
      emsg(_("E443: Cannot rotate when another window is split"));
      return;
    }
  }

  win_T *wp1 = NULL;
  win_T *wp2 = NULL;

  while (count--) {
    if (upwards) {              // first window becomes last window
      // remove first window/frame from the list
      frp = curwin->w_frame->fr_parent->fr_child;
      assert(frp != NULL);
      wp1 = frp->fr_win;
      win_remove(wp1, NULL);
      frame_remove(frp);
      assert(frp->fr_parent->fr_child);

      // find last frame and append removed window/frame after it
      for (; frp->fr_next != NULL; frp = frp->fr_next) {}
      win_append(frp->fr_win, wp1, NULL);
      frame_append(frp, wp1->w_frame);

      wp2 = frp->fr_win;                // previously last window
    } else {                  // last window becomes first window
      // find last window/frame in the list and remove it
      for (frp = curwin->w_frame; frp->fr_next != NULL;
           frp = frp->fr_next) {}
      wp1 = frp->fr_win;
      wp2 = wp1->w_prev;                    // will become last window
      win_remove(wp1, NULL);
      frame_remove(frp);
      assert(frp->fr_parent->fr_child);

      // append the removed window/frame before the first in the list
      win_append(frp->fr_parent->fr_child->fr_win->w_prev, wp1, NULL);
      frame_insert(frp->fr_parent->fr_child, frp);
    }

    // exchange status height, winbar height, hsep height and vsep width of old and new last window
    int n = wp2->w_status_height;
    wp2->w_status_height = wp1->w_status_height;
    wp1->w_status_height = n;
    n = wp2->w_hsep_height;
    wp2->w_hsep_height = wp1->w_hsep_height;
    wp1->w_hsep_height = n;
    frame_fix_height(wp1);
    frame_fix_height(wp2);
    n = wp2->w_vsep_width;
    wp2->w_vsep_width = wp1->w_vsep_width;
    wp1->w_vsep_width = n;
    frame_fix_width(wp1);
    frame_fix_width(wp2);

    // recompute w_winrow and w_wincol for all windows
    win_comp_pos();
  }

  wp1->w_pos_changed = true;
  wp2->w_pos_changed = true;

  redraw_all_later(UPD_NOT_VALID);
}

/// Move "wp" into a new split in a given direction, possibly relative to the
/// current window.
/// "wp" must be valid in the current tabpage.
/// Returns FAIL for failure, OK otherwise.
int win_splitmove(win_T *wp, int size, int flags)
{
  int dir = 0;
  int height = wp->w_height;

  if (one_window(wp, NULL)) {
    return OK;  // nothing to do
  }
  if (is_aucmd_win(wp) || check_split_disallowed(wp) == FAIL) {
    return FAIL;
  }

  frame_T *unflat_altfr = NULL;
  if (wp->w_floating) {
    win_remove(wp, NULL);
  } else {
    // Remove the window and frame from the tree of frames.  Don't flatten any
    // frames yet so we can restore things if win_split_ins fails.
    winframe_remove(wp, &dir, NULL, &unflat_altfr);
    assert(unflat_altfr != NULL);
    win_remove(wp, NULL);
    last_status(false);  // may need to remove last status line
    win_comp_pos();  // recompute window positions
  }

  // Split a window on the desired side and put "wp" there.
  if (win_split_ins(size, flags, wp, dir, unflat_altfr) == NULL) {
    if (!wp->w_floating) {
      assert(unflat_altfr != NULL);
      // win_split_ins doesn't change sizes or layout if it fails to insert an
      // existing window, so just undo winframe_remove.
      winframe_restore(wp, dir, unflat_altfr);
    }
    win_append(wp->w_prev, wp, NULL);
    return FAIL;
  }

  // If splitting horizontally, try to preserve height.
  // Note that win_split_ins autocommands may have immediately closed "wp", or made it floating!
  if (size == 0 && !(flags & WSP_VERT) && win_valid(wp) && !wp->w_floating) {
    win_setheight_win(height, wp);
    if (p_ea) {
      // Equalize windows.  Note that win_split_ins autocommands may have
      // made a window other than "wp" current.
      win_equal(curwin, curwin == wp, 'v');
    }
  }

  return OK;
}

// Move window "win1" to below/right of "win2" and make "win1" the current
// window.  Only works within the same frame!
void win_move_after(win_T *win1, win_T *win2)
{
  // check if the arguments are reasonable
  if (win1 == win2) {
    return;
  }

  // check if there is something to do
  if (win2->w_next != win1) {
    if (win1->w_frame->fr_parent != win2->w_frame->fr_parent) {
      iemsg("INTERNAL: trying to move a window into another frame");
      return;
    }

    // may need to move the status line, window bar, horizontal or vertical separator of the last
    // window
    if (win1 == lastwin) {
      int height = win1->w_prev->w_status_height;
      win1->w_prev->w_status_height = win1->w_status_height;
      win1->w_status_height = height;

      height = win1->w_prev->w_hsep_height;
      win1->w_prev->w_hsep_height = win1->w_hsep_height;
      win1->w_hsep_height = height;

      if (win1->w_prev->w_vsep_width == 1) {
        // Remove the vertical separator from the last-but-one window,
        // add it to the last window.  Adjust the frame widths.
        win1->w_prev->w_vsep_width = 0;
        win1->w_prev->w_frame->fr_width -= 1;
        win1->w_vsep_width = 1;
        win1->w_frame->fr_width += 1;
      }
    } else if (win2 == lastwin) {
      int height = win1->w_status_height;
      win1->w_status_height = win2->w_status_height;
      win2->w_status_height = height;

      height = win1->w_hsep_height;
      win1->w_hsep_height = win2->w_hsep_height;
      win2->w_hsep_height = height;

      if (win1->w_vsep_width == 1) {
        // Remove the vertical separator from win1, add it to the last
        // window, win2.  Adjust the frame widths.
        win2->w_vsep_width = 1;
        win2->w_frame->fr_width += 1;
        win1->w_vsep_width = 0;
        win1->w_frame->fr_width -= 1;
      }
    }
    win_remove(win1, NULL);
    frame_remove(win1->w_frame);
    win_append(win2, win1, NULL);
    frame_append(win2->w_frame, win1->w_frame);

    win_comp_pos();  // recompute w_winrow for all windows
    redraw_later(curwin, UPD_NOT_VALID);
  }
  win_enter(win1, false);

  win1->w_pos_changed = true;
  win2->w_pos_changed = true;
}

/// Compute maximum number of windows that can fit within "height" in frame "fr".
static int get_maximum_wincount(frame_T *fr, int height)
{
  if (fr->fr_layout != FR_COL) {
    return (height / ((int)p_wmh + STATUS_HEIGHT + frame2win(fr)->w_winbar_height));
  } else if (global_winbar_height()) {
    // If winbar is globally enabled, no need to check each window for it.
    return (height / ((int)p_wmh + STATUS_HEIGHT + 1));
  }

  frame_T *frp;
  int total_wincount = 0;

  // First, try to fit all child frames of "fr" into "height"
  FOR_ALL_FRAMES(frp, fr->fr_child) {
    win_T *wp = frame2win(frp);

    if (height < (p_wmh + STATUS_HEIGHT + wp->w_winbar_height)) {
      break;
    }
    height -= (int)p_wmh + STATUS_HEIGHT + wp->w_winbar_height;
    total_wincount += 1;
  }

  // If we still have enough room for more windows, just use the default winbar height (which is 0)
  // in order to get the amount of windows that'd fit in the remaining space
  total_wincount += height / ((int)p_wmh + STATUS_HEIGHT);

  return total_wincount;
}

/// Make all windows the same height.
/// 'next_curwin' will soon be the current window, make sure it has enough rows.
///
/// @param next_curwin  pointer to current window to be or NULL
/// @param current  do only frame with current window
/// @param dir  'v' for vertically, 'h' for horizontally, 'b' for both, 0 for using p_ead
void win_equal(win_T *next_curwin, bool current, int dir)
{
  rs_win_equal(next_curwin, current, dir);
}

void leaving_window(win_T *const win)
  FUNC_ATTR_NONNULL_ALL
{
  // Only matters for a prompt window.
  if (!bt_prompt(win->w_buffer)) {
    return;
  }

  // When leaving a prompt window stop Insert mode and perhaps restart
  // it when entering that window again.
  win->w_buffer->b_prompt_insert = restart_edit;
  if (restart_edit != NUL && mode_displayed) {
    clear_cmdline = true;  // unshow mode later
  }
  restart_edit = NUL;

  // When leaving the window (or closing the window) was done from a
  // callback we need to break out of the Insert mode loop and restart Insert
  // mode when entering the window again.
  if ((State & MODE_INSERT) && !stop_insert_mode) {
    stop_insert_mode = true;
    if (win->w_buffer->b_prompt_insert == NUL) {
      win->w_buffer->b_prompt_insert = 'A';
    }
  }
}

void entering_window(win_T *const win)
  FUNC_ATTR_NONNULL_ALL
{
  // Only matters for a prompt window.
  if (!bt_prompt(win->w_buffer)) {
    return;
  }

  // When switching to a prompt buffer that was in Insert mode, don't stop
  // Insert mode, it may have been set in leaving_window().
  if (win->w_buffer->b_prompt_insert != NUL) {
    stop_insert_mode = false;
  }

  // When entering the prompt window restart Insert mode if we were in Insert
  // mode when we left it and not already in Insert mode.
  if ((State & MODE_INSERT) == 0) {
    restart_edit = win->w_buffer->b_prompt_insert;
  }
}

void win_init_empty(win_T *wp)
{
  redraw_later(wp, UPD_NOT_VALID);
  wp->w_lines_valid = 0;
  wp->w_cursor.lnum = 1;
  wp->w_curswant = wp->w_cursor.col = 0;
  wp->w_cursor.coladd = 0;
  wp->w_pcmark.lnum = 1;        // pcmark not cleared but set to line 1
  wp->w_pcmark.col = 0;
  wp->w_prev_pcmark.lnum = 0;
  wp->w_prev_pcmark.col = 0;
  wp->w_topline = 1;
  wp->w_topfill = 0;
  wp->w_botline = 2;
  wp->w_valid = 0;
  wp->w_s = &wp->w_buffer->b_s;
}

/// Init the current window "curwin".
/// Called when a new file is being edited.
void curwin_init(void)
{
  win_init_empty(curwin);
}

/// Closes all windows for buffer `buf` unless there is only one non-floating window.
///
/// @param keep_curwin  don't close `curwin`
void close_windows(buf_T *buf, bool keep_curwin)
{
  RedrawingDisabled++;

  // Start from lastwin to close floating windows with the same buffer first.
  // When the autocommand window is involved win_close() may need to print an error message.
  for (win_T *wp = lastwin; wp != NULL && (is_aucmd_win(lastwin) || !one_window(wp, NULL));) {
    if (wp->w_buffer == buf && (!keep_curwin || wp != curwin)
        && !(win_locked(wp) || wp->w_buffer->b_locked > 0)) {
      if (win_close(wp, false, false) == FAIL) {
        // If closing the window fails give up, to avoid looping forever.
        break;
      }

      // Start all over, autocommands may change the window layout.
      wp = lastwin;
    } else {
      wp = wp->w_prev;
    }
  }

  tabpage_T *nexttp;

  // Also check windows in other tab pages.
  for (tabpage_T *tp = first_tabpage; tp != NULL; tp = nexttp) {
    nexttp = tp->tp_next;
    if (tp != curtab) {
      // Start from tp_lastwin to close floating windows with the same buffer first.
      for (win_T *wp = tp->tp_lastwin; wp != NULL; wp = wp->w_prev) {
        if (wp->w_buffer == buf
            && !(win_locked(wp) || wp->w_buffer->b_locked > 0)) {
          if (!win_close_othertab(wp, false, tp, false)) {
            // If closing the window fails give up, to avoid looping forever.
            break;
          }

          // Start all over, the tab page may be closed and
          // autocommands may change the window layout.
          nexttp = first_tabpage;
          break;
        }
      }
    }
  }

  RedrawingDisabled--;
}

/// Check if "win" is the last non-floating window that exists.
bool last_window(win_T *win) FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_last_window(win) != 0;
}

/// Check if "win" is the only non-floating window in tabpage "tp", or NULL for current tabpage.
///
/// This should be used in place of ONE_WINDOW when necessary,
/// with "firstwin" or the affected window as argument depending on the situation.
bool one_window(win_T *win, tabpage_T *tp)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ARG(1)
{
  return rs_one_window_in_tab(win, tp) != 0;
}

/// Check if floating windows in tabpage `tp` can be closed.
/// Do not call this when the autocommand window is in use!
///
/// @param tp tabpage to check. Must be NULL for the current tabpage.
/// @return true if all floating windows can be closed
static bool can_close_floating_windows(tabpage_T *tp)
{
  assert(tp != curtab && (tp || !is_aucmd_win(lastwin)));
  // Use Rust helper for current tabpage
  if (tp == NULL) {
    return rs_close_can_close_floating() != 0;
  }
  // For other tabpages, iterate in C (since we need tp_lastwin access)
  for (win_T *wp = tp->tp_lastwin; wp->w_floating; wp = wp->w_prev) {
    buf_T *buf = wp->w_buffer;
    int need_hide = (bufIsChanged(buf) && buf->b_nwindows <= 1);

    if (need_hide && !buf_hide(buf)) {
      return false;
    }
  }
  return true;
}

/// @return true if, considering the cmdwin, `win` is safe to close.
/// If false and `win` is the cmdwin, it is closed; otherwise, `err` is set.
bool can_close_in_cmdwin(win_T *win, Error *err)
  FUNC_ATTR_NONNULL_ALL
{
  if (cmdwin_type != 0) {
    if (win == cmdwin_win) {
      cmdwin_result = Ctrl_C;
      return false;
    } else if (win == cmdwin_old_curwin) {
      api_set_error(err, kErrorTypeException, "%s", e_cmdwin);
      return false;
    }
  }
  return true;
}

/// Close the possibly last window in a tab page.
///
/// @param  win          window to close
/// @param  free_buf     whether to free the window's current buffer
/// @param  prev_curtab  previous tabpage that will be closed if "win" is the
///                      last window in the tabpage
///
/// @return false if there are other windows and nothing is done, true otherwise.
static bool close_last_window_tabpage(win_T *win, bool free_buf, tabpage_T *prev_curtab)
  FUNC_ATTR_NONNULL_ARG(1)
{
  if (!ONE_WINDOW) {
    return false;
  }

  buf_T *old_curbuf = curbuf;

  Terminal *term = win->w_buffer ? win->w_buffer->terminal : NULL;
  if (term) {
    // Don't free terminal buffers
    free_buf = false;
  }

  // Closing the last window in a tab page.  First go to another tab
  // page and then close the window and the tab page.  This avoids that
  // curwin and curtab are invalid while we are freeing memory, they may
  // be used in GUI events.
  // Don't trigger *Enter autocommands yet, they may use wrong values, so do
  // that below.
  // Do trigger *Leave autocommands, unless win->w_buffer is NULL, in which
  // case they have already been triggered.
  goto_tabpage_tp(alt_tabpage(), false, win->w_buffer != NULL);

  // Safety check: Autocommands may have switched back to the old tab page
  // or closed the window when jumping to the other tab page.
  if (curtab != prev_curtab && valid_tabpage(prev_curtab) && prev_curtab->tp_firstwin == win) {
    win_close_othertab(win, free_buf, prev_curtab, false);
  }
  entering_window(curwin);

  // Since goto_tabpage_tp above did not trigger *Enter autocommands, do
  // that now.
  apply_autocmds(EVENT_WINENTER, NULL, NULL, false, curbuf);
  apply_autocmds(EVENT_TABENTER, NULL, NULL, false, curbuf);
  if (old_curbuf != curbuf) {
    apply_autocmds(EVENT_BUFENTER, NULL, NULL, false, curbuf);
  }
  return true;
}

/// Close the buffer of "win" and unload it if "action" is DOBUF_UNLOAD.
/// "action" can also be zero (do nothing).
/// "abort_if_last" is passed to close_buffer(): abort closing if all other
/// windows are closed.
static void win_close_buffer(win_T *win, int action, bool abort_if_last)
  FUNC_ATTR_NONNULL_ALL
{
  // Free independent synblock before the buffer is freed.
  if (win->w_buffer != NULL) {
    reset_synblock(win);
  }

  // When a quickfix/location list window is closed and the buffer is
  // displayed in only one window, then unlist the buffer.
  if (win->w_buffer != NULL && bt_quickfix(win->w_buffer)
      && win->w_buffer->b_nwindows == 1) {
    win->w_buffer->b_p_bl = false;
  }

  // Close the link to the buffer.
  if (win->w_buffer != NULL) {
    bufref_T bufref;
    set_bufref(&bufref, curbuf);
    win->w_locked = true;
    close_buffer(win, win->w_buffer, action, abort_if_last, true);
    if (win_valid_any_tab(win)) {
      win->w_locked = false;
    }

    // Make sure curbuf is valid. It can become invalid if 'bufhidden' is
    // "wipe".
    if (!bufref_valid(&bufref)) {
      curbuf = firstbuf;
    }
  }
}

// Close window "win".  Only works for the current tab page.
// If "free_buf" is true related buffer may be unloaded.
//
// Called by :quit, :close, :xit, :wq and findtag().
// Returns FAIL when the window was not closed.
int win_close(win_T *win, bool free_buf, bool force)
  FUNC_ATTR_NONNULL_ALL
{
  tabpage_T *prev_curtab = curtab;
  frame_T *win_frame = win->w_floating ? NULL : win->w_frame->fr_parent;
  const bool had_diffmode = win->w_p_diff;

  // --- Phase 1: Rust validation ---
  int vrc = rs_win_close_validate(win, free_buf ? 1 : 0, force ? 1 : 0);
  if (vrc == 1) {
    // Specific error messages for last_window, locked, aucmd_win, E814.
    // Rust doesn't emit these (except E814), so re-check and emit here.
    if (last_window(win)) {
      emsg(_("E444: Cannot close last window"));
    } else if (is_aucmd_win(win)) {
      emsg(_(e_autocmd_close));
    } else if (lastwin->w_floating && one_window(win, NULL) && is_aucmd_win(lastwin)) {
      emsg(_("E814: Cannot close window, only autocmd window would remain"));
    }
    return FAIL;
  }
  if (vrc == 2) {
    // Floating-only: recursive close loop (must stay in C).
    if (force || can_close_floating_windows(NULL)) {
      while (lastwin->w_floating) {
        if (win_close(lastwin, free_buf, true) == FAIL) {
          return FAIL;
        }
      }
      if (!win_valid_any_tab(win)) {
        return FAIL;
      }
    } else {
      emsg(e_floatonly);
      return FAIL;
    }
  }

  // close_last_window_tabpage (heavy autocmd interaction, stays in C).
  if (close_last_window_tabpage(win, free_buf, prev_curtab)) {
    return FAIL;
  }

  // --- Autocmd-heavy section (stays in C) ---

  bool help_window = false;
  if (bt_help(win->w_buffer)) {
    help_window = true;
  } else {
    clear_snapshot(curtab, SNAP_HELP_IDX);
  }

  win_T *wp;
  bool other_buffer = false;

  if (win == curwin) {
    leaving_window(curwin);

    wp = win->w_floating ? win_float_find_altwin(win, NULL) : frame2win(win_altframe(win, NULL));

    if (wp->w_buffer != curbuf) {
      reset_VIsual_and_resel();

      other_buffer = true;
      if (!win_valid(win)) {
        return FAIL;
      }
      win->w_locked = true;
      apply_autocmds(EVENT_BUFLEAVE, NULL, NULL, false, curbuf);
      if (!win_valid(win)) {
        return FAIL;
      }
      win->w_locked = false;
      if (last_window(win)) {
        return FAIL;
      }
    }
    win->w_locked = true;
    apply_autocmds(EVENT_WINLEAVE, NULL, NULL, false, curbuf);
    if (!win_valid(win)) {
      return FAIL;
    }
    win->w_locked = false;
    if (last_window(win)) {
      return FAIL;
    }
    if (aborting()) {
      return FAIL;
    }
  }

  do_autocmd_winclosed(win);
  if (!win_valid_any_tab(win)) {
    return OK;
  }

  win_close_buffer(win, free_buf ? DOBUF_UNLOAD : 0, true);

  if (win_valid(win) && win->w_buffer == NULL
      && !win->w_floating && last_window(win)) {
    if (curwin->w_buffer == NULL) {
      curwin->w_buffer = curbuf;
    }
    getout(0);
  }
  if (curtab != prev_curtab && win_valid_any_tab(win)
      && win->w_buffer == NULL) {
    win_close_othertab(win, false, prev_curtab, force);
    return FAIL;
  }

  if (!win_valid(win) || (!win->w_floating && last_window(win))
      || close_last_window_tabpage(win, free_buf, prev_curtab)) {
    return FAIL;
  }

  // --- Phase 2: Rust structural close ---
  WinCloseStructResult res = rs_win_close_structural(win, help_window ? 1 : 0, win_frame);
  wp = res.wp;

  // --- Phase 3: Rust post-layout ---
  rs_win_close_post_layout(res.was_floating, res.dir, win_frame);

  // --- Post-close autocmds (stay in C) ---
  if (res.close_curwin) {
    win_enter_ext(wp, WEE_CURWIN_INVALID | WEE_TRIGGER_ENTER_AUTOCMDS
                  | WEE_TRIGGER_LEAVE_AUTOCMDS);
    if (other_buffer) {
      apply_autocmds(EVENT_BUFENTER, NULL, NULL, false, curbuf);
    }
  }

  if (ONE_WINDOW && curwin->w_locked && curbuf->b_locked_split
      && first_tabpage->tp_next != NULL) {
    apply_autocmds(EVENT_TABLEAVE, NULL, NULL, false, curbuf);
  }

  nvim_dec_split_disallowed();

  if (help_window) {
    restore_snapshot(SNAP_HELP_IDX, res.close_curwin);
  }

  if (diffopt_closeoff() && had_diffmode && curtab == prev_curtab) {
    int diffcount = 0;

    FOR_ALL_WINDOWS_IN_TAB(dwin, curtab) {
      if (dwin->w_p_diff) {
        diffcount++;
      }
    }
    if (diffcount == 1) {
      do_cmdline_cmd("diffoff!");
    }
  }

  curwin->w_pos_changed = true;
  if (!res.was_floating) {
    redraw_all_later(UPD_NOT_VALID);
  }
  return OK;
}

static void do_autocmd_winclosed(win_T *win)
  FUNC_ATTR_NONNULL_ALL
{
  static bool recursive = false;
  if (recursive || !has_event(EVENT_WINCLOSED)) {
    return;
  }
  recursive = true;
  char winid[NUMBUFLEN];
  vim_snprintf(winid, sizeof(winid), "%d", win->handle);
  apply_autocmds(EVENT_WINCLOSED, winid, winid, false, win->w_buffer);
  recursive = false;
}

// Close window "win" in tab page "tp", which is not the current tab page.
// This may be the last window in that tab page and result in closing the tab,
// thus "tp" may become invalid!
// Caller must check if buffer is hidden and whether the tabline needs to be
// updated.
// @return false when the window was not closed as a direct result of this call
//         (e.g: not via autocmds).
bool win_close_othertab(win_T *win, int free_buf, tabpage_T *tp, bool force)
  FUNC_ATTR_NONNULL_ALL
{
  assert(tp != curtab);
  bool did_decrement = false;

  // Phase 1: Rust validation (locked, aucmd_win, floating-only check).
  int vrc = rs_close_othertab_validate(win, tp, force ? 1 : 0);
  if (vrc == 1) {
    return false;  // locked or aucmd_win
  }
  if (vrc == 2) {
    // Floating-only: recursively close floating windows (must stay in C
    // because win_close_othertab is recursive and triggers autocmds).
    while (tp->tp_lastwin->w_floating) {
      if (!win_close_othertab(tp->tp_lastwin, free_buf, tp, true)) {
        goto leave_open;
      }
    }
    if (!win_valid_any_tab(win)) {
      return false;
    }
  }
  if (vrc == 3) {
    goto leave_open;  // e_floatonly already emitted by Rust
  }

  // --- Autocmd section (must stay in C) ---

  // Fire WinClosed just before starting to free window-related resources.
  if (win->w_buffer != NULL) {
    do_autocmd_winclosed(win);
    if (!win_valid_any_tab(win)) {
      return false;
    }
  }

  bufref_T bufref;
  set_bufref(&bufref, win->w_buffer);

  if (win->w_buffer != NULL) {
    did_decrement = close_buffer(win, win->w_buffer, free_buf ? DOBUF_UNLOAD : 0, false, true);
  }

  // Re-validate after autocmds.
  if (!valid_tabpage(tp) || tp == curtab) {
    goto leave_open;
  }
  if (!tabpage_win_valid(tp, win)) {
    goto leave_open;
  }
  if (tp->tp_lastwin->w_floating && one_window(win, tp)) {
    emsg(e_floatonly);
    goto leave_open;
  }

  // Phase 2: Rust tabpage removal (pure structural logic).
  TabRemoveResult res = rs_close_othertab_remove_tabpage(win, tp);

  // Free the memory used for the window.
  buf_T *buf = win->w_buffer;
  int dir;
  win_free_mem(win, &dir, tp);

  if (res.free_tp_idx > 0) {
    free_tabpage(tp);

    if (has_event(EVENT_TABCLOSED)) {
      char prev_idx[NUMBUFLEN];
      vim_snprintf(prev_idx, NUMBUFLEN, "%i", res.free_tp_idx);
      apply_autocmds(EVENT_TABCLOSED, prev_idx, prev_idx, false, buf);
    }
  }
  return true;

leave_open:
  // Phase 3: Rust error recovery.
  rs_close_othertab_leave_open(win, did_decrement ? 1 : 0,
                               bufref.br_buf, bufref_valid(&bufref) ? 1 : 0);
  return false;
}

/// Free the memory used for a window.
///
/// @param dirp  set to 'v' or 'h' for direction if 'ea'
/// @param tp    tab page "win" is in, NULL for current
///
/// @return      a pointer to the window that got the freed up space.
static win_T *win_free_mem(win_T *win, int *dirp, tabpage_T *tp)
  FUNC_ATTR_NONNULL_ARG(1)
{
  win_T *wp;
  tabpage_T *win_tp = tp == NULL ? curtab : tp;

  if (!win->w_floating) {
    // Remove the window and its frame from the tree of frames.
    frame_T *frp = win->w_frame;
    wp = winframe_remove(win, dirp, tp, NULL);
    xfree(frp);
  } else {
    *dirp = 'h';  // Dummy value.
    wp = win_float_find_altwin(win, tp);
  }
  win_free(win, tp);

  // When deleting the current window in the tab, select a new current
  // window.
  if (win == win_tp->tp_curwin) {
    win_tp->tp_curwin = wp;
  }
  // Avoid executing cmdline_win logic after it is closed.
  if (win == cmdline_win) {
    cmdline_win = NULL;
  }

  return wp;
}

#if defined(EXITFREE)
void win_free_all(void)
{
  // avoid an error for switching tabpage with the cmdline window open
  cmdwin_type = 0;
  cmdwin_buf = NULL;
  cmdwin_win = NULL;
  cmdwin_old_curwin = NULL;

  while (first_tabpage->tp_next != NULL) {
    tabpage_close(true);
  }

  while (lastwin != NULL && lastwin->w_floating) {
    win_T *wp = lastwin;
    win_remove(lastwin, NULL);
    int dummy;
    win_free_mem(wp, &dummy, NULL);
    for (int i = 0; i < AUCMD_WIN_COUNT; i++) {
      if (aucmd_win[i].auc_win == wp) {
        aucmd_win[i].auc_win = NULL;
      }
    }
  }

  for (int i = 0; i < AUCMD_WIN_COUNT; i++) {
    if (aucmd_win[i].auc_win != NULL) {
      int dummy;
      win_free_mem(aucmd_win[i].auc_win, &dummy, NULL);
      aucmd_win[i].auc_win = NULL;
    }
  }

  kv_destroy(aucmd_win_vec);

  while (firstwin != NULL) {
    int dummy;
    win_free_mem(firstwin, &dummy, NULL);
  }

  // No window should be used after this. Set curwin to NULL to crash
  // instead of using freed memory.
  curwin = NULL;
}

#endif

/// Remove a window and its frame from the tree of frames.
///
/// @param dirp  set to 'v' or 'h' for direction if 'ea'
/// @param tp    tab page "win" is in, NULL for current
/// @param unflat_altfr if not NULL, set to pointer of frame that got
///                     the space, and it is not flattened
///
/// @return      a pointer to the window that got the freed up space.
win_T *winframe_remove(win_T *win, int *dirp, tabpage_T *tp, frame_T **unflat_altfr)
  FUNC_ATTR_NONNULL_ARG(1, 2)
{
  frame_T *altfr;
  win_T *wp = winframe_find_altwin(win, dirp, tp, &altfr);
  if (wp == NULL) {
    return NULL;
  }

  frame_T *frp_close = win->w_frame;

  // Save the position of the containing frame (which will also contain the
  // altframe) before we remove anything, to recompute window positions later.
  const win_T *const topleft = frame2win(frp_close->fr_parent);
  int row = topleft->w_winrow;
  int col = topleft->w_wincol;

  // If this is a rightmost window, remove vertical separators to the left.
  if (win->w_vsep_width == 0 && frp_close->fr_parent->fr_layout == FR_ROW
      && frp_close->fr_prev != NULL) {
    frame_set_vsep(frp_close->fr_prev, false);
  }

  // Remove this frame from the list of frames.
  frame_remove(frp_close);

  if (*dirp == 'v') {
    frame_new_height(altfr, altfr->fr_height + frp_close->fr_height,
                     altfr == frp_close->fr_next, false, false);
  } else {
    assert(*dirp == 'h');
    frame_new_width(altfr, altfr->fr_width + frp_close->fr_width,
                    altfr == frp_close->fr_next, false);
  }

  // If the altframe wasn't adjacent and left/above, resizing it will have
  // changed window positions within the parent frame.  Recompute them.
  if (altfr != frp_close->fr_prev) {
    frame_comp_pos(frp_close->fr_parent, &row, &col);
  }

  if (unflat_altfr == NULL) {
    frame_flatten(altfr);
  } else {
    *unflat_altfr = altfr;
  }

  return wp;
}

/// Find the window that will get the freed space from a call to `winframe_remove`.
/// Makes no changes to the window layout.
///
/// @param dirp  set to 'v' or 'h' for the direction where "altfr" will be resized
///              to fill the space
/// @param tp    tab page "win" is in, NULL for current
/// @param altfr if not NULL, set to pointer of frame that will get the space
///
/// @return      a pointer to the window that will get the freed up space, or NULL
///              if there is no other non-float to receive the space.
win_T *winframe_find_altwin(win_T *win, int *dirp, tabpage_T *tp, frame_T **altfr)
  FUNC_ATTR_NONNULL_ARG(1, 2)
{
  assert(tp == NULL || tp != curtab);

  // If there is only one non-floating window there is nothing to remove.
  if (one_window(win, tp)) {
    return NULL;
  }

  // Find the initial window and frame that gets the space.
  frame_T *frp2 = win_altframe(win, tp);

  // Call Rust to find the best altframe considering wfh/wfw constraints.
  WinframeResult result = rs_winframe_find_altwin(win, frp2);
  frp2 = result.altfr;
  *dirp = result.dir;

  win_T *wp = frame2win(frp2);

  assert(wp != win && frp2 != win->w_frame);
  if (altfr != NULL) {
    *altfr = frp2;
  }

  return wp;
}

/// Flatten "frp" into its parent frame if it's the only child, also merging its
/// list with the grandparent if they share the same layout.
/// Frees "frp" if flattened; also "frp->fr_parent" if it has the same layout.
static void frame_flatten(frame_T *frp)
  FUNC_ATTR_NONNULL_ALL
{
  if (frp->fr_next != NULL || frp->fr_prev != NULL) {
    return;
  }

  // There is no other frame in this list, move its info to the parent
  // and remove it.
  frp->fr_parent->fr_layout = frp->fr_layout;
  frp->fr_parent->fr_child = frp->fr_child;
  frame_T *frp2;
  FOR_ALL_FRAMES(frp2, frp->fr_child) {
    frp2->fr_parent = frp->fr_parent;
  }
  frp->fr_parent->fr_win = frp->fr_win;
  if (frp->fr_win != NULL) {
    frp->fr_win->w_frame = frp->fr_parent;
  }
  frp2 = frp->fr_parent;
  if (topframe->fr_child == frp) {
    topframe->fr_child = frp2;
  }
  xfree(frp);

  frp = frp2->fr_parent;
  if (frp != NULL && frp->fr_layout == frp2->fr_layout) {
    // The frame above the parent has the same layout, have to merge
    // the frames into this list.
    if (frp->fr_child == frp2) {
      frp->fr_child = frp2->fr_child;
    }
    assert(frp2->fr_child);
    frp2->fr_child->fr_prev = frp2->fr_prev;
    if (frp2->fr_prev != NULL) {
      frp2->fr_prev->fr_next = frp2->fr_child;
    }
    for (frame_T *frp3 = frp2->fr_child;; frp3 = frp3->fr_next) {
      frp3->fr_parent = frp;
      if (frp3->fr_next == NULL) {
        frp3->fr_next = frp2->fr_next;
        if (frp2->fr_next != NULL) {
          frp2->fr_next->fr_prev = frp3;
        }
        break;
      }
    }
    if (topframe->fr_child == frp2) {
      topframe->fr_child = frp;
    }
    xfree(frp2);
  }
}

/// Undo changes from a prior call to winframe_remove, also restoring lost
/// vertical separators and statuslines, and changed window positions for
/// windows within "unflat_altfr".
/// Caller must ensure no other changes were made to the layout or window sizes!
void winframe_restore(win_T *wp, int dir, frame_T *unflat_altfr)
  FUNC_ATTR_NONNULL_ALL
{
  frame_T *frp = wp->w_frame;

  // Put "wp"'s frame back where it was.
  if (frp->fr_prev != NULL) {
    frame_append(frp->fr_prev, frp);
  } else {
    frame_insert(frp->fr_next, frp);
  }

  // Vertical separators to the left may have been lost.  Restore them.
  if (wp->w_vsep_width == 0 && frp->fr_parent->fr_layout == FR_ROW && frp->fr_prev != NULL) {
    frame_set_vsep(frp->fr_prev, true);
  }

  // Statuslines or horizontal separators above may have been lost.  Restore them.
  if (frp->fr_parent->fr_layout == FR_COL && frp->fr_prev != NULL) {
    if (global_stl_height() == 0 && wp->w_status_height == 0) {
      frame_add_statusline(frp->fr_prev);
    } else if (global_stl_height() > 0 && wp->w_hsep_height == 0) {
      frame_add_hsep(frp->fr_prev);
    }
  }

  // Restore the lost room that was redistributed to the altframe.  Also
  // adjusts window sizes to fit restored statuslines/separators, if needed.
  if (dir == 'v') {
    frame_new_height(unflat_altfr, unflat_altfr->fr_height - frp->fr_height,
                     unflat_altfr == frp->fr_next, false, false);
  } else if (dir == 'h') {
    frame_new_width(unflat_altfr, unflat_altfr->fr_width - frp->fr_width,
                    unflat_altfr == frp->fr_next, false);
  }

  // Recompute window positions within the parent frame to restore them.
  // Positions were unchanged if the altframe was adjacent and left/above.
  if (unflat_altfr != frp->fr_prev) {
    const win_T *const topleft = frame2win(frp->fr_parent);
    int row = topleft->w_winrow;
    int col = topleft->w_wincol;

    frame_comp_pos(frp->fr_parent, &row, &col);
  }
}

/// If 'splitbelow' or 'splitright' is set, the space goes above or to the left
/// by default.  Otherwise, the free space goes below or to the right.  The
/// result is that opening a window and then immediately closing it will
/// preserve the initial window layout.  The 'wfh' and 'wfw' settings are
/// respected when possible.
///
/// @param  tp  tab page "win" is in, NULL for current
///
/// @return a pointer to the frame that will receive the empty screen space that
/// is left over after "win" is closed.
static frame_T *win_altframe(win_T *win, tabpage_T *tp)
  FUNC_ATTR_NONNULL_ARG(1)
{
  assert(tp == NULL || tp != curtab);

  if (one_window(win, tp)) {
    return alt_tabpage()->tp_curwin->w_frame;
  }

  return rs_win_altframe(win);
}

// Return the tabpage that will be used if the current one is closed.
static tabpage_T *alt_tabpage(void)
{
  // Use the last accessed tab page, if possible.
  if ((tcl_flags & kOptTclFlagUselast) && valid_tabpage(lastused_tabpage)) {
    return lastused_tabpage;
  }

  // Use the next tab page, if possible.
  bool forward = curtab->tp_next != NULL
                 && ((tcl_flags & kOptTclFlagLeft) == 0 || curtab == first_tabpage);

  tabpage_T *tp;
  if (forward) {
    tp = curtab->tp_next;
  } else {
    // Use the previous tab page.
    for (tp = first_tabpage; tp->tp_next != curtab; tp = tp->tp_next) {}
  }

  return tp;
}

// Find the left-upper window in frame "frp".
win_T *frame2win(frame_T *frp)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_frame2win(frp);
}

/// Check that the frame "frp" contains the window "wp".
///
/// @param  frp  frame
/// @param  wp   window
static bool frame_has_win(const frame_T *frp, const win_T *wp)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ARG(1)
{
  return rs_frame_has_win((frame_T *)frp, (win_T *)wp) != 0;
}

/// Check if current window is at the bottom
/// Returns true if there are no windows below current window
static bool is_bottom_win(win_T *wp)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
  return rs_is_bottom_win(wp) != 0;
}

/// Set a new height for a frame.  Recursively sets the height for contained
/// frames and windows.  Caller must take care of positions.
///
/// @param topfirst  resize topmost contained frame first.
/// @param wfh       obey 'winfixheight' when there is a choice;
///                  may cause the height not to be set.
/// @param set_ch    set 'cmdheight' to resize topframe.
void frame_new_height(frame_T *topfrp, int height, bool topfirst, bool wfh, bool set_ch)
  FUNC_ATTR_NONNULL_ALL
{
  if (topfrp->fr_parent == NULL && set_ch) {
    // topframe: update the command line height, with side effects.
    OptInt new_ch = MAX(min_set_ch, p_ch + topfrp->fr_height - height);
    if (new_ch != p_ch) {
      const OptInt save_ch = min_set_ch;
      set_option_value(kOptCmdheight, NUMBER_OPTVAL(new_ch), 0);
      min_set_ch = save_ch;
    }
    height = (int)MIN(ROWS_AVAIL, height);
  }
  if (topfrp->fr_win != NULL) {
    // Simple case: just one window.
    win_T *wp = topfrp->fr_win;
    if (is_bottom_win(wp)) {
      wp->w_hsep_height = 0;
    }
    win_new_height(wp, height - wp->w_hsep_height - wp->w_status_height);
  } else if (topfrp->fr_layout == FR_ROW) {
    frame_T *frp;
    do {
      // All frames in this row get the same new height.
      FOR_ALL_FRAMES(frp, topfrp->fr_child) {
        frame_new_height(frp, height, topfirst, wfh, set_ch);
        if (frp->fr_height > height) {
          // Could not fit the windows, make the whole row higher.
          height = frp->fr_height;
          break;
        }
      }
    } while (frp != NULL);
  } else {  // fr_layout == FR_COL
    // Complicated case: Resize a column of frames.  Resize the bottom
    // frame first, frames above that when needed.

    frame_T *frp = topfrp->fr_child;
    if (wfh) {
      // Advance past frames with one window with 'wfh' set.
      while (frame_fixed_height(frp)) {
        frp = frp->fr_next;
        if (frp == NULL) {
          return;                   // no frame without 'wfh', give up
        }
      }
    }
    if (!topfirst) {
      // Find the bottom frame of this column
      while (frp->fr_next != NULL) {
        frp = frp->fr_next;
      }
      if (wfh) {
        // Advance back for frames with one window with 'wfh' set.
        while (frame_fixed_height(frp)) {
          frp = frp->fr_prev;
        }
      }
    }

    int extra_lines = height - topfrp->fr_height;
    if (extra_lines < 0) {
      // reduce height of contained frames, bottom or top frame first
      while (frp != NULL) {
        int h = frame_minheight(frp, NULL);
        if (frp->fr_height + extra_lines < h) {
          extra_lines += frp->fr_height - h;
          frame_new_height(frp, h, topfirst, wfh, set_ch);
        } else {
          frame_new_height(frp, frp->fr_height + extra_lines, topfirst, wfh, set_ch);
          break;
        }
        if (topfirst) {
          do {
            frp = frp->fr_next;
          } while (wfh && frp != NULL && frame_fixed_height(frp));
        } else {
          do {
            frp = frp->fr_prev;
          } while (wfh && frp != NULL && frame_fixed_height(frp));
        }
        // Increase "height" if we could not reduce enough frames.
        if (frp == NULL) {
          height -= extra_lines;
        }
      }
    } else if (extra_lines > 0) {
      // increase height of bottom or top frame
      frame_new_height(frp, frp->fr_height + extra_lines, topfirst, wfh, set_ch);
    }
  }
  topfrp->fr_height = height;
}

/// Return true if height of frame "frp" should not be changed because of
/// the 'winfixheight' option.
///
/// @param  frp  frame
///
/// @return true if the frame has a fixed height
static bool frame_fixed_height(frame_T *frp)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
  return rs_frame_fixed_height(frp) != 0;
}

/// Return true if width of frame "frp" should not be changed because of
/// the 'winfixwidth' option.
///
/// @param  frp  frame
///
/// @return true if the frame has a fixed width
static bool frame_fixed_width(frame_T *frp)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
  return rs_frame_fixed_width(frp) != 0;
}

// Add a status line to windows at the bottom of "frp".
// Note: Does not check if there is room!
static void frame_add_statusline(frame_T *frp)
{
  rs_frame_add_statusline(frp);
}

/// Set width of a frame.  Handles recursively going through contained frames.
/// May remove separator line for windows at the right side (for win_close()).
///
/// @param leftfirst  resize leftmost contained frame first.
/// @param wfw        obey 'winfixwidth' when there is a choice;
///                   may cause the width not to be set.
static void frame_new_width(frame_T *topfrp, int width, bool leftfirst, bool wfw)
{
  if (topfrp->fr_layout == FR_LEAF) {
    // Simple case: just one window.
    win_T *wp = topfrp->fr_win;
    // Find out if there are any windows right of this one.
    frame_T *frp;
    for (frp = topfrp; frp->fr_parent != NULL; frp = frp->fr_parent) {
      if (frp->fr_parent->fr_layout == FR_ROW && frp->fr_next != NULL) {
        break;
      }
    }
    if (frp->fr_parent == NULL) {
      wp->w_vsep_width = 0;
    }
    win_new_width(wp, width - wp->w_vsep_width);
  } else if (topfrp->fr_layout == FR_COL) {
    frame_T *frp;
    do {
      // All frames in this column get the same new width.
      FOR_ALL_FRAMES(frp, topfrp->fr_child) {
        frame_new_width(frp, width, leftfirst, wfw);
        if (frp->fr_width > width) {
          // Could not fit the windows, make whole column wider.
          width = frp->fr_width;
          break;
        }
      }
    } while (frp != NULL);
  } else {  // fr_layout == FR_ROW
    // Complicated case: Resize a row of frames.  Resize the rightmost
    // frame first, frames left of it when needed.

    frame_T *frp = topfrp->fr_child;
    if (wfw) {
      // Advance past frames with one window with 'wfw' set.
      while (frame_fixed_width(frp)) {
        frp = frp->fr_next;
        if (frp == NULL) {
          return;                   // no frame without 'wfw', give up
        }
      }
    }
    if (!leftfirst) {
      // Find the rightmost frame of this row
      while (frp->fr_next != NULL) {
        frp = frp->fr_next;
      }
      if (wfw) {
        // Advance back for frames with one window with 'wfw' set.
        while (frame_fixed_width(frp)) {
          frp = frp->fr_prev;
        }
      }
    }

    int extra_cols = width - topfrp->fr_width;
    if (extra_cols < 0) {
      // reduce frame width, rightmost frame first
      while (frp != NULL) {
        int w = frame_minwidth(frp, NULL);
        if (frp->fr_width + extra_cols < w) {
          extra_cols += frp->fr_width - w;
          frame_new_width(frp, w, leftfirst, wfw);
        } else {
          frame_new_width(frp, frp->fr_width + extra_cols,
                          leftfirst, wfw);
          break;
        }
        if (leftfirst) {
          do {
            frp = frp->fr_next;
          } while (wfw && frp != NULL && frame_fixed_width(frp));
        } else {
          do {
            frp = frp->fr_prev;
          } while (wfw && frp != NULL && frame_fixed_width(frp));
        }
        // Increase "width" if we could not reduce enough frames.
        if (frp == NULL) {
          width -= extra_cols;
        }
      }
    } else if (extra_cols > 0) {
      // increase width of rightmost frame
      frame_new_width(frp, frp->fr_width + extra_cols, leftfirst, wfw);
    }
  }
  topfrp->fr_width = width;
}

/// Add or remove the vertical separator of windows to the right side of "frp".
/// Note: Does not check if there is room!
static void frame_set_vsep(const frame_T *frp, bool add)
  FUNC_ATTR_NONNULL_ARG(1)
{
  rs_frame_set_vsep(frp, add ? 1 : 0);
}

/// Add the horizontal separator to windows at the bottom of "frp".
/// Note: Does not check if there is room or whether the windows have a statusline!
static void frame_add_hsep(const frame_T *frp)
  FUNC_ATTR_NONNULL_ARG(1)
{
  rs_frame_add_hsep(frp);
}

// Set frame width from the window it contains.
static void frame_fix_width(win_T *wp)
{
  rs_frame_fix_width(wp);
}

// Set frame height from the window it contains.
static void frame_fix_height(win_T *wp)
  FUNC_ATTR_NONNULL_ALL
{
  rs_frame_fix_height(wp);
}

/// Compute the minimal height for frame "topfrp". Uses the 'winminheight' option.
/// When "next_curwin" isn't NULL, use p_wh for this window.
/// When "next_curwin" is NOWIN, don't use at least one line for the current window.
static int frame_minheight(frame_T *topfrp, win_T *next_curwin)
{
  return rs_frame_minheight(topfrp, next_curwin);
}

/// Compute the minimal width for frame "topfrp".
/// When "next_curwin" isn't NULL, use p_wiw for this window.
/// When "next_curwin" is NOWIN, don't use at least one column for the current
/// window.
///
/// @param next_curwin  use p_wh and p_wiw for next_curwin
static int frame_minwidth(frame_T *topfrp, win_T *next_curwin)
{
  return rs_frame_minwidth(topfrp, next_curwin);
}

/// Try to close all windows except current one.
/// Buffers in the other windows become hidden if 'hidden' is set, or '!' is
/// used and the buffer was modified.
///
/// Used by ":bdel" and ":only".
///
/// @param forceit  always hide all other windows
void close_others(int message, int forceit)
{
  win_T *const old_curwin = curwin;

  if (curwin->w_floating) {
    if (message && !autocmd_busy) {
      emsg(e_floatonly);
    }
    return;
  }

  if (one_window(firstwin, NULL) && !lastwin->w_floating) {
    if (message && !autocmd_busy) {
      msg(_(m_onlyone), 0);
    }
    return;
  }

  // Be very careful here: autocommands may change the window layout.
  win_T *nextwp;
  for (win_T *wp = firstwin; win_valid(wp); wp = nextwp) {
    nextwp = wp->w_next;

    // autocommands messed this one up
    if (old_curwin != curwin && win_valid(old_curwin)) {
      curwin = old_curwin;
      curbuf = curwin->w_buffer;
    }

    if (wp == curwin) {                 // don't close current window
      continue;
    }

    // autoccommands messed this one up
    if (!buf_valid(wp->w_buffer) && win_valid(wp)) {
      wp->w_buffer = NULL;
      win_close(wp, false, false);
      continue;
    }
    // Check if it's allowed to abandon this window
    int r = can_abandon(wp->w_buffer, forceit);
    if (!win_valid(wp)) {             // autocommands messed wp up
      nextwp = firstwin;
      continue;
    }
    if (!r) {
      if (message && (p_confirm || (cmdmod.cmod_flags & CMOD_CONFIRM)) && p_write) {
        dialog_changed(wp->w_buffer, false);
        if (!win_valid(wp)) {                 // autocommands messed wp up
          nextwp = firstwin;
          continue;
        }
      }
      if (bufIsChanged(wp->w_buffer)) {
        continue;
      }
    }
    win_close(wp, !buf_hide(wp->w_buffer) && !bufIsChanged(wp->w_buffer), false);
  }

  if (message && !ONE_WINDOW) {
    emsg(_("E445: Other window contains changes"));
  }
}

/// Store the relevant window pointers for tab page "tp".  To be used before
/// use_tabpage().
void unuse_tabpage(tabpage_T *tp)
{
  tp->tp_topframe = topframe;
  tp->tp_firstwin = firstwin;
  tp->tp_lastwin = lastwin;
  tp->tp_curwin = curwin;
}

// When switching tabpage, handle other side-effects in command_height(), but
// avoid setting frame sizes which are still correct.
static bool command_frame_height = true;

/// Set the relevant pointers to use tab page "tp".  May want to call
/// unuse_tabpage() first.
void use_tabpage(tabpage_T *tp)
{
  curtab = tp;
  topframe = curtab->tp_topframe;
  firstwin = curtab->tp_firstwin;
  lastwin = curtab->tp_lastwin;
  curwin = curtab->tp_curwin;
}

// Allocate the first window and put an empty buffer in it.
// Only called from main().
void win_alloc_first(void)
{
  if (win_alloc_firstwin(NULL) == FAIL) {
    // allocating first buffer before any autocmds should not fail.
    abort();
  }

  first_tabpage = alloc_tabpage();
  curtab = first_tabpage;
  unuse_tabpage(first_tabpage);
}

// Init `aucmd_win[idx]`. This can only be done after the first window
// is fully initialized, thus it can't be in win_alloc_first().
void win_alloc_aucmd_win(int idx)
{
  Error err = ERROR_INIT;
  WinConfig fconfig = WIN_CONFIG_INIT;
  fconfig.width = Columns;
  fconfig.height = 5;
  fconfig.focusable = false;
  fconfig.mouse = false;
  aucmd_win[idx].auc_win = win_new_float(NULL, true, fconfig, &err);
  aucmd_win[idx].auc_win->w_buffer->b_nwindows--;
  RESET_BINDING(aucmd_win[idx].auc_win);
}

// Allocate the first window or the first window in a new tab page.
// When "oldwin" is NULL create an empty buffer for it.
// When "oldwin" is not NULL copy info from it to the new window.
// Return FAIL when something goes wrong (out of memory).
static int win_alloc_firstwin(win_T *oldwin)
{
  curwin = win_alloc(NULL, false);
  if (oldwin == NULL) {
    // Very first window, need to create an empty buffer for it and
    // initialize from scratch.
    curbuf = buflist_new(NULL, NULL, 1, BLN_LISTED);
    if (curbuf == NULL) {
      return FAIL;
    }
    curwin->w_buffer = curbuf;
    curwin->w_s = &(curbuf->b_s);
    curbuf->b_nwindows = 1;     // there is one window
    curwin->w_alist = &global_alist;
    curwin_init();              // init current window
  } else {
    // First window in new tab page, initialize it from "oldwin".
    win_init(curwin, oldwin, 0);

    // We don't want cursor- and scroll-binding in the first window.
    RESET_BINDING(curwin);
  }

  new_frame(curwin);
  topframe = curwin->w_frame;
  topframe->fr_width = Columns;
  topframe->fr_height = Rows - (int)p_ch - global_stl_height();

  return OK;
}

// Create a frame for window "wp".
static void new_frame(win_T *wp)
{
  frame_T *frp = xcalloc(1, sizeof(frame_T));

  wp->w_frame = frp;
  frp->fr_layout = FR_LEAF;
  frp->fr_win = wp;
}

// Initialize the window and frame size to the maximum.
void win_init_size(void)
{
  firstwin->w_height = (int)ROWS_AVAIL;
  firstwin->w_prev_height = (int)ROWS_AVAIL;
  firstwin->w_view_height = firstwin->w_height - firstwin->w_winbar_height;
  firstwin->w_height_outer = firstwin->w_height;
  firstwin->w_winrow_off = firstwin->w_winbar_height;
  topframe->fr_height = (int)ROWS_AVAIL;
  firstwin->w_width = Columns;
  firstwin->w_view_width = firstwin->w_width;
  firstwin->w_width_outer = firstwin->w_width;
  topframe->fr_width = Columns;
}

// Allocate a new tabpage_T and init the values.
static tabpage_T *alloc_tabpage(void)
{
  static int last_tp_handle = 0;
  tabpage_T *tp = xcalloc(1, sizeof(tabpage_T));
  tp->handle = ++last_tp_handle;
  pmap_put(int)(&tabpage_handles, tp->handle, tp);

  // Init t: variables.
  tp->tp_vars = tv_dict_alloc();
  init_var_dict(tp->tp_vars, &tp->tp_winvar, VAR_SCOPE);
  tp->tp_diff_invalid = true;
  tp->tp_ch_used = p_ch;

  return tp;
}

void free_tabpage(tabpage_T *tp)
{
  pmap_del(int)(&tabpage_handles, tp->handle, NULL);
  rs_diff_clear(tp);
  for (int idx = 0; idx < SNAP_COUNT; idx++) {
    clear_snapshot(tp, idx);
  }
  vars_clear(&tp->tp_vars->dv_hashtab);         // free all t: variables
  hash_init(&tp->tp_vars->dv_hashtab);
  unref_var_dict(tp->tp_vars);

  if (tp == lastused_tabpage) {
    lastused_tabpage = NULL;
  }

  xfree(tp->tp_localdir);
  xfree(tp->tp_prevdir);
  xfree(tp);
}

/// Create a new tabpage with one window.
///
/// It will edit the current buffer, like after :split.
///
/// @param after Put new tabpage after tabpage "after", or after the current
///              tabpage in case of 0.
/// @param filename Will be passed to apply_autocmds().
/// @return Was the new tabpage created successfully? FAIL or OK.
int win_new_tabpage(int after, char *filename)
{
  tabpage_T *old_curtab = curtab;

  if (cmdwin_type != 0) {
    emsg(_(e_cmdwin));
    return FAIL;
  }

  tabpage_T *newtp = alloc_tabpage();

  // Remember the current windows in this Tab page.
  if (leave_tabpage(curbuf, true) == FAIL) {
    xfree(newtp);
    return FAIL;
  }

  newtp->tp_localdir = old_curtab->tp_localdir
                       ? xstrdup(old_curtab->tp_localdir) : NULL;

  curtab = newtp;

  // Create a new empty window.
  if (win_alloc_firstwin(old_curtab->tp_curwin) == OK) {
    // Make the new Tab page the new topframe.
    if (after == 1) {
      // New tab page becomes the first one.
      newtp->tp_next = first_tabpage;
      first_tabpage = newtp;
    } else {
      tabpage_T *tp = old_curtab;

      if (after > 0) {
        // Put new tab page before tab page "after".
        int n = 2;
        for (tp = first_tabpage; tp->tp_next != NULL
             && n < after; tp = tp->tp_next) {
          n++;
        }
      }
      newtp->tp_next = tp->tp_next;
      tp->tp_next = newtp;
    }
    newtp->tp_firstwin = newtp->tp_lastwin = newtp->tp_curwin = curwin;

    win_init_size();
    firstwin->w_winrow = tabline_height();
    firstwin->w_prev_winrow = firstwin->w_winrow;
    win_comp_scroll(curwin);

    newtp->tp_topframe = topframe;
    last_status(false);

    if (curbuf->terminal) {
      terminal_check_size(curbuf->terminal);
    }

    redraw_all_later(UPD_NOT_VALID);

    tabpage_check_windows(old_curtab);

    lastused_tabpage = old_curtab;

    entering_window(curwin);

    apply_autocmds(EVENT_WINNEW, NULL, NULL, false, curbuf);
    apply_autocmds(EVENT_WINENTER, NULL, NULL, false, curbuf);
    apply_autocmds(EVENT_TABNEW, filename, filename, false, curbuf);
    apply_autocmds(EVENT_TABENTER, NULL, NULL, false, curbuf);

    return OK;
  }

  // Failed, get back the previous Tab page
  enter_tabpage(curtab, curbuf, true, true);
  return FAIL;
}

// Open a new tab page if ":tab cmd" was used.  It will edit the same buffer,
// like with ":split".
// Returns OK if a new tab page was created, FAIL otherwise.
static int may_open_tabpage(void)
{
  int n = (cmdmod.cmod_tab == 0) ? postponed_split_tab : cmdmod.cmod_tab;

  if (n == 0) {
    return FAIL;
  }

  cmdmod.cmod_tab = 0;         // reset it to avoid doing it twice
  postponed_split_tab = 0;
  int status = win_new_tabpage(n, NULL);
  if (status == OK) {
    apply_autocmds(EVENT_TABNEWENTERED, NULL, NULL, false, curbuf);
  }
  return status;
}

// Create up to "maxcount" tabpages with empty windows.
// Returns the number of resulting tab pages.
int make_tabpages(int maxcount)
{
  int count = maxcount;

  // Limit to 'tabpagemax' tabs.
  count = MIN(count, (int)p_tpm);

  // Don't execute autocommands while creating the tab pages.  Must do that
  // when putting the buffers in the windows.
  block_autocmds();

  int todo;
  for (todo = count - 1; todo > 0; todo--) {
    if (win_new_tabpage(0, NULL) == FAIL) {
      break;
    }
  }

  unblock_autocmds();

  // return actual number of tab pages
  return count - todo;
}

/// Check that tpc points to a valid tab page.
///
/// @param[in]  tpc  Tabpage to check.
bool valid_tabpage(tabpage_T *tpc) FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_valid_tabpage(tpc) != 0;
}

/// Returns true when `tpc` is valid and at least one window is valid.
int valid_tabpage_win(tabpage_T *tpc)
{
  return rs_valid_tabpage_win(tpc);
}

/// Close tabpage `tab`, assuming it has no windows in it.
/// There must be another tabpage or this will crash.
void close_tabpage(tabpage_T *tab)
{
  tabpage_T *ptp;

  if (tab == first_tabpage) {
    first_tabpage = tab->tp_next;
    ptp = first_tabpage;
  } else {
    for (ptp = first_tabpage; ptp != NULL && ptp->tp_next != tab;
         ptp = ptp->tp_next) {
      // do nothing
    }
    assert(ptp != NULL);
    ptp->tp_next = tab->tp_next;
  }

  goto_tabpage_tp(ptp, false, false);
  free_tabpage(tab);
}

// Find tab page "n" (first one is 1).  Returns NULL when not found.
tabpage_T *find_tabpage(int n)
{
  return rs_find_tabpage(n);
}

// Get index of tab page "tp".  First one has index 1.
// When not found returns number of tab pages plus one.
int tabpage_index(tabpage_T *ftp)
{
  return rs_tabpage_index(ftp);
}

/// Prepare for leaving the current tab page.
/// When autocommands change "curtab" we don't leave the tab page and return
/// FAIL.
/// Careful: When OK is returned need to get a new tab page very very soon!
///
/// @param new_curbuf              what is going to be the new curbuf,
///                                NULL if unknown.
/// @param trigger_leave_autocmds  when true trigger *Leave autocommands.
static int leave_tabpage(buf_T *new_curbuf, bool trigger_leave_autocmds)
{
  tabpage_T *tp = curtab;

  leaving_window(curwin);
  reset_VIsual_and_resel();     // stop Visual mode
  if (trigger_leave_autocmds) {
    if (new_curbuf != curbuf) {
      apply_autocmds(EVENT_BUFLEAVE, NULL, NULL, false, curbuf);
      if (curtab != tp) {
        return FAIL;
      }
    }
    apply_autocmds(EVENT_WINLEAVE, NULL, NULL, false, curbuf);
    if (curtab != tp) {
      return FAIL;
    }
    apply_autocmds(EVENT_TABLEAVE, NULL, NULL, false, curbuf);
    if (curtab != tp) {
      return FAIL;
    }
  }

  reset_dragwin();
  tp->tp_curwin = curwin;
  tp->tp_prevwin = prevwin;
  tp->tp_firstwin = firstwin;
  tp->tp_lastwin = lastwin;
  tp->tp_old_Rows_avail = ROWS_AVAIL;
  if (tp->tp_old_Columns != -1) {
    tp->tp_old_Columns = Columns;
  }
  firstwin = NULL;
  lastwin = NULL;
  return OK;
}

/// Start using tab page "tp".
/// Only to be used after leave_tabpage() or freeing the current tab page.
///
/// @param trigger_enter_autocmds  when true trigger *Enter autocommands.
/// @param trigger_leave_autocmds  when true trigger *Leave autocommands.
static void enter_tabpage(tabpage_T *tp, buf_T *old_curbuf, bool trigger_enter_autocmds,
                          bool trigger_leave_autocmds)
{
  int old_off = tp->tp_firstwin->w_winrow;
  win_T *next_prevwin = tp->tp_prevwin;
  tabpage_T *old_curtab = curtab;

  use_tabpage(tp);

  if (old_curtab != curtab) {
    tabpage_check_windows(old_curtab);
    if (p_ch != curtab->tp_ch_used) {
      // Use the stored value of p_ch, so that it can be different for each tab page.
      // Handle other side-effects but avoid setting frame sizes, which are still correct.
      OptInt new_ch = curtab->tp_ch_used;
      curtab->tp_ch_used = p_ch;
      command_frame_height = false;
      set_option_value(kOptCmdheight, NUMBER_OPTVAL(new_ch), 0);
      command_frame_height = true;
    }
  }

  // We would like doing the TabEnter event first, but we don't have a
  // valid current window yet, which may break some commands.
  // This triggers autocommands, thus may make "tp" invalid.
  win_enter_ext(tp->tp_curwin, WEE_CURWIN_INVALID
                | (trigger_enter_autocmds ? WEE_TRIGGER_ENTER_AUTOCMDS : 0)
                | (trigger_leave_autocmds ? WEE_TRIGGER_LEAVE_AUTOCMDS : 0));
  prevwin = next_prevwin;

  last_status(false);  // status line may appear or disappear
  win_float_update_statusline();
  win_comp_pos();      // recompute w_winrow for all windows
  diff_need_scrollbind = true;

  // If there was a click in a window, it won't be usable for a following
  // drag.
  reset_dragwin();

  // The tabpage line may have appeared or disappeared, may need to resize the frames for that.
  // When the Vim window was resized or ROWS_AVAIL changed need to update frame sizes too.
  if (curtab->tp_old_Rows_avail != ROWS_AVAIL || (old_off != firstwin->w_winrow)) {
    win_new_screen_rows();
  }
  if (curtab->tp_old_Columns != Columns) {
    if (starting == 0) {
      win_new_screen_cols();  // update window widths
      curtab->tp_old_Columns = Columns;
    } else {
      curtab->tp_old_Columns = -1;  // update window widths later
    }
  }

  lastused_tabpage = old_curtab;

  // Apply autocommands after updating the display, when 'rows' and
  // 'columns' have been set correctly.
  if (trigger_enter_autocmds) {
    apply_autocmds(EVENT_TABENTER, NULL, NULL, false, curbuf);
    if (old_curbuf != curbuf) {
      apply_autocmds(EVENT_BUFENTER, NULL, NULL, false, curbuf);
    }
  }

  redraw_all_later(UPD_NOT_VALID);
}

/// tells external UI that windows and inline floats in old_curtab are invisible
/// and that floats in curtab is now visible.
///
/// External floats are considered independent of tabpages. This is
/// implemented by always moving them to curtab.
static void tabpage_check_windows(tabpage_T *old_curtab)
{
  win_T *next_wp;
  for (win_T *wp = old_curtab->tp_firstwin; wp; wp = next_wp) {
    next_wp = wp->w_next;
    if (wp->w_floating) {
      if (wp->w_config.external) {
        win_remove(wp, old_curtab);
        win_append(lastwin_nofloating(), wp, NULL);
      } else {
        ui_comp_remove_grid(&wp->w_grid_alloc);
      }
    }
    wp->w_pos_changed = true;
  }

  for (win_T *wp = firstwin; wp; wp = wp->w_next) {
    if (wp->w_floating && !wp->w_config.external) {
      win_config_float(wp, wp->w_config);
    }
    wp->w_pos_changed = true;
  }
}

// Go to tab page "n".  For ":tab N" and "Ngt".
// When "n" is 9999 go to the last tab page.
void goto_tabpage(int n)
{
  if (text_locked()) {
    // Not allowed when editing the command line.
    text_locked_msg();
    return;
  }

  // If there is only one it can't work.
  if (first_tabpage->tp_next == NULL) {
    if (n > 1) {
      beep_flush();
    }
    return;
  }

  tabpage_T *tp = NULL;  // shut up compiler

  if (n == 0) {
    // No count, go to next tab page, wrap around end.
    if (curtab->tp_next == NULL) {
      tp = first_tabpage;
    } else {
      tp = curtab->tp_next;
    }
  } else if (n < 0) {
    // "gT": go to previous tab page, wrap around end.  "N gT" repeats
    // this N times.
    tabpage_T *ttp = curtab;
    for (int i = n; i < 0; i++) {
      for (tp = first_tabpage; tp->tp_next != ttp && tp->tp_next != NULL;
           tp = tp->tp_next) {}
      ttp = tp;
    }
  } else if (n == 9999) {
    // Go to last tab page.
    for (tp = first_tabpage; tp->tp_next != NULL; tp = tp->tp_next) {}
  } else {
    // Go to tab page "n".
    tp = find_tabpage(n);
    if (tp == NULL) {
      beep_flush();
      return;
    }
  }

  goto_tabpage_tp(tp, true, true);
}

/// Go to tabpage "tp".
/// Note: doesn't update the GUI tab.
///
/// @param trigger_enter_autocmds  when true trigger *Enter autocommands.
/// @param trigger_leave_autocmds  when true trigger *Leave autocommands.
void goto_tabpage_tp(tabpage_T *tp, bool trigger_enter_autocmds, bool trigger_leave_autocmds)
{
  if (trigger_enter_autocmds || trigger_leave_autocmds) {
    if (cmdwin_type != 0) {
      emsg(_(e_cmdwin));
      return;
    }
  }

  // Don't repeat a message in another tab page.
  set_keep_msg(NULL, 0);

  skip_win_fix_scroll = true;
  if (tp != curtab && leave_tabpage(tp->tp_curwin->w_buffer,
                                    trigger_leave_autocmds) == OK) {
    if (valid_tabpage(tp)) {
      enter_tabpage(tp, curbuf, trigger_enter_autocmds,
                    trigger_leave_autocmds);
    } else {
      enter_tabpage(curtab, curbuf, trigger_enter_autocmds,
                    trigger_leave_autocmds);
    }
  }
  skip_win_fix_scroll = false;
}

/// Go to the last accessed tab page, if there is one.
/// @return true if the tab page is valid, false otherwise.
bool goto_tabpage_lastused(void)
{
  if (!valid_tabpage(lastused_tabpage)) {
    return false;
  }

  goto_tabpage_tp(lastused_tabpage, true, true);
  return true;
}

// Enter window "wp" in tab page "tp".
// Also updates the GUI tab.
void goto_tabpage_win(tabpage_T *tp, win_T *wp)
{
  goto_tabpage_tp(tp, true, true);
  if (curtab == tp && win_valid(wp)) {
    win_enter(wp, true);
  }
}

// Move the current tab page to after tab page "nr".
void tabpage_move(int nr)
{
  assert(curtab != NULL);

  if (first_tabpage->tp_next == NULL) {
    return;
  }

  if (tabpage_move_disallowed) {
    return;
  }

  int n = 1;
  tabpage_T *tp;

  for (tp = first_tabpage; tp->tp_next != NULL && n < nr; tp = tp->tp_next) {
    n++;
  }

  if (tp == curtab || (nr > 0 && tp->tp_next != NULL
                       && tp->tp_next == curtab)) {
    return;
  }

  tabpage_T *tp_dst = tp;

  // Remove the current tab page from the list of tab pages.
  if (curtab == first_tabpage) {
    first_tabpage = curtab->tp_next;
  } else {
    tp = NULL;
    FOR_ALL_TABS(tp2) {
      if (tp2->tp_next == curtab) {
        tp = tp2;
        break;
      }
    }
    if (tp == NULL) {   // "cannot happen"
      return;
    }
    tp->tp_next = curtab->tp_next;
  }

  // Re-insert it at the specified position.
  if (nr <= 0) {
    curtab->tp_next = first_tabpage;
    first_tabpage = curtab;
  } else {
    curtab->tp_next = tp_dst->tp_next;
    tp_dst->tp_next = curtab;
  }

  // Need to redraw the tabline.  Tab page contents doesn't change.
  redraw_tabline = true;
}

/// Go to another window.
/// When jumping to another buffer, stop Visual mode.  Do this before
/// changing windows so we can yank the selection into the '*' register.
/// (note: this may trigger ModeChanged autocommand!)
/// When jumping to another window on the same buffer, adjust its cursor
/// position to keep the same Visual area.
void win_goto(win_T *wp)
{
  win_T *owp = curwin;

  if (text_or_buf_locked()) {
    beep_flush();
    return;
  }

  if (wp->w_buffer != curbuf) {
    // careful: triggers ModeChanged autocommand
    reset_VIsual_and_resel();
  } else if (VIsual_active) {
    wp->w_cursor = curwin->w_cursor;
  }

  // autocommand may have made wp invalid
  if (!win_valid(wp)) {
    return;
  }

  win_enter(wp, true);

  // Conceal cursor line in previous window, unconceal in current window.
  if (win_valid(owp) && owp->w_p_cole > 0 && !msg_scrolled) {
    redrawWinline(owp, owp->w_cursor.lnum);
  }
  if (curwin->w_p_cole > 0 && !msg_scrolled) {
    redrawWinline(curwin, curwin->w_cursor.lnum);
  }
}

// Find the tabpage for window "win".
tabpage_T *win_find_tabpage(win_T *win)
{
  return rs_win_find_tabpage(win);
}

/// Get the above or below neighbor window of the specified window.
///
/// Returns the specified window if the neighbor is not found.
/// Returns the previous window if the specifiecied window is a floating window.
///
/// @param up     true for the above neighbor
/// @param count  nth neighbor window
///
/// @return       found window
win_T *win_vert_neighbor(tabpage_T *tp, win_T *wp, bool up, int count)
{
  return rs_win_vert_neighbor(tp, wp, up ? 1 : 0, count);
}

/// Move to window above or below "count" times.
///
/// @param up     true to go to win above
/// @param count  go count times into direction
static void win_goto_ver(bool up, int count)
{
  win_T *win = win_vert_neighbor(curtab, curwin, up, count);
  if (win != NULL) {
    win_goto(win);
  }
}

/// Get the left or right neighbor window of the specified window.
///
/// Returns the specified window if the neighbor is not found.
/// Returns the previous window if the specifiecied window is a floating window.
///
/// @param left  true for the left neighbor
/// @param count nth neighbor window
///
/// @return      found window
win_T *win_horz_neighbor(tabpage_T *tp, win_T *wp, bool left, int count)
{
  return rs_win_horz_neighbor(tp, wp, left ? 1 : 0, count);
}

/// Move to left or right window.
///
/// @param left   true to go to left window
/// @param count  go count times into direction
static void win_goto_hor(bool left, int count)
{
  win_T *win = win_horz_neighbor(curtab, curwin, left, count);
  if (win != NULL) {
    win_goto(win);
  }
}

/// Make window `wp` the current window.
///
/// @warning Autocmds may close the window immediately, so caller must check
///          win_valid(wp).
void win_enter(win_T *wp, bool undo_sync)
{
  win_enter_ext(wp, (undo_sync ? WEE_UNDO_SYNC : 0)
                | WEE_TRIGGER_ENTER_AUTOCMDS | WEE_TRIGGER_LEAVE_AUTOCMDS);
}

/// Make window "wp" the current window.
///
/// @param flags  if contains WEE_CURWIN_INVALID, it means curwin has just been
///               closed and isn't valid.
static void win_enter_ext(win_T *const wp, const int flags)
{
  bool other_buffer = false;
  const bool curwin_invalid = (flags & WEE_CURWIN_INVALID);

  if (wp == curwin && !curwin_invalid) {        // nothing to do
    return;
  }

  if (!curwin_invalid) {
    leaving_window(curwin);
  }

  if (!curwin_invalid && (flags & WEE_TRIGGER_LEAVE_AUTOCMDS)) {
    // Be careful: If autocommands delete the window, return now.
    if (wp->w_buffer != curbuf) {
      apply_autocmds(EVENT_BUFLEAVE, NULL, NULL, false, curbuf);
      other_buffer = true;
      if (!win_valid(wp)) {
        return;
      }
    }
    apply_autocmds(EVENT_WINLEAVE, NULL, NULL, false, curbuf);
    if (!win_valid(wp)) {
      return;
    }
    // autocmds may abort script processing
    if (aborting()) {
      return;
    }
  }

  // sync undo before leaving the current buffer
  if ((flags & WEE_UNDO_SYNC) && curbuf != wp->w_buffer) {
    u_sync(false);
  }

  // Might need to scroll the old window before switching, e.g., when the
  // cursor was moved.
  if (*p_spk == 'c' && !curwin_invalid) {
    update_topline(curwin);
  }

  // may have to copy the buffer options when 'cpo' contains 'S'
  if (wp->w_buffer != curbuf) {
    buf_copy_options(wp->w_buffer, BCO_ENTER | BCO_NOHELP);
  }
  if (!curwin_invalid) {
    prevwin = curwin;           // remember for CTRL-W p
    curwin->w_redr_status = true;
  }
  curwin = wp;
  curbuf = wp->w_buffer;

  check_cursor(curwin);
  if (!virtual_active(curwin)) {
    curwin->w_cursor.coladd = 0;
  }
  if (*p_spk == 'c') {
    changed_line_abv_curs();      // assume cursor position needs updating
  } else {
    // Make sure the cursor position is valid, either by moving the cursor
    // or by scrolling the text.
    win_fix_cursor(get_real_state() & (MODE_NORMAL|MODE_CMDLINE|MODE_TERMINAL));
  }

  win_fix_current_dir();

  entering_window(curwin);
  // Careful: autocommands may close the window and make "wp" invalid
  if (flags & WEE_TRIGGER_NEW_AUTOCMDS) {
    apply_autocmds(EVENT_WINNEW, NULL, NULL, false, curbuf);
  }
  if (flags & WEE_TRIGGER_ENTER_AUTOCMDS) {
    apply_autocmds(EVENT_WINENTER, NULL, NULL, false, curbuf);
    if (other_buffer) {
      apply_autocmds(EVENT_BUFENTER, NULL, NULL, false, curbuf);
    }
  }

  maketitle();
  curwin->w_redr_status = true;
  redraw_tabline = true;
  if (restart_edit) {
    redraw_later(curwin, UPD_VALID);  // causes status line redraw
  }

  // change background color according to NormalNC,
  // but only if actually defined (otherwise no extra redraw)
  if (curwin->w_hl_attr_normal != curwin->w_hl_attr_normalnc) {
    // TODO(bfredl): eventually we should be smart enough
    // to only recompose the window, not redraw it.
    redraw_later(curwin, UPD_NOT_VALID);
  }
  if (prevwin) {
    if (prevwin->w_hl_attr_normal != prevwin->w_hl_attr_normalnc) {
      redraw_later(prevwin, UPD_NOT_VALID);
    }
  }

  // set window height to desired minimal value
  if (curwin->w_height < p_wh && !curwin->w_p_wfh && !curwin->w_floating) {
    win_setheight((int)p_wh);
  } else if (curwin->w_height == 0) {
    win_setheight(1);
  }

  // set window width to desired minimal value
  if (curwin->w_width < p_wiw && !curwin->w_p_wfw && !curwin->w_floating) {
    win_setwidth((int)p_wiw);
  }

  setmouse();                   // in case jumped to/from help buffer

  // Change directories when the 'acd' option is set.
  do_autochdir();
}

/// Used after making another window the current one: change directory if needed.
void win_fix_current_dir(void)
{
  // New directory is either the local directory of the window, tab or NULL.
  char *new_dir = curwin->w_localdir ? curwin->w_localdir : curtab->tp_localdir;
  char cwd[MAXPATHL];
  if (os_dirname(cwd, MAXPATHL) != OK) {
    cwd[0] = NUL;
  }

  if (new_dir) {
    // Window/tab has a local directory: Save current directory as global
    // (unless that was done already) and change to the local directory.
    if (globaldir == NULL) {
      if (cwd[0] != NUL) {
        globaldir = xstrdup(cwd);
      }
    }
    bool dir_differs = pathcmp(new_dir, cwd, -1) != 0;
    if (!p_acd && dir_differs) {
      do_autocmd_dirchanged(new_dir, curwin->w_localdir ? kCdScopeWindow : kCdScopeTabpage,
                            kCdCauseWindow, true);
    }
    if (os_chdir(new_dir) == 0) {
      if (!p_acd && dir_differs) {
        do_autocmd_dirchanged(new_dir, curwin->w_localdir ? kCdScopeWindow : kCdScopeTabpage,
                              kCdCauseWindow, false);
      }
    }
    last_chdir_reason = NULL;
    shorten_fnames(true);
  } else if (globaldir != NULL) {
    // Window doesn't have a local directory and we are not in the global
    // directory: Change to the global directory.
    bool dir_differs = pathcmp(globaldir, cwd, -1) != 0;
    if (!p_acd && dir_differs) {
      do_autocmd_dirchanged(globaldir, kCdScopeGlobal, kCdCauseWindow, true);
    }
    if (os_chdir(globaldir) == 0) {
      if (!p_acd && dir_differs) {
        do_autocmd_dirchanged(globaldir, kCdScopeGlobal, kCdCauseWindow, false);
      }
    }
    XFREE_CLEAR(globaldir);
    last_chdir_reason = NULL;
    shorten_fnames(true);
  }
}

/// Jump to the first open window that contains buffer "buf", if one exists.
/// Returns a pointer to the window found, otherwise NULL.
win_T *buf_jump_open_win(buf_T *buf)
{
  if (curwin->w_buffer == buf) {
    win_enter(curwin, false);
    return curwin;
  }
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (wp->w_buffer == buf) {
      win_enter(wp, false);
      return wp;
    }
  }

  return NULL;
}

/// Jump to the first open window in any tab page that contains buffer "buf",
/// if one exists. First search in the windows present in the current tab page.
/// @return the found window, or NULL.
win_T *buf_jump_open_tab(buf_T *buf)
{
  // First try the current tab page.
  {
    win_T *wp = buf_jump_open_win(buf);
    if (wp != NULL) {
      return wp;
    }
  }

  FOR_ALL_TABS(tp) {
    // Skip the current tab since we already checked it.
    if (tp == curtab) {
      continue;
    }
    FOR_ALL_WINDOWS_IN_TAB(wp, tp) {
      if (wp->w_buffer == buf) {
        goto_tabpage_win(tp, wp);

        // If we the current window didn't switch,
        // something went wrong.
        if (curwin != wp) {
          wp = NULL;
        }

        // Return the window we switched to.
        return wp;
      }
    }
  }

  // If we made it this far, we didn't find the buffer.
  return NULL;
}

static int last_win_id = LOWEST_WIN_ID - 1;

/// Get the last_win_id global (accessor for Rust).
int nvim_get_last_win_id(void)
{
  return last_win_id;
}

/// @param hidden  allocate a window structure and link it in the window if
//                 false.
win_T *win_alloc(win_T *after, bool hidden)
{
  // allocate window structure and linesizes arrays
  win_T *new_wp = xcalloc(1, sizeof(win_T));

  new_wp->handle = ++last_win_id;
  pmap_put(int)(&window_handles, new_wp->handle, new_wp);

  new_wp->w_grid_alloc.mouse_enabled = true;

  grid_assign_handle(&new_wp->w_grid_alloc);

  // Init w: variables.
  new_wp->w_vars = tv_dict_alloc();
  init_var_dict(new_wp->w_vars, &new_wp->w_winvar, VAR_SCOPE);

  // Don't execute autocommands while the window is not properly
  // initialized yet.  gui_create_scrollbar() may trigger a FocusGained
  // event.
  block_autocmds();
  // link the window in the window list
  if (!hidden) {
    tabpage_T *tp = NULL;
    if (after) {
      tp = win_find_tabpage(after);
      if (tp == curtab) {
        tp = NULL;
      }
    }
    win_append(after, new_wp, tp);
  }

  new_wp->w_wincol = 0;
  new_wp->w_width = Columns;

  // position the display and the cursor at the top of the file.
  new_wp->w_topline = 1;
  new_wp->w_topfill = 0;
  new_wp->w_botline = 2;
  new_wp->w_cursor.lnum = 1;
  new_wp->w_scbind_pos = 1;
  new_wp->w_floating = 0;
  new_wp->w_config = WIN_CONFIG_INIT;
  new_wp->w_viewport_invalid = true;
  new_wp->w_viewport_last_topline = 1;

  new_wp->w_ns_hl = -1;

  Set(uint32_t) ns_set = SET_INIT;
  new_wp->w_ns_set = ns_set;

  // use global option for global-local options
  new_wp->w_allbuf_opt.wo_so = new_wp->w_p_so = -1;
  new_wp->w_allbuf_opt.wo_siso = new_wp->w_p_siso = -1;

  // We won't calculate w_fraction until resizing the window
  new_wp->w_fraction = 0;
  new_wp->w_prev_fraction_row = -1;

  foldInitWin(new_wp);
  unblock_autocmds();
  new_wp->w_next_match_id = 1000;  // up to 1000 can be picked by the user
  return new_wp;
}

// Free one WinInfo.
void free_wininfo(WinInfo *wip, buf_T *bp)
{
  if (wip->wi_optset) {
    clear_winopt(&wip->wi_opt);
    deleteFoldRecurse(bp, &wip->wi_folds);
  }
  xfree(wip);
}

/// Remove window 'wp' from the window list and free the structure.
///
/// @param tp  tab page "win" is in, NULL for current
void win_free(win_T *wp, tabpage_T *tp)
{
  pmap_del(int)(&window_handles, wp->handle, NULL);
  clearFolding(wp);

  // reduce the reference count to the argument list.
  alist_unlink(wp->w_alist);

  // Don't execute autocommands while the window is halfway being deleted.
  block_autocmds();

  set_destroy(uint32_t, &wp->w_ns_set);

  clear_winopt(&wp->w_onebuf_opt);
  clear_winopt(&wp->w_allbuf_opt);

  xfree(wp->w_p_lcs_chars.multispace);
  xfree(wp->w_p_lcs_chars.leadmultispace);

  vars_clear(&wp->w_vars->dv_hashtab);          // free all w: variables
  hash_init(&wp->w_vars->dv_hashtab);
  unref_var_dict(wp->w_vars);

  if (prevwin == wp) {
    prevwin = NULL;
  }
  FOR_ALL_TABS(ttp) {
    if (ttp->tp_prevwin == wp) {
      ttp->tp_prevwin = NULL;
    }
  }

  xfree(wp->w_lines);

  for (int i = 0; i < wp->w_tagstacklen; i++) {
    tagstack_clear_entry(&wp->w_tagstack[i]);
  }

  xfree(wp->w_localdir);
  xfree(wp->w_prevdir);

  stl_clear_click_defs(wp->w_status_click_defs, wp->w_status_click_defs_size);
  xfree(wp->w_status_click_defs);

  stl_clear_click_defs(wp->w_winbar_click_defs, wp->w_winbar_click_defs_size);
  xfree(wp->w_winbar_click_defs);

  stl_clear_click_defs(wp->w_statuscol_click_defs, wp->w_statuscol_click_defs_size);
  xfree(wp->w_statuscol_click_defs);

  // Remove the window from the b_wininfo lists, it may happen that the
  // freed memory is re-used for another window.
  FOR_ALL_BUFFERS(buf) {
    WinInfo *wip_wp = NULL;
    size_t pos_wip = kv_size(buf->b_wininfo);
    size_t pos_null = kv_size(buf->b_wininfo);
    for (size_t i = 0; i < kv_size(buf->b_wininfo); i++) {
      WinInfo *wip = kv_A(buf->b_wininfo, i);
      if (wip->wi_win == wp) {
        wip_wp = wip;
        pos_wip = i;
      } else if (wip->wi_win == NULL) {
        pos_null = i;
      }
    }

    if (wip_wp) {
      wip_wp->wi_win = NULL;
      // If there already is an entry with "wi_win" set to NULL, only
      // the first entry with NULL will ever be used, delete the other one.
      if (pos_null < kv_size(buf->b_wininfo)) {
        size_t pos_delete = MAX(pos_null, pos_wip);
        free_wininfo(kv_A(buf->b_wininfo, pos_delete), buf);
        kv_shift(buf->b_wininfo, pos_delete, 1);
      }
    }
  }

  // free the border text
  clear_virttext(&wp->w_config.title_chunks);
  clear_virttext(&wp->w_config.footer_chunks);

  clear_matches(wp);

  free_jumplist(wp);

  qf_free_all(wp);

  xfree(wp->w_p_cc_cols);

  win_free_grid(wp, false);

  if (win_valid_any_tab(wp)) {
    win_remove(wp, tp);
  }
  if (autocmd_busy) {
    wp->w_next = au_pending_free_win;
    au_pending_free_win = wp;
  } else {
    xfree(wp);
  }

  unblock_autocmds();
}

void win_free_grid(win_T *wp, bool reinit)
{
  if (wp->w_grid_alloc.handle != 0 && ui_has(kUIMultigrid)) {
    ui_call_grid_destroy(wp->w_grid_alloc.handle);
  }
  grid_free(&wp->w_grid_alloc);
  if (reinit) {
    // if a float is turned into a split, the grid data structure will be reused
    CLEAR_FIELD(wp->w_grid_alloc);
  }
}

/// Append window "wp" in the window list after window "after".
///
/// @param tp  tab page "win" (and "after", if not NULL) is in, NULL for current
void win_append(win_T *after, win_T *wp, tabpage_T *tp)
  FUNC_ATTR_NONNULL_ARG(2)
{
  assert(tp == NULL || tp != curtab);
  rs_win_append(after, wp, tp);
}

/// Remove a window from the window list.
///
/// @param tp  tab page "win" is in, NULL for current
void win_remove(win_T *wp, tabpage_T *tp)
  FUNC_ATTR_NONNULL_ARG(1)
{
  assert(tp == NULL || tp != curtab);
  rs_win_remove(wp, tp);
}

// Append frame "frp" in a frame list after frame "after".
static void frame_append(frame_T *after, frame_T *frp)
{
  rs_frame_append(after, frp);
}

// Insert frame "frp" in a frame list before frame "before".
static void frame_insert(frame_T *before, frame_T *frp)
{
  rs_frame_insert(before, frp);
}

// Remove a frame from a frame list.
static void frame_remove(frame_T *frp)
{
  rs_frame_remove(frp);
}

void win_new_screensize(void)
{
  static int old_Rows = 0;
  static int old_Columns = 0;

  if (old_Rows != Rows) {
    // If 'window' uses the whole screen, keep it using that.
    // Don't change it when set with "-w size" on the command line.
    if (p_window == old_Rows - 1 || (old_Rows == 0 && !option_was_set(kOptWindow))) {
      p_window = Rows - 1;
    }
    old_Rows = Rows;
    win_new_screen_rows();  // update window sizes
  }
  if (old_Columns != Columns) {
    old_Columns = Columns;
    win_new_screen_cols();  // update window sizes
  }
}
/// Called from win_new_screensize() after Rows changed.
///
/// This only does the current tab page, others must be done when made active.
void win_new_screen_rows(void)
{
  if (firstwin == NULL) {       // not initialized yet
    return;
  }
  int h = MAX((int)ROWS_AVAIL, frame_minheight(topframe, NULL));

  // First try setting the heights of windows with 'winfixheight'.  If
  // that doesn't result in the right height, forget about that option.
  frame_new_height(topframe, h, false, true, false);
  if (!frame_check_height(topframe, h)) {
    frame_new_height(topframe, h, false, false, false);
  }

  win_comp_pos();  // recompute w_winrow and w_wincol
  win_reconfig_floats();  // The size of floats might change
  compute_cmdrow();
  curtab->tp_ch_used = p_ch;

  if (!skip_win_fix_scroll) {
    win_fix_scroll(true);
  }
}

/// Called from win_new_screensize() after Columns changed.
void win_new_screen_cols(void)
{
  if (firstwin == NULL) {       // not initialized yet
    return;
  }

  // First try setting the widths of windows with 'winfixwidth'.  If that
  // doesn't result in the right width, forget about that option.
  frame_new_width(topframe, Columns, false, true);
  if (!frame_check_width(topframe, Columns)) {
    frame_new_width(topframe, Columns, false, false);
  }

  win_comp_pos();  // recompute w_winrow and w_wincol
  win_reconfig_floats();  // The size of floats might change
}

/// Make a snapshot of all the window scroll positions and sizes of the current
/// tab page.
void snapshot_windows_scroll_size(void)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    wp->w_last_topline = wp->w_topline;
    wp->w_last_topfill = wp->w_topfill;
    wp->w_last_leftcol = wp->w_leftcol;
    wp->w_last_skipcol = wp->w_skipcol;
    wp->w_last_width = wp->w_width;
    wp->w_last_height = wp->w_height;
  }
}

static bool did_initial_scroll_size_snapshot = false;

void may_make_initial_scroll_size_snapshot(void)
{
  if (!did_initial_scroll_size_snapshot) {
    did_initial_scroll_size_snapshot = true;
    snapshot_windows_scroll_size();
  }
}

/// Create a dictionary with information about size and scroll changes in a
/// window.
/// Returns the dictionary with refcount set to one.
/// Returns NULL on internal error.
static dict_T *make_win_info_dict(int width, int height, int topline, int topfill, int leftcol,
                                  int skipcol)
{
  dict_T *const d = tv_dict_alloc();
  d->dv_refcount = 1;

  // not actually looping, for breaking out on error
  while (true) {
    typval_T tv = {
      .v_lock = VAR_UNLOCKED,
      .v_type = VAR_NUMBER,
    };

    tv.vval.v_number = width;
    if (tv_dict_add_tv(d, S_LEN("width"), &tv) == FAIL) {
      break;
    }
    tv.vval.v_number = height;
    if (tv_dict_add_tv(d, S_LEN("height"), &tv) == FAIL) {
      break;
    }
    tv.vval.v_number = topline;
    if (tv_dict_add_tv(d, S_LEN("topline"), &tv) == FAIL) {
      break;
    }
    tv.vval.v_number = topfill;
    if (tv_dict_add_tv(d, S_LEN("topfill"), &tv) == FAIL) {
      break;
    }
    tv.vval.v_number = leftcol;
    if (tv_dict_add_tv(d, S_LEN("leftcol"), &tv) == FAIL) {
      break;
    }
    tv.vval.v_number = skipcol;
    if (tv_dict_add_tv(d, S_LEN("skipcol"), &tv) == FAIL) {
      break;
    }
    return d;
  }
  tv_dict_unref(d);
  return NULL;
}

/// This function is used for three purposes:
/// 1. Goes over all windows in the current tab page and sets:
///    "size_count" to the nr of windows with size changes.
///    "first_scroll_win" to the first window with any relevant changes.
///    "first_size_win" to the first window with size changes.
///
/// 2. When the first three arguments are NULL and "winlist" is not NULL,
///    "winlist" is set to the list of window IDs with size changes.
///
/// 3. When the first three arguments are NULL and "v_event" is not NULL,
///    information about changed windows is added to "v_event".
static void check_window_scroll_resize(int *size_count, win_T **first_scroll_win,
                                       win_T **first_size_win, list_T *winlist, dict_T *v_event)
{
  // int listidx = 0;
  int tot_width = 0;
  int tot_height = 0;
  int tot_topline = 0;
  int tot_topfill = 0;
  int tot_leftcol = 0;
  int tot_skipcol = 0;

  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    // Skip floating windows that do not have a snapshot (usually because they are newly-created),
    // as unlike split windows, creating floating windows doesn't cause other windows to resize.
    if (wp->w_floating && wp->w_last_topline == 0) {
      wp->w_last_topline = wp->w_topline;
      wp->w_last_topfill = wp->w_topfill;
      wp->w_last_leftcol = wp->w_leftcol;
      wp->w_last_skipcol = wp->w_skipcol;
      wp->w_last_width = wp->w_width;
      wp->w_last_height = wp->w_height;
      continue;
    }

    const bool ignore_scroll = event_ignored(EVENT_WINSCROLLED, wp->w_p_eiw);
    const bool size_changed = !event_ignored(EVENT_WINRESIZED, wp->w_p_eiw)
                              && (wp->w_last_width != wp->w_width
                                  || wp->w_last_height != wp->w_height);
    if (size_changed) {
      if (winlist != NULL) {
        // Add this window to the list of changed windows.
        typval_T tv = {
          .v_lock = VAR_UNLOCKED,
          .v_type = VAR_NUMBER,
          .vval.v_number = wp->handle,
        };
        // tv_list_set_item(winlist, listidx++, &tv);
        tv_list_append_owned_tv(winlist, tv);
      } else if (size_count != NULL) {
        assert(first_size_win != NULL && first_scroll_win != NULL);
        (*size_count)++;
        if (*first_size_win == NULL) {
          *first_size_win = wp;
        }
        // For WinScrolled the first window with a size change is used
        // even when it didn't scroll.
        if (*first_scroll_win == NULL && !ignore_scroll) {
          *first_scroll_win = wp;
        }
      }
    }

    const bool scroll_changed = !ignore_scroll
                                && (wp->w_last_topline != wp->w_topline
                                    || wp->w_last_topfill != wp->w_topfill
                                    || wp->w_last_leftcol != wp->w_leftcol
                                    || wp->w_last_skipcol != wp->w_skipcol);
    if (scroll_changed && first_scroll_win != NULL && *first_scroll_win == NULL) {
      *first_scroll_win = wp;
    }

    if ((size_changed || scroll_changed) && v_event != NULL) {
      // Add info about this window to the v:event dictionary.
      int width = wp->w_width - wp->w_last_width;
      int height = wp->w_height - wp->w_last_height;
      int topline = wp->w_topline - wp->w_last_topline;
      int topfill = wp->w_topfill - wp->w_last_topfill;
      int leftcol = wp->w_leftcol - wp->w_last_leftcol;
      int skipcol = wp->w_skipcol - wp->w_last_skipcol;
      dict_T *d = make_win_info_dict(width, height, topline,
                                     topfill, leftcol, skipcol);
      if (d == NULL) {
        break;
      }
      char winid[NUMBUFLEN];
      int key_len = vim_snprintf(winid, sizeof(winid), "%d", wp->handle);
      if (tv_dict_add_dict(v_event, winid, (size_t)key_len, d) == FAIL) {
        tv_dict_unref(d);
        break;
      }
      d->dv_refcount--;

      tot_width += abs(width);
      tot_height += abs(height);
      tot_topline += abs(topline);
      tot_topfill += abs(topfill);
      tot_leftcol += abs(leftcol);
      tot_skipcol += abs(skipcol);
    }
  }

  if (v_event != NULL) {
    dict_T *alldict = make_win_info_dict(tot_width, tot_height, tot_topline,
                                         tot_topfill, tot_leftcol, tot_skipcol);
    if (alldict != NULL) {
      if (tv_dict_add_dict(v_event, S_LEN("all"), alldict) == FAIL) {
        tv_dict_unref(alldict);
      } else {
        alldict->dv_refcount--;
      }
    }
  }
}

/// Trigger WinScrolled and/or WinResized if any window in the current tab page
/// scrolled or changed size.
void may_trigger_win_scrolled_resized(void)
{
  static bool recursive = false;
  const bool do_resize = has_event(EVENT_WINRESIZED);
  const bool do_scroll = has_event(EVENT_WINSCROLLED);

  if (recursive
      || !(do_scroll || do_resize)
      || !did_initial_scroll_size_snapshot) {
    return;
  }

  int size_count = 0;
  win_T *first_scroll_win = NULL;
  win_T *first_size_win = NULL;
  check_window_scroll_resize(&size_count, &first_scroll_win, &first_size_win, NULL, NULL);
  bool trigger_resize = do_resize && size_count > 0;
  bool trigger_scroll = do_scroll && first_scroll_win != NULL;
  if (!trigger_resize && !trigger_scroll) {
    return;  // no relevant changes
  }

  list_T *windows_list = NULL;
  if (trigger_resize) {
    // Create the list for v:event.windows before making the snapshot.
    // windows_list = tv_list_alloc_with_items(size_count);
    windows_list = tv_list_alloc(size_count);
    check_window_scroll_resize(NULL, NULL, NULL, windows_list, NULL);
  }

  dict_T *scroll_dict = NULL;
  if (trigger_scroll) {
    // Create the dict with entries for v:event before making the snapshot.
    scroll_dict = tv_dict_alloc();
    scroll_dict->dv_refcount = 1;
    check_window_scroll_resize(NULL, NULL, NULL, NULL, scroll_dict);
  }

  // WinScrolled/WinResized are triggered only once, even when multiple
  // windows scrolled or changed size.  Store the current values before
  // triggering the event, if a scroll or resize happens as a side effect
  // then WinScrolled/WinResized is triggered for that later.
  snapshot_windows_scroll_size();

  recursive = true;

  // Save window info before autocmds since they can free windows
  char resize_winid[NUMBUFLEN];
  bufref_T resize_bufref;
  if (trigger_resize) {
    vim_snprintf(resize_winid, sizeof(resize_winid), "%d", first_size_win->handle);
    set_bufref(&resize_bufref, first_size_win->w_buffer);
  }

  char scroll_winid[NUMBUFLEN];
  bufref_T scroll_bufref;
  if (trigger_scroll) {
    vim_snprintf(scroll_winid, sizeof(scroll_winid), "%d", first_scroll_win->handle);
    set_bufref(&scroll_bufref, first_scroll_win->w_buffer);
  }

  // If both are to be triggered do WinResized first.
  if (trigger_resize) {
    save_v_event_T save_v_event;
    dict_T *v_event = get_v_event(&save_v_event);

    if (tv_dict_add_list(v_event, S_LEN("windows"), windows_list) == OK) {
      tv_dict_set_keys_readonly(v_event);
      buf_T *buf = bufref_valid(&resize_bufref) ? resize_bufref.br_buf : curbuf;
      apply_autocmds(EVENT_WINRESIZED, resize_winid, resize_winid, false, buf);
    }
    restore_v_event(v_event, &save_v_event);
  }

  if (trigger_scroll) {
    save_v_event_T save_v_event;
    dict_T *v_event = get_v_event(&save_v_event);

    // Move the entries from scroll_dict to v_event.
    tv_dict_extend(v_event, scroll_dict, "move");
    tv_dict_set_keys_readonly(v_event);
    tv_dict_unref(scroll_dict);

    buf_T *buf = bufref_valid(&scroll_bufref) ? scroll_bufref.br_buf : curbuf;
    apply_autocmds(EVENT_WINSCROLLED, scroll_winid, scroll_winid, false, buf);

    restore_v_event(v_event, &save_v_event);
  }

  recursive = false;
}

// Save the size of all windows in "gap".
void win_size_save(garray_T *gap)
{
  rs_win_size_save(gap);
}

// Restore window sizes, but only if the number of windows is still the same
// and total lines available for windows didn't change.
// Does not free the growarray.
void win_size_restore(garray_T *gap)
  FUNC_ATTR_NONNULL_ALL
{
  rs_win_size_restore(gap);
}

// Update the position for all windows, using the width and height of the frames.
// Returns the row just after the last window and global statusline (if there is one).
int win_comp_pos(void)
{
  return rs_win_comp_pos();
}

// Update the position of the windows in frame "topfrp", using the width and
// height of the frames.
// "*row" and "*col" are the top-left position of the frame.  They are updated
// to the bottom-right position plus one.
static void frame_comp_pos(frame_T *topfrp, int *row, int *col)
{
  rs_frame_comp_pos(topfrp, row, col);
}

// Set current window height and take care of repositioning other windows to
// fit around it.
void win_setheight(int height)
{
  rs_win_setheight(height);
}

// Set the window height of window "win" and take care of repositioning other
// windows to fit around it.
void win_setheight_win(int height, win_T *win)
{
  rs_win_setheight_win(height, win);
}

// Set the height of a frame to "height" and take care that all frames and
// windows inside it are resized.  Also resize frames on the left and right if
// the are in the same FR_ROW frame.
//
// Strategy:
// If the frame is part of a FR_COL frame, try fitting the frame in that
// frame.  If that doesn't work (the FR_COL frame is too small), recursively
// go to containing frames to resize them and make room.
// If the frame is part of a FR_ROW frame, all frames must be resized as well.
// Check for the minimal height of the FR_ROW frame.
// At the top level we can also use change the command line height.
static void frame_setheight(frame_T *curfrp, int height)
{
  rs_frame_setheight(curfrp, height);
}

// Set current window width and take care of repositioning other windows to
// fit around it.
void win_setwidth(int width)
{
  rs_win_setwidth(width);
}

void win_setwidth_win(int width, win_T *wp)
{
  rs_win_setwidth_win(width, wp);
}

// Set the width of a frame to "width" and take care that all frames and
// windows inside it are resized.  Also resize frames above and below if the
// are in the same FR_ROW frame.
//
// Strategy is similar to frame_setheight().
static void frame_setwidth(frame_T *curfrp, int width)
{
  rs_frame_setwidth(curfrp, width);
}

// Check 'winminheight' for a valid value and reduce it if needed.
const char *did_set_winminheight(optset_T *args FUNC_ATTR_UNUSED)
{
  return rs_did_set_winminheight();
}

// Check 'winminwidth' for a valid value and reduce it if needed.
const char *did_set_winminwidth(optset_T *args FUNC_ATTR_UNUSED)
{
  return rs_did_set_winminwidth();
}

/// Status line of dragwin is dragged "offset" lines down (negative is up).
void win_drag_status_line(win_T *dragwin, int offset)
{
  rs_win_drag_status_line(dragwin, offset);
}

// Separator line of dragwin is dragged "offset" lines right (negative is left).
void win_drag_vsep_line(win_T *dragwin, int offset)
{
  rs_win_drag_vsep_line(dragwin, offset);
}

#define FRACTION_MULT   16384

// Set wp->w_fraction for the current w_wrow and w_height.
// Has no effect when the window is less than two lines.
void set_fraction(win_T *wp)
{
  rs_set_fraction(wp);
}

/// Handle scroll position, depending on 'splitkeep'.  Replaces the
/// scroll_to_fraction() call from win_new_height() if 'splitkeep' is "screen"
/// or "topline".  Instead we iterate over all windows in a tabpage and
/// calculate the new scroll position.
/// TODO(vim): Ensure this also works with wrapped lines.
/// Requires a not fully visible cursor line to be allowed at the bottom of
/// a window ("zb"), probably only when 'smoothscroll' is also set.
void win_fix_scroll(bool resize)
{
  if (*p_spk == 'c') {
    return;  // 'splitkeep' is "cursor"
  }

  skip_update_topline = true;
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    // Skip when window height has not changed or when floating.
    if (!wp->w_floating && wp->w_height != wp->w_prev_height) {
      // Cursor position in this window may now be invalid.  It is kept
      // potentially invalid until the window is made the current window.
      wp->w_do_win_fix_cursor = true;

      // If window has moved update botline to keep the same screenlines.
      if (*p_spk == 's' && wp->w_winrow != wp->w_prev_winrow
          && wp->w_botline - 1 <= wp->w_buffer->b_ml.ml_line_count) {
        int diff = (wp->w_winrow - wp->w_prev_winrow)
                   + (wp->w_height - wp->w_prev_height);
        pos_T cursor = wp->w_cursor;
        wp->w_cursor.lnum = wp->w_botline - 1;

        // Add difference in height and row to botline.
        if (diff > 0) {
          cursor_down_inner(wp, diff, false);
        } else {
          cursor_up_inner(wp, -diff, false);
        }

        // Scroll to put the new cursor position at the bottom of the
        // screen.
        wp->w_fraction = FRACTION_MULT;
        scroll_to_fraction(wp, wp->w_prev_height);
        wp->w_cursor = cursor;
        wp->w_valid &= ~VALID_WCOL;
      } else if (wp == curwin) {
        wp->w_valid &= ~VALID_CROW;
      }

      invalidate_botline(wp);
      validate_botline(wp);
    }
    wp->w_prev_height = wp->w_height;
    wp->w_prev_winrow = wp->w_winrow;
  }
  skip_update_topline = false;
  // Ensure cursor is valid when not in normal mode or when resized.
  if (!(get_real_state() & (MODE_NORMAL|MODE_CMDLINE|MODE_TERMINAL))) {
    win_fix_cursor(false);
  } else if (resize) {
    win_fix_cursor(true);
  }
}

/// Make sure the cursor position is valid for 'splitkeep'.
/// If it is not, put the cursor position in the jumplist and move it.
/// If we are not in normal mode ("normal" is false), make it valid by scrolling
/// instead.
static void win_fix_cursor(bool normal)
{
  win_T *wp = curwin;

  if (skip_win_fix_cursor
      || !wp->w_do_win_fix_cursor
      || wp->w_buffer->b_ml.ml_line_count < wp->w_view_height) {
    return;
  }

  wp->w_do_win_fix_cursor = false;
  // Determine valid cursor range.
  int so = MIN(wp->w_view_height / 2, get_scrolloff_value(wp));
  linenr_T lnum = wp->w_cursor.lnum;

  wp->w_cursor.lnum = wp->w_topline;
  cursor_down_inner(wp, so, false);
  linenr_T top = wp->w_cursor.lnum;

  wp->w_cursor.lnum = wp->w_botline - 1;
  cursor_up_inner(wp, so, false);
  linenr_T bot = wp->w_cursor.lnum;

  wp->w_cursor.lnum = lnum;
  // Check if cursor position is above or below valid cursor range.
  linenr_T nlnum = 0;
  if (lnum > bot && (wp->w_botline - wp->w_buffer->b_ml.ml_line_count) != 1) {
    nlnum = bot;
  } else if (lnum < top && wp->w_topline != 1) {
    nlnum = (so == wp->w_view_height / 2) ? bot : top;
  }

  if (nlnum != 0) {  // Cursor is invalid for current scroll position.
    if (normal) {    // Save to jumplist and set cursor to avoid scrolling.
      setmark('\'');
      wp->w_cursor.lnum = nlnum;
    } else {         // Scroll instead when not in normal mode.
      wp->w_fraction = (nlnum == bot) ? FRACTION_MULT : 0;
      scroll_to_fraction(wp, wp->w_prev_height);
      validate_botline(curwin);
    }
  }
}

// Set the height of a window.
// "height" excludes any window toolbar.
// This takes care of the things inside the window, not what happens to the
// window position, the frame or to other windows.
void win_new_height(win_T *wp, int height)
{
  rs_win_new_height(wp, height);
}

void scroll_to_fraction(win_T *wp, int prev_height)
{
  int height = wp->w_view_height;

  // Don't change w_topline in any of these cases:
  // - window height is 0
  // - 'scrollbind' is set and this isn't the current window
  // - window height is sufficient to display the whole buffer and first line
  //   is visible.
  if (height > 0
      && (!wp->w_p_scb || wp == curwin)
      && (height < wp->w_buffer->b_ml.ml_line_count
          || wp->w_topline > 1)) {
    // Find a value for w_topline that shows the cursor at the same
    // relative position in the window as before (more or less).
    linenr_T lnum = wp->w_cursor.lnum;
    // can happen when starting up
    lnum = MAX(lnum, 1);
    wp->w_wrow = (wp->w_fraction * height - 1) / FRACTION_MULT;
    int line_size = plines_win_col(wp, lnum, wp->w_cursor.col) - 1;
    int sline = wp->w_wrow - line_size;

    if (sline >= 0) {
      // Make sure the whole cursor line is visible, if possible.
      const int rows = plines_win(wp, lnum, false);

      if (sline > wp->w_view_height - rows) {
        sline = wp->w_view_height - rows;
        wp->w_wrow -= rows - line_size;
      }
    }

    if (sline < 0) {
      // Cursor line would go off top of screen if w_wrow was this high.
      // Make cursor line the first line in the window.  If not enough
      // room use w_skipcol;
      wp->w_wrow = line_size;
      if (wp->w_wrow >= wp->w_view_height
          && (wp->w_view_width - win_col_off(wp)) > 0) {
        wp->w_skipcol += wp->w_view_width - win_col_off(wp);
        wp->w_wrow--;
        while (wp->w_wrow >= wp->w_view_height) {
          wp->w_skipcol += wp->w_view_width - win_col_off(wp)
                           + win_col_off2(wp);
          wp->w_wrow--;
        }
      }
    } else if (sline > 0) {
      while (sline > 0 && lnum > 1) {
        hasFolding(wp, lnum, &lnum, NULL);
        if (lnum == 1) {
          // first line in buffer is folded
          line_size = !decor_conceal_line(wp, lnum - 1, false);
          sline--;
          break;
        }
        lnum--;
        if (lnum == wp->w_topline) {
          line_size = plines_win_nofill(wp, lnum, true)
                      + wp->w_topfill;
        } else {
          line_size = plines_win(wp, lnum, true);
        }
        sline -= line_size;
      }

      if (sline < 0) {
        // Line we want at top would go off top of screen.  Use next
        // line instead.
        hasFolding(wp, lnum, NULL, &lnum);
        lnum++;
        wp->w_wrow -= line_size + sline;
      } else if (sline > 0) {
        // First line of file reached, use that as topline.
        lnum = 1;
        wp->w_wrow -= sline;
      }
    }
    set_topline(wp, lnum);
  }

  if (wp == curwin) {
    curs_columns(wp, false);        // validate w_wrow
  }
  if (prev_height > 0) {
    wp->w_prev_fraction_row = wp->w_wrow;
  }

  redraw_later(wp, UPD_SOME_VALID);
  invalidate_botline(wp);
}

void win_set_inner_size(win_T *wp, bool valid_cursor)
{
  int width = wp->w_width_request;
  if (width == 0) {
    width = wp->w_width;
  }

  int prev_height = wp->w_view_height;
  int height = wp->w_height_request;
  if (height == 0) {
    height = MAX(0, wp->w_height - wp->w_winbar_height);
  }

  if (height != prev_height) {
    if (height > 0 && valid_cursor) {
      if (wp == curwin && (*p_spk == 'c' || wp->w_floating)) {
        // w_wrow needs to be valid. When setting 'laststatus' this may
        // call win_new_height() recursively.
        validate_cursor(curwin);
      }
      if (wp->w_view_height != prev_height) {
        return;  // Recursive call already changed the size, bail out.
      }
      if (wp->w_wrow != wp->w_prev_fraction_row) {
        set_fraction(wp);
      }
    }
    wp->w_view_height = height;
    win_comp_scroll(wp);

    // There is no point in adjusting the scroll position when exiting.  Some
    // values might be invalid.
    if (valid_cursor && !exiting && (*p_spk == 'c' || wp->w_floating)) {
      wp->w_skipcol = 0;
      scroll_to_fraction(wp, prev_height);
    }
    redraw_later(wp, UPD_SOME_VALID);
  }

  if (width != wp->w_view_width) {
    wp->w_view_width = width;
    wp->w_lines_valid = 0;
    if (valid_cursor) {
      changed_line_abv_curs_win(wp);
      invalidate_botline(wp);
      if (wp == curwin && (*p_spk == 'c' || wp->w_floating)) {
        curs_columns(wp, true);  // validate w_wrow
      }
    }
    redraw_later(wp, UPD_NOT_VALID);
  }

  if (wp->w_buffer->terminal) {
    terminal_check_size(wp->w_buffer->terminal);
  }

  int float_stl_height = wp->w_floating && wp->w_status_height ? STATUS_HEIGHT : 0;
  wp->w_height_outer = (wp->w_view_height + win_border_height(wp) + wp->w_winbar_height +
                        float_stl_height);
  wp->w_width_outer = (wp->w_view_width + win_border_width(wp));
  wp->w_winrow_off = wp->w_border_adj[0] + wp->w_winbar_height;
  wp->w_wincol_off = wp->w_border_adj[3];

  if (ui_has(kUIMultigrid)) {
    ui_call_win_viewport_margins(wp->w_grid_alloc.handle, wp->handle,
                                 wp->w_winrow_off, wp->w_border_adj[2],
                                 wp->w_wincol_off, wp->w_border_adj[1]);
  }

  wp->w_redr_status = true;
}

/// Set the width of a window.
void win_new_width(win_T *wp, int width)
{
  rs_win_new_width(wp, width);
}

OptInt win_default_scroll(win_T *wp)
{
  return rs_win_default_scroll(wp);
}

void win_comp_scroll(win_T *wp)
{
  const OptInt old_w_p_scr = wp->w_p_scr;
  wp->w_p_scr = win_default_scroll(wp);

  if (wp->w_p_scr != old_w_p_scr) {
    // Used by "verbose set scroll".
    wp->w_p_script_ctx[kWinOptScroll].sc_sid = SID_WINLAYOUT;
    wp->w_p_script_ctx[kWinOptScroll].sc_lnum = 0;
  }
}

/// command_height: called whenever p_ch has been changed.
void command_height(void)
{
  int old_p_ch = (int)curtab->tp_ch_used;

  // Find bottom frame with width of screen.
  frame_T *frp = lastwin_nofloating()->w_frame;
  while (frp->fr_width != Columns && frp->fr_parent != NULL) {
    frp = frp->fr_parent;
  }

  // Avoid changing the height of a window with 'winfixheight' set.
  while (frp->fr_prev != NULL && frp->fr_layout == FR_LEAF && frp->fr_win->w_p_wfh) {
    frp = frp->fr_prev;
  }

  while (p_ch > old_p_ch && command_frame_height) {
    if (frp == NULL) {
      emsg(_(e_noroom));
      p_ch = old_p_ch;
      break;
    }
    int h = MIN((int)(p_ch - old_p_ch), frp->fr_height - frame_minheight(frp, NULL));
    frame_add_height(frp, -h);
    old_p_ch += h;
    frp = frp->fr_prev;
  }
  if (p_ch < old_p_ch && command_frame_height && frp != NULL) {
    frame_add_height(frp, (int)(old_p_ch - p_ch));
  }

  // Recompute window positions.
  win_comp_pos();
  cmdline_row = Rows - (int)p_ch;
  redraw_cmdline = true;

  // Clear the cmdheight area.
  if (msg_scrolled == 0 && full_screen) {
    GridView *grid = &default_gridview;
    if (!ui_has(kUIMessages)) {
      msg_grid_validate();
      grid = &msg_grid_adj;
    }
    grid_clear(grid, cmdline_row, Rows, 0, Columns, 0);
    msg_row = cmdline_row;
  }

  // Use the value of p_ch that we remembered.  This is needed for when the
  // GUI starts up, we can't be sure in what order things happen.  And when
  // p_ch was changed in another tab page.
  curtab->tp_ch_used = p_ch;
  min_set_ch = p_ch;
}

// Resize frame "frp" to be "n" lines higher (negative for less high).
// Also resize the frames it is contained in.
static void frame_add_height(frame_T *frp, int n)
{
  rs_frame_add_height(frp, n);
}

/// Add or remove a status line from window(s), according to the
/// value of 'laststatus'.
///
/// @param morewin  pretend there are two or more windows if true.
void last_status(bool morewin)
{
  rs_last_status(morewin);
}

// Remove status line from window, replacing it with a horizontal separator if needed.
void win_remove_status_line(win_T *wp, bool add_hsep)
{
  rs_win_remove_status_line(wp, add_hsep);
}

/// Add or remove window bar from window "wp".
///
/// @param make_room Whether to resize frames to make room for winbar.
/// @param valid_cursor Whether the cursor is valid and should be used while
///                     resizing.
///
/// @return Success status.
int set_winbar_win(win_T *wp, bool make_room, bool valid_cursor)
{
  // Require the local value to be set in order to show winbar on a floating window.
  int winbar_height = wp->w_floating ? ((*wp->w_p_wbr != NUL) ? 1 : 0)
                                     : ((*p_wbr != NUL || *wp->w_p_wbr != NUL) ? 1 : 0);

  if (wp->w_winbar_height != winbar_height) {
    if (winbar_height == 1 && wp->w_view_height <= 1) {
      if (wp->w_floating) {
        emsg(_(e_noroom));
        return NOTDONE;
      } else if (!make_room || !rs_resize_frame_for_winbar(wp->w_frame)) {
        return FAIL;
      }
    }
    wp->w_winbar_height = winbar_height;
    win_set_inner_size(wp, valid_cursor);

    if (winbar_height == 0) {
      // When removing winbar, deallocate the w_winbar_click_defs array
      stl_clear_click_defs(wp->w_winbar_click_defs, wp->w_winbar_click_defs_size);
      xfree(wp->w_winbar_click_defs);
      wp->w_winbar_click_defs_size = 0;
      wp->w_winbar_click_defs = NULL;
    }
  }

  return OK;
}

/// Add or remove window bars from all windows in tab depending on the value of 'winbar'.
///
/// @param make_room Whether to resize frames to make room for winbar.
void set_winbar(bool make_room)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (set_winbar_win(wp, make_room, true) == FAIL) {
      break;
    }
  }
}

extern int rs_tabline_height(void);

/// Return the number of lines used by the tab page line.
int tabline_height(void)
{
  return rs_tabline_height();
}

extern int rs_global_winbar_height(void);
extern int rs_global_stl_height(void);
extern int rs_last_stl_height(int morewin);

/// Return the number of lines used by default by the window bar.
int global_winbar_height(void)
{
  return rs_global_winbar_height();
}

/// Return the number of lines used by the global statusline
int global_stl_height(void)
{
  return rs_global_stl_height();
}

/// Return the height of the last window's statusline, or the global statusline if set.
///
/// @param morewin  pretend there are two or more windows if true.
int last_stl_height(bool morewin)
{
  return rs_last_stl_height(morewin ? 1 : 0);
}

/// Return the minimal number of rows that is needed on the screen to display
/// the current number of windows for the given tab page.
int min_rows(tabpage_T *tp) FUNC_ATTR_NONNULL_ALL
{
  return rs_min_rows(tp);
}

/// Return the minimal number of rows that is needed on the screen to display
/// the current number of windows for all tab pages.
int min_rows_for_all_tabpages(void)
{
  return rs_min_rows_for_all_tabpages();
}

/// Check that there is only one window (and only one tab page), not counting a
/// help or preview window, unless it is the current window. Does not count
/// "aucmd_win". Does not count floats unless it is current.
bool only_one_window(void) FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_only_one_window() != 0;
}

/// Correct the cursor line number in other windows.  Used after changing the
/// current buffer, and before applying autocommands.
///
/// @param do_curwin  when true, also check current window.
void check_lnums(bool do_curwin)
{
  rs_check_lnums(do_curwin);
}

/// Like check_lnums() but for when check_lnums() was already called.
void check_lnums_nested(bool do_curwin)
{
  rs_check_lnums_nested(do_curwin);
}

/// Reset cursor and topline to its stored values from check_lnums().
/// check_lnums() must have been called first!
void reset_lnums(void)
{
  rs_reset_lnums();
}

// A snapshot of the window sizes, to restore them after closing the help
// window.
// Only these fields are used:
// fr_layout
// fr_width
// fr_height
// fr_next
// fr_child
// fr_win (only valid for the old curwin, NULL otherwise)

// Create a snapshot of the current frame sizes.
void make_snapshot(int idx)
{
  rs_make_snapshot(idx);
}

// Remove any existing snapshot.
static void clear_snapshot(tabpage_T *tp, int idx)
{
  rs_clear_snapshot(tp, idx);
}

/// @return  the current window stored in the snapshot or NULL.
static win_T *get_snapshot_curwin(int idx)
{
  return rs_get_snapshot_curwin(idx);
}

/// Restore a previously created snapshot, if there is any.
/// This is only done if the screen size didn't change and the window layout is
/// still the same.
///
/// @param close_curwin  closing current window
void restore_snapshot(int idx, int close_curwin)
{
  if (curtab->tp_snapshot[idx] != NULL
      && curtab->tp_snapshot[idx]->fr_width == topframe->fr_width
      && curtab->tp_snapshot[idx]->fr_height == topframe->fr_height
      && check_snapshot_rec(curtab->tp_snapshot[idx], topframe) == OK) {
    win_T *wp = restore_snapshot_rec(curtab->tp_snapshot[idx], topframe);
    win_comp_pos();
    if (wp != NULL && close_curwin) {
      win_goto(wp);
    }
    redraw_all_later(UPD_NOT_VALID);
  }
  clear_snapshot(curtab, idx);
}

/// Check if frames "sn" and "fr" have the same layout, same following frames
/// and same children.  And the window pointer is valid.
static int check_snapshot_rec(frame_T *sn, frame_T *fr)
{
  return rs_check_snapshot_rec(sn, fr);
}

// Copy the size of snapshot frame "sn" to frame "fr".  Do the same for all
// following frames and children.
// Returns a pointer to the old current window, or NULL.
static win_T *restore_snapshot_rec(frame_T *sn, frame_T *fr)
{
  return rs_restore_snapshot_rec(sn, fr);
}

/// Check that "topfrp" and its children are at the right height.
///
/// @param  topfrp  top frame pointer
/// @param  height  expected height
static bool frame_check_height(const frame_T *topfrp, int height)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
  return rs_frame_check_height((frame_T *)topfrp, height) != 0;
}

/// Check that "topfrp" and its children are at the right width.
///
/// @param  topfrp  top frame pointer
/// @param  width   expected width
static bool frame_check_width(const frame_T *topfrp, int width)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
  return rs_frame_check_width((frame_T *)topfrp, width) != 0;
}

/// Simple int comparison function for use with qsort()
static int int_cmp(const void *pa, const void *pb)
{
  const int a = *(const int *)pa;
  const int b = *(const int *)pb;
  return a == b ? 0 : a < b ? -1 : 1;
}

/// Check "cc" as 'colorcolumn' and update the members of "wp".
/// This is called when 'colorcolumn' or 'textwidth' is changed.
///
/// @param cc  when NULL: use "wp->w_p_cc"
/// @param wp  when NULL: only parse "cc"
///
/// @return error message, NULL if it's OK.
const char *check_colorcolumn(char *cc, win_T *wp)
{
  if (wp != NULL && wp->w_buffer == NULL) {
    return NULL;      // buffer was closed
  }

  char *s = empty_string_option;
  if (cc != NULL) {
    s = cc;
  } else if (wp != NULL) {
    s = wp->w_p_cc;
  }

  OptInt tw;
  if (wp != NULL) {
    tw = wp->w_buffer->b_p_tw;
  } else {
    // buffer-local value not set, assume zero
    tw = 0;
  }

  unsigned count = 0;
  int color_cols[256];
  while (*s != NUL && count < 255) {
    int col;
    if (*s == '-' || *s == '+') {
      // -N and +N: add to 'textwidth'
      col = (*s == '-') ? -1 : 1;
      s++;
      if (!ascii_isdigit(*s)) {
        return e_invarg;
      }
      col = col * getdigits_int(&s, true, 0);
      if (tw == 0) {
        goto skip;          // 'textwidth' not set, skip this item
      }
      assert((col >= 0 && tw <= INT_MAX - col && tw + col >= INT_MIN)
             || (col < 0 && tw >= INT_MIN - col && tw + col <= INT_MAX));
      col += (int)tw;
      if (col < 0) {
        goto skip;
      }
    } else if (ascii_isdigit(*s)) {
      col = getdigits_int(&s, true, 0);
    } else {
      return e_invarg;
    }
    color_cols[count++] = col - 1;      // 1-based to 0-based
skip:
    if (*s == NUL) {
      break;
    }
    if (*s != ',') {
      return e_invarg;
    }
    if (*++s == NUL) {
      return e_invarg;        // illegal trailing comma as in "set cc=80,"
    }
  }

  if (wp == NULL) {
    return NULL;  // only parse "cc"
  }

  xfree(wp->w_p_cc_cols);
  if (count == 0) {
    wp->w_p_cc_cols = NULL;
  } else {
    wp->w_p_cc_cols = xmalloc(sizeof(int) * (count + 1));
    // sort the columns for faster usage on screen redraw inside
    // win_line()
    qsort(color_cols, count, sizeof(int), int_cmp);

    int j = 0;
    for (unsigned i = 0; i < count; i++) {
      // skip duplicates
      if (j == 0 || wp->w_p_cc_cols[j - 1] != color_cols[i]) {
        wp->w_p_cc_cols[j++] = color_cols[i];
      }
    }
    wp->w_p_cc_cols[j] = -1;        // end marker
  }

  return NULL;    // no error
}

int get_last_winid(void)
{
  return rs_get_last_winid();
}

/// Don't let autocommands close the given window
int win_locked(win_T *wp)
{
  return rs_win_locked(wp);
}

void win_get_tabwin(handle_T id, int *tabnr, int *winnr)
{
  rs_win_get_tabwin(id, tabnr, winnr);
}

void win_ui_flush(bool validate)
{
  FOR_ALL_TAB_WINDOWS(tp, wp) {
    if ((wp->w_pos_changed || wp->w_grid_alloc.pending_comp_index_update)
        && wp->w_grid_alloc.chars != NULL) {
      if (tp == curtab) {
        ui_ext_win_position(wp, validate);
      } else {
        ui_call_win_hide(wp->w_grid_alloc.handle);
        wp->w_pos_changed = false;
      }
      wp->w_grid_alloc.pending_comp_index_update = false;
    }
    if (tp == curtab) {
      ui_ext_win_viewport(wp);
    }
  }
  // The popupmenu could also have moved or changed its comp_index
  pum_ui_flush();

  // And the message
  msg_ui_flush();
}

win_T *lastwin_nofloating(void)
{
  return rs_lastwin_nofloating();
}

// ============================================================================
// Window Accessor Functions (for Rust FFI)
// ============================================================================

/// Get the number of empty rows at the bottom of a window
int nvim_win_get_empty_rows(win_T *wp)
{
  return wp ? wp->w_empty_rows : 0;
}

/// Set the leftcol value for a window
void nvim_win_set_leftcol(win_T *wp, int val)
{
  if (wp) {
    wp->w_leftcol = (colnr_T)val;
  }
}

/// Set the botline value for a window
void nvim_win_set_botline(win_T *wp, int val)
{
  if (wp) {
    wp->w_botline = (linenr_T)val;
  }
}

/// Set the number of empty rows at the bottom of a window
void nvim_win_set_empty_rows(win_T *wp, int val)
{
  if (wp) {
    wp->w_empty_rows = val;
  }
}

/// Check if window's buffer equals the given buffer.
/// Returns 1 if equal, 0 otherwise.
int nvim_win_buffer_eq(win_T *wp, buf_T *buf)
{
  return (wp && wp->w_buffer == buf) ? 1 : 0;
}

/// Get the grid allocation valid flag for window.
int nvim_win_grid_alloc_valid(win_T *wp)
{
  return wp ? wp->w_grid_alloc.valid : 0;
}

/// Set the grid allocation valid flag for window.
void nvim_win_grid_alloc_set_valid(win_T *wp, int val)
{
  if (wp) {
    wp->w_grid_alloc.valid = (val != 0);
  }
}

/// Get the window's w_redr_type field (for Rust FFI compatibility).
/// This is an alias for nvim_win_get_redr_type.
int nvim_win_get_w_redr_type(win_T *wp)
{
  return wp ? wp->w_redr_type : 0;
}

/// Set the window's w_redr_type field (for Rust FFI compatibility).
/// This is an alias for nvim_win_set_redr_type.
void nvim_win_set_w_redr_type(win_T *wp, int val)
{
  if (wp) {
    wp->w_redr_type = val;
  }
}

/// Set the window's w_lines_valid field (for Rust FFI compatibility).
void nvim_win_set_w_lines_valid(win_T *wp, int val)
{
  if (wp) {
    wp->w_lines_valid = (linenr_T)val;
  }
}

/// Get the number of filler rows at the end of the window.
int nvim_win_get_filler_rows(win_T *wp)
{
  return wp ? wp->w_filler_rows : 0;
}

/// Set the number of filler rows at the end of the window.
void nvim_win_set_filler_rows(win_T *wp, int val)
{
  if (wp) {
    wp->w_filler_rows = val;
  }
}

/// Get the botfill flag (true when filler lines are at bottom).
int nvim_win_get_botfill(win_T *wp)
{
  return wp ? (wp->w_botfill ? 1 : 0) : 0;
}

/// Set the botfill flag (true when filler lines are at bottom).
void nvim_win_set_botfill(win_T *wp, int val)
{
  if (wp) {
    wp->w_botfill = (val != 0);
  }
}

/// Check if window's w_grid.target is non-NULL (accessor for Rust).
int nvim_win_grid_has_target(win_T *wp)
{
  return (wp && wp->w_grid.target) ? 1 : 0;
}

/// Get the scroll binding position (accessor for Rust).
int nvim_win_get_scbind_pos(win_T *wp)
{
  return wp ? wp->w_scbind_pos : 0;
}

/// Set the scroll binding position (accessor for Rust).
void nvim_win_set_scbind_pos(win_T *wp, int val)
{
  if (wp) {
    wp->w_scbind_pos = val;
  }
}

/// Check if window's buffer is empty (accessor for Rust).
int nvim_win_buf_is_empty(win_T *wp)
{
  return (wp && wp->w_buffer) ? buf_is_empty(wp->w_buffer) : 1;
}

// --- Phase 1 accessors ---

/// Set the w_fraction field for a window (accessor for Rust).
void nvim_win_set_fraction(win_T *wp, int val)
{
  if (wp) {
    wp->w_fraction = val;
  }
}

/// Get the tp_ch_used field from a tabpage (accessor for Rust).
int nvim_tabpage_get_ch_used(tabpage_T *tp)
{
  return tp ? (int)tp->tp_ch_used : 0;
}

/// Check if window has a winnr in tabpage (accessor for Rust).
/// Wraps win_has_winnr() from eval/window.c.
int nvim_win_has_winnr(win_T *wp, tabpage_T *tp)
{
  return (wp && tp) ? (int)win_has_winnr(wp, tp) : 0;
}

// --- Phase 2 accessors ---

/// Set the global p_wmh (winminheight) value.
void nvim_set_p_wmh(int64_t val)
{
  p_wmh = val;
}

/// Set the global p_wmw (winminwidth) value.
void nvim_set_p_wmw(int64_t val)
{
  p_wmw = val;
}

/// Emit the E36 "Not enough room" error message.
void nvim_emsg_noroom(void)
{
  emsg(_(e_noroom));
}

/// Wrapper for win_set_inner_size() (stays in C).
void nvim_win_set_inner_size(win_T *wp, int valid_cursor)
{
  if (wp) {
    win_set_inner_size(wp, valid_cursor != 0);
  }
}

// --- Phase 3 accessors ---

/// Get a snapshot pointer from a tabpage.
frame_T *nvim_tabpage_get_snapshot(tabpage_T *tp, int idx)
{
  if (!tp || idx < 0 || idx >= SNAP_COUNT) {
    return NULL;
  }
  return tp->tp_snapshot[idx];
}

/// Set a snapshot pointer in a tabpage.
void nvim_tabpage_set_snapshot(tabpage_T *tp, int idx, frame_T *val)
{
  if (tp && idx >= 0 && idx < SNAP_COUNT) {
    tp->tp_snapshot[idx] = val;
  }
}

// --- Phase 4 accessors ---

// nvim_curbuf_line_count() — already defined in move.c

/// Check if window's buffer is curbuf.
int nvim_win_buf_is_curbuf(win_T *wp)
{
  return wp && wp->w_buffer == curbuf;
}

/// Save cursor position to w_save_cursor.w_cursor_save.
void nvim_win_save_cursor_to_save(win_T *wp)
{
  if (wp) {
    wp->w_save_cursor.w_cursor_save = wp->w_cursor;
  }
}

/// Save topline to w_save_cursor.w_topline_save.
void nvim_win_save_topline_to_save(win_T *wp)
{
  if (wp) {
    wp->w_save_cursor.w_topline_save = wp->w_topline;
  }
}

/// Save (corrected) cursor position to w_save_cursor.w_cursor_corr.
void nvim_win_save_cursor_to_corr(win_T *wp)
{
  if (wp) {
    wp->w_save_cursor.w_cursor_corr = wp->w_cursor;
  }
}

/// Save (corrected) topline to w_save_cursor.w_topline_corr.
void nvim_win_save_topline_to_corr(win_T *wp)
{
  if (wp) {
    wp->w_save_cursor.w_topline_corr = wp->w_topline;
  }
}

/// Check if w_save_cursor.w_cursor_corr equals w_cursor (via equalpos).
int nvim_win_cursor_eq_save_corr(win_T *wp)
{
  if (!wp) {
    return 0;
  }
  return equalpos(wp->w_save_cursor.w_cursor_corr, wp->w_cursor);
}

/// Check if w_save_cursor.w_topline_corr equals w_topline.
int nvim_win_topline_eq_save_corr(win_T *wp)
{
  if (!wp) {
    return 0;
  }
  return wp->w_save_cursor.w_topline_corr == wp->w_topline;
}

/// Get w_save_cursor.w_cursor_save.lnum.
linenr_T nvim_win_get_save_cursor_save_lnum(win_T *wp)
{
  if (!wp) {
    return 0;
  }
  return wp->w_save_cursor.w_cursor_save.lnum;
}

/// Get w_save_cursor.w_topline_save.
linenr_T nvim_win_get_save_topline_save(win_T *wp)
{
  if (!wp) {
    return 0;
  }
  return wp->w_save_cursor.w_topline_save;
}

/// Restore cursor from w_save_cursor.w_cursor_save.
void nvim_win_restore_cursor_from_save(win_T *wp)
{
  if (wp) {
    wp->w_cursor = wp->w_save_cursor.w_cursor_save;
  }
}

/// Restore topline from w_save_cursor.w_topline_save.
void nvim_win_restore_topline_from_save(win_T *wp)
{
  if (wp) {
    wp->w_topline = wp->w_save_cursor.w_topline_save;
  }
}

/// Check if w_save_cursor.w_topline_save > buffer line count.
int nvim_win_save_topline_gt_buf_line_count(win_T *wp)
{
  if (!wp || !wp->w_buffer) {
    return 0;
  }
  return wp->w_save_cursor.w_topline_save > wp->w_buffer->b_ml.ml_line_count;
}

// Growarray accessors for Rust.

/// Initialize a growarray for int items.
void nvim_ga_init_int(garray_T *gap)
{
  ga_init(gap, (int)sizeof(int), 1);
}

/// Grow a growarray to accommodate n more items.
void nvim_ga_grow(garray_T *gap, int n)
{
  ga_grow(gap, n);
}

/// Get the current length of a growarray.
int nvim_ga_get_len(garray_T *gap)
{
  return gap ? gap->ga_len : 0;
}

// nvim_ga_set_len() — already defined in fold.c

/// Get an int item from a growarray by index.
int nvim_ga_get_int(garray_T *gap, int idx)
{
  if (!gap || !gap->ga_data || idx < 0 || idx >= gap->ga_len) {
    return 0;
  }
  return ((int *)gap->ga_data)[idx];
}

/// Set an int item in a growarray by index.
void nvim_ga_set_int(garray_T *gap, int idx, int val)
{
  if (gap && gap->ga_data && idx >= 0) {
    ((int *)gap->ga_data)[idx] = val;
  }
}

// --- Phase 5 accessors ---

/// Wrap comp_col() from drawscreen.c.
void nvim_comp_col(void)
{
  comp_col();
}

/// Compound accessor: clear and free w_status_click_defs.
void nvim_win_stl_clear_click_defs(win_T *wp)
{
  if (!wp) {
    return;
  }
  stl_clear_click_defs(wp->w_status_click_defs, wp->w_status_click_defs_size);
  xfree(wp->w_status_click_defs);
  wp->w_status_click_defs_size = 0;
  wp->w_status_click_defs = NULL;
}

/// Get w_prev_height from a window.
int nvim_win_get_prev_height(win_T *wp)
{
  return wp ? wp->w_prev_height : 0;
}

/// Set w_prev_height for a window.
void nvim_win_set_prev_height(win_T *wp, int val)
{
  if (wp) {
    wp->w_prev_height = val;
  }
}

/// Wrap win_float_anchor_laststatus() from winfloat.c.
void nvim_win_float_anchor_laststatus(void)
{
  win_float_anchor_laststatus();
}

// --- win_split_ins migration accessors ---

/// Set the w_floating field of a window.
void nvim_win_set_floating(win_T *wp, int val)
{
  if (wp) {
    wp->w_floating = val;
  }
}

/// Get the w_fraction field from a window.
int nvim_win_get_fraction(win_T *wp)
{
  return wp ? wp->w_fraction : 0;
}

/// Get the global p_ea (equalalways) value.
int nvim_get_p_ea(void)
{
  return p_ea ? 1 : 0;
}

/// Get the first char of the global p_ead (eadirection) option.
int nvim_get_p_ead_char(void)
{
  return (p_ead && *p_ead) ? (int)(unsigned char)*p_ead : 0;
}

// nvim_get_p_ch() — already defined in message.c
// nvim_get_sc_col() — already defined in message.c

/// Set the global p_wiw (winwidth) value.
void nvim_set_p_wiw(int64_t val)
{
  p_wiw = val;
}

/// Set the global p_wh (winheight) value.
void nvim_set_p_wh(int64_t val)
{
  p_wh = val;
}

/// Wrapper: allocate a new window via win_alloc().
win_T *nvim_win_alloc_wrapper(win_T *after, int hidden)
{
  return win_alloc(after, hidden != 0);
}

/// Wrapper: call new_frame(wp) to allocate and init a frame for a window.
void nvim_new_frame_wrapper(win_T *wp)
{
  new_frame(wp);
}

/// Wrapper: call win_init(wp, oldwin, flags) to init a new window.
void nvim_win_init_wrapper(win_T *wp, win_T *oldwin, int flags)
{
  win_init(wp, oldwin, flags);
}

/// Wrapper: call frame_flatten(frp).
void nvim_frame_flatten_wrapper(frame_T *frp)
{
  frame_flatten(frp);
}

/// Wrapper: allocate a new frame via xcalloc.
frame_T *nvim_xcalloc_frame(void)
{
  return xcalloc(1, sizeof(frame_T));
}

/// Wrapper: ui_comp_remove_grid(&wp->w_grid_alloc).
void nvim_ui_comp_remove_grid_win(win_T *wp)
{
  if (wp) {
    ui_comp_remove_grid(&wp->w_grid_alloc);
  }
}

/// Wrapper: check ui_has(kUIMultigrid).
int nvim_ui_has_multigrid(void)
{
  return ui_has(kUIMultigrid) ? 1 : 0;
}

/// Wrapper: ui_call_win_hide(wp->w_grid_alloc.handle).
void nvim_ui_call_win_hide_win(win_T *wp)
{
  if (wp) {
    ui_call_win_hide(wp->w_grid_alloc.handle);
  }
}

/// Wrapper: win_free_grid(wp, reinit).
void nvim_win_free_grid_wrapper(win_T *wp, int reinit)
{
  if (wp) {
    win_free_grid(wp, reinit != 0);
  }
}

/// Wrapper: merge_win_config(&wp->w_config, WIN_CONFIG_INIT) + CLEAR_FIELD(wp->w_border_adj).
void nvim_merge_win_config_init(win_T *wp)
{
  if (wp) {
    merge_win_config(&wp->w_config, WIN_CONFIG_INIT);
    CLEAR_FIELD(wp->w_border_adj);
  }
}

/// Wrapper: redraw_later(wp, type).
void nvim_redraw_later_wrapper(win_T *wp, int type)
{
  if (wp) {
    redraw_later(wp, type);
  }
}

/// Wrapper: status_redraw_all().
void nvim_status_redraw_all_wrapper(void)
{
  status_redraw_all();
}

/// Wrapper: msg_clr_eos_force().
void nvim_msg_clr_eos_force(void)
{
  msg_clr_eos_force();
}

/// Wrapper: set_fraction(wp).
void nvim_set_fraction_wrapper(win_T *wp)
{
  set_fraction(wp);
}

/// Wrapper: win_setheight_win(height, wp).
void nvim_win_setheight_win_wrapper(int height, win_T *wp)
{
  win_setheight_win(height, wp);
}

/// Wrapper: win_setwidth_win(width, wp).
void nvim_win_setwidth_win_wrapper(int width, win_T *wp)
{
  win_setwidth_win(width, wp);
}

/// Wrapper: win_enter_ext(wp, flags).
void nvim_win_enter_ext_wrapper(win_T *wp, int flags)
{
  win_enter_ext(wp, flags);
}

/// Wrapper: win_valid(wp).
int nvim_win_valid_wrapper(win_T *wp)
{
  return win_valid(wp) ? 1 : 0;
}

/// Wrapper: one_window(firstwin, NULL).
int nvim_one_window_firstwin(void)
{
  return one_window(firstwin, NULL) ? 1 : 0;
}

/// Wrapper: is_aucmd_win(wp).
int nvim_is_aucmd_win(win_T *wp)
{
  return is_aucmd_win(wp) ? 1 : 0;
}

/// Get the w_config.external field from a window.
int nvim_win_get_config_external_int(win_T *wp)
{
  return wp ? (int)wp->w_config.external : 0;
}

/// Iterate over all tabpages, setting tp_curwin to tp_firstwin
/// when tp != curtab && tp->tp_curwin == wp.
void nvim_fixup_external_curwin(win_T *wp)
{
  FOR_ALL_TABS(tp) {
    if (tp != curtab && tp->tp_curwin == wp) {
      tp->tp_curwin = tp->tp_firstwin;
    }
  }
}

/// Set msg_row global.
void nvim_set_msg_row_val(int val)
{
  msg_row = val;
}

/// Set msg_col global.
void nvim_set_msg_col_val(int val)
{
  msg_col = val;
}

/// Set w_frame for a window.
void nvim_win_set_frame(win_T *wp, frame_T *frp)
{
  if (wp) {
    wp->w_frame = frp;
  }
}

// --- Phase 2 accessors: win_close_othertab ---

/// Set first_tabpage global (for Rust tabpage list manipulation).
void nvim_set_first_tabpage(tabpage_T *tp)
{
  first_tabpage = tp;
}

/// Set tp_next field on a tabpage (for Rust tabpage list manipulation).
void nvim_tabpage_set_next(tabpage_T *tp, tabpage_T *next)
{
  tp->tp_next = next;
}

/// Set w_buffer on a window (raw, no side effects).
void nvim_win_set_buffer_raw(win_T *wp, buf_T *buf)
{
  wp->w_buffer = buf;
}

/// Increment b_nwindows on a buffer.
void nvim_buf_inc_nwindows(buf_T *buf)
{
  buf->b_nwindows++;
}

/// Wrapper: win_init_empty(wp).
void nvim_win_init_empty_wrapper(win_T *wp)
{
  win_init_empty(wp);
}

/// Wrapper: emsg(e_floatonly).
void nvim_emsg_e_floatonly(void)
{
  emsg(e_floatonly);
}

/// Wrapper: emsg(_(e_autocmd_close)).
void nvim_emsg_e_autocmd_close(void)
{
  emsg(_(e_autocmd_close));
}

/// Wrapper: internal_error("win_close_othertab()").
void nvim_internal_error_othertab(void)
{
  internal_error("win_close_othertab()");
}

/// Wrapper: win_new_screen_rows().
void nvim_win_new_screen_rows_wrapper(void)
{
  win_new_screen_rows();
}

/// Wrapper: win_free_mem(win, dirp, tp). Returns the window that got the space.
win_T *nvim_win_free_mem_wrapper(win_T *win, int *dirp, tabpage_T *tp)
{
  return win_free_mem(win, dirp, tp);
}

// --- Phase 3 accessors: win_close ---

/// Increment split_disallowed.
void nvim_inc_split_disallowed(void)
{
  split_disallowed++;
}

/// Decrement split_disallowed.
void nvim_dec_split_disallowed(void)
{
  split_disallowed--;
}

/// Wrapper: ui_call_win_close(wp->w_grid_alloc.handle).
void nvim_ui_call_win_close_win(win_T *wp)
{
  if (wp) {
    ui_call_win_close(wp->w_grid_alloc.handle);
  }
}

/// Set tp_curwin on a tabpage.
void nvim_tabpage_set_curwin(tabpage_T *tp, win_T *wp)
{
  tp->tp_curwin = wp;
}

/// Set curbuf = curwin->w_buffer.
void nvim_set_curbuf_from_curwin(void)
{
  curbuf = curwin->w_buffer;
}

/// Wrapper: check_cursor(wp).
void nvim_check_cursor_win_wrapper(win_T *wp)
{
  check_cursor(wp);
}

/// Wrapper: win_equal(wp, current, dir).
void nvim_win_equal_wrapper(win_T *wp, int current, int dir)
{
  win_equal(wp, current != 0, dir);
}

/// Get w_frame->fr_parent from a window (for win_close frame comparison).
frame_T *nvim_win_get_frame_parent(win_T *wp)
{
  if (wp && wp->w_frame) {
    return wp->w_frame->fr_parent;
  }
  return NULL;
}

/// Wrapper: free_tabpage(tp).
void nvim_free_tabpage_wrapper(tabpage_T *tp)
{
  free_tabpage(tp);
}

/// Wrapper: close_buffer(win, buf, action, abort_if_last, ignore_abort).
/// Returns did_decrement (1 if decrement happened).
int nvim_close_buffer_wrapper(win_T *win, buf_T *buf, int action, int abort_if_last,
                              int ignore_abort)
{
  return close_buffer(win, buf, action, abort_if_last != 0, ignore_abort != 0) ? 1 : 0;
}

/// Wrapper: do_autocmd_winclosed(win).
void nvim_do_autocmd_winclosed(win_T *win)
{
  do_autocmd_winclosed(win);
}

/// Get firstbuf global (for buffer recovery in leave_open).
buf_T *nvim_get_firstbuf_wrapper(void)
{
  return firstbuf;
}

/// Wrapper: can_close_floating_windows(tp).
int nvim_can_close_floating_windows(tabpage_T *tp)
{
  return can_close_floating_windows(tp) ? 1 : 0;
}

// =============================================================================
// Phase 4: do_window wrappers
// =============================================================================

/// Wrapper for static win_exchange(Prenum).
void nvim_win_exchange_wrapper(int Prenum)
{
  win_exchange(Prenum);
}

/// Wrapper for static win_rotate(upwards, count).
void nvim_win_rotate_wrapper(int upwards, int count)
{
  win_rotate(upwards != 0, count);
}

/// Wrapper for static win_goto_ver(up, count).
void nvim_win_goto_ver_wrapper(int up, int count)
{
  win_goto_ver(up != 0, count);
}

/// Wrapper for static win_goto_hor(left, count).
void nvim_win_goto_hor_wrapper(int left, int count)
{
  win_goto_hor(left != 0, count);
}

/// Get cmdmod.cmod_split.
int nvim_get_cmdmod_cmod_split(void)
{
  return cmdmod.cmod_split;
}

/// Get cmdmod.cmod_tab.
int nvim_get_cmdmod_cmod_tab(void)
{
  return cmdmod.cmod_tab;
}

/// Set cmdmod.cmod_tab.
void nvim_set_cmdmod_cmod_tab(int val)
{
  cmdmod.cmod_tab = val;
}

/// Get swb_flags global.
unsigned nvim_get_swb_flags(void)
{
  return swb_flags;
}

/// Get p_pvh (preview height) option.
int64_t nvim_get_p_pvh(void)
{
  return p_pvh;
}

/// Wrapper: win_goto(wp).
void nvim_win_goto_wrapper(win_T *wp)
{
  win_goto(wp);
}

/// Wrapper: win_split(size, flags).
int nvim_win_split_wrapper(int size, int flags)
{
  return win_split(size, flags);
}

/// Wrapper: win_splitmove(wp, size, flags).
int nvim_win_splitmove_wrapper(win_T *wp, int size, int flags)
{
  return win_splitmove(wp, size, flags);
}

/// Wrapper: reset_VIsual_and_resel().
void nvim_reset_visual_wrapper(void)
{
  reset_VIsual_and_resel();
}

/// Wrapper: do_cmdline_cmd(cmd).
int nvim_do_cmdline_cmd_wrapper(const char *cmd)
{
  return do_cmdline_cmd(cmd);
}

/// Wrapper: emsg(_(e_cmdwin)).
void nvim_emsg_e_cmdwin(void)
{
  emsg(_(e_cmdwin));
}

/// Wrapper: bt_quickfix(curbuf).
int nvim_bt_quickfix_curbuf(void)
{
  return bt_quickfix(curbuf) ? 1 : 0;
}

/// Wrapper: one_window(curwin, NULL).
int nvim_one_window_curwin(void)
{
  return one_window(curwin, NULL) ? 1 : 0;
}

/// Wrapper: msg(_(m_onlyone), 0).
void nvim_msg_onlyone(void)
{
  msg(_(m_onlyone), 0);
}

/// Wrapper: lastwin_nofloating().
win_T *nvim_lastwin_nofloating_wrapper(void)
{
  return lastwin_nofloating();
}

/// Wrapper: win_valid(prevwin) check for 'p' command.
/// Returns prevwin if valid and focusable, NULL otherwise.
win_T *nvim_get_valid_prevwin(void)
{
  if (!win_valid(prevwin) || prevwin->w_config.hide || !prevwin->w_config.focusable) {
    return NULL;
  }
  return prevwin;
}

/// Wrapper: The 'w'/'W' window navigation.
/// nchar='w' goes forward, nchar='W' goes backward.
/// Prenum selects a specific window number.
void nvim_do_window_wW(int nchar, int Prenum)
{
  if (ONE_WINDOW && Prenum != 1) {
    beep_flush();
  } else {
    win_T *wp;
    if (Prenum) {
      win_T *last_focusable = firstwin;
      for (wp = firstwin; --Prenum > 0;) {
        if (!wp->w_floating || (!wp->w_config.hide && wp->w_config.focusable)) {
          last_focusable = wp;
        }
        if (wp->w_next == NULL) {
          break;
        }
        wp = wp->w_next;
      }
      while (wp != NULL && wp->w_floating
             && (wp->w_config.hide || !wp->w_config.focusable)) {
        wp = wp->w_next;
      }
      if (wp == NULL) {
        wp = last_focusable;
      }
    } else {
      if (nchar == 'W') {
        wp = curwin->w_prev;
        if (wp == NULL) {
          wp = lastwin;
        }
        while (wp != NULL && wp->w_floating
               && (wp->w_config.hide || !wp->w_config.focusable)) {
          wp = wp->w_prev;
        }
      } else {
        wp = curwin->w_next;
        while (wp != NULL && wp->w_floating
               && (wp->w_config.hide || !wp->w_config.focusable)) {
          wp = wp->w_next;
        }
        if (wp == NULL) {
          wp = firstwin;
        }
      }
    }
    win_goto(wp);
  }
}

/// Wrapper: cursor to preview window ('P' command).
void nvim_do_window_P(void)
{
  win_T *wp = NULL;
  FOR_ALL_WINDOWS_IN_TAB(wp2, curtab) {
    if (wp2->w_p_pvw) {
      wp = wp2;
      break;
    }
  }
  if (wp == NULL) {
    emsg(_("E441: There is no preview window"));
  } else {
    win_goto(wp);
  }
}

/// Wrapper: 'T' move-to-tab command.
void nvim_do_window_T(int Prenum)
{
  if (one_window(curwin, NULL)) {
    msg(_(m_onlyone), 0);
  } else {
    tabpage_T *oldtab = curtab;
    win_T *wp = curwin;
    if (win_new_tabpage(Prenum, NULL) == OK
        && valid_tabpage(oldtab)) {
      tabpage_T *newtab = curtab;
      goto_tabpage_tp(oldtab, true, true);
      if (curwin == wp) {
        win_close(curwin, false, false);
      }
      if (valid_tabpage(newtab)) {
        goto_tabpage_tp(newtab, true, true);
        apply_autocmds(EVENT_TABNEWENTERED, NULL, NULL, false, curbuf);
      }
    }
  }
}

/// Wrapper: '^' split-and-edit-alternate command.
void nvim_do_window_hat(int Prenum)
{
  reset_VIsual_and_resel();

  if (buflist_findnr(Prenum == 0 ? curwin->w_alt_fnum : Prenum) == NULL) {
    if (Prenum == 0) {
      emsg(_(e_noalt));
    } else {
      semsg(_("E92: Buffer %" PRId64 " not found"), (int64_t)Prenum);
    }
    return;
  }

  if (!curbuf_locked() && win_split(0, 0) == OK) {
    buflist_getfile(Prenum == 0 ? curwin->w_alt_fnum : Prenum,
                    0, GETF_ALT, false);
  }
}

/// Wrapper: 'n'/'N' new window, including the 'newwindow' goto label logic.
/// nchar is the original command char (to detect 'v'/'V' for vertical).
void nvim_do_window_new(int nchar, int Prenum)
{
  char cbuf[40];
  if (Prenum) {
    vim_snprintf(cbuf, sizeof(cbuf) - 5, "%" PRId64, (int64_t)Prenum);
  } else {
    cbuf[0] = NUL;
  }
  if (nchar == 'v' || nchar == Ctrl_V) {
    xstrlcat(cbuf, "v", sizeof(cbuf));
  }
  xstrlcat(cbuf, "new", sizeof(cbuf));
  do_cmdline_cmd(cbuf);
}

/// Wrapper: cmd_with_count + do_cmdline_cmd.
void nvim_cmd_with_count_exec(const char *cmd, int64_t Prenum)
{
  char cbuf[40];
  size_t len = xstrlcpy(cbuf, cmd, sizeof(cbuf));
  if (Prenum > 0 && len < sizeof(cbuf)) {
    vim_snprintf(cbuf + len, sizeof(cbuf) - len, "%" PRId64, Prenum);
  }
  do_cmdline_cmd(cbuf);
}

/// Wrapper: The '=' equalize command.
void nvim_do_window_equalize(void)
{
  int mod = cmdmod.cmod_split & (WSP_VERT | WSP_HOR);
  win_equal(NULL, false, mod == WSP_VERT ? 'v' : mod == WSP_HOR ? 'h' : 'b');
}

/// Wrapper: The tag/preview commands (']', '}', Ctrl-]).
void nvim_do_window_tag(int nchar, int Prenum)
{
  if (nchar == '}') {
    if (Prenum) {
      g_do_tagpreview = Prenum;
    } else {
      g_do_tagpreview = (int)p_pvh;
    }
  }

  if (Prenum) {
    postponed_split = Prenum;
  } else {
    postponed_split = -1;
  }

  if (nchar != '}') {
    g_do_tagpreview = 0;
  }

  do_nv_ident(Ctrl_RSB, NUL);
  postponed_split = 0;
}

/// Wrapper: The 'f'/'F'/Ctrl-F file-goto command.
void nvim_do_window_goto_file(int nchar, int Prenum1)
{
  if (check_text_or_curbuf_locked(NULL)) {
    return;
  }

  linenr_T lnum = -1;
  char *ptr = grab_file_name(Prenum1, &lnum);
  if (ptr != NULL) {
    tabpage_T *oldtab = curtab;
    win_T *oldwin = curwin;
    setpcmark();

    win_T *wp = NULL;
    if ((swb_flags & (kOptSwbFlagUseopen | kOptSwbFlagUsetab))
        && cmdmod.cmod_tab == 0) {
      wp = swbuf_goto_win_with_buf(buflist_findname_exp(ptr));
    }

    if (wp == NULL && win_split(0, 0) == OK) {
      RESET_BINDING(curwin);
      if (do_ecmd(0, ptr, NULL, NULL, ECMD_LASTL, ECMD_HIDE, NULL) == FAIL) {
        win_close(curwin, false, false);
        goto_tabpage_win(oldtab, oldwin);
      } else {
        wp = curwin;
      }
    }

    if (wp != NULL && nchar == 'F' && lnum >= 0) {
      curwin->w_cursor.lnum = lnum;
      check_cursor_lnum(curwin);
      beginline(BL_SOL | BL_FIX);
    }
    xfree(ptr);
  }
}

/// Wrapper: The 'i'/'d' find-in-path command.
void nvim_do_window_find_in_path(int nchar, int Prenum, int Prenum1)
{
  int type = FIND_DEFINE;
  if (nchar == 'i' || nchar == Ctrl_I) {
    type = FIND_ANY;
  }

  size_t len;
  char *ptr;
  if ((len = find_ident_under_cursor(&ptr, FIND_IDENT)) == 0) {
    return;
  }

  ptr = xmemdupz(ptr, len);
  find_pattern_in_path(ptr, 0, len, true, Prenum == 0, type,
                       Prenum1, ACTION_SPLIT, 1, MAXLNUM, false, false);
  xfree(ptr);
  curwin->w_set_curswant = true;
}

/// Wrapper: The 'g' sub-switch.
void nvim_do_window_g(int Prenum, int xchar)
{
  int Prenum1 = Prenum == 0 ? 1 : Prenum;

  no_mapping++;
  allow_keys++;
  if (xchar == NUL) {
    xchar = plain_vgetc();
  }
  LANGMAP_ADJUST(xchar, true);
  no_mapping--;
  allow_keys--;
  add_to_showcmd(xchar);

  switch (xchar) {
  case '}':
    xchar = Ctrl_RSB;
    if (Prenum) {
      g_do_tagpreview = Prenum;
    } else {
      g_do_tagpreview = (int)p_pvh;
    }
    FALLTHROUGH;
  case ']':
  case Ctrl_RSB:
    if (Prenum) {
      postponed_split = Prenum;
    } else {
      postponed_split = -1;
    }
    do_nv_ident('g', xchar);
    postponed_split = 0;
    break;

  case 'f':
  case 'F':
    cmdmod.cmod_tab = tabpage_index(curtab) + 1;
    nvim_do_window_goto_file(xchar, Prenum1);
    break;

  case 't':
    goto_tabpage(Prenum);
    break;

  case 'T':
    goto_tabpage(-Prenum1);
    break;

  case TAB:
    if (!goto_tabpage_lastused()) {
      beep_flush();
    }
    break;

  case 'e':
    if (curwin->w_floating || !ui_has(kUIMultigrid)) {
      beep_flush();
      break;
    }
    WinConfig config = WIN_CONFIG_INIT;
    config.width = curwin->w_width;
    config.height = curwin->w_height;
    config.external = true;
    Error err = ERROR_INIT;
    if (!win_new_float(curwin, false, config, &err)) {
      emsg(err.msg);
      api_clear_error(&err);
      beep_flush();
    }
    break;
  default:
    beep_flush();
    break;
  }
}

/// Wrapper: qf_view_result(true).
void nvim_qf_view_result_split(void)
{
  qf_view_result(true);
}

_Static_assert(16384 == FRACTION_MULT, "FRACTION_MULT mismatch");
_Static_assert(2 == MIN_LINES, "MIN_LINES mismatch");
_Static_assert(2 == SNAP_COUNT, "SNAP_COUNT mismatch");
_Static_assert(0 == SNAP_HELP_IDX, "SNAP_HELP_IDX mismatch");
_Static_assert(1 == SNAP_AUCMD_IDX, "SNAP_AUCMD_IDX mismatch");
_Static_assert(0x80 == VALID_TOPLINE, "VALID_TOPLINE mismatch");
_Static_assert(1 == STATUS_HEIGHT, "STATUS_HEIGHT mismatch");

// WSP and WEE flag static asserts for Rust constant validation
_Static_assert(0x01 == WSP_ROOM, "WSP_ROOM mismatch");
_Static_assert(0x02 == WSP_VERT, "WSP_VERT mismatch");
_Static_assert(0x04 == WSP_HOR, "WSP_HOR mismatch");
_Static_assert(0x08 == WSP_TOP, "WSP_TOP mismatch");
_Static_assert(0x10 == WSP_BOT, "WSP_BOT mismatch");
_Static_assert(0x20 == WSP_HELP, "WSP_HELP mismatch");
_Static_assert(0x40 == WSP_BELOW, "WSP_BELOW mismatch");
_Static_assert(0x80 == WSP_ABOVE, "WSP_ABOVE mismatch");
_Static_assert(0x100 == WSP_NEWLOC, "WSP_NEWLOC mismatch");
_Static_assert(0x200 == WSP_NOENTER, "WSP_NOENTER mismatch");
_Static_assert(0x01 == WEE_UNDO_SYNC, "WEE_UNDO_SYNC mismatch");
_Static_assert(0x02 == WEE_CURWIN_INVALID, "WEE_CURWIN_INVALID mismatch");
_Static_assert(0x04 == WEE_TRIGGER_NEW_AUTOCMDS, "WEE_TRIGGER_NEW_AUTOCMDS mismatch");
_Static_assert(0x08 == WEE_TRIGGER_ENTER_AUTOCMDS, "WEE_TRIGGER_ENTER_AUTOCMDS mismatch");
_Static_assert(0x10 == WEE_TRIGGER_LEAVE_AUTOCMDS, "WEE_TRIGGER_LEAVE_AUTOCMDS mismatch");
_Static_assert(40 == UPD_NOT_VALID, "UPD_NOT_VALID mismatch");
_Static_assert(2 == DOBUF_UNLOAD, "DOBUF_UNLOAD mismatch");

// Key code static asserts for do_window Rust dispatch
_Static_assert(-30059 == K_UP, "K_UP mismatch");
_Static_assert(-25707 == K_DOWN, "K_DOWN mismatch");
_Static_assert(-27755 == K_LEFT, "K_LEFT mismatch");
_Static_assert(-29291 == K_RIGHT, "K_RIGHT mismatch");
_Static_assert(-25195 == K_BS, "K_BS mismatch");
_Static_assert(-16715 == K_KENTER, "K_KENTER mismatch");
_Static_assert(30 == Ctrl_HAT, "Ctrl_HAT mismatch");
_Static_assert(29 == Ctrl_RSB, "Ctrl_RSB mismatch");
_Static_assert(31 == Ctrl__, "Ctrl__ mismatch");
