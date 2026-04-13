// drawscreen_shim.c: Minimal C accessors for the Rust drawscreen crate.

#include <stdbool.h>
#include <stdint.h>

#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/decoration.h"
#include "nvim/decoration_defs.h"
#include "nvim/decoration_provider.h"
#include "nvim/cmdexpand.h"
#include "nvim/drawline.h"
#include "nvim/drawscreen.h"
#include "nvim/ex_getln.h"
#include "nvim/fold.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/grid_defs.h"
#include "nvim/highlight.h"
#include "nvim/insexpand.h"
#include "nvim/match.h"
#include "nvim/memline.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/plines.h"
#include "nvim/popupmenu.h"
#include "nvim/profile.h"
#include "nvim/regexp.h"
#include "nvim/search.h"
#include "nvim/statusline.h"
#include "nvim/spell.h"
#include "nvim/spell.h"
#include "nvim/syntax.h"
#include "nvim/syntax_bridge.h"
#include "nvim/terminal.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_compositor.h"
#include "nvim/ui_defs.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

// Global variables used across drawscreen and shim code
bool redraw_popupmenu = false;
bool msg_grid_invalid = false;
bool resizing_autocmd = false;

// Rust FFI declarations used in shim helpers
extern void rs_ins_compl_show_pum(void);
extern int rs_min_rows(tabpage_T *tp);
extern bool rs_win_redraw_signcols(win_T *wp);

#include "drawscreen_shim.c.generated.h"

/// Return 1 if default_grid needs reallocation (size mismatch or NULL), else 0.
/// Also returns 0 if Rows==0 or Columns==0.
int nvim_default_grid_needs_alloc(void)
{
  if (Rows == 0 || Columns == 0) {
    return 0;
  }
  if (default_grid.chars != NULL
      && Rows == default_grid.rows
      && Columns == default_grid.cols) {
    return 0;
  }
  return 1;
}

bool nvim_get_updating_screen(void) { return updating_screen; }

/// Return true if default_grid.chars is non-NULL.
bool nvim_default_grid_has_chars(void) { return default_grid.chars != NULL; }

/// Return true if msg_grid.chars is non-NULL.
bool nvim_msg_grid_has_chars(void) { return msg_grid.chars != NULL; }

/// Return true if default_grid.valid is set.
bool nvim_default_grid_is_valid(void) { return default_grid.valid; }

/// Set default_grid.valid.
void nvim_default_grid_set_valid(bool val) { default_grid.valid = val; }

/// Invalidate default_grid (grid_invalidate).
void nvim_default_grid_invalidate(void) { grid_invalidate(&default_grid); }

/// Handle msg_scrolled / msg_grid_invalid block of update_screen.
/// Called from rs_update_screen() in Rust.
/// Returns the updated redraw type.
int nvim_update_screen_msg_scroll(int type, int is_stl_global)
{
  if (!msg_scrolled && !msg_grid_invalid) {
    return type;
  }
  clear_cmdline = true;
  int valid = MAX(Rows - msg_scrollsize(), 0);
  if (msg_grid.chars) {
    for (int i = 0; i < MIN(msg_scrollsize(), msg_grid.rows); i++) {
      grid_clear_line(&msg_grid, msg_grid.line_offset[i],
                      msg_grid.cols, i < p_ch);
    }
  }
  msg_grid.throttled = false;
  bool was_invalidated = false;

  if (type == UPD_NOT_VALID && !ui_has(kUIMultigrid) && msg_scrolled) {
    was_invalidated = ui_comp_set_screen_valid(false);
    for (int i = valid; i < Rows - p_ch; i++) {
      grid_clear_line(&default_grid, default_grid.line_offset[i],
                      Columns, false);
    }
    FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
      if (wp->w_floating) {
        continue;
      }
      if (W_ENDROW(wp) > valid) {
        wp->w_redr_type = MAX(wp->w_redr_type, UPD_NOT_VALID);
      }
      if (!is_stl_global && W_ENDROW(wp) + wp->w_status_height > valid) {
        wp->w_redr_status = true;
      }
    }
    if (is_stl_global && Rows - p_ch - 1 > valid) {
      curwin->w_redr_status = true;
    }
  }
  msg_grid_set_pos(Rows - (int)p_ch, false);
  msg_grid_invalid = false;
  if (was_invalidated) {
    ui_comp_set_screen_valid(true);
  }
  msg_scrolled = 0;
  msg_scrolled_at_flush = 0;
  msg_grid_scroll_discount = 0;
  need_wait_return = false;
  return type;
}

