#pragma once

#include <stdbool.h>

#include "nvim/event/defs.h"  // IWYU pragma: keep

int ask_yesno(const char *str);
int get_keystroke(MultiQueue *events);
int prompt_for_input(char *prompt, int hl_id, bool one_key, bool *mouse_used);
