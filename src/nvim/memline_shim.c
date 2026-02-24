// for debugging
// #define CHECK(c, s) do { if (c) emsg(s); } while (0)
#define CHECK(c, s) do {} while (0)

// memline.c: Contains the functions for appending, deleting and changing the
// text lines. The memfile functions are used to store the information in
// blocks of memory, backed up by a file. The structure of the information is
// a tree.  The root of the tree is a pointer block. The leaves of the tree
// are data blocks. In between may be several layers of pointer blocks,
// forming branches.
//
// Three types of blocks are used:
// - Block nr 0 contains information for recovery
// - Pointer blocks contain list of pointers to other blocks.
// - Data blocks contain the actual text.
//
// Block nr 0 contains the block0 structure (see below).
//
// Block nr 1 is the first pointer block. It is the root of the tree.
// Other pointer blocks are branches.
//
//  If a line is too big to fit in a single page, the block containing that
//  line is made big enough to hold the line. It may span several pages.
//  Otherwise all blocks are one page.
//
//  A data block that was filled when starting to edit a file and was not
//  changed since then, can have a negative block number. This means that it
//  has not yet been assigned a place in the file. When recovering, the lines
//  in this data block can be read from the original file. When the block is
//  changed (lines appended/deleted/changed) or when it is flushed it gets a
//  positive number. Use mf_trans_del() to get the new number, before calling
//  mf_get().
//
// "Mom, can we get ropes?"
// "We have ropes at home."
// Ropes at home:

#include <assert.h>
#include <errno.h>
#include <fcntl.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
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
#include "nvim/cursor.h"
#include "nvim/drawscreen.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/fileio.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/highlight_defs.h"
#include "nvim/input.h"
#include "nvim/macros_defs.h"
#include "nvim/main.h"
#include "nvim/map_defs.h"
#include "nvim/mark.h"
#include "nvim/mbyte.h"
#include "nvim/memfile.h"
#include "nvim/memfile_defs.h"
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
#include "nvim/os/proc.h"
#include "nvim/os/time.h"
#include "nvim/os/time_defs.h"
#include "nvim/path.h"
#include "nvim/pos_defs.h"
#include "nvim/spell.h"
#include "nvim/statusline.h"
#include "nvim/strings.h"
#include "nvim/ui.h"
#include "nvim/undo.h"
#include "nvim/version.h"
#include "nvim/vim_defs.h"

#ifndef UNIX            // it's in os/unix_defs.h for Unix
# include <time.h>
#endif

enum {
  DATA_ID = (('d' << 8) + 'a'),  // data block id
  PTR_ID = (('p' << 8) + 't'),   // pointer block id
  BLOCK0_ID0 = 'b',              // block 0 id 0
  BLOCK0_ID1 = '0',              // block 0 id 1
};

// pointer to a block, used in a pointer block
typedef struct {
  blocknr_T pe_bnum;            // block number
  linenr_T pe_line_count;       // number of lines in this branch
  linenr_T pe_old_lnum;         // lnum for this block (for recovery)
  int pe_page_count;            // number of pages in block pe_bnum
} PointerEntry;

// A pointer block contains a list of branches in the tree.
typedef struct {
  uint16_t pb_id;               // ID for pointer block: PTR_ID
  uint16_t pb_count;            // number of pointers in this block
  uint16_t pb_count_max;        // maximum value for pb_count
  PointerEntry pb_pointer[];    // list of pointers to blocks
                                // followed by empty space until end of page
} PointerBlock;

// Value for pb_count_max.
#define PB_COUNT_MAX(mfp) \
  (uint16_t)((mfp->mf_page_size - offsetof(PointerBlock, pb_pointer)) / sizeof(PointerEntry))

// A data block is a leaf in the tree.
//
// The text of the lines is at the end of the block. The text of the first line
// in the block is put at the end, the text of the second line in front of it,
// etc. Thus the order of the lines is the opposite of the line number.
typedef struct {
  uint16_t db_id;               // ID for data block: DATA_ID
  unsigned db_free;             // free space available
  unsigned db_txt_start;        // byte where text starts
  unsigned db_txt_end;          // byte just after data block
  // linenr_T db_line_count;
  long db_line_count;           // number of lines in this block
  unsigned db_index[];          // index for start of line
                                // followed by empty space up to db_txt_start
                                // followed by the text in the lines until
                                // end of page
} DataBlock;

// The low bits of db_index hold the actual index. The topmost bit is
// used for the global command to be able to mark a line.
// This method is not clean, but otherwise there would be at least one extra
// byte used for each line.
// The mark has to be in this place to keep it with the correct line when other
// lines are inserted or deleted.
#define DB_MARKED       ((unsigned)1 << ((sizeof(unsigned) * 8) - 1))
#define DB_INDEX_MASK   (~DB_MARKED)

#define INDEX_SIZE  (sizeof(unsigned))      // size of one db_index entry
#define HEADER_SIZE (offsetof(DataBlock, db_index))  // size of data block header

