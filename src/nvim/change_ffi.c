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

void nvim_buf_set_b_changed(buf_T *buf, bool val) { buf->b_changed = val; }
bool nvim_buf_get_b_may_swap(buf_T *buf) { return buf->b_may_swap; }
bool nvim_buf_get_b_p_eol(buf_T *buf) { return buf->b_p_eol; }
int nvim_buf_get_b_flags(buf_T *buf) { return buf->b_flags; }
void nvim_buf_set_b_flags(buf_T *buf, int val) { buf->b_flags = val; }
linenr_T nvim_buf_get_b_ml_ml_line_count(buf_T *buf) { return buf->b_ml.ml_line_count; }

linenr_T nvim_buf_get_b_mod_top(buf_T *buf) { return buf->b_mod_top; }
linenr_T nvim_buf_get_b_mod_bot(buf_T *buf) { return buf->b_mod_bot; }
linenr_T nvim_buf_get_b_mod_xlines(buf_T *buf) { return buf->b_mod_xlines; }

bool nvim_buf_get_b_p_bin(buf_T *buf) { return buf->b_p_bin; }
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
colnr_T nvim_ml_get_len(linenr_T lnum) { return ml_get_len(lnum); }
int nvim_ml_replace(linenr_T lnum, char *line, bool copy) { return ml_replace(lnum, line, copy); }
int nvim_ml_append(linenr_T lnum, const char *line, colnr_T len, bool newfile)
{
  return ml_append(lnum, (char *)line, len, newfile);
}

void nvim_buf_inc_changedtick(buf_T *buf) { buf_inc_changedtick(buf); }

char *nvim_xstrnsave(const char *s, size_t len) { return xstrnsave(s, len); }
char *nvim_concat_str(const char *s1, const char *s2) { return concat_str(s1, s2); }
const char *nvim_gettext(const char *s) { return _(s); }

int nvim_utf_char2bytes(int c, char *buf) { return utf_char2bytes(c, buf); }
int nvim_utf_ptr2len(const char *p) { return utf_ptr2len(p); }
bool nvim_utf_composinglike(const char *p0, const char *p1, uint64_t *state)
{
  return utf_composinglike(p0, p1, (GraphemeState *)state);
}
int nvim_ptr2cells(const char *p) { return ptr2cells(p); }

char *nvim_get_cursor_line_ptr(void) { return get_cursor_line_ptr(); }
void nvim_replace_push(const char *ptr, size_t len) { replace_push(ptr, len); }
bool nvim_use_indentexpr_for_lisp(void) { return use_indentexpr_for_lisp(); }

// Curwin cursor accessors (used by open_line.rs)
pos_T nvim_change_get_curwin_cursor(void) { return curwin->w_cursor; }
void nvim_set_curwin_cursor(pos_T pos) { curwin->w_cursor = pos; }
void nvim_set_curwin_cursor_lnum(linenr_T lnum) { curwin->w_cursor.lnum = lnum; }
colnr_T nvim_get_curwin_cursor_col(void) { return curwin->w_cursor.col; }
void nvim_set_curwin_cursor_col(colnr_T col) { curwin->w_cursor.col = col; }
void nvim_set_curwin_cursor_coladd(colnr_T coladd) { curwin->w_cursor.coladd = coladd; }

void nvim_replace_push_nul(void) { replace_push_nul(); }

bool nvim_p_sm(void) { return p_sm; }
bool nvim_p_ri(void) { return p_ri; }
bool nvim_p_deco(void) { return p_deco; }
bool nvim_p_paste(void) { return p_paste; }
bool nvim_p_sr(void) { return p_sr; }

void nvim_set_state(int state) { State = state; }

colnr_T nvim_win_chartabsize(win_T *wp, const char *ptr, colnr_T vcol)
{
  return win_chartabsize(wp, (char *)ptr, vcol);
}

bool nvim_vim_strchr_cpo_listwm(void) { return vim_strchr(p_cpo, CPO_LISTWM) != NULL; }
bool nvim_vim_strchr_cpo_dollar(void) { return vim_strchr(p_cpo, CPO_DOLLAR) != NULL; }

