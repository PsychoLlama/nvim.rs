//! Turning the `pattern` / `buffer` options into a pattern list.
//!
//! `get_patterns_from_pattern_or_buf` is the one place the two mutually
//! exclusive spellings are reconciled: a pattern string (or array of them,
//! each `<buffer>`-expanded and path-normalised) or a buffer number that
//! becomes a single `<buffer=N>` pattern.  `unpack_string_or_array` is the
//! "one or many" decoder it and the event list share.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn unpack_string_or_array(
    mut v: Object,
    mut k: *mut ::core::ffi::c_char,
    mut required: bool,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    unsafe {
        if v.type_0 as ::core::ffi::c_uint
            == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut arr: Array = arena_array(arena, 1 as size_t);
            let c2rust_fresh23 = arr.size;
            arr.size = arr.size.wrapping_add(1);
            *arr.items.offset(c2rust_fresh23 as isize) = v;
            return arr;
        } else if v.type_0 as ::core::ffi::c_uint
            == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if !check_string_array(v.data.array, k, true_0 != 0, err) {
                return Array {
                    size: 0 as size_t,
                    capacity: 0 as size_t,
                    items: ::core::ptr::null_mut::<Object>(),
                };
            }
            return v.data.array;
        } else if !(!required
            && v.type_0 as ::core::ffi::c_uint
                == kObjectTypeNil as ::core::ffi::c_int as ::core::ffi::c_uint)
        {
            api_err_exp(
                err,
                k,
                b"Array or String\0".as_ptr() as *const ::core::ffi::c_char,
                api_typename(v.type_0),
            );
            return Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            };
        }
        return Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
    }
}

