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
#include "nvim/garray.h"
#include "nvim/highlight_group.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_getln.h"
#include "nvim/getchar.h"
#include "nvim/keycodes.h"
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
#include "nvim/ui_compositor.h"
#include "nvim/version.h"
#include "nvim/grid.h"
#include "nvim/mbyte.h"
#include "nvim/autocmd.h"
#include "nvim/api/private/converter.h"
#include "nvim/cursor.h"
#include "nvim/eval/window.h"
#include "nvim/event/time.h"
#include "nvim/ex_cmds.h"
#include "nvim/mark.h"
#include "nvim/mark_defs.h"
#include "nvim/move.h"
#include "nvim/plines.h"
#include "nvim/spell.h"
#include "nvim/spellsuggest.h"
#include "nvim/window.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/errors.h"
#include "nvim/eval/encode.h"
#include "nvim/eval/executor.h"
#include "nvim/eval/userfunc.h"
#include "nvim/memline.h"
#include "nvim/memory_defs.h"
#include "nvim/normal_defs.h"
#include "nvim/option_vars.h"
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
#include "nvim/register_defs.h"
#include "nvim/strings.h"
#include "nvim/syntax_bridge.h"
#include "nvim/os/dl.h"
#include "nvim/eval/funcs.h"
#include "nvim/lua/executor.h"
#include "nvim/msgpack_rpc/server.h"
#include "nvim/buffer.h"
#include "nvim/edit.h"
#include "nvim/ex_docmd.h"
#include "nvim/charset.h"

// Error strings used by moved functions
static const char e_string_list_or_blob_required[]
  = N_("E1098: String, List or Blob required");
static const char e_missing_function_argument[]
  = N_("E1132: Missing function argument");

// rs_partial_name is implemented in Rust (userfunc crate)
extern char *rs_partial_name(partial_T *pt);

// Rust helper functions used by moved functions
extern bool rs_op_pending(void);
extern int rs_ins_compl_active(void);
extern int rs_get_callback_depth(void);
extern int rs_get_copyID(void);
extern int rs_buf_byteidx_to_charidx(buf_T *buf, linenr_T lnum, int byteidx);
extern int rs_buf_charidx_to_byteidx(buf_T *buf, linenr_T lnum, int charidx);
extern bool rs_callback_from_typval(Callback *callback, const typval_T *arg);
// Rust tab navigation used by tabpagebuflist
extern tabpage_T *rs_find_tabpage(int n);
// Rust tag stack used by gettagstack
extern void rs_get_tagstack(void *wp, void *retdict);
// Rust tag stack set used by settagstack
extern int rs_set_tagstack(void *wp, const void *d, int action);

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

// =============================================================================
// cmdline f_ function helpers - CmdlineInfo/expand_T field access
// =============================================================================

// Rust helpers for cmdline functions
extern int rs_clamp_cmdpos(int pos, int cmdlen);

/// Get cmdline_star (> 0 when in secret/star mode).
int nvim_eval_get_cmdline_star(void)
{
  return (int)nvim_get_cmdline_star();
}

/// Get xpc pointer from CmdlineInfo (or NULL).
/// @param p  CmdlineInfo pointer (from nvim_get_ccline_ptr)
void *nvim_eval_ccline_get_xpc(void *p)
{
  return ((CmdlineInfo *)p)->xpc;
}

/// Get xp_context from expand_T.
int nvim_eval_xpc_get_context(void *xpc)
{
  return ((expand_T *)xpc)->xp_context;
}

/// Set xp_context in expand_T.
void nvim_eval_xpc_set_context(void *xpc, int ctx)
{
  ((expand_T *)xpc)->xp_context = ctx;
}

/// Get xp_pattern from expand_T (may be NULL).
const char *nvim_eval_xpc_get_pattern(void *xpc)
{
  return ((expand_T *)xpc)->xp_pattern;
}

/// Get xp_arg from expand_T (may be NULL).
const char *nvim_eval_xpc_get_arg(void *xpc)
{
  return ((expand_T *)xpc)->xp_arg;
}

/// Call set_expand_context(xpc) on the expand_T pointer.
void nvim_eval_xpc_set_expand_context(void *xpc)
{
  set_expand_context((expand_T *)xpc);
}

/// Get xstrdup of xp_pattern from expand_T.
char *nvim_eval_xpc_dup_pattern(void *xpc)
{
  const char *pat = ((expand_T *)xpc)->xp_pattern;
  return pat != NULL ? xstrdup(pat) : NULL;
}

/// Get cmdcomplete_type_to_str for the given context and xp_arg.
char *nvim_eval_cmdcomplete_type_to_str(int ctx, void *xpc)
{
  return (char *)nvim_cmdexpand_cmdcomplete_type_to_str(ctx, ((expand_T *)xpc)->xp_arg);
}

/// Get cmdlen from CmdlineInfo.
int nvim_eval_ccline_get_cmdlen(void *p)
{
  return ((CmdlineInfo *)p)->cmdlen;
}

/// Get cmdpos from CmdlineInfo.
int nvim_eval_ccline_get_cmdpos(void *p)
{
  return ((CmdlineInfo *)p)->cmdpos;
}

/// Set cmdlen in CmdlineInfo.
void nvim_eval_ccline_set_cmdlen(void *p, int len)
{
  ((CmdlineInfo *)p)->cmdlen = len;
}

/// Set cmdpos in CmdlineInfo.
void nvim_eval_ccline_set_cmdpos(void *p, int pos)
{
  ((CmdlineInfo *)p)->cmdpos = pos;
}

/// Get cmdbuff pointer from CmdlineInfo.
char *nvim_eval_ccline_get_cmdbuff(void *p)
{
  return ((CmdlineInfo *)p)->cmdbuff;
}

/// Call realloc_cmdbuff and then copy str into CmdlineInfo's cmdbuff.
/// Sets cmdlen and does STRCPY. Returns new cmdpos clamped to cmdlen.
int nvim_eval_setcmdline_str(void *p, const char *str, int pos)
{
  CmdlineInfo *cc = (CmdlineInfo *)p;
  int len = (int)strlen(str);
  realloc_cmdbuff(len + 1);
  cc->cmdlen = len;
  STRCPY(cc->cmdbuff, str);
  cc->cmdpos = rs_clamp_cmdpos(pos, cc->cmdlen);
  nvim_set_new_cmdpos(cc->cmdpos);
  redrawcmd();
  nvim_do_autocmd_cmdlinechanged(nvim_get_cmdline_type());
  return cc->cmdpos;
}

/// Check if wildmenu/pum trigger is possible.
/// Returns 1 if wildtrigger is possible, 0 otherwise.
int nvim_eval_wildtrigger_possible(void)
{
  return (!(State & MODE_CMDLINE) || nvim_cmdexpand_char_avail()
          || wild_menu_showing || cmdline_pum_active()) ? 0 : 1;
}

/// Get current cmdline type (wraps nvim_get_cmdline_type).
int nvim_eval_get_cmdline_type(void)
{
  return nvim_get_cmdline_type();
}

/// Insert the K_WILD key sequence into typeahead buffer.
void nvim_eval_ins_k_wild(void)
{
  uint8_t key_string[4];
  key_string[0] = K_SPECIAL;
  key_string[1] = KS_EXTRA;
  key_string[2] = KE_WILD;
  key_string[3] = NUL;
  ins_typebuf((char *)key_string, REMAP_NONE, 0, true, false);
}

// =============================================================================
// synIDattr() modec helper
// =============================================================================

/// Get the 'modec' (mode character) for synIDattr() from argvars[2].
/// Returns 'g' for GUI mode, 'c' for terminal mode, or the user-specified char.
/// @param argvars  pointer to typval_T array (argvars[2] is the optional mode arg)
int nvim_eval_synIDattr_get_modec(typval_T *argvars)
{
  if (argvars[2].v_type != VAR_UNKNOWN) {
    char modebuf[NUMBUFLEN];
    const char *mode = tv_get_string_buf(&argvars[2], modebuf);
    int modec = TOLOWER_ASC(mode[0]);
    if (modec != 'c' && modec != 'g') {
      modec = 0;
    }
    return modec;
  } else if (ui_rgb_attached()) {
    return 'g';
  } else {
    return 'c';
  }
}

// =============================================================================
// blob accessor helpers
// =============================================================================

/// Get blob data pointer from a blob typval (argvars[0]).
/// Returns NULL if v_blob is NULL.
const uint8_t *nvim_eval_tv_blob_get_data(typval_T *tv)
{
  blob_T *b = tv->vval.v_blob;
  return b != NULL ? (const uint8_t *)b->bv_ga.ga_data : NULL;
}

/// Get blob length from a blob typval.
/// Returns 0 if v_blob is NULL.
int nvim_eval_tv_blob_get_len(typval_T *tv)
{
  blob_T *b = tv->vval.v_blob;
  return b != NULL ? b->bv_ga.ga_len : 0;
}

// =============================================================================
// typval list/blob pointer accessors for repeat() inline
// =============================================================================

/// Get the list_T* pointer from a typval (argvars[0] of type VAR_LIST).
list_T *nvim_eval_tv_get_list_ptr(typval_T *tv)
{
  return tv->vval.v_list;
}

/// Set typval to VAR_LIST with an already-allocated list_T*.
/// The list_T* must already be referenced (caller is responsible).
void nvim_eval_tv_set_list(typval_T *tv, list_T *l)
{
  tv->v_type = VAR_LIST;
  tv->vval.v_list = l;
}

/// Get v_number from a typval.
varnumber_T nvim_eval_tv_get_number(typval_T *tv)
{
  return tv->vval.v_number;
}


/// Returns 1 if tv is a function type (VAR_FUNC, VAR_PARTIAL), else 0.
int nvim_eval_tv_is_func(const typval_T *tv)
{
  return tv_is_func(*tv) ? 1 : 0;
}

/// Get tv_get_string_buf_chk result for argvars[idx].
/// Writes to buf (must be NUMBUFLEN = 65 bytes).
/// Returns NULL on type error.
const char *nvim_eval_tv_get_string_buf(const typval_T *tv, char *buf)
{
  return tv_get_string_buf_chk(tv, buf);
}

// =============================================================================
// repeat() blob branch helper
// =============================================================================

// tv_blob_set_range is a Rust-side function (migrated in Phase 1 of blob migration)
extern int tv_blob_set_range(blob_T *dest, varnumber_T n1, varnumber_T n2, typval_T *src);

/// Handle the blob branch of repeat(blob, n).
/// Allocates the result blob in rettv and copies blob n times.
/// @param argvars  typval array; argvars[0] must be VAR_BLOB
/// @param rettv    destination typval
/// @param n        repeat count (already retrieved)
void nvim_eval_repeat_blob(typval_T *argvars, typval_T *rettv, varnumber_T n)
{
  tv_blob_alloc_ret(rettv);
  if (argvars[0].vval.v_blob == NULL || n <= 0) {
    return;
  }

  const int slen = argvars[0].vval.v_blob->bv_ga.ga_len;
  const int len = (int)(slen * n);
  if (len <= 0) {
    return;
  }

  ga_grow(&rettv->vval.v_blob->bv_ga, len);
  rettv->vval.v_blob->bv_ga.ga_len = len;

  int i;
  for (i = 0; i < slen; i++) {
    if (tv_blob_get(argvars[0].vval.v_blob, i) != 0) {
      break;
    }
  }

  if (i == slen) {
    // No need to copy since all bytes are already zero
    return;
  }

  for (i = 0; i < n; i++) {
    tv_blob_set_range(rettv->vval.v_blob, i * slen, (i + 1) * slen - 1, argvars);
  }
}

// =============================================================================
// Screen cell accessor functions (moved from funcs.c)
// =============================================================================

static void screenchar_adjust_inner(ScreenGrid **grid, int *row, int *col)
{
  msg_scroll_flush();
  *grid = ui_comp_get_grid_at_coord(*row, *col);
  *row -= (*grid)->comp_row;
  *col -= (*grid)->comp_col;
}

void nvim_eval_screenattr(typval_T *argvars, typval_T *rettv)
{
  int row = (int)tv_get_number_chk(&argvars[0], NULL) - 1;
  int col = (int)tv_get_number_chk(&argvars[1], NULL) - 1;
  ScreenGrid *grid;
  screenchar_adjust_inner(&grid, &row, &col);
  int c;
  if (row < 0 || row >= grid->rows || col < 0 || col >= grid->cols) {
    c = -1;
  } else {
    c = grid->attrs[grid->line_offset[row] + (size_t)col];
  }
  rettv->vval.v_number = c;
}

