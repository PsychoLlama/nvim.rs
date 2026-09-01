//! How many times a `SIMPLE` item matches from here.
//!
//! The matcher uses this for `STAR`, `PLUS` and `BRACE_SIMPLE`: an item that
//! matches exactly one character and cannot backtrack into itself can be run
//! as a counted loop instead of as a chain of program nodes.
//!
//! A `\_x` node is the one allowed to count a line break as one of its
//! matches, which [`BtOp::decode`] answers alongside the opcode.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;

use super::op::BtOp;
use crate::ascii::ascii_isdigit;
use crate::charset::{vim_is_ident_char, vim_isfilec, vim_isprintc, vim_iswordp_buf};
use crate::main::{e_re_corr, got_int};
use crate::mbyte::{mb_tolower, mb_toupper, utf_fold, utf_ptr2char, utfc_ptr2len};
use crate::message::iemsg;
use crate::os::cshim::gettext;
use crate::regexp::{
    RI_ALPHA, RI_DIGIT, RI_FLAGS, RI_HEAD, RI_HEX, RI_LOWER, RI_OCTAL, RI_UPPER, RI_WHITE, RI_WORD,
    Rex, cstrchr, reg_nextline,
};
use crate::types::{NUL, int64_t, uint8_t};

/// What stepping onto the next line produced.
enum Line {
    /// The line break counts as one match; here is the new cursor.
    Crossed(*mut uint8_t),
    /// The user interrupted while fetching the line.
    Interrupted,
    /// This node does not cross line breaks, or there is no next line.
    End,
}

