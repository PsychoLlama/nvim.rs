//! Normal mode command dispatch constants.
//!
//! This module provides constants for normal mode command flags,
//! motion types, and character table lookup.

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// NV_* Command Flags (from normal.c)
// =============================================================================

/// May need to get a second character.
pub const NV_NCH: c_int = 0x01;

/// Get second char when no operator pending (includes NV_NCH).
pub const NV_NCH_NOP: c_int = 0x02 | NV_NCH;

/// Always get a second character (includes NV_NCH).
pub const NV_NCH_ALW: c_int = 0x04 | NV_NCH;

/// Second char needs language adjustment.
pub const NV_LANG: c_int = 0x08;

/// May start selection.
pub const NV_SS: c_int = 0x10;

/// May start selection with shift modifier.
pub const NV_SSS: c_int = 0x20;

/// May stop selection without shift modifier.
pub const NV_STS: c_int = 0x40;

/// 'rightleft' modifies command.
pub const NV_RL: c_int = 0x80;

/// Don't clear regname.
pub const NV_KEEPREG: c_int = 0x100;

/// Not allowed in command-line window.
pub const NV_NCW: c_int = 0x200;

// =============================================================================
// Motion Type Constants (from ops.h)
// =============================================================================

/// No motion yet.
pub const MCHAR: c_int = 0;

/// Character-wise (inclusive).
pub const MLINE: c_int = 1;

/// Line-wise.
pub const MBLOCK: c_int = 2;

/// Block-wise.
pub const MUNKNOWN: c_int = -1;

// =============================================================================
// Direction Constants
// =============================================================================

/// Backward direction.
pub const BACKWARD: c_int = -1;

/// Forward direction.
pub const FORWARD: c_int = 1;

// =============================================================================
// Character Constants
// =============================================================================

/// NUL character value.
pub const NUL_CHAR: c_int = 0;

/// Space character value.
pub const SPACE_CHAR: c_int = b' ' as c_int;

/// Tab character value.
pub const TAB_CHAR: c_int = b'\t' as c_int;

// =============================================================================
// Command Retval Constants (from normal_defs.h)
// =============================================================================

/// Command is busy (needs more characters).
pub const CA_COMMAND_BUSY: c_int = 1;

/// Normal command execution complete.
pub const CA_NO_ADJ_OP_END: c_int = 2;

// =============================================================================
// Flag Checking Functions
// =============================================================================

/// Check if a command flag set indicates need for second character.
#[inline]
fn needs_second_char(flags: c_int) -> bool {
    (flags & NV_NCH) != 0
}

/// Check if a command flag set indicates second char only when no operator.
#[inline]
fn needs_second_char_no_op(flags: c_int) -> bool {
    (flags & NV_NCH_NOP) == NV_NCH_NOP
}

/// Check if a command flag set indicates always get second char.
#[inline]
fn needs_second_char_always(flags: c_int) -> bool {
    (flags & NV_NCH_ALW) == NV_NCH_ALW
}

/// Check if second character needs language adjustment.
#[inline]
fn needs_lang_adjust(flags: c_int) -> bool {
    (flags & NV_LANG) != 0
}

/// Check if command may start selection.
#[inline]
fn may_start_selection(flags: c_int) -> bool {
    (flags & NV_SS) != 0
}

/// Check if command may start selection with shift.
#[inline]
fn may_start_selection_shift(flags: c_int) -> bool {
    (flags & NV_SSS) != 0
}

/// Check if command may stop selection without shift.
#[inline]
fn may_stop_selection(flags: c_int) -> bool {
    (flags & NV_STS) != 0
}

/// Check if rightleft modifies command.
#[inline]
fn is_rightleft_modified(flags: c_int) -> bool {
    (flags & NV_RL) != 0
}

/// Check if command keeps regname.
#[inline]
fn keeps_regname(flags: c_int) -> bool {
    (flags & NV_KEEPREG) != 0
}

