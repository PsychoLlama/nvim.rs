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
_Static_assert(CMOD_LOCKMARKS == 0x0800, "CMOD_LOCKMARKS");
_Static_assert(BF_NEW == 0x10, "BF_NEW");
_Static_assert(BF_WRITE_MASK == 0x58, "BF_WRITE_MASK");
_Static_assert(EVENT_FILEAPPENDCMD == 48, "EVENT_FILEAPPENDCMD");
_Static_assert(EVENT_FILEAPPENDPRE == 50, "EVENT_FILEAPPENDPRE");
_Static_assert(EVENT_FILEAPPENDPOST == 49, "EVENT_FILEAPPENDPOST");
_Static_assert(EVENT_FILTERWRITEPRE == 65, "EVENT_FILTERWRITEPRE");
_Static_assert(EVENT_FILTERWRITEPOST == 64, "EVENT_FILTERWRITEPOST");
_Static_assert(EVENT_BUFWRITECMD == 20, "EVENT_BUFWRITECMD");
_Static_assert(EVENT_BUFWRITEPRE == 22, "EVENT_BUFWRITEPRE");
_Static_assert(EVENT_BUFWRITEPOST == 21, "EVENT_BUFWRITEPOST");
_Static_assert(EVENT_FILEWRITECMD == 59, "EVENT_FILEWRITECMD");
_Static_assert(EVENT_FILEWRITEPRE == 61, "EVENT_FILEWRITEPRE");
_Static_assert(EVENT_FILEWRITEPOST == 60, "EVENT_FILEWRITEPOST");
_Static_assert(ML_EMPTY == 0x01, "ML_EMPTY");
_Static_assert(FORCE_BIN == 1, "FORCE_BIN");
_Static_assert(SHA256_SUM_SIZE == 32, "UNDO_HASH_SIZE");

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
_Static_assert(sizeof(Error_T) == 24, "Error_T size");

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

// Autocmd accessors
// aco_save_T / bufref_T as opaque handles
size_t nvim_bw_sizeof_aco_save(void) { return sizeof(aco_save_T); }
size_t nvim_bw_sizeof_bufref(void) { return sizeof(bufref_T); }
void nvim_bw_aucmd_prepbuf(void *aco, buf_T *buf) { aucmd_prepbuf((aco_save_T *)aco, buf); }
void nvim_bw_aucmd_restbuf(void *aco) { aucmd_restbuf((aco_save_T *)aco); }
void nvim_bw_set_bufref(void *br, buf_T *buf) { set_bufref((bufref_T *)br, buf); }
int nvim_bw_bufref_valid(void *br) { return bufref_valid((bufref_T *)br); }
int nvim_bw_apply_autocmds_exarg(int event, char *fname, char *fname_io,
                                  int force, buf_T *buf, exarg_T *eap) {
  return apply_autocmds_exarg((event_T)event, fname, fname_io, force != 0, buf, eap);
}

// Buffer field accessors
linenr_T nvim_bw_buf_get_ml_line_count(buf_T *buf) { return buf->b_ml.ml_line_count; }
int nvim_bw_buf_get_ml_mfp_nonnull(buf_T *buf) { return buf->b_ml.ml_mfp != NULL; }
char *nvim_bw_buf_get_ffname(buf_T *buf) { return buf->b_ffname; }
char *nvim_bw_buf_get_sfname(buf_T *buf) { return buf->b_sfname; }
pos_T nvim_bw_buf_get_op_start(buf_T *buf) { return buf->b_op_start; }
void nvim_bw_buf_set_op_start(buf_T *buf, pos_T pos) { buf->b_op_start = pos; }
pos_T nvim_bw_buf_get_op_end(buf_T *buf) { return buf->b_op_end; }
void nvim_bw_buf_set_op_end(buf_T *buf, pos_T pos) { buf->b_op_end = pos; }
int nvim_bw_buf_get_flags(buf_T *buf) { return buf->b_flags; }
void nvim_bw_buf_set_flags(buf_T *buf, int flags) { buf->b_flags = flags; }
int nvim_bw_buf_get_changed(buf_T *buf) { return buf->b_changed; }
const char *nvim_bw_buf_get_p_bt(buf_T *buf) { return buf->b_p_bt; }
void nvim_bw_buf_set_no_eol_lnum(buf_T *buf, linenr_T lnum) { buf->b_no_eol_lnum = lnum; }

