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

use core::ffi::{c_char, c_int};
use core::mem::offset_of;

use super::atom::regatom;
use super::compile::{
    regc, regcomp_start, reginsert, reginsert_limits, reginsert_nr, regnext, regnode, regoptail,
    regtail,
};
use crate::semsg;
use crate::src::nvim::main::{e_null, rc_did_emsg};
use crate::src::nvim::mbyte::utf_ptr2char;
use crate::src::nvim::memory::{xfree, xmalloc};
use crate::src::nvim::message::iemsg;
use crate::src::nvim::os::libc::{gettext, strlen};
use crate::src::nvim::regexp::{
    BACK, BEHIND, BHPOS, BOL, BOW, BRACE_COMPLEX, BRACE_LIMITS, BRACE_SIMPLE, BRANCH, END, EOW,
    EXACTLY, HASLOOKBH, HASNL, HASWIDTH, INT_MAX, JUST_CALC_SIZE, MAGIC_ALL, MAGIC_NONE, MAGIC_OFF,
    MAGIC_ON, MATCH, MCLOSE, MOPEN, NCLOSE, NOBEHIND, NOMATCH, NOPEN, NOT_MULTI, NOTHING, NSUBEXP,
    NUL, PLUS, RE_BOF, REG_NOPAREN, REG_NPAREN, REG_PAREN, REG_ZPAREN, REGMAGIC, RF_HASNL,
    RF_ICASE, RF_ICOMBINE, RF_LOOKBH, RF_NOICASE, SIMPLE, SPSTART, STAR, SUBPAT, WORST, ZCLOSE,
    ZOPEN, bt_regengine, bt_regprog_T, curchr, getchr, getdecchrs, gethexchrs, getoctchrs,
    had_endbrace, had_eol, magic, magic_prefix, num_complex_braces, peekchr, re_has_z,
    re_multi_type, read_limits, reg_magic, reg_toolong, regcode, regflags, regnpar, regnzpar,
    regparse, regsize, skipchr, skipchr_keepstart, unmagic,
};
use crate::src::nvim::types::{int64_t, regprog_T, size_t, uint8_t, uint32_t};

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
pub(crate) fn regpiece(flagp: &mut c_int) -> *mut uint8_t {
    let mut flags = 0;
    // SAFETY: `regatom` is still transpiled and reads the pattern cursor.
    let ret = unsafe { regatom(&mut flags) };
    if ret.is_null() {
        return core::ptr::null_mut();
    }

    let op = peekchr();
    if re_multi_type(op) == NOT_MULTI {
        *flagp = flags;
        return ret;
    }
    // A repeat can match nothing, so whatever it wraps stops guaranteeing
    // width and becomes a possible start-of-pattern position.
    *flagp = WORST | SPSTART | (flags & (HASNL | HASLOOKBH));
    skipchr();

    match op {
        M_STAR => {
            if flags & SIMPLE != 0 {
                reginsert(STAR, ret);
            } else {
                // BRANCH ret BACK->ret BRANCH NOTHING: enter the loop or
                // skip it, and loop back after each pass.
                reginsert(BRANCH, ret);
                regoptail(ret, regnode(BACK));
                regoptail(ret, ret);
                regtail(ret, regnode(BRANCH));
                regtail(ret, regnode(NOTHING));
            }
        }
        M_PLUS => {
            if flags & SIMPLE != 0 {
                reginsert(PLUS, ret);
            } else {
                // As `*`, but the first pass is not optional.
                let next = regnode(BRANCH);
                regtail(ret, next);
                regtail(regnode(BACK), ret);
                regtail(next, regnode(BRANCH));
                regtail(ret, regnode(NOTHING));
            }
            *flagp = WORST | HASWIDTH | (flags & (HASNL | HASLOOKBH));
        }
        // `\@=`, `\@!`, `\@>`, `\@<=`, `\@<!` — the look-around family, with
        // an optional decimal in front giving the look-behind limit.
        M_AT => {
            let mut nr = getdecchrs();
            let lop = match unmagic(getchr()) as u8 {
                b'=' => MATCH,
                b'!' => NOMATCH,
                b'>' => SUBPAT,
                b'<' => match unmagic(getchr()) as u8 {
                    b'=' => BEHIND,
                    b'!' => NOBEHIND,
                    _ => END,
                },
                _ => END,
            };
            if lop == END {
                let prefix = magic_prefix();
                fail!("E59: Invalid character after {prefix}@");
            }
            let behind = lop == BEHIND || lop == NOBEHIND;
            if behind {
                regtail(ret, regnode(BHPOS));
                *flagp |= HASLOOKBH;
            }
            regtail(ret, regnode(END));
            if behind {
                // A missing limit reads as -1; the node carries an unsigned
                // count, and 0 means "no limit given".
                nr = nr.max(0);
                reginsert_nr(lop, nr as uint32_t as int64_t, ret);
            } else {
                reginsert(lop, ret);
            }
        }
        M_QUESTION | M_EQUAL => {
            // BRANCH ret BRANCH NOTHING: take the atom or step over it.
            reginsert(BRANCH, ret);
            regtail(ret, regnode(BRANCH));
            let next = regnode(NOTHING);
            regtail(ret, next);
            regoptail(ret, next);
        }
        M_BRACE => {
            let (mut minval, mut maxval) = (0, 0);
            if read_limits(&mut minval, &mut maxval) == 0 {
                return core::ptr::null_mut();
            }
            if flags & SIMPLE != 0 {
                reginsert(BRACE_SIMPLE, ret);
                reginsert_limits(BRACE_LIMITS, minval as int64_t, maxval as int64_t, ret);
            } else {
                // A complex `{}` needs a counter slot of its own at match
                // time, and there are only ten.
                if num_complex_braces.get() >= NSUBEXP as c_int {
                    let prefix = magic_prefix();
                    fail!("E60: Too many complex {prefix}{{...}}s");
                }
                reginsert(BRACE_COMPLEX + num_complex_braces.get(), ret);
                regoptail(ret, regnode(BACK));
                regoptail(ret, ret);
                reginsert_limits(BRACE_LIMITS, minval as int64_t, maxval as int64_t, ret);
                num_complex_braces.set(num_complex_braces.get() + 1);
            }
            if minval > 0 && maxval > 0 {
                *flagp = HASWIDTH | (flags & (HASNL | HASLOOKBH));
            }
        }
        _ => {}
    }

    // A second multi in a row has nothing to repeat.
    if re_multi_type(peekchr()) != NOT_MULTI {
        if peekchr() == M_STAR {
            // `\*` under 'nomagic' is a literal star, so this message wants
            // the backslash whenever `*` is *not* magic — a looser test than
            // the one every other message here uses.
            let prefix = if reg_magic.get() >= MAGIC_ON {
                ""
            } else {
                "\\"
            };
            fail!("E61: Nested {prefix}*");
        }
        let prefix = magic_prefix();
        let c = unmagic(peekchr()) as u8 as char;
        fail!("E62: Nested {prefix}{c}");
    }
    ret
}

