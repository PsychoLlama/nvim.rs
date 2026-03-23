//! Argument parsing types and utilities for Ex commands.
//!
//! This module provides types and functions for parsing command arguments,
//! including ++opt options, counts, registers, and filename expansion.

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

use crate::ExArgHandle;

// =============================================================================
// FFI declarations for exarg_T accessors and command index values
// =============================================================================

extern "C" {
    fn nvim_eap_get_arg(eap: ExArgHandle) -> *mut c_char;
    fn nvim_eap_set_arg(eap: ExArgHandle, arg: *mut c_char);
    fn nvim_eap_get_cmdidx(eap: ExArgHandle) -> c_int;
    fn nvim_eap_get_cmd(eap: ExArgHandle) -> *mut c_char;
    fn nvim_eap_get_argt(eap: ExArgHandle) -> u32;
    fn nvim_eap_set_line1(eap: ExArgHandle, line: i32);
    fn nvim_eap_get_line2(eap: ExArgHandle) -> i32;
    fn nvim_eap_set_line2(eap: ExArgHandle, line: i32);
    fn nvim_eap_get_addr_type(eap: ExArgHandle) -> c_int;
    fn nvim_eap_get_addr_count(eap: ExArgHandle) -> c_int;
    fn nvim_eap_set_addr_count(eap: ExArgHandle, count: c_int);

    // Phase 4: additional eap field accessors
    fn nvim_eap_set_regname(eap: ExArgHandle, r: c_int);
    fn nvim_eap_set_bad_char(eap: ExArgHandle, c: c_int);
    fn nvim_eap_set_force_bin(eap: ExArgHandle, v: c_int);
    fn nvim_eap_set_force_ff(eap: ExArgHandle, v: c_int);
    fn nvim_eap_set_force_enc(eap: ExArgHandle, v: c_int);
    fn nvim_eap_set_read_edit(eap: ExArgHandle, v: c_int);
    fn nvim_eap_set_mkdir_p(eap: ExArgHandle, v: c_int);
    fn nvim_eap_set_nextcmd(eap: ExArgHandle, p: *mut c_char);
    fn nvim_eap_get_skip(eap: ExArgHandle) -> c_int;

    fn nvim_docmd_grep_internal(cmdidx: c_int) -> c_int;
    fn nvim_docmd_get_curbuf_line_count() -> i32;

    // Phase 4: helper function wrappers
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn nvim_docmd_valid_yank_reg(regname: c_int, writing: c_int) -> c_int;
    fn nvim_docmd_set_expr_line(arg: *const c_char);
    fn nvim_docmd_check_ff_value(p: *const c_char) -> c_int;
    fn nvim_docmd_strmove(dst: *mut c_char, src: *const c_char);
    #[link_name = "utfc_ptr2len"]
    fn nvim_docmd_mb_ptr_adv_len(p: *const c_char) -> c_int;
    fn nvim_docmd_mb_byte2len(b: c_int) -> c_int;
    #[link_name = "rs_ascii_tolower"]
    fn nvim_docmd_tolower_asc(c: c_int) -> c_int;
    fn nvim_docmd_skip_expr(pp: *mut *mut c_char);
    fn nvim_docmd_cpo_has_bar() -> c_int;
    #[link_name = "del_trailing_spaces"]
    fn nvim_docmd_del_trailing_spaces(p: *mut c_char);
    fn nvim_docmd_get_dollar_command() -> *mut c_char;
    fn nvim_docmd_parse_count_digits(eap: ExArgHandle) -> c_int;
    fn nvim_docmd_get_e_zerocount() -> *const c_char;
    fn nvim_docmd_count_buf_check(eap: ExArgHandle) -> c_int;

    fn rs_skip_vimgrep_pat(p: *mut c_char, s: *mut *mut c_char, flags: *mut c_int) -> *mut c_char;
    #[link_name = "checkforcmd"]
    fn rs_checkforcmd(pp: *mut *mut c_char, cmd: *const c_char, len: c_int) -> bool;
    #[link_name = "check_nextcmd"]
    fn rs_check_nextcmd(p: *const c_char) -> *mut c_char;

    // replace_makeprg helpers
    fn nvim_docmd_get_grep_or_make_program(isgrep: c_int) -> *const c_char;
    fn msg_make(arg: *const c_char);
    fn strrep(src: *const c_char, what: *const c_char, rep: *const c_char) -> *mut c_char;
    fn xmalloc(size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);
    #[link_name = "strlen"]
    fn c_strlen(s: *const c_char) -> usize;
}

