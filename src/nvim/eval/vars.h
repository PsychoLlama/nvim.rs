#pragma once

#include <stdbool.h>  // IWYU pragma: keep
#include <stddef.h>  // IWYU pragma: keep

#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/eval_defs.h"
#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/garray_defs.h"  // IWYU pragma: keep
#include "nvim/hashtab_defs.h"  // IWYU pragma: keep
#include "nvim/option_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

// Phase 13: declarations for functions implemented in Rust (set_var.rs / lookup.rs).
bool before_set_vvar(const char *varname, dictitem_T *di, typval_T *tv,
                     bool copy, bool watched, bool *type_error);
void set_var(const char *name, size_t name_len, typval_T *tv, bool copy);
void set_var_const(const char *name, size_t name_len, typval_T *tv, bool copy, bool is_const);

/// Array mapping values from MessagePackType to corresponding list pointers
extern const list_T *eval_msgpack_type_lists[NUM_MSGPACK_TYPES];

/// Accessor for eval_msgpack_type_lists[idx] used by Rust decode module.
list_T *nvim_eval_msgpack_type_list(int idx);

#include "eval/vars.h.generated.h"
