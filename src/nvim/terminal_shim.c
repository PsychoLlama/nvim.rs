#include <assert.h>
#include <limits.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "klib/kvec.h"
#include "nvim/api/private/helpers.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/change.h"
#include "nvim/channel.h"
#include "nvim/channel_defs.h"
#include "nvim/cursor.h"
#include "nvim/cursor_shape.h"
#include "nvim/drawline.h"
#include "nvim/drawscreen.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
extern void rs_eval_call_provider(const char *provider, const char *method,
                                  list_T *arguments, bool discard, typval_T *out_rettv);
#include "nvim/event/defs.h"
#include "nvim/event/loop.h"
#include "nvim/event/multiqueue.h"
#include "nvim/event/time.h"
#include "nvim/ex_docmd.h"
#include "nvim/getchar.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/highlight.h"
#include "nvim/highlight_defs.h"
#include "nvim/highlight_group.h"
#include "nvim/keycodes.h"
#include "nvim/macros_defs.h"
#include "nvim/main.h"
#include "nvim/map_defs.h"
#include "nvim/mark.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/mouse.h"
#include "nvim/move.h"
#include "nvim/msgpack_rpc/channel_defs.h"
#include "nvim/normal_defs.h"
#include "nvim/ops.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/optionstr.h"
#include "nvim/pos_defs.h"
#include "nvim/state.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/terminal.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/vim_defs.h"
#include "nvim/vterm/parser.h"
#include "nvim/vterm/pen.h"
#include "nvim/vterm/screen.h"
#include "nvim/vterm/state.h"
#include "nvim/vterm/vterm.h"
#include "nvim/vterm/vterm_keycodes_defs.h"
#include "nvim/window.h"

typedef struct {
  VimState state;
  Terminal *term;
  int save_rd;          ///< saved value of RedrawingDisabled
  bool close;
  bool got_bsl;         ///< if the last input was <C-\>
  bool got_bsl_o;       ///< if left terminal mode with <c-\><c-o>
  bool cursor_visible;  ///< cursor's current visibility; ensures matched busy_start/stop UI events

  // These fields remember the prior values of window options before entering terminal mode.
  // Valid only when save_curwin_handle != 0.
  handle_T save_curwin_handle;
  bool save_w_p_cul;
  char *save_w_p_culopt;
  uint8_t save_w_p_culopt_flags;
  int save_w_p_cuc;
  OptInt save_w_p_so;
  OptInt save_w_p_siso;
} TerminalState;

#include "terminal_shim.c.generated.h"
extern int rs_win_valid(win_T *win);

// Rust implementation in nvim-event crate
extern MultiQueue *rs_loop_get_events(Loop *loop);
#define loop_get_events(l) rs_loop_get_events(l)

// Rust FFI declarations from nvim-vterm crate
extern void rs_vterm_keyboard_key(void *vt, int key, int mods);
extern void rs_vterm_keyboard_unichar(void *vt, unsigned int ch, int mods);
extern void rs_vterm_keyboard_start_paste(void *vt);
extern void rs_vterm_keyboard_end_paste(void *vt);
extern void rs_vterm_mouse_move(void *vt, int row, int col, int mods);
extern void rs_vterm_mouse_button(void *vt, int button, int pressed, int mods);

// Rust FFI declarations from nvim-terminal crate
extern int rs_terminal_is_filter_char_flags(int c, int flags);
extern int rs_terminal_row_to_linenr_term(void *term, int row);
extern int rs_terminal_linenr_to_row_term(void *term, int linenr);
extern void rs_terminal_focus_gain(void *term);
extern void rs_terminal_focus_lose(void *term);
extern int rs_terminal_underline_hl_flag(VTermScreenCellAttrs attrs);
extern int rs_terminal_parse_osc8(const char *str, int *attr);

// Result of rs_terminal_convert_key: VTerm key code and modifier mask.
typedef struct {
  int key;        ///< VTermKey code (VTERM_KEY_NONE if not a special key)
  int modifiers;  ///< VTermModifier flags
} VTermKeyResult;
extern VTermKeyResult rs_terminal_convert_key(int key, int nvim_mod_mask);

// Delay for refreshing the terminal buffer after receiving updates from
// libvterm. Improves performance when receiving large bursts of data.
#define REFRESH_DELAY 10

#define TEXTBUF_SIZE      0x1fff
#define SELECTIONBUF_SIZE 0x0400

static TimeWatcher refresh_timer;
static bool refresh_pending = false;

typedef struct {
  size_t cols;
  VTermScreenCell cells[];
} ScrollbackLine;

// NOTE: The authoritative definition of struct terminal is CTerminal in
// src/nvim-rs/terminal/src/lib.rs. This C definition must stay in sync with
// the Rust repr(C) struct. Layout is verified at startup by Rust
// size_of/offset_of assertions in the terminal crate.
struct terminal {
  TerminalOptions opts;  // options passed to terminal_open
  VTerm *vt;
  VTermScreen *vts;
  // buffer used to:
  //  - convert VTermScreen cell arrays into utf8 strings
  //  - receive data from libvterm as a result of key presses.
  char textbuf[TEXTBUF_SIZE];

  ScrollbackLine **sb_buffer;       // Scrollback storage.
  size_t sb_current;                // Lines stored in sb_buffer.
  size_t sb_size;                   // Capacity of sb_buffer.
  // "virtual index" that points to the first sb_buffer row that we need to
  // push to the terminal buffer when refreshing the scrollback. When negative,
  // it actually points to entries that are no longer in sb_buffer (because the
  // window height has increased) and must be deleted from the terminal buffer
  int sb_pending;
  size_t sb_deleted;                // Lines deleted from sb_buffer.
  size_t sb_deleted_last;           // Value of sb_deleted on last refresh_scrollback()

  char *title;     // VTermStringFragment buffer
  size_t title_len;
  size_t title_size;

  // buf_T instance that acts as a "drawing surface" for libvterm
  // we can't store a direct reference to the buffer because the
  // refresh_timer_cb may be called after the buffer was freed, and there's
  // no way to know if the memory was reused.
  handle_T buf_handle;
  // program exited
  bool closed;
  // when true, the terminal's destruction is already enqueued.
  bool destroy;

  // some vterm properties
  bool forward_mouse;
  int invalid_start, invalid_end;   // invalid rows in libvterm screen
  struct {
    int row, col;
    int shape;
    bool visible;  ///< Terminal wants to show cursor.
                   ///< `TerminalState.cursor_visible` indicates whether it is actually shown.
    bool blink;
  } cursor;

  struct {
    bool resize;          ///< pending width/height
    bool cursor;          ///< pending cursor shape or blink change
    StringBuilder *send;  ///< When there is a pending TermRequest autocommand, block and store input.
    MultiQueue *events;   ///< Events waiting for refresh.
  } pending;

  bool theme_updates;  ///< Send a theme update notification when 'bg' changes

  bool color_set[16];

  char *selection_buffer;  ///< libvterm selection buffer
  StringBuilder selection;  ///< Growable array containing full selection data

  StringBuilder termrequest_buffer;  ///< Growable array containing unfinished request sequence

  size_t refcount;                  // reference count
};

// Rust vterm callback implementations (Phase 1)
extern int rs_term_damage(VTermRect rect, void *data);
extern int rs_term_moverect(VTermRect dest, VTermRect src, void *data);
extern int rs_term_movecursor(VTermPos new_pos, VTermPos old_pos, int visible, void *data);
extern int rs_term_bell_cb(void *user);
extern int rs_term_theme_cb(bool *dark, void *user);
extern void rs_term_output_callback(const char *s, size_t len, void *user_data);
// Rust scrollback callback implementations (Phase 2)
extern int rs_term_sb_push(int cols, const VTermScreenCell *cells, void *data);
extern int rs_term_sb_pop(int cols, VTermScreenCell *cells, void *data);
// Rust key/theme implementations (Phase 3)
extern void rs_terminal_send_key_impl(void *term, int c);
extern void rs_terminal_notify_theme_impl(void *term, int dark);
// Rust refresh_size implementation (Phase 4)
extern void rs_terminal_refresh_size(void *term, void *buf);

