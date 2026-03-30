#pragma once

#include <stdbool.h>
#include <stddef.h>

#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

#include "eval/fs.h.generated.h"

// modify_fname moved to funcs_shim.c (no generated header there)
int modify_fname(char *src, bool tilde_file, size_t *usedlen, char **fnamep, char **bufp,
                 size_t *fnamelen);
