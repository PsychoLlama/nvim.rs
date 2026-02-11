//! Pattern-in-buffer matching helpers migrated from cmdexpand.c.
//!
//! - `is_regex_match`: Check if a string matches a regex pattern
//! - `concat_pattern_with_buffer_match`: Concatenate pattern with buffer text
//! - `copy_substring_from_pos`: Copy a substring spanning buffer positions

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

use std::ffi::c_void;

use libc::{c_char, c_int};

/// Maximum number of subexpressions in a regexp (from `regexp_defs.h`).
const NSUBEXP: usize = 10;

/// `RE_MAGIC` flag for `vim_regcomp` (verified by `_Static_assert` in `cmdexpand.c`).
const RE_MAGIC: c_int = 1;
/// `RE_STRING` flag for `vim_regcomp` (verified by `_Static_assert` in `cmdexpand.c`).
const RE_STRING: c_int = 2;

/// `kOptWopFlagExacttext` (verified by `_Static_assert` in `cmdexpand.c`).
const K_OPT_WOP_FLAG_EXACTTEXT: u32 = 0x08;

/// OK return value for C functions.
const OK: c_int = 1;
/// FAIL return value for C functions.
const FAIL: c_int = 0;

// =============================================================================
// Struct definitions
// =============================================================================

/// Structure matching `regmatch_T` for single-line matching.
/// Must match the C layout exactly (same as `eval/indexing.rs`).
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

/// Position in file or buffer (matches `pos_T`).
/// Verified by `_Static_assert` in `cmdexpand.c`:
///   `sizeof(pos_T)` == 12, lnum@0, col@4, coladd@8
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PosT {
    pub lnum: i32,
    pub col: i32,
    pub coladd: i32,
}

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    // Regex
    fn vim_regcomp(pat: *const c_char, flags: c_int) -> *mut c_void;
    fn vim_regexec_nl(rmp: *mut RegMatch, line: *const c_char, col: c_int) -> c_int;
    fn vim_regfree(prog: *mut c_void);

    // From search crate
    fn rs_pat_has_uppercase(pat: *const c_char) -> c_int;

    // From insexpand crate
    fn rs_find_word_end(ptr: *mut c_char) -> *mut c_char;

    // Error/message suppression
    fn nvim_cmdexpand_emsg_off_inc();
    fn nvim_cmdexpand_emsg_off_dec();
    fn nvim_cmdexpand_msg_silent_inc();
    fn nvim_cmdexpand_msg_silent_dec();

    // Option accessors
    fn nvim_cmdexpand_get_p_ic() -> c_int;
    fn nvim_cmdexpand_get_p_scs() -> c_int;
    fn nvim_get_wop_flags() -> libc::c_uint;

    // Buffer line access
    fn nvim_cmdexpand_ml_get(lnum: i32) -> *const c_char;
    fn nvim_cmdexpand_ml_get_len(lnum: i32) -> c_int;

    // Memory
    fn xmalloc(size: usize) -> *mut c_char;
    fn xfree(ptr: *mut c_void);
    fn xstrnsave(s: *const c_char, len: usize) -> *mut c_char;

    // String case conversion
    fn strcase_save(orig: *const c_char, upper: bool) -> *mut c_char;
}

// =============================================================================
// Pattern matching helpers
// =============================================================================

/// Check if a string matches a regex pattern, honoring 'ignorecase'/'smartcase'.
///
/// # Safety
///
/// `pat` and `str_` must be valid null-terminated C strings.
unsafe fn is_regex_match(pat: *const c_char, str_: *const c_char) -> bool {
    // Fast path: exact string match
    if libc::strcmp(pat, str_) == 0 {
        return true;
    }

    let mut regmatch = RegMatch::default();

    nvim_cmdexpand_emsg_off_inc();
    nvim_cmdexpand_msg_silent_inc();
    regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
    nvim_cmdexpand_emsg_off_dec();
    nvim_cmdexpand_msg_silent_dec();

    if regmatch.regprog.is_null() {
        return false;
    }

    let p_ic = nvim_cmdexpand_get_p_ic() != 0;
    let p_scs = nvim_cmdexpand_get_p_scs() != 0;

    regmatch.rm_ic = p_ic;
    if p_ic && p_scs {
        regmatch.rm_ic = rs_pat_has_uppercase(pat) == 0;
    }

    nvim_cmdexpand_emsg_off_inc();
    nvim_cmdexpand_msg_silent_inc();
    let result = vim_regexec_nl(std::ptr::addr_of_mut!(regmatch), str_, 0) != 0;
    nvim_cmdexpand_emsg_off_dec();
    nvim_cmdexpand_msg_silent_dec();

    vim_regfree(regmatch.regprog);
    result
}

