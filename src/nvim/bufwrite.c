// bufwrite.c: functions for writing a buffer

#include <iconv.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>
#include <sys/stat.h>

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

// Static assertions for constants used in Rust code only (no public Rust constant).
// Assertions for constants with matching Rust pub consts in nvim-bufwrite/src/lib.rs
// or nvim-bufwrite/src/ffi.rs have been removed: FIO_LATIN1, FIO_UTF8, FIO_ENDIAN_L,
// FIO_NOCONVERT, FIO_UCSBOM, FIO_ALL, CONV_RESTLEN, WRITEBUFSIZE, OK, FAIL, NOTDONE,
// EOL_UNIX, EOL_DOS, EOL_MAC.
_Static_assert(FIO_UCS2 == 0x04, "FIO_UCS2");
_Static_assert(FIO_UCS4 == 0x08, "FIO_UCS4");
_Static_assert(FIO_UTF16 == 0x10, "FIO_UTF16");
_Static_assert(ICONV_MULT == 8, "ICONV_MULT");
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

// Rust FFI forward declarations (used by accessor functions below)
extern int rs_time_differs(int64_t file_sec, int64_t file_nsec, int64_t mtime, int64_t mtime_ns,
                           int fat_tolerance);

static const char e_no_matching_autocommands_for_buftype_str_buffer[]
  = N_("E676: No matching autocommands for buftype=%s buffer");

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

// C accessor functions for Rust FFI

void nvim_bw_semsg_2(const char *fmt, const char *a, const char *b)
{
  (void)b;  // b is unused; Rust passes null for single-arg format
  semsg(fmt, a);
}

void nvim_bw_semsg_3(const char *fmt, const char *a, const char *b, const char *c) { semsg(fmt, a, b, c); }

void nvim_bw_semsg_4(const char *fmt, const char *a, const char *b, const char *c, const char *d)
{
  semsg(fmt, a, b, c, d);
}

const char *nvim_bw_os_strerror(int errnum) { return os_strerror(errnum); }

void nvim_bw_xfree(char *ptr) { xfree(ptr); }


// File info accessors
int nvim_bw_os_nodetype(const char *fname) { return os_nodetype(fname); }

int nvim_bw_fi_get_st_mode(FileInfo *info) { return (int)info->stat.st_mode; }

int nvim_bw_fi_is_regular(FileInfo *info) { return S_ISREG(info->stat.st_mode); }

int nvim_bw_fi_is_dir(FileInfo *info) { return S_ISDIR(info->stat.st_mode); }

// Buffer mtime accessors
int64_t nvim_bw_buf_get_mtime_read(buf_T *buf) { return buf->b_mtime_read; }

int64_t nvim_bw_buf_get_mtime_read_ns(buf_T *buf) { return buf->b_mtime_read_ns; }

int nvim_bw_time_differs(FileInfo *info, int64_t mtime, int64_t mtime_ns)
{
#if defined(__linux__) || defined(MSWIN)
  return rs_time_differs(info->stat.st_mtim.tv_sec, info->stat.st_mtim.tv_nsec,
                         mtime, mtime_ns, 1);
#else
  return rs_time_differs(info->stat.st_mtim.tv_sec, info->stat.st_mtim.tv_nsec,
                         mtime, mtime_ns, 0);
#endif
}

// Options
int nvim_bw_cpo_contains(int c) { return vim_strchr(p_cpo, c) != NULL; }

// Gettext
const char *nvim_bw_gettext(const char *s) { return _(s); }

// bw_info field accessors (use void* for opaque handle pattern)
int nvim_bw_info_get_fd(void *p) { struct bw_info *ip = p; return ip->bw_fd; }
char *nvim_bw_info_get_buf(void *p) { struct bw_info *ip = p; return ip->bw_buf; }
int nvim_bw_info_get_len(void *p) { struct bw_info *ip = p; return ip->bw_len; }
int nvim_bw_info_get_flags(void *p) { struct bw_info *ip = p; return ip->bw_flags; }
int nvim_bw_info_get_restlen(void *p) { struct bw_info *ip = p; return ip->bw_restlen; }
void nvim_bw_info_set_restlen(void *p, int val) { struct bw_info *ip = p; ip->bw_restlen = val; }
uint8_t *nvim_bw_info_get_rest_ptr(void *p) { struct bw_info *ip = p; return ip->bw_rest; }
char *nvim_bw_info_get_conv_buf(void *p) { struct bw_info *ip = p; return ip->bw_conv_buf; }
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

