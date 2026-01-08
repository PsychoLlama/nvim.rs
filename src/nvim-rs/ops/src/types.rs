//! Type definitions for operator system
//!
//! This module defines the core types used by the operator system,
//! including operator types, motion types, and block definitions.

use std::ffi::{c_char, c_int};

/// Operator type IDs - must match ops.h definitions!
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i32)]
pub enum OpType {
    /// No pending operation
    #[default]
    Nop = 0,
    /// "d" delete operator
    Delete = 1,
    /// "y" yank operator
    Yank = 2,
    /// "c" change operator
    Change = 3,
    /// "<" left shift operator
    LShift = 4,
    /// ">" right shift operator
    RShift = 5,
    /// "!" filter operator
    Filter = 6,
    /// "g~" switch case operator
    Tilde = 7,
    /// "=" indent operator
    Indent = 8,
    /// "gq" format operator
    Format = 9,
    /// ":" colon operator
    Colon = 10,
    /// "gU" make upper case operator
    Upper = 11,
    /// "gu" make lower case operator
    Lower = 12,
    /// "J" join operator, only for Visual mode
    Join = 13,
    /// "gJ" join operator without spaces, only for Visual mode
    JoinNs = 14,
    /// "g?" rot-13 encoding
    Rot13 = 15,
    /// "r" replace chars, only for Visual mode
    Replace = 16,
    /// "I" Insert column, only for Visual mode
    Insert = 17,
    /// "A" Append column, only for Visual mode
    Append = 18,
    /// "zf" define a fold
    Fold = 19,
    /// "zo" open folds
    FoldOpen = 20,
    /// "zO" open folds recursively
    FoldOpenRec = 21,
    /// "zc" close folds
    FoldClose = 22,
    /// "zC" close folds recursively
    FoldCloseRec = 23,
    /// "zd" delete folds
    FoldDel = 24,
    /// "zD" delete folds recursively
    FoldDelRec = 25,
    /// "gw" format operator, keeps cursor pos
    Format2 = 26,
    /// "g@" call 'operatorfunc'
    Function = 27,
    /// "<C-A>" Add to the number or alphabetic character
    NrAdd = 28,
    /// "<C-X>" Subtract from the number or alphabetic character
    NrSub = 29,
}

impl OpType {
    /// Convert from raw c_int, returns None for invalid values
    #[inline]
    #[must_use]
    pub fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Nop),
            1 => Some(Self::Delete),
            2 => Some(Self::Yank),
            3 => Some(Self::Change),
            4 => Some(Self::LShift),
            5 => Some(Self::RShift),
            6 => Some(Self::Filter),
            7 => Some(Self::Tilde),
            8 => Some(Self::Indent),
            9 => Some(Self::Format),
            10 => Some(Self::Colon),
            11 => Some(Self::Upper),
            12 => Some(Self::Lower),
            13 => Some(Self::Join),
            14 => Some(Self::JoinNs),
            15 => Some(Self::Rot13),
            16 => Some(Self::Replace),
            17 => Some(Self::Insert),
            18 => Some(Self::Append),
            19 => Some(Self::Fold),
            20 => Some(Self::FoldOpen),
            21 => Some(Self::FoldOpenRec),
            22 => Some(Self::FoldClose),
            23 => Some(Self::FoldCloseRec),
            24 => Some(Self::FoldDel),
            25 => Some(Self::FoldDelRec),
            26 => Some(Self::Format2),
            27 => Some(Self::Function),
            28 => Some(Self::NrAdd),
            29 => Some(Self::NrSub),
            _ => None,
        }
    }

    /// Convert to raw c_int
    #[inline]
    #[must_use]
    pub const fn as_raw(self) -> c_int {
        self as c_int
    }

    /// Check if this operator always works on whole lines
    #[inline]
    #[must_use]
    pub const fn on_lines(self) -> bool {
        matches!(
            self,
            Self::LShift
                | Self::RShift
                | Self::Filter
                | Self::Indent
                | Self::Format
                | Self::Colon
                | Self::Join
                | Self::JoinNs
                | Self::FoldOpen
                | Self::FoldOpenRec
                | Self::FoldClose
                | Self::FoldCloseRec
                | Self::FoldDel
                | Self::FoldDelRec
                | Self::Format2
        )
    }

    /// Check if this operator changes text
    #[inline]
    #[must_use]
    pub const fn is_change(self) -> bool {
        matches!(
            self,
            Self::Delete
                | Self::Change
                | Self::LShift
                | Self::RShift
                | Self::Filter
                | Self::Tilde
                | Self::Indent
                | Self::Format
                | Self::Upper
                | Self::Lower
                | Self::Join
                | Self::JoinNs
                | Self::Rot13
                | Self::Replace
                | Self::Insert
                | Self::Append
                | Self::Format2
                | Self::Function
                | Self::NrAdd
                | Self::NrSub
        )
    }
}

