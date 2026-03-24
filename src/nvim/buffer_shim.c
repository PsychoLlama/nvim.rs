// buffer_shim.c: C accessor wrappers for the Rust buffer crate (nvim-buffer).
//
// These thin wrappers provide a stable C ABI for Rust code to call into
// Neovim's C internals. Each function is called from one or more Rust
// modules in src/nvim-rs/buffer/.

#include <assert.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include "klib/kvec.h"
#include "nvim/arglist.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/channel.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/errors.h"
#include "nvim/eval/typval.h"
#include "nvim/fold.h"
#include "nvim/extmark_defs.h"
#include "nvim/file_search.h"
#include "nvim/fileio.h"
#include "nvim/fuzzy.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/indent.h"
#include "nvim/macros_defs.h"
#include "nvim/mark.h"
#include "nvim/memline.h"
#include "nvim/message.h"
#include "nvim/option.h"
#include "nvim/os/fs.h"
#include "nvim/os/os.h"
#include "nvim/path.h"
#include "nvim/quickfix.h"
#include "nvim/regexp.h"
#include "nvim/runtime.h"
#include "nvim/spell.h"
#include "nvim/strings.h"
#include "nvim/terminal.h"
#include "nvim/types_defs.h"
#include "nvim/syntax.h"
#include "nvim/undo.h"
#include "nvim/vim_defs.h"
#include "nvim/cursor.h"
#include "nvim/digraph.h"
#include "nvim/drawscreen.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds2.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_eval.h"
#include "nvim/getchar.h"
#include "nvim/memory.h"
#include "nvim/os/input.h"
#include "nvim/window.h"
#include "nvim/winfloat.h"

#include "buffer_shim.c.generated.h"

// Rust-exported fold functions used in wininfo cluster (Phase 11)
extern void rs_cloneFoldGrowArray(garray_T *from, garray_T *to);
extern void rs_clearFolding(win_T *win);
extern void rs_foldUpdateAll(win_T *win);
// Internal arglist function used in buf_name_changed (Phase 13)
extern void check_arg_idx(win_T *win);
// Rust file identity helper used in buflist_findname_file_id (Phase 14)
extern bool rs_otherfile_buf_4(buf_T *buf, char *ffname, void *file_id_p, bool file_id_valid);
// Rust window helpers used in ex_buffer_all (Phase 17)
extern void rs_reset_VIsual_and_resel(void);
extern int rs_win_locked(win_T *wp);
extern int rs_global_stl_height(void);
extern win_T *rs_lastwin_nofloating(void);
extern int rs_tabline_height(void);
extern int rs_tabpage_index(tabpage_T *ftp);
extern int rs_win_valid(win_T *win);
// rs_get_last_winid() is Rust-exported (used in set_curbuf, Phase 19).
extern int rs_get_last_winid(void);
// Rust diff helpers (used in enter_buffer, Phase 20).
extern void rs_diff_buf_add(buf_T *buf);

// ============================================================
// Core buf_T field accessors (Phase 1 / lifecycle)
// ============================================================

/// Get the buffer handle (b_fnum) from a buffer.
int nvim_buf_get_handle(buf_T *buf)
{
  return buf ? buf->b_fnum : 0;
}

/// Get the first character of the b_p_bt (buftype option) field.
char nvim_buf_get_buftype(buf_T *buf)
{
  return buf->b_p_bt[0];
}

/// Get the third character of the b_p_bt field (for nofile/nowrite check).
char nvim_buf_get_buftype_2(buf_T *buf)
{
  return buf->b_p_bt[2];
}

/// Get the b_help field from a buffer.
int nvim_buf_get_help(buf_T *buf)
{
  return buf->b_help;
}

/// Check if buffer has a terminal attached (buf->terminal != NULL).
int nvim_buf_get_terminal(buf_T *buf)
{
  return buf->terminal != NULL;
}

/// Get the first character of the b_p_ff (fileformat option) field.
char nvim_buf_get_fileformat(buf_T *buf)
{
  return buf->b_p_ff[0];
}

/// Get the b_p_bin (binary mode) field from a buffer.
int nvim_buf_get_bin(buf_T *buf)
{
  return buf->b_p_bin;
}

/// Get the last buffer in the buffer list (lastbuf global).
buf_T *nvim_get_lastbuf(void)
{
  return lastbuf;
}

/// Get the b_prev field from a buffer.
buf_T *nvim_buf_get_prev(buf_T *buf)
{
  return buf->b_prev;
}

/// Get the br_buf field from a bufref (accessor for Rust).
buf_T *nvim_bufref_get_buf(bufref_T *bufref)
{
  return bufref->br_buf;
}

/// Get the total sign HL metadata count for a buffer.
uint32_t nvim_buf_meta_total_sign_hl(buf_T *buf)
{
  return buf ? buf_meta_total(buf, kMTMetaSignHL) : 0;
}

/// Get the total sign text metadata count for a buffer.
uint32_t nvim_buf_meta_total_sign_text(buf_T *buf)
{
  return buf ? buf_meta_total(buf, kMTMetaSignText) : 0;
}

/// Get the br_fnum field from a bufref (accessor for Rust).
int nvim_bufref_get_fnum(bufref_T *bufref)
{
  return bufref->br_fnum;
}

/// Get the br_buf_free_count field from a bufref (accessor for Rust).
int nvim_bufref_get_buf_free_count(bufref_T *bufref)
{
  return bufref->br_buf_free_count;
}

/// Get the b_fnum field from a buffer (accessor for Rust).
int nvim_buf_get_fnum(buf_T *buf)
{
  return buf->b_fnum;
}

/// Get the first character of the b_p_bh (bufhidden option) field.
char nvim_buf_get_bufhidden(buf_T *buf)
{
  return buf->b_p_bh[0];
}

/// Get the b_fname field from a buffer (short filename).
const char *nvim_buf_get_b_fname(buf_T *buf)
{
  return buf->b_fname;
}
/// Get the b_p_syn option field from a buffer.
const char *nvim_buf_get_b_p_syn(buf_T *buf) { return buf ? buf->b_p_syn : NULL; }

/// Get the b_ffname field from a buffer (full filename).
const char *nvim_buf_get_b_ffname(buf_T *buf)
{
  return buf->b_ffname;
}

/// Get the b_sfname field from a buffer (short filename for display).
const char *nvim_buf_get_b_sfname(buf_T *buf)
{
  return buf->b_sfname;
}

/// Get the b_p_efm (errorformat option) field from a buffer.
const char *nvim_buf_get_b_p_efm(buf_T *buf)
{
  return buf->b_p_efm;
}

/// Get the b_p_ro (readonly option) field from a buffer.
int nvim_buf_get_b_p_ro(buf_T *buf)
{
  return buf->b_p_ro;
}

