#pragma once

#include "nvim/buffer_defs.h"  // IWYU pragma: keep
#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/option_defs.h"  // IWYU pragma: keep
#include "nvim/pos_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

// Opaque handles for quickfix state (for Rust FFI)
typedef void *QfStateHandle;
typedef void *EfmHandle;

/// flags for skip_vimgrep_pat()
enum {
  VGR_GLOBAL = 1,
  VGR_NOJUMP = 2,
  VGR_FUZZY  = 4,
};

// Rust quickfix implementations (called from external C files)
extern void rs_qf_view_result(bool split);
extern void rs_qf_jump_newwin(void *qi, int dir, int errornr, int forceit, bool newwin);

// Rust ex command implementations (called directly from command dispatch table)
extern void rs_ex_cc(void *eap);
extern void rs_ex_cnext(void *eap);
extern void rs_ex_cbelow(void *eap);
extern void rs_ex_cclose(void *eap);
extern void rs_ex_cbottom(void *eap);
extern void rs_ex_cwindow(void *eap);
extern void rs_ex_copen(void *eap);
extern void rs_ex_vimgrep(void *eap);
extern void rs_ex_helpgrep(void *eap);
extern void rs_qf_age(void *eap);
extern void rs_qf_history(void *eap);

// Forward declarations for Rust-implemented functions (exported under C names via #[export_name])
int qf_stack_get_bufnr(void);
void qf_free_all(win_T *wp);
#if defined(EXITFREE)
void check_quickfix_busy(void);
#endif
void qf_resize_stack(int n);
void ll_resize_stack(win_T *wp, int n);
linenr_T qf_current_entry(win_T *wp);
void f_getqflist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_setqflist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_getloclist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_setloclist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Forward declarations for Rust-implemented functions (exported via #[export_name])
bool qf_mark_adjust(buf_T *buf, win_T *wp, linenr_T line1, linenr_T line2, linenr_T amount,
                    linenr_T amount_after);
const char *did_set_quickfixtextfunc(optset_T *args);
int grep_internal(cmdidx_T cmdidx);
size_t qf_get_size(exarg_T *eap);
size_t qf_get_valid_size(exarg_T *eap);
size_t qf_get_cur_idx(exarg_T *eap);
int qf_get_cur_valid_idx(exarg_T *eap);

#include "quickfix_shim.h.generated.h"
