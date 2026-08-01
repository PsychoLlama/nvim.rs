//! Applying a ShaDa file to the running editor.
//!
//! `shada_read` walks the entries an already-opened file yields and puts each
//! one where it belongs — registers, marks, histories, global variables, the
//! buffer list — subject to the `kSDRead*` flags that say which kinds the
//! caller asked for and to the `'shada'` option's limits.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn find_buffer(
    fname_bufs: *mut Map_cstr_t_ptr_t,
    fname: *const ::core::ffi::c_char,
) -> *mut buf_T {
    unsafe {
        let mut key_alloc: *mut cstr_t = ::core::ptr::null_mut::<cstr_t>();
        let mut new_item: bool = false_0 != 0;
        let mut ref_0: *mut *mut buf_T = map_put_ref_cstr_t_ptr_t(
            fname_bufs,
            fname as cstr_t,
            &raw mut key_alloc,
            &raw mut new_item,
        ) as *mut *mut buf_T;
        if new_item {
            *key_alloc = xstrdup(fname) as cstr_t;
        } else {
            return *ref_0;
        }
        let mut buf: *mut buf_T = firstbuf.get();
        while !buf.is_null() {
            if !(*buf).b_ffname.is_null() {
                if path_fnamecmp(fname, (*buf).b_ffname) == 0 as ::core::ffi::c_int {
                    *ref_0 = buf;
                    return buf;
                }
            }
            buf = (*buf).b_next;
        }
        *ref_0 = ::core::ptr::null_mut::<buf_T>();
        return ::core::ptr::null_mut::<buf_T>();
    }
}

