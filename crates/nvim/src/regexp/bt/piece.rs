//! The parser above the atom: an atom with its multi, the branches around
//! it, and the whole pattern's program.
//!
//! `reg` -> `regbranch` -> `regconcat` -> `regpiece` -> `regatom` is the
//! grammar, one level per precedence step: alternation with `\|`, then
//! concatenation with `\&`, then a repeat suffix, then the atom itself.
//! Each level hands its caller a node and a bag of `HASWIDTH`/`SIMPLE`/
//! `SPSTART`/`HASNL`/`HASLOOKBH` flags describing what it built, and null
//! for "an error was already reported".

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::compile::Node;
use crate::cstr;
use crate::narrow::number_as_u32;
use crate::regexp::chars::{
    M_AMP, M_AT, M_BAR, M_BRACE, M_C_LOWER, M_C_UPPER, M_EQUAL, M_M_LOWER, M_M_UPPER,
    M_PAREN_CLOSE, M_PLUS, M_QUESTION, M_STAR, M_V_LOWER, M_V_UPPER, M_Z_UPPER,
};
use crate::regexp::{BtRegProg, RF_HAD_EOL, RegCompiler, byte_count, low_byte, set_magic};
use core::ffi::c_int;

use super::atom::regatom;
use super::compile::regnext;
use super::op::BtOp;
use crate::ascii::NUL;
use crate::os::libc::int64_t;
use crate::regexp::mbyte::char_at_nul;
use crate::regexp::{
    CompileError, Compiled, HASLOOKBH, HASNL, HASWIDTH, MAGIC_ALL, MAGIC_NONE, MAGIC_OFF, MAGIC_ON,
    NSUBEXP, REG_NOPAREN, REG_NPAREN, REG_PAREN, REG_ZPAREN, REGMAGIC, RF_HASNL, RF_ICASE,
    RF_ICOMBINE, RF_LOOKBH, RF_NOICASE, SIMPLE, SPSTART, Spelled, WORST, getchr, getdecchrs,
    gethexchrs, getoctchrs, multi_kind, peekchr, read_limits, skipchr, skipchr_keepstart, spelled,
    unmagic,
};
use core::cell::Cell;

