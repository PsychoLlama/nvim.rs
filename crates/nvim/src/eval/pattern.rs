//! Matching and substituting with a regexp built from an expression.
//!
//! Both entry points compile their pattern with 'cpoptions' emptied, so
//! that a user's `cpo` flags cannot change what an expression's pattern
//! means. Restoring it is not a plain assignment — see `do_string_sub`.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::mbyte::cluster_len;
use crate::memory::XString;
use crate::option::SavedCpo;
use crate::option::vars::p_ic;
use crate::regexp::{OwnedMatch, RE_MAGIC, RE_STRING, regsub_to_vec, vim_regexec_nl};
use crate::types::TypVal;
use core::ffi::CStr;

/// Does `pat` match anywhere in `text`?
pub fn pattern_match(pat: &CStr, text: &CStr, ic: bool) -> bool {
    let _cpo = SavedCpo::empty_under_user_code();
    let Some(mut regmatch) = OwnedMatch::compile(pat, RE_MAGIC + RE_STRING, ic) else {
        return false;
    };
    vim_regexec_nl(&mut regmatch, text, 0)
}

/// `substitute()`: replace `pat` in `text` with `sub`, or with the result of
/// `expr` for a `\=` replacement. Every match is replaced when `flags`
/// starts with `g`, only the first otherwise.
///
/// With no match at all the answer is `text` copied through. A NUL a
/// replacement produced stays in the answer, which a C string reader stops
/// at.
pub fn do_string_sub(
    text: &CStr,
    pat: &CStr,
    sub: Option<&CStr>,
    expr: Option<&TypVal>,
    flags: &CStr,
) -> Vec<u8> {
    let _cpo = SavedCpo::empty_under_user_code();
    let subject = text.to_bytes();
    let mut out = Vec::<u8>::new();
    // Whether anything was substituted. An empty result is a real answer
    // (`substitute("x", "x", "", "")`), so the buffer cannot say it.
    let mut substituted = false;

    if let Some(mut regmatch) = OwnedMatch::compile(pat, RE_MAGIC + RE_STRING, p_ic()) {
        // The replacement is lent to each expansion: a `\=` one is
        // evaluated in place.
        let mut source = sub.map(XString::from_cstr);
        let do_all = flags.to_bytes().first() == Some(&b'g');
        let mut tail = 0;
        // The start of the last zero-width match, so that the next one
        // at the same place is stepped over rather than repeated.
        let mut zero_width: Option<usize> = None;

        while vim_regexec_nl(&mut regmatch, text, tail) {
            let span = regmatch.group(0).unwrap_or(0..0);
            if span.is_empty() {
                if zero_width == Some(span.start) {
                    // Copy one whole character across and try again.
                    let len = cluster_len(&subject[tail..]);
                    out.extend_from_slice(&subject[tail..tail + len]);
                    tail += len;
                    continue;
                }
                zero_width = Some(span.start);
            }

            let Some(replacement) = regsub_to_vec(&mut regmatch, text, source.as_mut(), expr)
            else {
                out.clear();
                substituted = false;
                break;
            };
            out.reserve(subject.len() - tail + replacement.len() - span.len());
            // The match starts at or after `tail`.
            out.extend_from_slice(&subject[tail..span.start]);
            out.extend_from_slice(&replacement);
            substituted = true;
            tail = span.end;
            if tail >= subject.len() || !do_all {
                break;
            }
        }

        if substituted {
            out.extend_from_slice(&subject[tail..]);
        }
    }

    if substituted { out } else { subject.to_vec() }
}