pub(crate) unsafe extern "C" fn shada_read(
    sd_reader: *mut FileDescriptor,
    flags: ::core::ffi::c_int,
) {
    unsafe {
        let mut oldfiles_list: *mut list_T = get_vim_var_list(VV_OLDFILES);
        let force: bool = flags & kShaDaForceit as ::core::ffi::c_int != 0;
        let get_old_files: bool = flags
            & (kShaDaGetOldfiles as ::core::ffi::c_int | kShaDaForceit as ::core::ffi::c_int)
            != 0
            && (force as ::core::ffi::c_int != 0
                || tv_list_len(oldfiles_list) == 0 as ::core::ffi::c_int);
        let want_marks: bool = flags & kShaDaWantMarks as ::core::ffi::c_int != 0;
        let srni_flags: ::core::ffi::c_uint = ((if flags & kShaDaWantInfo as ::core::ffi::c_int != 0
        {
            kSDReadUndisableableData as ::core::ffi::c_int
                | kSDReadRegisters as ::core::ffi::c_int
                | kSDReadGlobalMarks as ::core::ffi::c_int
                | (if p_hi.get() != 0 {
                    kSDReadHistory as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                })
                | (if !find_shada_parameter('!' as ::core::ffi::c_int).is_null() {
                    kSDReadVariables as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                })
                | (if !find_shada_parameter('%' as ::core::ffi::c_int).is_null()
                    && (*(*curwin.get()).w_alist).al_ga.ga_len == 0 as ::core::ffi::c_int
                {
                    kSDReadBufferList as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                })
        } else {
            0 as ::core::ffi::c_int
        }) | (if want_marks as ::core::ffi::c_int != 0
            && get_shada_parameter('\'' as ::core::ffi::c_int) > 0 as ::core::ffi::c_int
        {
            kSDReadLocalMarks as ::core::ffi::c_int | kSDReadChanges as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | (if get_old_files as ::core::ffi::c_int != 0 {
            kSDReadLocalMarks as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        })) as ::core::ffi::c_uint;
        if srni_flags == 0 as ::core::ffi::c_uint {
            return;
        }
        let mut hms: [HistoryMergerState; 5] = [HistoryMergerState {
            hmll: HMLList {
                entries: ::core::ptr::null_mut::<HMLListEntry>(),
                first: ::core::ptr::null_mut::<HMLListEntry>(),
                last: ::core::ptr::null_mut::<HMLListEntry>(),
                free_entry: ::core::ptr::null_mut::<HMLListEntry>(),
                last_free_entry: ::core::ptr::null_mut::<HMLListEntry>(),
                size: 0,
                num_entries: 0,
                contained_entries: Map_cstr_t_ptr_t {
                    set: Set_cstr_t {
                        h: MapHash {
                            n_buckets: 0,
                            size: 0,
                            n_occupied: 0,
                            upper_bound: 0,
                            n_keys: 0,
                            keys_capacity: 0,
                            hash: ::core::ptr::null_mut::<uint32_t>(),
                        },
                        keys: ::core::ptr::null_mut::<cstr_t>(),
                    },
                    values: ::core::ptr::null_mut::<ptr_t>(),
                },
            },
            do_merge: false,
            reading: false,
            pending: ::core::ptr::null_mut::<ShadaEntry>(),
            pending_len: 0,
            pending_pos: 0,
            history_type: 0,
        }; 5];
        if srni_flags & kSDReadHistory as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < HIST_COUNT as ::core::ffi::c_int {
                hms_init(
                    (&raw mut hms as *mut HistoryMergerState).offset(i as isize),
                    i as uint8_t,
                    p_hi.get() as size_t,
                    true_0 != 0,
                    true_0 != 0,
                );
                i += 1;
            }
        }
        let mut cur_entry: ShadaEntry = ShadaEntry {
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
        };
        let mut cl_bufs: Set_ptr_t = Set_ptr_t {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<ptr_t>(),
        };
        let mut fname_bufs: Map_cstr_t_ptr_t = MAP_INIT;
        let mut oldfiles_set: Set_cstr_t = Set_cstr_t {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<cstr_t>(),
        };
        if get_old_files as ::core::ffi::c_int != 0
            && (oldfiles_list.is_null() || force as ::core::ffi::c_int != 0)
        {
            oldfiles_list = tv_list_alloc(kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
            set_vim_var_list(VV_OLDFILES, oldfiles_list);
        }
        let mut srni_ret: ShaDaReadResult = kSDReadStatusSuccess;
        loop {
            srni_ret = shada_read_next_item(sd_reader, &raw mut cur_entry, srni_flags, 0 as size_t);
            if srni_ret as ::core::ffi::c_uint
                == kSDReadStatusFinished as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                break;
            }
            match srni_ret as ::core::ffi::c_uint {
                1 => {
                    abort();
                }
                3 | 2 => {
                    break;
                }
                4 => {}
                0 | _ => {
                    let mut spat: SearchPattern = SearchPattern {
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
                    's_732: {
                        match cur_entry.type_0 as ::core::ffi::c_int {
                            0 => {
                                abort();
                            }
                            1 => {
                                shada_free_shada_entry(&raw mut cur_entry);
                            }
                            2 => {
                                if !force {
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
                                    if cur_entry.data.search_pattern.is_substitute_pattern {
                                        get_substitute_pattern(&raw mut pat);
                                    } else {
                                        get_search_pattern(&raw mut pat);
                                    }
                                    if !pat.pat.is_null() && pat.timestamp >= cur_entry.timestamp {
                                        shada_free_shada_entry(&raw mut cur_entry);
                                        break 's_732;
                                    }
                                }
                                spat = SearchPattern {
                                    pat: cur_entry.data.search_pattern.pat.data,
                                    patlen: cur_entry.data.search_pattern.pat.size,
                                    magic: cur_entry.data.search_pattern.magic as bool,
                                    no_scs: !cur_entry.data.search_pattern.smartcase,
                                    timestamp: cur_entry.timestamp,
                                    off: SearchOffset {
                                        dir: (if cur_entry.data.search_pattern.search_backward
                                            as ::core::ffi::c_int
                                            != 0
                                        {
                                            '?' as ::core::ffi::c_int
                                        } else {
                                            '/' as ::core::ffi::c_int
                                        })
                                            as ::core::ffi::c_char,
                                        line: cur_entry.data.search_pattern.has_line_offset as bool,
                                        end: cur_entry.data.search_pattern.place_cursor_at_end
                                            as bool,
                                        off: cur_entry.data.search_pattern.offset as int64_t,
                                    },
                                    additional_data: cur_entry.additional_data,
                                };
                                if cur_entry.data.search_pattern.is_substitute_pattern {
                                    set_substitute_pattern(spat);
                                } else {
                                    set_search_pattern(spat);
                                }
                                if cur_entry.data.search_pattern.is_last_used {
                                    set_last_used_pattern(
                                        cur_entry.data.search_pattern.is_substitute_pattern as bool,
                                    );
                                    set_no_hlsearch(!cur_entry.data.search_pattern.highlighted);
                                }
                            }
                            3 => {
                                if !force {
                                    let mut sub: SubReplacementString = SubReplacementString {
                                        sub: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                        timestamp: 0,
                                        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
                                    };
                                    sub_get_replacement(&raw mut sub);
                                    if !sub.sub.is_null() && sub.timestamp >= cur_entry.timestamp {
                                        shada_free_shada_entry(&raw mut cur_entry);
                                        break 's_732;
                                    }
                                }
                                sub_set_replacement(SubReplacementString {
                                    sub: cur_entry.data.sub_string.sub,
                                    timestamp: cur_entry.timestamp,
                                    additional_data: cur_entry.additional_data,
                                });
                                regtilde(
                                    cur_entry.data.sub_string.sub,
                                    magic_isset() as ::core::ffi::c_int,
                                    false_0 != 0,
                                );
                            }
                            4 => {
                                if cur_entry.data.history_item.histtype as ::core::ffi::c_int
                                    >= HIST_COUNT as ::core::ffi::c_int
                                {
                                    shada_free_shada_entry(&raw mut cur_entry);
                                } else {
                                    hms_insert(
                                        (&raw mut hms as *mut HistoryMergerState).offset(
                                            cur_entry.data.history_item.histtype
                                                as ::core::ffi::c_int
                                                as isize,
                                        ),
                                        cur_entry,
                                        true_0 != 0,
                                    );
                                }
                            }
                            5 => {
                                if cur_entry.data.reg.type_0 as ::core::ffi::c_int
                                    != kMTCharWise as ::core::ffi::c_int
                                    && cur_entry.data.reg.type_0 as ::core::ffi::c_int
                                        != kMTLineWise as ::core::ffi::c_int
                                    && cur_entry.data.reg.type_0 as ::core::ffi::c_int
                                        != kMTBlockWise as ::core::ffi::c_int
                                {
                                    shada_free_shada_entry(&raw mut cur_entry);
                                } else {
                                    if !force {
                                        let reg: *const yankreg_T =
                                            op_reg_get(cur_entry.data.reg.name);
                                        if reg.is_null() || (*reg).timestamp >= cur_entry.timestamp
                                        {
                                            shada_free_shada_entry(&raw mut cur_entry);
                                            break 's_732;
                                        }
                                    }
                                    if !op_reg_set(
                                        cur_entry.data.reg.name,
                                        yankreg_T {
                                            y_array: cur_entry.data.reg.contents,
                                            y_size: cur_entry.data.reg.contents_size,
                                            y_type: cur_entry.data.reg.type_0,
                                            y_width: cur_entry.data.reg.width as colnr_T,
                                            timestamp: cur_entry.timestamp,
                                            additional_data: cur_entry.additional_data,
                                        },
                                        cur_entry.data.reg.is_unnamed,
                                    ) {
                                        shada_free_shada_entry(&raw mut cur_entry);
                                    }
                                }
                            }
                            6 => {
                                var_set_global(
                                    cur_entry.data.global_var.name,
                                    cur_entry.data.global_var.value,
                                );
                                cur_entry.data.global_var.value.v_type = VAR_UNKNOWN;
                                shada_free_shada_entry(&raw mut cur_entry);
                            }
                            8 | 7 => {
                                let mut buf: *mut buf_T =
                                    find_buffer(&raw mut fname_bufs, cur_entry.data.filemark.fname);
                                if !buf.is_null() {
                                    let mut ptr_: *mut *mut ::core::ffi::c_void =
                                        &raw mut cur_entry.data.filemark.fname
                                            as *mut *mut ::core::ffi::c_void;
                                    xfree(*ptr_);
                                    *ptr_ = NULL_0;
                                    let _ = *ptr_;
                                }
                                let mut fm: xfmark_T = xfmark_T {
                                    fmark: fmark_T {
                                        mark: cur_entry.data.filemark.mark,
                                        fnum: if buf.is_null() {
                                            0 as ::core::ffi::c_int
                                        } else {
                                            (*buf).handle as ::core::ffi::c_int
                                        },
                                        timestamp: cur_entry.timestamp,
                                        view: fmarkv_T {
                                            topline_offset: MAXLNUM as ::core::ffi::c_int
                                                as linenr_T,
                                            skipcol: 0 as colnr_T,
                                        },
                                        additional_data: cur_entry.additional_data,
                                    },
                                    fname: if buf.is_null() {
                                        cur_entry.data.filemark.fname
                                    } else {
                                        ::core::ptr::null_mut::<::core::ffi::c_char>()
                                    },
                                };
                                if cur_entry.type_0 as ::core::ffi::c_int
                                    == kSDItemGlobalMark as ::core::ffi::c_int
                                {
                                    if !mark_set_global(cur_entry.data.filemark.name, fm, !force) {
                                        shada_free_shada_entry(&raw mut cur_entry);
                                    }
                                } else {
                                    let mut i_0: ::core::ffi::c_int = 0;
                                    i_0 = (*curwin.get()).w_jumplistlen;
                                    while i_0 > 0 as ::core::ffi::c_int {
                                        let jl_entry: xfmark_T = (*curwin.get()).w_jumplist
                                            [(i_0 - 1 as ::core::ffi::c_int) as usize];
                                        if jl_entry.fmark.timestamp <= cur_entry.timestamp {
                                            if marks_equal(
                                                jl_entry.fmark.mark,
                                                cur_entry.data.filemark.mark,
                                            )
                                                as ::core::ffi::c_int
                                                != 0
                                                && (if buf.is_null() {
                                                    (!jl_entry.fname.is_null()
                                                        && strcmp(fm.fname, jl_entry.fname)
                                                            == 0 as ::core::ffi::c_int)
                                                        as ::core::ffi::c_int
                                                } else {
                                                    (fm.fmark.fnum == jl_entry.fmark.fnum)
                                                        as ::core::ffi::c_int
                                                }) != 0
                                            {
                                                i_0 = -1 as ::core::ffi::c_int;
                                            }
                                            break;
                                        } else {
                                            i_0 -= 1;
                                        }
                                    }
                                    if i_0 > 0 as ::core::ffi::c_int
                                        && (*curwin.get()).w_jumplistlen == JUMPLISTSIZE
                                    {
                                        free_xfmark(
                                            (*curwin.get()).w_jumplist
                                                [0 as ::core::ffi::c_int as usize],
                                        );
                                    }
                                    i_0 = marklist_insert(
                                        &raw mut (*curwin.get()).w_jumplist as *mut xfmark_T
                                            as *mut ::core::ffi::c_void,
                                        ::core::mem::size_of::<xfmark_T>(),
                                        (*curwin.get()).w_jumplistlen,
                                        i_0,
                                    );
                                    if i_0 != -1 as ::core::ffi::c_int {
                                        (*curwin.get()).w_jumplist[i_0 as usize] = fm;
                                        if (*curwin.get()).w_jumplistlen < JUMPLISTSIZE {
                                            (*curwin.get()).w_jumplistlen += 1;
                                        }
                                        if (*curwin.get()).w_jumplistidx >= i_0
                                            && (*curwin.get()).w_jumplistidx
                                                + 1 as ::core::ffi::c_int
                                                <= (*curwin.get()).w_jumplistlen
                                        {
                                            (*curwin.get()).w_jumplistidx += 1;
                                        }
                                    } else {
                                        shada_free_shada_entry(&raw mut cur_entry);
                                    }
                                }
                            }
                            9 => {
                                let mut i_1: size_t = 0 as size_t;
                                while i_1 < cur_entry.data.buffer_list.size {
                                    let sfname: *mut ::core::ffi::c_char = path_try_shorten_fname(
                                        (*cur_entry.data.buffer_list.buffers.offset(i_1 as isize))
                                            .fname,
                                    );
                                    let buf_0: *mut buf_T = buflist_new(
                                        (*cur_entry.data.buffer_list.buffers.offset(i_1 as isize))
                                            .fname,
                                        sfname,
                                        0 as linenr_T,
                                        BLN_LISTED as ::core::ffi::c_int,
                                    );
                                    if !buf_0.is_null() {
                                        let mut view: fmarkv_T = fmarkv_T {
                                            topline_offset: MAXLNUM as ::core::ffi::c_int
                                                as linenr_T,
                                            skipcol: 0 as colnr_T,
                                        };
                                        let fmarkp___: *mut fmark_T =
                                            &raw mut (*buf_0).b_last_cursor;
                                        free_fmark(*fmarkp___);
                                        let fmarkp__: *mut fmark_T = fmarkp___;
                                        (*fmarkp__).mark = (*cur_entry
                                            .data
                                            .buffer_list
                                            .buffers
                                            .offset(i_1 as isize))
                                        .pos;
                                        (*fmarkp__).fnum = 0 as ::core::ffi::c_int;
                                        (*fmarkp__).timestamp = os_time();
                                        (*fmarkp__).view = view;
                                        (*fmarkp__).additional_data =
                                            ::core::ptr::null_mut::<AdditionalData>();
                                        buflist_setfpos(
                                            buf_0,
                                            curwin.get(),
                                            (*buf_0).b_last_cursor.mark.lnum,
                                            (*buf_0).b_last_cursor.mark.col,
                                            false_0 != 0,
                                        );
                                        xfree((*buf_0).additional_data as *mut ::core::ffi::c_void);
                                        (*buf_0).additional_data = (*cur_entry
                                            .data
                                            .buffer_list
                                            .buffers
                                            .offset(i_1 as isize))
                                        .additional_data;
                                        (*cur_entry
                                            .data
                                            .buffer_list
                                            .buffers
                                            .offset(i_1 as isize))
                                        .additional_data =
                                            ::core::ptr::null_mut::<AdditionalData>();
                                    }
                                    i_1 = i_1.wrapping_add(1);
                                }
                                shada_free_shada_entry(&raw mut cur_entry);
                            }
                            11 | 10 => {
                                if get_old_files as ::core::ffi::c_int != 0
                                    && !set_has_cstr_t(
                                        &raw mut oldfiles_set,
                                        cur_entry.data.filemark.fname as cstr_t,
                                    )
                                {
                                    let mut fname: *mut ::core::ffi::c_char =
                                        cur_entry.data.filemark.fname;
                                    if want_marks {
                                        fname = xstrdup(fname);
                                    }
                                    set_put_cstr_t(
                                        &raw mut oldfiles_set,
                                        fname as cstr_t,
                                        ::core::ptr::null_mut::<*mut cstr_t>(),
                                    );
                                    tv_list_append_allocated_string(oldfiles_list, fname);
                                    if !want_marks {
                                        cur_entry.data.filemark.fname =
                                            ::core::ptr::null_mut::<::core::ffi::c_char>();
                                    }
                                }
                                if !want_marks {
                                    shada_free_shada_entry(&raw mut cur_entry);
                                } else {
                                    let mut buf_1: *mut buf_T = find_buffer(
                                        &raw mut fname_bufs,
                                        cur_entry.data.filemark.fname,
                                    );
                                    if buf_1.is_null() {
                                        shada_free_shada_entry(&raw mut cur_entry);
                                    } else {
                                        let fm_0: fmark_T = fmark_T {
                                            mark: cur_entry.data.filemark.mark,
                                            fnum: (*buf_1).handle as ::core::ffi::c_int,
                                            timestamp: cur_entry.timestamp,
                                            view: fmarkv_T {
                                                topline_offset: MAXLNUM as ::core::ffi::c_int
                                                    as linenr_T,
                                                skipcol: 0 as colnr_T,
                                            },
                                            additional_data: cur_entry.additional_data,
                                        };
                                        if cur_entry.type_0 as ::core::ffi::c_int
                                            == kSDItemLocalMark as ::core::ffi::c_int
                                        {
                                            if !mark_set_local(
                                                cur_entry.data.filemark.name,
                                                buf_1,
                                                fm_0,
                                                !force,
                                            ) {
                                                shada_free_shada_entry(&raw mut cur_entry);
                                                break 's_732;
                                            }
                                        } else {
                                            set_put_ptr_t(
                                                &raw mut cl_bufs,
                                                buf_1 as ptr_t,
                                                ::core::ptr::null_mut::<*mut ptr_t>(),
                                            );
                                            let mut i_2: ::core::ffi::c_int = 0;
                                            i_2 = (*buf_1).b_changelistlen;
                                            while i_2 > 0 as ::core::ffi::c_int {
                                                let jl_entry_0: fmark_T = (*buf_1).b_changelist
                                                    [(i_2 - 1 as ::core::ffi::c_int) as usize];
                                                if jl_entry_0.timestamp <= cur_entry.timestamp {
                                                    if marks_equal(
                                                        jl_entry_0.mark,
                                                        cur_entry.data.filemark.mark,
                                                    ) {
                                                        i_2 = -1 as ::core::ffi::c_int;
                                                    }
                                                    break;
                                                } else {
                                                    i_2 -= 1;
                                                }
                                            }
                                            if i_2 > 0 as ::core::ffi::c_int
                                                && (*buf_1).b_changelistlen == JUMPLISTSIZE
                                            {
                                                free_fmark(
                                                    (*buf_1).b_changelist
                                                        [0 as ::core::ffi::c_int as usize],
                                                );
                                            }
                                            i_2 = marklist_insert(
                                                &raw mut (*buf_1).b_changelist as *mut fmark_T
                                                    as *mut ::core::ffi::c_void,
                                                ::core::mem::size_of::<fmark_T>(),
                                                (*buf_1).b_changelistlen,
                                                i_2,
                                            );
                                            if i_2 != -1 as ::core::ffi::c_int {
                                                (*buf_1).b_changelist[i_2 as usize] = fm_0;
                                                if (*buf_1).b_changelistlen < JUMPLISTSIZE {
                                                    (*buf_1).b_changelistlen += 1;
                                                }
                                            } else {
                                                xfree(
                                                    fm_0.additional_data
                                                        as *mut ::core::ffi::c_void,
                                                );
                                            }
                                        }
                                        xfree(
                                            cur_entry.data.filemark.fname
                                                as *mut ::core::ffi::c_void,
                                        );
                                    }
                                }
                            }
                            -1 | _ => {}
                        }
                    }
                }
            }
        }
        if srni_flags & kSDReadHistory as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            let mut i_3: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_3 < HIST_COUNT as ::core::ffi::c_int {
                hms_insert_whole_neovim_history(
                    (&raw mut hms as *mut HistoryMergerState).offset(i_3 as isize),
                );
                hms_to_history((&raw mut hms as *mut HistoryMergerState).offset(i_3 as isize));
                hms_dealloc((&raw mut hms as *mut HistoryMergerState).offset(i_3 as isize));
                i_3 += 1;
            }
        }
        if cl_bufs.h.n_occupied != 0 {
            let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
            while !tp.is_null() {
                let mut wp: *mut win_T = if tp == curtab.get() {
                    firstwin.get()
                } else {
                    (*tp).tp_firstwin
                };
                while !wp.is_null() {
                    if set_has_ptr_t(&raw mut cl_bufs, (*wp).w_buffer as ptr_t) {
                        (*wp).w_changelistidx = (*(*wp).w_buffer).b_changelistlen;
                    }
                    wp = (*wp).w_next;
                }
                tp = (*tp).tp_next as *mut tabpage_T;
            }
        }
        xfree(cl_bufs.keys as *mut ::core::ffi::c_void);
        xfree(cl_bufs.h.hash as *mut ::core::ffi::c_void);
        cl_bufs = Set_ptr_t {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<ptr_t>(),
        };
        let mut key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut __i: uint32_t = 0;
        __i = 0 as uint32_t;
        while __i < fname_bufs.set.h.n_keys {
            key = *fname_bufs.set.keys.offset(__i as isize) as *const ::core::ffi::c_char;
            xfree(key as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void);
            __i = __i.wrapping_add(1);
        }
        xfree(fname_bufs.set.keys as *mut ::core::ffi::c_void);
        xfree(fname_bufs.set.h.hash as *mut ::core::ffi::c_void);
        fname_bufs.set = Set_cstr_t {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<cstr_t>(),
        };
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut fname_bufs.values as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL_0;
        let _ = *ptr__0;
        xfree(oldfiles_set.keys as *mut ::core::ffi::c_void);
        xfree(oldfiles_set.h.hash as *mut ::core::ffi::c_void);
        oldfiles_set = Set_cstr_t {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<cstr_t>(),
        };
    }
}

