/// An abstraction to handle blocks of memory which can be stored in a file.
/// This is the implementation of a sort of virtual memory.
///
/// A memfile consists of a sequence of blocks:
/// - Blocks numbered from 0 upwards have been assigned a place in the actual
///   file. The block number is equal to the page number in the file.
/// - Blocks with negative numbers are currently in memory only. They can be
///   assigned a place in the file when too much memory is being used. At that
///   moment, they get a new, positive, number. A list is used for translation
///   of negative to positive numbers.
///
/// The size of a block is a multiple of a page size, normally the page size of
/// the device the file is on. Most blocks are 1 page long. A block of multiple
/// pages is used for a line that does not fit in a single page.
///
/// Each block can be in memory and/or in a file. The block stays in memory
/// as long as it is locked. If it is no longer locked it can be swapped out to
/// the file. It is only written to the file if it has been changed.
///
/// Under normal operation the file is created when opening the memory file and
/// deleted when closing the memory file. Only with recovery an existing memory
/// file is opened.
///
/// The functions for using a memfile:
///
/// mf_open()         open a new or existing memfile
/// mf_open_file()    open a swap file for an existing memfile
/// mf_close()        close (and delete) a memfile
/// mf_new()          create a new block in a memfile and lock it
/// mf_get()          get an existing block and lock it
/// mf_put()          unlock a block, may be marked for writing
/// mf_free()         remove a block
/// mf_sync()         sync changed parts of memfile to disk
/// mf_release_all()  release as much memory as possible
/// mf_trans_del()    may translate negative to positive block number
/// mf_fullname()     make file name full path (use before first :cd)

#include <assert.h>
#include <fcntl.h>
#include <inttypes.h>
#include <limits.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>
#include <sys/stat.h>

#include "nvim/assert_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/errors.h"
#include "nvim/fileio.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/map_defs.h"
#include "nvim/memfile.h"
#include "nvim/memfile_defs.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/os/fs.h"
#include "nvim/os/fs_defs.h"
#include "nvim/os/input.h"
#include "nvim/os/os_defs.h"
#include "nvim/path.h"
#include "nvim/pos_defs.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"

#include "memfile.c.generated.h"

// =============================================================================
// _Static_assert for all constants crossing the FFI boundary
// =============================================================================

_Static_assert(BH_DIRTY == 1U, "BH_DIRTY must be 1");
_Static_assert(BH_LOCKED == 2U, "BH_LOCKED must be 2");
_Static_assert(MF_DIRTY_NO == 0, "MF_DIRTY_NO must be 0");
_Static_assert(MF_DIRTY_YES == 1, "MF_DIRTY_YES must be 1");
_Static_assert(MF_DIRTY_YES_NOSYNC == 2, "MF_DIRTY_YES_NOSYNC must be 2");
_Static_assert(MFS_ALL == 1, "MFS_ALL must be 1");
_Static_assert(MFS_STOP == 2, "MFS_STOP must be 2");
_Static_assert(MFS_FLUSH == 4, "MFS_FLUSH must be 4");
_Static_assert(MFS_ZERO == 8, "MFS_ZERO must be 8");
_Static_assert(MIN_SWAP_PAGE_SIZE == 1048, "MIN_SWAP_PAGE_SIZE must be 1048");
_Static_assert(MAX_SWAP_PAGE_SIZE == 50000, "MAX_SWAP_PAGE_SIZE must be 50000");
_Static_assert(OK == 1, "OK must be 1");
_Static_assert(FAIL == 0, "FAIL must be 0");

// =============================================================================
// Rust function declarations
// =============================================================================

extern memfile_T *rs_mf_open(char *fname, int flags);
extern int rs_mf_open_file(memfile_T *mfp, char *fname);
extern void rs_mf_close(memfile_T *mfp, bool del_file);
extern void rs_mf_close_file_impl(memfile_T *mfp);
extern void rs_mf_new_page_size(memfile_T *mfp, unsigned new_size);
extern bhdr_T *rs_mf_new(memfile_T *mfp, bool negative, unsigned page_count);
extern bhdr_T *rs_mf_get(memfile_T *mfp, blocknr_T nr, unsigned page_count);
extern void rs_mf_put(memfile_T *mfp, bhdr_T *hp, bool dirty, bool infile);
extern void rs_mf_free(memfile_T *mfp, bhdr_T *hp);
extern int rs_mf_sync(memfile_T *mfp, int flags);
extern void rs_mf_set_dirty_all(memfile_T *mfp);
extern bool rs_mf_release_for_memfile(memfile_T *mfp);
extern blocknr_T rs_mf_trans_del(memfile_T *mfp, blocknr_T old_nr);
extern void rs_mf_free_fnames(memfile_T *mfp);
extern void rs_mf_set_fnames(memfile_T *mfp, char *fname);
extern void rs_mf_fullname(memfile_T *mfp);
extern bool rs_mf_need_trans(memfile_T *mfp);

