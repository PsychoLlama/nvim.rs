#pragma once

#include <stdint.h>  // IWYU pragma: keep

#include "nvim/cmdexpand_defs.h"  // IWYU pragma: keep
#include "nvim/option_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

typedef enum {
  kFillchars,
  kListchars,
} CharsOption;

// Implemented in Rust (src/nvim-rs/option/src/validate.rs)
const char *check_stl_option(char *s);

// Implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)
#include "nvim/option_defs.h"  // optset_T
const char *did_set_helplang(optset_T *args);
const char *did_set_breakat(optset_T *args);
const char *did_set_backupext_or_patchmode(optset_T *args);
const char *did_set_mousescroll(optset_T *args);
const char *did_set_str_generic(optset_T *args);
void didset_string_options(void);
const char *did_set_shada(optset_T *args);
const char *did_set_completeitemalign(optset_T *args);
const char *did_set_titleiconstring(optset_T *args, int flagval);
const char *did_set_colorcolumn(optset_T *args);
// Phase 1: Simple did_set_* callbacks (Rust)
const char *did_set_completeopt(optset_T *args);
const char *did_set_tagcase(optset_T *args);
const char *did_set_virtualedit(optset_T *args);
const char *did_set_signcolumn(optset_T *args);
const char *did_set_iskeyword(optset_T *args);
const char *did_set_isopt(optset_T *args);
const char *did_set_fileformat(optset_T *args);
const char *did_set_varsofttabstop(optset_T *args);
const char *did_set_vartabstop(optset_T *args);
const char *did_set_complete(optset_T *args);

// Phase 2: Complex did_set_* callbacks (Rust)
const char *did_set_background(optset_T *args);
const char *did_set_buftype(optset_T *args);
const char *did_set_encoding(optset_T *args);
const char *did_set_keymap(optset_T *args);
const char *did_set_statustabline_rulerformat(optset_T *args, bool rulerformat, bool statuscolumn);

// Phase 3: Chars, signcolumn, border (Rust)
#include "nvim/buffer_defs.h"  // win_T
const char *set_chars_option(win_T *wp, const char *value, CharsOption what, bool apply,
                             char *errbuf, size_t errbuflen);
const char *did_set_chars_option(optset_T *args);
const char *check_chars_options(void);
int check_signcolumn(char *scl, win_T *wp);
bool parse_border_opt(char *border_opt);

// Implemented in Rust (src/nvim-rs/optionstr/src/errors.rs)
char *illegal_char(char *errbuf, size_t errbuflen, int c);

// Implemented in Rust (src/nvim-rs/optionstr/src/lib.rs)
void free_string_option(char *p);
void clear_string_option(char **pp);
void check_string_option(char **pp);

// Implemented in Rust (src/nvim-rs/optionstr/src/chars.rs)
#include "nvim/cmdexpand_defs.h"  // expand_T
char *get_fillchars_name(expand_T *xp, int idx);
char *get_listchars_name(expand_T *xp, int idx);

// Implemented in Rust (src/nvim-rs/optionstr/src/expand.rs)
#include "nvim/option_defs.h"  // optexpand_T
int expand_set_concealcursor(optexpand_T *args, int *numMatches, char ***matches);
int expand_set_cpoptions(optexpand_T *args, int *numMatches, char ***matches);
int expand_set_formatoptions(optexpand_T *args, int *numMatches, char ***matches);
int expand_set_mouse(optexpand_T *args, int *numMatches, char ***matches);
int expand_set_shortmess(optexpand_T *args, int *numMatches, char ***matches);
int expand_set_whichwrap(optexpand_T *args, int *numMatches, char ***matches);
int expand_set_str_generic(optexpand_T *args, int *numMatches, char ***matches);
int expand_set_encoding(optexpand_T *args, int *numMatches, char ***matches);
int expand_set_winhighlight(optexpand_T *args, int *numMatches, char ***matches);
int expand_set_chars_option(optexpand_T *args, int *numMatches, char ***matches);
int expand_set_diffopt(optexpand_T *args, int *numMatches, char ***matches);
int expand_set_eventignore(optexpand_T *args, int *numMatches, char ***matches);
char *get_fileformat_name(expand_T *xp, int idx);
int check_ff_value(char *p);

#include "optionstr.h.generated.h"