pub(crate) unsafe extern "C" fn shada_free_shada_entry(entry: *mut ShadaEntry) {
    unsafe {
        if entry.is_null() || !(*entry).can_free_entry {
            return;
        }
        match (*entry).type_0 as ::core::ffi::c_int {
            -1 => {
                xfree((*entry).data.unknown_item.contents as *mut ::core::ffi::c_void);
            }
            1 => {
                api_free_dict((*entry).data.header);
            }
            11 | 8 | 7 | 10 => {
                xfree((*entry).data.filemark.fname as *mut ::core::ffi::c_void);
            }
            2 => {
                api_free_string((*entry).data.search_pattern.pat);
            }
            5 => {
                let mut i: size_t = 0 as size_t;
                while i < (*entry).data.reg.contents_size {
                    api_free_string(*(*entry).data.reg.contents.offset(i as isize));
                    i = i.wrapping_add(1);
                }
                xfree((*entry).data.reg.contents as *mut ::core::ffi::c_void);
            }
            4 => {
                xfree((*entry).data.history_item.string as *mut ::core::ffi::c_void);
            }
            6 => {
                xfree((*entry).data.global_var.name as *mut ::core::ffi::c_void);
                tv_clear(&raw mut (*entry).data.global_var.value);
            }
            3 => {
                xfree((*entry).data.sub_string.sub as *mut ::core::ffi::c_void);
            }
            9 => {
                let mut i_0: size_t = 0 as size_t;
                while i_0 < (*entry).data.buffer_list.size {
                    xfree(
                        (*(*entry).data.buffer_list.buffers.offset(i_0 as isize)).fname
                            as *mut ::core::ffi::c_void,
                    );
                    xfree(
                        (*(*entry).data.buffer_list.buffers.offset(i_0 as isize)).additional_data
                            as *mut ::core::ffi::c_void,
                    );
                    i_0 = i_0.wrapping_add(1);
                }
                xfree((*entry).data.buffer_list.buffers as *mut ::core::ffi::c_void);
            }
            0 | _ => {}
        }
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*entry).additional_data as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
    }
}

pub unsafe extern "C" fn shada_read_string(mut string: String_0, flags: ::core::ffi::c_int) {
    unsafe {
        if string.size == 0 as size_t {
            return;
        }
        let mut sd_reader: FileDescriptor = FileDescriptor {
            fd: 0,
            buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            wr: false,
            eof: false,
            non_blocking: false,
            bytes_read: 0,
        };
        file_open_buffer(&raw mut sd_reader, string.data, string.size);
        shada_read(&raw mut sd_reader, flags);
        close_file(&raw mut sd_reader);
    }
}
