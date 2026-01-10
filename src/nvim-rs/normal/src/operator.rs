//! Operator-pending state machine helpers
//!
//! This module provides helpers for managing operator-pending mode,
//! including operator types, motion handling, and text object processing.

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::match_same_arms)]

use std::ffi::c_int;

// =============================================================================
// Operator Types
// =============================================================================

/// Normal mode operator types.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OpType {
    /// No operator pending
    #[default]
    None = 0,
    /// Delete operator (d)
    Delete = 1,
    /// Yank operator (y)
    Yank = 2,
    /// Change operator (c)
    Change = 3,
    /// Filter operator (!)
    Filter = 4,
    /// Format operator (gq)
    Format = 5,
    /// Indent left operator (<)
    IndentLeft = 6,
    /// Indent right operator (>)
    IndentRight = 7,
    /// Lowercase operator (gu)
    Lower = 8,
    /// Uppercase operator (gU)
    Upper = 9,
    /// Toggle case operator (g~)
    ToggleCase = 10,
    /// Fold operator (zf)
    Fold = 11,
    /// Function operator (g@)
    Function = 12,
    /// Replace operator (gr)
    Replace = 13,
}

impl OpType {
    /// Convert from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Delete,
            2 => Self::Yank,
            3 => Self::Change,
            4 => Self::Filter,
            5 => Self::Format,
            6 => Self::IndentLeft,
            7 => Self::IndentRight,
            8 => Self::Lower,
            9 => Self::Upper,
            10 => Self::ToggleCase,
            11 => Self::Fold,
            12 => Self::Function,
            13 => Self::Replace,
            _ => Self::None,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Get the operator character.
    #[must_use]
    pub const fn char(&self) -> u8 {
        match self {
            Self::None => 0,
            Self::Delete => b'd',
            Self::Yank => b'y',
            Self::Change => b'c',
            Self::Filter => b'!',
            Self::Format => b'q',
            Self::IndentLeft => b'<',
            Self::IndentRight => b'>',
            Self::Lower => b'u',
            Self::Upper => b'U',
            Self::ToggleCase => b'~',
            Self::Fold => b'f',
            Self::Function => b'@',
            Self::Replace => b'r',
        }
    }

    /// Check if operator deletes text.
    #[must_use]
    pub const fn is_delete(&self) -> bool {
        matches!(self, Self::Delete | Self::Change)
    }

    /// Check if operator changes text.
    #[must_use]
    pub const fn is_change(&self) -> bool {
        !matches!(self, Self::None | Self::Yank)
    }

    /// Check if operator is a case operator.
    #[must_use]
    pub const fn is_case(&self) -> bool {
        matches!(self, Self::Lower | Self::Upper | Self::ToggleCase)
    }

    /// Check if operator is an indent operator.
    #[must_use]
    pub const fn is_indent(&self) -> bool {
        matches!(self, Self::IndentLeft | Self::IndentRight)
    }
}

// =============================================================================
// Motion Types
// =============================================================================

/// Motion types that can follow an operator.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MotionType {
    /// Character-wise motion
    #[default]
    Char = 0,
    /// Line-wise motion
    Line = 1,
    /// Block-wise motion
    Block = 2,
}

impl MotionType {
    /// Convert from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Line,
            2 => Self::Block,
            _ => Self::Char,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Operator Flags
// =============================================================================

/// Flags for operator behavior.
pub mod op_flags {
    use std::ffi::c_int;

    /// Operator was forced character-wise (v)
    pub const OPF_CHAR: c_int = 0x01;
    /// Operator was forced line-wise (V)
    pub const OPF_LINE: c_int = 0x02;
    /// Operator was forced block-wise (CTRL-V)
    pub const OPF_BLOCK: c_int = 0x04;
    /// Include end position in operation
    pub const OPF_INCLUSIVE: c_int = 0x08;
    /// Operation is a change (needs undo)
    pub const OPF_CHANGE: c_int = 0x10;
    /// Motion was double operator (dd, yy, etc.)
    pub const OPF_DOUBLE: c_int = 0x20;
}

/// Check if operator flags have a specific flag set.
#[must_use]
#[inline]
pub const fn has_op_flag(flags: c_int, flag: c_int) -> bool {
    (flags & flag) != 0
}

/// Set an operator flag.
#[must_use]
#[inline]
pub const fn set_op_flag(flags: c_int, flag: c_int) -> c_int {
    flags | flag
}

/// Clear an operator flag.
#[must_use]
#[inline]
pub const fn clear_op_flag(flags: c_int, flag: c_int) -> c_int {
    flags & !flag
}

// =============================================================================
// Operator-Pending State
// =============================================================================

/// State for operator-pending mode.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct OpPendingState {
    /// Current operator type
    pub op_type: c_int,
    /// Motion type (char, line, block)
    pub motion_type: c_int,
    /// Operator flags
    pub flags: c_int,
    /// Count before operator
    pub count1: c_int,
    /// Count after operator (for motion)
    pub count2: c_int,
    /// Whether motion is inclusive
    pub inclusive: bool,
    /// Whether we're in the middle of an operator
    pub is_pending: bool,
}

