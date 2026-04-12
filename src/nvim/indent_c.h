#pragma once

#include <stdbool.h>
#include <stdint.h>

#include "nvim/eval/typval_defs.h"
#include "nvim/pos_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

/// Result of findmatchlimit, passed across FFI boundary.
typedef struct {
  bool found;
  int lnum;
  int col;
} FindMatchResult;

/// All cindent option fields, used to bulk-set buffer options from Rust.
typedef struct {
  int ind_level;
  int ind_open_imag;
  int ind_no_brace;
  int ind_first_open;
  int ind_open_extra;
  int ind_close_extra;
  int ind_open_left_imag;
  int ind_jump_label;
  int ind_case;
  int ind_case_code;
  int ind_case_break;
  int ind_scopedecl;
  int ind_scopedecl_code;
  int ind_param;
  int ind_func_type;
  int ind_cpp_baseclass;
  int ind_continuation;
  int ind_unclosed;
  int ind_unclosed2;
  int ind_unclosed_noignore;
  int ind_unclosed_wrapped;
  int ind_unclosed_whiteok;
  int ind_matching_paren;
  int ind_paren_prev;
  int ind_comment;
  int ind_in_comment;
  int ind_in_comment2;
  int ind_maxparen;
  int ind_maxcomment;
  int ind_java;
  int ind_js;
  int ind_keep_case_label;
  int ind_cpp_namespace;
  int ind_if_for_while;
  int ind_hash_comment;
  int ind_cpp_extern_c;
  int ind_pragma;
} CindentOptions;

#include "indent_c.h.generated.h"

// Functions implemented in Rust (src/nvim-rs/indent_c/src/lib.rs and indent/src/lib.rs).
// These are exported directly from Rust via #[export_name]; declarations kept here
// so C callers don't need to change.
int is_pos_in_string(const char *line, colnr_T col);
bool cin_is_cinword(const char *line);
bool cindent_on(void);
bool in_cinkeys(int keytyped, int when, bool line_is_empty);
