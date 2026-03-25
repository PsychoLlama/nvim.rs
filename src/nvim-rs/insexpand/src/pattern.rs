//! Completion pattern helpers.
//!
//! This module provides Rust implementations for computing the pattern,
//! column, and length for various completion modes (normal, whole-line,
//! filename, spell). The heavy C string manipulation is done via compound
//! C accessors; Rust provides the dispatch and extern declarations.

#![allow(
    dead_code,
    unused_imports,
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clashing_extern_declarations
)]
use std::os::raw::{c_char, c_int};

use crate::vars::NvimString;

// Compound C accessors that implement the core pattern-building logic.
// Each calls the original C subsystem functions internally.
extern "C" {
    // nvim_get_normal_compl_info_impl: deleted (Phase 2), inlined below as rs_get_normal_compl_info
    // nvim_get_wholeline_compl_info_impl: deleted (Phase 2), inlined below as rs_get_wholeline_compl_info
    fn getwhitecols(p: *const c_char) -> isize;
    // nvim_get_filename_compl_info_impl: deleted (Phase 28), inlined below as rs_get_filename_compl_info
    // nvim_get_spell_compl_info_impl: deleted (Phase 26), inlined below
    // helpers for inlined rs_get_spell_compl_info (Phase 26)
    fn spell_word_start(startcol: c_int) -> c_int;
    fn spell_expand_check_cap(col: c_int);

    // nvim_set_compl_globals_impl: deleted (Phase 20), inlined below as set_compl_globals

    // helpers for inlined nvim_set_compl_globals_impl (Phase 20)
    pub(crate) static mut cpt_compl_pattern: NvimString;
    fn copy_string(str_: NvimString, arena: *mut core::ffi::c_void) -> NvimString;
    #[link_name = "xfree"]
    fn xfree_pattern(p: *mut c_char);
    fn memmove(dst: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
    fn nvim_ml_get_curline() -> *const c_char;

    // Underlying C functions used by inlined rs_get_normal_compl_info
    fn vim_isIDc(c: c_int) -> bool;
    fn vim_iswordp(p: *const c_char) -> bool;
    fn mb_prevptr(line: *mut c_char, p: *mut c_char) -> *mut c_char;
    fn utf_head_off(base: *const c_char, p: *const c_char) -> c_int;
    fn mb_get_class(ptr: *const c_char) -> c_int;
    fn cbuf_to_string(buf: *const c_char, size: usize) -> NvimString;
    fn cstr_as_string(str_: *const c_char) -> NvimString;
    fn utf_ptr2char(p: *const c_char) -> c_int;
    #[link_name = "vim_isfilec"]
    fn vim_isfilec_pat(c: c_int) -> bool;
    fn addstar(fname: *mut c_char, len: usize, context: c_int) -> *mut c_char;
    fn str_foldcase(
        str_: *mut c_char,
        orglen: c_int,
        buf: *mut c_char,
        buflen: c_int,
    ) -> *mut c_char;
    #[link_name = "xmalloc"]
    fn xmalloc_pattern(size: usize) -> *mut c_char;
    fn strcat(s1: *mut c_char, s2: *const c_char) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
    fn rs_ctrl_x_mode_normal() -> c_int;
    fn rs_ctrl_x_mode_path_defines() -> c_int;
    fn rs_compl_status_adding() -> c_int;
    fn rs_quote_meta(dest: *mut c_char, src: *mut c_char, len: c_int) -> c_int;
    fn rs_setup_cpt_sources();
    fn rs_prepare_cpt_compl_funcs();
}

const OK: c_int = 1;
const CONT_SOL: c_int = 16;
const EXPAND_FILES: c_int = 2;
const CONT_LOCAL: c_int = 32;

/// Get the pattern, column and length for normal (keyword) completion.
///
/// Sets compl_col, compl_length, compl_pattern, and compl_from_nonkeyword.
/// Also calls setup_cpt_sources/prepare_cpt_compl_funcs for normal CTRL-N.
///
/// Rust translation of the C `nvim_get_normal_compl_info_impl` compound accessor.
///
/// # Safety
/// Requires valid global state; line must be a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_get_normal_compl_info(
    line: *mut c_char,
    mut startcol: c_int,
    curs_col: c_int,
) -> c_int {
    let cont_status = crate::vars::nvim_get_compl_cont_status();

    if (cont_status & CONT_SOL) != 0 || rs_ctrl_x_mode_path_defines() != 0 {
        if rs_compl_status_adding() == 0 {
            // Scan backwards over identifier characters
            loop {
                startcol -= 1;
                if startcol < 0 {
                    break;
                }
                if !vim_isIDc(c_int::from(*line.add(startcol as usize) as u8)) {
                    break;
                }
            }
            startcol += 1;
            crate::vars::nvim_set_compl_col(crate::vars::nvim_get_compl_col() + startcol);
            crate::vars::nvim_set_compl_length(curs_col - startcol);
        }
        let col = crate::vars::nvim_get_compl_col() as usize;
        let len = crate::vars::nvim_get_compl_length();
        if crate::vars::nvim_get_p_ic() != 0 {
            crate::vars::compl_pattern =
                cstr_as_string(str_foldcase(line.add(col), len, core::ptr::null_mut(), 0));
        } else {
            #[allow(clippy::cast_sign_loss)]
            let usz = len as usize;
            crate::vars::compl_pattern = cbuf_to_string(line.add(col), usz);
        }
    } else if rs_compl_status_adding() != 0 {
        let col = crate::vars::nvim_get_compl_col() as usize;
        let len = crate::vars::nvim_get_compl_length();

        // Choose prefix based on whether we're at a word boundary
        let (prefix, prefixlen): (*const c_char, usize) = if !vim_iswordp(line.add(col))
            || (col > 0 && vim_iswordp(mb_prevptr(line, line.add(col))))
        {
            (c"".as_ptr(), 0usize)
        } else {
            (c"\\<".as_ptr(), 2usize)
        };

        // rs_quote_meta with NULL dest counts chars needed (returns count + 1 for NUL)
        #[allow(clippy::cast_sign_loss)]
        let quoted = rs_quote_meta(core::ptr::null_mut(), line.add(col), len) as usize;
        let n = quoted + prefixlen;
        let buf = xmalloc_pattern(n);
        // Ensure NUL so strcat works, then copy prefix and append quoted
        *buf = 0;
        strcat(buf, prefix);
        rs_quote_meta(buf.add(prefixlen), line.add(col), len);
        crate::vars::compl_pattern = NvimString {
            data: buf,
            size: n - 1,
        };
    } else {
        startcol -= 1;
        if startcol < 0 || !vim_iswordp(mb_prevptr(line, line.add((startcol + 1) as usize))) {
            // Match any word of at least two chars
            crate::vars::compl_pattern = cbuf_to_string(c"\\<\\k\\k".as_ptr(), 6);
            crate::vars::nvim_set_compl_col(crate::vars::nvim_get_compl_col() + curs_col);
            crate::vars::nvim_set_compl_length(0);
            crate::vars::nvim_set_compl_from_nonkeyword(1);
        } else {
            // Scan backwards to find start of word/char-class boundary
            #[allow(clippy::cast_sign_loss)]
            {
                startcol -=
                    utf_head_off(line.cast_const(), line.add(startcol as usize).cast_const());
            }
            let base_class = mb_get_class(line.add(startcol as usize).cast_const());
            while startcol > 0 {
                startcol -= 1;
                let head_off =
                    utf_head_off(line.cast_const(), line.add(startcol as usize).cast_const());
                if base_class != mb_get_class(line.add((startcol - head_off) as usize).cast_const())
                {
                    break;
                }
                startcol -= head_off;
            }

            startcol += 1;
            crate::vars::nvim_set_compl_col(crate::vars::nvim_get_compl_col() + startcol);
            crate::vars::nvim_set_compl_length(curs_col - startcol);

            let col = crate::vars::nvim_get_compl_col() as usize;
            let len = crate::vars::nvim_get_compl_length();

            if len == 1 {
                // Only match word with at least two chars -- webb
                let buf = xmalloc_pattern(7);
                *buf = b'\\' as c_char;
                *buf.add(1) = b'<' as c_char;
                rs_quote_meta(buf.add(2), line.add(col), 1);
                strcat(buf, c"\\k".as_ptr());
                let slen = strlen(buf.cast_const());
                crate::vars::compl_pattern = NvimString {
                    data: buf,
                    size: slen,
                };
            } else {
                #[allow(clippy::cast_sign_loss)]
                let quoted = rs_quote_meta(core::ptr::null_mut(), line.add(col), len) as usize;
                let n = quoted + 2;
                let buf = xmalloc_pattern(n);
                *buf = b'\\' as c_char;
                *buf.add(1) = b'<' as c_char;
                rs_quote_meta(buf.add(2), line.add(col), len);
                crate::vars::compl_pattern = NvimString {
                    data: buf,
                    size: n - 1,
                };
            }
        }
    }

    // Call functions in 'complete' with 'findstart=1'
    if rs_ctrl_x_mode_normal() != 0 && (cont_status & CONT_LOCAL) == 0 {
        rs_setup_cpt_sources();
        rs_prepare_cpt_compl_funcs();
    }

    OK
}