void nvim_eval_screenchar(typval_T *argvars, typval_T *rettv)
{
  int row = (int)tv_get_number_chk(&argvars[0], NULL) - 1;
  int col = (int)tv_get_number_chk(&argvars[1], NULL) - 1;
  ScreenGrid *grid;
  screenchar_adjust_inner(&grid, &row, &col);
  rettv->vval.v_number = (row < 0 || row >= grid->rows || col < 0 || col >= grid->cols)
    ? -1 : schar_get_first_codepoint(grid_getchar(grid, row, col, NULL));
}

void nvim_eval_screenchars(typval_T *argvars, typval_T *rettv)
{
  int row = (int)tv_get_number_chk(&argvars[0], NULL) - 1;
  int col = (int)tv_get_number_chk(&argvars[1], NULL) - 1;
  ScreenGrid *grid;
  screenchar_adjust_inner(&grid, &row, &col);
  tv_list_alloc_ret(rettv, kListLenMayKnow);
  if (row < 0 || row >= grid->rows || col < 0 || col >= grid->cols) {
    return;
  }
  char buf[MAX_SCHAR_SIZE + 1];
  schar_get(buf, grid_getchar(grid, row, col, NULL));
  size_t i = 0;
  do {
    int c = utf_ptr2char(buf + i);
    tv_list_append_number(rettv->vval.v_list, c);
    i += (size_t)utf_ptr2len(buf + i);
  } while (buf[i] != NUL);
}

void nvim_eval_screenstring(typval_T *argvars, typval_T *rettv)
{
  rettv->vval.v_string = NULL;
  rettv->v_type = VAR_STRING;
  int row = (int)tv_get_number_chk(&argvars[0], NULL) - 1;
  int col = (int)tv_get_number_chk(&argvars[1], NULL) - 1;
  ScreenGrid *grid;
  screenchar_adjust_inner(&grid, &row, &col);
  if (row < 0 || row >= grid->rows || col < 0 || col >= grid->cols) {
    return;
  }
  char buf[MAX_SCHAR_SIZE + 1];
  schar_get(buf, grid_getchar(grid, row, col, NULL));
  rettv->vval.v_string = xstrdup(buf);
}

// =============================================================================
// find_some_match: moved from funcs.c
// =============================================================================

/// Describe data to return from nvim_eval_find_some_match()
typedef enum {
  kSomeMatch,       ///< Data for match().
  kSomeMatchEnd,    ///< Data for matchend().
  kSomeMatchList,   ///< Data for matchlist().
  kSomeMatchStr,    ///< Data for matchstr().
  kSomeMatchStrPos, ///< Data for matchstrpos().
} SomeMatchType;

void nvim_eval_find_some_match(typval_T *argvars, typval_T *rettv, int kind)
{
  const SomeMatchType type = (SomeMatchType)kind;

  char *str = NULL;
  int64_t len = 0;
  char *expr = NULL;
  regmatch_T regmatch;
  int64_t start = 0;
  int64_t nth = 1;
  colnr_T startcol = 0;
  bool match = false;
  list_T *l = NULL;
  int idx = 0;
  char *tofree = NULL;

  // Make 'cpoptions' empty, the 'l' flag should not be used here.
  char *save_cpo = p_cpo;
  p_cpo = empty_string_option;

  rettv->vval.v_number = -1;
  switch (type) {
  // matchlist(): return empty list when there are no matches.
  case kSomeMatchList:
    tv_list_alloc_ret(rettv, kListLenMayKnow);
    break;
  // matchstrpos(): return ["", -1, -1, -1]
  case kSomeMatchStrPos:
    tv_list_alloc_ret(rettv, 4);
    tv_list_append_string(rettv->vval.v_list, "", 0);
    tv_list_append_number(rettv->vval.v_list, -1);
    tv_list_append_number(rettv->vval.v_list, -1);
    tv_list_append_number(rettv->vval.v_list, -1);
    break;
  case kSomeMatchStr:
    rettv->v_type = VAR_STRING;
    rettv->vval.v_string = NULL;
    break;
  case kSomeMatch:
  case kSomeMatchEnd:
    // Do nothing: zero is default.
    break;
  }

  listitem_T *li = NULL;
  if (argvars[0].v_type == VAR_LIST) {
    if ((l = argvars[0].vval.v_list) == NULL) {
      goto theend;
    }
    li = tv_list_first(l);
  } else {
    expr = str = (char *)tv_get_string(&argvars[0]);
    len = (int64_t)strlen(str);
  }

  char patbuf[NUMBUFLEN];
  const char *const pat = tv_get_string_buf_chk(&argvars[1], patbuf);
  if (pat == NULL) {
    goto theend;
  }

  if (argvars[2].v_type != VAR_UNKNOWN) {
    bool error = false;

    start = tv_get_number_chk(&argvars[2], &error);
    if (error) {
      goto theend;
    }
    if (l != NULL) {
      idx = tv_list_uidx(l, (int)start);
      if (idx == -1) {
        goto theend;
      }
      li = tv_list_find(l, idx);
    } else {
      if (start < 0) {
        start = 0;
      }
      if (start > len) {
        goto theend;
      }
      // When "count" argument is there ignore matches before "start",
      // otherwise skip part of the string.  Differs when pattern is "^"
      // or "\<".
      if (argvars[3].v_type != VAR_UNKNOWN) {
        startcol = (colnr_T)start;
      } else {
        str += start;
        len -= start;
      }
    }

    if (argvars[3].v_type != VAR_UNKNOWN) {
      nth = tv_get_number_chk(&argvars[3], &error);
    }
    if (error) {
      goto theend;
    }
  }

  regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
  if (regmatch.regprog != NULL) {
    regmatch.rm_ic = p_ic;

    while (true) {
      if (l != NULL) {
        if (li == NULL) {
          match = false;
          break;
        }
        xfree(tofree);
        tofree = expr = str = encode_tv2echo(TV_LIST_ITEM_TV(li), NULL);
        if (str == NULL) {
          break;
        }
      }

      match = vim_regexec_nl(&regmatch, str, startcol);

      if (match && --nth <= 0) {
        break;
      }
      if (l == NULL && !match) {
        break;
      }

      // Advance to just after the match.
      if (l != NULL) {
        li = TV_LIST_ITEM_NEXT(l, li);
        idx++;
      } else {
        startcol = (colnr_T)(regmatch.startp[0]
                             + utfc_ptr2len(regmatch.startp[0]) - str);
        if (startcol > (colnr_T)len || str + startcol <= regmatch.startp[0]) {
          match = false;
          break;
        }
      }
    }

    if (match) {
      switch (type) {
      case kSomeMatchStrPos: {
        list_T *const ret_l = rettv->vval.v_list;
        listitem_T *li1 = tv_list_first(ret_l);
        listitem_T *li2 = TV_LIST_ITEM_NEXT(ret_l, li1);
        listitem_T *li3 = TV_LIST_ITEM_NEXT(ret_l, li2);
        listitem_T *li4 = TV_LIST_ITEM_NEXT(ret_l, li3);
        xfree(TV_LIST_ITEM_TV(li1)->vval.v_string);

        const size_t rd = (size_t)(regmatch.endp[0] - regmatch.startp[0]);
        TV_LIST_ITEM_TV(li1)->vval.v_string = xmemdupz(regmatch.startp[0], rd);
        TV_LIST_ITEM_TV(li3)->vval.v_number = (varnumber_T)(regmatch.startp[0] - expr);
        TV_LIST_ITEM_TV(li4)->vval.v_number = (varnumber_T)(regmatch.endp[0] - expr);
        if (l != NULL) {
          TV_LIST_ITEM_TV(li2)->vval.v_number = (varnumber_T)idx;
        }
        break;
      }
      case kSomeMatchList:
        // Return list with matched string and submatches.
        for (int i = 0; i < NSUBEXP; i++) {
          if (regmatch.endp[i] == NULL) {
            tv_list_append_string(rettv->vval.v_list, NULL, 0);
          } else {
            tv_list_append_string(rettv->vval.v_list, regmatch.startp[i],
                                  (regmatch.endp[i] - regmatch.startp[i]));
          }
        }
        break;
      case kSomeMatchStr:
        // Return matched string.
        if (l != NULL) {
          tv_copy(TV_LIST_ITEM_TV(li), rettv);
        } else {
          rettv->vval.v_string = xmemdupz(regmatch.startp[0],
                                          (size_t)(regmatch.endp[0] -
                                                   regmatch.startp[0]));
        }
        break;
      case kSomeMatch:
      case kSomeMatchEnd:
        if (l != NULL) {
          rettv->vval.v_number = idx;
        } else {
          if (type == kSomeMatch) {
            rettv->vval.v_number = (varnumber_T)(regmatch.startp[0] - str);
          } else {
            rettv->vval.v_number = (varnumber_T)(regmatch.endp[0] - str);
          }
          rettv->vval.v_number += (varnumber_T)(str - expr);
        }
        break;
      }
    }
    vim_regfree(regmatch.regprog);
  }

theend:
  if (type == kSomeMatchStrPos && l == NULL && rettv->vval.v_list != NULL) {
    // matchstrpos() without a list: drop the second item
    list_T *const ret_l = rettv->vval.v_list;
    tv_list_item_remove(ret_l, TV_LIST_ITEM_NEXT(ret_l, tv_list_first(ret_l)));
  }

  xfree(tofree);
  p_cpo = save_cpo;
}

// =============================================================================
// common_function: moved from funcs.c
// =============================================================================

void nvim_eval_common_function(typval_T *argvars, typval_T *rettv, bool is_funcref)
{
  char *s;
  char *name;
  bool use_string = false;
  partial_T *arg_pt = NULL;
  char *trans_name = NULL;

  if (argvars[0].v_type == VAR_FUNC) {
    // function(MyFunc, [arg], dict)
    s = argvars[0].vval.v_string;
  } else if (argvars[0].v_type == VAR_PARTIAL
             && argvars[0].vval.v_partial != NULL) {
    // function(dict.MyFunc, [arg])
    arg_pt = argvars[0].vval.v_partial;
    s = rs_partial_name(arg_pt);
    // TODO(bfredl): do the entire nlua_is_table_from_lua dance
  } else {
    // function('MyFunc', [arg], dict)
    s = (char *)tv_get_string(&argvars[0]);
    use_string = true;
  }

  if ((use_string && vim_strchr(s, AUTOLOAD_CHAR) == NULL) || is_funcref) {
    name = s;
    trans_name = save_function_name(&name, false,
                                    TFN_INT | TFN_QUIET | TFN_NO_AUTOLOAD | TFN_NO_DEREF, NULL);
    if (*name != NUL) {
      s = NULL;
    }
  }
  if (s == NULL || *s == NUL || (use_string && ascii_isdigit(*s))
      || (is_funcref && trans_name == NULL)) {
    semsg(_(e_invarg2), (use_string ? tv_get_string(&argvars[0]) : s));
    // Don't check an autoload name for existence here.
  } else if (trans_name != NULL
             && (is_funcref
                 ? find_func(trans_name) == NULL
                 : !translated_function_exists(trans_name))) {
    semsg(_("E700: Unknown function: %s"), s);
  } else {
    int dict_idx = 0;
    int arg_idx = 0;
    list_T *list = NULL;
    if (strncmp(s, "s:", 2) == 0 || strncmp(s, "<SID>", 5) == 0) {
      // Expand s: and <SID> into <SNR>nr_, so that the function can
      // also be called from another script. Using trans_function_name()
      // would also work, but some plugins depend on the name being
      // printable text.
      name = get_scriptlocal_funcname(s);
    } else {
      name = xstrdup(s);
    }

    if (argvars[1].v_type != VAR_UNKNOWN) {
      if (argvars[2].v_type != VAR_UNKNOWN) {
        // function(name, [args], dict)
        arg_idx = 1;
        dict_idx = 2;
      } else if (argvars[1].v_type == VAR_DICT) {
        // function(name, dict)
        dict_idx = 1;
      } else {
        // function(name, [args])
        arg_idx = 1;
      }
      if (dict_idx > 0) {
        if (tv_check_for_dict_arg(argvars, dict_idx) == FAIL) {
          xfree(name);
          goto theend;
        }
        if (argvars[dict_idx].vval.v_dict == NULL) {
          dict_idx = 0;
        }
      }
      if (arg_idx > 0) {
        if (argvars[arg_idx].v_type != VAR_LIST) {
          emsg(_("E923: Second argument of function() must be "
                 "a list or a dict"));
          xfree(name);
          goto theend;
        }
        list = argvars[arg_idx].vval.v_list;
        if (tv_list_len(list) == 0) {
          arg_idx = 0;
        } else if (tv_list_len(list) > MAX_FUNC_ARGS) {
          emsg_funcname(e_toomanyarg, s);
          xfree(name);
          goto theend;
        }
      }
    }
    if (dict_idx > 0 || arg_idx > 0 || arg_pt != NULL || is_funcref) {
      partial_T *const pt = xcalloc(1, sizeof(*pt));

      // result is a VAR_PARTIAL
      if (arg_idx > 0 || (arg_pt != NULL && arg_pt->pt_argc > 0)) {
        const int arg_len = (arg_pt == NULL ? 0 : arg_pt->pt_argc);
        const int lv_len = tv_list_len(list);

        pt->pt_argc = arg_len + lv_len;
        pt->pt_argv = xmalloc(sizeof(pt->pt_argv[0]) * (size_t)pt->pt_argc);
        int i = 0;
        for (; i < arg_len; i++) {
          tv_copy(&arg_pt->pt_argv[i], &pt->pt_argv[i]);
        }
        if (lv_len > 0) {
          TV_LIST_ITER(list, li, {
            tv_copy(TV_LIST_ITEM_TV(li), &pt->pt_argv[i++]);
          });
        }
      }

      // For "function(dict.func, [], dict)" and "func" is a partial
      // use "dict". That is backwards compatible.
      if (dict_idx > 0) {
        // The dict is bound explicitly, pt_auto is false
        pt->pt_dict = argvars[dict_idx].vval.v_dict;
        (pt->pt_dict->dv_refcount)++;
      } else if (arg_pt != NULL) {
        // If the dict was bound automatically the result is also
        // bound automatically.
        pt->pt_dict = arg_pt->pt_dict;
        pt->pt_auto = arg_pt->pt_auto;
        if (pt->pt_dict != NULL) {
          (pt->pt_dict->dv_refcount)++;
        }
      }

      pt->pt_refcount = 1;
      if (arg_pt != NULL && arg_pt->pt_func != NULL) {
        pt->pt_func = arg_pt->pt_func;
        func_ptr_ref(pt->pt_func);
        xfree(name);
      } else if (is_funcref) {
        pt->pt_func = find_func(trans_name);
        func_ptr_ref(pt->pt_func);
        xfree(name);
      } else {
        pt->pt_name = name;
        func_ref(name);
      }

      rettv->v_type = VAR_PARTIAL;
      rettv->vval.v_partial = pt;
    } else {
      // result is a VAR_FUNC
      rettv->v_type = VAR_FUNC;
      rettv->vval.v_string = name;
      func_ref(name);
    }
  }
theend:
  xfree(trans_name);
}

