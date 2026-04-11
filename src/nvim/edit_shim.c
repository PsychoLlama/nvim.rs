// edit_shim.c: Rust FFI accessors for edit crate.
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <ctype.h>
#include "nvim/api/private/defs.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/buffer.h"
#include "nvim/change.h"
#include "nvim/charset.h"
#include "nvim/cursor.h"
#include "nvim/digraph.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_docmd.h"
#include "nvim/fold.h"
#include "nvim/getchar.h"
#include "nvim/globals.h"
#include "nvim/indent.h"
#include "nvim/indent_c.h"
#include "nvim/keycodes.h"
#include "nvim/mapping.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/plines.h"
#include "nvim/register.h"
#include "nvim/popupmenu.h"
#include "nvim/state.h"
#include "nvim/undo.h"
#include "nvim/statusline.h"
#include "nvim/strings.h"
#include "nvim/syntax_bridge.h"
#include "nvim/textformat.h"
#include "nvim/grid.h"
#include "nvim/highlight.h"
#include "nvim/highlight_defs.h"
#include "nvim/ui.h"
#include "edit_shim.c.generated.h"
extern int ins_apply_autocmds(event_T event);
extern int echeck_abbr(int c);
extern void rs_replace_stack_clear(void);
extern int rs_ctrl_x_mode_scroll(void);
extern void insert_special(int c, int allow_modmask, int ctrlv);
// Rust-owned statics used by stop_insert / ins_eol
extern int nvim_get_new_insert_skip(void);
extern int nvim_get_did_restart_edit(void);
extern void nvim_set_last_insert_skip(int val);
extern int nvim_get_ins_need_undo(void);
extern void nvim_clear_last_insert(void);
extern void nvim_set_last_insert(char *data, size_t size);
extern void nvim_set_can_cindent(int val);
extern void rs_foldOpenCursor(void);
extern int nvim_get_revins_on(void);
extern int nvim_get_revins_chars(void);
extern void nvim_set_revins_chars(int val);
extern int nvim_get_revins_legal(void);
extern void nvim_set_revins_legal(int val);
bool nvim_p_cpo_has_backspace(void) { return vim_strchr(p_cpo, CPO_BACKSPACE) != NULL; }
bool nvim_p_cpo_has_replcnt(void) { return vim_strchr(p_cpo, CPO_REPLCNT) != NULL; }
bool nvim_cmod_keepjumps(void) { return (cmdmod.cmod_flags & CMOD_KEEPJUMPS) != 0; }
void nvim_do_cmdline_getcmdkeycmd(void) { do_cmdline(NULL, getcmdkeycmd, NULL, 0); }
void nvim_ins_insert(int replaceState)
{
  set_vim_var_string(VV_INSERTMODE, ((State & REPLACE_FLAG)
                                     ? "i"
                                     : replaceState == MODE_VREPLACE ? "v" : "r"), 1);
  ins_apply_autocmds(EVENT_INSERTCHANGE);
  if (State & REPLACE_FLAG) {
    State = MODE_INSERT | (State & MODE_LANGMAP);
  } else {
    State = replaceState | (State & MODE_LANGMAP);
  }
  may_trigger_modechanged();
  AppendCharToRedobuff(K_INS);
  showmode();
  ui_cursor_shape();
}
void nvim_ins_ctrl_o(void)
{
  restart_VIsual_select = 0;
  if (State & VREPLACE_FLAG) {
    restart_edit = 'V';
  } else if (State & REPLACE_FLAG) {
    restart_edit = 'R';
  } else {
    restart_edit = 'I';
  }
  if (virtual_active(curwin)) {
    ins_at_eol = false;
  } else {
    ins_at_eol = (gchar_cursor() == NUL);
  }
}
void nvim_ins_ctrl_hat(void)
{
  if (map_to_exists_mode("", MODE_LANGMAP, false)) {
    if (State & MODE_LANGMAP) {
      curbuf->b_p_iminsert = B_IMODE_NONE;
      State &= ~MODE_LANGMAP;
    } else {
      curbuf->b_p_iminsert = B_IMODE_LMAP;
      State |= MODE_LANGMAP;
    }
  }
  set_iminsert_global(curbuf);
  showmode();
  status_redraw_curbuf();
}
void nvim_init_Insstart(int startln)
{
  if (where_paste_started.lnum != 0) {
    Insstart = where_paste_started;
  } else {
    Insstart = curwin->w_cursor;
    if (startln) {
      Insstart.col = 0;
    }
  }
}
int nvim_get_inserted_size(void)
{
  String inserted = get_inserted();
  int sz = (int)inserted.size;
  if (inserted.data != NULL) {
    xfree(inserted.data);
  }
  return sz;
}
extern int rs_get_scrolloff_value(win_T *wp);
int nvim_insert_check_scroll(int mincol, linenr_T old_topline, int old_topfill,
                              int did_backspace, int count)
{
  if (!curbuf->b_mod_set || !curwin->w_p_wrap || curwin->w_p_sms
      || did_backspace || curwin->w_topline != old_topline
      || curwin->w_topfill != old_topfill || count > 1) {
    return -1;
  }
  int new_mincol = curwin->w_wcol;
  validate_cursor_col(curwin);
  if (curwin->w_wcol < new_mincol - tabstop_at(get_nolist_virtcol(),
                                                curbuf->b_p_ts,
                                                curbuf->b_p_vts_array,
                                                false)
      && curwin->w_wrow == curwin->w_view_height - 1 - rs_get_scrolloff_value(curwin)
      && (curwin->w_cursor.lnum != curwin->w_topline || curwin->w_topfill > 0)) {
    if (curwin->w_topfill > 0) {
      curwin->w_topfill--;
    } else if (hasFolding(curwin, curwin->w_topline, NULL, &old_topline)) {
      set_topline(curwin, old_topline + 1);
    } else {
      set_topline(curwin, curwin->w_topline + 1);
    }
  }
  return new_mincol;
}
int nvim_ins_copychar(linenr_T lnum)
{
  if (lnum < 1 || lnum > curbuf->b_ml.ml_line_count) {
    vim_beep(kOptBoFlagCopy);
    return NUL;
  }
  validate_virtcol(curwin);
  int const end_vcol = curwin->w_virtcol;
  char *line = ml_get(lnum);
  CharsizeArg csarg;
  CSType cstype = init_charsize_arg(&csarg, curwin, lnum, line);
  StrCharInfo ci = utf_ptr2StrCharInfo(line);
  int vcol = 0;
  while (vcol < end_vcol && *ci.ptr != NUL) {
    vcol += win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &csarg).width;
    if (vcol > end_vcol) {
      break;
    }
    ci = utfc_next(ci);
  }
  int c = ci.chr.value < 0 ? (uint8_t)(*ci.ptr) : ci.chr.value;
  if (c == NUL) {
    vim_beep(kOptBoFlagCopy);
  }
  return c;
}
extern void rs_start_selection(void);
int nvim_ins_start_select(int c)
{
  if (!km_startsel) {
    return 0;
  }
  switch (c) {
  case K_KHOME:
  case K_KEND:
  case K_PAGEUP:
  case K_KPAGEUP:
  case K_PAGEDOWN:
  case K_KPAGEDOWN:
    if (!(mod_mask & MOD_MASK_SHIFT)) {
      break;
    }
    FALLTHROUGH;
  case K_S_LEFT:
  case K_S_RIGHT:
  case K_S_UP:
  case K_S_DOWN:
  case K_S_END:
  case K_S_HOME:
    rs_start_selection();
    stuffcharReadbuff(Ctrl_O);
    if (mod_mask) {
      const char buf[] = { (char)K_SPECIAL, (char)KS_MODIFIER,
                           (char)(uint8_t)mod_mask, NUL };
      stuffReadbuffLen(buf, 3);
    }
    stuffcharReadbuff(c);
    return 1;
  }
  return 0;
}
void nvim_handle_end_comment_pending(int c)
{
  char *p;
  char lead_end[COM_MAX_LEN];

  char *line = get_cursor_line_ptr();
  int i = get_leader_len(line, &p, false, true);
  if (i > 0 && vim_strchr(p, COM_MIDDLE) != NULL) {
    while (*p && p[-1] != ':') {
      p++;
    }
    int middle_len = (int)copy_option_part(&p, lead_end, COM_MAX_LEN, ",");
    while (middle_len > 0 && ascii_iswhite(lead_end[middle_len - 1])) {
      middle_len--;
    }
    while (*p && p[-1] != ':') {
      p++;
    }
    int end_len = (int)copy_option_part(&p, lead_end, COM_MAX_LEN, ",");
    i = curwin->w_cursor.col;
    while (--i >= 0 && ascii_iswhite(line[i])) {}
    i++;
    i -= middle_len;
    if (i >= 0 && end_len > 0
        && (uint8_t)lead_end[end_len - 1] == end_comment_pending) {
      backspace_until_column(i);
      ins_bytes_len(lead_end, (size_t)(end_len - 1));
    }
  }
}
extern int rs_ins_compl_col(void);
extern void start_arrow_with_change(pos_T *end_insert_pos, bool end_change);
extern void nvim_set_o_lnum(linenr_T val);
int nvim_merge_modifiers(int c) { return merge_modifiers(c, &mod_mask); }
int nvim_MB_BYTE2LEN_CHECK(int c) { return MB_BYTE2LEN_CHECK(c); }
int nvim_get_K_ZERO(void) { return K_ZERO; }
char *nvim_get_special_key_name(int c, int modifiers) { return get_special_key_name(c, modifiers); }
int nvim_comp_textwidth(int ff) { return comp_textwidth((bool)ff); }
void nvim_internal_format(int textwidth, int second_indent, int flags, int format_only, int c) { internal_format(textwidth, second_indent, flags, (bool)format_only, c); }
int nvim_byte2cells(int b) { return byte2cells((uint8_t)b); }
int nvim_mb_get_class_cursor(void) { return mb_get_class(get_cursor_pos_ptr()); }
int nvim_cursor_has_composing(void) { if (!p_deco) { return 0; } char *p0 = get_cursor_pos_ptr(); return utf_composinglike(p0, p0 + utf_ptr2len(p0), NULL) ? 1 : 0; }
void *nvim_get_yank_register_paste(int regname) { return get_yank_register(regname, YREG_PASTE); }
int nvim_insert_reg(int regname, int literally) { return insert_reg(regname, NULL, literally != 0); }
bool nvim_is_literal_register(int regname) { return is_literal_register(regname); }
size_t nvim_reg_y_size(void *reg) { return ((yankreg_T *)reg)->y_size; }
int nvim_curbuf_meta_total_inline(void) { return buf_meta_total(curbuf, kMTMetaInline); }
int nvim_get_p_ch_zero_no_ui_messages(void) { return (p_ch == 0 && !ui_has(kUIMessages)) ? 1 : 0; }
int nvim_has_event_insertcharpre(void) { return has_event(EVENT_INSERTCHARPRE) ? 1 : 0; }
int nvim_pagescroll_backward(void) { return pagescroll(BACKWARD, 1, false); }
int nvim_pagescroll_forward(void) { return pagescroll(FORWARD, 1, false); }
void nvim_map_execute_lua_false(void) { map_execute_lua(false, false); }
void nvim_auto_format_ins(int force_format) { auto_format(false, force_format != 0); }
int nvim_get_need_highlight_changed(void) { return need_highlight_changed ? 1 : 0; }
void nvim_set_need_start_insertmode(int val) { need_start_insertmode = (val != 0); }
void nvim_state_enter(void *state) { state_enter((VimState *)state); }
int nvim_ww_allows(int ch) { return vim_strchr(p_ww, (char)ch) != NULL ? 1 : 0; }
int nvim_vv_char_is_empty(void) { return (*get_vim_var_str(VV_CHAR) == NUL) ? 1 : 0; }
int nvim_cursor_on_tab_or_inline(void) { return (gchar_cursor() == TAB || buf_meta_total(curbuf, kMTMetaInline) > 0) ? 1 : 0; }
void nvim_set_vv_insertmode(int cmdchar) { const char *ptr = cmdchar == 'R' ? "r" : cmdchar == 'V' ? "v" : "i"; set_vim_var_string(VV_INSERTMODE, ptr, 1); }
int nvim_cursor_col_ge_compl_col(void) { return curwin->w_cursor.col >= rs_ins_compl_col() ? 1 : 0; }
void nvim_change_warning_col(int col) { change_warning(curbuf, col); }
void nvim_check_cursor_col_insert_mode(void) { int save_state = State; State = MODE_INSERT; check_cursor_col(curwin); State = save_state; }
void nvim_coladvance_insstart(void) { coladvance(curwin, getvcol_nolist(&Insstart)); }
int nvim_cursor_equals_saved(linenr_T lnum, colnr_T col, colnr_T coladd) { pos_T saved = { .lnum = lnum, .col = col, .coladd = coladd }; return equalpos(curwin->w_cursor, saved) ? 1 : 0; }
int nvim_in_cinkeys_int(int c, int type, int line_is_white) { return in_cinkeys(c, (char)type, line_is_white != 0) ? 1 : 0; }
int nvim_insstart_col_gt_orig(void) { return Insstart.col > Insstart_orig.col ? 1 : 0; }
colnr_T nvim_linetabsize_cursor_line(void) { return linetabsize_str(get_cursor_line_ptr()); }
void nvim_restore_cursor_pos(linenr_T lnum, colnr_T col, colnr_T coladd) { curwin->w_cursor.lnum = lnum; curwin->w_cursor.col = col; curwin->w_cursor.coladd = coladd; }
void nvim_save_cursor_pos(linenr_T *lnum_out, colnr_T *col_out, colnr_T *coladd_out) { *lnum_out = curwin->w_cursor.lnum; *col_out = curwin->w_cursor.col; *coladd_out = curwin->w_cursor.coladd; }
void nvim_set_vim_var_char(const char *buf, ptrdiff_t len) { set_vim_var_string(VV_CHAR, buf, len); }
void nvim_start_arrow_curpos(void) { start_arrow(&curwin->w_cursor); }
void nvim_start_arrow_with_change_curpos(bool end_change) { start_arrow_with_change(&curwin->w_cursor, end_change); }
void nvim_ui_cursor_shape_and_clear_digraph(void) { ui_cursor_shape(); do_digraph(-1); }
void nvim_clear_where_paste_started(void) { where_paste_started.lnum = 0; }
void nvim_update_o_lnum_if_at_eol(void) { if (ins_at_eol) { nvim_set_o_lnum(curwin->w_cursor.lnum); } }
const char *nvim_get_vim_var_char(void) { return get_vim_var_str(VV_CHAR); }

