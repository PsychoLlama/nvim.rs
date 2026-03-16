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
#include "nvim/mbyte.h"
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
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
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

#include "window_shim.c.generated.h"

// Static assertions for bulk snapshot struct sizes (types defined in window.h).
_Static_assert(sizeof(WinSnapshot) == 6 * sizeof(int), "WinSnapshot size mismatch");
_Static_assert(sizeof(WinViewportSnapshot) == 4 * sizeof(int32_t), "WinViewportSnapshot size mismatch");

// Rust FFI declarations (tag module)
extern void rs_tagstack_clear_entry(void *tg);
extern size_t rs_find_ident_under_cursor(char **text, int find_type);

// Rust fold FFI declarations
extern void rs_copyFoldingState(win_T *wp_from, win_T *wp_to);

extern int rs_tabline_height(void);
extern int rs_global_stl_height(void);
extern int rs_win_valid(win_T *win);
extern int rs_win_valid_any_tab(win_T *win);
extern int rs_tabpage_index(tabpage_T *ftp);
extern tabpage_T *rs_win_find_tabpage(win_T *win);

// Result structure from rs_winframe_find_altwin
typedef struct {
  frame_T *altfr;
  int dir;
} WinframeResult;
extern WinframeResult rs_winframe_find_altwin(win_T *wp, frame_T *altfr_initial);

// New Rust replacements for frame tree operations

// rs_alt_tabpage: dead (Phase 15)


// Split helper functions from Rust

// Close validation functions from Rust

// Pure calculations and thin wrappers
extern void rs_win_setheight(int height);

// Height/width setters
// rs_frame_new_height deleted: now exported as frame_new_height via #[export_name]

// Win exchange / rotate

// Snapshot lifecycle

// Utility and validation helpers
// rs_check_can_set_curbuf_disabled deleted: now exported as check_can_set_curbuf_disabled via #[export_name]
// rs_check_can_set_curbuf_forceit deleted: now exported as check_can_set_curbuf_forceit via #[export_name]
// rs_check_split_disallowed_err deleted: now exported as check_split_disallowed_err via #[export_name]

// Tabpage helpers
// rs_goto_tabpage_lastused deleted: now exported as goto_tabpage_lastused via #[export_name]

// can_close_floating_windows, maximum_wincount
// rs_can_close_floating_windows_tp: removed from C declarations (Rust uses #[link_name] directly)
// make_windows deleted: now exported from Rust utility.rs via #[export_name = "make_windows"]

// rs_win_fix_scroll deleted: now exported as win_fix_scroll via #[export_name]

// do_autocmd_winclosed, can_close_in_cmdwin, set_winbar_win, set_winbar
// rs_can_close_in_cmdwin deleted: now exported as can_close_in_cmdwin via #[export_name]
// rs_set_winbar_win deleted: now exported as set_winbar_win via #[export_name]
// rs_set_winbar deleted: now exported as set_winbar via #[export_name]

// Status line management

// goto_tabpage_tp deleted: now exported from Rust tabpage.rs via #[export_name = "goto_tabpage_tp"]

// win_split_ins migration: Rust orchestrator
typedef struct {
  win_T *wp;           // new window or NULL
  int do_enter;        // whether to call win_enter_ext
  int enter_flags;     // WEE_* flags for win_enter_ext
  int vertical;        // 1 if vertical split
  int saved_option;    // saved p_wiw or p_wh value
} SplitInsResult;

// win_close deleted: now exported from Rust win_close.rs via #[export_name = "win_close"]
extern int rs_win_close_othertab(win_T *win, int free_buf, tabpage_T *tp, int force);
// rs_win_free_mem still used by win_free_all
extern win_T *rs_win_free_mem(win_T *win, int *dirp, tabpage_T *tp);

int *nvim_win_get_ns_hl_attr(win_T *wp) { return wp->w_ns_hl_attr; }
int nvim_win_get_p_winbl(win_T *wp) { return (int)wp->w_p_winbl; }



ScreenGrid *nvim_get_curwin_grid_alloc(void) { return curwin ? &curwin->w_grid_alloc : NULL; }

/// Get the current window's cursor position for incsearch state (accessor for Rust).
/// Fills the provided pos struct with lnum, col, coladd.
void nvim_get_curwin_cursor_pos(void *pos) { int32_t *p = (int32_t *)pos; p[0] = (int32_t)curwin->w_cursor.lnum; p[1] = (int32_t)curwin->w_cursor.col; p[2] = (int32_t)curwin->w_cursor.coladd; }

/// Save current window view state (accessor for Rust).
/// Fills the provided viewstate struct.
void nvim_save_viewstate(void *vs) { int32_t *p = (int32_t *)vs; p[0] = (int32_t)curwin->w_curswant; p[1] = (int32_t)curwin->w_leftcol; p[2] = (int32_t)curwin->w_skipcol; p[3] = (int32_t)curwin->w_topline; p[4] = (int32_t)curwin->w_topfill; p[5] = (int32_t)curwin->w_botline; p[6] = (int32_t)curwin->w_empty_rows; }

/// Restore current window view state (accessor for Rust).
void nvim_restore_viewstate(const void *vs) { const int32_t *p = (const int32_t *)vs; curwin->w_curswant = (colnr_T)p[0]; curwin->w_leftcol = (colnr_T)p[1]; curwin->w_skipcol = (colnr_T)p[2]; curwin->w_topline = (linenr_T)p[3]; curwin->w_topfill = (int)p[4]; curwin->w_botline = (linenr_T)p[5]; curwin->w_empty_rows = (int)p[6]; }

/// Set the current window's cursor position (accessor for Rust).
void nvim_set_curwin_cursor_pos(const void *pos) { const int32_t *p = (const int32_t *)pos; curwin->w_cursor.lnum = (linenr_T)p[0]; curwin->w_cursor.col = (colnr_T)p[1]; curwin->w_cursor.coladd = (colnr_T)p[2]; }

/// Compare two positions for equality (accessor for Rust).
int nvim_equalpos(const void *pos1, const void *pos2) { const int32_t *p1 = (const int32_t *)pos1; const int32_t *p2 = (const int32_t *)pos2; return p1[0] == p2[0] && p1[1] == p2[1] && p1[2] == p2[2]; }

// nvim_validate_cursor: migrated to Rust wrappers.rs (Phase 8)
char nvim_win_get_fdm_char(win_T *wp, int idx) { return wp->w_p_fdm[idx]; }
int nvim_win_buf_has_terminal(win_T *wp) { return wp->w_buffer->terminal != NULL; }
int nvim_win_folds_empty(win_T *wp) { return GA_EMPTY(&wp->w_folds); }

// Note: nvim_win_get_skipcol is defined later in window.c (returns colnr_T)

const char *nvim_win_get_w_p_fdc(win_T *wp) { return wp->w_p_fdc; }
int nvim_win_is_curwin(win_T *wp) { return wp == curwin; }
char *nvim_win_get_p_stc(win_T *wp) { return wp->w_p_stc; }
char *nvim_win_get_p_cocu(win_T *wp) { return wp->w_p_cocu; }
linenr_T nvim_win_buf_line_count(win_T *wp) { return wp->w_buffer->b_ml.ml_line_count; }
// nvim_win_buf_meta_total_signtext: migrated to Rust wrappers.rs (Phase 9)
void nvim_frame_set_height(frame_T *frp, int val) { frp->fr_height = val; }
void nvim_frame_set_width(frame_T *frp, int val) { frp->fr_width = val; }
void nvim_win_config_float(win_T *wp) { win_config_float(wp, wp->w_config); }
void nvim_win_fix_scroll(bool upd_topline) { win_fix_scroll(upd_topline); }
// nvim_redraw_all_later: migrated to Rust wrappers.rs (Phase 8)
int nvim_get_real_state(void) { return get_real_state(); }
// nvim_win_buf_meta_total_lines: migrated to Rust wrappers.rs (Phase 9)
int nvim_win_is_cmdwin(win_T *wp) { return wp == cmdwin_win; }
char *nvim_win_get_p_sbr(win_T *wp) { return wp->w_p_sbr; }
// Colorcolumn accessors
char *nvim_win_get_p_cc(win_T *wp) { return wp->w_p_cc; }
int64_t nvim_win_get_buf_b_p_tw(win_T *wp) { return wp->w_buffer->b_p_tw; }
int nvim_win_has_buffer(win_T *wp) { return wp->w_buffer != NULL; }
int *nvim_win_get_p_cc_cols(win_T *wp) { return wp->w_p_cc_cols; }
void nvim_win_set_p_cc_cols(win_T *wp, int *cols) { wp->w_p_cc_cols = cols; }
void nvim_win_free_p_cc_cols(win_T *wp) { xfree(wp->w_p_cc_cols); wp->w_p_cc_cols = NULL; }
int nvim_first_tabpage_has_next(void) { return first_tabpage != NULL && first_tabpage->tp_next != NULL; }
// nvim_win_get_endrow: migrated to Rust wrappers.rs (Phase 9)
// nvim_win_get_endcol: migrated to Rust wrappers.rs (Phase 9)
int nvim_win_get_wrap_flags(win_T *wp) { return wp->w_p_wrap_flags; }
int nvim_win_get_p_culopt_flags(win_T *wp) { return wp->w_p_culopt_flags; }

// NOTE: nvim_win_set_lines_valid already defined earlier in this file

int nvim_win_argcount(win_T *wp) { return WARGCOUNT(wp); }

/// Set the valid cursor position (all fields).

colnr_T nvim_win_get_valid_leftcol(win_T *wp) { return wp->w_valid_leftcol; }
colnr_T nvim_win_get_valid_skipcol(win_T *wp) { return wp->w_valid_skipcol; }
int nvim_win_get_viewport_invalid(win_T *wp) { return wp->w_viewport_invalid ? 1 : 0; }
void *nvim_win_get_w_grid(win_T *wp) { return &wp->w_grid; }
bool nvim_win_get_briopt_sbr(win_T *wp) { return wp->w_briopt_sbr; }
int nvim_win_hl_attr(win_T *wp, int hlf) { return win_hl_attr(wp, hlf); }
buf_T *nvim_win_get_buffer(win_T *wp) { return wp->w_buffer; }
void nvim_win_set_p_wfb(win_T *wp, int val) { wp->w_p_wfb = val != 0; }
const char *nvim_win_ml_get_buf(win_T *wp, linenr_T lnum) { return ml_get_buf(wp->w_buffer, lnum); }
colnr_T nvim_win_ml_get_buf_len(win_T *wp, linenr_T lnum) { return ml_get_buf_len(wp->w_buffer, lnum); }
// nvim_ui_has_tabline: migrated to Rust wrappers.rs (Phase 8)


#define NOWIN           ((win_T *)-1)   // non-existing window

#define ROWS_AVAIL (Rows - p_ch - rs_tabline_height() - rs_global_stl_height())


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


// check_can_set_curbuf_disabled deleted: now exported from Rust utility.rs via #[export_name]
// check_can_set_curbuf_forceit deleted: now exported from Rust utility.rs via #[export_name]

// swbuf_goto_win_with_buf: thin wrapper defined in Phase 2 accessors section.

// 'cmdheight' value explicitly set by the user: window commands are allowed to
// resize the topframe to values higher than this minimum, but not lower.
static OptInt min_set_ch = 1;

OptInt nvim_get_min_set_ch(void) { return min_set_ch; }
// nvim_set_cmdheight_option deleted: logic migrated to Rust resize/frame.rs (Phase 8)



void win_set_buf(win_T *win, buf_T *buf, Error *err)
  FUNC_ATTR_NONNULL_ALL
{
  const handle_T win_handle = win->handle;
  tabpage_T *tab = rs_win_find_tabpage(win);

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

// ui_ext_win_position deleted: now exported from Rust events.rs via #[export_name]

// check_split_disallowed_err deleted: now exported from Rust utility.rs via #[export_name]

// make_windows deleted: now exported from Rust utility.rs via #[export_name = "make_windows"]

// can_close_floating_windows deleted: logic migrated to Rust close/win_close.rs (Phase 11)

// can_close_in_cmdwin deleted: now exported from Rust focus.rs via #[export_name = "can_close_in_cmdwin"]

/// Close the possibly last window in a tab page.
///
/// @param  win          window to close
/// @param  free_buf     whether to free the window's current buffer
/// @param  prev_curtab  previous tabpage that will be closed if "win" is the
///                      last window in the tabpage
///
/// @return false if there are other windows and nothing is done, true otherwise.
// close_last_window_tabpage deleted: logic migrated to Rust close/helpers.rs (Phase 10)

// win_close_buffer deleted: logic migrated to Rust close/helpers.rs (Phase 10)

// win_close deleted: now exported from Rust win_close.rs via #[export_name = "win_close"]

// do_autocmd_winclosed deleted: logic migrated to Rust close/win_close.rs (Phase 11)

// Close window "win" in tab page "tp", which is not the current tab page.
// This may be the last window in that tab page and result in closing the tab,
// thus "tp" may become invalid!
// Caller must check if buffer is hidden and whether the tabline needs to be
// updated.
// @return false when the window was not closed as a direct result of this call
//         (e.g: not via autocmds).
// Body migrated to Rust rs_win_close_othertab (Phase 11).
bool win_close_othertab(win_T *win, int free_buf, tabpage_T *tp, bool force)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_win_close_othertab(win, free_buf, tp, force ? 1 : 0) != 0;
}

