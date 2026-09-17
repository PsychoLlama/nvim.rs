//! What kind of statement a line is.
//!
//! The predicates `get_c_indent`'s backwards scan asks of each line it walks
//! past: is it a `case`/`default` label, a scope declaration (`private:`, and
//! whatever else 'cinscopedecls' names), a `break`, one of the 'cinwords'
//! keywords, an `if`/`else`/`do`, the `while` belonging to a `do`.
//! [`terminator`] is the one the whole state machine turns on -- it answers
//! the *character* a statement ended with (`;`, `,`, `{`, or 0 for "did not
//! end"), which is what tells a continuation line from a finished one.
//!
//! Every one of them takes the line as `&[u8]` and an offset into it, and the
//! ones that step take an offset back.  The C names, for anyone reading
//! upstream beside this:
//!
//! | C | here |
//! | --- | --- |
//! | `cin_is_cinword` | [`starts_with_cinword`] |
//! | `cin_has_js_key` | [`has_js_key`] |
//! | `cin_iscase` | [`is_case_label`] |
//! | `cin_isdefault` | [`is_default_label`] |
//! | `cin_isscopedecl` | [`is_scope_decl`] |
//! | `cin_starts_with` | [`starts_with_word`] |
//! | `cin_skip_close_brace` | [`past_close_brace`] |
//! | `cin_isif` / `cin_iselse` / `cin_isdo` / `cin_isbreak` | [`is_if`] / [`is_else`] / [`is_do`] / [`is_break`] |
//! | `cin_iswhileofdo` | [`starts_while`] + [`while_closes_do`] |
//! | `cin_iswhileofdo_end` | [`ends_a_do_while`] |
//! | `cin_is_if_for_while_before_offset` | [`control_clause_before`] |
//! | `cin_ends_in_backslash` | [`ends_in_backslash`] |
//! | `cin_ends_in` | [`ends_in`] |
//! | `cin_isterminated` | [`terminator`] |

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use super::*;
use crate::optionstr::LocalOptStr;

/// One part of a comma-separated option, appended to `part`, with the rest of
/// the option answered back.
///
/// `copy_option_part` over slices, for the two options this file reads:
/// a backslash before a comma makes it a literal, a leading `.` is taken
/// whatever it is, and the separator and the blanks behind it are dropped.
/// One `part` buffer serves a whole walk.
fn next_option_part<'a>(option: &'a [u8], part: &mut Vec<u8>) -> &'a [u8] {
    part.clear();
    let mut i = 0;
    // A leading '.' is copied without being tested against the separators.
    if option.first() == Some(&b'.') {
        part.push(b'.');
        i = 1;
    }
    while i < option.len() && option[i] != b',' {
        // A backslash escapes the separator, and is dropped.
        if option[i] == b'\\' && option.get(i + 1) == Some(&b',') {
            i += 1;
        }
        part.push(option[i]);
        i += 1;
    }
    if i < option.len() {
        i += 1; // the separator itself
    }
    while option.get(i) == Some(&b' ') {
        i += 1;
    }
    &option[i..]
}

/// Whether `line` starts with a word from 'cinwords' -- `if`, `else`,
/// `while`, `do`, `for`, `switch` by default.
///
/// The word must be delimited on at least one side: an option item that is a
/// prefix of a longer identifier does not count unless the character before
/// it is already a non-word one.
pub fn starts_with_cinword(line: &[u8]) -> bool {
    let start = line
        .iter()
        .position(|&b| !ascii_iswhite(c_int::from(b)))
        .unwrap_or(line.len());

    // SAFETY: 'cinwords' is a NUL-terminated option string of this buffer.
    let words = unsafe { cstr::bytes_at(Buf::current().b_p_cinw.value_ptr()) };
    let mut rest = words;
    let mut word = Vec::with_capacity(words.len());
    while !rest.is_empty() {
        rest = next_option_part(rest, &mut word);
        if !line[start..].starts_with(&word) {
            continue;
        }
        // Upstream reads `line[len - 1]`, which for an *empty* 'cinwords'
        // item on a line with no leading white space is the byte before
        // the buffer.  Refused: answering NUL gives the same verdict the
        // white-space case does, and no gate reaches it.
        let after = byte_at(line, start + word.len());
        let before = if start + word.len() == 0 {
            0
        } else {
            byte_at(line, start + word.len() - 1)
        };
        if !vim_iswordc(c_int::from(after)) || !vim_iswordc(c_int::from(before)) {
            return true;
        }
    }
    false
}

