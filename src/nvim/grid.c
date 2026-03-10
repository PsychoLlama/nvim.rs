// Low-level functions to manipulate individual character cells on the
// screen grid.
//
// Most of the routines in this file perform screen (grid) manipulations. The
// given operation is performed physically on the screen. The corresponding
// change is also made to the internal screen image. In this way, the editor
// anticipates the effect of editing changes on the appearance of the screen.
// That way, when we call update_screen() a complete redraw isn't usually
// necessary. Another advantage is that we can keep adding code to anticipate
// screen changes, and in the meantime, everything still works.
//
// The grid_*() functions write to the screen and handle updating grid->lines[].

#include <assert.h>
#include <limits.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#include "nvim/api/private/defs.h"
#include "nvim/arabic.h"
#include "nvim/ascii_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/decoration.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/highlight.h"
#include "nvim/log.h"
#include "nvim/mbyte.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/option_vars.h"
#include "nvim/optionstr.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_defs.h"

#include "grid.c.generated.h"

// Grid helper functions (Phase 135) - still needed as rs_* in C
extern int rs_get_bordertext_col(int total_col, int text_width, int align);

// temporary buffer for rendering a single screenline, so it can be
// compared with previous contents to calculate smallest delta.
// Per-cell attributes
static size_t linebuf_size = 0;

/// C accessor for Rust to call check_chars_options()
/// @return 0 on success, non-zero on error
int nvim_check_chars_options(void)
{
  return check_chars_options() != NULL ? 1 : 0;
}

// =============================================================================
// Line buffer accessors (Phase 28)
// =============================================================================

/// Get pointer to linebuf_char array
schar_T *nvim_get_linebuf_char(void)
{
  return linebuf_char;
}

/// Get pointer to linebuf_attr array
sattr_T *nvim_get_linebuf_attr(void)
{
  return linebuf_attr;
}

/// Get pointer to linebuf_vcol array
colnr_T *nvim_get_linebuf_vcol(void)
{
  return linebuf_vcol;
}

/// Get pointer to linebuf_scratch buffer
char *nvim_get_linebuf_scratch(void)
{
  return linebuf_scratch;
}

/// Get the current linebuf size
size_t nvim_get_linebuf_size(void)
{
  return linebuf_size;
}

/// Wrapper for ui_grid_cursor_goto
void nvim_ui_grid_cursor_goto(handle_T grid_handle, int row, int col)
{
  ui_grid_cursor_goto(grid_handle, row, col);
}

/// Get handle from a ScreenGrid pointer
handle_T nvim_screengrid_get_handle(ScreenGrid *grid)
{
  return grid ? grid->handle : 0;
}

/// Get pointer to handle in a ScreenGrid (for Rust to assign)
handle_T *nvim_screengrid_get_handle_ptr(ScreenGrid *grid)
{
  return grid ? &grid->handle : NULL;
}

/// Get rows from a ScreenGrid pointer
int nvim_screengrid_get_rows(ScreenGrid *grid)
{
  return grid ? grid->rows : 0;
}

/// Get rdb_flags global
unsigned int nvim_get_rdb_flags(void)
{
  return rdb_flags;
}

// ScreenGrid array accessors
schar_T *nvim_screengrid_get_chars(ScreenGrid *grid)
{
  return grid ? grid->chars : NULL;
}

sattr_T *nvim_screengrid_get_attrs(ScreenGrid *grid)
{
  return grid ? grid->attrs : NULL;
}

colnr_T *nvim_screengrid_get_vcols(ScreenGrid *grid)
{
  return grid ? grid->vcols : NULL;
}

size_t *nvim_screengrid_get_line_offset(ScreenGrid *grid)
{
  return grid ? grid->line_offset : NULL;
}

int *nvim_screengrid_get_dirty_col(ScreenGrid *grid)
{
  return grid ? grid->dirty_col : NULL;
}

// ScreenGrid field accessors
int nvim_screengrid_get_cols(ScreenGrid *grid)
{
  return grid ? grid->cols : 0;
}

bool nvim_screengrid_get_throttled(ScreenGrid *grid)
{
  return grid ? grid->throttled : false;
}

// ScreenGrid compositor field accessors (for Rust compositor crate)
int nvim_screengrid_get_comp_row(ScreenGrid *grid)
{
  return grid ? grid->comp_row : 0;
}

void nvim_screengrid_set_comp_row(ScreenGrid *grid, int val)
{
  if (grid) {
    grid->comp_row = val;
  }
}