/// Address type: lines in current buffer (matches ADDR_LINES in C).
const ADDR_LINES: c_int = 0;

/// BAD_KEEP: leave bad char as-is.
const BAD_KEEP: c_int = -1;
/// BAD_DROP: drop bad char.
const BAD_DROP: c_int = -2;

/// Ctrl-V character code.
const CTRL_V: u8 = 22;

/// OK return value (matches C OK).
const OK: c_int = 1;
/// FAIL return value (matches C FAIL).
const FAIL: c_int = 0;

// =============================================================================
// Force binary mode constants
// =============================================================================

/// Don't force binary mode
pub const FORCE_BIN_NONE: c_int = 0;
/// Force binary mode (:edit ++bin)
pub const FORCE_BIN: c_int = 1;
/// Force no binary mode (:edit ++nobin)
pub const FORCE_NOBIN: c_int = 2;

// =============================================================================
// ++opt argument parsing helpers
// =============================================================================

/// Check if the argument starts with "++" (option argument).
#[inline]
pub fn starts_with_plus_plus(arg: &[u8]) -> bool {
    arg.len() >= 2 && arg[0] == b'+' && arg[1] == b'+'
}

/// FFI wrapper for ++opt check.
///
/// # Safety
///
/// `arg` must be a valid pointer to at least 2 bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_starts_with_plus_plus(arg: *const c_char) -> c_int {
    if arg.is_null() {
        return 0;
    }
    let c0 = *arg as u8;
    let c1 = *arg.add(1) as u8;
    c_int::from(c0 == b'+' && c1 == b'+')
}

/// Check if argument matches a ++opt prefix.
///
/// Returns true if `arg` starts with `prefix`.
#[inline]
pub fn matches_argopt(arg: &[u8], prefix: &[u8]) -> bool {
    arg.len() >= prefix.len() && &arg[..prefix.len()] == prefix
}

// =============================================================================
// Count argument helpers
// =============================================================================

/// Check if a character is a valid count digit.
#[inline]
pub const fn is_count_digit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

/// FFI wrapper for count digit check.
#[no_mangle]
pub extern "C" fn rs_is_count_digit(c: c_int) -> c_int {
    c_int::from(is_count_digit(c as u8))
}

/// Parse a count from a string.
///
/// Returns the parsed count and the number of digits consumed.
pub fn parse_count(s: &[u8]) -> (u64, usize) {
    let mut count: u64 = 0;
    let mut pos = 0;

    while pos < s.len() && is_count_digit(s[pos]) {
        count = count
            .saturating_mul(10)
            .saturating_add((s[pos] - b'0') as u64);
        pos += 1;
    }

    (count, pos)
}

/// FFI wrapper for count parsing.
///
/// Returns the parsed count value, or 0 if no digits found.
///
/// # Safety
///
/// `s` must be a valid null-terminated string.
/// `consumed` must be a valid pointer for writing the number of digits.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_count(s: *const c_char, consumed: *mut c_int) -> u64 {
    if s.is_null() {
        if !consumed.is_null() {
            *consumed = 0;
        }
        return 0;
    }

    let mut count: u64 = 0;
    let mut pos = 0;
    let mut ptr = s;

    loop {
        let c = *ptr as u8;
        if c == 0 || !is_count_digit(c) {
            break;
        }
        count = count.saturating_mul(10).saturating_add((c - b'0') as u64);
        pos += 1;
        ptr = ptr.add(1);
    }

    if !consumed.is_null() {
        *consumed = pos;
    }
    count
}

// =============================================================================
// Register argument helpers
// =============================================================================

/// Check if a character is a valid register name.
///
/// Valid registers are:
/// - 0-9 (numbered)
/// - a-z, A-Z (named)
/// - ", -, _, +, *, ~, /, :, ., %, # (special)
#[inline]
pub fn is_valid_register(c: u8) -> bool {
    c.is_ascii_alphanumeric()
        || c == b'"'
        || c == b'-'
        || c == b'_'
        || c == b'+'
        || c == b'*'
        || c == b'~'
        || c == b'/'
        || c == b':'
        || c == b'.'
        || c == b'%'
        || c == b'#'
        || c == b'='
}

/// FFI wrapper for register validation.
#[no_mangle]
pub extern "C" fn rs_is_valid_register(c: c_int) -> c_int {
    c_int::from(is_valid_register(c as u8))
}

/// Check if this is a read-only register.
///
/// Read-only registers: %, #, :, .
#[inline]
pub const fn is_readonly_register(c: u8) -> bool {
    c == b'%' || c == b'#' || c == b':' || c == b'.'
}

