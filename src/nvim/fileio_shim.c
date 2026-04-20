// fileio_shim.c: C accessor shim functions for the Rust readfile() port.
//
// All functions here follow the nvim_* naming convention and are called from
// Rust via FFI. They expose global variables, struct fields, and C functions
// that Rust cannot access directly.

#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/stat.h>

#include "auto/config.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/change.h"
#include "nvim/cursor.h"
#include "nvim/diff.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_eval.h"
#include "nvim/option_vars.h"
#include "nvim/fileio.h"
#include "nvim/fold.h"
#include "nvim/getchar.h"
#include "nvim/globals.h"
#include "nvim/iconv_defs.h"
#include "nvim/macros_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memfile.h"
#include "nvim/memfile_defs.h"
#include "nvim/memline.h"
#include "nvim/memline_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/edit.h"
#include "nvim/move.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/os/fs.h"
#include "nvim/os/fs_defs.h"
#include "nvim/os/input.h"
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/path.h"
#include "nvim/pos_defs.h"
#include "nvim/regexp.h"
#include "nvim/sha256.h"
#include "nvim/shada.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/undo.h"
#include "nvim/undo_defs.h"
#include "nvim/vim_defs.h"

// =============================================================================
// Global variable accessors
// =============================================================================

int nvim_fileio_get_stdin_fd(void) { return stdin_fd; }
int nvim_fileio_get_msg_scroll(void) { return msg_scroll ? 1 : 0; }
void nvim_fileio_set_msg_scroll(int val) { msg_scroll = (val != 0); }
int nvim_fileio_get_msg_scrolled(void) { return msg_scrolled; }
int nvim_fileio_get_got_int(void) { return got_int ? 1 : 0; }
void nvim_fileio_set_need_fileinfo(int val) { need_fileinfo = (val != 0); }
int nvim_fileio_get_readonlymode(void) { return readonlymode ? 1 : 0; }
int nvim_fileio_get_recoverymode(void) { return recoverymode ? 1 : 0; }
int nvim_fileio_get_p_verbose(void) { return (int)p_verbose; }
const char *nvim_fileio_get_p_cpo(void) { return p_cpo; }
const char *nvim_fileio_get_p_ffs(void) { return p_ffs; }
const char *nvim_fileio_get_p_fencs(void) { return p_fencs; }
const char *nvim_fileio_get_p_ccv(void) { return p_ccv; }
int nvim_fileio_get_msg_listdo_overwrite(void) { return msg_listdo_overwrite ? 1 : 0; }
int nvim_fileio_get_exmode_active(void) { return exmode_active ? 1 : 0; }
int nvim_fileio_get_restart_edit(void) { return restart_edit; }
int nvim_fileio_get_need_wait_return(void) { return need_wait_return ? 1 : 0; }
int nvim_fileio_get_msg_col(void) { return msg_col; }
int nvim_fileio_get_msg_scrolled_ign(void) { return msg_scrolled_ign ? 1 : 0; }
void nvim_fileio_set_msg_scrolled_ign(int val) { msg_scrolled_ign = (val != 0); }
int nvim_fileio_get_cmdmod_lockmarks(void) {
  return (cmdmod.cmod_flags & CMOD_LOCKMARKS) != 0 ? 1 : 0;
}

// =============================================================================
// curbuf field accessors
// =============================================================================

