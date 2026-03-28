#include <fcntl.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stddef.h>
#include <string.h>
#include <time.h>
#include <uv.h>

#include "auto/config.h"
#include "klib/kvec.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/change.h"
#include "nvim/eval/typval.h"
#include "nvim/mbyte.h"
#include "nvim/eval/vars.h"
#include "nvim/fileio.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/input.h"
#include "nvim/main.h"
#include "nvim/map_defs.h"
#include "nvim/memfile.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
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
#include "nvim/statusline.h"
#include "nvim/strings.h"
#include "nvim/version.h"
#include "nvim/vim_defs.h"

enum {
  DATA_ID = (('d' << 8) + 'a'),  // data block id
  PTR_ID = (('p' << 8) + 't'),   // pointer block id
  BLOCK0_ID0 = 'b',              // block 0 id 0
  BLOCK0_ID1 = '0',              // block 0 id 1
};

// Restrict the numbers to 32 bits, otherwise most compilers will complain.
// This won't detect a 64 bit machine that only swaps a byte in the top 32
// bits, but that is crazy anyway.
enum {
  B0_MAGIC_LONG = 0x30313233,
  B0_MAGIC_INT = 0x20212223,
  B0_MAGIC_SHORT = 0x10111213,
  B0_MAGIC_CHAR = 0x55,
};

// Note: b0_dirty and b0_flags are put at the end of the file name.  For very
// long file names in older versions of Vim they are invalid.
// The 'fileencoding' comes before b0_flags, with a NUL in front.  But only
// when there is room, for very long file names it's omitted.
#define B0_DIRTY        0x55
#define b0_dirty        b0_fname[B0_FNAME_SIZE_ORG - 1]

// The b0_flags field is new in Vim 7.0.
#define b0_flags        b0_fname[B0_FNAME_SIZE_ORG - 2]

#include "memline_shim.c.generated.h"

extern void rs_long_to_char(long n, char *s);

int nvim_curbuf_get_ml_flags(void) { return curbuf->b_ml.ml_flags; }
memfile_T *nvim_buf_get_ml_mfp(buf_T *buf) { return buf->b_ml.ml_mfp; }
int nvim_buf_get_ml_flags(buf_T *buf) { return buf->b_ml.ml_flags; }
void nvim_buf_set_ml_flags(buf_T *buf, int flags) { buf->b_ml.ml_flags = flags; }
linenr_T nvim_buf_get_ml_line_lnum(buf_T *buf) { return buf->b_ml.ml_line_lnum; }
linenr_T nvim_buf_get_ml_line_count(buf_T *buf) { return buf->b_ml.ml_line_count; }
colnr_T nvim_buf_get_ml_line_len(buf_T *buf) { return buf->b_ml.ml_line_len; }
void nvim_buf_set_ml_line_len(buf_T *buf, colnr_T len) { buf->b_ml.ml_line_len = len; }
char *nvim_buf_get_ml_line_ptr(buf_T *buf) { return buf->b_ml.ml_line_ptr; }
linenr_T nvim_pos_get_lnum(const pos_T *pos) { return pos->lnum; }
colnr_T nvim_pos_get_col(const pos_T *pos) { return pos->col; }
colnr_T nvim_pos_get_coladd(const pos_T *pos) { return pos->coladd; }
colnr_T nvim_get_maxcol(void) { return MAXCOL; }
size_t nvim_get_maxpathl(void) { return MAXPATHL; }
int nvim_buf_has_ml_mfp(buf_T *buf) { return buf->b_ml.ml_mfp != NULL; }
int nvim_buf_get_ml_usedchunks(buf_T *buf) { return buf->b_ml.ml_usedchunks; }
size_t nvim_buf_get_ml_line_offset(buf_T *buf) { return buf->b_ml.ml_line_offset; }
void nvim_buf_set_ml_line_offset(buf_T *buf, size_t offset) { buf->b_ml.ml_line_offset = offset; }
linenr_T nvim_buf_get_ml_locked_high(buf_T *buf) { return buf->b_ml.ml_locked_high; }
linenr_T nvim_buf_get_ml_locked_low(buf_T *buf) { return buf->b_ml.ml_locked_low; }
int nvim_buf_get_ml_chunksize_numlines(buf_T *buf, int idx) { return buf->b_ml.ml_chunksize[idx].mlcs_numlines; }
int nvim_buf_get_ml_chunksize_totalsize(buf_T *buf, int idx) { return buf->b_ml.ml_chunksize[idx].mlcs_totalsize; }
int nvim_buf_get_ml_chunksize_is_null(buf_T *buf) { return buf->b_ml.ml_chunksize == NULL; }
int nvim_buf_get_ml_numchunks(buf_T *buf) { return buf->b_ml.ml_numchunks; }
void nvim_buf_set_ml_numchunks(buf_T *buf, int val) { buf->b_ml.ml_numchunks = val; }
void *nvim_buf_get_ml_chunksize_ptr(buf_T *buf) { return buf->b_ml.ml_chunksize; }
void nvim_buf_set_ml_chunksize_ptr(buf_T *buf, void *ptr) { buf->b_ml.ml_chunksize = ptr; }
size_t nvim_get_chunksize_t_size(void) { return sizeof(chunksize_T); }
void *nvim_bhdr_get_bh_data(bhdr_T *hp) { return hp->bh_data; }
char *nvim_buf_get_ml_mfp_fname(buf_T *buf) { return (buf->b_ml.ml_mfp != NULL) ? buf->b_ml.ml_mfp->mf_fname : NULL; }
char *nvim_get_p_dir(void) { return p_dir; }
void nvim_buf_set_ml_line_ptr(buf_T *buf, char *ptr) { buf->b_ml.ml_line_ptr = ptr; }
void nvim_buf_set_ml_line_lnum(buf_T *buf, linenr_T lnum) { buf->b_ml.ml_line_lnum = lnum; }
int nvim_buf_open_buffer_if_needed(buf_T *buf) { return buf->b_ml.ml_mfp == NULL ? open_buffer(false, NULL, 0) : OK; }

