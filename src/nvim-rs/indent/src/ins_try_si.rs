//! Implementation of `ins_try_si()` — smart indenting for '{', '}', '#'.

use std::ffi::{c_char, c_int};

use crate::getters::{rs_get_indent, rs_inindent};
use crate::set_indent::{rs_set_indent, SIN_CHANGED};

type LinenrT = i32;
type ColnrT = i32;

const NUL: c_char = 0;
const VREPLACE_FLAG: c_int = 0x200;
const INDENT_SET: c_int = 1;

/// Position in buffer (line, column, coladd).
#[repr(C)]
#[derive(Clone, Copy)]
struct PosT {
    lnum: LinenrT,
    col: ColnrT,
    coladd: ColnrT,
}

#[inline]
unsafe fn ascii_iswhite(c: c_char) -> bool {
    c == b' ' as c_char || c == b'\t' as c_char
}

// C accessor functions
extern "C" {
    static mut State: c_int;
    // Existing accessors (change_ffi.c, window.c, etc.)
    fn nvim_get_did_si() -> bool;
    fn nvim_get_can_si() -> bool;
    fn nvim_get_can_si_back() -> bool;
    fn nvim_get_ai_col() -> ColnrT;
    fn nvim_set_ai_col(val: ColnrT);
    fn nvim_ml_get(lnum: LinenrT) -> *mut c_char;
    fn nvim_skipwhite(s: *const c_char) -> *mut c_char;
    fn nvim_findmatch(initc: *mut c_char, ch: c_char) -> *mut PosT;
    fn nvim_change_get_curwin_cursor() -> PosT;
    fn nvim_set_curwin_cursor(pos: PosT);
    fn nvim_get_curwin_cursor_col() -> ColnrT;
    fn nvim_get_curwin_cursor_lnum() -> LinenrT;
    fn nvim_set_curwin_cursor_lnum(lnum: LinenrT);

    // indent_ffi.c accessors (new for this phase)
    fn nvim_shift_line(left: bool, round: bool, amount: c_int, call_changed_bytes: c_int);
    fn nvim_set_old_indent(val: c_int);
    fn nvim_change_indent(typ: c_int, amount: c_int, round: c_int, call_changed_bytes: bool);
}

/// Do some very smart indenting when entering '{', '}', or '#'.
///
/// # Safety
/// - Accesses global editor state (current window, buffer, cursor).
/// - Single-threaded (Neovim guarantee).
#[export_name = "ins_try_si"]
pub unsafe extern "C" fn rs_ins_try_si(c: c_int) {
    let did_si = nvim_get_did_si();
    let can_si = nvim_get_can_si();
    let can_si_back = nvim_get_can_si_back();

    // do some very smart indenting when entering '{' or '}'
    if ((did_si || can_si_back) && c == b'{' as c_int)
        || (can_si && c == b'}' as c_int && rs_inindent(0))
    {
        // for '}' set indent equal to indent of line containing matching '{'
        let pos = nvim_findmatch(std::ptr::null_mut(), b'{' as c_char);
        if c == b'}' as c_int && !pos.is_null() {
            let old_pos = nvim_change_get_curwin_cursor();
            // If the matching '{' has a ')' immediately before it (ignoring
            // white-space), then line up with the start of the line
            // containing the matching '(' if there is one.  This handles the
            // case where an "if (..\n..) {" statement continues over multiple
            // lines -- webb
            let ptr = nvim_ml_get((*pos).lnum);
            let mut i = (*pos).col;
            if i > 0 {
                // skip blanks before '{'
                i -= 1;
                while i > 0 && ascii_iswhite(*ptr.offset(i as isize)) {
                    i -= 1;
                }
            }
            nvim_set_curwin_cursor(PosT {
                lnum: (*pos).lnum,
                col: i,
                coladd: 0,
            });
            if *ptr.offset(i as isize) == b')' as c_char {
                let pos2 = nvim_findmatch(std::ptr::null_mut(), b'(' as c_char);
                if !pos2.is_null() {
                    nvim_set_curwin_cursor(*pos2);
                }
            }
            let indent = rs_get_indent();
            nvim_set_curwin_cursor(old_pos);
            if State & VREPLACE_FLAG != 0 {
                nvim_change_indent(INDENT_SET, indent, 0, true);
            } else {
                let _ = rs_set_indent(indent, SIN_CHANGED);
            }
        } else if nvim_get_curwin_cursor_col() > 0 {
            // when inserting '{' after "O" reduce indent, but not
            // more than indent of previous line
            let mut temp = true;
            if c == b'{' as c_int && can_si_back && nvim_get_curwin_cursor_lnum() > 1 {
                let old_pos = nvim_change_get_curwin_cursor();
                let cur_indent = rs_get_indent();
                let mut lnum = nvim_get_curwin_cursor_lnum();
                while lnum > 1 {
                    lnum -= 1;
                    nvim_set_curwin_cursor_lnum(lnum);
                    let line_ptr = nvim_skipwhite(nvim_ml_get(lnum));

                    // ignore empty lines and lines starting with '#'.
                    if *line_ptr != b'#' as c_char && *line_ptr != NUL {
                        break;
                    }
                }
                if rs_get_indent() >= cur_indent {
                    temp = false;
                }
                nvim_set_curwin_cursor(old_pos);
            }
            if temp {
                nvim_shift_line(true, false, 1, 1);
            }
        }
    }

    // set indent of '#' always to 0
    if nvim_get_curwin_cursor_col() > 0 && can_si && c == b'#' as c_int && rs_inindent(0) {
        // remember current indent for next line
        nvim_set_old_indent(rs_get_indent());
        let _ = rs_set_indent(0, SIN_CHANGED);
    }

    // Adjust ai_col, the char at this position can be deleted.
    let ai_col = nvim_get_ai_col();
    let cursor_col = nvim_get_curwin_cursor_col();
    if cursor_col < ai_col {
        nvim_set_ai_col(cursor_col);
    }
}
