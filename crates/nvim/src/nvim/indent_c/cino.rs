//! Parsing 'cinoptions'.
//!
//! One pass over the option string; each item is a letter, optionally
//! followed by `-`, then a signed number optionally suffixed by `s` meaning
//! "multiples of 'shiftwidth'" (with an optional `.N` fraction of one).
//! Every field it writes is a `b_ind_*` on the buffer, and those are the only
//! inputs `get_c_indent` has besides the text -- so this function is the
//! whole option surface of C indenting.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use ::core::ffi::c_int;

/// Set every `b_ind_*` on `buf` from its 'cinoptions'.
///
/// Must be called when 'cinoptions', 'shiftwidth' or 'tabstop' changes: the
/// `s` suffix and most of the defaults are multiples of 'shiftwidth', so the
/// parsed values go stale when it moves.
///
/// An unknown letter is silently ignored, and so is a letter's *value* when
/// the letter is unknown -- the grammar is parsed first and dispatched
/// second, which is why `set cinoptions=Z9` is accepted.
///
/// A value outside `int` range **aborts the process** (O-B15-4), through
/// `getdigits_int`'s strict arm: c2rust rendered upstream's C `assert()` as
/// an unconditional `assert!`, where release C truncates silently.  Left as
/// it is deliberately.  The defect is that one `assert!` and not this call --
/// `set shiftwidth=2147483647` aborts through the same path -- so clamping
/// here would hide one spelling of a tree-wide divergence and leave the rest.
/// Phase 16 owns the `assert!`/`debug_assert!` sweep; `fmtsweep`'s s91 pins
/// eleven aborted cases until then.
///
/// # Safety
/// `buf` must be a valid buffer.
pub unsafe fn parse_cino(buf: *mut buf_T) {
    unsafe {
        let sw = get_sw_value(buf);

        // The defaults.  A `sw` here means the option tracks 'shiftwidth'
        // unless 'cinoptions' overrides it.
        (*buf).b_ind_level = sw; // > the indent inside a block
        (*buf).b_ind_open_imag = 0; // e where a `{` ending a line is imagined
        (*buf).b_ind_no_brace = 0; // n extra when no `{` precedes the line
        (*buf).b_ind_first_open = 0; // f the column of a function's first `{`
        (*buf).b_ind_open_extra = 0; // { extra for a leftmost open brace
        (*buf).b_ind_close_extra = 0; // } extra for the matching close brace
        (*buf).b_ind_open_left_imag = 0; // ^ where a column-0 `{` is imagined
        (*buf).b_ind_jump_label = -1; // L shift for a jump label; <0 = column 1
        (*buf).b_ind_case = sw; // : `case xx` from the `switch`
        (*buf).b_ind_case_code = sw; // = the code from its `case xx:`
        (*buf).b_ind_case_break = 0; // b line a trailing `break` up with `case`
        (*buf).b_ind_scopedecl = sw; // g `private:` from the class declaration
        (*buf).b_ind_scopedecl_code = sw; // h the code from its `private:`
        (*buf).b_ind_param = sw; // p K&R-style parameters
        (*buf).b_ind_func_type = sw; // t a function's type specification
        (*buf).b_ind_cpp_baseclass = sw; // i a base class / constructor init
        (*buf).b_ind_continuation = sw; // + a continuation line
        (*buf).b_ind_unclosed = sw * 2; // ( from the line with the open paren
        (*buf).b_ind_unclosed2 = sw; // u for a paren that is itself unclosed
        (*buf).b_ind_unclosed_noignore = 0; // U do not ignore a leading `(`
        (*buf).b_ind_unclosed_wrapped = 0; // W a `(` last on its line
        (*buf).b_ind_unclosed_whiteok = 0; // w keep white space after a `(`
        (*buf).b_ind_matching_paren = 0; // m a `)` under its `(`'s line start
        (*buf).b_ind_paren_prev = 0; // M a `)` under the previous line
        (*buf).b_ind_comment = 0; // / extra for a comment
        (*buf).b_ind_in_comment = 3; // c from a comment opener with nothing after
        (*buf).b_ind_in_comment2 = 0; // C use `c` even when something follows
        (*buf).b_ind_maxparen = 20; // ) lines to search for an open paren
        (*buf).b_ind_maxcomment = 70; // * lines to search for an open comment
        (*buf).b_ind_java = 0; // j Java braces
        (*buf).b_ind_js = 0; // J JS object properties are not labels
        (*buf).b_ind_keep_case_label = 0; // l blocked `case` bodies
        (*buf).b_ind_cpp_namespace = 0; // N C++ `namespace`
        (*buf).b_ind_if_for_while = 0; // k conditions of if()/for()/while()
        (*buf).b_ind_hash_comment = 0; // # `#` comments
        (*buf).b_ind_cpp_extern_c = 0; // E C++ `extern "C"`
        (*buf).b_ind_pragma = 0; // P `#pragma` directives

        let mut p = (*buf).b_p_cino;
        while *p != 0 {
            let l = p;
            p = p.add(1);
            if *p as u8 == b'-' {
                p = p.add(1);
            }

            let digits_start = p;
            let mut n = int64_t::from(getdigits_int(&raw mut p, true, 0));

            // ".5s" is a fraction of a 'shiftwidth'.  Upstream declares
            // `fraction` outside the loop, but it is only ever read when
            // `divider` is non-zero and both are written by the same `if`, so
            // it never carries a value from one item to the next.
            let mut divider = 0;
            let mut fraction = 0;
            if *p as u8 == b'.' {
                p = p.add(1);
                fraction = atoi(p);
                while ascii_isdigit(c_int::from(*p as u8)) {
                    p = p.add(1);
                    divider = if divider == 0 { 10 } else { divider * 10 };
                }
            }
            // "2s" is two times 'shiftwidth'; a bare "s" is one.
            if *p as u8 == b's' {
                if p == digits_start {
                    n = int64_t::from(sw);
                } else {
                    n *= int64_t::from(sw);
                    if divider != 0 {
                        n += (int64_t::from(sw) * int64_t::from(fraction)
                            + int64_t::from(divider) / 2)
                            / int64_t::from(divider);
                    }
                }
                p = p.add(1);
            }
            if *l.add(1) as u8 == b'-' {
                n = -n;
            }
            let n = crate::src::nvim::math::trim_to_int(n);

            // When adding an entry here, also update the default 'cinoptions'
            // in doc/indent.txt, and add an explanation for it.
            let field: Option<&mut c_int> = match *l as u8 {
                b'>' => Some(&mut (*buf).b_ind_level),
                b'e' => Some(&mut (*buf).b_ind_open_imag),
                b'n' => Some(&mut (*buf).b_ind_no_brace),
                b'f' => Some(&mut (*buf).b_ind_first_open),
                b'{' => Some(&mut (*buf).b_ind_open_extra),
                b'}' => Some(&mut (*buf).b_ind_close_extra),
                b'^' => Some(&mut (*buf).b_ind_open_left_imag),
                b'L' => Some(&mut (*buf).b_ind_jump_label),
                b':' => Some(&mut (*buf).b_ind_case),
                b'=' => Some(&mut (*buf).b_ind_case_code),
                b'b' => Some(&mut (*buf).b_ind_case_break),
                b'p' => Some(&mut (*buf).b_ind_param),
                b't' => Some(&mut (*buf).b_ind_func_type),
                b'/' => Some(&mut (*buf).b_ind_comment),
                b'c' => Some(&mut (*buf).b_ind_in_comment),
                b'C' => Some(&mut (*buf).b_ind_in_comment2),
                b'i' => Some(&mut (*buf).b_ind_cpp_baseclass),
                b'+' => Some(&mut (*buf).b_ind_continuation),
                b'(' => Some(&mut (*buf).b_ind_unclosed),
                b'u' => Some(&mut (*buf).b_ind_unclosed2),
                b'U' => Some(&mut (*buf).b_ind_unclosed_noignore),
                b'W' => Some(&mut (*buf).b_ind_unclosed_wrapped),
                b'w' => Some(&mut (*buf).b_ind_unclosed_whiteok),
                b'm' => Some(&mut (*buf).b_ind_matching_paren),
                b'M' => Some(&mut (*buf).b_ind_paren_prev),
                b')' => Some(&mut (*buf).b_ind_maxparen),
                b'*' => Some(&mut (*buf).b_ind_maxcomment),
                b'g' => Some(&mut (*buf).b_ind_scopedecl),
                b'h' => Some(&mut (*buf).b_ind_scopedecl_code),
                b'j' => Some(&mut (*buf).b_ind_java),
                b'J' => Some(&mut (*buf).b_ind_js),
                b'l' => Some(&mut (*buf).b_ind_keep_case_label),
                b'#' => Some(&mut (*buf).b_ind_hash_comment),
                b'N' => Some(&mut (*buf).b_ind_cpp_namespace),
                b'k' => Some(&mut (*buf).b_ind_if_for_while),
                b'E' => Some(&mut (*buf).b_ind_cpp_extern_c),
                b'P' => Some(&mut (*buf).b_ind_pragma),
                _ => None,
            };
            if let Some(field) = field {
                *field = n;
            }

            if *p as u8 == b',' {
                p = p.add(1);
            }
        }
    }
}
