//! How many times a `SIMPLE` item matches from here.
//!
//! The matcher uses this for `STAR`, `PLUS` and `BRACE_SIMPLE`: an item that
//! matches exactly one character and cannot backtrack into itself can be run
//! as a counted loop instead of as a chain of program nodes.
//!
//! A `\_x` node is the one allowed to count a line break as one of its
//! matches, which [`BtOp::decode`] answers alongside the opcode.

#![forbid(unsafe_code)]

use core::ffi::c_int;

use super::compile::BtProg;
use super::op::BtOp;
use crate::ascii::ascii_isdigit;
use crate::charset::{is_word_at, vim_is_ident_char, vim_isfilec, vim_isprintc};
use crate::cstr;
use crate::getchar::state::got_int;
use crate::mbyte::{mb_tolower, mb_toupper, utf_fold};
use crate::message::e_re_corr;
use crate::message::iemsg;
use crate::os::cshim::gettext;
use crate::regexp::{ByteClass, RI_FLAGS, Rex, char_at_nul, cluster_len_at, cstrchr, reg_nextline};
use crate::types::{NUL, int64_t};

/// What stepping onto the next line produced.
enum Line {
    /// The line break counts as one match; the cursor is at the start of
    /// the next line.
    Crossed,
    /// The user interrupted while fetching the line.
    Interrupted,
    /// This node does not cross line breaks, or there is no next line.
    End,
}

