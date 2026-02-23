//! VimL type system helpers
//!
//! This module provides helpers for VimL types,
//! including type checking, coercion, and compatibility.

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::unnested_or_patterns)]

use std::ffi::c_int;

// =============================================================================
// VimL Value Types
// =============================================================================

/// VimL value types.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VarType {
    /// Unknown/uninitialized type
    #[default]
    Unknown = 0,
    /// Number (integer)
    Number = 1,
    /// String
    String = 2,
    /// Funcref (function reference)
    Func = 3,
    /// List
    List = 4,
    /// Dictionary
    Dict = 5,
    /// Float
    Float = 6,
    /// Boolean (v:true, v:false)
    Bool = 7,
    /// Special values (v:null, etc.)
    Special = 8,
    /// Blob (byte array)
    Blob = 9,
}

impl VarType {
    /// Convert from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Number,
            2 => Self::String,
            3 => Self::Func,
            4 => Self::List,
            5 => Self::Dict,
            6 => Self::Float,
            7 => Self::Bool,
            8 => Self::Special,
            9 => Self::Blob,
            _ => Self::Unknown,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Check if type is numeric (number or float).
    #[must_use]
    pub const fn is_numeric(&self) -> bool {
        matches!(self, Self::Number | Self::Float)
    }

    /// Check if type is a container (list, dict, blob).
    #[must_use]
    pub const fn is_container(&self) -> bool {
        matches!(self, Self::List | Self::Dict | Self::Blob)
    }

    /// Check if type is callable (funcref).
    #[must_use]
    pub const fn is_callable(&self) -> bool {
        matches!(self, Self::Func)
    }

    /// Check if type supports string conversion.
    #[must_use]
    pub const fn is_stringifiable(&self) -> bool {
        matches!(
            self,
            Self::Number | Self::String | Self::Float | Self::Bool | Self::Special
        )
    }

    /// Check if type can be used in arithmetic.
    #[must_use]
    pub const fn is_arithmetic(&self) -> bool {
        matches!(self, Self::Number | Self::Float)
    }
}

// =============================================================================
// Type Coercion
// =============================================================================

/// Result of type coercion.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoercionResult {
    /// Coercion succeeded
    Ok = 0,
    /// Types are incompatible
    Incompatible = 1,
    /// Value out of range for target type
    OutOfRange = 2,
    /// Precision lost in conversion
    PrecisionLoss = 3,
}

impl CoercionResult {
    /// Check if coercion succeeded.
    #[must_use]
    pub const fn is_ok(&self) -> bool {
        matches!(self, Self::Ok)
    }
}

/// Check if a type can be coerced to another type.
#[must_use]
pub const fn can_coerce(from: VarType, to: VarType) -> bool {
    if from.to_raw() == to.to_raw() {
        return true;
    }

    match (from, to) {
        // Number can be coerced to float
        (VarType::Number, VarType::Float) => true,
        // Float can be coerced to number (with data loss)
        (VarType::Float, VarType::Number) => true,
        // Bool can be coerced to number
        (VarType::Bool, VarType::Number) => true,
        // String to number (with parsing)
        (VarType::String, VarType::Number) => true,
        // String to float (with parsing)
        (VarType::String, VarType::Float) => true,
        // Most types can be coerced to string
        (_, VarType::String) => from.is_stringifiable(),
        // All types can be coerced to bool
        (_, VarType::Bool) => true,
        _ => false,
    }
}

/// Get the common type for two types in a binary operation.
#[must_use]
pub const fn common_type(a: VarType, b: VarType) -> VarType {
    let a_raw = a.to_raw();
    let b_raw = b.to_raw();

    if a_raw == b_raw {
        return a;
    }

    // Float takes precedence over number
    let float_raw = VarType::Float.to_raw();
    let number_raw = VarType::Number.to_raw();
    if (a_raw == float_raw && b_raw == number_raw) || (a_raw == number_raw && b_raw == float_raw) {
        return VarType::Float;
    }

    // For string operations, result is string
    let string_raw = VarType::String.to_raw();
    if a_raw == string_raw || b_raw == string_raw {
        return VarType::String;
    }

    VarType::Unknown
}

// =============================================================================
// Type Flags
// =============================================================================

/// Flags for type checking behavior.
pub mod type_flags {
    use std::ffi::c_int;

    /// Allow conversion to string
    pub const TF_TO_STRING: c_int = 0x01;
    /// Allow conversion to number
    pub const TF_TO_NUMBER: c_int = 0x02;
    /// Allow conversion to float
    pub const TF_TO_FLOAT: c_int = 0x04;
    /// Allow conversion to bool
    pub const TF_TO_BOOL: c_int = 0x08;
    /// Strict type checking (no coercion)
    pub const TF_STRICT: c_int = 0x10;
}