int nvim_fileio_curbuf_get_b_au_did_filetype(void) { return curbuf->b_au_did_filetype ? 1 : 0; }
void nvim_fileio_curbuf_set_b_au_did_filetype(int val) { curbuf->b_au_did_filetype = (val != 0); }
int nvim_fileio_curbuf_get_b_no_eol_lnum(void) { return (int)curbuf->b_no_eol_lnum; }
void nvim_fileio_curbuf_set_b_no_eol_lnum(int val) { curbuf->b_no_eol_lnum = (linenr_T)val; }
char *nvim_fileio_curbuf_get_b_ffname(void) { return curbuf->b_ffname; }
char *nvim_fileio_curbuf_get_b_fname(void) { return curbuf->b_fname; }
int nvim_fileio_curbuf_get_b_flags(void) { return (int)curbuf->b_flags; }
void nvim_fileio_curbuf_set_b_flags(int val) { curbuf->b_flags = (int)val; }
void nvim_fileio_curbuf_set_b_flags_and(int mask) { curbuf->b_flags &= mask; }
void nvim_fileio_curbuf_set_b_flags_or(int flags) { curbuf->b_flags |= flags; }
int nvim_fileio_curbuf_get_b_help(void) { return curbuf->b_help ? 1 : 0; }
int nvim_fileio_curbuf_get_b_p_ro(void) { return curbuf->b_p_ro ? 1 : 0; }
void nvim_fileio_curbuf_set_b_p_ro(int val) { curbuf->b_p_ro = (val != 0); }
void nvim_fileio_curbuf_set_b_p_eof(int val) { curbuf->b_p_eof = (val != 0); }
int nvim_fileio_curbuf_get_b_p_eol(void) { return curbuf->b_p_eol ? 1 : 0; }
void nvim_fileio_curbuf_set_b_p_eol(int val) { curbuf->b_p_eol = (val != 0); }
void nvim_fileio_curbuf_set_b_start_eof(int val) { curbuf->b_start_eof = (val != 0); }
void nvim_fileio_curbuf_set_b_start_eol(int val) { curbuf->b_start_eol = (val != 0); }
int nvim_fileio_curbuf_get_b_p_bomb(void) { return curbuf->b_p_bomb ? 1 : 0; }
void nvim_fileio_curbuf_set_b_p_bomb(int val) { curbuf->b_p_bomb = (val != 0); }
void nvim_fileio_curbuf_set_b_start_bomb(int val) { curbuf->b_start_bomb = (val != 0); }
int nvim_fileio_curbuf_get_b_p_bin(void) { return curbuf->b_p_bin ? 1 : 0; }
const char *nvim_fileio_curbuf_get_b_p_fenc(void) { return curbuf->b_p_fenc; }
int nvim_fileio_curbuf_get_b_p_udf(void) { return curbuf->b_p_udf ? 1 : 0; }
int nvim_fileio_curbuf_get_b_bad_char(void) { return (int)curbuf->b_bad_char; }
void nvim_fileio_curbuf_set_b_bad_char(int val) { curbuf->b_bad_char = (char)val; }
const char *nvim_fileio_curbuf_get_b_p_ft(void) { return curbuf->b_p_ft; }
int nvim_fileio_curbuf_get_ml_line_count(void) { return (int)curbuf->b_ml.ml_line_count; }
int nvim_fileio_curbuf_get_ml_flags(void) { return (int)curbuf->b_ml.ml_flags; }
int nvim_fileio_curbuf_get_b_mtime(void) { return (int)curbuf->b_mtime; }
void nvim_fileio_curbuf_set_b_mtime(int val) { curbuf->b_mtime = (time_t)val; }
void nvim_fileio_curbuf_set_b_mtime_ns(int val) { curbuf->b_mtime_ns = (long)val; }
void nvim_fileio_curbuf_set_b_mtime_read(int val) { curbuf->b_mtime_read = (time_t)val; }
void nvim_fileio_curbuf_set_b_mtime_read_ns(int val) { curbuf->b_mtime_read_ns = (long)val; }
void nvim_fileio_curbuf_set_b_orig_size(int64_t val) { curbuf->b_orig_size = (uint64_t)val; }
void nvim_fileio_curbuf_set_b_orig_mode(int val) { curbuf->b_orig_mode = val; }
void nvim_fileio_curbuf_set_deleted_bytes_zero(void) {
  curbuf->deleted_bytes = 0;
  curbuf->deleted_bytes2 = 0;
  curbuf->deleted_codepoints = 0;
  curbuf->deleted_codeunits = 0;
}
int nvim_fileio_curbuf_has_mfp(void) { return curbuf->b_ml.ml_mfp != NULL ? 1 : 0; }
int nvim_fileio_curbuf_mfp_dirty_is_nosync(void) {
  return (curbuf->b_ml.ml_mfp != NULL &&
          curbuf->b_ml.ml_mfp->mf_dirty == MF_DIRTY_YES_NOSYNC) ? 1 : 0;
}
void nvim_fileio_curbuf_mfp_set_dirty_yes(void) {
  if (curbuf->b_ml.ml_mfp != NULL) {
    curbuf->b_ml.ml_mfp->mf_dirty = MF_DIRTY_YES;
  }
}

