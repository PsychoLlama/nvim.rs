#include <inttypes.h>

#include "klib/kvec.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/arglist.h"
#include "nvim/autocmd.h"
#include "nvim/buffer.h"
#include "nvim/charset.h"
#include "nvim/cursor.h"
#include "nvim/decoration.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/vars.h"
#include "nvim/eval/window.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds2.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_getln.h"
#include "nvim/file_search.h"
#include "nvim/fold.h"
#include "nvim/garray.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/highlight.h"
#include "nvim/mark.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/main.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/os/fs.h"
#include "nvim/quickfix.h"
#include "nvim/regexp.h"
#include "nvim/search.h"
#include "nvim/statusline.h"
#include "nvim/terminal.h"
#include "nvim/ui.h"
#include "nvim/ui_compositor.h"
#include "nvim/undo.h"
#include "nvim/window.h"
#include "nvim/winfloat.h"

#include "window_shim.c.generated.h"

_Static_assert(sizeof(WinSnapshot) == 6 * sizeof(int), "WinSnapshot size mismatch");
_Static_assert(sizeof(WinViewportSnapshot) == 4 * sizeof(int32_t), "WinViewportSnapshot size mismatch");

extern void rs_tagstack_clear_entry(void *tg);
extern size_t rs_find_ident_under_cursor(char **text, int find_type);
extern void rs_copyFoldingState(win_T *wp_from, win_T *wp_to);
extern int rs_tabline_height(void);
extern int rs_global_stl_height(void);
extern int rs_tabpage_index(tabpage_T *ftp);
extern tabpage_T *rs_win_find_tabpage(win_T *win);
extern int rs_win_close_othertab(win_T *win, int free_buf, tabpage_T *tp, int force);

int *nvim_win_get_ns_hl_attr(win_T *wp) { return wp->w_ns_hl_attr; }
int nvim_win_get_p_winbl(win_T *wp) { return (int)wp->w_p_winbl; }
ScreenGrid *nvim_get_curwin_grid_alloc(void) { return curwin ? &curwin->w_grid_alloc : NULL; }
void nvim_get_curwin_cursor_pos(void *pos) { int32_t *p = (int32_t *)pos; p[0] = (int32_t)curwin->w_cursor.lnum; p[1] = (int32_t)curwin->w_cursor.col; p[2] = (int32_t)curwin->w_cursor.coladd; }
void nvim_save_viewstate(void *vs) { int32_t *p = (int32_t *)vs; p[0] = (int32_t)curwin->w_curswant; p[1] = (int32_t)curwin->w_leftcol; p[2] = (int32_t)curwin->w_skipcol; p[3] = (int32_t)curwin->w_topline; p[4] = (int32_t)curwin->w_topfill; p[5] = (int32_t)curwin->w_botline; p[6] = (int32_t)curwin->w_empty_rows; }
void nvim_restore_viewstate(const void *vs) { const int32_t *p = (const int32_t *)vs; curwin->w_curswant = (colnr_T)p[0]; curwin->w_leftcol = (colnr_T)p[1]; curwin->w_skipcol = (colnr_T)p[2]; curwin->w_topline = (linenr_T)p[3]; curwin->w_topfill = (int)p[4]; curwin->w_botline = (linenr_T)p[5]; curwin->w_empty_rows = (int)p[6]; }
void nvim_set_curwin_cursor_pos(const void *pos) { const int32_t *p = (const int32_t *)pos; curwin->w_cursor.lnum = (linenr_T)p[0]; curwin->w_cursor.col = (colnr_T)p[1]; curwin->w_cursor.coladd = (colnr_T)p[2]; }
int nvim_equalpos(const void *pos1, const void *pos2) { const int32_t *p1 = (const int32_t *)pos1; const int32_t *p2 = (const int32_t *)pos2; return p1[0] == p2[0] && p1[1] == p2[1] && p1[2] == p2[2]; }
char nvim_win_get_fdm_char(win_T *wp, int idx) { return wp->w_p_fdm[idx]; }
int nvim_win_buf_has_terminal(win_T *wp) { return wp->w_buffer->terminal != NULL; }
int nvim_win_folds_empty(win_T *wp) { return GA_EMPTY(&wp->w_folds); }
const char *nvim_win_get_w_p_fdc(win_T *wp) { return wp->w_p_fdc; }
int nvim_win_is_curwin(win_T *wp) { return wp == curwin; }
char *nvim_win_get_p_stc(win_T *wp) { return wp->w_p_stc; }
char *nvim_win_get_p_cocu(win_T *wp) { return wp->w_p_cocu; }
linenr_T nvim_win_buf_line_count(win_T *wp) { return wp->w_buffer->b_ml.ml_line_count; }
void nvim_win_config_float(win_T *wp) { win_config_float(wp, wp->w_config); }
int nvim_win_is_cmdwin(win_T *wp) { return wp == cmdwin_win; }
char *nvim_win_get_p_sbr(win_T *wp) { return wp->w_p_sbr; }
char *nvim_win_get_p_cc(win_T *wp) { return wp->w_p_cc; }
int64_t nvim_win_get_buf_b_p_tw(win_T *wp) { return wp->w_buffer->b_p_tw; }
int nvim_win_has_buffer(win_T *wp) { return wp->w_buffer != NULL; }
void nvim_win_set_p_cc_cols(win_T *wp, int *cols) { wp->w_p_cc_cols = cols; }
int nvim_first_tabpage_has_next(void) { return first_tabpage != NULL && first_tabpage->tp_next != NULL; }
int nvim_win_get_wrap_flags(win_T *wp) { return wp->w_p_wrap_flags; }
int nvim_win_get_p_culopt_flags(win_T *wp) { return wp->w_p_culopt_flags; }
int nvim_win_argcount(win_T *wp) { return WARGCOUNT(wp); }
colnr_T nvim_win_get_valid_leftcol(win_T *wp) { return wp->w_valid_leftcol; }
colnr_T nvim_win_get_valid_skipcol(win_T *wp) { return wp->w_valid_skipcol; }
int nvim_win_get_viewport_invalid(win_T *wp) { return wp->w_viewport_invalid ? 1 : 0; }
void *nvim_win_get_w_grid(win_T *wp) { return &wp->w_grid; }
ScreenGrid *nvim_win_get_w_grid_alloc(win_T *wp) { return wp ? &wp->w_grid_alloc : NULL; }
bool nvim_win_get_briopt_sbr(win_T *wp) { return wp->w_briopt_sbr; }
int nvim_win_hl_attr(win_T *wp, int hlf) { return win_hl_attr(wp, hlf); }
buf_T *nvim_win_get_buffer(win_T *wp) { return wp->w_buffer; }
void nvim_win_set_p_wfb(win_T *wp, int val) { wp->w_p_wfb = val != 0; }
const char *nvim_win_ml_get_buf(win_T *wp, linenr_T lnum) { return ml_get_buf(wp->w_buffer, lnum); }
colnr_T nvim_win_ml_get_buf_len(win_T *wp, linenr_T lnum) { return ml_get_buf_len(wp->w_buffer, lnum); }
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
static int split_disallowed = 0;
static OptInt min_set_ch = 1;
OptInt nvim_get_min_set_ch(void) { return min_set_ch; }

void win_set_buf(win_T *win, buf_T *buf, Error *err)
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

      do_buffer_ext(DOBUF_GOTO, DOBUF_FIRST, FORWARD, buf->b_fnum, 0);

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
void merge_win_config(WinConfig *dst, const WinConfig src)
{ if (dst->title_chunks.items != src.title_chunks.items) { clear_virttext(&dst->title_chunks); } if (dst->footer_chunks.items != src.footer_chunks.items) { clear_virttext(&dst->footer_chunks); } *dst = src; }
bool win_close_othertab(win_T *win, int free_buf, tabpage_T *tp, bool force)
{ return rs_win_close_othertab(win, free_buf, tp, force ? 1 : 0) != 0; }
static bool command_frame_height = true;