// win_free_mem deleted: logic migrated to Rust close/helpers.rs (Phase 10)

// win_free_all deleted: now exported from Rust free.rs via #[export_name]

#if defined(EXITFREE)
#endif

// alt_tabpage: dead static wrapper (Phase 15)

/// Set a new height for a frame.  Recursively sets the height for contained
/// frames and windows.  Caller must take care of positions.
///
/// @param topfirst  resize topmost contained frame first.
// frame_new_height deleted: Rust exports under the C name directly via #[export_name].

// frame_new_width: dead static wrapper (Phase 15)

/// Try to close all windows except current one.
/// Buffers in the other windows become hidden if 'hidden' is set, or '!' is
/// used and the buffer was modified.
///
/// Used by ":bdel" and ":only".
///
/// @param forceit  always hide all other windows

// When switching tabpage, handle other side-effects in command_height(), but
// avoid setting frame sizes which are still correct.
static bool command_frame_height = true;

// Phase 12 compound C accessors for rs_win_alloc_first / rs_win_alloc_firstwin
// / rs_win_alloc_aucmd_win / rs_win_free_all

/// buflist_new(NULL, NULL, 1, BLN_LISTED): allocate initial buffer.
buf_T *nvim_buflist_new_initial(void) { return buflist_new(NULL, NULL, 1, BLN_LISTED); }

/// Set curwin->w_buffer = buf, curwin->w_s, curbuf->b_nwindows = 1, curwin->w_alist.
void nvim_win_setup_first_buffer(win_T *wp, buf_T *buf)
{
  curwin = wp;
  curbuf = buf;
  wp->w_buffer = buf;
  wp->w_s = &(buf->b_s);
  buf->b_nwindows = 1;
  wp->w_alist = &global_alist;
}

/// curwin_init() for current window.
// nvim_curwin_init: migrated to Rust wrappers.rs (Phase 8)

/// RESET_BINDING for window wp.
void nvim_win_reset_binding(win_T *wp) { RESET_BINDING(wp); }

/// Set topframe from wp->w_frame and compute frame dimensions.
void nvim_alloc_firstwin_set_topframe(win_T *wp)
{
  topframe = wp->w_frame;
  topframe->fr_width = Columns;
  topframe->fr_height = Rows - (int)p_ch - rs_global_stl_height();
}


/// Allocate and initialize aucmd_win[idx] as a hidden float.
void nvim_win_alloc_aucmd_win_impl(int idx)
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

/// Clear cmdwin state (for win_free_all).
void nvim_clear_cmdwin_state(void)
{
  cmdwin_type = 0;
  cmdwin_buf = NULL;
  cmdwin_win = NULL;
  cmdwin_old_curwin = NULL;
}

/// tabpage_close(true) -- close a tab during EXITFREE.
void nvim_tabpage_close_once(void) { tabpage_close(true); }

/// Returns AUCMD_WIN_COUNT.
int nvim_aucmd_win_count(void) { return AUCMD_WIN_COUNT; }

/// Returns aucmd_win[idx].auc_win.
win_T *nvim_aucmd_win_get(int idx) { return aucmd_win[idx].auc_win; }

/// aucmd_win[idx].auc_win = NULL.
void nvim_aucmd_win_clear(int idx) { aucmd_win[idx].auc_win = NULL; }

/// kv_destroy(aucmd_win_vec).
void nvim_kv_destroy_aucmd_win_vec(void) { kv_destroy(aucmd_win_vec); }


// rs_win_free_all deleted: now exported as win_free_all via #[export_name]

// win_alloc_firstwin: dead static wrapper (Phase 15)

// =============================================================================
// Phase 12 compound C accessors for rs_alloc_tabpage / rs_free_tabpage
// =============================================================================

/// Allocate a raw tabpage_T (xcalloc only).
tabpage_T *nvim_alloc_tabpage_raw(void) { return xcalloc(1, sizeof(tabpage_T)); }

/// Increment last_tp_handle, set tp->handle, and insert into tabpage_handles map.
void nvim_tabpage_init_handle(tabpage_T *tp)
{
  static int last_tp_handle = 0;
  tp->handle = ++last_tp_handle;
  pmap_put(int)(&tabpage_handles, tp->handle, tp);
}

/// Allocate tp_vars dict and initialize t: variable scope.
void nvim_tabpage_init_vars(tabpage_T *tp)
{
  tp->tp_vars = tv_dict_alloc();
  init_var_dict(tp->tp_vars, &tp->tp_winvar, VAR_SCOPE);
}

// nvim_tabpage_set_diff_invalid: already defined in diff_shim.c

/// Set tp->tp_ch_used = p_ch.
void nvim_tabpage_set_ch_used_from_p_ch(tabpage_T *tp)
{
  tp->tp_ch_used = p_ch;
}

/// Remove tp from tabpage_handles map.
void nvim_tabpage_pmap_del(tabpage_T *tp)
{
  pmap_del(int)(&tabpage_handles, tp->handle, NULL);
}

/// Clear t: variables and unref the dict.
void nvim_tabpage_clear_vars(tabpage_T *tp)
{
  vars_clear(&tp->tp_vars->dv_hashtab);
  hash_init(&tp->tp_vars->dv_hashtab);
  unref_var_dict(tp->tp_vars);
}

/// Clear tp->tp_localdir and tp->tp_prevdir (xfree both).
void nvim_tabpage_free_dirs(tabpage_T *tp)
{
  xfree(tp->tp_localdir);
  xfree(tp->tp_prevdir);
}

// nvim_xfree_tabpage_raw: migrated to Rust wrappers.rs (Phase 8)



/// Allocate a raw frame_T (xcalloc only).
frame_T *nvim_alloc_frame_raw(void) { return xcalloc(1, sizeof(frame_T)); }

// alloc_tabpage, new_frame: dead static wrappers (Phase 15)

// win_new_tabpage: exported directly from Rust (Phase 15)

// nvim_win_alloc_firstwin_wrapper deleted: callers updated to call rs_win_alloc_firstwin directly (Phase 12)

/// Copy tp_localdir from src to dst with xstrdup (NULL-safe).
void nvim_tabpage_copy_localdir(tabpage_T *dst, tabpage_T *src)
{
  if (dst && src) {
    dst->tp_localdir = src->tp_localdir ? xstrdup(src->tp_localdir) : NULL;
  }
}

/// Free a tabpage on failure path (xfree only).
void nvim_xfree_tabpage(tabpage_T *tp) { xfree(tp); }

/// If curbuf has a terminal, call terminal_check_size on it.
void nvim_curbuf_terminal_check_size(void)
{
  if (curbuf->terminal) {
    terminal_check_size(curbuf->terminal);
  }
}

/// Fire EVENT_TABNEW autocmd with optional filename.
void nvim_apply_autocmds_tabnew(const char *filename)
{
  apply_autocmds(EVENT_TABNEW, (char *)filename, (char *)filename, false, curbuf);
}

// win_new_tabpage, make_tabpages, close_tabpage: exported directly from Rust (Phase 15)

// leave_tabpage, enter_tabpage: dead static wrappers (Phase 15)

/// tells external UI that windows and inline floats in old_curtab are invisible
/// and that floats in curtab is now visible.
///
/// External floats are considered independent of tabpages. This is
/// implemented by always moving them to curtab.
// goto_tabpage: exported directly from Rust (Phase 15)

// goto_tabpage_tp deleted: now exported from Rust tabpage.rs via #[export_name = "goto_tabpage_tp"]

// goto_tabpage_lastused deleted: now exported from Rust tabpage.rs via #[export_name]
// goto_tabpage_win, tabpage_move: exported directly from Rust (Phase 15)

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
extern void rs_win_enter_ext(win_T *wp, int flags);
static void win_enter_ext(win_T *const wp, const int flags) { rs_win_enter_ext(wp, flags); }

// win_fix_current_dir, buf_jump_open_win, buf_jump_open_tab: thin wrappers
// defined below in Phase 2 accessors section.

static int last_win_id = LOWEST_WIN_ID - 1;

int nvim_get_last_win_id(void) { return last_win_id; }

// =============================================================================
// Phase 12 compound C accessors for rs_win_alloc / rs_free_wininfo
// =============================================================================

/// Allocate raw win_T via xcalloc.
win_T *nvim_alloc_win_raw(void) { return xcalloc(1, sizeof(win_T)); }

/// Increment last_win_id, set wp->handle, insert into window_handles map.
void nvim_win_init_handle(win_T *wp)
{
  wp->handle = ++last_win_id;
  pmap_put(int)(&window_handles, wp->handle, wp);
}

/// Enable mouse on grid and assign a grid handle.
void nvim_win_init_grid(win_T *wp)
{
  wp->w_grid_alloc.mouse_enabled = true;
  grid_assign_handle(&wp->w_grid_alloc);
}

/// Allocate w_vars dict and initialize w: variable scope.
void nvim_win_init_vars(win_T *wp)
{
  wp->w_vars = tv_dict_alloc();
  init_var_dict(wp->w_vars, &wp->w_winvar, VAR_SCOPE);
}

/// Initialize w_ns_set with SET_INIT and set w_ns_hl = -1.
void nvim_win_init_ns_set(win_T *wp)
{
  wp->w_ns_hl = -1;
  Set(uint32_t) ns_set = SET_INIT;
  wp->w_ns_set = ns_set;
}

/// Set global-local scroll offset options to -1 (use global value).
void nvim_win_init_global_local_opts(win_T *wp)
{
  wp->w_allbuf_opt.wo_so = wp->w_p_so = -1;
  wp->w_allbuf_opt.wo_siso = wp->w_p_siso = -1;
}

/// Set wp->w_config = WIN_CONFIG_INIT.
void nvim_win_set_config_init(win_T *wp)
{
  WinConfig init = WIN_CONFIG_INIT;
  wp->w_config = init;
}

/// Set w_next_match_id to 1000 (user-visible IDs start above this).
void nvim_win_set_next_match_id(win_T *wp) { wp->w_next_match_id = 1000; }

/// Compound: clear_winopt + deleteFoldRecurse + xfree for a WinInfo.
void nvim_free_wininfo_raw(WinInfo *wip, buf_T *bp)
{
  if (wip->wi_optset) {
    clear_winopt(&wip->wi_opt);
    deleteFoldRecurse(bp, &wip->wi_folds);
  }
  xfree(wip);
}

// =============================================================================
// win_alloc: thin wrapper calling rs_win_alloc
// =============================================================================

// win_alloc deleted: Rust exports under the C name directly via #[export_name = "win_alloc"].

// Phase 12 compound C accessors for rs_win_free / rs_win_free_grid

/// pmap_del the window handle from window_handles.
void nvim_win_pmap_del(win_T *wp) { pmap_del(int)(&window_handles, wp->handle, NULL); }

/// alist_unlink for the window's argument list.
void nvim_win_alist_unlink(win_T *wp) { alist_unlink(wp->w_alist); }

/// Destroy the w_ns_set (uint32_t Set).
void nvim_win_clear_ns_set(win_T *wp) { set_destroy(uint32_t, &wp->w_ns_set); }

/// Clear both winopt structs.
void nvim_win_clear_winopts(win_T *wp)
{
  clear_winopt(&wp->w_onebuf_opt);
  clear_winopt(&wp->w_allbuf_opt);
}

/// Free lcs_chars multispace/leadmultispace.
void nvim_win_free_lcs_chars(win_T *wp)
{
  xfree(wp->w_p_lcs_chars.multispace);
  xfree(wp->w_p_lcs_chars.leadmultispace);
}

/// Free all w: variables.
void nvim_win_clear_vars(win_T *wp)
{
  vars_clear(&wp->w_vars->dv_hashtab);
  hash_init(&wp->w_vars->dv_hashtab);
  unref_var_dict(wp->w_vars);
}

/// Free w_lines.
void nvim_win_free_lines(win_T *wp) { xfree(wp->w_lines); }

/// Clear the tagstack entries.
void nvim_win_clear_tagstack(win_T *wp)
{
  for (int i = 0; i < wp->w_tagstacklen; i++) {
    rs_tagstack_clear_entry(&wp->w_tagstack[i]);
  }
}

/// Free w_localdir and w_prevdir.
void nvim_win_free_dirs(win_T *wp) { xfree(wp->w_localdir); xfree(wp->w_prevdir); }

/// Clear all three click_defs arrays.
void nvim_win_clear_click_defs_all(win_T *wp)
{
  stl_clear_click_defs(wp->w_status_click_defs, wp->w_status_click_defs_size);
  xfree(wp->w_status_click_defs);
  stl_clear_click_defs(wp->w_winbar_click_defs, wp->w_winbar_click_defs_size);
  xfree(wp->w_winbar_click_defs);
  stl_clear_click_defs(wp->w_statuscol_click_defs, wp->w_statuscol_click_defs_size);
  xfree(wp->w_statuscol_click_defs);
}

