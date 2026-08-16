//! `[abc]`: a collection, compiled to an `ANYOF`/`ANYBUT` node whose operand
//! is every character it accepts, as a NUL-terminated string.
//!
//! The operand is a *set*, so a range expands to its members and a
//! `[:alpha:]` class to the characters it contains. That is why a range over
//! multibyte characters is capped: it would otherwise write the whole span
//! into the program.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_int, c_uint};

use super::compile::{regc, regmbc, regnode, set_opcode};
use super::equi_class::reg_equi_class;
use super::piece::coll_get_char;
use crate::semsg;
use crate::src::nvim::ascii::{ascii_isdigit, ascii_isxdigit};
use crate::src::nvim::charset::{vim_isIDc, vim_isfilec, vim_isprintc};
use crate::src::nvim::main::rc_did_emsg;
use crate::src::nvim::mbyte::{mb_islower, mb_isupper, utf_char2len};
use crate::src::nvim::os::libc::__ctype_b_loc;
use crate::src::nvim::regexp::{
    _ISalnum, _ISalpha, _IScntrl, _ISgraph, _ISpunct, ADD_NL, ANYBUT, ANYOF, CLASS_ALNUM,
    CLASS_ALPHA, CLASS_BACKSPACE, CLASS_BLANK, CLASS_CNTRL, CLASS_DIGIT, CLASS_ESCAPE, CLASS_FNAME,
    CLASS_GRAPH, CLASS_IDENT, CLASS_KEYWORD, CLASS_LOWER, CLASS_NONE, CLASS_PRINT, CLASS_PUNCT,
    CLASS_RETURN, CLASS_SPACE, CLASS_TAB, CLASS_UPPER, CLASS_XDIGIT, ESC, HASNL, HASWIDTH, INT_MAX,
    JUST_CALC_SIZE, MAGIC_OFF, NUL, REGEXP_ABBR, REGEXP_INRANGE, Rex, SIMPLE, backslash_abbr,
    pat_byte, pat_char, pat_charlen, pat_seek, prevchr_len, reg_cpo_lit, reg_iswordc, reg_magic,
    reg_strict, regparse, skip_anyof, skipchr, take_bracketed, take_char_class,
};
use crate::src::nvim::types::uint8_t;

/// What a `[` at the cursor turned out to be.
pub(crate) enum Collection {
    Node(*mut uint8_t),
    /// No closing `]`: the `[` is an ordinary character.
    NotACollection,
    /// Already reported.
    Failed,
}

/// Parse a collection whose `[` has already been consumed.
pub(crate) fn collection(rex: Rex, flagp: &mut c_int, extra: c_int) -> Collection {
    if !collection_closes() {
        if reg_strict.get() != 0 {
            // Not `magic_prefix`: this one wants the backslash whenever `[`
            // is not magic, which is one 'magic' level lower.
            let prefix = if reg_magic.get() > MAGIC_OFF {
                ""
            } else {
                "\\"
            };
            semsg!("E769: Missing ] after {prefix}[");
            rc_did_emsg.set(true);
            return Collection::Failed;
        }
        return Collection::NotACollection;
    }

    let negated = pat_byte(0) == b'^';
    if negated {
        pat_seek(1);
    }
    let ret = regnode(if negated { ANYBUT } else { ANYOF } + extra);
    // `[\n]` widens the node itself, but only a plain `ANYOF` — `[^\n]` and
    // `\_[...]` are already what they need to be.
    let widens_on_nl = !negated && extra == 0;

    // A `]` or `-` in the very first position is that character, not the
    // close and not a range.
    let mut startc = -1;
    if matches!(pat_byte(0), b']' | b'-') {
        startc = pat_byte(0) as c_int;
        regc(startc);
        pat_seek(1);
    }

    while !matches!(pat_byte(0), 0 | b']') {
        match pat_byte(0) {
            b'-' => {
                pat_seek(1);
                if let Some(failed) = range(&mut startc) {
                    return failed;
                }
            }
            b'\\' if escaped_here() => {
                pat_seek(1);
                if let Some(failed) = escaped(flagp, &mut startc, ret, widens_on_nl) {
                    return failed;
                }
            }
            b'[' => bracketed_item(rex, &mut startc),
            _ => {
                // An ordinary character, plus any combining marks: they are
                // emitted as-is so that the set holds the whole grapheme.
                startc = pat_char(0);
                let len = pat_charlen(0);
                // A grapheme longer than its base character cannot be a
                // range endpoint.
                if utf_char2len(startc) != len {
                    startc = -1;
                }
                for _ in 0..len {
                    regc(pat_byte(0) as c_int);
                    pat_seek(1);
                }
            }
        }
    }

    regc(NUL);
    // The collection was consumed byte by byte rather than through the
    // character reader, so the reader's idea of how far back one character is
    // has to be reset before `skipchr` steps over the `]`.
    prevchr_len.set(1);
    if pat_byte(0) != b']' {
        // `e_toomsbra`'s text, inlined: `semsg!` needs a literal.
        semsg!("E76: Too many [");
        rc_did_emsg.set(true);
        return Collection::Failed;
    }
    skipchr();
    *flagp |= HASWIDTH | SIMPLE;
    Collection::Node(ret)
}