/// An atom, optionally followed by a repeat.
///
/// A `SIMPLE` atom — one that matches exactly one character and never
/// backtracks into itself — gets a single `STAR`/`PLUS`/`BRACE_SIMPLE` node
/// the matcher can run as a counted loop. Anything else has to be wired up
/// as an explicit branch-and-back loop in the program.
pub(crate) fn regpiece(rc: &mut RegCompiler, flagp: &mut c_int) -> Compiled<Node> {
    let mut flags = 0;
    let ret = regatom(rc, &mut flags)?;

    let op = peekchr(rc);
    if multi_kind(op).is_none() {
        *flagp = flags;
        return Ok(ret);
    }
    // A repeat can match nothing, so whatever it wraps stops guaranteeing
    // width and becomes a possible start-of-pattern position.
    *flagp = WORST | SPSTART | (flags & (HASNL | HASLOOKBH));
    skipchr(rc);

    match op {
        M_STAR => {
            if flags & SIMPLE != 0 {
                rc.code.insert(BtOp::Star, ret);
            } else {
                // BRANCH ret BACK->ret BRANCH NOTHING: enter the loop or
                // skip it, and loop back after each pass.
                rc.code.insert(BtOp::Branch, ret);
                let node = rc.code.node(BtOp::Back);
                rc.code.op_tail(ret, node);
                rc.code.op_tail(ret, ret);
                let node = rc.code.node(BtOp::Branch);
                rc.code.tail(ret, node);
                let node = rc.code.node(BtOp::Nothing);
                rc.code.tail(ret, node);
            }
        }
        M_PLUS => {
            if flags & SIMPLE != 0 {
                rc.code.insert(BtOp::Plus, ret);
            } else {
                // As `*`, but the first pass is not optional.
                let next = rc.code.node(BtOp::Branch);
                rc.code.tail(ret, next);
                let node = rc.code.node(BtOp::Back);
                rc.code.tail(node, ret);
                let node = rc.code.node(BtOp::Branch);
                rc.code.tail(next, node);
                let node = rc.code.node(BtOp::Nothing);
                rc.code.tail(ret, node);
            }
            *flagp = WORST | HASWIDTH | (flags & (HASNL | HASLOOKBH));
        }
        // `\@=`, `\@!`, `\@>`, `\@<=`, `\@<!` — the look-around family, with
        // an optional decimal in front giving the look-behind limit.
        M_AT => {
            let mut nr = getdecchrs(rc);
            let lop = match low_byte(unmagic(getchr(rc))) {
                b'=' => Some(BtOp::Match),
                b'!' => Some(BtOp::Nomatch),
                b'>' => Some(BtOp::Subpat),
                b'<' => match low_byte(unmagic(getchr(rc))) {
                    b'=' => Some(BtOp::Behind),
                    b'!' => Some(BtOp::Nobehind),
                    _ => None,
                },
                _ => None,
            };
            let Some(lop) = lop else {
                return Err(CompileError::InvalidAfterAt(spelled(rc)));
            };
            let behind = lop == BtOp::Behind || lop == BtOp::Nobehind;
            if behind {
                let node = rc.code.node(BtOp::Bhpos);
                rc.code.tail(ret, node);
                *flagp |= HASLOOKBH;
            }
            let node = rc.code.node(BtOp::End);
            rc.code.tail(ret, node);
            if behind {
                // A missing limit reads as -1; the node carries an unsigned
                // count, and 0 means "no limit given".
                nr = nr.max(0);
                rc.code.insert_nr(lop, number_as_u32(nr), ret);
            } else {
                rc.code.insert(lop, ret);
            }
        }
        M_QUESTION | M_EQUAL => {
            // BRANCH ret BRANCH NOTHING: take the atom or step over it.
            rc.code.insert(BtOp::Branch, ret);
            let node = rc.code.node(BtOp::Branch);
            rc.code.tail(ret, node);
            let next = rc.code.node(BtOp::Nothing);
            rc.code.tail(ret, next);
            rc.code.op_tail(ret, next);
        }
        M_BRACE => {
            let (mut minval, mut maxval) = (0, 0);
            read_limits(rc, &mut minval, &mut maxval)?;
            if flags & SIMPLE != 0 {
                rc.code.insert(BtOp::BraceSimple, ret);
                rc.code.insert_limits(
                    BtOp::BraceLimits,
                    minval.cast_unsigned(),
                    maxval.cast_unsigned(),
                    ret,
                );
            } else {
                // A complex `{}` needs a counter slot of its own at match
                // time, and there are only ten.
                if rc.complex_braces >= NSUBEXP as usize {
                    return Err(CompileError::TooManyComplexBraces(spelled(rc)));
                }
                rc.code.insert(BtOp::BRACE_COMPLEX[rc.complex_braces], ret);
                let node = rc.code.node(BtOp::Back);
                rc.code.op_tail(ret, node);
                rc.code.op_tail(ret, ret);
                rc.code.insert_limits(
                    BtOp::BraceLimits,
                    minval.cast_unsigned(),
                    maxval.cast_unsigned(),
                    ret,
                );
                rc.complex_braces += 1;
            }
            if minval > 0 && maxval > 0 {
                *flagp = HASWIDTH | (flags & (HASNL | HASLOOKBH));
            }
        }
        _ => {}
    }

    // A second multi in a row has nothing to repeat.
    if multi_kind(peekchr(rc)).is_some() {
        if peekchr(rc) == M_STAR {
            // `\*` under 'nomagic' is a literal star, so this message wants
            // the backslash whenever `*` is *not* magic — a looser test than
            // the one every other message here uses.
            return Err(CompileError::NestedStar(Spelled::bare_if(
                rc.magic >= MAGIC_ON,
            )));
        }
        let c = low_byte(unmagic(peekchr(rc)));
        return Err(CompileError::Nested(spelled(rc), c));
    }
    Ok(ret)
}

