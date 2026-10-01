//! The parser above the atom: an atom with its multi, the branches around
//! it, and the whole pattern's program.
//!
//! `reg` -> `regbranch` -> `regconcat` -> `regpiece` -> `regatom` is the
//! grammar, one level per precedence step: alternation with `\|`, then
//! concatenation with `\&`, then a repeat suffix, then the atom itself.
//! Each level hands its caller a node and a bag of `HASWIDTH`/`SIMPLE`/
//! `SPSTART`/`HASNL`/`HASLOOKBH` flags describing what it built, and null
//! for "an error was already reported".

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use crate::regexp::RF_HAD_EOL;
use crate::regexp::RegCompiler;
use crate::regexp::set_magic;
use core::ffi::c_int;

use super::atom::regatom;
use super::compile::{
    BtProg, chain_next, regc, reginsert, reginsert_limits, reginsert_nr, regnext, regnode,
    regoptail, regtail,
};
use super::op::BtOp;
use crate::mbyte::utf_ptr2char;
use crate::memory::xfree;
use crate::regexp::state::rc_did_emsg;
use crate::regexp::{
    HASLOOKBH, HASNL, HASWIDTH, INT_MAX, JUST_CALC_SIZE, MAGIC_ALL, MAGIC_NONE, MAGIC_OFF,
    MAGIC_ON, NOT_MULTI, NSUBEXP, REG_NOPAREN, REG_NPAREN, REG_PAREN, REG_ZPAREN, REGMAGIC,
    RF_HASNL, RF_ICASE, RF_ICOMBINE, RF_LOOKBH, RF_NOICASE, SIMPLE, SPSTART, WORST, bt_regengine,
    getchr, getdecchrs, gethexchrs, getoctchrs, magic, magic_prefix, peekchr, re_multi_type,
    read_limits, skipchr, skipchr_keepstart, unmagic,
};
use crate::semsg;
use crate::types::{NUL, RegProg, int64_t, uint8_t, uint32_t};

const M_AMP: c_int = magic(b'&');
const M_AT: c_int = magic(b'@');
const M_BAR: c_int = magic(b'|');
const M_BRACE: c_int = magic(b'{');
const M_C_LOWER: c_int = magic(b'c');
const M_C_UPPER: c_int = magic(b'C');
const M_EQUAL: c_int = magic(b'=');
const M_M_LOWER: c_int = magic(b'm');
const M_M_UPPER: c_int = magic(b'M');
const M_PAREN_CLOSE: c_int = magic(b')');
const M_PLUS: c_int = magic(b'+');
const M_QUESTION: c_int = magic(b'?');
const M_STAR: c_int = magic(b'*');
const M_V_LOWER: c_int = magic(b'v');
const M_V_UPPER: c_int = magic(b'V');
const M_Z_UPPER: c_int = magic(b'Z');

/// Report an error and mark the compile as failed, so that
/// [`super::super::api`] does not go on to blame the engine.
macro_rules! fail {
    ($($msg:tt)*) => {{
        semsg!($($msg)*);
        rc_did_emsg.set(true);
        return core::ptr::null_mut();
    }};
}

