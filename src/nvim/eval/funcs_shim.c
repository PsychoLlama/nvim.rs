/// Shim functions for Rust-side inlining of eval/funcs.c accessor functions.
///
/// These minimal C accessors provide access to C globals and struct fields that
/// Rust cannot reach directly. They are called from the Rust implementations
/// in src/nvim-rs/eval/src/funcs/ modules.

#include <stdbool.h>
#include <stdint.h>

#include "nvim/ascii_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/cmdexpand.h"
#include "nvim/context.h"
#include "nvim/eval.h"
#include "nvim/eval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/message.h"
#include "nvim/os/os.h"
#include "nvim/register.h"
#include "nvim/state.h"
#include "nvim/syntax.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/globals.h"
#include "nvim/indent.h"
#include "nvim/memory.h"
#include "nvim/state_defs.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/version.h"

#include "eval/funcs_shim.c.generated.h"

// =============================================================================
// typval helpers
// =============================================================================

/// Set typval to v:null (VAR_SPECIAL / kSpecialVarNull).
void nvim_tv_set_special_null(typval_T *tv)
{
  tv->v_type = VAR_SPECIAL;
  tv->vval.v_special = kSpecialVarNull;
}

// =============================================================================
// Wildmenu mode helper
// =============================================================================

/// Check if wildmenu is active (for the VimL wildmenumode() function).
/// Returns 1 if active, 0 otherwise.
int nvim_eval_wildmenumode_check(void)
{
  return (wild_menu_showing || ((State & MODE_CMDLINE) && cmdline_pum_active())) ? 1 : 0;
}

// =============================================================================
// shiftwidth helpers
// =============================================================================

/// Get shiftwidth for current buffer at a specific column.
int nvim_eval_get_sw_value_col(int col)
{
  return get_sw_value_col(curbuf, (colnr_T)col, false);
}

/// Get shiftwidth for current buffer (no column).
int nvim_eval_get_sw_value(void)
{
  return get_sw_value(curbuf);
}

// =============================================================================
// mode() helper
// =============================================================================

/// Fill buf (4 bytes) with current mode string via get_mode().
void nvim_eval_get_mode(char *buf)
{
  get_mode(buf);
}

// =============================================================================
// non_zero_arg helper
// =============================================================================

/// Equivalent of non_zero_arg(&argvars[idx]) — returns 1 if true, 0 if false.
/// @param argvars  pointer to the first element of the argvars array
/// @param idx      which element to check (0-based)
int nvim_eval_non_zero_arg(typval_T *argvars, int idx)
{
  typval_T *tv = &argvars[idx];
  bool result = ((tv->v_type == VAR_NUMBER && tv->vval.v_number != 0)
                 || (tv->v_type == VAR_BOOL && tv->vval.v_bool == kBoolVarTrue)
                 || (tv->v_type == VAR_STRING
                     && tv->vval.v_string != NULL
                     && *tv->vval.v_string != NUL));
  return result ? 1 : 0;
}

/// Get ml_line_count for curbuf.
linenr_T nvim_eval_curbuf_ml_line_count(void)
{
  return curbuf->b_ml.ml_line_count;
}

// =============================================================================
// nextnonblank / prevnonblank helpers
// =============================================================================

/// Get lnum from argvars[0] (wraps tv_get_lnum for Rust).
int32_t nvim_eval_tv_get_lnum(typval_T *argvars)
{
  return (int32_t)tv_get_lnum(argvars);
}

// =============================================================================
// empty() helpers - VAR_BOOL / VAR_SPECIAL field access
// =============================================================================

/// Returns 1 if tv->vval.v_bool == kBoolVarTrue, 0 otherwise.
int nvim_eval_tv_bool_is_true(const typval_T *tv)
{
  return tv->vval.v_bool == kBoolVarTrue ? 1 : 0;
}

/// Returns 1 if tv->vval.v_special == kSpecialVarNull, 0 otherwise.
int nvim_eval_tv_special_is_null(const typval_T *tv)
{
  return tv->vval.v_special == kSpecialVarNull ? 1 : 0;
}

// =============================================================================
// ctxpop() / ctx_size() helpers
// =============================================================================

/// Pops the context stack (ctx_restore(NULL, kCtxAll)).
/// Emits "Context stack is empty" error if the stack is empty.
void nvim_eval_ctxpop_impl(void)
{
  if (!ctx_restore(NULL, kCtxAll)) {
    emsg(_("Context stack is empty"));
  }
}

/// Returns ctx_size() as an int.
int nvim_eval_ctx_size_impl(void)
{
  return (int)ctx_size();
}

// =============================================================================
// stdpath() helpers
// =============================================================================

/// Get XDG home directory for the given XDG type.
/// Returns allocated string or NULL.
/// @param xdg  XDGVarType cast to int
char *nvim_eval_get_xdg_home(int xdg)
{
  return get_xdg_home((XDGVarType)xdg);
}

