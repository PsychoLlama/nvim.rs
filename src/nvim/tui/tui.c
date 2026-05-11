// Terminal UI functions. Invoked by the UI process (ui_client.c), not the server.

#include <assert.h>
#include <inttypes.h>
#include <signal.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <uv.h>

#include "auto/config.h"
#include "klib/kvec.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/ascii_defs.h"
#include "nvim/cursor_shape.h"
#include "nvim/event/defs.h"
#include "nvim/event/loop.h"
#include "nvim/event/multiqueue.h"
#include "nvim/event/signal.h"
#include "nvim/event/stream.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/highlight_defs.h"
#include "nvim/log.h"
#include "nvim/macros_defs.h"
#include "nvim/main.h"
#include "nvim/map_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memory.h"
#include "nvim/msgpack_rpc/channel.h"
#include "nvim/os/input.h"
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/strings.h"
#include "nvim/tui/input.h"
#include "nvim/tui/terminfo.h"
#include "nvim/tui/tui.h"
#include "nvim/types_defs.h"
#include "nvim/ugrid.h"
#include "nvim/ui_client.h"
#include "nvim/ui_defs.h"

#ifdef MSWIN
# include "nvim/os/os_win_console.h"
#endif

// Maximum amount of time (in ms) to wait to receive a Device Attributes
// response before exiting.
#define EXIT_TIMEOUT_MS 1000

#define OUTBUF_SIZE 0xffff

#define TOO_MANY_EVENTS 1000000

typedef struct {
  int top, bot, left, right;
} Rect;

struct TUIData {
  Loop *loop;
  char buf[OUTBUF_SIZE];
  char *buf_to_flush;  ///< If non-null, flush this instead of buf[].
  size_t bufpos;
  TermInput input;
  uv_loop_t write_loop;
  TerminfoEntry ti;
  char *term;  ///< value of $TERM
  union {
    uv_tty_t tty;
    uv_pipe_t pipe;
  } output_handle;
  bool out_isatty;
  SignalWatcher winch_handle;
  uv_timer_t startup_delay_timer;
  UGrid grid;
  kvec_t(Rect) invalid_regions;
  int row, col;
  int out_fd;
  int pending_resize_events;
  bool terminfo_found_in_db;
  bool can_change_scroll_region;
  bool has_left_and_right_margin_mode;
  bool has_sync_mode;
  bool can_set_lr_margin;  // smglr
  bool can_scroll;
  bool can_erase_chars;
  bool immediate_wrap_after_last_column;
  bool bce;
  bool mouse_enabled;
  bool mouse_move_enabled;
  bool mouse_enabled_save;
  bool title_enabled;
  bool sync_output;
  bool busy, is_invisible, want_invisible;
  bool set_cursor_color_as_str;
  bool cursor_has_color;
  bool is_starting;
  bool resize_events_enabled;

  // Terminal modes that Nvim enabled that it must disable on exit
  struct {
    bool grapheme_clusters : 1;
    bool theme_updates : 1;
    bool resize_events : 1;
  } modes;

  FILE *screenshot;
  cursorentry_T cursor_shapes[SHAPE_IDX_COUNT];
  HlAttrs clear_attrs;
  kvec_t(HlAttrs) attrs;
  int print_attr_id;
  bool default_attr;
  bool set_default_colors;
  bool can_clear_attr;
  ModeShape showing_mode;
  Integer verbose;
  struct {
    char *enable_focus_reporting;
    char *disable_focus_reporting;
    char *reset_scroll_region;
    char *enter_altfont_mode;
  } terminfo_ext;
  bool can_set_title;
  bool can_set_underline_color;
  bool can_resize_screen;
  bool stopped;
  int width;
  int height;
  bool rgb;
  bool screen_or_tmux;
  int url;  ///< Index of URL currently being printed, if any
  StringBuilder urlbuf;  ///< Re-usable buffer for writing OSC 8 control sequences
  Arena ti_arena;
};

static bool cursor_style_enabled = false;
#include "tui/tui.c.generated.h"

// Rust implementation in nvim-event crate
extern int rs_rstream_did_eof(RStream *stream);
#define rstream_did_eof(s) rs_rstream_did_eof(s)

// Rust terminal detection structures and functions

/// Context for terminal detection - passed to Rust
typedef struct {
  const char *term;
  const char *colorterm;
  int vte_version;
  int konsole_version;
  int iterm_env;
  int nsterm;
  int has_xterm_version;
  int tmux_env;
  const char *wezterm_version;
} TermDetectContext;

/// Mutable terminfo state
typedef struct {
  int bce;
  int max_colors;
  int has_tc_or_rgb;
  int has_su;
  const char **defs;
} TerminfoState;

/// Output flags from terminal detection
typedef struct {
  int can_resize_screen;
  int can_set_title;
  const char *reset_scroll_region;
  const char *enable_focus_reporting;
  const char *disable_focus_reporting;
  int set_cursor_color_as_str;
  int key_encoding;
  int enable_extended_underline;
} TermDetectOutput;

extern void rs_patch_terminfo_bugs(const TermDetectContext *ctx, TerminfoState *state);
extern void rs_augment_terminfo(const TermDetectContext *ctx, TerminfoState *state,
                                TermDetectOutput *output);
extern int rs_term_has_truecolor(const char *colorterm, int has_tc_or_rgb, const char **defs);

// Rust TUI output functions
extern void rs_tui_grid_cursor_goto(TUIData *tui, int64_t row, int64_t col);
extern void rs_tui_hl_attr_define(TUIData *tui, int64_t id, HlAttrs attrs, HlAttrs cterm_attrs);
extern void rs_tui_default_colors_set(TUIData *tui, int64_t rgb_fg, int64_t rgb_bg,
                                      int64_t rgb_sp, int64_t cterm_fg, int64_t cterm_bg);
extern void rs_tui_grid_resize(TUIData *tui, int64_t g, int64_t width, int64_t height);
extern void rs_tui_grid_clear(TUIData *tui, int64_t g);
extern void rs_tui_busy_start(TUIData *tui);
extern void rs_tui_busy_stop(TUIData *tui);
extern void rs_tui_bell(TUIData *tui);
extern void rs_tui_set_icon(TUIData *tui);
extern void rs_tui_mouse_on(TUIData *tui);
extern void rs_tui_mouse_off(TUIData *tui);
extern void rs_tui_update_menu(TUIData *tui);
extern void rs_tui_visual_bell(TUIData *tui);
extern void rs_tui_grid_scroll(TUIData *tui, int64_t g, int64_t startrow, int64_t endrow,
                               int64_t startcol, int64_t endcol, int64_t rows, int64_t cols);
extern bool tui_is_stopped(TUIData *tui);
extern void rs_tui_set_title(TUIData *tui, const char *data, size_t size);
extern void rs_tui_enable_extended_underline(TUIData *tui);
extern void rs_tui_query_bg_color(TUIData *tui);
extern void rs_out(TUIData *tui, const char *str, size_t len);
extern void rs_out_len(TUIData *tui, const char *str);
extern void rs_print_cell(TUIData *tui, char *buf, sattr_T attr);
extern void rs_update_attrs(TUIData *tui, int attr_id);
extern void rs_cursor_goto(TUIData *tui, int row, int col);
extern void rs_clear_region(TUIData *tui, int top, int bot, int left, int right, int attr_id);
extern void rs_invalidate(TUIData *tui, int top, int bot, int left, int right);
extern void rs_tui_flush(TUIData *tui);
extern void rs_tui_set_size(TUIData *tui, int width, int height);
extern void rs_tui_set_mode(TUIData *tui, int mode);
extern void rs_tui_guess_size(TUIData *tui);
extern void rs_tui_mode_change(TUIData *tui, int64_t mode_idx);
extern void rs_terminfo_disable(TUIData *tui);
extern void rs_tui_raw_line(TUIData *tui, int64_t g, int64_t linerow, int64_t startcol,
                             int64_t endcol, int64_t clearcol, int64_t clearattr,
                             int64_t flags, const schar_T *chunk, const sattr_T *attrs);

// Phase 1: Rust implementations for terminal mode / key encoding cluster
extern void rs_tui_request_term_mode(TUIData *tui, int mode);
extern void rs_tui_set_term_mode_impl(TUIData *tui, int mode, bool set);
extern void rs_tui_handle_term_mode(TUIData *tui, int mode, int state);
extern void rs_tui_query_extended_underline(TUIData *tui);
extern void rs_tui_query_kitty_keyboard(TUIData *tui);
extern void rs_tui_set_key_encoding_cb(TUIData *tui);
extern void rs_tui_reset_key_encoding_impl(TUIData *tui);

// Phase 2: Rust implementations for chdir, option_set, screenshot
extern void rs_tui_chdir(const char *path, size_t path_len);
extern void rs_tui_option_set(TUIData *tui, const char *name, size_t name_len,
                              int obj_type, int64_t int_val, bool bool_val);
