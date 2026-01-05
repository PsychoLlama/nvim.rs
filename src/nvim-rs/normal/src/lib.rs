//! Normal mode key processing and command handling for Neovim
//!
//! This crate provides Rust implementations of normal mode functions
//! from `src/nvim/normal.c`. It handles normal and visual mode command
//! processing.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// Key Constants (from keycodes.h)
// =============================================================================

/// Convert termcap codes to internal key representation
/// TERMCAP2KEY(a, b) = -((a) + ((int)(b) << 8))
const fn termcap2key(a: c_int, b: c_int) -> c_int {
    -((a) + (b << 8))
}

// KS_EXTRA for special keys
const KS_EXTRA: c_int = 253;

// KE_* values for special keys (from keycodes.h)
const KE_S_UP: c_int = 4;
const KE_S_DOWN: c_int = 5;
const KE_C_LEFT: c_int = 85;
const KE_C_RIGHT: c_int = 86;

// Standard arrow keys
const K_UP: c_int = termcap2key(b'k' as c_int, b'u' as c_int);
const K_DOWN: c_int = termcap2key(b'k' as c_int, b'd' as c_int);
const K_LEFT: c_int = termcap2key(b'k' as c_int, b'l' as c_int);
const K_RIGHT: c_int = termcap2key(b'k' as c_int, b'r' as c_int);
const K_HOME: c_int = termcap2key(b'k' as c_int, b'h' as c_int);
const K_END: c_int = termcap2key(b'@' as c_int, b'7' as c_int);

// Shifted arrow keys
const K_S_UP: c_int = termcap2key(KS_EXTRA, KE_S_UP);
const K_S_DOWN: c_int = termcap2key(KS_EXTRA, KE_S_DOWN);
const K_S_LEFT: c_int = termcap2key(b'#' as c_int, b'4' as c_int);
const K_S_RIGHT: c_int = termcap2key(b'%' as c_int, b'i' as c_int);
const K_S_HOME: c_int = termcap2key(b'#' as c_int, b'2' as c_int);
const K_S_END: c_int = termcap2key(b'*' as c_int, b'7' as c_int);

// Ctrl+arrow keys
const K_C_LEFT: c_int = termcap2key(KS_EXTRA, KE_C_LEFT);
const K_C_RIGHT: c_int = termcap2key(KS_EXTRA, KE_C_RIGHT);

// Direction constants (matching normal.c)
const BACKWARD: c_int = -1;
const FORWARD: c_int = 1;

// =============================================================================
// C accessor functions for normal mode state
// =============================================================================

extern "C" {
    /// Get the nv_max_linear value.
    fn nvim_get_nv_max_linear() -> c_int;

    /// Get the command character at index in nv_cmds.
    fn nvim_get_nv_cmd_char(idx: c_int) -> c_int;

    /// Get the NV_CMDS_SIZE constant.
    fn nvim_get_nv_cmds_size() -> c_int;

    /// Get the nv_cmd_idx value at position.
    fn nvim_get_nv_cmd_idx(idx: c_int) -> i16;

    /// Call the C simplify_key function.
    fn simplify_key(key: c_int, modp: *mut c_int) -> c_int;

    // Window accessors
    fn nvim_win_get_topline(wp: WinHandle) -> c_int;
    fn nvim_win_get_topfill(wp: WinHandle) -> c_int;

    // plines function
    fn plines_m_win_fill(wp: WinHandle, first: c_int, last: c_int) -> c_int;

    // oparg_T pointer accessors (takes explicit oap parameter)
    fn nvim_oap_get_op_type_ptr(oap: OapHandle) -> c_int;
    fn nvim_oap_set_op_type(oap: OapHandle, val: c_int);
    fn nvim_oap_set_regname(oap: OapHandle, val: c_int);
    fn nvim_oap_set_motion_force(oap: OapHandle, val: c_int);
    fn nvim_oap_set_use_reg_one(oap: OapHandle, val: bool);

    // Global motion_force accessor
    fn nvim_set_motion_force(val: c_int);

    // Lock check functions (remain in C)
    fn text_locked() -> bool;
    fn text_locked_msg();
    fn curbuf_locked() -> bool;

    // VIsual_active global (from plines.c, returns 0 or 1)
    fn nvim_get_VIsual_active() -> c_int;

    // Beep function
    fn beep_flush();

    // Redo buffer functions (from getchar.c)
    fn ResetRedobuff();
    fn AppendCharToRedobuff(c: c_int);
    fn AppendNumberToRedobuff(n: c_int);
}

