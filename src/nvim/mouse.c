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

/// Get tabnr from tab_page_click_defs at given column.
int nvim_mouse_get_tab_click_tabnr(int col)
{
  if (tab_page_click_defs == NULL || col < 0 || col >= (int)tab_page_click_defs_size) {
    return 0;
  }
  return tab_page_click_defs[col].tabnr;
}

extern void rs_set_mouse_topline(win_T *wp);
extern void rs_move_tab_to_mouse(void);
extern void rs_mouse_tab_close(int c1);
extern void rs_mouse_check_grid(colnr_T *vcolp, int *flagsp);
extern int rs_get_fpos_of_mouse(pos_T *mpos);
extern int rs_do_popup(int which_button, int m_pos_flag, pos_T m_pos);
extern void rs_call_click_def_func(StlClickDefinition *click_defs, int col, int which_button);

// got_click state is now owned by Rust; use rs_get_got_click()/rs_set_got_click().

/// Bridge for Rust-computed click arguments to VimL function call.
/// Builds typval_T args from pre-computed strings and calls the VimL callback.
void nvim_call_stl_click_func(StlClickDefinition *click_defs, int col,
                               int click_count, const char *button_str,
                               const char *modifier_str)
{
  typval_T argv[] = {
    {
      .v_lock = VAR_FIXED,
      .v_type = VAR_NUMBER,
      .vval = { .v_number = (varnumber_T)click_defs[col].tabnr },
    },
    {
      .v_lock = VAR_FIXED,
      .v_type = VAR_NUMBER,
      .vval = { .v_number = click_count },
    },
    {
      .v_lock = VAR_FIXED,
      .v_type = VAR_STRING,
      .vval = { .v_string = (char *)button_str },
    },
    {
      .v_lock = VAR_FIXED,
      .v_type = VAR_STRING,
      .vval = { .v_string = (char *)modifier_str },
    },
  };
  typval_T rettv;
  call_vim_function(click_defs[col].func, ARRAY_SIZE(argv), argv, &rettv);
  tv_clear(&rettv);
}

