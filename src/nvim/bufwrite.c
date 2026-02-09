// bufwrite.c: functions for writing a buffer

#include <fcntl.h>
#include <iconv.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>
#include <sys/stat.h>
#include <uv.h>

#include "auto/config.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/bufwrite.h"
#include "nvim/change.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_eval.h"
#include "nvim/fileio.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/highlight_defs.h"
#include "nvim/iconv_defs.h"
#include "nvim/input.h"
#include "nvim/macros_defs.h"
#include "nvim/mbyte.h"
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
#include "nvim/path.h"
#include "nvim/pos_defs.h"
#include "nvim/sha256.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/undo.h"
#include "nvim/undo_defs.h"
#include "nvim/vim_defs.h"

// Static assertions for constants shared with Rust
_Static_assert(FIO_LATIN1 == 0x01, "FIO_LATIN1");
_Static_assert(FIO_UTF8 == 0x02, "FIO_UTF8");
_Static_assert(FIO_UCS2 == 0x04, "FIO_UCS2");
_Static_assert(FIO_UCS4 == 0x08, "FIO_UCS4");
_Static_assert(FIO_UTF16 == 0x10, "FIO_UTF16");
_Static_assert(FIO_ENDIAN_L == 0x80, "FIO_ENDIAN_L");
_Static_assert(FIO_NOCONVERT == 0x2000, "FIO_NOCONVERT");
_Static_assert(CONV_RESTLEN == 30, "CONV_RESTLEN");
_Static_assert(WRITEBUFSIZE == 8192, "WRITEBUFSIZE");
_Static_assert(ICONV_MULT == 8, "ICONV_MULT");
_Static_assert(EOL_UNIX == 0, "EOL_UNIX");
_Static_assert(EOL_DOS == 1, "EOL_DOS");
_Static_assert(EOL_MAC == 2, "EOL_MAC");
_Static_assert(FIO_UCSBOM == 0x4000, "FIO_UCSBOM");
_Static_assert(FIO_ALL == -1, "FIO_ALL");
_Static_assert(OK == 1, "OK");
_Static_assert(FAIL == 0, "FAIL");
_Static_assert(NOTDONE == 2, "NOTDONE");
_Static_assert(NODE_WRITABLE == 1, "NODE_WRITABLE");
_Static_assert(NODE_OTHER == 2, "NODE_OTHER");
_Static_assert(HLF_E == 6, "HLF_E");
_Static_assert(kOptBkcFlagYes == 0x01, "kOptBkcFlagYes");
_Static_assert(kOptBkcFlagAuto == 0x02, "kOptBkcFlagAuto");
_Static_assert(kOptBkcFlagNo == 0x04, "kOptBkcFlagNo");
_Static_assert(kOptBkcFlagBreaksymlink == 0x08, "kOptBkcFlagBreaksymlink");
_Static_assert(kOptBkcFlagBreakhardlink == 0x10, "kOptBkcFlagBreakhardlink");
_Static_assert(MAXPATHL >= 4096, "MAXPATHL");
_Static_assert(IOSIZE == 1025, "IOSIZE");

static const char *err_readonly = "is read-only (cannot override: \"W\" in 'cpoptions')";
static const char e_patchmode_cant_touch_empty_original_file[]
  = N_("E206: Patchmode: can't touch empty original file");
static const char e_write_error_conversion_failed_make_fenc_empty_to_override[]
  = N_("E513: Write error, conversion failed (make 'fenc' empty to override)");
static const char e_write_error_conversion_failed_in_line_nr_make_fenc_empty_to_override[]
  = N_("E513: Write error, conversion failed in line %" PRIdLINENR
       " (make 'fenc' empty to override)");
static const char e_write_error_file_system_full[]
  = N_("E514: Write error (file system full?)");
static const char e_no_matching_autocommands_for_buftype_str_buffer[]
  = N_("E676: No matching autocommands for buftype=%s buffer");

typedef struct {
  const char *num;
  char *msg;
  int arg;
  bool alloc;
} Error_T;

#define SMALLBUFSIZE 256     // size of emergency write buffer

// Structure to pass arguments from buf_write() to buf_write_bytes().
struct bw_info {
  int bw_fd;                      // file descriptor
  char *bw_buf;                   // buffer with data to be written
  int bw_len;                     // length of data
  int bw_flags;                   // FIO_ flags
  uint8_t bw_rest[CONV_RESTLEN];  // not converted bytes
  int bw_restlen;                 // nr of bytes in bw_rest[]
  int bw_first;                   // first write call
  char *bw_conv_buf;              // buffer for writing converted chars
  size_t bw_conv_buflen;          // size of bw_conv_buf
  int bw_conv_error;              // set for conversion error
  linenr_T bw_conv_error_lnum;    // first line with error or zero
  linenr_T bw_start_lnum;         // line number at start of buffer
  iconv_t bw_iconv_fd;            // descriptor for iconv() or -1
};

// =============================================================================
// C accessor functions for Rust FFI
// =============================================================================

void nvim_bw_emsg(const char *msg)
{
  emsg(msg);
}

void nvim_bw_semsg_2(const char *fmt, const char *a, const char *b)
{
  (void)b;  // b is unused; Rust passes null for single-arg format
  semsg(fmt, a);
}

void nvim_bw_semsg_3(const char *fmt, const char *a, const char *b, const char *c)
{
  semsg(fmt, a, b, c);
}

void nvim_bw_semsg_4(const char *fmt, const char *a, const char *b, const char *c, const char *d)
{
  semsg(fmt, a, b, c, d);
}

const char *nvim_bw_os_strerror(int errnum)
{
  return os_strerror(errnum);
}

char *nvim_bw_get_IObuff(void)
{
  return IObuff;
}

void nvim_bw_xfree(char *ptr)
{
  xfree(ptr);
}

int nvim_bw_get_fio_flags(const char *name)
{
  return get_fio_flags(name);
}

// File info accessors
int nvim_bw_os_fileinfo(const char *fname, FileInfo *info)
{
  return os_fileinfo(fname, info);
}

int nvim_bw_os_nodetype(const char *fname)
{
  return os_nodetype(fname);
}

int nvim_bw_os_getperm(const char *fname)
{
  return os_getperm(fname);
}

int nvim_bw_os_isdir(const char *fname)
{
  return os_isdir((char *)fname);
}

int nvim_bw_os_file_is_writable(const char *fname)
{
  return os_file_is_writable(fname);
}

int nvim_bw_fi_get_st_mode(FileInfo *info)
{
  return (int)info->stat.st_mode;
}

int nvim_bw_fi_is_regular(FileInfo *info)
{
  return S_ISREG(info->stat.st_mode);
}

int nvim_bw_fi_is_dir(FileInfo *info)
{
  return S_ISDIR(info->stat.st_mode);
}

// Buffer mtime accessors
int64_t nvim_bw_buf_get_mtime_read(buf_T *buf)
{
  return buf->b_mtime_read;
}

int64_t nvim_bw_buf_get_mtime_read_ns(buf_T *buf)
{
  return buf->b_mtime_read_ns;
}

int nvim_bw_time_differs(FileInfo *info, int64_t mtime, int64_t mtime_ns)
{
  return time_differs(info, mtime, mtime_ns);
}

// Globals
int nvim_bw_get_msg_scroll(void) { return msg_scroll; }
void nvim_bw_set_msg_scroll(int val) { msg_scroll = val; }
int nvim_bw_get_msg_silent(void) { return msg_silent; }
void nvim_bw_set_msg_silent(int val) { msg_silent = val; }