/// Motion types, used for operators and for yank/delete registers.
///
/// The three valid numerical values must not be changed, as they
/// are used in external communication and serialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i32)]
pub enum MotionType {
    /// Character-wise movement/register
    #[default]
    CharWise = 0,
    /// Line-wise movement/register
    LineWise = 1,
    /// Block-wise movement/register
    BlockWise = 2,
    /// Unknown or invalid motion type
    Unknown = -1,
}

impl MotionType {
    /// Convert from raw c_int
    #[inline]
    #[must_use]
    pub fn from_raw(value: c_int) -> Self {
        match value {
            0 => Self::CharWise,
            1 => Self::LineWise,
            2 => Self::BlockWise,
            _ => Self::Unknown,
        }
    }

    /// Convert to raw c_int
    #[inline]
    #[must_use]
    pub const fn as_raw(self) -> c_int {
        self as c_int
    }
}

/// Block operation metadata.
///
/// Contains information about a block of text for block-wise operations.
/// This mirrors the C `struct block_def` in register_defs.h.
#[derive(Debug, Clone, Default)]
pub struct BlockDef {
    /// 'extra' cols before first char (spaces to fill)
    pub startspaces: c_int,
    /// 'extra' cols after last char (spaces to fill)
    pub endspaces: c_int,
    /// chars in block
    pub textlen: c_int,
    /// pointer to 1st char (partially) in block
    pub textstart: *const c_char,
    /// index of chars (partially) in block
    pub textcol: c_int,
    /// start col of 1st char wholly inside block
    pub start_vcol: c_int,
    /// start col of 1st char wholly after block
    pub end_vcol: c_int,
    /// true if line is too short to fit in block
    pub is_short: bool,
    /// true if curswant==MAXCOL when starting
    pub is_max: bool,
    /// true if block within one character
    pub is_one_char: bool,
    /// screen cols of ws before block
    pub pre_whitesp: c_int,
    /// chars of ws before block
    pub pre_whitesp_c: c_int,
    /// number of vcols of post-block char
    pub end_char_vcols: c_int,
    /// number of vcols of pre-block char
    pub start_char_vcols: c_int,
}

impl BlockDef {
    /// Create a new empty BlockDef
    #[must_use]
    pub const fn new() -> Self {
        Self {
            startspaces: 0,
            endspaces: 0,
            textlen: 0,
            textstart: std::ptr::null(),
            textcol: 0,
            start_vcol: 0,
            end_vcol: 0,
            is_short: false,
            is_max: false,
            is_one_char: false,
            pre_whitesp: 0,
            pre_whitesp_c: 0,
            end_char_vcols: 0,
            start_char_vcols: 0,
        }
    }

    /// Reset all fields to default values
    pub fn reset(&mut self) {
        self.startspaces = 0;
        self.endspaces = 0;
        self.textlen = 0;
        self.textstart = std::ptr::null();
        self.textcol = 0;
        self.start_vcol = 0;
        self.end_vcol = 0;
        self.is_short = false;
        self.is_max = false;
        self.is_one_char = false;
        self.pre_whitesp = 0;
        self.pre_whitesp_c = 0;
        self.end_char_vcols = 0;
        self.start_char_vcols = 0;
    }

