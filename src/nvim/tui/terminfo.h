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
extern const TerminfoEntry *rs_terminfo_from_builtin(const char *term, char **termname);
extern size_t rs_terminfo_fmt(char *buf_start, const char *buf_end, const char *str, TPVAR *params);
extern String rs_terminfo_info_msg(const TerminfoEntry *ti, const char *termname, bool from_db);

#include "tui/terminfo.h.generated.h"
