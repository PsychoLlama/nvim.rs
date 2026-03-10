#pragma once

#include <stdbool.h>

#include "nvim/buffer_defs.h"
#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/grid_defs.h"
#include "nvim/macros_defs.h"
#include "nvim/menu_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

/// Used for popup menu items.
typedef struct {
  char *pum_text;       ///< main menu text
  char *pum_kind;       ///< extra kind text (may be truncated)
  char *pum_extra;      ///< extra menu text (may be truncated)
  char *pum_info;       ///< extra info
  int pum_cpt_source_idx;    ///< index of completion source in 'cpt'
  int pum_user_abbr_hlattr;  ///< highlight attribute for abbr
  int pum_user_kind_hlattr;  ///< highlight attribute for kind
} pumitem_T;

EXTERN ScreenGrid pum_grid INIT( = SCREEN_GRID_INIT);

/// Result type for `nvim_pum_ui_pum_get_pos` (Rust FFI).
typedef struct {
  int valid;
  double width;
  double height;
  double row;
  double col;
} PumUiPos;

/// Result type for `nvim_pum_mouse_find_win_outer` (Rust FFI).
typedef struct {
  int grid;
  int row;
  int col;
} PumMouseFindResult;

/// Result type for `nvim_pum_get_win_by_grid` (Rust FFI).
typedef struct {
  int winrow;
  int wincol;
  int view_height;
  int view_width;
  int valid;
} PumWinInfo;

/// Opaque border configuration for popup menu rendering (Rust FFI).
typedef struct PumBorderConfig PumBorderConfig;

/// Result type for display geometry computation (Rust FFI).
typedef struct {
  int pum_win_row;
  int cursor_col;
  int anchor_grid;
  int win_row_offset;
  int win_col_offset;
  int above_row;
  int below_row;
} PumDisplayGeometry;

/// state for pum_ext_select_item.
EXTERN struct {
  bool active;
  int item;
  bool insert;
  bool finish;
} pum_want;

/// Popup menu state owned by Rust.
/// Field layout must match `PumState` in src/nvim-rs/popupmenu/src/lib.rs.
typedef struct {
  pumitem_T *array;
  int size;
  int selected;
  int first;
  int height;
  int width;
  int base_width;
  int kind_width;
  int extra_width;
  int scrollbar;
  int rl;
  int anchor_grid;
  int row;
  int col;
  int win_row_offset;
  int win_col_offset;
  int left_col;
  int right_col;
  int above;
  int is_visible;
  int is_drawn;
  int external;
  int invalid;
} PumState;

extern PumState PUM_STATE;

#include "popupmenu.h.generated.h"