buf_T *nvim_buflist_new_initial(void) { return buflist_new(NULL, NULL, 1, BLN_LISTED); }
void nvim_win_setup_first_buffer(win_T *wp, buf_T *buf) { curwin = wp; curbuf = buf; wp->w_buffer = buf; wp->w_s = &(buf->b_s); buf->b_nwindows = 1; wp->w_alist = &global_alist; }
void nvim_win_reset_binding(win_T *wp) { RESET_BINDING(wp); }
void nvim_alloc_firstwin_set_topframe(win_T *wp) { topframe = wp->w_frame; topframe->fr_width = Columns; topframe->fr_height = Rows - (int)p_ch - rs_global_stl_height(); }
void nvim_win_alloc_aucmd_win_impl(int idx)
{ Error err = ERROR_INIT; WinConfig fconfig = WIN_CONFIG_INIT; fconfig.width = Columns; fconfig.height = 5; fconfig.focusable = false; fconfig.mouse = false; aucmd_win[idx].auc_win = win_new_float(NULL, true, fconfig, &err); aucmd_win[idx].auc_win->w_buffer->b_nwindows--; RESET_BINDING(aucmd_win[idx].auc_win); }
void nvim_clear_cmdwin_state(void) { cmdwin_type = 0; cmdwin_buf = NULL; cmdwin_win = NULL; cmdwin_old_curwin = NULL; }
void nvim_tabpage_close_once(void) { tabpage_close(true); }
int nvim_aucmd_win_count(void) { return AUCMD_WIN_COUNT; }
win_T *nvim_aucmd_win_get(int idx) { return aucmd_win[idx].auc_win; }
void nvim_aucmd_win_clear(int idx) { aucmd_win[idx].auc_win = NULL; }
void nvim_kv_destroy_aucmd_win_vec(void) { kv_destroy(aucmd_win_vec); }
tabpage_T *nvim_alloc_tabpage_raw(void) { return xcalloc(1, sizeof(tabpage_T)); }
void nvim_tabpage_init_handle(tabpage_T *tp) { static int last_tp_handle = 0; tp->handle = ++last_tp_handle; pmap_put(int)(&tabpage_handles, tp->handle, tp); }
void nvim_tabpage_init_vars(tabpage_T *tp) { tp->tp_vars = tv_dict_alloc(); init_var_dict(tp->tp_vars, &tp->tp_winvar, VAR_SCOPE); }
void nvim_tabpage_set_ch_used_from_p_ch(tabpage_T *tp) { tp->tp_ch_used = p_ch; }
void nvim_tabpage_pmap_del(tabpage_T *tp) { pmap_del(int)(&tabpage_handles, tp->handle, NULL); }
void nvim_tabpage_clear_vars(tabpage_T *tp) { vars_clear(&tp->tp_vars->dv_hashtab); hash_init(&tp->tp_vars->dv_hashtab); unref_var_dict(tp->tp_vars); }
frame_T *nvim_alloc_frame_raw(void) { return xcalloc(1, sizeof(frame_T)); }
void nvim_tabpage_copy_localdir(tabpage_T *dst, tabpage_T *src) { if (dst && src) { dst->tp_localdir = src->tp_localdir ? xstrdup(src->tp_localdir) : NULL; } }
void nvim_curbuf_terminal_check_size(void) { if (curbuf->terminal) { terminal_check_size(curbuf->terminal); } }
void nvim_apply_autocmds_tabnew(const char *filename) { apply_autocmds(EVENT_TABNEW, (char *)filename, (char *)filename, false, curbuf); }
void win_enter(win_T *wp, bool undo_sync) { win_enter_ext(wp, (undo_sync ? WEE_UNDO_SYNC : 0) | WEE_TRIGGER_ENTER_AUTOCMDS | WEE_TRIGGER_LEAVE_AUTOCMDS); }
extern void rs_win_enter_ext(win_T *wp, int flags);
static void win_enter_ext(win_T *const wp, const int flags) { rs_win_enter_ext(wp, flags); }
static int last_win_id = LOWEST_WIN_ID - 1;
int nvim_get_last_win_id(void) { return last_win_id; }
win_T *nvim_alloc_win_raw(void) { return xcalloc(1, sizeof(win_T)); }
void nvim_win_init_handle(win_T *wp) { wp->handle = ++last_win_id; pmap_put(int)(&window_handles, wp->handle, wp); }
void nvim_win_init_grid(win_T *wp) { wp->w_grid_alloc.mouse_enabled = true; grid_assign_handle(&wp->w_grid_alloc); }
void nvim_win_init_vars(win_T *wp) { wp->w_vars = tv_dict_alloc(); init_var_dict(wp->w_vars, &wp->w_winvar, VAR_SCOPE); }
void nvim_win_init_ns_set(win_T *wp) { wp->w_ns_hl = -1; Set(uint32_t) ns_set = SET_INIT; wp->w_ns_set = ns_set; }
void nvim_win_init_global_local_opts(win_T *wp) { wp->w_allbuf_opt.wo_so = wp->w_p_so = -1; wp->w_allbuf_opt.wo_siso = wp->w_p_siso = -1; }
void nvim_win_set_config_init(win_T *wp) { WinConfig init = WIN_CONFIG_INIT; wp->w_config = init; }
void nvim_win_set_next_match_id(win_T *wp) { wp->w_next_match_id = 1000; }
void nvim_free_wininfo_raw(WinInfo *wip, buf_T *bp) { if (wip->wi_optset) { clear_winopt(&wip->wi_opt); deleteFoldRecurse(bp, &wip->wi_folds); } xfree(wip); }
void nvim_win_pmap_del(win_T *wp) { pmap_del(int)(&window_handles, wp->handle, NULL); }
void nvim_win_alist_unlink(win_T *wp) { alist_unlink(wp->w_alist); }
void nvim_win_clear_ns_set(win_T *wp) { set_destroy(uint32_t, &wp->w_ns_set); }
void nvim_win_clear_winopts(win_T *wp) { clear_winopt(&wp->w_onebuf_opt); clear_winopt(&wp->w_allbuf_opt); }
void nvim_win_free_lcs_chars(win_T *wp) { xfree(wp->w_p_lcs_chars.multispace); xfree(wp->w_p_lcs_chars.leadmultispace); }
void nvim_win_clear_vars(win_T *wp) { vars_clear(&wp->w_vars->dv_hashtab); hash_init(&wp->w_vars->dv_hashtab); unref_var_dict(wp->w_vars); }
void nvim_win_clear_tagstack(win_T *wp) { for (int i = 0; i < wp->w_tagstacklen; i++) { rs_tagstack_clear_entry(&wp->w_tagstack[i]); } }
void nvim_win_clear_click_defs_all(win_T *wp) { stl_clear_click_defs(wp->w_status_click_defs, wp->w_status_click_defs_size); xfree(wp->w_status_click_defs); stl_clear_click_defs(wp->w_winbar_click_defs, wp->w_winbar_click_defs_size); xfree(wp->w_winbar_click_defs); stl_clear_click_defs(wp->w_statuscol_click_defs, wp->w_statuscol_click_defs_size); xfree(wp->w_statuscol_click_defs); }
void nvim_win_clear_config_virttext(win_T *wp) { clear_virttext(&wp->w_config.title_chunks); clear_virttext(&wp->w_config.footer_chunks); }
void nvim_win_grid_clear_field(win_T *wp) { CLEAR_FIELD(wp->w_grid_alloc); }
int nvim_win_get_filler_rows(win_T *wp) { return wp ? wp->w_filler_rows : 0; }
int nvim_win_grid_has_target(win_T *wp) { return (wp && wp->w_grid.target) ? 1 : 0; }
int nvim_win_get_width_request(win_T *wp) { return wp ? wp->w_width_request : 0; }
int nvim_win_get_height_request(win_T *wp) { return wp ? wp->w_height_request : 0; }
int nvim_win_get_prev_fraction_row(win_T *wp) { return wp ? wp->w_prev_fraction_row : 0; }
int nvim_win_get_p_spk_char(void) { return (int)(unsigned char)*p_spk; }
void nvim_terminal_check_size_win(win_T *wp) { if (wp && wp->w_buffer->terminal) { terminal_check_size(wp->w_buffer->terminal); } }
int nvim_win_border_height_wrapper(win_T *wp) { return wp ? win_border_height(wp) : 0; }
int nvim_win_border_width_wrapper(win_T *wp) { return wp ? win_border_width(wp) : 0; }
win_T *nvim_get_au_pending_free_win(void) { return au_pending_free_win; }
void nvim_set_au_pending_free_win(win_T *wp) { au_pending_free_win = wp; }
void nvim_ui_call_win_viewport_margins_wrapper(win_T *wp) { if (wp && ui_has(kUIMultigrid)) { ui_call_win_viewport_margins(wp->w_grid_alloc.handle, wp->handle, wp->w_winrow_off, wp->w_border_adj[2], wp->w_wincol_off, wp->w_border_adj[1]); } }
int nvim_win_save_topline_gt_buf_line_count(win_T *wp) { return (wp && wp->w_buffer) ? wp->w_save_cursor.w_topline_save > wp->w_buffer->b_ml.ml_line_count : 0; }
void nvim_ga_init_int(garray_T *gap) { ga_init(gap, (int)sizeof(int), 1); }
void nvim_ga_grow(garray_T *gap, int n) { ga_grow(gap, n); }
int nvim_ga_get_len(garray_T *gap) { return gap ? gap->ga_len : 0; }
int nvim_ga_get_int(garray_T *gap, int idx) { return (gap && gap->ga_data && idx >= 0 && idx < gap->ga_len) ? ((int *)gap->ga_data)[idx] : 0; }
void nvim_ga_set_int(garray_T *gap, int idx, int val) { if (gap && gap->ga_data && idx >= 0) { ((int *)gap->ga_data)[idx] = val; } }
void nvim_win_stl_clear_click_defs(win_T *wp) { if (!wp) { return; } stl_clear_click_defs(wp->w_status_click_defs, wp->w_status_click_defs_size); xfree(wp->w_status_click_defs); wp->w_status_click_defs_size = 0; wp->w_status_click_defs = NULL; }
int nvim_win_get_prev_height(win_T *wp) { return wp ? wp->w_prev_height : 0; }
int nvim_win_get_fraction(win_T *wp) { return wp ? wp->w_fraction : 0; }
void nvim_ui_comp_remove_grid_win(win_T *wp) { if (wp) { ui_comp_remove_grid(&wp->w_grid_alloc); } }
void nvim_ui_call_win_hide_win(win_T *wp) { if (wp) { ui_call_win_hide(wp->w_grid_alloc.handle); } }
void nvim_merge_win_config_init(win_T *wp) { if (wp) { merge_win_config(&wp->w_config, WIN_CONFIG_INIT); CLEAR_FIELD(wp->w_border_adj); } }
int nvim_win_get_tcl_flags(void) { return (int)tcl_flags; }
void nvim_buf_inc_nwindows(buf_T *buf) { buf->b_nwindows++; }
void nvim_iemsg_move_other_frame(void) { iemsg("INTERNAL: trying to move a window into another frame"); }
int nvim_curbuf_locked(void) { return curbuf_locked() ? 1 : 0; }
int nvim_text_or_buf_locked(void) { return text_or_buf_locked() ? 1 : 0; }
int nvim_bt_quickfix_curbuf(void) { return bt_quickfix(curbuf) ? 1 : 0; }
int nvim_win_bt_prompt(win_T *wp) { return (wp && wp->w_buffer) ? (bt_prompt(wp->w_buffer) ? 1 : 0) : 0; }
void nvim_internal_error_othertab(void) { internal_error("win_close_othertab()"); }
void nvim_inc_split_disallowed(void) { split_disallowed++; }
void nvim_dec_split_disallowed(void) { split_disallowed--; }
frame_T *nvim_win_get_frame_parent(win_T *wp) { return (wp && wp->w_frame) ? wp->w_frame->fr_parent : NULL; }
buf_T *nvim_get_firstbuf_wrapper(void) { return firstbuf; }
void nvim_emsg_e_cmdwin(void) { emsg(_(e_cmdwin)); }
void nvim_msg_onlyone(void) { msg(_(m_onlyone), 0); }
int nvim_get_split_disallowed(void) { return split_disallowed; }
int nvim_win_buf_locked_split(win_T *wp) { return wp->w_buffer->b_locked_split ? 1 : 0; }
int nvim_get_cmdmod_split(void) { return cmdmod.cmod_split; }
int nvim_win_get_floating_win(win_T *wp) { return (wp && wp->w_floating) ? 1 : 0; }
buf_T *nvim_buflist_findname_exp(const char *ptr) { return buflist_findname_exp((char *)ptr); }
void nvim_reset_binding_curwin(void) { RESET_BINDING(curwin); }
int nvim_do_ecmd_lastl_hide(const char *ptr) { return do_ecmd(0, ptr, NULL, NULL, ECMD_LASTL, ECMD_HIDE, NULL); }
void nvim_find_pattern_in_path_split(const char *ptr, size_t len, int type, int prenum1, int whole)
{ find_pattern_in_path(ptr, 0, len, true, whole != 0, type, prenum1, ACTION_SPLIT, 1, MAXLNUM, false, false); }
void nvim_set_curswant_curwin(void) { curwin->w_set_curswant = true; }
size_t nvim_find_ident_under_cursor(char **pp) { return rs_find_ident_under_cursor(pp, FIND_IDENT); }
void nvim_set_cmdmod_tab_to_curtab_idx(void) { cmdmod.cmod_tab = rs_tabpage_index(curtab) + 1; }
int nvim_win_new_float_external(void)
{ if (curwin->w_floating || !ui_has(kUIMultigrid)) { return -1; } WinConfig config = WIN_CONFIG_INIT; config.width = curwin->w_width; config.height = curwin->w_height; config.external = true; Error err = ERROR_INIT; if (!win_new_float(curwin, false, config, &err)) { emsg(err.msg); api_clear_error(&err); return 0; } return 1; }
int nvim_win_get_pos_changed(win_T *wp) { return wp ? wp->w_pos_changed : 0; }
win_T *nvim_handle_get_window(int handle) { Error dummy = ERROR_INIT; win_T *wp = find_window_by_handle(handle, &dummy); api_clear_error(&dummy); return wp; }
void nvim_win_ui_call_win_pos(int grid, int win, int row, int col, int width, int height) { ui_call_win_pos(grid, win, row, col, width, height); }
void nvim_win_ui_call_win_float_pos(int grid_handle, int win_handle, int anchor,
                                     int anchor_grid, double row, double col,
                                     int mouse, int zindex, int comp_index,
                                     int screen_row, int screen_col)
{ String anchor_str = cstr_as_string(float_anchor_str[anchor]); ui_call_win_float_pos(grid_handle, win_handle, anchor_str, anchor_grid, row, col, (bool)mouse, zindex, comp_index, screen_row, screen_col); }
void nvim_win_ui_call_win_external_pos(int grid_handle, int win_handle) { ui_call_win_external_pos(grid_handle, win_handle); }
void nvim_win_ui_check_cursor_grid(int grid_handle) { ui_check_cursor_grid(grid_handle); }
int nvim_buf_get_prompt_insert(buf_T *buf) { return buf ? buf->b_prompt_insert : 0; }
void nvim_buf_set_prompt_insert(buf_T *buf, int val) { if (buf) { buf->b_prompt_insert = val; } }
buf_T *nvim_win_get_buf_ptr(win_T *wp) { return wp ? wp->w_buffer : NULL; }
void nvim_win_sync_s(win_T *wp) { if (wp && wp->w_buffer) { wp->w_s = &wp->w_buffer->b_s; } }
void nvim_win_set_script_ctx_scroll(win_T *wp) { if (wp) { wp->w_p_script_ctx[kWinOptScroll].sc_sid = SID_WINLAYOUT; wp->w_p_script_ctx[kWinOptScroll].sc_lnum = 0; } }
int nvim_win_get_do_win_fix_cursor(win_T *wp) { return wp ? (wp->w_do_win_fix_cursor ? 1 : 0) : 0; }
int nvim_win_get_prev_winrow(win_T *wp) { return wp ? wp->w_prev_winrow : 0; }
int nvim_has_event_winclosed(void) { return has_event(EVENT_WINCLOSED) ? 1 : 0; }
void nvim_apply_autocmds_winclosed_by_handle(const char *handle_str, win_T *win) { apply_autocmds(EVENT_WINCLOSED, (char *)handle_str, (char *)handle_str, false, win->w_buffer); }
win_T *nvim_get_cmdwin_win(void) { return cmdwin_win; }
win_T *nvim_get_cmdwin_old_curwin(void) { return cmdwin_old_curwin; }
void nvim_api_set_error_e_cmdwin(Error *err) { api_set_error(err, kErrorTypeException, "%s", e_cmdwin); }
void nvim_set_option_cmdheight(int64_t val) { set_option_value(kOptCmdheight, NUMBER_OPTVAL(val), 0); }
int nvim_win_get_p_wbr_empty(win_T *wp) { return (!wp || !wp->w_p_wbr || *wp->w_p_wbr == NUL) ? 1 : 0; }
int nvim_win_get_p_wbr_both_empty(win_T *wp) { return (!wp || ((*p_wbr == NUL) && (!wp->w_p_wbr || *wp->w_p_wbr == NUL))) ? 1 : 0; }
void nvim_win_clear_winbar_click_defs(win_T *wp) { if (!wp) { return; } stl_clear_click_defs(wp->w_winbar_click_defs, wp->w_winbar_click_defs_size); xfree(wp->w_winbar_click_defs); wp->w_winbar_click_defs_size = 0; wp->w_winbar_click_defs = NULL; }
void nvim_apply_autocmds_event(int event) { apply_autocmds((event_T)event, NULL, NULL, false, curbuf); }
void nvim_apply_autocmds_winresized(const char *winid_str, void *buf) { apply_autocmds(EVENT_WINRESIZED, (char *)winid_str, (char *)winid_str, false, (buf_T *)buf); }
void nvim_apply_autocmds_winscrolled(const char *winid_str, void *buf) { apply_autocmds(EVENT_WINSCROLLED, (char *)winid_str, (char *)winid_str, false, (buf_T *)buf); }
const char *nvim_curwin_get_localdir(void) { return curwin->w_localdir; }
const char *nvim_curtab_get_localdir(void) { return curtab->tp_localdir; }
const char *nvim_get_globaldir(void) { return globaldir; }
void nvim_set_globaldir_from_str(const char *s) { globaldir = xstrdup(s); }
void nvim_clear_globaldir(void) { XFREE_CLEAR(globaldir); }
int nvim_os_dirname_maxpathl(char *buf) { return (int)os_dirname(buf, MAXPATHL); }
int nvim_get_p_acd(void) { return p_acd ? 1 : 0; }
void nvim_set_last_chdir_reason_null(void) { last_chdir_reason = NULL; }
void nvim_do_autocmd_dirchanged_win(const char *new_dir, int localdir, int pre) { do_autocmd_dirchanged(new_dir, localdir ? kCdScopeWindow : kCdScopeTabpage, kCdCauseWindow, pre != 0); }
void nvim_do_autocmd_dirchanged_global(char *new_dir, int pre) { do_autocmd_dirchanged(new_dir, kCdScopeGlobal, kCdCauseWindow, pre != 0); }
int nvim_swb_has_useopen(void) { return (swb_flags & kOptSwbFlagUseopen) ? 1 : 0; }
int nvim_swb_has_usetab(void) { return (swb_flags & kOptSwbFlagUsetab) ? 1 : 0; }
void nvim_set_p_ch(int64_t val) { p_ch = val; }
int nvim_get_command_frame_height(void) { return command_frame_height ? 1 : 0; }
int nvim_get_curtab_ch_used(void) { return curtab ? (int)curtab->tp_ch_used : 0; }
void nvim_set_curtab_ch_used(int64_t val) { if (curtab) { curtab->tp_ch_used = val; } }
void nvim_set_min_set_ch(int64_t val) { min_set_ch = val; }
void nvim_update_cmdline_row(void) { cmdline_row = Rows - (int)p_ch; }
void nvim_grid_clear_cmd_area(void)
{ if (msg_scrolled != 0 || !full_screen) { return; } GridView *grid = &default_gridview; if (!ui_has(kUIMessages)) { msg_grid_validate(); grid = &msg_grid_adj; } grid_clear(grid, cmdline_row, Rows, 0, Columns, 0); msg_row = cmdline_row; }
void nvim_semsg_e92_buf_not_found(int64_t nr) { semsg(_("E92: Buffer %" PRId64 " not found"), nr); }
void nvim_apply_autocmds_tabnewentered(void) { apply_autocmds(EVENT_TABNEWENTERED, NULL, NULL, false, curbuf); }
int64_t nvim_get_p_tpm(void) { return p_tpm; }
void nvim_api_set_err_e242(Error *err) { api_set_error(err, kErrorTypeException, "E242: Can't split a window while closing another"); }
void nvim_api_set_err_cannot_split_closing(Error *err) { api_set_error(err, kErrorTypeException, "%s", e_cannot_split_window_when_closing_buffer); }
int nvim_win_can_abandon(win_T *wp, int forceit) { return (wp && wp->w_buffer) ? can_abandon(wp->w_buffer, forceit) : 1; }
void nvim_win_dialog_changed(win_T *wp) { if (wp && wp->w_buffer) { dialog_changed(wp->w_buffer, false); } }
int nvim_win_bufIsChanged(win_T *wp) { return (wp && wp->w_buffer) ? bufIsChanged(wp->w_buffer) : 0; }
int nvim_win_buf_hide(win_T *wp) { return (wp && wp->w_buffer) ? buf_hide(wp->w_buffer) : 0; }
int nvim_get_p_confirm(void) { return p_confirm ? 1 : 0; }
int nvim_get_cmdmod_confirm(void) { return (cmdmod.cmod_flags & CMOD_CONFIRM) ? 1 : 0; }
int nvim_get_p_write(void) { return p_write ? 1 : 0; }
void nvim_set_curwin_from_wp(win_T *wp) { if (wp) { curwin = wp; curbuf = wp->w_buffer; } }
int nvim_get_RedrawingDisabled(void) { return RedrawingDisabled; }
void nvim_set_RedrawingDisabled(int val) { RedrawingDisabled = val; }
void nvim_inc_RedrawingDisabled(void) { RedrawingDisabled++; }
void nvim_dec_RedrawingDisabled(void) { RedrawingDisabled--; }
int nvim_win_buf_b_locked(win_T *wp) { return (wp && wp->w_buffer && wp->w_buffer->b_locked > 0) ? 1 : 0; }
void nvim_ui_call_win_viewport_wrapper(int grid, int win, int topline, int botline,
                                       int curline, int curcol, int line_count, int64_t delta)
{
  ui_call_win_viewport(grid, win, topline, botline, curline, curcol, line_count, delta);
}
int nvim_get_postponed_split_tab(void) { return postponed_split_tab; }
void nvim_set_postponed_split_tab(int val) { postponed_split_tab = val; }
int nvim_get_cmdmod_tab(void) { return (int)cmdmod.cmod_tab; }
void nvim_set_cmdmod_tab(int val) { cmdmod.cmod_tab = val; }
void nvim_set_firstwin_null(void) { firstwin = NULL; }
void nvim_set_lastwin_null(void) { lastwin = NULL; }
int nvim_get_starting(void) { return starting; }
void nvim_set_lastused_tabpage_from_rust(tabpage_T *tp) { lastused_tabpage = tp; }
void nvim_set_skip_win_fix_scroll(int val) { skip_win_fix_scroll = (val != 0); }
void nvim_set_cmdheight_for_tabpage(int64_t new_ch) { command_frame_height = false; set_option_value(kOptCmdheight, NUMBER_OPTVAL(new_ch), 0); command_frame_height = true; }
int nvim_win_get_changelistidx(win_T *wp) { return wp ? wp->w_changelistidx : 0; }
void nvim_win_init_copy_compound(win_T *dst, win_T *src, int flags)
{
  if (!dst || !src) { return; }
  if (src->w_buffer) { dst->w_buffer = src->w_buffer; dst->w_s = &src->w_buffer->b_s; src->w_buffer->b_nwindows++; }
  dst->w_pcmark = src->w_pcmark; dst->w_prev_pcmark = src->w_prev_pcmark;
  copy_jumplist(src, dst);
  if (flags & WSP_NEWLOC) { dst->w_llist = NULL; dst->w_llist_ref = NULL; } else { copy_loclist_stack(src, dst); }
  dst->w_localdir = (src->w_localdir == NULL) ? NULL : xstrdup(src->w_localdir);
  dst->w_prevdir = (src->w_prevdir == NULL) ? NULL : xstrdup(src->w_prevdir);
  for (int i = 0; i < src->w_tagstacklen; i++) {
    taggy_T *tag = &dst->w_tagstack[i]; *tag = src->w_tagstack[i];
    if (tag->tagname != NULL) { tag->tagname = xstrdup(tag->tagname); }
    if (tag->user_data != NULL) { tag->user_data = xstrdup(tag->user_data); }
  }
  dst->w_tagstackidx = src->w_tagstackidx; dst->w_tagstacklen = src->w_tagstacklen;
  if (src->w_alist) { dst->w_alist = src->w_alist; dst->w_alist->al_refcount++; dst->w_arg_idx = src->w_arg_idx; }
  win_copy_options(src, dst); rs_copyFoldingState(src, dst);
}
int nvim_event_ignored_winscrolled(win_T *wp) { return wp ? (event_ignored(EVENT_WINSCROLLED, wp->w_p_eiw) ? 1 : 0) : 1; }
int nvim_event_ignored_winresized(win_T *wp) { return wp ? (event_ignored(EVENT_WINRESIZED, wp->w_p_eiw) ? 1 : 0) : 1; }
int nvim_has_event_winscrolled(void) { return has_event(EVENT_WINSCROLLED) ? 1 : 0; }
int nvim_has_event_winresized(void) { return has_event(EVENT_WINRESIZED) ? 1 : 0; }
int nvim_win_get_buf_fnum(win_T *wp) { return (wp && wp->w_buffer) ? (int)wp->w_buffer->handle : 0; }
void *nvim_tv_dict_alloc_refcount1(void) { dict_T *d = tv_dict_alloc(); if (d) { d->dv_refcount = 1; } return d; }
void nvim_tv_dict_unref_wrapper(void *dict) { if (dict) { tv_dict_unref((dict_T *)dict); } }
void *nvim_tv_list_alloc_wrapper(int count) { return tv_list_alloc((ptrdiff_t)count); }
void nvim_tv_list_append_number(void *list, int nr) { if (!list) { return; } typval_T tv = { .v_lock = VAR_UNLOCKED, .v_type = VAR_NUMBER, .vval.v_number = (varnumber_T)nr }; tv_list_append_owned_tv((list_T *)list, tv); }
int nvim_tv_dict_add_number(void *dict, const char *key, size_t key_len, int nr)
{ if (!dict || !key) { return 0; } typval_T tv = { .v_lock = VAR_UNLOCKED, .v_type = VAR_NUMBER, .vval.v_number = (varnumber_T)nr }; return tv_dict_add_tv((dict_T *)dict, key, key_len, &tv) == OK ? 1 : 0; }
int nvim_tv_dict_add_dict_wrapper(void *dict, const char *key, size_t key_len, void *child)
{ if (!dict || !key || !child) { return 0; } if (tv_dict_add_dict((dict_T *)dict, key, key_len, (dict_T *)child) == FAIL) { return 0; } ((dict_T *)child)->dv_refcount--; return 1; }
void *nvim_get_v_event_opaque(void *buf) { return get_v_event((save_v_event_T *)buf); }
void nvim_restore_v_event_opaque(void *dict, void *buf) { restore_v_event((dict_T *)dict, (save_v_event_T *)buf); }
buf_T *nvim_buflist_findnr_win(int nr) { return buflist_findnr(nr); }
void nvim_set_bufref_win(void *br, buf_T *buf) { set_bufref((bufref_T *)br, buf); }
int nvim_bufref_valid_win(void *br) { return bufref_valid((bufref_T *)br) ? 1 : 0; }
buf_T *nvim_bufref_get_buf_win(void *br) { return ((bufref_T *)br)->br_buf; }
void *nvim_tv_dict_add_list_win(void *dict, const char *key, size_t key_len, void *list) { tv_dict_add_list((dict_T *)dict, key, key_len, (list_T *)list); return dict; }
void nvim_tv_dict_extend_win(void *dst, void *src) { tv_dict_extend((dict_T *)dst, (dict_T *)src, "move"); }
void nvim_tv_dict_set_keys_readonly_win(void *dict) { tv_dict_set_keys_readonly((dict_T *)dict); }
buf_T *nvim_get_curbuf_ptr(void) { return curbuf; }
void nvim_buf_set_p_bl(buf_T *buf, int val) { if (buf) { buf->b_p_bl = (val != 0); } }
int nvim_win_buf_has_terminal_safe(win_T *win) { return (win && win->w_buffer && win->w_buffer->terminal) ? 1 : 0; }
int nvim_win_is_cmdline_win(win_T *win) { return (win == cmdline_win) ? 1 : 0; }
void nvim_set_cmdline_win_null(void) { cmdline_win = NULL; }
void nvim_apply_autocmds_bufenter_if_changed(buf_T *old_curbuf) { if (old_curbuf != curbuf) { apply_autocmds(EVENT_BUFENTER, NULL, NULL, false, curbuf); } }
int nvim_win_close_force(win_T *wp, int free_buf) { return win_close(wp, free_buf != 0, true); }
void nvim_getout_zero(void) { getout(0); }
void nvim_do_cmdline_cmd_diffoff(void) { do_cmdline_cmd("diffoff!"); }
void nvim_apply_autocmds_tabclosed(const char *idx_str, buf_T *buf) { apply_autocmds(EVENT_TABCLOSED, (char *)idx_str, (char *)idx_str, false, buf); }
int nvim_has_event_tabclosed(void) { return has_event(EVENT_TABCLOSED) ? 1 : 0; }
void nvim_curwin_set_buffer_to_curbuf(void) { if (curwin) { curwin->w_buffer = curbuf; } }
int nvim_one_window_and_locked_split(void) { return (ONE_WINDOW && curwin && curwin->w_locked && curbuf && curbuf->b_locked_split && first_tabpage && first_tabpage->tp_next != NULL) ? 1 : 0; }
wline_T *nvim_win_get_wl_entry(win_T *wp, int idx) { return (idx < 0 || idx >= wp->w_lines_valid) ? NULL : &wp->w_lines[idx]; }
linenr_T nvim_wline_get_lnum(wline_T *wl) { return wl->wl_lnum; }
linenr_T nvim_wline_get_foldend(wline_T *wl) { return wl->wl_foldend; }
bool nvim_wline_get_valid(wline_T *wl) { return wl->wl_valid; }
bool nvim_wline_get_folded(wline_T *wl) { return wl->wl_folded; }
uint16_t nvim_wline_get_size(wline_T *wl) { return wl->wl_size; }
linenr_T nvim_wline_get_lastlnum(wline_T *wl) { return wl->wl_lastlnum; }
int nvim_buf_get_mod_set(buf_T *buf) { return buf ? buf->b_mod_set : 0; }
void nvim_buf_set_mod_set(buf_T *buf, int val) { if (buf) { buf->b_mod_set = (val != 0); } }
int nvim_win_get_old_visual_mode(win_T *wp) { return wp ? wp->w_old_visual_mode : 0; }
int nvim_redrawing(void) { return redrawing() ? 1 : 0; }
int nvim_win_rl_cursor_col(win_T *wp) { if (!wp) { return 0; } char *cursor = ml_get_buf(wp->w_buffer, wp->w_cursor.lnum) + wp->w_cursor.col; return wp->w_view_width - wp->w_wcol - ((utf_ptr2cells(cursor) == 2 && vim_isprintc(utf_ptr2char(cursor))) ? 2 : 1); }
void nvim_grid_adjust_cursor_goto(win_T *wp, int row, int col) { ScreenGrid *grid = grid_adjust(&wp->w_grid, &row, &col); if (grid) { ui_grid_cursor_goto(grid->handle, row, col); } }
void nvim_validate_cursor_for_win(win_T *wp) { validate_cursor(wp); }
int nvim_VIsual_active(void) { return VIsual_active ? 1 : 0; }
void nvim_trans_characters(char *buf, size_t bufsize) { trans_characters(buf, (int)bufsize); }
void nvim_ui_call_set_title(const char *s) { ui_call_set_title(cstr_as_string(s ? s : "")); }
void nvim_ui_call_set_icon(const char *s) { ui_call_set_icon(cstr_as_string(s ? s : "")); }
int nvim_utf_cp_bounds_end_off(const char *str, const char *ptr) { return utf_cp_bounds((char *)str, (char *)ptr).end_off; }
int nvim_cmdline_mouse_used(void) { return get_cmdline_info()->mouse_used != NULL ? 1 : 0; }
void nvim_set_vim_var_echospace(int val) { set_vim_var_nr(VV_ECHOSPACE, val); }
bool nvim_win_get_b_cjk(const win_T *wp) { return wp->w_s->b_cjk != 0; }
const bool *nvim_win_get_b_spell_ismw(const win_T *wp) { return wp->w_s->b_spell_ismw; }
const char *nvim_win_get_b_spell_ismw_mb(const win_T *wp) { return wp->w_s->b_spell_ismw_mb; }
const garray_T *nvim_win_get_b_langp(const win_T *wp) { return &wp->w_s->b_langp; }
void nvim_emsg_no_spell(void) { emsg(_(e_no_spell)); }
regprog_T *nvim_win_get_b_cap_prog(const win_T *wp) { return wp->w_s->b_cap_prog; }
int nvim_win_spell_capcol_regexec(win_T *wp, char *ptr)
{ regmatch_T regmatch = { .regprog = wp->w_s->b_cap_prog, .rm_ic = false }; bool r = vim_regexec(&regmatch, ptr, 0); wp->w_s->b_cap_prog = regmatch.regprog; return r ? (int)(regmatch.endp[0] - ptr) : -1; }
void nvim_set_topline(win_T *wp, int lnum) { set_topline(wp, (linenr_T)lnum); }
typval_T *nvim_eval_tv_idx(typval_T *argvars, int i) { return &argvars[i]; }
void nvim_eval_tv_set_number(typval_T *tv, int64_t n) { tv->v_type = VAR_NUMBER; tv->vval.v_number = (varnumber_T)n; }
void nvim_eval_tv_set_string(typval_T *tv, char *s) { tv->vval.v_string = s; }
void nvim_eval_tv_set_type(typval_T *tv, int t) { tv->v_type = (VarType)t; }
dict_T *nvim_win_get_vars(win_T *wp) { return wp->w_vars; }
int nvim_win_get_nrwidth(win_T *wp) { return wp->w_nrwidth; }
linenr_T nvim_win_get_statuscol_line_count(win_T *wp) { return wp->w_statuscol_line_count; }
int nvim_stcp_get_width(statuscol_T *stcp) { return stcp->width; }
void nvim_stcp_set_width(statuscol_T *stcp, int val) { stcp->width = val; }
stl_hlrec_t *nvim_stcp_get_hlrec(statuscol_T *stcp) { return stcp->hlrec; }
colnr_T *nvim_stcp_get_fold_vcol(statuscol_T *stcp) { return stcp->fold_vcol; }
char *nvim_hlrec_get_start(stl_hlrec_t *sp) { return sp->start; }
int nvim_hlrec_get_item(stl_hlrec_t *sp) { return (int)sp->item; }
int nvim_hlrec_get_userhl(stl_hlrec_t *sp) { return sp->userhl; }
stl_hlrec_t *nvim_hlrec_next(stl_hlrec_t *sp) { return sp + 1; }
int nvim_build_statuscol_str(win_T *wp, linenr_T lnum, linenr_T relnum, char *buf, statuscol_T *stcp) { return build_statuscol_str(wp, lnum, relnum, buf, stcp); }
linenr_T nvim_get_cursor_rel_lnum(win_T *wp, linenr_T lnum) { return get_cursor_rel_lnum(wp, lnum); }
size_t nvim_transstr_buf(const char *s, ptrdiff_t slen, char *buf, size_t buflen) { return transstr_buf(s, slen, buf, buflen, true); }
const char *nvim_get_p_sel(void) { return p_sel; }
linenr_T nvim_get_spell_redraw_lnum(void) { return spell_redraw_lnum; }
void nvim_set_spell_redraw_lnum(linenr_T val) { spell_redraw_lnum = val; }
int nvim_get_dy_flags(void) { return dy_flags; }
linenr_T nvim_curwin_cursor_lnum(void) { return curwin->w_cursor.lnum; }
colnr_T nvim_curwin_cursor_col(void) { return curwin->w_cursor.col; }
colnr_T nvim_curwin_cursor_coladd(void) { return curwin->w_cursor.coladd; }
GridView *nvim_win_get_grid(win_T *wp) { return &wp->w_grid; }
int nvim_curwin_cursor_line_is_nul(void) { return *ml_get_buf(curwin->w_buffer, curwin->w_cursor.lnum) == NUL ? 1 : 0; }
void nvim_get_VIsual_pos_fields(int32_t *lnum, int32_t *col, int32_t *coladd) { *lnum = (int32_t)VIsual.lnum; *col = (int32_t)VIsual.col; *coladd = (int32_t)VIsual.coladd; }
int32_t nvim_curwin_get_stl_cursor_lnum(void) { return (int32_t)curwin->w_stl_cursor.lnum; }
int32_t nvim_curwin_get_stl_cursor_col(void) { return (int32_t)curwin->w_stl_cursor.col; }
int32_t nvim_curwin_get_stl_cursor_coladd(void) { return (int32_t)curwin->w_stl_cursor.coladd; }
int32_t nvim_curwin_get_stl_virtcol(void) { return (int32_t)curwin->w_stl_virtcol; }
int32_t nvim_curwin_get_stl_topline(void) { return (int32_t)curwin->w_stl_topline; }
int32_t nvim_curwin_get_stl_line_count(void) { return (int32_t)curwin->w_stl_line_count; }
int32_t nvim_curwin_get_stl_topfill(void) { return (int32_t)curwin->w_stl_topfill; }
int32_t nvim_curwin_get_stl_empty(void) { return (int32_t)curwin->w_stl_empty; }
int32_t nvim_curwin_get_stl_recording(void) { return (int32_t)curwin->w_stl_recording; }
int32_t nvim_curwin_get_stl_state(void) { return (int32_t)curwin->w_stl_state; }
int32_t nvim_curwin_get_stl_visual_mode(void) { return (int32_t)curwin->w_stl_visual_mode; }
int32_t nvim_curwin_get_stl_vis_lnum(void) { return (int32_t)curwin->w_stl_visual_pos.lnum; }
int32_t nvim_curwin_get_stl_vis_col(void) { return (int32_t)curwin->w_stl_visual_pos.col; }
int32_t nvim_curwin_get_stl_vis_coladd(void) { return (int32_t)curwin->w_stl_visual_pos.coladd; }
void nvim_curwin_set_stl_from_cursor(int32_t state, int32_t empty_line,
                                     int32_t visual_active, int32_t visual_mode,
                                     int32_t vis_lnum, int32_t vis_col, int32_t vis_coladd)
{ curwin->w_stl_cursor = curwin->w_cursor; curwin->w_stl_virtcol = curwin->w_virtcol; curwin->w_stl_empty = (char)empty_line; curwin->w_stl_topline = curwin->w_topline; curwin->w_stl_line_count = curwin->w_buffer->b_ml.ml_line_count; curwin->w_stl_topfill = curwin->w_topfill; curwin->w_stl_recording = reg_recording; curwin->w_stl_state = state; if (visual_active) { curwin->w_stl_visual_mode = visual_mode; curwin->w_stl_visual_pos.lnum = (linenr_T)vis_lnum; curwin->w_stl_visual_pos.col = (colnr_T)vis_col; curwin->w_stl_visual_pos.coladd = (colnr_T)vis_coladd; } }
char *nvim_get_empty_string_option(void) { return empty_string_option; }
int nvim_hasFoldingWin(win_T *wp, linenr_T lnum, linenr_T *firstp, linenr_T *lastp) { return hasFoldingWin(wp, lnum, firstp, lastp, true, NULL) ? 1 : 0; }
linenr_T nvim_curwin_get_topline(void) { return curwin->w_topline; }
int nvim_curwin_get_topfill(void) { return curwin->w_topfill; }
int nvim_curwin_is_qf_not_ll(void) { return curwin->w_llist_ref == NULL ? 1 : 0; }
void nvim_curwin_invalidate_wrow_wcol_virtcol(void) { curwin->w_valid &= ~(VALID_WROW | VALID_WCOL | VALID_VIRTCOL); }
void nvim_curwin_clear_wcol_virtcol(void) { curwin->w_valid &= ~(VALID_WCOL | VALID_VIRTCOL); }
void nvim_curwin_cursor_lnum_add(linenr_T delta) { curwin->w_cursor.lnum += delta; }
void nvim_set_Insstart_from_cursor(void) { Insstart = curwin->w_cursor; }
void *nvim_win_get_opt_field_addr(win_T *win, OptIndex idx)
{
  if (!win) { return NULL; }
  switch (idx) {
  case kOptSidescrolloff: return &win->w_p_siso;
  case kOptScrolloff: return &win->w_p_so;
  case kOptShowbreak: return &win->w_p_sbr;
  case kOptStatusline: return &win->w_p_stl;
  case kOptWinbar: return &win->w_p_wbr;
  case kOptFillchars: return &win->w_p_fcs;
  case kOptListchars: return &win->w_p_lcs;
  case kOptVirtualedit: return &win->w_p_ve;
  case kOptArabic: return &win->w_p_arab;
  case kOptList: return &win->w_p_list;
  case kOptSpell: return &win->w_p_spell;
  case kOptCursorcolumn: return &win->w_p_cuc;
  case kOptCursorline: return &win->w_p_cul;
  case kOptCursorlineopt: return &win->w_p_culopt;
  case kOptColorcolumn: return &win->w_p_cc;
  case kOptDiff: return &win->w_p_diff;
  case kOptEventignorewin: return &win->w_p_eiw;
  case kOptFoldcolumn: return &win->w_p_fdc;
  case kOptFoldenable: return &win->w_p_fen;
  case kOptFoldignore: return &win->w_p_fdi;
  case kOptFoldlevel: return &win->w_p_fdl;
  case kOptFoldmethod: return &win->w_p_fdm;
  case kOptFoldminlines: return &win->w_p_fml;
  case kOptFoldnestmax: return &win->w_p_fdn;
  case kOptFoldexpr: return &win->w_p_fde;
  case kOptFoldtext: return &win->w_p_fdt;
  case kOptFoldmarker: return &win->w_p_fmr;
  case kOptNumber: return &win->w_p_nu;
  case kOptRelativenumber: return &win->w_p_rnu;
  case kOptNumberwidth: return &win->w_p_nuw;
  case kOptWinfixbuf: return &win->w_p_wfb;
  case kOptWinfixheight: return &win->w_p_wfh;
  case kOptWinfixwidth: return &win->w_p_wfw;
  case kOptPreviewwindow: return &win->w_p_pvw;
  case kOptLhistory: return &win->w_p_lhi;
  case kOptRightleft: return &win->w_p_rl;
  case kOptRightleftcmd: return &win->w_p_rlc;
  case kOptScroll: return &win->w_p_scr;
  case kOptSmoothscroll: return &win->w_p_sms;
  case kOptWrap: return &win->w_p_wrap;
  case kOptLinebreak: return &win->w_p_lbr;
  case kOptBreakindent: return &win->w_p_bri;
  case kOptBreakindentopt: return &win->w_p_briopt;
  case kOptScrollbind: return &win->w_p_scb;
  case kOptCursorbind: return &win->w_p_crb;
  case kOptConcealcursor: return &win->w_p_cocu;
  case kOptConceallevel: return &win->w_p_cole;
  case kOptSpellcapcheck: return &win->w_s->b_p_spc;
  case kOptSpellfile: return &win->w_s->b_p_spf;
  case kOptSpelllang: return &win->w_s->b_p_spl;
  case kOptSpelloptions: return &win->w_s->b_p_spo;
  case kOptSigncolumn: return &win->w_p_scl;
  case kOptWinhighlight: return &win->w_p_winhl;
  case kOptWinblend: return &win->w_p_winbl;
  case kOptStatuscolumn: return &win->w_p_stc;
  default: abort();
  }
}
char **nvim_winopt_string_field_ptr(winopt_T *wop, int idx)
{
  switch (idx) {
  case 0: return &wop->wo_fdc;
  case 1: return &wop->wo_fdc_save;
  case 2: return &wop->wo_fdi;
  case 3: return &wop->wo_fdm;
  case 4: return &wop->wo_fdm_save;
  case 5: return &wop->wo_fde;
  case 6: return &wop->wo_fdt;
  case 7: return &wop->wo_fmr;
  case 8: return &wop->wo_eiw;
  case 9: return &wop->wo_scl;
  case 10: return &wop->wo_rlc;
  case 11: return &wop->wo_sbr;
  case 12: return &wop->wo_stl;
  case 13: return &wop->wo_culopt;
  case 14: return &wop->wo_cc;
  case 15: return &wop->wo_cocu;
  case 16: return &wop->wo_briopt;
  case 17: return &wop->wo_winhl;
  case 18: return &wop->wo_lcs;
  case 19: return &wop->wo_fcs;
  case 20: return &wop->wo_ve;
  case 21: return &wop->wo_wbr;
  case 22: return &wop->wo_stc;
  default: return NULL;
  }
}
void nvim_copy_winopt_scalars(winopt_T *from, winopt_T *to)
{
  to->wo_arab = from->wo_arab; to->wo_list = from->wo_list; to->wo_nu = from->wo_nu;
  to->wo_rnu = from->wo_rnu; to->wo_ve_flags = from->wo_ve_flags; to->wo_nuw = from->wo_nuw;
  to->wo_rl = from->wo_rl; to->wo_wrap = from->wo_wrap; to->wo_wrap_save = from->wo_wrap_save;
  to->wo_lbr = from->wo_lbr; to->wo_bri = from->wo_bri; to->wo_scb = from->wo_scb;
  to->wo_scb_save = from->wo_scb_save; to->wo_sms = from->wo_sms; to->wo_crb = from->wo_crb;
  to->wo_crb_save = from->wo_crb_save; to->wo_siso = from->wo_siso; to->wo_so = from->wo_so;
  to->wo_spell = from->wo_spell; to->wo_cuc = from->wo_cuc; to->wo_cul = from->wo_cul;
  to->wo_diff = from->wo_diff; to->wo_diff_saved = from->wo_diff_saved; to->wo_cole = from->wo_cole;
  to->wo_fen = from->wo_fen; to->wo_fen_save = from->wo_fen_save; to->wo_fml = from->wo_fml;
  to->wo_fdl = from->wo_fdl; to->wo_fdl_save = from->wo_fdl_save; to->wo_fdn = from->wo_fdn;
  to->wo_lhi = from->wo_lhi; to->wo_winbl = from->wo_winbl;
  to->wo_wrap_flags = from->wo_wrap_flags; to->wo_stl_flags = from->wo_stl_flags;
  to->wo_wbr_flags = from->wo_wbr_flags; to->wo_fde_flags = from->wo_fde_flags;
  to->wo_fdt_flags = from->wo_fdt_flags;
}
void nvim_copy_winopt_save_strs(winopt_T *from, winopt_T *to) { to->wo_fdc_save = from->wo_diff_saved ? xstrdup(from->wo_fdc_save) : empty_string_option; to->wo_fdm_save = from->wo_diff_saved ? xstrdup(from->wo_fdm_save) : empty_string_option; }
void nvim_copy_winopt_script_ctx(winopt_T *from, winopt_T *to) { memmove(to->wo_script_ctx, from->wo_script_ctx, sizeof(to->wo_script_ctx)); }
void nvim_win_update_grid_blending(win_T *wp) { wp->w_grid_alloc.blending = wp->w_p_winbl > 0; }


