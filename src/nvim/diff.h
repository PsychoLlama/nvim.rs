#pragma once

#include <stdbool.h>

#include "nvim/buffer_defs.h"  // IWYU pragma: keep
#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/macros_defs.h"
#include "nvim/pos_defs.h"  // IWYU pragma: keep

// Value set from 'diffopt'.
EXTERN int diff_context INIT( = 6);  ///< context for folds
EXTERN int diff_foldcolumn INIT( = 2);  ///< 'foldcolumn' for diff mode
EXTERN bool diff_need_scrollbind INIT( = false);

EXTERN bool need_diff_redraw INIT( = false);  ///< need to call diff_redraw()

// Rust-migrated functions (exported via #[export_name]).
#include "nvim/eval/funcs.h"  // IWYU pragma: keep (for typval_T, EvalFuncData)
void f_diff_filler(typval_T *args, typval_T *rvar, EvalFuncData data);
void f_diff_hlID(typval_T *args, typval_T *rvar, EvalFuncData data);
void diff_redraw(bool dofold);
void diff_win_options(win_T *wp, bool addbuf);
void ex_diffoff(exarg_T *eap);
void ex_diffpatch(exarg_T *eap);
void ex_diffsplit(exarg_T *eap);
void ex_diffthis(exarg_T *eap);
void ex_diffupdate(exarg_T *eap);
void nv_diffgetput(bool put, size_t count);

#include "diff_shim.h.generated.h"