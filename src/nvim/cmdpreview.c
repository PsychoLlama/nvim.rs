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

/// Sets up command preview buffer. Implemented in Rust.
///
/// @return Pointer to command preview buffer if succeeded, NULL if failed.
static buf_T *cmdpreview_open_buf(void)
{
  return rs_cmdpreview_open_buf();
}

/// Open command preview window if it's not already open. Implemented in Rust.
///
/// @param cmdpreview_buf Pointer to command preview buffer
///
/// @return Pointer to command preview window if succeeded, NULL if failed.
static win_T *cmdpreview_open_win(buf_T *cmdpreview_buf)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_cmdpreview_open_win(cmdpreview_buf);
}

/// Closes any open command preview windows. Implemented in Rust.
static void cmdpreview_close_win(void)
{
  rs_cmdpreview_close_win();
}

// cmdpreview_prepare and cmdpreview_restore_state are implemented in Rust
// (src/nvim-rs/cmdline/src/preview.rs).

/// Show 'inccommand' preview if command is previewable. It works like this:
///    1. Store current undo information so we can revert to current state later.
///    2. Execute the preview callback with the parsed command, preview buffer number and preview
///       namespace number as arguments. The preview callback sets the highlight and does the
///       changes required for the preview if needed.
///    3. Preview callback returns 0, 1 or 2. 0 means no preview is shown. 1 means preview is shown
///       but preview window doesn't need to be opened. 2 means preview is shown and preview window
///       needs to be opened if inccommand=split.
///    4. Use the return value of the preview callback to determine whether to
///       open the preview window or not and open preview window if needed.
///    5. If the return value of the preview callback is not 0, update the screen while the effects
///       of the preview are still in place.
///    6. Revert all changes made by the preview callback.
///
/// @return whether preview is shown or not.
bool cmdpreview_may_show(void)
{
  // Parse the command line and return if it fails.
  exarg_T ea;
  CmdParseInfo cmdinfo;
  // Copy the command line so we can modify it.
  int cmdpreview_type = 0;
  char *cmdline = xstrdup(nvim_get_ccline_cmdbuff());
  const char *errormsg = NULL;
  emsg_off++;  // Block errors when parsing the command line, and don't update v:errmsg
  if (!parse_cmdline(&cmdline, &ea, &cmdinfo, &errormsg)) {
    emsg_off--;
    goto end;
  }
  emsg_off--;

  // Check if command is previewable, if not, don't attempt to show preview
  if (!(ea.argt & EX_PREVIEW)) {
    undo_cmdmod(&cmdinfo.cmdmod);
    goto end;
  }

  // Cursor may be at the end of the message grid rather than at cmdspos.
  // Place it there in case preview callback flushes it. #30696
  cursorcmd();
  // Flush now: external cmdline may itself wish to update the screen which is
  // currently disallowed during cmdpreview (no longer needed in case that changes).
  cmdline_ui_flush();

  // Swap invalid command range if needed
  if ((ea.argt & EX_RANGE) && ea.line1 > ea.line2) {
    linenr_T lnum = ea.line1;
    ea.line1 = ea.line2;
    ea.line2 = lnum;
  }

  bool icm_split = *p_icm == 's';  // inccommand=split
  buf_T *cmdpreview_buf = NULL;
  win_T *cmdpreview_win = NULL;

  emsg_silent++;                 // Block error reporting as the command may be incomplete,
                                 // but still update v:errmsg
  msg_silent++;                  // Block messages, namely ones that prompt
  block_autocmds();              // Block events

  // Save current state and prepare for command preview. Implemented in Rust.
  void *cpstate = rs_cmdpreview_prepare();

  // Open preview buffer if inccommand=split.
  if (icm_split && (cmdpreview_buf = cmdpreview_open_buf()) == NULL) {
    // Failed to create preview buffer, so disable preview.
    set_option_direct(kOptInccommand, STATIC_CSTR_AS_OPTVAL("nosplit"), 0, SID_NONE);
    icm_split = false;
  }
  // Setup preview namespace if it's not already set.
  if (!cmdpreview_ns) {
    cmdpreview_ns = (int)nvim_create_namespace((String)STRING_INIT);
  }

  // Set cmdpreview state.
  cmdpreview = true;

  // Execute the preview callback and use its return value to determine whether to show preview or
  // open the preview window. The preview callback also handles doing the changes and highlights for
  // the preview.
  Error err = ERROR_INIT;
  TRY_WRAP(&err, {
    cmdpreview_type = execute_cmd(&ea, &cmdinfo, true);
  });
  if (ERROR_SET(&err)) {
    api_clear_error(&err);
    cmdpreview_type = 0;
  }

  // If inccommand=split and preview callback returns 2, open preview window.
  if (icm_split && cmdpreview_type == 2
      && (cmdpreview_win = cmdpreview_open_win(cmdpreview_buf)) == NULL) {
    // If there's not enough room to open the preview window, just preview without the window.
    cmdpreview_type = 1;
  }

  // If preview callback return value is nonzero, update screen now.
  if (cmdpreview_type != 0) {
    int save_rd = RedrawingDisabled;
    RedrawingDisabled = 0;
    update_screen();
    RedrawingDisabled = save_rd;
  }

  // Close preview window if it's open.
  if (icm_split && cmdpreview_type == 2 && cmdpreview_win != NULL) {
    cmdpreview_close_win();
  }

  // Restore state. Implemented in Rust.
  rs_cmdpreview_restore_state(cpstate);

  unblock_autocmds();                  // Unblock events
  msg_silent--;                        // Unblock messages
  emsg_silent--;                       // Unblock error reporting
  redrawcmdline();
end:
  xfree(cmdline);
  return cmdpreview_type != 0;
}
