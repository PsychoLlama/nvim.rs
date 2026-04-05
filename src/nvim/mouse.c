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

/// C accessor: the actual do_mouse logic — entry point called from rs_do_mouse.
bool nvim_do_mouse_impl(oparg_T *oap, int c, int dir, int count, bool fixindent)
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

// dragwin state is now owned by Rust; use rs_get_dragwin()/rs_set_dragwin()/rs_is_dragging().

/// C accessor: the actual jump_to_mouse logic — entry point called from rs_jump_to_mouse.
int nvim_jump_to_mouse_impl(int flags, bool *inclusive, int which_button)
{
  static int status_line_offset = 0;        // #lines offset from status line
  static int sep_line_offset = 0;           // #cols offset from sep line
  static bool on_status_line = false;
  static bool on_sep_line = false;
  static bool on_winbar = false;
  static bool on_statuscol = false;
  static int prev_row = -1;
  static int prev_col = -1;
  static int did_drag = false;          // drag was noticed

  int count;
  bool first;
  int row = mouse_row;
  int col = mouse_col;
  int grid = mouse_grid;
  int fdc = 0;
  bool keep_focus = flags & MOUSE_FOCUS;

  mouse_past_bottom = false;
  mouse_past_eol = false;

  if (flags & MOUSE_RELEASED) {
    // On button release we may change window focus if positioned on a
    // status line and no dragging happened.
    if (rs_is_dragging() && !did_drag) {
      flags &= ~(MOUSE_FOCUS | MOUSE_DID_MOVE);
    }
    rs_set_dragwin(NULL);
    did_drag = false;
  }

  if ((flags & MOUSE_DID_MOVE)
      && prev_row == mouse_row
      && prev_col == mouse_col) {
retnomove:
    if (status_line_offset) {
      return IN_STATUS_LINE;
    }
    if (sep_line_offset) {
      return IN_SEP_LINE;
    }
    if (on_winbar) {
      return IN_OTHER_WIN | MOUSE_WINBAR;
    }
    if (on_statuscol) {
      return IN_OTHER_WIN | MOUSE_STATUSCOL;
    }
    if (flags & MOUSE_MAY_STOP_VIS) {
      end_visual_mode();
      redraw_curbuf_later(UPD_INVERTED);  // delete the inversion
    }
    return IN_BUFFER;
  }

  prev_row = mouse_row;
  prev_col = mouse_col;

  if (flags & MOUSE_SETPOS) {
    goto retnomove;                             // ugly goto...
  }

  if (row < 0 || col < 0) {                   // check if it makes sense
    return IN_UNKNOWN;
  }

  win_T *wp = mouse_find_win_inner(&grid, &row, &col);
  if (wp == NULL) {
    return IN_UNKNOWN;
  }

  bool below_window = grid == DEFAULT_GRID_HANDLE && row + wp->w_winbar_height >= wp->w_height;
  on_status_line = below_window && row + wp->w_winbar_height - wp->w_height + 1 == 1;
  on_sep_line = grid == DEFAULT_GRID_HANDLE && col >= wp->w_width && col - wp->w_width + 1 == 1;
  on_winbar = row < 0 && row + wp->w_winbar_height >= 0;
  on_statuscol = !below_window && !on_status_line && !on_sep_line && !on_winbar
                 && *wp->w_p_stc != NUL
                 && (wp->w_p_rl
                     ? col >= wp->w_view_width - win_col_off(wp)
                     : col < win_col_off(wp));

  // The rightmost character of the status line might be a vertical
  // separator character if there is no connecting window to the right.
  if (on_status_line && on_sep_line) {
    if (stl_connected(wp)) {
      on_sep_line = false;
    } else {
      on_status_line = false;
    }
  }

  if (keep_focus) {
    row = mouse_row;
    col = mouse_col;
    grid = mouse_grid;
  }

  win_T *old_curwin = curwin;
  pos_T old_cursor = curwin->w_cursor;
  if (!keep_focus) {
    if (on_winbar) {
      return IN_OTHER_WIN | MOUSE_WINBAR;
    }

    if (on_statuscol) {
      goto foldclick;
    }

    fdc = rs_win_fdccol_count(wp);
    rs_set_dragwin(NULL);

    if (below_window) {
      // In (or below) status line
      status_line_offset = row + wp->w_winbar_height - wp->w_height + 1;
      rs_set_dragwin(wp);
    } else {
      status_line_offset = 0;
    }

    if (grid == DEFAULT_GRID_HANDLE && col >= wp->w_width) {
      // In separator line
      sep_line_offset = col - wp->w_width + 1;
      rs_set_dragwin(wp);
    } else {
      sep_line_offset = 0;
    }

    // The rightmost character of the status line might be a vertical
    // separator character if there is no connecting window to the right.
    if (status_line_offset && sep_line_offset) {
      if (stl_connected(wp)) {
        sep_line_offset = 0;
      } else {
        status_line_offset = 0;
      }
    }

    // Before jumping to another buffer, or moving the cursor for a left
    // click, stop Visual mode.
    if (VIsual_active
        && (wp->w_buffer != curwin->w_buffer
            || (!status_line_offset
                && !sep_line_offset
                && (wp->w_p_rl
                    ? col < wp->w_view_width - fdc
                    : col >= fdc + (wp != cmdwin_win ? 0 : 1))
                && (flags & MOUSE_MAY_STOP_VIS)))) {
      end_visual_mode();
      redraw_curbuf_later(UPD_INVERTED);  // delete the inversion
    }
    if (cmdwin_type != 0 && wp != cmdwin_win) {
      // A click outside the command-line window: Use modeless
      // selection if possible.  Allow dragging the status lines.
      sep_line_offset = 0;
      row = 0;
      col += wp->w_wincol;
      wp = cmdwin_win;
    }
    // Only change window focus when not clicking on or dragging the
    // status line.  Do change focus when releasing the mouse button
    // (MOUSE_FOCUS was set above if we dragged first).
    if (!rs_is_dragging() || (flags & MOUSE_RELEASED)) {
      win_enter(wp, true);                      // can make wp invalid!
    }
    if (curwin != old_curwin) {
      rs_set_mouse_topline(curwin);
    }
    if (status_line_offset) {                       // In (or below) status line
      // Don't use start_arrow() if we're in the same window
      if (curwin == old_curwin) {
        return IN_STATUS_LINE;
      }
      return IN_STATUS_LINE | CURSOR_MOVED;
    }
    if (sep_line_offset) {                          // In (or below) status line
      // Don't use start_arrow() if we're in the same window
      if (curwin == old_curwin) {
        return IN_SEP_LINE;
      }
      return IN_SEP_LINE | CURSOR_MOVED;
    }

    curwin->w_cursor.lnum = curwin->w_topline;
  } else if (status_line_offset) {
    win_T *dw = rs_get_dragwin();
    if (which_button == MOUSE_LEFT && dw != NULL) {
      // Drag the status line
      count = row - dw->w_winrow - dw->w_height + 1
              - status_line_offset;
      rs_win_drag_status_line(dw, count);
      did_drag |= count;
    }
    return IN_STATUS_LINE;                      // Cursor didn't move
  } else if (sep_line_offset && which_button == MOUSE_LEFT) {
    win_T *dw = rs_get_dragwin();
    if (dw != NULL) {
      // Drag the separator column
      count = col - dw->w_wincol - dw->w_width + 1
              - sep_line_offset;
      rs_win_drag_vsep_line(dw, count);
      did_drag |= count;
    }
    return IN_SEP_LINE;                         // Cursor didn't move
  } else if (on_status_line && which_button == MOUSE_RIGHT) {
    return IN_STATUS_LINE;
  } else if (on_winbar && which_button == MOUSE_RIGHT) {
    // After a click on the window bar don't start Visual mode.
    return IN_OTHER_WIN | MOUSE_WINBAR;
  } else if (on_statuscol && which_button == MOUSE_RIGHT) {
    // After a click on the status column don't start Visual mode.
    return IN_OTHER_WIN | MOUSE_STATUSCOL;
  } else {
    if (flags & MOUSE_MAY_STOP_VIS) {
      end_visual_mode();
      redraw_curbuf_later(UPD_INVERTED);  // delete the inversion
    }

    if (grid == 0) {
      row -= curwin->w_grid_alloc.comp_row + curwin->w_grid.row_offset;
      col -= curwin->w_grid_alloc.comp_col + curwin->w_grid.col_offset;
    } else if (grid != DEFAULT_GRID_HANDLE) {
      row -= curwin->w_grid.row_offset;
      col -= curwin->w_grid.col_offset;
    }

    if (row < 0) {
      count = 0;
      for (first = true; curwin->w_topline > 1;) {
        if (curwin->w_topfill < win_get_fill(curwin, curwin->w_topline)) {
          count++;
        } else {
          count += plines_win(curwin, curwin->w_topline - 1, true);
        }
        if (!first && count > -row) {
          break;
        }
        first = false;
        hasFolding(curwin, curwin->w_topline, &curwin->w_topline, NULL);
        if (curwin->w_topfill < win_get_fill(curwin, curwin->w_topline)) {
          curwin->w_topfill++;
        } else {
          curwin->w_topline--;
          curwin->w_topfill = 0;
        }
      }
      check_topfill(curwin, false);
      curwin->w_valid &=
        ~(VALID_WROW|VALID_CROW|VALID_BOTLINE|VALID_BOTLINE_AP);
      redraw_later(curwin, UPD_VALID);
      row = 0;
    } else if (row >= curwin->w_view_height) {
      count = 0;
      for (first = true; curwin->w_topline < curbuf->b_ml.ml_line_count;) {
        if (curwin->w_topfill > 0) {
          count++;
        } else {
          count += plines_win(curwin, curwin->w_topline, true);
        }

        if (!first && count > row - curwin->w_view_height + 1) {
          break;
        }
        first = false;

        if (curwin->w_topfill > 0) {
          curwin->w_topfill--;
        } else {
          if (hasFolding(curwin, curwin->w_topline, NULL, &curwin->w_topline)
              && curwin->w_topline == curbuf->b_ml.ml_line_count) {
            break;
          }
          curwin->w_topline++;
          curwin->w_topfill = win_get_fill(curwin, curwin->w_topline);
        }
      }
      check_topfill(curwin, false);
      redraw_later(curwin, UPD_VALID);
      curwin->w_valid &=
        ~(VALID_WROW|VALID_CROW|VALID_BOTLINE|VALID_BOTLINE_AP);
      row = curwin->w_view_height - 1;
    } else if (row == 0) {
      // When dragging the mouse, while the text has been scrolled up as
      // far as it goes, moving the mouse in the top line should scroll
      // the text down (done later when recomputing w_topline).
      if (mouse_dragging > 0
          && curwin->w_cursor.lnum
          == curwin->w_buffer->b_ml.ml_line_count
          && curwin->w_cursor.lnum == curwin->w_topline) {
        curwin->w_valid &= ~(VALID_TOPLINE);
      }
    }
  }

foldclick:;
  colnr_T col_from_screen = -1;
  int mouse_fold_flags = 0;
  rs_mouse_check_grid(&col_from_screen, &mouse_fold_flags);

  if (mouse_comp_pos(curwin, &row, &col, &curwin->w_cursor.lnum)) {
    mouse_past_bottom = true;
  }

  if ((flags & MOUSE_MAY_VIS) && !VIsual_active) {
    VIsual = old_cursor;
    VIsual_active = true;
    VIsual_reselect = true;
    rs_may_start_select('o');
    setmouse();

    if (p_smd && msg_silent == 0) {
      redraw_cmdline = true;            // show visual mode later
    }
  }

  if (col_from_screen >= 0) {
    col = col_from_screen;
  }

  curwin->w_curswant = col;
  curwin->w_set_curswant = false;       // May still have been true
  if (coladvance(curwin, col) == FAIL) {        // Mouse click beyond end of line
    if (inclusive != NULL) {
      *inclusive = true;
    }
    mouse_past_eol = true;
  } else if (inclusive != NULL) {
    *inclusive = false;
  }

  count = on_statuscol ? (IN_OTHER_WIN|MOUSE_STATUSCOL) : IN_BUFFER;
  if (curwin != old_curwin || curwin->w_cursor.lnum != old_cursor.lnum
      || curwin->w_cursor.col != old_cursor.col) {
    count |= CURSOR_MOVED;              // Cursor has moved
  }

  count |= mouse_fold_flags;

  return count;
}

