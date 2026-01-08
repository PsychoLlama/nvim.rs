//! Normal mode key processing and command handling for Neovim
//!
//! This crate provides Rust implementations of normal mode functions
//! from `src/nvim/normal.c`. It handles normal and visual mode command
//! processing.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::{c_int, c_uint};

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

    // Modifier and scrolling functions
    fn nvim_get_mod_mask() -> c_int;
    fn nvim_goto_tabpage(n: c_int);
    fn nvim_pagescroll(dir: c_int, count: c_int, half: bool);
    fn nvim_get_VIsual_select() -> bool;
    fn nvim_set_VIsual_select(val: bool);
    fn nvim_may_trigger_modechanged();
    fn nvim_showmode();
    fn nvim_fileinfo(fullname: c_int, shorthelp: bool, dont_truncate: bool);
    fn nvim_scroll_redraw(down: bool, count: c_int);
    fn nvim_u_undo(count: c_int);
    fn nvim_curwin_set_curswant(val: bool);
    fn nvim_get_line_count() -> c_int;
    #[allow(dead_code)]
    fn nvim_get_cursor_lnum() -> c_int;
    fn nvim_set_cursor_lnum(lnum: c_int);
    fn nvim_setpcmark();
    fn nvim_beginline(flags: c_int);
    fn nvim_cursor_down(n: c_int, upd_topline: bool) -> bool;
    fn nvim_get_KeyTyped() -> bool;
    fn nvim_get_fdo_flags() -> c_uint;
    fn nvim_foldOpenCursor();
    fn nvim_set_ins_at_eol(val: bool);
    fn nvim_set_curswant(val: c_int);
    fn nvim_virtual_active() -> bool;
    fn nvim_gchar_cursor() -> c_int;
    #[allow(dead_code)]
    fn nvim_nv_pipe(cap: CapHandle);
    fn nvim_coladvance(col: c_int);

    // oparg_T motion accessors
    #[allow(dead_code)]
    fn nvim_oap_get_motion_type(oap: OapHandle) -> c_int;
    fn nvim_oap_set_motion_type(oap: OapHandle, val: c_int);
    #[allow(dead_code)]
    fn nvim_oap_get_inclusive(oap: OapHandle) -> bool;
    fn nvim_oap_set_inclusive(oap: OapHandle, val: bool);

    // cmdarg_T accessors
    fn nvim_cap_get_oap(cap: CapHandle) -> OapHandle;
    #[allow(dead_code)]
    fn nvim_cap_get_retval(cap: CapHandle) -> c_int;
    #[allow(dead_code)]
    fn nvim_cap_set_retval(cap: CapHandle, val: c_int);
    fn nvim_cap_or_retval(cap: CapHandle, val: c_int);
    fn nvim_cap_get_cmdchar(cap: CapHandle) -> c_int;
    #[allow(dead_code)]
    fn nvim_cap_set_cmdchar(cap: CapHandle, val: c_int);
    #[allow(dead_code)]
    fn nvim_cap_get_nchar(cap: CapHandle) -> c_int;
    #[allow(dead_code)]
    fn nvim_cap_set_nchar(cap: CapHandle, val: c_int);
    #[allow(dead_code)]
    fn nvim_cap_get_extra_char(cap: CapHandle) -> c_int;
    #[allow(dead_code)]
    fn nvim_cap_set_extra_char(cap: CapHandle, val: c_int);
    fn nvim_cap_get_count0(cap: CapHandle) -> c_int;
    fn nvim_cap_set_count0(cap: CapHandle, val: c_int);
    fn nvim_cap_get_count1(cap: CapHandle) -> c_int;
    fn nvim_cap_set_count1(cap: CapHandle, val: c_int);
    #[allow(dead_code)]
    fn nvim_cap_get_opcount(cap: CapHandle) -> c_int;
    #[allow(dead_code)]
    fn nvim_cap_set_opcount(cap: CapHandle, val: c_int);
    fn nvim_cap_get_arg(cap: CapHandle) -> c_int;
    fn nvim_cap_set_arg(cap: CapHandle, val: c_int);
    #[allow(dead_code)]
    fn nvim_cap_get_prechar(cap: CapHandle) -> c_int;
    #[allow(dead_code)]
    fn nvim_cap_set_prechar(cap: CapHandle, val: c_int);

    // C functions for command handlers
    fn ex_help(eap: *mut std::ffi::c_void);
    fn do_cmdline_cmd(cmd: *const std::ffi::c_char);
    fn end_visual_mode();
}

// Operator type constants (must match ops.h)
const OP_NOP: c_int = 0;

// NUL constant for motion_force
const NUL_CHAR: c_int = 0;

// Command retval constants (from normal_defs.h)
const CA_COMMAND_BUSY: c_int = 1;

/// Opaque handle to a window (win_T*).
pub type WinHandle = *mut std::ffi::c_void;

/// Opaque handle to operator arguments (oparg_T*).
pub type OapHandle = *mut std::ffi::c_void;

/// Opaque handle to command arguments (cmdarg_T*).
pub type CapHandle = *mut std::ffi::c_void;

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
// Command Handlers (Tier 1 - Simple handlers)
// =============================================================================

/// Command handler that ignores input but keeps command busy.
///
/// Sets CA_COMMAND_BUSY flag to skip restarting edit() once.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_ignore(cap: CapHandle) {
    nvim_cap_or_retval(cap, CA_COMMAND_BUSY);
}

/// Command handler that does nothing.
///
/// Unlike nv_ignore, this does start edit().
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer (unused).
#[no_mangle]
pub extern "C" fn rs_nv_nop(_cap: CapHandle) {
    // Empty - does nothing but unlike nv_ignore does start edit()
}

/// Command handler for non-existent commands.
///
/// Clears any pending operator and beeps.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_error(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    rs_clearopbeep(oap);
}

/// Command handler for <Help> and <F1> commands.
///
/// Shows help if no operator is pending.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_help(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    if !rs_checkclearopq(oap) {
        ex_help(std::ptr::null_mut());
    }
}