/// An atom, optionally followed by a repeat.
///
/// A `SIMPLE` atom — one that matches exactly one character and never
/// backtracks into itself — gets a single `STAR`/`PLUS`/`BRACE_SIMPLE` node
/// the matcher can run as a counted loop. Anything else has to be wired up
/// as an explicit branch-and-back loop in the program.
pub(crate) fn regpiece(rc: &mut RegCompiler, flagp: &mut c_int) -> *mut uint8_t {
    let mut flags = 0;
    let ret = regatom(rc, &mut flags);
    if ret.is_null() {
        return core::ptr::null_mut();
    }

    let op = peekchr(rc);
    if re_multi_type(op) == NOT_MULTI {
        *flagp = flags;
        return ret;
    }
    // A repeat can match nothing, so whatever it wraps stops guaranteeing
    // width and becomes a possible start-of-pattern position.
    *flagp = WORST | SPSTART | (flags & (HASNL | HASLOOKBH));
    skipchr(rc);

    match op {
        M_STAR => {
            if flags & SIMPLE != 0 {
                reginsert(rc, BtOp::Star, ret);
            } else {
                // BRANCH ret BACK->ret BRANCH NOTHING: enter the loop or
                // skip it, and loop back after each pass.
                reginsert(rc, BtOp::Branch, ret);
                let node = regnode(rc, BtOp::Back);
                regoptail(rc, ret, node);
                regoptail(rc, ret, ret);
                let node = regnode(rc, BtOp::Branch);
                regtail(rc, ret, node);
                let node = regnode(rc, BtOp::Nothing);
                regtail(rc, ret, node);
            }
        }
        M_PLUS => {
            if flags & SIMPLE != 0 {
                reginsert(rc, BtOp::Plus, ret);
            } else {
                // As `*`, but the first pass is not optional.
                let next = regnode(rc, BtOp::Branch);
                regtail(rc, ret, next);
                let node = regnode(rc, BtOp::Back);
                regtail(rc, node, ret);
                let node = regnode(rc, BtOp::Branch);
                regtail(rc, next, node);
                let node = regnode(rc, BtOp::Nothing);
                regtail(rc, ret, node);
            }
            *flagp = WORST | HASWIDTH | (flags & (HASNL | HASLOOKBH));
        }
        // `\@=`, `\@!`, `\@>`, `\@<=`, `\@<!` — the look-around family, with
        // an optional decimal in front giving the look-behind limit.
        M_AT => {
            let mut nr = getdecchrs(rc);
            let lop = match unmagic(getchr(rc)) as u8 {
                b'=' => Some(BtOp::Match),
                b'!' => Some(BtOp::Nomatch),
                b'>' => Some(BtOp::Subpat),
                b'<' => match unmagic(getchr(rc)) as u8 {
                    b'=' => Some(BtOp::Behind),
                    b'!' => Some(BtOp::Nobehind),
                    _ => None,
                },
                _ => None,
            };
            let Some(lop) = lop else {
                let prefix = magic_prefix(rc);
                fail!("E59: Invalid character after {prefix}@");
            };
            let behind = lop == BtOp::Behind || lop == BtOp::Nobehind;
            if behind {
                let node = regnode(rc, BtOp::Bhpos);
                regtail(rc, ret, node);
                *flagp |= HASLOOKBH;
            }
            let node = regnode(rc, BtOp::End);
            regtail(rc, ret, node);
            if behind {
                // A missing limit reads as -1; the node carries an unsigned
                // count, and 0 means "no limit given".
                nr = nr.max(0);
                reginsert_nr(rc, lop, nr as uint32_t as int64_t, ret);
            } else {
                reginsert(rc, lop, ret);
            }
        }
        M_QUESTION | M_EQUAL => {
            // BRANCH ret BRANCH NOTHING: take the atom or step over it.
            reginsert(rc, BtOp::Branch, ret);
            let node = regnode(rc, BtOp::Branch);
            regtail(rc, ret, node);
            let next = regnode(rc, BtOp::Nothing);
            regtail(rc, ret, next);
            regoptail(rc, ret, next);
        }
        M_BRACE => {
            let (mut minval, mut maxval) = (0, 0);
            if read_limits(rc, &mut minval, &mut maxval).is_err() {
                return core::ptr::null_mut();
            }
            if flags & SIMPLE != 0 {
                reginsert(rc, BtOp::BraceSimple, ret);
                reginsert_limits(
                    rc,
                    BtOp::BraceLimits,
                    minval as int64_t,
                    maxval as int64_t,
                    ret,
                );
            } else {
                // A complex `{}` needs a counter slot of its own at match
                // time, and there are only ten.
                if rc.complex_braces >= NSUBEXP as c_int {
                    let prefix = magic_prefix(rc);
                    fail!("E60: Too many complex {prefix}{{...}}s");
                }
                reginsert(rc, BtOp::BRACE_COMPLEX[slot(rc.complex_braces)], ret);
                let node = regnode(rc, BtOp::Back);
                regoptail(rc, ret, node);
                regoptail(rc, ret, ret);
                reginsert_limits(
                    rc,
                    BtOp::BraceLimits,
                    minval as int64_t,
                    maxval as int64_t,
                    ret,
                );
                rc.complex_braces = rc.complex_braces + 1;
            }
            if minval > 0 && maxval > 0 {
                *flagp = HASWIDTH | (flags & (HASNL | HASLOOKBH));
            }
        }
        _ => {}
    }

    // A second multi in a row has nothing to repeat.
    if re_multi_type(peekchr(rc)) != NOT_MULTI {
        if peekchr(rc) == M_STAR {
            // `\*` under 'nomagic' is a literal star, so this message wants
            // the backslash whenever `*` is *not* magic — a looser test than
            // the one every other message here uses.
            let prefix = if rc.magic >= MAGIC_ON { "" } else { "\\" };
            fail!("E61: Nested {prefix}*");
        }
        let prefix = magic_prefix(rc);
        let c = unmagic(peekchr(rc)) as u8 as char;
        fail!("E62: Nested {prefix}{c}");
    }
    ret
}

