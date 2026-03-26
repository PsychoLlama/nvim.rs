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

extern void rs_emit_termrequest(void **argv);
extern void rs_schedule_termrequest(void *term);


extern int rs_on_osc(int command, const char *str, size_t len, int initial, int is_final,
                     void *user);
static int on_osc(int command, VTermStringFragment frag, void *user) FUNC_ATTR_NONNULL_ALL
  { return rs_on_osc(command, frag.str, frag.len, (int)frag.initial, (int)frag.final, user); }

extern int rs_on_dcs(const char *command, size_t commandlen, const char *str, size_t len,
                     int initial, int is_final, void *user);
static int on_dcs(const char *command, size_t commandlen, VTermStringFragment frag, void *user)
  { return rs_on_dcs(command, commandlen, frag.str, frag.len, (int)frag.initial, (int)frag.final, user); }

extern int rs_on_apc(const char *str, size_t len, int initial, int is_final, void *user);
static int on_apc(VTermStringFragment frag, void *user)
  { return rs_on_apc(frag.str, frag.len, (int)frag.initial, (int)frag.final, user); }

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
extern void rs_terminal_close(Terminal **termpp, int status);
void terminal_close(Terminal **termpp, int status)
  FUNC_ATTR_NONNULL_ALL { rs_terminal_close(termpp, status); }

extern void rs_terminal_check_size(Terminal *term);
void terminal_check_size(Terminal *term) { rs_terminal_check_size(term); }

/// Implements MODE_TERMINAL state. :help Terminal-mode
extern bool rs_terminal_enter(void);
bool terminal_enter(void) { return rs_terminal_enter(); }

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


extern void rs_terminal_destroy(Terminal **termpp);
void terminal_destroy(Terminal **termpp) FUNC_ATTR_NONNULL_ALL { rs_terminal_destroy(termpp); }

extern void rs_terminal_do_send(void *term, const char *data, size_t size);
static void terminal_send(Terminal *term, const char *data, size_t size)
  { rs_terminal_do_send(term, data, size); }

extern void rs_terminal_paste(int count, const String *y_array, size_t y_size);
void terminal_paste(int count, String *y_array, size_t y_size)
  { rs_terminal_paste(count, y_array, y_size); }

extern void rs_terminal_receive_impl(void *term, const char *data, size_t len);
void terminal_receive(Terminal *term, const char *data, size_t len)
  { rs_terminal_receive_impl(term, data, len); }

static int get_rgb(VTermState *state, VTermColor color)
{
  vterm_state_convert_color_to_rgb(state, &color);
  return RGB_(color.rgb.red, color.rgb.green, color.rgb.blue);
}


extern void rs_terminal_get_line_attributes(Terminal *term, win_T *wp, int linenr, int *term_attrs);
void terminal_get_line_attributes(Terminal *term, win_T *wp, int linenr, int *term_attrs)
  { rs_terminal_get_line_attributes(term, wp, linenr, term_attrs); }

void terminal_notify_theme(Terminal *term, bool dark)
  { rs_terminal_notify_theme_impl(term, (int)dark); }

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
  { return rs_term_settermprop(prop, val, data); }


extern void rs_term_clipboard_set(void **argv);
static void term_clipboard_set(void **argv) { rs_term_clipboard_set(argv); }

extern int rs_term_selection_set(int mask, const char *str, size_t len, int initial, int is_final,
                                  void *user);
static int term_selection_set(VTermSelectionMask mask, VTermStringFragment frag, void *user)
  { return rs_term_selection_set((int)mask, frag.str, frag.len, (int)frag.initial, (int)frag.final, user); }

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
static void invalidate_terminal(Terminal *term, int start_row, int end_row)
  { rs_invalidate_terminal(term, start_row, end_row); }

extern void rs_refresh_terminal(void *term);
static void refresh_terminal(Terminal *term) { rs_refresh_terminal(term); }

extern void rs_refresh_cursor(void *term, bool *cursor_visible);
static void refresh_cursor(Terminal *term, bool *cursor_visible) FUNC_ATTR_NONNULL_ALL
  { rs_refresh_cursor(term, cursor_visible); }

