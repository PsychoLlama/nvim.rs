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

#include "window_shim.c.generated.h"

// Rust FFI declarations (tag module)
extern void rs_tagstack_clear_entry(void *tg);
extern void rs_reset_VIsual_and_resel(void);
extern bool rs_check_text_or_curbuf_locked(oparg_T *oap);

// Rust fold FFI declarations
extern void rs_copyFoldingState(win_T *wp_from, win_T *wp_to);
extern void rs_clearFolding(win_T *win);
extern void rs_foldInitWin(win_T *wp);

extern int rs_get_scrolloff_value(win_T *wp);
extern int rs_global_winbar_height(void);
extern int rs_tabline_height(void);
extern int rs_global_stl_height(void);
extern int rs_win_locked(win_T *wp);
extern int rs_win_valid(win_T *win);
extern int rs_tabpage_win_valid(tabpage_T *tp, win_T *win);
extern int rs_win_valid_any_tab(win_T *win);
extern int rs_valid_tabpage(tabpage_T *tpc);
extern int rs_one_window_in_tab(win_T *win, tabpage_T *tp);
extern int rs_last_window(win_T *win);
extern int rs_tabpage_index(tabpage_T *ftp);
extern int rs_frame_check_height(frame_T *topfrp, int height);
extern int rs_frame_check_width(frame_T *topfrp, int width);
extern tabpage_T *rs_win_find_tabpage(win_T *win);
extern tabpage_T *rs_find_tabpage(int n);
extern win_T *rs_lastwin_nofloating(void);
extern win_T *rs_frame2win(frame_T *frp);
extern frame_T *rs_win_altframe(win_T *win);

// Result structure from rs_winframe_find_altwin
typedef struct {
  frame_T *altfr;
  int dir;
} WinframeResult;
extern WinframeResult rs_winframe_find_altwin(win_T *wp, frame_T *altfr_initial);

// New Rust replacements for frame tree operations
extern void rs_frame_flatten(frame_T *frp);
extern win_T *rs_winframe_remove(win_T *win, int *dirp, tabpage_T *tp, frame_T **unflat_altfr);
extern void rs_winframe_restore(win_T *wp, int dir, frame_T *unflat_altfr);

// New Rust replacements for tabpage operations (Phase 2)
extern tabpage_T *rs_alt_tabpage(void);
extern void rs_tabpage_move(int nr);
extern void rs_goto_tabpage(int n);

// New Rust replacements for window transition helpers (Phase 3)
extern void rs_leaving_window(win_T *win);
extern void rs_entering_window(win_T *win);
extern void rs_win_init_empty(win_T *wp);

// New Rust replacements for screen size and scroll helpers (Phase 4)
extern void rs_win_comp_scroll(win_T *wp);
extern void rs_win_new_screensize(void);
extern void rs_win_new_screen_cols(void);
extern void rs_win_init_size(void);
extern void rs_snapshot_windows_scroll_size(void);

extern int rs_frame_minheight(frame_T *topfrp, win_T *next_curwin);
extern int rs_win_comp_pos(void);
extern void rs_frame_comp_pos(frame_T *topfrp, int *row, int *col);
extern void rs_win_setheight_win(int height, win_T *win);
extern void rs_frame_add_height(frame_T *frp, int n);
extern void rs_frame_add_statusline(frame_T *frp);
extern void rs_frame_set_vsep(const frame_T *frp, int add);
extern void rs_frame_add_hsep(const frame_T *frp);
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
extern void rs_diff_clear(tabpage_T *tp);
extern int rs_diffopt_closeoff(void);

// Pure calculations and thin wrappers
extern int64_t rs_win_default_scroll(win_T *wp);
extern void rs_scroll_to_fraction(win_T *wp, int prev_height);
extern void rs_win_setheight(int height);
extern void rs_win_setwidth(int width);

// Height/width setters
extern void rs_frame_new_width(frame_T *topfrp, int width, int leftfirst, int wfw);
extern void rs_frame_new_height(frame_T *topfrp, int height, int topfirst, int wfh, int set_ch);

// Colorcolumn
extern const char *rs_check_colorcolumn(const char *cc, win_T *wp);

// Win exchange / rotate / move_after
extern void rs_win_exchange(int prenum);
extern void rs_win_rotate(int upwards, int count);
extern void rs_win_move_after(win_T *win1, win_T *win2);

// Phase 1: rs_win_split_ins_full absorbs C post-processing
extern win_T *rs_win_split_ins_full(int size, int flags, win_T *new_wp, int dir,
                                    frame_T *to_flatten);

// Snapshot lifecycle
extern void rs_clear_snapshot(tabpage_T *tp, int idx);
extern void rs_make_snapshot(int idx);
extern int rs_check_snapshot_rec(frame_T *sn, frame_T *fr);
extern win_T *rs_restore_snapshot_rec(frame_T *sn, frame_T *fr);

// Phase 1: Utility and validation helpers
extern bool rs_check_can_set_curbuf_disabled(void);
extern bool rs_check_can_set_curbuf_forceit(int forceit);
extern win_T *rs_prevwin_curwin(void);
extern int rs_check_split_disallowed(win_T *wp);

// Phase 2: Tabpage helpers and check_split_disallowed_err
extern void rs_close_tabpage(tabpage_T *tab);
extern int rs_make_tabpages(int maxcount);
extern int rs_goto_tabpage_lastused(void);
extern void rs_goto_tabpage_win(tabpage_T *tp, win_T *wp);
extern int rs_check_split_disallowed_err(const win_T *wp, Error *err);

// Phase 3: winframe_find_altwin, can_close_floating_windows
extern win_T *rs_winframe_find_altwin_full(win_T *win, int *dirp, tabpage_T *tp,
                                           frame_T **altfr);
extern int rs_can_close_floating_windows_tp(tabpage_T *tp);
extern int rs_get_maximum_wincount(frame_T *fr, int height);
extern int rs_make_windows(int count, int vertical);

// Phase 2: win_split and win_splitmove orchestration
extern int rs_win_split(int size, int flags);
extern int rs_win_splitmove(win_T *wp, int size, int flags);

// Phase 3: win_fix_scroll, win_fix_cursor, may_make_initial_scroll_size_snapshot
extern void rs_win_fix_scroll(int resize);
extern void rs_win_fix_cursor(int normal);
extern void rs_may_make_initial_scroll_size_snapshot(void);
extern int rs_get_did_initial_scroll_size_snapshot(void);

// Phase 4: win_new_screen_rows, unuse_tabpage, use_tabpage, win_goto,
//          restore_snapshot, do_autocmd_winclosed, can_close_in_cmdwin,
//          set_winbar_win, set_winbar
extern void rs_win_new_screen_rows(void);
extern void rs_unuse_tabpage(tabpage_T *tp);
extern void rs_use_tabpage(tabpage_T *tp);
extern void rs_win_goto(win_T *wp);
extern void rs_restore_snapshot(int idx, int close_curwin);
extern void rs_do_autocmd_winclosed(win_T *win);
extern bool rs_can_close_in_cmdwin(win_T *win, Error *err);
extern int rs_set_winbar_win(win_T *wp, int make_room, int valid_cursor);
extern void rs_set_winbar(int make_room);

// Status line management
extern void rs_last_status(int morewin);
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

int nvim_win_get_locked(win_T *wp) { return wp->w_locked; }
int nvim_win_get_floating(win_T *wp) { return wp->w_floating; }
int nvim_win_get_pvw(win_T *wp) { return wp->w_p_pvw; }
int nvim_win_get_ns_hl(win_T *wp) { return wp->w_ns_hl; }
int nvim_win_get_hl_attr_normal(win_T *wp) { return wp->w_hl_attr_normal; }
int nvim_win_get_hl_attr_normalnc(win_T *wp) { return wp->w_hl_attr_normalnc; }
int nvim_win_get_ns_hl_active(win_T *wp) { return wp->w_ns_hl_active; }
void nvim_win_set_ns_hl_active(win_T *wp, int val) { wp->w_ns_hl_active = val; }
int *nvim_win_get_ns_hl_attr(win_T *wp) { return wp->w_ns_hl_attr; }
void nvim_win_set_ns_hl_attr(win_T *wp, int *val) { wp->w_ns_hl_attr = val; }
bool nvim_win_get_hl_needs_update(win_T *wp) { return wp->w_hl_needs_update; }
void nvim_win_set_hl_needs_update(win_T *wp, bool val) { wp->w_hl_needs_update = val; }
void nvim_win_set_hl_attr_normal(win_T *wp, int val) { wp->w_hl_attr_normal = val; }
void nvim_win_set_hl_attr_normalnc(win_T *wp, int val) { wp->w_hl_attr_normalnc = val; }
bool nvim_win_get_config_external(win_T *wp) { return wp->w_config.external; }
bool nvim_win_get_config_border(win_T *wp) { return wp->w_config.border; }
int nvim_win_get_config_border_hl_id(win_T *wp, int idx) { return wp->w_config.border_hl_ids[idx]; }
void nvim_win_set_config_border_attr(win_T *wp, int idx, int val) { wp->w_config.border_attr[idx] = val; }
void nvim_win_set_config_shadow(win_T *wp, bool val) { wp->w_config.shadow = val; }
bool nvim_win_get_config_shadow(win_T *wp) { return wp->w_config.shadow; }
int nvim_win_get_p_winbl(win_T *wp) { return (int)wp->w_p_winbl; }
void nvim_win_set_grid_blending(win_T *wp, bool val) { wp->w_grid_alloc.blending = val; }
win_T *nvim_win_get_next(win_T *wp) { return wp->w_next; }
win_T *nvim_win_get_prev(win_T *wp) { return wp->w_prev; }
void nvim_win_set_next(win_T *wp, win_T *next) { wp->w_next = next; }
void nvim_win_set_prev(win_T *wp, win_T *prev) { wp->w_prev = prev; }

/// Set the firstwin global variable (accessor for Rust).
/// Also syncs curtab->tp_firstwin if curtab is not NULL.
void nvim_set_firstwin(win_T *wp) { firstwin = wp; if (curtab != NULL) { curtab->tp_firstwin = wp; } }

/// Set the lastwin global variable (accessor for Rust).
/// Also syncs curtab->tp_lastwin if curtab is not NULL.
void nvim_set_lastwin(win_T *wp) { lastwin = wp; if (curtab != NULL) { curtab->tp_lastwin = wp; } }

