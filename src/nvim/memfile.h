#pragma once

#include <stdbool.h>

#include "nvim/memfile_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

/// flags for mf_sync()
enum {
  MFS_ALL   = 1,  ///< also sync blocks with negative numbers
  MFS_STOP  = 2,  ///< stop syncing when a character is available
  MFS_FLUSH = 4,  ///< flushed file to disk
  MFS_ZERO  = 8,  ///< only write block 0
};

enum {
  /// Minimal size for block 0 of a swap file.
  /// NOTE: This depends on size of struct block0! It's not done with a sizeof(),
  /// because struct block0 is defined in memline.c (Sorry).
  /// The maximal block size is arbitrary.
  MIN_SWAP_PAGE_SIZE = 1048,
  MAX_SWAP_PAGE_SIZE = 50000,
};

// Functions implemented in Rust (nvim-memfile crate), exported directly.
memfile_T *mf_open(char *fname, int flags);
int mf_open_file(memfile_T *mfp, char *fname);
void mf_close(memfile_T *mfp, bool del_file);
void mf_new_page_size(memfile_T *mfp, unsigned new_size);
bhdr_T *mf_new(memfile_T *mfp, bool negative, unsigned page_count);
bhdr_T *mf_get(memfile_T *mfp, blocknr_T nr, unsigned page_count);
void mf_put(memfile_T *mfp, bhdr_T *hp, bool dirty, bool infile);
void mf_free(memfile_T *mfp, bhdr_T *hp);
int mf_sync(memfile_T *mfp, int flags);
blocknr_T mf_trans_del(memfile_T *mfp, blocknr_T old_nr);
void mf_free_fnames(memfile_T *mfp);
void mf_set_fnames(memfile_T *mfp, char *fname);
void mf_fullname(memfile_T *mfp);
bool mf_need_trans(memfile_T *mfp);

#include "memfile.h.generated.h"