/// Command handler for CTRL-Z (suspend).
///
/// Clears operator, ends visual mode if active, and executes ":st".
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_suspend(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    rs_clearop(oap);
    if nvim_get_VIsual_active() != 0 {
        end_visual_mode();
    }
    do_cmdline_cmd(c"st".as_ptr());
}

// =============================================================================
// Command Handlers (Tier 2 - Scrolling commands)
// =============================================================================

// Modifier mask constant
const MOD_MASK_CTRL: c_int = 0x04;

// Control character constant
const CTRL_D: c_int = 4; // Ctrl-D

// Motion type constants (from normal_defs.h)
const K_MT_CHAR_WISE: c_int = 0;
const K_MT_LINE_WISE: c_int = 1;
#[allow(dead_code)]
const K_MT_BLOCK_WISE: c_int = 2;

// Fold option flags (kOptFdoFlag*)
const K_OPT_FDO_FLAG_HOR: c_uint = 0x0001;
const K_OPT_FDO_FLAG_JUMP: c_uint = 0x0040;

// Beginline flags (BL_*)
const BL_SOL: c_int = 2; // go to start of line
const BL_FIX: c_int = 4; // fix cursor column

// Maximum column value
const MAXCOL: c_int = 0x7fff_ffff;

/// Command handler for CTRL-F, CTRL-B, etc: Scroll page up or down.
///
/// Handles page scrolling and tab page navigation with Ctrl modifier.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_page(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    if rs_checkclearop(oap) {
        return;
    }

    let arg = nvim_cap_get_arg(cap);
    let count0 = nvim_cap_get_count0(cap);
    let count1 = nvim_cap_get_count1(cap);

    if (nvim_get_mod_mask() & MOD_MASK_CTRL) != 0 {
        // <C-PageUp>: tab page back; <C-PageDown>: tab page forward
        if arg == BACKWARD {
            nvim_goto_tabpage(-count1);
        } else {
            nvim_goto_tabpage(count0);
        }
    } else {
        nvim_pagescroll(arg, count1, false);
    }
}

/// Command handler for CTRL-D, CTRL-U: Scroll half page.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_halfpage(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    if !rs_checkclearop(oap) {
        let cmdchar = nvim_cap_get_cmdchar(cap);
        let count0 = nvim_cap_get_count0(cap);
        let dir = if cmdchar == CTRL_D { FORWARD } else { BACKWARD };
        nvim_pagescroll(dir, count0, true);
    }
}

/// Command handler for CTRL-G: Show file info or toggle Select/Visual mode.
///
/// In Visual mode, toggles between Visual and Select mode.
/// Otherwise, shows file information.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_ctrlg(cap: CapHandle) {
    if nvim_get_VIsual_active() != 0 {
        // toggle Selection/Visual mode
        let select = nvim_get_VIsual_select();
        nvim_set_VIsual_select(!select);
        nvim_may_trigger_modechanged();
        nvim_showmode();
    } else {
        let oap = nvim_cap_get_oap(cap);
        if !rs_checkclearop(oap) {
            // print full name if count given or :cd used
            let count0 = nvim_cap_get_count0(cap);
            nvim_fileinfo(count0, false, true);
        }
    }
}

/// Command handler for CTRL-E and CTRL-Y: scroll a line up or down.
///
/// cap->arg must be true (non-zero) for CTRL-E.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_scroll_line(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    if !rs_checkclearop(oap) {
        let arg = nvim_cap_get_arg(cap);
        let count1 = nvim_cap_get_count1(cap);
        nvim_scroll_redraw(arg != 0, count1);
    }
}

/// Command handler for <Undo> command.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_kundo(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    if rs_checkclearopq(oap) {
        return;
    }

    let count1 = nvim_cap_get_count1(cap);
    nvim_u_undo(count1);
    nvim_curwin_set_curswant(true);
}

// =============================================================================
// Command Handlers (Tier 3 - Motion commands)
// =============================================================================

/// Command handler for "G", "gg", CTRL-END, CTRL-HOME.
///
/// cap->arg is true (non-zero) for "G".
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_goto(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    let arg = nvim_cap_get_arg(cap);
    let count0 = nvim_cap_get_count0(cap);

    let line_count = nvim_get_line_count();
    let mut lnum = if arg != 0 { line_count } else { 1 };

    nvim_oap_set_motion_type(oap, K_MT_LINE_WISE);
    nvim_setpcmark();

    // When a count is given, use it instead of the default lnum
    if count0 != 0 {
        lnum = count0;
    }
    lnum = lnum.clamp(1, line_count);
    nvim_set_cursor_lnum(lnum);
    nvim_beginline(BL_SOL | BL_FIX);

    if (nvim_get_fdo_flags() & K_OPT_FDO_FLAG_JUMP) != 0
        && nvim_get_KeyTyped()
        && nvim_oap_get_op_type_ptr(oap) == OP_NOP
    {
        nvim_foldOpenCursor();
    }
}

/// Command handler for "0" and "^" commands.
///
/// cap->arg is the argument for beginline().
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_beginline(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    let arg = nvim_cap_get_arg(cap);

    nvim_oap_set_motion_type(oap, K_MT_CHAR_WISE);
    nvim_oap_set_inclusive(oap, false);
    nvim_beginline(arg);

    if (nvim_get_fdo_flags() & K_OPT_FDO_FLAG_HOR) != 0
        && nvim_get_KeyTyped()
        && nvim_oap_get_op_type_ptr(oap) == OP_NOP
    {
        nvim_foldOpenCursor();
    }
    nvim_set_ins_at_eol(false);
}

/// Command handler for "$" command.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_dollar(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    let count1 = nvim_cap_get_count1(cap);

    nvim_oap_set_motion_type(oap, K_MT_CHAR_WISE);
    nvim_oap_set_inclusive(oap, true);

    // In virtual mode when off the edge of a line and an operator
    // is pending (whew!) keep the cursor where it is.
    // Otherwise, send it to the end of the line.
    if !nvim_virtual_active() || nvim_gchar_cursor() != 0 || nvim_oap_get_op_type_ptr(oap) == OP_NOP
    {
        nvim_set_curswant(MAXCOL); // so we stay at the end
    }

    if !nvim_cursor_down(count1 - 1, nvim_oap_get_op_type_ptr(oap) == OP_NOP) {
        rs_clearopbeep(oap);
    } else if (nvim_get_fdo_flags() & K_OPT_FDO_FLAG_HOR) != 0
        && nvim_get_KeyTyped()
        && nvim_oap_get_op_type_ptr(oap) == OP_NOP
    {
        nvim_foldOpenCursor();
    }
}