extern void rs_tui_screenshot(TUIData *tui, const char *path, size_t path_len);

// Phase 3: Rust implementations for sigwinch and terminal_after_startup
extern void rs_tui_sigwinch_cb(SignalWatcher *watcher, int signum, void *cbdata);
extern void rs_tui_terminal_after_startup(TUIData *tui);

// Phase 5: Rust implementations for flush_buf_start and flush_buf_end
extern size_t rs_flush_buf_start(TUIData *tui, char *buf, size_t len);
extern size_t rs_flush_buf_end(TUIData *tui, char *buf, size_t len);
extern void rs_terminfo_out(TUIData *tui, int what);
extern void rs_terminfo_print_num(TUIData *tui, int what, int num1, int num2, int num3);

// TUIData Accessor Functions for Rust


/// Get cursor row position (target)
int nvim_tui_get_row(TUIData *tui) { return tui->row; }

/// Set cursor row position
void nvim_tui_set_row(TUIData *tui, int row) { tui->row = row; }

/// Get cursor col position (target)
int nvim_tui_get_col(TUIData *tui) { return tui->col; }

/// Set cursor col position
void nvim_tui_set_col(TUIData *tui, int col) { tui->col = col; }

/// Set an HlAttrs entry in the attrs kvec (resizes if needed)
void nvim_tui_set_attrs(TUIData *tui, size_t idx, HlAttrs attrs) { kv_a(tui->attrs, idx) = attrs; }

void nvim_tui_set_clear_attrs(TUIData *tui, HlAttrs attrs) { tui->clear_attrs = attrs; }


void nvim_tui_set_print_attr_id(TUIData *tui, int id) { tui->print_attr_id = id; }

void nvim_tui_set_default_colors_flag(TUIData *tui, bool value) { tui->set_default_colors = value; }

int nvim_tui_get_grid_height(TUIData *tui) { return tui->grid.height; }

int nvim_tui_get_grid_width(TUIData *tui) { return tui->grid.width; }

/// Invalidate a region (wrapper for invalidate function)
void nvim_tui_invalidate(TUIData *tui, int top, int bot, int left, int right)
{
  Rect r = { top, bot, left, right };
  kv_push(tui->invalid_regions, r);
}

bool nvim_tui_get_is_starting(TUIData *tui) { return tui->is_starting; }

/// Get pending_resize_events count
int nvim_tui_get_pending_resize_events(TUIData *tui) { return tui->pending_resize_events; }

/// Set pending_resize_events count
void nvim_tui_set_pending_resize_events(TUIData *tui, int val) { tui->pending_resize_events = val; }

/// Get the number of invalid regions
size_t nvim_tui_get_invalid_regions_size(TUIData *tui) { return kv_size(tui->invalid_regions); }

/// Clear all invalid regions
void nvim_tui_clear_invalid_regions(TUIData *tui) { kv_size(tui->invalid_regions) = 0; }

/// Clip an invalid region to grid bounds
void nvim_tui_clip_invalid_region(TUIData *tui, size_t idx, int max_height, int max_width)
{
  if (idx < kv_size(tui->invalid_regions)) {
    Rect *r = &kv_A(tui->invalid_regions, idx);
    r->bot = MIN(r->bot, max_height);
    r->right = MIN(r->right, max_width);
  }
}

/// Get an invalid region by index
void nvim_tui_get_invalid_region(TUIData *tui, size_t idx,
                                  int *top, int *bot, int *left, int *right)
{
  if (idx < kv_size(tui->invalid_regions)) {
    Rect r = kv_A(tui->invalid_regions, idx);
    *top = r.top;
    *bot = r.bot;
    *left = r.left;
    *right = r.right;
  }
}

/// Set an invalid region by index
void nvim_tui_set_invalid_region(TUIData *tui, size_t idx,
                                  int top, int bot, int left, int right)
{
  if (idx < kv_size(tui->invalid_regions)) {
    kv_A(tui->invalid_regions, idx) = (Rect) { top, bot, left, right };
  }
}

/// Push a new invalid region
void nvim_tui_push_invalid_region(TUIData *tui, int top, int bot, int left, int right)
{
  kv_push(tui->invalid_regions, ((Rect) { top, bot, left, right }));
}

/// Set grid col position
void nvim_tui_set_grid_col(TUIData *tui, int col) { tui->grid.col = col; }

/// Get pointer to UGrid
UGrid *nvim_tui_get_grid(TUIData *tui) { return &tui->grid; }

/// Set grid row to -1 (invalidate cursor position)
void nvim_tui_invalidate_grid_cursor(TUIData *tui) { tui->grid.row = -1; }

int nvim_tui_get_width(TUIData *tui) { return tui->width; }
void nvim_tui_set_width(TUIData *tui, int width) { tui->width = width; }

int nvim_tui_get_height(TUIData *tui) { return tui->height; }
void nvim_tui_set_height(TUIData *tui, int height) { tui->height = height; }

// Forward declaration for clear_region
static void clear_region(TUIData *tui, int top, int bot, int left, int right, int attr_id);

/// Wrapper for clear_region callable from Rust
void nvim_tui_clear_region(TUIData *tui, int top, int bot, int left, int right, int attr_id)
{
  clear_region(tui, top, bot, left, right, attr_id);
}

/// Output resize escape sequence (wrapper for out_printf for resize)
void nvim_tui_out_resize(TUIData *tui, int height, int width)
{
  // Forward declaration - out_printf is defined later in this file
  extern void out_printf(TUIData *tui, size_t limit, const char *fmt, ...);
  out_printf(tui, 64, "\x1b[8;%d;%dt", height, width);
}

// Terminfo Output Infrastructure for Rust

// Forward declarations for output functions defined later
static void out(TUIData *tui, const char *str, size_t len);
static void out_len(TUIData *tui, const char *str);
static void terminfo_print(TUIData *tui, TerminfoDef what, TPVAR *params);
static void cursor_goto(TUIData *tui, int row, int col);
static void invalidate(TUIData *tui, int top, int bot, int left, int right);

/// Wrapper for cursor_goto callable from Rust
void nvim_tui_cursor_goto_internal(TUIData *tui, int row, int col) { cursor_goto(tui, row, col); }

/// Wrapper for update_attrs callable from Rust
void nvim_tui_update_attrs_internal(TUIData *tui, int attr_id) { rs_update_attrs(tui, attr_id); }

/// Wrapper for invalidate callable from Rust
void nvim_tui_invalidate_region(TUIData *tui, int top, int bot, int left, int right)
{
  invalidate(tui, top, bot, left, right);
}

/// Wrapper for ugrid_scroll callable from Rust
void nvim_tui_ugrid_scroll(TUIData *tui, int top, int bot, int left, int right, int rows)
{
  ugrid_scroll(&tui->grid, top, bot, left, right, rows);
}

/// Write raw bytes to output buffer
void nvim_tui_out(TUIData *tui, const char *str, size_t len) { rs_out(tui, str, len); }

/// Output a terminfo escape sequence (calls Rust rs_terminfo_out directly)
void nvim_tui_terminfo_out(TUIData *tui, int what) { rs_terminfo_out(tui, what); }

/// Output a terminfo escape sequence with 1 parameter
void nvim_tui_terminfo_print_num1(TUIData *tui, int what, int num1)
{
  rs_terminfo_print_num(tui, what, num1, 0, 0);
}

/// Output a terminfo escape sequence with 2 parameters
void nvim_tui_terminfo_print_num2(TUIData *tui, int what, int num1, int num2)
{
  rs_terminfo_print_num(tui, what, num1, num2, 0);
}

/// Get grid row position
int nvim_tui_get_grid_row(TUIData *tui) { return tui->grid.row; }

/// Get grid col position
int nvim_tui_get_grid_col(TUIData *tui) { return tui->grid.col; }

/// Increment grid col position
void nvim_tui_inc_grid_col(TUIData *tui) { tui->grid.col++; }

/// Get url index
int nvim_tui_get_url(TUIData *tui) { return tui->url; }

/// Set url index
void nvim_tui_set_url(TUIData *tui, int url) { tui->url = url; }

int nvim_tui_get_print_attr_id(TUIData *tui) { return tui->print_attr_id; }

/// Get immediate_wrap_after_last_column flag
bool nvim_tui_get_immediate_wrap(TUIData *tui) { return tui->immediate_wrap_after_last_column; }

/// Wrapper for update_attrs callable from Rust
void nvim_tui_update_attrs(TUIData *tui, int attr_id) { rs_update_attrs(tui, attr_id); }

bool nvim_tui_get_can_clear_attr(TUIData *tui) { return tui->can_clear_attr; }

bool nvim_tui_get_can_erase_chars(TUIData *tui) { return tui->can_erase_chars; }

bool nvim_tui_get_set_default_colors(TUIData *tui) { return tui->set_default_colors; }

bool nvim_tui_get_default_attr(TUIData *tui) { return tui->default_attr; }


