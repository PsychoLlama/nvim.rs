// edit_shim.c: Rust FFI accessors for edit crate.

#include <stdbool.h>
#include <stddef.h>

#include "nvim/ex_docmd.h"
#include "nvim/getchar.h"
#include "nvim/globals.h"
#include "nvim/option_vars.h"
#include "nvim/strings.h"

#include "edit_shim.c.generated.h"

// General cpo/cmdmod accessors (migrated from ex_cmds_shim.c)
bool nvim_p_cpo_has_backspace(void) { return vim_strchr(p_cpo, CPO_BACKSPACE) != NULL; }
bool nvim_p_cpo_has_replcnt(void) { return vim_strchr(p_cpo, CPO_REPLCNT) != NULL; }
bool nvim_cmod_keepjumps(void) { return (cmdmod.cmod_flags & CMOD_KEEPJUMPS) != 0; }
// Insert mode command/cmdline accessor (migrated from ex_cmds_shim.c)
void nvim_do_cmdline_getcmdkeycmd(void) { do_cmdline(NULL, getcmdkeycmd, NULL, 0); }