/// Remove window from all b_wininfo kvecs (FOR_ALL_BUFFERS loop).
void nvim_win_cleanup_b_wininfo(win_T *wp)
{
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
}

/// Clear border text virttext.
void nvim_win_clear_config_virttext(win_T *wp)
{
  clear_virttext(&wp->w_config.title_chunks);
  clear_virttext(&wp->w_config.footer_chunks);
}

/// Free w_p_cc_cols.
void nvim_win_free_cc_cols(win_T *wp) { xfree(wp->w_p_cc_cols); }

/// grid_free for the window's grid.
void nvim_win_grid_free(win_T *wp) { grid_free(&wp->w_grid_alloc); }

/// CLEAR_FIELD the window's grid (for reinit).
void nvim_win_grid_clear_field(win_T *wp) { CLEAR_FIELD(wp->w_grid_alloc); }

// win_free_grid deleted: Rust exports under the C name directly via #[export_name = "win_free_grid"].

// win_new_screensize, win_new_screen_rows, win_new_screen_cols,
// snapshot_windows_scroll_size, may_make_initial_scroll_size_snapshot,
// may_trigger_win_scrolled_resized: exported directly from Rust (Phase 15)

#define FRACTION_MULT   16384

// win_fix_scroll deleted: Rust exports under the C name directly via #[export_name = "win_fix_scroll"].
// win_fix_cursor: dead static wrapper (Phase 15)

// scroll_to_fraction: exported directly from Rust (Phase 15)

// rs_win_set_inner_size deleted: now exported as win_set_inner_size via #[export_name]
// win_set_inner_size deleted: Rust exports under the C name directly via #[export_name].

// command_height: thin wrapper defined in Phase 3 accessors section.

/// Add or remove window bar from window "wp".
///
/// @param make_room Whether to resize frames to make room for winbar.
/// @param valid_cursor Whether the cursor is valid and should be used while
///                     resizing.
///
/// @return Success status.
// set_winbar_win deleted: Rust exports under the C name directly via #[export_name = "set_winbar_win"].
// set_winbar deleted: Rust exports under the C name directly via #[export_name = "set_winbar"].

// A snapshot of the window sizes, to restore them after closing the help
// window.
// Only these fields are used:
// fr_layout
// fr_width
// fr_height
// fr_next
// fr_child
// fr_win (only valid for the old curwin, NULL otherwise)

// check_colorcolumn: exported directly from Rust (Phase 15)

// nvim_win_buffer_eq: migrated to Rust wrappers.rs (Phase 9)
int nvim_win_get_filler_rows(win_T *wp) { return wp ? wp->w_filler_rows : 0; }
int nvim_win_get_botfill(win_T *wp) { return wp ? (wp->w_botfill ? 1 : 0) : 0; }
int nvim_win_grid_has_target(win_T *wp) { return (wp && wp->w_grid.target) ? 1 : 0; }
int nvim_win_get_scbind_pos(win_T *wp) { return wp ? wp->w_scbind_pos : 0; }
// nvim_win_buf_is_empty: migrated to Rust wrappers.rs (Phase 9)
int nvim_win_has_winnr(win_T *wp, tabpage_T *tp) { return (wp && tp) ? (int)win_has_winnr(wp, tp) : 0; }

// Accessors for rs_win_set_inner_size (Phase 4)
int nvim_win_get_width_request(win_T *wp) { return wp ? wp->w_width_request : 0; }
int nvim_win_get_height_request(win_T *wp) { return wp ? wp->w_height_request : 0; }
int nvim_win_get_prev_fraction_row(win_T *wp) { return wp ? wp->w_prev_fraction_row : 0; }
int nvim_win_get_p_spk_char(void) { return (int)(unsigned char)*p_spk; }
// nvim_validate_cursor_win already defined in move.c
// nvim_changed_line_abv_curs_win already defined in change_ffi.c
// nvim_invalidate_botline already defined in move.c
// nvim_curs_columns_win: migrated to Rust wrappers.rs (Phase 8)
void nvim_terminal_check_size_win(win_T *wp) { if (wp && wp->w_buffer->terminal) { terminal_check_size(wp->w_buffer->terminal); } }
int nvim_win_border_height_wrapper(win_T *wp) { return wp ? win_border_height(wp) : 0; }
int nvim_win_border_width_wrapper(win_T *wp) { return wp ? win_border_width(wp) : 0; }
int nvim_win_get_w_handle(win_T *wp) { return wp ? wp->handle : 0; }
// nvim_win_get_border_adj already defined earlier in this file
// nvim_ui_has_multigrid: migrated to Rust wrappers.rs (Phase 8)
// nvim_ui_call_grid_destroy_handle: migrated to Rust wrappers.rs (Phase 8)
// nvim_clear_matches_win: migrated to Rust wrappers.rs (Phase 8)
// nvim_free_jumplist_win: migrated to Rust wrappers.rs (Phase 8)
win_T *nvim_get_au_pending_free_win(void) { return au_pending_free_win; }
void nvim_set_au_pending_free_win(win_T *wp) { au_pending_free_win = wp; }
// nvim_xfree_win_raw: migrated to Rust wrappers.rs (Phase 8)
void nvim_ui_call_win_viewport_margins_wrapper(win_T *wp) {
  if (wp && ui_has(kUIMultigrid)) {
    ui_call_win_viewport_margins(wp->w_grid_alloc.handle, wp->handle,
                                 wp->w_winrow_off, wp->w_border_adj[2],
                                 wp->w_wincol_off, wp->w_border_adj[1]);
  }
}


// nvim_curbuf_line_count() — already defined in move.c

// nvim_win_buf_is_curbuf: migrated to Rust wrappers.rs (Phase 9)

/// Check if w_save_cursor.w_cursor_corr equals w_cursor (via equalpos).

/// Check if w_save_cursor.w_topline_corr equals w_topline.

/// Get w_save_cursor.w_cursor_save.lnum.

/// Get w_save_cursor.w_topline_save.


/// Check if w_save_cursor.w_topline_save > buffer line count.
int nvim_win_save_topline_gt_buf_line_count(win_T *wp) { return (wp && wp->w_buffer) ? wp->w_save_cursor.w_topline_save > wp->w_buffer->b_ml.ml_line_count : 0; }

void nvim_ga_init_int(garray_T *gap) { ga_init(gap, (int)sizeof(int), 1); }
void nvim_ga_grow(garray_T *gap, int n) { ga_grow(gap, n); }
int nvim_ga_get_len(garray_T *gap) { return gap ? gap->ga_len : 0; }

// nvim_ga_set_len() — already defined in fold.c

/// Get an int item from a growarray by index.
int nvim_ga_get_int(garray_T *gap, int idx) { return (gap && gap->ga_data && idx >= 0 && idx < gap->ga_len) ? ((int *)gap->ga_data)[idx] : 0; }

void nvim_ga_set_int(garray_T *gap, int idx, int val) { if (gap && gap->ga_data && idx >= 0) { ((int *)gap->ga_data)[idx] = val; } }
// nvim_comp_col: migrated to Rust wrappers.rs (Phase 8)

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

int nvim_win_get_prev_height(win_T *wp) { return wp ? wp->w_prev_height : 0; }
// nvim_win_float_anchor_laststatus: migrated to Rust wrappers.rs (Phase 8)
int nvim_win_get_fraction(win_T *wp) { return wp ? wp->w_fraction : 0; }

// nvim_get_p_ch() — already defined in message.c
// nvim_get_sc_col() — already defined in message.c

// nvim_win_alloc_wrapper deleted: callers updated to call rs_win_alloc directly (Phase 12)
// nvim_new_frame_wrapper deleted: callers updated to call rs_new_frame directly (Phase 12)
frame_T *nvim_xcalloc_frame(void) { return xcalloc(1, sizeof(frame_T)); }
void nvim_ui_comp_remove_grid_win(win_T *wp) { if (wp) { ui_comp_remove_grid(&wp->w_grid_alloc); } }
void nvim_ui_call_win_hide_win(win_T *wp) { if (wp) { ui_call_win_hide(wp->w_grid_alloc.handle); } }
// nvim_win_free_grid_wrapper deleted: callers updated to call rs_win_free_grid directly (Phase 12)

/// Wrapper: merge_win_config(&wp->w_config, WIN_CONFIG_INIT) + CLEAR_FIELD(wp->w_border_adj).
void nvim_merge_win_config_init(win_T *wp)
{
  if (wp) {
    merge_win_config(&wp->w_config, WIN_CONFIG_INIT);
    CLEAR_FIELD(wp->w_border_adj);
  }
}

// nvim_redraw_later_wrapper: migrated to Rust wrappers.rs (Phase 8)
// nvim_status_redraw_all_wrapper: migrated to Rust wrappers.rs (Phase 8)
// nvim_msg_clr_eos_force: migrated to Rust wrappers.rs (Phase 8)
// nvim_is_aucmd_win: migrated to Rust wrappers.rs (Phase 9)
// nvim_win_get_config_external_int: migrated to Rust win_struct.rs (Phase 7)

// nvim_fixup_external_curwin deleted: logic migrated to Rust win_close.rs (Phase 8)

int nvim_win_get_tcl_flags(void) { return (int)tcl_flags; }
void nvim_buf_inc_nwindows(buf_T *buf) { buf->b_nwindows++; }
void nvim_iemsg_move_other_frame(void) { iemsg("INTERNAL: trying to move a window into another frame"); }
int nvim_text_or_buf_locked(void) { return text_or_buf_locked() ? 1 : 0; }
// nvim_win_enter: migrated to Rust wrappers.rs (Phase 8)
void nvim_internal_error_othertab(void) { internal_error("win_close_othertab()"); }
// nvim_win_free_mem_wrapper deleted: rs_win_close_structural now calls rs_win_free_mem directly (Phase 10)
void nvim_inc_split_disallowed(void) { split_disallowed++; }
void nvim_dec_split_disallowed(void) { split_disallowed--; }
// nvim_ui_call_win_close_win: caller uses nvim_win_get_grid_alloc_handle directly (Phase 8)
// nvim_check_cursor_win_wrapper: migrated to Rust wrappers.rs (Phase 8)

/// Get w_frame->fr_parent from a window (for win_close frame comparison).
frame_T *nvim_win_get_frame_parent(win_T *wp) { return (wp && wp->w_frame) ? wp->w_frame->fr_parent : NULL; }

buf_T *nvim_get_firstbuf_wrapper(void) { return firstbuf; }
// nvim_can_close_floating_windows deleted: Rust callers use #[link_name = "rs_can_close_floating_windows_tp"].
int nvim_do_cmdline_cmd_wrapper(const char *cmd) { return do_cmdline_cmd(cmd); }
void nvim_emsg_e_cmdwin(void) { emsg(_(e_cmdwin)); }
int nvim_bt_quickfix_curbuf(void) { return bt_quickfix(curbuf) ? 1 : 0; }
void nvim_msg_onlyone(void) { msg(_(m_onlyone), 0); }

// Phase 1 accessors: curbuf/winfixbuf, split_disallowed, cmdwin state
int nvim_get_split_disallowed(void) { return split_disallowed; }
int nvim_win_buf_locked_split(win_T *wp) { return wp->w_buffer->b_locked_split ? 1 : 0; }

// Phase 2 accessors: win_split and win_splitmove orchestration
// nvim_may_open_tabpage deleted: Rust orchestrate.rs now uses #[link_name = "rs_may_open_tabpage"].
int nvim_get_cmdmod_split(void) { return cmdmod.cmod_split; }
/// Wrapper for win_split_ins callable from Rust (handles win_enter_ext and option restore).
int nvim_win_get_floating_win(win_T *wp) { return (wp && wp->w_floating) ? 1 : 0; }
win_T *nvim_win_get_prev_win(win_T *wp) { return wp ? wp->w_prev : NULL; }

// nvim_get_valid_prevwin deleted: logic migrated to Rust dispatch.rs (Phase 8)

// nvim_do_window_equalize deleted: logic migrated to Rust dispatch.rs (Phase 8)

// nvim_do_window_tag deleted: logic migrated to Rust dispatch.rs (Phase 8)

// nvim_do_window_goto_file deleted: logic migrated to Rust dispatch.rs (Phase 10)
// nvim_do_window_find_in_path deleted: logic migrated to Rust dispatch.rs (Phase 10)

// Phase 10 accessors: goto_file and find_in_path handlers
// (nvim_grab_file_name already exists in normal_shim.c with int* lnum_out)

/// buflist_findname_exp wrapper.
buf_T *nvim_buflist_findname_exp(const char *ptr) { return buflist_findname_exp(ptr); }

/// setpcmark() wrapper.
// nvim_setpcmark_curwin: migrated to Rust wrappers.rs (Phase 8)

