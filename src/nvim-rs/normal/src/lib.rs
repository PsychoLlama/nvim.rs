//! Normal mode key processing and command handling for Neovim
//!
//! This crate provides Rust implementations of normal mode functions
//! from `src/nvim/normal.c`. It handles normal and visual mode command
//! processing.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]

pub mod additional_char;
pub mod check;
pub mod commands;
pub mod count;
pub mod dispatch;
pub mod execute;
pub mod finish_command;
pub mod insert;
pub mod motion;
pub mod normal_execute;
pub mod operator;
pub mod operator_cmds;
pub mod pending;
pub mod showcmd;
pub mod visual;

use std::ffi::{c_char, c_int, c_uint};

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

// Delete key codes (from keycodes.h)
const KE_KDEL: c_int = 80;
const K_DEL: c_int = termcap2key(b'k' as c_int, b'D' as c_int);
const K_KDEL: c_int = termcap2key(KS_EXTRA, KE_KDEL);

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
    fn rs_foldOpenCursor();
    fn nvim_set_ins_at_eol(val: bool);
    fn nvim_get_curswant() -> c_int;
    fn nvim_set_curswant(val: c_int);
    fn nvim_virtual_active() -> bool;
    fn nvim_gchar_cursor() -> c_int;
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

    // Wave 2 Phase 1: Visual state accessors
    fn nvim_redraw_curbuf_inverted();
    fn nvim_set_VIsual_reselect(val: bool);
    fn nvim_get_VIsual_mode_orig() -> c_int;
    fn nvim_set_VIsual_mode_orig(val: c_int);
    #[allow(dead_code)]
    fn nvim_get_curbuf_visual_vi_mode() -> c_int;
    fn nvim_set_curbuf_visual_vi_mode(val: c_int);
    fn nvim_get_mode_displayed() -> bool;
    fn nvim_set_clear_cmdline(val: bool);
    fn rs_clear_showcmd();

    // Wave 2 Phase 2: Redo/count/handler accessors
    fn nvim_oap_get_regname_ptr(oap: OapHandle) -> c_int;
    fn nvim_cap_get_nchar_len(cap: CapHandle) -> c_int;
    fn nvim_cap_append_nchar_composing_to_redobuff(cap: CapHandle);
    fn nvim_set_vcount_call(count: i64, count1: i64, set_prevcount: bool);
    fn rs_do_tag(
        tag: *mut std::ffi::c_char,
        typ: c_int,
        count: c_int,
        forceit: c_int,
        verbose: bool,
    );
    fn nvim_do_execreg_recorded() -> bool;
    fn nvim_normal_get_got_int() -> bool;
    fn nvim_normal_line_breakcheck();

    // Wave 2 Phase 3: Visual operator accessors
    fn nvim_set_VIsual_mode(val: c_int);
    fn nvim_oap_get_motion_force(oap: OapHandle) -> c_int;

    // Wave 2 Phase 4: Selection/g-cmd accessors
    fn nvim_get_cursor_line_byte_at_col(col: c_int) -> c_int;
    fn nvim_cursor_line_col_is_white(col: c_int) -> bool;
    fn nvim_stuff_empty() -> bool;
    fn nvim_typebuf_typed() -> bool;
    fn nvim_vim_strchr_p_slm(c: c_int) -> bool;
    fn nvim_n_start_visual_mode(c: c_int);
    fn nvim_set_cursor_from_last_insert() -> bool;
    fn nvim_check_cursor_lnum_call();
    fn nvim_get_cursor_line_len() -> c_int;
    fn nvim_get_cursor_coladd() -> c_int;
    fn nvim_normal_get_cmdwin_type() -> c_int;
    fn nvim_set_cmdwin_result(val: c_int);
    fn nvim_get_restart_edit() -> c_int;
    fn nvim_set_restart_edit(val: c_int);

    // Wave 2 Phase 5: Visual complex function accessors
    fn nvim_set_VIsual_active(val: bool);
    fn nvim_check_cursor();
    fn nvim_setmouse();
    fn nvim_get_VIsual_lnum() -> c_int;
    fn nvim_get_VIsual_col() -> c_int;
    fn nvim_get_VIsual_coladd() -> c_int;
    fn nvim_set_VIsual_pos(lnum: c_int, col: c_int, coladd: c_int);
    fn nvim_set_cursor_pos(lnum: c_int, col: c_int, coladd: c_int);
    fn nvim_get_b_visual_vi_start_lnum() -> c_int;
    fn nvim_get_b_visual_vi_start_col() -> c_int;
    fn nvim_get_b_visual_vi_start_coladd() -> c_int;
    fn nvim_set_b_visual_vi_start(lnum: c_int, col: c_int, coladd: c_int);
    fn nvim_get_b_visual_vi_end_lnum() -> c_int;
    fn nvim_get_b_visual_vi_end_col() -> c_int;
    fn nvim_get_b_visual_vi_end_coladd() -> c_int;
    fn nvim_set_b_visual_vi_end(lnum: c_int, col: c_int, coladd: c_int);
    fn nvim_get_b_visual_vi_curswant() -> c_int;
    fn nvim_set_b_visual_vi_curswant(val: c_int);
    fn nvim_set_curbuf_visual_mode_eval(val: c_int);
    fn nvim_set_VIsual_select_reg(val: c_int);
    fn nvim_update_topline_call();
    fn nvim_p_sel_is_exclusive() -> bool;
    fn nvim_equalpos_VIsual_cursor() -> bool;
    fn nvim_set_w_set_curswant(val: bool);
    fn nvim_getvcols_call(
        lnum1: c_int,
        col1: c_int,
        coladd1: c_int,
        lnum2: c_int,
        col2: c_int,
        coladd2: c_int,
        out_left: *mut c_int,
        out_right: *mut c_int,
    );
    fn nvim_coladvance_call(col: c_int);
    fn nvim_findmatch_nul(
        oap: OapHandle,
        out_lnum: *mut c_int,
        out_col: *mut c_int,
        out_coladd: *mut c_int,
    ) -> bool;
    fn nvim_unadjust_for_sel_inner_cursor() -> bool;
    fn nvim_unadjust_for_sel_inner_visual() -> bool;
    #[allow(dead_code)]
    fn nvim_ml_get_len_call(lnum: c_int) -> c_int;

    // Phase 1A: find_ident_at_pos accessors
    fn nvim_ml_get_buf_wrapper(buf: BufHandle, lnum: i32) -> *mut c_char;
    fn nvim_mb_get_class_wrapper(ptr: *const c_char) -> c_int;
    fn nvim_utfc_ptr2len_wrapper(ptr: *const c_char) -> c_int;
    fn nvim_utf_head_off_wrapper(base: *const c_char, ptr: *const c_char) -> c_int;
    fn nvim_win_get_w_buffer(wp: WinHandle) -> BufHandle;
    fn nvim_emsg_no_string_under_cursor();
    fn nvim_emsg_no_ident_under_cursor();
}

// Tag command type (must match tag_defs.h)
const DT_POP: c_int = 2;

// Operator type constants (must match ops.h)
const OP_NOP: c_int = 0;
const OP_DELETE: c_int = 1;
const OP_YANK: c_int = 2;
const OP_LSHIFT: c_int = 4;
const OP_RSHIFT: c_int = 5;

// NUL constant for motion_force
const NUL_CHAR: c_int = 0;

// Command retval constants (from normal_defs.h)
const CA_COMMAND_BUSY: c_int = 1;

// Ctrl key constants (must match ascii_defs.h)
const CTRL_C: c_int = 3;
const CTRL_G: c_int = 7;
const CTRL_N: c_int = 14;
const CTRL_V: c_int = 22;

// Motion type constants (must match normal_defs.h)
const K_MT_CHARWISE: c_int = 0;
const K_MT_LINEWISE: c_int = 1;

// Beginline flags (must match cursor_defs.h)
const BL_WHITE: c_int = 1;

// Fold open flag (must match option_vars.generated.h)
const K_OPT_FDO_FLAG_PERCENT: c_uint = 0x10;

/// Opaque handle to a window (win_T*).
pub type WinHandle = *mut std::ffi::c_void;

/// Opaque handle to a buffer (buf_T*).
pub type BufHandle = *mut std::ffi::c_void;

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
// find_ident_at_pos constants (from normal.h, verified with _Static_assert)
// =============================================================================

const FIND_IDENT: c_int = 1;
const FIND_STRING: c_int = 2;
const FIND_EVAL: c_int = 4;

// =============================================================================
// find_ident_at_pos (Phase 1A)
// =============================================================================