void nvim_siemsg_ml_get_invalid_lnum(int64_t lnum) { siemsg(_("E315: ml_get: Invalid lnum: %" PRId64), lnum); }

void nvim_siemsg_ml_get_cannot_find_line(int64_t lnum, buf_T *buf) { get_trans_bufname(buf); shorten_dir(NameBuff); siemsg(_("E316: ml_get: Cannot find line %" PRId64 "in buffer %d %s"), lnum, buf->b_fnum, NameBuff); }

void nvim_sb_append_str(void *sb, const char *s) { kv_printf(*(StringBuilder *)sb, "%s", s); }

#ifdef UNIX
int nvim_swapfile_get_uname(const char *fname, char *uname_buf, size_t uname_len) { FileInfo fi; return os_fileinfo(fname, &fi) ? (os_get_uname((uv_uid_t)fi.stat.st_uid, uname_buf, uname_len) == OK ? 1 : 0) : 0; }
#endif

int nvim_buf_get_ml_stack_top(buf_T *buf) { return buf->b_ml.ml_stack_top; }
infoptr_T *nvim_buf_get_ml_stack_ip(buf_T *buf, int idx) { return &(buf->b_ml.ml_stack[idx]); }
int64_t nvim_ip_get_bnum(const infoptr_T *ip) { return (int64_t)ip->ip_bnum; }
int nvim_ip_get_index(const infoptr_T *ip) { return ip->ip_index; }
void nvim_ip_add_high(infoptr_T *ip, int count) { ip->ip_high += count; }
void nvim_iemsg_pointer_block_id_wrong_two(void) { iemsg(_("E317: Pointer block id wrong 2")); }
void nvim_iemsg_e304_upd_block0(void) { iemsg(_("E304: ml_upd_block0(): Didn't get block 0??")); }
void *nvim_buf_get_ml_locked(buf_T *buf) { return buf->b_ml.ml_locked; }
void nvim_buf_set_ml_locked(buf_T *buf, void *hp) { buf->b_ml.ml_locked = hp; }
int nvim_buf_get_ml_locked_lineadd(buf_T *buf) { return buf->b_ml.ml_locked_lineadd; }
void nvim_buf_set_ml_locked_lineadd(buf_T *buf, int val) { buf->b_ml.ml_locked_lineadd = val; }
void nvim_buf_set_ml_locked_low(buf_T *buf, linenr_T val) { buf->b_ml.ml_locked_low = val; }
void nvim_buf_set_ml_locked_high(buf_T *buf, linenr_T val) { buf->b_ml.ml_locked_high = val; }
linenr_T nvim_ip_get_low(const infoptr_T *ip) { return ip->ip_low; }
linenr_T nvim_ip_get_high(const infoptr_T *ip) { return ip->ip_high; }
void nvim_ip_set_bnum(infoptr_T *ip, int64_t bnum) { ip->ip_bnum = (blocknr_T)bnum; }
void nvim_ip_set_low(infoptr_T *ip, linenr_T lnum) { ip->ip_low = lnum; }
void nvim_ip_set_high(infoptr_T *ip, linenr_T lnum) { ip->ip_high = lnum; }
void nvim_ip_set_index(infoptr_T *ip, int idx) { ip->ip_index = idx; }
void nvim_iemsg_pointer_block_id_wrong(void) { iemsg(_("E317: Pointer block id wrong")); }
void nvim_siemsg_line_number_out_of_range(int64_t lnum_past) { siemsg(_("E322: Line number out of range: %" PRId64 " past the end"), lnum_past); }
void nvim_siemsg_line_count_wrong_in_block(int64_t bnum) { siemsg(_("E323: Line count wrong in block %" PRId64), bnum); }
void nvim_buf_inc_flush_count(buf_T *buf) { buf->flush_count++; }
linenr_T nvim_buf_dec_ml_line_count(buf_T *buf) { return --buf->b_ml.ml_line_count; }
linenr_T nvim_buf_inc_ml_line_count(buf_T *buf) { return ++buf->b_ml.ml_line_count; }
linenr_T nvim_buf_get_b_prev_line_count(buf_T *buf) { return buf->b_prev_line_count; }
void nvim_buf_set_b_prev_line_count(buf_T *buf, linenr_T val) { buf->b_prev_line_count = val; }
void nvim_set_keep_msg_no_lines(void) { set_keep_msg(_(no_lines_msg), 0); }
void nvim_iemsg_pointer_block_id_wrong_four(void) { iemsg(_("E317: Pointer block id wrong 4")); }
void nvim_mf_free(memfile_T *mfp, bhdr_T *hp) { mf_free(mfp, hp); }
int64_t nvim_bhdr_get_bh_bnum(bhdr_T *hp) { return (int64_t)hp->bh_bnum; }
int nvim_bhdr_get_bh_page_count(bhdr_T *hp) { return (int)hp->bh_page_count; }
void nvim_iemsg_pointer_block_id_wrong_three(void) { iemsg(_("E317: Pointer block id wrong 3")); }
void nvim_iemsg_e318_updated_too_many(void) { iemsg(_("E318: Updated too many blocks?")); }
void nvim_buf_set_ml_chunksize_numlines(buf_T *buf, int idx, int val) { buf->b_ml.ml_chunksize[idx].mlcs_numlines = val; }
void nvim_buf_set_ml_chunksize_totalsize(buf_T *buf, int idx, int val) { buf->b_ml.ml_chunksize[idx].mlcs_totalsize = val; }
void nvim_buf_add_ml_chunksize_numlines(buf_T *buf, int idx, int val) { buf->b_ml.ml_chunksize[idx].mlcs_numlines += val; }
void nvim_buf_add_ml_chunksize_totalsize(buf_T *buf, int idx, int val) { buf->b_ml.ml_chunksize[idx].mlcs_totalsize += val; }
void nvim_buf_set_ml_usedchunks(buf_T *buf, int val) { buf->b_ml.ml_usedchunks = val; }
void nvim_buf_ml_chunksize_memmove(buf_T *buf, int dst_idx, int src_idx, int count) { memmove(buf->b_ml.ml_chunksize + dst_idx, buf->b_ml.ml_chunksize + src_idx, (size_t)count * sizeof(chunksize_T)); }
void nvim_siemsg_e320_cannot_find_line(int64_t lnum) { siemsg(_("E320: Cannot find line %" PRId64), lnum); }
void nvim_buf_set_b_mtime(buf_T *buf, int64_t val) { buf->b_mtime = val; }
void nvim_buf_set_b_mtime_ns(buf_T *buf, int64_t val) { buf->b_mtime_ns = val; }
void nvim_buf_set_b_mtime_read(buf_T *buf, int64_t val) { buf->b_mtime_read = val; }
void nvim_buf_set_b_mtime_read_ns(buf_T *buf, int64_t val) { buf->b_mtime_read_ns = val; }
void nvim_buf_set_b_orig_size(buf_T *buf, int64_t val) { buf->b_orig_size = (uint64_t)val; }
void nvim_buf_set_b_orig_mode(buf_T *buf, int val) { buf->b_orig_mode = val; }
void nvim_b0_set_fname0(ZeroBlock *b0p) { b0p->b0_fname[0] = NUL; }
char *nvim_b0_get_mtime(ZeroBlock *b0p) { return b0p->b0_mtime; }
char *nvim_b0_get_ino(ZeroBlock *b0p) { return b0p->b0_ino; }

