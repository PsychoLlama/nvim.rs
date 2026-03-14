#pragma once

#include "nvim/api/private/defs.h"  // IWYU pragma: keep
#include "nvim/tui/terminfo_defs.h"

typedef struct {
  long num;
  char *string;
} TPVAR;

// Functions implemented in Rust (nvim-tui crate)
extern bool terminfo_is_term_family(const char *term, const char *family);
extern bool terminfo_is_bsd_console(const char *term);

#include "tui/terminfo.h.generated.h"
