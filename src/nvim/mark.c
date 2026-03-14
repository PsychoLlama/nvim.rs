// mark.c: functions for setting marks and jumping to them

#include <assert.h>
#include <limits.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cursor.h"
#include "nvim/diff.h"
#include "nvim/edit.h"
#include "nvim/errors.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/extmark.h"
#include "nvim/extmark_defs.h"
#include "nvim/fold.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/highlight_defs.h"
#include "nvim/mark.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/normal_defs.h"
#include "nvim/option_vars.h"
#include "nvim/os/fs.h"
#include "nvim/os/input.h"
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/os/time.h"
#include "nvim/os/time_defs.h"
#include "nvim/path.h"
#include "nvim/pos_defs.h"
#include "nvim/quickfix.h"
#include "nvim/strings.h"
#include "nvim/tag.h"
#include "nvim/textobject.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"


// Rust FFI declarations (tag module)
extern void rs_tagstack_clear_entry(void *tg);

// This file contains routines to maintain and manipulate marks.

// If a named file mark's lnum is non-zero, it is valid.
// If a named file mark's fnum is non-zero, it is for an existing buffer,
// otherwise it is from .shada and namedfm[n].fname is the file name.
// There are marks 'A - 'Z (set by user) and '0 to '9 (set when writing
// shada).

// =============================================================================
// Static assertions for constants shared with Rust
// =============================================================================
_Static_assert(CMOD_KEEPJUMPS == 0x0400, "CMOD_KEEPJUMPS mismatch with Rust");
_Static_assert(kOptJopFlagStack == 0x01, "kOptJopFlagStack mismatch with Rust");
_Static_assert(JUMPLISTSIZE == 100, "JUMPLISTSIZE mismatch with Rust");
_Static_assert(TAGSTACKSIZE == 20, "TAGSTACKSIZE mismatch with Rust");
_Static_assert(CMOD_LOCKMARKS == 0x0800, "CMOD_LOCKMARKS mismatch with Rust");
_Static_assert(kMarkAdjustNormal == 0, "kMarkAdjustNormal mismatch with Rust");
_Static_assert(kMarkAdjustApi == 1, "kMarkAdjustApi mismatch with Rust");
_Static_assert(kMarkAdjustTerm == 2, "kMarkAdjustTerm mismatch with Rust");
_Static_assert(kExtmarkNOOP == 0, "kExtmarkNOOP mismatch with Rust");
_Static_assert(BUF_HAS_QF_ENTRY == 1, "BUF_HAS_QF_ENTRY mismatch with Rust");
_Static_assert(BUF_HAS_LL_ENTRY == 2, "BUF_HAS_LL_ENTRY mismatch with Rust");

// =============================================================================
// Rust FFI declarations
// =============================================================================

// Mark index functions (already used inline)

// Mark type checks (already used inline)

// Position comparison functions
extern int rs_lt(pos_T a, pos_T b);

// Position accessors
extern int rs_pos_get_col(pos_T pos);
extern void rs_pos_set_col(pos_T *pos, int col);

// Position manipulation
extern void rs_pos_clamp(pos_T *pos, int max_lnum, int max_col);

// Mark name utilities

// Mark view functions
extern fmarkv_T rs_mark_view_make(linenr_T topline, linenr_T pos_lnum);

// Mark validation functions

// fmark_T functions

// Visual mark selection

// Jumplist and changelist operations

// Mark movement functions
extern int rs_mark_move_calc_result(linenr_T prev_lnum, colnr_T prev_col,
                                     linenr_T new_lnum, colnr_T new_col, int initial_res);
extern int rs_mark_move_needs_cursor_check(int res);

// Mark adjustment result structures
typedef struct {
  linenr_T new_lnum;
  int modified;
} LineAdjustResult;

typedef struct {
  linenr_T new_lnum;
  colnr_T new_col;
  int modified;
} ColAdjustResult;

// Mark adjustment functions
extern LineAdjustResult rs_mark_adjust_lnum(linenr_T lnum, linenr_T line1, linenr_T line2,
                                             linenr_T amount, linenr_T amount_after);
extern ColAdjustResult rs_mark_col_adjust(linenr_T pos_lnum, colnr_T pos_col, linenr_T lnum,
                                           colnr_T mincol, linenr_T lnum_amount,
                                           colnr_T col_amount, int spaces_removed);

// Ex command helper structures
typedef struct {
  int from;
  int to;
  int error;
  int consumed;
} DelmarksRange;

// Ex command helper functions

// Phase 1: FFI infrastructure + memory/field operations
extern xfmark_T rs_get_raw_global_mark(int name);
extern int rs_mark_check_line_bounds(buf_T *buf, linenr_T fm_mark_lnum,
                                      const char **errormsg, const char *e_markinval_str);

// Phase 2: Simple window/buffer operations
extern void rs_ex_clearjumps(win_T *win);
extern void rs_free_all_marks(void);
extern void rs_checkpcmark(win_T *win);
extern void rs_mark_view_restore(const fmark_T *fm, win_T *win);
extern int rs_mark_check(const fmark_T *fm, const char **errormsg, buf_T *curbuf);

// Phase 3: Mark getting/setting
extern fmark_T *rs_mark_get(buf_T *buf, win_T *win, fmark_T *fmp, int flag, int name);
extern xfmark_T *rs_mark_get_global(int resolve, int name);
extern fmark_T *rs_mark_get_local(buf_T *buf, win_T *win, int name, buf_T *curbuf_ptr);
// setmark_pos stays in C due to pointer comparison (pos == &curwin->w_cursor)
extern int rs_mark_set_global(int name, xfmark_T fm, int update);
extern int rs_mark_set_local(int name, buf_T *buf, fmark_T fm, int update);
extern char *rs_fm_getname(fmark_T *fmark, int lead_len, buf_T *curbuf_ptr);