/// A run of pieces, plus the `\c`/`\C`/`\Z` and `\v`/`\m`/`\M`/`\V` switches,
/// which are not atoms at all: they change how the rest of the pattern is
/// read and emit nothing.
pub(crate) fn regconcat(flagp: &mut c_int) -> *mut uint8_t {
    let mut first: *mut uint8_t = core::ptr::null_mut();
    let mut chain: *mut uint8_t = core::ptr::null_mut();
    *flagp = WORST;

    loop {
        let mut set_magic = |magic| {
            reg_magic.set(magic);
            skipchr_keepstart();
            // The switch changes what the next byte means, so the lookahead
            // taken before it has to be dropped.
            curchr.set(-1);
        };
        match peekchr() {
            NUL | M_BAR | M_AMP | M_PAREN_CLOSE => return finish_concat(first),
            M_Z_UPPER => {
                regflags.set(regflags.get() | RF_ICOMBINE as u32);
                skipchr_keepstart();
            }
            M_C_LOWER => {
                regflags.set(regflags.get() | RF_ICASE as u32);
                skipchr_keepstart();
            }
            M_C_UPPER => {
                regflags.set(regflags.get() | RF_NOICASE as u32);
                skipchr_keepstart();
            }
            M_V_LOWER => set_magic(MAGIC_ALL),
            M_M_LOWER => set_magic(MAGIC_ON),
            M_M_UPPER => set_magic(MAGIC_OFF),
            M_V_UPPER => set_magic(MAGIC_NONE),
            _ => {
                let mut flags = 0;
                let latest = regpiece(&mut flags);
                if latest.is_null() || reg_toolong.get() != 0 {
                    return core::ptr::null_mut();
                }
                *flagp |= flags & (HASWIDTH | HASNL | HASLOOKBH);
                if chain.is_null() {
                    // Only the first piece can start the match.
                    *flagp |= flags & SPSTART;
                } else {
                    regtail(chain, latest);
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
fn finish_concat(first: *mut uint8_t) -> *mut uint8_t {
    if first.is_null() {
        regnode(NOTHING)
    } else {
        first
    }
}

/// The concatenations of one alternative, joined by `\&`: all of them have to
/// match at this position, and the last one's match is the alternative's.
pub(crate) fn regbranch(flagp: &mut c_int) -> *mut uint8_t {
    let mut chain: *mut uint8_t = core::ptr::null_mut();
    *flagp = WORST | HASNL;
    let ret = regnode(BRANCH);

    loop {
        let mut flags = 0;
        let latest = regconcat(&mut flags);
        if latest.is_null() {
            return core::ptr::null_mut();
        }
        *flagp |= flags & (HASWIDTH | SPSTART | HASLOOKBH);
        // HASNL only survives if every concatenation has it.
        *flagp &= !HASNL | (flags & HASNL);
        if !chain.is_null() {
            regtail(chain, latest);
        }
        if peekchr() != M_AMP {
            return ret;
        }
        skipchr();
        regtail(latest, regnode(END));
        if reg_toolong.get() != 0 {
            return ret;
        }
        reginsert(MATCH, latest);
        chain = latest;
    }
}

/// A whole pattern, or the body of one bracket.
///
/// `paren` says which bracket we are inside, and therefore which capture
/// node pair wraps the branches: `\(`..`\)`, `\z(`..`\)`, `\%(`..`\)` or
/// nothing at all for the outermost call.
pub(crate) fn reg(paren: c_int, flagp: &mut c_int) -> *mut uint8_t {
    let mut parno = 0;
    *flagp = HASWIDTH;

    let mut ret = match paren {
        REG_ZPAREN => {
            if regnzpar.get() >= NSUBEXP as c_int {
                fail!("E50: Too many \\z(");
            }
            parno = regnzpar.get();
            regnzpar.set(parno + 1);
            regnode(ZOPEN + parno)
        }
        REG_PAREN => {
            if regnpar.get() >= NSUBEXP as c_int {
                let prefix = magic_prefix();
                fail!("E51: Too many {prefix}(");
            }
            parno = regnpar.get();
            regnpar.set(parno + 1);
            regnode(MOPEN + parno)
        }
        REG_NPAREN => regnode(NOPEN),
        _ => core::ptr::null_mut(),
    };

    let mut flags = 0;
    let mut br = regbranch(&mut flags);
    if br.is_null() {
        return core::ptr::null_mut();
    }
    if ret.is_null() {
        ret = br;
    } else {
        regtail(ret, br);
    }
    let mut take_flags = |flagp: &mut c_int, flags: c_int| {
        if flags & HASWIDTH == 0 {
            *flagp &= !HASWIDTH;
        }
        *flagp |= flags & (SPSTART | HASNL | HASLOOKBH);
    };
    take_flags(flagp, flags);

    while peekchr() == M_BAR {
        skipchr();
        br = regbranch(&mut flags);
        if br.is_null() || reg_toolong.get() != 0 {
            return core::ptr::null_mut();
        }
        regtail(ret, br);
        take_flags(flagp, flags);
    }

    // Every branch's tail, and every branch's operand tail, ends at the
    // closing node.
    let ender = regnode(match paren {
        REG_ZPAREN => ZCLOSE + parno,
        REG_PAREN => MCLOSE + parno,
        REG_NPAREN => NCLOSE,
        _ => END,
    });
    regtail(ret, ender);
    let mut br = ret;
    while !br.is_null() {
        regoptail(br, ender);
        br = regnext(br);
    }

    if paren != REG_NOPAREN && getchr() != M_PAREN_CLOSE {
        match paren {
            REG_ZPAREN => {
                fail!("E52: Unmatched \\z(");
            }
            REG_NPAREN => {
                let prefix = magic_prefix();
                fail!("E53: Unmatched {prefix}%(");
            }
            _ => {
                let prefix = magic_prefix();
                fail!("E54: Unmatched {prefix}(");
            }
        }
    } else if paren == REG_NOPAREN && peekchr() != NUL {
        if curchr.get() == M_PAREN_CLOSE {
            let prefix = magic_prefix();
            fail!("E55: Unmatched {prefix})");
        }
        // `e_trailing`'s text, inlined: `semsg!` needs a literal.
        fail!("E488: Trailing characters");
    }
    if paren == REG_PAREN {
        had_endbrace_seen(parno);
    }
    ret
}

/// Note that group `parno` has closed, which is what makes a later `\N`
/// back-reference to it legal.
fn had_endbrace_seen(parno: c_int) {
    let mut seen = had_endbrace.get();
    seen[parno as usize] = 1;
    had_endbrace.set(seen);
}

/// Compile `expr` into a backtracking program.
///
/// Twice over: the first `reg` pass only accumulates [`regsize`], the second
/// writes into the block that size bought. Afterwards the head of the program
/// is inspected for a required first character or a required substring, which
/// [`super::exec`] uses to skip start positions cheaply.
///
/// # Safety
///
/// `expr` must be a NUL-terminated pattern.
pub(crate) unsafe extern "C" fn bt_regcomp(expr: *mut uint8_t, re_flags: c_int) -> *mut regprog_T {
    // SAFETY: `expr` is a NUL-terminated pattern. `r` is this function's own
    // allocation, sized by the pass that runs before anything is written into
    // it, and is either handed to the caller or freed here.
    unsafe {
        if expr.is_null() {
            iemsg(gettext(&raw const e_null as *const c_char));
            rc_did_emsg.set(true);
            return core::ptr::null_mut();
        }

        let mut flags = 0;
        regcomp_start(expr, re_flags);
        regcode.set(JUST_CALC_SIZE);
        regc(REGMAGIC);
        if reg(REG_NOPAREN, &mut flags).is_null() {
            return core::ptr::null_mut();
        }

        let r: *mut bt_regprog_T = xmalloc(
            (offset_of!(bt_regprog_T, program) as size_t).wrapping_add(regsize.get() as size_t),
        )
        .cast();
        (*r).re_in_use = false;

        regcomp_start(expr, re_flags);
        regcode.set((&raw mut (*r).program).cast());
        regc(REGMAGIC);
        if reg(REG_NOPAREN, &mut flags).is_null() || reg_toolong.get() != 0 {
            xfree(r.cast());
            if reg_toolong.get() != 0 {
                semsg!("E339: Pattern too long");
                rc_did_emsg.set(true);
            }
            return core::ptr::null_mut();
        }

        (*r).regstart = NUL;
        (*r).reganch = 0;
        (*r).regmust = core::ptr::null_mut();
        (*r).regmlen = 0;
        (*r).regflags = regflags.get();
        if flags & HASNL != 0 {
            (*r).regflags |= RF_HASNL as u32;
        }
        if flags & HASLOOKBH != 0 {
            (*r).regflags |= RF_LOOKBH as u32;
        }
        (*r).reghasz = re_has_z.get() as uint8_t;
        find_shortcuts(r, flags);
        (*r).engine = bt_regengine.ptr();
        r.cast()
    }
}

/// Fill in the `regstart`/`reganch`/`regmust` hints the executor uses to
/// rule out start positions without running the program.
///
/// Only worth doing when the whole pattern is one branch: `regnext` past the
/// leading `BRANCH` landing on `END` is what proves there is no `\|`.
///
/// `r` must be a program this module has just finished writing.
fn find_shortcuts(r: *mut bt_regprog_T, flags: c_int) {
    // SAFETY: walking the program just written, whose nodes are well formed.
    unsafe {
        // Past the REGMAGIC byte.
        let mut scan = (&raw mut (*r).program).cast::<uint8_t>().add(1);
        if *regnext(scan) as c_int != END {
            return;
        }
        scan = scan.add(3);

        // A pattern anchored at the start only has to be tried there.
        if *scan as c_int == BOL || *scan as c_int == RE_BOF {
            (*r).reganch += 1;
            scan = regnext(scan);
        }

        // A known first character lets the executor use a memchr-style skip.
        if *scan as c_int == EXACTLY {
            (*r).regstart = utf_ptr2char(scan.add(3).cast());
        } else if [BOW, EOW, NOTHING, MOPEN, NOPEN, MCLOSE, NCLOSE].contains(&(*scan as c_int)) {
            // Those all match empty, so look one node further.
            let next = regnext(scan);
            if *next as c_int == EXACTLY {
                (*r).regstart = utf_ptr2char(next.add(3).cast());
            }
        }

        // A required substring: the longest EXACTLY anywhere in the single
        // branch has to appear somewhere in the line for the line to match.
        // Only sound when the pattern can start anywhere in it, and never
        // across a line break.
        if (flags & SPSTART != 0 || *scan as c_int == BOW || *scan as c_int == EOW)
            && flags & HASNL == 0
        {
            let mut longest: *mut uint8_t = core::ptr::null_mut();
            let mut len = 0;
            while !scan.is_null() {
                if *scan as c_int == EXACTLY {
                    let scanlen = strlen(scan.add(3).cast()) as c_int;
                    // `>=` rather than `>`: upstream prefers the *last* of
                    // equally long candidates.
                    if scanlen >= len {
                        longest = scan.add(3);
                        len = scanlen;
                    }
                }
                scan = regnext(scan);
            }
            (*r).regmust = longest;
            (*r).regmlen = len;
        }
    }
}

/// Did the pattern just compiled end with a `\n` that a search should treat
/// as "match at end of line"? Read by the search code right after a compile.
pub extern "C" fn vim_regcomp_had_eol() -> c_int {
    had_eol.get()
}

/// The character a `\d123`, `\o40`, `\x2f`, `\u1234` or `\U12345678` escape
/// names inside a `[]` collection, with the cursor just past the backslash.
///
/// Anything else leaves the cursor where it was and stands for a literal
/// backslash.
pub(crate) fn coll_get_char() -> c_int {
    // SAFETY: `regparse` points into the NUL-terminated pattern, and the
    // readers below only ever advance it.
    unsafe {
        let start = regparse.get();
        regparse.set(start.add(1));
        let mut nr = match *start as u8 {
            b'd' => getdecchrs(),
            b'o' => getoctchrs(),
            b'x' => gethexchrs(2),
            b'u' => gethexchrs(4),
            b'U' => gethexchrs(8),
            _ => -1,
        };
        if nr < 0 {
            // Not an escape after all: the backslash stands for itself.
            regparse.set(start);
            nr = b'\\' as int64_t;
        }
        nr.min(INT_MAX as int64_t) as c_int
    }
}

/// # Safety
///
/// `prog` must be a program this module compiled, or null.
pub(crate) unsafe extern "C" fn bt_regfree(prog: *mut regprog_T) {
    // SAFETY: one `xmalloc` block, with nothing owned inside it.
    unsafe { xfree(prog.cast()) };
}