/// How many times the item at `p` matches starting at `rex.input`, capped at
/// `maxcount`. Leaves `rex.input` just past the last match.
pub(crate) fn regrepeat(rex: Rex, p: *mut uint8_t, maxcount: int64_t) -> c_int {
    // SAFETY: `p` is a node in the compiled program, so its opcode byte and
    // the operand three bytes in are readable; `scan` walks the current line,
    // which is NUL-terminated, and every advance below is by the length of
    // the character it just accepted.
    let opcode = BtOp::decode(unsafe { *p });
    let opnd = unsafe { p.add(3) };
    let mut scan = rex.input();
    let mut count: int64_t = 0;

    // Only a `\_x` node counts a line break, and only in a multi-line
    // match where the line break is not already part of the text.
    let Ok((op, crosses_lines)) = opcode else {
        iemsg(gettext(e_re_corr));
        return 0;
    };
    let next_line = || {
        if !rex.multi() || !crosses_lines || rex.lnum() > rex.reg_maxline() || rex.reg_line_lbr() {
            return Line::End;
        }
        reg_nextline(rex);
        if got_int.get() {
            Line::Interrupted
        } else {
            Line::Crossed(rex.input())
        }
    };
    // With 'reg_line_lbr' the text holds real newline bytes rather than
    // line breaks, so a `\_x` matches the byte itself.
    let literal_newline = |scan: *mut uint8_t| {
        rex.reg_line_lbr() && unsafe { *scan } as c_int == '\n' as c_int && crosses_lines
    };

    // Advance over one character of the class, cross a line break, or
    // stop. Shared by every class-shaped opcode below; `accept` sees the
    // cursor and says whether the character there belongs.
    macro_rules! count_class {
        ($nul_first:expr, $accept:expr) => {
            'walk: while count < maxcount {
                let at_nul = unsafe { *scan } as c_int == NUL;
                if at_nul && $nul_first {
                    match next_line() {
                        Line::Crossed(next) => scan = next,
                        Line::Interrupted => break 'walk,
                        Line::End => break 'walk,
                    }
                } else if $accept(scan) {
                    scan = unsafe { scan.add(utfc_ptr2len(scan.cast()) as usize) };
                } else if at_nul {
                    match next_line() {
                        Line::Crossed(next) => scan = next,
                        Line::Interrupted => break 'walk,
                        Line::End => break 'walk,
                    }
                } else if literal_newline(scan) {
                    scan = unsafe { scan.add(1) };
                } else {
                    break 'walk;
                }
                count += 1;
            }
        };
    }

    // Upstream's `testval`: this is the positive form of a pair. For
    // `\i`/`\I` and friends that means digits are members after all; for
    // `[]` it means a hit rather than a miss is what continues the count.
    let positive = matches!(
        op,
        BtOp::Ident | BtOp::Kword | BtOp::Fname | BtOp::Print | BtOp::Anyof
    );

    match op {
        // `.` and `\_.`: every character, and for `\_.` every line.
        BtOp::Any => {
            'any: while count < maxcount {
                while unsafe { *scan } as c_int != NUL && count < maxcount {
                    count += 1;
                    scan = unsafe { scan.add(utfc_ptr2len(scan.cast()) as usize) };
                }
                if count == maxcount {
                    break 'any;
                }
                // Unlike the class walks below, the line break is counted
                // before the interrupt check.
                match next_line() {
                    Line::Crossed(next) => {
                        count += 1;
                        scan = next;
                    }
                    Line::Interrupted => {
                        count += 1;
                        break 'any;
                    }
                    Line::End => break 'any,
                }
            }
        }

        // `\i`/`\I`: 'isident' characters.
        BtOp::Ident | BtOp::Sident => count_class!(false, |scan: *mut uint8_t| {
            unsafe {
                vim_is_ident_char(utf_ptr2char(scan.cast()))
                    && (positive || !ascii_isdigit(*scan as c_int))
            }
        }),

        // `\k`/`\K`: 'iskeyword' characters, which are buffer-local.
        BtOp::Kword | BtOp::Skword => count_class!(false, |scan: *mut uint8_t| {
            unsafe {
                vim_iswordp_buf(scan.cast(), rex.reg_buf())
                    && (positive || !ascii_isdigit(*scan as c_int))
            }
        }),

        // `\f`/`\F`: 'isfname' characters.
        BtOp::Fname | BtOp::Sfname => count_class!(false, |scan: *mut uint8_t| {
            unsafe {
                vim_isfilec(utf_ptr2char(scan.cast()))
                    && (positive || !ascii_isdigit(*scan as c_int))
            }
        }),

        // `\p`/`\P`: printable characters. This one tests for the end of
        // the line before the class, where the three above do it after.
        BtOp::Print | BtOp::Sprint => count_class!(true, |scan: *mut uint8_t| {
            unsafe {
                vim_isprintc(utf_ptr2char(scan.cast()))
                    && (positive || !ascii_isdigit(*scan as c_int))
            }
        }),

        // A literal character. Case folding here is byte-wise: an
        // `EXACTLY` that a multi can repeat is single-byte by
        // construction (`use_multibytecode` sends the rest to
        // `MULTIBYTECODE`).
        BtOp::Exactly => {
            if rex.reg_ic() {
                let upper = mb_toupper(unsafe { *opnd } as c_int);
                let lower = mb_tolower(unsafe { *opnd } as c_int);
                while count < maxcount
                    && (unsafe { *scan } as c_int == upper || unsafe { *scan } as c_int == lower)
                {
                    count += 1;
                    scan = unsafe { scan.add(1) };
                }
            } else {
                let want = unsafe { *opnd } as c_int;
                while count < maxcount && unsafe { *scan } as c_int == want {
                    count += 1;
                    scan = unsafe { scan.add(1) };
                }
            }
        }

        // One multibyte character, compared as bytes first and by case
        // fold only if the bytes differ.
        BtOp::Multibytecode => {
            let len = unsafe { utfc_ptr2len(opnd.cast()) };
            if len > 1 {
                let folded = if rex.reg_ic() {
                    utf_fold(unsafe { utf_ptr2char(opnd.cast()) })
                } else {
                    0
                };
                while count < maxcount && unsafe { utfc_ptr2len(scan.cast()) } >= len {
                    let same = (0..len).all(
                        |i| unsafe { *opnd.add(i as usize) } == unsafe { *scan.add(i as usize) },
                    );
                    if !same
                        && (!rex.reg_ic()
                            || utf_fold(unsafe { utf_ptr2char(scan.cast()) }) != folded)
                    {
                        break;
                    }
                    scan = unsafe { scan.add(len as usize) };
                    count += 1;
                }
            }
        }

        // A `[]` collection.
        BtOp::Anyof | BtOp::Anybut => {
            let wanted = c_int::from(positive);
            'coll: while count < maxcount {
                if unsafe { *scan } as c_int == NUL {
                    match next_line() {
                        Line::Crossed(next) => scan = next,
                        Line::Interrupted | Line::End => break 'coll,
                    }
                } else if literal_newline(scan) {
                    scan = unsafe { scan.add(1) };
                } else {
                    let len = unsafe { utfc_ptr2len(scan.cast()) };
                    let c = if len > 1 {
                        unsafe { utf_ptr2char(scan.cast()) }
                    } else {
                        (unsafe { *scan }) as c_int
                    };
                    if c_int::from(unsafe { cstrchr(rex, opnd.cast(), c) }.is_null()) == wanted {
                        break 'coll;
                    }
                    scan = unsafe { scan.add(if len > 1 { len as usize } else { 1 }) };
                }
                count += 1;
            }
        }

        // A line break as an atom of its own.
        BtOp::Newl => {
            while count < maxcount
                && ((unsafe { *scan } as c_int == NUL
                    && rex.lnum() <= rex.reg_maxline()
                    && !rex.reg_line_lbr()
                    && rex.multi())
                    || (unsafe { *scan } as c_int == '\n' as c_int && rex.reg_line_lbr()))
            {
                count += 1;
                if rex.reg_line_lbr() {
                    rex.advance_char();
                } else {
                    reg_nextline(rex);
                }
                scan = rex.input();
                if got_int.get() {
                    break;
                }
            }
        }

        // Everything else is one of the `RI_*` byte classes.
        _ => {
            let Some((mask, positive)) = byte_class(op) else {
                iemsg(gettext(e_re_corr));
                rex.set_input(scan);
                return count as c_int;
            };
            let testval = if positive { mask } else { 0 };
            'bytes: while count < maxcount {
                if unsafe { *scan } as c_int == NUL {
                    match next_line() {
                        Line::Crossed(next) => scan = next,
                        Line::Interrupted | Line::End => break 'bytes,
                    }
                } else {
                    let len = unsafe { utfc_ptr2len(scan.cast()) };
                    if len > 1 {
                        // A multibyte character is in none of these
                        // classes, so only the negative form takes it.
                        if positive {
                            break 'bytes;
                        }
                        scan = unsafe { scan.add(len as usize) };
                    } else if RI_FLAGS[unsafe { *scan } as usize] as c_int & mask == testval
                        || literal_newline(scan)
                    {
                        scan = unsafe { scan.add(1) };
                    } else {
                        break 'bytes;
                    }
                }
                count += 1;
            }
        }
    }

    rex.set_input(scan);
    count as c_int
}

/// The `RI_*` mask an opcode tests against, and whether it wants a hit or a
/// miss.
fn byte_class(op: BtOp) -> Option<(c_int, bool)> {
    let (mask, positive) = match op {
        BtOp::White => (RI_WHITE, true),
        BtOp::Nwhite => (RI_WHITE, false),
        BtOp::Digit => (RI_DIGIT, true),
        BtOp::Ndigit => (RI_DIGIT, false),
        BtOp::Hex => (RI_HEX, true),
        BtOp::Nhex => (RI_HEX, false),
        BtOp::Octal => (RI_OCTAL, true),
        BtOp::Noctal => (RI_OCTAL, false),
        BtOp::Word => (RI_WORD, true),
        BtOp::Nword => (RI_WORD, false),
        BtOp::Head => (RI_HEAD, true),
        BtOp::Nhead => (RI_HEAD, false),
        BtOp::Alpha => (RI_ALPHA, true),
        BtOp::Nalpha => (RI_ALPHA, false),
        BtOp::Lower => (RI_LOWER, true),
        BtOp::Nlower => (RI_LOWER, false),
        BtOp::Upper => (RI_UPPER, true),
        BtOp::Nupper => (RI_UPPER, false),
        _ => return None,
    };
    Some((mask, positive))
}
