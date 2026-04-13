#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/charset.h"
#include "nvim/indent.h"
#include "nvim/plines.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/buffer_updates.h"
#include "nvim/change.h"
#include "nvim/cursor.h"
#include "nvim/diff.h"
#include "nvim/edit.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/extmark.h"
#include "nvim/extmark_defs.h"
#include "nvim/fold.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/insexpand.h"
#include "nvim/macros_defs.h"
#include "nvim/mark.h"
#include "nvim/marktree.h"
#include "nvim/marktree_defs.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memline.h"
#include "nvim/memline_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/pos_defs.h"
#include "nvim/search.h"
#include "nvim/state.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/textformat.h"
#include "nvim/types_defs.h"
#include "nvim/drawscreen.h"
#include "nvim/ui.h"
#include "nvim/undo.h"
#include "nvim/vim_defs.h"

bool nvim_buf_get_b_may_swap(buf_T *buf) { return buf->b_may_swap; }
bool nvim_buf_get_b_p_eol(buf_T *buf) { return buf->b_p_eol; }
linenr_T nvim_buf_get_b_ml_ml_line_count(buf_T *buf) { return buf->b_ml.ml_line_count; }

bool nvim_buf_get_b_p_fixeol(buf_T *buf) { return buf->b_p_fixeol; }
const char *nvim_buf_get_b_p_fenc(buf_T *buf) { return buf->b_p_fenc; }
int nvim_buf_marktree_n_keys(buf_T *buf) { return (int)buf->b_marktree->n_keys; }
int nvim_buf_meta_total(buf_T *buf, int meta_type) { return (int)buf_meta_total(buf, meta_type); }

bool nvim_get_autocmd_busy(void) { return autocmd_busy; }
void apply_autocmds_filechangedro(buf_T *buf) { apply_autocmds(EVENT_FILECHANGEDRO, NULL, NULL, false, buf); }
int nvim_get_highlight_match(void) { return highlight_match; }
int nvim_curbufIsChanged(void) { return curbufIsChanged(); }
int nvim_msg_silent(void) { return msg_silent; }
bool nvim_silent_mode(void) { return silent_mode; }
bool nvim_in_assert_fails(void) { return in_assert_fails; }

char *nvim_ml_get(linenr_T lnum) { return ml_get(lnum); }

const char *nvim_gettext(const char *s) { return _(s); }

char *nvim_get_cursor_line_ptr(void) { return get_cursor_line_ptr(); }
int nvim_utf_char2bytes(int c, char *buf) { return utf_char2bytes(c, buf); }
colnr_T nvim_ml_get_len(linenr_T lnum) { return ml_get_len(lnum); }

// Curwin cursor accessors (used by open_line.rs)
pos_T nvim_change_get_curwin_cursor(void) { return curwin->w_cursor; }
void nvim_set_curwin_cursor(pos_T pos) { curwin->w_cursor = pos; }
void nvim_set_curwin_cursor_lnum(linenr_T lnum) { curwin->w_cursor.lnum = lnum; }
colnr_T nvim_get_curwin_cursor_col(void) { return curwin->w_cursor.col; }
void nvim_set_curwin_cursor_col(colnr_T col) { curwin->w_cursor.col = col; }
void nvim_set_curwin_cursor_coladd(colnr_T coladd) { curwin->w_cursor.coladd = coladd; }

bool nvim_p_sm(void) { return p_sm; }
bool nvim_p_ri(void) { return p_ri; }
bool nvim_p_deco(void) { return p_deco; }
bool nvim_p_paste(void) { return p_paste; }
bool nvim_p_sr(void) { return p_sr; }

void nvim_set_state(int state) { State = state; }

bool nvim_vim_strchr_cpo_listwm(void) { return vim_strchr(p_cpo, CPO_LISTWM) != NULL; }
bool nvim_vim_strchr_cpo_dollar(void) { return vim_strchr(p_cpo, CPO_DOLLAR) != NULL; }

