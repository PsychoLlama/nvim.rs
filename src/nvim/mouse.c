#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cursor.h"
#include "nvim/decoration.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/ex_docmd.h"
#include "nvim/fold.h"
#include "nvim/getchar.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/grid_defs.h"
#include "nvim/keycodes.h"
#include "nvim/macros_defs.h"
#include "nvim/mark_defs.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/mouse.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/ops.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/plines.h"
#include "nvim/popupmenu.h"
#include "nvim/pos_defs.h"
#include "nvim/register.h"
#include "nvim/search.h"
#include "nvim/state.h"
#include "nvim/state_defs.h"
#include "nvim/statusline.h"
#include "nvim/statusline_defs.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_compositor.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

#include "mouse.c.generated.h"
// Rust FFI declarations (functions implemented in src/nvim-rs/mouse/)
extern int rs_win_fdccol_count(win_T *wp);
extern int rs_global_stl_height(void);
extern void rs_win_drag_status_line(win_T *dragwin, int offset);
extern void rs_win_drag_vsep_line(win_T *dragwin, int offset);
extern int rs_get_scrolloff_value(win_T *wp);
extern void rs_setFoldRepeat(linenr_T lnum, int count, bool do_open);
extern bool rs_mouse_model_popup(const char *p_mousem);
extern int rs_find_start_of_word(const char *line, int col);
extern int rs_find_end_of_word(const char *line, int col, bool sel_exclusive);
extern void rs_clearop(oparg_T *oap);
extern void rs_clearopbeep(oparg_T *oap);
extern void rs_prep_redo(int regname, int num, int cmd1, int cmd2, int cmd3, int cmd4, int cmd5);
extern void rs_may_start_select(int c);
// Static state accessors (state owned in Rust)
extern bool rs_get_got_click(void);
extern void rs_set_got_click(bool val);
extern win_T *rs_get_dragwin(void);
extern void rs_set_dragwin(win_T *wp);
extern bool rs_is_dragging(void);

/// Get the mouse_dragging global value.
int nvim_get_mouse_dragging(void) { return mouse_dragging; }

/// Set mouse_past_bottom global (used from Rust).
void nvim_set_mouse_past_bottom(bool val) { mouse_past_bottom = val; }

/// Set mouse_past_eol global (used from Rust).
void nvim_set_mouse_past_eol(bool val) { mouse_past_eol = val; }


extern void rs_set_mouse_topline(win_T *wp);
extern void rs_move_tab_to_mouse(void);
extern void rs_mouse_tab_close(int c1);
extern void rs_mouse_check_grid(colnr_T *vcolp, int *flagsp);
extern int rs_get_fpos_of_mouse(pos_T *mpos);
extern int rs_do_popup(int which_button, int m_pos_flag, pos_T m_pos);
extern void rs_call_click_def_func(StlClickDefinition *click_defs, int col, int which_button);

// got_click state is now owned by Rust; use rs_get_got_click()/rs_set_got_click().
// nvim_call_stl_click_func is now implemented in Rust (rs_call_click_def_func).

// nvim_do_mouse_impl is now implemented in Rust (rs_do_mouse_impl).
// dragwin state and jump_to_mouse logic are now owned by Rust (rs_jump_to_mouse).


// Accessors used by Rust rs_mouse_find_grid_win / rs_frame_find_win.

/// Get msg_grid.handle (handle of the message grid).
int nvim_msg_grid_get_handle(void) { return msg_grid.handle; }

/// Get msg_grid_pos global (current message grid row position).
int nvim_mouse_get_msg_grid_pos(void) { return msg_grid_pos; }

/// Get window by grid handle (wraps get_win_by_grid_handle).
win_T *nvim_get_win_by_grid_handle(int handle) { return get_win_by_grid_handle((handle_T)handle); }

/// Return true if window config has mouse enabled (for floating windows).
bool nvim_win_config_get_mouse(win_T *wp) { return wp ? wp->w_config.mouse : false; }

/// Call ui_comp_mouse_focus and return opaque ScreenGrid pointer.
ScreenGrid *nvim_ui_comp_mouse_focus(int row, int col) { return ui_comp_mouse_focus(row, col); }

