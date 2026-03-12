#pragma once

#include <stdio.h>  // IWYU pragma: keep

#include "nvim/api/private/defs.h"  // IWYU pragma: keep
#include "nvim/api/private/helpers.h"
#include "nvim/cmdexpand_defs.h"  // IWYU pragma: keep
#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/macros_defs.h"
#include "nvim/option_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

/// flags for buf_copy_options()
enum {
  BCO_ENTER  = 1,  ///< going to enter the buffer
  BCO_ALWAYS = 2,  ///< always copy the options
  BCO_NOHELP = 4,  ///< don't touch the help related options
};

/// Flags for option-setting functions
///
/// When OPT_GLOBAL and OPT_LOCAL are both missing, set both local and global
/// values, get local value.
typedef enum {
  OPT_GLOBAL    = 0x01,  ///< Use global value.
  OPT_LOCAL     = 0x02,  ///< Use local value.
  OPT_MODELINE  = 0x04,  ///< Option in modeline.
  OPT_WINONLY   = 0x08,  ///< Only set window-local options.
  OPT_NOWIN     = 0x10,  ///< Don’t set window-local options.
  OPT_ONECOLUMN = 0x20,  ///< list options one per line
  OPT_NO_REDRAW = 0x40,  ///< ignore redraw flags on option
  OPT_SKIPRTP   = 0x80,  ///< "skiprtp" in 'sessionoptions'
} OptionSetFlags;

/// Get name of OptValType as a string.
static inline const char *optval_type_get_name(const OptValType type)
{
  switch (type) {
  case kOptValTypeNil:
    return "nil";
  case kOptValTypeBoolean:
    return "boolean";
  case kOptValTypeNumber:
    return "number";
  case kOptValTypeString:
    return "string";
  }
  UNREACHABLE;
}

// OptVal helper macros.
#define NIL_OPTVAL ((OptVal) { .type = kOptValTypeNil })
#define BOOLEAN_OPTVAL(b) ((OptVal) { .type = kOptValTypeBoolean, .data.boolean = b })
#define NUMBER_OPTVAL(n) ((OptVal) { .type = kOptValTypeNumber, .data.number = n })
#define STRING_OPTVAL(s) ((OptVal) { .type = kOptValTypeString, .data.string = s })

#define CSTR_AS_OPTVAL(s) STRING_OPTVAL(cstr_as_string(s))
#define CSTR_TO_OPTVAL(s) STRING_OPTVAL(cstr_to_string(s))
#define STATIC_CSTR_AS_OPTVAL(s) STRING_OPTVAL(STATIC_CSTR_AS_STRING(s))
#define STATIC_CSTR_TO_OPTVAL(s) STRING_OPTVAL(STATIC_CSTR_TO_STRING(s))

// Phase 34+: Rust-exported functions (via #[export_name])
void check_options(void);
int was_set_insecurely(win_T *wp, OptIndex opt_idx, int opt_flags);
uint32_t *insecure_flag(win_T *wp, OptIndex opt_idx, int opt_flags);
void check_redraw_for(buf_T *buf, win_T *win, uint32_t flags);
void set_option_sctx(OptIndex opt_idx, int opt_flags, sctx_T script_ctx);
OptIndex find_option_len(const char *name, size_t len);
OptIndex find_option(const char *name);
OptVal get_option_value(OptIndex opt_idx, int opt_flags);
int makeset(FILE *fd, int opt_flags, int local_only);
int makefoldset(FILE *fd);
void vimrc_found(char *fname, char *envname);
void reset_modifiable(void);
void set_iminsert_global(buf_T *buf);
void set_imsearch_global(buf_T *buf);
int fill_culopt_flags(char *val, win_T *wp);
void set_init_tablocal(void);
void set_init_3(void);
void set_helplang_default(const char *lang);
void set_title_defaults(void);
void ex_set(exarg_T *eap);
int do_set(char *arg, int opt_flags);
void set_options_bin(int oldval, int newval, int opt_flags);
void redraw_titles(void);
void check_blending(win_T *wp);
OptVal get_option_default(OptIndex opt_idx, int opt_flags);
int string_to_key(char *arg);
OptVal get_tty_option(const char *name);
bool set_tty_option(const char *name, char *value);
const char *set_option_value(OptIndex opt_idx, OptVal value, int opt_flags);
const char *set_option_value_handle_tty(const char *name, OptIndex opt_idx, OptVal value, int opt_flags);
void set_option_value_give_err(OptIndex opt_idx, OptVal value, int opt_flags);
void set_option_direct(OptIndex opt_idx, OptVal value, int opt_flags, scid_T set_sid);
void set_option_direct_for(OptIndex opt_idx, OptVal value, int opt_flags, scid_T set_sid, OptScope scope, void *from);
void set_context_in_set_cmd(expand_T *xp, char *arg, int opt_flags);
int ExpandOldSetting(int *numMatches, char ***matches);
int ExpandStringSetting(expand_T *xp, regmatch_T *regmatch, int *numMatches, char ***matches);
unsigned get_bkc_flags(buf_T *buf);
char *get_flp_value(buf_T *buf);
unsigned get_ve_flags(win_T *wp);
int get_fileformat_force(const buf_T *buf, const exarg_T *eap);
void set_fileformat(int eol_style, int opt_flags);
size_t copy_option_part(char **option, char *buf, size_t maxlen, char *sep_chars);
#if defined(EXITFREE)
void free_all_options(void);
#endif

#include "option_shim.h.generated.h"