extern void rs_refresh_timer_cb(void *watcher, void *data);
static void refresh_timer_cb(TimeWatcher *watcher, void *data) { rs_refresh_timer_cb(watcher, data); }

extern void rs_on_scrollback_option_changed(void *term);
void on_scrollback_option_changed(Terminal *term) { rs_on_scrollback_option_changed(term); }

// refresh_screen, adjust_scrollback, refresh_scrollback are implemented in Rust
extern void rs_refresh_screen_pub(Terminal *term, buf_T *buf);
static void refresh_screen(Terminal *term, buf_T *buf) { rs_refresh_screen_pub(term, buf); }

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

// Accessor functions for Rust callbacks
void nvim_terminal_invalidate(void *term, int start_row, int end_row)
  { invalidate_terminal((Terminal *)term, start_row, end_row); }
void nvim_terminal_send(void *term, const char *data, size_t size)
  { terminal_send((Terminal *)term, data, size); }
void nvim_terminal_set_put(void *term)
  { set_put(ptr_t, &invalidated_terminals, (Terminal *)term); }

// StringBuilder (kv_*) accessors for Rust
void nvim_term_sb_concat_len(void *sb, const char *data, size_t len)
  { kv_concat_len(*(StringBuilder *)sb, data, len); }
size_t nvim_term_sb_size(const void *sb) { return kv_size(*(const StringBuilder *)sb); }
char *nvim_term_sb_items(void *sb) { return ((StringBuilder *)sb)->items; }
void nvim_term_sb_reset(void *sb) { ((StringBuilder *)sb)->size = 0; }
void nvim_term_sb_push_char(void *sb, char c) { kv_push(*(StringBuilder *)sb, c); }

// ScrollbackLine accessors for Rust
size_t nvim_scrollback_line_size(size_t cols)
  { return sizeof(ScrollbackLine) + cols * sizeof(VTermScreenCell); }
size_t nvim_scrollback_line_cols(const void *sbrow) { return ((const ScrollbackLine *)sbrow)->cols; }
const void *nvim_scrollback_line_cells(const void *sbrow) { return ((const ScrollbackLine *)sbrow)->cells; }
void *nvim_scrollback_line_cells_mut(void *sbrow) { return ((ScrollbackLine *)sbrow)->cells; }
size_t nvim_vterm_screen_cell_size(void) { return sizeof(VTermScreenCell); }
void nvim_vterm_cell_zero(void *cell_ptr)
  { VTermScreenCell *c = (VTermScreenCell *)cell_ptr; c->schar = 0; c->width = 1; }

// terminal_destroy helpers
int nvim_terminal_invalidated_check_del(void *term)
{
  if (set_has(ptr_t, &invalidated_terminals, (Terminal *)term)) {
    block_autocmds();
    refresh_terminal((Terminal *)term);
    unblock_autocmds();
    set_del(ptr_t, &invalidated_terminals, (Terminal *)term);
    return 1;
  }
  return 0;
}
void nvim_buf_set_terminal(void *buf, void *term) { ((buf_T *)buf)->terminal = (Terminal *)term; }
void nvim_term_sb_destroy(void *sb) { kv_destroy(*(StringBuilder *)sb); }
void nvim_vterm_free(void *vt) { vterm_free((VTerm *)vt); }
void nvim_multiqueue_free(void *q) { multiqueue_free((MultiQueue *)q); }

// terminal_get_line_attributes helpers
int nvim_fetch_cell(void *term, int row, int col, void *cell)
  { return (int)fetch_cell((Terminal *)term, row, col, (VTermScreenCell *)cell); }
int nvim_get_rgb(void *state, VTermColor color)
  { return get_rgb((VTermState *)state, color); }

// terminal_paste helpers
int nvim_term_utf_ptr2len(const char *s) { return utf_ptr2len(s); }
int nvim_term_utf_ptr2char(const char *s) { return utf_ptr2char(s); }
int nvim_terminal_get_tpf_flags(void) { return (int)tpf_flags; }
Terminal *nvim_curbuf_terminal(void) { return curbuf->terminal; }

// Timer / refresh_pending accessors
void nvim_terminal_timer_start(void)
  { time_watcher_start(&refresh_timer, refresh_timer_cb, REFRESH_DELAY, 0); }
