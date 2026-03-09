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

use std::ffi::{c_char, c_int};
use std::io::Write;

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

    // -- redo_literal dependencies --
    fn nvim_edit_AppendToRedobuff(s: *const c_char);
    fn nvim_edit_append_char_to_redobuff(c: c_int);

    // -- do_insert_char_pre dependencies --
    fn nvim_edit_has_event_insertcharpre() -> c_int;
    fn nvim_edit_textlock_inc();
    fn nvim_edit_textlock_dec();
    fn nvim_edit_set_vim_var_char(buf: *const c_char, len: isize);
    fn nvim_edit_get_vim_var_char() -> *const c_char;
    fn nvim_edit_ins_apply_autocmds_insertcharpre() -> c_int;
    fn nvim_edit_set_State(val: c_int);
    fn nvim_get_State() -> c_int;
    fn utf_char2bytes(c: c_int, buf: *mut u8) -> c_int;
    fn xstrdup(s: *const c_char) -> *mut c_char;

    // -- insert_special dependencies --
    fn nvim_edit_get_mod_mask() -> c_int;
    fn nvim_edit_set_mod_mask(val: c_int);
    fn nvim_edit_get_special_key_name(c: c_int, modifiers: c_int) -> *mut c_char;
    fn nvim_edit_ins_str(p: *const c_char, len: usize);
    fn nvim_edit_AppendToRedobuffLit(s: *const c_char, len: c_int);
    fn nvim_edit_insertchar(c: c_int, flags: c_int, second_indent: c_int);
    fn rs_stop_arrow() -> c_int;

    // -- get_literal dependencies --
    fn nvim_edit_plain_vgetc() -> c_int;
    fn nvim_edit_merge_modifiers(c: c_int) -> c_int;
    fn nvim_edit_add_to_showcmd(c: c_int);
    fn nvim_edit_MB_BYTE2LEN_CHECK(c: c_int) -> c_int;
    fn nvim_edit_vungetc(c: c_int);
    fn nvim_edit_inc_no_mapping();
    fn nvim_edit_dec_no_mapping();
    fn nvim_edit_get_got_int() -> c_int;
    fn nvim_edit_set_got_int(val: c_int);
    fn nvim_edit_get_K_ZERO() -> c_int;
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

/// `Ctrl_RSB` from `ascii_defs.h`
const CTRL_RSB: c_int = 29;

/// `MB_MAXBYTES` from `mbyte_defs.h`
const MB_MAXBYTES: usize = 21;

/// `MOD_MASK_CMD` from `keycodes.h`
const MOD_MASK_CMD: c_int = 0x80;

/// `MOD_MASK_SHIFT` from `keycodes.h`
const MOD_MASK_SHIFT: c_int = 0x02;

/// `INSCHAR_CTRLV` from `edit.h`
const INSCHAR_CTRLV: c_int = 4;

/// `MODE_CMDLINE` from `state_defs.h`
const MODE_CMDLINE: c_int = 0x08;

/// `Ctrl_C` from `ascii_defs.h`
const CTRL_C: c_int = 3;

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