/// Command handler for <End>: to end of current line or last line.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_end(cap: CapHandle) {
    let arg = nvim_cap_get_arg(cap);

    if arg != 0 || (nvim_get_mod_mask() & MOD_MASK_CTRL) != 0 {
        // CTRL-END = goto last line
        nvim_cap_set_arg(cap, 1);
        rs_nv_goto(cap);
        nvim_cap_set_count1(cap, 1); // to end of current line
    }
    rs_nv_dollar(cap);
}

/// Command handler for <Home>: to column 1 or first line.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_home(cap: CapHandle) {
    // CTRL-HOME is like "gg"
    if (nvim_get_mod_mask() & MOD_MASK_CTRL) != 0 {
        rs_nv_goto(cap);
    } else {
        nvim_cap_set_count0(cap, 1);
        rs_nv_pipe(cap);
    }
    nvim_set_ins_at_eol(false);
}

/// Command handler for "|" command: go to column.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_pipe(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    let count0 = nvim_cap_get_count0(cap);

    nvim_oap_set_motion_type(oap, K_MT_CHAR_WISE);
    nvim_oap_set_inclusive(oap, false);
    nvim_beginline(0);

    if count0 > 0 {
        nvim_coladvance(count0 - 1);
        nvim_set_curswant(count0 - 1);
    } else {
        nvim_set_curswant(0);
    }
    // keep curswant at the column where we wanted to go, not where
    // we ended; differs if line is too short
    nvim_curwin_set_curswant(false);
}

// =============================================================================
// Word Motion Accessors
// =============================================================================

extern "C" {
    // Word motion functions
    fn nvim_curwin_set_set_curswant(val: bool);
    fn nvim_fwd_word(count: c_int, bigword: bool, eol: bool) -> c_int;
    fn nvim_bck_word(count: c_int, bigword: bool, stop: bool) -> c_int;
    fn nvim_end_word(count: c_int, bigword: bool, stop: bool, empty: bool) -> c_int;
    #[allow(dead_code)]
    fn nvim_bckend_word(count: c_int, bigword: bool, eol: bool) -> c_int;
    fn nvim_findsent(dir: c_int, count: c_int) -> c_int;
    fn nvim_findpar(pincl: *mut bool, dir: c_int, count: c_int, what: c_int, both: bool) -> bool;
    fn nvim_get_cursor_col() -> c_int;
    fn nvim_set_cursor_col(col: c_int);
    fn nvim_set_cursor_coladd_zero();
    fn nvim_gchar_cursor_call() -> c_int;
    fn nvim_inc_cursor() -> c_int;
    fn nvim_mb_adjust_cursor();
    fn nvim_cpo_has_changew() -> bool;
    fn nvim_ascii_iswhite(c: c_int) -> bool;
    fn nvim_get_p_sel_first() -> std::ffi::c_char;
    fn nvim_lt_VIsual_cursor() -> bool;
    fn nvim_lt_pos_cursor(lnum: c_int, col: c_int) -> bool;
    fn nvim_set_VIsual_select_exclu_adj(val: bool);
    fn nvim_get_ve_flags() -> c_uint;

    // Character search functions
    fn nvim_get_VIsual_mode() -> c_int;
    fn nvim_get_VIsual_select_exclu_adj() -> bool;
    fn nvim_unadjust_for_sel() -> bool;
    fn nvim_searchc(cap: CapHandle, t_cmd: bool) -> c_int;
    fn nvim_is_special(key: c_int) -> bool;
    fn nvim_getvcol_cursor(scol: *mut c_int, ecol: *mut c_int);
    fn nvim_set_cursor_coladd(val: c_int);
    fn nvim_get_TAB() -> c_int;

    // Mark command functions
    fn nvim_setmark(name: c_int) -> bool;
    fn nvim_get_jop_flags() -> c_uint;
    fn nvim_mark_get(name: c_int) -> FmarkHandle;
    fn nvim_nv_mark_move_to(cap: CapHandle, flags: c_int, fm: FmarkHandle) -> c_int;
    fn nvim_get_changelist(count1: c_int) -> FmarkHandle;
    fn nvim_get_jumplist(count1: c_int) -> FmarkHandle;
    fn nvim_goto_tabpage_lastused() -> bool;
    fn nvim_get_changelistlen() -> c_int;
    fn nvim_emsg(msg: *const std::ffi::c_char);
    fn nvim_get_e_changelist_is_empty() -> *const std::ffi::c_char;
    fn nvim_get_e_start_of_changelist() -> *const std::ffi::c_char;
    fn nvim_get_e_end_of_changelist() -> *const std::ffi::c_char;

    // Register command functions
    fn nvim_get_expr_register() -> c_int;
    fn nvim_valid_yank_reg(regname: c_int, writing: bool) -> bool;
    fn nvim_set_reg_var(regname: c_int);
    fn nvim_nv_put_opt(cap: CapHandle, fix_indent: bool);

    // Visual mode functions
    fn nvim_nv_visual_impl(cap: CapHandle);

    // Window command functions
    fn nvim_do_window(nchar: c_int, count: c_int, xchar: c_int);
    fn nvim_nv_colon(cap: CapHandle);
}

/// Opaque handle to fmark_T*.
pub type FmarkHandle = *mut std::ffi::c_void;

// Operator type constant for OP_CHANGE
const OP_CHANGE: c_int = 5;

// Fold option flag for block
const K_OPT_FDO_FLAG_BLOCK: c_uint = 0x0002;

// Virtual edit flag for onemore
const K_OPT_VE_FLAG_ONEMORE: c_uint = 0x0004;

// FAIL return value
const FAIL: c_int = 0;

// =============================================================================
// Word Motion Command Handlers
// =============================================================================

