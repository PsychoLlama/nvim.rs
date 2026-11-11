//! How many times a `SIMPLE` item matches from here.
//!
//! The matcher uses this for `STAR`, `PLUS` and `BRACE_SIMPLE`: an item that
//! matches exactly one character and cannot backtrack into itself can be run
//! as a counted loop instead of as a chain of program nodes.
//!
//! A `\_x` node is the one allowed to count a line break as one of its
//! matches, which [`BtOp::decode`] answers alongside the opcode.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

use super::compile::BtProg;
use super::op::BtOp;
use crate::ascii::NUL;
use crate::ascii::ascii_isdigit;
use crate::charset::{vim_is_ident_char, vim_isfilec, vim_isprintc};
use crate::getchar::state::GOT_INT;
use crate::mbyte::{mb_tolower, mb_toupper, utf_fold};
use crate::message::E_RE_CORR;
use crate::message::iemsg;
use crate::os::cshim::gettext;
use crate::os::libc::int64_t;
use crate::regexp::{
    ByteClass, RI_FLAGS, Rex, char_at_nul, cluster_len_at, cstrchr, reg_nextline, word_char_at,
};

/// What stepping onto the next line produced.
#[derive(Debug)]
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
pub(crate) fn regrepeat(rex: Rex, prog: BtProg<'_>, p: usize, maxcount: int64_t) -> int64_t {
    let opcode = BtOp::decode(prog.op_byte(p));
    let opnd = prog.operand(p);

    // Only a `\_x` node counts a line break, and only in a multi-line
    // match where the line break is not already part of the text. A byte
    // that names no opcode is upstream's `default:` arm: the program is
    // corrupt, which is said, and nothing is counted.
    let mut walk = Count {
        rex,
        line: rex.line(),
        scan: rex.input(),
        count: 0,
        maxcount,
        crosses_lines: false,
    };
    let Ok((op, crosses_lines)) = opcode else {
        iemsg(gettext(E_RE_CORR));
        return walk.finish();
    };
    walk.crosses_lines = crosses_lines;

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
        BtOp::Any => walk.any(),

        // `\i`/`\I`: 'isident' characters.
        BtOp::Ident | BtOp::Sident => walk.class(false, |text: &[u8]| {
            vim_is_ident_char(char_at_nul(text, 0)) && digit_ok(text)
        }),

        // `\k`/`\K`: 'iskeyword' characters, which are buffer-local.
        BtOp::Kword | BtOp::Skword => walk.class(false, |text: &[u8]| {
            word_char_at(text, rex.reg_buf()) && digit_ok(text)
        }),

        // `\f`/`\F`: 'isfname' characters.
        BtOp::Fname | BtOp::Sfname => walk.class(false, |text: &[u8]| {
            vim_isfilec(char_at_nul(text, 0)) && digit_ok(text)
        }),

        // `\p`/`\P`: printable characters. This one tests for the end of
        // the line before the class, where the three above do it after.
        BtOp::Print | BtOp::Sprint => walk.class(true, |text: &[u8]| {
            vim_isprintc(char_at_nul(text, 0)) && digit_ok(text)
        }),

        BtOp::Exactly => walk.exactly(opnd[0]),
        BtOp::Multibytecode => walk.multibyte(opnd),
        // A `[]` collection.
        BtOp::Anyof | BtOp::Anybut => walk.collection(opnd, positive),
        BtOp::Newl => walk.newlines(),

        // Everything else is one of the `RI_*` byte classes.
        _ => {
            let Some((mask, positive)) = byte_class(op) else {
                iemsg(gettext(E_RE_CORR));
                return walk.finish();
            };
            walk.bytes(mask, positive);
        }
    }

    walk.finish()
}

/// One [`regrepeat`] count in progress: the line the walk is on, the
/// offset into it, and the matches so far.
struct Count<'t> {
    rex: Rex<'t>,
    line: &'t [u8],
    scan: usize,
    count: int64_t,
    maxcount: int64_t,
    /// A `\_x` node: a line break counts as one of its matches.
    crosses_lines: bool,
}

