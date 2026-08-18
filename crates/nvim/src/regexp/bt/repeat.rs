//! How many times a `SIMPLE` item matches from here.
//!
//! The matcher uses this for `STAR`, `PLUS` and `BRACE_SIMPLE`: an item that
//! matches exactly one character and cannot backtrack into itself can be run
//! as a counted loop instead of as a chain of program nodes.
//!
//! Every opcode with an `ADD_NL` variant sits `ADD_NL` above its plain form,
//! so the `FIRST_NL..=LAST_NL` band is exactly the set of `\_x` nodes — the
//! ones allowed to count a line break as one of their matches.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use crate::ascii::ascii_isdigit;
use crate::charset::{vim_isIDc, vim_isfilec, vim_isprintc, vim_iswordp_buf};
use crate::main::{e_re_corr, got_int};
use crate::mbyte::{mb_tolower, mb_toupper, utf_fold, utf_ptr2char, utfc_ptr2len};
use crate::message::iemsg;
use crate::os::cshim::gettext;
use crate::regexp::{
    ADD_NL, ALPHA, ANY, ANYBUT, ANYOF, DIGIT, EXACTLY, FIRST_NL, FNAME, HEAD, HEX, IDENT, KWORD,
    LAST_NL, LOWER, MULTIBYTECODE, NALPHA, NDIGIT, NEWL, NHEAD, NHEX, NLOWER, NOCTAL, NUL, NUPPER,
    NWHITE, NWORD, OCTAL, PRINT, RI_ALPHA, RI_DIGIT, RI_FLAGS, RI_HEAD, RI_HEX, RI_LOWER, RI_OCTAL,
    RI_UPPER, RI_WHITE, RI_WORD, Rex, SFNAME, SIDENT, SKWORD, SPRINT, UPPER, WHITE, WORD, cstrchr,
    reg_nextline,
};
use crate::types::{int64_t, uint8_t};