/// win_split(0,0) wrapper returning OK/FAIL.
// (nvim_win_split_wrapper already exists)

/// RESET_BINDING(curwin) wrapper.
void nvim_reset_binding_curwin(void) { RESET_BINDING(curwin); }

/// do_ecmd(0, ptr, NULL, NULL, ECMD_LASTL, ECMD_HIDE, NULL) wrapper.
int nvim_do_ecmd_lastl_hide(const char *ptr)
{
  return do_ecmd(0, ptr, NULL, NULL, ECMD_LASTL, ECMD_HIDE, NULL);
}

/// check_cursor_lnum(curwin) wrapper.
// nvim_check_cursor_lnum_curwin: migrated to Rust wrappers.rs (Phase 8)

// nvim_beginline_sol_fix: migrated to Rust wrappers.rs (Phase 8)

// nvim_set_curwin_cursor_lnum already exists in change_ffi.c

/// find_pattern_in_path with ACTION_SPLIT and fixed extra params.
/// @param whole  1 to search whole file (Prenum == 0), 0 otherwise
void nvim_find_pattern_in_path_split(const char *ptr, size_t len, int type, int prenum1, int whole)
{
  find_pattern_in_path(ptr, 0, len, true, whole != 0, type,
                       prenum1, ACTION_SPLIT, 1, MAXLNUM, false, false);
}

/// curwin->w_set_curswant = true wrapper.
void nvim_set_curswant_curwin(void) { curwin->w_set_curswant = true; }

/// rs_find_ident_under_cursor wrapper: sets *pp to pointer into buffer, returns len.
size_t nvim_find_ident_under_cursor(char **pp) { return rs_find_ident_under_cursor(pp, FIND_IDENT); }

// New one-liner wrappers for rs_do_window_g (Phase 3)
// (nvim_inc/dec_no_mapping, nvim_inc/dec_allow_keys, nvim_goto_tabpage,
//  nvim_langmap_adjust, nvim_goto_tabpage_lastused, nvim_set_g_do_tagpreview,
//  nvim_set_postponed_split already exist in normal_shim.c / tag_shim.c)
// nvim_do_nv_ident: migrated to Rust wrappers.rs (Phase 8)
void nvim_set_cmdmod_tab_to_curtab_idx(void) { cmdmod.cmod_tab = rs_tabpage_index(curtab) + 1; }

// nvim_do_window_g_external deleted: logic migrated to Rust dispatch.rs (Phase 10)

