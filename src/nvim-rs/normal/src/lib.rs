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
pub mod scrollbind;
pub mod showcmd;
pub mod types;
pub mod visual;

use nvim_ascii::{rs_ascii_isdigit, rs_ascii_iswhite, rs_ascii_iswhite_or_nul};
use std::ffi::{c_char, c_int, c_uint, c_void};

use crate::types::{CmdargT, OpargT};
use std::sync::atomic::{AtomicI32, Ordering};

extern "C" {
    static mut msg_silent: c_int;
    static mut got_int: bool;
    static mut State: c_int;
    static mut redraw_mode: c_int;
    static mut p_sta: c_int;
}

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

// Insert key codes (from keycodes.h)
const KE_KINS: c_int = 79;
const K_INS: c_int = termcap2key(b'k' as c_int, b'I' as c_int);
const K_KINS: c_int = termcap2key(KS_EXTRA, KE_KINS);

// =============================================================================
// File-static variables (migrated from C normal_shim.c)
// =============================================================================

/// Saved Visual mode (VIsual_mode_orig in C). Initial value NUL (0).
/// SAFETY: Neovim is single-threaded; Relaxed ordering is sufficient.
static VISUAL_MODE_ORIG: AtomicI32 = AtomicI32::new(0);

// =============================================================================
// C accessor functions for normal mode state
// =============================================================================

extern "C" {
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
    fn nvim_oap_get_regname_ptr(oap: OapHandle) -> c_int;

    // Global motion_force accessor
    fn nvim_set_motion_force(val: c_int);

    // Lock check functions (remain in C)
    fn text_locked() -> bool;
    fn text_locked_msg();
    fn curbuf_locked() -> bool;

    // VIsual_active global (from plines.c, returns 0 or 1)
    static mut VIsual_active: bool;

    // Beep function
    fn beep_flush();

    // Redo buffer functions (from getchar.c)
    fn ResetRedobuff();
    fn AppendCharToRedobuff(c: c_int);
    fn AppendNumberToRedobuff(n: c_int);

    // Modifier and scrolling functions
    fn nvim_get_mod_mask() -> c_int;
    fn nvim_goto_tabpage(n: c_int);
    fn pagescroll(dir: c_int, count: c_int, half: bool);
    fn nvim_get_VIsual_select() -> bool;
    fn nvim_set_VIsual_select(val: bool);
    fn nvim_may_trigger_modechanged();
    fn nvim_showmode();
    fn fileinfo(fullname: c_int, shorthelp: bool, dont_truncate: bool);
    fn scroll_redraw(down: bool, count: c_int);
    fn u_undo(count: c_int);
    fn nvim_curwin_set_curswant(val: bool);
    fn nvim_get_line_count() -> c_int;
    #[allow(dead_code)]
    fn nvim_get_cursor_lnum() -> c_int;
    fn nvim_set_cursor_lnum(lnum: c_int);
    fn nvim_setpcmark();
    fn nvim_beginline(flags: c_int);
    fn cursor_down(n: c_int, upd_topline: bool) -> bool;
    fn nvim_get_KeyTyped() -> bool;
    fn nvim_get_fdo_flags() -> c_uint;
    fn rs_foldOpenCursor();
    fn nvim_set_ins_at_eol(val: bool);
    fn nvim_get_curswant() -> c_int;
    fn nvim_set_curswant(val: c_int);
    fn virtual_active(wp: WinHandle) -> bool;
    fn utf_ptr2char(p: *const c_char) -> c_int;
    fn coladvance(wp: WinHandle, col: c_int) -> c_int;

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
    fn nvim_cap_get_cmdchar(cap: CapHandle) -> c_int;
    fn nvim_cap_get_nchar(cap: CapHandle) -> c_int;
    fn nvim_cap_get_count0(cap: CapHandle) -> c_int;
    fn nvim_cap_set_count0(cap: CapHandle, val: c_int);
    fn nvim_cap_get_count1(cap: CapHandle) -> c_int;
    fn nvim_cap_set_count1(cap: CapHandle, val: c_int);
    fn nvim_cap_get_arg(cap: CapHandle) -> c_int;

    // C functions for command handlers
    fn ex_help(eap: *mut std::ffi::c_void);
    fn do_cmdline_cmd(cmd: *const std::ffi::c_char);

    // Wave 2 Phase 1: Visual state accessors
    fn nvim_redraw_curbuf_inverted();
    fn nvim_get_VIsual_reselect() -> c_int;
    fn nvim_set_VIsual_reselect(val: bool);
    #[allow(dead_code)]
    fn nvim_get_curbuf_visual_vi_mode() -> c_int;
    fn nvim_set_curbuf_visual_vi_mode(val: c_int);
    fn nvim_get_mode_displayed() -> bool;
    fn nvim_set_clear_cmdline(val: bool);
    fn rs_clear_showcmd();

    // Wave 2 Phase 2: Redo/count/handler accessors
    fn nvim_cap_get_nchar_len(cap: CapHandle) -> c_int;
    fn AppendToRedobuff(s: *const c_char);
    fn set_vcount(count: i64, count1: i64, set_prevcount: bool);
    fn rs_do_tag(
        tag: *mut std::ffi::c_char,
        typ: c_int,
        count: c_int,
        forceit: c_int,
        verbose: bool,
    );
    fn line_breakcheck();
    fn do_execreg(regname: c_int, colon: c_int, addcr: c_int, silent: c_int) -> c_int;
    static reg_recorded: c_int;

    // Wave 2 Phase 3: Visual operator accessors
    fn nvim_set_VIsual_mode(val: c_int);
    fn nvim_oap_get_motion_force(oap: OapHandle) -> c_int;

    // Wave 2 Phase 4: Selection/g-cmd accessors
    fn nvim_get_cursor_line_byte_at_col(col: c_int) -> c_int;
    fn nvim_cursor_line_col_is_white(col: c_int) -> bool;
    fn nvim_stuff_empty() -> bool;
    fn typebuf_typed() -> bool;
    fn nvim_vim_strchr_p_slm(c: c_int) -> bool;
    fn nvim_set_cursor_from_last_insert() -> bool;
    fn nvim_check_cursor_lnum_call();
    fn nvim_get_cursor_line_len() -> c_int;
    fn nvim_get_cursor_coladd() -> c_int;
    fn nvim_normal_get_cmdwin_type() -> c_int;
    fn nvim_set_cmdwin_result(val: c_int);
    static mut restart_edit: c_int;
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
    fn nvim_p_sel_is_exclusive() -> bool;
    fn nvim_equalpos_VIsual_cursor() -> bool;
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
    fn findmatch(oap: OapHandle, initc: c_int) -> *mut crate::types::PosT;
    #[allow(dead_code)]
    fn nvim_ml_get_len_call(lnum: c_int) -> c_int;

    // Phase 2: unadjust_for_sel_inner generic pos accessors
    fn nvim_mark_mb_adjustpos_pos(lnum: c_int, col: c_int, col_out: *mut c_int) -> c_int;
    fn nvim_getvcol_pos_coladd(lnum: c_int, col: c_int, coladd: c_int) -> c_int;

    // Phase 1A: find_ident_at_pos accessors
    fn nvim_ml_get_buf_wrapper(buf: BufHandle, lnum: i32) -> *mut c_char;
    fn mb_get_class(ptr: *const c_char) -> c_int;
    fn nvim_utfc_ptr2len_wrapper(ptr: *const c_char) -> c_int;
    fn utf_head_off(base: *const c_char, ptr: *const c_char) -> c_int;
    fn nvim_win_get_w_buffer(wp: WinHandle) -> BufHandle;
    fn nvim_emsg_no_string_under_cursor();
    fn nvim_emsg_no_ident_under_cursor();

    // pos_T accessors (from memline_shim.c)
    fn nvim_pos_get_lnum(pos: *const c_void) -> c_int;
    fn nvim_pos_get_col(pos: *const c_void) -> c_int;
    fn nvim_pos_get_coladd(pos: *const c_void) -> c_int;
    fn nvim_pos_set_lnum(pos: *mut c_void, lnum: c_int);
    fn nvim_pos_set_col(pos: *mut c_void, col: c_int);
    fn nvim_pos_set_coladd(pos: *mut c_void, coladd: c_int);
}

// Tag command type (must match tag_defs.h)
const DT_POP: c_int = 2;

// Operator type constants (must match ops.h)
const OP_NOP: c_int = 0;
const OP_DELETE: c_int = 1;
const OP_YANK: c_int = 2;
const OP_LSHIFT: c_int = 4;
const OP_RSHIFT: c_int = 5;
const OP_ROT13: c_int = 15;

// Search flag constants (must match search.h)
const SEARCH_MARK: c_int = 0x200;

// Redraw type constants (must match drawscreen.h)
const UPD_SOME_VALID: c_int = 35;

// NUL constant for motion_force
const NUL_CHAR: c_int = 0;

// Command retval constants (from normal_defs.h)
const CA_COMMAND_BUSY: c_int = 1;

