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
#include "nvim/buffer_updates.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/vars.h"
#include "nvim/hashtab.h"
#include "nvim/usercmd.h"
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
// Rust window helpers used in do_buffer_ext (Phase 21).
extern int rs_last_window(win_T *win);
// Rust window/diff helpers used in close_buffer/buf_freeall (Phase 22).
extern int rs_win_valid_any_tab(win_T *win);
extern int rs_one_window_in_tab(win_T *win, tabpage_T *tp);
extern void rs_diff_buf_delete(buf_T *buf);
extern int rs_diffopt_hiddenoff(void);
extern int rs_buf_effective_action(buf_T *buf, int action);
// buffer.c non-static helpers for close_buffer/buf_freeall cluster (Phase 22).
extern void free_buffer(buf_T *buf);
extern void clear_wininfo(buf_T *buf);
extern void free_buffer_stuff(buf_T *buf, int free_flags);
// Rust buffer-lifecycle helpers used in do_buffer_ext (Phase 21).
extern buf_T *rs_find_and_validate_buffer(int action, int start, int dir, int count, int flags,
                                          int unload);
extern buf_T *rs_find_buffer_for_delete(int buf_fnum, int *update_jumplist);

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

/// Perform the body of buf_set_name: free old names, set new ffname, expand paths.
/// Rust calls rs_buflist_findnr + this to implement buf_set_name.
void nvim_buf_set_name_body(buf_T *buf, char *name)
{
  if (buf->b_sfname != buf->b_ffname) {
    xfree(buf->b_sfname);
  }
  xfree(buf->b_ffname);
  buf->b_ffname = xstrdup(name);
  buf->b_sfname = NULL;
  fname_expand(buf, &buf->b_ffname, &buf->b_sfname);
  buf->b_fname = buf->b_sfname;
}

/// Call check_arg_idx(curwin) if curwin->w_buffer == buf.
void nvim_check_arg_idx_if_curbuf(buf_T *buf)
{
  if (curwin->w_buffer == buf) {
    check_arg_idx(curwin);
  }
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

// ============================================================
// WinInfo / buffer position functions (Phase 11)
// ============================================================

// ============================================================
// WinInfo FFI accessor helpers (for Rust wininfo.rs)
// ============================================================

size_t nvim_buf_wininfo_count(buf_T *buf) { return kv_size(buf->b_wininfo); }
WinInfo *nvim_buf_wininfo_get(buf_T *buf, size_t i) { return kv_A(buf->b_wininfo, i); }
win_T *nvim_wininfo_get_win(WinInfo *wip) { return wip->wi_win; }
bool nvim_wininfo_get_optset(WinInfo *wip) { return wip->wi_optset; }
bool nvim_wininfo_get_wo_diff(WinInfo *wip) { return wip->wi_opt.wo_diff; }
int nvim_wininfo_get_changelistidx(WinInfo *wip) { return wip->wi_changelistidx; }
fmark_T *nvim_wininfo_get_mark_ptr(WinInfo *wip) { return &wip->wi_mark; }
bool nvim_wininfo_get_fold_manual(WinInfo *wip) { return wip->wi_fold_manual; }
garray_T *nvim_wininfo_get_folds_ptr(WinInfo *wip) { return &wip->wi_folds; }
garray_T *nvim_win_get_folds_ptr(win_T *wp) { return &wp->w_folds; }
bool nvim_win_get_fold_manual(win_T *wp) { return wp->w_fold_manual; }

/// Returns true if wip->wi_win is a window in the current tab page.
bool nvim_wininfo_win_in_curtab(WinInfo *wip)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (wip->wi_win == wp) {
      return true;
    }
  }
  return false;
}

/// Find and detach a WinInfo entry for win from buf->b_wininfo.
/// If found: shifts the entry out of the vector and returns it
///   (if copy_options && wi_optset: clears wi_opt and folds first).
/// If not found: allocates a new WinInfo with wi_win=win and lnum forced to 1 if lnum==0.
/// Returns non-NULL always.
WinInfo *nvim_buf_wininfo_find_and_detach(buf_T *buf, win_T *win, bool copy_options,
                                          linenr_T *lnum_inout)
{
  for (size_t i = 0; i < kv_size(buf->b_wininfo); i++) {
    WinInfo *wip = kv_A(buf->b_wininfo, i);
    if (wip->wi_win == win) {
      kv_shift(buf->b_wininfo, i, 1);
      if (copy_options && wip->wi_optset) {
        clear_winopt(&wip->wi_opt);
        deleteFoldRecurse(buf, &wip->wi_folds);
      }
      return wip;
    }
  }
  // Not found: allocate new entry
  WinInfo *wip = xcalloc(1, sizeof(WinInfo));
  wip->wi_win = win;
  if (*lnum_inout == 0) {
    *lnum_inout = 1;
  }
  return wip;
}

