//
// buffer.c: functions for dealing with the buffer structure
//

//
// The buffer list is a double linked list of all buffers.
// Each buffer can be in one of these states:
// never loaded: BF_NEVERLOADED is set, only the file name is valid
//   not loaded: b_ml.ml_mfp == NULL, no memfile allocated
//       hidden: b_nwindows == 0, loaded but not displayed in a window
//       normal: loaded and displayed in a window
//
// Instead of storing file names all over the place, each file name is
// stored in the buffer list. It can be referenced by a number.
//
// The current implementation remembers all file names ever used.
//

#include <assert.h>
#include <ctype.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <time.h>

#include "auto/config.h"
#include "klib/kvec.h"
#include "nvim/api/private/helpers.h"
#include "nvim/arglist.h"
#include "nvim/ascii_defs.h"
#include "nvim/assert_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_updates.h"
#include "nvim/change.h"
#include "nvim/channel.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand.h"
#include "nvim/cursor.h"
#include "nvim/diff.h"
#include "nvim/digraph.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds2.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_eval.h"
#include "nvim/ex_eval_defs.h"
#include "nvim/ex_getln.h"
#include "nvim/extmark.h"
#include "nvim/file_search.h"
#include "nvim/fileio.h"
#include "nvim/fold.h"
#include "nvim/fuzzy.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/hashtab.h"
#include "nvim/hashtab_defs.h"
#include "nvim/help.h"
#include "nvim/indent.h"
#include "nvim/indent_c.h"
#include "nvim/insexpand.h"
#include "nvim/main.h"
#include "nvim/map_defs.h"
#include "nvim/mapping.h"
#include "nvim/mark.h"
#include "nvim/mark_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memfile_defs.h"
#include "nvim/memline.h"
#include "nvim/memline_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/optionstr.h"
#include "nvim/os/fs.h"
#include "nvim/os/fs_defs.h"
#include "nvim/os/input.h"
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/os/time.h"
#include "nvim/path.h"
#include "nvim/plines.h"
#include "nvim/pos_defs.h"
#include "nvim/quickfix.h"
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
#include "nvim/runtime.h"
#include "nvim/runtime_defs.h"
#include "nvim/spell.h"
#include "nvim/state_defs.h"
#include "nvim/statusline.h"
#include "nvim/strings.h"
#include "nvim/syntax.h"
#include "nvim/terminal.h"
#include "nvim/ui.h"
#include "nvim/undo.h"
#include "nvim/usercmd.h"
#include "nvim/version.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"
#include "nvim/winfloat.h"

#include "buffer.c.generated.h"
extern bool rs_is_dev_fd_file(const char *fname);
extern int rs_win_valid(win_T *win);
extern int rs_win_valid_any_tab(win_T *win);
extern int rs_one_window_in_tab(win_T *win, tabpage_T *tp);
extern int rs_last_window(win_T *win);

// Rust FFI declarations (window wrappers removed)
extern int rs_get_last_winid(void);
extern int rs_global_stl_height(void);
extern win_T *rs_lastwin_nofloating(void);
extern int rs_tabline_height(void);
extern int rs_tabpage_index(tabpage_T *ftp);
extern int rs_win_locked(win_T *wp);

// Rust fold FFI declarations
extern void rs_clearFolding(win_T *win);
extern void rs_foldUpdateAll(win_T *win);
extern void rs_cloneFoldGrowArray(garray_T *from, garray_T *to);

// Rust implementations
extern int rs_bt_nofileread(buf_T *buf);

extern void rs_diff_buf_delete(buf_T *buf);
extern void rs_diff_buf_add(buf_T *buf);
extern int rs_diffopt_hiddenoff(void);

// File identity helpers from Rust
extern bool rs_otherfile_buf_4(buf_T *buf, char *ffname, void *file_id_p, bool file_id_valid);

extern void rs_reset_VIsual_and_resel(void);
extern buf_T *rs_find_buffer_for_delete(int buf_fnum, int *update_jumplist);
extern buf_T *rs_find_and_validate_buffer(int action, int start, int dir, int count, int flags,
                                          int unload);
extern int rs_buf_effective_action(buf_T *buf, int action);


// Accessor functions for Rust opaque handle pattern are in buffer_shim.c.
// Only accessor functions that reference file-scope static variables remain here.

static const char e_attempt_to_delete_buffer_that_is_in_use_str[]
  = N_("E937: Attempt to delete a buffer that is in use: %s");

// Number of times free_buffer() was called.
static int buf_free_count = 0;

/// Get the buf_free_count global (accessor for Rust).
int nvim_get_buf_free_count(void)
{
  return buf_free_count;
}

static int top_file_num = 1;            ///< highest file number

/// Get the top_file_num global (accessor for Rust).
int nvim_get_top_file_num(void)
{
  return top_file_num;
}


// Static assertions for constants used in Rust (Phase 1).
_Static_assert(ML_EMPTY == 0x01, "ML_EMPTY mismatch with Rust");
_Static_assert(DOBUF_WIPE == 4, "DOBUF_WIPE mismatch with Rust");

// Static assertions for constants used in Rust (Phase 4).
_Static_assert(EVENT_BUFADD == 0, "EVENT_BUFADD mismatch with Rust");
_Static_assert(EVENT_BUFDELETE == 2, "EVENT_BUFDELETE mismatch with Rust");

typedef enum {
  kBffClearWinInfo = 1,
  kBffInitChangedtick = 2,
} BufFreeFlags;


/// Read data from buffer for retrying.
///
/// @param read_stdin  read file from stdin, otherwise fifo
/// @param eap  for forced 'ff' and 'fenc' or NULL
/// @param flags  extra flags for readfile()
static int read_buffer(bool read_stdin, exarg_T *eap, int flags)
{
  int retval = OK;
  bool silent = shortmess(SHM_FILEINFO);

  // Read from the buffer which the text is already filled in and append at
  // the end.  This makes it possible to retry when 'fileformat' or
  // 'fileencoding' was guessed wrong.
  linenr_T line_count = curbuf->b_ml.ml_line_count;
  retval = readfile(read_stdin ? NULL : curbuf->b_ffname,
                    read_stdin ? NULL : curbuf->b_fname,
                    line_count, 0, (linenr_T)MAXLNUM, eap,
                    flags | READ_BUFFER, silent);
  if (retval == OK) {
    // Delete the binary lines.
    while (--line_count >= 0) {
      ml_delete(1);
    }
  } else {
    // Delete the converted lines.
    while (curbuf->b_ml.ml_line_count > line_count) {
      ml_delete(line_count);
    }
  }
  // Put the cursor on the first line.
  curwin->w_cursor.lnum = 1;
  curwin->w_cursor.col = 0;

  if (read_stdin) {
    // Set or reset 'modified' before executing autocommands, so that
    // it can be changed there.
    if (!readonlymode && !buf_is_empty(curbuf)) {
      changed(curbuf);
    } else if (retval != FAIL) {
      unchanged(curbuf, false, true);
    }

    apply_autocmds_retval(EVENT_STDINREADPOST, NULL, NULL, false,
                          curbuf, &retval);
  }
  return retval;
}

/// Ensure buffer "buf" is loaded.
bool buf_ensure_loaded(buf_T *buf)
{
  if (buf->b_ml.ml_mfp != NULL) {
    // already open (common case)
    return true;
  }

  aco_save_T aco;

  // Make sure the buffer is in a window.
  aucmd_prepbuf(&aco, buf);
  // status can be OK or NOTDONE (which also means ok/done)
  int status = open_buffer(false, NULL, 0);
  aucmd_restbuf(&aco);
  return (status != FAIL);
}

/// Open current buffer, that is: open the memfile and read the file into
/// memory.
///
/// @param read_stdin  read file from stdin
/// @param eap  for forced 'ff' and 'fenc' or NULL
/// @param flags_arg  extra flags for readfile()
///
/// @return  FAIL for failure, OK otherwise.
int open_buffer(bool read_stdin, exarg_T *eap, int flags_arg)
{
  int flags = flags_arg;
  int retval = OK;
  bufref_T old_curbuf;
  OptInt old_tw = curbuf->b_p_tw;
  bool read_fifo = false;
  bool silent = shortmess(SHM_FILEINFO);

  // The 'readonly' flag is only set when BF_NEVERLOADED is being reset.
  // When re-entering the same buffer, it should not change, because the
  // user may have reset the flag by hand.
  if (readonlymode && curbuf->b_ffname != NULL
      && (curbuf->b_flags & BF_NEVERLOADED)) {
    curbuf->b_p_ro = true;
  }

  if (ml_open(curbuf) == FAIL) {
    // There MUST be a memfile, otherwise we can't do anything
    // If we can't create one for the current buffer, take another buffer
    close_buffer(NULL, curbuf, 0, false, false);

    curbuf = NULL;
    FOR_ALL_BUFFERS(buf) {
      if (buf->b_ml.ml_mfp != NULL) {
        curbuf = buf;
        break;
      }
    }

    // If there is no memfile at all, exit.
    // This is OK, since there are no changes to lose.
    if (curbuf == NULL) {
      emsg(_("E82: Cannot allocate any buffer, exiting..."));

      // Don't try to do any saving, with "curbuf" NULL almost nothing
      // will work.
      v_dying = 2;
      getout(2);
    }

    emsg(_("E83: Cannot allocate buffer, using other one..."));
    enter_buffer(curbuf);
    if (old_tw != curbuf->b_p_tw) {
      check_colorcolumn(NULL, curwin);
    }
    return FAIL;
  }

  // Do not sync this buffer yet, may first want to read the file.
  if (curbuf->b_ml.ml_mfp != NULL) {
    curbuf->b_ml.ml_mfp->mf_dirty = MF_DIRTY_YES_NOSYNC;
  }

  // The autocommands in readfile() may change the buffer, but only AFTER
  // reading the file.
  set_bufref(&old_curbuf, curbuf);
  curbuf->b_modified_was_set = false;

  // mark cursor position as being invalid
  curwin->w_valid = 0;

  // A buffer without an actual file should not use the buffer name to read a
  // file.
  if (rs_bt_nofileread(curbuf)) {
    flags |= READ_NOFILE;
  }

  // Read the file if there is one.
  if (curbuf->b_ffname != NULL) {
#ifdef UNIX
    int save_bin = curbuf->b_p_bin;
    int perm = os_getperm(curbuf->b_ffname);
    if (perm >= 0 && (0 || S_ISFIFO(perm)
                      || S_ISSOCK(perm)
# ifdef OPEN_CHR_FILES
                      || (S_ISCHR(perm)
                          && rs_is_dev_fd_file(curbuf->b_ffname))
# endif
                      )) {
      read_fifo = true;
    }
    if (read_fifo) {
      curbuf->b_p_bin = true;
    }
#endif

    retval = readfile(curbuf->b_ffname, curbuf->b_fname,
                      0, 0, (linenr_T)MAXLNUM, eap,
                      flags | READ_NEW | (read_fifo ? READ_FIFO : 0), silent);
#ifdef UNIX
    if (read_fifo) {
      curbuf->b_p_bin = save_bin;
      if (retval == OK) {
        // don't add READ_FIFO here, otherwise we won't be able to
        // detect the encoding
        retval = read_buffer(false, eap, flags);
      }
    }
#endif

    // Help buffer: populate *local-additions* in help.txt
    if (bt_help(curbuf)) {
      get_local_additions();
    }
  } else if (read_stdin) {
    int save_bin = curbuf->b_p_bin;

    // First read the text in binary mode into the buffer.
    // Then read from that same buffer and append at the end.  This makes
    // it possible to retry when 'fileformat' or 'fileencoding' was
    // guessed wrong.
    curbuf->b_p_bin = true;
    retval = readfile(NULL, NULL, 0,
                      0, (linenr_T)MAXLNUM, NULL,
                      flags | (READ_NEW + READ_STDIN), silent);
    curbuf->b_p_bin = save_bin;
    if (retval == OK) {
      retval = read_buffer(true, eap, flags);
    }
  }

  // Can now sync this buffer in ml_sync_all().
  if (curbuf->b_ml.ml_mfp != NULL
      && curbuf->b_ml.ml_mfp->mf_dirty == MF_DIRTY_YES_NOSYNC) {
    curbuf->b_ml.ml_mfp->mf_dirty = MF_DIRTY_YES;
  }

  // if first time loading this buffer, init b_chartab[]
  if (curbuf->b_flags & BF_NEVERLOADED) {
    buf_init_chartab(curbuf, false);
    parse_cino(curbuf);
  }

  // Set/reset the Changed flag first, autocmds may change the buffer.
  // Apply the automatic commands, before processing the modelines.
  // So the modelines have priority over autocommands.

  // When reading stdin, the buffer contents always needs writing, so set
  // the changed flag.  Unless in readonly mode: "ls | nvim -R -".
  // When interrupted and 'cpoptions' contains 'i' set changed flag.
  if ((got_int && vim_strchr(p_cpo, CPO_INTMOD) != NULL)
      || curbuf->b_modified_was_set  // autocmd did ":set modified"
      || (aborting() && vim_strchr(p_cpo, CPO_INTMOD) != NULL)) {
    changed(curbuf);
  } else if (retval != FAIL && !read_stdin && !read_fifo) {
    unchanged(curbuf, false, true);
  }
  save_file_ff(curbuf);                 // keep this fileformat

  // Set last_changedtick to avoid triggering a TextChanged autocommand right
  // after it was added.
  curbuf->b_last_changedtick = buf_get_changedtick(curbuf);
  curbuf->b_last_changedtick_i = buf_get_changedtick(curbuf);
  curbuf->b_last_changedtick_pum = buf_get_changedtick(curbuf);

  // require "!" to overwrite the file, because it wasn't read completely
  if (aborting()) {
    curbuf->b_flags |= BF_READERR;
  }

  // Need to update automatic folding.  Do this before the autocommands,
  // they may use the fold info.
  rs_foldUpdateAll(curwin);

  // need to set w_topline, unless some autocommand already did that.
  if (!(curwin->w_valid & VALID_TOPLINE)) {
    curwin->w_topline = 1;
    curwin->w_topfill = 0;
  }
  apply_autocmds_retval(EVENT_BUFENTER, NULL, NULL, false, curbuf, &retval);

  // if (retval != OK) {
  if (retval == FAIL) {
    return retval;
  }

  // The autocommands may have changed the current buffer.  Apply the
  // modelines to the correct buffer, if it still exists and is loaded.
  if (bufref_valid(&old_curbuf) && old_curbuf.br_buf->b_ml.ml_mfp != NULL) {
    aco_save_T aco;

    // Go to the buffer that was opened, make sure it is in a window.
    aucmd_prepbuf(&aco, old_curbuf.br_buf);
    do_modelines(0);
    curbuf->b_flags &= ~(BF_CHECK_RO | BF_NEVERLOADED);

    if ((flags & READ_NOWINENTER) == 0) {
      apply_autocmds_retval(EVENT_BUFWINENTER, NULL, NULL, false, curbuf,
                            &retval);
    }

    // restore curwin/curbuf and a few other things
    aucmd_restbuf(&aco);
  }

  return retval;
}

