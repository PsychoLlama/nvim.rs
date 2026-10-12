//! Options whose value is a set of flag letters or one word from a fixed
//! list.
//!
//! Two shapes, and they are not the same check.
//!
//! A **word list** option ('sessionoptions', 'switchbuf', 'backupcopy',
//! 'display', …) carries an array of accepted spellings in the generated
//! table. Matching a word sets its bit in the option's `flags_var`, so the
//! rest of the editor tests a bitmask instead of re-parsing the string.
//! [`opt_strings_mask`] is that check, and it is also the only thing that
//! keeps the mask in step with the value.
//!
//! A **flag letter** option ('formatoptions', 'cpoptions', 'shortmess',
//! 'whichwrap', …) carries a plain string of accepted letters and keeps its
//! value as a string. [`did_set_option_listflag`] only rejects a letter
//! outside that set; nothing derives a mask from it here.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_int, c_uint};

use crate::message::e_invarg;
use crate::option::{get_option, kOptFlagComma, kOptFlagOneComma, option_var};
use crate::options::{kOptFileformat, kOptFileformats, kOptSessionoptions, kOptViewoptions};
use crate::types::{FAIL, Failed, OK, OptError, OptIndex, OptSet};

use super::illegal_char;

/// The accepted words for an option, as the generated table lists them.
///
/// Two options borrow another row's list rather than repeating it —
/// 'viewoptions' takes 'sessionoptions'' and 'fileformats' takes
/// 'fileformat''s.
pub(crate) fn opt_values(idx: OptIndex) -> &'static [&'static CStr] {
    let shared = match idx {
        kOptViewoptions => kOptSessionoptions,
        kOptFileformats => kOptFileformat,
        _ => idx,
    };
    get_option(shared).values
}

/// Does `value` open with `word`, ending there or at a separating comma?
///
/// The comma only counts for a list option; for a single-word option the
/// word has to be the whole value.
fn opens_with(value: &[u8], word: &[u8], list: bool) -> bool {
    value.starts_with(word)
        && ((list && value.get(word.len()) == Some(&b',')) || value.len() == word.len())
}

/// Match an option's value against its accepted words, and answer a bitmask
/// with bit *i* set for the *i*th accepted word the value named.
///
/// `None` for a value naming anything that is not accepted. Upstream writes
/// the mask through an `unsigned *` instead, which is what made six option
/// masks reachable as raw addresses; the caller that keeps a mask stores it.
///
/// The empty value is where the two shapes diverge. A list option's empty
/// value names nothing and is fine — the loop simply does not run. A
/// single-word option still takes one pass, which looks `""` up among the
/// accepted words like any other spelling; no option accepts it, so
/// `:set fileformat=` fails. That forced pass is what `once` is for.
pub(crate) fn opt_strings_mask(mut rest: &[u8], values: &[&CStr], list: bool) -> Option<c_uint> {
    let once = rest.is_empty() && !list;
    let mut mask: c_uint = 0;
    while !rest.is_empty() || once {
        let (bit, word) = values
            .iter()
            .map(|word| word.to_bytes())
            .enumerate()
            .find(|&(_, word)| opens_with(rest, word, list))?;
        assert!(bit < c_uint::BITS as usize, "more accepted words than bits");
        mask |= 1 << bit;
        rest = &rest[word.len()..];
        rest = rest.strip_prefix(b",").unwrap_or(rest);
        if once {
            break;
        }
    }
    Some(mask)
}

/// Whether every word in `value` is one the option accepts — the question
/// the callers that keep no mask are asking.
pub(crate) fn opt_strings_ok(value: &[u8], values: &[&CStr], list: bool) -> bool {
    opt_strings_mask(value, values, list).is_some()
}

/// [`opt_strings_ok`] as an option-table callback reports it: nothing when
/// the value is good, "E474: Invalid argument" when it is not.
pub(crate) fn did_set_opt_flags(
    value: &[u8],
    values: &[&CStr],
    list: bool,
) -> Result<(), OptError> {
    if opt_strings_ok(value, values, list) {
        Ok(())
    } else {
        Err(e_invarg.into())
    }
}

