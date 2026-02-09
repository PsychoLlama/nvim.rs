// input.c: high level functions for prompting the user or input
// like yes/no or number prompts.
//
// The implementations live in Rust (src/nvim-rs/input/src/lib.rs).
// This file provides thin C wrappers that delegate to the Rust code.

#include <stdbool.h>
#include <stdint.h>

#include "nvim/cmdexpand_defs.h"
#include "nvim/highlight_defs.h"
#include "nvim/input.h"
#include "nvim/keycodes.h"

#include "input.c.generated.h"  // IWYU pragma: export

// Static assertions for hardcoded constants in Rust
_Static_assert(HLF_R == 18, "HLF_R mismatch");
_Static_assert(K_SPECIAL == 0x80, "K_SPECIAL mismatch");
_Static_assert(KS_MODIFIER == 252, "KS_MODIFIER mismatch");
_Static_assert(KS_EXTRA == 253, "KS_EXTRA mismatch");
_Static_assert(KE_IGNORE == 53, "KE_IGNORE mismatch");
_Static_assert(KE_LEFTMOUSE == 44, "KE_LEFTMOUSE mismatch");
_Static_assert(EXPAND_NOTHING == 0, "EXPAND_NOTHING mismatch");

// Rust implementations
extern int rs_ask_yesno(const char *str);
extern int rs_get_keystroke(void *events);
extern int rs_prompt_for_input(char *prompt, int hl_id, bool one_key, bool *mouse_used);

int ask_yesno(const char *const str)
{
  return rs_ask_yesno(str);
}

int get_keystroke(MultiQueue *events)
{
  return rs_get_keystroke(events);
}

int prompt_for_input(char *prompt, int hl_id, bool one_key, bool *mouse_used)
{
  return rs_prompt_for_input(prompt, hl_id, one_key, mouse_used);
}
