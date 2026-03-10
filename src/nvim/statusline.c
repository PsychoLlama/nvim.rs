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

// ============================================================================
// Statusline Accessor Functions (for Rust FFI)
// ============================================================================

/// Evaluate an expression for the statusline.
/// Returns the length of the result string.
int nvim_stl_eval_expr(win_T *wp, const char *expr, int expr_len, char *out, int out_len)
{
  if (wp == NULL || expr == NULL || out == NULL || out_len <= 0) {
    out[0] = '\0';
    return 0;
  }

  // Create a null-terminated copy of the expression
  char *expr_copy = xmemdupz(expr, (size_t)expr_len);

  // Evaluate the expression
  char *result = eval_to_string(expr_copy, true, false);
  xfree(expr_copy);

  if (result == NULL) {
    out[0] = '\0';
    return 0;
  }

  // Copy result to output buffer
  int len = (int)strlen(result);
  if (len >= out_len) {
    len = out_len - 1;
  }
  memcpy(out, result, (size_t)len);
  out[len] = '\0';

  xfree(result);
  return len;
}

/// Get highlight group ID by name.
int nvim_syn_name2id(const char *name)
{
  if (name == NULL) {
    return 0;
  }
  return syn_name2id(name);
}

/// Get byte value at cursor position in window.
int nvim_stl_get_byte_value(win_T *wp)
{
  if (wp == NULL || wp->w_buffer == NULL) {
    return 0;
  }
  char *line = ml_get_buf(wp->w_buffer, wp->w_cursor.lnum);
  if (line == NULL) {
    return 0;
  }
  colnr_T col = wp->w_cursor.col;
  if (col >= ml_get_buf_len(wp->w_buffer, wp->w_cursor.lnum)) {
    return 0;
  }
  return (uint8_t)line[col];
}

