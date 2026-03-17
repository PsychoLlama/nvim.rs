//! Command lookup types and utilities for Ex commands.
//!
//! This module provides types and functions for looking up command names
//! in the command table.

use std::ffi::{c_char, c_int};

extern "C" {
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn nvim_docmd_cmd_exists_inner(
        name: *const c_char,
        out_cmdidx: *mut c_int,
        out_full: *mut c_int,
        out_useridx: *mut c_int,
    ) -> *mut c_char;
    fn nvim_docmd_cmdnames_func_is_ni(cmdidx: c_int) -> c_int;

    // Phase 3: command table accessors
    fn nvim_eap_get_cmd(eap: ExArgHandle) -> *mut c_char;
    fn nvim_eap_get_cmdidx(eap: ExArgHandle) -> c_int;
    fn nvim_eap_set_cmdidx(eap: ExArgHandle, idx: c_int);
    fn nvim_eap_get_flags(eap: ExArgHandle) -> c_int;
    fn nvim_eap_set_flags(eap: ExArgHandle, flags: c_int);
    fn nvim_docmd_get_command_count() -> c_int;
    fn nvim_docmd_get_cmdidxs1(c: c_int) -> c_int;
    fn nvim_docmd_get_cmdidxs2(c1: c_int, c2: c_int) -> c_int;
    fn nvim_docmd_cmdnames_prefix_match(idx: c_int, cmd: *const c_char, len: c_int) -> c_int;
    fn nvim_docmd_cmdnames_name_complete(idx: c_int, len: c_int) -> c_int;
    fn nvim_docmd_cmdnames_name(idx: c_int) -> *mut c_char;
    fn nvim_docmd_find_ucmd(eap: ExArgHandle, p: *mut c_char, full: *mut c_int) -> *mut c_char;
    fn nvim_docmd_expand_user_cmd_name(idx: c_int) -> *mut c_char;
    fn nvim_docmd_e943_abort();

    // Phase 3: f_fullcommand helpers
    fn nvim_docmd_tv_get_string(argvars: *const c_void) -> *mut c_char;
    fn nvim_docmd_rettv_init_string(rettv: *mut c_void);
    fn nvim_docmd_rettv_set_string(rettv: *mut c_void, s: *const c_char);
    fn nvim_docmd_get_user_command_name(useridx: c_int, cmdidx: c_int) -> *mut c_char;
}

use std::ffi::c_void;

use crate::ExArgHandle;

// =============================================================================
// One-letter command helpers
// =============================================================================

/// Check if the character at position is a one-letter command.
///
/// One-letter commands are:
/// - 'k' (mark)
/// - 's' (substitute) followed by non-alpha or the substitute pattern delimiter
///
/// Returns true if it's a one-letter command.
#[inline]
pub fn is_one_letter_cmd_char(c: u8, next: u8) -> bool {
    // 'k' is always a one-letter command (mark)
    if c == b'k' {
        return true;
    }

    // 's' is a one-letter command if followed by specific characters
    if c == b's' {
        // 's' followed by non-alpha, or by a delimiter character
        // is the substitute command
        if !next.is_ascii_alphabetic() {
            return true;
        }
    }

    false
}

/// FFI wrapper for checking one-letter commands.
#[no_mangle]
pub extern "C" fn rs_is_one_letter_cmd_char(c: c_int, next: c_int) -> c_int {
    c_int::from(is_one_letter_cmd_char(c as u8, next as u8))
}

// =============================================================================
// Command name classification
// =============================================================================

/// Check if a character can start a command name.
///
/// Command names can start with:
/// - ASCII letters (a-z, A-Z)
/// - Special characters: @ ! = > < & ~ #
#[inline]
pub const fn is_cmd_name_start(c: u8) -> bool {
    c.is_ascii_alphabetic()
        || c == b'@'
        || c == b'!'
        || c == b'='
        || c == b'>'
        || c == b'<'
        || c == b'&'
        || c == b'~'
        || c == b'#'
}

/// FFI wrapper for command name start check.
#[no_mangle]
pub extern "C" fn rs_is_cmd_name_start(c: c_int) -> c_int {
    c_int::from(is_cmd_name_start(c as u8))
}

