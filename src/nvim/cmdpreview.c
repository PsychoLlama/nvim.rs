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

typedef struct {
  u_header_T *save_b_u_oldhead;
  u_header_T *save_b_u_newhead;
  u_header_T *save_b_u_curhead;
  int save_b_u_numhead;
  bool save_b_u_synced;
  int save_b_u_seq_last;
  int save_b_u_save_nr_last;
  int save_b_u_seq_cur;
  time_t save_b_u_time_cur;
  int save_b_u_save_nr_cur;
  char *save_b_u_line_ptr;
  linenr_T save_b_u_line_lnum;
  colnr_T save_b_u_line_colnr;
} CpUndoInfo;

typedef struct {
  buf_T *buf;
  OptInt save_b_p_ul;
  int save_b_p_ma;
  int save_b_changed;
  pos_T save_b_op_start;
  pos_T save_b_op_end;
  varnumber_T save_changedtick;
  CpUndoInfo undo_info;
} CpBufInfo;

typedef struct {
  win_T *win;
  pos_T save_w_cursor;
  viewstate_T save_viewstate;
  int save_w_p_cul;
  int save_w_p_cuc;
} CpWinInfo;

typedef struct {
  kvec_t(CpWinInfo) win_info;
  kvec_t(CpBufInfo) buf_info;
  bool save_hls;
  cmdmod_T save_cmdmod;
  garray_T save_view;
} CpInfo;

#include "cmdpreview.c.generated.h"

// Command preview helpers from Rust
extern int rs_cmdpreview_should_skip_buffer(int64_t buf_handle, int64_t preview_bufnr);
extern int rs_cmdpreview_needs_undo_restore(int64_t current_seq, int64_t saved_seq);
extern void rs_save_viewstate_win(win_T *win, viewstate_T *vs);
extern void rs_restore_viewstate_win(win_T *win, const viewstate_T *vs);
extern void rs_win_size_save(garray_T *gap);
extern void rs_win_size_restore(garray_T *gap);

static handle_T cmdpreview_bufnr = 0;
static int cmdpreview_ns = 0;

// C accessors for static variables (used by Rust)
handle_T nvim_get_cmdpreview_bufnr(void) { return cmdpreview_bufnr; }
void nvim_set_cmdpreview_bufnr(handle_T val) { cmdpreview_bufnr = val; }
int nvim_get_cmdpreview_ns(void) { return cmdpreview_ns; }
void nvim_set_cmdpreview_ns(int val) { cmdpreview_ns = val; }

/// Sets up command preview buffer.
///
/// @return Pointer to command preview buffer if succeeded, NULL if failed.
static buf_T *cmdpreview_open_buf(void)
{
  buf_T *cmdpreview_buf = cmdpreview_bufnr ? buflist_findnr(cmdpreview_bufnr) : NULL;

  // If preview buffer doesn't exist, open one.
  if (cmdpreview_buf == NULL) {
    Error err = ERROR_INIT;
    handle_T bufnr = nvim_create_buf(false, true, &err);

    if (ERROR_SET(&err)) {
      return NULL;
    }

    cmdpreview_buf = buflist_findnr(bufnr);
  }

  // Preview buffer cannot preview itself!
  if (cmdpreview_buf == curbuf) {
    return NULL;
  }

  // Rename preview buffer.
  aco_save_T aco;
  aucmd_prepbuf(&aco, cmdpreview_buf);
  int retv = rename_buffer("[Preview]");
  aucmd_restbuf(&aco);

  if (retv == FAIL) {
    return NULL;
  }

  // Temporarily switch to preview buffer to set it up for previewing.
  aucmd_prepbuf(&aco, cmdpreview_buf);
  buf_clear();
  curbuf->b_p_ma = true;
  curbuf->b_p_ul = -1;
  curbuf->b_p_tw = 0;  // Reset 'textwidth' (was set by ftplugin)
  aucmd_restbuf(&aco);
  cmdpreview_bufnr = cmdpreview_buf->handle;

  return cmdpreview_buf;
}

/// Open command preview window if it's not already open.
/// Returns to original window after opening command preview window.
///
/// @param cmdpreview_buf Pointer to command preview buffer
///
/// @return Pointer to command preview window if succeeded, NULL if failed.
static win_T *cmdpreview_open_win(buf_T *cmdpreview_buf)
  FUNC_ATTR_NONNULL_ALL
{
  win_T *save_curwin = curwin;

  // Open preview window.
  if (win_split((int)p_cwh, WSP_BOT) == FAIL) {
    return NULL;
  }

  win_T *preview_win = curwin;
  Error err = ERROR_INIT;
  int result = OK;

  // Switch to preview buffer
  TRY_WRAP(&err, {
    result = do_buffer_ext(DOBUF_GOTO, DOBUF_FIRST, FORWARD, cmdpreview_buf->handle, 0);
  });
  if (ERROR_SET(&err) || result == FAIL) {
    api_clear_error(&err);
    return NULL;
  }

  curwin->w_p_cul = false;
  curwin->w_p_cuc = false;
  curwin->w_p_spell = false;
  curwin->w_p_fen = false;

  win_enter(save_curwin, false);
  return preview_win;
}