/// FFI wrapper for read-only register check.
#[no_mangle]
pub extern "C" fn rs_is_readonly_register(c: c_int) -> c_int {
    c_int::from(is_readonly_register(c as u8))
}

/// Check if this is a special register.
///
/// Special registers: ", -, _, +, *, ~, /, =
#[inline]
pub const fn is_special_register(c: u8) -> bool {
    c == b'"'
        || c == b'-'
        || c == b'_'
        || c == b'+'
        || c == b'*'
        || c == b'~'
        || c == b'/'
        || c == b'='
}

/// FFI wrapper for special register check.
#[no_mangle]
pub extern "C" fn rs_is_special_register(c: c_int) -> c_int {
    c_int::from(is_special_register(c as u8))
}

// =============================================================================
// Filename expansion helpers
// =============================================================================

/// Check if character needs expansion in filenames.
///
/// Characters that trigger expansion: %, #, <
#[inline]
pub const fn needs_filename_expansion(c: u8) -> bool {
    c == b'%' || c == b'#' || c == b'<'
}

/// FFI wrapper for filename expansion check.
#[no_mangle]
pub extern "C" fn rs_needs_filename_expansion(c: c_int) -> c_int {
    c_int::from(needs_filename_expansion(c as u8))
}

/// Check if the position is at a backslash-escaped character.
///
/// Returns true if position has a backslash before it.
#[inline]
pub fn is_escaped(s: &[u8], pos: usize) -> bool {
    if pos == 0 {
        return false;
    }

    // Count consecutive backslashes before pos
    let mut count = 0;
    let mut i = pos;
    while i > 0 && s[i - 1] == b'\\' {
        count += 1;
        i -= 1;
    }

    // Odd number of backslashes means the character is escaped
    count % 2 == 1
}

// =============================================================================
// parse_bang - Check for `!` after command
// =============================================================================

/// Check if `!` follows the command (and it's not a substitute variant).
///
/// Returns true if bang is found and consumed. Advances `*p` past the `!`.
///
/// # Safety
///
/// `eap` must be a valid ExArgHandle.
/// `p` must point to a valid `*mut c_char` pointer.
#[export_name = "parse_bang"]
pub unsafe extern "C" fn rs_parse_bang(eap: ExArgHandle, p: *mut *mut c_char) -> bool {
    if eap.is_null() || p.is_null() {
        return false;
    }

    let cmdidx = nvim_eap_get_cmdidx(eap);

    if *(*p) as u8 == b'!'
        && cmdidx != crate::commands::CMD_SUBSTITUTE
        && cmdidx != crate::commands::CMD_SMAGIC
        && cmdidx != crate::commands::CMD_SNOMAGIC
    {
        *p = (*p).add(1);
        return true;
    }
    false
}

// =============================================================================
// skip_grep_pat - Skip grep pattern in arguments
// =============================================================================

/// Skip the grep pattern in command arguments for vimgrep-like commands.
///
/// Returns a pointer past the pattern, or the original arg if not a grep command.
///
/// # Safety
///
/// `eap` must be a valid ExArgHandle.
#[export_name = "skip_grep_pat"]
pub unsafe extern "C" fn rs_skip_grep_pat(eap: ExArgHandle) -> *mut c_char {
    if eap.is_null() {
        return ptr::null_mut();
    }

    let arg = nvim_eap_get_arg(eap);
    if arg.is_null() || *arg as u8 == 0 {
        return arg;
    }

    let cmdidx = nvim_eap_get_cmdidx(eap);

    if cmdidx == crate::commands::CMD_VIMGREP
        || cmdidx == crate::commands::CMD_LVIMGREP
        || cmdidx == crate::commands::CMD_VIMGREPADD
        || cmdidx == crate::commands::CMD_LVIMGREPADD
        || nvim_docmd_grep_internal(cmdidx) != 0
    {
        let p = rs_skip_vimgrep_pat(arg, ptr::null_mut(), ptr::null_mut());
        if p.is_null() {
            return arg;
        }
        return p;
    }
    arg
}

// =============================================================================
// set_cmd_count - Set count from address into eap fields
// =============================================================================

