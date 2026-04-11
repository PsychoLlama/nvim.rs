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
#include "nvim/ui.h"
#include "nvim/undo.h"
#include "nvim/usercmd.h"
#include "nvim/version.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"
#include "nvim/winfloat.h"

#include "buffer.c.generated.h"
extern void rs_aubuflocal_remove(int bufnr);
extern bool rs_is_dev_fd_file(const char *fname);
extern int rs_win_valid(win_T *win);
extern int rs_win_valid_any_tab(win_T *win);
extern int rs_one_window_in_tab(win_T *win, tabpage_T *tp);
extern int rs_last_window(win_T *win);

// Rust fold FFI declarations
extern void rs_clearFolding(win_T *win);
extern void rs_foldUpdateAll(win_T *win);
extern void rs_cloneFoldGrowArray(garray_T *from, garray_T *to);

// Rust implementations
extern bool rs_bt_nofileread(buf_T *buf);

extern void rs_diff_buf_delete(buf_T *buf);
extern void rs_diff_buf_add(buf_T *buf);
extern int rs_diffopt_hiddenoff(void);

// File identity helpers from Rust
extern bool rs_otherfile_buf_4(buf_T *buf, char *ffname, void *file_id_p, bool file_id_valid);

extern buf_T *rs_find_buffer_for_delete(int buf_fnum, int *update_jumplist);
extern buf_T *rs_find_and_validate_buffer(int action, int start, int dir, int count, int flags,
                                          int unload);
extern int rs_buf_effective_action(buf_T *buf, int action);

// Accessor functions for Rust opaque handle pattern are in buffer_shim.c.
// Only accessor functions that reference file-scope static variables remain here.

// Number of times free_buffer() was called.
static int buf_free_count = 0;

/// Get the buf_free_count global (accessor for Rust).
int nvim_get_buf_free_count(void) { return buf_free_count; }

static int top_file_num = 1;            ///< highest file number

/// Get the top_file_num global (accessor for Rust).
int nvim_get_top_file_num(void) { return top_file_num; }


// Static assertions for constants used in Rust (Phase 1).
_Static_assert(ML_EMPTY == 0x01, "ML_EMPTY mismatch with Rust");
_Static_assert(DOBUF_WIPE == 4, "DOBUF_WIPE mismatch with Rust");

// Static assertions for constants used in Rust (Phase 4).
_Static_assert(EVENT_BUFADD == 0, "EVENT_BUFADD mismatch with Rust");
_Static_assert(EVENT_BUFDELETE == 2, "EVENT_BUFDELETE mismatch with Rust");

// Static assertions for CMD_* constants used by goto_buffer in Rust.
_Static_assert(CMD_bnext == 30, "CMD_bnext mismatch with Rust");
_Static_assert(CMD_sbnext == 393, "CMD_sbnext mismatch with Rust");
_Static_assert(CMD_bNext == 21, "CMD_bNext mismatch with Rust");
_Static_assert(CMD_bprevious == 32, "CMD_bprevious mismatch with Rust");
_Static_assert(CMD_sbNext == 388, "CMD_sbNext mismatch with Rust");
_Static_assert(CMD_sbprevious == 394, "CMD_sbprevious mismatch with Rust");

// BufFreeFlags moved to buffer.h (Phase 22).

// read_buffer() migrated to Rust misc.rs (Phase 2).
extern int rs_read_buffer(bool read_stdin, exarg_T *eap, int flags);
// open_buffer() migrated to Rust lifecycle.rs (Phase N).
extern int open_buffer(bool read_stdin, exarg_T *eap, int flags_arg);




// free_buffer(), clear_wininfo(), free_buffer_stuff() migrated to Rust close.rs (Phase N).
extern void free_buffer(buf_T *buf);
extern void clear_wininfo(buf_T *buf);
extern void free_buffer_stuff(buf_T *buf, int free_flags);

// handle_swap_exists() moved to buffer_shim.c (Phase 18).

// set_curbuf() moved to buffer_shim.c (Phase 19).

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

/// Non-static wrapper for buf_init_changedtick (called from buffer_shim.c).
void nvim_buf_init_changedtick_c(buf_T *buf)
{
  buf_init_changedtick(buf);
}

/// Increment buf_free_count (called from buffer_shim.c compound wrappers).
void nvim_inc_buf_free_count(void)
{
  buf_free_count++;
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

// buflist_getfpos(), enter_buffer(), handle_swap_exists(), ex_buffer_all()
// migrated to Rust lifecycle.rs (Phases 1, 3, 4).
_Static_assert(CMD_unhide == 495, "CMD_unhide value changed");
_Static_assert(CMD_sunhide == 437, "CMD_sunhide value changed");

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
