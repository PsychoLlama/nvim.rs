#pragma once

#include <stdbool.h>
#include <stdint.h>

#include "nvim/cmdexpand_defs.h"  // IWYU pragma: keep
#include "nvim/eval/typval_defs.h"
#include "nvim/pos_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"

/// Prototype of C function that implements Vimscript function
typedef void (*VimLFunc)(typval_T *args, typval_T *rvar, EvalFuncData data);

/// Special flags for base_arg @see EvalFuncDef
enum {
  BASE_NONE = 0,          ///< Not a method (no base argument).
  BASE_LAST = UINT8_MAX,  ///< Use the last argument as the method base.
};

/// Structure holding Vimscript function definition
typedef struct {
  char *name;         ///< Name of the function.
  uint8_t min_argc;   ///< Minimal number of arguments.
  uint8_t max_argc;   ///< Maximal number of arguments.
  uint8_t base_arg;   ///< Method base arg # (1-indexed), BASE_NONE or BASE_LAST.
  bool fast;          ///< Can be run in |api-fast| events
  VimLFunc func;      ///< Function implementation.
  EvalFuncData data;  ///< Userdata for function implementation.
} EvalFuncDef;

#include "eval/funcs.h.generated.h"

// Phase 12: var2fpos exported from Rust (eval/src/indexing.rs via #[export_name])
// FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL deliberately omitted here
// since this header is included without macros_defs.h in some TUs.
pos_T *var2fpos(const typval_T *tv, bool dollar_lnum, int *ret_fnum, bool charcol);

// Phase 29: get_user_input moved from funcs.c to funcs_shim.c
void get_user_input(const typval_T *argvars, typval_T *rettv, bool inputdialog, bool secret);

// Phase 32: functions moved from funcs.c to funcs_shim.c
void execute_common(typval_T *argvars, typval_T *rettv, int arg_off);
win_T *get_optional_window(typval_T *argvars, int idx);
void f_jobstart(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_jobstop(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
