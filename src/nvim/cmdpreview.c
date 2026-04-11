// cmdpreview.c: Command preview ('inccommand') support.

#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include "klib/kvec.h"
#include "nvim/api/extmark.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/api/vim.h"
#include "nvim/autocmd.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_getln.h"
#include "nvim/extmark.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/globals.h"
#include "nvim/map_defs.h"
#include "nvim/mark_defs.h"
#include "nvim/memory.h"
#include "nvim/memory_defs.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/pos_defs.h"
#include "nvim/search.h"
#include "nvim/state_defs.h"
#include "nvim/types_defs.h"
#include "nvim/undo.h"
#include "nvim/undo_defs.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"
#include "nvim/cmdpreview.h"

#include "cmdpreview.c.generated.h"

// Command preview helpers from Rust
extern buf_T *rs_cmdpreview_open_buf(void);
extern win_T *rs_cmdpreview_open_win(buf_T *cmdpreview_buf);
extern void rs_cmdpreview_close_win(void);

static handle_T cmdpreview_bufnr = 0;
static int cmdpreview_ns = 0;

// C accessors for static variables (used by Rust)
handle_T nvim_get_cmdpreview_bufnr(void) { return cmdpreview_bufnr; }
void nvim_set_cmdpreview_bufnr(handle_T val) { cmdpreview_bufnr = val; }
int nvim_get_cmdpreview_ns(void) { return cmdpreview_ns; }
void nvim_set_cmdpreview_ns(int val) { cmdpreview_ns = val; }

// C helper wrappers used by Rust implementations of cmdpreview functions.

/// Create an unlisted scratch buffer. Returns handle on success, -1 on error.
int nvim_cmdpreview_create_buf(void)
{
  Error err = ERROR_INIT;
  handle_T bufnr = nvim_create_buf(false, true, &err);
  if (ERROR_SET(&err)) {
    api_clear_error(&err);
    return -1;
  }
  return (int)bufnr;
}

/// Switch to buf via do_buffer_ext wrapped in TRY_WRAP.
/// Returns OK (0) or FAIL (-1).
int nvim_cmdpreview_try_do_buffer(handle_T buf_handle)
{
  Error err = ERROR_INIT;
  int result = OK;
  TRY_WRAP(&err, {
    result = do_buffer_ext(DOBUF_GOTO, DOBUF_FIRST, FORWARD, buf_handle, 0);
  });
  if (ERROR_SET(&err) || result == FAIL) {
    api_clear_error(&err);
    return FAIL;
  }
  return OK;
}

/// Get the 'cmdwinheight' option value.
int nvim_get_p_cwh(void) { return (int)p_cwh; }

/// Set window options for preview windows.
void nvim_win_set_preview_options(win_T *win)
{
  win->w_p_cul = false;
  win->w_p_cuc = false;
  win->w_p_spell = false;
  win->w_p_fen = false;
}

/// Get w_p_cul for a window.
int nvim_win_get_w_p_cul(win_T *win) { return win->w_p_cul ? 1 : 0; }
/// Get w_p_cuc for a window.
int nvim_win_get_w_p_cuc(win_T *win) { return win->w_p_cuc ? 1 : 0; }
/// Set b_p_tw for a buffer.
void nvim_buf_set_b_p_tw(buf_T *buf, OptInt val) { buf->b_p_tw = val; }

/// Save cmdmod to a heap-allocated buffer. Returns opaque pointer.
void *nvim_cmdpreview_save_cmdmod(void)
{
  cmdmod_T *save = xmalloc(sizeof(cmdmod_T));
  nvim_cmdmod_store_to_save(save);
  return save;
}

/// Restore cmdmod from heap-allocated save and free it.
void nvim_cmdpreview_restore_cmdmod(void *saved)
{
  cmdmod_T *save = (cmdmod_T *)saved;
  nvim_cmdmod_restore_from_save(save);
  xfree(save);
}