// ============================================================================
// Phase 3: functions moved from edit.c
// ============================================================================

/// Stop insert mode: save inserted text, auto-format, strip trailing whitespace.
/// Moved from edit.c; identical logic.
void nvim_edit_stop_insert(void *end_insert_pos, int esc, int nomove)
{
  pos_T *pos = (pos_T *)end_insert_pos;
  stop_redo_ins();
  rs_replace_stack_clear();

  // Save inserted text for redo (^@ / CTRL-A).
  String inserted = get_inserted();
  int added = inserted.data == NULL ? 0 : (int)inserted.size - nvim_get_new_insert_skip();
  if (nvim_get_did_restart_edit() == 0 || added > 0) {
    nvim_clear_last_insert();
    nvim_set_last_insert(inserted.data, inserted.size);
    nvim_set_last_insert_skip(added < 0 ? 0 : nvim_get_new_insert_skip());
  } else {
    xfree(inserted.data);
  }

  if (!arrow_used && pos != NULL) {
    int cc;
    if (!nvim_get_ins_need_undo() && has_format_option(FO_AUTO)) {
      pos_T tpos = curwin->w_cursor;
      cc = 'x';
      if (curwin->w_cursor.col > 0 && gchar_cursor() == NUL) {
        dec_cursor();
        cc = gchar_cursor();
        if (!ascii_iswhite(cc)) {
          curwin->w_cursor = tpos;
        }
      }
      auto_format(true, false);
      if (ascii_iswhite(cc)) {
        if (gchar_cursor() != NUL) {
          inc_cursor();
        }
        if (gchar_cursor() == NUL
            && curwin->w_cursor.lnum == tpos.lnum
            && curwin->w_cursor.col == tpos.col) {
          curwin->w_cursor.coladd = tpos.coladd;
        }
      }
    }
    check_auto_format(true);
    if (!nomove && did_ai && (esc || (vim_strchr(p_cpo, CPO_INDENT) == NULL
                                      && curwin->w_cursor.lnum != pos->lnum))
        && pos->lnum <= curbuf->b_ml.ml_line_count) {
      pos_T tpos = curwin->w_cursor;
      colnr_T prev_col = pos->col;
      curwin->w_cursor = *pos;
      check_cursor_col(curwin);
      while (true) {
        if (gchar_cursor() == NUL && curwin->w_cursor.col > 0) {
          curwin->w_cursor.col--;
        }
        cc = gchar_cursor();
        if (!ascii_iswhite(cc)) {
          break;
        }
        if (del_char(true) == FAIL) {
          break;
        }
      }
      if (curwin->w_cursor.lnum != tpos.lnum) {
        curwin->w_cursor = tpos;
      } else if (curwin->w_cursor.col < prev_col) {
        tpos = curwin->w_cursor;
        tpos.col++;
        if (cc != NUL && gchar_pos(&tpos) == NUL) {
          curwin->w_cursor.col++;
        }
      }
      if (VIsual_active) {
        check_visual_pos();
      }
    }
  }

  did_ai = false;
  did_si = false;
  can_si = false;
  can_si_back = false;
  if (pos != NULL) {
    curbuf->b_op_start = Insstart;
    curbuf->b_op_start_orig = Insstart_orig;
    curbuf->b_op_end = *pos;
  }
}

