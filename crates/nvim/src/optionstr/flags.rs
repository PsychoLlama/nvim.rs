//! Options whose value is a set of flag letters or one word from a fixed
//! list.
//!
//! Two shapes, and they are not the same check.
//!
//! A **word list** option ('sessionoptions', 'switchbuf', 'backupcopy',
//! 'display', …) carries an array of accepted spellings in the generated
//! table. Matching a word sets its bit in the option's `flags_var`, so the
//! rest of the editor tests a bitmask instead of re-parsing the string.
//! [`opt_strings_flags`] is that check, and it is also the only thing that
//! keeps the mask in step with the value.
//!
//! A **flag letter** option ('formatoptions', 'cpoptions', 'shortmess',
//! 'whichwrap', …) carries a plain string of accepted letters and keeps its
//! value as a string. [`did_set_option_listflag`] only rejects a letter
//! outside that set; nothing derives a mask from it here.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_uint};
use core::ptr;

use crate::main::e_invarg;
use crate::option::{get_option, kOptFlagComma, kOptFlagOneComma, option_var};
use crate::options::{kOptFileformat, kOptFileformats, kOptSessionoptions, kOptViewoptions};
use crate::strings::vim_strchr;
use crate::types::{FAIL, Failed, OK, OptIndex, optset_T, size_t};

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
/// `:set fileformat=` fails. That forced pass is the whole reason this is a
/// `loop` rather than a `while`.
///
/// # Safety
/// `val` is a C string, from the option table or from an option's variable.
pub(crate) unsafe fn opt_strings_mask(
    val: *const c_char,
    values: &[&CStr],
    list: bool,
) -> Option<c_uint> {
    // SAFETY: the caller guarantees a C string.
    let value = unsafe { CStr::from_ptr(val) };
    words_mask(value.to_bytes(), values, list)
}

/// Whether every word in `val` is one the option accepts — the question the
/// callers that keep no mask are asking.
///
/// # Safety
/// As [`opt_strings_mask`].
pub(crate) unsafe fn opt_strings_ok(val: *const c_char, values: &[&CStr], list: bool) -> bool {
    // SAFETY: the caller's obligation.
    unsafe { opt_strings_mask(val, values, list) }.is_some()
}

/// The bitmask [`opt_strings_flags`] stores, or `None` for a value naming a
/// word that is not accepted.
fn words_mask(mut rest: &[u8], values: &[&CStr], list: bool) -> Option<c_uint> {
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

/// [`opt_strings_flags`] as an option-table callback reports it: null when
/// the value is good, "E474: Invalid argument" when it is not.
///
/// # Safety
/// As [`opt_strings_flags`].
pub(crate) unsafe fn did_set_opt_flags(
    val: *const c_char,
    values: &[&CStr],
    list: bool,
) -> *const c_char {
    if unsafe { opt_strings_ok(val, values, list) } {
        ptr::null()
    } else {
        e_invarg.as_ptr()
    }
}

/// The table callback for every option whose whole check is "is each word
/// one of the accepted ones".
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_str_generic(args: *mut optset_T) -> *const c_char {
    let (idx, varp) = unsafe { ((*args).os_idx, (*args).os_varp.string_var()) };
    if unsafe { check_str_opt(idx, varp) }.is_err() {
        e_invarg.as_ptr()
    } else {
        ptr::null()
    }
}

/// Reject the first letter of `val` that is not in `flags`.
///
/// The message goes into the caller's `errbuf`; see [`illegal_char`] for
/// what a null one means.
///
/// # Safety
/// `val` and `flags` are C strings; `errbuf` is null or points at
/// `errbuflen` writable bytes.
pub(crate) unsafe fn did_set_option_listflag(
    val: *const c_char,
    flags: *const c_char,
    errbuf: *mut c_char,
    errbuflen: size_t,
) -> *const c_char {
    // SAFETY: the caller guarantees a C string.
    for &byte in unsafe { CStr::from_ptr(val) }.to_bytes() {
        // SAFETY: `flags` is a C string; `vim_strchr` only reads it.
        if unsafe { vim_strchr(flags, c_int::from(byte)) }.is_null() {
            // SAFETY: the caller's buffer, as documented above.
            return unsafe { illegal_char(errbuf, errbuflen, c_int::from(byte)) };
        }
    }
    ptr::null()
}

/// Re-run an option's word-list check against its current value, refreshing
/// the mask. `varp` may be null for "wherever the option keeps its global
/// value".
///
/// # Safety
/// `varp` is null or points at the option's `char *` variable.
pub(crate) unsafe fn check_str_opt(idx: OptIndex, varp: *mut *mut c_char) -> Result<(), Failed> {
    let opt = get_option(idx);
    let varp = if varp.is_null() {
        option_var(idx).string_var()
    } else {
        varp
    };
    let list = opt.flags & (kOptFlagComma | kOptFlagOneComma) != 0;
    // SAFETY: the option's variable holds a C string.
    let Some(mask) = (unsafe { opt_strings_mask(*varp, opt_values(idx), list) }) else {
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
///
/// # Safety
/// `p` is a C string.
pub unsafe fn check_ff_value(p: *mut c_char) -> c_int {
    let values = opt_values(kOptFileformat);
    // SAFETY: the caller's C string, against the table's own word list.
    if unsafe { opt_strings_ok(p, values, false) } {
        OK
    } else {
        FAIL
    }
}

#[cfg(test)]
mod tests {
    use super::opens_with;

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
