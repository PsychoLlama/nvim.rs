//! High-level sign operations
//!
//! This module provides the high-level API for sign operations:
//! - Sign placement (sign_place)
//! - Sign unplacement (sign_unplace)
//! - Sign jump (sign_jump)
//!
//! These functions orchestrate the lower-level placement and query functions.

use std::ffi::{c_char, c_int};

use crate::{LinenrT, SignHandle, SIGN_DEF_PRIO};

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    /// Get sign by name from the sign map
    fn nvim_sign_map_get(name: *const c_char) -> SignHandle;

    /// Get sign priority
    fn nvim_sign_get_priority(sp: SignHandle) -> c_int;
}

// =============================================================================
// Sign Place Operation
// =============================================================================

/// Result of a sign_place operation.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignPlaceOpResult {
    /// Success - sign was placed
    Ok = 0,
    /// Invalid group name (reserved character)
    InvalidGroup = 1,
    /// Unknown sign name
    UnknownSign = 2,
    /// Could not modify existing sign
    ModifyFailed = 3,
    /// General failure
    Failed = 4,
}

/// Parameters validated for sign_place operation.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SignPlaceOpParams {
    /// Sign definition handle
    pub sp: SignHandle,
    /// Effective priority
    pub priority: c_int,
    /// Whether this is a new placement (lnum > 0) or modification (lnum == 0)
    pub is_new_placement: bool,
}

impl std::fmt::Debug for SignPlaceOpParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignPlaceOpParams")
            .field("sp", &"SignHandle")
            .field("priority", &self.priority)
            .field("is_new_placement", &self.is_new_placement)
            .finish()
    }
}

/// Validate and prepare sign_place operation.
///
/// Returns the validated parameters for the operation, or an error.
///
/// # Safety
/// `group` and `name` must be null or valid null-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_place_prepare(
    group: *const c_char,
    name: *const c_char,
    lnum: LinenrT,
    prio: c_int,
) -> SignPlaceOpParams {
    // Default error result
    let error_result = SignPlaceOpParams {
        sp: SignHandle::null(),
        priority: SIGN_DEF_PRIO,
        is_new_placement: false,
    };

    // Name must be provided
    if name.is_null() {
        return error_result;
    }

    // Check for reserved character '*' in group name
    if !group.is_null() {
        let group_byte = *group.cast::<u8>();
        if group_byte == b'*' || group_byte == 0 {
            return error_result;
        }
    }

    // Look up sign definition
    let sp = nvim_sign_map_get(name);
    if sp.is_null() {
        return error_result;
    }

    // Calculate effective priority
    let sign_prio = nvim_sign_get_priority(sp);
    let effective_prio = if prio == -1 && sign_prio != -1 {
        if sign_prio == -1 {
            SIGN_DEF_PRIO
        } else {
            sign_prio
        }
    } else if prio == -1 {
        SIGN_DEF_PRIO
    } else {
        prio
    };

    SignPlaceOpParams {
        sp,
        priority: effective_prio,
        is_new_placement: lnum > 0,
    }
}

/// Check if sign_place preparation was successful.
///
/// # Safety
/// `params` must be null or a valid pointer to `SignPlaceOpParams`.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_place_params_valid(params: *const SignPlaceOpParams) -> c_int {
    if params.is_null() {
        return 0;
    }
    c_int::from(!(*params).sp.is_null())
}

// =============================================================================
// Sign Unplace Operation (Batch vs Single)
// =============================================================================

/// Batch mode for sign unplace operation.
///
/// Determines whether to delete a single sign or multiple signs.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignUnplaceBatch {
    /// Delete a single sign by ID
    Single = 0,
    /// Delete multiple signs (by line, group, or all)
    Multiple = 1,
}

/// Determine the unplace batch mode based on parameters.
#[no_mangle]
pub extern "C" fn rs_sign_unplace_batch_mode(
    id: c_int,
    atlnum: LinenrT,
    group_is_all: c_int,
) -> SignUnplaceBatch {
    if id == 0 || atlnum > 0 || group_is_all != 0 {
        SignUnplaceBatch::Multiple
    } else {
        SignUnplaceBatch::Single
    }
}

// =============================================================================
// Sign Jump Operation
// =============================================================================

/// Result of a sign_jump operation.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SignJumpResult {
    /// Line number (0 or -1 on error)
    pub lnum: LinenrT,
    /// Whether the sign was found
    pub found: bool,
    /// Whether the buffer has a name (needed for jumping to unopened buffers)
    pub buffer_has_name: bool,
}

/// Create a sign jump result for a found sign.
#[no_mangle]
pub extern "C" fn rs_sign_jump_found(lnum: LinenrT, buffer_has_name: c_int) -> SignJumpResult {
    SignJumpResult {
        lnum,
        found: true,
        buffer_has_name: buffer_has_name != 0,
    }
}

