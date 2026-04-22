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
// curbuf field accessors (mfp dirty state only -- others accessed via BufStruct)
// =============================================================================

int nvim_fileio_curbuf_mfp_dirty_is_nosync(void) {
  return (curbuf->b_ml.ml_mfp != NULL &&
          curbuf->b_ml.ml_mfp->mf_dirty == MF_DIRTY_YES_NOSYNC) ? 1 : 0;
}
void nvim_fileio_curbuf_mfp_set_dirty_yes(void) {
  if (curbuf->b_ml.ml_mfp != NULL) {
    curbuf->b_ml.ml_mfp->mf_dirty = MF_DIRTY_YES;
  }
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
// Shortmess / misc message helpers
// =============================================================================
int nvim_fileio_aborting(void) { return aborting() ? 1 : 0; }

// vim_lseek wrapper
int64_t nvim_fileio_vim_lseek(int fd, int64_t offset, int whence) {
  return (int64_t)vim_lseek(fd, (off_T)offset, whence);
}

// =============================================================================
// file options / encoding setup
// =============================================================================
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
char *nvim_fileio_get_IObuff(void) { return IObuff; }
int nvim_fileio_strlen_IObuff(void) { return (int)strlen(IObuff); }

// =============================================================================
// Undo helpers
// =============================================================================
void nvim_fileio_sha256_update(void *ctx, const uint8_t *data, size_t len) {
  sha256_update((context_sha256_T *)ctx, data, len);
}
void nvim_fileio_sha256_finish(void *ctx, uint8_t *hash) {
  sha256_finish((context_sha256_T *)ctx, hash);
}
void nvim_fileio_u_read_undo(uint8_t *hash, const char *fname) {
  u_read_undo(NULL, hash, (char *)fname);
}
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
int nvim_fileio_utf_ptr2len_len(const char *p, int size) {
  return utf_ptr2len_len((char *)p, size);
}


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


