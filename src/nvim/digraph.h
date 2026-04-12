#pragma once

#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/garray_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

#include "digraph.h.generated.h"
int do_digraph(int c);
int get_digraph(bool cmdline);
int digraph_get(int char1, int char2, bool meta_char);
void f_digraph_get(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_digraph_getlist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_digraph_set(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_digraph_setlist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Keymap functions moved to Rust (digraph/src/keymap.rs)
void keymap_ga_clear(garray_T *kmap_ga);
int get_keymap_str(win_T *wp, char *fmt, char *buf, int len);
