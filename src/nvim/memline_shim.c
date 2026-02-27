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
// Phase 2 Rust function declarations
extern int rs_swapfile_proc_running(const ZeroBlock *b0p, const char *swap_fname);
// Pass 2 Phase 1: Mark tracking Rust function declarations
extern void rs_ml_setmarked(linenr_T lnum);
extern linenr_T rs_ml_firstmarked(void);
extern void rs_ml_clearmarked(void);
extern linenr_T rs_ml_get_lowest_marked(void);
extern void rs_ml_set_lowest_marked(linenr_T lnum);
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
// Pass 5 Phase 2: ml_flush_line Rust function declaration
extern void rs_ml_flush_line(buf_T *buf, int noalloc);
// Pass 8 Phase 2: ml_preserve Rust function declaration
extern void rs_ml_preserve(buf_T *buf, bool message, bool do_fsync);
// Pass 9 Phase 1: ml_open_file + ml_open_files Rust function declarations
extern void rs_ml_open_file(buf_T *buf);
extern void rs_ml_open_files(void);
// Pass 9 Phase 2: ml_setname Rust function declaration
extern void rs_ml_setname(buf_T *buf);
// Pass 9 Phase 3: ml_open Rust function declaration
extern int rs_ml_open(buf_T *buf);
// Pass 9 Phase 4: buffer-iteration wrappers Rust function declarations
extern void rs_ml_close_all(int del_file);
extern void rs_ml_close_notmod(void);
extern void rs_ml_sync_all(int check_file, int check_char, bool do_fsync);

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

/// Open a new memline for "buf". (thin wrapper calling Rust)
///
/// @return  FAIL for failure, OK otherwise.
int ml_open(buf_T *buf) { return rs_ml_open(buf); }

/// ml_setname() is called when the file name of "buf" has been changed.
/// It may rename the swapfile. (thin wrapper calling Rust)
void ml_setname(buf_T *buf) { rs_ml_setname(buf); }

/// Open a file for the memfile for all buffers that are not readonly or have
/// been modified.
/// Used when 'updatecount' changes from zero to non-zero.
/// (thin wrapper calling Rust)
void ml_open_files(void) { rs_ml_open_files(); }

/// Open a swapfile for an existing memfile, if there is no swapfile yet.
/// (thin wrapper calling Rust)
void ml_open_file(buf_T *buf) { rs_ml_open_file(buf); }

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

/// Close all existing memlines and memfiles. (thin wrapper calling Rust)
/// Only used when exiting.
///
/// @param del_file  if true, delete the memfiles.
void ml_close_all(bool del_file) { rs_ml_close_all((int)del_file); }

/// Close all memfiles for not modified buffers. (thin wrapper calling Rust)
/// Only use just before exiting!
void ml_close_notmod(void) { rs_ml_close_notmod(); }

/// Update the timestamp in the .swp file (thin wrapper calling Rust).
/// Used when the file has been written.
void ml_timestamp(buf_T *buf) { rs_ml_timestamp(buf); }


// Forward declaration for Rust implementation (migrated from C)
extern int rs_recover_names(const char *fname, int do_list, void *ret_list, int nr,
                            char **fname_out);
// Pass 10 Phase 1: ml_recover migrated to Rust
extern void rs_ml_recover(int checkext);

/// Try to recover curbuf from the .swp file. (thin wrapper calling Rust)
///
/// @param checkext  if true, check the extension and detect whether it is a swapfile.
void ml_recover(bool checkext) { rs_ml_recover(checkext ? 1 : 0); }

// recover_names and recov_file_names migrated to Rust (recovery.rs)

/// Append the full path to name with path separators made into percent signs.
/// (thin wrapper calling Rust)
char *make_percent_swname(char *dir, char *dir_end, const char *name)
  FUNC_ATTR_NONNULL_ARG(1, 2)
{
  return rs_make_percent_swname(dir, dir_end, name);
}

/// For Vimscript "swapinfo()".
///
/// @return  information found in swapfile "fname" in dictionary "d".
void swapfile_dict(const char *fname, dict_T *d)
{
  rs_swapfile_dict(fname, d);
}

// recov_file_names migrated to Rust (recovery.rs)
// proc_running static, swapfile_info wrapper migrated to Rust (swap.rs Phase 8)

/// sync all memlines (thin wrapper calling Rust)
void ml_sync_all(int check_file, int check_char, bool do_fsync)
{
  rs_ml_sync_all(check_file, check_char, do_fsync);
}