/// win_new_float external wrapper: converts curwin to external float.
/// Returns 1 on success, 0 on failure (also calls beep_flush on failure).
/// Returns -1 if curwin is already floating or kUIMultigrid not supported.
int nvim_win_new_float_external(void)
{
  if (curwin->w_floating || !ui_has(kUIMultigrid)) {
    return -1;
  }
  WinConfig config = WIN_CONFIG_INIT;
  config.width = curwin->w_width;
  config.height = curwin->w_height;
  config.external = true;
  Error err = ERROR_INIT;
  if (!win_new_float(curwin, false, config, &err)) {
    emsg(err.msg);
    api_clear_error(&err);
    return 0;
  }
  return 1;
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

// === Accessors for rs_ui_ext_win_position ===

// WinConfig field accessors
int nvim_win_get_pos_changed(win_T *wp) { return wp ? wp->w_pos_changed : 0; }
// nvim_win_get_config_row/col/anchor/fixed/mouse_flag/bufpos_*: migrated to Rust win_struct.rs (Phase 7)
int nvim_win_get_w_height_outer(win_T *wp) { return wp ? wp->w_height_outer : 0; }
int nvim_win_get_w_width_outer(win_T *wp) { return wp ? wp->w_width_outer : 0; }

// Find a window by its integer handle (wraps find_window_by_handle)
win_T *nvim_handle_get_window(int handle)
{
  Error dummy = ERROR_INIT;
  win_T *wp = find_window_by_handle(handle, &dummy);
  api_clear_error(&dummy);
  return wp;
}

// Call win_grid_alloc on a window
// nvim_win_call_win_grid_alloc: migrated to Rust wrappers.rs (Phase 8)

// UI call wrappers (these wrap auto-generated ui_call_* functions)
void nvim_win_ui_call_win_pos(int grid, int win, int row, int col, int width, int height)
{
  ui_call_win_pos(grid, win, row, col, width, height);
}

/// Wrapper for ui_call_win_float_pos.
/// Takes anchor as a C integer index into float_anchor_str[].
void nvim_win_ui_call_win_float_pos(int grid_handle, int win_handle, int anchor,
                                     int anchor_grid, double row, double col,
                                     int mouse, int zindex, int comp_index,
                                     int screen_row, int screen_col)
{
  String anchor_str = cstr_as_string(float_anchor_str[anchor]);
  ui_call_win_float_pos(grid_handle, win_handle, anchor_str, anchor_grid,
                        row, col, (bool)mouse, zindex, comp_index, screen_row, screen_col);
}

// nvim_win_ui_call_win_hide: migrated to Rust wrappers.rs (Phase 8)

void nvim_win_ui_call_win_external_pos(int grid_handle, int win_handle)
{
  ui_call_win_external_pos(grid_handle, win_handle);
}

void nvim_win_ui_check_cursor_grid(int grid_handle) { ui_check_cursor_grid(grid_handle); }


// =============================================================================
// C Accessors for Phase 3: leaving_window / entering_window / win_init_empty
// =============================================================================

/// Check if window's buffer is a prompt buffer.
int nvim_win_bt_prompt(win_T *wp) { return (wp && wp->w_buffer) ? (bt_prompt(wp->w_buffer) ? 1 : 0) : 0; }

/// Get b_prompt_insert from a buffer (restart_edit value for prompt buffer).
int nvim_buf_get_prompt_insert(buf_T *buf) { return buf ? buf->b_prompt_insert : 0; }

/// Set b_prompt_insert on a buffer.
void nvim_buf_set_prompt_insert(buf_T *buf, int val) { if (buf) { buf->b_prompt_insert = val; } }



/// Get the window's buffer pointer.
buf_T *nvim_win_get_buf_ptr(win_T *wp) { return wp ? wp->w_buffer : NULL; }

/// Set w_pcmark (lnum and col).


/// Sync w_s to point to the window's buffer's b_s.
void nvim_win_sync_s(win_T *wp) { if (wp && wp->w_buffer) { wp->w_s = &wp->w_buffer->b_s; } }

// =============================================================================
// C Accessors for Phase 4: win_comp_scroll / win_new_screensize / etc.
// =============================================================================

/// Set w_height field (inner height).

/// Set w_width field (inner width).

/// Set scroll option script context to SID_WINLAYOUT (for win_comp_scroll).
void nvim_win_set_script_ctx_scroll(win_T *wp)
{
  if (wp) {
    wp->w_p_script_ctx[kWinOptScroll].sc_sid = SID_WINLAYOUT;
    wp->w_p_script_ctx[kWinOptScroll].sc_lnum = 0;
  }
}

/// Call win_reconfig_floats() (for win_new_screen_cols).
// nvim_win_reconfig_floats: migrated to Rust wrappers.rs (Phase 8)




// Phase 3 accessors: win_fix_scroll, win_fix_cursor, may_make_initial_scroll_size_snapshot
int nvim_win_get_do_win_fix_cursor(win_T *wp) { return wp ? (wp->w_do_win_fix_cursor ? 1 : 0) : 0; }
int nvim_win_get_prev_winrow(win_T *wp) { return wp ? wp->w_prev_winrow : 0; }

// Phase 4 accessors: win_new_screen_rows, unuse_tabpage, use_tabpage, win_goto,
// restore_snapshot, do_autocmd_winclosed, can_close_in_cmdwin, set_winbar_win, set_winbar
// nvim_compute_cmdrow: migrated to Rust wrappers.rs (Phase 8)
int nvim_has_event_winclosed(void) { return has_event(EVENT_WINCLOSED) ? 1 : 0; }
// nvim_apply_autocmds_winclosed deleted: logic migrated to Rust events.rs (Phase 8)
/// Apply WinClosed autocmd using pre-formatted handle string (Phase 8).
/// handle_str must be a NUL-terminated string of the window's integer handle.
void nvim_apply_autocmds_winclosed_by_handle(const char *handle_str, win_T *win)
{
  apply_autocmds(EVENT_WINCLOSED, (char *)handle_str, (char *)handle_str, false, win->w_buffer);
}
win_T *nvim_get_cmdwin_win(void) { return cmdwin_win; }
win_T *nvim_get_cmdwin_old_curwin(void) { return cmdwin_old_curwin; }
// nvim_can_close_in_cmdwin_check deleted: logic migrated to Rust focus.rs (Phase 8)
// nvim_set_cmdwin_result already exists in normal_shim.c
/// Set api_error for cmdwin (Phase 8 accessor).
void nvim_api_set_error_e_cmdwin(Error *err) { api_set_error(err, kErrorTypeException, "%s", e_cmdwin); }
// nvim_set_cmdheight_option deleted: logic migrated to Rust resize/frame.rs (Phase 8)
/// Set set_option_value for kOptCmdheight with given value (Phase 8 accessor).
void nvim_set_option_cmdheight(int64_t val) { set_option_value(kOptCmdheight, NUMBER_OPTVAL(val), 0); }
// nvim_set_min_set_ch already exists below (line ~2867); no duplicate needed here
/// Returns 1 if local w_p_wbr is empty/NULL (for floating window check).
int nvim_win_get_p_wbr_empty(win_T *wp) { return (!wp || !wp->w_p_wbr || *wp->w_p_wbr == NUL) ? 1 : 0; }
/// Returns 1 if BOTH global p_wbr AND local w_p_wbr are empty (for non-floating window check).
int nvim_win_get_p_wbr_both_empty(win_T *wp) { return (!wp || ((*p_wbr == NUL) && (!wp->w_p_wbr || *wp->w_p_wbr == NUL))) ? 1 : 0; }
/// Free winbar click defs for a window.
void nvim_win_clear_winbar_click_defs(win_T *wp)
{
  if (!wp) { return; }
  stl_clear_click_defs(wp->w_winbar_click_defs, wp->w_winbar_click_defs_size);
  xfree(wp->w_winbar_click_defs);
  wp->w_winbar_click_defs_size = 0;
  wp->w_winbar_click_defs = NULL;
}

// =============================================================================
// Phase 1 accessors: win_enter_ext migration
// =============================================================================

/// Generic autocmd dispatcher: apply_autocmds(event, NULL, NULL, false, curbuf).
/// Rust callers pass EVENT_* integer constants defined in auevents_enum.generated.h.
_Static_assert(EVENT_BUFENTER == 3, "EVENT_BUFENTER value mismatch");
_Static_assert(EVENT_BUFLEAVE == 7, "EVENT_BUFLEAVE value mismatch");
_Static_assert(EVENT_TABENTER == 110, "EVENT_TABENTER value mismatch");
_Static_assert(EVENT_TABLEAVE == 111, "EVENT_TABLEAVE value mismatch");
_Static_assert(EVENT_WINENTER == 136, "EVENT_WINENTER value mismatch");
_Static_assert(EVENT_WINLEAVE == 137, "EVENT_WINLEAVE value mismatch");
_Static_assert(EVENT_WINNEW == 138, "EVENT_WINNEW value mismatch");
void nvim_apply_autocmds_event(int event)
{
  apply_autocmds((event_T)event, NULL, NULL, false, curbuf);
}


/// Update topline for curwin (used in win_enter_ext).
// nvim_update_topline_curwin_enter: migrated to Rust wrappers.rs (Phase 8)

/// Copy buffer options on enter: buf_copy_options(buf, BCO_ENTER | BCO_NOHELP).
void nvim_buf_copy_options_enter(buf_T *buf) { buf_copy_options(buf, BCO_ENTER | BCO_NOHELP); }

/// Call changed_line_abv_curs().
// nvim_changed_line_abv_curs_wrap: migrated to Rust wrappers.rs (Phase 8)

/// Call do_autochdir().
// nvim_do_autochdir_wrap: migrated to Rust wrappers.rs (Phase 8)

/// Get restart_edit global (non-zero if in restart-edit mode).
int nvim_get_restart_edit_bool(void) { return restart_edit ? 1 : 0; }

// =============================================================================
// Phase 2 accessors: win_fix_current_dir, buf_jump_open_win/tab, swbuf_goto
// =============================================================================

/// Get curwin->w_localdir (or NULL if not set).
const char *nvim_curwin_get_localdir(void) { return curwin->w_localdir; }
/// Get curtab->tp_localdir (or NULL if not set).
const char *nvim_curtab_get_localdir(void) { return curtab->tp_localdir; }
/// Get globaldir global (or NULL).
const char *nvim_get_globaldir(void) { return globaldir; }
/// Set globaldir to a copy of cwd string.
void nvim_set_globaldir_from_str(const char *s) { globaldir = xstrdup(s); }
/// XFREE_CLEAR(globaldir): free and set to NULL.
void nvim_clear_globaldir(void) { XFREE_CLEAR(globaldir); }
/// Get the current working directory (os_dirname). Returns non-zero on success.
int nvim_os_dirname_maxpathl(char *buf) { return (int)os_dirname(buf, MAXPATHL); }
/// Attempt to chdir to dir. Returns 0 on success.
int nvim_os_chdir(const char *dir) { return os_chdir(dir); }
/// pathcmp(a, b, -1): returns 0 if equal.
int nvim_pathcmp_unlen(const char *a, const char *b) { return pathcmp(a, b, -1); }
/// p_acd option.
int nvim_get_p_acd(void) { return p_acd ? 1 : 0; }
/// Set last_chdir_reason to NULL.
void nvim_set_last_chdir_reason_null(void) { last_chdir_reason = NULL; }
/// shorten_fnames(true).
void nvim_shorten_fnames_force(void) { shorten_fnames(true); }
/// do_autocmd_dirchanged for window-scoped dir change (kCdScopeWindow or kCdScopeTabpage).
/// localdir==1 means window scope, localdir==0 means tabpage scope.
/// pre==1 means pre-change, pre==0 means post-change.
void nvim_do_autocmd_dirchanged_win(const char *new_dir, int localdir, int pre) {
  do_autocmd_dirchanged(new_dir,
                        localdir ? kCdScopeWindow : kCdScopeTabpage,
                        kCdCauseWindow, pre != 0);
}
/// do_autocmd_dirchanged for global scope (kCdScopeGlobal).
void nvim_do_autocmd_dirchanged_global(const char *new_dir, int pre) {
  do_autocmd_dirchanged(new_dir, kCdScopeGlobal, kCdCauseWindow, pre != 0);
}
/// goto_tabpage_win wrapper.
/// swb_flags & kOptSwbFlagUseopen.
int nvim_swb_has_useopen(void) { return (swb_flags & kOptSwbFlagUseopen) ? 1 : 0; }
/// swb_flags & kOptSwbFlagUsetab.
int nvim_swb_has_usetab(void) { return (swb_flags & kOptSwbFlagUsetab) ? 1 : 0; }
// win_fix_current_dir, buf_jump_open_win, buf_jump_open_tab,
// swbuf_goto_win_with_buf: exported directly from Rust (Phase 15)

// =============================================================================
// Phase 3 accessors: command_height migration
// =============================================================================

/// Set p_ch directly (for restoring after e_noroom).
void nvim_set_p_ch(int64_t val) { p_ch = val; }

/// Get command_frame_height static.
int nvim_get_command_frame_height(void) { return command_frame_height ? 1 : 0; }
/// Set command_frame_height static.
/// Get curtab->tp_ch_used as int.
int nvim_get_curtab_ch_used(void) { return curtab ? (int)curtab->tp_ch_used : 0; }
/// Set curtab->tp_ch_used from Rust.
void nvim_set_curtab_ch_used(int64_t val) { if (curtab) { curtab->tp_ch_used = val; } }

/// Set min_set_ch = val (the 'cmdheight' minimum).
void nvim_set_min_set_ch(int64_t val) { min_set_ch = val; }

/// Set cmdline_row = Rows - p_ch.
void nvim_update_cmdline_row(void) { cmdline_row = Rows - (int)p_ch; }


/// Wrapper: grid_clear for cmdheight area. Selects msg_grid_adj or default_gridview
/// depending on ui_has(kUIMessages). Also sets msg_row = cmdline_row.
/// Only called when msg_scrolled==0 and full_screen.
void nvim_grid_clear_cmd_area(void)
{
  if (msg_scrolled != 0 || !full_screen) {
    return;
  }
  GridView *grid = &default_gridview;
  if (!ui_has(kUIMessages)) {
    msg_grid_validate();
    grid = &msg_grid_adj;
  }
  grid_clear(grid, cmdline_row, Rows, 0, Columns, 0);
  msg_row = cmdline_row;
}

// command_height: exported directly from Rust (Phase 15)

// =============================================================================
// Phase 4: CTRL-W dispatch wrapper accessors
// =============================================================================

/// Return 1 if curbuf is locked (prevents split/new).
int nvim_curbuf_locked(void) { return curbuf_locked() ? 1 : 0; }

/// Error: E441 no preview window.


/// Error: E23 no alternate file.


/// Error: E92 buffer N not found.
void nvim_semsg_e92_buf_not_found(int64_t nr) { semsg(_("E92: Buffer %" PRId64 " not found"), nr); }

/// apply_autocmds for EVENT_TABNEWENTERED.
void nvim_apply_autocmds_tabnewentered(void)
{
  apply_autocmds(EVENT_TABNEWENTERED, NULL, NULL, false, curbuf);
}

/// get curwin->w_alt_fnum.


// Phase 2 accessors: tabpage helpers and check_split_disallowed_err migration

/// Get p_tpm (tabpagemax option).
int64_t nvim_get_p_tpm(void) { return p_tpm; }

/// api_set_error for E242 (split while closing).
void nvim_api_set_err_e242(Error *err)
{
  api_set_error(err, kErrorTypeException, "E242: Can't split a window while closing another");
}

/// api_set_error for e_cannot_split_window_when_closing_buffer.
void nvim_api_set_err_cannot_split_closing(Error *err)
{
  api_set_error(err, kErrorTypeException, "%s", e_cannot_split_window_when_closing_buffer);
}

// Phase 6 accessors: close_others

/// Check if a buffer can be abandoned (wraps can_abandon(wp->w_buffer, forceit)).
int nvim_win_can_abandon(win_T *wp, int forceit)
{
  return (wp && wp->w_buffer) ? can_abandon(wp->w_buffer, forceit) : 1;
}

/// Prompt user to save changes (wraps dialog_changed(wp->w_buffer, false)).
void nvim_win_dialog_changed(win_T *wp)
{
  if (wp && wp->w_buffer) {
    dialog_changed(wp->w_buffer, false);
  }
}

/// Check if the window's buffer has unsaved changes (wraps bufIsChanged).
int nvim_win_bufIsChanged(win_T *wp)
{
  return (wp && wp->w_buffer) ? bufIsChanged(wp->w_buffer) : 0;
}

/// Check if the window's buffer should be hidden (wraps buf_hide).
int nvim_win_buf_hide(win_T *wp)
{
  return (wp && wp->w_buffer) ? buf_hide(wp->w_buffer) : 0;
}

/// Check if a buffer pointer is valid (wraps buf_valid).
int nvim_buf_valid_ptr(buf_T *buf)
{
  return buf_valid(buf) ? 1 : 0;
}

/// Get the p_confirm option.
int nvim_get_p_confirm(void) { return p_confirm ? 1 : 0; }

/// Check if CMOD_CONFIRM flag is set in cmdmod.
int nvim_get_cmdmod_confirm(void) { return (cmdmod.cmod_flags & CMOD_CONFIRM) ? 1 : 0; }

/// Get the p_write option.
int nvim_get_p_write(void) { return p_write ? 1 : 0; }

// nvim_get_autocmd_busy() is defined in change_ffi.c (returns bool)

/// Set curwin and curbuf from wp->w_buffer.
void nvim_set_curwin_from_wp(win_T *wp)
{
  if (wp) {
    curwin = wp;
    curbuf = wp->w_buffer;
  }
}

/// Get the w_buffer field raw pointer.
buf_T *nvim_win_get_buffer_raw(win_T *wp) { return wp ? wp->w_buffer : NULL; }

// close_others: exported directly from Rust (Phase 15)

// Phase 6 accessors: close_windows

/// Get RedrawingDisabled.
int nvim_get_RedrawingDisabled(void) { return RedrawingDisabled; }
/// Set RedrawingDisabled.
void nvim_set_RedrawingDisabled(int val) { RedrawingDisabled = val; }
/// Increment RedrawingDisabled.
void nvim_inc_RedrawingDisabled(void) { RedrawingDisabled++; }
/// Decrement RedrawingDisabled.
void nvim_dec_RedrawingDisabled(void) { RedrawingDisabled--; }

/// Check if wp->w_buffer->b_locked > 0.
int nvim_win_buf_b_locked(win_T *wp)
{
  return (wp && wp->w_buffer && wp->w_buffer->b_locked > 0) ? 1 : 0;
}


// close_windows deleted: Rust exports under the C name directly via #[export_name = "close_windows"].

// Phase 6 accessors: ui_ext_win_viewport



/// Wrap ui_call_win_viewport for Rust.
void nvim_ui_call_win_viewport_wrapper(int grid, int win, int topline, int botline,
                                       int curline, int curcol, int line_count, int64_t delta)
{
  ui_call_win_viewport(grid, win, topline, botline, curline, curcol, line_count, delta);
}

// ui_ext_win_viewport: exported directly from Rust (Phase 15)

// Phase 6 accessors: tabpage_check_windows + win_ui_flush

/// Wrap pum_ui_flush() for Rust.
// nvim_pum_ui_flush_wrapper: migrated to Rust wrappers.rs (Phase 8)
// nvim_msg_ui_flush_wrapper: migrated to Rust wrappers.rs (Phase 8)

// nvim_win_get_grid_pending_comp/set/get_grid_chars_valid: migrated to Rust win_struct.rs (Phase 7)

// tabpage_check_windows: dead static wrapper (Phase 15)

// win_ui_flush deleted: Rust exports under the C name directly via #[export_name = "win_ui_flush"].

// Phase 6 accessors: may_open_tabpage

/// Get postponed_split_tab global.
int nvim_get_postponed_split_tab(void) { return postponed_split_tab; }

/// Set postponed_split_tab global.
void nvim_set_postponed_split_tab(int val) { postponed_split_tab = val; }

/// Get cmdmod.cmod_tab.
int nvim_get_cmdmod_tab(void) { return (int)cmdmod.cmod_tab; }

/// Set cmdmod.cmod_tab.
void nvim_set_cmdmod_tab(int val) { cmdmod.cmod_tab = val; }

// =============================================================================
// Phase 7 accessors: leave_tabpage, enter_tabpage, goto_tabpage_tp
// =============================================================================


/// Call reset_dragwin().
// nvim_reset_dragwin: migrated to Rust wrappers.rs (Phase 8)

/// Set firstwin = NULL (without syncing curtab->tp_firstwin).
void nvim_set_firstwin_null(void) { firstwin = NULL; }

/// Set lastwin = NULL (without syncing curtab->tp_lastwin).
void nvim_set_lastwin_null(void) { lastwin = NULL; }

/// Get the `starting` global (nonzero while Vim is starting up).
int nvim_get_starting(void) { return starting; }

/// Call win_float_update_statusline().
// nvim_win_float_update_statusline: migrated to Rust wrappers.rs (Phase 8)

/// Set lastused_tabpage global (for enter_tabpage migration).
void nvim_set_lastused_tabpage_from_rust(tabpage_T *tp) { lastused_tabpage = tp; }

/// Call set_keep_msg(NULL, 0).
// nvim_set_keep_msg_null: migrated to Rust wrappers.rs (Phase 8)

/// Set skip_win_fix_scroll global.
void nvim_set_skip_win_fix_scroll(int val) { skip_win_fix_scroll = (val != 0); }

/// Wrap set_option_value for cmdheight with command_frame_height guard.
/// Sets command_frame_height=false, calls set_option_value(kOptCmdheight, new_ch, 0),
/// then restores command_frame_height=true.
void nvim_set_cmdheight_for_tabpage(int64_t new_ch)
{
  command_frame_height = false;
  set_option_value(kOptCmdheight, NUMBER_OPTVAL(new_ch), 0);
  command_frame_height = true;
}


// =============================================================================
// Phase 9 accessors: win_init migration
// =============================================================================

/// Get w_changelistidx.
int nvim_win_get_changelistidx(win_T *wp) { return wp ? wp->w_changelistidx : 0; }

/// Set w_changelistidx.

/// Copy all compound data from src to dst for win_init.
/// flags: WSP_NEWLOC (0x100) to skip location list copy.
/// Handles: buffer link, pcmarks, jumplist, loclist, localdir,
///          tagstack, alist, options, folding state.
void nvim_win_init_copy_compound(win_T *dst, win_T *src, int flags)
{
  if (!dst || !src) {
    return;
  }
  // buffer link
  if (src->w_buffer) {
    dst->w_buffer = src->w_buffer;
    dst->w_s = &src->w_buffer->b_s;
    src->w_buffer->b_nwindows++;
  }
  // pcmarks
  dst->w_pcmark = src->w_pcmark;
  dst->w_prev_pcmark = src->w_prev_pcmark;
  // jumplist
  copy_jumplist(src, dst);
  // loclist
  if (flags & WSP_NEWLOC) {
    dst->w_llist = NULL;
    dst->w_llist_ref = NULL;
  } else {
    copy_loclist_stack(src, dst);
  }
  // localdir
  dst->w_localdir = (src->w_localdir == NULL) ? NULL : xstrdup(src->w_localdir);
  dst->w_prevdir = (src->w_prevdir == NULL) ? NULL : xstrdup(src->w_prevdir);
  // tagstack
  for (int i = 0; i < src->w_tagstacklen; i++) {
    taggy_T *tag = &dst->w_tagstack[i];
    *tag = src->w_tagstack[i];
    if (tag->tagname != NULL) {
      tag->tagname = xstrdup(tag->tagname);
    }
    if (tag->user_data != NULL) {
      tag->user_data = xstrdup(tag->user_data);
    }
  }
  dst->w_tagstackidx = src->w_tagstackidx;
  dst->w_tagstacklen = src->w_tagstacklen;
  // alist
  if (src->w_alist) {
    dst->w_alist = src->w_alist;
    dst->w_alist->al_refcount++;
    dst->w_arg_idx = src->w_arg_idx;
  }
  // options
  win_copy_options(src, dst);
  // folding state
  rs_copyFoldingState(src, dst);
}

// =============================================================================
// Phase 9 accessors: may_trigger_win_scrolled_resized migration
// =============================================================================

// Event ignored wrappers
int nvim_event_ignored_winscrolled(win_T *wp)
{
  return wp ? (event_ignored(EVENT_WINSCROLLED, wp->w_p_eiw) ? 1 : 0) : 1;
}
int nvim_event_ignored_winresized(win_T *wp)
{
  return wp ? (event_ignored(EVENT_WINRESIZED, wp->w_p_eiw) ? 1 : 0) : 1;
}

// has_event wrappers for scroll/resize events
int nvim_has_event_winscrolled(void) { return has_event(EVENT_WINSCROLLED) ? 1 : 0; }
int nvim_has_event_winresized(void) { return has_event(EVENT_WINRESIZED) ? 1 : 0; }

/// Get buf handle (b_fnum / handle) for bufref validity check.
int nvim_win_get_buf_fnum(win_T *wp)
{
  return (wp && wp->w_buffer) ? (int)wp->w_buffer->handle : 0;
}

// =============================================================================
// Typval compound operations for scroll/resize event building
// =============================================================================

/// Allocate a new dict with refcount=1.
void *nvim_tv_dict_alloc_refcount1(void)
{
  dict_T *d = tv_dict_alloc();
  if (d) {
    d->dv_refcount = 1;
  }
  return d;
}

/// Add a number entry to a dict. Returns 1 on success, 0 on failure.
/// key_len is the length of the key (not including NUL).
int nvim_tv_dict_add_number(void *dict, const char *key, size_t key_len, int nr)
{
  if (!dict || !key) {
    return 0;
  }
  typval_T tv = {
    .v_lock = VAR_UNLOCKED,
    .v_type = VAR_NUMBER,
    .vval.v_number = (varnumber_T)nr,
  };
  return tv_dict_add_tv((dict_T *)dict, key, key_len, &tv) == OK ? 1 : 0;
}

/// Add a dict child under a key.
/// Decrements child->dv_refcount on success (ownership transferred).
/// Returns 1 on success, 0 on failure.
/// On failure, does NOT free child (caller must free it).
int nvim_tv_dict_add_dict_wrapper(void *dict, const char *key, size_t key_len, void *child)
{
  if (!dict || !key || !child) {
    return 0;
  }
  if (tv_dict_add_dict((dict_T *)dict, key, key_len, (dict_T *)child) == FAIL) {
    return 0;
  }
  ((dict_T *)child)->dv_refcount--;
  return 1;
}

/// Decrement refcount on dict (frees it when it reaches zero).
void nvim_tv_dict_unref_wrapper(void *dict)
{
  if (dict) {
    tv_dict_unref((dict_T *)dict);
  }
}

/// Allocate a new list (kListLenMayKnow hint with count).
void *nvim_tv_list_alloc_wrapper(int count)
{
  return tv_list_alloc((ptrdiff_t)count);
}

/// Append a number to a list.
void nvim_tv_list_append_number(void *list, int nr)
{
  if (!list) {
    return;
  }
  typval_T tv = {
    .v_lock = VAR_UNLOCKED,
    .v_type = VAR_NUMBER,
    .vval.v_number = (varnumber_T)nr,
  };
  tv_list_append_owned_tv((list_T *)list, tv);
}

// =============================================================================
// Compound autocmd/v:event fire operations
// =============================================================================

/// Fire WinResized autocmd.
///
/// Takes full ownership of `list`. Handles the entire v:event lifecycle:
/// get_v_event -> tv_dict_add_list -> tv_dict_set_keys_readonly
/// -> apply_autocmds -> restore_v_event.
///
/// Uses `first_size_win` and `first_size_win_buf_fnum` to find a valid buf:
/// if the buffer identified by `first_size_win_buf_fnum` is still valid,
/// uses it; otherwise falls back to curbuf.
void nvim_fire_winresized(void *list, const char *winid_str,
                          win_T *first_size_win, int first_size_win_buf_fnum)
{
  if (!list) {
    return;
  }
  save_v_event_T save_v_event;
  dict_T *v_event = get_v_event(&save_v_event);

  buf_T *buf = curbuf;
  if (first_size_win_buf_fnum != 0) {
    bufref_T bufref;
    // Find buffer by fnum and set bufref for validity check
    buf_T *b = buflist_findnr(first_size_win_buf_fnum);
    if (b != NULL) {
      set_bufref(&bufref, b);
      if (bufref_valid(&bufref)) {
        buf = bufref.br_buf;
      }
    }
  }

  if (tv_dict_add_list(v_event, S_LEN("windows"), (list_T *)list) == OK) {
    tv_dict_set_keys_readonly(v_event);
    apply_autocmds(EVENT_WINRESIZED, (char *)winid_str, (char *)winid_str, false, buf);
  }
  restore_v_event(v_event, &save_v_event);
}

/// Fire WinScrolled autocmd.
///
/// Takes full ownership of `dict`. Handles the entire v:event lifecycle:
/// get_v_event -> tv_dict_extend(v_event, dict, "move") -> tv_dict_set_keys_readonly
/// -> tv_dict_unref(dict) -> apply_autocmds -> restore_v_event.
///
/// Uses `first_scroll_win` and `first_scroll_win_buf_fnum` to find a valid buf.
void nvim_fire_winscrolled(void *dict, const char *winid_str,
                           win_T *first_scroll_win, int first_scroll_win_buf_fnum)
{
  if (!dict) {
    return;
  }
  save_v_event_T save_v_event;
  dict_T *v_event = get_v_event(&save_v_event);

  buf_T *buf = curbuf;
  if (first_scroll_win_buf_fnum != 0) {
    buf_T *b = buflist_findnr(first_scroll_win_buf_fnum);
    if (b != NULL) {
      bufref_T bufref;
      set_bufref(&bufref, b);
      if (bufref_valid(&bufref)) {
        buf = bufref.br_buf;
      }
    }
  }

  // Move entries from scroll_dict to v_event.
  tv_dict_extend(v_event, (dict_T *)dict, "move");
  tv_dict_set_keys_readonly(v_event);
  tv_dict_unref((dict_T *)dict);

  apply_autocmds(EVENT_WINSCROLLED, (char *)winid_str, (char *)winid_str, false, buf);

  restore_v_event(v_event, &save_v_event);
}

// =============================================================================
// Phase 10 (Pass 2): win_close_buffer helpers
// =============================================================================

/// Set wp->w_locked.

/// Set buf->b_p_bl.
void nvim_buf_set_p_bl(buf_T *buf, int val) { if (buf) { buf->b_p_bl = (val != 0); } }

/// Compound wrapper for close_buffer with bufref guard.
/// Calls close_buffer(win, win->w_buffer, action, abort_if_last, true).
/// Returns 1 if curbuf became invalid (was wiped), 0 otherwise.
int nvim_close_buffer_for_win(win_T *win, int action, int abort_if_last)
{
  if (!win || !win->w_buffer) {
    return 0;
  }
  bufref_T bufref;
  set_bufref(&bufref, curbuf);
  win->w_locked = true;
  close_buffer(win, win->w_buffer, action, abort_if_last != 0, true);
  if (rs_win_valid_any_tab(win)) {
    win->w_locked = false;
  }
  if (!bufref_valid(&bufref)) {
    return 1;
  }
  return 0;
}

// =============================================================================
// Phase 10 (Pass 3): close_last_window_tabpage + win_free_mem helpers
// =============================================================================

/// Safe terminal check: 1 if win->w_buffer != NULL && w_buffer->terminal != NULL.
int nvim_win_buf_has_terminal_safe(win_T *win)
{
  return (win && win->w_buffer && win->w_buffer->terminal) ? 1 : 0;
}

/// win_float_find_altwin wrapper.
win_T *nvim_win_float_find_altwin(win_T *win, tabpage_T *tp)
{
  return win_float_find_altwin(win, tp);
}

// nvim_xfree_frame: migrated to Rust wrappers.rs (Phase 8)

/// win_free(win, tp) wrapper.
// nvim_win_free_wrapper deleted: callers updated to call rs_win_free directly (Phase 12)

/// Check if win == cmdline_win.
int nvim_win_is_cmdline_win(win_T *win) { return (win == cmdline_win) ? 1 : 0; }

/// Set cmdline_win = NULL.
void nvim_set_cmdline_win_null(void) { cmdline_win = NULL; }

/// Wrapper for apply_autocmds(EVENT_BUFENTER) with old_curbuf comparison.
/// Fires if curbuf != old_curbuf_saved.
void nvim_apply_autocmds_bufenter_if_changed(buf_T *old_curbuf)
{
  if (old_curbuf != curbuf) {
    apply_autocmds(EVENT_BUFENTER, NULL, NULL, false, curbuf);
  }
}

// =============================================================================
// Phase 11: win_close + win_close_othertab consolidation accessors
// =============================================================================

/// Generic error-message dispatcher.
///
/// ID constants (must match EMSG_* in Rust):
///   0 = E444  "Cannot close last window"
///   1 = E814  "Cannot close window, only autocmd window would remain"
///   2 = E443  "Cannot rotate when another window is split"
///   3 = E442  "Can't split topleft and botright at the same time"
///   4 = E242  "Can't split a window while closing another"
///   5 = E445  "Other window contains changes"
///   6 = E441  "There is no preview window"
///   7 = noalt (e_noalt)
///   8 = e_floatonly
///   9 = e_floatexchange
///  10 = e_autocmd_close
///  11 = e_winfixbuf_cannot_go_to_buffer
///  12 = e_cannot_split_window_when_closing_buffer
///  13 = e_noroom
void nvim_emsg_id(int id)
{
  switch (id) {
  case 0: emsg(_("E444: Cannot close last window")); break;
  case 1: emsg(_("E814: Cannot close window, only autocmd window would remain")); break;
  case 2: emsg(_("E443: Cannot rotate when another window is split")); break;
  case 3: emsg(_("E442: Can't split topleft and botright at the same time")); break;
  case 4: emsg(_("E242: Can't split a window while closing another")); break;
  case 5: emsg(_("E445: Other window contains changes")); break;
  case 6: emsg(_("E441: There is no preview window")); break;
  case 7: emsg(_(e_noalt)); break;
  case 8: emsg(e_floatonly); break;
  case 9: emsg(e_floatexchange); break;
  case 10: emsg(_(e_autocmd_close)); break;
  case 11: emsg(_(e_winfixbuf_cannot_go_to_buffer)); break;
  case 12: emsg(_(e_cannot_split_window_when_closing_buffer)); break;
  case 13: emsg(_(e_noroom)); break;
  default: break;
  }
}

/// Returns 1 if wp->w_buffer is a help buffer (bt_help), 0 otherwise.
int nvim_bt_help_win(win_T *wp)
{
  return (wp && wp->w_buffer && bt_help(wp->w_buffer)) ? 1 : 0;
}

/// Calls win_close(wp, free_buf != 0, true) for recursive floating close from Rust.
/// Returns FAIL(1) or OK(0).
int nvim_win_close_force(win_T *wp, int free_buf)
{
  return win_close(wp, free_buf != 0, true);
}

/// Calls getout(0) -- never returns.
void nvim_getout_zero(void)
{
  getout(0);
}

/// Counts windows in curtab with w_p_diff set.
int nvim_count_diff_windows_in_curtab(void)
{
  int count = 0;
  FOR_ALL_WINDOWS_IN_TAB(dwin, curtab) {
    if (dwin->w_p_diff) {
      count++;
    }
  }
  return count;
}

/// Calls do_cmdline_cmd("diffoff!").
void nvim_do_cmdline_cmd_diffoff(void)
{
  do_cmdline_cmd("diffoff!");
}

/// Calls close_buffer(win, win->w_buffer, free_buf ? DOBUF_UNLOAD : 0, false, true).
/// Returns 1 if did_decrement (close_buffer returned true), 0 otherwise.
int nvim_close_buffer_othertab(win_T *win, int free_buf)
{
  if (!win || !win->w_buffer) {
    return 0;
  }
  return close_buffer(win, win->w_buffer, free_buf ? DOBUF_UNLOAD : 0, false, true) ? 1 : 0;
}

/// Fires EVENT_TABCLOSED with idx_str as the tab index string.
void nvim_apply_autocmds_tabclosed(const char *idx_str, buf_T *buf)
{
  apply_autocmds(EVENT_TABCLOSED, (char *)idx_str, (char *)idx_str, false, buf);
}

/// Returns 1 if has_event(EVENT_TABCLOSED), 0 otherwise.
int nvim_has_event_tabclosed(void)
{
  return has_event(EVENT_TABCLOSED) ? 1 : 0;
}

/// Sets curwin->w_buffer = curbuf (for getout edge case).
void nvim_curwin_set_buffer_to_curbuf(void)
{
  if (curwin) {
    curwin->w_buffer = curbuf;
  }
}

/// Returns 1 if ONE_WINDOW && curwin->w_locked && curbuf->b_locked_split
/// && first_tabpage->tp_next != NULL.
int nvim_one_window_and_locked_split(void)
{
  return (ONE_WINDOW && curwin && curwin->w_locked
          && curbuf && curbuf->b_locked_split
          && first_tabpage && first_tabpage->tp_next != NULL) ? 1 : 0;
}

// =============================================================================
// Window/wline accessors (relocated from fold_shim.c)
// =============================================================================

/// Get a wline_T pointer at index in a window's w_lines array.
/// Returns NULL if index is out of bounds.
wline_T *nvim_win_get_wl_entry(win_T *wp, int idx)
{
  if (idx < 0 || idx >= wp->w_lines_valid) {
    return NULL;
  }
  return &wp->w_lines[idx];
}

/// Get the wl_lnum field from a wline_T.
linenr_T nvim_wline_get_lnum(wline_T *wl)
{
  return wl->wl_lnum;
}

/// Get the wl_foldend field from a wline_T.
linenr_T nvim_wline_get_foldend(wline_T *wl)
{
  return wl->wl_foldend;
}

/// Get the wl_valid field from a wline_T.
bool nvim_wline_get_valid(wline_T *wl)
{
  return wl->wl_valid;
}

/// Get the wl_folded field from a wline_T.
bool nvim_wline_get_folded(wline_T *wl)
{
  return wl->wl_folded;
}

/// Get the wl_size field from a wline_T.
uint16_t nvim_wline_get_size(wline_T *wl)
{
  return wl->wl_size;
}

/// Get the wl_lastlnum field from a wline_T.
linenr_T nvim_wline_get_lastlnum(wline_T *wl)
{
  return wl->wl_lastlnum;
}


// =============================================================================
// Moved from drawscreen.c: window/buffer visual state accessors
// =============================================================================



/// Get the buffer's mod_set flag.
int nvim_buf_get_mod_set(buf_T *buf)
{
  return buf ? buf->b_mod_set : 0;
}

/// Set the buffer's mod_set flag.
void nvim_buf_set_mod_set(buf_T *buf, int val)
{
  if (buf) {
    buf->b_mod_set = (val != 0);
  }
}

/// Get the window's old_visual_mode.
int nvim_win_get_old_visual_mode(win_T *wp)
{
  return wp ? wp->w_old_visual_mode : 0;
}

/// Get the window's old_cursor_lnum.
linenr_T nvim_win_get_old_cursor_lnum(win_T *wp)
{
  return wp ? wp->w_old_cursor_lnum : 0;
}

/// Get the window's old_visual_lnum.
linenr_T nvim_win_get_old_visual_lnum(win_T *wp)
{
  return wp ? wp->w_old_visual_lnum : 0;
}

/// Get the window's old_visual_col.
colnr_T nvim_win_get_old_visual_col(win_T *wp)
{
  return wp ? wp->w_old_visual_col : 0;
}

/// Check if redrawing is currently being done (accessor for Rust).
int nvim_redrawing(void)
{
  return redrawing() ? 1 : 0;
}

/// Scroll lines in window (wrapper for win_scroll_lines for Rust FFI).
void nvim_win_scroll_lines(win_T *wp, int row, int line_count)
{
  win_scroll_lines(wp, row, line_count);
}

// Drawscreen Phase 5/6 accessors for window cursor/fold fields

extern foldinfo_T rs_fold_info(win_T *win, linenr_T lnum);

/// Get w_p_cole option for a window.
int nvim_win_get_w_p_cole(win_T *wp) { return wp ? wp->w_p_cole : 0; }

/// Get w_p_cul option for a window.
int nvim_win_get_w_p_cul(win_T *wp) { return wp ? wp->w_p_cul : 0; }

/// Set w_cursorline for a window.

/// Call fold_info and write results to output fields. Returns fi_level.
int nvim_fold_info(win_T *wp, linenr_T lnum, linenr_T *out_fi_lnum, linenr_T *out_fi_lines,
                   foldinfo_T *out_foldinfo)
{
  foldinfo_T fi = rs_fold_info(wp, lnum);
  if (out_foldinfo) {
    *out_foldinfo = fi;
  }
  if (out_fi_lnum) {
    *out_fi_lnum = fi.fi_lnum;
  }
  if (out_fi_lines) {
    *out_fi_lines = fi.fi_lines;
  }
  return fi.fi_level;
}

/// Get w_wrow for a window.
int nvim_win_get_w_wrow(win_T *wp) { return wp ? wp->w_wrow : 0; }

/// Get w_wcol for a window.
int nvim_win_get_w_wcol(win_T *wp) { return wp ? wp->w_wcol : 0; }

/// Get w_p_rl option for a window.
int nvim_win_get_w_p_rl(win_T *wp) { return wp ? wp->w_p_rl : 0; }

/// Compute rightleft column adjustment for cursor positioning.
/// Returns the adjusted column, accounting for double-wide chars.
int nvim_win_rl_cursor_col(win_T *wp)
{
  if (!wp) { return 0; }
  char *cursor = ml_get_buf(wp->w_buffer, wp->w_cursor.lnum) + wp->w_cursor.col;
  int view_width = wp->w_view_width;
  int wcol = wp->w_wcol;
  return view_width - wcol - ((utf_ptr2cells(cursor) == 2
                               && vim_isprintc(utf_ptr2char(cursor))) ? 2 : 1);
}

/// Combined grid_adjust + ui_grid_cursor_goto.
/// Avoids exposing ScreenGrid pointer to Rust.
void nvim_grid_adjust_cursor_goto(win_T *wp, int row, int col)
{
  ScreenGrid *grid = grid_adjust(&wp->w_grid, &row, &col);
  if (grid) {
    ui_grid_cursor_goto(grid->handle, row, col);
  }
}

/// Wrapper for validate_cursor(wp) for Rust FFI.
void nvim_validate_cursor_for_win(win_T *wp)
{
  validate_cursor(wp);
}

/// Get VIsual_active state (Rust FFI).
int nvim_VIsual_active(void) { return VIsual_active ? 1 : 0; }

/// Mark status lines for redraw for all windows (Rust incsearch).
void nvim_status_redraw_all(void)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (wp->w_status_height > 0 || (wp == curwin && rs_global_stl_height() > 0)) {
      wp->w_redr_status = true;
    }
    if (wp->w_winbar_height > 0) {
      wp->w_redr_status = true;
    }
  }
}

