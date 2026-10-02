//! `[abc]`: a collection, compiled to an `ANYOF`/`ANYBUT` node whose operand
//! is every character it accepts, as a NUL-terminated string.
//!
//! The operand is a *set*, so a range expands to its members and a
//! `[:alpha:]` class to the characters it contains. That is why a range over
//! multibyte characters is capped: it would otherwise write the whole span
//! into the program.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::compile::Node;
use crate::charset::vim_iswordc_buf;
use crate::regexp::RegCompiler;
use core::ffi::{c_int, c_uint};

use super::equi_class::reg_equi_class;
use super::op::BtOp;
use super::piece::coll_get_char;
use crate::ascii::{ascii_isdigit, ascii_isxdigit};
use crate::charset::{vim_is_ident_char, vim_isfilec, vim_isprintc};
use crate::mbyte::{mb_islower, mb_isupper, utf_char2len};
use crate::os::cshim::__ctype_b_loc;
use crate::regexp::state::rc_did_emsg;
use crate::regexp::{
    _ISalnum, _ISalpha, _IScntrl, _ISgraph, _ISpunct, CharClass, ESC, HASNL, HASWIDTH, INT_MAX,
    MAGIC_OFF, REGEXP_ABBR, REGEXP_INRANGE, SIMPLE, backslash_abbr, pat_byte, pat_char,
    pat_charlen, pat_seek, skip_anyof, skipchr, take_bracketed, take_char_class,
};
use crate::semsg;
use crate::types::NUL;

/// What a `[` at the cursor turned out to be.
pub(crate) enum Collection {
    Node(Node),
    /// No closing `]`: the `[` is an ordinary character.
    Literal,
    /// Already reported.
    Failed,
}

/// Parse a collection whose `[` has already been consumed.
pub(crate) fn collection(
    rc: &mut RegCompiler,
    flagp: &mut c_int,
    crosses_lines: bool,
) -> Collection {
    if !collection_closes(rc) {
        if rc.strict != 0 {
            // Not `magic_prefix`: this one wants the backslash whenever `[`
            // is not magic, which is one 'magic' level lower.
            let prefix = if rc.magic > MAGIC_OFF { "" } else { "\\" };
            semsg!("E769: Missing ] after {prefix}[");
            rc_did_emsg.set(true);
            return Collection::Failed;
        }
        return Collection::Literal;
    }

    let negated = pat_byte(rc, 0) == b'^';
    if negated {
        pat_seek(rc, 1);
    }
    let ret = rc.code.node_nl(
        if negated { BtOp::Anybut } else { BtOp::Anyof },
        crosses_lines,
    );
    // `[\n]` widens the node itself, but only a plain `ANYOF` — `[^\n]` and
    // `\_[...]` are already what they need to be.
    let widens_on_nl = !negated && !crosses_lines;

    // A `]` or `-` in the very first position is that character, not the
    // close and not a range.
    let mut startc = -1;
    if matches!(pat_byte(rc, 0), b']' | b'-') {
        startc = pat_byte(rc, 0) as c_int;
        rc.code.byte(startc);
        pat_seek(rc, 1);
    }

    while !matches!(pat_byte(rc, 0), 0 | b']') {
        match pat_byte(rc, 0) {
            b'-' => {
                pat_seek(rc, 1);
                if let Some(failed) = range(rc, &mut startc) {
                    return failed;
                }
            }
            b'\\' if escaped_here(rc) => {
                pat_seek(rc, 1);
                if let Some(failed) = escaped(rc, flagp, &mut startc, ret, widens_on_nl) {
                    return failed;
                }
            }
            b'[' => bracketed_item(rc, &mut startc),
            _ => {
                // An ordinary character, plus any combining marks: they are
                // emitted as-is so that the set holds the whole grapheme.
                startc = pat_char(rc, 0);
                let len = pat_charlen(rc, 0);
                // A grapheme longer than its base character cannot be a
                // range endpoint.
                if utf_char2len(startc) != len {
                    startc = -1;
                }
                for _ in 0..len {
                    let byte = pat_byte(rc, 0);
                    rc.code.byte(byte as c_int);
                    pat_seek(rc, 1);
                }
            }
        }
    }

    rc.code.byte(NUL);
    // The collection was consumed byte by byte rather than through the
    // character reader, so the reader's idea of how far back one character is
    // has to be reset before `skipchr` steps over the `]`.
    rc.prev_token_len = 1;
    if pat_byte(rc, 0) != b']' {
        // `e_toomsbra`'s text, inlined: `semsg!` needs a literal.
        semsg!("E76: Too many [");
        rc_did_emsg.set(true);
        return Collection::Failed;
    }
    skipchr(rc);
    *flagp |= HASWIDTH | SIMPLE;
    Collection::Node(ret)
}