int nvim_terminal_get_refresh_pending(void) { return (int)refresh_pending; }
void nvim_terminal_set_refresh_pending(int v) { refresh_pending = (bool)v; }

// Buffer / title accessors
void *nvim_terminal_handle_get_buffer(int buf_handle) { return handle_get_buffer(buf_handle); }
void nvim_terminal_buf_set_title(void *buf, const char *title, size_t len)
  { buf_set_term_title((buf_T *)buf, title, len); }
void *nvim_term_xrealloc(void *ptr, size_t size) { return xrealloc(ptr, size); }
void nvim_terminal_write_cb(void *term, const char *data, size_t size)
  { Terminal *t = (Terminal *)term; t->opts.write_cb(data, size, t->opts.data); }
void *nvim_terminal_get_pending_send(void *term) { return ((Terminal *)term)->pending.send; }

// VTermValue accessors for term_settermprop
int nvim_vterm_value_boolean(const void *val) { return ((const VTermValue *)val)->boolean; }
int nvim_vterm_value_number(const void *val) { return ((const VTermValue *)val)->number; }
const char *nvim_vterm_frag_str(const void *val) { return ((const VTermValue *)val)->string.str; }
size_t nvim_vterm_frag_len(const void *val) { return ((const VTermValue *)val)->string.len; }
int nvim_vterm_frag_initial(const void *val) { return (int)((const VTermValue *)val)->string.initial; }
int nvim_vterm_frag_final(const void *val) { return (int)((const VTermValue *)val)->string.final; }

// Buffer / memline accessors for Rust refresh pipeline
int nvim_term_buf_line_count(const void *buf) { return ((const buf_T *)buf)->b_ml.ml_line_count; }
int64_t nvim_buf_get_scrollback(const void *buf) { return ((const buf_T *)buf)->b_p_scbk; }
void nvim_buf_set_scrollback(void *buf, int64_t val) { ((buf_T *)buf)->b_p_scbk = val; }
int nvim_rs_buf_valid(void *buf) { return rs_buf_valid((buf_T *)buf); }
void *nvim_terminal_get_buffer(int buf_handle) { return handle_get_buffer(buf_handle); }
int nvim_ml_append_buf_term(void *buf, int lnum, char *line, bool newfile)
  { return ml_append_buf((buf_T *)buf, (linenr_T)lnum, line, 0, newfile); }
int nvim_ml_replace_buf_term(void *buf, int lnum, char *line, bool copy)
  { return ml_replace_buf((buf_T *)buf, (linenr_T)lnum, line, copy, false); }
int nvim_ml_delete_buf_term(void *buf, int lnum)
  { return ml_delete_buf((buf_T *)buf, (linenr_T)lnum, false); }
void nvim_mark_adjust_buf_term(void *buf, int line1, int line2, int amount, int amount_after)
{
  mark_adjust_buf((buf_T *)buf, (linenr_T)line1, (linenr_T)line2, (linenr_T)amount,
                  (linenr_T)amount_after, true, kMarkAdjustTerm, kExtmarkUndo);
}
void nvim_appended_lines_buf_term(void *buf, int lnum, int count)
  { appended_lines_buf((buf_T *)buf, (linenr_T)lnum, (linenr_T)count); }
void nvim_deleted_lines_buf_term(void *buf, int lnum, int count)
  { deleted_lines_buf((buf_T *)buf, (linenr_T)lnum, (linenr_T)count); }
void nvim_changed_lines_term(void *buf, int first, int last, int added)
  { changed_lines((buf_T *)buf, (linenr_T)first, 0, (linenr_T)last, (linenr_T)added, true); }
void nvim_multiqueue_move_events_term(void *term)
  { Terminal *t = (Terminal *)term; multiqueue_move_events(loop_get_events(&main_loop), t->pending.events); }
void *nvim_terminal_sb_get(void *term, size_t idx) { return ((Terminal *)term)->sb_buffer[idx]; }
void nvim_terminal_sb_set(void *term, size_t idx, void *sbrow)
  { ((Terminal *)term)->sb_buffer[idx] = (ScrollbackLine *)sbrow; }
