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

/// Resize the compositor line buffer.
///
/// This function reallocates linebuf and attrbuf if the new size differs.
void nvim_comp_resize_linebuf(size_t new_size)
{
  if (bufsize != new_size) {
    xfree(linebuf);
    xfree(attrbuf);
    linebuf = xmalloc(new_size * sizeof(*linebuf));
    attrbuf = xmalloc(new_size * sizeof(*attrbuf));
    bufsize = new_size;
  }
}

#ifndef NDEBUG
void nvim_comp_set_chk_dimensions(int width, int height)
{
  chk_width = width;
  chk_height = height;
}
#endif

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


// Forward declaration for Rust-exported compose_area
extern void compose_area(int startrow, int endrow, int startcol, int endcol);

// Non-static wrapper for compose_area callable from Rust
void nvim_compose_area(int startrow, int endrow, int startcol, int endcol)
{
  compose_area(startrow, endrow, startcol, endcol);
}

// Rust implementation of ui_comp_msg_set_pos
extern void rs_ui_comp_msg_set_pos(Integer grid, Integer row, Boolean scrolled,
                                   const char *sep_char_data, size_t sep_char_size,
                                   Integer zindex, Integer compindex);

void ui_comp_msg_set_pos(Integer grid, Integer row, Boolean scrolled, String sep_char,
                         Integer zindex, Integer compindex)
{
  rs_ui_comp_msg_set_pos(grid, row, scrolled, sep_char.data, sep_char.size, zindex, compindex);
}