/// Store "buf" in "bufref" and set the free count.
///
/// @param bufref Reference to be used for the buffer.
/// @param buf    The buffer to reference.
void set_bufref(bufref_T *bufref, buf_T *buf)
{
  bufref->br_buf = buf;
  bufref->br_fnum = buf == NULL ? 0 : buf->b_fnum;
  bufref->br_buf_free_count = buf_free_count;
}


/// Return true when buffer "buf" can be unloaded.
/// Give an error message and return false when the buffer is locked or the
/// screen is being redrawn and the buffer is in a window.
static bool can_unload_buffer(buf_T *buf)
{
  bool can_unload = !buf->b_locked;

  if (can_unload && updating_screen) {
    FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
      if (wp->w_buffer == buf) {
        can_unload = false;
        break;
      }
    }
  }
  if (!can_unload) {
    char *fname = buf->b_fname != NULL ? buf->b_fname : buf->b_ffname;
    semsg(_(e_attempt_to_delete_buffer_that_is_in_use_str),
          fname != NULL ? fname : "[No Name]");
  }
  return can_unload;
}

/// Close the link to a buffer.
///
/// @param win    If not NULL, set b_last_cursor.
/// @param buf
/// @param action Used when there is no longer a window for the buffer.
///               Possible values:
///                 0            buffer becomes hidden
///                 DOBUF_UNLOAD buffer is unloaded
///                 DOBUF_DEL    buffer is unloaded and removed from buffer list
///                 DOBUF_WIPE   buffer is unloaded and really deleted
///               When doing all but the first one on the current buffer, the
///               caller should get a new buffer very soon!
///               The 'bufhidden' option can force freeing and deleting.
/// @param abort_if_last
///               If true, do not close the buffer if autocommands cause
///               there to be only one window with this buffer. e.g. when
///               ":quit" is supposed to close the window but autocommands
///               close all other windows.
/// @param ignore_abort
///               If true, don't abort even when aborting() returns true.
/// @return  true if b_nwindows was decremented directly by this call (e.g: not via autocmds).
bool close_buffer(win_T *win, buf_T *buf, int action, bool abort_if_last, bool ignore_abort)
{
  // Adjust action for 'bufhidden' and terminal: migrated to Rust.
  action = rs_buf_effective_action(buf, action);
  bool unload_buf = (action != 0);
  bool del_buf = (action == DOBUF_DEL || action == DOBUF_WIPE);
  bool wipe_buf = (action == DOBUF_WIPE);

  bool is_curwin = (curwin != NULL && curwin->w_buffer == buf);
  win_T *the_curwin = curwin;
  tabpage_T *the_curtab = curtab;

  // Disallow deleting the buffer when it is locked (already being closed or
  // halfway a command that relies on it). Unloading is allowed.
  if ((del_buf || wipe_buf) && !can_unload_buffer(buf)) {
    return false;
  }

  // check no autocommands closed the window
  if (win != NULL  // Avoid bogus clang warning.
      && rs_win_valid_any_tab(win)) {
    // Set b_last_cursor when closing the last window for the buffer.
    // Remember the last cursor position and window options of the buffer.
    // This used to be only for the current window, but then options like
    // 'foldmethod' may be lost with a ":only" command.
    if (buf->b_nwindows == 1) {
      set_last_cursor(win);
    }
    buflist_setfpos(buf, win,
                    win->w_cursor.lnum == 1 ? 0 : win->w_cursor.lnum,
                    win->w_cursor.col, true);
  }

  bufref_T bufref;
  set_bufref(&bufref, buf);

  // When the buffer is no longer in a window, trigger BufWinLeave
  if (buf->b_nwindows == 1) {
    buf->b_locked++;
    buf->b_locked_split++;
    if (apply_autocmds(EVENT_BUFWINLEAVE, buf->b_fname, buf->b_fname, false,
                       buf) && !bufref_valid(&bufref)) {
      // Autocommands deleted the buffer.
      emsg(_(e_auabort));
      return false;
    }
    buf->b_locked--;
    buf->b_locked_split--;
    if (abort_if_last && win != NULL && rs_one_window_in_tab(win, NULL)) {
      // Autocommands made this the only window.
      emsg(_(e_auabort));
      return false;
    }

    // When the buffer becomes hidden, but is not unloaded, trigger
    // BufHidden
    if (!unload_buf) {
      buf->b_locked++;
      buf->b_locked_split++;
      if (apply_autocmds(EVENT_BUFHIDDEN, buf->b_fname, buf->b_fname, false,
                         buf) && !bufref_valid(&bufref)) {
        // Autocommands deleted the buffer.
        emsg(_(e_auabort));
        return false;
      }
      buf->b_locked--;
      buf->b_locked_split--;
      if (abort_if_last && win != NULL && rs_one_window_in_tab(win, NULL)) {
        // Autocommands made this the only window.
        emsg(_(e_auabort));
        return false;
      }
    }
    // autocmds may abort script processing
    if (!ignore_abort && aborting()) {
      return false;
    }
  }

  // If the buffer was in curwin and the window has changed, go back to that
  // window, if it still exists.  This avoids that ":edit x" triggering a
  // "tabnext" BufUnload autocmd leaves a window behind without a buffer.
  if (is_curwin && curwin != the_curwin && rs_win_valid_any_tab(the_curwin)) {
    block_autocmds();
    goto_tabpage_win(the_curtab, the_curwin);
    unblock_autocmds();
  }

  int nwindows = buf->b_nwindows;

  // decrease the link count from windows (unless not in any window)
  if (buf->b_nwindows > 0) {
    buf->b_nwindows--;
  }

  if (rs_diffopt_hiddenoff() && !unload_buf && buf->b_nwindows == 0) {
    rs_diff_buf_delete(buf);   // Clear 'diff' for hidden buffer.
  }

  // Return when a window is displaying the buffer or when it's not
  // unloaded.
  if (buf->b_nwindows > 0 || !unload_buf) {
    return true;
  }

  if (buf->terminal) {
    buf->b_locked++;
    terminal_close(&buf->terminal, -1);
    buf->b_locked--;
  }

  // Always remove the buffer when there is no file name.
  if (buf->b_ffname == NULL) {
    del_buf = true;
  }

  // Free all things allocated for this buffer.
  // Also calls the "BufDelete" autocommands when del_buf is true.
  // Remember if we are closing the current buffer.  Restore the number of
  // windows, so that autocommands in buf_freeall() don't get confused.
  bool is_curbuf = (buf == curbuf);

  // When closing the current buffer stop Visual mode before freeing
  // anything.
  if (is_curbuf && VIsual_active
#if defined(EXITFREE)
      && !entered_free_all_mem
#endif
      ) {
    end_visual_mode();
  }

  buf->b_nwindows = nwindows;

  buf_freeall(buf, ((del_buf ? BFA_DEL : 0)
                    + (wipe_buf ? BFA_WIPE : 0)
                    + (ignore_abort ? BFA_IGNORE_ABORT : 0)));

  if (!bufref_valid(&bufref)) {
    // Autocommands may have deleted the buffer.
    return false;
  }
  // autocmds may abort script processing.
  if (!ignore_abort && aborting()) {
    return false;
  }

  // It's possible that autocommands change curbuf to the one being deleted.
  // This might cause the previous curbuf to be deleted unexpectedly.  But
  // in some cases it's OK to delete the curbuf, because a new one is
  // obtained anyway.  Therefore only return if curbuf changed to the
  // deleted buffer.
  if (buf == curbuf && !is_curbuf) {
    return false;
  }

  bool clear_w_buf = false;
  if (win != NULL  // Avoid bogus clang warning.
      && rs_win_valid_any_tab(win)
      && win->w_buffer == buf) {
    // Defer clearing w_buffer until after operations that may invoke dict
    // watchers (e.g., buf_clear_file()), so callers like tabpagebuflist()
    // never see a window in the winlist with a NULL buffer.
    clear_w_buf = true;
  }

  // Autocommands may have opened or closed windows for this buffer.
  // Decrement the count for the close we do here.
  if (buf->b_nwindows > 0) {
    buf->b_nwindows--;
  }

  // Remove the buffer from the list.
  if (wipe_buf) {
    if (clear_w_buf) {
      win->w_buffer = NULL;
    }
    // Do not wipe out the buffer if it is used in a window.
    if (buf->b_nwindows > 0) {
      return true;
    }
    FOR_ALL_TAB_WINDOWS(tp, wp) {
      mark_forget_file(wp, buf->b_fnum);
    }
    if (buf->b_sfname != buf->b_ffname) {
      XFREE_CLEAR(buf->b_sfname);
    } else {
      buf->b_sfname = NULL;
    }
    XFREE_CLEAR(buf->b_ffname);
    if (buf->b_prev == NULL) {
      firstbuf = buf->b_next;
    } else {
      buf->b_prev->b_next = buf->b_next;
    }
    if (buf->b_next == NULL) {
      lastbuf = buf->b_prev;
    } else {
      buf->b_next->b_prev = buf->b_prev;
    }
    free_buffer(buf);
  } else {
    if (del_buf) {
      // Free all internal variables and reset option values, to make
      // ":bdel" compatible with Vim 5.7.
      free_buffer_stuff(buf, kBffClearWinInfo | kBffInitChangedtick);

      // Make it look like a new buffer.
      buf->b_flags = BF_CHECK_RO | BF_NEVERLOADED;

      // Init the options when loaded again.
      buf->b_p_initialized = false;
    }
    buf_clear_file(buf);
    if (clear_w_buf) {
      win->w_buffer = NULL;
    }
    if (del_buf) {
      buf->b_p_bl = false;
    }
  }
  // NOTE: at this point "curbuf" may be invalid!
  return true;
}


/// Clears the current buffer contents.
void buf_clear(void)
{
  linenr_T line_count = curbuf->b_ml.ml_line_count;
  extmark_free_all(curbuf);   // delete any extmarks
  while (!(curbuf->b_ml.ml_flags & ML_EMPTY)) {
    ml_delete(1);
  }
  deleted_lines_mark(1, line_count);  // prepare for display
}