/// Handle curwin nrwidth check and tabline redraw in update_screen.
/// Called from rs_update_screen() in Rust.
void nvim_update_screen_nrwidth_check(int type)
{
  if (curwin->w_redr_type < UPD_NOT_VALID
      && curwin->w_nrwidth != ((curwin->w_p_nu || curwin->w_p_rnu || *curwin->w_p_stc)
                               ? number_width(curwin) : 0)) {
    curwin->w_redr_type = UPD_NOT_VALID;
  }

  if (curwin->w_redr_type == UPD_INVERTED) {
    update_curswant();
  }

  if (redraw_tabline || type >= UPD_NOT_VALID) {
    update_window_hl(curwin, type >= UPD_NOT_VALID);
    FOR_ALL_TABS(tp) {
      if (tp != curtab) {
        update_window_hl(tp->tp_curwin, type >= UPD_NOT_VALID);
      }
    }
    draw_tabline();
  }
}

/// Clamp 'cmdheight' for all tabpages after screen resize.
/// Also sets p_lines and p_columns.
/// Called from rs_drawscreen_screen_resize() in Rust.
void nvim_screen_resize_clamp_cmdheight(void)
{
  if (!ui_has(kUIMessages)) {
    int max_p_ch = Rows - rs_min_rows(curtab) + 1;
    if (p_ch > 0 && p_ch > max_p_ch) {
      p_ch = MAX(max_p_ch, 1);
      curtab->tp_ch_used = p_ch;
    }
    FOR_ALL_TABS(tp) {
      if (tp == curtab) {
        continue;
      }
      int max_tp_ch = Rows - rs_min_rows(tp) + 1;
      if (tp->tp_ch_used > 0 && tp->tp_ch_used > max_tp_ch) {
        tp->tp_ch_used = MAX(max_tp_ch, 1);
      }
    }
  }
  p_lines = Rows;
  p_columns = Columns;
}


/// Perform the grid_alloc + click_defs + field setup for default_grid.
/// Relocated from drawscreen.c (Phase 3).
void nvim_default_grid_do_alloc(void)
{
  grid_alloc(&default_grid, Rows, Columns, true, true);
  stl_clear_click_defs(tab_page_click_defs, tab_page_click_defs_size);
  tab_page_click_defs = stl_alloc_click_defs(tab_page_click_defs, Columns,
                                             &tab_page_click_defs_size);
  default_grid.comp_height = Rows;
  default_grid.comp_width = Columns;
  default_grid.handle = DEFAULT_GRID_HANDLE;
}

/// Perform screenclear grid operations (blanking + UI calls + state reset).
/// Relocated from drawscreen.c (Phase 3). Called from rs_screenclear() in Rust.
void nvim_screenclear_impl(void)
{
  msg_check_for_delay(false);

  if (starting == NO_SCREEN || default_grid.chars == NULL) {
    return;
  }

  // blank out the default grid
  for (int i = 0; i < default_grid.rows; i++) {
    grid_clear_line(&default_grid, default_grid.line_offset[i],
                    default_grid.cols, true);
  }

  ui_call_grid_clear(1);  // clear the display
  ui_comp_set_screen_valid(true);

  ns_hl_fast = -1;

  clear_cmdline = false;
  mode_displayed = false;

  redraw_all_later(UPD_NOT_VALID);
  cmdline_was_last_drawn = false;
  redraw_cmdline = true;
  redraw_tabline = true;
  redraw_popupmenu = true;
  pum_invalidate();
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (wp->w_floating) {
      wp->w_redr_type = UPD_CLEAR;
    }
  }
  if (must_redraw == UPD_CLEAR) {
    must_redraw = UPD_NOT_VALID;  // no need to clear again
  }
  compute_cmdrow();
  msg_row = cmdline_row;  // put cursor on last line for messages
  msg_col = 0;
  msg_reset_scroll();     // can't scroll back
  msg_didany = false;
  msg_didout = false;
  if (HL_ATTR(HLF_MSG) > 0 && msg_use_grid() && msg_grid.chars) {
    grid_invalidate(&msg_grid);
    msg_grid_validate();
    msg_grid_invalid = false;
    clear_cmdline = true;
  }
}

