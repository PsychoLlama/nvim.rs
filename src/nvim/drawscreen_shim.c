// drawscreen_shim.c: C accessor wrappers for the Rust drawscreen crate.
//
// These are thin wrappers that give Rust access to globals and functions
// from drawscreen.c and related modules.

#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand.h"
#include "nvim/cursor.h"
#include "nvim/decoration.h"
#include "nvim/decoration_provider.h"
#include "nvim/diff.h"
#include "nvim/digraph.h"
#include "nvim/drawscreen.h"
#include "nvim/ex_getln.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/grid_defs.h"
#include "nvim/highlight.h"
#include "nvim/highlight_group.h"
#include "nvim/insexpand.h"
#include "nvim/memline.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/popupmenu.h"
#include "nvim/plines.h"
#include "nvim/search.h"
#include "nvim/state_defs.h"
#include "nvim/statusline.h"
#include "nvim/strings.h"
#include "nvim/syntax.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_compositor.h"
#include "nvim/version.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

// Rust FFI declarations needed by the shim
extern int rs_cmdline_number_prompt(void);  // NOLINT(readability-redundant-declaration)
extern int rs_global_stl_height(void);  // NOLINT(readability-redundant-declaration)
extern bool redrawing(void);  // NOLINT(readability-redundant-declaration)

// Accessors for drawscreen.c file-statics (defined in drawscreen.c)
extern void nvim_drawscreen_set_redraw_popupmenu(int val);
extern int nvim_drawscreen_get_redraw_popupmenu(void);
extern void nvim_drawscreen_set_msg_grid_invalid(int val);
extern int nvim_drawscreen_get_msg_grid_invalid(void);
extern void nvim_drawscreen_set_resizing_autocmd(int val);
extern int nvim_drawscreen_get_resizing_autocmd(void);
extern int nvim_default_grid_get_valid(void);
extern void nvim_default_grid_set_valid(int val);
extern void nvim_win_update(win_T *wp);

#include "drawscreen_shim.c.generated.h"

// default_grid_alloc, screenclear, screen_resize batch helpers

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



// update_screen batch helper

/// Full implementation of update_screen(), called from rs_update_screen() in Rust.
/// Manages the `still_may_intro` static locally.
/// Returns OK (0) on success, FAIL (1) on early exit.
int nvim_update_screen_impl(void)
{
  static bool still_may_intro = true;
  if (still_may_intro) {
    if (!may_show_intro()) {
      redraw_later(firstwin, UPD_NOT_VALID);
      still_may_intro = false;
    }
  }

  bool is_stl_global = rs_global_stl_height() > 0;

  // Don't do anything if the screen structures are (not yet) valid.
  if (nvim_drawscreen_get_resizing_autocmd() || !default_grid.chars) {
    return FAIL;
  }

  // May have postponed updating diffs.
  if (need_diff_redraw) {
    diff_redraw(true);
  }

  // Postpone redrawing when not needed or called recursively.
  if (!redrawing() || updating_screen || rs_cmdline_number_prompt() != 0) {
    return FAIL;
  }

  int type = must_redraw;
  must_redraw = 0;
  updating_screen = true;

  display_tick++;

  if (schar_cache_clear_if_full()) {
    type = MAX(type, UPD_CLEAR);
  }

  if (msg_did_scroll) {
    msg_did_scroll = false;
    msg_scrolled_at_flush = 0;
  }

  if (type >= UPD_CLEAR || !nvim_default_grid_get_valid()) {
    ui_comp_set_screen_valid(false);
  }

  if (msg_scrolled || nvim_drawscreen_get_msg_grid_invalid()) {
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
    nvim_drawscreen_set_msg_grid_invalid(0);
    if (was_invalidated) {
      ui_comp_set_screen_valid(true);
    }
    msg_scrolled = 0;
    msg_scrolled_at_flush = 0;
    msg_grid_scroll_discount = 0;
    need_wait_return = false;
  }

  win_ui_flush(true);
  compute_cmdrow();

  bool hl_changed = false;
  if (need_highlight_changed) {
    highlight_changed();
    hl_changed = true;
  }

  if (type == UPD_CLEAR) {
    screenclear();
    cmdline_screen_cleared();
    if (ui_has(kUIMessages)) {
      ui_call_msg_clear();
    }
    type = UPD_NOT_VALID;
    must_redraw = 0;
  } else if (!nvim_default_grid_get_valid()) {
    grid_invalidate(&default_grid);
    nvim_default_grid_set_valid(1);
  }

  if (type == UPD_NOT_VALID && clear_cmdline && !ui_has(kUIMessages)) {
    grid_clear(&default_gridview, Rows - (int)p_ch, Rows, 0, Columns, 0);
  }

  ui_comp_set_screen_valid(true);

  decor_providers_start();

  if (win_check_ns_hl(NULL)) {
    redraw_cmdline = true;
    redraw_tabline = true;
  }

  if (clear_cmdline) {
    msg_check_for_delay(false);
  }

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

  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    update_window_hl(wp, type >= UPD_NOT_VALID || hl_changed);

    buf_T *buf = wp->w_buffer;
    if (buf->b_mod_set) {
      if (buf->b_mod_tick_syn < display_tick
          && syntax_present(wp)) {
        syn_stack_apply_changes(buf);
        buf->b_mod_tick_syn = display_tick;
      }

      if (buf->b_mod_tick_decor < display_tick) {
        decor_providers_invoke_buf(buf);
        buf->b_mod_tick_decor = display_tick;
      }
    }
  }

  bool did_one = false;
  screen_search_hl.rm.regprog = NULL;

  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (wp->w_redr_type == UPD_CLEAR && wp->w_floating && wp->w_grid_alloc.chars) {
      grid_invalidate(&wp->w_grid_alloc);
      wp->w_redr_type = UPD_NOT_VALID;
    }

    win_check_ns_hl(wp);
    win_grid_alloc(wp);

    if (wp->w_redr_border || wp->w_redr_type >= UPD_NOT_VALID) {
      grid_draw_border(&wp->w_grid_alloc, &wp->w_config, wp->w_border_adj, (int)wp->w_p_winbl,
                       wp->w_ns_hl_attr);
    }

    if (wp->w_redr_type != 0) {
      if (!did_one) {
        did_one = true;
        start_search_hl();
      }
      nvim_win_update(wp);
    }

    if (wp->w_redr_status) {
      win_redr_winbar(wp);
      win_redr_status(wp);
    }
  }

  end_search_hl();

  if (pum_drawn() && must_redraw_pum) {
    win_check_ns_hl(curwin);
    pum_redraw();
  }

  win_check_ns_hl(NULL);

  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    wp->w_buffer->b_mod_set = false;
  }

  updating_screen = false;

  if (need_maketitle) {
    maketitle();
  }

  if (clear_cmdline || redraw_cmdline || redraw_mode) {
    showmode();
  }

  if (still_may_intro) {
    intro_message(false);
  }
  repeat_message();

  decor_providers_invoke_end();

  if (!ui_has(kUICmdline)) {
    cmdline_was_last_drawn = false;
  }
  return OK;
}

bool nvim_get_updating_screen(void) { return updating_screen; }