/// Set the command count from an address value.
///
/// For non-line address types (e.g. `:buffer 2`), stores count in line2.
/// For line addresses, treats count as an offset from line2.
/// If `validate` is non-zero, clamps line2 to buffer line count.
///
/// # Safety
///
/// `eap` must be a valid ExArgHandle.
#[export_name = "set_cmd_count"]
pub unsafe extern "C" fn rs_set_cmd_count(eap: ExArgHandle, count: c_int, validate: c_int) {
    if eap.is_null() {
        return;
    }

    let addr_type = nvim_eap_get_addr_type(eap);

    if addr_type != ADDR_LINES {
        // e.g. :buffer 2, :sleep 3
        nvim_eap_set_line2(eap, count);
        if nvim_eap_get_addr_count(eap) == 0 {
            nvim_eap_set_addr_count(eap, 1);
        }
    } else {
        let line2 = nvim_eap_get_line2(eap);
        nvim_eap_set_line1(eap, line2);

        if line2 >= i32::MAX - (count - 1) {
            nvim_eap_set_line2(eap, i32::MAX);
        } else {
            nvim_eap_set_line2(eap, line2 + count - 1);
        }

        nvim_eap_set_addr_count(eap, nvim_eap_get_addr_count(eap) + 1);

        // Be vi compatible: no error message for out of range.
        if validate != 0 {
            let line_count = nvim_docmd_get_curbuf_line_count();
            let new_line2 = nvim_eap_get_line2(eap);
            if new_line2 > line_count {
                nvim_eap_set_line2(eap, line_count);
            }
        }
    }
}

// =============================================================================
// Phase 4: parse_register - Parse register name from eap->arg
// =============================================================================

/// Parse a register name from `eap->arg`.
///
/// Matches C `parse_register()`.
///
/// # Safety
///
/// `eap` must be a valid ExArgHandle.
#[export_name = "parse_register"]
pub unsafe extern "C" fn rs_parse_register(eap: ExArgHandle) {
    if eap.is_null() {
        return;
    }

    let argt = nvim_eap_get_argt(eap);
    let arg = nvim_eap_get_arg(eap);
    let cmdidx = nvim_eap_get_cmdidx(eap);

    // EX_REGSTR must be set, arg must not be NUL
    if (argt & crate::table::EX_REGSTR) == 0 || *arg as u8 == 0 {
        return;
    }

    // IS_USER_CMDIDX: cmdidx < 0
    let is_user_cmd = cmdidx < 0;

    // Do not allow register = for user commands
    if is_user_cmd && *arg as u8 == b'=' {
        return;
    }

    // Don't accept numbered register when count is allowed
    if (argt & crate::table::EX_COUNT) != 0 && (*arg as u8).is_ascii_digit() {
        return;
    }

    // Check writing: allowed if not user command and not :put/:iput
    let writing = if !is_user_cmd
        && cmdidx != crate::commands::CMD_PUT
        && cmdidx != crate::commands::CMD_IPUT
    {
        1
    } else {
        0
    };

    if nvim_docmd_valid_yank_reg(*arg as c_int, writing) != 0 {
        let reg_char = *arg as u8;
        // Advance arg past the register name
        let new_arg = arg.add(1);
        nvim_eap_set_arg(eap, new_arg);
        nvim_eap_set_regname(eap, reg_char as c_int);

        // For '=' register: accept rest of line as expression
        if reg_char == b'=' && *new_arg as u8 != 0 {
            if nvim_eap_get_skip(eap) == 0 {
                nvim_docmd_set_expr_line(new_arg as *const c_char);
            }
            nvim_docmd_arg_skip_to_end(eap);
        }
        nvim_eap_set_arg(eap, skipwhite(nvim_eap_get_arg(eap) as *const c_char));
    }
}

extern "C" {
    fn nvim_docmd_arg_skip_to_end(eap: ExArgHandle);
}

// =============================================================================
// Phase 4: parse_count_ex - Parse count from eap->arg
// =============================================================================

/// Parse a count from `eap->arg`.
///
/// Returns OK (1) or FAIL (0). Sets `*errormsg` on error.
///
/// Matches C `parse_count()`.
///
/// # Safety
///
/// `eap` must be a valid ExArgHandle.
#[export_name = "parse_count"]
pub unsafe extern "C" fn rs_parse_count_ex(
    eap: ExArgHandle,
    errormsg: *mut *const c_char,
    validate: c_int,
) -> c_int {
    if eap.is_null() {
        return OK;
    }

    let argt = nvim_eap_get_argt(eap);
    let arg = nvim_eap_get_arg(eap);

    // Check for a count: EX_COUNT must be set, first char must be digit
    if (argt & crate::table::EX_COUNT) == 0 || !(*arg as u8).is_ascii_digit() {
        return OK;
    }

    // When accepting EX_BUFNAME, don't use "123foo" as a count
    if (argt & crate::table::EX_BUFNAME) != 0 && nvim_docmd_count_buf_check(eap) == 0 {
        return OK;
    }

    // Parse the digits (this also handles eap->args adjustment)
    let n = nvim_docmd_parse_count_digits(eap);

    if n <= 0 && (argt & crate::table::EX_ZEROR) == 0 {
        if !errormsg.is_null() {
            *errormsg = nvim_docmd_get_e_zerocount();
        }
        return FAIL;
    }
    rs_set_cmd_count(eap, n, validate);

    OK
}