/// Get the b_p_ft (filetype option) field from a buffer.
const char *nvim_buf_get_b_p_ft(buf_T *buf)
{
  return buf->b_p_ft;
}

/// Get the b_p_ma (modifiable option) field from a buffer.
int nvim_buf_get_b_p_ma(buf_T *buf)
{
  return buf->b_p_ma;
}


/// Set the b_p_ml (modeline) field on a buffer.
void nvim_buf_set_b_p_ml(buf_T *buf, int val)
{
  if (buf) {
    buf->b_p_ml = val != 0;
  }
}

/// Set the b_p_iminsert field on a buffer.
void nvim_buf_set_b_p_iminsert(buf_T *buf, int val)
{
  if (buf) {
    buf->b_p_iminsert = val;
  }
}

/// Set the b_p_imsearch field on a buffer.
void nvim_buf_set_b_p_imsearch(buf_T *buf, int val)
{
  if (buf) {
    buf->b_p_imsearch = val;
  }
}

/// Get the global p_hid option (hidden buffers).
int nvim_get_p_hid(void)
{
  return p_hid;
}

/// Get the cmdmod.cmod_flags field.
int nvim_get_cmdmod_cmod_flags(void)
{
  return cmdmod.cmod_flags;
}

/// Get the b_chartab field from a buffer.
uint64_t *nvim_buf_get_chartab(buf_T *buf)
{
  return buf->b_chartab;
}

/// Get the 'tabstop' option value for a buffer.
OptInt nvim_buf_get_p_ts(buf_T *buf)
{
  return buf->b_p_ts;
}

/// Get the 'vartabstop' array for a buffer.
int *nvim_buf_get_p_vts_array(buf_T *buf)
{
  return buf->b_p_vts_array;
}

/// Get the 'shiftwidth' option value for a buffer.
OptInt nvim_buf_get_p_sw(buf_T *buf)
{
  return buf->b_p_sw;
}

/// Get the b_nwindows field from a buffer (number of windows).
int nvim_buf_get_nwindows(buf_T *buf)
{
  return buf->b_nwindows;
}

/// Get the b_locked field from a buffer.
int nvim_buf_get_locked(buf_T *buf)
{
  return buf->b_locked;
}

/// Get the b_locked_split field from a buffer.
int nvim_buf_get_locked_split(buf_T *buf)
{
  return buf->b_locked_split;
}

/// Get the b_flags field from a buffer.
int nvim_buf_get_flags(buf_T *buf)
{
  return buf->b_flags;
}

/// Get the b_changed field from a buffer.
int nvim_buf_get_changed(buf_T *buf)
{
  return buf->b_changed;
}

/// Get the b_p_bl (buflisted option) field from a buffer.
int nvim_buf_get_b_p_bl(buf_T *buf)
{
  return buf->b_p_bl;
}

/// Get the b_ffname field from a buffer (full filename) - for Rust.
const char *nvim_buf_get_ffname(buf_T *buf)
{
  return buf->b_ffname;
}

/// Get the b_sfname field from a buffer (short filename) - for Rust.
const char *nvim_buf_get_sfname(buf_T *buf)
{
  return buf->b_sfname;
}

/// Get curbuf->b_ffname (full filename) - accessor for Rust.
const char *nvim_curbuf_get_ffname(void)
{
  return curbuf->b_ffname;
}

/// Get curbuf->b_p_path (buffer-local 'path' option) - accessor for Rust.
const char *nvim_curbuf_get_path(void)
{
  return curbuf->b_p_path;
}

/// Get curbuf->b_p_inex (buffer-local 'includeexpr' option) - accessor for Rust.
const char *nvim_curbuf_get_inex(void)
{
  return curbuf->b_p_inex;
}

/// Get the NameBuff global (accessor for Rust).
char *nvim_get_namebuff(void)
{
  return NameBuff;
}

/// Get the 'softtabstop' option value for a buffer.
OptInt nvim_buf_get_p_sts(buf_T *buf)
{
  return buf ? buf->b_p_sts : 0;
}

/// Get pointer to current line in current buffer (accessor for Rust).
const char *nvim_curbuf_get_line_ptr(void)
{
  return ml_get_buf(curbuf, curwin->w_cursor.lnum);
}

/// Get pointer to line at lnum in current buffer (accessor for Rust).
const char *nvim_curbuf_get_line_at(linenr_T lnum)
{
  return ml_get(lnum);
}

/// Get pointer to line at lnum in specified buffer (accessor for Rust).
const char *nvim_buf_get_line_at(buf_T *buf, linenr_T lnum)
{
  return ml_get_buf(buf, lnum);
}

/// Get whitespace column count at start of current line (accessor for Rust).
int nvim_getwhitecols_curline(void)
{
  return (int)getwhitecols_curline();
}

/// Check if the first line of a buffer is empty (accessor for Rust).
int nvim_buf_first_line_empty(buf_T *buf)
{
  return *ml_get_buf(buf, 1) == NUL;
}

/// Get translated "[No Name]" string (accessor for Rust).
const char *nvim_no_name_msg(void)
{
  return _("[No Name]");
}

/// Get translated E382 error message string (accessor for Rust).
const char *nvim_e382_msg(void)
{
  return _("E382: Cannot write, 'buftype' option is set");
}

/// Emit E84 error message (accessor for Rust).
void nvim_emsg_e84(void) { emsg(_("E84: No modified buffer found")); }

/// Emit E85 error message (accessor for Rust).
void nvim_emsg_e85(void) { emsg(_("E85: There is no listed buffer")); }

/// Emit E87 error message (accessor for Rust).
void nvim_emsg_e87(void) { emsg(_("E87: Cannot go beyond last buffer")); }

/// Emit E88 error message (accessor for Rust).
void nvim_emsg_e88(void) { emsg(_("E88: Cannot go before first buffer")); }

/// Emit e_nobufnr error with a count (accessor for Rust).
void nvim_semsg_e_nobufnr(int64_t count) { semsg(_(e_nobufnr), count); }

/// Check if the memfile pointer is NULL for a buffer (accessor for Rust).
int nvim_buf_get_ml_mfp_null(buf_T *buf)
{
  return buf->b_ml.ml_mfp == NULL;
}

/// Compare two file paths (case-sensitive or not depending on platform).
/// Returns 0 if equal, non-zero otherwise (accessor for Rust).
int nvim_path_fnamecmp(const char *a, const char *b)
{
  return path_fnamecmp(a, b);
}

/// Get file identity for a path (accessor for Rust).
/// Returns true if successful. The file_id buffer must be at least sizeof(FileID) bytes.
bool nvim_os_fileid(const char *path, void *file_id_out)
{
  return os_fileid(path, (FileID *)file_id_out);
}