// Operator type constants (must match ops.h)
const OP_NOP: c_int = 0;

// NUL constant for motion_force
const NUL_CHAR: c_int = 0;

/// Opaque handle to a window (win_T*).
pub type WinHandle = *mut std::ffi::c_void;

/// Opaque handle to operator arguments (oparg_T*).
pub type OapHandle = *mut std::ffi::c_void;

// =============================================================================
// Invert Horizontal Commands (for RTL mode)
// =============================================================================

/// Invert horizontal commands for right-to-left mode.
///
/// Swaps left/right movement keys and < > operators.
/// Returns the inverted command character.
#[inline]
fn invert_horizontal_impl(cmdchar: c_int) -> c_int {
    match cmdchar {
        x if x == c_int::from(b'l') => c_int::from(b'h'),
        x if x == K_RIGHT => K_LEFT,
        x if x == K_S_RIGHT => K_S_LEFT,
        x if x == K_C_RIGHT => K_C_LEFT,
        x if x == c_int::from(b'h') => c_int::from(b'l'),
        x if x == K_LEFT => K_RIGHT,
        x if x == K_S_LEFT => K_S_RIGHT,
        x if x == K_C_LEFT => K_C_RIGHT,
        x if x == c_int::from(b'>') => c_int::from(b'<'),
        x if x == c_int::from(b'<') => c_int::from(b'>'),
        _ => cmdchar,
    }
}

/// FFI wrapper for invert_horizontal.
#[no_mangle]
pub extern "C" fn rs_invert_horizontal(cmdchar: c_int) -> c_int {
    invert_horizontal_impl(cmdchar)
}

// =============================================================================
// Find Command (command lookup by character)
// =============================================================================

/// Search for a command in the commands table.
///
/// Returns -1 for invalid command.
#[inline]
fn find_command_impl(cmdchar: c_int) -> c_int {
    // A multi-byte character is never a command.
    if cmdchar >= 0x100 {
        return -1;
    }

    // We use the absolute value of the character. Special keys have a
    // negative value, but are sorted on their absolute value.
    let abs_char = cmdchar.abs();

    // SAFETY: These are safe accessors to C globals
    unsafe {
        let nv_max_linear = nvim_get_nv_max_linear();
        let nv_cmds_size = nvim_get_nv_cmds_size();

        // If the character is in the first part: The character is the index into
        // nv_cmd_idx[].
        if abs_char <= nv_max_linear {
            return c_int::from(nvim_get_nv_cmd_idx(abs_char));
        }

        // Perform a binary search.
        let mut bot = nv_max_linear + 1;
        let mut top = nv_cmds_size - 1;
        let mut idx = -1;

        while bot <= top {
            let i = c_int::midpoint(bot, top);
            let c = nvim_get_nv_cmd_char(c_int::from(nvim_get_nv_cmd_idx(i))).abs();
            if abs_char == c {
                idx = c_int::from(nvim_get_nv_cmd_idx(i));
                break;
            }
            if abs_char > c {
                bot = i + 1;
            } else {
                top = i - 1;
            }
        }
        idx
    }
}

/// FFI wrapper for find_command.
#[no_mangle]
pub extern "C" fn rs_find_command(cmdchar: c_int) -> c_int {
    find_command_impl(cmdchar)
}