/// Get XDG variable path for the given XDG type (raw variable, not home).
/// Returns allocated string or NULL.
/// @param xdg  XDGVarType cast to int
char *nvim_eval_stdpaths_get_xdg_var(int xdg)
{
  return stdpaths_get_xdg_var((XDGVarType)xdg);
}

// =============================================================================
// visualmode() - needs curbuf struct access
// =============================================================================

/// "visualmode([expr])" implementation - last visual mode used.
/// Still in C because it accesses curbuf->b_visual_mode_eval directly.
void nvim_eval_visualmode(typval_T *argvars, typval_T *rettv)
{
  char str[2];
  rettv->v_type = VAR_STRING;
  str[0] = (char)curbuf->b_visual_mode_eval;
  str[1] = NUL;
  rettv->vval.v_string = xstrdup(str);
  if (nvim_eval_non_zero_arg(argvars, 0)) {
    curbuf->b_visual_mode_eval = NUL;
  }
}

// =============================================================================
// getreg() / getregtype() helpers
// =============================================================================

/// Get register name from argvars[0], defaulting to v:register.
/// Returns the register name char, or 0 on error.
int nvim_eval_getreg_get_regname(typval_T *argvars)
{
  const char *strregname;

  if (argvars[0].v_type != VAR_UNKNOWN) {
    strregname = tv_get_string_chk(&argvars[0]);
    if (strregname == NULL) {
      return 0;
    }
  } else {
    strregname = get_vim_var_str(VV_REG);
  }

  return *strregname == 0 ? '"' : (uint8_t)(*strregname);
}

/// Get register type as int (MotionType enum value), with block width.
/// Returns the MotionType integer value.
int nvim_eval_get_reg_type(int regname, int *reg_width)
{
  colnr_T w = 0;
  MotionType t = get_reg_type(regname, &w);
  *reg_width = (int)w;
  return (int)t;
}

/// Format register type to buf (at least NUMBUFLEN+2 = 67 bytes).
void nvim_eval_format_reg_type(int reg_type, int reg_width, char *buf, size_t buf_len)
{
  format_reg_type((MotionType)reg_type, (colnr_T)reg_width, buf, buf_len);
}

/// Get unnamedregister name char.
int nvim_eval_get_unname_register(void)
{
  return get_unname_register();
}

/// Get register name char for register number num.
int nvim_eval_get_register_name(int num)
{
  return rs_get_register_name(num);
}

/// Set rettv to VAR_LIST with list from get_reg_contents.
/// @param regname   register name char
/// @param flags     get_reg_contents flags (kGReg*)
void nvim_eval_getreg_set_list(typval_T *rettv, int regname, int flags)
{
  rettv->v_type = VAR_LIST;
  rettv->vval.v_list = get_reg_contents(regname, flags);
  if (rettv->vval.v_list == NULL) {
    rettv->vval.v_list = tv_list_alloc(0);
  }
  tv_list_ref(rettv->vval.v_list);
}

/// Set rettv to VAR_STRING with string from get_reg_contents.
/// @param regname   register name char
/// @param flags     get_reg_contents flags (kGReg*)
void nvim_eval_getreg_set_str(typval_T *rettv, int regname, int flags)
{
  rettv->v_type = VAR_STRING;
  rettv->vval.v_string = get_reg_contents(regname, flags);
}

// =============================================================================
// has() helpers
// =============================================================================

/// Get v:shell_error value (save/restore pattern for has()).
int64_t nvim_eval_get_shell_error(void)
{
  return (int64_t)get_vim_var_nr(VV_SHELL_ERROR);
}

/// Set v:shell_error value.
void nvim_eval_set_shell_error(int64_t val)
{
  set_vim_var_nr(VV_SHELL_ERROR, (varnumber_T)val);
}

// nvim_eval_has_wsl: defined in funcs.c since has_wsl() is static there

/// Check if syntax is present in curwin.
int nvim_eval_syntax_present(void)
{
  return syntax_present(curwin) ? 1 : 0;
}

/// Check if GUI is attached.
int nvim_eval_ui_gui_attached(void)
{
  return ui_gui_attached() ? 1 : 0;
}

/// Check if starting != 0 (vim is starting up).
int nvim_eval_is_starting(void)
{
  return starting != 0 ? 1 : 0;
}

/// Check for vim patch number: has_vim_patch(vp, 0) or major version: has_vim_patch(vp, major*100+minor).
int nvim_eval_has_vim_patch(int vp, int v)
{
  return has_vim_patch(vp, v) ? 1 : 0;
}

/// Check nvim version string.
int nvim_eval_has_nvim_version(const char *name)
{
  return has_nvim_version(name) ? 1 : 0;
}

/// Check provider availability (throw_if_fast=true, as used by has()).
int nvim_eval_has_provider(const char *name)
{
  return eval_has_provider(name, true) ? 1 : 0;
}