/// Compare two file identities (accessor for Rust).
/// Each buffer must be at least sizeof(FileID) bytes.
bool nvim_os_fileid_equal(const void *a, const void *b)
{
  return os_fileid_equal((const FileID *)a, (const FileID *)b);
}

// Rust uses a 16-byte buffer to hold FileID; assert this is sufficient.
_Static_assert(sizeof(FileID) <= 16, "FileID size exceeds Rust FILE_ID_SIZE");

/// Check if buffer has a valid cached file_id (accessor for Rust).
int nvim_buf_file_id_valid(buf_T *buf)
{
  return buf->file_id_valid;
}

/// Copy buffer's cached file_id into output buffer (accessor for Rust).
void nvim_buf_get_file_id(buf_T *buf, void *out)
{
  *(FileID *)out = buf->file_id;
}

/// Set buffer's file_id from a FileID and validity flag (accessor for Rust).
void nvim_buf_set_file_id_data(buf_T *buf, const void *file_id, bool valid)
{
  if (valid) {
    buf->file_id = *(const FileID *)file_id;
  }
  buf->file_id_valid = valid;
}

/// Find a buffer by its number (accessor for Rust).
buf_T *nvim_buflist_findnr(int fnum)
{
  return buflist_findnr(fnum);
}

/// Get the stored line number for a buffer (accessor for Rust).
linenr_T nvim_buflist_findlnum(buf_T *buf)
{
  return buflist_findfmark(buf)->mark.lnum;
}

/// Get the quickfix stack buffer number (accessor for Rust).
int nvim_qf_stack_get_bufnr(void)
{
  return qf_stack_get_bufnr();
}

/// Get translated "[Quickfix List]" string (accessor for Rust).
const char *nvim_msg_qflist(void)
{
  return _(msg_qflist);
}

/// Get translated "[Location List]" string (accessor for Rust).
const char *nvim_msg_loclist(void)
{
  return _(msg_loclist);
}

/// Get the cmdwin_buf global (accessor for Rust).
buf_T *nvim_get_cmdwin_buf(void)
{
  return cmdwin_buf;
}

/// Get translated "[Command Line]" string (accessor for Rust).
const char *nvim_msg_command_line(void)
{
  return _("[Command Line]");
}

/// Get translated "[Prompt]" string (accessor for Rust).
const char *nvim_msg_prompt(void)
{
  return _("[Prompt]");
}

/// Get translated "[Scratch]" string (accessor for Rust).
const char *nvim_msg_scratch(void)
{
  return _("[Scratch]");
}

/// Get translated "E23: No alternate file" string (accessor for Rust).
const char *nvim_e_noalt(void)
{
  return _(e_noalt);
}

/// Get ARGCOUNT value (accessor for Rust).
int nvim_get_argcount(void)
{
  return ARGCOUNT;
}

/// Get translated " ((%d) of %d)" format string (accessor for Rust).
const char *nvim_msg_arg_number_invalid(void)
{
  return _(" ((%d) of %d)");
}

/// Get translated " (%d of %d)" format string (accessor for Rust).
const char *nvim_msg_arg_number(void)
{
  return _(" (%d of %d)");
}

/// Get translated "All" string (accessor for Rust).
const char *nvim_msg_all(void)
{
  return _("All");
}

/// Get translated "Top" string (accessor for Rust).
const char *nvim_msg_top(void)
{
  return _("Top");
}

/// Get translated "Bot" string (accessor for Rust).
const char *nvim_msg_bot(void)
{
  return _("Bot");
}

/// Get translated "%d%%" format string (accessor for Rust).
const char *nvim_msg_pct(void)
{
  return _("%d%%");
}

/// Get translated "%3s" format string (accessor for Rust).
const char *nvim_msg_3s(void)
{
  return _("%3s");
}

/// Set b_ml.ml_line_count on a buffer (accessor for Rust).
void nvim_buf_set_ml_line_count(buf_T *buf, linenr_T val)
{
  buf->b_ml.ml_line_count = val;
}

/// Set b_ml.ml_mfp to NULL on a buffer (accessor for Rust).
void nvim_buf_set_ml_mfp_null(buf_T *buf)
{
  buf->b_ml.ml_mfp = NULL;
}

// nvim_buf_set_ml_flags already defined in memline.c

/// Expand filename to full path (accessor for Rust).
char *nvim_fix_fname(const char *fname)
{
  return fix_fname(fname);
}

/// Create a new buffer in the buffer list (accessor for Rust).
buf_T *nvim_buflist_new(char *ffname, char *sfname, linenr_T lnum, int flags)
{
  return buflist_new(ffname, sfname, lnum, flags);
}

/// Get buf_get_changedtick value (direct accessor for Rust, avoids API function).
int64_t nvim_buf_get_changedtick_direct(buf_T *buf)
{
  return buf_get_changedtick(buf);
}

/// Set b_p_eof on a buffer (accessor for Rust).
void nvim_buf_set_p_eof(buf_T *buf, int val)
{
  buf->b_p_eof = val;
}

/// Set b_start_eof on a buffer (accessor for Rust).
void nvim_buf_set_start_eof(buf_T *buf, int val)
{
  buf->b_start_eof = val;
}

/// Set b_p_eol on a buffer (accessor for Rust).
void nvim_buf_set_p_eol(buf_T *buf, int val)
{
  buf->b_p_eol = val;
}

/// Set b_start_eol on a buffer (accessor for Rust).
void nvim_buf_set_start_eol(buf_T *buf, int val)
{
  buf->b_start_eol = val;
}

/// Set b_p_bomb on a buffer (accessor for Rust).
void nvim_buf_set_p_bomb(buf_T *buf, int val)
{
  buf->b_p_bomb = val;
}

/// Set b_start_bomb on a buffer (accessor for Rust).
void nvim_buf_set_start_bomb(buf_T *buf, int val)
{
  buf->b_start_bomb = val;
}

// ============================================================
// Phase 2 accessor functions for buffer lookup helpers.
// ============================================================

/// Get alternate file number for the current window (accessor for Rust).
int nvim_curwin_get_alt_fnum(void)
{
  return curwin->w_alt_fnum;
}

/// Look up a buffer by handle number (accessor for Rust).
buf_T *nvim_handle_get_buffer(handle_T handle)
{
  return handle_get_buffer(handle);
}

// nvim_FullName_save already defined in undo.c

/// Home-replace a filename, returning an allocated string (accessor for Rust).
char *nvim_home_replace_save(buf_T *buf, const char *src)
{
  return home_replace_save(buf, src);
}