// ZeroBlock is now defined in memline_defs.h
// Additional block 0 constants kept here for internal use
enum {
  B0_FNAME_SIZE_NOCRYPT = 898,  // 2 bytes used for other things
  B0_FNAME_SIZE_CRYPT = 890,    // 10 bytes used for other things
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

// The lowest two bits contain the fileformat.  Zero means it's not set
// (compatible with Vim 6.x), otherwise it's EOL_UNIX + 1, EOL_DOS + 1 or
// EOL_MAC + 1.
#define B0_FF_MASK      3

// Swapfile is in directory of edited file.  Used to find the file from different mount points.
#define B0_SAME_DIR     4

// The 'fileencoding' is at the end of b0_fname[], with a NUL in front of it.
// When empty there is only the NUL.
#define B0_HAS_FENC     8

#define STACK_INCR      5       // nr of entries added to ml_stack at a time

// lowest_marked is now owned by Rust (LOWEST_MARKED in modify.rs).
// Use rs_ml_get_lowest_marked() / rs_ml_set_lowest_marked() to access it.

// arguments for ml_find_line()
enum {
  ML_DELETE = 0x11,  // delete line
  ML_INSERT = 0x12,  // insert line
  ML_FIND = 0x13,    // just find the line
  ML_FLUSH = 0x02,   // flush locked block
};
#define ML_SIMPLE(x)    ((x) & 0x10)  // DEL, INS or FIND

// argument for ml_upd_block0()
typedef enum {
  UB_FNAME = 0,         // update timestamp and filename
  UB_SAME_DIR,  // update the B0_SAME_DIR flag
} upd_block0_T;

typedef enum {
  SEA_CHOICE_NONE = 0,
  SEA_CHOICE_READONLY = 1,
  SEA_CHOICE_EDIT = 2,
  SEA_CHOICE_RECOVER = 3,
  SEA_CHOICE_DELETE = 4,
  SEA_CHOICE_QUIT = 5,
  SEA_CHOICE_ABORT = 6,
} sea_choice_T;

#include "memline_shim.c.generated.h"

extern int rs_get_fileformat(buf_T *buf);
// Phase 1 Rust function declarations
extern void rs_long_to_char(long n, char *s);
extern long rs_char_to_long(const char *s);
extern int rs_b0_magic_wrong(const ZeroBlock *b0p);
extern int rs_ml_check_b0_id(const ZeroBlock *b0p);
extern int rs_ml_check_b0_strings(const ZeroBlock *b0p);
extern int rs_fnamecmp_ino(const char *fname_c, const char *fname_s, long ino_block0);
// Phase 2 Rust function declarations
extern int rs_swapfile_proc_running(const ZeroBlock *b0p, const char *swap_fname);
extern void rs_set_b0_dir_flag(ZeroBlock *b0p, buf_T *buf);
extern void rs_add_b0_fenc(ZeroBlock *b0p, buf_T *buf);
extern bool rs_swapfile_unchanged(char *fname);
// Phase 4 Rust function declarations
extern void rs_ml_upd_block0(buf_T *buf, int what);
extern void rs_set_b0_fname(ZeroBlock *b0p, buf_T *buf);
extern void *rs_ml_new_data(memfile_T *mfp, bool negative, int page_count);
extern void *rs_ml_new_ptr(memfile_T *mfp);
extern void rs_ml_lineadd(buf_T *buf, int count);
// Pass 2 Phase 1: Mark tracking Rust function declarations
extern void rs_ml_setmarked(linenr_T lnum);
extern linenr_T rs_ml_firstmarked(void);
extern void rs_ml_clearmarked(void);
extern linenr_T rs_ml_get_lowest_marked(void);
extern void rs_ml_set_lowest_marked(linenr_T lnum);
extern void rs_ml_adjust_lowest_marked_for_insert(linenr_T lnum);
extern void rs_ml_adjust_lowest_marked_for_delete(linenr_T lnum);
// Pass 2 Phase 2: Swap file path helper Rust function declarations
extern char *rs_make_percent_swname(char *dir, char *dir_end, const char *name);
#ifdef HAVE_READLINK
extern int rs_resolve_symlink(const char *fname, char *buf);
#endif
extern char *rs_get_file_in_dir(char *fname, char *dname);
extern char *rs_makeswapname(char *fname, char *ffname, buf_T *buf, char *dir_name);
// Pass 2 Phase 3: Buffer lifecycle Rust function declarations
extern void rs_ml_close(buf_T *buf, int del_file);
extern void rs_check_need_swap(int newfile);
extern void rs_ml_timestamp(buf_T *buf);
extern size_t rs_ml_flush_deleted_bytes(buf_T *buf, size_t *codepoints, size_t *codeunits);
// Pass 2 Phase 4: Deleted-length tracking and stack Rust function declarations
extern void rs_ml_add_deleted_len(char *ptr, ssize_t len);
extern void rs_ml_add_deleted_len_buf(buf_T *buf, char *ptr, ssize_t len);
extern int rs_ml_add_stack(buf_T *buf);
// Pass 2 Phase 5: ml_setflags Rust function declaration
extern void rs_ml_setflags(buf_T *buf);
// Pass 3 Phase 1: swapfile_dict Rust function declaration
extern void rs_swapfile_dict(const char *fname, dict_T *d);
// Pass 3 Phase 2: swapfile_info Rust function declaration
extern int64_t rs_swapfile_info(char *fname, void *sb, int *proc_running_out);
// Pass 3 Phase 3: ml_replace_buf_len Rust function declaration
extern int rs_ml_replace_buf_len(buf_T *buf, linenr_T lnum, char *line_arg, size_t len_arg,
                                  bool copy, bool noalloc);
// Pass 3 Phase 4: ml_get_buf_impl Rust function declaration
extern char *rs_ml_get_buf_impl(buf_T *buf, linenr_T lnum, bool will_change);
// Pass 4 Phase 1: line-access thin wrapper Rust function declarations
extern char *rs_ml_get_pos(const pos_T *pos);
extern colnr_T rs_ml_get_len(linenr_T lnum);
extern colnr_T rs_ml_get_pos_len(pos_T *pos);
extern colnr_T rs_ml_get_buf_len(buf_T *buf, linenr_T lnum);
extern int rs_gchar_pos(pos_T *pos);
// Pass 4 Phase 2: modification dispatch _impl Rust function declarations
extern int rs_ml_append_flags_impl(linenr_T lnum, char *line, colnr_T len, int flags);
extern int rs_ml_append_buf_impl(buf_T *buf, linenr_T lnum, char *line, colnr_T len, bool newfile);
extern int rs_ml_delete_flags_impl(linenr_T lnum, int flags);
extern int rs_ml_delete_buf_impl(buf_T *buf, linenr_T lnum, bool message);
extern int rs_ml_replace_buf_impl(buf_T *buf, linenr_T lnum, char *line, bool copy, bool noalloc);
// Pass 4 Phase 3: ml_append_flush Rust function declaration
extern int rs_ml_append_flush(buf_T *buf, linenr_T lnum, char *line, colnr_T len, int flags);

static const char e_ml_get_invalid_lnum_nr[]
  = N_("E315: ml_get: Invalid lnum: %" PRId64);
static const char e_ml_get_cannot_find_line_nr_in_buffer_nr_str[]
  = N_("E316: ml_get: Cannot find line %" PRId64 "in buffer %d %s");
static const char e_pointer_block_id_wrong[]
  = N_("E317: Pointer block id wrong");
static const char e_pointer_block_id_wrong_two[]
  = N_("E317: Pointer block id wrong 2");
static const char e_pointer_block_id_wrong_three[]
  = N_("E317: Pointer block id wrong 3");
static const char e_pointer_block_id_wrong_four[]
  = N_("E317: Pointer block id wrong 4");
static const char e_line_number_out_of_range_nr_past_the_end[]
  = N_("E322: Line number out of range: %" PRId64 " past the end");
static const char e_line_count_wrong_in_block_nr[]
  = N_("E323: Line count wrong in block %" PRId64);
static const char e_warning_pointer_block_corrupted[]
  = N_("E1364: Warning: Pointer block corrupted");

#if __has_feature(address_sanitizer)
# define ML_GET_ALLOC_LINES
#endif

/// Open a new memline for "buf".
///
/// @return  FAIL for failure, OK otherwise.
int ml_open(buf_T *buf)
{
  // init fields in memline struct
  buf->b_ml.ml_stack_size = 0;   // no stack yet
  buf->b_ml.ml_stack = NULL;    // no stack yet
  buf->b_ml.ml_stack_top = 0;   // nothing in the stack
  buf->b_ml.ml_locked = NULL;   // no cached block
  buf->b_ml.ml_line_lnum = 0;   // no cached line
  buf->b_ml.ml_line_offset = 0;
  buf->b_ml.ml_chunksize = NULL;
  buf->b_ml.ml_usedchunks = 0;

  if (cmdmod.cmod_flags & CMOD_NOSWAPFILE) {
    buf->b_p_swf = false;
  }

  // When 'updatecount' is non-zero swapfile may be opened later.
  if (!buf->terminal && p_uc && buf->b_p_swf) {
    buf->b_may_swap = true;
  } else {
    buf->b_may_swap = false;
  }

  // Open the memfile.  No swapfile is created yet.
  memfile_T *mfp = mf_open(NULL, 0);
  if (mfp == NULL) {
    goto error;
  }

  buf->b_ml.ml_mfp = mfp;
  buf->b_ml.ml_flags = ML_EMPTY;
  buf->b_ml.ml_line_count = 1;

  // fill block0 struct and write page 0
  bhdr_T *hp = mf_new(mfp, false, 1);
  if (hp->bh_bnum != 0) {
    iemsg(_("E298: Didn't get block nr 0?"));
    goto error;
  }
  ZeroBlock *b0p = hp->bh_data;

  b0p->b0_id[0] = BLOCK0_ID0;
  b0p->b0_id[1] = BLOCK0_ID1;
  b0p->b0_magic_long = B0_MAGIC_LONG;
  b0p->b0_magic_int = B0_MAGIC_INT;
  b0p->b0_magic_short = (int16_t)B0_MAGIC_SHORT;
  b0p->b0_magic_char = B0_MAGIC_CHAR;
  xstrlcpy(xstpcpy(b0p->b0_version, "VIM "), Versions[0], 6);
  rs_long_to_char((long)mfp->mf_page_size, b0p->b0_page_size);

  if (!buf->b_spell) {
    b0p->b0_dirty = buf->b_changed ? B0_DIRTY : 0;
    b0p->b0_flags = (char)(rs_get_fileformat((buf_T *)buf) + 1);
    set_b0_fname(b0p, buf);
    os_get_username(b0p->b0_uname, B0_UNAME_SIZE);
    b0p->b0_uname[B0_UNAME_SIZE - 1] = NUL;
    os_get_hostname(b0p->b0_hname, B0_HNAME_SIZE);
    b0p->b0_hname[B0_HNAME_SIZE - 1] = NUL;
    rs_long_to_char((long)os_get_pid(), b0p->b0_pid);
  }

  // Always sync block number 0 to disk, so we can check the file name in
  // the swapfile in findswapname(). Don't do this for a help files or
  // a spell buffer though.
  // Only works when there's a swapfile, otherwise it's done when the file
  // is created.
  mf_put(mfp, hp, true, false);
  if (!buf->b_help && !buf->b_spell) {
    mf_sync(mfp, 0);
  }

  // Fill in root pointer block and write page 1.
  hp = ml_new_ptr(mfp);
  assert(hp != NULL);
  if (hp->bh_bnum != 1) {
    iemsg(_("E298: Didn't get block nr 1?"));
    goto error;
  }
  PointerBlock *pp = hp->bh_data;
  pp->pb_count = 1;
  pp->pb_pointer[0].pe_bnum = 2;
  pp->pb_pointer[0].pe_page_count = 1;
  pp->pb_pointer[0].pe_old_lnum = 1;
  pp->pb_pointer[0].pe_line_count = 1;      // line count after insertion
  mf_put(mfp, hp, true, false);

  // Allocate first data block and create an empty line 1.
  hp = ml_new_data(mfp, false, 1);
  if (hp->bh_bnum != 2) {
    iemsg(_("E298: Didn't get block nr 2?"));
    goto error;
  }

  DataBlock *dp = hp->bh_data;
  dp->db_index[0] = --dp->db_txt_start;         // at end of block
  dp->db_free -= 1 + (unsigned)INDEX_SIZE;
  dp->db_line_count = 1;
  *((char *)dp + dp->db_txt_start) = NUL;     // empty line

  return OK;

error:
  if (mfp != NULL) {
    if (hp) {
      mf_put(mfp, hp, false, false);
    }
    mf_close(mfp, true);  // will also xfree(mfp->mf_fname)
  }
  buf->b_ml.ml_mfp = NULL;
  return FAIL;
}

/// ml_setname() is called when the file name of "buf" has been changed.
/// It may rename the swapfile.
void ml_setname(buf_T *buf)
{
  bool success = false;

  memfile_T *mfp = buf->b_ml.ml_mfp;
  if (mfp->mf_fd < 0) {             // there is no swapfile yet
    // When 'updatecount' is 0 and 'noswapfile' there is no swapfile.
    // For help files we will make a swapfile now.
    if (p_uc != 0 && (cmdmod.cmod_flags & CMOD_NOSWAPFILE) == 0) {
      ml_open_file(buf);  // create a swapfile
    }
    return;
  }

  // Try all directories in the 'directory' option.
  char *dirp = p_dir;
  bool found_existing_dir = false;
  while (true) {
    if (*dirp == NUL) {             // tried all directories, fail
      break;
    }
    char *fname = findswapname(buf, &dirp, mfp->mf_fname, &found_existing_dir);
    // alloc's fname
    if (dirp == NULL) {             // out of memory
      break;
    }
    if (fname == NULL) {            // no file name found for this dir
      continue;
    }

    // if the file name is the same we don't have to do anything
    if (path_fnamecmp(fname, mfp->mf_fname) == 0) {
      xfree(fname);
      success = true;
      break;
    }
    // need to close the swapfile before renaming
    if (mfp->mf_fd >= 0) {
      close(mfp->mf_fd);
      mfp->mf_fd = -1;
    }

    // try to rename the swapfile
    if (vim_rename(mfp->mf_fname, fname) == 0) {
      success = true;
      mf_free_fnames(mfp);
      mf_set_fnames(mfp, fname);
      ml_upd_block0(buf, UB_SAME_DIR);
      break;
    }
    xfree(fname);                // this fname didn't work, try another
  }

  if (mfp->mf_fd == -1) {           // need to (re)open the swapfile
    mfp->mf_fd = os_open(mfp->mf_fname, O_RDWR, 0);
    if (mfp->mf_fd < 0) {
      // could not (re)open the swapfile, what can we do????
      emsg(_("E301: Oops, lost the swap file!!!"));
      return;
    }
    os_set_cloexec(mfp->mf_fd);
  }
  if (!success) {
    emsg(_("E302: Could not rename swap file"));
  }
}

/// Open a file for the memfile for all buffers that are not readonly or have
/// been modified.
/// Used when 'updatecount' changes from zero to non-zero.
void ml_open_files(void)
{
  FOR_ALL_BUFFERS(buf) {
    if (!buf->b_p_ro || buf->b_changed) {
      ml_open_file(buf);
    }
  }
}

/// Open a swapfile for an existing memfile, if there is no swapfile yet.
/// If we are unable to find a file name, mf_fname will be NULL
/// and the memfile will be in memory only (no recovery possible).
void ml_open_file(buf_T *buf)
{
  memfile_T *mfp = buf->b_ml.ml_mfp;
  if (mfp == NULL || mfp->mf_fd >= 0 || !buf->b_p_swf
      || (cmdmod.cmod_flags & CMOD_NOSWAPFILE)
      || buf->terminal) {
    return;  // nothing to do
  }

  // For a spell buffer use a temp file name.
  if (buf->b_spell) {
    char *fname = vim_tempname();
    if (fname != NULL) {
      mf_open_file(mfp, fname);           // consumes fname!
    }
    buf->b_may_swap = false;
    return;
  }

  // Try all directories in 'directory' option.
  char *dirp = p_dir;
  bool found_existing_dir = false;
  while (true) {
    if (*dirp == NUL) {
      break;
    }
    // There is a small chance that between choosing the swapfile name
    // and creating it, another Vim creates the file.  In that case the
    // creation will fail and we will use another directory.
    char *fname = findswapname(buf, &dirp, NULL, &found_existing_dir);
    if (dirp == NULL) {
      break;        // out of memory
    }
    if (fname == NULL) {
      continue;
    }
    if (mf_open_file(mfp, fname) == OK) {       // consumes fname!
      // don't sync yet in ml_sync_all()
      mfp->mf_dirty = MF_DIRTY_YES_NOSYNC;
      ml_upd_block0(buf, UB_SAME_DIR);

      // Flush block zero, so others can read it
      if (mf_sync(mfp, MFS_ZERO) == OK) {
        // Mark all blocks that should be in the swapfile as dirty.
        // Needed for when the 'swapfile' option was reset, so that
        // the swapfile was deleted, and then on again.
        mf_set_dirty(mfp);
        break;
      }
      // Writing block 0 failed: close the file and try another dir
      mf_close_file(buf, false);
    }
  }

  if (*p_dir != NUL && mfp->mf_fname == NULL) {
    need_wait_return = true;  // call wait_return() later
    no_wait_return++;
    semsg(_("E303: Unable to open swap file for \"%s\", recovery impossible"),
          buf_spname(buf) != NULL ? buf_spname(buf) : buf->b_fname);
    no_wait_return--;
  }

  // don't try to open a swapfile again
  buf->b_may_swap = false;
}

/// If still need to create a swapfile, and starting to edit a not-readonly
/// file, or reading into an existing buffer, create a swapfile now.
///
/// @param newfile reading file into new buffer
/// @param newfile  reading file into new buffer (thin wrapper calling Rust)
void check_need_swap(bool newfile) { rs_check_need_swap(newfile); }

/// Close memline for buffer 'buf' (thin wrapper calling Rust).
///
/// @param del_file  if true, delete the swapfile
void ml_close(buf_T *buf, int del_file) { rs_ml_close(buf, del_file); }

/// Close all existing memlines and memfiles.
/// Only used when exiting.
///
/// @param del_file  if true, delete the memfiles.
void ml_close_all(bool del_file)
{
  FOR_ALL_BUFFERS(buf) {
    ml_close(buf, del_file);
  }
  spell_delete_wordlist();      // delete the internal wordlist
  vim_deltempdir();             // delete created temp directory
}

/// Close all memfiles for not modified buffers.
/// Only use just before exiting!
void ml_close_notmod(void)
{
  FOR_ALL_BUFFERS(buf) {
    if (!bufIsChanged(buf)) {
      ml_close(buf, true);          // close all not-modified buffers
    }
  }
}

/// Update the timestamp in the .swp file (thin wrapper calling Rust).
/// Used when the file has been written.
void ml_timestamp(buf_T *buf) { rs_ml_timestamp(buf); }

/// Update the timestamp or the B0_SAME_DIR flag of the .swp file.
/// Update block 0 (thin wrapper calling Rust).
static void ml_upd_block0(buf_T *buf, upd_block0_T what) { rs_ml_upd_block0(buf, what); }

/// Write file name and timestamp into block 0 (thin wrapper calling Rust).
static void set_b0_fname(ZeroBlock *b0p, buf_T *buf) { rs_set_b0_fname(b0p, buf); }

// Forward declaration for Rust implementation (migrated from C)
extern int rs_recover_names(const char *fname, int do_list, void *ret_list, int nr,
                            char **fname_out);

/// Try to recover curbuf from the .swp file.
///
/// @param checkext  if true, check the extension and detect whether it is a swapfile.
void ml_recover(bool checkext)
{
  buf_T *buf = NULL;
  memfile_T *mfp = NULL;
  char *fname_used = NULL;
  bhdr_T *hp = NULL;
  char *b0_fenc = NULL;
  infoptr_T *ip;
  bool directly;
  bool serious_error = true;
  int orig_file_status = NOTDONE;

  recoverymode = true;
  int called_from_main = (curbuf->b_ml.ml_mfp == NULL);

  // If the file name ends in ".s[a-w][a-z]" we assume this is the swapfile.
  // Otherwise a search is done to find the swapfile(s).
  char *fname = curbuf->b_fname;
  if (fname == NULL) {              // When there is no file name
    fname = "";
  }
  int len = (int)strlen(fname);
  if (checkext && len >= 4
      && STRNICMP(fname + len - 4, ".s", 2) == 0
      && vim_strchr("abcdefghijklmnopqrstuvw", TOLOWER_ASC((uint8_t)fname[len - 2])) != NULL
      && ASCII_ISALPHA(fname[len - 1])) {
    directly = true;
    fname_used = xstrdup(fname);     // make a copy for mf_open()
  } else {
    directly = false;

    // count the number of matching swapfiles
    len = rs_recover_names(fname, false, NULL, 0, NULL);
    if (len == 0) {                 // no swapfiles found
      semsg(_("E305: No swap file found for %s"), fname);
      goto theend;
    }
    int i;
    if (len == 1) {  // one swapfile found, use it
      i = 1;
    } else {  // several swapfiles found, choose
      // list the names of the swapfiles
      rs_recover_names(fname, true, NULL, 0, NULL);
      msg_putchar('\n');
      i = prompt_for_input(_("Enter number of swap file to use (0 to quit): "), 0, false, NULL);
      if (i < 1 || i > len) {
        goto theend;
      }
    }
    // get the swapfile name that will be used
    rs_recover_names(fname, false, NULL, i, &fname_used);
  }
  if (fname_used == NULL) {
    goto theend;  // user chose invalid number.
  }
  // When called from main() still need to initialize storage structure
  if (called_from_main && ml_open(curbuf) == FAIL) {
    getout(1);
  }

  // Allocate a buffer structure for the swapfile that is used for recovery.
  // Only the memline in it is really used.
  buf = xmalloc(sizeof(buf_T));

  // init fields in memline struct
  buf->b_ml.ml_stack_size = 0;          // no stack yet
  buf->b_ml.ml_stack = NULL;            // no stack yet
  buf->b_ml.ml_stack_top = 0;           // nothing in the stack
  buf->b_ml.ml_line_lnum = 0;           // no cached line
  buf->b_ml.ml_line_offset = 0;
  buf->b_ml.ml_locked = NULL;           // no locked block
  buf->b_ml.ml_flags = 0;

  // open the memfile from the old swapfile
  char *p = xstrdup(fname_used);  // save "fname_used" for the message:
  // mf_open() will consume "fname_used"!
  mfp = mf_open(fname_used, O_RDONLY);
  fname_used = p;
  if (mfp == NULL || mfp->mf_fd < 0) {
    semsg(_("E306: Cannot open %s"), fname_used);
    goto theend;
  }
  buf->b_ml.ml_mfp = mfp;

  // The page size set in mf_open() might be different from the page size
  // used in the swapfile, we must get it from block 0.  But to read block
  // 0 we need a page size.  Use the minimal size for block 0 here, it will
  // be set to the real value below.
  mfp->mf_page_size = MIN_SWAP_PAGE_SIZE;

  int hl_id = HLF_E;
  // try to read block 0
  if ((hp = mf_get(mfp, 0, 1)) == NULL) {
    msg_start();
    msg_puts_hl(_("Unable to read block 0 from "), hl_id, true);
    msg_outtrans(mfp->mf_fname, hl_id, true);
    msg_puts_hl(_("\nMaybe no changes were made or Nvim did not update the swap file."), hl_id,
                true);
    msg_end();
    goto theend;
  }
  ZeroBlock *b0p = hp->bh_data;
  if (strncmp(b0p->b0_version, "VIM 3.0", 7) == 0) {
    msg_start();
    msg_outtrans(mfp->mf_fname, 0, true);
    msg_puts_hl(_(" cannot be used with this version of Nvim.\n"), 0, true);
    msg_puts_hl(_("Use Vim version 3.0.\n"), 0, true);
    msg_end();
    goto theend;
  }
  if (rs_ml_check_b0_id(b0p) != 0) {
    semsg(_("E307: %s does not look like a Nvim swap file"), mfp->mf_fname);
    goto theend;
  }
  if (rs_b0_magic_wrong(b0p)) {
    msg_start();
    msg_outtrans(mfp->mf_fname, hl_id, true);
    msg_puts_hl(_(" cannot be used on this computer.\n"), hl_id, true);
    msg_puts_hl(_("The file was created on "), hl_id, true);
    // avoid going past the end of a corrupted hostname
    b0p->b0_fname[0] = NUL;
    msg_puts_hl(b0p->b0_hname, hl_id, true);
    msg_puts_hl(_(",\nor the file has been damaged."), hl_id, true);
    msg_end();
    goto theend;
  }

  // If we guessed the wrong page size, we have to recalculate the
  // highest block number in the file.
  if (mfp->mf_page_size != (unsigned)rs_char_to_long(b0p->b0_page_size)) {
    unsigned previous_page_size = mfp->mf_page_size;

    mf_new_page_size(mfp, (unsigned)rs_char_to_long(b0p->b0_page_size));
    if (mfp->mf_page_size < previous_page_size) {
      msg_start();
      msg_outtrans(mfp->mf_fname, hl_id, true);
      msg_puts_hl(_(" has been damaged (page size is smaller than minimum value).\n"), hl_id, true);
      msg_end();
      goto theend;
    }
    off_T size = vim_lseek(mfp->mf_fd, 0, SEEK_END);
    // 0 means no file or empty file
    mfp->mf_blocknr_max = size <= 0 ? 0 : size / mfp->mf_page_size;
    mfp->mf_infile_count = mfp->mf_blocknr_max;

    // need to reallocate the memory used to store the data
    p = xmalloc(mfp->mf_page_size);
    memmove(p, hp->bh_data, previous_page_size);
    xfree(hp->bh_data);
    hp->bh_data = p;
    b0p = hp->bh_data;
  }

  // If .swp file name given directly, use name from swapfile for buffer.
  if (directly) {
    expand_env(b0p->b0_fname, NameBuff, MAXPATHL);
    if (setfname(curbuf, NameBuff, NULL, true) == FAIL) {
      goto theend;
    }
  }

  home_replace(NULL, mfp->mf_fname, NameBuff, MAXPATHL, true);
  smsg(0, _("Using swap file \"%s\""), NameBuff);

  if (buf_spname(curbuf) != NULL) {
    xstrlcpy(NameBuff, buf_spname(curbuf), MAXPATHL);
  } else {
    home_replace(NULL, curbuf->b_ffname, NameBuff, MAXPATHL, true);
  }
  smsg(0, _("Original file \"%s\""), NameBuff);
  msg_putchar('\n');

  // check date of swapfile and original file
  FileInfo org_file_info;
  FileInfo swp_file_info;
  int mtime = (int)rs_char_to_long(b0p->b0_mtime);
  if (curbuf->b_ffname != NULL
      && os_fileinfo(curbuf->b_ffname, &org_file_info)
      && ((os_fileinfo(mfp->mf_fname, &swp_file_info)
           && org_file_info.stat.st_mtim.tv_sec
           > swp_file_info.stat.st_mtim.tv_sec)
          || org_file_info.stat.st_mtim.tv_sec != mtime)) {
    emsg(_("E308: Warning: Original file may have been changed"));
  }
  ui_flush();

  // Get the 'fileformat' and 'fileencoding' from block zero.
  int b0_ff = (b0p->b0_flags & B0_FF_MASK);
  if (b0p->b0_flags & B0_HAS_FENC) {
    int fnsize = B0_FNAME_SIZE_NOCRYPT;

    for (p = b0p->b0_fname + fnsize; p > b0p->b0_fname && p[-1] != NUL; p--) {}
    b0_fenc = xstrnsave(p, (size_t)(b0p->b0_fname + fnsize - p));
  }

  mf_put(mfp, hp, false, false);        // release block 0
  hp = NULL;

  // Now that we are sure that the file is going to be recovered, clear the
  // contents of the current buffer.
  while (!(curbuf->b_ml.ml_flags & ML_EMPTY)) {
    ml_delete(1);
  }

  // Try reading the original file to obtain the values of 'fileformat',
  // 'fileencoding', etc.  Ignore errors.  The text itself is not used.
  if (curbuf->b_ffname != NULL) {
    orig_file_status = readfile(curbuf->b_ffname, NULL, 0,
                                0, MAXLNUM, NULL, READ_NEW, false);
  }

  // Use the 'fileformat' and 'fileencoding' as stored in the swapfile.
  if (b0_ff != 0) {
    set_fileformat(b0_ff - 1, OPT_LOCAL);
  }
  if (b0_fenc != NULL) {
    set_option_value_give_err(kOptFileencoding, CSTR_AS_OPTVAL(b0_fenc), OPT_LOCAL);
    xfree(b0_fenc);
  }
  unchanged(curbuf, true, true);

  blocknr_T bnum = 1;       // start with block 1
  unsigned page_count = 1;  // which is 1 page
  linenr_T lnum = 0;        // append after line 0 in curbuf
  linenr_T line_count = 0;
  int idx = 0;              // start with first index in block 1
  int error = 0;
  buf->b_ml.ml_stack_top = 0;
  buf->b_ml.ml_stack = NULL;
  buf->b_ml.ml_stack_size = 0;

  bool cannot_open = (curbuf->b_ffname == NULL);

  serious_error = false;
  for (; !got_int; line_breakcheck()) {
    if (hp != NULL) {
      mf_put(mfp, hp, false, false);            // release previous block
    }
    // get block
    if ((hp = mf_get(mfp, bnum, page_count)) == NULL) {
      if (bnum == 1) {
        semsg(_("E309: Unable to read block 1 from %s"), mfp->mf_fname);
        goto theend;
      }
      error++;
      ml_append(lnum++, _("???MANY LINES MISSING"), 0, true);
    } else {          // there is a block
      PointerBlock *pp = hp->bh_data;
      if (pp->pb_id == PTR_ID) {                // it is a pointer block
        bool ptr_block_error = false;
        if (pp->pb_count_max != PB_COUNT_MAX(mfp)) {
          ptr_block_error = true;
          pp->pb_count_max = PB_COUNT_MAX(mfp);
        }
        if (pp->pb_count > pp->pb_count_max) {
          ptr_block_error = true;
          pp->pb_count = pp->pb_count_max;
        }
        if (ptr_block_error) {
          emsg(_(e_warning_pointer_block_corrupted));
        }

        // check line count when using pointer block first time
        if (idx == 0 && line_count != 0) {
          for (int i = 0; i < (int)pp->pb_count; i++) {
            line_count -= pp->pb_pointer[i].pe_line_count;
          }
          if (line_count != 0) {
            error++;
            ml_append(lnum++, _("???LINE COUNT WRONG"), 0, true);
          }
        }

        if (pp->pb_count == 0) {
          ml_append(lnum++, _("???EMPTY BLOCK"), 0, true);
          error++;
        } else if (idx < (int)pp->pb_count) {         // go a block deeper
          if (pp->pb_pointer[idx].pe_bnum < 0) {
            // Data block with negative block number.
            // Try to read lines from the original file.
            // This is slow, but it works.
            if (!cannot_open) {
              line_count = pp->pb_pointer[idx].pe_line_count;
              if (readfile(curbuf->b_ffname, NULL, lnum,
                           pp->pb_pointer[idx].pe_old_lnum - 1, line_count,
                           NULL, 0, false) != OK) {
                cannot_open = true;
              } else {
                lnum += line_count;
              }
            }
            if (cannot_open) {
              error++;
              ml_append(lnum++, _("???LINES MISSING"), 0, true);
            }
            idx++;                  // get same block again for next index
            continue;
          }

          // going one block deeper in the tree
          int top = ml_add_stack(buf);  // new entry in stack
          ip = &(buf->b_ml.ml_stack[top]);
          ip->ip_bnum = bnum;
          ip->ip_index = idx;

          bnum = pp->pb_pointer[idx].pe_bnum;
          line_count = pp->pb_pointer[idx].pe_line_count;
          page_count = (unsigned)pp->pb_pointer[idx].pe_page_count;
          idx = 0;
          continue;
        }
      } else {            // not a pointer block
        DataBlock *dp = hp->bh_data;
        if (dp->db_id != DATA_ID) {             // block id wrong
          if (bnum == 1) {
            semsg(_("E310: Block 1 ID wrong (%s not a .swp file?)"),
                  mfp->mf_fname);
            goto theend;
          }
          error++;
          ml_append(lnum++, _("???BLOCK MISSING"), 0, true);
        } else {
          // It is a data block.
          // Append all the lines in this block.
          bool has_error = false;

          // Check the length of the block.
          // If wrong, use the length given in the pointer block.
          if (page_count * mfp->mf_page_size != dp->db_txt_end) {
            ml_append(lnum++,
                      _("??? from here until ???END lines" " may be messed up"),
                      0, true);
            error++;
            has_error = true;
            dp->db_txt_end = page_count * mfp->mf_page_size;
          }

          // Make sure there is a NUL at the end of the block so we
          // don't go over the end when copying text.
          *((char *)dp + dp->db_txt_end - 1) = NUL;

          // Check the number of lines in the block.
          // If wrong, use the count in the data block.
          if (line_count != dp->db_line_count) {
            ml_append(lnum++,
                      _("??? from here until ???END lines"
                        " may have been inserted/deleted"),
                      0, true);
            error++;
            has_error = true;
          }

          bool did_questions = false;
          for (int i = 0; i < dp->db_line_count; i++) {
            if ((char *)&(dp->db_index[i]) >= (char *)dp + dp->db_txt_start) {
              // line count must be wrong
              error++;
              ml_append(lnum++, _("??? lines may be missing"), 0, true);
              break;
            }

            int txt_start = (dp->db_index[i] & DB_INDEX_MASK);
            if (txt_start <= (int)HEADER_SIZE
                || txt_start >= (int)dp->db_txt_end) {
              error++;
              // avoid lots of lines with "???"
              if (did_questions) {
                continue;
              }
              did_questions = true;
              p = "???";
            } else {
              did_questions = false;
              p = (char *)dp + txt_start;
            }
            ml_append(lnum++, p, 0, true);
          }
          if (has_error) {
            ml_append(lnum++, _("???END"), 0, true);
          }
        }
      }
    }

    if (buf->b_ml.ml_stack_top == 0) {          // finished
      break;
    }

    // go one block up in the tree
    ip = &(buf->b_ml.ml_stack[--(buf->b_ml.ml_stack_top)]);
    bnum = ip->ip_bnum;
    idx = ip->ip_index + 1;         // go to next index
    page_count = 1;
  }

  // Compare the buffer contents with the original file.  When they differ
  // set the 'modified' flag.
  // Lines 1 - lnum are the new contents.
  // Lines lnum + 1 to ml_line_count are the original contents.
  // Line ml_line_count + 1 in the dummy empty line.
  if (orig_file_status != OK || curbuf->b_ml.ml_line_count != lnum * 2 + 1) {
    // Recovering an empty file results in two lines and the first line is
    // empty.  Don't set the modified flag then.
    if (!(curbuf->b_ml.ml_line_count == 2 && *ml_get(1) == NUL)) {
      changed_internal(curbuf);
      buf_inc_changedtick(curbuf);
    }
  } else {
    for (idx = 1; idx <= lnum; idx++) {
      // Need to copy one line, fetching the other one may flush it.
      p = xstrnsave(ml_get(idx), (size_t)ml_get_len(idx));
      int i = strcmp(p, ml_get(idx + lnum));
      xfree(p);
      if (i != 0) {
        changed_internal(curbuf);
        buf_inc_changedtick(curbuf);
        break;
      }
    }
  }

  // Delete the lines from the original file and the dummy line from the
  // empty buffer.  These will now be after the last line in the buffer.
  while (curbuf->b_ml.ml_line_count > lnum
         && !(curbuf->b_ml.ml_flags & ML_EMPTY)) {
    ml_delete(curbuf->b_ml.ml_line_count);
  }
  curbuf->b_flags |= BF_RECOVERED;
  check_cursor(curwin);

  recoverymode = false;
  if (got_int) {
    emsg(_("E311: Recovery Interrupted"));
  } else if (error) {
    no_wait_return++;
    msg(">>>>>>>>>>>>>", 0);
    emsg(_("E312: Errors detected while recovering; look for lines starting with ???"));
    no_wait_return--;
    msg(_("See \":help E312\" for more information."), 0);
    msg(">>>>>>>>>>>>>", 0);
  } else {
    if (curbuf->b_changed) {
      msg(_("Recovery completed. You should check if everything is OK."), 0);
      msg_puts(_("\n(You might want to write out this file under another name\n"));
      msg_puts(_("and run diff with the original file to check for changes)"));
    } else {
      msg(_("Recovery completed. Buffer contents equals file contents."), 0);
    }
    msg_puts(_("\nYou may want to delete the .swp file now."));
    if (rs_swapfile_proc_running(b0p, fname_used)) {
      // Warn there could be an active Vim on the same file, the user may
      // want to kill it.
      msg_puts(_("\nNote: process STILL RUNNING: "));
      msg_outnum((int)rs_char_to_long(b0p->b0_pid));
    }
    msg_puts("\n\n");
    cmdline_row = msg_row;
  }
  redraw_curbuf_later(UPD_NOT_VALID);

theend:
  xfree(fname_used);
  recoverymode = false;
  if (mfp != NULL) {
    if (hp != NULL) {
      mf_put(mfp, hp, false, false);
    }
    mf_close(mfp, false);           // will also xfree(mfp->mf_fname)
  }
  if (buf != NULL) {  // may be NULL if swapfile not found.
    xfree(buf->b_ml.ml_stack);
    xfree(buf);
  }
  if (serious_error && called_from_main) {
    ml_close(curbuf, true);
  } else {
    apply_autocmds(EVENT_BUFREADPOST, NULL, curbuf->b_fname, false, curbuf);
    apply_autocmds(EVENT_BUFWINENTER, NULL, curbuf->b_fname, false, curbuf);
  }
}

// recover_names and recov_file_names migrated to Rust (recovery.rs)

/// Append the full path to name with path separators made into percent signs.
/// (thin wrapper calling Rust)
char *make_percent_swname(char *dir, char *dir_end, const char *name)
  FUNC_ATTR_NONNULL_ARG(1, 2)
{
  return rs_make_percent_swname(dir, dir_end, name);
}

// PID of swapfile owner, or zero if not running.
static int proc_running;

/// For Vimscript "swapinfo()".
///
/// @return  information found in swapfile "fname" in dictionary "d".
void swapfile_dict(const char *fname, dict_T *d)
{
  rs_swapfile_dict(fname, d);
}

/// Loads info from swapfile `fname`, and displays it to the user.
///
/// @return  timestamp (0 when unknown).
static time_t swapfile_info(char *fname, StringBuilder *msg)
{
  int running = 0;
  int64_t mtime = rs_swapfile_info(fname, msg, &running);
  proc_running = running;
  return (time_t)mtime;
}

// recov_file_names migrated to Rust (recovery.rs)

/// sync all memlines
///
/// @param check_file  if true, check if original file exists and was not changed.
/// @param check_char  if true, stop syncing when character becomes available, but
///
/// always sync at least one block.
void ml_sync_all(int check_file, int check_char, bool do_fsync)
{
  FOR_ALL_BUFFERS(buf) {
    if (buf->b_ml.ml_mfp == NULL || buf->b_ml.ml_mfp->mf_fname == NULL) {
      continue;                             // no file
    }
    ml_flush_line(buf, false);              // flush buffered line
                                            // flush locked block
    ml_find_line(buf, 0, ML_FLUSH);
    if (bufIsChanged(buf) && check_file && mf_need_trans(buf->b_ml.ml_mfp)
        && buf->b_ffname != NULL) {
      // If the original file does not exist anymore or has been changed
      // call ml_preserve() to get rid of all negative numbered blocks.
      FileInfo file_info;
      if (!os_fileinfo(buf->b_ffname, &file_info)
          || file_info.stat.st_mtim.tv_sec != buf->b_mtime_read
          || file_info.stat.st_mtim.tv_nsec != buf->b_mtime_read_ns
          || os_fileinfo_size(&file_info) != buf->b_orig_size) {
        ml_preserve(buf, false, do_fsync);
        did_check_timestamps = false;
        need_check_timestamps = true;           // give message later
      }
    }
    if (buf->b_ml.ml_mfp->mf_dirty == MF_DIRTY_YES) {
      mf_sync(buf->b_ml.ml_mfp, (check_char ? MFS_STOP : 0)
              | (do_fsync && bufIsChanged(buf) ? MFS_FLUSH : 0));
      if (check_char && os_char_avail()) {      // character available now
        break;
      }
    }
  }
}

/// sync one buffer, including negative blocks
///
/// after this all the blocks are in the swapfile
///
/// Used for the :preserve command and when the original file has been
/// changed or deleted.
///
/// @param message  if true, the success of preserving is reported.
void ml_preserve(buf_T *buf, bool message, bool do_fsync)
{
  memfile_T *mfp = buf->b_ml.ml_mfp;
  int got_int_save = got_int;

  if (mfp == NULL || mfp->mf_fname == NULL) {
    if (message) {
      emsg(_("E313: Cannot preserve, there is no swap file"));
    }
    return;
  }

  // We only want to stop when interrupted here, not when interrupted
  // before.
  got_int = false;

  ml_flush_line(buf, false);        // flush buffered line
  ml_find_line(buf, 0, ML_FLUSH);   // flush locked block
  int status = mf_sync(mfp, MFS_ALL | (do_fsync ? MFS_FLUSH : 0));

  // stack is invalid after mf_sync(.., MFS_ALL)
  buf->b_ml.ml_stack_top = 0;

  // Some of the data blocks may have been changed from negative to
  // positive block number. In that case the pointer blocks need to be
  // updated.
  //
  // We don't know in which pointer block the references are, so we visit
  // all data blocks until there are no more translations to be done (or
  // we hit the end of the file, which can only happen in case a write fails,
  // e.g. when file system if full).
  // ml_find_line() does the work by translating the negative block numbers
  // when getting the first line of each data block.
  if (mf_need_trans(mfp) && !got_int) {
    linenr_T lnum = 1;
    while (mf_need_trans(mfp) && lnum <= buf->b_ml.ml_line_count) {
      bhdr_T *hp = ml_find_line(buf, lnum, ML_FIND);
      if (hp == NULL) {
        status = FAIL;
        goto theend;
      }
      CHECK(buf->b_ml.ml_locked_low != lnum, "low != lnum");
      lnum = buf->b_ml.ml_locked_high + 1;
    }
    ml_find_line(buf, 0, ML_FLUSH);  // flush locked block
    // sync the updated pointer blocks
    if (mf_sync(mfp, MFS_ALL | (do_fsync ? MFS_FLUSH : 0)) == FAIL) {
      status = FAIL;
    }
    buf->b_ml.ml_stack_top = 0;  // stack is invalid now
  }
theend:
  got_int |= got_int_save;

  if (message) {
    if (status == OK) {
      msg(_("File preserved"), 0);
    } else {
      emsg(_("E314: Preserve failed"));
    }
  }
}

// NOTE: The pointer returned by the ml_get_*() functions only remains valid
// until the next call!
//  line1 = ml_get(1);
//  line2 = ml_get(2);  // line1 is now invalid!
// Make a copy of the line if necessary.

/// @return  a pointer to a (read-only copy of a) line in curbuf.
///
/// On failure an error message is given and IObuff is returned (to avoid
/// having to check for error everywhere).
char *ml_get(linenr_T lnum)
{
  return rs_ml_get_buf_impl(curbuf, lnum, false);
}

/// @return  a pointer to a (read-only copy of a) line.
///
/// This is the same as ml_get(), but taking in the buffer
/// as an argument.
char *ml_get_buf(buf_T *buf, linenr_T lnum)
{
  return rs_ml_get_buf_impl(buf, lnum, false);
}

/// Like `ml_get_buf`, but allow the line to be mutated in place.
///
/// This is very limited. Generally ml_replace_buf()
/// should be used to modify a line.
///
/// @return a pointer to a line in the buffer
char *ml_get_buf_mut(buf_T *buf, linenr_T lnum)
{
  return rs_ml_get_buf_impl(buf, lnum, true);
}

/// @return  pointer to position "pos".
char *ml_get_pos(const pos_T *pos)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_ml_get_pos(pos);
}