// Dialog
void nvim_bw_msg(const char *s, int hlf)
{
  msg(s, hlf);
}

int nvim_bw_ask_yesno(const char *s)
{
  return ask_yesno(s);
}

// Options
int nvim_bw_cpo_contains(int c)
{
  return vim_strchr(p_cpo, c) != NULL;
}

// Gettext
const char *nvim_bw_gettext(const char *s)
{
  return _(s);
}

// bw_info field accessors (use void* for opaque handle pattern)
int nvim_bw_info_get_fd(void *p) { struct bw_info *ip = p; return ip->bw_fd; }
char *nvim_bw_info_get_buf(void *p) { struct bw_info *ip = p; return ip->bw_buf; }
int nvim_bw_info_get_len(void *p) { struct bw_info *ip = p; return ip->bw_len; }
int nvim_bw_info_get_flags(void *p) { struct bw_info *ip = p; return ip->bw_flags; }
int nvim_bw_info_get_restlen(void *p) { struct bw_info *ip = p; return ip->bw_restlen; }
void nvim_bw_info_set_restlen(void *p, int val) { struct bw_info *ip = p; ip->bw_restlen = val; }
uint8_t *nvim_bw_info_get_rest_ptr(void *p) { struct bw_info *ip = p; return ip->bw_rest; }
int nvim_bw_info_get_first(void *p) { struct bw_info *ip = p; return ip->bw_first; }
void nvim_bw_info_set_first(void *p, int val) { struct bw_info *ip = p; ip->bw_first = val; }
char *nvim_bw_info_get_conv_buf(void *p) { struct bw_info *ip = p; return ip->bw_conv_buf; }
size_t nvim_bw_info_get_conv_buflen(void *p) { struct bw_info *ip = p; return ip->bw_conv_buflen; }
int nvim_bw_info_get_conv_error(void *p) { struct bw_info *ip = p; return ip->bw_conv_error; }
void nvim_bw_info_set_conv_error(void *p, int val) { struct bw_info *ip = p; ip->bw_conv_error = val; }
linenr_T nvim_bw_info_get_conv_error_lnum(void *p) { struct bw_info *ip = p; return ip->bw_conv_error_lnum; }
void nvim_bw_info_set_conv_error_lnum(void *p, linenr_T val) { struct bw_info *ip = p; ip->bw_conv_error_lnum = val; }
linenr_T nvim_bw_info_get_start_lnum(void *p) { struct bw_info *ip = p; return ip->bw_start_lnum; }
void nvim_bw_info_set_start_lnum(void *p, linenr_T val) { struct bw_info *ip = p; ip->bw_start_lnum = val; }
int nvim_bw_info_has_iconv(void *p) { struct bw_info *ip = p; return ip->bw_iconv_fd != (iconv_t)-1; }

// iconv wrapper: handles the full iconv conversion with remainder management
int nvim_bw_iconv_convert(void *p, char **bufp, int *lenp)
{
  struct bw_info *ip = p;
  const char *from;
  size_t fromlen;
  size_t tolen;

  int len = *lenp;

  if (ip->bw_restlen > 0) {
    fromlen = (size_t)len + (size_t)ip->bw_restlen;
    char *fp = ip->bw_conv_buf + ip->bw_conv_buflen - fromlen;
    memmove(fp, ip->bw_rest, (size_t)ip->bw_restlen);
    memmove(fp + ip->bw_restlen, *bufp, (size_t)len);
    from = fp;
    tolen = ip->bw_conv_buflen - fromlen;
  } else {
    from = *bufp;
    fromlen = (size_t)len;
    tolen = ip->bw_conv_buflen;
  }
  char *to = ip->bw_conv_buf;

  if (ip->bw_first) {
    size_t save_len = tolen;
    iconv(ip->bw_iconv_fd, NULL, NULL, &to, &tolen);
    if (to == NULL) {
      to = ip->bw_conv_buf;
      tolen = save_len;
    }
    ip->bw_first = false;
  }

  if ((iconv(ip->bw_iconv_fd, (void *)&from, &fromlen, &to, &tolen)
       == (size_t)-1 && ICONV_ERRNO != ICONV_EINVAL)
      || fromlen > CONV_RESTLEN) {
    ip->bw_conv_error = true;
    return FAIL;
  }

  if (fromlen > 0) {
    memmove(ip->bw_rest, (void *)from, fromlen);
  }
  ip->bw_restlen = (int)fromlen;

  *bufp = ip->bw_conv_buf;
  *lenp = (int)(to - ip->bw_conv_buf);

  return OK;
}

// mbyte wrappers
int nvim_bw_utf_char2bytes(int c, char *buf)
{
  return utf_char2bytes(c, buf);
}

int nvim_bw_utf_ptr2char(const char *p)
{
  return utf_ptr2char(p);
}

int nvim_bw_utf_ptr2len_len(const char *p, int len)
{
  return utf_ptr2len_len(p, len);
}

// I/O
int nvim_bw_write_eintr(int fd, const char *buf, size_t len)
{
  return write_eintr(fd, (void *)buf, len);
}