/// Handle CR/NL insertion in insert mode.  Moved from edit.c.
int nvim_edit_ins_eol(int c)
{
  if (echeck_abbr(c + ABBR_OFF)) {
    return true;
  }
  if (stop_arrow() == FAIL) {
    return false;
  }
  undisplay_dollar();

  if ((State & REPLACE_FLAG) && !(State & VREPLACE_FLAG)) {
    replace_push_nul();
  }

  if (virtual_active(curwin) && curwin->w_cursor.coladd > 0) {
    coladvance(curwin, getviscol());
  }

  if (nvim_get_revins_on()) {
    curwin->w_cursor.col += get_cursor_pos_len();
  }

  AppendToRedobuff(NL_STR);
  bool i = open_line(FORWARD,
                     has_format_option(FO_RET_COMS) ? OPENLINE_DO_COM : 0,
                     old_indent, NULL);
  old_indent = 0;
  nvim_set_can_cindent(1);
  rs_foldOpenCursor();

  return i;
}

/// Handle Ctrl-Y/Ctrl-E (copy char above/below) in insert mode.  Moved from edit.c.
int nvim_edit_ins_ctrl_ey(int tc)
{
  int c = tc;

  if (rs_ctrl_x_mode_scroll()) {
    if (c == Ctrl_Y) {
      scrolldown_clamp();
    } else {
      scrollup_clamp();
    }
    redraw_later(curwin, UPD_VALID);
  } else {
    c = ins_copychar(curwin->w_cursor.lnum + (c == Ctrl_Y ? -1 : 1));
    if (c != NUL) {
      if (c < 256 && !isalnum(c)) {
        AppendToRedobuff(CTRL_V_STR);
      }
      OptInt tw_save = curbuf->b_p_tw;
      curbuf->b_p_tw = -1;
      insert_special(c, true, false);
      curbuf->b_p_tw = tw_save;
      nvim_set_revins_chars(nvim_get_revins_chars() + 1);
      nvim_set_revins_legal(nvim_get_revins_legal() + 1);
      c = Ctrl_V;
      auto_format(false, true);
    }
  }
  return c;
}

