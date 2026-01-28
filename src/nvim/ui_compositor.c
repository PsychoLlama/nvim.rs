// Compositor: merge floating grids with the main grid for display in
// TUI and non-multigrid UIs.
//
// Layer-based compositing: https://en.wikipedia.org/wiki/Digital_compositing

#include <assert.h>
#include <inttypes.h>
#include <limits.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>
#include <uv.h>

#include "klib/kvec.h"
#include "nvim/api/private/defs.h"
#include "nvim/ascii_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/highlight.h"
#include "nvim/highlight_defs.h"
#include "nvim/highlight_group.h"
#include "nvim/log.h"
#include "nvim/macros_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/option_vars.h"
#include "nvim/os/time.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_compositor.h"

#include "ui_compositor.c.generated.h"

static int composed_uis = 0;
kvec_t(ScreenGrid *) layers = KV_INITIAL_VALUE;

static size_t bufsize = 0;
static schar_T *linebuf;
static sattr_T *attrbuf;

#ifndef NDEBUG
static int chk_width = 0, chk_height = 0;
#endif

static ScreenGrid *curgrid;

static bool valid_screen = true;
static int msg_current_row = INT_MAX;
static bool msg_was_scrolled = false;

static int msg_sep_row = -1;
static schar_T msg_sep_char = schar_from_ascii(' ');

static int dbghl_normal, dbghl_clear, dbghl_composed, dbghl_recompose;

void ui_comp_init(void)
{
  kv_push(layers, &default_grid);
  curgrid = &default_grid;
}

#ifdef EXITFREE
void ui_comp_free_all_mem(void)
{
  kv_destroy(layers);
  xfree(linebuf);
  xfree(attrbuf);
}
#endif

void ui_comp_syn_init(void)
{
  dbghl_normal = syn_check_group(S_LEN("RedrawDebugNormal"));
  dbghl_clear = syn_check_group(S_LEN("RedrawDebugClear"));
  dbghl_composed = syn_check_group(S_LEN("RedrawDebugComposed"));
  dbghl_recompose = syn_check_group(S_LEN("RedrawDebugRecompose"));
}

void ui_comp_attach(RemoteUI *ui)
{
  composed_uis++;
  ui->composed = true;
}

void ui_comp_detach(RemoteUI *ui)
{
  composed_uis--;
  if (composed_uis == 0) {
    XFREE_CLEAR(linebuf);
    XFREE_CLEAR(attrbuf);
    bufsize = 0;
  }
  ui->composed = false;
}

extern int rs_ui_comp_should_draw(void);

/// C accessor for composed_uis static.
int nvim_get_composed_uis(void)
{
  return composed_uis;
}

/// C accessor for valid_screen static.
int nvim_get_valid_screen(void)
{
  return valid_screen;
}

/// C setter for valid_screen static.
void nvim_set_valid_screen(bool valid)
{
  valid_screen = valid;
}

/// C accessor for 'writedelay' option.
int64_t nvim_get_p_wd(void)
{
  return p_wd;
}

// Layer stack accessors for Rust compositor crate
size_t nvim_layers_size(void)
{
  return kv_size(layers);
}

ScreenGrid *nvim_layers_get(size_t i)
{
  if (i < kv_size(layers)) {
    return kv_A(layers, i);
  }
  return NULL;
}

void nvim_layers_set(size_t i, ScreenGrid *grid)
{
  if (i < kv_size(layers)) {
    kv_A(layers, i) = grid;
  }
}

void nvim_layers_push(ScreenGrid *grid)
{
  kv_push(layers, grid);
}

void nvim_layers_insert(size_t i, ScreenGrid *grid)
{
  if (i <= kv_size(layers)) {
    kv_push(layers, NULL);  // Make space
    // Shift elements right
    for (size_t j = kv_size(layers) - 1; j > i; j--) {
      kv_A(layers, j) = kv_A(layers, j - 1);
    }
    kv_A(layers, i) = grid;
  }
}

void nvim_layers_remove(size_t i)
{
  if (i < kv_size(layers)) {
    // Shift elements left
    for (size_t j = i; j < kv_size(layers) - 1; j++) {
      kv_A(layers, j) = kv_A(layers, j + 1);
    }
    kv_pop(layers);
  }
}

void nvim_layers_pop(void)
{
  if (kv_size(layers) > 0) {
    kv_pop(layers);
  }
}

// Compositor buffer accessors
schar_T *nvim_comp_get_linebuf_char(void)
{
  return linebuf;
}

sattr_T *nvim_comp_get_linebuf_attr(void)
{
  return attrbuf;
}