/// @return  length (excluding the NUL) of the given line.
colnr_T ml_get_len(linenr_T lnum)
{
  return rs_ml_get_len(lnum);
}

/// @return  length (excluding the NUL) of the text after position "pos".
colnr_T ml_get_pos_len(pos_T *pos)
{
  return rs_ml_get_pos_len(pos);
}

/// @return  length (excluding the NUL) of the given line in the given buffer.
colnr_T ml_get_buf_len(buf_T *buf, linenr_T lnum)
{
  return rs_ml_get_buf_len(buf, lnum);
}

/// @return  codepoint at pos. pos must be either valid or have col set to MAXCOL!
int gchar_pos(pos_T *pos)
  FUNC_ATTR_NONNULL_ARG(1)
{
  return rs_gchar_pos(pos);
}

// =============================================================================
// C accessors for Rust FFI (memline crate)
// =============================================================================

// Current buffer accessor
int nvim_curbuf_get_ml_flags(void) { return curbuf->b_ml.ml_flags; }

// ML_LINE_DIRTY constant accessor
int nvim_get_ml_line_dirty(void) { return ML_LINE_DIRTY; }

// Buffer memline field accessors
memfile_T *nvim_buf_get_ml_mfp(buf_T *buf) { return buf->b_ml.ml_mfp; }
int nvim_buf_get_ml_flags(buf_T *buf) { return buf->b_ml.ml_flags; }
void nvim_buf_set_ml_flags(buf_T *buf, int flags) { buf->b_ml.ml_flags = flags; }
linenr_T nvim_buf_get_ml_line_lnum(buf_T *buf) { return buf->b_ml.ml_line_lnum; }
linenr_T nvim_buf_get_ml_line_count(buf_T *buf) { return buf->b_ml.ml_line_count; }
colnr_T nvim_buf_get_ml_line_len(buf_T *buf) { return buf->b_ml.ml_line_len; }
void nvim_buf_set_ml_line_len(buf_T *buf, colnr_T len) { buf->b_ml.ml_line_len = len; }
char *nvim_buf_get_ml_line_ptr(buf_T *buf) { return buf->b_ml.ml_line_ptr; }

