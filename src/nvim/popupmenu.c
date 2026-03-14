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
extern int rs_ins_compl_active(void);
extern const char *rs_ins_compl_leader(void);
extern int rs_compl_match_curr_select(int selected);
extern unsigned rs_get_cot_flags(void);
extern void rs_win_setheight(int height);
extern void rs_pum_ext_select_item(int item, int insert, int finish);
extern int rs_pum_undisplay(int immediate);
extern int rs_pum_border_width(void);

extern int rs_win_valid(win_T *win);
extern int rs_valid_tabpage(tabpage_T *tpc);


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

// pum_want setters for Rust FFI (some getters are defined elsewhere)
int nvim_get_pum_want_finish(void)
{
  return pum_want.finish;
}

void nvim_set_pum_want_active(int val)
{
  pum_want.active = val != 0;
}

void nvim_set_pum_want_item(int val)
{
  pum_want.item = val;
}

void nvim_set_pum_want_insert(int val)
{
  pum_want.insert = val != 0;
}

void nvim_set_pum_want_finish(int val)
{
  pum_want.finish = val != 0;
}

// Accessors for pum_array items (for Rust FFI)
const char *nvim_pum_item_get_text(const pumitem_T *array, int index)
{
  return array[index].pum_text;
}

const char *nvim_pum_item_get_kind(const pumitem_T *array, int index)
{
  return array[index].pum_kind;
}

const char *nvim_pum_item_get_extra(const pumitem_T *array, int index)
{
  return array[index].pum_extra;
}

int nvim_pum_item_get_user_abbr_hlattr(const pumitem_T *array, int index)
{
  return array[index].pum_user_abbr_hlattr;
}

int nvim_pum_item_get_user_kind_hlattr(const pumitem_T *array, int index)
{
  return array[index].pum_user_kind_hlattr;
}

// Accessor for the 'pumborder' option
const char *nvim_get_p_pumborder(void)
{
  return p_pumborder;
}

// Accessor for completion item align flags
int nvim_get_cia_flags(void)
{
  return cia_flags;
}

// Static string constants for border comparison (exposed to Rust)
const char *const opt_winborder_shadow = "shadow";
const char *const opt_winborder_none = "none";

// Accessor for window highlight attribute (for Rust FFI)
int nvim_curwin_hl_attr(int hlf)
{
  return win_hl_attr(curwin, hlf);
}

// Phase 1 accessors: pum_grid field accessors for Rust FFI
ScreenGrid *nvim_pum_get_grid_ptr(void)
{
  return &pum_grid;
}

int nvim_pum_grid_get_handle(void)
{
  return pum_grid.handle;
}

int nvim_pum_grid_get_pending_comp_index_update(void)
{
  return pum_grid.pending_comp_index_update;
}

void nvim_pum_grid_set_pending_comp_index_update(int val)
{
  pum_grid.pending_comp_index_update = val != 0;
}

int nvim_pum_grid_get_zindex(void)
{
  return pum_grid.zindex;
}

int nvim_pum_grid_get_comp_index(void)
{
  return (int)pum_grid.comp_index;
}

int nvim_pum_grid_get_comp_row(void)
{
  return pum_grid.comp_row;
}

int nvim_pum_grid_get_comp_col(void)
{
  return pum_grid.comp_col;
}

// Phase 1 accessor: ui_call_win_float_pos wrapper for pum_grid
void nvim_pum_ui_call_win_float_pos(int handle, const char *anchor, int anchor_grid,
                                     int row, int col, int zindex, int comp_index,
                                     int comp_row, int comp_col)
{
  ui_call_win_float_pos(handle, -1, cstr_as_string(anchor), anchor_grid,
                        row, col, false, zindex, comp_index, comp_row, comp_col);
}

// Phase 1 accessor: menu linked-list traversal
void *nvim_pum_menu_children(void *menu)
{
  return ((vimmenu_T *)menu)->children;
}

void *nvim_pum_menu_next(void *menu)
{
  return ((vimmenu_T *)menu)->next;
}

int nvim_pum_menu_matches_mode(void *menu, int mode)
{
  vimmenu_T *mp = (vimmenu_T *)menu;
  return (mp->modes & mp->enabled & mode) != 0;
}