/// Get byte offset at cursor position in window.
int nvim_stl_get_byte_offset(win_T *wp)
{
  if (wp == NULL || wp->w_buffer == NULL) {
    return 0;
  }
  // Use ml_find_line_or_offset which is O(log n) for byte offset calculation
  int l = rs_ml_find_line_or_offset(wp->w_buffer, wp->w_cursor.lnum, NULL, false);
  if (wp->w_buffer->b_ml.ml_flags & ML_EMPTY) {
    return 0;
  }
  if (l < 0) {
    return 0;
  }
  // Add column position, but handle empty lines
  bool empty_line = ml_get_buf_len(wp->w_buffer, wp->w_cursor.lnum) == 0;
  int col_offset = ((State & MODE_INSERT) == 0 && empty_line) ? 0 : (int)wp->w_cursor.col;
  return l + 1 + col_offset;
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

// Phase 1 accessors for Rust FFI

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


/// Set v:lnum variable.
void nvim_stl_set_vv_lnum(int64_t lnum)
{
  set_vim_var_nr(VV_LNUM, lnum);
}

/// Set v:relnum variable.
void nvim_stl_set_vv_relnum(int64_t relnum)
{
  set_vim_var_nr(VV_RELNUM, relnum);
}

/// Get wp->w_p_stc (statuscolumn option).
const char *nvim_stl_win_get_p_stc(win_T *wp)
{
  return wp->w_p_stc;
}

/// Call build_stl_str_hl for statuscolumn rendering.
/// Returns width.
int nvim_stl_build_stl_str_hl(win_T *wp, char *buf, int buflen, const char *stc,
                               int maxwidth, stl_hlrec_t **hlrec, StlClickRecord **clickrec,
                               statuscol_T *stcp)
{
  // build_stl_str_hl requires a mutable copy of the format string
  char *stc_copy = xstrdup(stc);
  int width = build_stl_str_hl(wp, buf, (size_t)buflen, stc_copy, (OptIndex)kOptStatuscolumn,
                                OPT_LOCAL, 0, maxwidth, hlrec, NULL, clickrec, stcp);
  xfree(stc_copy);
  return width;
}

/// Get window statuscol click defs pointer.
StlClickDefinition *nvim_stl_win_get_statuscol_click_defs(win_T *wp)
{
  return wp->w_statuscol_click_defs;
}

/// Get window statuscol click defs size.
size_t nvim_stl_win_get_statuscol_click_defs_size(win_T *wp)
{
  return wp->w_statuscol_click_defs_size;
}

/// Set window statuscol click defs.
void nvim_stl_win_set_statuscol_click_defs(win_T *wp, StlClickDefinition *defs)
{
  wp->w_statuscol_click_defs = defs;
}

/// Set window statuscol click defs size.
void nvim_stl_win_set_statuscol_click_defs_size(win_T *wp, size_t size)
{
  wp->w_statuscol_click_defs_size = size;
}

/// Get stcp->width.
int nvim_stl_stcp_get_width(statuscol_T *stcp)
{
  return stcp->width;
}

/// Get stcp->hlrec pointer address (for passing to build_stl_str_hl).
stl_hlrec_t **nvim_stl_stcp_get_hlrec_ptr(statuscol_T *stcp)
{
  return &stcp->hlrec;
}



_Static_assert(OPT_LOCAL == 0x02, "OPT_LOCAL must be 0x02");

// Phase 6 accessors: build_stl_str_hl format parsing support

/// Evaluate a VimL expression for the statusline with full context switching.
/// Sets g:actual_curbuf, g:actual_curwin, switches curwin/curbuf, saves VIsual_active.
/// Returns allocated string result (caller must free via nvim_stl_xfree), or NULL.
char *nvim_stl_eval_expr_full(win_T *wp, char *expr, bool use_sandbox)
{
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

  char *result = eval_to_string_safe(expr, use_sandbox, false);

  curwin = save_curwin;
  curbuf = save_curbuf;
  VIsual_active = save_VIsual_active;

  do_unlet(S_LEN("g:actual_curbuf"), true);
  do_unlet(S_LEN("g:actual_curwin"), true);

  return result;
}

/// Evaluate "%!" expression format: set g:statusline_winid, call eval_to_string_safe,
/// clean up. Returns allocated string result (caller must free), or NULL.
char *nvim_stl_eval_fmt_expr(win_T *wp, char *expr, bool use_sandbox)
{
  typval_T tv = {
    .v_type = VAR_NUMBER,
    .vval.v_number = wp->handle,
  };
  set_var(S_LEN("g:statusline_winid"), &tv, false);
  char *result = eval_to_string_safe(expr, use_sandbox, false);
  do_unlet(S_LEN("g:statusline_winid"), true);
  return result;
}

/// Check if the option was set insecurely.
int nvim_stl_was_set_insecurely(win_T *wp, int opt_idx, int opt_scope)
{
  if (opt_idx < 0) {
    return 0;
  }
  return was_set_insecurely(wp, (OptIndex)opt_idx, opt_scope) ? 1 : 0;
}

/// Get buf_spname result, or NULL.
const char *nvim_stl_buf_spname(buf_T *buf)
{
  return buf_spname(buf);
}

/// Call home_replace + trans_characters to fill the provided buffer.
void nvim_stl_home_replace_trans(buf_T *buf, const char *src, char *dst, int dstlen)
{
  home_replace(buf, src, dst, dstlen, true);
  trans_characters(dst, dstlen);
}


/// Get cursor line text pointer and length.
/// Returns pointer to line text, sets *len_out to length.
const char *nvim_stl_get_cursor_line(win_T *wp, int *len_out)
{
  linenr_T lnum = wp->w_cursor.lnum;
  if (lnum > wp->w_buffer->b_ml.ml_line_count) {
    lnum = wp->w_buffer->b_ml.ml_line_count;
  }
  const char *line = ml_get_buf(wp->w_buffer, lnum);
  if (len_out) {
    *len_out = (int)ml_get_buf_len(wp->w_buffer, lnum);
  }
  return line;
}

/// Clamp cursor to line length if needed.
void nvim_stl_clamp_cursor(win_T *wp)
{
  linenr_T lnum = wp->w_cursor.lnum;
  if (lnum > wp->w_buffer->b_ml.ml_line_count) {
    lnum = wp->w_buffer->b_ml.ml_line_count;
    wp->w_cursor.lnum = lnum;
  }
  colnr_T len = ml_get_buf_len(wp->w_buffer, lnum);
  if (wp->w_cursor.col > len) {
    wp->w_cursor.col = len;
    wp->w_cursor.coladd = 0;
  }
}

/// Get global state: updating_screen flag.
int nvim_stl_get_updating_screen(void)
{
  return updating_screen ? 1 : 0;
}

/// Set global state: redraw_not_allowed flag.
void nvim_stl_set_redraw_not_allowed(int val)
{
  redraw_not_allowed = val ? true : false;
}

/// Get global state: redraw_not_allowed flag.
int nvim_stl_get_redraw_not_allowed(void)
{
  return redraw_not_allowed ? 1 : 0;
}

/// Save and get KeyTyped value.
int nvim_stl_get_KeyTyped(void)
{
  return KeyTyped ? 1 : 0;
}

/// Set KeyTyped value.
void nvim_stl_set_KeyTyped(int val)
{
  KeyTyped = val ? true : false;
}


/// Set an option to empty string on error (SID_ERROR).
void nvim_stl_set_option_empty(int opt_idx, int opt_scope)
{
  set_option_direct((OptIndex)opt_idx, STATIC_CSTR_AS_OPTVAL(""), opt_scope, SID_ERROR);
}


/// Get the ML_EMPTY flag for a buffer.
int nvim_stl_buf_ml_empty(buf_T *buf)
{
  return (buf->b_ml.ml_flags & ML_EMPTY) ? 1 : 0;
}

/// Get window cursor lnum (clamped to line count).
int nvim_stl_win_get_clamped_lnum(win_T *wp)
{
  linenr_T lnum = wp->w_cursor.lnum;
  if (lnum > wp->w_buffer->b_ml.ml_line_count) {
    lnum = wp->w_buffer->b_ml.ml_line_count;
  }
  return (int)lnum;
}



/// Get the relative position string ("Top", "Bot", "All", or "NN%").
int nvim_stl_get_rel_pos(win_T *wp, char *buf, int buflen)
{
  get_rel_pos(wp, buf, buflen);
  return (int)strlen(buf);
}

/// Get argument number string (e.g. "(2 of 5)").
int nvim_stl_append_arg_number(win_T *wp, char *buf, int buflen)
{
  buf[0] = NUL;
  return append_arg_number(wp, buf, (size_t)buflen);
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
const char *nvim_stl_get_showcmd_buf(void)
{
  return showcmd_buf;
}

/// Get vim_var_nr value.
int64_t nvim_stl_get_vim_var_nr(int vv_idx)
{
  return get_vim_var_nr(vv_idx);
}


/// Get wp->w_maxscwidth (sign column setting).
int nvim_stl_win_get_maxscwidth(win_T *wp)
{
  return (int)wp->w_maxscwidth;
}

/// Get stcp->sattrs[0].text[0] != 0 (has sign text).
int nvim_stl_stcp_has_sign_text(statuscol_T *stcp)
{
  if (stcp == NULL) { return 0; }
  return stcp->sattrs[0].text[0] ? 1 : 0;
}

/// Compute fold column width.
int nvim_stl_compute_foldcolumn(win_T *wp)
{
  return compute_foldcolumn(wp, 0);
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

/// Check use_cursor_line_highlight for a line.
int nvim_stl_use_cursor_line_hl(win_T *wp, int lnum)
{
  return use_cursor_line_highlight(wp, (linenr_T)lnum) ? 1 : 0;
}

/// Describe sign text into a buffer. Returns bytes written.
int nvim_stl_describe_sign_text(char *buf, schar_T *text)
{
  return (int)describe_sign_text(buf, text);
}

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

/// Get stcp->sattrs[idx].text pointer.
const schar_T *nvim_stl_stcp_get_sattr_text(statuscol_T *stcp, int idx)
{
  if (stcp == NULL) { return NULL; }
  return stcp->sattrs[idx].text;
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

// Phase 2 accessors for Rust FFI


/// Check if wildmenu is showing and UI does not have kUIWildmenu.
/// Returns true if statusline redraw should be blocked.
int nvim_stl_wildmenu_blocking(void)
{
  return wild_menu_showing != 0 && !ui_has(kUIWildmenu);
}

/// Get global p_wbr (winbar) option string.
const char *nvim_stl_get_p_wbr(void)
{
  return p_wbr;
}

/// Get wp->w_p_stl (window-local statusline option).
const char *nvim_stl_win_get_p_stl(win_T *wp)
{
  return wp->w_p_stl;
}

/// Get global p_stl (statusline option).
const char *nvim_stl_get_p_stl(void)
{
  return p_stl;
}

_Static_assert(kUIWildmenu == 3, "kUIWildmenu must be 3");
_Static_assert(HLF_C == 21, "HLF_C must be 21");

// Phase 3 accessors for Rust FFI

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


/// Get global ru_col.
int nvim_stl_get_ru_col(void) { return ru_col; }

/// Get global p_tal (tabline option).
char *nvim_stl_get_p_tal(void) { return p_tal; }

/// Get global p_ruf (ruler format option).
char *nvim_stl_get_p_ruf(void) { return p_ruf; }




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



/// Call win_hl_attr.
int nvim_stl_win_hl_attr(win_T *wp, int hlf) { return win_hl_attr(wp, hlf); }


/// Get HL_ATTR value.
int nvim_stl_HL_ATTR(int hlf) { return HL_ATTR((hlf_T)hlf); }



/// Build a UI msg_ruler content chunk and call ui_call_msg_ruler.
/// Takes arrays of (attr, text, tsize, group) tuples.
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


// Phase 4 accessors for redraw_ruler Rust FFI

/// Get p_ru (ruler option).
int nvim_stl_get_p_ru(void) { return p_ru ? 1 : 0; }



/// Call ui_call_msg_ruler with empty array (to clear ruler).
void nvim_stl_ui_call_msg_ruler_empty(void) { ui_call_msg_ruler((Array)ARRAY_DICT_INIT); }



/// Get edit_submode != NULL (check if in edit submode).
int nvim_stl_edit_submode_not_null(void) { return edit_submode != NULL ? 1 : 0; }


/// Call getvvcol and return the cursor virtual column.
int nvim_stl_getvvcol_cursor(win_T *wp)
{
  colnr_T virtcol;
  getvvcol(wp, &wp->w_cursor, NULL, &virtcol, NULL);
  return (int)virtcol;
}

/// Get first char of cursor line in buffer (0 if empty).
int nvim_stl_ml_get_buf_first_char(win_T *wp)
{
  return (int)(uint8_t)(*ml_get_buf(wp->w_buffer, wp->w_cursor.lnum));
}


/// Check if cursor lnum > line count (invalid position).
int nvim_stl_win_cursor_invalid(win_T *wp)
{
  return wp->w_cursor.lnum > wp->w_buffer->b_ml.ml_line_count ? 1 : 0;
}

/// Start grid line on msg_grid_adj at given row.
void nvim_stl_msg_grid_line_start(int row)
{
  grid_line_start(&msg_grid_adj, row);
}

_Static_assert(MODE_INSERT == 0x10, "MODE_INSERT must be 0x10");
_Static_assert(ML_EMPTY == 0x01, "ML_EMPTY must be 0x01");
_Static_assert(kUIMessages == 4, "kUIMessages must be 4");

// Phase 5 accessors for draw_tabline Rust FFI


/// Set redraw_tabline flag.
void nvim_stl_set_redraw_tabline(int val) { redraw_tabline = val ? true : false; }



/// Start grid line on default_gridview at given row.
void nvim_stl_default_grid_line_start(int row)
{
  grid_line_start(&default_gridview, row);
}


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

/// Per-tab info for Rust draw_tabline.
typedef struct {
  win_T *cwp;          // current window in this tab
  int wincount;        // number of focusable, non-hidden windows
  bool modified;       // any buffer changed?
  bool is_curtab;      // is this the active tab?
  bool topframe_match; // tp->tp_topframe == topframe
  char name[MAXPATHL]; // shortened buffer name
  int name_len;        // display width of name
} TabInfo;

/// Collect all tab page info into a flat array.
/// Returns the number of tabs (written to *out_count).
/// The caller must free the result with xfree().
void *nvim_stl_collect_tab_info(int *out_count)
{
  int tabcount = 0;
  FOR_ALL_TABS(tp) {
    tabcount++;
  }
  *out_count = tabcount;
  if (tabcount == 0) {
    return NULL;
  }

  TabInfo *tabs = xcalloc((size_t)tabcount, sizeof(TabInfo));
  int idx = 0;
  FOR_ALL_TABS(tp) {
    TabInfo *t = &tabs[idx++];
    t->is_curtab = (tp == curtab);
    t->topframe_match = (tp->tp_topframe == topframe);

    if (tp == curtab) {
      t->cwp = curwin;
    } else {
      t->cwp = tp->tp_curwin;
    }

    win_T *wp = t->is_curtab ? firstwin : tp->tp_firstwin;
    t->wincount = 0;
    t->modified = false;
    for (; wp != NULL; wp = wp->w_next) {
      if (!wp->w_config.focusable || wp->w_config.hide) {
        // skip non-focusable/hidden windows
      } else {
        t->wincount++;
        if (bufIsChanged(wp->w_buffer)) {
          t->modified = true;
        }
      }
    }

    // Get buffer name
    get_trans_bufname(t->cwp->w_buffer);
    shorten_dir(NameBuff);
    xstrlcpy(t->name, NameBuff, sizeof(t->name));
    t->name_len = vim_strsize(t->name);
  }

  return tabs;
}

/// Get cwp from TabInfo (opaque pointer).
win_T *nvim_stl_tab_info_get_cwp(void *ptr) { return ((TabInfo *)ptr)->cwp; }

/// Get wincount from TabInfo.
int nvim_stl_tab_info_get_wincount(void *ptr) { return ((TabInfo *)ptr)->wincount; }

/// Get modified from TabInfo.
int nvim_stl_tab_info_get_modified(void *ptr) { return ((TabInfo *)ptr)->modified ? 1 : 0; }

/// Get is_curtab from TabInfo.
int nvim_stl_tab_info_get_is_curtab(void *ptr) { return ((TabInfo *)ptr)->is_curtab ? 1 : 0; }

/// Get topframe_match from TabInfo.
int nvim_stl_tab_info_get_topframe_match(void *ptr) { return ((TabInfo *)ptr)->topframe_match ? 1 : 0; }

/// Get name from TabInfo.
const char *nvim_stl_tab_info_get_name(void *ptr) { return ((TabInfo *)ptr)->name; }

/// Get name display width from TabInfo.
int nvim_stl_tab_info_get_name_len(void *ptr) { return ((TabInfo *)ptr)->name_len; }

/// Get size of TabInfo struct for Rust array iteration.
size_t nvim_stl_tab_info_size(void) { return sizeof(TabInfo); }


_Static_assert(HLF_T == 23, "HLF_T must be 23");
_Static_assert(HLF_TP == 52, "HLF_TP must be 52");
_Static_assert(HLF_TPS == 53, "HLF_TPS must be 53");
_Static_assert(HLF_TPF == 54, "HLF_TPF must be 54");
_Static_assert(kStlClickTabSwitch == 1, "kStlClickTabSwitch must be 1");
_Static_assert(kStlClickTabClose == 2, "kStlClickTabClose must be 2");
_Static_assert(kUITabline == 2, "kUITabline must be 2");