// =============================================================================
// index / indexof: moved from funcs.c
// =============================================================================

void nvim_eval_index(typval_T *argvars, typval_T *rettv)
{
  int idx = 0;
  bool ic = false;

  rettv->vval.v_number = -1;
  if (argvars[0].v_type == VAR_BLOB) {
    bool error = false;
    int start = 0;

    if (argvars[2].v_type != VAR_UNKNOWN) {
      start = (int)tv_get_number_chk(&argvars[2], &error);
      if (error) {
        return;
      }
    }
    blob_T *const b = argvars[0].vval.v_blob;
    if (b == NULL) {
      return;
    }
    if (start < 0) {
      start = tv_blob_len(b) + start;
      if (start < 0) {
        start = 0;
      }
    }
    for (idx = start; idx < tv_blob_len(b); idx++) {
      typval_T tv;
      tv.v_type = VAR_NUMBER;
      tv.vval.v_number = tv_blob_get(b, idx);
      if (tv_equal(&tv, &argvars[1], ic)) {
        rettv->vval.v_number = idx;
        return;
      }
    }
    return;
  } else if (argvars[0].v_type != VAR_LIST) {
    emsg(_(e_listblobreq));
    return;
  }

  list_T *const l = argvars[0].vval.v_list;
  if (l == NULL) {
    return;
  }

  listitem_T *item = tv_list_first(l);
  if (argvars[2].v_type != VAR_UNKNOWN) {
    bool error = false;

    // Start at specified item.
    idx = tv_list_uidx(l, (int)tv_get_number_chk(&argvars[2], &error));
    if (error || idx == -1) {
      item = NULL;
    } else {
      item = tv_list_find(l, idx);
      assert(item != NULL);
    }
    if (argvars[3].v_type != VAR_UNKNOWN) {
      ic = !!tv_get_number_chk(&argvars[3], &error);
      if (error) {
        item = NULL;
      }
    }
  }

  for (; item != NULL; item = TV_LIST_ITEM_NEXT(l, item), idx++) {
    if (tv_equal(TV_LIST_ITEM_TV(item), &argvars[1], ic)) {
      rettv->vval.v_number = idx;
      break;
    }
  }
}

static varnumber_T nvim_eval_indexof_eval_expr(typval_T *expr)
{
  typval_T argv[3];
  argv[0] = *get_vim_var_tv(VV_KEY);
  argv[1] = *get_vim_var_tv(VV_VAL);
  typval_T newtv;
  newtv.v_type = VAR_UNKNOWN;

  if (eval_expr_typval(expr, false, argv, 2, &newtv) == FAIL) {
    return false;
  }

  bool error = false;
  varnumber_T found = tv_get_bool_chk(&newtv, &error);
  tv_clear(&newtv);

  return error ? false : found;
}

static varnumber_T nvim_eval_indexof_blob(blob_T *b, varnumber_T startidx, typval_T *expr)
{
  if (b == NULL) {
    return -1;
  }

  if (startidx < 0) {
    startidx = tv_blob_len(b) + startidx;
    if (startidx < 0) {
      startidx = 0;
    }
  }

  set_vim_var_type(VV_KEY, VAR_NUMBER);
  set_vim_var_type(VV_VAL, VAR_NUMBER);

  const int called_emsg_start = called_emsg;
  for (varnumber_T idx = startidx; idx < tv_blob_len(b); idx++) {
    set_vim_var_nr(VV_KEY, idx);
    set_vim_var_nr(VV_VAL, tv_blob_get(b, (int)idx));

    if (nvim_eval_indexof_eval_expr(expr)) {
      return idx;
    }

    if (called_emsg != called_emsg_start) {
      return -1;
    }
  }

  return -1;
}

static varnumber_T nvim_eval_indexof_list(list_T *l, varnumber_T startidx, typval_T *expr)
{
  if (l == NULL) {
    return -1;
  }

  listitem_T *item;
  varnumber_T idx = 0;
  if (startidx == 0) {
    item = tv_list_first(l);
  } else {
    idx = tv_list_uidx(l, (int)startidx);
    if (idx == -1) {
      item = NULL;
    } else {
      item = tv_list_find(l, (int)idx);
      assert(item != NULL);
    }
  }

  set_vim_var_type(VV_KEY, VAR_NUMBER);

  const int called_emsg_start = called_emsg;
  for (; item != NULL; item = TV_LIST_ITEM_NEXT(l, item), idx++) {
    set_vim_var_nr(VV_KEY, idx);
    tv_copy(TV_LIST_ITEM_TV(item), get_vim_var_tv(VV_VAL));

    bool found = nvim_eval_indexof_eval_expr(expr);
    tv_clear(get_vim_var_tv(VV_VAL));

    if (found) {
      return idx;
    }

    if (called_emsg != called_emsg_start) {
      return -1;
    }
  }

  return -1;
}

void nvim_eval_indexof(typval_T *argvars, typval_T *rettv)
{
  rettv->vval.v_number = -1;

  if (tv_check_for_list_or_blob_arg(argvars, 0) == FAIL
      || tv_check_for_string_or_func_arg(argvars, 1) == FAIL
      || tv_check_for_opt_dict_arg(argvars, 2) == FAIL) {
    return;
  }

  if ((argvars[1].v_type == VAR_STRING
       && (argvars[1].vval.v_string == NULL || *argvars[1].vval.v_string == NUL))
      || (argvars[1].v_type == VAR_FUNC && argvars[1].vval.v_partial == NULL)) {
    return;
  }

  varnumber_T startidx = 0;
  if (argvars[2].v_type == VAR_DICT) {
    startidx = tv_dict_get_number_def(argvars[2].vval.v_dict, "startidx", 0);
  }

  typval_T save_val;
  typval_T save_key;
  prepare_vimvar(VV_VAL, &save_val);
  prepare_vimvar(VV_KEY, &save_key);

  const int save_did_emsg = did_emsg;
  did_emsg = false;

  if (argvars[0].v_type == VAR_BLOB) {
    rettv->vval.v_number = nvim_eval_indexof_blob(argvars[0].vval.v_blob, startidx, &argvars[1]);
  } else {
    rettv->vval.v_number = nvim_eval_indexof_list(argvars[0].vval.v_list, startidx, &argvars[1]);
  }

  restore_vimvar(VV_KEY, &save_key);
  restore_vimvar(VV_VAL, &save_val);
  did_emsg |= save_did_emsg;
}

// =============================================================================
// reduce: moved from funcs.c
// =============================================================================

static void nvim_eval_reduce_list(typval_T *argvars, typval_T *expr, typval_T *rettv)
{
  list_T *const l = argvars[0].vval.v_list;
  const int called_emsg_start = called_emsg;

  typval_T initial;
  const listitem_T *li = NULL;
  if (argvars[2].v_type == VAR_UNKNOWN) {
    if (tv_list_len(l) == 0) {
      semsg(_(e_reduce_of_an_empty_str_with_no_initial_value), "List");
      return;
    }
    const listitem_T *const first = tv_list_first(l);
    initial = *TV_LIST_ITEM_TV(first);
    li = TV_LIST_ITEM_NEXT(l, first);
  } else {
    initial = argvars[2];
    li = tv_list_first(l);
  }

  tv_copy(&initial, rettv);

  if (l == NULL) {
    return;
  }

  const VarLockStatus prev_locked = tv_list_locked(l);

  tv_list_set_lock(l, VAR_FIXED);  // disallow the list changing here
  for (; li != NULL; li = TV_LIST_ITEM_NEXT(l, li)) {
    typval_T argv[3];
    argv[0] = *rettv;
    argv[1] = *TV_LIST_ITEM_TV(li);
    rettv->v_type = VAR_UNKNOWN;

    const int r = eval_expr_typval(expr, true, argv, 2, rettv);

    tv_clear(&argv[0]);
    if (r == FAIL || called_emsg != called_emsg_start) {
      break;
    }
  }
  tv_list_set_lock(l, prev_locked);
}

static void nvim_eval_reduce_string(typval_T *argvars, typval_T *expr, typval_T *rettv)
{
  const char *p = tv_get_string(&argvars[0]);
  int len;
  const int called_emsg_start = called_emsg;

  if (argvars[2].v_type == VAR_UNKNOWN) {
    if (*p == NUL) {
      semsg(_(e_reduce_of_an_empty_str_with_no_initial_value), "String");
      return;
    }
    len = utfc_ptr2len(p);
    *rettv = (typval_T){
      .v_type = VAR_STRING,
      .v_lock = VAR_UNLOCKED,
      .vval.v_string = xmemdupz(p, (size_t)len),
    };
    p += len;
  } else if (tv_check_for_string_arg(argvars, 2) == FAIL) {
    return;
  } else {
    tv_copy(&argvars[2], rettv);
  }

  for (; *p != NUL; p += len) {
    typval_T argv[3];
    argv[0] = *rettv;
    len = utfc_ptr2len(p);
    argv[1] = (typval_T){
      .v_type = VAR_STRING,
      .v_lock = VAR_UNLOCKED,
      .vval.v_string = xmemdupz(p, (size_t)len),
    };

    const int r = eval_expr_typval(expr, true, argv, 2, rettv);

    tv_clear(&argv[0]);
    tv_clear(&argv[1]);
    if (r == FAIL || called_emsg != called_emsg_start) {
      break;
    }
  }
}