/// Used after a movement command: If the cursor ends up on the NUL after the
/// end of the line, may move it back to the last character and make the motion
/// inclusive.
///
/// # Safety
/// `oap` must be a valid oparg_T pointer.
#[inline]
unsafe fn adjust_cursor(oap: OapHandle) {
    // The cursor cannot remain on the NUL when:
    // - the column is > 0
    // - not in Visual mode or 'selection' is "o"
    // - 'virtualedit' is not "all" and not "onemore".
    #[allow(clippy::cast_possible_wrap)]
    let sel_o = b'o' as std::ffi::c_char;
    if nvim_get_cursor_col() > 0
        && nvim_gchar_cursor_call() == NUL_CHAR
        && (nvim_get_VIsual_active() == 0 || nvim_get_p_sel_first() == sel_o)
        && !nvim_virtual_active()
        && (nvim_get_ve_flags() & K_OPT_VE_FLAG_ONEMORE) == 0
    {
        nvim_set_cursor_col(nvim_get_cursor_col() - 1);
        // prevent cursor from moving on the trail byte
        nvim_mb_adjust_cursor();
        nvim_oap_set_inclusive(oap, true);
    }
}

/// In exclusive Visual mode, may include the last character.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[inline]
unsafe fn adjust_for_sel(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    #[allow(clippy::cast_possible_wrap)]
    let sel_e = b'e' as std::ffi::c_char;
    if nvim_get_VIsual_active() != 0
        && nvim_oap_get_inclusive(oap)
        && nvim_get_p_sel_first() == sel_e
        && nvim_gchar_cursor_call() != NUL_CHAR
        && nvim_lt_VIsual_cursor()
    {
        nvim_inc_cursor();
        nvim_oap_set_inclusive(oap, false);
        nvim_set_VIsual_select_exclu_adj(true);
    }
}

/// Command handler for "b" and "B" commands.
///
/// cap->arg is 1 for "B".
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_bck_word(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    let count1 = nvim_cap_get_count1(cap);
    let arg = nvim_cap_get_arg(cap);

    nvim_oap_set_motion_type(oap, K_MT_CHAR_WISE);
    nvim_oap_set_inclusive(oap, false);
    nvim_curwin_set_set_curswant(true);

    if nvim_bck_word(count1, arg != 0, false) == FAIL {
        rs_clearopbeep(oap);
    } else if (nvim_get_fdo_flags() & K_OPT_FDO_FLAG_HOR) != 0
        && nvim_get_KeyTyped()
        && nvim_oap_get_op_type_ptr(oap) == OP_NOP
    {
        nvim_foldOpenCursor();
    }
}

/// Command handler for "e", "E", "w" and "W" commands.
///
/// cap->arg is true (non-zero) for "E" and "W".
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_wordcmd(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    let count1 = nvim_cap_get_count1(cap);
    let arg = nvim_cap_get_arg(cap);
    let cmdchar = nvim_cap_get_cmdchar(cap);

    // Save starting position for later comparison
    let start_lnum = nvim_get_cursor_lnum();
    let start_col = nvim_get_cursor_col();

    // Set inclusive for the "E" and "e" command.
    let mut word_end = cmdchar == c_int::from(b'e') || cmdchar == c_int::from(b'E');
    nvim_oap_set_inclusive(oap, word_end);

    let mut flag = false;

    // "cw" and "cW" are a special case.
    if !word_end && nvim_oap_get_op_type_ptr(oap) == OP_CHANGE {
        let n = nvim_gchar_cursor_call();
        if n != NUL_CHAR && !nvim_ascii_iswhite(n) {
            // This is a little strange. To match what the real Vi does, we
            // effectively map "cw" to "ce", and "cW" to "cE", provided that we are
            // not on a space or a TAB. This seems impolite at first, but it's
            // really more what we mean when we say "cw".
            //
            // Another strangeness: When standing on the end of a word "ce" will
            // change until the end of the next word, but "cw" will change only one
            // character! This is done by setting "flag".
            if nvim_cpo_has_changew() {
                nvim_oap_set_inclusive(oap, true);
                word_end = true;
            }
            flag = true;
        }
    }

    nvim_oap_set_motion_type(oap, K_MT_CHAR_WISE);
    nvim_curwin_set_set_curswant(true);

    let n = if word_end {
        nvim_end_word(count1, arg != 0, flag, false)
    } else {
        nvim_fwd_word(count1, arg != 0, nvim_oap_get_op_type_ptr(oap) != OP_NOP)
    };

    // Don't leave the cursor on the NUL past the end of line. Unless we
    // didn't move it forward.
    if nvim_lt_pos_cursor(start_lnum, start_col) {
        adjust_cursor(oap);
    }

    if n == FAIL && nvim_oap_get_op_type_ptr(oap) == OP_NOP {
        rs_clearopbeep(oap);
    } else {
        adjust_for_sel(cap);
        if (nvim_get_fdo_flags() & K_OPT_FDO_FLAG_HOR) != 0
            && nvim_get_KeyTyped()
            && nvim_oap_get_op_type_ptr(oap) == OP_NOP
        {
            nvim_foldOpenCursor();
        }
    }
}

/// Command handler for "{" and "}" commands.
///
/// cap->arg is BACKWARD for "{" and FORWARD for "}".
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_findpar(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    let arg = nvim_cap_get_arg(cap);
    let count1 = nvim_cap_get_count1(cap);

    nvim_oap_set_motion_type(oap, K_MT_CHAR_WISE);
    nvim_oap_set_inclusive(oap, false);
    nvim_oap_set_use_reg_one(oap, true);
    nvim_curwin_set_set_curswant(true);

    let mut inclusive = false;
    if !nvim_findpar(&raw mut inclusive, arg, count1, NUL_CHAR, false) {
        rs_clearopbeep(oap);
        return;
    }
    nvim_oap_set_inclusive(oap, inclusive);

    nvim_set_cursor_coladd_zero();
    if (nvim_get_fdo_flags() & K_OPT_FDO_FLAG_BLOCK) != 0
        && nvim_get_KeyTyped()
        && nvim_oap_get_op_type_ptr(oap) == OP_NOP
    {
        nvim_foldOpenCursor();
    }
}

