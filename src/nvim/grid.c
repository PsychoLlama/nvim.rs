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
int nvim_check_chars_options(void) { return check_chars_options() != NULL ? 1 : 0; }

// Line buffer accessors (Phase 28)

/// Get pointer to linebuf_char array
schar_T *nvim_get_linebuf_char(void) { return linebuf_char; }

/// Get pointer to linebuf_attr array
sattr_T *nvim_get_linebuf_attr(void) { return linebuf_attr; }

/// Get pointer to linebuf_vcol array
colnr_T *nvim_get_linebuf_vcol(void) { return linebuf_vcol; }

/// Get pointer to linebuf_scratch buffer
char *nvim_get_linebuf_scratch(void) { return linebuf_scratch; }

/// Get the current linebuf size
size_t nvim_get_linebuf_size(void) { return linebuf_size; }

/// Update linebuf arrays if columns exceeds current linebuf_size (used by rs_grid_alloc).
void nvim_grid_alloc_update_linebuf(int columns)
{
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

void nvim_ui_grid_cursor_goto(handle_T grid_handle, int row, int col) { ui_grid_cursor_goto(grid_handle, row, col); }

/// Get handle from a ScreenGrid pointer
handle_T nvim_screengrid_get_handle(ScreenGrid *grid) { return grid ? grid->handle : 0; }

/// Get pointer to handle in a ScreenGrid (for Rust to assign)
handle_T *nvim_screengrid_get_handle_ptr(ScreenGrid *grid) { return grid ? &grid->handle : NULL; }

/// Get rows from a ScreenGrid pointer
int nvim_screengrid_get_rows(ScreenGrid *grid) { return grid ? grid->rows : 0; }

/// Get rdb_flags global
unsigned int nvim_get_rdb_flags(void) { return rdb_flags; }

// ScreenGrid array accessors
schar_T *nvim_screengrid_get_chars(ScreenGrid *grid) { return grid ? grid->chars : NULL; }

sattr_T *nvim_screengrid_get_attrs(ScreenGrid *grid) { return grid ? grid->attrs : NULL; }

colnr_T *nvim_screengrid_get_vcols(ScreenGrid *grid) { return grid ? grid->vcols : NULL; }

size_t *nvim_screengrid_get_line_offset(ScreenGrid *grid) { return grid ? grid->line_offset : NULL; }

int *nvim_screengrid_get_dirty_col(ScreenGrid *grid) { return grid ? grid->dirty_col : NULL; }

// ScreenGrid field setters (used by rs_grid_alloc)
void nvim_screengrid_set_chars(ScreenGrid *grid, schar_T *val) { if (grid) { grid->chars = val; } }
void nvim_screengrid_set_attrs(ScreenGrid *grid, sattr_T *val) { if (grid) { grid->attrs = val; } }
void nvim_screengrid_set_vcols(ScreenGrid *grid, colnr_T *val) { if (grid) { grid->vcols = val; } }
void nvim_screengrid_set_line_offset(ScreenGrid *grid, size_t *val) { if (grid) { grid->line_offset = val; } }
void nvim_screengrid_set_rows(ScreenGrid *grid, int val) { if (grid) { grid->rows = val; } }
void nvim_screengrid_set_cols(ScreenGrid *grid, int val) { if (grid) { grid->cols = val; } }

// ScreenGrid field accessors
int nvim_screengrid_get_cols(ScreenGrid *grid) { return grid ? grid->cols : 0; }

bool nvim_screengrid_get_throttled(ScreenGrid *grid) { return grid ? grid->throttled : false; }

// ScreenGrid compositor field accessors (for Rust compositor crate)
int nvim_screengrid_get_comp_row(ScreenGrid *grid) { return grid ? grid->comp_row : 0; }

void nvim_screengrid_set_comp_row(ScreenGrid *grid, int val)
{
  if (grid) {
    grid->comp_row = val;
  }
}

int nvim_screengrid_get_comp_col(ScreenGrid *grid) { return grid ? grid->comp_col : 0; }

void nvim_screengrid_set_comp_col(ScreenGrid *grid, int val)
{
  if (grid) {
    grid->comp_col = val;
  }
}

int nvim_screengrid_get_comp_width(ScreenGrid *grid) { return grid ? grid->comp_width : 0; }

void nvim_screengrid_set_comp_width(ScreenGrid *grid, int val)
{
  if (grid) {
    grid->comp_width = val;
  }
}

int nvim_screengrid_get_comp_height(ScreenGrid *grid) { return grid ? grid->comp_height : 0; }

void nvim_screengrid_set_comp_height(ScreenGrid *grid, int val)
{
  if (grid) {
    grid->comp_height = val;
  }
}

size_t nvim_screengrid_get_comp_index(ScreenGrid *grid) { return grid ? grid->comp_index : 0; }

void nvim_screengrid_set_comp_index(ScreenGrid *grid, size_t val)
{
  if (grid) {
    grid->comp_index = val;
  }
}

int nvim_screengrid_get_zindex(ScreenGrid *grid) { return grid ? grid->zindex : 0; }

bool nvim_screengrid_get_blending(ScreenGrid *grid) { return grid ? grid->blending : false; }

bool nvim_screengrid_get_comp_disabled(ScreenGrid *grid) { return grid ? grid->comp_disabled : false; }

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

bool nvim_screengrid_get_mouse_enabled(ScreenGrid *grid) { return grid ? grid->mouse_enabled : false; }

void nvim_screengrid_set_zindex(ScreenGrid *grid, int val) { if (grid) { grid->zindex = val; } }
void nvim_screengrid_set_valid(ScreenGrid *grid, bool val) { if (grid) { grid->valid = val; } }
bool nvim_screengrid_get_valid(ScreenGrid *grid) { return grid ? grid->valid : false; }
void nvim_screengrid_set_mouse_enabled(ScreenGrid *grid, bool val) { if (grid) { grid->mouse_enabled = val; } }

// Null-setters for grid arrays (used by rs_grid_free)
void nvim_screengrid_set_chars_null(ScreenGrid *grid) { if (grid) { grid->chars = NULL; } }
void nvim_screengrid_set_attrs_null(ScreenGrid *grid) { if (grid) { grid->attrs = NULL; } }
void nvim_screengrid_set_vcols_null(ScreenGrid *grid) { if (grid) { grid->vcols = NULL; } }
void nvim_screengrid_set_line_offset_null(ScreenGrid *grid) { if (grid) { grid->line_offset = NULL; } }

/// Find a window in curtab by its grid_alloc handle (used by rs_get_win_by_grid_handle).
win_T *nvim_find_win_by_grid_handle(handle_T handle)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (wp->w_grid_alloc.handle == handle) {
      return wp;
    }
  }
  return NULL;
}

