#pragma once

#include "nvim/vterm/vterm_defs.h"

#include "vterm/state.h.generated.h"

// Hand-written prototype: grammar skips function-pointer params.
void nvim_vterm_scroll_rect(VTermRect rect, int downward, int rightward,
                            int (*moverect)(VTermRect dest, VTermRect src, void *user),
                            int (*erase)(VTermRect rect, int selective, void *user),
                            void *user);