// op_start/op_end accessors (pos_T is {lnum: i32, col: i32, coladd: i32})
void nvim_fileio_curbuf_get_b_op_start(int *lnum, int *col) {
  *lnum = (int)curbuf->b_op_start.lnum;
  *col = (int)curbuf->b_op_start.col;
}
void nvim_fileio_curbuf_set_b_op_start(int lnum, int col) {
  curbuf->b_op_start.lnum = (linenr_T)lnum;
  curbuf->b_op_start.col = (colnr_T)col;
}
void nvim_fileio_curbuf_set_b_op_end(int lnum, int col) {
  curbuf->b_op_end.lnum = (linenr_T)lnum;
  curbuf->b_op_end.col = (colnr_T)col;
}

// =============================================================================
// curbuf identity checks (needed for autocmd safety guards)
// =============================================================================

// Returns 1 if curbuf == ptr recorded before autocmd, 0 otherwise.
// Also checks if the given string pointers have changed (b_ffname or b_fname).
int nvim_fileio_curbuf_check_identity(void *saved_curbuf_ptr,
                                       const char *saved_b_ffname,
                                       const char *saved_b_fname,
                                       int using_b_ffname,
                                       int using_b_fname) {
  if ((buf_T *)saved_curbuf_ptr != curbuf) return 1;
  if (using_b_ffname && saved_b_ffname != curbuf->b_ffname) return 1;
  if (using_b_fname && saved_b_fname != curbuf->b_fname) return 1;
  return 0;
}

void *nvim_fileio_get_curbuf(void) { return (void *)curbuf; }
void *nvim_fileio_get_curwin(void) { return (void *)curwin; }

// =============================================================================
// curwin field accessors
// =============================================================================

int nvim_fileio_curwin_cursor_lnum(void) { return (int)curwin->w_cursor.lnum; }
void nvim_fileio_curwin_set_cursor_lnum(int lnum) { curwin->w_cursor.lnum = (linenr_T)lnum; }

// =============================================================================
// os_fileinfo combined accessor for newfile path
// =============================================================================

/// Fills curbuf's file info (b_mtime, b_mtime_ns, b_orig_size, b_orig_mode)
/// from fname via os_fileinfo(). Returns swap_mode on Unix (protection bits), or 0.
/// Returns -1 if os_fileinfo failed (clears all timestamps).
int nvim_fileio_store_fileinfo_for_newfile(const char *fname) {
  FileInfo fi;
  if (!os_fileinfo(fname, &fi)) {
    curbuf->b_mtime = 0;
    curbuf->b_mtime_ns = 0;
    curbuf->b_mtime_read = 0;
    curbuf->b_mtime_read_ns = 0;
    curbuf->b_orig_size = 0;
    curbuf->b_orig_mode = 0;
    return -1;
  }
  buf_store_file_info(curbuf, &fi);
  curbuf->b_mtime_read = curbuf->b_mtime;
  curbuf->b_mtime_read_ns = curbuf->b_mtime_ns;
#ifdef UNIX
  return ((int)fi.stat.st_mode & 0644) | 0600;
#else
  return 0;
#endif
}

// =============================================================================
// file permission / type checks
// =============================================================================

int nvim_fileio_os_file_is_writable(const char *fname) { return os_file_is_writable(fname) ? 1 : 0; }
int nvim_fileio_os_open_rdonly(const char *fname) { return os_open(fname, O_RDONLY, 0); }
int nvim_fileio_after_pathsep(const char *b, const char *p) { return after_pathsep(b, p) ? 1 : 0; }
int nvim_fileio_bt_dontwrite(void) { return bt_dontwrite(curbuf) ? 1 : 0; }
int nvim_fileio_dir_of_file_exists(const char *fname) { return dir_of_file_exists(fname) ? 1 : 0; }
int nvim_fileio_is_s_isreg(int perm) { return S_ISREG(perm) ? 1 : 0; }
int nvim_fileio_is_s_isfifo(int perm) { return S_ISFIFO(perm) ? 1 : 0; }
int nvim_fileio_is_s_issock(int perm) { return S_ISSOCK(perm) ? 1 : 0; }
int nvim_fileio_is_s_isdir(int perm) { return S_ISDIR(perm) ? 1 : 0; }
int nvim_fileio_perm_is_writable(int perm) { return (perm & 0222) ? 1 : 0; }