size_t nvim_comp_get_linebuf_size(void)
{
  return bufsize;
}

// Compositor state accessors
ScreenGrid *nvim_get_curgrid(void)
{
  return curgrid;
}

void nvim_set_curgrid(ScreenGrid *grid)
{
  curgrid = grid;
}

int nvim_get_msg_sep_row(void)
{
  return msg_sep_row;
}

void nvim_set_msg_sep_row(int row)
{
  msg_sep_row = row;
}

schar_T nvim_get_msg_sep_char(void)
{
  return msg_sep_char;
}

void nvim_set_msg_sep_char(schar_T c)
{
  msg_sep_char = c;
}

int nvim_get_msg_current_row(void)
{
  return msg_current_row;
}

void nvim_set_msg_current_row(int row)
{
  msg_current_row = row;
}

bool nvim_get_msg_was_scrolled(void)
{
  return msg_was_scrolled;
}

void nvim_set_msg_was_scrolled(bool scrolled)
{
  msg_was_scrolled = scrolled;
}

ScreenGrid *nvim_get_msg_grid(void)
{
  return &msg_grid;
}

/// Check if curgrid is the default grid
bool nvim_curgrid_is_default(void)
{
  return curgrid == &default_grid;
}

// Grid position-based accessors for compositor (different from grid.c array accessors)

/// Get character at grid position (compositor)
schar_T nvim_comp_grid_get_char_at(ScreenGrid *grid, int row, int col)
{
  if (!grid || row < 0 || col < 0 || row >= grid->rows || col >= grid->cols) {
    return 0;
  }
  return grid->chars[grid->line_offset[row] + (size_t)col];
}

/// Get attribute at grid position (compositor)
sattr_T nvim_comp_grid_get_attr_at(ScreenGrid *grid, int row, int col)
{
  if (!grid || row < 0 || col < 0 || row >= grid->rows || col >= grid->cols) {
    return 0;
  }
  return grid->attrs[grid->line_offset[row] + (size_t)col];
}

/// Set character at grid position (compositor)
void nvim_comp_grid_set_char_at(ScreenGrid *grid, int row, int col, schar_T c)
{
  if (!grid || row < 0 || col < 0 || row >= grid->rows || col >= grid->cols) {
    return;
  }
  grid->chars[grid->line_offset[row] + (size_t)col] = c;
}

/// Set attribute at grid position (compositor)
void nvim_comp_grid_set_attr_at(ScreenGrid *grid, int row, int col, sattr_T a)
{
  if (!grid || row < 0 || col < 0 || row >= grid->rows || col >= grid->cols) {
    return;
  }
  grid->attrs[grid->line_offset[row] + (size_t)col] = a;
}


/// Wrapper for hl_blend_attrs
int nvim_hl_blend_attrs(int back_attr, int front_attr, bool *through)
{
  return hl_blend_attrs(back_attr, front_attr, through);
}

// Debug highlight accessors

/// Get dbghl_normal
int nvim_comp_get_dbghl_normal(void)
{
  return dbghl_normal;
}

/// Get dbghl_clear
int nvim_comp_get_dbghl_clear(void)
{
  return dbghl_clear;
}

/// Get dbghl_composed
int nvim_comp_get_dbghl_composed(void)
{
  return dbghl_composed;
}

/// Get dbghl_recompose
int nvim_comp_get_dbghl_recompose(void)
{
  return dbghl_recompose;
}

/// C wrapper for ui_composed_call_grid_cursor_goto (generated function)
void nvim_ui_composed_call_grid_cursor_goto(int grid_handle, int row, int col)
{
  ui_composed_call_grid_cursor_goto(grid_handle, row, col);
}

// Rust implementation of curgrid_covered_above
extern bool rs_curgrid_covered_above(int row);

bool ui_comp_should_draw(void)
{
  return rs_ui_comp_should_draw() != 0;
}

// Rust implementation of ui_comp_layers_adjust
extern void rs_ui_comp_layers_adjust(size_t layer_idx, bool raise);

/// Raises or lowers the layer, syncing comp_index with zindex.
///
/// This function adjusts the position of a layer in the layers array
/// based on its zindex, either raising or lowering it.
///
/// @param[in]  layer_idx  Index of the layer to be raised or lowered.
/// @param[in]  raise      Raise the layer if true, else lower it.
void ui_comp_layers_adjust(size_t layer_idx, bool raise)
{
  rs_ui_comp_layers_adjust(layer_idx, raise);
}