// Position accessors
linenr_T nvim_pos_get_lnum(const pos_T *pos) { return pos->lnum; }
colnr_T nvim_pos_get_col(const pos_T *pos) { return pos->col; }

// Constants
colnr_T nvim_get_maxcol(void) { return MAXCOL; }
size_t nvim_get_maxpathl(void) { return MAXPATHL; }

// Validation helpers
int nvim_buf_has_ml_mfp(buf_T *buf) { return buf->b_ml.ml_mfp != NULL; }
int nvim_buf_get_ml_usedchunks(buf_T *buf) { return buf->b_ml.ml_usedchunks; }


// Byte offset cache accessor
size_t nvim_buf_get_ml_line_offset(buf_T *buf) { return buf->b_ml.ml_line_offset; }
void nvim_buf_set_ml_line_offset(buf_T *buf, size_t offset) { buf->b_ml.ml_line_offset = offset; }

// Locked block line range accessors
linenr_T nvim_buf_get_ml_locked_high(buf_T *buf) { return buf->b_ml.ml_locked_high; }
linenr_T nvim_buf_get_ml_locked_low(buf_T *buf) { return buf->b_ml.ml_locked_low; }

// Chunk size accessors (index into ml_chunksize[])
int nvim_buf_get_ml_chunksize_numlines(buf_T *buf, int idx) { return buf->b_ml.ml_chunksize[idx].mlcs_numlines; }
int nvim_buf_get_ml_chunksize_totalsize(buf_T *buf, int idx) { return buf->b_ml.ml_chunksize[idx].mlcs_totalsize; }
int nvim_buf_get_ml_chunksize_is_null(buf_T *buf) { return buf->b_ml.ml_chunksize == NULL; }

// Block header data accessor (returns void* to DataBlock)
void *nvim_bhdr_get_bh_data(bhdr_T *hp) { return hp->bh_data; }

// Swap file name accessors for recover_names (Phase 2)
char *nvim_buf_get_ml_mfp_fname(buf_T *buf)
{
  return (buf->b_ml.ml_mfp != NULL) ? buf->b_ml.ml_mfp->mf_fname : NULL;
}
char *nvim_get_p_dir(void) { return p_dir; }

// Pass 3 Phase 3: ml_replace_buf_len accessors
void nvim_buf_set_ml_line_ptr(buf_T *buf, char *ptr) { buf->b_ml.ml_line_ptr = ptr; }
void nvim_buf_set_ml_line_lnum(buf_T *buf, linenr_T lnum) { buf->b_ml.ml_line_lnum = lnum; }
/// open_buffer() if ml_mfp is NULL. Returns FAIL if it fails.
int nvim_buf_open_buffer_if_needed(buf_T *buf)
{
  if (buf->b_ml.ml_mfp == NULL) {
    return open_buffer(false, NULL, 0);
  }
  return OK;
}

// Pass 3 Phase 4: Error message wrappers for rs_ml_get_buf_impl
/// Emit the "invalid lnum" internal error for ml_get_buf_impl.
/// Wraps: siemsg(_(e_ml_get_invalid_lnum_nr), lnum)
void nvim_siemsg_ml_get_invalid_lnum(int64_t lnum)
{
  siemsg(_(e_ml_get_invalid_lnum_nr), lnum);
}

/// Emit the "cannot find line" internal error for ml_get_buf_impl.
/// Wraps: get_trans_bufname + shorten_dir + siemsg(...)
void nvim_siemsg_ml_get_cannot_find_line(int64_t lnum, buf_T *buf)
{
  get_trans_bufname(buf);
  shorten_dir(NameBuff);
  siemsg(_(e_ml_get_cannot_find_line_nr_in_buffer_nr_str),
         lnum, buf->b_fnum, NameBuff);
}

// Print swapfile info (wraps swapfile_info which uses StringBuilder internally)
void nvim_swapfile_info_and_print(char *fname)
{
  StringBuilder msg = KV_INITIAL_VALUE;
  kv_resize(msg, IOSIZE);
  swapfile_info(fname, &msg);
  bool need_clear = false;
  msg_multiline(cbuf_as_string(msg.items, msg.size), 0, false, false, &need_clear);
  kv_destroy(msg);
}

// Pass 3 Phase 2: C wrappers for StringBuilder operations (used by rs_swapfile_info)

/// Append a string to a StringBuilder (opaque void* pointer from Rust)
void nvim_sb_append_str(void *sb, const char *s)
{
  kv_printf(*(StringBuilder *)sb, "%s", s);
}

/// Get file mtime and owner name from a file, for swapfile_info display.
/// On UNIX, also fills uname_buf with the owner's name (if available).
/// Returns mtime in seconds, or 0 if file not found.
/// uname_found is set to 1 if uname_buf was filled.
int64_t nvim_swapfile_get_file_info(const char *fname, char *uname_buf, size_t uname_len,
                                    int *uname_found)
{
  FileInfo file_info;
  *uname_found = 0;
  if (!os_fileinfo(fname, &file_info)) {
    return 0;
  }
#ifdef UNIX
  if (os_get_uname((uv_uid_t)file_info.stat.st_uid, uname_buf, uname_len) == OK) {
    *uname_found = 1;
  }
#endif
  return (int64_t)file_info.stat.st_mtim.tv_sec;
}

/// Append the ctime string for an mtime value to a StringBuilder.
void nvim_sb_append_ctime(void *sb, int64_t mtime)
{
  time_t x = (time_t)mtime;
  char ctime_buf[100];
  kv_printf(*(StringBuilder *)sb, "%s", os_ctime_r(&x, ctime_buf, sizeof(ctime_buf), true));
}

// Translated message helpers for rs_swapfile_info

void nvim_sb_swapinfo_owned_by(void *sb, const char *uname)
{
  kv_printf(*(StringBuilder *)sb, "%s%s", _("          owned by: "), uname);
}

void nvim_sb_swapinfo_dated(void *sb, int has_owner)
{
  if (has_owner) {
    kv_printf(*(StringBuilder *)sb, _("   dated: "));
  } else {
    kv_printf(*(StringBuilder *)sb, _("             dated: "));
  }
}

void nvim_sb_swapinfo_vim3(void *sb)
{
  kv_printf(*(StringBuilder *)sb, _("         [from Vim version 3.0]"));
}

void nvim_sb_swapinfo_not_nvim(void *sb)
{
  kv_printf(*(StringBuilder *)sb, _("         [does not look like a Nvim swap file]"));
}

void nvim_sb_swapinfo_garbled(void *sb)
{
  kv_printf(*(StringBuilder *)sb, _("         [garbled strings (not nul terminated)]"));
}

void nvim_sb_swapinfo_filename(void *sb, const char *b0_fname)
{
  kv_printf(*(StringBuilder *)sb, _("         file name: "));
  if (b0_fname[0] == NUL) {
    kv_printf(*(StringBuilder *)sb, _("[No Name]"));
  } else {
    kv_printf(*(StringBuilder *)sb, "%s", b0_fname);
  }
}

void nvim_sb_swapinfo_modified(void *sb, int dirty)
{
  kv_printf(*(StringBuilder *)sb, _("\n          modified: "));
  kv_printf(*(StringBuilder *)sb, dirty ? _("YES") : _("no"));
}

void nvim_sb_swapinfo_user(void *sb, const char *uname)
{
  kv_printf(*(StringBuilder *)sb, _("\n         user name: "));
  kv_printf(*(StringBuilder *)sb, "%s", uname);
}

void nvim_sb_swapinfo_host(void *sb, const char *hname, int after_user)
{
  if (after_user) {
    kv_printf(*(StringBuilder *)sb, _("   host name: "));
  } else {
    kv_printf(*(StringBuilder *)sb, _("\n         host name: "));
  }
  kv_printf(*(StringBuilder *)sb, "%s", hname);
}

void nvim_sb_swapinfo_pid(void *sb, int pid)
{
  kv_printf(*(StringBuilder *)sb, _("\n        process ID: "));
  kv_printf(*(StringBuilder *)sb, "%d", pid);
}

void nvim_sb_swapinfo_still_running(void *sb)
{
  kv_printf(*(StringBuilder *)sb, _(" (STILL RUNNING)"));
}

void nvim_sb_swapinfo_not_usable(void *sb)
{
  kv_printf(*(StringBuilder *)sb, _("\n         [not usable on this computer]"));
}

void nvim_sb_swapinfo_cannot_read(void *sb)
{
  kv_printf(*(StringBuilder *)sb, _("         [cannot be read]"));
}

void nvim_sb_swapinfo_cannot_open(void *sb)
{
  kv_printf(*(StringBuilder *)sb, _("         [cannot be opened]"));
}

// Phase 4 accessors for Rust FFI (ml_new_ptr, ml_new_data, ml_lineadd, ml_upd_block0)

// B-tree stack accessors
int nvim_buf_get_ml_stack_top(buf_T *buf) { return buf->b_ml.ml_stack_top; }
infoptr_T *nvim_buf_get_ml_stack_ip(buf_T *buf, int idx) { return &(buf->b_ml.ml_stack[idx]); }
int64_t nvim_ip_get_bnum(const infoptr_T *ip) { return (int64_t)ip->ip_bnum; }
int nvim_ip_get_index(const infoptr_T *ip) { return ip->ip_index; }
void nvim_ip_add_high(infoptr_T *ip, int count) { ip->ip_high += count; }

// PointerBlock field accessors
uint16_t nvim_pp_get_id(const void *pp) { return ((const PointerBlock *)pp)->pb_id; }
void nvim_pp_pe_linecount_add(void *pp, int idx, int count)
{
  ((PointerBlock *)pp)->pb_pointer[idx].pe_line_count += count;
}

// upd_block0_T enum constants
int nvim_get_ub_fname(void) { return UB_FNAME; }
void nvim_iemsg_pointer_block_id_wrong_two(void) { iemsg(_(e_pointer_block_id_wrong_two)); }
void nvim_iemsg_e304_upd_block0(void) { iemsg(_("E304: ml_upd_block0(): Didn't get block 0??")); }

// buf->b_ffname accessor (use existing buffer.c version)
// buf->b_mtime accessors
int64_t nvim_buf_get_b_mtime(const buf_T *buf) { return buf->b_mtime; }
void nvim_buf_set_b_mtime(buf_T *buf, int64_t val) { buf->b_mtime = val; }
int64_t nvim_buf_get_b_mtime_ns(const buf_T *buf) { return buf->b_mtime_ns; }
void nvim_buf_set_b_mtime_ns(buf_T *buf, int64_t val) { buf->b_mtime_ns = val; }
int64_t nvim_buf_get_b_mtime_read(const buf_T *buf) { return buf->b_mtime_read; }
void nvim_buf_set_b_mtime_read(buf_T *buf, int64_t val) { buf->b_mtime_read = val; }
int64_t nvim_buf_get_b_mtime_read_ns(const buf_T *buf) { return buf->b_mtime_read_ns; }
void nvim_buf_set_b_mtime_read_ns(buf_T *buf, int64_t val) { buf->b_mtime_read_ns = val; }
int64_t nvim_buf_get_b_orig_size(const buf_T *buf) { return (int64_t)buf->b_orig_size; }
void nvim_buf_set_b_orig_size(buf_T *buf, int64_t val) { buf->b_orig_size = (uint64_t)val; }
int nvim_buf_get_b_orig_mode(const buf_T *buf) { return buf->b_orig_mode; }
void nvim_buf_set_b_orig_mode(buf_T *buf, int val) { buf->b_orig_mode = val; }

// ZeroBlock field setters for set_b0_fname
void nvim_b0_set_fname0(ZeroBlock *b0p) { b0p->b0_fname[0] = NUL; }
char *nvim_b0_get_fname_for_replace(ZeroBlock *b0p) { return b0p->b0_fname; }
int nvim_b0_get_fname0(const ZeroBlock *b0p) { return (unsigned char)b0p->b0_fname[0]; }
// mtime/ino setters for set_b0_fname
char *nvim_b0_get_mtime(ZeroBlock *b0p) { return b0p->b0_mtime; }
char *nvim_b0_get_ino(ZeroBlock *b0p) { return b0p->b0_ino; }

// home_replace wrapper: write home-replaced path into b0_fname
void nvim_home_replace_b0_fname(const buf_T *buf, ZeroBlock *b0p, size_t maxlen)
{
  home_replace(NULL, buf->b_ffname, b0p->b0_fname, maxlen, true);
}

// os_get_username wrapper
int nvim_os_get_username(char *buf, size_t len) { return os_get_username(buf, len); }

// Wrapper: fills b0_mtime and b0_ino from buf->b_ffname, returns 1 on success
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

// Position accessors/setters
colnr_T nvim_pos_get_coladd(const pos_T *pos) { return pos->coladd; }
void nvim_pos_set_lnum(pos_T *pos, linenr_T lnum) { pos->lnum = lnum; }
void nvim_pos_set_col(pos_T *pos, colnr_T col) { pos->col = col; }
void nvim_pos_set_coladd(pos_T *pos, colnr_T coladd) { pos->coladd = coladd; }

// Opaque handle accessors for cursor crate (take void* for FFI compatibility)
linenr_T nvim_buf_get_line_count(void *buf)
{
  if (buf == NULL) {
    return 0;
  }
  return ((buf_T *)buf)->b_ml.ml_line_count;
}

colnr_T nvim_buf_get_line_len(void *buf, linenr_T lnum)
{
  if (buf == NULL) {
    return 0;
  }
  buf_T *b = (buf_T *)buf;
  if (lnum < 1 || lnum > b->b_ml.ml_line_count) {
    return 0;
  }
  return ml_get_buf_len(b, lnum);
}

// ZeroBlock field accessors for Rust FFI (Phase 1 migration)
int64_t nvim_b0_get_magic_long(const ZeroBlock *b0p) { return (int64_t)b0p->b0_magic_long; }
int32_t nvim_b0_get_magic_int(const ZeroBlock *b0p) { return (int32_t)b0p->b0_magic_int; }
int16_t nvim_b0_get_magic_short(const ZeroBlock *b0p) { return b0p->b0_magic_short; }
uint8_t nvim_b0_get_magic_char(const ZeroBlock *b0p) { return (uint8_t)b0p->b0_magic_char; }
uint8_t nvim_b0_get_id(const ZeroBlock *b0p, int idx) { return (uint8_t)b0p->b0_id[idx]; }
const char *nvim_b0_get_version_ptr(const ZeroBlock *b0p) { return b0p->b0_version; }
size_t nvim_b0_get_version_size(void) { return sizeof(((ZeroBlock *)NULL)->b0_version); }
const char *nvim_b0_get_uname_ptr(const ZeroBlock *b0p) { return b0p->b0_uname; }
const char *nvim_b0_get_hname_ptr(const ZeroBlock *b0p) { return b0p->b0_hname; }
const char *nvim_b0_get_fname_ptr(const ZeroBlock *b0p) { return b0p->b0_fname; }

// File inode accessor for fnamecmp_ino native Rust implementation
uint64_t nvim_get_file_inode(const char *fname)
{
  FileInfo file_info;
  if (os_fileinfo(fname, &file_info)) {
    return os_fileinfo_inode(&file_info);
  }
  return 0;
}

