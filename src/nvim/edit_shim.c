// edit_shim.c: Rust FFI accessors for edit crate.

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include "nvim/ascii_defs.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/change.h"
#include "nvim/charset.h"
#include "nvim/cursor.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_docmd.h"
#include "nvim/fold.h"
#include "nvim/getchar.h"
#include "nvim/globals.h"
#include "nvim/indent.h"
#include "nvim/keycodes.h"
#include "nvim/mapping.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/plines.h"
#include "nvim/state.h"
#include "nvim/statusline.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/vim_defs.h"

#include "edit_shim.c.generated.h"

// General cpo/cmdmod accessors (migrated from ex_cmds_shim.c)
bool nvim_p_cpo_has_backspace(void) { return vim_strchr(p_cpo, CPO_BACKSPACE) != NULL; }
bool nvim_p_cpo_has_replcnt(void) { return vim_strchr(p_cpo, CPO_REPLCNT) != NULL; }
bool nvim_cmod_keepjumps(void) { return (cmdmod.cmod_flags & CMOD_KEEPJUMPS) != 0; }
// Insert mode command/cmdline accessor (migrated from ex_cmds_shim.c)
void nvim_do_cmdline_getcmdkeycmd(void) { do_cmdline(NULL, getcmdkeycmd, NULL, 0); }

// Multi-line insert-mode helpers (migrated from normal_shim.c)

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