/// The table callback for every option whose whole check is "is each word
/// one of the accepted ones".
pub fn did_set_str_generic(args: &mut OptSet) -> Result<(), OptError> {
    let (idx, varp) = (args.os_idx, args.os_varp.string_var());
    if check_str_opt(idx, Some(varp)).is_err() {
        Err(e_invarg.into())
    } else {
        Ok(())
    }
}

/// Reject the first letter of `value` that is not in `flags`.
pub(crate) fn did_set_option_listflag(value: &[u8], flags: &CStr) -> Result<(), OptError> {
    match value.iter().find(|byte| !flags.to_bytes().contains(byte)) {
        Some(&byte) => Err(illegal_char(c_int::from(byte))),
        None => Ok(()),
    }
}

/// Re-run an option's word-list check against its current value, refreshing
/// the mask. `varp` is the option's variable, or `None` for "wherever the
/// option keeps its global value".
pub(crate) fn check_str_opt(
    idx: OptIndex,
    varp: Option<crate::option::StrVar>,
) -> Result<(), Failed> {
    let opt = get_option(idx);
    let varp = varp.unwrap_or_else(|| option_var(idx).string_var());
    let list = opt.flags & (kOptFlagComma | kOptFlagOneComma) != 0;
    let Some(mask) = varp.with_bytes(|value| opt_strings_mask(value, opt_values(idx), list)) else {
        return Err(Failed);
    };
    // The table names the mask cell itself; an option with no mask has none.
    if let Some(cell) = opt.flags_var {
        cell.set(mask);
    }
    Ok(())
}

/// Is `p` one of "unix", "dos" or "mac"? `OK` or `FAIL`.
///
/// `:read ++ff=…` and `:write ++ff=…` check their argument with this before
/// touching a buffer, which is why it is reachable without going through
/// `did_set_*`.
pub fn check_ff_value(p: &CStr) -> c_int {
    let values = opt_values(kOptFileformat);
    if opt_strings_ok(p.to_bytes(), values, false) {
        OK
    } else {
        FAIL
    }
}

#[cfg(test)]
mod tests {
    use super::{opens_with, opt_strings_mask};

    const WORDS: &[&core::ffi::CStr] = &[c"menu", c"menuone", c"popup"];

    #[test]
    fn a_list_sets_one_bit_per_word_named_and_repeats_are_harmless() {
        assert_eq!(opt_strings_mask(b"menu,popup", WORDS, true), Some(0b101));
        assert_eq!(
            opt_strings_mask(b"menuone,menu,menuone", WORDS, true),
            Some(0b011)
        );
        assert_eq!(opt_strings_mask(b"", WORDS, true), Some(0));
    }

    #[test]
    fn a_word_that_is_not_accepted_rejects_the_value() {
        assert_eq!(opt_strings_mask(b"menu,bogus", WORDS, true), None);
        assert_eq!(opt_strings_mask(b"menu,", WORDS, true), Some(0b001));
        assert_eq!(opt_strings_mask(b",menu", WORDS, true), None);
        assert_eq!(opt_strings_mask(b"menu,popup", WORDS, false), None);
    }

    #[test]
    fn a_single_word_option_looks_the_empty_value_up() {
        assert_eq!(opt_strings_mask(b"", WORDS, false), None);
        assert_eq!(opt_strings_mask(b"", &[c"", c"x"], false), Some(0b01));
        assert_eq!(opt_strings_mask(b"popup", WORDS, false), Some(0b100));
    }

    #[test]
    fn a_word_can_end_the_value_in_either_shape() {
        assert!(opens_with(b"unix", b"unix", false));
        assert!(opens_with(b"unix", b"unix", true));
    }

    #[test]
    fn only_a_list_may_carry_on_past_a_comma() {
        assert!(opens_with(b"unix,dos", b"unix", true));
        assert!(!opens_with(b"unix,dos", b"unix", false));
    }

    #[test]
    fn a_prefix_of_the_value_is_not_a_word() {
        assert!(!opens_with(b"unixy", b"unix", true));
        assert!(!opens_with(b"uni", b"unix", true));
    }

    #[test]
    fn the_empty_value_names_only_the_empty_word() {
        assert!(opens_with(b"", b"", false));
        assert!(!opens_with(b"", b"unix", false));
    }
}