// Pass 2 Phase 3: Buffer lifecycle accessors for Rust FFI
// ml_close accessors
void *nvim_buf_get_ml_stack_void(buf_T *buf) { return buf->b_ml.ml_stack; }
void nvim_buf_clear_ml_after_close(buf_T *buf)
{
  buf->b_ml.ml_mfp = NULL;
  buf->b_flags &= ~BF_RECOVERED;
}
void nvim_buf_xfree_clear_ml_chunksize(buf_T *buf)
{
  xfree(buf->b_ml.ml_chunksize);
  buf->b_ml.ml_chunksize = NULL;
}
// ml_flush_deleted_bytes accessors
size_t nvim_buf_get_deleted_bytes(buf_T *buf) { return buf->deleted_bytes; }
void nvim_buf_set_deleted_bytes(buf_T *buf, size_t val) { buf->deleted_bytes = val; }
size_t nvim_buf_get_deleted_codepoints(buf_T *buf) { return buf->deleted_codepoints; }
void nvim_buf_set_deleted_codepoints(buf_T *buf, size_t val) { buf->deleted_codepoints = val; }
size_t nvim_buf_get_deleted_codeunits(buf_T *buf) { return buf->deleted_codeunits; }
void nvim_buf_set_deleted_codeunits(buf_T *buf, size_t val) { buf->deleted_codeunits = val; }
// check_need_swap accessors
// nvim_get_msg_silent / nvim_set_msg_silent - already defined in message.c
// nvim_buf_get_b_may_swap - already defined in change_ffi.c (returns bool)
// nvim_buf_get_b_p_ro     - already defined in buffer.c (returns int)
// ml_setflags accessors (also used by Phase 5)
bhdr_T *nvim_mf_get_block0_hp(memfile_T *mfp) { return pmap_get(int64_t)(&mfp->mf_hash, 0); }
void nvim_bhdr_set_bh_flags_dirty(bhdr_T *hp) { hp->bh_flags |= BH_DIRTY; }

// Pass 2 Phase 4: Deleted-length tracking and stack accessors for Rust FFI
// nvim_get_inhibit_delete_count - already in change_ffi.c
// nvim_buf_get_deleted_bytes2 / nvim_buf_set_deleted_bytes2 - already in buffer.c
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

// Phase 2 accessors for Rust FFI
uint8_t nvim_b0_get_flags_byte(const ZeroBlock *b0p) { return (uint8_t)b0p->b0_flags; }
void nvim_b0_set_flags_byte(ZeroBlock *b0p, uint8_t val) { b0p->b0_flags = (char)val; }
char *nvim_b0_get_fname_mut(ZeroBlock *b0p) { return b0p->b0_fname; }
const char *nvim_b0_get_pid_ptr(const ZeroBlock *b0p) { return b0p->b0_pid; }
uint8_t nvim_b0_get_dirty(const ZeroBlock *b0p) { return (uint8_t)b0p->b0_dirty; }
void nvim_b0_set_hname_end(ZeroBlock *b0p) { b0p->b0_hname[B0_HNAME_SIZE - 1] = NUL; }
size_t nvim_b0_get_struct_size(void) { return sizeof(ZeroBlock); }

// Get mtime of a file (returns 0 if file not found)
int64_t nvim_get_file_mtime(const char *fname)
{
  FileInfo file_info;
  if (os_fileinfo(fname, &file_info)) {
    return (int64_t)file_info.stat.st_mtim.tv_sec;
  }
  return 0;
}

/// @param lnum  append after this line (can be 0)
/// @param line_arg  text of the new line
/// @param len_arg  length of line, including NUL, or 0
/// @param flags  ML_APPEND_ flags
///
/// @return  FAIL for failure, OK otherwise
static int ml_append_int(buf_T *buf, linenr_T lnum, char *line_arg, colnr_T len_arg, int flags)
  FUNC_ATTR_NONNULL_ARG(1)
{
  char *line = line_arg;
  colnr_T len = len_arg;

  if (lnum > buf->b_ml.ml_line_count || buf->b_ml.ml_mfp == NULL) {
    return FAIL;  // lnum out of range
  }

  rs_ml_adjust_lowest_marked_for_insert(lnum);

  if (len == 0) {
    len = (colnr_T)strlen(line) + 1;            // space needed for the text
  }
  int space_needed = len + (int)INDEX_SIZE;     // space needed for text + index

  memfile_T *mfp = buf->b_ml.ml_mfp;
  int page_size = (int)mfp->mf_page_size;

  // find the data block containing the previous line
  // This also fills the stack with the blocks from the root to the data block
  // This also releases any locked block.
  int ret = FAIL;
  bhdr_T *hp;
  if ((hp = ml_find_line(buf, lnum == 0 ? 1 : lnum,
                         ML_INSERT)) == NULL) {
    goto theend;
  }

  buf->b_ml.ml_flags &= ~ML_EMPTY;

  int db_idx;                   // index for lnum in data block
  if (lnum == 0) {              // got line one instead, correct db_idx
    db_idx = -1;                // careful, it is negative!
  } else {
    db_idx = lnum - buf->b_ml.ml_locked_low;
  }
  // get line count (number of indexes in current block) before the insertion
  int line_count = buf->b_ml.ml_locked_high - buf->b_ml.ml_locked_low;

  DataBlock *dp = hp->bh_data;

  // If
  // - there is not enough room in the current block
  // - appending to the last line in the block
  // - not appending to the last line in the file
  // insert in front of the next block.
  if ((int)dp->db_free < space_needed && db_idx == line_count - 1
      && lnum < buf->b_ml.ml_line_count) {
    // Now that the line is not going to be inserted in the block that we
    // expected, the line count has to be adjusted in the pointer blocks
    // by using ml_locked_lineadd.
    (buf->b_ml.ml_locked_lineadd)--;
    (buf->b_ml.ml_locked_high)--;
    if ((hp = ml_find_line(buf, lnum + 1, ML_INSERT)) == NULL) {
      goto theend;
    }

    db_idx = -1;                    // careful, it is negative!
    // get line count before the insertion
    line_count = buf->b_ml.ml_locked_high - buf->b_ml.ml_locked_low;
    CHECK(buf->b_ml.ml_locked_low != lnum + 1, "locked_low != lnum + 1");

    dp = hp->bh_data;
  }

  if (buf->b_prev_line_count == 0) {
    buf->b_prev_line_count = buf->b_ml.ml_line_count;
  }
  buf->b_ml.ml_line_count++;

  if ((int)dp->db_free >= space_needed) {       // enough room in data block
    // Insert the new line in an existing data block, or in the data block
    // allocated above.
    dp->db_txt_start -= (unsigned)len;
    dp->db_free -= (unsigned)space_needed;
    dp->db_line_count++;

    // move the text of the lines that follow to the front
    // adjust the indexes of the lines that follow
    if (line_count > db_idx + 1) {          // if there are following lines
      // Offset is the start of the previous line.
      // This will become the character just after the new line.
      int offset = db_idx < 0 ? (int)dp->db_txt_end
                              : (int)((dp->db_index[db_idx]) & DB_INDEX_MASK);
      memmove((char *)dp + dp->db_txt_start,
              (char *)dp + dp->db_txt_start + len,
              (size_t)offset - (dp->db_txt_start + (size_t)len));
      for (int i = line_count - 1; i > db_idx; i--) {
        dp->db_index[i + 1] = dp->db_index[i] - (unsigned)len;
      }
      dp->db_index[db_idx + 1] = (unsigned)(offset - len);
    } else {
      // add line at the end (which is the start of the text)
      dp->db_index[db_idx + 1] = dp->db_txt_start;
    }

    // copy the text into the block
    memmove((char *)dp + dp->db_index[db_idx + 1], line, (size_t)len);
    if (flags & ML_APPEND_MARK) {
      dp->db_index[db_idx + 1] |= DB_MARKED;
    }

    // Mark the block dirty.
    buf->b_ml.ml_flags |= ML_LOCKED_DIRTY;
    if (!(flags & ML_APPEND_NEW)) {
      buf->b_ml.ml_flags |= ML_LOCKED_POS;
    }
  } else {        // not enough space in data block
    int line_count_left, line_count_right;
    int page_count_left, page_count_right;
    bhdr_T *hp_left;
    bhdr_T *hp_right;
    bhdr_T *hp_new;
    int lines_moved;
    int data_moved = 0;                     // init to shut up gcc
    int total_moved = 0;                    // init to shut up gcc
    int stack_idx;
    bool in_left;
    linenr_T lnum_left, lnum_right;
    PointerBlock *pp_new;

    // There is not enough room, we have to create a new data block and
    // copy some lines into it.
    // Then we have to insert an entry in the pointer block.
    // If this pointer block also is full, we go up another block, and so
    // on, up to the root if necessary.
    // The line counts in the pointer blocks have already been adjusted by
    // ml_find_line().
    //
    // We are going to allocate a new data block. Depending on the
    // situation it will be put to the left or right of the existing
    // block.  If possible we put the new line in the left block and move
    // the lines after it to the right block. Otherwise the new line is
    // also put in the right block. This method is more efficient when
    // inserting a lot of lines at one place.
    if (db_idx < 0) {           // left block is new, right block is existing
      lines_moved = 0;
      in_left = true;
      // space_needed does not change
    } else {                  // left block is existing, right block is new
      lines_moved = line_count - db_idx - 1;
      if (lines_moved == 0) {
        in_left = false;                // put new line in right block
                                        // space_needed does not change
      } else {
        data_moved = (int)(((dp->db_index[db_idx]) & DB_INDEX_MASK) -
                           dp->db_txt_start);
        total_moved = data_moved + lines_moved * (int)INDEX_SIZE;
        if ((int)dp->db_free + total_moved >= space_needed) {
          in_left = true;               // put new line in left block
          space_needed = total_moved;
        } else {
          in_left = false;                  // put new line in right block
          space_needed += total_moved;
        }
      }
    }

    int page_count = ((space_needed + (int)HEADER_SIZE) + page_size - 1) / page_size;
    hp_new = ml_new_data(mfp, flags & ML_APPEND_NEW, page_count);
    if (db_idx < 0) {           // left block is new
      hp_left = hp_new;
      hp_right = hp;
      line_count_left = 0;
      line_count_right = line_count;
    } else {                  // right block is new
      hp_left = hp;
      hp_right = hp_new;
      line_count_left = line_count;
      line_count_right = 0;
    }
    DataBlock *dp_right = hp_right->bh_data;
    DataBlock *dp_left = hp_left->bh_data;
    blocknr_T bnum_left = hp_left->bh_bnum;
    blocknr_T bnum_right = hp_right->bh_bnum;
    page_count_left = (int)hp_left->bh_page_count;
    page_count_right = (int)hp_right->bh_page_count;

    // May move the new line into the right/new block.
    if (!in_left) {
      dp_right->db_txt_start -= (unsigned)len;
      dp_right->db_free -= (unsigned)len + (unsigned)INDEX_SIZE;
      dp_right->db_index[0] = dp_right->db_txt_start;
      if (flags & ML_APPEND_MARK) {
        dp_right->db_index[0] |= DB_MARKED;
      }

      memmove((char *)dp_right + dp_right->db_txt_start,
              line, (size_t)len);
      line_count_right++;
    }
    // may move lines from the left/old block to the right/new one.
    if (lines_moved) {
      dp_right->db_txt_start -= (unsigned)data_moved;
      dp_right->db_free -= (unsigned)total_moved;
      memmove((char *)dp_right + dp_right->db_txt_start,
              (char *)dp_left + dp_left->db_txt_start,
              (size_t)data_moved);
      int offset = (int)(dp_right->db_txt_start - dp_left->db_txt_start);
      dp_left->db_txt_start += (unsigned)data_moved;
      dp_left->db_free += (unsigned)total_moved;

      // update indexes in the new block
      for (int to = line_count_right, from = db_idx + 1;
           from < line_count_left; from++, to++) {
        dp_right->db_index[to] = dp->db_index[from] + (unsigned)offset;
      }
      line_count_right += lines_moved;
      line_count_left -= lines_moved;
    }

    // May move the new line into the left (old or new) block.
    if (in_left) {
      dp_left->db_txt_start -= (unsigned)len;
      dp_left->db_free -= (unsigned)len + (unsigned)INDEX_SIZE;
      dp_left->db_index[line_count_left] = dp_left->db_txt_start;
      if (flags & ML_APPEND_MARK) {
        dp_left->db_index[line_count_left] |= DB_MARKED;
      }
      memmove((char *)dp_left + dp_left->db_txt_start,
              line, (size_t)len);
      line_count_left++;
    }

    if (db_idx < 0) {           // left block is new
      lnum_left = lnum + 1;
      lnum_right = 0;
    } else {                  // right block is new
      lnum_left = 0;
      if (in_left) {
        lnum_right = lnum + 2;
      } else {
        lnum_right = lnum + 1;
      }
    }
    dp_left->db_line_count = line_count_left;
    dp_right->db_line_count = line_count_right;

    // release the two data blocks
    // The new one (hp_new) already has a correct blocknumber.
    // The old one (hp, in ml_locked) gets a positive blocknumber if
    // we changed it and we are not editing a new file.
    if (lines_moved || in_left) {
      buf->b_ml.ml_flags |= ML_LOCKED_DIRTY;
    }
    if (!(flags & ML_APPEND_NEW) && db_idx >= 0 && in_left) {
      buf->b_ml.ml_flags |= ML_LOCKED_POS;
    }
    mf_put(mfp, hp_new, true, false);

    // flush the old data block
    // set ml_locked_lineadd to 0, because the updating of the
    // pointer blocks is done below
    int lineadd = buf->b_ml.ml_locked_lineadd;
    buf->b_ml.ml_locked_lineadd = 0;
    ml_find_line(buf, 0, ML_FLUSH);  // flush data block

    // update pointer blocks for the new data block
    for (stack_idx = buf->b_ml.ml_stack_top - 1; stack_idx >= 0; stack_idx--) {
      infoptr_T *ip = &(buf->b_ml.ml_stack[stack_idx]);
      int pb_idx = ip->ip_index;
      if ((hp = mf_get(mfp, ip->ip_bnum, 1)) == NULL) {
        goto theend;
      }
      PointerBlock *pp = hp->bh_data;         // must be pointer block
      if (pp->pb_id != PTR_ID) {
        iemsg(_(e_pointer_block_id_wrong_three));
        mf_put(mfp, hp, false, false);
        goto theend;
      }
      // TODO(vim): If the pointer block is full and we are adding at the end
      // try to insert in front of the next block
      // block not full, add one entry
      if (pp->pb_count < pp->pb_count_max) {
        if (pb_idx + 1 < (int)pp->pb_count) {
          memmove(&pp->pb_pointer[pb_idx + 2],
                  &pp->pb_pointer[pb_idx + 1],
                  (size_t)(pp->pb_count - pb_idx - 1) * sizeof(PointerEntry));
        }
        pp->pb_count++;
        pp->pb_pointer[pb_idx].pe_line_count = line_count_left;
        pp->pb_pointer[pb_idx].pe_bnum = bnum_left;
        pp->pb_pointer[pb_idx].pe_page_count = page_count_left;
        pp->pb_pointer[pb_idx + 1].pe_line_count = line_count_right;
        pp->pb_pointer[pb_idx + 1].pe_bnum = bnum_right;
        pp->pb_pointer[pb_idx + 1].pe_page_count = page_count_right;

        if (lnum_left != 0) {
          pp->pb_pointer[pb_idx].pe_old_lnum = lnum_left;
        }
        if (lnum_right != 0) {
          pp->pb_pointer[pb_idx + 1].pe_old_lnum = lnum_right;
        }

        mf_put(mfp, hp, true, false);
        buf->b_ml.ml_stack_top = stack_idx + 1;             // truncate stack

        if (lineadd) {
          (buf->b_ml.ml_stack_top)--;
          // fix line count for rest of blocks in the stack
          ml_lineadd(buf, lineadd);
          // fix stack itself
          buf->b_ml.ml_stack[buf->b_ml.ml_stack_top].ip_high += lineadd;
          (buf->b_ml.ml_stack_top)++;
        }

        // We are finished, break the loop here.
        break;
      }
      // pointer block full
      //
      // split the pointer block
      // allocate a new pointer block
      // move some of the pointer into the new block
      // prepare for updating the parent block
      while (true) {          // do this twice when splitting block 1
        hp_new = ml_new_ptr(mfp);
        if (hp_new == NULL) {             // TODO(vim): try to fix tree
          goto theend;
        }
        pp_new = hp_new->bh_data;

        if (hp->bh_bnum != 1) {
          break;
        }

        // if block 1 becomes full the tree is given an extra level
        // The pointers from block 1 are moved into the new block.
        // block 1 is updated to point to the new block
        // then continue to split the new block
        memmove(pp_new, pp, (size_t)page_size);
        pp->pb_count = 1;
        pp->pb_pointer[0].pe_bnum = hp_new->bh_bnum;
        pp->pb_pointer[0].pe_line_count = buf->b_ml.ml_line_count;
        pp->pb_pointer[0].pe_old_lnum = 1;
        pp->pb_pointer[0].pe_page_count = 1;
        mf_put(mfp, hp, true, false);             // release block 1
        hp = hp_new;                          // new block is to be split
        pp = pp_new;
        CHECK(stack_idx != 0, _("stack_idx should be 0"));
        ip->ip_index = 0;
        stack_idx++;                  // do block 1 again later
      }
      // move the pointers after the current one to the new block
      // If there are none, the new entry will be in the new block.
      total_moved = pp->pb_count - pb_idx - 1;
      if (total_moved) {
        memmove(&pp_new->pb_pointer[0],
                &pp->pb_pointer[pb_idx + 1],
                (size_t)(total_moved) * sizeof(PointerEntry));
        pp_new->pb_count = (uint16_t)total_moved;
        pp->pb_count = (uint16_t)(pp->pb_count - (total_moved - 1));
        pp->pb_pointer[pb_idx + 1].pe_bnum = bnum_right;
        pp->pb_pointer[pb_idx + 1].pe_line_count = line_count_right;
        pp->pb_pointer[pb_idx + 1].pe_page_count = page_count_right;
        if (lnum_right) {
          pp->pb_pointer[pb_idx + 1].pe_old_lnum = lnum_right;
        }
      } else {
        pp_new->pb_count = 1;
        pp_new->pb_pointer[0].pe_bnum = bnum_right;
        pp_new->pb_pointer[0].pe_line_count = line_count_right;
        pp_new->pb_pointer[0].pe_page_count = page_count_right;
        pp_new->pb_pointer[0].pe_old_lnum = lnum_right;
      }
      pp->pb_pointer[pb_idx].pe_bnum = bnum_left;
      pp->pb_pointer[pb_idx].pe_line_count = line_count_left;
      pp->pb_pointer[pb_idx].pe_page_count = page_count_left;
      if (lnum_left) {
        pp->pb_pointer[pb_idx].pe_old_lnum = lnum_left;
      }
      lnum_left = 0;
      lnum_right = 0;

      // recompute line counts
      line_count_right = 0;
      for (int i = 0; i < (int)pp_new->pb_count; i++) {
        line_count_right += pp_new->pb_pointer[i].pe_line_count;
      }
      line_count_left = 0;
      for (int i = 0; i < (int)pp->pb_count; i++) {
        line_count_left += pp->pb_pointer[i].pe_line_count;
      }

      bnum_left = hp->bh_bnum;
      bnum_right = hp_new->bh_bnum;
      page_count_left = 1;
      page_count_right = 1;
      mf_put(mfp, hp, true, false);
      mf_put(mfp, hp_new, true, false);
    }

    // Safety check: fallen out of for loop?
    if (stack_idx < 0) {
      iemsg(_("E318: Updated too many blocks?"));
      buf->b_ml.ml_stack_top = 0;       // invalidate stack
    }
  }

  // The line was inserted below 'lnum'
  ml_updatechunk(buf, lnum + 1, len, ML_CHNK_ADDLINE);
  ret = OK;

theend:
  return ret;
}

