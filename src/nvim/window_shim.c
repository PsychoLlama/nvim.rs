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
extern size_t rs_find_ident_under_cursor(char **text, int find_type);

// Phase 9: win_init migration
extern void rs_win_init(win_T *newp, win_T *oldp, int flags);

// Phase 9: may_trigger_win_scrolled_resized migration
extern void rs_may_trigger_win_scrolled_resized(void);

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

// Phase 7: leave_tabpage, enter_tabpage, goto_tabpage_tp
extern int rs_leave_tabpage(buf_T *new_curbuf, int trigger_leave);
extern void rs_enter_tabpage(tabpage_T *tp, buf_T *old_curbuf, int trigger_enter, int trigger_leave);
extern void rs_goto_tabpage_tp_impl(tabpage_T *tp, int trigger_enter, int trigger_leave);

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

// win_close / win_close_othertab: consolidated Rust entry points (Phase 11)
extern int rs_win_close(win_T *win, int free_buf, int force);
extern int rs_win_close_othertab(win_T *win, int free_buf, tabpage_T *tp, int force);
// rs_win_free_mem still used by win_free_all
extern win_T *rs_win_free_mem(win_T *win, int *dirp, tabpage_T *tp);

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
int nvim_win_get_redr_status(win_T *wp) { return wp ? wp->w_redr_status : 0; }
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
// nvim_set_cmdheight_option deleted: logic migrated to Rust resize/frame.rs (Phase 8)

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
  rs_win_init(newp, oldp, flags);
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

// can_close_floating_windows deleted: logic migrated to Rust close/win_close.rs (Phase 11)

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
// close_last_window_tabpage deleted: logic migrated to Rust close/helpers.rs (Phase 10)

// win_close_buffer deleted: logic migrated to Rust close/helpers.rs (Phase 10)

// Close window "win".  Only works for the current tab page.
// If "free_buf" is true related buffer may be unloaded.
//
// Called by :quit, :close, :xit, :wq and findtag().
// Returns FAIL when the window was not closed.
// Body migrated to Rust rs_win_close (Phase 11).
int win_close(win_T *win, bool free_buf, bool force)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_win_close(win, free_buf ? 1 : 0, force ? 1 : 0);
}

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

#if defined(EXITFREE)
void win_free_all(void)
{
  rs_win_free_all();
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
/// Store the relevant window pointers for tab page "tp".  To be used before
/// use_tabpage().
void unuse_tabpage(tabpage_T *tp) { rs_unuse_tabpage(tp); }

// When switching tabpage, handle other side-effects in command_height(), but
// avoid setting frame sizes which are still correct.
static bool command_frame_height = true;

/// Set the relevant pointers to use tab page "tp".  May want to call
/// unuse_tabpage() first.
void use_tabpage(tabpage_T *tp) { rs_use_tabpage(tp); }

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
void nvim_curwin_init(void) { curwin_init(); }

/// RESET_BINDING for window wp.
void nvim_win_reset_binding(win_T *wp) { RESET_BINDING(wp); }

/// Set topframe from wp->w_frame and compute frame dimensions.
void nvim_alloc_firstwin_set_topframe(win_T *wp)
{
  topframe = wp->w_frame;
  topframe->fr_width = Columns;
  topframe->fr_height = Rows - (int)p_ch - rs_global_stl_height();
}

/// Set curwin = wp.
void nvim_set_curwin_to_wp(win_T *wp) { curwin = wp; }

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

/// curwin = NULL.
void nvim_set_curwin_null(void) { curwin = NULL; }

extern void rs_win_alloc_first(void);
extern int rs_win_alloc_firstwin(win_T *oldwin);
extern void rs_win_alloc_aucmd_win(int idx);
extern void rs_win_free_all(void);

// Allocate the first window and put an empty buffer in it.
// Only called from main().
void win_alloc_first(void)
{
  rs_win_alloc_first();
}

// Init `aucmd_win[idx]`. This can only be done after the first window
// is fully initialized, thus it can't be in win_alloc_first().
void win_alloc_aucmd_win(int idx)
{
  rs_win_alloc_aucmd_win(idx);
}

// Allocate the first window or the first window in a new tab page.
// When "oldwin" is NULL create an empty buffer for it.
// When "oldwin" is not NULL copy info from it to the new window.
// Return FAIL when something goes wrong (out of memory).
static int win_alloc_firstwin(win_T *oldwin)
{
  return rs_win_alloc_firstwin(oldwin);
}

// Initialize the window and frame size to the maximum.
void win_init_size(void) { rs_win_init_size(); }

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

/// Free raw tabpage struct (xfree only — does not clear fields).
void nvim_xfree_tabpage_raw(tabpage_T *tp) { xfree(tp); }

/// Get lastused_tabpage.
tabpage_T *nvim_get_lastused_tabpage_raw(void) { return lastused_tabpage; }

/// Set lastused_tabpage = NULL.
void nvim_set_lastused_tabpage_null(void) { lastused_tabpage = NULL; }

/// Allocate a raw frame_T (xcalloc only).
frame_T *nvim_alloc_frame_raw(void) { return xcalloc(1, sizeof(frame_T)); }

// =============================================================================
// alloc_tabpage: thin wrapper calling rs_alloc_tabpage
// =============================================================================

extern tabpage_T *rs_alloc_tabpage(void);

/// Allocate a new tabpage_T and init the values.
static tabpage_T *alloc_tabpage(void)
{
  return rs_alloc_tabpage();
}

// =============================================================================
// free_tabpage: thin wrapper calling rs_free_tabpage
// =============================================================================

extern void rs_free_tabpage(tabpage_T *tp);

void free_tabpage(tabpage_T *tp)
{
  rs_free_tabpage(tp);
}

// =============================================================================
// new_frame: thin wrapper calling rs_new_frame
// =============================================================================

extern void rs_new_frame(win_T *wp);

// Create a frame for window "wp".
static void new_frame(win_T *wp)
{
  rs_new_frame(wp);
}

// =============================================================================
// Phase 8 wrappers for rs_win_new_tabpage
// =============================================================================

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

// =============================================================================
// win_new_tabpage: body replaced by thin wrapper calling rs_win_new_tabpage
// =============================================================================

extern int rs_win_new_tabpage(int after, const char *filename);

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
  return rs_win_new_tabpage(after, filename);
}

