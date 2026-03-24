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

static void ui_ext_msg_set_pos(int row, bool scrolled)
{
  char buf[MAX_SCHAR_SIZE];
  size_t size = schar_get(buf, curwin->w_p_fcs_chars.msgsep);
  ui_call_msg_set_pos(msg_grid.handle, row, scrolled,
                      (String){ .data = buf, .size = size }, msg_grid.zindex,
                      (int)msg_grid.comp_index);
  msg_grid.pending_comp_index_update = false;
}

void msg_grid_set_pos(int row, bool scrolled)
{
  if (!msg_grid.throttled) {
    ui_ext_msg_set_pos(row, scrolled);
    msg_grid_pos_at_flush = row;
  }
  msg_grid_pos = row;
  if (msg_grid.chars) {
    msg_grid_adj.row_offset = -row;
  }
}

extern msgchunk_T *last_msgchunk;  // owned by Rust (scrollback.rs)

// Rust implementation of emsg_not_now()
extern int rs_emsg_not_now(void);
// Rust implementation of msg_show_console_dialog()
extern char *rs_msg_show_console_dialog(const char *message, const char *buttons, int dfltbutton);

// Forward declarations (non-static functions accessible from Rust via extern "C")
// get_emsg_source and get_emsg_lnum migrated to Rust (error.rs)
void msg_puts_printf(const char *str, ptrdiff_t maxlen);
void msg_puts_display(const char *str, int maxlen, int hl_id, int recurse);
void hit_return_msg(bool newline_sb);  // defined in Rust (misc.rs) with #[export_name]
void msg_moremsg(bool full);  // defined in Rust (misc.rs) with #[export_name]
void msg_ext_emit_chunk(void);  // defined in Rust (display.rs) with #[no_mangle]
void msg_hist_add_multihl(MsgID msg_id, HlMessage msg, bool temp, MessageData *msg_data);
// Formerly static; now defined in Rust (scrollback.rs) with #[export_name]
msgchunk_T *msg_sb_start(msgchunk_T *mps);
void store_sb_text(const char **sb_str, const char *s, int hl_id, int *sb_col, int finish);
void inc_msg_scrolled(void);

void nvim_msg_set_pos_for_scroll(int pos, bool scrolled)
{
  ui_ext_msg_set_pos(pos, scrolled);
}

void nvim_msg_show_empty(void)
{
  ui_call_msg_show(cstr_as_string("empty"), (Array)ARRAY_DICT_INIT, false, false, false,
                   INTEGER_OBJ(-1));
}

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

void msg_grid_validate(void)
{
  grid_assign_handle(&msg_grid);
  bool should_alloc = msg_use_grid();
  int max_rows = Rows - (int)p_ch;
  if (should_alloc && (msg_grid.rows != Rows || msg_grid.cols != Columns
                       || !msg_grid.chars)) {
    // TODO(bfredl): eventually should be set to "invalid". I e all callers
    // will use the grid including clear to EOS if necessary.
    grid_alloc(&msg_grid, Rows, Columns, false, true);
    msg_grid.zindex = kZIndexMessages;

    xfree(msg_grid.dirty_col);
    msg_grid.dirty_col = xcalloc((size_t)Rows, sizeof(*msg_grid.dirty_col));

    // Tricky: allow resize while pager or ex mode is active
    int pos = (State & MODE_ASKMORE) ? 0 : MAX(max_rows - msg_scrolled, 0);
    msg_grid.throttled = false;  // don't throttle in 'cmdheight' area
    msg_grid_set_pos(pos, msg_scrolled);
    ui_comp_put_grid(&msg_grid, pos, 0, msg_grid.rows, msg_grid.cols,
                     false, true);
    ui_call_grid_resize(msg_grid.handle, msg_grid.cols, msg_grid.rows);

    msg_scrolled_at_flush = msg_scrolled;
    msg_grid.mouse_enabled = false;
    msg_grid_adj.target = &msg_grid;
  } else if (!should_alloc && msg_grid.chars) {
    ui_comp_remove_grid(&msg_grid);
    grid_free(&msg_grid);
    XFREE_CLEAR(msg_grid.dirty_col);
    ui_call_grid_destroy(msg_grid.handle);
    msg_grid.throttled = false;
    msg_grid_adj.row_offset = 0;
    msg_grid_adj.target = &default_grid;
    redraw_cmdline = true;
  } else if (msg_grid.chars && !msg_scrolled && msg_grid_pos != max_rows) {
    int diff = msg_grid_pos - max_rows;
    msg_grid_set_pos(max_rows, false);
    if (diff > 0) {
      grid_clear(&msg_grid_adj, Rows - diff, Rows, 0, Columns, HL_ATTR(HLF_MSG));
    }
  }

  if (msg_grid.chars && !msg_scrolled && cmdline_row < msg_grid_pos) {
    // TODO(bfredl): this should already be the case, but fails in some
    // "batched" executions where compute_cmdrow() use stale positions or
    // something.
    cmdline_row = msg_grid_pos;
  }
}

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

