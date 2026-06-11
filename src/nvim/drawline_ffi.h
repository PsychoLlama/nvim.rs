#pragma once

#include "nvim/buffer_defs.h"  // IWYU pragma: keep
#include "nvim/drawline.h"  // IWYU pragma: keep
#include "nvim/pos_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

/// Result struct for advance_to_start_vcol: carries outputs from the
/// charsize iteration loop back to Rust.
typedef struct {
  int ptr_offset;       ///< byte offset of ptr into the line buffer
  int vcol;             ///< updated vcol after advancing
  bool in_multispace;   ///< whether current char is in multispace run
  int multispace_pos;   ///< current position in multispace sequence
  int skip_cells;       ///< number of cells to skip (start_vcol - vcol - head)
  int fromcol;          ///< updated wlv->fromcol
  bool need_showbreak;  ///< whether showbreak is needed
} AdvanceToStartVcolResult;

#include "drawline_ffi.h.generated.h"
