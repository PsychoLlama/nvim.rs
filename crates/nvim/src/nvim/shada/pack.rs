//! Turning a [`ShadaEntry`] into msgpack.
//!
//! One entry becomes one `<type> <timestamp> <length> <payload>` quadruple,
//! where the payload is a map for everything but the plain-string entries.
//! `shada_pack_entry` writes them all; the rest of this file is the buffer
//! plumbing under it, which keeps at least [`SHADA_MPACK_FREE_SPACE`] bytes
//! free and flushes to the file when it runs out.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn shada_check_buffer(mut packer: *mut PackerBuffer) {
    unsafe {
        if mpack_remaining(&*packer) < SHADA_MPACK_FREE_SPACE as size_t {
            (*packer).packer_flush.expect("non-null function pointer")(packer);
        }
    }
}

pub(crate) unsafe extern "C" fn additional_data_len(mut src: *mut AdditionalData) -> uint32_t {
    unsafe {
        return if !src.is_null() {
            (*src).nitems
        } else {
            0 as uint32_t
        };
    }
}

pub(crate) unsafe extern "C" fn dump_additional_data(
    mut src: *mut AdditionalData,
    mut sbuf: *mut PackerBuffer,
) {
    unsafe {
        if !src.is_null() {
            mpack_raw(
                &raw mut (*src).data as *mut ::core::ffi::c_char,
                (*src).nbytes as size_t,
                &mut *sbuf,
            );
        }
    }
}

