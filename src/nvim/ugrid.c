/// @file ugrid.c
///
/// Rust bridge for UGrid functions.
/// All implementations are in src/nvim-rs/ugrid/.

#include "nvim/ugrid.h"

#include "ugrid.c.generated.h"

// Rust FFI declarations
extern void rs_ugrid_init(UGrid *grid);
extern void rs_ugrid_free(UGrid *grid);
extern void rs_ugrid_resize(UGrid *grid, int width, int height);
extern void rs_ugrid_clear(UGrid *grid);
extern void rs_ugrid_clear_chunk(UGrid *grid, int row, int col, int endcol, sattr_T attr);
extern void rs_ugrid_goto(UGrid *grid, int row, int col);
extern void rs_ugrid_scroll(UGrid *grid, int top, int bot, int left, int right, int count);

void ugrid_init(UGrid *grid)
{
  rs_ugrid_init(grid);
}

void ugrid_free(UGrid *grid)
{
  rs_ugrid_free(grid);
}

void ugrid_resize(UGrid *grid, int width, int height)
{
  rs_ugrid_resize(grid, width, height);
}

void ugrid_clear(UGrid *grid)
{
  rs_ugrid_clear(grid);
}

void ugrid_clear_chunk(UGrid *grid, int row, int col, int endcol, sattr_T attr)
{
  rs_ugrid_clear_chunk(grid, row, col, endcol, attr);
}

void ugrid_goto(UGrid *grid, int row, int col)
{
  rs_ugrid_goto(grid, row, col);
}

void ugrid_scroll(UGrid *grid, int top, int bot, int left, int right, int count)
{
  rs_ugrid_scroll(grid, top, bot, left, right, count);
}