void nvim_tui_set_busy(TUIData *tui, bool busy) { tui->busy = busy; }
bool nvim_tui_get_busy(TUIData *tui) { return tui->busy; }

bool nvim_tui_get_mouse_enabled(TUIData *tui) { return tui->mouse_enabled; }

void nvim_tui_set_mouse_enabled(TUIData *tui, bool enabled) { tui->mouse_enabled = enabled; }

bool nvim_tui_get_mouse_move_enabled(TUIData *tui) { return tui->mouse_move_enabled; }

bool nvim_tui_get_screen_or_tmux(TUIData *tui) { return tui->screen_or_tmux; }

bool nvim_tui_get_can_scroll(TUIData *tui) { return tui->can_scroll; }

bool nvim_tui_get_can_change_scroll_region(TUIData *tui) { return tui->can_change_scroll_region; }

/// Get has_left_and_right_margin_mode flag
bool nvim_tui_get_has_lr_margin_mode(TUIData *tui) { return tui->has_left_and_right_margin_mode; }

bool nvim_tui_get_can_set_lr_margin(TUIData *tui) { return tui->can_set_lr_margin; }

bool nvim_tui_get_stopped(TUIData *tui) { return tui->stopped; }

bool nvim_tui_get_can_set_title(TUIData *tui) { return tui->can_set_title; }

bool nvim_tui_get_title_enabled(TUIData *tui) { return tui->title_enabled; }

void nvim_tui_set_title_enabled(TUIData *tui, bool enabled) { tui->title_enabled = enabled; }

/// Get available buffer space for title
size_t nvim_tui_get_buf_space(TUIData *tui) { return sizeof(tui->buf) - tui->bufpos; }

// Buffer management accessors for Rust
size_t nvim_tui_get_bufpos(TUIData *tui) { return tui->bufpos; }
void nvim_tui_set_bufpos(TUIData *tui, size_t pos) { tui->bufpos = pos; }
char *nvim_tui_get_buf_ptr(TUIData *tui) { return tui->buf; }
size_t nvim_tui_get_buf_capacity(void) { return OUTBUF_SIZE; }
void nvim_tui_set_buf_to_flush(TUIData *tui, char *ptr) { tui->buf_to_flush = ptr; }

// Terminfo defs accessor for Rust
const char *nvim_tui_get_ti_def(TUIData *tui, int idx) { return tui->ti.defs[idx]; }

// Phase 3: Core rendering accessors
bool nvim_tui_get_rgb(TUIData *tui) { return tui->rgb; }
bool nvim_tui_get_bce(TUIData *tui) { return tui->bce; }
bool nvim_tui_get_can_set_underline_color(TUIData *tui) { return tui->can_set_underline_color; }
HlAttrs nvim_tui_get_clear_attrs(TUIData *tui) { return tui->clear_attrs; }
void nvim_tui_set_default_attr(TUIData *tui, bool val) { tui->default_attr = val; }
void nvim_tui_set_can_clear_attr(TUIData *tui, bool val) { tui->can_clear_attr = val; }
HlAttrs nvim_tui_get_attrs_entry(TUIData *tui, size_t idx) { return kv_A(tui->attrs, idx); }
size_t nvim_tui_get_attrs_size(TUIData *tui) { return kv_size(tui->attrs); }
const HlAttrs *nvim_tui_get_attrs_ptr(TUIData *tui) { return tui->attrs.items; }
bool nvim_tui_ti_has_def(TUIData *tui, int idx) { return tui->ti.defs[idx] != NULL; }
schar_T nvim_tui_get_cell_data(TUIData *tui, int row, int col) { return tui->grid.cells[row][col].data; }
sattr_T nvim_tui_get_cell_attr(TUIData *tui, int row, int col) { return tui->grid.cells[row][col].attr; }

void nvim_tui_ui_client_set_size(TUIData *tui, int width, int height) { ui_client_set_size(width, height); }
void nvim_tui_inc_pending_resize_events(TUIData *tui) { tui->pending_resize_events++; }

/// Pop the last invalid region (like kv_pop). Returns false if empty.
bool nvim_tui_pop_invalid_region(TUIData *tui, int *top, int *bot, int *left, int *right)
{
  if (kv_size(tui->invalid_regions) == 0) {
    return false;
  }
  Rect r = kv_pop(tui->invalid_regions);
  *top = r.top; *bot = r.bot; *left = r.left; *right = r.right;
  return true;
}

/// Purge loop events
void nvim_tui_loop_purge(TUIData *tui) { loop_purge(tui->loop); }

/// Returns true if loop is flooded and should be purged
bool nvim_tui_loop_flooded(TUIData *tui) { return loop_size(tui->loop) > TOO_MANY_EVENTS; }

/// Set grid cell data and attr from chunk arrays (for tui_raw_line)
void nvim_tui_set_grid_cell(TUIData *tui, int row, int col, schar_T data, sattr_T attr)
{
  tui->grid.cells[row][col].data = data;
  tui->grid.cells[row][col].attr = attr;
}

/// Find the clear_col: last non-trailing-space column in [left, right)
/// Returns the column where trailing spaces with clear_attr start
int nvim_tui_find_clear_col(TUIData *tui, int row, int left, int right, sattr_T clear_attr)
{
  int clear_col;
  for (clear_col = right; clear_col > left; clear_col--) {
    UCell *cell = &tui->grid.cells[row][clear_col - 1];
    if (!(cell->data == schar_from_ascii(' ') && cell->attr == clear_attr)) {
      break;
    }
  }
  return clear_col;
}

/// Check if cell at (row, col+1) has NUL data (for double-width detection)
bool nvim_tui_next_cell_is_nul(TUIData *tui, int row, int col)
{
  return tui->grid.cells[row][col + 1].data == NUL;
}

/// ugrid_clear_chunk wrapper
void nvim_tui_ugrid_clear_chunk(TUIData *tui, int row, int col, int endcol, sattr_T attr)
{
  ugrid_clear_chunk(&tui->grid, row, col, endcol, attr);
}

/// Output 3-param terminfo sequence
void nvim_tui_terminfo_print_num3(TUIData *tui, int what, int n1, int n2, int n3)
{
  rs_terminfo_print_num(tui, what, n1, n2, n3);
}

/// Output kTerm_set_attributes with 9 params
void nvim_tui_terminfo_print_attrs(TUIData *tui, int standout, int underline, int reverse,
                                    int blink, int dim, int bold, int blank, int protect, int acs)
{
  TPVAR params[9] = { 0 };
  params[0].num = standout;
  params[1].num = underline;
  params[2].num = reverse;
  params[3].num = blink;
  params[4].num = dim;
  params[5].num = bold;
  params[6].num = blank;
  params[7].num = protect;
  params[8].num = acs;
  terminfo_print(tui, kTerm_set_attributes, params);
}

/// Output altfont mode sequence
void nvim_tui_out_altfont(TUIData *tui)
{
  out_len(tui, tui->terminfo_ext.enter_altfont_mode);
}

void nvim_tui_set_can_set_underline_color(TUIData *tui, bool val) { tui->can_set_underline_color = val; }

/// Wrapper for terminfo_set_if_empty (set underline style)
void nvim_tui_terminfo_set_underline_style(TUIData *tui)
{
  terminfo_set_if_empty(tui, kTerm_set_underline_style, "\x1b[4:%p1%dm");
}

// Forward declarations
static void flush_buf(TUIData *tui);
static bool should_invisible(TUIData *tui);

/// Wrapper for flush_buf callable from Rust
void nvim_tui_flush_buf(TUIData *tui) { flush_buf(tui); }

/// Wrapper for uv_sleep callable from Rust
void nvim_tui_uv_sleep(uint64_t ms) { uv_sleep(ms); }

/// Wrapper for tui_set_term_mode callable from Rust
void nvim_tui_set_term_mode(TUIData *tui, int mode, bool set) { tui_set_term_mode(tui, (TermMode)mode, set); }

// Phase 5 accessors for cursor mode and size detection

/// Get cursor_style_enabled (file-static)
bool nvim_tui_cursor_style_enabled(void) { return cursor_style_enabled; }

/// Get cursor shape entry id field
int nvim_tui_get_cursor_shape_id(TUIData *tui, int mode) { return tui->cursor_shapes[mode].id; }

/// Get cursor shape entry shape field (CursorShape enum as int)
int nvim_tui_get_cursor_shape_shape(TUIData *tui, int mode) { return (int)tui->cursor_shapes[mode].shape; }

/// Get cursor shape entry blinkon field
int nvim_tui_get_cursor_shape_blinkon(TUIData *tui, int mode) { return tui->cursor_shapes[mode].blinkon; }

/// Get cursor shape entry blinkoff field
int nvim_tui_get_cursor_shape_blinkoff(TUIData *tui, int mode) { return tui->cursor_shapes[mode].blinkoff; }

