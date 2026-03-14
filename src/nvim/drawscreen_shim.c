// drawscreen_shim.c: C accessor wrappers for the Rust drawscreen crate.
//
// These are thin wrappers that give Rust access to globals and functions
// from drawscreen.c and related modules.

#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/digraph.h"
#include "nvim/drawscreen.h"
#include "nvim/globals.h"
#include "nvim/message.h"
#include "nvim/statusline.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

#include "drawscreen_shim.c.generated.h"

// ---------------------------------------------------------------------------
// showmode helpers
// ---------------------------------------------------------------------------

/// Call msg_grid_validate() (not in public headers).
void nvim_drawscreen_msg_grid_validate(void)
{
  msg_grid_validate();
}

/// Call msg_check_for_delay(false).
void nvim_drawscreen_msg_check_for_delay(void)
{
  msg_check_for_delay(false);
}

/// Call msg_clr_cmdline() (already declared as nvim_msg_clr_cmdline_wrap in
/// insexpand_shim.c; provide a second alias here for use by drawscreen).
void nvim_drawscreen_msg_clr_cmdline(void)
{
  msg_clr_cmdline();
}

/// Return 1 if edit_submode is NULL, else 0.
int nvim_drawscreen_edit_submode_is_null(void)
{
  return edit_submode == NULL ? 1 : 0;
}

/// Return the edit_submode pointer (may be NULL; Rust must not free it).
const char *nvim_drawscreen_edit_submode_ptr(void)
{
  return edit_submode;
}

/// Return 1 if edit_submode_pre is NULL, else 0.
int nvim_drawscreen_edit_submode_pre_is_null(void)
{
  return edit_submode_pre == NULL ? 1 : 0;
}

/// Return the edit_submode_pre pointer (may be NULL; Rust must not free it).
const char *nvim_drawscreen_edit_submode_pre_ptr(void)
{
  return edit_submode_pre;
}

/// Call get_keymap_str(curwin, " (%s)", NameBuff, MAXPATHL).
/// Returns the result length (>0 means something was written to NameBuff).
int nvim_drawscreen_get_keymap_str(void)
{
  return get_keymap_str(curwin, " (%s)", NameBuff, MAXPATHL);
}

/// Return the NameBuff pointer (global char buffer, valid until next C call).
const char *nvim_drawscreen_namebuff_ptr(void)
{
  return NameBuff;
}

/// Return wp->w_p_arab (arabic option).
int nvim_win_get_w_p_arab(win_T *wp)
{
  return wp ? (int)wp->w_p_arab : 0;
}