/// Check if screen_search_hl has a regprog.
int nvim_search_hl_has_regprog(void) { return screen_search_hl.rm.regprog != NULL ? 1 : 0; }

/// Prepare search highlight: set regprog and time limit.
/// Keeps regexp lifetime management in C.
void nvim_search_hl_start(void)
{
  last_pat_prog(&screen_search_hl.rm);
  screen_search_hl.tm = profile_setlimit(p_rdt);
}

/// Free search highlight regprog. Keeps regexp lifetime management in C.
void nvim_search_hl_end(void)
{
  vim_regfree(screen_search_hl.rm.regprog);
  screen_search_hl.rm.regprog = NULL;
}

/// Return a pointer to the global screen_search_hl (as void*).
/// Used by Rust win_line phase functions to pass to get_prevcol_hl_flag
/// and get_search_match_hl.
void *nvim_get_screen_search_hl_ptr(void)
{
  return &screen_search_hl;
}

/// Update curwin w_cline_row/height/folded after drawing the cursor line.
/// Called by rs_win_line_eol_fill when in_curline is true.
void nvim_curwin_update_cline(int startrow, int row, bool has_fold)
{
  curwin->w_cline_row = startrow;
  curwin->w_cline_height = row - startrow;
  curwin->w_cline_folded = has_fold;
  curwin->w_valid |= VALID_CHEIGHT | VALID_CROW;
}

/// Invalidate the first column of the next row after a wrapped line.
/// Called by rs_win_line_end_check after wlv_put_linebuf with SLF_WRAP.
/// @param grid  GridView pointer (wp->w_grid)
/// @param row   current row (before wrap increment)
void nvim_grid_invalidate_next_row(void *grid, int row)
{
  int dummy_col = 0;
  ScreenGrid *g = grid_adjust((GridView *)grid, &row, &dummy_col);
  g->attrs[g->line_offset[row + 1]] = -1;
}

/// Get wp->w_grid.target->cols (the width of the grid target).
/// Used by rs_win_line_end_check to determine if wrapping applies.
int nvim_win_get_w_grid_target_cols(win_T *wp)
{
  return wp->w_grid.target->cols;
}

/// Get a pointer to virt_lines[idx].line (VirtText*) as void*.
/// Used by rs_win_line_end_check to pass to draw_virt_text_item.
/// @param virt_lines  pointer to VirtLines KVec
/// @param idx         index into virt_lines
void *nvim_virt_lines_get_line(void *virt_lines, int idx)
{
  VirtLines *vl = (VirtLines *)virt_lines;
  return &kv_A(*vl, (size_t)idx).line;
}

// =============================================================================
// Phase 1: win_loop accessors
// =============================================================================

/// Return true if wp->w_grid_alloc.chars is non-NULL.
bool nvim_win_get_grid_alloc_chars(win_T *wp) { return wp->w_grid_alloc.chars != NULL; }

/// Call grid_draw_border for wp->w_grid_alloc using wp's config fields.
/// Bundles the struct-heavy call for Rust FFI.
void nvim_win_draw_border(win_T *wp)
{
  grid_draw_border(&wp->w_grid_alloc, &wp->w_config, wp->w_border_adj, (int)wp->w_p_winbl,
                   wp->w_ns_hl_attr);
}

/// Invalidate wp->w_grid_alloc (sets CLEAR state for floating windows).
void nvim_win_grid_alloc_invalidate(win_T *wp)
{
  grid_invalidate(&wp->w_grid_alloc);
}

// =============================================================================
// Phase 2: nvim_win_visual_region_impl accessors
// =============================================================================

/// Return wp->w_lines[idx].wl_size.
int nvim_win_wlines_get_size(win_T *wp, int idx)
{
  return wp->w_lines[idx].wl_size;
}

/// Return curwin->w_p_lbr.
int nvim_curwin_get_w_p_lbr(void) { return curwin->w_p_lbr; }

/// Return curwin->w_ve_flags.
unsigned nvim_curwin_get_w_ve_flags(void) { return curwin->w_ve_flags; }

