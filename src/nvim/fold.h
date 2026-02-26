#pragma once

#include <stdio.h>  // IWYU pragma: keep

#include "nvim/decoration_defs.h"  // IWYU pragma: keep
#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/fold_defs.h"  // IWYU pragma: keep
#include "nvim/garray_defs.h"  // IWYU pragma: keep
#include "nvim/macros_defs.h"
#include "nvim/pos_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

EXTERN int disable_fold_update INIT( = 0);

/// Result struct for fold level calculations (used by Rust FFI).
typedef struct {
  int lvl;
  int lvl_next;
  int start;
  int end;
} FoldLevelResult_C;

// Rust FFI: fold method checks and fold state queries
extern int rs_foldLevel(linenr_T lnum);
extern int rs_lineFolded(win_T *wp, linenr_T lnum);

// Phase 5 Pass 5: hasFolding/hasFoldingWin/nvim_hasFolding exported from Rust
extern bool hasFolding(win_T *win, linenr_T lnum, linenr_T *firstp, linenr_T *lastp);
extern bool hasFoldingWin(win_T *win, linenr_T lnum, linenr_T *firstp, linenr_T *lastp,
                          bool cache, foldinfo_T *infop);
extern int nvim_hasFolding(win_T *wp, linenr_T lnum, linenr_T *firstp, linenr_T *lastp);

// Phase 5 Pass 5: deleteFoldRecurse exported from Rust
extern void deleteFoldRecurse(buf_T *bp, garray_T *gap);
extern int rs_foldmethodIsManual(win_T *wp);
extern int rs_foldmethodIsIndent(win_T *wp);
extern int rs_foldmethodIsExpr(win_T *wp);
extern int rs_foldmethodIsMarker(win_T *wp);
extern int rs_foldmethodIsSyntax(win_T *wp);
extern int rs_foldmethodIsDiff(win_T *wp);
extern void rs_foldUpdate(win_T *wp, linenr_T top, linenr_T bot);
extern int rs_put_folds(FILE *fd, win_T *wp);

// Rust FFI: VimL fold functions (direct Rust implementations, no C wrapper needed)
extern void rs_f_foldclosed(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void rs_f_foldclosedend(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void rs_f_foldlevel(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void rs_f_foldtext(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void rs_f_foldtextresult(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

#include "fold_shim.h.generated.h"