/// Does the collection at the cursor have a closing `]`?
fn collection_closes(rc: &RegCompiler) -> bool {
    // SAFETY: the cursor points into the NUL-terminated pattern, and
    // `skip_anyof` stops at its NUL.
    unsafe { *skip_anyof(rc.cursor, rc.cpo_lit) as u8 == b']' }
}

/// A `-` inside the collection: either a range from `startc`, or a literal
/// dash. Returns `Some` only on error.
fn range(rc: &mut RegCompiler, startc: &mut c_int) -> Option<Collection> {
    // A dash is literal at the end, with nothing in front of it, or before a
    // `\n` — which is a line break rather than a character.
    if matches!(pat_byte(rc, 0), b']' | 0)
        || *startc == -1
        || (pat_byte(rc, 0) == b'\\' && pat_byte(rc, 1) == b'n')
    {
        rc.code.byte(b'-' as c_int);
        *startc = b'-' as c_int;
        return None;
    }

    let mut endc = 0;
    if pat_byte(rc, 0) == b'[' {
        endc = take_cursor_bracketed(rc, b'.');
    }
    if endc == 0 {
        endc = pat_char(rc, 0);
        let len = pat_charlen(rc, 0);
        pat_seek(rc, len as isize);
    }
    if endc == b'\\' as c_int && !rc.cpo_lit {
        endc = coll_get_char(rc);
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
            rc.code.char(c);
        }
    } else {
        for c in *startc + 1..=endc {
            rc.code.byte(c);
        }
    }
    *startc = -1;
    None
}

/// Does the backslash at the cursor escape something, or is it a literal
/// backslash? `[]^-n\` always escape; the `\r`/`\t` abbreviations only when
/// 'cpoptions' does not contain `l`.
fn escaped_here(rc: &RegCompiler) -> bool {
    let next = pat_byte(rc, 1);
    REGEXP_INRANGE.to_bytes().contains(&next)
        || (!rc.cpo_lit && REGEXP_ABBR.to_bytes().contains(&next))
}

/// The character after a backslash inside the collection, with the cursor
/// already past the backslash. Returns `Some` only on error.
fn escaped(
    rc: &mut RegCompiler,
    flagp: &mut c_int,
    startc: &mut c_int,
    ret: Node,
    widens_on_nl: bool,
) -> Option<Collection> {
    match pat_byte(rc, 0) {
        b'n' => {
            // A line break is not a member of the set but a widening of the
            // node itself.
            if widens_on_nl {
                rc.code.set_opcode(ret, BtOp::Anyof, true);
                *flagp |= HASNL;
            }
            pat_seek(rc, 1);
            *startc = -1;
            None
        }
        b'd' | b'o' | b'x' | b'u' | b'U' => {
            *startc = coll_get_char(rc);
            if *startc == INT_MAX {
                semsg!("E1541: Value too large, max Unicode codepoint is U+10FFFF");
                rc_did_emsg.set(true);
                return Some(Collection::Failed);
            }
            // As elsewhere, a NUL in the pattern stands for a newline.
            if *startc == 0 {
                rc.code.byte(0xa);
            } else {
                rc.code.char(*startc);
            }
            None
        }
        c => {
            pat_seek(rc, 1);
            *startc = backslash_abbr(c as c_int);
            rc.code.byte(*startc);
            None
        }
    }
}

