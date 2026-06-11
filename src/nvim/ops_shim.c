// ops_shim.c: Rust FFI accessors for ops crate (oparg_T / b_op_* wrappers).

#include <stdbool.h>

#include "nvim/buffer_defs.h"
#include "nvim/cursor.h"
#include "nvim/globals.h"
#include "nvim/normal.h"
#include "nvim/pos_defs.h"

#include "nvim/ops_shim.h"
#include "ops_shim.c.generated.h"

void nvim_curbuf_set_op_start_from_oap_start(void *oap_ptr) { oparg_T *oap = (oparg_T *)oap_ptr; curbuf->b_op_start = oap->start; }
void nvim_curbuf_set_op_end_from_oap_start(void *oap_ptr) { oparg_T *oap = (oparg_T *)oap_ptr; curbuf->b_op_end = oap->start; }
void nvim_curbuf_set_op_end_blockwise(void *oap_ptr) { oparg_T *oap = (oparg_T *)oap_ptr; curbuf->b_op_end.lnum = oap->end.lnum; curbuf->b_op_end.col = oap->start.col; }
void nvim_curbuf_set_op_end_from_oap_end(void *oap_ptr) { oparg_T *oap = (oparg_T *)oap_ptr; curbuf->b_op_end = oap->end; }
void nvim_curwin_set_cursor_from_oap_start(void *oap_ptr) { oparg_T *oap = (oparg_T *)oap_ptr; curwin->w_cursor = oap->start; }

// Accessors for curbuf->b_op_start_orig (used by op_insert Rust migration)
linenr_T nvim_curbuf_get_b_op_start_orig_lnum(void) { return curbuf->b_op_start_orig.lnum; }
colnr_T nvim_curbuf_get_b_op_start_orig_col(void) { return curbuf->b_op_start_orig.col; }
colnr_T nvim_curbuf_get_b_op_start_orig_coladd(void) { return curbuf->b_op_start_orig.coladd; }
void nvim_oap_set_start_from_b_op_start_orig(void *oap_ptr) { oparg_T *oap = (oparg_T *)oap_ptr; oap->start = curbuf->b_op_start_orig; }

// Set curwin->w_cursor = oap->end then check_cursor_col (used by op_insert Rust migration)
void nvim_curwin_set_cursor_from_oap_end(void *oap_ptr) { oparg_T *oap = (oparg_T *)oap_ptr; curwin->w_cursor = oap->end; check_cursor_col(curwin); }

// Set curbuf->b_op_end.lnum and curbuf->b_op_end.col (used by block_insert Rust migration)
void nvim_curbuf_set_op_end_lnum_col(int lnum, int col) { curbuf->b_op_end.lnum = (linenr_T)lnum; curbuf->b_op_end.col = (colnr_T)col; }
