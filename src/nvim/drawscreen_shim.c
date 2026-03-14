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
#include "nvim/cmdexpand.h"
#include "nvim/cursor.h"
#include "nvim/digraph.h"
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
#include "nvim/state_defs.h"
#include "nvim/statusline.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_compositor.h"
#include "nvim/version.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

// Rust FFI declarations needed by the shim
extern int rs_min_rows(tabpage_T *tp);  // NOLINT(readability-redundant-declaration)
extern int rs_cmdline_number_prompt(void);  // NOLINT(readability-redundant-declaration)

// Accessors for drawscreen.c file-statics (defined in drawscreen.c)
extern void nvim_drawscreen_set_redraw_popupmenu(int val);
extern int nvim_drawscreen_get_redraw_popupmenu(void);
extern void nvim_drawscreen_set_msg_grid_invalid(int val);
extern void nvim_drawscreen_set_resizing_autocmd(int val);

#include "drawscreen_shim.c.generated.h"

// ---------------------------------------------------------------------------
// showmode helpers
// ---------------------------------------------------------------------------

/// Call msg_grid_validate() (not in public headers).
void nvim_drawscreen_msg_grid_validate(void)
{
  msg_grid_validate();
}

/// Call msg_check_for_delay(false).
void nvim_drawscreen_msg_check_for_delay(void)
{
  msg_check_for_delay(false);
}

/// Call msg_clr_cmdline() (already declared as nvim_msg_clr_cmdline_wrap in
/// insexpand_shim.c; provide a second alias here for use by drawscreen).
void nvim_drawscreen_msg_clr_cmdline(void)
{
  msg_clr_cmdline();
}

/// Return 1 if edit_submode is NULL, else 0.
int nvim_drawscreen_edit_submode_is_null(void)
{
  return edit_submode == NULL ? 1 : 0;
}

/// Return the edit_submode pointer (may be NULL; Rust must not free it).
const char *nvim_drawscreen_edit_submode_ptr(void)
{
  return edit_submode;
}

/// Return 1 if edit_submode_pre is NULL, else 0.
int nvim_drawscreen_edit_submode_pre_is_null(void)
{
  return edit_submode_pre == NULL ? 1 : 0;
}

/// Return the edit_submode_pre pointer (may be NULL; Rust must not free it).
const char *nvim_drawscreen_edit_submode_pre_ptr(void)
{
  return edit_submode_pre;
}

/// Call get_keymap_str(curwin, " (%s)", NameBuff, MAXPATHL).
/// Returns the result length (>0 means something was written to NameBuff).
int nvim_drawscreen_get_keymap_str(void)
{
  return get_keymap_str(curwin, " (%s)", NameBuff, MAXPATHL);
}

/// Return the NameBuff pointer (global char buffer, valid until next C call).
const char *nvim_drawscreen_namebuff_ptr(void)
{
  return NameBuff;
}

/// Return wp->w_p_arab (arabic option).
int nvim_win_get_w_p_arab(win_T *wp)
{
  return wp ? (int)wp->w_p_arab : 0;
}

// ---------------------------------------------------------------------------
// Phase 3: default_grid_alloc, screenclear, screen_resize batch helpers
// ---------------------------------------------------------------------------

/// Return default_grid.rows.
int nvim_default_grid_get_rows(void)
{
  return default_grid.rows;
}

/// Return default_grid.cols.
int nvim_default_grid_get_cols(void)
{
  return default_grid.cols;
}

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

/// Perform the grid_alloc + click_defs + field setup for default_grid.
/// Equivalent to the core of default_grid_alloc() after the guard checks.
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
/// Called from rs_screenclear() in Rust.
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
  nvim_drawscreen_set_redraw_popupmenu(1);
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
    nvim_drawscreen_set_msg_grid_invalid(0);
    clear_cmdline = true;
  }
}