/// Places `grid` at (col,row) position with (width * height) size.
/// Adds `grid` as the top layer if it is a new layer.
///
/// TODO(bfredl): later on the compositor should just use win_float_pos events,
// Rust implementation of ui_comp_put_grid
extern bool rs_ui_comp_put_grid(ScreenGrid *grid, int row, int col, int height, int width,
                                bool valid, bool on_top);

/// though that will require slight event order adjustment: emit the win_pos
/// events in the beginning of update_screen(), rather than in ui_flush()
bool ui_comp_put_grid(ScreenGrid *grid, int row, int col, int height, int width, bool valid,
                      bool on_top)
{
  return rs_ui_comp_put_grid(grid, row, col, height, width, valid, on_top);
}

// Rust implementation of ui_comp_remove_grid
extern void rs_ui_comp_remove_grid(ScreenGrid *grid);

void ui_comp_remove_grid(ScreenGrid *grid)
{
  rs_ui_comp_remove_grid(grid);
}

// Rust implementation of ui_comp_set_grid
extern int rs_ui_comp_set_grid(handle_T handle);

bool ui_comp_set_grid(handle_T handle)
{
  return rs_ui_comp_set_grid(handle) != 0;
}

// Rust implementation of ui_comp_raise_grid
extern void rs_ui_comp_raise_grid(ScreenGrid *grid, size_t new_index);

void ui_comp_raise_grid(ScreenGrid *grid, size_t new_index)
{
  rs_ui_comp_raise_grid(grid, new_index);
}

// Rust implementation of ui_comp_grid_cursor_goto
extern void rs_ui_comp_grid_cursor_goto(Integer grid_handle, Integer r, Integer c);

void ui_comp_grid_cursor_goto(Integer grid_handle, Integer r, Integer c)
{
  rs_ui_comp_grid_cursor_goto(grid_handle, r, c);
}

// Rust implementation of ui_comp_mouse_focus
extern ScreenGrid *rs_ui_comp_mouse_focus(int row, int col);

ScreenGrid *ui_comp_mouse_focus(int row, int col)
{
  return rs_ui_comp_mouse_focus(row, col);
}

// Rust implementation of ui_comp_get_grid_at_coord
extern ScreenGrid *rs_ui_comp_get_grid_at_coord(int row, int col);

/// Compute which grid is on top at supplied screen coordinates
ScreenGrid *ui_comp_get_grid_at_coord(int row, int col)
{
  return rs_ui_comp_get_grid_at_coord(row, col);
}

// Rust implementation of compose_line
extern void rs_compose_line(Integer row, Integer startcol, Integer endcol, int flags);

/// Baseline implementation. This is always correct, but we can sometimes
/// do something more efficient (where efficiency means smaller deltas to
/// the downstream UI.)
static void compose_line(Integer row, Integer startcol, Integer endcol, LineFlags flags)
{
  rs_compose_line(row, startcol, endcol, flags);
}

// Rust implementations of compose_debug and debug_delay
extern void rs_compose_debug(Integer startrow, Integer endrow, Integer startcol, Integer endcol,
                             int syn_id, bool delay);
extern void rs_debug_delay(Integer lines);

static void compose_debug(Integer startrow, Integer endrow, Integer startcol, Integer endcol,
                          int syn_id, bool delay)
{
  rs_compose_debug(startrow, endrow, startcol, endcol, syn_id, delay);
}

static void debug_delay(Integer lines)
{
  rs_debug_delay(lines);
}

// Rust implementation of compose_area
extern void rs_compose_area(Integer startrow, Integer endrow, Integer startcol, Integer endcol);

static void compose_area(Integer startrow, Integer endrow, Integer startcol, Integer endcol)
{
  rs_compose_area(startrow, endrow, startcol, endcol);
}

// Non-static wrapper for compose_area callable from Rust
void nvim_compose_area(int startrow, int endrow, int startcol, int endcol)
{
  compose_area(startrow, endrow, startcol, endcol);
}

// Rust implementation of ui_comp_compose_grid
extern void rs_ui_comp_compose_grid(ScreenGrid *grid);

/// compose the area under the grid.
///
/// This is needed when some option affecting composition is changed,
/// such as 'pumblend' for popupmenu grid.
void ui_comp_compose_grid(ScreenGrid *grid)
{
  rs_ui_comp_compose_grid(grid);
}

// Rust implementation of ui_comp_raw_line
extern void rs_ui_comp_raw_line(Integer grid, Integer row, Integer startcol, Integer endcol,
                                Integer clearcol, Integer clearattr, int flags,
                                const schar_T *chunk, const sattr_T *attrs);