/// Check if a character can be part of a command name.
///
/// Command names consist of ASCII letters.
#[inline]
pub const fn is_cmd_name_char(c: u8) -> bool {
    c.is_ascii_alphabetic()
}

/// FFI wrapper for command name character check.
#[no_mangle]
pub extern "C" fn rs_is_cmd_name_char(c: c_int) -> c_int {
    c_int::from(is_cmd_name_char(c as u8))
}

/// Check if this could be a Python command prefix.
///
/// Python commands start with "py" and can be followed by alphanumeric
/// characters (e.g., py3, python3, py3file).
#[inline]
pub fn is_python_cmd_prefix(cmd: &[u8]) -> bool {
    cmd.len() >= 2 && cmd[0] == b'p' && cmd[1] == b'y'
}

/// FFI wrapper for Python command prefix check.
///
/// # Safety
///
/// The pointer must be valid and point to at least 2 bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_is_python_cmd_prefix(cmd: *const c_char) -> c_int {
    if cmd.is_null() {
        return 0;
    }
    let c0 = *cmd as u8;
    let c1 = *cmd.add(1) as u8;
    c_int::from(c0 == b'p' && c1 == b'y')
}

// =============================================================================
// User command detection
// =============================================================================

/// Check if a character can start a user-defined command.
///
/// User commands must start with an uppercase letter (A-Z).
#[inline]
pub const fn is_user_cmd_start(c: u8) -> bool {
    c >= b'A' && c <= b'Z'
}

/// FFI wrapper for user command start check.
#[no_mangle]
pub extern "C" fn rs_is_user_cmd_start(c: c_int) -> c_int {
    c_int::from(is_user_cmd_start(c as u8))
}

/// Check if a character can be part of a user-defined command name.
///
/// User commands can contain letters and digits.
#[inline]
pub const fn is_user_cmd_char(c: u8) -> bool {
    c.is_ascii_alphanumeric()
}

/// FFI wrapper for user command character check.
#[no_mangle]
pub extern "C" fn rs_is_user_cmd_char(c: c_int) -> c_int {
    c_int::from(is_user_cmd_char(c as u8))
}

// =============================================================================
// Command index calculation helpers
// =============================================================================

/// Calculate the ordinal of a lowercase letter (0-25).
///
/// Returns the ordinal value, or 0 if not a lowercase letter.
#[inline]
pub const fn cmd_char_ord_low(c: u8) -> usize {
    if c >= b'a' && c <= b'z' {
        (c - b'a') as usize
    } else {
        0
    }
}

/// FFI wrapper for lowercase character ordinal for command lookup.
#[no_mangle]
pub extern "C" fn rs_cmd_char_ord_low(c: c_int) -> c_int {
    cmd_char_ord_low(c as u8) as c_int
}

// =============================================================================
// checkforcmd - Check command name prefix match
// =============================================================================

/// Check if the string at `*pp` matches the command name `cmd` with
/// at least `len` characters. If so, advance `*pp` past the match
/// and any trailing whitespace.
///
/// Matches C `checkforcmd()`.
///
/// # Safety
///
/// `pp` must point to a valid `*mut c_char` pointer.
/// `cmd` must be a valid null-terminated C string.
#[export_name = "checkforcmd"]
pub unsafe extern "C" fn rs_checkforcmd(
    pp: *mut *mut c_char,
    cmd: *const c_char,
    len: c_int,
) -> bool {
    if pp.is_null() || cmd.is_null() {
        return false;
    }

    let mut i = 0i32;
    loop {
        let c = *cmd.add(i as usize) as u8;
        if c == 0 {
            break;
        }
        if c != *(*pp).add(i as usize) as u8 {
            break;
        }
        i += 1;
    }

    if i >= len && !(*(*pp).add(i as usize) as u8).is_ascii_alphabetic() {
        *pp = skipwhite((*pp).add(i as usize) as *const c_char);
        return true;
    }
    false
}

// =============================================================================
// one_letter_cmd - Full implementation matching C
// =============================================================================