/// Return true if the given ScreenGrid pointer is pum_grid.
bool nvim_grid_is_pum_grid(ScreenGrid *grid) { return grid == &pum_grid; }

/// Find the window in the current tab whose w_grid_alloc pointer matches grid.
/// Returns the window and adjusts *rowp/*colp relative to the window's grid.
/// Returns NULL if no matching window.
win_T *nvim_curtab_find_win_by_grid_alloc(ScreenGrid *grid, int *rowp, int *colp)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (&wp->w_grid_alloc != grid) {
      continue;
    }
    *colp -= wp->w_wincol + wp->w_grid.col_offset;
    *rowp -= wp->w_winrow + wp->w_grid.row_offset;
    return wp;
  }
  return NULL;
}

/// Get frame width (fr_width field).
int nvim_ses_frame_get_width(frame_T *fp) { return fp ? fp->fr_width : 0; }

/// Get frame height (fr_height field).
int nvim_ses_frame_get_height(frame_T *fp) { return fp ? fp->fr_height : 0; }

/// Find the window in the current tab whose frame pointer matches fp->fr_win.
/// Returns NULL if not found.
win_T *nvim_curtab_find_win_for_frame(frame_T *fp)
{
  if (fp == NULL || fp->fr_win == NULL) {
    return NULL;
  }
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (wp == fp->fr_win) {
      return wp;
    }
  }
  return NULL;
}

/// Get firstwin->w_winrow (row position of the first window).
int nvim_get_firstwin_winrow(void) { return firstwin ? firstwin->w_winrow : 0; }

/// C accessor: convert a virtual (screen) column to a character column.
colnr_T nvim_vcol2col(win_T *wp, linenr_T lnum, colnr_T vcol, colnr_T *coladdp)
  FUNC_ATTR_NONNULL_ARG(1) FUNC_ATTR_WARN_UNUSED_RESULT
{
  char *line = ml_get_buf(wp->w_buffer, lnum);
  CharsizeArg csarg;
  CSType cstype = init_charsize_arg(&csarg, wp, lnum, line);
  StrCharInfo ci = utf_ptr2StrCharInfo(line);
  int cur_vcol = 0;
  while (cur_vcol < vcol && *ci.ptr != NUL) {
    int next_vcol = cur_vcol + win_charsize(cstype, cur_vcol, ci.ptr, ci.chr.value, &csarg).width;
    if (next_vcol > vcol) {
      break;
    }
    cur_vcol = next_vcol;
    ci = utfc_next(ci);
  }

  if (coladdp != NULL) {
    *coladdp = vcol - cur_vcol;
  }
  return (colnr_T)(ci.ptr - line);
}

// nvim_mouse_check_grid_impl and f_getmousepos migrated to Rust.

// =============================================================================
// Phase 9 accessor functions — used by rs_do_mouse_impl in Rust
// =============================================================================

// --- Input helpers -----------------------------------------------------------


/// Peek at the next character in the input queue without consuming it.
int nvim_vpeekc(void) { return vpeekc(); }

/// Get next character safely (handles special keys).
int nvim_safe_vgetc(void) { return safe_vgetc(); }

/// Push a character back onto the input queue.
void nvim_vungetc(int c) { vungetc(c); }

/// Insert a character into the readbuff (stuff buffer).
void nvim_stuffcharReadbuff(int c) { stuffcharReadbuff(c); }

/// Insert a number into the readbuff (for count prefixes).
void nvim_stuffnumReadbuff(int n) { stuffnumReadbuff(n); }

/// Insert a string into the readbuff.
void nvim_stuffReadbuff(const char *s) { stuffReadbuff(s); }

/// Append a character to the redo buffer.
void nvim_AppendCharToRedobuff(int c) { AppendCharToRedobuff(c); }

// --- Mouse global setters ----------------------------------------------------

/// Set mouse_grid global.
void nvim_set_mouse_grid(int val) { mouse_grid = val; }

/// Set mouse_row global.
void nvim_set_mouse_row(int val) { mouse_row = val; }

/// Set mouse_col global.
void nvim_set_mouse_col(int val) { mouse_col = val; }

// nvim_set_mouse_dragging already defined in normal_shim.c

// --- Mouse global getters ----------------------------------------------------

