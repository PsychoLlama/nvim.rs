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
extern int rs_min_rows(tabpage_T *tp);  // NOLINT(readability-redundant-declaration)
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
// default_grid_alloc, screenclear, screen_resize batch helpers
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

// ---------------------------------------------------------------------------
// win_update visual region helper
// ---------------------------------------------------------------------------

/// Full implementation of the visual region update section of win_update().
/// Called from rs_win_update_visual_region() in Rust.
void nvim_win_visual_region_impl(win_T *wp, buf_T *buf, int type,
                                 int top_end, bool scrolled_down,
                                 int *mid_start, int *mid_end)
{
  // check if we are updating or removing the inverted part
  if ((VIsual_active && buf == curwin->w_buffer)
      || (wp->w_old_cursor_lnum != 0 && type != UPD_NOT_VALID)) {
    linenr_T from, to;

    if (VIsual_active) {
      if (VIsual_mode != wp->w_old_visual_mode || type == UPD_INVERTED_ALL) {
        if (curwin->w_cursor.lnum < VIsual.lnum) {
          from = curwin->w_cursor.lnum;
          to = VIsual.lnum;
        } else {
          from = VIsual.lnum;
          to = curwin->w_cursor.lnum;
        }
        from = MIN(MIN(from, wp->w_old_cursor_lnum), wp->w_old_visual_lnum);
        to = MAX(MAX(to, wp->w_old_cursor_lnum), wp->w_old_visual_lnum);
      } else {
        if (curwin->w_cursor.lnum < wp->w_old_cursor_lnum) {
          from = curwin->w_cursor.lnum;
          to = wp->w_old_cursor_lnum;
        } else {
          from = wp->w_old_cursor_lnum;
          to = curwin->w_cursor.lnum;
          if (from == 0) {
            from = to;
          }
        }

        if (VIsual.lnum != wp->w_old_visual_lnum
            || VIsual.col != wp->w_old_visual_col) {
          if (wp->w_old_visual_lnum < from
              && wp->w_old_visual_lnum != 0) {
            from = wp->w_old_visual_lnum;
          }
          to = MAX(MAX(to, wp->w_old_visual_lnum), VIsual.lnum);
          from = MIN(from, VIsual.lnum);
        }
      }

      if (VIsual_mode == Ctrl_V) {
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

        if (fromc != wp->w_old_cursor_fcol
            || toc != wp->w_old_cursor_lcol) {
          from = MIN(from, VIsual.lnum);
          to = MAX(to, VIsual.lnum);
        }
        wp->w_old_cursor_fcol = fromc;
        wp->w_old_cursor_lcol = toc;
      }
    } else {
      if (wp->w_old_cursor_lnum < wp->w_old_visual_lnum) {
        from = wp->w_old_cursor_lnum;
        to = wp->w_old_visual_lnum;
      } else {
        from = wp->w_old_visual_lnum;
        to = wp->w_old_cursor_lnum;
      }
    }

    from = MAX(from, wp->w_topline);

    if (wp->w_valid & VALID_BOTLINE) {
      from = MIN(from, wp->w_botline - 1);
      to = MIN(to, wp->w_botline - 1);
    }

    if (*mid_start > 0) {
      linenr_T lnum = wp->w_topline;
      int idx = 0;
      int srow = 0;
      if (scrolled_down) {
        *mid_start = top_end;
      } else {
        *mid_start = 0;
      }
      while (lnum < from && idx < wp->w_lines_valid) {
        if (wp->w_lines[idx].wl_valid) {
          *mid_start += wp->w_lines[idx].wl_size;
        } else if (!scrolled_down) {
          srow += wp->w_lines[idx].wl_size;
        }
        idx++;
        if (idx < wp->w_lines_valid && wp->w_lines[idx].wl_valid) {
          lnum = wp->w_lines[idx].wl_lnum;
        } else {
          lnum++;
        }
      }
      srow += *mid_start;
      *mid_end = wp->w_view_height;
      for (; idx < wp->w_lines_valid; idx++) {
        if (wp->w_lines[idx].wl_valid
            && wp->w_lines[idx].wl_lnum >= to + 1) {
          *mid_end = srow;
          break;
        }
        srow += wp->w_lines[idx].wl_size;
      }
    }
  }

  if (VIsual_active && buf == curwin->w_buffer) {
    wp->w_old_visual_mode = (char)VIsual_mode;
    wp->w_old_cursor_lnum = curwin->w_cursor.lnum;
    wp->w_old_visual_lnum = VIsual.lnum;
    wp->w_old_visual_col = VIsual.col;
    wp->w_old_curswant = curwin->w_curswant;
  } else {
    wp->w_old_visual_mode = 0;
    wp->w_old_cursor_lnum = 0;
    wp->w_old_visual_lnum = 0;
    wp->w_old_visual_col = 0;
  }
}

// ---------------------------------------------------------------------------
// update_screen batch helper
// ---------------------------------------------------------------------------

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

/// Get the `updating_screen` global flag.
bool nvim_get_updating_screen(void)
{
  return updating_screen;
}