pub(crate) unsafe extern "C" fn get_patterns_from_pattern_or_buf(
    mut pattern: Object,
    mut has_buf: bool,
    mut buf: Buffer,
    mut fallback: *mut ::core::ffi::c_char,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    unsafe {
        let mut patterns: ArrayBuilder = ArrayBuilder {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
            init_array: [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            }; 16],
        };
        patterns.capacity = ::core::mem::size_of::<[Object; 16]>()
            .wrapping_div(::core::mem::size_of::<Object>())
            .wrapping_div(
                (::core::mem::size_of::<[Object; 16]>()
                    .wrapping_rem(::core::mem::size_of::<Object>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as size_t;
        patterns.size = 0 as size_t;
        patterns.items = &raw mut patterns.init_array as *mut Object;
        if pattern.type_0 as ::core::ffi::c_uint
            != kObjectTypeNil as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if pattern.type_0 as ::core::ffi::c_uint
                == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut pat: *const ::core::ffi::c_char = pattern.data.string.data;
                let mut patlen: size_t = aucmd_span_pattern(pat, &raw mut pat);
                while patlen != 0 {
                    if patterns.size == patterns.capacity {
                        patterns.capacity = if patterns.capacity << 1 as ::core::ffi::c_int
                            > ::core::mem::size_of::<[Object; 16]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 16]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            patterns.capacity << 1 as ::core::ffi::c_int
                        } else {
                            ::core::mem::size_of::<[Object; 16]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 16]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                )
                        };
                        patterns.items = (if patterns.capacity
                            == ::core::mem::size_of::<[Object; 16]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 16]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            if patterns.items == &raw mut patterns.init_array as *mut Object {
                                patterns.items as *mut ::core::ffi::c_void
                            } else {
                                _memcpy_free(
                                    &raw mut patterns.init_array as *mut Object
                                        as *mut ::core::ffi::c_void,
                                    patterns.items as *mut ::core::ffi::c_void,
                                    patterns.size.wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            }
                        } else {
                            if patterns.items == &raw mut patterns.init_array as *mut Object {
                                memcpy(
                                    xmalloc(
                                        patterns
                                            .capacity
                                            .wrapping_mul(::core::mem::size_of::<Object>()),
                                    ),
                                    patterns.items as *const ::core::ffi::c_void,
                                    patterns.size.wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            } else {
                                xrealloc(
                                    patterns.items as *mut ::core::ffi::c_void,
                                    patterns
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            }
                        }) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh19 = patterns.size;
                    patterns.size = patterns.size.wrapping_add(1);
                    *patterns.items.offset(c2rust_fresh19 as isize) = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed {
                            string: arena_string(
                                arena,
                                String_0 {
                                    data: pat as *mut ::core::ffi::c_char,
                                    size: patlen,
                                },
                            ),
                        },
                    };
                    patlen = aucmd_span_pattern(pat.offset(patlen as isize), &raw mut pat);
                }
            } else if pattern.type_0 as ::core::ffi::c_uint
                == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if !check_string_array(
                    pattern.data.array,
                    b"pattern\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    true_0 != 0,
                    err,
                ) {
                    return Array {
                        size: 0 as size_t,
                        capacity: 0 as size_t,
                        items: ::core::ptr::null_mut::<Object>(),
                    };
                }
                let mut array: Array = pattern.data.array;
                let mut entry_index: size_t = 0 as size_t;
                while entry_index < array.size {
                    let mut entry: Object = *array.items.offset(entry_index as isize);
                    let mut pat_0: *const ::core::ffi::c_char = entry.data.string.data;
                    let mut patlen_0: size_t = aucmd_span_pattern(pat_0, &raw mut pat_0);
                    while patlen_0 != 0 {
                        if patterns.size == patterns.capacity {
                            patterns.capacity = if patterns.capacity << 1 as ::core::ffi::c_int
                                > ::core::mem::size_of::<[Object; 16]>()
                                    .wrapping_div(::core::mem::size_of::<Object>())
                                    .wrapping_div(
                                        (::core::mem::size_of::<[Object; 16]>()
                                            .wrapping_rem(::core::mem::size_of::<Object>())
                                            == 0)
                                            as ::core::ffi::c_int
                                            as usize,
                                    ) {
                                patterns.capacity << 1 as ::core::ffi::c_int
                            } else {
                                ::core::mem::size_of::<[Object; 16]>()
                                    .wrapping_div(::core::mem::size_of::<Object>())
                                    .wrapping_div(
                                        (::core::mem::size_of::<[Object; 16]>()
                                            .wrapping_rem(::core::mem::size_of::<Object>())
                                            == 0)
                                            as ::core::ffi::c_int
                                            as size_t,
                                    )
                            };
                            patterns.items = (if patterns.capacity
                                == ::core::mem::size_of::<[Object; 16]>()
                                    .wrapping_div(::core::mem::size_of::<Object>())
                                    .wrapping_div(
                                        (::core::mem::size_of::<[Object; 16]>()
                                            .wrapping_rem(::core::mem::size_of::<Object>())
                                            == 0)
                                            as ::core::ffi::c_int
                                            as usize,
                                    ) {
                                if patterns.items == &raw mut patterns.init_array as *mut Object {
                                    patterns.items as *mut ::core::ffi::c_void
                                } else {
                                    _memcpy_free(
                                        &raw mut patterns.init_array as *mut Object
                                            as *mut ::core::ffi::c_void,
                                        patterns.items as *mut ::core::ffi::c_void,
                                        patterns
                                            .size
                                            .wrapping_mul(::core::mem::size_of::<Object>()),
                                    )
                                }
                            } else {
                                if patterns.items == &raw mut patterns.init_array as *mut Object {
                                    memcpy(
                                        xmalloc(
                                            patterns
                                                .capacity
                                                .wrapping_mul(::core::mem::size_of::<Object>()),
                                        ),
                                        patterns.items as *const ::core::ffi::c_void,
                                        patterns
                                            .size
                                            .wrapping_mul(::core::mem::size_of::<Object>()),
                                    )
                                } else {
                                    xrealloc(
                                        patterns.items as *mut ::core::ffi::c_void,
                                        patterns
                                            .capacity
                                            .wrapping_mul(::core::mem::size_of::<Object>()),
                                    )
                                }
                            }) as *mut Object;
                        } else {
                        };
                        let c2rust_fresh20 = patterns.size;
                        patterns.size = patterns.size.wrapping_add(1);
                        *patterns.items.offset(c2rust_fresh20 as isize) = object {
                            type_0: kObjectTypeString,
                            data: C2Rust_Unnamed {
                                string: arena_string(
                                    arena,
                                    String_0 {
                                        data: pat_0 as *mut ::core::ffi::c_char,
                                        size: patlen_0,
                                    },
                                ),
                            },
                        };
                        patlen_0 =
                            aucmd_span_pattern(pat_0.offset(patlen_0 as isize), &raw mut pat_0);
                    }
                    entry_index = entry_index.wrapping_add(1);
                }
            } else if true {
                api_err_exp(
                    err,
                    b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                    b"String or Table\0".as_ptr() as *const ::core::ffi::c_char,
                    api_typename(pattern.type_0),
                );
                return Array {
                    size: 0 as size_t,
                    capacity: 0 as size_t,
                    items: ::core::ptr::null_mut::<Object>(),
                };
            }
        } else if has_buf {
            let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
            if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                return Array {
                    size: 0 as size_t,
                    capacity: 0 as size_t,
                    items: ::core::ptr::null_mut::<Object>(),
                };
            }
            if patterns.size == patterns.capacity {
                patterns.capacity = if patterns.capacity << 1 as ::core::ffi::c_int
                    > ::core::mem::size_of::<[Object; 16]>()
                        .wrapping_div(::core::mem::size_of::<Object>())
                        .wrapping_div(
                            (::core::mem::size_of::<[Object; 16]>()
                                .wrapping_rem(::core::mem::size_of::<Object>())
                                == 0) as ::core::ffi::c_int as usize,
                        ) {
                    patterns.capacity << 1 as ::core::ffi::c_int
                } else {
                    ::core::mem::size_of::<[Object; 16]>()
                        .wrapping_div(::core::mem::size_of::<Object>())
                        .wrapping_div(
                            (::core::mem::size_of::<[Object; 16]>()
                                .wrapping_rem(::core::mem::size_of::<Object>())
                                == 0) as ::core::ffi::c_int as size_t,
                        )
                };
                patterns.items = (if patterns.capacity
                    == ::core::mem::size_of::<[Object; 16]>()
                        .wrapping_div(::core::mem::size_of::<Object>())
                        .wrapping_div(
                            (::core::mem::size_of::<[Object; 16]>()
                                .wrapping_rem(::core::mem::size_of::<Object>())
                                == 0) as ::core::ffi::c_int as usize,
                        ) {
                    if patterns.items == &raw mut patterns.init_array as *mut Object {
                        patterns.items as *mut ::core::ffi::c_void
                    } else {
                        _memcpy_free(
                            &raw mut patterns.init_array as *mut Object as *mut ::core::ffi::c_void,
                            patterns.items as *mut ::core::ffi::c_void,
                            patterns.size.wrapping_mul(::core::mem::size_of::<Object>()),
                        )
                    }
                } else {
                    if patterns.items == &raw mut patterns.init_array as *mut Object {
                        memcpy(
                            xmalloc(
                                patterns
                                    .capacity
                                    .wrapping_mul(::core::mem::size_of::<Object>()),
                            ),
                            patterns.items as *const ::core::ffi::c_void,
                            patterns.size.wrapping_mul(::core::mem::size_of::<Object>()),
                        )
                    } else {
                        xrealloc(
                            patterns.items as *mut ::core::ffi::c_void,
                            patterns
                                .capacity
                                .wrapping_mul(::core::mem::size_of::<Object>()),
                        )
                    }
                }) as *mut Object;
            } else {
            };
            let c2rust_fresh21 = patterns.size;
            patterns.size = patterns.size.wrapping_add(1);
            *patterns.items.offset(c2rust_fresh21 as isize) = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: arena_printf(
                        arena,
                        b"<buffer=%d>\0".as_ptr() as *const ::core::ffi::c_char,
                        (*b).handle,
                    ),
                },
            };
        }
        if patterns.size == 0 as size_t && !fallback.is_null() {
            if patterns.size == patterns.capacity {
                patterns.capacity = if patterns.capacity << 1 as ::core::ffi::c_int
                    > ::core::mem::size_of::<[Object; 16]>()
                        .wrapping_div(::core::mem::size_of::<Object>())
                        .wrapping_div(
                            (::core::mem::size_of::<[Object; 16]>()
                                .wrapping_rem(::core::mem::size_of::<Object>())
                                == 0) as ::core::ffi::c_int as usize,
                        ) {
                    patterns.capacity << 1 as ::core::ffi::c_int
                } else {
                    ::core::mem::size_of::<[Object; 16]>()
                        .wrapping_div(::core::mem::size_of::<Object>())
                        .wrapping_div(
                            (::core::mem::size_of::<[Object; 16]>()
                                .wrapping_rem(::core::mem::size_of::<Object>())
                                == 0) as ::core::ffi::c_int as size_t,
                        )
                };
                patterns.items = (if patterns.capacity
                    == ::core::mem::size_of::<[Object; 16]>()
                        .wrapping_div(::core::mem::size_of::<Object>())
                        .wrapping_div(
                            (::core::mem::size_of::<[Object; 16]>()
                                .wrapping_rem(::core::mem::size_of::<Object>())
                                == 0) as ::core::ffi::c_int as usize,
                        ) {
                    if patterns.items == &raw mut patterns.init_array as *mut Object {
                        patterns.items as *mut ::core::ffi::c_void
                    } else {
                        _memcpy_free(
                            &raw mut patterns.init_array as *mut Object as *mut ::core::ffi::c_void,
                            patterns.items as *mut ::core::ffi::c_void,
                            patterns.size.wrapping_mul(::core::mem::size_of::<Object>()),
                        )
                    }
                } else {
                    if patterns.items == &raw mut patterns.init_array as *mut Object {
                        memcpy(
                            xmalloc(
                                patterns
                                    .capacity
                                    .wrapping_mul(::core::mem::size_of::<Object>()),
                            ),
                            patterns.items as *const ::core::ffi::c_void,
                            patterns.size.wrapping_mul(::core::mem::size_of::<Object>()),
                        )
                    } else {
                        xrealloc(
                            patterns.items as *mut ::core::ffi::c_void,
                            patterns
                                .capacity
                                .wrapping_mul(::core::mem::size_of::<Object>()),
                        )
                    }
                }) as *mut Object;
            } else {
            };
            let c2rust_fresh22 = patterns.size;
            patterns.size = patterns.size.wrapping_add(1);
            *patterns.items.offset(c2rust_fresh22 as isize) = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string(fallback),
                },
            };
        }
        return arena_take_arraybuilder(arena, &raw mut patterns);
    }
}