void nvim_home_replace_b0_fname(const buf_T *buf, ZeroBlock *b0p, size_t maxlen) { home_replace(NULL, buf->b_ffname, b0p->b0_fname, maxlen, true); }

int nvim_os_get_username(char *buf, size_t len) { return os_get_username(buf, len); }

int nvim_set_b0_mtime_ino(buf_T *buf, ZeroBlock *b0p)
{
  FileInfo fi;
  if (os_fileinfo(buf->b_ffname, &fi)) {
    rs_long_to_char(fi.stat.st_mtim.tv_sec, b0p->b0_mtime);
    rs_long_to_char((long)os_fileinfo_inode(&fi), b0p->b0_ino);
    buf_store_file_info(buf, &fi);
    buf->b_mtime_read = buf->b_mtime;
    buf->b_mtime_read_ns = buf->b_mtime_ns;
    return 1;
  }
  return 0;
}

void nvim_pos_set_lnum(pos_T *pos, linenr_T lnum) { pos->lnum = lnum; }
void nvim_pos_set_col(pos_T *pos, colnr_T col) { pos->col = col; }
void nvim_pos_set_coladd(pos_T *pos, colnr_T coladd) { pos->coladd = coladd; }

linenr_T nvim_buf_get_line_count(void *buf) { return buf == NULL ? 0 : ((buf_T *)buf)->b_ml.ml_line_count; }