/// Get want_invisible field
bool nvim_tui_get_want_invisible(TUIData *tui) { return tui->want_invisible; }

/// Set want_invisible field
void nvim_tui_set_want_invisible(TUIData *tui, bool val) { tui->want_invisible = val; }

/// Get cursor_has_color field
bool nvim_tui_get_cursor_has_color(TUIData *tui) { return tui->cursor_has_color; }

/// Set cursor_has_color field
void nvim_tui_set_cursor_has_color(TUIData *tui, bool val) { tui->cursor_has_color = val; }

/// Get set_cursor_color_as_str field
bool nvim_tui_get_set_cursor_color_as_str(TUIData *tui) { return tui->set_cursor_color_as_str; }

/// Call terminfo_print with a single string parameter
void nvim_tui_terminfo_print_str(TUIData *tui, int what, const char *str)
{
  TPVAR params[9] = { 0 };
  params[0].string = (char *)str;
  terminfo_print(tui, (TerminfoDef)what, params);
}

/// Set showing_mode field
void nvim_tui_set_showing_mode(TUIData *tui, int mode) { tui->showing_mode = (ModeShape)mode; }

/// Set is_starting field
void nvim_tui_set_is_starting(TUIData *tui, bool val) { tui->is_starting = val; }

/// Get verbose field
int64_t nvim_tui_get_verbose(TUIData *tui) { return tui->verbose; }

/// Get out_isatty field
bool nvim_tui_get_out_isatty(TUIData *tui) { return tui->out_isatty; }

/// Get terminal window size via uv_tty_get_winsize. Returns true on success.
bool nvim_tui_uv_tty_get_winsize(TUIData *tui, int *width, int *height)
{
  return uv_tty_get_winsize(&tui->output_handle.tty, width, height) == 0;
}

/// Get terminfo lines field
int nvim_tui_get_ti_lines(TUIData *tui) { return tui->ti.lines; }

/// Get terminfo columns field
int nvim_tui_get_ti_columns(TUIData *tui) { return tui->ti.columns; }

/// Get stdin_isatty global
bool nvim_tui_stdin_isatty(void) { return stdin_isatty; }

/// Get modes.theme_updates bit
bool nvim_tui_get_mode_theme_updates(TUIData *tui) { return tui->modes.theme_updates; }

/// Get modes.resize_events bit
bool nvim_tui_get_mode_resize_events(TUIData *tui) { return tui->modes.resize_events; }

/// Get modes.grapheme_clusters bit
bool nvim_tui_get_mode_grapheme_clusters(TUIData *tui) { return tui->modes.grapheme_clusters; }

/// Get disable_focus_reporting string
const char *nvim_tui_get_disable_focus_reporting(TUIData *tui) { return tui->terminfo_ext.disable_focus_reporting; }

/// Get has_sync_mode flag
bool nvim_tui_get_has_sync_mode(TUIData *tui) { return tui->has_sync_mode; }

/// Set has_sync_mode flag
void nvim_tui_set_has_sync_mode(TUIData *tui, bool val) { tui->has_sync_mode = val; }

// Phase 1 accessors: mode flags and input callbacks
void nvim_tui_set_modes_grapheme_clusters(TUIData *tui, bool val) { tui->modes.grapheme_clusters = val; }
void nvim_tui_set_modes_theme_updates(TUIData *tui, bool val) { tui->modes.theme_updates = val; }
void nvim_tui_set_modes_resize_events(TUIData *tui, bool val) { tui->modes.resize_events = val; }
void nvim_tui_set_has_lr_margin_mode(TUIData *tui, bool val) { tui->has_left_and_right_margin_mode = val; }
void nvim_tui_set_resize_events_enabled(TUIData *tui, bool val) { tui->resize_events_enabled = val; }
bool nvim_tui_get_resize_events_enabled(TUIData *tui) { return tui->resize_events_enabled; }
void nvim_tui_set_primary_device_attr_cb(TUIData *tui, void (*cb)(TUIData *)) { tui->input.callbacks.primary_device_attr = cb; }
int nvim_tui_input_get_key_encoding(TUIData *tui) { return (int)tui->input.key_encoding; }

// Phase 3 accessors: focus reporting, sigwinch/startup callback helpers
const char *nvim_tui_get_enable_focus_reporting(TUIData *tui) { return tui->terminfo_ext.enable_focus_reporting; }

// Phase 5 accessors: flush_buf_start/end fields
bool nvim_tui_get_sync_output(TUIData *tui) { return tui->sync_output; }
bool nvim_tui_get_is_invisible(TUIData *tui) { return tui->is_invisible; }
void nvim_tui_set_is_invisible(TUIData *tui, bool val) { tui->is_invisible = val; }

/// Wrapper for tui_reset_key_encoding (static) callable from Rust
void nvim_tui_reset_key_encoding(TUIData *tui) { tui_reset_key_encoding(tui); }

// Phase 2 accessors: option_set and screenshot fields
void nvim_tui_set_mouse_move_enabled(TUIData *tui, bool val) { tui->mouse_move_enabled = val; }
void nvim_tui_set_rgb(TUIData *tui, bool val) { tui->rgb = val; }
void nvim_tui_set_verbose(TUIData *tui, int64_t val) { tui->verbose = val; }
void nvim_tui_set_sync_output(TUIData *tui, bool val) { tui->sync_output = val; }
void nvim_tui_set_input_ttimeout(TUIData *tui, bool val) { tui->input.ttimeout = val; }
void nvim_tui_set_input_ttimeoutlen(TUIData *tui, int64_t val) { tui->input.ttimeoutlen = (OptInt)val; }
void nvim_tui_set_screenshot(TUIData *tui, FILE *f) { tui->screenshot = f; }
FILE *nvim_tui_get_screenshot(TUIData *tui) { return tui->screenshot; }
void nvim_tui_set_grid_row_val(TUIData *tui, int row) { tui->grid.row = row; }
void nvim_tui_set_grid_col_val(TUIData *tui, int col) { tui->grid.col = col; }
/// Send termguicolors RGB mode change via RPC (wraps MAXSIZE_TEMP_ARRAY macros)
void nvim_tui_rpc_send_termguicolors(bool value)
{
  if (ui_client_channel_id) {
    MAXSIZE_TEMP_ARRAY(args, 2);
    ADD_C(args, CSTR_AS_OBJ("rgb"));
    ADD_C(args, BOOLEAN_OBJ(value));
    rpc_send_event(ui_client_channel_id, "nvim_ui_set_option", args);
  }
}

/// Wrapper to reset terminal title from Rust (NULL_STRING)
void nvim_tui_reset_title(TUIData *tui) { tui_set_title(tui, NULL_STRING); }

// Forward declarations for Phase 5c wrappers
static void show_verbose_terminfo(TUIData *tui);
static void tui_set_term_mode(TUIData *tui, TermMode mode, bool set);
static void tui_reset_key_encoding(TUIData *tui);

/// Dump verbose terminfo info to messages (called from Rust for tui_mode_change)
void nvim_tui_show_verbose_terminfo(TUIData *tui) { show_verbose_terminfo(tui); }

/// Reset TTY modes on UNIX when is_starting and not a TTY stdin
void nvim_tui_tty_reset_mode_hack(TUIData *tui)
{
#ifdef UNIX
  int ret = uv_tty_set_mode(&tui->output_handle.tty, UV_TTY_MODE_NORMAL);
  if (ret) {
    ELOG("uv_tty_set_mode failed: %s", uv_strerror(ret));
  }
  ret = uv_tty_set_mode(&tui->output_handle.tty, UV_TTY_MODE_IO);
  if (ret) {
    ELOG("uv_tty_set_mode failed: %s", uv_strerror(ret));
  }
#endif
}

/// Get reset_scroll_region string from terminfo_ext
const char *nvim_tui_get_reset_scroll_region(TUIData *tui) { return tui->terminfo_ext.reset_scroll_region; }

/// Wrapper for out_len callable from Rust
void nvim_tui_out_len(TUIData *tui, const char *str) { out_len(tui, str); }

#define TERMINFO_SEQ_LIMIT 128


static Set(cstr_t) urls = SET_INIT;

/// Get URL key string from global urls set
const char *nvim_tui_get_url_key(int idx) { return urls.keys[idx]; }

/// URL buffer operations
void nvim_tui_urlbuf_reset(TUIData *tui) { kv_size(tui->urlbuf) = 0; }
void nvim_tui_urlbuf_append_fmt(TUIData *tui, uint64_t id, const char *url)
{
  kv_printf(tui->urlbuf, "\x1b]8;id=%" PRIu64 ";%s\x1b\\", id, url);
}
const char *nvim_tui_urlbuf_ptr(TUIData *tui) { return tui->urlbuf.items; }
size_t nvim_tui_urlbuf_size(TUIData *tui) { return kv_size(tui->urlbuf); }

