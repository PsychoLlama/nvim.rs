#include <assert.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/ascii_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/digraph.h"
#include "nvim/drawline.h"
#include "nvim/drawscreen.h"
#include "nvim/eval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/grid_defs.h"
#include "nvim/highlight.h"
#include "nvim/highlight_defs.h"
#include "nvim/highlight_group.h"
#include "nvim/macros_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/memline_defs.h"
#include "nvim/memory.h"
#include "nvim/memory_defs.h"
#include "nvim/message.h"
#include "nvim/normal.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/path.h"
#include "nvim/plines.h"
#include "nvim/pos_defs.h"
#include "nvim/sign.h"
#include "nvim/sign_defs.h"
#include "nvim/state_defs.h"
#include "nvim/statusline.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_defs.h"
#include "nvim/undo.h"
#include "nvim/window.h"

// Rust FFI declarations
extern int rs_ml_find_line_or_offset(buf_T *buf, linenr_T lnum, int *offp, bool no_ff);


/// Evaluate an expression for the statusline, returning an allocated string.
/// Returns allocated string result (caller must free via xfree), or NULL.
char *nvim_stl_eval_expr_alloc(win_T *wp, const char *expr, int expr_len)
{
  if (wp == NULL || expr == NULL || expr_len <= 0) {
    return NULL;
  }

  // Create a null-terminated copy of the expression
  char *expr_copy = xmemdupz(expr, (size_t)expr_len);

  // Evaluate the expression
  char *result = eval_to_string(expr_copy, true, false);
  xfree(expr_copy);

  return result;
}

/// Get highlight group ID by name.
int nvim_syn_name2id(const char *name)
{
  if (name == NULL) {
    return 0;
  }
  return syn_name2id(name);
}


/// Get batch cursor information for a window in one C call.
/// Clamps cursor, then fills StlCursorInfo with all commonly-needed fields.
StlCursorInfo nvim_stl_get_win_cursor_info(win_T *wp)
{
  StlCursorInfo info = { 0 };
  if (wp == NULL || wp->w_buffer == NULL) {
    return info;
  }

  buf_T *buf = wp->w_buffer;

  // Clamp cursor lnum to line count
  linenr_T lnum = wp->w_cursor.lnum;
  if (lnum > buf->b_ml.ml_line_count) {
    lnum = buf->b_ml.ml_line_count;
    wp->w_cursor.lnum = lnum;
  }
  // Clamp cursor col
  colnr_T linelen = ml_get_buf_len(buf, lnum);
  if (wp->w_cursor.col > linelen) {
    wp->w_cursor.col = linelen;
    wp->w_cursor.coladd = 0;
  }

  info.clamped_lnum = (int)lnum;
  info.cursor_invalid = (wp->w_cursor.lnum > buf->b_ml.ml_line_count) ? 1 : 0;
  info.ml_empty = (buf->b_ml.ml_flags & ML_EMPTY) ? 1 : 0;

  // Get cursor line text
  const char *line = ml_get_buf(buf, lnum);
  info.first_char = line ? (int)(uint8_t)line[0] : 0;
  info.empty_line = (line == NULL || line[0] == NUL) ? 1 : 0;

  // Byte value at cursor
  colnr_T col = wp->w_cursor.col;
  if (line != NULL && col < linelen) {
    info.byte_value = (int)(uint8_t)line[col];
  }

  // Byte offset using rs_ml_find_line_or_offset
  int l = rs_ml_find_line_or_offset(buf, lnum, NULL, false);
  if (!info.ml_empty && l >= 0) {
    info.byte_offset = l;
  }

  return info;
}

/// Get showcmd output.
int nvim_stl_get_showcmd(char *buf, int buflen)
{
  if (buf == NULL || buflen <= 0) {
    return 0;
  }
  // showcmd_buf is declared in normal.h
  if (showcmd_buf[0] == NUL) {
    buf[0] = '\0';
    return 0;
  }
  int len = (int)strlen(showcmd_buf);
  if (len >= buflen) {
    len = buflen - 1;
  }
  memcpy(buf, showcmd_buf, (size_t)len);
  buf[len] = '\0';
  return len;
}