/// Whether `line[at..]` starts with `key:` -- the Javascript object-literal
/// shape 'cinoptions' `j1` indents against.
///
/// The key may be quoted (`'key':`, `"key":`), and `::` is C++ scope
/// resolution rather than a label.
pub(crate) fn has_js_key(line: &[u8], at: usize) -> bool {
    let mut i = at + skip::white(&line[at.min(line.len())..]);

    let quote = match byte_at(line, i) {
        q @ (b'\'' | b'"') => q,
        _ => 0,
    };
    i += usize::from(quote != 0);
    if !vim_is_ident_char(c_int::from(byte_at(line, i))) {
        return false; // need at least one ID character
    }
    while vim_is_ident_char(c_int::from(byte_at(line, i))) {
        i += 1;
    }
    if byte_at(line, i) != 0 && byte_at(line, i) == quote {
        i += 1;
    }
    let i = code_at(line, i);
    byte_at(line, i) == b':' && byte_at(line, i + 1) != b':'
}

/// Whether `line[at..]` is a `case` or `default` switch label.
///
/// `strict` is the C reading; without it a `"` after the `case` still counts,
/// which is what makes `case "x":` a label in Javascript.
pub(crate) fn is_case_label(line: &[u8], at: usize, strict: bool) -> bool {
    let start = code_at(line, at);
    if !starts_with_word(line, start, b"case") {
        return is_default_label(line, start);
    }

    let mut i = start + 4;
    while byte_at(line, i) != 0 {
        i = code_at(line, i);
        if byte_at(line, i) == 0 {
            break;
        }
        if byte_at(line, i) == b':' {
            if byte_at(line, i + 1) == b':' {
                i += 1; // skip over "::" for C++
            } else {
                return true;
            }
        }
        if byte_at(line, i) == b'\'' && byte_at(line, i + 1) != 0 && byte_at(line, i + 2) == b'\'' {
            i += 2; // skip over ':'
        } else if byte_at(line, i) == b'/'
            && (byte_at(line, i + 1) == b'*' || byte_at(line, i + 1) == b'/')
        {
            return false; // stop at comment
        } else if byte_at(line, i) == b'"' {
            // A string ends the search under the C rules; under the
            // relaxed ones it *is* the label (`case "x":` in JS).
            return !strict;
        }
        i += 1;
    }
    false
}

/// Whether `line[at..]` is a `default:` switch label.
pub(crate) fn is_default_label(line: &[u8], at: usize) -> bool {
    if !line[at.min(line.len())..].starts_with(b"default") {
        return false;
    }
    let after = code_at(line, at + 7);
    byte_at(line, after) == b':' && byte_at(line, after + 1) != b':'
}

/// Whether `line[at..]` is a label named by one of `option`'s comma-separated
/// words: the word, then a `:` that is not `::`.
fn is_option_label(line: &[u8], at: usize, option: &[u8]) -> bool {
    let start = code_at(line, at);
    let mut rest = option;
    let mut word = Vec::with_capacity(option.len());
    while !rest.is_empty() {
        rest = next_option_part(rest, &mut word);
        if !line[start.min(line.len())..].starts_with(&word) {
            continue;
        }
        let after = code_at(line, start + word.len());
        if byte_at(line, after) == b':' && byte_at(line, after + 1) != b':' {
            return true;
        }
    }
    false
}

/// Whether `line[at..]` is a scope declaration label named by
/// 'cinscopedecls' -- `public`, `protected`, `private` by default.
pub(crate) fn is_scope_decl(line: &[u8], at: usize) -> bool {
    // SAFETY: 'cinscopedecls' is a NUL-terminated option string of this
    // buffer, and `is_option_label` only reads the bytes it is handed.
    let decls = unsafe { cstr::bytes_at(Buf::current().b_p_cinsd.value_ptr()) };
    is_option_label(line, at, decls)
}

/// Whether `line[at..]` starts with `word` followed by a non-identifier
/// character.
pub(crate) fn starts_with_word(line: &[u8], at: usize, word: &[u8]) -> bool {
    let after = byte_at(line, at + word.len());
    line[at.min(line.len())..].starts_with(word) && !vim_is_ident_char(c_int::from(after))
}

/// `at` with a leading `}` -- and any comment behind it -- stepped over: the
/// `} else` / `} while (cond);` shape three of the predicates here accept.
pub(crate) fn past_close_brace(line: &[u8], at: usize) -> usize {
    if byte_at(line, at) == b'}' {
        code_at(line, at + 1)
    } else {
        at
    }
}