// Backup accessors
int nvim_bw_os_fileinfo_hardlinks(FileInfo *fi) { return (int)os_fileinfo_hardlinks(fi); }
int nvim_bw_os_fileinfo_link(const char *fname, FileInfo *fi) { return os_fileinfo_link(fname, fi); }
int nvim_bw_os_fileinfo_id_equal(FileInfo *a, FileInfo *b) { return os_fileinfo_id_equal(a, b); }
char *nvim_bw_path_tail(const char *fname) { return path_tail(fname); }
int nvim_bw_after_pathsep(const char *b, const char *p) { return after_pathsep(b, p); }
char *nvim_bw_make_percent_swname(char *dir, char *dir_end, const char *name) { return make_percent_swname(dir, dir_end, name); }
char *nvim_bw_modname(const char *fname, const char *ext, int prepend_dot) { return modname(fname, ext, prepend_dot != 0); }
char *nvim_bw_get_file_in_dir(char *fname, char *dname) { return get_file_in_dir(fname, dname); }
size_t nvim_bw_copy_option_part(char **option, char *buf, size_t maxlen, const char *sep) { return copy_option_part(option, buf, maxlen, (char *)sep); }
char *nvim_bw_get_p_bex(void) { return p_bex; }
char *nvim_bw_get_p_bdir(void) { return p_bdir; }
int nvim_bw_get_p_bk(void) { return p_bk; }
int nvim_bw_os_open(const char *path, int flags, int mode) { return os_open(path, flags, mode); }
int nvim_bw_os_close(int fd) { return close(fd); }
int nvim_bw_os_remove(const char *path) { return os_remove(path); }
int nvim_bw_os_copy(const char *from, const char *to, int flags) { return os_copy(from, to, flags); }
int nvim_bw_os_setperm(const char *path, int perm) { return os_setperm(path, perm); }
int nvim_bw_os_path_exists(const char *path) { return os_path_exists(path); }
int nvim_bw_vim_rename(const char *from, const char *to) { return vim_rename(from, to); }
char *nvim_bw_get_IObuff_mut(void) { return IObuff; }
int nvim_bw_get_IOSIZE(void) { return IOSIZE; }
void nvim_bw_os_set_acl(const char *fname, vim_acl_T acl) { os_set_acl(fname, acl); }
int nvim_bw_os_mkdir_recurse(const char *dir, int32_t mode, char **failed_dir) { return os_mkdir_recurse(dir, mode, failed_dir, NULL); }
void nvim_bw_xmemcpyz(void *dst, const void *src, size_t len) { xmemcpyz(dst, src, len); }
int nvim_bw_snprintf_int(char *buf, size_t len, int val) { return snprintf(buf, len, "%d", val); }
size_t nvim_bw_strlen(const char *s) { return strlen(s); }
void nvim_bw_XFREE_CLEAR(char **pp) { XFREE_CLEAR(*pp); }
int nvim_bw_open_flags_creat_wronly_excl_nofollow(void) { return O_CREAT|O_WRONLY|O_EXCL|O_NOFOLLOW; }
int nvim_bw_uv_fs_copyfile_ficlone(void) { return UV_FS_COPYFILE_FICLONE; }
int nvim_bw_get_MAXPATHL(void) { return MAXPATHL; }
const char *nvim_bw_get_err_readonly(void) { return _(err_readonly); }
size_t nvim_bw_sizeof_FileInfo(void) { return sizeof(FileInfo); }
#ifdef UNIX
void nvim_bw_os_fchown(int fd, uint32_t uid, uint32_t gid) { os_fchown(fd, (uv_uid_t)uid, (uv_gid_t)gid); }
int nvim_bw_os_chown(const char *path, uint32_t uid, uint32_t gid) { return os_chown(path, (uv_uid_t)uid, (uv_gid_t)gid); }
void nvim_bw_os_file_settime(const char *path, double atime, double mtime) { os_file_settime(path, atime, mtime); }
uint32_t nvim_bw_fi_get_st_uid(FileInfo *fi) { return (uint32_t)fi->stat.st_uid; }
uint32_t nvim_bw_fi_get_st_gid(FileInfo *fi) { return (uint32_t)fi->stat.st_gid; }
int64_t nvim_bw_fi_get_atime_sec(FileInfo *fi) { return (int64_t)fi->stat.st_atim.tv_sec; }
int64_t nvim_bw_fi_get_mtime_sec(FileInfo *fi) { return (int64_t)fi->stat.st_mtim.tv_sec; }
uint32_t nvim_bw_getuid(void) { return (uint32_t)getuid(); }
#else
void nvim_bw_os_fchown(int fd, uint32_t uid, uint32_t gid) { (void)fd; (void)uid; (void)gid; }
int nvim_bw_os_chown(const char *path, uint32_t uid, uint32_t gid) { (void)path; (void)uid; (void)gid; return -1; }
void nvim_bw_os_file_settime(const char *path, double atime, double mtime) { (void)path; (void)atime; (void)mtime; }
uint32_t nvim_bw_fi_get_st_uid(FileInfo *fi) { (void)fi; return 0; }
uint32_t nvim_bw_fi_get_st_gid(FileInfo *fi) { (void)fi; return 0; }
int64_t nvim_bw_fi_get_atime_sec(FileInfo *fi) { (void)fi; return 0; }
int64_t nvim_bw_fi_get_mtime_sec(FileInfo *fi) { (void)fi; return 0; }
uint32_t nvim_bw_getuid(void) { return 0; }
#endif
#ifdef HAVE_XATTR
void nvim_bw_os_copy_xattr(const char *from, const char *to) { os_copy_xattr(from, to); }
#else
void nvim_bw_os_copy_xattr(const char *from, const char *to) { (void)from; (void)to; }
#endif

// =============================================================================
// Rust FFI declarations
// =============================================================================
extern Error_T rs_set_err_num(const char *num, const char *msg);
extern Error_T rs_set_err(const char *msg);
extern Error_T rs_set_err_arg(const char *msg, int arg);
extern void rs_emit_err(const Error_T *e);
extern bool rs_ucs2bytes(unsigned c, char **pp, int flags);
extern int rs_make_bom(char *buf, char *name);
extern int rs_check_mtime(buf_T *buf, FileInfo *file_info);
extern int rs_get_fileinfo_os(char *fname, FileInfo *file_info_old, int overwriting,
                              int *perm, bool *device, bool *newfile, Error_T *err);
extern int rs_get_fileinfo(buf_T *buf, char *fname, int overwriting, int forceit,
                           FileInfo *file_info_old, int *perm, bool *device, bool *newfile,
                           bool *readonly, Error_T *err);
extern int rs_buf_write_convert_with_iconv(struct bw_info *ip, char **bufp, int *lenp);
extern int rs_buf_write_convert(struct bw_info *ip, char **bufp, int *lenp);
extern int rs_buf_write_bytes(struct bw_info *ip);
extern int rs_buf_write_make_backup(char *fname, int append, FileInfo *file_info_old,
                                    vim_acl_T acl, int perm, unsigned bkc, int file_readonly,
                                    int forceit, bool *backup_copyp, char **backupp,
                                    Error_T *err);

#include "bufwrite.c.generated.h"

/// Convert a Unicode character to bytes.
///
/// @param c character to convert
/// @param[in,out] pp pointer to store the result at
/// @param flags FIO_ flags that specify which encoding to use
///
/// @return true for an error, false when it's OK.
static bool ucs2bytes(unsigned c, char **pp, int flags)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_ucs2bytes(c, pp, flags);
}

static int buf_write_convert_with_iconv(struct bw_info *ip, char **bufp, int *lenp)
{
  return rs_buf_write_convert_with_iconv(ip, bufp, lenp);
}

static int buf_write_convert(struct bw_info *ip, char **bufp, int *lenp)
{
  return rs_buf_write_convert(ip, bufp, lenp);
}

/// Call write() to write a number of bytes to the file.
/// Handles 'encoding' conversion.
///
/// @return  FAIL for failure, OK otherwise.
static int buf_write_bytes(struct bw_info *ip)
{
  return rs_buf_write_bytes(ip);
}

/// Check modification time of file, before writing to it.
static int check_mtime(buf_T *buf, FileInfo *file_info)
{
  return rs_check_mtime(buf, file_info);
}

/// Generate a BOM in "buf[4]" for encoding "name".
///
/// @return  the length of the BOM (zero when no BOM).
static int make_bom(char *buf_in, char *name)
{
  return rs_make_bom(buf_in, name);
}