// ============================================================
// Phase 4 accessor functions for buffer display & info helpers.
// ============================================================

/// Call buflist_setfpos (accessor for Rust).
void nvim_buflist_setfpos(buf_T *buf, win_T *win, linenr_T lnum, colnr_T col,
                          bool copy_options)
{
  buflist_setfpos(buf, win, lnum, col, copy_options);
}

/// Get stored lnum from buflist_findfmark (accessor for Rust).
linenr_T nvim_buflist_findfmark_lnum(buf_T *buf)
{
  return buflist_findfmark(buf)->mark.lnum;
}

/// Set b_p_bl on a buffer (accessor for Rust).
void nvim_buf_set_b_p_bl(buf_T *buf, int val)
{
  buf->b_p_bl = val;
}

/// Call apply_autocmds with EVENT_BUFADD (accessor for Rust).
bool nvim_apply_autocmds_bufadd(buf_T *buf)
{
  return apply_autocmds(EVENT_BUFADD, NULL, NULL, false, buf);
}

/// Call apply_autocmds with EVENT_BUFDELETE (accessor for Rust).
bool nvim_apply_autocmds_bufdelete(buf_T *buf)
{
  return apply_autocmds(EVENT_BUFDELETE, NULL, NULL, false, buf);
}

/// Call apply_autocmds with EVENT_QUITPRE for the given buffer (accessor for Rust).
void nvim_docmd_apply_autocmds_quitpre(buf_T *buf)
{
  apply_autocmds(EVENT_QUITPRE, NULL, NULL, false, buf);
}

/// Call apply_autocmds with EVENT_EXITPRE for curbuf (accessor for Rust).
void nvim_docmd_apply_autocmds_exitpre(void)
{
  apply_autocmds(EVENT_EXITPRE, NULL, NULL, false, curbuf);
}

// ============================================================
// Phase 4 accessor functions for buflist_list.
// ============================================================

// nvim_get_firstbuf is already defined in undo.c.
// nvim_buf_get_next is already defined in undo.c.
// nvim_msg_ext_set_kind is already defined in change_ffi.c.
// nvim_eap_get_arg is already defined in ex_docmd.c.
// nvim_eap_get_forceit is already defined in indent_ffi.c.

/// Get buf->b_last_used (accessor for Rust).
int64_t nvim_buf_get_last_used(buf_T *buf)
{
  return buf ? (int64_t)buf->b_last_used : 0;
}

/// Check if the buffer's terminal is running (accessor for Rust).
int nvim_buf_terminal_running(buf_T *buf)
{
  if (!buf || !buf->terminal) {
    return 0;
  }
  return terminal_running(buf->terminal) ? 1 : 0;
}

/// Check if buf's channel job is running (accessor for Rust).
int nvim_buf_channel_job_running(buf_T *buf)
{
  if (!buf || !buf->terminal) {
    return 0;
  }
  return channel_job_running((uint64_t)buf->b_p_channel) ? 1 : 0;
}

/// Write formatted time to buf (for buflist display) (accessor for Rust).
void nvim_undo_fmt_time(char *buf, size_t buflen, int64_t last_used)
{
  undo_fmt_time(buf, buflen, (time_t)last_used);
}

// nvim_get_iobuff is already defined in option_shim.c.

/// Get translated "line %" PRId64 format string (accessor for Rust).
const char *nvim_buflist_line_fmt(void)
{
  return _("line %" PRId64);
}

// ============================================================
// Phase 3 accessor functions for fileinfo.
// NOTE: nvim_get_p_ru is already defined in drawscreen.c.
// ============================================================

/// Get msg_scroll value (accessor for Rust).
int nvim_msg_scroll_get(void)
{
  return msg_scroll;
}

/// Set msg_scroll value (accessor for Rust).
void nvim_msg_scroll_set(int val)
{
  msg_scroll = val;
}

/// Get restart_edit value (accessor for Rust).
int nvim_restart_edit_get(void)
{
  return restart_edit;
}

/// Get msg_scrolled value (accessor for Rust).
int nvim_msg_scrolled_get(void)
{
  return msg_scrolled;
}

/// Get need_wait_return value (accessor for Rust).
bool nvim_need_wait_return_get(void)
{
  return need_wait_return;
}

/// Call msg() with a string and hl_id (accessor for Rust).
bool nvim_msg_call(const char *s, int hl_id)
{
  return msg(s, hl_id);
}

/// Call msg_trunc() (accessor for Rust).
char *nvim_msg_trunc(char *s, bool force, int hl_id)
{
  return msg_trunc(s, force, hl_id);
}

/// Call set_keep_msg() with hl_id=0 (accessor for Rust).
void nvim_set_keep_msg(const char *s)
{
  set_keep_msg(s, 0);
}

/// home_replace() wrapper for Rust - replaces home dir in name, writing into dst.
/// Returns number of bytes written (excluding NUL).
size_t nvim_home_replace(const buf_T *buf, const char *src, char *dst, size_t dstlen, bool one)
{
  return home_replace(buf, src, dst, dstlen, one);
}

/// Get curbuf->b_fname (short filename) (accessor for Rust).
const char *nvim_curbuf_get_fname(void)
{
  return curbuf->b_fname;
}

/// Get translated plural line-count format string (accessor for Rust).
/// Returns "%" PRId64 " line --%d%%--" (singular) or "%" PRId64 " lines --%d%%--" (plural).
const char *nvim_ngettext_line_count(int64_t n)
{
  return NGETTEXT("%" PRId64 " line --%d%%--",
                  "%" PRId64 " lines --%d%%--",
                  (unsigned long)n);
}

/// Get translated "[Modified]" string (accessor for Rust).
const char *nvim_msg_modified(void)
{
  return _(" [Modified]");
}

/// Get translated "[Not edited]" string (accessor for Rust).
const char *nvim_msg_not_edited(void)
{
  return _("[Not edited]");
}

/// Get translated "[New]" string (accessor for Rust).
const char *nvim_msg_new(void)
{
  return _("[New]");
}

/// Get translated "[Read errors]" string (accessor for Rust).
const char *nvim_msg_read_errors(void)
{
  return _("[Read errors]");
}

/// Get translated "[RO]" string (accessor for Rust).
const char *nvim_msg_ro(void)
{
  return _("[RO]");
}

/// Get translated "[readonly]" string (accessor for Rust).
const char *nvim_msg_readonly(void)
{
  return _("[readonly]");
}

/// Get translated "--No lines in buffer--" string (accessor for Rust).
const char *nvim_no_lines_msg(void)
{
  return _(no_lines_msg);
}

/// Get shortmess(SHM_MOD) (accessor for Rust).
int nvim_shortmess_mod(void)
{
  return shortmess(SHM_MOD) ? 1 : 0;
}