// get_emsg_source, get_emsg_lnum migrated to Rust (error.rs) as rs_get_emsg_source/rs_get_emsg_lnum
// msg_source() migrated to Rust: src/nvim-rs/message/src/error.rs (rs_msg_source)
// emsg_multiline() migrated to Rust: src/nvim-rs/message/src/error.rs (rs_emsg_multiline)

void emsg_invreg(int name)
{
  semsg(_("E354: Invalid register name: '%s'"), transchar_buf(NULL, name));
}

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
void internal_error(const char *where)
{
  siemsg(_(e_intern2), where);
}

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

/// :messages command implementation
void ex_messages(exarg_T *eap)
  FUNC_ATTR_NONNULL_ALL
{
  if (strcmp(eap->arg, "clear") == 0) {
    msg_hist_clear(eap->addr_count ? eap->line2 : 0);
    return;
  }

  if (*eap->arg != NUL) {
    emsg(_(e_invarg));
    return;
  }

  Array entries = ARRAY_DICT_INIT;
  MessageHistoryEntry *p = eap->skip ? msg_hist_temp : msg_hist_first;
  int skip = eap->addr_count ? (msg_hist_len - eap->line2) : 0;
  for (; p != NULL; p = p->next) {
    // Skip over count or temporary "g<" messages.
    if ((p->temp && !eap->skip) || skip-- > 0) {
      continue;
    }
    if (ui_has(kUIMessages) && !msg_silent) {
      Array entry = ARRAY_DICT_INIT;
      ADD(entry, CSTR_TO_OBJ(p->kind));
      Array content = ARRAY_DICT_INIT;
      for (uint32_t i = 0; i < kv_size(p->msg); i++) {
        HlMessageChunk chunk = kv_A(p->msg, i);
        Array content_entry = ARRAY_DICT_INIT;
        ADD(content_entry, INTEGER_OBJ(chunk.hl_id ? syn_id2attr(chunk.hl_id) : 0));
        ADD(content_entry, STRING_OBJ(copy_string(chunk.text, NULL)));
        ADD(content_entry, INTEGER_OBJ(chunk.hl_id));
        ADD(content, ARRAY_OBJ(content_entry));
      }
      ADD(entry, ARRAY_OBJ(content));
      ADD(entry, BOOLEAN_OBJ(p->append));
      ADD(entries, ARRAY_OBJ(entry));
    }
    if (redirecting() || !ui_has(kUIMessages)) {
      msg_silent += ui_has(kUIMessages);
      bool needs_clear = false;
      msg_multihl(INTEGER_OBJ(0), p->msg, p->kind, false, false, NULL, &needs_clear);
      msg_silent -= ui_has(kUIMessages);
    }
  }
  if (kv_size(entries) > 0) {
    ui_call_msg_history_show(entries, eap->skip != 0);
    api_free_array(entries);
  }
}

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