impl OpPendingState {
    /// Create a new operator-pending state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            op_type: 0,
            motion_type: 0,
            flags: 0,
            count1: 0,
            count2: 0,
            inclusive: false,
            is_pending: false,
        }
    }

    /// Start a new operator.
    pub fn start(&mut self, op: OpType, count: c_int) {
        self.op_type = op.to_raw();
        self.count1 = count;
        self.count2 = 0;
        self.flags = 0;
        self.is_pending = true;
    }

    /// Get the operator type.
    #[must_use]
    pub const fn get_op_type(&self) -> OpType {
        OpType::from_raw(self.op_type)
    }

    /// Get the motion type.
    #[must_use]
    pub const fn get_motion_type(&self) -> MotionType {
        MotionType::from_raw(self.motion_type)
    }

    /// Get the effective count.
    #[must_use]
    pub const fn get_count(&self) -> c_int {
        if self.count1 > 1 && self.count2 > 1 {
            self.count1 * self.count2
        } else if self.count1 > 1 {
            self.count1
        } else if self.count2 > 1 {
            self.count2
        } else {
            1
        }
    }

    /// Clear the state.
    pub fn clear(&mut self) {
        *self = Self::new();
    }

    /// Force motion type.
    pub fn force_motion(&mut self, mtype: MotionType) {
        self.motion_type = mtype.to_raw();
        self.flags = match mtype {
            MotionType::Char => set_op_flag(self.flags, op_flags::OPF_CHAR),
            MotionType::Line => set_op_flag(self.flags, op_flags::OPF_LINE),
            MotionType::Block => set_op_flag(self.flags, op_flags::OPF_BLOCK),
        };
    }

    /// Check if motion was forced.
    #[must_use]
    pub const fn is_forced(&self) -> bool {
        has_op_flag(
            self.flags,
            op_flags::OPF_CHAR | op_flags::OPF_LINE | op_flags::OPF_BLOCK,
        )
    }
}

// =============================================================================
// Text Object Types
// =============================================================================

/// Text object types for operator-pending mode.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextObject {
    /// No text object
    #[default]
    None = 0,
    /// Word (w)
    Word = 1,
    /// WORD (W)
    BigWord = 2,
    /// Sentence (s)
    Sentence = 3,
    /// Paragraph (p)
    Paragraph = 4,
    /// Block/parentheses (b or ()
    Block = 5,
    /// Curly braces (B or {})
    Brace = 6,
    /// Square brackets ([])
    Bracket = 7,
    /// Angle brackets (<>)
    Angle = 8,
    /// Tag (<tag>)
    Tag = 9,
    /// Double quotes (")
    DoubleQuote = 10,
    /// Single quotes (')
    SingleQuote = 11,
    /// Backtick (`)
    Backtick = 12,
}

impl TextObject {
    /// Convert from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Word,
            2 => Self::BigWord,
            3 => Self::Sentence,
            4 => Self::Paragraph,
            5 => Self::Block,
            6 => Self::Brace,
            7 => Self::Bracket,
            8 => Self::Angle,
            9 => Self::Tag,
            10 => Self::DoubleQuote,
            11 => Self::SingleQuote,
            12 => Self::Backtick,
            _ => Self::None,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Get the text object character.
    #[must_use]
    pub const fn char(&self) -> u8 {
        match self {
            Self::None => 0,
            Self::Word => b'w',
            Self::BigWord => b'W',
            Self::Sentence => b's',
            Self::Paragraph => b'p',
            Self::Block => b'b',
            Self::Brace => b'B',
            Self::Bracket => b'[',
            Self::Angle => b'<',
            Self::Tag => b't',
            Self::DoubleQuote => b'"',
            Self::SingleQuote => b'\'',
            Self::Backtick => b'`',
        }
    }

    /// Check if this is a paired text object.
    #[must_use]
    pub const fn is_paired(&self) -> bool {
        matches!(
            self,
            Self::Block
                | Self::Brace
                | Self::Bracket
                | Self::Angle
                | Self::Tag
                | Self::DoubleQuote
                | Self::SingleQuote
                | Self::Backtick
        )
    }

    /// Get the closing character for paired text objects.
    #[must_use]
    pub const fn closing_char(&self) -> u8 {
        match self {
            Self::Block => b')',
            Self::Brace => b'}',
            Self::Bracket => b']',
            Self::Angle => b'>',
            Self::DoubleQuote => b'"',
            Self::SingleQuote => b'\'',
            Self::Backtick => b'`',
            _ => 0,
        }
    }
}

/// Text object selection type.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextObjectSelect {
    /// Inner text object (i)
    #[default]
    Inner = 0,
    /// A(n) text object (a)
    Outer = 1,
}

