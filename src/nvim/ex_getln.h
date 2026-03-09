#pragma once

#include <stddef.h>  // IWYU pragma: keep

#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/ex_getln_defs.h"  // IWYU pragma: keep
#include "nvim/option_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

/// flags used by vim_strsave_fnameescape()
enum {
  VSE_NONE   = 0,
  VSE_SHELL  = 1,  ///< escape for a shell command
  VSE_BUFFER = 2,  ///< escape for a ":buffer" command
};

#include "ex_getln.h.generated.h"

// Functions implemented in Rust (src/nvim-rs/) that replace C implementations:
#ifdef __cplusplus
extern "C" {
#endif

bool text_locked(void);
const char *get_text_locked_msg(void);
bool cmdline_overstrike(void);
bool cmdline_at_end(void);
bool is_in_cmdwin(void);
int cmdpreview_get_ns(void);
int get_cmdline_firstc(void);
bool cmdline_is_empty(void);
bool cmdline_is_search(void);
bool cmdline_is_ex_cmd(void);
int cmdline_level(void);
bool cmdline_at_max_level(void);
int cmdline_get_pos(void);
int cmdline_get_len(void);
bool cmdline_is_password(void);
int cmdline_parse_search_delim(const char *pattern, size_t len);
bool cmdline_is_literal_pattern(const char *pattern, size_t len);
bool cmdline_has_word_boundary(const char *pattern, size_t len);
int cmdline_check_bracket_balance(const char *expr, size_t len);
bool cmdline_is_expr_complete(const char *expr, size_t len);
int cmdline_find_last_token(const char *expr, size_t len);
bool cmdline_fname_needs_escape(const char *fname, size_t len);
bool cmdline_starts_with_tilde(const char *path, size_t len);
bool cmdline_expand_fuzzy_supported(const void *xp);
bool cmdline_expand_is_file_context(const void *xp);
bool cmdline_expand_uses_internal(const void *xp);

#ifdef __cplusplus
}
#endif
