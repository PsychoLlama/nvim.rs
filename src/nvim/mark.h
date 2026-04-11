#pragma once

#include <locale.h>

#include "nvim/ascii_defs.h"
#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/extmark_defs.h"  // IWYU pragma: keep
#include "nvim/macros_defs.h"
#include "nvim/mark_defs.h"  // IWYU pragma: keep
#include "nvim/os/time.h"

#include "mark.h.generated.h"
#include "mark.h.inline.generated.h"

int setmark_pos(int c, pos_T *pos, int fnum, fmarkv_T *view_pt);
extern int rs_mark_global_index(int name);
extern int rs_mark_local_index(int name);
extern bool rs_mark_is_valid_named(int name);
extern bool rs_mark_is_file_mark(int name);
extern bool rs_mark_is_jump_mark(int name);
extern bool rs_mark_is_special(int name);
extern bool rs_mark_is_visual(int name);
extern bool rs_mark_is_last_cursor(int name);
extern bool rs_mark_is_last_insert(int name);
extern bool rs_mark_is_last_change(int name);
extern bool rs_mark_is_sentence(int name);
extern int rs_pos_is_valid(pos_T pos);
extern int rs_pos_in_range(pos_T pos, int line_count);

// Functions implemented in Rust (exported under their C names via #[export_name])
extern void fmarks_check_names(buf_T *buf);

// Phase 7/8: mark_move_to and display commands now implemented in Rust
extern MarkMoveRes mark_move_to(fmark_T *fm, MarkMove flags);
extern void ex_marks(exarg_T *eap);
extern void ex_jumps(exarg_T *eap);
extern void ex_changes(exarg_T *eap);

// Phase 1: Pass-through wrappers now implemented in Rust
extern int setmark(int c);
extern void setpcmark(void);
extern void checkpcmark(void);
extern fmark_T *get_jumplist(win_T *win, int count);
extern fmark_T *mark_get(buf_T *buf, win_T *win, fmark_T *fmp, MarkGet flag, int name);
extern xfmark_T *mark_get_global(bool resolve, int name);
extern fmark_T *mark_get_local(buf_T *buf, win_T *win, int name);
extern void mark_view_restore(fmark_T *fm);
extern fmarkv_T mark_view_make(linenr_T topline, pos_T pos);
extern fmark_T *getnextmark(pos_T *startpos, int dir, int begin_line);
extern bool mark_check(fmark_T *fm, const char **errormsg);
extern bool mark_check_line_bounds(buf_T *buf, fmark_T *fm, const char **errormsg);
extern char *fm_getname(fmark_T *fmark, int lead_len);
extern bool mark_set_global(const char name, const xfmark_T fm, const bool update);
extern bool mark_set_local(const char name, buf_T *const buf, const fmark_T fm, const bool update);
extern void free_all_marks(void);
extern xfmark_T get_raw_global_mark(char name);
extern void ex_clearjumps(exarg_T *eap);
extern void ex_delmarks(exarg_T *eap);
extern void cleanup_jumplist(win_T *wp, bool loadfiles);
extern void free_fmark(fmark_T fm);
extern void free_xfmark(xfmark_T fm);
extern void clear_fmark(fmark_T *fm, Timestamp timestamp);
extern void mark_jumplist_forget_file(win_T *wp, int fnum);
extern void mark_forget_file(win_T *wp, int fnum);
extern fmark_T *get_changelist(buf_T *buf, win_T *win, int count);
extern fmark_T *mark_get_motion(buf_T *buf, win_T *win, int name);
extern fmark_T *mark_get_visual(buf_T *buf, int name);
extern fmark_T *pos_to_mark(buf_T *buf, fmark_T *fmp, pos_T pos);
extern void clrallmarks(buf_T *buf, Timestamp timestamp);
extern void copy_jumplist(win_T *from, win_T *to);
extern void free_jumplist(win_T *wp);
extern void set_last_cursor(win_T *win);
extern void mark_mb_adjustpos(buf_T *buf, pos_T *lp);
extern void mark_adjust(linenr_T line1, linenr_T line2, linenr_T amount, linenr_T amount_after,
                        ExtmarkOp op);
extern void mark_adjust_nofold(linenr_T line1, linenr_T line2, linenr_T amount,
                               linenr_T amount_after, ExtmarkOp op);
extern void mark_adjust_buf(buf_T *buf, linenr_T line1, linenr_T line2, linenr_T amount,
                            linenr_T amount_after, bool adjust_folds, MarkAdjustMode mode,
                            ExtmarkOp op);
extern void mark_col_adjust(linenr_T lnum, colnr_T mincol, linenr_T lnum_amount,
                            colnr_T col_amount, int spaces_removed);

/// Convert mark name to the offset
static inline int mark_global_index(const char name)
  FUNC_ATTR_CONST
{
  return rs_mark_global_index((int)name);
}

/// Convert local mark name to the offset
static inline int mark_local_index(const char name)
  FUNC_ATTR_CONST
{
  return rs_mark_local_index((int)name);
}

/// Global marks (marks with file number or name)
EXTERN xfmark_T namedfm[NGLOBALMARKS] INIT( = { 0 });

#define SET_FMARK(fmarkp_, mark_, fnum_, view_) \
  do { \
    fmark_T *const fmarkp__ = fmarkp_; \
    fmarkp__->mark = mark_; \
    fmarkp__->fnum = fnum_; \
    fmarkp__->timestamp = os_time(); \
    fmarkp__->view = view_; \
    fmarkp__->additional_data = NULL; \
  } while (0)

/// Free and set fmark using given value
#define RESET_FMARK(fmarkp_, mark_, fnum_, view_) \
  do { \
    fmark_T *const fmarkp___ = fmarkp_; \
    free_fmark(*fmarkp___); \
    SET_FMARK(fmarkp___, mark_, fnum_, view_); \
  } while (0)

/// Set given extended mark (regular mark + file name)
#define SET_XFMARK(xfmarkp_, mark_, fnum_, view_, fname_) \
  do { \
    xfmark_T *const xfmarkp__ = xfmarkp_; \
    xfmarkp__->fname = fname_; \
    SET_FMARK(&(xfmarkp__->fmark), mark_, fnum_, view_); \
  } while (0)

/// Free and set given extended mark (regular mark + file name)
#define RESET_XFMARK(xfmarkp_, mark_, fnum_, view_, fname_) \
  do { \
    xfmark_T *const xfmarkp__ = xfmarkp_; \
    free_xfmark(*xfmarkp__); \
    xfmarkp__->fname = fname_; \
    SET_FMARK(&(xfmarkp__->fmark), mark_, fnum_, view_); \
  } while (0)
