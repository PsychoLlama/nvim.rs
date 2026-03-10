//! Sign definition management
//!
//! This module handles creating, updating, and removing sign definitions.
//! Sign definitions specify the visual properties of signs (text, highlights, icons).

use std::ffi::{c_char, c_int, CStr};

use crate::{DecorSignHighlightHandle, SignHandle, SIGN_DEF_PRIO};

// =============================================================================
// C Accessor Extern Declarations
// =============================================================================

extern "C" {
    // Sign definition accessors
    fn nvim_sign_get_name(sp: SignHandle) -> *const c_char;
    fn nvim_sign_get_icon(sp: SignHandle) -> *const c_char;
    fn nvim_sign_get_text_hl(sp: SignHandle) -> c_int;
    fn nvim_sign_get_line_hl(sp: SignHandle) -> c_int;
    fn nvim_sign_get_num_hl(sp: SignHandle) -> c_int;
    fn nvim_sign_get_cul_hl(sp: SignHandle) -> c_int;
    fn nvim_sign_get_priority(sp: SignHandle) -> c_int;

    // Sign map operations
    fn nvim_sign_map_get(name: *const c_char) -> SignHandle;
    fn nvim_sign_map_has(name: *const c_char) -> c_int;
}

// =============================================================================
// Sign Definition Queries
// =============================================================================

/// Check if a sign with the given name is defined.
///
/// # Safety
///
/// `name` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_is_defined(name: *const c_char) -> bool {
    if name.is_null() {
        return false;
    }
    nvim_sign_map_has(name) != 0
}

/// Get a sign definition by name.
///
/// Returns a null handle if the sign is not defined.
///
/// # Safety
///
/// `name` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_get_by_name(name: *const c_char) -> SignHandle {
    if name.is_null() {
        return SignHandle::null();
    }
    nvim_sign_map_get(name)
}

// =============================================================================
// Sign Definition Properties
// =============================================================================

/// Get the name of a sign definition.
///
/// # Safety
///
/// `sp` must be a valid sign handle.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_def_get_name(sp: SignHandle) -> *const c_char {
    if sp.is_null() {
        return std::ptr::null();
    }
    nvim_sign_get_name(sp)
}

/// Get the icon path of a sign definition.
///
/// Returns null if no icon is set.
///
/// # Safety
///
/// `sp` must be a valid sign handle.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_def_get_icon(sp: SignHandle) -> *const c_char {
    if sp.is_null() {
        return std::ptr::null();
    }
    nvim_sign_get_icon(sp)
}

/// Get the text highlight ID of a sign definition.
///
/// # Safety
///
/// `sp` must be a valid sign handle.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_def_get_text_hl(sp: SignHandle) -> c_int {
    if sp.is_null() {
        return 0;
    }
    nvim_sign_get_text_hl(sp)
}

/// Get the line highlight ID of a sign definition.
///
/// # Safety
///
/// `sp` must be a valid sign handle.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_def_get_line_hl(sp: SignHandle) -> c_int {
    if sp.is_null() {
        return 0;
    }
    nvim_sign_get_line_hl(sp)
}

/// Get the number column highlight ID of a sign definition.
///
/// # Safety
///
/// `sp` must be a valid sign handle.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_def_get_num_hl(sp: SignHandle) -> c_int {
    if sp.is_null() {
        return 0;
    }
    nvim_sign_get_num_hl(sp)
}

/// Get the cursorline highlight ID of a sign definition.
///
/// # Safety
///
/// `sp` must be a valid sign handle.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_def_get_cul_hl(sp: SignHandle) -> c_int {
    if sp.is_null() {
        return 0;
    }
    nvim_sign_get_cul_hl(sp)
}

/// Get the priority of a sign definition.
///
/// Returns SIGN_DEF_PRIO if not explicitly set (-1) or if handle is null.
///
/// # Safety
///
/// `sp` must be a valid sign handle.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_def_get_priority(sp: SignHandle) -> c_int {
    if sp.is_null() {
        return SIGN_DEF_PRIO;
    }
    let prio = nvim_sign_get_priority(sp);
    if prio == -1 {
        SIGN_DEF_PRIO
    } else {
        prio
    }
}