static VTermScreenCallbacks vterm_screen_callbacks = {
  .damage = rs_term_damage,
  .moverect = rs_term_moverect,
  .movecursor = rs_term_movecursor,
  .settermprop = term_settermprop,
  .bell = rs_term_bell_cb,
  .theme = rs_term_theme_cb,
  .sb_pushline = rs_term_sb_push,  // Called before a line goes offscreen.
  .sb_popline = rs_term_sb_pop,
};

static VTermSelectionCallbacks vterm_selection_callbacks = {
  .set = term_selection_set,
  // For security reasons we don't support querying the system clipboard from the embedded terminal
  .query = NULL,
};

static Set(ptr_t) invalidated_terminals = SET_INIT;

static void emit_termrequest(void **argv)
{
  Terminal *term = argv[0];
  char *sequence = argv[1];
  size_t sequence_length = (size_t)argv[2];
  StringBuilder *pending_send = argv[3];
  int row = (int)(intptr_t)argv[4];
  int col = (int)(intptr_t)argv[5];
  size_t sb_deleted = (size_t)(intptr_t)argv[6];

  if (term->sb_pending > 0) {
    // Don't emit the event while there is pending scrollback because we need
    // the buffer contents to be fully updated. If this is the case, schedule
    // the event onto the pending queue where it will be executed after the
    // terminal is refreshed and the pending scrollback is cleared.
    multiqueue_put(term->pending.events, emit_termrequest, term, sequence, (void *)sequence_length,
                   pending_send, (void *)(intptr_t)row, (void *)(intptr_t)col,
                   (void *)(intptr_t)sb_deleted);
    return;
  }

  set_vim_var_string(VV_TERMREQUEST, sequence, (ptrdiff_t)sequence_length);

  MAXSIZE_TEMP_ARRAY(cursor, 2);
  ADD_C(cursor, INTEGER_OBJ(row - (int64_t)(term->sb_deleted - sb_deleted)));
  ADD_C(cursor, INTEGER_OBJ(col));

  MAXSIZE_TEMP_DICT(data, 2);
  String termrequest = { .data = sequence, .size = sequence_length };
  PUT_C(data, "sequence", STRING_OBJ(termrequest));
  PUT_C(data, "cursor", ARRAY_OBJ(cursor));

  buf_T *buf = handle_get_buffer(term->buf_handle);
  apply_autocmds_group(EVENT_TERMREQUEST, NULL, NULL, true, AUGROUP_ALL, buf, NULL,
                       &DICT_OBJ(data));
  xfree(sequence);

  StringBuilder *term_pending_send = term->pending.send;
  term->pending.send = NULL;
  if (kv_size(*pending_send)) {
    terminal_send(term, pending_send->items, pending_send->size);
    kv_destroy(*pending_send);
  }
  if (term_pending_send != pending_send) {
    term->pending.send = term_pending_send;
  }
  xfree(pending_send);
}

static void schedule_termrequest(Terminal *term)
{
  term->pending.send = xmalloc(sizeof(StringBuilder));
  kv_init(*term->pending.send);

  int line = rs_terminal_row_to_linenr_term(term, term->cursor.row);
  multiqueue_put(loop_get_events(&main_loop), emit_termrequest, term,
                 xmemdup(term->termrequest_buffer.items, term->termrequest_buffer.size),
                 (void *)(intptr_t)term->termrequest_buffer.size, term->pending.send,
                 (void *)(intptr_t)line, (void *)(intptr_t)term->cursor.col,
                 (void *)(intptr_t)term->sb_deleted);
}


extern int rs_on_osc(int command, const char *str, size_t len, int initial, int is_final,
                     void *user);
static int on_osc(int command, VTermStringFragment frag, void *user)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_on_osc(command, frag.str, frag.len, (int)frag.initial, (int)frag.final, user);
}

extern int rs_on_dcs(const char *command, size_t commandlen, const char *str, size_t len,
                     int initial, int is_final, void *user);
static int on_dcs(const char *command, size_t commandlen, VTermStringFragment frag, void *user)
{
  return rs_on_dcs(command, commandlen, frag.str, frag.len, (int)frag.initial,
                   (int)frag.final, user);
}

extern int rs_on_apc(const char *str, size_t len, int initial, int is_final, void *user);
static int on_apc(VTermStringFragment frag, void *user)
{
  return rs_on_apc(frag.str, frag.len, (int)frag.initial, (int)frag.final, user);
}

static VTermStateFallbacks vterm_fallbacks = {
  .control = NULL,
  .csi = NULL,
  .osc = on_osc,
  .dcs = on_dcs,
  .apc = on_apc,
  .pm = NULL,
  .sos = NULL,
};

void terminal_init(void)
{
  time_watcher_init(&main_loop, &refresh_timer, NULL);
  // refresh_timer_cb will redraw the screen which can call vimscript
  refresh_timer.events = multiqueue_new_child(loop_get_events(&main_loop));
}

void terminal_teardown(void)
{
  time_watcher_stop(&refresh_timer);
  multiqueue_free(refresh_timer.events);
  time_watcher_close(&refresh_timer, NULL);
  set_destroy(ptr_t, &invalidated_terminals);
  // terminal_destroy might be called after terminal_teardown is invoked
  // make sure it is in an empty, valid state
  invalidated_terminals = (Set(ptr_t)) SET_INIT;
}

// public API {{{