/// Whether `line[at..]` is an `if`.
pub(crate) fn is_if(line: &[u8], at: usize) -> bool {
    starts_with_word(line, at, b"if")
}

/// Whether `line[at..]` is an `else`, accepting `} else`.
pub(crate) fn is_else(line: &[u8], at: usize) -> bool {
    starts_with_word(line, past_close_brace(line, at), b"else")
}

/// Whether `line[at..]` is a `do`.
pub(crate) fn is_do(line: &[u8], at: usize) -> bool {
    starts_with_word(line, at, b"do")
}

/// Whether `line[at..]` is a `break`.
pub(crate) fn is_break(line: &[u8], at: usize) -> bool {
    starts_with_word(line, at, b"break")
}

/// Whether `line[at..]` starts a `while`, accepting a leading `}`.
///
/// Half of upstream's `cin_iswhileofdo`: the text half.  The other half --
/// whether that `while` closes a `do` -- is [`while_closes_do`], and it
/// re-enters the editor, which is why the two are separate.  A caller writes
/// them in that order and lets the borrow end in between.
pub(crate) fn starts_while(line: &[u8], at: usize) -> bool {
    starts_with_word(line, past_close_brace(line, code_at(line, at)), b"while")
}

/// Whether the `while` on line `lnum` closes a `do`.
///
/// Only `while (condition);` counts -- nothing but white space between the
/// `)` and the `;` -- because that is the shape that ends a statement rather
/// than opening one.  The condition may span lines, which is why the answer
/// needs the cursor and `findmatchlimit` rather than the text alone.
pub(crate) fn while_closes_do(lnum: LineNr) -> bool {
    let cursor_save = Win::current().w_cursor;
    Win::current().w_cursor.lnum = lnum;
    // Step over any '}' until the 'w' of the "while".
    let w = {
        let mut lines = Lines::current();
        let text = lines.line(lnum);
        text.iter().position(|&b| b == b'w').unwrap_or(text.len())
    };
    Win::current().w_cursor.col = w as ColNr;

    let maxparen = int64_t::from(Buf::current().b_ind_maxparen);
    // SAFETY: the cursor is on a line of the current buffer, which is where
    // the match search starts.
    let matched = findmatchlimit(None, 0, 0, maxparen);
    let retval = matched.is_some_and(|pos| {
        // The match is a position in this buffer, so the cache answers with
        // its line; a column past its end answers the terminator.
        let mut lines = Lines::current();
        let text = lines.line(pos.lnum);
        let after = usize::try_from(pos.col).unwrap_or(0) + 1;
        byte_at(text, code_at(text, after)) == b';'
    });
    Win::current().w_cursor = cursor_save;
    retval
}

/// Whether an `if`, `for` or `while` sits just before `*offset` in `line`,
/// and if so where -- 'cinoptions' `U`'s "is this paren a control clause's"
/// test.
pub(crate) fn control_clause_before(line: &[u8], offset: &mut c_int) -> bool {
    let mut at = *offset;
    if at < 2 {
        return false;
    }
    at -= 1;
    while at > 2 && ascii_iswhite(c_int::from(byte_at(line, at as usize))) {
        at -= 1;
    }

    // Each keyword is tested at the offset its *last* character would sit at,
    // walking further left as the words get longer.
    let starts_at = |off: c_int, word: &[u8]| {
        line.get(off as usize..)
            .is_some_and(|tail| tail.starts_with(word))
    };
    at -= 1;
    if !starts_at(at, b"if") {
        if at < 1 {
            return false;
        }
        at -= 1;
        if !starts_at(at, b"for") {
            if at < 2 {
                return false;
            }
            at -= 2;
            if !starts_at(at, b"while") {
                return false;
            }
        }
    }

    // It is only the keyword if nothing identifier-ish precedes it.
    let before = byte_at(line, (at - 1) as usize);
    if at != 0 && vim_is_ident_char(c_int::from(before)) {
        return false;
    }
    *offset = at;
    true
}