/// Get mouse_past_bottom global.
bool nvim_get_mouse_past_bottom(void) { return mouse_past_bottom; }

/// Get mouse_past_eol global.
bool nvim_get_mouse_past_eol(void) { return mouse_past_eol; }

// --- Mode/state accessors ----------------------------------------------------

/// Get the current editor State.
int nvim_get_state_mouse(void) { return State; }

/// Get the current mod_mask.
int nvim_get_mod_mask_mouse(void) { return mod_mask; }

/// Get VIsual_active flag.
bool nvim_get_VIsual_active_mouse(void) { return VIsual_active; }

/// Get VIsual_mode value.
int nvim_get_VIsual_mode_mouse(void) { return VIsual_mode; }

/// Set VIsual_mode value.
void nvim_set_VIsual_mode_mouse(int val) { VIsual_mode = val; }

/// Get VIsual_select flag.
bool nvim_get_VIsual_select_mouse(void) { return VIsual_select; }

/// Get mode_displayed flag.
bool nvim_get_mode_displayed_mouse(void) { return mode_displayed; }

/// Get p_smd (show mode) option.
int nvim_get_p_smd_mouse(void) { return p_smd; }

/// Get msg_silent value.
int nvim_get_msg_silent_mouse(void) { return msg_silent; }

// nvim_get_p_sel already defined in window_shim.c

/// Get p_mousem (mousemodel) string pointer.
const char *nvim_get_p_mousem(void) { return (const char *)p_mousem; }

/// Get Columns (screen width) global.
int nvim_get_Columns_mouse(void) { return Columns; }

/// Get firstwin->w_winrow (or 0 if firstwin is NULL).
int nvim_get_firstwin_winrow_mouse(void) { return firstwin ? firstwin->w_winrow : 0; }

/// Get cmdwin_type global.
int nvim_get_cmdwin_type_mouse(void) { return cmdwin_type; }

/// Return true if tab_page_click_defs is initialized (non-NULL).
bool nvim_tab_page_click_defs_valid(void) { return tab_page_click_defs != NULL; }

/// Get tab_page_click_defs_size.
int nvim_get_tab_page_click_defs_size(void) { return (int)tab_page_click_defs_size; }


/// Get pointer to tab_page_click_defs (as opaque handle).
StlClickDefinition *nvim_get_tab_page_click_defs_ptr(void) { return tab_page_click_defs; }

/// Get restart_edit global.
int nvim_get_restart_edit_mouse(void) { return restart_edit; }

/// Get VIsual_reselect flag.
bool nvim_get_VIsual_reselect(void) { return VIsual_reselect; }

// --- Insert/put/register operations ------------------------------------------

// nvim_eval_has_provider already defined in eval/funcs_shim.c
// nvim_mouse_middle_insert_mode is now implemented in Rust (inlined in rs_do_mouse_impl).
// nvim_do_put_middle_click is now implemented in Rust (inlined in rs_do_mouse_impl).

/// Set where_paste_started to current cursor position.
void nvim_set_where_paste_started_to_cursor(void)
{
  where_paste_started = curwin->w_cursor;
}

// --- Click defs accessors ----------------------------------------------------

/// Get w_status_click_defs for a window.
StlClickDefinition *nvim_win_get_status_click_defs(win_T *wp)
{
  return wp ? wp->w_status_click_defs : NULL;
}

/// Get w_winbar_click_defs for a window.
StlClickDefinition *nvim_win_get_winbar_click_defs(win_T *wp)
{
  return wp ? wp->w_winbar_click_defs : NULL;
}

/// Get w_statuscol_click_defs for a window.
StlClickDefinition *nvim_win_get_statuscol_click_defs(win_T *wp)
{
  return wp ? wp->w_statuscol_click_defs : NULL;
}

/// Get w_status_click_defs_size for a window.
int nvim_win_get_status_click_defs_size(win_T *wp)
{
  return wp ? (int)wp->w_status_click_defs_size : 0;
}

/// Get w_statuscol_click_defs_size for a window.
int nvim_win_get_statuscol_click_defs_size(win_T *wp)
{
  return wp ? (int)wp->w_statuscol_click_defs_size : 0;
}


// --- Navigation/tag/quickfix -------------------------------------------------

