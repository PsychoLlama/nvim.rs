/// @file popupmenu.c
///
/// Popup menu (PUM)

#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <string.h>

#include "nvim/api/buffer.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/api/vim.h"
#include "nvim/api/win_config.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand.h"
#include "nvim/decoration.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/eval/typval.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/fuzzy.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/grid_defs.h"
#include "nvim/highlight.h"
#include "nvim/highlight_defs.h"
#include "nvim/highlight_group.h"
#include "nvim/insexpand.h"
#include "nvim/keycodes.h"
#include "nvim/mbyte.h"
#include "nvim/memory.h"
#include "nvim/memory_defs.h"
#include "nvim/menu.h"
#include "nvim/message.h"
#include "nvim/mouse.h"
#include "nvim/move.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/plines.h"
#include "nvim/popupmenu.h"
#include "nvim/pos_defs.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/syntax.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_compositor.h"
#include "nvim/ui_defs.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"
#include "nvim/winfloat.h"

// Rust FFI declarations
extern const char *rs_ins_compl_leader(void);
extern unsigned rs_get_cot_flags(void);
extern void rs_pum_ext_select_item(int item, int insert, int finish);
extern int rs_pum_undisplay(int immediate);
extern int rs_pum_border_width(void);


typedef struct {
  int row;
  int height;
  int above;
} PumVerticalResult;
extern PumVerticalResult rs_pum_compute_vertical(int size, int pum_win_row, int above_row,
                                                  int below_row, int pum_border_size,
                                                  int cmdline_row, int is_cmdline,
                                                  int has_target_win, int context_above,
                                                  int context_below);

typedef struct {
  int col;
  int width;
} PumHorizontalResult;
extern PumHorizontalResult rs_pum_compute_horizontal(int cursor_col, int max_col, int pum_rl,
                                                     int pum_scrollbar, int pum_base_width,
                                                     int pum_kind_width, int pum_extra_width);

extern void rs_pum_display(pumitem_T *array, int size, int selected, int array_changed,
                            int cmd_startcol);



// Static string constants for border comparison (exposed to Rust)
const char *const opt_winborder_shadow = "shadow";
const char *const opt_winborder_none = "none";


// ui_call_win_float_pos wrapper for pum_grid
void nvim_pum_ui_call_win_float_pos(int handle, const char *anchor, int anchor_grid,
                                     int row, int col, int zindex, int comp_index,
                                     int comp_row, int comp_col)
{
  ui_call_win_float_pos(handle, -1, cstr_as_string(anchor), anchor_grid,
                        row, col, false, zindex, comp_index, comp_row, comp_col);
}



// Static assertions for constants used by Rust FFI
_Static_assert(kUIMultigrid == 6, "kUIMultigrid must be 6");
_Static_assert(kUIPopupmenu == 1, "kUIPopupmenu must be 1");
_Static_assert(kUIWildmenu == 3, "kUIWildmenu must be 3");

// mouse_find_win_outer wrapper
PumMouseFindResult nvim_pum_mouse_find_win_outer(int grid, int row, int col)
{
  mouse_find_win_outer(&grid, &row, &col);
  return (PumMouseFindResult){ grid, row, col };
}


// curwin geometry for make_popup
/// Batch curwin geometry accessor (replaces 10 individual nvim_pum_curwin_* functions).
PumCurwinGeometry nvim_pum_get_curwin_geometry(void)
{
  return (PumCurwinGeometry) {
    .row_offset = curwin->w_grid.row_offset,
    .col_offset = curwin->w_grid.col_offset,
    .wrow = curwin->w_wrow,
    .wcol = curwin->w_wcol,
    .p_rl = curwin->w_p_rl,
    .view_width = curwin->w_view_width,
    .winrow = curwin->w_winrow,
    .wincol = curwin->w_wincol,
    .grid_target_handle = curwin->w_grid.target->handle,
    .grid_target_is_default = (curwin->w_grid.target == &default_grid) ? 1 : 0,
  };
}

// Text attrs computation helpers

/// Get completion leader string.
char *nvim_pum_get_compl_leader(void)
{
  return (State & MODE_CMDLINE) ? cmdline_compl_pattern() : (char *)rs_ins_compl_leader();
}