colnr_T nvim_buf_get_line_len(void *buf, linenr_T lnum) { if (buf == NULL) { return 0; } buf_T *b = (buf_T *)buf; return (lnum < 1 || lnum > b->b_ml.ml_line_count) ? 0 : ml_get_buf_len(b, lnum); }

int64_t nvim_b0_get_magic_long(const ZeroBlock *b0p) { return (int64_t)b0p->b0_magic_long; }
int32_t nvim_b0_get_magic_int(const ZeroBlock *b0p) { return (int32_t)b0p->b0_magic_int; }
int16_t nvim_b0_get_magic_short(const ZeroBlock *b0p) { return b0p->b0_magic_short; }
uint8_t nvim_b0_get_magic_char(const ZeroBlock *b0p) { return (uint8_t)b0p->b0_magic_char; }
uint8_t nvim_b0_get_id(const ZeroBlock *b0p, int idx) { return (uint8_t)b0p->b0_id[idx]; }
const char *nvim_b0_get_version_ptr(const ZeroBlock *b0p) { return b0p->b0_version; }
const char *nvim_b0_get_page_size_ptr(const ZeroBlock *b0p) { return b0p->b0_page_size; }
const char *nvim_b0_get_uname_ptr(const ZeroBlock *b0p) { return b0p->b0_uname; }
const char *nvim_b0_get_hname_ptr(const ZeroBlock *b0p) { return b0p->b0_hname; }
const char *nvim_b0_get_fname_ptr(const ZeroBlock *b0p) { return b0p->b0_fname; }
uint64_t nvim_get_file_inode(const char *fname) { FileInfo fi; return os_fileinfo(fname, &fi) ? os_fileinfo_inode(&fi) : 0; }
void *nvim_buf_get_ml_stack_void(buf_T *buf) { return buf->b_ml.ml_stack; }
void nvim_buf_clear_ml_after_close(buf_T *buf) { buf->b_ml.ml_mfp = NULL; buf->b_flags &= ~BF_RECOVERED; }
void nvim_buf_xfree_clear_ml_chunksize(buf_T *buf) { xfree(buf->b_ml.ml_chunksize); buf->b_ml.ml_chunksize = NULL; }
size_t nvim_buf_get_deleted_bytes(buf_T *buf) { return buf->deleted_bytes; }
void nvim_buf_set_deleted_bytes(buf_T *buf, size_t val) { buf->deleted_bytes = val; }
size_t nvim_buf_get_deleted_codepoints(buf_T *buf) { return buf->deleted_codepoints; }
void nvim_buf_set_deleted_codepoints(buf_T *buf, size_t val) { buf->deleted_codepoints = val; }
size_t nvim_buf_get_deleted_codeunits(buf_T *buf) { return buf->deleted_codeunits; }
void nvim_buf_set_deleted_codeunits(buf_T *buf, size_t val) { buf->deleted_codeunits = val; }
bhdr_T *nvim_mf_get_block0_hp(memfile_T *mfp) { return pmap_get(int64_t)(&mfp->mf_hash, 0); }
void nvim_bhdr_set_bh_flags_dirty(bhdr_T *hp) { hp->bh_flags |= BH_DIRTY; }
void nvim_buf_add_deleted_bytes(buf_T *buf, size_t n) { buf->deleted_bytes += n; }
void nvim_buf_add_deleted_bytes2(buf_T *buf, size_t n) { buf->deleted_bytes2 += n; }
bool nvim_buf_get_update_need_codepoints(buf_T *buf) { return buf->update_need_codepoints; }
void nvim_buf_add_deleted_codepoints(buf_T *buf, size_t n) { buf->deleted_codepoints += n; }
void nvim_buf_add_deleted_codeunits(buf_T *buf, size_t n) { buf->deleted_codeunits += n; }
int nvim_buf_get_ml_stack_size(buf_T *buf) { return buf->b_ml.ml_stack_size; }
void nvim_buf_set_ml_stack_size(buf_T *buf, int n) { buf->b_ml.ml_stack_size = n; }
void nvim_buf_set_ml_stack_top(buf_T *buf, int n) { buf->b_ml.ml_stack_top = n; }
int nvim_buf_inc_ml_stack_top(buf_T *buf) { return buf->b_ml.ml_stack_top++; }
void *nvim_buf_get_ml_stack(buf_T *buf) { return buf->b_ml.ml_stack; }
void nvim_buf_set_ml_stack(buf_T *buf, void *ptr) { buf->b_ml.ml_stack = ptr; }
size_t nvim_get_infoptr_size(void) { return sizeof(infoptr_T); }
uint8_t nvim_b0_get_flags_byte(const ZeroBlock *b0p) { return (uint8_t)b0p->b0_flags; }
void nvim_b0_set_flags_byte(ZeroBlock *b0p, uint8_t val) { b0p->b0_flags = (char)val; }
char *nvim_b0_get_fname_mut(ZeroBlock *b0p) { return b0p->b0_fname; }
const char *nvim_b0_get_pid_ptr(const ZeroBlock *b0p) { return b0p->b0_pid; }
uint8_t nvim_b0_get_dirty(const ZeroBlock *b0p) { return (uint8_t)b0p->b0_dirty; }
void nvim_b0_set_hname_end(ZeroBlock *b0p) { b0p->b0_hname[B0_HNAME_SIZE - 1] = NUL; }
size_t nvim_b0_get_struct_size(void) { return sizeof(ZeroBlock); }
int64_t nvim_get_file_mtime(const char *fname) { FileInfo fi; return os_fileinfo(fname, &fi) ? (int64_t)fi.stat.st_mtim.tv_sec : 0; }
int nvim_buf_get_b_spell(buf_T *buf) { return buf->b_spell ? 1 : 0; }
void nvim_buf_set_b_may_swap(buf_T *buf, int val) { buf->b_may_swap = (val != 0); }
void nvim_os_set_cloexec(int fd) { os_set_cloexec(fd); }
int nvim_vim_rename(const char *from, const char *to) { return vim_rename(from, to); }