void nvim_pum_execute_menu_item(void *menu)
{
  exarg_T ea;
  CLEAR_FIELD(ea);
  execute_menu(&ea, (vimmenu_T *)menu, -1);
}

// Phase 1 accessor: ui_pum_get_pos wrapper
PumUiPos nvim_pum_ui_pum_get_pos(void)
{
  PumUiPos result;
  double w, h, r, c;
  if (ui_pum_get_pos(&w, &h, &r, &c)) {
    result.valid = 1;
    result.width = w;
    result.height = h;
    result.row = r;
    result.col = c;
  } else {
    result.valid = 0;
    result.width = 0;
    result.height = 0;
    result.row = 0;
    result.col = 0;
  }
  return result;
}

// Phase 1 accessor: tv_dict_add_* wrappers for Rust FFI
void nvim_pum_dict_add_float(void *dict, const char *key, size_t key_len, double val)
{
  tv_dict_add_float((dict_T *)dict, key, key_len, val);
}

void nvim_pum_dict_add_nr(void *dict, const char *key, size_t key_len, int val)
{
  tv_dict_add_nr((dict_T *)dict, key, key_len, val);
}

void nvim_pum_dict_add_bool(void *dict, const char *key, size_t key_len, int val)
{
  tv_dict_add_bool((dict_T *)dict, key, key_len, val ? kBoolVarTrue : kBoolVarFalse);
}

// Static assertions for constants used by Rust FFI
_Static_assert(kUIMultigrid == 6, "kUIMultigrid must be 6");
_Static_assert(kUIPopupmenu == 1, "kUIPopupmenu must be 1");
_Static_assert(kUIWildmenu == 3, "kUIWildmenu must be 3");

// Phase 2 accessors: grid cleanup wrappers for Rust FFI
void nvim_pum_ui_call_popupmenu_hide(void)
{
  ui_call_popupmenu_hide();
}

void nvim_pum_ui_comp_remove_grid(void)
{
  ui_comp_remove_grid(&pum_grid);
}

void nvim_pum_ui_call_win_close_grid(void)
{
  ui_call_win_close(pum_grid.handle);
}

void nvim_pum_ui_call_grid_destroy(void)
{
  ui_call_grid_destroy(pum_grid.handle);
}

void nvim_pum_grid_free(void)
{
  grid_free(&pum_grid);
}

void *nvim_pum_win_float_find_preview(void)
{
  return win_float_find_preview();
}

void nvim_pum_win_close(void *wp)
{
  win_close((win_T *)wp, false, false);
}

// Phase 2 accessors: mouse_find_win_outer wrapper
PumMouseFindResult nvim_pum_mouse_find_win_outer(int grid, int row, int col)
{
  mouse_find_win_outer(&grid, &row, &col);
  return (PumMouseFindResult){ grid, row, col };
}

// Phase 2 accessor: check if pum_array item text is non-empty
int nvim_pum_array_item_is_nonempty(int idx)
{
  return PUM_STATE.array != NULL && *PUM_STATE.array[idx].pum_text != NUL;
}

// Phase 2 accessors: curwin geometry for make_popup
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

void *nvim_pum_menu_find(const char *path_name)
{
  return menu_find(path_name);
}

// Phase 3 accessors: text attrs computation helpers

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

/// Case-insensitive multibyte string comparison.
int nvim_pum_mb_strnicmp(const char *s1, const char *s2, size_t len)
{
  return mb_strnicmp(s1, s2, len);
}

/// Allocate int array via xmalloc.
int *nvim_pum_alloc_int_array(int count)
{
  return xmalloc(sizeof(int) * (size_t)count);
}

/// Get display width of string in cells.
int nvim_pum_vim_strsize(const char *text)
{
  return vim_strsize(text);
}

// Phase 3 accessors: grid rendering
int nvim_pum_grid_line_puts(int col, const char *text, int textlen, int attr)
{
  return grid_line_puts(col, text, textlen, attr);
}

// Phase 3 accessors: position_at_mouse helpers

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

// Phase 4 accessors: preview window helpers

/// Get line count for a window's buffer.
int nvim_pum_win_get_line_count(win_T *wp)
{
  return (int)wp->w_buffer->b_ml.ml_line_count;
}

/// Wrapper for plines_m_win.
int nvim_pum_plines_m_win(win_T *wp, int first, int last, int max_lines)
{
  return plines_m_win(wp, first, last, max_lines);
}

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


