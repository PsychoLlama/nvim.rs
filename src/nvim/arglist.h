#pragma once

#include "nvim/arglist_defs.h"  // IWYU pragma: keep
#include "nvim/buffer_defs.h"  // IWYU pragma: keep
#include "nvim/cmdexpand_defs.h"  // IWYU pragma: keep
#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/garray_defs.h"  // IWYU pragma: keep

#include "arglist.h.generated.h"

// Declarations for functions implemented in Rust via #[export_name].
// These were previously declared as forward declarations in arglist.c.
// arglist management
void alist_clear(alist_T *al);
void alist_init(alist_T *al);
void alist_unlink(alist_T *al);
void alist_new(void);
void alist_add(alist_T *al, char *fname, int set_fnum);
void alist_set(alist_T *al, int count, char **files, int use_curbuf, int *fnum_list, int fnum_len);
int get_arglist_exp(char *str, int *fcountp, char ***fnamesp, bool wig);
// arglist query
char *alist_name(aentry_T *aep);
char *get_arglist_name(expand_T *xp, int idx);
bool editing_arg_idx(win_T *win);
void check_arg_idx(win_T *win);
char *arg_all(void);
void set_arglist(char *str);
// ex commands
void do_argfile(exarg_T *eap, int argn);
void ex_previous(exarg_T *eap);
void ex_rewind(exarg_T *eap);
void ex_last(exarg_T *eap);
void ex_argument(exarg_T *eap);
void ex_next(exarg_T *eap);
void ex_argdedupe(exarg_T *eap);
void ex_args(exarg_T *eap);
void ex_argedit(exarg_T *eap);
void ex_argadd(exarg_T *eap);
void ex_argdelete(exarg_T *eap);
void ex_all(exarg_T *eap);