/// Get keymap name for statusline.
int nvim_stl_get_keymap(win_T *wp, char *buf, int buflen)
{
  if (wp == NULL || buf == NULL || buflen <= 0) {
    return 0;
  }
  buf[0] = '\0';
  // Call get_keymap_str to get the keymap name formatted as "<%s>"
  int len = get_keymap_str(wp, "<%s>", buf, buflen);
  return len > 0 ? len : 0;
}


/// Get quickfix info for statusline.
int nvim_stl_get_qf_info(win_T *wp, char *buf, int buflen)
{
  if (wp == NULL || buf == NULL || buflen <= 0) {
    return 0;
  }
  buf[0] = '\0';
  // For quickfix window, show list info
  if (bt_quickfix(wp->w_buffer)) {
    const char *msg = wp->w_llist_ref ? _(msg_loclist) : _(msg_qflist);
    int len = (int)strlen(msg);
    if (len >= buflen) {
      len = buflen - 1;
    }
    memcpy(buf, msg, (size_t)len);
    buf[len] = '\0';
    return len;
  }
  return 0;
}

/// Fill NameBuff with the translated buffer name via buf_spname/home_replace/trans_characters.
void nvim_stl_get_trans_bufname(buf_T *buf)
{
  if (buf_spname(buf) != NULL) {
    xstrlcpy(NameBuff, buf_spname(buf), MAXPATHL);
  } else {
    home_replace(buf, buf->b_fname, NameBuff, MAXPATHL, true);
  }
  trans_characters(NameBuff, MAXPATHL);
}


/// Get wp->w_p_stc (statuscolumn option).
const char *nvim_stl_win_get_p_stc(win_T *wp) { return wp->w_p_stc; }


/// Get window statuscol click defs pointer.
StlClickDefinition *nvim_stl_win_get_statuscol_click_defs(win_T *wp) { return wp->w_statuscol_click_defs; }

/// Get window statuscol click defs size.
size_t nvim_stl_win_get_statuscol_click_defs_size(win_T *wp) { return wp->w_statuscol_click_defs_size; }

/// Set window statuscol click defs.
void nvim_stl_win_set_statuscol_click_defs(win_T *wp, StlClickDefinition *defs) { wp->w_statuscol_click_defs = defs; }

/// Set window statuscol click defs size.
void nvim_stl_win_set_statuscol_click_defs_size(win_T *wp, size_t size) { wp->w_statuscol_click_defs_size = size; }

/// Get stcp->width.
int nvim_stl_stcp_get_width(statuscol_T *stcp) { return stcp->width; }

/// Get stcp->hlrec pointer address (for passing to build_stl_str_hl).
stl_hlrec_t **nvim_stl_stcp_get_hlrec_ptr(statuscol_T *stcp) { return &stcp->hlrec; }



/// Evaluate a VimL expression for the statusline with context switching.
/// mode=0: full context (save/restore curwin/curbuf, set g:actual_curbuf/g:actual_curwin)
/// mode=1: fmt expr (set g:statusline_winid for "%!" expressions)
/// Returns allocated string result (caller must free via xfree), or NULL.
char *nvim_stl_eval_with_context(win_T *wp, char *expr, int mode, bool use_sandbox)
{
  char *result;

  if (mode == 1) {
    // "%!" format expression: set g:statusline_winid
    typval_T tv = {
      .v_type = VAR_NUMBER,
      .vval.v_number = wp->handle,
    };
    set_var(S_LEN("g:statusline_winid"), &tv, false);
    result = eval_to_string_safe(expr, use_sandbox, false);
    do_unlet(S_LEN("g:statusline_winid"), true);
  } else {
    // Full context switching: save/restore curwin/curbuf, set g:actual_curbuf/g:actual_curwin
    char buf_tmp[70];

    vim_snprintf(buf_tmp, sizeof(buf_tmp), "%d", curbuf->b_fnum);
    set_internal_string_var("g:actual_curbuf", buf_tmp);
    vim_snprintf(buf_tmp, sizeof(buf_tmp), "%d", curwin->handle);
    set_internal_string_var("g:actual_curwin", buf_tmp);

    buf_T *const save_curbuf = curbuf;
    win_T *const save_curwin = curwin;
    const int save_VIsual_active = VIsual_active;
    curwin = wp;
    curbuf = wp->w_buffer;
    if (curwin != save_curwin) {
      VIsual_active = false;
    }

    result = eval_to_string_safe(expr, use_sandbox, false);

    curwin = save_curwin;
    curbuf = save_curbuf;
    VIsual_active = save_VIsual_active;

    do_unlet(S_LEN("g:actual_curbuf"), true);
    do_unlet(S_LEN("g:actual_curwin"), true);
  }

  return result;
}


