#pragma once

#include "nvim/garray_defs.h"
#include "nvim/pos_defs.h"
#include "nvim/types_defs.h"

/// Info used to pass info about a fold from the fold-detection code to the
/// code that displays the foldcolumn.
typedef struct {
  linenr_T fi_lnum;  ///< line number where fold starts
  int fi_level;      ///< level of the fold; when this is zero the
                     ///< other fields are invalid
  int fi_low_level;  ///< lowest fold level that starts in the same line
  linenr_T fi_lines;
} foldinfo_T;

/// The toplevel folds for each window are stored in the w_folds growarray.
/// Each toplevel fold can contain an array of second level folds in the
/// fd_nested growarray.
/// The info stored in both growarrays is the same: An array of fold_T.
typedef struct {
  linenr_T fd_top;              ///< first line of fold; for nested fold
                                ///< relative to parent
  linenr_T fd_len;              ///< number of lines in the fold
  garray_T fd_nested;           ///< array of nested folds
  char fd_flags;                ///< see FD_OPEN, FD_CLOSED, FD_LEVEL
  TriState fd_small;            ///< kTrue, kFalse, or kNone: fold smaller than
                                ///< 'foldminlines'; kNone applies to nested
                                ///< folds too
} fold_T;

enum { FOLD_TEXT_LEN = 51, };  ///< buffer size for get_foldtext()