/// Output RGB underline color sequence
void nvim_tui_out_underline_color(TUIData *tui, int r, int g, int b)
{
  extern void out_printf(TUIData *tui, size_t limit, const char *fmt, ...);
  out_printf(tui, 128, "\x1b[58:2::%d:%d:%dm", r, g, b);
}

void tui_start(TUIData **tui_p, int *width, int *height, char **term, bool *rgb)
  FUNC_ATTR_NONNULL_ALL
{
  TUIData *tui = xcalloc(1, sizeof(TUIData));
  tui->is_starting = true;
  tui->screenshot = NULL;
  tui->stopped = false;
  tui->loop = &main_loop;
  tui->url = -1;

  kv_init(tui->invalid_regions);
  kv_init(tui->urlbuf);
  signal_watcher_init(tui->loop, &tui->winch_handle, tui);
  signal_watcher_start(&tui->winch_handle, rs_tui_sigwinch_cb, SIGWINCH);

  // TODO(bfredl): zero hl is empty, send this explicitly?
  kv_push(tui->attrs, HLATTRS_INIT);

  tui->input.tk_ti_hook_fn = tui_tk_ti_getstr;
  ugrid_init(&tui->grid);
  tui_terminal_start(tui);

  uv_timer_init(&tui->loop->uv, &tui->startup_delay_timer);
  tui->startup_delay_timer.data = tui;
  uv_timer_start(&tui->startup_delay_timer, after_startup_cb, 100, 0);

  *tui_p = tui;
  loop_poll_events(&main_loop, 1);
  *width = tui->width;
  *height = tui->height;
  *term = tui->term;
  *rgb = tui->rgb;
}

/// Request the terminal's mode (DECRQM). Rust implementation.
static void tui_request_term_mode(TUIData *tui, TermMode mode)
  FUNC_ATTR_NONNULL_ALL
{
  rs_tui_request_term_mode(tui, (int)mode);
}

/// Set (DECSET) or reset (DECRST) a terminal mode. Rust implementation.
static void tui_set_term_mode(TUIData *tui, TermMode mode, bool set)
  FUNC_ATTR_NONNULL_ALL
{
  rs_tui_set_term_mode_impl(tui, (int)mode, set);
}

/// Handle a mode report (DECRPM) from the terminal. Rust implementation.
void tui_handle_term_mode(TUIData *tui, TermMode mode, TermModeState state)
  FUNC_ATTR_NONNULL_ALL
{
  rs_tui_handle_term_mode(tui, (int)mode, (int)state);
}

/// Query the terminal emulator to see if it supports extended underline. Rust implementation.
static void tui_query_extended_underline(TUIData *tui)
{
  rs_tui_query_extended_underline(tui);
}

/// Enable extended underline support. Rust implementation.
void tui_enable_extended_underline(TUIData *tui) { rs_tui_enable_extended_underline(tui); }

/// Query the terminal emulator to see if it supports Kitty's keyboard protocol.
/// Rust implementation.
static void tui_query_kitty_keyboard(TUIData *tui)
  FUNC_ATTR_NONNULL_ALL
{
  rs_tui_query_kitty_keyboard(tui);
}

/// Called when DA1 response is received; sets the appropriate key encoding.
/// Rust implementation.
void tui_set_key_encoding(TUIData *tui)
  FUNC_ATTR_NONNULL_ALL
{
  rs_tui_set_key_encoding_cb(tui);
}

/// Reset the active key encoding protocol. Rust implementation.
static void tui_reset_key_encoding(TUIData *tui)
  FUNC_ATTR_NONNULL_ALL
{
  rs_tui_reset_key_encoding_impl(tui);
}

/// Write the OSC 11 sequence to the terminal emulator to query the current
/// background color.
///
/// The response will be handled by the TermResponse autocommand created in
/// _defaults.lua.
/// Query the terminal background color. Rust implementation.
void tui_query_bg_color(TUIData *tui)
  FUNC_ATTR_NONNULL_ALL
{
  rs_tui_query_bg_color(tui);
}

/// Enable the alternate screen and emit other control sequences to start the TUI.
///
/// This is also called when the TUI is resumed after being suspended. We reinitialize all state
/// from terminfo just in case the controlling terminal has changed (#27177).
static void terminfo_start(TUIData *tui)
{
  tui->bufpos = 0;
  tui->default_attr = false;
  tui->can_clear_attr = false;
  tui->is_invisible = true;
  tui->want_invisible = false;
  tui->busy = false;
  tui->set_cursor_color_as_str = false;
  tui->cursor_has_color = false;
  tui->resize_events_enabled = false;
  tui->modes.grapheme_clusters = false;
  tui->modes.resize_events = false;
  tui->modes.theme_updates = false;
  tui->showing_mode = SHAPE_IDX_N;
  tui->terminfo_ext.enable_focus_reporting = NULL;
  tui->terminfo_ext.disable_focus_reporting = NULL;

  tui->out_fd = STDOUT_FILENO;
  tui->out_isatty = os_isatty(tui->out_fd);
  tui->input.tui_data = tui;

  tui->ti_arena = (Arena)ARENA_EMPTY;
  assert(tui->term == NULL);

  char *term = os_getenv("TERM");
#ifdef MSWIN
  const char *guessed_term = NULL;
  os_tty_guess_term(&guessed_term, tui->out_fd);
  if (term == NULL && guessed_term != NULL) {
    // TODO(bfredl): should be arena_strdup, make os_getenv ready for the BIG STAGE?
    term = xstrdup(guessed_term);
    os_setenv("TERM", guessed_term, 1);
  }
#endif

  // Set up terminfo.
  tui->terminfo_found_in_db = false;
  if (term) {
    if (terminfo_from_database(&tui->ti, term, &tui->ti_arena)) {
      tui->term = arena_strdup(&tui->ti_arena, term);
      tui->terminfo_found_in_db = true;
    }
  }

  if (!tui->terminfo_found_in_db) {
    const TerminfoEntry *new = rs_terminfo_from_builtin(term, &tui->term);
    // we will patch it below, so make a copy
    memcpy(&tui->ti, new, sizeof tui->ti);
  }

  // None of the following work over SSH; see :help TERM .
  char *colorterm = os_getenv("COLORTERM");
  char *termprg = os_getenv("TERM_PROGRAM");
  char *vte_version_env = os_getenv("VTE_VERSION");
  char *konsolev_env = os_getenv("KONSOLE_VERSION");
  char *term_program_version_env = os_getenv("TERM_PROGRAM_VERSION");

  int vtev = vte_version_env ? (int)strtol(vte_version_env, NULL, 10) : 0;
  bool iterm_env = termprg && strstr(termprg, "iTerm.app");
  bool nsterm = (termprg && strstr(termprg, "Apple_Terminal"))
                || terminfo_is_term_family(term, "nsterm");
  bool konsole = terminfo_is_term_family(term, "konsole")
                 || os_env_exists("KONSOLE_PROFILE_NAME", true)
                 || os_env_exists("KONSOLE_DBUS_SESSION", true);
  int konsolev = konsolev_env ? (int)strtol(konsolev_env, NULL, 10)
                              : (konsole ? 1 : 0);
  bool wezterm = strequal(termprg, "WezTerm");
  const char *weztermv = wezterm ? term_program_version_env : NULL;
  bool screen = terminfo_is_term_family(term, "screen");
  bool tmux = terminfo_is_term_family(term, "tmux") || os_env_exists("TMUX", true);
  tui->screen_or_tmux = screen || tmux;

  // truecolor support must be checked before patching/augmenting terminfo
  tui->rgb = rs_term_has_truecolor(colorterm, tui->ti.has_Tc_or_RGB ? 1 : 0,
                                    (const char **)tui->ti.defs) != 0;

  // Use Rust implementations for terminal detection
  char *xterm_version_env = os_getenv("XTERM_VERSION");
  TermDetectContext ctx = {
    .term = term,
    .colorterm = colorterm,
    .vte_version = vtev,
    .konsole_version = konsolev,
    .iterm_env = iterm_env ? 1 : 0,
    .nsterm = nsterm ? 1 : 0,
    .has_xterm_version = xterm_version_env ? 1 : 0,
    .tmux_env = tmux ? 1 : 0,
    .wezterm_version = weztermv,
  };
  TerminfoState state = {
    .bce = tui->ti.bce ? 1 : 0,
    .max_colors = tui->ti.max_colors,
    .has_tc_or_rgb = tui->ti.has_Tc_or_RGB ? 1 : 0,
    .has_su = tui->ti.Su ? 1 : 0,
    .defs = tui->ti.defs,
  };
  TermDetectOutput output = { 0 };

  rs_patch_terminfo_bugs(&ctx, &state);

  // Apply state changes back
  tui->ti.bce = state.bce != 0;
  tui->ti.max_colors = state.max_colors;

  rs_augment_terminfo(&ctx, &state, &output);

  // Apply output flags
  tui->can_resize_screen = output.can_resize_screen != 0;
  tui->can_set_title = output.can_set_title != 0;
  tui->terminfo_ext.reset_scroll_region = (char *)output.reset_scroll_region;
  tui->terminfo_ext.enable_focus_reporting = (char *)output.enable_focus_reporting;
  tui->terminfo_ext.disable_focus_reporting = (char *)output.disable_focus_reporting;
  tui->set_cursor_color_as_str = output.set_cursor_color_as_str != 0;
  tui->input.key_encoding = (KeyEncoding)output.key_encoding;
  if (output.enable_extended_underline) {
    tui_enable_extended_underline(tui);
  }
  // It should be pretty safe to always enable this
  tui->terminfo_ext.enter_altfont_mode = "\x1b[11m";
  xfree(xterm_version_env);

#define TI_HAS(name) (tui->ti.defs[name] != NULL)
  tui->can_change_scroll_region = TI_HAS(kTerm_change_scroll_region);
  // note: also gated by tui->has_left_and_right_margin_mode
  tui->can_set_lr_margin = TI_HAS(kTerm_set_lr_margin);
  tui->can_scroll =
    TI_HAS(kTerm_delete_line)
    && TI_HAS(kTerm_parm_delete_line)
    && TI_HAS(kTerm_insert_line)
    && TI_HAS(kTerm_parm_insert_line);
  tui->can_erase_chars = TI_HAS(kTerm_erase_chars);
  tui->immediate_wrap_after_last_column =
    terminfo_is_term_family(term, "conemu")
    || terminfo_is_term_family(term, "cygwin")
    || terminfo_is_term_family(term, "win32con")
    || terminfo_is_term_family(term, "interix");
  tui->bce = tui->ti.bce;
  // Set 't_Co' from the result of terminfo & fix_terminfo.
  t_colors = tui->ti.max_colors;
  // Enter alternate screen, save title, and clear.
  // NOTE: Do this *before* changing terminal settings. #6433
  nvim_tui_terminfo_out(tui, kTerm_enter_ca_mode);
  nvim_tui_terminfo_out(tui, kTerm_keypad_xmit);
  nvim_tui_terminfo_out(tui, kTerm_clear_screen);

  /// Terminals usually ignore unrecognized private modes, and there is no
  /// known ambiguity with these. So we just set them unconditionally.
  // Enable bracketed paste
  tui_set_term_mode(tui, kTermModeBracketedPaste, true);

  tui->has_left_and_right_margin_mode = false;
  tui->has_sync_mode = false;

  // Query support for private DEC modes that Nvim can take advantage of.
  // Some terminals (such as Terminal.app) do not support DECRQM, so skip the query.
  if (!nsterm) {
    tui_request_term_mode(tui, kTermModeLeftAndRightMargins);
    tui_request_term_mode(tui, kTermModeSynchronizedOutput);
    tui_request_term_mode(tui, kTermModeGraphemeClusters);
    tui_request_term_mode(tui, kTermModeThemeUpdates);
    tui_request_term_mode(tui, kTermModeResizeEvents);
  }

  // Don't use DECRQSS in screen or tmux, as they behave strangely when receiving it.
  // Terminal.app also doesn't support DECRQSS.
  if (!TI_HAS(kTerm_set_underline_style) && !(screen || tmux || nsterm)) {
    // Query the terminal to see if it supports extended underline.
    tui_query_extended_underline(tui);
  }

  // Query the terminal to see if it supports Kitty's keyboard protocol
  tui_query_kitty_keyboard(tui);

  int ret;
  uv_loop_init(&tui->write_loop);
  if (tui->out_isatty) {
    ret = uv_tty_init(&tui->write_loop, &tui->output_handle.tty, tui->out_fd, 0);
    if (ret) {
      ELOG("uv_tty_init failed: %s", uv_strerror(ret));
    }
#ifndef MSWIN
    int retry_count = 10;
    // A signal may cause uv_tty_set_mode() to fail (e.g., SIGCONT). Retry a
    // few times. #12322
    while ((ret = uv_tty_set_mode(&tui->output_handle.tty, UV_TTY_MODE_IO)) == UV_EINTR
           && retry_count > 0) {
      retry_count--;
    }
    if (ret) {
      ELOG("uv_tty_set_mode failed: %s", uv_strerror(ret));
    }
#endif
  } else {
    ret = uv_pipe_init(&tui->write_loop, &tui->output_handle.pipe, 0);
    if (ret) {
      ELOG("uv_pipe_init failed: %s", uv_strerror(ret));
    }
    ret = uv_pipe_open(&tui->output_handle.pipe, tui->out_fd);
    if (ret) {
      ELOG("uv_pipe_open failed: %s", uv_strerror(ret));
    }
  }
  flush_buf(tui);

  xfree(term);
  xfree(colorterm);
  xfree(termprg);
  xfree(vte_version_env);
  xfree(konsolev_env);
  xfree(term_program_version_env);
#undef TI_HAS
}