/// Command handler for "(" and ")" commands.
///
/// cap->arg is BACKWARD for "(" and FORWARD for ")".
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_brace(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    let arg = nvim_cap_get_arg(cap);
    let count1 = nvim_cap_get_count1(cap);

    nvim_oap_set_motion_type(oap, K_MT_CHAR_WISE);
    nvim_oap_set_use_reg_one(oap, true);
    // The motion used to be inclusive for "(", but that is not what Vi does.
    nvim_oap_set_inclusive(oap, false);
    nvim_curwin_set_set_curswant(true);

    if nvim_findsent(arg, count1) == FAIL {
        rs_clearopbeep(oap);
        return;
    }

    // Don't leave the cursor on the NUL past end of line.
    adjust_cursor(oap);
    nvim_set_cursor_coladd_zero();
    if (nvim_get_fdo_flags() & K_OPT_FDO_FLAG_BLOCK) != 0
        && nvim_get_KeyTyped()
        && nvim_oap_get_op_type_ptr(oap) == OP_NOP
    {
        nvim_foldOpenCursor();
    }
}

// Mark move flags (from mark_defs.h)
const K_MARK_SET_VIEW: c_int = 0x01;
const K_MARK_NO_CONTEXT: c_int = 0x02;
const K_MARK_CONTEXT: c_int = 0x04;
const K_MARK_BEGIN_LINE: c_int = 0x08;
const K_MARK_JUMP_LIST: c_int = 0x10;

// Mark move result flags
const K_MARK_MOVE_SUCCESS: c_int = 0x01;
const K_MARK_SWITCHED_BUF: c_int = 0x02;
const K_MARK_CHANGED_CURSOR: c_int = 0x04;
const K_MARK_CHANGED_LINE: c_int = 0x08;

// Jop flag for view
const K_OPT_JOP_FLAG_VIEW: c_uint = 0x0004;

// Fold flag for mark
const K_OPT_FDO_FLAG_MARK: c_uint = 0x0080;

// MOD_MASK_CTRL
const MOD_MASK_CTRL_VALUE: c_int = 0x04;

// TAB character
const TAB_CHAR: c_int = 9;

// =============================================================================
// Mark Command Handlers
// =============================================================================

/// Command handler for "m" command: Mark a position.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_mark(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    if rs_checkclearop(oap) {
        return;
    }

    let nchar = nvim_cap_get_nchar(cap);
    if !nvim_setmark(nchar) {
        rs_clearopbeep(oap);
    }
}

/// Command handler for "'" and "`" commands. Also for "g'" and "g`".
///
/// cap->arg is true for "'" and "g'".
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_gomark(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    let cmdchar = nvim_cap_get_cmdchar(cap);
    let arg = nvim_cap_get_arg(cap);
    let count0 = nvim_cap_get_count0(cap);

    // flags for moving to the mark
    // When there is a pending operator, do not restore the view as this is usually unexpected.
    let mut flags: c_int = if nvim_oap_get_op_type_ptr(oap) != OP_NOP {
        0
    } else if (nvim_get_jop_flags() & K_OPT_JOP_FLAG_VIEW) != 0 {
        K_MARK_SET_VIEW
    } else {
        0
    };
    let old_key_typed = nvim_get_KeyTyped();

    let name: c_int;
    if cmdchar == c_int::from(b'g') {
        let extra_char = nvim_cap_get_extra_char(cap);
        name = extra_char;
        flags |= K_MARK_NO_CONTEXT;
    } else {
        name = nvim_cap_get_nchar(cap);
        flags |= K_MARK_CONTEXT;
    }
    if arg != 0 {
        flags |= K_MARK_BEGIN_LINE;
    }
    if count0 != 0 {
        flags |= K_MARK_SET_VIEW;
    }

    let fm = nvim_mark_get(name);
    let move_res = nvim_nv_mark_move_to(cap, flags, fm);

    // May need to clear the coladd that a mark includes.
    if !nvim_virtual_active() {
        nvim_set_cursor_coladd(0);
    }

    if nvim_oap_get_op_type_ptr(oap) == OP_NOP
        && (move_res & K_MARK_MOVE_SUCCESS) != 0
        && ((move_res & K_MARK_SWITCHED_BUF) != 0 || (move_res & K_MARK_CHANGED_CURSOR) != 0)
        && (nvim_get_fdo_flags() & K_OPT_FDO_FLAG_MARK) != 0
        && old_key_typed
    {
        nvim_foldOpenCursor();
    }
}

/// Command handler for CTRL-O, CTRL-I, "g;", "g,", and "CTRL-Tab" commands.
///
/// Movement in the jumplist and changelist.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_pcmark(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    let cmdchar = nvim_cap_get_cmdchar(cap);
    let count1 = nvim_cap_get_count1(cap);

    // flags for moving to the mark
    let mut flags: c_int = if (nvim_get_jop_flags() & K_OPT_JOP_FLAG_VIEW) != 0 {
        K_MARK_SET_VIEW
    } else {
        0
    };
    let old_key_typed = nvim_get_KeyTyped();

    if rs_checkclearopq(oap) {
        return;
    }

    if cmdchar == TAB_CHAR && nvim_get_mod_mask() == MOD_MASK_CTRL_VALUE {
        if !nvim_goto_tabpage_lastused() {
            rs_clearopbeep(oap);
        }
        return;
    }

    let fm: FmarkHandle;
    if cmdchar == c_int::from(b'g') {
        fm = nvim_get_changelist(count1);
    } else {
        fm = nvim_get_jumplist(count1);
        flags |= K_MARK_NO_CONTEXT | K_MARK_JUMP_LIST;
    }

    // Changelist and jumplist have their own error messages. Therefore avoid
    // calling nv_mark_move_to() when not found to avoid incorrect error messages.
    let move_res: c_int;
    if !fm.is_null() {
        move_res = nvim_nv_mark_move_to(cap, flags, fm);
    } else if cmdchar == c_int::from(b'g') {
        if nvim_get_changelistlen() == 0 {
            nvim_emsg(nvim_get_e_changelist_is_empty());
        } else if count1 < 0 {
            nvim_emsg(nvim_get_e_start_of_changelist());
        } else {
            nvim_emsg(nvim_get_e_end_of_changelist());
        }
        return;
    } else {
        rs_clearopbeep(oap);
        return;
    }

    if nvim_oap_get_op_type_ptr(oap) == OP_NOP
        && ((move_res & K_MARK_SWITCHED_BUF) != 0 || (move_res & K_MARK_CHANGED_LINE) != 0)
        && (nvim_get_fdo_flags() & K_OPT_FDO_FLAG_MARK) != 0
        && old_key_typed
    {
        nvim_foldOpenCursor();
    }
}