void nvim_siemsg(const char *s, int64_t arg) { siemsg(s, arg); }
void nvim_siemsg_str(const char *s, const char *arg) { siemsg(s, arg); }

int nvim_curbuf_get_ml_line_count(void) { return curbuf->b_ml.ml_line_count; }
colnr_T nvim_curbuf_get_ml_line_len(void) { return curbuf->b_ml.ml_line_len; }
void nvim_curbuf_set_ml_line_len(colnr_T len) { curbuf->b_ml.ml_line_len = len; }
char *nvim_curbuf_get_ml_line_ptr(void) { return curbuf->b_ml.ml_line_ptr; }
linenr_T nvim_curbuf_get_b_ml_ml_line_count(void) { return curbuf->b_ml.ml_line_count; }
bool nvim_curbuf_get_b_p_ci(void) { return curbuf->b_p_ci; }
bool nvim_curbuf_get_b_p_cin(void) { return curbuf->b_p_cin; }
bool nvim_curbuf_get_b_p_lisp(void) { return curbuf->b_p_lisp; }
bool nvim_curbuf_get_b_p_pi(void) { return curbuf->b_p_pi; }
void nvim_curbuf_set_b_p_pi(bool val) { curbuf->b_p_pi = val; }
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

void nvim_for_all_tab_windows_end(void *handle) { xfree(handle); }

win_T *nvim_curtab_first_win(void) { return firstwin; }
win_T *nvim_win_get_next_in_tab(win_T *wp) { return wp->w_next; }

void nvim_changed_line_abv_curs_win(win_T *wp) { changed_line_abv_curs_win(wp); }
void nvim_approximate_botline_win(win_T *wp) { approximate_botline_win(wp); }

bool nvim_win_get_lines_wl_valid(win_T *wp, int idx) { return wp->w_lines[idx].wl_valid; }
void nvim_win_set_lines_wl_valid(win_T *wp, int idx, bool val) { wp->w_lines[idx].wl_valid = val; }
linenr_T nvim_win_get_lines_wl_lnum(win_T *wp, int idx) { return wp->w_lines[idx].wl_lnum; }
void nvim_win_set_lines_wl_lnum(win_T *wp, int idx, linenr_T val) { wp->w_lines[idx].wl_lnum = val; }
linenr_T nvim_win_get_lines_wl_foldend(win_T *wp, int idx) { return wp->w_lines[idx].wl_foldend; }
void nvim_win_set_lines_wl_foldend(win_T *wp, int idx, linenr_T val) { wp->w_lines[idx].wl_foldend = val; }
linenr_T nvim_win_get_lines_wl_lastlnum(win_T *wp, int idx) { return wp->w_lines[idx].wl_lastlnum; }
void nvim_win_set_lines_wl_lastlnum(win_T *wp, int idx, linenr_T val) { wp->w_lines[idx].wl_lastlnum = val; }

void nvim_extmark_splice_cols(buf_T *buf, int start_row, colnr_T start_col,
                              colnr_T old_col, colnr_T new_col, int undo)
{
  extmark_splice_cols(buf, start_row, start_col, old_col, new_col, undo);
}

void nvim_extmark_adjust(buf_T *buf, linenr_T line1, linenr_T line2,
                         linenr_T amount, linenr_T amount_after, int op)
{
  extmark_adjust(buf, line1, line2, amount, amount_after, op);
}

void nvim_extmark_splice(buf_T *buf, int start_row, colnr_T start_col,
                         int old_row, colnr_T old_col, bcount_t old_byte,
                         int new_row, colnr_T new_col, bcount_t new_byte, int undo)
{
  extmark_splice(buf, start_row, start_col, old_row, old_col, old_byte,
                 new_row, new_col, new_byte, undo);
}

void nvim_mark_adjust(linenr_T line1, linenr_T line2,
                      linenr_T amount, linenr_T amount_after, int op)
{
  mark_adjust(line1, line2, amount, amount_after, op);
}

