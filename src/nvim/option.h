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
void set_init_tablocal(void);
void set_init_3(void);
void set_helplang_default(const char *lang);
void set_title_defaults(void);
void ex_set(exarg_T *eap);
int do_set(char *arg, int opt_flags);
void set_options_bin(int oldval, int newval, int opt_flags);
void redraw_titles(void);
void check_blending(win_T *wp);
#if defined(EXITFREE)
void free_all_options(void);
#endif

#include "option_shim.h.generated.h"
