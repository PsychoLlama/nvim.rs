//! Port of `test/unit/garray_spec.lua`.

use std::ffi::{c_char, c_int, CStr};
use std::mem::size_of;
use std::ptr;

use c2rust_neovim::src::nvim::garray::{
    ga_append, ga_clear, ga_clear_strings, ga_concat, ga_concat_strings, ga_grow, ga_init,
    ga_remove_duplicate_strings, garray_T,
};
use c2rust_neovim::src::nvim::memory::xstrdup;

use crate::support::{cstr, internalize};

fn new_garray() -> garray_T {
    garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ptr::null_mut(),
    }
}

unsafe fn string_at(garr: &garray_T, i: usize) -> String {
    let ptr = *(garr.ga_data as *const *const c_char).add(i);
    String::from_utf8(CStr::from_ptr(ptr).to_bytes().to_vec()).unwrap()
}

/// The spec's `ga_append_string`: push an allocated copy of `s` onto a
/// string garray. `ga_clear_strings` frees the copies.
unsafe fn append_str(garr: &mut garray_T, s: &str) {
    assert_eq!(size_of::<*mut c_char>() as c_int, garr.ga_itemsize);
    let copy = xstrdup(cstr(s).as_ptr());
    ga_grow(garr, 1);
    *(garr.ga_data as *mut *mut c_char).add(garr.ga_len as usize) = copy;
    garr.ga_len += 1;
}

unsafe fn append_strs(garr: &mut garray_T, strs: &[&str]) {
    let prev = garr.ga_len;
    for s in strs {
        append_str(garr, s);
    }
    assert_eq!(prev + strs.len() as c_int, garr.ga_len);
}

#[test]
fn ga_init_initializes_the_values_of_the_garray() {
    let mut garr = new_garray();
    unsafe {
        ga_init(&mut garr, 14, 95);
        assert_eq!(0, garr.ga_len);
        assert_eq!(0, garr.ga_maxlen);
        assert_eq!(95, garr.ga_growsize);
        assert_eq!(14, garr.ga_itemsize);
        assert!(garr.ga_data.is_null());
        ga_clear(&mut garr);
    }
}

unsafe fn new_and_grow(itemsize: c_int, growsize: c_int, req: c_int) -> garray_T {
    let mut garr = new_garray();
    ga_init(&mut garr, itemsize, growsize);
    assert_eq!(0, garr.ga_len * garr.ga_itemsize); // should be empty at first
    assert!(garr.ga_data.is_null());
    ga_grow(&mut garr, req);
    garr
}

#[test]
fn ga_grow_grows_by_growsize_items_if_num_is_less_than_growsize() {
    unsafe {
        let mut garr = new_and_grow(16, 4, 3);
        assert!(!garr.ga_data.is_null());
        assert_eq!(4, garr.ga_maxlen); // requested LESS than growsize
        ga_clear(&mut garr);
    }
}

#[test]
fn ga_grow_grows_by_num_items_if_num_is_more_than_growsize() {
    unsafe {
        let mut garr = new_and_grow(16, 4, 5);
        assert!(!garr.ga_data.is_null());
        assert_eq!(5, garr.ga_maxlen); // requested MORE than growsize
        ga_clear(&mut garr);
    }
}

#[test]
fn ga_grow_does_not_grow_when_nothing_is_requested() {
    unsafe {
        let mut garr = new_and_grow(16, 4, 0);
        assert!(garr.ga_data.is_null());
        assert_eq!(0, garr.ga_maxlen);
        ga_clear(&mut garr);
    }
}

#[test]
fn ga_clear_clears_an_already_allocated_array() {
    unsafe {
        // Allocate and fill an array with nonzero bytes.
        let mut garr = new_garray();
        ga_init(&mut garr, 14, 95);
        ga_grow(&mut garr, 4);
        garr.ga_len = 4;
        let size = (garr.ga_len * garr.ga_itemsize) as usize;
        let bytes = std::slice::from_raw_parts_mut(garr.ga_data as *mut u8, size);
        for (i, b) in bytes.iter_mut().enumerate() {
            *b = (i * 37 + 11) as u8;
        }

        ga_clear(&mut garr);
        assert!(garr.ga_data.is_null());
        assert_eq!(0, garr.ga_maxlen);
        assert_eq!(0, garr.ga_len);
    }
}

