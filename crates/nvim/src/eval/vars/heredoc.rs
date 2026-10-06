//! `=<< MARKER` -- the here-document form of an assignment.
//!
//! [`heredoc_get`] collects the lines and applies `trim`'s indent rules; the
//! two `eval_*_expr_in_str` implement `eval`'s `{expr}` interpolation, which
//! is the only thing in the file that evaluates its own input.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::ascii::{ascii_iswhite, ascii_iswhite_or_nul};
use crate::cstr::byte_at;
use crate::eval::typval::{ListRef, TV_INITIAL_VALUE, tv_list_alloc};
use crate::eval::{Cursor, eval_to_string, eval1};
use crate::ex_docmd::exarg_getline;
use crate::memory::XString;
use crate::message_fmt::msg_bytes;
use crate::os::cshim::is_lower_in_locale;
use crate::semsg;
use crate::types::{ExArg, TypVal};
use core::ffi::c_int;

use super::{e_cannot_use_heredoc_here, emsg_static};

/// The comment character a marker line may carry after it.
const COMMENT_CHAR: u8 = b'"';

/// Evaluate the `{expr}` block `text` starts with and append its value to
/// `gap`. Answers how many bytes of `text` the block took, closing brace
/// included, or `None` after an error.
///
/// `evaluate` false parses the block without running it, which is what a
/// skipped `:let` wants.
pub(crate) fn eval_one_expr_in_text(
    text: &[u8],
    gap: &mut Vec<u8>,
    evaluate: bool,
) -> Option<usize> {
    let missing = || {
        let text = msg_bytes(text);
        semsg!("E1279: Missing '}}': {text}");
    };
    let mut cursor = Cursor::new(text);
    cursor.bump(1);
    cursor.skip_white();
    let block_start = cursor.offset();
    if cursor.byte() == 0 {
        missing();
        return None;
    }
    let mut skipped = TV_INITIAL_VALUE;
    eval1(&mut cursor, &mut skipped, false).ok()?;
    cursor.skip_white();
    let block_end = cursor.offset();
    if cursor.byte() != b'}' {
        missing();
        return None;
    }
    if evaluate {
        let value = eval_to_string(&text[block_start..block_end], false, false)?;
        gap.extend_from_slice(&value);
    }
    Some(block_end + 1)
}

/// Every `{expr}` in `line` evaluated into the text around it, or `None`
/// after an error. `{{` and `}}` are the escapes for a literal brace.
fn eval_all_expr_in_str(line: &[u8]) -> Option<Vec<u8>> {
    let mut text = Vec::<u8>::new();
    let mut p = 0;
    while byte_at(line, p) != 0 {
        let mut escaped_brace = false;

        // Everything up to the next brace is literal.
        let lit_start = p;
        while !matches!(byte_at(line, p), b'{' | b'}' | 0) {
            p += 1;
        }

        let here = byte_at(line, p);
        if here != 0 && here == byte_at(line, p + 1) {
            // A doubled brace: keep one of the pair in the literal part
            // and skip the other below.
            p += 1;
            escaped_brace = true;
        } else if here == b'}' {
            let line = msg_bytes(line);
            semsg!("E1278: Stray '}}' without a matching '{{': {line}");
            return None;
        }

        text.extend_from_slice(&line[lit_start..p]);
        if here == 0 {
            break;
        }
        if escaped_brace {
            p += 1;
            continue;
        }

        p += eval_one_expr_in_text(&line[p..], &mut text, true)?;
    }
    Some(text)
}

