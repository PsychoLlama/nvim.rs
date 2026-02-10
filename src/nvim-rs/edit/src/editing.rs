//! Medium-complexity editing functions migrated from edit.c
//!
//! These handle Enter/NL insertion, Ctrl-V literal input, Ctrl-E/Y
//! copy-from-line, digraph input, and `stuff_inserted` for redo.
//!
//! Most functions delegate to C helper wrappers due to heavy dependencies
//! on UI, charsize, digraph, and format systems. `stuff_inserted` is
//! implemented in Rust using the already-migrated `get_last_insert`.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_safety_doc)]

use std::ffi::c_int;

/// Line number type (matches `linenr_T` in Neovim).
type LinenrT = i32;

// ============================================================================
// C accessor / helper functions
// ============================================================================

extern "C" {
    // -- Delegated wrappers for complex functions --
    fn nvim_edit_ins_eol(c: c_int) -> c_int;
    fn nvim_edit_ins_ctrl_v();
    fn nvim_edit_ins_copychar(lnum: LinenrT) -> c_int;
    fn nvim_edit_ins_ctrl_ey(tc: c_int) -> c_int;
    fn nvim_edit_ins_digraph() -> c_int;

    // -- stuff_inserted dependencies --
    fn rs_get_last_insert() -> NvimString;
    fn nvim_stuffcharReadbuff(c: c_int);
    fn nvim_stuffReadbuffLen(data: *const u8, len: isize);
    fn nvim_emsg_noinstext();
}

// ============================================================================
// Constants (verified against C headers with `_Static_assert` in `edit.c`)
// ============================================================================

/// `OK` from `vim_defs.h`
const OK: c_int = 1;

/// `FAIL` from `vim_defs.h`
const FAIL: c_int = 0;

/// NUL byte
const NUL: u8 = 0;

/// ESC from `ascii_defs.h`
const ESC: u8 = 0x1b;

/// `Ctrl_D` from `ascii_defs.h`
const CTRL_D: u8 = 4;

// ============================================================================
// NvimString (matches helpers.rs definition)
// ============================================================================

/// FFI-compatible String type matching Neovim's `String`.
#[repr(C)]
struct NvimString {
    data: *mut u8,
    size: usize,
}

// ============================================================================
// ins_eol — delegated to C helper
// ============================================================================

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_ins_eol(c: c_int) -> c_int {
    nvim_edit_ins_eol(c)
}

// ============================================================================
// ins_ctrl_v — delegated to C helper
// ============================================================================

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_ins_ctrl_v() {
    nvim_edit_ins_ctrl_v();
}

// ============================================================================
// ins_copychar — delegated to C helper
// ============================================================================

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_ins_copychar(lnum: LinenrT) -> c_int {
    nvim_edit_ins_copychar(lnum)
}

// ============================================================================
// ins_ctrl_ey — delegated to C helper
// ============================================================================

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_ins_ctrl_ey(tc: c_int) -> c_int {
    nvim_edit_ins_ctrl_ey(tc)
}

// ============================================================================
// ins_digraph — delegated to C helper
// ============================================================================

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_ins_digraph() -> c_int {
    nvim_edit_ins_digraph()
}

// ============================================================================
// stuff_inserted — implemented in Rust
// ============================================================================

/// Stuff the last inserted text into the redo buffer.
///
/// `c` is the command character to stuff first (NUL for none).
/// `count` is how many times to repeat the insert.
/// `no_esc` if true, don't append ESC at the end.
///
/// Returns OK or FAIL.
unsafe fn stuff_inserted_impl(c: c_int, count: c_int, no_esc: c_int) -> c_int {
    let insert = rs_get_last_insert();
    if insert.data.is_null() {
        nvim_emsg_noinstext();
        return FAIL;
    }

    // May want to stuff the command character, to start Insert mode
    if c != 0 {
        nvim_stuffcharReadbuff(c);
    }

    let data = insert.data;
    let mut size = insert.size;

    if size > 0 {
        // Look for the last ESC in 'insert' and truncate there
        let mut i = size;
        while i > 0 {
            i -= 1;
            if *data.add(i) == ESC {
                size = i;
                break;
            }
        }
    }

    let mut last: u8 = NUL;
    if size > 0 {
        let p = *data.add(size - 1);
        // When the last char is either "0" or "^" it will be quoted if no ESC
        // comes after it OR if it will be inserted more than once and "ptr"
        // starts with ^D. -- Acevedo
        if (p == b'0' || p == b'^') && (no_esc != 0 || (*data == CTRL_D && count > 1)) {
            last = p;
            size -= 1;
        }
    }

    let mut remaining = count;
    loop {
        nvim_stuffReadbuffLen(data, size as isize);
        // A trailing "0" is inserted as "<C-V>048", "^" as "<C-V>^".
        match last {
            b'0' => {
                // "\026\060\064\070" = Ctrl-V 0 4 8
                let seq: &[u8] = b"\x16\x30\x34\x38";
                nvim_stuffReadbuffLen(seq.as_ptr(), seq.len() as isize);
            }
            b'^' => {
                // "\026^" = Ctrl-V ^
                let seq: &[u8] = b"\x16^";
                nvim_stuffReadbuffLen(seq.as_ptr(), seq.len() as isize);
            }
            _ => {}
        }
        remaining -= 1;
        if remaining <= 0 {
            break;
        }
    }

    // May want to stuff a trailing ESC, to get out of Insert mode
    if no_esc == 0 {
        nvim_stuffcharReadbuff(c_int::from(ESC));
    }

    OK
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_stuff_inserted(c: c_int, count: c_int, no_esc: c_int) -> c_int {
    stuff_inserted_impl(c, count, no_esc)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(OK, 1);
        assert_eq!(FAIL, 0);
        assert_eq!(NUL, 0);
        assert_eq!(ESC, 0x1b);
        assert_eq!(CTRL_D, 4);
    }

    #[test]
    fn test_nvim_string_layout() {
        assert_eq!(
            std::mem::size_of::<NvimString>(),
            std::mem::size_of::<*mut u8>() + std::mem::size_of::<usize>()
        );
    }
}