/// C accessor: resolve grid handle to window and adjust row/col.
win_T *nvim_mouse_find_grid_win(int *gridp, int *rowp, int *colp)
{
  if (*gridp == msg_grid.handle) {
    *rowp += msg_grid_pos;
    *gridp = DEFAULT_GRID_HANDLE;
  } else if (*gridp > 1) {
    win_T *wp = get_win_by_grid_handle(*gridp);
    if (wp && wp->w_grid_alloc.chars
        && !(wp->w_floating && !wp->w_config.mouse)) {
      *rowp = MIN(*rowp - wp->w_grid.row_offset, wp->w_view_height - 1);
      *colp = MIN(*colp - wp->w_grid.col_offset, wp->w_view_width - 1);
      return wp;
    }
  } else if (*gridp == 0) {
    ScreenGrid *grid = ui_comp_mouse_focus(*rowp, *colp);
    if (grid == &pum_grid) {
      *gridp = grid->handle;
      *rowp -= grid->comp_row;
      *colp -= grid->comp_col;
      // The popup menu doesn't have a window, so return NULL
      return NULL;
    } else {
      FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
        if (&wp->w_grid_alloc != grid) {
          continue;
        }
        *gridp = grid->handle;
        *rowp -= wp->w_winrow + wp->w_grid.row_offset;
        *colp -= wp->w_wincol + wp->w_grid.col_offset;
        return wp;
      }
    }

    // No grid found, return the default grid. With multigrid this happens for split separators for
    // example.
    *gridp = DEFAULT_GRID_HANDLE;
  }
  return NULL;
}

