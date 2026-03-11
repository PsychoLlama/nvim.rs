// File searching functions for 'path', 'tags' and 'cdpath' options.
//
// Core search functions (vim_findfile_init, vim_findfile, vim_findfile_cleanup,
// etc.) are implemented in the Rust filesearch crate.
//
// This file contains:
//   VIsual_active_get()        - C accessor for Rust
//   eval_includeexpr()         - requires eval infrastructure (deferred)
//   do_autocmd_dirchanged()    - requires VimL dict/autocmd (deferred)

#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>

#include "nvim/api/private/helpers.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/file_search.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/vim_defs.h"

// Accessor functions for Rust
int VIsual_active_get(void) { return VIsual_active ? 1 : 0; }

/// Evaluate 'includeexpr' and return the result (caller must free).
char *eval_includeexpr(const char *const ptr, const size_t len)
{
  const sctx_T save_sctx = current_sctx;
  set_vim_var_string(VV_FNAME, ptr, (ptrdiff_t)len);
  current_sctx = curbuf->b_p_script_ctx[kBufOptIncludeexpr];

  char *res = eval_to_string_safe(curbuf->b_p_inex,
                                  was_set_insecurely(curwin, kOptIncludeexpr, OPT_LOCAL),
                                  true);

  set_vim_var_string(VV_FNAME, NULL, 0);
  current_sctx = save_sctx;
  return res;
}

void do_autocmd_dirchanged(char *new_dir, CdScope scope, CdCause cause, bool pre)
{
  static bool recursive = false;

  event_T event = pre ? EVENT_DIRCHANGEDPRE : EVENT_DIRCHANGED;

  if (recursive || !has_event(event)) {
    // No autocommand was defined or we changed
    // the directory from this autocommand.
    return;
  }

  recursive = true;

  save_v_event_T save_v_event;
  dict_T *dict = get_v_event(&save_v_event);
  char buf[8];

  switch (scope) {
  case kCdScopeGlobal:
    snprintf(buf, sizeof(buf), "global");
    break;
  case kCdScopeTabpage:
    snprintf(buf, sizeof(buf), "tabpage");
    break;
  case kCdScopeWindow:
    snprintf(buf, sizeof(buf), "window");
    break;
  case kCdScopeInvalid:
    // Should never happen.
    abort();
  }

#ifdef BACKSLASH_IN_FILENAME
  char new_dir_buf[MAXPATHL];
  STRCPY(new_dir_buf, new_dir);
  slash_adjust(new_dir_buf);
  new_dir = new_dir_buf;
#endif

  if (pre) {
    tv_dict_add_str(dict, S_LEN("directory"), new_dir);
  } else {
    tv_dict_add_str(dict, S_LEN("cwd"), new_dir);
  }
  tv_dict_add_str(dict, S_LEN("scope"), buf);
  tv_dict_add_bool(dict, S_LEN("changed_window"), cause == kCdCauseWindow);
  tv_dict_set_keys_readonly(dict);

  switch (cause) {
  case kCdCauseManual:
  case kCdCauseWindow:
    break;
  case kCdCauseAuto:
    snprintf(buf, sizeof(buf), "auto");
    break;
  case kCdCauseOther:
    // Should never happen.
    abort();
  }

  apply_autocmds(event, buf, new_dir, false, curbuf);

  restore_v_event(dict, &save_v_event);

  recursive = false;
}