void nvim_tabpage_set_firstwin(tabpage_T *tp, win_T *wp) { tp->tp_firstwin = wp; }
void nvim_tabpage_set_lastwin(tabpage_T *tp, win_T *wp) { tp->tp_lastwin = wp; }
win_T *nvim_tabpage_get_lastwin(tabpage_T *tp) { return tp->tp_lastwin; }
win_T *nvim_get_curwin(void) { return curwin; }
ScreenGrid *nvim_get_curwin_grid_alloc(void) { return curwin ? &curwin->w_grid_alloc : NULL; }
win_T *nvim_get_firstwin(void) { return firstwin; }
win_T *nvim_get_lastwin(void) { return lastwin; }
int nvim_get_curwin_handle(void) { return curwin->handle; }

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

void nvim_validate_cursor(void) { validate_cursor(curwin); }
buf_T *nvim_get_curbuf(void) { return curbuf; }
tabpage_T *nvim_get_curtab(void) { return curtab; }
win_T *nvim_tabpage_get_firstwin(tabpage_T *tp) { return tp->tp_firstwin; }
tabpage_T *nvim_tabpage_get_next(tabpage_T *tp) { return tp->tp_next; }
tabpage_T *nvim_get_first_tabpage(void) { return first_tabpage; }
tabpage_T *nvim_get_lastused_tabpage(void) { return lastused_tabpage; }
win_T *nvim_tabpage_get_curwin(tabpage_T *tp) { return tp->tp_curwin; }
int nvim_tabpage_get_handle(tabpage_T *tp) { return (int)tp->handle; }
frame_T *nvim_win_get_frame(win_T *wp) { return wp->w_frame; }
int nvim_win_get_wfh(win_T *wp) { return wp->w_p_wfh; }
int nvim_win_get_wfw(win_T *wp) { return wp->w_p_wfw; }
int nvim_win_get_handle(win_T *wp) { return wp->handle; }
char nvim_win_get_fdm_char(win_T *wp, int idx) { return wp->w_p_fdm[idx]; }
int nvim_win_get_p_fen(win_T *wp) { return wp->w_p_fen; }
int nvim_win_buf_has_terminal(win_T *wp) { return wp->w_buffer->terminal != NULL; }
int nvim_win_folds_empty(win_T *wp) { return GA_EMPTY(&wp->w_folds); }
int nvim_win_get_valid(win_T *wp) { return wp->w_valid; }
void nvim_win_set_valid(win_T *wp, int val) { wp->w_valid = val; }
void nvim_win_clear_valid_bits(win_T *wp, int bits) { wp->w_valid &= ~bits; }
void nvim_win_set_lines_valid(win_T *wp, int val) { wp->w_lines_valid = val; }
int nvim_win_get_view_width(win_T *wp) { return wp->w_view_width; }
int nvim_win_get_view_height(win_T *wp) { return wp->w_view_height; }

// Note: nvim_win_get_skipcol is defined later in window.c (returns colnr_T)

buf_T *nvim_win_get_w_buffer(win_T *wp) { return wp->w_buffer; }
const char *nvim_win_get_w_p_fdc(win_T *wp) { return wp->w_p_fdc; }
int nvim_win_is_curwin(win_T *wp) { return wp == curwin; }
int nvim_win_get_p_rnu(win_T *wp) { return wp->w_p_rnu; }
int nvim_win_get_p_nu(win_T *wp) { return wp->w_p_nu; }
OptInt nvim_win_get_p_nuw(win_T *wp) { return wp->w_p_nuw; }
char *nvim_win_get_p_stc(win_T *wp) { return wp->w_p_stc; }
char *nvim_win_get_p_cocu(win_T *wp) { return wp->w_p_cocu; }
int nvim_win_get_minscwidth(win_T *wp) { return wp->w_minscwidth; }
linenr_T nvim_win_get_nrwidth_line_count(win_T *wp) { return wp->w_nrwidth_line_count; }
void nvim_win_set_nrwidth_line_count(win_T *wp, linenr_T val) { wp->w_nrwidth_line_count = val; }
int nvim_win_get_nrwidth_width(win_T *wp) { return wp->w_nrwidth_width; }
void nvim_win_set_nrwidth_width(win_T *wp, int val) { wp->w_nrwidth_width = val; }
void nvim_win_set_statuscol_line_count(win_T *wp, linenr_T val) { wp->w_statuscol_line_count = val; }
linenr_T nvim_win_buf_line_count(win_T *wp) { return wp->w_buffer->b_ml.ml_line_count; }
int nvim_win_buf_meta_total_signtext(win_T *wp) { return buf_meta_total(wp->w_buffer, kMTMetaSignText) > 0; }
OptInt nvim_get_p_wmw(void) { return p_wmw; }
OptInt nvim_get_p_wh(void) { return p_wh; }
OptInt nvim_get_p_wmh(void) { return p_wmh; }
OptInt nvim_get_p_wiw(void) { return p_wiw; }
int nvim_get_p_sb(void) { return p_sb ? 1 : 0; }
int nvim_get_p_spr(void) { return p_spr ? 1 : 0; }
int nvim_get_Rows(void) { return Rows; }
int nvim_get_Columns(void) { return Columns; }
int nvim_win_field_height(win_T *wp) { return wp->w_height; }
void nvim_win_field_set_height(win_T *wp, int val) { wp->w_height = val; }
void nvim_win_set_hsep_height(win_T *wp, int val) { wp->w_hsep_height = val; }
void nvim_win_set_status_height(win_T *wp, int val) { wp->w_status_height = val; }
int nvim_win_field_width(win_T *wp) { return wp->w_width; }
void nvim_win_field_set_width(win_T *wp, int val) { wp->w_width = val; }
void nvim_win_set_vsep_width(win_T *wp, int val) { wp->w_vsep_width = val; }
void nvim_frame_set_height(frame_T *frp, int val) { frp->fr_height = val; }
void nvim_frame_set_width(frame_T *frp, int val) { frp->fr_width = val; }
void nvim_frame_new_height(frame_T *topfrp, int height, bool topfirst, bool wfh, bool set_ch) { frame_new_height(topfrp, height, topfirst, wfh, set_ch); }
void nvim_win_config_float(win_T *wp) { win_config_float(wp, wp->w_config); }
void nvim_win_fix_scroll(bool upd_topline) { win_fix_scroll(upd_topline); }
void nvim_redraw_all_later(int type) { redraw_all_later(type); }
int nvim_win_get_config_height(win_T *wp) { return wp->w_config.height; }
void nvim_win_set_config_height(win_T *wp, int val) { wp->w_config.height = val; }
int nvim_win_get_config_width(win_T *wp) { return wp->w_config.width; }
void nvim_win_set_config_width(win_T *wp, int val) { wp->w_config.width = val; }
int64_t nvim_get_window_p_ch(void) { return p_ch; }
void nvim_set_redraw_cmdline(bool val) { redraw_cmdline = val; }
int nvim_win_get_winbar_height(win_T *wp) { return wp->w_winbar_height; }
int nvim_win_get_status_height(win_T *wp) { return wp->w_status_height; }
int nvim_get_State(void) { return State; }
void nvim_set_State(int val) { State = val; }
int nvim_get_real_state(void) { return get_real_state(); }
int64_t nvim_get_p_ph(void) { return p_ph; }
int64_t nvim_get_p_pw(void) { return p_pw; }
int64_t nvim_get_p_pmw(void) { return p_pmw; }
int nvim_win_get_p_diff(win_T *wp) { return wp->w_p_diff; }
int nvim_win_get_p_crb(win_T *wp) { return wp->w_p_crb; }
void nvim_set_curwin(win_T *wp) { curwin = wp; }
void nvim_set_curbuf(buf_T *buf) { curbuf = buf; }
int nvim_win_buf_meta_total_lines(win_T *wp) { return buf_meta_total(wp->w_buffer, kMTMetaLines) > 0; }
int nvim_win_is_cmdwin(win_T *wp) { return wp == cmdwin_win; }
int nvim_win_get_scwidth(win_T *wp) { return wp->w_scwidth; }
char *nvim_get_p_cpo(void) { return p_cpo; }
char *nvim_win_get_p_sbr(win_T *wp) { return wp->w_p_sbr; }
char *nvim_get_p_sbr(void) { return p_sbr; }
char *nvim_get_empty_string_option(void) { return empty_string_option; }
// Colorcolumn accessors
char *nvim_win_get_p_cc(win_T *wp) { return wp->w_p_cc; }
int64_t nvim_win_get_buf_b_p_tw(win_T *wp) { return wp->w_buffer->b_p_tw; }
int nvim_win_has_buffer(win_T *wp) { return wp->w_buffer != NULL; }
int *nvim_win_get_p_cc_cols(win_T *wp) { return wp->w_p_cc_cols; }
void nvim_win_set_p_cc_cols(win_T *wp, int *cols) { wp->w_p_cc_cols = cols; }
void nvim_win_free_p_cc_cols(win_T *wp) { xfree(wp->w_p_cc_cols); wp->w_p_cc_cols = NULL; }
int nvim_win_get_p_list(win_T *wp) { return wp->w_p_list; }
uint32_t nvim_win_get_lcs_prec(win_T *wp) { return wp->w_p_lcs_chars.prec; }
int nvim_win_get_p_cul(win_T *wp) { return wp->w_p_cul; }
OptInt nvim_win_get_p_cole(win_T *wp) { return wp->w_p_cole; }
OptInt nvim_win_get_p_so(win_T *wp) { return wp->w_p_so; }
OptInt nvim_win_get_p_siso(win_T *wp) { return wp->w_p_siso; }
OptInt nvim_get_p_so(void) { return p_so; }
OptInt nvim_get_p_siso(void) { return p_siso; }
OptInt nvim_get_p_ls(void) { return p_ls; }
int nvim_get_p_wbr_empty(void) { return *p_wbr == NUL; }
OptInt nvim_get_p_stal(void) { return p_stal; }
const char *nvim_get_p_ead(void) { return p_ead; }
int nvim_first_tabpage_has_next(void) { return first_tabpage != NULL && first_tabpage->tp_next != NULL; }
int nvim_win_get_winrow(win_T *wp) { return wp->w_winrow; }
int nvim_win_get_wincol(win_T *wp) { return wp->w_wincol; }
int nvim_win_get_winrow_off(win_T *wp) { return wp->w_winrow_off; }
int nvim_win_get_wincol_off(win_T *wp) { return wp->w_wincol_off; }
void nvim_win_set_winrow(win_T *wp, int val) { wp->w_winrow = val; }
void nvim_win_set_wincol(win_T *wp, int val) { wp->w_wincol = val; }
void nvim_win_set_redr_status(win_T *wp, int val) { wp->w_redr_status = val; }
ScreenGrid *nvim_win_get_grid_alloc(win_T *wp) { return &wp->w_grid_alloc; }
int nvim_win_get_config_hide(win_T *wp) { return wp ? wp->w_config.hide : 0; }
void nvim_win_set_pos_changed(win_T *wp, int val) { wp->w_pos_changed = val; }
int nvim_win_get_config_relative(win_T *wp) { return (int)wp->w_config.relative; }
int nvim_win_get_config_window(win_T *wp) { return wp ? wp->w_config.window : 0; }

