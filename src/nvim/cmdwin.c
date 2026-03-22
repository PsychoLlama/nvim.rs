// cmdwin.c: Command-line window (q:, q/, q?) implementation.

#include <stdbool.h>
#include <stddef.h>
#include <string.h>

#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/clipboard.h"
#include "nvim/cmdhist.h"
#include "nvim/cursor.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/ex_eval.h"
#include "nvim/ex_getln.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/getchar.h"
#include "nvim/globals.h"
#include "nvim/highlight_defs.h"
#include "nvim/keycodes.h"
#include "nvim/macros_defs.h"
#include "nvim/mapping.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/memory_defs.h"
#include "nvim/strings.h"
#include "nvim/message.h"
#include "nvim/mouse.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/popupmenu.h"
#include "nvim/pos_defs.h"
#include "nvim/state.h"
#include "nvim/state_defs.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_defs.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"
#include "nvim/cmdwin.h"

// Rust helpers used by nvim_open_cmdwin
extern int rs_cmdwin_can_open(int is_in_cmdwin, int buf_locked, int cmdline_star);
extern int rs_cmdwin_split_invalid(int win_valid, int same_win, int buf_valid, int buf_changed);
extern int rs_cmdwin_buffer_invalid(int buf_ok, int cmdwin_valid, int same_win,
                                    int old_win_valid, int old_buf_valid, int buf_changed);
extern int rs_cmdwin_cleanup_had_error(int win_valid, int buf_valid, int buf_changed);
extern int rs_cmdwin_to_hist_type(int cmdwin_type);
extern int rs_cmdwin_needs_tab_mapping(int histtype, OptInt p_wc);
extern int rs_cmdwin_needs_vim_filetype(int histtype);
extern int rs_win_valid(win_T *win);
extern int rs_last_window(win_T *win);
extern void rs_win_size_save(garray_T *gap);
extern void rs_win_size_restore(garray_T *gap);
extern void rs_clear_showcmd(void);

#include "cmdwin.c.generated.h"

/// Trigger CmdwinEnter autocommand for the current cmdwin_type.
static void trigger_cmdwinenter(void)
{
  char typestr[2] = { (char)cmdwin_type, NUL };
  apply_autocmds(EVENT_CMDWINENTER, typestr, typestr, false, curbuf);
}

/// Trigger CmdwinLeave autocommand for the current cmdwin_type.
static void trigger_cmdwinleave(void)
{
  char typestr[2] = { (char)cmdwin_type, NUL };
  apply_autocmds(EVENT_CMDWINLEAVE, typestr, typestr, false, curbuf);
}

static const char e_cmdwin_active_window_or_buffer_changed_or_deleted[]
  = N_("E199: Active window or buffer changed or deleted");