/// Does the collection at the cursor have a closing `]`?
fn collection_closes() -> bool {
    // SAFETY: `regparse` points into the NUL-terminated pattern, and
    // `skip_anyof` stops at its NUL.
    unsafe { *skip_anyof(regparse.get()) as u8 == b']' }
}

/// A `-` inside the collection: either a range from `startc`, or a literal
/// dash. Returns `Some` only on error.
fn range(startc: &mut c_int) -> Option<Collection> {
    // A dash is literal at the end, with nothing in front of it, or before a
    // `\n` — which is a line break rather than a character.
    if matches!(pat_byte(0), b']' | 0)
        || *startc == -1
        || (pat_byte(0) == b'\\' && pat_byte(1) == b'n')
    {
        regc(b'-' as c_int);
        *startc = b'-' as c_int;
        return None;
    }

    let mut endc = 0;
    if pat_byte(0) == b'[' {
        endc = take_cursor_bracketed(b'.');
    }
    if endc == 0 {
        endc = pat_char(0);
        pat_seek(pat_charlen(0) as isize);
    }
    if endc == b'\\' as c_int && reg_cpo_lit.get() == 0 {
        endc = coll_get_char();
    }
    if *startc > endc {
        semsg!("E944: Reverse range in character class");
        rc_did_emsg.set(true);
        return Some(Collection::Failed);
    }
    let multibyte = utf_char2len(*startc) > 1 || utf_char2len(endc) > 1;
    if multibyte {
        // Every member is written into the program, so a wide range would
        // blow it up.
        if endc > *startc + 256 {
            semsg!("E945: Range too large in character class");
            rc_did_emsg.set(true);
            return Some(Collection::Failed);
        }
        for c in *startc + 1..=endc {
            regmbc(c);
        }
    } else {
        for c in *startc + 1..=endc {
            regc(c);
        }
    }
    *startc = -1;
    None
}

/// Does the backslash at the cursor escape something, or is it a literal
/// backslash? `[]^-n\` always escape; the `\r`/`\t` abbreviations only when
/// 'cpoptions' does not contain `l`.
fn escaped_here() -> bool {
    let next = pat_byte(1);
    REGEXP_INRANGE.to_bytes().contains(&next)
        || (reg_cpo_lit.get() == 0 && REGEXP_ABBR.to_bytes().contains(&next))
}

/// The character after a backslash inside the collection, with the cursor
/// already past the backslash. Returns `Some` only on error.
fn escaped(
    flagp: &mut c_int,
    startc: &mut c_int,
    ret: *mut uint8_t,
    widens_on_nl: bool,
) -> Option<Collection> {
    match pat_byte(0) {
        b'n' => {
            // A line break is not a member of the set but a widening of the
            // node itself.
            if ret != JUST_CALC_SIZE && widens_on_nl {
                set_opcode(ret, ANYOF + ADD_NL);
                *flagp |= HASNL;
            }
            pat_seek(1);
            *startc = -1;
            None
        }
        b'd' | b'o' | b'x' | b'u' | b'U' => {
            *startc = coll_get_char();
            if *startc == INT_MAX {
                semsg!("E1541: Value too large, max Unicode codepoint is U+10FFFF");
                rc_did_emsg.set(true);
                return Some(Collection::Failed);
            }
            // As elsewhere, a NUL in the pattern stands for a newline.
            if *startc == 0 {
                regc(0xa);
            } else {
                regmbc(*startc);
            }
            None
        }
        c => {
            pat_seek(1);
            *startc = backslash_abbr(c as c_int);
            regc(*startc);
            None
        }
    }
}