/// sync one buffer, including negative blocks
///
/// after this all the blocks are in the swapfile (thin wrapper calling Rust)
///
/// Used for the :preserve command and when the original file has been
/// changed or deleted.
///
/// @param message  if true, the success of preserving is reported.
void ml_preserve(buf_T *buf, bool message, bool do_fsync)
{
  rs_ml_preserve(buf, message, do_fsync);
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

// Print swapfile info (calls rs_swapfile_info directly; proc_running discarded for display)
void nvim_swapfile_info_and_print(char *fname)
{
  StringBuilder msg = KV_INITIAL_VALUE;
  kv_resize(msg, IOSIZE);
  int proc_running_unused = 0;
  rs_swapfile_info(fname, &msg, &proc_running_unused);
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
void nvim_iemsg_pointer_block_id_wrong_two(void) { iemsg(_(e_pointer_block_id_wrong_two)); }
void nvim_iemsg_e304_upd_block0(void) { iemsg(_("E304: ml_upd_block0(): Didn't get block 0??")); }

// Pass 5 Phase 1: ml_find_line accessors for Rust FFI

// ml_locked block pointer accessors
void *nvim_buf_get_ml_locked(buf_T *buf) { return buf->b_ml.ml_locked; }
void nvim_buf_set_ml_locked(buf_T *buf, void *hp) { buf->b_ml.ml_locked = hp; }

// ml_locked_lineadd accessors
int nvim_buf_get_ml_locked_lineadd(buf_T *buf) { return buf->b_ml.ml_locked_lineadd; }
void nvim_buf_set_ml_locked_lineadd(buf_T *buf, int val) { buf->b_ml.ml_locked_lineadd = val; }

// ml_locked_low and ml_locked_high setters (getters already exist above)
void nvim_buf_set_ml_locked_low(buf_T *buf, linenr_T val) { buf->b_ml.ml_locked_low = val; }
void nvim_buf_set_ml_locked_high(buf_T *buf, linenr_T val) { buf->b_ml.ml_locked_high = val; }

// infoptr_T ip_low / ip_high getters and ip_bnum / ip_low / ip_high / ip_index setters
linenr_T nvim_ip_get_low(const infoptr_T *ip) { return ip->ip_low; }
linenr_T nvim_ip_get_high(const infoptr_T *ip) { return ip->ip_high; }
void nvim_ip_set_bnum(infoptr_T *ip, int64_t bnum) { ip->ip_bnum = (blocknr_T)bnum; }
void nvim_ip_set_low(infoptr_T *ip, linenr_T lnum) { ip->ip_low = lnum; }
void nvim_ip_set_high(infoptr_T *ip, linenr_T lnum) { ip->ip_high = lnum; }
void nvim_ip_set_index(infoptr_T *ip, int idx) { ip->ip_index = idx; }

// PointerBlock field accessors for ml_find_line
uint16_t nvim_pp_get_count(const void *pp) { return ((const PointerBlock *)pp)->pb_count; }
int64_t nvim_pp_pe_get_bnum(const void *pp, int idx)
{
  return (int64_t)((const PointerBlock *)pp)->pb_pointer[idx].pe_bnum;
}
linenr_T nvim_pp_pe_get_line_count(const void *pp, int idx)
{
  return ((const PointerBlock *)pp)->pb_pointer[idx].pe_line_count;
}
int nvim_pp_pe_get_page_count(const void *pp, int idx)
{
  return ((const PointerBlock *)pp)->pb_pointer[idx].pe_page_count;
}
void nvim_pp_pe_set_bnum(void *pp, int idx, int64_t bnum)
{
  ((PointerBlock *)pp)->pb_pointer[idx].pe_bnum = (blocknr_T)bnum;
}
void nvim_pp_pe_dec_line_count(void *pp, int idx)
{
  ((PointerBlock *)pp)->pb_pointer[idx].pe_line_count--;
}
void nvim_pp_pe_inc_line_count(void *pp, int idx)
{
  ((PointerBlock *)pp)->pb_pointer[idx].pe_line_count++;
}

// uint16_t db_id accessor for DataBlock (to distinguish from PointerBlock)
uint16_t nvim_dp_get_id(const void *dp) { return ((const DataBlock *)dp)->db_id; }

// Error message wrappers for ml_find_line
void nvim_iemsg_pointer_block_id_wrong(void) { iemsg(_(e_pointer_block_id_wrong)); }
void nvim_siemsg_line_number_out_of_range(int64_t lnum_past)
{
  siemsg(_(e_line_number_out_of_range_nr_past_the_end), lnum_past);
}
void nvim_siemsg_line_count_wrong_in_block(int64_t bnum)
{
  siemsg(_(e_line_count_wrong_in_block_nr), bnum);
}

// Pass 5 Phase 2: ml_flush_line accessors for Rust FFI

/// Increment buf->flush_count
void nvim_buf_inc_flush_count(buf_T *buf) { buf->flush_count++; }


// Pass 6 Phase 1: ml_delete_int accessors for Rust FFI

/// Decrement buf->b_ml.ml_line_count and return new value.
linenr_T nvim_buf_dec_ml_line_count(buf_T *buf) { return --buf->b_ml.ml_line_count; }

/// Increment buf->b_ml.ml_line_count and return new value.
linenr_T nvim_buf_inc_ml_line_count(buf_T *buf) { return ++buf->b_ml.ml_line_count; }

/// Get buf->b_prev_line_count
linenr_T nvim_buf_get_b_prev_line_count(buf_T *buf) { return buf->b_prev_line_count; }

/// Set buf->b_prev_line_count
void nvim_buf_set_b_prev_line_count(buf_T *buf, linenr_T val) { buf->b_prev_line_count = val; }

/// set_keep_msg(_(no_lines_msg), 0) -- "No lines in buffer" message
void nvim_set_keep_msg_no_lines(void) { set_keep_msg(_(no_lines_msg), 0); }

/// iemsg for "E317: Pointer block id wrong 4"
void nvim_iemsg_pointer_block_id_wrong_four(void) { iemsg(_(e_pointer_block_id_wrong_four)); }

/// Free a block in the memfile (mf_free wrapper).
void nvim_mf_free(memfile_T *mfp, bhdr_T *hp) { mf_free(mfp, hp); }

/// Decrement pp->pb_count and return new value.
int nvim_pp_dec_count(void *pp) { return --(((PointerBlock *)pp)->pb_count); }

/// memmove(pp->pb_pointer + dst_idx, pp->pb_pointer + src_idx, count * sizeof(PointerEntry))
void nvim_pp_pe_memmove(void *pp, int dst_idx, int src_idx, int count)
{
  PointerBlock *pb = (PointerBlock *)pp;
  memmove(&pb->pb_pointer[dst_idx], &pb->pb_pointer[src_idx],
          (size_t)count * sizeof(PointerEntry));
}

// Pass 7 Phase 1: ml_append_int accessors for Rust FFI

/// Get hp->bh_bnum as int64_t.
int64_t nvim_bhdr_get_bh_bnum(bhdr_T *hp) { return (int64_t)hp->bh_bnum; }

/// Get hp->bh_page_count as int.
int nvim_bhdr_get_bh_page_count(bhdr_T *hp) { return (int)hp->bh_page_count; }

/// iemsg for "E317: Pointer block id wrong 3"
void nvim_iemsg_pointer_block_id_wrong_three(void) { iemsg(_(e_pointer_block_id_wrong_three)); }

/// iemsg for "E318: Updated too many blocks?"
void nvim_iemsg_e318_updated_too_many(void) { iemsg(_("E318: Updated too many blocks?")); }

/// Increment pp->pb_count and return new value.
uint16_t nvim_pp_inc_count(void *pp) { return ++(((PointerBlock *)pp)->pb_count); }

// Pass 6 Phase 2: ml_updatechunk accessors for Rust FFI

/// Set buf->b_ml.ml_chunksize[idx].mlcs_numlines
void nvim_buf_set_ml_chunksize_numlines(buf_T *buf, int idx, int val)
{
  buf->b_ml.ml_chunksize[idx].mlcs_numlines = val;
}

/// Set buf->b_ml.ml_chunksize[idx].mlcs_totalsize
void nvim_buf_set_ml_chunksize_totalsize(buf_T *buf, int idx, int val)
{
  buf->b_ml.ml_chunksize[idx].mlcs_totalsize = val;
}

/// Add val to buf->b_ml.ml_chunksize[idx].mlcs_numlines
void nvim_buf_add_ml_chunksize_numlines(buf_T *buf, int idx, int val)
{
  buf->b_ml.ml_chunksize[idx].mlcs_numlines += val;
}

/// Add val to buf->b_ml.ml_chunksize[idx].mlcs_totalsize
void nvim_buf_add_ml_chunksize_totalsize(buf_T *buf, int idx, int val)
{
  buf->b_ml.ml_chunksize[idx].mlcs_totalsize += val;
}

/// Set buf->b_ml.ml_numchunks
void nvim_buf_set_ml_numchunks(buf_T *buf, int val) { buf->b_ml.ml_numchunks = val; }

/// Set buf->b_ml.ml_usedchunks
void nvim_buf_set_ml_usedchunks(buf_T *buf, int val) { buf->b_ml.ml_usedchunks = val; }

/// memmove within ml_chunksize: move count entries from src_idx to dst_idx.
void nvim_buf_ml_chunksize_memmove(buf_T *buf, int dst_idx, int src_idx, int count)
{
  memmove(buf->b_ml.ml_chunksize + dst_idx,
          buf->b_ml.ml_chunksize + src_idx,
          (size_t)count * sizeof(chunksize_T));
}

/// Ensure capacity for usedchunks+1: grow ml_chunksize array by 3/2 if needed.
void nvim_buf_ml_chunksize_ensure_capacity(buf_T *buf)
{
  if (buf->b_ml.ml_usedchunks + 1 >= buf->b_ml.ml_numchunks) {
    buf->b_ml.ml_numchunks = buf->b_ml.ml_numchunks * 3 / 2;
    buf->b_ml.ml_chunksize = xrealloc(buf->b_ml.ml_chunksize,
                                      sizeof(chunksize_T) * (size_t)buf->b_ml.ml_numchunks);
  }
}

/// Allocate initial ml_chunksize array (100 entries) and set first entry to (1, 1).
void nvim_buf_ml_chunksize_init(buf_T *buf)
{
  buf->b_ml.ml_chunksize = xmalloc(sizeof(chunksize_T) * 100);
  buf->b_ml.ml_numchunks = 100;
  buf->b_ml.ml_usedchunks = 1;
  buf->b_ml.ml_chunksize[0].mlcs_numlines = 1;
  buf->b_ml.ml_chunksize[0].mlcs_totalsize = 1;
}

/// Print "E320: Cannot find line N" error for ml_flush_line.
void nvim_siemsg_e320_cannot_find_line(int64_t lnum)
{
  siemsg(_("E320: Cannot find line %" PRId64), lnum);
}

// buf->b_mtime setters
void nvim_buf_set_b_mtime(buf_T *buf, int64_t val) { buf->b_mtime = val; }
void nvim_buf_set_b_mtime_ns(buf_T *buf, int64_t val) { buf->b_mtime_ns = val; }
void nvim_buf_set_b_mtime_read(buf_T *buf, int64_t val) { buf->b_mtime_read = val; }
void nvim_buf_set_b_mtime_read_ns(buf_T *buf, int64_t val) { buf->b_mtime_read_ns = val; }
void nvim_buf_set_b_orig_size(buf_T *buf, int64_t val) { buf->b_orig_size = (uint64_t)val; }
void nvim_buf_set_b_orig_mode(buf_T *buf, int val) { buf->b_orig_mode = val; }

// ZeroBlock field setters for set_b0_fname
void nvim_b0_set_fname0(ZeroBlock *b0p) { b0p->b0_fname[0] = NUL; }
char *nvim_b0_get_fname_for_replace(ZeroBlock *b0p) { return b0p->b0_fname; }
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

// Pass 9 Phase 1: ml_open_file + ml_open_files accessors for Rust FFI

/// Get buf->b_spell (returns 1 if true, 0 if false)
int nvim_buf_get_b_spell(buf_T *buf) { return buf->b_spell ? 1 : 0; }

/// Set buf->b_may_swap (0 = false, non-zero = true)
void nvim_buf_set_b_may_swap(buf_T *buf, int val) { buf->b_may_swap = (val != 0); }

// Pass 9 Phase 2: ml_setname accessors for Rust FFI

/// Wrap os_set_cloexec (os_set_cloexec itself is now a thin wrapper around Rust)
void nvim_os_set_cloexec(int fd) { os_set_cloexec(fd); }

/// Rename a file (vim_rename wrapper)
int nvim_vim_rename(const char *from, const char *to) { return vim_rename(from, to); }

/// Get O_RDWR flag value for os_open calls
int nvim_get_o_rdwr(void) { return O_RDWR; }

// Pass 9 Phase 3: ml_open accessors for Rust FFI

/// Initialize all ml fields to their zero/NULL defaults for ml_open
void nvim_buf_init_ml_empty(buf_T *buf)
{
  buf->b_ml.ml_stack_size = 0;
  buf->b_ml.ml_stack = NULL;
  buf->b_ml.ml_stack_top = 0;
  buf->b_ml.ml_locked = NULL;
  buf->b_ml.ml_line_lnum = 0;
  buf->b_ml.ml_line_offset = 0;
  buf->b_ml.ml_chunksize = NULL;
  buf->b_ml.ml_usedchunks = 0;
}

/// Set buf->b_ml.ml_mfp
void nvim_buf_set_ml_mfp(buf_T *buf, void *mfp) { buf->b_ml.ml_mfp = mfp; }

/// Initialize block 0 header fields (magic numbers, version, page_size)
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

/// Initialize the root pointer block for ml_open (block 1)
void nvim_pp_init_root(void *pp_raw)
{
  PointerBlock *pp = (PointerBlock *)pp_raw;
  pp->pb_count = 1;
  pp->pb_pointer[0].pe_bnum = 2;
  pp->pb_pointer[0].pe_page_count = 1;
  pp->pb_pointer[0].pe_old_lnum = 1;
  pp->pb_pointer[0].pe_line_count = 1;  // line count after insertion
}

/// Initialize the first data block with an empty line for ml_open (block 2)
void nvim_dp_init_empty_line(void *dp_raw)
{
  DataBlock *dp = (DataBlock *)dp_raw;
  dp->db_index[0] = --dp->db_txt_start;       // at end of block
  dp->db_free -= 1 + (unsigned)INDEX_SIZE;
  dp->db_line_count = 1;
  *((char *)dp + dp->db_txt_start) = NUL;     // empty line
}


/// Set buf->b_p_swf = false
void nvim_buf_set_b_p_swf_false(buf_T *buf) { buf->b_p_swf = false; }

/// Set buf->b_may_swap = true
void nvim_buf_set_b_may_swap_true(buf_T *buf) { buf->b_may_swap = true; }

/// Get buf->b_help (returns 1 if true)
// nvim_buf_get_b_help already in option_shim.c

/// Set b0_dirty field in block 0 from buf->b_changed
void nvim_b0_set_dirty_from_buf(ZeroBlock *b0p, buf_T *buf)
{
  b0p->b0_dirty = buf->b_changed ? B0_DIRTY : 0;
}

/// Set b0_flags from fileformat
void nvim_b0_set_flags_from_ff(ZeroBlock *b0p, int fileformat)
{
  b0p->b0_flags = (char)(fileformat + 1);
}

/// Copy username into b0_uname, NUL-terminate
void nvim_b0_fill_uname(ZeroBlock *b0p) { os_get_username(b0p->b0_uname, B0_UNAME_SIZE); }

/// Copy hostname into b0_hname, NUL-terminate
void nvim_b0_fill_hname(ZeroBlock *b0p) { os_get_hostname(b0p->b0_hname, B0_HNAME_SIZE); }

/// Write PID into b0_pid
void nvim_b0_fill_pid(ZeroBlock *b0p) { rs_long_to_char((long)os_get_pid(), b0p->b0_pid); }

// Pass 8 Phase 1: findswapname cluster accessors for Rust FFI

/// Get the swap_exists_action global
int nvim_get_swap_exists_action(void) { return swap_exists_action; }

/// Set the swap_exists_action global
void nvim_set_swap_exists_action(int val) { swap_exists_action = val; }

/// Get the recoverymode global
int nvim_get_recoverymode(void) { return recoverymode ? 1 : 0; }

/// Get the p_shm option string
const char *nvim_get_p_shm(void) { return p_shm; }

/// Increment no_wait_return
void nvim_inc_no_wait_return(void) { no_wait_return++; }

/// Decrement no_wait_return
void nvim_dec_no_wait_return(void) { no_wait_return--; }

// nvim_get_no_wait_return is already in option_shim.c

/// Set buf->b_p_ro = true
void nvim_buf_set_b_p_ro_true(buf_T *buf) { buf->b_p_ro = true; }

/// Check if a path link exists (os_fileinfo_link wrapper)
int nvim_os_fileinfo_link(const char *fname)
{
  FileInfo fi;
  return os_fileinfo_link(fname, &fi) ? 1 : 0;
}

/// Read block 0 from a swap file fd. Returns sizeof(ZeroBlock) on success, -1 on failure.
int nvim_read_block0(int fd, ZeroBlock *b0p)
{
  ssize_t n = read_eintr(fd, b0p, sizeof(*b0p));
  return (n == (ssize_t)sizeof(*b0p)) ? 1 : 0;
}

// nvim_path_fnamecmp is already in buffer.c

/// Check if two paths are in the same directory (same_directory wrapper)
int nvim_same_directory(const char *a, const char *b) { return same_directory(a, b); }

/// Expand environment variables in src to dst, maxlen version (expand_env wrapper)
void nvim_expand_env_maxpathl(const char *src, char *dst, int len) { expand_env((char *)src, dst, len); }

/// Check if path is a directory (os_isdir wrapper)
int nvim_os_isdir(const char *name) { return os_isdir(name) ? 1 : 0; }

/// Create directory recursively (os_mkdir_recurse wrapper)
int nvim_os_mkdir_recurse(const char *dir, int mode, char **failed_dir)
{
  return os_mkdir_recurse(dir, mode, failed_dir, NULL);
}

// nvim_os_remove is already in undo.c

/// Get path tail pointer from a file path (for findswapname, const-correct version)
char *nvim_path_tail_const(const char *fname) { return path_tail(fname); }

/// Check whether SwapExists autocmd exists for this file
int nvim_has_autocmd_swapexists(const char *fname, buf_T *buf)
{
  return has_autocmd(EVENT_SWAPEXISTS, fname, buf) ? 1 : 0;
}

/// Apply SwapExists autocommands for the given buffer + fname
void nvim_apply_autocmds_swapexists(const char *fname, buf_T *buf)
{
  allbuf_lock++;
  apply_autocmds(EVENT_SWAPEXISTS, (char *)fname, NULL, false, NULL);
  allbuf_lock--;
}

/// Get v:swapchoice string value (get_vim_var_str wrapper)
const char *nvim_get_vim_var_swapchoice(void) { return get_vim_var_str(VV_SWAPCHOICE); }

/// Set v:swapname (set_vim_var_string wrapper)
void nvim_set_vim_var_swapname(const char *fname) { set_vim_var_string(VV_SWAPNAME, (char *)fname, -1); }

/// Clear v:swapname (set to NULL)
void nvim_clear_vim_var_swapname(void) { set_vim_var_string(VV_SWAPNAME, NULL, -1); }

/// Clear v:swapchoice (set to NULL)
void nvim_clear_vim_var_swapchoice(void) { set_vim_var_string(VV_SWAPCHOICE, NULL, -1); }

/// Show a multiline confirm dialog and return user choice (do_dialog wrapper).
/// Returns the button index (1-based) or 0 if no choice.
int nvim_do_dialog_warning(const char *title, const char *message,
                           const char *buttons, int dflt_button, bool mouse_used)
{
  return do_dialog(VIM_WARNING, (char *)title, (char *)message,
                   (char *)buttons, dflt_button, NULL, mouse_used);
}

/// Flush type-ahead buffers (flush_buffers wrapper)
void nvim_flush_buffers_typeahead(void) { flush_buffers(FLUSH_TYPEAHEAD); }

/// Reset scroll position for messages (msg_reset_scroll wrapper)
void nvim_msg_reset_scroll(void) { msg_reset_scroll(); }

/// Output a multiline message string with highlight (msg_multiline wrapper)
void nvim_msg_multiline(const char *s, int hl_id)
{
  bool need_clear = false;
  msg_multiline(cbuf_as_string((char *)s, strlen(s)), hl_id, false, false, &need_clear);
}

/// Print a "verbose" message (verb_msg wrapper)
void nvim_verb_msg(const char *s) { verb_msg((char *)s); }

/// Get SHM_ATTENTION character constant
int nvim_get_shm_attention(void) { return SHM_ATTENTION; }

/// Open file for reading, return fd (os_open wrapper)
int nvim_os_open_rdonly(const char *fname) { return os_open(fname, O_RDONLY, 0); }

/// Close an fd (close wrapper)
void nvim_close_fd(int fd) { close(fd); }

/// Allocate and initialize a StringBuilder for attention message building
void *nvim_alloc_stringbuilder_iosize(void)
{
  StringBuilder *sb = xmalloc(sizeof(StringBuilder));
  *sb = (StringBuilder)KV_INITIAL_VALUE;
  kv_resize(*sb, IOSIZE);
  return sb;
}

/// Get the items pointer from a StringBuilder (as char*)
const char *nvim_sb_get_items(void *sb) { return ((StringBuilder *)sb)->items; }

/// Get the size of a StringBuilder
size_t nvim_sb_get_size(void *sb) { return ((StringBuilder *)sb)->size; }

/// Destroy and free a StringBuilder
void nvim_free_stringbuilder(void *sb)
{
  kv_destroy(*(StringBuilder *)sb);
  xfree(sb);
}

/// Append a literal string to a StringBuilder (no format)
void nvim_sb_append(void *sb, const char *s)
{
  kv_printf(*(StringBuilder *)sb, "%s", s);
}

// nvim_emsg(const char *s) is already in normal_shim.c

/// msg_puts("\n") wrapper
void nvim_msg_puts_newline(void) { msg_puts("\n"); }

/// os_strerror wrapper (os_strerror is a macro, cannot be referenced directly from Rust)
const char *nvim_os_strerror(int err) { return os_strerror(err); }

// Pass 8 Phase 2: ml_preserve and ml_sync_all accessors for Rust FFI

/// Sync memfile blocks to disk
int nvim_mf_sync(memfile_T *mfp, int flags) { return mf_sync(mfp, flags); }

/// Check if memfile has blocks needing block number translation
int nvim_mf_need_trans(memfile_T *mfp) { return mf_need_trans(mfp); }

/// Check if memfile has dirty blocks (mf_dirty == MF_DIRTY_YES)
int nvim_mf_is_dirty(memfile_T *mfp) { return mfp->mf_dirty == MF_DIRTY_YES ? 1 : 0; }

/// Check if a character is available (for stopping sync mid-loop)
int nvim_os_char_avail(void) { return os_char_avail() ? 1 : 0; }

/// Set need_check_timestamps global
void nvim_set_need_check_timestamps(int val) { need_check_timestamps = val != 0; }

/// Check if original file changed since last read (mtime, mtime_ns, size comparison).
/// Returns 1 if the file has changed or doesn't exist, 0 if unchanged.
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

/// Emit "File preserved" message
void nvim_msg_file_preserved(void) { msg(_("File preserved"), 0); }

/// Emit E314 "Preserve failed" error
void nvim_emsg_preserve_failed(void) { emsg(_("E314: Preserve failed")); }

/// Emit E313 "Cannot preserve, there is no swap file" error
void nvim_emsg_no_swapfile(void) { emsg(_("E313: Cannot preserve, there is no swap file")); }

/// Public wrapper around rs_ml_append_flush, used by Rust _impl functions.
int nvim_ml_append_flush(buf_T *buf, linenr_T lnum, char *line, colnr_T len, int flags)
  FUNC_ATTR_NONNULL_ARG(1)
{
  return rs_ml_append_flush(buf, lnum, line, len, flags);
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

/// flush ml_line if necessary (thin wrapper calling Rust)
void ml_flush_line(buf_T *buf, bool noalloc)
{
  rs_ml_flush_line(buf, noalloc);
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

// attention_message, do_swapexists, findswapname migrated to Rust (swap.rs Phase 8 Pass 1)

/// Set the flags in the first block of the swapfile. (thin wrapper calling Rust)
void ml_setflags(buf_T *buf) { rs_ml_setflags(buf); }

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

// ============================================================================
// Pass 10 Phase 1: ml_recover migration accessors for Rust FFI
// ============================================================================

/// Set the recoverymode global
void nvim_set_recoverymode(int val) { recoverymode = (val != 0); }

/// Get the called_from_main flag (curbuf->b_ml.ml_mfp == NULL)
int nvim_get_called_from_main(void) { return curbuf->b_ml.ml_mfp == NULL ? 1 : 0; }

/// Open a memfile from an existing swapfile (O_RDONLY).
/// Consumes fname (mf_open frees it). Returns mfp or NULL.
memfile_T *nvim_mf_open_rdonly(char *fname) { return mf_open(fname, O_RDONLY); }

/// Close a memfile without deleting the swap file.
void nvim_mf_close_nodelete(memfile_T *mfp) { mf_close(mfp, false); }

/// Get a block from a memfile (mf_get wrapper).
bhdr_T *nvim_mf_get_block(memfile_T *mfp, int64_t bnum, unsigned page_count)
{
  return mf_get(mfp, (blocknr_T)bnum, page_count);
}

/// Release a block back to the memfile (mf_put wrapper).
void nvim_mf_put_block(memfile_T *mfp, bhdr_T *hp, bool dirty, bool infile)
{
  mf_put(mfp, hp, dirty, infile);
}

/// Set the page size of a memfile (mf_new_page_size wrapper).
void nvim_mf_new_page_size_wrapper(memfile_T *mfp, unsigned new_size) { mf_new_page_size(mfp, new_size); }

/// Set hp->bh_data to a new pointer (for block reallocation during recovery).
void nvim_bhdr_set_bh_data(bhdr_T *hp, void *data) { hp->bh_data = data; }

/// Get the MIN_SWAP_PAGE_SIZE constant.
unsigned nvim_get_min_swap_page_size(void) { return MIN_SWAP_PAGE_SIZE; }


/// Get the HLF_E highlight ID.
int nvim_get_hlf_e(void) { return HLF_E; }

/// Get the PTR_ID constant for PointerBlock identification.
uint16_t nvim_get_ptr_id(void) { return PTR_ID; }

/// Get the DATA_ID constant for DataBlock identification.
uint16_t nvim_get_data_id(void) { return DATA_ID; }

/// Get the HEADER_SIZE constant (offsetof DataBlock db_index).
unsigned nvim_get_header_size(void) { return (unsigned)HEADER_SIZE; }

/// Get PB_COUNT_MAX for a memfile.
uint16_t nvim_pp_count_max_for_mfp(memfile_T *mfp) { return PB_COUNT_MAX(mfp); }

/// Set pb_count_max on a PointerBlock.
void nvim_pp_set_count_max(void *pp, uint16_t val) { ((PointerBlock *)pp)->pb_count_max = val; }

/// Set pb_count on a PointerBlock.
void nvim_pp_set_count(void *pp, uint16_t val) { ((PointerBlock *)pp)->pb_count = val; }

/// Get pe_old_lnum from a PointerEntry by index.
linenr_T nvim_pp_pe_get_old_lnum(const void *pp, int idx)
{
  return ((const PointerBlock *)pp)->pb_pointer[idx].pe_old_lnum;
}

/// Get db_txt_end from a DataBlock.
unsigned nvim_dp_get_txt_end(const void *dp) { return ((const DataBlock *)dp)->db_txt_end; }

/// Set db_txt_end on a DataBlock.
void nvim_dp_set_txt_end(void *dp, unsigned val) { ((DataBlock *)dp)->db_txt_end = val; }


/// Get db_line_count from a DataBlock.
long nvim_dp_get_line_count(const void *dp) { return ((const DataBlock *)dp)->db_line_count; }

/// Get db_index[i] & DB_INDEX_MASK from a DataBlock.
unsigned nvim_dp_get_index_masked(const void *dp, int i)
{
  return ((const DataBlock *)dp)->db_index[i] & DB_INDEX_MASK;
}

/// Check if &dp->db_index[i] >= (char *)dp + dp->db_txt_start (index overrun check).
int nvim_dp_index_overruns_txt(const void *dp, int i)
{
  const DataBlock *d = (const DataBlock *)dp;
  return (const char *)&d->db_index[i] >= (const char *)d + d->db_txt_start ? 1 : 0;
}

/// Get a pointer to text inside a DataBlock: (char *)dp + offset.
const char *nvim_dp_get_txt_ptr(const void *dp, unsigned offset)
{
  return (const char *)dp + offset;
}

/// Write NUL at db_txt_end - 1 in a DataBlock (safety terminator).
void nvim_dp_write_nul_at_txt_end(void *dp)
{
  DataBlock *d = (DataBlock *)dp;
  *((char *)d + d->db_txt_end - 1) = NUL;
}

/// Set BF_RECOVERED flag on curbuf.
void nvim_curbuf_set_b_flags_recovered(void) { curbuf->b_flags |= BF_RECOVERED; }


/// Call getout(1) -- used when ml_open fails during recovery from main.
void nvim_getout_one(void) { getout(1); }

/// Call ml_open(curbuf) for recovery.
int nvim_ml_open_curbuf(void) { return ml_open(curbuf); }

/// Call ml_close(curbuf, true) for recovery cleanup.
void nvim_ml_close_curbuf_true(void) { ml_close(curbuf, true); }

/// Call setfname(curbuf, name, NULL, true) for recovery.
/// Returns OK (1) or FAIL (0).
int nvim_setfname_for_recovery(const char *name)
{
  return setfname(curbuf, (char *)name, NULL, true);
}

/// Get buf_spname(curbuf) -- special name for curbuf (e.g. "[No Name]"), or NULL.
const char *nvim_buf_spname_curbuf(void) { return buf_spname(curbuf); }

/// home_replace(NULL, mfp->mf_fname, NameBuff, MAXPATHL, true) -- fill NameBuff.
void nvim_home_replace_into_namebuff(const char *fname)
{
  home_replace(NULL, (char *)fname, NameBuff, MAXPATHL, true);
}

/// home_replace(NULL, curbuf->b_ffname, NameBuff, MAXPATHL, true) -- fill NameBuff.
void nvim_home_replace_curbuf_ffname_into_namebuff(void)
{
  home_replace(NULL, curbuf->b_ffname, NameBuff, MAXPATHL, true);
}

/// xstrlcpy(NameBuff, src, MAXPATHL) -- copy spname into NameBuff.
void nvim_xstrlcpy_namebuff(const char *src)
{
  xstrlcpy(NameBuff, src, MAXPATHL);
}

/// expand_env(b0_fname, NameBuff, MAXPATHL) -- expand env vars from block0 fname.
void nvim_expand_env_into_namebuff(const char *src)
{
  expand_env((char *)src, NameBuff, MAXPATHL);
}

/// Get pointer to NameBuff (read-only for passing to C funcs).
const char *nvim_get_namebuff_ptr(void) { return NameBuff; }

/// smsg(0, _("Using swap file \"%s\""), NameBuff)
void nvim_smsg_using_swap_file(void) { smsg(0, _("Using swap file \"%s\""), NameBuff); }

/// smsg(0, _("Original file \"%s\""), NameBuff)
void nvim_smsg_original_file(void) { smsg(0, _("Original file \"%s\""), NameBuff); }

/// Compare timestamps of swap file and original file (curbuf->b_ffname).
/// Returns 1 if there is a timestamp mismatch warning, 0 otherwise.
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

/// Get b0_mtime as an int (rs_char_to_long(b0p->b0_mtime)).
int nvim_b0_get_mtime_int(const ZeroBlock *b0p)
{
  return (int)rs_char_to_long(b0p->b0_mtime);
}

/// Get b0_page_size as an unsigned int (rs_char_to_long(b0p->b0_page_size)).
unsigned nvim_b0_get_page_size_int(const ZeroBlock *b0p)
{
  return (unsigned)rs_char_to_long(b0p->b0_page_size);
}

/// Get b0_pid as a char* pointer for rs_char_to_long / rs_swapfile_proc_running.
const char *nvim_b0_get_pid_as_ptr(const ZeroBlock *b0p) { return b0p->b0_pid; }

/// Read original file for recovery (readfile with READ_NEW flag, lnum=0, topline=0, MAXLNUM).
int nvim_readfile_for_recovery(const char *fname)
{
  return readfile((char *)fname, NULL, 0, 0, MAXLNUM, NULL, READ_NEW, false);
}

/// Read lines from original file starting at lnum, from topline, count lines.
int nvim_readfile_from_original(const char *fname, linenr_T lnum, linenr_T topline, linenr_T line_count)
{
  return readfile((char *)fname, NULL, lnum, topline, line_count, NULL, 0, false);
}

/// set_fileformat(ff, OPT_LOCAL) -- set file format from swap file.
void nvim_set_fileformat_local(int ff) { set_fileformat(ff, OPT_LOCAL); }

/// set_option_value_give_err(kOptFileencoding, fenc, OPT_LOCAL) -- set fileencoding.
void nvim_set_fenc_local(const char *fenc)
{
  set_option_value_give_err(kOptFileencoding, CSTR_AS_OPTVAL((char *)fenc), OPT_LOCAL);
}

/// unchanged(curbuf, true, true) -- mark curbuf as unchanged.
void nvim_unchanged_curbuf(void) { unchanged(curbuf, true, true); }

/// changed_internal(curbuf) -- mark curbuf as changed without triggering autocmds.
void nvim_changed_internal_curbuf(void) { changed_internal(curbuf); }

/// Get curbuf->b_changed.
int nvim_curbuf_get_b_changed(void) { return curbuf->b_changed ? 1 : 0; }

/// ml_delete(curbuf->b_ml.ml_line_count) -- delete last line of curbuf.
void nvim_ml_delete_last_curbuf(void) { ml_delete(curbuf->b_ml.ml_line_count); }


/// curbuf->b_ml.ml_line_count accessor.
linenr_T nvim_get_curbuf_ml_line_count(void) { return curbuf->b_ml.ml_line_count; }

/// curbuf->b_ml.ml_flags accessor.
int nvim_get_curbuf_ml_flags(void) { return curbuf->b_ml.ml_flags; }

/// Get got_int global.
int nvim_get_got_int_val(void) { return got_int ? 1 : 0; }




/// Decrement and return buf->b_ml.ml_stack_top (for stack pop).
int nvim_buf_dec_ml_stack_top(buf_T *buf) { return --(buf->b_ml.ml_stack_top); }


/// Reset stack memory for recovery (set to NULL, size 0).
void nvim_buf_reset_ml_stack(buf_T *buf)
{
  buf->b_ml.ml_stack_top = 0;
  buf->b_ml.ml_stack = NULL;
  buf->b_ml.ml_stack_size = 0;
}

/// apply_autocmds(EVENT_BUFREADPOST, NULL, curbuf->b_fname, false, curbuf).
void nvim_apply_autocmds_bufreadpost(void)
{
  apply_autocmds(EVENT_BUFREADPOST, NULL, curbuf->b_fname, false, curbuf);
}

/// apply_autocmds(EVENT_BUFWINENTER, NULL, curbuf->b_fname, false, curbuf).
void nvim_apply_autocmds_bufwinenter(void)
{
  apply_autocmds(EVENT_BUFWINENTER, NULL, curbuf->b_fname, false, curbuf);
}

/// semsg for E305: No swap file found for %s
void nvim_semsg_e305_no_swap(const char *fname) { semsg(_("E305: No swap file found for %s"), fname); }

/// semsg for E306: Cannot open %s
void nvim_semsg_e306_cannot_open(const char *fname) { semsg(_("E306: Cannot open %s"), fname); }

/// semsg for E307: %s does not look like a Nvim swap file
void nvim_semsg_e307_not_swap(const char *fname) { semsg(_("E307: %s does not look like a Nvim swap file"), fname); }

/// semsg for E309: Unable to read block 1 from %s
void nvim_semsg_e309_block1(const char *fname) { semsg(_("E309: Unable to read block 1 from %s"), fname); }

/// semsg for E310: Block 1 ID wrong (%s not a .swp file?)
void nvim_semsg_e310_block1_id(const char *fname) { semsg(_("E310: Block 1 ID wrong (%s not a .swp file?)"), fname); }

/// msg_start(); msg_puts_hl(msg, hl_id, true); msg_outtrans(fname, hl_id, true); for "Unable to read block 0"
void nvim_recover_msg_block0_unreadable(const char *fname, int hl_id)
{
  msg_start();
  msg_puts_hl(_("Unable to read block 0 from "), hl_id, true);
  msg_outtrans(fname, hl_id, true);
  msg_puts_hl(_("\nMaybe no changes were made or Nvim did not update the swap file."), hl_id, true);
  msg_end();
}

/// msg for VIM 3.0 swap file
void nvim_recover_msg_vim3(const char *fname)
{
  msg_start();
  msg_outtrans(fname, 0, true);
  msg_puts_hl(_(" cannot be used with this version of Nvim.\n"), 0, true);
  msg_puts_hl(_("Use Vim version 3.0.\n"), 0, true);
  msg_end();
}

/// msg for wrong byte order
void nvim_recover_msg_wrong_byte_order(const char *fname, int hl_id, const char *hname)
{
  msg_start();
  msg_outtrans(fname, hl_id, true);
  msg_puts_hl(_(" cannot be used on this computer.\n"), hl_id, true);
  msg_puts_hl(_("The file was created on "), hl_id, true);
  msg_puts_hl(hname, hl_id, true);
  msg_puts_hl(_(",\nor the file has been damaged."), hl_id, true);
  msg_end();
}

/// msg for page size too small
void nvim_recover_msg_page_size_too_small(const char *fname, int hl_id)
{
  msg_start();
  msg_outtrans(fname, hl_id, true);
  msg_puts_hl(_(" has been damaged (page size is smaller than minimum value).\n"), hl_id, true);
  msg_end();
}

/// emsg for E308: Warning: Original file may have been changed
void nvim_emsg_e308_original_changed(void) { emsg(_("E308: Warning: Original file may have been changed")); }

/// emsg for pointer block corrupted
void nvim_emsg_ptr_block_corrupted(void) { emsg(_(e_warning_pointer_block_corrupted)); }

/// emsg for E311: Recovery Interrupted
void nvim_emsg_e311_interrupted(void) { emsg(_("E311: Recovery Interrupted")); }


/// Post-recovery success messages
void nvim_recover_msg_success(int has_changes)
{
  if (has_changes) {
    msg(_("Recovery completed. You should check if everything is OK."), 0);
    msg_puts(_("\n(You might want to write out this file under another name\n"));
    msg_puts(_("and run diff with the original file to check for changes)"));
  } else {
    msg(_("Recovery completed. Buffer contents equals file contents."), 0);
  }
  msg_puts(_("\nYou may want to delete the .swp file now."));
}


/// Post-recovery error block output (no_wait_return bracketed messages)
void nvim_recover_msg_errors(void)
{
  no_wait_return++;
  msg(">>>>>>>>>>>>>", 0);
  emsg(_("E312: Errors detected while recovering; look for lines starting with ???"));
  no_wait_return--;
  msg(_("See \":help E312\" for more information."), 0);
  msg(">>>>>>>>>>>>>", 0);
}

/// Final: cmdline_row = msg_row
void nvim_set_cmdline_row_to_msg_row(void) { cmdline_row = msg_row; }

/// prompt_for_input for recovery: returns chosen number (0 to quit)
int nvim_prompt_for_recovery(void)
{
  return prompt_for_input(_("Enter number of swap file to use (0 to quit): "), 0, false, NULL);
}

/// Check if b0_fname[0] is NUL (used for setting up wrong_byte_order display).
void nvim_b0_set_fname0_nul(ZeroBlock *b0p) { b0p->b0_fname[0] = NUL; }


/// UPD_NOT_VALID constant for redraw
int nvim_get_upd_not_valid_val(void) { return UPD_NOT_VALID; }



/// Get sizeof(buf_T) for Rust allocation of temporary recovery buffer.
size_t nvim_get_buf_t_size(void) { return sizeof(buf_T); }


/// Get the whole b0 "proc running" check + pid message in one call at end of recovery.
/// Re-reads block 0 from the swap file (fname_used) and checks if proc is still running.
/// Prints "process STILL RUNNING: <pid>" if so. Returns 1 if running, 0 otherwise.
int nvim_recover_check_proc_and_print(const char *fname_used)
{
  // Open the swap file read-only temporarily
  int fd = os_open(fname_used, O_RDONLY, 0);
  if (fd < 0) {
    return 0;
  }
  ZeroBlock b0;
  ssize_t n = read_eintr(fd, &b0, sizeof(b0));
  close(fd);
  if (n != (ssize_t)sizeof(b0)) {
    return 0;
  }
  if (rs_swapfile_proc_running(&b0, fname_used)) {
    msg_puts(_("\nNote: process STILL RUNNING: "));
    msg_outnum((int)rs_char_to_long(b0.b0_pid));
    return 1;
  }
  return 0;
}

/// Check if b0_version starts with "VIM 3.0" (7 bytes).
int nvim_b0_is_vim3(const void *b0p)
{
  return strncmp(((const ZeroBlock *)b0p)->b0_version, "VIM 3.0", 7) == 0 ? 1 : 0;
}

/// Delete the first line of curbuf (ml_delete(1)).
void nvim_ml_delete_first_curbuf(void) { ml_delete(1); }

/// Extract the fileencoding string from block 0 b0_fname area (B0_HAS_FENC).
/// Returns a newly allocated string (caller must xfree), or NULL if not present.
char *nvim_b0_extract_fenc(const ZeroBlock *b0p)
{
  if (!(b0p->b0_flags & B0_HAS_FENC)) {
    return NULL;
  }
  int fnsize = B0_FNAME_SIZE_NOCRYPT;
  const char *p = b0p->b0_fname + fnsize;
  while (p > b0p->b0_fname && p[-1] != NUL) {
    p--;
  }
  return xstrnsave((char *)p, (size_t)(b0p->b0_fname + fnsize - p));
}

/// Get b0_flags & B0_FF_MASK (file format bits). Returns 0 if no ff stored.
int nvim_b0_get_ff(const ZeroBlock *b0p) { return b0p->b0_flags & B0_FF_MASK; }

/// Get the size of file by seeking to end (for page size recalculation).
/// Returns the file size in bytes (or 0 on error).
int64_t nvim_mf_get_file_size(memfile_T *mfp)
{
  off_T size = vim_lseek(mfp->mf_fd, 0, SEEK_END);
  return (int64_t)(size <= 0 ? 0 : size);
}

/// ml_append wrapper for recovery (appends a line to curbuf).
int nvim_ml_append_recovery(linenr_T lnum, const char *line, bool is_new)
{
  return ml_append(lnum, (char *)line, 0, is_new);
}

/// Get page count from pointer entry.
unsigned nvim_pp_pe_get_page_count_uint(const void *pp, int idx)
{
  return (unsigned)((const PointerBlock *)pp)->pb_pointer[idx].pe_page_count;
}

/// Get the pb_count_max field from a PointerBlock.
uint16_t nvim_pp_get_count_max(const void *pp) { return ((const PointerBlock *)pp)->pb_count_max; }