void nvim_mark_col_adjust(linenr_T lnum, colnr_T col,
                          linenr_T amount_lnum, colnr_T amount_col, int spaces_removed)
{
  mark_col_adjust(lnum, col, amount_lnum, amount_col, spaces_removed);
}

void nvim_u_clearline(void) { u_clearline(curbuf); }
int nvim_u_save_cursor(void) { return u_save_cursor(); }

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

bool nvim_has_format_option(int opt) { return has_format_option(opt); }
pos_T *nvim_findmatch(char *initc, char ch) { return findmatch((oparg_T *)initc, ch); }
int nvim_indent_size_ts(const char *ptr, colnr_T ts, const colnr_T *vts_array)
{
  return indent_size_ts(ptr, ts, (colnr_T *)vts_array);
}
int nvim_get_indent(void) { return get_indent(); }
bool nvim_set_indent(int size, int flags) { return set_indent(size, flags); }
int nvim_change_get_sw_value(void) { return get_sw_value(curbuf); }
int nvim_get_leader_len(const char *line, char **flags, bool backward, bool include_space)
{
  return get_leader_len((char *)line, flags, backward, include_space);
}
colnr_T nvim_check_linecomment(const char *line) { return check_linecomment(line); }
size_t nvim_change_copy_option_part(char **option, char *buf, int maxlen, const char *sep)
{
  return copy_option_part(option, buf, maxlen, (char *)sep);
}

const char *nvim_prompt_text(void) { return prompt_text(); }
bool nvim_change_bt_prompt(void) { return bt_prompt(curbuf); }
linenr_T nvim_get_curbuf_b_prompt_start_mark_lnum(void) { return curbuf->b_prompt_start.mark.lnum; }

void nvim_ins_bytes(const char *p) { ins_bytes((char *)p); }

char *nvim_ml_get_buf(buf_T *buf, linenr_T lnum) { return ml_get_buf(buf, lnum); }

void nvim_changed_lines(buf_T *buf, linenr_T first, int col, linenr_T last, linenr_T xtra,
                        bool add_undo)
{
  changed_lines(buf, first, col, last, xtra, add_undo);
}

void nvim_buf_updates_send_changes(buf_T *buf, linenr_T firstlnum, int64_t num_added,
                                   int64_t num_removed)
{
  buf_updates_send_changes(buf, firstlnum, num_added, num_removed);
}

int nvim_get_last_leader_offset(const char *line, char **flags) { return get_last_leader_offset((char *)line, flags); }

// =============================================================================
// Phase 1: Accessors for changed_common migration
// =============================================================================

// Window last_cursor_lnum_rnu accessors (not yet in Rust window crate)
linenr_T nvim_win_get_last_cursor_lnum_rnu(win_T *wp) { return wp->w_last_cursor_lnum_rnu; }
void nvim_win_set_last_cursor_lnum_rnu(win_T *wp, linenr_T val) { wp->w_last_cursor_lnum_rnu = val; }

// Window last_cursorline accessors (not yet in Rust window crate)
linenr_T nvim_win_get_last_cursorline(win_T *wp) { return wp->w_last_cursorline; }
void nvim_win_set_last_cursorline(win_T *wp, linenr_T val) { wp->w_last_cursorline = val; }

// Wrappers for functions that need window context
int nvim_change_linetabsize_eol(win_T *wp, linenr_T lnum) { return linetabsize_eol(wp, lnum); }
int nvim_change_sms_marker_overlap(win_T *wp, int extra2) { return sms_marker_overlap(wp, extra2); }
void nvim_change_set_topline(win_T *wp, linenr_T topline) { set_topline(wp, topline); }

// Global state accessors
bool nvim_get_redraw_not_allowed(void) { return redraw_not_allowed; }
bool nvim_get_VIsual_active_bool(void) { return VIsual_active; }
void nvim_change_check_visual_pos(void) { check_visual_pos(); }
void nvim_change_set_must_redraw(int type) { set_must_redraw(type); }
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
