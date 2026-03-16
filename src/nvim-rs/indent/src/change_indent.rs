//! Implementation of `change_indent()` — insert-mode indent change with
//! VREPLACE support.

use std::ffi::{c_char, c_int, c_void};

use crate::getters::rs_get_indent;
use crate::set_indent::{rs_set_indent, SIN_CHANGED};

type LinenrT = i32;
type ColnrT = i32;
type BufHandle = *mut c_void;

const NUL: c_char = 0;
const BL_WHITE: c_int = 1;
const MODE_INSERT: c_int = 0x10;
const REPLACE_FLAG: c_int = 0x100;
const VREPLACE_FLAG: c_int = 0x200;
const MAXCOL: c_int = 0x7fffffff;
const INDENT_SET: c_int = 1;
const INDENT_DEC: c_int = 3;
const KEXTMARK_UNDO: c_int = 1;

#[inline]
fn replace_normal(state: c_int) -> bool {
    (state & REPLACE_FLAG) != 0 && (state & VREPLACE_FLAG) == 0
}

// C accessor functions
extern "C" {
    // State
    fn nvim_get_State() -> c_int;
    fn nvim_set_State(val: c_int);

    // Cursor
    fn nvim_get_curwin_cursor_lnum() -> LinenrT;
    fn nvim_get_curwin_cursor_col() -> ColnrT;
    fn nvim_set_curwin_cursor_col(col: ColnrT);

    // Insstart
    fn nvim_get_Insstart_lnum() -> LinenrT;
    fn nvim_get_Insstart_col() -> ColnrT;
    fn nvim_set_Insstart_col(val: ColnrT);

    // ai_col
    fn nvim_get_ai_col() -> ColnrT;
    fn nvim_set_ai_col(val: ColnrT);

    // curwin properties
    fn nvim_curwin_w_p_list() -> c_int;
    fn nvim_curwin_set_w_p_list(val: c_int);
    fn nvim_curwin_set_w_set_curswant(val: bool);
    fn nvim_curwin_set_w_virtcol(val: ColnrT);

    // Cursor line operations
    fn nvim_get_cursor_line_ptr() -> *mut c_char;
    fn nvim_get_cursor_line_len() -> c_int;
    fn nvim_getvcol_nolist() -> ColnrT;

    // Navigation
    fn nvim_beginline(flags: c_int);

    // Indentation
    fn nvim_shift_line(left: bool, round: bool, amount: c_int, call_changed_bytes: c_int);

    // Display
    fn nvim_changed_cline_bef_curs(wp: *mut c_void);

    // Memory
    fn nvim_xstrnsave(s: *const c_char, len: usize) -> *mut c_char;
    fn nvim_xfree(ptr: *mut c_void);

    // Replace mode
    fn nvim_replace_push_nul();
    fn nvim_ins_bytes(p: *const c_char);
    fn nvim_ins_str(ptr: *mut c_char, len: usize);

    // ml_replace for cursor line
    fn nvim_ml_replace_curline(line: *mut c_char, copy: bool) -> c_int;

    // curwin handle (for changed_cline_bef_curs)
    fn nvim_get_curwin() -> *mut c_void;

    // Extmark splice
    fn nvim_extmark_splice_cols(
        buf: BufHandle,
        start_row: c_int,
        start_col: ColnrT,
        old_col: ColnrT,
        new_col: ColnrT,
        undo: c_int,
    );
    static mut curbuf_splice_pending: c_int;
    fn nvim_indent_get_curbuf() -> BufHandle;

    // Higher-level cursor advance
    fn nvim_advance_to_vcol(line: *mut c_char, target_vcol: c_int, out_vcol: *mut c_int) -> c_int;

    // xmallocz
    fn nvim_xmallocz(size: usize) -> *mut c_char;

    // Rust functions in edit crate (via FFI)
    fn replace_join(off: c_int);
    fn backspace_until_column(col: c_int);
}