/// Initializes terminal properties, and triggers TermOpen.
///
/// The PTY process (TerminalOptions.data) was already started by jobstart(),
/// via ex_terminal() or the term:// BufReadCmd.
///
/// @param buf Buffer used for presentation of the terminal.
/// @param opts PTY process channel, various terminal properties and callbacks.
void terminal_open(Terminal **termpp, buf_T *buf, TerminalOptions opts)
  FUNC_ATTR_NONNULL_ALL
{
  // Create a new terminal instance and configure it
  Terminal *term = *termpp = xcalloc(1, sizeof(Terminal));
  term->opts = opts;

  // Associate the terminal instance with the new buffer
  term->buf_handle = buf->handle;
  buf->terminal = term;
  // Create VTerm
  term->vt = vterm_new(opts.height, opts.width);
  vterm_set_utf8(term->vt, 1);
  // Setup state
  VTermState *state = vterm_obtain_state(term->vt);
  // Set up screen
  term->vts = vterm_obtain_screen(term->vt);
  vterm_screen_enable_altscreen(term->vts, true);
  vterm_screen_enable_reflow(term->vts, true);
  // delete empty lines at the end of the buffer
  vterm_screen_set_callbacks(term->vts, &vterm_screen_callbacks, term);
  vterm_screen_set_unrecognised_fallbacks(term->vts, &vterm_fallbacks, term);
  vterm_screen_set_damage_merge(term->vts, VTERM_DAMAGE_SCROLL);
  vterm_screen_reset(term->vts, 1);
  vterm_output_set_callback(term->vt, rs_term_output_callback, term);

  term->selection_buffer = xcalloc(SELECTIONBUF_SIZE, 1);
  vterm_state_set_selection_callbacks(state, &vterm_selection_callbacks, term,
                                      term->selection_buffer, SELECTIONBUF_SIZE);

  VTermValue cursor_shape;
  switch (shape_table[SHAPE_IDX_TERM].shape) {
  case SHAPE_BLOCK:
    cursor_shape.number = VTERM_PROP_CURSORSHAPE_BLOCK;
    break;
  case SHAPE_HOR:
    cursor_shape.number = VTERM_PROP_CURSORSHAPE_UNDERLINE;
    break;
  case SHAPE_VER:
    cursor_shape.number = VTERM_PROP_CURSORSHAPE_BAR_LEFT;
    break;
  }
  vterm_state_set_termprop(state, VTERM_PROP_CURSORSHAPE, &cursor_shape);

  VTermValue cursor_blink;
  if (shape_table[SHAPE_IDX_TERM].blinkon != 0 && shape_table[SHAPE_IDX_TERM].blinkoff != 0) {
    cursor_blink.boolean = true;
  } else {
    cursor_blink.boolean = false;
  }
  vterm_state_set_termprop(state, VTERM_PROP_CURSORBLINK, &cursor_blink);

  // force a initial refresh of the screen to ensure the buffer will always
  // have as many lines as screen rows when refresh_scrollback is called
  term->invalid_start = 0;
  term->invalid_end = opts.height;

  // Create a separate queue for events which need to wait for a terminal
  // refresh. We cannot reschedule events back onto the main queue because this
  // can create an infinite loop (#32753).
  // This queue is never processed directly: when the terminal is refreshed, all
  // events from this queue are copied back onto the main event queue.
  term->pending.events = multiqueue_new(NULL, NULL);

  aco_save_T aco;
  aucmd_prepbuf(&aco, buf);

  refresh_screen(term, buf);
  set_option_value(kOptBuftype, STATIC_CSTR_AS_OPTVAL("terminal"), OPT_LOCAL);

  if (buf->b_ffname != NULL) {
    buf_set_term_title(buf, buf->b_ffname, strlen(buf->b_ffname));
  }
  RESET_BINDING(curwin);
  // Reset cursor in current window.
  curwin->w_cursor = (pos_T){ .lnum = 1, .col = 0, .coladd = 0 };
  // Initialize to check if the scrollback buffer has been allocated in a TermOpen autocmd.
  term->sb_buffer = NULL;
  // Apply TermOpen autocmds _before_ configuring the scrollback buffer.
  apply_autocmds(EVENT_TERMOPEN, NULL, NULL, false, buf);

  aucmd_restbuf(&aco);

  if (*termpp == NULL) {
    return;  // Terminal has already been destroyed.
  }

  if (term->sb_buffer == NULL) {
    // Local 'scrollback' _after_ autocmds.
    if (buf->b_p_scbk < 1) {
      buf->b_p_scbk = SB_MAX;
    }
    // Configure the scrollback buffer.
    term->sb_size = (size_t)buf->b_p_scbk;
    term->sb_buffer = xmalloc(sizeof(ScrollbackLine *) * term->sb_size);
  }

  // Configure the color palette. Try to get the color from:
  //
  // - b:terminal_color_{NUM}
  // - g:terminal_color_{NUM}
  // - the VTerm instance
  for (int i = 0; i < 16; i++) {
    char var[64];
    snprintf(var, sizeof(var), "terminal_color_%d", i);
    char *name = get_config_string(var);
    if (name) {
      int dummy;
      RgbValue color_val = name_to_color(name, &dummy);

      if (color_val != -1) {
        VTermColor color;
        vterm_color_rgb(&color,
                        (uint8_t)((color_val >> 16) & 0xFF),
                        (uint8_t)((color_val >> 8) & 0xFF),
                        (uint8_t)((color_val >> 0) & 0xFF));
        vterm_state_set_palette_color(state, i, &color);
        term->color_set[i] = true;
      }
    }
  }
}

/// Closes the Terminal buffer.
///
/// May call terminal_destroy, which sets caller storage to NULL.
void terminal_close(Terminal **termpp, int status)
  FUNC_ATTR_NONNULL_ALL
{
  Terminal *term = *termpp;
  if (term->destroy) {
    return;
  }

#ifdef EXITFREE
  if (entered_free_all_mem) {
    // If called from close_buffer() inside free_all_mem(), the main loop has
    // already been freed, so it is not safe to call the close callback here.
    terminal_destroy(termpp);
    return;
  }
#endif

  bool only_destroy = false;

  if (term->closed) {
    // If called from close_buffer() after the process has already exited, we
    // only need to call the close callback to clean up the terminal object.
    only_destroy = true;
  } else {
    term->forward_mouse = false;
    // flush any pending changes to the buffer
    if (!exiting) {
      block_autocmds();
      refresh_terminal(term);
      unblock_autocmds();
    }
    term->closed = true;
  }

  buf_T *buf = handle_get_buffer(term->buf_handle);

  if (status == -1 || exiting) {
    // If this was called by close_buffer() (status is -1), or if exiting, we
    // must inform the buffer the terminal no longer exists so that
    // close_buffer() won't call this again.
    // If inside Terminal mode event handling, setting buf_handle to 0 also
    // informs terminal_enter() to call the close callback before returning.
    term->buf_handle = 0;
    if (buf) {
      buf->terminal = NULL;
    }
    if (!term->refcount) {
      // Not inside Terminal mode event handling.
      // We should not wait for the user to press a key.
      term->destroy = true;
      term->opts.close_cb(term->opts.data);
    }
  } else if (!only_destroy) {
    // Associated channel has been closed and the editor is not exiting.
    // Do not call the close callback now. Wait for the user to press a key.
    char msg[sizeof("\r\n[Process exited ]") + NUMBUFLEN];
    if (((Channel *)term->opts.data)->streamtype == kChannelStreamInternal) {
      snprintf(msg, sizeof msg, "\r\n[Terminal closed]");
    } else {
      snprintf(msg, sizeof msg, "\r\n[Process exited %d]", status);
    }
    terminal_receive(term, msg, strlen(msg));
  }

  if (only_destroy) {
    return;
  }

  if (buf && !is_autocmd_blocked()) {
    save_v_event_T save_v_event;
    dict_T *dict = get_v_event(&save_v_event);
    tv_dict_add_nr(dict, S_LEN("status"), status);
    tv_dict_set_keys_readonly(dict);
    apply_autocmds(EVENT_TERMCLOSE, NULL, NULL, false, buf);
    restore_v_event(dict, &save_v_event);
  }
}

void terminal_check_size(Terminal *term)
{
  if (term->closed) {
    return;
  }

  int curwidth, curheight;
  vterm_get_size(term->vt, &curheight, &curwidth);
  uint16_t width = 0;
  uint16_t height = 0;

  // Check if there is a window that displays the terminal and find the maximum width and height.
  // Skip the autocommand window which isn't actually displayed.
  FOR_ALL_TAB_WINDOWS(tp, wp) {
    if (is_aucmd_win(wp)) {
      continue;
    }
    if (wp->w_buffer && wp->w_buffer->terminal == term) {
      const uint16_t win_width =
        (uint16_t)(MAX(0, wp->w_view_width - win_col_off(wp)));
      width = MAX(width, win_width);
      height = (uint16_t)MAX(height, wp->w_view_height);
    }
  }

  // if no window displays the terminal, or such all windows are zero-height,
  // don't resize the terminal.
  if ((curheight == height && curwidth == width) || height == 0 || width == 0) {
    return;
  }

  vterm_set_size(term->vt, height, width);
  vterm_screen_flush_damage(term->vts);
  term->pending.resize = true;
  invalidate_terminal(term, -1, -1);
}

static void set_terminal_winopts(TerminalState *const s)
  FUNC_ATTR_NONNULL_ALL
{
  assert(s->save_curwin_handle == 0);

  // Disable these options in terminal-mode. They are nonsense because cursor is
  // placed at end of buffer to "follow" output. #11072
  s->save_curwin_handle = curwin->handle;
  s->save_w_p_cul = curwin->w_p_cul;
  s->save_w_p_culopt = NULL;
  s->save_w_p_culopt_flags = curwin->w_p_culopt_flags;
  s->save_w_p_cuc = curwin->w_p_cuc;
  s->save_w_p_so = curwin->w_p_so;
  s->save_w_p_siso = curwin->w_p_siso;

  if (curwin->w_p_cul && curwin->w_p_culopt_flags & kOptCuloptFlagNumber) {
    if (!strequal(curwin->w_p_culopt, "number")) {
      s->save_w_p_culopt = curwin->w_p_culopt;
      curwin->w_p_culopt = xstrdup("number");
    }
    curwin->w_p_culopt_flags = kOptCuloptFlagNumber;
  } else {
    curwin->w_p_cul = false;
  }
  curwin->w_p_cuc = false;
  curwin->w_p_so = 0;
  curwin->w_p_siso = 0;

  if (curwin->w_p_cuc != s->save_w_p_cuc) {
    redraw_later(curwin, UPD_SOME_VALID);
  } else if (curwin->w_p_cul != s->save_w_p_cul
             || (curwin->w_p_cul && curwin->w_p_culopt_flags != s->save_w_p_culopt_flags)) {
    redraw_later(curwin, UPD_VALID);
  }
}