static void nvim_eval_reduce_blob(typval_T *argvars, typval_T *expr, typval_T *rettv)
{
  const blob_T *const b = argvars[0].vval.v_blob;
  const int called_emsg_start = called_emsg;

  typval_T initial;
  int i;
  if (argvars[2].v_type == VAR_UNKNOWN) {
    if (tv_blob_len(b) == 0) {
      semsg(_(e_reduce_of_an_empty_str_with_no_initial_value), "Blob");
      return;
    }
    initial = (typval_T){
      .v_type = VAR_NUMBER,
      .v_lock = VAR_UNLOCKED,
      .vval.v_number = tv_blob_get(b, 0),
    };
    i = 1;
  } else if (tv_check_for_number_arg(argvars, 2) == FAIL) {
    return;
  } else {
    initial = argvars[2];
    i = 0;
  }

  tv_copy(&initial, rettv);
  for (; i < tv_blob_len(b); i++) {
    typval_T argv[3];
    argv[0] = *rettv;
    argv[1] = (typval_T){
      .v_type = VAR_NUMBER,
      .v_lock = VAR_UNLOCKED,
      .vval.v_number = tv_blob_get(b, i),
    };

    const int r = eval_expr_typval(expr, true, argv, 2, rettv);

    if (r == FAIL || called_emsg != called_emsg_start) {
      return;
    }
  }
}

void nvim_eval_reduce(typval_T *argvars, typval_T *rettv)
{
  if (argvars[0].v_type != VAR_STRING
      && argvars[0].v_type != VAR_LIST
      && argvars[0].v_type != VAR_BLOB) {
    emsg(_(e_string_list_or_blob_required));
    return;
  }

  const char *func_name;
  if (argvars[1].v_type == VAR_FUNC) {
    func_name = argvars[1].vval.v_string;
  } else if (argvars[1].v_type == VAR_PARTIAL) {
    func_name = rs_partial_name(argvars[1].vval.v_partial);
  } else {
    func_name = tv_get_string(&argvars[1]);
  }
  if (func_name == NULL || *func_name == NUL) {
    emsg(_(e_missing_function_argument));
    return;
  }

  if (argvars[0].v_type == VAR_LIST) {
    nvim_eval_reduce_list(argvars, &argvars[1], rettv);
  } else if (argvars[0].v_type == VAR_STRING) {
    nvim_eval_reduce_string(argvars, &argvars[1], rettv);
  } else {
    nvim_eval_reduce_blob(argvars, &argvars[1], rettv);
  }
}

// =============================================================================
// max/min: moved from funcs.c
// =============================================================================

void nvim_eval_max_min(typval_T *argvars, typval_T *rettv, bool domax)
{
  bool error = false;

  rettv->vval.v_number = 0;
  varnumber_T n = (domax ? VARNUMBER_MIN : VARNUMBER_MAX);
  if (argvars->v_type == VAR_LIST) {
    if (tv_list_len(argvars->vval.v_list) == 0) {
      return;
    }
    TV_LIST_ITER_CONST(argvars->vval.v_list, li, {
      const varnumber_T i = tv_get_number_chk(TV_LIST_ITEM_TV(li), &error);
      if (error) {
        return;  // type error; errmsg already given
      }
      if (domax ? i > n : i < n) {
        n = i;
      }
    });
  } else if (argvars->v_type == VAR_DICT) {
    if (tv_dict_len(argvars->vval.v_dict) == 0) {
      return;
    }
    TV_DICT_ITER(argvars->vval.v_dict, di, {
      const varnumber_T i = tv_get_number_chk(&di->di_tv, &error);
      if (error) {
        return;  // type error; errmsg already given
      }
      if (domax ? i > n : i < n) {
        n = i;
      }
    });
  } else {
    semsg(_(e_listdictarg), domax ? "max()" : "min()");
    return;
  }

  rettv->vval.v_number = n;
}

// =============================================================================
// ctx: moved from funcs.c
// =============================================================================

void nvim_eval_ctxget(typval_T *argvars, typval_T *rettv)
{
  size_t index = 0;
  if (argvars[0].v_type == VAR_NUMBER) {
    index = (size_t)argvars[0].vval.v_number;
  } else if (argvars[0].v_type != VAR_UNKNOWN) {
    semsg(_(e_invarg2), "expected nothing or a Number as an argument");
    return;
  }

  Context *ctx = ctx_get(index);
  if (ctx == NULL) {
    semsg(_(e_invargNval), "index", "out of bounds");
    return;
  }

  Arena arena = ARENA_EMPTY;
  Dict ctx_dict = ctx_to_dict(ctx, &arena);
  Error err = ERROR_INIT;
  object_to_vim(DICT_OBJ(ctx_dict), rettv, &err);
  arena_mem_free(arena_finish(&arena));
  api_clear_error(&err);
}

void nvim_eval_ctxpush(typval_T *argvars, typval_T *rettv)
{
  int types = kCtxAll;
  if (argvars[0].v_type == VAR_LIST) {
    types = 0;
    TV_LIST_ITER(argvars[0].vval.v_list, li, {
      typval_T *tv_li = TV_LIST_ITEM_TV(li);
      if (tv_li->v_type == VAR_STRING) {
        if (strequal(tv_li->vval.v_string, "regs")) {
          types |= kCtxRegs;
        } else if (strequal(tv_li->vval.v_string, "jumps")) {
          types |= kCtxJumps;
        } else if (strequal(tv_li->vval.v_string, "bufs")) {
          types |= kCtxBufs;
        } else if (strequal(tv_li->vval.v_string, "gvars")) {
          types |= kCtxGVars;
        } else if (strequal(tv_li->vval.v_string, "sfuncs")) {
          types |= kCtxSFuncs;
        } else if (strequal(tv_li->vval.v_string, "funcs")) {
          types |= kCtxFuncs;
        }
      }
    });
  } else if (argvars[0].v_type != VAR_UNKNOWN) {
    semsg(_(e_invarg2), "expected nothing or a List as an argument");
    return;
  }
  ctx_save(NULL, types);
}

void nvim_eval_ctxset(typval_T *argvars, typval_T *rettv)
{
  if (argvars[0].v_type != VAR_DICT) {
    semsg(_(e_invarg2), "expected dictionary as first argument");
    return;
  }

  size_t index = 0;
  if (argvars[1].v_type == VAR_NUMBER) {
    index = (size_t)argvars[1].vval.v_number;
  } else if (argvars[1].v_type != VAR_UNKNOWN) {
    semsg(_(e_invarg2), "expected nothing or a Number as second argument");
    return;
  }

  Context *ctx = ctx_get(index);
  if (ctx == NULL) {
    semsg(_(e_invargNval), "index", "out of bounds");
    return;
  }

  const int save_did_emsg = did_emsg;
  did_emsg = false;

  Arena arena = ARENA_EMPTY;
  Dict dict = vim_to_object(&argvars[0], &arena, true).data.dict;
  Context tmp = CONTEXT_INIT;
  Error err = ERROR_INIT;
  ctx_from_dict(dict, &tmp, &err);

  if (ERROR_SET(&err)) {
    semsg("%s", err.msg);
    ctx_free(&tmp);
  } else {
    ctx_free(ctx);
    *ctx = tmp;
  }

  arena_mem_free(arena_finish(&arena));
  api_clear_error(&err);
  did_emsg = save_did_emsg;
}

// =============================================================================
// state: moved from funcs.c
// =============================================================================

static void nvim_eval_may_add_state_char(garray_T *gap, const char *include, uint8_t c)
{
  if (include == NULL || vim_strchr(include, c) != NULL) {
    ga_append(gap, c);
  }
}

void nvim_eval_state(typval_T *argvars, typval_T *rettv)
{
  garray_T ga;
  ga_init(&ga, 1, 20);
  const char *include = NULL;

  if (argvars[0].v_type != VAR_UNKNOWN) {
    include = tv_get_string(&argvars[0]);
  }

  if (!(stuff_empty() && typebuf.tb_len == 0 && !using_script())) {
    nvim_eval_may_add_state_char(&ga, include, 'm');
  }
  if (rs_op_pending()) {
    nvim_eval_may_add_state_char(&ga, include, 'o');
  }
  if (autocmd_busy) {
    nvim_eval_may_add_state_char(&ga, include, 'x');
  }
  if (rs_ins_compl_active()) {
    nvim_eval_may_add_state_char(&ga, include, 'a');
  }
  if (!get_was_safe_state()) {
    nvim_eval_may_add_state_char(&ga, include, 'S');
  }
  for (int i = 0; i < rs_get_callback_depth() && i < 3; i++) {
    nvim_eval_may_add_state_char(&ga, include, 'c');
  }
  if (msg_scrolled > 0) {
    nvim_eval_may_add_state_char(&ga, include, 's');
  }

  rettv->v_type = VAR_STRING;
  rettv->vval.v_string = ga.ga_data;
}

// =============================================================================
// getreginfo: moved from funcs.c
// =============================================================================

void nvim_eval_getreginfo(typval_T *argvars, typval_T *rettv)
{
  int regname = nvim_eval_getreg_get_regname(argvars);
  if (regname == 0) {
    return;
  }

  if (regname == '@') {
    regname = '"';
  }

  tv_dict_alloc_ret(rettv);
  dict_T *const dict = rettv->vval.v_dict;

  list_T *const list = get_reg_contents(regname, kGRegExprSrc | kGRegList);
  if (list == NULL) {
    return;
  }
  tv_dict_add_list(dict, S_LEN("regcontents"), list);

  char buf[NUMBUFLEN + 2];
  buf[0] = NUL;
  buf[1] = NUL;
  colnr_T reglen = 0;
  switch (get_reg_type(regname, &reglen)) {
  case kMTLineWise:
    buf[0] = 'V';
    break;
  case kMTCharWise:
    buf[0] = 'v';
    break;
  case kMTBlockWise:
    vim_snprintf(buf, sizeof(buf), "%c%d", Ctrl_V, reglen + 1);
    break;
  case kMTUnknown:
    abort();
  }
  tv_dict_add_str(dict, S_LEN("regtype"), buf);

  buf[0] = (char)get_register_name(get_unname_register());
  buf[1] = NUL;
  if (regname == '"') {
    tv_dict_add_str(dict, S_LEN("points_to"), buf);
  } else {
    tv_dict_add_bool(dict, S_LEN("isunnamed"),
                     regname == buf[0] ? kBoolVarTrue : kBoolVarFalse);
  }
}

// =============================================================================
// synID, synconcealed, synstack: moved from funcs.c
// =============================================================================

void nvim_eval_synID(typval_T *argvars, typval_T *rettv)
{
  // -1 on type error (both)
  const linenr_T lnum = tv_get_lnum(argvars);
  const colnr_T col = (colnr_T)tv_get_number(&argvars[1]) - 1;

  bool transerr = false;
  const int trans = (int)tv_get_number_chk(&argvars[2], &transerr);

  int id = 0;
  if (!transerr && lnum >= 1 && lnum <= curbuf->b_ml.ml_line_count
      && col >= 0 && col < ml_get_len(lnum)) {
    id = syn_get_id(curwin, lnum, col, trans, NULL, false);
  }

  rettv->vval.v_number = id;
}

void nvim_eval_synconcealed(typval_T *argvars, typval_T *rettv)
{
  int syntax_flags = 0;
  int matchid = 0;
  char str[NUMBUFLEN];

  tv_list_set_ret(rettv, NULL);

  // -1 on type error (both)
  const linenr_T lnum = tv_get_lnum(argvars);
  const colnr_T col = (colnr_T)tv_get_number(&argvars[1]) - 1;

  CLEAR_FIELD(str);

  if (lnum >= 1 && lnum <= curbuf->b_ml.ml_line_count
      && col >= 0 && col <= ml_get_len(lnum) && curwin->w_p_cole > 0) {
    syn_get_id(curwin, lnum, col, false, NULL, false);
    syntax_flags = get_syntax_info(&matchid);

    // get the conceal character
    if ((syntax_flags & HL_CONCEAL) && curwin->w_p_cole < 3) {
      schar_T cchar = schar_from_char(syn_get_sub_char());
      if (cchar == NUL && curwin->w_p_cole == 1) {
        cchar = (curwin->w_p_lcs_chars.conceal == NUL)
                ? schar_from_ascii(' ') : curwin->w_p_lcs_chars.conceal;
      }
      if (cchar != NUL) {
        schar_get(str, cchar);
      }
    }
  }

  tv_list_alloc_ret(rettv, 3);
  tv_list_append_number(rettv->vval.v_list, (syntax_flags & HL_CONCEAL) != 0);
  // -1 to auto-determine strlen
  tv_list_append_string(rettv->vval.v_list, str, -1);
  tv_list_append_number(rettv->vval.v_list, matchid);
}