int nvim_curbuf_get_ml_line_count(void) { return curbuf->b_ml.ml_line_count; }
colnr_T nvim_curbuf_get_ml_line_len(void) { return curbuf->b_ml.ml_line_len; }
void nvim_curbuf_set_ml_line_len(colnr_T len) { curbuf->b_ml.ml_line_len = len; }
char *nvim_curbuf_get_ml_line_ptr(void) { return curbuf->b_ml.ml_line_ptr; }
linenr_T nvim_curbuf_get_b_ml_ml_line_count(void) { return curbuf->b_ml.ml_line_count; }
bool nvim_curbuf_get_b_p_cin(void) { return curbuf->b_p_cin; }
bool nvim_curbuf_get_b_p_lisp(void) { return curbuf->b_p_lisp; }
bool nvim_curbuf_get_b_p_pi(void) { return curbuf->b_p_pi; }
colnr_T nvim_curbuf_get_b_p_ts(void) { return curbuf->b_p_ts; }
int64_t nvim_curbuf_get_b_p_sw(void) { return curbuf->b_p_sw; }
const colnr_T *nvim_curbuf_get_b_p_vts_array(void) { return curbuf->b_p_vts_array; }
const char *nvim_curbuf_get_b_p_inde_ptr(void) { return curbuf->b_p_inde; }
char *nvim_curbuf_get_b_p_com(void) { return curbuf->b_p_com; }

/// Iterator for FOR_ALL_TAB_WINDOWS - returns an opaque handle.
/// Usage: start → next (until NULL) → end.
typedef struct {
  tabpage_T *tp;
  win_T *wp;
  bool started;
} TabWinIter;

void *nvim_for_all_tab_windows_start(void)
{
  TabWinIter *iter = xmalloc(sizeof(TabWinIter));
  iter->tp = first_tabpage;
  iter->wp = (iter->tp != NULL) ? ((iter->tp == curtab) ? firstwin : iter->tp->tp_firstwin) : NULL;
  iter->started = false;
  return iter;
}

win_T *nvim_for_all_tab_windows_next(void *handle)
{
  TabWinIter *iter = (TabWinIter *)handle;

  if (!iter->started) {
    iter->started = true;
    if (iter->wp != NULL) {
      return iter->wp;
    }
  }

  // Advance to next window
  if (iter->wp != NULL) {
    iter->wp = iter->wp->w_next;
  }

  while (iter->wp == NULL) {
    // Move to next tabpage
    iter->tp = iter->tp->tp_next;
    if (iter->tp == NULL) {
      return NULL;
    }
    iter->wp = (iter->tp == curtab) ? firstwin : iter->tp->tp_firstwin;
  }
  return iter->wp;
}

win_T *nvim_curtab_first_win(void) { return firstwin; }
win_T *nvim_win_get_next_in_tab(win_T *wp) { return wp->w_next; }

bool nvim_win_get_lines_wl_valid(win_T *wp, int idx) { return wp->w_lines[idx].wl_valid; }
void nvim_win_set_lines_wl_valid(win_T *wp, int idx, bool val) { wp->w_lines[idx].wl_valid = val; }
linenr_T nvim_win_get_lines_wl_lnum(win_T *wp, int idx) { return wp->w_lines[idx].wl_lnum; }
void nvim_win_set_lines_wl_lnum(win_T *wp, int idx, linenr_T val) { wp->w_lines[idx].wl_lnum = val; }
linenr_T nvim_win_get_lines_wl_foldend(win_T *wp, int idx) { return wp->w_lines[idx].wl_foldend; }
void nvim_win_set_lines_wl_foldend(win_T *wp, int idx, linenr_T val) { wp->w_lines[idx].wl_foldend = val; }
linenr_T nvim_win_get_lines_wl_lastlnum(win_T *wp, int idx) { return wp->w_lines[idx].wl_lastlnum; }
void nvim_win_set_lines_wl_lastlnum(win_T *wp, int idx, linenr_T val) { wp->w_lines[idx].wl_lastlnum = val; }

void nvim_u_clearline(void) { u_clearline(curbuf); }

bool nvim_get_did_si(void) { return did_si; }
void nvim_set_did_si(bool val) { did_si = val; }
bool nvim_get_can_si(void) { return can_si; }
void nvim_set_can_si(bool val) { can_si = val; }
bool nvim_get_can_si_back(void) { return can_si_back; }
void nvim_set_can_si_back(bool val) { can_si_back = val; }
bool nvim_get_did_ai(void) { return did_ai; }
void nvim_set_did_ai(bool val) { did_ai = val; }
colnr_T nvim_get_ai_col(void) { return ai_col; }
void nvim_set_ai_col(colnr_T val) { ai_col = val; }
int nvim_get_end_comment_pending(void) { return end_comment_pending; }
void nvim_set_end_comment_pending(int val) { end_comment_pending = val; }
linenr_T nvim_get_orig_line_count(void) { return orig_line_count; }
void nvim_set_orig_line_count(linenr_T val) { orig_line_count = val; }
pos_T nvim_get_insstart(void) { return Insstart; }
int nvim_get_vr_lines_changed(void) { return vr_lines_changed; }
void nvim_set_vr_lines_changed(int val) { vr_lines_changed = val; }
int nvim_get_inhibit_delete_count(void) { return inhibit_delete_count; }
void nvim_set_inhibit_delete_count(int val) { inhibit_delete_count = val; }