void ui_comp_raw_line(Integer grid, Integer row, Integer startcol, Integer endcol, Integer clearcol,
                      Integer clearattr, LineFlags flags, const schar_T *chunk,
                      const sattr_T *attrs)
{
  rs_ui_comp_raw_line(grid, row, startcol, endcol, clearcol, clearattr, flags, chunk, attrs);
}

/// The screen is invalid and will soon be cleared
// Rust implementation of ui_comp_set_screen_valid
extern bool rs_ui_comp_set_screen_valid(bool valid);

///
/// Don't redraw floats until screen is cleared
bool ui_comp_set_screen_valid(bool valid)
{
  return rs_ui_comp_set_screen_valid(valid);
}

void ui_comp_msg_set_pos(Integer grid, Integer row, Boolean scrolled, String sep_char,
                         Integer zindex, Integer compindex)
{
  msg_grid.pending_comp_index_update = true;
  msg_grid.comp_row = (int)row;
  if (scrolled && row > 0) {
    msg_sep_row = (int)row - 1;
    if (sep_char.data) {
      msg_sep_char = schar_from_buf(sep_char.data, sep_char.size);
    }
  } else {
    msg_sep_row = -1;
  }

  if (row > msg_current_row && ui_comp_should_draw()) {
    compose_area(MAX(msg_current_row - 1, 0), row, 0, default_grid.cols);
  } else if (row < msg_current_row && ui_comp_should_draw()
             && (msg_current_row < Rows || (scrolled && !msg_was_scrolled))) {
    int delta = msg_current_row - (int)row;
    if (msg_grid.blending) {
      int first_row = MAX((int)row - (scrolled ? 1 : 0), 0);
      compose_area(first_row, Rows - delta, 0, Columns);
    } else {
      // scroll separator together with message text
      int first_row = MAX((int)row - (msg_was_scrolled ? 1 : 0), 0);
      ui_composed_call_grid_scroll(1, first_row, Rows, 0, Columns, delta, 0);
      if (scrolled && !msg_was_scrolled && row > 0) {
        compose_area(row - 1, row, 0, Columns);
      }
    }
  }

  msg_current_row = (int)row;
  msg_was_scrolled = scrolled;
}

/// check if curgrid is covered on row or above
///
/// TODO(bfredl): currently this only handles message row
static bool curgrid_covered_above(int row)
{
  return rs_curgrid_covered_above(row);
}

void ui_comp_grid_scroll(Integer grid, Integer top, Integer bot, Integer left, Integer right,
                         Integer rows, Integer cols)
{
  if (!ui_comp_should_draw() || !ui_comp_set_grid((int)grid)) {
    return;
  }
  top += curgrid->comp_row;
  bot += curgrid->comp_row;
  left += curgrid->comp_col;
  right += curgrid->comp_col;
  bool covered = curgrid_covered_above((int)(bot - MAX(rows, 0)));

  if (covered || curgrid->blending) {
    // TODO(bfredl):
    // 1. check if rectangles actually overlap
    // 2. calculate subareas that can scroll.
    compose_debug(top, bot, left, right, dbghl_recompose, true);
    for (int r = (int)(top + MAX(-rows, 0)); r < bot - MAX(rows, 0); r++) {
      // TODO(bfredl): workaround for win_update() performing two scrolls in a
      // row, where the latter might scroll invalid space created by the first.
      // ideally win_update() should keep track of this itself and not scroll
      // the invalid space.
      if (curgrid->attrs[curgrid->line_offset[r - curgrid->comp_row]
                         + (size_t)left - (size_t)curgrid->comp_col] >= 0) {
        compose_line(r, left, right, 0);
      }
    }
  } else {
    ui_composed_call_grid_scroll(1, top, bot, left, right, rows, cols);
    if (rdb_flags & kOptRdbFlagCompositor) {
      debug_delay(2);
    }
  }
}

void ui_comp_grid_resize(Integer grid, Integer width, Integer height)
{
  if (grid == 1) {
    ui_composed_call_grid_resize(1, width, height);
#ifndef NDEBUG
    chk_width = (int)width;
    chk_height = (int)height;
#endif
    size_t new_bufsize = (size_t)width;
    if (bufsize != new_bufsize) {
      xfree(linebuf);
      xfree(attrbuf);
      linebuf = xmalloc(new_bufsize * sizeof(*linebuf));
      attrbuf = xmalloc(new_bufsize * sizeof(*attrbuf));
      bufsize = new_bufsize;
    }
  }
}