/// Check if the string at `p` starts a one-letter command.
///
/// If so, sets `*idx` to the command index (CMD_k or CMD_substitute)
/// and returns 1. Otherwise returns 0.
///
/// Matches C `one_letter_cmd()` exactly — including the complex 's' exclusions
/// for :scriptnames, :source, :simalt, :sign, :smagic, :snomagic, etc.
///
/// # Safety
///
/// `p` must be a valid null-terminated C string.
/// `idx` must be a valid pointer for writes.
#[export_name = "one_letter_cmd"]
pub unsafe extern "C" fn rs_one_letter_cmd(p: *const c_char, idx: *mut c_int) -> c_int {
    if p.is_null() || idx.is_null() {
        return 0;
    }

    let p0 = *p as u8;
    let p1 = *p.add(1) as u8;
    let p2 = *p.add(2) as u8;

    // 'k' command - mark
    // Match: k followed by anything except "ee" (which would be :keepXXX)
    if p0 == b'k' && !(p1 == b'e' && p2 == b'e') {
        *idx = crate::commands::CMD_K;
        return 1;
    }

    // 's' command - substitute
    if p0 == b's' {
        let p3 = *p.add(3) as u8;
        let p4 = *p.add(4) as u8;

        if (p1 == b'c'
            && (p2 == 0 || (p2 != b's' && p2 != b'r' && (p3 == 0 || (p3 != b'i' && p4 != b'p')))))
            || p1 == b'g'
            || (p1 == b'i' && p2 != b'm' && p2 != b'l' && p2 != b'g')
            || p1 == b'I'
            || (p1 == b'r' && p2 != b'e')
        {
            *idx = crate::commands::CMD_SUBSTITUTE;
            return 1;
        }
    }

    0
}

// =============================================================================
// cmd_exists - Check if command name exists
// =============================================================================

/// Check if an Ex command `name` exists.
///
/// Returns:
/// - 0: command doesn't exist
/// - 1: partial match (abbreviation)
/// - 2: exact match
/// - 3: ambiguous match
///
/// Matches C `cmd_exists()`.
///
/// # Safety
///
/// `name` must be a valid null-terminated C string.
#[export_name = "cmd_exists"]
pub unsafe extern "C" fn rs_cmd_exists(name: *const c_char) -> c_int {
    if name.is_null() {
        return 0;
    }

    // Check command modifiers first.
    let modifier_result = crate::modifiers::check_modifier(name);
    if modifier_result > 0 {
        return modifier_result;
    }

    // Check built-in commands and user defined commands.
    let mut cmdidx: c_int = 0;
    let mut full: c_int = 0;
    let p = nvim_docmd_cmd_exists_inner(name, &mut cmdidx, &mut full, std::ptr::null_mut());

    if p.is_null() {
        return 3;
    }

    if (*name as u8).is_ascii_digit() && cmdidx != crate::commands::CMD_MATCH {
        return 0;
    }

    if *skipwhite(p as *const c_char) as u8 != 0 {
        return 0; // trailing garbage
    }

    if cmdidx == crate::commands::CMD_SIZE {
        0
    } else if full != 0 {
        2
    } else {
        1
    }
}

// =============================================================================
// is_cmd_ni - Check if command is "not implemented"
// =============================================================================

/// Check if a command is "not implemented" (ex_ni or ex_script_ni).
///
/// Matches C `is_cmd_ni()`.
#[export_name = "is_cmd_ni"]
pub extern "C" fn rs_is_cmd_ni(cmdidx: c_int) -> c_int {
    unsafe { nvim_docmd_cmdnames_func_is_ni(cmdidx) }
}

// =============================================================================
// find_ex_command - Central command lookup
// =============================================================================

