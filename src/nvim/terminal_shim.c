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
extern MultiQueue *rs_loop_get_events(Loop *loop);
#define loop_get_events(l) rs_loop_get_events(l)
extern int rs_terminal_row_to_linenr_term(void *term, int row);
#define REFRESH_DELAY     10       // Refresh delay (ms) for burst performance
#define TEXTBUF_SIZE      0x1fff
#define SELECTIONBUF_SIZE 0x0400
static TimeWatcher refresh_timer;
extern bool refresh_pending;
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
extern int rs_term_damage(VTermRect rect, void *data);
extern int rs_term_moverect(VTermRect dest, VTermRect src, void *data);
extern int rs_term_movecursor(VTermPos new_pos, VTermPos old_pos, int visible, void *data);
extern int rs_term_bell_cb(void *user);
extern int rs_term_theme_cb(bool *dark, void *user);
extern void rs_term_output_callback(const char *s, size_t len, void *user_data);
extern int rs_term_sb_push(int cols, const VTermScreenCell *cells, void *data);
extern int rs_term_sb_pop(int cols, VTermScreenCell *cells, void *data);
extern void rs_terminal_notify_theme_impl(void *term, int dark);
extern void rs_invalidate_terminal(void *term, int start_row, int end_row);
extern void rs_refresh_screen_pub(Terminal *term, buf_T *buf);

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
extern int rs_on_osc(int command, const char *str, size_t len, int initial, int is_final,
                     void *user);
static int on_osc(int command, VTermStringFragment frag, void *user) FUNC_ATTR_NONNULL_ALL { return rs_on_osc(command, frag.str, frag.len, (int)frag.initial, (int)frag.final, user); }
extern int rs_on_dcs(const char *command, size_t commandlen, const char *str, size_t len,
                     int initial, int is_final, void *user);
static int on_dcs(const char *command, size_t commandlen, VTermStringFragment frag, void *user) { return rs_on_dcs(command, commandlen, frag.str, frag.len, (int)frag.initial, (int)frag.final, user); }
extern int rs_on_apc(const char *str, size_t len, int initial, int is_final, void *user);
static int on_apc(VTermStringFragment frag, void *user) { return rs_on_apc(frag.str, frag.len, (int)frag.initial, (int)frag.final, user); }

static VTermStateFallbacks vterm_fallbacks = {
  .control = NULL,
  .csi = NULL,
  .osc = on_osc,
  .dcs = on_dcs,
  .apc = on_apc,
  .pm = NULL,
  .sos = NULL,
};

void nvim_terminal_init_timer(void)
{
  time_watcher_init(&main_loop, &refresh_timer, NULL);
  // refresh_timer_cb will redraw the screen which can call vimscript
  refresh_timer.events = multiqueue_new_child(loop_get_events(&main_loop));
}
void nvim_terminal_teardown_timer(void)
{
  time_watcher_stop(&refresh_timer);
  multiqueue_free(refresh_timer.events);
  time_watcher_close(&refresh_timer, NULL);
  set_destroy(ptr_t, &invalidated_terminals);
  // terminal_destroy might be called after terminal_teardown is invoked
  // make sure it is in an empty, valid state
  invalidated_terminals = (Set(ptr_t)) SET_INIT;
}
extern void rs_terminal_init(void);
void terminal_init(void) { rs_terminal_init(); }
extern void rs_terminal_teardown(void);
void terminal_teardown(void) { rs_terminal_teardown(); }
extern void rs_terminal_open(Terminal **termpp, buf_T *buf, TerminalOptions opts);
void terminal_open(Terminal **termpp, buf_T *buf, TerminalOptions opts)
  FUNC_ATTR_NONNULL_ALL { rs_terminal_open(termpp, buf, opts); }
extern void rs_terminal_close(Terminal **termpp, int status);
void terminal_close(Terminal **termpp, int status)
  FUNC_ATTR_NONNULL_ALL { rs_terminal_close(termpp, status); }
extern void rs_terminal_check_size(Terminal *term);
void terminal_check_size(Terminal *term) { rs_terminal_check_size(term); }
extern bool rs_terminal_enter(void);
bool terminal_enter(void) { return rs_terminal_enter(); }

extern void rs_terminal_check_cursor(void);
static void terminal_check_cursor(void) { rs_terminal_check_cursor(); }
extern void rs_terminal_destroy(Terminal **termpp);
void terminal_destroy(Terminal **termpp) FUNC_ATTR_NONNULL_ALL { rs_terminal_destroy(termpp); }
extern void rs_terminal_paste(int count, const String *y_array, size_t y_size);
void terminal_paste(int count, String *y_array, size_t y_size) { rs_terminal_paste(count, y_array, y_size); }
extern void rs_terminal_receive_impl(void *term, const char *data, size_t len);
void terminal_receive(Terminal *term, const char *data, size_t len) { rs_terminal_receive_impl(term, data, len); }