// =============================================================================
// Unshift Special Keys
// =============================================================================

/// Remove the shift modifier from a special key.
///
/// Converts shifted special keys to their unshifted versions and
/// applies simplify_key.
///
/// # Safety
/// `modp` must be a valid pointer to the modifier mask.
#[no_mangle]
pub unsafe extern "C" fn rs_unshift_special(cmdchar: c_int, modp: *mut c_int) -> c_int {
    let unshifted = match cmdchar {
        K_S_RIGHT => K_RIGHT,
        K_S_LEFT => K_LEFT,
        K_S_UP => K_UP,
        K_S_DOWN => K_DOWN,
        K_S_HOME => K_HOME,
        K_S_END => K_END,
        _ => cmdchar,
    };

    // Call C's simplify_key to handle additional simplification
    simplify_key(unshifted, modp)
}

// =============================================================================
// Is Ident (check if position is not in comment/string)
// =============================================================================

/// NUL character constant.
const NUL: u8 = 0;

/// Check if line[offset] is not inside a C-style comment or string.
///
/// Scans from the beginning of the line to the given offset to determine
/// if the character at that position is within a comment or string literal.
///
/// # Safety
/// `line` must be a valid pointer to a NUL-terminated C string.
#[no_mangle]
#[allow(clippy::cast_sign_loss)] // offset is checked to be non-negative
pub unsafe extern "C" fn rs_is_ident(line: *const std::ffi::c_char, offset: c_int) -> bool {
    if line.is_null() || offset < 0 {
        return false;
    }

    let mut incomment = false;
    let mut instring: u8 = 0;
    let mut prev: u8 = 0;

    let line_bytes = line.cast::<u8>();
    let offset_usize = offset as usize;

    for i in 0..offset_usize {
        let ch = *line_bytes.add(i);
        if ch == NUL {
            break;
        }

        if instring != 0 {
            if prev != b'\\' && ch == instring {
                instring = 0;
            }
        } else if (ch == b'"' || ch == b'\'') && !incomment {
            instring = ch;
        } else if incomment {
            if prev == b'*' && ch == b'/' {
                incomment = false;
            }
        } else if prev == b'/' && ch == b'*' {
            incomment = true;
        } else if prev == b'/' && ch == b'/' {
            return false;
        }

        prev = ch;
    }

    !incomment && instring == 0
}

// =============================================================================
// Find Is Eval Item (eval item detection)
// =============================================================================

/// Check if the current character is part of an eval item.
///
/// Detects brackets [], dot notation (s.var), and arrow notation (s->var).
/// Used by find_ident_at_pos for FIND_EVAL mode.
///
/// # Arguments
/// * `ptr` - Pointer to current character
/// * `colp` - Pointer to column offset (updated for -> notation)
/// * `bnp` - Pointer to bracket nesting counter
/// * `dir` - Direction: BACKWARD (-1) or FORWARD (1)
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_find_is_eval_item(
    ptr: *const std::ffi::c_char,
    colp: *mut c_int,
    bnp: *mut c_int,
    dir: c_int,
) -> bool {
    if ptr.is_null() || colp.is_null() || bnp.is_null() {
        return false;
    }

    let ch = *ptr.cast::<u8>();

    // Accept everything inside [].
    if (ch == b']' && dir == BACKWARD) || (ch == b'[' && dir == FORWARD) {
        *bnp += 1;
    }
    if *bnp > 0 {
        if (ch == b'[' && dir == BACKWARD) || (ch == b']' && dir == FORWARD) {
            *bnp -= 1;
        }
        return true;
    }

    // skip over "s.var"
    if ch == b'.' {
        return true;
    }

    // two-character item: s->var
    let ptr_bytes = ptr.cast::<u8>();
    let check_idx = isize::from(dir != BACKWARD);
    let other_idx = check_idx - 1;

    // Check if we can safely access ptr[other_idx]
    // For BACKWARD, this is ptr[-1] which requires caller to ensure valid
    // For FORWARD, this is ptr[0] which is always valid if ptr is valid
    if *ptr_bytes.offset(check_idx) == b'>' && *ptr_bytes.offset(other_idx) == b'-' {
        *colp += dir;
        return true;
    }

    false
}