#[test]
fn ga_append_can_append_bytes() {
    unsafe {
        let mut garr = new_garray();
        ga_init(&mut garr, size_of::<u8>() as c_int, 1);
        for &b in b"hello\0" {
            ga_append(&mut garr, b);
        }
        let s = CStr::from_ptr(garr.ga_data as *const c_char);
        assert_eq!(b"hello", s.to_bytes());
        ga_clear(&mut garr);
    }
}

#[test]
fn ga_append_can_append_integers() {
    unsafe {
        let mut garr = new_garray();
        ga_init(&mut garr, size_of::<c_int>() as c_int, 1);
        let input: [c_int; 5] = [-20, 94, 867615, 90927, 86];
        for &it in &input {
            ga_grow(&mut garr, 1);
            *(garr.ga_data as *mut c_int).add(garr.ga_len as usize) = it;
            garr.ga_len += 1;
        }
        let ints = std::slice::from_raw_parts(garr.ga_data as *const c_int, input.len());
        assert_eq!(input, ints);
        ga_clear(&mut garr);
    }
}

#[test]
fn ga_append_can_append_strings_to_a_growing_array_of_strings() {
    unsafe {
        let mut garr = new_garray();
        ga_init(&mut garr, size_of::<*mut c_char>() as c_int, 1);
        let input = ["some", "str", "\r\n\r●●●●●●,,,", "hmm", "got it"];
        append_strs(&mut garr, &input);
        for (i, s) in input.iter().enumerate() {
            assert_eq!(*s, string_at(&garr, i));
        }
        ga_clear_strings(&mut garr);
    }
}

#[test]
fn ga_concat_concatenates_the_parameter_to_the_growing_byte_array() {
    unsafe {
        let mut garr = new_garray();
        ga_init(&mut garr, size_of::<c_char>() as c_int, 1);
        let s = cstr("ohwell●●");
        for _ in 0..5 {
            ga_concat(&mut garr, s.as_ptr());
        }
        // ga_concat does NOT append the NUL of the src string; do it by hand
        // as the C code always does.
        ga_append(&mut garr, 0);
        let got = CStr::from_ptr(garr.ga_data as *const c_char);
        assert_eq!("ohwell●●".repeat(5).as_bytes(), got.to_bytes());
        ga_clear(&mut garr);
    }
}

unsafe fn check_concat_strings(input: &[&str], sep: &str) {
    let mut garr = new_garray();
    ga_init(&mut garr, size_of::<*mut c_char>() as c_int, 1);
    append_strs(&mut garr, input);
    let sep_c = cstr(sep);
    assert_eq!(
        input.join(sep),
        internalize(ga_concat_strings(&garr, sep_c.as_ptr()))
    );
    ga_clear_strings(&mut garr);
}

#[test]
fn ga_concat_strings_returns_an_empty_string_when_concatenating_an_empty_array() {
    unsafe {
        check_concat_strings(&[], ",");
        check_concat_strings(&[], "---");
    }
}

#[test]
fn ga_concat_strings_can_concatenate_a_non_empty_array() {
    unsafe {
        check_concat_strings(&["oh", "my", "neovim"], ",");
        check_concat_strings(&["oh", "my", "neovim"], "-●●-");
    }
}

#[test]
fn ga_remove_duplicate_strings_sorts_and_removes_duplicate_strings() {
    unsafe {
        let mut garr = new_garray();
        ga_init(&mut garr, size_of::<*mut c_char>() as c_int, 1);
        let input = [
            "ccc",
            "aaa",
            "bbb",
            "ddd●●",
            "aaa",
            "bbb",
            "ccc",
            "ccc",
            "ddd●●",
        ];
        let sorted_dedup = ["aaa", "bbb", "ccc", "ddd●●"];
        append_strs(&mut garr, &input);
        ga_remove_duplicate_strings(&mut garr);
        assert_eq!(sorted_dedup.len() as c_int, garr.ga_len);
        for (i, s) in sorted_dedup.iter().enumerate() {
            assert_eq!(*s, string_at(&garr, i));
        }
        ga_clear_strings(&mut garr);
    }
}