/// Check if completion is fuzzy.
int nvim_pum_compl_is_fuzzy(void)
{
  return (State & MODE_CMDLINE) ? cmdline_compl_is_fuzzy()
                                : (rs_get_cot_flags() & kOptCotFlagFuzzy) != 0;
}

/// Fuzzy match helper: returns flat array of matching positions and count.
/// Caller must free the returned array with xfree().
/// Returns NULL if no match. Sets *out_len to number of positions.
uint32_t *nvim_pum_fuzzy_match_positions(const char *text, const char *leader, int *out_len)
{
  garray_T *ga = fuzzy_match_str_with_pos(text, leader);
  if (!ga) {
    *out_len = 0;
    return NULL;
  }
  int len = ga->ga_len;
  uint32_t *positions = xmalloc(sizeof(uint32_t) * (size_t)len);
  memcpy(positions, ga->ga_data, sizeof(uint32_t) * (size_t)len);
  ga_clear(ga);
  xfree(ga);
  *out_len = len;
  return positions;
}


// position_at_mouse helpers

PumWinInfo nvim_pum_get_win_by_grid(int grid)
{
  win_T *wp = get_win_by_grid_handle(grid);
  if (wp == NULL) {
    return (PumWinInfo){ 0, 0, 0, 0, 0 };
  }
  return (PumWinInfo){
    wp->w_winrow, wp->w_wincol,
    wp->w_view_height, wp->w_view_width, 1
  };
}


_Static_assert(kOptCotFlagFuzzy == 0x80, "kOptCotFlagFuzzy must be 0x80");

// Preview window helpers

/// Set window config fields and call win_config_float.
void nvim_pum_win_config_set_and_apply(win_T *wp, int width, int col, int anchor,
                                       int height, int row, int hide)
{
  wp->w_config.width = width;
  wp->w_config.col = col;
  wp->w_config.anchor = anchor;
  wp->w_view_width = width;
  wp->w_config.height = height;
  wp->w_config.row = row;
  wp->w_config.hide = hide != 0;
  win_config_float(wp, wp->w_config);
}



_Static_assert(kFloatAnchorSouth == 2, "kFloatAnchorSouth must be 2");

// show_popupmenu helpers

/// Set mousemoveevent option via UI.
void nvim_pum_ui_set_mousemoveevent(int val)
{
  ui_call_option_set(STATIC_CSTR_AS_STRING("mousemoveevent"), BOOLEAN_OBJ(val != 0));
}

/// Batch key constant accessor (replaces 15 individual nvim_key_* functions).
PumKeyConstants nvim_pum_get_key_constants(void)
{
  return (PumKeyConstants) {
    .key_esc = ESC,
    .key_ctrl_c = Ctrl_C,
    .key_car = CAR,
    .key_nl = NL,
    .key_k_up = K_UP,
    .key_k_down = K_DOWN,
    .key_k_mouseup = K_MOUSEUP,
    .key_k_mousedown = K_MOUSEDOWN,
    .key_k_rightmouse = K_RIGHTMOUSE,
    .key_k_leftdrag = K_LEFTDRAG,
    .key_k_rightdrag = K_RIGHTDRAG,
    .key_k_mousemove = K_MOUSEMOVE,
    .key_k_leftmouse = K_LEFTMOUSE,
    .key_k_leftmouse_nm = K_LEFTMOUSE_NM,
    .key_k_rightrelease = K_RIGHTRELEASE,
  };
}


/// Emit error message.
void nvim_pum_emsg_menu_mode(void)
{
  emsg(_(e_menu_only_exists_in_another_mode));
}

/// Opaque border configuration for popup menu rendering.
/// Bundles WinConfig + border attrs/chars so Rust doesn't need WinConfig layout.
struct PumBorderConfig {
  WinConfig fconfig;
  bool has_border;
  bool is_shadow;
  schar_T scrollbar_border_char;
  int scrollbar_border_attr;
};