/// Create a sign jump result for a not-found sign.
#[no_mangle]
pub extern "C" fn rs_sign_jump_not_found() -> SignJumpResult {
    SignJumpResult {
        lnum: -1,
        found: false,
        buffer_has_name: false,
    }
}

/// Jump target type for sign_jump.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignJumpTarget {
    /// Jump within current window
    CurrentWindow = 0,
    /// Need to open buffer in new/existing window
    OpenBuffer = 1,
    /// Cannot jump - buffer has no name
    NoName = 2,
    /// Sign not found
    NotFound = 3,
}

/// Determine jump target type.
#[no_mangle]
pub extern "C" fn rs_sign_jump_target(
    sign_found: c_int,
    win_is_current: c_int,
    buffer_has_name: c_int,
) -> SignJumpTarget {
    if sign_found == 0 {
        SignJumpTarget::NotFound
    } else if win_is_current != 0 {
        SignJumpTarget::CurrentWindow
    } else if buffer_has_name != 0 {
        SignJumpTarget::OpenBuffer
    } else {
        SignJumpTarget::NoName
    }
}

// =============================================================================
// Sign Command Dispatching
// =============================================================================

/// Sign command to execute.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignCommand {
    /// Place a sign
    Place = 0,
    /// Unplace a sign
    Unplace = 1,
    /// Jump to a sign
    Jump = 2,
    /// Define a sign
    Define = 3,
    /// Undefine a sign
    Undefine = 4,
    /// List signs
    List = 5,
}

impl SignCommand {
    /// Convert from command index.
    pub const fn from_index(idx: c_int) -> Option<Self> {
        match idx {
            3 => Some(Self::Place),
            4 => Some(Self::Unplace),
            5 => Some(Self::Jump),
            0 => Some(Self::Define),
            1 => Some(Self::Undefine),
            2 => Some(Self::List),
            _ => None,
        }
    }
}

/// FFI export: Convert command index to SignCommand enum.
#[no_mangle]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub extern "C" fn rs_sign_cmd_from_index(idx: c_int) -> c_int {
    SignCommand::from_index(idx).map_or(-1, |cmd| cmd as c_int)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_place_op_result() {
        assert_eq!(SignPlaceOpResult::Ok as c_int, 0);
        assert_eq!(SignPlaceOpResult::InvalidGroup as c_int, 1);
        assert_eq!(SignPlaceOpResult::UnknownSign as c_int, 2);
    }

    #[test]
    fn test_sign_unplace_batch_mode() {
        // Single deletion
        assert_eq!(
            rs_sign_unplace_batch_mode(5, 0, 0),
            SignUnplaceBatch::Single
        );

        // Multiple deletion by line
        assert_eq!(
            rs_sign_unplace_batch_mode(0, 10, 0),
            SignUnplaceBatch::Multiple
        );

        // Multiple deletion all
        assert_eq!(
            rs_sign_unplace_batch_mode(0, 0, 1),
            SignUnplaceBatch::Multiple
        );

        // Multiple deletion id=0
        assert_eq!(
            rs_sign_unplace_batch_mode(0, 0, 0),
            SignUnplaceBatch::Multiple
        );
    }

    #[test]
    fn test_sign_jump_result_default() {
        let result = SignJumpResult::default();
        assert_eq!(result.lnum, 0);
        assert!(!result.found);
        assert!(!result.buffer_has_name);
    }

    #[test]
    fn test_sign_jump_found() {
        let result = rs_sign_jump_found(42, 1);
        assert_eq!(result.lnum, 42);
        assert!(result.found);
        assert!(result.buffer_has_name);
    }

    #[test]
    fn test_sign_jump_not_found() {
        let result = rs_sign_jump_not_found();
        assert_eq!(result.lnum, -1);
        assert!(!result.found);
    }

    #[test]
    fn test_sign_jump_target() {
        assert_eq!(rs_sign_jump_target(0, 0, 0), SignJumpTarget::NotFound);
        assert_eq!(rs_sign_jump_target(1, 1, 0), SignJumpTarget::CurrentWindow);
        assert_eq!(rs_sign_jump_target(1, 0, 1), SignJumpTarget::OpenBuffer);
        assert_eq!(rs_sign_jump_target(1, 0, 0), SignJumpTarget::NoName);
    }

    #[test]
    fn test_sign_command_from_index() {
        assert_eq!(SignCommand::from_index(0), Some(SignCommand::Define));
        assert_eq!(SignCommand::from_index(3), Some(SignCommand::Place));
        assert_eq!(SignCommand::from_index(5), Some(SignCommand::Jump));
        assert_eq!(SignCommand::from_index(99), None);
    }
}