/// Get the pattern, column and length for whole-line completion.
///
/// Sets compl_col, compl_length, compl_pattern.
///
/// Rust translation of the C `nvim_get_wholeline_compl_info_impl` compound accessor.
///
/// # Safety
/// Requires valid global state; line must be a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_get_wholeline_compl_info(line: *mut c_char, curs_col: c_int) -> c_int {
    let wcols = getwhitecols(line.cast_const()) as c_int;
    crate::vars::nvim_set_compl_col(wcols);
    let mut len = curs_col - wcols;
    if len < 0 {
        len = 0;
    }
    crate::vars::nvim_set_compl_length(len);
    let col = crate::vars::nvim_get_compl_col() as usize;
    if crate::vars::nvim_get_p_ic() != 0 {
        crate::vars::compl_pattern =
            cstr_as_string(str_foldcase(line.add(col), len, core::ptr::null_mut(), 0));
    } else {
        crate::vars::compl_pattern = cbuf_to_string(line.add(col), len as usize);
    }
    OK
}

/// Get the pattern, column and length for filename completion.
///
/// Sets compl_col, compl_length, compl_pattern.
///
/// Rust translation of nvim_get_filename_compl_info_impl (Phase 28).
///
/// # Safety
/// Requires valid global state; line must be a valid C string.
#[no_mangle]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_get_filename_compl_info(
    line: *mut c_char,
    mut startcol: c_int,
    curs_col: c_int,
) -> c_int {
    // Go back to just before the first filename character.
    if startcol > 0 {
        let mut p = line.add(startcol as usize);
        p = mb_prevptr(line, p);
        while p > line && vim_isfilec_pat(utf_ptr2char(p.cast_const())) {
            p = mb_prevptr(line, p);
        }
        let p_is_filec = vim_isfilec_pat(utf_ptr2char(p.cast_const()));
        if p == line && p_is_filec {
            startcol = 0;
        } else {
            // SAFETY: p is within the same allocation as line
            startcol = p.offset_from(line) as c_int + 1;
        }
    }

    crate::vars::nvim_set_compl_col(crate::vars::nvim_get_compl_col() + startcol);
    crate::vars::nvim_set_compl_length(curs_col - startcol);
    #[allow(clippy::cast_sign_loss)]
    let col = crate::vars::nvim_get_compl_col() as usize;
    #[allow(clippy::cast_sign_loss)]
    let len = crate::vars::nvim_get_compl_length() as usize;
    crate::vars::compl_pattern = cstr_as_string(addstar(line.add(col), len, EXPAND_FILES));
    OK
}