// Open a new tab page if ":tab cmd" was used.  It will edit the same buffer,
// like with ":split".
// Returns OK if a new tab page was created, FAIL otherwise.
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
  return rs_leave_tabpage(new_curbuf, trigger_leave_autocmds ? 1 : 0);
}

/// Start using tab page "tp".
/// Only to be used after leave_tabpage() or freeing the current tab page.
///
/// @param trigger_enter_autocmds  when true trigger *Enter autocommands.
/// @param trigger_leave_autocmds  when true trigger *Leave autocommands.
static void enter_tabpage(tabpage_T *tp, buf_T *old_curbuf, bool trigger_enter_autocmds,
                          bool trigger_leave_autocmds)
{
  rs_enter_tabpage(tp, old_curbuf, trigger_enter_autocmds ? 1 : 0,
                   trigger_leave_autocmds ? 1 : 0);
}

/// tells external UI that windows and inline floats in old_curtab are invisible
/// and that floats in curtab is now visible.
///
/// External floats are considered independent of tabpages. This is
/// implemented by always moving them to curtab.
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
  rs_goto_tabpage_tp_impl(tp, trigger_enter_autocmds ? 1 : 0, trigger_leave_autocmds ? 1 : 0);
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

extern win_T *rs_win_alloc(win_T *after, int hidden);

/// @param hidden  allocate a window structure and link it in the window if
//                 false.
win_T *win_alloc(win_T *after, bool hidden)
{
  return rs_win_alloc(after, hidden ? 1 : 0);
}

// =============================================================================
// free_wininfo: thin wrapper calling rs_free_wininfo
// =============================================================================

extern void rs_free_wininfo(WinInfo *wip, buf_T *bp);

// Free one WinInfo.
void free_wininfo(WinInfo *wip, buf_T *bp)
{
  rs_free_wininfo(wip, bp);
}

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

extern void rs_win_free(win_T *wp, tabpage_T *tp);
extern void rs_win_free_grid(win_T *wp, int reinit);

/// Remove window 'wp' from the window list and free the structure.
///
/// @param tp  tab page "win" is in, NULL for current
void win_free(win_T *wp, tabpage_T *tp)
{
  rs_win_free(wp, tp);
}