/// Get the w_config.zindex field.
int nvim_win_get_config_zindex(win_T *wp) { return wp ? wp->w_config.zindex : 50; }

int nvim_win_get_config_focusable(win_T *wp) { return wp ? wp->w_config.focusable : 0; }
frame_T *nvim_get_topframe(void) { return topframe; }
int nvim_win_get_w_width(win_T *wp) { return wp->w_width; }
int nvim_win_get_w_height(win_T *wp) { return wp->w_height; }
int nvim_win_get_hsep_height(win_T *wp) { return wp->w_hsep_height; }
int nvim_win_get_vsep_width(win_T *wp) { return wp->w_vsep_width; }
int nvim_win_get_wcol(win_T *wp) { return wp->w_wcol; }
void nvim_win_set_wcol(win_T *wp, int val) { wp->w_wcol = val; }
int nvim_win_get_wrow(win_T *wp) { return wp->w_wrow; }
void nvim_win_set_wrow(win_T *wp, int val) { if (wp) { wp->w_wrow = val; } }
int nvim_win_get_p_sms(win_T *wp) { return wp ? wp->w_p_sms : 0; }
void nvim_win_set_p_sms(win_T *wp, int val) { if (wp) { wp->w_p_sms = val; } }
frame_T *nvim_tabpage_get_topframe(tabpage_T *tp) { return tp->tp_topframe; }
win_T *nvim_get_prevwin(void) { return prevwin; }
int nvim_win_get_endrow(win_T *wp) { return W_ENDROW(wp); }
size_t nvim_win_get_status_click_defs_size(win_T *wp) { return wp ? wp->w_status_click_defs_size : 0; }
int nvim_win_get_redr_status(win_T *wp) { return wp ? wp->w_redr_status : 0; }
size_t nvim_get_tab_page_click_defs_size(void) { return tab_page_click_defs_size; }
int nvim_win_get_endcol(win_T *wp) { return W_ENDCOL(wp); }
schar_T nvim_win_get_fcs_vert(win_T *wp) { return wp->w_p_fcs_chars.vert; }
schar_T nvim_win_get_fcs_horiz(win_T *wp) { return wp->w_p_fcs_chars.horiz; }
schar_T nvim_win_get_fcs_verthoriz(win_T *wp) { return wp->w_p_fcs_chars.verthoriz; }
schar_T nvim_win_get_fcs_vertright(win_T *wp) { return wp->w_p_fcs_chars.vertright; }
schar_T nvim_win_get_fcs_vertleft(win_T *wp) { return wp->w_p_fcs_chars.vertleft; }
schar_T nvim_win_get_fcs_horizdown(win_T *wp) { return wp->w_p_fcs_chars.horizdown; }
schar_T nvim_win_get_fcs_horizup(win_T *wp) { return wp->w_p_fcs_chars.horizup; }
schar_T nvim_win_get_fcs_stl(win_T *wp) { return wp->w_p_fcs_chars.stl; }
schar_T nvim_win_get_fcs_stlnc(win_T *wp) { return wp->w_p_fcs_chars.stlnc; }
schar_T nvim_win_get_fcs_foldclosed(win_T *wp) { return wp->w_p_fcs_chars.foldclosed; }
schar_T nvim_win_get_fcs_foldopen(win_T *wp) { return wp->w_p_fcs_chars.foldopen; }
schar_T nvim_win_get_fcs_foldsep(win_T *wp) { return wp->w_p_fcs_chars.foldsep; }
schar_T nvim_win_get_fcs_foldinner(win_T *wp) { return wp->w_p_fcs_chars.foldinner; }
schar_T nvim_win_get_fcs_diff(win_T *wp) { return wp->w_p_fcs_chars.diff; }
schar_T nvim_win_get_lcs_ext(win_T *wp) { return wp->w_p_lcs_chars.ext; }
int nvim_win_get_wrap_flags(win_T *wp) { return wp->w_p_wrap_flags; }
int nvim_win_get_p_wrap(win_T *wp) { return wp->w_p_wrap; }
colnr_T nvim_win_get_virtcol(win_T *wp) { return wp->w_virtcol; }
void nvim_win_set_virtcol(win_T *wp, colnr_T val) { if (wp) { wp->w_virtcol = val; } }
int nvim_win_get_p_cuc(win_T *wp) { return wp->w_p_cuc; }
linenr_T nvim_win_get_cursorline(win_T *wp) { return wp->w_cursorline; }
int nvim_win_get_p_culopt_flags(win_T *wp) { return wp->w_p_culopt_flags; }
linenr_T nvim_win_get_cursor_lnum(win_T *wp) { return wp->w_cursor.lnum; }
linenr_T nvim_win_get_topline(win_T *wp) { return wp->w_topline; }
void nvim_win_set_topline(win_T *wp, linenr_T val) { wp->w_topline = val; }
void nvim_win_set_topline_was_set(win_T *wp, int val) { if (wp) { wp->w_topline_was_set = val != 0; } }
linenr_T nvim_win_get_botline(win_T *wp) { return wp->w_botline; }
int nvim_win_get_redr_type(win_T *wp) { return wp ? wp->w_redr_type : 0; }
void nvim_win_set_redr_type(win_T *wp, int val) { if (wp) { wp->w_redr_type = val; } }
int nvim_win_get_lines_valid(win_T *wp) { return wp ? wp->w_lines_valid : 0; }

// NOTE: nvim_win_set_lines_valid already defined earlier in this file

linenr_T nvim_win_get_redraw_top(win_T *wp) { return wp ? wp->w_redraw_top : 0; }
void nvim_win_set_redraw_top(win_T *wp, linenr_T val) { if (wp) { wp->w_redraw_top = val; } }
linenr_T nvim_win_get_redraw_bot(win_T *wp) { return wp ? wp->w_redraw_bot : 0; }
void nvim_win_set_redraw_bot(win_T *wp, linenr_T val) { if (wp) { wp->w_redraw_bot = val; } }
int nvim_win_get_topfill(win_T *wp) { return wp->w_topfill; }
void nvim_win_set_topfill(win_T *wp, int val) { wp->w_topfill = val; }
int nvim_win_get_arg_idx(win_T *wp) { return wp->w_arg_idx; }
int nvim_win_get_arg_idx_invalid(win_T *wp) { return wp->w_arg_idx_invalid; }
int nvim_win_argcount(win_T *wp) { return WARGCOUNT(wp); }
colnr_T nvim_win_get_skipcol(win_T *wp) { return wp->w_skipcol; }
void nvim_win_set_skipcol(win_T *wp, colnr_T val) { wp->w_skipcol = val; }
colnr_T nvim_win_get_cursor_col(win_T *wp) { return wp->w_cursor.col; }
void nvim_win_set_cursor_lnum(win_T *wp, linenr_T lnum) { wp->w_cursor.lnum = lnum; }
void nvim_win_set_cursor_col(win_T *wp, colnr_T col) { wp->w_cursor.col = col; }
colnr_T nvim_win_get_cursor_coladd(win_T *wp) { return wp->w_cursor.coladd; }
linenr_T nvim_win_get_valid_cursor_lnum(win_T *wp) { return wp->w_valid_cursor.lnum; }
colnr_T nvim_win_get_valid_cursor_col(win_T *wp) { return wp->w_valid_cursor.col; }
colnr_T nvim_win_get_valid_cursor_coladd(win_T *wp) { return wp->w_valid_cursor.coladd; }

/// Set the valid cursor position (all fields).
void nvim_win_set_valid_cursor(win_T *wp, linenr_T lnum, colnr_T col, colnr_T coladd) { wp->w_valid_cursor.lnum = lnum; wp->w_valid_cursor.col = col; wp->w_valid_cursor.coladd = coladd; }

void nvim_win_set_valid_cursor_col(win_T *wp, colnr_T col) { wp->w_valid_cursor.col = col; }
void nvim_win_set_valid_cursor_coladd(win_T *wp, colnr_T coladd) { wp->w_valid_cursor.coladd = coladd; }
colnr_T nvim_win_get_leftcol(win_T *wp) { return wp->w_leftcol; }
colnr_T nvim_win_get_valid_leftcol(win_T *wp) { return wp->w_valid_leftcol; }
void nvim_win_set_valid_leftcol(win_T *wp, colnr_T val) { wp->w_valid_leftcol = val; }
colnr_T nvim_win_get_valid_skipcol(win_T *wp) { return wp->w_valid_skipcol; }
void nvim_win_set_valid_skipcol(win_T *wp, colnr_T val) { wp->w_valid_skipcol = val; }
int nvim_win_get_viewport_invalid(win_T *wp) { return wp->w_viewport_invalid ? 1 : 0; }
void nvim_win_set_viewport_invalid(win_T *wp, int val) { wp->w_viewport_invalid = val != 0; }
int nvim_win_get_cline_row(win_T *wp) { return wp->w_cline_row; }
void nvim_win_set_cline_row(win_T *wp, int val) { wp->w_cline_row = val; }
int nvim_win_get_cline_height(win_T *wp) { return wp->w_cline_height; }
void nvim_win_set_cline_height(win_T *wp, int val) { wp->w_cline_height = val; }
int nvim_win_get_cline_folded(win_T *wp) { return wp->w_cline_folded ? 1 : 0; }
void nvim_win_set_cline_folded(win_T *wp, int val) { wp->w_cline_folded = val != 0; }
colnr_T nvim_win_get_curswant(win_T *wp) { return wp->w_curswant; }
void nvim_win_set_curswant(win_T *wp, colnr_T val) { wp->w_curswant = val; }
int nvim_win_get_set_curswant(win_T *wp) { return wp->w_set_curswant ? 1 : 0; }
void nvim_win_set_set_curswant(win_T *wp, int val) { wp->w_set_curswant = val != 0; }
int nvim_win_get_p_bri(win_T *wp) { return wp->w_p_bri; }
int nvim_win_get_p_rl(win_T *wp) { return wp->w_p_rl; }
void nvim_win_set_p_rl(win_T *wp, int val) { if (wp) { wp->w_p_rl = val != 0; } }
int nvim_win_get_p_arab(win_T *wp) { return wp ? wp->w_p_arab : 0; }
void *nvim_win_get_w_grid(win_T *wp) { return &wp->w_grid; }
uint32_t nvim_win_get_lcs_tab1(win_T *wp) { return wp->w_p_lcs_chars.tab1; }
bool nvim_win_get_briopt_sbr(win_T *wp) { return wp->w_briopt_sbr; }
int nvim_win_hl_attr(win_T *wp, int hlf) { return win_hl_attr(wp, hlf); }
buf_T *nvim_win_get_buffer(win_T *wp) { return wp->w_buffer; }
const char *nvim_win_ml_get_buf(win_T *wp, linenr_T lnum) { return ml_get_buf(wp->w_buffer, lnum); }
int nvim_ui_has_tabline(void) { return ui_has(kUITabline); }