/// Trigger a full screen update (Rust incsearch).
// nvim_update_screen: migrated to Rust wrappers.rs (Phase 8)

/// Get w_view_width for a window (visible text column count).
int nvim_win_get_w_view_width(win_T *wp) { return wp ? wp->w_view_width : 0; }

/// Get w_scwidth for a window (sign column width in chars).
int nvim_win_get_w_scwidth(win_T *wp) { return wp ? wp->w_scwidth : 0; }

/// Get w_p_nu option for a window (show line numbers).
int nvim_win_get_w_p_nu(win_T *wp) { return wp ? wp->w_p_nu : 0; }

/// Get w_p_rnu option for a window (show relative line numbers).
int nvim_win_get_w_p_rnu(win_T *wp) { return wp ? wp->w_p_rnu : 0; }

// Moved from drawscreen.c — display Rust FFI wrappers

// nvim_win_check_ns_hl: migrated to Rust wrappers.rs (Phase 8)
// nvim_win_redr_winbar: migrated to Rust wrappers.rs (Phase 8)
// nvim_win_redr_status: migrated to Rust wrappers.rs (Phase 8)
// nvim_draw_tabline: migrated to Rust wrappers.rs (Phase 8)
// nvim_maketitle: migrated to Rust wrappers.rs (Phase 8)