/// Toggle p_ri (reverse insert) and return the new value.
void nvim_toggle_p_ri(void) { p_ri = !p_ri; }

/// Advance cursor column by 1 (curwin->w_cursor.col++).
void nvim_curwin_cursor_col_inc(void) { curwin->w_cursor.col++; }

/// Compute the target columns for softtabstop backspace.
///
/// Iterates the current line up to the cursor, tracking virtual columns, and
/// returns:
///   want_col_out  -- the buffer column to delete back to
///   start_vcol_out -- the virtual column at want_col (start of insert loop)
///   want_vcol_out  -- the target virtual column (end of insert loop)
///
/// Used by the Rust ins_bs_softtabstop implementation.
void nvim_ins_bs_softtabstop_want_col(bool in_indent, colnr_T *want_col_out,
                                       colnr_T *start_vcol_out,
                                       colnr_T *want_vcol_out)
{
  bool const use_ts = !curwin->w_p_list || curwin->w_p_lcs_chars.tab1;
  char *const line = get_cursor_line_ptr();
  char *const cursor_ptr = line + curwin->w_cursor.col;

  colnr_T vcol = 0;
  colnr_T space_vcol = 0;
  StrCharInfo sci = utf_ptr2StrCharInfo(line);
  StrCharInfo space_sci = sci;
  bool prev_space = false;

  while (sci.ptr < cursor_ptr) {
    bool cur_space = ascii_iswhite(sci.chr.value);
    if (!prev_space && cur_space) {
      space_sci = sci;
      space_vcol = vcol;
    }
    vcol += charsize_nowrap(curbuf, sci.ptr, use_ts, vcol, sci.chr.value);
    sci = utfc_next(sci);
    prev_space = cur_space;
  }

  colnr_T want_vcol = vcol > 0 ? vcol - 1 : 0;
  if (p_sta && in_indent) {
    want_vcol -= want_vcol % get_sw_value(curbuf);
  } else {
    want_vcol = tabstop_start(want_vcol, get_sts_value(), curbuf->b_p_vsts_array);
  }

  while (true) {
    int size = charsize_nowrap(curbuf, space_sci.ptr, use_ts, space_vcol, space_sci.chr.value);
    if (space_vcol + size > want_vcol) {
      break;
    }
    space_vcol += size;
    space_sci = utfc_next(space_sci);
  }

  *want_col_out = (colnr_T)(space_sci.ptr - line);
  // Insertion loop starts at space_vcol and inserts until want_vcol
  *start_vcol_out = space_vcol;
  *want_vcol_out = want_vcol;
}

