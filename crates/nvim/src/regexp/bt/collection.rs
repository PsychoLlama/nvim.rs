//! `[abc]`: a collection, compiled to an `ANYOF`/`ANYBUT` node whose operand
//! is every character it accepts, as a NUL-terminated string.
//!
//! The operand is a *set*, so a range expands to its members and a
//! `[:alpha:]` class to the characters it contains. That is why a range over
//! multibyte characters is capped: it would otherwise write the whole span
//! into the program.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::compile::Node;
use crate::charset::vim_iswordc_buf;
use crate::cstr;
use crate::regexp::RegCompiler;
use core::ffi::c_int;

use super::equi_class::reg_equi_class;
use super::op::BtOp;
use super::piece::coll_get_char;
use crate::ascii::{ascii_isdigit, ascii_isxdigit};
use crate::charset::{vim_is_ident_char, vim_isfilec, vim_isprintc};
use crate::mbyte::{mb_islower, mb_isupper, utf_char2len};
use crate::regexp::{
    CharClass, CompileError, Compiled, ESC, HASNL, HASWIDTH, INT_MAX, MAGIC_OFF, REGEXP_ABBR,
    REGEXP_INRANGE, SIMPLE, Spelled, backslash_abbr, pat_byte, pat_char, pat_charlen, pat_seek,
    skip_anyof, skipchr, take_bracketed, take_char_class,
};
use crate::types::NUL;

/// What a `[` at the cursor turned out to be.
pub(crate) enum Collection {
    Node(Node),
    /// No closing `]`: the `[` is an ordinary character.
    Literal,
}

/// Parse a collection whose `[` has already been consumed.
pub(crate) fn collection(
    rc: &mut RegCompiler,
    flagp: &mut c_int,
    crosses_lines: bool,
) -> Compiled<Collection> {
    if !collection_closes(rc) {
        if rc.strict != 0 {
            // Not `magic_prefix`: this one wants the backslash whenever `[`
            // is not magic, which is one 'magic' level lower.
            let spelled = Spelled::bare_if(rc.magic > MAGIC_OFF);
            return Err(CompileError::MissingBracket(spelled));
        }
        return Ok(Collection::Literal);
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
        startc = c_int::from(pat_byte(rc, 0));
        rc.code.byte(startc);
        pat_seek(rc, 1);
    }

    while !matches!(pat_byte(rc, 0), 0 | b']') {
        match pat_byte(rc, 0) {
            b'-' => {
                pat_seek(rc, 1);
                range(rc, &mut startc)?;
            }
            b'\\' if escaped_here(rc) => {
                pat_seek(rc, 1);
                escaped(rc, flagp, &mut startc, ret, widens_on_nl)?;
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
                    rc.code.byte(c_int::from(byte));
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
        return Err(CompileError::TooManyBrackets);
    }
    skipchr(rc);
    *flagp |= HASWIDTH | SIMPLE;
    Ok(Collection::Node(ret))
}

/// Does the collection at the cursor have a closing `]`?
fn collection_closes(rc: &RegCompiler) -> bool {
    let end = skip_anyof(rc.text, rc.cursor, rc.cpo_lit);
    cstr::byte_at(rc.text, end) == b']'
}

/// A `-` inside the collection: either a range from `startc`, or a literal
/// dash.
fn range(rc: &mut RegCompiler, startc: &mut c_int) -> Compiled {
    // A dash is literal at the end, with nothing in front of it, or before a
    // `\n` — which is a line break rather than a character.
    if matches!(pat_byte(rc, 0), b']' | 0)
        || *startc == -1
        || (pat_byte(rc, 0) == b'\\' && pat_byte(rc, 1) == b'n')
    {
        rc.code.byte(c_int::from(b'-'));
        *startc = c_int::from(b'-');
        return Ok(());
    }

    let mut endc = 0;
    if pat_byte(rc, 0) == b'[' {
        endc = take_cursor_bracketed(rc, b'.');
    }
    if endc == 0 {
        endc = pat_char(rc, 0);
        let len = pat_charlen(rc, 0);
        pat_seek(
            rc,
            isize::try_from(len).expect("a character is a few bytes"),
        );
    }
    if endc == c_int::from(b'\\') && !rc.cpo_lit {
        endc = coll_get_char(rc);
    }
    if *startc > endc {
        return Err(CompileError::ReverseRange);
    }
    let multibyte = utf_char2len(*startc) > 1 || utf_char2len(endc) > 1;
    if multibyte {
        // Every member is written into the program, so a wide range would
        // blow it up.
        if endc > *startc + 256 {
            return Err(CompileError::RangeTooLarge);
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
    Ok(())
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
/// already past the backslash.
fn escaped(
    rc: &mut RegCompiler,
    flagp: &mut c_int,
    startc: &mut c_int,
    ret: Node,
    widens_on_nl: bool,
) -> Compiled {
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
            Ok(())
        }
        b'd' | b'o' | b'x' | b'u' | b'U' => {
            *startc = coll_get_char(rc);
            if *startc == INT_MAX {
                return Err(CompileError::CodepointTooLarge);
            }
            // As elsewhere, a NUL in the pattern stands for a newline.
            if *startc == 0 {
                rc.code.byte(0xa);
            } else {
                rc.code.char(*startc);
            }
            Ok(())
        }
        c => {
            pat_seek(rc, 1);
            *startc = backslash_abbr(c_int::from(c));
            rc.code.byte(*startc);
            Ok(())
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
    *startc = c_int::from(pat_byte(rc, 0));
    rc.code.byte(*startc);
    pat_seek(rc, 1);
}

/// [`take_char_class`] against the parse cursor.
fn take_cursor_char_class(rc: &mut RegCompiler) -> Option<CharClass> {
    take_char_class(rc.text, &mut rc.cursor)
}

/// [`take_bracketed`] against the parse cursor.
fn take_cursor_bracketed(rc: &mut RegCompiler, delim: u8) -> c_int {
    take_bracketed(rc.text, &mut rc.cursor, delim)
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
///
/// Upstream asks the C library's `isalnum` and kin for the classes that
/// only ever cover ASCII ([`class_ceiling`] stops them at 127); for ASCII
/// those answer the same in every locale, which is what `u8`'s own tests
/// answer.
fn in_class(rc: &RegCompiler, class: CharClass, c: c_int) -> bool {
    let ascii = |test: fn(&u8) -> bool| u8::try_from(c).is_ok_and(|b| b.is_ascii() && test(&b));
    use CharClass::*;
    match class {
        Alnum => ascii(u8::is_ascii_alphanumeric),
        Alpha => ascii(u8::is_ascii_alphabetic),
        Cntrl => ascii(u8::is_ascii_control),
        Digit => ascii_isdigit(c),
        Graph => ascii(u8::is_ascii_graphic),
        Punct => ascii(u8::is_ascii_punctuation),
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
            rc.code.byte(c_int::from(b' '));
            rc.code.byte(c_int::from(b'\t'));
        }
        CharClass::Space => {
            for c in 9..=13 {
                rc.code.byte(c);
            }
            rc.code.byte(c_int::from(b' '));
        }
        CharClass::Tab => rc.code.byte(c_int::from(b'\t')),
        CharClass::Return => rc.code.byte(c_int::from(b'\r')),
        CharClass::Backspace => rc.code.byte(0x08),
        CharClass::Escape => rc.code.byte(ESC),
        _ => {}
    }
}