// =============================================================================
// Window Functions
// =============================================================================

/// Get the virtual top line of a window.
///
/// Returns the number of physical lines from line 1 to the top of the window,
/// accounting for folds and virtual lines.
///
/// # Safety
/// `wp` must be a valid window pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_get_vtopline(wp: WinHandle) -> c_int {
    let topline = nvim_win_get_topline(wp);
    let topfill = nvim_win_get_topfill(wp);
    plines_m_win_fill(wp, 1, topline) - topfill
}

// =============================================================================
// Operator State Functions
// =============================================================================

/// Clear operator state.
///
/// Resets op_type, regname, motion_force, use_reg_one in the oparg_T,
/// and clears the global motion_force.
///
/// # Safety
/// `oap` must be a valid oparg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_clearop(oap: OapHandle) {
    nvim_oap_set_op_type(oap, OP_NOP);
    nvim_oap_set_regname(oap, 0);
    nvim_oap_set_motion_force(oap, NUL_CHAR);
    nvim_oap_set_use_reg_one(oap, false);
    nvim_set_motion_force(NUL_CHAR);
}

/// Clear operator state and beep.
///
/// # Safety
/// `oap` must be a valid oparg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_clearopbeep(oap: OapHandle) {
    rs_clearop(oap);
    beep_flush();
}

/// Check for operator pending.
///
/// Returns true (and beeps) if an operator is pending.
///
/// # Safety
/// `oap` must be a valid oparg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_checkclearop(oap: OapHandle) -> bool {
    if nvim_oap_get_op_type_ptr(oap) == OP_NOP {
        return false;
    }
    rs_clearopbeep(oap);
    true
}

/// Check for operator or Visual active.
///
/// Returns true (and beeps) if an operator is pending or Visual is active.
///
/// # Safety
/// `oap` must be a valid oparg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_checkclearopq(oap: OapHandle) -> bool {
    if nvim_oap_get_op_type_ptr(oap) == OP_NOP && nvim_get_VIsual_active() == 0 {
        return false;
    }
    rs_clearopbeep(oap);
    true
}

/// Check if text is locked.
///
/// If text is locked, beeps (if oap != NULL) and shows an error message.
/// Returns true if text is locked.
///
/// # Safety
/// `oap` may be NULL, otherwise must be a valid oparg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_check_text_locked(oap: OapHandle) -> bool {
    if !text_locked() {
        return false;
    }

    if !oap.is_null() {
        rs_clearopbeep(oap);
    }
    text_locked_msg();
    true
}

/// Check if text or curbuf is locked.
///
/// If text is locked or curbuf is locked, beeps (if oap != NULL) and
/// shows an error message. Returns true if locked.
///
/// # Safety
/// `oap` may be NULL, otherwise must be a valid oparg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_check_text_or_curbuf_locked(oap: OapHandle) -> bool {
    if rs_check_text_locked(oap) {
        return true;
    }

    if !curbuf_locked() {
        return false;
    }

    if !oap.is_null() {
        rs_clearop(oap);
    }
    true
}

// =============================================================================
// Redo Preparation Functions
// =============================================================================

/// Prepare for redo of any command.
///
/// Builds the redo buffer with the given register, count, and command characters.
///
/// # Safety
/// Calls into C redo buffer functions which must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_prep_redo(
    regname: c_int,
    num: c_int,
    cmd1: c_int,
    cmd2: c_int,
    cmd3: c_int,
    cmd4: c_int,
    cmd5: c_int,
) {
    rs_prep_redo_num2(regname, num, cmd1, cmd2, 0, cmd3, cmd4, cmd5);
}