/// Find an Ex command by its name.
///
/// Start of the name can be found at `eap->cmd`.
/// Sets `eap->cmdidx` and returns a pointer to char after the command name.
/// `full` is set to true (1) if the whole command name matched.
///
/// Returns NULL for an ambiguous user command.
///
/// Matches C `find_ex_command()`.
///
/// # Safety
///
/// `eap` must be a valid ExArgHandle. `full` may be NULL.
#[export_name = "find_ex_command"]
pub unsafe extern "C" fn rs_find_ex_command(eap: ExArgHandle, full: *mut c_int) -> *mut c_char {
    if eap.is_null() {
        return std::ptr::null_mut();
    }

    let cmd = nvim_eap_get_cmd(eap);
    let mut p = cmd;

    // Try one-letter command first.
    let mut idx_val: c_int = 0;
    if rs_one_letter_cmd(cmd as *const c_char, &mut idx_val) != 0 {
        nvim_eap_set_cmdidx(eap, idx_val);
        p = p.add(1);
        if !full.is_null() {
            *full = 1;
        }
    } else {
        // Skip alphabetic chars.
        while (*p as u8).is_ascii_alphabetic() {
            p = p.add(1);
        }
        // For python 3.x: ":py3", ":python3", ":py3file", etc.
        if *cmd as u8 == b'p' && *cmd.add(1) as u8 == b'y' {
            while (*p as u8).is_ascii_alphanumeric() {
                p = p.add(1);
            }
        }

        // Check for non-alpha command.
        if p == cmd {
            let c = *p as u8;
            if c == b'@'
                || c == b'!'
                || c == b'='
                || c == b'>'
                || c == b'<'
                || c == b'&'
                || c == b'~'
                || c == b'#'
            {
                p = p.add(1);
            }
        }

        let len = p.offset_from(cmd) as c_int;

        // The "d" command can directly be followed by 'l' or 'p' flag.
        let mut effective_len = len;
        if *cmd as u8 == b'd' && len > 0 {
            let last_char = *p.sub(1) as u8;
            if last_char == b'l' || last_char == b'p' {
                // Check for ":dl", ":dell", etc. to ":deletel"
                let delete_str = b"delete";
                let mut i = 0i32;
                while (i as usize) < delete_str.len() && i < len {
                    if *cmd.add(i as usize) as u8 != delete_str[i as usize] {
                        break;
                    }
                    i += 1;
                }
                if i == len - 1 {
                    effective_len -= 1;
                    let flags = nvim_eap_get_flags(eap);
                    if last_char == b'l' {
                        nvim_eap_set_flags(eap, flags | crate::execute::EXFLAG_LIST);
                    } else {
                        nvim_eap_set_flags(eap, flags | crate::execute::EXFLAG_PRINT);
                    }
                }
            }
        }

        // Determine starting cmdidx.
        let c0 = *cmd as u8;
        if c0.is_ascii_lowercase() {
            let c1 = c0 as c_int;
            let c2 = if effective_len == 1 {
                0
            } else {
                *cmd.add(1) as u8 as c_int
            };

            if nvim_docmd_get_command_count() != crate::commands::CMD_SIZE {
                nvim_docmd_e943_abort();
            }

            let mut start_idx = nvim_docmd_get_cmdidxs1(c1);
            if (c2 as u8).is_ascii_lowercase() {
                start_idx += nvim_docmd_get_cmdidxs2(c1, c2);
            }
            nvim_eap_set_cmdidx(eap, start_idx);
        } else if c0.is_ascii_uppercase() {
            nvim_eap_set_cmdidx(eap, crate::commands::CMD_NEXT);
        } else {
            nvim_eap_set_cmdidx(eap, crate::commands::CMD_BANG);
        }

        // Make :def an unknown command (#23149).
        if effective_len == 3
            && *cmd as u8 == b'd'
            && *cmd.add(1) as u8 == b'e'
            && *cmd.add(2) as u8 == b'f'
        {
            nvim_eap_set_cmdidx(eap, crate::commands::CMD_SIZE);
        }

        // Iterate cmdnames[] for prefix match.
        let mut cidx = nvim_eap_get_cmdidx(eap);
        while cidx < crate::commands::CMD_SIZE {
            if nvim_docmd_cmdnames_prefix_match(cidx, cmd as *const c_char, effective_len) != 0 {
                if !full.is_null() && nvim_docmd_cmdnames_name_complete(cidx, effective_len) != 0 {
                    *full = 1;
                }
                break;
            }
            cidx += 1;
        }
        nvim_eap_set_cmdidx(eap, cidx);

        // Look for a user defined command as a last resort.
        if nvim_eap_get_cmdidx(eap) == crate::commands::CMD_SIZE && (*cmd as u8) >= b'A' && (*cmd as u8) <= b'Z' {
            // User defined commands may contain digits.
            while (*p as u8).is_ascii_alphanumeric() {
                p = p.add(1);
            }
            p = nvim_docmd_find_ucmd(eap, p, full);
        }
        if p == cmd {
            nvim_eap_set_cmdidx(eap, crate::commands::CMD_SIZE);
        }
    }

    p
}