static void unset_terminal_winopts(TerminalState *const s)
  FUNC_ATTR_NONNULL_ALL
{
  assert(s->save_curwin_handle != 0);

  win_T *const wp = handle_get_window(s->save_curwin_handle);
  if (!wp) {
    free_string_option(s->save_w_p_culopt);
    s->save_curwin_handle = 0;
    return;
  }

  if (rs_win_valid(wp)) {  // No need to redraw if window not in curtab.
    if (s->save_w_p_cuc != wp->w_p_cuc) {
      redraw_later(wp, UPD_SOME_VALID);
    } else if (s->save_w_p_cul != wp->w_p_cul
               || (s->save_w_p_cul && s->save_w_p_culopt_flags != wp->w_p_culopt_flags)) {
      redraw_later(wp, UPD_VALID);
    }
  }

  wp->w_p_cul = s->save_w_p_cul;
  if (s->save_w_p_culopt) {
    free_string_option(wp->w_p_culopt);
    wp->w_p_culopt = s->save_w_p_culopt;
  }
  wp->w_p_culopt_flags = s->save_w_p_culopt_flags;
  wp->w_p_cuc = s->save_w_p_cuc;
  wp->w_p_so = s->save_w_p_so;
  wp->w_p_siso = s->save_w_p_siso;
  s->save_curwin_handle = 0;
}

/// Implements MODE_TERMINAL state. :help Terminal-mode
bool terminal_enter(void)
{
  buf_T *buf = curbuf;
  assert(buf->terminal);  // Should only be called when curbuf has a terminal.
  TerminalState s[1] = { 0 };
  s->term = buf->terminal;
  s->cursor_visible = true;  // Assume visible; may change via refresh_cursor later.
  stop_insert_mode = false;

  // Ensure the terminal is properly sized. Ideally window size management
  // code should always have resized the terminal already, but check here to
  // be sure.
  terminal_check_size(s->term);

  int save_state = State;
  s->save_rd = RedrawingDisabled;
  State = MODE_TERMINAL;
  mapped_ctrl_c |= MODE_TERMINAL;  // Always map CTRL-C to avoid interrupt.
  RedrawingDisabled = false;

  set_terminal_winopts(s);

  s->term->pending.cursor = true;  // Update the cursor shape table
  adjust_topline_cursor(s->term, buf, 0);  // scroll to end
  showmode();
  ui_cursor_shape();

  // Tell the terminal it has focus
  rs_terminal_focus_gain(s->term);
  // Don't fire TextChangedT from changes in Normal mode.
  curbuf->b_last_changedtick_i = buf_get_changedtick(curbuf);

  apply_autocmds(EVENT_TERMENTER, NULL, NULL, false, curbuf);
  may_trigger_modechanged();

  s->state.execute = terminal_execute;
  s->state.check = terminal_check;
  state_enter(&s->state);

  if (!s->got_bsl_o) {
    restart_edit = 0;
  }
  State = save_state;
  RedrawingDisabled = s->save_rd;
  if (!s->cursor_visible) {
    // If cursor was hidden, show it again. Do so right after restoring State.
    ui_busy_stop();
  }

  // Restore the terminal cursor to what is set in 'guicursor'
  (void)parse_shape_opt(SHAPE_CURSOR);

  unset_terminal_winopts(s);

  // Tell the terminal it lost focus
  rs_terminal_focus_lose(s->term);
  // Don't fire TextChanged from changes in terminal mode.
  curbuf->b_last_changedtick = buf_get_changedtick(curbuf);

  if (curbuf->terminal == s->term && !s->close) {
    terminal_check_cursor();
  }
  if (restart_edit) {
    showmode();
  } else {
    unshowmode(true);
  }
  ui_cursor_shape();

  // If we're to close the terminal, don't let TermLeave autocommands free it first!
  if (s->close) {
    s->term->refcount++;
  }
  apply_autocmds(EVENT_TERMLEAVE, NULL, NULL, false, curbuf);
  if (s->close) {
    s->term->refcount--;
    const handle_T buf_handle = s->term->buf_handle;  // Callback may free s->term.
    s->term->destroy = true;
    s->term->opts.close_cb(s->term->opts.data);
    if (buf_handle != 0) {
      do_buffer_ext(DOBUF_WIPE, DOBUF_FIRST, FORWARD, buf_handle, DOBUF_FORCEIT);
    }
  }

  return s->got_bsl_o;
}

static void terminal_check_cursor(void)
{
  Terminal *term = curbuf->terminal;
  curwin->w_cursor.lnum = MIN(curbuf->b_ml.ml_line_count,
                              rs_terminal_row_to_linenr_term(term, term->cursor.row));
  const linenr_T topline = MAX(curbuf->b_ml.ml_line_count - curwin->w_view_height + 1, 1);
  // Don't update topline if unchanged to avoid unnecessary redraws.
  if (topline != curwin->w_topline) {
    set_topline(curwin, topline);
  }
  // Nudge cursor when returning to normal-mode.
  int off = (State & MODE_TERMINAL && curbuf->terminal == term) ? 0 : (curwin->w_p_rl ? 1 : -1);
  coladvance(curwin, MAX(0, term->cursor.col + off));
}

static bool terminal_check_focus(TerminalState *const s)
  FUNC_ATTR_NONNULL_ALL
{
  if (curbuf->terminal == NULL) {
    return false;
  }

  if (s->save_curwin_handle != curwin->handle) {
    // Terminal window changed, update window options.
    unset_terminal_winopts(s);
    set_terminal_winopts(s);
  }
  if (s->term != curbuf->terminal) {
    // Active terminal buffer changed, flush terminal's cursor state to the UI.
    rs_terminal_focus_lose(s->term);

    s->term = curbuf->terminal;
    s->term->pending.cursor = true;
    invalidate_terminal(s->term, -1, -1);
    rs_terminal_focus_gain(s->term);
  }
  return true;
}

/// Function executed before each iteration of terminal mode.
///
/// @return:
///           1 if the iteration should continue normally
///           0 if the main loop must exit
static int terminal_check(VimState *state)
{
  TerminalState *const s = (TerminalState *)state;

  if (stop_insert_mode || !terminal_check_focus(s)) {
    return 0;
  }

  // Validate topline and cursor position for autocommands. Especially important for WinScrolled.
  terminal_check_cursor();
  validate_cursor(curwin);

  // Don't let autocommands free the terminal from under our fingers.
  s->term->refcount++;
  if (has_event(EVENT_TEXTCHANGEDT)
      && curbuf->b_last_changedtick_i != buf_get_changedtick(curbuf)) {
    apply_autocmds(EVENT_TEXTCHANGEDT, NULL, NULL, false, curbuf);
    curbuf->b_last_changedtick_i = buf_get_changedtick(curbuf);
  }
  may_trigger_win_scrolled_resized();
  s->term->refcount--;
  if (s->term->buf_handle == 0) {
    s->close = true;
    return 0;
  }

  // Autocommands above may have changed focus, scrolled, or moved the cursor.
  if (!terminal_check_focus(s)) {
    return 0;
  }
  terminal_check_cursor();
  validate_cursor(curwin);

  show_cursor_info_later(false);
  if (must_redraw) {
    update_screen();
  } else {
    redraw_statuslines();
    if (clear_cmdline || redraw_cmdline || redraw_mode) {
      showmode();  // clear cmdline and show mode
    }
  }

  setcursor();
  refresh_cursor(s->term, &s->cursor_visible);
  ui_flush();
  return 1;
}