extern void rs_terminal_get_line_attributes(Terminal *term, win_T *wp, int linenr, int *term_attrs);
void terminal_get_line_attributes(Terminal *term, win_T *wp, int linenr, int *term_attrs) { rs_terminal_get_line_attributes(term, wp, linenr, term_attrs); }
void terminal_notify_theme(Terminal *term, bool dark) { rs_terminal_notify_theme_impl(term, (int)dark); }

extern int rs_term_settermprop(VTermProp prop, VTermValue *val, void *data);
static int term_settermprop(VTermProp prop, VTermValue *val, void *data) { return rs_term_settermprop(prop, val, data); }
extern void rs_term_clipboard_set(void **argv);
extern int rs_term_selection_set(int mask, const char *str, size_t len, int initial, int is_final,
                                  void *user);
static int term_selection_set(VTermSelectionMask mask, VTermStringFragment frag, void *user) { return rs_term_selection_set((int)mask, frag.str, frag.len, (int)frag.initial, (int)frag.final, user); }
extern void rs_refresh_terminal(void *term);
extern void rs_refresh_cursor(void *term, bool *cursor_visible);
extern void rs_refresh_timer_cb(void *watcher, void *data);
static void refresh_timer_cb(TimeWatcher *watcher, void *data) { rs_refresh_timer_cb(watcher, data); }
extern void rs_on_scrollback_option_changed(void *term);
void on_scrollback_option_changed(Terminal *term) { rs_on_scrollback_option_changed(term); }

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

char *nvim_get_config_string(const char *key)
{ Error err = ERROR_INIT;
  Object obj = dict_get_value(curbuf->b_vars, cstr_as_string(key), NULL, &err);
  api_clear_error(&err);
  if (obj.type == kObjectTypeNil) {
    obj = dict_get_value(get_globvar_dict(), cstr_as_string(key), NULL, &err);
    api_clear_error(&err);
  }
  if (obj.type == kObjectTypeString) { return obj.data.string.data; }
  api_free_object(obj); return NULL; }
void nvim_terminal_vterm_set_callbacks(void *term)
{ Terminal *t = (Terminal *)term;
  vterm_screen_set_callbacks(t->vts, &vterm_screen_callbacks, t);
  vterm_screen_set_unrecognised_fallbacks(t->vts, &vterm_fallbacks, t);
  VTermState *state = vterm_obtain_state(t->vt);
  vterm_state_set_selection_callbacks(state, &vterm_selection_callbacks, t,
                                      t->selection_buffer, SELECTIONBUF_SIZE); }
void nvim_set_option_value_buftype_terminal(void) { set_option_value(kOptBuftype, STATIC_CSTR_AS_OPTVAL("terminal"), OPT_LOCAL); }
void *nvim_aucmd_prepbuf_alloc(void *buf) { aco_save_T *aco = xmalloc(sizeof(aco_save_T)); aucmd_prepbuf(aco, (buf_T *)buf); return aco; }
void nvim_aucmd_restbuf_free(void *aco) { aucmd_restbuf((aco_save_T *)aco); xfree(aco); }
int64_t nvim_term_buf_get_changedtick(const void *buf) { return buf_get_changedtick((const buf_T *)buf); }
void nvim_terminal_vterm_set_palette(void *state, int i, int r, int g, int b)
{ VTermColor color; vterm_color_rgb(&color, (uint8_t)r, (uint8_t)g, (uint8_t)b);
  vterm_state_set_palette_color((VTermState *)state, i, &color); }
void nvim_terminal_set_put(void *term) { set_put(ptr_t, &invalidated_terminals, (Terminal *)term); }

int nvim_terminal_invalidated_check_del(void *term)
{ if (!set_has(ptr_t, &invalidated_terminals, (Terminal *)term)) { return 0; }
  block_autocmds(); rs_refresh_terminal((Terminal *)term); unblock_autocmds();
  set_del(ptr_t, &invalidated_terminals, (Terminal *)term); return 1; }
void nvim_terminal_timer_start(void) { time_watcher_start(&refresh_timer, refresh_timer_cb, REFRESH_DELAY, 0); }
void nvim_terminal_buf_set_title(void *buf, const char *title, size_t len)
{ Error err = ERROR_INIT;
  dict_set_var(((buf_T *)buf)->b_vars, STATIC_CSTR_AS_STRING("term_title"),
               STRING_OBJ(((String){ .data = (char *)title, .size = len })),
               false, false, NULL, &err);
  api_clear_error(&err); status_redraw_buf((buf_T *)buf); }
void nvim_shape_table_set_cursor(int blink, int shape, int percentage)
{ shape_table[SHAPE_IDX_TERM].blinkon = blink ? 500 : 0;
  shape_table[SHAPE_IDX_TERM].blinkoff = blink ? 500 : 0;
  shape_table[SHAPE_IDX_TERM].shape = shape; shape_table[SHAPE_IDX_TERM].percentage = percentage; }