/// Get the pattern, column and length for spell completion.
///
/// Sets compl_col, compl_length, compl_pattern.
///
/// Rust translation of nvim_get_spell_compl_info_impl (Phase 26).
///
/// # Safety
/// Requires valid global state.
#[no_mangle]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_get_spell_compl_info(startcol: c_int, curs_col: c_int) -> c_int {
    let spell_len = crate::vars::nvim_get_spell_bad_len();
    if spell_len > 0 {
        crate::vars::nvim_set_compl_col(curs_col - spell_len as c_int);
    } else {
        crate::vars::nvim_set_compl_col(spell_word_start(startcol));
    }
    if crate::vars::nvim_get_compl_col() >= startcol {
        crate::vars::nvim_set_compl_length(0);
        crate::vars::nvim_set_compl_col(curs_col);
    } else {
        spell_expand_check_cap(crate::vars::nvim_get_compl_col());
        crate::vars::nvim_set_compl_length(curs_col - crate::vars::nvim_get_compl_col());
    }
    // Re-fetch line (may have become invalid)
    let line = nvim_ml_get_curline();
    #[allow(clippy::cast_sign_loss)]
    let col = crate::vars::nvim_get_compl_col() as usize;
    #[allow(clippy::cast_sign_loss)]
    let len = crate::vars::nvim_get_compl_length() as usize;
    crate::vars::compl_pattern = cbuf_to_string(line.add(col), len);
    OK
}