/// Processes one char of terminal-mode input.
static int terminal_execute(VimState *state, int key)
{
  TerminalState *s = (TerminalState *)state;

  // Check for certain control keys like Ctrl-C and Ctrl-\. We still send the
  // unmerged key and modifiers to the terminal.
  int tmp_mod_mask = mod_mask;
  int mod_key = merge_modifiers(key, &tmp_mod_mask);

  switch (mod_key) {
  case K_LEFTMOUSE:
  case K_LEFTDRAG:
  case K_LEFTRELEASE:
  case K_MIDDLEMOUSE:
  case K_MIDDLEDRAG:
  case K_MIDDLERELEASE:
  case K_RIGHTMOUSE:
  case K_RIGHTDRAG:
  case K_RIGHTRELEASE:
  case K_X1MOUSE:
  case K_X1DRAG:
  case K_X1RELEASE:
  case K_X2MOUSE:
  case K_X2DRAG:
  case K_X2RELEASE:
  case K_MOUSEDOWN:
  case K_MOUSEUP:
  case K_MOUSELEFT:
  case K_MOUSERIGHT:
  case K_MOUSEMOVE:
    if (send_mouse_event(s->term, key)) {
      return 0;
    }
    break;

  case K_PASTE_START:
    paste_repeat(1);
    break;

  case K_EVENT:
    // We cannot let an event free the terminal yet. It is still needed.
    s->term->refcount++;
    state_handle_k_event();
    s->term->refcount--;
    if (s->term->buf_handle == 0) {
      s->close = true;
      return 0;
    }
    break;

  case K_COMMAND:
    do_cmdline(NULL, getcmdkeycmd, NULL, 0);
    break;

  case K_LUA:
    map_execute_lua(false, false);
    break;

  case Ctrl_N:
    if (s->got_bsl) {
      return 0;
    }
    FALLTHROUGH;

  case Ctrl_O:
    if (s->got_bsl) {
      s->got_bsl_o = true;
      restart_edit = 'I';
      return 0;
    }
    FALLTHROUGH;

  default:
    if (mod_key == Ctrl_C) {
      // terminal_enter() always sets `mapped_ctrl_c` to avoid `got_int`. 8eeda7169aa4
      // But `got_int` may be set elsewhere, e.g. by interrupt() or an autocommand,
      // so ensure that it is cleared.
      got_int = false;
    }
    if (mod_key == Ctrl_BSL && !s->got_bsl) {
      s->got_bsl = true;
      break;
    }
    if (s->term->closed) {
      s->close = true;
      return 0;
    }

    s->got_bsl = false;
    terminal_send_key(s->term, key);
  }

  return 1;
}

/// Frees the given Terminal structure and sets the caller storage to NULL (in the spirit of
/// XFREE_CLEAR).
void terminal_destroy(Terminal **termpp)
  FUNC_ATTR_NONNULL_ALL
{
  Terminal *term = *termpp;
  buf_T *buf = handle_get_buffer(term->buf_handle);
  if (buf) {
    term->buf_handle = 0;
    buf->terminal = NULL;
  }

  if (!term->refcount) {
    if (set_has(ptr_t, &invalidated_terminals, term)) {
      // flush any pending changes to the buffer
      block_autocmds();
      refresh_terminal(term);
      unblock_autocmds();
      set_del(ptr_t, &invalidated_terminals, term);
    }
    for (size_t i = 0; i < term->sb_current; i++) {
      xfree(term->sb_buffer[i]);
    }
    xfree(term->sb_buffer);
    xfree(term->title);
    xfree(term->selection_buffer);
    kv_destroy(term->selection);
    kv_destroy(term->termrequest_buffer);
    vterm_free(term->vt);
    multiqueue_free(term->pending.events);
    xfree(term);
    *termpp = NULL;  // coverity[dead-store]
  }
}

extern void rs_terminal_do_send(void *term, const char *data, size_t size);
static void terminal_send(Terminal *term, const char *data, size_t size)
{
  rs_terminal_do_send(term, data, size);
}

void terminal_paste(int count, String *y_array, size_t y_size)
{
  if (y_size == 0) {
    return;
  }
  rs_vterm_keyboard_start_paste(curbuf->terminal->vt);
  size_t buff_len = y_array[0].size;
  char *buff = xmalloc(buff_len);
  for (int i = 0; i < count; i++) {
    // feed the lines to the terminal
    for (size_t j = 0; j < y_size; j++) {
      if (j) {
        // terminate the previous line
#ifdef MSWIN
        terminal_send(curbuf->terminal, "\r\n", 2);
#else
        terminal_send(curbuf->terminal, "\n", 1);
#endif
      }
      size_t len = y_array[j].size;
      if (len > buff_len) {
        buff = xrealloc(buff, len);
        buff_len = len;
      }
      char *dst = buff;
      char *src = y_array[j].data;
      while (*src != NUL) {
        len = (size_t)utf_ptr2len(src);
        int c = utf_ptr2char(src);
        if (!rs_terminal_is_filter_char_flags(c, (int)tpf_flags)) {
          memcpy(dst, src, len);
          dst += len;
        }
        src += len;
      }
      terminal_send(curbuf->terminal, buff, (size_t)(dst - buff));
    }
  }
  xfree(buff);
  rs_vterm_keyboard_end_paste(curbuf->terminal->vt);
}

static void terminal_send_key(Terminal *term, int c)
{
  rs_terminal_send_key_impl(term, c);
}

extern void rs_terminal_receive_impl(void *term, const char *data, size_t len);
void terminal_receive(Terminal *term, const char *data, size_t len)
{
  rs_terminal_receive_impl(term, data, len);
}

static int get_rgb(VTermState *state, VTermColor color)
{
  vterm_state_convert_color_to_rgb(state, &color);
  return RGB_(color.rgb.red, color.rgb.green, color.rgb.blue);
}


void terminal_get_line_attributes(Terminal *term, win_T *wp, int linenr, int *term_attrs)
{
  int height, width;
  vterm_get_size(term->vt, &height, &width);
  VTermState *state = vterm_obtain_state(term->vt);
  assert(linenr);
  int row = rs_terminal_linenr_to_row_term(term, linenr);
  if (row >= height) {
    // Terminal height was decreased but the change wasn't reflected into the
    // buffer yet
    return;
  }

  width = MIN(TERM_ATTRS_MAX, width);
  for (int col = 0; col < width; col++) {
    VTermScreenCell cell;
    bool color_valid = fetch_cell(term, row, col, &cell);
    bool fg_default = !color_valid || VTERM_COLOR_IS_DEFAULT_FG(&cell.fg);
    bool bg_default = !color_valid || VTERM_COLOR_IS_DEFAULT_BG(&cell.bg);

    // Get the rgb value set by libvterm.
    int vt_fg = fg_default ? -1 : get_rgb(state, cell.fg);
    int vt_bg = bg_default ? -1 : get_rgb(state, cell.bg);

    bool fg_indexed = VTERM_COLOR_IS_INDEXED(&cell.fg);
    bool bg_indexed = VTERM_COLOR_IS_INDEXED(&cell.bg);

    int16_t vt_fg_idx = ((!fg_default && fg_indexed) ? cell.fg.indexed.idx + 1 : 0);
    int16_t vt_bg_idx = ((!bg_default && bg_indexed) ? cell.bg.indexed.idx + 1 : 0);

    bool fg_set = vt_fg_idx && vt_fg_idx <= 16 && term->color_set[vt_fg_idx - 1];
    bool bg_set = vt_bg_idx && vt_bg_idx <= 16 && term->color_set[vt_bg_idx - 1];

    int hl_attrs = (cell.attrs.bold ? HL_BOLD : 0)
                   | (cell.attrs.italic ? HL_ITALIC : 0)
                   | (cell.attrs.reverse ? HL_INVERSE : 0)
                   | rs_terminal_underline_hl_flag(cell.attrs)
                   | (cell.attrs.strike ? HL_STRIKETHROUGH : 0)
                   | ((fg_indexed && !fg_set) ? HL_FG_INDEXED : 0)
                   | ((bg_indexed && !bg_set) ? HL_BG_INDEXED : 0);

    int attr_id = 0;

    if (hl_attrs || !fg_default || !bg_default) {
      attr_id = hl_get_term_attr(&(HlAttrs) {
        .cterm_ae_attr = (int16_t)hl_attrs,
        .cterm_fg_color = vt_fg_idx,
        .cterm_bg_color = vt_bg_idx,
        .rgb_ae_attr = (int16_t)hl_attrs,
        .rgb_fg_color = vt_fg,
        .rgb_bg_color = vt_bg,
        .rgb_sp_color = -1,
        .hl_blend = -1,
        .url = -1,
      });
    }

    if (cell.uri > 0) {
      attr_id = hl_combine_attr(attr_id, cell.uri);
    }

    term_attrs[col] = attr_id;
  }
}