/// Prepare for redo of any command with extra count after cmd2.
///
/// Builds the redo buffer with the given register, counts, and command characters.
///
/// # Safety
/// Calls into C redo buffer functions which must be valid.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn rs_prep_redo_num2(
    regname: c_int,
    num1: c_int,
    cmd1: c_int,
    cmd2: c_int,
    num2: c_int,
    cmd3: c_int,
    cmd4: c_int,
    cmd5: c_int,
) {
    ResetRedobuff();

    // Yank from specified buffer
    if regname != 0 {
        AppendCharToRedobuff(c_int::from(b'"'));
        AppendCharToRedobuff(regname);
    }

    if num1 != 0 {
        AppendNumberToRedobuff(num1);
    }

    if cmd1 != NUL_CHAR {
        AppendCharToRedobuff(cmd1);
    }

    if cmd2 != NUL_CHAR {
        AppendCharToRedobuff(cmd2);
    }

    if num2 != 0 {
        AppendNumberToRedobuff(num2);
    }

    if cmd3 != NUL_CHAR {
        AppendCharToRedobuff(cmd3);
    }

    if cmd4 != NUL_CHAR {
        AppendCharToRedobuff(cmd4);
    }

    if cmd5 != NUL_CHAR {
        AppendCharToRedobuff(cmd5);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::cast_lossless)]
mod tests {
    use super::*;

    #[test]
    fn test_invert_horizontal() {
        // Basic letter swaps
        assert_eq!(invert_horizontal_impl(c_int::from(b'l')), c_int::from(b'h'));
        assert_eq!(invert_horizontal_impl(c_int::from(b'h')), c_int::from(b'l'));

        // Operator swaps
        assert_eq!(invert_horizontal_impl(c_int::from(b'>')), c_int::from(b'<'));
        assert_eq!(invert_horizontal_impl(c_int::from(b'<')), c_int::from(b'>'));

        // Key swaps
        assert_eq!(invert_horizontal_impl(K_RIGHT), K_LEFT);
        assert_eq!(invert_horizontal_impl(K_LEFT), K_RIGHT);
        assert_eq!(invert_horizontal_impl(K_S_RIGHT), K_S_LEFT);
        assert_eq!(invert_horizontal_impl(K_S_LEFT), K_S_RIGHT);
        assert_eq!(invert_horizontal_impl(K_C_RIGHT), K_C_LEFT);
        assert_eq!(invert_horizontal_impl(K_C_LEFT), K_C_RIGHT);

        // Non-horizontal commands pass through
        assert_eq!(invert_horizontal_impl(c_int::from(b'j')), c_int::from(b'j'));
        assert_eq!(invert_horizontal_impl(c_int::from(b'k')), c_int::from(b'k'));
        assert_eq!(invert_horizontal_impl(K_UP), K_UP);
        assert_eq!(invert_horizontal_impl(K_DOWN), K_DOWN);
    }

    #[test]
    fn test_is_ident_no_comment() {
        // Simple code without comments
        let line = b"int x = 5;\0";
        unsafe {
            assert!(rs_is_ident(line.as_ptr().cast(), 4)); // 'x' is outside comment
            assert!(rs_is_ident(line.as_ptr().cast(), 0)); // 'i' is outside comment
        }
    }

    #[test]
    fn test_is_ident_line_comment() {
        // Line with // comment
        let line = b"int x = 5; // comment\0";
        unsafe {
            assert!(rs_is_ident(line.as_ptr().cast(), 4)); // 'x' before comment
            assert!(!rs_is_ident(line.as_ptr().cast(), 15)); // inside // comment
        }
    }

