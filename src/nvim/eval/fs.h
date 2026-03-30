#pragma once

#include <stdbool.h>
#include <stddef.h>

#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

// modify_fname: filename modifier engine (defined in eval/funcs_shim.c)
int modify_fname(char *src, bool tilde_file, size_t *usedlen, char **fnamep, char **bufp,
                 size_t *fnamelen);