// =============================================================================
// Phase 4: get_bad_opt - Parse ++bad= option value
// =============================================================================

/// Parse the `++bad=` option value.
///
/// Sets `eap->bad_char` to BAD_KEEP, BAD_DROP, or a single-byte character.
/// Returns OK or FAIL.
///
/// Matches C `get_bad_opt()`.
///
/// # Safety
///
/// `p` must be a valid null-terminated string. `eap` must be valid.
#[export_name = "get_bad_opt"]
pub unsafe extern "C" fn rs_get_bad_opt(p: *const c_char, eap: ExArgHandle) -> c_int {
    if p.is_null() || eap.is_null() {
        return FAIL;
    }

    // Check "keep"
    if stricmp_matches(p, b"keep") {
        nvim_eap_set_bad_char(eap, BAD_KEEP);
        return OK;
    }

    // Check "drop"
    if stricmp_matches(p, b"drop") {
        nvim_eap_set_bad_char(eap, BAD_DROP);
        return OK;
    }

    // Single-byte character: MB_BYTE2LEN(*p) == 1 && p[1] == NUL
    let b = *p as u8;
    if nvim_docmd_mb_byte2len(b as c_int) == 1 && *p.add(1) as u8 == 0 {
        nvim_eap_set_bad_char(eap, b as c_int);
        return OK;
    }

    FAIL
}

/// Case-insensitive comparison of C string against known byte string.
unsafe fn stricmp_matches(p: *const c_char, target: &[u8]) -> bool {
    for (i, &t) in target.iter().enumerate() {
        let c = *p.add(i) as u8;
        if c == 0 {
            return false;
        }
        if !c.eq_ignore_ascii_case(&t) {
            return false;
        }
    }
    // Must be at NUL
    *p.add(target.len()) as u8 == 0
}

// =============================================================================
// Phase 4: getargopt - Parse ++opt arguments
// =============================================================================

/// Parse `++opt` arguments for commands like `:edit`, `:read`, `:write`.
///
/// Returns OK or FAIL.
///
/// Matches C `getargopt()`.
///
/// # Safety
///
/// `eap` must be a valid ExArgHandle.
#[export_name = "getargopt"]
pub unsafe extern "C" fn rs_getargopt(eap: ExArgHandle) -> c_int {
    if eap.is_null() {
        return FAIL;
    }

    let eap_arg = nvim_eap_get_arg(eap);
    let mut arg = eap_arg.add(2); // skip "++"

    // ":edit ++[no]bin[ary] file"
    if strncmp_prefix(arg, b"bin") || strncmp_prefix(arg, b"nobin") {
        if *arg as u8 == b'n' {
            arg = arg.add(2); // skip "no"
            nvim_eap_set_force_bin(eap, FORCE_NOBIN as c_int);
        } else {
            nvim_eap_set_force_bin(eap, FORCE_BIN as c_int);
        }
        if !rs_checkforcmd(&mut arg, c"binary".as_ptr(), 3) {
            return FAIL;
        }
        nvim_eap_set_arg(eap, skipwhite(arg as *const c_char));
        return OK;
    }

    // ":read ++edit file"
    if strncmp_prefix(arg, b"edit") {
        nvim_eap_set_read_edit(eap, 1);
        nvim_eap_set_arg(eap, skipwhite(arg.add(4) as *const c_char));
        return OK;
    }

    // ":write ++p foo/bar/file"
    if *arg as u8 == b'p' {
        nvim_eap_set_mkdir_p(eap, 1);
        nvim_eap_set_arg(eap, skipwhite(arg.add(1) as *const c_char));
        return OK;
    }

    // Determine which option: ff, fileformat, enc, encoding, bad
    #[derive(PartialEq)]
    enum OptKind {
        Ff,
        Enc,
        Bad,
    }
    let opt_kind;

    if strncmp_prefix(arg, b"ff") {
        arg = arg.add(2);
        opt_kind = OptKind::Ff;
    } else if strncmp_prefix(arg, b"fileformat") {
        arg = arg.add(10);
        opt_kind = OptKind::Ff;
    } else if strncmp_prefix(arg, b"encoding") {
        arg = arg.add(8);
        opt_kind = OptKind::Enc;
    } else if strncmp_prefix(arg, b"enc") {
        arg = arg.add(3);
        opt_kind = OptKind::Enc;
    } else if strncmp_prefix(arg, b"bad") {
        arg = arg.add(3);
        opt_kind = OptKind::Bad;
    } else {
        return FAIL;
    }

    if *arg as u8 != b'=' {
        return FAIL;
    }
    arg = arg.add(1); // skip '='

    let cmd = nvim_eap_get_cmd(eap);
    let val_offset = arg.offset_from(cmd) as c_int;
    arg = rs_skip_cmd_arg(arg, 0);
    nvim_eap_set_arg(eap, skipwhite(arg as *const c_char));
    *arg = 0; // NUL-terminate the value

    match opt_kind {
        OptKind::Ff => {
            if nvim_docmd_check_ff_value(cmd.add(val_offset as usize) as *const c_char) == FAIL {
                return FAIL;
            }
            nvim_eap_set_force_ff(eap, *cmd.add(val_offset as usize) as u8 as c_int);
        }
        OptKind::Enc => {
            // Make 'fileencoding' lower case
            let mut p = cmd.add(val_offset as usize);
            while *p as u8 != 0 {
                *p = nvim_docmd_tolower_asc(*p as c_int) as c_char;
                p = p.add(1);
            }
            nvim_eap_set_force_enc(eap, val_offset);
        }
        OptKind::Bad => {
            if rs_get_bad_opt(cmd.add(val_offset as usize) as *const c_char, eap) == FAIL {
                return FAIL;
            }
        }
    }

    OK
}