void nvim_terminal_foreach_invalidated(void (*fn)(void *term, void *ctx), void *ctx)
{
  Terminal *term; void *stub; (void)(stub);
  block_autocmds();
  set_foreach(&invalidated_terminals, term, { fn(term, ctx); });
  set_clear(ptr_t, &invalidated_terminals);
  unblock_autocmds();
}
int nvim_terminal_opts_is_internal(void *term) { return ((Channel *)((Terminal *)term)->opts.data)->streamtype == kChannelStreamInternal; }
void nvim_terminal_apply_termclose_event(void *buf, int status)
{
  if (!buf || is_autocmd_blocked()) { return; }
  save_v_event_T save_v_event;
  dict_T *dict = get_v_event(&save_v_event);
  tv_dict_add_nr(dict, S_LEN("status"), status);
  tv_dict_set_keys_readonly(dict);
  apply_autocmds(EVENT_TERMCLOSE, NULL, NULL, false, (buf_T *)buf);
  restore_v_event(dict, &save_v_event);
}
void rs_adjust_topline_cursor(void *term, void *buf, int added) { adjust_topline_cursor((Terminal *)term, (buf_T *)buf, added); }
void nvim_terminal_find_size(void *term, uint16_t *out_width, uint16_t *out_height)
{
  uint16_t width = 0, height = 0;
  FOR_ALL_TAB_WINDOWS(tp, wp) {
    if (is_aucmd_win(wp) || !wp->w_buffer || wp->w_buffer->terminal != (Terminal *)term) { continue; }
    width = MAX(width, (uint16_t)(MAX(0, wp->w_view_width - win_col_off(wp))));
    height = (uint16_t)MAX(height, wp->w_view_height);
  }
  *out_width = width; *out_height = height;
}
void nvim_set_got_int(int v) { got_int = (bool)v; }
int nvim_must_redraw(void) { return (int)must_redraw; }
extern void rs_set_terminal_winopts(void *s);
extern void rs_unset_terminal_winopts(void *s);
void nvim_win_set_p_culopt(win_T *wp, char *s) { wp->w_p_culopt = s; }
void nvim_terminal_clipboard_queue(long mask, char *data) { multiqueue_put(loop_get_events(&main_loop), rs_term_clipboard_set, (void *)mask, data); }
void nvim_terminal_set_vim_var_termrequest(const char *seq, size_t seqlen) { set_vim_var_string(VV_TERMREQUEST, seq, (ptrdiff_t)seqlen); }
void nvim_terminal_apply_termrequest_autocmd(void *buf, int64_t row, int64_t col,
                                             const char *seq, size_t seqlen)
{
  MAXSIZE_TEMP_ARRAY(cursor, 2); ADD_C(cursor, INTEGER_OBJ(row)); ADD_C(cursor, INTEGER_OBJ(col));
  MAXSIZE_TEMP_DICT(data, 2);
  PUT_C(data, "sequence", STRING_OBJ(((String){ .data = (char *)seq, .size = seqlen })));
  PUT_C(data, "cursor", ARRAY_OBJ(cursor));
  apply_autocmds_group(EVENT_TERMREQUEST, NULL, NULL, true, AUGROUP_ALL, (buf_T *)buf, NULL, &DICT_OBJ(data));
}
void nvim_terminal_pending_put_termrequest(void *term, emit_termrequest_fn_t fn,
                                           char *sequence, size_t seqlen,
                                           void *pending_send, intptr_t row, intptr_t col,
                                           intptr_t sb_deleted) { multiqueue_put(((Terminal *)term)->pending.events, fn, term, sequence, (void *)seqlen, pending_send, (void *)row, (void *)col, (void *)sb_deleted); }
void nvim_terminal_main_put_termrequest(emit_termrequest_fn_t fn, void *term,
                                        char *sequence, size_t seqlen,
                                        void *pending_send, intptr_t row, intptr_t col,
                                        intptr_t sb_deleted) { multiqueue_put(loop_get_events(&main_loop), fn, term, sequence, (void *)seqlen, pending_send, (void *)row, (void *)col, (void *)sb_deleted); }
// Wraps the do_mousescroll block: saves/restores curwin/curbuf, calls do_mousescroll,
// redraws status, and returns whether mouse_win == old curwin.
int nvim_do_mousescroll_c(void *term, void *mouse_win, int c)
{
  win_T *save_curwin = curwin;
  curwin = (win_T *)mouse_win;
  curbuf = curwin->w_buffer;
  cmdarg_T cap;
  oparg_T oa;
  CLEAR_FIELD(cap);
  clear_oparg(&oa);
  cap.oap = &oa;
  cap.cmdchar = c;
  switch (c) {
  case K_MOUSEUP:    cap.arg = MSCR_UP; break;
  case K_MOUSEDOWN:  cap.arg = MSCR_DOWN; break;
  case K_MOUSELEFT:  cap.arg = MSCR_LEFT; break;
  case K_MOUSERIGHT: cap.arg = MSCR_RIGHT; break;
  default: abort();
  }
  do_mousescroll(&cap);
  curwin->w_redr_status = true;
  curwin = save_curwin;
  curbuf = curwin->w_buffer;
  redraw_later((win_T *)mouse_win, UPD_NOT_VALID);
  rs_invalidate_terminal(term, -1, -1);
  return (win_T *)mouse_win == curwin;
}
