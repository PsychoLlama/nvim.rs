//! Merging what is already in the file with what Nvim holds.
//!
//! Writing a ShaDa file preserves entries the running Nvim has no opinion
//! about, and merges the ones it does. Two structures do that:
//!
//! * `HMLList` — a fixed-size ring of history entries in a doubly linked
//!   list, so the merger can hold the newest N of a history and drop the
//!   oldest in constant time. `HistoryMergerState` wraps one per history type
//!   and remembers which of Nvim's own entries have been folded in yet.
//! * the file-mark lists, which `marklist_insert` keeps sorted by timestamp
//!   with the same "keep the newest N" bound.
//!
//! `shada_read_when_writing` is the pass over the existing file that feeds
//! both of them, and holds on to everything it decides to copy through.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

#[inline]
pub(crate) unsafe extern "C" fn hmll_init(hmll: *mut HMLList, size: size_t) {
    unsafe {
        *hmll = HMLList {
            entries: xcalloc(size, ::core::mem::size_of::<HMLListEntry>()) as *mut HMLListEntry,
            first: ::core::ptr::null_mut::<HMLListEntry>(),
            last: ::core::ptr::null_mut::<HMLListEntry>(),
            free_entry: ::core::ptr::null_mut::<HMLListEntry>(),
            last_free_entry: ::core::ptr::null_mut::<HMLListEntry>(),
            size: size,
            num_entries: 0 as size_t,
            contained_entries: MAP_INIT,
        };
        (*hmll).last_free_entry = (*hmll).entries;
    }
}