void nvim_terminal_sb_buffer_resize(void *term, size_t new_size)
  { Terminal *t = (Terminal *)term; t->sb_buffer = xrealloc(t->sb_buffer, sizeof(ScrollbackLine *) * new_size); t->sb_size = new_size; }
void nvim_fetch_row(void *term, int row, int end_col) { fetch_row((Terminal *)term, row, end_col); }

// Cursor / UI accessors
int nvim_terminal_is_active(void *term)
  { return (State & MODE_TERMINAL) && curbuf->terminal == (Terminal *)term; }
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

// Invalidated-terminals set + exiting
void nvim_terminal_foreach_invalidated(void (*fn)(void *term, void *ctx), void *ctx)
{
  Terminal *term;
  void *stub; (void)(stub);
  block_autocmds();
  set_foreach(&invalidated_terminals, term, { fn(term, ctx); });
  set_clear(ptr_t, &invalidated_terminals);
  unblock_autocmds();
}
int nvim_is_exiting(void) { return exiting; }

// terminal_close helpers
int nvim_entered_free_all_mem(void)
{
#ifdef EXITFREE
  return (int)entered_free_all_mem;
#else
  return 0;
#endif
}
void nvim_terminal_refresh_blocking(void *term)
  { block_autocmds(); refresh_terminal((Terminal *)term); unblock_autocmds(); }
int nvim_terminal_opts_is_internal(void *term)
  { return ((Channel *)((Terminal *)term)->opts.data)->streamtype == kChannelStreamInternal; }
void nvim_terminal_call_close_cb(void *term)
  { Terminal *t = (Terminal *)term; ((void (*)(void *))t->opts.close_cb)(t->opts.data); }
void nvim_terminal_apply_termclose_event(void *buf, int status)
{
  if (buf && !is_autocmd_blocked()) {
    save_v_event_T save_v_event;
    dict_T *dict = get_v_event(&save_v_event);
    tv_dict_add_nr(dict, S_LEN("status"), status);
    tv_dict_set_keys_readonly(dict);
    apply_autocmds(EVENT_TERMCLOSE, NULL, NULL, false, (buf_T *)buf);
    restore_v_event(dict, &save_v_event);
  }
}
// FOR_ALL_TAB_WINDOWS wrappers (macro not callable from Rust)
void rs_adjust_topline_cursor(void *term, void *buf, int added)
  { adjust_topline_cursor((Terminal *)term, (buf_T *)buf, added); }
void nvim_terminal_find_size(void *term, uint16_t *out_width, uint16_t *out_height)
{
  uint16_t width = 0;
  uint16_t height = 0;
  FOR_ALL_TAB_WINDOWS(tp, wp) {
    if (is_aucmd_win(wp)) { continue; }
    if (wp->w_buffer && wp->w_buffer->terminal == (Terminal *)term) {
      const uint16_t win_width = (uint16_t)(MAX(0, wp->w_view_width - win_col_off(wp)));
      width = MAX(width, win_width);
      height = (uint16_t)MAX(height, wp->w_view_height);
    }
  }
  *out_width = width;
  *out_height = height;
}

// kv_printf wrappers (macro not callable from Rust)
void nvim_term_treqbuf_printf_osc(void *term, int command)
  { kv_printf(((Terminal *)term)->termrequest_buffer, "\x1b]%d;", command); }
void nvim_term_treqbuf_printf_dcs(void *term, const char *command, int cmdlen)
  { kv_printf(((Terminal *)term)->termrequest_buffer, "\x1bP%*s", cmdlen, command); }
void nvim_term_treqbuf_printf_apc(void *term)
  { kv_printf(((Terminal *)term)->termrequest_buffer, "\x1b_"); }
int nvim_terminal_has_termrequest_event(void) { return (int)has_event(EVENT_TERMREQUEST); }
void nvim_terminal_schedule_termrequest(void *term) { rs_schedule_termrequest(term); }
void *nvim_terminal_treqbuf_ptr(void *term) { return &((Terminal *)term)->termrequest_buffer; }
void *nvim_terminal_get_vt(void *term) { return ((Terminal *)term)->vt; }
void nvim_term_set_osc8_attr(void *vt, int attr)
{
  VTermState *state = vterm_obtain_state((VTerm *)vt);
  VTermValue value = { .number = attr };
  vterm_state_set_penattr(state, VTERM_ATTR_URI, VTERM_VALUETYPE_INT, &value);
}