void nvim_buf_set_ml_mfp(buf_T *buf, void *mfp) { buf->b_ml.ml_mfp = mfp; }

void nvim_b0_init_header(ZeroBlock *b0p, unsigned page_size)
{
  b0p->b0_id[0] = BLOCK0_ID0;
  b0p->b0_id[1] = BLOCK0_ID1;
  b0p->b0_magic_long = B0_MAGIC_LONG;
  b0p->b0_magic_int = B0_MAGIC_INT;
  b0p->b0_magic_short = (int16_t)B0_MAGIC_SHORT;
  b0p->b0_magic_char = B0_MAGIC_CHAR;
  xstrlcpy(xstpcpy(b0p->b0_version, "VIM "), Versions[0], 6);
  rs_long_to_char((long)page_size, b0p->b0_page_size);
}

void nvim_buf_set_b_p_swf_false(buf_T *buf) { buf->b_p_swf = false; }
void nvim_buf_set_b_may_swap_true(buf_T *buf) { buf->b_may_swap = true; }
void nvim_b0_set_dirty_from_buf(ZeroBlock *b0p, buf_T *buf) { b0p->b0_dirty = buf->b_changed ? B0_DIRTY : 0; }
void nvim_b0_set_flags_from_ff(ZeroBlock *b0p, int fileformat) { b0p->b0_flags = (char)(fileformat + 1); }
void nvim_b0_fill_uname(ZeroBlock *b0p) { os_get_username(b0p->b0_uname, B0_UNAME_SIZE); }
void nvim_b0_fill_hname(ZeroBlock *b0p) { os_get_hostname(b0p->b0_hname, B0_HNAME_SIZE); }
void nvim_b0_fill_pid(ZeroBlock *b0p) { rs_long_to_char((long)os_get_pid(), b0p->b0_pid); }
int nvim_get_swap_exists_action(void) { return swap_exists_action; }
void nvim_set_swap_exists_action(int val) { swap_exists_action = val; }
int nvim_get_recoverymode(void) { return recoverymode ? 1 : 0; }
const char *nvim_get_p_shm(void) { return p_shm; }
void nvim_inc_no_wait_return(void) { no_wait_return++; }
void nvim_dec_no_wait_return(void) { no_wait_return--; }
void nvim_buf_set_b_p_ro_true(buf_T *buf) { buf->b_p_ro = true; }
int nvim_os_fileinfo_link(const char *fname) { FileInfo fi; return os_fileinfo_link(fname, &fi) ? 1 : 0; }
int nvim_read_block0(int fd, ZeroBlock *b0p) { ssize_t n = read_eintr(fd, b0p, sizeof(*b0p)); return (n == (ssize_t)sizeof(*b0p)) ? 1 : 0; }
int nvim_same_directory(const char *a, const char *b) { return same_directory(a, b); }
void nvim_expand_env_maxpathl(const char *src, char *dst, int len) { expand_env((char *)src, dst, len); }
int nvim_os_isdir(const char *name) { return os_isdir(name) ? 1 : 0; }

