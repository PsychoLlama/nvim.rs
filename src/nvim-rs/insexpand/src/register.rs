//! Register completion support.
//!
//! This module provides Rust implementation for register-based completion
//! (CTRL-X CTRL-R). Iterates all registers, extracts words, and adds as
//! completion matches via ins_compl_add_infercase.

use std::os::raw::{c_char, c_int};

const FUZZY_SCORE_NONE: c_int = -1;
const FORWARD: c_int = 1;
#[allow(clippy::cast_possible_wrap)]
const BLACK_HOLE_REGISTER: c_char = b'_' as c_char;

// C accessor functions
extern "C" {
    // Register access (from normal_shim.c)
    fn nvim_valid_yank_reg(regname: c_int, writing: bool) -> bool;
    fn nvim_put_copy_register(regname: c_int) -> *mut std::ffi::c_void;
    fn nvim_put_free_register(reg: *mut std::ffi::c_void);

    // Register field accessors (from insexpand_shim.c)
    // nvim_get_num_registers: deleted (Phase 31, use NUM_REGISTERS = 39 constant)
    fn nvim_yankreg_y_size(reg: *mut std::ffi::c_void) -> usize;
    fn nvim_yankreg_y_array_null(reg: *mut std::ffi::c_void) -> c_int;
    fn nvim_yankreg_y_array_entry_data(reg: *mut std::ffi::c_void, j: usize) -> *const c_char;

    // ins_compl_add_infercase wrapper
    fn nvim_ins_compl_add_infercase_ffi(
        str_: *const c_char,
        len: c_int,
        icase: c_int,
        fname: *const c_char,
        dir: c_int,
        cont_s_ipos: c_int,
        score: c_int,
    ) -> c_int;

    // Completion state accessors
    fn rs_compl_status_adding() -> c_int;
    // nvim_get_p_ic: inlined in vars.rs (Phase 28)
    fn nvim_vim_strnicmp(s1: *const c_char, s2: *const c_char, len: usize) -> c_int;

    // Word boundary helpers (from insexpand_shim.c)
    fn rs_find_word_start(ptr: *mut c_char) -> *mut c_char;
    fn rs_find_word_end(ptr: *mut c_char) -> *mut c_char;
    fn utfc_ptr2len(ptr: *const c_char) -> c_int;

    // get_register_name is rs_get_register_name (from nvim-rs/register crate)
    fn rs_get_register_name(num: c_int) -> c_int;
}

/// Check if a string (with length) matches the completion orig_text prefix.
///
/// Returns true if there is no orig_text, or if str[..orig_size] matches
/// orig_text (case-insensitive if p_ic is set).
#[inline]
unsafe fn matches_orig_text(str_: *const c_char, _str_len: usize) -> bool {
    let orig_data = crate::vars::nvim_get_compl_orig_text_data();
    if orig_data.is_null() {
        return true;
    }
    let orig_size = crate::vars::nvim_get_compl_orig_text_size();
    if crate::vars::nvim_get_p_ic() != 0 {
        nvim_vim_strnicmp(str_, orig_data, orig_size) == 0
    } else {
        // strncmp equivalent
        let s1 = std::slice::from_raw_parts(str_.cast::<u8>(), orig_size);
        let s2 = std::slice::from_raw_parts(orig_data.cast::<u8>(), orig_size);
        s1 == s2
    }
}

/// NUM_REGISTERS = 39 (register_defs.h)
const NUM_REGISTERS: c_int = 39;

/// Perform register-based completion.
///
/// Iterates all named registers and adds their contents as completion matches.
/// For each register entry, extracts individual words (unless in adding mode,
/// where the whole entry string is used) and calls ins_compl_add_infercase.
///
/// # Safety
/// Requires valid completion state (compl_orig_text, compl_direction, etc.)
#[no_mangle]
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
pub unsafe extern "C" fn rs_get_register_completion() {
    let mut dir = crate::vars::nvim_get_compl_direction();
    let adding_mode = rs_compl_status_adding() != 0;
    let p_ic = crate::vars::nvim_get_p_ic() != 0;

    for i in 0..NUM_REGISTERS {
        let regname = rs_get_register_name(i);

        // Skip invalid or black hole register
        if !nvim_valid_yank_reg(regname, false) || regname == c_int::from(BLACK_HOLE_REGISTER) {
            continue;
        }

        let reg = nvim_put_copy_register(regname);

        if nvim_yankreg_y_array_null(reg) != 0 || nvim_yankreg_y_size(reg) == 0 {
            nvim_put_free_register(reg);
            continue;
        }

        let y_size = nvim_yankreg_y_size(reg);

        for j in 0..y_size {
            let str_ = nvim_yankreg_y_array_entry_data(reg, j);
            if str_.is_null() {
                continue;
            }

            if adding_mode {
                // In adding mode: add the entire entry string
                let str_len = strlen_c(str_) as c_int;
                if str_len == 0 {
                    continue;
                }

                if matches_orig_text(str_, str_len as usize) {
                    let r = nvim_ins_compl_add_infercase_ffi(
                        str_,
                        str_len,
                        c_int::from(p_ic),
                        std::ptr::null(),
                        dir,
                        0,
                        FUZZY_SCORE_NONE,
                    );
                    if r == 0 {
                        // OK
                        dir = FORWARD;
                    }
                }
            } else {
                // Normal mode: extract individual words from the entry
                let str_mut = str_.cast_mut();
                let str_end = str_mut.add(strlen_c(str_));
                let mut p = str_mut;

                while p < str_end && *p != 0 {
                    let old_p = p;
                    p = rs_find_word_start(p);

                    if p >= str_end || *p == 0 {
                        break;
                    }

                    let word_end = rs_find_word_end(p);

                    // word_end must be > p
                    let word_end = if word_end <= p {
                        p.add(utfc_ptr2len(p) as usize)
                    } else {
                        word_end
                    };

                    let word_end = if word_end > str_end {
                        str_end
                    } else {
                        word_end
                    };

                    let len = word_end.offset_from(p) as c_int;
                    if len > 0 && matches_orig_text(p.cast_const(), len as usize) {
                        let r = nvim_ins_compl_add_infercase_ffi(
                            p.cast_const(),
                            len,
                            c_int::from(p_ic),
                            std::ptr::null(),
                            dir,
                            0,
                            FUZZY_SCORE_NONE,
                        );
                        if r == 0 {
                            // OK
                            dir = FORWARD;
                        }
                    }

                    p = word_end;

                    // Safety: avoid infinite loop if advance didn't happen
                    if p <= old_p {
                        p = old_p.add(utfc_ptr2len(old_p) as usize);
                    }
                }
            }
        }

        nvim_put_free_register(reg);
    }
}

/// Compute `strlen` of a NUL-terminated C string.
///
/// # Safety
/// `s` must point to a valid NUL-terminated C string.
#[inline]
#[allow(clippy::missing_const_for_fn, clippy::cast_sign_loss)]
unsafe fn strlen_c(s: *const c_char) -> usize {
    let mut p = s;
    while *p != 0 {
        p = p.add(1);
    }
    p.offset_from(s) as usize
}
