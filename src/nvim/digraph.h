#pragma once

#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/garray_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

#include "digraph.h.generated.h"
int do_digraph(int c);
int get_digraph(bool cmdline);
int digraph_get(int char1, int char2, bool meta_char);