/// A `[` inside the collection: a `[:alpha:]` class, a `[=a=]` equivalence
/// class, a `[.a.]` collation element, or a literal `[`.
fn bracketed_item(rex: Rex, startc: &mut c_int) {
    let class = take_cursor_char_class();
    *startc = -1;
    if class != CLASS_NONE as c_int {
        emit_char_class(rex, class);
        return;
    }
    let equi = take_cursor_bracketed(b'=');
    if equi != 0 {
        reg_equi_class(equi);
        return;
    }
    let coll = take_cursor_bracketed(b'.');
    if coll != 0 {
        regmbc(coll);
        return;
    }
    *startc = pat_byte(0) as c_int;
    regc(*startc);
    pat_seek(1);
}

/// [`take_char_class`] against the parse cursor.
fn take_cursor_char_class() -> c_int {
    // SAFETY: the cursor points into the NUL-terminated pattern, and
    // `take_char_class` only ever advances it.
    unsafe { take_char_class(&mut *regparse.ptr()) }
}

/// [`take_bracketed`] against the parse cursor.
fn take_cursor_bracketed(delim: u8) -> c_int {
    // SAFETY: as `take_cursor_char_class`.
    unsafe { take_bracketed(&mut *regparse.ptr(), delim) }
}

/// Upstream's per-class predicates.
///
/// The ceilings the classes walk to are upstream's and are not uniform: the
/// ASCII-only ones stop at 127, the rest run the whole Latin-1 range.
fn class_ceiling(class: c_uint) -> Option<c_int> {
    match class {
        CLASS_ALNUM | CLASS_ALPHA | CLASS_CNTRL | CLASS_DIGIT | CLASS_GRAPH | CLASS_PUNCT => {
            Some(127)
        }
        CLASS_LOWER | CLASS_PRINT | CLASS_UPPER | CLASS_XDIGIT | CLASS_IDENT | CLASS_KEYWORD
        | CLASS_FNAME => Some(255),
        _ => None,
    }
}

/// Is `c` a member of `class`?
fn in_class(rex: Rex, class: c_uint, c: c_int) -> bool {
    // SAFETY: every predicate here is a pure test on a code point, reading
    // only locale or option state; the ctype table is indexable over the
    // range `class_ceiling` allows.
    unsafe {
        let ctype = |mask: c_uint| *(*__ctype_b_loc()).offset(c as isize) as c_uint & mask != 0;
        match class {
            CLASS_ALNUM => ctype(_ISalnum),
            CLASS_ALPHA => ctype(_ISalpha),
            CLASS_CNTRL => ctype(_IScntrl),
            CLASS_DIGIT => ascii_isdigit(c),
            CLASS_GRAPH => ctype(_ISgraph),
            CLASS_PUNCT => ctype(_ISpunct),
            // U+00AA and U+00BA are the ordinal indicators: lowercase
            // letters, but not the lower half of a case pair.
            CLASS_LOWER => mb_islower(c) && c != 170 && c != 186,
            CLASS_PRINT => vim_isprintc(c),
            CLASS_UPPER => mb_isupper(c),
            CLASS_XDIGIT => ascii_isxdigit(c),
            CLASS_IDENT => vim_isIDc(c),
            CLASS_KEYWORD => reg_iswordc(rex, c),
            CLASS_FNAME => vim_isfilec(c),
            _ => false,
        }
    }
}

/// Write out every character of a `[:name:]` class.
fn emit_char_class(rex: Rex, class: c_int) {
    let class = class as c_uint;
    if let Some(hi) = class_ceiling(class) {
        for c in 1..=hi {
            if in_class(rex, class, c) {
                regmbc(c);
            }
        }
        return;
    }
    // The rest are short literal sets.
    match class {
        CLASS_BLANK => {
            regc(b' ' as c_int);
            regc(b'\t' as c_int);
        }
        CLASS_SPACE => {
            for c in 9..=13 {
                regc(c);
            }
            regc(b' ' as c_int);
        }
        CLASS_TAB => regc(b'\t' as c_int),
        CLASS_RETURN => regc(b'\r' as c_int),
        CLASS_BACKSPACE => regc(0x08),
        CLASS_ESCAPE => regc(ESC),
        _ => {}
    }
}