// Phase 4: Jumplist/changelist navigation
extern void rs_setpcmark(win_T *win, buf_T *buf);
extern fmark_T *rs_get_jumplist(win_T *win, buf_T *curbuf_ptr, int count);
extern void rs_cleanup_jumplist(win_T *wp, int loadfiles);
extern fmark_T *rs_getnextmark(pos_T *startpos, int dir, int begin_line, buf_T *curbuf_ptr);

// Phase 5: Mark adjustment

// Phase 6: Ex commands + remaining
extern void rs_ex_delmarks(const char *arg, int forceit, buf_T *curbuf_ptr);

// =============================================================================
// C accessor functions called from Rust
// =============================================================================

/// Wrapper around xfree for Rust to call
void nvim_mark_xfree(void *ptr)
{
  xfree(ptr);
}

/// Wrapper around xstrdup for Rust to call
char *nvim_mark_xstrdup(const char *s)
{
  return xstrdup(s);
}

/// Wrapper around os_time for Rust to call
Timestamp nvim_mark_os_time(void)
{
  return os_time();
}

/// Get pointer to the global namedfm array
xfmark_T *nvim_mark_get_namedfm(void)
{
  return namedfm;
}

/// Compare file names using path_fnamecmp
int nvim_mark_path_fnamecmp(const char *a, const char *b)
{
  return path_fnamecmp(a, b);
}

// Window jumplist accessors
int nvim_mark_win_get_jumplistlen(win_T *win) { return win->w_jumplistlen; }
void nvim_mark_win_set_jumplistlen(win_T *win, int len) { win->w_jumplistlen = len; }
int nvim_mark_win_get_jumplistidx(win_T *win) { return win->w_jumplistidx; }
void nvim_mark_win_set_jumplistidx(win_T *win, int idx) { win->w_jumplistidx = idx; }
xfmark_T *nvim_mark_win_get_jumplist_entry(win_T *win, int idx) { return &win->w_jumplist[idx]; }

// Window pcmark/cursor accessors
pos_T nvim_mark_win_get_pcmark(win_T *win) { return win->w_pcmark; }
void nvim_mark_win_set_pcmark(win_T *win, pos_T pos) { win->w_pcmark = pos; }
pos_T nvim_mark_win_get_prev_pcmark(win_T *win) { return win->w_prev_pcmark; }
void nvim_mark_win_set_prev_pcmark(win_T *win, pos_T pos) { win->w_prev_pcmark = pos; }
pos_T nvim_mark_win_get_cursor(win_T *win) { return win->w_cursor; }
buf_T *nvim_mark_win_get_buffer(win_T *win) { return win->w_buffer; }
void nvim_mark_win_set_topline(win_T *win, linenr_T topline) { set_topline(win, topline); }

// Buffer mark accessors
fmark_T *nvim_mark_buf_get_last_cursor(buf_T *buf) { return &buf->b_last_cursor; }

// Error message string accessors (with gettext)
const char *nvim_mark_get_e_umark(void) { return _(e_umark); }
const char *nvim_mark_get_e_marknotset(void) { return _(e_marknotset); }
const char *nvim_mark_get_e_markinval(void) { return _(e_markinval); }

// Clear the global namedfm array
void nvim_mark_clear_namedfm(void) { CLEAR_FIELD(namedfm); }

// Buffer mark field accessors (Phase 3)
fmark_T *nvim_mark_buf_get_namedm(buf_T *buf, int idx) { return &buf->b_namedm[idx]; }
fmark_T *nvim_mark_buf_get_last_insert(buf_T *buf) { return &buf->b_last_insert; }
fmark_T *nvim_mark_buf_get_last_change(buf_T *buf) { return &buf->b_last_change; }
pos_T *nvim_mark_buf_get_op_start(buf_T *buf) { return &buf->b_op_start; }
pos_T *nvim_mark_buf_get_op_end(buf_T *buf) { return &buf->b_op_end; }
pos_T nvim_mark_buf_get_op_start_val(buf_T *buf) { return buf->b_op_start; }
pos_T nvim_mark_buf_get_op_end_val(buf_T *buf) { return buf->b_op_end; }
pos_T nvim_mark_buf_get_visual_start(buf_T *buf) { return buf->b_visual.vi_start; }
pos_T nvim_mark_buf_get_visual_end(buf_T *buf) { return buf->b_visual.vi_end; }
pos_T *nvim_mark_buf_get_visual_start_ptr(buf_T *buf) { return &buf->b_visual.vi_start; }
pos_T *nvim_mark_buf_get_visual_end_ptr(buf_T *buf) { return &buf->b_visual.vi_end; }
int nvim_mark_buf_get_visual_mode(buf_T *buf) { return buf->b_visual.vi_mode; }
void nvim_mark_buf_set_visual_mode(buf_T *buf, int mode) { buf->b_visual.vi_mode = mode; }
fmark_T *nvim_mark_buf_get_prompt_start(buf_T *buf) { return &buf->b_prompt_start; }
fmark_T *nvim_mark_buf_get_changelist(buf_T *buf, int idx) { return &buf->b_changelist[idx]; }
int nvim_mark_buf_get_changelistlen(buf_T *buf) { return buf->b_changelistlen; }
void nvim_mark_buf_set_changelistlen(buf_T *buf, int len) { buf->b_changelistlen = len; }

// Global state (Phase 3)
win_T *nvim_mark_get_curwin(void) { return curwin; }
buf_T *nvim_mark_get_curbuf(void) { return curbuf; }
buf_T *nvim_mark_buflist_findnr(int fnum) { return buflist_findnr(fnum); }
int nvim_mark_bt_prompt(buf_T *buf) { return bt_prompt(buf); }

// Phase 4: Global state accessors
int nvim_mark_get_global_busy(void) { return global_busy; }
int nvim_mark_get_listcmd_busy(void) { return (int)listcmd_busy; }
unsigned nvim_mark_get_jop_flags(void) { return jop_flags; }
unsigned nvim_mark_get_cmod_flags(void) { return cmdmod.cmod_flags; }

// Phase 4: Window topline accessor
linenr_T nvim_mark_win_get_topline(win_T *win) { return win->w_topline; }

