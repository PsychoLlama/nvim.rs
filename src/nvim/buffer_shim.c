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
#include "nvim/file_search.h"
#include "nvim/fileio.h"
#include "nvim/fuzzy.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
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
#include "nvim/strings.h"
#include "nvim/terminal.h"
#include "nvim/types_defs.h"
#include "nvim/undo.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

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