/// Collect the lines of a here-document into a List, or answer `None`.
///
/// ```text
///     cmd << {marker}
///       {line1}
///       {line2}
///       ...
///     {marker}
/// ```
///
/// `at` is where the command line continues after the `<<`. `trim` before
/// the marker strips the leading indentation the *first* body line has, and
/// strips the `:let` line's own indentation when looking for the end
/// marker. `eval` runs `{expr}` interpolation over every line. `script_get`
/// is an embedded script (`:lua <<`, `:python <<` and friends): a missing
/// marker is then `.`, a lower-case marker is allowed, and a missing end
/// marker is not an error.
///
/// A here-document inside a string (`execute "let x =<< END\n...\nEND"`) is
/// its own body, newline-separated; the marker and each body line are
/// terminated in the command line where they end, as the C did, so that
/// whatever reads the line after this sees them cut.
pub(crate) fn heredoc_get(excmd: &mut ExArg, at: usize, script_get: bool) -> Option<ListRef> {
    let mut marker_indent_len = 0;
    // `trim` asks for the first body line's indent; this is it once known.
    let mut text_indent: Option<Vec<u8>> = None;
    let mut want_text_indent = false;

    // A here-document inside a string argument is the whole body,
    // newline-separated, rather than lines read from the source.
    let mut line_arg = None;
    if let Some(nl) = excmd.line.rest_of(at).iter().position(|&b| b == b'\n') {
        excmd.line.set_byte(at + nl, 0);
        line_arg = Some(at + nl + 1);
    } else if excmd.ea_getline.is_none() {
        emsg_static(e_cannot_use_heredoc_here);
        return None;
    }

    // Whether `at` starts with the four-letter `word` as a whole word.
    let is_word = |excmd: &ExArg, at: usize, word: &[u8]| {
        excmd.line.starts_with(at, word)
            && ascii_iswhite_or_nul(c_int::from(excmd.line.byte_at(at + 4)))
    };

    // The optional `trim` and `eval` words before the marker, in either
    // order and either number.
    let mut cmd = excmd.line.skip_white(at);
    let mut evalstr = false;
    loop {
        if is_word(excmd, cmd, b"trim") {
            cmd = excmd.line.skip_white(cmd + 4);
            // The end marker is matched with the `:let` line's own
            // indentation stripped; the body's comes from its first
            // line.
            marker_indent_len += excmd.line.skip_white(0);
            want_text_indent = true;
        } else if is_word(excmd, cmd, b"eval") {
            cmd = excmd.line.skip_white(cmd + 4);
            evalstr = true;
        } else {
            break;
        }
    }

    // The marker is the next word.
    let lead = excmd.line.byte_at(cmd);
    let marker: Vec<u8> = if lead != 0 && lead != COMMENT_CHAR {
        let p = excmd.line.skip_to_white(cmd);
        let after = excmd.line.byte_at(excmd.line.skip_white(p));
        if after != 0 && after != COMMENT_CHAR {
            let rest = msg_bytes(excmd.line.rest_of(p));
            semsg!("E488: Trailing characters: {rest}");
            return None;
        }
        excmd.line.set_byte(p, 0);
        let marker = excmd.line.rest_of(cmd).to_vec();
        // `islower` here is the locale's, not ASCII's: in a non-C locale it
        // covers more than a-z.
        if !script_get && is_lower_in_locale(marker[0]) {
            emsg_static(c"E221: Marker cannot start with lower case letter");
            return None;
        }
        marker
    } else if script_get {
        // An embedded script with no marker takes '.'.
        b".".to_vec()
    } else {
        emsg_static(c"E172: Missing marker");
        return None;
    };
    // A line shorter than the indent it is asked for never matches it.
    let indent = excmd
        .line
        .line()
        .get(..marker_indent_len)
        .map(<[u8]>::to_vec);

    let mut eval_failed = false;
    let mut list = tv_list_alloc(0);
    loop {
        let theline = match line_arg {
            Some(next) => {
                if excmd.line.byte_at(next) == 0 {
                    if !script_get {
                        let marker = msg_bytes(&marker);
                        semsg!("E990: Missing end marker '{marker}'");
                    }
                    break;
                }
                let rest = excmd.line.rest_of(next);
                let (len, after) = match rest.iter().position(|&b| b == b'\n') {
                    Some(nl) => (nl, next + nl + 1),
                    None => (rest.len(), next + rest.len()),
                };
                if after != next + len {
                    excmd.line.set_byte(next + len, 0);
                }
                line_arg = Some(after);
                excmd.line.rest_of(next).to_vec()
            }
            None => match exarg_getline(excmd, 0, 0, false) {
                Some(line) => line.to_vec(),
                None => {
                    if !script_get {
                        let marker = msg_bytes(&marker);
                        semsg!("E990: Missing end marker '{marker}'");
                    }
                    break;
                }
            },
        };

        // With `trim`, skip the indent matching the `:let` line before
        // looking for the marker.
        let mi = if marker_indent_len > 0
            && indent
                .as_ref()
                .is_some_and(|indent| theline.starts_with(indent))
        {
            marker_indent_len
        } else {
            0
        };
        if theline[mi..] == marker[..] {
            break;
        }

        // Once interpolation has failed, the rest of the body is only
        // read to find the end marker.
        if eval_failed {
            continue;
        }

        if want_text_indent && !theline.is_empty() {
            // The body's indent is the first non-empty line's.
            let len = theline
                .iter()
                .take_while(|&&b| ascii_iswhite(c_int::from(b)))
                .count();
            text_indent = Some(theline[..len].to_vec());
            want_text_indent = false;
        }
        // With `trim`, skip as much of that indent as this line matches.
        let ti = text_indent.as_ref().map_or(0, |indent| {
            indent
                .iter()
                .zip(&theline)
                .take_while(|(a, b)| a == b)
                .count()
        });

        let line = &theline[ti..];
        if evalstr && !excmd.skip {
            let Some(evaluated) = eval_all_expr_in_str(line) else {
                eval_failed = true;
                continue;
            };
            list.push(owned_string(&evaluated));
        } else {
            list.push(owned_string(line));
        }
    }

    if let Some(next) = line_arg {
        // The next command follows the here-document in the string.
        excmd.line.next = Some(next);
    }

    // The partly built list goes with the handle, which is its only
    // reference.
    (!eval_failed).then_some(list)
}

/// A String value owning a copy of `text`.
fn owned_string(text: &[u8]) -> TypVal {
    TypVal::String(XString::from_bytes(text).into_raw())
}
