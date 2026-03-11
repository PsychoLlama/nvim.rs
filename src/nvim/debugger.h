#pragma once

#include <stdbool.h>

#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/pos_defs.h"      // IWYU pragma: keep

// Declarations for functions implemented in Rust (via #[export_name]).
// These replace auto-generated declarations for deleted C thin wrappers.
#ifndef DLLEXPORT
#  ifdef MSWIN
#    define DLLEXPORT __declspec(dllexport)
#  else
#    define DLLEXPORT
#  endif
#endif

DLLEXPORT void do_debug(char *cmd);
DLLEXPORT void ex_debug(exarg_T *eap);
DLLEXPORT void ex_debuggreedy(exarg_T *eap);
DLLEXPORT void dbg_check_breakpoint(exarg_T *eap);
DLLEXPORT bool dbg_check_skipped(exarg_T *eap);
DLLEXPORT void ex_breakadd(exarg_T *eap);
DLLEXPORT void ex_breakdel(exarg_T *eap);
DLLEXPORT void ex_breaklist(exarg_T *eap);
DLLEXPORT linenr_T dbg_find_breakpoint(bool file, char *fname, linenr_T after);
DLLEXPORT bool has_profiling(bool file, char *fname, bool *fp);
DLLEXPORT void dbg_breakpoint(char *name, linenr_T lnum);

#include "debugger.h.generated.h"