pub(crate) unsafe extern "C" fn shada_pack_entry(
    packer: *mut PackerBuffer,
    mut entry: ShadaEntry,
    max_kbyte: size_t,
) -> ShaDaWriteResult {
    unsafe {
        let mut packed: String_0 = String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        };
        let mut ret: ShaDaWriteResult = kSDWriteFailed;
        let mut sbuf: PackerBuffer = packer_string_buffer();
        shada_check_buffer(&mut sbuf);
        '_shada_pack_entry_error: {
            match entry.type_0 as ::core::ffi::c_int {
                0 => {
                    abort();
                }
                -1 => {
                    mpack_raw(
                        entry.data.unknown_item.contents,
                        entry.data.unknown_item.size,
                        &mut sbuf,
                    );
                }
                4 => {
                    let is_hist_search: bool = entry.data.history_item.histtype
                        as ::core::ffi::c_int
                        == HIST_SEARCH as ::core::ffi::c_int;
                    let mut arr_size: uint32_t = (2 as uint32_t)
                        .wrapping_add(is_hist_search as uint32_t)
                        .wrapping_add(additional_data_len(entry.additional_data));
                    mpack_array(&mut sbuf.ptr, arr_size);
                    mpack_uint(&mut sbuf.ptr, entry.data.history_item.histtype as uint32_t);
                    mpack_bin(cstr_as_string(entry.data.history_item.string), &mut sbuf);
                    if is_hist_search {
                        mpack_uint(
                            &mut sbuf.ptr,
                            entry.data.history_item.sep as uint8_t as uint32_t,
                        );
                    }
                    dump_additional_data(entry.additional_data, &mut sbuf);
                }
                6 => {
                    let mut is_blob: bool = entry.data.global_var.value.v_type
                        as ::core::ffi::c_uint
                        == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint;
                    let mut arr_size_0: uint32_t = ((2 as ::core::ffi::c_int
                        + (if is_blob as ::core::ffi::c_int != 0 {
                            1 as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        })) as uint32_t)
                        .wrapping_add(additional_data_len(entry.additional_data));
                    mpack_array(&mut sbuf.ptr, arr_size_0);
                    let varname: String_0 = cstr_as_string(entry.data.global_var.name);
                    mpack_bin(varname, &mut sbuf);
                    let mut vardesc: [::core::ffi::c_char; 256] = ::core::mem::transmute::<
                    [u8; 256],
                    [::core::ffi::c_char; 256],
                >(
                    *b"variable g:\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                );
                    memcpy(
                        (&raw mut vardesc as *mut ::core::ffi::c_char).offset(
                            ::core::mem::size_of::<[::core::ffi::c_char; 12]>()
                                .wrapping_sub(1 as usize) as isize,
                        ) as *mut ::core::ffi::c_void,
                        varname.data as *const ::core::ffi::c_void,
                        varname.size.wrapping_add(1 as size_t),
                    );
                    if encode_vim_to_msgpack(
                        &mut sbuf,
                        &raw mut entry.data.global_var.value,
                        &raw mut vardesc as *mut ::core::ffi::c_char,
                    ) == FAIL
                    {
                        ret = kSDWriteIgnError;
                        semsg(
                            gettext(b"E574: Failed to write variable %s\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            entry.data.global_var.name,
                        );
                        break '_shada_pack_entry_error;
                    } else {
                        if is_blob {
                            mpack_check_buffer(&mut sbuf);
                            mpack_integer(
                                &mut sbuf.ptr,
                                VAR_TYPE_BLOB as ::core::ffi::c_int as Integer,
                            );
                        }
                        dump_additional_data(entry.additional_data, &mut sbuf);
                    }
                }
                3 => {
                    let mut arr_size_1: uint32_t =
                        (1 as uint32_t).wrapping_add(additional_data_len(entry.additional_data));
                    mpack_array(&mut sbuf.ptr, arr_size_1);
                    mpack_bin(cstr_as_string(entry.data.sub_string.sub), &mut sbuf);
                    dump_additional_data(entry.additional_data, &mut sbuf);
                }
                2 => {
                    let mut entry_map_size: uint32_t = (1 as uint32_t)
                        .wrapping_add(
                            !((*sd_default_values.ptr())[entry.type_0 as usize]
                                .data
                                .search_pattern
                                .magic as ::core::ffi::c_int
                                == entry.data.search_pattern.magic as ::core::ffi::c_int)
                                as ::core::ffi::c_int as uint32_t,
                        )
                        .wrapping_add(
                            !((*sd_default_values.ptr())[entry.type_0 as usize]
                                .data
                                .search_pattern
                                .is_last_used as ::core::ffi::c_int
                                == entry.data.search_pattern.is_last_used as ::core::ffi::c_int)
                                as ::core::ffi::c_int as uint32_t,
                        )
                        .wrapping_add(
                            !((*sd_default_values.ptr())[entry.type_0 as usize]
                                .data
                                .search_pattern
                                .smartcase as ::core::ffi::c_int
                                == entry.data.search_pattern.smartcase as ::core::ffi::c_int)
                                as ::core::ffi::c_int as uint32_t,
                        )
                        .wrapping_add(
                            !((*sd_default_values.ptr())[entry.type_0 as usize]
                                .data
                                .search_pattern
                                .has_line_offset
                                as ::core::ffi::c_int
                                == entry.data.search_pattern.has_line_offset as ::core::ffi::c_int)
                                as ::core::ffi::c_int as uint32_t,
                        )
                        .wrapping_add(
                            !((*sd_default_values.ptr())[entry.type_0 as usize]
                                .data
                                .search_pattern
                                .place_cursor_at_end
                                as ::core::ffi::c_int
                                == entry.data.search_pattern.place_cursor_at_end
                                    as ::core::ffi::c_int)
                                as ::core::ffi::c_int as uint32_t,
                        )
                        .wrapping_add(
                            !((*sd_default_values.ptr())[entry.type_0 as usize]
                                .data
                                .search_pattern
                                .is_substitute_pattern
                                as ::core::ffi::c_int
                                == entry.data.search_pattern.is_substitute_pattern
                                    as ::core::ffi::c_int)
                                as ::core::ffi::c_int as uint32_t,
                        )
                        .wrapping_add(
                            !((*sd_default_values.ptr())[entry.type_0 as usize]
                                .data
                                .search_pattern
                                .highlighted as ::core::ffi::c_int
                                == entry.data.search_pattern.highlighted as ::core::ffi::c_int)
                                as ::core::ffi::c_int as uint32_t,
                        )
                        .wrapping_add(
                            !((*sd_default_values.ptr())[entry.type_0 as usize]
                                .data
                                .search_pattern
                                .offset
                                == entry.data.search_pattern.offset)
                                as ::core::ffi::c_int as uint32_t,
                        )
                        .wrapping_add(
                            !((*sd_default_values.ptr())[entry.type_0 as usize]
                                .data
                                .search_pattern
                                .search_backward
                                as ::core::ffi::c_int
                                == entry.data.search_pattern.search_backward as ::core::ffi::c_int)
                                as ::core::ffi::c_int as uint32_t,
                        )
                        .wrapping_add(additional_data_len(entry.additional_data));
                    mpack_map(&mut sbuf.ptr, entry_map_size);
                    mpack_str(
                        String_0 {
                            data: b"sp\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                .wrapping_sub(1 as size_t),
                        },
                        &mut sbuf,
                    );
                    mpack_bin(entry.data.search_pattern.pat, &mut sbuf);
                    if !((*sd_default_values.ptr())[entry.type_0 as usize]
                        .data
                        .search_pattern
                        .magic as ::core::ffi::c_int
                        == entry.data.search_pattern.magic as ::core::ffi::c_int)
                    {
                        mpack_str(
                            String_0 {
                                data: b"sm\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                    .wrapping_sub(1 as size_t),
                            },
                            &mut sbuf,
                        );
                        mpack_bool(
                            &mut sbuf.ptr,
                            !(*sd_default_values.ptr())[entry.type_0 as usize]
                                .data
                                .search_pattern
                                .magic,
                        );
                    }
                    if !((*sd_default_values.ptr())[entry.type_0 as usize]
                        .data
                        .search_pattern
                        .is_last_used as ::core::ffi::c_int
                        == entry.data.search_pattern.is_last_used as ::core::ffi::c_int)
                    {
                        mpack_str(
                            String_0 {
                                data: b"su\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                    .wrapping_sub(1 as size_t),
                            },
                            &mut sbuf,
                        );
                        mpack_bool(
                            &mut sbuf.ptr,
                            !(*sd_default_values.ptr())[entry.type_0 as usize]
                                .data
                                .search_pattern
                                .is_last_used,
                        );
                    }
                    if !((*sd_default_values.ptr())[entry.type_0 as usize]
                        .data
                        .search_pattern
                        .smartcase as ::core::ffi::c_int
                        == entry.data.search_pattern.smartcase as ::core::ffi::c_int)
                    {
                        mpack_str(
                            String_0 {
                                data: b"sc\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                    .wrapping_sub(1 as size_t),
                            },
                            &mut sbuf,
                        );
                        mpack_bool(
                            &mut sbuf.ptr,
                            !(*sd_default_values.ptr())[entry.type_0 as usize]
                                .data
                                .search_pattern
                                .smartcase,
                        );
                    }
                    if !((*sd_default_values.ptr())[entry.type_0 as usize]
                        .data
                        .search_pattern
                        .has_line_offset as ::core::ffi::c_int
                        == entry.data.search_pattern.has_line_offset as ::core::ffi::c_int)
                    {
                        mpack_str(
                            String_0 {
                                data: b"sl\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                    .wrapping_sub(1 as size_t),
                            },
                            &mut sbuf,
                        );
                        mpack_bool(
                            &mut sbuf.ptr,
                            !(*sd_default_values.ptr())[entry.type_0 as usize]
                                .data
                                .search_pattern
                                .has_line_offset,
                        );
                    }
                    if !((*sd_default_values.ptr())[entry.type_0 as usize]
                        .data
                        .search_pattern
                        .place_cursor_at_end as ::core::ffi::c_int
                        == entry.data.search_pattern.place_cursor_at_end as ::core::ffi::c_int)
                    {
                        mpack_str(
                            String_0 {
                                data: b"se\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                    .wrapping_sub(1 as size_t),
                            },
                            &mut sbuf,
                        );
                        mpack_bool(
                            &mut sbuf.ptr,
                            !(*sd_default_values.ptr())[entry.type_0 as usize]
                                .data
                                .search_pattern
                                .place_cursor_at_end,
                        );
                    }
                    if !((*sd_default_values.ptr())[entry.type_0 as usize]
                        .data
                        .search_pattern
                        .is_substitute_pattern as ::core::ffi::c_int
                        == entry.data.search_pattern.is_substitute_pattern as ::core::ffi::c_int)
                    {
                        mpack_str(
                            String_0 {
                                data: b"ss\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                    .wrapping_sub(1 as size_t),
                            },
                            &mut sbuf,
                        );
                        mpack_bool(
                            &mut sbuf.ptr,
                            !(*sd_default_values.ptr())[entry.type_0 as usize]
                                .data
                                .search_pattern
                                .is_substitute_pattern,
                        );
                    }
                    if !((*sd_default_values.ptr())[entry.type_0 as usize]
                        .data
                        .search_pattern
                        .highlighted as ::core::ffi::c_int
                        == entry.data.search_pattern.highlighted as ::core::ffi::c_int)
                    {
                        mpack_str(
                            String_0 {
                                data: b"sh\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                    .wrapping_sub(1 as size_t),
                            },
                            &mut sbuf,
                        );
                        mpack_bool(
                            &mut sbuf.ptr,
                            !(*sd_default_values.ptr())[entry.type_0 as usize]
                                .data
                                .search_pattern
                                .highlighted,
                        );
                    }
                    if !((*sd_default_values.ptr())[entry.type_0 as usize]
                        .data
                        .search_pattern
                        .search_backward as ::core::ffi::c_int
                        == entry.data.search_pattern.search_backward as ::core::ffi::c_int)
                    {
                        mpack_str(
                            String_0 {
                                data: b"sb\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                    .wrapping_sub(1 as size_t),
                            },
                            &mut sbuf,
                        );
                        mpack_bool(
                            &mut sbuf.ptr,
                            !(*sd_default_values.ptr())[entry.type_0 as usize]
                                .data
                                .search_pattern
                                .search_backward,
                        );
                    }
                    if !((*sd_default_values.ptr())[entry.type_0 as usize]
                        .data
                        .search_pattern
                        .offset
                        == entry.data.search_pattern.offset)
                    {
                        mpack_str(
                            String_0 {
                                data: b"so\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                    .wrapping_sub(1 as size_t),
                            },
                            &mut sbuf,
                        );
                        mpack_integer(&mut sbuf.ptr, entry.data.search_pattern.offset);
                    }
                    dump_additional_data(entry.additional_data, &mut sbuf);
                }
                11 | 7 | 10 | 8 => {
                    let mut entry_map_size_0: size_t = (1 as uint32_t)
                        .wrapping_add(
                            !((*sd_default_values.ptr())[entry.type_0 as usize]
                                .data
                                .filemark
                                .mark
                                .lnum
                                == entry.data.filemark.mark.lnum)
                                as ::core::ffi::c_int as uint32_t,
                        )
                        .wrapping_add(
                            !((*sd_default_values.ptr())[entry.type_0 as usize]
                                .data
                                .filemark
                                .mark
                                .col
                                == entry.data.filemark.mark.col)
                                as ::core::ffi::c_int as uint32_t,
                        )
                        .wrapping_add(
                            !((*sd_default_values.ptr())[entry.type_0 as usize]
                                .data
                                .filemark
                                .name as ::core::ffi::c_int
                                == entry.data.filemark.name as ::core::ffi::c_int)
                                as ::core::ffi::c_int as uint32_t,
                        )
                        .wrapping_add(additional_data_len(entry.additional_data))
                        as size_t;
                    mpack_map(&mut sbuf.ptr, entry_map_size_0 as uint32_t);
                    mpack_str(
                        String_0 {
                            data: b"f\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                                .wrapping_sub(1 as size_t),
                        },
                        &mut sbuf,
                    );
                    mpack_bin(cstr_as_string(entry.data.filemark.fname), &mut sbuf);
                    if !((*sd_default_values.ptr())[entry.type_0 as usize]
                        .data
                        .filemark
                        .mark
                        .lnum
                        == entry.data.filemark.mark.lnum)
                    {
                        mpack_str(
                            String_0 {
                                data: b"l\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                size: ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                                    .wrapping_sub(1 as size_t),
                            },
                            &mut sbuf,
                        );
                        mpack_integer(&mut sbuf.ptr, entry.data.filemark.mark.lnum as Integer);
                    }
                    if !((*sd_default_values.ptr())[entry.type_0 as usize]
                        .data
                        .filemark
                        .mark
                        .col
                        == entry.data.filemark.mark.col)
                    {
                        mpack_str(
                            String_0 {
                                data: b"c\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                size: ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                                    .wrapping_sub(1 as size_t),
                            },
                            &mut sbuf,
                        );
                        mpack_integer(&mut sbuf.ptr, entry.data.filemark.mark.col as Integer);
                    }
                    '_c2rust_label: {
                        if (if entry.type_0 as ::core::ffi::c_int
                            == kSDItemJump as ::core::ffi::c_int
                            || entry.type_0 as ::core::ffi::c_int
                                == kSDItemChange as ::core::ffi::c_int
                        {
                            ((*sd_default_values.ptr())[entry.type_0 as usize]
                                .data
                                .filemark
                                .name as ::core::ffi::c_int
                                == entry.data.filemark.name as ::core::ffi::c_int)
                                as ::core::ffi::c_int
                        } else {
                            1 as ::core::ffi::c_int
                        }) != 0
                        {
                        } else {
                            __assert_fail(
                            b"entry.type == kSDItemJump || entry.type == kSDItemChange ? CHECK_DEFAULT(entry, filemark.name) : true\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/shada.rs\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            1471 as ::core::ffi::c_uint,
                            b"ShaDaWriteResult shada_pack_entry(PackerBuffer *const, ShadaEntry, const size_t)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                        }
                    };
                    if !((*sd_default_values.ptr())[entry.type_0 as usize]
                        .data
                        .filemark
                        .name as ::core::ffi::c_int
                        == entry.data.filemark.name as ::core::ffi::c_int)
                    {
                        mpack_str(
                            String_0 {
                                data: b"n\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                size: ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                                    .wrapping_sub(1 as size_t),
                            },
                            &mut sbuf,
                        );
                        mpack_uint(
                            &mut sbuf.ptr,
                            entry.data.filemark.name as uint8_t as uint32_t,
                        );
                    }
                    dump_additional_data(entry.additional_data, &mut sbuf);
                }
                5 => {
                    let mut entry_map_size_1: uint32_t = (2 as uint32_t)
                        .wrapping_add(
                            !((*sd_default_values.ptr())[entry.type_0 as usize]
                                .data
                                .reg
                                .type_0 as ::core::ffi::c_int
                                == entry.data.reg.type_0 as ::core::ffi::c_int)
                                as ::core::ffi::c_int as uint32_t,
                        )
                        .wrapping_add(
                            !((*sd_default_values.ptr())[entry.type_0 as usize]
                                .data
                                .reg
                                .width
                                == entry.data.reg.width)
                                as ::core::ffi::c_int as uint32_t,
                        )
                        .wrapping_add(
                            !((*sd_default_values.ptr())[entry.type_0 as usize]
                                .data
                                .reg
                                .is_unnamed as ::core::ffi::c_int
                                == entry.data.reg.is_unnamed as ::core::ffi::c_int)
                                as ::core::ffi::c_int as uint32_t,
                        )
                        .wrapping_add(additional_data_len(entry.additional_data));
                    mpack_map(&mut sbuf.ptr, entry_map_size_1);
                    mpack_str(
                        String_0 {
                            data: b"rc\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                .wrapping_sub(1 as size_t),
                        },
                        &mut sbuf,
                    );
                    mpack_array(&mut sbuf.ptr, entry.data.reg.contents_size as uint32_t);
                    let mut i: size_t = 0 as size_t;
                    while i < entry.data.reg.contents_size {
                        mpack_bin(*entry.data.reg.contents.offset(i as isize), &mut sbuf);
                        i = i.wrapping_add(1);
                    }
                    mpack_str(
                        String_0 {
                            data: b"n\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                                .wrapping_sub(1 as size_t),
                        },
                        &mut sbuf,
                    );
                    mpack_uint(&mut sbuf.ptr, entry.data.reg.name as uint8_t as uint32_t);
                    if !((*sd_default_values.ptr())[entry.type_0 as usize]
                        .data
                        .reg
                        .type_0 as ::core::ffi::c_int
                        == entry.data.reg.type_0 as ::core::ffi::c_int)
                    {
                        mpack_str(
                            String_0 {
                                data: b"rt\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                    .wrapping_sub(1 as size_t),
                            },
                            &mut sbuf,
                        );
                        mpack_uint(&mut sbuf.ptr, entry.data.reg.type_0 as uint8_t as uint32_t);
                    }
                    if !((*sd_default_values.ptr())[entry.type_0 as usize]
                        .data
                        .reg
                        .width
                        == entry.data.reg.width)
                    {
                        mpack_str(
                            String_0 {
                                data: b"rw\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                    .wrapping_sub(1 as size_t),
                            },
                            &mut sbuf,
                        );
                        mpack_uint64(&mut sbuf.ptr, entry.data.reg.width as uint64_t);
                    }
                    if !((*sd_default_values.ptr())[entry.type_0 as usize]
                        .data
                        .reg
                        .is_unnamed as ::core::ffi::c_int
                        == entry.data.reg.is_unnamed as ::core::ffi::c_int)
                    {
                        mpack_str(
                            String_0 {
                                data: b"ru\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                    .wrapping_sub(1 as size_t),
                            },
                            &mut sbuf,
                        );
                        mpack_bool(&mut sbuf.ptr, entry.data.reg.is_unnamed);
                    }
                    dump_additional_data(entry.additional_data, &mut sbuf);
                }
                9 => {
                    mpack_array(&mut sbuf.ptr, entry.data.buffer_list.size as uint32_t);
                    let mut i_0: size_t = 0 as size_t;
                    while i_0 < entry.data.buffer_list.size {
                        let mut entry_map_size_2: size_t = (1 as size_t)
                            .wrapping_add(
                                ((*entry.data.buffer_list.buffers.offset(i_0 as isize))
                                    .pos
                                    .lnum
                                    != (*default_pos.ptr()).lnum)
                                    as ::core::ffi::c_int as size_t,
                            )
                            .wrapping_add(
                                ((*entry.data.buffer_list.buffers.offset(i_0 as isize))
                                    .pos
                                    .col
                                    != (*default_pos.ptr()).col)
                                    as ::core::ffi::c_int as size_t,
                            )
                            .wrapping_add(additional_data_len(
                                (*entry.data.buffer_list.buffers.offset(i_0 as isize))
                                    .additional_data,
                            ) as size_t);
                        mpack_map(&mut sbuf.ptr, entry_map_size_2 as uint32_t);
                        mpack_str(
                            String_0 {
                                data: b"f\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                size: ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                                    .wrapping_sub(1 as size_t),
                            },
                            &mut sbuf,
                        );
                        mpack_bin(
                            cstr_as_string(
                                (*entry.data.buffer_list.buffers.offset(i_0 as isize)).fname,
                            ),
                            &mut sbuf,
                        );
                        if (*entry.data.buffer_list.buffers.offset(i_0 as isize))
                            .pos
                            .lnum
                            != 1 as linenr_T
                        {
                            mpack_str(
                                String_0 {
                                    data: b"l\0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char,
                                    size: ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                                        .wrapping_sub(1 as size_t),
                                },
                                &mut sbuf,
                            );
                            mpack_uint64(
                                &mut sbuf.ptr,
                                (*entry.data.buffer_list.buffers.offset(i_0 as isize))
                                    .pos
                                    .lnum as uint64_t,
                            );
                        }
                        if (*entry.data.buffer_list.buffers.offset(i_0 as isize))
                            .pos
                            .col
                            != 0 as ::core::ffi::c_int
                        {
                            mpack_str(
                                String_0 {
                                    data: b"c\0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char,
                                    size: ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                                        .wrapping_sub(1 as size_t),
                                },
                                &mut sbuf,
                            );
                            mpack_uint64(
                                &mut sbuf.ptr,
                                (*entry.data.buffer_list.buffers.offset(i_0 as isize))
                                    .pos
                                    .col as uint64_t,
                            );
                        }
                        dump_additional_data(
                            (*entry.data.buffer_list.buffers.offset(i_0 as isize)).additional_data,
                            &mut sbuf,
                        );
                        i_0 = i_0.wrapping_add(1);
                    }
                }
                1 => {
                    mpack_map(&mut sbuf.ptr, entry.data.header.size as uint32_t);
                    let mut i_1: size_t = 0 as size_t;
                    while i_1 < entry.data.header.size {
                        mpack_str(
                            (*entry.data.header.items.offset(i_1 as isize)).key,
                            &mut sbuf,
                        );
                        let obj: Object = (*entry.data.header.items.offset(i_1 as isize)).value;
                        match obj.type_0 as ::core::ffi::c_uint {
                            4 => {
                                mpack_bin(obj.data.string, &mut sbuf);
                            }
                            2 => {
                                mpack_integer(&mut sbuf.ptr, obj.data.integer);
                            }
                            _ => {
                                abort();
                            }
                        }
                        i_1 = i_1.wrapping_add(1);
                    }
                }
                _ => {}
            }
            packed = packer_take_string(&mut sbuf);
            if max_kbyte == 0 || packed.size <= max_kbyte.wrapping_mul(1024 as size_t) {
                shada_check_buffer(packer);
                if entry.type_0 as ::core::ffi::c_int == kSDItemUnknown as ::core::ffi::c_int {
                    mpack_uint64(&mut (*packer).ptr, entry.data.unknown_item.type_0);
                } else {
                    mpack_uint64(&mut (*packer).ptr, entry.type_0 as uint64_t);
                }
                mpack_uint64(&mut (*packer).ptr, entry.timestamp);
                if packed.size > 0 as size_t {
                    mpack_uint64(&mut (*packer).ptr, packed.size as uint64_t);
                    mpack_raw(packed.data, packed.size, &mut *packer);
                }
                if (*packer).anyint != 0 as int64_t {
                    break '_shada_pack_entry_error;
                }
            }
            ret = kSDWriteSuccessful;
        }
        xfree(sbuf.startptr as *mut ::core::ffi::c_void);
        return ret;
    }
}

#[inline(always)]
pub(crate) unsafe extern "C" fn shada_pack_pfreed_entry(
    packer: *mut PackerBuffer,
    mut entry: ShadaEntry,
    max_kbyte: size_t,
) -> ShaDaWriteResult {
    unsafe {
        let mut ret: ShaDaWriteResult = shada_pack_entry(packer, entry, max_kbyte);
        shada_free_shada_entry(&raw mut entry);
        return ret;
    }
}

pub(crate) unsafe extern "C" fn packer_buffer_for_file(
    mut file: *mut FileDescriptor,
) -> PackerBuffer {
    unsafe {
        if file_space(file) < SHADA_MPACK_FREE_SPACE as size_t {
            file_flush(file);
        }
        return packer_buffer_t {
            startptr: (*file).buffer,
            ptr: (*file).write_pos,
            endptr: (*file).buffer.offset(ARENA_BLOCK_SIZE as isize),
            anydata: file as *mut ::core::ffi::c_void,
            anyint: 0 as int64_t,
            packer_flush: Some(flush_file_buffer as unsafe extern "C" fn(*mut PackerBuffer) -> ()),
        };
    }
}

pub(crate) unsafe extern "C" fn flush_file_buffer(mut buffer: *mut PackerBuffer) {
    unsafe {
        let mut fd: *mut FileDescriptor = (*buffer).anydata as *mut FileDescriptor;
        (*fd).write_pos = (*buffer).ptr;
        (*buffer).anyint = file_flush(fd) as int64_t;
        (*buffer).ptr = (*fd).write_pos;
    }
}