/// Prepend wip to buf->b_wininfo (push to front of vector).
void nvim_buf_wininfo_prepend(buf_T *buf, WinInfo *wip)
{
  kv_pushp(buf->b_wininfo);
  memmove(&kv_A(buf->b_wininfo, 1), &kv_A(buf->b_wininfo, 0),
          (kv_size(buf->b_wininfo) - 1) * sizeof(kv_A(buf->b_wininfo, 0)));
  kv_A(buf->b_wininfo, 0) = wip;
}

/// Set wi_mark on wip from lnum/col, and compute view if win != NULL.
void nvim_wininfo_set_mark(WinInfo *wip, linenr_T lnum, colnr_T col, win_T *win)
{
  wip->wi_mark.mark.lnum = lnum;
  wip->wi_mark.mark.col = col;
  if (win != NULL) {
    wip->wi_mark.view = mark_view_make(win->w_topline, wip->wi_mark.mark);
  }
}

/// Copy window options from win into wip, set fold_manual and clone folds.
void nvim_wininfo_copy_from_win(WinInfo *wip, win_T *win)
{
  copy_winopt(&win->w_onebuf_opt, &wip->wi_opt);
  wip->wi_fold_manual = win->w_fold_manual;
  rs_cloneFoldGrowArray(&win->w_folds, &wip->wi_folds);
  wip->wi_optset = true;
}

/// get_winopts: apply WinInfo wip to curwin (copy opts + folds), or copy from win.
/// Returns 0 if no matching wip (fall back to allbuf_opt),
///         1 if copied from wip->wi_win (other window with same buffer),
///         2 if copied from wip->wi_opt (saved options).
int nvim_get_winopts_apply(WinInfo *wip, buf_T *buf)
{
  if (wip == NULL) {
    copy_winopt(&curwin->w_allbuf_opt, &curwin->w_onebuf_opt);
    return 0;
  }
  if (wip->wi_win != curwin && wip->wi_win != NULL && wip->wi_win->w_buffer == buf) {
    win_T *wp = wip->wi_win;
    copy_winopt(&wp->w_onebuf_opt, &curwin->w_onebuf_opt);
    curwin->w_fold_manual = wp->w_fold_manual;
    curwin->w_foldinvalid = true;
    rs_cloneFoldGrowArray(&wp->w_folds, &curwin->w_folds);
    return 1;
  } else if (wip->wi_optset) {
    copy_winopt(&wip->wi_opt, &curwin->w_onebuf_opt);
    curwin->w_fold_manual = wip->wi_fold_manual;
    curwin->w_foldinvalid = true;
    rs_cloneFoldGrowArray(&wip->wi_folds, &curwin->w_folds);
    return 2;
  } else {
    copy_winopt(&curwin->w_allbuf_opt, &curwin->w_onebuf_opt);
    return 0;
  }
}

void nvim_clear_winopt_curwin(void) { clear_winopt(&curwin->w_onebuf_opt); }
void nvim_curwin_set_changelistidx(int val) { curwin->w_changelistidx = val; }
bool nvim_curwin_config_is_minimal(void) { return curwin->w_config.style == kWinStyleMinimal; }
int64_t nvim_get_p_fdls(void) { return p_fdls; }
void nvim_curwin_set_p_fdl(int val) { curwin->w_p_fdl = (OptInt)val; }
void nvim_didset_window_options_curwin(void) { didset_window_options(curwin, false); }
void nvim_win_set_minimal_style_curwin(void) { win_set_minimal_style(curwin); }
// nvim_get_curwin: defined in Rust window crate (window/src/globals.rs)
// nvim_win_get_changelistidx: defined in window_shim.c
void nvim_wininfo_set_changelistidx(WinInfo *wip, int val) { wip->wi_changelistidx = val; }
void nvim_wininfo_set_optset(WinInfo *wip, bool val) { wip->wi_optset = val; }
void nvim_wininfo_set_fold_manual(WinInfo *wip, bool val) { wip->wi_fold_manual = val; }