// hit_return_msg() migrated to Rust: src/nvim-rs/message/src/misc.rs (rs_hit_return_msg)
// msgmore() migrated to Rust: src/nvim-rs/message/src/misc.rs (rs_msgmore)
// str2special_arena() migrated to Rust: src/nvim-rs/message/src/keys.rs (rs_str2special_arena)

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

/// The display part of msg_puts_len().
/// May be called recursively to display scroll-back text.
void msg_puts_display(const char *str, int maxlen, int hl_id, int recurse)
{
  const char *s = str;
  const char *sb_str = str;
  int sb_col = msg_col;
  int attr = hl_id ? syn_id2attr(hl_id) : 0;

  did_wait_return = false;

  if (ui_has(kUIMessages)) {
    if (attr != msg_ext_last_attr) {
      msg_ext_emit_chunk();
      msg_ext_last_attr = attr;
      msg_ext_last_hl_id = hl_id;
    }
    // Concat pieces with the same highlight
    size_t len = maxlen < 0 ? strlen(str) : strnlen(str, (size_t)maxlen);
    ga_concat_len(&msg_ext_last_chunk, str, len);

    // Find last newline in the message and calculate the current message column
    const char *lastline = strrchr(str, '\n');
    maxlen -= (int)(lastline ? (lastline - str) : 0);
    const char *p = lastline ? lastline + 1 : str;
    int col = (int)(maxlen < 0 ? mb_string2cells(p) : mb_string2cells_len(p, (size_t)(maxlen)));
    msg_col = (lastline ? 0 : msg_col) + col;

    return;
  }

  int print_attr = hl_combine_attr(HL_ATTR(HLF_MSG), attr);
  msg_grid_validate();

  cmdline_was_last_drawn = redrawing_cmdline;

  int msg_row_pending = -1;

  while (true) {
    if (msg_col >= Columns) {
      if (p_more && !recurse) {
        // Store text for scrolling back.
        store_sb_text(&sb_str, s, hl_id, &sb_col, true);
      }
      if (msg_no_more && lines_left == 0) {
        break;
      }

      msg_col = 0;
      msg_row++;
      msg_didout = false;
    }

    if (msg_row >= Rows) {
      msg_row = Rows - 1;

      // When no more prompt and no more room, truncate here
      if (msg_no_more && lines_left == 0) {
        break;
      }

      if (!recurse) {
        if (msg_row_pending >= 0) {
          msg_line_flush();
          msg_row_pending = -1;
        }

        // Scroll the screen up one line.
        msg_scroll_up(true, false);

        inc_msg_scrolled();
        need_wait_return = true;       // may need wait_return() in main()
        redraw_cmdline = true;
        if (cmdline_row > 0 && !exmode_active) {
          cmdline_row--;
        }

        // If screen is completely filled and 'more' is set then wait
        // for a character.
        if (lines_left > 0) {
          lines_left--;
        }

        if (p_more && lines_left == 0 && State != MODE_HITRETURN
            && !msg_no_more && !exmode_active) {
          if (do_more_prompt(NUL)) {
            s = confirm_buttons;
          }
          if (quit_more) {
            return;
          }
        }
      }
    }

    if (!((maxlen < 0 || (int)(s - str) < maxlen) && *s != NUL)) {
      break;
    }

    if (msg_row != msg_row_pending && ((uint8_t)(*s) >= 0x20 || *s == TAB)) {
      // TODO(bfredl): this logic is messier that it has to be. What
      // messages really want is its own private linebuf_char buffer.
      if (msg_row_pending >= 0) {
        msg_line_flush();
      }
      grid_line_start(&msg_grid_adj, msg_row);
      msg_row_pending = msg_row;
    }

    if ((uint8_t)(*s) >= 0x20) {  // printable char
      int cw = utf_ptr2cells(s);
      // avoid including composing chars after the end
      int l = (maxlen >= 0) ? utfc_ptr2len_len(s, (int)((str + maxlen) - s)) : utfc_ptr2len(s);

      if (cw > 1 && (msg_col == Columns - 1)) {
        // Doesn't fit, print a highlighted '>' to fill it up.
        grid_line_puts(msg_col, ">", 1, HL_ATTR(HLF_AT));
        cw = 1;
      } else {
        grid_line_puts(msg_col, s, l, print_attr);
        s += l;
      }
      msg_didout = true;  // remember that line is not empty
      msg_col += cw;
    } else {
      char c = *s++;
      if (c == '\n') {  // go to next line
        msg_didout = false;  // remember that line is empty
        msg_col = 0;
        msg_row++;
        if (p_more && !recurse) {
          // Store text for scrolling back.
          store_sb_text(&sb_str, s, hl_id, &sb_col, true);
        }
      } else if (c == '\r') {  // go to column 0
        msg_col = 0;
      } else if (c == '\b') {  // go to previous char
        if (msg_col) {
          msg_col--;
        }
      } else if (c == TAB) {  // translate Tab into spaces
        do {
          grid_line_puts(msg_col, " ", 1, print_attr);
          msg_col += 1;

          if (msg_col == Columns) {
            break;
          }
        } while (msg_col & 7);
      } else if (c == BELL) {  // beep (from ":sh")
        vim_beep(kOptBoFlagShell);
      }
    }
  }

  if (msg_row_pending >= 0) {
    msg_line_flush();
  }
  msg_cursor_goto(msg_row, msg_col);

  if (p_more && !recurse) {
    store_sb_text(&sb_str, s, hl_id, &sb_col, false);
  }

  msg_check();
}

