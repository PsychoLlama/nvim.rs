//! VimL expression operator helpers
//!
//! This module provides helpers for VimL expression operators,
//! including precedence, associativity, and type compatibility.

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::c_int;

// =============================================================================
// Operator Types
// =============================================================================

/// VimL expression operator types.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Operator {
    /// Invalid/no operator
    #[default]
    Invalid = 0,
    /// Logical OR (||)
    Or = 1,
    /// Logical AND (&&)
    And = 2,
    /// Comparison equal (==)
    Equal = 3,
    /// Comparison not equal (!=)
    NotEqual = 4,
    /// Greater than (>)
    Greater = 5,
    /// Greater or equal (>=)
    GreaterEqual = 6,
    /// Less than (<)
    Less = 7,
    /// Less or equal (<=)
    LessEqual = 8,
    /// Match regex (=~)
    Match = 9,
    /// Not match regex (!~)
    NotMatch = 10,
    /// Is same value (is)
    Is = 11,
    /// Is not same value (isnot)
    IsNot = 12,
    /// Addition (+)
    Add = 13,
    /// Subtraction (-)
    Subtract = 14,
    /// String concatenation (.)
    Concat = 15,
    /// String concatenation (..)
    ConcatLiteral = 16,
    /// Multiplication (*)
    Multiply = 17,
    /// Division (/)
    Divide = 18,
    /// Modulo (%)
    Modulo = 19,
    /// Unary minus (-)
    Negate = 20,
    /// Unary plus (+)
    UnaryPlus = 21,
    /// Logical not (!)
    Not = 22,
    /// Ternary condition (?:)
    Ternary = 23,
    /// Subscript ([])
    Subscript = 24,
    /// Member access (.)
    Member = 25,
    /// Function call
    Call = 26,
}

impl Operator {
    /// Convert from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Or,
            2 => Self::And,
            3 => Self::Equal,
            4 => Self::NotEqual,
            5 => Self::Greater,
            6 => Self::GreaterEqual,
            7 => Self::Less,
            8 => Self::LessEqual,
            9 => Self::Match,
            10 => Self::NotMatch,
            11 => Self::Is,
            12 => Self::IsNot,
            13 => Self::Add,
            14 => Self::Subtract,
            15 => Self::Concat,
            16 => Self::ConcatLiteral,
            17 => Self::Multiply,
            18 => Self::Divide,
            19 => Self::Modulo,
            20 => Self::Negate,
            21 => Self::UnaryPlus,
            22 => Self::Not,
            23 => Self::Ternary,
            24 => Self::Subscript,
            25 => Self::Member,
            26 => Self::Call,
            _ => Self::Invalid,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Check if operator is a comparison operator.
    #[must_use]
    pub const fn is_comparison(&self) -> bool {
        matches!(
            self,
            Self::Equal
                | Self::NotEqual
                | Self::Greater
                | Self::GreaterEqual
                | Self::Less
                | Self::LessEqual
                | Self::Match
                | Self::NotMatch
                | Self::Is
                | Self::IsNot
        )
    }

    /// Check if operator is a logical operator.
    #[must_use]
    pub const fn is_logical(&self) -> bool {
        matches!(self, Self::Or | Self::And | Self::Not)
    }

    /// Check if operator is an arithmetic operator.
    #[must_use]
    pub const fn is_arithmetic(&self) -> bool {
        matches!(
            self,
            Self::Add
                | Self::Subtract
                | Self::Multiply
                | Self::Divide
                | Self::Modulo
                | Self::Negate
                | Self::UnaryPlus
        )
    }

    /// Check if operator is a unary operator.
    #[must_use]
    pub const fn is_unary(&self) -> bool {
        matches!(self, Self::Negate | Self::UnaryPlus | Self::Not)
    }

    /// Check if operator is a binary operator.
    #[must_use]
    pub const fn is_binary(&self) -> bool {
        !self.is_unary() && !matches!(self, Self::Invalid | Self::Ternary)
    }
}

// =============================================================================
// Operator Precedence
// =============================================================================

/// Precedence levels (higher = binds tighter).
pub mod precedence {
    use std::ffi::c_int;

    /// Lowest precedence (ternary)
    pub const TERNARY: c_int = 1;
    /// Logical OR
    pub const OR: c_int = 2;
    /// Logical AND
    pub const AND: c_int = 3;
    /// Comparison operators
    pub const COMPARISON: c_int = 4;
    /// Addition/subtraction/concatenation
    pub const ADDITIVE: c_int = 5;
    /// Multiplication/division/modulo
    pub const MULTIPLICATIVE: c_int = 6;
    /// Unary operators
    pub const UNARY: c_int = 7;
    /// Subscript/member/call
    pub const POSTFIX: c_int = 8;
}