    /// Get the total size including spaces for padding
    #[inline]
    #[must_use]
    pub const fn total_size(&self) -> c_int {
        self.startspaces + self.textlen + self.endspaces
    }
}

/// Position in a buffer (line number and column)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(C)]
pub struct Pos {
    /// Line number (1-based)
    pub lnum: c_int,
    /// Column (0-based byte offset)
    pub col: c_int,
    /// Column offset for 'virtualedit'
    pub coladd: c_int,
}

impl Pos {
    /// Create a new position
    #[inline]
    #[must_use]
    pub const fn new(lnum: c_int, col: c_int) -> Self {
        Self {
            lnum,
            col,
            coladd: 0,
        }
    }

    /// Create position with coladd
    #[inline]
    #[must_use]
    pub const fn with_coladd(lnum: c_int, col: c_int, coladd: c_int) -> Self {
        Self { lnum, col, coladd }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_op_type_from_raw() {
        assert_eq!(OpType::from_raw(0), Some(OpType::Nop));
        assert_eq!(OpType::from_raw(1), Some(OpType::Delete));
        assert_eq!(OpType::from_raw(2), Some(OpType::Yank));
        assert_eq!(OpType::from_raw(29), Some(OpType::NrSub));
        assert_eq!(OpType::from_raw(30), None);
        assert_eq!(OpType::from_raw(-1), None);
    }

    #[test]
    fn test_op_type_as_raw() {
        assert_eq!(OpType::Nop.as_raw(), 0);
        assert_eq!(OpType::Delete.as_raw(), 1);
        assert_eq!(OpType::NrSub.as_raw(), 29);
    }

    #[test]
    fn test_op_type_on_lines() {
        assert!(OpType::LShift.on_lines());
        assert!(OpType::RShift.on_lines());
        assert!(OpType::Join.on_lines());
        assert!(OpType::Colon.on_lines());
        assert!(!OpType::Delete.on_lines());
        assert!(!OpType::Yank.on_lines());
        assert!(!OpType::Tilde.on_lines());
    }

    #[test]
    fn test_op_type_is_change() {
        assert!(OpType::Delete.is_change());
        assert!(OpType::Change.is_change());
        assert!(OpType::Tilde.is_change());
        assert!(!OpType::Yank.is_change());
        assert!(!OpType::Nop.is_change());
        assert!(!OpType::Colon.is_change());
    }

    #[test]
    fn test_motion_type_from_raw() {
        assert_eq!(MotionType::from_raw(0), MotionType::CharWise);
        assert_eq!(MotionType::from_raw(1), MotionType::LineWise);
        assert_eq!(MotionType::from_raw(2), MotionType::BlockWise);
        assert_eq!(MotionType::from_raw(-1), MotionType::Unknown);
        assert_eq!(MotionType::from_raw(99), MotionType::Unknown);
    }

    #[test]
    fn test_block_def_new() {
        let bd = BlockDef::new();
        assert_eq!(bd.startspaces, 0);
        assert_eq!(bd.endspaces, 0);
        assert_eq!(bd.textlen, 0);
        assert!(bd.textstart.is_null());
        assert!(!bd.is_short);
        assert!(!bd.is_one_char);
    }

    #[test]
    fn test_block_def_total_size() {
        let mut bd = BlockDef::new();
        bd.startspaces = 2;
        bd.textlen = 10;
        bd.endspaces = 3;
        assert_eq!(bd.total_size(), 15);
    }

    #[test]
    fn test_pos_new() {
        let pos = Pos::new(10, 5);
        assert_eq!(pos.lnum, 10);
        assert_eq!(pos.col, 5);
        assert_eq!(pos.coladd, 0);
    }
}