static int buf_write_do_autocmds(buf_T *buf, char **fnamep, char **sfnamep, char **ffnamep,
                                 linenr_T start, linenr_T *endp, exarg_T *eap, bool append,
                                 bool filtering, bool reset_changed, bool overwriting, bool whole,
                                 const pos_T orig_start, const pos_T orig_end)
{
  linenr_T old_line_count = buf->b_ml.ml_line_count;
  int msg_save = msg_scroll;

  aco_save_T aco;
  bool did_cmd = false;
  bool nofile_err = false;
  bool empty_memline = buf->b_ml.ml_mfp == NULL;
  bufref_T bufref;

  char *sfname = *sfnamep;

  // Apply PRE autocommands.
  // Set curbuf to the buffer to be written.
  // Careful: The autocommands may call buf_write() recursively!
  bool buf_ffname = *ffnamep == buf->b_ffname;
  bool buf_sfname = sfname == buf->b_sfname;
  bool buf_fname_f = *fnamep == buf->b_ffname;
  bool buf_fname_s = *fnamep == buf->b_sfname;

  // Set curwin/curbuf to buf and save a few things.
  aucmd_prepbuf(&aco, buf);
  set_bufref(&bufref, buf);

  if (append) {
    did_cmd = apply_autocmds_exarg(EVENT_FILEAPPENDCMD, sfname, sfname, false, curbuf, eap);
    if (!did_cmd) {
      if (overwriting && bt_nofilename(curbuf)) {
        nofile_err = true;
      } else {
        apply_autocmds_exarg(EVENT_FILEAPPENDPRE,
                             sfname, sfname, false, curbuf, eap);
      }
    }
  } else if (filtering) {
    apply_autocmds_exarg(EVENT_FILTERWRITEPRE,
                         NULL, sfname, false, curbuf, eap);
  } else if (reset_changed && whole) {
    bool was_changed = curbufIsChanged();

    did_cmd = apply_autocmds_exarg(EVENT_BUFWRITECMD, sfname, sfname, false, curbuf, eap);
    if (did_cmd) {
      if (was_changed && !curbufIsChanged()) {
        // Written everything correctly and BufWriteCmd has reset
        // 'modified': Correct the undo information so that an
        // undo now sets 'modified'.
        u_unchanged(curbuf);
        u_update_save_nr(curbuf);
      }
    } else {
      if (overwriting && bt_nofilename(curbuf)) {
        nofile_err = true;
      } else {
        apply_autocmds_exarg(EVENT_BUFWRITEPRE,
                             sfname, sfname, false, curbuf, eap);
      }
    }
  } else {
    did_cmd = apply_autocmds_exarg(EVENT_FILEWRITECMD, sfname, sfname, false, curbuf, eap);
    if (!did_cmd) {
      if (overwriting && bt_nofilename(curbuf)) {
        nofile_err = true;
      } else {
        apply_autocmds_exarg(EVENT_FILEWRITEPRE,
                             sfname, sfname, false, curbuf, eap);
      }
    }
  }

  // restore curwin/curbuf and a few other things
  aucmd_restbuf(&aco);

  // In three situations we return here and don't write the file:
  // 1. the autocommands deleted or unloaded the buffer.
  // 2. The autocommands abort script processing.
  // 3. If one of the "Cmd" autocommands was executed.
  if (!bufref_valid(&bufref)) {
    buf = NULL;
  }
  if (buf == NULL || (buf->b_ml.ml_mfp == NULL && !empty_memline)
      || did_cmd || nofile_err
      || aborting()) {
    if (buf != NULL && (cmdmod.cmod_flags & CMOD_LOCKMARKS)) {
      // restore the original '[ and '] positions
      buf->b_op_start = orig_start;
      buf->b_op_end = orig_end;
    }

    no_wait_return--;
    msg_scroll = msg_save;
    if (nofile_err) {
      semsg(_(e_no_matching_autocommands_for_buftype_str_buffer), curbuf->b_p_bt);
    }

    if (nofile_err || aborting()) {
      // An aborting error, interrupt or exception in the
      // autocommands.
      return FAIL;
    }
    if (did_cmd) {
      if (buf == NULL) {
        // The buffer was deleted.  We assume it was written
        // (can't retry anyway).
        return OK;
      }
      if (overwriting) {
        // Assume the buffer was written, update the timestamp.
        ml_timestamp(buf);
        if (append) {
          buf->b_flags &= ~BF_NEW;
        } else {
          buf->b_flags &= ~BF_WRITE_MASK;
        }
      }
      if (reset_changed && buf->b_changed && !append
          && (overwriting || vim_strchr(p_cpo, CPO_PLUS) != NULL)) {
        // Buffer still changed, the autocommands didn't work properly.
        return FAIL;
      }
      return OK;
    }
    if (!aborting()) {
      emsg(_("E203: Autocommands deleted or unloaded buffer to be written"));
    }
    return FAIL;
  }

  // The autocommands may have changed the number of lines in the file.
  // When writing the whole file, adjust the end.
  // When writing part of the file, assume that the autocommands only
  // changed the number of lines that are to be written (tricky!).
  if (buf->b_ml.ml_line_count != old_line_count) {
    if (whole) {                                              // write all
      *endp = buf->b_ml.ml_line_count;
    } else if (buf->b_ml.ml_line_count > old_line_count) {           // more lines
      *endp += buf->b_ml.ml_line_count - old_line_count;
    } else {                                                    // less lines
      *endp -= old_line_count - buf->b_ml.ml_line_count;
      if (*endp < start) {
        no_wait_return--;
        msg_scroll = msg_save;
        emsg(_("E204: Autocommand changed number of lines in unexpected way"));
        return FAIL;
      }
    }
  }

  // The autocommands may have changed the name of the buffer, which may
  // be kept in fname, ffname and sfname.
  if (buf_ffname) {
    *ffnamep = buf->b_ffname;
  }
  if (buf_sfname) {
    *sfnamep = buf->b_sfname;
  }
  if (buf_fname_f) {
    *fnamep = buf->b_ffname;
  }
  if (buf_fname_s) {
    *fnamep = buf->b_sfname;
  }
  return NOTDONE;
}

static void buf_write_do_post_autocmds(buf_T *buf, char *fname, exarg_T *eap, bool append,
                                       bool filtering, bool reset_changed, bool whole)
{
  aco_save_T aco;

  curbuf->b_no_eol_lnum = 0;      // in case it was set by the previous read

  // Apply POST autocommands.
  // Careful: The autocommands may call buf_write() recursively!
  aucmd_prepbuf(&aco, buf);

  if (append) {
    apply_autocmds_exarg(EVENT_FILEAPPENDPOST, fname, fname,
                         false, curbuf, eap);
  } else if (filtering) {
    apply_autocmds_exarg(EVENT_FILTERWRITEPOST, NULL, fname,
                         false, curbuf, eap);
  } else if (reset_changed && whole) {
    apply_autocmds_exarg(EVENT_BUFWRITEPOST, fname, fname,
                         false, curbuf, eap);
  } else {
    apply_autocmds_exarg(EVENT_FILEWRITEPOST, fname, fname,
                         false, curbuf, eap);
  }

  // restore curwin/curbuf and a few other things
  aucmd_restbuf(&aco);
}

static inline Error_T set_err_num(const char *num, const char *msg)
{
  return rs_set_err_num(num, msg);
}

static inline Error_T set_err(const char *msg)
{
  return rs_set_err(msg);
}

static inline Error_T set_err_arg(const char *msg, int arg)
{
  return rs_set_err_arg(msg, arg);
}

static void emit_err(Error_T *e)
{
  rs_emit_err(e);
}

static int get_fileinfo_os(char *fname, FileInfo *file_info_old, bool overwriting, int *perm,
                           bool *device, bool *newfile, Error_T *err)
{
  return rs_get_fileinfo_os(fname, file_info_old, overwriting, perm, device, newfile, err);
}

static int get_fileinfo(buf_T *buf, char *fname, bool overwriting, bool forceit,
                        FileInfo *file_info_old, int *perm, bool *device, bool *newfile,
                        bool *readonly, Error_T *err)
{
  return rs_get_fileinfo(buf, fname, overwriting, forceit, file_info_old, perm, device, newfile,
                         readonly, err);
}

static int buf_write_make_backup(char *fname, bool append, FileInfo *file_info_old, vim_acl_T acl,
                                 int perm, unsigned bkc, bool file_readonly, bool forceit,
                                 bool *backup_copyp, char **backupp, Error_T *err)
{
  return rs_buf_write_make_backup(fname, append, file_info_old, acl, perm, bkc,
                                  file_readonly, forceit, backup_copyp, backupp, err);
}