/// Get shortmess(SHM_RO) (accessor for Rust).
int nvim_shortmess_ro(void)
{
  return shortmess(SHM_RO) ? 1 : 0;
}

/// Get translated line-position format string "line %..." (accessor for Rust).
const char *nvim_fileinfo_line_fmt(void)
{
  return _("line %" PRId64 " of %" PRId64 " --%d%%-- col ");
}

// ============================================================
// Phase 6 accessor functions for do_modelines / chk_modeline.
// ============================================================

/// Get p_mls (modelines option) value (accessor for Rust).
int nvim_get_p_mls(void)
{
  return (int)p_mls;
}

/// Push a ETYPE_MODELINE entry onto the execution stack (accessor for Rust).
void nvim_estack_push_modeline(linenr_T lnum)
{
  estack_push(ETYPE_MODELINE, "modelines", lnum);
}

/// Pop top entry from execution stack (accessor for Rust).
void nvim_estack_pop(void)
{
  estack_pop();
}

/// Call do_set(s, OPT_MODELINE|OPT_LOCAL|flags) with modeline context saved/restored.
/// This handles the secure and current_sctx save/restore internally.
int nvim_modeline_do_set(char *s, linenr_T lnum, int flags)
{
  const int secure_save = secure;
  const sctx_T save_current_sctx = current_sctx;
  current_sctx.sc_sid = SID_MODELINE;
  current_sctx.sc_seq = 0;
  current_sctx.sc_lnum = lnum;
  secure = 1;
  int retval = do_set(s, OPT_MODELINE | OPT_LOCAL | flags);
  secure = secure_save;
  current_sctx = save_current_sctx;
  return retval;
}

/// Wrapper for try_getdigits: parses digits at s, sets *vers, returns bytes consumed.
/// Returns -1 on failure (no digits parsed).
int nvim_try_getdigits(const char *s, int64_t *vers)
{
  char *p = (char *)s;
  intmax_t v = 0;
  if (!try_getdigits(&p, &v)) {
    return -1;
  }
  *vers = (int64_t)v;
  return (int)(p - s);
}

// ============================================================
// buflist_findpat Rust FFI accessor helpers (Phase 8)
// ============================================================

/// Compile a regex for buflist_findpat, returning heap-allocated regmatch_T or NULL.
void *nvim_blfp_regex_compile(const char *pat, int magic)
{
  regmatch_T *rmp = xmalloc(sizeof(regmatch_T));
  rmp->regprog = vim_regcomp((char *)pat, magic);
  return rmp;
}

/// Returns 1 if the regprog is still valid (non-NULL).
int nvim_blfp_regex_valid(void *handle)
{
  if (handle == NULL) { return 0; }
  return ((regmatch_T *)handle)->regprog != NULL ? 1 : 0;
}

/// Free a heap-allocated regmatch_T handle.
void nvim_blfp_regex_free(void *handle)
{
  if (handle == NULL) { return; }
  vim_regfree(((regmatch_T *)handle)->regprog);
  xfree(handle);
}

/// Error message E93: More than one match for <pattern>.
void nvim_blfp_errmsg_e93(const char *pattern)
{
  semsg(_("E93: More than one match for %s"), pattern);
}

/// Error message E94: No matching buffer for <pattern>.
void nvim_blfp_errmsg_e94(const char *pattern)
{
  semsg(_("E94: No matching buffer for %s"), pattern);
}

// ============================================================
// Phase 5 accessor functions for ExpandBufnames.
// ============================================================

/// Check if pattern should use fuzzy matching (accessor for Rust).
int nvim_cmdline_fuzzy_complete(const char *pat)
{
  return cmdline_fuzzy_complete(pat) ? 1 : 0;
}

/// Compile a regex pattern for buffer name matching. Returns opaque handle or NULL.
void *nvim_bufname_regex_compile(char *pat)
{
  regmatch_T *rmp = xmalloc(sizeof(regmatch_T));
  rmp->regprog = vim_regcomp(pat, RE_MAGIC);
  if (rmp->regprog == NULL) {
    xfree(rmp);
    return NULL;
  }
  return rmp;
}

/// Check if the compiled regex handle is still valid (regprog not NULL).
int nvim_bufname_regex_valid(void *handle)
{
  if (handle == NULL) {
    return 0;
  }
  return ((regmatch_T *)handle)->regprog != NULL ? 1 : 0;
}

/// Free a compiled regex handle from nvim_bufname_regex_compile.
void nvim_bufname_regex_free(void *handle)
{
  if (handle == NULL) {
    return;
  }
  regmatch_T *rmp = handle;
  vim_regfree(rmp->regprog);
  xfree(rmp);
}

/// Get curwin->w_p_diff value (accessor for Rust).
int nvim_curwin_get_p_diff(void)
{
  return curwin->w_p_diff ? 1 : 0;
}

/// Call home_replace_save() for a buffer (accessor for Rust).
/// Caller is responsible for freeing the returned string with nvim_xfree().
char *nvim_home_replace_save_buf(buf_T *buf, const char *src)
{
  return home_replace_save(buf, src);
}

/// Call fuzzymatches_to_strmatches() (accessor for Rust).
/// fuzmatch is an array of count fuzmatch_str_T items (idx: int, str: char*, score: int).
void nvim_fuzzymatches_to_strmatches(void *fuzmatch, char ***file, int count, bool escape)
{
  fuzzymatches_to_strmatches((fuzmatch_str_T *)fuzmatch, file, count, escape);
}

// Accessors for cmdwin migration
int nvim_curbuf_ml_line_count(void) { return curbuf->b_ml.ml_line_count; }
void nvim_curbuf_set_p_ma(bool val) { curbuf->b_p_ma = val; }
void nvim_curbuf_ro_locked_inc(void) { curbuf->b_ro_locked++; }
void nvim_curbuf_ro_locked_dec(void) { curbuf->b_ro_locked--; }
void nvim_curbuf_set_p_tw(int64_t val) { curbuf->b_p_tw = val; }
/// Set current buffer's bufhidden to "wipe" (for cmdwin migration).
void nvim_cmdwin_set_bufhidden_wipe(void)
{
  set_option_value_give_err(kOptBufhidden, STATIC_CSTR_AS_OPTVAL("wipe"), OPT_LOCAL);
}
/// Set current buffer's filetype to "vim" (for cmdwin migration).
void nvim_cmdwin_set_filetype_vim(void)
{
  set_option_value_give_err(kOptFiletype, STATIC_CSTR_AS_OPTVAL("vim"), OPT_LOCAL);
}

bool nvim_get_curbuf_b_u_synced(void) { return curbuf->b_u_synced; }
bool nvim_curbuf_has_b_p_fex(void) { return *curbuf->b_p_fex != NUL; }