/// Parse border configuration. Returns heap-allocated PumBorderConfig.
/// Returns NULL on parse error (emsg already called).
PumBorderConfig *nvim_pum_parse_border(int has_scrollbar)
{
  PumBorderConfig *cfg = xcalloc(1, sizeof(*cfg));
  cfg->fconfig = WIN_CONFIG_INIT;

  int bw = rs_pum_border_width();
  cfg->has_border = bw > 0;

  if (bw > 0) {
    Error err = ERROR_INIT;
    if (!parse_winborder(&cfg->fconfig, p_pumborder, &err)) {
      if (ERROR_SET(&err)) {
        emsg(err.msg);
      }
      api_clear_error(&err);
      xfree(cfg);
      return NULL;
    }

    // Shadow style
    if (strequal(p_pumborder, opt_winborder_values[3])) {
      cfg->fconfig.shadow = true;
      int blend = SYN_GROUP_STATIC("PmenuShadow");
      int through = SYN_GROUP_STATIC("PmenuShadowThrough");
      cfg->fconfig.border_hl_ids[2] = through;
      cfg->fconfig.border_hl_ids[3] = blend;
      cfg->fconfig.border_hl_ids[4] = blend;
      cfg->fconfig.border_hl_ids[5] = blend;
      cfg->fconfig.border_hl_ids[6] = through;
    }
    cfg->is_shadow = cfg->fconfig.shadow;

    // Convert border highlight IDs to attributes
    for (int i = 0; i < 8; i++) {
      int attr = hl_attr_active[HLF_PBR];
      if (cfg->fconfig.border_hl_ids[i]) {
        attr = hl_get_ui_attr(-1, HLF_PBR, cfg->fconfig.border_hl_ids[i], false);
      }
      cfg->fconfig.border_attr[i] = attr;
    }
    api_clear_error(&err);

    if (has_scrollbar) {
      cfg->scrollbar_border_char = schar_from_str(cfg->fconfig.border_chars[3]);
      cfg->scrollbar_border_attr = cfg->fconfig.border_attr[3];
    }
  }

  return cfg;
}

/// Get has_border from PumBorderConfig.
int nvim_pum_border_cfg_has_border(PumBorderConfig *cfg)
{
  return cfg->has_border ? 1 : 0;
}

/// Get is_shadow from PumBorderConfig.
int nvim_pum_border_cfg_is_shadow(PumBorderConfig *cfg)
{
  return cfg->is_shadow ? 1 : 0;
}

/// Get fconfig.border (whether border chars are set) from PumBorderConfig.
int nvim_pum_border_cfg_has_border_chars(PumBorderConfig *cfg)
{
  return cfg->fconfig.border ? 1 : 0;
}

/// Get scrollbar border char from PumBorderConfig.
schar_T nvim_pum_border_cfg_scrollbar_char(PumBorderConfig *cfg)
{
  return cfg->scrollbar_border_char;
}

/// Get scrollbar border attr from PumBorderConfig.
int nvim_pum_border_cfg_scrollbar_attr(PumBorderConfig *cfg)
{
  return cfg->scrollbar_border_attr;
}

/// Draw border on pum_grid using PumBorderConfig.
void nvim_pum_border_draw(PumBorderConfig *cfg)
{
  grid_draw_border(&pum_grid, &cfg->fconfig, NULL, 0, NULL);
}

/// Free PumBorderConfig.
void nvim_pum_border_cfg_free(PumBorderConfig *cfg)
{
  xfree(cfg);
}

// Selection / preview window management


/// Hide a floating preview window by setting its hide flag and reconfiguring.
void nvim_pum_win_config_float_hide(win_T *wp)
{
  wp->w_config.hide = true;
  win_config_float(wp, wp->w_config);
}


// nvim_get_Rows: already defined in window.c

/// Check if the current buffer can be reused as a wipeout buffer.
/// Checks: b_nwindows == 1, no filename, nofile buftype, bh starts with 'w'.
int nvim_pum_curbuf_can_reuse(void)
{
  return (curbuf->b_nwindows == 1)
         && (curbuf->b_fname == NULL)
         && bt_nofile(curbuf)
         && (curbuf->b_p_bh[0] == 'w');
}


/// Set wipeout buffer options on curbuf (swapfile, buflisted, buftype, bufhidden, diff).
void nvim_pum_set_wipeout_options(void)
{
  set_option_value_give_err(kOptSwapfile, BOOLEAN_OPTVAL(false), OPT_LOCAL);
  set_option_value_give_err(kOptBuflisted, BOOLEAN_OPTVAL(false), OPT_LOCAL);
  set_option_value_give_err(kOptBuftype, STATIC_CSTR_AS_OPTVAL("nofile"), OPT_LOCAL);
  set_option_value_give_err(kOptBufhidden, STATIC_CSTR_AS_OPTVAL("wipe"), OPT_LOCAL);
  set_option_value_give_err(kOptDiff, BOOLEAN_OPTVAL(false), OPT_LOCAL);
}


