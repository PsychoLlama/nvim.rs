//! Parsing 'cinoptions'.
//!
//! One pass over the option string; each item is a letter, optionally
//! followed by `-`, then a signed number optionally suffixed by `s` meaning
//! "multiples of 'shiftwidth'" (with an optional `.N` fraction of one).
//! Every field it writes is a `b_ind_*` on the buffer, and those are the only
//! inputs `get_c_indent` has besides the text -- so this function is the
//! whole option surface of C indenting.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::charset::{atoi, getdigits_int_at};
use crate::option::TABSTOP_MAX;
use crate::optionstr::OptString;
use crate::winlayer::Buf;
use core::ffi::c_int;

/// Set every `b_ind_*` on `buffer` from its 'cinoptions'.
///
/// Must be called when 'cinoptions', 'shiftwidth' or 'tabstop' changes: the
/// `s` suffix and most of the defaults are multiples of 'shiftwidth', so the
/// parsed values go stale when it moves.
///
/// An unknown letter is silently ignored, and so is a letter's *value* when
/// the letter is unknown -- the grammar is parsed first and dispatched
/// second, which is why `set cinoptions=Z9` is accepted.
///
/// A value [`cino_amounts_fit`] refuses never gets here: the option setter
/// checks it first.
pub fn parse_cino(buffer: Buf) {
    let sw = get_sw_value(buffer);
    // A copy of the option: the fields below are written through the same
    // handle the value would be borrowed from.
    let cino = buffer.read().b_p_cino.get();

    // The defaults.  A `sw` here means the option tracks 'shiftwidth'
    // unless 'cinoptions' overrides it.
    buffer.edit().b_ind_level = sw; // > the indent inside a block
    buffer.edit().b_ind_open_imag = 0; // e where a `{` ending a line is imagined
    buffer.edit().b_ind_no_brace = 0; // n extra when no `{` precedes the line
    buffer.edit().b_ind_first_open = 0; // f the column of a function's first `{`
    buffer.edit().b_ind_open_extra = 0; // { extra for a leftmost open brace
    buffer.edit().b_ind_close_extra = 0; // } extra for the matching close brace
    buffer.edit().b_ind_open_left_imag = 0; // ^ where a column-0 `{` is imagined
    buffer.edit().b_ind_jump_label = -1; // L shift for a jump label; <0 = column 1
    buffer.edit().b_ind_case = sw; // : `case xx` from the `switch`
    buffer.edit().b_ind_case_code = sw; // = the code from its `case xx:`
    buffer.edit().b_ind_case_break = 0; // b line a trailing `break` up with `case`
    buffer.edit().b_ind_scopedecl = sw; // g `private:` from the class declaration
    buffer.edit().b_ind_scopedecl_code = sw; // h the code from its `private:`
    buffer.edit().b_ind_param = sw; // p K&R-style parameters
    buffer.edit().b_ind_func_type = sw; // t a function's type specification
    buffer.edit().b_ind_cpp_baseclass = sw; // i a base class / constructor init
    buffer.edit().b_ind_continuation = sw; // + a continuation line
    buffer.edit().b_ind_unclosed = sw.saturating_mul(2); // ( from the line with the open paren
    buffer.edit().b_ind_unclosed2 = sw; // u for a paren that is itself unclosed
    buffer.edit().b_ind_unclosed_noignore = 0; // U do not ignore a leading `(`
    buffer.edit().b_ind_unclosed_wrapped = 0; // W a `(` last on its line
    buffer.edit().b_ind_unclosed_whiteok = 0; // w keep white space after a `(`
    buffer.edit().b_ind_matching_paren = 0; // m a `)` under its `(`'s line start
    buffer.edit().b_ind_paren_prev = 0; // M a `)` under the previous line
    buffer.edit().b_ind_comment = 0; // / extra for a comment
    buffer.edit().b_ind_in_comment = 3; // c from a comment opener with nothing after
    buffer.edit().b_ind_in_comment2 = 0; // C use `c` even when something follows
    buffer.edit().b_ind_maxparen = 20; // ) lines to search for an open paren
    buffer.edit().b_ind_maxcomment = 70; // * lines to search for an open comment
    buffer.edit().b_ind_java = 0; // j Java braces
    buffer.edit().b_ind_js = 0; // J JS object properties are not labels
    buffer.edit().b_ind_keep_case_label = 0; // l blocked `case` bodies
    buffer.edit().b_ind_cpp_namespace = 0; // N C++ `namespace`
    buffer.edit().b_ind_if_for_while = 0; // k conditions of if()/for()/while()
    buffer.edit().b_ind_hash_comment = 0; // # `#` comments
    buffer.edit().b_ind_cpp_extern_c = 0; // E C++ `extern "C"`
    buffer.edit().b_ind_pragma = 0; // P `#pragma` directives

    let cino: &[u8] = &cino;

    let mut i = 0usize;
    while byte_at(cino, i) != 0 {
        let letter = i;
        i += 1;
        if byte_at(cino, i) == b'-' {
            i += 1;
        }

        let digits_start = i;
        let digits;
        (digits, i) = getdigits_int_at(cino, i, true, 0);
        let mut n = int64_t::from(digits);

        // ".5s" is a fraction of a 'shiftwidth'.  Upstream declares
        // `fraction` outside the loop, but it is only ever read when
        // `divider` is non-zero and both are written by the same `if`, so
        // it never carries a value from one item to the next.
        let mut divider = 0;
        let mut fraction = 0;
        if byte_at(cino, i) == b'.' {
            i += 1;
            fraction = atoi(&cino[i..]);
            while ascii_isdigit(c_int::from(byte_at(cino, i))) {
                i += 1;
                divider = if divider == 0 { 10 } else { divider * 10 };
            }
        }
        // "2s" is two times 'shiftwidth'; a bare "s" is one.
        if byte_at(cino, i) == b's' {
            if i == digits_start {
                n = int64_t::from(sw);
            } else {
                n *= int64_t::from(sw);
                if divider != 0 {
                    n += (int64_t::from(sw) * int64_t::from(fraction) + int64_t::from(divider) / 2)
                        / int64_t::from(divider);
                }
            }
            i += 1;
        }
        if byte_at(cino, letter + 1) == b'-' {
            n = -n;
        }
        let n = crate::math::trim_to_int(n);

        // When adding an entry here, also update the default 'cinoptions'
        // in doc/indent.txt, and add an explanation for it.
        let set: Option<fn(Buf, c_int)> = match byte_at(cino, letter) {
            b'>' => Some(Buf::set_b_ind_level as fn(Buf, c_int)),
            b'e' => Some(Buf::set_b_ind_open_imag as fn(Buf, c_int)),
            b'n' => Some(Buf::set_b_ind_no_brace as fn(Buf, c_int)),
            b'f' => Some(Buf::set_b_ind_first_open as fn(Buf, c_int)),
            b'{' => Some(Buf::set_b_ind_open_extra as fn(Buf, c_int)),
            b'}' => Some(Buf::set_b_ind_close_extra as fn(Buf, c_int)),
            b'^' => Some(Buf::set_b_ind_open_left_imag as fn(Buf, c_int)),
            b'L' => Some(Buf::set_b_ind_jump_label as fn(Buf, c_int)),
            b':' => Some(Buf::set_b_ind_case as fn(Buf, c_int)),
            b'=' => Some(Buf::set_b_ind_case_code as fn(Buf, c_int)),
            b'b' => Some(Buf::set_b_ind_case_break as fn(Buf, c_int)),
            b'p' => Some(Buf::set_b_ind_param as fn(Buf, c_int)),
            b't' => Some(Buf::set_b_ind_func_type as fn(Buf, c_int)),
            b'/' => Some(Buf::set_b_ind_comment as fn(Buf, c_int)),
            b'c' => Some(Buf::set_b_ind_in_comment as fn(Buf, c_int)),
            b'C' => Some(Buf::set_b_ind_in_comment2 as fn(Buf, c_int)),
            b'i' => Some(Buf::set_b_ind_cpp_baseclass as fn(Buf, c_int)),
            b'+' => Some(Buf::set_b_ind_continuation as fn(Buf, c_int)),
            b'(' => Some(Buf::set_b_ind_unclosed as fn(Buf, c_int)),
            b'u' => Some(Buf::set_b_ind_unclosed2 as fn(Buf, c_int)),
            b'U' => Some(Buf::set_b_ind_unclosed_noignore as fn(Buf, c_int)),
            b'W' => Some(Buf::set_b_ind_unclosed_wrapped as fn(Buf, c_int)),
            b'w' => Some(Buf::set_b_ind_unclosed_whiteok as fn(Buf, c_int)),
            b'm' => Some(Buf::set_b_ind_matching_paren as fn(Buf, c_int)),
            b'M' => Some(Buf::set_b_ind_paren_prev as fn(Buf, c_int)),
            b')' => Some(Buf::set_b_ind_maxparen as fn(Buf, c_int)),
            b'*' => Some(Buf::set_b_ind_maxcomment as fn(Buf, c_int)),
            b'g' => Some(Buf::set_b_ind_scopedecl as fn(Buf, c_int)),
            b'h' => Some(Buf::set_b_ind_scopedecl_code as fn(Buf, c_int)),
            b'j' => Some(Buf::set_b_ind_java as fn(Buf, c_int)),
            b'J' => Some(Buf::set_b_ind_js as fn(Buf, c_int)),
            b'l' => Some(Buf::set_b_ind_keep_case_label as fn(Buf, c_int)),
            b'#' => Some(Buf::set_b_ind_hash_comment as fn(Buf, c_int)),
            b'N' => Some(Buf::set_b_ind_cpp_namespace as fn(Buf, c_int)),
            b'k' => Some(Buf::set_b_ind_if_for_while as fn(Buf, c_int)),
            b'E' => Some(Buf::set_b_ind_cpp_extern_c as fn(Buf, c_int)),
            b'P' => Some(Buf::set_b_ind_pragma as fn(Buf, c_int)),
            _ => None,
        };
        if let Some(set) = set {
            set(buffer, n);
        }

        if byte_at(cino, i) == b',' {
            i += 1;
        }
    }
}

