// message.c: functions for displaying messages on the command line

#include <assert.h>
#include <inttypes.h>
#include <limits.h>
#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <uv.h>

#include "klib/kvec.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/buffer_defs.h"
#include "nvim/channel.h"
#include "nvim/charset.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/event/defs.h"
#include "nvim/event/loop.h"
#include "nvim/event/multiqueue.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_eval.h"
#include "nvim/ex_getln.h"
#include "nvim/fileio.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/highlight.h"
#include "nvim/highlight_defs.h"
#include "nvim/highlight_group.h"
#include "nvim/indent.h"
#include "nvim/input.h"
#include "nvim/keycodes.h"
#include "nvim/log.h"
#include "nvim/main.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memory.h"
#include "nvim/memory_defs.h"
#include "nvim/message.h"
#include "nvim/message_defs.h"
#include "nvim/mouse.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/os/fs.h"
#include "nvim/os/input.h"
#include "nvim/os/os.h"
#include "nvim/os/time.h"
#include "nvim/pos_defs.h"
#include "nvim/regexp.h"
#include "nvim/register.h"
#include "nvim/runtime.h"
#include "nvim/runtime_defs.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_compositor.h"
#include "nvim/ui_defs.h"
#include "nvim/vim_defs.h"

// msgchunk_T is now defined in message_defs.h

extern int confirm_msg_used;   // owned by Rust (dialog.rs)
#include "message.c.generated.h"
// do_more_prompt migrated to Rust (scrollback.rs) — forward declaration for callers in this file
bool do_more_prompt(int typed_char);
extern char *confirm_msg;      // owned by Rust (dialog.rs)
extern char *confirm_buttons;  // owned by Rust (dialog.rs)

extern MessageHistoryEntry *msg_hist_last;   // owned by Rust (history.rs)
extern MessageHistoryEntry *msg_hist_first;  // owned by Rust (history.rs)
extern MessageHistoryEntry *msg_hist_temp;   // owned by Rust (history.rs)
extern int msg_hist_len;                     // owned by Rust (history.rs)

// msg_flags and msg_wait owned by Rust (misc.rs)
extern int msg_flags;
extern int msg_wait;

extern FILE *verbose_fd;        // owned by Rust (verbose.rs)
extern bool verbose_did_open;   // owned by Rust (verbose.rs)

// Extended msg state owned by Rust (display.rs)
extern garray_T msg_ext_last_chunk;
extern sattr_T msg_ext_last_attr;
extern int msg_ext_last_hl_id;

extern int msg_grid_pos_at_flush;  // owned by Rust (misc.rs)

extern int64_t msg_id_next;  // owned by Rust (display.rs)

void nvim_ui_ext_msg_set_pos(int row, bool scrolled)
{
  char buf[MAX_SCHAR_SIZE];
  size_t size = schar_get(buf, curwin->w_p_fcs_chars.msgsep);
  ui_call_msg_set_pos(msg_grid.handle, row, scrolled,
                      (String){ .data = buf, .size = size }, msg_grid.zindex,
                      (int)msg_grid.comp_index);
  msg_grid.pending_comp_index_update = false;
}

// msg_grid_set_pos() migrated to Rust (misc.rs) with #[export_name]

extern msgchunk_T *last_msgchunk;  // owned by Rust (scrollback.rs)

// Rust implementation of emsg_not_now()
extern int rs_emsg_not_now(void);
// Rust implementation of msg_show_console_dialog()
extern char *rs_msg_show_console_dialog(const char *message, const char *buttons, int dfltbutton);

// Forward declarations (non-static functions accessible from Rust via extern "C")
void msg_puts_printf(const char *str, ptrdiff_t maxlen);  // defined in Rust (output.rs) with #[export_name]
void msg_puts_display(const char *str, int maxlen, int hl_id, int recurse);  // defined in Rust (output.rs) with #[export_name]
void hit_return_msg(bool newline_sb);  // defined in Rust (misc.rs) with #[export_name]
void msg_moremsg(bool full);  // defined in Rust (misc.rs) with #[export_name]
void msg_ext_emit_chunk(void);  // defined in Rust (display.rs) with #[no_mangle]
void ex_messages(exarg_T *eap);  // defined in Rust (display.rs) with #[export_name]
void msg_hist_add_multihl(MsgID msg_id, HlMessage msg, bool temp, MessageData *msg_data);
// Formerly static; now defined in Rust (scrollback.rs) with #[export_name]
msgchunk_T *msg_sb_start(msgchunk_T *mps);
void store_sb_text(const char **sb_str, const char *s, int hl_id, int *sb_col, int finish);
void inc_msg_scrolled(void);

