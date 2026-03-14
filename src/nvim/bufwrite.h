#pragma once

#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/pos_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

int buf_write(buf_T *buf, char *fname, char *sfname, linenr_T start, linenr_T end, exarg_T *eap,
              bool append, bool forceit, bool reset_changed, bool filtering);

#include "bufwrite.h.generated.h"