// Phase 14: terminal_enter state machine helpers

// vim state
// nvim_get_state - already in cursor_shape.c
// nvim_set_state - already in change_ffi.c
// nvim_get_RedrawingDisabled/nvim_set_RedrawingDisabled - already in window_shim.c
// nvim_get/set_mapped_ctrl_c - already in getchar.c
// nvim_get/set_stop_insert_mode - already in nvim-window crate
// nvim_get/set_restart_edit - already in ex_docmd.c
void nvim_set_got_int(int v) { got_int = (bool)v; }

// cursor/display
// nvim_showmode - already in normal_shim.c
void nvim_unshowmode(void) { unshowmode(true); }
// nvim_ui_cursor_shape - already in mouse.c
void nvim_setcursor(void) { setcursor(); }
void nvim_parse_shape_opt(int scope) { (void)parse_shape_opt(scope); }
void nvim_show_cursor_info_later(void) { show_cursor_info_later(false); }
void nvim_refresh_cursor_c(void *term, int *cursor_visible)
  { bool vis = (bool)*cursor_visible; refresh_cursor((Terminal *)term, &vis); *cursor_visible = (int)vis; }

// redraw
void nvim_validate_cursor_cw(void) { validate_cursor(curwin); }
void nvim_update_screen_c(void) { update_screen(); }
void nvim_redraw_statuslines(void) { redraw_statuslines(); }
int nvim_must_redraw(void) { return (int)must_redraw; }
int nvim_clear_cmdline(void) { return (int)clear_cmdline; }
int nvim_redraw_cmdline(void) { return (int)redraw_cmdline; }
int nvim_redraw_mode(void) { return (int)redraw_mode; }
void nvim_ui_flush(void) { ui_flush(); }

// autocmds
void nvim_apply_termenter_autocmd(void) { apply_autocmds(EVENT_TERMENTER, NULL, NULL, false, curbuf); }
void nvim_apply_termleave_autocmd(void) { apply_autocmds(EVENT_TERMLEAVE, NULL, NULL, false, curbuf); }
void nvim_apply_textchangedt_autocmd(void) { apply_autocmds(EVENT_TEXTCHANGEDT, NULL, NULL, false, curbuf); }
// nvim_may_trigger_modechanged - already in normal_shim.c
void nvim_may_trigger_win_scrolled_resized(void) { may_trigger_win_scrolled_resized(); }
int nvim_has_event_textchangedt(void) { return (int)has_event(EVENT_TEXTCHANGEDT); }

// changedtick
// nvim_buf_get_changedtick - already defined (inline in buffer.h)
void nvim_curbuf_update_changedtick_i(void) { curbuf->b_last_changedtick_i = buf_get_changedtick(curbuf); }
void nvim_curbuf_update_changedtick(void) { curbuf->b_last_changedtick = buf_get_changedtick(curbuf); }
int nvim_curbuf_last_changedtick_i(void) { return (int)curbuf->b_last_changedtick_i; }

// state machine
void nvim_state_enter_c(void *state) { state_enter((VimState *)state); }

// key handling
// nvim_get_mod_mask - already in getchar.c
int nvim_merge_modifiers_c(int key, int *tmp_mod_mask) { return merge_modifiers(key, tmp_mod_mask); }
void nvim_paste_repeat_c(void) { paste_repeat(1); }
// nvim_state_handle_k_event - already in normal_shim.c
void nvim_do_cmdline_key_cmd(void) { do_cmdline(NULL, getcmdkeycmd, NULL, 0); }
void nvim_map_execute_lua_c(void) { map_execute_lua(false, false); }