#[inline]
pub(crate) unsafe extern "C" fn hmll_remove(hmll: *mut HMLList, hmll_entry: *mut HMLListEntry) {
    unsafe {
        if hmll_entry
            == (*hmll)
                .last_free_entry
                .offset(-(1 as ::core::ffi::c_int as isize))
        {
            (*hmll).last_free_entry = (*hmll).last_free_entry.offset(-1);
        } else {
            '_c2rust_label: {
                if (*hmll).free_entry.is_null() {
                } else {
                    __assert_fail(
                        b"hmll->free_entry == NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/shada.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        449 as ::core::ffi::c_uint,
                        b"void hmll_remove(HMLList *const, HMLListEntry *const)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            (*hmll).free_entry = hmll_entry;
        }
        let mut val: ptr_t = map_del_cstr_t_ptr_t(
            &raw mut (*hmll).contained_entries,
            (*hmll_entry).data.data.history_item.string as cstr_t,
            ::core::ptr::null_mut::<cstr_t>(),
        );
        '_c2rust_label_0: {
            if !val.is_null() {
            } else {
                __assert_fail(
                    b"val\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/shada.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    454 as ::core::ffi::c_uint,
                    b"void hmll_remove(HMLList *const, HMLListEntry *const)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if (*hmll_entry).next.is_null() {
            (*hmll).last = (*hmll_entry).prev as *mut HMLListEntry;
        } else {
            (*(*hmll_entry).next).prev = (*hmll_entry).prev;
        }
        if (*hmll_entry).prev.is_null() {
            (*hmll).first = (*hmll_entry).next as *mut HMLListEntry;
        } else {
            (*(*hmll_entry).prev).next = (*hmll_entry).next;
        }
        (*hmll).num_entries = (*hmll).num_entries.wrapping_sub(1);
        shada_free_shada_entry(&raw mut (*hmll_entry).data);
    }
}

#[inline]
pub(crate) unsafe extern "C" fn hmll_insert(
    hmll: *mut HMLList,
    mut hmll_entry: *mut HMLListEntry,
    data: ShadaEntry,
) {
    unsafe {
        if (*hmll).num_entries == (*hmll).size {
            if hmll_entry == (*hmll).first {
                hmll_entry = ::core::ptr::null_mut::<HMLListEntry>();
            }
            '_c2rust_label: {
                if !(*hmll).first.is_null() {
                } else {
                    __assert_fail(
                        b"hmll->first != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/shada.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        484 as ::core::ffi::c_uint,
                        b"void hmll_insert(HMLList *const, HMLListEntry *, const ShadaEntry)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            hmll_remove(hmll, (*hmll).first);
        }
        let mut target_entry: *mut HMLListEntry = ::core::ptr::null_mut::<HMLListEntry>();
        if (*hmll).free_entry.is_null() {
            '_c2rust_label_0: {
                if (*hmll).last_free_entry.offset_from((*hmll).entries) as size_t
                    == (*hmll).num_entries
                {
                } else {
                    __assert_fail(
                        b"(size_t)(hmll->last_free_entry - hmll->entries) == hmll->num_entries\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/shada.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        490 as ::core::ffi::c_uint,
                        b"void hmll_insert(HMLList *const, HMLListEntry *, const ShadaEntry)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            let c2rust_fresh20 = (*hmll).last_free_entry;
            (*hmll).last_free_entry = (*hmll).last_free_entry.offset(1);
            target_entry = c2rust_fresh20;
        } else {
            '_c2rust_label_1: {
                if ((*hmll).last_free_entry.offset_from((*hmll).entries) as size_t)
                    .wrapping_sub(1 as size_t)
                    == (*hmll).num_entries
                {
                } else {
                    __assert_fail(
                    b"(size_t)(hmll->last_free_entry - hmll->entries) - 1 == hmll->num_entries\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/shada.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    494 as ::core::ffi::c_uint,
                    b"void hmll_insert(HMLList *const, HMLListEntry *, const ShadaEntry)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
                }
            };
            target_entry = (*hmll).free_entry;
            (*hmll).free_entry = ::core::ptr::null_mut::<HMLListEntry>();
        }
        (*target_entry).data = data;
        let mut new_item: bool = false_0 != 0;
        let mut val: *mut ptr_t = map_put_ref_cstr_t_ptr_t(
            &raw mut (*hmll).contained_entries,
            data.data.history_item.string as cstr_t,
            ::core::ptr::null_mut::<*mut cstr_t>(),
            &raw mut new_item,
        );
        if new_item {
            *val = target_entry as ptr_t;
        }
        (*hmll).num_entries = (*hmll).num_entries.wrapping_add(1);
        (*target_entry).prev = hmll_entry as *mut hm_llist_entry;
        if hmll_entry.is_null() {
            (*target_entry).next = (*hmll).first as *mut hm_llist_entry;
            (*hmll).first = target_entry;
        } else {
            (*target_entry).next = (*hmll_entry).next;
            (*hmll_entry).next = target_entry as *mut hm_llist_entry;
        }
        if (*target_entry).next.is_null() {
            (*hmll).last = target_entry;
        } else {
            (*(*target_entry).next).prev = target_entry as *mut hm_llist_entry;
        };
    }
}

#[inline]
pub(crate) unsafe extern "C" fn hmll_dealloc(hmll: *mut HMLList) {
    unsafe {
        xfree((*hmll).contained_entries.set.keys as *mut ::core::ffi::c_void);
        xfree((*hmll).contained_entries.set.h.hash as *mut ::core::ffi::c_void);
        (*hmll).contained_entries.set = Set_cstr_t {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<cstr_t>(),
        };
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*hmll).contained_entries.values as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
        xfree((*hmll).entries as *mut ::core::ffi::c_void);
    }
}

/// Snapshot neovim's own history of `hms_p`'s type into `pending`, oldest
/// first. When reading (merge-back), the entries are moved out of the
/// history rings and owned here; when writing, they are borrowed
/// (`can_free_entry` is false and no free path touches them).
pub(crate) unsafe extern "C" fn hms_load_pending(hms_p: *mut HistoryMergerState) {
    unsafe {
        let history_type = (*hms_p).history_type;
        let entries = if (*hms_p).reading {
            hist_shada_take(history_type as ::core::ffi::c_int)
        } else {
            hist_shada_view(history_type as ::core::ffi::c_int)
        };
        let pending: Box<[ShadaEntry]> = entries
            .into_iter()
            .map(|he| ShadaEntry {
                type_0: kSDItemHistoryEntry,
                can_free_entry: (*hms_p).reading,
                timestamp: he.timestamp,
                data: C2Rust_Unnamed_22 {
                    history_item: history_item {
                        histtype: history_type,
                        string: he.text,
                        sep: he.sep,
                    },
                },
                additional_data: he.additional_data,
            })
            .collect();
        (*hms_p).pending_len = pending.len() as size_t;
        (*hms_p).pending_pos = 0 as size_t;
        (*hms_p).pending = Box::into_raw(pending) as *mut ShadaEntry;
    }
}

pub(crate) unsafe extern "C" fn hms_insert(
    hms_p: *mut HistoryMergerState,
    entry: ShadaEntry,
    do_iter: bool,
) {
    unsafe {
        if do_iter {
            while (*hms_p).pending_pos < (*hms_p).pending_len {
                let next: ShadaEntry = *(*hms_p).pending.add((*hms_p).pending_pos as usize);
                if next.timestamp >= entry.timestamp {
                    break;
                }
                (*hms_p).pending_pos = (*hms_p).pending_pos.wrapping_add(1);
                hms_insert(hms_p, next, false_0 != 0);
            }
        }
        let hmll: *mut HMLList = &raw mut (*hms_p).hmll;
        let mut key_alloc: *mut cstr_t = ::core::ptr::null_mut::<cstr_t>();
        let mut val: *mut ptr_t = map_ref_cstr_t_ptr_t(
            &raw mut (*hms_p).hmll.contained_entries,
            entry.data.history_item.string as cstr_t,
            &raw mut key_alloc,
        );
        if !val.is_null() {
            let existing_entry: *mut HMLListEntry = *val as *mut HMLListEntry;
            if entry.timestamp > (*existing_entry).data.timestamp {
                hmll_remove(hmll, existing_entry);
            } else if !do_iter && entry.timestamp == (*existing_entry).data.timestamp {
                shada_free_shada_entry(&raw mut (*existing_entry).data);
                (*existing_entry).data = entry;
                *key_alloc = entry.data.history_item.string as cstr_t;
                return;
            } else {
                return;
            }
        }
        let mut insert_after: *mut HMLListEntry = ::core::ptr::null_mut::<HMLListEntry>();
        insert_after = (*hmll).last;
        while !insert_after.is_null() {
            if (*insert_after).data.timestamp <= entry.timestamp {
                break;
            }
            insert_after = (*insert_after).prev as *mut HMLListEntry;
        }
        hmll_insert(hmll, insert_after, entry);
    }
}

#[inline]
pub(crate) unsafe extern "C" fn hms_init(
    hms_p: *mut HistoryMergerState,
    history_type: uint8_t,
    num_elements: size_t,
    do_merge: bool,
    reading: bool,
) {
    unsafe {
        hmll_init(&raw mut (*hms_p).hmll, num_elements);
        (*hms_p).do_merge = do_merge;
        (*hms_p).reading = reading;
        (*hms_p).history_type = history_type;
        hms_load_pending(hms_p);
    }
}

#[inline]
pub(crate) unsafe extern "C" fn hms_insert_whole_neovim_history(hms_p: *mut HistoryMergerState) {
    unsafe {
        while (*hms_p).pending_pos < (*hms_p).pending_len {
            let next: ShadaEntry = *(*hms_p).pending.add((*hms_p).pending_pos as usize);
            (*hms_p).pending_pos = (*hms_p).pending_pos.wrapping_add(1);
            hms_insert(hms_p, next, false_0 != 0);
        }
    }
}

/// Hand the merged entries in the HMLL back to cmdhist, oldest first.
/// String and additional-data ownership transfers to cmdhist; the HMLL
/// scaffolding is still deallocated by [`hms_dealloc`] afterwards (which
/// never frees entry payloads).
#[inline]
pub(crate) unsafe extern "C" fn hms_to_history(hms_p: *const HistoryMergerState) {
    unsafe {
        let mut merged: Vec<HistShadaEntry> = Vec::new();
        let mut cur_entry: *mut HMLListEntry = (*hms_p).hmll.first as *mut HMLListEntry;
        while !cur_entry.is_null() {
            merged.push(HistShadaEntry {
                text: (*cur_entry).data.data.history_item.string,
                sep: (*cur_entry).data.data.history_item.sep,
                timestamp: (*cur_entry).data.timestamp,
                additional_data: (*cur_entry).data.additional_data,
            });
            cur_entry = (*cur_entry).next as *mut HMLListEntry;
        }
        hist_shada_replace((*hms_p).history_type as ::core::ffi::c_int, merged);
    }
}

#[inline]
pub(crate) unsafe extern "C" fn hms_dealloc(hms_p: *mut HistoryMergerState) {
    unsafe {
        // Free whatever part of the neovim-history snapshot was never merged
        // (only owned on the reading path; `shada_free_shada_entry` checks
        // `can_free_entry` itself).
        while (*hms_p).pending_pos < (*hms_p).pending_len {
            let mut entry: ShadaEntry = *(*hms_p).pending.add((*hms_p).pending_pos as usize);
            shada_free_shada_entry(&raw mut entry);
            (*hms_p).pending_pos = (*hms_p).pending_pos.wrapping_add(1);
        }
        if !(*hms_p).pending.is_null() {
            drop(Box::from_raw(core::ptr::slice_from_raw_parts_mut(
                (*hms_p).pending,
                (*hms_p).pending_len as usize,
            )));
            (*hms_p).pending = ::core::ptr::null_mut::<ShadaEntry>();
            (*hms_p).pending_len = 0 as size_t;
        }
        hmll_dealloc(&raw mut (*hms_p).hmll);
    }
}

#[inline]
pub(crate) unsafe extern "C" fn marks_equal(a: pos_T, b: pos_T) -> bool {
    return a.lnum == b.lnum && a.col == b.col;
}

pub(crate) unsafe extern "C" fn marklist_insert(
    mut jumps_arr: *mut ::core::ffi::c_void,
    mut jump_size: size_t,
    mut jl_len: ::core::ffi::c_int,
    mut i: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut jumps: *mut ::core::ffi::c_char = jumps_arr as *mut ::core::ffi::c_char;
        if i > 0 as ::core::ffi::c_int {
            if jl_len == JUMPLISTSIZE {
                i -= 1;
                if i > 0 as ::core::ffi::c_int {
                    memmove(
                        jumps as *mut ::core::ffi::c_void,
                        jumps.offset(jump_size as isize) as *const ::core::ffi::c_void,
                        jump_size.wrapping_mul(i as size_t),
                    );
                }
            } else if i != jl_len {
                memmove(
                    jumps.offset(
                        ((i + 1 as ::core::ffi::c_int) as size_t).wrapping_mul(jump_size) as isize,
                    ) as *mut ::core::ffi::c_void,
                    jumps.offset((i as size_t).wrapping_mul(jump_size) as isize)
                        as *const ::core::ffi::c_void,
                    jump_size.wrapping_mul((jl_len - i) as size_t),
                );
            }
        } else if i == 0 as ::core::ffi::c_int {
            if jl_len == JUMPLISTSIZE {
                return -1 as ::core::ffi::c_int;
            } else if jl_len > 0 as ::core::ffi::c_int {
                memmove(
                    jumps.offset(jump_size as isize) as *mut ::core::ffi::c_void,
                    jumps as *const ::core::ffi::c_void,
                    jump_size.wrapping_mul(jl_len as size_t),
                );
            }
        }
        return i;
    }
}

pub(crate) unsafe extern "C" fn compare_file_marks(
    mut a: *const ::core::ffi::c_void,
    mut b: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        let a_fms: *const *const FileMarks = a as *const *const FileMarks;
        let b_fms: *const *const FileMarks = b as *const *const FileMarks;
        return if (**a_fms).greatest_timestamp == (**b_fms).greatest_timestamp {
            0 as ::core::ffi::c_int
        } else if (**a_fms).greatest_timestamp > (**b_fms).greatest_timestamp {
            -1 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        };
    }
}

#[inline]
pub(crate) unsafe extern "C" fn shada_read_when_writing(
    sd_reader: *mut FileDescriptor,
    srni_flags: ::core::ffi::c_uint,
    max_kbyte: size_t,
    wms: *mut WriteMergerState,
    packer: *mut PackerBuffer,
) -> ShaDaWriteResult {
    unsafe {
        let mut ret: ShaDaWriteResult = kSDWriteSuccessful;
        let mut entry: ShadaEntry = ShadaEntry {
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
        let mut srni_ret: ShaDaReadResult = kSDReadStatusSuccess;
        loop {
            srni_ret = shada_read_next_item(sd_reader, &raw mut entry, srni_flags, max_kbyte);
            if srni_ret as ::core::ffi::c_uint
                == kSDReadStatusFinished as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                break;
            }
            match srni_ret as ::core::ffi::c_uint {
                1 => {
                    abort();
                }
                3 => {
                    ret = kSDWriteReadNotShada;
                }
                2 => {}
                4 => {
                    continue;
                }
                0 | _ => {
                    's_781: {
                        match entry.type_0 as ::core::ffi::c_int {
                            1 | 9 => {
                                abort();
                            }
                            -1 => {
                                ret = shada_pack_entry(packer, entry, 0 as size_t);
                                shada_free_shada_entry(&raw mut entry);
                            }
                            2 => {
                                let wms_entry: *mut ShadaEntry =
                                    if entry.data.search_pattern.is_substitute_pattern
                                        as ::core::ffi::c_int
                                        != 0
                                    {
                                        &raw mut (*wms).sub_search_pattern
                                    } else {
                                        &raw mut (*wms).search_pattern
                                    };
                                's_94: {
                                    if (*wms_entry).type_0 as ::core::ffi::c_int
                                        != kSDItemMissing as ::core::ffi::c_int
                                    {
                                        if (*wms_entry).timestamp >= entry.timestamp {
                                            shada_free_shada_entry(&raw mut entry);
                                            break 's_94;
                                        } else {
                                            shada_free_shada_entry(wms_entry);
                                        }
                                    }
                                    *wms_entry = entry;
                                }
                            }
                            3 => {
                                let wms_entry_0: *mut ShadaEntry = &raw mut (*wms).replacement;
                                's_132: {
                                    if (*wms_entry_0).type_0 as ::core::ffi::c_int
                                        != kSDItemMissing as ::core::ffi::c_int
                                    {
                                        if (*wms_entry_0).timestamp >= entry.timestamp {
                                            shada_free_shada_entry(&raw mut entry);
                                            break 's_132;
                                        } else {
                                            shada_free_shada_entry(wms_entry_0);
                                        }
                                    }
                                    *wms_entry_0 = entry;
                                }
                            }
                            4 => {
                                if entry.data.history_item.histtype as ::core::ffi::c_int
                                    >= HIST_COUNT as ::core::ffi::c_int
                                {
                                    ret = shada_pack_entry(packer, entry, 0 as size_t);
                                    shada_free_shada_entry(&raw mut entry);
                                } else if (*wms).hms[entry.data.history_item.histtype as usize]
                                    .hmll
                                    .size
                                    != 0 as size_t
                                {
                                    hms_insert(
                                        (&raw mut (*wms).hms as *mut HistoryMergerState)
                                            .offset(entry.data.history_item.histtype as isize),
                                        entry,
                                        true_0 != 0,
                                    );
                                } else {
                                    shada_free_shada_entry(&raw mut entry);
                                }
                            }
                            5 => {
                                let idx: ::core::ffi::c_int =
                                    op_reg_index(entry.data.reg.name as ::core::ffi::c_int);
                                if idx < 0 as ::core::ffi::c_int {
                                    ret = shada_pack_entry(packer, entry, 0 as size_t);
                                    shada_free_shada_entry(&raw mut entry);
                                } else {
                                    let wms_entry_1: *mut ShadaEntry = (&raw mut (*wms).registers
                                        as *mut ShadaEntry)
                                        .offset(idx as isize);
                                    's_223: {
                                        if (*wms_entry_1).type_0 as ::core::ffi::c_int
                                            != kSDItemMissing as ::core::ffi::c_int
                                        {
                                            if (*wms_entry_1).timestamp >= entry.timestamp {
                                                shada_free_shada_entry(&raw mut entry);
                                                break 's_223;
                                            } else {
                                                shada_free_shada_entry(wms_entry_1);
                                            }
                                        }
                                        *wms_entry_1 = entry;
                                    }
                                }
                            }
                            6 => {
                                if !set_has_cstr_t(
                                    &raw mut (*wms).dumped_variables,
                                    entry.data.global_var.name as cstr_t,
                                ) {
                                    ret = shada_pack_entry(packer, entry, 0 as size_t);
                                }
                                shada_free_shada_entry(&raw mut entry);
                            }
                            7 => {
                                if ascii_isdigit(entry.data.filemark.name as ::core::ffi::c_int) {
                                    let mut processed_mark: bool = false_0 != 0;
                                    let mut i: size_t = ::core::mem::size_of::<[ShadaEntry; 10]>()
                                        .wrapping_div(::core::mem::size_of::<ShadaEntry>())
                                        .wrapping_div(
                                            (::core::mem::size_of::<[ShadaEntry; 10]>()
                                                .wrapping_rem(::core::mem::size_of::<ShadaEntry>())
                                                == 0)
                                                as ::core::ffi::c_int
                                                as size_t,
                                        );
                                    while i > 0 as size_t {
                                        let mut wms_entry_2: ShadaEntry = (*wms).numbered_marks
                                            [i.wrapping_sub(1 as size_t) as usize];
                                        if wms_entry_2.type_0 as ::core::ffi::c_int
                                            == kSDItemGlobalMark as ::core::ffi::c_int
                                        {
                                            if wms_entry_2.timestamp == entry.timestamp
                                                && (wms_entry_2.additional_data.is_null()
                                                    && entry.additional_data.is_null())
                                                && marks_equal(
                                                    wms_entry_2.data.filemark.mark,
                                                    entry.data.filemark.mark,
                                                )
                                                    as ::core::ffi::c_int
                                                    != 0
                                                && strcmp(
                                                    wms_entry_2.data.filemark.fname,
                                                    entry.data.filemark.fname,
                                                ) == 0 as ::core::ffi::c_int
                                            {
                                                shada_free_shada_entry(&raw mut entry);
                                                processed_mark = true_0 != 0;
                                                break;
                                            } else if wms_entry_2.timestamp >= entry.timestamp {
                                                processed_mark = true_0 != 0;
                                                if i < ::core::mem::size_of::<[ShadaEntry; 10]>()
                                                    .wrapping_div(
                                                        ::core::mem::size_of::<ShadaEntry>(),
                                                    )
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<[ShadaEntry; 10]>()
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                ShadaEntry,
                                                            >(
                                                            ))
                                                            == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    )
                                                {
                                                    replace_numbered_mark(wms, i, entry);
                                                } else {
                                                    shada_free_shada_entry(&raw mut entry);
                                                }
                                                break;
                                            }
                                        }
                                        i = i.wrapping_sub(1);
                                    }
                                    if !processed_mark {
                                        replace_numbered_mark(wms, 0 as size_t, entry);
                                    }
                                } else {
                                    let idx_0: ::core::ffi::c_int =
                                        mark_global_index(entry.data.filemark.name);
                                    if idx_0 < 0 as ::core::ffi::c_int {
                                        ret = shada_pack_entry(packer, entry, 0 as size_t);
                                        shada_free_shada_entry(&raw mut entry);
                                    } else {
                                        let mut mark: *mut ShadaEntry = if idx_0
                                            < 26 as ::core::ffi::c_int
                                        {
                                            (&raw mut (*wms).global_marks as *mut ShadaEntry)
                                                .offset(idx_0 as isize)
                                        } else {
                                            (&raw mut (*wms).numbered_marks as *mut ShadaEntry)
                                                .offset((idx_0 - 26 as ::core::ffi::c_int) as isize)
                                        };
                                        if (*mark).type_0 as ::core::ffi::c_int
                                            == kSDItemMissing as ::core::ffi::c_int
                                        {
                                            if (*namedfm.ptr())[idx_0 as usize].fmark.timestamp
                                                >= entry.timestamp
                                            {
                                                shada_free_shada_entry(&raw mut entry);
                                                break 's_781;
                                            }
                                        }
                                        let wms_entry_3: *mut ShadaEntry = mark;
                                        's_401: {
                                            if (*wms_entry_3).type_0 as ::core::ffi::c_int
                                                != kSDItemMissing as ::core::ffi::c_int
                                            {
                                                if (*wms_entry_3).timestamp >= entry.timestamp {
                                                    shada_free_shada_entry(&raw mut entry);
                                                    break 's_401;
                                                } else {
                                                    shada_free_shada_entry(wms_entry_3);
                                                }
                                            }
                                            *wms_entry_3 = entry;
                                        }
                                    }
                                }
                            }
                            11 | 10 => {
                                if shada_removable(entry.data.filemark.fname) {
                                    shada_free_shada_entry(&raw mut entry);
                                } else {
                                    let fname: *const ::core::ffi::c_char =
                                        entry.data.filemark.fname;
                                    let mut key: *mut cstr_t = ::core::ptr::null_mut::<cstr_t>();
                                    let mut new_item: bool = false_0 != 0;
                                    let mut val: *mut ptr_t = map_put_ref_cstr_t_ptr_t(
                                        &raw mut (*wms).file_marks,
                                        fname as cstr_t,
                                        &raw mut key,
                                        &raw mut new_item,
                                    );
                                    if new_item {
                                        *key = xstrdup(fname) as cstr_t;
                                    }
                                    if (*val).is_null() {
                                        *val = xcalloc(
                                            1 as size_t,
                                            ::core::mem::size_of::<FileMarks>(),
                                        ) as ptr_t;
                                    }
                                    let filemarks: *mut FileMarks = *val as *mut FileMarks;
                                    if entry.timestamp > (*filemarks).greatest_timestamp {
                                        (*filemarks).greatest_timestamp = entry.timestamp;
                                    }
                                    if entry.type_0 as ::core::ffi::c_int
                                        == kSDItemLocalMark as ::core::ffi::c_int
                                    {
                                        let idx_1: ::core::ffi::c_int =
                                            mark_local_index(entry.data.filemark.name);
                                        if idx_1 < 0 as ::core::ffi::c_int {
                                            (*filemarks).additional_marks_size =
                                                (*filemarks).additional_marks_size.wrapping_add(1);
                                            (*filemarks).additional_marks = xrealloc(
                                                (*filemarks).additional_marks
                                                    as *mut ::core::ffi::c_void,
                                                (*filemarks).additional_marks_size.wrapping_mul(
                                                    ::core::mem::size_of::<ShadaEntry>(),
                                                ),
                                            )
                                                as *mut ShadaEntry;
                                            *(*filemarks).additional_marks.offset(
                                                (*filemarks)
                                                    .additional_marks_size
                                                    .wrapping_sub(1 as size_t)
                                                    as isize,
                                            ) = entry;
                                        } else {
                                            let wms_entry_4: *mut ShadaEntry =
                                                (&raw mut (*filemarks).marks as *mut ShadaEntry)
                                                    .offset(idx_1 as isize);
                                            let mut set_wms: bool = true_0 != 0;
                                            if (*wms_entry_4).type_0 as ::core::ffi::c_int
                                                != kSDItemMissing as ::core::ffi::c_int
                                            {
                                                if (*wms_entry_4).timestamp >= entry.timestamp {
                                                    shada_free_shada_entry(&raw mut entry);
                                                    break 's_781;
                                                } else if (*wms_entry_4).can_free_entry {
                                                    if *key
                                                        == (*wms_entry_4).data.filemark.fname
                                                            as cstr_t
                                                    {
                                                        *key = entry.data.filemark.fname as cstr_t;
                                                    }
                                                    shada_free_shada_entry(wms_entry_4);
                                                }
                                            } else {
                                                let mut buf: *mut buf_T = firstbuf.get();
                                                while !buf.is_null() {
                                                    if !(*buf).b_ffname.is_null()
                                                        && path_fnamecmp(
                                                            entry.data.filemark.fname,
                                                            (*buf).b_ffname,
                                                        ) == 0 as ::core::ffi::c_int
                                                    {
                                                        let mut fm: fmark_T = fmark_T {
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
                                                            additional_data: ::core::ptr::null_mut::<
                                                                AdditionalData,
                                                            >(
                                                            ),
                                                        };
                                                        mark_get(
                                                            buf,
                                                            curwin.get(),
                                                            &raw mut fm,
                                                            kMarkBufLocal,
                                                            entry.data.filemark.name
                                                                as ::core::ffi::c_int,
                                                        );
                                                        if fm.timestamp >= entry.timestamp {
                                                            set_wms = false_0 != 0;
                                                            shada_free_shada_entry(&raw mut entry);
                                                            break;
                                                        }
                                                    }
                                                    buf = (*buf).b_next;
                                                }
                                            }
                                            if set_wms {
                                                *wms_entry_4 = entry;
                                            }
                                        }
                                    } else {
                                        let mut i_0: ::core::ffi::c_int = 0;
                                        i_0 = (*filemarks).changes_size as ::core::ffi::c_int;
                                        while i_0 > 0 as ::core::ffi::c_int {
                                            let jl_entry: ShadaEntry = (*filemarks).changes
                                                [(i_0 - 1 as ::core::ffi::c_int) as usize];
                                            if jl_entry.timestamp <= entry.timestamp {
                                                if marks_equal(
                                                    jl_entry.data.filemark.mark,
                                                    entry.data.filemark.mark,
                                                ) {
                                                    i_0 = -1 as ::core::ffi::c_int;
                                                }
                                                break;
                                            } else {
                                                i_0 -= 1;
                                            }
                                        }
                                        if i_0 > 0 as ::core::ffi::c_int
                                            && (*filemarks).changes_size == JUMPLISTSIZE as size_t
                                        {
                                            shada_free_shada_entry(
                                                (&raw mut (*filemarks).changes as *mut ShadaEntry)
                                                    .offset(0 as ::core::ffi::c_int as isize),
                                            );
                                        }
                                        i_0 = marklist_insert(
                                            &raw mut (*filemarks).changes as *mut ShadaEntry
                                                as *mut ::core::ffi::c_void,
                                            ::core::mem::size_of::<ShadaEntry>(),
                                            (*filemarks).changes_size as ::core::ffi::c_int,
                                            i_0,
                                        );
                                        if i_0 != -1 as ::core::ffi::c_int {
                                            (*filemarks).changes[i_0 as usize] = entry;
                                            if (*filemarks).changes_size < JUMPLISTSIZE as size_t {
                                                (*filemarks).changes_size =
                                                    (*filemarks).changes_size.wrapping_add(1);
                                            }
                                        } else {
                                            shada_free_shada_entry(&raw mut entry);
                                        }
                                    }
                                }
                            }
                            8 => {
                                let mut i_1: ::core::ffi::c_int = 0;
                                i_1 = (*wms).jumps_size as ::core::ffi::c_int;
                                while i_1 > 0 as ::core::ffi::c_int {
                                    let jl_entry_0: ShadaEntry =
                                        (*wms).jumps[(i_1 - 1 as ::core::ffi::c_int) as usize];
                                    if jl_entry_0.timestamp <= entry.timestamp {
                                        if marks_equal(
                                            jl_entry_0.data.filemark.mark,
                                            entry.data.filemark.mark,
                                        )
                                            as ::core::ffi::c_int
                                            != 0
                                            && strcmp(
                                                jl_entry_0.data.filemark.fname,
                                                entry.data.filemark.fname,
                                            ) == 0 as ::core::ffi::c_int
                                        {
                                            i_1 = -1 as ::core::ffi::c_int;
                                        }
                                        break;
                                    } else {
                                        i_1 -= 1;
                                    }
                                }
                                if i_1 > 0 as ::core::ffi::c_int
                                    && (*wms).jumps_size == JUMPLISTSIZE as size_t
                                {
                                    shada_free_shada_entry(
                                        (&raw mut (*wms).jumps as *mut ShadaEntry)
                                            .offset(0 as ::core::ffi::c_int as isize),
                                    );
                                }
                                i_1 = marklist_insert(
                                    &raw mut (*wms).jumps as *mut ShadaEntry
                                        as *mut ::core::ffi::c_void,
                                    ::core::mem::size_of::<ShadaEntry>(),
                                    (*wms).jumps_size as ::core::ffi::c_int,
                                    i_1,
                                );
                                if i_1 != -1 as ::core::ffi::c_int {
                                    (*wms).jumps[i_1 as usize] = entry;
                                    if (*wms).jumps_size < JUMPLISTSIZE as size_t {
                                        (*wms).jumps_size = (*wms).jumps_size.wrapping_add(1);
                                    }
                                } else {
                                    shada_free_shada_entry(&raw mut entry);
                                }
                            }
                            0 | _ => {}
                        }
                    }
                    continue;
                }
            }
            return ret;
        }
        return ret;
    }
}