// nvim_msg_set_pos_for_scroll migrated to Rust (misc.rs) — calls nvim_ui_ext_msg_set_pos

// nvim_msg_show_empty() migrated to Rust (display.rs) with #[export_name]

// message_filtered() is implemented in Rust (#[export_name]); this is its C helper.
bool nvim_message_filtered_impl(const char *msg)
{
  if (cmdmod.cmod_filter_regmatch.regprog == NULL) {
    return false;
  }
  bool match = vim_regexec(&cmdmod.cmod_filter_regmatch, msg, 0);
  return cmdmod.cmod_filter_force ? match : !match;
}

// nvim_msg_ui_refresh_impl and nvim_msg_ui_flush_impl migrated to Rust (misc.rs)
// msg_grid_validate() migrated to Rust (misc.rs) with #[export_name]

// Avoid starting a new message for each chunk and adding message to history in msg_keep().
extern bool is_multihl;  // owned by Rust (misc.rs)

// format_progress_message() and msg_multihl() migrated to Rust (display.rs)

/// Shows a printf-style message with highlight id.
///
/// Note: Caller must check the resulting string is shorter than IOSIZE!!!
///
/// @see semsg
/// @see swmsg
///
/// @param s printf-style format message
int smsg(int hl_id, const char *s, ...)
  FUNC_ATTR_PRINTF(2, 3)
{
  va_list arglist;

  va_start(arglist, s);
  vim_vsnprintf(IObuff, IOSIZE, s, arglist);
  va_end(arglist);
  return msg(IObuff, hl_id);
}

int smsg_keep(int hl_id, const char *s, ...)
  FUNC_ATTR_PRINTF(2, 3)
{
  va_list arglist;

  va_start(arglist, s);
  vim_vsnprintf(IObuff, IOSIZE, s, arglist);
  va_end(arglist);
  return msg_keep(IObuff, hl_id, true, false);
}

// get_emsg_source, get_emsg_lnum, msg_source, emsg_multiline migrated to Rust (error.rs)

void emsg_invreg(int name) { semsg(_("E354: Invalid register name: '%s'"), transchar_buf(NULL, name)); }

/// Print an error message with unknown number of arguments
///
/// @return whether the message was displayed
bool semsg(const char *const fmt, ...)
  FUNC_ATTR_PRINTF(1, 2)
{
  bool ret;

  va_list ap;
  va_start(ap, fmt);
  ret = semsgv(fmt, ap);
  va_end(ap);

  return ret;
}

#define MULTILINE_BUFSIZE 8192

bool semsg_multiline(const char *kind, const char *const fmt, ...)
{
  bool ret;
  va_list ap;

  static char errbuf[MULTILINE_BUFSIZE];
  if (rs_emsg_not_now()) {
    return true;
  }

  va_start(ap, fmt);
  vim_vsnprintf(errbuf, sizeof(errbuf), fmt, ap);
  va_end(ap);

  ret = emsg_multiline(errbuf, kind, HLF_E, true);

  return ret;
}

/// Print an error message with unknown number of arguments
static bool semsgv(const char *fmt, va_list ap)
{
  static char errbuf[IOSIZE];
  if (rs_emsg_not_now()) {
    return true;
  }

  vim_vsnprintf(errbuf, sizeof(errbuf), fmt, ap);

  return emsg(errbuf);
}

/// Same as semsg(...) but abort on error when ABORT_ON_INTERNAL_ERROR is
/// defined. It is used for internal errors only, so that they can be
/// detected when fuzzing vim.
void siemsg(const char *s, ...)
{
  if (rs_emsg_not_now()) {
    return;
  }

  va_list ap;
  va_start(ap, s);
  semsgv(s, ap);
  va_end(ap);
#ifdef ABORT_ON_INTERNAL_ERROR
  msg_putchar('\n');  // avoid overwriting the error message
  ui_flush();
  abort();
#endif
}

/// Give an "Internal error" message.
void internal_error(const char *where) { siemsg(_(e_intern2), where); }

static void msg_semsg_event(void **argv)
{
  char *s = argv[0];
  emsg(s);
  xfree(s);
}

void msg_schedule_semsg(const char *const fmt, ...)
  FUNC_ATTR_PRINTF(1, 2)
{
  va_list ap;
  va_start(ap, fmt);
  vim_vsnprintf(IObuff, IOSIZE, fmt, ap);
  va_end(ap);

  char *s = xstrdup(IObuff);
  loop_schedule_deferred(&main_loop, event_create(msg_semsg_event, s));
}

