//! Port of `test/unit/keycodes_spec.lua`.

use std::ffi::{c_char, c_int};
use std::ptr;

use c2rust_neovim::src::nvim::keycodes::{find_special_key, FSK_IN_STRING};

use crate::support::cstr;

/// Wrapper over `find_special_key`: returns the key code and the modifier
/// mask it reported.
fn special_key(src: &str, flags: c_int) -> (c_int, c_int) {
    let s = cstr(src);
    let mut srcp: *const c_char = s.as_ptr();
    let mut modifiers: c_int = 0;
    let key =
        unsafe { find_special_key(&mut srcp, src.len(), &mut modifiers, flags, ptr::null_mut()) };
    (key, modifiers)
}

#[test]
fn find_special_key_no_keycode() {
    let (key, _) = special_key("abc", 0);
    assert_eq!(0, key);
}

#[test]
fn find_special_key_keycode_with_multiple_modifiers() {
    let (key, modifiers) = special_key("<C-M-S-A>", 0);
    assert_ne!(0, key);
    assert_ne!(0, modifiers);
}

#[test]
fn find_special_key_is_case_insensitive() {
    // Compare other capitalizations to this.
    let (all_caps_key, all_caps_mod) = special_key("<C-A>", 0);
    assert_eq!((all_caps_key, all_caps_mod), special_key("<C-a>", 0));
    assert_eq!((all_caps_key, all_caps_mod), special_key("<c-A>", 0));
    assert_eq!((all_caps_key, all_caps_mod), special_key("<c-a>", 0));
}

#[test]
fn find_special_key_double_quote_in_keycode() {
    let in_string = FSK_IN_STRING as c_int;

    // Unescaped with in_string=false
    assert_eq!('"' as c_int, special_key("<C-\">", 0).0);

    // Unescaped with in_string=true
    assert_eq!(0, special_key("<C-\">", in_string).0);

    // Escaped with in_string=false: should fail because the key is invalid
    // (more than 1 non-modifier character).
    assert_eq!(0, special_key("<C-\\\">", 0).0);

    // Escaped with in_string=true
    assert_eq!('"' as c_int, special_key("<C-\\\">", in_string).0);
}