/// buf_freeall() - free all things allocated for a buffer that are related to
/// the file.  Careful: get here with "curwin" NULL when exiting.
///
/// @param flags BFA_DEL           buffer is going to be deleted
///              BFA_WIPE          buffer is going to be wiped out
///              BFA_KEEP_UNDO     do not free undo information
///              BFA_IGNORE_ABORT  don't abort even when aborting() returns true
void buf_freeall(buf_T *buf, int flags)
{
  bool is_curbuf = (buf == curbuf);
  int is_curwin = (curwin != NULL && curwin->w_buffer == buf);
  win_T *the_curwin = curwin;
  tabpage_T *the_curtab = curtab;

  // Make sure the buffer isn't closed by autocommands.
  buf->b_locked++;
  buf->b_locked_split++;

  bufref_T bufref;
  set_bufref(&bufref, buf);

  buf_updates_unload(buf, false);
  if (!bufref_valid(&bufref)) {
    // on_detach callback deleted the buffer.
    return;
  }
  if ((buf->b_ml.ml_mfp != NULL)
      && apply_autocmds(EVENT_BUFUNLOAD, buf->b_fname, buf->b_fname, false, buf)
      && !bufref_valid(&bufref)) {
    // Autocommands deleted the buffer.
    return;
  }
  if ((flags & BFA_DEL)
      && buf->b_p_bl
      && apply_autocmds(EVENT_BUFDELETE, buf->b_fname, buf->b_fname, false, buf)
      && !bufref_valid(&bufref)) {
    // Autocommands may delete the buffer.
    return;
  }
  if ((flags & BFA_WIPE)
      && apply_autocmds(EVENT_BUFWIPEOUT, buf->b_fname, buf->b_fname, false,
                        buf)
      && !bufref_valid(&bufref)) {
    // Autocommands may delete the buffer.
    return;
  }
  buf->b_locked--;
  buf->b_locked_split--;

  // If the buffer was in curwin and the window has changed, go back to that
  // window, if it still exists.  This avoids that ":edit x" triggering a
  // "tabnext" BufUnload autocmd leaves a window behind without a buffer.
  if (is_curwin && curwin != the_curwin && rs_win_valid_any_tab(the_curwin)) {
    block_autocmds();
    goto_tabpage_win(the_curtab, the_curwin);
    unblock_autocmds();
  }
  // autocmds may abort script processing
  if ((flags & BFA_IGNORE_ABORT) == 0 && aborting()) {
    return;
  }

  // It's possible that autocommands change curbuf to the one being deleted.
  // This might cause curbuf to be deleted unexpectedly.  But in some cases
  // it's OK to delete the curbuf, because a new one is obtained anyway.
  // Therefore only return if curbuf changed to the deleted buffer.
  if (buf == curbuf && !is_curbuf) {
    return;
  }
  rs_diff_buf_delete(buf);             // Can't use 'diff' for unloaded buffer.
  // Remove any ownsyntax, unless exiting.
  if (curwin != NULL && curwin->w_buffer == buf) {
    reset_synblock(curwin);
  }

  // No folds in an empty buffer.
  FOR_ALL_TAB_WINDOWS(tp, win) {
    if (win->w_buffer == buf) {
      rs_clearFolding(win);
    }
  }

  ml_close(buf, true);              // close and delete the memline/memfile
  buf->b_ml.ml_line_count = 0;      // no lines in buffer
  if ((flags & BFA_KEEP_UNDO) == 0) {
    // free the memory allocated for undo
    // and reset all undo information
    u_clearallandblockfree(buf);
  }
  syntax_clear(&buf->b_s);          // reset syntax info
  buf->b_flags &= ~BF_READERR;      // a read error is no longer relevant
}

/// Free a buffer structure and the things it contains related to the buffer
/// itself (not the file, that must have been done already).
static void free_buffer(buf_T *buf)
{
  pmap_del(int)(&buffer_handles, buf->b_fnum, NULL);
  buf_free_count++;
  // b:changedtick uses an item in buf_T.
  free_buffer_stuff(buf, kBffClearWinInfo);
  if (buf->b_vars->dv_refcount > DO_NOT_FREE_CNT) {
    tv_dict_add(buf->b_vars,
                tv_dict_item_copy((dictitem_T *)(&buf->changedtick_di)));
  }
  unref_var_dict(buf->b_vars);
  aubuflocal_remove(buf);
  xfree(buf->additional_data);
  xfree(buf->b_prompt_text);
  kv_destroy(buf->b_wininfo);
  callback_free(&buf->b_prompt_callback);
  callback_free(&buf->b_prompt_interrupt);
  clear_fmark(&buf->b_last_cursor, 0);
  clear_fmark(&buf->b_last_insert, 0);
  clear_fmark(&buf->b_last_change, 0);
  clear_fmark(&buf->b_prompt_start, 0);
  for (size_t i = 0; i < NMARKS; i++) {
    free_fmark(buf->b_namedm[i]);
  }
  for (int i = 0; i < buf->b_changelistlen; i++) {
    free_fmark(buf->b_changelist[i]);
  }
  if (autocmd_busy) {
    // Do not free the buffer structure while autocommands are executing,
    // it's still needed. Free it when autocmd_busy is reset.
    CLEAR_FIELD(buf->b_namedm);
    CLEAR_FIELD(buf->b_changelist);
    buf->b_next = au_pending_free_buf;
    au_pending_free_buf = buf;
  } else {
    xfree(buf);
  }
}

/// Free the b_wininfo list for buffer "buf".
static void clear_wininfo(buf_T *buf)
{
  for (size_t i = 0; i < kv_size(buf->b_wininfo); i++) {
    free_wininfo(kv_A(buf->b_wininfo, i), buf);
  }
  kv_size(buf->b_wininfo) = 0;
}

/// Free stuff in the buffer for ":bdel" and when wiping out the buffer.
///
/// @param buf  Buffer pointer
/// @param free_flags  BufFreeFlags
static void free_buffer_stuff(buf_T *buf, int free_flags)
{
  if (free_flags & kBffClearWinInfo) {
    clear_wininfo(buf);                 // including window-local options
    free_buf_options(buf, true);
    ga_clear(&buf->b_s.b_langp);
  }
  {
    // Avoid losing b:changedtick when deleting buffer: clearing variables
    // implies using clear_tv() on b:changedtick and that sets changedtick to
    // zero.
    hashitem_T *const changedtick_hi = hash_find(&buf->b_vars->dv_hashtab, "changedtick");
    assert(changedtick_hi != NULL);
    hash_remove(&buf->b_vars->dv_hashtab, changedtick_hi);
  }
  vars_clear(&buf->b_vars->dv_hashtab);   // free all internal variables
  hash_init(&buf->b_vars->dv_hashtab);
  if (free_flags & kBffInitChangedtick) {
    buf_init_changedtick(buf);
  }
  uc_clear(&buf->b_ucmds);               // clear local user commands
  extmark_free_all(buf);                 // delete any extmarks
  map_clear_mode(buf, MAP_ALL_MODES, true, false);  // clear local mappings
  map_clear_mode(buf, MAP_ALL_MODES, true, true);   // clear local abbrevs
  XFREE_CLEAR(buf->b_start_fenc);

  buf_free_callbacks(buf);
}

/// Go to another buffer.  Handles the result of the ATTENTION dialog.
void goto_buffer(exarg_T *eap, int start, int dir, int count)
{
  const int save_sea = swap_exists_action;
  bool skip_help_buf;

  switch (eap->cmdidx) {
  case CMD_bnext:
  case CMD_sbnext:
  case CMD_bNext:
  case CMD_bprevious:
  case CMD_sbNext:
  case CMD_sbprevious:
    skip_help_buf = true;
    break;
  default:
    skip_help_buf = false;
    break;
  }

  bufref_T old_curbuf;
  set_bufref(&old_curbuf, curbuf);

  if (swap_exists_action == SEA_NONE) {
    swap_exists_action = SEA_DIALOG;
  }
  (void)do_buffer_ext(*eap->cmd == 's' ? DOBUF_SPLIT : DOBUF_GOTO, start, dir, count,
                      (eap->forceit ? DOBUF_FORCEIT : 0) |
                      (skip_help_buf ? DOBUF_SKIPHELP : 0));

  if (swap_exists_action == SEA_QUIT && *eap->cmd == 's') {
    cleanup_T cs;

    // Reset the error/interrupt/exception state here so that
    // aborting() returns false when closing a window.
    enter_cleanup(&cs);

    // Quitting means closing the split window, nothing else.
    win_close(curwin, true, false);
    swap_exists_action = save_sea;
    swap_exists_did_quit = true;

    // Restore the error/interrupt/exception state if not discarded by a
    // new aborting error, interrupt, or uncaught exception.
    leave_cleanup(&cs);
  } else {
    handle_swap_exists(&old_curbuf);
  }
}

/// Handle the situation of swap_exists_action being set.
///
/// It is allowed for "old_curbuf" to be NULL or invalid.
///
/// @param old_curbuf The buffer to check for.
void handle_swap_exists(bufref_T *old_curbuf)
{
  cleanup_T cs;
  OptInt old_tw = curbuf->b_p_tw;
  buf_T *buf;

  if (swap_exists_action == SEA_QUIT) {
    // Reset the error/interrupt/exception state here so that
    // aborting() returns false when closing a buffer.
    enter_cleanup(&cs);

    // User selected Quit at ATTENTION prompt.  Go back to previous
    // buffer.  If that buffer is gone or the same as the current one,
    // open a new, empty buffer.
    swap_exists_action = SEA_NONE;      // don't want it again
    swap_exists_did_quit = true;
    close_buffer(curwin, curbuf, DOBUF_UNLOAD, false, false);
    if (old_curbuf == NULL
        || !bufref_valid(old_curbuf)
        || old_curbuf->br_buf == curbuf) {
      // Block autocommands here because curwin->w_buffer is NULL.
      block_autocmds();
      buf = buflist_new(NULL, NULL, 1, BLN_CURBUF | BLN_LISTED);
      unblock_autocmds();
    } else {
      buf = old_curbuf->br_buf;
    }
    if (buf != NULL) {
      enter_buffer(buf);

      if (old_tw != curbuf->b_p_tw) {
        check_colorcolumn(NULL, curwin);
      }
    }
    // If "old_curbuf" is NULL we are in big trouble here...

    // Restore the error/interrupt/exception state if not discarded by a
    // new aborting error, interrupt, or uncaught exception.
    leave_cleanup(&cs);
  } else if (swap_exists_action == SEA_RECOVER) {
    // Reset the error/interrupt/exception state here so that
    // aborting() returns false when closing a buffer.
    enter_cleanup(&cs);

    // User selected Recover at ATTENTION prompt.
    msg_scroll = true;
    ml_recover(false);
    msg_puts("\n");     // don't overwrite the last message
    cmdline_row = msg_row;
    do_modelines(0);

    // Restore the error/interrupt/exception state if not discarded by a
    // new aborting error, interrupt, or uncaught exception.
    leave_cleanup(&cs);
  }
  swap_exists_action = SEA_NONE;
}

/// do_bufdel() - delete or unload buffer(s)
///
/// addr_count == 0: ":bdel" - delete current buffer
/// addr_count == 1: ":N bdel" or ":bdel N [N ..]" - first delete
///                  buffer "end_bnr", then any other arguments.
/// addr_count == 2: ":N,N bdel" - delete buffers in range
///
/// command can be DOBUF_UNLOAD (":bunload"), DOBUF_WIPE (":bwipeout") or
/// DOBUF_DEL (":bdel")
///
/// @param arg  pointer to extra arguments
/// @param start_bnr  first buffer number in a range
/// @param end_bnr  buffer nr or last buffer nr in a range
///
/// @return  error message or NULL
char *do_bufdel(int command, char *arg, int addr_count, int start_bnr, int end_bnr, int forceit)
{
  int do_current = 0;             // delete current buffer?
  int deleted = 0;                // number of buffers deleted
  char *errormsg = NULL;          // return value
  int bnr;                        // buffer number

  if (addr_count == 0) {
    do_buffer(command, DOBUF_CURRENT, FORWARD, 0, forceit);
  } else {
    if (addr_count == 2) {
      if (*arg) {               // both range and argument is not allowed
        return ex_errmsg(e_trailing_arg, arg);
      }
      bnr = start_bnr;
    } else {    // addr_count == 1
      bnr = end_bnr;
    }

    for (; !got_int; os_breakcheck()) {
      // delete the current buffer last, otherwise when the
      // current buffer is deleted, the next buffer becomes
      // the current one and will be loaded, which may then
      // also be deleted, etc.
      if (bnr == curbuf->b_fnum) {
        do_current = bnr;
      } else if (do_buffer(command, DOBUF_FIRST, FORWARD, bnr,
                           forceit) == OK) {
        deleted++;
      }

      // find next buffer number to delete/unload
      if (addr_count == 2) {
        if (++bnr > end_bnr) {
          break;
        }
      } else {    // addr_count == 1
        arg = skipwhite(arg);
        if (*arg == NUL) {
          break;
        }
        if (!ascii_isdigit(*arg)) {
          char *p = skiptowhite_esc(arg);
          bnr = buflist_findpat(arg, p, command == DOBUF_WIPE, false, false);
          if (bnr < 0) {                    // failed
            break;
          }
          arg = p;
        } else {
          bnr = getdigits_int(&arg, false, 0);
        }
      }
    }
    if (!got_int && do_current
        && do_buffer(command, DOBUF_FIRST,
                     FORWARD, do_current, forceit) == OK) {
      deleted++;
    }

    if (deleted == 0) {
      if (command == DOBUF_UNLOAD) {
        xstrlcpy(IObuff, _("E515: No buffers were unloaded"), IOSIZE);
      } else if (command == DOBUF_DEL) {
        xstrlcpy(IObuff, _("E516: No buffers were deleted"), IOSIZE);
      } else {
        xstrlcpy(IObuff, _("E517: No buffers were wiped out"), IOSIZE);
      }
      errormsg = IObuff;
    } else if (deleted >= p_report) {
      if (command == DOBUF_UNLOAD) {
        smsg(0, NGETTEXT("%d buffer unloaded", "%d buffers unloaded", deleted),
             deleted);
      } else if (command == DOBUF_DEL) {
        smsg(0, NGETTEXT("%d buffer deleted", "%d buffers deleted", deleted),
             deleted);
      } else {
        smsg(0, NGETTEXT("%d buffer wiped out", "%d buffers wiped out", deleted),
             deleted);
      }
    }
  }

  return errormsg;
}