/// Whether the cursor's line is the end of a `do ... while (...);`, and if so
/// leave the cursor on the line holding the `while`.
///
/// ```text
/// do
///    nothing;
/// while (foo
///          && bar);  <-- here
/// ```
pub(crate) fn ends_a_do_while(terminated: u8) -> bool {
    if terminated != b';' {
        return false; // there must be a ';' at the end
    }
    let mut at = 0usize;
    loop {
        // The cursor's line, re-read every round: the paren search below may
        // have unlocked it, and it left the cursor where it found it.
        let closed = {
            let mut lines = Lines::current();
            let line = lines.line(Win::current().w_cursor.lnum);
            if at >= line.len() {
                return false;
            }
            at = code_at(line, at);
            let after = at + 1 + skip::white(&line[(at + 1).min(line.len())..]);
            byte_at(line, at) == b')'
                && byte_at(line, after) == b';'
                && only_comment_left(line, after + 1)
        };
        if closed {
            // Found ");" at end of the line; now check there is a "while"
            // before the matching '('.
            Win::current().w_cursor.col = at as ColNr;
            if let Some(trypos) = find_match_paren(Buf::current().b_ind_maxparen) {
                // `trypos` is a position in this buffer, so the cache answers
                // with the line the `(` sits on.
                let opens_while = {
                    let mut lines = Lines::current();
                    let opener = lines.line(trypos.lnum);
                    let start = past_close_brace(opener, code_at(opener, 0));
                    starts_with_word(opener, start, b"while")
                };
                if opens_while {
                    Win::current().w_cursor.lnum = trypos.lnum;
                    return true;
                }
            }
        }
        at += 1;
    }
}

/// Whether `line` ends in a backslash: it is continued on the next one.
///
/// Unlike [`ends_in`] this is the *last byte* of the line, comments and all
/// -- a backslash only continues a line when nothing follows it.
pub(crate) fn ends_in_backslash(line: &[u8]) -> bool {
    line.last() == Some(&b'\\')
}

/// Whether `line[at..]` ends with `find`, allowing white space and comments
/// after it.  Strings and comments in between are skipped.
pub(crate) fn ends_in(line: &[u8], at: usize, find: &[u8]) -> bool {
    let mut i = at;
    while byte_at(line, i) != 0 {
        i = code_at(line, i);
        if line[i.min(line.len())..].starts_with(find) {
            let after = i + find.len();
            let after = after + skip::white(&line[after.min(line.len())..]);
            if only_comment_left(line, after) {
                return true;
            }
        }
        if byte_at(line, i) != 0 {
            i += 1;
        }
    }
    false
}