// Ctrl key constants (must match ascii_defs.h)
const CTRL_C: c_int = 3;
const CTRL_G: c_int = 7;
const CTRL_N: c_int = 14;
const CTRL_V: c_int = 22;
const ESC_CHAR: c_int = 0o33; // '\033' = Escape

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

    let nv_max_linear = crate::dispatch::table::rs_table_get_max_linear();
    let nv_cmds_size = crate::dispatch::table::rs_table_get_size();

    // If the character is in the first part: The character is the index into
    // nv_cmd_idx[].
    if abs_char <= nv_max_linear {
        return c_int::from(crate::dispatch::table::rs_table_get_cmd_idx(abs_char));
    }

    // Perform a binary search.
    let mut bot = nv_max_linear + 1;
    let mut top = nv_cmds_size - 1;
    let mut idx = -1;

    while bot <= top {
        let i = c_int::midpoint(bot, top);
        let c = crate::dispatch::table::rs_table_get_cmd_char(c_int::from(
            crate::dispatch::table::rs_table_get_cmd_idx(i),
        ))
        .abs();
        if abs_char == c {
            idx = c_int::from(crate::dispatch::table::rs_table_get_cmd_idx(i));
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
pub(crate) unsafe fn rs_find_is_eval_item(
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
            this_class = mb_get_class(ptr.offset(col as isize));
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
            this_class = mb_get_class(c"a".as_ptr());
        } else {
            this_class = mb_get_class(ptr.offset(col as isize));
        }
        while col > 0 && this_class != 0 {
            let mut prevcol = col - 1 - utf_head_off(ptr, ptr.offset(col as isize - 1));
            let prev_class = mb_get_class(ptr.offset(prevcol as isize));
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
    this_class = mb_get_class(result_ptr);
    while *result_ptr.offset(end_col as isize) != nul
        && ((if i == 0 {
            mb_get_class(result_ptr.offset(end_col as isize)) == this_class
        } else {
            mb_get_class(result_ptr.offset(end_col as isize)) != 0
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
    (*oap.cast::<OpargT>()).motion_force = NUL_CHAR;
    (*oap.cast::<OpargT>()).use_reg_one = false;
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
    if nvim_oap_get_op_type_ptr(oap) == OP_NOP && !VIsual_active {
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
    (*cap.cast::<CmdargT>()).retval |= CA_COMMAND_BUSY;
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
    if VIsual_active {
        rs_end_visual_mode();
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
        pagescroll(arg, count1, false);
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
        pagescroll(dir, count0, true);
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
    if VIsual_active {
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
            fileinfo(count0, false, true);
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
        scroll_redraw(arg != 0, count1);
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
    u_undo(count1);
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
    if !virtual_active(nvim_get_curwin())
        || utf_ptr2char(get_cursor_pos_ptr()) != 0
        || nvim_oap_get_op_type_ptr(oap) == OP_NOP
    {
        nvim_set_curswant(MAXCOL); // so we stay at the end
    }

    if !cursor_down(count1 - 1, nvim_oap_get_op_type_ptr(oap) == OP_NOP) {
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
        (*cap.cast::<CmdargT>()).arg = 1;
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
        coladvance(nvim_get_curwin(), count0 - 1);
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
    // nvim_curwin_set_set_curswant consolidated into nvim_curwin_set_curswant (Phase 4)
    fn nvim_fwd_word(count: c_int, bigword: bool, eol: bool) -> c_int;
    fn nvim_bck_word(count: c_int, bigword: bool, stop: bool) -> c_int;
    fn end_word(count: c_int, bigword: bool, stop: bool, empty: bool) -> c_int;
    #[allow(dead_code)]
    fn bckend_word(count: c_int, bigword: bool, eol: bool) -> c_int;
    fn findsent(dir: c_int, count: c_int) -> c_int;
    fn findpar(pincl: *mut bool, dir: c_int, count: c_int, what: c_int, both: bool) -> bool;
    fn nvim_get_cursor_col() -> c_int;
    fn nvim_set_cursor_col(col: c_int);
    fn nvim_inc_cursor() -> c_int;
    fn nvim_mb_adjust_cursor();
    fn nvim_vim_strchr_p_cpo(c: c_int) -> bool;
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
    fn searchc(cap: CapHandle, t_cmd: bool) -> c_int;
    fn nvim_getvcol_cursor(scol: *mut c_int, ecol: *mut c_int);
    fn nvim_set_cursor_coladd(val: c_int);

    // Mark command functions
    fn nvim_setmark(name: c_int) -> bool;
    fn nvim_get_jop_flags() -> c_uint;
    fn nvim_mark_get(name: c_int) -> FmarkHandle;
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
    fn set_reg_var(regname: c_int);
    // nv_put_opt lower-level C accessors (Phase 1 inlined helpers)
    fn nvim_put_get_save_fen() -> bool;
    fn nvim_get_cb_flags() -> c_int;
    fn nvim_put_copy_register(regname: c_int) -> *mut std::ffi::c_void;
    fn nvim_put_do_put(
        regname: c_int,
        savereg: *mut std::ffi::c_void,
        dir: c_int,
        count: c_int,
        flags: c_int,
    );
    fn nvim_put_free_register(savereg: *mut std::ffi::c_void);
    fn auto_format(trailblank: bool, prev_line: bool);
    // Phase 1 new lower-level accessors for put helpers
    fn nvim_get_b_prompt_start_lnum_put() -> c_int;
    fn nvim_set_cursor_col_to_prompt_text_len();
    fn nvim_set_w_p_fen(val: bool);
    fn nvim_check_vd_condition(regname: c_int) -> bool;
    fn nvim_inc_msg_silent();
    fn nvim_dec_msg_silent();
    fn nvim_curbuf_ml_empty() -> bool;
    fn nvim_get_cursor_col_vs_b_op_start_col() -> c_int;
    fn nvim_get_cursor_lnum_vs_b_op_start_lnum() -> c_int;
    fn nvim_set_b_visual_from_op();
    fn nvim_inc_b_visual_vi_end();
    fn nvim_last_line_is_empty() -> bool;
    fn nvim_ml_delete_last_line();
    fn nvim_coladvance_maxcol();
    // Phase 1 new lower-level accessors for replace helpers
    fn coladvance_force(col: c_int);
    fn get_cursor_pos_len() -> c_int;
    fn nvim_mb_charlen_cursor() -> c_int;
    fn nvim_curbuf_b_p_et() -> bool;
    fn del_chars(count: c_int, fixpos: bool);
    fn nvim_ins_char_call(c: c_int);
    fn ins_copychar(lnum: c_int) -> c_int;
    fn nvim_ins_char_bytes_from_cap(cap: CapHandle);
    fn set_last_insert(c: c_int);
    fn nvim_set_b_op_start_cursor();
    fn nvim_AppendToRedobuff_composing(cap: CapHandle);
    fn do_pending_operator(cap: CapHandle, old_col: c_int, gui_yank: bool);

    // Phase 3: nv_visual_impl accessors (C functions already exist, add Rust declarations)
    fn nvim_get_resel_VIsual_mode() -> c_int;
    fn nvim_get_resel_VIsual_line_count() -> c_int;
    fn nvim_get_resel_VIsual_vcol() -> c_int;
    fn update_curswant_force();
    // nvim_get_p_smd: inlined (Phase 39, use p_smd directly)
    #[link_name = "p_smd"]
    static p_smd: c_int;
    fn nvim_set_redraw_cmdline(val: bool);

    // Window command functions
    fn rs_do_window(nchar: c_int, count: c_int, xchar: c_int);
}

// Phase 3 constants
const CTRL_Q_P3: c_int = 17; // Ctrl-Q (matches Ctrl_Q in ascii_defs.h)

/// Opaque handle to fmark_T*.
pub type FmarkHandle = *mut std::ffi::c_void;

// Operator type constant for OP_CHANGE - from src/nvim/ops.h
const OP_CHANGE: c_int = 3;

// Fold option flag for block - from build/src/nvim/auto/option_vars.generated.h
const K_OPT_FDO_FLAG_BLOCK: c_uint = 0x02;

// findmatchlimit flags (from search.h)
const FM_BACKWARD: c_int = 0x01;
const FM_FORWARD: c_int = 0x02;

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
        && utf_ptr2char(get_cursor_pos_ptr()) == NUL_CHAR
        && (!VIsual_active || nvim_get_p_sel_first() == sel_o)
        && !virtual_active(nvim_get_curwin())
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
    if VIsual_active
        && nvim_oap_get_inclusive(oap)
        && nvim_get_p_sel_first() == sel_e
        && utf_ptr2char(get_cursor_pos_ptr()) != NUL_CHAR
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
    nvim_curwin_set_curswant(true);

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
        let n = utf_ptr2char(get_cursor_pos_ptr());
        if n != NUL_CHAR && rs_ascii_iswhite(n) == 0 {
            // This is a little strange. To match what the real Vi does, we
            // effectively map "cw" to "ce", and "cW" to "cE", provided that we are
            // not on a space or a TAB. This seems impolite at first, but it's
            // really more what we mean when we say "cw".
            //
            // Another strangeness: When standing on the end of a word "ce" will
            // change until the end of the next word, but "cw" will change only one
            // character! This is done by setting "flag".
            if nvim_vim_strchr_p_cpo(i32::from(b'_')) {
                nvim_oap_set_inclusive(oap, true);
                word_end = true;
            }
            flag = true;
        }
    }

    nvim_oap_set_motion_type(oap, K_MT_CHAR_WISE);
    nvim_curwin_set_curswant(true);

    let n = if word_end {
        end_word(count1, arg != 0, flag, false)
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
    (*oap.cast::<OpargT>()).use_reg_one = true;
    nvim_curwin_set_curswant(true);

    let mut inclusive = false;
    if !findpar(&raw mut inclusive, arg, count1, NUL_CHAR, false) {
        rs_clearopbeep(oap);
        return;
    }
    nvim_oap_set_inclusive(oap, inclusive);

    nvim_set_cursor_coladd(0);
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
    (*oap.cast::<OpargT>()).use_reg_one = true;
    // The motion used to be inclusive for "(", but that is not what Vi does.
    nvim_oap_set_inclusive(oap, false);
    nvim_curwin_set_curswant(true);

    if findsent(arg, count1) == FAIL {
        rs_clearopbeep(oap);
        return;
    }

    // Don't leave the cursor on the NUL past end of line.
    adjust_cursor(oap);
    nvim_set_cursor_coladd(0);
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

const MODE_REPLACE: c_int = 0x110;
const GETF_SETMARK: c_int = 0x01;
const GETF_ALT: c_int = 0x02;

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
        let extra_char = (*cap.cast::<CmdargT>()).extra_char;
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
    let move_res = rs_nv_mark_move_to(cap, flags, fm);

    // May need to clear the coladd that a mark includes.
    if !virtual_active(nvim_get_curwin()) {
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
        move_res = rs_nv_mark_move_to(cap, flags, fm);
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
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_nv_visual(cap: CapHandle) {
    let mut cmdchar = nvim_cap_get_cmdchar(cap);
    // Ctrl-Q is treated the same as Ctrl-V
    if cmdchar == CTRL_Q_P3 {
        cmdchar = CTRL_V;
        (*cap.cast::<CmdargT>()).cmdchar = CTRL_V;
    }

    let oap = nvim_cap_get_oap(cap);

    // 'v', 'V' and CTRL-V can be used while an operator is pending
    // to make it charwise, linewise, or blockwise.
    if nvim_oap_get_op_type_ptr(oap) != OP_NOP {
        (*oap.cast::<OpargT>()).motion_force = cmdchar;
        nvim_set_motion_force(cmdchar);
        nvim_set_finish_op(false); // operator doesn't finish now but later
        return;
    }

    // VIsual_select = cap->arg (arg != 0 means select mode)
    nvim_set_VIsual_select(nvim_cap_get_arg(cap) != 0);

    if VIsual_active {
        // change Visual mode
        if nvim_get_VIsual_mode() == cmdchar {
            // stop visual mode
            rs_end_visual_mode();
        } else {
            // toggle char/block mode or char/line mode
            nvim_set_VIsual_mode(cmdchar);
            nvim_showmode();
            nvim_may_trigger_modechanged();
        }
        nvim_redraw_curbuf_inverted(); // update the inversion
    } else {
        // start Visual mode
        let count0 = nvim_cap_get_count0(cap);
        let resel_mode = nvim_get_resel_VIsual_mode();
        if count0 > 0 && resel_mode != 0 {
            // use previously selected part
            // VIsual = curwin->w_cursor
            nvim_set_VIsual_pos(
                nvim_get_cursor_lnum(),
                nvim_get_cursor_col(),
                nvim_get_cursor_coladd(),
            );
            nvim_set_VIsual_active(true);
            nvim_set_VIsual_reselect(true);
            if nvim_cap_get_arg(cap) == 0 {
                // start Select mode when 'selectmode' contains "cmd"
                rs_may_start_select(c_int::from(b'c'));
            }
            nvim_setmouse();
            if p_smd != 0 && msg_silent == 0 {
                nvim_set_redraw_cmdline(true); // show visual mode later
            }
            let resel_line_count = nvim_get_resel_VIsual_line_count();
            let resel_vcol = nvim_get_resel_VIsual_vcol();
            // For V and ^V, multiply number of lines
            if resel_mode != c_int::from(b'v') || resel_line_count > 1 {
                let new_lnum = nvim_get_cursor_lnum() + resel_line_count * count0 - 1;
                nvim_set_cursor_lnum(new_lnum);
                nvim_check_cursor();
            }
            nvim_set_VIsual_mode(resel_mode);
            if resel_mode == c_int::from(b'v') {
                if resel_line_count <= 1 {
                    update_curswant_force();
                    let new_cw = nvim_get_curswant() + resel_vcol * count0;
                    nvim_set_curswant(new_cw);
                    if !nvim_p_sel_is_exclusive() {
                        nvim_set_curswant(nvim_get_curswant() - 1);
                    }
                } else {
                    nvim_set_curswant(resel_vcol);
                }
                coladvance(nvim_get_curwin(), nvim_get_curswant());
            }
            if resel_vcol == MAXCOL {
                nvim_set_curswant(MAXCOL);
                coladvance(nvim_get_curwin(), MAXCOL);
            } else if nvim_get_VIsual_mode() == CTRL_V {
                // Update curswant on the original line (where "col" is valid)
                let lnum = nvim_get_cursor_lnum();
                nvim_set_cursor_lnum(nvim_get_VIsual_lnum());
                update_curswant_force();
                let new_cw = nvim_get_curswant() + resel_vcol * count0 - 1;
                nvim_set_curswant(new_cw);
                nvim_set_cursor_lnum(lnum);
                if nvim_p_sel_is_exclusive() {
                    nvim_set_curswant(nvim_get_curswant() + 1);
                }
                coladvance(nvim_get_curwin(), nvim_get_curswant());
            } else {
                nvim_curwin_set_curswant(true);
            }
            nvim_redraw_curbuf_inverted(); // show the inversion
        } else {
            if nvim_cap_get_arg(cap) == 0 {
                // start Select mode when 'selectmode' contains "cmd"
                rs_may_start_select(c_int::from(b'c'));
            }
            rs_n_start_visual_mode(cmdchar);
            if nvim_get_VIsual_mode() != c_int::from(b'V') && nvim_p_sel_is_exclusive() {
                // include one more char
                let c1 = nvim_cap_get_count1(cap);
                nvim_cap_set_count1(cap, c1 + 1);
            } else {
                nvim_set_VIsual_select_exclu_adj(false);
            }
            if count0 > 0 && {
                (*cap.cast::<CmdargT>()).count1 -= 1;
                (*cap.cast::<CmdargT>()).count1 > 0
            } {
                // With a count select that many characters or lines.
                let vmode = nvim_get_VIsual_mode();
                if vmode == c_int::from(b'v') || vmode == CTRL_V {
                    rs_nv_right(cap);
                } else if vmode == c_int::from(b'V') {
                    rs_nv_down(cap);
                }
            }
        }
    }
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
        (*cap.cast::<CmdargT>()).cmdchar = c_int::from(b':');
        (*cap.cast::<CmdargT>()).nchar = c_int::from(NUL);
        rs_nv_colon(cap);
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
        (*cap.cast::<CmdargT>()).opcount = count0; // remember count before '"'
        set_reg_var(nchar);
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
    nv_put_opt_impl(cap, false);
}

// PUT_* flag constants for do_put()
const PUT_FIXINDENT: c_int = 1;
const PUT_CURSEND: c_int = 2;
const PUT_BLOCK_INNER: c_int = 64;

// Clipboard flag constants
const CB_FLAG_UNNAMED: c_int = 0x01;
const CB_FLAG_UNNAMEDPLUS: c_int = 0x02;

// PUT_LINE_* flag constants for do_put() visual mode
const PUT_LINE: c_int = 4;
const PUT_LINE_SPLIT: c_int = 8;
const PUT_LINE_FORWARD: c_int = 16;

/// Implementation of nv_put_opt - paste with optional indent fixing.
/// Inlines helpers: nvim_put_check_op_type, nvim_put_check_prompt,
/// nvim_put_visual_delete, nvim_put_visual_flags, nvim_put_was_visual_cleanup,
/// nvim_put_delete_empty_line.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[allow(clippy::cast_lossless)]
#[allow(clippy::too_many_lines)]
unsafe fn nv_put_opt_impl(cap: CapHandle, fix_indent: bool) {
    let oap = nvim_cap_get_oap(cap);

    // Inlined nvim_put_check_op_type: check if there's a pending operator
    let op_type = nvim_oap_get_op_type_ptr(oap);
    if op_type != OP_NOP {
        let cmdchar = nvim_cap_get_cmdchar(cap);
        let opcount = (*cap.cast::<CmdargT>()).opcount;
        if op_type == OP_DELETE && cmdchar == b'p' as c_int {
            rs_clearop(oap);
            assert!(opcount >= 0);
            nv_diffgetput(true, opcount.unsigned_abs() as usize);
        } else {
            rs_clearopbeep(oap);
        }
        return;
    }

    // Inlined nvim_put_check_prompt: check prompt buffer restrictions
    if nvim_bt_prompt_curbuf() && !prompt_curpos_editable() {
        let cursor_lnum = nvim_get_cursor_lnum();
        let b_prompt_lnum = nvim_get_b_prompt_start_lnum_put();
        if cursor_lnum == b_prompt_lnum {
            nvim_set_cursor_col_to_prompt_text_len();
            (*cap.cast::<CmdargT>()).cmdchar = b'P' as c_int;
            // continue (return 0 in C means don't do early return)
        } else {
            rs_clearopbeep(oap);
            return;
        }
    }

    let save_fen = nvim_put_get_save_fen();
    let mut savereg: *mut std::ffi::c_void = std::ptr::null_mut();
    let mut empty = false;
    let mut was_visual = false;

    // Determine direction and flags
    let mut dir;
    let mut flags: c_int = 0;
    let cmdchar = nvim_cap_get_cmdchar(cap);
    let nchar = nvim_cap_get_nchar(cap);

    if fix_indent {
        dir = if cmdchar == b']' as c_int && nchar == b'p' as c_int {
            FORWARD
        } else {
            BACKWARD
        };
        flags |= PUT_FIXINDENT;
    } else {
        dir = if cmdchar == b'P' as c_int
            || ((cmdchar == b'g' as c_int || cmdchar == b'z' as c_int) && nchar == b'P' as c_int)
        {
            BACKWARD
        } else {
            FORWARD
        };
    }

    rs_prep_redo_cmd(cap);

    let cmdchar = nvim_cap_get_cmdchar(cap);
    if cmdchar == b'g' as c_int {
        flags |= PUT_CURSEND;
    } else if cmdchar == b'z' as c_int {
        flags |= PUT_BLOCK_INNER;
    }

    if VIsual_active {
        was_visual = true;
        let regname = nvim_oap_get_regname_ptr(oap);
        let keep_registers = nvim_cap_get_cmdchar(cap) == b'P' as c_int;
        let clipoverwrite = (regname == b'+' as c_int || regname == b'*' as c_int)
            && (nvim_get_cb_flags() & (CB_FLAG_UNNAMED | CB_FLAG_UNNAMEDPLUS)) != 0;
        if regname == 0
            || regname == b'"' as c_int
            || clipoverwrite
            || rs_ascii_isdigit(regname) != 0
            || regname == b'-' as c_int
        {
            savereg = nvim_put_copy_register(regname);
        }

        // Inlined nvim_put_visual_delete: delete visual selection before put
        nvim_set_w_p_fen(false);
        if nvim_check_vd_condition(regname) {
            (*cap.cast::<CmdargT>()).cmdchar = b'd' as c_int;
            (*cap.cast::<CmdargT>()).nchar = NUL_CHAR;
            let underscore = if keep_registers {
                b'_' as c_int
            } else {
                NUL_CHAR
            };
            nvim_oap_set_regname(oap, underscore);
            nvim_inc_msg_silent();
            rs_nv_operator(cap);
            do_pending_operator(cap, 0, false);
            empty = nvim_curbuf_ml_empty();
            nvim_dec_msg_silent();
            nvim_oap_set_regname(oap, regname);
        }

        // Inlined nvim_put_visual_flags: compute put flags for visual mode
        let vis_mode = nvim_get_VIsual_mode();
        if vis_mode == b'V' as c_int {
            flags |= PUT_LINE;
        } else if vis_mode == b'v' as c_int {
            flags |= PUT_LINE_SPLIT;
        }
        if vis_mode == CTRL_V && dir == FORWARD {
            flags |= PUT_LINE_FORWARD;
        }
        dir = BACKWARD;
        if (vis_mode != b'V' as c_int && nvim_get_cursor_col_vs_b_op_start_col() < 0)
            || (vis_mode == b'V' as c_int && nvim_get_cursor_lnum_vs_b_op_start_lnum() < 0)
        {
            dir = FORWARD;
        }
        // VIsual_active = true (needed after delete)
        nvim_set_VIsual_active(true);
    }

    let regname = nvim_oap_get_regname_ptr(oap);
    let count1 = nvim_cap_get_count1(cap);
    nvim_put_do_put(regname, savereg, dir, count1, flags);

    // Free saved register
    nvim_put_free_register(savereg);

    if was_visual {
        // Inlined nvim_put_was_visual_cleanup: restore visual state after put
        if save_fen {
            nvim_set_w_p_fen(true);
        }
        nvim_set_b_visual_from_op();
        #[allow(clippy::cast_possible_wrap)]
        let sel_e_char = b'e' as std::ffi::c_char;
        if nvim_get_p_sel_first() == sel_e_char {
            nvim_inc_b_visual_vi_end();
        }
    }

    // Inlined nvim_put_delete_empty_line: delete trailing empty line after put
    if empty && nvim_last_line_is_empty() {
        nvim_ml_delete_last_line();
        if nvim_get_cursor_lnum() > nvim_get_curwin_ml_line_count() as c_int {
            nvim_set_cursor_lnum(nvim_get_curwin_ml_line_count() as c_int);
            nvim_coladvance_maxcol();
        }
    }
    auto_format(false, true);
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
        && VIsual_active
        && nvim_get_VIsual_mode() == visual_v
        && nvim_get_VIsual_select_exclu_adj()
    {
        nvim_unadjust_for_sel();
        cursor_dec = true;
    }

    let t_cmd = cmdchar == c_int::from(b't') || cmdchar == c_int::from(b'T');

    nvim_oap_set_motion_type(oap, K_MT_CHAR_WISE);
    if nchar < 0 || searchc(cap, t_cmd) == FAIL {
        rs_clearopbeep(oap);
        // Revert unadjust when failed.
        if cursor_dec {
            adjust_for_sel(cap);
        }
        return;
    }

    nvim_curwin_set_curswant(true);
    // Include a Tab for "tx" and for "dfx".
    if utf_ptr2char(get_cursor_pos_ptr()) == TAB_CHAR
        && virtual_active(nvim_get_curwin())
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
    // nvim_nv_clear_impl migrated to Rust rs_nv_clear_impl (Phase 6)
    fn nvim_syn_stack_free_all_curwin();
    fn nvim_clear_b_syn_slow_all_windows();
    #[allow(dead_code)]
    fn nvim_get_restart_VIsual_select() -> c_int;
    fn nvim_set_restart_VIsual_select(val: c_int);
    fn nvim_buflist_getfile(n: c_int, lnum: c_int, flags: c_int, setpm: bool);
    // Phase 4 accessors
    fn nvim_get_ex_normal_busy() -> c_int;
    fn nvim_get_typebuf_was_empty() -> bool;
    fn nvim_vim_beep_esc();
    fn nvim_get_curbuf_terminal() -> bool;
    fn nvim_esc_show_msg();
    fn nvim_getviscol() -> c_int;
    fn edit(cmd: c_int, startln: bool, count: c_int) -> bool;
    fn vim_append_digit_int(n_ptr: *mut c_int, digit: c_int) -> c_int;
    fn rs_win_setheight(height: c_int);
}

/// Inner implementation of CTRL-L clear: frees syntax states and redraws.
///
/// Migrated from `nvim_nv_clear_impl` in normal_shim.c (Phase 6).
///
/// # Safety
/// Calls C functions to free syntax state and mark for redraw.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_clear_impl() {
    nvim_syn_stack_free_all_curwin();
    nvim_clear_b_syn_slow_all_windows();
    redraw_later(nvim_get_curwin(), UPD_CLEAR);
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
    rs_nv_clear_impl();
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
    if VIsual_active && nvim_get_VIsual_select() {
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
        let flags = GETF_SETMARK | GETF_ALT;
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
    let oap = nvim_cap_get_oap(cap);
    if rs_checkclearopq(oap) {
        return;
    }
    let nchar = nvim_cap_get_nchar(cap);
    if nchar == c_int::from(b'Z') {
        // "ZZ": equivalent to ":x".
        do_cmdline_cmd(c"x".as_ptr());
    } else if nchar == c_int::from(b'Q') {
        // "ZQ": equivalent to ":q!" (Elvis compatible).
        do_cmdline_cmd(c"q!".as_ptr());
    } else {
        rs_clearopbeep(oap);
    }
}

/// Invoke edit() and take care of restart_edit and the return value.
///
/// Port of C `invoke_edit()`.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
unsafe fn invoke_edit_impl(cap: CapHandle, repl: bool, cmd: c_int, startln: bool) {
    // Complicated: When the user types "a<C-O>a" we don't want to do Insert
    // mode recursively.  But when doing "a<C-O>." or "a<C-O>rx" we do allow it.
    let restart_edit_save = if repl || !nvim_stuff_empty() {
        restart_edit
    } else {
        0
    };
    // Always reset "restart_edit", this is not a restarted edit.
    restart_edit = 0;
    // Reset b_last_changedtick_i, so that TextChangedI will only be triggered
    // for stuff from insert mode; for 'o/O' this has already been done in n_opencmd.
    let cmdchar = nvim_cap_get_cmdchar(cap);
    if cmdchar != c_int::from(b'O') && cmdchar != c_int::from(b'o') {
        nvim_set_curbuf_b_last_changedtick_i();
    }
    if edit(cmd, startln, nvim_cap_get_count1(cap)) {
        (*cap.cast::<CmdargT>()).retval |= CA_COMMAND_BUSY;
    }
    if restart_edit == 0 {
        restart_edit = restart_edit_save;
    }
}

/// FFI export for `invoke_edit` (used by C callers n_opencmd, nvim_invoke_edit_R, etc.).
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[export_name = "invoke_edit"]
pub unsafe extern "C" fn rs_invoke_edit(cap: CapHandle, repl: bool, cmd: c_int, startln: bool) {
    invoke_edit_impl(cap, repl, cmd, startln);
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
    let oap = nvim_cap_get_oap(cap);
    let no_reason = nvim_oap_get_op_type_ptr(oap) == OP_NOP
        && (*cap.cast::<CmdargT>()).opcount == 0
        && nvim_cap_get_count0(cap) == 0
        && nvim_oap_get_regname_ptr(oap) == 0;

    if nvim_cap_get_arg(cap) != 0 {
        // true for CTRL-C
        if restart_edit == 0 && nvim_normal_get_cmdwin_type() == 0 && !VIsual_active && no_reason {
            nvim_esc_show_msg();
        }
        if restart_edit != 0 {
            redraw_mode = 1; // remove "-- (insert) --"
        }
        restart_edit = 0;
        if nvim_normal_get_cmdwin_type() != 0 {
            nvim_set_cmdwin_result(K_IGNORE);
            unsafe {
                got_int = false;
            } // don't stop executing autocommands et al.
            return;
        }
    } else if nvim_normal_get_cmdwin_type() != 0
        && nvim_get_ex_normal_busy() != 0
        && nvim_get_typebuf_was_empty()
    {
        // When :normal runs out of characters while in the command line window
        // vgetorpeek() will repeatedly return ESC.  Exit the cmdline window to
        // break the loop.
        nvim_set_cmdwin_result(K_IGNORE);
        return;
    }

    if VIsual_active {
        rs_end_visual_mode(); // stop Visual
        check_cursor_col(nvim_get_curwin()); // make sure cursor is not beyond EOL
        nvim_curwin_set_curswant(true);
        nvim_redraw_curbuf_inverted();
    } else if no_reason {
        nvim_vim_beep_esc();
    }
    rs_clearop(oap);
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
    let mut cmdchar = nvim_cap_get_cmdchar(cap);
    // <Insert> is equal to "i"
    if cmdchar == K_INS || cmdchar == K_KINS {
        cmdchar = c_int::from(b'i');
        (*cap.cast::<CmdargT>()).cmdchar = cmdchar;
    }
    let oap = nvim_cap_get_oap(cap);
    // in Visual mode "A" and "I" are an operator
    if VIsual_active && (cmdchar == c_int::from(b'A') || cmdchar == c_int::from(b'I')) {
        rs_v_visop(cap);
    // in Visual mode and after an operator "a" and "i" are for text objects
    } else if (cmdchar == c_int::from(b'a') || cmdchar == c_int::from(b'i'))
        && (nvim_oap_get_op_type_ptr(oap) != OP_NOP || VIsual_active)
    {
        rs_nv_object(cap);
    } else if !nvim_curbuf_modifiable() && !nvim_get_curbuf_terminal() {
        nvim_emsg_modifiable();
        rs_clearop(oap);
    } else if !rs_checkclearopq(oap) {
        if cmdchar == c_int::from(b'A') {
            // "A"ppend after the line
            rs_set_cursor_for_append_to_line();
        } else if cmdchar == c_int::from(b'I') {
            // "I"nsert before the first non-blank
            nvim_beginline(BL_WHITE);
        } else if cmdchar == c_int::from(b'a') {
            // "a"ppend is like "i"nsert on the next character.
            // increment coladd when in virtual space, increment the
            // column otherwise, also to append after an unprintable char
            if virtual_active(nvim_get_curwin())
                && (nvim_get_cursor_coladd() > 0
                    || utf_ptr2char(get_cursor_pos_ptr()) == NUL_CHAR
                    || utf_ptr2char(get_cursor_pos_ptr()) == TAB_CHAR)
            {
                nvim_set_cursor_coladd(nvim_get_cursor_coladd() + 1);
            } else if utf_ptr2char(get_cursor_pos_ptr()) != NUL_CHAR {
                nvim_inc_cursor();
            }
        }
        if nvim_get_cursor_coladd() != 0 && cmdchar != c_int::from(b'A') {
            let save_state = State;
            // Pretend Insert mode here to allow the cursor on the
            // character past the end of the line
            State = MODE_INSERT;
            coladvance(nvim_get_curwin(), nvim_getviscol());
            State = save_state;
        }
        invoke_edit_impl(cap, false, cmdchar, false);
    }
}

// =============================================================================
// Phase 2 Command Handlers (Search)
// =============================================================================

extern "C" {
    // Phase 4: nv_search / nv_next accessors
    fn nvim_getcmdline_for_search(cap: CapHandle) -> *mut c_char;
    // Accessors for rs_ident_build_and_exec (Phase 2 migration)
    fn nvim_ident_get_kp() -> *mut c_char;
    fn nvim_ident_curbuf_is_help() -> bool;
    fn nvim_ident_get_curbuf_ft() -> *mut c_char;
    fn nvim_ident_get_cursor_line_ptr() -> *mut c_char;
    fn vim_iswordp(p: *const c_char) -> bool;
    fn nvim_ident_mb_prevptr(line: *mut c_char, p: *mut c_char) -> *mut c_char;
    fn nvim_ident_set_g_tag_at_cursor(val: bool);

    fn nvim_set_no_smartcase(val: c_int);
    fn rs_magic_isset() -> c_int;
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn vim_strchr(s: *const c_char, c: c_int) -> *mut c_char;
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn ml_get(lnum: c_int) -> *const c_char;
    static p_ww: *mut c_char;
    fn vim_strsave_fnameescape(s: *const c_char, what: c_int) -> *mut c_char;
    fn vim_strsave_shellescape(s: *const c_char, do_special: bool, do_newline: bool)
        -> *mut c_char;
    fn xstrnsave(s: *const c_char, len: usize) -> *mut c_char;
    fn xfree(p: *mut c_void);
    fn add_map(lhs: *mut c_char, rhs: *mut c_char, mode: c_int, buffer: bool);

    // History functions (Rust exports from cmdhist crate)
    fn init_history();
    fn add_to_history(
        histype: c_int,
        new_entry: *const c_char,
        new_entrylen: usize,
        in_map: bool,
        sep: c_int,
    );
}

/// Command handler for "/" and "?" commands: Search forward/backward.
///
/// cap->arg is true to not set PC mark.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_search(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    let cmdchar = nvim_cap_get_cmdchar(cap);

    if cmdchar == c_int::from(b'?') && nvim_oap_get_op_type_ptr(oap) == OP_ROT13 {
        // Translate "g??" to "g?g?"
        (*cap.cast::<CmdargT>()).cmdchar = c_int::from(b'g');
        (*cap.cast::<CmdargT>()).nchar = c_int::from(b'?');
        rs_nv_operator(cap);
        return;
    }

    // Save cursor position before getcmdline (incsearch may move cursor).
    let save_lnum = nvim_get_cursor_lnum();
    let save_col = nvim_get_cursor_col();
    let save_coladd = nvim_get_cursor_coladd();

    // When using 'incsearch' the cursor may be moved to set a different search start position.
    let pat = nvim_getcmdline_for_search(cap);

    if pat.is_null() {
        rs_clearop(oap);
        return;
    }

    // If cap->arg is set or cursor moved (incsearch), skip setting PC mark.
    let cursor_moved = nvim_get_cursor_lnum() != save_lnum
        || nvim_get_cursor_col() != save_col
        || nvim_get_cursor_coladd() != save_coladd;
    let arg = nvim_cap_get_arg(cap);
    let opt = if arg != 0 || cursor_moved {
        0
    } else {
        SEARCH_MARK
    };

    let patlen = std::ffi::CStr::from_ptr(pat).to_bytes().len();
    rs_normal_search(cap, cmdchar, pat, patlen, opt, core::ptr::null_mut());
}

/// Command handler for "n" and "N" commands: Repeat search.
///
/// cap->arg is SEARCH_REV for "N", 0 for "n".
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_next(cap: CapHandle) {
    // Save cursor position to detect if search left us in the same spot.
    let old_lnum = nvim_get_cursor_lnum();
    let old_col = nvim_get_cursor_col();
    let old_coladd = nvim_get_cursor_coladd();

    let arg = nvim_cap_get_arg(cap);
    let mut wrapped: c_int = 0;
    let i = rs_normal_search(
        cap,
        0,
        core::ptr::null_mut(),
        0,
        SEARCH_MARK | arg,
        &raw mut wrapped,
    );

    if i == 1
        && wrapped == 0
        && nvim_get_cursor_lnum() == old_lnum
        && nvim_get_cursor_col() == old_col
        && nvim_get_cursor_coladd() == old_coladd
    {
        // Avoid getting stuck on current cursor position.
        // Repeat with count + 1.
        let count1 = nvim_cap_get_count1(cap);
        nvim_cap_set_count1(cap, count1 + 1);
        rs_normal_search(
            cap,
            0,
            core::ptr::null_mut(),
            0,
            SEARCH_MARK | arg,
            core::ptr::null_mut(),
        );
        nvim_cap_set_count1(cap, count1);
    }
    // Note: hlsearch redraw is now handled inside rs_normal_search
}

/// Command handler for identifier commands: *, #, K, CTRL-], g], g*.
///
/// Handles searching for the word under cursor and related operations.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_ident(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);

    // Inlined nvim_ident_init:
    let raw_cmdchar = nvim_cap_get_cmdchar(cap);
    let (mut cmdchar, g_cmd): (c_int, c_int) = if raw_cmdchar == c_int::from(b'g') {
        (nvim_cap_get_nchar(cap), 1)
    } else {
        (raw_cmdchar, 0)
    };
    if cmdchar == POUND {
        cmdchar = c_int::from(b'#');
    }

    let mut ptr: *mut c_char = core::ptr::null_mut();
    let mut n: usize = 0;

    // The "]", "CTRL-]" and "K" commands accept an argument in Visual mode.
    if cmdchar == c_int::from(b']') || cmdchar == CTRL_RSB || cmdchar == c_int::from(b'K') {
        if VIsual_active && !rs_get_visual_text(cap, &raw mut ptr, &raw mut n) {
            return;
        }
        if rs_checkclearopq(oap) {
            return;
        }
    }

    if ptr.is_null() {
        let find_type = if cmdchar == c_int::from(b'*') || cmdchar == c_int::from(b'#') {
            FIND_IDENT | FIND_STRING
        } else {
            FIND_IDENT
        };
        n = rs_find_ident_at_pos(
            nvim_get_curwin(),
            nvim_get_cursor_lnum(),
            nvim_get_cursor_col(),
            &raw mut ptr,
            core::ptr::null_mut(),
            find_type,
        );
        if n == 0 {
            rs_clearop(oap);
            return;
        }
    }

    rs_ident_build_and_exec(cap, cmdchar, g_cmd, ptr, n);
}

// VSE_NONE: flag for vim_strsave_fnameescape (no special escaping).
const VSE_NONE: c_int = 0;

// HIST_SEARCH: history type for search patterns.
const HIST_SEARCH: c_int = 1;

// MODE_INSERT: insert mode flag.
const MODE_INSERT: c_int = 0x10;
// MODE_TERMINAL: terminal mode flag.
const MODE_TERMINAL: c_int = 0x80;

/// Append a C string (char*) to a Vec<u8>, up to `len` bytes.
///
/// # Safety
/// `s` must point to at least `len` valid bytes.
#[allow(clippy::ptr_as_ptr)]
unsafe fn append_cptr_bytes(buf: &mut Vec<u8>, s: *const c_char, len: usize) {
    // SAFETY: Caller guarantees s points to len valid bytes.
    unsafe {
        buf.extend_from_slice(std::slice::from_raw_parts(s.cast::<u8>(), len));
    }
}

/// Private helper: implements the K command's keywordprg logic (was nv_K_getcmd).
///
/// Returns the (possibly adjusted) `n` on success, or `None` to abort.
/// `ptr` may be advanced past leading dashes for external programs.
///
/// # Safety
/// All pointers must be valid.
unsafe fn nv_k_getcmd(
    cap: CapHandle,
    kp: *const c_char,
    kp_help: bool,
    kp_ex: bool,
    ptr: &mut *mut c_char,
    mut n: usize,
    buf: &mut Vec<u8>,
) -> Option<usize> {
    // SAFETY: All pointer ops use valid pointers from caller.
    unsafe {
        if kp_help {
            // In the help buffer: use "he! " prefix
            buf.extend_from_slice(b"he! ");
            return Some(n);
        }

        if kp_ex {
            // 'keywordprg' is an ex command
            buf.clear();
            let count0 = nvim_cap_get_count0(cap);
            if count0 != 0 {
                // Send count to the ex command
                let count_str = format!("{count0}");
                buf.extend_from_slice(count_str.as_bytes());
            }
            let kp_len = libc::strlen(kp);
            append_cptr_bytes(buf, kp, kp_len);
            buf.push(b' ');
            return Some(n);
        }

        // External command: skip leading dashes
        #[allow(clippy::cast_sign_loss)]
        while *(*ptr).cast::<u8>() == b'-' && n > 0 {
            *ptr = (*ptr).add(1);
            n -= 1;
        }
        if n == 0 {
            // found dashes only
            nvim_emsg_no_ident_under_cursor();
            return None;
        }

        // When a count is given, turn it into a range.
        let kp_cstr = std::ffi::CStr::from_ptr(kp);
        let kp_bytes = kp_cstr.to_bytes();
        let isman = kp_bytes == b"man";
        let isman_s = kp_bytes == b"man -s";

        let count0 = nvim_cap_get_count0(cap);
        if count0 != 0 && !(isman || isman_s) {
            let range_str = format!(".,.+{}", count0 - 1);
            buf.extend_from_slice(range_str.as_bytes());
        }

        // Open a new tab for the terminal
        do_cmdline_cmd(c"tabnew".as_ptr());

        // Add "terminal " prefix
        buf.extend_from_slice(b"terminal ");

        if count0 == 0 && isman_s {
            buf.extend_from_slice(b"man ");
        } else {
            let kp_len = libc::strlen(kp);
            append_cptr_bytes(buf, kp, kp_len);
            buf.push(b' ');
        }

        if count0 != 0 && (isman || isman_s) {
            let count_str = format!("{count0} ");
            buf.extend_from_slice(count_str.as_bytes());
        }

        Some(n)
    }
}

/// Build and execute the command for *, #, K, CTRL-], g], g* commands.
///
/// Translated from C `nvim_ident_build_and_exec` in `normal_shim.c`.
/// Absorbs `nv_K_getcmd` (formerly a static C helper).
///
/// # Safety
/// - `cap` must be a valid `cmdarg_T*`.
/// - `ptr` must point to at least `n` bytes of identifier text.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_ident_build_and_exec(
    cap: CapHandle,
    cmdchar: c_int,
    g_cmd: c_int,
    mut ptr: *mut c_char,
    mut n: usize,
) {
    // SAFETY: All pointer operations assume valid pointers from C callers.
    unsafe {
        let kp = nvim_ident_get_kp();
        #[allow(clippy::cast_sign_loss)]
        let kp_help = *kp.cast::<u8>() == 0
            || libc::strcmp(kp, c":he".as_ptr()) == 0
            || libc::strcmp(kp, c":help".as_ptr()) == 0;

        #[allow(clippy::cast_sign_loss)]
        if kp_help && *skipwhite(ptr.cast_const()).cast::<u8>() == 0 {
            nvim_emsg_no_ident_under_cursor();
            return;
        }
        #[allow(clippy::cast_sign_loss)]
        let kp_ex = *kp.cast::<u8>() == b':';

        // Build command buffer (replaces xmalloc)
        let kp_len = libc::strlen(kp);
        let initial_cap = n * 2 + 30 + kp_len;
        let mut buf: Vec<u8> = Vec::with_capacity(initial_cap);

        let mut tag_cmd = false;
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let cmdchar_byte = cmdchar as u8;

        match cmdchar_byte {
            b'*' | b'#' => {
                nvim_setpcmark();
                // Compute column as byte offset from line start
                let line_start = nvim_ident_get_cursor_line_ptr();
                #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                let cursor_col = (ptr as usize).wrapping_sub(line_start as usize) as c_int;
                nvim_set_cursor_col(cursor_col);
                if g_cmd == 0 && vim_iswordp(ptr) {
                    buf.extend_from_slice(b"\\<");
                }
                nvim_set_no_smartcase(1);
            }
            b'K' => match nv_k_getcmd(cap, kp, kp_help, kp_ex, &mut ptr, n, &mut buf) {
                None => return,
                Some(new_n) => n = new_n,
            },
            b']' => {
                tag_cmd = true;
                buf.extend_from_slice(b"ts ");
            }
            _ => {
                tag_cmd = true;
                if nvim_ident_curbuf_is_help() {
                    buf.extend_from_slice(b"he! ");
                } else if g_cmd != 0 {
                    buf.extend_from_slice(b"tj ");
                } else {
                    let count0 = nvim_cap_get_count0(cap);
                    if count0 == 0 {
                        buf.extend_from_slice(b"ta ");
                    } else {
                        let count_str = format!(":{count0}ta ");
                        buf.extend_from_slice(count_str.as_bytes());
                    }
                }
            }
        }

        // Grab the chars in the identifier
        if cmdchar_byte == b'K' && !kp_help {
            // Save ptr as a C string for escaping
            let saved_ptr = xstrnsave(ptr, n);
            let escaped = if kp_ex {
                vim_strsave_fnameescape(saved_ptr, VSE_NONE)
            } else {
                vim_strsave_shellescape(saved_ptr, true, true)
            };
            xfree(saved_ptr.cast::<c_void>());
            let plen = libc::strlen(escaped);
            append_cptr_bytes(&mut buf, escaped, plen);
            xfree(escaped.cast::<c_void>());
        } else {
            let magic = rs_magic_isset() != 0;
            // We need a NUL-terminated aux string for vim_strchr.
            // Build it as a Vec<u8> to avoid repeated allocations.
            let aux_bytes: &[u8] = if cmdchar_byte == b'*' {
                if magic {
                    b"/.*~[^$\\"
                } else {
                    b"/^$\\"
                }
            } else if cmdchar_byte == b'#' {
                if magic {
                    b"/?.*~[^$\\"
                } else {
                    b"/?^$\\"
                }
            } else if tag_cmd {
                let ft = nvim_ident_get_curbuf_ft();
                let ft_cstr = std::ffi::CStr::from_ptr(ft);
                if ft_cstr.to_bytes() == b"help" {
                    b""
                } else {
                    b"\\|\"\n["
                }
            } else {
                b"\\|\"\n*?["
            };

            let mut aux_c_str: Vec<u8> = aux_bytes.to_vec();
            aux_c_str.push(0);

            let mut remaining = n;
            let mut src = ptr;
            while remaining > 0 {
                #[allow(clippy::cast_sign_loss)]
                let ch = *src.cast::<u8>();
                // Check if this char needs escaping
                #[allow(clippy::cast_possible_wrap)]
                if !vim_strchr(aux_c_str.as_ptr().cast(), c_int::from(ch)).is_null() {
                    buf.push(b'\\');
                }
                #[allow(clippy::cast_sign_loss)]
                let char_len = utfc_ptr2len(src) as usize;
                let multi_len = char_len.saturating_sub(1);
                // Copy multi-byte continuation bytes (all but the last)
                for i in 0..multi_len {
                    if remaining == 0 {
                        break;
                    }
                    #[allow(clippy::cast_sign_loss)]
                    buf.push(*src.add(i).cast::<u8>());
                    remaining -= 1;
                }
                src = src.add(multi_len);
                // Copy the final byte of this char
                #[allow(clippy::cast_sign_loss)]
                buf.push(*src.cast::<u8>());
                src = src.add(1);
                remaining -= 1;
            }
        }

        // Execute the command
        if cmdchar_byte == b'*' || cmdchar_byte == b'#' {
            let line_ptr = nvim_ident_get_cursor_line_ptr();
            if g_cmd == 0 && vim_iswordp(nvim_ident_mb_prevptr(line_ptr, ptr)) {
                buf.extend_from_slice(b"\\>");
            }
            init_history();
            // add_to_history reads but doesn't own the bytes
            add_to_history(
                HIST_SEARCH,
                buf.as_ptr().cast::<c_char>(),
                buf.len(),
                true,
                0,
            );
            rs_normal_search(
                cap,
                if cmdchar_byte == b'*' {
                    c_int::from(b'/')
                } else {
                    c_int::from(b'?')
                },
                buf.as_mut_ptr().cast::<c_char>(),
                buf.len(),
                0,
                core::ptr::null_mut(),
            );
        } else {
            nvim_ident_set_g_tag_at_cursor(true);
            buf.push(0); // NUL terminate
            do_cmdline_cmd(buf.as_ptr().cast::<c_char>());
            buf.pop(); // remove NUL
            nvim_ident_set_g_tag_at_cursor(false);
            if cmdchar_byte == b'K' && !kp_ex && !kp_help {
                restart_edit = c_int::from(b'i');
                add_map(
                    c"<esc>".as_ptr().cast_mut(),
                    c"<Cmd>bdelete!<CR>".as_ptr().cast_mut(),
                    MODE_TERMINAL,
                    true,
                );
            }
        }
    }
}

// =============================================================================
// Phase 3 Command Handlers (Operators) -- now real Rust implementations
// =============================================================================

extern "C" {
    // Phase 3 accessors
    fn nvim_get_p_to() -> bool;
    fn nvim_bt_prompt_curbuf() -> bool;
    fn prompt_curpos_editable() -> bool;
    fn op_is_change(op_type: c_int) -> bool;
    fn stuffnumReadbuff(n: c_int);
    fn stuffReadbuff(s: *const c_char);
    fn nvim_get_op_type_wrapper(c1: c_int, c2: c_int) -> c_int;
}

// OP_TILDE constant
const OP_TILDE: c_int = 7;

// Table for nv_optrans: maps command chars to replacement strings
// Same order as the C static arrays: str="xXDCsSY&", ar={"dl","dh","d$","c$","cl","cc","yy",":s\r"}
const OPTRANS_STR: &[u8] = b"xXDCsSY&";
const OPTRANS_AR: [&[u8]; 8] = [b"dl", b"dh", b"d$", b"c$", b"cl", b"cc", b"yy", b":s\r"];

/// Internal helper: implement the operator command setup logic.
/// This is called by rs_nv_operator, rs_nv_tilde (when acting as operator), and rs_nv_subst.
unsafe fn nv_operator_impl(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    let cmdchar = nvim_cap_get_cmdchar(cap);
    let nchar = nvim_cap_get_nchar(cap);
    let op_type = nvim_get_op_type_wrapper(cmdchar, nchar);

    if nvim_bt_prompt_curbuf() && op_is_change(op_type) && !prompt_curpos_editable() {
        rs_clearopbeep(oap);
        return;
    }

    if op_type == nvim_oap_get_op_type_ptr(oap) {
        // double operator works on lines
        rs_nv_lineop(cap);
    } else if !rs_checkclearop(oap) {
        {
            let oap_typed = oap.cast::<OpargT>();
            (*oap_typed).start.lnum = nvim_get_cursor_lnum();
            (*oap_typed).start.col = nvim_get_cursor_col();
            (*oap_typed).start.coladd = nvim_get_cursor_coladd();
        }
        nvim_oap_set_op_type(oap, op_type);
        rs_set_op_var(op_type);
    }
}

/// Internal helper: implement the command translation logic (x->dl, X->dh, etc.)
unsafe fn nv_optrans_impl(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    if !rs_checkclearopq(oap) {
        let count0 = nvim_cap_get_count0(cap);
        if count0 != 0 {
            stuffnumReadbuff(count0);
        }
        let cmdchar_raw = nvim_cap_get_cmdchar(cap);
        // cmdchar is always a positive ASCII byte for abbreviated commands (x,X,D,C,s,S,Y,&)
        #[allow(clippy::cast_sign_loss)]
        let cmdchar = (cmdchar_raw & 0xFF) as u8;
        if let Some(idx) = OPTRANS_STR.iter().position(|&c| c == cmdchar) {
            let replacement = OPTRANS_AR[idx];
            // SAFETY: replacement bytes are all valid ASCII; we add a NUL terminator.
            let mut buf = [0u8; 8]; // max length is 4 bytes ":s\r\0"
            let len = replacement.len();
            buf[..len].copy_from_slice(replacement);
            buf[len] = 0;
            stuffReadbuff(buf.as_ptr().cast::<c_char>());
        }
    }
    (*cap.cast::<CmdargT>()).opcount = 0;
}

/// Command handler for operator commands (d, c, y, >, <, !, =, gq, gw, g?, etc.).
///
/// Sets up the operator state; actual work is done by do_pending_operator().
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_operator(cap: CapHandle) {
    nv_operator_impl(cap);
}

/// Command handler for abbreviated commands (x, X, D, C, s, S, Y, &).
///
/// Translates these commands to their full equivalents.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_optrans(cap: CapHandle) {
    nv_optrans_impl(cap);
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
    let oap = nvim_cap_get_oap(cap);
    if !nvim_get_p_to() && !VIsual_active && nvim_oap_get_op_type_ptr(oap) != OP_TILDE {
        if nvim_replace_check_prompt() != 0 {
            rs_clearopbeep(oap);
            return;
        }
        rs_n_swapchar(cap);
    } else {
        rs_nv_operator(cap);
    }
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
    let oap = nvim_cap_get_oap(cap);
    if nvim_bt_prompt_curbuf() && !prompt_curpos_editable() {
        rs_clearopbeep(oap);
        return;
    }
    if VIsual_active {
        // "vs" and "vS" are the same as "vc"
        let cmdchar = nvim_cap_get_cmdchar(cap);
        if cmdchar == c_int::from(b'S') {
            let vis_mode = nvim_get_VIsual_mode();
            VISUAL_MODE_ORIG.store(vis_mode, Ordering::Relaxed);
            nvim_set_VIsual_mode(c_int::from(b'V'));
        }
        (*cap.cast::<CmdargT>()).cmdchar = c_int::from(b'c');
        nv_operator_impl(cap);
    } else {
        nv_optrans_impl(cap);
    }
}

// =============================================================================
// Phase 4 (n_swapchar): Rust implementation of n_swapchar
// =============================================================================

extern "C" {
    // Phase 4: n_swapchar accessors
    fn nvim_swapchar_call(op_type: c_int, lnum: c_int, col: c_int) -> bool;
    fn u_savesub(lnum: c_int) -> bool;
    fn nvim_u_clearline_curbuf();
    fn nvim_changed_lines_call(lnum: c_int, col: c_int, lnum_end: c_int, do_concealed: bool);
    fn nvim_set_b_op_start(lnum: c_int, col: c_int, coladd: c_int);
    fn nvim_set_b_op_end_cursor();
    fn nvim_dec_b_op_end_col();
}

/// Swap case of character under cursor (implementation of "~" without tildeop).
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_n_swapchar(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);

    if rs_checkclearopq(oap) {
        return;
    }

    if *ml_get(nvim_get_cursor_lnum()) == 0 && vim_strchr(p_ww, c_int::from(b'~')).is_null() {
        rs_clearopbeep(oap);
        return;
    }

    rs_prep_redo_cmd(cap);

    if u_save_cursor() == 0 {
        return;
    }

    let start_lnum = nvim_get_cursor_lnum();
    let start_col = nvim_get_cursor_col();
    let start_coladd = nvim_get_cursor_coladd();

    let count1 = nvim_cap_get_count1(cap);
    let op_type = nvim_oap_get_op_type_ptr(oap);
    let mut did_change = false;
    let mut n = count1;
    loop {
        if n <= 0 {
            break;
        }
        let lnum = nvim_get_cursor_lnum();
        let col = nvim_get_cursor_col();
        if nvim_swapchar_call(op_type, lnum, col) {
            did_change = true;
        }
        nvim_inc_cursor();
        if utf_ptr2char(get_cursor_pos_ptr()) == NUL_CHAR {
            if !vim_strchr(p_ww, c_int::from(b'~')).is_null()
                && nvim_get_cursor_lnum() < nvim_get_line_count()
            {
                let new_lnum = nvim_get_cursor_lnum() + 1;
                nvim_set_cursor_lnum(new_lnum);
                nvim_set_cursor_col(0);
                if n > 1 {
                    if !u_savesub(nvim_get_cursor_lnum()) {
                        break;
                    }
                    nvim_u_clearline_curbuf();
                }
            } else {
                break;
            }
        }
        n -= 1;
    }

    nvim_check_cursor();
    nvim_curwin_set_curswant(true);
    if did_change {
        let end_lnum = nvim_get_cursor_lnum();
        nvim_changed_lines_call(start_lnum, start_col, end_lnum + 1, true);
        nvim_set_b_op_start(start_lnum, start_col, start_coladd);
        nvim_set_b_op_end_cursor();
        nvim_dec_b_op_end_col();
    }
}

// =============================================================================
// Phase 4: Text object handlers
// =============================================================================

extern "C" {
    // Phase 2: nv_object_impl accessors
    fn nvim_save_and_set_mps();
    fn nvim_restore_mps();
    fn current_tagblock(oap: OapHandle, count: c_int, include: bool) -> bool;
    fn nvim_current_quote_call(
        oap: OapHandle,
        count: c_int,
        include: bool,
        quotechar: c_int,
    ) -> bool;

    // Text object functions (now exported directly from textobject crate)
    #[link_name = "current_word"]
    fn rs_current_word(oap: OapHandle, count: c_int, include: bool, bigword: bool) -> c_int;
    #[link_name = "current_block"]
    fn rs_current_block(
        oap: OapHandle,
        count: c_int,
        include: bool,
        what: c_int,
        other: c_int,
    ) -> c_int;
    #[link_name = "current_par"]
    fn rs_current_par(oap: OapHandle, count: c_int, include: bool, par_type: c_int) -> c_int;
    #[link_name = "current_sent"]
    fn rs_current_sent(oap: OapHandle, count: c_int, include: bool) -> c_int;

    // cursor crate
    fn adjust_cursor_col();

    // nv_brackets_impl C accessors
    // Phase 2: new lower-level accessors replacing the bracket helpers (migrated to Rust)
    fn nvim_find_pattern_in_path_call(
        ptr: *mut c_char,
        len: usize,
        count0: c_int,
        nchar: c_int,
        count1: i64,
        from_rbracket: bool,
    );
    fn nvim_pos_to_mark_cursor() -> FmarkHandle;
    fn nvim_getnextmark_call(fm: FmarkHandle, dir: c_int, begin_line: c_int) -> FmarkHandle;
    fn do_mouse(oap: OapHandle, nchar: c_int, dir: c_int, count1: i64);
    fn nvim_spell_move_to_cap_call(dir: c_int, smt_type: c_int) -> usize;
    fn findmatchlimit(
        oap: OapHandle,
        initc: c_int,
        flags: c_int,
        maxtravel: i64,
    ) -> *mut crate::types::PosT;
    fn nvim_dec_cursor() -> c_int;
    // Phase 4: find_decl accessors
    fn nvim_searchit_decl(pat: *const c_char, patlen: usize, searchflags: c_int) -> c_int;
    fn nvim_findpar_decl() -> c_int;
    fn nvim_vim_iswordp_char(ptr: *const c_char) -> c_int;
    fn nvim_get_leader_len_cursor_line() -> c_int;
    fn nvim_cursor_line_is_blank() -> c_int;
    fn reset_search_dir();
    fn nvim_get_p_ws_bool() -> c_int;
    fn nvim_set_p_ws_bool(val: c_int);
    fn nvim_get_p_scs_bool() -> c_int;
    fn nvim_set_p_scs_bool(val: c_int);
}

// Phase 2 constants
const CA_NO_ADJ_OP_END_P2: c_int = 2;

// SMT (spell move type) constants from spell.h
const SMT_ALL: c_int = 0; // move to "all" words
const SMT_BAD: c_int = 1; // move to "bad" words only
const SMT_RARE: c_int = 2; // move to "rare" words only

/// Command handler for "a" or "i" text objects.
///
/// Handles text object selection when an operator is pending or in Visual mode.
/// Examples: "aw" (a word), "iw" (inner word), "a(" (a parentheses block), etc.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_object(cap: CapHandle) {
    let cmdchar = nvim_cap_get_cmdchar(cap);
    let nchar = nvim_cap_get_nchar(cap);
    let oap = nvim_cap_get_oap(cap);
    let count1 = nvim_cap_get_count1(cap);

    // "ix" = inner object: exclude white space
    // "ax" = an object: include white space
    let include = cmdchar != c_int::from(b'i');

    // Make sure (), [], {} and <> are in 'matchpairs'
    nvim_save_and_set_mps();

    let flag = match nchar {
        n if n == c_int::from(b'w') => rs_current_word(oap, count1, include, false) != 0,
        n if n == c_int::from(b'W') => rs_current_word(oap, count1, include, true) != 0,
        n if n == c_int::from(b'b') || n == c_int::from(b'(') || n == c_int::from(b')') => {
            rs_current_block(oap, count1, include, c_int::from(b'('), c_int::from(b')')) != 0
        }
        n if n == c_int::from(b'B') || n == c_int::from(b'{') || n == c_int::from(b'}') => {
            rs_current_block(oap, count1, include, c_int::from(b'{'), c_int::from(b'}')) != 0
        }
        n if n == c_int::from(b'[') || n == c_int::from(b']') => {
            rs_current_block(oap, count1, include, c_int::from(b'['), c_int::from(b']')) != 0
        }
        n if n == c_int::from(b'<') || n == c_int::from(b'>') => {
            rs_current_block(oap, count1, include, c_int::from(b'<'), c_int::from(b'>')) != 0
        }
        n if n == c_int::from(b't') => {
            // Do not adjust oap->end in do_pending_operator()
            (*cap.cast::<CmdargT>()).retval |= CA_NO_ADJ_OP_END_P2;
            current_tagblock(oap, count1, include)
        }
        n if n == c_int::from(b'p') => rs_current_par(oap, count1, include, c_int::from(b'p')) != 0,
        n if n == c_int::from(b's') => rs_current_sent(oap, count1, include) != 0,
        n if n == c_int::from(b'"') || n == c_int::from(b'\'') || n == c_int::from(b'`') => {
            nvim_current_quote_call(oap, count1, include, nchar)
        }
        _ => false,
    };

    nvim_restore_mps();

    if !flag {
        rs_clearopbeep(oap);
    }
    adjust_cursor_col();
    nvim_curwin_set_curswant(true);
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
    if VIsual_active {
        nvim_set_VIsual_select(true);
        nvim_set_VIsual_select_reg(0);
    } else if nvim_get_VIsual_reselect() != 0 {
        (*cap.cast::<CmdargT>()).nchar = c_int::from(b'v'); // fake "gv" command
        (*cap.cast::<CmdargT>()).arg = 1;
        rs_nv_g_cmd(cap);
    }
}

/// Helper: call findmatchlimit and return position as Option<(lnum, col, coladd)>.
unsafe fn findmatchlimit_pos(
    oap: OapHandle,
    findc: c_int,
    flags: c_int,
) -> Option<(c_int, c_int, c_int)> {
    let pos = findmatchlimit(oap, findc, flags, 0);
    if pos.is_null() {
        None
    } else {
        Some(((*pos).lnum, (*pos).col, (*pos).coladd))
    }
}

/// Implement `[{`, `[(`, `]}`, `])`, `[/`, `]*`, `[m`, `]m`, `[M`, `]M` bracket motions.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_bracket_block(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    let mut nchar = nvim_cap_get_nchar(cap);
    let cmdchar = nvim_cap_get_cmdchar(cap);
    let count1 = nvim_cap_get_count1(cap);

    if nchar == c_int::from(b'*') {
        nchar = c_int::from(b'/');
    }

    let old_pos = (
        nvim_get_cursor_lnum(),
        nvim_get_cursor_col(),
        nvim_get_cursor_coladd(),
    );
    let is_method = nchar == c_int::from(b'm') || nchar == c_int::from(b'M');
    let findc = if is_method {
        if cmdchar == c_int::from(b'[') {
            c_int::from(b'{')
        } else {
            c_int::from(b'}')
        }
    } else {
        nchar
    };
    let n_init = if is_method { 9999 } else { count1 };
    let flags = if cmdchar == c_int::from(b'[') {
        FM_BACKWARD
    } else {
        FM_FORWARD
    };

    let (new_pos, prev_pos) = bracket_find_loop(oap, findc, flags, n_init, is_method);
    nvim_set_cursor_pos(old_pos.0, old_pos.1, old_pos.2);

    let final_pos = if is_method {
        bracket_method_nav(oap, findc, flags, nchar, count1, old_pos, new_pos, prev_pos)
    } else {
        new_pos
    };

    if let Some((lnum, col, coladd)) = final_pos {
        nvim_setpcmark();
        nvim_set_cursor_pos(lnum, col, coladd);
        nvim_curwin_set_curswant(true);
        if (nvim_get_fdo_flags() & K_OPT_FDO_FLAG_BLOCK) != 0
            && nvim_get_KeyTyped()
            && nvim_oap_get_op_type_ptr(oap) == OP_NOP
        {
            rs_foldOpenCursor();
        }
    }
}

type BracketPos = (c_int, c_int, c_int);

/// First loop: find bracket match up to n_init times.
unsafe fn bracket_find_loop(
    oap: OapHandle,
    findc: c_int,
    flags: c_int,
    n_init: c_int,
    is_method: bool,
) -> (Option<BracketPos>, Option<BracketPos>) {
    let mut new_pos: Option<BracketPos> = None;
    let mut prev_pos: Option<BracketPos> = None;
    let mut n = n_init;
    while n > 0 {
        match findmatchlimit_pos(oap, findc, flags) {
            None => {
                if new_pos.is_none() && !is_method {
                    rs_clearopbeep(oap);
                }
                break;
            }
            Some(found) => {
                prev_pos = new_pos;
                nvim_set_cursor_pos(found.0, found.1, found.2);
                new_pos = Some(found);
            }
        }
        n -= 1;
    }
    (new_pos, prev_pos)
}

/// Method navigation loop for [m, ]m, [M, ]M.
#[allow(clippy::too_many_arguments)]
unsafe fn bracket_method_nav(
    oap: OapHandle,
    findc: c_int,
    flags: c_int,
    nchar: c_int,
    count1: c_int,
    old_pos: BracketPos,
    mut new_pos: Option<BracketPos>,
    prev_pos: Option<BracketPos>,
) -> Option<BracketPos> {
    let norm = (findc == c_int::from(b'{')) == (nchar == c_int::from(b'm'));
    let mut n = prev_pos.map_or(count1, |pp| {
        nvim_set_cursor_pos(pp.0, pp.1, pp.2);
        if norm {
            count1 - 1
        } else {
            count1
        }
    });
    let mut pos: Option<BracketPos> = prev_pos;

    'outer: while n > 0 {
        loop {
            let step = if findc == c_int::from(b'{') {
                nvim_dec_cursor()
            } else {
                nvim_inc_cursor()
            };
            if step < 0 {
                if pos.is_none() {
                    rs_clearopbeep(oap);
                }
                break 'outer;
            }
            let c = utf_ptr2char(get_cursor_pos_ptr());
            if c == c_int::from(b'{') || c == c_int::from(b'}') {
                let cur = (
                    nvim_get_cursor_lnum(),
                    nvim_get_cursor_col(),
                    nvim_get_cursor_coladd(),
                );
                if (c == findc && norm) || (n == 1 && !norm) {
                    new_pos = Some(cur);
                    pos = new_pos;
                    break 'outer;
                } else if new_pos.is_none() {
                    new_pos = Some(cur);
                    pos = new_pos;
                } else {
                    pos = findmatchlimit_pos(oap, findc, flags);
                    if let Some(found) = pos {
                        nvim_set_cursor_pos(found.0, found.1, found.2);
                    } else {
                        break 'outer;
                    }
                }
                break;
            }
        }
        n -= 1;
    }
    nvim_set_cursor_pos(old_pos.0, old_pos.1, old_pos.2);
    if pos.is_none() && new_pos.is_some() {
        rs_clearopbeep(oap);
    }
    pos
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
#[allow(clippy::too_many_lines)]
#[allow(clippy::missing_panics_doc)]
pub unsafe extern "C" fn rs_nv_brackets(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    nvim_oap_set_motion_type(oap, K_MT_CHARWISE);
    nvim_oap_set_inclusive(oap, false);
    nvim_set_cursor_coladd(0);

    let nchar = nvim_cap_get_nchar(cap);
    let cmdchar = nvim_cap_get_cmdchar(cap);

    if nchar == b'f' as c_int {
        // "[f" or "]f": Edit file under cursor (same as "gf")
        rs_nv_gotofile(cap);
    } else if nvim_vim_strchr_str(c"iI\x09dD\x04".as_ptr(), nchar) {
        // "[i", "[I", "[TAB", "[d", "[D", "[CTRL-D": find occurrence(s) of identifier/define
        // (inlined nvim_bracket_find_ident)
        let mut ptr: *mut c_char = core::ptr::null_mut();
        let len = rs_find_ident_at_pos(
            nvim_get_curwin(),
            nvim_get_cursor_lnum(),
            nvim_get_cursor_col(),
            &raw mut ptr,
            core::ptr::null_mut(),
            FIND_IDENT,
        );
        if len == 0 {
            rs_clearop(oap);
        } else {
            let count0 = nvim_cap_get_count0(cap);
            let count1 = nvim_cap_get_count1(cap);
            let from_rbracket = cmdchar == c_int::from(b']');
            // xmemdupz is called inside the C wrapper since find_pattern_in_path takes ownership
            nvim_find_pattern_in_path_call(ptr, len, count0, nchar, count1 as i64, from_rbracket);
            nvim_curwin_set_curswant(true);
        }
    } else if (cmdchar == b'[' as c_int && nvim_vim_strchr_str(c"{(*/#mM".as_ptr(), nchar))
        || (cmdchar == b']' as c_int && nvim_vim_strchr_str(c"})*/#mM".as_ptr(), nchar))
    {
        // "[{", "[(", "]}" or "])": bracket/method matching
        rs_nv_bracket_block(cap);
    } else if nchar == b'[' as c_int || nchar == b']' as c_int {
        // "[[", "[]", "]]" and "][": move to start or end of function
        // (inlined nvim_bracket_findpar)
        let flag = if nchar == cmdchar {
            b'{' as c_int
        } else {
            b'}' as c_int
        };
        let arg = nvim_cap_get_arg(cap);
        let count1 = nvim_cap_get_count1(cap);
        let op_type = nvim_oap_get_op_type_ptr(oap);
        nvim_curwin_set_curswant(true);
        let mut pincl = nvim_oap_get_inclusive(oap);
        let found = findpar(
            &raw mut pincl,
            arg,
            count1,
            flag,
            op_type != OP_NOP && arg == FORWARD && flag == c_int::from(b'{'),
        );
        nvim_oap_set_inclusive(oap, pincl);
        if found {
            if op_type == OP_NOP {
                nvim_beginline(BL_WHITE | BL_FIX);
            }
            if (nvim_get_fdo_flags() & K_OPT_FDO_FLAG_BLOCK) != 0
                && nvim_get_KeyTyped()
                && op_type == OP_NOP
            {
                rs_foldOpenCursor();
            }
        } else {
            rs_clearopbeep(oap);
        }
    } else if nchar == b'p' as c_int || nchar == b'P' as c_int {
        // "[p", "[P", "]P" and "]p": put with indent adjustment
        nv_put_opt_impl(cap, true);
    } else if nchar == b'\'' as c_int || nchar == b'`' as c_int {
        // "['", "[`", "]'" and "]`": jump to next mark
        // (inlined nvim_bracket_mark_jump)
        let count1 = nvim_cap_get_count1(cap);
        let mut fm = nvim_pos_to_mark_cursor();
        assert!(!fm.is_null());
        let dir = if cmdchar == c_int::from(b'[') {
            BACKWARD
        } else {
            FORWARD
        };
        let begin_line = c_int::from(nchar == c_int::from(b'\''));
        let mut prev_fm = fm;
        for _ in 0..count1 {
            prev_fm = fm;
            let next = nvim_getnextmark_call(fm, dir, begin_line);
            if next.is_null() {
                break;
            }
            fm = next;
        }
        if fm.is_null() {
            fm = prev_fm;
        }
        let mut flags = K_MARK_CONTEXT;
        if nchar == c_int::from(b'\'') {
            flags |= K_MARK_BEGIN_LINE;
        }
        rs_nv_mark_move_to(cap, flags, fm);
    } else if (K_RIGHTRELEASE..=K_LEFTMOUSE).contains(&nchar) {
        // Mouse click: put selected text with indent adjustment
        // (inlined nvim_bracket_do_mouse)
        let dir = if cmdchar == c_int::from(b']') {
            FORWARD
        } else {
            BACKWARD
        };
        do_mouse(oap, nchar, dir, nvim_cap_get_count1(cap) as i64);
    } else if nchar == b'z' as c_int {
        // "[z" and "]z": move to start or end of open fold
        let dir = if cmdchar == c_int::from(b']') {
            FORWARD
        } else {
            BACKWARD
        };
        if rs_foldMoveTo(false, dir, nvim_cap_get_count1(cap)) == 0 {
            rs_clearopbeep(oap);
        }
    } else if nchar == b'c' as c_int {
        // "[c" and "]c": move to next or previous diff-change
        let dir = if cmdchar == c_int::from(b']') {
            FORWARD
        } else {
            BACKWARD
        };
        if rs_diff_move_to(dir, nvim_cap_get_count1(cap)) == 0 {
            rs_clearopbeep(oap);
        }
    } else if nchar == b'r' as c_int || nchar == b's' as c_int || nchar == b'S' as c_int {
        // "[r", "[s", "[S", "]r", "]s" and "]S": move to next spell error
        // (inlined nvim_bracket_spell_move)
        nvim_setpcmark();
        let dir = if cmdchar == c_int::from(b']') {
            FORWARD
        } else {
            BACKWARD
        };
        let smt_type = if nchar == c_int::from(b's') {
            SMT_ALL
        } else if nchar == c_int::from(b'r') {
            SMT_RARE
        } else {
            SMT_BAD
        };
        let count1 = nvim_cap_get_count1(cap);
        let op_type = nvim_oap_get_op_type_ptr(oap);
        'spell: for _ in 0..count1 {
            if nvim_spell_move_to_cap_call(dir, smt_type) == 0 {
                rs_clearopbeep(oap);
                break 'spell;
            }
            nvim_curwin_set_curswant(true);
        }
        if op_type == OP_NOP
            && (nvim_get_fdo_flags() & K_OPT_FDO_FLAG_SEARCH) != 0
            && nvim_get_KeyTyped()
        {
            rs_foldOpenCursor();
        }
    } else {
        // Not a valid cap->nchar
        rs_clearopbeep(oap);
    }
}

// =============================================================================
// Phase 4 (nv_gd): find_decl migration
// =============================================================================

/// Constants for find_decl (from search.h)
const SEARCH_START: c_int = 0x100; // start search without col offset

/// Build the `\V`-escaped search pattern for `find_decl`.
///
/// Returns a NUL-terminated `Vec<u8>` and the pattern length (excluding NUL).
///
/// # Safety
/// `ptr` must point to at least `len` valid bytes.
unsafe fn find_decl_build_pat(ptr: *mut c_char, len: usize) -> (Vec<u8>, usize) {
    let is_word = nvim_vim_iswordp_char(ptr) != 0;
    let pat: Vec<u8> = if is_word {
        let mut p = b"\\V\\<".to_vec();
        p.extend_from_slice(std::slice::from_raw_parts(ptr.cast::<u8>(), len));
        p.extend_from_slice(b"\\>");
        p.push(0);
        p
    } else {
        let mut p = b"\\V".to_vec();
        p.extend_from_slice(std::slice::from_raw_parts(ptr.cast::<u8>(), len));
        p.push(0);
        p
    };
    let patlen = pat.len() - 1;
    (pat, patlen)
}

/// Inner search loop for `rs_find_decl`.
///
/// Returns `true` if a match was found (cursor is at match position).
///
/// # Safety
/// Caller must ensure cursor state is valid.
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
unsafe fn find_decl_search(
    pat: &[u8],
    patlen: usize,
    old_lnum: c_int,
    par_lnum: c_int,
    locally: bool,
    thisblock: bool,
    flags_arg: c_int,
) -> bool {
    let mut found_lnum: c_int = 0;
    let mut found_col: c_int = 0;
    let mut found_coladd: c_int = 0;
    let mut searchflags = flags_arg;
    let mut t: bool;
    loop {
        t = nvim_searchit_decl(pat.as_ptr().cast(), patlen, searchflags) != 0;
        if nvim_get_cursor_lnum() >= old_lnum {
            t = false; // match after start is failure too
        }
        if thisblock && t {
            let maxtravel = i64::from(old_lnum - nvim_get_cursor_lnum() + 1);
            {
                const FM_FORWARD: c_int = 0x02;
                let blk_pos = findmatchlimit(
                    std::ptr::null_mut(),
                    c_int::from(b'}'),
                    FM_FORWARD,
                    maxtravel,
                );
                if blk_pos.is_null() || (*blk_pos).lnum >= old_lnum {
                    // no match in block scope, continue normal search
                } else {
                    nvim_set_cursor_pos((*blk_pos).lnum, (*blk_pos).col, (*blk_pos).coladd);
                    continue;
                }
            }
        }
        if !t {
            if found_lnum != 0 {
                nvim_set_cursor_pos(found_lnum, found_col, found_coladd);
                t = true;
            }
            break;
        }
        if nvim_get_leader_len_cursor_line() > 0 {
            // Ignore comment lines — continue at start of next line.
            nvim_set_cursor_pos(nvim_get_cursor_lnum() + 1, 0, 0);
            continue;
        }
        let cur_lnum = nvim_get_cursor_lnum();
        let cur_col = nvim_get_cursor_col();
        let valid = rs_is_ident(nvim_ident_get_cursor_line_ptr(), cur_col);
        if !valid && found_lnum != 0 {
            nvim_set_cursor_pos(found_lnum, found_col, found_coladd);
            break;
        }
        // global search: use first match found
        if valid && !locally {
            break;
        }
        if valid && cur_lnum >= par_lnum {
            if found_lnum != 0 {
                nvim_set_cursor_pos(found_lnum, found_col, found_coladd);
            }
            break;
        }
        if valid {
            found_lnum = cur_lnum;
            found_col = cur_col;
            found_coladd = nvim_get_cursor_coladd();
        } else {
            found_lnum = 0;
            found_col = 0;
            found_coladd = 0;
        }
        searchflags &= !SEARCH_START;
    }
    t
}

/// Search for variable declaration of `ptr[len]`.
///
/// When `locally` is true, searches within the current function ("gd");
/// otherwise searches in the current file ("gD").
///
/// Returns true on success, false when not found.
///
/// # Safety
/// `ptr` must be a valid pointer to at least `len` bytes.
#[export_name = "find_decl"]
pub unsafe extern "C" fn rs_find_decl(
    ptr: *mut c_char,
    len: usize,
    locally: bool,
    thisblock: bool,
    flags_arg: c_int,
) -> bool {
    let (pat, patlen) = find_decl_build_pat(ptr, len);

    let old_lnum = nvim_get_cursor_lnum();
    let old_col = nvim_get_cursor_col();
    let old_coladd = nvim_get_cursor_coladd();

    let save_p_ws = nvim_get_p_ws_bool();
    let save_p_scs = nvim_get_p_scs_bool();
    nvim_set_p_ws_bool(0);
    nvim_set_p_scs_bool(0);

    // Position cursor at start of search range.
    let par_lnum: c_int;
    if !locally || nvim_findpar_decl() == 0 {
        nvim_setpcmark();
        nvim_set_cursor_pos(1, 0, 0);
        par_lnum = 1;
    } else {
        par_lnum = nvim_get_cursor_lnum();
        while nvim_get_cursor_lnum() > 1 && nvim_cursor_line_is_blank() == 0 {
            nvim_set_cursor_lnum(nvim_get_cursor_lnum() - 1);
        }
    }
    nvim_set_cursor_col(0);

    let found = find_decl_search(
        &pat, patlen, old_lnum, par_lnum, locally, thisblock, flags_arg,
    );

    if found {
        nvim_curwin_set_curswant(true);
        reset_search_dir();
    } else {
        nvim_set_cursor_pos(old_lnum, old_col, old_coladd);
    }

    nvim_set_p_ws_bool(save_p_ws);
    nvim_set_p_scs_bool(save_p_scs);

    found
}

// =============================================================================
// Phase 5: Undo/Redo handlers (now real Rust implementations)
// =============================================================================

extern "C" {
    fn start_redo(count: c_int, restart: bool) -> bool;
    fn u_redo(count: c_int);
    fn u_undoline();
    fn nvim_get_arrow_used() -> c_int; // defined in edit.c, returns int
}

// OP_ constants for undo/redo
const OP_LOWER: c_int = 12;
const OP_UPPER: c_int = 11;

/// Command handler for "u" undo command.
///
/// In Visual mode or when `op_type` is `OP_LOWER`, translates to `gu` command.
/// Otherwise performs undo via `nv_kundo`.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_undo(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    if nvim_oap_get_op_type_ptr(oap) == OP_LOWER || VIsual_active {
        // translate "<Visual>u" to "<Visual>gu" and "guu" to "gugu"
        (*cap.cast::<CmdargT>()).cmdchar = c_int::from(b'g');
        (*cap.cast::<CmdargT>()).nchar = c_int::from(b'u');
        rs_nv_operator(cap);
    } else {
        rs_nv_kundo(cap);
    }
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
    let oap = nvim_cap_get_oap(cap);
    if nvim_oap_get_op_type_ptr(oap) == OP_UPPER || VIsual_active {
        // translate "gUU" to "gUgU"
        (*cap.cast::<CmdargT>()).cmdchar = c_int::from(b'g');
        (*cap.cast::<CmdargT>()).nchar = c_int::from(b'U');
        rs_nv_operator(cap);
        return;
    }

    if rs_checkclearopq(oap) {
        return;
    }

    u_undoline();
    nvim_curwin_set_curswant(true);
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
    let oap = nvim_cap_get_oap(cap);
    if rs_checkclearopq(oap) {
        return;
    }

    // If restart_edit is non-zero, the last but one command is repeated
    // instead of the last command (inserting text). Used for CTRL-O <.>.
    let arrow_used = nvim_get_arrow_used() != 0;
    let count0 = nvim_cap_get_count0(cap);
    if !start_redo(count0, restart_edit != 0 && !arrow_used) {
        rs_clearopbeep(oap);
    }
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
    if nvim_get_VIsual_select() && VIsual_active {
        // Get register name
        nvim_inc_no_mapping();
        let mut reg = nvim_plain_vgetc_wrapper();
        reg = nvim_langmap_adjust(reg, true);
        nvim_dec_no_mapping();

        if reg == c_int::from(b'"') {
            // the unnamed register is 0
            reg = 0;
        }

        let valid = nvim_valid_yank_reg(reg, true);
        nvim_set_VIsual_select_reg(if valid { reg } else { 0 });
        return;
    }

    let oap = nvim_cap_get_oap(cap);
    if rs_checkclearopq(oap) {
        return;
    }

    let count1 = nvim_cap_get_count1(cap);
    u_redo(count1);
    nvim_curwin_set_curswant(true);
}