void nvim_set_cmdmod_cmod_flags(int val) { cmdmod.cmod_flags = val; }

pos_T *nvim_findmatch(char *initc, char ch) { return findmatch((oparg_T *)initc, ch); }
int nvim_change_get_sw_value(void) { return get_sw_value(curbuf); }
bool nvim_change_bt_prompt(void) { return bt_prompt(curbuf); }
linenr_T nvim_get_curbuf_b_prompt_start_mark_lnum(void) { return curbuf->b_prompt_start.mark.lnum; }

// =============================================================================
// Phase 1: Accessors for changed_common migration
// =============================================================================

// Global state accessors
bool nvim_get_redraw_not_allowed(void) { return redraw_not_allowed; }
bool nvim_get_VIsual_active_bool(void) { return VIsual_active; }
linenr_T nvim_get_search_hl_has_cursor_lnum(void) { return search_hl_has_cursor_lnum; }
void nvim_set_search_hl_has_cursor_lnum(linenr_T val) { search_hl_has_cursor_lnum = val; }

// curtab diff update
void nvim_curtab_set_diff_update(bool val) { curtab->tp_diff_update = val; }

// Compound helper: last_cursormoved reset check for changed_common lines 302-306
void nvim_change_last_cursormoved_reset_check(buf_T *buf, linenr_T lnum, linenr_T lnume,
                                               linenr_T xtra)
{
  if (last_cursormoved_win == curwin && curwin->w_buffer == buf
      && lnum <= curwin->w_cursor.lnum
      && lnume + (xtra < 0 ? -xtra : xtra) > curwin->w_cursor.lnum) {
    last_cursormoved.lnum = 0;
  }
}

// Compound helper: changelist/mark update block (lines 141-212 of changed_common)
void nvim_changed_common_update_changelist(buf_T *buf, linenr_T lnum, colnr_T col)
{
  fmarkv_T view = INIT_FMARKV;
  // Set the markview only if lnum is visible in curwin
  if (curwin->w_buffer == buf) {
    if (lnum >= curwin->w_topline && lnum <= curwin->w_botline) {
      view = mark_view_make(curwin->w_topline, curwin->w_cursor);
    }
  }
  RESET_FMARK(&buf->b_last_change, ((pos_T) { lnum, col, 0 }), buf->handle, view);

  if (buf->b_new_change || buf->b_changelistlen == 0) {
    bool add;
    if (buf->b_changelistlen == 0) {
      add = true;
    } else {
      pos_T *p = &buf->b_changelist[buf->b_changelistlen - 1].mark;
      if (p->lnum != lnum) {
        add = true;
      } else {
        int cols = comp_textwidth(false);
        if (cols == 0) {
          cols = 79;
        }
        add = (p->col + cols < col || col + cols < p->col);
      }
    }
    if (add) {
      buf->b_new_change = false;
      if (buf->b_changelistlen == JUMPLISTSIZE) {
        buf->b_changelistlen = JUMPLISTSIZE - 1;
        memmove(buf->b_changelist, buf->b_changelist + 1,
                sizeof(buf->b_changelist[0]) * (JUMPLISTSIZE - 1));
        FOR_ALL_TAB_WINDOWS(tp, wp) {
          if (wp->w_buffer == buf && wp->w_changelistidx > 0) {
            wp->w_changelistidx--;
          }
        }
      }
      FOR_ALL_TAB_WINDOWS(tp, wp) {
        if (wp->w_buffer == buf && wp->w_changelistidx == buf->b_changelistlen) {
          wp->w_changelistidx++;
        }
      }
      buf->b_changelistlen++;
    }
  }
  buf->b_changelist[buf->b_changelistlen - 1] = buf->b_last_change;
  if (curwin->w_buffer == buf) {
    curwin->w_changelistidx = buf->b_changelistlen;
  }
}
