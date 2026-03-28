// drawscreen_shim.c: Minimal C accessors for the Rust drawscreen crate.

#include <stdbool.h>

#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/cmdexpand.h"
#include "nvim/drawscreen.h"
#include "nvim/ex_getln.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/grid_defs.h"
#include "nvim/highlight.h"
#include "nvim/insexpand.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/option_vars.h"
#include "nvim/popupmenu.h"
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
