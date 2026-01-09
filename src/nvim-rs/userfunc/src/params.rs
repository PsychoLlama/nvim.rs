//! Parameter parsing and validation for user-defined functions.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]

use std::ffi::c_int;

// =============================================================================
// Parameter Types
// =============================================================================

/// Parameter type for function arguments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ParamType {
    /// Regular positional parameter
    Positional = 0,
    /// Optional parameter with default
    Optional = 1,
    /// Variadic parameter (...)
    Variadic = 2,
    /// Dictionary self parameter
    DictSelf = 3,
}

impl ParamType {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Optional,
            2 => Self::Variadic,
            3 => Self::DictSelf,
            _ => Self::Positional,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Parameter Info
// =============================================================================

/// Information about a single function parameter.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ParamInfo {
    /// Parameter type
    pub param_type: i32,
    /// Whether parameter has a type annotation
    pub has_type: bool,
    /// Expected type (if has_type is true)
    pub expected_type: i32,
    /// Default value index (-1 if no default)
    pub default_idx: i32,
}

impl Default for ParamInfo {
    fn default() -> Self {
        Self {
            param_type: ParamType::Positional as i32,
            has_type: false,
            expected_type: 0,
            default_idx: -1,
        }
    }
}

impl ParamInfo {
    /// Create a positional parameter.
    pub const fn positional() -> Self {
        Self {
            param_type: ParamType::Positional as i32,
            has_type: false,
            expected_type: 0,
            default_idx: -1,
        }
    }

    /// Create an optional parameter with default.
    pub const fn optional(default_idx: i32) -> Self {
        Self {
            param_type: ParamType::Optional as i32,
            has_type: false,
            expected_type: 0,
            default_idx,
        }
    }

    /// Create a variadic parameter.
    pub const fn variadic() -> Self {
        Self {
            param_type: ParamType::Variadic as i32,
            has_type: false,
            expected_type: 0,
            default_idx: -1,
        }
    }

    /// Check if parameter is optional.
    pub const fn is_optional(&self) -> bool {
        self.param_type == ParamType::Optional as i32
    }

    /// Check if parameter is variadic.
    pub const fn is_variadic(&self) -> bool {
        self.param_type == ParamType::Variadic as i32
    }

    /// Check if parameter has a default value.
    pub const fn has_default(&self) -> bool {
        self.default_idx >= 0
    }
}

// =============================================================================
// Parameter Name Validation
// =============================================================================

/// Check if a character is valid for a parameter name start.
pub const fn is_valid_param_name_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_'
}

/// Check if a character is valid in a parameter name.
pub const fn is_valid_param_name_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// Validate a parameter name.
pub fn is_valid_param_name(name: &[u8]) -> bool {
    if name.is_empty() {
        return false;
    }

    if !is_valid_param_name_start(name[0]) {
        return false;
    }

    name.iter().all(|&c| is_valid_param_name_char(c))
}

/// FFI export: validate parameter name.
///
/// # Safety
/// - `name` must be a valid pointer to `len` bytes, or null.
#[no_mangle]
pub unsafe extern "C" fn rs_userfunc_is_valid_param_name(name: *const u8, len: c_int) -> bool {
    if name.is_null() || len < 0 {
        return false;
    }

    let slice = unsafe { std::slice::from_raw_parts(name, len as usize) };
    is_valid_param_name(slice)
}

// =============================================================================
// Parameter List Parsing
// =============================================================================

/// Parse parameter count from signature.
///
/// Returns (min_args, max_args) where max_args is -1 for variadic.
pub fn parse_param_counts(params: &[ParamInfo]) -> (i32, i32) {
    let mut min_args = 0i32;
    let mut is_variadic = false;

    for param in params {
        match ParamType::from_c_int(param.param_type) {
            ParamType::Positional => min_args += 1,
            ParamType::Variadic => is_variadic = true,
            ParamType::Optional | ParamType::DictSelf => {} // doesn't increase min
        }
    }

    let total = params.iter().filter(|p| {
        let pt = ParamType::from_c_int(p.param_type);
        !matches!(pt, ParamType::DictSelf | ParamType::Variadic)
    }).count() as i32;

    let max_args = if is_variadic { -1 } else { total };

    (min_args, max_args)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_type() {
        assert_eq!(ParamType::from_c_int(0), ParamType::Positional);
        assert_eq!(ParamType::from_c_int(1), ParamType::Optional);
        assert_eq!(ParamType::from_c_int(2), ParamType::Variadic);
    }

    #[test]
    fn test_param_info() {
        let pos = ParamInfo::positional();
        assert!(!pos.is_optional());
        assert!(!pos.has_default());

        let opt = ParamInfo::optional(0);
        assert!(opt.is_optional());
        assert!(opt.has_default());

        let var = ParamInfo::variadic();
        assert!(var.is_variadic());
    }

    #[test]
    fn test_is_valid_param_name() {
        assert!(is_valid_param_name(b"foo"));
        assert!(is_valid_param_name(b"_bar"));
        assert!(is_valid_param_name(b"arg1"));

        assert!(!is_valid_param_name(b""));
        assert!(!is_valid_param_name(b"1foo"));
        assert!(!is_valid_param_name(b"foo-bar"));
    }

    #[test]
    fn test_parse_param_counts() {
        // Two required args
        let params = [ParamInfo::positional(), ParamInfo::positional()];
        assert_eq!(parse_param_counts(&params), (2, 2));

        // One required, one optional
        let params = [ParamInfo::positional(), ParamInfo::optional(0)];
        assert_eq!(parse_param_counts(&params), (1, 2));

        // One required, variadic
        let params = [ParamInfo::positional(), ParamInfo::variadic()];
        assert_eq!(parse_param_counts(&params), (1, -1));
    }
}