// =============================================================================
// C accessor functions for memfile_T fields
// =============================================================================

char *nvim_mf_get_fname(memfile_T *mfp)
{
  return mfp->mf_fname;
}

void nvim_mf_set_fname(memfile_T *mfp, char *fname)
{
  mfp->mf_fname = fname;
}

char *nvim_mf_get_ffname(memfile_T *mfp)
{
  return mfp->mf_ffname;
}

void nvim_mf_set_ffname(memfile_T *mfp, char *ffname)
{
  mfp->mf_ffname = ffname;
}

int nvim_mf_get_fd(memfile_T *mfp)
{
  return mfp->mf_fd;
}

void nvim_mf_set_fd(memfile_T *mfp, int fd)
{
  mfp->mf_fd = fd;
}

int nvim_mf_get_flags(memfile_T *mfp)
{
  return mfp->mf_flags;
}

void nvim_mf_set_flags(memfile_T *mfp, int flags)
{
  mfp->mf_flags = flags;
}

bool nvim_mf_get_reopen(memfile_T *mfp)
{
  return mfp->mf_reopen;
}

void nvim_mf_set_reopen(memfile_T *mfp, bool reopen)
{
  mfp->mf_reopen = reopen;
}

bhdr_T *nvim_mf_get_free_first(memfile_T *mfp)
{
  return mfp->mf_free_first;
}

void nvim_mf_set_free_first(memfile_T *mfp, bhdr_T *hp)
{
  mfp->mf_free_first = hp;
}

blocknr_T nvim_mf_get_blocknr_max(memfile_T *mfp)
{
  return mfp->mf_blocknr_max;
}

void nvim_mf_set_blocknr_max(memfile_T *mfp, blocknr_T val)
{
  mfp->mf_blocknr_max = val;
}

blocknr_T nvim_mf_get_blocknr_min(memfile_T *mfp)
{
  return mfp->mf_blocknr_min;
}

void nvim_mf_set_blocknr_min(memfile_T *mfp, blocknr_T val)
{
  mfp->mf_blocknr_min = val;
}

blocknr_T nvim_mf_get_neg_count(memfile_T *mfp)
{
  return mfp->mf_neg_count;
}

void nvim_mf_set_neg_count(memfile_T *mfp, blocknr_T val)
{
  mfp->mf_neg_count = val;
}

blocknr_T nvim_mf_get_infile_count(memfile_T *mfp)
{
  return mfp->mf_infile_count;
}

void nvim_mf_set_infile_count(memfile_T *mfp, blocknr_T val)
{
  mfp->mf_infile_count = val;
}

unsigned nvim_mf_get_page_size(memfile_T *mfp)
{
  return mfp->mf_page_size;
}

void nvim_mf_set_page_size(memfile_T *mfp, unsigned val)
{
  mfp->mf_page_size = val;
}

int nvim_mf_get_dirty(memfile_T *mfp)
{
  return (int)mfp->mf_dirty;
}

void nvim_mf_set_dirty(memfile_T *mfp, int val)
{
  mfp->mf_dirty = (mfdirty_T)val;
}

// =============================================================================
// C accessor functions for bhdr_T fields
// =============================================================================

blocknr_T nvim_bh_get_bnum(bhdr_T *hp)
{
  return hp->bh_bnum;
}

void nvim_bh_set_bnum(bhdr_T *hp, blocknr_T bnum)
{
  hp->bh_bnum = bnum;
}

void *nvim_bh_get_data(bhdr_T *hp)
{
  return hp->bh_data;
}

void nvim_bh_set_data(bhdr_T *hp, void *data)
{
  hp->bh_data = data;
}

unsigned nvim_bh_get_page_count(bhdr_T *hp)
{
  return hp->bh_page_count;
}

void nvim_bh_set_page_count(bhdr_T *hp, unsigned count)
{
  hp->bh_page_count = count;
}

unsigned nvim_bh_get_flags(bhdr_T *hp)
{
  return hp->bh_flags;
}

void nvim_bh_set_flags(bhdr_T *hp, unsigned flags)
{
  hp->bh_flags = flags;
}

// =============================================================================
// Allocation wrappers
// =============================================================================

memfile_T *nvim_mf_alloc(void)
{
  return xmalloc(sizeof(memfile_T));
}