/// Get curwin->w_p_rl (right-to-left mode) for Rust FFI.
int nvim_get_curwin_p_rl(void) { return curwin->w_p_rl ? 1 : 0; }

/// Get whether curwin->w_p_rlc contains 's' for Rust FFI.
int nvim_get_curwin_p_rlc_has_s(void)
{
  return (curwin->w_p_rlc != NULL && *curwin->w_p_rlc == 's') ? 1 : 0;
}

// Accessors for create_windows / edit_buffers (Phase 3+4)
int nvim_firstwin_next_null_or_floating(void)
{ return firstwin->w_next == NULL || firstwin->w_next->w_floating ? 1 : 0; }
void nvim_set_curwin_to_firstwin(void) { curwin = firstwin; curbuf = firstwin->w_buffer; }
int nvim_curtab_get_tp_next_null(void) { return curtab->tp_next == NULL ? 1 : 0; }
int nvim_curwin_get_next_null(void) { return curwin->w_next == NULL ? 1 : 0; }
void nvim_advance_curwin_to_next(void) { curwin = curwin->w_next; }
// nvim_set_curbuf_from_curwin is exported from Rust (nvim-window crate)
int nvim_curbuf_get_ml_mfp_null(void) { return curbuf->b_ml.ml_mfp == NULL ? 1 : 0; }
int64_t nvim_get_p_fdls(void) { return p_fdls; }
// nvim_curwin_set_p_fdl already exists in buffer_shim.c
int nvim_curwin_get_arg_idx(void) { return curwin->w_arg_idx; }
void nvim_curwin_set_arg_idx(int val) { curwin->w_arg_idx = val; }
void nvim_curbuf_setfname_null(void) { setfname(curbuf, NULL, NULL, false); }
win_T *nvim_get_curwin_ptr(void) { return curwin; }
win_T *nvim_get_firstwin_ptr(void) { return firstwin; }
win_T *nvim_curwin_get_next(void) { return curwin->w_next; }
buf_T *nvim_firstwin_get_buffer(void) { return firstwin->w_buffer; }

