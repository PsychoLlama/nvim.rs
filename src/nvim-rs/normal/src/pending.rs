//! Operator-pending state machine and motion handling
//!
//! This module provides state machine helpers for operator-pending mode,
//! including the logic for determining when an operator is pending,
//! what motion is expected, and how to execute pending operators.

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(dead_code)]

use std::ffi::c_int;

use crate::types::{CmdargT, OpargT};

/// Typed handle to command arguments (`cmdarg_T*`).
pub type CapHandle = *mut CmdargT;

/// Typed handle to operator arguments (`oparg_T*`).
pub type OapHandle = *mut OpargT;

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    fn nvim_cap_get_oap(cap: CapHandle) -> OapHandle;
    fn nvim_oap_get_op_type_ptr(oap: OapHandle) -> c_int;
    fn nvim_oap_set_op_type(oap: OapHandle, val: c_int);
    fn nvim_oap_get_motion_type(oap: OapHandle) -> c_int;
    fn nvim_oap_set_motion_type(oap: OapHandle, val: c_int);
    fn nvim_oap_get_inclusive(oap: OapHandle) -> bool;
    fn nvim_oap_set_inclusive(oap: OapHandle, val: bool);
    fn nvim_oap_get_motion_force(oap: OapHandle) -> c_int;
    fn nvim_cap_get_count0(cap: CapHandle) -> c_int;
    fn nvim_cap_get_count1(cap: CapHandle) -> c_int;
}

// =============================================================================
// Operator Type Constants (matching ops.h)
// =============================================================================

/// Operator types from ops.h
pub mod op_types {
    use std::ffi::c_int;

    /// No operation pending
    pub const OP_NOP: c_int = 0;
    /// Delete operator
    pub const OP_DELETE: c_int = 1;
    /// Yank operator
    pub const OP_YANK: c_int = 2;
    /// Change operator
    pub const OP_CHANGE: c_int = 3;
    /// Insert after cursor
    pub const OP_LSHIFT: c_int = 4;
    /// Insert before cursor
    pub const OP_RSHIFT: c_int = 5;
    /// Filter operator
    pub const OP_FILTER: c_int = 6;
    /// Toggle case
    pub const OP_TILDE: c_int = 7;
    /// Indent operator
    pub const OP_INDENT: c_int = 8;
    /// Format operator
    pub const OP_FORMAT: c_int = 9;
    /// C-indent operator
    pub const OP_COLON: c_int = 10;
    /// Make uppercase
    pub const OP_UPPER: c_int = 11;
    /// Make lowercase
    pub const OP_LOWER: c_int = 12;
    /// Join lines
    pub const OP_JOIN: c_int = 13;
    /// Join lines without spaces
    pub const OP_JOIN_NS: c_int = 14;
    /// ROT13 encode
    pub const OP_ROT13: c_int = 15;
    /// Replace characters
    pub const OP_REPLACE: c_int = 16;
    /// Insert operator
    pub const OP_INSERT: c_int = 17;
    /// Append operator
    pub const OP_APPEND: c_int = 18;
    /// Create fold
    pub const OP_FOLD: c_int = 19;
    /// Open folds
    pub const OP_FOLDOPEN: c_int = 20;
    /// Open folds recursively
    pub const OP_FOLDOPENREC: c_int = 21;
    /// Close folds
    pub const OP_FOLDCLOSE: c_int = 22;
    /// Close folds recursively
    pub const OP_FOLDCLOSEREC: c_int = 23;
    /// Delete folds
    pub const OP_FOLDDEL: c_int = 24;
    /// Delete folds recursively
    pub const OP_FOLDDELREC: c_int = 25;
    /// Format with gw
    pub const OP_FORMAT2: c_int = 26;
    /// Call operatorfunc
    pub const OP_FUNCTION: c_int = 27;
    /// Add to number
    pub const OP_NR_ADD: c_int = 28;
    /// Subtract from number
    pub const OP_NR_SUB: c_int = 29;
}