void nvim_eval_synstack(typval_T *argvars, typval_T *rettv)
{
  tv_list_set_ret(rettv, NULL);

  // -1 on type error (both)
  const linenr_T lnum = tv_get_lnum(argvars);
  const colnr_T col = (colnr_T)tv_get_number(&argvars[1]) - 1;

  if (lnum >= 1 && lnum <= curbuf->b_ml.ml_line_count
      && col >= 0 && col <= ml_get_len(lnum)) {
    tv_list_alloc_ret(rettv, kListLenMayKnow);
    syn_get_id(curwin, lnum, col, false, NULL, true);

    int id;
    int i = 0;
    while ((id = syn_get_stack_item(i++)) >= 0) {
      tv_list_append_number(rettv->vval.v_list, id);
    }
  }
}

// =============================================================================
// api_info: moved from funcs.c
// =============================================================================

void nvim_eval_api_info(typval_T *argvars, typval_T *rettv)
{
  object_to_vim(api_metadata(), rettv, NULL);
}

// =============================================================================
// get_col / getpos_both: moved from funcs.c
// =============================================================================

void nvim_eval_get_col(typval_T *argvars, typval_T *rettv, bool charcol)
{
  if (tv_check_for_string_or_list_arg(argvars, 0) == FAIL
      || tv_check_for_opt_number_arg(argvars, 1) == FAIL) {
    return;
  }

  switchwin_T switchwin;
  bool winchanged = false;

  if (argvars[1].v_type != VAR_UNKNOWN) {
    // use the window specified in the second argument
    tabpage_T *tp;
    win_T *wp = win_id2wp_tp((int)tv_get_number(&argvars[1]), &tp);
    if (wp == NULL || tp == NULL) {
      return;
    }

    if (switch_win_noblock(&switchwin, wp, tp, true) != OK) {
      return;
    }

    check_cursor(curwin);
    winchanged = true;
  }

  colnr_T col = 0;
  int fnum = curbuf->b_fnum;
  pos_T *fp = var2fpos(&argvars[0], false, &fnum, charcol);
  if (fp != NULL && fnum == curbuf->b_fnum) {
    if (fp->col == MAXCOL) {
      // '> can be MAXCOL, get the length of the line then
      if (fp->lnum <= curbuf->b_ml.ml_line_count) {
        col = ml_get_len(fp->lnum) + 1;
      } else {
        col = MAXCOL;
      }
    } else {
      col = fp->col + 1;
      // col(".") when the cursor is on the NUL at the end of the line
      // because of "coladd" can be seen as an extra column.
      if (virtual_active(curwin) && fp == &curwin->w_cursor) {
        char *p = get_cursor_pos_ptr();
        if (curwin->w_cursor.coladd >=
            (colnr_T)win_chartabsize(curwin, p,
                                     curwin->w_virtcol - curwin->w_cursor.coladd)) {
          int l;
          if (*p != NUL && p[(l = utfc_ptr2len(p))] == NUL) {
            col += l;
          }
        }
      }
    }
  }
  rettv->vval.v_number = col;

  if (winchanged) {
    restore_win_noblock(&switchwin, true);
  }
}

void nvim_eval_getpos_both(typval_T *argvars, typval_T *rettv, bool getcurpos, bool charcol)
{
  pos_T *fp = NULL;
  pos_T pos;
  win_T *wp = curwin;
  int fnum = -1;

  if (getcurpos) {
    if (argvars[0].v_type != VAR_UNKNOWN) {
      wp = find_win_by_nr_or_id(&argvars[0]);
      if (wp != NULL) {
        fp = &wp->w_cursor;
      }
    } else {
      fp = &curwin->w_cursor;
    }
    if (fp != NULL && charcol) {
      pos = *fp;
      pos.col = rs_buf_byteidx_to_charidx(wp->w_buffer, pos.lnum, pos.col);
      fp = &pos;
    }
  } else {
    fp = var2fpos(&argvars[0], true, &fnum, charcol);
  }

  list_T *const l = tv_list_alloc_ret(rettv, 4 + getcurpos);
  tv_list_append_number(l, (fnum != -1) ? (varnumber_T)fnum : (varnumber_T)0);
  tv_list_append_number(l, ((fp != NULL) ? (varnumber_T)fp->lnum : (varnumber_T)0));
  tv_list_append_number(l, ((fp != NULL)
                            ? (varnumber_T)(fp->col == MAXCOL ? MAXCOL : fp->col + 1)
                            : (varnumber_T)0));
  tv_list_append_number(l, (fp != NULL) ? (varnumber_T)fp->coladd : (varnumber_T)0);
  if (getcurpos) {
    const bool save_set_curswant = curwin->w_set_curswant;
    const colnr_T save_curswant = curwin->w_curswant;
    const colnr_T save_virtcol = curwin->w_virtcol;

    if (wp == curwin) {
      update_curswant();
    }
    tv_list_append_number(l, (wp == NULL) ? 0 : ((wp->w_curswant == MAXCOL)
                                                 ? (varnumber_T)MAXCOL
                                                 : (varnumber_T)wp->w_curswant + 1));

    // Do not change "curswant", as it is unexpected that a get
    // function has a side effect.
    if (wp == curwin && save_set_curswant) {
      curwin->w_set_curswant = save_set_curswant;
      curwin->w_curswant = save_curswant;
      curwin->w_virtcol = save_virtcol;
      curwin->w_valid &= ~VALID_VIRTCOL;
    }
  }
}

// =============================================================================
// set_position / set_cursorpos: moved from funcs.c
// =============================================================================

void nvim_eval_set_position(typval_T *argvars, typval_T *rettv, bool charpos)
{
  colnr_T curswant = -1;

  rettv->vval.v_number = -1;
  const char *const name = tv_get_string_chk(argvars);
  if (name == NULL) {
    return;
  }

  pos_T pos;
  int fnum;
  if (list2fpos(&argvars[1], &pos, &fnum, &curswant, charpos) != OK) {
    return;
  }

  if (pos.col != MAXCOL && --pos.col < 0) {
    pos.col = 0;
  }
  if (name[0] == '.' && name[1] == NUL) {
    // set cursor; "fnum" is ignored
    curwin->w_cursor = pos;
    if (curswant >= 0) {
      curwin->w_curswant = curswant - 1;
      curwin->w_set_curswant = false;
    }
    check_cursor(curwin);
    rettv->vval.v_number = 0;
  } else if (name[0] == '\'' && name[1] != NUL && name[2] == NUL) {
    // set mark
    if (setmark_pos((uint8_t)name[1], &pos, fnum, NULL) == OK) {
      rettv->vval.v_number = 0;
    }
  } else {
    emsg(_(e_invarg));
  }
}

void nvim_eval_set_cursorpos(typval_T *argvars, typval_T *rettv, bool charcol)
{
  linenr_T lnum;
  colnr_T col;
  colnr_T coladd = 0;
  bool set_curswant = true;

  rettv->vval.v_number = -1;
  if (argvars[0].v_type == VAR_LIST) {
    pos_T pos;
    colnr_T curswant = -1;

    if (list2fpos(argvars, &pos, NULL, &curswant, charcol) == FAIL) {
      emsg(_(e_invarg));
      return;
    }

    lnum = pos.lnum;
    col = pos.col;
    coladd = pos.coladd;
    if (curswant >= 0) {
      curwin->w_curswant = curswant - 1;
      set_curswant = false;
    }
  } else if ((argvars[0].v_type == VAR_NUMBER || argvars[0].v_type == VAR_STRING)
             && (argvars[1].v_type == VAR_NUMBER || argvars[1].v_type == VAR_STRING)) {
    lnum = tv_get_lnum(argvars);
    if (lnum < 0) {
      semsg(_(e_invarg2), tv_get_string(&argvars[0]));
    } else if (lnum == 0) {
      lnum = curwin->w_cursor.lnum;
    }
    col = (colnr_T)tv_get_number_chk(&argvars[1], NULL);
    if (charcol) {
      col = rs_buf_charidx_to_byteidx(curbuf, lnum, (int)col) + 1;
    }
    if (argvars[2].v_type != VAR_UNKNOWN) {
      coladd = (colnr_T)tv_get_number_chk(&argvars[2], NULL);
    }
  } else {
    emsg(_(e_invarg));
    return;
  }
  if (lnum < 0 || col < 0 || coladd < 0) {
    return;  // type error; errmsg already given
  }
  if (lnum > 0) {
    curwin->w_cursor.lnum = lnum;
  }
  if (col != MAXCOL && --col < 0) {
    col = 0;
  }
  curwin->w_cursor.col = col;
  curwin->w_cursor.coladd = coladd;

  // Make sure the cursor is in a valid position.
  check_cursor(curwin);
  // Correct cursor for multi-byte character.
  mb_adjust_cursor();

  curwin->w_set_curswant = set_curswant;
  rettv->vval.v_number = 0;
}

// =============================================================================
// flatten: moved from funcs.c
// =============================================================================

void nvim_eval_flatten(typval_T *argvars, typval_T *rettv, bool make_copy)
{
  bool error = false;

  if (argvars[0].v_type != VAR_LIST) {
    semsg(_(e_listarg), "flatten()");
    return;
  }

  int maxdepth;
  if (argvars[1].v_type == VAR_UNKNOWN) {
    maxdepth = 999999;
  } else {
    maxdepth = (int)tv_get_number_chk(&argvars[1], &error);
    if (error) {
      return;
    }
    if (maxdepth < 0) {
      emsg(_("E900: maxdepth must be non-negative number"));
      return;
    }
  }

  list_T *list = argvars[0].vval.v_list;
  rettv->v_type = VAR_LIST;
  rettv->vval.v_list = list;
  if (list == NULL) {
    return;
  }

  if (make_copy) {
    list = tv_list_copy(NULL, list, false, rs_get_copyID());
    rettv->vval.v_list = list;
    if (list == NULL) {
      return;
    }
  } else {
    if (value_check_lock(tv_list_locked(list), N_("flatten() argument"), TV_TRANSLATE)) {
      return;
    }
    tv_list_ref(list);
  }

  tv_list_flatten(list, NULL, tv_list_len(list), maxdepth);
}

// =============================================================================
// timer: moved from funcs.c
// =============================================================================

void nvim_eval_timer_info(typval_T *argvars, typval_T *rettv)
{
  tv_list_alloc_ret(rettv, kListLenUnknown);

  if (tv_check_for_opt_number_arg(argvars, 0) == FAIL) {
    return;
  }

  if (argvars[0].v_type != VAR_UNKNOWN) {
    timer_T *timer = find_timer_by_nr(tv_get_number(&argvars[0]));
    if (timer != NULL && (!timer->stopped || timer->refcount > 1)) {
      add_timer_info(rettv, timer);
    }
  } else {
    add_timer_info_all(rettv);
  }
}

void nvim_eval_timer_pause(typval_T *argvars, typval_T *rettv)
{
  if (argvars[0].v_type != VAR_NUMBER) {
    emsg(_(e_number_exp));
    return;
  }

  int paused = (bool)tv_get_number(&argvars[1]);
  timer_T *timer = find_timer_by_nr(tv_get_number(&argvars[0]));
  if (timer != NULL) {
    if (!timer->paused && paused) {
      time_watcher_stop(&timer->tw);
    } else if (timer->paused && !paused) {
      time_watcher_start(&timer->tw, timer_due_cb, (uint64_t)timer->timeout,
                         (uint64_t)timer->timeout);
    }
    timer->paused = paused;
  }
}

void nvim_eval_timer_start(typval_T *argvars, typval_T *rettv)
{
  int repeat = 1;

  rettv->vval.v_number = -1;
  if (rs_check_secure()) {
    return;
  }

  if (argvars[2].v_type != VAR_UNKNOWN) {
    if (tv_check_for_nonnull_dict_arg(argvars, 2) == FAIL) {
      return;
    }
    dict_T *dict = argvars[2].vval.v_dict;
    dictitem_T *const di = tv_dict_find(dict, S_LEN("repeat"));
    if (di != NULL) {
      repeat = (int)tv_get_number(&di->di_tv);
      if (repeat == 0) {
        repeat = 1;
      }
    }
  }

  Callback callback;
  if (!rs_callback_from_typval(&callback, &argvars[1])) {
    return;
  }
  rettv->vval.v_number = (varnumber_T)timer_start(tv_get_number(&argvars[0]), repeat, &callback);
}

void nvim_eval_timer_stop(typval_T *argvars, typval_T *rettv)
{
  if (tv_check_for_number_arg(argvars, 0) == FAIL) {
    return;
  }

  timer_T *timer = find_timer_by_nr(tv_get_number(&argvars[0]));
  if (timer != NULL) {
    timer_stop(timer);
  }
}