// =============================================================================
// Phase 6: Insert mode entry handlers
// =============================================================================

extern "C" {
    // Phase 3: nv_Replace / nv_vreplace accessors
    fn nvim_curbuf_modifiable() -> bool;
    fn nvim_emsg_modifiable();
    fn nvim_coladvance_getviscol();
    fn nvim_invoke_edit_R(cap: CapHandle, repl: bool, cmd: c_int);
    fn nvim_get_literal_call(no_simplify: bool) -> c_int;
    fn nvim_stuffcharReadbuff(c: c_int);

    // nv_replace C wrappers (lower-level after Phase 1 inlining)
    fn nvim_replace_check_prompt() -> c_int;
    fn u_save_cursor() -> c_int;
    fn rs_foldUpdateAfterInsert();
}

// nv_replace constants
const REPLACE_CR_NCHAR: c_int = -1;
const REPLACE_NL_NCHAR: c_int = -2;
const DEL_CHAR: c_int = 127; // DEL character value

/// Command handler for "r" single-character replace.
///
/// Replaces character(s) under the cursor with the typed character.
/// In Visual mode, delegates to the operator system.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_nv_replace(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);

    if rs_checkclearop(oap) {
        return;
    }
    if nvim_replace_check_prompt() != 0 {
        rs_clearopbeep(oap);
        return;
    }

    // Inlined nvim_replace_get_literal: handle Ctrl-V/Ctrl-Q literal input
    let had_ctrl_v = {
        let nch = nvim_cap_get_nchar(cap);
        if nch == CTRL_V || nch == CTRL_Q_P3 {
            let new_nchar = nvim_get_literal_call(false);
            (*cap.cast::<CmdargT>()).nchar = new_nchar;
            if new_nchar > DEL_CHAR {
                NUL_CHAR
            } else {
                CTRL_V
            }
        } else {
            NUL_CHAR
        }
    };

    // Abort if the character is a special key
    let nchar = nvim_cap_get_nchar(cap);
    if nchar < 0 {
        rs_clearopbeep(oap);
        return;
    }

    // Visual mode "r"
    if VIsual_active {
        if unsafe { got_int } {
            unsafe {
                got_int = false;
            }
        }
        if had_ctrl_v != 0 {
            let nchar = nvim_cap_get_nchar(cap);
            if nchar == CAR_CHAR {
                (*cap.cast::<CmdargT>()).nchar = REPLACE_CR_NCHAR;
            } else if nchar == NL_CHAR {
                (*cap.cast::<CmdargT>()).nchar = REPLACE_NL_NCHAR;
            }
        }
        rs_nv_operator(cap);
        return;
    }

    // Inlined nvim_replace_virtual_edit: break tabs in virtual edit mode
    if virtual_active(nvim_get_curwin()) {
        if u_save_cursor() == 0 {
            return;
        }
        let gc = utf_ptr2char(get_cursor_pos_ptr());
        let count1 = nvim_cap_get_count1(cap);
        if gc == NUL_CHAR {
            let viscol = nvim_getviscol();
            coladvance_force(viscol + count1);
            let new_col = nvim_get_cursor_col() - count1;
            nvim_set_cursor_col(new_col);
        } else if gc == TAB_CHAR {
            let viscol = nvim_getviscol();
            coladvance_force(viscol);
        }
    }

    // Inlined nvim_replace_check_length: abort if not enough chars to replace
    let count1 = nvim_cap_get_count1(cap);
    if get_cursor_pos_len() < count1 || nvim_mb_charlen_cursor() < count1 {
        rs_clearopbeep(oap);
        return;
    }

    // Inlined nvim_replace_tab_expand: TAB with expandtab/smarttab via edit()
    let nchar = nvim_cap_get_nchar(cap);
    if had_ctrl_v != CTRL_V && nchar == TAB_CHAR && (nvim_curbuf_b_p_et() || p_sta != 0) {
        stuffnumReadbuff(count1);
        nvim_stuffcharReadbuff(c_int::from(b'R'));
        nvim_stuffcharReadbuff(TAB_CHAR);
        nvim_stuffcharReadbuff(ESC_CHAR);
        return;
    }

    // Save line for undo
    if u_save_cursor() == 0 {
        return;
    }

    let nchar = nvim_cap_get_nchar(cap);
    if had_ctrl_v != CTRL_V && (nchar == c_int::from(b'\r') || nchar == c_int::from(b'\n')) {
        // Inlined nvim_replace_newline: replace char(s) by single newline
        del_chars(count1, false);
        nvim_stuffcharReadbuff(c_int::from(b'\r'));
        nvim_stuffcharReadbuff(ESC_CHAR);
        invoke_edit_impl(cap, true, c_int::from(b'r'), false);
    } else {
        // Replace with typed character(s)
        let regname = nvim_oap_get_regname_ptr(oap);
        rs_prep_redo(
            regname,
            count1,
            NUL_CHAR,
            c_int::from(b'r'),
            NUL_CHAR,
            had_ctrl_v,
            0,
        );

        // Inlined nvim_replace_chars: character replacement loop
        nvim_set_b_op_start_cursor();
        let old_state = State;

        let nchar_len = nvim_cap_get_nchar_len(cap);
        if nchar_len > 0 {
            nvim_AppendToRedobuff_composing(cap);
        } else {
            AppendCharToRedobuff(nchar);
        }

        for _ in 0..count1 {
            State = MODE_REPLACE;
            let ctrl_e = c_int::from(b'\x05'); // Ctrl-E
            let ctrl_y = c_int::from(b'\x19'); // Ctrl-Y
            if nchar == ctrl_e || nchar == ctrl_y {
                let lnum = nvim_get_cursor_lnum() + if nchar == ctrl_y { -1 } else { 1 };
                let c = ins_copychar(lnum);
                if c == NUL_CHAR {
                    nvim_set_cursor_col(nvim_get_cursor_col() + 1);
                } else {
                    nvim_ins_char_call(c);
                }
            } else if nchar_len > 0 {
                nvim_ins_char_bytes_from_cap(cap);
            } else {
                nvim_ins_char_call(nchar);
            }
            State = old_state;
        }
        nvim_set_cursor_col(nvim_get_cursor_col() - 1);
        nvim_mb_adjust_cursor();
        nvim_set_b_op_end_cursor();
        nvim_curwin_set_curswant(true);
        set_last_insert(nchar);
    }

    rs_foldUpdateAfterInsert();
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
    if VIsual_active {
        // "R" is replace lines in Visual mode
        (*cap.cast::<CmdargT>()).cmdchar = c_int::from(b'c');
        (*cap.cast::<CmdargT>()).nchar = NUL_CHAR;
        let vis_mode = nvim_get_VIsual_mode();
        VISUAL_MODE_ORIG.store(vis_mode, Ordering::Relaxed);
        nvim_set_VIsual_mode(c_int::from(b'V'));
        rs_nv_operator(cap);
        return;
    }

    let oap = nvim_cap_get_oap(cap);
    if rs_checkclearopq(oap) {
        return;
    }

    if nvim_curbuf_modifiable() {
        if virtual_active(nvim_get_curwin()) {
            nvim_coladvance_getviscol();
        }
        let arg = nvim_cap_get_arg(cap);
        let cmd = if arg != 0 {
            c_int::from(b'V')
        } else {
            c_int::from(b'R')
        };
        nvim_invoke_edit_R(cap, false, cmd);
    } else {
        nvim_emsg_modifiable();
    }
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
    if VIsual_active {
        // In Visual mode: do same as "r" for now
        let extra_char = (*cap.cast::<CmdargT>()).extra_char;
        (*cap.cast::<CmdargT>()).cmdchar = c_int::from(b'r');
        (*cap.cast::<CmdargT>()).nchar = extra_char;
        rs_nv_replace(cap);
        return;
    }

    let oap = nvim_cap_get_oap(cap);
    if rs_checkclearopq(oap) {
        return;
    }

    if nvim_curbuf_modifiable() {
        let mut extra_char = (*cap.cast::<CmdargT>()).extra_char;
        if extra_char == CTRL_V || extra_char == CTRL_Q_P3 {
            // get another character
            extra_char = nvim_get_literal_call(false);
        }
        if extra_char < c_int::from(b' ') {
            // Prefix a control character with CTRL-V to avoid it being used as a command.
            nvim_stuffcharReadbuff(CTRL_V);
        }
        nvim_stuffcharReadbuff(extra_char);
        nvim_stuffcharReadbuff(ESC_CHAR);
        if virtual_active(nvim_get_curwin()) {
            nvim_coladvance_getviscol();
        }
        nvim_invoke_edit_R(cap, true, c_int::from(b'v'));
    } else {
        nvim_emsg_modifiable();
    }
}