int nvim_screengrid_get_comp_col(ScreenGrid *grid)
{
  return grid ? grid->comp_col : 0;
}

void nvim_screengrid_set_comp_col(ScreenGrid *grid, int val)
{
  if (grid) {
    grid->comp_col = val;
  }
}

int nvim_screengrid_get_comp_width(ScreenGrid *grid)
{
  return grid ? grid->comp_width : 0;
}

void nvim_screengrid_set_comp_width(ScreenGrid *grid, int val)
{
  if (grid) {
    grid->comp_width = val;
  }
}

int nvim_screengrid_get_comp_height(ScreenGrid *grid)
{
  return grid ? grid->comp_height : 0;
}

void nvim_screengrid_set_comp_height(ScreenGrid *grid, int val)
{
  if (grid) {
    grid->comp_height = val;
  }
}

size_t nvim_screengrid_get_comp_index(ScreenGrid *grid)
{
  return grid ? grid->comp_index : 0;
}

void nvim_screengrid_set_comp_index(ScreenGrid *grid, size_t val)
{
  if (grid) {
    grid->comp_index = val;
  }
}

int nvim_screengrid_get_zindex(ScreenGrid *grid)
{
  return grid ? grid->zindex : 0;
}

bool nvim_screengrid_get_blending(ScreenGrid *grid)
{
  return grid ? grid->blending : false;
}

bool nvim_screengrid_get_comp_disabled(ScreenGrid *grid)
{
  return grid ? grid->comp_disabled : false;
}

void nvim_screengrid_set_comp_disabled(ScreenGrid *grid, bool val)
{
  if (grid) {
    grid->comp_disabled = val;
  }
}

bool nvim_screengrid_get_pending_comp_index_update(ScreenGrid *grid)
{
  return grid ? grid->pending_comp_index_update : false;
}

void nvim_screengrid_set_pending_comp_index_update(ScreenGrid *grid, bool val)
{
  if (grid) {
    grid->pending_comp_index_update = val;
  }
}

bool nvim_screengrid_get_mouse_enabled(ScreenGrid *grid)
{
  return grid ? grid->mouse_enabled : false;
}

void nvim_screengrid_set_zindex(ScreenGrid *grid, int val) { if (grid) { grid->zindex = val; } }
void nvim_screengrid_set_valid(ScreenGrid *grid, bool val) { if (grid) { grid->valid = val; } }
void nvim_screengrid_set_mouse_enabled(ScreenGrid *grid, bool val) { if (grid) { grid->mouse_enabled = val; } }

// Global accessors
ScreenGrid *nvim_get_default_grid(void)
{
  return &default_grid;
}

GridView *nvim_get_default_gridview(void)
{
  return &default_gridview;
}

bool nvim_get_exmode_active(void)
{
  return exmode_active;
}

int nvim_get_p_arshape(void)
{
  return p_arshape;
}

int nvim_get_p_tbidi(void)
{
  return p_tbidi;
}

bool nvim_get_full_screen(void)
{
  return full_screen;
}

int nvim_get_default_grid_has_chars(void)
{
  return default_grid.chars != NULL;
}

// GridView field accessors
ScreenGrid *nvim_gridview_get_target(GridView *view)
{
  return view->target;
}

int nvim_gridview_get_row_offset(GridView *view)
{
  return view->row_offset;
}

int nvim_gridview_get_col_offset(GridView *view)
{
  return view->col_offset;
}

void nvim_ui_line(ScreenGrid *grid, int row, bool invalid_row, int startcol, int endcol,
                  int clearcol, int clearattr, bool wrap)
{
  ui_line(grid, row, invalid_row, startcol, endcol, clearcol, clearattr, wrap);
}

void nvim_ui_call_grid_scroll(handle_T handle, int top, int bot, int left, int right, int rows,
                              int cols)
{
  ui_call_grid_scroll(handle, top, bot, left, right, rows, cols);
}