// =============================================================================
// Spell lifecycle + command helpers for Rust (Phase 1+2 spell migration)
// =============================================================================
#include "nvim/spell.h"
#include "nvim/spellfile.h"
#include "nvim/change.h"
#include "nvim/search.h"
#include "nvim/undo.h"

/// Clear b_langp for all buffers (FOR_ALL_BUFFERS loop helper for spell_free_all).
void nvim_for_all_bufs_clear_langp(void)
{
  FOR_ALL_BUFFERS(buf) {
    ga_clear(&buf->b_s.b_langp);
  }
}

/// Reload spell for the first matching window in curtab (FOR_ALL_WINDOWS_IN_TAB helper).
/// Calls parse_spelllang on first window with spell enabled and non-empty spelllang.
void nvim_for_all_wins_spell_reload(void)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (*wp->w_s->b_p_spl != NUL) {
      if (wp->w_p_spell) {
        parse_spelllang(wp);
        break;
      }
    }
  }
}

// =============================================================================
// Spell command helpers for Rust (Phase 2)
// =============================================================================

/// Run do_search for spellrepall: search forward for the given pattern.
/// Returns 0 if not found, non-zero if found (do_search return value).
int nvim_spell_do_search(char *frompat, size_t frompatlen)
{
  return do_search(NULL, '/', '/', frompat, frompatlen, 1, SEARCH_KEEP, NULL);
}