// =============================================================================
// Visual Mode Command Handlers
// =============================================================================

/// Command handler for "v", "V" and "CTRL-V" commands.
///
/// Also for "gh", "gH" and "g^H" commands: Always start Select mode, cap->arg
/// is true. Handle CTRL-Q just like CTRL-V.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_visual(cap: CapHandle) {
    // Delegate to C implementation which handles the complex visual mode logic
    nvim_nv_visual_impl(cap);
}

// =============================================================================
// Window Command Handlers
// =============================================================================

/// Command handler for CTRL-W commands.
///
/// "CTRL-W :" is the same as typing ":"; useful in a terminal window.
/// Otherwise, delegate to do_window() for window operations.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_window(cap: CapHandle) {
    let nchar = nvim_cap_get_nchar(cap);
    let oap = nvim_cap_get_oap(cap);

    if nchar == c_int::from(b':') {
        // "CTRL-W :" is the same as typing ":"
        nvim_cap_set_cmdchar(cap, c_int::from(b':'));
        nvim_cap_set_nchar(cap, c_int::from(NUL));
        nvim_nv_colon(cap);
    } else if !rs_checkclearop(oap) {
        let count0 = nvim_cap_get_count0(cap);
        nvim_do_window(nchar, count0, c_int::from(NUL));
    }
}

// =============================================================================
// Register Command Handlers
// =============================================================================

/// Command handler for '"' command: Select register for next command.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_regname(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    if rs_checkclearop(oap) {
        return;
    }

    let mut nchar = nvim_cap_get_nchar(cap);
    let eq_char = c_int::from(b'=');
    if nchar == eq_char {
        nchar = nvim_get_expr_register();
    }
    if nchar != NUL_CHAR && nvim_valid_yank_reg(nchar, false) {
        nvim_oap_set_regname(oap, nchar);
        let count0 = nvim_cap_get_count0(cap);
        nvim_cap_set_opcount(cap, count0); // remember count before '"'
        nvim_set_reg_var(nchar);
    } else {
        rs_clearopbeep(oap);
    }
}

/// Command handler for "p" command.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_put(cap: CapHandle) {
    nvim_nv_put_opt(cap, false);
}

// =============================================================================
// Character Search Command Handlers
// =============================================================================

/// Command handler for f, F, t, T, ; and , commands.
///
/// cap->arg is BACKWARD for 'F' and 'T', FORWARD for 'f' and 't', true for
/// ',' and false for ';'.
/// cap->nchar is NUL for ',' and ';' (repeat the search)
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_csearch(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    let cmdchar = nvim_cap_get_cmdchar(cap);
    let nchar = nvim_cap_get_nchar(cap);
    let arg = nvim_cap_get_arg(cap);

    let mut cursor_dec = false;

    // If adjusted cursor position previously, unadjust it.
    #[allow(clippy::cast_possible_wrap)]
    let sel_e = b'e' as std::ffi::c_char;
    let visual_v = c_int::from(b'v');
    if nvim_get_p_sel_first() == sel_e
        && nvim_get_VIsual_active() != 0
        && nvim_get_VIsual_mode() == visual_v
        && nvim_get_VIsual_select_exclu_adj()
    {
        nvim_unadjust_for_sel();
        cursor_dec = true;
    }

    let t_cmd = cmdchar == c_int::from(b't') || cmdchar == c_int::from(b'T');

    nvim_oap_set_motion_type(oap, K_MT_CHAR_WISE);
    if nvim_is_special(nchar) || nvim_searchc(cap, t_cmd) == FAIL {
        rs_clearopbeep(oap);
        // Revert unadjust when failed.
        if cursor_dec {
            adjust_for_sel(cap);
        }
        return;
    }

    nvim_curwin_set_set_curswant(true);
    // Include a Tab for "tx" and for "dfx".
    if nvim_gchar_cursor_call() == nvim_get_TAB()
        && nvim_virtual_active()
        && arg == FORWARD
        && (t_cmd || nvim_oap_get_op_type_ptr(oap) != OP_NOP)
    {
        let mut scol: c_int = 0;
        let mut ecol: c_int = 0;
        nvim_getvcol_cursor(&raw mut scol, &raw mut ecol);
        nvim_set_cursor_coladd(ecol - scol);
    } else {
        nvim_set_cursor_coladd(0);
    }
    adjust_for_sel(cap);
    if (nvim_get_fdo_flags() & K_OPT_FDO_FLAG_HOR) != 0
        && nvim_get_KeyTyped()
        && nvim_oap_get_op_type_ptr(oap) == OP_NOP
    {
        nvim_foldOpenCursor();
    }
}

// =============================================================================
// Phase 1 Command Handlers
// =============================================================================

extern "C" {
    // Phase 1 accessor functions
    fn nvim_nv_clear_impl();
    #[allow(dead_code)]
    fn nvim_get_restart_VIsual_select() -> c_int;
    fn nvim_set_restart_VIsual_select(val: c_int);
    fn nvim_buflist_getfile(n: c_int, lnum: c_int, flags: c_int, setpm: bool);
    fn nvim_get_GETF_SETMARK() -> c_int;
    fn nvim_get_GETF_ALT() -> c_int;
    fn nvim_nv_Zet_impl(cap: CapHandle);
    fn nvim_nv_esc_impl(cap: CapHandle);
    fn nvim_nv_edit_impl(cap: CapHandle);
}

/// Command handler for CTRL-L: Clear and redraw screen.
///
/// Clears all syntax states to force resyncing and redraws the screen.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_clear(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    if rs_checkclearop(oap) {
        return;
    }
    nvim_nv_clear_impl();
}