// Phase 4: Window changelist index
int nvim_mark_win_get_changelistidx(win_T *win) { return win->w_changelistidx; }
void nvim_mark_win_set_changelistidx(win_T *win, int idx) { win->w_changelistidx = idx; }

// Phase 4: Jumplist memmove helper (moves entries [from_idx+1..len) down to [from_idx..])
void nvim_mark_win_jumplist_remove(win_T *win, int from_idx, int len)
{
  memmove(&win->w_jumplist[from_idx], &win->w_jumplist[from_idx + 1],
          (size_t)(len - from_idx) * sizeof(win->w_jumplist[0]));
}

// Phase 4: Jumplist shift down (remove oldest entry, shift all down by one)
void nvim_mark_win_jumplist_shift_down(win_T *win)
{
  memmove(&win->w_jumplist[0], &win->w_jumplist[1],
          (JUMPLISTSIZE - 1) * sizeof(win->w_jumplist[0]));
}

// Phase 4: Jumplist set entry by copying from another entry
void nvim_mark_win_jumplist_copy_entry(win_T *win, int to_idx, int from_idx)
{
  win->w_jumplist[to_idx] = win->w_jumplist[from_idx];
}

// Phase 4: Tag stack accessors
int nvim_mark_win_get_tagstacklen(win_T *win) { return win->w_tagstacklen; }
void nvim_mark_win_set_tagstacklen(win_T *win, int len) { win->w_tagstacklen = len; }
int nvim_mark_win_get_tagstackidx(win_T *win) { return win->w_tagstackidx; }
void nvim_mark_win_set_tagstackidx(win_T *win, int idx) { win->w_tagstackidx = idx; }
int nvim_mark_win_get_tagstack_fnum(win_T *win, int idx) { return win->w_tagstack[idx].fmark.fnum; }
void nvim_mark_win_tagstack_clear_entry(win_T *win, int idx) { rs_tagstack_clear_entry(&win->w_tagstack[idx]); }
void nvim_mark_win_tagstack_remove(win_T *win, int from_idx, int len)
{
  memmove(&win->w_tagstack[from_idx], &win->w_tagstack[from_idx + 1],
          (size_t)(len - from_idx) * sizeof(win->w_tagstack[0]));
}

// Phase 4: SET_XFMARK helper for jumplist
void nvim_mark_win_set_jumplist_xfmark(win_T *win, int idx, pos_T mark, int fnum, fmarkv_T view)
{
  SET_XFMARK(&win->w_jumplist[idx], mark, fnum, view, NULL);
}

// Phase 4: Get jumplist entry fnum
int nvim_mark_win_get_jumplist_fnum(win_T *win, int idx) { return win->w_jumplist[idx].fmark.fnum; }
linenr_T nvim_mark_win_get_jumplist_lnum(win_T *win, int idx) { return win->w_jumplist[idx].fmark.mark.lnum; }

// Phase 4: Free jumplist entry fname
void nvim_mark_win_jumplist_free_fname(win_T *win, int idx) { xfree(win->w_jumplist[idx].fname); }

// Phase 5: Mark adjustment accessors
int nvim_mark_buf_get_has_qf_entry(buf_T *buf) { return buf->b_has_qf_entry; }
void nvim_mark_buf_set_has_qf_entry(buf_T *buf, int val) { buf->b_has_qf_entry = val; }
pos_T *nvim_mark_get_saved_cursor(void) { return &saved_cursor; }
win_T *nvim_mark_win_get_next(win_T *win) { return win->w_next; }
buf_T *nvim_mark_win_get_buf(win_T *win) { return win->w_buffer; }
linenr_T nvim_mark_win_get_old_cursor_lnum(win_T *win) { return win->w_old_cursor_lnum; }
linenr_T *nvim_mark_win_get_old_cursor_lnum_ptr(win_T *win) { return &win->w_old_cursor_lnum; }
linenr_T *nvim_mark_win_get_old_visual_lnum_ptr(win_T *win) { return &win->w_old_visual_lnum; }
linenr_T nvim_mark_win_get_topline_val(win_T *win) { return win->w_topline; }
void nvim_mark_win_set_topline_val(win_T *win, linenr_T val) { win->w_topline = val; }
void nvim_mark_win_set_topfill(win_T *win, int val) { win->w_topfill = val; }
pos_T *nvim_mark_win_get_cursor_ptr(win_T *win) { return &win->w_cursor; }
pos_T *nvim_mark_win_get_pcmark_ptr(win_T *win) { return &win->w_pcmark; }
pos_T *nvim_mark_win_get_prev_pcmark_ptr(win_T *win) { return &win->w_prev_pcmark; }

// Phase 5: Tabpage iteration
tabpage_T *nvim_mark_get_first_tabpage(void) { return first_tabpage; }
tabpage_T *nvim_mark_tabpage_next(tabpage_T *tp) { return tp->tp_next; }
win_T *nvim_mark_tabpage_firstwin(tabpage_T *tp) { return (tp == curtab) ? firstwin : tp->tp_firstwin; }

// Phase 5: External function callbacks (keep iteration in C)
int nvim_mark_qf_mark_adjust(buf_T *buf, win_T *win, linenr_T line1, linenr_T line2,
                              linenr_T amount, linenr_T amount_after)
{
  return qf_mark_adjust(buf, win, line1, line2, amount, amount_after);
}
void nvim_mark_extmark_adjust(buf_T *buf, linenr_T line1, linenr_T line2,
                               linenr_T amount, linenr_T amount_after, int op)
{
  extmark_adjust(buf, line1, line2, amount, amount_after, (ExtmarkOp)op);
}

// Phase 5: Wininfo iteration
int nvim_mark_buf_get_wininfo_count(buf_T *buf) { return (int)kv_size(buf->b_wininfo); }
pos_T *nvim_mark_buf_get_wininfo_mark(buf_T *buf, int idx) { return &kv_A(buf->b_wininfo, idx)->wi_mark.mark; }