/// Find identifier or text at position in a window.
///
/// This is the Rust implementation of `find_ident_at_pos()` from normal.c.
/// Searches for an identifier or text at the given position, respecting
/// FIND_IDENT, FIND_STRING, and FIND_EVAL flags.
///
/// # Safety
/// `wp` must be a valid window pointer. `text` must be a valid `char**`.
/// `textcol` may be NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_find_ident_at_pos(
    wp: WinHandle,
    lnum: i32,
    startcol: i32,
    text: *mut *mut c_char,
    textcol: *mut c_int,
    find_type: c_int,
) -> usize {
    let buf = nvim_win_get_w_buffer(wp);
    let ptr = nvim_ml_get_buf_wrapper(buf, lnum);
    let nul: c_char = 0;
    #[allow(clippy::cast_possible_wrap)]
    let rbracket: c_char = b']' as c_char;

    let mut col: c_int = 0;
    let mut this_class: c_int = 0;
    let mut bn: c_int;
    let mut i: c_int = c_int::from(find_type & FIND_IDENT == 0);

    // if i == 0: try to find an identifier
    // if i == 1: try to find any non-white text
    while i < 2 {
        // 1. skip to start of identifier/text
        col = startcol;
        while *ptr.offset(col as isize) != nul {
            // Stop at a ']' to evaluate "a[x]".
            if (find_type & FIND_EVAL != 0) && *ptr.offset(col as isize) == rbracket {
                break;
            }
            this_class = nvim_mb_get_class_wrapper(ptr.offset(col as isize));
            if this_class != 0 && (i == 1 || this_class != 1) {
                break;
            }
            col += nvim_utfc_ptr2len_wrapper(ptr.offset(col as isize));
        }

        // When starting on a ']' count it, so that we include the '['.
        bn = c_int::from(*ptr.offset(col as isize) == rbracket);

        // 2. Back up to start of identifier/text.
        // Remember class of character under cursor.
        if (find_type & FIND_EVAL != 0) && *ptr.offset(col as isize) == rbracket {
            // Use class of 'a' (identifier class)
            this_class = nvim_mb_get_class_wrapper(c"a".as_ptr());
        } else {
            this_class = nvim_mb_get_class_wrapper(ptr.offset(col as isize));
        }
        while col > 0 && this_class != 0 {
            let mut prevcol =
                col - 1 - nvim_utf_head_off_wrapper(ptr, ptr.offset(col as isize - 1));
            let prev_class = nvim_mb_get_class_wrapper(ptr.offset(prevcol as isize));
            if this_class != prev_class
                && (i == 0 || prev_class == 0 || (find_type & FIND_IDENT != 0))
                && (find_type & FIND_EVAL == 0
                    || prevcol == 0
                    || !rs_find_is_eval_item(
                        ptr.offset(prevcol as isize),
                        std::ptr::addr_of_mut!(prevcol),
                        std::ptr::addr_of_mut!(bn),
                        BACKWARD,
                    ))
            {
                break;
            }
            col = prevcol;
        }

        // If we don't want just any old text, or we've found an identifier, stop.
        if this_class > 2 {
            this_class = 2;
        }
        if (find_type & FIND_STRING == 0) || this_class == 2 {
            break;
        }
        i += 1;
    }

    if *ptr.offset(col as isize) == nul || (i == 0 && this_class != 2) {
        // Didn't find an identifier or text.
        if find_type & FIND_STRING != 0 {
            nvim_emsg_no_string_under_cursor();
        } else {
            nvim_emsg_no_ident_under_cursor();
        }
        return 0;
    }
    let result_ptr = ptr.offset(col as isize);
    *text = result_ptr;
    if !textcol.is_null() {
        *textcol = col;
    }

    // 3. Find the end of the identifier/text.
    bn = 0;
    let startcol_remaining = startcol - col;
    let mut end_col: c_int = 0;
    // Search for point of changing multibyte character class.
    this_class = nvim_mb_get_class_wrapper(result_ptr);
    while *result_ptr.offset(end_col as isize) != nul
        && ((if i == 0 {
            nvim_mb_get_class_wrapper(result_ptr.offset(end_col as isize)) == this_class
        } else {
            nvim_mb_get_class_wrapper(result_ptr.offset(end_col as isize)) != 0
        }) || ((find_type & FIND_EVAL != 0)
            && end_col <= startcol_remaining
            && rs_find_is_eval_item(
                result_ptr.offset(end_col as isize),
                std::ptr::addr_of_mut!(end_col),
                std::ptr::addr_of_mut!(bn),
                FORWARD,
            )))
    {
        end_col += nvim_utfc_ptr2len_wrapper(result_ptr.offset(end_col as isize));
    }

    debug_assert!(end_col >= 0);
    #[allow(clippy::cast_sign_loss)]
    let result = end_col as usize;
    result
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

// Fold option flags (kOptFdoFlag*) - from build/src/nvim/auto/option_vars.generated.h
const K_OPT_FDO_FLAG_HOR: c_uint = 0x04;
const K_OPT_FDO_FLAG_JUMP: c_uint = 0x400;

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
        rs_foldOpenCursor();
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
        rs_foldOpenCursor();
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
        rs_foldOpenCursor();
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
    #[link_name = "rs_unadjust_for_sel"]
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
    fn rs_do_window(nchar: c_int, count: c_int, xchar: c_int);
    fn nvim_nv_colon(cap: CapHandle);
}

/// Opaque handle to fmark_T*.
pub type FmarkHandle = *mut std::ffi::c_void;

// Operator type constant for OP_CHANGE - from src/nvim/ops.h
const OP_CHANGE: c_int = 3;

// Fold option flag for block - from build/src/nvim/auto/option_vars.generated.h
const K_OPT_FDO_FLAG_BLOCK: c_uint = 0x02;

// Virtual edit flag for onemore - from build/src/nvim/auto/option_vars.generated.h
const K_OPT_VE_FLAG_ONEMORE: c_uint = 0x08;

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
        rs_foldOpenCursor();
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
            rs_foldOpenCursor();
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
        rs_foldOpenCursor();
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
        rs_foldOpenCursor();
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

// Jop flag for view - from build/src/nvim/auto/option_vars.generated.h
const K_OPT_JOP_FLAG_VIEW: c_uint = 0x02;

// Fold flag for mark - from build/src/nvim/auto/option_vars.generated.h
const K_OPT_FDO_FLAG_MARK: c_uint = 0x08;

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
        rs_foldOpenCursor();
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
        rs_foldOpenCursor();
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
        rs_do_window(nchar, count0, c_int::from(NUL));
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
        rs_foldOpenCursor();
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

    // nv_brackets_impl C accessors
    fn nvim_nv_bracket_block_call(cap: CapHandle);
    fn nvim_bracket_find_ident(cap: CapHandle);
    fn nvim_bracket_findpar(cap: CapHandle, flag: c_int) -> bool;
    fn nvim_bracket_mark_jump(cap: CapHandle);
    fn nvim_bracket_do_mouse(cap: CapHandle);
    fn nvim_bracket_fold_move(cap: CapHandle);
    fn nvim_bracket_diff_move(cap: CapHandle);
    fn nvim_bracket_spell_move(cap: CapHandle);
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
#[allow(clippy::cast_lossless)]
pub unsafe extern "C" fn rs_nv_brackets(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    nvim_oap_set_motion_type(oap, K_MT_CHARWISE);
    nvim_oap_set_inclusive(oap, false);
    nvim_set_cursor_coladd_zero();

    let nchar = nvim_cap_get_nchar(cap);
    let cmdchar = nvim_cap_get_cmdchar(cap);

    if nchar == b'f' as c_int {
        // "[f" or "]f": Edit file under cursor (same as "gf")
        nvim_nv_gotofile(cap);
    } else if nvim_vim_strchr_str(c"iI\x09dD\x04".as_ptr(), nchar) {
        // Find the occurrence(s) of the identifier or define under cursor
        nvim_bracket_find_ident(cap);
    } else if (cmdchar == b'[' as c_int && nvim_vim_strchr_str(c"{(*/#mM".as_ptr(), nchar))
        || (cmdchar == b']' as c_int && nvim_vim_strchr_str(c"})*/#mM".as_ptr(), nchar))
    {
        // "[{", "[(", "]}" or "])": bracket/method matching
        nvim_nv_bracket_block_call(cap);
    } else if nchar == b'[' as c_int || nchar == b']' as c_int {
        // "[[", "[]", "]]" and "][": move to start or end of function
        let flag = if nchar == cmdchar {
            b'{' as c_int
        } else {
            b'}' as c_int
        };
        if !nvim_bracket_findpar(cap, flag) {
            rs_clearopbeep(oap);
        }
    } else if nchar == b'p' as c_int || nchar == b'P' as c_int {
        // "[p", "[P", "]P" and "]p": put with indent adjustment
        nvim_nv_put_opt(cap, true);
    } else if nchar == b'\'' as c_int || nchar == b'`' as c_int {
        // "['", "[`", "]'" and "]`": jump to next mark
        nvim_bracket_mark_jump(cap);
    } else if (K_RIGHTRELEASE..=K_LEFTMOUSE).contains(&nchar) {
        // Mouse click: put selected text with indent adjustment
        nvim_bracket_do_mouse(cap);
    } else if nchar == b'z' as c_int {
        // "[z" and "]z": move to start or end of open fold
        nvim_bracket_fold_move(cap);
    } else if nchar == b'c' as c_int {
        // "[c" and "]c": move to next or previous diff-change
        nvim_bracket_diff_move(cap);
    } else if nchar == b'r' as c_int || nchar == b's' as c_int || nchar == b'S' as c_int {
        // "[r", "[s", "[S", "]r", "]s" and "]S": move to next spell error
        nvim_bracket_spell_move(cap);
    } else {
        // Not a valid cap->nchar
        rs_clearopbeep(oap);
    }
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
    fn nvim_nv_scroll_impl(cap: CapHandle);
    fn nvim_nv_right_impl(cap: CapHandle);
    fn nvim_nv_left_impl(cap: CapHandle);
    fn nvim_nv_up_impl(cap: CapHandle);
    fn nvim_nv_down_impl(cap: CapHandle);

    // z-command C accessors
    fn nvim_get_curwin_w_p_fdl() -> c_int;
    fn nvim_set_curwin_w_p_fdl(val: c_int);
    fn nvim_get_curwin_w_p_fen() -> bool;
    fn nvim_set_curwin_w_p_fen(val: bool);
    fn nvim_set_curwin_w_foldinvalid(val: bool);
    fn nvim_get_curwin_w_view_width() -> c_int;
    fn nvim_get_curwin_w_leftcol() -> c_int;
    fn nvim_set_curwin_w_leftcol(val: c_int);
    fn nvim_validate_botline_curwin();
    fn nvim_get_curwin_w_botline() -> c_int;
    fn nvim_check_cursor_col_call();
    fn nvim_scroll_cursor_top(off: c_int, always: bool);
    fn nvim_scroll_cursor_bot(off: c_int, always: bool);
    fn nvim_scroll_cursor_halfway(atend: bool, prefer_above: bool);
    fn nvim_redraw_later_curwin(redraw_type: c_int);
    fn nvim_set_leftcol_call(col: c_int);
    fn nvim_hasFolding_curwin(lnum: c_int) -> bool;
    fn nvim_getvcol_curwin_cursor(vcol: *mut c_int);
    fn nvim_getvcol_curwin_cursor_end(vcol: *mut c_int);
    fn nvim_win_col_off_curwin() -> c_int;
    fn nvim_changed_window_setting_curwin();
    fn nvim_spell_suggest_call(count: c_int);
    fn nvim_get_curwin_w_p_wrap() -> bool;
    fn nvim_nv_z_get_count(cap: CapHandle, nchar_arg: *mut c_int) -> bool;
    fn nvim_nv_zg_zw(cap: CapHandle, nchar: c_int) -> c_int;
    fn nvim_sync_fen_in_diff_windows();
    fn nvim_vim_strchr_str(s: *const c_char, c: c_int) -> bool;
    fn nvim_ascii_isdigit(c: c_int) -> bool;
    fn nvim_get_curwin() -> WinHandle;

    // Fold functions from fold crate
    fn rs_foldManualAllowed(create: bool) -> c_int;
    fn rs_deleteFold(wp: WinHandle, first: c_int, last: c_int, recursive: bool, had_visual: bool);
    fn rs_foldmethodIsManual(wp: WinHandle) -> c_int;
    fn rs_clearFolding(wp: WinHandle);
    fn rs_foldmethodIsMarker(wp: WinHandle) -> c_int;
    fn rs_foldmethodIsDiff(wp: WinHandle) -> c_int;
    fn rs_newFoldLevel();
    fn rs_setFoldRepeat(lnum: c_int, count: c_int, do_open: bool);
    fn rs_setManualFold(lnum: c_int, do_open: bool, recursive: bool, donep: *mut bool);
    fn rs_getDeepestNesting(wp: WinHandle) -> c_int;
    fn rs_foldMoveTo(updown: bool, dir: c_int, count: c_int) -> c_int;
    fn rs_set_fraction(wp: WinHandle);
    fn rs_get_sidescrolloff_value(wp: WinHandle) -> c_int;
    fn nvim_curwin_get_p_scb() -> bool;
    fn nvim_get_e352_msg() -> *const c_char;
    fn nvim_set_finish_op(val: bool);
}

