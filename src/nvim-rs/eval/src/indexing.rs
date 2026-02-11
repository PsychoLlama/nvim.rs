//! Buffer index conversion and pattern matching utilities migrated from eval.c.
//!
//! - `buf_byteidx_to_charidx`: Byte→char index in buffer line
//! - `buf_charidx_to_byteidx`: Char→byte index in buffer line
//! - `pattern_match`: Regex pattern match wrapper

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr,
    clippy::borrow_as_ptr
)]

use std::ffi::{c_char, c_int, c_void};

/// Maximum number of subexpressions in a regexp (from regexp_defs.h)
const NSUBEXP: usize = 10;

/// RE_MAGIC flag for vim_regcomp (verified by _Static_assert in eval.c)
const RE_MAGIC: c_int = 1;
/// RE_STRING flag for vim_regcomp (verified by _Static_assert in eval.c)
const RE_STRING: c_int = 2;

/// Opaque buffer handle (buf_T*)
type BufHandle = *mut c_void;

extern "C" {
    fn rs_utfc_ptr2len(p: *const c_char) -> c_int;

    // Buffer accessors (defined in eval.c)
    fn nvim_eval_buf_ml_valid(buf: BufHandle) -> c_int;
    fn nvim_eval_buf_line_count(buf: BufHandle) -> i32;
    fn nvim_eval_ml_get_buf(buf: BufHandle, lnum: i32) -> *const c_char;

    // Regex functions
    fn vim_regcomp(pat: *const c_char, flags: c_int) -> *mut c_void;
    fn vim_regexec_nl(rmp: *mut RegMatch, line: *const c_char, col: c_int) -> c_int;
    fn vim_regfree(prog: *mut c_void);

    // p_cpo save/restore accessors (defined in eval.c)
    fn nvim_eval_save_set_cpo();
    fn nvim_eval_restore_cpo();
}

/// Structure matching regmatch_T for single-line matching.
/// Must match the C layout exactly.
#[repr(C)]
struct RegMatch {
    regprog: *mut c_void,
    startp: [*mut c_char; NSUBEXP],
    endp: [*mut c_char; NSUBEXP],
    rm_matchcol: c_int,
    rm_ic: bool,
}

impl Default for RegMatch {
    fn default() -> Self {
        Self {
            regprog: std::ptr::null_mut(),
            startp: [std::ptr::null_mut(); NSUBEXP],
            endp: [std::ptr::null_mut(); NSUBEXP],
            rm_matchcol: 0,
            rm_ic: false,
        }
    }
}

/// Convert the specified byte index of line `lnum` in buffer `buf` to a
/// character index. Works only for loaded buffers.
///
/// Returns -1 on failure.
///
/// # Safety
///
/// `buf` must be a valid buffer handle or null.
#[no_mangle]
pub unsafe extern "C" fn rs_buf_byteidx_to_charidx(
    buf: BufHandle,
    mut lnum: i32,
    byteidx: c_int,
) -> c_int {
    if buf.is_null() || nvim_eval_buf_ml_valid(buf) == 0 {
        return -1;
    }

    let line_count = nvim_eval_buf_line_count(buf);
    if lnum > line_count {
        lnum = line_count;
    }

    let str = nvim_eval_ml_get_buf(buf, lnum);

    if *str == 0 {
        return 0;
    }

    // count the number of characters
    let mut t = str;
    let mut count: c_int = 0;
    while *t != 0 && t <= str.add(byteidx as usize) {
        t = t.add(rs_utfc_ptr2len(t) as usize);
        count += 1;
    }

    // In insert mode, when the cursor is at the end of a non-empty line,
    // byteidx points to the NUL character immediately past the end of the
    // string. In this case, add one to the character count.
    if *t == 0 && byteidx != 0 && t == str.add(byteidx as usize) {
        count += 1;
    }

    count - 1
}

/// Convert the specified character index of line `lnum` in buffer `buf` to a
/// byte index. Works only for loaded buffers.
///
/// Returns -1 on failure.
///
/// # Safety
///
/// `buf` must be a valid buffer handle or null.
#[no_mangle]
pub unsafe extern "C" fn rs_buf_charidx_to_byteidx(
    buf: BufHandle,
    mut lnum: i32,
    mut charidx: c_int,
) -> c_int {
    if buf.is_null() || nvim_eval_buf_ml_valid(buf) == 0 {
        return -1;
    }

    let line_count = nvim_eval_buf_line_count(buf);
    if lnum > line_count {
        lnum = line_count;
    }

    let str = nvim_eval_ml_get_buf(buf, lnum);

    // Convert the character offset to a byte offset
    let mut t = str;
    charidx -= 1;
    while *t != 0 && charidx > 0 {
        t = t.add(rs_utfc_ptr2len(t) as usize);
        charidx -= 1;
    }

    t.offset_from(str) as c_int
}

/// Check if `pat` matches `text`.
///
/// Does not use 'cpo' and always uses 'magic'.
///
/// # Safety
///
/// `pat` and `text` must be valid null-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_pattern_match(
    pat: *const c_char,
    text: *const c_char,
    ic: bool,
) -> c_int {
    let mut matches: c_int = 0;

    // avoid 'l' flag in 'cpoptions'
    nvim_eval_save_set_cpo();

    let prog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
    if !prog.is_null() {
        let mut regmatch = RegMatch {
            regprog: prog,
            rm_ic: ic,
            ..RegMatch::default()
        };
        matches = vim_regexec_nl(&raw mut regmatch, text, 0);
        vim_regfree(regmatch.regprog);
    }

    nvim_eval_restore_cpo();
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(RE_MAGIC, 1);
        assert_eq!(RE_STRING, 2);
        assert_eq!(NSUBEXP, 10);
    }
}