/// Call home_replace + trans_characters to fill the provided buffer.
void nvim_stl_home_replace_trans(buf_T *buf, const char *src, char *dst, int dstlen)
{
  home_replace(buf, src, dst, dstlen, true);
  trans_characters(dst, dstlen);
}





/// Set an option to empty string on error (SID_ERROR).
void nvim_stl_set_option_empty(int opt_idx, int opt_scope)
{
  set_option_direct((OptIndex)opt_idx, STATIC_CSTR_AS_OPTVAL(""), opt_scope, SID_ERROR);
}






/// Find option index by 'showcmdloc' value.
int nvim_stl_showcmd_matches_opt(int opt_idx)
{
  if (p_sc && (opt_idx < 0 || find_option(p_sloc) == (OptIndex)opt_idx)) {
    return 1;
  }
  return 0;
}

/// Get showcmd_buf contents.
const char *nvim_stl_get_showcmd_buf(void) { return showcmd_buf; }

/// Get wp->w_maxscwidth (sign column setting).
int nvim_stl_win_get_maxscwidth(win_T *wp) { return (int)wp->w_maxscwidth; }

/// Get stcp->sattrs[0].text[0] != 0 (has sign text).
int nvim_stl_stcp_has_sign_text(statuscol_T *stcp)
{
  if (stcp == NULL) { return 0; }
  return stcp->sattrs[0].text[0] ? 1 : 0;
}


/// Fill fold column into buf. Returns bytes written.
int nvim_stl_fill_foldcolumn(win_T *wp, statuscol_T *stcp, int lnum, int fdc, char *buf, int buflen)
{
  if (stcp == NULL || fdc <= 0 || buf == NULL) {
    return 0;
  }
  schar_T fold_buf[9];
  fill_foldcolumn(wp, stcp->foldinfo, lnum, 0, fdc, NULL, stcp->fold_vcol, fold_buf);
  size_t written = 0;
  for (int i = 0; i < fdc && (int)written < buflen - 4; i++) {
    written += schar_get(buf + written, fold_buf[i]);
  }
  buf[written] = NUL;
  return (int)written;
}


/// Describe sign text into a buffer. Returns bytes written.
int nvim_stl_describe_sign_text(char *buf, schar_T *text) { return (int)describe_sign_text(buf, text); }

/// Get stcp->sign_cul_id.
int nvim_stl_stcp_get_sign_cul_id(statuscol_T *stcp)
{
  if (stcp == NULL) { return 0; }
  return stcp->sign_cul_id;
}

/// Get stcp->sattrs[idx].hl_id.
int nvim_stl_stcp_get_sattr_hl_id(statuscol_T *stcp, int idx)
{
  if (stcp == NULL) { return 0; }
  return stcp->sattrs[idx].hl_id;
}


/// Get stcp->sattrs[idx].text[0] != 0.
int nvim_stl_stcp_sattr_has_text(statuscol_T *stcp, int idx)
{
  if (stcp == NULL) { return 0; }
  return stcp->sattrs[idx].text[0] ? 1 : 0;
}



_Static_assert(kOptInvalid == -1, "kOptInvalid must be -1");
_Static_assert(kOptStatuscolumn == 293, "kOptStatuscolumn");
_Static_assert(kOptStatusline == 294, "kOptStatusline");
_Static_assert(kOptTabline == 302, "kOptTabline");
_Static_assert(kOptWinbar == 355, "kOptWinbar");
_Static_assert(kOptRulerformat == 241, "kOptRulerformat");
_Static_assert(MODE_INSERT == 0x10, "MODE_INSERT");
_Static_assert(VV_LNUM == 9, "VV_LNUM");
_Static_assert(VV_RELNUM == 101, "VV_RELNUM");
_Static_assert(VV_VIRTNUM == 102, "VV_VIRTNUM");
_Static_assert(SCL_NUM == -2, "SCL_NUM");
_Static_assert(EOL_MAC == 2, "EOL_MAC");
_Static_assert(SID_ERROR == -5, "SID_ERROR");
_Static_assert(NUL == 0, "NUL");
_Static_assert(NL == 10, "NL");
_Static_assert(CAR == 13, "CAR");
_Static_assert(HLF_CLF == 17, "HLF_CLF");
_Static_assert(HLF_FC == 29, "HLF_FC");
_Static_assert(HLF_TPF == 54, "HLF_TPF");
_Static_assert(HLF_WBR == 65, "HLF_WBR");
_Static_assert(HLF_WBRNC == 66, "HLF_WBRNC");
_Static_assert(HLF_MSG == 63, "HLF_MSG");
_Static_assert(OPT_LOCAL == 0x02, "OPT_LOCAL");