// =============================================================================
// spellbadword / spellsuggest: moved from funcs.c
// =============================================================================

void nvim_eval_spellbadword(typval_T *argvars, typval_T *rettv)
{
  const int wo_spell_save = curwin->w_p_spell;

  if (!curwin->w_p_spell) {
    parse_spelllang(curwin);
    curwin->w_p_spell = true;
  }

  if (*curwin->w_s->b_p_spl == NUL) {
    emsg(_(e_no_spell));
    curwin->w_p_spell = wo_spell_save;
    return;
  }

  const char *word = "";
  hlf_T attr = HLF_COUNT;
  size_t len = 0;
  if (argvars[0].v_type == VAR_UNKNOWN) {
    // Find the start and length of the badly spelled word.
    len = spell_move_to(curwin, FORWARD, SMT_ALL, true, &attr);
    if (len != 0) {
      word = get_cursor_pos_ptr();
      curwin->w_set_curswant = true;
    }
  } else if (*curbuf->b_s.b_p_spl != NUL) {
    const char *str = tv_get_string_chk(&argvars[0]);
    int capcol = -1;

    if (str != NULL) {
      // Check the argument for spelling.
      while (*str != NUL) {
        len = spell_check(curwin, (char *)str, &attr, &capcol, false);
        if (attr != HLF_COUNT) {
          word = str;
          break;
        }
        str += len;
        capcol -= (int)len;
        len = 0;
      }
    }
  }
  curwin->w_p_spell = wo_spell_save;

  assert(len <= INT_MAX);
  tv_list_alloc_ret(rettv, 2);
  tv_list_append_string(rettv->vval.v_list, word, (ssize_t)len);
  tv_list_append_string(rettv->vval.v_list,
                        (attr == HLF_SPB
                         ? "bad" : (attr == HLF_SPR
                                    ? "rare" : (attr == HLF_SPL
                                                ? "local" : (attr == HLF_SPC
                                                             ? "caps" : NULL)))), -1);
}

void nvim_eval_spellsuggest(typval_T *argvars, typval_T *rettv)
{
  garray_T ga = GA_EMPTY_INIT_VALUE;
  const int wo_spell_save = curwin->w_p_spell;

  if (!curwin->w_p_spell) {
    parse_spelllang(curwin);
    curwin->w_p_spell = true;
  }

  if (*curwin->w_s->b_p_spl == NUL) {
    emsg(_(e_no_spell));
    curwin->w_p_spell = wo_spell_save;
    return;
  }

  int maxcount;
  bool need_capital = false;
  const char *const str = tv_get_string(&argvars[0]);
  if (argvars[1].v_type != VAR_UNKNOWN) {
    bool typeerr = false;
    maxcount = (int)tv_get_number_chk(&argvars[1], &typeerr);
    if (maxcount <= 0) {
      goto nvim_eval_spellsuggest_return;
    }
    if (argvars[2].v_type != VAR_UNKNOWN) {
      need_capital = tv_get_number_chk(&argvars[2], &typeerr);
      if (typeerr) {
        goto nvim_eval_spellsuggest_return;
      }
    }
  } else {
    maxcount = 25;
  }

  spell_suggest_list(&ga, (char *)str, maxcount, need_capital, false);

nvim_eval_spellsuggest_return:
  tv_list_alloc_ret(rettv, (ptrdiff_t)ga.ga_len);
  for (int i = 0; i < ga.ga_len; i++) {
    char *const p = ((char **)ga.ga_data)[i];
    tv_list_append_allocated_string(rettv->vval.v_list, p);
  }
  ga_clear(&ga);
  curwin->w_p_spell = wo_spell_save;
}

// =============================================================================
// Phase 22: moved from funcs.c
// =============================================================================

// nvim_eval_get_windows_version: moved from funcs.c
const char *nvim_eval_get_windows_version(void) { return windowsVersion; }

// nvim_eval_libcall: moved from funcs.c
void nvim_eval_libcall(typval_T *argvars, typval_T *rettv, bool retstr)
{
  int out_type = retstr ? VAR_STRING : VAR_NUMBER;

  rettv->v_type = (VarType)out_type;
  if (out_type != VAR_NUMBER) {
    rettv->vval.v_string = NULL;
  }

  if (rs_check_secure()) {
    return;
  }

  // The first two args (libname and funcname) must be strings
  if (argvars[0].v_type != VAR_STRING || argvars[1].v_type != VAR_STRING) {
    return;
  }

  const char *libname = argvars[0].vval.v_string;
  const char *funcname = argvars[1].vval.v_string;

  VarType in_type = argvars[2].v_type;

  // input variables
  char *str_in = (in_type == VAR_STRING) ? argvars[2].vval.v_string : NULL;
  int int_in = (int)argvars[2].vval.v_number;

  // output variables
  char **str_out = (out_type == VAR_STRING) ? &rettv->vval.v_string : NULL;
  int int_out = 0;

  bool success = os_libcall(libname, funcname,
                            str_in, int_in,
                            str_out, &int_out);

  if (!success) {
    semsg(_(e_libcall), funcname);
    return;
  }

  if (out_type == VAR_NUMBER) {
    rettv->vval.v_number = (varnumber_T)int_out;
  }
}

// dummy_ap + nvim_eval_printf: moved from funcs.c
/// Used because:
/// - passing a NULL pointer doesn't work when va_list isn't a pointer
/// - locally in the function results in a "used before set" warning
/// - using va_start() to initialize it gives "function with fixed args" error
static va_list dummy_ap_shim;

void nvim_eval_printf(typval_T *argvars, typval_T *rettv)
{
  rettv->v_type = VAR_STRING;
  rettv->vval.v_string = NULL;
  {
    int saved_did_emsg = did_emsg;

    // Get the required length, allocate the buffer and do it for real.
    did_emsg = false;
    char buf[NUMBUFLEN];
    const char *fmt = tv_get_string_buf(&argvars[0], buf);
    int len = vim_vsnprintf_typval(NULL, 0, fmt, dummy_ap_shim, argvars + 1);
    if (!did_emsg) {
      char *s = xmalloc((size_t)len + 1);
      rettv->vval.v_string = s;
      vim_vsnprintf_typval(s, (size_t)len + 1, fmt, dummy_ap_shim, argvars + 1);
    }
    did_emsg |= saved_did_emsg;
  }
}

// =============================================================================
// Phase 23: f_ VimL functions moved from funcs.c
// =============================================================================

/// "line()" function - get line number from position
void f_line(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  linenr_T lnum = 0;
  pos_T *fp = NULL;
  int fnum;

  if (argvars[1].v_type != VAR_UNKNOWN) {
    // use window specified in the second argument
    int id = (int)tv_get_number(&argvars[1]);
    tabpage_T *tp;
    win_T *wp = win_id2wp_tp(id, &tp);
    if (wp != NULL && tp != NULL) {
      switchwin_T switchwin;
      if (switch_win_noblock(&switchwin, wp, tp, true) == OK) {
        // With 'splitkeep' != cursor and in diff mode, prevent that the
        // window scrolls and keep the topline.
        if (*p_spk != 'c' || (curwin->w_p_diff && switchwin.sw_curwin->w_p_diff)) {
          skip_update_topline = true;
        }
        check_cursor(curwin);
        fp = var2fpos(&argvars[0], true, &fnum, false);
      }
      skip_update_topline = false;
      restore_win_noblock(&switchwin, true);
    }
  } else {
    // use current window
    fp = var2fpos(&argvars[0], true, &fnum, false);
  }

  if (fp != NULL) {
    lnum = fp->lnum;
  }
  rettv->vval.v_number = lnum;
}

/// "serverlist()" function
void f_serverlist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  size_t n;
  char **addrs = server_address_list(&n);

  Arena arena = ARENA_EMPTY;
  // Passed to vim._core.server.serverlist() to avoid duplicates
  Array addrs_arr = arena_array(&arena, n);

  // Copy addrs into a linked list.
  list_T *const l = tv_list_alloc_ret(rettv, (ptrdiff_t)n);
  for (size_t i = 0; i < n; i++) {
    tv_list_append_allocated_string(l, addrs[i]);
    ADD_C(addrs_arr, CSTR_AS_OBJ(addrs[i]));
  }

  if (!(argvars[0].v_type == VAR_DICT && tv_dict_get_bool(argvars[0].vval.v_dict, "peer", false))) {
    goto cleanup;
  }

  MAXSIZE_TEMP_ARRAY(args, 1);
  ADD_C(args, ARRAY_OBJ(addrs_arr));

  Error err = ERROR_INIT;
  Object rv = NLUA_EXEC_STATIC("return require('vim._core.server').serverlist(...)",
                               args, kRetObject,
                               &arena, &err);

  if (ERROR_SET(&err)) {
    ELOG("vim._core.serverlist failed: %s", err.msg);
    goto cleanup;
  }

  for (size_t i = 0; i < rv.data.array.size; i++) {
    char *curr_server = rv.data.array.items[i].data.string.data;
    tv_list_append_string(l, curr_server, -1);
  }

cleanup:
  xfree(addrs);
  arena_mem_free(arena_finish(&arena));
}

/// "swapname()" function
void f_swapname(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rettv->v_type = VAR_STRING;
  buf_T *buf = tv_get_buf(&argvars[0], false);
  if (buf == NULL
      || buf->b_ml.ml_mfp == NULL
      || buf->b_ml.ml_mfp->mf_fname == NULL) {
    rettv->vval.v_string = NULL;
  } else {
    rettv->vval.v_string = xstrdup(buf->b_ml.ml_mfp->mf_fname);
  }
}

/// "tabpagebuflist()" function
void f_tabpagebuflist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  win_T *wp = NULL;

  if (argvars[0].v_type == VAR_UNKNOWN) {
    wp = firstwin;
  } else {
    tabpage_T *const tp = rs_find_tabpage((int)tv_get_number(&argvars[0]));
    if (tp != NULL) {
      wp = (tp == curtab) ? firstwin : tp->tp_firstwin;
    }
  }
  if (wp != NULL) {
    tv_list_alloc_ret(rettv, kListLenMayKnow);
    while (wp != NULL) {
      tv_list_append_number(rettv->vval.v_list, wp->w_buffer->b_fnum);
      wp = wp->w_next;
    }
  }
}

/// "virtcol({expr}, [, {list} [, {winid}]])" function
void f_virtcol(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  colnr_T vcol_start = 0;
  colnr_T vcol_end = 0;
  switchwin_T switchwin;
  bool winchanged = false;

  if (argvars[1].v_type != VAR_UNKNOWN && argvars[2].v_type != VAR_UNKNOWN) {
    // use the window specified in the third argument
    tabpage_T *tp;
    win_T *wp = win_id2wp_tp((int)tv_get_number(&argvars[2]), &tp);
    if (wp == NULL || tp == NULL) {
      goto theend;
    }

    if (switch_win_noblock(&switchwin, wp, tp, true) != OK) {
      goto theend;
    }

    check_cursor(curwin);
    winchanged = true;
  }

  int fnum = curbuf->b_fnum;
  pos_T *fp = var2fpos(&argvars[0], false, &fnum, false);
  if (fp != NULL && fp->lnum <= curbuf->b_ml.ml_line_count
      && fnum == curbuf->b_fnum) {
    // Limit the column to a valid value, getvvcol() doesn't check.
    if (fp->col < 0) {
      fp->col = 0;
    } else {
      const colnr_T len = ml_get_len(fp->lnum);
      if (fp->col > len) {
        fp->col = len;
      }
    }
    getvvcol(curwin, fp, &vcol_start, NULL, &vcol_end);
    vcol_start++;
    vcol_end++;
  }

theend:
  if (argvars[1].v_type != VAR_UNKNOWN && tv_get_bool(&argvars[1])) {
    tv_list_alloc_ret(rettv, 2);
    tv_list_append_number(rettv->vval.v_list, vcol_start);
    tv_list_append_number(rettv->vval.v_list, vcol_end);
  } else {
    rettv->vval.v_number = vcol_end;
  }

  if (winchanged) {
    restore_win_noblock(&switchwin, true);
  }
}

// =============================================================================
// Phase 24: more f_ VimL functions moved from funcs.c
// =============================================================================