// nvim_do_mouse_impl is now implemented in Rust (rs_do_mouse_impl, Phase 9).
// The C body is preserved in #if 0 for reference; it is dead code.
#if 0
/// OLD C implementation (replaced by Rust rs_do_mouse_impl).
static bool nvim_do_mouse_impl_DELETED(oparg_T *oap, int c, int dir, int count, bool fixindent)
{
  int which_button;             // MOUSE_LEFT, _MIDDLE or _RIGHT
  bool is_click;                // If false it's a drag or release event
  bool is_drag;                 // If true it's a drag event
  static bool in_tab_line = false;   // mouse clicked in tab line
  static pos_T orig_cursor;

  while (true) {
    which_button = get_mouse_button(KEY2TERMCAP1(c), &is_click, &is_drag);
    if (is_drag) {
      // If the next character is the same mouse event then use that
      // one. Speeds up dragging the status line.
      // Note: Since characters added to the stuff buffer in the code
      // below need to come before the next character, do not do this
      // when the current character was stuffed.
      if (!KeyStuffed && vpeekc() != NUL) {
        int nc;
        int save_mouse_grid = mouse_grid;
        int save_mouse_row = mouse_row;
        int save_mouse_col = mouse_col;

        // Need to get the character, peeking doesn't get the actual one.
        nc = safe_vgetc();
        if (c == nc) {
          continue;
        }
        vungetc(nc);
        mouse_grid = save_mouse_grid;
        mouse_row = save_mouse_row;
        mouse_col = save_mouse_col;
      }
    }
    break;
  }

  if (c == K_MOUSEMOVE) {
    // Mouse moved without a button pressed.
    return false;
  }

  // Ignore drag and release events if we didn't get a click.
  if (is_click) {
    rs_set_got_click(true);
  } else {
    if (!rs_get_got_click()) {          // didn't get click, ignore
      return false;
    }
    if (!is_drag) {                     // release, reset got_click
      rs_set_got_click(false);
      if (in_tab_line) {
        in_tab_line = false;
        return false;
      }
    }
  }

  // CTRL right mouse button does CTRL-T
  if (is_click && (mod_mask & MOD_MASK_CTRL) && which_button == MOUSE_RIGHT) {
    if (State & MODE_INSERT) {
      stuffcharReadbuff(Ctrl_O);
    }
    if (count > 1) {
      stuffnumReadbuff(count);
    }
    stuffcharReadbuff(Ctrl_T);
    rs_set_got_click(false);      // ignore drag&release now
    return false;
  }

  // CTRL only works with left mouse button
  if ((mod_mask & MOD_MASK_CTRL) && which_button != MOUSE_LEFT) {
    return false;
  }

  // When a modifier is down, ignore drag and release events, as well as
  // multiple clicks and the middle mouse button.
  // Accept shift-leftmouse drags when 'mousemodel' is "popup.*".
  if ((mod_mask & (MOD_MASK_SHIFT | MOD_MASK_CTRL | MOD_MASK_ALT
                   | MOD_MASK_META))
      && (!is_click
          || (mod_mask & MOD_MASK_MULTI_CLICK)
          || which_button == MOUSE_MIDDLE)
      && !((mod_mask & (MOD_MASK_SHIFT|MOD_MASK_ALT))
           && rs_mouse_model_popup((const char *)p_mousem)
           && which_button == MOUSE_LEFT)
      && !((mod_mask & MOD_MASK_ALT)
           && !rs_mouse_model_popup((const char *)p_mousem)
           && which_button == MOUSE_RIGHT)) {
    return false;
  }

  // If the button press was used as the movement command for an operator (eg
  // "d<MOUSE>"), or it is the middle button that is held down, ignore
  // drag/release events.
  if (!is_click && which_button == MOUSE_MIDDLE) {
    return false;
  }

  int regname = oap != NULL ? oap->regname : 0;
  // Middle mouse button does a 'put' of the selected text
  if (which_button == MOUSE_MIDDLE) {
    if (State == MODE_NORMAL) {
      // If an operator was pending, we don't know what the user wanted to do.
      // Go back to normal mode: Clear the operator and beep().
      if (oap != NULL && oap->op_type != OP_NOP) {
        rs_clearopbeep(oap);
        return false;
      }

      // If visual was active, yank the highlighted text and put it
      // before the mouse pointer position.
      // In Select mode replace the highlighted text with the clipboard.
      if (VIsual_active) {
        if (VIsual_select) {
          stuffcharReadbuff(Ctrl_G);
          stuffReadbuff("\"+p");
        } else {
          stuffcharReadbuff('y');
          stuffcharReadbuff(K_MIDDLEMOUSE);
        }
        return false;
      }
      // The rest is below jump_to_mouse()
    } else if ((State & MODE_INSERT) == 0) {
      return false;
    }

    // Middle click in insert mode doesn't move the mouse, just insert the
    // contents of a register.  '.' register is special, can't insert that
    // with do_put().
    // Also paste at the cursor if the current mode isn't in 'mouse' (only
    // happens for the GUI).
    if ((State & MODE_INSERT)) {
      if (regname == '.') {
        insert_reg(regname, NULL, true);
      } else {
        if (regname == 0 && eval_has_provider("clipboard", false)) {
          regname = '*';
        }
        yankreg_T *reg = NULL;
        if ((State & REPLACE_FLAG) && !yank_register_mline(regname, &reg)) {
          insert_reg(regname, reg, true);
        } else {
          do_put(regname, reg, BACKWARD, 1,
                 (fixindent ? PUT_FIXINDENT : 0) | PUT_CURSEND);

          // Repeat it with CTRL-R CTRL-O r or CTRL-R CTRL-P r
          AppendCharToRedobuff(Ctrl_R);
          AppendCharToRedobuff(fixindent ? Ctrl_P : Ctrl_O);
          AppendCharToRedobuff(regname == 0 ? '"' : regname);
        }
      }
      return false;
    }
  }

  int jump_flags = is_click ? 0 : (MOUSE_FOCUS|MOUSE_DID_MOVE);
  win_T *old_curwin = curwin;

  if (tab_page_click_defs != NULL) {  // only when initialized
    if (mouse_grid <= 1 && mouse_row == 0 && firstwin->w_winrow > 0) {
      if (is_drag) {
        if (in_tab_line) {
          rs_move_tab_to_mouse();
        }
        return false;
      }

      if (is_click && cmdwin_type == 0 && mouse_col < Columns) {
        int tabnr = tab_page_click_defs[mouse_col].tabnr;
        in_tab_line = true;

        switch (tab_page_click_defs[mouse_col].type) {
        case kStlClickDisabled:
          break;
        case kStlClickTabSwitch:
          if (which_button != MOUSE_MIDDLE) {
            if ((mod_mask & MOD_MASK_MULTI_CLICK) == MOD_MASK_2CLICK) {
              end_visual_mode();
              tabpage_new();
              tabpage_move(tabnr == 0 ? 9999 : tabnr - 1);
            } else {
              goto_tabpage(tabnr);
              if (curwin != old_curwin) {
                end_visual_mode();
              }
            }
            break;
          }
          FALLTHROUGH;
        case kStlClickTabClose:
          rs_mouse_tab_close(tabnr);
          break;
        case kStlClickFuncRun:
          rs_call_click_def_func(tab_page_click_defs, mouse_col, which_button);
          break;
        }
      }
      return true;
    } else if (is_drag && in_tab_line) {
      rs_move_tab_to_mouse();
      return false;
    }
  }

  int m_pos_flag = 0;
  pos_T m_pos = { 0 };
  // When 'mousemodel' is "popup" or "popup_setpos", translate mouse events:
  // right button up   -> pop-up menu
  // shift-left button -> right button
  // alt-left button   -> alt-right button
  if (rs_mouse_model_popup((const char *)p_mousem)) {
    m_pos_flag = rs_get_fpos_of_mouse(&m_pos);
    if (!(m_pos_flag & (IN_STATUS_LINE|MOUSE_WINBAR|MOUSE_STATUSCOL))
        && which_button == MOUSE_RIGHT && !(mod_mask & (MOD_MASK_SHIFT|MOD_MASK_CTRL))) {
      if (!is_click) {
        // Ignore right button release events, only shows the popup
        // menu on the button down event.
        return false;
      }
      return (rs_do_popup(which_button, m_pos_flag, m_pos) & CURSOR_MOVED);
    }
    if (!(m_pos_flag & (IN_STATUS_LINE|MOUSE_WINBAR|MOUSE_STATUSCOL))
        && (which_button == MOUSE_LEFT && (mod_mask & (MOD_MASK_SHIFT|MOD_MASK_ALT)))) {
      which_button = MOUSE_RIGHT;
      mod_mask &= ~MOD_MASK_SHIFT;
    }
  }

  pos_T end_visual = { 0 };
  pos_T start_visual = { 0 };
  if ((State & (MODE_NORMAL | MODE_INSERT))
      && !(mod_mask & (MOD_MASK_SHIFT | MOD_MASK_CTRL))) {
    if (which_button == MOUSE_LEFT) {
      if (is_click) {
        // stop Visual mode for a left click in a window, but not when on a status line
        if (VIsual_active) {
          jump_flags |= MOUSE_MAY_STOP_VIS;
        }
      } else {
        jump_flags |= MOUSE_MAY_VIS;
      }
    } else if (which_button == MOUSE_RIGHT) {
      if (is_click && VIsual_active) {
        // Remember the start and end of visual before moving the cursor.
        if (lt(curwin->w_cursor, VIsual)) {
          start_visual = curwin->w_cursor;
          end_visual = VIsual;
        } else {
          start_visual = VIsual;
          end_visual = curwin->w_cursor;
        }
      }
      jump_flags |= MOUSE_FOCUS;
      jump_flags |= MOUSE_MAY_VIS;
    }
  }

  // If an operator is pending, ignore all drags and releases until the next mouse click.
  if (!is_drag && oap != NULL && oap->op_type != OP_NOP) {
    rs_set_got_click(false);
    oap->motion_type = kMTCharWise;
  }

  // When releasing the button let jump_to_mouse() know.
  if (!is_click && !is_drag) {
    jump_flags |= MOUSE_RELEASED;
  }

  int old_active = VIsual_active;
  pos_T save_cursor = curwin->w_cursor;
  jump_flags = jump_to_mouse(jump_flags, oap == NULL ? NULL : &(oap->inclusive), which_button);

  bool moved = (jump_flags & CURSOR_MOVED);
  bool in_winbar = (jump_flags & MOUSE_WINBAR);
  bool in_statuscol = (jump_flags & MOUSE_STATUSCOL);
  bool in_status_line = (jump_flags & IN_STATUS_LINE);
  bool in_global_statusline = in_status_line && rs_global_stl_height() > 0;
  bool in_sep_line = (jump_flags & IN_SEP_LINE);

  if ((in_winbar || in_status_line || in_statuscol) && is_click) {
    // Handle click event on window bar, status line or status column
    int click_grid = mouse_grid;
    int click_row = mouse_row;
    int click_col = mouse_col;
    win_T *wp = mouse_find_win_inner(&click_grid, &click_row, &click_col);
    if (wp == NULL) {
      return false;
    }

    StlClickDefinition *click_defs = in_status_line ? wp->w_status_click_defs
                                                    : in_winbar ? wp->w_winbar_click_defs
                                                                : wp->w_statuscol_click_defs;

    if (in_global_statusline) {
      // global statusline is displayed for the current window,
      // and spans the whole screen.
      click_defs = curwin->w_status_click_defs;
      click_col = mouse_col;
    }

    if (in_statuscol && wp->w_p_rl) {
      click_col = wp->w_view_width - click_col - 1;
    }

    if ((in_statuscol && click_col >= (int)wp->w_statuscol_click_defs_size)
        || (in_status_line
            && click_col >=
            (int)(in_global_statusline ? curwin : wp)->w_status_click_defs_size)) {
      return false;
    }

    if (click_defs != NULL) {
      switch (click_defs[click_col].type) {
      case kStlClickDisabled:
        // If there is no click definition, still open the popupmenu for a
        // statuscolumn click like a click in the sign/number column does.
        if (in_statuscol && rs_mouse_model_popup((const char *)p_mousem)
            && which_button == MOUSE_RIGHT && !(mod_mask & (MOD_MASK_SHIFT|MOD_MASK_CTRL))) {
          rs_do_popup(which_button, m_pos_flag, m_pos);
        }
        break;
      case kStlClickFuncRun:
        rs_call_click_def_func(click_defs, click_col, which_button);
        break;
      default:
        assert(false && "winbar, statusline and statuscolumn only support %@ for clicks");
        break;
      }
    }

    if (!(in_statuscol && (jump_flags & (MOUSE_FOLD_CLOSE|MOUSE_FOLD_OPEN)))) {
      return false;
    }
  } else if (in_winbar || in_statuscol) {
    return false;
  }

  if (curwin != old_curwin && oap != NULL && oap->op_type != OP_NOP) {
    rs_clearop(oap);
  }

  if (mod_mask == 0
      && !is_drag
      && (jump_flags & (MOUSE_FOLD_CLOSE | MOUSE_FOLD_OPEN))
      && which_button == MOUSE_LEFT) {
    if (jump_flags & MOUSE_FOLD_OPEN) {
      rs_setFoldRepeat(curwin->w_cursor.lnum, 1, true);
    } else {
      rs_setFoldRepeat(curwin->w_cursor.lnum, 1, false);
    }
    if (curwin == old_curwin) {
      curwin->w_cursor = save_cursor;
    }
  }

  if (VIsual_active && is_drag && rs_get_scrolloff_value(curwin)) {
    // In the very first line, allow scrolling one line
    if (mouse_row == 0) {
      mouse_dragging = 2;
    } else {
      mouse_dragging = 1;
    }
  }

  // When dragging the mouse above the window, scroll down.
  if (is_drag && mouse_row < 0 && !in_status_line) {
    scroll_redraw(false, 1);
    mouse_row = 0;
  }

  int old_mode = VIsual_mode;
  if (start_visual.lnum) {              // right click in visual mode
    linenr_T diff;
    // When ALT is pressed make Visual mode blockwise.
    if (mod_mask & MOD_MASK_ALT) {
      VIsual_mode = Ctrl_V;
    }

    if (VIsual_mode == Ctrl_V) {
      colnr_T leftcol, rightcol;
      getvcols(curwin, &start_visual, &end_visual, &leftcol, &rightcol);
      if (curwin->w_curswant > (leftcol + rightcol) / 2) {
        end_visual.col = leftcol;
      } else {
        end_visual.col = rightcol;
      }
      if (curwin->w_cursor.lnum >=
          (start_visual.lnum + end_visual.lnum) / 2) {
        end_visual.lnum = start_visual.lnum;
      }

      // move VIsual to the right column
      start_visual = curwin->w_cursor;              // save the cursor pos
      curwin->w_cursor = end_visual;
      coladvance(curwin, end_visual.col);
      VIsual = curwin->w_cursor;
      curwin->w_cursor = start_visual;              // restore the cursor
    } else {
      // If the click is before the start of visual, change the start.
      // If the click is after the end of visual, change the end.  If
      // the click is inside the visual, change the closest side.
      if (lt(curwin->w_cursor, start_visual)) {
        VIsual = end_visual;
      } else if (lt(end_visual, curwin->w_cursor)) {
        VIsual = start_visual;
      } else {
        // In the same line, compare column number
        if (end_visual.lnum == start_visual.lnum) {
          if (curwin->w_cursor.col - start_visual.col >
              end_visual.col - curwin->w_cursor.col) {
            VIsual = start_visual;
          } else {
            VIsual = end_visual;
          }
        } else {
          // In different lines, compare line number
          diff = (curwin->w_cursor.lnum - start_visual.lnum) -
                 (end_visual.lnum - curwin->w_cursor.lnum);

          if (diff > 0) {                       // closest to end
            VIsual = start_visual;
          } else if (diff < 0) {                   // closest to start
            VIsual = end_visual;
          } else {                                // in the middle line
            if (curwin->w_cursor.col <
                (start_visual.col + end_visual.col) / 2) {
              VIsual = end_visual;
            } else {
              VIsual = start_visual;
            }
          }
        }
      }
    }
  } else if ((State & MODE_INSERT) && VIsual_active) {
    // If Visual mode started in insert mode, execute "CTRL-O"
    stuffcharReadbuff(Ctrl_O);
  }

  // Middle mouse click: Put text before cursor.
  if (which_button == MOUSE_MIDDLE) {
    if (regname == 0 && eval_has_provider("clipboard", false)) {
      regname = '*';
    }
    yankreg_T *reg = NULL;
    if (yank_register_mline(regname, &reg)) {
      if (mouse_past_bottom) {
        dir = FORWARD;
      }
    } else if (mouse_past_eol) {
      dir = FORWARD;
    }

    int c1, c2;
    if (fixindent) {
      c1 = (dir == BACKWARD) ? '[' : ']';
      c2 = 'p';
    } else {
      c1 = (dir == FORWARD) ? 'p' : 'P';
      c2 = NUL;
    }
    rs_prep_redo(regname, count, NUL, c1, NUL, c2, NUL);

    // Remember where the paste started, so in edit() Insstart can be set to this position
    if (restart_edit != 0) {
      where_paste_started = curwin->w_cursor;
    }
    do_put(regname, reg, dir, count,
           (fixindent ? PUT_FIXINDENT : 0)| PUT_CURSEND);
  } else if (((mod_mask & MOD_MASK_CTRL) || (mod_mask & MOD_MASK_MULTI_CLICK) == MOD_MASK_2CLICK)
             && bt_quickfix(curbuf)) {
    // Ctrl-Mouse click or double click in a quickfix window jumps to the
    // error under the mouse pointer.
    if (curwin->w_llist_ref == NULL) {          // quickfix window
      do_cmdline_cmd(".cc");
    } else {                                    // location list window
      do_cmdline_cmd(".ll");
    }
    rs_set_got_click(false);                    // ignore drag&release now
  } else if ((mod_mask & MOD_MASK_CTRL)
             || (curbuf->b_help && (mod_mask & MOD_MASK_MULTI_CLICK) == MOD_MASK_2CLICK)) {
    // Ctrl-Mouse click (or double click in a help window) jumps to the tag
    // under the mouse pointer.
    if (State & MODE_INSERT) {
      stuffcharReadbuff(Ctrl_O);
    }
    stuffcharReadbuff(Ctrl_RSB);
    rs_set_got_click(false);                    // ignore drag&release now
  } else if ((mod_mask & MOD_MASK_SHIFT)) {
    // Shift-Mouse click searches for the next occurrence of the word under
    // the mouse pointer
    if (State & MODE_INSERT || (VIsual_active && VIsual_select)) {
      stuffcharReadbuff(Ctrl_O);
    }
    if (which_button == MOUSE_LEFT) {
      stuffcharReadbuff('*');
    } else {  // MOUSE_RIGHT
      stuffcharReadbuff('#');
    }
  } else if (in_status_line || in_sep_line) {
    // Do nothing if on status line or vertical separator
    // Handle double clicks otherwise
  } else if ((mod_mask & MOD_MASK_MULTI_CLICK) && (State & (MODE_NORMAL | MODE_INSERT))) {
    if (is_click || !VIsual_active) {
      if (VIsual_active) {
        orig_cursor = VIsual;
      } else {
        VIsual = curwin->w_cursor;
        orig_cursor = VIsual;
        VIsual_active = true;
        VIsual_reselect = true;
        // start Select mode if 'selectmode' contains "mouse"
        rs_may_start_select('o');
        setmouse();
      }
      if ((mod_mask & MOD_MASK_MULTI_CLICK) == MOD_MASK_2CLICK) {
        // Double click with ALT pressed makes it blockwise.
        if (mod_mask & MOD_MASK_ALT) {
          VIsual_mode = Ctrl_V;
        } else {
          VIsual_mode = 'v';
        }
      } else if ((mod_mask & MOD_MASK_MULTI_CLICK) == MOD_MASK_3CLICK) {
        VIsual_mode = 'V';
      } else if ((mod_mask & MOD_MASK_MULTI_CLICK) == MOD_MASK_4CLICK) {
        VIsual_mode = Ctrl_V;
      }
    }
    // A double click selects a word or a block.
    if ((mod_mask & MOD_MASK_MULTI_CLICK) == MOD_MASK_2CLICK) {
      pos_T *pos = NULL;

      if (is_click) {
        // If the character under the cursor (skipping white space) is
        // not a word character, try finding a match and select a (),
        // {}, [], #if/#endif, etc. block.
        end_visual = curwin->w_cursor;
        int gc;
        while (gc = gchar_pos(&end_visual), ascii_iswhite(gc)) {
          inc(&end_visual);
        }
        if (oap != NULL) {
          oap->motion_type = kMTCharWise;
        }
        if (oap != NULL
            && VIsual_mode == 'v'
            && !vim_iswordc(gchar_pos(&end_visual))
            && equalpos(curwin->w_cursor, VIsual)
            && (pos = findmatch(oap, NUL)) != NULL) {
          curwin->w_cursor = *pos;
          if (oap->motion_type == kMTLineWise) {
            VIsual_mode = 'V';
          } else if (*p_sel == 'e') {
            if (lt(curwin->w_cursor, VIsual)) {
              VIsual.col++;
            } else {
              curwin->w_cursor.col++;
            }
          }
        }
      }

      if (pos == NULL && (is_click || is_drag)) {
        // When not found a match or when dragging: extend to include a word.
        if (lt(curwin->w_cursor, orig_cursor)) {
          { char *line = ml_get(curwin->w_cursor.lnum); curwin->w_cursor.col = rs_find_start_of_word(line, curwin->w_cursor.col); }
          { char *line = ml_get(VIsual.lnum); VIsual.col = rs_find_end_of_word(line, VIsual.col, *p_sel == 'e'); }
        } else {
          { char *line = ml_get(VIsual.lnum); VIsual.col = rs_find_start_of_word(line, VIsual.col); }
          if (*p_sel == 'e' && *get_cursor_pos_ptr() != NUL) {
            curwin->w_cursor.col +=
              utfc_ptr2len(get_cursor_pos_ptr());
          }
          { char *line = ml_get(curwin->w_cursor.lnum); curwin->w_cursor.col = rs_find_end_of_word(line, curwin->w_cursor.col, *p_sel == 'e'); }
        }
      }
      curwin->w_set_curswant = true;
    }
    if (is_click) {
      redraw_curbuf_later(UPD_INVERTED);  // update the inversion
    }
  } else if (VIsual_active && !old_active) {
    if (mod_mask & MOD_MASK_ALT) {
      VIsual_mode = Ctrl_V;
    } else {
      VIsual_mode = 'v';
    }
  }

  // If Visual mode changed show it later.
  if ((!VIsual_active && old_active && mode_displayed)
      || (VIsual_active && p_smd && msg_silent == 0
          && (!old_active || VIsual_mode != old_mode))) {
    redraw_cmdline = true;
  }

  return moved;
}
#endif  // nvim_do_mouse_impl_DELETED

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

