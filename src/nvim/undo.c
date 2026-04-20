// undo.c: multi level undo facility

// The saved lines are stored in a list of lists (one for each buffer):
//
// b_u_oldhead------------------------------------------------+
//                                                            |
//                                                            V
//                +--------------+    +--------------+    +--------------+
// b_u_newhead--->| u_header     |    | u_header     |    | u_header     |
//                |     uh_next------>|     uh_next------>|     uh_next---->NULL
//         NULL<--------uh_prev  |<---------uh_prev  |<---------uh_prev  |
//                |     uh_entry |    |     uh_entry |    |     uh_entry |
//                +--------|-----+    +--------|-----+    +--------|-----+
//                         |                   |                   |
//                         V                   V                   V
//                +--------------+    +--------------+    +--------------+
//                | u_entry      |    | u_entry      |    | u_entry      |
//                |     ue_next  |    |     ue_next  |    |     ue_next  |
//                +--------|-----+    +--------|-----+    +--------|-----+
//                         |                   |                   |
//                         V                   V                   V
//                +--------------+            NULL                NULL
//                | u_entry      |
//                |     ue_next  |
//                +--------|-----+
//                         |
//                         V
//                        etc.
//
// Each u_entry list contains the information for one undo or redo.
// curbuf->b_u_curhead points to the header of the last undo (the next redo),
// or is NULL if nothing has been undone (end of the branch).
//
// For keeping alternate undo/redo branches the uh_alt field is used.  Thus at
// each point in the list a branch may appear for an alternate to redo.  The
// uh_seq field is numbered sequentially to be able to find a newer or older
// branch.
//
//                 +---------------+    +---------------+
// b_u_oldhead --->| u_header      |    | u_header      |
//                 |   uh_alt_next ---->|   uh_alt_next ----> NULL
//         NULL <----- uh_alt_prev |<------ uh_alt_prev |
//                 |   uh_prev     |    |   uh_prev     |
//                 +-----|---------+    +-----|---------+
//                       |                    |
//                       V                    V
//                 +---------------+    +---------------+
//                 | u_header      |    | u_header      |
//                 |   uh_alt_next |    |   uh_alt_next |
// b_u_newhead --->|   uh_alt_prev |    |   uh_alt_prev |
//                 |   uh_prev     |    |   uh_prev     |
//                 +-----|---------+    +-----|---------+
//                       |                    |
//                       V                    V
//                     NULL             +---------------+    +---------------+
//                                      | u_header      |    | u_header      |
//                                      |   uh_alt_next ---->|   uh_alt_next |
//                                      |   uh_alt_prev |<------ uh_alt_prev |
//                                      |   uh_prev     |    |   uh_prev     |
//                                      +-----|---------+    +-----|---------+
//                                            |                    |
//                                           etc.                 etc.
//
//
// All data is allocated and will all be freed when the buffer is unloaded.

#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <uv.h>

#include "auto/config.h"
#include "klib/kvec.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/change.h"
#include "nvim/cursor.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/errors.h"
#include "nvim/eval/funcs.h"
#include "nvim/eval/typval.h"
#include "nvim/extmark.h"
#include "nvim/extmark_defs.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/highlight_defs.h"
#include "nvim/macros_defs.h"
#include "nvim/mark.h"
#include "nvim/mark_defs.h"
#include "nvim/memline.h"
#include "nvim/memline_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/os/fs.h"
#include "nvim/os/fs_defs.h"
#include "nvim/os/input.h"
#include "nvim/os/os_defs.h"
#include "nvim/os/time.h"
#include "nvim/os/time_defs.h"
#include "nvim/path.h"
#include "nvim/pos_defs.h"
#include "nvim/state.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/undo.h"
#include "nvim/undo_defs.h"
#include "nvim/vim_defs.h"

extern char *rs_f_undofile(const char *fname);
extern void rs_foldOpenCursor(void);

#include "undo.c.generated.h"

static const char e_undo_list_corrupt[]
  = N_("E439: Undo list corrupt");
static const char e_undo_line_missing[]
  = N_("E440: Undo line missing");
static const char e_write_error_in_undo_file_str[]
  = N_("E829: Write error in undo file: %s");


