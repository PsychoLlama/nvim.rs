//! Message attribute handling
//!
//! Provides utilities for managing message highlight attributes
//! and integrating with the highlight system.

use std::ffi::c_int;

// C accessor declarations
extern "C" {
    /// Convert highlight ID to attribute
    fn syn_id2attr(hl_id: c_int) -> c_int;
    /// Combine two attributes
    fn hl_combine_attr(a: c_int, b: c_int) -> c_int;
    /// hl_attr_active: pointer to the active highlight attribute table
    static mut hl_attr_active: *mut c_int;
}

/// Get HL_ATTR for a highlight field index.
///
/// Equivalent to `hl_attr_active[hlf]` in C.
///
/// # Safety
/// Reads from the hl_attr_active global array.
#[inline]
#[allow(clippy::cast_sign_loss)]
unsafe fn hl_attr(hlf: c_int) -> c_int {
    *hl_attr_active.add(hlf as usize)
}

/// Highlight field constants (mirrors HLF_* in C)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HlField(pub c_int);

impl HlField {
    /// Error messages (HLF_E)
    pub const ERROR: Self = Self(0);
    /// Warning messages
    pub const WARNING: Self = Self(1);
    /// Info/notice messages (HLF_N)
    pub const INFO: Self = Self(2);
    /// Question/prompt messages (HLF_R)
    pub const QUESTION: Self = Self(3);
    /// Title (HLF_T)
    pub const TITLE: Self = Self(4);
    /// Message area (HLF_MSG)
    pub const MSG: Self = Self(5);
    /// More prompt (HLF_M)
    pub const MORE: Self = Self(6);
    /// Mode message
    pub const MODE: Self = Self(7);
    /// Special characters (HLF_8)
    pub const SPECIAL: Self = Self(8);
    /// AT sign for truncation
    pub const AT: Self = Self(9);
}

/// Convert a highlight ID to display attribute.
///
/// # Safety
/// Calls C function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hl_id2attr(hl_id: c_int) -> c_int {
    if hl_id == 0 {
        return 0;
    }
    syn_id2attr(hl_id)
}

/// Combine message attribute with a base attribute.
///
/// The message area highlight is combined with the given attribute.
///
/// # Safety
/// Calls C functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_combine_attr(hl_id: c_int) -> c_int {
    let attr = if hl_id != 0 { syn_id2attr(hl_id) } else { 0 };
    let msg_attr = hl_attr(HlField::MSG.0);
    hl_combine_attr(msg_attr, attr)
}

/// Get the attribute for error messages.
///
/// # Safety
/// Calls C function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_error_attr() -> c_int {
    hl_attr(HlField::ERROR.0)
}

/// Get the attribute for warning messages.
///
/// # Safety
/// Calls C function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_warning_attr() -> c_int {
    hl_attr(HlField::WARNING.0)
}

/// Get the attribute for the message area.
///
/// # Safety
/// Calls C function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_area_attr() -> c_int {
    hl_attr(HlField::MSG.0)
}

/// Get the attribute for the "more" prompt.
///
/// # Safety
/// Calls C function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_more_attr() -> c_int {
    hl_attr(HlField::MORE.0)
}

/// Get the attribute for special characters.
///
/// # Safety
/// Calls C function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_special_attr() -> c_int {
    hl_attr(HlField::SPECIAL.0)
}

/// Get the attribute for question/prompt messages.
///
/// # Safety
/// Calls C function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_question_attr() -> c_int {
    hl_attr(HlField::QUESTION.0)
}

/// Get the attribute for title messages.
///
/// # Safety
/// Calls C function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_title_attr() -> c_int {
    hl_attr(HlField::TITLE.0)
}

#[cfg(test)]
mod tests {
    // Integration tests would require mocking C functions
}