/// "getchangelist()" function
void f_getchangelist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  tv_list_alloc_ret(rettv, 2);

  const buf_T *buf;
  if (argvars[0].v_type == VAR_UNKNOWN) {
    buf = curbuf;
  } else {
    vim_ignored = (int)tv_get_number(&argvars[0]);  // issue errmsg if type error
    emsg_off++;
    buf = tv_get_buf(&argvars[0], false);
    emsg_off--;
  }
  if (buf == NULL) {
    return;
  }

  list_T *const l = tv_list_alloc(buf->b_changelistlen);
  tv_list_append_list(rettv->vval.v_list, l);
  // The current window change list index tracks only the position for the
  // current buffer. For other buffers use the stored index for the current
  // window, or, if that's not available, the change list length.
  int changelistindex;
  if (buf == curwin->w_buffer) {
    changelistindex = curwin->w_changelistidx;
  } else {
    changelistindex = buf->b_changelistlen;

    for (size_t i = 0; i < kv_size(buf->b_wininfo); i++) {
      WinInfo *wip = kv_A(buf->b_wininfo, i);
      if (wip->wi_win == curwin) {
        changelistindex = wip->wi_changelistidx;
        break;
      }
    }
  }
  tv_list_append_number(rettv->vval.v_list, (varnumber_T)changelistindex);

  for (int i = 0; i < buf->b_changelistlen; i++) {
    if (buf->b_changelist[i].mark.lnum == 0) {
      continue;
    }
    dict_T *const d = tv_dict_alloc();
    tv_list_append_dict(l, d);
    tv_dict_add_nr(d, S_LEN("lnum"), buf->b_changelist[i].mark.lnum);
    tv_dict_add_nr(d, S_LEN("col"), buf->b_changelist[i].mark.col);
    tv_dict_add_nr(d, S_LEN("coladd"), buf->b_changelist[i].mark.coladd);
  }
}

/// "getjumplist()" function
void f_getjumplist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  tv_list_alloc_ret(rettv, kListLenMayKnow);
  win_T *const wp = find_tabwin(&argvars[0], &argvars[1]);
  if (wp == NULL) {
    return;
  }

  cleanup_jumplist(wp, true);

  list_T *const l = tv_list_alloc(wp->w_jumplistlen);
  tv_list_append_list(rettv->vval.v_list, l);
  tv_list_append_number(rettv->vval.v_list, wp->w_jumplistidx);

  for (int i = 0; i < wp->w_jumplistlen; i++) {
    if (wp->w_jumplist[i].fmark.mark.lnum == 0) {
      continue;
    }
    dict_T *const d = tv_dict_alloc();
    tv_list_append_dict(l, d);
    tv_dict_add_nr(d, S_LEN("lnum"), wp->w_jumplist[i].fmark.mark.lnum);
    tv_dict_add_nr(d, S_LEN("col"), wp->w_jumplist[i].fmark.mark.col);
    tv_dict_add_nr(d, S_LEN("coladd"), wp->w_jumplist[i].fmark.mark.coladd);
    tv_dict_add_nr(d, S_LEN("bufnr"), wp->w_jumplist[i].fmark.fnum);
    if (wp->w_jumplist[i].fname != NULL) {
      tv_dict_add_str(d, S_LEN("filename"), wp->w_jumplist[i].fname);
    }
  }
}

/// "getmarklist()" function
void f_getmarklist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  tv_list_alloc_ret(rettv, kListLenMayKnow);

  if (argvars[0].v_type == VAR_UNKNOWN) {
    get_global_marks(rettv->vval.v_list);
    return;
  }

  buf_T *buf = tv_get_buf(&argvars[0], false);
  if (buf == NULL) {
    return;
  }

  get_buf_local_marks(buf, rettv->vval.v_list);
}

/// "gettagstack()" function
void f_gettagstack(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  win_T *wp = curwin;                  // default is current window

  tv_dict_alloc_ret(rettv);

  if (argvars[0].v_type != VAR_UNKNOWN) {
    wp = find_win_by_nr_or_id(&argvars[0]);
    if (wp == NULL) {
      return;
    }
  }

  rs_get_tagstack(wp, rettv->vval.v_dict);
}