/// Closes any open command preview windows.
static void cmdpreview_close_win(void)
{
  buf_T *buf = cmdpreview_bufnr ? buflist_findnr(cmdpreview_bufnr) : NULL;
  if (buf != NULL) {
    close_windows(buf, false);
  }
}

/// Save the undo state of a buffer for command preview.
static void cmdpreview_save_undo(CpUndoInfo *cp_undoinfo, buf_T *buf)
  FUNC_ATTR_NONNULL_ALL
{
  cp_undoinfo->save_b_u_synced = buf->b_u_synced;
  cp_undoinfo->save_b_u_oldhead = buf->b_u_oldhead;
  cp_undoinfo->save_b_u_newhead = buf->b_u_newhead;
  cp_undoinfo->save_b_u_curhead = buf->b_u_curhead;
  cp_undoinfo->save_b_u_numhead = buf->b_u_numhead;
  cp_undoinfo->save_b_u_seq_last = buf->b_u_seq_last;
  cp_undoinfo->save_b_u_save_nr_last = buf->b_u_save_nr_last;
  cp_undoinfo->save_b_u_seq_cur = buf->b_u_seq_cur;
  cp_undoinfo->save_b_u_time_cur = buf->b_u_time_cur;
  cp_undoinfo->save_b_u_save_nr_cur = buf->b_u_save_nr_cur;
  cp_undoinfo->save_b_u_line_ptr = buf->b_u_line_ptr;
  cp_undoinfo->save_b_u_line_lnum = buf->b_u_line_lnum;
  cp_undoinfo->save_b_u_line_colnr = buf->b_u_line_colnr;
}

/// Restore the undo state of a buffer for command preview.
static void cmdpreview_restore_undo(const CpUndoInfo *cp_undoinfo, buf_T *buf)
{
  buf->b_u_oldhead = cp_undoinfo->save_b_u_oldhead;
  buf->b_u_newhead = cp_undoinfo->save_b_u_newhead;
  buf->b_u_curhead = cp_undoinfo->save_b_u_curhead;
  buf->b_u_numhead = cp_undoinfo->save_b_u_numhead;
  buf->b_u_seq_last = cp_undoinfo->save_b_u_seq_last;
  buf->b_u_save_nr_last = cp_undoinfo->save_b_u_save_nr_last;
  buf->b_u_seq_cur = cp_undoinfo->save_b_u_seq_cur;
  buf->b_u_time_cur = cp_undoinfo->save_b_u_time_cur;
  buf->b_u_save_nr_cur = cp_undoinfo->save_b_u_save_nr_cur;
  buf->b_u_line_ptr = cp_undoinfo->save_b_u_line_ptr;
  buf->b_u_line_lnum = cp_undoinfo->save_b_u_line_lnum;
  buf->b_u_line_colnr = cp_undoinfo->save_b_u_line_colnr;
  if (buf->b_u_curhead == NULL) {
    buf->b_u_synced = cp_undoinfo->save_b_u_synced;
  }
}

/// Save current state and prepare windows and buffers for command preview.
static void cmdpreview_prepare(CpInfo *cpinfo)
  FUNC_ATTR_NONNULL_ALL
{
  Set(ptr_t) saved_bufs = SET_INIT;

  kv_init(cpinfo->buf_info);
  kv_init(cpinfo->win_info);

  FOR_ALL_WINDOWS_IN_TAB(win, curtab) {
    buf_T *buf = win->w_buffer;

    // Don't save state of command preview buffer or preview window.
    // Use Rust helper to check if buffer should be skipped for preview.
    if (rs_cmdpreview_should_skip_buffer(buf->handle, cmdpreview_bufnr)) {
      continue;
    }

    if (!set_has(ptr_t, &saved_bufs, buf)) {
      CpBufInfo cp_bufinfo;
      cp_bufinfo.buf = buf;
      cp_bufinfo.save_b_p_ma = buf->b_p_ma;
      cp_bufinfo.save_b_p_ul = buf->b_p_ul;
      cp_bufinfo.save_b_changed = buf->b_changed;
      cp_bufinfo.save_b_op_start = buf->b_op_start;
      cp_bufinfo.save_b_op_end = buf->b_op_end;
      cp_bufinfo.save_changedtick = buf_get_changedtick(buf);
      cmdpreview_save_undo(&cp_bufinfo.undo_info, buf);
      kv_push(cpinfo->buf_info, cp_bufinfo);
      set_put(ptr_t, &saved_bufs, buf);

      u_clearall(buf);
      buf->b_p_ul = INT_MAX;  // Make sure we can undo all changes
    }

    CpWinInfo cp_wininfo;
    cp_wininfo.win = win;

    // Save window cursor position and viewstate
    cp_wininfo.save_w_cursor = win->w_cursor;
    rs_save_viewstate_win(win, &cp_wininfo.save_viewstate);

    // Save 'cursorline' and 'cursorcolumn'
    cp_wininfo.save_w_p_cul = win->w_p_cul;
    cp_wininfo.save_w_p_cuc = win->w_p_cuc;

    kv_push(cpinfo->win_info, cp_wininfo);

    win->w_p_cul = false;       // Disable 'cursorline' so it doesn't mess up the highlights
    win->w_p_cuc = false;       // Disable 'cursorcolumn' so it doesn't mess up the highlights
  }

  set_destroy(ptr_t, &saved_bufs);

  cpinfo->save_hls = p_hls;
  cpinfo->save_cmdmod = cmdmod;
  rs_win_size_save(&cpinfo->save_view);
  save_search_patterns();

  p_hls = false;                 // Don't show search highlighting during live substitution
  cmdmod.cmod_split = 0;         // Disable :leftabove/botright modifiers
  cmdmod.cmod_tab = 0;           // Disable :tab modifier
  cmdmod.cmod_flags |= CMOD_NOSWAPFILE;  // Disable swap for preview buffer

  u_sync(true);
}