/// Disable various terminal modes and other features.
/// Disable terminal modes and flush. Rust implementation.
static void terminfo_disable(TUIData *tui)
{
  rs_terminfo_disable(tui);
}

/// Disable the alternate screen and prepare for the TUI to close.
static void terminfo_stop(TUIData *tui)
{
  if (ui_client_exit_status == 0 && ui_client_error_exit > 0) {
    ui_client_exit_status = ui_client_error_exit;
  }

  // If Nvim exited with nonzero status, without indicating this was an
  // intentional exit (like `:1cquit`), it likely was an internal failure.
  // Don't clobber the stderr error message in this case. #21608
  if (ui_client_exit_status == MAX(ui_client_error_exit, 0)) {
    // Position the cursor on the last screen line, below all the text
    cursor_goto(tui, tui->height - 1, 0);
    // Exit alternate screen.
    nvim_tui_terminfo_out(tui, kTerm_exit_ca_mode);
  }

  flush_buf(tui);
  uv_tty_reset_mode();
  uv_close((uv_handle_t *)&tui->output_handle, NULL);
  uv_run(&tui->write_loop, UV_RUN_DEFAULT);
  if (uv_loop_close(&tui->write_loop)) {
    abort();
  }
  arena_mem_free(arena_finish(&tui->ti_arena));
  // Avoid using freed memory.
  memset(&tui->ti, 0, sizeof(tui->ti));
  tui->term = NULL;
}

static void tui_terminal_start(TUIData *tui)
{
  tui->print_attr_id = -1;
  terminfo_start(tui);
  if (tui->input.loop == NULL) {
    tinput_init(&tui->input, &main_loop, &tui->ti);
  }
  tui_guess_size(tui);
  tinput_start(&tui->input);
}

static void after_startup_cb(uv_timer_t *handle)
{
  TUIData *tui = handle->data;
  tui_terminal_after_startup(tui);
}

/// Emit focus reporting and flush after startup. Rust implementation.
static void tui_terminal_after_startup(TUIData *tui)
  FUNC_ATTR_NONNULL_ALL
{
  rs_tui_terminal_after_startup(tui);
}

void tui_stop(TUIData *tui)
  FUNC_ATTR_NONNULL_ALL
{
  if (uv_is_closing((uv_handle_t *)&tui->output_handle)) {
    // Race between SIGCONT (tui.c) and SIGHUP (os/signal.c)? #8075
    ELOG("TUI already stopped (race?)");
    tui->stopped = true;
    return;
  }

  tui->input.callbacks.primary_device_attr = tui_stop_cb;
  terminfo_disable(tui);

  // Wait until DA1 response is received, or stdin is closed (#35744).
  LOOP_PROCESS_EVENTS_UNTIL(tui->loop, tui->loop->events, EXIT_TIMEOUT_MS,
                            tui->stopped || rstream_did_eof(&tui->input.read_stream));
  if (!tui->stopped && !rstream_did_eof(&tui->input.read_stream)) {
    WLOG("TUI: timed out waiting for DA1 response");
  }
  tui->stopped = true;

  tui_terminal_stop(tui);
  stream_set_blocking(tui->input.in_fd, true);   // normalize stream (#2598)
  tinput_destroy(&tui->input);
  signal_watcher_stop(&tui->winch_handle);
  signal_watcher_close(&tui->winch_handle, NULL);
  uv_close((uv_handle_t *)&tui->startup_delay_timer, NULL);
}

/// Callback function called when the response to the Device Attributes (DA1)
/// request is sent during shutdown.
static void tui_stop_cb(TUIData *tui)
  FUNC_ATTR_NONNULL_ALL
{
  tui->stopped = true;
}

/// Stop the terminal but allow it to restart later (like after suspend)
///
/// This is called after we receive the response to the DA1 request sent from
/// terminfo_disable.
static void tui_terminal_stop(TUIData *tui)
  FUNC_ATTR_NONNULL_ALL
{
  tinput_stop(&tui->input);
  terminfo_stop(tui);
}

