//! Normal mode execution infrastructure
//!
//! This module provides Rust implementations of the normal mode execution
//! infrastructure from `src/nvim/normal.c`, including:
//! - Command flags and state checking
//! - Multi-key command detection
//! - Count accumulation helpers

use std::ffi::c_int;

use crate::OapHandle;

// =============================================================================
// Command flag constants (from normal.c)
// =============================================================================

/// May need to get a second char
pub const NV_NCH: c_int = 0x01;
/// Get second char when no operator pending
pub const NV_NCH_NOP: c_int = 0x02 | NV_NCH;
/// Always get a second char
pub const NV_NCH_ALW: c_int = 0x04 | NV_NCH;
/// Second char needs language adjustment
pub const NV_LANG: c_int = 0x08;
/// May start selection
pub const NV_SS: c_int = 0x10;
/// May start selection with shift modifier
pub const NV_SSS: c_int = 0x20;
/// May stop selection without shift modifier
pub const NV_STS: c_int = 0x40;
/// Keep register name
pub const NV_KEEPREG: c_int = 0x80;
/// Invert horizontal movement
pub const NV_RL: c_int = 0x100;
/// Not allowed when text or curbuf locked
pub const NV_NCW: c_int = 0x200;

// =============================================================================
// Operator type constants (from ops.h)
// =============================================================================

/// No pending operation
pub const OP_NOP: c_int = 0;

// =============================================================================
// C accessor function declarations
// =============================================================================

extern "C" {
    fn nvim_get_nv_cmd_flags(idx: c_int) -> c_int;
    fn nvim_get_nv_cmd_arg(idx: c_int) -> c_int;
    fn nvim_oap_get_op_type_ptr(oap: OapHandle) -> c_int;
    fn nvim_get_VIsual_active() -> c_int;
    fn nvim_get_reg_recording() -> c_int;
    fn nvim_get_reg_executing() -> c_int;
}

// =============================================================================
// Command flag utilities
// =============================================================================

/// Get the command flags for the command at index.
///
/// Returns the NV_* flag combination for the given command index.
///
/// # Safety
/// `idx` must be a valid index returned by `rs_find_command`.
#[inline]
#[must_use]
pub fn get_cmd_flags(idx: c_int) -> c_int {
    unsafe { nvim_get_nv_cmd_flags(idx) }
}

/// Get the command argument for the command at index.
///
/// Returns the cmd_arg value for the given command index.
///
/// # Safety
/// `idx` must be a valid index returned by `rs_find_command`.
#[inline]
#[must_use]
pub fn get_cmd_arg(idx: c_int) -> c_int {
    unsafe { nvim_get_nv_cmd_arg(idx) }
}

/// Check if a command needs an additional character.
///
/// This function determines whether the command at `idx` with character `cmdchar`
/// requires reading an additional character based on:
/// - NV_NCH flag and its variants (NV_NCH_NOP, NV_NCH_ALW)
/// - Whether an operator is pending
/// - Special cases for 'q' (macro recording), 'a', and 'i' (text objects)
///
/// This is the Rust implementation of `normal_need_additional_char()` from normal.c.
///
/// # Arguments
/// * `idx` - Command index from `rs_find_command`
/// * `cmdchar` - The command character
/// * `pending_op` - Whether an operator is pending (oap->op_type != OP_NOP)
///
/// # Returns
/// `true` if an additional character is needed.
#[no_mangle]
pub extern "C" fn rs_need_additional_char(idx: c_int, cmdchar: c_int, pending_op: bool) -> bool {
    let flags = get_cmd_flags(idx);

    // Without NV_NCH we never need to check for an additional char
    if flags & NV_NCH == 0 {
        return false;
    }

    // NV_NCH_NOP is set and no operator is pending, get a second char
    if (flags & NV_NCH_NOP) == NV_NCH_NOP && !pending_op {
        return true;
    }

    // NV_NCH_ALW is set, always get a second char
    if (flags & NV_NCH_ALW) == NV_NCH_ALW {
        return true;
    }

    // 'q' without a pending operator, recording or executing a register,
    // needs to be followed by a second char
    if cmdchar == c_int::from(b'q') && !pending_op {
        let reg_recording = unsafe { nvim_get_reg_recording() };
        let reg_executing = unsafe { nvim_get_reg_executing() };
        if reg_recording == 0 && reg_executing == 0 {
            return true;
        }
    }

    // 'a' or 'i' after an operator is a text object
    // Also, don't do anything when these keys are received in visual mode
    if cmdchar == c_int::from(b'a') || cmdchar == c_int::from(b'i') {
        let visual_active = unsafe { nvim_get_VIsual_active() != 0 };
        if pending_op || visual_active {
            return true;
        }
    }

    false
}

