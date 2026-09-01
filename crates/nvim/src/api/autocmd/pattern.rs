//! Turning the `pattern` / `buffer` options into a pattern list.
//!
//! `get_patterns_from_pattern_or_buf` is the one place the two mutually
//! exclusive spellings are reconciled: a pattern string (or array of them,
//! each `<buffer>`-expanded and path-normalised) or a buffer number that
//! becomes a single `<buffer=N>` pattern.  `unpack_string_or_array` is the
//! "one or many" decoder it and the event list share.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::array_add;
use crate::api::private::validate::err_expected;
use crate::kvec::InitVec;

pub(crate) unsafe fn unpack_string_or_array(
    mut v: Object,
    mut k: *mut ::core::ffi::c_char,
    mut required: bool,
    mut arena: *mut Arena,
    err: &mut Error,
) -> Array {
    if matches!(v, Object::String(_)) {
        let mut arr: Array = arena_array(arena, 1 as size_t);
        unsafe { array_add(&mut arr, v) };
        return arr;
    } else if let Object::Array(array) = v {
        // SAFETY: `k` is a NUL-terminated key.
        let key = unsafe { core::ffi::CStr::from_ptr(k) };
        // SAFETY: the array is the caller's.
        if let Err(e) = unsafe { check_string_array(array, key, true) } {
            *err = e;
            return Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            };
        }
        return array;
    } else if !(!required && v.is_nil()) {
        let got = api_typename(v.kind());
        // SAFETY: `k` is a NUL-terminated key.
        let k = unsafe { core::ffi::CStr::from_ptr(k) };
        *err = err_expected(k, c"Array or String", Some(got));
        return Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
    }
    Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    }
}

pub(crate) unsafe fn get_patterns_from_pattern_or_buf(
    mut pattern: Object,
    mut has_buf: bool,
    mut buf: Buffer,
    mut fallback: *mut ::core::ffi::c_char,
    mut arena: *mut Arena,
    err: &mut Error,
) -> Array {
    let mut patterns: ArrayBuilder = ArrayBuilder {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
        init_array: [Object::Nil; 16],
    };
    patterns.capacity = ::core::mem::size_of::<[Object; 16]>()
        .wrapping_div(::core::mem::size_of::<Object>())
        .wrapping_div(
            (::core::mem::size_of::<[Object; 16]>().wrapping_rem(::core::mem::size_of::<Object>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    patterns.size = 0 as size_t;
    patterns.items = &raw mut patterns.init_array as *mut Object;
    if !pattern.is_nil() {
        if let Object::String(string) = pattern {
            let mut pat: *const ::core::ffi::c_char = string.data();
            let mut patlen: size_t = unsafe { aucmd_span_pattern(pat, &raw mut pat) };
            while patlen != 0 {
                // `kv_push`, whose growth step c2rust expanded inline.
                InitVec::new(
                    &mut patterns.size,
                    &mut patterns.capacity,
                    &mut patterns.items,
                    &mut patterns.init_array,
                )
                .push(Object::string(unsafe {
                    arena_string(
                        arena,
                        String_0::from_raw_parts(pat as *mut ::core::ffi::c_char, patlen),
                    )
                }));
                patlen = unsafe { aucmd_span_pattern(pat.add(patlen), &raw mut pat) };
            }
        } else if let Object::Array(array) = pattern {
            // SAFETY: the array is the caller's.
            if let Err(e) = unsafe { check_string_array(array, c"pattern", true) } {
                *err = e;
                return Array {
                    size: 0 as size_t,
                    capacity: 0 as size_t,
                    items: ::core::ptr::null_mut::<Object>(),
                };
            }
            let mut entry_index: size_t = 0 as size_t;
            while entry_index < array.size {
                let entry: Object = unsafe { *array.items.add(entry_index) };
                let entry = entry
                    .as_string()
                    .expect("`check_string_array` accepted only Strings");
                let mut pat_0: *const ::core::ffi::c_char = entry.data();
                let mut patlen_0: size_t = unsafe { aucmd_span_pattern(pat_0, &raw mut pat_0) };
                while patlen_0 != 0 {
                    // `kv_push`, whose growth step c2rust expanded inline.
                    InitVec::new(
                        &mut patterns.size,
                        &mut patterns.capacity,
                        &mut patterns.items,
                        &mut patterns.init_array,
                    )
                    .push(Object::string(unsafe {
                        arena_string(
                            arena,
                            String_0::from_raw_parts(pat_0 as *mut ::core::ffi::c_char, patlen_0),
                        )
                    }));
                    patlen_0 = unsafe { aucmd_span_pattern(pat_0.add(patlen_0), &raw mut pat_0) };
                }
                entry_index = entry_index.wrapping_add(1);
            }
        } else if true {
            let want = c"String or Table";
            let got = api_typename(pattern.kind());
            *err = err_expected(c"pattern", want, Some(got));
            return Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            };
        }
    } else if has_buf {
        let mut b: *mut buf_T = unsafe { find_buffer_by_handle(buf, err) };
        if err.kind() as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            };
        }
        // `kv_push`, whose growth step c2rust expanded inline.
        InitVec::new(
            &mut patterns.size,
            &mut patterns.capacity,
            &mut patterns.items,
            &mut patterns.init_array,
        )
        .push(Object::string(unsafe {
            arena_printf(arena, c"<buffer=%d>".as_ptr(), (*b).handle)
        }));
    }
    if patterns.size == 0 as size_t && !fallback.is_null() {
        // `kv_push`, whose growth step c2rust expanded inline.
        InitVec::new(
            &mut patterns.size,
            &mut patterns.capacity,
            &mut patterns.items,
            &mut patterns.init_array,
        )
        .push(Object::string(unsafe { cstr_as_string(fallback) }));
    }
    unsafe { arena_take_arraybuilder(arena, &raw mut patterns) }
}