/// A run of pieces, plus the `\c`/`\C`/`\Z` and `\v`/`\m`/`\M`/`\V` switches,
/// which are not atoms at all: they change how the rest of the pattern is
/// read and emit nothing.
pub(crate) fn regconcat(rc: &mut RegCompiler, flagp: &mut c_int) -> *mut uint8_t {
    let mut first: *mut uint8_t = core::ptr::null_mut();
    let mut chain: *mut uint8_t = core::ptr::null_mut();
    *flagp = WORST;

    loop {
        match peekchr(rc) {
            NUL | M_BAR | M_AMP | M_PAREN_CLOSE => return finish_concat(rc, first),
            M_Z_UPPER => {
                rc.flags = rc.flags | RF_ICOMBINE as u32;
                skipchr_keepstart(rc);
            }
            M_C_LOWER => {
                rc.flags = rc.flags | RF_ICASE as u32;
                skipchr_keepstart(rc);
            }
            M_C_UPPER => {
                rc.flags = rc.flags | RF_NOICASE as u32;
                skipchr_keepstart(rc);
            }
            M_V_LOWER => set_magic(rc, MAGIC_ALL),
            M_M_LOWER => set_magic(rc, MAGIC_ON),
            M_M_UPPER => set_magic(rc, MAGIC_OFF),
            M_V_UPPER => set_magic(rc, MAGIC_NONE),
            _ => {
                let mut flags = 0;
                let latest = regpiece(rc, &mut flags);
                if latest.is_null() || rc.code.too_long != 0 {
                    return core::ptr::null_mut();
                }
                *flagp |= flags & (HASWIDTH | HASNL | HASLOOKBH);
                if chain.is_null() {
                    // Only the first piece can start the match.
                    *flagp |= flags & SPSTART;
                } else {
                    regtail(rc, chain, latest);
                }
                chain = latest;
                if first.is_null() {
                    first = latest;
                }
            }
        }
    }
}

/// An empty concatenation still needs a node for its caller to chain onto.
fn finish_concat(rc: &mut RegCompiler, first: *mut uint8_t) -> *mut uint8_t {
    if first.is_null() {
        regnode(rc, BtOp::Nothing)
    } else {
        first
    }
}

/// The concatenations of one alternative, joined by `\&`: all of them have to
/// match at this position, and the last one's match is the alternative's.
pub(crate) fn regbranch(rc: &mut RegCompiler, flagp: &mut c_int) -> *mut uint8_t {
    let mut chain: *mut uint8_t = core::ptr::null_mut();
    *flagp = WORST | HASNL;
    let ret = regnode(rc, BtOp::Branch);

    loop {
        let mut flags = 0;
        let latest = regconcat(rc, &mut flags);
        if latest.is_null() {
            return core::ptr::null_mut();
        }
        *flagp |= flags & (HASWIDTH | SPSTART | HASLOOKBH);
        // HASNL only survives if every concatenation has it.
        *flagp &= !HASNL | (flags & HASNL);
        if !chain.is_null() {
            regtail(rc, chain, latest);
        }
        if peekchr(rc) != M_AMP {
            return ret;
        }
        skipchr(rc);
        let node = regnode(rc, BtOp::End);
        regtail(rc, latest, node);
        if rc.code.too_long != 0 {
            return ret;
        }
        reginsert(rc, BtOp::Match, latest);
        chain = latest;
    }
}