/// What stepping onto the next line produced.
enum Line {
    /// The line break counts as one match; here is the new cursor.
    Crossed(*mut uint8_t),
    /// The user interrupted while fetching the line.
    Interrupted(*mut uint8_t),
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
    unsafe {
        let op = *p as c_int;
        let opnd = p.add(3);
        let mut scan = rex.input();
        let mut count: int64_t = 0;

        // Only a `\_x` node counts a line break, and only in a multi-line
        // match where the line break is not already part of the text.
        let crosses_lines = (FIRST_NL..=LAST_NL).contains(&op);
        let next_line = || {
            if !rex.multi()
                || !crosses_lines
                || rex.lnum() > rex.reg_maxline()
                || rex.reg_line_lbr()
            {
                return Line::End;
            }
            reg_nextline(rex);
            if got_int.get() {
                Line::Interrupted(rex.input())
            } else {
                Line::Crossed(rex.input())
            }
        };
        // With 'reg_line_lbr' the text holds real newline bytes rather than
        // line breaks, so a `\_x` matches the byte itself.
        let literal_newline = |scan: *mut uint8_t| {
            rex.reg_line_lbr() && *scan as c_int == '\n' as c_int && crosses_lines
        };

        // Advance over one character of the class, cross a line break, or
        // stop. Shared by every class-shaped opcode below; `accept` sees the
        // cursor and says whether the character there belongs.
        macro_rules! count_class {
            ($nul_first:expr, $accept:expr) => {
                'walk: while count < maxcount {
                    let at_nul = *scan as c_int == NUL;
                    if at_nul && $nul_first {
                        match next_line() {
                            Line::Crossed(next) => scan = next,
                            Line::Interrupted(_) => break 'walk,
                            Line::End => break 'walk,
                        }
                    } else if $accept(scan) {
                        scan = scan.add(utfc_ptr2len(scan.cast()) as usize);
                    } else if at_nul {
                        match next_line() {
                            Line::Crossed(next) => scan = next,
                            Line::Interrupted(_) => break 'walk,
                            Line::End => break 'walk,
                        }
                    } else if literal_newline(scan) {
                        scan = scan.add(1);
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
            op - if crosses_lines { ADD_NL } else { 0 },
            IDENT | KWORD | FNAME | PRINT | ANYOF
        );

        match op {
            // `.` and `\_.`: every character, and for `\_.` every line.
            _ if is(op, ANY) => {
                'any: while count < maxcount {
                    while *scan as c_int != NUL && count < maxcount {
                        count += 1;
                        scan = scan.add(utfc_ptr2len(scan.cast()) as usize);
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
                        Line::Interrupted(_) => {
                            count += 1;
                            break 'any;
                        }
                        Line::End => break 'any,
                    }
                }
            }

            // `\i`/`\I`: 'isident' characters.
            _ if is(op, IDENT) || is(op, SIDENT) => count_class!(false, |scan: *mut uint8_t| {
                vim_isIDc(utf_ptr2char(scan.cast())) && (positive || !ascii_isdigit(*scan as c_int))
            }),

            // `\k`/`\K`: 'iskeyword' characters, which are buffer-local.
            _ if is(op, KWORD) || is(op, SKWORD) => count_class!(false, |scan: *mut uint8_t| {
                vim_iswordp_buf(scan.cast(), rex.reg_buf())
                    && (positive || !ascii_isdigit(*scan as c_int))
            }),

            // `\f`/`\F`: 'isfname' characters.
            _ if is(op, FNAME) || is(op, SFNAME) => count_class!(false, |scan: *mut uint8_t| {
                vim_isfilec(utf_ptr2char(scan.cast()))
                    && (positive || !ascii_isdigit(*scan as c_int))
            }),

            // `\p`/`\P`: printable characters. This one tests for the end of
            // the line before the class, where the three above do it after.
            _ if is(op, PRINT) || is(op, SPRINT) => count_class!(true, |scan: *mut uint8_t| {
                vim_isprintc(utf_ptr2char(scan.cast()))
                    && (positive || !ascii_isdigit(*scan as c_int))
            }),

            // A literal character. Case folding here is byte-wise: an
            // `EXACTLY` that a multi can repeat is single-byte by
            // construction (`use_multibytecode` sends the rest to
            // `MULTIBYTECODE`).
            EXACTLY => {
                if rex.reg_ic() {
                    let upper = mb_toupper(*opnd as c_int);
                    let lower = mb_tolower(*opnd as c_int);
                    while count < maxcount && (*scan as c_int == upper || *scan as c_int == lower) {
                        count += 1;
                        scan = scan.add(1);
                    }
                } else {
                    let want = *opnd as c_int;
                    while count < maxcount && *scan as c_int == want {
                        count += 1;
                        scan = scan.add(1);
                    }
                }
            }

            // One multibyte character, compared as bytes first and by case
            // fold only if the bytes differ.
            MULTIBYTECODE => {
                let len = utfc_ptr2len(opnd.cast());
                if len > 1 {
                    let folded = if rex.reg_ic() {
                        utf_fold(utf_ptr2char(opnd.cast()))
                    } else {
                        0
                    };
                    while count < maxcount && utfc_ptr2len(scan.cast()) >= len {
                        let same = (0..len).all(|i| *opnd.add(i as usize) == *scan.add(i as usize));
                        if !same && (!rex.reg_ic() || utf_fold(utf_ptr2char(scan.cast())) != folded)
                        {
                            break;
                        }
                        scan = scan.add(len as usize);
                        count += 1;
                    }
                }
            }

            // A `[]` collection.
            _ if is(op, ANYOF) || is(op, ANYBUT) => {
                let wanted = c_int::from(positive);
                'coll: while count < maxcount {
                    if *scan as c_int == NUL {
                        match next_line() {
                            Line::Crossed(next) => scan = next,
                            Line::Interrupted(_) | Line::End => break 'coll,
                        }
                    } else if literal_newline(scan) {
                        scan = scan.add(1);
                    } else {
                        let len = utfc_ptr2len(scan.cast());
                        let c = if len > 1 {
                            utf_ptr2char(scan.cast())
                        } else {
                            *scan as c_int
                        };
                        if c_int::from(cstrchr(rex, opnd.cast(), c).is_null()) == wanted {
                            break 'coll;
                        }
                        scan = scan.add(if len > 1 { len as usize } else { 1 });
                    }
                    count += 1;
                }
            }

            // A line break as an atom of its own.
            NEWL => {
                while count < maxcount
                    && ((*scan as c_int == NUL
                        && rex.lnum() <= rex.reg_maxline()
                        && !rex.reg_line_lbr()
                        && rex.multi())
                        || (*scan as c_int == '\n' as c_int && rex.reg_line_lbr()))
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
                    iemsg(gettext(&raw const e_re_corr as *const c_char));
                    rex.set_input(scan);
                    return count as c_int;
                };
                let testval = if positive { mask } else { 0 };
                'bytes: while count < maxcount {
                    if *scan as c_int == NUL {
                        match next_line() {
                            Line::Crossed(next) => scan = next,
                            Line::Interrupted(_) | Line::End => break 'bytes,
                        }
                    } else {
                        let len = utfc_ptr2len(scan.cast());
                        if len > 1 {
                            // A multibyte character is in none of these
                            // classes, so only the negative form takes it.
                            if positive {
                                break 'bytes;
                            }
                            scan = scan.add(len as usize);
                        } else if RI_FLAGS[*scan as usize] as c_int & mask == testval {
                            scan = scan.add(1);
                        } else if literal_newline(scan) {
                            scan = scan.add(1);
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
}

/// Is `op` `base` or its `\_` twin?
fn is(op: c_int, base: c_int) -> bool {
    op == base || op == base + ADD_NL
}

/// The `RI_*` mask an opcode tests against, and whether it wants a hit or a
/// miss.
fn byte_class(op: c_int) -> Option<(c_int, bool)> {
    let (mask, positive) = match op - if op > NUPPER { ADD_NL } else { 0 } {
        WHITE => (RI_WHITE, true),
        NWHITE => (RI_WHITE, false),
        DIGIT => (RI_DIGIT, true),
        NDIGIT => (RI_DIGIT, false),
        HEX => (RI_HEX, true),
        NHEX => (RI_HEX, false),
        OCTAL => (RI_OCTAL, true),
        NOCTAL => (RI_OCTAL, false),
        WORD => (RI_WORD, true),
        NWORD => (RI_WORD, false),
        HEAD => (RI_HEAD, true),
        NHEAD => (RI_HEAD, false),
        ALPHA => (RI_ALPHA, true),
        NALPHA => (RI_ALPHA, false),
        LOWER => (RI_LOWER, true),
        NLOWER => (RI_LOWER, false),
        UPPER => (RI_UPPER, true),
        NUPPER => (RI_UPPER, false),
        _ => return None,
    };
    Some((mask, positive))
}