/// Public wrapper around static ml_append_int, used by rs_ml_append_flush.
int nvim_ml_append_int(buf_T *buf, linenr_T lnum, char *line, colnr_T len, int flags)
  FUNC_ATTR_NONNULL_ARG(1)
{
  return ml_append_int(buf, lnum, line, len, flags);
}

/// Flush any pending change and call ml_append_int()
///
/// @param buf
/// @param lnum  append after this line (can be 0)
/// @param line  text of the new line
/// @param len  length of line, including NUL, or 0
/// @param flags  ML_APPEND_ flags
///
/// @return  FAIL for failure, OK otherwise
static int ml_append_flush(buf_T *buf, linenr_T lnum, char *line, colnr_T len, int flags)
  FUNC_ATTR_NONNULL_ARG(1)
{
  return rs_ml_append_flush(buf, lnum, line, len, flags);
}

/// Public wrapper around static ml_append_flush, used by Rust _impl functions.
int nvim_ml_append_flush(buf_T *buf, linenr_T lnum, char *line, colnr_T len, int flags)
  FUNC_ATTR_NONNULL_ARG(1)
{
  return ml_append_flush(buf, lnum, line, len, flags);
}

/// Append a line after lnum (may be 0 to insert a line in front of the file).
/// "line" does not need to be allocated, but can't be another line in a
/// buffer, unlocking may make it invalid.
///
/// "newfile": true when starting to edit a new file, meaning that pe_old_lnum
///              will be set for recovery
/// Check: The caller of this function should probably also call
/// appended_lines().
///
/// @param lnum  append after this line (can be 0)
/// @param line  text of the new line
/// @param len  length of new line, including NUL, or 0
/// @param newfile  flag, see above
///
/// @return  FAIL for failure, OK otherwise
int ml_append(linenr_T lnum, char *line, colnr_T len, bool newfile)
{
  return ml_append_flags(lnum, line, len, newfile ? ML_APPEND_NEW : 0);
}

/// @param lnum  append after this line (can be 0)
/// @param line  text of the new line
/// @param len  length of new line, including nul, or 0
/// @param flags  ML_APPEND_ values
///
/// @return  FAIL for failure, OK otherwise
int ml_append_flags(linenr_T lnum, char *line, colnr_T len, int flags)
{
  return rs_ml_append_flags_impl(lnum, line, len, flags);
}

/// Like ml_append() but for an arbitrary buffer.  The buffer must already have
/// a memline.
///
/// @param lnum  append after this line (can be 0)
/// @param line  text of the new line
/// @param len  length of new line, including NUL, or 0
/// @param newfile  flag, see above
int ml_append_buf(buf_T *buf, linenr_T lnum, char *line, colnr_T len, bool newfile)
  FUNC_ATTR_NONNULL_ARG(1)
{
  return rs_ml_append_buf_impl(buf, lnum, line, len, newfile);
}

/// Track deleted text length for the current buffer (thin wrapper calling Rust).
void ml_add_deleted_len(char *ptr, ssize_t len) { rs_ml_add_deleted_len(ptr, len); }

/// Track deleted text length for a specific buffer (thin wrapper calling Rust).
void ml_add_deleted_len_buf(buf_T *buf, char *ptr, ssize_t len)
{
  rs_ml_add_deleted_len_buf(buf, ptr, len);
}

/// Replace line "lnum", with buffering, in current buffer.
int ml_replace(linenr_T lnum, char *line, bool copy)
{
  return ml_replace_buf(curbuf, lnum, line, copy, false);
}

/// Replace a line for the current buffer.  Like ml_replace() with:
/// "len" is the length of the text, excluding NUL.
int ml_replace_len(linenr_T lnum, char *line, size_t len, bool copy)
{
  return ml_replace_buf_len(curbuf, lnum, line, len, copy, false);
}

int ml_replace_buf(buf_T *buf, linenr_T lnum, char *line, bool copy, bool noalloc)
  FUNC_ATTR_NONNULL_ARG(1)
{
  return rs_ml_replace_buf_impl(buf, lnum, line, copy, noalloc);
}

/// Replace line "lnum", with buffering.
///
/// @param copy  if true, make a copy of the line, otherwise the line has been
///              copied to allocated memory already.
///              if false, the "line" may be freed to add text properties!
/// @param len_arg  length of the text, excluding NUL
///
/// Do not use it after calling ml_replace().
///
/// Check: The caller of this function should probably also call
/// changed_lines(), unless update_screen(UPD_NOT_VALID) is used.
///
/// @return  FAIL for failure, OK otherwise
int ml_replace_buf_len(buf_T *buf, linenr_T lnum, char *line_arg, size_t len_arg, bool copy,
                       bool noalloc)
  FUNC_ATTR_NONNULL_ARG(1)
{
  return rs_ml_replace_buf_len(buf, lnum, line_arg, len_arg, copy, noalloc);
}

/// Delete line `lnum` in buffer
///
/// @note The caller of this function should probably also call changed_lines() after this.
///
/// @param message  Show "--No lines in buffer--" message.
///
/// @return  FAIL for failure, OK otherwise
int ml_delete_buf(buf_T *buf, linenr_T lnum, bool message)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_ml_delete_buf_impl(buf, lnum, message);
}

/// Delete line `lnum` in the current buffer.
///
/// @param flags  ML_DEL_MESSAGE may give a "No lines in buffer" message.
///               ML_DEL_UNDO this is called from undo.
///
/// @return  FAIL for failure, OK otherwise
static int ml_delete_int(buf_T *buf, linenr_T lnum, int flags)
  FUNC_ATTR_NONNULL_ALL
{
  rs_ml_adjust_lowest_marked_for_delete(lnum);

  // If the file becomes empty the last line is replaced by an empty line.
  if (buf->b_ml.ml_line_count == 1) {       // file becomes empty
    if (flags & ML_DEL_MESSAGE) {
      set_keep_msg(_(no_lines_msg), 0);
    }

    int i = ml_replace_buf(buf, 1, "", true, false);
    buf->b_ml.ml_flags |= ML_EMPTY;

    return i;
  }

  // Find the data block containing the line.
  // This also fills the stack with the blocks from the root to the data block.
  // This also releases any locked block.
  memfile_T *mfp = buf->b_ml.ml_mfp;
  if (mfp == NULL) {
    return FAIL;
  }

  bhdr_T *hp;
  if ((hp = ml_find_line(buf, lnum, ML_DELETE)) == NULL) {
    return FAIL;
  }

  DataBlock *dp = hp->bh_data;
  // compute line count (number of entries in block) before the delete
  int count = buf->b_ml.ml_locked_high - buf->b_ml.ml_locked_low + 2;
  int idx = lnum - buf->b_ml.ml_locked_low;

  if (buf->b_prev_line_count == 0) {
    buf->b_prev_line_count = buf->b_ml.ml_line_count;
  }
  buf->b_ml.ml_line_count--;

  int line_start = ((dp->db_index[idx]) & DB_INDEX_MASK);
  int line_size;
  if (idx == 0) {               // first line in block, text at the end
    line_size = (int)(dp->db_txt_end - (unsigned)line_start);
  } else {
    line_size = (int)(((dp->db_index[idx - 1]) & DB_INDEX_MASK) - (unsigned)line_start);
  }

  // Line should always have an NL char internally (represented as NUL),
  // even if 'noeol' is set.
  assert(line_size >= 1);
  ml_add_deleted_len_buf(buf, (char *)dp + line_start, line_size - 1);

  // special case: If there is only one line in the data block it becomes empty.
  // Then we have to remove the entry, pointing to this data block, from the
  // pointer block. If this pointer block also becomes empty, we go up another
  // block, and so on, up to the root if necessary.
  // The line counts in the pointer blocks have already been adjusted by
  // ml_find_line().
  int ret = FAIL;
  if (count == 1) {
    mf_free(mfp, hp);           // free the data block
    buf->b_ml.ml_locked = NULL;

    for (int stack_idx = buf->b_ml.ml_stack_top - 1; stack_idx >= 0; stack_idx--) {
      buf->b_ml.ml_stack_top = 0;           // stack is invalid when failing
      infoptr_T *ip = &(buf->b_ml.ml_stack[stack_idx]);
      idx = ip->ip_index;
      if ((hp = mf_get(mfp, ip->ip_bnum, 1)) == NULL) {
        goto theend;
      }
      PointerBlock *pp = hp->bh_data;         // must be pointer block
      if (pp->pb_id != PTR_ID) {
        iemsg(_(e_pointer_block_id_wrong_four));
        mf_put(mfp, hp, false, false);
        goto theend;
      }
      count = --(pp->pb_count);
      if (count == 0) {             // the pointer block becomes empty!
        mf_free(mfp, hp);
      } else {
        if (count != idx) {             // move entries after the deleted one
          memmove(&pp->pb_pointer[idx], &pp->pb_pointer[idx + 1],
                  (size_t)(count - idx) * sizeof(PointerEntry));
        }
        mf_put(mfp, hp, true, false);

        buf->b_ml.ml_stack_top = stack_idx;             // truncate stack
        // fix line count for rest of blocks in the stack
        if (buf->b_ml.ml_locked_lineadd != 0) {
          ml_lineadd(buf, buf->b_ml.ml_locked_lineadd);
          buf->b_ml.ml_stack[buf->b_ml.ml_stack_top].ip_high +=
            buf->b_ml.ml_locked_lineadd;
        }
        (buf->b_ml.ml_stack_top)++;

        break;
      }
    }
    CHECK(stack_idx < 0, _("deleted block 1?"));
  } else {
    // delete the text by moving the next lines forwards
    int text_start = (int)dp->db_txt_start;
    memmove((char *)dp + text_start + line_size,
            (char *)dp + text_start, (size_t)(line_start - text_start));

    // delete the index by moving the next indexes backwards
    // Adjust the indexes for the text movement.
    for (int i = idx; i < count - 1; i++) {
      dp->db_index[i] = dp->db_index[i + 1] + (unsigned)line_size;
    }

    dp->db_free += (unsigned)line_size + (unsigned)INDEX_SIZE;
    dp->db_txt_start += (unsigned)line_size;
    dp->db_line_count--;

    // mark the block dirty and make sure it is in the file (for recovery)
    buf->b_ml.ml_flags |= (ML_LOCKED_DIRTY | ML_LOCKED_POS);
  }

  ml_updatechunk(buf, lnum, line_size, ML_CHNK_DELLINE);
  ret = OK;

theend:
  return ret;
}

/// Public wrapper around static ml_delete_int, used by Rust _impl functions.
int nvim_ml_delete_int(buf_T *buf, linenr_T lnum, int flags)
  FUNC_ATTR_NONNULL_ALL
{
  return ml_delete_int(buf, lnum, flags);
}

/// Delete line "lnum" in the current buffer.
///
/// @note The caller of this function should probably also call
/// deleted_lines() after this.
///
/// @return  FAIL for failure, OK otherwise
int ml_delete(linenr_T lnum)
{
  return ml_delete_flags(lnum, 0);
}

/// Like ml_delete() but using flags (see ml_delete_int()).
///
/// @return  FAIL for failure, OK otherwise
int ml_delete_flags(linenr_T lnum, int flags)
{
  return rs_ml_delete_flags_impl(lnum, flags);
}

/// set the DB_MARKED flag for line 'lnum' (thin wrapper calling Rust)
void ml_setmarked(linenr_T lnum) { rs_ml_setmarked(lnum); }

/// find the first line with its DB_MARKED flag set (thin wrapper calling Rust)
linenr_T ml_firstmarked(void) { return rs_ml_firstmarked(); }

/// clear all DB_MARKED flags (thin wrapper calling Rust)
void ml_clearmarked(void) { rs_ml_clearmarked(); }

/// Flush deleted byte tracking counters (thin wrapper calling Rust).
size_t ml_flush_deleted_bytes(buf_T *buf, size_t *codepoints, size_t *codeunits)
{
  return rs_ml_flush_deleted_bytes(buf, codepoints, codeunits);
}