/// Check if a sign definition has any highlight configured.
///
/// Returns true if text_hl, line_hl, num_hl, or cul_hl is set.
///
/// # Safety
///
/// `sp` must be a valid sign handle.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_def_has_highlight(sp: SignHandle) -> bool {
    if sp.is_null() {
        return false;
    }
    nvim_sign_get_text_hl(sp) > 0
        || nvim_sign_get_line_hl(sp) > 0
        || nvim_sign_get_num_hl(sp) > 0
        || nvim_sign_get_cul_hl(sp) > 0
}

// =============================================================================
// DecorSignHighlight Properties
// =============================================================================

extern "C" {
    fn nvim_decor_sh_get_priority(sh: DecorSignHighlightHandle) -> u16;
    fn nvim_decor_sh_get_hl_id(sh: DecorSignHighlightHandle) -> c_int;
    fn nvim_decor_sh_get_sign_name(sh: DecorSignHighlightHandle) -> *const c_char;
    fn nvim_decor_sh_get_sign_add_id(sh: DecorSignHighlightHandle) -> c_int;
}

/// Get the priority of a placed sign (from DecorSignHighlight).
///
/// # Safety
///
/// `sh` must be a valid DecorSignHighlight handle.
#[no_mangle]
pub unsafe extern "C" fn rs_decor_sh_get_priority(sh: DecorSignHighlightHandle) -> c_int {
    if sh.is_null() {
        return SIGN_DEF_PRIO;
    }
    c_int::from(nvim_decor_sh_get_priority(sh))
}

/// Get the sign name from a placed sign (from DecorSignHighlight).
///
/// # Safety
///
/// `sh` must be a valid DecorSignHighlight handle.
#[no_mangle]
pub unsafe extern "C" fn rs_decor_sh_get_sign_name(sh: DecorSignHighlightHandle) -> *const c_char {
    if sh.is_null() {
        return std::ptr::null();
    }
    nvim_decor_sh_get_sign_name(sh)
}

/// Get the text highlight ID from a placed sign.
///
/// # Safety
///
/// `sh` must be a valid DecorSignHighlight handle.
#[no_mangle]
pub unsafe extern "C" fn rs_decor_sh_get_hl_id(sh: DecorSignHighlightHandle) -> c_int {
    if sh.is_null() {
        return 0;
    }
    nvim_decor_sh_get_hl_id(sh)
}

/// Get the sign_add_id from a placed sign (used for sorting recency).
///
/// # Safety
///
/// `sh` must be a valid DecorSignHighlight handle.
#[no_mangle]
pub unsafe extern "C" fn rs_decor_sh_get_sign_add_id(sh: DecorSignHighlightHandle) -> c_int {
    if sh.is_null() {
        return 0;
    }
    nvim_decor_sh_get_sign_add_id(sh)
}

// =============================================================================
// Sign Definition Parameters
// =============================================================================

/// Parameters for defining or updating a sign.
///
/// This structure holds all the parameters that can be specified when
/// defining a sign via `:sign define` or `sign_define()`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SignDefineParams {
    /// Sign name (required)
    pub name: *const c_char,
    /// Icon file path (optional)
    pub icon: *const c_char,
    /// Sign text (optional, max 2 display cells)
    pub text: *const c_char,
    /// Line highlight group name (optional)
    pub linehl: *const c_char,
    /// Text highlight group name (optional)
    pub texthl: *const c_char,
    /// Cursorline highlight group name (optional)
    pub culhl: *const c_char,
    /// Number column highlight group name (optional)
    pub numhl: *const c_char,
    /// Default priority (-1 for default)
    pub priority: c_int,
}

impl Default for SignDefineParams {
    fn default() -> Self {
        Self {
            name: std::ptr::null(),
            icon: std::ptr::null(),
            text: std::ptr::null(),
            linehl: std::ptr::null(),
            texthl: std::ptr::null(),
            culhl: std::ptr::null(),
            numhl: std::ptr::null(),
            priority: -1,
        }
    }
}

impl SignDefineParams {
    /// Check if the params have a valid name.
    ///
    /// # Safety
    /// All string pointers must be valid or null.
    pub unsafe fn has_valid_name(&self) -> bool {
        if self.name.is_null() {
            return false;
        }
        let cstr = CStr::from_ptr(self.name);
        !cstr.to_bytes().is_empty()
    }