/// Make the current buffer empty.
/// Used when it is wiped out and it's the last buffer.
static int empty_curbuf(bool close_others, int forceit, int action)
{
  buf_T *buf = curbuf;

  if (action == DOBUF_UNLOAD) {
    emsg(_("E90: Cannot unload last buffer"));
    return FAIL;
  }

  bufref_T bufref;
  set_bufref(&bufref, buf);

  if (close_others) {
    bool can_close_all_others = true;
    if (curwin->w_floating) {
      // Closing all other windows with this buffer may leave only floating windows.
      can_close_all_others = false;
      for (win_T *wp = firstwin; !wp->w_floating; wp = wp->w_next) {
        if (wp->w_buffer != curbuf) {
          // Found another non-floating window with a different (probably unlisted) buffer.
          // Closing all other windows with this buffer is fine in this case.
          can_close_all_others = true;
          break;
        }
      }
    }
    // If it is fine to close all other windows with this buffer, keep the current window and
    // close any other windows with this buffer, then make it empty.
    // Otherwise close_windows() will refuse to close the last non-floating window, so allow it
    // to close the current window instead.
    close_windows(buf, can_close_all_others);
  }

  setpcmark();
  int retval = do_ecmd(0, NULL, NULL, NULL, ECMD_ONE, forceit ? ECMD_FORCEIT : 0, curwin);

  // do_ecmd() may create a new buffer, then we have to delete
  // the old one.  But do_ecmd() may have done that already, check
  // if the buffer still exists.
  if (buf != curbuf && bufref_valid(&bufref) && buf->b_nwindows == 0) {
    close_buffer(NULL, buf, action, false, false);
  }

  if (!close_others) {
    need_fileinfo = false;
  }

  return retval;
}

/// Implementation of the commands for the buffer list.
///
/// action == DOBUF_GOTO     go to specified buffer
/// action == DOBUF_SPLIT    split window and go to specified buffer
/// action == DOBUF_UNLOAD   unload specified buffer(s)
/// action == DOBUF_DEL      delete specified buffer(s) from buffer list
/// action == DOBUF_WIPE     delete specified buffer(s) really
///
/// start == DOBUF_CURRENT   go to "count" buffer from current buffer
/// start == DOBUF_FIRST     go to "count" buffer from first buffer
/// start == DOBUF_LAST      go to "count" buffer from last buffer
/// start == DOBUF_MOD       go to "count" modified buffer from current buffer
///
/// @param dir  FORWARD or BACKWARD
/// @param count  buffer number or number of buffers
/// @param flags  see @ref dobuf_flags_value
///
/// @return  FAIL or OK.
static int do_buffer_ext(int action, int start, int dir, int count, int flags)
{
  buf_T *buf;
  buf_T *bp;
  bool update_jumplist = true;
  bool unload = (action == DOBUF_UNLOAD || action == DOBUF_DEL
                 || action == DOBUF_WIPE);

  // Find and validate target buffer (navigation + pre-action checks).
  // Migrated to Rust (src/nvim-rs/buffer/src/lifecycle.rs).
  buf = rs_find_and_validate_buffer(action, start, dir, count, flags, (int)unload);
  if (buf == NULL) {
    return FAIL;
  }

  // delete buffer "buf" from memory and/or the list
  if (unload) {
    bufref_T bufref;
    if (!can_unload_buffer(buf)) {
      return FAIL;
    }
    set_bufref(&bufref, buf);

    // When unloading or deleting a buffer that's already unloaded and
    // unlisted: fail silently.
    if (action != DOBUF_WIPE && buf->b_ml.ml_mfp == NULL && !buf->b_p_bl) {
      return FAIL;
    }

    if ((flags & DOBUF_FORCEIT) == 0 && bufIsChanged(buf)) {
      if ((p_confirm || (cmdmod.cmod_flags & CMOD_CONFIRM)) && p_write) {
        dialog_changed(buf, false);
        if (!bufref_valid(&bufref)) {
          // Autocommand deleted buffer, oops! It's not changed now.
          return FAIL;
        }
        // If it's still changed fail silently, the dialog already
        // mentioned why it fails.
        if (bufIsChanged(buf)) {
          return FAIL;
        }
      } else {
        semsg(_("E89: No write since last change for buffer %" PRId64
                " (add ! to override)"),
              (int64_t)buf->b_fnum);
        return FAIL;
      }
    }

    if (!(flags & DOBUF_FORCEIT) && buf->terminal && terminal_running(buf->terminal)) {
      if (p_confirm || (cmdmod.cmod_flags & CMOD_CONFIRM)) {
        if (!dialog_close_terminal(buf)) {
          return FAIL;
        }
      } else {
        semsg(_("E89: %s will be killed (add ! to override)"), buf->b_fname);
        return FAIL;
      }
    }

    int buf_fnum = buf->b_fnum;

    // When closing the current buffer stop Visual mode.
    if (buf == curbuf && VIsual_active) {
      end_visual_mode();
    }

    // If deleting the last (listed) buffer, make it empty.
    // The last (listed) buffer cannot be unloaded.
    bp = NULL;
    FOR_ALL_BUFFERS(bp2) {
      if (bp2->b_p_bl && bp2 != buf) {
        bp = bp2;
        break;
      }
    }
    if (bp == NULL && buf == curbuf) {
      return empty_curbuf(true, (flags & DOBUF_FORCEIT), action);
    }

    // If the deleted buffer is the current one, close the current window
    // (unless it's the only non-floating window).
    // When the autocommand window is involved win_close() may need to print an error message.
    // Repeat this so long as we end up in a window with this buffer.
    while (buf == curbuf
           && !(rs_win_locked(curwin) || curwin->w_buffer->b_locked > 0)
           && (is_aucmd_win(lastwin) || !rs_last_window(curwin))) {
      if (win_close(curwin, false, false) == FAIL) {
        break;
      }
    }

    // If the buffer to be deleted is not the current one, delete it here.
    if (buf != curbuf) {
      if (jop_flags & kOptJopFlagClean) {
        // Remove the buffer to be deleted from the jump list.
        mark_jumplist_forget_file(curwin, buf_fnum);
      }

      close_windows(buf, false);

      if (buf != curbuf && bufref_valid(&bufref) && buf->b_nwindows <= 0) {
        close_buffer(NULL, buf, action, false, false);
      }
      return OK;
    }

    // Deleting the current buffer: Need to find another buffer to go to.
    // There should be another, otherwise it would have been handled
    // above.  However, autocommands may have deleted all buffers.
    int update_jumplist_int = update_jumplist ? 1 : 0;
    buf = rs_find_buffer_for_delete(buf_fnum, &update_jumplist_int);
    update_jumplist = (update_jumplist_int != 0);
  }

  if (buf == NULL) {
    // Autocommands must have wiped out all other buffers.  Only option
    // now is to make the current buffer empty.
    return empty_curbuf(false, (flags & DOBUF_FORCEIT), action);
  }

  // make "buf" the current buffer
  if (action == DOBUF_SPLIT) {      // split window first
    // If 'switchbuf' is set jump to the window containing "buf".
    if (swbuf_goto_win_with_buf(buf) != NULL) {
      return OK;
    }

    if (win_split(0, 0) == FAIL) {
      return FAIL;
    }
  }

  // go to current buffer - nothing to do
  if (buf == curbuf) {
    return OK;
  }

  // Check if the current buffer may be abandoned.
  if (action == DOBUF_GOTO && !can_abandon(curbuf, (flags & DOBUF_FORCEIT))) {
    if ((p_confirm || (cmdmod.cmod_flags & CMOD_CONFIRM)) && p_write) {
      bufref_T bufref;
      set_bufref(&bufref, buf);
      dialog_changed(curbuf, false);
      if (!bufref_valid(&bufref)) {
        // Autocommand deleted buffer, oops!
        return FAIL;
      }
    }
    if (bufIsChanged(curbuf)) {
      no_write_message();
      return FAIL;
    }
  }

  // Go to the other buffer.
  set_curbuf(buf, action, update_jumplist);

  if (action == DOBUF_SPLIT) {
    RESET_BINDING(curwin);      // reset 'scrollbind' and 'cursorbind'
  }

  if (aborting()) {         // autocmds may abort script processing
    return FAIL;
  }

  return OK;
}

int do_buffer(int action, int start, int dir, int count, int forceit)
{
  return do_buffer_ext(action, start, dir, count, forceit ? DOBUF_FORCEIT : 0);
}

/// Set current buffer to "buf".  Executes autocommands and closes current
/// buffer.
///
/// @param action  tells how to close the current buffer:
///                DOBUF_GOTO       free or hide it
///                DOBUF_SPLIT      nothing
///                DOBUF_UNLOAD     unload it
///                DOBUF_DEL        delete it
///                DOBUF_WIPE       wipe it out
void set_curbuf(buf_T *buf, int action, bool update_jumplist)
{
  buf_T *prevbuf;
  int unload = (action == DOBUF_UNLOAD || action == DOBUF_DEL
                || action == DOBUF_WIPE);
  OptInt old_tw = curbuf->b_p_tw;
  const int last_winid = rs_get_last_winid();

  if (update_jumplist) {
    setpcmark();
  }

  if ((cmdmod.cmod_flags & CMOD_KEEPALT) == 0) {
    curwin->w_alt_fnum = curbuf->b_fnum;     // remember alternate file
  }
  buflist_altfpos(curwin);                       // remember curpos

  // Don't restart Select mode after switching to another buffer.
  VIsual_reselect = false;

  // close_windows() or apply_autocmds() may change curbuf and wipe out "buf"
  prevbuf = curbuf;
  bufref_T newbufref;
  bufref_T prevbufref;
  set_bufref(&prevbufref, prevbuf);
  set_bufref(&newbufref, buf);

  // Autocommands may delete the current buffer and/or the buffer we want to
  // go to.  In those cases don't close the buffer.
  if (!apply_autocmds(EVENT_BUFLEAVE, NULL, NULL, false, curbuf)
      || (bufref_valid(&prevbufref) && bufref_valid(&newbufref)
          && !aborting())) {
    if (prevbuf == curwin->w_buffer) {
      reset_synblock(curwin);
    }
    // autocommands may have opened a new window
    // with prevbuf, grr
    if (unload
        || (last_winid != rs_get_last_winid()
            && strchr("wdu", prevbuf->b_p_bh[0]) != NULL)) {
      close_windows(prevbuf, false);
    }
    if (bufref_valid(&prevbufref) && !aborting()) {
      win_T *previouswin = curwin;

      // Do not sync when in Insert mode and the buffer is open in
      // another window, might be a timer doing something in another
      // window.
      if (prevbuf == curbuf && ((State & MODE_INSERT) == 0 || curbuf->b_nwindows <= 1)) {
        u_sync(false);
      }
      close_buffer(prevbuf == curwin->w_buffer ? curwin : NULL,
                   prevbuf,
                   unload
                   ? action
                   : (action == DOBUF_GOTO && !buf_hide(prevbuf)
                      && !bufIsChanged(prevbuf)) ? DOBUF_UNLOAD : 0,
                   false, false);
      if (curwin != previouswin && rs_win_valid(previouswin)) {
        // autocommands changed curwin, Grr!
        curwin = previouswin;
      }
    }
  }
  // An autocommand may have deleted "buf", already entered it (e.g., when
  // it did ":bunload") or aborted the script processing!
  // If curwin->w_buffer is null, enter_buffer() will make it valid again
  bool valid = buf_valid(buf);
  if ((valid && buf != curbuf && !aborting()) || curwin->w_buffer == NULL) {
    // autocommands changed curbuf and we will move to another
    // buffer soon, so decrement curbuf->b_nwindows
    if (curbuf != NULL && prevbuf != curbuf) {
      curbuf->b_nwindows--;
    }
    // If the buffer is not valid but curwin->w_buffer is NULL we must
    // enter some buffer.  Using the last one is hopefully OK.
    enter_buffer(valid ? buf : lastbuf);
    if (old_tw != curbuf->b_p_tw) {
      check_colorcolumn(NULL, curwin);
    }
  }

  if (bufref_valid(&prevbufref) && prevbuf->terminal != NULL) {
    terminal_check_size(prevbuf->terminal);
  }
}

