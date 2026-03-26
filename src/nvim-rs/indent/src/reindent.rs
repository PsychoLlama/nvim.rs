//! Implementation of `op_reindent()` — block reindent with function pointer.

use std::ffi::{c_char, c_int};

use crate::set_indent::rs_set_indent;
use nvim_normal::types::OpargT;

type LinenrT = i32;
type ColnrT = i32;

const NUL: c_char = 0;
const BL_SOL: c_int = 2;
const BL_FIX: c_int = 4;
const UPD_INVERTED: c_int = 20;

/// Typed pointer to oparg_T.
type OapHandle = *mut OpargT;

/// Function pointer type for indenter functions (matching C `Indenter`).
type Indenter = unsafe extern "C" fn() -> c_int;

// C accessor functions
extern "C" {
    static mut got_int: bool;
    // Existing accessors
    fn nvim_get_curwin_cursor_lnum() -> LinenrT;
    fn nvim_set_curwin_cursor_lnum(lnum: LinenrT);
    fn nvim_set_curwin_cursor_col(col: ColnrT);
    fn skipwhite(s: *const c_char) -> *mut c_char;
    fn nvim_get_cursor_line_ptr() -> *mut c_char;
    fn nvim_beginline(flags: c_int);

    // indent_ffi.c accessors (new for this phase)
    fn nvim_curbuf_is_modifiable() -> bool;
    fn nvim_emsg_modifiable();
    fn nvim_u_savecommon_range(start: LinenrT, count: LinenrT) -> c_int;
    fn nvim_smsg_lines_to_indent(i: i64);
    fn nvim_smsg_lines_indented(count: i64);
    fn nvim_get_p_report() -> i64;
    fn nvim_get_cmdmod_lockmarks() -> bool;
    fn redraw_curbuf_later(typ: c_int);

    // Buffer change notification
    fn nvim_indent_changed_lines(first: LinenrT, last: LinenrT, xtra: LinenrT);

    // oparg_T field accessors (oap_is_visual and oap_set_marks remain in indent_ffi.c)
    fn nvim_oap_is_visual(oap: OapHandle) -> bool;
    fn nvim_oap_set_marks(oap: OapHandle);

    // Function pointer comparison
    fn nvim_is_lisp_indent(how: Indenter) -> bool;
}

/// OK result from u_savecommon.
const OK: c_int = 1;

/// Reindent a range of lines using the given indent function.
///
/// # Safety
/// - `oap` must be a valid oparg_T handle.
/// - `how` must be a valid function pointer.
/// - Accesses global editor state (single-threaded).
#[export_name = "op_reindent"]
pub unsafe extern "C" fn rs_op_reindent(oap: OapHandle, how: Indenter) {
    let start_lnum = nvim_get_curwin_cursor_lnum();
    let line_count = (*oap).line_count;

    // Don't even try when 'modifiable' is off.
    if !nvim_curbuf_is_modifiable() {
        nvim_emsg_modifiable();
        return;
    }

    let mut i: LinenrT = 0;
    let mut first_changed: LinenrT = 0;
    let mut last_changed: LinenrT = 0;

    // Save for undo.
    if nvim_u_savecommon_range(start_lnum, line_count) == OK {
        let p_report = nvim_get_p_report();
        let is_lisp = nvim_is_lisp_indent(how);

        i = line_count - 1;
        while i >= 0 && !unsafe { got_int } {
            // Give feedback for slow operations.
            if i > 1 && (i % 50 == 0 || i == line_count - 1) && i64::from(line_count) > p_report {
                nvim_smsg_lines_to_indent(i64::from(i));
            }

            // Be vi-compatible: For lisp indenting the first line is not
            // indented, unless there is only one line.
            if i != line_count - 1 || line_count == 1 || !is_lisp {
                let l = skipwhite(nvim_get_cursor_line_ptr());
                let amount = if *l == NUL {
                    // empty or blank line
                    0
                } else {
                    how() // get the indent for this line
                };
                if amount >= 0 && rs_set_indent(amount, 0) {
                    // did change the indent, call changed_lines() later
                    if first_changed == 0 {
                        first_changed = nvim_get_curwin_cursor_lnum();
                    }
                    last_changed = nvim_get_curwin_cursor_lnum();
                }
            }
            let lnum = nvim_get_curwin_cursor_lnum();
            nvim_set_curwin_cursor_lnum(lnum + 1);
            nvim_set_curwin_cursor_col(0); // make sure it's valid

            i -= 1;
        }
    }

    // put cursor on first non-blank of indented line
    nvim_set_curwin_cursor_lnum(start_lnum);
    nvim_beginline(BL_SOL | BL_FIX);

    // Mark changed lines so that they will be redrawn.
    if last_changed != 0 {
        let end = if nvim_oap_is_visual(oap) {
            start_lnum + line_count
        } else {
            last_changed + 1
        };
        nvim_indent_changed_lines(first_changed, end, 0);
    } else if nvim_oap_is_visual(oap) {
        redraw_curbuf_later(UPD_INVERTED);
    }

    let p_report = nvim_get_p_report();
    if i64::from(line_count) > p_report {
        let indented = line_count - (i + 1);
        nvim_smsg_lines_indented(i64::from(indented));
    }
    if !nvim_get_cmdmod_lockmarks() {
        // set '[ and '] marks
        nvim_oap_set_marks(oap);
    }
}