static void msg_semsg_multiline_event(void **argv)
{
  char *s = argv[0];
  emsg_multiline(s, "emsg", HLF_E, true);
  xfree(s);
}

void msg_schedule_semsg_multiline(const char *const fmt, ...)
{
  va_list ap;
  va_start(ap, fmt);
  vim_vsnprintf(IObuff, IOSIZE, fmt, ap);
  va_end(ap);

  char *s = xstrdup(IObuff);
  loop_schedule_deferred(&main_loop, event_create(msg_semsg_multiline_event, s));
}

// hl_msg_free, msg_hist_add migrated to Rust: src/nvim-rs/message/src/history.rs

void do_autocmd_progress(MsgID msg_id, HlMessage msg, MessageData *msg_data)
{
  if (!has_event(EVENT_PROGRESS)) {
    return;
  }

  MAXSIZE_TEMP_DICT(data, 7);
  ArrayOf(String) messages = ARRAY_DICT_INIT;
  for (size_t i = 0; i < msg.size; i++) {
    ADD(messages, STRING_OBJ(msg.items[i].text));
  }

  PUT_C(data, "id", OBJECT_OBJ(msg_id));
  PUT_C(data, "text", ARRAY_OBJ(messages));
  if (msg_data != NULL) {
    PUT_C(data, "percent", INTEGER_OBJ(msg_data->percent));
    PUT_C(data, "status", STRING_OBJ(msg_data->status));
    PUT_C(data, "title", STRING_OBJ(msg_data->title));
    PUT_C(data, "data", DICT_OBJ(msg_data->data));
  }

  apply_autocmds_group(EVENT_PROGRESS, msg_data ? msg_data->title.data : "", NULL, true,
                       AUGROUP_ALL, NULL, NULL, &DICT_OBJ(data));
  kv_destroy(messages);
}

// msg_hist_add_multihl migrated to Rust: src/nvim-rs/message/src/history.rs
// ex_messages() migrated to Rust: src/nvim-rs/message/src/display.rs (rs_ex_messages)