/// Set cmdmod fields for command preview mode.
void nvim_cmdpreview_setup_cmdmod(void)
{
  p_hls = false;                 // Don't show search highlighting during live substitution
  cmdmod.cmod_split = 0;         // Disable :leftabove/botright modifiers
  cmdmod.cmod_tab = 0;           // Disable :tab modifier
  cmdmod.cmod_flags |= CMOD_NOSWAPFILE;  // Disable swap for preview buffer
}

/// Get b_op_start lnum from buffer.
linenr_T nvim_buf_get_b_op_start_lnum(buf_T *buf) { return buf->b_op_start.lnum; }
/// Get b_op_start col from buffer.
colnr_T nvim_buf_get_b_op_start_col(buf_T *buf) { return buf->b_op_start.col; }
/// Get b_op_start coladd from buffer.
colnr_T nvim_buf_get_b_op_start_coladd(buf_T *buf) { return buf->b_op_start.coladd; }
/// Get b_op_end lnum from buffer.
linenr_T nvim_buf_get_b_op_end_lnum(buf_T *buf) { return buf->b_op_end.lnum; }
/// Get b_op_end col from buffer.
colnr_T nvim_buf_get_b_op_end_col(buf_T *buf) { return buf->b_op_end.col; }
/// Get b_op_end coladd from buffer.
colnr_T nvim_buf_get_b_op_end_coladd(buf_T *buf) { return buf->b_op_end.coladd; }
/// Set b_op_start from lnum/col/coladd.
void nvim_buf_set_b_op_start(buf_T *buf, linenr_T lnum, colnr_T col, colnr_T coladd)
{ buf->b_op_start.lnum = lnum; buf->b_op_start.col = col; buf->b_op_start.coladd = coladd; }
/// Set b_op_end from lnum/col/coladd.
void nvim_buf_set_b_op_end(buf_T *buf, linenr_T lnum, colnr_T col, colnr_T coladd)
{ buf->b_op_end.lnum = lnum; buf->b_op_end.col = col; buf->b_op_end.coladd = coladd; }

/// Count undo steps needed to restore to an earlier state.
/// Iterates b_u_curhead (if non-null) or b_u_newhead chain.
int nvim_buf_count_undo_steps(buf_T *buf)
{
  int count = 0;
  for (u_header_T *uhp = buf->b_u_curhead ? buf->b_u_curhead : buf->b_u_newhead;
       uhp != NULL;
       uhp = uhp->uh_next.ptr, ++count) {}
  return count;
}

/// Returns b_u_curhead == NULL ? 1 : 0.
int nvim_buf_u_curhead_is_null(buf_T *buf) { return buf->b_u_curhead == NULL ? 1 : 0; }

/// Allocate a heap garray_T initialised for int storage. Returns opaque pointer.
void *nvim_ga_alloc_int(void)
{
  garray_T *gap = xcalloc(1, sizeof(garray_T));
  ga_init(gap, (int)sizeof(int), 1);
  return gap;
}

/// Clear and free a heap-allocated garray_T.
void nvim_ga_clear_free(void *gap_void)
{
  garray_T *gap = (garray_T *)gap_void;
  ga_clear(gap);
  xfree(gap);
}

/// Prepare Rust-managed cmdpreview state. Returns opaque pointer.
extern void *rs_cmdpreview_prepare(void);
/// Restore Rust-managed cmdpreview state and free it.
extern void rs_cmdpreview_restore_state(void *state);

/// Heap-allocated context combining exarg_T + CmdParseInfo for Rust FFI.
typedef struct {
  exarg_T ea;
  CmdParseInfo cmdinfo;
  const char *errormsg;
} CpParseCtx;

/// Allocate a zeroed CpParseCtx. Returns opaque pointer.
void *nvim_cmdpreview_alloc_parse_ctx(void)
{
  return xcalloc(1, sizeof(CpParseCtx));
}

/// Free a CpParseCtx. (exarg_T / CmdParseInfo do not own heap data
/// after parse_cmdline; the cmdline pointer is caller-owned.)
void nvim_cmdpreview_free_parse_ctx(void *ctx)
{
  xfree(ctx);
}

