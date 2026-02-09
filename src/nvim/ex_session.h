#pragma once

#include <stdio.h>  // IWYU pragma: keep

#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep

// Rust FFI: session writer
extern int rs_put_eol(FILE *fd);
extern int rs_put_line(FILE *fd, const char *s);

#include "ex_session.h.generated.h"