/// Wait for the user to hit a key (normally Enter)
///
/// @param redraw  if true, redraw the entire screen UPD_NOT_VALID
///                if false, do a normal redraw
///                if -1, don't redraw at all
void wait_return(int redraw)
{
  int c;
  int had_got_int;
  FILE *save_scriptout;

  if (redraw == true) {
    redraw_all_later(UPD_NOT_VALID);
  }

  if (ui_has(kUIMessages)) {
    prompt_for_input("Press any key to continue", HLF_M, true, NULL);
    return;
  }

  // If using ":silent cmd", don't wait for a return.  Also don't set
  // need_wait_return to do it later.
  if (msg_silent != 0) {
    return;
  }

  if (headless_mode && !ui_active()) {
    return;
  }

  // When inside vgetc(), we can't wait for a typed character at all.
  // With the global command (and some others) we only need one return at
  // the end. Adjust cmdline_row to avoid the next message overwriting the
  // last one.
  if (vgetc_busy > 0) {
    return;
  }
  need_wait_return = true;
  if (no_wait_return) {
    if (!exmode_active) {
      cmdline_row = msg_row;
    }
    return;
  }

  redir_off = true;             // don't redirect this message
  int oldState = State;
  if (quit_more) {
    c = CAR;                    // just pretend CR was hit
    quit_more = false;
    got_int = false;
  } else if (exmode_active) {
    msg_puts(" ");              // make sure the cursor is on the right line
    c = CAR;                    // no need for a return in ex mode
    got_int = false;
  } else {
    State = MODE_HITRETURN;
    setmouse();
    cmdline_row = msg_row;
    // Avoid the sequence that the user types ":" at the hit-return prompt
    // to start an Ex command, but the file-changed dialog gets in the
    // way.
    if (need_check_timestamps) {
      check_timestamps(false);
    }

    // if cmdheight=0, we need to scroll in the first line of msg_grid upon the screen
    if (p_ch == 0 && !ui_has(kUIMessages) && !msg_scrolled) {
      msg_grid_validate();
      msg_scroll_up(false, true);
      msg_scrolled++;
      cmdline_row = Rows - 1;
    }

    if (msg_flags & kOptMoptFlagHitEnter) {
      hit_return_msg(true);

      do {
        // Remember "got_int", if it is set vgetc() probably returns a
        // CTRL-C, but we need to loop then.
        had_got_int = got_int;

        // Don't do mappings here, we put the character back in the
        // typeahead buffer.
        no_mapping++;
        allow_keys++;

        // Temporarily disable Recording. If Recording is active, the
        // character will be recorded later, since it will be added to the
        // typebuf after the loop
        const int save_reg_recording = reg_recording;
        save_scriptout = scriptout;
        reg_recording = 0;
        scriptout = NULL;
        c = safe_vgetc();
        if (had_got_int && !global_busy) {
          got_int = false;
        }
        no_mapping--;
        allow_keys--;
        reg_recording = save_reg_recording;
        scriptout = save_scriptout;

        // Allow scrolling back in the messages.
        // Also accept scroll-down commands when messages fill the screen,
        // to avoid that typing one 'j' too many makes the messages
        // disappear.
        if (p_more) {
          if (c == 'b' || c == Ctrl_B || c == 'k' || c == 'u' || c == 'g'
              || c == K_UP || c == K_PAGEUP) {
            if (msg_scrolled > Rows) {
              // scroll back to show older messages
              do_more_prompt(c);
            } else {
              msg_didout = false;
              c = K_IGNORE;
              msg_col = 0;
            }
            if (quit_more) {
              c = CAR;  // just pretend CR was hit
              quit_more = false;
              got_int = false;
            } else if (c != K_IGNORE) {
              c = K_IGNORE;
              hit_return_msg(false);
            }
          } else if (msg_scrolled > Rows - 2
                     && (c == 'j' || c == 'd' || c == 'f' || c == Ctrl_F
                         || c == K_DOWN || c == K_PAGEDOWN)) {
            c = K_IGNORE;
          }
        }
      } while ((had_got_int && c == Ctrl_C)
               || c == K_IGNORE
               || c == K_LEFTDRAG || c == K_LEFTRELEASE
               || c == K_MIDDLEDRAG || c == K_MIDDLERELEASE
               || c == K_RIGHTDRAG || c == K_RIGHTRELEASE
               || c == K_MOUSELEFT || c == K_MOUSERIGHT
               || c == K_MOUSEDOWN || c == K_MOUSEUP
               || c == K_MOUSEMOVE);
      os_breakcheck();

      // Avoid that the mouse-up event causes visual mode to start.
      if (c == K_LEFTMOUSE || c == K_MIDDLEMOUSE || c == K_RIGHTMOUSE
          || c == K_X1MOUSE || c == K_X2MOUSE) {
        jump_to_mouse(MOUSE_SETPOS, NULL, 0);
      } else if (vim_strchr("\r\n ", c) == NULL && c != Ctrl_C) {
        // Put the character back in the typeahead buffer.  Don't use the
        // stuff buffer, because lmaps wouldn't work.
        ins_char_typebuf(vgetc_char, vgetc_mod_mask, true);
        do_redraw = true;  // need a redraw even though there is typeahead
      }
    } else {
      c = CAR;
      // Wait to allow the user to verify the output.
      do_sleep(msg_wait, true);
    }
  }
  redir_off = false;

  // If the user hits ':', '?' or '/' we get a command line from the next
  // line.
  if (c == ':' || c == '?' || c == '/') {
    if (!exmode_active) {
      cmdline_row = msg_row;
    }
    skip_redraw = true;  // skip redraw once
    do_redraw = false;
  }

  // If the screen size changed screen_resize() will redraw the screen.
  // Otherwise the screen is only redrawn if 'redraw' is set and no ':'
  // typed.
  int tmpState = State;
  State = oldState;  // restore State before screen_resize()
  setmouse();
  msg_check();
  need_wait_return = false;
  did_wait_return = true;
  emsg_on_display = false;      // can delete error message now
  lines_left = -1;              // reset lines_left at next msg_start()
  reset_last_sourcing();
  if (keep_msg != NULL && vim_strsize(keep_msg) >=
      (Rows - cmdline_row - 1) * Columns + sc_col) {
    XFREE_CLEAR(keep_msg);          // don't redisplay message, it's too long
  }

  if (tmpState == MODE_SETWSIZE) {       // got resize event while in vgetc()
    ui_refresh();
  } else if (!skip_redraw) {
    if (redraw == true || (msg_scrolled != 0 && redraw != -1)) {
      redraw_later(curwin, UPD_VALID);
    }
  }
}

// hit_return_msg, msgmore, str2special_arena migrated to Rust (misc.rs/keys.rs)

