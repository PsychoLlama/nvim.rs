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

// nvim_ui_ext_msg_set_pos() migrated to Rust (misc.rs) with #[export_name]

/// C accessor: curwin msgsep fillchar for Rust FFI.
uint32_t nvim_curwin_get_fcs_msgsep(void) { return (uint32_t)curwin->w_p_fcs_chars.msgsep; }

/// C wrapper for ui_call_msg_set_pos to avoid exposing String struct to Rust.
void nvim_ui_call_msg_set_pos_impl(int handle, int row, bool scrolled,
                                    const char *buf, size_t size, int zindex,
                                    int comp_index)
{
  ui_call_msg_set_pos(handle, row, scrolled,
                      (String){ .data = (char *)buf, .size = size },
                      zindex, comp_index);
}

/// C helper for schar_get used by Rust FFI.
size_t nvim_schar_get_impl(char *buf_out, uint32_t sc) { return schar_get(buf_out, (schar_T)sc); }

/// C accessor: check if EVENT_PROGRESS autocmd is registered.
int nvim_has_event_progress(void) { return has_event(EVENT_PROGRESS) ? 1 : 0; }

/// C helper for do_autocmd_progress autocmd dispatch (dict construction uses API macros).
void nvim_apply_autocmds_progress_c(Object msg_id, const char **text_ptrs,
                                     const size_t *text_sizes, int num_chunks,
                                     bool has_data, int64_t percent,
                                     const char *title_data, size_t title_size,
                                     const char *status_data, size_t status_size,
                                     const char *pat)
{
  MAXSIZE_TEMP_DICT(data, 7);
  ArrayOf(String) messages = ARRAY_DICT_INIT;
  for (int i = 0; i < num_chunks; i++) {
    ADD(messages, STRING_OBJ(((String){ .data = (char *)text_ptrs[i], .size = text_sizes[i] })));
  }
  PUT_C(data, "id", OBJECT_OBJ(msg_id));
  PUT_C(data, "text", ARRAY_OBJ(messages));
  if (has_data) {
    PUT_C(data, "percent", INTEGER_OBJ(percent));
    PUT_C(data, "status", STRING_OBJ(((String){ .data = (char *)status_data, .size = status_size })));
    PUT_C(data, "title", STRING_OBJ(((String){ .data = (char *)title_data, .size = title_size })));
  }
  apply_autocmds_group(EVENT_PROGRESS, pat ? pat : "", NULL, true,
                       AUGROUP_ALL, NULL, NULL, &DICT_OBJ(data));
  kv_destroy(messages);
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

// do_autocmd_progress() migrated to Rust (display.rs) with #[export_name]
// (uses nvim_apply_autocmds_progress_c helper for dict construction)

// msg_hist_add_multihl migrated to Rust: src/nvim-rs/message/src/history.rs
// ex_messages() migrated to Rust: src/nvim-rs/message/src/display.rs (rs_ex_messages)

// wait_return() migrated to Rust (wait.rs) with #[export_name = "wait_return"]
// hit_return_msg, msgmore, str2special_arena migrated to Rust (misc.rs/keys.rs)

// msg_prt_line() migrated to Rust (line.rs) with #[export_name = "msg_prt_line"]
// C accessors for lcs_chars_T: nvim_lcs_* functions (see below)

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

// ============================================================================
// lcs_chars_T accessors for msg_prt_line (migrated to Rust)
// These expose the curwin->w_p_lcs_chars and curbuf fields.
// ============================================================================

// nvim_curwin_w_p_list is already defined in edit.c
uint32_t nvim_lcs_eol(void) { return curwin->w_p_lcs_chars.eol; }
uint32_t nvim_lcs_trail(void) { return curwin->w_p_lcs_chars.trail; }
uint32_t nvim_lcs_lead(void) { return curwin->w_p_lcs_chars.lead; }
int nvim_lcs_has_leadmultispace(void) { return curwin->w_p_lcs_chars.leadmultispace != NULL; }
uint32_t nvim_lcs_leadmultispace_at(int idx) {
  if (curwin->w_p_lcs_chars.leadmultispace == NULL) { return 0; }
  return (uint32_t)curwin->w_p_lcs_chars.leadmultispace[idx];
}
uint32_t nvim_lcs_tab1(void) { return curwin->w_p_lcs_chars.tab1; }
uint32_t nvim_lcs_tab2(void) { return curwin->w_p_lcs_chars.tab2; }
uint32_t nvim_lcs_tab3(void) { return curwin->w_p_lcs_chars.tab3; }
uint32_t nvim_lcs_nbsp(void) { return curwin->w_p_lcs_chars.nbsp; }
uint32_t nvim_lcs_space(void) { return curwin->w_p_lcs_chars.space; }
int nvim_lcs_has_multispace(void) { return curwin->w_p_lcs_chars.multispace != NULL; }
uint32_t nvim_lcs_multispace_at(int idx) {
  if (curwin->w_p_lcs_chars.multispace == NULL) { return 0; }
  return (uint32_t)curwin->w_p_lcs_chars.multispace[idx];
}
int64_t nvim_curbuf_ts(void) { return curbuf->b_p_ts; }
int *nvim_curbuf_vts_array(void) { return curbuf->b_p_vts_array; }
uint32_t nvim_schar_from_ascii(int c) { return schar_from_ascii(c); }

/// HLF_AT highlight constant (for Rust FFI).
int nvim_hlf_at(void) { return HLF_AT; }

/// HLF_0 highlight constant (for Rust FFI).
int nvim_hlf_0(void) { return HLF_0; }