// z-command constants
const NL_CHAR: c_int = 0o12; // '\012' = newline
const CAR_CHAR: c_int = 0o15; // '\015' = carriage return
const K_KENTER: c_int = termcap2key(b'K' as c_int, b'A' as c_int);
const OP_FOLD: c_int = 19;
const UPD_VALID: c_int = 10;
const UPD_NOT_VALID: c_int = 40;

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
    nv_zet_impl(cap);
}

/// Implementation of the 'z' command dispatch.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[allow(
    clippy::cast_lossless,
    clippy::useless_let_if_seq,
    clippy::borrow_as_ptr,
    clippy::too_many_lines,
    clippy::manual_c_str_literals
)]
unsafe fn nv_zet_impl(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    let mut nchar = nvim_cap_get_nchar(cap);
    let old_fdl = nvim_get_curwin_w_p_fdl();
    let old_fen = nvim_get_curwin_w_p_fen();
    let curwin = nvim_get_curwin();

    let siso = rs_get_sidescrolloff_value(curwin);

    if nvim_ascii_isdigit(nchar) && !nvim_nv_z_get_count(cap, &mut nchar) {
        return;
    }

    // "zf" and "zF" are always an operator, "zd", "zo", "zO", "zc"
    // and "zC" only in Visual mode.  "zj" and "zk" are motion commands.
    let cap_nchar = nvim_cap_get_nchar(cap);
    if cap_nchar != b'f' as c_int
        && cap_nchar != b'F' as c_int
        && !(nvim_get_VIsual_active() != 0 && nvim_vim_strchr_str(c"dcCoO".as_ptr(), cap_nchar))
        && cap_nchar != b'j' as c_int
        && cap_nchar != b'k' as c_int
        && rs_checkclearop(oap)
    {
        return;
    }

    // For "z+", "z<CR>", "zt", "z.", "zz", "z^", "z-", "zb":
    // If line number given, set cursor.
    if nvim_vim_strchr_str(c"+\r\nt.z^-b".as_ptr(), nchar)
        && nvim_cap_get_count0(cap) != 0
        && nvim_cap_get_count0(cap) != nvim_get_cursor_lnum()
    {
        nvim_setpcmark();
        let line_count = nvim_get_line_count();
        if nvim_cap_get_count0(cap) > line_count {
            nvim_set_cursor_lnum(line_count);
        } else {
            nvim_set_cursor_lnum(nvim_cap_get_count0(cap));
        }
        nvim_check_cursor_col_call();
    }

    match nchar {
        // "z+", "z<CR>" and "zt": put cursor at top of screen
        n if n == b'+' as c_int => {
            if nvim_cap_get_count0(cap) == 0 {
                // No count given: put cursor at the line below screen
                nvim_validate_botline_curwin();
                let botline = nvim_get_curwin_w_botline();
                let line_count = nvim_get_line_count();
                let lnum = if botline < line_count {
                    botline
                } else {
                    line_count
                };
                nvim_set_cursor_lnum(lnum);
            }
            // FALLTHROUGH to NL/CAR/K_KENTER -> 't'
            nvim_beginline(BL_WHITE | BL_FIX);
            nvim_scroll_cursor_top(0, true);
            nvim_redraw_later_curwin(UPD_VALID);
            rs_set_fraction(curwin);
        }
        n if n == NL_CHAR || n == CAR_CHAR || n == K_KENTER => {
            // FALLTHROUGH to 't'
            nvim_beginline(BL_WHITE | BL_FIX);
            nvim_scroll_cursor_top(0, true);
            nvim_redraw_later_curwin(UPD_VALID);
            rs_set_fraction(curwin);
        }
        n if n == b't' as c_int => {
            nvim_scroll_cursor_top(0, true);
            nvim_redraw_later_curwin(UPD_VALID);
            rs_set_fraction(curwin);
        }

        // "z." and "zz": put cursor in middle of screen
        n if n == b'.' as c_int => {
            nvim_beginline(BL_WHITE | BL_FIX);
            // FALLTHROUGH to 'z'
            nvim_scroll_cursor_halfway(true, false);
            nvim_redraw_later_curwin(UPD_VALID);
            rs_set_fraction(curwin);
        }
        n if n == b'z' as c_int => {
            nvim_scroll_cursor_halfway(true, false);
            nvim_redraw_later_curwin(UPD_VALID);
            rs_set_fraction(curwin);
        }

        // "z^", "z-" and "zb": put cursor at bottom of screen
        n if n == b'^' as c_int => {
            if nvim_cap_get_count0(cap) != 0 {
                nvim_scroll_cursor_bot(0, true);
                nvim_set_cursor_lnum(nvim_win_get_topline(curwin));
            } else if nvim_win_get_topline(curwin) == 1 {
                nvim_set_cursor_lnum(1);
            } else {
                nvim_set_cursor_lnum(nvim_win_get_topline(curwin) - 1);
            }
            // FALLTHROUGH to '-' -> 'b'
            nvim_beginline(BL_WHITE | BL_FIX);
            nvim_scroll_cursor_bot(0, true);
            nvim_redraw_later_curwin(UPD_VALID);
            rs_set_fraction(curwin);
        }
        n if n == b'-' as c_int => {
            nvim_beginline(BL_WHITE | BL_FIX);
            // FALLTHROUGH to 'b'
            nvim_scroll_cursor_bot(0, true);
            nvim_redraw_later_curwin(UPD_VALID);
            rs_set_fraction(curwin);
        }
        n if n == b'b' as c_int => {
            nvim_scroll_cursor_bot(0, true);
            nvim_redraw_later_curwin(UPD_VALID);
            rs_set_fraction(curwin);
        }

        // "zH" - scroll screen right half-page
        n if n == b'H' as c_int => {
            let half = nvim_get_curwin_w_view_width() / 2;
            nvim_cap_set_count1(cap, nvim_cap_get_count1(cap) * half);
            // FALLTHROUGH to 'h'
            if !nvim_get_curwin_w_p_wrap() {
                let leftcol = nvim_get_curwin_w_leftcol();
                let count1 = nvim_cap_get_count1(cap);
                if count1 > leftcol {
                    nvim_set_leftcol_call(0);
                } else {
                    nvim_set_leftcol_call(leftcol - count1);
                }
            }
        }

        // "zh" - scroll screen to the right
        n if n == b'h' as c_int || n == K_LEFT => {
            if !nvim_get_curwin_w_p_wrap() {
                let leftcol = nvim_get_curwin_w_leftcol();
                let count1 = nvim_cap_get_count1(cap);
                if count1 > leftcol {
                    nvim_set_leftcol_call(0);
                } else {
                    nvim_set_leftcol_call(leftcol - count1);
                }
            }
        }

        // "zL" - scroll window left half-page
        n if n == b'L' as c_int => {
            let half = nvim_get_curwin_w_view_width() / 2;
            nvim_cap_set_count1(cap, nvim_cap_get_count1(cap) * half);
            // FALLTHROUGH to 'l'
            if !nvim_get_curwin_w_p_wrap() {
                nvim_set_leftcol_call(nvim_get_curwin_w_leftcol() + nvim_cap_get_count1(cap));
            }
        }

        // "zl" - scroll window to the left if not wrapping
        n if n == b'l' as c_int || n == K_RIGHT => {
            if !nvim_get_curwin_w_p_wrap() {
                nvim_set_leftcol_call(nvim_get_curwin_w_leftcol() + nvim_cap_get_count1(cap));
            }
        }

        // "zs" - scroll screen, cursor at the start
        n if n == b's' as c_int => {
            if !nvim_get_curwin_w_p_wrap() {
                let mut col: c_int = 0;
                if nvim_hasFolding_curwin(nvim_get_cursor_lnum()) {
                    col = 0;
                } else {
                    nvim_getvcol_curwin_cursor(&mut col);
                }
                if col > siso {
                    col -= siso;
                } else {
                    col = 0;
                }
                if nvim_get_curwin_w_leftcol() != col {
                    nvim_set_curwin_w_leftcol(col);
                    nvim_redraw_later_curwin(UPD_NOT_VALID);
                }
            }
        }

        // "ze" - scroll screen, cursor at the end
        n if n == b'e' as c_int => {
            if !nvim_get_curwin_w_p_wrap() {
                let mut col: c_int = 0;
                if nvim_hasFolding_curwin(nvim_get_cursor_lnum()) {
                    col = 0;
                } else {
                    nvim_getvcol_curwin_cursor_end(&mut col);
                }
                let n_val = nvim_get_curwin_w_view_width() - nvim_win_col_off_curwin();
                if col + siso < n_val {
                    col = 0;
                } else {
                    col = col + siso - n_val + 1;
                }
                if nvim_get_curwin_w_leftcol() != col {
                    nvim_set_curwin_w_leftcol(col);
                    nvim_redraw_later_curwin(UPD_NOT_VALID);
                }
            }
        }

        // "zp", "zP" in block mode put without adding trailing spaces
        n if n == b'P' as c_int || n == b'p' as c_int => {
            rs_nv_put(cap);
        }

        // "zy" Yank without trailing spaces
        n if n == b'y' as c_int => {
            rs_nv_operator(cap);
        }

        // "zF": create fold command
        // "zf": create fold operator
        n if n == b'F' as c_int || n == b'f' as c_int => {
            if rs_foldManualAllowed(true) != 0 {
                nvim_cap_set_nchar(cap, b'f' as c_int);
                rs_nv_operator(cap);
                nvim_set_curwin_w_p_fen(true);

                // "zF" is like "zfzf"
                if nchar == b'F' as c_int && nvim_oap_get_op_type_ptr(oap) == OP_FOLD {
                    rs_nv_operator(cap);
                    nvim_set_finish_op(true);
                }
            } else {
                rs_clearopbeep(oap);
            }
        }

        // "zd": delete fold at cursor
        // "zD": delete fold at cursor recursively
        n if n == b'd' as c_int || n == b'D' as c_int => {
            if rs_foldManualAllowed(false) != 0 {
                if nvim_get_VIsual_active() != 0 {
                    rs_nv_operator(cap);
                } else {
                    let cursor_lnum = nvim_get_cursor_lnum();
                    rs_deleteFold(
                        curwin,
                        cursor_lnum,
                        cursor_lnum,
                        nchar == b'D' as c_int,
                        false,
                    );
                }
            }
        }

        // "zE": erase all folds
        n if n == b'E' as c_int => {
            if rs_foldmethodIsManual(curwin) != 0 {
                rs_clearFolding(curwin);
                nvim_changed_window_setting_curwin();
            } else if rs_foldmethodIsMarker(curwin) != 0 {
                let line_count = nvim_get_line_count();
                rs_deleteFold(curwin, 1, line_count, true, false);
            } else {
                nvim_emsg(nvim_get_e352_msg());
            }
        }

        // "zn": fold none: reset 'foldenable'
        n if n == b'n' as c_int => {
            nvim_set_curwin_w_p_fen(false);
        }

        // "zN": fold Normal: set 'foldenable'
        n if n == b'N' as c_int => {
            nvim_set_curwin_w_p_fen(true);
        }

        // "zi": invert folding: toggle 'foldenable'
        n if n == b'i' as c_int => {
            nvim_set_curwin_w_p_fen(!nvim_get_curwin_w_p_fen());
        }

        // "za": open closed fold or close open fold at cursor
        n if n == b'a' as c_int => {
            let cursor_lnum = nvim_get_cursor_lnum();
            if nvim_hasFolding_curwin(cursor_lnum) {
                rs_setFoldRepeat(cursor_lnum, nvim_cap_get_count1(cap), true);
            } else {
                rs_setFoldRepeat(cursor_lnum, nvim_cap_get_count1(cap), false);
                nvim_set_curwin_w_p_fen(true);
            }
        }

        // "zA": open fold at cursor recursively
        n if n == b'A' as c_int => {
            let cursor_lnum = nvim_get_cursor_lnum();
            if nvim_hasFolding_curwin(cursor_lnum) {
                rs_setManualFold(cursor_lnum, true, true, std::ptr::null_mut());
            } else {
                rs_setManualFold(cursor_lnum, false, true, std::ptr::null_mut());
                nvim_set_curwin_w_p_fen(true);
            }
        }

        // "zo": open fold at cursor or Visual area
        n if n == b'o' as c_int => {
            if nvim_get_VIsual_active() != 0 {
                rs_nv_operator(cap);
            } else {
                rs_setFoldRepeat(nvim_get_cursor_lnum(), nvim_cap_get_count1(cap), true);
            }
        }

        // "zO": open fold recursively
        n if n == b'O' as c_int => {
            if nvim_get_VIsual_active() != 0 {
                rs_nv_operator(cap);
            } else {
                rs_setManualFold(nvim_get_cursor_lnum(), true, true, std::ptr::null_mut());
            }
        }

        // "zc": close fold at cursor or Visual area
        n if n == b'c' as c_int => {
            if nvim_get_VIsual_active() != 0 {
                rs_nv_operator(cap);
            } else {
                rs_setFoldRepeat(nvim_get_cursor_lnum(), nvim_cap_get_count1(cap), false);
            }
            nvim_set_curwin_w_p_fen(true);
        }

        // "zC": close fold recursively
        n if n == b'C' as c_int => {
            if nvim_get_VIsual_active() != 0 {
                rs_nv_operator(cap);
            } else {
                rs_setManualFold(nvim_get_cursor_lnum(), false, true, std::ptr::null_mut());
            }
            nvim_set_curwin_w_p_fen(true);
        }

        // "zv": open folds at the cursor
        n if n == b'v' as c_int => {
            rs_foldOpenCursor();
        }

        // "zx": re-apply 'foldlevel' and open folds at the cursor
        n if n == b'x' as c_int => {
            nvim_set_curwin_w_p_fen(true);
            nvim_set_curwin_w_foldinvalid(true);
            rs_newFoldLevel();
            rs_foldOpenCursor();
        }

        // "zX": undo manual opens/closes, re-apply 'foldlevel'
        n if n == b'X' as c_int => {
            nvim_set_curwin_w_p_fen(true);
            nvim_set_curwin_w_foldinvalid(true);
            // old_fdl = -1 to force update; we call rs_newFoldLevel directly
            rs_newFoldLevel();
        }

        // "zm": fold more
        n if n == b'm' as c_int => {
            let mut fdl = nvim_get_curwin_w_p_fdl();
            if fdl > 0 {
                fdl -= nvim_cap_get_count1(cap);
                if fdl < 0 {
                    fdl = 0;
                }
                nvim_set_curwin_w_p_fdl(fdl);
            }
            // Force update
            rs_newFoldLevel();
            nvim_set_curwin_w_p_fen(true);
        }

        // "zM": close all folds
        n if n == b'M' as c_int => {
            nvim_set_curwin_w_p_fdl(0);
            // Force update
            rs_newFoldLevel();
            nvim_set_curwin_w_p_fen(true);
        }

        // "zr": reduce folding
        n if n == b'r' as c_int => {
            let mut fdl = nvim_get_curwin_w_p_fdl();
            fdl += nvim_cap_get_count1(cap);
            let d = rs_getDeepestNesting(curwin);
            if fdl > d {
                fdl = d;
            }
            nvim_set_curwin_w_p_fdl(fdl);
        }

        // "zR": open all folds
        n if n == b'R' as c_int => {
            nvim_set_curwin_w_p_fdl(rs_getDeepestNesting(curwin));
            // Force update
            rs_newFoldLevel();
        }

        // "zj" move to next fold downwards
        // "zk" move to next fold upwards
        n if n == b'j' as c_int || n == b'k' as c_int => {
            let dir = if nchar == b'j' as c_int {
                FORWARD
            } else {
                BACKWARD
            };
            if rs_foldMoveTo(true, dir, nvim_cap_get_count1(cap)) == 0 {
                rs_clearopbeep(oap);
            }
        }

        // "zug" and "zuw": undo "zg" and "zw"
        // "zg": add good word to word list
        // "zw": add wrong word to word list
        // "zG": add good word to temp word list
        // "zW": add wrong word to temp word list
        n if n == b'u' as c_int
            || n == b'g' as c_int
            || n == b'w' as c_int
            || n == b'G' as c_int
            || n == b'W' as c_int =>
        {
            if nvim_nv_zg_zw(cap, nchar) == FAIL {
                return;
            }
        }

        // "z=": suggestions for a badly spelled word
        n if n == b'=' as c_int => {
            if !rs_checkclearop(oap) {
                nvim_spell_suggest_call(nvim_cap_get_count0(cap));
            }
        }

        _ => {
            rs_clearopbeep(oap);
        }
    }

    // Redraw when 'foldenable' changed
    if old_fen != nvim_get_curwin_w_p_fen() {
        if rs_foldmethodIsDiff(curwin) != 0 && nvim_curwin_get_p_scb() {
            // Adjust 'foldenable' in diff-synced windows.
            nvim_sync_fen_in_diff_windows();
        }
        nvim_changed_window_setting_curwin();
    }

    // Redraw when 'foldlevel' changed.
    if old_fdl != nvim_get_curwin_w_p_fdl() {
        rs_newFoldLevel();
    }
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
    fn nvim_nv_at_impl(cap: CapHandle);
    fn nvim_nv_join_impl(cap: CapHandle);
    fn nvim_nv_open_impl(cap: CapHandle);

    // g-command C accessors
    fn nvim_nv_addsub(cap: CapHandle);
    fn nvim_current_search(count: c_int, forward: bool) -> bool;
    fn nvim_cursor_up(count: c_int, upd_topline: bool) -> c_int;
    fn nvim_cursor_down_call(count: c_int, upd_topline: bool) -> c_int;
    fn nvim_linetabsize_curwin(lnum: c_int) -> c_int;
    fn nvim_coladvance_curwin(col: c_int);
    fn nvim_cursor_pos_info_call();
    fn nvim_invoke_edit_g(cap: CapHandle);
    fn nvim_nv_gotofile(cap: CapHandle);
    fn nvim_set_mod_mask_ctrl();
    fn nvim_do_mouse_g(oap: OapHandle, nchar: c_int, count1: c_int);
    fn nvim_goto_byte_call(count: c_int);
    fn nvim_undo_time_call(count: c_int, sec: bool, file: bool, absolute: bool);
    fn nvim_show_sb_text_call();
    fn nvim_show_utf8_call();
    fn nvim_utf_find_illegal_call();
    fn nvim_set_oap_cursor_start(oap: OapHandle);
    fn nvim_set_curwin_w_set_curswant(val: bool);
    fn nvim_nv_g_home_m_cmd_call(cap: CapHandle);
    fn nvim_nv_g_dollar_cmd(cap: CapHandle);
    fn nvim_nv_gd_impl(oap: OapHandle, nchar: c_int, thisblock: c_int);
    fn nvim_do_sleep_wrapper(ms: c_int, allow_int: bool);
    fn nvim_do_exmode_wrapper();
    fn rs_do_ascii(eap: *mut std::ffi::c_void);
}

