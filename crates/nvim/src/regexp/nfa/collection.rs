//! `[abc]`: a collection, emitted as its members joined by `NFA_CONCAT`
//! between an `NFA_START_COLL` and an `NFA_END_COLL`.
//!
//! Unlike the backtracking engine, which writes the whole accepted set into
//! one node's operand, this engine keeps a state per member and an
//! `NFA_RANGE` pair for a span — so a wide range costs three items rather
//! than one per character.

#![forbid(unsafe_code)]

use core::ffi::{c_char, c_int};

use super::cursor;
use super::equi_class::nfa_emit_equi_class;
use super::postfix;
use crate::main::rc_did_emsg;
use crate::mbyte::utf_char2len;
use crate::regexp::{
    CLASS_NONE, FAIL, INT_MAX, MAGIC_OFF, NFA_ADD_NL, NFA_CLASS_ALNUM, NFA_CLASS_ALPHA,
    NFA_CLASS_BACKSPACE, NFA_CLASS_BLANK, NFA_CLASS_CNTRL, NFA_CLASS_DIGIT, NFA_CLASS_ESCAPE,
    NFA_CLASS_FNAME, NFA_CLASS_GRAPH, NFA_CLASS_IDENT, NFA_CLASS_KEYWORD, NFA_CLASS_LOWER,
    NFA_CLASS_PRINT, NFA_CLASS_PUNCT, NFA_CLASS_RETURN, NFA_CLASS_SPACE, NFA_CLASS_TAB,
    NFA_CLASS_UPPER, NFA_CLASS_XDIGIT, NFA_COMPOSING, NFA_CONCAT, NFA_END_COLL, NFA_END_NEG_COLL,
    NFA_FIRST_NL, NFA_LAST_NL, NFA_NEWL, NFA_OR, NFA_RANGE, NFA_START_COLL, NFA_START_NEG_COLL, NL,
    REGEXP_ABBR, REGEXP_INRANGE, backslash_abbr, coll_get_char, pat_byte, pat_char, pat_charlen,
    reg_cpo_lit, reg_magic, reg_strict, reg_string, wants_nfa,
};
use crate::semsg;

use super::cursor::{
    advance_grapheme, byte_at, char_at, grapheme_len, recognize_char_class, take_cursor_bracketed,
    take_cursor_char_class,
};

/// The `NFA_CLASS_*` opcodes indexed by the `CLASS_*` value
/// [`take_cursor_char_class`] returns. The two enumerations were written in
/// the same order; this table says so rather than relying on it.
const CLASS_OPCODES: [c_int; 19] = [
    NFA_CLASS_ALNUM,
    NFA_CLASS_ALPHA,
    NFA_CLASS_BLANK,
    NFA_CLASS_CNTRL,
    NFA_CLASS_DIGIT,
    NFA_CLASS_GRAPH,
    NFA_CLASS_LOWER,
    NFA_CLASS_PRINT,
    NFA_CLASS_PUNCT,
    NFA_CLASS_SPACE,
    NFA_CLASS_UPPER,
    NFA_CLASS_XDIGIT,
    NFA_CLASS_TAB,
    NFA_CLASS_RETURN,
    NFA_CLASS_BACKSPACE,
    NFA_CLASS_ESCAPE,
    NFA_CLASS_IDENT,
    NFA_CLASS_KEYWORD,
    NFA_CLASS_FNAME,
];

/// `[[:lower:]]` and `[[:upper:]]` follow the locale, which the backtracking
/// engine expands at compile time and so gets wrong; asking for this engine
/// keeps the test at match time.
const CLASS_LOWER: c_int = 6;
const CLASS_UPPER: c_int = 10;

/// What a `[` at the cursor turned out to be.
pub(crate) enum Collection {
    /// Emitted.
    Done,
    /// No closing `]`: the `[` is an ordinary character.
    NotACollection,
    /// Already reported.
    Failed,
}