int nvim_os_mkdir_recurse(const char *dir, int mode, char **failed_dir) { return os_mkdir_recurse(dir, mode, failed_dir, NULL); }

char *nvim_path_tail_const(const char *fname) { return path_tail(fname); }

extern bool rs_has_autocmd(int event, const char *sfname, int buf_fnum);
int nvim_has_autocmd_swapexists(const char *fname, buf_T *buf) { return rs_has_autocmd(EVENT_SWAPEXISTS, fname, buf ? buf->b_fnum : 0) ? 1 : 0; }

void nvim_apply_autocmds_swapexists(const char *fname, buf_T *buf) { allbuf_lock++; apply_autocmds(EVENT_SWAPEXISTS, (char *)fname, NULL, false, NULL); allbuf_lock--; }

const char *nvim_get_vim_var_swapchoice(void) { return get_vim_var_str(VV_SWAPCHOICE); }
void nvim_set_vim_var_swapname(const char *fname) { set_vim_var_string(VV_SWAPNAME, (char *)fname, -1); }
void nvim_clear_vim_var_swapname(void) { set_vim_var_string(VV_SWAPNAME, NULL, -1); }
void nvim_clear_vim_var_swapchoice(void) { set_vim_var_string(VV_SWAPCHOICE, NULL, -1); }

int nvim_do_dialog_warning(const char *title, const char *message, const char *buttons, int dflt_button, bool mouse_used) { return do_dialog(VIM_WARNING, (char *)title, (char *)message, (char *)buttons, dflt_button, NULL, mouse_used); }

void nvim_flush_buffers_typeahead(void) { flush_buffers(FLUSH_TYPEAHEAD); }
void nvim_msg_reset_scroll(void) { msg_reset_scroll(); }

void nvim_msg_multiline(const char *s, int hl_id) { bool need_clear = false; msg_multiline(cbuf_as_string((char *)s, strlen(s)), hl_id, false, false, &need_clear); }

void nvim_verb_msg(const char *s) { verb_msg((char *)s); }
int nvim_os_open_rdonly(const char *fname) { return os_open(fname, O_RDONLY, 0); }
void nvim_close_fd(int fd) { close(fd); }

void *nvim_alloc_stringbuilder_iosize(void) { StringBuilder *sb = xmalloc(sizeof(StringBuilder)); *sb = (StringBuilder)KV_INITIAL_VALUE; kv_resize(*sb, IOSIZE); return sb; }

const char *nvim_sb_get_items(void *sb) { return ((StringBuilder *)sb)->items; }
size_t nvim_sb_get_size(void *sb) { return ((StringBuilder *)sb)->size; }
void nvim_free_stringbuilder(void *sb) { kv_destroy(*(StringBuilder *)sb); xfree(sb); }
void nvim_sb_append(void *sb, const char *s) { kv_printf(*(StringBuilder *)sb, "%s", s); }
void nvim_msg_puts_newline(void) { msg_puts("\n"); }
const char *nvim_os_strerror(int err) { return os_strerror(err); }
int nvim_mf_sync(memfile_T *mfp, int flags) { return mf_sync(mfp, flags); }
int nvim_mf_need_trans(memfile_T *mfp) { return mf_need_trans(mfp); }
int nvim_mf_is_dirty(memfile_T *mfp) { return mfp->mf_dirty == MF_DIRTY_YES ? 1 : 0; }
int nvim_os_char_avail(void) { return os_char_avail() ? 1 : 0; }
void nvim_set_need_check_timestamps(int val) { need_check_timestamps = val != 0; }