/// Concatenate pattern with text from the buffer at a position.
///
/// Returns a newly `xmalloc`'d string: `pat[0..pat_len] + word_at(end_match_pos)`.
/// If `lowercase` is true, the appended text is lowercased.
///
/// # Safety
///
/// `pat` must be valid for `pat_len` bytes. `end_match_pos` must be a valid `pos_T` pointer.
unsafe fn concat_pattern_with_buffer_match(
    pat: *const c_char,
    pat_len: c_int,
    end_match_pos: *const PosT,
    lowercase: bool,
) -> *mut c_char {
    let pos = &*end_match_pos;
    let line = nvim_cmdexpand_ml_get(pos.lnum);
    let line_at_col = line.add(pos.col as usize);
    let word_end = rs_find_word_end(line_at_col.cast_mut());
    let match_len = word_end.offset_from(line_at_col) as usize;
    let pat_len_u = pat_len as usize;

    let result = xmalloc(match_len + pat_len_u + 1);

    // Copy pattern prefix
    std::ptr::copy_nonoverlapping(pat, result, pat_len_u);

    if match_len > 0 {
        if lowercase {
            let mword = xstrnsave(line_at_col, match_len);
            let lower = strcase_save(mword, false);
            xfree(mword.cast::<c_void>());
            std::ptr::copy_nonoverlapping(lower.cast_const(), result.add(pat_len_u), match_len);
            xfree(lower.cast::<c_void>());
        } else {
            std::ptr::copy_nonoverlapping(line_at_col, result.add(pat_len_u), match_len);
        }
    }

    *result.add(pat_len_u + match_len) = 0; // NUL terminate
    result
}

/// Copy a substring from the current buffer spanning from `start` to word
/// boundary after `end`. The result is stored in `*match_out` (`xmalloc`'d) and
/// the actual end position in `*match_end`.
///
/// Uses a Rust `Vec<u8>` internally instead of C `garray_T`.
///
/// # Safety
///
/// All pointer parameters must be valid. `match_out` will be set to an `xmalloc`'d buffer.
unsafe fn copy_substring_from_pos(
    start: *mut PosT,
    end: *mut PosT,
    match_out: *mut *mut c_char,
    match_end: *mut PosT,
) -> c_int {
    let start_pos = &*start;
    let end_pos = &*end;
    let wop_flags = nvim_get_wop_flags();
    let exacttext = (wop_flags & K_OPT_WOP_FLAG_EXACTTEXT) != 0;

    if start_pos.lnum > end_pos.lnum
        || (start_pos.lnum == end_pos.lnum && start_pos.col >= end_pos.col)
    {
        return FAIL;
    }

    let mut buf: Vec<u8> = Vec::with_capacity(128);
    let is_single_line = start_pos.lnum == end_pos.lnum;

    // Append start line from start->col to end of line (or end->col if single line)
    let start_line = nvim_cmdexpand_ml_get(start_pos.lnum);
    let start_ptr = start_line.add(start_pos.col as usize);
    let segment_len = if is_single_line {
        (end_pos.col - start_pos.col) as usize
    } else {
        (nvim_cmdexpand_ml_get_len(start_pos.lnum) - start_pos.col) as usize
    };
    let segment = std::slice::from_raw_parts(start_ptr.cast::<u8>(), segment_len);
    buf.extend_from_slice(segment);
    if !is_single_line {
        if exacttext {
            buf.extend_from_slice(b"\\n");
        } else {
            buf.push(b'\n');
        }
    }

    // Append full lines between start and end
    if !is_single_line {
        let mut lnum = start_pos.lnum + 1;
        while lnum < end_pos.lnum {
            let line = nvim_cmdexpand_ml_get(lnum);
            let line_len = nvim_cmdexpand_ml_get_len(lnum) as usize;
            let line_slice = std::slice::from_raw_parts(line.cast::<u8>(), line_len);
            buf.extend_from_slice(line_slice);
            if exacttext {
                buf.extend_from_slice(b"\\n");
            } else {
                buf.push(b'\n');
            }
            lnum += 1;
        }
    }

    // Append partial end line (up to word end)
    let end_line = nvim_cmdexpand_ml_get(end_pos.lnum);
    let end_col_ptr = end_line.add(end_pos.col as usize);
    let word_end = rs_find_word_end(end_col_ptr.cast_mut());
    let word_end_offset = word_end.offset_from(end_line) as usize;

    let copy_start = if is_single_line {
        end_pos.col as usize
    } else {
        0
    };
    let copy_len = word_end_offset - copy_start;
    let end_slice = std::slice::from_raw_parts(end_line.add(copy_start).cast::<u8>(), copy_len);
    buf.extend_from_slice(end_slice);

    // NUL-terminate and copy into xmalloc'd buffer
    buf.push(0);
    let result = xmalloc(buf.len());
    std::ptr::copy_nonoverlapping(buf.as_ptr(), result.cast::<u8>(), buf.len());

    *match_out = result;
    (*match_end).lnum = end_pos.lnum;
    (*match_end).col = word_end_offset as i32;

    OK
}

// =============================================================================
// FFI Interface
// =============================================================================

/// FFI entry point for `is_regex_match`.
///
/// # Safety
///
/// `pat` and `str_` must be valid null-terminated C strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_is_regex_match(pat: *const c_char, str_: *const c_char) -> c_int {
    c_int::from(is_regex_match(pat, str_))
}

/// FFI entry point for `concat_pattern_with_buffer_match`.
///
/// # Safety
///
/// `pat` must be valid for `pat_len` bytes. `end_match_pos` must be a valid `pos_T` pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_concat_pattern_with_buffer_match(
    pat: *const c_char,
    pat_len: c_int,
    end_match_pos: *const PosT,
    lowercase: c_int,
) -> *mut c_char {
    concat_pattern_with_buffer_match(pat, pat_len, end_match_pos, lowercase != 0)
}

/// FFI entry point for `copy_substring_from_pos`.
///
/// # Safety
///
/// All pointer parameters must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_copy_substring_from_pos(
    start: *mut PosT,
    end: *mut PosT,
    match_out: *mut *mut c_char,
    match_end: *mut PosT,
) -> c_int {
    copy_substring_from_pos(start, end, match_out, match_end)
}