/// A `[` inside the collection: a `[:alpha:]` class, a `[=a=]` equivalence
/// class, a `[.a.]` collation element, or a literal `[`.
fn bracketed_item(rc: &mut RegCompiler, startc: &mut c_int) {
    let class = take_cursor_char_class(rc);
    *startc = -1;
    if let Some(class) = class {
        emit_char_class(rc, class);
        return;
    }
    let equi = take_cursor_bracketed(rc, b'=');
    if equi != 0 {
        reg_equi_class(rc, equi);
        return;
    }
    let coll = take_cursor_bracketed(rc, b'.');
    if coll != 0 {
        rc.code.char(coll);
        return;
    }
    *startc = pat_byte(rc, 0) as c_int;
    rc.code.byte(*startc);
    pat_seek(rc, 1);
}

/// [`take_char_class`] against the parse cursor.
fn take_cursor_char_class(rc: &mut RegCompiler) -> Option<CharClass> {
    // SAFETY: the cursor points into the NUL-terminated pattern, and
    // `take_char_class` only ever advances it -- it walks bytes and calls
    // nothing, so it cannot re-enter the cell it is handed.
    unsafe { take_char_class(&mut rc.cursor) }
}

/// [`take_bracketed`] against the parse cursor.
fn take_cursor_bracketed(rc: &mut RegCompiler, delim: u8) -> c_int {
    // SAFETY: as `take_cursor_char_class`.
    unsafe { take_bracketed(&mut rc.cursor, delim) }
}

/// Upstream's per-class predicates.
///
/// The ceilings the classes walk to are upstream's and are not uniform: the
/// ASCII-only ones stop at 127, the rest run the whole Latin-1 range.
fn class_ceiling(class: CharClass) -> Option<c_int> {
    use CharClass::*;
    match class {
        Alnum | Alpha | Cntrl | Digit | Graph | Punct => Some(127),
        Lower | Print | Upper | Xdigit | Ident | Keyword | Fname => Some(255),
        _ => None,
    }
}

/// Is `c` a member of `class`?
fn in_class(rc: &RegCompiler, class: CharClass, c: c_int) -> bool {
    // SAFETY: every predicate here is a pure test on a code point, reading
    // only locale or option state; the ctype table is indexable over the
    // range `class_ceiling` allows.
    let ctype =
        |mask: c_uint| unsafe { *(*__ctype_b_loc()).offset(c as isize) } as c_uint & mask != 0;
    use CharClass::*;
    match class {
        Alnum => ctype(_ISalnum),
        Alpha => ctype(_ISalpha),
        Cntrl => ctype(_IScntrl),
        Digit => ascii_isdigit(c),
        Graph => ctype(_ISgraph),
        Punct => ctype(_ISpunct),
        // U+00AA and U+00BA are the ordinal indicators: lowercase
        // letters, but not the lower half of a case pair.
        Lower => mb_islower(c) && c != 170 && c != 186,
        Print => vim_isprintc(c),
        Upper => mb_isupper(c),
        Xdigit => ascii_isxdigit(c),
        Ident => vim_is_ident_char(c),
        Keyword => vim_iswordc_buf(c, rc.buf),
        Fname => vim_isfilec(c),
        _ => false,
    }
}

/// Write out every character of a `[:name:]` class.
fn emit_char_class(rc: &mut RegCompiler, class: CharClass) {
    if let Some(hi) = class_ceiling(class) {
        for c in 1..=hi {
            if in_class(rc, class, c) {
                rc.code.char(c);
            }
        }
        return;
    }
    // The rest are short literal sets.
    match class {
        CharClass::Blank => {
            rc.code.byte(b' ' as c_int);
            rc.code.byte(b'\t' as c_int);
        }
        CharClass::Space => {
            for c in 9..=13 {
                rc.code.byte(c);
            }
            rc.code.byte(b' ' as c_int);
        }
        CharClass::Tab => rc.code.byte(b'\t' as c_int),
        CharClass::Return => rc.code.byte(b'\r' as c_int),
        CharClass::Backspace => rc.code.byte(0x08),
        CharClass::Escape => rc.code.byte(ESC),
        _ => {}
    }
}