// ============================================================================
// ins_redraw accessors for Rust redraw.rs
// ============================================================================

/// True when CursorMovedI should fire (cursor moved and popup not visible).
bool nvim_ins_redraw_cursormoved_pending(void)
{
  return has_event(EVENT_CURSORMOVEDI)
    && (last_cursormoved_win != curwin
        || !equalpos(last_cursormoved, curwin->w_cursor))
    && !pum_visible();
}

/// True when syntax highlighting is present in curwin and must_redraw is set.
bool nvim_ins_redraw_syntax_must_redraw(void) { return syntax_present(curwin) && must_redraw; }

/// Trigger CursorMovedI and update last_cursormoved tracking.
void nvim_ins_redraw_trigger_cursormovedi(void)
{
  update_curswant();
  ins_apply_autocmds(EVENT_CURSORMOVEDI);
  last_cursormoved_win = curwin;
  last_cursormoved = curwin->w_cursor;
}

/// True when b_last_changedtick_i differs from current changedtick (not popup).
bool nvim_curbuf_textchangedi_pending(void)
{
  return curbuf->b_last_changedtick_i != buf_get_changedtick(curbuf) && !pum_visible();
}

/// Apply TextChangedI autocmds, sync changedtick, u_save if tick changed.
void nvim_edit_apply_textchangedi(void)
{
  aco_save_T aco;
  varnumber_T tick = buf_get_changedtick(curbuf);
  aucmd_prepbuf(&aco, curbuf);
  apply_autocmds(EVENT_TEXTCHANGEDI, NULL, NULL, false, curbuf);
  aucmd_restbuf(&aco);
  curbuf->b_last_changedtick_i = buf_get_changedtick(curbuf);
  if (tick != buf_get_changedtick(curbuf)) {
    u_save(curwin->w_cursor.lnum, (linenr_T)(curwin->w_cursor.lnum + 1));
  }
}