/// Check if C string starts with given byte prefix.
unsafe fn strncmp_prefix(p: *const c_char, prefix: &[u8]) -> bool {
    for (i, &b) in prefix.iter().enumerate() {
        if *p.add(i) as u8 != b {
            return false;
        }
    }
    true
}

// =============================================================================
// Phase 4: getargcmd - Get +command from argument
// =============================================================================

/// Get `+command` from an argument string.
///
/// If `*argp` starts with `+`, extracts the command and advances `*argp`.
/// Returns the command string or NULL.
///
/// Matches C `getargcmd()`.
///
/// # Safety
///
/// `argp` must point to a valid `*mut c_char` pointer.
#[export_name = "getargcmd"]
pub unsafe extern "C" fn rs_getargcmd(argp: *mut *mut c_char) -> *mut c_char {
    if argp.is_null() {
        return ptr::null_mut();
    }

    let arg = *argp;
    if *arg as u8 != b'+' {
        return ptr::null_mut();
    }

    let arg = arg.add(1); // skip '+'

    if (*arg as u8).is_ascii_whitespace() || *arg as u8 == 0 {
        let command = nvim_docmd_get_dollar_command();
        *argp = skipwhite(arg as *const c_char);
        return command;
    }

    let command = arg;
    let end = rs_skip_cmd_arg(arg, 1);
    if *end as u8 != 0 {
        *end = 0; // NUL-terminate command
        *argp = skipwhite(end.add(1) as *const c_char);
    } else {
        *argp = end;
    }
    command
}

// =============================================================================
// Phase 4: skip_cmd_arg - Find end of +command argument
// =============================================================================

/// Find end of `+command` argument. Skip over `\ ` and `\\`.
///
/// If `rembs` is non-zero, halve the number of backslashes.
///
/// Matches C `skip_cmd_arg()`.
///
/// # Safety
///
/// `p` must be a valid null-terminated string.
#[export_name = "skip_cmd_arg"]
pub unsafe extern "C" fn rs_skip_cmd_arg(p: *mut c_char, rembs: c_int) -> *mut c_char {
    if p.is_null() {
        return p;
    }

    let mut ptr = p;
    while *ptr as u8 != 0 && !(*ptr as u8).is_ascii_whitespace() {
        if *ptr as u8 == b'\\' && *ptr.add(1) as u8 != 0 {
            if rembs != 0 {
                nvim_docmd_strmove(ptr, ptr.add(1) as *const c_char);
            } else {
                ptr = ptr.add(1);
            }
        }
        let adv = nvim_docmd_mb_ptr_adv_len(ptr as *const c_char);
        ptr = ptr.add(adv as usize);
    }
    ptr
}

// =============================================================================
// Phase 4: separate_nextcmd - Separate next command at | or \n
// =============================================================================