int nvim_buf_file_unchanged(buf_T *buf)
{
  if (buf->b_ffname == NULL) {
    return 0;
  }
  FileInfo file_info;
  if (!os_fileinfo(buf->b_ffname, &file_info)
      || file_info.stat.st_mtim.tv_sec != buf->b_mtime_read
      || file_info.stat.st_mtim.tv_nsec != buf->b_mtime_read_ns
      || os_fileinfo_size(&file_info) != buf->b_orig_size) {
    return 1;
  }
  return 0;
}

void nvim_msg_file_preserved(void) { msg(_("File preserved"), 0); }
void nvim_emsg_preserve_failed(void) { emsg(_("E314: Preserve failed")); }
void nvim_emsg_no_swapfile(void) { emsg(_("E313: Cannot preserve, there is no swap file")); }
void nvim_set_recoverymode(int val) { recoverymode = (val != 0); }
int nvim_get_called_from_main(void) { return curbuf->b_ml.ml_mfp == NULL ? 1 : 0; }
memfile_T *nvim_mf_open_rdonly(char *fname) { return mf_open(fname, O_RDONLY); }
void nvim_mf_close_nodelete(memfile_T *mfp) { mf_close(mfp, false); }

bhdr_T *nvim_mf_get_block(memfile_T *mfp, int64_t bnum, unsigned page_count) { return mf_get(mfp, (blocknr_T)bnum, page_count); }

void nvim_mf_put_block(memfile_T *mfp, bhdr_T *hp, bool dirty, bool infile) { mf_put(mfp, hp, dirty, infile); }
void nvim_mf_new_page_size_wrapper(memfile_T *mfp, unsigned new_size) { mf_new_page_size(mfp, new_size); }
void nvim_bhdr_set_bh_data(bhdr_T *hp, void *data) { hp->bh_data = data; }
void nvim_curbuf_set_b_flags_recovered(void) { curbuf->b_flags |= BF_RECOVERED; }
void nvim_getout_one(void) { getout(1); }
int nvim_ml_open_curbuf(void) { return ml_open(curbuf); }
void nvim_ml_close_curbuf_true(void) { ml_close(curbuf, true); }
int nvim_setfname_for_recovery(const char *name) { return setfname(curbuf, (char *)name, NULL, true); }
const char *nvim_buf_spname_curbuf(void) { return buf_spname(curbuf); }
void nvim_home_replace_into_namebuff(const char *fname) { home_replace(NULL, (char *)fname, NameBuff, MAXPATHL, true); }
void nvim_home_replace_curbuf_ffname_into_namebuff(void) { home_replace(NULL, curbuf->b_ffname, NameBuff, MAXPATHL, true); }
void nvim_xstrlcpy_namebuff(const char *src) { xstrlcpy(NameBuff, src, MAXPATHL); }
void nvim_expand_env_into_namebuff(const char *src) { expand_env((char *)src, NameBuff, MAXPATHL); }
const char *nvim_get_namebuff_ptr(void) { return NameBuff; }
void nvim_smsg_using_swap_file(void) { smsg(0, _("Using swap file \"%s\""), NameBuff); }
void nvim_smsg_original_file(void) { smsg(0, _("Original file \"%s\""), NameBuff); }

int nvim_recover_check_timestamps(memfile_T *mfp, int mtime_b0)
{
  if (curbuf->b_ffname == NULL) {
    return 0;
  }
  FileInfo org_file_info;
  FileInfo swp_file_info;
  if (os_fileinfo(curbuf->b_ffname, &org_file_info)
      && ((os_fileinfo(mfp->mf_fname, &swp_file_info)
           && org_file_info.stat.st_mtim.tv_sec > swp_file_info.stat.st_mtim.tv_sec)
          || org_file_info.stat.st_mtim.tv_sec != mtime_b0)) {
    return 1;
  }
  return 0;
}

int nvim_readfile_for_recovery(const char *fname) { return readfile((char *)fname, NULL, 0, 0, MAXLNUM, NULL, READ_NEW, false); }
int nvim_readfile_from_original(const char *fname, linenr_T lnum, linenr_T topline, linenr_T line_count) { return readfile((char *)fname, NULL, lnum, topline, line_count, NULL, 0, false); }

void nvim_set_fileformat_local(int ff) { set_fileformat(ff, OPT_LOCAL); }

