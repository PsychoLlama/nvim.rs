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