// Phase 5: Jumplist/tagstack mark pointers for col_adjust
pos_T *nvim_mark_win_get_jumplist_mark_ptr(win_T *win, int idx) { return &win->w_jumplist[idx].fmark.mark; }
pos_T *nvim_mark_win_get_tagstack_mark_ptr(win_T *win, int idx) { return &win->w_tagstack[idx].fmark.mark; }

// Phase 5: curtab accessor
tabpage_T *nvim_mark_get_curtab(void) { return curtab; }

// Phase 6: Error message wrappers
void nvim_mark_emsg_invarg(void) { emsg(_(e_invarg)); }
void nvim_mark_emsg_argreq(void) { emsg(_(e_argreq)); }
void nvim_mark_semsg_invarg2(const char *p) { semsg(_(e_invarg2), p); }

// Phase 6: Multibyte function wrappers
const char *nvim_mark_ml_get_buf(buf_T *buf, linenr_T lnum) { return ml_get_buf(buf, lnum); }
colnr_T nvim_mark_ml_get_buf_len(buf_T *buf, linenr_T lnum) { return ml_get_buf_len(buf, lnum); }
int nvim_mark_utf_head_off(const char *base, const char *p) { return utf_head_off(base, p); }
int nvim_mark_utf_ptr2char(const char *p) { return utf_ptr2char(p); }
int nvim_mark_vim_isprintc(int c) { return vim_isprintc(c); }
int nvim_mark_ptr2cells(const char *p) { return ptr2cells(p); }

// Phase 6: Motion function wrappers
int nvim_mark_findpar(int *inclusive, int dir, int count, int what, int do_sentences) {
  bool pincl = false;
  int result = (int)findpar(&pincl, dir, count, what, (bool)do_sentences);
  if (inclusive) { *inclusive = (int)pincl; }
  return result;
}
int nvim_mark_findsent(int dir, int count) { return (int)findsent(dir, count); }
void nvim_mark_set_listcmd_busy(int val) { listcmd_busy = (bool)val; }
void nvim_mark_win_set_cursor(win_T *win, pos_T pos) { win->w_cursor = pos; }


#include "mark.c.generated.h"

// Set named mark "c" at current cursor position.
// Returns OK on success, FAIL if bad name given.
int setmark(int c)
{
  fmarkv_T view = mark_view_make(curwin->w_topline, curwin->w_cursor);
  return setmark_pos(c, &curwin->w_cursor, curbuf->b_fnum, &view);
}


// Set named mark "c" to position "pos".
// Set the previous context mark to the current position and add it to the
// jump list.
void setpcmark(void)
{
  rs_setpcmark(curwin, curbuf);
}

// To change context, call setpcmark(), then move the current position to
// where ever, then call checkpcmark().  This ensures that the previous
// context will only be changed if the cursor moved to a different line.
// If pcmark was deleted (with "dG") the previous mark is restored.
void checkpcmark(void)
{
  rs_checkpcmark(curwin);
}

/// Get mark in "count" position in the |jumplist| relative to the current index.
fmark_T *get_jumplist(win_T *win, int count)
{
  return rs_get_jumplist(win, curbuf, count);
}


/// Get a named mark.
///
/// All types of marks, even those that are not technically a mark will be returned as such. Use
/// mark_move_to() to move to the mark.
/// @note Some of the pointers are statically allocated, if in doubt make a copy. For more
/// information read mark_get_local().
/// @param buf  Buffer to get the mark from.
/// @param win  Window to get or calculate the mark from (motion type marks, context mark).
/// @param fmp[out] Optional pointer to store the result in, as a workaround for the note above.
/// @param flag MarkGet value
/// @param name Name of the mark.
///
/// @return          Mark if found, otherwise NULL.  For @c kMarkBufLocal, NULL is returned
///                  when no mark is found in @a buf.
fmark_T *mark_get(buf_T *buf, win_T *win, fmark_T *fmp, MarkGet flag, int name)
{
  return rs_mark_get(buf, win, fmp, (int)flag, name);
}

/// Get a global mark {A-Z0-9}.
///
/// @param name  the name of the mark.
/// @param resolve  Whether to try resolving the mark fnum (i.e., load the buffer stored in
///                 the mark fname and update the xfmark_T (expensive)).
///
/// @return  Mark
xfmark_T *mark_get_global(bool resolve, int name)
{
  return rs_mark_get_global(resolve ? 1 : 0, name);
}

/// Get a local mark (lowercase and symbols).
///
/// Some marks are not actually marks, but positions that are never adjusted or motions presented as
/// marks. Search first for marks and fallback to finding motion type marks. If it's known
/// ahead of time that the mark is actually a motion use the mark_get_motion() directly.
///
/// @note  Lowercase, last_cursor '"', last insert '^', last change '.' are not statically
/// allocated, everything else is.
/// @param name  the name of the mark.
/// @param win  window to retrieve marks that belong to it (motions and context mark).
/// @param buf  buf to retrieve marks that belong to it.
///
/// @return  Mark, NULL if not found.
fmark_T *mark_get_local(buf_T *buf, win_T *win, int name)
{
  return rs_mark_get_local(buf, win, name, curbuf);
}


/// Attempt to switch to the buffer of the given global mark
///
/// @param fm
/// @param pcmark_on_switch  leave a context mark when switching buffer.
/// @return whether the buffer was switched or not.
static MarkMoveRes switch_to_mark_buf(fmark_T *fm, bool pcmark_on_switch)
{
  if (fm->fnum != curbuf->b_fnum) {
    // Switch to another file.
    int getfile_flag = pcmark_on_switch ? GETF_SETMARK : 0;
    bool res = buflist_getfile(fm->fnum, fm->mark.lnum, getfile_flag, false) == OK;
    return res == true ? kMarkSwitchedBuf : kMarkMoveFailed;
  }
  return 0;
}