// =============================================================================
// Phase 7: Scroll and screen handlers
// =============================================================================

extern "C" {
    // Phase 1: nv_right_impl / nv_left_impl accessors
    fn get_cursor_pos_ptr() -> *const c_char;
    #[allow(dead_code)]
    fn nvim_utfc_ptr2len_cursor() -> c_int;
    fn oneleft() -> c_int;
}

extern "C" {
    // Phase 1: nv_scroll_impl accessors (window-parameterized)
    fn nvim_validate_botline(wp: WinHandle);
    fn nvim_cursor_correct(wp: WinHandle);
    fn nvim_win_get_botline(wp: WinHandle) -> c_int;
    fn nvim_win_get_view_height(wp: WinHandle) -> c_int;
    fn nvim_win_get_empty_rows(wp: WinHandle) -> c_int;
    fn nvim_win_get_fill(wp: WinHandle, lnum: c_int) -> c_int;
    fn nvim_plines_win(wp: WinHandle, lnum: c_int, limit: c_int) -> c_int;
    fn nvim_win_lines_concealed(wp: WinHandle) -> c_int;
    fn nvim_decor_conceal_line(wp: WinHandle, row: c_int, check_cursor: c_int) -> c_int;
    fn nvim_hasFolding(wp: WinHandle, lnum: c_int, firstp: *mut c_int, lastp: *mut c_int) -> c_int;
    fn nvim_win_get_cursor_lnum(wp: WinHandle) -> c_int;
    fn nvim_win_set_cursor_lnum(wp: WinHandle, lnum: c_int);
    fn nvim_buf_get_line_count(buf: BufHandle) -> c_int;
    fn nvim_get_curbuf() -> BufHandle;

    // Phase 3: nv_up / nv_down accessors
    fn nvim_bt_quickfix_curbuf() -> c_int; // defined in window_shim.c, returns int
    fn rs_qf_view_result(split: bool);
    fn prompt_invoke_callback();
    // (nvim_nv_scroll_impl removed: nv_scroll_impl migrated to Rust)

    // z-command C accessors
    fn nvim_get_curwin_w_p_fdl() -> c_int;
    fn nvim_set_curwin_w_p_fdl(val: c_int);
    fn nvim_set_curwin_w_foldinvalid(val: bool);
    fn nvim_get_curwin_w_view_width() -> c_int;
    fn nvim_get_curwin_w_leftcol() -> c_int;
    fn nvim_set_curwin_w_leftcol(val: c_int);
    fn nvim_get_curwin_w_botline() -> c_int;
    fn set_leftcol(col: c_int);
    fn nvim_hasFolding_curwin(lnum: c_int) -> bool;
    fn nvim_getvcol_curwin_cursor(vcol: *mut c_int);
    fn nvim_getvcol_curwin_cursor_end(vcol: *mut c_int);
    fn spell_suggest(count: c_int);
    fn nvim_get_curwin_w_p_wrap() -> bool;
    fn nvim_sync_fen_in_diff_windows();
    fn nvim_spell_move_to_wrapper(dir: c_int) -> usize;
    fn nvim_ml_get_pos_cursor() -> *mut c_char;
    fn nvim_inc_emsg_off();
    fn nvim_dec_emsg_off();
    fn spell_add_word(word: *mut c_char, len: c_int, what: c_int, idx: c_int, undo: bool);
    fn nvim_plain_vgetc_wrapper() -> c_int;
    fn nvim_langmap_adjust(c: c_int, condition: bool) -> c_int;
    fn nvim_add_to_showcmd_wrapper(c: c_int) -> bool;
    fn nvim_inc_no_mapping();
    fn nvim_dec_no_mapping();
    fn nvim_inc_allow_keys();
    fn nvim_dec_allow_keys();
    fn nvim_vim_strchr_str(s: *const c_char, c: c_int) -> bool;
    fn nvim_get_curwin() -> WinHandle;
    fn redraw_later(win: WinHandle, type_val: c_int);
    fn cursor_down_inner(win: WinHandle, n: c_int, skip_conceal: bool);
    fn cursor_up_inner(win: WinHandle, n: c_int, skip_conceal: bool);
    fn update_topline(win: WinHandle);
    fn scroll_cursor_bot(win: WinHandle, off: c_int, always: bool);
    fn scroll_cursor_top(win: WinHandle, off: c_int, always: bool);
    fn scroll_cursor_halfway(win: WinHandle, atend: bool, prefer_above: bool);
    fn validate_cheight(win: WinHandle);
    fn changed_window_setting(win: WinHandle);
    fn check_cursor_col(win: WinHandle);
    fn linetabsize(win: WinHandle, lnum: c_int) -> c_int;
    fn win_col_off(win: WinHandle) -> c_int;
    fn win_col_off2(win: WinHandle) -> c_int;
    fn sms_marker_overlap(win: WinHandle, width: c_int) -> c_int;
    fn win_cursorline_standout(win: WinHandle) -> bool;

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
    fn rs_diff_move_to(dir: c_int, count: c_int) -> c_int;
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
const UPD_CLEAR: c_int = 50;
const UPD_NOT_VALID: c_int = 40;

// Phase 2: spell constants (from spell_defs.h)
const OK: c_int = 1;
const SPELL_ADD_GOOD: c_int = 0;
const SPELL_ADD_BAD: c_int = 1;
const SMT_ALL_DIR: c_int = 1; // FORWARD direction for spell_move_to

/// Command handler for "z" commands.
///
/// Handle zg/zw/zG/zW/zug/zuw commands for adding/removing words to spell lists.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_zg_zw(cap: CapHandle, mut nchar: c_int) -> c_int {
    let undo = if nchar == c_int::from(b'u') {
        // Get the next character to determine which word list to affect.
        nvim_inc_no_mapping();
        nvim_inc_allow_keys();
        nchar = nvim_plain_vgetc_wrapper();
        nchar = nvim_langmap_adjust(nchar, true);
        nvim_dec_no_mapping();
        nvim_dec_allow_keys();
        nvim_add_to_showcmd_wrapper(nchar);

        // Must be one of g/G/w/W.
        let valid = nchar == c_int::from(b'g')
            || nchar == c_int::from(b'G')
            || nchar == c_int::from(b'w')
            || nchar == c_int::from(b'W');
        if !valid {
            let oap = nvim_cap_get_oap(cap);
            rs_clearopbeep(oap);
            return OK;
        }
        true
    } else {
        false
    };

    let oap = nvim_cap_get_oap(cap);
    if rs_checkclearop(oap) {
        return OK;
    }

    let mut ptr: *mut c_char = std::ptr::null_mut();
    let mut len: usize = 0;

    if VIsual_active && !rs_get_visual_text(cap, &raw mut ptr, &raw mut len) {
        return FAIL;
    }

    if ptr.is_null() {
        // Save cursor position.
        let saved_col = nvim_get_cursor_col();
        let saved_lnum = nvim_get_cursor_lnum();

        // Find bad word under the cursor. When 'spell' is off this fails
        // and find_ident_under_cursor() is used below.
        nvim_inc_emsg_off();
        len = nvim_spell_move_to_wrapper(SMT_ALL_DIR);
        nvim_dec_emsg_off();

        if len != 0 && nvim_get_cursor_col() <= saved_col {
            ptr = nvim_ml_get_pos_cursor();
        }
        // Restore cursor position.
        nvim_set_cursor_pos(saved_lnum, saved_col, 0);
    }

    if ptr.is_null() {
        len = rs_find_ident_at_pos(
            nvim_get_curwin(),
            nvim_get_cursor_lnum(),
            nvim_get_cursor_col(),
            &raw mut ptr,
            core::ptr::null_mut(),
            FIND_IDENT,
        );
        if len == 0 {
            return FAIL;
        }
    }

    let what = if nchar == c_int::from(b'w') || nchar == c_int::from(b'W') {
        SPELL_ADD_BAD
    } else {
        SPELL_ADD_GOOD
    };
    let idx = if nchar == c_int::from(b'G') || nchar == c_int::from(b'W') {
        0
    } else {
        nvim_cap_get_count1(cap)
    };
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    spell_add_word(ptr, len as c_int, what, idx, undo);

    OK
}

