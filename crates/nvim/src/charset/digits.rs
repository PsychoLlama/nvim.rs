//! Reading a decimal number out of text, as upstream's `getdigits` family
//! does: `strtoimax`'s shape (white space, a sign, digits), with the three
//! ways a caller can want an unrepresentable value answered.
//!
//! Original: `src/nvim/charset.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

use super::skip;
use crate::types::intmax_t;

/// The number `text[at..]` starts with and the offset past it, or `None`
/// when the value did not fit; see [`skip::decimal`].
pub(crate) fn try_getdigits_at(text: &[u8], at: usize) -> Option<(intmax_t, usize)> {
    let (number, used, overflow) = skip::decimal(&text[at..]);
    (!overflow).then_some((number, at + used))
}

/// The number `text[at..]` starts with, read as `strtoimax` reads one, and
/// the offset past what it consumed.
///
/// `strict` says the caller has already established that there *are* digits
/// here, so a value it cannot represent is a bad number rather than a parse
/// failure, and `def` would be misleading. Every one of those callers is
/// reading text a user typed -- an option value, a `:sign` id, a `:breakadd`
/// line number -- so an unrepresentable value **saturates** rather than
/// failing. Upstream `abort()`s here instead, which
/// `:set breakindentopt=min:99999999999999999999999` reaches from a modeline.
pub fn getdigits_at(text: &[u8], at: usize, strict: bool, def: intmax_t) -> (intmax_t, usize) {
    let (number, used, overflow) = skip::decimal(&text[at..]);
    (if overflow && !strict { def } else { number }, at + used)
}

/// [`getdigits_at`] narrowed to an `int`. A `strict` value outside the range
/// saturates; otherwise one that does not fit answers `def`.
pub fn getdigits_int_at(text: &[u8], at: usize, strict: bool, def: c_int) -> (c_int, usize) {
    let (number, past) = getdigits_at(text, at, strict, intmax_t::from(def));
    (narrow(number, strict, def), past)
}

/// A number narrowed to an `int`: clamped when `strict`, `def` when it does
/// not fit otherwise.
fn narrow(number: intmax_t, strict: bool, def: c_int) -> c_int {
    if strict {
        let clamped = number.clamp(intmax_t::from(c_int::MIN), intmax_t::from(c_int::MAX));
        return c_int::try_from(clamped).expect("clamped into `c_int`'s range");
    }
    c_int::try_from(number).unwrap_or(def)
}