/// Decode a mouse key code into which_button, is_click, is_drag.
int nvim_get_mouse_button(int code, bool *is_click, bool *is_drag)
{
  return get_mouse_button(code, is_click, is_drag);
}

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

/// Get the click type for a given column from tab_page_click_defs.
int nvim_mouse_get_tab_click_type(int col)
{
  if (tab_page_click_defs == NULL || col < 0 || col >= (int)tab_page_click_defs_size) {
    return 0;  // kStlClickDisabled
  }
  return (int)tab_page_click_defs[col].type;
}

/// Get pointer to tab_page_click_defs (as opaque handle).
StlClickDefinition *nvim_get_tab_page_click_defs_ptr(void) { return tab_page_click_defs; }

/// Get restart_edit global.
int nvim_get_restart_edit_mouse(void) { return restart_edit; }

/// Get VIsual_reselect flag.
bool nvim_get_VIsual_reselect(void) { return VIsual_reselect; }

// --- Insert/put/register operations ------------------------------------------

// nvim_eval_has_provider already defined in eval/funcs_shim.c

/// Handle middle-click paste in insert mode.
/// Returns false always (caller should return false after calling this).
bool nvim_mouse_middle_insert_mode(int regname, int count, bool fixindent)
{
  if (regname == '.') {
    insert_reg(regname, NULL, true);
  } else {
    if (regname == 0 && eval_has_provider("clipboard", false)) {
      regname = '*';
    }
    yankreg_T *reg = NULL;
    if ((State & REPLACE_FLAG) && !yank_register_mline(regname, &reg)) {
      insert_reg(regname, reg, true);
    } else {
      do_put(regname, reg, BACKWARD, 1,
             (fixindent ? PUT_FIXINDENT : 0) | PUT_CURSEND);
      AppendCharToRedobuff(Ctrl_R);
      AppendCharToRedobuff(fixindent ? Ctrl_P : Ctrl_O);
      AppendCharToRedobuff(regname == 0 ? '"' : regname);
    }
  }
  return false;
}