/// Command handler for CTRL-O: Switch to Visual mode for one command or go to older pcmark.
///
/// In Select mode: switch to Visual mode for one command.
/// Otherwise: Go to older pcmark (calls nv_pcmark with negated count).
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_ctrlo(cap: CapHandle) {
    if nvim_get_VIsual_active() != 0 && nvim_get_VIsual_select() {
        nvim_set_VIsual_select(false);
        nvim_may_trigger_modechanged();
        nvim_showmode();
        nvim_set_restart_VIsual_select(2); // restart Select mode later
    } else {
        // Negate count1 for backward jump
        let count1 = nvim_cap_get_count1(cap);
        nvim_cap_set_count1(cap, -count1);
        rs_nv_pcmark(cap);
    }
}

/// Command handler for CTRL-^: Edit alternate file.
///
/// Short for ":e #". Works even when the alternate buffer is not named.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_hat(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    if !rs_checkclearopq(oap) {
        let count0 = nvim_cap_get_count0(cap);
        let flags = nvim_get_GETF_SETMARK() | nvim_get_GETF_ALT();
        nvim_buflist_getfile(count0, 0, flags, false);
    }
}

/// Command handler for "Z" commands (ZZ, ZQ).
///
/// ZZ: equivalent to ":x" (save and quit).
/// ZQ: equivalent to ":q!" (quit without saving).
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_Zet(cap: CapHandle) {
    // Delegate to C implementation which handles the switch logic
    nvim_nv_Zet_impl(cap);
}

/// Command handler for <Esc> and CTRL-C.
///
/// Handles escape from various modes, clears operators, and may show exit messages.
/// cap->arg is true for CTRL-C.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_esc(cap: CapHandle) {
    // Delegate to C implementation which handles complex logic
    nvim_nv_esc_impl(cap);
}

/// Command handler for "A", "a", "I", "i" and <Insert> commands.
///
/// Handles entering insert mode with various cursor positioning:
/// - A: Append after the line
/// - a: Append after cursor
/// - I: Insert before first non-blank
/// - i: Insert before cursor
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_edit(cap: CapHandle) {
    // Delegate to C implementation which handles complex logic
    nvim_nv_edit_impl(cap);
}

// =============================================================================
// Phase 2 Command Handlers (Search)
// =============================================================================

extern "C" {
    // Phase 2 accessor functions
    fn nvim_nv_search_impl(cap: CapHandle);
    fn nvim_nv_next_impl(cap: CapHandle);
    fn nvim_nv_ident_impl(cap: CapHandle);
}

/// Command handler for "/" and "?" commands: Search forward/backward.
///
/// cap->arg is true to not set PC mark.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_search(cap: CapHandle) {
    // Delegate to C implementation which handles command-line input
    nvim_nv_search_impl(cap);
}

/// Command handler for "n" and "N" commands: Repeat search.
///
/// cap->arg is SEARCH_REV for "N", 0 for "n".
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_next(cap: CapHandle) {
    // Delegate to C implementation which handles search state
    nvim_nv_next_impl(cap);
}

/// Command handler for identifier commands: *, #, K, CTRL-], g], g*.
///
/// Handles searching for the word under cursor and related operations.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_ident(cap: CapHandle) {
    // Delegate to C implementation which handles string manipulation
    nvim_nv_ident_impl(cap);
}

// =============================================================================
// Phase 3 Command Handlers (Operators)
// =============================================================================

extern "C" {
    // Phase 3 accessor functions
    fn nvim_nv_operator_impl(cap: CapHandle);
    fn nvim_nv_optrans_impl(cap: CapHandle);
    fn nvim_nv_tilde_impl(cap: CapHandle);
    fn nvim_nv_subst_impl(cap: CapHandle);
}

/// Command handler for operator commands (d, c, y, >, <, !, =, gq, gw, g?, etc.).
///
/// Sets up the operator state; actual work is done by do_pending_operator().
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_operator(cap: CapHandle) {
    nvim_nv_operator_impl(cap);
}

/// Command handler for abbreviated commands (x, X, D, C, s, S, Y, &).
///
/// Translates these commands to their full equivalents.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_optrans(cap: CapHandle) {
    nvim_nv_optrans_impl(cap);
}

/// Command handler for '~' command: Toggle case.
///
/// If tilde is not an operator and Visual is off, swaps case of a single character.
/// Otherwise, acts as an operator.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_tilde(cap: CapHandle) {
    nvim_nv_tilde_impl(cap);
}

/// Command handler for "s" and "S" substitute commands.
///
/// In Visual mode, "vs" and "vS" are the same as "vc".
/// Otherwise, translates to the equivalent command.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_subst(cap: CapHandle) {
    nvim_nv_subst_impl(cap);
}

// =============================================================================
// Phase 4: Text object handlers
// =============================================================================

extern "C" {
    fn nvim_nv_object_impl(cap: CapHandle);
    fn nvim_nv_select_impl(cap: CapHandle);
    fn nvim_nv_brackets_impl(cap: CapHandle);
}

/// Command handler for "a" or "i" text objects.
///
/// Handles text object selection when an operator is pending or in Visual mode.
/// Examples: "aw" (a word), "iw" (inner word), "a(" (a parentheses block), etc.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_object(cap: CapHandle) {
    nvim_nv_object_impl(cap);
}

/// Command handler for SELECT key in Normal or Visual mode.
///
/// In Visual mode, switches to Select mode.
/// Otherwise, if VIsual_reselect is set, fakes a "gv" command.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_select(cap: CapHandle) {
    nvim_nv_select_impl(cap);
}

/// Command handler for `[` and `]` bracket commands.
///
/// Handles various bracket-related motions and commands:
/// - `[f` / `]f`: Edit file under cursor (same as `gf`)
/// - `[i` / `]i`: Find identifier under cursor
/// - `[{` / `]}`: Go to unclosed brace
/// - `[[` / `]]`: Move to start/end of function
/// - `[p` / `]p`: Put with indent adjustment
/// - And many more...
///
/// `cap->arg` is BACKWARD for `[` and FORWARD for `]`.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_brackets(cap: CapHandle) {
    nvim_nv_brackets_impl(cap);
}

// =============================================================================
// Phase 5: Undo/Redo handlers
// =============================================================================

