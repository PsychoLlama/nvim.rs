#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical definitions, hoisted out of the per-module copies c2rust emitted.
// One definition per logical name; every module imports from here.

/// Operator ids — upstream's anonymous enum in `ops.h`, whose order must
/// match `opchars` in `ops/optype.rs`: a row's index in that table *is* the
/// operator's number, and [`OpType::ALL`] is what turns one back.
///
/// The discriminants are written out because that correspondence is the
/// whole design, and because `oparg_T::op_type` is a `repr(C)` field.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum OpType {
    /// no pending operation
    Nop = 0,
    /// `d` — delete
    Delete = 1,
    /// `y` — yank
    Yank = 2,
    /// `c` — change
    Change = 3,
    /// `<` — left shift
    Lshift = 4,
    /// `>` — right shift
    Rshift = 5,
    /// `!` — filter
    Filter = 6,
    /// `g~` — switch case
    Tilde = 7,
    /// `=` — indent
    Indent = 8,
    /// `gq` — format
    Format = 9,
    /// `:` — colon
    Colon = 10,
    /// `gU` — upper case
    Upper = 11,
    /// `gu` — lower case
    Lower = 12,
    /// `J` — join, Visual mode only
    Join = 13,
    /// `gJ` — join without spaces, Visual mode only
    JoinNs = 14,
    /// `g?` — rot-13
    Rot13 = 15,
    /// `r` — replace chars, Visual mode only
    Replace = 16,
    /// `I` — insert column, Visual mode only
    Insert = 17,
    /// `A` — append column, Visual mode only
    Append = 18,
    /// `zf` — define a fold
    Fold = 19,
    /// `zo` — open folds
    Foldopen = 20,
    /// `zO` — open folds recursively
    Foldopenrec = 21,
    /// `zc` — close folds
    Foldclose = 22,
    /// `zC` — close folds recursively
    Foldcloserec = 23,
    /// `zd` — delete folds
    Folddel = 24,
    /// `zD` — delete folds recursively
    Folddelrec = 25,
    /// `gw` — format, keeping the cursor position
    Format2 = 26,
    /// `g@` — call `'operatorfunc'`
    Function = 27,
    /// `CTRL-A` — add to the number or alphabetic character
    NrAdd = 28,
    /// `CTRL-X` — subtract from the number or alphabetic character
    NrSub = 29,
}

impl OpType {
    /// Every operator, in `opchars` order, so that a row index names one.
    pub const ALL: [OpType; 30] = [
        OpType::Nop,
        OpType::Delete,
        OpType::Yank,
        OpType::Change,
        OpType::Lshift,
        OpType::Rshift,
        OpType::Filter,
        OpType::Tilde,
        OpType::Indent,
        OpType::Format,
        OpType::Colon,
        OpType::Upper,
        OpType::Lower,
        OpType::Join,
        OpType::JoinNs,
        OpType::Rot13,
        OpType::Replace,
        OpType::Insert,
        OpType::Append,
        OpType::Fold,
        OpType::Foldopen,
        OpType::Foldopenrec,
        OpType::Foldclose,
        OpType::Foldcloserec,
        OpType::Folddel,
        OpType::Folddelrec,
        OpType::Format2,
        OpType::Function,
        OpType::NrAdd,
        OpType::NrSub,
    ];
}