/// print line for :print or :list command
void msg_prt_line(const char *s, bool list)
{
  schar_T sc;
  int col = 0;
  int n_extra = 0;
  schar_T sc_extra = 0;
  schar_T sc_final = 0;
  const char *p_extra = NULL;  // init to make SASC shut up. ASCII only!
  int n;
  int hl_id = 0;
  const char *lead = NULL;
  bool in_multispace = false;
  int multispace_pos = 0;
  const char *trail = NULL;
  int l;

  if (curwin->w_p_list) {
    list = true;
  }

  if (list) {
    // find start of trailing whitespace
    if (curwin->w_p_lcs_chars.trail) {
      trail = s + strlen(s);
      while (trail > s && ascii_iswhite(trail[-1])) {
        trail--;
      }
    }
    // find end of leading whitespace
    if (curwin->w_p_lcs_chars.lead || curwin->w_p_lcs_chars.leadmultispace != NULL) {
      lead = s;
      while (ascii_iswhite(lead[0])) {
        lead++;
      }
      // in a line full of spaces all of them are treated as trailing
      if (*lead == NUL) {
        lead = NULL;
      }
    }
  }

  // output a space for an empty line, otherwise the line will be overwritten
  if (*s == NUL && !(list && curwin->w_p_lcs_chars.eol != NUL)) {
    msg_putchar(' ');
  }

  while (!got_int) {
    if (n_extra > 0) {
      n_extra--;
      if (n_extra == 0 && sc_final) {
        sc = sc_final;
      } else if (sc_extra) {
        sc = sc_extra;
      } else {
        assert(p_extra != NULL);
        sc = schar_from_ascii((unsigned char)(*p_extra++));
      }
    } else if ((l = utfc_ptr2len(s)) > 1) {
      col += utf_ptr2cells(s);
      char buf[MB_MAXBYTES + 1];
      if (l >= MB_MAXBYTES) {
        xstrlcpy(buf, "?", sizeof(buf));
      } else if (curwin->w_p_lcs_chars.nbsp != NUL && list
                 && (utf_ptr2char(s) == 160 || utf_ptr2char(s) == 0x202f)) {
        schar_get(buf, curwin->w_p_lcs_chars.nbsp);
      } else {
        memmove(buf, s, (size_t)l);
        buf[l] = NUL;
      }
      msg_puts(buf);
      s += l;
      continue;
    } else {
      hl_id = 0;
      int c = (uint8_t)(*s++);
      if (c >= 0x80) {  // Illegal byte
        col += utf_char2cells(c);
        msg_putchar(c);
        continue;
      }
      sc_extra = NUL;
      sc_final = NUL;
      if (list) {
        in_multispace = c == ' ' && (*s == ' '
                                     || (col > 0 && s[-2] == ' '));
        if (!in_multispace) {
          multispace_pos = 0;
        }
      }
      if (c == TAB && (!list || curwin->w_p_lcs_chars.tab1)) {
        // tab amount depends on current column
        n_extra = tabstop_padding(col, curbuf->b_p_ts,
                                  curbuf->b_p_vts_array) - 1;
        if (!list) {
          sc = schar_from_ascii(' ');
          sc_extra = schar_from_ascii(' ');
        } else {
          sc = (n_extra == 0 && curwin->w_p_lcs_chars.tab3)
               ? curwin->w_p_lcs_chars.tab3
               : curwin->w_p_lcs_chars.tab1;
          sc_extra = curwin->w_p_lcs_chars.tab2;
          sc_final = curwin->w_p_lcs_chars.tab3;
          hl_id = HLF_0;
        }
      } else if (c == NUL && list && curwin->w_p_lcs_chars.eol != NUL) {
        p_extra = "";
        n_extra = 1;
        sc = curwin->w_p_lcs_chars.eol;
        hl_id = HLF_AT;
        s--;
      } else if (c != NUL && (n = byte2cells(c)) > 1) {
        n_extra = n - 1;
        p_extra = transchar_byte_buf(NULL, c);
        sc = schar_from_ascii(*p_extra++);
        // Use special coloring to be able to distinguish <hex> from
        // the same in plain text.
        hl_id = HLF_0;
      } else if (c == ' ') {
        if (lead != NULL && s <= lead && in_multispace
            && curwin->w_p_lcs_chars.leadmultispace != NULL) {
          sc = curwin->w_p_lcs_chars.leadmultispace[multispace_pos++];
          if (curwin->w_p_lcs_chars.leadmultispace[multispace_pos] == NUL) {
            multispace_pos = 0;
          }
          hl_id = HLF_0;
        } else if (lead != NULL && s <= lead && curwin->w_p_lcs_chars.lead != NUL) {
          sc = curwin->w_p_lcs_chars.lead;
          hl_id = HLF_0;
        } else if (trail != NULL && s > trail) {
          sc = curwin->w_p_lcs_chars.trail;
          hl_id = HLF_0;
        } else if (in_multispace
                   && curwin->w_p_lcs_chars.multispace != NULL) {
          sc = curwin->w_p_lcs_chars.multispace[multispace_pos++];
          if (curwin->w_p_lcs_chars.multispace[multispace_pos] == NUL) {
            multispace_pos = 0;
          }
          hl_id = HLF_0;
        } else if (list && curwin->w_p_lcs_chars.space != NUL) {
          sc = curwin->w_p_lcs_chars.space;
          hl_id = HLF_0;
        } else {
          sc = schar_from_ascii(' ');  // SPACE!
        }
      } else {
        sc = schar_from_ascii(c);
      }
    }

    if (sc == NUL) {
      break;
    }

    // TODO(bfredl): this is such baloney. need msg_put_schar
    char buf[MAX_SCHAR_SIZE];
    schar_get(buf, sc);
    msg_puts_hl(buf, hl_id, false);
    col++;
  }
  msg_clr_eos();
}