/// Get the precedence level of an operator.
#[must_use]
pub const fn get_precedence(op: Operator) -> c_int {
    match op {
        Operator::Invalid => 0,
        Operator::Ternary => precedence::TERNARY,
        Operator::Or => precedence::OR,
        Operator::And => precedence::AND,
        Operator::Equal
        | Operator::NotEqual
        | Operator::Greater
        | Operator::GreaterEqual
        | Operator::Less
        | Operator::LessEqual
        | Operator::Match
        | Operator::NotMatch
        | Operator::Is
        | Operator::IsNot => precedence::COMPARISON,
        Operator::Add | Operator::Subtract | Operator::Concat | Operator::ConcatLiteral => {
            precedence::ADDITIVE
        }
        Operator::Multiply | Operator::Divide | Operator::Modulo => precedence::MULTIPLICATIVE,
        Operator::Negate | Operator::UnaryPlus | Operator::Not => precedence::UNARY,
        Operator::Subscript | Operator::Member | Operator::Call => precedence::POSTFIX,
    }
}

// =============================================================================
// Comparison Modifiers
// =============================================================================

/// Comparison modifier for case sensitivity.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CaseSensitivity {
    /// Use 'ignorecase' option
    #[default]
    Option = 0,
    /// Case-sensitive (suffix #)
    Sensitive = 1,
    /// Case-insensitive (suffix ?)
    Insensitive = 2,
}

impl CaseSensitivity {
    /// Convert from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Sensitive,
            2 => Self::Insensitive,
            _ => Self::Option,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }
}

/// Comparison operation with modifier.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ComparisonOp {
    /// The comparison operator
    pub op: c_int,
    /// Case sensitivity modifier
    pub case: c_int,
}

impl ComparisonOp {
    /// Create a new comparison operation.
    #[must_use]
    pub const fn new(op: Operator, case: CaseSensitivity) -> Self {
        Self {
            op: op.to_raw(),
            case: case.to_raw(),
        }
    }

    /// Get the operator.
    #[must_use]
    pub const fn get_op(&self) -> Operator {
        Operator::from_raw(self.op)
    }

    /// Get the case sensitivity.
    #[must_use]
    pub const fn get_case(&self) -> CaseSensitivity {
        CaseSensitivity::from_raw(self.case)
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Get operator from raw value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_eval_operator(value: c_int) -> c_int {
    Operator::from_raw(value).to_raw()
}

/// Check if operator is a comparison.
#[unsafe(no_mangle)]
pub extern "C" fn rs_eval_op_is_comparison(value: c_int) -> c_int {
    c_int::from(Operator::from_raw(value).is_comparison())
}

/// Check if operator is logical.
#[unsafe(no_mangle)]
pub extern "C" fn rs_eval_op_is_logical(value: c_int) -> c_int {
    c_int::from(Operator::from_raw(value).is_logical())
}

/// Check if operator is arithmetic.
#[unsafe(no_mangle)]
pub extern "C" fn rs_eval_op_is_arithmetic(value: c_int) -> c_int {
    c_int::from(Operator::from_raw(value).is_arithmetic())
}

/// Get operator precedence.
#[unsafe(no_mangle)]
pub extern "C" fn rs_eval_op_precedence(value: c_int) -> c_int {
    get_precedence(Operator::from_raw(value))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operator() {
        assert_eq!(Operator::from_raw(0), Operator::Invalid);
        assert_eq!(Operator::from_raw(1), Operator::Or);
        assert_eq!(Operator::from_raw(13), Operator::Add);
        assert_eq!(Operator::from_raw(99), Operator::Invalid);

        assert_eq!(Operator::Add.to_raw(), 13);
    }

    #[test]
    fn test_operator_classification() {
        assert!(Operator::Equal.is_comparison());
        assert!(Operator::NotEqual.is_comparison());
        assert!(Operator::Greater.is_comparison());
        assert!(!Operator::Add.is_comparison());

        assert!(Operator::Or.is_logical());
        assert!(Operator::And.is_logical());
        assert!(Operator::Not.is_logical());
        assert!(!Operator::Add.is_logical());

        assert!(Operator::Add.is_arithmetic());
        assert!(Operator::Multiply.is_arithmetic());
        assert!(Operator::Negate.is_arithmetic());
        assert!(!Operator::Equal.is_arithmetic());

        assert!(Operator::Not.is_unary());
        assert!(Operator::Negate.is_unary());
        assert!(!Operator::Add.is_unary());

        assert!(Operator::Add.is_binary());
        assert!(!Operator::Not.is_binary());
    }

    #[test]
    fn test_precedence() {
        assert!(get_precedence(Operator::Multiply) > get_precedence(Operator::Add));
        assert!(get_precedence(Operator::Add) > get_precedence(Operator::And));
        assert!(get_precedence(Operator::And) > get_precedence(Operator::Or));
        assert!(get_precedence(Operator::UnaryPlus) > get_precedence(Operator::Multiply));
    }

    #[test]
    fn test_case_sensitivity() {
        assert_eq!(CaseSensitivity::from_raw(0), CaseSensitivity::Option);
        assert_eq!(CaseSensitivity::from_raw(1), CaseSensitivity::Sensitive);
        assert_eq!(CaseSensitivity::from_raw(2), CaseSensitivity::Insensitive);

        assert_eq!(CaseSensitivity::Sensitive.to_raw(), 1);
    }

    #[test]
    fn test_comparison_op() {
        let cmp = ComparisonOp::new(Operator::Equal, CaseSensitivity::Sensitive);
        assert_eq!(cmp.get_op(), Operator::Equal);
        assert_eq!(cmp.get_case(), CaseSensitivity::Sensitive);
    }
}
