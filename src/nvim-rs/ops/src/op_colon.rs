//! op_colon: handle the ':' operator (filter, indent, format).
//!
//! Implements `op_colon`, which stuffs a ':' command with range into the
//! readbuff so that `do_cmdline` can execute it.
//!
//! Migrated from ops.c `static void op_colon()`.

use nvim_normal::types::OpargT;
use std::ffi::{c_char, c_int};

// =============================================================================
// Operator type constants (must match ops.h)
// =============================================================================

const OP_COLON: c_int = 10;
const OP_INDENT: c_int = 8;
const OP_FORMAT: c_int = 9;

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    // Read buffer stuffing (implemented in Rust getchar crate)
    fn stuffcharReadbuff(c: c_int);
    fn stuffReadbuff(s: *const c_char);
    fn stuffnumReadbuff(n: c_int);

    // get_equalprg is already in Rust (exported as "get_equalprg")
    fn get_equalprg() -> *const c_char;

    // Cursor / buffer accessors
    fn nvim_get_cursor_lnum() -> c_int;
    fn nvim_curbuf_get_ml_line_count() -> c_int;

    // Format program string pointers
    fn nvim_get_curbuf_b_p_fp() -> *const c_char;
    fn nvim_get_p_fp() -> *const c_char;
    fn nvim_get_curbuf_b_p_fp_nonempty() -> bool;
    fn nvim_get_p_fp_nonempty() -> bool;

    // hasFolding wrapper: hasFolding(curwin, lnum, NULL, &end_out)
    fn nvim_hasFolding_lnum_end_out(lnum: c_int, end_out: *mut c_int) -> bool;

    // hasFolding for a specific lnum with no outputs (just test closed-fold)
    fn nvim_hasFolding_curwin(lnum: c_int) -> bool;
}

// =============================================================================
// op_colon implementation
// =============================================================================

/// Handle indent and format operators and visual mode ":".
///
/// Stuffs a ':' command with the appropriate range into the readbuff so that
/// `do_cmdline` can execute it.
///
/// Mirrors C `static void op_colon(oparg_T *oap)` exactly.
///
/// # Safety
/// `oap` must be a valid non-null pointer to an initialized `oparg_T`.
///
/// # Export
/// Replaces the C `op_colon` function (previously a static).
pub unsafe fn rs_op_colon_impl(oap: *mut OpargT) {
    let is_visual = (*oap).is_visual;
    let start_lnum = (*oap).start.lnum;
    let end_lnum = (*oap).end.lnum;
    let line_count = (*oap).line_count;
    let op_type = (*oap).op_type;

    stuffcharReadbuff(c_int::from(b':'));

    if is_visual {
        stuffReadbuff(c"'<,'>".as_ptr());
    } else {
        let cursor_lnum = nvim_get_cursor_lnum();

        if start_lnum == cursor_lnum {
            stuffcharReadbuff(c_int::from(b'.'));
        } else {
            stuffnumReadbuff(start_lnum);
        }

        // Check if start is in a closed fold; endOfStartFold is updated.
        let mut end_of_start_fold: c_int = start_lnum;
        nvim_hasFolding_lnum_end_out(start_lnum, &raw mut end_of_start_fold);

        if end_lnum != start_lnum && end_lnum != end_of_start_fold {
            stuffcharReadbuff(c_int::from(b','));
            if end_lnum == cursor_lnum {
                stuffcharReadbuff(c_int::from(b'.'));
            } else if end_lnum == nvim_curbuf_get_ml_line_count() {
                stuffcharReadbuff(c_int::from(b'$'));
            } else if start_lnum == cursor_lnum && !nvim_hasFolding_curwin(end_lnum) {
                stuffReadbuff(c".+".as_ptr());
                stuffnumReadbuff(line_count - 1);
            } else {
                stuffnumReadbuff(end_lnum);
            }
        }
    }

    if op_type != OP_COLON {
        stuffReadbuff(c"!".as_ptr());
    }

    if op_type == OP_INDENT {
        stuffReadbuff(get_equalprg());
        stuffReadbuff(c"\n".as_ptr());
    } else if op_type == OP_FORMAT {
        if nvim_get_curbuf_b_p_fp_nonempty() {
            stuffReadbuff(nvim_get_curbuf_b_p_fp());
        } else if nvim_get_p_fp_nonempty() {
            stuffReadbuff(nvim_get_p_fp());
        } else {
            stuffReadbuff(c"fmt".as_ptr());
        }
        stuffReadbuff(c"\n']".as_ptr());
    }
    // do_cmdline() does the rest
}