/// Save curwin->w_cursor position.
void nvim_curwin_save_pos(int32_t *lnum, int32_t *col)
{
  *lnum = (int32_t)curwin->w_cursor.lnum;
  *col = (int32_t)curwin->w_cursor.col;
}

/// Restore curwin->w_cursor position.
void nvim_curwin_restore_pos(int32_t lnum, int32_t col)
{
  curwin->w_cursor.lnum = (linenr_T)lnum;
  curwin->w_cursor.col = (colnr_T)col;
  curwin->w_cursor.coladd = 0;
}

/// Set curwin->w_cursor.lnum.
void nvim_curwin_set_lnum(int32_t lnum)
{
  curwin->w_cursor.lnum = (linenr_T)lnum;
}

/// Add n to curwin->w_cursor.col.
void nvim_curwin_col_add(int32_t n)
{
  curwin->w_cursor.col += (colnr_T)n;
}

/// Get curwin->w_cursor.lnum.
int32_t nvim_curwin_get_lnum(void)
{
  return (int32_t)curwin->w_cursor.lnum;
}

/// Get curwin->w_cursor.col.
int32_t nvim_curwin_get_col(void)
{
  return (int32_t)curwin->w_cursor.col;
}

/// Get current window's buffer pointer (for ml_get_buf etc.)
void *nvim_curwin_get_buf(void)
{
  return (void *)curwin->w_buffer;
}