#ifdef UNIX
int nvim_fileio_check_swap_mode_group(const char *swap_fname, int gid, int fd, int swap_mode) {
  // If group-read bit is set but not world-read bit, try to set group to match.
  if ((swap_mode & 044) == 040) {
    FileInfo swap_info;
    if (os_fileinfo(swap_fname, &swap_info)
        && (gid_t)gid != swap_info.stat.st_gid
        && os_fchown(fd, (uv_uid_t)(-1), (uv_gid_t)gid) == -1) {
      return swap_mode & 0600;
    }
  }
  return swap_mode;
}
int nvim_fileio_curbuf_swap_gid(void) {
  if (curbuf->b_ml.ml_mfp == NULL) return -1;
  FileInfo fi;
  if (!os_fileinfo(curbuf->b_ml.ml_mfp->mf_fname, &fi)) return -1;
  return (int)fi.stat.st_gid;
}
int nvim_fileio_curbuf_swap_fd(void) {
  if (curbuf->b_ml.ml_mfp == NULL) return -1;
  return curbuf->b_ml.ml_mfp->mf_fd;
}
const char *nvim_fileio_curbuf_swap_fname(void) {
  if (curbuf->b_ml.ml_mfp == NULL) return NULL;
  return curbuf->b_ml.ml_mfp->mf_fname;
}
#endif

// =============================================================================
// Autocmd firing wrappers
// =============================================================================

int nvim_fileio_apply_autocmds_exarg(int event, const char *fname, const char *fname_io,
                                      int force_it, void *buf, void *eap) {
  return apply_autocmds_exarg((event_T)event, (char *)fname, (char *)fname_io,
                               force_it != 0, (buf_T *)buf, (exarg_T *)eap) ? 1 : 0;
}
int nvim_fileio_apply_autocmds(int event, const char *pat, const char *fname,
                                 int force_it, void *buf) {
  return apply_autocmds((event_T)event, (char *)pat, (char *)fname,
                          force_it != 0, (buf_T *)buf) ? 1 : 0;
}

// =============================================================================
// Autocmd event ID constants
// =============================================================================

int nvim_event_BUFREADCMD(void) { return EVENT_BUFREADCMD; }
int nvim_event_FILEREADCMD(void) { return EVENT_FILEREADCMD; }
int nvim_event_BUFNEWFILE(void) { return EVENT_BUFNEWFILE; }
int nvim_event_FILTERREADPRE(void) { return EVENT_FILTERREADPRE; }
int nvim_event_STDINREADPRE(void) { return EVENT_STDINREADPRE; }
int nvim_event_BUFREADPRE(void) { return EVENT_BUFREADPRE; }
int nvim_event_FILEREADPRE(void) { return EVENT_FILEREADPRE; }
int nvim_event_FILTERREADPOST(void) { return EVENT_FILTERREADPOST; }
int nvim_event_BUFREADPOST(void) { return EVENT_BUFREADPOST; }
int nvim_event_FILEREADPOST(void) { return EVENT_FILEREADPOST; }
int nvim_event_FILETYPE(void) { return EVENT_FILETYPE; }

// =============================================================================
// Shortmess / misc message helpers
// =============================================================================

int nvim_fileio_shortmess_over(void) { return shortmess(SHM_OVER) ? 1 : 0; }
int nvim_fileio_shortmess_ro(void) { return shortmess(SHM_RO) ? 1 : 0; }
int nvim_fileio_aborting(void) { return aborting() ? 1 : 0; }

// =============================================================================
// memline accessors
// =============================================================================

int nvim_fileio_ml_line_count(void) { return (int)curbuf->b_ml.ml_line_count; }
const char *nvim_fileio_ml_get(int lnum) { return ml_get((linenr_T)lnum); }
int nvim_fileio_ml_get_len(int lnum) { return ml_get_len((linenr_T)lnum); }
int nvim_fileio_ml_append(int lnum, const char *line, int len, int newfile) {
  return ml_append((linenr_T)lnum, (char *)line, (colnr_T)len, newfile != 0);
}
int nvim_fileio_ml_delete(int lnum) {
  return ml_delete((linenr_T)lnum);
}

// vim_lseek wrapper
int64_t nvim_fileio_vim_lseek(int fd, int64_t offset, int whence) {
  return (int64_t)vim_lseek(fd, (off_T)offset, whence);
}

// =============================================================================
// Memory helpers
// =============================================================================

void *nvim_fileio_verbose_try_malloc(size_t size) { return verbose_try_malloc(size); }
void *nvim_fileio_xcalloc(size_t nmemb, size_t size) { return xcalloc(nmemb, size); }

