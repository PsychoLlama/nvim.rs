//! The operator vocabulary: which keys name an operator, and what it does
//! to a region.
//!
//! [`OPCHARS`] is the table upstream keeps in lock-step with the [`OpType`]
//! order,
//! one row per operator: the first character typed, the optional second (`g~`,
//! `zf`, `g@`) and two flags. Everything else in this file reads one column of
//! it. [`get_op_type`] is the reverse lookup normal mode does on the keys it
//! just read, with five special cases (`r`, `~`, `g CTRL-A`, `g CTRL-X`, `zy`)
//! the table cannot express, because those keys mean an operator only in
//! Visual mode and something else outside it.
//!
//! The table was a `static` in C and is a `const` here: nothing has ever
//! written to it, and the row order *is* the [`OpType`] numbering.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::keycodes::{Ctrl_A, Ctrl_X};
use crate::message::internal_error;
use crate::types::OpType;

use core::ffi::c_int;

/// The operator always works on whole lines, whatever the motion said.
const OPF_LINES: u8 = 1;
/// The operator modifies the buffer.
const OPF_CHANGE: u8 = 2;

/// One row of [`OPCHARS`]: the keys that name an operator, and what it does.
struct OpChar {
    /// First character typed, `NUL` for [`OpType::Nop`]'s placeholder row.
    first: u8,
    /// Second character, `NUL` when the first is enough.
    second: u8,
    /// [`OPF_LINES`] and/or [`OPF_CHANGE`].
    flags: u8,
}

/// One row per operator, **indexed by its [`OpType`]**: the row order is the
/// numbering, so inserting a row here without adding a variant in
/// `types/ops.rs` renames every operator after it.
static OPCHARS: [OpChar; 30] = {
    /// Row for an operator whose first character is enough, e.g. `d`.
    const fn one(first: u8, flags: u8) -> OpChar {
        OpChar {
            first,
            second: b'\0',
            flags,
        }
    }
    /// Row for a two-character operator, e.g. `g~`.
    const fn two(first: u8, second: u8, flags: u8) -> OpChar {
        OpChar {
            first,
            second,
            flags,
        }
    }
    [
        one(b'\0', 0),                           // OpType::Nop
        one(b'd', OPF_CHANGE),                   // OpType::Delete
        one(b'y', 0),                            // OpType::Yank
        one(b'c', OPF_CHANGE),                   // OpType::Change
        one(b'<', OPF_LINES | OPF_CHANGE),       // OpType::Lshift
        one(b'>', OPF_LINES | OPF_CHANGE),       // OpType::Rshift
        one(b'!', OPF_LINES | OPF_CHANGE),       // OpType::Filter
        two(b'g', b'~', OPF_CHANGE),             // OpType::Tilde
        one(b'=', OPF_LINES | OPF_CHANGE),       // OpType::Indent
        two(b'g', b'q', OPF_LINES | OPF_CHANGE), // OpType::Format
        one(b':', OPF_LINES),                    // OpType::Colon
        two(b'g', b'U', OPF_CHANGE),             // OpType::Upper
        two(b'g', b'u', OPF_CHANGE),             // OpType::Lower
        one(b'J', OPF_LINES | OPF_CHANGE),       // OpType::Join
        two(b'g', b'J', OPF_LINES | OPF_CHANGE), // OpType::JoinNs
        two(b'g', b'?', OPF_CHANGE),             // OpType::Rot13
        one(b'r', OPF_CHANGE),                   // OpType::Replace
        one(b'I', OPF_CHANGE),                   // OpType::Insert
        one(b'A', OPF_CHANGE),                   // OpType::Append
        two(b'z', b'f', 0),                      // OpType::Fold
        two(b'z', b'o', OPF_LINES),              // OpType::Foldopen
        two(b'z', b'O', OPF_LINES),              // OpType::Foldopenrec
        two(b'z', b'c', OPF_LINES),              // OpType::Foldclose
        two(b'z', b'C', OPF_LINES),              // OpType::Foldcloserec
        two(b'z', b'd', OPF_LINES),              // OpType::Folddel
        two(b'z', b'D', OPF_LINES),              // OpType::Folddelrec
        two(b'g', b'w', OPF_LINES | OPF_CHANGE), // OpType::Format2
        two(b'g', b'@', OPF_CHANGE),             // OpType::Function
        one(Ctrl_A as u8, OPF_CHANGE),           // OpType::NrAdd
        one(Ctrl_X as u8, OPF_CHANGE),           // OpType::NrSub
    ]
};