/// A whole pattern, or the body of one bracket.
///
/// `paren` says which bracket we are inside, and therefore which capture
/// node pair wraps the branches: `\(`..`\)`, `\z(`..`\)`, `\%(`..`\)` or
/// nothing at all for the outermost call.
pub(crate) fn reg(rc: &mut RegCompiler, paren: c_int, flagp: &mut c_int) -> *mut uint8_t {
    let mut parno = 0;
    *flagp = HASWIDTH;

    let mut ret = match paren {
        REG_ZPAREN => {
            if rc.next_zgroup >= NSUBEXP as c_int {
                fail!("E50: Too many \\z(");
            }
            parno = rc.next_zgroup;
            rc.next_zgroup = parno + 1;
            regnode(rc, BtOp::ZOPEN[slot(parno)])
        }
        REG_PAREN => {
            if rc.next_group >= NSUBEXP as c_int {
                let prefix = magic_prefix(rc);
                fail!("E51: Too many {prefix}(");
            }
            parno = rc.next_group;
            rc.next_group = parno + 1;
            regnode(rc, BtOp::MOPEN[slot(parno)])
        }
        REG_NPAREN => regnode(rc, BtOp::Nopen),
        _ => core::ptr::null_mut(),
    };

    let mut flags = 0;
    let mut br = regbranch(rc, &mut flags);
    if br.is_null() {
        return core::ptr::null_mut();
    }
    if ret.is_null() {
        ret = br;
    } else {
        regtail(rc, ret, br);
    }
    let take_flags = |flagp: &mut c_int, flags: c_int| {
        if flags & HASWIDTH == 0 {
            *flagp &= !HASWIDTH;
        }
        *flagp |= flags & (SPSTART | HASNL | HASLOOKBH);
    };
    take_flags(flagp, flags);

    while peekchr(rc) == M_BAR {
        skipchr(rc);
        br = regbranch(rc, &mut flags);
        if br.is_null() || rc.code.too_long != 0 {
            return core::ptr::null_mut();
        }
        regtail(rc, ret, br);
        take_flags(flagp, flags);
    }

    // Every branch's tail, and every branch's operand tail, ends at the
    // closing node.
    let ender = regnode(
        rc,
        match paren {
            REG_ZPAREN => BtOp::ZCLOSE[slot(parno)],
            REG_PAREN => BtOp::MCLOSE[slot(parno)],
            REG_NPAREN => BtOp::Nclose,
            _ => BtOp::End,
        },
    );
    regtail(rc, ret, ender);
    let mut br = ret;
    while !br.is_null() {
        regoptail(rc, br, ender);
        br = chain_next(rc, br);
    }

    if paren != REG_NOPAREN && getchr(rc) != M_PAREN_CLOSE {
        match paren {
            REG_ZPAREN => {
                fail!("E52: Unmatched \\z(");
            }
            REG_NPAREN => {
                let prefix = magic_prefix(rc);
                fail!("E53: Unmatched {prefix}%(");
            }
            _ => {
                let prefix = magic_prefix(rc);
                fail!("E54: Unmatched {prefix}(");
            }
        }
    } else if paren == REG_NOPAREN && peekchr(rc) != NUL {
        if rc.token == M_PAREN_CLOSE {
            let prefix = magic_prefix(rc);
            fail!("E55: Unmatched {prefix})");
        }
        // `e_trailing`'s text, inlined: `semsg!` needs a literal.
        fail!("E488: Trailing characters");
    }
    if paren == REG_PAREN {
        had_endbrace_seen(rc, parno);
    }
    ret
}

/// Note that group `parno` has closed, which is what makes a later `\N`
/// back-reference to it legal.
fn had_endbrace_seen(rc: &mut RegCompiler, parno: c_int) {
    rc.closed_groups[parno as usize] = 1;
}

/// Compile `rc`'s pattern into a backtracking program, or answer null
/// having reported why not.
///
/// Twice over: the first `reg` pass only measures the program, the second
/// writes into the block that size bought. Afterwards the head of the program
/// is inspected for a required first character or a required substring, which
/// [`super::exec`] uses to skip start positions cheaply.
pub(crate) fn bt_regcomp(rc: &mut RegCompiler) -> *mut RegProg {
    let mut flags = 0;
    rc.restart();
    rc.code.code = JUST_CALC_SIZE;
    regc(rc, REGMAGIC);
    if reg(rc, REG_NOPAREN, &mut flags).is_null() {
        return core::ptr::null_mut();
    }

    let prog = BtProg::alloc(rc.code.size as usize);

    rc.restart();
    rc.code.code = prog.text();
    regc(rc, REGMAGIC);
    if reg(rc, REG_NOPAREN, &mut flags).is_null() || rc.code.too_long != 0 {
        prog.discard();
        if rc.code.too_long != 0 {
            semsg!("E339: Pattern too long");
            rc_did_emsg.set(true);
        }
        return core::ptr::null_mut();
    }

    prog.set_regstart(NUL);
    prog.set_anchored(false);
    prog.set_regmust(core::ptr::null_mut(), 0);
    prog.set_regflags(rc.flags);
    if flags & HASNL != 0 {
        prog.add_regflags(RF_HASNL as u32);
    }
    if flags & HASLOOKBH != 0 {
        prog.add_regflags(RF_LOOKBH as u32);
    }
    if rc.had_eol != 0 {
        prog.add_regflags(RF_HAD_EOL);
    }
    prog.set_reghasz(rc.has_z as uint8_t);
    find_shortcuts(prog, flags);
    prog.set_engine((&raw const bt_regengine).cast_mut());
    prog.into_regprog()
}