/// Get the count specified after a 'z' command.
///
/// Only 'z<CR>', 'zl', 'zh', 'z<Left>', and 'z<Right>' commands accept a
/// count after 'z'.
///
/// Returns `true` to process the 'z' command and `false` to skip it.
/// Updates `nchar_arg` in place.
///
/// Port of C `nv_z_get_count()`.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer; `nchar_arg` must be a valid pointer.
unsafe fn nv_z_get_count_impl(cap: CapHandle, nchar_arg: &mut c_int) -> bool {
    let oap = nvim_cap_get_oap(cap);
    if rs_checkclearop(oap) {
        return false;
    }
    let mut n = *nchar_arg - c_int::from(b'0');
    loop {
        nvim_inc_no_mapping();
        nvim_inc_allow_keys(); // no mapping for nchar, but allow key codes
        let nchar = nvim_plain_vgetc_wrapper();
        let nchar = nvim_langmap_adjust(nchar, true);
        nvim_dec_no_mapping();
        nvim_dec_allow_keys();
        nvim_add_to_showcmd_wrapper(nchar);

        if nchar == K_DEL || nchar == K_KDEL {
            n /= 10;
        } else if rs_ascii_isdigit(nchar) != 0 {
            if vim_append_digit_int(&raw mut n, nchar - c_int::from(b'0')) == FAIL {
                rs_clearopbeep(oap);
                break;
            }
        } else if nchar == CAR_CHAR {
            rs_win_setheight(n);
            break;
        } else if nchar == c_int::from(b'l')
            || nchar == c_int::from(b'h')
            || nchar == K_LEFT
            || nchar == K_RIGHT
        {
            let count1 = nvim_cap_get_count1(cap);
            nvim_cap_set_count1(cap, if n != 0 { n * count1 } else { count1 });
            *nchar_arg = nchar;
            return true;
        } else {
            rs_clearopbeep(oap);
            break;
        }
    }
    nvim_oap_set_op_type(oap, OP_NOP);
    false
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
    let old_fen = nvim_put_get_save_fen();
    let curwin = nvim_get_curwin();

    let siso = rs_get_sidescrolloff_value(curwin);

    if rs_ascii_isdigit(nchar) != 0 && !nv_z_get_count_impl(cap, &mut nchar) {
        return;
    }

    // "zf" and "zF" are always an operator, "zd", "zo", "zO", "zc"
    // and "zC" only in Visual mode.  "zj" and "zk" are motion commands.
    let cap_nchar = nvim_cap_get_nchar(cap);
    if cap_nchar != b'f' as c_int
        && cap_nchar != b'F' as c_int
        && !(VIsual_active && nvim_vim_strchr_str(c"dcCoO".as_ptr(), cap_nchar))
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
        check_cursor_col(nvim_get_curwin());
    }

    match nchar {
        // "z+", "z<CR>" and "zt": put cursor at top of screen
        n if n == b'+' as c_int => {
            if nvim_cap_get_count0(cap) == 0 {
                // No count given: put cursor at the line below screen
                nvim_validate_botline(nvim_get_curwin());
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
            scroll_cursor_top(nvim_get_curwin(), 0, true);
            redraw_later(nvim_get_curwin(), UPD_VALID);
            rs_set_fraction(curwin);
        }
        n if n == NL_CHAR || n == CAR_CHAR || n == K_KENTER => {
            // FALLTHROUGH to 't'
            nvim_beginline(BL_WHITE | BL_FIX);
            scroll_cursor_top(nvim_get_curwin(), 0, true);
            redraw_later(nvim_get_curwin(), UPD_VALID);
            rs_set_fraction(curwin);
        }
        n if n == b't' as c_int => {
            scroll_cursor_top(nvim_get_curwin(), 0, true);
            redraw_later(nvim_get_curwin(), UPD_VALID);
            rs_set_fraction(curwin);
        }

        // "z." and "zz": put cursor in middle of screen
        n if n == b'.' as c_int => {
            nvim_beginline(BL_WHITE | BL_FIX);
            // FALLTHROUGH to 'z'
            scroll_cursor_halfway(nvim_get_curwin(), true, false);
            redraw_later(nvim_get_curwin(), UPD_VALID);
            rs_set_fraction(curwin);
        }
        n if n == b'z' as c_int => {
            scroll_cursor_halfway(nvim_get_curwin(), true, false);
            redraw_later(nvim_get_curwin(), UPD_VALID);
            rs_set_fraction(curwin);
        }

        // "z^", "z-" and "zb": put cursor at bottom of screen
        n if n == b'^' as c_int => {
            if nvim_cap_get_count0(cap) != 0 {
                scroll_cursor_bot(nvim_get_curwin(), 0, true);
                nvim_set_cursor_lnum(nvim_win_get_topline(curwin));
            } else if nvim_win_get_topline(curwin) == 1 {
                nvim_set_cursor_lnum(1);
            } else {
                nvim_set_cursor_lnum(nvim_win_get_topline(curwin) - 1);
            }
            // FALLTHROUGH to '-' -> 'b'
            nvim_beginline(BL_WHITE | BL_FIX);
            scroll_cursor_bot(nvim_get_curwin(), 0, true);
            redraw_later(nvim_get_curwin(), UPD_VALID);
            rs_set_fraction(curwin);
        }
        n if n == b'-' as c_int => {
            nvim_beginline(BL_WHITE | BL_FIX);
            // FALLTHROUGH to 'b'
            scroll_cursor_bot(nvim_get_curwin(), 0, true);
            redraw_later(nvim_get_curwin(), UPD_VALID);
            rs_set_fraction(curwin);
        }
        n if n == b'b' as c_int => {
            scroll_cursor_bot(nvim_get_curwin(), 0, true);
            redraw_later(nvim_get_curwin(), UPD_VALID);
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
                    set_leftcol(0);
                } else {
                    set_leftcol(leftcol - count1);
                }
            }
        }

        // "zh" - scroll screen to the right
        n if n == b'h' as c_int || n == K_LEFT => {
            if !nvim_get_curwin_w_p_wrap() {
                let leftcol = nvim_get_curwin_w_leftcol();
                let count1 = nvim_cap_get_count1(cap);
                if count1 > leftcol {
                    set_leftcol(0);
                } else {
                    set_leftcol(leftcol - count1);
                }
            }
        }

        // "zL" - scroll window left half-page
        n if n == b'L' as c_int => {
            let half = nvim_get_curwin_w_view_width() / 2;
            nvim_cap_set_count1(cap, nvim_cap_get_count1(cap) * half);
            // FALLTHROUGH to 'l'
            if !nvim_get_curwin_w_p_wrap() {
                set_leftcol(nvim_get_curwin_w_leftcol() + nvim_cap_get_count1(cap));
            }
        }

        // "zl" - scroll window to the left if not wrapping
        n if n == b'l' as c_int || n == K_RIGHT => {
            if !nvim_get_curwin_w_p_wrap() {
                set_leftcol(nvim_get_curwin_w_leftcol() + nvim_cap_get_count1(cap));
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
                    redraw_later(nvim_get_curwin(), UPD_NOT_VALID);
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
                let n_val = nvim_get_curwin_w_view_width() - win_col_off(nvim_get_curwin());
                if col + siso < n_val {
                    col = 0;
                } else {
                    col = col + siso - n_val + 1;
                }
                if nvim_get_curwin_w_leftcol() != col {
                    nvim_set_curwin_w_leftcol(col);
                    redraw_later(nvim_get_curwin(), UPD_NOT_VALID);
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
                (*cap.cast::<CmdargT>()).nchar = b'f' as c_int;
                rs_nv_operator(cap);
                nvim_set_w_p_fen(true);

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
                if VIsual_active {
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
                changed_window_setting(nvim_get_curwin());
            } else if rs_foldmethodIsMarker(curwin) != 0 {
                let line_count = nvim_get_line_count();
                rs_deleteFold(curwin, 1, line_count, true, false);
            } else {
                nvim_emsg(nvim_get_e352_msg());
            }
        }

        // "zn": fold none: reset 'foldenable'
        n if n == b'n' as c_int => {
            nvim_set_w_p_fen(false);
        }

        // "zN": fold Normal: set 'foldenable'
        n if n == b'N' as c_int => {
            nvim_set_w_p_fen(true);
        }

        // "zi": invert folding: toggle 'foldenable'
        n if n == b'i' as c_int => {
            nvim_set_w_p_fen(!nvim_put_get_save_fen());
        }

        // "za": open closed fold or close open fold at cursor
        n if n == b'a' as c_int => {
            let cursor_lnum = nvim_get_cursor_lnum();
            if nvim_hasFolding_curwin(cursor_lnum) {
                rs_setFoldRepeat(cursor_lnum, nvim_cap_get_count1(cap), true);
            } else {
                rs_setFoldRepeat(cursor_lnum, nvim_cap_get_count1(cap), false);
                nvim_set_w_p_fen(true);
            }
        }

        // "zA": open fold at cursor recursively
        n if n == b'A' as c_int => {
            let cursor_lnum = nvim_get_cursor_lnum();
            if nvim_hasFolding_curwin(cursor_lnum) {
                rs_setManualFold(cursor_lnum, true, true, std::ptr::null_mut());
            } else {
                rs_setManualFold(cursor_lnum, false, true, std::ptr::null_mut());
                nvim_set_w_p_fen(true);
            }
        }

        // "zo": open fold at cursor or Visual area
        n if n == b'o' as c_int => {
            if VIsual_active {
                rs_nv_operator(cap);
            } else {
                rs_setFoldRepeat(nvim_get_cursor_lnum(), nvim_cap_get_count1(cap), true);
            }
        }

        // "zO": open fold recursively
        n if n == b'O' as c_int => {
            if VIsual_active {
                rs_nv_operator(cap);
            } else {
                rs_setManualFold(nvim_get_cursor_lnum(), true, true, std::ptr::null_mut());
            }
        }

        // "zc": close fold at cursor or Visual area
        n if n == b'c' as c_int => {
            if VIsual_active {
                rs_nv_operator(cap);
            } else {
                rs_setFoldRepeat(nvim_get_cursor_lnum(), nvim_cap_get_count1(cap), false);
            }
            nvim_set_w_p_fen(true);
        }

        // "zC": close fold recursively
        n if n == b'C' as c_int => {
            if VIsual_active {
                rs_nv_operator(cap);
            } else {
                rs_setManualFold(nvim_get_cursor_lnum(), false, true, std::ptr::null_mut());
            }
            nvim_set_w_p_fen(true);
        }

        // "zv": open folds at the cursor
        n if n == b'v' as c_int => {
            rs_foldOpenCursor();
        }

        // "zx": re-apply 'foldlevel' and open folds at the cursor
        n if n == b'x' as c_int => {
            nvim_set_w_p_fen(true);
            nvim_set_curwin_w_foldinvalid(true);
            rs_newFoldLevel();
            rs_foldOpenCursor();
        }

        // "zX": undo manual opens/closes, re-apply 'foldlevel'
        n if n == b'X' as c_int => {
            nvim_set_w_p_fen(true);
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
            nvim_set_w_p_fen(true);
        }

        // "zM": close all folds
        n if n == b'M' as c_int => {
            nvim_set_curwin_w_p_fdl(0);
            // Force update
            rs_newFoldLevel();
            nvim_set_w_p_fen(true);
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
            if rs_nv_zg_zw(cap, nchar) == FAIL {
                return;
            }
        }

        // "z=": suggestions for a badly spelled word
        n if n == b'=' as c_int => {
            if !rs_checkclearop(oap) {
                spell_suggest(nvim_cap_get_count0(cap));
            }
        }

        _ => {
            rs_clearopbeep(oap);
        }
    }

    // Redraw when 'foldenable' changed
    if old_fen != nvim_put_get_save_fen() {
        if rs_foldmethodIsDiff(curwin) != 0 && nvim_curwin_get_p_scb() {
            // Adjust 'foldenable' in diff-synced windows.
            nvim_sync_fen_in_diff_windows();
        }
        changed_window_setting(nvim_get_curwin());
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
    rs_nv_scroll_impl(cap);
}

/// Implementation of H/L/M scrolling commands.
///
/// H: Move cursor to top of window (with optional count).
/// M: Move cursor to middle of window.
/// L: Move cursor to bottom of window (with optional count).
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
unsafe fn rs_nv_scroll_impl(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    nvim_oap_set_motion_type(oap, K_MT_LINEWISE);
    nvim_setpcmark();

    let curwin = nvim_get_curwin();
    let curbuf = nvim_get_curbuf();
    let cmdchar = nvim_cap_get_cmdchar(cap);
    let count1 = nvim_cap_get_count1(cap);
    let line_count = nvim_buf_get_line_count(curbuf);

    if cmdchar == c_int::from(b'L') {
        nv_scroll_bottom(curwin, count1);
    } else if cmdchar == c_int::from(b'M') {
        let n = nv_scroll_middle(curwin, line_count);
        let new_lnum = (nvim_win_get_topline(curwin) + n).min(line_count);
        nvim_win_set_cursor_lnum(curwin, new_lnum);
    } else {
        // H: move to top of window
        let n = nv_scroll_top(curwin, count1);
        let topline = nvim_win_get_topline(curwin);
        let new_lnum = (topline + n).min(line_count);
        nvim_win_set_cursor_lnum(curwin, new_lnum);
    }

    // Correct for 'so', except when an operator is pending.
    if nvim_oap_get_op_type_ptr(oap) == OP_NOP {
        nvim_cursor_correct(curwin);
    }
    nvim_beginline(BL_SOL | BL_FIX);
}

/// L command: move cursor to bottom of window (with optional count from bottom).
unsafe fn nv_scroll_bottom(curwin: WinHandle, count1: c_int) {
    nvim_validate_botline(curwin);
    let botline = nvim_win_get_botline(curwin);
    nvim_win_set_cursor_lnum(curwin, botline - 1);
    let cursor_lnum = nvim_win_get_cursor_lnum(curwin);
    if count1 > cursor_lnum {
        nvim_win_set_cursor_lnum(curwin, 1);
    } else if nvim_win_lines_concealed(curwin) != 0 {
        // Count a fold for one screen line.
        let mut remaining = count1 - 1;
        let topline = nvim_win_get_topline(curwin);
        while remaining > 0 && nvim_win_get_cursor_lnum(curwin) > topline {
            let mut fold_first: c_int = 0;
            nvim_hasFolding(
                curwin,
                nvim_win_get_cursor_lnum(curwin),
                &raw mut fold_first,
                std::ptr::null_mut(),
            );
            let conceal = nvim_decor_conceal_line(curwin, nvim_win_get_cursor_lnum(curwin), 1);
            remaining -= conceal + 1;
            if nvim_win_get_cursor_lnum(curwin) > topline {
                nvim_win_set_cursor_lnum(curwin, nvim_win_get_cursor_lnum(curwin) - 1);
            }
        }
    } else {
        nvim_win_set_cursor_lnum(curwin, nvim_win_get_cursor_lnum(curwin) - (count1 - 1));
    }
}

/// M command: compute line offset from topline for middle of window.
unsafe fn nv_scroll_middle(curwin: WinHandle, line_count: c_int) -> c_int {
    let topline = nvim_win_get_topline(curwin);
    let topfill = nvim_win_get_topfill(curwin);
    let mut used: c_int = -(nvim_win_get_fill(curwin, topline) - topfill);
    nvim_validate_botline(curwin);
    let view_height = nvim_win_get_view_height(curwin);
    let empty_rows = nvim_win_get_empty_rows(curwin);
    let half = (view_height - empty_rows + 1) / 2;
    let mut n_val: c_int = 0;
    loop {
        if topline + n_val >= line_count {
            break;
        }
        // Count half the number of filler lines to be "below this
        // line" and half to be "above the next line".
        if n_val > 0 && used + nvim_win_get_fill(curwin, topline + n_val) / 2 >= half {
            n_val -= 1;
            break;
        }
        used += nvim_plines_win(curwin, topline + n_val, 1);
        if used >= half {
            break;
        }
        let mut fold_last: c_int = 0;
        if nvim_hasFolding(
            curwin,
            topline + n_val,
            std::ptr::null_mut(),
            &raw mut fold_last,
        ) != 0
        {
            n_val = fold_last - topline;
        }
        n_val += 1;
    }
    if n_val > 0 && used > view_height {
        n_val -= 1;
    }
    n_val
}

/// H command: compute line offset from topline for top of window (with count).
unsafe fn nv_scroll_top(curwin: WinHandle, count1: c_int) -> c_int {
    let mut n_val = count1 - 1;
    if nvim_win_lines_concealed(curwin) != 0 {
        // Count a fold for one screen line.
        let mut lnum = nvim_win_get_topline(curwin);
        let botline = nvim_win_get_botline(curwin);
        loop {
            let conceal = nvim_decor_conceal_line(curwin, lnum - 1, 1);
            if conceal == 0 && n_val <= 0 {
                break;
            }
            if lnum >= botline - 1 {
                break;
            }
            let mut fold_last: c_int = 0;
            nvim_hasFolding(curwin, lnum, std::ptr::null_mut(), &raw mut fold_last);
            lnum = fold_last + 1;
            n_val -= conceal + 1;
        }
        lnum - nvim_win_get_topline(curwin)
    } else {
        n_val
    }
}

// Phase 1 constants
const MOD_MASK_SHIFT_P1: c_int = 0x02;
const CA_NO_ADJ_OP_END_P1: c_int = 2;

/// Command handler for cursor right commands.
///
/// Handles 'l', space, and right arrow key movement.
/// With Shift/Ctrl modifiers, moves by word instead.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_right(cap: CapHandle) {
    // <C-Right> and <S-Right> move a word or WORD right
    if (nvim_get_mod_mask() & (MOD_MASK_SHIFT_P1 | MOD_MASK_CTRL)) != 0 {
        if (nvim_get_mod_mask() & MOD_MASK_CTRL) != 0 {
            (*cap.cast::<CmdargT>()).arg = 1;
        }
        rs_nv_wordcmd(cap);
        return;
    }

    let oap = nvim_cap_get_oap(cap);
    nvim_oap_set_motion_type(oap, K_MT_CHARWISE);
    nvim_oap_set_inclusive(oap, false);

    // past_line: in Visual mode with 'selection' != 'o', cursor can go past EOL
    #[allow(clippy::cast_possible_wrap)]
    let sel_o_p1 = b'o' as std::ffi::c_char;
    let past_line = if virtual_active(nvim_get_curwin()) {
        // In virtual edit mode there is no past_line
        false
    } else {
        VIsual_active && nvim_get_p_sel_first() != sel_o_p1
    };

    let count1 = nvim_cap_get_count1(cap);
    let cmdchar = nvim_cap_get_cmdchar(cap);
    let mut n = count1;
    loop {
        if n <= 0 {
            break;
        }

        // Check if we can move right; for non-past_line, oneright() moves the cursor
        let cannot_move_right = if past_line {
            *get_cursor_pos_ptr() == 0
        } else {
            oneright() == 0
        };

        if cannot_move_right {
            // Check whichwrap wrapping to next line
            let wrap = (cmdchar == c_int::from(b' ')
                && !vim_strchr(p_ww, c_int::from(b's')).is_null())
                || (cmdchar == c_int::from(b'l') && !vim_strchr(p_ww, c_int::from(b'l')).is_null())
                || (cmdchar == K_RIGHT && !vim_strchr(p_ww, c_int::from(b'>')).is_null());

            if wrap && nvim_get_cursor_lnum() < nvim_get_line_count() {
                // When deleting, count NL as a character
                if nvim_oap_get_op_type_ptr(oap) != OP_NOP
                    && !nvim_oap_get_inclusive(oap)
                    && !*ml_get(nvim_get_cursor_lnum()) == 0
                {
                    nvim_oap_set_inclusive(oap, true);
                } else {
                    // Move to start of next line
                    let lnum = nvim_get_cursor_lnum();
                    nvim_set_cursor_lnum(lnum + 1);
                    nvim_set_cursor_col(0);
                    nvim_set_cursor_coladd(0);
                    nvim_curwin_set_curswant(true);
                    nvim_oap_set_inclusive(oap, false);
                }
                n -= 1;
                continue;
            }

            if nvim_oap_get_op_type_ptr(oap) == OP_NOP {
                // Only beep if not moved at all
                if n == count1 {
                    beep_flush();
                }
            } else if !*ml_get(nvim_get_cursor_lnum()) == 0 {
                nvim_oap_set_inclusive(oap, true);
            }
            break;
        } else if past_line {
            // past_line move: set curswant and advance col
            nvim_curwin_set_curswant(true);
            if virtual_active(nvim_get_curwin()) {
                oneright();
            } else {
                nvim_set_cursor_col(nvim_get_cursor_col() + utfc_ptr2len(get_cursor_pos_ptr()));
            }
        }
        n -= 1;
    }

    if n != count1
        && (nvim_get_fdo_flags() & K_OPT_FDO_FLAG_HOR) != 0
        && nvim_get_KeyTyped()
        && nvim_oap_get_op_type_ptr(oap) == OP_NOP
    {
        rs_foldOpenCursor();
    }
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
    // <C-Left> and <S-Left> move a word or WORD left
    if (nvim_get_mod_mask() & (MOD_MASK_SHIFT_P1 | MOD_MASK_CTRL)) != 0 {
        if (nvim_get_mod_mask() & MOD_MASK_CTRL) != 0 {
            (*cap.cast::<CmdargT>()).arg = 1;
        }
        rs_nv_bck_word(cap);
        return;
    }

    let oap = nvim_cap_get_oap(cap);
    nvim_oap_set_motion_type(oap, K_MT_CHARWISE);
    nvim_oap_set_inclusive(oap, false);

    let count1 = nvim_cap_get_count1(cap);
    let cmdchar = nvim_cap_get_cmdchar(cap);
    let mut n = count1;
    loop {
        if n <= 0 {
            break;
        }

        if oneleft() == 0 {
            // Check whichwrap wrapping to previous line
            let wrap = (((cmdchar == K_BS || cmdchar == CTRL_H_KEY)
                && !vim_strchr(p_ww, c_int::from(b'b')).is_null())
                || (cmdchar == c_int::from(b'h')
                    && !vim_strchr(p_ww, c_int::from(b'h')).is_null())
                || (cmdchar == K_LEFT && !vim_strchr(p_ww, c_int::from(b'<')).is_null()))
                && nvim_get_cursor_lnum() > 1;

            if wrap {
                nvim_set_cursor_lnum(nvim_get_cursor_lnum() - 1);
                coladvance(nvim_get_curwin(), MAXCOL);
                nvim_curwin_set_curswant(true);

                // When deleting NL before first char: put cursor on NUL after prev line
                if (nvim_oap_get_op_type_ptr(oap) == OP_DELETE
                    || nvim_oap_get_op_type_ptr(oap) == OP_CHANGE)
                    && !*ml_get(nvim_get_cursor_lnum()) == 0
                {
                    if *get_cursor_pos_ptr() != 0 {
                        nvim_set_cursor_col(
                            nvim_get_cursor_col() + utfc_ptr2len(get_cursor_pos_ptr()),
                        );
                    }
                    (*cap.cast::<CmdargT>()).retval |= CA_NO_ADJ_OP_END_P1;
                }
                n -= 1;
                continue;
            } else if nvim_oap_get_op_type_ptr(oap) == OP_NOP && n == count1 {
                // Only beep if not moved at all
                beep_flush();
            }
            break;
        }
        n -= 1;
    }

    if n != count1
        && (nvim_get_fdo_flags() & K_OPT_FDO_FLAG_HOR) != 0
        && nvim_get_KeyTyped()
        && nvim_oap_get_op_type_ptr(oap) == OP_NOP
    {
        rs_foldOpenCursor();
    }
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
    if (nvim_get_mod_mask() & MOD_MASK_SHIFT_P1) != 0 {
        // <S-Up> is page up
        (*cap.cast::<CmdargT>()).arg = BACKWARD;
        rs_nv_page(cap);
        return;
    }

    let oap = nvim_cap_get_oap(cap);
    nvim_oap_set_motion_type(oap, K_MT_LINEWISE);
    let count1 = nvim_cap_get_count1(cap);
    if cursor_up(count1, nvim_oap_get_op_type_ptr(oap) == OP_NOP) == 0 {
        rs_clearopbeep(oap);
    } else if nvim_cap_get_arg(cap) != 0 {
        nvim_beginline(BL_WHITE | BL_FIX);
    }
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
    let oap = nvim_cap_get_oap(cap);
    let cmdchar = nvim_cap_get_cmdchar(cap);
    if (nvim_get_mod_mask() & MOD_MASK_SHIFT_P1) != 0 {
        // <S-Down> is page down
        (*cap.cast::<CmdargT>()).arg = FORWARD;
        rs_nv_page(cap);
    } else if nvim_bt_quickfix_curbuf() != 0 && cmdchar == CAR_CHAR {
        // Quickfix window only: view the result under the cursor.
        rs_qf_view_result(false);
    } else {
        // In the cmdline window a <CR> executes the command.
        if nvim_normal_get_cmdwin_type() != 0 && cmdchar == CAR_CHAR {
            nvim_set_cmdwin_result(CAR_CHAR);
        } else if nvim_bt_prompt_curbuf()
            && cmdchar == CAR_CHAR
            && nvim_win_get_cursor_lnum(nvim_get_curwin()) == nvim_get_line_count()
        {
            // In a prompt buffer a <CR> in the last line invokes the callback.
            prompt_invoke_callback();
            if restart_edit == 0 {
                restart_edit = c_int::from(b'a');
            }
        } else {
            nvim_oap_set_motion_type(oap, K_MT_LINEWISE);
            let count1 = nvim_cap_get_count1(cap);
            if !cursor_down(count1, nvim_oap_get_op_type_ptr(oap) == OP_NOP) {
                rs_clearopbeep(oap);
            } else if nvim_cap_get_arg(cap) != 0 {
                nvim_beginline(BL_WHITE | BL_FIX);
            }
        }
    }
}

