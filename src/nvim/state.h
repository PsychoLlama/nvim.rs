#pragma once

#include "nvim/state_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

// Forward declarations for Rust-implemented functions (exported under C names via #[export_name])
int get_real_state(void);
bool get_was_safe_state(void);

#include "state.h.generated.h"