// =============================================================================
// nv_screengo: screen-based movement for gj/gk
// =============================================================================

extern "C" {
    fn nvim_get_curwin_w_virtcol() -> c_int;
    fn nvim_set_curwin_w_curswant_int(val: c_int);
    fn nvim_get_curwin_ml_line_count() -> c_int;
    fn nvim_win_col_off2_curwin() -> c_int;
    fn nvim_validate_virtcol_curwin();
    fn nvim_cursor_up_inner_curwin(n: c_int, skip_conceal: bool);
    fn nvim_cursor_down_inner_curwin(n: c_int, skip_conceal: bool);
    fn nvim_oneright_call() -> c_int;
    fn nvim_get_cursor_char() -> c_int;
    fn nvim_vim_isprintc(c: c_int) -> bool;
    fn nvim_vim_strsize_call(s: *const c_char) -> c_int;
    fn nvim_adjust_skipcol_call();
    fn nvim_dec_cursor_col();
    fn rs_get_showbreak_value(wp: WinHandle) -> *const c_char;
    fn nvim_get_curwin_w_curswant() -> c_int;
}

/// Screen-based cursor movement for gj/gk commands.
///
/// Moves the cursor up or down by screen lines (not file lines),
/// handling wrapped lines, column offsets, and multi-byte characters.
///
/// # Safety
/// `oap` must be a valid oparg_T pointer.
#[no_mangle]
#[allow(clippy::too_many_lines, clippy::cast_lossless)]
pub unsafe extern "C" fn rs_nv_screengo(
    oap: OapHandle,
    dir: c_int,
    dist: c_int,
    skip_conceal: bool,
) -> bool {
    let mut linelen = nvim_linetabsize_curwin(nvim_get_cursor_lnum());
    let mut retval = true;
    let mut atend = false;

    nvim_oap_set_motion_type(oap, K_MT_CHARWISE);
    nvim_oap_set_inclusive(oap, nvim_get_curwin_w_curswant() == MAXCOL);

    let col_off1 = nvim_win_col_off_curwin();
    let col_off2 = col_off1 - nvim_win_col_off2_curwin();
    let width1 = nvim_get_curwin_w_view_width() - col_off1;
    let mut width2 = nvim_get_curwin_w_view_width() - col_off2;

    if width2 == 0 {
        width2 = 1; // Avoid divide by zero.
    }

    if nvim_get_curwin_w_view_width() != 0 {
        // Instead of sticking at the last character of the buffer line we
        // try to stick in the last column of the screen.
        if nvim_get_curwin_w_curswant() == MAXCOL {
            atend = true;
            nvim_validate_virtcol_curwin();
            if width1 <= 0 {
                nvim_set_curwin_w_curswant_int(0);
            } else {
                nvim_set_curwin_w_curswant_int(width1 - 1);
                if nvim_get_curwin_w_virtcol() > nvim_get_curwin_w_curswant() {
                    let extra = ((nvim_get_curwin_w_virtcol() - nvim_get_curwin_w_curswant() - 1)
                        / width2
                        + 1)
                        * width2;
                    nvim_set_curwin_w_curswant_int(nvim_get_curwin_w_curswant() + extra);
                }
            }
        } else {
            let n = if linelen > width1 {
                ((linelen - width1 - 1) / width2 + 1) * width2 + width1
            } else {
                width1
            };
            if nvim_get_curwin_w_curswant() >= n {
                nvim_set_curwin_w_curswant_int(n - 1);
            }
        }

        let mut remaining = dist;
        while remaining > 0 {
            remaining -= 1;
            if dir == BACKWARD {
                if nvim_get_curwin_w_curswant() >= width1
                    && !nvim_hasFolding_curwin(nvim_get_cursor_lnum())
                {
                    // Move back within the line. This can give a negative value
                    // for w_curswant if width1 < width2 (with cpoptions+=n),
                    // which will get clipped to column 0.
                    nvim_set_curwin_w_curswant_int(nvim_get_curwin_w_curswant() - width2);
                } else {
                    // to previous line
                    if nvim_get_cursor_lnum() <= 1 {
                        retval = false;
                        break;
                    }
                    nvim_cursor_up_inner_curwin(1, skip_conceal);

                    linelen = nvim_linetabsize_curwin(nvim_get_cursor_lnum());
                    if linelen > width1 {
                        let w = (((linelen - width1 - 1) / width2) + 1) * width2;
                        nvim_set_curwin_w_curswant_int(nvim_get_curwin_w_curswant() + w);
                    }
                }
            } else {
                // dir == FORWARD
                let n = if linelen > width1 {
                    ((linelen - width1 - 1) / width2 + 1) * width2 + width1
                } else {
                    width1
                };
                if nvim_get_curwin_w_curswant() + width2 < n
                    && !nvim_hasFolding_curwin(nvim_get_cursor_lnum())
                {
                    // move forward within line
                    nvim_set_curwin_w_curswant_int(nvim_get_curwin_w_curswant() + width2);
                } else {
                    // to next line
                    if nvim_get_cursor_lnum() >= nvim_get_curwin_ml_line_count() {
                        retval = false;
                        break;
                    }
                    nvim_cursor_down_inner_curwin(1, skip_conceal);
                    let remainder = nvim_get_curwin_w_curswant() % width2;
                    nvim_set_curwin_w_curswant_int(remainder);

                    // Check if the cursor has moved below the number display
                    // when width1 < width2 (with cpoptions+=n). Subtract width2
                    // to get a negative value for w_curswant, which will get
                    // clipped to column 0.
                    if nvim_get_curwin_w_curswant() >= width1 {
                        nvim_set_curwin_w_curswant_int(nvim_get_curwin_w_curswant() - width2);
                    }
                    linelen = nvim_linetabsize_curwin(nvim_get_cursor_lnum());
                }
            }
        }
    }

    if nvim_virtual_active() && atend {
        nvim_coladvance_curwin(MAXCOL);
    } else {
        nvim_coladvance_curwin(nvim_get_curwin_w_curswant());
    }

    if nvim_get_cursor_col() > 0 && nvim_get_curwin_w_p_wrap() {
        // Check for landing on a character that got split at the end of the
        // last line. We want to advance a screenline, not end up in the same
        // screenline or move two screenlines.
        nvim_validate_virtcol_curwin();
        let mut virtcol = nvim_get_curwin_w_virtcol();
        let sbr = rs_get_showbreak_value(nvim_get_curwin());
        if virtcol > width1 && !sbr.is_null() && *sbr != 0 {
            virtcol -= nvim_vim_strsize_call(sbr);
        }

        let c = nvim_get_cursor_char();
        if dir == FORWARD
            && virtcol < nvim_get_curwin_w_curswant()
            && nvim_get_curwin_w_curswant() <= width1
            && !nvim_vim_isprintc(c)
            && c > 255
        {
            nvim_oneright_call();
        }

        if virtcol > nvim_get_curwin_w_curswant()
            && (if nvim_get_curwin_w_curswant() < width1 {
                nvim_get_curwin_w_curswant() > width1 / 2
            } else {
                (nvim_get_curwin_w_curswant() - width1) % width2 > width2 / 2
            })
        {
            nvim_dec_cursor_col();
        }
    }

    if atend {
        nvim_set_curwin_w_curswant_int(MAXCOL); // stick in the last column
    }
    nvim_adjust_skipcol_call();

    retval
}