#ifdef EXITFREE
void tui_free_all_mem(TUIData *tui)
{
  ugrid_free(&tui->grid);
  kv_destroy(tui->invalid_regions);

  const char *url;
  set_foreach(&urls, url, {
    xfree((void *)url);
  });
  set_destroy(cstr_t, &urls);

  kv_destroy(tui->attrs);
  kv_destroy(tui->urlbuf);
  xfree(tui);
}
#endif

/// SIGWINCH handler. Rust implementation.
static void sigwinch_cb(SignalWatcher *watcher, int signum, void *cbdata)
{
  rs_tui_sigwinch_cb(watcher, signum, cbdata);
}

/// Optimized cursor positioning. Rust implementation.
static void cursor_goto(TUIData *tui, int row, int col)
{
  rs_cursor_goto(tui, row, col);
}

/// Clear a rectangular region. Rust implementation.
static void clear_region(TUIData *tui, int top, int bot, int left, int right, int attr_id)
{
  rs_clear_region(tui, top, bot, left, right, attr_id);
}

/// Resize the TUI grid. Rust implementation in nvim-tui crate.
void tui_grid_resize(TUIData *tui, Integer g, Integer width, Integer height)
{
  rs_tui_grid_resize(tui, g, width, height);
}

/// Clear the TUI grid. Rust implementation in nvim-tui crate.
void tui_grid_clear(TUIData *tui, Integer g) { rs_tui_grid_clear(tui, g); }

/// Set cursor position for the grid. Rust implementation in nvim-tui crate.
void tui_grid_cursor_goto(TUIData *tui, Integer grid, Integer row, Integer col)
{
  rs_tui_grid_cursor_goto(tui, row, col);
}

static CursorShape tui_cursor_decode_shape(const char *shape_str)
{
  CursorShape shape;
  if (strequal(shape_str, "block")) {
    shape = SHAPE_BLOCK;
  } else if (strequal(shape_str, "vertical")) {
    shape = SHAPE_VER;
  } else if (strequal(shape_str, "horizontal")) {
    shape = SHAPE_HOR;
  } else {
    WLOG("Unknown shape value '%s'", shape_str);
    shape = SHAPE_BLOCK;
  }
  return shape;
}

static cursorentry_T decode_cursor_entry(Dict args)
{
  cursorentry_T r = shape_table[0];

  for (size_t i = 0; i < args.size; i++) {
    char *key = args.items[i].key.data;
    Object value = args.items[i].value;

    if (strequal(key, "cursor_shape")) {
      r.shape = tui_cursor_decode_shape(args.items[i].value.data.string.data);
    } else if (strequal(key, "blinkon")) {
      r.blinkon = (int)value.data.integer;
    } else if (strequal(key, "blinkoff")) {
      r.blinkoff = (int)value.data.integer;
    } else if (strequal(key, "attr_id")) {
      r.id = (int)value.data.integer;
    }
  }
  return r;
}

void tui_mode_info_set(TUIData *tui, bool guicursor_enabled, Array args)
{
  cursor_style_enabled = guicursor_enabled;
  if (!guicursor_enabled) {
    return;  // Do not send cursor style control codes.
  }

  assert(args.size);

  // cursor style entries as defined by `shape_table`.
  for (size_t i = 0; i < args.size; i++) {
    assert(args.items[i].type == kObjectTypeDict);
    cursorentry_T r = decode_cursor_entry(args.items[i].data.dict);
    tui->cursor_shapes[i] = r;
  }

  tui_set_mode(tui, tui->showing_mode);
}

/// Update menu (stub - menus are for GUI only). Rust implementation.
void tui_update_menu(TUIData *tui) { rs_tui_update_menu(tui); }

/// Mark TUI as busy. Rust implementation.
void tui_busy_start(TUIData *tui) { rs_tui_busy_start(tui); }

/// Mark TUI as not busy. Rust implementation.
void tui_busy_stop(TUIData *tui) { rs_tui_busy_stop(tui); }

/// Enable mouse tracking. Rust implementation.
void tui_mouse_on(TUIData *tui) { rs_tui_mouse_on(tui); }

/// Disable mouse tracking. Rust implementation.
void tui_mouse_off(TUIData *tui) { rs_tui_mouse_off(tui); }

/// Set cursor mode based on mode index. Rust implementation.
static void tui_set_mode(TUIData *tui, ModeShape mode)
{
  rs_tui_set_mode(tui, (int)mode);
}

/// @param mode editor mode
/// Handle mode change events. Rust implementation.
void tui_mode_change(TUIData *tui, String mode, Integer mode_idx)
{
  rs_tui_mode_change(tui, (int64_t)mode_idx);
}

/// Scroll a region of the grid. Rust implementation.
void tui_grid_scroll(TUIData *tui, Integer g, Integer startrow, Integer endrow, Integer startcol,
                     Integer endcol, Integer rows, Integer cols FUNC_ATTR_UNUSED)
{
  rs_tui_grid_scroll(tui, g, startrow, endrow, startcol, endcol, rows, cols);
}

/// Add a URL to be used in an OSC 8 hyperlink.
///
/// @param tui TUIData
/// @param url URL to add
/// @return Index of new URL, or -1 if URL is invalid
int32_t tui_add_url(TUIData *tui, const char *url)
  FUNC_ATTR_NONNULL_ARG(1)
{
  if (url == NULL) {
    return -1;
  }

  MHPutStatus status;
  uint32_t k = set_put_idx(cstr_t, &urls, url, &status);
  if (status != kMHExisting) {
    urls.keys[k] = xstrdup(url);
  }
  return (int32_t)k;
}

/// Store highlight attributes. Rust implementation in nvim-tui crate.
void tui_hl_attr_define(TUIData *tui, Integer id, HlAttrs attrs, HlAttrs cterm_attrs, Array info)
{
  rs_tui_hl_attr_define(tui, id, attrs, cterm_attrs);
}

/// Ring the terminal bell. Rust implementation.
void tui_bell(TUIData *tui) { rs_tui_bell(tui); }

/// Visual bell - inverts screen briefly or uses screen/tmux bell. Rust implementation.
void tui_visual_bell(TUIData *tui) { rs_tui_visual_bell(tui); }

/// Set default colors and invalidate entire grid. Rust implementation in nvim-tui crate.
void tui_default_colors_set(TUIData *tui, Integer rgb_fg, Integer rgb_bg, Integer rgb_sp,
                            Integer cterm_fg, Integer cterm_bg)
{
  rs_tui_default_colors_set(tui, rgb_fg, rgb_bg, rgb_sp, cterm_fg, cterm_bg);
}

/// Writes directly to the TTY, bypassing the buffer.
void tui_ui_send(TUIData *tui, String content)
  FUNC_ATTR_NONNULL_ALL
{
  uv_write_t req;
  uv_buf_t buf = { .base = content.data, .len = UV_BUF_LEN(content.size) };
  int ret = uv_write(&req, (uv_stream_t *)&tui->output_handle, &buf, 1, NULL);
  if (ret) {
    ELOG("uv_write failed: %s", uv_strerror(ret));
  }
  uv_run(&tui->write_loop, UV_RUN_DEFAULT);
}

/// Flushes TUI grid state to a buffer (which is later flushed to the TTY by `flush_buf`).
///
/// @see flush_buf
/// Flush TUI grid to terminal. Rust implementation.
void tui_flush(TUIData *tui)
{
  rs_tui_flush(tui);
}

/// Dumps termcap info to the messages area, if 'verbose' >= 3.
static void show_verbose_terminfo(TUIData *tui)
{
  MAXSIZE_TEMP_ARRAY(chunks, 3);
  MAXSIZE_TEMP_ARRAY(title, 2);
  ADD_C(title, CSTR_AS_OBJ("\n\n--- Terminal info --- {{{\n"));
  ADD_C(title, CSTR_AS_OBJ("Title"));
  ADD_C(chunks, ARRAY_OBJ(title));
  MAXSIZE_TEMP_ARRAY(info, 1);
  String str = (String)rs_terminfo_info_msg(&tui->ti, tui->term, tui->terminfo_found_in_db);
  ADD_C(info, STRING_OBJ(str));
  ADD_C(chunks, ARRAY_OBJ(info));
  MAXSIZE_TEMP_ARRAY(end_fold, 2);
  ADD_C(end_fold, CSTR_AS_OBJ("}}}\n"));
  ADD_C(end_fold, CSTR_AS_OBJ("Title"));
  ADD_C(chunks, ARRAY_OBJ(end_fold));

  MAXSIZE_TEMP_ARRAY(args, 3);
  ADD_C(args, ARRAY_OBJ(chunks));
  ADD_C(args, BOOLEAN_OBJ(true));  // history
  MAXSIZE_TEMP_DICT(opts, 1);
  PUT_C(opts, "verbose", BOOLEAN_OBJ(true));
  ADD_C(args, DICT_OBJ(opts));
  rpc_send_event(ui_client_channel_id, "nvim_echo", args);
  xfree(str.data);
}

