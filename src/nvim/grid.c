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
#include "nvim/map_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/option_vars.h"
#include "nvim/optionstr.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_defs.h"

#include "grid.c.generated.h"

// Rust implementations of schar functions
extern bool rs_schar_high(schar_T sc);
extern char rs_schar_get_ascii(schar_T sc);
extern schar_T rs_schar_from_char(int c);
extern schar_T rs_schar_from_str(const char *str);
extern schar_T rs_schar_from_buf(const char *buf, size_t len);
extern size_t rs_schar_get(char *buf_out, schar_T sc);
extern size_t rs_schar_get_adv(char **buf_out, schar_T sc);
extern size_t rs_schar_len(schar_T sc);
extern int rs_schar_cells(schar_T sc);
extern int rs_schar_get_first_codepoint(schar_T sc);
extern bool rs_schar_cache_clear_if_full(void);
extern void rs_schar_cache_clear(void);
// Rust implementations of grid_line functions
extern void rs_grid_line_put_schar(int col, schar_T schar, int attr);
extern int rs_grid_line_puts(int col, const char *text, int textlen, int attr);
extern int rs_grid_line_fill(int start_col, int end_col, schar_T sc, int attr);
extern void rs_grid_line_clear_end(int start_col, int end_col, int bg_attr, int clear_attr);
extern void rs_grid_line_cursor_goto(int col);
extern void rs_grid_line_flush(void);
extern void rs_grid_line_flush_if_valid_row(void);
extern void rs_grid_put_linebuf(ScreenGrid *grid, int row, int coloff, int col, int endcol,
                                int clear_width, int bg_attr, int clear_attr, colnr_T last_vcol,
                                int flags);
extern void rs_line_do_arabic_shape(schar_T *buf, int cols);
// Grid operations
extern ScreenGrid *rs_grid_adjust(GridView *view, int *row_off, int *col_off);
extern void rs_grid_clear_line(ScreenGrid *grid, size_t off, int width, bool valid);
extern void rs_grid_invalidate(ScreenGrid *grid);
extern schar_T rs_grid_getchar(ScreenGrid *grid, int row, int col, int *attrp);
extern void rs_grid_clear(GridView *grid, int start_row, int end_row, int start_col, int end_col,
                          int attr);
// Grid scrolling
extern void rs_grid_ins_lines(ScreenGrid *grid, int row, int line_count, int end, int col,
                              int width);
extern void rs_grid_del_lines(ScreenGrid *grid, int row, int line_count, int end, int col,
                              int width);
// Grid line start/getchar/mirror
extern void rs_screengrid_line_start(ScreenGrid *grid, int row, int col);
extern void rs_grid_line_start(GridView *view, int row);
extern schar_T rs_grid_line_getchar(int col, int *attr);
extern void rs_linebuf_mirror(int *firstp, int *lastp, int *clearp, int width);
extern void rs_grid_line_mirror(int width);
// Grid handle assignment and border text
extern void rs_grid_assign_handle(ScreenGrid *grid);
extern int rs_get_bordertext_col(int total_col, int text_width, int align);
// Grid helper functions (Phase 135)
extern schar_T rs_schar_from_ascii(int c);
extern int rs_slf_is_rightleft(int flags);
extern int rs_slf_is_wrap(int flags);
extern int rs_slf_is_inc_vcol(int flags);
extern int rs_slf_rightleft(void);
extern int rs_slf_wrap(void);
extern int rs_slf_inc_vcol(void);
extern int rs_slf_combine(int rightleft, int wrap, int inc_vcol);
extern size_t rs_grid_cell_count(int rows, int cols);
extern size_t rs_grid_line_offset(int row, int cols);
extern int rs_grid_row_valid(int row, int rows);
extern int rs_grid_col_valid(int col, int cols);
extern int rs_grid_pos_valid(int row, int col, int rows, int cols);
extern int rs_grid_clamp_col(int col, int cols);
extern int rs_grid_clamp_row(int row, int rows);
extern int rs_schar_is_nul(schar_T sc);
extern int rs_schar_is_space(schar_T sc);
extern schar_T rs_schar_space(void);
extern schar_T rs_schar_gt(void);
extern schar_T rs_schar_lt(void);
extern schar_T rs_schar_tilde(void);
extern schar_T rs_schar_at(void);
extern int rs_sattr_is_invalid(int attr);
extern int rs_sattr_invalid(void);
extern int rs_sattr_default(void);
extern int rs_colnr_invalid(void);
extern int rs_colnr_is_invalid(int col);
extern int rs_grid_copy_len(int old_cols, int new_cols);
extern int rs_grid_should_copy(int new_row, int old_rows, int old_chars_not_null);
extern int rs_grid_align_is_left(int align);
extern int rs_grid_align_is_center(int align);
extern int rs_grid_align_is_right(int align);
extern int rs_grid_align_left_val(void);
extern int rs_grid_align_center_val(void);
extern int rs_grid_align_right_val(void);
extern int rs_rdb_is_nodelta(unsigned int flags);
extern int rs_rdb_is_invalid(unsigned int flags);
extern unsigned int rs_rdb_flag_nodelta(void);
extern unsigned int rs_rdb_flag_invalid(void);

