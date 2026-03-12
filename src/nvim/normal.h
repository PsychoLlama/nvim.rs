#pragma once

#include <stddef.h>  // IWYU pragma: keep

#include "nvim/macros_defs.h"
#include "nvim/normal_defs.h"  // IWYU pragma: keep
#include "nvim/pos_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

/// Values for find_ident_under_cursor()
enum {
  FIND_IDENT  = 1,  ///< find identifier (word)
  FIND_STRING = 2,  ///< find any string (WORD)
  FIND_EVAL   = 4,  ///< include "->", "[]" and "."
};

/// 'showcmd' buffer shared between normal.c and statusline.c
EXTERN char showcmd_buf[SHOWCMD_BUFLEN];

// Forward declarations for Rust-implemented functions (exported under C names via #[export_name])
void push_showcmd(void);
void pop_showcmd(void);
void do_nv_ident(int c1, int c2);
bool add_to_showcmd(int c);
bool find_decl(char *ptr, size_t len, bool locally, bool thisblock, int flags_arg);
void end_visual_mode(void);
void do_check_scrollbind(bool check);
void check_scrollbind(linenr_T vtopline_diff, int leftcol_diff);

#include "normal_shim.h.generated.h"