void nvim_mf_dealloc(memfile_T *mfp)
{
  xfree(mfp);
}

bhdr_T *nvim_bh_alloc(void)
{
  return xmalloc(sizeof(bhdr_T));
}

void nvim_bh_dealloc(bhdr_T *hp)
{
  xfree(hp);
}

// =============================================================================
// Map/PMap wrappers
// =============================================================================

void nvim_mf_hash_init(memfile_T *mfp)
{
  mfp->mf_hash = (PMap(int64_t))MAP_INIT;
}

void nvim_mf_hash_destroy(memfile_T *mfp)
{
  map_destroy(int64_t, &mfp->mf_hash);
}

bhdr_T *nvim_mf_hash_get(memfile_T *mfp, blocknr_T key)
{
  return pmap_get(int64_t)(&mfp->mf_hash, key);
}

void nvim_mf_hash_put(memfile_T *mfp, blocknr_T key, bhdr_T *hp)
{
  pmap_put(int64_t)(&mfp->mf_hash, key, hp);
}

void nvim_mf_hash_del(memfile_T *mfp, blocknr_T key)
{
  pmap_del(int64_t)(&mfp->mf_hash, key, NULL);
}

int nvim_mf_hash_size(memfile_T *mfp)
{
  return (int)map_size(&mfp->mf_hash);
}

bhdr_T *nvim_mf_hash_value_at(memfile_T *mfp, int index)
{
  return mfp->mf_hash.values[index];
}

void nvim_mf_trans_init(memfile_T *mfp)
{
  mfp->mf_trans = (Map(int64_t, int64_t))MAP_INIT;
}

void nvim_mf_trans_destroy(memfile_T *mfp)
{
  map_destroy(int64_t, &mfp->mf_trans);
}

void nvim_mf_trans_put(memfile_T *mfp, blocknr_T old_bnum, blocknr_T new_bnum)
{
  map_put(int64_t, int64_t)(&mfp->mf_trans, old_bnum, new_bnum);
}

blocknr_T *nvim_mf_trans_ref(memfile_T *mfp, blocknr_T old_bnum)
{
  return map_ref(int64_t, int64_t)(&mfp->mf_trans, old_bnum, NULL);
}

void nvim_mf_trans_del(memfile_T *mfp, blocknr_T old_bnum)
{
  map_del(int64_t, int64_t)(&mfp->mf_trans, old_bnum, NULL);
}

// =============================================================================
// Global variable accessors
// =============================================================================

int nvim_mf_get_got_int(void)
{
  return got_int;
}

void nvim_mf_set_got_int(int val)
{
  got_int = val;
}

int nvim_mf_get_did_swapwrite_msg(void)
{
  return did_swapwrite_msg;
}

void nvim_mf_set_did_swapwrite_msg(int val)
{
  did_swapwrite_msg = val;
}

// =============================================================================
// Message wrappers (apply _() for localization)
// =============================================================================

void nvim_mf_emsg(const char *msg)
{
  emsg(_(msg));
}

void nvim_mf_iemsg(const char *msg)
{
  iemsg(_(msg));
}

void nvim_mf_perror(const char *msg)
{
  PERROR(_(msg));
}

// =============================================================================
// FileInfo wrappers
// =============================================================================

bool nvim_mf_fileinfo_fd(int fd, uint64_t *blocksize_out)
{
  FileInfo file_info;
  if (os_fileinfo_fd(fd, &file_info)) {
    *blocksize_out = os_fileinfo_blocksize(&file_info);
    return true;
  }
  return false;
}

bool nvim_mf_fileinfo_link_exists(const char *fname)
{
  FileInfo file_info;
  return os_fileinfo_link(fname, &file_info);
}

// =============================================================================
// File I/O wrappers
// =============================================================================

int nvim_mf_os_open(const char *fname, int flags, int mode)
{
  return os_open(fname, flags, mode);
}

void nvim_mf_os_remove(const char *fname)
{
  os_remove(fname);
}

void nvim_mf_os_set_cloexec(int fd)
{
  os_set_cloexec(fd);
}

int nvim_mf_os_fsync(int fd)
{
  return os_fsync(fd);
}

bool nvim_mf_os_char_avail(void)
{
  return os_char_avail();
}

void nvim_mf_os_breakcheck(void)
{
  os_breakcheck();
}

int64_t nvim_mf_vim_lseek(int fd, int64_t offset, int whence)
{
  return (int64_t)vim_lseek(fd, (off_T)offset, whence);
}

int nvim_mf_read_eintr(int fd, void *buf, unsigned size)
{
  return read_eintr(fd, buf, size);
}