/// Whether every number in a 'cinoptions' value is one the indenter can
/// use: a whole part of at most [`TABSTOP_MAX`] and a fraction of at most
/// four digits. An amount is an indent, and a larger one only ever built a
/// line of megabytes of white space; the line counts (`)`, `*`) never need
/// that many either.
///
/// Every digit run is checked, wherever it sits, so the test does not have
/// to agree with [`parse_cino`] on which byte is a letter.
pub(crate) fn cino_amounts_fit(value: &[u8]) -> bool {
    let mut i = 0;
    while i < value.len() {
        if !value[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let start = i;
        while i < value.len() && value[i].is_ascii_digit() {
            i += 1;
        }
        let digits = &value[start..i];
        let fits = if start > 0 && value[start - 1] == b'.' {
            digits.len() <= 4
        } else {
            let significant = &digits[digits.iter().take_while(|&&d| d == b'0').count()..];
            significant.len() <= 4
                && significant
                    .iter()
                    .fold(0, |n, &d| n * 10 + c_int::from(d - b'0'))
                    <= TABSTOP_MAX
        };
        if !fits {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::cino_amounts_fit;

    #[test]
    fn amounts_up_to_the_tab_stop_bound_fit() {
        for value in [
            &b""[..],
            b">4,n-2,{.5s",
            b">9999,(09999",
            b"c.2500s",
            b"Z9",
            b"g0,)200,*500",
        ] {
            assert!(
                cino_amounts_fit(value),
                "{}",
                String::from_utf8_lossy(value)
            );
        }
        for value in [
            &b">10000"[..],
            b">2147483647",
            b"n-99999",
            b">1.99999s",
            b"(4,)100000",
        ] {
            assert!(
                !cino_amounts_fit(value),
                "{}",
                String::from_utf8_lossy(value)
            );
        }
    }
}
