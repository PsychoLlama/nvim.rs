//! The operator vocabulary: which keys name an operator, and what it does
//! to a region.
//!
//! [`OPCHARS`] is the table upstream keeps in lock-step with the `OP_*` order,
//! one row per operator: the first character typed, the optional second (`g~`,
//! `zf`, `g@`) and two flags. Everything else in this file reads one column of
//! it. [`get_op_type`] is the reverse lookup normal mode does on the keys it
//! just read, with five special cases (`r`, `~`, `g CTRL-A`, `g CTRL-X`, `zy`)
//! the table cannot express, because those keys mean an operator only in
//! Visual mode and something else outside it.
//!
//! The table was a `static` in C and is a `const` here: nothing has ever
//! written to it, and the row order *is* the `OP_*` numbering.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::keycodes::{Ctrl_A, Ctrl_X};
use crate::message::internal_error;
use crate::types::{OP_NR_ADD, OP_NR_SUB, OP_REPLACE, OP_TILDE, OP_YANK, OpType};

use ::core::ffi::c_int;

/// The operator always works on whole lines, whatever the motion said.
const OPF_LINES: u8 = 1;
/// The operator modifies the buffer.
const OPF_CHANGE: u8 = 2;

/// One row of [`OPCHARS`]: the keys that name an operator, and what it does.
struct OpChar {
    /// First character typed, `NUL` for [`OP_NOP`]'s placeholder row.
    first: u8,
    /// Second character, `NUL` when the first is enough.
    second: u8,
    /// [`OPF_LINES`] and/or [`OPF_CHANGE`].
    flags: u8,
}

/// One row per operator, **indexed by `OP_*`**: the row order is the
/// numbering, so inserting a row here without inserting a name in
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
        one(b'\0', 0),                           // OP_NOP
        one(b'd', OPF_CHANGE),                   // OP_DELETE
        one(b'y', 0),                            // OP_YANK
        one(b'c', OPF_CHANGE),                   // OP_CHANGE
        one(b'<', OPF_LINES | OPF_CHANGE),       // OP_LSHIFT
        one(b'>', OPF_LINES | OPF_CHANGE),       // OP_RSHIFT
        one(b'!', OPF_LINES | OPF_CHANGE),       // OP_FILTER
        two(b'g', b'~', OPF_CHANGE),             // OP_TILDE
        one(b'=', OPF_LINES | OPF_CHANGE),       // OP_INDENT
        two(b'g', b'q', OPF_LINES | OPF_CHANGE), // OP_FORMAT
        one(b':', OPF_LINES),                    // OP_COLON
        two(b'g', b'U', OPF_CHANGE),             // OP_UPPER
        two(b'g', b'u', OPF_CHANGE),             // OP_LOWER
        one(b'J', OPF_LINES | OPF_CHANGE),       // OP_JOIN
        two(b'g', b'J', OPF_LINES | OPF_CHANGE), // OP_JOIN_NS
        two(b'g', b'?', OPF_CHANGE),             // OP_ROT13
        one(b'r', OPF_CHANGE),                   // OP_REPLACE
        one(b'I', OPF_CHANGE),                   // OP_INSERT
        one(b'A', OPF_CHANGE),                   // OP_APPEND
        two(b'z', b'f', 0),                      // OP_FOLD
        two(b'z', b'o', OPF_LINES),              // OP_FOLDOPEN
        two(b'z', b'O', OPF_LINES),              // OP_FOLDOPENREC
        two(b'z', b'c', OPF_LINES),              // OP_FOLDCLOSE
        two(b'z', b'C', OPF_LINES),              // OP_FOLDCLOSEREC
        two(b'z', b'd', OPF_LINES),              // OP_FOLDDEL
        two(b'z', b'D', OPF_LINES),              // OP_FOLDDELREC
        two(b'g', b'w', OPF_LINES | OPF_CHANGE), // OP_FORMAT2
        two(b'g', b'@', OPF_CHANGE),             // OP_FUNCTION
        one(Ctrl_A as u8, OPF_CHANGE),           // OP_NR_ADD
        one(Ctrl_X as u8, OPF_CHANGE),           // OP_NR_SUB
    ]
};