// msg_line_flush() migrated to Rust: src/nvim-rs/message/src/display.rs (rs_msg_line_flush)
// msg_cursor_goto() migrated to Rust: src/nvim-rs/message/src/misc.rs (rs_msg_cursor_goto_impl)

// inc_msg_scrolled() migrated to Rust: src/nvim-rs/message/src/scrollback.rs (rs_inc_msg_scrolled_full)
// store_sb_text() migrated to Rust: src/nvim-rs/message/src/scrollback.rs (rs_store_sb_text)

/// "g<" command.
void show_sb_text(void)
{
  if (ui_has(kUIMessages)) {
    exarg_T ea = { .arg = "", .skip = true };
    ex_messages(&ea);
    return;
  }
  // Only show something if there is more than one line, otherwise it looks
  // weird, typing a command without output results in one line.
  msgchunk_T *mp = msg_sb_start(last_msgchunk);
  if (mp == NULL || mp->sb_prev == NULL) {
    vim_beep(kOptBoFlagMess);
  } else {
    do_more_prompt('G');
    wait_return(false);
  }
}

// msg_sb_start() migrated to Rust: src/nvim-rs/message/src/chunk.rs (rs_msg_sb_start)

/// Display a screen line from previously displayed text at row "row".
///
/// @return  a pointer to the text for the next line (can be NULL).
static msgchunk_T *disp_sb_line(int row, msgchunk_T *smp)
{
  msgchunk_T *mp = smp;

  while (true) {
    msg_row = row;
    msg_col = mp->sb_msg_col;
    char *p = mp->sb_text;
    msg_puts_display(p, -1, mp->sb_hl_id, true);
    if (mp->sb_eol || mp->sb_next == NULL) {
      break;
    }
    mp = mp->sb_next;
  }

  return mp->sb_next;
}

/// Print a message when there is no valid screen.
void msg_puts_printf(const char *str, const ptrdiff_t maxlen)
{
  const char *s = str;
  char buf[7];
  char *p;

  if (on_print.type != kCallbackNone) {
    typval_T argv[1];
    argv[0].v_type = VAR_STRING;
    argv[0].v_lock = VAR_UNLOCKED;
    argv[0].vval.v_string = (char *)str;
    typval_T rettv = TV_INITIAL_VALUE;
    callback_call(&on_print, 1, argv, &rettv);
    tv_clear(&rettv);
    return;
  }

  while ((maxlen < 0 || s - str < maxlen) && *s != NUL) {
    int len = utf_ptr2len(s);
    if (!(silent_mode && p_verbose == 0)) {
      // NL --> CR NL translation (for Unix, not for "--version")
      p = &buf[0];
      if (*s == '\n' && !info_message) {
        *p++ = '\r';
      }
      memcpy(p, s, (size_t)len);
      *(p + len) = NUL;
      if (info_message) {
        printf("%s", buf);
      } else {
        fprintf(stderr, "%s", buf);
      }
    }

    int cw = utf_char2cells(utf_ptr2char(s));
    // primitive way to compute the current column
    if (*s == '\r' || *s == '\n') {
      msg_col = 0;
      msg_didout = false;
    } else {
      msg_col += cw;
      msg_didout = true;
    }
    s += len;
  }
}