/// "prompt_getprompt({buffer})" function
void f_prompt_getprompt(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
  FUNC_ATTR_NONNULL_ALL
{
  // return an empty string by default, e.g. it's not a prompt buffer
  rettv->v_type = VAR_STRING;
  rettv->vval.v_string = NULL;

  buf_T *const buf = tv_get_buf_from_arg(&argvars[0]);
  if (buf == NULL) {
    return;
  }

  if (!bt_prompt(buf)) {
    return;
  }

  rettv->vval.v_string = xstrdup(buf_prompt_text(buf));
}

/// "prompt_getinput({buffer})" function
void f_prompt_getinput(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
  FUNC_ATTR_NONNULL_ALL
{
  // return an empty string by default, e.g. it's not a prompt buffer
  rettv->v_type = VAR_STRING;
  rettv->vval.v_string = NULL;

  buf_T *const buf = tv_get_buf_from_arg(&argvars[0]);
  if (buf == NULL) {
    return;
  }

  if (!bt_prompt(buf)) {
    return;
  }

  rettv->vval.v_string = prompt_get_input(buf);
}

// =============================================================================
// Phase 25: more f_ VimL functions moved from funcs.c
// =============================================================================

/// "expandcmd()" function
/// Expand all the special characters in a command string.
void f_expandcmd(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  const char *errormsg = NULL;
  bool emsgoff = true;

  if (argvars[1].v_type == VAR_DICT
      && tv_dict_get_bool(argvars[1].vval.v_dict, "errmsg", kBoolVarFalse)) {
    emsgoff = false;
  }

  rettv->v_type = VAR_STRING;
  char *cmdstr = xstrdup(tv_get_string(&argvars[0]));

  exarg_T eap = {
    .cmd = cmdstr,
    .arg = cmdstr,
    .usefilter = false,
    .nextcmd = NULL,
    .cmdidx = CMD_USER,
  };
  eap.argt |= EX_NOSPC;

  if (emsgoff) {
    emsg_off++;
  }
  if (expand_filename(&eap, &cmdstr, &errormsg) == FAIL) {
    if (!emsgoff && errormsg != NULL && *errormsg != NUL) {
      emsg(errormsg);
    }
  }
  if (emsgoff) {
    emsg_off--;
  }

  rettv->vval.v_string = cmdstr;
}

/// "islocked()" function
void f_islocked(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  lval_T lv;

  rettv->vval.v_number = -1;
  const char *const end = get_lval((char *)tv_get_string(&argvars[0]),
                                   NULL,
                                   &lv, false, false,
                                   GLV_NO_AUTOLOAD|GLV_READ_ONLY,
                                   FNE_CHECK_START);
  if (end != NULL && lv.ll_name != NULL) {
    if (*end != NUL) {
      semsg(_(lv.ll_name_len == 0 ? e_invarg2 : e_trailing_arg), end);
    } else {
      if (lv.ll_tv == NULL) {
        dictitem_T *di = find_var(lv.ll_name, lv.ll_name_len, NULL, true);
        if (di != NULL) {
          // Consider a variable locked when:
          // 1. the variable itself is locked
          // 2. the value of the variable is locked.
          // 3. the List or Dict value is locked.
          rettv->vval.v_number = ((di->di_flags & DI_FLAGS_LOCK)
                                  || tv_islocked(&di->di_tv));
        }
      } else if (lv.ll_range) {
        emsg(_("E786: Range not allowed"));
      } else if (lv.ll_newkey != NULL) {
        semsg(_(e_dictkey), lv.ll_newkey);
      } else if (lv.ll_list != NULL) {
        // List item.
        rettv->vval.v_number = tv_islocked(TV_LIST_ITEM_TV(lv.ll_li));
      } else {
        // Dictionary item.
        rettv->vval.v_number = tv_islocked(&lv.ll_di->di_tv);
      }
    }
  }

  clear_lval(&lv);
}

/// "settagstack()" function
void f_settagstack(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  static const char *e_invact2 = N_("E962: Invalid action: '%s'");
  char action = 'r';

  rettv->vval.v_number = -1;

  // first argument: window number or id
  win_T *wp = find_win_by_nr_or_id(&argvars[0]);
  if (wp == NULL) {
    return;
  }

  // second argument: dict with items to set in the tag stack
  if (tv_check_for_dict_arg(argvars, 1) == FAIL) {
    return;
  }
  dict_T *d = argvars[1].vval.v_dict;
  if (d == NULL) {
    return;
  }

  // third argument: action - 'a' for append and 'r' for replace.
  // default is to replace the stack.
  if (argvars[2].v_type == VAR_UNKNOWN) {
    // action = 'r';
  } else if (tv_check_for_string_arg(argvars, 2) == FAIL) {
    return;
  } else {
    const char *actstr;
    actstr = tv_get_string_chk(&argvars[2]);
    if (actstr == NULL) {
      return;
    }
    if ((*actstr == 'r' || *actstr == 'a' || *actstr == 't')
        && actstr[1] == NUL) {
      action = *actstr;
    } else {
      semsg(_(e_invact2), actstr);
      return;
    }
  }

  if (rs_set_tagstack(wp, d, action) == OK) {
    rettv->vval.v_number = 0;
  }
}

// =============================================================================
// Phase 26: more f_ VimL functions moved from funcs.c
// =============================================================================

/// "call(func, arglist [, dict])" function
void f_call(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  if (tv_check_for_list_arg(argvars, 1) == FAIL) {
    return;
  }
  if (argvars[1].vval.v_list == NULL) {
    return;
  }

  bool owned = false;
  char *func;
  partial_T *partial = NULL;
  if (argvars[0].v_type == VAR_FUNC) {
    func = argvars[0].vval.v_string;
  } else if (argvars[0].v_type == VAR_PARTIAL) {
    partial = argvars[0].vval.v_partial;
    func = rs_partial_name(partial);
  } else if (nlua_is_table_from_lua(&argvars[0])) {
    // TODO(tjdevries): UnifiedCallback
    func = nlua_register_table_as_callable(&argvars[0]);
    owned = true;
  } else {
    func = (char *)tv_get_string(&argvars[0]);
  }

  if (func == NULL || *func == NUL) {
    return;         // type error, empty name or null function
  }
  char *tofree = NULL;
  if (argvars[0].v_type == VAR_STRING) {
    char *p = func;
    tofree = trans_function_name(&p, false, TFN_INT|TFN_QUIET, NULL, NULL);
    if (tofree == NULL) {
      emsg_funcname(e_unknown_function_str, func);
      return;
    }
    func = tofree;
  }

  dict_T *selfdict = NULL;
  if (argvars[2].v_type != VAR_UNKNOWN) {
    if (tv_check_for_dict_arg(argvars, 2) == FAIL) {
      goto done;
    }
    selfdict = argvars[2].vval.v_dict;
  }

  func_call(func, &argvars[1], partial, selfdict, rettv);

done:
  if (owned) {
    func_unref(func);
  }
  xfree(tofree);
}

/// "expand()" function
void f_expand(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  int options = WILD_SILENT|WILD_USE_NL|WILD_LIST_NOTFOUND;
  bool error = false;
#ifdef BACKSLASH_IN_FILENAME
  char *p_csl_save = p_csl;

  // avoid using 'completeslash' here
  p_csl = empty_string_option;
#endif

  rettv->v_type = VAR_STRING;
  if (argvars[1].v_type != VAR_UNKNOWN
      && argvars[2].v_type != VAR_UNKNOWN
      && tv_get_number_chk(&argvars[2], &error)
      && !error) {
    tv_list_set_ret(rettv, NULL);
  }

  const char *s = tv_get_string(&argvars[0]);
  if (*s == '%' || *s == '#' || *s == '<') {
    if (p_verbose == 0) {
      emsg_off++;
    }
    size_t len;
    const char *errormsg = NULL;
    char *result = eval_vars((char *)s, s, &len, NULL, &errormsg, NULL, false);
    if (p_verbose == 0) {
      emsg_off--;
    } else if (errormsg != NULL) {
      emsg(errormsg);
    }
    if (rettv->v_type == VAR_LIST) {
      tv_list_alloc_ret(rettv, (result != NULL));
      if (result != NULL) {
        tv_list_append_string(rettv->vval.v_list, result, -1);
      }
      XFREE_CLEAR(result);
    } else {
      rettv->vval.v_string = result;
    }
  } else {
    // When the optional second argument is non-zero, don't remove matches
    // for 'wildignore' and don't put matches for 'suffixes' at the end.
    if (argvars[1].v_type != VAR_UNKNOWN
        && tv_get_number_chk(&argvars[1], &error)) {
      options |= WILD_KEEP_ALL;
    }
    if (!error) {
      expand_T xpc;
      ExpandInit(&xpc);
      xpc.xp_context = EXPAND_FILES;
      if (p_wic) {
        options += WILD_ICASE;
      }
      if (rettv->v_type == VAR_STRING) {
        rettv->vval.v_string = ExpandOne(&xpc, (char *)s, NULL, options, WILD_ALL);
      } else {
        ExpandOne(&xpc, (char *)s, NULL, options, WILD_ALL_KEEP);
        tv_list_alloc_ret(rettv, xpc.xp_numfiles);
        for (int i = 0; i < xpc.xp_numfiles; i++) {
          tv_list_append_string(rettv->vval.v_list, xpc.xp_files[i], -1);
        }
        ExpandCleanup(&xpc);
      }
    } else {
      rettv->vval.v_string = NULL;
    }
  }
#ifdef BACKSLASH_IN_FILENAME
  p_csl = p_csl_save;
#endif
}

/// "split()" function
void f_split(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  colnr_T col = 0;
  bool keepempty = false;
  bool typeerr = false;

  // Make 'cpoptions' empty, the 'l' flag should not be used here.
  char *save_cpo = p_cpo;
  p_cpo = empty_string_option;

  const char *str = tv_get_string(&argvars[0]);
  const char *pat = NULL;
  char patbuf[NUMBUFLEN];
  if (argvars[1].v_type != VAR_UNKNOWN) {
    pat = tv_get_string_buf_chk(&argvars[1], patbuf);
    if (pat == NULL) {
      typeerr = true;
    }
    if (argvars[2].v_type != VAR_UNKNOWN) {
      keepempty = (bool)tv_get_bool_chk(&argvars[2], &typeerr);
    }
  }
  if (pat == NULL || *pat == NUL) {
    pat = "[\\x01- ]\\+";
  }

  tv_list_alloc_ret(rettv, kListLenMayKnow);

  if (typeerr) {
    goto theend;
  }

  regmatch_T regmatch = {
    .regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING),
    .startp = { NULL },
    .endp = { NULL },
    .rm_ic = false,
  };
  if (regmatch.regprog != NULL) {
    while (*str != NUL || keepempty) {
      bool match;
      if (*str == NUL) {
        match = false;  // Empty item at the end.
      } else {
        match = vim_regexec_nl(&regmatch, str, col);
      }
      const char *end;
      if (match) {
        end = regmatch.startp[0];
      } else {
        end = str + strlen(str);
      }
      if (keepempty || end > str || (tv_list_len(rettv->vval.v_list) > 0
                                     && *str != NUL
                                     && match
                                     && end < regmatch.endp[0])) {
        tv_list_append_string(rettv->vval.v_list, str, end - str);
      }
      if (!match) {
        break;
      }
      // Advance to just after the match.
      if (regmatch.endp[0] > str) {
        col = 0;
      } else {
        // Don't get stuck at the same match.
        col = utfc_ptr2len(regmatch.endp[0]);
      }
      str = regmatch.endp[0];
    }

    vim_regfree(regmatch.regprog);
  }

theend:
  p_cpo = save_cpo;
}

// =============================================================================
// Phase 27: more f_ VimL functions moved from funcs.c
// =============================================================================

/// "get()" function
void f_get(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  typval_T *tv = NULL;
  bool what_is_dict = false;

  if (argvars[0].v_type == VAR_BLOB) {
    bool error = false;
    int idx = (int)tv_get_number_chk(&argvars[1], &error);

    if (!error) {
      rettv->v_type = VAR_NUMBER;
      if (idx < 0) {
        idx = tv_blob_len(argvars[0].vval.v_blob) + idx;
      }
      if (idx < 0 || idx >= tv_blob_len(argvars[0].vval.v_blob)) {
        rettv->vval.v_number = -1;
      } else {
        rettv->vval.v_number = tv_blob_get(argvars[0].vval.v_blob, idx);
        tv = rettv;
      }
    }
  } else if (argvars[0].v_type == VAR_LIST) {
    list_T *l = argvars[0].vval.v_list;
    if (l != NULL) {
      bool error = false;

      listitem_T *li = tv_list_find(l, (int)tv_get_number_chk(&argvars[1], &error));
      if (!error && li != NULL) {
        tv = TV_LIST_ITEM_TV(li);
      }
    }
  } else if (argvars[0].v_type == VAR_DICT) {
    dict_T *d = argvars[0].vval.v_dict;
    if (d != NULL) {
      dictitem_T *di = tv_dict_find(d, tv_get_string(&argvars[1]), -1);
      if (di != NULL) {
        tv = &di->di_tv;
      }
    }
  } else if (tv_is_func(argvars[0])) {
    partial_T *pt;
    partial_T fref_pt;

    if (argvars[0].v_type == VAR_PARTIAL) {
      pt = argvars[0].vval.v_partial;
    } else {
      CLEAR_FIELD(fref_pt);
      fref_pt.pt_name = argvars[0].vval.v_string;
      pt = &fref_pt;
    }

    if (pt != NULL) {
      const char *const what = tv_get_string(&argvars[1]);

      if (strcmp(what, "func") == 0 || strcmp(what, "name") == 0) {
        const char *name = rs_partial_name(pt);
        rettv->v_type = (*what == 'f' ? VAR_FUNC : VAR_STRING);
        assert(name != NULL);
        if (rettv->v_type == VAR_FUNC) {
          func_ref((char *)name);
        }
        if (*what == 'n' && pt->pt_name == NULL && pt->pt_func != NULL) {
          // use <SNR> instead of the byte code
          name = printable_func_name(pt->pt_func);
        }
        rettv->vval.v_string = xstrdup(name);
      } else if (strcmp(what, "dict") == 0) {
        what_is_dict = true;
        if (pt->pt_dict != NULL) {
          tv_dict_set_ret(rettv, pt->pt_dict);
        }
      } else if (strcmp(what, "args") == 0) {
        rettv->v_type = VAR_LIST;
        tv_list_alloc_ret(rettv, pt->pt_argc);
        for (int i = 0; i < pt->pt_argc; i++) {
          tv_list_append_tv(rettv->vval.v_list, &pt->pt_argv[i]);
        }
      } else if (strcmp(what, "arity") == 0) {
        int required = 0;
        int optional = 0;
        bool varargs = false;
        const char *name = rs_partial_name(pt);

        get_func_arity(name, &required, &optional, &varargs);

        rettv->v_type = VAR_DICT;
        tv_dict_alloc_ret(rettv);
        dict_T *dict = rettv->vval.v_dict;

        // Take into account the arguments of the partial, if any.
        // Note that it is possible to supply more arguments than the function
        // accepts.
        if (pt->pt_argc >= required + optional) {
          required = optional = 0;
        } else if (pt->pt_argc > required) {
          optional -= pt->pt_argc - required;
          required = 0;
        } else {
          required -= pt->pt_argc;
        }

        tv_dict_add_nr(dict, S_LEN("required"), required);
        tv_dict_add_nr(dict, S_LEN("optional"), optional);
        tv_dict_add_bool(dict, S_LEN("varargs"), varargs);
      } else {
        semsg(_(e_invarg2), what);
      }

      // When {what} == "dict" and pt->pt_dict == NULL, evaluate the
      // third argument
      if (!what_is_dict) {
        return;
      }
    }
  } else {
    semsg(_(e_listdictblobarg), "get()");
  }

  if (tv == NULL) {
    if (argvars[2].v_type != VAR_UNKNOWN) {
      tv_copy(&argvars[2], rettv);
    }
  } else {
    tv_copy(tv, rettv);
  }
}

/// Translate a register type string to the yank type and block length
static int get_yank_type(char **const pp, MotionType *const yank_type, int *const block_len)
  FUNC_ATTR_NONNULL_ALL
{
  char *stropt = *pp;
  switch (*stropt) {
  case 'v':
  case 'c':  // character-wise selection
    *yank_type = kMTCharWise;
    break;
  case 'V':
  case 'l':  // line-wise selection
    *yank_type = kMTLineWise;
    break;
  case 'b':
  case Ctrl_V:  // block-wise selection
    *yank_type = kMTBlockWise;
    if (ascii_isdigit(stropt[1])) {
      stropt++;
      *block_len = getdigits_int(&stropt, false, 0) - 1;
      stropt--;
    }
    break;
  default:
    return FAIL;
  }
  *pp = stropt;
  return OK;
}

/// "setreg()" function
void f_setreg(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  bool append = false;

  int block_len = -1;
  MotionType yank_type = kMTUnknown;

  rettv->vval.v_number = 1;  // FAIL is default.

  const char *const strregname = tv_get_string_chk(argvars);
  if (strregname == NULL) {
    return;  // Type error; errmsg already given.
  }
  char regname = *strregname;
  if (regname == 0 || regname == '@') {
    regname = '"';
  }

  const typval_T *regcontents = NULL;
  char pointreg = 0;
  if (argvars[1].v_type == VAR_DICT) {
    dict_T *const d = argvars[1].vval.v_dict;

    if (tv_dict_len(d) == 0) {
      // Empty dict, clear the register (like setreg(0, []))
      char *lstval[2] = { NULL, NULL };
      write_reg_contents_lst(regname, lstval, false, kMTUnknown, -1);
      return;
    }

    dictitem_T *const di = tv_dict_find(d, "regcontents", -1);
    if (di != NULL) {
      regcontents = &di->di_tv;
    }

    const char *stropt = tv_dict_get_string(d, "regtype", false);
    if (stropt != NULL) {
      const int ret = get_yank_type((char **)&stropt, &yank_type, &block_len);

      if (ret == FAIL || *(++stropt) != NUL) {
        semsg(_(e_invargval), "value");
        return;
      }
    }

    if (regname == '"') {
      stropt = tv_dict_get_string(d, "points_to", false);
      if (stropt != NULL) {
        pointreg = *stropt;
        regname = pointreg;
      }
    } else if (tv_dict_get_number(d, "isunnamed")) {
      pointreg = regname;
    }
  } else {
    regcontents = &argvars[1];
  }

  bool set_unnamed = false;
  if (argvars[2].v_type != VAR_UNKNOWN) {
    if (yank_type != kMTUnknown) {
      semsg(_(e_toomanyarg), "setreg");
      return;
    }

    const char *stropt = tv_get_string_chk(&argvars[2]);
    if (stropt == NULL) {
      return;  // Type error.
    }
    for (; *stropt != NUL; stropt++) {
      switch (*stropt) {
      case 'a':
      case 'A':    // append
        append = true;
        break;
      case 'u':
      case '"':    // unnamed register
        set_unnamed = true;
        break;
      default:
        get_yank_type((char **)&stropt, &yank_type, &block_len);
      }
    }
  }

  if (regcontents != NULL && regcontents->v_type == VAR_LIST) {
    list_T *const ll = regcontents->vval.v_list;
    // If the list is NULL handle like an empty list.
    const int len = tv_list_len(ll);

    // First half: use for pointers to result lines; second half: use for
    // pointers to allocated copies.
    char **lstval = xmalloc(sizeof(char *) * (((size_t)len + 1) * 2));
    const char **curval = (const char **)lstval;
    char **allocval = lstval + len + 2;
    char **curallocval = allocval;

    TV_LIST_ITER_CONST(ll, li, {
      char buf[NUMBUFLEN];
      *curval = tv_get_string_buf_chk(TV_LIST_ITEM_TV(li), buf);
      if (*curval == NULL) {
        goto free_lstval;
      }
      if (*curval == buf) {
        // Need to make a copy,
        // next tv_get_string_buf_chk() will overwrite the string.
        *curallocval = xstrdup(*curval);
        *curval = *curallocval;
        curallocval++;
      }
      curval++;
    });
    *curval++ = NULL;

    write_reg_contents_lst(regname, lstval, append, yank_type, (colnr_T)block_len);

free_lstval:
    while (curallocval > allocval) {
      xfree(*--curallocval);
    }
    xfree(lstval);
  } else if (regcontents != NULL) {
    const char *const strval = tv_get_string_chk(regcontents);
    if (strval == NULL) {
      return;
    }
    write_reg_contents_ex(regname, strval, (ssize_t)strlen(strval),
                          append, yank_type, (colnr_T)block_len);
  }
  if (pointreg != 0) {
    get_yank_register(pointreg, YREG_YANK);
  }
  rettv->vval.v_number = 0;

  if (set_unnamed) {
    // Discard the result. We already handle the error case.
    op_reg_set_previous(regname);
  }
}