/// True when b_last_changedtick_pum differs from current changedtick (popup visible).
bool nvim_curbuf_textchangedp_pending(void)
{
  return curbuf->b_last_changedtick_pum != buf_get_changedtick(curbuf) && pum_visible();
}

/// Apply TextChangedP autocmds, sync changedtick, u_save if tick changed.
void nvim_edit_apply_textchangedp(void)
{
  aco_save_T aco;
  varnumber_T tick = buf_get_changedtick(curbuf);
  aucmd_prepbuf(&aco, curbuf);
  apply_autocmds(EVENT_TEXTCHANGEDP, NULL, NULL, false, curbuf);
  aucmd_restbuf(&aco);
  curbuf->b_last_changedtick_pum = buf_get_changedtick(curbuf);
  if (tick != buf_get_changedtick(curbuf)) {
    u_save(curwin->w_cursor.lnum, (linenr_T)(curwin->w_cursor.lnum + 1));
  }
}

/// True when BufModifiedSet should fire (b_changed_invalid set, popup not visible).
bool nvim_curbuf_bufmodifiedset_pending(void)
{
  return has_event(EVENT_BUFMODIFIEDSET) && curbuf->b_changed_invalid && !pum_visible();
}

/// Apply BufModifiedSet autocmds and clear b_changed_invalid.
void nvim_edit_apply_bufmodifiedset(void)
{
  apply_autocmds(EVENT_BUFMODIFIEDSET, NULL, NULL, false, curbuf);
  curbuf->b_changed_invalid = false;
}