/// Check if type flags allow a conversion.
#[must_use]
pub const fn flags_allow(flags: c_int, to: VarType) -> bool {
    match to {
        VarType::String => (flags & type_flags::TF_TO_STRING) != 0,
        VarType::Number => (flags & type_flags::TF_TO_NUMBER) != 0,
        VarType::Float => (flags & type_flags::TF_TO_FLOAT) != 0,
        VarType::Bool => (flags & type_flags::TF_TO_BOOL) != 0,
        _ => false,
    }
}

// =============================================================================
// Type Compatibility
// =============================================================================

/// Check type compatibility for assignment.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TypeCompat {
    /// Whether assignment is allowed
    pub allowed: bool,
    /// Whether coercion is needed
    pub needs_coercion: bool,
    /// Whether there might be data loss
    pub data_loss: bool,
}

impl TypeCompat {
    /// Create a compatible result.
    #[must_use]
    pub const fn compatible() -> Self {
        Self {
            allowed: true,
            needs_coercion: false,
            data_loss: false,
        }
    }

    /// Create a result requiring coercion.
    #[must_use]
    pub const fn with_coercion(data_loss: bool) -> Self {
        Self {
            allowed: true,
            needs_coercion: true,
            data_loss,
        }
    }

    /// Create an incompatible result.
    #[must_use]
    pub const fn incompatible() -> Self {
        Self {
            allowed: false,
            needs_coercion: false,
            data_loss: false,
        }
    }
}

/// Check type compatibility for assignment.
#[must_use]
pub const fn check_type_compat(from: VarType, to: VarType) -> TypeCompat {
    if from.to_raw() == to.to_raw() {
        return TypeCompat::compatible();
    }

    if can_coerce(from, to) {
        // Check for potential data loss
        let data_loss = matches!(
            (from, to),
            (VarType::Float, VarType::Number) | (VarType::String, VarType::Number)
        );
        return TypeCompat::with_coercion(data_loss);
    }

    TypeCompat::incompatible()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_var_type() {
        assert_eq!(VarType::from_raw(0), VarType::Unknown);
        assert_eq!(VarType::from_raw(1), VarType::Number);
        assert_eq!(VarType::from_raw(4), VarType::List);
        assert_eq!(VarType::from_raw(99), VarType::Unknown);

        assert_eq!(VarType::Number.to_raw(), 1);
        assert_eq!(VarType::String.to_raw(), 2);
    }

    #[test]
    fn test_type_classification() {
        assert!(VarType::Number.is_numeric());
        assert!(VarType::Float.is_numeric());
        assert!(!VarType::String.is_numeric());

        assert!(VarType::List.is_container());
        assert!(VarType::Dict.is_container());
        assert!(VarType::Blob.is_container());
        assert!(!VarType::Number.is_container());

        assert!(VarType::Func.is_callable());
        assert!(!VarType::String.is_callable());

        assert!(VarType::Number.is_stringifiable());
        assert!(VarType::String.is_stringifiable());
        assert!(VarType::Bool.is_stringifiable());
        assert!(!VarType::List.is_stringifiable());
    }

    #[test]
    fn test_coercion() {
        // Same type always coercible
        assert!(can_coerce(VarType::Number, VarType::Number));
        assert!(can_coerce(VarType::String, VarType::String));

        // Number to float
        assert!(can_coerce(VarType::Number, VarType::Float));

        // Bool to number
        assert!(can_coerce(VarType::Bool, VarType::Number));

        // String to number
        assert!(can_coerce(VarType::String, VarType::Number));

        // To string
        assert!(can_coerce(VarType::Number, VarType::String));
        assert!(can_coerce(VarType::Float, VarType::String));

        // To bool
        assert!(can_coerce(VarType::Number, VarType::Bool));
        assert!(can_coerce(VarType::String, VarType::Bool));

        // Incompatible
        assert!(!can_coerce(VarType::List, VarType::Number));
        assert!(!can_coerce(VarType::Dict, VarType::Float));
    }

    #[test]
    fn test_common_type() {
        assert_eq!(
            common_type(VarType::Number, VarType::Number),
            VarType::Number
        );
        assert_eq!(common_type(VarType::Number, VarType::Float), VarType::Float);
        assert_eq!(common_type(VarType::Float, VarType::Number), VarType::Float);
        assert_eq!(
            common_type(VarType::String, VarType::Number),
            VarType::String
        );
    }

    #[test]
    fn test_type_compat() {
        let compat = check_type_compat(VarType::Number, VarType::Number);
        assert!(compat.allowed);
        assert!(!compat.needs_coercion);

        let compat = check_type_compat(VarType::Number, VarType::Float);
        assert!(compat.allowed);
        assert!(compat.needs_coercion);
        assert!(!compat.data_loss);

        let compat = check_type_compat(VarType::Float, VarType::Number);
        assert!(compat.allowed);
        assert!(compat.needs_coercion);
        assert!(compat.data_loss);

        let compat = check_type_compat(VarType::List, VarType::Number);
        assert!(!compat.allowed);
    }
}
