#pragma once

#include <stdbool.h>

#include "nvim/garray_defs.h"  // IWYU pragma: keep

#include "spellsuggest.h.generated.h"

// Rust-exported function declarations (not in generated header)
void spell_suggest_list(garray_T *gap, char *word, int maxcount, bool need_cap, bool interactive);
int spell_check_sps(void);
