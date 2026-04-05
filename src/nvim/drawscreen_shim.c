// drawscreen_shim.c: Minimal C accessors for the Rust drawscreen crate.

#include <stdbool.h>

#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/decoration_defs.h"
#include "nvim/cmdexpand.h"
#include "nvim/drawscreen.h"
#include "nvim/ex_getln.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/grid_defs.h"
#include "nvim/highlight.h"
#include "nvim/insexpand.h"
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
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_compositor.h"
#include "nvim/ui_defs.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

// file-static variables owned in drawscreen.c, used in these helpers
extern bool redraw_popupmenu;
extern bool msg_grid_invalid;
extern bool resizing_autocmd;

// Rust FFI declarations used in shim helpers
extern void rs_ins_compl_show_pum(void);
extern int rs_min_rows(tabpage_T *tp);

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