void grid_alloc(ScreenGrid *grid, int rows, int columns, bool copy, bool valid)
{
  int new_row;
  ScreenGrid ngrid = *grid;
  assert(rows >= 0 && columns >= 0);
  size_t ncells = (size_t)rows * (size_t)columns;
  ngrid.chars = xmalloc(ncells * sizeof(schar_T));
  ngrid.attrs = xmalloc(ncells * sizeof(sattr_T));
  ngrid.vcols = xmalloc(ncells * sizeof(colnr_T));
  memset(ngrid.vcols, -1, ncells * sizeof(colnr_T));
  ngrid.line_offset = xmalloc((size_t)rows * sizeof(*ngrid.line_offset));

  ngrid.rows = rows;
  ngrid.cols = columns;

  for (new_row = 0; new_row < ngrid.rows; new_row++) {
    ngrid.line_offset[new_row] = (size_t)new_row * (size_t)ngrid.cols;

    grid_clear_line(&ngrid, ngrid.line_offset[new_row], columns, valid);

    if (copy) {
      // If the screen is not going to be cleared, copy as much as
      // possible from the old screen to the new one and clear the rest
      // (used when resizing the window at the "--more--" prompt or when
      // executing an external command, for the GUI).
      if (new_row < grid->rows && grid->chars != NULL) {
        int len = MIN(grid->cols, ngrid.cols);
        memmove(ngrid.chars + ngrid.line_offset[new_row],
                grid->chars + grid->line_offset[new_row],
                (size_t)len * sizeof(schar_T));
        memmove(ngrid.attrs + ngrid.line_offset[new_row],
                grid->attrs + grid->line_offset[new_row],
                (size_t)len * sizeof(sattr_T));
        memmove(ngrid.vcols + ngrid.line_offset[new_row],
                grid->vcols + grid->line_offset[new_row],
                (size_t)len * sizeof(colnr_T));
      }
    }
  }
  grid_free(grid);
  *grid = ngrid;

  // Share a single scratch buffer for all grids, by
  // ensuring it is as wide as the widest grid.
  if (linebuf_size < (size_t)columns) {
    xfree(linebuf_char);
    xfree(linebuf_attr);
    xfree(linebuf_vcol);
    xfree(linebuf_scratch);
    linebuf_char = xmalloc((size_t)columns * sizeof(schar_T));
    linebuf_attr = xmalloc((size_t)columns * sizeof(sattr_T));
    linebuf_vcol = xmalloc((size_t)columns * sizeof(colnr_T));
    linebuf_scratch = xmalloc((size_t)columns * sizeof(sscratch_T));
    linebuf_size = (size_t)columns;
  }
}

void grid_free(ScreenGrid *grid)
{
  xfree(grid->chars);
  xfree(grid->attrs);
  xfree(grid->vcols);
  xfree(grid->line_offset);

  grid->chars = NULL;
  grid->attrs = NULL;
  grid->vcols = NULL;
  grid->line_offset = NULL;
}

#ifdef EXITFREE
/// Doesn't allow reinit, so must only be called by free_all_mem!
void grid_free_all_mem(void)
{
  grid_free(&default_grid);
  grid_free(&msg_grid);
  XFREE_CLEAR(msg_grid.dirty_col);
  xfree(linebuf_char);
  xfree(linebuf_attr);
  xfree(linebuf_vcol);
  xfree(linebuf_scratch);
}
#endif

/// (Re)allocates a window grid if size changed while in ext_multigrid mode.
/// Updates size, offsets and handle for the grid regardless.
///
/// If "doclear" is true, don't try to copy from the old grid rather clear the
/// resized grid.
void win_grid_alloc(win_T *wp)
{
  GridView *grid = &wp->w_grid;
  ScreenGrid *grid_allocated = &wp->w_grid_alloc;

  int total_rows = wp->w_height_outer;
  int total_cols = wp->w_width_outer;

  bool want_allocation = ui_has(kUIMultigrid) || wp->w_floating;
  bool has_allocation = (grid_allocated->chars != NULL);

  if (wp->w_view_height > wp->w_lines_size) {
    wp->w_lines_valid = 0;
    xfree(wp->w_lines);
    wp->w_lines = xcalloc((size_t)wp->w_view_height + 1, sizeof(wline_T));
    wp->w_lines_size = wp->w_view_height;
  }

  bool was_resized = false;
  if (want_allocation && (!has_allocation
                          || grid_allocated->rows != total_rows
                          || grid_allocated->cols != total_cols)) {
    grid_alloc(grid_allocated, total_rows, total_cols,
               wp->w_grid_alloc.valid, false);
    grid_allocated->valid = true;
    if (wp->w_floating && wp->w_config.border) {
      wp->w_redr_border = true;
    }
    was_resized = true;
  } else if (!want_allocation && has_allocation) {
    // Single grid mode, all rendering will be redirected to default_grid.
    // Only keep track of the size and offset of the window.
    grid_free(grid_allocated);
    grid_allocated->valid = false;
    was_resized = true;
  } else if (want_allocation && has_allocation && !wp->w_grid_alloc.valid) {
    grid_invalidate(grid_allocated);
    grid_allocated->valid = true;
  }

  if (want_allocation) {
    grid->target = grid_allocated;
    grid->row_offset = wp->w_winrow_off;
    grid->col_offset = wp->w_wincol_off;
  } else {
    grid->target = &default_grid;
    grid->row_offset = wp->w_winrow + wp->w_winrow_off;
    grid->col_offset = wp->w_wincol + wp->w_wincol_off;
  }

  // send grid resize event if:
  // - a grid was just resized
  // - screen_resize was called and all grid sizes must be sent
  // - the UI wants multigrid event (necessary)
  if ((resizing_screen || was_resized) && want_allocation) {
    ui_call_grid_resize(grid_allocated->handle,
                        grid_allocated->cols, grid_allocated->rows);
    ui_check_cursor_grid(grid_allocated->handle);
  }
}