// =============================================================================
// excmd_get_cmdidx - Get command index from name
// =============================================================================

/// Get the command index for a command name of given length.
///
/// Matches C `excmd_get_cmdidx()`.
///
/// # Safety
///
/// `cmd` must be a valid pointer to at least `len` bytes.
#[export_name = "excmd_get_cmdidx"]
pub unsafe extern "C" fn rs_excmd_get_cmdidx(cmd: *const c_char, len: usize) -> c_int {
    if cmd.is_null() {
        return crate::commands::CMD_SIZE;
    }

    // Make :def an unknown command (#23149).
    if len == 3 && *cmd as u8 == b'd' && *cmd.add(1) as u8 == b'e' && *cmd.add(2) as u8 == b'f' {
        return crate::commands::CMD_SIZE;
    }

    let mut idx_val: c_int = 0;
    if rs_one_letter_cmd(cmd, &mut idx_val) != 0 {
        return idx_val;
    }

    let len_i = len as c_int;
    let mut idx: c_int = 0;
    while idx < crate::commands::CMD_SIZE {
        if nvim_docmd_cmdnames_prefix_match(idx, cmd, len_i) != 0 {
            break;
        }
        idx += 1;
    }
    idx
}

// =============================================================================
// get_command_name - Get name string from cmdidx
// =============================================================================

/// Get command name for completion.
///
/// Returns the name of the command at `idx`, or a user command name
/// if `idx >= CMD_SIZE`.
///
/// Matches C `get_command_name()`.
///
/// # Safety
///
/// `xp` is unused (passed through for API compat). `idx` must be valid.
#[export_name = "get_command_name"]
pub unsafe extern "C" fn rs_get_command_name(_xp: *mut c_void, idx: c_int) -> *mut c_char {
    if idx >= crate::commands::CMD_SIZE {
        return nvim_docmd_expand_user_cmd_name(idx);
    }
    nvim_docmd_cmdnames_name(idx)
}

// =============================================================================
// f_fullcommand - VimL fullcommand() function
// =============================================================================

