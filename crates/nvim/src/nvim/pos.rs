#![forbid(unsafe_code)]

//! Ordering and reset for a buffer position.
//!
//! `pos_T` orders lexicographically by line, then column, then `coladd` (the
//! virtual columns past the end of a line that 'virtualedit' allows). The C
//! had these as `static inline`s next to the struct, so the transpiler left a
//! copy in every module that compared two positions.

use crate::src::nvim::types::pos_T;

/// One past the last addressable line: the line number `$` and an open-ended
/// range resolve to, and the sentinel a "no line" mark carries.
pub const MAXLNUM: ::core::ffi::c_uint = 2147483647;

/// Whether two positions name the same place, `coladd` included.
pub fn equalpos(a: pos_T, b: pos_T) -> bool {
    a.lnum == b.lnum && a.col == b.col && a.coladd == b.coladd
}

/// Whether `a` comes strictly before `b`.
pub fn lt(a: pos_T, b: pos_T) -> bool {
    if a.lnum != b.lnum {
        a.lnum < b.lnum
    } else if a.col != b.col {
        a.col < b.col
    } else {
        a.coladd < b.coladd
    }
}

/// Whether `a` comes before `b`, or is the same place.
pub fn ltoreq(a: pos_T, b: pos_T) -> bool {
    lt(a, b) || equalpos(a, b)
}

/// Reset a position to line 0, column 0 — the "no position" the editor uses.
pub fn clearpos(a: &mut pos_T) {
    a.lnum = 0;
    a.col = 0;
    a.coladd = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn pos(lnum: i32, col: i32, coladd: i32) -> pos_T {
        pos_T { lnum, col, coladd }
    }

    #[test]
    fn orders_by_line_then_column_then_coladd() {
        assert!(lt(pos(1, 9, 9), pos(2, 0, 0)));
        assert!(lt(pos(2, 0, 9), pos(2, 1, 0)));
        assert!(lt(pos(2, 1, 0), pos(2, 1, 1)));
        assert!(!lt(pos(2, 1, 1), pos(2, 1, 1)));
        assert!(!lt(pos(2, 0, 0), pos(1, 9, 9)));
    }

    #[test]
    fn equality_and_the_inclusive_order_agree_with_it() {
        assert!(equalpos(pos(3, 4, 5), pos(3, 4, 5)));
        assert!(!equalpos(pos(3, 4, 5), pos(3, 4, 6)));
        assert!(ltoreq(pos(3, 4, 5), pos(3, 4, 5)));
        assert!(ltoreq(pos(3, 4, 5), pos(3, 4, 6)));
        assert!(!ltoreq(pos(3, 4, 6), pos(3, 4, 5)));
    }

    #[test]
    fn clearing_leaves_the_zero_position() {
        let mut p = pos(7, 7, 7);
        clearpos(&mut p);
        assert!(equalpos(p, pos(0, 0, 0)));
    }
}