/// buf_write() - write to file "fname" lines "start" through "end"
///
/// We do our own buffering here because fwrite() is so slow.
///
/// If "forceit" is true, we don't care for errors when attempting backups.
/// In case of an error everything possible is done to restore the original
/// file.  But when "forceit" is true, we risk losing it.
///
/// When "reset_changed" is true and "append" == false and "start" == 1 and
/// "end" == curbuf->b_ml.ml_line_count, reset curbuf->b_changed.
///
/// This function must NOT use NameBuff (because it's called by autowrite()).
///
///
/// @param eap     for forced 'ff' and 'fenc', can be NULL!
/// @param append  append to the file
///
/// @return        FAIL for failure, OK otherwise
int buf_write(buf_T *buf, char *fname, char *sfname, linenr_T start, linenr_T end, exarg_T *eap,
              bool append, bool forceit, bool reset_changed, bool filtering)
{
  int retval = OK;
  int msg_save = msg_scroll;
  bool prev_got_int = got_int;
  // writing everything
  bool whole = (start == 1 && end == buf->b_ml.ml_line_count);
  bool write_undo_file = false;
  context_sha256_T sha_ctx;
  unsigned bkc = get_bkc_flags(buf);

  if (fname == NULL || *fname == NUL) {  // safety check
    return FAIL;
  }
  if (buf->b_ml.ml_mfp == NULL) {
    // This can happen during startup when there is a stray "w" in the
    // vimrc file.
    emsg(_(e_empty_buffer));
    return FAIL;
  }

  // Disallow writing in secure mode.
  if (check_secure()) {
    return FAIL;
  }

  // Avoid a crash for a long name.
  if (strlen(fname) >= MAXPATHL) {
    emsg(_(e_longname));
    return FAIL;
  }

  // must init bw_conv_buf and bw_iconv_fd before jumping to "fail"
  struct bw_info write_info;            // info for buf_write_bytes()
  write_info.bw_conv_buf = NULL;
  write_info.bw_conv_error = false;
  write_info.bw_conv_error_lnum = 0;
  write_info.bw_restlen = 0;
  write_info.bw_iconv_fd = (iconv_t)-1;

  // After writing a file changedtick changes but we don't want to display
  // the line.
  ex_no_reprint = true;

  // If there is no file name yet, use the one for the written file.
  // BF_NOTEDITED is set to reflect this (in case the write fails).
  // Don't do this when the write is for a filter command.
  // Don't do this when appending.
  // Only do this when 'cpoptions' contains the 'F' flag.
  if (buf->b_ffname == NULL
      && reset_changed
      && whole
      && buf == curbuf
      && !bt_nofilename(buf)
      && !filtering
      && (!append || vim_strchr(p_cpo, CPO_FNAMEAPP) != NULL)
      && vim_strchr(p_cpo, CPO_FNAMEW) != NULL) {
    if (set_rw_fname(fname, sfname) == FAIL) {
      return FAIL;
    }
    buf = curbuf;           // just in case autocmds made "buf" invalid
  }

  if (sfname == NULL) {
    sfname = fname;
  }

  // For Unix: Use the short file name whenever possible.
  // Avoids problems with networks and when directory names are changed.
  // Don't do this for Windows, a "cd" in a sub-shell may have moved us to
  // another directory, which we don't detect.
  char *ffname = fname;                           // remember full fname
#ifdef UNIX
  fname = sfname;
#endif

  // true if writing over original
  bool overwriting = buf->b_ffname != NULL && path_fnamecmp(ffname, buf->b_ffname) == 0;

  no_wait_return++;                 // don't wait for return yet

  const pos_T orig_start = buf->b_op_start;
  const pos_T orig_end = buf->b_op_end;

  // Set '[ and '] marks to the lines to be written.
  buf->b_op_start.lnum = start;
  buf->b_op_start.col = 0;
  buf->b_op_end.lnum = end;
  buf->b_op_end.col = 0;

  int res = buf_write_do_autocmds(buf, &fname, &sfname, &ffname, start, &end, eap, append,
                                  filtering, reset_changed, overwriting, whole, orig_start,
                                  orig_end);
  if (res != NOTDONE) {
    return res;
  }

  if (cmdmod.cmod_flags & CMOD_LOCKMARKS) {
    // restore the original '[ and '] positions
    buf->b_op_start = orig_start;
    buf->b_op_end = orig_end;
  }

  if (shortmess(SHM_OVER) && !exiting) {
    msg_scroll = false;             // overwrite previous file message
  } else {
    msg_scroll = true;              // don't overwrite previous file message
  }
  if (!filtering) {
    msg_ext_set_kind("bufwrite");
    // show that we are busy
#ifndef UNIX
    filemess(buf, sfname, "");
#else
    filemess(buf, fname, "");
#endif
  }
  msg_scroll = false;               // always overwrite the file message now

  char *buffer = verbose_try_malloc(WRITEBUFSIZE);
  int bufsize;
  char smallbuf[SMALLBUFSIZE];
  // can't allocate big buffer, use small one (to be able to write when out of
  // memory)
  if (buffer == NULL) {
    buffer = smallbuf;
    bufsize = SMALLBUFSIZE;
  } else {
    bufsize = WRITEBUFSIZE;
  }

  Error_T err = { 0 };
  int perm;              // file permissions
  bool newfile = false;  // true if file doesn't exist yet
  bool device = false;   // writing to a device
  bool file_readonly = false;  // overwritten file is read-only
  char *backup = NULL;
  char *fenc_tofree = NULL;   // allocated "fenc"

  // Get information about original file (if there is one).
  FileInfo file_info_old;

  vim_acl_T acl = NULL;                 // ACL copied from original file to
                                        // backup or new file

  if (get_fileinfo(buf, fname, overwriting, forceit, &file_info_old, &perm, &device, &newfile,
                   &file_readonly, &err) == FAIL) {
    goto fail;
  }

  // For systems that support ACL: get the ACL from the original file.
  if (!newfile) {
    acl = os_get_acl(fname);
  }

  // If 'backupskip' is not empty, don't make a backup for some files.
  bool dobackup = (p_wb || p_bk || *p_pm != NUL);
  if (dobackup && *p_bsk != NUL && match_file_list(p_bsk, sfname, ffname)) {
    dobackup = false;
  }

  bool backup_copy = false;  // copy the original file?

  // Save the value of got_int and reset it.  We don't want a previous
  // interruption cancel writing, only hitting CTRL-C while writing should
  // abort it.
  prev_got_int = got_int;
  got_int = false;

  // Mark the buffer as 'being saved' to prevent changed buffer warnings
  buf->b_saving = true;

  // If we are not appending or filtering, the file exists, and the
  // 'writebackup', 'backup' or 'patchmode' option is set, need a backup.
  // When 'patchmode' is set also make a backup when appending.
  //
  // Do not make any backup, if 'writebackup' and 'backup' are both switched
  // off.  This helps when editing large files on almost-full disks.
  if (!(append && *p_pm == NUL) && !filtering && perm >= 0 && dobackup) {
    if (buf_write_make_backup(fname, append, &file_info_old, acl, perm, bkc, file_readonly, forceit,
                              &backup_copy, &backup, &err) == FAIL) {
      retval = FAIL;
      goto fail;
    }
  }

#if defined(UNIX)
  bool made_writable = false;  // 'w' bit has been set

  // When using ":w!" and the file was read-only: make it writable
  if (forceit && perm >= 0 && !(perm & 0200)
      && file_info_old.stat.st_uid == getuid()
      && vim_strchr(p_cpo, CPO_FWRITE) == NULL) {
    perm |= 0200;
    os_setperm(fname, perm);
    made_writable = true;
  }
#endif

  // When using ":w!" and writing to the current file, 'readonly' makes no
  // sense, reset it, unless 'Z' appears in 'cpoptions'.
  if (forceit && overwriting && vim_strchr(p_cpo, CPO_KEEPRO) == NULL) {
    buf->b_p_ro = false;
    need_maketitle = true;          // set window title later
    status_redraw_all();            // redraw status lines later
  }

  end = MIN(end, buf->b_ml.ml_line_count);
  if (buf->b_ml.ml_flags & ML_EMPTY) {
    start = end + 1;
  }

  char *wfname = NULL;       // name of file to write to

  // If the original file is being overwritten, there is a small chance that
  // we crash in the middle of writing. Therefore the file is preserved now.
  // This makes all block numbers positive so that recovery does not need
  // the original file.
  // Don't do this if there is a backup file and we are exiting.
  if (reset_changed && !newfile && overwriting && !(exiting && backup != NULL)) {
    ml_preserve(buf, false, !!p_fs);
    if (got_int) {
      err = set_err(_(e_interr));
      goto restore_backup;
    }
  }

  // Default: write the file directly.  May write to a temp file for
  // multi-byte conversion.
  wfname = fname;

  char *fenc;  // effective 'fileencoding'

  // Check for forced 'fileencoding' from "++opt=val" argument.
  if (eap != NULL && eap->force_enc != 0) {
    fenc = eap->cmd + eap->force_enc;
    fenc = enc_canonize(fenc);
    fenc_tofree = fenc;
  } else {
    fenc = buf->b_p_fenc;
  }

  // Check if the file needs to be converted.
  bool converted = need_conversion(fenc);
  int wb_flags = 0;

  // Check if UTF-8 to UCS-2/4 or Latin1 conversion needs to be done.  Or
  // Latin1 to Unicode conversion.  This is handled in buf_write_bytes().
  // Prepare the flags for it and allocate bw_conv_buf when needed.
  if (converted) {
    wb_flags = get_fio_flags(fenc);
    if (wb_flags & (FIO_UCS2 | FIO_UCS4 | FIO_UTF16 | FIO_UTF8)) {
      // Need to allocate a buffer to translate into.
      if (wb_flags & (FIO_UCS2 | FIO_UTF16 | FIO_UTF8)) {
        write_info.bw_conv_buflen = (size_t)bufsize * 2;
      } else {       // FIO_UCS4
        write_info.bw_conv_buflen = (size_t)bufsize * 4;
      }
      write_info.bw_conv_buf = verbose_try_malloc(write_info.bw_conv_buflen);
      if (!write_info.bw_conv_buf) {
        end = 0;
      }
    }
  }

  if (converted && wb_flags == 0) {
    // Use iconv() conversion when conversion is needed and it's not done
    // internally.
    write_info.bw_iconv_fd = (iconv_t)my_iconv_open(fenc, "utf-8");
    if (write_info.bw_iconv_fd != (iconv_t)-1) {
      // We're going to use iconv(), allocate a buffer to convert in.
      write_info.bw_conv_buflen = (size_t)bufsize * ICONV_MULT;
      write_info.bw_conv_buf = verbose_try_malloc(write_info.bw_conv_buflen);
      if (!write_info.bw_conv_buf) {
        end = 0;
      }
      write_info.bw_first = true;
    } else {
      // When the file needs to be converted with 'charconvert' after
      // writing, write to a temp file instead and let the conversion
      // overwrite the original file.
      if (*p_ccv != NUL) {
        wfname = vim_tempname();
        if (wfname == NULL) {  // Can't write without a tempfile!
          err = set_err(_("E214: Can't find temp file for writing"));
          goto restore_backup;
        }
      }
    }
  }

  bool notconverted = false;

  if (converted && wb_flags == 0
      && write_info.bw_iconv_fd == (iconv_t)-1
      && wfname == fname) {
    if (!forceit) {
      err = set_err(_("E213: Cannot convert (add ! to write without conversion)"));
      goto restore_backup;
    }
    notconverted = true;
  }

  bool no_eol = false;  // no end-of-line written
  int nchars;
  linenr_T lnum;
  int fileformat;
  bool checking_conversion;

  int fd;

  // If conversion is taking place, we may first pretend to write and check
  // for conversion errors.  Then loop again to write for real.
  // When not doing conversion this writes for real right away.
  for (checking_conversion = true;; checking_conversion = false) {
    // There is no need to check conversion when:
    // - there is no conversion
    // - we make a backup file, that can be restored in case of conversion
    // failure.
    if (!converted || dobackup) {
      checking_conversion = false;
    }

    if (checking_conversion) {
      // Make sure we don't write anything.
      fd = -1;
      write_info.bw_fd = fd;
    } else {
      // Open the file "wfname" for writing.
      // We may try to open the file twice: If we can't write to the file
      // and forceit is true we delete the existing file and try to
      // create a new one. If this still fails we may have lost the
      // original file!  (this may happen when the user reached his
      // quotum for number of files).
      // Appending will fail if the file does not exist and forceit is
      // false.
      const int fflags = O_WRONLY | (append
                                     ? (forceit ? (O_APPEND | O_CREAT) : O_APPEND)
                                     : (O_CREAT | O_TRUNC));
      const int mode = perm < 0 ? 0666 : (perm & 0777);

      while ((fd = os_open(wfname, fflags, mode)) < 0) {
        // A forced write will try to create a new file if the old one
        // is still readonly. This may also happen when the directory
        // is read-only. In that case the os_remove() will fail.
        if (err.msg == NULL) {
#ifdef UNIX
          FileInfo file_info;

          // Don't delete the file when it's a hard or symbolic link.
          if ((!newfile && os_fileinfo_hardlinks(&file_info_old) > 1)
              || (os_fileinfo_link(fname, &file_info)
                  && !os_fileinfo_id_equal(&file_info, &file_info_old))) {
            err = set_err(_("E166: Can't open linked file for writing"));
          } else {
            err = set_err_arg(_("E212: Can't open file for writing: %s"), fd);
            if (forceit && vim_strchr(p_cpo, CPO_FWRITE) == NULL && perm >= 0) {
              // we write to the file, thus it should be marked
              // writable after all
              if (!(perm & 0200)) {
                made_writable = true;
              }
              perm |= 0200;
              if (file_info_old.stat.st_uid != getuid()
                  || file_info_old.stat.st_gid != getgid()) {
                perm &= 0777;
              }
              if (!append) {                    // don't remove when appending
                os_remove(wfname);
              }
              continue;
            }
          }
#else
          err = set_err_arg(_("E212: Can't open file for writing: %s"), fd);
          if (forceit && vim_strchr(p_cpo, CPO_FWRITE) == NULL && perm >= 0) {
            if (!append) {                    // don't remove when appending
              os_remove(wfname);
            }
            continue;
          }
#endif
        }

restore_backup:
        {
          // If we failed to open the file, we don't need a backup. Throw it
          // away.  If we moved or removed the original file try to put the
          // backup in its place.
          if (backup != NULL && wfname == fname) {
            if (backup_copy) {
              // There is a small chance that we removed the original,
              // try to move the copy in its place.
              // This may not work if the vim_rename() fails.
              // In that case we leave the copy around.
              // If file does not exist, put the copy in its place
              if (!os_path_exists(fname)) {
                vim_rename(backup, fname);
              }
              // if original file does exist throw away the copy
              if (os_path_exists(fname)) {
                os_remove(backup);
              }
            } else {
              // try to put the original file back
              vim_rename(backup, fname);
            }
          }

          // if original file no longer exists give an extra warning
          if (!newfile && !os_path_exists(fname)) {
            end = 0;
          }
        }

        if (wfname != fname) {
          xfree(wfname);
        }
        goto fail;
      }
      write_info.bw_fd = fd;
    }
    err = set_err(NULL);

    write_info.bw_buf = buffer;
    nchars = 0;

    // use "++bin", "++nobin" or 'binary'
    int write_bin;
    if (eap != NULL && eap->force_bin != 0) {
      write_bin = (eap->force_bin == FORCE_BIN);
    } else {
      write_bin = buf->b_p_bin;
    }

    // Skip the BOM when appending and the file already existed, the BOM
    // only makes sense at the start of the file.
    if (buf->b_p_bomb && !write_bin && (!append || perm < 0)) {
      write_info.bw_len = make_bom(buffer, fenc);
      if (write_info.bw_len > 0) {
        // don't convert
        write_info.bw_flags = FIO_NOCONVERT | wb_flags;
        if (buf_write_bytes(&write_info) == FAIL) {
          end = 0;
        } else {
          nchars += write_info.bw_len;
        }
      }
    }
    write_info.bw_start_lnum = start;

    write_undo_file = (buf->b_p_udf && overwriting && !append
                       && !filtering && reset_changed && !checking_conversion);
    if (write_undo_file) {
      // Prepare for computing the hash value of the text.
      sha256_start(&sha_ctx);
    }

    write_info.bw_len = bufsize;
    write_info.bw_flags = wb_flags;
    fileformat = get_fileformat_force(buf, eap);
    char *s = buffer;
    int len = 0;
    for (lnum = start; lnum <= end; lnum++) {
      // The next while loop is done once for each character written.
      // Keep it fast!
      char *ptr = ml_get_buf(buf, lnum) - 1;
      if (write_undo_file) {
        sha256_update(&sha_ctx, (uint8_t *)ptr + 1, (uint32_t)(strlen(ptr + 1) + 1));
      }
      char c;
      while ((c = *++ptr) != NUL) {
        if (c == NL) {
          *s = NUL;                       // replace newlines with NULs
        } else if (c == CAR && fileformat == EOL_MAC) {
          *s = NL;                        // Mac: replace CRs with NLs
        } else {
          *s = c;
        }
        s++;
        if (++len != bufsize) {
          continue;
        }
        if (buf_write_bytes(&write_info) == FAIL) {
          end = 0;                        // write error: break loop
          break;
        }
        nchars += bufsize;
        s = buffer;
        len = 0;
        write_info.bw_start_lnum = lnum;
      }
      // write failed or last line has no EOL: stop here
      if (end == 0
          || (lnum == end
              && (write_bin || !buf->b_p_fixeol)
              && ((write_bin && lnum == buf->b_no_eol_lnum)
                  || (lnum == buf->b_ml.ml_line_count && !buf->b_p_eol)))) {
        lnum++;                           // written the line, count it
        no_eol = true;
        break;
      }
      if (fileformat == EOL_UNIX) {
        *s++ = NL;
      } else {
        *s++ = CAR;                       // EOL_MAC or EOL_DOS: write CR
        if (fileformat == EOL_DOS) {      // write CR-NL
          if (++len == bufsize) {
            if (buf_write_bytes(&write_info) == FAIL) {
              end = 0;                    // write error: break loop
              break;
            }
            nchars += bufsize;
            s = buffer;
            len = 0;
          }
          *s++ = NL;
        }
      }
      if (++len == bufsize) {
        if (buf_write_bytes(&write_info) == FAIL) {
          end = 0;  // Write error: break loop.
          break;
        }
        nchars += bufsize;
        s = buffer;
        len = 0;

        os_breakcheck();
        if (got_int) {
          end = 0;  // Interrupted, break loop.
          break;
        }
      }
    }
    if (len > 0 && end > 0) {
      write_info.bw_len = len;
      if (buf_write_bytes(&write_info) == FAIL) {
        end = 0;                      // write error
      }
      nchars += len;
    }

    if (!buf->b_p_fixeol && buf->b_p_eof) {
      // write trailing CTRL-Z
      write_eintr(write_info.bw_fd, "\x1a", 1);
    }

    // Stop when writing done or an error was encountered.
    if (!checking_conversion || end == 0) {
      break;
    }

    // If no error happened until now, writing should be ok, so loop to
    // really write the buffer.
  }

  // If we started writing, finish writing. Also when an error was
  // encountered.
  if (!checking_conversion) {
    // On many journalling file systems there is a bug that causes both the
    // original and the backup file to be lost when halting the system right
    // after writing the file.  That's because only the meta-data is
    // journalled.  Syncing the file slows down the system, but assures it has
    // been written to disk and we don't lose it.
    // For a device do try the fsync() but don't complain if it does not work
    // (could be a pipe).
    // If the 'fsync' option is false, don't fsync().  Useful for laptops.
    int error;
    if (p_fs && (error = os_fsync(fd)) != 0 && !device
        // fsync not supported on this storage.
        && error != UV_ENOTSUP) {
      err = set_err_arg(e_fsync, error);
      end = 0;
    }

    if (!backup_copy) {
#ifdef HAVE_XATTR
      os_copy_xattr(backup, wfname);
#endif
    }

#ifdef UNIX
    // When creating a new file, set its owner/group to that of the original
    // file.  Get the new device and inode number.
    if (backup != NULL && !backup_copy) {
      // don't change the owner when it's already OK, some systems remove
      // permission or ACL stuff
      FileInfo file_info;
      if (!os_fileinfo(wfname, &file_info)
          || file_info.stat.st_uid != file_info_old.stat.st_uid
          || file_info.stat.st_gid != file_info_old.stat.st_gid) {
        os_fchown(fd, (uv_uid_t)file_info_old.stat.st_uid, (uv_gid_t)file_info_old.stat.st_gid);
        if (perm >= 0) {  // Set permission again, may have changed.
          os_setperm(wfname, perm);
        }
      }
      buf_set_file_id(buf);
    } else if (!buf->file_id_valid) {
      // Set the file_id when creating a new file.
      buf_set_file_id(buf);
    }
#endif

    if ((error = os_close(fd)) != 0) {
      err = set_err_arg(_("E512: Close failed: %s"), error);
      end = 0;
    }

#ifdef UNIX
    if (made_writable) {
      perm &= ~0200;              // reset 'w' bit for security reasons
    }
#endif
    if (perm >= 0) {  // Set perm. of new file same as old file.
      os_setperm(wfname, perm);
    }
    // Probably need to set the ACL before changing the user (can't set the
    // ACL on a file the user doesn't own).
    if (!backup_copy) {
      os_set_acl(wfname, acl);
    }

    if (wfname != fname) {
      // The file was written to a temp file, now it needs to be converted
      // with 'charconvert' to (overwrite) the output file.
      if (end != 0) {
        if (eval_charconvert("utf-8", fenc, wfname, fname) == FAIL) {
          write_info.bw_conv_error = true;
          end = 0;
        }
      }
      os_remove(wfname);
      xfree(wfname);
    }
  }

  if (end == 0) {
    // Error encountered.
    if (err.msg == NULL) {
      if (write_info.bw_conv_error) {
        if (write_info.bw_conv_error_lnum == 0) {
          err = set_err(_(e_write_error_conversion_failed_make_fenc_empty_to_override));
        } else {
          err = set_err(xmalloc(300));
          err.alloc = true;
          vim_snprintf(err.msg, 300,  // NOLINT(runtime/printf)
                       _(e_write_error_conversion_failed_in_line_nr_make_fenc_empty_to_override),
                       write_info.bw_conv_error_lnum);
        }
      } else if (got_int) {
        err = set_err(_(e_interr));
      } else {
        err = set_err(_(e_write_error_file_system_full));
      }
    }

    // If we have a backup file, try to put it in place of the new file,
    // because the new file is probably corrupt.  This avoids losing the
    // original file when trying to make a backup when writing the file a
    // second time.
    // When "backup_copy" is set we need to copy the backup over the new
    // file.  Otherwise rename the backup file.
    // If this is OK, don't give the extra warning message.
    if (backup != NULL) {
      if (backup_copy) {
        // This may take a while, if we were interrupted let the user
        // know we got the message.
        if (got_int) {
          msg(_(e_interr), 0);
          ui_flush();
        }

        // copy the file.
        if (os_copy(backup, fname, UV_FS_COPYFILE_FICLONE)
            == 0) {
          end = 1;  // success
        }
      } else {
        if (vim_rename(backup, fname) == 0) {
          end = 1;
        }
      }
    }
    goto fail;
  }

  lnum -= start;            // compute number of written lines
  no_wait_return--;         // may wait for return now

#if !defined(UNIX)
  fname = sfname;           // use shortname now, for the messages
#endif
  if (!filtering) {
    add_quoted_fname(IObuff, IOSIZE, buf, fname);
    bool insert_space = false;
    if (write_info.bw_conv_error) {
      xstrlcat(IObuff, _(" CONVERSION ERROR"), IOSIZE);
      insert_space = true;
      if (write_info.bw_conv_error_lnum != 0) {
        vim_snprintf_add(IObuff, IOSIZE, _(" in line %" PRId64 ";"),
                         (int64_t)write_info.bw_conv_error_lnum);
      }
    } else if (notconverted) {
      xstrlcat(IObuff, _("[NOT converted]"), IOSIZE);
      insert_space = true;
    } else if (converted) {
      xstrlcat(IObuff, _("[converted]"), IOSIZE);
      insert_space = true;
    }
    if (device) {
      xstrlcat(IObuff, _("[Device]"), IOSIZE);
      insert_space = true;
    } else if (newfile) {
      xstrlcat(IObuff, _("[New]"), IOSIZE);
      insert_space = true;
    }
    if (no_eol) {
      xstrlcat(IObuff, _("[noeol]"), IOSIZE);
      insert_space = true;
    }
    // may add [unix/dos/mac]
    if (msg_add_fileformat(fileformat)) {
      insert_space = true;
    }
    msg_add_lines(insert_space, lnum, nchars);       // add line/char count
    if (!shortmess(SHM_WRITE)) {
      if (append) {
        xstrlcat(IObuff, shortmess(SHM_WRI) ? _(" [a]") : _(" appended"), IOSIZE);
      } else {
        xstrlcat(IObuff, shortmess(SHM_WRI) ? _(" [w]") : _(" written"), IOSIZE);
      }
    }

    msg_ext_set_kind("bufwrite");
    msg_ext_overwrite = true;
    set_keep_msg(msg_trunc(IObuff, false, 0), 0);
  }

  // When written everything correctly: reset 'modified'.  Unless not
  // writing to the original file and '+' is not in 'cpoptions'.
  if (reset_changed && whole && !append
      && !write_info.bw_conv_error
      && (overwriting || vim_strchr(p_cpo, CPO_PLUS) != NULL)) {
    unchanged(buf, true, false);
    const varnumber_T changedtick = buf_get_changedtick(buf);
    if (buf->b_last_changedtick + 1 == changedtick) {
      // b:changedtick may be incremented in unchanged() but that should not
      // trigger a TextChanged event.
      buf->b_last_changedtick = changedtick;
    }
    u_unchanged(buf);
    u_update_save_nr(buf);
  }

  // If written to the current file, update the timestamp of the swap file
  // and reset the BF_WRITE_MASK flags. Also sets buf->b_mtime.
  if (overwriting) {
    ml_timestamp(buf);
    if (append) {
      buf->b_flags &= ~BF_NEW;
    } else {
      buf->b_flags &= ~BF_WRITE_MASK;
    }
  }

  // If we kept a backup until now, and we are in patch mode, then we make
  // the backup file our 'original' file.
  if (*p_pm && dobackup) {
    char *const org = modname(fname, p_pm, false);

    if (backup != NULL) {
      // If the original file does not exist yet
      // the current backup file becomes the original file
      if (org == NULL) {
        emsg(_("E205: Patchmode: can't save original file"));
      } else if (!os_path_exists(org)) {
        vim_rename(backup, org);
        XFREE_CLEAR(backup);                   // don't delete the file
#ifdef UNIX
        os_file_settime(org,
                        (double)file_info_old.stat.st_atim.tv_sec,
                        (double)file_info_old.stat.st_mtim.tv_sec);
#endif
      }
    } else {
      // If there is no backup file, remember that a (new) file was
      // created.
      int empty_fd;

      if (org == NULL
          || (empty_fd = os_open(org,
                                 O_CREAT | O_EXCL | O_NOFOLLOW,
                                 perm < 0 ? 0666 : (perm & 0777))) < 0) {
        emsg(_(e_patchmode_cant_touch_empty_original_file));
      } else {
        close(empty_fd);
      }
    }
    if (org != NULL) {
      os_setperm(org, os_getperm(fname) & 0777);
      xfree(org);
    }
  }

  // Remove the backup unless 'backup' option is set
  if (!p_bk && backup != NULL
      && !write_info.bw_conv_error
      && os_remove(backup) != 0) {
    emsg(_("E207: Can't delete backup file"));
  }

  goto nofail;

  // Finish up.  We get here either after failure or success.
fail:
  no_wait_return--;             // may wait for return now
nofail:

  // Done saving, we accept changed buffer warnings again
  buf->b_saving = false;

  xfree(backup);
  if (buffer != smallbuf) {
    xfree(buffer);
  }
  xfree(fenc_tofree);
  xfree(write_info.bw_conv_buf);
  if (write_info.bw_iconv_fd != (iconv_t)-1) {
    iconv_close(write_info.bw_iconv_fd);
    write_info.bw_iconv_fd = (iconv_t)-1;
  }
  os_free_acl(acl);

  if (err.msg != NULL) {
    // - 100 to save some space for further error message
#ifndef UNIX
    add_quoted_fname(IObuff, IOSIZE - 100, buf, sfname);
#else
    add_quoted_fname(IObuff, IOSIZE - 100, buf, fname);
#endif
    emit_err(&err);

    retval = FAIL;
    if (end == 0) {
      const int hl_id = HLF_E;  // Set highlight for error messages.
      msg_puts_hl(_("\nWARNING: Original file may be lost or damaged\n"), hl_id, true);
      msg_puts_hl(_("don't quit the editor until the file is successfully written!"), hl_id, true);

      // Update the timestamp to avoid an "overwrite changed file"
      // prompt when writing again.
      if (os_fileinfo(fname, &file_info_old)) {
        buf_store_file_info(buf, &file_info_old);
        buf->b_mtime_read = buf->b_mtime;
        buf->b_mtime_read_ns = buf->b_mtime_ns;
      }
    }
  }
  msg_scroll = msg_save;

  // When writing the whole file and 'undofile' is set, also write the undo
  // file.
  if (retval == OK && write_undo_file) {
    uint8_t hash[UNDO_HASH_SIZE];

    sha256_finish(&sha_ctx, hash);
    u_write_undo(NULL, false, buf, hash);
  }

  if (!should_abort(retval)) {
    buf_write_do_post_autocmds(buf, fname, eap, append, filtering, reset_changed, whole);
    if (aborting()) {       // autocmds may abort script processing
      retval = false;
    }
  }

  got_int |= prev_got_int;

  return retval;
}