/// Get a specific border adjustment value for a window.
int nvim_win_get_border_adj(win_T *wp, int idx) { return (wp && idx >= 0 && idx < 4) ? wp->w_border_adj[idx] : 0; }

#define NOWIN           ((win_T *)-1)   // non-existing window

#define ROWS_AVAIL (Rows - p_ch - rs_tabline_height() - rs_global_stl_height())

int nvim_get_rows_avail(void) { return ROWS_AVAIL; }

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


/// Check if the current window is allowed to move to a different buffer.
///
/// @return If the window has 'winfixbuf', or this function will return false.
bool check_can_set_curbuf_disabled(void)
{
  return rs_check_can_set_curbuf_disabled();
}

/// Check if the current window is allowed to move to a different buffer.
///
/// @param forceit If true, do not error. If false and 'winfixbuf' is enabled, error.
///
/// @return If the window has 'winfixbuf', then forceit must be true
///     or this function will return false.
bool check_can_set_curbuf_forceit(int forceit)
{
  return rs_check_can_set_curbuf_forceit(forceit);
}

/// @return the current window, unless in the cmdline window and "prevwin" is
/// set, then return "prevwin".
win_T *prevwin_curwin(void)
  FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_prevwin_curwin();
}

// swbuf_goto_win_with_buf: thin wrapper defined in Phase 2 accessors section.

// 'cmdheight' value explicitly set by the user: window commands are allowed to
// resize the topframe to values higher than this minimum, but not lower.
static OptInt min_set_ch = 1;

OptInt nvim_get_min_set_ch(void) { return min_set_ch; }
void nvim_set_cmdheight_option(int64_t new_ch)
{
  const OptInt save_ch = min_set_ch;
  set_option_value(kOptCmdheight, NUMBER_OPTVAL(new_ch), 0);
  min_set_ch = save_ch;
}

extern void rs_win_equal(win_T *next_curwin, int current, int dir);


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

extern void rs_ui_ext_win_position(win_T *wp, bool validate);

void ui_ext_win_position(win_T *wp, bool validate)
{
  rs_ui_ext_win_position(wp, validate);
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
  return rs_check_split_disallowed((win_T *)wp);
}

/// Like `check_split_disallowed`, but set `err` to the (untranslated) error message on failure and
/// return false. Otherwise return true.
/// @see check_split_disallowed
bool check_split_disallowed_err(const win_T *wp, Error *err)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_check_split_disallowed_err(wp, err);
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
  return rs_win_split(size, flags);
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
  return rs_win_split_ins_full(size, flags, new_wp, dir, to_flatten);
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

  rs_copyFoldingState(oldp, newp);

  // Use the same argument list.
  newp->w_alist = oldp->w_alist;
  newp->w_alist->al_refcount++;
  newp->w_arg_idx = oldp->w_arg_idx;

  // copy options from existing window
  win_copy_options(oldp, newp);

  newp->w_winbar_height = oldp->w_winbar_height;
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
  return rs_make_windows(count, vertical ? 1 : 0);
}


/// Move "wp" into a new split in a given direction, possibly relative to the
/// current window.
/// "wp" must be valid in the current tabpage.
/// Returns FAIL for failure, OK otherwise.
int win_splitmove(win_T *wp, int size, int flags)
{
  return rs_win_splitmove(wp, size, flags);
}

// Move window "win1" to below/right of "win2" and make "win1" the current
// window.  Only works within the same frame!
void win_move_after(win_T *win1, win_T *win2)
{
  rs_win_move_after(win1, win2);
}

void leaving_window(win_T *const win) { rs_leaving_window(win); }

void entering_window(win_T *const win) { rs_entering_window(win); }

void win_init_empty(win_T *wp) { rs_win_init_empty(wp); }

void curwin_init(void) { rs_win_init_empty(curwin); }

