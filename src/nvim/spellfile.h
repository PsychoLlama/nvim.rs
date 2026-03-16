#pragma once

#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/spell_defs.h"  // IWYU pragma: keep

// Forward declaration of the internal word-tree node type.
// Full definition is in spellfile.c; external users only see an opaque pointer.
typedef struct wordnode_S wordnode_T;

// Forward declaration of the spell compilation state.
// Full definition is in spellfile.c; external users only see an opaque pointer.
typedef struct spellinfo_S spellinfo_T;

#include "spellfile.h.generated.h"