void win_free_grid(win_T *wp, bool reinit)
{
  rs_win_free_grid(wp, reinit ? 1 : 0);
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

/// Trigger WinScrolled and/or WinResized if any window in the current tab page
/// scrolled or changed size.
void may_trigger_win_scrolled_resized(void)
{
  rs_may_trigger_win_scrolled_resized();
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
void nvim_ui_call_grid_destroy_handle(int handle) { ui_call_grid_destroy(handle); }
void nvim_clear_matches_win(win_T *wp) { clear_matches(wp); }
void nvim_free_jumplist_win(win_T *wp) { free_jumplist(wp); }
win_T *nvim_get_au_pending_free_win(void) { return au_pending_free_win; }
void nvim_set_au_pending_free_win(win_T *wp) { au_pending_free_win = wp; }
void nvim_xfree_win_raw(win_T *wp) { xfree(wp); }
void nvim_ui_call_win_viewport_margins_wrapper(win_T *wp) {
  if (wp && ui_has(kUIMultigrid)) {
    ui_call_win_viewport_margins(wp->w_grid_alloc.handle, wp->handle,
                                 wp->w_winrow_off, wp->w_border_adj[2],
                                 wp->w_wincol_off, wp->w_border_adj[1]);
  }
}

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
// nvim_win_alloc_wrapper deleted: callers updated to call rs_win_alloc directly (Phase 12)
// nvim_new_frame_wrapper deleted: callers updated to call rs_new_frame directly (Phase 12)
void nvim_win_init_wrapper(win_T *wp, win_T *oldwin, int flags) { win_init(wp, oldwin, flags); }
void nvim_frame_flatten_wrapper(frame_T *frp) { rs_frame_flatten(frp); }
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

void nvim_redraw_later_wrapper(win_T *wp, int type) { if (wp) { redraw_later(wp, type); } }
void nvim_status_redraw_all_wrapper(void) { status_redraw_all(); }
void nvim_msg_clr_eos_force(void) { msg_clr_eos_force(); }
int nvim_is_aucmd_win(win_T *wp) { return is_aucmd_win(wp) ? 1 : 0; }
int nvim_win_get_config_external_int(win_T *wp) { return wp ? (int)wp->w_config.external : 0; }

// nvim_fixup_external_curwin deleted: logic migrated to Rust win_close.rs (Phase 8)

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
// nvim_win_free_mem_wrapper deleted: rs_win_close_structural now calls rs_win_free_mem directly (Phase 10)
void nvim_inc_split_disallowed(void) { split_disallowed++; }
void nvim_dec_split_disallowed(void) { split_disallowed--; }
void nvim_ui_call_win_close_win(win_T *wp) { if (wp) { ui_call_win_close(wp->w_grid_alloc.handle); } }
void nvim_tabpage_set_curwin(tabpage_T *tp, win_T *wp) { tp->tp_curwin = wp; }
void nvim_set_curbuf_from_curwin(void) { curbuf = curwin->w_buffer; }
void nvim_check_cursor_win_wrapper(win_T *wp) { check_cursor(wp); }

/// Get w_frame->fr_parent from a window (for win_close frame comparison).
frame_T *nvim_win_get_frame_parent(win_T *wp) { return (wp && wp->w_frame) ? wp->w_frame->fr_parent : NULL; }

buf_T *nvim_get_firstbuf_wrapper(void) { return firstbuf; }
int nvim_can_close_floating_windows(tabpage_T *tp) { return rs_can_close_floating_windows_tp(tp) != 0 ? 1 : 0; }
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
void nvim_setpcmark_curwin(void) { setpcmark(); }

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
void nvim_check_cursor_lnum_curwin(void) { check_cursor_lnum(curwin); }

/// beginline(BL_SOL | BL_FIX) wrapper.
void nvim_beginline_sol_fix(void) { beginline(BL_SOL | BL_FIX); }

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
int nvim_get_p_pvh(void) { return (int)p_pvh; }
void nvim_do_nv_ident(int prefix, int xchar) { do_nv_ident(prefix, xchar); }
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
/// Now delegates directly to rs_win_new_tabpage (Phase 8).
int nvim_win_new_tabpage_wrapper(int after, const char *filename)
{
  return rs_win_new_tabpage(after, filename);
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

/// Emit E445 "Other window contains changes" error.
void nvim_emsg_e445(void) { emsg(_("E445: Other window contains changes")); }

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

/// Call win_close(wp, free_buf, false) from Rust.
int nvim_win_close_wrapper(win_T *wp, int free_buf)
{
  return win_close(wp, free_buf != 0, false);
}

extern void rs_close_others(int message, int forceit);
void close_others(int message, int forceit) { rs_close_others(message, forceit); }

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

/// Wrap win_close_othertab(wp, free_buf, tp, force) returning int (0=FAIL, 1=OK).
int nvim_win_close_othertab_wrapper(win_T *wp, int free_buf, tabpage_T *tp, int force)
{
  return win_close_othertab(wp, free_buf != 0, tp, force != 0) ? 1 : 0;
}

extern void rs_close_windows(buf_T *buf, int keep_curwin);
void close_windows(buf_T *buf, bool keep_curwin) { rs_close_windows(buf, keep_curwin ? 1 : 0); }

// Phase 6 accessors: ui_ext_win_viewport

linenr_T nvim_win_get_viewport_last_topline(win_T *wp) { return wp ? wp->w_viewport_last_topline : 0; }
void nvim_win_set_viewport_last_topline(win_T *wp, int32_t val) { if (wp) { wp->w_viewport_last_topline = (linenr_T)val; } }
linenr_T nvim_win_get_viewport_last_botline(win_T *wp) { return wp ? wp->w_viewport_last_botline : 0; }
void nvim_win_set_viewport_last_botline(win_T *wp, int32_t val) { if (wp) { wp->w_viewport_last_botline = (linenr_T)val; } }
int nvim_win_get_viewport_last_topfill(win_T *wp) { return wp ? wp->w_viewport_last_topfill : 0; }
void nvim_win_set_viewport_last_topfill(win_T *wp, int32_t val) { if (wp) { wp->w_viewport_last_topfill = (linenr_T)val; } }
int64_t nvim_win_get_viewport_last_skipcol(win_T *wp) { return wp ? (int64_t)wp->w_viewport_last_skipcol : 0; }
void nvim_win_set_viewport_last_skipcol(win_T *wp, int64_t val) { if (wp) { wp->w_viewport_last_skipcol = (linenr_T)val; } }

/// Wrap ui_call_win_viewport for Rust.
void nvim_ui_call_win_viewport_wrapper(int grid, int win, int topline, int botline,
                                       int curline, int curcol, int line_count, int64_t delta)
{
  ui_call_win_viewport(grid, win, topline, botline, curline, curcol, line_count, delta);
}

extern void rs_ui_ext_win_viewport(win_T *wp);
void ui_ext_win_viewport(win_T *wp) { rs_ui_ext_win_viewport(wp); }

// Phase 6 accessors: tabpage_check_windows + win_ui_flush

/// Wrap pum_ui_flush() for Rust.
void nvim_pum_ui_flush_wrapper(void) { pum_ui_flush(); }

/// Wrap msg_ui_flush() for Rust.
void nvim_msg_ui_flush_wrapper(void) { msg_ui_flush(); }

/// Get wp->w_grid_alloc.pending_comp_index_update.
int nvim_win_get_grid_pending_comp(win_T *wp)
{
  return (wp && wp->w_grid_alloc.pending_comp_index_update) ? 1 : 0;
}

/// Set wp->w_grid_alloc.pending_comp_index_update.
void nvim_win_set_grid_pending_comp(win_T *wp, int val)
{
  if (wp) {
    wp->w_grid_alloc.pending_comp_index_update = (val != 0);
  }
}

/// Check if wp->w_grid_alloc.chars != NULL.
int nvim_win_get_grid_chars_valid(win_T *wp)
{
  return (wp && wp->w_grid_alloc.chars != NULL) ? 1 : 0;
}

extern void rs_tabpage_check_windows(tabpage_T *old_curtab);
static void tabpage_check_windows(tabpage_T *old_curtab) { rs_tabpage_check_windows(old_curtab); }

extern void rs_win_ui_flush(int validate);
void win_ui_flush(bool validate) { rs_win_ui_flush(validate ? 1 : 0); }

// Phase 6 accessors: may_open_tabpage

/// Get postponed_split_tab global.
int nvim_get_postponed_split_tab(void) { return postponed_split_tab; }

/// Set postponed_split_tab global.
void nvim_set_postponed_split_tab(int val) { postponed_split_tab = val; }

/// Get cmdmod.cmod_tab.
int nvim_get_cmdmod_tab(void) { return (int)cmdmod.cmod_tab; }

/// Set cmdmod.cmod_tab.
void nvim_set_cmdmod_tab(int val) { cmdmod.cmod_tab = val; }

extern int rs_may_open_tabpage(void);
static int may_open_tabpage(void) { return rs_may_open_tabpage(); }

// =============================================================================
// Phase 7 accessors: leave_tabpage, enter_tabpage, goto_tabpage_tp
// =============================================================================

/// Set tp->tp_prevwin.
void nvim_tabpage_set_prevwin(tabpage_T *tp, win_T *wp) { if (tp) { tp->tp_prevwin = wp; } }

/// Get tp->tp_prevwin.
win_T *nvim_tabpage_get_prevwin(tabpage_T *tp) { return tp ? tp->tp_prevwin : NULL; }

/// Set tp->tp_old_Rows_avail.
void nvim_tabpage_set_old_rows_avail(tabpage_T *tp, int val) { if (tp) { tp->tp_old_Rows_avail = val; } }

/// Get tp->tp_old_Rows_avail.
int nvim_tabpage_get_old_rows_avail(tabpage_T *tp) { return tp ? tp->tp_old_Rows_avail : 0; }

/// Get tp->tp_old_Columns.
int nvim_tabpage_get_old_columns(tabpage_T *tp) { return tp ? tp->tp_old_Columns : 0; }

/// Set tp->tp_old_Columns.
void nvim_tabpage_set_old_columns(tabpage_T *tp, int val) { if (tp) { tp->tp_old_Columns = val; } }

/// Call reset_dragwin().
void nvim_reset_dragwin(void) { reset_dragwin(); }

/// Fire EVENT_TABLEAVE autocmd.
void nvim_apply_autocmds_tableave(void) { apply_autocmds(EVENT_TABLEAVE, NULL, NULL, false, curbuf); }

/// Fire EVENT_TABENTER autocmd.
void nvim_apply_autocmds_tabenter(void) { apply_autocmds(EVENT_TABENTER, NULL, NULL, false, curbuf); }

/// Set firstwin = NULL (without syncing curtab->tp_firstwin).
void nvim_set_firstwin_null(void) { firstwin = NULL; }

/// Set lastwin = NULL (without syncing curtab->tp_lastwin).
void nvim_set_lastwin_null(void) { lastwin = NULL; }

/// Get the `starting` global (nonzero while Vim is starting up).
int nvim_get_starting(void) { return starting; }

/// Call win_float_update_statusline().
void nvim_win_float_update_statusline(void) { win_float_update_statusline(); }

/// Set lastused_tabpage global (for enter_tabpage migration).
void nvim_set_lastused_tabpage_from_rust(tabpage_T *tp) { lastused_tabpage = tp; }

/// Call set_keep_msg(NULL, 0).
void nvim_set_keep_msg_null(void) { set_keep_msg(NULL, 0); }

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

/// Get w_winrow from tp->tp_firstwin (for enter_tabpage old_off comparison).
int nvim_tabpage_get_firstwin_winrow(tabpage_T *tp)
{
  return (tp && tp->tp_firstwin) ? tp->tp_firstwin->w_winrow : 0;
}

// =============================================================================
// Phase 9 accessors: win_init migration
// =============================================================================

/// Copy buffer link: dst->w_buffer = src->w_buffer; dst->w_s = &src->w_buffer->b_s;
/// src->w_buffer->b_nwindows++.
void nvim_win_copy_buffer_link(win_T *dst, win_T *src)
{
  if (!dst || !src || !src->w_buffer) {
    return;
  }
  dst->w_buffer = src->w_buffer;
  dst->w_s = &src->w_buffer->b_s;
  src->w_buffer->b_nwindows++;
}

/// Copy pcmarks: dst->w_pcmark = src->w_pcmark, dst->w_prev_pcmark = src->w_prev_pcmark.
void nvim_win_copy_pcmarks(win_T *dst, win_T *src)
{
  if (!dst || !src) {
    return;
  }
  dst->w_pcmark = src->w_pcmark;
  dst->w_prev_pcmark = src->w_prev_pcmark;
}

/// Set w_alt_fnum.
void nvim_win_set_alt_fnum(win_T *wp, int val)
{
  if (wp) {
    wp->w_alt_fnum = val;
  }
}

/// Wrap copy_jumplist(old, new) for Rust.
void nvim_copy_jumplist_wrapper(win_T *old, win_T *new)
{
  if (old && new) {
    copy_jumplist(old, new);
  }
}

/// Wrap copy_loclist_stack(old, new) for Rust.
void nvim_copy_loclist_stack_wrapper(win_T *old, win_T *new)
{
  if (old && new) {
    copy_loclist_stack(old, new);
  }
}

/// Set w_llist = NULL, w_llist_ref = NULL (skip location list copy).
void nvim_win_clear_loclist(win_T *wp)
{
  if (wp) {
    wp->w_llist = NULL;
    wp->w_llist_ref = NULL;
  }
}

/// Copy local directory strings with xstrdup (handles NULL).
void nvim_win_copy_localdir(win_T *dst, win_T *src)
{
  if (!dst || !src) {
    return;
  }
  dst->w_localdir = (src->w_localdir == NULL) ? NULL : xstrdup(src->w_localdir);
  dst->w_prevdir = (src->w_prevdir == NULL) ? NULL : xstrdup(src->w_prevdir);
}

/// Copy entire tagstack from src to dst, with xstrdup of tagname/user_data strings.
void nvim_win_copy_tagstack(win_T *dst, win_T *src)
{
  if (!dst || !src) {
    return;
  }
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
}

/// Get w_changelistidx.
int nvim_win_get_changelistidx(win_T *wp) { return wp ? wp->w_changelistidx : 0; }

/// Set w_changelistidx.
void nvim_win_set_changelistidx(win_T *wp, int val) { if (wp) { wp->w_changelistidx = val; } }

/// Wrap rs_copyFoldingState(old, new) callable from Rust.
void nvim_copy_folding_state_wrapper(win_T *old, win_T *new)
{
  if (old && new) {
    rs_copyFoldingState(old, new);
  }
}

/// Copy alist: dst->w_alist = src->w_alist; al_refcount++; dst->w_arg_idx = src->w_arg_idx.
void nvim_win_copy_alist(win_T *dst, win_T *src)
{
  if (!dst || !src || !src->w_alist) {
    return;
  }
  dst->w_alist = src->w_alist;
  dst->w_alist->al_refcount++;
  dst->w_arg_idx = src->w_arg_idx;
}

/// Wrap win_copy_options(old, new) for Rust.
void nvim_win_copy_options_wrapper(win_T *old, win_T *new)
{
  if (old && new) {
    win_copy_options(old, new);
  }
}

// =============================================================================
// Phase 9 accessors: may_trigger_win_scrolled_resized migration
// =============================================================================

// Window snapshot field getters
linenr_T nvim_win_get_last_topline(win_T *wp) { return wp ? wp->w_last_topline : 0; }
int nvim_win_get_last_topfill(win_T *wp) { return wp ? wp->w_last_topfill : 0; }
colnr_T nvim_win_get_last_leftcol(win_T *wp) { return wp ? wp->w_last_leftcol : 0; }
colnr_T nvim_win_get_last_skipcol(win_T *wp) { return wp ? wp->w_last_skipcol : 0; }
int nvim_win_get_last_width(win_T *wp) { return wp ? wp->w_last_width : 0; }
int nvim_win_get_last_height(win_T *wp) { return wp ? wp->w_last_height : 0; }

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
void nvim_win_set_locked(win_T *wp, int val) { if (wp) { wp->w_locked = (val != 0); } }

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

/// xfree wrapper for frame_T* (frees the frame pointer).
void nvim_xfree_frame(void *frp) { xfree(frp); }

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

/// E444: Cannot close last window.
void nvim_emsg_e444(void) { emsg(_("E444: Cannot close last window")); }

/// E814: Cannot close window, only autocmd window would remain.
void nvim_emsg_e814(void) { emsg(_("E814: Cannot close window, only autocmd window would remain")); }

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

/// Get the w_lines_valid field from a window.
int nvim_win_get_w_lines_valid(win_T *wp)
{
  return wp->w_lines_valid;
}

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

/// Get the w_p_scb (scrollbind) field from a window.
bool nvim_win_get_p_scb(win_T *wp)
{
  return wp->w_p_scb;
}

