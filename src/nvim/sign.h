#pragma once

#include <stdbool.h>
#include <stddef.h>

#include "nvim/cmdexpand_defs.h"  // IWYU pragma: keep
#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/sign_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep
#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep

#include "sign.h.generated.h"

// Rust-implemented sign functions (exported via #[export_name])
bool buf_has_signs(const buf_T *buf);
size_t describe_sign_text(char *buf, schar_T *sign_text);
int init_sign_text(sign_T *sp, schar_T *sign_text, char *text);
list_T *get_buffer_signs(buf_T *buf);
void free_signs(void);
void ex_sign(exarg_T *eap);
char *get_sign_name(expand_T *xp, int idx);
void set_context_in_sign_cmd(expand_T *xp, char *arg);
void f_sign_define(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_sign_getdefined(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_sign_getplaced(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_sign_jump(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_sign_place(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_sign_placelist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_sign_undefine(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_sign_unplace(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_sign_unplacelist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
