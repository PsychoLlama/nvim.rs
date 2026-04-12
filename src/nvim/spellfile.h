#pragma once

#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/spell_defs.h"  // IWYU pragma: keep

// Forward declaration of the internal word-tree node type.
// Full definition is in spellfile.c; external users only see an opaque pointer.
typedef struct wordnode_S wordnode_T;

// Forward declaration of the spell compilation state.
// Full definition is in spellfile.c; external users only see an opaque pointer.
typedef struct spellinfo_S spellinfo_T;

// Forward declaration of the spell arena block.
// Full definition is in spellfile.c; external users only see an opaque pointer.
typedef struct sblock_S sblock_T;

// spell_check_msm is implemented in Rust; declare it here for callers.
int spell_check_msm(void);

// Functions now exported directly from Rust (no C wrapper).
#include <stdbool.h>
#include <stddef.h>
slang_T *spell_load_file(char *fname, char *lang, slang_T *old_lp, bool silent);
void suggest_load_files(void);
void *getroom(spellinfo_T *spin, size_t len, bool align);
void free_blocks(sblock_T *bl);
void ex_mkspell(exarg_T *eap);
void ex_spell(exarg_T *eap);

#include "spellfile.h.generated.h"