/// Check for `|` to separate commands and `"` to start comments.
///
/// Matches C `separate_nextcmd()`.
///
/// # Safety
///
/// `eap` must be a valid ExArgHandle.
#[export_name = "separate_nextcmd"]
pub unsafe extern "C" fn rs_separate_nextcmd(eap: ExArgHandle) {
    if eap.is_null() {
        return;
    }

    let argt = nvim_eap_get_argt(eap);
    let cmdidx = nvim_eap_get_cmdidx(eap);
    let eap_arg = nvim_eap_get_arg(eap);

    let mut p = rs_skip_grep_pat(eap);

    loop {
        let c = *p as u8;
        if c == 0 {
            break;
        }

        if c == CTRL_V {
            if (argt & (crate::table::EX_CTRLV | crate::table::EX_XFILE)) != 0 {
                p = p.add(1); // skip CTRL-V and next char
            } else {
                // remove CTRL-V and skip next char
                nvim_docmd_strmove(p, p.add(1) as *const c_char);
            }
            if *p as u8 == 0 {
                break;
            }
        } else if c == b'`' && *p.add(1) as u8 == b'=' && (argt & crate::table::EX_XFILE) != 0 {
            // Skip over `=expr` when wildcards are expanded.
            p = p.add(2);
            nvim_docmd_skip_expr(&mut p);
            if *p as u8 == 0 {
                break;
            }
        } else {
            // Check for '"' (comment) or '|' (next command) or '\n'
            let is_comment = c == b'"'
                && (argt & crate::table::EX_NOTRLCOM) == 0
                && (cmdidx != crate::commands::CMD_AT || p != eap_arg)
                && (cmdidx != crate::commands::CMD_REDIR
                    || p != eap_arg.add(1)
                    || *p.sub(1) as u8 != b'@');

            let is_pipe = c == b'|'
                && cmdidx != crate::commands::CMD_APPEND
                && cmdidx != crate::commands::CMD_CHANGE
                && cmdidx != crate::commands::CMD_INSERT;

            let is_newline = c == b'\n';

            if is_comment || is_pipe || is_newline {
                // Remove '\' before '|' unless EX_CTRLV and CPO_BAR
                if (nvim_docmd_cpo_has_bar() == 0 || (argt & crate::table::EX_CTRLV) == 0)
                    && p > eap_arg
                    && *p.sub(1) as u8 == b'\\'
                {
                    nvim_docmd_strmove(p.sub(1), p as *const c_char);
                    p = p.sub(1);
                } else {
                    nvim_eap_set_nextcmd(eap, rs_check_nextcmd(p as *const c_char));
                    *p = 0;
                    break;
                }
            }
        }

        // Advance by multibyte char length
        let adv = nvim_docmd_mb_ptr_adv_len(p as *const c_char);
        p = p.add(adv as usize);
    }

    // Remove trailing spaces if not EX_NOTRLCOM
    if (argt & crate::table::EX_NOTRLCOM) == 0 {
        nvim_docmd_del_trailing_spaces(nvim_eap_get_arg(eap));
    }
}

