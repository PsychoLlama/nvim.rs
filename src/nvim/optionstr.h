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

#include "optionstr.h.generated.h"