/// Show the more-prompt and handle the user response.
/// This takes care of scrolling back and displaying previously displayed text.
/// When at hit-enter prompt "typed_char" is the already typed character,
/// otherwise it's NUL.
///
/// @return  true when jumping ahead to "confirm_buttons".
static bool do_more_prompt(int typed_char)
{
  static bool entered = false;
  int used_typed_char = typed_char;
  int oldState = State;
  int c;
  bool retval = false;
  bool to_redraw = false;
  msgchunk_T *mp_last = NULL;
  msgchunk_T *mp;

  // If headless mode is enabled and no input is required, this variable
  // will be true. However If server mode is enabled, the message "--more--"
  // should be displayed.
  bool no_need_more = headless_mode && !embedded_mode && !ui_active();

  // We get called recursively when a timer callback outputs a message. In
  // that case don't show another prompt. Also when at the hit-Enter prompt
  // and nothing was typed.
  if (no_need_more || entered || (State == MODE_HITRETURN && typed_char == 0)) {
    return false;
  }
  entered = true;

  if (typed_char == 'G') {
    // "g<": Find first line on the last page.
    mp_last = msg_sb_start(last_msgchunk);
    for (int i = 0; i < Rows - 2 && mp_last != NULL
         && mp_last->sb_prev != NULL; i++) {
      mp_last = msg_sb_start(mp_last->sb_prev);
    }
  }

  State = MODE_ASKMORE;
  setmouse();
  if (typed_char == NUL) {
    msg_moremsg(false);
  }
  while (true) {
    // Get a typed character directly from the user.
    if (used_typed_char != NUL) {
      c = used_typed_char;              // was typed at hit-enter prompt
      used_typed_char = NUL;
    } else {
      c = get_keystroke(resize_events);
    }

    int toscroll = 0;
    switch (c) {
    case BS:                    // scroll one line back
    case K_BS:
    case 'k':
    case K_UP:
      toscroll = -1;
      break;

    case CAR:                   // one extra line
    case NL:
    case 'j':
    case K_DOWN:
      toscroll = 1;
      break;

    case 'u':                   // Up half a page
      toscroll = -(Rows / 2);
      break;

    case 'd':                   // Down half a page
      toscroll = Rows / 2;
      break;

    case 'b':                   // one page back
    case Ctrl_B:
    case K_PAGEUP:
      toscroll = -(Rows - 1);
      break;

    case ' ':                   // one extra page
    case 'f':
    case Ctrl_F:
    case K_PAGEDOWN:
    case K_LEFTMOUSE:
      toscroll = Rows - 1;
      break;

    case 'g':                   // all the way back to the start
      toscroll = -999999;
      break;

    case 'G':                   // all the way to the end
      toscroll = 999999;
      lines_left = 999999;
      break;

    case ':':                   // start new command line
      if (!confirm_msg_used) {
        // Since got_int is set all typeahead will be flushed, but we
        // want to keep this ':', remember that in a special way.
        typeahead_noflush(':');
        cmdline_row = Rows - 1;                 // put ':' on this line
        skip_redraw = true;                     // skip redraw once
        need_wait_return = false;               // don't wait in main()
      }
      FALLTHROUGH;
    case 'q':                   // quit
    case Ctrl_C:
    case ESC:
      if (confirm_msg_used) {
        // Jump to the choices of the dialog.
        retval = true;
      } else {
        got_int = true;
        quit_more = true;
      }
      // When there is some more output (wrapping line) display that
      // without another prompt.
      lines_left = Rows - 1;
      break;

    case K_EVENT:
      // only resize_events are processed here
      // Attempt to redraw the screen. sb_text doesn't support reflow
      // so this only really works for vertical resize.
      multiqueue_process_events(resize_events);
      to_redraw = true;
      break;

    default:                    // no valid response
      msg_moremsg(true);
      continue;
    }

    // code assumes we only do one at a time
    assert((toscroll == 0) || !to_redraw);

    if (toscroll != 0 || to_redraw) {
      if (toscroll < 0 || to_redraw) {
        // go to start of last line
        if (mp_last == NULL) {
          mp = msg_sb_start(last_msgchunk);
        } else if (mp_last->sb_prev != NULL) {
          mp = msg_sb_start(mp_last->sb_prev);
        } else {
          mp = NULL;
        }

        // go to start of line at top of the screen
        for (int i = 0; i < Rows - 2 && mp != NULL && mp->sb_prev != NULL; i++) {
          mp = msg_sb_start(mp->sb_prev);
        }

        if (mp != NULL && (mp->sb_prev != NULL || to_redraw)) {
          // Find line to be displayed at top
          for (int i = 0; i > toscroll; i--) {
            if (mp == NULL || mp->sb_prev == NULL) {
              break;
            }
            mp = msg_sb_start(mp->sb_prev);
            if (mp_last == NULL) {
              mp_last = msg_sb_start(last_msgchunk);
            } else {
              mp_last = msg_sb_start(mp_last->sb_prev);
            }
          }

          if (toscroll == -1 && !to_redraw) {
            grid_ins_lines(&msg_grid, 0, 1, Rows, 0, Columns);
            grid_clear(&msg_grid_adj, 0, 1, 0, Columns, HL_ATTR(HLF_MSG));
            // display line at top
            disp_sb_line(0, mp);
          } else {
            // redisplay all lines
            // TODO(bfredl): this case is not optimized (though only concerns
            // event fragmentation, not unnecessary scroll events).
            grid_clear(&msg_grid_adj, 0, Rows, 0, Columns, HL_ATTR(HLF_MSG));
            for (int i = 0; mp != NULL && i < Rows - 1; i++) {
              mp = disp_sb_line(i, mp);
              msg_scrolled++;
            }
            to_redraw = false;
          }
          toscroll = 0;
        }
      } else {
        // First display any text that we scrolled back.
        // if p_ch=0 we need to allocate a line for "press enter" messages!
        if (cmdline_row >= Rows && !ui_has(kUIMessages)) {
          msg_scroll_up(true, false);
          msg_scrolled++;
        }
        while (toscroll > 0 && mp_last != NULL) {
          if (msg_do_throttle() && !msg_grid.throttled) {
            // Tricky: we redraw at one line higher than usual. Therefore
            // the non-flushed area is one line larger.
            msg_scrolled_at_flush--;
            msg_grid_scroll_discount++;
          }
          // scroll up, display line at bottom
          msg_scroll_up(true, false);
          inc_msg_scrolled();
          grid_clear(&msg_grid_adj, Rows - 2, Rows - 1, 0, Columns, HL_ATTR(HLF_MSG));
          mp_last = disp_sb_line(Rows - 2, mp_last);
          toscroll--;
        }
      }

      if (toscroll <= 0) {
        // displayed the requested text, more prompt again
        grid_clear(&msg_grid_adj, Rows - 1, Rows, 0, Columns, HL_ATTR(HLF_MSG));
        msg_moremsg(false);
        continue;
      }

      // display more text, return to caller
      lines_left = toscroll;
    }

    break;
  }

  // clear the --more-- message
  grid_clear(&msg_grid_adj, Rows - 1, Rows, 0, Columns, HL_ATTR(HLF_MSG));
  redraw_cmdline = true;
  clear_cmdline = false;
  mode_displayed = false;

  State = oldState;
  setmouse();
  if (quit_more) {
    msg_row = Rows - 1;
    msg_col = 0;
  }

  entered = false;
  return retval;
}