// =============================================================================
// Phase 8: Miscellaneous handlers
// =============================================================================

extern "C" {
    // nvim_nv_at_impl migrated to Rust in Phase 1
    // Phase 3: nv_join / nv_open accessors
    fn do_join(
        count: usize,
        insert_space: bool,
        save_undo: bool,
        use_fo: bool,
        setmark: bool,
    ) -> c_int;
    fn nv_diffgetput(put: bool, count: usize);

    // g-command C accessors
    fn current_search(count: c_int, forward: bool) -> bool;
    fn cursor_up(count: c_int, upd_topline: bool) -> c_int;
    fn cursor_pos_info(dict: *mut std::ffi::c_void);
    fn nvim_invoke_edit_g(cap: CapHandle);
    fn nvim_set_mod_mask_ctrl();
    fn nvim_do_mouse_g(oap: OapHandle, nchar: c_int, count1: c_int);
    fn rs_goto_byte(count: c_int);
    fn undo_time(count: c_int, sec: bool, file: bool, absolute: bool);
    fn show_sb_text();
    fn show_utf8();
    fn utf_find_illegal();
    fn nvim_set_oap_cursor_start(oap: OapHandle);
    // nvim_curwin_set_curswant already declared above (consolidated, Phase 4)
    // Phase 3: nv_g_home_m_cmd / nv_g_dollar_cmd / n_opencmd / unadjust_for_sel_inner
    fn nvim_get_curwin_w_skipcol() -> c_int;
    fn nvim_get_curwin_w_topline() -> c_int;
    fn nvim_get_curwin_w_cline_folded() -> bool;
    fn nvim_clear_curwin_w_valid_wcol();
    fn nvim_utf_ptr2cells_cursor() -> c_int;
    fn nvim_getvvcol_cursor_end() -> c_int;
    fn nvim_hasFolding_cursor_set_lnum_up();
    fn nvim_hasFolding_cursor_set_lnum_down();
    fn nvim_set_curbuf_b_last_changedtick_i();
    fn nvim_u_save_for_opencmd(backward: bool) -> bool;
    fn nvim_open_line_for_opencmd(backward: bool, do_com: bool) -> bool;
    fn nvim_has_format_option_fo_open_coms() -> bool;
    fn nvim_clear_curwin_w_valid_crow();
    fn nvim_mark_mb_adjustpos_cursor_new() -> c_int;
    fn nvim_getvcol_cursor_coladd_after_adj() -> c_int;
    fn nvim_mark_mb_adjustpos_visual_new() -> c_int;
    fn nvim_getvcol_visual_coladd_after_adj() -> c_int;
    // nvim_get_VIsual_coladd/col/lnum already declared above at lines ~261-263
    fn nvim_set_VIsual_coladd(val: c_int);
    fn nvim_set_VIsual_col(val: c_int);
    fn nvim_set_VIsual_lnum(val: c_int);
    fn messaging() -> bool;
    fn rs_hasAnyFolding(win: WinHandle) -> c_int;
    fn do_sleep(ms: c_int, allow_int: bool);
    fn do_exmode();
    fn rs_do_ascii(eap: *mut std::ffi::c_void);
}

// =============================================================================
// nv_screengo: screen-based movement for gj/gk
// =============================================================================