// temporary buffer for rendering a single screenline, so it can be
// compared with previous contents to calculate smallest delta.
// Per-cell attributes
static size_t linebuf_size = 0;

// Used to cache glyphs which doesn't fit an a sizeof(schar_T) length UTF-8 string.
// Then it instead stores an index into glyph_cache.keys[] which is a flat char array.
// The hash part is used by schar_from_buf() to quickly lookup glyphs which already
// has been interned. schar_get() should used to convert a schar_T value
// back to a string buffer.
//
// The maximum byte size of a glyph is MAX_SCHAR_SIZE (including the final NUL).
static Set(glyph) glyph_cache = SET_INIT;

/// C accessor for Rust to read from glyph cache
/// @param idx Index into glyph_cache.keys
/// @return Pointer to NUL-terminated string at idx, or NULL if out of bounds
const char *nvim_glyph_cache_get(uint32_t idx)
{
  if (idx >= glyph_cache.h.n_keys) {
    return NULL;
  }
  return &glyph_cache.keys[idx];
}

/// Get number of keys in glyph cache (for bounds checking)
uint32_t nvim_glyph_cache_n_keys(void)
{
  return glyph_cache.h.n_keys;
}

/// C accessor for Rust to call decor_check_invalid_glyphs()
void nvim_decor_check_invalid_glyphs(void)
{
  decor_check_invalid_glyphs();
}

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

// =============================================================================
// Grid line state accessors (Phase 29)
// =============================================================================

// Forward declarations for static grid line state variables (defined later)
static ScreenGrid *grid_line_grid;
static int grid_line_row;
static int grid_line_coloff;
static int grid_line_maxcol;
static int grid_line_first;
static int grid_line_last;
static int grid_line_clear_to;
static int grid_line_bg_attr;
static int grid_line_clear_attr;
static int grid_line_flags;

/// Get current grid_line_grid pointer
ScreenGrid *nvim_get_grid_line_grid(void)
{
  return grid_line_grid;
}

/// Set current grid_line_grid pointer
void nvim_set_grid_line_grid(ScreenGrid *grid)
{
  grid_line_grid = grid;
}

/// Get current grid_line_row
int nvim_get_grid_line_row(void)
{
  return grid_line_row;
}

/// Set current grid_line_row
void nvim_set_grid_line_row(int row)
{
  grid_line_row = row;
}

/// Get current grid_line_coloff
int nvim_get_grid_line_coloff(void)
{
  return grid_line_coloff;
}

/// Set current grid_line_coloff
void nvim_set_grid_line_coloff(int coloff)
{
  grid_line_coloff = coloff;
}

/// Get current grid_line_maxcol
int nvim_get_grid_line_maxcol(void)
{
  return grid_line_maxcol;
}

/// Set current grid_line_maxcol
void nvim_set_grid_line_maxcol(int maxcol)
{
  grid_line_maxcol = maxcol;
}

/// Get current grid_line_first
int nvim_get_grid_line_first(void)
{
  return grid_line_first;
}

/// Set current grid_line_first
void nvim_set_grid_line_first(int first)
{
  grid_line_first = first;
}

/// Get current grid_line_last
int nvim_get_grid_line_last(void)
{
  return grid_line_last;
}

/// Set current grid_line_last
void nvim_set_grid_line_last(int last)
{
  grid_line_last = last;
}

/// Get current grid_line_clear_to
int nvim_get_grid_line_clear_to(void)
{
  return grid_line_clear_to;
}