/// Return 1 if curwin is a quickfix window (not location list).
int nvim_curwin_is_qf(void)
{
  return (bt_quickfix(curbuf) && curwin->w_llist_ref == NULL) ? 1 : 0;
}

/// Return 1 if curwin is a location list window.
int nvim_curwin_is_ll(void)
{
  return (bt_quickfix(curbuf) && curwin->w_llist_ref != NULL) ? 1 : 0;
}

/// Run a cmdline command (wraps do_cmdline_cmd).
int nvim_do_cmdline_cmd_mouse(const char *cmd)
{
  return do_cmdline_cmd(cmd);
}

/// Return 1 if curbuf is a help buffer.
int nvim_curbuf_is_help_mouse(void)
{
  return curbuf->b_help ? 1 : 0;
}

// --- Position operations -----------------------------------------------------

/// Find match using findmatch(). Returns false if no match.
/// On match, writes the position into *lnum/*col/*coladd and motion_type into *motion_type_out.
bool nvim_findmatch_nul(oparg_T *oap, linenr_T *lnum, int *col, int *coladd, int *motion_type_out)
{
  pos_T *pos = findmatch(oap, NUL);
  if (pos == NULL) {
    return false;
  }
  *lnum = pos->lnum;
  *col = pos->col;
  *coladd = pos->coladd;
  if (motion_type_out != NULL && oap != NULL) {
    *motion_type_out = (int)oap->motion_type;
  }
  return true;
}

/// Get line from memline (ml_get).
const char *nvim_ml_get_line(linenr_T lnum) { return ml_get(lnum); }

/// Check if character is ASCII whitespace.
int nvim_ascii_iswhite_mouse(int c) { return ascii_iswhite(c) ? 1 : 0; }

/// Check if character is a word character (vim_iswordc).
int nvim_vim_iswordc_mouse(int c) { return vim_iswordc((unsigned)c) ? 1 : 0; }

/// Get pointer to character at cursor position.
const char *nvim_get_cursor_pos_ptr_mouse(void) { return get_cursor_pos_ptr(); }

/// Get the byte length of the UTF-8 char at the cursor.
int nvim_utfc_ptr2len_at_cursor(void) { return utfc_ptr2len(get_cursor_pos_ptr()); }

// --- Visual/cursor operations ------------------------------------------------

/// Advance the cursor in curwin to a given column.
int nvim_curwin_coladvance(int col) { return coladvance(curwin, (colnr_T)col); }

// nvim_curwin_get_curswant already defined in drawscreen_shim.c

/// Set curwin->w_set_curswant = true.
void nvim_set_curswant_flag(void) { curwin->w_set_curswant = true; }

/// Set VIsual to current cursor position.
void nvim_set_VIsual_to_cursor(void) { VIsual = curwin->w_cursor; }

/// Get VIsual position.
void nvim_get_VIsual_pos(linenr_T *lnum, int *col, int *coladd)
{
  *lnum = VIsual.lnum;
  *col = VIsual.col;
  *coladd = VIsual.coladd;
}

/// Set VIsual position (lnum, col, coladd).
void nvim_set_VIsual_lnum_col_coladd(linenr_T lnum, int col, int coladd)
{
  VIsual.lnum = lnum;
  VIsual.col = col;
  VIsual.coladd = coladd;
}

/// Set only VIsual.col (for adjusting column in block-visual).
void nvim_set_VIsual_col_only(int col) { VIsual.col = col; }

/// Increment VIsual.col by 1.
void nvim_inc_VIsual_col(void) { VIsual.col++; }

/// Increment cursor col by 1.
void nvim_inc_cursor_col(void) { curwin->w_cursor.col++; }

// nvim_set_cursor_pos and nvim_set_cursor_col already defined in normal_shim.c

// --- Tabpage ops -------------------------------------------------------------

/// Go to a tab page by number (wraps goto_tabpage).
void nvim_goto_tabpage(int n) { goto_tabpage(n); }

/// Create a new tab page (wraps tabpage_new).
void nvim_tabpage_new(void) { tabpage_new(); }

// --- Scroll ------------------------------------------------------------------

/// Scroll and redraw (wraps scroll_redraw).
void nvim_scroll_redraw(bool up, int count) { scroll_redraw(up, count); }
