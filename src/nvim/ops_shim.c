// ops_shim.c: Rust FFI accessors for ops crate (oparg_T / b_op_* wrappers).

#include <stdbool.h>

#include "nvim/buffer_defs.h"
#include "nvim/globals.h"
#include "nvim/normal.h"
#include "nvim/pos_defs.h"

#include "ops_shim.c.generated.h"

void nvim_curbuf_set_op_start_from_oap_start(void *oap_ptr) { oparg_T *oap = (oparg_T *)oap_ptr; curbuf->b_op_start = oap->start; }
void nvim_curbuf_set_op_end_from_oap_start(void *oap_ptr) { oparg_T *oap = (oparg_T *)oap_ptr; curbuf->b_op_end = oap->start; }
void nvim_curbuf_set_op_end_blockwise(void *oap_ptr) { oparg_T *oap = (oparg_T *)oap_ptr; curbuf->b_op_end.lnum = oap->end.lnum; curbuf->b_op_end.col = oap->start.col; }
void nvim_curbuf_set_op_end_from_oap_end(void *oap_ptr) { oparg_T *oap = (oparg_T *)oap_ptr; curbuf->b_op_end = oap->end; }
void nvim_curwin_set_cursor_from_oap_start(void *oap_ptr) { oparg_T *oap = (oparg_T *)oap_ptr; curwin->w_cursor = oap->start; }