// Moved from drawscreen.c — title/icon Rust FFI helpers

/// Call trans_characters() on a buffer (for maketitle icon).
void nvim_trans_characters(char *buf, size_t bufsize) { trans_characters(buf, (int)bufsize); }
/// Call ui_call_set_title() with a C string (NULL treated as empty string).
void nvim_ui_call_set_title(const char *s) { ui_call_set_title(cstr_as_string(s ? s : "")); }
/// Call ui_call_set_icon() with a C string (NULL treated as empty string).
void nvim_ui_call_set_icon(const char *s) { ui_call_set_icon(cstr_as_string(s ? s : "")); }
/// Get the end_off from utf_cp_bounds(str, ptr).
int nvim_utf_cp_bounds_end_off(const char *str, const char *ptr)
{
  return utf_cp_bounds((char *)str, (char *)ptr).end_off;
}

// Moved from drawscreen.c — Rust FFI accessors

/// Check if cmdline mouse_used is set (for cmdline_number_prompt).
int nvim_cmdline_mouse_used(void)
{
  return get_cmdline_info()->mouse_used != NULL ? 1 : 0;
}

/// Set v:echospace variable.
void nvim_set_vim_var_echospace(int val) { set_vim_var_nr(VV_ECHOSPACE, val); }

// Spell checking synblock accessors — used by Rust spell crate for spell_iswordp*
bool nvim_win_get_b_cjk(const win_T *wp) { return wp->w_s->b_cjk != 0; }
const bool *nvim_win_get_b_spell_ismw(const win_T *wp) { return wp->w_s->b_spell_ismw; }
const char *nvim_win_get_b_spell_ismw_mb(const win_T *wp) { return wp->w_s->b_spell_ismw_mb; }
const garray_T *nvim_win_get_b_langp(const win_T *wp) { return &wp->w_s->b_langp; }
void nvim_emsg_no_spell(void) { emsg(_(e_no_spell)); }
regprog_T *nvim_win_get_b_cap_prog(const win_T *wp) { return wp->w_s->b_cap_prog; }

// Runs vim_regexec with b_cap_prog against ptr. Updates b_cap_prog (may be GC'd).
// Returns the offset of endp[0] from ptr if matched, -1 if no match.
int nvim_win_spell_capcol_regexec(win_T *wp, char *ptr)
{
  regmatch_T regmatch = { .regprog = wp->w_s->b_cap_prog, .rm_ic = false };
  bool r = vim_regexec(&regmatch, ptr, 0);
  wp->w_s->b_cap_prog = regmatch.regprog;
  if (r) {
    return (int)(regmatch.endp[0] - ptr);
  }
  return -1;
}

// set_topline wrapper for winrestview (window viml)
void nvim_set_topline(win_T *wp, int lnum) { set_topline(wp, (linenr_T)lnum); }

// Typval accessors for window VimL functions (viml.rs)
typval_T *nvim_eval_tv_idx(typval_T *argvars, int i) { return &argvars[i]; }
void nvim_eval_tv_set_number(typval_T *tv, int64_t n) { tv->v_type = VAR_NUMBER; tv->vval.v_number = (varnumber_T)n; }
void nvim_eval_tv_set_string(typval_T *tv, char *s) { tv->vval.v_string = s; }
void nvim_eval_tv_set_type(typval_T *tv, int t) { tv->v_type = (VarType)t; }
dict_T *nvim_win_get_vars(win_T *wp) { return wp->w_vars; }

// ============================================================================
// draw_statuscol accessors for Rust (Phase 1 drawline migration)
// ============================================================================

/// w_nrwidth getter/setter
int nvim_win_get_nrwidth(win_T *wp) { return wp->w_nrwidth; }

/// w_statuscol_line_count getter
linenr_T nvim_win_get_statuscol_line_count(win_T *wp) { return wp->w_statuscol_line_count; }

/// w_redr_statuscol setter

/// statuscol_T field accessors (opaque pointer)
int nvim_stcp_get_width(statuscol_T *stcp) { return stcp->width; }
void nvim_stcp_set_width(statuscol_T *stcp, int val) { stcp->width = val; }
stl_hlrec_t *nvim_stcp_get_hlrec(statuscol_T *stcp) { return stcp->hlrec; }
colnr_T *nvim_stcp_get_fold_vcol(statuscol_T *stcp) { return stcp->fold_vcol; }

/// stl_hlrec_t field accessors (opaque pointer)
char *nvim_hlrec_get_start(stl_hlrec_t *sp) { return sp->start; }
int nvim_hlrec_get_item(stl_hlrec_t *sp) { return (int)sp->item; }
int nvim_hlrec_get_userhl(stl_hlrec_t *sp) { return sp->userhl; }
stl_hlrec_t *nvim_hlrec_next(stl_hlrec_t *sp) { return sp + 1; }