int nvim_mf_write_eintr(int fd, const void *buf, unsigned size)
{
  return write_eintr(fd, (void *)buf, size);
}

int nvim_mf_close_fd(int fd)
{
  return close(fd);
}

// =============================================================================
// String/path wrappers
// =============================================================================

char *nvim_mf_fullname_save(const char *fname)
{
  return FullName_save(fname, false);
}

void nvim_mf_xfree_clear_fname(memfile_T *mfp)
{
  XFREE_CLEAR(mfp->mf_fname);
}

void nvim_mf_xfree_clear_ffname(memfile_T *mfp)
{
  XFREE_CLEAR(mfp->mf_ffname);
}

// =============================================================================
// Cross-module (buf_T) wrappers
// =============================================================================

buf_T *nvim_mf_firstbuf(void)
{
  return firstbuf;
}

buf_T *nvim_mf_buf_next(buf_T *buf)
{
  return buf->b_next;
}

memfile_T *nvim_mf_buf_get_ml_mfp(buf_T *buf)
{
  return buf->b_ml.ml_mfp;
}

bool nvim_mf_buf_may_swap(buf_T *buf)
{
  return buf->b_may_swap;
}

void nvim_mf_buf_ml_open_file(buf_T *buf)
{
  ml_open_file(buf);
}

// =============================================================================
// Thin C wrappers delegating to Rust
// =============================================================================

memfile_T *mf_open(char *fname, int flags)
{
  return rs_mf_open(fname, flags);
}

int mf_open_file(memfile_T *mfp, char *fname)
{
  return rs_mf_open_file(mfp, fname);
}

void mf_close(memfile_T *mfp, bool del_file)
{
  rs_mf_close(mfp, del_file);
}

/// Close the swap file for a memfile. Used when 'swapfile' is reset.
///
/// @param getlines  Whether to get all lines into memory.
void mf_close_file(buf_T *buf, bool getlines)
{
  memfile_T *mfp = buf->b_ml.ml_mfp;
  if (mfp == NULL || mfp->mf_fd < 0) {   // nothing to close
    return;
  }

  if (getlines) {
    // get all blocks in memory by accessing all lines (clumsy!)
    for (linenr_T lnum = 1; lnum <= buf->b_ml.ml_line_count; lnum++) {
      ml_get_buf(buf, lnum);
    }
  }

  rs_mf_close_file_impl(mfp);
}

void mf_new_page_size(memfile_T *mfp, unsigned new_size)
{
  rs_mf_new_page_size(mfp, new_size);
}

bhdr_T *mf_new(memfile_T *mfp, bool negative, unsigned page_count)
{
  return rs_mf_new(mfp, negative, page_count);
}

bhdr_T *mf_get(memfile_T *mfp, blocknr_T nr, unsigned page_count)
{
  return rs_mf_get(mfp, nr, page_count);
}

void mf_put(memfile_T *mfp, bhdr_T *hp, bool dirty, bool infile)
{
  rs_mf_put(mfp, hp, dirty, infile);
}

void mf_free(memfile_T *mfp, bhdr_T *hp)
{
  rs_mf_free(mfp, hp);
}

int mf_sync(memfile_T *mfp, int flags)
{
  return rs_mf_sync(mfp, flags);
}

void mf_set_dirty(memfile_T *mfp)
{
  rs_mf_set_dirty_all(mfp);
}

/// Release as many blocks as possible.
///
/// Used in case of out of memory
///
/// @return  Whether any memory was released.
bool mf_release_all(void)
{
  bool retval = false;
  FOR_ALL_BUFFERS(buf) {
    memfile_T *mfp = buf->b_ml.ml_mfp;
    if (mfp != NULL) {
      // If no swap file yet, try to open one.
      if (mfp->mf_fd < 0 && buf->b_may_swap) {
        ml_open_file(buf);
      }

      // Flush as many blocks as possible, only if there is a swapfile.
      if (mfp->mf_fd >= 0) {
        if (rs_mf_release_for_memfile(mfp)) {
          retval = true;
        }
      }
    }
  }
  return retval;
}

blocknr_T mf_trans_del(memfile_T *mfp, blocknr_T old_nr)
{
  return rs_mf_trans_del(mfp, old_nr);
}

void mf_free_fnames(memfile_T *mfp)
{
  rs_mf_free_fnames(mfp);
}

void mf_set_fnames(memfile_T *mfp, char *fname)
{
  rs_mf_set_fnames(mfp, fname);
}

void mf_fullname(memfile_T *mfp)
{
  rs_mf_fullname(mfp);
}

bool mf_need_trans(memfile_T *mfp)
{
  return rs_mf_need_trans(mfp);
}