// g-command key constants
const CTRL_A: c_int = 1;
const CTRL_X: c_int = 24;
const CTRL_H_KEY: c_int = 8;
const CTRL_G_KEY: c_int = 7;
const CTRL_RSB: c_int = 29;
const K_BS: c_int = termcap2key(b'k' as c_int, b'b' as c_int);
const K_KHOME: c_int = termcap2key(b'K' as c_int, b'1' as c_int);
const K_KEND: c_int = termcap2key(b'K' as c_int, b'4' as c_int);
const KE_IGNORE: c_int = 53;
const K_IGNORE: c_int = termcap2key(KS_EXTRA, KE_IGNORE);
const KE_LEFTMOUSE: c_int = 44;
const KE_LEFTDRAG: c_int = 45;
const KE_LEFTRELEASE: c_int = 46;
const KE_MIDDLEMOUSE: c_int = 47;
const KE_MIDDLEDRAG: c_int = 48;
const KE_MIDDLERELEASE: c_int = 49;
const KE_RIGHTMOUSE: c_int = 50;
const KE_RIGHTDRAG: c_int = 51;
const KE_RIGHTRELEASE: c_int = 52;
const KE_X1MOUSE: c_int = 89;
const KE_X1DRAG: c_int = 90;
const KE_X1RELEASE: c_int = 91;
const KE_X2MOUSE: c_int = 92;
const KE_X2DRAG: c_int = 93;
const KE_X2RELEASE: c_int = 94;
const KE_MOUSEMOVE: c_int = 100;
const K_MIDDLEMOUSE: c_int = termcap2key(KS_EXTRA, KE_MIDDLEMOUSE);
const K_MIDDLEDRAG: c_int = termcap2key(KS_EXTRA, KE_MIDDLEDRAG);
const K_MIDDLERELEASE: c_int = termcap2key(KS_EXTRA, KE_MIDDLERELEASE);
const K_LEFTMOUSE: c_int = termcap2key(KS_EXTRA, KE_LEFTMOUSE);
const K_LEFTDRAG: c_int = termcap2key(KS_EXTRA, KE_LEFTDRAG);
const K_LEFTRELEASE: c_int = termcap2key(KS_EXTRA, KE_LEFTRELEASE);
const K_MOUSEMOVE: c_int = termcap2key(KS_EXTRA, KE_MOUSEMOVE);
const K_RIGHTMOUSE: c_int = termcap2key(KS_EXTRA, KE_RIGHTMOUSE);
const K_RIGHTDRAG: c_int = termcap2key(KS_EXTRA, KE_RIGHTDRAG);
const K_RIGHTRELEASE: c_int = termcap2key(KS_EXTRA, KE_RIGHTRELEASE);
const K_X1MOUSE: c_int = termcap2key(KS_EXTRA, KE_X1MOUSE);
const K_X1DRAG: c_int = termcap2key(KS_EXTRA, KE_X1DRAG);
const K_X1RELEASE: c_int = termcap2key(KS_EXTRA, KE_X1RELEASE);
const K_X2MOUSE: c_int = termcap2key(KS_EXTRA, KE_X2MOUSE);
const K_X2DRAG: c_int = termcap2key(KS_EXTRA, KE_X2DRAG);
const K_X2RELEASE: c_int = termcap2key(KS_EXTRA, KE_X2RELEASE);
const NUL_VAL: c_int = 0;
const POUND: c_int = 0xA3;

/// Command handler for "g" prefix commands.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_g_cmd(cap: CapHandle) {
    nv_g_cmd_impl(cap);
}