/// Run parse_cmdline on a copy of the cmdline. Returns true on success.
bool nvim_cmdpreview_do_parse(void *ctx_void, char **cmdline)
{
  CpParseCtx *ctx = (CpParseCtx *)ctx_void;
  return parse_cmdline(cmdline, &ctx->ea, &ctx->cmdinfo, &ctx->errormsg);
}

/// Return true if the parsed command has EX_PREVIEW set.
bool nvim_cmdpreview_ctx_has_preview(void *ctx_void)
{
  return ((CpParseCtx *)ctx_void)->ea.argt & EX_PREVIEW;
}

/// Return true if the parsed command has EX_RANGE set.
bool nvim_cmdpreview_ctx_has_range(void *ctx_void)
{
  return ((CpParseCtx *)ctx_void)->ea.argt & EX_RANGE;
}

/// Get ea.line1.
linenr_T nvim_cmdpreview_ctx_get_line1(void *ctx_void)
{
  return ((CpParseCtx *)ctx_void)->ea.line1;
}

/// Get ea.line2.
linenr_T nvim_cmdpreview_ctx_get_line2(void *ctx_void)
{
  return ((CpParseCtx *)ctx_void)->ea.line2;
}

/// Set ea.line1.
void nvim_cmdpreview_ctx_set_line1(void *ctx_void, linenr_T val)
{
  ((CpParseCtx *)ctx_void)->ea.line1 = val;
}

/// Set ea.line2.
void nvim_cmdpreview_ctx_set_line2(void *ctx_void, linenr_T val)
{
  ((CpParseCtx *)ctx_void)->ea.line2 = val;
}

/// Call undo_cmdmod on the parsed cmdinfo.cmdmod.
void nvim_cmdpreview_ctx_undo_cmdmod(void *ctx_void)
{
  undo_cmdmod(&((CpParseCtx *)ctx_void)->cmdinfo.cmdmod);
}

/// Execute the preview command via TRY_WRAP. Returns cmdpreview_type (0/1/2).
int nvim_cmdpreview_try_execute(void *ctx_void)
{
  CpParseCtx *ctx = (CpParseCtx *)ctx_void;
  Error err = ERROR_INIT;
  int cmdpreview_type = 0;
  TRY_WRAP(&err, {
    cmdpreview_type = execute_cmd(&ctx->ea, &ctx->cmdinfo, true);
  });
  if (ERROR_SET(&err)) {
    api_clear_error(&err);
    cmdpreview_type = 0;
  }
  return cmdpreview_type;
}

/// Get the cmdpreview global bool.
bool nvim_get_cmdpreview_global(void) { return cmdpreview; }

/// Set the cmdpreview global bool.
void nvim_set_cmdpreview_global(bool val) { cmdpreview = val; }

/// Return true if p_icm starts with 's' (inccommand=split).
bool nvim_get_p_icm_is_split(void) { return *p_icm == 's'; }

/// Set inccommand option to "nosplit".
void nvim_set_option_icm_nosplit(void)
{
  set_option_direct(kOptInccommand, STATIC_CSTR_AS_OPTVAL("nosplit"), 0, SID_NONE);
}

/// Ensure cmdpreview_ns is set; return the ns value.
int nvim_cmdpreview_ensure_ns(void)
{
  if (!cmdpreview_ns) {
    cmdpreview_ns = (int)nvim_create_namespace((String)STRING_INIT);
  }
  return cmdpreview_ns;
}

/// main cmdpreview_may_show implementation, implemented in Rust.
extern bool rs_cmdpreview_may_show(void);

/// Show 'inccommand' preview if command is previewable.
/// Implementation is in Rust (src/nvim-rs/cmdline/src/preview.rs).
///
/// @return whether preview is shown or not.
bool cmdpreview_may_show(void)
{
  return rs_cmdpreview_may_show();
}
