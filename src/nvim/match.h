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