impl TextObjectSelect {
    /// Convert from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Outer,
            _ => Self::Inner,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Get operator type from raw value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_op_type(value: c_int) -> c_int {
    OpType::from_raw(value).to_raw()
}

/// Check if operator deletes text.
#[unsafe(no_mangle)]
pub extern "C" fn rs_op_is_delete(value: c_int) -> c_int {
    c_int::from(OpType::from_raw(value).is_delete())
}

/// Check if operator changes text.
#[unsafe(no_mangle)]
pub extern "C" fn rs_normal_op_is_change(value: c_int) -> c_int {
    c_int::from(OpType::from_raw(value).is_change())
}

/// Get motion type from raw value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_type(value: c_int) -> c_int {
    MotionType::from_raw(value).to_raw()
}

/// Get text object from raw value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_text_object(value: c_int) -> c_int {
    TextObject::from_raw(value).to_raw()
}

/// Check if text object is paired.
#[unsafe(no_mangle)]
pub extern "C" fn rs_text_object_is_paired(value: c_int) -> c_int {
    c_int::from(TextObject::from_raw(value).is_paired())
}

/// Check if operator flags have a specific flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_has_op_flag(flags: c_int, flag: c_int) -> c_int {
    c_int::from(has_op_flag(flags, flag))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_op_type() {
        assert_eq!(OpType::from_raw(0), OpType::None);
        assert_eq!(OpType::from_raw(1), OpType::Delete);
        assert_eq!(OpType::from_raw(2), OpType::Yank);
        assert_eq!(OpType::from_raw(3), OpType::Change);

        assert!(OpType::Delete.is_delete());
        assert!(OpType::Change.is_delete());
        assert!(!OpType::Yank.is_delete());

        assert!(OpType::Delete.is_change());
        assert!(!OpType::Yank.is_change());
        assert!(!OpType::None.is_change());

        assert!(OpType::Lower.is_case());
        assert!(OpType::Upper.is_case());
        assert!(OpType::ToggleCase.is_case());
        assert!(!OpType::Delete.is_case());
    }

    #[test]
    fn test_op_type_char() {
        assert_eq!(OpType::Delete.char(), b'd');
        assert_eq!(OpType::Yank.char(), b'y');
        assert_eq!(OpType::Change.char(), b'c');
        assert_eq!(OpType::IndentLeft.char(), b'<');
        assert_eq!(OpType::IndentRight.char(), b'>');
    }

    #[test]
    fn test_motion_type() {
        assert_eq!(MotionType::from_raw(0), MotionType::Char);
        assert_eq!(MotionType::from_raw(1), MotionType::Line);
        assert_eq!(MotionType::from_raw(2), MotionType::Block);
    }

    #[test]
    fn test_op_flags() {
        let flags = 0;
        assert!(!has_op_flag(flags, op_flags::OPF_CHAR));

        let flags = set_op_flag(flags, op_flags::OPF_CHAR);
        assert!(has_op_flag(flags, op_flags::OPF_CHAR));

        let flags = set_op_flag(flags, op_flags::OPF_INCLUSIVE);
        assert!(has_op_flag(flags, op_flags::OPF_CHAR));
        assert!(has_op_flag(flags, op_flags::OPF_INCLUSIVE));

        let flags = clear_op_flag(flags, op_flags::OPF_CHAR);
        assert!(!has_op_flag(flags, op_flags::OPF_CHAR));
        assert!(has_op_flag(flags, op_flags::OPF_INCLUSIVE));
    }

    #[test]
    fn test_op_pending_state() {
        let mut state = OpPendingState::new();
        assert!(!state.is_pending);
        assert_eq!(state.get_op_type(), OpType::None);

        state.start(OpType::Delete, 5);
        assert!(state.is_pending);
        assert_eq!(state.get_op_type(), OpType::Delete);
        assert_eq!(state.count1, 5);

        state.count2 = 3;
        assert_eq!(state.get_count(), 15); // 5 * 3

        state.force_motion(MotionType::Line);
        assert!(state.is_forced());
        assert_eq!(state.get_motion_type(), MotionType::Line);

        state.clear();
        assert!(!state.is_pending);
    }

    #[test]
    fn test_text_object() {
        assert_eq!(TextObject::from_raw(1), TextObject::Word);
        assert_eq!(TextObject::from_raw(5), TextObject::Block);

        assert!(!TextObject::Word.is_paired());
        assert!(TextObject::Block.is_paired());
        assert!(TextObject::DoubleQuote.is_paired());

        assert_eq!(TextObject::Block.closing_char(), b')');
        assert_eq!(TextObject::Brace.closing_char(), b'}');
        assert_eq!(TextObject::Bracket.closing_char(), b']');
    }

    #[test]
    fn test_text_object_select() {
        assert_eq!(TextObjectSelect::from_raw(0), TextObjectSelect::Inner);
        assert_eq!(TextObjectSelect::from_raw(1), TextObjectSelect::Outer);
    }
}