// =============================================================================
// buf_T option field offset table (moved from option_shim.c)
// =============================================================================

// Fill offset table for buf_T option fields indexed by OptIndex.
// Writes offsetof(buf_T, field) into out[idx] for each handled OptIndex.
// Unhandled indices receive -1 (sentinel). len must equal kOptCount.
void nvim_buf_opt_field_offsets(ptrdiff_t *out, int len)
{
  // Initialize all to -1 (unhandled)
  for (int i = 0; i < len; i++) {
    out[i] = -1;
  }
  // global-local string options
  out[kOptEqualprg]      = offsetof(buf_T, b_p_ep);
  out[kOptKeywordprg]    = offsetof(buf_T, b_p_kp);
  out[kOptPath]          = offsetof(buf_T, b_p_path);
  out[kOptTags]          = offsetof(buf_T, b_p_tags);
  out[kOptTagcase]       = offsetof(buf_T, b_p_tc);
  out[kOptBackupcopy]    = offsetof(buf_T, b_p_bkc);
  out[kOptDefine]        = offsetof(buf_T, b_p_def);
  out[kOptInclude]       = offsetof(buf_T, b_p_inc);
  out[kOptCompleteopt]   = offsetof(buf_T, b_p_cot);
  out[kOptDictionary]    = offsetof(buf_T, b_p_dict);
  out[kOptDiffanchors]   = offsetof(buf_T, b_p_dia);
  out[kOptThesaurus]     = offsetof(buf_T, b_p_tsr);
  out[kOptThesaurusfunc] = offsetof(buf_T, b_p_tsrfu);
  out[kOptFormatprg]     = offsetof(buf_T, b_p_fp);
  out[kOptFindfunc]      = offsetof(buf_T, b_p_ffu);
  out[kOptErrorformat]   = offsetof(buf_T, b_p_efm);
  out[kOptGrepformat]    = offsetof(buf_T, b_p_gefm);
  out[kOptGrepprg]       = offsetof(buf_T, b_p_gp);
  out[kOptMakeprg]       = offsetof(buf_T, b_p_mp);
  out[kOptLispwords]     = offsetof(buf_T, b_p_lw);
  out[kOptMakeencoding]  = offsetof(buf_T, b_p_menc);
  // global-local numeric options
  out[kOptAutocomplete]  = offsetof(buf_T, b_p_ac);
  out[kOptAutoread]      = offsetof(buf_T, b_p_ar);
  out[kOptUndolevels]    = offsetof(buf_T, b_p_ul);
  // buf-local options (non-global-local)
  out[kOptAutoindent]    = offsetof(buf_T, b_p_ai);
  out[kOptBinary]        = offsetof(buf_T, b_p_bin);
  out[kOptBomb]          = offsetof(buf_T, b_p_bomb);
  out[kOptBufhidden]     = offsetof(buf_T, b_p_bh);
  out[kOptBuftype]       = offsetof(buf_T, b_p_bt);
  out[kOptBuflisted]     = offsetof(buf_T, b_p_bl);
  out[kOptBusy]          = offsetof(buf_T, b_p_busy);
  out[kOptChannel]       = offsetof(buf_T, b_p_channel);
  out[kOptCopyindent]    = offsetof(buf_T, b_p_ci);
  out[kOptCindent]       = offsetof(buf_T, b_p_cin);
  out[kOptCinkeys]       = offsetof(buf_T, b_p_cink);
  out[kOptCinoptions]    = offsetof(buf_T, b_p_cino);
  out[kOptCinscopedecls] = offsetof(buf_T, b_p_cinsd);
  out[kOptCinwords]      = offsetof(buf_T, b_p_cinw);
  out[kOptComments]      = offsetof(buf_T, b_p_com);
  out[kOptCommentstring] = offsetof(buf_T, b_p_cms);
  out[kOptComplete]      = offsetof(buf_T, b_p_cpt);
#ifdef BACKSLASH_IN_FILENAME
  out[kOptCompleteslash] = offsetof(buf_T, b_p_csl);
#endif
  out[kOptCompletefunc]  = offsetof(buf_T, b_p_cfu);
  out[kOptOmnifunc]      = offsetof(buf_T, b_p_ofu);
  out[kOptEndoffile]     = offsetof(buf_T, b_p_eof);
  out[kOptEndofline]     = offsetof(buf_T, b_p_eol);
  out[kOptFixendofline]  = offsetof(buf_T, b_p_fixeol);
  out[kOptExpandtab]     = offsetof(buf_T, b_p_et);
  out[kOptFileencoding]  = offsetof(buf_T, b_p_fenc);
  out[kOptFileformat]    = offsetof(buf_T, b_p_ff);
  out[kOptFiletype]      = offsetof(buf_T, b_p_ft);
  out[kOptFormatoptions] = offsetof(buf_T, b_p_fo);
  out[kOptFormatlistpat] = offsetof(buf_T, b_p_flp);
  out[kOptIminsert]      = offsetof(buf_T, b_p_iminsert);
  out[kOptImsearch]      = offsetof(buf_T, b_p_imsearch);
  out[kOptInfercase]     = offsetof(buf_T, b_p_inf);
  out[kOptIskeyword]     = offsetof(buf_T, b_p_isk);
  out[kOptIncludeexpr]   = offsetof(buf_T, b_p_inex);
  out[kOptIndentexpr]    = offsetof(buf_T, b_p_inde);
  out[kOptIndentkeys]    = offsetof(buf_T, b_p_indk);
  out[kOptFormatexpr]    = offsetof(buf_T, b_p_fex);
  out[kOptLisp]          = offsetof(buf_T, b_p_lisp);
  out[kOptLispoptions]   = offsetof(buf_T, b_p_lop);
  out[kOptModeline]      = offsetof(buf_T, b_p_ml);
  out[kOptMatchpairs]    = offsetof(buf_T, b_p_mps);
  out[kOptModifiable]    = offsetof(buf_T, b_p_ma);
  out[kOptModified]      = offsetof(buf_T, b_changed);
  out[kOptNrformats]     = offsetof(buf_T, b_p_nf);
  out[kOptPreserveindent]= offsetof(buf_T, b_p_pi);
  out[kOptQuoteescape]   = offsetof(buf_T, b_p_qe);
  out[kOptReadonly]      = offsetof(buf_T, b_p_ro);
  out[kOptScrollback]    = offsetof(buf_T, b_p_scbk);
  out[kOptSmartindent]   = offsetof(buf_T, b_p_si);
  out[kOptSofttabstop]   = offsetof(buf_T, b_p_sts);
  out[kOptSuffixesadd]   = offsetof(buf_T, b_p_sua);
  out[kOptSwapfile]      = offsetof(buf_T, b_p_swf);
  out[kOptSynmaxcol]     = offsetof(buf_T, b_p_smc);
  out[kOptSyntax]        = offsetof(buf_T, b_p_syn);
  out[kOptShiftwidth]    = offsetof(buf_T, b_p_sw);
  out[kOptTagfunc]       = offsetof(buf_T, b_p_tfu);
  out[kOptTabstop]       = offsetof(buf_T, b_p_ts);
  out[kOptTextwidth]     = offsetof(buf_T, b_p_tw);
  out[kOptUndofile]      = offsetof(buf_T, b_p_udf);
  out[kOptWrapmargin]    = offsetof(buf_T, b_p_wm);
  out[kOptVarsofttabstop]= offsetof(buf_T, b_p_vsts);
  out[kOptVartabstop]    = offsetof(buf_T, b_p_vts);
  out[kOptKeymap]        = offsetof(buf_T, b_p_keymap);
}