// Display orchestrator

/// Compute the complete display geometry for popup menu positioning.
/// This encapsulates all target_win field access and cmdline_win queries.
PumDisplayGeometry nvim_pum_compute_geometry(int cmd_startcol)
{
  PumDisplayGeometry g = { 0, 0, DEFAULT_GRID_HANDLE, 0, 0, 0, 0 };
  int is_cmdline = (State & MODE_CMDLINE) != 0;

  g.below_row = MAX(cmdline_row, curwin->w_winrow + curwin->w_view_height);
  if (is_cmdline) {
    g.below_row = cmdline_row;
  }

  win_T *target_win = is_cmdline ? cmdline_win : curwin;

  if (is_cmdline) {
    g.pum_win_row = cmdline_win ? cmdline_win->w_wrow : ui_has(kUICmdline) ? 0 : cmdline_row;
    g.cursor_col = (cmdline_win ? cmdline_win->w_config._cmdline_offset : 0) + cmd_startcol;
    g.cursor_col %= cmdline_win ? cmdline_win->w_view_width : Columns;
    g.anchor_grid = ui_has(kUICmdline) ? -1 : DEFAULT_GRID_HANDLE;
  } else {
    g.pum_win_row = curwin->w_wrow;
    if (PUM_STATE.rl) {
      g.cursor_col = curwin->w_view_width - curwin->w_wcol - 1;
    } else {
      g.cursor_col = curwin->w_wcol;
    }
  }

  if (target_win != NULL) {
    g.anchor_grid = target_win->w_grid.target->handle;
    g.pum_win_row += target_win->w_grid.row_offset;
    g.cursor_col += target_win->w_grid.col_offset;
    if (target_win->w_grid.target != &default_grid) {
      g.pum_win_row += target_win->w_winrow;
      g.cursor_col += target_win->w_wincol;
      if (!ui_has(kUIMultigrid)) {
        g.anchor_grid = DEFAULT_GRID_HANDLE;
      } else {
        g.win_row_offset = target_win->w_winrow;
        g.win_col_offset = target_win->w_wincol;
      }
    }
  }

  return g;
}


/// Call ui_call_popupmenu_show with Arena-built array. Handles all Arena allocation.
void nvim_pum_ext_show(pumitem_T *array, int size, int selected,
                       int pum_win_row, int cursor_col, int anchor_grid,
                       int win_row_offset, int win_col_offset)
{
  Arena arena = ARENA_EMPTY;
  Array arr = arena_array(&arena, (size_t)size);
  for (int i = 0; i < size; i++) {
    Array item = arena_array(&arena, 4);
    ADD_C(item, CSTR_AS_OBJ(array[i].pum_text));
    ADD_C(item, CSTR_AS_OBJ(array[i].pum_kind));
    ADD_C(item, CSTR_AS_OBJ(array[i].pum_extra));
    ADD_C(item, CSTR_AS_OBJ(array[i].pum_info));
    ADD_C(arr, ARRAY_OBJ(item));
  }
  ui_call_popupmenu_show(arr, selected, pum_win_row - win_row_offset,
                         cursor_col - win_col_offset, anchor_grid);
  arena_mem_free(arena_finish(&arena));
}


/// Find preview window row adjustments using FOR_ALL_WINDOWS_IN_TAB.
/// Returns (above_row_adj, below_row_adj) via output params.
/// above_row_adj > 0 means above_row should be updated.
/// below_row_adj > 0 means below_row should be updated.
void nvim_pum_find_pvwin_rows(int *above_row_out, int *below_row_out)
{
  *above_row_out = 0;
  *below_row_out = 0;
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (wp->w_p_pvw) {
      if (wp->w_winrow < curwin->w_winrow) {
        *above_row_out = wp->w_winrow + wp->w_height;
      } else if (wp->w_winrow > curwin->w_winrow + curwin->w_height) {
        *below_row_out = wp->w_winrow;
      }
      break;
    }
  }
}