/// A run of pieces, plus the `\c`/`\C`/`\Z` and `\v`/`\m`/`\M`/`\V` switches,
/// which are not atoms at all: they change how the rest of the pattern is
/// read and emit nothing.
pub(crate) fn regconcat(rc: &mut RegCompiler, flagp: &mut c_int) -> Compiled<Node> {
    let mut first: Option<Node> = None;
    let mut chain: Option<Node> = None;
    *flagp = WORST;

    loop {
        match peekchr(rc) {
            // An empty concatenation still needs a node for its caller to
            // chain onto.
            NUL | M_BAR | M_AMP | M_PAREN_CLOSE => {
                return Ok(first.unwrap_or_else(|| rc.code.node(BtOp::Nothing)));
            }
            M_Z_UPPER => {
                rc.flags |= RF_ICOMBINE as u32;
                skipchr_keepstart(rc);
            }
            M_C_LOWER => {
                rc.flags |= RF_ICASE as u32;
                skipchr_keepstart(rc);
            }
            M_C_UPPER => {
                rc.flags |= RF_NOICASE as u32;
                skipchr_keepstart(rc);
            }
            M_V_LOWER => set_magic(rc, MAGIC_ALL),
            M_M_LOWER => set_magic(rc, MAGIC_ON),
            M_M_UPPER => set_magic(rc, MAGIC_OFF),
            M_V_UPPER => set_magic(rc, MAGIC_NONE),
            _ => {
                let mut flags = 0;
                let latest = regpiece(rc, &mut flags)?;
                if rc.code.too_long {
                    return Err(CompileError::TooLong);
                }
                *flagp |= flags & (HASWIDTH | HASNL | HASLOOKBH);
                match chain {
                    // Only the first piece can start the match.
                    None => *flagp |= flags & SPSTART,
                    Some(chain) => rc.code.tail(chain, latest),
                }
                chain = Some(latest);
                first.get_or_insert(latest);
            }
        }
    }
}

/// The concatenations of one alternative, joined by `\&`: all of them have to
/// match at this position, and the last one's match is the alternative's.
pub(crate) fn regbranch(rc: &mut RegCompiler, flagp: &mut c_int) -> Compiled<Node> {
    let mut chain: Option<Node> = None;
    *flagp = WORST | HASNL;
    let ret = rc.code.node(BtOp::Branch);

    loop {
        let mut flags = 0;
        let latest = regconcat(rc, &mut flags)?;
        *flagp |= flags & (HASWIDTH | SPSTART | HASLOOKBH);
        // HASNL only survives if every concatenation has it.
        *flagp &= !HASNL | (flags & HASNL);
        if let Some(chain) = chain {
            rc.code.tail(chain, latest);
        }
        if peekchr(rc) != M_AMP {
            return Ok(ret);
        }
        skipchr(rc);
        let node = rc.code.node(BtOp::End);
        rc.code.tail(latest, node);
        if rc.code.too_long {
            return Ok(ret);
        }
        rc.code.insert(BtOp::Match, latest);
        chain = Some(latest);
    }
}