// msg_puts_len() migrated to Rust: src/nvim-rs/message/src/output.rs (rs_msg_puts_len)
// msg_ext_emit_chunk() migrated to Rust: src/nvim-rs/message/src/display.rs


// msg_line_flush, msg_cursor_goto, inc_msg_scrolled, store_sb_text migrated to Rust (display.rs/misc.rs/scrollback.rs)
// show_sb_text, disp_sb_line migrated to Rust (scrollback.rs) with #[export_name]

// msg_sb_start() migrated to Rust: src/nvim-rs/message/src/chunk.rs (rs_msg_sb_start)

// msg_puts_printf migrated to Rust (output.rs) with #[export_name = "msg_puts_printf"]
// C helpers: nvim_on_print_active, nvim_on_print_call (see below)

// do_more_prompt migrated to Rust (scrollback.rs) with #[export_name]
// DELETED: static bool do_more_prompt(int typed_char) {

// msg_moremsg, repeat_message, msg_ext_init_chunks, msg_ext_emit_chunk, msg_ext_ui_flush,
// msg_ext_flush_showmode, redir_write migrated to Rust (misc.rs/display.rs/verbose.rs)

/// Shows a warning, with optional highlighting.
///
/// @param hl enable highlighting
/// @param fmt printf-style format message
///
/// @see smsg
/// @see semsg
void swmsg(bool hl, const char *const fmt, ...)
  FUNC_ATTR_PRINTF(2, 3)
{
  va_list args;

  va_start(args, fmt);
  vim_vsnprintf(IObuff, IOSIZE, fmt, args);
  va_end(args);

  give_warning(IObuff, hl);
}

// do_dialog() migrated to Rust (dialog.rs) with #[export_name = "do_dialog"]
// copy_char, console_dialog_alloc, msg_show_console_dialog, copy_confirm_hotkeys
// migrated to Rust (dialog.rs) as rs_msg_show_console_dialog
// display_confirm_msg, vim_dialog_yesno, vim_dialog_yesnocancel, vim_dialog_yesnoallcancel
// migrated to Rust (dialog.rs)

// nvim_get_msg_col, nvim_set_msg_col, nvim_get_msg_silent, nvim_set_msg_silent removed:
// Rust callers now access msg_col and msg_silent directly as extern statics.

/// Check if on_print callback is active (for Rust FFI).
int nvim_on_print_active(void) { return on_print.type != kCallbackNone; }

/// Call the on_print callback with a string (for Rust FFI).
void nvim_on_print_call(const char *str)
{
  typval_T argv[1];
  argv[0].v_type = VAR_STRING;
  argv[0].v_lock = VAR_UNLOCKED;
  argv[0].vval.v_string = (char *)str;
  typval_T rettv = TV_INITIAL_VALUE;
  callback_call(&on_print, 1, argv, &rettv);
  tv_clear(&rettv);
}

/// stdout printf helper for Rust FFI (avoids Rust depending on libc directly).
void nvim_printf_stdout(const char *s) { printf("%s", s); }

/// stderr fprintf helper for Rust FFI (avoids Rust depending on libc directly).
void nvim_fprintf_stderr(const char *s) { fprintf(stderr, "%s", s); }