/// Block autocmds.
void nvim_pum_block_autocmds(void)
{
  block_autocmds();
}

/// Unblock autocmds.
void nvim_pum_unblock_autocmds(void)
{
  unblock_autocmds();
}

/// Increment RedrawingDisabled.
void nvim_pum_redrawing_disabled_inc(void)
{
  RedrawingDisabled++;
}

/// Decrement RedrawingDisabled.
void nvim_pum_redrawing_disabled_dec(void)
{
  RedrawingDisabled--;
}

/// Increment no_u_sync.
void nvim_pum_no_u_sync_inc(void)
{
  no_u_sync++;
}

/// Decrement no_u_sync.
void nvim_pum_no_u_sync_dec(void)
{
  no_u_sync--;
}

/// Create a floating info window (false, true args).
win_T *nvim_pum_win_float_create_info(void)
{
  return win_float_create(false, true);
}

/// Set w_topline for a window.
void nvim_pum_win_set_topline(win_T *wp, int val)
{
  wp->w_topline = val;
}

/// Set w_p_wfb for a window.
void nvim_pum_win_set_wfb(win_T *wp, int val)
{
  wp->w_p_wfb = val != 0;
}

/// Get buffer handle from a window.
buf_T *nvim_pum_win_get_buffer(win_T *wp)
{
  return wp->w_buffer;
}

/// Call redraw_later for a window.
void nvim_pum_redraw_later_win(win_T *wp, int type)
{
  redraw_later(wp, type);
}

_Static_assert(kFloatAnchorSouth == 2, "kFloatAnchorSouth must be 2");

// Phase 5 accessors: show_popupmenu helpers

/// Get get_menu_mode_flag().
int nvim_pum_get_menu_mode_flag(void)
{
  return get_menu_mode_flag();
}

/// Check if menu item name is a separator.
int nvim_pum_menu_is_separator(vimmenu_T *mp)
{
  return menu_is_separator(mp->dname) ? 1 : 0;
}

/// Get menu item display name.
char *nvim_pum_menu_get_dname(vimmenu_T *mp)
{
  return mp->dname;
}

/// Allocate pumitem_T array.
pumitem_T *nvim_pum_alloc_items(int count)
{
  return (pumitem_T *)xcalloc((size_t)count, sizeof(pumitem_T));
}

/// Set text for a pumitem_T.
void nvim_pum_item_set_text(pumitem_T *array, int idx, const char *text)
{
  array[idx].pum_text = xstrdup(text);
}

/// Free text in pumitem_T array and the array itself.
void nvim_pum_free_items(pumitem_T *array, int count)
{
  for (int i = 0; i < count; i++) {
    xfree(array[i].pum_text);
  }
  xfree(array);
}

/// Get pum_array pointer (NULL check).
int nvim_pum_array_is_null(void)
{
  return PUM_STATE.array == NULL ? 1 : 0;
}

/// Get curwin->w_p_rl.
int nvim_pum_curwin_get_p_rl(void)
{
  return curwin->w_p_rl ? 1 : 0;
}

/// Call vgetc().
int nvim_pum_vgetc(void)
{
  return vgetc();
}

/// Call vungetc().
void nvim_pum_vungetc(int c)
{
  vungetc(c);
}

/// Call setcursor_mayforce(curwin, true).
void nvim_pum_setcursor_mayforce(void)
{
  setcursor_mayforce(curwin, true);
}

/// Get p_mousemev.
int nvim_pum_get_p_mousemev(void)
{
  return p_mousemev ? 1 : 0;
}

/// Set mousemoveevent option via UI.
void nvim_pum_ui_set_mousemoveevent(int val)
{
  ui_call_option_set(STATIC_CSTR_AS_STRING("mousemoveevent"), BOOLEAN_OBJ(val != 0));
}

/// Call `rs_pum_undisplay(1)`.
void nvim_pum_call_undisplay(void)
{
  rs_pum_undisplay(1);
}