void nvim_set_fenc_local(const char *fenc) { set_option_value_give_err(kOptFileencoding, CSTR_AS_OPTVAL((char *)fenc), OPT_LOCAL); }

void nvim_unchanged_curbuf(void) { unchanged(curbuf, true, true); }
void nvim_changed_internal_curbuf(void) { changed_internal(curbuf); }
int nvim_curbuf_get_b_changed(void) { return curbuf->b_changed ? 1 : 0; }
void nvim_ml_delete_last_curbuf(void) { ml_delete(curbuf->b_ml.ml_line_count); }
linenr_T nvim_get_curbuf_ml_line_count(void) { return curbuf->b_ml.ml_line_count; }
int nvim_get_curbuf_ml_flags(void) { return curbuf->b_ml.ml_flags; }
int nvim_buf_dec_ml_stack_top(buf_T *buf) { return --(buf->b_ml.ml_stack_top); }
void nvim_buf_reset_ml_stack(buf_T *buf) { buf->b_ml.ml_stack_top = 0; buf->b_ml.ml_stack = NULL; buf->b_ml.ml_stack_size = 0; }
void nvim_apply_autocmds_bufreadpost(void) { apply_autocmds(EVENT_BUFREADPOST, NULL, curbuf->b_fname, false, curbuf); }
void nvim_apply_autocmds_bufwinenter(void) { apply_autocmds(EVENT_BUFWINENTER, NULL, curbuf->b_fname, false, curbuf); }
void nvim_set_cmdline_row_to_msg_row(void) { cmdline_row = msg_row; }

int nvim_prompt_for_recovery(void) { return prompt_for_input(_("Enter number of swap file to use (0 to quit): "), 0, false, NULL); }

size_t nvim_get_buf_t_size(void) { return sizeof(buf_T); }
void nvim_ml_delete_first_curbuf(void) { ml_delete(1); }
int64_t nvim_mf_get_file_size(memfile_T *mfp) { off_T size = vim_lseek(mfp->mf_fd, 0, SEEK_END); return (int64_t)(size <= 0 ? 0 : size); }
int nvim_ml_append_recovery(linenr_T lnum, const char *line, bool is_new) { return ml_append(lnum, (char *)line, 0, is_new); }
int nvim_curbuf_nf_has(int c) { return vim_strchr(curbuf->b_p_nf, c) != NULL; }
void nvim_curbuf_set_op_start_to_cursor_col(int col) { curbuf->b_op_start = curwin->w_cursor; curbuf->b_op_start.col = col; }
void nvim_curbuf_set_op_end_to_cursor_col(int col) { curbuf->b_op_end = curwin->w_cursor; curbuf->b_op_end.col = col; if (curbuf->b_op_end.col > 0) { curbuf->b_op_end.col--; } }
void nvim_curwin_set_cursor_from_pos(const pos_T *pos) { curwin->w_cursor = *pos; }
void nvim_curwin_set_cursor_coladd(int v) { curwin->w_cursor.coladd = (colnr_T)v; }
void nvim_os_breakcheck(void) { os_breakcheck(); }
int nvim_bomb_size(void) { return bomb_size(); }
void nvim_msg_no_lines(void) { msg(_(no_lines_msg), 0); }
bool nvim_curbuf_get_b_p_eol(void) { return curbuf->b_p_eol; }
bool nvim_curbuf_get_b_p_fixeol(void) { return curbuf->b_p_fixeol; }
void nvim_ml_setname(buf_T *buf) { ml_setname(buf); }
void nvim_ml_timestamp(buf_T *buf) { ml_timestamp(buf); }
int nvim_buf_get_no_eol_lnum(buf_T *buf) { return (int)buf->b_no_eol_lnum; }
void nvim_sb_push_byte(void *sb, char byte) { kv_push(*(StringBuilder *)sb, byte); }
void nvim_sb_concat_len(void *sb, const char *ptr, size_t len) { kv_concat_len(*(StringBuilder *)sb, ptr, len); }

// Phase 2: set_file_options / set_rw_fname accessors
const char *nvim_get_p_ffs(void) { return p_ffs; }
int nvim_get_fileformat_force(buf_T *buf, exarg_T *eap) { return get_fileformat_force(buf, eap); }
void nvim_set_options_bin(int oldval, int newval, int opt) { set_options_bin(oldval, newval, opt); }
void nvim_buf_set_b_p_bin(buf_T *buf, int val) { buf->b_p_bin = (bool)val; }
int nvim_exarg_get_force_bin(const exarg_T *eap) { return eap->force_bin; }
int nvim_exarg_get_force_ff(const exarg_T *eap) { return eap->force_ff; }