/// Set global variables related to completion.
///
/// Sets `compl_col`, `compl_length`, `compl_pattern`, and `cpt_compl_pattern`
/// based on the mode (`is_cpt_compl != 0` for cpt function completion).
///
/// Inlined from deleted C `nvim_set_compl_globals_impl` (Phase 20).
///
/// # Safety
/// Requires valid global completion state. Mutates C static globals.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_set_compl_globals(
    mut startcol: c_int,
    curs_col: c_int,
    is_cpt_compl: c_int,
) {
    if is_cpt_compl != 0 {
        // API_CLEAR_STRING(cpt_compl_pattern)
        if !cpt_compl_pattern.data.is_null() {
            xfree_pattern(cpt_compl_pattern.data);
            cpt_compl_pattern.data = core::ptr::null_mut();
            cpt_compl_pattern.size = 0;
        }
        let compl_col = crate::vars::nvim_get_compl_col();
        if startcol < compl_col {
            let prepend_len = (compl_col - startcol) as usize;
            let orig_size = core::ptr::read(&raw const crate::vars::compl_orig_text.size);
            let orig_data = core::ptr::read(&raw const crate::vars::compl_orig_text.data);
            let new_length = prepend_len + orig_size;
            cpt_compl_pattern.size = new_length;
            cpt_compl_pattern.data = xmalloc_pattern(new_length + 1);
            let line = nvim_ml_get_curline();
            memmove(
                cpt_compl_pattern.data,
                line.add(startcol as usize),
                prepend_len,
            );
            memmove(
                cpt_compl_pattern.data.add(prepend_len),
                orig_data.cast_const(),
                orig_size,
            );
            *cpt_compl_pattern.data.add(new_length) = 0;
        } else {
            // copy_string does a heap-dup of the String struct
            cpt_compl_pattern = copy_string(
                core::ptr::read(&raw const crate::vars::compl_orig_text),
                core::ptr::null_mut(),
            );
        }
    } else {
        if startcol < 0 || startcol > curs_col {
            startcol = curs_col;
        }
        let line = nvim_ml_get_curline();
        let len = curs_col - startcol;
        crate::vars::compl_pattern = cbuf_to_string(line.add(startcol as usize), len as usize);
        crate::vars::nvim_set_compl_col(startcol);
        crate::vars::nvim_set_compl_length(len);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_declarations_exist() {
        // Verify the module compiles and FFI declarations are present.
        // Actual function calls require a running Neovim session.
        let _: unsafe extern "C" fn(*mut c_char, c_int, c_int) -> c_int = rs_get_normal_compl_info;
    }
}