// =============================================================================
// file options / encoding setup
// =============================================================================

char *nvim_fileio_enc_canonize(const char *enc) { return enc_canonize((char *)enc); }
void nvim_fileio_save_file_ff(void) { save_file_ff(curbuf); }
void nvim_fileio_set_fileformat(int ff) { set_fileformat((int)ff, OPT_LOCAL); }
void nvim_fileio_set_option_direct_fenc(const char *fenc) {
  set_option_direct(kOptFileencoding, CSTR_AS_OPTVAL((char *)fenc), OPT_LOCAL, 0);
}
int nvim_fileio_get_fileformat_force(void *eap) {
  return get_fileformat_force(curbuf, (exarg_T *)eap);
}
int nvim_fileio_shortmess(int msg_id) { return shortmess(msg_id) ? 1 : 0; }

// =============================================================================
// check_need_swap wrapper
// =============================================================================

void nvim_fileio_check_need_swap(int newfile) { check_need_swap(newfile != 0); }

// =============================================================================
// Message / display helpers
// =============================================================================

void nvim_fileio_filemess(const char *fname, const char *s) {
  filemess(curbuf, (char *)fname, (char *)s);
}
char *nvim_fileio_msg_trunc(char *s) { return msg_trunc(s, false, 0); }
void nvim_fileio_set_keep_msg(const char *s) { set_keep_msg((char *)s, 0); }
void nvim_fileio_XFREE_CLEAR_keep_msg(void) { XFREE_CLEAR(keep_msg); }
void nvim_fileio_xstrlcat(char *dst, const char *src, size_t dst_size) {
  xstrlcat(dst, (char *)src, dst_size);
}
void nvim_fileio_snprintf_iobuff(int offset, const char *fmt, int64_t val) {
  // For the two snprintf calls in readfile post-read section
  snprintf(IObuff + offset, (size_t)(IOSIZE - offset), fmt, val);
}
void nvim_fileio_add_quoted_fname(char *sfname) {
  add_quoted_fname(IObuff, IOSIZE, curbuf, sfname);
}
int nvim_fileio_msg_add_fileformat(int ff) { return msg_add_fileformat(ff) ? 1 : 0; }
void nvim_fileio_msg_add_lines(int insert_space, int linecnt, int64_t filesize) {
  msg_add_lines(insert_space, (linenr_T)linecnt, (off_T)filesize);
}
char *nvim_fileio_get_IObuff(void) { return IObuff; }
int nvim_fileio_get_IOSIZE(void) { return IOSIZE; }
int nvim_fileio_strlen_IObuff(void) { return (int)strlen(IObuff); }

// Translated messages (needed for gettext)
const char *nvim_fileio_msg_illegal_filename(void) { return _("Illegal file name"); }
const char *nvim_fileio_msg_is_a_directory(void) { return _("is a directory"); }
const char *nvim_fileio_msg_is_not_a_file(void) { return _("is not a file"); }
const char *nvim_fileio_msg_file_too_big(void) { return _("[File too big]"); }
const char *nvim_fileio_msg_permission_denied(void) { return _("[Permission Denied]"); }
const char *nvim_fileio_msg_new(void) { return _("[New]"); }
const char *nvim_fileio_msg_new_directory(void) { return _("[New DIRECTORY]"); }
const char *nvim_fileio_msg_conversion_error(void) { return _("[CONVERSION ERROR in line %" PRId64 "]"); }
const char *nvim_fileio_msg_illegal_byte(void) { return _("[ILLEGAL BYTE in line %" PRId64 "]"); }
const char *nvim_fileio_msg_read_errors(void) { return _("[READ ERRORS]"); }
const char *nvim_fileio_msg_read_interrupted(void) { return e_interr; }
const char *nvim_fileio_msg_e200(void) { return _("E200: *ReadPre autocommands made the file unreadable"); }
const char *nvim_fileio_msg_e201(void) { return _("E201: *ReadPre autocommands must not change current buffer"); }
const char *nvim_fileio_msg_e202(void) { return _("E202: Conversion made file unreadable!"); }
const char *nvim_fileio_msg_e_auchangedbuf(void) {
  return _("E812: Autocommands changed buffer or buffer name");
}
const char *nvim_fileio_msg_fifo(void) { return _("[fifo]"); }
const char *nvim_fileio_msg_socket(void) { return _("[socket]"); }
const char *nvim_fileio_msg_character_special(void) { return _("[character special]"); }
const char *nvim_fileio_msg_readonly(void) { return _("[readonly]"); }
const char *nvim_fileio_msg_ro(void) { return _("[RO]"); }
const char *nvim_fileio_msg_noeol(void) { return _("[noeol]"); }
const char *nvim_fileio_msg_cr_missing(void) { return _("[CR missing]"); }
const char *nvim_fileio_msg_long_lines_split(void) { return _("[long lines split]"); }
const char *nvim_fileio_msg_not_converted(void) { return _("[NOT converted]"); }
const char *nvim_fileio_msg_converted(void) { return _("[converted]"); }