// =============================================================================
// buf_copy_options accessors (moved from option_shim.c Phase 11)
// =============================================================================

/// Returns buf->b_p_initialized.
int nvim_buf_get_b_p_initialized(buf_T *buf) { return buf->b_p_initialized ? 1 : 0; }
/// Sets buf->b_p_initialized.
void nvim_buf_set_b_p_initialized(buf_T *buf, int val) { buf->b_p_initialized = val != 0; }

/// Returns buf->b_help.
int nvim_buf_get_b_help(buf_T *buf) { return buf->b_help ? 1 : 0; }
/// Sets buf->b_help.
void nvim_buf_set_b_help(buf_T *buf, int val) { buf->b_help = val != 0; }

/// CLEAR_FIELD(buf->b_p_script_ctx) -- zeroes the script_ctx array for the buffer.
void nvim_buf_clear_b_p_script_ctx(buf_T *buf) { CLEAR_FIELD(buf->b_p_script_ctx); }

/// Returns 1 if buf->b_p_bt[0] == 'h' (help buftype), else 0.
int nvim_buf_get_b_p_bt_is_help(buf_T *buf)
{
  return (buf->b_p_bt && buf->b_p_bt[0] == 'h') ? 1 : 0;
}

/// Saves b_p_isk pointer and NULLs the field.
char *nvim_buf_save_and_clear_b_p_isk(buf_T *buf)
{
  char *saved = buf->b_p_isk;
  buf->b_p_isk = NULL;
  return saved;
}

/// Restores b_p_isk from a previously saved pointer.
void nvim_buf_restore_b_p_isk(buf_T *buf, char *saved) { buf->b_p_isk = saved; }

/// buf->b_p_ro = false
void nvim_buf_clear_b_p_ro(buf_T *buf) { buf->b_p_ro = false; }

/// compile_cap_prog(&buf->b_s)
void nvim_call_compile_cap_prog_buf(buf_T *buf) { compile_cap_prog(&buf->b_s); }

/// tabstop_set(str, &buf->b_p_vsts_array)
void nvim_call_tabstop_set_vsts(buf_T *buf, const char *str) { tabstop_set(str, &buf->b_p_vsts_array); }
/// tabstop_set(str, &buf->b_p_vts_array)
void nvim_call_tabstop_set_vts(buf_T *buf, const char *str) { tabstop_set(str, &buf->b_p_vts_array); }

/// Returns buf->b_p_vts_array (for null check)
int nvim_buf_get_b_p_vts_array_is_null(buf_T *buf) { return buf->b_p_vts_array == NULL ? 1 : 0; }

/// buf->b_kmap_state |= KEYMAP_INIT
void nvim_buf_kmap_state_set_init(buf_T *buf) { buf->b_kmap_state |= KEYMAP_INIT; }

// Generic helpers for offset-based buf_T field writes (used by bufcopy.rs):

/// Set a buf_T char* field at the given byte offset to xstrdup(s).
void nvim_buf_set_string_field(buf_T *buf, ptrdiff_t offset, const char *s)
{
  char **field = (char **)(((char *)buf) + offset);
  *field = xstrdup(s);
}

/// Set a buf_T char* field at the given byte offset to empty_string_option.
void nvim_buf_empty_string_field(buf_T *buf, ptrdiff_t offset)
{
  char **field = (char **)(((char *)buf) + offset);
  *field = empty_string_option;
}

/// Generic buf_T bool field setter: writes 0 or 1 via byte offset.
void nvim_buf_set_bool_field(buf_T *buf, ptrdiff_t offset, int val)
{
  *(int *)(((char *)buf) + offset) = val != 0;
}

/// Generic buf_T OptInt field setter: writes OptInt value via byte offset.
void nvim_buf_set_optint_field(buf_T *buf, ptrdiff_t offset, OptInt val)
{
  *(OptInt *)(((char *)buf) + offset) = val;
}

/// Generic buf_T OptInt field getter: reads OptInt value via byte offset.
OptInt nvim_buf_get_optint_field(buf_T *buf, ptrdiff_t offset) { return *(OptInt *)(((char *)buf) + offset); }
/// Generic buf_T bool field getter: reads bool field via byte offset (returns 0 or 1).
int nvim_buf_get_bool_field(buf_T *buf, ptrdiff_t offset) { return (int)(*(bool *)(((char *)buf) + offset)); }
/// Sets buf->b_p_fenc = xstrdup(p_fenc).
void nvim_buf_set_b_p_fenc_dup(buf_T *buf) { buf->b_p_fenc = xstrdup(p_fenc); }

/// Sets buf->b_p_bh = empty_string_option.
void nvim_buf_set_b_p_bh_empty(buf_T *buf) { buf->b_p_bh = empty_string_option; }
/// Sets buf->b_p_bt = empty_string_option.
void nvim_buf_set_b_p_bt_empty(buf_T *buf) { buf->b_p_bt = empty_string_option; }

// Setters for global-local fields that default to "no local value":
void nvim_buf_set_b_p_ac_minus1(buf_T *buf) { buf->b_p_ac = -1; }
void nvim_buf_set_b_p_ar_minus1(buf_T *buf) { buf->b_p_ar = -1; }
void nvim_buf_set_b_p_ul_no_local(buf_T *buf) { buf->b_p_ul = NO_LOCAL_UNDOLEVEL; }
// These also zero flag fields, so they cannot be replaced by the generic helper:
void nvim_buf_set_b_p_bkc_empty(buf_T *buf) { buf->b_p_bkc = empty_string_option; buf->b_bkc_flags = 0; }
void nvim_buf_set_b_p_tc_empty(buf_T *buf) { buf->b_p_tc = empty_string_option; buf->b_tc_flags = 0; }
void nvim_buf_set_b_p_cot_empty(buf_T *buf) { buf->b_p_cot = empty_string_option; buf->b_cot_flags = 0; }
/// Sets buf->b_s.b_syn_isk = empty_string_option.
void nvim_buf_set_b_s_syn_isk_empty(buf_T *buf) { buf->b_s.b_syn_isk = empty_string_option; }