/// Get pointer to static no_position fmark_T for buflist_findfmark.
fmark_T *nvim_get_no_position_ptr(void)
{
  static fmark_T no_position = { { 1, 0, 0 }, 0, 0, { 0 }, NULL };
  return &no_position;
}

// buflist_setfpos, wininfo_other_tab_diff, find_wininfo, get_winopts, buflist_findfmark:
// migrated to Rust (src/nvim-rs/buffer/src/wininfo.rs)

/// Compound accessor: full body of buf_set_changedtick (migrated to Rust).
void nvim_buf_set_changedtick_compound(buf_T *const buf, const varnumber_T changedtick)
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



// wininfo_other_tab_diff, find_wininfo, get_winopts, buflist_findfmark:
// migrated to Rust (src/nvim-rs/buffer/src/wininfo.rs)


/// Expand ffname and sfname for "buf". Calls fname_expand().
/// Updates *ffname_ptr (allocated) and *sfname_ptr (points into expanded path).
/// Accessor for Rust setfname.
void nvim_fname_expand(buf_T *buf, char **ffname_ptr, char **sfname_ptr)
{
  fname_expand(buf, ffname_ptr, sfname_ptr);
}

/// Return true if "buf" is displayed in any window across all tabs.
/// Used by setfname to check if obuf is in use. Accessor for Rust.
bool nvim_buf_is_in_any_window(buf_T *buf)
{
  FOR_ALL_TAB_WINDOWS(tab, win) {
    if (win->w_buffer == buf) {
      return true;
    }
  }
  return false;
}

/// Free and clear b_sfname and b_ffname for "buf" (the "removing the name" branch).
/// Accessor for Rust setfname.
void nvim_buf_remove_fnames(buf_T *buf)
{
  if (buf->b_sfname != buf->b_ffname) {
    XFREE_CLEAR(buf->b_sfname);
  } else {
    buf->b_sfname = NULL;
  }
  XFREE_CLEAR(buf->b_ffname);
  buf->b_fname = buf->b_sfname;
}

/// Set b_ffname/b_sfname/b_fname for "buf", freeing any previous names.
/// "sfname" is xstrdup'd (path_fix_case applied on case-insensitive systems).
/// Accessor for Rust setfname.
void nvim_buf_set_fnames(buf_T *buf, char *ffname, char *sfname)
{
  char *sfname_copy = xstrdup(sfname);
#ifdef CASE_INSENSITIVE_FILENAME
  path_fix_case(sfname_copy);
#endif
  if (buf->b_sfname != buf->b_ffname) {
    xfree(buf->b_sfname);
  }
  xfree(buf->b_ffname);
  buf->b_ffname = ffname;
  buf->b_sfname = sfname_copy;
  buf->b_fname = buf->b_sfname;
}

/// Emit E95 "Buffer with this name already exists" error message.
/// Accessor for Rust setfname.
void nvim_emsg_e95_buffer_exists(void)
{
  emsg(_("E95: Buffer with this name already exists"));
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


// setfname migrated to Rust (src/nvim-rs/buffer/src/filename.rs)


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

// ============================================================
// Buffer loading and command dispatch (Phase 21)
// ============================================================

static const char e_attempt_to_delete_buffer_that_is_in_use_str[]
  = N_("E937: Attempt to delete a buffer that is in use: %s");

/// Emit the E937 error message for can_unload_buffer.
/// Prefers buf->b_fname, falls back to buf->b_ffname, then "[No Name]".
void nvim_emsg_e937_buf_in_use(buf_T *buf)
{
  const char *fname = buf->b_fname != NULL ? buf->b_fname : buf->b_ffname;
  semsg(_(e_attempt_to_delete_buffer_that_is_in_use_str),
        fname != NULL ? fname : "[No Name]");
}

/// Compound accessor: aucmd_prepbuf + open_buffer(false,NULL,0) + aucmd_restbuf.
/// Returns 0 on FAIL, non-zero on OK/NOTDONE (accessor for Rust buf_ensure_loaded).
int nvim_buf_aucmd_open_buffer(buf_T *buf)
{
  aco_save_T aco;
  aucmd_prepbuf(&aco, buf);
  int status = open_buffer(false, NULL, 0);
  aucmd_restbuf(&aco);
  return (status != FAIL) ? 1 : 0;
}

// buf_ensure_loaded migrated to Rust (src/nvim-rs/buffer/src/lifecycle.rs)

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
int do_buffer_ext(int action, int start, int dir, int count, int flags)
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