extern "C" {
    fn nvim_nv_undo_impl(cap: CapHandle);
    fn nvim_nv_Undo_impl(cap: CapHandle);
    fn nvim_nv_dot_impl(cap: CapHandle);
    fn nvim_nv_redo_or_register_impl(cap: CapHandle);
}

/// Command handler for "u" undo command.
///
/// In Visual mode or when `op_type` is `OP_LOWER`, translates to `gu` command.
/// Otherwise performs undo via `nv_kundo`.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_undo(cap: CapHandle) {
    nvim_nv_undo_impl(cap);
}

/// Command handler for "U" line undo command.
///
/// In Visual mode or when `op_type` is `OP_UPPER`, translates to `gU` command.
/// Otherwise performs line undo via `u_undoline`.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_Undo(cap: CapHandle) {
    nvim_nv_Undo_impl(cap);
}

/// Command handler for "." repeat command.
///
/// Repeats the last change. If `restart_edit` is true, repeats the last but one
/// command instead (used for CTRL-O <.> in insert mode).
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_dot(cap: CapHandle) {
    nvim_nv_dot_impl(cap);
}

/// Command handler for CTRL-R (redo or register selection).
///
/// In Visual select mode, selects a register for the next operation.
/// Otherwise, performs redo via `u_redo`.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_redo_or_register(cap: CapHandle) {
    nvim_nv_redo_or_register_impl(cap);
}

// =============================================================================
// Phase 6: Insert mode entry handlers
// =============================================================================

extern "C" {
    fn nvim_nv_replace_impl(cap: CapHandle);
    fn nvim_nv_Replace_impl(cap: CapHandle);
    fn nvim_nv_vreplace_impl(cap: CapHandle);
}

/// Command handler for "r" single-character replace.
///
/// Replaces character(s) under the cursor with the typed character.
/// In Visual mode, delegates to the operator system.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_replace(cap: CapHandle) {
    nvim_nv_replace_impl(cap);
}

/// Command handler for "R" and "gR" replace mode.
///
/// "R" enters replace mode, "gR" enters virtual replace mode.
/// In Visual mode, acts as line-wise change operation.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_Replace(cap: CapHandle) {
    nvim_nv_Replace_impl(cap);
}

/// Command handler for "gr" virtual replace.
///
/// Replaces a single character visually (handles virtual columns).
/// In Visual mode, delegates to `nv_replace`.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_vreplace(cap: CapHandle) {
    nvim_nv_vreplace_impl(cap);
}

// =============================================================================
// Phase 7: Scroll and screen handlers
// =============================================================================

extern "C" {
    fn nvim_nv_zet_impl(cap: CapHandle);
    fn nvim_nv_scroll_impl(cap: CapHandle);
    fn nvim_nv_right_impl(cap: CapHandle);
    fn nvim_nv_left_impl(cap: CapHandle);
    fn nvim_nv_up_impl(cap: CapHandle);
    fn nvim_nv_down_impl(cap: CapHandle);
}

/// Command handler for "z" commands.
///
/// Handles various z-prefix commands for scrolling, folding, and window management:
/// - zt, zz, zb: scroll line to top/center/bottom
/// - zo, zc, za: fold open/close/toggle
/// - zf, zd: create/delete fold
/// - And many more...
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_zet(cap: CapHandle) {
    nvim_nv_zet_impl(cap);
}

/// Command handler for 'H', 'L' and 'M' scrolling commands.
///
/// - H: Move cursor to top of window
/// - M: Move cursor to middle of window
/// - L: Move cursor to bottom of window
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_scroll(cap: CapHandle) {
    nvim_nv_scroll_impl(cap);
}

/// Command handler for cursor right commands.
///
/// Handles 'l', space, and right arrow key movement.
/// With Shift/Ctrl modifiers, moves by word instead.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_right(cap: CapHandle) {
    nvim_nv_right_impl(cap);
}

/// Command handler for cursor left commands.
///
/// Handles 'h', backspace, and left arrow key movement.
/// With Shift/Ctrl modifiers, moves by word instead.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_left(cap: CapHandle) {
    nvim_nv_left_impl(cap);
}

/// Command handler for cursor up commands.
///
/// Handles 'k', '-', and up arrow key movement.
/// With Shift modifier, acts as page up.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_up(cap: CapHandle) {
    nvim_nv_up_impl(cap);
}

/// Command handler for cursor down commands.
///
/// Handles 'j', '+', CR, and down arrow key movement.
/// With Shift modifier, acts as page down.
/// In quickfix window, CR views the result.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_down(cap: CapHandle) {
    nvim_nv_down_impl(cap);
}

// =============================================================================
// Phase 8: Miscellaneous handlers
// =============================================================================

extern "C" {
    fn nvim_nv_g_cmd_impl(cap: CapHandle);
    fn nvim_nv_at_impl(cap: CapHandle);
    fn nvim_nv_join_impl(cap: CapHandle);
    fn nvim_nv_open_impl(cap: CapHandle);
}

/// Command handler for "g" prefix commands.
///
/// Handles a large number of g-prefixed commands:
/// - g0, g^, g$, gm, gM: screen column movement
/// - gj, gk: display line movement
/// - ge, gE: backward end of word
/// - gg: go to line (default first)
/// - gd, gD: go to definition
/// - gf, gF: go to file under cursor
/// - gi: go to Insert position
/// - And many more...
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_g_cmd(cap: CapHandle) {
    nvim_nv_g_cmd_impl(cap);
}

/// Command handler for "@" macro execution command.
///
/// Executes the contents of a register as normal mode commands.
/// - @{a-z}: execute register a-z
/// - @@: execute the last used register
/// - @:: repeat last command-line
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_at(cap: CapHandle) {
    nvim_nv_at_impl(cap);
}

/// Command handler for "J" and "gJ" join commands.
///
/// - J: Join lines with space, adjusting whitespace
/// - gJ: Join lines without adjusting whitespace
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_join(cap: CapHandle) {
    nvim_nv_join_impl(cap);
}

/// Command handler for "o" and "O" open line commands.
///
/// - o: Open a new line below the cursor and enter Insert mode
/// - O: Open a new line above the cursor and enter Insert mode
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_open(cap: CapHandle) {
    nvim_nv_open_impl(cap);
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