/// C accessor: traverse frame tree to find window at row/col position.
win_T *nvim_frame_find_win(int *rowp, int *colp)
{
  frame_T *fp = topframe;
  *rowp -= firstwin->w_winrow;
  while (true) {
    if (fp->fr_layout == FR_LEAF) {
      break;
    }
    if (fp->fr_layout == FR_ROW) {
      for (fp = fp->fr_child; fp->fr_next != NULL; fp = fp->fr_next) {
        if (*colp < fp->fr_width) {
          break;
        }
        *colp -= fp->fr_width;
      }
    } else {  // fr_layout == FR_COL
      for (fp = fp->fr_child; fp->fr_next != NULL; fp = fp->fr_next) {
        if (*rowp < fp->fr_height) {
          break;
        }
        *rowp -= fp->fr_height;
      }
    }
  }
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (wp == fp->fr_win) {
      *rowp -= wp->w_winbar_height;
      return wp;
    }
  }
  return NULL;
}

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

// nvim_mouse_check_grid_impl migrated to Rust (rs_mouse_check_grid in mouse crate).

/// "getmousepos()" function
void f_getmousepos(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  int row = mouse_row;
  int col = mouse_col;
  int grid = mouse_grid;
  varnumber_T winid = 0;
  varnumber_T winrow = 0;
  varnumber_T wincol = 0;
  linenr_T lnum = 0;
  varnumber_T column = 0;
  colnr_T coladd = 0;

  tv_dict_alloc_ret(rettv);
  dict_T *d = rettv->vval.v_dict;

  tv_dict_add_nr(d, S_LEN("screenrow"), (varnumber_T)mouse_row + 1);
  tv_dict_add_nr(d, S_LEN("screencol"), (varnumber_T)mouse_col + 1);

  win_T *wp = mouse_find_win_inner(&grid, &row, &col);
  if (wp != NULL) {
    int height = wp->w_height + wp->w_hsep_height + wp->w_status_height;
    // The height is adjusted by 1 when there is a bottom border. This is not
    // necessary for a top border since `row` starts at -1 in that case.
    if (row < height + wp->w_border_adj[2]) {
      winid = wp->handle;
      winrow = row + 1 + wp->w_winrow_off;  // Adjust by 1 for top border
      wincol = col + 1 + wp->w_wincol_off;  // Adjust by 1 for left border
      if (row >= 0 && row < wp->w_height && col >= 0 && col < wp->w_width) {
        mouse_comp_pos(wp, &row, &col, &lnum);
        col = vcol2col(wp, lnum, col, &coladd);
        column = col + 1;
      }
    }
  }
  tv_dict_add_nr(d, S_LEN("winid"), winid);
  tv_dict_add_nr(d, S_LEN("winrow"), winrow);
  tv_dict_add_nr(d, S_LEN("wincol"), wincol);
  tv_dict_add_nr(d, S_LEN("line"), (varnumber_T)lnum);
  tv_dict_add_nr(d, S_LEN("column"), column);
  tv_dict_add_nr(d, S_LEN("coladd"), coladd);
}