/// Closes all windows for buffer `buf` unless there is only one non-floating window.
///
/// @param keep_curwin  don't close `curwin`
void close_windows(buf_T *buf, bool keep_curwin)
{
  RedrawingDisabled++;

  // Start from lastwin to close floating windows with the same buffer first.
  // When the autocommand window is involved win_close() may need to print an error message.
  for (win_T *wp = lastwin; wp != NULL && (is_aucmd_win(lastwin) || !rs_one_window_in_tab(wp, NULL));) {
    if (wp->w_buffer == buf && (!keep_curwin || wp != curwin)
        && !(rs_win_locked(wp) || wp->w_buffer->b_locked > 0)) {
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
            && !(rs_win_locked(wp) || wp->w_buffer->b_locked > 0)) {
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

/// Check if floating windows in tabpage `tp` can be closed.
/// Do not call this when the autocommand window is in use!
///
/// @param tp tabpage to check. Must be NULL for the current tabpage.
/// @return true if all floating windows can be closed
static bool can_close_floating_windows(tabpage_T *tp)
{
  return rs_can_close_floating_windows_tp(tp) != 0;
}

/// @return true if, considering the cmdwin, `win` is safe to close.
/// If false and `win` is the cmdwin, it is closed; otherwise, `err` is set.
bool can_close_in_cmdwin(win_T *win, Error *err)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_can_close_in_cmdwin(win, err);
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
  if (curtab != prev_curtab && rs_valid_tabpage(prev_curtab) && prev_curtab->tp_firstwin == win) {
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
    if (rs_win_valid_any_tab(win)) {
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
    if (rs_last_window(win)) {
      emsg(_("E444: Cannot close last window"));
    } else if (is_aucmd_win(win)) {
      emsg(_(e_autocmd_close));
    } else if (lastwin->w_floating && rs_one_window_in_tab(win, NULL) && is_aucmd_win(lastwin)) {
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
      if (!rs_win_valid_any_tab(win)) {
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
    rs_clear_snapshot(curtab, SNAP_HELP_IDX);
  }

  win_T *wp;
  bool other_buffer = false;

  if (win == curwin) {
    leaving_window(curwin);

    wp = win->w_floating ? win_float_find_altwin(win, NULL) : rs_frame2win(rs_win_altframe(win));

    if (wp->w_buffer != curbuf) {
      rs_reset_VIsual_and_resel();

      other_buffer = true;
      if (!rs_win_valid(win)) {
        return FAIL;
      }
      win->w_locked = true;
      apply_autocmds(EVENT_BUFLEAVE, NULL, NULL, false, curbuf);
      if (!rs_win_valid(win)) {
        return FAIL;
      }
      win->w_locked = false;
      if (rs_last_window(win)) {
        return FAIL;
      }
    }
    win->w_locked = true;
    apply_autocmds(EVENT_WINLEAVE, NULL, NULL, false, curbuf);
    if (!rs_win_valid(win)) {
      return FAIL;
    }
    win->w_locked = false;
    if (rs_last_window(win)) {
      return FAIL;
    }
    if (aborting()) {
      return FAIL;
    }
  }

  do_autocmd_winclosed(win);
  if (!rs_win_valid_any_tab(win)) {
    return OK;
  }

  win_close_buffer(win, free_buf ? DOBUF_UNLOAD : 0, true);

  if (rs_win_valid(win) && win->w_buffer == NULL
      && !win->w_floating && rs_last_window(win)) {
    if (curwin->w_buffer == NULL) {
      curwin->w_buffer = curbuf;
    }
    getout(0);
  }
  if (curtab != prev_curtab && rs_win_valid_any_tab(win)
      && win->w_buffer == NULL) {
    win_close_othertab(win, false, prev_curtab, force);
    return FAIL;
  }

  if (!rs_win_valid(win) || (!win->w_floating && rs_last_window(win))
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

  if (rs_diffopt_closeoff() && had_diffmode && curtab == prev_curtab) {
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
  rs_do_autocmd_winclosed(win);
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
    if (!rs_win_valid_any_tab(win)) {
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
    if (!rs_win_valid_any_tab(win)) {
      return false;
    }
  }

  bufref_T bufref;
  set_bufref(&bufref, win->w_buffer);

  if (win->w_buffer != NULL) {
    did_decrement = close_buffer(win, win->w_buffer, free_buf ? DOBUF_UNLOAD : 0, false, true);
  }

  // Re-validate after autocmds.
  if (!rs_valid_tabpage(tp) || tp == curtab) {
    goto leave_open;
  }
  if (!rs_tabpage_win_valid(tp, win)) {
    goto leave_open;
  }
  if (tp->tp_lastwin->w_floating && rs_one_window_in_tab(win, tp)) {
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
    rs_win_remove(lastwin, NULL);
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
  return rs_winframe_remove(win, dirp, tp, unflat_altfr);
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
  return rs_winframe_find_altwin_full(win, dirp, tp, altfr);
}


/// Undo changes from a prior call to winframe_remove, also restoring lost
/// vertical separators and statuslines, and changed window positions for
/// windows within "unflat_altfr".
/// Caller must ensure no other changes were made to the layout or window sizes!
void winframe_restore(win_T *wp, int dir, frame_T *unflat_altfr)
  FUNC_ATTR_NONNULL_ALL
{
  rs_winframe_restore(wp, dir, unflat_altfr);
}

// Return the tabpage that will be used if the current one is closed.
static tabpage_T *alt_tabpage(void)
{
  return rs_alt_tabpage();
}

/// Set a new height for a frame.  Recursively sets the height for contained
/// frames and windows.  Caller must take care of positions.
///
/// @param topfirst  resize topmost contained frame first.
/// @param wfh       obey 'winfixheight' when there is a choice;
///                  may cause the height not to be set.
/// @param set_ch    set 'cmdheight' to resize topframe.
/// Set height of a frame (thin wrapper -- implementation is in Rust).
void frame_new_height(frame_T *topfrp, int height, bool topfirst, bool wfh, bool set_ch)
{
  rs_frame_new_height(topfrp, height, topfirst, wfh, set_ch);
}

/// Set width of a frame (thin wrapper -- implementation is in Rust).
static void frame_new_width(frame_T *topfrp, int width, bool leftfirst, bool wfw)
{
  rs_frame_new_width(topfrp, width, leftfirst, wfw);
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

  if (rs_one_window_in_tab(firstwin, NULL) && !lastwin->w_floating) {
    if (message && !autocmd_busy) {
      msg(_(m_onlyone), 0);
    }
    return;
  }

  // Be very careful here: autocommands may change the window layout.
  win_T *nextwp;
  for (win_T *wp = firstwin; rs_win_valid(wp); wp = nextwp) {
    nextwp = wp->w_next;

    // autocommands messed this one up
    if (old_curwin != curwin && rs_win_valid(old_curwin)) {
      curwin = old_curwin;
      curbuf = curwin->w_buffer;
    }

    if (wp == curwin) {                 // don't close current window
      continue;
    }

    // autoccommands messed this one up
    if (!buf_valid(wp->w_buffer) && rs_win_valid(wp)) {
      wp->w_buffer = NULL;
      win_close(wp, false, false);
      continue;
    }
    // Check if it's allowed to abandon this window
    int r = can_abandon(wp->w_buffer, forceit);
    if (!rs_win_valid(wp)) {             // autocommands messed wp up
      nextwp = firstwin;
      continue;
    }
    if (!r) {
      if (message && (p_confirm || (cmdmod.cmod_flags & CMOD_CONFIRM)) && p_write) {
        dialog_changed(wp->w_buffer, false);
        if (!rs_win_valid(wp)) {                 // autocommands messed wp up
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
void unuse_tabpage(tabpage_T *tp) { rs_unuse_tabpage(tp); }

// When switching tabpage, handle other side-effects in command_height(), but
// avoid setting frame sizes which are still correct.
static bool command_frame_height = true;

/// Set the relevant pointers to use tab page "tp".  May want to call
/// unuse_tabpage() first.
void use_tabpage(tabpage_T *tp) { rs_use_tabpage(tp); }

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
  topframe->fr_height = Rows - (int)p_ch - rs_global_stl_height();

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
void win_init_size(void) { rs_win_init_size(); }

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
    rs_clear_snapshot(tp, idx);
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
    firstwin->w_winrow = rs_tabline_height();
    firstwin->w_prev_winrow = firstwin->w_winrow;
    win_comp_scroll(curwin);

    newtp->tp_topframe = topframe;
    rs_last_status(0);

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
  return rs_make_tabpages(maxcount);
}

/// Close tabpage `tab`, assuming it has no windows in it.
/// There must be another tabpage or this will crash.
void close_tabpage(tabpage_T *tab)
{
  rs_close_tabpage(tab);
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
  rs_reset_VIsual_and_resel();     // stop Visual mode
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

  rs_last_status(0);  // status line may appear or disappear
  win_float_update_statusline();
  rs_win_comp_pos();      // recompute w_winrow for all windows
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
        rs_win_remove(wp, old_curtab);
        rs_win_append(rs_lastwin_nofloating(), wp, NULL);
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
  rs_goto_tabpage(n);
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
    if (rs_valid_tabpage(tp)) {
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
  return rs_goto_tabpage_lastused();
}

// Enter window "wp" in tab page "tp".
// Also updates the GUI tab.
void goto_tabpage_win(tabpage_T *tp, win_T *wp)
{
  rs_goto_tabpage_win(tp, wp);
}

// Move the current tab page to after tab page "nr".
void tabpage_move(int nr)
{
  rs_tabpage_move(nr);
}

/// Go to another window.
/// When jumping to another buffer, stop Visual mode.  Do this before
/// changing windows so we can yank the selection into the '*' register.
/// (note: this may trigger ModeChanged autocommand!)
/// When jumping to another window on the same buffer, adjust its cursor
/// position to keep the same Visual area.
void win_goto(win_T *wp) { rs_win_goto(wp); }

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
      tp = rs_win_find_tabpage(after);
      if (tp == curtab) {
        tp = NULL;
      }
    }
    rs_win_append(after, new_wp, tp);
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

  rs_foldInitWin(new_wp);
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
  rs_clearFolding(wp);

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
    rs_tagstack_clear_entry(&wp->w_tagstack[i]);
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

  if (rs_win_valid_any_tab(wp)) {
    rs_win_remove(wp, tp);
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

void win_new_screensize(void) { rs_win_new_screensize(); }
/// Called from win_new_screensize() after Rows changed.
///
/// This only does the current tab page, others must be done when made active.
void win_new_screen_rows(void) { rs_win_new_screen_rows(); }

/// Called from win_new_screensize() after Columns changed.
void win_new_screen_cols(void) { rs_win_new_screen_cols(); }

/// Make a snapshot of all the window scroll positions and sizes of the current
/// tab page.
void snapshot_windows_scroll_size(void) { rs_snapshot_windows_scroll_size(); }

void may_make_initial_scroll_size_snapshot(void) { rs_may_make_initial_scroll_size_snapshot(); }

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
      || !rs_get_did_initial_scroll_size_snapshot()) {
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

#define FRACTION_MULT   16384

void win_fix_scroll(bool resize) { rs_win_fix_scroll(resize ? 1 : 0); }
static void win_fix_cursor(bool normal) { rs_win_fix_cursor(normal ? 1 : 0); }

void scroll_to_fraction(win_T *wp, int prev_height)
{
  rs_scroll_to_fraction(wp, prev_height);
}

extern void rs_win_set_inner_size(win_T *wp, int valid_cursor);

void win_set_inner_size(win_T *wp, bool valid_cursor)
{
  rs_win_set_inner_size(wp, valid_cursor ? 1 : 0);
}

void win_comp_scroll(win_T *wp) { rs_win_comp_scroll(wp); }

// command_height: thin wrapper defined in Phase 3 accessors section.

/// Add or remove window bar from window "wp".
///
/// @param make_room Whether to resize frames to make room for winbar.
/// @param valid_cursor Whether the cursor is valid and should be used while
///                     resizing.
///
/// @return Success status.
int set_winbar_win(win_T *wp, bool make_room, bool valid_cursor)
{
  return rs_set_winbar_win(wp, make_room ? 1 : 0, valid_cursor ? 1 : 0);
}

/// Add or remove window bars from all windows in tab depending on the value of 'winbar'.
///
/// @param make_room Whether to resize frames to make room for winbar.
void set_winbar(bool make_room) { rs_set_winbar(make_room ? 1 : 0); }

// A snapshot of the window sizes, to restore them after closing the help
// window.
// Only these fields are used:
// fr_layout
// fr_width
// fr_height
// fr_next
// fr_child
// fr_win (only valid for the old curwin, NULL otherwise)

/// Restore a previously created snapshot, if there is any.
/// This is only done if the screen size didn't change and the window layout is
/// still the same.
///
/// @param close_curwin  closing current window
void restore_snapshot(int idx, int close_curwin) { rs_restore_snapshot(idx, close_curwin); }

/// Simple int comparison function for use with qsort()
/// Check "cc" as 'colorcolumn' and update the members of "wp" (thin wrapper).
///
/// @param cc  when NULL: use "wp->w_p_cc"
/// @param wp  when NULL: only parse "cc"
///
/// @return error message, NULL if it's OK.
const char *check_colorcolumn(char *cc, win_T *wp)
{
  return rs_check_colorcolumn(cc, wp);
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

int nvim_win_get_empty_rows(win_T *wp) { return wp ? wp->w_empty_rows : 0; }
void nvim_win_set_leftcol(win_T *wp, int val) { if (wp) { wp->w_leftcol = (colnr_T)val; } }
void nvim_win_set_botline(win_T *wp, int val) { if (wp) { wp->w_botline = (linenr_T)val; } }
void nvim_win_set_empty_rows(win_T *wp, int val) { if (wp) { wp->w_empty_rows = val; } }
int nvim_win_buffer_eq(win_T *wp, buf_T *buf) { return (wp && wp->w_buffer == buf) ? 1 : 0; }
int nvim_win_grid_alloc_valid(win_T *wp) { return wp ? wp->w_grid_alloc.valid : 0; }
void nvim_win_grid_alloc_set_valid(win_T *wp, int val) { if (wp) { wp->w_grid_alloc.valid = (val != 0); } }
int nvim_win_get_w_redr_type(win_T *wp) { return wp ? wp->w_redr_type : 0; }
void nvim_win_set_w_redr_type(win_T *wp, int val) { if (wp) { wp->w_redr_type = val; } }
void nvim_win_set_w_lines_valid(win_T *wp, int val) { if (wp) { wp->w_lines_valid = (linenr_T)val; } }
int nvim_win_get_filler_rows(win_T *wp) { return wp ? wp->w_filler_rows : 0; }
void nvim_win_set_filler_rows(win_T *wp, int val) { if (wp) { wp->w_filler_rows = val; } }
int nvim_win_get_botfill(win_T *wp) { return wp ? (wp->w_botfill ? 1 : 0) : 0; }
void nvim_win_set_botfill(win_T *wp, int val) { if (wp) { wp->w_botfill = (val != 0); } }
int nvim_win_grid_has_target(win_T *wp) { return (wp && wp->w_grid.target) ? 1 : 0; }
int nvim_win_get_scbind_pos(win_T *wp) { return wp ? wp->w_scbind_pos : 0; }
void nvim_win_set_scbind_pos(win_T *wp, int val) { if (wp) { wp->w_scbind_pos = val; } }
int nvim_win_buf_is_empty(win_T *wp) { return (wp && wp->w_buffer) ? buf_is_empty(wp->w_buffer) : 1; }
void nvim_win_set_fraction(win_T *wp, int val) { if (wp) { wp->w_fraction = val; } }
int nvim_tabpage_get_ch_used(tabpage_T *tp) { return tp ? (int)tp->tp_ch_used : 0; }
int nvim_win_has_winnr(win_T *wp, tabpage_T *tp) { return (wp && tp) ? (int)win_has_winnr(wp, tp) : 0; }
void nvim_set_p_wmh(int64_t val) { p_wmh = val; }
void nvim_set_p_wmw(int64_t val) { p_wmw = val; }
void nvim_emsg_noroom(void) { emsg(_(e_noroom)); }

// Accessors for rs_win_set_inner_size (Phase 4)
int nvim_win_get_width_request(win_T *wp) { return wp ? wp->w_width_request : 0; }
int nvim_win_get_height_request(win_T *wp) { return wp ? wp->w_height_request : 0; }
int nvim_win_get_prev_fraction_row(win_T *wp) { return wp ? wp->w_prev_fraction_row : 0; }
void nvim_win_set_view_height(win_T *wp, int val) { if (wp) { wp->w_view_height = val; } }
void nvim_win_set_view_width(win_T *wp, int val) { if (wp) { wp->w_view_width = val; } }
void nvim_win_set_height_outer(win_T *wp, int val) { if (wp) { wp->w_height_outer = val; } }
void nvim_win_set_width_outer(win_T *wp, int val) { if (wp) { wp->w_width_outer = val; } }
void nvim_win_set_winrow_off(win_T *wp, int val) { if (wp) { wp->w_winrow_off = val; } }
void nvim_win_set_wincol_off(win_T *wp, int val) { if (wp) { wp->w_wincol_off = val; } }
int nvim_win_get_p_spk_char(void) { return (int)(unsigned char)*p_spk; }
int nvim_get_exiting(void) { return exiting ? 1 : 0; }
void nvim_win_comp_scroll_wrapper(win_T *wp) { if (wp) { rs_win_comp_scroll(wp); } }
// nvim_validate_cursor_win already defined in move.c
// nvim_changed_line_abv_curs_win already defined in change_ffi.c
// nvim_invalidate_botline already defined in move.c
void nvim_curs_columns_win(win_T *wp) { if (wp) { curs_columns(wp, true); } }
void nvim_terminal_check_size_win(win_T *wp) { if (wp && wp->w_buffer->terminal) { terminal_check_size(wp->w_buffer->terminal); } }
int nvim_win_border_height_wrapper(win_T *wp) { return wp ? win_border_height(wp) : 0; }
int nvim_win_border_width_wrapper(win_T *wp) { return wp ? win_border_width(wp) : 0; }
int nvim_win_get_grid_alloc_handle(win_T *wp) { return wp ? wp->w_grid_alloc.handle : 0; }
int nvim_win_get_w_handle(win_T *wp) { return wp ? wp->handle : 0; }
// nvim_win_get_border_adj already defined earlier in this file
int nvim_ui_has_multigrid(void) { return ui_has(kUIMultigrid) ? 1 : 0; }
void nvim_ui_call_win_viewport_margins_wrapper(win_T *wp) {
  if (wp && ui_has(kUIMultigrid)) {
    ui_call_win_viewport_margins(wp->w_grid_alloc.handle, wp->handle,
                                 wp->w_winrow_off, wp->w_border_adj[2],
                                 wp->w_wincol_off, wp->w_border_adj[1]);
  }
}
void nvim_win_set_inner_size(win_T *wp, int valid_cursor) { if (wp) { rs_win_set_inner_size(wp, valid_cursor != 0); } }

/// Get a snapshot pointer from a tabpage.
frame_T *nvim_tabpage_get_snapshot(tabpage_T *tp, int idx)
{
  if (!tp || idx < 0 || idx >= SNAP_COUNT) {
    return NULL;
  }
  return tp->tp_snapshot[idx];
}

void nvim_tabpage_set_snapshot(tabpage_T *tp, int idx, frame_T *val) { if (tp && idx >= 0 && idx < SNAP_COUNT) { tp->tp_snapshot[idx] = val; } }

// nvim_curbuf_line_count() — already defined in move.c

int nvim_win_buf_is_curbuf(win_T *wp) { return wp && wp->w_buffer == curbuf; }
void nvim_win_save_cursor_to_save(win_T *wp) { if (wp) { wp->w_save_cursor.w_cursor_save = wp->w_cursor; } }
void nvim_win_save_topline_to_save(win_T *wp) { if (wp) { wp->w_save_cursor.w_topline_save = wp->w_topline; } }
void nvim_win_save_cursor_to_corr(win_T *wp) { if (wp) { wp->w_save_cursor.w_cursor_corr = wp->w_cursor; } }
void nvim_win_save_topline_to_corr(win_T *wp) { if (wp) { wp->w_save_cursor.w_topline_corr = wp->w_topline; } }

/// Check if w_save_cursor.w_cursor_corr equals w_cursor (via equalpos).
int nvim_win_cursor_eq_save_corr(win_T *wp) { return wp ? equalpos(wp->w_save_cursor.w_cursor_corr, wp->w_cursor) : 0; }

/// Check if w_save_cursor.w_topline_corr equals w_topline.
int nvim_win_topline_eq_save_corr(win_T *wp) { return wp ? wp->w_save_cursor.w_topline_corr == wp->w_topline : 0; }

/// Get w_save_cursor.w_cursor_save.lnum.
linenr_T nvim_win_get_save_cursor_save_lnum(win_T *wp) { return wp ? wp->w_save_cursor.w_cursor_save.lnum : 0; }

/// Get w_save_cursor.w_topline_save.
linenr_T nvim_win_get_save_topline_save(win_T *wp) { return wp ? wp->w_save_cursor.w_topline_save : 0; }

void nvim_win_restore_cursor_from_save(win_T *wp) { if (wp) { wp->w_cursor = wp->w_save_cursor.w_cursor_save; } }
void nvim_win_restore_topline_from_save(win_T *wp) { if (wp) { wp->w_topline = wp->w_save_cursor.w_topline_save; } }

/// Check if w_save_cursor.w_topline_save > buffer line count.
int nvim_win_save_topline_gt_buf_line_count(win_T *wp) { return (wp && wp->w_buffer) ? wp->w_save_cursor.w_topline_save > wp->w_buffer->b_ml.ml_line_count : 0; }

void nvim_ga_init_int(garray_T *gap) { ga_init(gap, (int)sizeof(int), 1); }
void nvim_ga_grow(garray_T *gap, int n) { ga_grow(gap, n); }
int nvim_ga_get_len(garray_T *gap) { return gap ? gap->ga_len : 0; }

// nvim_ga_set_len() — already defined in fold.c

/// Get an int item from a growarray by index.
int nvim_ga_get_int(garray_T *gap, int idx) { return (gap && gap->ga_data && idx >= 0 && idx < gap->ga_len) ? ((int *)gap->ga_data)[idx] : 0; }

void nvim_ga_set_int(garray_T *gap, int idx, int val) { if (gap && gap->ga_data && idx >= 0) { ((int *)gap->ga_data)[idx] = val; } }
void nvim_comp_col(void) { comp_col(); }

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
void nvim_win_set_prev_height(win_T *wp, int val) { if (wp) { wp->w_prev_height = val; } }
void nvim_win_float_anchor_laststatus(void) { win_float_anchor_laststatus(); }
void nvim_win_set_floating(win_T *wp, int val) { if (wp) { wp->w_floating = val; } }
int nvim_win_get_fraction(win_T *wp) { return wp ? wp->w_fraction : 0; }
void nvim_win_set_prev_fraction_row(win_T *wp, int val) { if (wp) { wp->w_prev_fraction_row = val; } }
int nvim_get_p_ea(void) { return p_ea ? 1 : 0; }
int nvim_get_p_ead_char(void) { return (p_ead && *p_ead) ? (int)(unsigned char)*p_ead : 0; }

// nvim_get_p_ch() — already defined in message.c
// nvim_get_sc_col() — already defined in message.c

void nvim_set_p_wiw(int64_t val) { p_wiw = val; }
void nvim_set_p_wh(int64_t val) { p_wh = val; }
win_T *nvim_win_alloc_wrapper(win_T *after, int hidden) { return win_alloc(after, hidden != 0); }
void nvim_new_frame_wrapper(win_T *wp) { new_frame(wp); }
void nvim_win_init_wrapper(win_T *wp, win_T *oldwin, int flags) { win_init(wp, oldwin, flags); }
void nvim_frame_flatten_wrapper(frame_T *frp) { rs_frame_flatten(frp); }
frame_T *nvim_xcalloc_frame(void) { return xcalloc(1, sizeof(frame_T)); }
void nvim_ui_comp_remove_grid_win(win_T *wp) { if (wp) { ui_comp_remove_grid(&wp->w_grid_alloc); } }
void nvim_ui_call_win_hide_win(win_T *wp) { if (wp) { ui_call_win_hide(wp->w_grid_alloc.handle); } }
void nvim_win_free_grid_wrapper(win_T *wp, int reinit) { if (wp) { win_free_grid(wp, reinit != 0); } }

/// Wrapper: merge_win_config(&wp->w_config, WIN_CONFIG_INIT) + CLEAR_FIELD(wp->w_border_adj).
void nvim_merge_win_config_init(win_T *wp)
{
  if (wp) {
    merge_win_config(&wp->w_config, WIN_CONFIG_INIT);
    CLEAR_FIELD(wp->w_border_adj);
  }
}

void nvim_redraw_later_wrapper(win_T *wp, int type) { if (wp) { redraw_later(wp, type); } }
void nvim_status_redraw_all_wrapper(void) { status_redraw_all(); }
void nvim_msg_clr_eos_force(void) { msg_clr_eos_force(); }
int nvim_is_aucmd_win(win_T *wp) { return is_aucmd_win(wp) ? 1 : 0; }
int nvim_win_get_config_external_int(win_T *wp) { return wp ? (int)wp->w_config.external : 0; }

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

void nvim_set_msg_row_val(int val) { msg_row = val; }
void nvim_set_msg_col_val(int val) { msg_col = val; }
void nvim_win_set_frame(win_T *wp, frame_T *frp) { if (wp) { wp->w_frame = frp; } }
void nvim_set_first_tabpage(tabpage_T *tp) { first_tabpage = tp; }
void nvim_tabpage_set_next(tabpage_T *tp, tabpage_T *next) { tp->tp_next = next; }
int nvim_win_get_tcl_flags(void) { return (int)tcl_flags; }
void nvim_win_set_buffer_raw(win_T *wp, buf_T *buf) { wp->w_buffer = buf; }
void nvim_buf_inc_nwindows(buf_T *buf) { buf->b_nwindows++; }
void nvim_win_init_empty_wrapper(win_T *wp) { rs_win_init_empty(wp); }
void nvim_emsg_e_floatonly(void) { emsg(e_floatonly); }
void nvim_emsg_e_floatexchange(void) { emsg(e_floatexchange); }
void nvim_emsg_e443(void) { emsg(_("E443: Cannot rotate when another window is split")); }
void nvim_iemsg_move_other_frame(void) { iemsg("INTERNAL: trying to move a window into another frame"); }
int nvim_text_or_buf_locked(void) { return text_or_buf_locked() ? 1 : 0; }
void nvim_win_copy_cursor(win_T *dst, win_T *src) { if (dst && src) { dst->w_cursor = src->w_cursor; } }
void nvim_win_enter(win_T *wp, int undo_sync) { win_enter(wp, undo_sync != 0); }
void nvim_emsg_e_autocmd_close(void) { emsg(_(e_autocmd_close)); }
void nvim_internal_error_othertab(void) { internal_error("win_close_othertab()"); }
void nvim_win_new_screen_rows_wrapper(void) { win_new_screen_rows(); }
win_T *nvim_win_free_mem_wrapper(win_T *win, int *dirp, tabpage_T *tp) { return win_free_mem(win, dirp, tp); }
void nvim_inc_split_disallowed(void) { split_disallowed++; }
void nvim_dec_split_disallowed(void) { split_disallowed--; }
void nvim_ui_call_win_close_win(win_T *wp) { if (wp) { ui_call_win_close(wp->w_grid_alloc.handle); } }
void nvim_tabpage_set_curwin(tabpage_T *tp, win_T *wp) { tp->tp_curwin = wp; }
void nvim_set_curbuf_from_curwin(void) { curbuf = curwin->w_buffer; }
void nvim_check_cursor_win_wrapper(win_T *wp) { check_cursor(wp); }

/// Get w_frame->fr_parent from a window (for win_close frame comparison).
frame_T *nvim_win_get_frame_parent(win_T *wp) { return (wp && wp->w_frame) ? wp->w_frame->fr_parent : NULL; }

buf_T *nvim_get_firstbuf_wrapper(void) { return firstbuf; }
int nvim_can_close_floating_windows(tabpage_T *tp) { return can_close_floating_windows(tp) ? 1 : 0; }
unsigned nvim_get_swb_flags(void) { return swb_flags; }
void nvim_win_goto_wrapper(win_T *wp) { win_goto(wp); }
int nvim_win_split_wrapper(int size, int flags) { return win_split(size, flags); }
int nvim_win_splitmove_wrapper(win_T *wp, int size, int flags) { return win_splitmove(wp, size, flags); }
int nvim_do_cmdline_cmd_wrapper(const char *cmd) { return do_cmdline_cmd(cmd); }
void nvim_emsg_e_cmdwin(void) { emsg(_(e_cmdwin)); }
int nvim_bt_quickfix_curbuf(void) { return bt_quickfix(curbuf) ? 1 : 0; }
void nvim_msg_onlyone(void) { msg(_(m_onlyone), 0); }

// Phase 1 accessors: curbuf/winfixbuf, split_disallowed, cmdwin state
int nvim_get_curwin_p_wfb(void) { return curwin->w_p_wfb ? 1 : 0; }
void nvim_emsg_e_winfixbuf(void) { emsg(_(e_winfixbuf_cannot_go_to_buffer)); }
int nvim_get_split_disallowed(void) { return split_disallowed; }
int nvim_win_buf_locked_split(win_T *wp) { return wp->w_buffer->b_locked_split ? 1 : 0; }
void nvim_emsg_e242(void) { emsg(_("E242: Can't split a window while closing another")); }
void nvim_emsg_e_cannot_split_when_closing(void) { emsg(_(e_cannot_split_window_when_closing_buffer)); }

// Phase 2 accessors: win_split and win_splitmove orchestration
int nvim_may_open_tabpage(void) { return may_open_tabpage(); }
int nvim_get_cmdmod_split(void) { return cmdmod.cmod_split; }
void nvim_emsg_e442(void) { emsg(_("E442: Can't split topleft and botright at the same time")); }
/// Wrapper for win_split_ins callable from Rust (handles win_enter_ext and option restore).
win_T *nvim_win_split_ins_wrapper(int size, int flags, win_T *new_wp, int dir, frame_T *to_flatten) { return rs_win_split_ins_full(size, flags, new_wp, dir, to_flatten); }
int nvim_win_get_floating_win(win_T *wp) { return (wp && wp->w_floating) ? 1 : 0; }
win_T *nvim_win_get_prev_win(win_T *wp) { return wp ? wp->w_prev : NULL; }

/// Wrapper: rs_win_valid(prevwin) check for 'p' command.
/// Returns prevwin if valid and focusable, NULL otherwise.
win_T *nvim_get_valid_prevwin(void)
{
  if (!rs_win_valid(prevwin) || prevwin->w_config.hide || !prevwin->w_config.focusable) {
    return NULL;
  }
  return prevwin;
}

/// Wrapper: The '=' equalize command.
void nvim_do_window_equalize(void)
{
  int mod = cmdmod.cmod_split & (WSP_VERT | WSP_HOR);
  rs_win_equal(NULL, 0, mod == WSP_VERT ? 'v' : mod == WSP_HOR ? 'h' : 'b');
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
  if (rs_check_text_or_curbuf_locked(NULL)) {
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
  if ((len = rs_find_ident_under_cursor(&ptr, FIND_IDENT)) == 0) {
    return;
  }

  ptr = xmemdupz(ptr, len);
  find_pattern_in_path(ptr, 0, len, true, Prenum == 0, type,
                       Prenum1, ACTION_SPLIT, 1, MAXLNUM, false, false);
  xfree(ptr);
  curwin->w_set_curswant = true;
}

// New one-liner wrappers for rs_do_window_g (Phase 3)
// (nvim_inc/dec_no_mapping, nvim_inc/dec_allow_keys, nvim_goto_tabpage,
//  nvim_langmap_adjust, nvim_goto_tabpage_lastused, nvim_set_g_do_tagpreview,
//  nvim_set_postponed_split already exist in normal_shim.c / tag_shim.c)
int nvim_get_p_pvh(void) { return (int)p_pvh; }
void nvim_do_nv_ident(int prefix, int xchar) { do_nv_ident(prefix, xchar); }
void nvim_set_cmdmod_tab_to_curtab_idx(void) { cmdmod.cmod_tab = rs_tabpage_index(curtab) + 1; }

/// External window case for 'g' sub-switch ('e' command).
/// Kept in C to avoid marshalling WinConfig and Error types.
void nvim_do_window_g_external(void)
{
  if (curwin->w_floating || !ui_has(kUIMultigrid)) {
    beep_flush();
    return;
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
double nvim_win_get_config_row(win_T *wp) { return wp ? wp->w_config.row : 0.0; }
double nvim_win_get_config_col(win_T *wp) { return wp ? wp->w_config.col : 0.0; }
int nvim_win_get_config_anchor(win_T *wp) { return wp ? (int)wp->w_config.anchor : 0; }
int nvim_win_get_config_fixed(win_T *wp) { return wp ? wp->w_config.fixed : 0; }
int nvim_win_get_config_mouse_flag(win_T *wp) { return wp ? wp->w_config.mouse : 0; }
int nvim_win_get_config_bufpos_lnum(win_T *wp) { return wp ? (int)wp->w_config.bufpos.lnum : -1; }
int nvim_win_get_config_bufpos_col(win_T *wp) { return wp ? (int)wp->w_config.bufpos.col : 0; }
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
void nvim_win_call_win_grid_alloc(win_T *wp) { if (wp) { win_grid_alloc(wp); } }

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

void nvim_win_ui_call_win_hide(int grid_handle) { ui_call_win_hide(grid_handle); }

void nvim_win_ui_call_win_external_pos(int grid_handle, int win_handle)
{
  ui_call_win_external_pos(grid_handle, win_handle);
}

void nvim_win_ui_check_cursor_grid(int grid_handle) { ui_check_cursor_grid(grid_handle); }

// Accessor: get the 'handle' integer field from a ScreenGrid pointer
int nvim_screengrid_get_handle_from_win_grid_alloc(win_T *wp)
{
  return wp ? wp->w_grid_alloc.handle : 0;
}

// =============================================================================
// C Accessors for Phase 3: leaving_window / entering_window / win_init_empty
// =============================================================================

/// Check if window's buffer is a prompt buffer.
int nvim_win_bt_prompt(win_T *wp) { return (wp && wp->w_buffer) ? (bt_prompt(wp->w_buffer) ? 1 : 0) : 0; }

/// Get b_prompt_insert from a buffer (restart_edit value for prompt buffer).
int nvim_buf_get_prompt_insert(buf_T *buf) { return buf ? buf->b_prompt_insert : 0; }

/// Set b_prompt_insert on a buffer.
void nvim_buf_set_prompt_insert(buf_T *buf, int val) { if (buf) { buf->b_prompt_insert = val; } }

/// Get the stop_insert_mode global.
int nvim_get_stop_insert_mode(void) { return stop_insert_mode ? 1 : 0; }

/// Set the stop_insert_mode global.
void nvim_set_stop_insert_mode(int val) { stop_insert_mode = val != 0; }

/// Get the window's buffer pointer.
buf_T *nvim_win_get_buf_ptr(win_T *wp) { return wp ? wp->w_buffer : NULL; }

/// Set w_pcmark (lnum and col).
void nvim_win_set_pcmark(win_T *wp, linenr_T lnum, colnr_T col)
{
  if (wp) {
    wp->w_pcmark.lnum = lnum;
    wp->w_pcmark.col = col;
  }
}

/// Set w_prev_pcmark (lnum and col).
void nvim_win_set_prev_pcmark(win_T *wp, linenr_T lnum, colnr_T col)
{
  if (wp) {
    wp->w_prev_pcmark.lnum = lnum;
    wp->w_prev_pcmark.col = col;
  }
}

/// Sync w_s to point to the window's buffer's b_s.
void nvim_win_sync_s(win_T *wp) { if (wp && wp->w_buffer) { wp->w_s = &wp->w_buffer->b_s; } }

// =============================================================================
// C Accessors for Phase 4: win_comp_scroll / win_new_screensize / etc.
// =============================================================================

/// Set w_height field (inner height).
void nvim_win_set_field_height(win_T *wp, int val) { if (wp) { wp->w_height = val; } }

/// Set w_width field (inner width).
void nvim_win_set_field_width(win_T *wp, int val) { if (wp) { wp->w_width = val; } }

/// Set scroll option script context to SID_WINLAYOUT (for win_comp_scroll).
void nvim_win_set_script_ctx_scroll(win_T *wp)
{
  if (wp) {
    wp->w_p_script_ctx[kWinOptScroll].sc_sid = SID_WINLAYOUT;
    wp->w_p_script_ctx[kWinOptScroll].sc_lnum = 0;
  }
}

/// Call win_reconfig_floats() (for win_new_screen_cols).
void nvim_win_reconfig_floats(void) { win_reconfig_floats(); }

/// Set w_last_topline snapshot field.
void nvim_win_set_last_topline(win_T *wp, linenr_T val) { if (wp) { wp->w_last_topline = val; } }

/// Set w_last_topfill snapshot field.
void nvim_win_set_last_topfill(win_T *wp, int val) { if (wp) { wp->w_last_topfill = val; } }

/// Set w_last_leftcol snapshot field.
void nvim_win_set_last_leftcol(win_T *wp, colnr_T val) { if (wp) { wp->w_last_leftcol = val; } }

/// Set w_last_skipcol snapshot field.
void nvim_win_set_last_skipcol(win_T *wp, colnr_T val) { if (wp) { wp->w_last_skipcol = val; } }

/// Set w_last_width snapshot field.
void nvim_win_set_last_width(win_T *wp, int val) { if (wp) { wp->w_last_width = val; } }

/// Set w_last_height snapshot field.
void nvim_win_set_last_height(win_T *wp, int val) { if (wp) { wp->w_last_height = val; } }

// Phase 3 accessors: win_fix_scroll, win_fix_cursor, may_make_initial_scroll_size_snapshot
int nvim_get_skip_win_fix_cursor(void) { return skip_win_fix_cursor ? 1 : 0; }
int nvim_win_get_do_win_fix_cursor(win_T *wp) { return wp ? (wp->w_do_win_fix_cursor ? 1 : 0) : 0; }
void nvim_win_set_do_win_fix_cursor(win_T *wp, int val) { if (wp) { wp->w_do_win_fix_cursor = val != 0; } }
int nvim_win_get_prev_winrow(win_T *wp) { return wp ? wp->w_prev_winrow : 0; }
void nvim_win_set_prev_winrow(win_T *wp, int val) { if (wp) { wp->w_prev_winrow = val; } }

// Phase 4 accessors: win_new_screen_rows, unuse_tabpage, use_tabpage, win_goto,
// restore_snapshot, do_autocmd_winclosed, can_close_in_cmdwin, set_winbar_win, set_winbar
int nvim_get_skip_win_fix_scroll(void) { return skip_win_fix_scroll ? 1 : 0; }
void nvim_set_curtab(tabpage_T *tp) { curtab = tp; }
void nvim_set_topframe(frame_T *fr) { topframe = fr; }
void nvim_tabpage_set_topframe(tabpage_T *tp, frame_T *fr) { if (tp) { tp->tp_topframe = fr; } }
void nvim_tabpage_set_ch_used(tabpage_T *tp, int64_t val) { if (tp) { tp->tp_ch_used = val; } }
void nvim_compute_cmdrow(void) { compute_cmdrow(); }
int nvim_has_event_winclosed(void) { return has_event(EVENT_WINCLOSED) ? 1 : 0; }
/// Apply WinClosed autocmd for window wp (wrapper for Rust do_autocmd_winclosed).
void nvim_apply_autocmds_winclosed(win_T *win)
{
  char winid[NUMBUFLEN];
  vim_snprintf(winid, sizeof(winid), "%d", win->handle);
  apply_autocmds(EVENT_WINCLOSED, winid, winid, false, win->w_buffer);
}
win_T *nvim_get_cmdwin_win(void) { return cmdwin_win; }
win_T *nvim_get_cmdwin_old_curwin(void) { return cmdwin_old_curwin; }
/// Check if win can close in cmdwin context.
/// Returns 0=OK, 1=set cmdwin_result (Ctrl_C), 2=api_set_error called.
int nvim_can_close_in_cmdwin_check(win_T *win, Error *err)
{
  if (cmdwin_type != 0) {
    if (win == cmdwin_win) {
      cmdwin_result = Ctrl_C;
      return 1;
    } else if (win == cmdwin_old_curwin) {
      api_set_error(err, kErrorTypeException, "%s", e_cmdwin);
      return 2;
    }
  }
  return 0;
}
void nvim_win_set_winbar_height(win_T *wp, int val) { if (wp) { wp->w_winbar_height = val; } }
void nvim_win_set_inner_size_wrapper(win_T *wp, int valid_cursor) { win_set_inner_size(wp, valid_cursor != 0); }
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

/// Apply BufLeave autocmd for current buffer.
void nvim_apply_autocmds_bufleave(void) { apply_autocmds(EVENT_BUFLEAVE, NULL, NULL, false, curbuf); }
/// Apply WinLeave autocmd for current buffer.
void nvim_apply_autocmds_winleave(void) { apply_autocmds(EVENT_WINLEAVE, NULL, NULL, false, curbuf); }
/// Apply WinNew autocmd for current buffer.
void nvim_apply_autocmds_winnew(void) { apply_autocmds(EVENT_WINNEW, NULL, NULL, false, curbuf); }
/// Apply WinEnter autocmd for current buffer.
void nvim_apply_autocmds_winenter(void) { apply_autocmds(EVENT_WINENTER, NULL, NULL, false, curbuf); }
/// Apply BufEnter autocmd for current buffer.
void nvim_apply_autocmds_bufenter(void) { apply_autocmds(EVENT_BUFENTER, NULL, NULL, false, curbuf); }

/// Set prevwin global.
void nvim_set_prevwin(win_T *wp) { prevwin = wp; }

/// Update topline for curwin (used in win_enter_ext).
void nvim_update_topline_curwin_enter(void) { update_topline(curwin); }

/// Copy buffer options on enter: buf_copy_options(buf, BCO_ENTER | BCO_NOHELP).
void nvim_buf_copy_options_enter(buf_T *buf) { buf_copy_options(buf, BCO_ENTER | BCO_NOHELP); }

/// Call changed_line_abv_curs().
void nvim_changed_line_abv_curs_wrap(void) { changed_line_abv_curs(); }

/// Call do_autochdir().
void nvim_do_autochdir_wrap(void) { do_autochdir(); }

/// Get restart_edit global (non-zero if in restart-edit mode).
int nvim_get_restart_edit_bool(void) { return restart_edit ? 1 : 0; }

/// Get w_hl_attr_normal field.
int nvim_win_get_hl_attr_normal_wrap(win_T *wp) { return wp ? wp->w_hl_attr_normal : 0; }
/// Get w_hl_attr_normalnc field.
int nvim_win_get_hl_attr_normalnc_wrap(win_T *wp) { return wp ? wp->w_hl_attr_normalnc : 0; }

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
/// get curtab.
tabpage_T *nvim_get_curtab_ptr(void) { return curtab; }
/// goto_tabpage_win wrapper.
void nvim_goto_tabpage_win_wrapper(tabpage_T *tp, win_T *wp) { goto_tabpage_win(tp, wp); }
/// swb_flags & kOptSwbFlagUseopen.
int nvim_swb_has_useopen(void) { return (swb_flags & kOptSwbFlagUseopen) ? 1 : 0; }
/// swb_flags & kOptSwbFlagUsetab.
int nvim_swb_has_usetab(void) { return (swb_flags & kOptSwbFlagUsetab) ? 1 : 0; }
/// win_fix_current_dir thin wrapper.
extern void rs_win_fix_current_dir(void);
void win_fix_current_dir(void) { rs_win_fix_current_dir(); }

/// buf_jump_open_win thin wrapper.
extern win_T *rs_buf_jump_open_win(buf_T *buf);
win_T *buf_jump_open_win(buf_T *buf) { return rs_buf_jump_open_win(buf); }

/// buf_jump_open_tab thin wrapper.
extern win_T *rs_buf_jump_open_tab(buf_T *buf);
win_T *buf_jump_open_tab(buf_T *buf) { return rs_buf_jump_open_tab(buf); }

/// swbuf_goto_win_with_buf thin wrapper.
extern win_T *rs_swbuf_goto_win_with_buf(buf_T *buf);
win_T *swbuf_goto_win_with_buf(buf_T *buf) { return rs_swbuf_goto_win_with_buf(buf); }

// =============================================================================
// Phase 3 accessors: command_height migration
// =============================================================================

/// Set p_ch directly (for restoring after e_noroom).
void nvim_set_p_ch(int64_t val) { p_ch = val; }

/// Get command_frame_height static.
int nvim_get_command_frame_height(void) { return command_frame_height ? 1 : 0; }
/// Set command_frame_height static.
void nvim_set_command_frame_height(int val) { command_frame_height = (val != 0); }

/// Get curtab->tp_ch_used as int.
int nvim_get_curtab_ch_used(void) { return curtab ? (int)curtab->tp_ch_used : 0; }
/// Set curtab->tp_ch_used from Rust.
void nvim_set_curtab_ch_used(int64_t val) { if (curtab) { curtab->tp_ch_used = val; } }

/// Set min_set_ch = val (the 'cmdheight' minimum).
void nvim_set_min_set_ch(int64_t val) { min_set_ch = val; }

/// Set cmdline_row = Rows - p_ch.
void nvim_update_cmdline_row(void) { cmdline_row = Rows - (int)p_ch; }

/// Get cmdline_row.
int nvim_get_cmdline_row_val(void) { return cmdline_row; }

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

/// command_height thin wrapper.
extern void rs_command_height(void);
void command_height(void) { rs_command_height(); }

// =============================================================================
// Phase 4: CTRL-W dispatch wrapper accessors
// =============================================================================

/// Return 1 if curbuf is locked (prevents split/new).
int nvim_curbuf_locked(void) { return curbuf_locked() ? 1 : 0; }

/// Error: E441 no preview window.
void nvim_emsg_e441_no_preview(void) { emsg(_("E441: There is no preview window")); }

/// Error: E23 no alternate file.
void nvim_emsg_noalt(void) { emsg(_(e_noalt)); }

/// Error: E92 buffer N not found.
void nvim_semsg_e92_buf_not_found(int64_t nr) { semsg(_("E92: Buffer %" PRId64 " not found"), nr); }

/// apply_autocmds for EVENT_TABNEWENTERED.
void nvim_apply_autocmds_tabnewentered(void)
{
  apply_autocmds(EVENT_TABNEWENTERED, NULL, NULL, false, curbuf);
}

/// win_new_tabpage wrapper (returns OK/FAIL as int).
int nvim_win_new_tabpage_wrapper(int after, const char *filename)
{
  return win_new_tabpage(after, (char *)filename);
}

/// get curwin->w_alt_fnum.
int nvim_win_get_alt_fnum(win_T *wp) { return wp ? wp->w_alt_fnum : 0; }

// Phase 4 Rust thin wrappers:

extern void rs_do_window_wW(int nchar, int prenum);
void nvim_do_window_wW(int nchar, int Prenum) { rs_do_window_wW(nchar, Prenum); }

extern void rs_do_window_P(void);
void nvim_do_window_P(void) { rs_do_window_P(); }

extern void rs_do_window_T(int prenum);
void nvim_do_window_T(int Prenum) { rs_do_window_T(Prenum); }

extern void rs_do_window_hat(int prenum);
void nvim_do_window_hat(int Prenum) { rs_do_window_hat(Prenum); }

extern void rs_do_window_new(int nchar, int prenum);
void nvim_do_window_new(int nchar, int Prenum) { rs_do_window_new(nchar, Prenum); }

extern void rs_cmd_with_count_exec(const char *cmd, int64_t prenum);
void nvim_cmd_with_count_exec(const char *cmd, int64_t Prenum) { rs_cmd_with_count_exec(cmd, Prenum); }

// Phase 2 accessors: tabpage helpers and check_split_disallowed_err migration

/// free_tabpage wrapper for Rust.
void nvim_free_tabpage_wrapper(tabpage_T *tp)
{
  free_tabpage(tp);
}

/// Get p_tpm (tabpagemax option).
int64_t nvim_get_p_tpm(void) { return p_tpm; }

/// Set lastused_tabpage global.
void nvim_set_lastused_tabpage(tabpage_T *tp) { lastused_tabpage = tp; }

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