/// A whole pattern, or the body of one bracket.
///
/// `paren` says which bracket we are inside, and therefore which capture
/// node pair wraps the branches: `\(`..`\)`, `\z(`..`\)`, `\%(`..`\)` or
/// nothing at all for the outermost call.
pub(crate) fn reg(rc: &mut RegCompiler, paren: c_int, flagp: &mut c_int) -> Compiled<Node> {
    let mut parno = 0;
    *flagp = HASWIDTH;

    let open = match paren {
        REG_ZPAREN => {
            if rc.next_zgroup >= NSUBEXP as usize {
                return Err(CompileError::TooManyZGroups);
            }
            parno = rc.next_zgroup;
            rc.next_zgroup = parno + 1;
            Some(rc.code.node(BtOp::ZOPEN[parno]))
        }
        REG_PAREN => {
            if rc.next_group >= NSUBEXP as usize {
                return Err(CompileError::TooManyGroups(spelled(rc)));
            }
            parno = rc.next_group;
            rc.next_group = parno + 1;
            Some(rc.code.node(BtOp::MOPEN[parno]))
        }
        REG_NPAREN => Some(rc.code.node(BtOp::Nopen)),
        _ => None,
    };

    let mut flags = 0;
    let br = regbranch(rc, &mut flags)?;
    let ret = match open {
        None => br,
        Some(open) => {
            rc.code.tail(open, br);
            open
        }
    };
    let take_flags = |flagp: &mut c_int, flags: c_int| {
        if flags & HASWIDTH == 0 {
            *flagp &= !HASWIDTH;
        }
        *flagp |= flags & (SPSTART | HASNL | HASLOOKBH);
    };
    take_flags(flagp, flags);

    while peekchr(rc) == M_BAR {
        skipchr(rc);
        let br = regbranch(rc, &mut flags)?;
        if rc.code.too_long {
            return Err(CompileError::TooLong);
        }
        rc.code.tail(ret, br);
        take_flags(flagp, flags);
    }

    // Every branch's tail, and every branch's operand tail, ends at the
    // closing node.
    let ender = rc.code.node(match paren {
        REG_ZPAREN => BtOp::ZCLOSE[parno],
        REG_PAREN => BtOp::MCLOSE[parno],
        REG_NPAREN => BtOp::Nclose,
        _ => BtOp::End,
    });
    rc.code.tail(ret, ender);
    let mut br = Some(ret);
    while let Some(node) = br {
        rc.code.op_tail(node, ender);
        br = rc.code.next(node);
    }

    if paren != REG_NOPAREN && getchr(rc) != M_PAREN_CLOSE {
        return Err(match paren {
            REG_ZPAREN => CompileError::UnmatchedZGroup,
            REG_NPAREN => CompileError::UnmatchedNoCapture(spelled(rc)),
            _ => CompileError::UnmatchedGroup(spelled(rc)),
        });
    } else if paren == REG_NOPAREN && peekchr(rc) != NUL {
        if rc.token == M_PAREN_CLOSE {
            return Err(CompileError::UnmatchedClose(spelled(rc)));
        }
        return Err(CompileError::Trailing);
    }
    if paren == REG_PAREN {
        had_endbrace_seen(rc, parno);
    }
    Ok(ret)
}

/// Note that group `parno` has closed, which is what makes a later `\N`
/// back-reference to it legal.
fn had_endbrace_seen(rc: &mut RegCompiler, parno: usize) {
    rc.closed_groups[parno] = 1;
}

/// Compile `rc`'s pattern into a backtracking program, or answer why not.
///
/// One pass of `reg` writes the program (upstream's first, measuring, pass
/// is not needed: see [`super::compile`]). Afterwards the head of the program
/// is inspected for a required first character or a required substring, which
/// [`super::exec`] uses to skip start positions cheaply.
pub(crate) fn bt_regcomp(rc: &mut RegCompiler) -> Compiled<BtRegProg> {
    let mut flags = 0;
    rc.restart();
    // Most patterns compile to a few bytes per pattern byte; a `[:class:]`
    // is the outlier, and a reallocation covers it.
    rc.code.reserve(4 * rc.pattern_len + 32);
    rc.code.byte(REGMAGIC);
    // A refusal found after the program got too long is said with the
    // length after it: `bt_compile` reads `too_long` for that.
    reg(rc, REG_NOPAREN, &mut flags)?;
    if rc.code.too_long {
        return Err(CompileError::TooLong);
    }
    let mut regflags = rc.flags;
    if flags & HASNL != 0 {
        regflags |= RF_HASNL as u32;
    }
    if flags & HASLOOKBH != 0 {
        regflags |= RF_LOOKBH as u32;
    }
    if rc.had_eol != 0 {
        regflags |= RF_HAD_EOL;
    }
    let mut prog = BtRegProg {
        regflags,
        regstart: NUL,
        reganch: 0,
        regmust: None,
        regmlen: Cell::new(0),
        reghasz: low_byte(rc.has_z),
        program: rc.code.take_text(),
    };
    find_shortcuts(&mut prog, flags);
    Ok(prog)
}