/// Set curwin->w_ve_flags.
void nvim_curwin_set_w_ve_flags(unsigned val) { curwin->w_ve_flags = val; }

/// Return curwin->w_curswant.
colnr_T nvim_curwin_get_curswant(void) { return curwin->w_curswant; }

/// Return true if buf == curwin->w_buffer.
bool nvim_buf_is_curwin_buf(buf_T *buf) { return buf == curwin->w_buffer; }

// =============================================================================
// Phase 3: win_redraw_signcols accessors
// =============================================================================

/// Return true if *wp->w_p_stc != NUL (statuscolumn option is set).
bool nvim_win_get_w_p_stc_nul(win_T *wp) { return *wp->w_p_stc != NUL; }


// =============================================================================
// Phase 1 (plan 132e8cc7): win_update accessor infrastructure
// =============================================================================

// --- w_lines[] / wline_T setters (getters already in window_shim.c) ---



// --- Global variable accessors ---

/// Get global got_int.
int nvim_get_got_int(void) { return got_int; }

// --- FOR_ALL_WINDOWS_IN_TAB signcols helper ---

/// Run win_redraw_signcols check for all windows sharing wp's buffer.
/// If any window needs redraw, calls changed_line_abv_curs_win + redraw_later.
/// Also updates buf->b_signcols.last_max = buf->b_signcols.max.
void nvim_win_update_signcols_for_tab(win_T *wp)
{
  buf_T *buf = wp->w_buffer;
  FOR_ALL_WINDOWS_IN_TAB(win, curtab) {
    if (win->w_buffer == buf && rs_win_redraw_signcols(win)) {
      changed_line_abv_curs_win(win);
      redraw_later(win, UPD_NOT_VALID);
    }
  }
  buf->b_signcols.last_max = buf->b_signcols.max;
}

/// Setup syntax timeout before win_update. Wraps profile_setlimit + syn_set_timeout.
/// @return opaque proftime_T as int64_t (safe if sizeof(proftime_T) <= 8).
void nvim_win_update_syn_timeout_start(win_T *wp)
{
  (void)wp;
  proftime_T syntax_tm = profile_setlimit(p_rdt);
  syn_set_timeout(&syntax_tm);
}

/// Call win_col_off for wp.
int nvim_win_col_off(win_T *wp) { return win_col_off(wp); }

/// Call win_col_off2 for wp.
int nvim_win_col_off2(win_T *wp) { return win_col_off2(wp); }

// nvim_buf_terminal_check_size is defined in buffer_shim.c

/// Call win_scroll_lines(wp, row, count).
void nvim_win_scroll_lines(win_T *wp, int row, int count) { win_scroll_lines(wp, row, count); }

/// Call win_lines_concealed(wp).
int nvim_win_lines_concealed(win_T *wp) { return win_lines_concealed(wp) ? 1 : 0; }

/// Call spell_check_window(wp).
int nvim_spell_check_window(win_T *wp) { return spell_check_window(wp) ? 1 : 0; }

// rs_foldmethodIsSyntax is defined in fold/src/lib.rs and exported from libnvim_rs.a

/// Call decor_virt_lines(wp, start, end, NULL, NULL, true) > 0.
int nvim_decor_virt_lines_check(win_T *wp, linenr_T start, linenr_T end)
{
  return decor_virt_lines(wp, start, end, NULL, NULL, true) > 0 ? 1 : 0;
}

// =============================================================================
// Phase 1 (plan 78e2a5ac): nvim_win_update_body init accessors
// =============================================================================

/// Return true if any match item in wp->w_match_head has a multiline regprog.
bool nvim_win_match_has_multiline_regprog(win_T *wp)
{
  const matchitem_T *cur = wp->w_match_head;
  while (cur != NULL) {
    if (cur->mit_match.regprog != NULL && re_multiline(cur->mit_match.regprog)) {
      return true;
    }
    cur = cur->mit_next;
  }
  return false;
}

/// Return true if screen_search_hl has a multiline regprog.
bool nvim_search_hl_is_multiline(void)
{
  return screen_search_hl.rm.regprog != NULL
         && re_multiline(screen_search_hl.rm.regprog);
}

/// Reset win_extmark_arr.size to 0.
void nvim_win_extmark_arr_reset(void)
{
  win_extmark_arr.size = 0;
}