#[must_use]
#[unsafe(export_name = "ins_eol")]
pub unsafe extern "C" fn rs_ins_eol(c: c_int) -> bool {
    nvim_edit_ins_eol(c) != 0
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
// redo_literal — encode literal character into redo buffer
// ============================================================================

/// Put a character in the redo buffer, for when just after a CTRL-V.
/// Digits are encoded as a 3-digit decimal string to avoid ambiguity.
unsafe fn redo_literal_impl(c: c_int) {
    if (c as u8).is_ascii_digit() {
        let mut buf = [0u8; 10];
        let _ = write!(&mut buf.as_mut_slice()[..], "{c:03}");
        // Find NUL terminator position and ensure it
        let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        buf[len] = 0;
        nvim_edit_AppendToRedobuff(buf.as_ptr().cast());
    } else {
        nvim_edit_append_char_to_redobuff(c);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_redo_literal(c: c_int) {
    redo_literal_impl(c);
}

// ============================================================================
// do_insert_char_pre — trigger InsertCharPre autocmd
// ============================================================================

/// Handle the `InsertCharPre` autocommand.
/// `c` is the character that was typed.
/// Returns a pointer to allocated memory with the replacement string,
/// or NULL to continue inserting `c`.
unsafe fn do_insert_char_pre_impl(c: c_int) -> *mut c_char {
    if c == CTRL_RSB {
        return std::ptr::null_mut();
    }

    // Return quickly when there is nothing to do.
    if nvim_edit_has_event_insertcharpre() == 0 {
        return std::ptr::null_mut();
    }

    let mut buf = [0u8; MB_MAXBYTES + 1];
    let buflen = utf_char2bytes(c, buf.as_mut_ptr()) as usize;
    buf[buflen] = 0; // NUL-terminate

    let save_state = nvim_get_State();

    // Lock the text to avoid weird things from happening.
    nvim_edit_textlock_inc();
    nvim_edit_set_vim_var_char(buf.as_ptr().cast(), buflen as isize);

    let mut res: *mut c_char = std::ptr::null_mut();
    if nvim_edit_ins_apply_autocmds_insertcharpre() != 0 {
        // Get the value of v:char. Only use it when changed.
        let vchar = nvim_edit_get_vim_var_char();
        // Compare buf (our original) with v:char
        if !vchar.is_null() {
            let orig = buf.as_ptr().cast::<c_char>();
            if libc_strcmp(orig, vchar) != 0 {
                res = xstrdup(vchar);
            }
        }
    }

    nvim_edit_set_vim_var_char(std::ptr::null(), -1);
    nvim_edit_textlock_dec();

    // Restore the State, it may have been changed.
    nvim_edit_set_State(save_state);

    res
}

/// Simple strcmp implementation to avoid libc dependency.
unsafe fn libc_strcmp(a: *const c_char, b: *const c_char) -> c_int {
    let mut i = 0;
    loop {
        let ca = *a.offset(i) as u8;
        let cb = *b.offset(i) as u8;
        if ca != cb {
            return c_int::from(ca) - c_int::from(cb);
        }
        if ca == 0 {
            return 0;
        }
        i += 1;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_do_insert_char_pre(c: c_int) -> *mut c_char {
    do_insert_char_pre_impl(c)
}

// ============================================================================
// insert_special — handle special key insertion with modifiers
// ============================================================================

/// Insert character, taking care of special keys and `mod_mask`.
///
/// `allow_modmask`: if true, use `mod_mask` for non-special keys too.
/// `ctrlv`: if true, `c` was typed after CTRL-V.
unsafe fn insert_special_impl(mut c: c_int, mut allow_modmask: c_int, mut ctrlv: c_int) {
    let mod_mask = nvim_edit_get_mod_mask();

    // Command-key never produces a normal key.
    if mod_mask & MOD_MASK_CMD != 0 {
        allow_modmask = 1;
    }
    // IS_SPECIAL(c) is (c < 0)
    if c < 0 || (mod_mask != 0 && allow_modmask != 0) {
        let p = nvim_edit_get_special_key_name(c, mod_mask);
        let len = c_strlen(p);
        c = c_int::from(*p.add(len - 1) as u8);
        if len > 2 {
            if rs_stop_arrow() == FAIL {
                return;
            }
            // Temporarily NUL-terminate before the last char
            let saved = *p.add(len - 1);
            *p.add(len - 1) = 0;
            nvim_edit_ins_str(p.cast_const(), len - 1);
            nvim_edit_AppendToRedobuffLit(p.cast_const(), -1);
            *p.add(len - 1) = saved;
            ctrlv = 0;
        }
    }
    if rs_stop_arrow() == OK {
        nvim_edit_insertchar(c, if ctrlv != 0 { INSCHAR_CTRLV } else { 0 }, -1);
    }
}

/// Compute the length of a NUL-terminated C string.
unsafe fn c_strlen(p: *mut c_char) -> usize {
    let mut len = 0;
    while *p.add(len) != 0 {
        len += 1;
    }
    len
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_insert_special(c: c_int, allow_modmask: c_int, ctrlv: c_int) {
    insert_special_impl(c, allow_modmask, ctrlv);
}

// ============================================================================
// get_literal — CTRL-V literal character input
// ============================================================================

/// Convert a hex digit character to its numeric value.
fn hex2nr(c: c_int) -> c_int {
    let ch = c as u8;
    if ch.is_ascii_digit() {
        c_int::from(ch - b'0')
    } else if (b'a'..=b'f').contains(&ch) {
        c_int::from(ch - b'a' + 10)
    } else if (b'A'..=b'F').contains(&ch) {
        c_int::from(ch - b'A' + 10)
    } else {
        0
    }
}

/// Next character is interpreted literally.
/// A one, two or three digit decimal number is interpreted as its byte value.
/// If one or two digits are entered, the next character is given to `vungetc()`.
/// For Unicode a character > 255 may be returned.
unsafe fn get_literal_impl(no_simplify: c_int) -> c_int {
    let mut nc: c_int;
    let mut hex = false;
    let mut octal = false;
    let mut unicode: c_int = 0;

    if nvim_edit_get_got_int() != 0 {
        return CTRL_C;
    }

    nvim_edit_inc_no_mapping(); // don't map the next key hits
    let mut cc: c_int = 0;
    let mut i: c_int = 0;
    loop {
        nc = nvim_edit_plain_vgetc();
        if no_simplify == 0 {
            nc = nvim_edit_merge_modifiers(nc);
        }
        let mod_mask = nvim_edit_get_mod_mask();
        if (mod_mask & !MOD_MASK_SHIFT) != 0 {
            // A character with non-Shift modifiers should not be a valid
            // character for i_CTRL-V_digit.
            break;
        }
        let state = nvim_get_State();
        if (state & MODE_CMDLINE) == 0 && nvim_edit_MB_BYTE2LEN_CHECK(nc) == 1 {
            nvim_edit_add_to_showcmd(nc);
        }
        if nc == c_int::from(b'x') || nc == c_int::from(b'X') {
            hex = true;
        } else if nc == c_int::from(b'o') || nc == c_int::from(b'O') {
            octal = true;
        } else if nc == c_int::from(b'u') || nc == c_int::from(b'U') {
            unicode = nc;
        } else {
            if hex || unicode != 0 {
                if !(nc as u8).is_ascii_hexdigit() {
                    break;
                }
                cc = cc * 16 + hex2nr(nc);
            } else if octal {
                if nc < c_int::from(b'0') || nc > c_int::from(b'7') {
                    break;
                }
                cc = cc * 8 + nc - c_int::from(b'0');
            } else {
                if !(nc as u8).is_ascii_digit() {
                    break;
                }
                cc = cc * 10 + nc - c_int::from(b'0');
            }

            i += 1;
        }

        if cc > 255 && unicode == 0 {
            cc = 255; // limit range to 0-255
        }
        nc = 0;

        if hex {
            // hex: up to two chars
            if i >= 2 {
                break;
            }
        } else if unicode != 0 {
            // Unicode: up to four or eight chars
            if (unicode == c_int::from(b'u') && i >= 4) || (unicode == c_int::from(b'U') && i >= 8)
            {
                break;
            }
        } else if i >= 3 {
            // decimal or octal: up to three chars
            break;
        }
    }
    if i == 0 {
        // no number entered
        let k_zero = nvim_edit_get_K_ZERO();
        if nc == k_zero {
            // NUL is stored as NL
            cc = c_int::from(b'\n');
        } else {
            cc = nc;
        }
        nc = 0;
    }

    if cc == 0 {
        // NUL is stored as NL
        cc = c_int::from(b'\n');
    }

    nvim_edit_dec_no_mapping();
    if nc != 0 {
        nvim_edit_vungetc(nc);
        // A character typed with i_CTRL-V_digit cannot have modifiers.
        nvim_edit_set_mod_mask(0);
    }
    nvim_edit_set_got_int(0); // CTRL-C typed after CTRL-V is not an interrupt
    cc
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_get_literal(no_simplify: c_int) -> c_int {
    get_literal_impl(no_simplify)
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