/// Open a command-line window for the current command-line type.
/// The cmdline window is used for editing with q:, q/, q?.
int nvim_open_cmdwin(void)
{
  bufref_T old_curbuf;
  bufref_T bufref;
  win_T *old_curwin = curwin;
  int i;
  garray_T winsizes;
  int save_restart_edit = restart_edit;
  int save_State = State;
  bool save_exmode = exmode_active;
  bool save_cmdmsg_rl = cmdmsg_rl;

  // Use Rust helper to check if cmdwin can be opened.
  // Can't do this when text or buffer is locked, recursively, or typing a password.
  if (rs_cmdwin_can_open(cmdwin_type != 0, text_or_buf_locked(), cmdline_star) != 0) {
    beep_flush();
    return K_IGNORE;
  }

  set_bufref(&old_curbuf, curbuf);

  // Save current window sizes.
  rs_win_size_save(&winsizes);

  // When using completion in Insert mode with <C-R>=<C-F> one can open the
  // command line window, but we don't want the popup menu then.
  pum_undisplay(true);

  // don't use a new tab page
  cmdmod.cmod_tab = 0;
  cmdmod.cmod_flags |= CMOD_NOSWAPFILE;

  // Create a window for the command-line buffer.
  if (win_split((int)p_cwh, WSP_BOT) == FAIL) {
    beep_flush();
    ga_clear(&winsizes);
    return K_IGNORE;
  }
  // win_split() autocommands may have messed with the old window or buffer.
  // Treat it as abandoning this command-line.
  // Use Rust helper for validation check.
  if (rs_cmdwin_split_invalid(rs_win_valid(old_curwin), curwin == old_curwin,
                              bufref_valid(&old_curbuf),
                              old_curwin->w_buffer != old_curbuf.br_buf)) {
    beep_flush();
    ga_clear(&winsizes);
    return Ctrl_C;
  }
  // Don't let quitting the More prompt make this fail.
  got_int = false;

  // Set "cmdwin_..." variables before any autocommands may mess things up.
  cmdwin_type = nvim_get_cmdline_type();
  cmdwin_level = nvim_get_ccline_level();
  cmdwin_win = curwin;
  cmdwin_old_curwin = old_curwin;

  // Create empty command-line buffer.  Be especially cautious of BufLeave
  // autocommands from do_ecmd(), as cmdwin restrictions do not apply to them!
  const int newbuf_status = buf_open_scratch(0, NULL);
  const bool cmdwin_valid = rs_win_valid(cmdwin_win);
  // Use Rust helper for buffer creation validation.
  if (rs_cmdwin_buffer_invalid(newbuf_status == OK, cmdwin_valid, curwin == cmdwin_win,
                               rs_win_valid(old_curwin), bufref_valid(&old_curbuf),
                               old_curwin->w_buffer != old_curbuf.br_buf)) {
    if (newbuf_status == OK) {
      set_bufref(&bufref, curbuf);
    }
    if (cmdwin_valid && !rs_last_window(cmdwin_win)) {
      win_close(cmdwin_win, true, false);
    }
    // win_close() autocommands may have already deleted the buffer.
    if (newbuf_status == OK && bufref_valid(&bufref) && bufref.br_buf != curbuf) {
      close_buffer(NULL, bufref.br_buf, DOBUF_WIPE, false, false);
    }

    cmdwin_type = 0;
    cmdwin_level = 0;
    cmdwin_win = NULL;
    cmdwin_old_curwin = NULL;
    beep_flush();
    ga_clear(&winsizes);
    return Ctrl_C;
  }
  cmdwin_buf = curbuf;

  // Command-line buffer has bufhidden=wipe, unlike a true "scratch" buffer.
  set_option_value_give_err(kOptBufhidden, STATIC_CSTR_AS_OPTVAL("wipe"), OPT_LOCAL);
  curbuf->b_p_ma = true;
  curwin->w_p_fen = false;
  curwin->w_p_rl = cmdmsg_rl;
  cmdmsg_rl = false;

  // Don't allow switching to another buffer.
  curbuf->b_ro_locked++;

  // Showing the prompt may have set need_wait_return, reset it.
  need_wait_return = false;

  // Use Rust helper to get history type and check if Tab mapping/filetype needed.
  const int histtype = rs_cmdwin_to_hist_type(cmdwin_type);
  if (rs_cmdwin_needs_tab_mapping(histtype, p_wc)) {
    add_map("<Tab>", "<C-X><C-V>", MODE_INSERT, true);
    add_map("<Tab>", "a<C-X><C-V>", MODE_NORMAL, true);
  }
  if (rs_cmdwin_needs_vim_filetype(histtype)) {
    set_option_value_give_err(kOptFiletype, STATIC_CSTR_AS_OPTVAL("vim"), OPT_LOCAL);
  }
  curbuf->b_ro_locked--;

  // Reset 'textwidth' after setting 'filetype' (the Vim filetype plugin
  // sets 'textwidth' to 78).
  curbuf->b_p_tw = 0;

  // Fill the buffer with the history.
  init_history();
  if (get_hislen() > 0 && histtype != HIST_INVALID) {
    i = *get_hisidx(histtype);
    if (i >= 0) {
      linenr_T lnum = 0;
      do {
        if (++i == get_hislen()) {
          i = 0;
        }
        if (get_histentry(histtype)[i].hisstr != NULL) {
          ml_append(lnum++, get_histentry(histtype)[i].hisstr, 0, false);
        }
      } while (i != *get_hisidx(histtype));
    }
  }

  // Replace the empty last line with the current command-line and put the
  // cursor there.
  ml_replace(curbuf->b_ml.ml_line_count, nvim_get_ccline_cmdbuff(), true);
  curwin->w_cursor.lnum = curbuf->b_ml.ml_line_count;
  curwin->w_cursor.col = nvim_get_ccline_cmdpos();
  changed_line_abv_curs();
  invalidate_botline(curwin);
  if (ui_has(kUICmdline)) {
    cmdline_was_last_drawn = false;
    nvim_set_ccline_redraw_state(kCmdRedrawNone);
    ui_call_cmdline_hide(nvim_get_ccline_level(), false);
  }
  redraw_later(curwin, UPD_SOME_VALID);

  // No Ex mode here!
  exmode_active = false;

  State = MODE_NORMAL;
  setmouse();
  rs_clear_showcmd();

  // Reset here so it can be set by a CmdwinEnter autocommand.
  cmdwin_result = 0;

  // Trigger CmdwinEnter autocommands.
  trigger_cmdwinenter();
  if (restart_edit != 0) {  // autocmd with ":startinsert"
    stuffcharReadbuff(K_NOP);
  }

  i = RedrawingDisabled;
  RedrawingDisabled = 0;
  int save_count = save_batch_count();

  // Call the main loop until <CR> or CTRL-C is typed.
  normal_enter(true, false);

  RedrawingDisabled = i;
  restore_batch_count(save_count);

  const bool save_KeyTyped = KeyTyped;

  // Trigger CmdwinLeave autocommands.
  trigger_cmdwinleave();

  // Restore KeyTyped in case it is modified by autocommands
  KeyTyped = save_KeyTyped;

  cmdwin_type = 0;
  cmdwin_level = 0;
  cmdwin_buf = NULL;
  cmdwin_win = NULL;
  cmdwin_old_curwin = NULL;

  exmode_active = save_exmode;

  // Safety check: The old window or buffer was changed or deleted: It's a bug
  // when this happens! Use Rust helper for validation.
  if (rs_cmdwin_cleanup_had_error(rs_win_valid(old_curwin), bufref_valid(&old_curbuf),
                                  old_curwin->w_buffer != old_curbuf.br_buf)) {
    cmdwin_result = Ctrl_C;
    emsg(_(e_cmdwin_active_window_or_buffer_changed_or_deleted));
  } else {
    win_T *wp;
    // autocmds may abort script processing
    if (aborting() && cmdwin_result != K_IGNORE) {
      cmdwin_result = Ctrl_C;
    }
    // Set the new command line from the cmdline buffer.
    dealloc_cmdbuff();

    if (cmdwin_result == K_XF1 || cmdwin_result == K_XF2) {  // :qa[!] typed
      const char *p = (cmdwin_result == K_XF2) ? "qa" : "qa!";
      size_t plen = (cmdwin_result == K_XF2) ? 2 : 3;

      if (histtype == HIST_CMD) {
        // Execute the command directly.
        nvim_set_ccline_cmdbuff(xmemdupz(p, plen));
        nvim_set_ccline_cmdlen((int)plen);
        nvim_set_ccline_cmdbufflen((int)plen + 1);
        cmdwin_result = CAR;
      } else {
        // First need to cancel what we were doing.
        stuffcharReadbuff(':');
        stuffReadbuff(p);
        stuffcharReadbuff(CAR);
      }
    } else if (cmdwin_result == Ctrl_C) {
      // :q or :close, don't execute any command
      // and don't modify the cmd window.
      nvim_set_ccline_cmdbuff(NULL);
    } else {
      nvim_set_ccline_cmdlen(get_cursor_line_len());
      nvim_set_ccline_cmdbufflen(nvim_get_ccline_cmdlen() + 1);
      nvim_set_ccline_cmdbuff(xstrnsave(get_cursor_line_ptr(), (size_t)nvim_get_ccline_cmdlen()));
    }

    if (nvim_get_ccline_cmdbuff() == NULL) {
      nvim_set_ccline_cmdbuff(xmemdupz("", 0));
      nvim_set_ccline_cmdlen(0);
      nvim_set_ccline_cmdbufflen(1);
      nvim_set_ccline_cmdpos(0);
      cmdwin_result = Ctrl_C;
    } else {
      nvim_set_ccline_cmdpos(curwin->w_cursor.col);
      // If the cursor is on the last character, it probably should be after it.
      if (nvim_get_ccline_cmdpos() == nvim_get_ccline_cmdlen() - 1
          || nvim_get_ccline_cmdpos() > nvim_get_ccline_cmdlen()) {
        nvim_set_ccline_cmdpos(nvim_get_ccline_cmdlen());
      }
      if (cmdwin_result == K_IGNORE) {
        nvim_set_ccline_cmdspos(cmd_screencol(nvim_get_ccline_cmdpos()));
        redrawcmd();
      }
    }

    // Avoid command-line window first character being concealed.
    curwin->w_p_cole = 0;
    // First go back to the original window.
    wp = curwin;
    set_bufref(&bufref, curbuf);
    skip_win_fix_cursor = true;
    win_goto(old_curwin);

    // win_goto() may trigger an autocommand that already closes the
    // cmdline window.
    if (rs_win_valid(wp) && wp != curwin) {
      win_close(wp, true, false);
    }

    // win_close() may have already wiped the buffer when 'bh' is
    // set to 'wipe', autocommands may have closed other windows
    if (bufref_valid(&bufref) && bufref.br_buf != curbuf) {
      close_buffer(NULL, bufref.br_buf, DOBUF_WIPE, false, false);
    }

    // Restore window sizes.
    rs_win_size_restore(&winsizes);
    skip_win_fix_cursor = false;
  }

  ga_clear(&winsizes);
  restart_edit = save_restart_edit;
  cmdmsg_rl = save_cmdmsg_rl;

  State = save_State;
  may_trigger_modechanged();
  setmouse();
  setcursor();

  return cmdwin_result;
}