/// Implementation of the VimL `fullcommand()` function.
///
/// Matches C `f_fullcommand()`.
///
/// # Safety
///
/// `argvars` and `rettv` must be valid pointers to typval_T.
#[export_name = "f_fullcommand"]
pub unsafe extern "C" fn rs_f_fullcommand(argvars: *mut c_void, rettv: *mut c_void, _fptr: u64) {
    let name_ptr = nvim_docmd_tv_get_string(argvars as *const c_void);
    nvim_docmd_rettv_init_string(rettv);

    // Skip leading colons.
    let mut name = name_ptr;
    while *name as u8 == b':' {
        name = name.add(1);
    }

    // Skip range.
    name = crate::rs_skip_range(name as *const c_char, std::ptr::null_mut()) as *mut c_char;

    // Use cmd_exists_inner to create temp exarg_T and call find_ex_command.
    let mut cmdidx: c_int = 0;
    let mut full_val: c_int = 0;
    let mut useridx: c_int = 0;
    let p = nvim_docmd_cmd_exists_inner(
        name as *const c_char,
        &mut cmdidx,
        &mut full_val,
        &mut useridx,
    );

    if p.is_null() || cmdidx == crate::commands::CMD_SIZE {
        return;
    }

    // IS_USER_CMDIDX: cmdidx < 0
    if cmdidx < 0 {
        let user_name = nvim_docmd_get_user_command_name(useridx, cmdidx);
        if !user_name.is_null() {
            nvim_docmd_rettv_set_string(rettv, user_name as *const c_char);
        }
    } else {
        let cmd_name = nvim_docmd_cmdnames_name(cmdidx);
        if !cmd_name.is_null() {
            nvim_docmd_rettv_set_string(rettv, cmd_name as *const c_char);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_one_letter_cmd_char() {
        // 'k' is always a one-letter command
        assert!(is_one_letter_cmd_char(b'k', b'x'));
        assert!(is_one_letter_cmd_char(b'k', b' '));
        assert!(is_one_letter_cmd_char(b'k', 0));

        // 's' followed by non-alpha is substitute
        assert!(is_one_letter_cmd_char(b's', b'/'));
        assert!(is_one_letter_cmd_char(b's', b' '));
        assert!(is_one_letter_cmd_char(b's', 0));

        // 's' followed by alpha is not a one-letter command
        assert!(!is_one_letter_cmd_char(b's', b'e')); // :set
        assert!(!is_one_letter_cmd_char(b's', b'o')); // :sort

        // Other letters are not one-letter commands
        assert!(!is_one_letter_cmd_char(b'w', b' '));
        assert!(!is_one_letter_cmd_char(b'q', b' '));
    }

    #[test]
    fn test_is_cmd_name_start() {
        // Letters
        assert!(is_cmd_name_start(b'a'));
        assert!(is_cmd_name_start(b'z'));
        assert!(is_cmd_name_start(b'A'));
        assert!(is_cmd_name_start(b'Z'));

        // Special characters
        assert!(is_cmd_name_start(b'@'));
        assert!(is_cmd_name_start(b'!'));
        assert!(is_cmd_name_start(b'='));
        assert!(is_cmd_name_start(b'>'));
        assert!(is_cmd_name_start(b'<'));
        assert!(is_cmd_name_start(b'&'));
        assert!(is_cmd_name_start(b'~'));
        assert!(is_cmd_name_start(b'#'));

        // Not valid starts
        assert!(!is_cmd_name_start(b'1'));
        assert!(!is_cmd_name_start(b' '));
        assert!(!is_cmd_name_start(b':'));
    }

    #[test]
    fn test_is_cmd_name_char() {
        assert!(is_cmd_name_char(b'a'));
        assert!(is_cmd_name_char(b'Z'));
        assert!(!is_cmd_name_char(b'1'));
        assert!(!is_cmd_name_char(b' '));
    }

    #[test]
    fn test_is_python_cmd_prefix() {
        assert!(is_python_cmd_prefix(b"py"));
        assert!(is_python_cmd_prefix(b"python"));
        assert!(is_python_cmd_prefix(b"py3"));
        assert!(!is_python_cmd_prefix(b"p"));
        assert!(!is_python_cmd_prefix(b"pe"));
    }

    #[test]
    fn test_is_user_cmd_start() {
        assert!(is_user_cmd_start(b'A'));
        assert!(is_user_cmd_start(b'Z'));
        assert!(!is_user_cmd_start(b'a'));
        assert!(!is_user_cmd_start(b'1'));
    }

    #[test]
    fn test_is_user_cmd_char() {
        assert!(is_user_cmd_char(b'A'));
        assert!(is_user_cmd_char(b'z'));
        assert!(is_user_cmd_char(b'5'));
        assert!(!is_user_cmd_char(b' '));
        assert!(!is_user_cmd_char(b'_'));
    }

    #[test]
    fn test_cmd_char_ord_low() {
        assert_eq!(cmd_char_ord_low(b'a'), 0);
        assert_eq!(cmd_char_ord_low(b'b'), 1);
        assert_eq!(cmd_char_ord_low(b'z'), 25);
        assert_eq!(cmd_char_ord_low(b'A'), 0); // Returns 0 for non-lowercase
    }

    // Note: rs_checkforcmd, rs_one_letter_cmd, rs_cmd_exists, rs_is_cmd_ni tests
    // require C FFI and are verified through integration tests (just smoke-test).
}