/// Enter a new current buffer.
/// Old curbuf must have been abandoned already!  This also means "curbuf" may
/// be pointing to freed memory.
static void enter_buffer(buf_T *buf)
{
  // when closing the current buffer stop Visual mode
  if (VIsual_active
#if defined(EXITFREE)
      && !entered_free_all_mem
#endif
      ) {
    end_visual_mode();
  }

  // Get the buffer in the current window.
  curwin->w_buffer = buf;
  curbuf = buf;
  curbuf->b_nwindows++;

  // Copy buffer and window local option values.  Not for a help buffer.
  buf_copy_options(buf, BCO_ENTER | BCO_NOHELP);
  if (!buf->b_help) {
    get_winopts(buf);
  } else {
    // Remove all folds in the window.
    rs_clearFolding(curwin);
  }
  rs_foldUpdateAll(curwin);        // update folds (later).

  if (curwin->w_p_diff) {
    rs_diff_buf_add(curbuf);
  }

  curwin->w_s = &(curbuf->b_s);

  // Cursor on first line by default.
  curwin->w_cursor.lnum = 1;
  curwin->w_cursor.col = 0;
  curwin->w_cursor.coladd = 0;
  curwin->w_set_curswant = true;
  curwin->w_topline_was_set = false;

  // mark cursor position as being invalid
  curwin->w_valid = 0;

  // Make sure the buffer is loaded.
  if (curbuf->b_ml.ml_mfp == NULL) {    // need to load the file
    // If there is no filetype, allow for detecting one.  Esp. useful for
    // ":ball" used in an autocommand.  If there already is a filetype we
    // might prefer to keep it.
    if (*curbuf->b_p_ft == NUL) {
      curbuf->b_did_filetype = false;
    }

    open_buffer(false, NULL, 0);
  } else {
    if (!msg_silent && !shortmess(SHM_FILEINFO)) {
      need_fileinfo = true;             // display file info after redraw
    }
    // check if file changed
    buf_check_timestamp(curbuf);

    curwin->w_topline = 1;
    curwin->w_topfill = 0;
    apply_autocmds(EVENT_BUFENTER, NULL, NULL, false, curbuf);
    apply_autocmds(EVENT_BUFWINENTER, NULL, NULL, false, curbuf);
  }

  // If autocommands did not change the cursor position, restore cursor lnum
  // and possibly cursor col.
  if (curwin->w_cursor.lnum == 1 && inindent(0)) {
    buflist_getfpos();
  }

  check_arg_idx(curwin);                // check for valid arg_idx
  maketitle();
  // when autocmds didn't change it
  if (curwin->w_topline == 1 && !curwin->w_topline_was_set) {
    scroll_cursor_halfway(curwin, false, false);  // redisplay at correct position
  }

  // Change directories when the 'acd' option is set.
  do_autochdir();

  if (curbuf->b_kmap_state & KEYMAP_INIT) {
    keymap_init();
  }
  // May need to set the spell language.  Can only do this after the buffer
  // has been properly setup.
  if (!curbuf->b_help && curwin->w_p_spell && *curwin->w_s->b_p_spl != NUL) {
    parse_spelllang(curwin);
  }
  curbuf->b_last_used = time(NULL);

  if (curbuf->terminal != NULL) {
    terminal_check_size(curbuf->terminal);
  }

  redraw_later(curwin, UPD_NOT_VALID);
}

/// Change to the directory of the current buffer.
/// Don't do this while still starting up.
void do_autochdir(void)
{
  if (p_acd) {
    if (starting == 0
        && curbuf->b_ffname != NULL
        && vim_chdirfile(curbuf->b_ffname, kCdCauseAuto) == OK) {
      last_chdir_reason = "autochdir";
      shorten_fnames(true);
    }
  }
}

void no_write_message(void)
{
  if (curbuf->terminal
      && channel_job_running((uint64_t)curbuf->b_p_channel)) {
    emsg(_("E948: Job still running (add ! to end the job)"));
  } else {
    emsg(_("E37: No write since last change (add ! to override)"));
  }
}

void no_write_message_nobang(const buf_T *const buf)
  FUNC_ATTR_NONNULL_ALL
{
  if (buf->terminal
      && channel_job_running((uint64_t)buf->b_p_channel)) {
    emsg(_("E948: Job still running"));
  } else {
    emsg(_("E37: No write since last change"));
  }
}

//
// functions for dealing with the buffer list
//

/// Initialize b:changedtick and changedtick_val attribute
///
/// @param[out]  buf  Buffer to initialize for.
static inline void buf_init_changedtick(buf_T *const buf)
  FUNC_ATTR_ALWAYS_INLINE FUNC_ATTR_NONNULL_ALL
{
  STATIC_ASSERT(sizeof("changedtick") <= sizeof(buf->changedtick_di.di_key),
                "buf->changedtick_di cannot hold large enough keys");
  buf->changedtick_di = (ChangedtickDictItem) {
    .di_flags = DI_FLAGS_RO|DI_FLAGS_FIX,  // Must not include DI_FLAGS_ALLOC.
    .di_tv = (typval_T) {
      .v_type = VAR_NUMBER,
      .v_lock = VAR_FIXED,
      .vval.v_number = buf_get_changedtick(buf),
    },
    .di_key = "changedtick",
  };
  tv_dict_add(buf->b_vars, (dictitem_T *)&buf->changedtick_di);
}

/// Add a file name to the buffer list.
/// If the same file name already exists return a pointer to that buffer.
/// If it does not exist, or if fname == NULL, a new entry is created.
/// If (flags & BLN_CURBUF) is true, may use current buffer.
/// If (flags & BLN_LISTED) is true, add new buffer to buffer list.
/// If (flags & BLN_DUMMY) is true, don't count it as a real buffer.
/// If (flags & BLN_NEW) is true, don't use an existing buffer.
/// If (flags & BLN_NOOPT) is true, don't copy options from the current buffer
///                                 if the buffer already exists.
/// This is the ONLY way to create a new buffer.
///
/// @param ffname_arg  full path of fname or relative
/// @param sfname_arg  short fname or NULL
/// @param lnum   preferred cursor line
/// @param flags  BLN_ defines
/// @param bufnr
///
/// @return  pointer to the buffer
buf_T *buflist_new(char *ffname_arg, char *sfname_arg, linenr_T lnum, int flags)
{
  char *ffname = ffname_arg;
  char *sfname = sfname_arg;
  buf_T *buf;

  fname_expand(curbuf, &ffname, &sfname);       // will allocate ffname

  // If the file name already exists in the list, update the entry.

  // We can use inode numbers when the file exists.  Works better
  // for hard links.
  FileID file_id;
  bool file_id_valid = (sfname != NULL && os_fileid(sfname, &file_id));
  if (ffname != NULL && !(flags & (BLN_DUMMY | BLN_NEW))
      && (buf = buflist_findname_file_id(ffname, &file_id, file_id_valid)) != NULL) {
    xfree(ffname);
    if (lnum != 0) {
      buflist_setfpos(buf, (flags & BLN_NOCURWIN) ? NULL : curwin,
                      lnum, 0, false);
    }
    if ((flags & BLN_NOOPT) == 0) {
      // Copy the options now, if 'cpo' doesn't have 's' and not done already.
      buf_copy_options(buf, 0);
    }
    if ((flags & BLN_LISTED) && !buf->b_p_bl) {
      buf->b_p_bl = true;
      bufref_T bufref;
      set_bufref(&bufref, buf);
      if (!(flags & BLN_DUMMY)) {
        if (apply_autocmds(EVENT_BUFADD, NULL, NULL, false, buf)
            && !bufref_valid(&bufref)) {
          return NULL;
        }
      }
    }
    return buf;
  }

  // If the current buffer has no name and no contents, use the current
  // buffer.    Otherwise: Need to allocate a new buffer structure.
  //
  // This is the ONLY place where a new buffer structure is allocated!
  // (A spell file buffer is allocated in spell.c, but that's not a normal
  // buffer.)
  buf = NULL;
  if ((flags & BLN_CURBUF) && curbuf_reusable()) {
    bufref_T bufref;

    assert(curbuf != NULL);
    buf = curbuf;
    set_bufref(&bufref, buf);
    // It's like this buffer is deleted.  Watch out for autocommands that
    // change curbuf!  If that happens, allocate a new buffer anyway.
    buf_freeall(buf, BFA_WIPE | BFA_DEL);
    if (aborting()) {           // autocmds may abort script processing
      xfree(ffname);
      return NULL;
    }
    if (!bufref_valid(&bufref)) {
      buf = NULL;  // buf was deleted; allocate a new buffer
    }
  }
  if (buf != curbuf || curbuf == NULL) {
    buf = xcalloc(1, sizeof(buf_T));
    // init b: variables
    buf->b_vars = tv_dict_alloc();
    init_var_dict(buf->b_vars, &buf->b_bufvar, VAR_SCOPE);
    buf_init_changedtick(buf);
  }

  if (ffname != NULL) {
    buf->b_ffname = ffname;
    buf->b_sfname = xstrdup(sfname);
  }

  clear_wininfo(buf);
  WinInfo *curwin_info = xcalloc(1, sizeof(WinInfo));
  kv_push(buf->b_wininfo, curwin_info);

  if (buf == curbuf) {
    free_buffer_stuff(buf, kBffInitChangedtick);  // delete local vars et al.

    // Init the options.
    buf->b_p_initialized = false;
    buf_copy_options(buf, BCO_ENTER);

    // need to reload lmaps and set b:keymap_name
    curbuf->b_kmap_state |= KEYMAP_INIT;
  } else {
    // put new buffer at the end of the buffer list
    buf->b_next = NULL;
    if (firstbuf == NULL) {             // buffer list is empty
      buf->b_prev = NULL;
      firstbuf = buf;
    } else {                            // append new buffer at end of list
      lastbuf->b_next = buf;
      buf->b_prev = lastbuf;
    }
    lastbuf = buf;

    buf->b_fnum = top_file_num++;
    pmap_put(int)(&buffer_handles, buf->b_fnum, buf);
    if (top_file_num < 0) {  // wrap around (may cause duplicates)
      emsg(_("W14: Warning: List of file names overflow"));
      if (emsg_silent == 0 && !in_assert_fails && !ui_has(kUIMessages)) {
        ui_flush();
        os_delay(3001, true);  // make sure it is noticed
      }
      top_file_num = 1;
    }

    // Always copy the options from the current buffer.
    buf_copy_options(buf, BCO_ALWAYS);
  }

  curwin_info->wi_mark = (fmark_T)INIT_FMARK;
  curwin_info->wi_mark.mark.lnum = lnum;
  curwin_info->wi_win = curwin;

  hash_init(&buf->b_s.b_keywtab);
  hash_init(&buf->b_s.b_keywtab_ic);

  buf->b_fname = buf->b_sfname;
  if (!file_id_valid) {
    buf->file_id_valid = false;
  } else {
    buf->file_id_valid = true;
    buf->file_id = file_id;
  }
  buf->b_u_synced = true;
  buf->b_flags = BF_CHECK_RO | BF_NEVERLOADED;
  if (flags & BLN_DUMMY) {
    buf->b_flags |= BF_DUMMY;
  }
  buf_clear_file(buf);
  clrallmarks(buf, 0);                  // clear marks
  fmarks_check_names(buf);              // check file marks for this file
  buf->b_p_bl = (flags & BLN_LISTED) ? true : false;    // init 'buflisted'
  kv_destroy(buf->update_channels);
  kv_init(buf->update_channels);
  kv_destroy(buf->update_callbacks);
  kv_init(buf->update_callbacks);
  if (!(flags & BLN_DUMMY)) {
    // Tricky: these autocommands may change the buffer list.  They could also
    // split the window with re-using the one empty buffer. This may result in
    // unexpectedly losing the empty buffer.
    bufref_T bufref;
    set_bufref(&bufref, buf);
    if (apply_autocmds(EVENT_BUFNEW, NULL, NULL, false, buf)
        && !bufref_valid(&bufref)) {
      return NULL;
    }
    if ((flags & BLN_LISTED)
        && apply_autocmds(EVENT_BUFADD, NULL, NULL, false, buf)
        && !bufref_valid(&bufref)) {
      return NULL;
    }
    if (aborting()) {
      // Autocmds may abort script processing.
      return NULL;
    }
  }

  buf->b_prompt_callback.type = kCallbackNone;
  buf->b_prompt_interrupt.type = kCallbackNone;
  buf->b_prompt_text = NULL;
  clear_fmark(&buf->b_prompt_start, 0);

  return buf;
}


