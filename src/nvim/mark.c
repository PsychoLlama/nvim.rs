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

// Static assertions for constants shared with Rust
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

// Rust FFI declarations

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

// mark_move_to is now implemented in Rust (mark/src/lib.rs, exported_mark_move_to).
// The Rust implementation exports as "mark_move_to" via #[export_name].
extern MarkMoveRes mark_move_to(fmark_T *fm, MarkMove flags);

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




#include "mark.c.generated.h"



// switch_to_mark_buf and mark_move_to are now implemented in Rust.
// mark_move_to is exported via #[export_name = "mark_move_to"] in mark/src/lib.rs.



// fname2fnum is now implemented in Rust (mark/src/lib.rs, rs_fname2fnum).
// It resolves a .shada file mark's filename to a buffer number.

// Check all file marks for a name that matches the file name in buf.
// May replace the name with an fnum.
// Used for marks that come from the .shada file.
extern void fmarks_check_names(buf_T *buf);


// mark_line() is now implemented in Rust (mark/src/lib.rs, rs_mark_line).
// The Rust implementation exports as "mark_line" via #[export_name].
extern char *mark_line(pos_T *mp, int lead_len);

// ex_marks, show_one_mark, ex_jumps, ex_changes are now implemented in Rust.
// They are exported via #[export_name] in mark/src/lib.rs (Phase 8).
extern void ex_marks(exarg_T *eap);
extern void ex_jumps(exarg_T *eap);
extern void ex_changes(exarg_T *eap);




// mark_jumplist_iter, mark_global_iter, next_buffer_mark, mark_buffer_iter
// are now implemented in Rust (mark/src/lib.rs, Phase 9).
// They are exported via #[export_name] and declared in mark.h.
extern const void *mark_jumplist_iter(const void *iter, const win_T *win, xfmark_T *fm);
extern const void *mark_global_iter(const void *iter, char *name, xfmark_T *fm);
extern const void *mark_buffer_iter(const void *iter, const buf_T *buf, char *name, fmark_T *fm);


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

// Cross-function callbacks from Rust (Phase 3)
// nvim_mark_fname2fnum was removed: fname2fnum is now implemented in Rust (rs_fname2fnum).
char *nvim_mark_buflist_nr2name(int fnum, int listed, int unstripped) {
  return buflist_nr2name(fnum, listed, unstripped);
}
