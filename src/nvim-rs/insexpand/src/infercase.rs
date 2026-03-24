//! Completion case inference.
//!
//! This module provides case inference for completion text, matching the
//! case style of the originally typed text (infercase feature).

#![allow(
    dead_code,
    unused_imports,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::struct_field_names,
    clashing_extern_declarations
)]
use std::os::raw::{c_char, c_int};

/// Matches C `garray_T` exactly.
#[repr(C)]
struct GarrayT {
    ga_len: c_int,
    ga_maxlen: c_int,
    ga_itemsize: c_int,
    ga_growsize: c_int,
    ga_data: *mut core::ffi::c_void,
}

extern "C" {
    fn mb_ptr2char_adv(pp: *mut *const c_char) -> c_int;
    fn mb_islower(c: c_int) -> bool;
    fn mb_isupper(c: c_int) -> bool;
    fn mb_tolower(c: c_int) -> c_int;
    fn mb_toupper(c: c_int) -> c_int;
    fn utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;
    fn ga_init(gap: *mut GarrayT, itemsize: c_int, growsize: c_int);
    fn ga_grow(gap: *mut GarrayT, n: c_int);
    #[link_name = "xmalloc"]
    fn xmalloc_ic(size: usize) -> *mut u8;
    #[link_name = "xfree"]
    fn xfree_ic(p: *mut u8);
    fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    #[link_name = "IObuff"]
    static mut IObuff_infercase: [c_char; 1025];
}

const IOSIZE: usize = 1025;

/// Infer the case of completed text based on the originally typed text.
///
/// Returns the case-adjusted completion text. If a new allocation was made,
/// `*tofree` is set to the allocated buffer (caller must free it).
///
/// Rust translation of the C `nvim_ins_compl_infercase_gettext_impl` compound accessor.
///
/// # Safety
/// - `str_ptr` must point to a valid NUL-terminated C string.
/// - `tofree` must be a valid pointer to a mutable `*mut c_char` (initialized to null).
/// - The returned pointer is valid until `*tofree` is freed.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_infercase_gettext(
    str_ptr: *const c_char,
    char_len: c_int,
    compl_char_len: c_int,
    min_len: c_int,
    tofree: *mut *mut c_char,
) -> *mut c_char {
    let mut has_lower = false;

    // Allocate wide character array for the completion and fill it.
    let wca: *mut c_int = xmalloc_ic((char_len as usize) * core::mem::size_of::<c_int>()).cast();
    {
        let mut p: *const c_char = str_ptr;
        for i in 0..char_len {
            *wca.add(i as usize) = mb_ptr2char_adv(&raw mut p);
        }
    }

    // Rule 1: Were any chars converted to lower?
    {
        let mut p: *const c_char = crate::vars::compl_orig_text.data.cast_const();
        let mut i = 0;
        while i < min_len {
            let c = mb_ptr2char_adv(&raw mut p);
            if mb_islower(c) {
                has_lower = true;
                if mb_isupper(*wca.add(i as usize)) {
                    // Rule 1 is satisfied.
                    let mut j = compl_char_len;
                    while j < char_len {
                        *wca.add(j as usize) = mb_tolower(*wca.add(j as usize));
                        j += 1;
                    }
                    break;
                }
            }
            i += 1;
        }
    }

    // Rule 2: No lower case, 2nd consecutive letter converted to upper case.
    if !has_lower {
        let mut was_letter = false;
        let mut p: *const c_char = crate::vars::compl_orig_text.data.cast_const();
        let mut i = 0;
        while i < min_len {
            let c = mb_ptr2char_adv(&raw mut p);
            if was_letter && mb_isupper(c) && mb_islower(*wca.add(i as usize)) {
                // Rule 2 is satisfied.
                let mut j = compl_char_len;
                while j < char_len {
                    *wca.add(j as usize) = mb_toupper(*wca.add(j as usize));
                    j += 1;
                }
                break;
            }
            was_letter = mb_islower(c) || mb_isupper(c);
            i += 1;
        }
    }

    // Copy the original case of the part we typed.
    {
        let mut p: *const c_char = crate::vars::compl_orig_text.data.cast_const();
        for i in 0..min_len {
            let c = mb_ptr2char_adv(&raw mut p);
            if mb_islower(c) {
                *wca.add(i as usize) = mb_tolower(*wca.add(i as usize));
            } else if mb_isupper(c) {
                *wca.add(i as usize) = mb_toupper(*wca.add(i as usize));
            }
        }
    }

    // Generate encoding specific output from wide character array.
    let mut gap = GarrayT {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 1,
        ga_growsize: 500,
        ga_data: core::ptr::null_mut(),
    };
    ga_init(&raw mut gap, 1, 500);
    let mut p: *mut c_char = core::ptr::addr_of_mut!(IObuff_infercase).cast();
    let mut i = 0;
    while i < char_len {
        if !gap.ga_data.is_null() {
            ga_grow(&raw mut gap, 10);
            p = gap.ga_data.cast::<c_char>().add(gap.ga_len as usize);
            gap.ga_len += utf_char2bytes(*wca.add(i as usize), p);
            i += 1;
        } else if (p.offset_from(core::ptr::addr_of!(IObuff_infercase).cast::<c_char>()) as usize)
            + 6
            >= IOSIZE
        {
            // Multi-byte characters can occupy up to five bytes more than
            // ASCII characters, and we also need one byte for NUL, so when
            // getting to six bytes from the edge of IObuff switch to using a
            // growarray.  Add the character in the next round.
            ga_grow(&raw mut gap, IOSIZE as c_int);
            *p = 0;
            strcpy(
                gap.ga_data.cast(),
                core::ptr::addr_of!(IObuff_infercase).cast(),
            );
            gap.ga_len =
                p.offset_from(core::ptr::addr_of!(IObuff_infercase).cast::<c_char>()) as c_int;
            // Don't advance i -- add char in next iteration
        } else {
            p = p.add(utf_char2bytes(*wca.add(i as usize), p) as usize);
            i += 1;
        }
    }
    xfree_ic(wca.cast::<u8>());

    if !gap.ga_data.is_null() {
        *tofree = gap.ga_data.cast();
        return gap.ga_data.cast();
    }

    *p = 0;
    core::ptr::addr_of_mut!(IObuff_infercase).cast()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_exists() {
        // Module compiles successfully.
    }
}