// Scalar field setters for nopaste/nobin variants:
void nvim_buf_set_b_p_ai_nopaste(buf_T *buf, int v) { buf->b_p_ai_nopaste = v != 0; }
void nvim_buf_set_b_p_tw_nopaste(buf_T *buf, OptInt v) { buf->b_p_tw_nopaste = v; }
void nvim_buf_set_b_p_tw_nobin(buf_T *buf, OptInt v) { buf->b_p_tw_nobin = v; }
void nvim_buf_set_b_p_wm_nopaste(buf_T *buf, OptInt v) { buf->b_p_wm_nopaste = v; }
void nvim_buf_set_b_p_wm_nobin(buf_T *buf, OptInt v) { buf->b_p_wm_nobin = v; }
void nvim_buf_set_b_p_et_nobin(buf_T *buf, int v) { buf->b_p_et_nobin = v != 0; }
void nvim_buf_set_b_p_et_nopaste(buf_T *buf, int v) { buf->b_p_et_nopaste = v != 0; }
void nvim_buf_set_b_p_ml_nobin(buf_T *buf, int v) { buf->b_p_ml_nobin = v != 0; }
void nvim_buf_set_b_p_sts_nopaste(buf_T *buf, OptInt v) { buf->b_p_sts_nopaste = v; }
// Per-buffer nopaste/nobin field getters (for paste restore in Rust)
int nvim_buf_get_b_p_ai_nopaste(buf_T *buf) { return (int)buf->b_p_ai_nopaste; }
OptInt nvim_buf_get_b_p_tw_nopaste(buf_T *buf) { return buf->b_p_tw_nopaste; }
OptInt nvim_buf_get_b_p_wm_nopaste(buf_T *buf) { return buf->b_p_wm_nopaste; }
OptInt nvim_buf_get_b_p_sts_nopaste(buf_T *buf) { return buf->b_p_sts_nopaste; }
int nvim_buf_get_b_p_et_nopaste(buf_T *buf) { return (int)buf->b_p_et_nopaste; }
char *nvim_buf_get_b_p_vsts(buf_T *buf) { return buf->b_p_vsts; }
char *nvim_buf_get_b_p_vsts_nopaste(buf_T *buf) { return buf->b_p_vsts_nopaste; }
void nvim_buf_set_b_p_vsts_raw(buf_T *buf, char *val) { buf->b_p_vsts = val; }
int *volatile *nvim_buf_get_b_p_vsts_array_ptr(buf_T *buf) { return (int *volatile *)&buf->b_p_vsts_array; }
void nvim_buf_set_b_p_ma(buf_T *buf, int v) { buf->b_p_ma = v != 0; }

// String field setters using xstrdup (b_s substructure fields):
void nvim_buf_set_b_p_vsts_nopaste_dup(buf_T *buf, const char *s) { buf->b_p_vsts_nopaste = s ? xstrdup(s) : NULL; }
void nvim_buf_set_b_s_spc_dup(buf_T *buf, const char *s) { buf->b_s.b_p_spc = xstrdup(s); }
void nvim_buf_set_b_s_spf_dup(buf_T *buf, const char *s) { buf->b_s.b_p_spf = xstrdup(s); }
void nvim_buf_set_b_s_spl_dup(buf_T *buf, const char *s) { buf->b_s.b_p_spl = xstrdup(s); }
void nvim_buf_set_b_s_spo_dup(buf_T *buf, const char *s) { buf->b_s.b_p_spo = xstrdup(s); }

/// Set b_s.b_p_spo_flags from global spo_flags.
void nvim_buf_set_b_s_spo_flags_from_global(buf_T *buf) { buf->b_s.b_p_spo_flags = spo_flags; }

// ============================================================================
// lasttitle/lasticon statics and accessors (moved from buffer.c, used by Rust info.rs)
// ============================================================================

static char *lasttitle = NULL;
static char *lasticon = NULL;

/// Get lasttitle static variable.
const char *nvim_buf_get_lasttitle(void) { return lasttitle; }
/// Set lasttitle static variable (caller transfers ownership of s).
void nvim_buf_set_lasttitle(char *s) { lasttitle = s; }
/// Get lasticon static variable.
const char *nvim_buf_get_lasticon(void) { return lasticon; }
/// Set lasticon static variable (caller transfers ownership of s).
void nvim_buf_set_lasticon(char *s) { lasticon = s; }

// ============================================================================
// Extmark Accessor Functions (moved from buffer.c, for Rust FFI - extmark crate)
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

// =============================================================================
// Buffer name matching (moved from buffer.c so buffer.c need not include regexp)
// =============================================================================

#include "nvim/option_vars.h"

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

/// Wrapper for buflist_match(). Returns a matched name or NULL.
const char *nvim_blfp_buflist_match(void *handle, buf_T *buf, bool ignore_case)
{
  if (handle == NULL) { return NULL; }
  return buflist_match((regmatch_T *)handle, buf, ignore_case);
}

/// Check if buffer matches the compiled regex. Returns matched name or NULL.
const char *nvim_bufname_regex_match(void *handle, buf_T *buf, bool ignore_case)
{
  if (handle == NULL) {
    return NULL;
  }
  return buflist_match((regmatch_T *)handle, buf, ignore_case);
}

// ============================================================
// WinInfo / buffer position functions (Phase 11)
// ============================================================

/// Set the last cursor position in the info for the current window in buffer "buf".
/// This is in the WinInfo list in buf->b_wininfo.
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

// ============================================================
// WinInfo lookup cluster (Phase 12)
// ============================================================

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

// ============================================================
// Buffer name / file management (Phase 13)
// ============================================================

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

// ============================================================
// Buffer file name management cluster (Phase 14)
// ============================================================

/// Find a buffer by file name and file ID. Returns NULL if not found.
buf_T *buflist_findname_file_id(char *ffname, FileID *file_id, bool file_id_valid)
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

// ============================================================
// Buffer navigation (Phase 15)
// ============================================================

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

// ============================================================
// Buffer content comparison (Phase 16)
// ============================================================

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

// ============================================================
// Open all buffers (Phase 17)
// ============================================================

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

/// Enter a new current buffer.
/// Old curbuf must have been abandoned already!  This also means "curbuf" may
/// be pointing to freed memory.
void enter_buffer(buf_T *buf)
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