    #[test]
    fn test_is_ident_block_comment() {
        // Code with block comment
        let line = b"int /* comment */ x;\0";
        unsafe {
            assert!(rs_is_ident(line.as_ptr().cast(), 2)); // 't' before comment
            assert!(!rs_is_ident(line.as_ptr().cast(), 8)); // inside /* comment */
            assert!(rs_is_ident(line.as_ptr().cast(), 18)); // 'x' after comment
        }
    }

    #[test]
    fn test_is_ident_string() {
        // Code with string literal
        let line = b"char *s = \"hello\";\0";
        unsafe {
            assert!(rs_is_ident(line.as_ptr().cast(), 6)); // 's' outside string
            assert!(!rs_is_ident(line.as_ptr().cast(), 12)); // 'e' inside string
        }
    }

    #[test]
    fn test_is_ident_char_literal() {
        // Code with character literal
        let line = b"char c = 'x';\0";
        unsafe {
            assert!(rs_is_ident(line.as_ptr().cast(), 5)); // 'c' outside literal
            assert!(!rs_is_ident(line.as_ptr().cast(), 10)); // 'x' inside literal
        }
    }

    #[test]
    fn test_is_ident_escaped_quote() {
        // String with escaped quote
        let line = b"char *s = \"he\\\"llo\";\0";
        unsafe {
            assert!(!rs_is_ident(line.as_ptr().cast(), 15)); // still inside string
        }
    }

    #[test]
    fn test_find_is_eval_item_dot() {
        // Dot notation
        let line = b"s.var\0";
        let mut col = 1;
        let mut bn = 0;
        unsafe {
            assert!(rs_find_is_eval_item(
                line.as_ptr().add(1).cast(),
                &raw mut col,
                &raw mut bn,
                FORWARD
            ));
        }
    }

    #[test]
    fn test_find_is_eval_item_bracket_forward() {
        // Opening bracket going forward
        let line = b"a[0]\0";
        let mut col = 1;
        let mut bn = 0;
        unsafe {
            // '[' when going forward increments bracket nesting
            assert!(rs_find_is_eval_item(
                line.as_ptr().add(1).cast(),
                &raw mut col,
                &raw mut bn,
                FORWARD
            ));
            assert_eq!(bn, 1);
        }
    }

    #[test]
    fn test_find_is_eval_item_bracket_backward() {
        // Closing bracket going backward
        let line = b"a[0]\0";
        let mut col = 3;
        let mut bn = 0;
        unsafe {
            // ']' when going backward increments bracket nesting
            assert!(rs_find_is_eval_item(
                line.as_ptr().add(3).cast(),
                &raw mut col,
                &raw mut bn,
                BACKWARD
            ));
            assert_eq!(bn, 1);
        }
    }

    #[test]
    fn test_find_is_eval_item_arrow() {
        // Arrow notation s->var (testing going forward)
        let line = b"s->var\0";
        let mut col = 1;
        let mut bn = 0;
        unsafe {
            // At '-', check for '->'
            assert!(rs_find_is_eval_item(
                line.as_ptr().add(1).cast(),
                &raw mut col,
                &raw mut bn,
                FORWARD
            ));
            // col should be incremented by dir
            assert_eq!(col, 2);
        }
    }

    #[test]
    fn test_find_is_eval_item_not_eval() {
        // Regular identifier character
        let line = b"abc\0";
        let mut col = 1;
        let mut bn = 0;
        unsafe {
            assert!(!rs_find_is_eval_item(
                line.as_ptr().add(1).cast(),
                &raw mut col,
                &raw mut bn,
                FORWARD
            ));
        }
    }

    #[test]
    fn test_key_constants() {
        // Verify key constants are different (all special keys are negative)
        assert_ne!(K_UP, K_DOWN);
        assert_ne!(K_LEFT, K_RIGHT);
        assert_ne!(K_S_LEFT, K_LEFT);
        assert_ne!(K_S_RIGHT, K_RIGHT);
        assert_ne!(K_C_LEFT, K_S_LEFT);
        assert_ne!(K_C_RIGHT, K_S_RIGHT);
    }
}
