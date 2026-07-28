//! Characters and character classes: the `magic` toggles, the named
//! `[:alpha:]` classes and the collation elements a `[]` item can name.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub(crate) unsafe extern "C" fn no_Magic(mut x: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if x < 0 as ::core::ffi::c_int {
        return x + 256 as ::core::ffi::c_int;
    }
    return x;
}
pub(crate) unsafe extern "C" fn toggle_Magic(mut x: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if x < 0 as ::core::ffi::c_int {
        return x + 256 as ::core::ffi::c_int;
    }
    return x - 256 as ::core::ffi::c_int;
}
pub(crate) unsafe extern "C" fn backslash_trans(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    match c {
        114 => return CAR,
        116 => return TAB,
        101 => return ESC,
        98 => return BS,
        _ => {}
    }
    return c;
}
pub(crate) unsafe extern "C" fn get_char_class(
    mut pp: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if *(*pp).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == ':' as ::core::ffi::c_int
        && (*(*pp).offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
            >= 'a' as ::core::ffi::c_uint
            && *(*pp).offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                <= 'z' as ::core::ffi::c_uint)
        && (*(*pp).offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
            >= 'a' as ::core::ffi::c_uint
            && *(*pp).offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                <= 'z' as ::core::ffi::c_uint)
        && (*(*pp).offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
            >= 'a' as ::core::ffi::c_uint
            && *(*pp).offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                <= 'z' as ::core::ffi::c_uint)
    {
        static last_entry: GlobalCell<*mut keyvalue_T> =
            GlobalCell::new(::core::ptr::null_mut::<keyvalue_T>());
        let mut target: keyvalue_T = keyvalue_T {
            key: 0 as ::core::ffi::c_int,
            value: (*pp).offset(2 as ::core::ffi::c_int as isize),
            length: 0 as size_t,
        };
        let mut entry: *mut keyvalue_T = ::core::ptr::null_mut::<keyvalue_T>();
        if !(*last_entry.ptr()).is_null()
            && cmp_keyvalue_value_n(
                &raw mut target as *const ::core::ffi::c_void,
                last_entry.get() as *const ::core::ffi::c_void,
            ) == 0 as ::core::ffi::c_int
        {
            entry = last_entry.get();
        } else {
            entry = bsearch(
                &raw mut target as *const ::core::ffi::c_void,
                char_class_tab.ptr() as *const ::core::ffi::c_void,
                ::core::mem::size_of::<[keyvalue_T; 19]>()
                    .wrapping_div(::core::mem::size_of::<keyvalue_T>())
                    .wrapping_div(
                        (::core::mem::size_of::<[keyvalue_T; 19]>()
                            .wrapping_rem(::core::mem::size_of::<keyvalue_T>())
                            == 0) as ::core::ffi::c_int as size_t,
                    ),
                ::core::mem::size_of::<keyvalue_T>(),
                Some(
                    cmp_keyvalue_value_n
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_void,
                            *const ::core::ffi::c_void,
                        ) -> ::core::ffi::c_int,
                ),
            ) as *mut keyvalue_T;
        }
        if !entry.is_null() {
            last_entry.set(entry);
            *pp = (*pp).offset((*entry).length.wrapping_add(2 as size_t) as isize);
            return (*entry).key;
        }
    }
    return CLASS_NONE as ::core::ffi::c_int;
}
pub(crate) unsafe extern "C" fn init_class_tab() {
    let mut i: ::core::ffi::c_int = 0;
    static done: GlobalCell<::core::ffi::c_int> = GlobalCell::new(false_0);
    if done.get() != 0 {
        return;
    }
    i = 0 as ::core::ffi::c_int;
    while i < 256 as ::core::ffi::c_int {
        if i >= '0' as ::core::ffi::c_int && i <= '7' as ::core::ffi::c_int {
            (*class_tab.ptr())[i as usize] = (RI_DIGIT + RI_HEX + RI_OCTAL + RI_WORD) as int16_t;
        } else if i >= '8' as ::core::ffi::c_int && i <= '9' as ::core::ffi::c_int {
            (*class_tab.ptr())[i as usize] = (RI_DIGIT + RI_HEX + RI_WORD) as int16_t;
        } else if i >= 'a' as ::core::ffi::c_int && i <= 'f' as ::core::ffi::c_int {
            (*class_tab.ptr())[i as usize] =
                (RI_HEX + RI_WORD + RI_HEAD + RI_ALPHA + RI_LOWER) as int16_t;
        } else if i >= 'g' as ::core::ffi::c_int && i <= 'z' as ::core::ffi::c_int {
            (*class_tab.ptr())[i as usize] = (RI_WORD + RI_HEAD + RI_ALPHA + RI_LOWER) as int16_t;
        } else if i >= 'A' as ::core::ffi::c_int && i <= 'F' as ::core::ffi::c_int {
            (*class_tab.ptr())[i as usize] =
                (RI_HEX + RI_WORD + RI_HEAD + RI_ALPHA + RI_UPPER) as int16_t;
        } else if i >= 'G' as ::core::ffi::c_int && i <= 'Z' as ::core::ffi::c_int {
            (*class_tab.ptr())[i as usize] = (RI_WORD + RI_HEAD + RI_ALPHA + RI_UPPER) as int16_t;
        } else if i == '_' as ::core::ffi::c_int {
            (*class_tab.ptr())[i as usize] = (RI_WORD + RI_HEAD) as int16_t;
        } else {
            (*class_tab.ptr())[i as usize] = 0 as int16_t;
        }
        i += 1;
    }
    (*class_tab.ptr())[' ' as ::core::ffi::c_int as usize] =
        ((*class_tab.ptr())[' ' as ::core::ffi::c_int as usize] as ::core::ffi::c_int | RI_WHITE)
            as int16_t;
    (*class_tab.ptr())['\t' as ::core::ffi::c_int as usize] =
        ((*class_tab.ptr())['\t' as ::core::ffi::c_int as usize] as ::core::ffi::c_int | RI_WHITE)
            as int16_t;
    done.set(true_0);
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn re_multiline(mut prog: *const regprog_T) -> ::core::ffi::c_int {
    return ((*prog).regflags & RF_HASNL as ::core::ffi::c_uint) as ::core::ffi::c_int;
}
pub(crate) unsafe extern "C" fn get_equi_class(
    mut pp: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = 0;
    let mut l: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut p: *mut ::core::ffi::c_char = *pp;
    if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '=' as ::core::ffi::c_int
        && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
    {
        l = utfc_ptr2len(p.offset(2 as ::core::ffi::c_int as isize));
        if *p.offset((l + 2 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            == '=' as ::core::ffi::c_int
            && *p.offset((l + 3 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                == ']' as ::core::ffi::c_int
        {
            c = utf_ptr2char(p.offset(2 as ::core::ffi::c_int as isize));
            *pp = (*pp).offset((l + 4 as ::core::ffi::c_int) as isize);
            return c;
        }
    }
    return 0 as ::core::ffi::c_int;
}
pub(crate) unsafe extern "C" fn get_coll_element(
    mut pp: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = 0;
    let mut l: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut p: *mut ::core::ffi::c_char = *pp;
    if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '.' as ::core::ffi::c_int
        && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
    {
        l = utfc_ptr2len(p.offset(2 as ::core::ffi::c_int as isize));
        if *p.offset((l + 2 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            == '.' as ::core::ffi::c_int
            && *p.offset((l + 3 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                == ']' as ::core::ffi::c_int
        {
            c = utf_ptr2char(p.offset(2 as ::core::ffi::c_int as isize));
            *pp = (*pp).offset((l + 4 as ::core::ffi::c_int) as isize);
            return c;
        }
    }
    return 0 as ::core::ffi::c_int;
}
pub(crate) unsafe extern "C" fn get_cpo_flags() {
    reg_cpo_lit.set(!vim_strchr(p_cpo.get(), CPO_LITERAL).is_null() as ::core::ffi::c_int);
}