/// build_statuscol_str wrapper (statuscol_T is opaque from Rust side)
int nvim_build_statuscol_str(win_T *wp, linenr_T lnum, linenr_T relnum, char *buf,
                             statuscol_T *stcp)
{
  return build_statuscol_str(wp, lnum, relnum, buf, stcp);
}

/// get_cursor_rel_lnum wrapper
linenr_T nvim_get_cursor_rel_lnum(win_T *wp, linenr_T lnum)
{
  return get_cursor_rel_lnum(wp, lnum);
}

/// transstr_buf wrapper (Rust can't call variadic-adjacent ssize_t param easily)
size_t nvim_transstr_buf(const char *s, ptrdiff_t slen, char *buf, size_t buflen)
{
  return transstr_buf(s, slen, buf, buflen, true);
}

// ============================================================================
// Phase 3: FFI infrastructure for win_line migration
// Global variable accessors
// ============================================================================

// nvim_get_VIsual_lnum, _col, _coladd already defined in plines.c.
// nvim_get_VIsual_mode already defined in normal_shim.c.
// nvim_get_highlight_match already defined in change_ffi.c.
// nvim_get_search_match_lines, nvim_get_search_match_endcol already defined in search.c.
// nvim_get_dollar_vcol already defined in edit.c.
// nvim_get_cmdwin_type already defined in ex_getln.c.
// nvim_get_did_emsg, nvim_set_did_emsg already defined in message.c.

/// 'selection' option
const char *nvim_get_p_sel(void) { return p_sel; }

/// spell_redraw_lnum
linenr_T nvim_get_spell_redraw_lnum(void) { return spell_redraw_lnum; }
void nvim_set_spell_redraw_lnum(linenr_T val) { spell_redraw_lnum = val; }

/// highlight_attr[] indexed access (renamed to avoid clash with highlight.c nvim_get_highlight_attr)
int nvim_get_highlight_attr_idx(int idx) { return highlight_attr[idx]; }

/// dy_flags (display options)
int nvim_get_dy_flags(void) { return dy_flags; }
// nvim_get_p_cpo already defined at line 314; nvim_get_curwin at line 211.

/// curwin->w_cursor accessors
linenr_T nvim_curwin_cursor_lnum(void) { return curwin->w_cursor.lnum; }
colnr_T nvim_curwin_cursor_col(void) { return curwin->w_cursor.col; }
colnr_T nvim_curwin_cursor_coladd(void) { return curwin->w_cursor.coladd; }

// ============================================================================
// Phase 3: window field accessors for win_line
// ============================================================================

// nvim_win_get_p_lbr already defined in plines.c.

/// b_syn_error / b_syn_slow (via w_s)
int nvim_win_get_syn_error(win_T *wp) { return wp->w_s->b_syn_error ? 1 : 0; }
void nvim_win_set_syn_error(win_T *wp, int val) { wp->w_s->b_syn_error = (bool)val; }
int nvim_win_get_syn_slow(win_T *wp) { return wp->w_s->b_syn_slow ? 1 : 0; }

/// syntax_present wrapper
int nvim_win_syntax_present(win_T *wp) { return syntax_present(wp) ? 1 : 0; }

/// w_p_fdt (foldtext option, check if set)
int nvim_win_p_fdt_empty(win_T *wp) { return *wp->w_p_fdt == NUL ? 1 : 0; }

// nvim_win_lcs_*/fcs_fold: migrated to Rust win_struct.rs (Phase 7)

/// w_p_cc_cols (colorcolumn array, NULL if not set)
int *nvim_win_get_cc_cols(win_T *wp) { return wp->w_p_cc_cols; }

/// w_old_cursor_fcol / w_old_cursor_lcol (block visual mode columns)
colnr_T nvim_win_get_old_cursor_fcol(win_T *wp) { return wp->w_old_cursor_fcol; }
colnr_T nvim_win_get_old_cursor_lcol(win_T *wp) { return wp->w_old_cursor_lcol; }

// nvim_win_get_topline, nvim_win_get_topfill already defined above.
// nvim_win_get_leftcol already defined above.

/// w_virtcol
colnr_T nvim_win_get_virtcol_val(win_T *wp) { return wp->w_virtcol; }

/// w_cursor compound setter (temporarily move cursor, e.g. for spell checking)
/// Renamed to avoid clash with Neovim API nvim_win_set_cursor(Window, Array, Error*).

// nvim_win_get_botfill already defined above.

/// w_p_cuc (cursorcolumn)
int nvim_win_get_p_cuc_val(win_T *wp) { return wp->w_p_cuc ? 1 : 0; }

// nvim_win_set_cline_row, nvim_win_set_cline_height, nvim_win_set_cline_folded,
// nvim_win_get_cline_row, nvim_win_get_cline_height already defined above.

/// w_valid set bits

// nvim_win_get_wrow, nvim_win_set_wrow, nvim_win_get_wcol, nvim_win_set_wcol already defined above.
// nvim_win_get_scwidth already defined above.

/// w_p_wrap getter
int nvim_win_get_wrap_val(win_T *wp) { return wp->w_p_wrap ? 1 : 0; }

/// buffer terminal pointer (non-null if terminal buffer)
int nvim_win_buf_is_terminal(win_T *wp) { return wp->w_buffer->terminal != NULL ? 1 : 0; }

/// buffer line count
linenr_T nvim_win_buf_line_count_direct(win_T *wp) { return wp->w_buffer->b_ml.ml_line_count; }

/// VirtLines helpers for win_line (Phase 3 infrastructure)
/// VirtLines is kvec_t(struct virt_line { VirtText line; int flags; }) from decoration_defs.h
int nvim_virt_lines_size(void *vl) { return (int)((VirtLines *)vl)->size; }
int nvim_virt_lines_flags(void *vl, int idx) { return ((VirtLines *)vl)->items[idx].flags; }
void *nvim_virt_lines_line(void *vl, int idx) { return &((VirtLines *)vl)->items[idx].line; }
void nvim_virt_lines_destroy(void *vl) { kv_destroy(*(VirtLines *)vl); }

/// ScreenGrid field accessors (for the wrap line offset update in win_line)
int nvim_grid_get_cols(ScreenGrid *grid) { return grid->cols; }
size_t *nvim_grid_get_line_offset(ScreenGrid *grid) { return grid->line_offset; }
sattr_T *nvim_grid_get_attrs(ScreenGrid *grid) { return grid->attrs; }

/// wp->w_grid (GridView) pointer accessor for win_line
GridView *nvim_win_get_grid(win_T *wp) { return &wp->w_grid; }

// nvim_ml_get already defined in change_ffi.c.

/// ml_get_buf_len for a window's buffer
colnr_T nvim_win_ml_get_buf_len2(win_T *wp, linenr_T lnum) { return ml_get_buf_len(wp->w_buffer, lnum); }

/// win_bg_attr wrapper
int nvim_win_bg_attr(win_T *wp) { return win_bg_attr(wp); }

/// bt_quickfix check
int nvim_win_bt_quickfix(win_T *wp) { return bt_quickfix(wp->w_buffer) ? 1 : 0; }

/// qf_current_entry wrapper
linenr_T nvim_win_qf_current_entry(win_T *wp) { return qf_current_entry(wp); }

/// buf_meta_total wrapper (kMTMetaInline = 2 -- from marktree_defs.h)
int nvim_win_buf_meta_total_inline(win_T *wp) { return buf_meta_total(wp->w_buffer, kMTMetaInline) > 0 ? 1 : 0; }

/// schar functions needed for win_line
int nvim_schar_cells(schar_T sc) { return schar_cells(sc); }
int nvim_schar_len(schar_T sc) { return schar_len(sc); }
/// schar_get_adv wrapper: buf_out is advanced past the character, returns bytes consumed.
size_t nvim_schar_get_adv(char **buf_out, schar_T sc) { return schar_get_adv(buf_out, sc); }
int nvim_schar_get_first_codepoint(schar_T sc) { return (int)schar_get_first_codepoint(sc); }

/// CharsizeArg init (opaque; we pass it around as void* from Rust)
/// We need to call init_charsize_arg from Rust.
/// Returns CSType as int (0 = kCharsizeRegular, 1 = kCharsizeFast).
int nvim_init_charsize_arg_wrap(void *csarg, win_T *wp, linenr_T lnum, const char *line)
{
  return (int)init_charsize_arg((CharsizeArg *)csarg, wp, lnum, line);
}
/// win_charsize wrapper -- returns width and head packed as {width, head}
void nvim_win_charsize_wrap(bool cstype, int vcol, const char *ptr, int32_t chr,
                            void *csarg, int *out_width, int *out_head)
{
  CharSize cs = win_charsize(cstype, vcol, (char *)ptr, chr, (CharsizeArg *)csarg);
  *out_width = cs.width;
  *out_head = cs.head;
}
/// Size of CharsizeArg (for stack allocation in Rust)
int nvim_charsize_arg_size(void) { return (int)sizeof(CharsizeArg); }

// ---------------------------------------------------------------------------
// Phase 4: show_cursor_info_later accessors
// ---------------------------------------------------------------------------

/// Return 1 if the cursor line in curwin starts with NUL (empty line, not insert mode).
int nvim_curwin_cursor_line_is_nul(void)
{
  return *ml_get_buf(curwin->w_buffer, curwin->w_cursor.lnum) == NUL ? 1 : 0;
}

/// Get VIsual position (lnum, col, coladd) as three int32 values.
void nvim_get_VIsual_pos_fields(int32_t *lnum, int32_t *col, int32_t *coladd)
{
  *lnum = (int32_t)VIsual.lnum;
  *col = (int32_t)VIsual.col;
  *coladd = (int32_t)VIsual.coladd;
}

/// Batch read curwin's w_stl_* state into output buffer.
/// Layout: [cursor_lnum, cursor_col, cursor_coladd, virtcol, topline,
///          line_count, topfill, empty, recording, state, visual_mode,
///          vis_pos_lnum, vis_pos_col, vis_pos_coladd]
void nvim_curwin_get_stl_state(int32_t *out)
{
  out[0]  = (int32_t)curwin->w_stl_cursor.lnum;
  out[1]  = (int32_t)curwin->w_stl_cursor.col;
  out[2]  = (int32_t)curwin->w_stl_cursor.coladd;
  out[3]  = (int32_t)curwin->w_stl_virtcol;
  out[4]  = (int32_t)curwin->w_stl_topline;
  out[5]  = (int32_t)curwin->w_stl_line_count;
  out[6]  = (int32_t)curwin->w_stl_topfill;
  out[7]  = (int32_t)curwin->w_stl_empty;
  out[8]  = (int32_t)curwin->w_stl_recording;
  out[9]  = (int32_t)curwin->w_stl_state;
  out[10] = (int32_t)curwin->w_stl_visual_mode;
  out[11] = (int32_t)curwin->w_stl_visual_pos.lnum;
  out[12] = (int32_t)curwin->w_stl_visual_pos.col;
  out[13] = (int32_t)curwin->w_stl_visual_pos.coladd;
}

/// Batch write curwin's w_stl_* state fields from current window state.
/// Also copies w_stl_visual_mode/visual_pos when visual is active.
void nvim_curwin_set_stl_state(int32_t state, int32_t empty_line,
                               int32_t visual_active, int32_t visual_mode,
                               int32_t vis_lnum, int32_t vis_col, int32_t vis_coladd)
{
  curwin->w_stl_cursor      = curwin->w_cursor;
  curwin->w_stl_virtcol     = curwin->w_virtcol;
  curwin->w_stl_empty       = (char)empty_line;
  curwin->w_stl_topline     = curwin->w_topline;
  curwin->w_stl_line_count  = curwin->w_buffer->b_ml.ml_line_count;
  curwin->w_stl_topfill     = curwin->w_topfill;
  curwin->w_stl_recording   = reg_recording;
  curwin->w_stl_state       = state;
  if (visual_active) {
    curwin->w_stl_visual_mode       = visual_mode;
    curwin->w_stl_visual_pos.lnum   = (linenr_T)vis_lnum;
    curwin->w_stl_visual_pos.col    = (colnr_T)vis_col;
    curwin->w_stl_visual_pos.coladd = (colnr_T)vis_coladd;
  }
}

/// Return pointer to empty_string_option[].
///
/// empty_string_option is a char array (not a pointer), so Rust cannot access
/// its address directly via extern static without generating a double-dereference.
/// This thin C wrapper returns the correct address.
char *nvim_get_empty_string_option(void) { return empty_string_option; }

// Compile-time constant checks for Rust FFI (constants used in buffer/info crate)
_Static_assert(MIN_COLUMNS == 12, "MIN_COLUMNS must be 12");
_Static_assert(STL_IN_ICON == 1, "STL_IN_ICON must be 1");
_Static_assert(STL_IN_TITLE == 2, "STL_IN_TITLE must be 2");
_Static_assert(kOptTitlestring == 327, "kOptTitlestring mismatch");
_Static_assert(kOptIconstring == 138, "kOptIconstring mismatch");