/// Change the indent of the current line and adjust cursor position.
///
/// # Safety
/// - Accesses global editor state (current window, buffer, cursor).
/// - Single-threaded (Neovim guarantee).
#[export_name = "change_indent"]
pub unsafe extern "C" fn rs_change_indent(
    type_: c_int,
    amount: c_int,
    round: c_int,
    call_changed_bytes: bool,
) {
    let mut orig_col: ColnrT = 0;
    let mut orig_line: *mut c_char = std::ptr::null_mut();

    let state = nvim_get_State();

    // MODE_VREPLACE state needs to know what the line was like before changing
    if state & VREPLACE_FLAG != 0 {
        orig_line = nvim_xstrnsave(
            nvim_get_cursor_line_ptr(),
            nvim_get_cursor_line_len() as usize,
        );
        orig_col = nvim_get_curwin_cursor_col();
    }

    // for the following tricks we don't want list mode
    let save_p_list = nvim_curwin_w_p_list();
    nvim_curwin_set_w_p_list(0);
    let vc = nvim_getvcol_nolist();
    let mut vcol: c_int = vc;

    // For Replace mode we need to fix the replace stack later, which is only
    // possible when the cursor is in the indent. Remember the number of
    // characters before the cursor if it's possible.
    let mut start_col: c_int = nvim_get_curwin_cursor_col();

    // determine offset from first non-blank
    let mut new_cursor_col: c_int = nvim_get_curwin_cursor_col();
    nvim_beginline(BL_WHITE);
    new_cursor_col -= nvim_get_curwin_cursor_col();

    let insstart_less_before = nvim_get_curwin_cursor_col();

    // If the cursor is in the indent, compute how many screen columns the
    // cursor is to the left of the first non-blank.
    if new_cursor_col < 0 {
        vcol = rs_get_indent() - vcol;
    }

    if new_cursor_col > 0 {
        // can't fix replace stack
        start_col = -1;
    }

    // Set the new indent. The cursor will be put on the first non-blank.
    if type_ == INDENT_SET {
        let _ = rs_set_indent(amount, if call_changed_bytes { SIN_CHANGED } else { 0 });
    } else {
        let save_state = nvim_get_State();

        // Avoid being called recursively.
        if save_state & VREPLACE_FLAG != 0 {
            nvim_set_State(MODE_INSERT);
        }
        nvim_shift_line(
            type_ == INDENT_DEC,
            round != 0,
            1,
            call_changed_bytes as c_int,
        );
        nvim_set_State(save_state);
    }
    let mut insstart_less: c_int = insstart_less_before - nvim_get_curwin_cursor_col();

    // Try to put cursor on same character.
    if new_cursor_col >= 0 {
        // When changing the indent while the cursor is touching it, reset
        // Insstart_col to 0.
        if new_cursor_col == 0 {
            insstart_less = MAXCOL;
        }
        new_cursor_col += nvim_get_curwin_cursor_col();
    } else if state & MODE_INSERT == 0 {
        new_cursor_col = nvim_get_curwin_cursor_col();
    } else {
        // Compute the screen column where the cursor should be.
        vcol = rs_get_indent() - vcol;
        let end_vcol: c_int = if vcol < 0 { 0 } else { vcol };
        nvim_curwin_set_w_virtcol(end_vcol);

        // Advance the cursor until we reach the right screen column.
        let line = nvim_get_cursor_line_ptr();
        let mut actual_vcol: c_int = 0;
        new_cursor_col = nvim_advance_to_vcol(line, end_vcol, &mut actual_vcol);

        // May need to insert spaces to be able to position the cursor on
        // the right screen column.
        if actual_vcol != end_vcol {
            nvim_set_curwin_cursor_col(new_cursor_col);
            let ptrlen = (end_vcol - actual_vcol) as usize;
            let ptr = nvim_xmallocz(ptrlen);
            std::ptr::write_bytes(ptr as *mut u8, b' ', ptrlen);
            new_cursor_col += ptrlen as c_int;
            nvim_ins_str(ptr, ptrlen);
            nvim_xfree(ptr.cast());
        }

        // When changing the indent while the cursor is in it, reset
        // Insstart_col to 0.
        insstart_less = MAXCOL;
    }

    nvim_curwin_set_w_p_list(save_p_list);

    let final_col = if new_cursor_col < 0 {
        0
    } else {
        new_cursor_col
    };
    nvim_set_curwin_cursor_col(final_col);
    nvim_curwin_set_w_set_curswant(true);
    nvim_changed_cline_bef_curs(nvim_get_curwin());

    // May have to adjust the start of the insert.
    if state & MODE_INSERT != 0 {
        if nvim_get_curwin_cursor_lnum() == nvim_get_Insstart_lnum() && nvim_get_Insstart_col() != 0
        {
            if nvim_get_Insstart_col() as c_int <= insstart_less {
                nvim_set_Insstart_col(0);
            } else {
                nvim_set_Insstart_col(nvim_get_Insstart_col() - insstart_less as ColnrT);
            }
        }
        let ai_col = nvim_get_ai_col();
        if ai_col as c_int <= insstart_less {
            nvim_set_ai_col(0);
        } else {
            nvim_set_ai_col(ai_col - insstart_less as ColnrT);
        }
    }

    // For MODE_REPLACE state, may have to fix the replace stack.
    if replace_normal(state) && start_col >= 0 {
        let cursor_col = nvim_get_curwin_cursor_col() as c_int;
        while start_col > cursor_col {
            replace_join(0); // remove a NUL from the replace stack
            start_col -= 1;
        }
        while start_col < cursor_col {
            nvim_replace_push_nul();
            start_col += 1;
        }
    }

    // For MODE_VREPLACE state, we also have to fix the replace stack.
    if state & VREPLACE_FLAG != 0 {
        // Save new line
        let new_line = nvim_xstrnsave(
            nvim_get_cursor_line_ptr(),
            nvim_get_cursor_line_len() as usize,
        );

        // We only put back the new line up to the cursor
        let new_col = nvim_get_curwin_cursor_col();
        *new_line.offset(new_col as isize) = NUL;

        // Put back original line
        nvim_ml_replace_curline(orig_line, false);
        nvim_set_curwin_cursor_col(orig_col);

        let pending = curbuf_splice_pending;
        curbuf_splice_pending = pending + 1;

        // Backspace from cursor to start of line
        backspace_until_column(0);

        // Insert new stuff into line again
        nvim_ins_bytes(new_line);

        nvim_xfree(new_line.cast());

        curbuf_splice_pending = pending;

        let delta = orig_col - new_col;
        let curbuf = nvim_indent_get_curbuf();
        nvim_extmark_splice_cols(
            curbuf,
            nvim_get_curwin_cursor_lnum() - 1,
            new_col,
            if delta < 0 { -delta } else { 0 },
            if delta > 0 { delta } else { 0 },
            KEXTMARK_UNDO,
        );
    }
}