/// Move to the given file mark, changing the buffer and cursor position.
///
/// Validate the mark, switch to the buffer, and move the cursor.
/// @param fm  Mark, can be NULL will raise E78: Unknown mark
/// @param flags  MarkMove flags to configure the movement to the mark.
///
/// @return  MarkMovekRes flags representing the outcome
MarkMoveRes mark_move_to(fmark_T *fm, MarkMove flags)
{
  static fmark_T fm_copy = INIT_FMARK;
  MarkMoveRes res = kMarkMoveSuccess;
  const char *errormsg = NULL;
  if (!mark_check(fm, &errormsg)) {
    if (errormsg != NULL) {
      emsg(errormsg);
    }
    res = kMarkMoveFailed;
    goto end;
  }

  if (fm->fnum != curbuf->handle) {
    // Need to change buffer
    fm_copy = *fm;  // Copy, autocommand may change it
    fm = &fm_copy;
    // Jump to the file with the mark
    res |= switch_to_mark_buf(fm, !(flags & kMarkJumpList));
    // Failed switching buffer
    if (res & kMarkMoveFailed) {
      goto end;
    }
    // Check line count now that the **destination buffer is loaded**.
    if (!mark_check_line_bounds(curbuf, fm, &errormsg)) {
      if (errormsg != NULL) {
        emsg(errormsg);
      }
      res |= kMarkMoveFailed;
      goto end;
    }
  } else if (flags & kMarkContext) {
    // Doing it in this condition avoids double context mark when switching buffer.
    setpcmark();
  }
  // Move the cursor while keeping track of what changed for the caller
  pos_T prev_pos = curwin->w_cursor;
  pos_T pos = fm->mark;
  // Set lnum again, autocommands my have changed it
  curwin->w_cursor = fm->mark;
  if (flags & kMarkBeginLine) {
    beginline(BL_WHITE | BL_FIX);
  }
  // Use Rust helper to calculate result flags based on position changes
  res = rs_mark_move_calc_result(prev_pos.lnum, prev_pos.col, pos.lnum, pos.col, res);
  if (flags & kMarkSetView) {
    mark_view_restore(fm);
  }

  // Use Rust helper to check if cursor check is needed
  if (rs_mark_move_needs_cursor_check(res)) {
    check_cursor(curwin);
  }
end:
  return res;
}

/// Restore the mark view.
/// By remembering the offset between topline and mark lnum at the time of
/// definition, this function restores the "view".
/// Restore the mark view.
/// By remembering the offset between topline and mark lnum at the time of
/// definition, this function restores the "view".
/// @note  Assumes the mark has been checked, is valid.
/// @param  fm the named mark.
void mark_view_restore(fmark_T *fm)
{
  rs_mark_view_restore(fm, curwin);
}

/// Create fmarkv_T from topline and position. Rust implementation.
fmarkv_T mark_view_make(linenr_T topline, pos_T pos)
{
  return rs_mark_view_make(topline, pos.lnum);
}

/// Search for the next named mark in the current file from a start position.
///
/// @param startpos  where to start.
/// @param dir  direction for search.
///
/// @return  next mark or NULL if no mark is found.
fmark_T *getnextmark(pos_T *startpos, int dir, int begin_line)
{
  return rs_getnextmark(startpos, dir, begin_line, curbuf);
}

// For an xtended filemark: set the fnum from the fname.
// This is used for marks obtained from the .shada file.  It's postponed
// until the mark is used to avoid a long startup delay.
static void fname2fnum(xfmark_T *fm)
{
  if (fm->fname == NULL) {
    return;
  }

  // First expand "~/" in the file name to the home directory.
  // Don't expand the whole name, it may contain other '~' chars.
  if (fm->fname[0] == '~' && vim_ispathsep_nocolon(fm->fname[1])) {
    size_t len = expand_env("~/", NameBuff, MAXPATHL);
    xstrlcpy(NameBuff + len, fm->fname + 2, MAXPATHL - len);
  } else {
    xstrlcpy(NameBuff, fm->fname, MAXPATHL);
  }

  // Try to shorten the file name.
  os_dirname(IObuff, IOSIZE);
  char *p = path_shorten_fname(NameBuff, IObuff);

  // buflist_new() will call fmarks_check_names()
  (void)buflist_new(NameBuff, p, 1, 0);
}

// Check all file marks for a name that matches the file name in buf.
// May replace the name with an fnum.
// Used for marks that come from the .shada file.
extern void fmarks_check_names(buf_T *buf);

/// Check the position in @a fm is valid.
///
/// Checks for:
/// - NULL raising unknown mark error.
/// - Line number <= 0 raising mark not set.
/// - Line number > buffer line count, raising invalid mark.
///
/// @param fm[in]  File mark to check.
/// @param errormsg[out]  Error message, if any.
///
/// @return  true if the mark passes all the above checks, else false.
bool mark_check(fmark_T *fm, const char **errormsg)
{
  return rs_mark_check(fm, errormsg, curbuf) != 0;
}

/// Check if a mark line number is greater than the buffer line count, and set e_markinval.
///
/// @note  Should be done after the buffer is loaded into memory.
/// @param buf  Buffer where the mark is set.
/// @param fm  Mark to check.
/// @param errormsg[out]  Error message, if any.
/// @return  true if below line count else false.
bool mark_check_line_bounds(buf_T *buf, fmark_T *fm, const char **errormsg)
{
  return rs_mark_check_line_bounds(buf, fm->mark.lnum, errormsg, _(e_markinval)) != 0;
}


// Get name of file from a filemark.
// When it's in the current buffer, return the text at the mark.
// Returns an allocated string.
char *fm_getname(fmark_T *fmark, int lead_len)
{
  return rs_fm_getname(fmark, lead_len, curbuf);
}

/// Return the line at mark "mp".  Truncate to fit in window.
/// The returned string has been allocated.
static char *mark_line(pos_T *mp, int lead_len)
  FUNC_ATTR_NONNULL_RET
{
  char *p;

  if (mp->lnum == 0 || mp->lnum > curbuf->b_ml.ml_line_count) {
    return xstrdup("-invalid-");
  }
  assert(Columns >= 0);
  // Allow for up to 5 bytes per character.
  char *s = xstrnsave(skipwhite(ml_get(mp->lnum)), (size_t)Columns * 5);

  // Truncate the line to fit it in the window
  int len = 0;
  for (p = s; *p != NUL; MB_PTR_ADV(p)) {
    len += ptr2cells(p);
    if (len >= Columns - lead_len) {
      break;
    }
  }
  *p = NUL;
  return s;
}

