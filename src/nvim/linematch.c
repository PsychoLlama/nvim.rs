/// @file linematch.c
///
/// Rust bridge for line matching algorithm.
/// All implementations are in src/nvim-rs/linematch/.

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include "nvim/linematch.h"
#include "nvim/pos_defs.h"
#include "xdiff/xdiff.h"

#include "linematch.c.generated.h"

// Rust FFI declarations
extern mmfile_t rs_fastforward_buf_to_lnum(mmfile_t s, linenr_T lnum);
extern size_t rs_linematch_nbuffers(const mmfile_t **diff_blk, const int *diff_len,
                                    size_t ndiffs, int **decisions, bool iwhite);

/// Fast-forward buffer to a specific line number.
mmfile_t fastforward_buf_to_lnum(mmfile_t s, linenr_T lnum)
{
  return rs_fastforward_buf_to_lnum(s, lnum);
}

/// Find optimal line alignment across multiple diff buffers.
///
/// Algorithm to find an optimal alignment of lines of a diff block with 2 or
/// more files. The algorithm is generalized to work for any number of files
/// which corresponds to another dimension added to the tensor used in the
/// algorithm.
///
/// @param diff_blk Array of mmfile_t pointers for each buffer
/// @param diff_len Array of line counts for each buffer
/// @param ndiffs Number of buffers (max 8)
/// @param[out] decisions Allocated array of decisions
/// @param iwhite Whether to ignore whitespace
/// @return Number of decisions
size_t linematch_nbuffers(const mmfile_t **diff_blk, const int *diff_len, const size_t ndiffs,
                          int **decisions, bool iwhite)
{
  return rs_linematch_nbuffers(diff_blk, diff_len, ndiffs, decisions, iwhite);
}