// Static assertions for constants used by Rust FFI
_Static_assert(DEFAULT_GRID_HANDLE == 1, "DEFAULT_GRID_HANDLE must be 1");
_Static_assert(kZIndexPopupMenu == 100, "kZIndexPopupMenu must be 100");
_Static_assert(kOptCotFlagPopup == 0x10, "kOptCotFlagPopup must be 0x10");
_Static_assert(kOptCotFlagPreview == 0x08, "kOptCotFlagPreview must be 0x08");
_Static_assert(ECMD_ONE == 1, "ECMD_ONE must be 1");
_Static_assert(UPD_SOME_VALID == 35, "UPD_SOME_VALID must be 35");
_Static_assert(HLF_PNI == 41, "HLF_PNI must be 41");
_Static_assert(HLF_PSI == 42, "HLF_PSI must be 42");
_Static_assert(HLF_PNK == 45, "HLF_PNK must be 45");
_Static_assert(HLF_PSK == 46, "HLF_PSK must be 46");
_Static_assert(HLF_PNX == 47, "HLF_PNX must be 47");
_Static_assert(HLF_PSX == 48, "HLF_PSX must be 48");
_Static_assert(HLF_PSB == 49, "HLF_PSB must be 49");
_Static_assert(HLF_PST == 50, "HLF_PST must be 50");
_Static_assert(HLF_PBR == 51, "HLF_PBR must be 51");
_Static_assert(CPT_ABBR == 0, "CPT_ABBR must be 0");
_Static_assert(kZIndexCmdlinePopupMenu == 250, "kZIndexCmdlinePopupMenu must be 250");
_Static_assert(MODE_CMDLINE == 0x08, "MODE_CMDLINE must be 0x08");

#include "popupmenu.c.generated.h"

#define PUM_DEF_HEIGHT 10

/// Compute vertical placement for popup menu (writes PUM_STATE.row, .height, .above).
void nvim_pum_compute_vp(int size, int pum_win_row, int above_row, int below_row,
                         int border_width)
{
  win_T *target_win = (State & MODE_CMDLINE) ? cmdline_win : curwin;
  int is_cmdline = (State & MODE_CMDLINE) != 0;
  int has_target_win = target_win != NULL;
  int context_above = 0;
  int context_below = 0;

  if (has_target_win) {
    context_above = target_win->w_wrow - target_win->w_cline_row;
    validate_cheight(target_win);
    context_below = target_win->w_cline_row + target_win->w_cline_height - target_win->w_wrow;
  }

  PumVerticalResult result = rs_pum_compute_vertical(
      size, pum_win_row, above_row, below_row, border_width,
      cmdline_row, is_cmdline, has_target_win, context_above, context_below);
  PUM_STATE.row = result.row;
  PUM_STATE.height = result.height;
  PUM_STATE.above = result.above;
}

/// Compute horizontal placement for popup menu (writes PUM_STATE.col, .width).
void nvim_pum_compute_hp(int cursor_col)
{
  win_T *target_win = (State & MODE_CMDLINE) ? cmdline_win : curwin;
  int max_col = MAX(Columns, target_win ? (target_win->w_wincol + target_win->w_view_width) : 0);
  PumHorizontalResult result = rs_pum_compute_horizontal(
      cursor_col, max_col, PUM_STATE.rl, PUM_STATE.scrollbar,
      PUM_STATE.base_width, PUM_STATE.kind_width, PUM_STATE.extra_width);
  PUM_STATE.col = result.col;
  PUM_STATE.width = result.width;
}

/// Show the popup menu with items "array[size]".
/// "array" must remain valid until pum_undisplay() is called!
/// When possible the leftmost character is aligned with cursor column.
/// The menu appears above the screen line "row" or at "row" + "height" - 1.
///
/// @param array
/// @param size
/// @param selected index of initially selected item, -1 if out of range
/// @param array_changed if true, array contains different items since last call
///                      if false, a new item is selected, but the array
///                      is the same
/// @param cmd_startcol only for cmdline mode: column of completed match
void pum_display(pumitem_T *array, int size, int selected, bool array_changed, int cmd_startcol)
{
  rs_pum_display(array, size, selected, (int)array_changed, cmd_startcol);
}