/// Get the window's buffer pointer for spell check_need_cap use.
void *nvim_win_get_buf_ptr_void(const win_T *wp)
{
  return (void *)wp->w_buffer;
}

/// sub_nsubs global access.
void nvim_sub_nsubs_inc(void) { sub_nsubs++; }
void nvim_sub_nsubs_reset(void) { sub_nsubs = 0; }
int nvim_sub_nsubs_get(void) { return sub_nsubs; }

/// sub_nlines global access.
void nvim_sub_nlines_inc(void) { sub_nlines++; }
void nvim_sub_nlines_reset(void) { sub_nlines = 0; }

/// Check if the window's b_cap_prog is NULL.
int nvim_win_cap_prog_is_null(const win_T *wp) { return wp->w_s->b_cap_prog == NULL ? 1 : 0; }

/// Spell-specific error messages for Rust FFI.
void nvim_spell_emsg_e752(void) { emsg(_("E752: No previous spell replacement")); }
void nvim_spell_semsg_e753(const char *word) { semsg(_("E753: Not found: %s"), word); }

// =============================================================================
// ex_spelldump helpers for Rust (Phase 3)
// =============================================================================
#include "nvim/option.h"
#include "nvim/drawscreen.h"

extern void rs_optval_free(OptVal o);

/// Setup for :spelldump - get spelllang, open new buffer, set options.
/// Returns 1 if we should proceed with the dump, 0 if we should bail out.
/// The OptVal spl is freed internally.
int nvim_spelldump_setup(void)
{
  OptVal spl = get_option_value(kOptSpelllang, OPT_LOCAL);
  do_cmdline_cmd("new");
  set_option_value_give_err(kOptSpell, BOOLEAN_OPTVAL(true), OPT_LOCAL);
  set_option_value_give_err(kOptSpelllang, spl, OPT_LOCAL);
  rs_optval_free(spl);
  return buf_is_empty(curbuf) ? 1 : 0;
}

