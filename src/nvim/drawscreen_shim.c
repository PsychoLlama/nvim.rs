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

/// Handle post-resize redraw. Called from rs_drawscreen_screen_resize() in Rust.
/// @param state  Current editor state (State global value at call time).
void nvim_screen_resize_post(int state)
{
  if (starting == NO_SCREEN) {
    return;
  }

  maketitle();
  changed_line_abv_curs();
  invalidate_botline(curwin);

  if (state == MODE_ASKMORE || state == MODE_EXTERNCMD || exmode_active
      || ((state & MODE_CMDLINE) && get_cmdline_info()->one_key)) {
    if (state & MODE_CMDLINE) {
      update_screen();
    }
    if (msg_grid.chars) {
      msg_grid_validate();
    }
    ui_comp_set_screen_valid(true);
    repeat_message();
  } else {
    if (curwin->w_p_scb) {
      do_check_scrollbind(true);
    }
    if (state & MODE_CMDLINE) {
      redraw_popupmenu = false;
      update_screen();
      redrawcmdline();
      if (pum_drawn()) {
        cmdline_pum_display(false);
      }
    } else {
      update_topline(curwin);
      if (pum_drawn()) {
        redraw_popupmenu = false;
        rs_ins_compl_show_pum();
      }
      update_screen();
      if (redrawing()) {
        setcursor();
      }
    }
  }
  ui_flush();
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

/// Get buf->b_mod_tick_syn (display tick when syntax was last updated).
uint64_t nvim_buf_get_mod_tick_syn(buf_T *buf) { return buf->b_mod_tick_syn; }

/// Set buf->b_mod_tick_syn.
void nvim_buf_set_mod_tick_syn(buf_T *buf, uint64_t val) { buf->b_mod_tick_syn = val; }

/// Get buf->b_mod_tick_decor (display tick when decoration providers were last invoked).
uint64_t nvim_buf_get_mod_tick_decor(buf_T *buf) { return buf->b_mod_tick_decor; }

/// Set buf->b_mod_tick_decor.
void nvim_buf_set_mod_tick_decor(buf_T *buf, uint64_t val) { buf->b_mod_tick_decor = val; }

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

/// Return ml_get_buf_len(buf, lnum).
colnr_T nvim_buf_ml_get_len(buf_T *buf, linenr_T lnum)
{
  return ml_get_buf_len(buf, lnum);
}

// =============================================================================
// Phase 3: win_redraw_signcols accessors
// =============================================================================

/// Return true if *wp->w_p_stc != NUL (statuscolumn option is set).
bool nvim_win_get_w_p_stc_nul(win_T *wp) { return *wp->w_p_stc != NUL; }

/// Compute getvcols(&VIsual, &curwin->w_cursor) for block visual mode.
/// Returns fromc via *fromc_out and pre-incremented toc via *toc_out.
/// Also handles MAXCOL curswant block expansion.
/// This is the core of the block-visual column range calculation.
void nvim_win_visual_block_cols(win_T *wp, colnr_T *fromc_out, colnr_T *toc_out)
{
  colnr_T fromc, toc;
  unsigned save_ve_flags = curwin->w_ve_flags;

  if (curwin->w_p_lbr) {
    curwin->w_ve_flags = kOptVeFlagAll;
  }

  getvcols(wp, &VIsual, &curwin->w_cursor, &fromc, &toc);
  toc++;
  curwin->w_ve_flags = save_ve_flags;

  if (curwin->w_curswant == MAXCOL) {
    if (get_ve_flags(curwin) & kOptVeFlagBlock) {
      pos_T pos;
      int cursor_above = curwin->w_cursor.lnum < VIsual.lnum;

      toc = 0;
      pos.coladd = 0;
      for (pos.lnum = curwin->w_cursor.lnum;
           cursor_above ? pos.lnum <= VIsual.lnum : pos.lnum >= VIsual.lnum;
           pos.lnum += cursor_above ? 1 : -1) {
        colnr_T t;

        pos.col = ml_get_buf_len(wp->w_buffer, pos.lnum);
        getvvcol(wp, &pos, NULL, NULL, &t);
        toc = MAX(toc, t);
      }
      toc++;
    } else {
      toc = MAXCOL;
    }
  }

  *fromc_out = fromc;
  *toc_out = toc;
}

// =============================================================================
// Phase 1 (plan 132e8cc7): win_update accessor infrastructure
// =============================================================================

// --- w_lines[] / wline_T setters (getters already in window_shim.c) ---

/// Unchecked access to wp->w_lines[idx] (no bounds check).
/// Used by win_update which may access beyond w_lines_valid.
wline_T *nvim_win_wlines_get_ptr(win_T *wp, int idx)
{
  return &wp->w_lines[idx];
}

/// Copy wp->w_lines[dst_idx] = wp->w_lines[src_idx].
void nvim_win_wlines_copy(win_T *wp, int dst_idx, int src_idx)
{
  wp->w_lines[dst_idx] = wp->w_lines[src_idx];
}

/// Set wp->w_lines[idx].wl_lnum.
void nvim_wline_set_lnum(wline_T *wl, linenr_T val) { wl->wl_lnum = val; }

/// Set wp->w_lines[idx].wl_lastlnum.
void nvim_wline_set_lastlnum(wline_T *wl, linenr_T val) { wl->wl_lastlnum = val; }

/// Set wp->w_lines[idx].wl_foldend.
void nvim_wline_set_foldend(wline_T *wl, linenr_T val) { wl->wl_foldend = val; }

/// Set wp->w_lines[idx].wl_folded.
void nvim_wline_set_folded(wline_T *wl, bool val) { wl->wl_folded = val; }

/// Set wp->w_lines[idx].wl_valid.
void nvim_wline_set_valid(wline_T *wl, bool val) { wl->wl_valid = val; }

/// Set wp->w_lines[idx].wl_size.
void nvim_wline_set_size(wline_T *wl, uint16_t val) { wl->wl_size = val; }

// --- Window field accessors ---


/// Get wp->w_old_botfill.
int nvim_win_get_old_botfill(win_T *wp) { return wp->w_old_botfill ? 1 : 0; }

/// Set wp->w_old_botfill.
void nvim_win_set_old_botfill(win_T *wp, int val) { wp->w_old_botfill = (val != 0); }

/// Get wp->w_old_topfill.
int nvim_win_get_old_topfill(win_T *wp) { return wp->w_old_topfill; }

/// Set wp->w_old_topfill.
void nvim_win_set_old_topfill(win_T *wp, int val) { wp->w_old_topfill = val; }


/// Get wp->w_upd_rows.
int nvim_win_get_upd_rows(win_T *wp) { return wp->w_upd_rows; }


/// Get wp->w_p_fdt (pointer), check if it is NUL.
int nvim_win_get_p_fdt_nul(win_T *wp) { return *wp->w_p_fdt == NUL ? 1 : 0; }

/// Check if wp->w_match_head is non-NULL.
int nvim_win_has_match_head(win_T *wp) { return wp->w_match_head != NULL ? 1 : 0; }

/// Get wp->w_p_fcs_chars.lastline (the 'lastline' fillchar).
uint32_t nvim_win_get_fcs_lastline(win_T *wp) { return (uint32_t)wp->w_p_fcs_chars.lastline; }

/// Get wp->w_p_fcs_chars.eob (the 'eob' fillchar).
uint32_t nvim_win_get_fcs_eob(win_T *wp) { return (uint32_t)wp->w_p_fcs_chars.eob; }

/// Get &wp->w_grid as a GridView* (opaque pointer for Rust).
void *nvim_win_get_grid_view(win_T *wp) { return &wp->w_grid; }


// --- Global variable accessors ---

/// Get the global search_hl_has_cursor_lnum.
linenr_T nvim_get_search_hl_cursor_lnum(void) { return search_hl_has_cursor_lnum; }

/// Set the global search_hl_has_cursor_lnum.
void nvim_set_search_hl_cursor_lnum(linenr_T val) { search_hl_has_cursor_lnum = val; }

/// Get global got_int.
int nvim_get_got_int(void) { return got_int; }


/// Get p_rdt (redrawtime option).
int64_t nvim_get_p_rdt(void) { return p_rdt; }


/// Get kOptDyFlagLastline constant.
unsigned nvim_get_kOptDyFlagLastline(void) { return kOptDyFlagLastline; }

/// Get kOptDyFlagTruncate constant.
unsigned nvim_get_kOptDyFlagTruncate(void) { return kOptDyFlagTruncate; }

/// Get VALID_BOTLINE constant.
int nvim_get_VALID_BOTLINE(void) { return VALID_BOTLINE; }

/// Get VALID_TOPLINE constant.
int nvim_get_VALID_TOPLINE(void) { return VALID_TOPLINE; }

/// Get VALID_WCOL constant.
int nvim_get_VALID_WCOL(void) { return VALID_WCOL; }

/// Set wp->w_valid |= VALID_BOTLINE.
void nvim_win_set_valid_botline(win_T *wp) { wp->w_valid |= VALID_BOTLINE; }

/// Clear VALID_TOPLINE from wp->w_valid.
void nvim_win_clear_valid_topline(win_T *wp) { wp->w_valid &= ~VALID_TOPLINE; }

/// Clear VALID_WCOL from wp->w_valid.
void nvim_win_clear_valid_wcol(win_T *wp) { wp->w_valid &= ~VALID_WCOL; }

// --- win_extmark_arr accessors ---

/// Get size of win_extmark_arr.
size_t nvim_win_extmark_arr_size(void) { return kv_size(win_extmark_arr); }

/// Clear win_extmark_arr (set size to 0).
void nvim_win_extmark_arr_clear(void) { win_extmark_arr.size = 0; }

/// Get ns_id of win_extmark_arr[n].
uint64_t nvim_win_extmark_arr_get_ns_id(size_t n) { return (uint64_t)kv_A(win_extmark_arr, n).ns_id; }

/// Get mark_id of win_extmark_arr[n].
uint64_t nvim_win_extmark_arr_get_mark_id(size_t n)
{
  return kv_A(win_extmark_arr, n).mark_id;
}

/// Get win_row of win_extmark_arr[n].
int nvim_win_extmark_arr_get_win_row(size_t n) { return kv_A(win_extmark_arr, n).win_row; }

/// Get win_col of win_extmark_arr[n].
int nvim_win_extmark_arr_get_win_col(size_t n) { return kv_A(win_extmark_arr, n).win_col; }

/// Call ui_call_win_extmark for win_extmark_arr[n].
void nvim_win_extmark_dispatch(win_T *wp, size_t n)
{
  ui_call_win_extmark(wp->w_grid_alloc.handle, wp->handle,
                      (NS)kv_A(win_extmark_arr, n).ns_id,
                      (Integer)kv_A(win_extmark_arr, n).mark_id,
                      kv_A(win_extmark_arr, n).win_row,
                      kv_A(win_extmark_arr, n).win_col);
}

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

/// Reset syntax timeout after win_update. Wraps syn_set_timeout(NULL).
void nvim_win_update_syn_timeout_end(void) { syn_set_timeout(NULL); }

/// Call decor_redraw_reset for the window (uses global decor_state).
void nvim_decor_redraw_reset(win_T *wp) { decor_redraw_reset(wp, &decor_state); }

/// Call decor_providers_invoke_win for the window.
void nvim_decor_providers_invoke_win_wrapper(win_T *wp) { decor_providers_invoke_win(wp); }

/// Call init_search_hl for the window.
void nvim_init_search_hl(win_T *wp) { init_search_hl(wp, &screen_search_hl); }

/// Check re_multiline on screen_search_hl.rm.regprog.
int nvim_search_hl_is_multiline(void)
{
  return (screen_search_hl.rm.regprog != NULL
          && re_multiline(screen_search_hl.rm.regprog)) ? 1 : 0;
}

/// Check if any match in wp->w_match_head uses a multiline pattern.
int nvim_win_match_has_multiline(win_T *wp)
{
  const matchitem_T *cur = wp->w_match_head;
  while (cur != NULL) {
    if (cur->mit_match.regprog != NULL && re_multiline(cur->mit_match.regprog)) {
      return 1;
    }
    cur = cur->mit_next;
  }
  return 0;
}


/// Get buf->b_mod_set.
int nvim_buf_get_mod_set_direct(buf_T *buf) { return buf->b_mod_set ? 1 : 0; }


/// Get buf->b_s.b_syn_sync_linebreaks.
linenr_T nvim_buf_get_syn_sync_linebreaks(buf_T *buf)
{
  return buf->b_s.b_syn_sync_linebreaks;
}


/// Call win_col_off for wp.
int nvim_win_col_off(win_T *wp) { return win_col_off(wp); }

/// Call win_col_off2 for wp.
int nvim_win_col_off2(win_T *wp) { return win_col_off2(wp); }

/// Call number_width for wp.
int nvim_win_number_width(win_T *wp) { return number_width(wp); }

/// Call syntax_present for wp.
int nvim_syntax_present(win_T *wp) { return syntax_present(wp) ? 1 : 0; }

/// Get wp->w_cursor.lnum.
linenr_T nvim_win_get_cursor_lnum_direct(win_T *wp) { return wp->w_cursor.lnum; }

/// Get curbuf->b_mod_set.
int nvim_curbuf_get_mod_set(void) { return curbuf->b_mod_set ? 1 : 0; }

/// Set curbuf->b_mod_set.
void nvim_curbuf_set_mod_set(int val) { curbuf->b_mod_set = (val != 0); }

/// Get curwin->w_valid.
int nvim_curwin_get_valid(void) { return curwin->w_valid; }

/// Set curwin->w_valid &= ~VALID_TOPLINE.
void nvim_curwin_clear_valid_topline(void) { curwin->w_valid &= ~VALID_TOPLINE; }

/// Get must_redraw.
int nvim_get_must_redraw(void) { return must_redraw; }

/// Set must_redraw.
void nvim_set_must_redraw_direct(int val) { must_redraw = val; }

/// Call curs_columns(curwin, true).
void nvim_curs_columns_curwin(void) { curs_columns(curwin, true); }

/// Check if buf has a terminal (buf->terminal != NULL).
int nvim_buf_has_terminal(buf_T *buf) { return buf->terminal != NULL ? 1 : 0; }

// nvim_buf_terminal_check_size is defined in buffer_shim.c

/// Call hasFolding(wp, mod_top, &mod_top, NULL).
void nvim_hasFolding_mod_top(win_T *wp, linenr_T *lnum) { hasFolding(wp, *lnum, lnum, NULL); }

/// Call hasFolding(wp, lnum, NULL, &lnum).
void nvim_hasFolding_mod_bot(win_T *wp, linenr_T *lnum) { hasFolding(wp, *lnum, NULL, lnum); }

/// Call hasFolding(wp, lnum, NULL, &lnum) for topline_conceal loop.
void nvim_hasFolding_topline_conceal(win_T *wp, linenr_T *lnum)
{
  hasFolding(wp, *lnum, NULL, lnum);
}


/// Call win_scroll_lines(wp, row, count).
void nvim_win_scroll_lines(win_T *wp, int row, int count) { win_scroll_lines(wp, row, count); }

/// Call win_lines_concealed(wp).
int nvim_win_lines_concealed(win_T *wp) { return win_lines_concealed(wp) ? 1 : 0; }


/// Call win_may_fill(wp).
int nvim_win_may_fill(win_T *wp) { return win_may_fill(wp) ? 1 : 0; }

/// Call set_empty_rows(wp, srow).
void nvim_set_empty_rows(win_T *wp, int srow) { set_empty_rows(wp, srow); }

/// Get wp->w_p_stc pointer (for checking if statuscolumn is set).
int nvim_win_p_stc_nonempty(win_T *wp) { return *wp->w_p_stc != NUL ? 1 : 0; }

/// Call spell_check_window(wp).
int nvim_spell_check_window(win_T *wp) { return spell_check_window(wp) ? 1 : 0; }

/// Call syntax_end_parsing(wp, lnum + 1).
void nvim_syntax_end_parsing(win_T *wp, linenr_T lnum) { syntax_end_parsing(wp, lnum); }

/// Call compute_foldcolumn(wp, 0).
int nvim_compute_foldcolumn(win_T *wp) { return compute_foldcolumn(wp, 0); }


/// Get dollar_vcol >= 0 check.
int nvim_dollar_vcol_nonneg(void) { return dollar_vcol >= 0 ? 1 : 0; }

/// Check syntax state: syntax_check_changed(lnum).
int nvim_syntax_check_changed(linenr_T lnum) { return syntax_check_changed(lnum) ? 1 : 0; }

// rs_foldmethodIsSyntax is defined in fold/src/lib.rs and exported from libnvim_rs.a

/// Call win_draw_end for the window.
void nvim_win_draw_end_lastline(win_T *wp, uint32_t fcs_char, bool draw_margin,
                                int startrow, int endrow, int hlf)
{
  win_draw_end(wp, (schar_T)fcs_char, draw_margin, startrow, endrow, (hlf_T)hlf);
}

/// Call hl_combine_attr(win_bg_attr(wp), win_hl_attr(wp, HLF_AT)).
int nvim_win_at_attr(win_T *wp)
{
  return hl_combine_attr(win_bg_attr(wp), win_hl_attr(wp, HLF_AT));
}

/// Call grid_line_start(&wp->w_grid, row).
void nvim_win_grid_line_start(win_T *wp, int row) { grid_line_start(&wp->w_grid, row); }

/// Call grid_line_fill(start, end, schar, attr).
void nvim_win_grid_line_fill(win_T *wp, int start, int end, uint32_t schar, int attr)
{
  (void)wp;
  grid_line_fill(start, end, (schar_T)schar, attr);
}

/// Call grid_line_getchar(col, NULL).
uint32_t nvim_win_grid_line_getchar(int col)
{
  return (uint32_t)grid_line_getchar(col, NULL);
}

/// Call grid_line_flush().
void nvim_win_grid_line_flush(void) { grid_line_flush(); }

/// Call schar_from_ascii(' ').
uint32_t nvim_schar_from_space(void) { return (uint32_t)schar_from_ascii(' '); }

/// Call prepare_search_hl(wp, &screen_search_hl, lnum).
void nvim_prepare_search_hl(win_T *wp, linenr_T lnum)
{
  prepare_search_hl(wp, &screen_search_hl, lnum);
}

/// Call win_check_ns_hl(wp).
void nvim_win_check_ns_hl_wrapper(win_T *wp) { win_check_ns_hl(wp); }

/// Call decor_virt_lines(wp, start, end, NULL, NULL, true) > 0.
int nvim_decor_virt_lines_check(win_T *wp, linenr_T start, linenr_T end)
{
  return decor_virt_lines(wp, start, end, NULL, NULL, true) > 0 ? 1 : 0;
}