/// How many times the item at `p` matches starting at `rex.input`, capped at
/// `maxcount`. Leaves `rex.input` just past the last match.
///
/// The walk holds the line it is on and an offset into it, and takes the
/// line again from the match only when it crosses onto the next one: the
/// per-character loops index one slice.
pub(crate) fn regrepeat(rex: Rex, prog: BtProg<'_>, p: usize, maxcount: int64_t) -> c_int {
    let opcode = BtOp::decode(prog.op_byte(p));
    let opnd = prog.operand(p);
    let mut line = rex.line();
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
            Line::Crossed
        }
    };
    // The byte at `at`; the line ends in its NUL, so `at` never passes it.
    let byte = |line: &[u8], at: usize| line[at];
    // With 'reg_line_lbr' the text holds real newline bytes rather than
    // line breaks, so a `\_x` matches the byte itself.
    let literal_newline =
        |line: &[u8], at: usize| rex.reg_line_lbr() && line[at] == b'\n' && crosses_lines;

    // Advance over one character of the class, cross a line break, or
    // stop. Shared by every class-shaped opcode below; `accept` sees the
    // text from the cursor and says whether the character there belongs.
    macro_rules! count_class {
        ($nul_first:expr, $accept:expr) => {
            'walk: while count < maxcount {
                let at_nul = c_int::from(byte(line, scan)) == NUL;
                if at_nul && $nul_first {
                    match next_line() {
                        Line::Crossed => (line, scan) = (rex.line(), 0),
                        Line::Interrupted => break 'walk,
                        Line::End => break 'walk,
                    }
                } else if $accept(&line[scan..]) {
                    scan += cluster_len_at(line, scan);
                } else if at_nul {
                    match next_line() {
                        Line::Crossed => (line, scan) = (rex.line(), 0),
                        Line::Interrupted => break 'walk,
                        Line::End => break 'walk,
                    }
                } else if literal_newline(line, scan) {
                    scan += 1;
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
    // The `S`-prefixed classes reject a leading digit.
    let digit_ok = |text: &[u8]| positive || !ascii_isdigit(c_int::from(text[0]));

    match op {
        // `.` and `\_.`: every character, and for `\_.` every line.
        BtOp::Any => {
            'any: while count < maxcount {
                while c_int::from(byte(line, scan)) != NUL && count < maxcount {
                    count += 1;
                    scan += cluster_len_at(line, scan);
                }
                if count == maxcount {
                    break 'any;
                }
                // Unlike the class walks below, the line break is counted
                // before the interrupt check.
                match next_line() {
                    Line::Crossed => {
                        count += 1;
                        (line, scan) = (rex.line(), 0);
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
        BtOp::Ident | BtOp::Sident => count_class!(false, |text: &[u8]| {
            vim_is_ident_char(char_at_nul(text, 0)) && digit_ok(text)
        }),

        // `\k`/`\K`: 'iskeyword' characters, which are buffer-local.
        BtOp::Kword | BtOp::Skword => count_class!(false, |text: &[u8]| {
            is_word_at(text, rex.reg_buf()) && digit_ok(text)
        }),

        // `\f`/`\F`: 'isfname' characters.
        BtOp::Fname | BtOp::Sfname => count_class!(false, |text: &[u8]| {
            vim_isfilec(char_at_nul(text, 0)) && digit_ok(text)
        }),

        // `\p`/`\P`: printable characters. This one tests for the end of
        // the line before the class, where the three above do it after.
        BtOp::Print | BtOp::Sprint => count_class!(true, |text: &[u8]| {
            vim_isprintc(char_at_nul(text, 0)) && digit_ok(text)
        }),

        // A literal character. Case folding here is byte-wise: an
        // `EXACTLY` that a multi can repeat is single-byte by
        // construction (`use_multibytecode` sends the rest to
        // `MULTIBYTECODE`).
        BtOp::Exactly => {
            if rex.reg_ic() {
                let upper = mb_toupper(c_int::from(opnd[0]));
                let lower = mb_tolower(c_int::from(opnd[0]));
                while count < maxcount
                    && (c_int::from(byte(line, scan)) == upper
                        || c_int::from(byte(line, scan)) == lower)
                {
                    count += 1;
                    scan += 1;
                }
            } else {
                let want = opnd[0];
                while count < maxcount && byte(line, scan) == want {
                    count += 1;
                    scan += 1;
                }
            }
        }

        // One multibyte character, compared as bytes first and by case
        // fold only if the bytes differ.
        BtOp::Multibytecode => {
            let len = cluster_len_at(opnd, 0);
            if len > 1 {
                let folded = if rex.reg_ic() {
                    utf_fold(char_at_nul(opnd, 0))
                } else {
                    0
                };
                while count < maxcount && cluster_len_at(line, scan) >= len {
                    let same = opnd[..len] == line[scan..scan + len];
                    if !same && (!rex.reg_ic() || utf_fold(char_at_nul(line, scan)) != folded) {
                        break;
                    }
                    scan += len;
                    count += 1;
                }
            }
        }

        // A `[]` collection.
        BtOp::Anyof | BtOp::Anybut => {
            let set = cstr::until_nul(opnd);
            'coll: while count < maxcount {
                if c_int::from(byte(line, scan)) == NUL {
                    match next_line() {
                        Line::Crossed => (line, scan) = (rex.line(), 0),
                        Line::Interrupted | Line::End => break 'coll,
                    }
                } else if literal_newline(line, scan) {
                    scan += 1;
                } else {
                    let len = cluster_len_at(line, scan);
                    let c = if len > 1 {
                        char_at_nul(line, scan)
                    } else {
                        c_int::from(line[scan])
                    };
                    if cstrchr(rex, set, c).is_some() != positive {
                        break 'coll;
                    }
                    scan += len.max(1);
                }
                count += 1;
            }
        }

        // A line break as an atom of its own.
        BtOp::Newl => {
            while count < maxcount
                && ((c_int::from(byte(line, scan)) == NUL
                    && rex.lnum() <= rex.reg_maxline()
                    && !rex.reg_line_lbr()
                    && rex.multi())
                    || (byte(line, scan) == b'\n' && rex.reg_line_lbr()))
            {
                count += 1;
                if rex.reg_line_lbr() {
                    rex.set_input(scan);
                    rex.advance_char();
                } else {
                    reg_nextline(rex);
                }
                (line, scan) = (rex.line(), rex.input());
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

            'bytes: while count < maxcount {
                if c_int::from(byte(line, scan)) == NUL {
                    match next_line() {
                        Line::Crossed => (line, scan) = (rex.line(), 0),
                        Line::Interrupted | Line::End => break 'bytes,
                    }
                } else {
                    let len = cluster_len_at(line, scan);
                    if len > 1 {
                        // A multibyte character is in none of these
                        // classes, so only the negative form takes it.
                        if positive {
                            break 'bytes;
                        }
                        scan += len;
                    } else if RI_FLAGS[usize::from(line[scan])].has(mask) == positive
                        || literal_newline(line, scan)
                    {
                        scan += 1;
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
fn byte_class(op: BtOp) -> Option<(ByteClass, bool)> {
    let (mask, positive) = match op {
        BtOp::White => (ByteClass::WHITE, true),
        BtOp::Nwhite => (ByteClass::WHITE, false),
        BtOp::Digit => (ByteClass::DIGIT, true),
        BtOp::Ndigit => (ByteClass::DIGIT, false),
        BtOp::Hex => (ByteClass::HEX, true),
        BtOp::Nhex => (ByteClass::HEX, false),
        BtOp::Octal => (ByteClass::OCTAL, true),
        BtOp::Noctal => (ByteClass::OCTAL, false),
        BtOp::Word => (ByteClass::WORD, true),
        BtOp::Nword => (ByteClass::WORD, false),
        BtOp::Head => (ByteClass::HEAD, true),
        BtOp::Nhead => (ByteClass::HEAD, false),
        BtOp::Alpha => (ByteClass::ALPHA, true),
        BtOp::Nalpha => (ByteClass::ALPHA, false),
        BtOp::Lower => (ByteClass::LOWER, true),
        BtOp::Nlower => (ByteClass::LOWER, false),
        BtOp::Upper => (ByteClass::UPPER, true),
        BtOp::Nupper => (ByteClass::UPPER, false),
        _ => return None,
    };
    Some((mask, positive))
}