    /// Check if any highlight is specified.
    pub const fn has_highlight(&self) -> bool {
        !self.linehl.is_null()
            || !self.texthl.is_null()
            || !self.culhl.is_null()
            || !self.numhl.is_null()
    }

    /// Check if text is specified.
    pub const fn has_text(&self) -> bool {
        !self.text.is_null()
    }

    /// Check if icon is specified.
    pub const fn has_icon(&self) -> bool {
        !self.icon.is_null()
    }
}

/// FFI export: Create default sign define params.
#[no_mangle]
pub extern "C" fn rs_sign_define_params_default() -> SignDefineParams {
    SignDefineParams::default()
}

/// FFI export: Check if sign define params have a valid name.
///
/// # Safety
/// All string pointers in `params` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_define_params_valid(params: *const SignDefineParams) -> c_int {
    if params.is_null() {
        return 0;
    }
    c_int::from((*params).has_valid_name())
}

// =============================================================================
// Sign Definition Validation
// =============================================================================

/// Validation result for sign definition.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignDefineError {
    /// No error
    Ok = 0,
    /// Sign name is missing or empty
    MissingName = 1,
    /// Sign name is invalid (bad characters)
    InvalidName = 2,
    /// Sign text is invalid (non-printable or too wide)
    InvalidText = 3,
    /// Priority is invalid (negative, except -1)
    InvalidPriority = 4,
}

/// Validate sign definition parameters.
///
/// Returns `SignDefineError::Ok` if all parameters are valid.
///
/// # Safety
/// All string pointers in `params` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_define_validate(
    params: *const SignDefineParams,
) -> SignDefineError {
    if params.is_null() {
        return SignDefineError::MissingName;
    }

    let p = &*params;

    // Check name
    if p.name.is_null() {
        return SignDefineError::MissingName;
    }
    let name_cstr = CStr::from_ptr(p.name);
    let name_bytes = name_cstr.to_bytes();
    if name_bytes.is_empty() {
        return SignDefineError::MissingName;
    }

    // Validate name characters (first must be letter or underscore)
    if let Some(&first) = name_bytes.first() {
        if !first.is_ascii_alphabetic() && first != b'_' {
            return SignDefineError::InvalidName;
        }
    }
    // Rest must be alphanumeric or underscore
    for &b in &name_bytes[1..] {
        if !b.is_ascii_alphanumeric() && b != b'_' {
            return SignDefineError::InvalidName;
        }
    }

    // Validate priority
    if p.priority < -1 {
        return SignDefineError::InvalidPriority;
    }

    // Text validation is done by init_sign_text in C
    // We can't fully validate it here without calling C functions

    SignDefineError::Ok
}

// =============================================================================
// Highlight Group Resolution
// =============================================================================

extern "C" {
    /// Look up a highlight group by name, creating if necessary.
    fn syn_check_group(name: *const c_char, len: usize) -> c_int;

    // Composite C accessors for sign definition management
    fn nvim_sign_define_by_name_impl(
        name: *const c_char,
        icon: *const c_char,
        text: *const c_char,
        linehl: *const c_char,
        texthl: *const c_char,
        culhl: *const c_char,
        numhl: *const c_char,
        prio: c_int,
    ) -> c_int;
    fn nvim_sign_undefine_by_name_impl(name: *const c_char) -> c_int;
    fn nvim_sign_free_all_impl();
}

/// Resolve a highlight group name to its ID.
///
/// Returns 0 if the name is null, empty, or invalid.
///
/// # Safety
/// `name` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_resolve_highlight(name: *const c_char) -> c_int {
    if name.is_null() {
        return 0;
    }

    let cstr = CStr::from_ptr(name);
    let bytes = cstr.to_bytes();
    if bytes.is_empty() {
        return 0;
    }

    syn_check_group(name, bytes.len())
}