// msg_moremsg() migrated to Rust: src/nvim-rs/message/src/misc.rs (rs_msg_moremsg)
// repeat_message() migrated to Rust: src/nvim-rs/message/src/misc.rs (rs_repeat_message)
// msg_ext_init_chunks(), msg_ext_emit_chunk(), msg_ext_ui_flush(), msg_ext_flush_showmode()
// migrated to Rust: src/nvim-rs/message/src/display.rs

// redir_write() migrated to Rust: src/nvim-rs/message/src/verbose.rs (rs_redir_write)

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

/// Used for "confirm()" function, and the :confirm command prefix.
/// Versions which haven't got flexible dialogs yet, and console
/// versions, get this generic handler which uses the command line.
///
/// type  = one of:
///         VIM_QUESTION, VIM_INFO, VIM_WARNING, VIM_ERROR or VIM_GENERIC
/// title = title string (can be NULL for default)
/// (neither used in console dialogs at the moment)
///
/// Format of the "buttons" string:
/// "Button1Name\nButton2Name\nButton3Name"
/// The first button should normally be the default/accept
/// The second button should be the 'Cancel' button
/// Other buttons- use your imagination!
/// A '&' in a button name becomes a shortcut, so each '&' should be before a
/// different letter.
///
/// @param textfiel  IObuff for inputdialog(), NULL otherwise
/// @param ex_cmd  when true pressing : accepts default and starts Ex command
/// @returns 0 if cancelled, otherwise the nth button (1-indexed).
int do_dialog(int type, const char *title, const char *message, const char *buttons, int dfltbutton,
              const char *textfield, int ex_cmd)
{
  int retval = 0;
  int i;

  if (silent_mode) {  // No dialogs in silent mode ("ex -s")
    return dfltbutton;  // return default option
  }

  int save_msg_silent = msg_silent;
  int oldState = State;

  msg_silent = 0;  // If dialog prompts for input, user needs to see it! #8788

  // Since we wait for a keypress, don't make the
  // user press RETURN as well afterwards.
  no_wait_return++;
  char *hotkeys = rs_msg_show_console_dialog(message, buttons, dfltbutton);

  while (true) {
    // Without a UI Nvim waits for input forever.
    if (!ui_active() && !input_available()) {
      retval = dfltbutton;
      break;
    }

    // Get a typed character directly from the user.
    int c = prompt_for_input(confirm_buttons, HLF_M, true, NULL);
    switch (c) {
    case CAR:                 // User accepts default option
    case NUL:
      retval = dfltbutton;
      break;
    case Ctrl_C:              // User aborts/cancels
    case ESC:
      retval = 0;
      break;
    default:                  // Could be a hotkey?
      if (c < 0) {            // special keys are ignored here
        msg_didout = msg_didany = false;
        continue;
      }
      if (c == ':' && ex_cmd) {
        retval = dfltbutton;
        ins_char_typebuf(':', 0, false);
        break;
      }

      // Make the character lowercase, as chars in "hotkeys" are.
      c = mb_tolower(c);
      retval = 1;
      for (i = 0; hotkeys[i]; i++) {
        if (utf_ptr2char(hotkeys + i) == c) {
          break;
        }
        i += utfc_ptr2len(hotkeys + i) - 1;
        retval++;
      }
      if (hotkeys[i]) {
        break;
      }
      // No hotkey match, so keep waiting
      msg_didout = msg_didany = false;
      continue;
    }
    break;
  }

  xfree(hotkeys);
  xfree(confirm_msg);
  confirm_msg = NULL;

  msg_silent = save_msg_silent;
  State = oldState;
  setmouse();
  no_wait_return--;
  msg_end_prompt();

  return retval;
}

// copy_char, console_dialog_alloc, msg_show_console_dialog, copy_confirm_hotkeys
// migrated to Rust (dialog.rs) as rs_msg_show_console_dialog
// display_confirm_msg, vim_dialog_yesno, vim_dialog_yesnocancel, vim_dialog_yesnoallcancel
// migrated to Rust (dialog.rs)