/// Set current grid_line_clear_to
void nvim_set_grid_line_clear_to(int clear_to)
{
  grid_line_clear_to = clear_to;
}

/// Get current grid_line_bg_attr
int nvim_get_grid_line_bg_attr(void)
{
  return grid_line_bg_attr;
}

/// Set current grid_line_bg_attr
void nvim_set_grid_line_bg_attr(int bg_attr)
{
  grid_line_bg_attr = bg_attr;
}

/// Get current grid_line_clear_attr
int nvim_get_grid_line_clear_attr(void)
{
  return grid_line_clear_attr;
}

/// Set current grid_line_clear_attr
void nvim_set_grid_line_clear_attr(int clear_attr)
{
  grid_line_clear_attr = clear_attr;
}

/// Get current grid_line_flags
int nvim_get_grid_line_flags(void)
{
  return grid_line_flags;
}

/// Set current grid_line_flags
void nvim_set_grid_line_flags(int flags)
{
  grid_line_flags = flags;
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

/// Wrapper for grid_put_linebuf (called from Rust)
void nvim_grid_put_linebuf(ScreenGrid *grid, int row, int coloff, int col, int endcol,
                           int clear_width, int bg_attr, int clear_attr, int last_vcol, int flags)
{
  grid_put_linebuf(grid, row, coloff, col, endcol, clear_width, bg_attr, clear_attr,
                   (colnr_T)last_vcol, flags);
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

// Function wrappers
void nvim_line_do_arabic_shape(schar_T *buf, int cols)
{
  line_do_arabic_shape(buf, cols);
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

/// Determine if dedicated window grid should be used or the default_grid
///
/// If UI did not request multigrid support, draw all windows on the
/// default_grid.
///
/// NB: this function can only been used with window grids in a context where
/// win_grid_alloc already has been called!
///
/// If the default_grid is used, adjust window relative positions to global
/// screen positions.
ScreenGrid *grid_adjust(GridView *grid, int *row_off, int *col_off)
{
  return rs_grid_adjust(grid, row_off, col_off);
}

schar_T schar_from_str(const char *str)
{
  return rs_schar_from_str(str);
}

/// @param buf need not be NUL terminated, but may not contain embedded NULs.
///
/// caller must ensure len < MAX_SCHAR_SIZE (not =, as NUL needs a byte)
schar_T schar_from_buf(const char *buf, size_t len)
{
  return rs_schar_from_buf(buf, len);
}

/// Check if cache is full, and if it is, clear it.
///
/// This should normally only be called in update_screen()
///
/// @return true if cache was clered, and all your screen buffers now are hosed
/// and you need to use UPD_CLEAR
bool schar_cache_clear_if_full(void)
{
  return rs_schar_cache_clear_if_full();
}

void schar_cache_clear(void)
{
  rs_schar_cache_clear();
}

bool schar_high(schar_T sc)
{
  return rs_schar_high(sc);
}

#ifdef ORDER_BIG_ENDIAN
# define schar_idx(sc) (sc & (0x00FFFFFF))
#else
# define schar_idx(sc) (sc >> 8)
#endif

/// sets final NUL
size_t schar_get(char *buf_out, schar_T sc)
{
  return rs_schar_get(buf_out, sc);
}

/// advance buf_out. do NOT set final NUL
size_t schar_get_adv(char **buf_out, schar_T sc)
{
  return rs_schar_get_adv(buf_out, sc);
}

size_t schar_len(schar_T sc)
{
  return rs_schar_len(sc);
}

int schar_cells(schar_T sc)
{
  return rs_schar_cells(sc);
}

/// gets first raw UTF-8 byte of an schar
static char schar_get_first_byte(schar_T sc)
{
  assert(!(schar_high(sc) && schar_idx(sc) >= glyph_cache.h.n_keys));
  return schar_high(sc) ? glyph_cache.keys[schar_idx(sc)] : *(char *)&sc;
}

int schar_get_first_codepoint(schar_T sc)
{
  return rs_schar_get_first_codepoint(sc);
}

/// @return ascii char or NUL if not ascii
char schar_get_ascii(schar_T sc)
{
  return rs_schar_get_ascii(sc);
}

static bool schar_in_arabic_block(schar_T sc)
{
  char first_byte = schar_get_first_byte(sc);
  return ((uint8_t)first_byte & 0xFE) == 0xD8;
}

/// Get the first two codepoints of an schar, or NUL when not available
static void schar_get_first_two_codepoints(schar_T sc, int *c0, int *c1)
{
  char sc_buf[MAX_SCHAR_SIZE];
  schar_get(sc_buf, sc);

  *c0 = utf_ptr2char(sc_buf);
  int len = utf_ptr2len(sc_buf);
  if (*c0 == NUL) {
    *c1 = NUL;
  } else {
    *c1 = utf_ptr2char(sc_buf + len);
  }
}

void line_do_arabic_shape(schar_T *buf, int cols)
{
  rs_line_do_arabic_shape(buf, cols);
}

/// clear a line in the grid starting at "off" until "width" characters
/// are cleared.
void grid_clear_line(ScreenGrid *grid, size_t off, int width, bool valid)
{
  rs_grid_clear_line(grid, off, width, valid);
}

void grid_invalidate(ScreenGrid *grid)
{
  rs_grid_invalidate(grid);
}

static bool grid_invalid_row(ScreenGrid *grid, int row)
{
  return grid->attrs[grid->line_offset[row]] < 0;
}

/// Get a single character directly from grid.chars
///
/// @param[out] attrp  set to the character's attribute (optional)
schar_T grid_getchar(ScreenGrid *grid, int row, int col, int *attrp)
{
  return rs_grid_getchar(grid, row, col, attrp);
}

static ScreenGrid *grid_line_grid = NULL;
static int grid_line_row = -1;
static int grid_line_coloff = 0;
static int grid_line_maxcol = 0;
static int grid_line_first = INT_MAX;
static int grid_line_last = 0;
static int grid_line_clear_to = 0;
static int grid_line_bg_attr = 0;
static int grid_line_clear_attr = 0;
static int grid_line_flags = 0;

/// Start a group of grid_line_puts calls that builds a single grid line.
///
/// Must be matched with a grid_line_flush call before moving to
/// another line.
void grid_line_start(GridView *view, int row)
{
  rs_grid_line_start(view, row);
}

void screengrid_line_start(ScreenGrid *grid, int row, int col)
{
  rs_screengrid_line_start(grid, row, col);
}

/// Get present char from current rendered screen line
///
/// This indicates what already is on screen, not the pending render buffer.
///
/// @return char or space if out of bounds
schar_T grid_line_getchar(int col, int *attr)
{
  return rs_grid_line_getchar(col, attr);
}

void grid_line_put_schar(int col, schar_T schar, int attr)
{
  rs_grid_line_put_schar(col, schar, attr);
}

/// Put string "text" at "col" position relative to the grid line from the
/// recent grid_line_start() call.
///
/// @param textlen length of string or -1 to use strlen(text)
/// Note: only outputs within one row!
///
/// @return number of grid cells used
int grid_line_puts(int col, const char *text, int textlen, int attr)
{
  return rs_grid_line_puts(col, text, textlen, attr);
}

int grid_line_fill(int start_col, int end_col, schar_T sc, int attr)
{
  return rs_grid_line_fill(start_col, end_col, sc, attr);
}

/// @param bg_attr     applies to both the buffered line and the columns to clear
/// @param clear_attr  applies only to the columns to clear
void grid_line_clear_end(int start_col, int end_col, int bg_attr, int clear_attr)
{
  rs_grid_line_clear_end(start_col, end_col, bg_attr, clear_attr);
}

/// move the cursor to a position in a currently rendered line.
void grid_line_cursor_goto(int col)
{
  rs_grid_line_cursor_goto(col);
}

void grid_line_mirror(int width)
{
  rs_grid_line_mirror(width);
}

void linebuf_mirror(int *firstp, int *lastp, int *clearp, int width)
{
  rs_linebuf_mirror(firstp, lastp, clearp, width);
}

/// End a group of grid_line_puts calls and send the screen buffer to the UI layer.
void grid_line_flush(void)
{
  rs_grid_line_flush();
}

/// flush grid line but only if on a valid row
///
/// This is a stopgap until message.c has been refactored to behave
void grid_line_flush_if_valid_row(void)
{
  rs_grid_line_flush_if_valid_row();
}

void grid_clear(GridView *grid, int start_row, int end_row, int start_col, int end_col, int attr)
{
  rs_grid_clear(grid, start_row, end_row, start_col, end_col, attr);
}

/// Check whether the given character needs redrawing:
/// - the (first byte of the) character is different
/// - the attributes are different
/// - the character is multi-byte and the next byte is different
/// - the character is two cells wide and the second cell differs.
static int grid_char_needs_redraw(ScreenGrid *grid, int col, size_t off_to, int cols)
{
  return (cols > 0
          && ((linebuf_char[col] != grid->chars[off_to]
               || linebuf_attr[col] != grid->attrs[off_to]
               || (cols > 1 && linebuf_char[col + 1] == 0
                   && linebuf_char[col + 1] != grid->chars[off_to + 1]))
              || exmode_active  // TODO(bfredl): what in the actual fuck
              || rdb_flags & kOptRdbFlagNodelta));
}

/// Move one buffered line to the window grid, but only the characters that
/// have actually changed.  Handle insert/delete character.
///
/// @param coloff  gives the first column on the grid for this line.
/// @param endcol  gives the columns where valid characters are.
/// @param clear_width  see SLF_RIGHTLEFT.
/// @param clear_attr   combined with "bg_attr" for the columns to clear.
/// @param flags  can have bits:
/// - SLF_RIGHTLEFT  rightleft text, like a window with 'rightleft' option set:
///   - When false, clear columns "endcol" to "clear_width".
///   - When true, clear columns "col" to "endcol".
/// - SLF_WRAP  hint to UI that "row" contains a line wrapped into the next row.
/// - SLF_INC_VCOL:
///   - When false, use "last_vcol" for grid->vcols[] of the columns to clear.
///   - When true, use an increasing sequence starting from "last_vcol + 1" for
///     grid->vcols[] of the columns to clear.
void grid_put_linebuf(ScreenGrid *grid, int row, int coloff, int col, int endcol, int clear_width,
                      int bg_attr, int clear_attr, colnr_T last_vcol, int flags)
{
  rs_grid_put_linebuf(grid, row, coloff, col, endcol, clear_width, bg_attr, clear_attr, last_vcol,
                      flags);
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
  set_destroy(glyph, &glyph_cache);
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

/// assign a handle to the grid. The grid need not be allocated.
void grid_assign_handle(ScreenGrid *grid)
{
  rs_grid_assign_handle(grid);
}

/// insert lines on the screen and move the existing lines down
/// 'line_count' is the number of lines to be inserted.
/// 'end' is the line after the scrolled part. Normally it is Rows.
/// 'col' is the column from with we start inserting.
//
/// 'row', 'col' and 'end' are relative to the start of the region.
void grid_ins_lines(ScreenGrid *grid, int row, int line_count, int end, int col, int width)
{
  rs_grid_ins_lines(grid, row, line_count, end, col, width);
}

/// delete lines on the screen and move lines up.
/// 'end' is the line after the scrolled part. Normally it is Rows.
/// When scrolling region used 'off' is the offset from the top for the region.
/// 'row' and 'end' are relative to the start of the region.
void grid_del_lines(ScreenGrid *grid, int row, int line_count, int end, int col, int width)
{
  rs_grid_del_lines(grid, row, line_count, end, col, width);
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

static int get_bordertext_col(int total_col, int text_width, AlignTextPos align)
{
  return rs_get_bordertext_col(total_col, text_width, (int)align);
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
      int title_col = get_bordertext_col(icol, config->title_width, config->title_pos);
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
      int footer_col = get_bordertext_col(icol, config->footer_width, config->footer_pos);
      grid_draw_bordertext(config->footer_chunks, footer_col, winbl, hl_attr, kBorderTextFooter);
    }
    if (adj[1]) {
      grid_line_put_schar(icol + adj[3], chars[4], attrs[4]);
    }
    grid_line_flush();
  }
}

static void linecopy(ScreenGrid *grid, int to, int from, int col, int width)
{
  unsigned off_to = (unsigned)(grid->line_offset[to] + (size_t)col);
  unsigned off_from = (unsigned)(grid->line_offset[from] + (size_t)col);

  memmove(grid->chars + off_to, grid->chars + off_from, (size_t)width * sizeof(schar_T));
  memmove(grid->attrs + off_to, grid->attrs + off_from, (size_t)width * sizeof(sattr_T));
  memmove(grid->vcols + off_to, grid->vcols + off_from, (size_t)width * sizeof(colnr_T));
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

/// Put a unicode character in a screen cell.
schar_T schar_from_char(int c)
{
  return rs_schar_from_char(c);
}