// =============================================================================
// Motion Type Constants (matching vim.h)
// =============================================================================

/// Motion type constants
pub mod motion_types {
    use std::ffi::c_int;

    /// Character-wise motion
    pub const MCHAR: c_int = 0;
    /// Line-wise motion
    pub const MLINE: c_int = 1;
    /// Block-wise motion
    pub const MBLOCK: c_int = 2;
}

// =============================================================================
// Pending State
// =============================================================================

/// State of pending operations
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PendingState {
    /// Whether an operator is pending
    pub op_pending: bool,
    /// The pending operator type
    pub op_type: c_int,
    /// Motion type (char, line, block)
    pub motion_type: c_int,
    /// Whether motion is inclusive
    pub inclusive: bool,
    /// Motion force character (v, V, Ctrl-V, or NUL)
    pub motion_force: c_int,
    /// Operator count
    pub op_count: c_int,
    /// Total count (opcount * count1)
    pub total_count: c_int,
}

impl PendingState {
    /// Create a new pending state (no operation pending)
    #[must_use]
    pub const fn new() -> Self {
        Self {
            op_pending: false,
            op_type: op_types::OP_NOP,
            motion_type: motion_types::MCHAR,
            inclusive: false,
            motion_force: 0,
            op_count: 0,
            total_count: 1,
        }
    }

    /// Check if delete is pending
    #[must_use]
    pub const fn is_delete_pending(&self) -> bool {
        self.op_pending && self.op_type == op_types::OP_DELETE
    }

    /// Check if yank is pending
    #[must_use]
    pub const fn is_yank_pending(&self) -> bool {
        self.op_pending && self.op_type == op_types::OP_YANK
    }

    /// Check if change is pending
    #[must_use]
    pub const fn is_change_pending(&self) -> bool {
        self.op_pending && self.op_type == op_types::OP_CHANGE
    }

    /// Check if this is a line-wise motion
    #[must_use]
    pub const fn is_linewise(&self) -> bool {
        self.motion_type == motion_types::MLINE
    }

    /// Check if this is a block-wise motion
    #[must_use]
    pub const fn is_blockwise(&self) -> bool {
        self.motion_type == motion_types::MBLOCK
    }
}

/// Get the pending state from operator arguments.
///
/// # Safety
///
/// Calls external C functions. `oap` must be valid.
#[must_use]
pub unsafe fn get_pending_state(oap: OapHandle) -> PendingState {
    if oap.is_null() {
        return PendingState::new();
    }

    let op_type = (*oap).op_type;
    let op_pending = op_type != op_types::OP_NOP;

    PendingState {
        op_pending,
        op_type,
        motion_type: (*oap).motion_type,
        inclusive: (*oap).inclusive,
        motion_force: (*oap).motion_force,
        op_count: 0, // Would need cap to get this
        total_count: 1,
    }
}

/// Get the pending state from command arguments.
///
/// # Safety
///
/// Calls external C functions. `cap` must be valid.
#[must_use]
pub unsafe fn get_pending_state_from_cap(cap: CapHandle) -> PendingState {
    if cap.is_null() {
        return PendingState::new();
    }

    let oap = (*cap).oap;
    let mut state = get_pending_state(oap);

    state.op_count = (*cap.cast::<CmdargT>()).opcount;
    let count1 = (*cap).count1;
    state.total_count = if state.op_count > 0 {
        state.op_count * count1
    } else {
        count1
    };

    state
}

// =============================================================================
// Motion Force Handling
// =============================================================================

/// Motion force characters
pub mod motion_force {
    use std::ffi::c_int;

    /// No motion force
    pub const NONE: c_int = 0;
    /// Character-wise force (v)
    pub const CHAR: c_int = b'v' as c_int;
    /// Line-wise force (V)
    pub const LINE: c_int = b'V' as c_int;
    /// Block-wise force (Ctrl-V = 0x16)
    pub const BLOCK: c_int = 0x16;
}

