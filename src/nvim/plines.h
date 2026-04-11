#pragma once

#include <stdbool.h>
#include <stdint.h>

#include "nvim/marktree_defs.h"
#include "nvim/pos_defs.h"
#include "nvim/types_defs.h"

typedef bool CSType;

enum {
  kCharsizeRegular,
  kCharsizeFast,
};

/// Argument for char size functions.
typedef struct {
  win_T *win;
  char *line;                ///< Start of the line.

  bool use_tabstop;          ///< Use 'tabstop' instead of char2cells() for a TAB.
  int indent_width;          ///< Width of 'showbreak' and 'breakindent' on wrapped
                             ///< parts of lines, INT_MIN if not yet calculated.

  int virt_row;              ///< Row for virtual text, -1 if no virtual text.
  int cur_text_width_left;   ///< Width of virtual text left of cursor.
  int cur_text_width_right;  ///< Width of virtual text right of cursor.

  int max_head_vcol;         ///< See charsize_regular().
  MarkTreeIter iter[1];
} CharsizeArg;

typedef struct {
  int width;
  int head;  ///< Size of 'breakindent' etc. before the character (included in width).
} CharSize;

#include "plines.h.generated.h"
#include "plines.h.inline.generated.h"

int win_chartabsize(win_T *wp, char *p, colnr_T col);
int linetabsize_col(int startvcol, char *s);
int linetabsize(win_T *wp, linenr_T lnum);
int linetabsize_eol(win_T *wp, linenr_T lnum);
CharSize charsize_regular(CharsizeArg *csarg, char *const cur, colnr_T const vcol, int32_t const cur_char);
int charsize_nowrap(buf_T *buf, const char *cur, bool use_tabstop, colnr_T vcol, int32_t cur_char);
int linesize_regular(CharsizeArg *const csarg, int vcol_arg, colnr_T const len);
colnr_T getvcol_nolist(pos_T *posp);
bool win_may_fill(win_T *wp);
int64_t win_text_height(win_T *const wp, const linenr_T start_lnum, const int64_t start_vcol, linenr_T *const end_lnum, int64_t *const end_vcol, int64_t *const fill, int64_t const max);

// Rust-implemented plines functions (plines crate, export_name)
int plines_win(win_T *wp, linenr_T lnum, int limit_winheight);
int plines_win_nofill(win_T *wp, linenr_T lnum, int limit_winheight);
int plines_win_full(win_T *wp, linenr_T lnum, linenr_T *nextp, int *foldedp, int cache, int limit_winheight);
int plines_m_win(win_T *wp, linenr_T first, linenr_T last, int max);

/// Get the number of cells taken up on the screen by the given character at vcol.
/// "csarg->cur_text_width_left" and "csarg->cur_text_width_right" are set
/// to the extra size for inline virtual text.
///
/// When "csarg->max_head_vcol" is positive, only count in "head" the size
/// of 'showbreak'/'breakindent' before "csarg->max_head_vcol".
/// When "csarg->max_head_vcol" is negative, only count in "head" the size
/// of 'showbreak'/'breakindent' before where cursor should be placed.
static inline CharSize win_charsize(CSType cstype, int vcol, char *ptr, int32_t chr,
                                    CharsizeArg *csarg)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_ALWAYS_INLINE
{
  if (cstype == kCharsizeFast) {
    return charsize_fast(csarg, ptr, vcol, chr);
  } else {
    return charsize_regular(csarg, ptr, vcol, chr);
  }
}

/// Return the number of cells the string "s" will take on the screen,
/// taking into account the size of a tab.
///
/// @param s
///
/// @return Number of cells the string will take on the screen.
static inline int linetabsize_str(char *s)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_ALWAYS_INLINE
{
  return linetabsize_col(0, s);
}

/// Like linetabsize_str(), but for a given window instead of the current one.
/// Doesn't count the size of 'listchars' "eol".
///
/// @param wp
/// @param line
/// @param len
///
/// @return Number of cells the string will take on the screen.
static inline int win_linetabsize(win_T *wp, linenr_T lnum, char *line, colnr_T len)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_ALWAYS_INLINE
{
  CharsizeArg csarg;
  CSType const cstype = init_charsize_arg(&csarg, wp, lnum, line);
  if (cstype == kCharsizeFast) {
    return linesize_fast(&csarg, 0, len);
  } else {
    return linesize_regular(&csarg, 0, len);
  }
}