// terminal mode window options
extern void rs_set_terminal_winopts(void *s);
extern void rs_unset_terminal_winopts(void *s);
void nvim_terminal_set_winopts(void *s) { rs_set_terminal_winopts(s); }
void nvim_terminal_unset_winopts(void *s) { rs_unset_terminal_winopts(s); }
// window option accessors for rs_set_terminal_winopts / rs_unset_terminal_winopts
// (nvim_win_get_p_cul/cuc/so/siso are already in nvim-window crate; nvim_xstrdup in register.c)
win_T *nvim_curwin_ptr(void) { return curwin; }
void nvim_win_set_p_cul(win_T *wp, bool v) { wp->w_p_cul = v; }
void nvim_win_set_p_cuc(win_T *wp, bool v) { wp->w_p_cuc = v; }
void nvim_win_set_p_so(win_T *wp, int64_t v) { wp->w_p_so = v; }
void nvim_win_set_p_siso(win_T *wp, int64_t v) { wp->w_p_siso = v; }
void nvim_win_redraw_later_some_valid(win_T *wp) { redraw_later(wp, UPD_SOME_VALID); }
void nvim_win_redraw_later_valid(win_T *wp) { redraw_later(wp, UPD_VALID); }
void nvim_free_string_option(char *str) { free_string_option(str); }
void nvim_win_set_p_culopt(win_T *wp, char *s) { wp->w_p_culopt = s; }
void nvim_terminal_check_cursor_c(void) { terminal_check_cursor(); }
int nvim_terminal_send_mouse_event_c(void *term, int c) { return (int)send_mouse_event((Terminal *)term, c); }
int nvim_curwin_handle(void) { return curwin->handle; }
int nvim_buf_get_changedtick_curbuf(void) { return (int)buf_get_changedtick(curbuf); }

// buffer wipe
void nvim_do_buffer_wipe(int buf_handle)
  { do_buffer_ext(DOBUF_WIPE, DOBUF_FIRST, FORWARD, (handle_T)buf_handle, DOBUF_FORCEIT); }

// term_selection_set / term_clipboard_set helpers (Phase 16)
void nvim_terminal_clipboard_queue(long mask, char *data)
  { multiqueue_put(loop_get_events(&main_loop), term_clipboard_set, (void *)mask, data); }
char *nvim_terminal_selection_dupz(void *term, size_t *out_len)
{
  Terminal *t = (Terminal *)term;
  *out_len = kv_size(t->selection);
  return xmemdupz(t->selection.items, *out_len);
}

// emit_termrequest helpers (Phase 15)
void nvim_terminal_set_vim_var_termrequest(const char *seq, size_t seqlen)
  { set_vim_var_string(VV_TERMREQUEST, seq, (ptrdiff_t)seqlen); }
void nvim_terminal_apply_termrequest_autocmd(void *buf, int64_t row, int64_t col,
                                             const char *seq, size_t seqlen)
{
  MAXSIZE_TEMP_ARRAY(cursor, 2);
  ADD_C(cursor, INTEGER_OBJ(row));
  ADD_C(cursor, INTEGER_OBJ(col));
  MAXSIZE_TEMP_DICT(data, 2);
  String termrequest = { .data = (char *)seq, .size = seqlen };
  PUT_C(data, "sequence", STRING_OBJ(termrequest));
  PUT_C(data, "cursor", ARRAY_OBJ(cursor));
  apply_autocmds_group(EVENT_TERMREQUEST, NULL, NULL, true, AUGROUP_ALL, (buf_T *)buf, NULL,
                       &DICT_OBJ(data));
}
typedef void (*emit_termrequest_fn_t)(void **);
void nvim_terminal_pending_put_termrequest(void *term, emit_termrequest_fn_t fn,
                                           char *sequence, size_t seqlen,
                                           void *pending_send, intptr_t row, intptr_t col,
                                           intptr_t sb_deleted)
{
  multiqueue_put(((Terminal *)term)->pending.events, fn, term, sequence, (void *)seqlen,
                 pending_send, (void *)row, (void *)col, (void *)sb_deleted);
}
void nvim_terminal_main_put_termrequest(emit_termrequest_fn_t fn, void *term,
                                        char *sequence, size_t seqlen,
                                        void *pending_send, intptr_t row, intptr_t col,
                                        intptr_t sb_deleted)
{
  multiqueue_put(loop_get_events(&main_loop), fn, term, sequence, (void *)seqlen,
                 pending_send, (void *)row, (void *)col, (void *)sb_deleted);
}
void *nvim_term_sb_alloc_init(void)
{
  StringBuilder *sb = xmalloc(sizeof(StringBuilder));
  kv_init(*sb);
  return sb;
}

// vim: foldmethod=marker