/// Implementation of the 'g' command dispatch.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[allow(
    clippy::cast_lossless,
    clippy::too_many_lines,
    clippy::manual_c_str_literals
)]
unsafe fn nv_g_cmd_impl(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    let nchar = nvim_cap_get_nchar(cap);

    match nchar {
        // "g^A/g^X": Sequentially increment visually selected region.
        n if n == CTRL_A || n == CTRL_X => {
            if nvim_get_VIsual_active() != 0 {
                nvim_cap_set_arg(cap, 1); // cap->arg = true
                nvim_cap_set_cmdchar(cap, nchar);
                nvim_cap_set_nchar(cap, NUL_VAL);
                nvim_nv_addsub(cap);
            } else {
                rs_clearopbeep(oap);
            }
        }

        // "gR": Enter virtual replace mode.
        n if n == b'R' as c_int => {
            nvim_cap_set_arg(cap, 1);
            rs_nv_Replace(cap);
        }

        n if n == b'r' as c_int => {
            rs_nv_vreplace(cap);
        }

        n if n == b'&' as c_int => {
            do_cmdline_cmd(c"%s//~/&".as_ptr());
        }

        // "gv": Reselect the previous Visual area.
        n if n == b'v' as c_int => {
            rs_nv_gv_cmd(cap);
        }

        // "gV": Don't reselect the previous Visual area.
        n if n == b'V' as c_int => {
            nvim_set_VIsual_reselect(false);
        }

        // "gh", "gH", "g^H": start Select mode.
        n if n == K_BS => {
            nvim_cap_set_nchar(cap, CTRL_H_KEY);
            // FALLTHROUGH
            nvim_cap_set_cmdchar(
                cap,
                nvim_cap_get_nchar(cap) + (b'v' as c_int - b'h' as c_int),
            );
            nvim_cap_set_arg(cap, 1);
            rs_nv_visual(cap);
        }
        n if n == b'h' as c_int || n == b'H' as c_int || n == CTRL_H_KEY => {
            nvim_cap_set_cmdchar(cap, nchar + (b'v' as c_int - b'h' as c_int));
            nvim_cap_set_arg(cap, 1);
            rs_nv_visual(cap);
        }

        // "gn", "gN" visually select next/previous search match
        n if n == b'N' as c_int || n == b'n' as c_int => {
            if !nvim_current_search(nvim_cap_get_count1(cap), nchar == b'n' as c_int) {
                rs_clearopbeep(oap);
            }
        }

        // "gj" and "gk": screen-line movement
        n if n == b'j' as c_int || n == K_DOWN => {
            let ok = if nvim_get_curwin_w_p_wrap() {
                rs_nv_screengo(oap, FORWARD, nvim_cap_get_count1(cap), false)
            } else {
                nvim_oap_set_motion_type(oap, K_MT_LINEWISE);
                nvim_cursor_down_call(
                    nvim_cap_get_count1(cap),
                    nvim_oap_get_op_type_ptr(oap) == OP_NOP,
                ) != 0
            };
            if !ok {
                rs_clearopbeep(oap);
            }
        }

        n if n == b'k' as c_int || n == K_UP => {
            let ok = if nvim_get_curwin_w_p_wrap() {
                rs_nv_screengo(oap, BACKWARD, nvim_cap_get_count1(cap), false)
            } else {
                nvim_oap_set_motion_type(oap, K_MT_LINEWISE);
                nvim_cursor_up(
                    nvim_cap_get_count1(cap),
                    nvim_oap_get_op_type_ptr(oap) == OP_NOP,
                ) != 0
            };
            if !ok {
                rs_clearopbeep(oap);
            }
        }

        // "gJ": join two lines without inserting a space.
        n if n == b'J' as c_int => {
            rs_nv_join(cap);
        }

        // "g0", "g^", "gm": screen column movement
        n if n == b'^' as c_int
            || n == b'0' as c_int
            || n == b'm' as c_int
            || n == K_HOME
            || n == K_KHOME =>
        {
            nvim_nv_g_home_m_cmd_call(cap);
        }

        // "gM": middle of text in the line
        n if n == b'M' as c_int => {
            nvim_oap_set_motion_type(oap, K_MT_CHARWISE);
            nvim_oap_set_inclusive(oap, false);
            let i = nvim_linetabsize_curwin(nvim_get_cursor_lnum());
            let count0 = nvim_cap_get_count0(cap);
            if count0 > 0 && count0 <= 100 {
                nvim_coladvance_curwin(i * count0 / 100);
            } else {
                nvim_coladvance_curwin(i / 2);
            }
            nvim_set_curwin_w_set_curswant(true);
        }

        // "g_": to the last non-blank character
        n if n == b'_' as c_int => {
            rs_nv_g_underscore_cmd(cap);
        }

        // "g$": like "$" but for screen lines
        n if n == b'$' as c_int || n == K_END || n == K_KEND => {
            nvim_nv_g_dollar_cmd(cap);
        }

        // "g*", "g#", CTRL-], g]
        n if n == b'*' as c_int
            || n == b'#' as c_int
            || n == POUND
            || n == CTRL_RSB
            || n == b']' as c_int =>
        {
            rs_nv_ident(cap);
        }

        // ge and gE: go back to end of word
        n if n == b'e' as c_int || n == b'E' as c_int => {
            nvim_oap_set_motion_type(oap, K_MT_CHARWISE);
            nvim_set_curwin_w_set_curswant(true);
            nvim_oap_set_inclusive(oap, true);
            if nvim_bckend_word(nvim_cap_get_count1(cap), nchar == b'E' as c_int, false) == 0 {
                rs_clearopbeep(oap);
            }
        }

        // "g CTRL-G": display info about cursor position
        n if n == CTRL_G_KEY => {
            nvim_cursor_pos_info_call();
        }

        // "gi": start Insert at the last position.
        n if n == b'i' as c_int => {
            rs_nv_gi_cmd(cap);
        }

        // "gI": Start insert in column 1.
        n if n == b'I' as c_int => {
            nvim_beginline(0);
            if !rs_checkclearopq(oap) {
                nvim_invoke_edit_g(cap);
            }
        }

        // "gf": goto file, edit file under cursor
        n if n == b'f' as c_int || n == b'F' as c_int => {
            nvim_nv_gotofile(cap);
        }

        // "g'm" and "g`m": jump to mark without setting pcmark
        n if n == b'\'' as c_int => {
            nvim_cap_set_arg(cap, 1);
            // FALLTHROUGH
            rs_nv_gomark(cap);
        }
        n if n == b'`' as c_int => {
            rs_nv_gomark(cap);
        }

        // "gs": Goto sleep.
        n if n == b's' as c_int => {
            nvim_do_sleep_wrapper(nvim_cap_get_count1(cap) * 1000, false);
        }

        // "ga": Display the ascii value of the character under the cursor.
        n if n == b'a' as c_int => {
            rs_do_ascii(std::ptr::null_mut());
        }

        // "g8": Display UTF-8 bytes or find illegal byte sequence.
        n if n == b'8' as c_int => {
            if nvim_cap_get_count0(cap) == 8 {
                nvim_utf_find_illegal_call();
            } else {
                nvim_show_utf8_call();
            }
        }

        // "g<": show scrollback text
        n if n == b'<' as c_int => {
            nvim_show_sb_text_call();
        }

        // "gg": Goto first line or line number.
        n if n == b'g' as c_int => {
            nvim_cap_set_arg(cap, 0);
            rs_nv_goto(cap);
        }

        // "gq", "gw": Format text
        // "g~", "gu", "gU", "g?", "g@": operators
        n if n == b'q' as c_int || n == b'w' as c_int => {
            nvim_set_oap_cursor_start(oap);
            // FALLTHROUGH
            rs_nv_operator(cap);
        }
        n if n == b'~' as c_int
            || n == b'u' as c_int
            || n == b'U' as c_int
            || n == b'?' as c_int
            || n == b'@' as c_int =>
        {
            rs_nv_operator(cap);
        }

        // "gd", "gD": Find definition
        n if n == b'd' as c_int || n == b'D' as c_int => {
            nvim_nv_gd_impl(oap, nchar, nvim_cap_get_count0(cap));
        }

        // g<*Mouse>: <C-*mouse>
        n if n == K_MIDDLEMOUSE
            || n == K_MIDDLEDRAG
            || n == K_MIDDLERELEASE
            || n == K_LEFTMOUSE
            || n == K_LEFTDRAG
            || n == K_LEFTRELEASE
            || n == K_MOUSEMOVE
            || n == K_RIGHTMOUSE
            || n == K_RIGHTDRAG
            || n == K_RIGHTRELEASE
            || n == K_X1MOUSE
            || n == K_X1DRAG
            || n == K_X1RELEASE
            || n == K_X2MOUSE
            || n == K_X2DRAG
            || n == K_X2RELEASE =>
        {
            nvim_set_mod_mask_ctrl();
            nvim_do_mouse_g(oap, nchar, nvim_cap_get_count1(cap));
        }

        n if n == K_IGNORE => {}

        // "gP", "gp": same as "P" and "p" but leave cursor just after new text
        n if n == b'p' as c_int || n == b'P' as c_int => {
            rs_nv_put(cap);
        }

        // "go": goto byte count from start of buffer
        n if n == b'o' as c_int => {
            nvim_oap_set_inclusive(oap, false);
            nvim_goto_byte_call(nvim_cap_get_count0(cap));
        }

        // "gQ": improved Ex mode
        n if n == b'Q' as c_int => {
            if !rs_check_text_locked(oap) && !rs_checkclearopq(oap) {
                nvim_do_exmode_wrapper();
            }
        }

        n if n == b',' as c_int => {
            rs_nv_pcmark(cap);
        }

        n if n == b';' as c_int => {
            nvim_cap_set_count1(cap, -nvim_cap_get_count1(cap));
            rs_nv_pcmark(cap);
        }

        n if n == b't' as c_int => {
            if !rs_checkclearop(oap) {
                nvim_goto_tabpage(nvim_cap_get_count0(cap));
            }
        }

        n if n == b'T' as c_int => {
            if !rs_checkclearop(oap) {
                nvim_goto_tabpage(-nvim_cap_get_count1(cap));
            }
        }

        n if n == nvim_get_TAB() => {
            if !rs_checkclearop(oap) && !nvim_goto_tabpage_lastused() {
                rs_clearopbeep(oap);
            }
        }

        // "g+" and "g-": undo or redo along the timeline
        n if n == b'+' as c_int || n == b'-' as c_int => {
            if !rs_checkclearopq(oap) {
                let count = if nchar == b'-' as c_int {
                    -nvim_cap_get_count1(cap)
                } else {
                    nvim_cap_get_count1(cap)
                };
                nvim_undo_time_call(count, false, false, false);
            }
        }

        _ => {
            rs_clearopbeep(oap);
        }
    }
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
// Wave 2 Phase 1: Visual State Helpers
// =============================================================================

/// Reset VIsual_active and VIsual_reselect.
///
/// Ends visual mode and redraws if visual was active, then unconditionally
/// clears VIsual_reselect.
#[no_mangle]
pub extern "C" fn rs_reset_VIsual_and_resel() {
    unsafe {
        if nvim_get_VIsual_active() != 0 {
            end_visual_mode();
            nvim_redraw_curbuf_inverted();
        }
        nvim_set_VIsual_reselect(false);
    }
}

/// Reset VIsual_active and VIsual_reselect if visual was active.
///
/// Only clears VIsual_reselect when visual mode was active (unlike
/// `rs_reset_VIsual_and_resel` which always clears it).
#[no_mangle]
pub extern "C" fn rs_reset_VIsual() {
    unsafe {
        if nvim_get_VIsual_active() != 0 {
            end_visual_mode();
            nvim_redraw_curbuf_inverted();
            nvim_set_VIsual_reselect(false);
        }
    }
}

/// Restore VIsual_mode_orig to curbuf's visual mode.
///
/// If VIsual_mode_orig is set (non-NUL), copies it to curbuf->b_visual.vi_mode
/// and resets VIsual_mode_orig to NUL.
#[no_mangle]
pub extern "C" fn rs_restore_visual_mode() {
    unsafe {
        let orig = nvim_get_VIsual_mode_orig();
        if orig != NUL_CHAR {
            nvim_set_curbuf_visual_vi_mode(orig);
            nvim_set_VIsual_mode_orig(NUL_CHAR);
        }
    }
}

/// Clear the command line or update the displayed command.
///
/// If mode is currently displayed, sets `clear_cmdline` to clear it later.
/// Otherwise calls `clear_showcmd()` to update the displayed command.
#[no_mangle]
pub extern "C" fn rs_may_clear_cmdline() {
    unsafe {
        if nvim_get_mode_displayed() {
            nvim_set_clear_cmdline(true);
        } else {
            rs_clear_showcmd();
        }
    }
}

// =============================================================================
// Wave 2 Phase 2: Redo/Count Helpers + Simple nv_* Handlers
// =============================================================================

/// Prepare for redo of a command with nchar.
///
/// Calls `prep_redo` with the cap's register, count, and cmdchar, then appends
/// either the nchar_composing string or the single nchar to the redo buffer.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_prep_redo_cmd(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    let regname = nvim_oap_get_regname_ptr(oap);
    let count0 = nvim_cap_get_count0(cap);
    let cmdchar = nvim_cap_get_cmdchar(cap);

    rs_prep_redo(
        regname, count0, NUL_CHAR, cmdchar, NUL_CHAR, NUL_CHAR, NUL_CHAR,
    );

    if nvim_cap_get_nchar_len(cap) > 0 {
        nvim_cap_append_nchar_composing_to_redobuff(cap);
    } else {
        let nchar = nvim_cap_get_nchar(cap);
        AppendCharToRedobuff(nchar);
    }
}