/// Restore the state of buffers and windows for command preview.
static void cmdpreview_restore_state(CpInfo *cpinfo)
  FUNC_ATTR_NONNULL_ALL
{
  for (size_t i = 0; i < cpinfo->buf_info.size; i++) {
    CpBufInfo cp_bufinfo = cpinfo->buf_info.items[i];
    buf_T *buf = cp_bufinfo.buf;

    buf->b_changed = cp_bufinfo.save_b_changed;

    // Clear preview highlights.
    extmark_clear(buf, (uint32_t)cmdpreview_ns, 0, 0, MAXLNUM, MAXCOL);

    // Use Rust helper to check if undo restoration is needed.
    if (rs_cmdpreview_needs_undo_restore(buf->b_u_seq_cur,
                                         cp_bufinfo.undo_info.save_b_u_seq_cur)) {
      int count = 0;

      // Calculate how many undo steps are necessary to restore earlier state.
      for (u_header_T *uhp = buf->b_u_curhead ? buf->b_u_curhead : buf->b_u_newhead;
           uhp != NULL;
           uhp = uhp->uh_next.ptr, ++count) {}

      aco_save_T aco;
      aucmd_prepbuf(&aco, buf);
      // Ensure all the entries will be undone
      if (curbuf->b_u_synced == false) {
        u_sync(true);
      }
      // Undo invisibly. This also moves the cursor!
      if (!u_undo_and_forget(count, false)) {
        abort();
      }
      aucmd_restbuf(&aco);
    }

    u_blockfree(buf);
    cmdpreview_restore_undo(&cp_bufinfo.undo_info, buf);

    buf->b_op_start = cp_bufinfo.save_b_op_start;
    buf->b_op_end = cp_bufinfo.save_b_op_end;

    if (cp_bufinfo.save_changedtick != buf_get_changedtick(buf)) {
      buf_set_changedtick(buf, cp_bufinfo.save_changedtick);
    }

    buf->b_p_ul = cp_bufinfo.save_b_p_ul;        // Restore 'undolevels'
    buf->b_p_ma = cp_bufinfo.save_b_p_ma;        // Restore 'modifiable'
  }

  for (size_t i = 0; i < cpinfo->win_info.size; i++) {
    CpWinInfo cp_wininfo = cpinfo->win_info.items[i];
    win_T *win = cp_wininfo.win;

    // Restore window cursor position and viewstate
    win->w_cursor = cp_wininfo.save_w_cursor;
    rs_restore_viewstate_win(win, &cp_wininfo.save_viewstate);

    // Restore 'cursorline' and 'cursorcolumn'
    win->w_p_cul = cp_wininfo.save_w_p_cul;
    win->w_p_cuc = cp_wininfo.save_w_p_cuc;

    update_topline(win);
  }

  cmdmod = cpinfo->save_cmdmod;                // Restore cmdmod
  p_hls = cpinfo->save_hls;                    // Restore 'hlsearch'
  restore_search_patterns();           // Restore search patterns
  rs_win_size_restore(&cpinfo->save_view);        // Restore window sizes

  ga_clear(&cpinfo->save_view);
  kv_destroy(cpinfo->win_info);
  kv_destroy(cpinfo->buf_info);
}

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

  CpInfo cpinfo;
  bool icm_split = *p_icm == 's';  // inccommand=split
  buf_T *cmdpreview_buf = NULL;
  win_T *cmdpreview_win = NULL;

  emsg_silent++;                 // Block error reporting as the command may be incomplete,
                                 // but still update v:errmsg
  msg_silent++;                  // Block messages, namely ones that prompt
  block_autocmds();              // Block events

  // Save current state and prepare for command preview.
  cmdpreview_prepare(&cpinfo);

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

  // Restore state.
  cmdpreview_restore_state(&cpinfo);

  unblock_autocmds();                  // Unblock events
  msg_silent--;                        // Unblock messages
  emsg_silent--;                       // Unblock error reporting
  redrawcmdline();
end:
  xfree(cmdline);
  return cmdpreview_type != 0;
}