/// Final screen-update sequence for ins_redraw (after all autocmd triggers).
void nvim_ins_redraw_screen_update(void)
{
  pum_check_clear();
  show_cursor_info_later(false);
  if (must_redraw) {
    update_screen();
  } else {
    redraw_statuslines();
    if (clear_cmdline || redraw_cmdline || redraw_mode) {
      showmode();
    }
  }
  setcursor();
  emsg_on_display = false;
}

// ---- edit_putchar / edit_unputchar grid accessors ----

/// Start a grid line for curwin->w_grid at the given row.
void nvim_edit_grid_line_start(int row) { grid_line_start(&curwin->w_grid, row); }

/// Get character at column from the current grid line.
/// Returns the schar_T value (uint32_t). attr_out may be NULL.
uint32_t nvim_edit_grid_line_getchar(int col, int *attr_out)
{
  return (uint32_t)grid_line_getchar(col, attr_out);
}

/// Put an schar_T character at column with attr.
void nvim_edit_grid_line_put_schar(int col, uint32_t schar, int attr)
{
  grid_line_put_schar(col, (schar_T)schar, attr);
}

/// Put a string at column with attr, returns number of columns written.
int nvim_edit_grid_line_puts(int col, const char *buf, int len, int attr)
{
  return grid_line_puts(col, buf, len, attr);
}

/// Flush the current grid line.
void nvim_edit_grid_line_flush(void) { grid_line_flush(); }

/// True when curwin->w_grid_alloc.chars is non-NULL.
bool nvim_curwin_grid_alloc_has_chars(void) { return curwin->w_grid_alloc.chars != NULL; }

/// Get curwin->w_p_rl (rightleft option).
int nvim_curwin_get_p_rl(void) { return curwin->w_p_rl; }

/// HL_ATTR(HLF_8) -- highlight attr for special keys.
int nvim_hl_attr_8(void) { return HL_ATTR(HLF_8); }

/// redrawWinline(curwin, curwin->w_cursor.lnum).
void nvim_redrawwinline_cursor(void) { redrawWinline(curwin, curwin->w_cursor.lnum); }

/// schar_from_ascii(' ') -- space character as schar_T.
uint32_t nvim_schar_space(void) { return (uint32_t)schar_from_ascii(' '); }

// ---- Phase 2 accessors: display_dollar, nvim_trim_eol_space, restart_edit ----

/// Get ins_at_eol global.
bool nvim_get_ins_at_eol(void) { return ins_at_eol; }

/// Get where_paste_started.lnum.
linenr_T nvim_get_where_paste_started_lnum(void) { return where_paste_started.lnum; }

/// Call update_curswant().
void nvim_update_curswant(void) { update_curswant(); }

/// Decrement curbuf->b_ml.ml_line_len by 1.
void nvim_curbuf_dec_ml_line_len(void) { curbuf->b_ml.ml_line_len--; }

/// Call ml_get_buf_mut(curbuf, curwin->w_cursor.lnum).
char *nvim_curbuf_get_cursor_line_mut(void)
{
  return ml_get_buf_mut(curbuf, curwin->w_cursor.lnum);
}

/// Compute utf_head_off(line, line + col) for the cursor line.
/// Used by display_dollar to adjust col to first byte of multi-byte char.
int nvim_edit_utf_head_off_cursor_col(int col)
{
  const char *p = get_cursor_line_ptr();
  return utf_head_off(p, p + col);
}

/// Call curs_columns(curwin, false) -- recompute w_wrow and w_wcol.
void nvim_curs_columns_curwin_no_scroll(void) { curs_columns(curwin, false); }

// ---- Phase 3 accessors: nvim_edit_init_prompt_impl ----

/// Increment curbuf->b_prompt_start.mark.lnum by 1.
void nvim_curbuf_inc_prompt_start_lnum(void) { curbuf->b_prompt_start.mark.lnum++; }

/// Set curwin->w_cursor.lnum to curbuf->b_ml.ml_line_count.
void nvim_curwin_set_cursor_lnum_to_line_count(void)
{
  curwin->w_cursor.lnum = curbuf->b_ml.ml_line_count;
}

/// Call inserted_bytes(lnum, 0, 0, new_col) for prompt text insertion.
void nvim_inserted_bytes_prompt(linenr_T lnum, colnr_T new_col)
{
  inserted_bytes(lnum, 0, 0, new_col);
}