/// Determine the motion type with force applied.
///
/// The motion force character (v, V, Ctrl-V) can override the default
/// motion type of a command.
#[must_use]
pub const fn apply_motion_force(
    default_type: c_int,
    force: c_int,
    inclusive: bool,
) -> (c_int, bool) {
    match force {
        motion_force::CHAR => {
            // v forces character-wise, toggles inclusive
            (motion_types::MCHAR, !inclusive)
        }
        motion_force::LINE => {
            // V forces line-wise
            (motion_types::MLINE, inclusive)
        }
        motion_force::BLOCK => {
            // Ctrl-V forces block-wise
            (motion_types::MBLOCK, inclusive)
        }
        _ => {
            // No force, use default
            (default_type, inclusive)
        }
    }
}

// =============================================================================
// Operator Categories
// =============================================================================

/// Check if operator type is a delete-like operator.
#[must_use]
pub const fn is_delete_like(op_type: c_int) -> bool {
    matches!(op_type, op_types::OP_DELETE | op_types::OP_CHANGE)
}

/// Check if operator type is a yank-like operator.
#[must_use]
pub const fn is_yank_like(op_type: c_int) -> bool {
    op_type == op_types::OP_YANK
}

/// Check if operator type modifies text.
#[must_use]
pub const fn modifies_text(op_type: c_int) -> bool {
    matches!(
        op_type,
        op_types::OP_DELETE
            | op_types::OP_CHANGE
            | op_types::OP_LSHIFT
            | op_types::OP_RSHIFT
            | op_types::OP_FILTER
            | op_types::OP_TILDE
            | op_types::OP_INDENT
            | op_types::OP_FORMAT
            | op_types::OP_UPPER
            | op_types::OP_LOWER
            | op_types::OP_JOIN
            | op_types::OP_JOIN_NS
            | op_types::OP_ROT13
            | op_types::OP_REPLACE
            | op_types::OP_INSERT
            | op_types::OP_APPEND
            | op_types::OP_FORMAT2
            | op_types::OP_FUNCTION
            | op_types::OP_NR_ADD
            | op_types::OP_NR_SUB
    )
}

/// Check if operator type is a fold operation.
#[must_use]
pub const fn is_fold_op(op_type: c_int) -> bool {
    matches!(
        op_type,
        op_types::OP_FOLD
            | op_types::OP_FOLDOPEN
            | op_types::OP_FOLDOPENREC
            | op_types::OP_FOLDCLOSE
            | op_types::OP_FOLDCLOSEREC
            | op_types::OP_FOLDDEL
            | op_types::OP_FOLDDELREC
    )
}

/// Get the character representation of an operator.
#[must_use]
pub const fn op_char(op_type: c_int) -> u8 {
    match op_type {
        op_types::OP_DELETE => b'd',
        op_types::OP_YANK => b'y',
        op_types::OP_CHANGE => b'c',
        op_types::OP_LSHIFT => b'<',
        op_types::OP_RSHIFT => b'>',
        op_types::OP_FILTER => b'!',
        op_types::OP_TILDE => b'~',
        op_types::OP_FORMAT | op_types::OP_FORMAT2 => b'q',
        op_types::OP_UPPER => b'U',
        op_types::OP_LOWER => b'u',
        op_types::OP_JOIN | op_types::OP_JOIN_NS => b'J',
        op_types::OP_ROT13 => b'?',
        op_types::OP_REPLACE => b'r',
        op_types::OP_FOLD => b'f',
        op_types::OP_FUNCTION => b'@',
        _ => 0,
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Check if operator is pending for given oap.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_is_op_pending(oap: OapHandle) -> c_int {
    if oap.is_null() {
        return 0;
    }
    c_int::from((*oap).op_type != op_types::OP_NOP)
}

/// Get pending operator type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_get_pending_op_type(oap: OapHandle) -> c_int {
    if oap.is_null() {
        return op_types::OP_NOP;
    }
    (*oap).op_type
}

/// Check if delete is pending.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_delete_pending(oap: OapHandle) -> c_int {
    if oap.is_null() {
        return 0;
    }
    c_int::from((*oap).op_type == op_types::OP_DELETE)
}