// pum_redraw: migrated to Rust (redraw.rs) via #[export_name]

// nvim_pum_display_impl: migrated to Rust (display.rs)

// nvim_pum_compute_text_attrs_impl: migrated to Rust (render.rs)
// nvim_pum_grid_puts_with_attrs_impl: migrated to Rust (render.rs)

// nvim_pum_redraw_impl: migrated to Rust (redraw.rs)

/// Set the informational text in the preview buffer when the completion
/// item does not include a dedicated preview or popup window.
///
/// @param[in]  buf        Buffer where the text will be set.
/// @param[in]  info       Informational text to display in the preview buffer.
/// @param[in]  lnum       Where to start the text. Incremented for each added line.
/// @param[out] max_width  Maximum width of the displayed text.
void nvim_pum_preview_set_text_impl(buf_T *buf, char *info, linenr_T *lnum, int *max_width)
{
  Error err = ERROR_INIT;
  Arena arena = ARENA_EMPTY;
  Array replacement = ARRAY_DICT_INIT;
  buf->b_p_ma = true;

  // Iterate through the string line by line by temporarily replacing newlines with NUL
  for (char *curr = info, *next; curr; curr = next ? next + 1 : NULL) {
    if ((next = strchr(curr, '\n'))) {
      *next = NUL;  // Temporarily replace the newline with a string terminator
    }
    // Only skip if this is an empty line AND it's the last line
    if (*curr == '\0' && !next) {
      break;
    }

    *max_width = MAX(*max_width, (int)mb_string2cells(curr));
    ADD(replacement, STRING_OBJ(cstr_to_string(curr)));
    (*lnum)++;

    if (next) {
      *next = '\n';
    }
  }

  int original_textlock = textlock;
  if (textlock > 0) {
    textlock = 0;
  }
  nvim_buf_set_lines(0, buf->handle, 0, -1, false, replacement, &arena, &err);
  textlock = original_textlock;
  if (ERROR_SET(&err)) {
    emsg(err.msg);
    api_clear_error(&err);
  }
  arena_mem_free(arena_finish(&arena));
  api_free_array(replacement);
  buf->b_p_ma = false;
}

// nvim_pum_adjust_info_position_impl: migrated to Rust (preview.rs)
// pum_set_info: migrated to Rust (preview.rs) via #[export_name]

// pum_set_selected: migrated to Rust (selection.rs)

/// Undisplay the popup menu (later).
void pum_undisplay(bool immediate)
{
  if (rs_pum_undisplay(immediate ? 1 : 0)) {
    pum_check_clear();
  }
}

// pum_check_clear: migrated to Rust (display.rs) via #[export_name]
// pum_clear: migrated to Rust (lib.rs) via #[export_name]
// pum_visible: migrated to Rust (lib.rs) via #[export_name]
// pum_drawn: migrated to Rust (lib.rs) via #[export_name]
// pum_invalidate: migrated to Rust (display.rs) via #[export_name]
// pum_recompose: migrated to Rust (display.rs) via #[export_name]
// pum_get_height: migrated to Rust (lib.rs) via #[export_name]
// pum_set_event_info: migrated to Rust (event.rs) via #[export_name]
// pum_show_popupmenu: migrated to Rust (context_menu.rs) via #[export_name]
// pum_make_popup: migrated to Rust (context_menu.rs) via #[export_name]
// pum_ui_flush: migrated to Rust (display.rs) via #[export_name]

void pum_ext_select_item(int item, bool insert, bool finish)
{
  rs_pum_ext_select_item(item, insert ? 1 : 0, finish ? 1 : 0);
}

// nvim_pum_position_at_mouse_impl: migrated to Rust (mouse.rs)

/// Select the pum entry at the mouse position.
// nvim_pum_select_mouse_pos_impl: migrated to Rust (mouse.rs)

// nvim_pum_execute_menu_impl: migrated to Rust (context_menu.rs)

/// Open the terminal version of the popup menu and don't return until it is closed.
// nvim_pum_show_popupmenu_impl: migrated to Rust (context_menu.rs)

// nvim_pum_make_popup_impl: migrated to Rust (context_menu.rs)

// nvim_pum_ui_flush_impl: migrated to Rust (display.rs)