/// Call decor_redraw_reset(wp, &decor_state).
void nvim_decor_redraw_reset_win(win_T *wp)
{
  decor_redraw_reset(wp, &decor_state);
}

/// Call decor_providers_invoke_win(wp).
void nvim_decor_providers_invoke_win_shim(win_T *wp)
{
  decor_providers_invoke_win(wp);
}

/// Call init_search_hl(wp, &screen_search_hl).
void nvim_init_search_hl_win(win_T *wp)
{
  init_search_hl(wp, &screen_search_hl);
}

/// Get buf->b_s.b_syn_sync_linebreaks.
linenr_T nvim_buf_get_syn_sync_linebreaks(buf_T *buf)
{
  return buf->b_s.b_syn_sync_linebreaks;
}

/// Get wp->w_lines[idx].wl_lnum.
linenr_T nvim_win_get_wlines_lnum(win_T *wp, int idx)
{
  return wp->w_lines[idx].wl_lnum;
}

/// Get wp->w_lines[idx].wl_lastlnum.
linenr_T nvim_win_get_wlines_lastlnum(win_T *wp, int idx)
{
  return wp->w_lines[idx].wl_lastlnum;
}

/// Get wp->w_lines[idx].wl_valid.
bool nvim_win_get_wlines_valid(win_T *wp, int idx)
{
  return wp->w_lines[idx].wl_valid;
}

/// Get wp->w_lines[idx].wl_size.
int nvim_win_get_wlines_size(win_T *wp, int idx)
{
  return (int)wp->w_lines[idx].wl_size;
}

/// Wrap hasFolding(wp, lnum, lo, hi). lo and hi are linenr_T pointers (may be NULL).
/// Returns true if lnum is in a fold.
bool nvim_hasFolding_win(win_T *wp, linenr_T lnum, linenr_T *lo, linenr_T *hi)
{
  return hasFolding(wp, lnum, lo, hi);
}

/// Call number_width(wp) and return.
int nvim_number_width(win_T *wp)
{
  return number_width(wp);
}

// =============================================================================
// Phase 2+3 (plan 78e2a5ac): scroll + draw loop + finalize accessors
// =============================================================================

/// Get wp->w_redr_statuscol.
bool nvim_win_get_redr_statuscol(win_T *wp) { return wp->w_redr_statuscol; }

/// Set wp->w_valid &= ~mask.
void nvim_win_and_w_valid(win_T *wp, int mask) { wp->w_valid &= (unsigned)mask; }

/// Set wp->w_valid |= mask.
void nvim_win_or_w_valid(win_T *wp, int mask) { wp->w_valid |= (unsigned)mask; }

/// Return true if wp->w_match_head != NULL.
bool nvim_win_get_match_head_nonnull(win_T *wp) { return wp->w_match_head != NULL; }

// nvim_buf_get_mod_xlines is in syntax_accessors.c

/// Return true if *wp->w_p_fdt == NUL.
bool nvim_win_get_w_p_fdt_nul(win_T *wp) { return *wp->w_p_fdt == NUL; }

/// Return raw pointer to wp->w_lines array (unchecked).
/// The caller must bounds-check using w_lines_size/w_view_height.
wline_T *nvim_win_get_wlines_ptr(win_T *wp) { return wp->w_lines; }

/// Send win_extmarks if needed (kv_size/kv_A loop + ui_call_win_extmark).
void nvim_win_send_extmarks(win_T *wp)
{
  for (size_t n = 0; n < kv_size(win_extmark_arr); n++) {
    ui_call_win_extmark(wp->w_grid_alloc.handle, wp->handle,
                        kv_A(win_extmark_arr, n).ns_id,
                        (Integer)kv_A(win_extmark_arr, n).mark_id,
                        kv_A(win_extmark_arr, n).win_row,
                        kv_A(win_extmark_arr, n).win_col);
  }
}

/// Reset state for statuscolumn restart inside the draw loop.
/// Resets idx=0, row=0, lnum=topline, w_lines_valid=0, w_valid &= ~VALID_WCOL,
/// then calls decor_redraw_reset + decor_providers_invoke_win.
void nvim_win_redr_statuscol_restart(win_T *wp)
{
  wp->w_redr_statuscol = false;
  wp->w_lines_valid = 0;
  wp->w_valid &= ~VALID_WCOL;
  decor_redraw_reset(wp, &decor_state);
  decor_providers_invoke_win(wp);
}