/// Set pum_grid.zindex to kZIndexCmdlinePopupMenu.
void nvim_pum_grid_set_zindex_cmdline(void)
{
  pum_grid.zindex = kZIndexCmdlinePopupMenu;
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

/// Get pum_array[idx].pum_text[0] (first character, NUL check).
int nvim_pum_array_item_text_char(int idx)
{
  return (int)(unsigned char)PUM_STATE.array[idx].pum_text[0];
}

/// Emit error message.
void nvim_pum_emsg_menu_mode(void)
{
  emsg(_(e_menu_only_exists_in_another_mode));
}

// Phase 6 accessors: grid operations for redraw

/// Call `screengrid_line_start` for pum_grid.
void nvim_pum_screengrid_line_start(int row, int col)
{
  screengrid_line_start(&pum_grid, row, col);
}

/// Call `grid_line_fill`.
void nvim_pum_grid_line_fill(int start, int end, schar_T fillchar, int attr)
{
  grid_line_fill(start, end, fillchar, attr);
}

/// Call `grid_line_put_schar`.
void nvim_pum_grid_line_put_schar(int col, schar_T sc, int attr)
{
  grid_line_put_schar(col, sc, attr);
}

/// Call `grid_line_flush`.
void nvim_pum_grid_line_flush(void)
{
  grid_line_flush();
}

/// Call `grid_assign_handle` for pum_grid.
void nvim_pum_grid_assign_handle(void)
{
  grid_assign_handle(&pum_grid);
}

/// Call `grid_alloc` for pum_grid.
void nvim_pum_grid_alloc(int rows, int cols, int keep_contents)
{
  grid_alloc(&pum_grid, rows, cols, keep_contents != 0, false);
}

/// Call `grid_invalidate` for pum_grid.
void nvim_pum_grid_invalidate(void)
{
  grid_invalidate(&pum_grid);
}

/// Call `ui_call_grid_resize` for pum_grid.
void nvim_pum_ui_call_grid_resize(void)
{
  ui_call_grid_resize(pum_grid.handle, pum_grid.cols, pum_grid.rows);
}

/// Check if pum_grid.chars is non-null.
int nvim_pum_grid_has_chars(void)
{
  return pum_grid.chars != NULL;
}

/// Get pum_grid.rows.
int nvim_pum_grid_get_rows(void)
{
  return pum_grid.rows;
}

/// Get pum_grid.cols.
int nvim_pum_grid_get_cols(void)
{
  return pum_grid.cols;
}

/// Call `ui_comp_put_grid` for pum_grid. Returns 1 if grid moved.
int nvim_pum_ui_comp_put_grid(int row, int col, int height, int width)
{
  return ui_comp_put_grid(&pum_grid, row, col, height, width, false, true) ? 1 : 0;
}

/// Call `ui_call_win_float_pos` for pum_grid.
void nvim_pum_ui_call_win_float_pos_grid(const char *anchor, int anchor_grid,
                                          int anchor_row, int anchor_col)
{
  ui_call_win_float_pos(pum_grid.handle, -1, cstr_as_string(anchor), anchor_grid,
                        anchor_row, anchor_col,
                        false, pum_grid.zindex, (int)pum_grid.comp_index,
                        pum_grid.comp_row, pum_grid.comp_col);
}

/// Check if `ui_has(kUIMultigrid)`.
int nvim_pum_ui_has_multigrid(void)
{
  return ui_has(kUIMultigrid) ? 1 : 0;
}

/// Get `W_ENDCOL(curwin)`.
int nvim_pum_curwin_end_col(void)
{
  return W_ENDCOL(curwin);
}

/// Get `fcs_trunc` character for current pum_rl.
schar_T nvim_pum_fcs_trunc(int is_rl)
{
  return is_rl ? curwin->w_p_fcs_chars.truncrl : curwin->w_p_fcs_chars.trunc;
}

/// `schar_from_ascii` wrapper.
schar_T nvim_pum_schar_from_ascii(char c)
{
  return schar_from_ascii(c);
}

/// `schar_from_str` wrapper.
schar_T nvim_pum_schar_from_str(const char *str)
{
  return schar_from_str(str);
}

/// `hl_combine_attr` wrapper.
int nvim_pum_hl_combine_attr(int a, int b)
{
  return hl_combine_attr(a, b);
}

/// Get `hl_attr_active[hlf]`.
int nvim_pum_hl_attr_active(int hlf)
{
  return hl_attr_active[hlf];
}

/// `hl_get_ui_attr` wrapper.
int nvim_pum_hl_get_ui_attr(int hlf, int id)
{
  return hl_get_ui_attr(-1, hlf, id, false);
}

/// `transstr` wrapper.
char *nvim_pum_transstr(const char *s)
{
  return transstr(s, true);
}

/// `reverse_text` wrapper.
char *nvim_pum_reverse_text(char *s)
{
  return reverse_text(s);
}

/// `mb_string2cells` wrapper.
int nvim_pum_mb_string2cells(const char *s)
{
  return (int)mb_string2cells(s);
}

/// `ptr2cells` wrapper.
int nvim_pum_ptr2cells(const char *p)
{
  return ptr2cells(p);
}

/// Advance multi-byte pointer. Returns number of bytes advanced.
int nvim_pum_mb_ptr_adv(const char *p)
{
  int len = utfc_ptr2len(p);
  return len > 0 ? len : 1;
}

/// `xfree` wrapper.
void nvim_pum_xfree(void *ptr)
{
  xfree(ptr);
}

/// Set `linebuf_char[col]`.
void nvim_pum_set_linebuf_char(int col, schar_T sc)
{
  linebuf_char[col] = sc;
}

/// Get `linebuf_char[col]`.
schar_T nvim_pum_get_linebuf_char(int col)
{
  return linebuf_char[col];
}

/// Set `linebuf_attr[col]`.
void nvim_pum_set_linebuf_attr(int col, int attr)
{
  linebuf_attr[col] = (sattr_T)attr;
}

/// Check if `State & MODE_CMDLINE`.
int nvim_pum_is_cmdline(void)
{
  return (State & MODE_CMDLINE) != 0;
}

/// Get kZIndexCmdlinePopupMenu constant.
int nvim_pum_kZIndexCmdlinePopupMenu(void)
{
  return kZIndexCmdlinePopupMenu;
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

// Phase 7 C accessor functions (selection / preview window management)


/// Hide a floating preview window by setting its hide flag and reconfiguring.
void nvim_pum_win_config_float_hide(win_T *wp)
{
  wp->w_config.hide = true;
  win_config_float(wp, wp->w_config);
}

/// Check if pum_array[idx] has pum_info (non-NULL).
int nvim_pum_array_has_info(int idx)
{
  return PUM_STATE.array[idx].pum_info != NULL;
}

/// Get pum_array[idx].pum_info pointer.
char *nvim_pum_array_get_info(int idx)
{
  return PUM_STATE.array[idx].pum_info;
}

// nvim_get_Rows: already defined in window.c

/// Get `p_pvh` (preview height option).
int nvim_pum_get_p_pvh(void)
{
  return (int)p_pvh;
}

/// Get `cmdwin_type` global.
int nvim_pum_get_cmdwin_type(void)
{
  return cmdwin_type;
}

/// Set `g_do_tagpreview` global.
void nvim_pum_set_g_do_tagpreview(int val)
{
  g_do_tagpreview = val;
}

/// Enter a window (wrapper for win_enter).
void nvim_pum_win_enter(win_T *wp, int set_curwin)
{
  win_enter(wp, set_curwin != 0);
}

/// Prepare tag preview window (wrapper for prepare_tagpreview).
/// Returns 1 if window was resized.
int nvim_pum_prepare_tagpreview(void)
{
  return prepare_tagpreview(false) ? 1 : 0;
}

/// Create a floating preview window (wrapper for win_float_create).
/// Returns the window pointer, or NULL if failed.
win_T *nvim_pum_win_float_create_preview(void)
{
  return win_float_create(true, true);
}

/// Check if curwin is a preview window or float info window.
int nvim_pum_curwin_is_pvw_or_info(void)
{
  return curwin->w_p_pvw || curwin->w_float_is_info;
}

/// Check if the current buffer can be reused as a wipeout buffer.
/// Checks: b_nwindows == 1, no filename, nofile buftype, bh starts with 'w'.
int nvim_pum_curbuf_can_reuse(void)
{
  return (curbuf->b_nwindows == 1)
         && (curbuf->b_fname == NULL)
         && bt_nofile(curbuf)
         && (curbuf->b_p_bh[0] == 'w');
}

/// Clear the current buffer (wrapper for buf_clear).
void nvim_pum_buf_clear(void)
{
  buf_clear();
}

/// Execute do_ecmd to edit a new empty buffer.
/// Returns OK (1) or FAIL (0).
int nvim_pum_do_ecmd(void)
{
  return do_ecmd(0, NULL, NULL, NULL, ECMD_ONE, 0, NULL);
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


/// Get curwin->w_height.
int nvim_pum_curwin_get_height(void)
{
  return curwin->w_height;
}

/// Set curbuf->b_changed.
void nvim_pum_set_curbuf_changed(int val)
{
  curbuf->b_changed = val != 0;
}

/// Set curbuf->b_p_ma (modifiable).
void nvim_pum_set_curbuf_modifiable(int val)
{
  curbuf->b_p_ma = val != 0;
}

/// Get curwin->w_topline.
int nvim_pum_curwin_get_topline(void)
{
  return (int)curwin->w_topline;
}

/// Set curwin->w_topline.
void nvim_pum_curwin_set_topline(int val)
{
  curwin->w_topline = (linenr_T)val;
}

/// Set curwin->w_cursor.lnum and w_cursor.col.
void nvim_pum_curwin_set_cursor(int lnum, int col)
{
  curwin->w_cursor.lnum = (linenr_T)lnum;
  curwin->w_cursor.col = (colnr_T)col;
}

/// Get curbuf->b_ml.ml_line_count.
int nvim_pum_curbuf_line_count(void)
{
  return (int)curbuf->b_ml.ml_line_count;
}


/// Go to a tabpage (wrapper for goto_tabpage_tp).
void nvim_pum_goto_tabpage(tabpage_T *tp)
{
  goto_tabpage_tp(tp, false, false);
}


/// Set curwin->w_redr_status.
void nvim_pum_curwin_set_redr_status(int val)
{
  curwin->w_redr_status = val != 0;
}

/// Validate cursor in curwin (wrapper for validate_cursor).
void nvim_pum_validate_cursor(void)
{
  validate_cursor(curwin);
}

/// Redraw later for curwin (wrapper for redraw_later).
void nvim_pum_redraw_later_curwin(int update_type)
{
  redraw_later(curwin, update_type);
}

/// Update topline for curwin (wrapper for update_topline).
void nvim_pum_update_topline(void)
{
  update_topline(curwin);
}

/// Update screen (wrapper for update_screen).
void nvim_pum_update_screen(void)
{
  update_screen();
}

/// Get curwin pointer (for save/restore).
win_T *nvim_pum_get_curwin(void)
{
  return curwin;
}

/// Get curtab pointer (for save/restore).
tabpage_T *nvim_pum_get_curtab(void)
{
  return curtab;
}

/// Check if curwin equals the saved window pointer.
int nvim_pum_curwin_is(win_T *wp)
{
  return curwin == wp;
}

/// Check if curtab equals the saved tabpage pointer.
int nvim_pum_curtab_is(tabpage_T *tp)
{
  return curtab == tp;
}

/// Get curbuf pointer (for preview_set_text).
void *nvim_pum_get_curbuf(void)
{
  return curbuf;
}

// Phase 8 C accessor functions (display orchestrator)

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

/// Call validate_cursor_col(curwin).
void nvim_pum_validate_cursor_col(void)
{
  validate_cursor_col(curwin);
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

/// Call ui_call_popupmenu_select.
void nvim_pum_ext_select(int selected)
{
  ui_call_popupmenu_select(selected);
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

// Phase 8 static assertions
_Static_assert(DEFAULT_GRID_HANDLE == 1, "DEFAULT_GRID_HANDLE must be 1");
_Static_assert(kZIndexPopupMenu == 100, "kZIndexPopupMenu must be 100");

// Phase 7 static assertions
_Static_assert(kOptCotFlagPopup == 0x10, "kOptCotFlagPopup must be 0x10");
_Static_assert(kOptCotFlagPreview == 0x08, "kOptCotFlagPreview must be 0x08");
_Static_assert(ECMD_ONE == 1, "ECMD_ONE must be 1");
_Static_assert(UPD_SOME_VALID == 35, "UPD_SOME_VALID must be 35");

// Phase 6 static assertions
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

/// Set grid zindex based on current mode (Phase 8 accessor).
void nvim_pum_set_grid_zindex_for_mode(void)
{
  pum_grid.zindex = (State & MODE_CMDLINE) ? kZIndexCmdlinePopupMenu : kZIndexPopupMenu;
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