/// The operator an operator's one or two characters name.
///
/// `char2` is `NUL` when only one character was typed. Five operators are not
/// in [`OPCHARS`] under the keys that reach them, because in Visual mode those
/// keys mean an operator and elsewhere they do not: `r`, `~`, `g CTRL-A`,
/// `g CTRL-X` and `zy`.
///
/// A pair that names nothing is a bug in the caller; upstream reports it and
/// answers the last row, and so does this.
pub fn get_op_type(char1: c_int, char2: c_int) -> OpType {
    // A key outside 0..=255 matches none of these, and none of the operators
    // below is spelled with one either.
    match u8::try_from(char1).ok() {
        Some(b'r') => return OpType::Replace,
        Some(b'~') => return OpType::Tilde,
        Some(b'g') if char2 == Ctrl_A => return OpType::NrAdd,
        Some(b'g') if char2 == Ctrl_X => return OpType::NrSub,
        Some(b'z') if char2 == c_int::from(b'y') => return OpType::Yank,
        _ => {}
    }
    for (op, row) in OPCHARS.iter().enumerate() {
        if c_int::from(row.first) == char1 && c_int::from(row.second) == char2 {
            return OpType::ALL[op];
        }
    }
    // SAFETY: a literal C string.
    unsafe { internal_error(c"get_op_type()".as_ptr()) };
    OpType::ALL[OPCHARS.len() - 1]
}

/// Does this operator always work on whole lines?
pub fn op_on_lines(op: OpType) -> bool {
    OPCHARS[op as usize].flags & OPF_LINES != 0
}

/// Does this operator modify the buffer?
pub fn op_is_change(op: OpType) -> bool {
    OPCHARS[op as usize].flags & OPF_CHANGE != 0
}

/// First character of the keys that name this operator.
pub fn get_op_char(op: OpType) -> c_int {
    c_int::from(OPCHARS[op as usize].first)
}

/// Second character of the keys that name this operator, `NUL` if there is
/// only one.
pub fn get_extra_op_char(op: OpType) -> c_int {
    c_int::from(OPCHARS[op as usize].second)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The row order is the [`OpType`] numbering, and a table typo here would
    /// silently rename every operator after it.
    #[test]
    fn rows_are_the_op_numbering() {
        for (op, keys) in [
            (OpType::Nop, (b'\0', b'\0')),
            (OpType::Delete, (b'd', b'\0')),
            (OpType::Yank, (b'y', b'\0')),
            (OpType::Change, (b'c', b'\0')),
            (OpType::Lshift, (b'<', b'\0')),
            (OpType::Rshift, (b'>', b'\0')),
            (OpType::Filter, (b'!', b'\0')),
            (OpType::Tilde, (b'g', b'~')),
            (OpType::Indent, (b'=', b'\0')),
            (OpType::Format, (b'g', b'q')),
            (OpType::Colon, (b':', b'\0')),
            (OpType::Upper, (b'g', b'U')),
            (OpType::Lower, (b'g', b'u')),
            (OpType::Join, (b'J', b'\0')),
            (OpType::JoinNs, (b'g', b'J')),
            (OpType::Rot13, (b'g', b'?')),
            (OpType::Replace, (b'r', b'\0')),
            (OpType::Insert, (b'I', b'\0')),
            (OpType::Append, (b'A', b'\0')),
            (OpType::Fold, (b'z', b'f')),
            (OpType::Foldopen, (b'z', b'o')),
            (OpType::Foldopenrec, (b'z', b'O')),
            (OpType::Foldclose, (b'z', b'c')),
            (OpType::Foldcloserec, (b'z', b'C')),
            (OpType::Folddel, (b'z', b'd')),
            (OpType::Folddelrec, (b'z', b'D')),
            (OpType::Format2, (b'g', b'w')),
            (OpType::Function, (b'g', b'@')),
            (OpType::NrAdd, (Ctrl_A as u8, b'\0')),
            (OpType::NrSub, (Ctrl_X as u8, b'\0')),
        ] {
            let row = &OPCHARS[op as usize];
            assert_eq!((row.first, row.second), keys, "row {op:?}");
        }
    }

    /// Every two-character operator is reachable through the table, and the
    /// five special cases through the prefix match above it.
    #[test]
    fn lookup_round_trips() {
        for &op in &OpType::ALL[1..] {
            // `r` and `~` are the two rows the special cases shadow: `r`
            // answers `Replace` either way, and `~` is not in the table.
            let found = get_op_type(get_op_char(op), get_extra_op_char(op));
            assert_eq!(found, op, "row {op:?}");
        }
        assert_eq!(get_op_type(b'~'.into(), 0), OpType::Tilde);
        assert_eq!(get_op_type(b'g'.into(), Ctrl_A), OpType::NrAdd);
        assert_eq!(get_op_type(b'g'.into(), Ctrl_X), OpType::NrSub);
        assert_eq!(get_op_type(b'z'.into(), b'y'.into()), OpType::Yank);
    }

    #[test]
    fn flags_match_upstream() {
        assert!(op_on_lines(OpType::Join) && op_is_change(OpType::Join));
        assert!(!op_on_lines(OpType::Delete) && op_is_change(OpType::Delete));
        assert!(!op_on_lines(OpType::Yank) && !op_is_change(OpType::Yank));
        assert!(op_on_lines(OpType::Colon) && !op_is_change(OpType::Colon));
        assert!(!op_on_lines(OpType::Fold) && !op_is_change(OpType::Fold));
    }
}