/// Free the memory for the options of a buffer.
/// If "free_p_ff" is true also free 'fileformat', 'buftype' and
/// 'fileencoding'.
void free_buf_options(buf_T *buf, bool free_p_ff)
{
  if (free_p_ff) {
    clear_string_option(&buf->b_p_fenc);
    clear_string_option(&buf->b_p_ff);
    clear_string_option(&buf->b_p_bh);
    clear_string_option(&buf->b_p_bt);
  }
  clear_string_option(&buf->b_p_def);
  clear_string_option(&buf->b_p_inc);
  clear_string_option(&buf->b_p_inex);
  clear_string_option(&buf->b_p_inde);
  clear_string_option(&buf->b_p_indk);
  clear_string_option(&buf->b_p_fp);
  clear_string_option(&buf->b_p_fex);
  clear_string_option(&buf->b_p_kp);
  clear_string_option(&buf->b_p_mps);
  clear_string_option(&buf->b_p_fo);
  clear_string_option(&buf->b_p_flp);
  clear_string_option(&buf->b_p_isk);
  clear_string_option(&buf->b_p_vsts);
  XFREE_CLEAR(buf->b_p_vsts_nopaste);
  XFREE_CLEAR(buf->b_p_vsts_array);
  clear_string_option(&buf->b_p_vts);
  XFREE_CLEAR(buf->b_p_vts_array);
  clear_string_option(&buf->b_p_keymap);
  keymap_ga_clear(&buf->b_kmap_ga);
  ga_clear(&buf->b_kmap_ga);
  clear_string_option(&buf->b_p_com);
  clear_string_option(&buf->b_p_cms);
  clear_string_option(&buf->b_p_nf);
  clear_string_option(&buf->b_p_syn);
  clear_string_option(&buf->b_s.b_syn_isk);
  clear_string_option(&buf->b_s.b_p_spc);
  clear_string_option(&buf->b_s.b_p_spf);
  vim_regfree(buf->b_s.b_cap_prog);
  buf->b_s.b_cap_prog = NULL;
  clear_string_option(&buf->b_s.b_p_spl);
  clear_string_option(&buf->b_s.b_p_spo);
  clear_string_option(&buf->b_p_sua);
  clear_string_option(&buf->b_p_ft);
  clear_string_option(&buf->b_p_cink);
  clear_string_option(&buf->b_p_cino);
  clear_string_option(&buf->b_p_lop);
  clear_string_option(&buf->b_p_cinsd);
  clear_string_option(&buf->b_p_cinw);
  clear_string_option(&buf->b_p_cot);
  clear_string_option(&buf->b_p_cpt);
  clear_string_option(&buf->b_p_cfu);
  callback_free(&buf->b_cfu_cb);
  clear_string_option(&buf->b_p_ofu);
  callback_free(&buf->b_ofu_cb);
  clear_string_option(&buf->b_p_tsrfu);
  callback_free(&buf->b_tsrfu_cb);
  clear_cpt_callbacks(&buf->b_p_cpt_cb, buf->b_p_cpt_count);
  buf->b_p_cpt_count = 0;
  clear_string_option(&buf->b_p_gefm);
  clear_string_option(&buf->b_p_gp);
  clear_string_option(&buf->b_p_mp);
  clear_string_option(&buf->b_p_efm);
  clear_string_option(&buf->b_p_ep);
  clear_string_option(&buf->b_p_path);
  clear_string_option(&buf->b_p_tags);
  clear_string_option(&buf->b_p_tc);
  clear_string_option(&buf->b_p_tfu);
  callback_free(&buf->b_tfu_cb);
  clear_string_option(&buf->b_p_ffu);
  callback_free(&buf->b_ffu_cb);
  clear_string_option(&buf->b_p_dict);
  clear_string_option(&buf->b_p_dia);
  clear_string_option(&buf->b_p_tsr);
  clear_string_option(&buf->b_p_qe);
  buf->b_p_ac = -1;
  buf->b_p_ar = -1;
  buf->b_p_ul = NO_LOCAL_UNDOLEVEL;
  clear_string_option(&buf->b_p_lw);
  clear_string_option(&buf->b_p_bkc);
  clear_string_option(&buf->b_p_menc);
}

/// Get alternate file "n".
/// Set linenr to "lnum" or altfpos.lnum if "lnum" == 0.
/// Also set cursor column to altfpos.col if 'startofline' is not set.
/// if (options & GETF_SETMARK) call setpcmark()
/// if (options & GETF_ALT) we are jumping to an alternate file.
/// if (options & GETF_SWITCH) respect 'switchbuf' settings when jumping
///
/// Return FAIL for failure, OK for success.
int buflist_getfile(int n, linenr_T lnum, int options, int forceit)
{
  win_T *wp = NULL;
  fmark_T *fm = NULL;

  buf_T *buf = buflist_findnr(n);
  if (buf == NULL) {
    if ((options & GETF_ALT) && n == 0) {
      emsg(_(e_noalt));
    } else {
      semsg(_("E92: Buffer %" PRId64 " not found"), (int64_t)n);
    }
    return FAIL;
  }

  // if alternate file is the current buffer, nothing to do
  if (buf == curbuf) {
    return OK;
  }

  if (text_or_buf_locked()) {
    return FAIL;
  }

  colnr_T col;
  bool restore_view = false;
  // altfpos may be changed by getfile(), get it now
  if (lnum == 0) {
    fm = buflist_findfmark(buf);
    lnum = fm->mark.lnum;
    col = fm->mark.col;
    restore_view = true;
  } else {
    col = 0;
  }

  if (options & GETF_SWITCH) {
    // If 'switchbuf' is set jump to the window containing "buf".
    wp = swbuf_goto_win_with_buf(buf);

    // If 'switchbuf' contains "split", "vsplit" or "newtab" and the
    // current buffer isn't empty: open new tab or window
    if (wp == NULL && (swb_flags & (kOptSwbFlagVsplit | kOptSwbFlagSplit | kOptSwbFlagNewtab))
        && !buf_is_empty(curbuf)) {
      if (swb_flags & kOptSwbFlagNewtab) {
        tabpage_new();
      } else if (win_split(0, (swb_flags & kOptSwbFlagVsplit) ? WSP_VERT : 0)
                 == FAIL) {
        return FAIL;
      }
      RESET_BINDING(curwin);
    }
  }

  RedrawingDisabled++;
  if (GETFILE_SUCCESS(getfile(buf->b_fnum, NULL, NULL,
                              (options & GETF_SETMARK), lnum, forceit))) {
    RedrawingDisabled--;

    // cursor is at to BOL and w_cursor.lnum is checked due to getfile()
    if (!p_sol && col != 0) {
      curwin->w_cursor.col = col;
      check_cursor_col(curwin);
      curwin->w_cursor.coladd = 0;
      curwin->w_set_curswant = true;
    }
    if (jop_flags & kOptJopFlagView && restore_view) {
      mark_view_restore(fm);
    }
    return OK;
  }
  RedrawingDisabled--;
  return FAIL;
}

/// Go to the last known line number for the current buffer.
static void buflist_getfpos(void)
{
  pos_T *fpos = &buflist_findfmark(curbuf)->mark;

  curwin->w_cursor.lnum = fpos->lnum;
  check_cursor_lnum(curwin);

  if (p_sol) {
    curwin->w_cursor.col = 0;
  } else {
    curwin->w_cursor.col = fpos->col;
    check_cursor_col(curwin);
    curwin->w_cursor.coladd = 0;
    curwin->w_set_curswant = true;
  }
}


/// Same as buflist_findname(), but pass the FileID structure to avoid
/// getting it twice for the same file.
///
/// @return  buffer or NULL if not found
static buf_T *buflist_findname_file_id(char *ffname, FileID *file_id, bool file_id_valid)
  FUNC_ATTR_PURE
{
  FOR_ALL_BUFFERS_BACKWARDS(buf) {
    if ((buf->b_flags & BF_DUMMY) == 0
        && !rs_otherfile_buf_4(buf, ffname, (void *)file_id, file_id_valid)) {
      return buf;
    }
  }
  return NULL;
}


/// Wrapper for buflist_match(). Returns a matched name or NULL.
const char *nvim_blfp_buflist_match(void *handle, buf_T *buf, bool ignore_case)
{
  if (handle == NULL) { return NULL; }
  return buflist_match((regmatch_T *)handle, buf, ignore_case);
}



// buflist_findpat() is implemented in Rust (see src/nvim-rs/buffer/src/list.rs).
// Deleted here (100 lines) in Phase 8.

/// Check if buffer matches the compiled regex. Returns matched name or NULL.
const char *nvim_bufname_regex_match(void *handle, buf_T *buf, bool ignore_case)
{
  if (handle == NULL) {
    return NULL;
  }
  return buflist_match((regmatch_T *)handle, buf, ignore_case);
}


// ExpandBufnames() is implemented in Rust (see src/nvim-rs/buffer/src/expand.rs).
// The C declaration is in buffer.h as a static inline wrapper.
// buflist_match() and fname_match() are kept as static C helpers for nvim_blfp_buflist_match() and nvim_bufname_regex_match().
// NOTE: int ExpandBufnames(char *pat, int *num_file, char ***file, int options)
// was deleted here (152 lines) in Phase 5.
// buf_time_compare() was deleted here (10 lines) in Phase 5.

/// Check for a match on the file name for buffer "buf" with regprog "prog".
/// Note that rmp->regprog may become NULL when switching regexp engine.
///
/// @param ignore_case  When true, ignore case. Use 'fic' otherwise.
static char *buflist_match(regmatch_T *rmp, buf_T *buf, bool ignore_case)
{
  // First try the short file name, then the long file name.
  char *match = fname_match(rmp, buf->b_sfname, ignore_case);
  if (match == NULL && rmp->regprog != NULL) {
    match = fname_match(rmp, buf->b_ffname, ignore_case);
  }
  return match;
}

/// Try matching the regexp in "rmp->regprog" with file name "name".
/// Note that rmp->regprog may become NULL when switching regexp engine.
///
/// @param ignore_case  When true, ignore case. Use 'fileignorecase' otherwise.
///
/// @return  "name" when there is a match, NULL when not.
static char *fname_match(regmatch_T *rmp, char *name, bool ignore_case)
{
  char *match = NULL;

  // extra check for valid arguments
  if (name == NULL || rmp->regprog == NULL) {
    return NULL;
  }

  // Ignore case when 'fileignorecase' or the argument is set.
  rmp->rm_ic = p_fic || ignore_case;
  if (vim_regexec(rmp, name, 0)) {
    match = name;
  } else if (rmp->regprog != NULL) {
    // Replace $(HOME) with '~' and try matching again.
    char *p = home_replace_save(NULL, name);
    if (vim_regexec(rmp, p, 0)) {
      match = name;
    }
    xfree(p);
  }

  return match;
}