/// Fill in the `regstart`/`reganch`/`regmust` hints the executor uses to
/// rule out start positions without running the program.
///
/// Only worth doing when the whole pattern is one branch: `regnext` past the
/// leading `BRANCH` landing on `END` is what proves there is no `\|`.
fn find_shortcuts(prog: &mut BtRegProg, flags: c_int) {
    let code = &prog.program;
    let opcode_at = |p: usize| opcode_at(code, p);
    let mut scan = 1;
    if opcode_at(regnext(code, scan)) != Some(BtOp::End) {
        return;
    }
    scan += 3;

    // A pattern anchored at the start only has to be tried there.
    if matches!(opcode_at(scan), Some(BtOp::Bol | BtOp::ReBof)) {
        prog.reganch = 1;
        scan = regnext(code, scan);
    }

    // A known first character lets the executor use a memchr-style skip.
    let first = opcode_at(scan);
    if first == Some(BtOp::Exactly) {
        prog.regstart = char_at_nul(code, scan + 3).get();
    } else if matches!(
        first,
        Some(
            BtOp::Bow
                | BtOp::Eow
                | BtOp::Nothing
                | BtOp::Mopen
                | BtOp::Nopen
                | BtOp::Mclose
                | BtOp::Nclose
        )
    ) {
        // Those all match empty, so look one node further.
        let next = regnext(code, scan);
        if opcode_at(next) == Some(BtOp::Exactly) {
            prog.regstart = char_at_nul(code, next + 3).get();
        }
    }

    // A required substring: the longest EXACTLY anywhere in the single
    // branch has to appear somewhere in the line for the line to match.
    // Only sound when the pattern can start anywhere in it, and never
    // across a line break.
    if (flags & SPSTART != 0 || matches!(opcode_at(scan), Some(BtOp::Bow | BtOp::Eow)))
        && flags & HASNL == 0
    {
        let mut longest = None;
        let mut len = 0;
        while scan != 0 {
            if opcode_at(scan) == Some(BtOp::Exactly) {
                let scanlen = cstr::until_nul(&code[scan + 3..]).len();
                // `>=` rather than `>`: upstream prefers the *last* of
                // equally long candidates.
                if scanlen >= len {
                    longest = Some(scan + 3);
                    len = scanlen;
                }
            }
            scan = regnext(code, scan);
        }
        prog.regmust = longest;
        prog.regmlen.set(byte_count(len));
    }
}

/// The character a `\d123`, `\o40`, `\x2f`, `\u1234` or `\U12345678` escape
/// names inside a `[]` collection, with the cursor just past the backslash.
///
/// Anything else leaves the cursor where it was and stands for a literal
/// backslash.
pub(crate) fn coll_get_char(rc: &mut RegCompiler) -> c_int {
    let start = rc.cursor;
    rc.cursor = start + 1;
    let mut nr = match cstr::byte_at(rc.text, start) {
        b'd' => getdecchrs(rc),
        b'o' => getoctchrs(rc),
        b'x' => gethexchrs(rc, 2),
        b'u' => gethexchrs(rc, 4),
        b'U' => gethexchrs(rc, 8),
        _ => -1,
    };
    if nr < 0 {
        // Not an escape after all: the backslash stands for itself.
        rc.cursor = start;
        nr = int64_t::from(b'\\');
    }
    c_int::try_from(nr).unwrap_or(c_int::MAX)
}

/// The opcode of node `p` of `code`, or `None` when `p` is 0 (no node) or
/// holds no opcode.
fn opcode_at(code: &[u8], p: usize) -> Option<BtOp> {
    if p == 0 {
        return None;
    }
    BtOp::decode(code[p]).ok().map(|(op, _)| op)
}
