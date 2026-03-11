#pragma once

#include <stdbool.h>

#include "nvim/buffer_defs.h"
#include "nvim/pos_defs.h"
#include "nvim/types_defs.h"

#ifndef DLLEXPORT
#  ifdef MSWIN
#    define DLLEXPORT __declspec(dllexport)
#  else
#    define DLLEXPORT
#  endif
#endif

DLLEXPORT void syntax_start(win_T *wp, linenr_T lnum);
DLLEXPORT void syntax_end_parsing(win_T *wp, linenr_T lnum);
DLLEXPORT bool syntax_check_changed(linenr_T lnum);
DLLEXPORT int get_syntax_attr(const colnr_T col, bool *const can_spell, const bool keep_state);
DLLEXPORT bool syntax_present(win_T *win);
DLLEXPORT int syn_get_sub_char(void);
DLLEXPORT int syn_get_foldlevel(win_T *wp, linenr_T lnum);
