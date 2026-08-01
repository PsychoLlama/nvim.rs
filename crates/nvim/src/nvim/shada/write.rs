//! Writing a ShaDa file.
//!
//! `shada_write` is the whole of it: work out what the `'shada'` option
//! allows, merge in what the old file held (see `merge`), collect the
//! editor's state (see `collect`), and pack the result (see `pack`) in the
//! order the format wants.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn shada_write(
    sd_writer: *mut FileDescriptor,
    sd_reader: *mut FileDescriptor,
) -> ShaDaWriteResult {
    unsafe {
        let mut file_markss_size: size_t = 0;
        let mut all_file_markss: *mut *mut FileMarks = ::core::ptr::null_mut::<*mut FileMarks>();
        let mut cur_file_marks: *mut *mut FileMarks = ::core::ptr::null_mut::<*mut FileMarks>();
        let mut val_0: ptr_t = ::core::ptr::null_mut::<::core::ffi::c_void>();
        let mut file_markss_to_dump: size_t = 0;
        let mut ret: ShaDaWriteResult = kSDWriteSuccessful;
        let mut max_kbyte_i: ::core::ffi::c_int = get_shada_parameter('s' as ::core::ffi::c_int);
        if max_kbyte_i < 0 as ::core::ffi::c_int {
            max_kbyte_i = 10 as ::core::ffi::c_int;
        }
        if max_kbyte_i == 0 as ::core::ffi::c_int {
            return ret;
        }
        let wms: *mut WriteMergerState =
            xcalloc(1 as size_t, ::core::mem::size_of::<WriteMergerState>())
                as *mut WriteMergerState;
        let mut dump_one_history: [bool; 5] = [false; 5];
        let dump_global_vars: bool = !find_shada_parameter('!' as ::core::ffi::c_int).is_null();
        let mut max_reg_lines: ::core::ffi::c_int = get_shada_parameter('<' as ::core::ffi::c_int);
        if max_reg_lines < 0 as ::core::ffi::c_int {
            max_reg_lines = get_shada_parameter('"' as ::core::ffi::c_int);
        }
        let dump_registers: bool = max_reg_lines != 0 as ::core::ffi::c_int;
        let mut removable_bufs: Set_ptr_t = Set_ptr_t {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<ptr_t>(),
        };
        let max_kbyte: size_t = max_kbyte_i as size_t;
        let num_marked_files: size_t = get_shada_parameter('\'' as ::core::ffi::c_int) as size_t;
        let dump_global_marks: bool =
            get_shada_parameter('f' as ::core::ffi::c_int) != 0 as ::core::ffi::c_int;
        let mut dump_history: bool = false_0 != 0;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < HIST_COUNT as ::core::ffi::c_int {
            let mut num_saved: ::core::ffi::c_int = get_shada_parameter(hist_type2char(i));
            if num_saved == -1 as ::core::ffi::c_int {
                num_saved = p_hi.get() as ::core::ffi::c_int;
            }
            if num_saved > 0 as ::core::ffi::c_int {
                dump_history = true_0 != 0;
                dump_one_history[i as usize] = true_0 != 0;
                hms_init(
                    (&raw mut (*wms).hms as *mut HistoryMergerState).offset(i as isize),
                    i as uint8_t,
                    num_saved as size_t,
                    !sd_reader.is_null(),
                    false_0 != 0,
                );
            } else {
                dump_one_history[i as usize] = false_0 != 0;
            }
            i += 1;
        }
        let srni_flags: ::core::ffi::c_uint = (kSDReadUndisableableData as ::core::ffi::c_int
            | kSDReadUnknown as ::core::ffi::c_int
            | (if dump_history as ::core::ffi::c_int != 0 {
                kSDReadHistory as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            })
            | (if dump_registers as ::core::ffi::c_int != 0 {
                kSDReadRegisters as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            })
            | (if dump_global_vars as ::core::ffi::c_int != 0 {
                kSDReadVariables as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            })
            | (if dump_global_marks as ::core::ffi::c_int != 0 {
                kSDReadGlobalMarks as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            })
            | (if num_marked_files != 0 {
                kSDReadLocalMarks as ::core::ffi::c_int | kSDReadChanges as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            })) as ::core::ffi::c_uint;
        let mut packer: PackerBuffer = packer_buffer_for_file(sd_writer);
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut wp: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() {
                set_last_cursor(wp);
                wp = (*wp).w_next;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        find_removable_bufs(&raw mut removable_bufs);
        '_shada_write_exit: {
            let mut c2rust_lvalue: [KeyValuePair; 5] = [
                key_value_pair {
                    key: String_0 {
                        data: b"generator\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        size: ::core::mem::size_of::<[::core::ffi::c_char; 10]>()
                            .wrapping_sub(1 as size_t),
                    },
                    value: object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed_1 {
                            string: String_0 {
                                data: b"nvim\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                size: ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                                    .wrapping_sub(1 as size_t),
                            },
                        },
                    },
                },
                key_value_pair {
                    key: String_0 {
                        data: b"version\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        size: ::core::mem::size_of::<[::core::ffi::c_char; 8]>()
                            .wrapping_sub(1 as size_t),
                    },
                    value: object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed_1 {
                            string: cstr_as_string(longVersion.get()),
                        },
                    },
                },
                key_value_pair {
                    key: String_0 {
                        data: b"max_kbyte\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        size: ::core::mem::size_of::<[::core::ffi::c_char; 10]>()
                            .wrapping_sub(1 as size_t),
                    },
                    value: object {
                        type_0: kObjectTypeInteger,
                        data: C2Rust_Unnamed_1 {
                            integer: max_kbyte as Integer,
                        },
                    },
                },
                key_value_pair {
                    key: String_0 {
                        data: b"pid\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        size: ::core::mem::size_of::<[::core::ffi::c_char; 4]>()
                            .wrapping_sub(1 as size_t),
                    },
                    value: object {
                        type_0: kObjectTypeInteger,
                        data: C2Rust_Unnamed_1 {
                            integer: os_get_pid(),
                        },
                    },
                },
                key_value_pair {
                    key: String_0 {
                        data: b"encoding\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        size: ::core::mem::size_of::<[::core::ffi::c_char; 9]>()
                            .wrapping_sub(1 as size_t),
                    },
                    value: object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed_1 {
                            string: cstr_as_string(p_enc.get()),
                        },
                    },
                },
            ];
            if shada_pack_entry(
                &raw mut packer,
                ShadaEntry {
                    type_0: kSDItemHeader,
                    can_free_entry: false,
                    timestamp: os_time(),
                    data: C2Rust_Unnamed_22 {
                        header: Dict {
                            size: 5 as size_t,
                            capacity: 5 as size_t,
                            items: &raw mut c2rust_lvalue as *mut KeyValuePair,
                        },
                    },
                    additional_data: ::core::ptr::null_mut::<AdditionalData>(),
                },
                0 as size_t,
            ) as ::core::ffi::c_uint
                == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                ret = kSDWriteFailed;
            } else {
                if !find_shada_parameter('%' as ::core::ffi::c_int).is_null() {
                    let mut buflist_entry: ShadaEntry = shada_get_buflist(&raw mut removable_bufs);
                    if shada_pack_entry(&raw mut packer, buflist_entry, 0 as size_t)
                        as ::core::ffi::c_uint
                        == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        xfree(buflist_entry.data.buffer_list.buffers as *mut ::core::ffi::c_void);
                        ret = kSDWriteFailed;
                        break '_shada_write_exit;
                    } else {
                        xfree(buflist_entry.data.buffer_list.buffers as *mut ::core::ffi::c_void);
                    }
                }
                's_310: {
                    if dump_global_vars {
                        let mut var_iter: *const ::core::ffi::c_void =
                            ::core::ptr::null::<::core::ffi::c_void>();
                        let cur_timestamp: Timestamp = os_time();
                        loop {
                            let mut vartv: typval_T = typval_T {
                                v_type: VAR_UNKNOWN,
                                v_lock: VAR_UNLOCKED,
                                vval: typval_vval_union { v_number: 0 },
                            };
                            let mut name: *const ::core::ffi::c_char =
                                ::core::ptr::null::<::core::ffi::c_char>();
                            var_iter = var_shada_iter(
                                var_iter,
                                &raw mut name,
                                &raw mut vartv,
                                VAR_FLAVOUR_SHADA,
                            );
                            if name.is_null() {
                                break 's_310;
                            }
                            's_190: {
                                match vartv.v_type as ::core::ffi::c_uint {
                                    3 | 9 => {
                                        tv_clear(&raw mut vartv);
                                        break 's_190;
                                    }
                                    5 => {
                                        let mut di: *mut dict_T = vartv.vval.v_dict;
                                        let mut copyID: ::core::ffi::c_int = get_copyID();
                                        if !set_ref_in_ht(
                                            &raw mut (*di).dv_hashtab,
                                            copyID,
                                            ::core::ptr::null_mut::<*mut list_stack_T>(),
                                        ) && copyID == (*di).dv_copyID
                                        {
                                            tv_clear(&raw mut vartv);
                                            break 's_190;
                                        }
                                    }
                                    4 => {
                                        let mut l: *mut list_T = vartv.vval.v_list;
                                        let mut copyID_0: ::core::ffi::c_int = get_copyID();
                                        if !set_ref_in_list_items(
                                            l,
                                            copyID_0,
                                            ::core::ptr::null_mut::<*mut ht_stack_T>(),
                                        ) && copyID_0 == (*l).lv_copyID
                                        {
                                            tv_clear(&raw mut vartv);
                                            break 's_190;
                                        }
                                    }
                                    _ => {}
                                }
                                let mut tgttv: typval_T = typval_T {
                                    v_type: VAR_UNKNOWN,
                                    v_lock: VAR_UNLOCKED,
                                    vval: typval_vval_union { v_number: 0 },
                                };
                                tv_copy(&raw mut vartv, &raw mut tgttv);
                                let mut spe_ret: ShaDaWriteResult = kSDWriteSuccessful;
                                spe_ret = shada_pack_entry(
                                    &raw mut packer,
                                    ShadaEntry {
                                        type_0: kSDItemVariable,
                                        can_free_entry: false,
                                        timestamp: cur_timestamp,
                                        data: C2Rust_Unnamed_22 {
                                            global_var: global_var {
                                                name: name as *mut ::core::ffi::c_char,
                                                value: tgttv,
                                            },
                                        },
                                        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
                                    },
                                    max_kbyte,
                                );
                                if spe_ret as ::core::ffi::c_uint
                                    == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                                {
                                    tv_clear(&raw mut vartv);
                                    tv_clear(&raw mut tgttv);
                                    ret = kSDWriteFailed;
                                    break '_shada_write_exit;
                                } else {
                                    tv_clear(&raw mut vartv);
                                    tv_clear(&raw mut tgttv);
                                    if spe_ret as ::core::ffi::c_uint
                                        == kSDWriteSuccessful as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        set_put_cstr_t(
                                            &raw mut (*wms).dumped_variables,
                                            name as cstr_t,
                                            ::core::ptr::null_mut::<*mut cstr_t>(),
                                        );
                                    }
                                }
                            }
                            if var_iter.is_null() {
                                break 's_310;
                            }
                        }
                    }
                }
                if num_marked_files > 0 as size_t {
                    (*wms).jumps_size = shada_init_jumps(
                        &raw mut (*wms).jumps as *mut ShadaEntry,
                        &raw mut removable_bufs,
                    );
                }
                if dump_one_history[HIST_SEARCH as ::core::ffi::c_int as usize]
                    as ::core::ffi::c_int
                    > 0 as ::core::ffi::c_int
                {
                    let search_highlighted: bool = !(no_hlsearch.get() as ::core::ffi::c_int != 0
                        || !find_shada_parameter('h' as ::core::ffi::c_int).is_null());
                    let search_last_used: bool = search_was_last_used();
                    add_search_pattern(
                        &raw mut (*wms).search_pattern,
                        Some(get_search_pattern as unsafe extern "C" fn(*mut SearchPattern) -> ()),
                        false_0 != 0,
                        search_last_used,
                        search_highlighted,
                    );
                    add_search_pattern(
                        &raw mut (*wms).sub_search_pattern,
                        Some(
                            get_substitute_pattern
                                as unsafe extern "C" fn(*mut SearchPattern) -> (),
                        ),
                        true_0 != 0,
                        search_last_used,
                        search_highlighted,
                    );
                    let mut sub: SubReplacementString = SubReplacementString {
                        sub: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        timestamp: 0,
                        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
                    };
                    sub_get_replacement(&raw mut sub);
                    if !sub.sub.is_null() {
                        (*wms).replacement = ShadaEntry {
                            type_0: kSDItemSubString,
                            can_free_entry: false_0 != 0,
                            timestamp: sub.timestamp,
                            data: C2Rust_Unnamed_22 {
                                sub_string: sub_string { sub: sub.sub },
                            },
                            additional_data: sub.additional_data,
                        };
                    }
                }
                if dump_global_marks {
                    let mut global_mark_iter: *const ::core::ffi::c_void =
                        ::core::ptr::null::<::core::ffi::c_void>();
                    let mut digit_mark_idx: size_t = 0 as size_t;
                    loop {
                        let mut name_0: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
                        let mut fm: xfmark_T = xfmark_T {
                            fmark: fmark_T {
                                mark: pos_T {
                                    lnum: 0,
                                    col: 0,
                                    coladd: 0,
                                },
                                fnum: 0,
                                timestamp: 0,
                                view: fmarkv_T {
                                    topline_offset: 0,
                                    skipcol: 0,
                                },
                                additional_data: ::core::ptr::null_mut::<AdditionalData>(),
                            },
                            fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        };
                        global_mark_iter =
                            mark_global_iter(global_mark_iter, &raw mut name_0, &raw mut fm);
                        if name_0 as ::core::ffi::c_int == NUL {
                            break;
                        }
                        let mut fname: *const ::core::ffi::c_char =
                            ::core::ptr::null::<::core::ffi::c_char>();
                        's_367: {
                            if fm.fmark.fnum == 0 as ::core::ffi::c_int {
                                '_c2rust_label: {
                                    if !fm.fname.is_null() {
                                    } else {
                                        __assert_fail(
                                        b"fm.fname != NULL\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        b"src/nvim/shada.rs\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                        2441 as ::core::ffi::c_uint,
                                        b"ShaDaWriteResult shada_write(FileDescriptor *const, FileDescriptor *const)\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    );
                                    }
                                };
                                if shada_removable(fm.fname) {
                                    break 's_367;
                                } else {
                                    fname = fm.fname;
                                }
                            } else {
                                let buf: *const buf_T = buflist_findnr(fm.fmark.fnum);
                                if buf.is_null()
                                    || (*buf).b_ffname.is_null()
                                    || set_has_ptr_t(&raw mut removable_bufs, buf as ptr_t)
                                        as ::core::ffi::c_int
                                        != 0
                                {
                                    break 's_367;
                                } else {
                                    fname = (*buf).b_ffname;
                                }
                            }
                            let entry: ShadaEntry = ShadaEntry {
                                type_0: kSDItemGlobalMark,
                                can_free_entry: false_0 != 0,
                                timestamp: fm.fmark.timestamp,
                                data: C2Rust_Unnamed_22 {
                                    filemark: shada_filemark {
                                        name: name_0,
                                        mark: fm.fmark.mark,
                                        fname: fname as *mut ::core::ffi::c_char,
                                    },
                                },
                                additional_data: fm.fmark.additional_data,
                            };
                            if ascii_isdigit(name_0 as ::core::ffi::c_int) {
                                let c2rust_fresh18 = digit_mark_idx;
                                digit_mark_idx = digit_mark_idx.wrapping_add(1);
                                replace_numbered_mark(wms, c2rust_fresh18, entry);
                            } else {
                                (*wms).global_marks[mark_global_index(name_0) as usize] = entry;
                            }
                        }
                        if global_mark_iter.is_null() {
                            break;
                        }
                    }
                }
                if dump_registers {
                    shada_initialize_registers(wms, max_reg_lines);
                }
                if num_marked_files > 0 as size_t {
                    let mut buf_0: *mut buf_T = firstbuf.get();
                    while !buf_0.is_null() {
                        if !ignore_buf(buf_0, &raw mut removable_bufs) {
                            let mut local_marks_iter: *const ::core::ffi::c_void =
                                ::core::ptr::null::<::core::ffi::c_void>();
                            let fname_0: *const ::core::ffi::c_char = (*buf_0).b_ffname;
                            let mut map_key: *mut cstr_t = ::core::ptr::null_mut::<cstr_t>();
                            let mut new_item: bool = false_0 != 0;
                            let mut val: *mut ptr_t = map_put_ref_cstr_t_ptr_t(
                                &raw mut (*wms).file_marks,
                                fname_0 as cstr_t,
                                &raw mut map_key,
                                &raw mut new_item,
                            );
                            if new_item {
                                *map_key = xstrdup(fname_0) as cstr_t;
                            }
                            if (*val).is_null() {
                                *val = xcalloc(1 as size_t, ::core::mem::size_of::<FileMarks>())
                                    as ptr_t;
                            }
                            let filemarks: *mut FileMarks = *val as *mut FileMarks;
                            loop {
                                let mut fm_0: fmark_T = fmark_T {
                                    mark: pos_T {
                                        lnum: 0,
                                        col: 0,
                                        coladd: 0,
                                    },
                                    fnum: 0,
                                    timestamp: 0,
                                    view: fmarkv_T {
                                        topline_offset: 0,
                                        skipcol: 0,
                                    },
                                    additional_data: ::core::ptr::null_mut::<AdditionalData>(),
                                };
                                let mut name_1: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
                                local_marks_iter = mark_buffer_iter(
                                    local_marks_iter,
                                    buf_0,
                                    &raw mut name_1,
                                    &raw mut fm_0,
                                );
                                if name_1 as ::core::ffi::c_int == NUL {
                                    break;
                                }
                                (*filemarks).marks[mark_local_index(name_1) as usize] =
                                    ShadaEntry {
                                        type_0: kSDItemLocalMark,
                                        can_free_entry: false_0 != 0,
                                        timestamp: fm_0.timestamp,
                                        data: C2Rust_Unnamed_22 {
                                            filemark: shada_filemark {
                                                name: name_1,
                                                mark: fm_0.mark,
                                                fname: fname_0 as *mut ::core::ffi::c_char,
                                            },
                                        },
                                        additional_data: fm_0.additional_data,
                                    };
                                if fm_0.timestamp > (*filemarks).greatest_timestamp {
                                    (*filemarks).greatest_timestamp = fm_0.timestamp;
                                }
                                if local_marks_iter.is_null() {
                                    break;
                                }
                            }
                            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            while i_0 < (*buf_0).b_changelistlen {
                                let fm_1: fmark_T = (*buf_0).b_changelist[i_0 as usize];
                                (*filemarks).changes[i_0 as usize] = ShadaEntry {
                                    type_0: kSDItemChange,
                                    can_free_entry: false_0 != 0,
                                    timestamp: fm_1.timestamp,
                                    data: C2Rust_Unnamed_22 {
                                        filemark: shada_filemark {
                                            name: 0,
                                            mark: fm_1.mark,
                                            fname: fname_0 as *mut ::core::ffi::c_char,
                                        },
                                    },
                                    additional_data: fm_1.additional_data,
                                };
                                if fm_1.timestamp > (*filemarks).greatest_timestamp {
                                    (*filemarks).greatest_timestamp = fm_1.timestamp;
                                }
                                i_0 += 1;
                            }
                            (*filemarks).changes_size = (*buf_0).b_changelistlen as size_t;
                        }
                        buf_0 = (*buf_0).b_next;
                    }
                }
                if !sd_reader.is_null() {
                    let srww_ret: ShaDaWriteResult =
                        shada_read_when_writing(sd_reader, srni_flags, max_kbyte, wms, &mut packer);
                    if srww_ret as ::core::ffi::c_uint
                        != kSDWriteSuccessful as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        ret = srww_ret;
                    }
                }
                if dump_global_marks as ::core::ffi::c_int != 0
                    && !ignore_buf(curbuf.get(), &raw mut removable_bufs)
                    && (*curwin.get()).w_cursor.lnum != 0 as linenr_T
                {
                    replace_numbered_mark(
                        wms,
                        0 as size_t,
                        ShadaEntry {
                            type_0: kSDItemGlobalMark,
                            can_free_entry: false_0 != 0,
                            timestamp: os_time(),
                            data: C2Rust_Unnamed_22 {
                                filemark: shada_filemark {
                                    name: '0' as ::core::ffi::c_char,
                                    mark: (*curwin.get()).w_cursor,
                                    fname: (*curbuf.get()).b_ffname,
                                },
                            },
                            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
                        },
                    );
                }
                let mut i_: size_t = 0 as size_t;
                while i_
                    < ::core::mem::size_of::<[ShadaEntry; 26]>()
                        .wrapping_div(::core::mem::size_of::<ShadaEntry>())
                        .wrapping_div(
                            (::core::mem::size_of::<[ShadaEntry; 26]>()
                                .wrapping_rem(::core::mem::size_of::<ShadaEntry>())
                                == 0) as ::core::ffi::c_int as usize,
                        )
                {
                    if (*wms).global_marks[i_ as usize].type_0 as ::core::ffi::c_int
                        != kSDItemMissing as ::core::ffi::c_int
                    {
                        if shada_pack_pfreed_entry(
                            &raw mut packer,
                            (*wms).global_marks[i_ as usize],
                            max_kbyte,
                        ) as ::core::ffi::c_uint
                            == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            ret = kSDWriteFailed;
                            break '_shada_write_exit;
                        }
                    }
                    i_ = i_.wrapping_add(1);
                }
                let mut i__0: size_t = 0 as size_t;
                while i__0
                    < ::core::mem::size_of::<[ShadaEntry; 10]>()
                        .wrapping_div(::core::mem::size_of::<ShadaEntry>())
                        .wrapping_div(
                            (::core::mem::size_of::<[ShadaEntry; 10]>()
                                .wrapping_rem(::core::mem::size_of::<ShadaEntry>())
                                == 0) as ::core::ffi::c_int as usize,
                        )
                {
                    if (*wms).numbered_marks[i__0 as usize].type_0 as ::core::ffi::c_int
                        != kSDItemMissing as ::core::ffi::c_int
                    {
                        if shada_pack_pfreed_entry(
                            &raw mut packer,
                            (*wms).numbered_marks[i__0 as usize],
                            max_kbyte,
                        ) as ::core::ffi::c_uint
                            == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            ret = kSDWriteFailed;
                            break '_shada_write_exit;
                        }
                    }
                    i__0 = i__0.wrapping_add(1);
                }
                let mut i__1: size_t = 0 as size_t;
                while i__1
                    < ::core::mem::size_of::<[ShadaEntry; 37]>()
                        .wrapping_div(::core::mem::size_of::<ShadaEntry>())
                        .wrapping_div(
                            (::core::mem::size_of::<[ShadaEntry; 37]>()
                                .wrapping_rem(::core::mem::size_of::<ShadaEntry>())
                                == 0) as ::core::ffi::c_int as usize,
                        )
                {
                    if (*wms).registers[i__1 as usize].type_0 as ::core::ffi::c_int
                        != kSDItemMissing as ::core::ffi::c_int
                    {
                        if shada_pack_pfreed_entry(
                            &raw mut packer,
                            (*wms).registers[i__1 as usize],
                            max_kbyte,
                        ) as ::core::ffi::c_uint
                            == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            ret = kSDWriteFailed;
                            break '_shada_write_exit;
                        }
                    }
                    i__1 = i__1.wrapping_add(1);
                }
                let mut i_1: size_t = 0 as size_t;
                while i_1 < (*wms).jumps_size {
                    if shada_pack_pfreed_entry(
                        &raw mut packer,
                        (*wms).jumps[i_1 as usize],
                        max_kbyte,
                    ) as ::core::ffi::c_uint
                        == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        ret = kSDWriteFailed;
                        break '_shada_write_exit;
                    } else {
                        i_1 = i_1.wrapping_add(1);
                    }
                }
                if (*wms).search_pattern.type_0 as ::core::ffi::c_int
                    != kSDItemMissing as ::core::ffi::c_int
                {
                    if shada_pack_pfreed_entry(&raw mut packer, (*wms).search_pattern, max_kbyte)
                        as ::core::ffi::c_uint
                        == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        ret = kSDWriteFailed;
                        break '_shada_write_exit;
                    }
                }
                if (*wms).sub_search_pattern.type_0 as ::core::ffi::c_int
                    != kSDItemMissing as ::core::ffi::c_int
                {
                    if shada_pack_pfreed_entry(
                        &raw mut packer,
                        (*wms).sub_search_pattern,
                        max_kbyte,
                    ) as ::core::ffi::c_uint
                        == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        ret = kSDWriteFailed;
                        break '_shada_write_exit;
                    }
                }
                if (*wms).replacement.type_0 as ::core::ffi::c_int
                    != kSDItemMissing as ::core::ffi::c_int
                {
                    if shada_pack_pfreed_entry(&raw mut packer, (*wms).replacement, max_kbyte)
                        as ::core::ffi::c_uint
                        == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        ret = kSDWriteFailed;
                        break '_shada_write_exit;
                    }
                }
                file_markss_size = (*wms).file_marks.set.h.size as size_t;
                all_file_markss = xmalloc(
                    file_markss_size.wrapping_mul(::core::mem::size_of::<*mut FileMarks>()),
                ) as *mut *mut FileMarks;
                cur_file_marks = all_file_markss;
                val_0 = ::core::ptr::null_mut::<::core::ffi::c_void>();
                let mut __i: uint32_t = 0;
                __i = 0 as uint32_t;
                while __i < (*wms).file_marks.set.h.n_keys {
                    val_0 = *(*wms).file_marks.values.offset(__i as isize);
                    let c2rust_fresh19 = cur_file_marks;
                    cur_file_marks = cur_file_marks.offset(1);
                    let c2rust_lvalue_ptr = &raw mut *c2rust_fresh19;
                    *c2rust_lvalue_ptr = val_0 as *mut FileMarks;
                    __i = __i.wrapping_add(1);
                }
                qsort(
                    all_file_markss as *mut ::core::ffi::c_void,
                    file_markss_size,
                    ::core::mem::size_of::<*mut FileMarks>(),
                    Some(
                        compare_file_marks
                            as unsafe extern "C" fn(
                                *const ::core::ffi::c_void,
                                *const ::core::ffi::c_void,
                            )
                                -> ::core::ffi::c_int,
                    ),
                );
                file_markss_to_dump = if num_marked_files < file_markss_size {
                    num_marked_files
                } else {
                    file_markss_size
                };
                let mut i_2: size_t = 0 as size_t;
                while i_2 < file_markss_to_dump {
                    let mut i__2: size_t = 0 as size_t;
                    while i__2
                        < ::core::mem::size_of::<[ShadaEntry; 29]>()
                            .wrapping_div(::core::mem::size_of::<ShadaEntry>())
                            .wrapping_div(
                                (::core::mem::size_of::<[ShadaEntry; 29]>()
                                    .wrapping_rem(::core::mem::size_of::<ShadaEntry>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            )
                    {
                        if (**all_file_markss.offset(i_2 as isize)).marks[i__2 as usize].type_0
                            as ::core::ffi::c_int
                            != kSDItemMissing as ::core::ffi::c_int
                        {
                            if shada_pack_pfreed_entry(
                                &raw mut packer,
                                (**all_file_markss.offset(i_2 as isize)).marks[i__2 as usize],
                                max_kbyte,
                            ) as ::core::ffi::c_uint
                                == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                ret = kSDWriteFailed;
                                break '_shada_write_exit;
                            }
                        }
                        i__2 = i__2.wrapping_add(1);
                    }
                    let mut j: size_t = 0 as size_t;
                    while j < (**all_file_markss.offset(i_2 as isize)).changes_size {
                        if shada_pack_pfreed_entry(
                            &raw mut packer,
                            (**all_file_markss.offset(i_2 as isize)).changes[j as usize],
                            max_kbyte,
                        ) as ::core::ffi::c_uint
                            == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            ret = kSDWriteFailed;
                            break '_shada_write_exit;
                        } else {
                            j = j.wrapping_add(1);
                        }
                    }
                    let mut j_0: size_t = 0 as size_t;
                    while j_0 < (**all_file_markss.offset(i_2 as isize)).additional_marks_size {
                        if shada_pack_entry(
                            &raw mut packer,
                            *(**all_file_markss.offset(i_2 as isize))
                                .additional_marks
                                .offset(j_0 as isize),
                            0 as size_t,
                        ) as ::core::ffi::c_uint
                            == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            shada_free_shada_entry(
                                (**all_file_markss.offset(i_2 as isize))
                                    .additional_marks
                                    .offset(j_0 as isize),
                            );
                            ret = kSDWriteFailed;
                            break '_shada_write_exit;
                        } else {
                            shada_free_shada_entry(
                                (**all_file_markss.offset(i_2 as isize))
                                    .additional_marks
                                    .offset(j_0 as isize),
                            );
                            j_0 = j_0.wrapping_add(1);
                        }
                    }
                    xfree(
                        (**all_file_markss.offset(i_2 as isize)).additional_marks
                            as *mut ::core::ffi::c_void,
                    );
                    i_2 = i_2.wrapping_add(1);
                }
                xfree(all_file_markss as *mut ::core::ffi::c_void);
                if dump_history {
                    let mut i_3: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    loop {
                        if i_3 >= HIST_COUNT as ::core::ffi::c_int {
                            break '_shada_write_exit;
                        }
                        if dump_one_history[i_3 as usize] {
                            hms_insert_whole_neovim_history(
                                (&raw mut (*wms).hms as *mut HistoryMergerState)
                                    .offset(i_3 as isize),
                            );
                            let mut cur_entry: *mut HMLListEntry =
                                (*wms).hms[i_3 as usize].hmll.first as *mut HMLListEntry;
                            while !cur_entry.is_null() {
                                if shada_pack_pfreed_entry(
                                    &raw mut packer,
                                    (*cur_entry).data,
                                    max_kbyte,
                                ) as ::core::ffi::c_uint
                                    == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                                {
                                    ret = kSDWriteFailed;
                                    break;
                                } else {
                                    cur_entry = (*cur_entry).next as *mut HMLListEntry;
                                }
                            }
                            if ret as ::core::ffi::c_uint
                                == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                break '_shada_write_exit;
                            }
                        }
                        i_3 += 1;
                    }
                }
            }
        }
        let mut i_4: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_4 < HIST_COUNT as ::core::ffi::c_int {
            if dump_one_history[i_4 as usize] {
                hms_dealloc((&raw mut (*wms).hms as *mut HistoryMergerState).offset(i_4 as isize));
            }
            i_4 += 1;
        }
        let mut stored_key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut __i_0: uint32_t = 0;
        __i_0 = 0 as uint32_t;
        while __i_0 < (*wms).file_marks.set.h.n_keys {
            stored_key =
                *(*wms).file_marks.set.keys.offset(__i_0 as isize) as *const ::core::ffi::c_char;
            val_0 = *(*wms).file_marks.values.offset(__i_0 as isize);
            xfree(stored_key as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void);
            xfree(val_0 as *mut ::core::ffi::c_void);
            __i_0 = __i_0.wrapping_add(1);
        }
        xfree((*wms).file_marks.set.keys as *mut ::core::ffi::c_void);
        xfree((*wms).file_marks.set.h.hash as *mut ::core::ffi::c_void);
        (*wms).file_marks.set = Set_cstr_t {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<cstr_t>(),
        };
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*wms).file_marks.values as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
        xfree(removable_bufs.keys as *mut ::core::ffi::c_void);
        xfree(removable_bufs.h.hash as *mut ::core::ffi::c_void);
        removable_bufs = Set_ptr_t {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<ptr_t>(),
        };
        packer.packer_flush.expect("non-null function pointer")(&mut packer);
        xfree((*wms).dumped_variables.keys as *mut ::core::ffi::c_void);
        xfree((*wms).dumped_variables.h.hash as *mut ::core::ffi::c_void);
        (*wms).dumped_variables = Set_cstr_t {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<cstr_t>(),
        };
        xfree(wms as *mut ::core::ffi::c_void);
        return ret;
    }
}