/// Check if change is pending.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_change_pending(oap: OapHandle) -> c_int {
    if oap.is_null() {
        return 0;
    }
    c_int::from((*oap).op_type == op_types::OP_CHANGE)
}

/// Check if yank is pending.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_yank_pending(oap: OapHandle) -> c_int {
    if oap.is_null() {
        return 0;
    }
    c_int::from((*oap).op_type == op_types::OP_YANK)
}

/// Apply motion force to get effective motion type.
#[unsafe(no_mangle)]
pub extern "C" fn rs_apply_motion_force(
    default_type: c_int,
    force: c_int,
    inclusive: c_int,
) -> c_int {
    let (motion_type, _) = apply_motion_force(default_type, force, inclusive != 0);
    motion_type
}

/// Check if operator modifies text.
#[unsafe(no_mangle)]
pub extern "C" fn rs_op_modifies_text(op_type: c_int) -> c_int {
    c_int::from(modifies_text(op_type))
}

/// Check if operator is fold operation.
#[unsafe(no_mangle)]
pub extern "C" fn rs_op_is_fold(op_type: c_int) -> c_int {
    c_int::from(is_fold_op(op_type))
}

/// Get operator character.
#[unsafe(no_mangle)]
pub extern "C" fn rs_op_char(op_type: c_int) -> c_int {
    op_char(op_type) as c_int
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pending_state_new() {
        let state = PendingState::new();
        assert!(!state.op_pending);
        assert_eq!(state.op_type, op_types::OP_NOP);
        assert_eq!(state.motion_type, motion_types::MCHAR);
        assert!(!state.inclusive);
    }

    #[test]
    fn test_motion_force_application() {
        // No force
        let (mt, inc) = apply_motion_force(motion_types::MCHAR, motion_force::NONE, false);
        assert_eq!(mt, motion_types::MCHAR);
        assert!(!inc);

        // v force toggles inclusive
        let (mt, inc) = apply_motion_force(motion_types::MLINE, motion_force::CHAR, false);
        assert_eq!(mt, motion_types::MCHAR);
        assert!(inc);

        // V force makes line-wise
        let (mt, _) = apply_motion_force(motion_types::MCHAR, motion_force::LINE, false);
        assert_eq!(mt, motion_types::MLINE);

        // Ctrl-V force makes block-wise
        let (mt, _) = apply_motion_force(motion_types::MCHAR, motion_force::BLOCK, false);
        assert_eq!(mt, motion_types::MBLOCK);
    }

    #[test]
    fn test_operator_categories() {
        assert!(is_delete_like(op_types::OP_DELETE));
        assert!(is_delete_like(op_types::OP_CHANGE));
        assert!(!is_delete_like(op_types::OP_YANK));

        assert!(is_yank_like(op_types::OP_YANK));
        assert!(!is_yank_like(op_types::OP_DELETE));

        assert!(modifies_text(op_types::OP_DELETE));
        assert!(modifies_text(op_types::OP_CHANGE));
        assert!(!modifies_text(op_types::OP_YANK));
        assert!(!modifies_text(op_types::OP_NOP));

        assert!(is_fold_op(op_types::OP_FOLD));
        assert!(is_fold_op(op_types::OP_FOLDOPEN));
        assert!(!is_fold_op(op_types::OP_DELETE));
    }

    #[test]
    fn test_op_char() {
        assert_eq!(op_char(op_types::OP_DELETE), b'd');
        assert_eq!(op_char(op_types::OP_YANK), b'y');
        assert_eq!(op_char(op_types::OP_CHANGE), b'c');
        assert_eq!(op_char(op_types::OP_NOP), 0);
    }
}
