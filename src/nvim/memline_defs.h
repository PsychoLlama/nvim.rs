#pragma once

#include <stdint.h>

#include "nvim/memfile_defs.h"
#include "nvim/pos_defs.h"

// Block 0 constants (for ZeroBlock)
enum {
  B0_FNAME_SIZE_ORG = 900,      // what it was in older versions
  B0_UNAME_SIZE = 40,
  B0_HNAME_SIZE = 40,
};

/// Block zero holds all info about the swapfile.
/// NOTE: DEFINITION OF BLOCK 0 SHOULD NOT CHANGE! It would make all existing swapfiles unusable!
typedef struct {
  char b0_id[2];                     ///< ID for block 0: BLOCK0_ID0 and BLOCK0_ID1.
  char b0_version[10];               ///< Vim version string
  char b0_page_size[4];              ///< number of bytes per page
  char b0_mtime[4];                  ///< last modification time of file
  char b0_ino[4];                    ///< inode of b0_fname
  char b0_pid[4];                    ///< process id of creator (or 0)
  char b0_uname[B0_UNAME_SIZE];      ///< name of user (uid if no name)
  char b0_hname[B0_HNAME_SIZE];      ///< host name (if it has a name)
  char b0_fname[B0_FNAME_SIZE_ORG];  ///< name of file being edited
  long b0_magic_long;                ///< check for byte order of long
  int b0_magic_int;                  ///< check for byte order of int
  int16_t b0_magic_short;            ///< check for byte order of short
  char b0_magic_char;                ///< check for last char
} ZeroBlock;

///
/// When searching for a specific line, we remember what blocks in the tree
/// are the branches leading to that block. This is stored in ml_stack.  Each
/// entry is a pointer to info in a block (may be data block or pointer block)
///
typedef struct {
  blocknr_T ip_bnum;            // block number
  linenr_T ip_low;              // lowest lnum in this block
  linenr_T ip_high;             // highest lnum in this block
  int ip_index;                 // index for block with current lnum
} infoptr_T;    // block/index pair

typedef struct {
  int mlcs_numlines;
  int mlcs_totalsize;
} chunksize_T;

// Flags when calling ml_updatechunk()
#define ML_CHNK_ADDLINE 1
#define ML_CHNK_DELLINE 2
#define ML_CHNK_UPDLINE 3

/// memline structure: the contents of a buffer.
/// Essentially a tree with a branch factor of 128.
/// Lines are stored at leaf nodes.
/// Nodes are stored on ml_mfp (memfile_T):
///   pointer_block: internal nodes
///   data_block: leaf nodes
///
/// Memline also has "chunks" of 800 lines that are separate from the 128-tree
/// structure, primarily used to speed up line2byte() and byte2line().
///
/// Motivation: If you have a file that is 10000 lines long, and you insert
///             a line at linenr 1000, you don't want to move 9000 lines in
///             memory.  With this structure it is roughly (N * 128) pointer
///             moves, where N is the height (typically 1-3).
///
typedef struct {
  linenr_T ml_line_count;       // number of lines in the buffer

  memfile_T *ml_mfp;          // pointer to associated memfile

  infoptr_T *ml_stack;        // stack of pointer blocks (array of IPTRs)
  int ml_stack_top;             // current top of ml_stack
  int ml_stack_size;            // total number of entries in ml_stack

#define ML_EMPTY        0x01    // empty buffer
#define ML_LINE_DIRTY   0x02    // cached line was changed and allocated
#define ML_LOCKED_DIRTY 0x04    // ml_locked was changed
#define ML_LOCKED_POS   0x08    // ml_locked needs positive block number
#define ML_ALLOCATED    0x10    // ml_line_ptr is an allocated copy
  int ml_flags;

  colnr_T ml_line_len;          // length of the cached line + NUL
  linenr_T ml_line_lnum;        // line number of cached line, 0 if not valid
  char *ml_line_ptr;            // pointer to cached line
  size_t ml_line_offset;        // cached byte offset of ml_line_lnum
  int ml_line_offset_ff;        // fileformat of cached line

  bhdr_T *ml_locked;       // block used by last ml_get
  linenr_T ml_locked_low;       // first line in ml_locked
  linenr_T ml_locked_high;      // last line in ml_locked
  int ml_locked_lineadd;        // number of lines inserted in ml_locked
  chunksize_T *ml_chunksize;
  int ml_numchunks;
  int ml_usedchunks;
} memline_T;