// I/O
int nvim_bw_write_eintr(int fd, const char *buf, size_t len) { return write_eintr(fd, (void *)buf, len); }

// Backup accessors
void nvim_bw_os_set_acl(const char *fname, vim_acl_T acl) { os_set_acl(fname, acl); }
int nvim_bw_os_mkdir_recurse(const char *dir, int32_t mode, char **failed_dir) { return os_mkdir_recurse(dir, mode, failed_dir, NULL); }
void nvim_bw_XFREE_CLEAR(char **pp) { XFREE_CLEAR(*pp); }
size_t nvim_bw_sizeof_FileInfo(void) { return sizeof(FileInfo); }
#ifdef UNIX
uint32_t nvim_bw_fi_get_st_uid(FileInfo *fi) { return (uint32_t)fi->stat.st_uid; }
uint32_t nvim_bw_fi_get_st_gid(FileInfo *fi) { return (uint32_t)fi->stat.st_gid; }
int64_t nvim_bw_fi_get_atime_sec(FileInfo *fi) { return (int64_t)fi->stat.st_atim.tv_sec; }
int64_t nvim_bw_fi_get_mtime_sec(FileInfo *fi) { return (int64_t)fi->stat.st_mtim.tv_sec; }
#else
uint32_t nvim_bw_fi_get_st_uid(FileInfo *fi) { (void)fi; return 0; }
uint32_t nvim_bw_fi_get_st_gid(FileInfo *fi) { (void)fi; return 0; }
int64_t nvim_bw_fi_get_atime_sec(FileInfo *fi) { (void)fi; return 0; }
int64_t nvim_bw_fi_get_mtime_sec(FileInfo *fi) { (void)fi; return 0; }
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
void nvim_bw_buf_set_no_eol_lnum(buf_T *buf, linenr_T lnum) { buf->b_no_eol_lnum = lnum; }

// Global state accessors
int nvim_bw_bt_nofilename(buf_T *buf) { return bt_nofilename(buf); }
int nvim_bw_get_cmdmod_cmod_flags(void) { return cmdmod.cmod_flags; }
void nvim_bw_semsg_nofile_err(buf_T *buf) { semsg(_(e_no_matching_autocommands_for_buftype_str_buffer), buf->b_p_bt); }

// Phase 8+9: buf_write main function accessors

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
int nvim_bw_set_rw_fname(char *fname, char *sfname) { return set_rw_fname(fname, sfname); }
char *nvim_bw_vim_tempname(void) { return vim_tempname(); }
// Encoding

// iconv
void *nvim_bw_my_iconv_open(const char *tocode, const char *fromcode)
{
  return (void *)my_iconv_open((char *)tocode, (char *)fromcode);
}
void nvim_bw_iconv_close(void *cd) { iconv_close((iconv_t)cd); }

// Memline
char *nvim_bw_ml_get_buf(buf_T *buf, linenr_T lnum) { return ml_get_buf(buf, lnum); }

// Message/UI
void nvim_bw_filemess(buf_T *buf, const char *fname, const char *s) { filemess(buf, (char *)fname, (char *)s); }
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
void nvim_bw_ui_flush(void) { ui_flush(); }

// OS operations
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
void nvim_bw_sha256_finish(void *ctx, uint8_t *hash) { sha256_finish((context_sha256_T *)ctx, hash); }

// Undo
void nvim_bw_u_write_undo(buf_T *buf, const uint8_t *hash) { u_write_undo(NULL, false, buf, (uint8_t *)hash); }

// Eval
int nvim_bw_eval_charconvert(const char *from, const char *to, const char *src, const char *dst)
{
  return eval_charconvert(from, to, src, dst);
}

// I/O - direct write_eintr (for use in write loop)
int nvim_bw_write_eintr_direct(int fd, const char *buf, size_t len) { return write_eintr(fd, (void *)buf, len); }

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


#include "bufwrite.c.generated.h"