extern "C" {
    fn nvim_get_curwin_w_virtcol() -> c_int;
    fn nvim_get_curwin_ml_line_count() -> c_int;
    fn nvim_validate_virtcol_curwin();
    fn oneright() -> c_int;
    fn nvim_vim_isprintc_wrapper(c: c_int) -> bool;
    fn vim_strsize(s: *const c_char) -> c_int;
    fn adjust_skipcol();
    fn rs_get_showbreak_value(wp: WinHandle) -> *const c_char;
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
    let mut linelen = linetabsize(nvim_get_curwin(), nvim_get_cursor_lnum());
    let mut retval = true;
    let mut atend = false;

    nvim_oap_set_motion_type(oap, K_MT_CHARWISE);
    nvim_oap_set_inclusive(oap, nvim_get_curswant() == MAXCOL);

    let col_off1 = win_col_off(nvim_get_curwin());
    let col_off2 = col_off1 - win_col_off2(nvim_get_curwin());
    let width1 = nvim_get_curwin_w_view_width() - col_off1;
    let mut width2 = nvim_get_curwin_w_view_width() - col_off2;

    if width2 == 0 {
        width2 = 1; // Avoid divide by zero.
    }

    if nvim_get_curwin_w_view_width() != 0 {
        // Instead of sticking at the last character of the buffer line we
        // try to stick in the last column of the screen.
        if nvim_get_curswant() == MAXCOL {
            atend = true;
            nvim_validate_virtcol_curwin();
            if width1 <= 0 {
                nvim_set_curswant(0);
            } else {
                nvim_set_curswant(width1 - 1);
                if nvim_get_curwin_w_virtcol() > nvim_get_curswant() {
                    let extra = ((nvim_get_curwin_w_virtcol() - nvim_get_curswant() - 1) / width2
                        + 1)
                        * width2;
                    nvim_set_curswant(nvim_get_curswant() + extra);
                }
            }
        } else {
            let n = if linelen > width1 {
                ((linelen - width1 - 1) / width2 + 1) * width2 + width1
            } else {
                width1
            };
            if nvim_get_curswant() >= n {
                nvim_set_curswant(n - 1);
            }
        }

        let mut remaining = dist;
        while remaining > 0 {
            remaining -= 1;
            if dir == BACKWARD {
                if nvim_get_curswant() >= width1 && !nvim_hasFolding_curwin(nvim_get_cursor_lnum())
                {
                    // Move back within the line. This can give a negative value
                    // for w_curswant if width1 < width2 (with cpoptions+=n),
                    // which will get clipped to column 0.
                    nvim_set_curswant(nvim_get_curswant() - width2);
                } else {
                    // to previous line
                    if nvim_get_cursor_lnum() <= 1 {
                        retval = false;
                        break;
                    }
                    cursor_up_inner(nvim_get_curwin(), 1, skip_conceal);

                    linelen = linetabsize(nvim_get_curwin(), nvim_get_cursor_lnum());
                    if linelen > width1 {
                        let w = (((linelen - width1 - 1) / width2) + 1) * width2;
                        nvim_set_curswant(nvim_get_curswant() + w);
                    }
                }
            } else {
                // dir == FORWARD
                let n = if linelen > width1 {
                    ((linelen - width1 - 1) / width2 + 1) * width2 + width1
                } else {
                    width1
                };
                if nvim_get_curswant() + width2 < n
                    && !nvim_hasFolding_curwin(nvim_get_cursor_lnum())
                {
                    // move forward within line
                    nvim_set_curswant(nvim_get_curswant() + width2);
                } else {
                    // to next line
                    if nvim_get_cursor_lnum() >= nvim_get_curwin_ml_line_count() {
                        retval = false;
                        break;
                    }
                    cursor_down_inner(nvim_get_curwin(), 1, skip_conceal);
                    let remainder = nvim_get_curswant() % width2;
                    nvim_set_curswant(remainder);

                    // Check if the cursor has moved below the number display
                    // when width1 < width2 (with cpoptions+=n). Subtract width2
                    // to get a negative value for w_curswant, which will get
                    // clipped to column 0.
                    if nvim_get_curswant() >= width1 {
                        nvim_set_curswant(nvim_get_curswant() - width2);
                    }
                    linelen = linetabsize(nvim_get_curwin(), nvim_get_cursor_lnum());
                }
            }
        }
    }

    if virtual_active(nvim_get_curwin()) && atend {
        coladvance(nvim_get_curwin(), MAXCOL);
    } else {
        coladvance(nvim_get_curwin(), nvim_get_curswant());
    }

    if nvim_get_cursor_col() > 0 && nvim_get_curwin_w_p_wrap() {
        // Check for landing on a character that got split at the end of the
        // last line. We want to advance a screenline, not end up in the same
        // screenline or move two screenlines.
        nvim_validate_virtcol_curwin();
        let mut virtcol = nvim_get_curwin_w_virtcol();
        let sbr = rs_get_showbreak_value(nvim_get_curwin());
        if virtcol > width1 && !sbr.is_null() && *sbr != 0 {
            virtcol -= vim_strsize(sbr);
        }

        let c = utf_ptr2char(get_cursor_pos_ptr());
        if dir == FORWARD
            && virtcol < nvim_get_curswant()
            && nvim_get_curswant() <= width1
            && !nvim_vim_isprintc_wrapper(c)
            && c > 255
        {
            oneright();
        }

        if virtcol > nvim_get_curswant()
            && (if nvim_get_curswant() < width1 {
                nvim_get_curswant() > width1 / 2
            } else {
                (nvim_get_curswant() - width1) % width2 > width2 / 2
            })
        {
            nvim_set_cursor_col(nvim_get_cursor_col() - 1);
        }
    }

    if atend {
        nvim_set_curswant(MAXCOL); // stick in the last column
    }
    adjust_skipcol();

    retval
}

// =============================================================================
// Phase 3: nv_g_home_m_cmd, nv_g_dollar_cmd, n_opencmd, unadjust_for_sel_inner
// =============================================================================

/// "g0", "g^", "gm": Move to start, first non-blank, or middle of screen line.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_nv_g_home_m_cmd(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    let nchar = nvim_cap_get_nchar(cap);
    let flag = nchar == c_int::from(b'^');

    nvim_oap_set_motion_type(oap, K_MT_CHARWISE);
    nvim_oap_set_inclusive(oap, false);

    let mut i: c_int;
    if nvim_get_curwin_w_p_wrap() && nvim_get_curwin_w_view_width() != 0 {
        let col_off1 = win_col_off(nvim_get_curwin());
        let col_off2 = col_off1 - win_col_off2(nvim_get_curwin());
        let width1 = nvim_get_curwin_w_view_width() - col_off1;
        let width2 = width1 + col_off2;

        nvim_validate_virtcol_curwin();
        let virtcol = nvim_get_curwin_w_virtcol();
        i = 0;
        if virtcol >= width1 && width2 > 0 {
            i = (virtcol - width1) / width2 * width2 + width1;
        }

        // When ending up below 'smoothscroll' marker, move just beyond it so
        // that skipcol is not adjusted later.
        let skipcol = nvim_get_curwin_w_skipcol();
        if skipcol > 0 && nvim_get_cursor_lnum() == nvim_get_curwin_w_topline() {
            let overlap =
                sms_marker_overlap(nvim_get_curwin(), nvim_get_curwin_w_view_width() - col_off2);
            if overlap > 0 && i == skipcol {
                i += overlap;
            }
        }
    } else {
        i = nvim_get_curwin_w_leftcol();
    }

    // Go to the middle of the screen line.
    if nchar == c_int::from(b'm') {
        let col_off1 = win_col_off(nvim_get_curwin());
        let col_off2 = col_off1 - win_col_off2(nvim_get_curwin());
        i += i32::midpoint(
            nvim_get_curwin_w_view_width() - col_off1,
            if nvim_get_curwin_w_p_wrap() && i > 0 {
                col_off2
            } else {
                0
            },
        );
    }

    coladvance(nvim_get_curwin(), i);

    if flag {
        loop {
            let c = utf_ptr2char(get_cursor_pos_ptr());
            if rs_ascii_iswhite(c) == 0 || oneright() != 0 {
                break;
            }
        }
        nvim_clear_curwin_w_valid_wcol();
    }

    nvim_curwin_set_curswant(true);

    if rs_hasAnyFolding(nvim_get_curwin()) != 0 {
        validate_cheight(nvim_get_curwin());
        if nvim_get_curwin_w_cline_folded() {
            update_curswant_force();
        }
    }
    adjust_skipcol();
}

/// "g$": like "$" but for screen lines.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_nv_g_dollar_cmd(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    let nchar = nvim_cap_get_nchar(cap);
    let count1 = nvim_cap_get_count1(cap);
    let col_off = win_col_off(nvim_get_curwin());
    let flag = nchar == K_END || nchar == K_KEND;

    nvim_oap_set_motion_type(oap, K_MT_CHARWISE);
    nvim_oap_set_inclusive(oap, true);

    if nvim_get_curwin_w_p_wrap() && nvim_get_curwin_w_view_width() != 0 {
        nvim_set_curswant(MAXCOL);
        if count1 == 1 {
            let col_off2 = col_off - win_col_off2(nvim_get_curwin());
            let width1 = nvim_get_curwin_w_view_width() - col_off;
            let width2 = width1 + col_off2;

            nvim_validate_virtcol_curwin();
            let virtcol = nvim_get_curwin_w_virtcol();
            let mut i = width1 - 1;
            if virtcol >= width1 {
                i += ((virtcol - width1) / width2 + 1) * width2;
            }
            coladvance(nvim_get_curwin(), i);

            // Make sure we stick in this column.
            update_curswant_force();
            if nvim_get_cursor_col() > 0 && nvim_get_curwin_w_p_wrap() {
                // Check for landing on a character that got split at
                // the end of the line.
                nvim_validate_virtcol_curwin();
                if nvim_get_curwin_w_virtcol() > i {
                    let col = nvim_get_cursor_col() - 1;
                    nvim_set_cursor_col(col);
                }
            }
        } else if !rs_nv_screengo(oap, FORWARD, count1 - 1, false) {
            rs_clearopbeep(oap);
        }
    } else {
        if count1 > 1 {
            // if it fails, let the cursor still move to the last char
            cursor_down(count1 - 1, false);
        }
        let i = nvim_get_curwin_w_leftcol() + nvim_get_curwin_w_view_width() - col_off - 1;
        coladvance(nvim_get_curwin(), i);

        // if the character doesn't fit move one back
        if nvim_get_cursor_col() > 0 && nvim_utf_ptr2cells_cursor() > 1 {
            let vcol = nvim_getvvcol_cursor_end();
            if vcol >= nvim_get_curwin_w_leftcol() + nvim_get_curwin_w_view_width() - col_off {
                let col = nvim_get_cursor_col() - 1;
                nvim_set_cursor_col(col);
            }
        }

        // Make sure we stick in this column.
        update_curswant_force();
    }

    if flag {
        loop {
            let c = utf_ptr2char(get_cursor_pos_ptr());
            if rs_ascii_iswhite_or_nul(c) == 0 || oneleft() != 0 {
                break;
            }
        }
        nvim_clear_curwin_w_valid_wcol();
    }
}

/// Handle "o" and "O" commands.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_n_opencmd(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    if rs_checkclearopq(oap) {
        return;
    }

    let cmdchar = nvim_cap_get_cmdchar(cap);
    let backward = cmdchar == c_int::from(b'O');

    if backward {
        // Open above the first line of a folded sequence of lines
        nvim_hasFolding_cursor_set_lnum_up();
    } else {
        // Open below the last line of a folded sequence of lines
        nvim_hasFolding_cursor_set_lnum_down();
    }

    // trigger TextChangedI for the 'o/O' command
    nvim_set_curbuf_b_last_changedtick_i();

    if nvim_u_save_for_opencmd(backward) {
        let do_com = nvim_has_format_option_fo_open_coms();
        if nvim_open_line_for_opencmd(backward, do_com) {
            if win_cursorline_standout(nvim_get_curwin()) {
                // force redraw of cursorline
                nvim_clear_curwin_w_valid_crow();
            }
            rs_invoke_edit(cap, false, cmdchar, true);
        }
    }
}

/// Move an arbitrary position back one char for 'selection' == "exclusive".
///
/// Takes lnum/col/coladd as in/out pointer params and updates them in place.
/// Returns true when backed up to the previous line.
///
/// # Safety
/// All pointer arguments must be valid non-null pointers. curwin/curbuf must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_unadjust_for_sel_inner(
    lnum: *mut c_int,
    col: *mut c_int,
    coladd: *mut c_int,
) -> bool {
    nvim_set_VIsual_select_exclu_adj(false);
    if *coladd > 0 {
        *coladd -= 1;
    } else if *col > 0 {
        *col -= 1;
        let mut new_col: c_int = 0;
        nvim_mark_mb_adjustpos_pos(*lnum, *col, &raw mut new_col);
        *col = new_col;
        if virtual_active(nvim_get_curwin()) {
            *coladd = nvim_getvcol_pos_coladd(*lnum, *col, *coladd);
        }
    } else if *lnum > 1 {
        *lnum -= 1;
        *col = nvim_ml_get_len_call(*lnum);
        return true;
    }
    false
}

/// Move position `*pp` back one character for 'selection' == "exclusive".
///
/// This is the public C-facing entry point that takes a pos_T pointer directly.
/// Returns true when backed up to the previous line.
///
/// # Safety
/// `pp` must be a valid non-null pointer to a `pos_T`.
#[export_name = "unadjust_for_sel_inner"]
pub unsafe extern "C" fn rs_unadjust_for_sel_inner_pos(pp: *mut c_void) -> bool {
    let mut lnum = nvim_pos_get_lnum(pp);
    let mut col = nvim_pos_get_col(pp);
    let mut coladd = nvim_pos_get_coladd(pp);
    let backed_up = rs_unadjust_for_sel_inner(&raw mut lnum, &raw mut col, &raw mut coladd);
    nvim_pos_set_lnum(pp, lnum);
    nvim_pos_set_col(pp, col);
    nvim_pos_set_coladd(pp, coladd);
    backed_up
}

/// Move position of curwin cursor back one char for 'selection' == "exclusive".
///
/// Returns true when backed up to the previous line.
///
/// # Safety
/// Caller must ensure curwin global is valid.
#[no_mangle]
pub unsafe extern "C" fn rs_unadjust_for_sel_inner_cursor() -> bool {
    nvim_set_VIsual_select_exclu_adj(false);
    let coladd = nvim_get_cursor_coladd();
    if coladd > 0 {
        nvim_set_cursor_coladd(coladd - 1);
    } else {
        let col = nvim_get_cursor_col();
        if col > 0 {
            nvim_set_cursor_col(col - 1);
            nvim_mark_mb_adjustpos_cursor_new();
            if virtual_active(nvim_get_curwin()) {
                let coladd_new = nvim_getvcol_cursor_coladd_after_adj();
                nvim_set_cursor_coladd(coladd_new);
            }
        } else {
            let lnum = nvim_get_cursor_lnum();
            if lnum > 1 {
                let new_lnum = lnum - 1;
                nvim_set_cursor_lnum(new_lnum);
                nvim_set_cursor_col(nvim_ml_get_len_call(new_lnum));
                return true;
            }
        }
    }
    false
}