/// Set v:count and v:count1 from cmdarg_T counts.
///
/// Multiplies count0 with opcount (same way as normal_execute), then calls
/// `set_vcount()`. Clears `*set_prevcount` after the first call.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer. `set_prevcount` must be a valid bool pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_set_vcount_ca(cap: CapHandle, set_prevcount: *mut bool) {
    let count0 = i64::from(nvim_cap_get_count0(cap));
    let opcount = i64::from(nvim_cap_get_opcount(cap));

    let count = if opcount != 0 {
        opcount * (if count0 == 0 { 1 } else { count0 })
    } else {
        count0
    };
    let count1 = if count == 0 { 1 } else { count };

    nvim_set_vcount_call(count, count1, *set_prevcount);
    *set_prevcount = false;
}

/// CTRL-T: jump to older tag.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_tagpop(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    if !rs_checkclearopq(oap) {
        rs_do_tag(
            c"".as_ptr().cast_mut(),
            DT_POP,
            nvim_cap_get_count1(cap),
            0,
            true,
        );
    }
}

/// Q: replay last recorded register.
///
/// Loops `count1` times executing `do_execreg(reg_recorded)`, with
/// `line_breakcheck()` between iterations. Stops on failure or interrupt.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_regreplay(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    if rs_checkclearop(oap) {
        return;
    }

    let mut count1 = nvim_cap_get_count1(cap);
    while count1 > 0 && !nvim_normal_get_got_int() {
        count1 -= 1;
        if !nvim_do_execreg_recorded() {
            rs_clearopbeep(oap);
            break;
        }
        nvim_normal_line_breakcheck();
    }
}

/// CTRL-H / BS: in Select mode behaves like 'x', else like left.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_ctrlh(cap: CapHandle) {
    if nvim_get_VIsual_active() != 0 && nvim_get_VIsual_select() {
        nvim_cap_set_cmdchar(cap, c_int::from(b'x'));
        rs_v_visop(cap);
    } else {
        rs_nv_left(cap);
    }
}

// =============================================================================
// Wave 2 Phase 3: Visual Operator Helpers
// =============================================================================

/// Translate visual commands and call nv_operator.
///
/// For uppercase commands (Y, D, C, X): switches to linewise mode unless in
/// block mode. For C/D in block mode, sets curswant to MAXCOL. Translates
/// the command character via the mapping: Y→y, D→d, C→c, x→d, X→d, A→A, I→I, r→r.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_v_visop(cap: CapHandle) {
    let cmdchar = nvim_cap_get_cmdchar(cap);

    // Uppercase means linewise, except in block mode (isupper equivalent)
    if cmdchar >= c_int::from(b'A') && cmdchar <= c_int::from(b'Z') {
        if nvim_get_VIsual_mode() != CTRL_V {
            nvim_set_VIsual_mode_orig(nvim_get_VIsual_mode());
            nvim_set_VIsual_mode(c_int::from(b'V'));
        } else if cmdchar == c_int::from(b'C') || cmdchar == c_int::from(b'D') {
            nvim_set_curswant(MAXCOL);
        }
    }

    // Translate the command character
    let translated = match cmdchar {
        c if c == c_int::from(b'Y') => c_int::from(b'y'),
        c if c == c_int::from(b'D') => c_int::from(b'd'),
        c if c == c_int::from(b'C') => c_int::from(b'c'),
        c if c == c_int::from(b'x') => c_int::from(b'd'),
        c if c == c_int::from(b'X') => c_int::from(b'd'),
        c if c == c_int::from(b'A') => c_int::from(b'A'),
        c if c == c_int::from(b'I') => c_int::from(b'I'),
        c if c == c_int::from(b'r') => c_int::from(b'r'),
        _ => cmdchar, // shouldn't happen for valid visual ops
    };
    nvim_cap_set_cmdchar(cap, translated);
    rs_nv_operator(cap);
}

/// Abbreviated commands (DEL/KDEL → 'x', then visual or optrans).
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_abbrev(cap: CapHandle) {
    let cmdchar = nvim_cap_get_cmdchar(cap);
    if cmdchar == K_DEL || cmdchar == K_KDEL {
        nvim_cap_set_cmdchar(cap, c_int::from(b'x'));
    }
    // in Visual mode these commands are operators
    if nvim_get_VIsual_active() != 0 {
        rs_v_visop(cap);
    } else {
        nvim_nv_optrans_impl(cap);
    }
}

/// '_' command: linewise motion, cursor down, then beginline.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_lineop(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    nvim_oap_set_motion_type(oap, K_MT_LINEWISE);
    let count1 = nvim_cap_get_count1(cap);
    let op_type = nvim_oap_get_op_type_ptr(oap);

    if nvim_cursor_down(count1 - 1, op_type == OP_NOP) {
        let motion_force = nvim_oap_get_motion_force(oap);
        let is_linewise_delete =
            op_type == OP_DELETE && motion_force != c_int::from(b'v') && motion_force != CTRL_V;
        if is_linewise_delete || op_type == OP_LSHIFT || op_type == OP_RSHIFT {
            nvim_beginline(BL_SOL | BL_FIX);
        } else if op_type != OP_YANK {
            // 'Y' does not move cursor
            nvim_beginline(BL_WHITE | BL_FIX);
        }
    } else {
        rs_clearopbeep(oap);
    }
}

// =============================================================================
// Wave 2 Phase 4: Selection Start + g-Command Sub-handlers
// =============================================================================

/// Set VIsual_select based on selectmode option and input context.
///
/// When "c" is 'o' (mouse), always checks selectmode. Otherwise only when
/// stuff buffer is empty and typebuf was typed.
///
/// # Safety
/// Calls into C accessors for globals.
#[no_mangle]
pub extern "C" fn rs_may_start_select(c: c_int) {
    unsafe {
        let select = (c == c_int::from(b'o') || (nvim_stuff_empty() && nvim_typebuf_typed()))
            && nvim_vim_strchr_p_slm(c);
        nvim_set_VIsual_select(select);
    }
}

/// Start selection for Shift-movement keys.
///
/// Calls may_start_select('k') then enters visual mode with 'v'.
#[no_mangle]
pub extern "C" fn rs_start_selection() {
    rs_may_start_select(c_int::from(b'k'));
    unsafe {
        nvim_n_start_visual_mode(c_int::from(b'v'));
    }
}

/// "g_": go to last non-blank in line, optionally count lines down.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_g_underscore_cmd(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    nvim_oap_set_motion_type(oap, K_MT_CHARWISE);
    nvim_oap_set_inclusive(oap, true);
    nvim_set_curswant(MAXCOL);

    let count1 = nvim_cap_get_count1(cap);
    let op_type = nvim_oap_get_op_type_ptr(oap);
    if !nvim_cursor_down(count1 - 1, op_type == OP_NOP) {
        rs_clearopbeep(oap);
        return;
    }

    let mut col = nvim_get_cursor_col();

    // In Visual mode we may end up after the line.
    if col > 0 && nvim_get_cursor_line_byte_at_col(col) == 0 {
        col -= 1;
        nvim_set_cursor_col(col);
    }

    // Decrease the cursor column until it's on a non-blank.
    while col > 0 && nvim_cursor_line_col_is_white(col) {
        col -= 1;
        nvim_set_cursor_col(col);
    }

    nvim_curwin_set_curswant(true);
    adjust_for_sel(cap);
}