static void grid_draw_bordertext(VirtText vt, int col, int winbl, const int *hl_attr,
                                 BorderTextType bt)
{
  for (size_t i = 0; i < kv_size(vt);) {
    int attr = -1;
    char *text = next_virt_text_chunk(vt, &i, &attr);
    if (text == NULL) {
      break;
    }
    if (attr == -1) {  // No highlight specified.
      attr = hl_attr[bt == kBorderTextTitle ? HLF_BTITLE : HLF_BFOOTER];
    }
    attr = hl_apply_winblend(winbl, attr);
    col += grid_line_puts(col, text, -1, attr);
  }
}

/// draw border on floating window grid
void grid_draw_border(ScreenGrid *grid, WinConfig *config, int *adj, int winbl, int *hl_attr)
{
  int *attrs = config->border_attr;
  int default_adj[4] = { 1, 1, 1, 1 };
  if (adj == NULL) {
    adj = default_adj;
  }
  schar_T chars[8];
  if (!hl_attr) {
    hl_attr = hl_attr_active;
  }

  for (int i = 0; i < 8; i++) {
    chars[i] = schar_from_str(config->border_chars[i]);
  }

  int irow = grid->rows - adj[0] - adj[2];
  int icol = grid->cols - adj[1] - adj[3];

  if (adj[0]) {
    screengrid_line_start(grid, 0, 0);
    if (adj[3]) {
      grid_line_put_schar(0, chars[0], attrs[0]);
    }

    for (int i = 0; i < icol; i++) {
      grid_line_put_schar(i + adj[3], chars[1], attrs[1]);
    }

    if (config->title) {
      int title_col = rs_get_bordertext_col(icol, config->title_width, (int)config->title_pos);
      grid_draw_bordertext(config->title_chunks, title_col, winbl, hl_attr, kBorderTextTitle);
    }
    if (adj[1]) {
      grid_line_put_schar(icol + adj[3], chars[2], attrs[2]);
    }
    grid_line_flush();
  }

  for (int i = 0; i < irow; i++) {
    if (adj[3]) {
      screengrid_line_start(grid, i + adj[0], 0);
      grid_line_put_schar(0, chars[7], attrs[7]);
      grid_line_flush();
    }
    if (adj[1]) {
      int ic = (i == 0 && !adj[0] && chars[2]) ? 2 : 3;
      screengrid_line_start(grid, i + adj[0], 0);
      grid_line_put_schar(icol + adj[3], chars[ic], attrs[ic]);
      grid_line_flush();
    }
  }

  if (adj[2]) {
    screengrid_line_start(grid, irow + adj[0], 0);
    if (adj[3]) {
      grid_line_put_schar(0, chars[6], attrs[6]);
    }

    for (int i = 0; i < icol; i++) {
      int ic = (i == 0 && !adj[3] && chars[6]) ? 6 : 5;
      grid_line_put_schar(i + adj[3], chars[ic], attrs[ic]);
    }

    if (config->footer) {
      int footer_col = rs_get_bordertext_col(icol, config->footer_width, (int)config->footer_pos);
      grid_draw_bordertext(config->footer_chunks, footer_col, winbl, hl_attr, kBorderTextFooter);
    }
    if (adj[1]) {
      grid_line_put_schar(icol + adj[3], chars[4], attrs[4]);
    }
    grid_line_flush();
  }
}

win_T *get_win_by_grid_handle(handle_T handle)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (wp->w_grid_alloc.handle == handle) {
      return wp;
    }
  }
  return NULL;
}