// Global state accessors
buf_T *nvim_bw_get_curbuf(void) { return curbuf; }
int nvim_bw_curbufIsChanged(void) { return curbufIsChanged(); }
void nvim_bw_u_unchanged(buf_T *buf) { u_unchanged(buf); }
void nvim_bw_u_update_save_nr(buf_T *buf) { u_update_save_nr(buf); }
void nvim_bw_ml_timestamp(buf_T *buf) { ml_timestamp(buf); }
int nvim_bw_bt_nofilename(buf_T *buf) { return bt_nofilename(buf); }
int nvim_bw_get_no_wait_return(void) { return no_wait_return; }
void nvim_bw_dec_no_wait_return(void) { no_wait_return--; }
int nvim_bw_get_cmdmod_cmod_flags(void) { return cmdmod.cmod_flags; }
int nvim_bw_aborting(void) { return aborting(); }
void nvim_bw_semsg_nofile_err(buf_T *buf) { semsg(_(e_no_matching_autocommands_for_buftype_str_buffer), buf->b_p_bt); }

// Phase 8+9: buf_write main function accessors

// Global state
int nvim_bw_get_got_int(void) { return got_int; }
void nvim_bw_set_got_int(int val) { got_int = val != 0; }
int nvim_bw_get_exiting(void) { return exiting; }
void nvim_bw_set_ex_no_reprint(int val) { ex_no_reprint = val != 0; }
void nvim_bw_set_msg_ext_overwrite(int val) { msg_ext_overwrite = val != 0; }
void nvim_bw_set_need_maketitle(int val) { need_maketitle = val != 0; }
void nvim_bw_inc_no_wait_return(void) { no_wait_return++; }

// Options
int nvim_bw_get_p_wb(void) { return p_wb; }
char *nvim_bw_get_p_pm(void) { return p_pm; }
char *nvim_bw_get_p_bsk(void) { return p_bsk; }
char *nvim_bw_get_p_ccv(void) { return p_ccv; }
int nvim_bw_get_p_fs(void) { return p_fs; }

// Buffer field accessors - phase 8+9
int nvim_bw_buf_get_ml_flags(buf_T *buf) { return buf->b_ml.ml_flags; }
int nvim_bw_buf_get_p_bin(buf_T *buf) { return buf->b_p_bin; }
int nvim_bw_buf_get_p_bomb(buf_T *buf) { return buf->b_p_bomb; }
int nvim_bw_buf_get_p_eol(buf_T *buf) { return buf->b_p_eol; }
int nvim_bw_buf_get_p_eof(buf_T *buf) { return buf->b_p_eof; }
int nvim_bw_buf_get_p_fixeol(buf_T *buf) { return buf->b_p_fixeol; }
void nvim_bw_buf_set_p_ro(buf_T *buf, int val) { buf->b_p_ro = val; }
int nvim_bw_buf_get_p_udf(buf_T *buf) { return buf->b_p_udf; }
char *nvim_bw_buf_get_p_fenc(buf_T *buf) { return buf->b_p_fenc; }
linenr_T nvim_bw_buf_get_no_eol_lnum(buf_T *buf) { return buf->b_no_eol_lnum; }
void nvim_bw_buf_set_saving(buf_T *buf, int val) { buf->b_saving = val != 0; }
int64_t nvim_bw_buf_get_changedtick(buf_T *buf) { return buf_get_changedtick(buf); }
int64_t nvim_bw_buf_get_last_changedtick(buf_T *buf) { return buf->b_last_changedtick; }
void nvim_bw_buf_set_last_changedtick(buf_T *buf, int64_t val) { buf->b_last_changedtick = val; }
int nvim_bw_buf_get_file_id_valid(buf_T *buf) { return buf->file_id_valid; }
void nvim_bw_buf_set_file_id(buf_T *buf) { buf_set_file_id(buf); }
void nvim_bw_buf_store_file_info(buf_T *buf, FileInfo *fi) { buf_store_file_info(buf, fi); }
int64_t nvim_bw_buf_get_mtime(buf_T *buf) { return buf->b_mtime; }
int64_t nvim_bw_buf_get_mtime_ns(buf_T *buf) { return buf->b_mtime_ns; }
void nvim_bw_buf_set_mtime_read(buf_T *buf, int64_t val) { buf->b_mtime_read = val; }
void nvim_bw_buf_set_mtime_read_ns(buf_T *buf, int64_t val) { buf->b_mtime_read_ns = val; }