void tui_suspend(TUIData *tui)
{
// on a non-UNIX system, this is a no-op
#ifdef UNIX
  ui_client_detach();
  tui->mouse_enabled_save = tui->mouse_enabled;
  tui->input.callbacks.primary_device_attr = tui_suspend_cb;
  terminfo_disable(tui);
#endif
}

#ifdef UNIX
static void tui_suspend_cb(TUIData *tui)
  FUNC_ATTR_NONNULL_ALL
{
  tui_terminal_stop(tui);
  stream_set_blocking(tui->input.in_fd, true);   // normalize stream (#2598)

  // Avoid os/signal.c SIGTSTP handler. ex_stop calls auto_writeall. #33258
  kill(0, SIGSTOP);

  tui_terminal_start(tui);
  tui_terminal_after_startup(tui);
  if (tui->mouse_enabled_save) {
    tui_mouse_on(tui);
  }
  stream_set_blocking(tui->input.in_fd, false);  // libuv expects this
  ui_client_attach(tui->width, tui->height, tui->term, tui->rgb);
}
#endif

/// Set terminal title. Rust implementation.
void tui_set_title(TUIData *tui, String title) { rs_tui_set_title(tui, title.data, title.size); }

/// Set icon (stub - not implemented). Rust implementation.
void tui_set_icon(TUIData *tui, String icon) { rs_tui_set_icon(tui); }

/// Capture a screenshot of the TUI grid to a file. Rust implementation.
void tui_screenshot(TUIData *tui, String path)
{
  rs_tui_screenshot(tui, path.data, path.size);
}

/// Handle option changes from the server. Rust implementation.
void tui_option_set(TUIData *tui, String name, Object value)
{
  rs_tui_option_set(tui, name.data, name.size, (int)value.type,
                    value.data.integer, value.data.boolean);
}

/// Change working directory. Rust implementation.
void tui_chdir(TUIData *tui, String path)
{
  rs_tui_chdir(path.data, path.size);
}

/// Render a raw line. Rust implementation.
void tui_raw_line(TUIData *tui, Integer g, Integer linerow, Integer startcol, Integer endcol,
                  Integer clearcol, Integer clearattr, LineFlags flags, const schar_T *chunk,
                  const sattr_T *attrs)
{
  rs_tui_raw_line(tui, g, linerow, startcol, endcol, clearcol, clearattr,
                  (int64_t)flags, chunk, attrs);
}

/// Merge invalidation rect into existing invalid regions. Rust implementation.
static void invalidate(TUIData *tui, int top, int bot, int left, int right)
{
  rs_invalidate(tui, top, bot, left, right);
}

/// Set terminal size. Rust implementation.
void tui_set_size(TUIData *tui, int width, int height)
  FUNC_ATTR_NONNULL_ALL
{
  rs_tui_set_size(tui, width, height);
}

/// Tries to get the user's wanted dimensions (columns and rows) for the entire
/// application (i.e., the host terminal). Rust implementation.
void tui_guess_size(TUIData *tui)
{
  rs_tui_guess_size(tui);
}

static void out(TUIData *tui, const char *str, size_t len) { rs_out(tui, str, len); }

static void out_len(TUIData *tui, const char *str) { rs_out_len(tui, str); }

/// drops the entire message if it doesn't fit in "limit"
void out_printf(TUIData *tui, size_t limit, const char *fmt, ...)
  FUNC_ATTR_PRINTF(3, 4)
{
  assert(limit <= sizeof(tui->buf));
  size_t available = sizeof(tui->buf) - tui->bufpos;
  if (available < limit) {
    flush_buf(tui);
  }

  va_list ap;
  va_start(ap, fmt);
  int printed = vsnprintf(tui->buf + tui->bufpos, limit, fmt, ap);
  va_end(ap);

  if (printed > 0) {
    tui->bufpos += (size_t)printed;
  }
}

static void terminfo_print(TUIData *tui, TerminfoDef what, TPVAR *params)
{
  if (what >= kTermCount) {
    abort();
  }

  const char *str = tui->ti.defs[what];
  if (str == NULL || *str == NUL) {
    return;
  }

  if (sizeof(tui->buf) - tui->bufpos > TERMINFO_SEQ_LIMIT) {
    TPVAR copy_params[9];
    memcpy(copy_params, params, sizeof copy_params);
    size_t len = rs_terminfo_fmt(tui->buf + tui->bufpos, tui->buf + sizeof(tui->buf), str,
                              copy_params);
    if (len > 0) {
      tui->bufpos += len;
      return;
    }
  }

  // try again with fresh buffer
  flush_buf(tui);
  size_t len = rs_terminfo_fmt(tui->buf + tui->bufpos, tui->buf + sizeof(tui->buf), str, params);
  if (len > 0) {
    tui->bufpos += len;
  }
}
static void terminfo_set_if_empty(TUIData *tui, TerminfoDef str, const char *val)
{
  if (!tui->ti.defs[str]) {
    tui->ti.defs[str] = val;
  }
}

static void terminfo_set_str(TUIData *tui, TerminfoDef str, const char *val) { tui->ti.defs[str] = val; }


/// Compute flush prefix (sync start / hide cursor). Rust implementation.
static size_t flush_buf_start(TUIData *tui, char *buf, size_t len)
{
  return rs_flush_buf_start(tui, buf, len);
}

/// Compute flush suffix (sync end / show/hide cursor). Rust implementation.
static size_t flush_buf_end(TUIData *tui, char *buf, size_t len)
{
  return rs_flush_buf_end(tui, buf, len);
}

static void flush_buf(TUIData *tui)
{
  uv_write_t req;
  uv_buf_t bufs[3];
  char pre[32];
  char post[32];

  if (tui->bufpos <= 0 && tui->is_invisible == (tui->busy || tui->want_invisible)) {
    return;
  }

  bufs[0].base = pre;
  bufs[0].len = UV_BUF_LEN(flush_buf_start(tui, pre, sizeof(pre)));

  bufs[1].base = tui->buf_to_flush != NULL ? tui->buf_to_flush : tui->buf;
  bufs[1].len = UV_BUF_LEN(tui->bufpos);

  bufs[2].base = post;
  bufs[2].len = UV_BUF_LEN(flush_buf_end(tui, post, sizeof(post)));

  if (tui->screenshot) {
    for (size_t i = 0; i < ARRAY_SIZE(bufs); i++) {
      fwrite(bufs[i].base, bufs[i].len, 1, tui->screenshot);
    }
  } else {
    int ret
      = uv_write(&req, (uv_stream_t *)&tui->output_handle, bufs, ARRAY_SIZE(bufs), NULL);
    if (ret) {
      ELOG("uv_write failed: %s", uv_strerror(ret));
    }
    uv_run(&tui->write_loop, UV_RUN_DEFAULT);
  }
  tui->buf_to_flush = NULL;
  tui->bufpos = 0;
}

/// Try to get "kbs" code from stty because "the terminfo kbs entry is extremely
/// unreliable." (Vim, Bash, and tmux also do this.)
///
/// @see tmux/tty-keys.c fe4e9470bb504357d073320f5d305b22663ee3fd
/// @see https://bugzilla.redhat.com/show_bug.cgi?id=142659
static const char *tui_get_stty_erase(int fd)
{
  static char stty_erase[2] = { 0 };
#if defined(HAVE_TERMIOS_H)
  struct termios t;
  if (tcgetattr(fd, &t) != -1) {
    stty_erase[0] = (char)t.c_cc[VERASE];
    stty_erase[1] = NUL;
    DLOG("stty/termios:erase=%s", stty_erase);
  }
#endif
  return stty_erase;
}

/// libtermkey hook to override terminfo entries.
/// @see TermInput.tk_ti_hook_fn
static const char *tui_tk_ti_getstr(const char *name, const char *value, void *data)
{
  TermInput *input = data;
  static const char *stty_erase = NULL;
  if (stty_erase == NULL) {
    stty_erase = tui_get_stty_erase(input->in_fd);
  }

  if (strequal(name, "key_backspace")) {
    DLOG("libtermkey:kbs=%s", value);
    if (stty_erase[0] != 0) {
      return stty_erase;
    }
  } else if (strequal(name, "key_dc")) {
    DLOG("libtermkey:kdch1=%s", value);
    // Vim: "If <BS> and <DEL> are now the same, redefine <DEL>."
    if (value != NULL && value != (char *)-1 && strequal(stty_erase, value)) {
      return stty_erase[0] == DEL ? CTRL_H_STR : DEL_STR;
    }
  } else if (strequal(name, "key_mouse")) {
    DLOG("libtermkey:kmous=%s", value);
    // If key_mouse is found, libtermkey uses its terminfo driver (driver-ti.c)
    // for mouse input, which by accident only supports X10 protocol.
    // Force libtermkey to fallback to its CSI driver (driver-csi.c). #7948
    return NULL;
  }

  return value;
}