/// Delete the last line of curbuf.
void nvim_curbuf_ml_delete_last(void) { ml_delete(curbuf->b_ml.ml_line_count); }

/// Redraw the current window.
void nvim_redraw_later_not_valid(void) { redraw_later(curwin, UPD_NOT_VALID); }

// =============================================================================
// C accessors for ui.c Rust migration (Phase 1: ui_refresh / ui_grid_resize)
// Note: nvim_win_get_w_width, nvim_win_get_w_height, nvim_win_get_config_width/height,
//       nvim_win_set_config_width/height, nvim_get_first_tabpage, nvim_tabpage_get_next,
//       nvim_tabpage_set_ch_used, nvim_get_exiting are already defined as Rust drop-ins
//       in the nvim-window crate (window/src/win_struct.rs, globals.rs, tabpage_struct.rs).
// =============================================================================

/// Set window width_request (w_width_request)
void nvim_win_set_width_request(win_T *wp, int val) { if (wp) { wp->w_width_request = val; } }

/// Set window height_request (w_height_request)
void nvim_win_set_height_request(win_T *wp, int val) { if (wp) { wp->w_height_request = val; } }

/// Call win_set_inner_size on the window
void nvim_win_set_inner_size(win_T *wp, bool valid_cursor) { win_set_inner_size(wp, valid_cursor); }