// Exarg accessors
int nvim_bw_eap_get_force_enc(exarg_T *eap) { return eap ? eap->force_enc : 0; }
char *nvim_bw_eap_get_cmd(exarg_T *eap) { return eap ? eap->cmd : NULL; }
int nvim_bw_eap_get_force_bin(exarg_T *eap) { return eap ? eap->force_bin : 0; }

// File/Path
int nvim_bw_check_secure(void) { return check_secure(); }
int nvim_bw_path_fnamecmp(const char *a, const char *b) { return path_fnamecmp(a, b); }
unsigned nvim_bw_get_bkc_flags(buf_T *buf) { return get_bkc_flags(buf); }
int nvim_bw_set_rw_fname(char *fname, char *sfname) { return set_rw_fname(fname, sfname); }
char *nvim_bw_vim_tempname(void) { return vim_tempname(); }
int nvim_bw_match_file_list(const char *list, const char *sfname, const char *ffname)
{
  return match_file_list((char *)list, (char *)sfname, (char *)ffname);
}

// Encoding
char *nvim_bw_enc_canonize(const char *enc) { return enc_canonize((char *)enc); }
int nvim_bw_need_conversion(const char *fenc) { return need_conversion(fenc); }
int nvim_bw_get_fileformat_force(buf_T *buf, exarg_T *eap) { return get_fileformat_force(buf, eap); }

// iconv
void *nvim_bw_my_iconv_open(const char *tocode, const char *fromcode)
{
  return (void *)my_iconv_open(tocode, fromcode);
}
void nvim_bw_iconv_close(void *cd) { iconv_close((iconv_t)cd); }

// Memline
char *nvim_bw_ml_get_buf(buf_T *buf, linenr_T lnum) { return ml_get_buf(buf, lnum); }
void nvim_bw_ml_preserve(buf_T *buf, int message, int do_fsync)
{
  ml_preserve(buf, message != 0, do_fsync != 0);
}

// Message/UI
void nvim_bw_shortmess_emit(int c, int *result) { *result = shortmess(c); }
void nvim_bw_filemess(buf_T *buf, const char *fname, const char *s) { filemess(buf, (char *)fname, (char *)s); }
void nvim_bw_msg_ext_set_kind(const char *kind) { msg_ext_set_kind(kind); }
void nvim_bw_add_quoted_fname(char *buf, int bufsize, buf_T *bp, const char *fname)
{
  add_quoted_fname(buf, (size_t)bufsize, bp, fname);
}
void nvim_bw_xstrlcat(char *dst, const char *src, size_t dsize) { xstrlcat(dst, src, dsize); }
void nvim_bw_vim_snprintf_add(char *buf, size_t len, const char *fmt, int64_t val)
{
  vim_snprintf_add(buf, len, fmt, (long long)val);
}
int nvim_bw_msg_add_fileformat(int fileformat) { return msg_add_fileformat(fileformat); }
void nvim_bw_msg_add_lines(int insert_space, linenr_T lnum, int nchars)
{
  msg_add_lines(insert_space, lnum, (off_T)nchars);
}
char *nvim_bw_msg_trunc(char *s, int force, int attr) { return msg_trunc(s, force != 0, attr); }
void nvim_bw_set_keep_msg(char *s, int attr) { set_keep_msg(s, attr); }
void nvim_bw_msg_puts_hl(const char *s, int hl_id, int wrap) { msg_puts_hl(s, hl_id, wrap != 0); }
void nvim_bw_ui_flush(void) { ui_flush(); }
void nvim_bw_status_redraw_all(void) { status_redraw_all(); }

// Buffer change tracking
void nvim_bw_unchanged(buf_T *buf, int mesg, int ff)
{
  unchanged(buf, mesg != 0, ff != 0);
}

// OS operations
int nvim_bw_os_fsync(int fd) { return os_fsync(fd); }
vim_acl_T nvim_bw_os_get_acl(const char *fname) { return os_get_acl(fname); }
void nvim_bw_os_free_acl(vim_acl_T acl) { os_free_acl(acl); }
void nvim_bw_os_breakcheck(void) { os_breakcheck(); }

// Allocation
void *nvim_bw_verbose_try_malloc(size_t size) { return verbose_try_malloc(size); }
void *nvim_bw_xmalloc(size_t size) { return xmalloc(size); }
void nvim_bw_vim_snprintf(char *buf, size_t len, const char *fmt, int64_t val)
{
  vim_snprintf(buf, len, fmt, (long long)val);
}