/// Get the 'undolevels' value for the current buffer.
static OptInt get_undolevel(buf_T *buf)
{
  if (buf->b_p_ul == NO_LOCAL_UNDOLEVEL) {
    return p_ul;
  }
  return buf->b_p_ul;
}

// zero_fmark_additional_data: migrated to Rust (rs_zero_fmark_additional_data).

/// "undofile(name)" function
void f_undofile(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rettv->v_type = VAR_STRING;
  const char *const fname = tv_get_string(&argvars[0]);
  rettv->vval.v_string = rs_f_undofile(fname);
}

// Buffer state accessors
bool nvim_buf_get_b_changed(buf_T *buf) { return buf->b_changed; }

bool nvim_bt_dontwrite(buf_T *buf) { return bt_dontwrite(buf); }

// Global buffer iteration
buf_T *nvim_get_firstbuf(void) { return firstbuf; }

buf_T *nvim_buf_get_next(buf_T *buf) { return buf->b_next; }

// Error message wrappers
void nvim_iemsg_undo_list_corrupt(void) { iemsg(_(e_undo_list_corrupt)); }

void nvim_iemsg_undo_line_missing(void) { iemsg(_(e_undo_line_missing)); }

void nvim_iemsg_undo_line_numbers_wrong(void) { iemsg(_("E438: u_undo: line numbers wrong")); }

// Global state accessors
int nvim_get_no_u_sync(void) { return no_u_sync; }

OptInt nvim_get_undolevel(buf_T *buf) { return get_undolevel(buf); }


bool nvim_buf_is_modifiable(buf_T *buf) { return MODIFIABLE(buf); }

int nvim_get_sandbox(void) { return sandbox; }

// undo_allowed error message wrappers
void nvim_emsg_modifiable(void) { emsg(_(e_modifiable)); }

void nvim_emsg_sandbox(void) { emsg(_(e_sandbox)); }

void nvim_emsg_textlock(void) { emsg(_(e_textlock)); }

void nvim_emsg_undojoin_after_undo(void) { emsg(_("E790: undojoin is not allowed after undo")); }

bool nvim_has_cpo_undo(void) { return vim_strchr(p_cpo, CPO_UNDO) != NULL; }

bool nvim_buf_ml_is_empty(buf_T *buf) { return buf->b_ml.ml_flags & ML_EMPTY; }

void nvim_change_warning_curbuf(void) { change_warning(curbuf, 0); }

void nvim_msg_oldest_change(void) { msg(_("Already at oldest change"), 0); }

void nvim_msg_newest_change(void) { msg(_("Already at newest change"), 0); }

void nvim_get_curwin_cursor(linenr_T *lnum, colnr_T *col, colnr_T *coladd)
{
  *lnum = curwin->w_cursor.lnum;
  *col = curwin->w_cursor.col;
  *coladd = curwin->w_cursor.coladd;
}

bool nvim_curwin_virtual_active(void) { return virtual_active(curwin); }

colnr_T nvim_getviscol(void) { return getviscol(); }

void nvim_buf_set_b_new_change(buf_T *buf, bool val) { buf->b_new_change = val; }

// nvim_uhp_copy_marks_visual: migrated to Rust (rs_uhp_copy_marks_visual_impl).

// Error message wrapper
void nvim_emsg_line_count_changed(void) { emsg(_("E881: Line count changed unexpectedly")); }

// Check if buf equals curbuf
bool nvim_buf_is_curbuf(buf_T *buf) { return buf == curbuf; }

colnr_T nvim_undo_curwin_get_cursor_col(void) { return curwin->w_cursor.col; }

void nvim_undo_curwin_set_cursor_col(colnr_T col) { curwin->w_cursor.col = col; }

void nvim_undo_curwin_set_cursor_lnum(linenr_T lnum) { curwin->w_cursor.lnum = lnum; }

void nvim_check_cursor_col_curwin(void) { check_cursor_col(curwin); }

linenr_T nvim_undo_curwin_get_cursor_lnum(void) { return curwin->w_cursor.lnum; }

// Strftime wrapper for Rust time formatting
size_t nvim_undo_strftime(char *buf, size_t buflen, const char *fmt, time_t tt)
{
  struct tm curtime;
  os_localtime_r(&tt, &curtime);
  return strftime(buf, buflen, fmt, &curtime);
}