/// Resolve all highlight groups for a sign definition.
///
/// Returns a struct with resolved highlight IDs.
///
/// # Safety
/// All string pointers in `params` must be valid or null.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SignHighlights {
    /// Text highlight ID
    pub text_hl: c_int,
    /// Line highlight ID
    pub line_hl: c_int,
    /// Number column highlight ID
    pub num_hl: c_int,
    /// Cursorline highlight ID
    pub cul_hl: c_int,
}

/// FFI export: Resolve all sign highlights at once.
///
/// # Safety
/// All string pointers must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_resolve_highlights(
    texthl: *const c_char,
    linehl: *const c_char,
    numhl: *const c_char,
    culhl: *const c_char,
) -> SignHighlights {
    SignHighlights {
        text_hl: rs_sign_resolve_highlight(texthl),
        line_hl: rs_sign_resolve_highlight(linehl),
        num_hl: rs_sign_resolve_highlight(numhl),
        cul_hl: rs_sign_resolve_highlight(culhl),
    }
}

// =============================================================================
// Core Sign Definition Management
// =============================================================================

/// Define a new sign or update an existing sign.
///
/// Returns OK (1) on success, FAIL (0) on failure.
///
/// # Safety
/// All string pointers must be valid null-terminated C strings or null.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_define_by_name(
    name: *const c_char,
    icon: *const c_char,
    text: *const c_char,
    linehl: *const c_char,
    texthl: *const c_char,
    culhl: *const c_char,
    numhl: *const c_char,
    prio: c_int,
) -> c_int {
    nvim_sign_define_by_name_impl(name, icon, text, linehl, texthl, culhl, numhl, prio)
}

/// Undefine a sign by name.
///
/// Returns OK (1) on success, FAIL (0) if not found.
/// Does NOT emit an error message — caller handles that.
///
/// # Safety
/// `name` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_undefine_by_name(name: *const c_char) -> c_int {
    nvim_sign_undefine_by_name_impl(name)
}

/// Undefine a sign by name with E155 error message on failure.
///
/// This replaces `sign_undefine_by_name()` in sign.c.
///
/// # Safety
/// `name` must be a valid null-terminated C string.
#[unsafe(export_name = "sign_undefine_by_name")]
pub unsafe extern "C" fn rs_sign_undefine_by_name_wrapper(name: *const c_char) -> c_int {
    extern "C" {
        fn semsg(s: *const c_char, ...);
    }
    let result = nvim_sign_undefine_by_name_impl(name);
    if result == 0 {
        // FAIL — emit E155
        static E155_FMT: &[u8] = b"E155: Unknown sign: %s\0";
        semsg(E155_FMT.as_ptr().cast(), name);
    }
    result
}

/// Free all sign definitions.
///
/// # Safety
/// Must be called during cleanup only.
#[unsafe(export_name = "free_signs")]
pub unsafe extern "C" fn rs_free_signs() {
    nvim_sign_free_all_impl();
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_handle_null() {
        let handle = SignHandle::null();
        assert!(handle.is_null());
    }

    #[test]
    fn test_decor_sh_handle_null() {
        let handle = DecorSignHighlightHandle::null();
        assert!(handle.is_null());
    }

    #[test]
    fn test_sign_define_params_default() {
        let params = SignDefineParams::default();
        assert!(params.name.is_null());
        assert!(params.text.is_null());
        assert!(params.icon.is_null());
        assert_eq!(params.priority, -1);
    }

    #[test]
    fn test_sign_define_params_has_highlight() {
        let params = SignDefineParams::default();
        assert!(!params.has_highlight());
        assert!(!params.has_text());
        assert!(!params.has_icon());
    }

    #[test]
    fn test_sign_define_error() {
        assert_eq!(SignDefineError::Ok as c_int, 0);
        assert_eq!(SignDefineError::MissingName as c_int, 1);
        assert_eq!(SignDefineError::InvalidName as c_int, 2);
        assert_eq!(SignDefineError::InvalidText as c_int, 3);
        assert_eq!(SignDefineError::InvalidPriority as c_int, 4);
    }

    #[test]
    fn test_sign_highlights_default() {
        let hl = SignHighlights::default();
        assert_eq!(hl.text_hl, 0);
        assert_eq!(hl.line_hl, 0);
        assert_eq!(hl.num_hl, 0);
        assert_eq!(hl.cul_hl, 0);
    }
}