/// Call prepare_search_hl(wp, &screen_search_hl, lnum).
void nvim_prepare_search_hl_win(win_T *wp, linenr_T lnum)
{
  prepare_search_hl(wp, &screen_search_hl, lnum);
}

// nvim_plines_m_win is already in move.c

/// Call plines_correct_topline(wp, lnum, &lnum, true, NULL).
/// Advances lnum (concealed fold skipping) and returns number of screen lines.
int nvim_plines_correct_topline_adv(win_T *wp, linenr_T lnum, linenr_T *nextp)
{
  return plines_correct_topline(wp, lnum, nextp, true, NULL);
}

/// Call plines_win(wp, lnum, true) (convenience with limit=true).
int nvim_plines_win_true(win_T *wp, linenr_T lnum)
{
  return plines_win(wp, lnum, true);
}

// nvim_win_get_fill is already in plines.c

/// Call win_may_fill(wp).
bool nvim_win_may_fill(win_T *wp)
{
  return win_may_fill(wp);
}

/// Call curs_columns(curwin, true).
void nvim_curs_columns_curwin(void)
{
  curs_columns(curwin, true);
}

/// Set curbuf->b_mod_set.
void nvim_curbuf_set_mod_set(int val) { curbuf->b_mod_set = (val != 0); }

/// Get curbuf->b_mod_set.
int nvim_curbuf_get_mod_set(void) { return curbuf->b_mod_set ? 1 : 0; }

/// Return true if buf->terminal != NULL.
bool nvim_buf_has_terminal(buf_T *buf) { return buf->terminal != NULL; }

// nvim_buf_terminal_check_size is in buffer_shim.c

/// Set syn_set_timeout(NULL) (end syntax parsing timeout).
void nvim_syn_set_timeout_null(void) { syn_set_timeout(NULL); }

/// Zero got_int and set up syntax timeout.
/// Returns the proftime_T value as opaque int64 (not used by Rust; timeout ptr is stored in C).
/// IMPORTANT: The proftime_T is a local in the C function and MUST outlive the draw loop.
/// This function stores the timeout pointer in the syntax engine.
/// Call nvim_syn_set_timeout_null() to clear it after the draw loop.
void nvim_win_setup_syntax_tm(void)
{
  // The proftime_T is allocated on the heap so it outlives this call.
  // This is a workaround: the C original kept it as a local in the same function scope.
  // We use a static to keep it alive. Thread-safe since Neovim is single-threaded.
  static proftime_T syntax_tm;
  got_int = 0;
  syntax_tm = profile_setlimit(p_rdt);
  syn_set_timeout(&syntax_tm);
}

/// Call set_empty_rows(wp, srow).
void nvim_set_empty_rows_win(win_T *wp, int srow)
{
  set_empty_rows(wp, srow);
}

/// Call hl_combine_attr(a, b).
int nvim_hl_combine_attr(int a, int b) { return hl_combine_attr(a, b); }

/// Call win_bg_attr(wp).
int nvim_win_bg_attr(win_T *wp) { return win_bg_attr(wp); }

/// Return wp->w_p_fcs_chars.lastline (as schar_T / uint32_t).
uint32_t nvim_win_get_fcs_lastline(win_T *wp) { return (uint32_t)wp->w_p_fcs_chars.lastline; }

/// Return wp->w_p_fcs_chars.eob (as schar_T / uint32_t).
uint32_t nvim_win_get_fcs_eob(win_T *wp) { return (uint32_t)wp->w_p_fcs_chars.eob; }

/// Return dy_flags (as int/unsigned - uses existing nvim_get_dy_flags in window_shim.c).
/// nvim_get_dy_flags() is already defined in window_shim.c

// nvim_get_dollar_vcol is in edit.c

/// Return must_redraw.
int nvim_get_must_redraw(void) { return must_redraw; }

// nvim_set_must_redraw is already in window_shim.c (calls set_must_redraw)

// nvim_set_got_int is in terminal_shim.c
// nvim_update_topline_curwin is in eval_shim.c