// =============================================================================
// emsg wrapper
// =============================================================================

void nvim_fileio_emsg(const char *s) { emsg((char *)s); }

// =============================================================================
// vim_strchr wrapper
// =============================================================================

int nvim_fileio_vim_strchr(const char *s, int c) { return vim_strchr(s, c) != NULL ? 1 : 0; }

// =============================================================================
// Undo helpers
// =============================================================================

void nvim_fileio_u_clearline(void) { u_clearline(curbuf); }
void nvim_fileio_sha256_start(void *ctx) { sha256_start((context_sha256_T *)ctx); }
void nvim_fileio_sha256_update(void *ctx, const uint8_t *data, size_t len) {
  sha256_update((context_sha256_T *)ctx, data, len);
}
void nvim_fileio_sha256_finish(void *ctx, uint8_t *hash) {
  sha256_finish((context_sha256_T *)ctx, hash);
}
void nvim_fileio_u_read_undo(uint8_t *hash, const char *fname) {
  u_read_undo(NULL, hash, (char *)fname);
}
int nvim_fileio_sha256_ctx_size(void) { return (int)sizeof(context_sha256_T); }

// =============================================================================
// redraw / display
// =============================================================================

void nvim_fileio_redraw_curbuf_later(void) { redraw_curbuf_later(UPD_NOT_VALID); }
void nvim_fileio_appended_lines_mark(int from, int linecnt) {
  appended_lines_mark((linenr_T)from, (long)linecnt);
}

// =============================================================================
// cursor movement
// =============================================================================

void nvim_fileio_check_cursor_lnum(void) { check_cursor_lnum(curwin); }
void nvim_fileio_beginline(void) { beginline(BL_WHITE | BL_FIX); }

// =============================================================================
// os_set_cloexec
// =============================================================================


// =============================================================================
// os_breakcheck
// =============================================================================


// =============================================================================
// os_remove
// =============================================================================


// =============================================================================
// iconv helpers (C wrappers for Rust)
// =============================================================================

iconv_t nvim_fileio_my_iconv_open(const char *to, const char *from) {
  return (iconv_t)my_iconv_open((char *)to, (char *)from);
}
iconv_t nvim_fileio_iconv_invalid(void) { return (iconv_t)-1; }
int nvim_fileio_iconv_is_invalid(iconv_t fd) { return fd == (iconv_t)-1 ? 1 : 0; }
size_t nvim_fileio_iconv(iconv_t cd, const char **inbuf, size_t *inbytesleft,
                          char **outbuf, size_t *outbytesleft) {
  return iconv(cd, (void *)inbuf, inbytesleft, outbuf, outbytesleft);
}
int nvim_fileio_iconv_errno(void) { return ICONV_ERRNO; }
int nvim_fileio_iconv_einval(void) { return ICONV_EINVAL; }

// =============================================================================
// utf8 / mbyte helpers
// =============================================================================

int nvim_fileio_utf_head_off(const char *base, const char *p) {
  return utf_head_off((char *)base, (char *)p);
}
int nvim_fileio_utf_ptr2char(const char *p) { return utf_ptr2char((char *)p); }
int nvim_fileio_utf_ptr2len_len(const char *p, int size) {
  return utf_ptr2len_len((char *)p, size);
}
int nvim_fileio_utf_char2bytes(int c, char *buf) { return utf_char2bytes(c, (char *)buf); }

// =============================================================================
// SHM_* constants
// =============================================================================
int nvim_fileio_SHM_OVER(void) { return SHM_OVER; }
int nvim_fileio_SHM_RO(void) { return SHM_RO; }