// =============================================================================
// C accessors for winfloat Phase 4: win_set_minimal_style
// =============================================================================
void nvim_win_set_p_nu(win_T *wp, int val) { if (wp) { wp->w_p_nu = val != 0; } }
void nvim_win_set_p_rnu(win_T *wp, int val) { if (wp) { wp->w_p_rnu = val != 0; } }
void nvim_win_set_p_cul_wrap(win_T *wp, int val) { if (wp) { wp->w_p_cul = val != 0; } }
void nvim_win_set_p_cuc_wrap(win_T *wp, int val) { if (wp) { wp->w_p_cuc = val != 0; } }
void nvim_win_set_p_spell(win_T *wp, int val) { if (wp) { wp->w_p_spell = val != 0; } }
// nvim_win_set_p_list already in plines.c
int nvim_win_get_p_fcs_eob(win_T *wp) { return wp ? (unsigned char)wp->w_p_fcs_chars.eob : 0; }
char *nvim_win_get_p_fcs_ptr(win_T *wp) { return wp ? wp->w_p_fcs : NULL; }
void nvim_win_set_p_fcs(win_T *wp, char *val) { if (wp) { wp->w_p_fcs = val; } }
char *nvim_win_get_p_winhl_ptr(win_T *wp) { return wp ? wp->w_p_winhl : NULL; }
void nvim_win_set_p_winhl(win_T *wp, char *val) { if (wp) { wp->w_p_winhl = val; } }
char *nvim_win_get_p_scl_ptr(win_T *wp) { return wp ? wp->w_p_scl : NULL; }
void nvim_win_set_p_scl(win_T *wp, char *val) { if (wp) { wp->w_p_scl = val; } }
char *nvim_win_get_p_fdc_ptr(win_T *wp) { return wp ? wp->w_p_fdc : NULL; }
void nvim_win_set_p_fdc(win_T *wp, char *val) { if (wp) { wp->w_p_fdc = val; } }
char *nvim_win_get_p_cc_ptr(win_T *wp) { return wp ? wp->w_p_cc : NULL; }
void nvim_win_set_p_cc(win_T *wp, char *val) { if (wp) { wp->w_p_cc = val; } }
char *nvim_win_get_p_stc_ptr(win_T *wp) { return wp ? wp->w_p_stc : NULL; }
void nvim_win_set_p_stc(win_T *wp, char *val) { if (wp) { wp->w_p_stc = val; } }
char *nvim_win_get_p_stl_ptr(win_T *wp) { return wp ? wp->w_p_stl : NULL; }
void nvim_win_set_p_stl(win_T *wp, char *val) { if (wp) { wp->w_p_stl = val; } }

