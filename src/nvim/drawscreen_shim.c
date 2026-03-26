// drawscreen_shim.c: Minimal C accessors for the Rust drawscreen crate.

#include <stdbool.h>

#include "nvim/drawscreen.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/grid_defs.h"
#include "nvim/statusline.h"
#include "nvim/vim_defs.h"

#include "drawscreen_shim.c.generated.h"

/// Return 1 if default_grid needs reallocation (size mismatch or NULL), else 0.
/// Also returns 0 if Rows==0 or Columns==0.
int nvim_default_grid_needs_alloc(void)
{
  if (Rows == 0 || Columns == 0) {
    return 0;
  }
  if (default_grid.chars != NULL
      && Rows == default_grid.rows
      && Columns == default_grid.cols) {
    return 0;
  }
  return 1;
}

bool nvim_get_updating_screen(void) { return updating_screen; }