/// The `OP_*` an operator's one or two characters name.
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
        Some(b'r') => return OP_REPLACE,
        Some(b'~') => return OP_TILDE,
        Some(b'g') if char2 == Ctrl_A => return OP_NR_ADD,
        Some(b'g') if char2 == Ctrl_X => return OP_NR_SUB,
        Some(b'z') if char2 == c_int::from(b'y') => return OP_YANK,
        _ => {}
    }
    for (op, row) in OPCHARS.iter().enumerate() {
        if c_int::from(row.first) == char1 && c_int::from(row.second) == char2 {
            return op as OpType;
        }
    }
    // SAFETY: a literal C string.
    unsafe { internal_error(c"get_op_type()".as_ptr()) };
    OPCHARS.len() as OpType - 1
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
    use crate::types::{
        OP_APPEND, OP_CHANGE, OP_COLON, OP_DELETE, OP_FILTER, OP_FOLD, OP_FOLDCLOSE,
        OP_FOLDCLOSEREC, OP_FOLDDEL, OP_FOLDDELREC, OP_FOLDOPEN, OP_FOLDOPENREC, OP_FORMAT,
        OP_FORMAT2, OP_FUNCTION, OP_INDENT, OP_INSERT, OP_JOIN, OP_JOIN_NS, OP_LOWER, OP_LSHIFT,
        OP_NOP, OP_ROT13, OP_RSHIFT, OP_UPPER,
    };

    /// The row order is the `OP_*` numbering, and a table typo here would
    /// silently rename every operator after it.
    #[test]
    fn rows_are_the_op_numbering() {
        for (op, keys) in [
            (OP_NOP, (b'\0', b'\0')),
            (OP_DELETE, (b'd', b'\0')),
            (OP_YANK, (b'y', b'\0')),
            (OP_CHANGE, (b'c', b'\0')),
            (OP_LSHIFT, (b'<', b'\0')),
            (OP_RSHIFT, (b'>', b'\0')),
            (OP_FILTER, (b'!', b'\0')),
            (OP_TILDE, (b'g', b'~')),
            (OP_INDENT, (b'=', b'\0')),
            (OP_FORMAT, (b'g', b'q')),
            (OP_COLON, (b':', b'\0')),
            (OP_UPPER, (b'g', b'U')),
            (OP_LOWER, (b'g', b'u')),
            (OP_JOIN, (b'J', b'\0')),
            (OP_JOIN_NS, (b'g', b'J')),
            (OP_ROT13, (b'g', b'?')),
            (OP_REPLACE, (b'r', b'\0')),
            (OP_INSERT, (b'I', b'\0')),
            (OP_APPEND, (b'A', b'\0')),
            (OP_FOLD, (b'z', b'f')),
            (OP_FOLDOPEN, (b'z', b'o')),
            (OP_FOLDOPENREC, (b'z', b'O')),
            (OP_FOLDCLOSE, (b'z', b'c')),
            (OP_FOLDCLOSEREC, (b'z', b'C')),
            (OP_FOLDDEL, (b'z', b'd')),
            (OP_FOLDDELREC, (b'z', b'D')),
            (OP_FORMAT2, (b'g', b'w')),
            (OP_FUNCTION, (b'g', b'@')),
            (OP_NR_ADD, (Ctrl_A as u8, b'\0')),
            (OP_NR_SUB, (Ctrl_X as u8, b'\0')),
        ] {
            let row = &OPCHARS[op as usize];
            assert_eq!((row.first, row.second), keys, "row {op}");
        }
    }

    /// Every two-character operator is reachable through the table, and the
    /// five special cases through the prefix match above it.
    #[test]
    fn lookup_round_trips() {
        for op in 1..OPCHARS.len() as OpType {
            // `r` and `~` are the two rows the special cases shadow: `r`
            // answers OP_REPLACE either way, and `~` is not in the table.
            let found = get_op_type(get_op_char(op), get_extra_op_char(op));
            assert_eq!(found, op, "row {op}");
        }
        assert_eq!(get_op_type(b'~'.into(), 0), OP_TILDE);
        assert_eq!(get_op_type(b'g'.into(), Ctrl_A), OP_NR_ADD);
        assert_eq!(get_op_type(b'g'.into(), Ctrl_X), OP_NR_SUB);
        assert_eq!(get_op_type(b'z'.into(), b'y'.into()), OP_YANK);
    }

    #[test]
    fn flags_match_upstream() {
        assert!(op_on_lines(OP_JOIN) && op_is_change(OP_JOIN));
        assert!(!op_on_lines(OP_DELETE) && op_is_change(OP_DELETE));
        assert!(!op_on_lines(OP_YANK) && !op_is_change(OP_YANK));
        assert!(op_on_lines(OP_COLON) && !op_is_change(OP_COLON));
        assert!(!op_on_lines(OP_FOLD) && !op_is_change(OP_FOLD));
    }
}