/// The character a statement on `line[at..]` ended with -- `;`, `}`, `,` or
/// `{` -- or 0 when it did not end.
///
/// This is the fact `get_c_indent`'s whole backwards scan turns on: a
/// terminated line above is something to line up with, an unterminated one is
/// a statement still being written.  A `,` only counts with `incl_comma`, and
/// an opening `{` only with `incl_open`; a `{` or `}` at the *start* is the
/// fallback answer when nothing else terminates.
///
/// `} else` is deliberately not terminated -- the `else` continues the
/// statement -- which is what `is_else` suppresses until its block closes.
pub(crate) fn terminator(line: &[u8], at: usize, incl_open: bool, incl_comma: bool) -> u8 {
    let mut i = code_at(line, at);
    let mut n_open = 0u32;

    // The `}` test is kept in front of `is_else`, as upstream has it, so it
    // is asked no more often than it was.
    let first = byte_at(line, i);
    let found_start = if first == b'{' || (first == b'}' && !is_else(line, i)) {
        first
    } else {
        0
    };
    let is_else_line = found_start == 0 && is_else(line, i);

    while byte_at(line, i) != 0 {
        // Skip over comments, "" strings and 'c'haracters.  The string skip
        // is *not* guarded against landing on the end of the line, so a
        // trailing comment sends it back onto the line's last byte and that
        // byte is judged a second time -- which is how `foo // }` answers
        // `}` (O-B29-2).  Reproduced.
        i = string_end_at(line, code_at(line, i));
        if byte_at(line, i) == b'}' && n_open > 0 {
            n_open -= 1;
        }
        if (!is_else_line || n_open == 0)
            && (byte_at(line, i) == b';'
                || byte_at(line, i) == b'}'
                || (incl_comma && byte_at(line, i) == b','))
            && only_comment_left(line, i + 1)
        {
            return byte_at(line, i);
        } else if byte_at(line, i) == b'{' {
            if incl_open && only_comment_left(line, i + 1) {
                return byte_at(line, i);
            }
            n_open += 1;
        }
        if byte_at(line, i) != 0 {
            i += 1;
        }
    }
    found_start
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charset::init_chartab_for_tests;

    /// The predicates below ask `vim_is_ident_char`, which reads the global
    /// character table; nothing else in the harness fills it in.
    fn chartab() {
        init_chartab_for_tests();
    }

    #[test]
    fn a_word_needs_a_non_identifier_behind_it() {
        chartab();
        assert!(starts_with_word(b"if (x)", 0, b"if"));
        assert!(starts_with_word(b"if", 0, b"if")); // end of line ends it
        assert!(starts_with_word(b"  if(x)", 2, b"if"));
        assert!(!starts_with_word(b"ifx", 0, b"if"));
        assert!(!starts_with_word(b"if_", 0, b"if"));
        assert!(!starts_with_word(b"if9", 0, b"if"));
        assert!(!starts_with_word(b"elif", 1, b"if")); // matches, but at 1
    }

    #[test]
    fn the_keyword_predicates_are_that_word_test() {
        chartab();
        assert!(is_if(b"if (x)", 0));
        assert!(is_do(b"do {", 0));
        assert!(is_break(b"break;", 0));
        assert!(is_else(b"else {", 0));
        // `} else` counts, with the comment between the two stepped over.
        assert!(is_else(b"} else {", 0));
        assert!(is_else(b"} /* c */ else", 0));
        assert!(!is_else(b"} elsewhere", 0));
    }

    #[test]
    fn a_close_brace_and_its_comment_are_stepped_over() {
        assert_eq!(past_close_brace(b"} else", 0), 2);
        assert_eq!(past_close_brace(b"else", 0), 0);
        assert_eq!(past_close_brace(b"}", 0), 1);
    }

    #[test]
    fn a_case_label_ends_at_its_colon() {
        chartab();
        assert!(is_case_label(b"case 1:", 0, true));
        assert!(is_case_label(b"case 1: x = 2;", 0, true));
        assert!(is_case_label(b"  /* c */ case 1:", 0, true));
        assert!(is_case_label(b"case A::B:", 0, true)); // `::` is not the end
        assert!(is_case_label(b"default:", 0, true));
        assert!(!is_case_label(b"case 1", 0, true));
        assert!(!is_case_label(b"cases:", 0, true)); // not the word `case`
        assert!(!is_case_label(b"default::x", 0, true));
        // A `/* ... ` before the colon stops the search.
        assert!(!is_case_label(b"case 1 /* x", 0, true));
    }

    #[test]
    fn a_string_ends_a_case_label_only_under_the_c_rules() {
        chartab();
        // `case "x":` is a label in Javascript and not in C.
        assert!(!is_case_label(b"case \"x\":", 0, true));
        assert!(is_case_label(b"case \"x\":", 0, false));
    }

    #[test]
    fn an_option_label_is_one_of_its_words_then_a_colon() {
        let decls = &b"public,protected,private"[..];
        assert!(is_option_label(b"public:", 0, decls));
        assert!(is_option_label(b"  private: x", 0, decls));
        assert!(is_option_label(b"protected : x", 0, decls)); // the comment skip
        assert!(!is_option_label(b"public::x", 0, decls)); // C++ scope
        assert!(!is_option_label(b"publicx:", 0, decls));
        assert!(!is_option_label(b"public", 0, decls));
        // An unset option names nothing.
        assert!(!is_option_label(b"public:", 0, b""));
    }

    #[test]
    fn an_option_splits_on_commas_and_unescapes_them() {
        let mut part = Vec::new();
        assert_eq!(next_option_part(b"a,b", &mut part), b"b");
        assert_eq!(part, b"a");
        // A blank after the separator belongs to the separator.
        assert_eq!(next_option_part(b"a, b", &mut part), b"b");
        assert_eq!(part, b"a");
        // A backslash makes the comma a literal, and is dropped.
        assert_eq!(next_option_part(b"a\\,b,c", &mut part), b"c");
        assert_eq!(part, b"a,b");
        // The last part, and an empty one.
        assert_eq!(next_option_part(b"a", &mut part), b"");
        assert_eq!(part, b"a");
        assert_eq!(next_option_part(b",b", &mut part), b"b");
        assert_eq!(part, b"");
        // A leading '.' is taken whatever it is.
        assert_eq!(next_option_part(b".a,b", &mut part), b"b");
        assert_eq!(part, b".a");
    }

    #[test]
    fn a_js_key_is_an_identifier_then_a_single_colon() {
        chartab();
        assert!(has_js_key(b"key: 1", 0));
        assert!(has_js_key(b"  key: 1", 0));
        assert!(has_js_key(b"'key': 1", 0));
        assert!(has_js_key(b"\"key\": 1", 0));
        assert!(!has_js_key(b"key", 0));
        assert!(!has_js_key(b"a::b", 0)); // C++ scope resolution
        assert!(!has_js_key(b": 1", 0)); // no identifier at all
    }

    #[test]
    fn a_terminator_is_the_character_the_statement_ended_with() {
        chartab();
        assert_eq!(terminator(b"x = 1;", 0, false, false), b';');
        assert_eq!(terminator(b"x = 1", 0, false, false), 0);
        assert_eq!(terminator(b"x = 1; // c", 0, false, false), b';');
        assert_eq!(terminator(b"x = 1; y", 0, false, false), 0);
        // A comma only counts when asked for.
        assert_eq!(terminator(b"x = 1,", 0, false, true), b',');
        assert_eq!(terminator(b"x = 1,", 0, false, false), 0);
        // So does an opening brace.
        assert_eq!(terminator(b"f() {", 0, true, false), b'{');
        assert_eq!(terminator(b"f() {", 0, false, false), 0);
        // A `;` inside a string is not the statement's.
        assert_eq!(terminator(b"x = \";\"", 0, false, false), 0);
        assert_eq!(terminator(b"x = \";\";", 0, false, false), b';');
    }

    #[test]
    fn a_trailing_comment_makes_the_last_byte_judged_twice() {
        chartab();
        // Upstream's string skip backs off the terminator unconditionally,
        // so once the walk steps into a trailing comment it lands on the
        // line's last byte and asks about that (O-B29-2).  Reproduced: a
        // `}` or `;` inside the *comment*, at the very end, terminates.
        assert_eq!(terminator(b"foo // }", 0, false, false), b'}');
        assert_eq!(terminator(b"foo // ;", 0, false, false), b';');
        // Anything else there is not a terminator, so nothing changes.
        assert_eq!(terminator(b"foo // x", 0, false, false), 0);
    }

    #[test]
    fn a_leading_brace_is_the_fallback_terminator() {
        chartab();
        // Nothing else terminates, so the `{`/`}` the line opens with is the
        // answer -- except after a `}` that an `else` continues.
        assert_eq!(terminator(b"{ x = 1", 0, false, false), b'{');
        assert_eq!(terminator(b"}", 0, false, false), b'}');
        assert_eq!(terminator(b"} else {", 0, false, false), 0);
        assert_eq!(terminator(b"} else", 0, false, false), 0);
    }

    #[test]
    fn a_line_ends_in_a_word_past_its_comments() {
        chartab();
        assert!(ends_in(b"foo,", 0, b","));
        assert!(ends_in(b"foo, // c", 0, b","));
        assert!(ends_in(b"foo, /* c */", 0, b","));
        assert!(ends_in(b"x = a[", 0, b"["));
        assert!(ends_in(b"};", 0, b"};"));
        assert!(!ends_in(b"foo, bar", 0, b","));
        assert!(!ends_in(b"foo", 0, b","));
    }

    #[test]
    fn a_backslash_continues_a_line_only_as_its_last_byte() {
        assert!(ends_in_backslash(b"#define X \\"));
        assert!(!ends_in_backslash(b"#define X \\ "));
        assert!(!ends_in_backslash(b"x"));
        assert!(!ends_in_backslash(b""));
    }

    #[test]
    fn a_control_clause_is_looked_for_just_left_of_the_paren() {
        chartab();
        // The offset handed in is the `(`'s; the answer is the keyword's.
        let mut offset = 2;
        assert!(control_clause_before(b"if(x)", &mut offset));
        assert_eq!(offset, 0);
        let mut offset = 7;
        assert!(control_clause_before(b"    if (x)", &mut offset));
        assert_eq!(offset, 4);
        let mut offset = 10;
        assert!(control_clause_before(b"    while (x)", &mut offset));
        assert_eq!(offset, 4);
        let mut offset = 8;
        assert!(control_clause_before(b"    for (x)", &mut offset));
        assert_eq!(offset, 4);
        // An identifier in front of the keyword makes it part of a name.
        let mut offset = 9;
        assert!(!control_clause_before(b"    xxif (x)", &mut offset));
        // Upstream's white-space skip is guarded by `offset > 2`, so a
        // keyword at the very left margin with a space after it is missed
        // (O-B29-1).  Reproduced: `if (x)` in column 0 is not one.
        let mut offset = 3;
        assert!(!control_clause_before(b"if (x)", &mut offset));
    }
}