void terminal_notify_theme(Terminal *term, bool dark)
  FUNC_ATTR_NONNULL_ALL
{
  rs_terminal_notify_theme_impl(term, (int)dark);
}

// }}}
// libvterm callbacks {{{

static void buf_set_term_title(buf_T *buf, const char *title, size_t len)
  FUNC_ATTR_NONNULL_ALL
{
  Error err = ERROR_INIT;
  dict_set_var(buf->b_vars,
               STATIC_CSTR_AS_STRING("term_title"),
               STRING_OBJ(((String){ .data = (char *)title, .size = len })),
               false,
               false,
               NULL,
               &err);
  api_clear_error(&err);
  status_redraw_buf(buf);
}

extern int rs_term_settermprop(VTermProp prop, VTermValue *val, void *data);
static int term_settermprop(VTermProp prop, VTermValue *val, void *data)
{
  return rs_term_settermprop(prop, val, data);
}


static void term_clipboard_set(void **argv)
{
  VTermSelectionMask mask = (VTermSelectionMask)(long)argv[0];
  char *data = argv[1];

  char regname;
  switch (mask) {
  case VTERM_SELECTION_CLIPBOARD:
    regname = '+';
    break;
  case VTERM_SELECTION_PRIMARY:
    regname = '*';
    break;
  default:
    regname = '+';
    break;
  }

  list_T *lines = tv_list_alloc(1);
  tv_list_append_allocated_string(lines, data);

  list_T *args = tv_list_alloc(3);
  tv_list_append_list(args, lines);

  const char regtype = 'v';
  tv_list_append_string(args, &regtype, 1);

  tv_list_append_string(args, &regname, 1);
  typval_T rettv;
  rs_eval_call_provider("clipboard", "set", args, true, &rettv);
}

static int term_selection_set(VTermSelectionMask mask, VTermStringFragment frag, void *user)
{
  Terminal *term = user;
  if (frag.initial) {
    kv_size(term->selection) = 0;
  }

  kv_concat_len(term->selection, frag.str, frag.len);

  if (frag.final) {
    char *data = xmemdupz(term->selection.items, kv_size(term->selection));
    multiqueue_put(loop_get_events(&main_loop), term_clipboard_set, (void *)mask, data);
  }

  return 1;
}

// }}}
// input handling {{{

// process a mouse event while the terminal is focused. return true if the
// terminal should lose focus
static bool send_mouse_event(Terminal *term, int c)
{
  int row = mouse_row;
  int col = mouse_col;
  int grid = mouse_grid;
  win_T *mouse_win = mouse_find_win_inner(&grid, &row, &col);
  if (mouse_win == NULL) {
    goto end;
  }

  int offset;
  if (term->forward_mouse && mouse_win->w_buffer->terminal == term && row >= 0
      && (grid > 1 || row + mouse_win->w_winbar_height < mouse_win->w_height)
      && col >= (offset = win_col_off(mouse_win))
      && (grid > 1 || col < mouse_win->w_width)) {
    // event in the terminal window and mouse events was enabled by the
    // program. translate and forward the event
    int button;
    bool pressed = false;

    switch (c) {
    case K_LEFTDRAG:
    case K_LEFTMOUSE:
      pressed = true; FALLTHROUGH;
    case K_LEFTRELEASE:
      button = 1; break;
    case K_MIDDLEDRAG:
    case K_MIDDLEMOUSE:
      pressed = true; FALLTHROUGH;
    case K_MIDDLERELEASE:
      button = 2; break;
    case K_RIGHTDRAG:
    case K_RIGHTMOUSE:
      pressed = true; FALLTHROUGH;
    case K_RIGHTRELEASE:
      button = 3; break;
    case K_X1DRAG:
    case K_X1MOUSE:
      pressed = true; FALLTHROUGH;
    case K_X1RELEASE:
      button = 8; break;
    case K_X2DRAG:
    case K_X2MOUSE:
      pressed = true; FALLTHROUGH;
    case K_X2RELEASE:
      button = 9; break;
    case K_MOUSEDOWN:
      pressed = true; button = 4; break;
    case K_MOUSEUP:
      pressed = true; button = 5; break;
    case K_MOUSELEFT:
      pressed = true; button = 7; break;
    case K_MOUSERIGHT:
      pressed = true; button = 6; break;
    case K_MOUSEMOVE:
      button = 0; break;
    default:
      return false;
    }

    VTermKeyResult mouse_result = rs_terminal_convert_key(c, mod_mask);
    VTermModifier mouse_mod = (VTermModifier)mouse_result.modifiers;
    rs_vterm_mouse_move(term->vt, row, col - offset, mouse_mod);
    if (button) {
      rs_vterm_mouse_button(term->vt, button, (int)pressed, mouse_mod);
    }
    return false;
  }

  if (c == K_MOUSEUP || c == K_MOUSEDOWN || c == K_MOUSELEFT || c == K_MOUSERIGHT) {
    win_T *save_curwin = curwin;
    // switch window/buffer to perform the scroll
    curwin = mouse_win;
    curbuf = curwin->w_buffer;

    cmdarg_T cap;
    oparg_T oa;
    CLEAR_FIELD(cap);
    clear_oparg(&oa);
    cap.oap = &oa;

    switch (cap.cmdchar = c) {
    case K_MOUSEUP:
      cap.arg = MSCR_UP;
      break;
    case K_MOUSEDOWN:
      cap.arg = MSCR_DOWN;
      break;
    case K_MOUSELEFT:
      cap.arg = MSCR_LEFT;
      break;
    case K_MOUSERIGHT:
      cap.arg = MSCR_RIGHT;
      break;
    default:
      abort();
    }

    // Call the common mouse scroll function shared with other modes.
    do_mousescroll(&cap);

    curwin->w_redr_status = true;
    curwin = save_curwin;
    curbuf = curwin->w_buffer;
    redraw_later(mouse_win, UPD_NOT_VALID);
    invalidate_terminal(term, -1, -1);
    // Only need to exit focus if the scrolled window is the terminal window
    return mouse_win == curwin;
  }

end:
  // Ignore left release action if it was not forwarded to prevent
  // leaving Terminal mode after entering to it using a mouse.
  if ((c == K_LEFTRELEASE && mouse_win != NULL && mouse_win->w_buffer->terminal == term)
      || c == K_MOUSEMOVE) {
    return false;
  }

  int len = ins_char_typebuf(vgetc_char, vgetc_mod_mask, true);
  if (KeyTyped) {
    ungetchars(len);
  }
  return true;
}

// }}}
// terminal buffer refresh & misc {{{

static void fetch_row(Terminal *term, int row, int end_col)
{
  int col = 0;
  size_t line_len = 0;
  char *ptr = term->textbuf;

  while (col < end_col) {
    VTermScreenCell cell;
    fetch_cell(term, row, col, &cell);
    if (cell.schar) {
      schar_get_adv(&ptr, cell.schar);
      line_len = (size_t)(ptr - term->textbuf);
    } else {
      *ptr++ = ' ';
    }
    col += cell.width;
  }

  // end of line
  term->textbuf[line_len] = NUL;
}

static bool fetch_cell(Terminal *term, int row, int col, VTermScreenCell *cell)
{
  if (row < 0) {
    ScrollbackLine *sbrow = term->sb_buffer[-row - 1];
    if ((size_t)col < sbrow->cols) {
      *cell = sbrow->cells[col];
    } else {
      // fill the pointer with an empty cell
      *cell = (VTermScreenCell) {
        .schar = 0,
        .width = 1,
      };
      return false;
    }
  } else {
    vterm_screen_get_cell(term->vts, (VTermPos){ .row = row, .col = col },
                          cell);
  }
  return true;
}