// Return the localized "N second(s) ago" string
const char *nvim_undo_time(int64_t seconds)
{
  static char buf[256];
  vim_snprintf(buf, sizeof(buf),
               NGETTEXT("%" PRId64 " second ago",
                        "%" PRId64 " seconds ago", (uint32_t)seconds),
               seconds);
  return buf;
}

void nvim_semsg_undo_number_not_found(int64_t step) { semsg(_("E830: Undo number %" PRId64 " not found"), step); }

// File system operations (kept as wrappers; other crates use these names)
bool nvim_os_path_exists(const char *path) { return os_path_exists(path); }

int nvim_os_remove(const char *path) { return os_remove(path); }

int nvim_get_p_verbose(void) { return p_verbose; }

void nvim_set_p_verbose(int val) { p_verbose = val; }

bool nvim_get_p_fs(void) { return p_fs; }

// u_sync wrapper (still called from ex_cmds and window Rust crates)
void nvim_u_sync(bool force) { u_sync(force); }

// Extmark Accessor Functions (for Rust FFI - extmark crate)

/// Get extmark undo vector pointer from undo header.
extmark_undo_vec_t *nvim_uhp_get_extmark(u_header_T *uhp) { return &uhp->uh_extmark; }

// Undo File I/O Message Functions (for Rust FFI)

void nvim_undo_cannot_write_no_dir(void) { verb_msg(_("Cannot write undo file in any directory in 'undodir'")); }

void nvim_undo_will_not_overwrite_cannot_read(const char *file_name)
{
  smsg(0, _("Will not overwrite with undo file, cannot read: %s"), file_name);
}

void nvim_undo_will_not_overwrite_not_undo(const char *file_name)
{
  smsg(0, _("Will not overwrite, this is not an undo file: %s"), file_name);
}

void nvim_undo_skip_write_nothing(void) { verb_msg(_("Skipping undo file write, nothing to undo")); }

void nvim_undo_write_error(const char *file_name) { semsg(_(e_write_error_in_undo_file_str), file_name); }

void nvim_undo_writing(const char *file_name) { smsg(0, _("Writing undo file: %s"), file_name); }

void nvim_undo_reading(const char *file_name) { smsg(0, _("Reading undo file: %s"), file_name); }

void nvim_undo_not_reading_owner_differs(const char *file_name)
{
  smsg(0, _("Not reading undo file, owner differs: %s"), file_name);
}

void nvim_undo_cannot_open_for_reading(const char *file_name)
{
  semsg(_("E822: Cannot open undo file for reading: %s"), file_name);
}

void nvim_undo_not_undo_file(const char *file_name) { semsg(_("E823: Not an undo file: %s"), file_name); }

void nvim_undo_incompatible_version(const char *file_name) { semsg(_("E824: Incompatible undo file: %s"), file_name); }

void nvim_undo_corruption_error(const char *what, const char *file_name)
{
  semsg(_("E825: Corrupted undo file (%s): %s"), what, file_name);
}

void nvim_undo_file_changed_warning(void) { give_warning(_("File contents changed, cannot use undo info"), true); }

void nvim_undo_finished_reading(const char *file_name) { smsg(0, _("Finished reading undo file %s"), file_name); }

// nvim_undoredo_save_marks, nvim_undoredo_restore_marks,
// nvim_undoredo_swap_visual, nvim_undoredo_get_buf_marks,
// nvim_undoredo_init_op_marks: migrated to Rust (Phase 1).

linenr_T nvim_buf_get_op_start_lnum(buf_T *buf) { return buf->b_op_start.lnum; }

linenr_T nvim_buf_get_op_end_lnum(buf_T *buf) { return buf->b_op_end.lnum; }

void nvim_buf_set_op_start_lnum(buf_T *buf, linenr_T lnum) { buf->b_op_start.lnum = lnum; }

void nvim_buf_adjust_op_start_lnum(buf_T *buf, linenr_T delta) { buf->b_op_start.lnum += delta; }

void nvim_buf_set_op_end_lnum(buf_T *buf, linenr_T lnum) { buf->b_op_end.lnum = lnum; }

void nvim_buf_adjust_op_end_lnum(buf_T *buf, linenr_T delta) { buf->b_op_end.lnum += delta; }

