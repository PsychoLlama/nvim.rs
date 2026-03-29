#pragma once

#include "nvim/buffer_defs.h"  // IWYU pragma: keep
#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

#include "match.h.generated.h"

// Rust-exported symbols (via #[export_name]) replacing C wrapper functions.
// These are no longer generated from match.c but provided by libnvim_rs.a.
extern void clear_matches(win_T *wp);
extern void init_search_hl(win_T *wp, match_T *search_hl);
extern void prepare_search_hl(win_T *wp, match_T *search_hl, linenr_T lnum);
extern void get_search_match_hl(win_T *wp, match_T *search_hl, colnr_T col, int *char_attr);
extern bool prepare_search_hl_line(win_T *wp, linenr_T lnum, colnr_T mincol, char **line,
                                   match_T *search_hl, int *search_attr,
                                   bool *search_attr_from_match);
extern int update_search_hl(win_T *wp, linenr_T lnum, colnr_T col, char **line,
                            match_T *search_hl, int *has_match_conc, int *match_conc,
                            bool lcs_eol_todo, bool *on_last_col, bool *search_attr_from_match);
extern bool get_prevcol_hl_flag(win_T *wp, match_T *search_hl, colnr_T curcol);
extern void f_clearmatches(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_matchdelete(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_matchadd(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_matchaddpos(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void ex_match(exarg_T *eap);