// SHA256
size_t nvim_bw_sizeof_sha256_ctx(void) { return sizeof(context_sha256_T); }
void nvim_bw_sha256_start(void *ctx) { sha256_start((context_sha256_T *)ctx); }
void nvim_bw_sha256_update(void *ctx, const uint8_t *data, uint32_t len)
{
  sha256_update((context_sha256_T *)ctx, data, (size_t)len);
}
void nvim_bw_sha256_finish(void *ctx, uint8_t *hash)
{
  sha256_finish((context_sha256_T *)ctx, hash);
}

// Undo
void nvim_bw_u_write_undo(buf_T *buf, const uint8_t *hash)
{
  u_write_undo(NULL, false, buf, (uint8_t *)hash);
}

// Eval
int nvim_bw_eval_charconvert(const char *from, const char *to, const char *src, const char *dst)
{
  return eval_charconvert(from, to, src, dst);
}
int nvim_bw_should_abort(int retval) { return should_abort(retval); }

// I/O - direct write_eintr (for use in write loop)
int nvim_bw_write_eintr_direct(int fd, const char *buf, size_t len)
{
  return write_eintr(fd, (void *)buf, len);
}

// bw_info management
size_t nvim_bw_sizeof_bw_info(void) { return sizeof(struct bw_info); }
void nvim_bw_info_init(void *p) { memset(p, 0, sizeof(struct bw_info)); struct bw_info *ip = p; ip->bw_iconv_fd = (iconv_t)-1; }
void nvim_bw_info_set_fd(void *p, int fd) { struct bw_info *ip = p; ip->bw_fd = fd; }
void nvim_bw_info_set_buf(void *p, char *buf) { struct bw_info *ip = p; ip->bw_buf = buf; }
void nvim_bw_info_set_len(void *p, int len) { struct bw_info *ip = p; ip->bw_len = len; }
void nvim_bw_info_set_flags(void *p, int flags) { struct bw_info *ip = p; ip->bw_flags = flags; }
void nvim_bw_info_set_conv_buflen(void *p, size_t len) { struct bw_info *ip = p; ip->bw_conv_buflen = len; }
void nvim_bw_info_set_conv_buf(void *p, char *buf) { struct bw_info *ip = p; ip->bw_conv_buf = buf; }
void *nvim_bw_info_get_iconv_fd(void *p) { struct bw_info *ip = p; return (void *)ip->bw_iconv_fd; }
void nvim_bw_info_set_iconv_fd(void *p, void *fd) { struct bw_info *ip = p; ip->bw_iconv_fd = (iconv_t)fd; }

// Open flags
int nvim_bw_open_flags_wronly(void) { return O_WRONLY; }
int nvim_bw_open_flags_append(void) { return O_APPEND; }
int nvim_bw_open_flags_creat(void) { return O_CREAT; }
int nvim_bw_open_flags_trunc(void) { return O_TRUNC; }
int nvim_bw_open_flags_nofollow(void) { return O_NOFOLLOW; }
int nvim_bw_uv_enotsup(void) { return UV_ENOTSUP; }

#ifdef UNIX
uint32_t nvim_bw_getgid(void) { return (uint32_t)getgid(); }
#else
uint32_t nvim_bw_getgid(void) { return 0; }
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
extern int rs_buf_write_do_autocmds(buf_T *buf, char **fnamep, char **sfnamep, char **ffnamep,
                                     linenr_T start, linenr_T *endp, exarg_T *eap, int append,
                                     int filtering, int reset_changed, int overwriting, int whole,
                                     pos_T orig_start, pos_T orig_end);
extern void rs_buf_write_do_post_autocmds(buf_T *buf, char *fname, exarg_T *eap, int append,
                                           int filtering, int reset_changed, int whole);
extern int rs_buf_write(buf_T *buf, char *fname, char *sfname, linenr_T start, linenr_T end,
                        exarg_T *eap, int append, int forceit, int reset_changed, int filtering);

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
  return rs_buf_write_do_autocmds(buf, fnamep, sfnamep, ffnamep, start, endp, eap,
                                   append, filtering, reset_changed, overwriting, whole,
                                   orig_start, orig_end);
}

static void buf_write_do_post_autocmds(buf_T *buf, char *fname, exarg_T *eap, bool append,
                                       bool filtering, bool reset_changed, bool whole)
{
  rs_buf_write_do_post_autocmds(buf, fname, eap, append, filtering, reset_changed, whole);
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
  return rs_buf_write(buf, fname, sfname, start, end, eap, append, forceit, reset_changed,
                      filtering);
}

