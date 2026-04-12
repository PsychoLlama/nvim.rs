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

// buf_free_count and top_file_num migrated to Rust state.rs (Phase 1).
// Accessors (nvim_get/inc/set/reset_top_file_num, nvim_get/inc_buf_free_count)
// are exported by the Rust buffer crate and called from buffer_shim.c.

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

// buf_init_changedtick() and nvim_buf_init_changedtick_c() migrated to buffer_shim.c (Phase 4).

// buflist_new() migrated to Rust close.rs (Phase 3).
// Implementation body moved to nvim_buflist_new_impl() in buffer_shim.c.
extern buf_T *buflist_new(char *ffname_arg, char *sfname_arg, linenr_T lnum, int flags);

// free_buf_options() migrated to Rust close.rs (Phase 2).
// Implementation body moved to nvim_buf_do_free_options() in buffer_shim.c.
extern void free_buf_options(buf_T *buf, bool free_p_ff);

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