/// Check if wildmenu is showing and UI does not have kUIWildmenu.
/// Returns true if statusline redraw should be blocked.
int nvim_stl_wildmenu_blocking(void) { return wild_menu_showing != 0 && !ui_has(kUIWildmenu); }

/// Get wp->w_p_stl (window-local statusline option).
const char *nvim_stl_win_get_p_stl(win_T *wp) { return wp->w_p_stl; }

_Static_assert(kUIWildmenu == 3, "kUIWildmenu must be 3");
_Static_assert(HLF_C == 21, "HLF_C must be 21");

/// Accessor: build arena-based API objects from flat arrays and emit
/// ui_call_tabline_update.  Called from Rust rs_ui_ext_tabline_update().
void nvim_stl_emit_tabline_update(int *tab_handles, const char **tab_names,
                                  int tab_count, int *buf_handles,
                                  const char **buf_names, int buf_count,
                                  int curtab_handle, int curbuf_handle)
{
  Arena arena = ARENA_EMPTY;

  Array tabs = arena_array(&arena, (size_t)tab_count);
  for (int i = 0; i < tab_count; i++) {
    Dict tab_info = arena_dict(&arena, 2);
    PUT_C(tab_info, "tab", TABPAGE_OBJ(tab_handles[i]));
    PUT_C(tab_info, "name", CSTR_TO_ARENA_OBJ(&arena, tab_names[i]));
    ADD_C(tabs, DICT_OBJ(tab_info));
  }

  Array buffers = arena_array(&arena, (size_t)buf_count);
  for (int i = 0; i < buf_count; i++) {
    Dict buffer_info = arena_dict(&arena, 2);
    PUT_C(buffer_info, "buffer", BUFFER_OBJ(buf_handles[i]));
    PUT_C(buffer_info, "name", CSTR_TO_ARENA_OBJ(&arena, buf_names[i]));
    ADD_C(buffers, DICT_OBJ(buffer_info));
  }

  ui_call_tabline_update(curtab_handle, tabs, curbuf_handle, buffers);
  arena_mem_free(arena_finish(&arena));
}


/// Get window winbar fill character (w_p_fcs_chars.wbr).
schar_T nvim_stl_win_get_fcs_wbr(win_T *wp) { return wp->w_p_fcs_chars.wbr; }




/// Call grid_adjust for a window's grid_alloc. Returns the grid handle.
/// Updates row and col through pointers.
void *nvim_stl_grid_adjust_win(win_T *wp, int *row, int *col)
{
  ScreenGrid *grid = grid_adjust(&wp->w_grid, row, col);
  return grid;
}

/// Call grid_adjust for msg_grid_adj. Returns the grid handle.
void *nvim_stl_grid_adjust_msg(int *row, int *col)
{
  ScreenGrid *grid = grid_adjust(&msg_grid_adj, row, col);
  return grid;
}




/// Get HL_ATTR value.
int nvim_stl_HL_ATTR(int hlf) { return HL_ATTR((hlf_T)hlf); }



/// Build a UI msg_ruler content chunk and call ui_call_msg_ruler.
/// Takes arrays of (attr, text, tsize, group) tuples.
/// When count == 0, emits an empty array (clears the ruler).
void nvim_stl_ui_call_msg_ruler_content(int *attrs, const char **texts, size_t *tsizes,
                                        int *groups, int count)
{
  Array content = ARRAY_DICT_INIT;
  for (int i = 0; i < count; i++) {
    Array chunk = ARRAY_DICT_INIT;
    ADD(chunk, INTEGER_OBJ(attrs[i]));
    ADD(chunk, STRING_OBJ(cbuf_as_string(xmemdupz(texts[i], tsizes[i]), tsizes[i])));
    ADD(chunk, INTEGER_OBJ(groups[i]));
    ADD(content, ARRAY_OBJ(chunk));
  }
  ui_call_msg_ruler(content);
  api_free_array(content);
}