/// "gi": start Insert at the last insert position.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_gi_cmd(cap: CapHandle) {
    if nvim_set_cursor_from_last_insert() {
        nvim_check_cursor_lnum_call();
        let line_len = nvim_get_cursor_line_len();
        let col = nvim_get_cursor_col();
        if col > line_len {
            if nvim_virtual_active() {
                let coladd = nvim_get_cursor_coladd();
                nvim_set_cursor_coladd(coladd + col - line_len);
            }
            nvim_set_cursor_col(line_len);
        }
    }
    nvim_cap_set_cmdchar(cap, c_int::from(b'i'));
    rs_nv_edit(cap);
}

/// CTRL-\ in Normal mode: CTRL-N/CTRL-G clear state, stop visual.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_normal(cap: CapHandle) {
    let nchar = nvim_cap_get_nchar(cap);
    let oap = nvim_cap_get_oap(cap);
    if nchar == CTRL_N || nchar == CTRL_G {
        rs_clearop(oap);
        if nvim_get_restart_edit() != 0 && nvim_get_mode_displayed() {
            nvim_set_clear_cmdline(true);
        }
        nvim_set_restart_edit(0);
        if nvim_normal_get_cmdwin_type() != 0 {
            nvim_set_cmdwin_result(CTRL_C);
        }
        if nvim_get_VIsual_active() != 0 {
            end_visual_mode();
            nvim_redraw_curbuf_inverted();
        }
    } else {
        rs_clearopbeep(oap);
    }
}

// =============================================================================
// Wave 2 Phase 5: Visual complex functions
// =============================================================================

/// `gv` command: reselect previous Visual area (or exchange with current).
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_gv_cmd(cap: CapHandle) {
    let vi_start_lnum = nvim_get_b_visual_vi_start_lnum();
    let line_count = nvim_get_line_count();

    if vi_start_lnum == 0 || vi_start_lnum > line_count || nvim_get_b_visual_vi_end_lnum() == 0 {
        beep_flush();
        return;
    }

    // tpos holds the end position
    let tpos_lnum: c_int;
    let tpos_col: c_int;
    let tpos_coladd: c_int;

    if nvim_get_VIsual_active() != 0 {
        // Swap VIsual_mode with b_visual.vi_mode
        let i = nvim_get_VIsual_mode();
        nvim_set_VIsual_mode(nvim_get_curbuf_visual_vi_mode());
        nvim_set_curbuf_visual_vi_mode(i);
        nvim_set_curbuf_visual_mode_eval(i);

        // Swap curswant with b_visual.vi_curswant
        let i = nvim_get_curswant();
        nvim_set_curswant(nvim_get_b_visual_vi_curswant());
        nvim_set_b_visual_vi_curswant(i);

        // tpos = b_visual.vi_end
        tpos_lnum = nvim_get_b_visual_vi_end_lnum();
        tpos_col = nvim_get_b_visual_vi_end_col();
        tpos_coladd = nvim_get_b_visual_vi_end_coladd();

        // b_visual.vi_end = cursor
        nvim_set_b_visual_vi_end(
            nvim_get_cursor_lnum(),
            nvim_get_cursor_col(),
            nvim_get_cursor_coladd(),
        );

        // cursor = b_visual.vi_start
        nvim_set_cursor_pos(
            nvim_get_b_visual_vi_start_lnum(),
            nvim_get_b_visual_vi_start_col(),
            nvim_get_b_visual_vi_start_coladd(),
        );

        // b_visual.vi_start = VIsual
        nvim_set_b_visual_vi_start(
            nvim_get_VIsual_lnum(),
            nvim_get_VIsual_col(),
            nvim_get_VIsual_coladd(),
        );
    } else {
        nvim_set_VIsual_mode(nvim_get_curbuf_visual_vi_mode());
        nvim_set_curswant(nvim_get_b_visual_vi_curswant());

        tpos_lnum = nvim_get_b_visual_vi_end_lnum();
        tpos_col = nvim_get_b_visual_vi_end_col();
        tpos_coladd = nvim_get_b_visual_vi_end_coladd();

        nvim_set_cursor_pos(
            nvim_get_b_visual_vi_start_lnum(),
            nvim_get_b_visual_vi_start_col(),
            nvim_get_b_visual_vi_start_coladd(),
        );
    }

    nvim_set_VIsual_active(true);
    nvim_set_VIsual_reselect(true);

    // Make sure cursor is on an existing character
    nvim_check_cursor();
    // VIsual = cursor
    nvim_set_VIsual_pos(
        nvim_get_cursor_lnum(),
        nvim_get_cursor_col(),
        nvim_get_cursor_coladd(),
    );
    // cursor = tpos
    nvim_set_cursor_pos(tpos_lnum, tpos_col, tpos_coladd);
    nvim_check_cursor();
    nvim_update_topline_call();

    // Start Select mode or may_start_select
    if nvim_cap_get_arg(cap) != 0 {
        nvim_set_VIsual_select(true);
        nvim_set_VIsual_select_reg(0);
    } else {
        rs_may_start_select(c_int::from(b'c'));
    }
    nvim_setmouse();
    nvim_redraw_curbuf_inverted();
    nvim_showmode();
}

/// `o`/`O` in Visual mode: exchange start/end corners.
///
/// # Safety
/// Called from C with valid state.
#[no_mangle]
pub unsafe extern "C" fn rs_v_swap_corners(cmdchar: c_int) {
    // Save old cursor (needed in both branches)
    let old_lnum = nvim_get_cursor_lnum();
    let old_col = nvim_get_cursor_col();
    let old_coladd = nvim_get_cursor_coladd();

    if cmdchar == c_int::from(b'O') && nvim_get_VIsual_mode() == CTRL_V {
        let mut left: c_int = 0;
        let mut right: c_int = 0;
        nvim_getvcols_call(
            old_lnum,
            old_col,
            old_coladd,
            nvim_get_VIsual_lnum(),
            nvim_get_VIsual_col(),
            nvim_get_VIsual_coladd(),
            &raw mut left,
            &raw mut right,
        );

        // Move cursor to VIsual line, advance to left column
        nvim_set_cursor_lnum(nvim_get_VIsual_lnum());
        nvim_coladvance_call(left);
        // VIsual = cursor
        nvim_set_VIsual_pos(
            nvim_get_cursor_lnum(),
            nvim_get_cursor_col(),
            nvim_get_cursor_coladd(),
        );

        // Restore cursor to old line, set curswant to right
        nvim_set_cursor_lnum(old_lnum);
        nvim_set_curswant(right);

        // 'selection' "exclusive" and cursor at right-bottom corner: move right
        if old_lnum >= nvim_get_VIsual_lnum() && nvim_p_sel_is_exclusive() {
            nvim_set_curswant(nvim_get_curswant() + 1);
        }
        nvim_coladvance_call(nvim_get_curswant());

        if nvim_get_cursor_col() == old_col
            && (!nvim_virtual_active() || nvim_get_cursor_coladd() == old_coladd)
        {
            nvim_set_cursor_lnum(nvim_get_VIsual_lnum());
            if old_lnum <= nvim_get_VIsual_lnum() && nvim_p_sel_is_exclusive() {
                right += 1;
            }
            nvim_coladvance_call(right);
            nvim_set_VIsual_pos(
                nvim_get_cursor_lnum(),
                nvim_get_cursor_col(),
                nvim_get_cursor_coladd(),
            );

            nvim_set_cursor_lnum(old_lnum);
            nvim_coladvance_call(left);
            nvim_set_curswant(left);
        }
    } else {
        // Simple swap: cursor <-> VIsual
        nvim_set_cursor_pos(
            nvim_get_VIsual_lnum(),
            nvim_get_VIsual_col(),
            nvim_get_VIsual_coladd(),
        );
        nvim_set_VIsual_pos(old_lnum, old_col, old_coladd);
        nvim_set_w_set_curswant(true);
    }
}

/// Exclude last char for 'selection' == "exclusive".
///
/// # Safety
/// Called from C with valid state.
#[no_mangle]
pub unsafe extern "C" fn rs_unadjust_for_sel() -> bool {
    if nvim_p_sel_is_exclusive() && !nvim_equalpos_VIsual_cursor() {
        if nvim_lt_VIsual_cursor() {
            return nvim_unadjust_for_sel_inner_cursor();
        }
        return nvim_unadjust_for_sel_inner_visual();
    }
    false
}

/// `%` command: goto percentage in file or find matching paren.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_percent(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    let lnum = nvim_get_cursor_lnum();

    nvim_oap_set_inclusive(oap, true);

    let count0 = nvim_cap_get_count0(cap);
    if count0 != 0 {
        // {cnt}% : goto {cnt} percentage in file
        if count0 > 100 {
            rs_clearopbeep(oap);
        } else {
            nvim_oap_set_motion_type(oap, K_MT_LINEWISE);
            nvim_setpcmark();

            let line_count = nvim_get_line_count();
            // Round up, so 'normal 100%' always jumps to the last line.
            // Beyond 21474836 lines, (ml_line_count * 100 + 99) would
            // overflow on 32-bits, so use a formula with less accuracy.
            #[allow(clippy::cast_sign_loss)]
            let target = if line_count >= 21_474_836 {
                (line_count + 99) / 100 * count0
            } else {
                (line_count * count0 + 99) / 100
            };
            let target = target.max(1).min(line_count);
            nvim_set_cursor_lnum(target);

            nvim_beginline(BL_SOL | BL_FIX);
        }
    } else {
        // "%" : go to matching paren
        nvim_oap_set_motion_type(oap, K_MT_CHARWISE);
        nvim_oap_set_use_reg_one(oap, true);

        let mut out_lnum: c_int = 0;
        let mut out_col: c_int = 0;
        let mut out_coladd: c_int = 0;
        if nvim_findmatch_nul(
            oap,
            &raw mut out_lnum,
            &raw mut out_col,
            &raw mut out_coladd,
        ) {
            nvim_setpcmark();
            nvim_set_cursor_pos(out_lnum, out_col, 0);
            nvim_set_w_set_curswant(true);
            adjust_for_sel(cap);
        } else {
            rs_clearopbeep(oap);
        }
    }

    if nvim_oap_get_op_type_ptr(oap) == OP_NOP
        && lnum != nvim_get_cursor_lnum()
        && (nvim_get_fdo_flags() & K_OPT_FDO_FLAG_PERCENT) != 0
        && nvim_get_KeyTyped()
    {
        rs_foldOpenCursor();
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