/// Parse a collection whose `[` has already been consumed. `extra` is
/// `NFA_ADD_NL` for the `\_[` form, which also accepts a line break.
///
/// `atom_start` is where the atom began, which bounds the one step back the
/// parser takes at the end.
pub(crate) fn collection(mut extra: c_int, atom_start: *mut c_char) -> Collection {
    let end = cursor::collection_end();
    if byte_at(end) != b']' {
        if reg_strict.get() != 0 {
            // Not `magic_prefix`: this wants the backslash whenever `[` is
            // not magic, which is one 'magic' level lower.
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

    // `[0-9]` and its kin are really `\d` and can be one state instead of
    // ten.
    let recognized = recognize_char_class(end, extra);
    if recognized != FAIL {
        if (NFA_FIRST_NL..=NFA_LAST_NL).contains(&recognized) {
            // The class in its line-break-accepting form: emit the plain
            // class and the line break as alternatives.
            postfix::emit(recognized - NFA_ADD_NL);
            postfix::emit(NFA_NEWL);
            postfix::emit(NFA_OR);
        } else {
            postfix::emit(recognized);
        }
        skip_past(end);
        return Collection::Done;
    }

    let negated = pat_byte(0) == b'^';
    if negated {
        advance_grapheme();
        postfix::emit(NFA_START_NEG_COLL);
    } else {
        postfix::emit(NFA_START_COLL);
    }

    // A `-` in the very first position is that character, not a range.
    let mut startc = -1;
    if pat_byte(0) == b'-' {
        startc = b'-' as c_int;
        postfix::emit_concat(startc);
        advance_grapheme();
    }

    let mut emit_range = false;
    while cursor::before(end) {
        let oldstartc = startc;
        let mut range_endpoint = false;
        startc = -1;
        let mut got_coll_char = false;

        if pat_byte(0) == b'[' {
            match bracketed_item() {
                Bracketed::Emitted => continue,
                Bracketed::Collated(c) => startc = c,
                Bracketed::Literal => {}
            }
        }

        if pat_byte(0) == b'-' && oldstartc != -1 {
            // The `-` of a range: what follows is its upper bound.
            emit_range = true;
            startc = oldstartc;
            advance_grapheme();
            continue;
        }

        if escapes_here(end) {
            advance_grapheme();
            match pat_byte(0) {
                b'n' => {
                    // Inside a range, and in a string match where there are
                    // no lines, `\n` is the byte; otherwise it widens the
                    // collection to accept a line break.
                    startc = if reg_string.get() != 0 || emit_range || pat_byte(1) == b'-' {
                        NL
                    } else {
                        NFA_NEWL
                    };
                }
                b'd' | b'o' | b'x' | b'u' | b'U' => {
                    startc = coll_get_char();
                    if startc == INT_MAX {
                        semsg!("E1541: Value too large, max Unicode codepoint is U+10FFFF");
                        rc_did_emsg.set(true);
                        return Collection::Failed;
                    }
                    got_coll_char = true;
                    // `coll_get_char` left the cursor past the escape; the
                    // loop tail advances again, so step back one character.
                    cursor::step_back(atom_start);
                }
                c => startc = backslash_abbr(c as c_int),
            }
        }
        if startc == -1 {
            startc = pat_char(0);
        }

        if emit_range {
            let endc = startc;
            range_endpoint = true;
            startc = oldstartc;
            if startc > endc {
                semsg!("E944: Reverse range in character class");
                rc_did_emsg.set(true);
                return Collection::Failed;
            }
            emit_span(startc, endc);
            emit_range = false;
            startc = -1;
        } else if startc == NFA_NEWL {
            // Not a member but a widening of the whole collection — except
            // for `[^...]`, where accepting a line break is the caller's
            // business.
            if !negated {
                extra = NFA_ADD_NL;
            }
        } else if got_coll_char && startc == 0 {
            // A `\x00` in the pattern stands for a newline.
            postfix::emit_concat(0xa);
        } else {
            postfix::emit(startc);
            // A character carrying combining marks is joined by the
            // `NFA_COMPOSING` group below instead.
            if pat_charlen(0) == grapheme_len(cursor::here()) {
                postfix::emit(NFA_CONCAT);
            }
        }

        // A range endpoint cannot carry combining marks; anything else can,
        // and the whole grapheme becomes one member.
        if !range_endpoint {
            emit_combining_marks();
        }
        advance_grapheme();
    }

    // Back up over the last character read so that a trailing `-` — which
    // the loop above consumed as an ordinary member's neighbour — can be
    // seen.
    cursor::step_back(atom_start);
    if pat_byte(0) == b'-' {
        postfix::emit_concat(b'-' as c_int);
    }
    skip_past(end);

    postfix::emit(if negated {
        NFA_END_NEG_COLL
    } else {
        NFA_END_COLL
    });
    if extra == NFA_ADD_NL {
        postfix::emit(if reg_string.get() != 0 { NL } else { NFA_NEWL });
        postfix::emit(NFA_OR);
    }
    Collection::Done
}

/// Put the cursor past the collection's closing `]`.
fn skip_past(end: *mut c_char) {
    cursor::seek_to(end);
    advance_grapheme();
}

/// What a `[` inside the collection turned out to be.
enum Bracketed {
    /// A `[:alpha:]` class or a `[=a=]` equivalence class: already emitted.
    Emitted,
    /// A `[.a.]` collation element, which stands for one character.
    Collated(c_int),
    /// A literal `[`.
    Literal,
}

fn bracketed_item() -> Bracketed {
    let class = take_cursor_char_class();
    if class != CLASS_NONE as c_int {
        if matches!(class, CLASS_LOWER | CLASS_UPPER) {
            wants_nfa.set(true);
        }
        if let Some(&op) = CLASS_OPCODES.get(class as usize) {
            postfix::emit(op);
        }
        postfix::emit(NFA_CONCAT);
        return Bracketed::Emitted;
    }
    let equi = take_cursor_bracketed(b'=');
    if equi != 0 {
        nfa_emit_equi_class(equi);
        return Bracketed::Emitted;
    }
    match take_cursor_bracketed(b'.') {
        0 => Bracketed::Literal,
        coll => Bracketed::Collated(coll),
    }
}

/// Does the backslash at the cursor escape something? `[]^-n\` always do;
/// the `\r`/`\t` abbreviations only when 'cpoptions' does not contain `l`.
///
/// `end` bounds it: a backslash as the collection's last byte is literal.
fn escapes_here(end: *mut c_char) -> bool {
    if pat_byte(0) != b'\\' || cursor::here().wrapping_add(1) > end {
        return false;
    }
    let next = pat_byte(1);
    REGEXP_INRANGE.to_bytes().contains(&next)
        || (reg_cpo_lit.get() == 0 && REGEXP_ABBR.to_bytes().contains(&next))
}

/// Emit `startc`..`endc` as an `NFA_RANGE`, or as its individual members
/// when there are only a couple of them.
fn emit_span(startc: c_int, endc: c_int) {
    if endc > startc + 2 {
        if startc == 0 {
            // `\x00` was emitted as `\x0a` above, so it stays a member of
            // its own and the range starts at 1.
            postfix::emit(1);
        } else {
            // Reclaim the `NFA_CONCAT` the start character was emitted
            // with: `NFA_RANGE` needs both endpoints on the stack.
            postfix::drop_last();
        }
        postfix::emit(endc);
        postfix::emit(NFA_RANGE);
        postfix::emit(NFA_CONCAT);
        return;
    }
    // Upstream splits this in two — one loop for a range whose endpoints are
    // multibyte and one for the rest — with identical bodies.
    for c in startc + 1..=endc {
        postfix::emit_concat(c);
    }
}

/// Emit the combining marks on the character at the cursor, wrapped in an
/// `NFA_COMPOSING` group so the whole grapheme is one member.
fn emit_combining_marks() {
    let plen = grapheme_len(cursor::here());
    let mut i = pat_charlen(0);
    if i == plen {
        return;
    }
    let here = cursor::here();
    loop {
        let c = char_at(here, i);
        // A NUL in the pattern cannot be emitted as itself.
        postfix::emit_concat(if c == 0 { 1 } else { c });
        i += utf_char2len(c);
        if i >= plen {
            break;
        }
    }
    postfix::emit(NFA_COMPOSING);
    postfix::emit(NFA_CONCAT);
}
