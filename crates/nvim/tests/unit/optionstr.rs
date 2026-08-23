//! `'fileformat'`'s validity check, which is the one place the generated
//! option table's word list is consulted from outside the option machinery:
//! `:read ++ff=…` and `:write ++ff=…` route their argument through it before
//! any buffer is touched.
//!
//! Ported from `test/unit/optionstr_spec.lua`, which only covered the three
//! accepted spellings, the empty value and one rejected word.

use std::ffi::c_int;

use neovim::optionstr::check_ff_value;

use crate::support::cstr;

/// `check_ff_value` takes a mutable C string because the option machinery
/// hands it one; it does not write through it.
fn check(value: &str) -> c_int {
    let owned = cstr(value);
    unsafe { check_ff_value(owned.as_ptr().cast_mut()) }
}

#[test]
fn the_three_accepted_spellings_are_the_whole_list() {
    for value in ["unix", "dos", "mac"] {
        assert_eq!(check(value), 1, "{value:?} is a 'fileformat'");
    }
}

#[test]
fn anything_else_is_rejected() {
    for value in ["", "foo", "UNIX", "unixx", "uni", "unix "] {
        assert_eq!(check(value), 0, "{value:?} is not a 'fileformat'");
    }
}

/// 'fileformat' is a single-word option even though 'fileformats' shares its
/// word list, so a comma-separated value naming two accepted words is still
/// not an accepted value. The Lua spec never asked.
#[test]
fn a_comma_list_is_not_a_fileformat() {
    assert_eq!(check("unix,dos"), 0);
    assert_eq!(check("unix,"), 0);
}