// print the marks
void ex_marks(exarg_T *eap)
{
  char *arg = eap->arg;
  char *name;
  pos_T *posp;

  if (arg != NULL && *arg == NUL) {
    arg = NULL;
  }

  msg_ext_set_kind("list_cmd");
  show_one_mark('\'', arg, &curwin->w_pcmark, NULL, true);
  for (int i = 0; i < NMARKS; i++) {
    show_one_mark(i + 'a', arg, &curbuf->b_namedm[i].mark, NULL, true);
  }
  for (int i = 0; i < NGLOBALMARKS; i++) {
    if (namedfm[i].fmark.fnum != 0) {
      name = fm_getname(&namedfm[i].fmark, 15);
    } else {
      name = namedfm[i].fname;
    }
    if (name != NULL) {
      show_one_mark(i >= NMARKS ? i - NMARKS + '0' : i + 'A',
                    arg, &namedfm[i].fmark.mark, name,
                    namedfm[i].fmark.fnum == curbuf->b_fnum);
      if (namedfm[i].fmark.fnum != 0) {
        xfree(name);
      }
    }
  }
  show_one_mark('"', arg, &curbuf->b_last_cursor.mark, NULL, true);
  show_one_mark('[', arg, &curbuf->b_op_start, NULL, true);
  show_one_mark(']', arg, &curbuf->b_op_end, NULL, true);
  show_one_mark('^', arg, &curbuf->b_last_insert.mark, NULL, true);
  show_one_mark('.', arg, &curbuf->b_last_change.mark, NULL, true);
  if (bt_prompt(curbuf)) {
    show_one_mark(':', arg, &curbuf->b_prompt_start.mark, NULL, true);
  }

  // Show the marks as where they will jump to.
  pos_T *startp = &curbuf->b_visual.vi_start;
  pos_T *endp = &curbuf->b_visual.vi_end;
  if ((lt(*startp, *endp) || endp->lnum == 0) && startp->lnum != 0) {
    posp = startp;
  } else {
    posp = endp;
  }
  show_one_mark('<', arg, posp, NULL, true);
  show_one_mark('>', arg, posp == startp ? endp : startp, NULL, true);

  show_one_mark(-1, arg, NULL, NULL, false);
}

/// @param current  in current file
static void show_one_mark(int c, char *arg, pos_T *p, char *name_arg, int current)
{
  static bool did_title = false;
  bool mustfree = false;
  char *name = name_arg;

  if (c == -1) {  // finish up
    if (did_title) {
      did_title = false;
    } else {
      if (arg == NULL) {
        msg(_("No marks set"), 0);
      } else {
        semsg(_("E283: No marks matching \"%s\""), arg);
      }
    }
  } else if (!got_int
             && (arg == NULL || vim_strchr(arg, c) != NULL)
             && p->lnum != 0) {
    // don't output anything if 'q' typed at --more-- prompt
    if (name == NULL && current) {
      name = mark_line(p, 15);
      mustfree = true;
    }
    if (!message_filtered(name)) {
      if (!did_title) {
        // Highlight title
        msg_puts_title(_("\nmark line  col file/text"));
        did_title = true;
      }
      msg_putchar('\n');
      if (!got_int) {
        snprintf(IObuff, IOSIZE, " %c %6" PRIdLINENR " %4d ", c, p->lnum, p->col);
        msg_outtrans(IObuff, 0, false);
        if (name != NULL) {
          msg_outtrans(name, current ? HLF_D : 0, false);
        }
      }
    }
    if (mustfree) {
      xfree(name);
    }
  }
}

// ":delmarks[!] [marks]"
void ex_delmarks(exarg_T *eap)
{
  rs_ex_delmarks(eap->arg, eap->forceit, curbuf);
}

// print the jumplist
void ex_jumps(exarg_T *eap)
{
  cleanup_jumplist(curwin, true);
  // Highlight title
  msg_ext_set_kind("list_cmd");
  msg_puts_title(_("\n jump line  col file/text"));
  for (int i = 0; i < curwin->w_jumplistlen && !got_int; i++) {
    if (curwin->w_jumplist[i].fmark.mark.lnum != 0) {
      char *name = fm_getname(&curwin->w_jumplist[i].fmark, 16);

      // Make sure to output the current indicator, even when on an wiped
      // out buffer.  ":filter" may still skip it.
      if (name == NULL && i == curwin->w_jumplistidx) {
        name = xstrdup("-invalid-");
      }
      // apply :filter /pat/ or file name not available
      if (name == NULL || message_filtered(name)) {
        xfree(name);
        continue;
      }

      msg_putchar('\n');
      if (got_int) {
        xfree(name);
        break;
      }
      snprintf(IObuff, IOSIZE, "%c %2d %5" PRIdLINENR " %4d ",
               i == curwin->w_jumplistidx ? '>' : ' ',
               i > curwin->w_jumplistidx ? i - curwin->w_jumplistidx : curwin->w_jumplistidx - i,
               curwin->w_jumplist[i].fmark.mark.lnum, curwin->w_jumplist[i].fmark.mark.col);
      msg_outtrans(IObuff, 0, false);
      msg_outtrans(name, curwin->w_jumplist[i].fmark.fnum == curbuf->b_fnum ? HLF_D : 0, false);
      xfree(name);
      os_breakcheck();
    }
  }
  if (curwin->w_jumplistidx == curwin->w_jumplistlen) {
    msg_puts("\n>");
  }
}

void ex_clearjumps(exarg_T *eap)
{
  rs_ex_clearjumps(curwin);
}