/// `nvim_docmd_replace_makeprg_impl` - replace make/grep program in command line.
///
/// When `eap->cmdidx` is CMD_make, CMD_lmake, CMD_grep, etc. and NOT vimgrep,
/// replace the argument with the makeprg/grepprg program string, substituting
/// `$*` for the provided args. Returns the new argument pointer.
///
/// # Safety
/// `eap`, `arg`, `cmdlinep` must be valid. `arg` must be a valid C string.
#[export_name = "nvim_docmd_replace_makeprg_impl"]
pub unsafe extern "C" fn rs_replace_makeprg(
    eap: ExArgHandle,
    arg: *mut c_char,
    cmdlinep: *mut *mut c_char,
) -> *mut c_char {
    use crate::commands::{CMD_GREP, CMD_GREPADD, CMD_LGREP, CMD_LGREPADD, CMD_LMAKE, CMD_MAKE};

    let cmdidx = nvim_eap_get_cmdidx(eap);
    let isgrep = (cmdidx == CMD_GREP
        || cmdidx == CMD_LGREP
        || cmdidx == CMD_GREPADD
        || cmdidx == CMD_LGREPADD) as c_int;

    // Only act for make/grep commands when not using internal vimgrep.
    if (cmdidx == CMD_MAKE
        || cmdidx == CMD_LMAKE
        || cmdidx == CMD_GREP
        || cmdidx == CMD_LGREP
        || cmdidx == CMD_GREPADD
        || cmdidx == CMD_LGREPADD)
        && nvim_docmd_grep_internal(cmdidx) == 0
    {
        let program = nvim_docmd_get_grep_or_make_program(isgrep);
        let arg = skipwhite(arg as *const c_char);

        // Replace $* by given arguments, or build "<program> <arg>".
        let new_cmdline = strrep(program, c"$*".as_ptr(), arg);
        let new_cmdline: *mut c_char = if new_cmdline.is_null() {
            // No $* in program: build "<program> <arg>"
            let prog_len = c_strlen(program);
            let arg_len = c_strlen(arg);
            let buf = xmalloc(prog_len + arg_len + 2) as *mut c_char;
            std::ptr::copy_nonoverlapping(program, buf, prog_len);
            *buf.add(prog_len) = b' ' as c_char;
            std::ptr::copy_nonoverlapping(arg, buf.add(prog_len + 1), arg_len);
            *buf.add(prog_len + 1 + arg_len) = 0;
            buf
        } else {
            new_cmdline
        };

        msg_make(arg);

        // Replace the old cmdline string with the new one.
        xfree(*cmdlinep as *mut c_void);
        *cmdlinep = new_cmdline;
        return new_cmdline;
    }

    arg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starts_with_plus_plus() {
        assert!(starts_with_plus_plus(b"++bin"));
        assert!(starts_with_plus_plus(b"++"));
        assert!(!starts_with_plus_plus(b"+bin"));
        assert!(!starts_with_plus_plus(b"bin"));
        assert!(!starts_with_plus_plus(b"+"));
        assert!(!starts_with_plus_plus(b""));
    }

    #[test]
    fn test_matches_argopt() {
        assert!(matches_argopt(b"binary", b"bin"));
        assert!(matches_argopt(b"bin", b"bin"));
        assert!(!matches_argopt(b"bi", b"bin"));
        assert!(matches_argopt(b"nobinary", b"nobin"));
    }

    #[test]
    fn test_parse_count() {
        assert_eq!(parse_count(b"123abc"), (123, 3));
        assert_eq!(parse_count(b"0"), (0, 1));
        assert_eq!(parse_count(b"42"), (42, 2));
        assert_eq!(parse_count(b"abc"), (0, 0));
        assert_eq!(parse_count(b""), (0, 0));
    }

    #[test]
    fn test_is_valid_register() {
        // Named registers
        assert!(is_valid_register(b'a'));
        assert!(is_valid_register(b'z'));
        assert!(is_valid_register(b'A'));
        assert!(is_valid_register(b'Z'));

        // Numbered registers
        assert!(is_valid_register(b'0'));
        assert!(is_valid_register(b'9'));

        // Special registers
        assert!(is_valid_register(b'"'));
        assert!(is_valid_register(b'-'));
        assert!(is_valid_register(b'_'));
        assert!(is_valid_register(b'+'));
        assert!(is_valid_register(b'*'));
        assert!(is_valid_register(b'/'));

        // Invalid
        assert!(!is_valid_register(b' '));
        assert!(!is_valid_register(b'\n'));
    }

    #[test]
    fn test_is_readonly_register() {
        assert!(is_readonly_register(b'%'));
        assert!(is_readonly_register(b'#'));
        assert!(is_readonly_register(b':'));
        assert!(is_readonly_register(b'.'));
        assert!(!is_readonly_register(b'a'));
        assert!(!is_readonly_register(b'"'));
    }

    #[test]
    fn test_needs_filename_expansion() {
        assert!(needs_filename_expansion(b'%'));
        assert!(needs_filename_expansion(b'#'));
        assert!(needs_filename_expansion(b'<'));
        assert!(!needs_filename_expansion(b'a'));
        assert!(!needs_filename_expansion(b'/'));
    }

    #[test]
    fn test_is_escaped() {
        assert!(!is_escaped(b"abc", 0));
        assert!(!is_escaped(b"abc", 1));
        assert!(is_escaped(b"a\\bc", 2));
        assert!(!is_escaped(b"a\\\\bc", 3)); // Even backslashes
        assert!(is_escaped(b"a\\\\\\bc", 4)); // Odd backslashes
    }

    #[test]
    fn test_ffi_parse_count() {
        use std::ffi::CString;

        let s = CString::new("123abc").unwrap();
        let mut consumed: c_int = 0;
        let count = unsafe { rs_parse_count(s.as_ptr(), &mut consumed) };
        assert_eq!(count, 123);
        assert_eq!(consumed, 3);

        let s = CString::new("abc").unwrap();
        let count = unsafe { rs_parse_count(s.as_ptr(), &mut consumed) };
        assert_eq!(count, 0);
        assert_eq!(consumed, 0);
    }
}