// =============================================================================
// perm constants
// =============================================================================
int nvim_fileio_UV_ENOENT(void) { return UV_ENOENT; }
#if defined(UNIX) && defined(EOVERFLOW)
int nvim_fileio_UV_EFBIG(void) { return UV_EFBIG; }
int nvim_fileio_EOVERFLOW_neg(void) { return -EOVERFLOW; }
#else
int nvim_fileio_UV_EFBIG(void) { return UV_EFBIG; }
int nvim_fileio_EOVERFLOW_neg(void) { return INT_MIN; }
#endif

// =============================================================================
// ENC_UCSBOM constant
// =============================================================================
const char *nvim_fileio_ENC_UCSBOM(void) { return ENC_UCSBOM; }

// =============================================================================
// CPO_FNAMER constant
// =============================================================================
int nvim_fileio_CPO_FNAMER(void) { return CPO_FNAMER; }

// =============================================================================
// ML_EMPTY, BF_* constants
// =============================================================================
int nvim_fileio_ML_EMPTY(void) { return ML_EMPTY; }
int nvim_fileio_BF_CHECK_RO(void) { return BF_CHECK_RO; }
int nvim_fileio_BF_NEW(void) { return BF_NEW; }
int nvim_fileio_BF_NEW_W(void) { return BF_NEW_W; }
int nvim_fileio_BF_NOTEDITED(void) { return BF_NOTEDITED; }
int nvim_fileio_SEA_QUIT(void) { return SEA_QUIT; }

// =============================================================================
// EOL_* constants
// =============================================================================
int nvim_fileio_EOL_UNKNOWN(void) { return EOL_UNKNOWN; }
int nvim_fileio_EOL_UNIX(void) { return EOL_UNIX; }
int nvim_fileio_EOL_DOS(void) { return EOL_DOS; }
int nvim_fileio_EOL_MAC(void) { return EOL_MAC; }

// =============================================================================
// FAIL/OK/NOTDONE constants
// =============================================================================
int nvim_fileio_FAIL(void) { return FAIL; }
int nvim_fileio_OK(void) { return OK; }
int nvim_fileio_NOTDONE(void) { return NOTDONE; }

// =============================================================================
// BAD_* constants
// =============================================================================
int nvim_fileio_BAD_REPLACE(void) { return BAD_REPLACE; }
int nvim_fileio_BAD_KEEP(void) { return BAD_KEEP; }
int nvim_fileio_BAD_DROP(void) { return BAD_DROP; }

// =============================================================================
// exarg_T field accessors (opaque pointer)
// =============================================================================
int nvim_fileio_eap_bad_char(void *eap) { return (int)((exarg_T *)eap)->bad_char; }
int nvim_fileio_eap_force_enc(void *eap) { return (int)((exarg_T *)eap)->force_enc; }
int nvim_fileio_eap_force_ff(void *eap) { return (int)((exarg_T *)eap)->force_ff; }
int nvim_fileio_eap_read_edit(void *eap) { return (int)((exarg_T *)eap)->read_edit; }
const char *nvim_fileio_eap_force_enc_str(void *eap) {
  exarg_T *ea = (exarg_T *)eap;
  return ea->cmd + ea->force_enc;
}

// =============================================================================
// stdin dup for post-read stdin handling
// =============================================================================
void nvim_fileio_stdin_post_read(void) {
#ifndef MSWIN
  vim_ignored = dup(2);
#else
  // On Windows, use the console input handle for stdin.
  HANDLE conin = CreateFile("CONIN$", GENERIC_READ | GENERIC_WRITE,
                            FILE_SHARE_READ, (LPSECURITY_ATTRIBUTES)NULL,
                            OPEN_EXISTING, 0, (HANDLE)NULL);
  vim_ignored = _open_osfhandle((intptr_t)conin, _O_RDONLY);
#endif
}

// =============================================================================
// OPEN_CHR_FILES / is_dev_fd_file check
// =============================================================================
int nvim_fileio_is_chr_dev(int perm, const char *fname) {
#ifdef OPEN_CHR_FILES
  return (S_ISCHR(perm) && rs_is_dev_fd_file(fname)) ? 1 : 0;
#else
  (void)perm; (void)fname;
  return 0;
#endif
}

// =============================================================================
// swap_exists_action accessor
// =============================================================================
int nvim_fileio_get_swap_exists_action(void) { return swap_exists_action; }

// UNDO_HASH_SIZE
int nvim_fileio_UNDO_HASH_SIZE(void) { return UNDO_HASH_SIZE; }
