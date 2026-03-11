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

#include "optionstr.h.generated.h"