/// Set the line and column numbers for the given buffer and window
///
/// @param[in,out]  buf           Buffer for which line and column are set.
/// @param[in,out]  win           Window for which line and column are set.
///                               May be NULL when using :badd.
/// @param[in]      lnum          Line number to be set. If it is zero then only
///                               options are touched.
/// @param[in]      col           Column number to be set.
/// @param[in]      copy_options  If true save the local window option values.
void buflist_setfpos(buf_T *const buf, win_T *const win, linenr_T lnum, colnr_T col,
                     bool copy_options)
  FUNC_ATTR_NONNULL_ARG(1)
{
  WinInfo *wip;

  size_t i;
  for (i = 0; i < kv_size(buf->b_wininfo); i++) {
    wip = kv_A(buf->b_wininfo, i);
    if (wip->wi_win == win) {
      break;
    }
  }

  if (i == kv_size(buf->b_wininfo)) {
    // allocate a new entry
    wip = xcalloc(1, sizeof(WinInfo));
    wip->wi_win = win;
    if (lnum == 0) {            // set lnum even when it's 0
      lnum = 1;
    }
  } else {
    // remove the entry from the list
    kv_shift(buf->b_wininfo, i, 1);
    if (copy_options && wip->wi_optset) {
      clear_winopt(&wip->wi_opt);
      deleteFoldRecurse(buf, &wip->wi_folds);
    }
  }
  if (lnum != 0) {
    wip->wi_mark.mark.lnum = lnum;
    wip->wi_mark.mark.col = col;
    if (win != NULL) {
      wip->wi_mark.view = mark_view_make(win->w_topline, wip->wi_mark.mark);
    }
  }
  if (win != NULL) {
    wip->wi_changelistidx = win->w_changelistidx;
  }
  if (copy_options && win != NULL) {
    // Save the window-specific option values.
    copy_winopt(&win->w_onebuf_opt, &wip->wi_opt);
    wip->wi_fold_manual = win->w_fold_manual;
    rs_cloneFoldGrowArray(&win->w_folds, &wip->wi_folds);
    wip->wi_optset = true;
  }

  // insert the entry in front of the list
  kv_pushp(buf->b_wininfo);
  memmove(&kv_A(buf->b_wininfo, 1), &kv_A(buf->b_wininfo, 0),
          (kv_size(buf->b_wininfo) - 1) * sizeof(kv_A(buf->b_wininfo, 0)));
  kv_A(buf->b_wininfo, 0) = wip;
}

/// Check that "wip" has 'diff' set and the diff is only for another tab page.
/// That's because a diff is local to a tab page.
static bool wininfo_other_tab_diff(WinInfo *wip)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
  if (!wip->wi_opt.wo_diff) {
    return false;
  }

  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    // return false when it's a window in the current tab page, thus
    // the buffer was in diff mode here
    if (wip->wi_win == wp) {
      return false;
    }
  }
  return true;
}

/// Find info for the current window in buffer "buf".
/// If not found, return the info for the most recently used window.
///
/// @param need_options      when true, skip entries where wi_optset is false.
/// @param skip_diff_buffer  when true, avoid windows with 'diff' set that is in another tab page.
///
/// @return  NULL when there isn't any info.
static WinInfo *find_wininfo(buf_T *buf, bool need_options, bool skip_diff_buffer)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_PURE
{
  for (size_t i = 0; i < kv_size(buf->b_wininfo); i++) {
    WinInfo *wip = kv_A(buf->b_wininfo, i);
    if (wip->wi_win == curwin
        && (!skip_diff_buffer || !wininfo_other_tab_diff(wip))
        && (!need_options || wip->wi_optset)) {
      return wip;
    }
  }

  // If no wininfo for curwin, use the first in the list (that doesn't have
  // 'diff' set and is in another tab page).
  // If "need_options" is true skip entries that don't have options set,
  // unless the window is editing "buf", so we can copy from the window
  // itself.
  if (skip_diff_buffer) {
    for (size_t i = 0; i < kv_size(buf->b_wininfo); i++) {
      WinInfo *wip = kv_A(buf->b_wininfo, i);
      if (!wininfo_other_tab_diff(wip)
          && (!need_options
              || wip->wi_optset
              || (wip->wi_win != NULL
                  && wip->wi_win->w_buffer == buf))) {
        return wip;
      }
    }
  } else if (kv_size(buf->b_wininfo)) {
    return kv_A(buf->b_wininfo, 0);
  }
  return NULL;
}

/// Reset the local window options to the values last used in this window.
/// If the buffer wasn't used in this window before, use the values from
/// the most recently used window.  If the values were never set, use the
/// global values for the window.
void get_winopts(buf_T *buf)
{
  clear_winopt(&curwin->w_onebuf_opt);
  rs_clearFolding(curwin);

  WinInfo *const wip = find_wininfo(buf, true, true);
  if (wip != NULL && wip->wi_win != curwin && wip->wi_win != NULL
      && wip->wi_win->w_buffer == buf) {
    win_T *wp = wip->wi_win;
    copy_winopt(&wp->w_onebuf_opt, &curwin->w_onebuf_opt);
    curwin->w_fold_manual = wp->w_fold_manual;
    curwin->w_foldinvalid = true;
    rs_cloneFoldGrowArray(&wp->w_folds, &curwin->w_folds);
  } else if (wip != NULL && wip->wi_optset) {
    copy_winopt(&wip->wi_opt, &curwin->w_onebuf_opt);
    curwin->w_fold_manual = wip->wi_fold_manual;
    curwin->w_foldinvalid = true;
    rs_cloneFoldGrowArray(&wip->wi_folds, &curwin->w_folds);
  } else {
    copy_winopt(&curwin->w_allbuf_opt, &curwin->w_onebuf_opt);
  }
  if (wip != NULL) {
    curwin->w_changelistidx = wip->wi_changelistidx;
  }

  if (curwin->w_config.style == kWinStyleMinimal) {
    didset_window_options(curwin, false);
    win_set_minimal_style(curwin);
  }

  // Set 'foldlevel' to 'foldlevelstart' if it's not negative.
  if (p_fdls >= 0) {
    curwin->w_p_fdl = p_fdls;
  }
  didset_window_options(curwin, false);
}

/// Find the mark for the buffer 'buf' for the current window.
///
/// @return  a pointer to no_position if no position is found.
fmark_T *buflist_findfmark(buf_T *buf)
  FUNC_ATTR_PURE
{
  static fmark_T no_position = { { 1, 0, 0 }, 0, 0, { 0 }, NULL };

  WinInfo *const wip = find_wininfo(buf, false, false);
  return (wip == NULL) ? &no_position : &(wip->wi_mark);
}


/// List all known file names (for :files and :buffers command).
// buflist_list() is implemented in Rust (see src/nvim-rs/buffer/src/list.rs).
// The C declaration is in buffer.h as a static inline wrapper.


/// Set the file name for "buf" to "ffname_arg", short file name to
/// "sfname_arg".
/// The file name with the full path is also remembered, for when :cd is used.
///
/// @param message  give message when buffer already exists
///
/// @return  FAIL for failure (file name already in use by other buffer) OK otherwise.
int setfname(buf_T *buf, char *ffname_arg, char *sfname_arg, bool message)
{
  char *ffname = ffname_arg;
  char *sfname = sfname_arg;
  buf_T *obuf = NULL;
  FileID file_id;
  bool file_id_valid = false;

  if (ffname == NULL || *ffname == NUL) {
    // Removing the name.
    if (buf->b_sfname != buf->b_ffname) {
      XFREE_CLEAR(buf->b_sfname);
    } else {
      buf->b_sfname = NULL;
    }
    XFREE_CLEAR(buf->b_ffname);
  } else {
    fname_expand(buf, &ffname, &sfname);    // will allocate ffname
    if (ffname == NULL) {                   // out of memory
      return FAIL;
    }

    // If the file name is already used in another buffer:
    // - if the buffer is loaded, fail
    // - if the buffer is not loaded, delete it from the list
    file_id_valid = os_fileid(ffname, &file_id);
    if (!(buf->b_flags & BF_DUMMY)) {
      obuf = buflist_findname_file_id(ffname, &file_id, file_id_valid);
    }
    if (obuf != NULL && obuf != buf) {
      bool in_use = false;

      // during startup a window may use a buffer that is not loaded yet
      FOR_ALL_TAB_WINDOWS(tab, win) {
        if (win->w_buffer == obuf) {
          in_use = true;
        }
      }

      // it's loaded or used in a window, fail
      if (obuf->b_ml.ml_mfp != NULL || in_use) {
        if (message) {
          emsg(_("E95: Buffer with this name already exists"));
        }
        xfree(ffname);
        return FAIL;
      }
      // delete from the list
      close_buffer(NULL, obuf, DOBUF_WIPE, false, false);
    }
    sfname = xstrdup(sfname);
#ifdef CASE_INSENSITIVE_FILENAME
    path_fix_case(sfname);            // set correct case for short file name
#endif
    if (buf->b_sfname != buf->b_ffname) {
      xfree(buf->b_sfname);
    }
    xfree(buf->b_ffname);
    buf->b_ffname = ffname;
    buf->b_sfname = sfname;
  }
  buf->b_fname = buf->b_sfname;
  if (!file_id_valid) {
    buf->file_id_valid = false;
  } else {
    buf->file_id_valid = true;
    buf->file_id = file_id;
  }

  buf_name_changed(buf);
  return OK;
}

/// Crude way of changing the name of a buffer.  Use with care!
/// The name should be relative to the current directory.
void buf_set_name(int fnum, char *name)
{
  buf_T *buf = buflist_findnr(fnum);
  if (buf == NULL) {
    return;
  }

  if (buf->b_sfname != buf->b_ffname) {
    xfree(buf->b_sfname);
  }
  xfree(buf->b_ffname);
  buf->b_ffname = xstrdup(name);
  buf->b_sfname = NULL;
  // Allocate ffname and expand into full path.  Also resolves .lnk
  // files on Win32.
  fname_expand(buf, &buf->b_ffname, &buf->b_sfname);
  buf->b_fname = buf->b_sfname;
}

/// Take care of what needs to be done when the name of buffer "buf" has changed.
void buf_name_changed(buf_T *buf)
{
  // If the file name changed, also change the name of the swapfile
  if (buf->b_ml.ml_mfp != NULL) {
    ml_setname(buf);
  }

  if (curwin->w_buffer == buf) {
    check_arg_idx(curwin);      // check file name for arg list
  }
  maketitle();                  // set window title
  status_redraw_all();          // status lines need to be redrawn
  fmarks_check_names(buf);      // check named file marks
  ml_timestamp(buf);            // reset timestamp
}

/// Set alternate file name for current window
///
/// Used by do_one_cmd(), do_write() and do_ecmd().
///
/// @return  the buffer.
buf_T *setaltfname(char *ffname, char *sfname, linenr_T lnum)
{
  // Create a buffer.  'buflisted' is not set if it's a new buffer
  buf_T *buf = buflist_new(ffname, sfname, lnum, 0);
  if (buf != NULL && (cmdmod.cmod_flags & CMOD_KEEPALT) == 0) {
    curwin->w_alt_fnum = buf->b_fnum;
  }
  return buf;
}


#if defined(BACKSLASH_IN_FILENAME)
/// Adjust slashes in file names.  Called after 'shellslash' was set.
void buflist_slash_adjust(void)
{
  FOR_ALL_BUFFERS(bp) {
    if (bp->b_ffname != NULL) {
      slash_adjust(bp->b_ffname);
    }
    if (bp->b_sfname != NULL) {
      slash_adjust(bp->b_sfname);
    }
  }
}

#endif


// fileinfo() is implemented in Rust (see src/nvim-rs/buffer/src/info.rs).
// The C declaration is in buffer.h as a static inline wrapper.


static char *lasttitle = NULL;
static char *lasticon = NULL;

// Phase 9 accessors for maketitle() (Rust migration)
/// Get lasttitle static variable.
const char *nvim_buf_get_lasttitle(void) { return lasttitle; }
/// Set lasttitle static variable (caller transfers ownership of s).
void nvim_buf_set_lasttitle(char *s) { lasttitle = s; }
/// Get lasticon static variable.
const char *nvim_buf_get_lasticon(void) { return lasticon; }
/// Set lasticon static variable (caller transfers ownership of s).
void nvim_buf_set_lasticon(char *s) { lasticon = s; }

// maketitle(), value_change(), resettitle(), free_titles() migrated to Rust
// in src/nvim-rs/buffer/src/info.rs (Phase 9).