/// flush ml_line if necessary
void ml_flush_line(buf_T *buf, bool noalloc)
{
  static bool entered = false;

  if (buf->b_ml.ml_line_lnum == 0 || buf->b_ml.ml_mfp == NULL) {
    return;             // nothing to do
  }
  if (buf->b_ml.ml_flags & ML_LINE_DIRTY) {
    // This code doesn't work recursively.
    if (entered) {
      return;
    }
    entered = true;

    buf->flush_count++;

    linenr_T lnum = buf->b_ml.ml_line_lnum;
    char *new_line = buf->b_ml.ml_line_ptr;

    bhdr_T *hp = ml_find_line(buf, lnum, ML_FIND);
    if (hp == NULL) {
      siemsg(_("E320: Cannot find line %" PRId64), (int64_t)lnum);
    } else {
      DataBlock *dp = hp->bh_data;
      int idx = lnum - buf->b_ml.ml_locked_low;
      int start = ((dp->db_index[idx]) & DB_INDEX_MASK);
      char *old_line = (char *)dp + start;
      int old_len;
      if (idx == 0) {           // line is last in block
        old_len = (int)dp->db_txt_end - start;
      } else {  // text of previous line follows
        old_len = (int)(dp->db_index[idx - 1] & DB_INDEX_MASK) - start;
      }
      colnr_T new_len = buf->b_ml.ml_line_len;
      int extra = new_len - old_len;            // negative if lines gets smaller

      // if new line fits in data block, replace directly
      if ((int)dp->db_free >= extra) {
        // if the length changes and there are following lines
        int count = buf->b_ml.ml_locked_high - buf->b_ml.ml_locked_low + 1;
        if (extra != 0 && idx < count - 1) {
          // move text of following lines
          memmove((char *)dp + dp->db_txt_start - extra,
                  (char *)dp + dp->db_txt_start,
                  (size_t)(start - (int)dp->db_txt_start));

          // adjust pointers of this and following lines
          for (int i = idx + 1; i < count; i++) {
            dp->db_index[i] -= (unsigned)extra;
          }
        }
        dp->db_index[idx] -= (unsigned)extra;

        // adjust free space
        dp->db_free -= (unsigned)extra;
        dp->db_txt_start -= (unsigned)extra;

        // copy new line into the data block
        memmove(old_line - extra, new_line, (size_t)new_len);
        buf->b_ml.ml_flags |= (ML_LOCKED_DIRTY | ML_LOCKED_POS);
        // The else case is already covered by the insert and delete
        if (extra != 0) {
          ml_updatechunk(buf, lnum, extra, ML_CHNK_UPDLINE);
        }
      } else {
        // Cannot do it in one data block: Delete and append.
        // Append first, because ml_delete_int() cannot delete the
        // last line in a buffer, which causes trouble for a buffer
        // that has only one line.
        // Don't forget to copy the mark!
        // How about handling errors???
        (void)ml_append_int(buf, lnum, new_line, new_len,
                            (dp->db_index[idx] & DB_MARKED) ? ML_APPEND_MARK : 0);
        (void)ml_delete_int(buf, lnum, 0);
      }
    }
    if (!noalloc) {
      xfree(new_line);
    }

    entered = false;
  } else if (buf->b_ml.ml_flags & ML_ALLOCATED) {
    assert(!noalloc);  // caller must set ML_LINE_DIRTY with noalloc, handled above
    xfree(buf->b_ml.ml_line_ptr);
  }

  buf->b_ml.ml_flags &= ~(ML_LINE_DIRTY | ML_ALLOCATED);
  buf->b_ml.ml_line_lnum = 0;
  buf->b_ml.ml_line_offset = 0;
}

/// create a new, empty, data block
static bhdr_T *ml_new_data(memfile_T *mfp, bool negative, int page_count)
{
  return rs_ml_new_data(mfp, negative, page_count);
}

/// create a new, empty, pointer block
static bhdr_T *ml_new_ptr(memfile_T *mfp)
{
  return rs_ml_new_ptr(mfp);
}

/// Lookup line 'lnum' in a memline.
///
/// @param action: if ML_DELETE or ML_INSERT the line count is updated while searching
///                if ML_FLUSH only flush a locked block
///                if ML_FIND just find the line
///
/// If the block was found it is locked and put in ml_locked.
/// The stack is updated to lead to the locked block. The ip_high field in
/// the stack is updated to reflect the last line in the block AFTER the
/// insert or delete, also if the pointer block has not been updated yet. But
/// if ml_locked != NULL ml_locked_lineadd must be added to ip_high.
///
/// @return  NULL for failure, pointer to block header otherwise
static bhdr_T *ml_find_line(buf_T *buf, linenr_T lnum, int action)
{
  bhdr_T *hp;
  int top;

  memfile_T *mfp = buf->b_ml.ml_mfp;

  // If there is a locked block check if the wanted line is in it.
  // If not, flush and release the locked block.
  // Don't do this for ML_INSERT_SAME, because the stack need to be updated.
  // Don't do this for ML_FLUSH, because we want to flush the locked block.
  // Don't do this when 'swapfile' is reset, we want to load all the blocks.
  if (buf->b_ml.ml_locked) {
    if (ML_SIMPLE(action)
        && buf->b_ml.ml_locked_low <= lnum
        && buf->b_ml.ml_locked_high >= lnum) {
      // remember to update pointer blocks and stack later
      if (action == ML_INSERT) {
        (buf->b_ml.ml_locked_lineadd)++;
        (buf->b_ml.ml_locked_high)++;
      } else if (action == ML_DELETE) {
        (buf->b_ml.ml_locked_lineadd)--;
        (buf->b_ml.ml_locked_high)--;
      }
      return buf->b_ml.ml_locked;
    }

    mf_put(mfp, buf->b_ml.ml_locked, buf->b_ml.ml_flags & ML_LOCKED_DIRTY,
           buf->b_ml.ml_flags & ML_LOCKED_POS);
    buf->b_ml.ml_locked = NULL;

    // If lines have been added or deleted in the locked block, need to
    // update the line count in pointer blocks.
    if (buf->b_ml.ml_locked_lineadd != 0) {
      ml_lineadd(buf, buf->b_ml.ml_locked_lineadd);
    }
  }

  if (action == ML_FLUSH) {         // nothing else to do
    return NULL;
  }

  blocknr_T bnum = 1;                         // start at the root of the tree
  blocknr_T bnum2;
  int page_count = 1;
  linenr_T low = 1;
  linenr_T high = buf->b_ml.ml_line_count;

  if (action == ML_FIND) {      // first try stack entries
    for (top = buf->b_ml.ml_stack_top - 1; top >= 0; top--) {
      infoptr_T *ip = &(buf->b_ml.ml_stack[top]);
      if (ip->ip_low <= lnum && ip->ip_high >= lnum) {
        bnum = ip->ip_bnum;
        low = ip->ip_low;
        high = ip->ip_high;
        buf->b_ml.ml_stack_top = top;           // truncate stack at prev entry
        break;
      }
    }
    if (top < 0) {
      buf->b_ml.ml_stack_top = 0;               // not found, start at the root
    }
  } else {  // ML_DELETE or ML_INSERT
    buf->b_ml.ml_stack_top = 0;         // start at the root
  }
  // search downwards in the tree until a data block is found
  while (true) {
    if ((hp = mf_get(mfp, bnum, (unsigned)page_count)) == NULL) {
      goto error_noblock;
    }

    // update high for insert/delete
    if (action == ML_INSERT) {
      high++;
    } else if (action == ML_DELETE) {
      high--;
    }

    DataBlock *dp = hp->bh_data;
    if (dp->db_id == DATA_ID) {         // data block
      buf->b_ml.ml_locked = hp;
      buf->b_ml.ml_locked_low = low;
      buf->b_ml.ml_locked_high = high;
      buf->b_ml.ml_locked_lineadd = 0;
      buf->b_ml.ml_flags &= ~(ML_LOCKED_DIRTY | ML_LOCKED_POS);
      return hp;
    }

    PointerBlock *pp = (PointerBlock *)(dp);                // must be pointer block
    if (pp->pb_id != PTR_ID) {
      iemsg(_(e_pointer_block_id_wrong));
      goto error_block;
    }

    top = ml_add_stack(buf);  // add new entry to stack
    infoptr_T *ip = &(buf->b_ml.ml_stack[top]);
    ip->ip_bnum = bnum;
    ip->ip_low = low;
    ip->ip_high = high;
    ip->ip_index = -1;                  // index not known yet

    bool dirty = false;
    int idx;
    for (idx = 0; idx < (int)pp->pb_count; idx++) {
      linenr_T t = pp->pb_pointer[idx].pe_line_count;
      CHECK(t == 0, _("pe_line_count is zero"));
      if ((low += t) > lnum) {
        ip->ip_index = idx;
        bnum = pp->pb_pointer[idx].pe_bnum;
        page_count = pp->pb_pointer[idx].pe_page_count;
        high = low - 1;
        low -= t;

        // a negative block number may have been changed
        if (bnum < 0) {
          bnum2 = mf_trans_del(mfp, bnum);
          if (bnum != bnum2) {
            bnum = bnum2;
            pp->pb_pointer[idx].pe_bnum = bnum;
            dirty = true;
          }
        }

        break;
      }
    }
    if (idx >= (int)pp->pb_count) {         // past the end: something wrong!
      if (lnum > buf->b_ml.ml_line_count) {
        siemsg(_(e_line_number_out_of_range_nr_past_the_end),
               (int64_t)lnum - buf->b_ml.ml_line_count);
      } else {
        siemsg(_(e_line_count_wrong_in_block_nr), bnum);
      }
      goto error_block;
    }
    if (action == ML_DELETE) {
      pp->pb_pointer[idx].pe_line_count--;
      dirty = true;
    } else if (action == ML_INSERT) {
      pp->pb_pointer[idx].pe_line_count++;
      dirty = true;
    }
    mf_put(mfp, hp, dirty, false);
  }

error_block:
  mf_put(mfp, hp, false, false);
error_noblock:
  // If action is ML_DELETE or ML_INSERT we have to correct the tree for
  // the incremented/decremented line counts, because there won't be a line
  // inserted/deleted after all.
  if (action == ML_DELETE) {
    ml_lineadd(buf, 1);
  } else if (action == ML_INSERT) {
    ml_lineadd(buf, -1);
  }
  buf->b_ml.ml_stack_top = 0;
  return NULL;
}

/// Public wrapper around the static ml_find_line, for Rust FFI.
bhdr_T *nvim_ml_find_line(buf_T *buf, linenr_T lnum, int action)
{
  return ml_find_line(buf, lnum, action);
}

/// add an entry to the info pointer stack
///
/// @return  number of the new entry
/// Add entry to B-tree info pointer stack (thin wrapper calling Rust).
static int ml_add_stack(buf_T *buf) { return rs_ml_add_stack(buf); }

/// Update the pointer blocks on the stack for inserted/deleted lines.
/// The stack itself is also updated.
///
/// When an insert/delete line action fails, the line is not inserted/deleted,
/// but the pointer blocks have already been updated. That is fixed here by
/// walking through the stack.
///
/// Count is the number of lines added, negative if lines have been deleted.
static void ml_lineadd(buf_T *buf, int count)
{
  rs_ml_lineadd(buf, count);
}

#if defined(HAVE_READLINK)
/// Resolve a symlink in the last component of a file name. (thin wrapper calling Rust)
int resolve_symlink(const char *fname, char *buf) { return rs_resolve_symlink(fname, buf); }
#endif

/// Make swapfile name out of the file name and a directory name. (thin wrapper calling Rust)
///
/// @return  pointer to allocated memory or NULL.
char *makeswapname(char *fname, char *ffname, buf_T *buf, char *dir_name)
{
  return rs_makeswapname(fname, ffname, buf, dir_name);
}

/// Get file name to use for swapfile or backup file. (thin wrapper calling Rust)
char *get_file_in_dir(char *fname, char *dname)
{
  return rs_get_file_in_dir(fname, dname);
}

/// Build the ATTENTION message: info about an existing swapfile.
///
/// @param buf  buffer being edited
/// @param fname  swapfile name
/// @param fhname  swapfile name, home replaced
/// @param msg  string buffer, emitted as either a regular or confirm message
static void attention_message(buf_T *buf, char *fname, char *fhname, StringBuilder *msg)
{
  assert(buf->b_fname != NULL);

  emsg(_("E325: ATTENTION"));
  kv_printf(*msg, _("Found a swap file by the name \""));
  kv_printf(*msg, "%s\"\n", fhname);
  const time_t swap_mtime = swapfile_info(fname, msg);
  kv_printf(*msg, (_("While opening file \"")));
  kv_printf(*msg, "%s\"\n", buf->b_fname);
  FileInfo file_info;
  if (!os_fileinfo(buf->b_fname, &file_info)) {
    kv_printf(*msg, _("      CANNOT BE FOUND"));
  } else {
    kv_printf(*msg, _("             dated: "));
    time_t x = file_info.stat.st_mtim.tv_sec;
    char ctime_buf[50];
    kv_printf(*msg, "%s", os_ctime_r(&x, ctime_buf, sizeof(ctime_buf), true));
    if (swap_mtime != 0 && x > swap_mtime) {
      kv_printf(*msg, _("      NEWER than swap file!\n"));
    }
  }
  // Some of these messages are long to allow translation to
  // other languages.
  kv_printf(*msg, _("\n(1) Another program may be editing the same file.  If this is"
                    " the case,\n    be careful not to end up with two different"
                    " instances of the same\n    file when making changes."
                    "  Quit, or continue with caution.\n"));
  kv_printf(*msg, _("(2) An edit session for this file crashed.\n"));
  kv_printf(*msg, _("    If this is the case, use \":recover\" or \"nvim -r "));
  kv_printf(*msg, "%s", buf->b_fname);
  kv_printf(*msg, (_("\"\n    to recover the changes (see \":help recovery\").\n")));
  kv_printf(*msg, _("    If you did this already, delete the swap file \""));
  kv_printf(*msg, "%s", fname);
  kv_printf(*msg, _("\"\n    to avoid this message.\n"));
}

/// Trigger the SwapExists autocommands.
///
/// @return  a value for equivalent to do_dialog().
static sea_choice_T do_swapexists(buf_T *buf, char *fname)
{
  set_vim_var_string(VV_SWAPNAME, fname, -1);
  set_vim_var_string(VV_SWAPCHOICE, NULL, -1);

  // Trigger SwapExists autocommands with <afile> set to the file being
  // edited.  Disallow changing directory here.
  allbuf_lock++;
  apply_autocmds(EVENT_SWAPEXISTS, buf->b_fname, NULL, false, NULL);
  allbuf_lock--;

  set_vim_var_string(VV_SWAPNAME, NULL, -1);

  switch (*get_vim_var_str(VV_SWAPCHOICE)) {
  case 'o':
    return SEA_CHOICE_READONLY;
  case 'e':
    return SEA_CHOICE_EDIT;
  case 'r':
    return SEA_CHOICE_RECOVER;
  case 'd':
    return SEA_CHOICE_DELETE;
  case 'q':
    return SEA_CHOICE_QUIT;
  case 'a':
    return SEA_CHOICE_ABORT;
  }

  return SEA_CHOICE_NONE;
}

