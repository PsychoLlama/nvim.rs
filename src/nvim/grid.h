#pragma once

#include <stdbool.h>
#include <stddef.h>  // IWYU pragma: keep

#include "nvim/buffer_defs.h"  // IWYU pragma: keep
#include "nvim/grid_defs.h"  // IWYU pragma: keep
#include "nvim/macros_defs.h"
#include "nvim/pos_defs.h"
#include "nvim/types_defs.h"

/// By default, all windows are drawn on a single rectangular grid, represented by
/// this ScreenGrid instance. In multigrid mode each window will have its own
/// grid, then this is only used for global screen elements that hasn't been
/// externalized.
///
/// Note: before the screen is initialized and when out of memory these can be
/// NULL.
EXTERN ScreenGrid default_grid INIT( = SCREEN_GRID_INIT);
EXTERN GridView default_gridview INIT( = { .target = &default_grid });

#define DEFAULT_GRID_HANDLE 1  // handle for the default_grid

/// While resizing the screen this flag is set.
EXTERN bool resizing_screen INIT( = 0);

EXTERN schar_T *linebuf_char INIT( = NULL);
EXTERN sattr_T *linebuf_attr INIT( = NULL);
EXTERN colnr_T *linebuf_vcol INIT( = NULL);
EXTERN char *linebuf_scratch INIT( = NULL);

/// flags for grid_put_linebuf()
enum {
  SLF_RIGHTLEFT = 1,
  SLF_WRAP      = 2,
  SLF_INC_VCOL  = 4,
};

/// Put a ASCII character in a screen cell.
///
/// If `x` is a compile time constant, schar_from_ascii(x) will also be.
/// But the specific value varies per platform.
#ifdef ORDER_BIG_ENDIAN
# define schar_from_ascii(x) ((schar_T)((x) << 24))
#else
# define schar_from_ascii(x) ((schar_T)(x))
#endif

#include "grid.h.generated.h"

// Declarations for functions now implemented in Rust via #[export_name].
// These were previously generated from C wrappers in grid.c.
schar_T schar_from_str(const char *str);
schar_T schar_from_buf(const char *buf, size_t len);
schar_T schar_from_char(int c);
bool schar_cache_clear_if_full(void);
void schar_cache_clear(void);
bool schar_high(schar_T sc);
size_t schar_get(char *buf_out, schar_T sc);
size_t schar_get_adv(char **buf_out, schar_T sc);
size_t schar_len(schar_T sc);
int schar_cells(schar_T sc);
int schar_get_first_codepoint(schar_T sc);
char schar_get_ascii(schar_T sc);
void line_do_arabic_shape(schar_T *buf, int cols);
ScreenGrid *grid_adjust(GridView *grid, int *row_off, int *col_off);
void grid_clear_line(ScreenGrid *grid, size_t off, int width, bool valid);
void grid_invalidate(ScreenGrid *grid);
schar_T grid_getchar(ScreenGrid *grid, int row, int col, int *attrp);
void grid_line_start(GridView *view, int row);
void screengrid_line_start(ScreenGrid *grid, int row, int col);
schar_T grid_line_getchar(int col, int *attr);
void grid_line_put_schar(int col, schar_T schar, int attr);
int grid_line_puts(int col, const char *text, int textlen, int attr);
int grid_line_fill(int start_col, int end_col, schar_T sc, int attr);
void grid_line_clear_end(int start_col, int end_col, int bg_attr, int clear_attr);
void grid_line_cursor_goto(int col);
void grid_line_mirror(int width);
void linebuf_mirror(int *firstp, int *lastp, int *clearp, int width);
void grid_line_flush(void);
void grid_line_flush_if_valid_row(void);
void grid_clear(GridView *grid, int start_row, int end_row, int start_col, int end_col, int attr);
void grid_put_linebuf(ScreenGrid *grid, int row, int coloff, int col, int endcol, int clear_width,
                      int bg_attr, int clear_attr, colnr_T last_vcol, int flags);
void grid_ins_lines(ScreenGrid *grid, int row, int line_count, int end, int col, int width);
void grid_del_lines(ScreenGrid *grid, int row, int line_count, int end, int col, int width);
void grid_assign_handle(ScreenGrid *grid);