// Rust implementations (Phase 1, 2 & 3)
extern void rs_grid_free(ScreenGrid *grid);
extern win_T *rs_get_win_by_grid_handle(handle_T handle);
extern void rs_grid_alloc(ScreenGrid *grid, int rows, int columns, bool copy, bool valid);
extern void rs_win_grid_alloc(win_T *wp);

// Phase 4: WinConfig border field accessors
int nvim_winconfig_get_border_attr(WinConfig *cfg, int idx) { return (cfg && idx >= 0 && idx < 8) ? cfg->border_attr[idx] : 0; }
const char *nvim_winconfig_get_border_char(WinConfig *cfg, int idx) { return (cfg && idx >= 0 && idx < 8) ? cfg->border_chars[idx] : NULL; }
int nvim_winconfig_get_title_flag(WinConfig *cfg) { return cfg ? (int)cfg->title : 0; }
int nvim_winconfig_get_title_pos(WinConfig *cfg) { return cfg ? (int)cfg->title_pos : 0; }
int nvim_winconfig_get_title_width(WinConfig *cfg) { return cfg ? cfg->title_width : 0; }
VirtText *nvim_winconfig_get_title_chunks(WinConfig *cfg) { return cfg ? &cfg->title_chunks : NULL; }
int nvim_winconfig_get_footer_flag(WinConfig *cfg) { return cfg ? (int)cfg->footer : 0; }
int nvim_winconfig_get_footer_pos(WinConfig *cfg) { return cfg ? (int)cfg->footer_pos : 0; }
int nvim_winconfig_get_footer_width(WinConfig *cfg) { return cfg ? cfg->footer_width : 0; }
VirtText *nvim_winconfig_get_footer_chunks(WinConfig *cfg) { return cfg ? &cfg->footer_chunks : NULL; }

// Phase 4: highlight attribute accessors
int nvim_hl_attr_active_get(int idx) { return (idx >= 0 && idx < (int)HLF_COUNT) ? hl_attr_active[idx] : 0; }
int nvim_hlf_btitle_val(void) { return (int)HLF_BTITLE; }
int nvim_hlf_bfooter_val(void) { return (int)HLF_BFOOTER; }
int nvim_hl_apply_winblend(int winbl, int attr) { return hl_apply_winblend(winbl, attr); }

size_t nvim_kv_size_virttext(VirtText *vt) { return vt ? kv_size(*vt) : 0; }
// nvim_next_virt_text_chunk removed from grid.c - conflicts with decoration.c version

// Phase 4: Rust implementation
extern void rs_grid_draw_border(ScreenGrid *grid, WinConfig *config, int *adj, int winbl, int *hl_attr);

// Global accessors
ScreenGrid *nvim_get_default_grid(void) { return &default_grid; }

GridView *nvim_get_default_gridview(void) { return &default_gridview; }

/// Get resizing_screen global (used by rs_win_grid_alloc)
bool nvim_get_resizing_screen(void) { return resizing_screen; }

/// Wrapper for ui_call_grid_resize (used by rs_win_grid_alloc)
void nvim_call_grid_resize(handle_T handle, int cols, int rows)
{
  ui_call_grid_resize(handle, cols, rows);
}

// GridView field setters (used by rs_win_grid_alloc)
void nvim_gridview_set_target(GridView *view, ScreenGrid *target)
{
  if (view) { view->target = target; }
}

void nvim_gridview_set_row_offset(GridView *view, int val)
{
  if (view) { view->row_offset = val; }
}

void nvim_gridview_set_col_offset(GridView *view, int val)
{
  if (view) { view->col_offset = val; }
}

int nvim_get_p_arshape(void) { return p_arshape; }

int nvim_get_p_tbidi(void) { return p_tbidi; }

bool nvim_get_full_screen(void) { return full_screen; }

int nvim_get_default_grid_has_chars(void) { return default_grid.chars != NULL; }

// GridView field accessors
ScreenGrid *nvim_gridview_get_target(GridView *view) { return view->target; }

int nvim_gridview_get_row_offset(GridView *view) { return view->row_offset; }

int nvim_gridview_get_col_offset(GridView *view) { return view->col_offset; }

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
  rs_grid_alloc(grid, rows, columns, copy, valid);
}

void grid_free(ScreenGrid *grid)
{
  rs_grid_free(grid);
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
  rs_win_grid_alloc(wp);
}

/// draw border on floating window grid
void grid_draw_border(ScreenGrid *grid, WinConfig *config, int *adj, int winbl, int *hl_attr)
{
  rs_grid_draw_border(grid, config, adj, winbl, hl_attr);
}

win_T *get_win_by_grid_handle(handle_T handle)
{
  return rs_get_win_by_grid_handle(handle);
}