// print the changelist
void ex_changes(exarg_T *eap)
{
  msg_ext_set_kind("list_cmd");
  // Highlight title
  msg_puts_title(_("\nchange line  col text"));

  for (int i = 0; i < curbuf->b_changelistlen && !got_int; i++) {
    if (curbuf->b_changelist[i].mark.lnum != 0) {
      msg_putchar('\n');
      if (got_int) {
        break;
      }
      snprintf(IObuff, IOSIZE, "%c %3d %5" PRIdLINENR " %4d ",
               i == curwin->w_changelistidx ? '>' : ' ',
               i >
               curwin->w_changelistidx ? i - curwin->w_changelistidx : curwin->w_changelistidx - i,
               curbuf->b_changelist[i].mark.lnum,
               curbuf->b_changelist[i].mark.col);
      msg_outtrans(IObuff, 0, false);
      char *name = mark_line(&curbuf->b_changelist[i].mark, 17);
      msg_outtrans(name, HLF_D, false);
      xfree(name);
      os_breakcheck();
    }
  }
  if (curwin->w_changelistidx == curbuf->b_changelistlen) {
    msg_puts("\n>");
  }
}


// When deleting lines, this may create duplicate marks in the
// jumplist. They will be removed here for the specified window.
// When "loadfiles" is true first ensure entries have the "fnum" field set
// (this may be a bit slow).
void cleanup_jumplist(win_T *wp, bool loadfiles)
{
  rs_cleanup_jumplist(wp, (int)loadfiles);
}


/// Iterate over jumplist items
///
/// @warning No jumplist-editing functions must be called while iteration is in
///          progress.
///
/// @param[in]   iter  Iterator. Pass NULL to start iteration.
/// @param[in]   win   Window for which jump list is processed.
/// @param[out]  fm    Item definition.
///
/// @return Pointer that needs to be passed to next `mark_jumplist_iter` call or
///         NULL if iteration is over.
const void *mark_jumplist_iter(const void *const iter, const win_T *const win, xfmark_T *const fm)
  FUNC_ATTR_NONNULL_ARG(2, 3) FUNC_ATTR_WARN_UNUSED_RESULT
{
  if (iter == NULL && win->w_jumplistlen == 0) {
    *fm = (xfmark_T)INIT_XFMARK;
    return NULL;
  }
  const xfmark_T *const iter_mark = iter == NULL ? &(win->w_jumplist[0])
                                                 : (const xfmark_T *const)iter;
  *fm = *iter_mark;
  if (iter_mark == &(win->w_jumplist[win->w_jumplistlen - 1])) {
    return NULL;
  }
  return iter_mark + 1;
}

/// Iterate over global marks
///
/// @warning No mark-editing functions must be called while iteration is in
///          progress.
///
/// @param[in]   iter  Iterator. Pass NULL to start iteration.
/// @param[out]  name  Mark name.
/// @param[out]  fm    Mark definition.
///
/// @return Pointer that needs to be passed to next `mark_global_iter` call or
///         NULL if iteration is over.
const void *mark_global_iter(const void *const iter, char *const name, xfmark_T *const fm)
  FUNC_ATTR_NONNULL_ARG(2, 3) FUNC_ATTR_WARN_UNUSED_RESULT
{
  *name = NUL;
  const xfmark_T *iter_mark = (iter == NULL
                               ? &(namedfm[0])
                               : (const xfmark_T *const)iter);
  while ((size_t)(iter_mark - &(namedfm[0])) < ARRAY_SIZE(namedfm)
         && !iter_mark->fmark.mark.lnum) {
    iter_mark++;
  }
  if ((size_t)(iter_mark - &(namedfm[0])) == ARRAY_SIZE(namedfm)
      || !iter_mark->fmark.mark.lnum) {
    return NULL;
  }
  size_t iter_off = (size_t)(iter_mark - &(namedfm[0]));
  *name = (char)(iter_off < NMARKS
                 ? 'A' + (char)iter_off
                 : '0' + (char)(iter_off - NMARKS));
  *fm = *iter_mark;
  while ((size_t)(++iter_mark - &(namedfm[0])) < ARRAY_SIZE(namedfm)) {
    if (iter_mark->fmark.mark.lnum) {
      return (const void *)iter_mark;
    }
  }
  return NULL;
}

/// Get next mark and its name
///
/// @param[in]      buf        Buffer for which next mark is taken.
/// @param[in,out]  mark_name  Pointer to the current mark name. Next mark name
///                            will be saved at this address as well.
///
///                            Current mark name must either be NUL, '"', '^',
///                            '.' or 'a' .. 'z'. If it is neither of these
///                            behaviour is undefined.
///
/// @return Pointer to the next mark or NULL.
static inline const fmark_T *next_buffer_mark(const buf_T *const buf, char *const mark_name)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_WARN_UNUSED_RESULT
{
  switch (*mark_name) {
  case NUL:
    *mark_name = '"';
    return &(buf->b_last_cursor);
  case '"':
    *mark_name = '^';
    return &(buf->b_last_insert);
  case '^':
    *mark_name = '.';
    return &(buf->b_last_change);
  case '.':
    *mark_name = 'a';
    return &(buf->b_namedm[0]);
  case 'z':
    return NULL;
  default:
    (*mark_name)++;
    return &(buf->b_namedm[*mark_name - 'a']);
  }
}

/// Iterate over buffer marks
///
/// @warning No mark-editing functions must be called while iteration is in
///          progress.
///
/// @param[in]   iter  Iterator. Pass NULL to start iteration.
/// @param[in]   buf   Buffer.
/// @param[out]  name  Mark name.
/// @param[out]  fm    Mark definition.
///
/// @return Pointer that needs to be passed to next `mark_buffer_iter` call or
///         NULL if iteration is over.
const void *mark_buffer_iter(const void *const iter, const buf_T *const buf, char *const name,
                             fmark_T *const fm)
  FUNC_ATTR_NONNULL_ARG(2, 3, 4) FUNC_ATTR_WARN_UNUSED_RESULT
{
  *name = NUL;
  char mark_name = (char)(iter == NULL
                          ? NUL
                          : (iter == &(buf->b_last_cursor)
                             ? '"'
                             : (iter == &(buf->b_last_insert)
                                ? '^'
                                : (iter == &(buf->b_last_change)
                                   ? '.'
                                   : 'a' + (const fmark_T *)iter - &(buf->b_namedm[0])))));
  const fmark_T *iter_mark = next_buffer_mark(buf, &mark_name);
  while (iter_mark != NULL && iter_mark->mark.lnum == 0) {
    iter_mark = next_buffer_mark(buf, &mark_name);
  }
  if (iter_mark == NULL) {
    return NULL;
  }
  size_t iter_off = (size_t)(iter_mark - &(buf->b_namedm[0]));
  if (mark_name) {
    *name = mark_name;
  } else {
    *name = (char)('a' + (char)iter_off);
  }
  *fm = *iter_mark;
  return (const void *)iter_mark;
}