/// Check if command not allowed in cmdline window.
#[inline]
fn not_in_cmdwin(flags: c_int) -> bool {
    (flags & NV_NCW) != 0
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get NV_NCH constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nv_flag_nch() -> c_int {
    NV_NCH
}

/// FFI: Get NV_NCH_NOP constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nv_flag_nch_nop() -> c_int {
    NV_NCH_NOP
}

/// FFI: Get NV_NCH_ALW constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nv_flag_nch_alw() -> c_int {
    NV_NCH_ALW
}

/// FFI: Get NV_LANG constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nv_flag_lang() -> c_int {
    NV_LANG
}

/// FFI: Get NV_SS constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nv_flag_ss() -> c_int {
    NV_SS
}

/// FFI: Get NV_SSS constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nv_flag_sss() -> c_int {
    NV_SSS
}

/// FFI: Get NV_STS constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nv_flag_sts() -> c_int {
    NV_STS
}

/// FFI: Get NV_RL constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nv_flag_rl() -> c_int {
    NV_RL
}

/// FFI: Get NV_KEEPREG constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nv_flag_keepreg() -> c_int {
    NV_KEEPREG
}

/// FFI: Get NV_NCW constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nv_flag_ncw() -> c_int {
    NV_NCW
}

/// FFI: Check if flags indicate need for second char.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nv_needs_second_char(flags: c_int) -> c_int {
    c_int::from(needs_second_char(flags))
}

/// FFI: Check if flags indicate second char only without operator.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nv_needs_second_char_no_op(flags: c_int) -> c_int {
    c_int::from(needs_second_char_no_op(flags))
}

/// FFI: Check if flags indicate always need second char.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nv_needs_second_char_always(flags: c_int) -> c_int {
    c_int::from(needs_second_char_always(flags))
}

/// FFI: Check if flags indicate language adjustment needed.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nv_needs_lang_adjust(flags: c_int) -> c_int {
    c_int::from(needs_lang_adjust(flags))
}

/// FFI: Check if flags indicate may start selection.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nv_may_start_selection(flags: c_int) -> c_int {
    c_int::from(may_start_selection(flags))
}

/// FFI: Check if flags indicate may start selection with shift.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nv_may_start_selection_shift(flags: c_int) -> c_int {
    c_int::from(may_start_selection_shift(flags))
}

/// FFI: Check if flags indicate may stop selection.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nv_may_stop_selection(flags: c_int) -> c_int {
    c_int::from(may_stop_selection(flags))
}

/// FFI: Check if flags indicate rightleft modification.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nv_is_rightleft_modified(flags: c_int) -> c_int {
    c_int::from(is_rightleft_modified(flags))
}

/// FFI: Check if flags indicate keep regname.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nv_keeps_regname(flags: c_int) -> c_int {
    c_int::from(keeps_regname(flags))
}

/// FFI: Check if flags indicate not in cmdwin.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nv_not_in_cmdwin(flags: c_int) -> c_int {
    c_int::from(not_in_cmdwin(flags))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nv_nch_flags() {
        assert_eq!(NV_NCH, 0x01);
        assert_eq!(NV_NCH_NOP, 0x03);
        assert_eq!(NV_NCH_ALW, 0x05);
    }

    #[test]
    fn test_needs_second_char() {
        assert!(needs_second_char(NV_NCH));
        assert!(needs_second_char(NV_NCH_NOP));
        assert!(needs_second_char(NV_NCH_ALW));
        assert!(!needs_second_char(NV_SS));
    }

    #[test]
    fn test_direction_constants() {
        assert_eq!(BACKWARD, -1);
        assert_eq!(FORWARD, 1);
    }

    #[test]
    fn test_flag_checks() {
        let flags = NV_SS | NV_STS;
        assert!(may_start_selection(flags));
        assert!(may_stop_selection(flags));
        assert!(!may_start_selection_shift(flags));
        assert!(!is_rightleft_modified(flags));
    }
}
