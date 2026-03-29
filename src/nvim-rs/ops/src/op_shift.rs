//! `op_shift` migration (Phase 3)
//!
//! Port of `op_shift()` from ops.c — the `>` and `<` shift operators.
//! Exported as `op_shift` for existing C callers (ex_docmd).
//! `pending.rs` can call it directly via `crate::op_shift::rs_op_shift`.

#![allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_lossless,
    clippy::similar_names
)]

use nvim_normal::types::OpargT;
use std::ffi::{c_char, c_int, c_long, c_void};

// -----------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------

const NUL: c_int = 0;
const FAIL: c_int = 0;
const OP_LSHIFT: c_int = 4;
const OP_RSHIFT: c_int = 5;
const K_MT_BLOCK_WISE: c_int = 2;
const BL_SOL: c_int = 2;
const BL_FIX: c_int = 4;

// -----------------------------------------------------------------------
// FFI declarations
// -----------------------------------------------------------------------

#[allow(clashing_extern_declarations)]
extern "C" {
    // Undo
    fn u_save(top: c_int, bot: c_int) -> c_int;

    // Cursor accessors
    fn nvim_get_cursor_lnum() -> c_int;
    fn nvim_set_cursor_lnum(lnum: c_int);
    fn nvim_get_cursor_col() -> c_int;
    fn nvim_set_cursor_col(col: c_int);

    // Cursor line helpers
    fn nvim_get_cursor_line_ptr() -> *mut c_char;

    // beginline
    fn beginline(flags: c_int);

    // Fold
    fn rs_foldOpenCursor();

    // Change notification
    fn changed_lines(
        buf: *mut c_void,
        lnum: c_int,
        col: c_int,
        lnum_end: c_int,
        added: c_int,
        do_buf_event: bool,
    );

    // Message
    fn smsg(priority: c_int, fmt: *const c_char, ...) -> c_int;

    // Line length
    fn ml_get_len(lnum: c_int) -> c_int;

    // Marks
    fn nvim_cmdmod_has_lockmarks() -> c_int;
    fn nvim_curbuf_set_op_start(lnum: c_int, col: c_int);
    fn nvim_curbuf_set_op_end(lnum: c_int, col: c_int);

    // preprocs_left (from indent crate)
    fn preprocs_left() -> bool;

    // p_sr (shiftround option)
    fn nvim_p_sr() -> bool;

    // Globals
    static p_report: i64;
    static mut curbuf: *mut c_void;
}

// -----------------------------------------------------------------------
// `rs_op_shift` -- full migration of `op_shift`
// -----------------------------------------------------------------------

/// Port of `op_shift()` from ops.c.
///
/// Exported as `op_shift` for C callers (ex_docmd).
///
/// # Safety
/// - `oap` must be a valid `oparg_T *`.
/// - `curs_top`: if true, leave cursor on first line (like `>>`); if false, on last line.
/// - `amount`: number of shift levels.
#[unsafe(export_name = "op_shift")]
pub unsafe extern "C" fn rs_op_shift(oap: *mut c_void, curs_top: c_int, amount: c_int) {
    let oap_t: *mut OpargT = oap.cast();

    if u_save((*oap_t).start.lnum - 1, (*oap_t).end.lnum + 1) == FAIL {
        return;
    }

    let block_col = if (*oap_t).motion_type == K_MT_BLOCK_WISE {
        nvim_get_cursor_col()
    } else {
        0
    };

    // for (int i = oap->line_count - 1; i >= 0; i--)
    let mut i = (*oap_t).line_count - 1;
    while i >= 0 {
        let first_char = c_int::from(*nvim_get_cursor_line_ptr() as u8);
        if first_char == NUL {
            // empty line
            nvim_set_cursor_col(0);
        } else if (*oap_t).motion_type == K_MT_BLOCK_WISE {
            crate::shift_full::rs_shift_block(oap, amount);
        } else if first_char != c_int::from(b'#') || !preprocs_left() {
            // Move the line right if it doesn't start with '#', 'smartindent'
            // isn't set or 'cindent' isn't set or '#' isn't in 'cino'.
            crate::shift::rs_shift_line((*oap_t).op_type == OP_LSHIFT, nvim_p_sr(), amount, 0);
        }
        nvim_set_cursor_lnum(nvim_get_cursor_lnum() + 1);
        i -= 1;
    }

    if (*oap_t).motion_type == K_MT_BLOCK_WISE {
        nvim_set_cursor_lnum((*oap_t).start.lnum);
        nvim_set_cursor_col(block_col);
    } else if curs_top != 0 {
        // put cursor on first line, for ">>"
        nvim_set_cursor_lnum((*oap_t).start.lnum);
        beginline(BL_SOL | BL_FIX); // shift_line() may have set cursor.col
    } else {
        // put cursor on last line, for ":>"
        nvim_set_cursor_lnum(nvim_get_cursor_lnum() - 1);
    }

    // The cursor line is not in a closed fold
    rs_foldOpenCursor();

    // Message: only if line_count > p_report
    let line_count = (*oap_t).line_count;
    if i64::from(line_count) > p_report {
        let op_char: u8 = if (*oap_t).op_type == OP_RSHIFT {
            b'>'
        } else {
            b'<'
        };

        // 4 variants: (line singular/plural) x (time singular/plural)
        // Matches the C NGETTEXT logic:
        //   msg_line_single = NGETTEXT("%" PRId64 " line %sed %d time",
        //                              "%" PRId64 " line %sed %d times", amount)
        //   msg_line_plural = NGETTEXT("%" PRId64 " lines %sed %d time",
        //                              "%" PRId64 " lines %sed %d times", amount)
        //   vim_snprintf(IObuff, IOSIZE,
        //                NGETTEXT(msg_line_single, msg_line_plural, line_count),
        //                line_count, op, amount)
        #[allow(clippy::cast_lossless)]
        let n: c_long = line_count as c_long;
        #[allow(clippy::cast_lossless)]
        let amt: c_int = amount;
        // Format: op_char is a byte; pass as a single-char C string
        // We use a 2-byte array [op_char, NUL] as the op string argument.
        let op_str = [op_char, 0u8];
        let op_ptr = op_str.as_ptr().cast::<c_char>();

        let fmt: *const c_char = if line_count == 1 {
            if amount == 1 {
                c"%ld line %sed %d time".as_ptr()
            } else {
                c"%ld line %sed %d times".as_ptr()
            }
        } else if amount == 1 {
            c"%ld lines %sed %d time".as_ptr()
        } else {
            c"%ld lines %sed %d times".as_ptr()
        };
        smsg(0, fmt, n, op_ptr, amt);
    }

    // Set "'[" and "']" marks if LOCKMARKS is not set
    if nvim_cmdmod_has_lockmarks() == 0 {
        nvim_curbuf_set_op_start((*oap_t).start.lnum, (*oap_t).start.col);
        let end_lnum = (*oap_t).end.lnum;
        let mut end_col = ml_get_len(end_lnum);
        if end_col > 0 {
            end_col -= 1;
        }
        nvim_curbuf_set_op_end(end_lnum, end_col);
    }

    changed_lines(
        curbuf,
        (*oap_t).start.lnum,
        0,
        (*oap_t).end.lnum + 1,
        0,
        true,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(OP_LSHIFT, 4);
        assert_eq!(OP_RSHIFT, 5);
        assert_eq!(K_MT_BLOCK_WISE, 2);
        assert_eq!(BL_SOL, 2);
        assert_eq!(BL_FIX, 4);
    }
}