/// Get tab_page_click_defs pointer.
void *nvim_stl_get_tab_page_click_defs(void) { return tab_page_click_defs; }

/// Get wp->w_status_click_defs.
void *nvim_stl_win_get_status_click_defs(win_T *wp) { return wp->w_status_click_defs; }
size_t nvim_stl_win_get_status_click_defs_size(win_T *wp) { return wp->w_status_click_defs_size; }
void nvim_stl_win_set_status_click_defs(win_T *wp, void *defs) { wp->w_status_click_defs = defs; }
void nvim_stl_win_set_status_click_defs_size(win_T *wp, size_t size) { wp->w_status_click_defs_size = size; }

/// Get wp->w_winbar_click_defs.
void *nvim_stl_win_get_winbar_click_defs(win_T *wp) { return wp->w_winbar_click_defs; }
size_t nvim_stl_win_get_winbar_click_defs_size(win_T *wp) { return wp->w_winbar_click_defs_size; }
void nvim_stl_win_set_winbar_click_defs(win_T *wp, void *defs) { wp->w_winbar_click_defs = defs; }
void nvim_stl_win_set_winbar_click_defs_size(win_T *wp, size_t size) { wp->w_winbar_click_defs_size = size; }







/// Get edit_submode != NULL (check if in edit submode).
int nvim_stl_edit_submode_not_null(void) { return edit_submode != NULL ? 1 : 0; }


/// Call getvvcol and return the cursor virtual column.
int nvim_stl_getvvcol_cursor(win_T *wp)
{
  colnr_T virtcol;
  getvvcol(wp, &wp->w_cursor, NULL, &virtcol, NULL);
  return (int)virtcol;
}


/// Start grid line on msg_grid_adj at given row.
void nvim_stl_msg_grid_line_start(int row) { grid_line_start(&msg_grid_adj, row); }

_Static_assert(ML_EMPTY == 0x01, "ML_EMPTY must be 0x01");
_Static_assert(kUIMessages == 4, "kUIMessages must be 4");



/// Set redraw_tabline flag.
void nvim_stl_set_redraw_tabline(int val) { redraw_tabline = val ? true : false; }



/// Start grid line on default_gridview at given row.
void nvim_stl_default_grid_line_start(int row) { grid_line_start(&default_gridview, row); }


/// Get tab_page_click_defs_size.
size_t nvim_stl_get_tab_page_click_defs_size(void) { return tab_page_click_defs_size; }

/// Set a tab_page_click_def entry.
void nvim_stl_set_tab_click_def(int col, int click_type, int tabnr)
{
  tab_page_click_defs[col] = (StlClickDefinition) {
    .type = click_type,
    .tabnr = tabnr,
    .func = NULL,
  };
}

/// Get p_sc (showcmd option).
int nvim_stl_get_p_sc(void) { return p_sc; }

/// Check if showcmdloc == "tabline".
int nvim_stl_showcmd_loc_is_tabline(void) { return *p_sloc == 't' ? 1 : 0; }

/// Check if tp is the current tab page.
int nvim_stl_tabpage_is_curtab(tabpage_T *tp) { return tp == curtab ? 1 : 0; }

/// Check if tp->tp_topframe == topframe.
int nvim_stl_tabpage_topframe_matches(tabpage_T *tp) { return tp->tp_topframe == topframe ? 1 : 0; }


_Static_assert(HLF_T == 23, "HLF_T must be 23");
_Static_assert(HLF_TP == 52, "HLF_TP must be 52");
_Static_assert(HLF_TPS == 53, "HLF_TPS must be 53");
_Static_assert(kStlClickTabSwitch == 1, "kStlClickTabSwitch must be 1");
_Static_assert(kStlClickTabClose == 2, "kStlClickTabClose must be 2");
_Static_assert(kUITabline == 2, "kUITabline must be 2");