extern void rs_invalidate_terminal(void *term, int start_row, int end_row);
// queue a terminal instance for refresh
static void invalidate_terminal(Terminal *term, int start_row, int end_row)
{
  rs_invalidate_terminal(term, start_row, end_row);
}

extern void rs_refresh_terminal(void *term);
static void refresh_terminal(Terminal *term)
{
  rs_refresh_terminal(term);
}

extern void rs_refresh_cursor(void *term, bool *cursor_visible);
static void refresh_cursor(Terminal *term, bool *cursor_visible)
  FUNC_ATTR_NONNULL_ALL
{
  rs_refresh_cursor(term, cursor_visible);
}

extern void rs_refresh_timer_cb(void *watcher, void *data);
/// Calls refresh_terminal() on all invalidated_terminals.
static void refresh_timer_cb(TimeWatcher *watcher, void *data)
{
  rs_refresh_timer_cb(watcher, data);
}

extern void rs_on_scrollback_option_changed(void *term);
void on_scrollback_option_changed(Terminal *term)
{
  rs_on_scrollback_option_changed(term);
}

// adjust_scrollback is now implemented in Rust as rs_adjust_scrollback

// refresh_scrollback is now implemented in Rust as rs_refresh_scrollback

// refresh_screen is now implemented in Rust as rs_refresh_screen
extern void rs_refresh_screen_pub(Terminal *term, buf_T *buf);
static void refresh_screen(Terminal *term, buf_T *buf)
{
  rs_refresh_screen_pub(term, buf);
}

static void adjust_topline_cursor(Terminal *term, buf_T *buf, int added)
{
  linenr_T ml_end = buf->b_ml.ml_line_count;

  FOR_ALL_TAB_WINDOWS(tp, wp) {
    if (wp->w_buffer == buf) {
      if (wp == curwin && (State & MODE_TERMINAL && curbuf->terminal == term)) {
        // Move window cursor to terminal cursor's position and "follow" output.
        terminal_check_cursor();
        continue;
      }

      bool following = ml_end == wp->w_cursor.lnum + added;  // cursor at end?
      if (following) {
        // "Follow" the terminal output
        wp->w_cursor.lnum = ml_end;
        set_topline(wp, MAX(wp->w_cursor.lnum - wp->w_view_height + 1, 1));
      } else {
        // Ensure valid cursor for each window displaying this terminal.
        wp->w_cursor.lnum = MIN(wp->w_cursor.lnum, ml_end);
      }
      mb_check_adjust_col(wp);
    }
  }

  if (ml_end == buf->b_last_cursor.mark.lnum + added) {
    buf->b_last_cursor.mark.lnum = ml_end;
  }

  for (size_t i = 0; i < kv_size(buf->b_wininfo); i++) {
    WinInfo *wip = kv_A(buf->b_wininfo, i);
    if (ml_end == wip->wi_mark.mark.lnum + added) {
      wip->wi_mark.mark.lnum = ml_end;
    }
  }
}

static char *get_config_string(char *key)
{
  Error err = ERROR_INIT;
  // Only called from terminal_open where curbuf->terminal is the context.
  Object obj = dict_get_value(curbuf->b_vars, cstr_as_string(key), NULL, &err);
  api_clear_error(&err);
  if (obj.type == kObjectTypeNil) {
    obj = dict_get_value(get_globvar_dict(), cstr_as_string(key), NULL, &err);
    api_clear_error(&err);
  }
  if (obj.type == kObjectTypeString) {
    return obj.data.string.data;
  }
  api_free_object(obj);
  return NULL;
}

// }}}

void nvim_vim_beep_term(void) { vim_beep(kOptBoFlagTerm); }
char nvim_get_bg_char(void) { return *p_bg; }

// C accessor functions for Rust callbacks (Phase 1)

/// Accessor: queue a terminal for refresh (wraps `invalidate_terminal`).
/// Called from Rust vterm callbacks.
void nvim_terminal_invalidate(void *term, int start_row, int end_row)
{
  invalidate_terminal((Terminal *)term, start_row, end_row);
}

/// Accessor: send data to a terminal process (wraps `terminal_send`).
/// Called from Rust output callback.
void nvim_terminal_send(void *term, const char *data, size_t size)
{
  terminal_send((Terminal *)term, data, size);
}

/// Accessor: add terminal to invalidated set (without starting timer).
/// Used by Rust scrollback callbacks.
void nvim_terminal_set_put(void *term)
{
  set_put(ptr_t, &invalidated_terminals, (Terminal *)term);
}

/// Accessor: concat data into a StringBuilder (wraps `kv_concat_len`).
/// `sb` is a `StringBuilder *` pointer.
void nvim_term_sb_concat_len(void *sb, const char *data, size_t len)
{
  kv_concat_len(*(StringBuilder *)sb, data, len);
}

/// Accessor: get size of a StringBuilder (wraps `kv_size`).
/// `sb` is a `StringBuilder *` pointer.
size_t nvim_term_sb_size(const void *sb)
{
  return kv_size(*(const StringBuilder *)sb);
}

/// Accessor: get items pointer from a StringBuilder.
/// `sb` is a `StringBuilder *` pointer.
char *nvim_term_sb_items(void *sb)
{
  return ((StringBuilder *)sb)->items;
}

/// Accessor: reset size of a StringBuilder to 0 (wraps `kv_size(v) = 0`).
/// `sb` is a `StringBuilder *` pointer.
void nvim_term_sb_reset(void *sb)
{
  ((StringBuilder *)sb)->size = 0;
}

/// Accessor: push a char onto a StringBuilder (wraps `kv_push`).
/// `sb` is a `StringBuilder *` pointer.
void nvim_term_sb_push_char(void *sb, char c)
{
  kv_push(*(StringBuilder *)sb, c);
}

/// Return the size of a ScrollbackLine with `cols` cells.
/// Used by Rust to allocate scrollback rows.
size_t nvim_scrollback_line_size(size_t cols)
{
  return sizeof(ScrollbackLine) + cols * sizeof(VTermScreenCell);
}

/// Get cols from a ScrollbackLine (the first field).
size_t nvim_scrollback_line_cols(const void *sbrow)
{
  return ((const ScrollbackLine *)sbrow)->cols;
}

/// Get the cells pointer from a ScrollbackLine.
const void *nvim_scrollback_line_cells(const void *sbrow)
{
  return ((const ScrollbackLine *)sbrow)->cells;
}

/// Get mutable cells pointer from a ScrollbackLine.
void *nvim_scrollback_line_cells_mut(void *sbrow)
{
  return ((ScrollbackLine *)sbrow)->cells;
}

/// Return the size of a single VTermScreenCell (for Rust allocations).
size_t nvim_vterm_screen_cell_size(void)
{
  return sizeof(VTermScreenCell);
}

/// Zero-fill a single VTermScreenCell at the given pointer.
/// Used by term_sb_pop to fill cells beyond the scrollback row width.
void nvim_vterm_cell_zero(void *cell_ptr)
{
  VTermScreenCell *c = (VTermScreenCell *)cell_ptr;
  c->schar = 0;
  c->width = 1;
}

// Phase 5 accessor functions

/// Start the refresh timer (wraps time_watcher_start for refresh_timer_cb).
void nvim_terminal_timer_start(void)
{
  time_watcher_start(&refresh_timer, refresh_timer_cb, REFRESH_DELAY, 0);
}

/// Get the current value of refresh_pending.
int nvim_terminal_get_refresh_pending(void)
{
  return (int)refresh_pending;
}

/// Set refresh_pending to v (0 or 1).
void nvim_terminal_set_refresh_pending(int v)
{
  refresh_pending = (bool)v;
}

/// Get a buf_T* from a buffer handle (wraps handle_get_buffer macro).
void *nvim_terminal_handle_get_buffer(int buf_handle)
{
  return handle_get_buffer(buf_handle);
}

/// Set terminal title on buffer b: variable (wraps buf_set_term_title).
void nvim_terminal_buf_set_title(void *buf, const char *title, size_t len)
{
  buf_set_term_title((buf_T *)buf, title, len);
}

