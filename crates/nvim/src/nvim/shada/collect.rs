//! Gathering the editor's state into [`ShadaEntry`]s.
//!
//! One function per kind of thing that gets remembered: the buffer list, the
//! last search and substitute patterns, the registers, the numbered file
//! marks, the jump list. `find_removable_bufs` collects the buffers whose
//! marks are not to be written at all, which several of the others consult.
//!
//! The `shada_encode_*` entry points at the end pack a single kind into a
//! string rather than a file; `:mksession` and the msgpack API use them.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn var_shada_iter(
    iter: *const ::core::ffi::c_void,
    name: *mut *const ::core::ffi::c_char,
    mut rettv: *mut typval_T,
    mut flavour: var_flavour_T,
) -> *const ::core::ffi::c_void {
    unsafe {
        let mut hi: *const hashitem_T = ::core::ptr::null::<hashitem_T>();
        let mut globvarht: *mut hashtab_T = get_globvar_ht();
        let mut hifirst: *const hashitem_T = (*globvarht).ht_array;
        let hinum: size_t = (*globvarht).ht_mask.wrapping_add(1 as size_t);
        *name = ::core::ptr::null::<::core::ffi::c_char>();
        if iter.is_null() {
            hi = (*globvarht).ht_array;
            while (hi.offset_from(hifirst) as size_t) < hinum
                && ((*hi).hi_key.is_null()
                    || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
                    || var_flavour((*hi).hi_key) as ::core::ffi::c_uint
                        & flavour as ::core::ffi::c_uint
                        == 0)
            {
                hi = hi.offset(1);
            }
            if hi.offset_from(hifirst) as size_t == hinum {
                return ::core::ptr::null::<::core::ffi::c_void>();
            }
        } else {
            hi = iter as *const hashitem_T;
        }
        *name = &raw mut (*((*hi).hi_key.offset(-(17 as ::core::ffi::c_ulong as isize))
            as *mut dictitem_T))
            .di_key as *mut ::core::ffi::c_char;
        tv_copy(
            &raw mut (*((*hi).hi_key.offset(-(17 as ::core::ffi::c_ulong as isize))
                as *mut dictitem_T))
                .di_tv,
            rettv,
        );
        loop {
            hi = hi.offset(1);
            if (hi.offset_from(hifirst) as size_t) >= hinum {
                break;
            }
            if !((*hi).hi_key.is_null()
                || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
                && var_flavour((*hi).hi_key) as ::core::ffi::c_uint & flavour as ::core::ffi::c_uint
                    != 0
            {
                return hi as *const ::core::ffi::c_void;
            }
        }
        return ::core::ptr::null::<::core::ffi::c_void>();
    }
}

#[inline(always)]
pub(crate) unsafe extern "C" fn ignore_buf(
    buf: *const buf_T,
    removable_bufs: *mut Set_ptr_t,
) -> bool {
    unsafe {
        return buf.is_null()
            || (*buf).b_ffname.is_null()
            || (*buf).b_p_bl == 0 && (*buf).b_p_initialized as ::core::ffi::c_int != 0
            || bt_quickfix(buf) as ::core::ffi::c_int != 0
            || bt_terminal(buf) as ::core::ffi::c_int != 0
            || set_has_ptr_t(removable_bufs, buf as ptr_t) as ::core::ffi::c_int != 0;
    }
}

#[inline(always)]
pub(crate) unsafe extern "C" fn shada_get_buflist(removable_bufs: *mut Set_ptr_t) -> ShadaEntry {
    unsafe {
        let mut max_bufs: ::core::ffi::c_int = get_shada_parameter('%' as ::core::ffi::c_int);
        let mut buf_count: size_t = 0 as size_t;
        let mut buf: *mut buf_T = firstbuf.get();
        while !buf.is_null() {
            if !ignore_buf(buf, removable_bufs)
                && (*buf).b_p_bl != 0
                && (max_bufs < 0 as ::core::ffi::c_int || buf_count < max_bufs as size_t)
            {
                buf_count = buf_count.wrapping_add(1);
            }
            buf = (*buf).b_next;
        }
        let mut buflist_entry: ShadaEntry = ShadaEntry {
            type_0: kSDItemBufferList,
            can_free_entry: false,
            timestamp: os_time(),
            data: C2Rust_Unnamed_22 {
                buffer_list: buffer_list {
                    size: buf_count,
                    buffers: xmalloc(
                        buf_count.wrapping_mul(::core::mem::size_of::<buffer_list_buffer>()),
                    ) as *mut buffer_list_buffer,
                },
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        };
        let mut i: size_t = 0 as size_t;
        let mut buf_0: *mut buf_T = firstbuf.get();
        while !buf_0.is_null() {
            if !(ignore_buf(buf_0, removable_bufs) as ::core::ffi::c_int != 0
                || (*buf_0).b_p_bl == 0)
            {
                if i >= buf_count {
                    break;
                }
                *buflist_entry.data.buffer_list.buffers.offset(i as isize) = buffer_list_buffer {
                    pos: (*buf_0).b_last_cursor.mark,
                    fname: (*buf_0).b_ffname,
                    additional_data: (*buf_0).additional_data,
                }
                    as buffer_list_buffer;
                i = i.wrapping_add(1);
            }
            buf_0 = (*buf_0).b_next;
        }
        return buflist_entry;
    }
}

#[inline(always)]
pub(crate) unsafe extern "C" fn add_search_pattern(
    ret_pse: *mut ShadaEntry,
    get_pattern: SearchPatternGetter,
    is_substitute_pattern: bool,
    search_last_used: bool,
    search_highlighted: bool,
) {
    unsafe {
        let defaults: ShadaEntry =
            (*sd_default_values.ptr())[kSDItemSearchPattern as ::core::ffi::c_int as usize];
        let mut pat: SearchPattern = SearchPattern {
            pat: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            patlen: 0,
            magic: false,
            no_scs: false,
            timestamp: 0,
            off: SearchOffset {
                dir: 0,
                line: false,
                end: false,
                off: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        };
        get_pattern.expect("non-null function pointer")(&raw mut pat);
        if !pat.pat.is_null() {
            *ret_pse = ShadaEntry {
                type_0: kSDItemSearchPattern,
                can_free_entry: false_0 != 0,
                timestamp: pat.timestamp,
                data: C2Rust_Unnamed_22 {
                    search_pattern: KeyDict__shada_search_pat {
                        is_set___shada_search_pat_: 0,
                        magic: pat.magic as Boolean,
                        smartcase: !pat.no_scs,
                        has_line_offset: if is_substitute_pattern as ::core::ffi::c_int != 0 {
                            defaults.data.search_pattern.has_line_offset as ::core::ffi::c_int
                        } else {
                            pat.off.line as ::core::ffi::c_int
                        } != 0,
                        place_cursor_at_end: if is_substitute_pattern as ::core::ffi::c_int != 0 {
                            defaults.data.search_pattern.place_cursor_at_end as ::core::ffi::c_int
                        } else {
                            pat.off.end as ::core::ffi::c_int
                        } != 0,
                        is_last_used: is_substitute_pattern as ::core::ffi::c_int
                            ^ search_last_used as ::core::ffi::c_int
                            != 0,
                        is_substitute_pattern: is_substitute_pattern as Boolean,
                        highlighted: is_substitute_pattern as ::core::ffi::c_int
                            ^ search_last_used as ::core::ffi::c_int
                            != 0
                            && search_highlighted as ::core::ffi::c_int != 0,
                        search_backward: !is_substitute_pattern
                            && pat.off.dir as ::core::ffi::c_int == '?' as ::core::ffi::c_int,
                        offset: if is_substitute_pattern as ::core::ffi::c_int != 0 {
                            defaults.data.search_pattern.offset
                        } else {
                            pat.off.off as Integer
                        },
                        pat: cstr_as_string(pat.pat),
                    },
                },
                additional_data: pat.additional_data,
            };
        }
    }
}

#[inline(always)]
pub(crate) unsafe extern "C" fn shada_initialize_registers(
    wms: *mut WriteMergerState,
    mut max_reg_lines: ::core::ffi::c_int,
) {
    unsafe {
        let mut reg_iter: *const ::core::ffi::c_void = ::core::ptr::null::<::core::ffi::c_void>();
        let limit_reg_lines: bool = max_reg_lines >= 0 as ::core::ffi::c_int;
        loop {
            let mut reg: yankreg_T = yankreg_T {
                y_array: ::core::ptr::null_mut::<String_0>(),
                y_size: 0,
                y_type: kMTCharWise,
                y_width: 0,
                timestamp: 0,
                additional_data: ::core::ptr::null_mut::<AdditionalData>(),
            };
            let mut name: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
            let mut is_unnamed: bool = false_0 != 0;
            reg_iter =
                op_global_reg_iter(reg_iter, &raw mut name, &raw mut reg, &raw mut is_unnamed);
            if name as ::core::ffi::c_int == NUL {
                break;
            }
            if !(limit_reg_lines as ::core::ffi::c_int != 0 && reg.y_size > max_reg_lines as size_t)
            {
                (*wms).registers[op_reg_index(name as ::core::ffi::c_int) as usize] = ShadaEntry {
                    type_0: kSDItemRegister,
                    can_free_entry: false_0 != 0,
                    timestamp: reg.timestamp,
                    data: C2Rust_Unnamed_22 {
                        reg: reg {
                            name: name,
                            type_0: reg.y_type,
                            contents: reg.y_array,
                            is_unnamed: is_unnamed,
                            contents_size: reg.y_size,
                            width: (if reg.y_type as ::core::ffi::c_int
                                == kMTBlockWise as ::core::ffi::c_int
                            {
                                reg.y_width as ::core::ffi::c_int
                            } else {
                                0 as ::core::ffi::c_int
                            }) as size_t,
                        },
                    },
                    additional_data: reg.additional_data,
                };
            }
            if reg_iter.is_null() {
                break;
            }
        }
    }
}

#[inline(always)]
pub(crate) unsafe extern "C" fn replace_numbered_mark(
    wms: *mut WriteMergerState,
    idx: size_t,
    entry: ShadaEntry,
) {
    unsafe {
        shada_free_shada_entry(
            (&raw mut (*wms).numbered_marks as *mut ShadaEntry).offset(
                ::core::mem::size_of::<[ShadaEntry; 10]>()
                    .wrapping_div(::core::mem::size_of::<ShadaEntry>())
                    .wrapping_div(
                        (::core::mem::size_of::<[ShadaEntry; 10]>()
                            .wrapping_rem(::core::mem::size_of::<ShadaEntry>())
                            == 0) as ::core::ffi::c_int as usize,
                    )
                    .wrapping_sub(1 as usize) as isize,
            ),
        );
        let mut i: size_t = idx;
        while i < ::core::mem::size_of::<[ShadaEntry; 10]>()
            .wrapping_div(::core::mem::size_of::<ShadaEntry>())
            .wrapping_div(
                (::core::mem::size_of::<[ShadaEntry; 10]>()
                    .wrapping_rem(::core::mem::size_of::<ShadaEntry>())
                    == 0) as ::core::ffi::c_int as usize,
            )
            .wrapping_sub(1 as usize)
        {
            if (*wms).numbered_marks[i as usize].type_0 as ::core::ffi::c_int
                == kSDItemGlobalMark as ::core::ffi::c_int
            {
                (*wms).numbered_marks[i as usize].data.filemark.name =
                    ('0' as ::core::ffi::c_int + i as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        as ::core::ffi::c_char;
            }
            i = i.wrapping_add(1);
        }
        memmove(
            (&raw mut (*wms).numbered_marks as *mut ShadaEntry)
                .offset(idx as isize)
                .offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            (&raw mut (*wms).numbered_marks as *mut ShadaEntry).offset(idx as isize)
                as *const ::core::ffi::c_void,
            ::core::mem::size_of::<ShadaEntry>().wrapping_mul(
                ::core::mem::size_of::<[ShadaEntry; 10]>()
                    .wrapping_div(::core::mem::size_of::<ShadaEntry>())
                    .wrapping_div(
                        (::core::mem::size_of::<[ShadaEntry; 10]>()
                            .wrapping_rem(::core::mem::size_of::<ShadaEntry>())
                            == 0) as ::core::ffi::c_int as size_t,
                    )
                    .wrapping_sub(1 as size_t)
                    .wrapping_sub(idx),
            ),
        );
        (*wms).numbered_marks[idx as usize] = entry;
        (*wms).numbered_marks[idx as usize].data.filemark.name =
            ('0' as ::core::ffi::c_int + idx as ::core::ffi::c_int) as ::core::ffi::c_char;
    }
}

#[inline]
pub(crate) unsafe extern "C" fn find_removable_bufs(mut removable_bufs: *mut Set_ptr_t) {
    unsafe {
        let mut buf: *mut buf_T = firstbuf.get();
        while !buf.is_null() {
            if !(*buf).b_ffname.is_null()
                && shada_removable((*buf).b_ffname) as ::core::ffi::c_int != 0
            {
                set_put_ptr_t(
                    removable_bufs,
                    buf as ptr_t,
                    ::core::ptr::null_mut::<*mut ptr_t>(),
                );
            }
            buf = (*buf).b_next;
        }
    }
}

pub(crate) unsafe extern "C" fn hist_type2char(type_0: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        match type_0 {
            0 => return ':' as ::core::ffi::c_int,
            1 => return '/' as ::core::ffi::c_int,
            2 => return '=' as ::core::ffi::c_int,
            3 => return '@' as ::core::ffi::c_int,
            4 => return '>' as ::core::ffi::c_int,
            _ => {
                abort();
            }
        };
    }
}

#[inline]
pub(crate) unsafe extern "C" fn shada_init_jumps(
    mut jumps: *mut ShadaEntry,
    removable_bufs: *mut Set_ptr_t,
) -> size_t {
    unsafe {
        let mut jumps_size: size_t = 0 as size_t;
        let mut jump_iter: *const ::core::ffi::c_void = ::core::ptr::null::<::core::ffi::c_void>();
        setpcmark();
        cleanup_jumplist(curwin.get(), false_0 != 0);
        loop {
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
            jump_iter = mark_jumplist_iter(jump_iter, curwin.get(), &raw mut fm);
            if fm.fmark.mark.lnum == 0 as linenr_T {
                siemsg(
                    b"ShaDa: mark lnum zero (ji:%p, js:%p, len:%i)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    jump_iter as *mut ::core::ffi::c_void,
                    (&raw mut (*curwin.get()).w_jumplist as *mut xfmark_T)
                        .offset(0 as ::core::ffi::c_int as isize)
                        as *mut ::core::ffi::c_void,
                    (*curwin.get()).w_jumplistlen,
                );
            } else {
                let buf: *const buf_T = if fm.fmark.fnum == 0 as ::core::ffi::c_int {
                    ::core::ptr::null_mut::<buf_T>()
                } else {
                    buflist_findnr(fm.fmark.fnum)
                };
                if if !buf.is_null() {
                    ignore_buf(buf, removable_bufs) as ::core::ffi::c_int
                } else {
                    (fm.fmark.fnum != 0 as ::core::ffi::c_int) as ::core::ffi::c_int
                } == 0
                {
                    let fname: *const ::core::ffi::c_char =
                        if fm.fmark.fnum == 0 as ::core::ffi::c_int {
                            if fm.fname.is_null() {
                                ::core::ptr::null_mut::<::core::ffi::c_char>()
                            } else {
                                fm.fname
                            }
                        } else if !buf.is_null() {
                            (*buf).b_ffname
                        } else {
                            ::core::ptr::null_mut::<::core::ffi::c_char>()
                        };
                    if !fname.is_null() {
                        let c2rust_fresh21 = jumps_size;
                        jumps_size = jumps_size.wrapping_add(1);
                        *jumps.offset(c2rust_fresh21 as isize) = ShadaEntry {
                            type_0: kSDItemJump,
                            can_free_entry: false_0 != 0,
                            timestamp: fm.fmark.timestamp,
                            data: C2Rust_Unnamed_22 {
                                filemark: shada_filemark {
                                    name: NUL as ::core::ffi::c_char,
                                    mark: fm.fmark.mark,
                                    fname: fname as *mut ::core::ffi::c_char,
                                },
                            },
                            additional_data: fm.fmark.additional_data,
                        };
                    }
                }
            }
            if jump_iter.is_null() {
                break;
            }
        }
        return jumps_size;
    }
}

pub unsafe extern "C" fn shada_encode_regs() -> String_0 {
    unsafe {
        let wms: *mut WriteMergerState =
            xcalloc(1 as size_t, ::core::mem::size_of::<WriteMergerState>())
                as *mut WriteMergerState;
        shada_initialize_registers(wms, -1 as ::core::ffi::c_int);
        let mut packer: PackerBuffer = packer_string_buffer();
        let mut i: size_t = 0 as size_t;
        while i < ::core::mem::size_of::<[ShadaEntry; 37]>()
            .wrapping_div(::core::mem::size_of::<ShadaEntry>())
            .wrapping_div(
                (::core::mem::size_of::<[ShadaEntry; 37]>()
                    .wrapping_rem(::core::mem::size_of::<ShadaEntry>())
                    == 0) as ::core::ffi::c_int as usize,
            )
        {
            if (*wms).registers[i as usize].type_0 as ::core::ffi::c_int
                == kSDItemRegister as ::core::ffi::c_int
            {
                if kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                    == shada_pack_pfreed_entry(
                        &raw mut packer,
                        (*wms).registers[i as usize],
                        0 as size_t,
                    ) as ::core::ffi::c_uint
                {
                    abort();
                }
            }
            i = i.wrapping_add(1);
        }
        xfree(wms as *mut ::core::ffi::c_void);
        return packer_take_string(&mut packer);
    }
}

pub unsafe extern "C" fn shada_encode_jumps() -> String_0 {
    unsafe {
        let mut removable_bufs: Set_ptr_t = Set_ptr_t {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<ptr_t>(),
        };
        find_removable_bufs(&raw mut removable_bufs);
        let mut jumps: [ShadaEntry; 100] = [ShadaEntry {
            type_0: kSDItemMissing,
            can_free_entry: false,
            timestamp: 0,
            data: C2Rust_Unnamed_22 {
                header: Dict {
                    size: 0,
                    capacity: 0,
                    items: ::core::ptr::null_mut::<KeyValuePair>(),
                },
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        }; 100];
        let mut jumps_size: size_t =
            shada_init_jumps(&raw mut jumps as *mut ShadaEntry, &raw mut removable_bufs);
        let mut packer: PackerBuffer = packer_string_buffer();
        let mut i: size_t = 0 as size_t;
        while i < jumps_size {
            if kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                == shada_pack_pfreed_entry(&raw mut packer, jumps[i as usize], 0 as size_t)
                    as ::core::ffi::c_uint
            {
                abort();
            }
            i = i.wrapping_add(1);
        }
        return packer_take_string(&mut packer);
    }
}

pub unsafe extern "C" fn shada_encode_buflist() -> String_0 {
    unsafe {
        let mut removable_bufs: Set_ptr_t = Set_ptr_t {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<ptr_t>(),
        };
        find_removable_bufs(&raw mut removable_bufs);
        let mut buflist_entry: ShadaEntry = shada_get_buflist(&raw mut removable_bufs);
        let mut packer: PackerBuffer = packer_string_buffer();
        if kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
            == shada_pack_entry(&raw mut packer, buflist_entry, 0 as size_t) as ::core::ffi::c_uint
        {
            abort();
        }
        xfree(buflist_entry.data.buffer_list.buffers as *mut ::core::ffi::c_void);
        return packer_take_string(&mut packer);
    }
}

pub unsafe extern "C" fn shada_encode_gvars() -> String_0 {
    unsafe {
        let mut packer: PackerBuffer = packer_string_buffer();
        let mut var_iter: *const ::core::ffi::c_void = ::core::ptr::null::<::core::ffi::c_void>();
        let cur_timestamp: Timestamp = os_time();
        loop {
            let mut vartv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
            var_iter = var_shada_iter(
                var_iter,
                &raw mut name,
                &raw mut vartv,
                (VAR_FLAVOUR_DEFAULT as ::core::ffi::c_int
                    | VAR_FLAVOUR_SESSION as ::core::ffi::c_int
                    | VAR_FLAVOUR_SHADA as ::core::ffi::c_int) as var_flavour_T,
            );
            if name.is_null() {
                break;
            }
            if vartv.v_type as ::core::ffi::c_uint
                != VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
                && vartv.v_type as ::core::ffi::c_uint
                    != VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut tgttv: typval_T = typval_T {
                    v_type: VAR_UNKNOWN,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union { v_number: 0 },
                };
                tv_copy(&raw mut vartv, &raw mut tgttv);
                let mut r: ShaDaWriteResult = shada_pack_entry(
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
                    0 as size_t,
                );
                if kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                    == r as ::core::ffi::c_uint
                {
                    abort();
                }
                tv_clear(&raw mut tgttv);
            }
            tv_clear(&raw mut vartv);
            if var_iter.is_null() {
                break;
            }
        }
        return packer_take_string(&mut packer);
    }
}