/// Fill in the `regstart`/`reganch`/`regmust` hints the executor uses to
/// rule out start positions without running the program.
///
/// Only worth doing when the whole pattern is one branch: `regnext` past the
/// leading `BRANCH` landing on `END` is what proves there is no `\|`.
///
/// `prog` must be a program this module has just finished writing.
fn find_shortcuts(prog: BtProg, flags: c_int) {
    // SAFETY: walking the program just written, whose nodes are well formed.
    let mut scan = prog.first_node();
    if opcode_at(regnext(scan)) != Some(BtOp::End) {
        return;
    }
    scan = unsafe { scan.add(3) };

    // A pattern anchored at the start only has to be tried there.
    if matches!(opcode_at(scan), Some(BtOp::Bol | BtOp::ReBof)) {
        prog.set_anchored(true);
        scan = regnext(scan);
    }

    // A known first character lets the executor use a memchr-style skip.
    let first = opcode_at(scan);
    if first == Some(BtOp::Exactly) {
        prog.set_regstart(unsafe { utf_ptr2char(scan.add(3).cast()) });
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
        let next = regnext(scan);
        if opcode_at(next) == Some(BtOp::Exactly) {
            prog.set_regstart(unsafe { utf_ptr2char(next.add(3).cast()) });
        }
    }

    // A required substring: the longest EXACTLY anywhere in the single
    // branch has to appear somewhere in the line for the line to match.
    // Only sound when the pattern can start anywhere in it, and never
    // across a line break.
    if (flags & SPSTART != 0 || matches!(opcode_at(scan), Some(BtOp::Bow | BtOp::Eow)))
        && flags & HASNL == 0
    {
        let mut longest: *mut uint8_t = core::ptr::null_mut();
        let mut len = 0;
        while !scan.is_null() {
            if opcode_at(scan) == Some(BtOp::Exactly) {
                let scanlen = unsafe { cstr::bytes_at(scan.add(3).cast()) }.len() as c_int;
                // `>=` rather than `>`: upstream prefers the *last* of
                // equally long candidates.
                if scanlen >= len {
                    longest = unsafe { scan.add(3) };
                    len = scanlen;
                }
            }
            scan = regnext(scan);
        }
        prog.set_regmust(longest, len);
    }
}

/// Did `prog`'s pattern end with a `\n` that a search should treat as
/// "match at end of line"? False for a null program. Read by the syntax
/// code right after a compile.
pub fn vim_regcomp_had_eol(prog: *const RegProg) -> bool {
    // SAFETY: a program `vim_regcomp` answered, or null.
    !prog.is_null() && unsafe { (*prog).regflags } & RF_HAD_EOL != 0
}

/// The character a `\d123`, `\o40`, `\x2f`, `\u1234` or `\U12345678` escape
/// names inside a `[]` collection, with the cursor just past the backslash.
///
/// Anything else leaves the cursor where it was and stands for a literal
/// backslash.
pub(crate) fn coll_get_char(rc: &mut RegCompiler) -> c_int {
    // SAFETY: `regparse` points into the NUL-terminated pattern, and the
    // readers below only ever advance it.
    let start = rc.cursor;
    rc.cursor = unsafe { start.add(1) };
    let mut nr = match unsafe { *start } as u8 {
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
        nr = b'\\' as int64_t;
    }
    nr.min(INT_MAX as int64_t) as c_int
}

/// # Safety
///
/// `prog` must be a program this module compiled, or null.
pub(crate) unsafe fn bt_regfree(prog: *mut RegProg) {
    // SAFETY: one `xmalloc` block, with nothing owned inside it.
    unsafe { xfree(prog.cast()) };
}

/// A capture group or complex-brace number as an index into one of
/// [`BtOp`]'s runs. The parser has already refused a tenth of either.
fn slot(number: c_int) -> usize {
    usize::try_from(number).expect("a group number")
}

/// The opcode of the node at `p`, or `None` when `p` is null or holds no
/// opcode.
///
/// # Safety
/// Only called on nodes of a program this module has just written.
fn opcode_at(p: *const uint8_t) -> Option<BtOp> {
    if p.is_null() {
        return None;
    }
    // SAFETY: the caller's node, whose opcode byte is readable.
    BtOp::decode(unsafe { *p }).ok().map(|(op, _)| op)
}