impl Count<'_> {
    /// Leave `rex.input` at the walk's position and answer the count.
    fn finish(&self) -> int64_t {
        self.rex.set_input(self.scan);
        self.count
    }

    /// The byte at the cursor; the line ends in its NUL, so the cursor
    /// never passes it.
    fn byte(&self) -> u8 {
        self.line[self.scan]
    }

    /// With 'reg_line_lbr' the text holds real newline bytes rather than
    /// line breaks, so a `\_x` matches the byte itself.
    fn literal_newline(&self) -> bool {
        self.rex.reg_line_lbr() && self.line[self.scan] == b'\n' && self.crosses_lines
    }

    /// `.` and `\_.`: every character, and for `\_.` every line.
    fn any(&mut self) {
        while self.count < self.maxcount {
            while c_int::from(self.byte()) != NUL && self.count < self.maxcount {
                self.count += 1;
                self.scan += cluster_len_at(self.line, self.scan);
            }
            if self.count == self.maxcount {
                break;
            }
            // Unlike the class walks below, the line break is counted
            // before the interrupt check.
            match next_line(self.rex, self.crosses_lines) {
                Line::Crossed => {
                    self.count += 1;
                    (self.line, self.scan) = (self.rex.line(), 0);
                }
                Line::Interrupted => {
                    self.count += 1;
                    break;
                }
                Line::End => break,
            }
        }
    }

    /// Advance over one character of a class, cross a line break, or stop.
    /// Shared by every class-shaped opcode; `accept` sees the text from the
    /// cursor and says whether the character there belongs. `nul_first`
    /// tests for the end of the line before the class rather than after.
    fn class(&mut self, nul_first: bool, accept: impl Fn(&[u8]) -> bool) {
        while self.count < self.maxcount {
            let at_nul = c_int::from(self.byte()) == NUL;
            if at_nul && nul_first {
                match next_line(self.rex, self.crosses_lines) {
                    Line::Crossed => (self.line, self.scan) = (self.rex.line(), 0),
                    Line::Interrupted | Line::End => break,
                }
            } else if accept(&self.line[self.scan..]) {
                self.scan += cluster_len_at(self.line, self.scan);
            } else if at_nul {
                match next_line(self.rex, self.crosses_lines) {
                    Line::Crossed => (self.line, self.scan) = (self.rex.line(), 0),
                    Line::Interrupted | Line::End => break,
                }
            } else if self.literal_newline() {
                self.scan += 1;
            } else {
                break;
            }
            self.count += 1;
        }
    }

    /// A literal character. Case folding here is byte-wise: an `EXACTLY`
    /// that a multi can repeat is single-byte by construction
    /// (`use_multibytecode` sends the rest to `MULTIBYTECODE`).
    fn exactly(&mut self, want: u8) {
        if self.rex.reg_ic() {
            let upper = mb_toupper(c_int::from(want));
            let lower = mb_tolower(c_int::from(want));
            while self.count < self.maxcount
                && (c_int::from(self.byte()) == upper || c_int::from(self.byte()) == lower)
            {
                self.count += 1;
                self.scan += 1;
            }
        } else {
            while self.count < self.maxcount && self.byte() == want {
                self.count += 1;
                self.scan += 1;
            }
        }
    }

    /// One multibyte character, compared as bytes first and by case fold
    /// only if the bytes differ.
    fn multibyte(&mut self, opnd: &[u8]) {
        let len = cluster_len_at(opnd, 0);
        if len <= 1 {
            return;
        }
        let ic = self.rex.reg_ic();
        let folded = if ic {
            utf_fold(char_at_nul(opnd, 0))
        } else {
            0
        };
        while self.count < self.maxcount && cluster_len_at(self.line, self.scan) >= len {
            let same = opnd[..len] == self.line[self.scan..self.scan + len];
            if !same && (!ic || utf_fold(char_at_nul(self.line, self.scan)) != folded) {
                break;
            }
            self.scan += len;
            self.count += 1;
        }
    }

    /// A `[]` collection, `set`: a hit continues the count when
    /// `positive`, a miss otherwise.
    fn collection(&mut self, set: &[u8], positive: bool) {
        while self.count < self.maxcount {
            if c_int::from(self.byte()) == NUL {
                match next_line(self.rex, self.crosses_lines) {
                    Line::Crossed => (self.line, self.scan) = (self.rex.line(), 0),
                    Line::Interrupted | Line::End => break,
                }
            } else if self.literal_newline() {
                self.scan += 1;
            } else {
                let (line, scan) = (self.line, self.scan);
                let len = cluster_len_at(line, scan);
                let c = if len > 1 {
                    char_at_nul(line, scan)
                } else {
                    c_int::from(line[scan])
                };
                if cstrchr(self.rex, set, c).is_some() != positive {
                    break;
                }
                self.scan += len.max(1);
            }
            self.count += 1;
        }
    }

    /// A line break as an atom of its own.
    fn newlines(&mut self) {
        let rex = self.rex;
        while self.count < self.maxcount
            && ((c_int::from(self.byte()) == NUL
                && rex.lnum() <= rex.reg_maxline()
                && !rex.reg_line_lbr()
                && rex.multi())
                || (self.byte() == b'\n' && rex.reg_line_lbr()))
        {
            self.count += 1;
            if rex.reg_line_lbr() {
                rex.set_input(self.scan);
                rex.advance_char();
            } else {
                reg_nextline(rex);
            }
            (self.line, self.scan) = (rex.line(), rex.input());
            if GOT_INT.get() {
                break;
            }
        }
    }

    /// One of the `RI_*` byte classes, `mask`: a hit continues the count
    /// when `positive`, a miss otherwise.
    fn bytes(&mut self, mask: ByteClass, positive: bool) {
        while self.count < self.maxcount {
            if c_int::from(self.byte()) == NUL {
                match next_line(self.rex, self.crosses_lines) {
                    Line::Crossed => (self.line, self.scan) = (self.rex.line(), 0),
                    Line::Interrupted | Line::End => break,
                }
            } else {
                let len = cluster_len_at(self.line, self.scan);
                if len > 1 {
                    // A multibyte character is in none of these classes,
                    // so only the negative form takes it.
                    if positive {
                        break;
                    }
                    self.scan += len;
                } else if RI_FLAGS[usize::from(self.line[self.scan])].has(mask) == positive
                    || self.literal_newline()
                {
                    self.scan += 1;
                } else {
                    break;
                }
            }
            self.count += 1;
        }
    }
}

/// Step onto the next line, for a node that `crosses_lines` and when there
/// is one. A free function rather than a [`Count`] method, so that the
/// walks keep their position in registers.
// Inline, as the closure it was: called out of line from the five walks
// it cost rebench `bt` +0.10 % (measured, callgrind on line tables).
#[inline]
fn next_line(rex: Rex, crosses_lines: bool) -> Line {
    if !rex.multi() || !crosses_lines || rex.lnum() > rex.reg_maxline() || rex.reg_line_lbr() {
        return Line::End;
    }
    reg_nextline(rex);
    if GOT_INT.get() {
        Line::Interrupted
    } else {
        Line::Crossed
    }
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