/// Set global mark
///
/// @param[in]  name    Mark name.
/// @param[in]  fm      Mark to be set.
/// @param[in]  update  If true then only set global mark if it was created
///                     later then existing one.
///
/// @return true on success, false on failure.
bool mark_set_global(const char name, const xfmark_T fm, const bool update)
{
  return rs_mark_set_global((int)name, fm, update ? 1 : 0) != 0;
}

/// Set local mark
///
/// @param[in]  name    Mark name.
/// @param[in]  buf     Pointer to the buffer to set mark in.
/// @param[in]  fm      Mark to be set.
/// @param[in]  update  If true then only set global mark if it was created
///                     later then existing one.
///
/// @return true on success, false on failure.
bool mark_set_local(const char name, buf_T *const buf, const fmark_T fm, const bool update)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_mark_set_local((int)name, buf, fm, update ? 1 : 0) != 0;
}

#if defined(EXITFREE)
void free_all_marks(void)
{
  rs_free_all_marks();
}
#endif

// Add information about mark 'mname' to list 'l'
static int add_mark(list_T *l, const char *mname, const pos_T *pos, int bufnr, const char *fname)
  FUNC_ATTR_NONNULL_ARG(1, 2, 3)
{
  if (pos->lnum <= 0) {
    return OK;
  }

  dict_T *d = tv_dict_alloc();
  tv_list_append_dict(l, d);

  list_T *lpos = tv_list_alloc(kListLenMayKnow);

  tv_list_append_number(lpos, bufnr);
  tv_list_append_number(lpos, pos->lnum);
  tv_list_append_number(lpos, pos->col < MAXCOL ? pos->col + 1 : MAXCOL);
  tv_list_append_number(lpos, pos->coladd);

  if (tv_dict_add_str(d, S_LEN("mark"), mname) == FAIL
      || tv_dict_add_list(d, S_LEN("pos"), lpos) == FAIL
      || (fname != NULL && tv_dict_add_str(d, S_LEN("file"), fname) == FAIL)) {
    return FAIL;
  }

  return OK;
}

/// Get information about marks local to a buffer.
///
/// @param[in] buf  Buffer to get the marks from
/// @param[out] l   List to store marks
void get_buf_local_marks(const buf_T *buf, list_T *l)
  FUNC_ATTR_NONNULL_ALL
{
  char mname[3] = "' ";

  // Marks 'a' to 'z'
  for (int i = 0; i < NMARKS; i++) {
    mname[1] = (char)('a' + i);
    add_mark(l, mname, &buf->b_namedm[i].mark, buf->b_fnum, NULL);
  }

  // Mark '' is a window local mark and not a buffer local mark
  add_mark(l, "''", &curwin->w_pcmark, curbuf->b_fnum, NULL);

  add_mark(l, "'\"", &buf->b_last_cursor.mark, buf->b_fnum, NULL);
  add_mark(l, "'[", &buf->b_op_start, buf->b_fnum, NULL);
  add_mark(l, "']", &buf->b_op_end, buf->b_fnum, NULL);
  add_mark(l, "'^", &buf->b_last_insert.mark, buf->b_fnum, NULL);
  add_mark(l, "'.", &buf->b_last_change.mark, buf->b_fnum, NULL);
  add_mark(l, "'<", &buf->b_visual.vi_start, buf->b_fnum, NULL);
  add_mark(l, "'>", &buf->b_visual.vi_end, buf->b_fnum, NULL);
}

/// Get a global mark
///
/// @note  Mark might not have it's fnum resolved.
/// @param[in]  Name of named mark
/// @param[out] Global/file mark
xfmark_T get_raw_global_mark(char name)
{
  return rs_get_raw_global_mark((int)name);
}

/// Get information about global marks ('A' to 'Z' and '0' to '9')
///
/// @param[out] l  List to store global marks
void get_global_marks(list_T *l)
  FUNC_ATTR_NONNULL_ALL
{
  char mname[3] = "' ";
  char *name;

  // Marks 'A' to 'Z' and '0' to '9'
  for (int i = 0; i < NMARKS + EXTRA_MARKS; i++) {
    if (namedfm[i].fmark.fnum != 0) {
      name = buflist_nr2name(namedfm[i].fmark.fnum, true, true);
    } else {
      name = namedfm[i].fname;
    }
    if (name != NULL) {
      mname[1] = i >= NMARKS ? (char)(i - NMARKS + '0') : (char)(i + 'A');

      add_mark(l, mname, &namedfm[i].fmark.mark, namedfm[i].fmark.fnum, name);
      if (namedfm[i].fmark.fnum != 0) {
        xfree(name);
      }
    }
  }
}

// =============================================================================
// Cross-function callbacks from Rust (Phase 3)
// Placed at end of file after static function definitions
// =============================================================================
void nvim_mark_setpcmark(void) { setpcmark(); }
void nvim_mark_fname2fnum(xfmark_T *xfm) { fname2fnum(xfm); }
char *nvim_mark_buflist_nr2name(int fnum, int listed, int unstripped) {
  return buflist_nr2name(fnum, listed, unstripped);
}
char *nvim_mark_mark_line(pos_T *pos, int lead_len) { return mark_line(pos, lead_len); }