/// Move VIsual position back one char for 'selection' == "exclusive".
///
/// Returns true when backed up to the previous line.
///
/// # Safety
/// Caller must ensure VIsual global is valid.
#[no_mangle]
pub unsafe extern "C" fn rs_unadjust_for_sel_inner_visual() -> bool {
    nvim_set_VIsual_select_exclu_adj(false);
    let coladd = nvim_get_VIsual_coladd();
    if coladd > 0 {
        nvim_set_VIsual_coladd(coladd - 1);
    } else {
        let col = nvim_get_VIsual_col();
        if col > 0 {
            nvim_set_VIsual_col(col - 1);
            nvim_mark_mb_adjustpos_visual_new();
            if virtual_active(nvim_get_curwin()) {
                let coladd_new = nvim_getvcol_visual_coladd_after_adj();
                nvim_set_VIsual_coladd(coladd_new);
            }
        } else {
            let lnum = nvim_get_VIsual_lnum();
            if lnum > 1 {
                let new_lnum = lnum - 1;
                nvim_set_VIsual_lnum(new_lnum);
                nvim_set_VIsual_col(nvim_ml_get_len_call(new_lnum));
                return true;
            }
        }
    }
    false
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
            if VIsual_active {
                (*cap.cast::<CmdargT>()).arg = 1; // cap->arg = true
                (*cap.cast::<CmdargT>()).cmdchar = nchar;
                (*cap.cast::<CmdargT>()).nchar = NUL_VAL;
                rs_nv_addsub(cap);
            } else {
                rs_clearopbeep(oap);
            }
        }

        // "gR": Enter virtual replace mode.
        n if n == b'R' as c_int => {
            (*cap.cast::<CmdargT>()).arg = 1;
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
            (*cap.cast::<CmdargT>()).nchar = CTRL_H_KEY;
            // FALLTHROUGH
            (*cap.cast::<CmdargT>()).cmdchar =
                nvim_cap_get_nchar(cap) + (b'v' as c_int - b'h' as c_int);
            (*cap.cast::<CmdargT>()).arg = 1;
            rs_nv_visual(cap);
        }
        n if n == b'h' as c_int || n == b'H' as c_int || n == CTRL_H_KEY => {
            (*cap.cast::<CmdargT>()).cmdchar = nchar + (b'v' as c_int - b'h' as c_int);
            (*cap.cast::<CmdargT>()).arg = 1;
            rs_nv_visual(cap);
        }

        // "gn", "gN" visually select next/previous search match
        n if n == b'N' as c_int || n == b'n' as c_int => {
            if !current_search(nvim_cap_get_count1(cap), nchar == b'n' as c_int) {
                rs_clearopbeep(oap);
            }
        }

        // "gj" and "gk": screen-line movement
        n if n == b'j' as c_int || n == K_DOWN => {
            let ok = if nvim_get_curwin_w_p_wrap() {
                rs_nv_screengo(oap, FORWARD, nvim_cap_get_count1(cap), false)
            } else {
                nvim_oap_set_motion_type(oap, K_MT_LINEWISE);
                cursor_down(
                    nvim_cap_get_count1(cap),
                    nvim_oap_get_op_type_ptr(oap) == OP_NOP,
                )
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
                cursor_up(
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
            rs_nv_g_home_m_cmd(cap);
        }

        // "gM": middle of text in the line
        n if n == b'M' as c_int => {
            nvim_oap_set_motion_type(oap, K_MT_CHARWISE);
            nvim_oap_set_inclusive(oap, false);
            let i = linetabsize(nvim_get_curwin(), nvim_get_cursor_lnum());
            let count0 = nvim_cap_get_count0(cap);
            if count0 > 0 && count0 <= 100 {
                coladvance(nvim_get_curwin(), i * count0 / 100);
            } else {
                coladvance(nvim_get_curwin(), i / 2);
            }
            nvim_curwin_set_curswant(true);
        }

        // "g_": to the last non-blank character
        n if n == b'_' as c_int => {
            rs_nv_g_underscore_cmd(cap);
        }

        // "g$": like "$" but for screen lines
        n if n == b'$' as c_int || n == K_END || n == K_KEND => {
            rs_nv_g_dollar_cmd(cap);
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
            nvim_curwin_set_curswant(true);
            nvim_oap_set_inclusive(oap, true);
            if bckend_word(nvim_cap_get_count1(cap), nchar == b'E' as c_int, false) == 0 {
                rs_clearopbeep(oap);
            }
        }

        // "g CTRL-G": display info about cursor position
        n if n == CTRL_G_KEY => {
            cursor_pos_info(std::ptr::null_mut());
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
            rs_nv_gotofile(cap);
        }

        // "g'm" and "g`m": jump to mark without setting pcmark
        n if n == b'\'' as c_int => {
            (*cap.cast::<CmdargT>()).arg = 1;
            // FALLTHROUGH
            rs_nv_gomark(cap);
        }
        n if n == b'`' as c_int => {
            rs_nv_gomark(cap);
        }

        // "gs": Goto sleep.
        n if n == b's' as c_int => {
            do_sleep(nvim_cap_get_count1(cap) * 1000, false);
        }

        // "ga": Display the ascii value of the character under the cursor.
        n if n == b'a' as c_int => {
            rs_do_ascii(std::ptr::null_mut());
        }

        // "g8": Display UTF-8 bytes or find illegal byte sequence.
        n if n == b'8' as c_int => {
            if nvim_cap_get_count0(cap) == 8 {
                utf_find_illegal();
            } else {
                show_utf8();
            }
        }

        // "g<": show scrollback text
        n if n == b'<' as c_int => {
            show_sb_text();
        }

        // "gg": Goto first line or line number.
        n if n == b'g' as c_int => {
            (*cap.cast::<CmdargT>()).arg = 0;
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

        // "gd", "gD": Find definition (inlined nvim_nv_gd_impl)
        n if n == b'd' as c_int || n == b'D' as c_int => {
            let thisblock = nvim_cap_get_count0(cap);
            let mut ptr: *mut c_char = core::ptr::null_mut();
            let len = rs_find_ident_at_pos(
                nvim_get_curwin(),
                nvim_get_cursor_lnum(),
                nvim_get_cursor_col(),
                &raw mut ptr,
                core::ptr::null_mut(),
                FIND_IDENT,
            );
            if len == 0
                || !rs_find_decl(
                    ptr,
                    len,
                    nchar == c_int::from(b'd'),
                    thisblock != 0,
                    SEARCH_START,
                )
            {
                rs_clearopbeep(oap);
            } else {
                if (nvim_get_fdo_flags() & K_OPT_FDO_FLAG_SEARCH) != 0
                    && nvim_get_KeyTyped()
                    && nvim_oap_get_op_type_ptr(oap) == OP_NOP
                {
                    rs_foldOpenCursor();
                }
                // clear any search statistics
                if messaging() {
                    nvim_set_clear_cmdline(true);
                }
            }
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
            rs_goto_byte(nvim_cap_get_count0(cap));
        }

        // "gQ": improved Ex mode
        n if n == b'Q' as c_int => {
            if !rs_check_text_locked(oap) && !rs_checkclearopq(oap) {
                do_exmode();
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

        n if n == TAB_CHAR => {
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
                undo_time(count, false, false, false);
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
    let oap = nvim_cap_get_oap(cap);
    if rs_checkclearop(oap) {
        return;
    }
    let nchar = nvim_cap_get_nchar(cap);
    if nchar == c_int::from(b'=') && nvim_get_expr_register() == NUL_CHAR {
        return;
    }
    let mut count = nvim_cap_get_count1(cap);
    while count > 0 && !got_int {
        count -= 1;
        if do_execreg(nchar, 0, 0, 0) == 0 {
            rs_clearopbeep(oap);
            break;
        }
        line_breakcheck();
    }
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
    if VIsual_active {
        // join the visual lines
        rs_nv_operator(cap);
        return;
    }

    let oap = nvim_cap_get_oap(cap);
    if rs_checkclearop(oap) {
        return;
    }

    // default for join is two lines!
    let mut count0 = (*cap.cast::<CmdargT>()).count0.max(2);
    nvim_cap_set_count0(cap, count0);

    let cursor_lnum = nvim_get_cursor_lnum();
    let ml_line_count = nvim_get_line_count();
    if cursor_lnum + count0 - 1 > ml_line_count {
        // can't join when on the last line
        if count0 <= 2 {
            rs_clearopbeep(oap);
            return;
        }
        count0 = ml_line_count - cursor_lnum + 1;
        nvim_cap_set_count0(cap, count0);
    }

    let regname = nvim_oap_get_regname_ptr(oap);
    let cmdchar = nvim_cap_get_cmdchar(cap);
    let nchar = nvim_cap_get_nchar(cap);
    rs_prep_redo(
        regname, count0, NUL_CHAR, cmdchar, NUL_CHAR, NUL_CHAR, nchar,
    );
    // count0 is clamped to >= 2 above, so cast is safe
    #[allow(clippy::cast_sign_loss)]
    let count0_usize = count0 as usize;
    do_join(count0_usize, nchar == NUL_CHAR, true, true, true);
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
    let oap = nvim_cap_get_oap(cap);
    let cmdchar = nvim_cap_get_cmdchar(cap);
    if nvim_oap_get_op_type_ptr(oap) == OP_DELETE && cmdchar == c_int::from(b'o') {
        // "do" is ":diffget"
        rs_clearop(oap);
        let opcount = (*cap.cast::<CmdargT>()).opcount;
        debug_assert!(opcount >= 0);
        #[allow(clippy::cast_sign_loss)]
        nv_diffgetput(false, opcount as usize);
    } else if VIsual_active {
        // switch start and end of visual
        rs_v_swap_corners(cmdchar);
    } else if nvim_bt_prompt_curbuf() && nvim_get_cursor_lnum() < nvim_get_b_prompt_start_lnum_put()
    {
        rs_clearopbeep(oap);
    } else {
        rs_n_opencmd(cap);
    }
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
        if VIsual_active {
            rs_end_visual_mode();
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
        if VIsual_active {
            rs_end_visual_mode();
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
        let orig = VISUAL_MODE_ORIG.load(Ordering::Relaxed);
        if orig != NUL_CHAR {
            nvim_set_curbuf_visual_vi_mode(orig);
            VISUAL_MODE_ORIG.store(NUL_CHAR, Ordering::Relaxed);
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
        AppendToRedobuff((*cap.cast::<CmdargT>()).nchar_composing.as_ptr());
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
    let opcount = i64::from((*cap.cast::<CmdargT>()).opcount);

    let count = if opcount != 0 {
        opcount * (if count0 == 0 { 1 } else { count0 })
    } else {
        count0
    };
    let count1 = if count == 0 { 1 } else { count };

    set_vcount(count, count1, *set_prevcount);
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
    while count1 > 0 && !got_int {
        count1 -= 1;
        if do_execreg(reg_recorded, 0, 0, 0) == 0 {
            rs_clearopbeep(oap);
            break;
        }
        line_breakcheck();
    }
}

/// CTRL-H / BS: in Select mode behaves like 'x', else like left.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_ctrlh(cap: CapHandle) {
    if VIsual_active && nvim_get_VIsual_select() {
        (*cap.cast::<CmdargT>()).cmdchar = c_int::from(b'x');
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
            VISUAL_MODE_ORIG.store(nvim_get_VIsual_mode(), Ordering::Relaxed);
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
    (*cap.cast::<CmdargT>()).cmdchar = translated;
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
        (*cap.cast::<CmdargT>()).cmdchar = c_int::from(b'x');
    }
    // in Visual mode these commands are operators
    if VIsual_active {
        rs_v_visop(cap);
    } else {
        nv_optrans_impl(cap);
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

    if cursor_down(count1 - 1, op_type == OP_NOP) {
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
        let select = (c == c_int::from(b'o') || (nvim_stuff_empty() && typebuf_typed()))
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
        rs_n_start_visual_mode(c_int::from(b'v'));
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
    if !cursor_down(count1 - 1, op_type == OP_NOP) {
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
            if virtual_active(nvim_get_curwin()) {
                let coladd = nvim_get_cursor_coladd();
                nvim_set_cursor_coladd(coladd + col - line_len);
            }
            nvim_set_cursor_col(line_len);
        }
    }
    (*cap.cast::<CmdargT>()).cmdchar = c_int::from(b'i');
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
        if restart_edit != 0 && nvim_get_mode_displayed() {
            nvim_set_clear_cmdline(true);
        }
        restart_edit = 0;
        if nvim_normal_get_cmdwin_type() != 0 {
            nvim_set_cmdwin_result(CTRL_C);
        }
        if VIsual_active {
            rs_end_visual_mode();
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

    if VIsual_active {
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
    update_topline(nvim_get_curwin());

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
        coladvance(nvim_get_curwin(), left);
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
        coladvance(nvim_get_curwin(), nvim_get_curswant());

        if nvim_get_cursor_col() == old_col
            && (!virtual_active(nvim_get_curwin()) || nvim_get_cursor_coladd() == old_coladd)
        {
            nvim_set_cursor_lnum(nvim_get_VIsual_lnum());
            if old_lnum <= nvim_get_VIsual_lnum() && nvim_p_sel_is_exclusive() {
                right += 1;
            }
            coladvance(nvim_get_curwin(), right);
            nvim_set_VIsual_pos(
                nvim_get_cursor_lnum(),
                nvim_get_cursor_col(),
                nvim_get_cursor_coladd(),
            );

            nvim_set_cursor_lnum(old_lnum);
            coladvance(nvim_get_curwin(), left);
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
        nvim_curwin_set_curswant(true);
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
            return rs_unadjust_for_sel_inner_cursor();
        }
        return rs_unadjust_for_sel_inner_visual();
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
        (*oap.cast::<OpargT>()).use_reg_one = true;

        let pos = findmatch(oap, 0);
        if pos.is_null() {
            rs_clearopbeep(oap);
        } else {
            nvim_setpcmark();
            nvim_set_cursor_pos((*pos).lnum, (*pos).col, 0);
            nvim_curwin_set_curswant(true);
            adjust_for_sel(cap);
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
// Phase 1: Last dispatch table handlers (nv_addsub, nv_colon, nv_record,
// nv_paste, nv_event)
// =============================================================================

// Key constants for Phase 1
const KE_COMMAND: c_int = 104;
const KE_LUA: c_int = 107;
const KE_CMDWIN: c_int = 84;

const K_COMMAND: c_int = termcap2key(KS_EXTRA, KE_COMMAND);
const K_LUA: c_int = termcap2key(KS_EXTRA, KE_LUA);
const K_CMDWIN: c_int = termcap2key(KS_EXTRA, KE_CMDWIN);

// OP_* constants for Phase 1
const OP_NR_ADD: c_int = 28;
const OP_NR_SUB: c_int = 29;
const OP_FORMAT: c_int = 9;

extern "C" {
    // Phase 1: addsub
    fn op_addsub(oap: OapHandle, count1: c_int, arg: c_int);

    // Phase 1: colon
    fn nvim_do_cmdline_for_colon(cap: CapHandle, is_cmdkey: bool) -> bool;
    fn nvim_map_execute_lua_for_colon() -> bool;
    fn nvim_compute_cmdrow();
    fn nvim_get_oap_start_lnum(cap: CapHandle) -> c_int;
    fn nvim_get_oap_start_col(cap: CapHandle) -> c_int;
    fn nvim_did_emsg_check() -> c_int;
    // nvim_ml_get_len_call already declared above (line ~299)

    // Phase 1: record
    fn do_record(nchar: c_int) -> c_int;
    fn nvim_get_reg_executing() -> c_int;
    fn nvim_get_e_cmdline_window_already_open() -> *const std::ffi::c_char;

    // Phase 1: paste
    fn nvim_paste_repeat(count: c_int);

    // Phase 1: event
    fn nvim_state_handle_k_event();
    fn nvim_set_may_garbage_collect(val: bool);
}

/// Command handler for CTRL-A and CTRL-X: Add or subtract from number/letter.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_addsub(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    if nvim_bt_prompt_curbuf() && !prompt_curpos_editable() {
        rs_clearopbeep(oap);
    } else if !VIsual_active && nvim_oap_get_op_type_ptr(oap) == OP_NOP {
        rs_prep_redo_cmd(cap);
        let cmdchar = nvim_cap_get_cmdchar(cap);
        nvim_oap_set_op_type(
            oap,
            if cmdchar == CTRL_A {
                OP_NR_ADD
            } else {
                OP_NR_SUB
            },
        );
        let count1 = nvim_cap_get_count1(cap);
        let arg = nvim_cap_get_arg(cap);
        op_addsub(oap, count1, arg);
        nvim_oap_set_op_type(oap, OP_NOP);
    } else if VIsual_active {
        rs_nv_operator(cap);
    } else {
        rs_clearop(oap);
    }
}

/// Command handler for ":", K_COMMAND, and K_LUA: Execute ex command or Lua mapping.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_colon(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);
    let cmdchar = nvim_cap_get_cmdchar(cap);
    let is_cmdkey = cmdchar == K_COMMAND;
    let is_lua = cmdchar == K_LUA;

    if VIsual_active && !is_cmdkey && !is_lua {
        rs_nv_operator(cap);
        return;
    }

    if nvim_oap_get_op_type_ptr(oap) != OP_NOP {
        // Using ":" as a movement is charwise exclusive.
        nvim_oap_set_motion_type(oap, K_MT_CHARWISE);
        nvim_oap_set_inclusive(oap, false);
    } else if nvim_cap_get_count0(cap) != 0 && !is_cmdkey && !is_lua {
        // translate "count:" into ":.,.+(count - 1)"
        nvim_stuffcharReadbuff(c_int::from(b'.'));
        if nvim_cap_get_count0(cap) > 1 {
            stuffReadbuff(c",.+".as_ptr());
            stuffnumReadbuff(nvim_cap_get_count0(cap) - 1);
        }
    }

    // When typing, don't type below an old message
    if nvim_get_KeyTyped() {
        nvim_compute_cmdrow();
    }

    let cmd_result = if is_lua {
        nvim_map_execute_lua_for_colon()
    } else {
        nvim_do_cmdline_for_colon(cap, is_cmdkey)
    };

    if !cmd_result {
        // The Ex command failed, do not execute the operator.
        rs_clearop(oap);
    } else if nvim_oap_get_op_type_ptr(oap) != OP_NOP
        && (nvim_get_oap_start_lnum(cap) > nvim_get_line_count()
            || nvim_get_oap_start_col(cap) > nvim_ml_get_len_call(nvim_get_oap_start_lnum(cap))
            || nvim_did_emsg_check() != 0)
    {
        // The start of the operator has become invalid by the Ex command.
        rs_clearopbeep(oap);
    }
}

/// Command handler for "q": Start/stop recording or open command-line window.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_record(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);

    if nvim_oap_get_op_type_ptr(oap) == OP_FORMAT {
        // "gqq" is the same as "gqgq": format line
        (*cap.cast::<CmdargT>()).cmdchar = c_int::from(b'g');
        (*cap.cast::<CmdargT>()).nchar = c_int::from(b'q');
        rs_nv_operator(cap);
        return;
    }

    if rs_checkclearop(oap) {
        return;
    }

    let nchar = nvim_cap_get_nchar(cap);
    if nchar == c_int::from(b':') || nchar == c_int::from(b'/') || nchar == c_int::from(b'?') {
        if nvim_normal_get_cmdwin_type() != 0 {
            nvim_emsg(nvim_get_e_cmdline_window_already_open());
            return;
        }
        nvim_stuffcharReadbuff(nchar);
        nvim_stuffcharReadbuff(K_CMDWIN);
    } else {
        // (stop) recording into a named register, unless executing a register.
        if nvim_get_reg_executing() == 0 && do_record(nchar) == FAIL {
            rs_clearopbeep(oap);
        }
    }
}

/// Command handler for K_PASTE_START: Repeat paste.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_paste(cap: CapHandle) {
    nvim_paste_repeat(nvim_cap_get_count1(cap));
}

/// Command handler for K_EVENT: Handle arbitrary events in normal mode.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nv_event(cap: CapHandle) {
    // Disable garbage collection during event handling (see comment in C original).
    nvim_set_may_garbage_collect(false);
    let may_restart = restart_edit != 0 || nvim_get_restart_VIsual_select() != 0;
    nvim_state_handle_k_event();
    nvim_set_finish_op(false);
    if may_restart {
        // If restart_edit was set before the handler we are in ctrl-o mode,
        // but if not, the event should be allowed to trigger :startinsert.
        (*cap.cast::<CmdargT>()).retval |= CA_COMMAND_BUSY;
    }
}

// =============================================================================
// Phase 2: normal_search, nv_gotofile, get_visual_text, nv_mark_move_to
// =============================================================================

// kMTLineWise constant for Phase 2
const K_MT_LINE_WISE_P2: c_int = 2;

// kMarkMoveFailed bit (MarkMoveRes::kMarkMoveFailed = 0x10)
const K_MARK_MOVED_FAILED: c_int = 0x10;

// kOptFdoFlagSearch == 0x40 (from option_vars.generated.h)
const K_OPT_FDO_FLAG_SEARCH: c_uint = 0x40;

extern "C" {
    // Phase 2: normal_search accessors
    fn nvim_do_search_call(
        oap: OapHandle,
        dir: c_int,
        pat: *mut std::ffi::c_char,
        patlen: usize,
        count1: c_int,
        opt: c_int,
        wrapped: *mut c_int,
    ) -> c_int;
    fn nvim_search_hls_needs_redraw(prev_lnum: c_int, prev_col: c_int, prev_coladd: c_int) -> bool;

    // Phase 2: nv_gotofile accessors
    fn nvim_grab_file_name(count1: c_int, lnum_out: *mut c_int) -> *mut std::ffi::c_char;
    fn curbufIsChanged() -> bool;
    fn nvim_autowrite_curbuf();
    fn check_can_set_curbuf_disabled() -> bool;
    fn nvim_do_ecmd_for_gotofile(ptr: *mut std::ffi::c_char) -> c_int;

    // Phase 2: get_visual_text accessors
    fn nvim_ml_get_pos_visual() -> *mut std::ffi::c_char;
    fn nvim_get_cursor_line_ptr() -> *mut std::ffi::c_char;

    // Phase 2: nv_mark_move_to accessor
    fn nvim_mark_move_to_call(fm: FmarkHandle, flags: c_int) -> c_int;
}

/// Search for "pat" in direction "dir" ('/' or '?', 0 for repeat).
///
/// Rust implementation of the formerly-C `normal_search`.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
pub unsafe fn rs_normal_search(
    cap: CapHandle,
    dir: c_int,
    pat: *mut std::ffi::c_char,
    patlen: usize,
    opt: c_int,
    wrapped: *mut c_int,
) -> c_int {
    let oap = nvim_cap_get_oap(cap);
    let count1 = nvim_cap_get_count1(cap);

    // Save cursor position for hlsearch redraw check
    let prev_lnum = nvim_get_cursor_lnum();
    let prev_col = nvim_get_cursor_col();
    let prev_coladd = nvim_get_cursor_coladd();

    nvim_oap_set_motion_type(oap, K_MT_CHARWISE);
    nvim_oap_set_inclusive(oap, false);
    (*oap.cast::<OpargT>()).use_reg_one = true;
    nvim_curwin_set_curswant(true);

    let i = nvim_do_search_call(oap, dir, pat, patlen, count1, opt, wrapped);

    if i == 0 {
        rs_clearop(oap);
    } else {
        if i == 2 {
            nvim_oap_set_motion_type(oap, K_MT_LINE_WISE_P2);
        }
        nvim_set_cursor_coladd(0);
        if nvim_oap_get_op_type_ptr(oap) == OP_NOP
            && (nvim_get_fdo_flags() & K_OPT_FDO_FLAG_SEARCH) != 0
            && nvim_get_KeyTyped()
        {
            rs_foldOpenCursor();
        }
    }

    if nvim_search_hls_needs_redraw(prev_lnum, prev_col, prev_coladd) {
        redraw_later(nvim_get_curwin(), UPD_SOME_VALID);
    }

    nvim_check_cursor();

    i
}

/// Get visually selected text within one line.
///
/// Rust implementation of the formerly-C `get_visual_text`.
/// Returns false if more than one line selected.
///
/// # Safety
/// `cap` may be null. If non-null, must be a valid cmdarg_T pointer.
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_get_visual_text(
    cap: CapHandle,
    pp: *mut *mut std::ffi::c_char,
    lenp: *mut usize,
) -> bool {
    let visual_mode_linewise = c_int::from(b'V');
    if nvim_get_VIsual_mode() != visual_mode_linewise {
        rs_unadjust_for_sel();
    }
    if nvim_get_VIsual_lnum() != nvim_get_cursor_lnum() {
        if !cap.is_null() {
            let oap = nvim_cap_get_oap(cap);
            rs_clearopbeep(oap);
        }
        return false;
    }
    if nvim_get_VIsual_mode() == visual_mode_linewise {
        *pp = nvim_get_cursor_line_ptr();
        *lenp = nvim_get_cursor_line_len() as usize;
    } else {
        // cursor_gt_VIsual: cursor is after VIsual
        if nvim_lt_VIsual_cursor() {
            // text from VIsual to cursor
            *pp = nvim_ml_get_pos_visual();
            *lenp = (nvim_get_cursor_col() - nvim_get_VIsual_col()) as usize + 1;
        } else {
            // text from cursor to VIsual
            *pp = nvim_ml_get_pos_cursor();
            *lenp = (nvim_get_VIsual_col() - nvim_get_cursor_col()) as usize + 1;
        }
        if !(*pp).is_null() && (**pp as u8) == b'\0' {
            *lenp = 0;
        }
        if *lenp > 0 {
            let tail_ptr = (*pp).add(*lenp - 1);
            let extra = (utfc_ptr2len(tail_ptr) - 1) as usize;
            *lenp += extra;
        }
    }
    rs_reset_VIsual_and_resel();
    true
}

/// Move the cursor to the mark position (nv_gotofile).
///
/// Rust implementation of the formerly-C `nv_gotofile`.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer.
pub unsafe fn rs_nv_gotofile(cap: CapHandle) {
    let oap = nvim_cap_get_oap(cap);

    if rs_check_text_or_curbuf_locked(oap) {
        return;
    }
    if !check_can_set_curbuf_disabled() {
        return;
    }

    let count1 = nvim_cap_get_count1(cap);
    let mut lnum: c_int = -1;
    let ptr = nvim_grab_file_name(count1, &raw mut lnum);

    if ptr.is_null() {
        rs_clearop(oap);
    } else {
        if curbufIsChanged() {
            nvim_autowrite_curbuf();
        }
        nvim_setpcmark();
        let nchar = nvim_cap_get_nchar(cap);
        if nvim_do_ecmd_for_gotofile(ptr) == OK && nchar == c_int::from(b'F') && lnum >= 0 {
            nvim_set_cursor_lnum(lnum);
            nvim_check_cursor_lnum_call();
            nvim_beginline(BL_SOL | BL_FIX);
        }
        xfree(ptr.cast::<c_void>());
    }
}

/// Move cursor to a mark position and set motion type (nv_mark_move_to).
///
/// Rust implementation of the formerly-C `nv_mark_move_to`.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer, `fm` must be a valid fmark_T pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_nv_mark_move_to(
    cap: CapHandle,
    flags: c_int,
    fm: FmarkHandle,
) -> c_int {
    let oap = nvim_cap_get_oap(cap);
    let res = nvim_mark_move_to_call(fm, flags);
    if (res & K_MARK_MOVED_FAILED) != 0 {
        rs_clearop(oap);
    }
    nvim_oap_set_motion_type(
        oap,
        if (flags & K_MARK_BEGIN_LINE) != 0 {
            K_MT_LINE_WISE_P2
        } else {
            K_MT_CHARWISE
        },
    );
    if nvim_cap_get_cmdchar(cap) == c_int::from(b'`') {
        (*oap.cast::<OpargT>()).use_reg_one = true;
    }
    nvim_oap_set_inclusive(oap, false);
    nvim_curwin_set_curswant(true);
    res
}

// =============================================================================
// Phase 3: Visual mode core helpers and set_op_var
// =============================================================================

// Phase 3 constants
// kOptVeFlagBlock == 0x05 (from option_vars.generated.h)
const K_OPT_VE_FLAG_BLOCK: c_uint = 0x05;
// kOptVeFlagAll == 0x04 (from option_vars.generated.h)
const K_OPT_VE_FLAG_ALL: c_uint = 0x04;
// UPD_INVERTED == 20
const UPD_INVERTED: c_int = 20;
// (CTRL_V, TAB_CHAR, UPD_VALID, MAXCOL are already defined above)

extern "C" {
    // Phase 3: n_start_visual_mode accessors
    fn conceal_check_cursor_line();
    fn nvim_set_mouse_dragging(val: c_int);
    fn adjust_cursor_eol();
    fn nvim_get_op_char(optype: c_int) -> c_int;
    fn nvim_get_extra_op_char(optype: c_int) -> c_int;
    fn nvim_set_vim_var_string_vv_op(opchars: *const std::ffi::c_char, len: c_int);
    fn nvim_get_cursor_pos_ptr_len() -> c_int;
    fn nvim_get_curwin_w_redr_type() -> c_int;
    fn nvim_curwin_set_old_visual_lnums();
    fn redraw_curbuf_later(type_val: c_int);
    fn rs_foldAdjustVisual();
}

/// Enter Visual mode `c`.
///
/// Rust implementation of the formerly-C `n_start_visual_mode`.
///
/// # Safety
/// Calls C accessor functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_n_start_visual_mode(c: c_int) {
    nvim_set_VIsual_mode(c);
    nvim_set_VIsual_active(true);
    nvim_set_VIsual_reselect(true);
    // Corner case: the 0 position in a tab may change when going into
    // virtualedit. Recalculate curwin->w_cursor to avoid bad highlighting.
    if c == CTRL_V
        && (nvim_get_ve_flags() & K_OPT_VE_FLAG_BLOCK) != 0
        && utf_ptr2char(get_cursor_pos_ptr()) == TAB_CHAR
    {
        nvim_validate_virtcol_curwin();
        coladvance(nvim_get_curwin(), nvim_get_curwin_w_virtcol());
    }
    // VIsual = curwin->w_cursor
    let lnum = nvim_get_cursor_lnum();
    let col = nvim_get_cursor_col();
    let coladd = nvim_get_cursor_coladd();
    nvim_set_VIsual_pos(lnum, col, coladd);

    rs_foldAdjustVisual();

    nvim_may_trigger_modechanged();
    nvim_setmouse();
    // Check for redraw after changing the state.
    conceal_check_cursor_line();

    if p_smd != 0 && msg_silent == 0 {
        nvim_set_redraw_cmdline(true); // show visual mode later
    }
    // Only need to redraw this line, unless still need to redraw an old
    // Visual area (when 'lazyredraw' is set).
    if nvim_get_curwin_w_redr_type() < UPD_INVERTED {
        nvim_curwin_set_old_visual_lnums();
    }
    redraw_curbuf_later(10); // UPD_VALID = 10
}

/// Exit Visual mode.
///
/// Rust implementation of the formerly-C `end_visual_mode`.
///
/// # Safety
/// Calls C accessor functions.
#[export_name = "end_visual_mode"]
pub unsafe extern "C" fn rs_end_visual_mode() {
    nvim_set_VIsual_select_exclu_adj(false);
    nvim_set_VIsual_active(false);
    nvim_setmouse();
    nvim_set_mouse_dragging(0);

    // Save the current VIsual area for '< and '> marks, and "gv"
    let vis_mode = nvim_get_VIsual_mode();
    nvim_set_curbuf_visual_vi_mode(vis_mode);
    nvim_set_b_visual_vi_start(
        nvim_get_VIsual_lnum(),
        nvim_get_VIsual_col(),
        nvim_get_VIsual_coladd(),
    );
    nvim_set_b_visual_vi_end(
        nvim_get_cursor_lnum(),
        nvim_get_cursor_col(),
        nvim_get_cursor_coladd(),
    );
    nvim_set_b_visual_vi_curswant(nvim_get_curswant());
    nvim_set_curbuf_visual_mode_eval(vis_mode);

    if !virtual_active(nvim_get_curwin()) {
        nvim_set_cursor_coladd(0);
    }

    rs_may_clear_cmdline();
    adjust_cursor_eol();
    nvim_may_trigger_modechanged();
}

/// Move the cursor for the "A" command.
///
/// Rust implementation of the formerly-C `set_cursor_for_append_to_line`.
///
/// # Safety
/// Calls C accessor functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_set_cursor_for_append_to_line() {
    nvim_curwin_set_curswant(true);
    if nvim_get_ve_flags() == K_OPT_VE_FLAG_ALL {
        // Pretend Insert mode to allow cursor past end of line
        let save_state = State;
        State = MODE_INSERT;
        nvim_coladvance_maxcol();
        State = save_state;
    } else {
        let extra = nvim_get_cursor_pos_ptr_len();
        nvim_set_cursor_col(nvim_get_cursor_col() + extra);
    }
}

/// Set v:operator variable based on optype.
///
/// Rust implementation of the formerly-C `set_op_var`.
///
/// # Safety
/// Calls C accessor functions.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::similar_names
)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_set_op_var(optype: c_int) {
    if optype == OP_NOP {
        nvim_set_vim_var_string_vv_op(core::ptr::null(), 0);
    } else {
        let opchar0 = nvim_get_op_char(optype) as u8;
        let opchar1 = nvim_get_extra_op_char(optype) as u8;
        let opchars: [std::ffi::c_char; 3] =
            [opchar0 as std::ffi::c_char, opchar1 as std::ffi::c_char, 0];
        nvim_set_vim_var_string_vv_op(opchars.as_ptr(), -1);
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