/// Perform screen_resize operations. Called from rs_screen_resize() in Rust.
/// Handles all guards (recursion, HITRETURN/SETWSIZE, negative dimensions).
void nvim_screen_resize_impl(int width, int height)
{
  // Avoid recursiveness, can happen when setting the window size causes
  // another window-changed signal.
  if (updating_screen || resizing_screen || rs_cmdline_number_prompt() != 0) {
    return;
  }

  if (width < 0 || height < 0) {    // just checking...
    return;
  }

  if (State == MODE_HITRETURN || State == MODE_SETWSIZE) {
    // postpone the resizing
    State = MODE_SETWSIZE;
    return;
  }

  resizing_screen = true;

  Rows = height;
  Columns = width;
  check_screensize();
  if (!ui_has(kUIMessages)) {
    // clamp 'cmdheight'
    int max_p_ch = Rows - rs_min_rows(curtab) + 1;
    if (p_ch > 0 && p_ch > max_p_ch) {
      p_ch = MAX(max_p_ch, 1);
      curtab->tp_ch_used = p_ch;
    }
    // clamp 'cmdheight' for other tab pages
    FOR_ALL_TABS(tp) {
      if (tp == curtab) {
        continue;  // already set above
      }
      int max_tp_ch = Rows - rs_min_rows(tp) + 1;
      if (tp->tp_ch_used > 0 && tp->tp_ch_used > max_tp_ch) {
        tp->tp_ch_used = MAX(max_tp_ch, 1);
      }
    }
  }
  height = Rows;
  width = Columns;
  p_lines = Rows;
  p_columns = Columns;

  ui_call_grid_resize(1, width, height);

  int retry_count = 0;
  nvim_drawscreen_set_resizing_autocmd(1);

  // In rare cases, autocommands may have altered Rows or Columns,
  // so retry to check if we need to allocate the screen again.
  while (default_grid_alloc()) {
    // win_new_screensize will recompute floats position, but tell the
    // compositor to not redraw them yet
    ui_comp_set_screen_valid(false);
    if (msg_grid.chars) {
      nvim_drawscreen_set_msg_grid_invalid(1);
    }

    RedrawingDisabled++;

    win_new_screensize();      // fit the windows in the new sized screen

    comp_col();           // recompute columns for shown command and ruler

    RedrawingDisabled--;

    // Do not apply autocommands more than 3 times to avoid an endless loop
    // in case applying autocommands always changes Rows or Columns.
    if (++retry_count > 3) {
      break;
    }

    apply_autocmds(EVENT_VIMRESIZED, NULL, NULL, false, curbuf);
  }

  nvim_drawscreen_set_resizing_autocmd(0);
  redraw_all_later(UPD_CLEAR);

  if (State != MODE_ASKMORE && State != MODE_EXTERNCMD) {
    screenclear();
  }

  if (starting != NO_SCREEN) {
    maketitle();

    changed_line_abv_curs();
    invalidate_botline(curwin);

    // We only redraw when it's needed:
    // - While at the more prompt or executing an external command, don't
    //   redraw, but position the cursor.
    // - While editing the command line, only redraw that. TODO: lies
    // - in Ex mode, don't redraw anything.
    // - Otherwise, redraw right now, and position the cursor.
    if (State == MODE_ASKMORE || State == MODE_EXTERNCMD || exmode_active
        || ((State & MODE_CMDLINE) && get_cmdline_info()->one_key)) {
      if (State & MODE_CMDLINE) {
        update_screen();
      }
      if (msg_grid.chars) {
        msg_grid_validate();
      }
      // TODO(bfredl): sometimes messes up the output. Implement clear+redraw
      // also for the pager? (or: what if the pager was just a modal window?)
      ui_comp_set_screen_valid(true);
      repeat_message();
    } else {
      if (curwin->w_p_scb) {
        do_check_scrollbind(true);
      }
      if (State & MODE_CMDLINE) {
        nvim_drawscreen_set_redraw_popupmenu(0);
        update_screen();
        redrawcmdline();
        if (pum_drawn()) {
          cmdline_pum_display(false);
        }
      } else {
        update_topline(curwin);
        if (pum_drawn()) {
          // TODO(bfredl): rs_ins_compl_show_pum wants to redraw the screen first.
          // For now make sure the nested update_screen() won't redraw the
          // pum at the old position. Try to untangle this later.
          nvim_drawscreen_set_redraw_popupmenu(0);
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
  resizing_screen = false;
}