/// Check if the command has NV_LANG flag set.
///
/// Commands with this flag need language adjustment for the second character.
#[no_mangle]
pub extern "C" fn rs_cmd_has_lang_flag(idx: c_int) -> bool {
    get_cmd_flags(idx) & NV_LANG != 0
}

/// Check if the command has NV_NCW flag set.
///
/// Commands with this flag are not allowed when text or curbuf is locked.
#[no_mangle]
pub extern "C" fn rs_cmd_has_ncw_flag(idx: c_int) -> bool {
    get_cmd_flags(idx) & NV_NCW != 0
}

/// Check if the command has NV_RL flag set.
///
/// Commands with this flag should have horizontal movements inverted
/// when 'rightleft' is set.
#[no_mangle]
pub extern "C" fn rs_cmd_has_rl_flag(idx: c_int) -> bool {
    get_cmd_flags(idx) & NV_RL != 0
}

/// Check if the command has NV_KEEPREG flag set.
///
/// Commands with this flag should keep the register name for next time.
#[no_mangle]
pub extern "C" fn rs_cmd_has_keepreg_flag(idx: c_int) -> bool {
    get_cmd_flags(idx) & NV_KEEPREG != 0
}

/// Check if the command has NV_SS flag set.
///
/// Commands with this flag may start selection.
#[no_mangle]
pub extern "C" fn rs_cmd_has_ss_flag(idx: c_int) -> bool {
    get_cmd_flags(idx) & NV_SS != 0
}

/// Check if the command has NV_SSS flag set.
///
/// Commands with this flag may start selection with shift modifier.
#[no_mangle]
pub extern "C" fn rs_cmd_has_sss_flag(idx: c_int) -> bool {
    get_cmd_flags(idx) & NV_SSS != 0
}

/// Check if the command has NV_STS flag set.
///
/// Commands with this flag may stop selection without shift modifier.
#[no_mangle]
pub extern "C" fn rs_cmd_has_sts_flag(idx: c_int) -> bool {
    get_cmd_flags(idx) & NV_STS != 0
}

// =============================================================================
// Count handling utilities
// =============================================================================

/// Multiply counts safely, capping at 999999999.
///
/// This implements the count multiplication logic from normal_execute():
/// - If count0 is 0, return opcount
/// - If opcount >= 999999999 / count0, return 999999999
/// - Otherwise return opcount * count0
#[no_mangle]
pub extern "C" fn rs_multiply_counts(opcount: c_int, count0: c_int) -> c_int {
    if count0 == 0 {
        return opcount;
    }
    if opcount >= 999_999_999 / count0 {
        return 999_999_999;
    }
    opcount * count0
}

/// Check if an operator is pending.
///
/// # Safety
/// `oap` must be a valid oparg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_is_operator_pending(oap: OapHandle) -> bool {
    nvim_oap_get_op_type_ptr(oap) != OP_NOP
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nv_flags_constants() {
        // Verify flag combinations
        assert_eq!(NV_NCH_NOP, 0x03); // 0x02 | 0x01
        assert_eq!(NV_NCH_ALW, 0x05); // 0x04 | 0x01
    }

    #[test]
    fn test_nv_nch_flag_detection() {
        // NV_NCH_NOP contains NV_NCH
        assert_ne!(NV_NCH_NOP & NV_NCH, 0);
        // NV_NCH_ALW contains NV_NCH
        assert_ne!(NV_NCH_ALW & NV_NCH, 0);
        // NV_LANG does not contain NV_NCH
        assert_eq!(NV_LANG & NV_NCH, 0);
    }

    #[test]
    fn test_multiply_counts_zero() {
        assert_eq!(rs_multiply_counts(5, 0), 5);
        assert_eq!(rs_multiply_counts(0, 0), 0);
    }

    #[test]
    fn test_multiply_counts_normal() {
        assert_eq!(rs_multiply_counts(3, 4), 12);
        assert_eq!(rs_multiply_counts(10, 5), 50);
        assert_eq!(rs_multiply_counts(1, 1), 1);
    }

    #[test]
    fn test_multiply_counts_overflow() {
        // Test overflow protection
        assert_eq!(rs_multiply_counts(999_999_999, 2), 999_999_999);
        assert_eq!(rs_multiply_counts(500_000_000, 3), 999_999_999);
        // Just under the threshold
        assert_eq!(rs_multiply_counts(333_333_333, 3), 999_999_999);
    }

    #[test]
    fn test_multiply_counts_large() {
        // Large but valid multiplication
        assert_eq!(rs_multiply_counts(100, 1_000_000), 100_000_000);
    }
}