/// Realloc wrapper for Rust terminal title buffer.
void *nvim_term_xrealloc(void *ptr, size_t size)
{
  return xrealloc(ptr, size);
}

/// Call write_cb on a terminal (sends data to PTY).
void nvim_terminal_write_cb(void *term, const char *data, size_t size)
{
  Terminal *t = (Terminal *)term;
  t->opts.write_cb(data, size, t->opts.data);
}

/// Get pending.send StringBuilder pointer (NULL if not pending).
void *nvim_terminal_get_pending_send(void *term)
{
  return ((Terminal *)term)->pending.send;
}

// VTermValue accessors for term_settermprop
int nvim_vterm_value_boolean(const void *val) { return ((const VTermValue *)val)->boolean; }
int nvim_vterm_value_number(const void *val) { return ((const VTermValue *)val)->number; }
const char *nvim_vterm_frag_str(const void *val) { return ((const VTermValue *)val)->string.str; }
size_t nvim_vterm_frag_len(const void *val) { return ((const VTermValue *)val)->string.len; }
int nvim_vterm_frag_initial(const void *val) { return (int)((const VTermValue *)val)->string.initial; }
int nvim_vterm_frag_final(const void *val) { return (int)((const VTermValue *)val)->string.final; }

// Phase 7: Buffer manipulation accessors for refresh pipeline
int nvim_term_buf_line_count(const void *buf) { return ((const buf_T *)buf)->b_ml.ml_line_count; }
int64_t nvim_buf_get_scrollback(const void *buf) { return ((const buf_T *)buf)->b_p_scbk; }
void nvim_buf_set_scrollback(void *buf, int64_t val) { ((buf_T *)buf)->b_p_scbk = val; }
int nvim_rs_buf_valid(void *buf) { return rs_buf_valid((buf_T *)buf); }
void *nvim_terminal_get_buffer(int buf_handle)
{
  return handle_get_buffer(buf_handle);
}

// ml_append_buf/ml_replace_buf/ml_delete_buf wrappers
int nvim_ml_append_buf_term(void *buf, int lnum, char *line, bool newfile)
{
  return ml_append_buf((buf_T *)buf, (linenr_T)lnum, line, 0, newfile);
}
int nvim_ml_replace_buf_term(void *buf, int lnum, char *line, bool copy)
{
  return ml_replace_buf((buf_T *)buf, (linenr_T)lnum, line, copy, false);
}
int nvim_ml_delete_buf_term(void *buf, int lnum)
{
  return ml_delete_buf((buf_T *)buf, (linenr_T)lnum, false);
}
void nvim_mark_adjust_buf_term(void *buf, int line1, int line2, int amount, int amount_after)
{
  mark_adjust_buf((buf_T *)buf, (linenr_T)line1, (linenr_T)line2, (linenr_T)amount,
                  (linenr_T)amount_after, true, kMarkAdjustTerm, kExtmarkUndo);
}
void nvim_appended_lines_buf_term(void *buf, int lnum, int count)
{
  appended_lines_buf((buf_T *)buf, (linenr_T)lnum, (linenr_T)count);
}
void nvim_deleted_lines_buf_term(void *buf, int lnum, int count)
{
  deleted_lines_buf((buf_T *)buf, (linenr_T)lnum, (linenr_T)count);
}
void nvim_changed_lines_term(void *buf, int first, int last, int added)
{
  changed_lines((buf_T *)buf, (linenr_T)first, 0, (linenr_T)last, (linenr_T)added, true);
}
void nvim_multiqueue_move_events_term(void *term)
{
  Terminal *t = (Terminal *)term;
  multiqueue_move_events(loop_get_events(&main_loop), t->pending.events);
}
// Get term->sb_buffer[idx] (the idx-th ScrollbackLine *)
void *nvim_terminal_sb_get(void *term, size_t idx)
{
  return ((Terminal *)term)->sb_buffer[idx];
}
// Set term->sb_buffer[idx]
void nvim_terminal_sb_set(void *term, size_t idx, void *sbrow)
{
  ((Terminal *)term)->sb_buffer[idx] = (ScrollbackLine *)sbrow;
}
// Resize sb_buffer array
void nvim_terminal_sb_buffer_resize(void *term, size_t new_size)
{
  Terminal *t = (Terminal *)term;
  t->sb_buffer = xrealloc(t->sb_buffer, sizeof(ScrollbackLine *) * new_size);
  t->sb_size = new_size;
}

// fetch_row accessor (static function, needed by Rust refresh functions)
void nvim_fetch_row(void *term, int row, int end_col)
{
  fetch_row((Terminal *)term, row, end_col);
}

// refresh_cursor accessors
int nvim_terminal_is_active(void *term)
{
  return (State & MODE_TERMINAL) && curbuf->terminal == (Terminal *)term;
}
void nvim_ui_busy_start(void) { ui_busy_start(); }
void nvim_ui_busy_stop(void) { ui_busy_stop(); }
void nvim_term_ui_mode_info_set(void) { ui_mode_info_set(); }
void nvim_shape_table_set_cursor(int blink, int shape, int percentage)
{
  if (blink) {
    shape_table[SHAPE_IDX_TERM].blinkon = 500;
    shape_table[SHAPE_IDX_TERM].blinkoff = 500;
  } else {
    shape_table[SHAPE_IDX_TERM].blinkon = 0;
    shape_table[SHAPE_IDX_TERM].blinkoff = 0;
  }
  shape_table[SHAPE_IDX_TERM].shape = shape;
  shape_table[SHAPE_IDX_TERM].percentage = percentage;
}

// refresh_timer_cb accessors
void nvim_terminal_foreach_invalidated(void (*fn)(void *term, void *ctx), void *ctx)
{
  Terminal *term;
  void *stub; (void)(stub);
  block_autocmds();
  set_foreach(&invalidated_terminals, term, {
    fn(term, ctx);
  });
  set_clear(ptr_t, &invalidated_terminals);
  unblock_autocmds();
}
int nvim_is_exiting(void) { return exiting; }

// C wrapper for adjust_topline_cursor (uses FOR_ALL_TAB_WINDOWS macro, can't call from Rust)
void rs_adjust_topline_cursor(void *term, void *buf, int added)
{
  adjust_topline_cursor((Terminal *)term, (buf_T *)buf, added);
}

// Phase 6: termrequest buffer printf wrappers (kv_printf is a macro, can't call from Rust)
void nvim_term_treqbuf_printf_osc(void *term, int command)
{
  kv_printf(((Terminal *)term)->termrequest_buffer, "\x1b]%d;", command);
}
void nvim_term_treqbuf_printf_dcs(void *term, const char *command, int cmdlen)
{
  kv_printf(((Terminal *)term)->termrequest_buffer, "\x1bP%*s", cmdlen, command);
}
void nvim_term_treqbuf_printf_apc(void *term)
{
  kv_printf(((Terminal *)term)->termrequest_buffer, "\x1b_");
}
// Check if the TermRequest event is registered
int nvim_terminal_has_termrequest_event(void) { return (int)has_event(EVENT_TERMREQUEST); }
// Schedule a termrequest event from a Rust fallback callback
void nvim_terminal_schedule_termrequest(void *term)
{
  schedule_termrequest((Terminal *)term);
}
// Get a pointer to term->termrequest_buffer (a StringBuilder *)
void *nvim_terminal_treqbuf_ptr(void *term)
{
  return &((Terminal *)term)->termrequest_buffer;
}
// Get a pointer to term->vt for vterm_obtain_state
void *nvim_terminal_get_vt(void *term) { return ((Terminal *)term)->vt; }
// Set OSC8 URI attribute on vterm state
void nvim_term_set_osc8_attr(void *vt, int attr)
{
  VTermState *state = vterm_obtain_state((VTerm *)vt);
  VTermValue value = { .number = attr };
  vterm_state_set_penattr(state, VTERM_ATTR_URI, VTERM_VALUETYPE_INT, &value);
}

// vim: foldmethod=marker