void nvim_buf_set_op_start_col(buf_T *buf, colnr_T col) { buf->b_op_start.col = col; }

void nvim_buf_set_op_end_col(buf_T *buf, colnr_T col) { buf->b_op_end.col = col; }

fmark_T *nvim_buf_get_namedm_ptr(buf_T *buf) { return buf->b_namedm; }

visualinfo_T *nvim_buf_get_b_visual_ptr(buf_T *buf) { return &buf->b_visual; }

void nvim_undoredo_set_ml_empty(buf_T *buf, int old_flags)
{
  if ((old_flags & UH_EMPTYBUF) && buf_is_empty(buf)) {
    buf->b_ml.ml_flags |= ML_EMPTY;
  }
}

// Redraw conceal for all windows showing this buffer
void nvim_undo_end_redraw_conceal(buf_T *buf)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (wp->w_buffer == buf && wp->w_p_cole > 0) {
      redraw_later(wp, UPD_NOT_VALID);
    }
  }
}

// Check VIsual and call check_pos
void nvim_undo_end_check_visual(buf_T *buf)
{
  if (VIsual_active) {
    check_pos(buf, &VIsual);
  }
}

// Format and display the undo end message
void nvim_undo_end_smsg(int64_t count, const char *msgstr, bool did_undo,
                         int64_t seq, const char *timebuf)
{
  smsg_keep(0, _("%" PRId64 " %s; %s #%" PRId64 "  %s"),
            count,
            _(msgstr),
            did_undo ? _("before") : _("after"),
            seq,
            timebuf);
}

// nvim_undo_resolve_symlink: migrated to Rust (nvim_undo_resolve_symlink).

const char *nvim_undo_get_p_udir(void) { return p_udir; }

// Emit a semsg with one string argument (used from Rust undo code)
void nvim_undo_semsg(const char *msg, const char *arg) { semsg(msg, arg); }

// Format and emit E5003 error message
void nvim_undo_semsg_mkdir(const char *failed_dir, int err)
{
  semsg(_("E5003: Unable to create directory \"%s\" for undo file: %s"),
        failed_dir, os_strerror(err));
}

size_t nvim_undo_get_maxpathl(void) { return MAXPATHL; }

void nvim_undo_msg_simple(const char *s) { msg(s, 0); }

void nvim_undo_msg_puts_hl_title(const char *s) { msg_puts_hl(s, HLF_T, false); }

list_T *nvim_tv_list_alloc(void) { return tv_list_alloc(kListLenMayKnow); }

void nvim_tv_dict_add_nr(dict_T *dict, const char *key, size_t key_len, varnumber_T nr)
{
  tv_dict_add_nr(dict, key, key_len, nr);
}

/// Current window handle accessor
win_T *nvim_undo_get_curwin(void) { return curwin; }

/// Window buffer accessor
buf_T *nvim_undo_win_get_buffer(win_T *win) { return win->w_buffer; }

/// Set window cursor
void nvim_undo_win_set_cursor_pos(win_T *win, linenr_T lnum, colnr_T col, colnr_T coladd)
{
  win->w_cursor.lnum = lnum;
  win->w_cursor.col = col;
  win->w_cursor.coladd = coladd;
}

/// Get window cursor line
linenr_T nvim_undo_win_get_cursor_lnum(win_T *win) { return win->w_cursor.lnum; }

bool nvim_get_global_busy(void) { return global_busy; }

/// Increment global_busy (for breaking :global command on error)
void nvim_inc_global_busy(void) { global_busy++; }

/// Check if messaging is allowed
bool nvim_messaging(void) { return messaging(); }

/// Get KeyTyped flag
bool nvim_undo_get_key_typed(void) { return KeyTyped; }

/// Get fdo_flags for fold options
int nvim_undo_get_fdo_flags(void) { return fdo_flags; }

/// Increment no_u_sync.
void nvim_inc_no_u_sync(void) { no_u_sync++; }
/// Decrement no_u_sync.
void nvim_dec_no_u_sync(void) { no_u_sync--; }

/// Increment sandbox.
void nvim_inc_sandbox(void) { sandbox++; }
/// Decrement sandbox.
void nvim_dec_sandbox(void) { sandbox--; }