/// Middle-click put after jump_to_mouse (normal mode).
void nvim_do_put_middle_click(int regname, int dir, int count, bool fixindent)
{
  yankreg_T *reg = NULL;
  bool is_mline = yank_register_mline(regname, &reg);
  if (is_mline) {
    if (mouse_past_bottom) {
      dir = FORWARD;
    }
  } else if (mouse_past_eol) {
    dir = FORWARD;
  }

  int c1, c2;
  if (fixindent) {
    c1 = (dir == BACKWARD) ? '[' : ']';
    c2 = 'p';
  } else {
    c1 = (dir == FORWARD) ? 'p' : 'P';
    c2 = NUL;
  }
  rs_prep_redo(regname, count, NUL, c1, NUL, c2, NUL);

  if (restart_edit != 0) {
    where_paste_started = curwin->w_cursor;
  }
  do_put(regname, reg, dir, count,
         (fixindent ? PUT_FIXINDENT : 0) | PUT_CURSEND);
}

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

/// Get click type from a StlClickDefinition array at given column.
int nvim_stl_click_defs_get_type(StlClickDefinition *click_defs, int col)
{
  if (click_defs == NULL) {
    return 0;  // kStlClickDisabled
  }
  return (int)click_defs[col].type;
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

/// Get the character at a given (lnum, col, coladd) position.
int nvim_gchar_pos(linenr_T lnum, int col, int coladd)
{
  pos_T p = { .lnum = lnum, .col = col, .coladd = coladd };
  return gchar_pos(&p);
}

/// Increment a position (lnum, col, coladd). Returns 1 if moved past EOL.
int nvim_inc_pos(linenr_T *lnum, int *col, int *coladd)
{
  pos_T p = { .lnum = *lnum, .col = *col, .coladd = *coladd };
  int r = inc(&p);
  *lnum = p.lnum;
  *col = p.col;
  *coladd = p.coladd;
  return r;
}

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

/// Get virtual column ranges for a visual selection.
void nvim_getvcols_mouse(colnr_T *leftcol, colnr_T *rightcol,
                          linenr_T sv_lnum, int sv_col, int sv_coladd,
                          linenr_T ev_lnum, int ev_col, int ev_coladd)
{
  pos_T sv = { .lnum = sv_lnum, .col = sv_col, .coladd = sv_coladd };
  pos_T ev = { .lnum = ev_lnum, .col = ev_col, .coladd = ev_coladd };
  getvcols(curwin, &sv, &ev, leftcol, rightcol);
}

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