/// Find out what name to use for the swapfile for buffer 'buf'.
///
/// Several names are tried to find one that does not exist. Last directory in
/// option is automatically created.
///
/// @note If BASENAMELEN is not correct, you will get error messages for
///   not being able to open the swap or undo file.
/// @note May trigger SwapExists autocmd, pointers may change!
///
/// @param[in]  buf  Buffer for which swapfile names needs to be found.
/// @param[in,out]  dirp  Pointer to a list of directories. When out of memory,
///                       is set to NULL. Is advanced to the next directory in
///                       the list otherwise.
/// @param[in]  old_fname  Allowed existing swapfile name. Except for this
///                        case, name of the non-existing file is used.
/// @param[in,out]  found_existing_dir  If points to true, then new directory
///                                     for swapfile is not created. At first
///                                     findswapname() call this argument must
///                                     point to false. This parameter may only
///                                     be set to true by this function, it is
///                                     never set to false.
///
/// @return [allocated] Name of the swapfile.
static char *findswapname(buf_T *buf, char **dirp, char *old_fname, bool *found_existing_dir)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ARG(1, 2, 4)
{
  char *buf_fname = buf->b_fname;

  // Isolate a directory name from *dirp and put it in dir_name.
  // First allocate some memory to put the directory name in.
  const size_t dir_len = strlen(*dirp) + 1;
  char *dir_name = xmalloc(dir_len);
  copy_option_part(dirp, dir_name, dir_len, ",");

  // We try different swapfile names until we find one that does not exist yet.
  char *fname = makeswapname(buf_fname, buf->b_ffname, buf, dir_name);

  while (true) {
    size_t n;
    if (fname == NULL) {        // must be out of memory
      break;
    }
    if ((n = strlen(fname)) == 0) {        // safety check
      XFREE_CLEAR(fname);
      break;
    }
    // check if the swapfile already exists
    // Extra security check: When a swapfile is a symbolic link, this
    // is most likely a symlink attack.
    FileInfo file_info;
    bool file_or_link_found = os_fileinfo_link(fname, &file_info);
    if (!file_or_link_found) {
      break;
    }

    // A file name equal to old_fname is OK to use.
    if (old_fname != NULL && path_fnamecmp(fname, old_fname) == 0) {
      break;
    }

    // get here when file already exists
    if (fname[n - 2] == 'w' && fname[n - 1] == 'p') {   // first try
      // If we get here the ".swp" file really exists.
      // Give an error message, unless recovering, no file name, we are
      // viewing a help file or when the path of the file is different
      // (happens when all .swp files are in one directory).
      if (!recoverymode && buf_fname != NULL && !buf->b_help && !(buf->b_flags & BF_DUMMY)) {
        int fd;
        ZeroBlock b0;
        bool differ = false;

        // Try to read block 0 from the swapfile to get the original file name (and inode number).
        fd = os_open(fname, O_RDONLY, 0);
        if (fd >= 0) {
          if (read_eintr(fd, &b0, sizeof(b0)) == sizeof(b0)) {
            proc_running = rs_swapfile_proc_running(&b0, fname);

            // If the swapfile has the same directory as the
            // buffer don't compare the directory names, they can
            // have a different mountpoint.
            if (b0.b0_flags & B0_SAME_DIR) {
              if (path_fnamecmp(path_tail(buf->b_ffname),
                                path_tail(b0.b0_fname)) != 0
                  || !same_directory(fname, buf->b_ffname)) {
                // Symlinks may point to the same file even
                // when the name differs, need to check the
                // inode too.
                expand_env(b0.b0_fname, NameBuff, MAXPATHL);
                if (rs_fnamecmp_ino(buf->b_ffname, NameBuff,
                                    rs_char_to_long(b0.b0_ino))) {
                  differ = true;
                }
              }
            } else {
              // The name in the swapfile may be "~user/path/file".  Expand it first.
              expand_env(b0.b0_fname, NameBuff, MAXPATHL);
              if (rs_fnamecmp_ino(buf->b_ffname, NameBuff,
                                  rs_char_to_long(b0.b0_ino))) {
                differ = true;
              }
            }
          }
          close(fd);
        }

        // Show the ATTENTION message when:
        //  - there is an old swapfile for the current file
        //  - the buffer was not recovered
        if (!differ && !(curbuf->b_flags & BF_RECOVERED)
            && vim_strchr(p_shm, SHM_ATTENTION) == NULL) {
          sea_choice_T choice = SEA_CHOICE_NONE;

          // It's safe to delete the swapfile if all these are true:
          // - the edited file exists
          // - the swapfile has no changes and looks OK
          if (os_path_exists(buf->b_fname) && rs_swapfile_unchanged(fname)) {
            choice = SEA_CHOICE_DELETE;
            if (p_verbose > 0) {
              verb_msg(_("Found a swap file that is not useful, deleting it"));
            }
          }

          // If there is a SwapExists autocommand and we can handle the
          // response, trigger it.  It may return 0 to ask the user anyway.
          if (choice == SEA_CHOICE_NONE
              && swap_exists_action != SEA_NONE
              && has_autocmd(EVENT_SWAPEXISTS, buf_fname, buf)) {
            choice = do_swapexists(buf, fname);
          }

          if (choice == SEA_CHOICE_NONE && swap_exists_action == SEA_READONLY) {
            // always open readonly.
            choice = SEA_CHOICE_READONLY;
          }

          proc_running = 0;  // Set by attention_message..swapfile_info.
          if (choice == SEA_CHOICE_NONE) {
            no_wait_return++;
            // Show info about the existing swapfile.
            StringBuilder msg = KV_INITIAL_VALUE;
            kv_resize(msg, IOSIZE);
            char *fhname = home_replace_save(NULL, fname);
            attention_message(buf, fname, fhname, &msg);

            // We don't want a 'q' typed at the more-prompt
            // interrupt loading a file.
            got_int = false;

            // If vimrc has "simalt ~x" we don't want it to
            // interfere with the prompt here.
            flush_buffers(FLUSH_TYPEAHEAD);

            if (swap_exists_action != SEA_NONE) {
              kv_printf(msg, _("Swap file \""));
              kv_printf(msg, "%s", fhname);
              kv_printf(msg, _("\" already exists!"));
              char *run_but = _("&Open Read-Only\n&Edit anyway\n&Recover\n&Quit\n&Abort");
              char *but = _("&Open Read-Only\n&Edit anyway\n&Recover\n&Delete it\n&Quit\n&Abort");
              choice = (sea_choice_T)do_dialog(VIM_WARNING, _("VIM - ATTENTION"), msg.items,
                                               proc_running ? run_but : but, 1, NULL, false);

              // compensate for missing "Delete it" button
              choice += proc_running && choice >= 4;
              // pretend screen didn't scroll, need redraw anyway
              msg_reset_scroll();
            } else {
              bool need_clear = false;
              msg_multiline(cbuf_as_string(msg.items, msg.size), 0, false, false, &need_clear);
            }
            no_wait_return--;
            kv_destroy(msg);
            xfree(fhname);
          }

          switch (choice) {
          case SEA_CHOICE_READONLY:  // "Open Read-Only"
            buf->b_p_ro = true;
            break;
          case SEA_CHOICE_EDIT:  // "Edit anyway"
            break;
          case SEA_CHOICE_RECOVER:  // "Recover"
            swap_exists_action = SEA_RECOVER;
            break;
          case SEA_CHOICE_DELETE:  // "Delete it"
            os_remove(fname);
            break;
          case SEA_CHOICE_QUIT:  // "Quit"
            swap_exists_action = SEA_QUIT;
            break;
          case SEA_CHOICE_ABORT:  // "Abort"
            swap_exists_action = SEA_QUIT;
            got_int = true;
            break;
          case SEA_CHOICE_NONE:
            msg_puts("\n");
            if (msg_silent == 0) {
              // call wait_return() later
              need_wait_return = true;
            }
            break;
          }

          // If the swapfile was deleted this `fname` can be used.
          if (choice != SEA_CHOICE_NONE && !os_path_exists(fname)) {
            break;
          }
        }
      }
    }

    // Permute the ".swp" extension to find a unique swapfile name.
    // First decrement the last char: ".swo", ".swn", etc.
    // If that still isn't enough decrement the last but one char: ".svz"
    // Can happen when many Nvim instances are editing the same file (including "No Name" buffers).
    if (fname[n - 1] == 'a') {          // ".s?a"
      if (fname[n - 2] == 'a') {        // ".saa": tried enough, give up
        emsg(_("E326: Too many swap files found"));
        XFREE_CLEAR(fname);
        break;
      }
      fname[n - 2]--;                   // ".svz", ".suz", etc.
      fname[n - 1] = 'z' + 1;
    }
    fname[n - 1]--;                     // ".swo", ".swn", etc.
  }

  if (os_isdir(dir_name)) {
    *found_existing_dir = true;
  } else if (!*found_existing_dir && **dirp == NUL) {
    int ret;
    char *failed_dir;
    if ((ret = os_mkdir_recurse(dir_name, 0755, &failed_dir, NULL)) != 0) {
      semsg(_("E303: Unable to create directory \"%s\" for swap file, "
              "recovery impossible: %s"),
            failed_dir, os_strerror(ret));
      xfree(failed_dir);
    }
  }

  xfree(dir_name);
  return fname;
}

/// Set the flags in the first block of the swapfile. (thin wrapper calling Rust)
void ml_setflags(buf_T *buf) { rs_ml_setflags(buf); }

enum {
  MLCS_MAXL = 800,  // max no of lines in chunk
  MLCS_MINL = 400,  // should be half of MLCS_MAXL
};

/// Keep information for finding byte offset of a line
///
/// @param updtype  may be one of:
///                 ML_CHNK_ADDLINE: Add len to parent chunk, possibly splitting it
///                         Careful: ML_CHNK_ADDLINE may cause ml_find_line() to be called.
///                 ML_CHNK_DELLINE: Subtract len from parent chunk, possibly deleting it
///                 ML_CHNK_UPDLINE: Add len to parent chunk, as a signed entity.
static void ml_updatechunk(buf_T *buf, linenr_T line, int len, int updtype)
{
  static buf_T *ml_upd_lastbuf = NULL;
  static linenr_T ml_upd_lastline;
  static linenr_T ml_upd_lastcurline;
  static int ml_upd_lastcurix;

  linenr_T curline = ml_upd_lastcurline;
  int curix = ml_upd_lastcurix;
  bhdr_T *hp;

  if (buf->b_ml.ml_usedchunks == -1 || len == 0) {
    return;
  }
  if (buf->b_ml.ml_chunksize == NULL) {
    buf->b_ml.ml_chunksize = xmalloc(sizeof(chunksize_T) * 100);
    buf->b_ml.ml_numchunks = 100;
    buf->b_ml.ml_usedchunks = 1;
    buf->b_ml.ml_chunksize[0].mlcs_numlines = 1;
    buf->b_ml.ml_chunksize[0].mlcs_totalsize = 1;
  }

  if (updtype == ML_CHNK_UPDLINE && buf->b_ml.ml_line_count == 1) {
    // First line in empty buffer from ml_flush_line() -- reset
    buf->b_ml.ml_usedchunks = 1;
    buf->b_ml.ml_chunksize[0].mlcs_numlines = 1;
    buf->b_ml.ml_chunksize[0].mlcs_totalsize = buf->b_ml.ml_line_len;
    return;
  }

  // Find chunk that our line belongs to, curline will be at start of the
  // chunk.
  if (buf != ml_upd_lastbuf || line != ml_upd_lastline + 1
      || updtype != ML_CHNK_ADDLINE) {
    for (curline = 1, curix = 0;
         curix < buf->b_ml.ml_usedchunks - 1
         && line >= curline +
         buf->b_ml.ml_chunksize[curix].mlcs_numlines;
         curix++) {
      curline += buf->b_ml.ml_chunksize[curix].mlcs_numlines;
    }
  } else if (curix < buf->b_ml.ml_usedchunks - 1
             && line >= curline + buf->b_ml.ml_chunksize[curix].mlcs_numlines) {
    // Adjust cached curix & curline
    curline += buf->b_ml.ml_chunksize[curix].mlcs_numlines;
    curix++;
  }
  chunksize_T *curchnk = buf->b_ml.ml_chunksize + curix;

  if (updtype == ML_CHNK_DELLINE) {
    len = -len;
  }
  curchnk->mlcs_totalsize += len;
  if (updtype == ML_CHNK_ADDLINE) {
    int rest;
    DataBlock *dp;
    curchnk->mlcs_numlines++;

    // May resize here so we don't have to do it in both cases below
    if (buf->b_ml.ml_usedchunks + 1 >= buf->b_ml.ml_numchunks) {
      buf->b_ml.ml_numchunks = buf->b_ml.ml_numchunks * 3 / 2;
      buf->b_ml.ml_chunksize = xrealloc(buf->b_ml.ml_chunksize,
                                        sizeof(chunksize_T) * (size_t)buf->b_ml.ml_numchunks);
    }

    if (buf->b_ml.ml_chunksize[curix].mlcs_numlines >= MLCS_MAXL) {
      int end_idx;
      int text_end;

      memmove(buf->b_ml.ml_chunksize + curix + 1,
              buf->b_ml.ml_chunksize + curix,
              (size_t)(buf->b_ml.ml_usedchunks - curix) * sizeof(chunksize_T));
      // Compute length of first half of lines in the split chunk
      int size = 0;
      int linecnt = 0;
      while (curline < buf->b_ml.ml_line_count
             && linecnt < MLCS_MINL) {
        if ((hp = ml_find_line(buf, curline, ML_FIND)) == NULL) {
          buf->b_ml.ml_usedchunks = -1;
          return;
        }
        dp = hp->bh_data;
        int count
          = buf->b_ml.ml_locked_high - buf->b_ml.ml_locked_low + 1;  // number of entries in block
        int idx = curline - buf->b_ml.ml_locked_low;
        curline = buf->b_ml.ml_locked_high + 1;
        // compute index of last line to use in this MEMLINE
        rest = count - idx;
        if (linecnt + rest > MLCS_MINL) {
          end_idx = idx + MLCS_MINL - linecnt - 1;
          linecnt = MLCS_MINL;
        } else {
          end_idx = count - 1;
          linecnt += rest;
        }
        if (idx == 0) {  // first line in block, text at the end
          text_end = (int)dp->db_txt_end;
        } else {
          text_end = ((dp->db_index[idx - 1]) & DB_INDEX_MASK);
        }
        size += text_end - (int)((dp->db_index[end_idx]) & DB_INDEX_MASK);
      }
      buf->b_ml.ml_chunksize[curix].mlcs_numlines = linecnt;
      buf->b_ml.ml_chunksize[curix + 1].mlcs_numlines -= linecnt;
      buf->b_ml.ml_chunksize[curix].mlcs_totalsize = size;
      buf->b_ml.ml_chunksize[curix + 1].mlcs_totalsize -= size;
      buf->b_ml.ml_usedchunks++;
      ml_upd_lastbuf = NULL;         // Force recalc of curix & curline
      return;
    } else if (buf->b_ml.ml_chunksize[curix].mlcs_numlines >= MLCS_MINL
               && curix == buf->b_ml.ml_usedchunks - 1
               && buf->b_ml.ml_line_count - line <= 1) {
      // We are in the last chunk and it is cheap to create a new one
      // after this. Do it now to avoid the loop above later on
      curchnk = buf->b_ml.ml_chunksize + curix + 1;
      buf->b_ml.ml_usedchunks++;
      if (line == buf->b_ml.ml_line_count) {
        curchnk->mlcs_numlines = 0;
        curchnk->mlcs_totalsize = 0;
      } else {
        // Line is just prior to last, move count for last
        // This is the common case  when loading a new file
        hp = ml_find_line(buf, buf->b_ml.ml_line_count, ML_FIND);
        if (hp == NULL) {
          buf->b_ml.ml_usedchunks = -1;
          return;
        }
        dp = hp->bh_data;
        if (dp->db_line_count == 1) {
          rest = (int)(dp->db_txt_end - dp->db_txt_start);
        } else {
          rest = (int)((dp->db_index[dp->db_line_count - 2]) & DB_INDEX_MASK)
                 - (int)dp->db_txt_start;
        }
        curchnk->mlcs_totalsize = rest;
        curchnk->mlcs_numlines = 1;
        curchnk[-1].mlcs_totalsize -= rest;
        curchnk[-1].mlcs_numlines -= 1;
      }
    }
  } else if (updtype == ML_CHNK_DELLINE) {
    curchnk->mlcs_numlines--;
    ml_upd_lastbuf = NULL;       // Force recalc of curix & curline
    if (curix < (buf->b_ml.ml_usedchunks - 1)
        && (curchnk->mlcs_numlines + curchnk[1].mlcs_numlines)
        <= MLCS_MINL) {
      curix++;
      curchnk = buf->b_ml.ml_chunksize + curix;
    } else if (curix == 0 && curchnk->mlcs_numlines <= 0) {
      buf->b_ml.ml_usedchunks--;
      memmove(buf->b_ml.ml_chunksize, buf->b_ml.ml_chunksize + 1,
              (size_t)buf->b_ml.ml_usedchunks * sizeof(chunksize_T));
      return;
    } else if (curix == 0 || (curchnk->mlcs_numlines > 10
                              && (curchnk->mlcs_numlines +
                                  curchnk[-1].mlcs_numlines)
                              > MLCS_MINL)) {
      return;
    }

    // Collapse chunks
    curchnk[-1].mlcs_numlines += curchnk->mlcs_numlines;
    curchnk[-1].mlcs_totalsize += curchnk->mlcs_totalsize;
    buf->b_ml.ml_usedchunks--;
    if (curix < buf->b_ml.ml_usedchunks) {
      memmove(buf->b_ml.ml_chunksize + curix,
              buf->b_ml.ml_chunksize + curix + 1,
              (size_t)(buf->b_ml.ml_usedchunks - curix) * sizeof(chunksize_T));
    }
    return;
  }
  ml_upd_lastbuf = buf;
  ml_upd_lastline = line;
  ml_upd_lastcurline = curline;
  ml_upd_lastcurix = curix;
}

// ml_find_line_or_offset and goto_byte migrated to Rust (navigate.rs)

// inc/incl/dec/decl migrated to Rust (navigate.rs); thin wrappers below.
extern int rs_inc(pos_T *lp);
extern int rs_incl(pos_T *lp);
extern int rs_dec(pos_T *lp);
extern int rs_decl(pos_T *lp);

/// Increment position (thin wrapper calling Rust).
int inc(pos_T *lp) { return rs_inc(lp); }

/// Increment position, skipping NUL at end of non-empty lines (thin wrapper calling Rust).
int incl(pos_T *lp) { return rs_incl(lp); }

/// Decrement position (thin wrapper calling Rust).
int dec(pos_T *lp) { return rs_dec(lp); }

/// Decrement position, skipping NUL at end of non-empty lines (thin wrapper calling Rust).
int decl(pos_T *lp) { return rs_decl(lp); }

// ============================================================================
// Extmark Accessor Functions (for Rust FFI - extmark crate)
// ============================================================================

extern int rs_ml_find_line_or_offset(buf_T *buf, linenr_T lnum, int *offp, bool no_ff);

/// Find line or byte offset (thin wrapper for extmark crate FFI).
bcount_t nvim_ml_find_line_or_offset(buf_T *buf, linenr_T lnum, int *offp, bool no_ff)
{
  return rs_ml_find_line_or_offset(buf, lnum, offp, no_ff);
}