/// Open a window for a number of buffers.
void ex_buffer_all(exarg_T *eap)
{
  win_T *wpnext;
  int split_ret = OK;
  int open_wins = 0;
  int had_tab = cmdmod.cmod_tab;

  // Maximum number of windows to open.
  linenr_T count = eap->addr_count == 0
                   ? 9999         // make as many windows as possible
                   : eap->line2;  // make as many windows as specified

  // When true also load inactive buffers.
  int all = eap->cmdidx != CMD_unhide && eap->cmdidx != CMD_sunhide;

  // Stop Visual mode, the cursor and "VIsual" may very well be invalid after
  // switching to another buffer.
  rs_reset_VIsual_and_resel();

  setpcmark();

  // Close superfluous windows (two windows for the same buffer).
  // Also close windows that are not full-width.
  if (had_tab > 0) {
    goto_tabpage_tp(first_tabpage, true, true);
  }
  while (true) {
    tabpage_T *tpnext = curtab->tp_next;
    // Try to close floating windows first
    for (win_T *wp = lastwin->w_floating ? lastwin : firstwin; wp != NULL; wp = wpnext) {
      wpnext = wp->w_floating
               ? wp->w_prev->w_floating ? wp->w_prev : firstwin
               : (wp->w_next == NULL || wp->w_next->w_floating) ? NULL : wp->w_next;
      if ((wp->w_buffer->b_nwindows > 1
           || wp->w_floating
           || ((cmdmod.cmod_split & WSP_VERT)
               ? wp->w_height + wp->w_hsep_height + wp->w_status_height < Rows - p_ch
               - rs_tabline_height() - rs_global_stl_height()
               : wp->w_width != Columns)
           || (had_tab > 0 && wp != firstwin))
          && !ONE_WINDOW
          && !(rs_win_locked(curwin) || wp->w_buffer->b_locked > 0)
          && !is_aucmd_win(wp)) {
        if (win_close(wp, false, false) == FAIL) {
          break;
        }
        // Just in case an autocommand does something strange with
        // windows: start all over...
        wpnext = lastwin->w_floating ? lastwin : firstwin;
        tpnext = first_tabpage;
        open_wins = 0;
      } else {
        open_wins++;
      }
    }

    // Without the ":tab" modifier only do the current tab page.
    if (had_tab == 0 || tpnext == NULL) {
      break;
    }
    goto_tabpage_tp(tpnext, true, true);
  }

  // Go through the buffer list.  When a buffer doesn't have a window yet,
  // open one.  Otherwise move the window to the right position.
  // Watch out for autocommands that delete buffers or windows!
  //
  // Don't execute Win/Buf Enter/Leave autocommands here.
  autocmd_no_enter++;
  // lastwin may be aucmd_win
  win_enter(rs_lastwin_nofloating(), false);
  autocmd_no_leave++;
  for (buf_T *buf = firstbuf; buf != NULL && open_wins < count; buf = buf->b_next) {
    // Check if this buffer needs a window
    if ((!all && buf->b_ml.ml_mfp == NULL) || !buf->b_p_bl) {
      continue;
    }

    win_T *wp;
    if (had_tab != 0) {
      // With the ":tab" modifier don't move the window.
      wp = buf->b_nwindows > 0
           ? lastwin  // buffer has a window, skip it
           : NULL;
    } else {
      // Check if this buffer already has a window
      for (wp = firstwin; wp != NULL; wp = wp->w_next) {
        if (!wp->w_floating && wp->w_buffer == buf) {
          break;
        }
      }
      // If the buffer already has a window, move it
      if (wp != NULL) {
        win_move_after(wp, curwin);
      }
    }

    if (wp == NULL && split_ret == OK) {
      bufref_T bufref;
      set_bufref(&bufref, buf);
      // Split the window and put the buffer in it.
      bool p_ea_save = p_ea;
      p_ea = true;                      // use space from all windows
      split_ret = win_split(0, WSP_ROOM | WSP_BELOW);
      open_wins++;
      p_ea = p_ea_save;
      if (split_ret == FAIL) {
        continue;
      }

      // Open the buffer in this window.
      swap_exists_action = SEA_DIALOG;
      set_curbuf(buf, DOBUF_GOTO, !(jop_flags & kOptJopFlagClean));
      if (!bufref_valid(&bufref)) {
        // Autocommands deleted the buffer.
        swap_exists_action = SEA_NONE;
        break;
      }
      if (swap_exists_action == SEA_QUIT) {
        cleanup_T cs;

        // Reset the error/interrupt/exception state here so that
        // aborting() returns false when closing a window.
        enter_cleanup(&cs);

        // User selected Quit at ATTENTION prompt; close this window.
        win_close(curwin, true, false);
        open_wins--;
        swap_exists_action = SEA_NONE;
        swap_exists_did_quit = true;

        // Restore the error/interrupt/exception state if not
        // discarded by a new aborting error, interrupt, or uncaught
        // exception.
        leave_cleanup(&cs);
      } else {
        handle_swap_exists(NULL);
      }
    }

    os_breakcheck();
    if (got_int) {
      vgetc();            // only break the file loading, not the rest
      break;
    }
    // Autocommands deleted the buffer or aborted script processing!!!
    if (aborting()) {
      break;
    }
    // When ":tab" was used open a new tab for a new window repeatedly.
    if (had_tab > 0 && rs_tabpage_index(NULL) <= p_tpm) {
      cmdmod.cmod_tab = 9999;
    }
  }
  autocmd_no_enter--;
  win_enter(firstwin, false);           // back to first window
  autocmd_no_leave--;

  // Close superfluous windows.
  for (win_T *wp = lastwin; open_wins > count;) {
    bool r = (buf_hide(wp->w_buffer) || !bufIsChanged(wp->w_buffer)
              || autowrite(wp->w_buffer, false) == OK) && !is_aucmd_win(wp);
    if (!rs_win_valid(wp)) {
      // BufWrite Autocommands made the window invalid, start over
      wp = lastwin;
    } else if (r) {
      win_close(wp, !buf_hide(wp->w_buffer), false);
      open_wins--;
      wp = lastwin;
    } else {
      wp = wp->w_prev;
      if (wp == NULL) {
        break;
      }
    }
  }
}

// do_modelines() and chk_modeline() are implemented in Rust
// (see src/nvim-rs/buffer/src/modeline.rs).
// The C declaration is in buffer.h as a static inline wrapper.


/// Read the file for "buf" again and check if the contents changed.
/// Return true if it changed or this could not be checked.
///
/// @param  buf  buffer to check
///
/// @return  true if the buffer's contents have changed
bool buf_contents_changed(buf_T *buf)
  FUNC_ATTR_NONNULL_ALL
{
  bool differ = true;

  // Allocate a buffer without putting it in the buffer list.
  buf_T *newbuf = buflist_new(NULL, NULL, 1, BLN_DUMMY);
  if (newbuf == NULL) {
    return true;
  }

  // Force the 'fileencoding' and 'fileformat' to be equal.
  exarg_T ea;
  prep_exarg(&ea, buf);

  // Set curwin/curbuf to buf and save a few things.
  aco_save_T aco;
  aucmd_prepbuf(&aco, newbuf);

  // We don't want to trigger autocommands now, they may have nasty
  // side-effects like wiping buffers
  block_autocmds();

  if (ml_open(curbuf) == OK
      && readfile(buf->b_ffname, buf->b_fname,
                  0, 0, (linenr_T)MAXLNUM,
                  &ea, READ_NEW | READ_DUMMY, false) == OK) {
    // compare the two files line by line
    if (buf->b_ml.ml_line_count == curbuf->b_ml.ml_line_count) {
      differ = false;
      for (linenr_T lnum = 1; lnum <= curbuf->b_ml.ml_line_count; lnum++) {
        if (strcmp(ml_get_buf(buf, lnum), ml_get(lnum)) != 0) {
          differ = true;
          break;
        }
      }
    }
  }
  xfree(ea.cmd);

  // restore curwin/curbuf and a few other things
  aucmd_restbuf(&aco);

  if (curbuf != newbuf) {  // safety check
    wipe_buffer(newbuf, false);
  }

  unblock_autocmds();

  return differ;
}


/// Creates or switches to a scratch buffer. :h special-buffers
/// Scratch buffer is:
///   - buftype=nofile bufhidden=hide noswapfile
///   - Always considered 'nomodified'
///
/// @param bufnr     Buffer to switch to, or 0 to create a new buffer.
/// @param bufname   Buffer name, or NULL.
///
/// @see curbufIsChanged()
///
/// @return  FAIL for failure, OK otherwise
int buf_open_scratch(handle_T bufnr, char *bufname)
{
  if (do_ecmd((int)bufnr, NULL, NULL, NULL, ECMD_ONE, ECMD_HIDE, NULL) == FAIL) {
    return FAIL;
  }
  if (bufname != NULL) {
    apply_autocmds(EVENT_BUFFILEPRE, NULL, NULL, false, curbuf);
    setfname(curbuf, bufname, NULL, true);
    apply_autocmds(EVENT_BUFFILEPOST, NULL, NULL, false, curbuf);
  }
  set_option_value_give_err(kOptBufhidden, STATIC_CSTR_AS_OPTVAL("hide"), OPT_LOCAL);
  set_option_value_give_err(kOptBuftype, STATIC_CSTR_AS_OPTVAL("nofile"), OPT_LOCAL);
  set_option_value_give_err(kOptSwapfile, BOOLEAN_OPTVAL(false), OPT_LOCAL);
  RESET_BINDING(curwin);
  return OK;
}


/// Set b:changedtick, also checking b: for consistency in debug build
///
/// @param[out]  buf  Buffer to set changedtick in.
/// @param[in]  changedtick  New value.
void buf_set_changedtick(buf_T *const buf, const varnumber_T changedtick)
  FUNC_ATTR_NONNULL_ALL
{
  typval_T old_val = buf->changedtick_di.di_tv;

#ifndef NDEBUG
  dictitem_T *const changedtick_di = tv_dict_find(buf->b_vars, S_LEN("changedtick"));
  assert(changedtick_di != NULL);
  assert(changedtick_di->di_tv.v_type == VAR_NUMBER);
  assert(changedtick_di->di_tv.v_lock == VAR_FIXED);
  // For some reason formatc does not like the below.
# ifndef UNIT_TESTING_LUA_PREPROCESSING
  assert(changedtick_di->di_flags == (DI_FLAGS_RO|DI_FLAGS_FIX));
# endif
  assert(changedtick_di == (dictitem_T *)&buf->changedtick_di);
#endif
  buf->changedtick_di.di_tv.vval.v_number = changedtick;

  if (tv_dict_is_watched(buf->b_vars)) {
    buf->b_locked++;
    tv_dict_watcher_notify(buf->b_vars,
                           (char *)buf->changedtick_di.di_key,
                           &buf->changedtick_di.di_tv,
                           &old_val);
    buf->b_locked--;
  }
}

/// Read the given buffer contents into a string.
void read_buffer_into(buf_T *buf, linenr_T start, linenr_T end, StringBuilder *sb)
  FUNC_ATTR_NONNULL_ALL
{
  assert(buf);
  assert(sb);

  if (buf->b_ml.ml_flags & ML_EMPTY) {
    return;
  }

  size_t written = 0;
  size_t len = 0;
  linenr_T lnum = start;
  char *lp = ml_get_buf(buf, lnum);
  size_t lplen = (size_t)ml_get_buf_len(buf, lnum);

  while (true) {
    if (lplen == 0) {
      len = 0;
    } else if (lp[written] == NL) {
      // NL -> NUL translation
      len = 1;
      kv_push(*sb, NUL);
    } else {
      char *s = vim_strchr(lp + written, NL);
      len = s == NULL ? lplen - written : (size_t)(s - (lp + written));
      kv_concat_len(*sb, lp + written, len);
    }

    if (len == lplen - written) {
      // Finished a line, add a NL, unless this line should not have one.
      if (lnum != end
          || (!buf->b_p_bin && buf->b_p_fixeol)
          || (lnum != buf->b_no_eol_lnum
              && (lnum != buf->b_ml.ml_line_count || buf->b_p_eol))) {
        kv_push(*sb, NL);
      }
      lnum++;
      if (lnum > end) {
        break;
      }
      lp = ml_get_buf(buf, lnum);
      lplen = (size_t)ml_get_buf_len(buf, lnum);
      written = 0;
    } else if (len > 0) {
      written += len;
    }
  }
}

// ============================================================================
// Extmark Accessor Functions (for Rust FFI - extmark crate)
// ============================================================================

/// Get the marktree pointer from a buffer.
MarkTree *nvim_buf_get_marktree(buf_T *buf)
{
  return buf->b_marktree;
}

/// Get the deleted_bytes2 field from a buffer.
bcount_t nvim_buf_get_deleted_bytes2(buf_T *buf)
{
  return buf->deleted_bytes2;
}

/// Set the deleted_bytes2 field in a buffer.
void nvim_buf_set_deleted_bytes2(buf_T *buf, bcount_t val)
{
  buf->deleted_bytes2 = val;
}

/// Get the b_prev_line_count field from a buffer (for extmark adjust).
int nvim_buf_get_prev_line_count(buf_T *buf)
{
  return buf->b_prev_line_count;
}

/// Set the b_prev_line_count field in a buffer.
void nvim_buf_set_prev_line_count(buf_T *buf, int val)
{
  buf->b_prev_line_count = val;
}

/// Get the autom field from b_signcols.
bool nvim_buf_signcols_get_autom(buf_T *buf)
{
  return buf->b_signcols.autom;
}

/// Clear the b_signcols structure.
void nvim_buf_signcols_clear(buf_T *buf)
{
  buf->b_signcols.max = 0;
  CLEAR_FIELD(buf->b_signcols.count);
}
