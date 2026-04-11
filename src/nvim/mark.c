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


// mark_line() is now implemented in Rust (mark/src/lib.rs, rs_mark_line).
// The Rust implementation exports as "mark_line" via #[export_name].
extern char *mark_line(pos_T *mp, int lead_len);

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
// Placed at end of file after static function definitions
void nvim_mark_fname2fnum(xfmark_T *xfm) { fname2fnum(xfm); }
char *nvim_mark_buflist_nr2name(int fnum, int listed, int unstripped) {
  return buflist_nr2name(fnum, listed, unstripped);
}
