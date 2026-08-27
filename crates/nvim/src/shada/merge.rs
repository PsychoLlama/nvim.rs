//! Merging what is already in the ShaDa file with what Nvim holds.
//!
//! Writing a ShaDa file is not writing this session's state over the old
//! one: it is a merge. Entries this Nvim has no opinion about are copied
//! straight through, and the ones it does have an opinion about are decided
//! by timestamp — the newer of the two wins, and where there is room for
//! several (histories, jumps, changes) the newest N are kept.
//!
//! Two structures do that. [`HMLList`] is a fixed-size ring of history
//! entries in a doubly linked list, so an entry can be dropped from the
//! middle in constant time; [`HistoryMergerState`] wraps one per history
//! type and remembers how much of Nvim's own history has been folded in.
//! File marks and jumps use plain arrays kept sorted by timestamp, which
//! [`marklist_insert`] makes room in.
//!
//! [`shada_read_when_writing`] is the pass over the existing file that feeds
//! all of them.

#![deny(unsafe_op_in_unsafe_fn)]

use core::cmp::Ordering;
use core::ffi::{c_int, c_uint, c_void};

use super::*;
use crate::mark::global_mark_timestamp;

/// Start a ring that holds at most `size` history entries.
///
/// The entries live in one array — `entries` — handed out from the front
/// (`last_free_entry`) until it is full; after that a removal leaves exactly
/// one hole, which `free_entry` remembers. The array stays a raw allocation
/// rather than a `Box<[_]>` because `HMLList` is embedded in a
/// [`WriteMergerState`] that is allocated zeroed.
pub(crate) unsafe fn hmll_init(hmll: *mut HMLList, size: size_t) {
    unsafe {
        let entries = xcalloc(size, size_of::<HMLListEntry>()).cast::<HMLListEntry>();
        hmll.write(HMLList {
            entries,
            first: core::ptr::null_mut(),
            last: core::ptr::null_mut(),
            free_entry: core::ptr::null_mut(),
            last_free_entry: entries,
            size,
            num_entries: 0,
            contained_entries: MAP_INIT,
        });
    }
}

/// Take one entry out of the ring, freeing what it holds.
pub(crate) unsafe fn hmll_remove(hmll: *mut HMLList, hmll_entry: *mut HMLListEntry) {
    unsafe {
        // Removing the slot that was handed out last just gives it back;
        // anything else leaves the one hole `free_entry` holds.
        if hmll_entry == (*hmll).last_free_entry.sub(1) {
            (*hmll).last_free_entry = (*hmll).last_free_entry.sub(1);
        } else {
            debug_assert!((*hmll).free_entry.is_null(), "shada: two holes in the ring");
            (*hmll).free_entry = hmll_entry;
        }

        let removed = map_del_cstr_t_ptr_t(
            &raw mut (*hmll).contained_entries,
            (*hmll_entry).data.data.history_item.string,
            core::ptr::null_mut(),
        );
        debug_assert!(!removed.is_null(), "shada: ring entry was not in the map");

        if (*hmll_entry).next.is_null() {
            (*hmll).last = (*hmll_entry).prev;
        } else {
            (*(*hmll_entry).next).prev = (*hmll_entry).prev;
        }
        if (*hmll_entry).prev.is_null() {
            (*hmll).first = (*hmll_entry).next;
        } else {
            (*(*hmll_entry).prev).next = (*hmll_entry).next;
        }
        (*hmll).num_entries -= 1;
        shada_free_shada_entry(&raw mut (*hmll_entry).data);
    }
}

/// Put `data` into the ring, just after `after` (or at the front when that
/// is null). A full ring drops its oldest entry first.
pub(crate) unsafe fn hmll_insert(
    hmll: *mut HMLList,
    mut after: *mut HMLListEntry,
    data: ShadaEntry,
) {
    unsafe {
        if (*hmll).num_entries == (*hmll).size {
            // The entry being inserted after may be the one about to go.
            if after == (*hmll).first {
                after = core::ptr::null_mut();
            }
            debug_assert!(!(*hmll).first.is_null(), "shada: a full ring is not empty");
            hmll_remove(hmll, (*hmll).first);
        }

        let target = if (*hmll).free_entry.is_null() {
            debug_assert_eq!(
                (*hmll)
                    .last_free_entry
                    .offset_from_unsigned((*hmll).entries),
                (*hmll).num_entries,
                "shada: the ring's free slot is not where it should be"
            );
            let target = (*hmll).last_free_entry;
            (*hmll).last_free_entry = target.add(1);
            target
        } else {
            debug_assert_eq!(
                (*hmll)
                    .last_free_entry
                    .offset_from_unsigned((*hmll).entries)
                    - 1,
                (*hmll).num_entries,
                "shada: the ring's hole is not where it should be"
            );
            let target = (*hmll).free_entry;
            (*hmll).free_entry = core::ptr::null_mut();
            target
        };

        (*target).data = data;
        let mut new_item = false;
        let val = map_put_ref_cstr_t_ptr_t(
            &raw mut (*hmll).contained_entries,
            data.data.history_item.string,
            core::ptr::null_mut(),
            &raw mut new_item,
        );
        if new_item {
            *val = target.cast::<c_void>();
        }
        (*hmll).num_entries += 1;

        (*target).prev = after;
        if after.is_null() {
            (*target).next = (*hmll).first;
            (*hmll).first = target;
        } else {
            (*target).next = (*after).next;
            (*after).next = target;
        }
        if (*target).next.is_null() {
            (*hmll).last = target;
        } else {
            (*(*target).next).prev = target;
        }
    }
}

/// Release the ring. Whatever the entries in it hold has been given away by
/// now — to Nvim's history, or to the file.
pub(crate) unsafe fn hmll_dealloc(hmll: *mut HMLList) {
    unsafe {
        let map = &raw mut (*hmll).contained_entries;
        xfree((*map).set.keys.cast::<c_void>());
        xfree((*map).set.h.hash.cast::<c_void>());
        (*map).set = Set_cstr_t {
            h: MAPHASH_INIT,
            keys: core::ptr::null_mut(),
        };
        xfree((*map).values.cast::<c_void>());
        (*map).values = core::ptr::null_mut();
        xfree((*hmll).entries.cast::<c_void>());
    }
}

/// Take a snapshot of Nvim's own history for this merger to fold in.
///
/// On the reading path the entries are taken out of the history table (the
/// merger will put the merged result back); on the writing path they are
/// only borrowed, so nothing here owns their strings.
pub(crate) unsafe fn hms_load_pending(hms_p: *mut HistoryMergerState) {
    unsafe {
        let history_type = (*hms_p).history_type;
        let entries = if (*hms_p).reading {
            hist_shada_take(history_type as c_int)
        } else {
            hist_shada_view(history_type as c_int)
        };
        let pending: Box<[ShadaEntry]> = entries
            .into_iter()
            .map(|he| ShadaEntry {
                type_0: kSDItemHistoryEntry,
                can_free_entry: (*hms_p).reading,
                timestamp: he.timestamp,
                data: ShadaEntryData {
                    history_item: history_item {
                        histtype: history_type,
                        string: he.text,
                        sep: he.sep,
                    },
                },
                additional_data: he.additional_data,
            })
            .collect();
        (*hms_p).pending_len = pending.len();
        (*hms_p).pending_pos = 0;
        (*hms_p).pending = Box::into_raw(pending).cast::<ShadaEntry>();
    }
}

/// Nvim's next own history entry, if one is still waiting to be merged.
unsafe fn next_pending(hms_p: *mut HistoryMergerState) -> Option<ShadaEntry> {
    unsafe {
        ((*hms_p).pending_pos < (*hms_p).pending_len)
            .then(|| *(*hms_p).pending.add((*hms_p).pending_pos))
    }
}

/// Insert one history entry, keeping the ring ordered by timestamp.
///
/// An entry whose text is already in the ring replaces it only if it is
/// newer — or, for one of Nvim's own entries, if it is as new: a tie goes to
/// the running session.
///
/// `do_iter` says the entry came from the file rather than from Nvim's own
/// history, in which case everything of Nvim's that is older is folded in
/// first, so that the two sequences interleave by timestamp.
pub(crate) unsafe fn hms_insert(hms_p: *mut HistoryMergerState, entry: ShadaEntry, do_iter: bool) {
    unsafe {
        if do_iter {
            while let Some(next) = next_pending(hms_p) {
                if next.timestamp >= entry.timestamp {
                    break;
                }
                (*hms_p).pending_pos += 1;
                hms_insert(hms_p, next, false);
            }
        }

        let hmll = &raw mut (*hms_p).hmll;
        let mut key_alloc: *mut cstr_t = core::ptr::null_mut();
        let val = map_ref_cstr_t_ptr_t(
            &raw mut (*hmll).contained_entries,
            entry.data.history_item.string,
            &raw mut key_alloc,
        );
        if !val.is_null() {
            let existing = (*val).cast::<HMLListEntry>();
            if entry.timestamp > (*existing).data.timestamp {
                hmll_remove(hmll, existing);
            } else if !do_iter && entry.timestamp == (*existing).data.timestamp {
                shada_free_shada_entry(&raw mut (*existing).data);
                (*existing).data = entry;
                // Freeing the entry above freed the key the map held.
                *key_alloc = entry.data.history_item.string;
                return;
            } else {
                return;
            }
        }

        // Walk back from the newest to the first entry no newer than this
        // one; that is what the new entry goes after.
        let mut after = (*hmll).last;
        while !after.is_null() && (*after).data.timestamp > entry.timestamp {
            after = (*after).prev;
        }
        hmll_insert(hmll, after, entry);
    }
}

/// Start a merger for one history type, holding at most `num_elements`.
pub(crate) unsafe fn hms_init(
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

/// Fold in everything of Nvim's own history that is left.
pub(crate) unsafe fn hms_insert_whole_neovim_history(hms_p: *mut HistoryMergerState) {
    unsafe {
        while let Some(next) = next_pending(hms_p) {
            (*hms_p).pending_pos += 1;
            hms_insert(hms_p, next, false);
        }
    }
}

/// Make the merged ring Nvim's history for this type.
pub(crate) unsafe fn hms_to_history(hms_p: *const HistoryMergerState) {
    unsafe {
        let mut merged: Vec<HistShadaEntry> = Vec::new();
        let mut cur = (*hms_p).hmll.first;
        while !cur.is_null() {
            merged.push(HistShadaEntry {
                text: (*cur).data.data.history_item.string,
                sep: (*cur).data.data.history_item.sep,
                timestamp: (*cur).data.timestamp,
                additional_data: (*cur).data.additional_data,
            });
            cur = (*cur).next;
        }
        hist_shada_replace((*hms_p).history_type as c_int, merged);
    }
}

/// Release the merger.
pub(crate) unsafe fn hms_dealloc(hms_p: *mut HistoryMergerState) {
    unsafe {
        // Free whatever part of the snapshot was never merged; it is only
        // owned on the reading path, and `shada_free_shada_entry` checks
        // `can_free_entry` itself.
        while let Some(mut entry) = next_pending(hms_p) {
            shada_free_shada_entry(&raw mut entry);
            (*hms_p).pending_pos += 1;
        }
        if !(*hms_p).pending.is_null() {
            drop(Box::from_raw(core::ptr::slice_from_raw_parts_mut(
                (*hms_p).pending,
                (*hms_p).pending_len,
            )));
            (*hms_p).pending = core::ptr::null_mut();
            (*hms_p).pending_len = 0;
        }
        hmll_dealloc(&raw mut (*hms_p).hmll);
    }
}

/// Whether two marks are at the same place.
#[inline]
pub(crate) fn marks_equal(a: pos_T, b: pos_T) -> bool {
    a.lnum == b.lnum && a.col == b.col
}

/// Order one file's marks by timestamp, newest first — the reverse of the
/// usual sense, so that the files with the freshest marks come first when
/// only so many of them fit. Signature for `qsort`.
pub(crate) unsafe extern "C" fn compare_file_marks(a: *const c_void, b: *const c_void) -> c_int {
    unsafe {
        let a = *a.cast::<*const FileMarks>();
        let b = *b.cast::<*const FileMarks>();
        match (*a).greatest_timestamp.cmp(&(*b).greatest_timestamp) {
            Ordering::Equal => 0,
            Ordering::Greater => -1,
            Ordering::Less => 1,
        }
    }
}

/// Make room in `list` for an item that belongs just before index `i` (or
/// after the last item when `i == len`), and answer where to put it.
///
/// Higher indices are newer. A full list drops its oldest item to make
/// room; if the new item is older than everything in a full list there is
/// nowhere for it to go and this answers −1. So does an `i` of −1, which is
/// how a caller says the item is one the list already holds.
///
/// The items are *moved* within the list, not duplicated: a mark owns its
/// ShaDa extra data, and the slot a shift vacates is written over by the
/// caller. [`shift_within`] is `slice::copy_within` for a type that is only
/// `Clone` for that reason.
pub(crate) fn marklist_insert<T: Clone>(list: &mut [T], len: c_int, i: c_int) -> c_int {
    let len = len as usize;
    match i.cmp(&0) {
        Ordering::Less => -1,
        Ordering::Greater => {
            let mut at = i as usize;
            if len == JUMPLISTSIZE as usize {
                at -= 1;
                if at > 0 {
                    shift_within(list, 1..at + 1, 0); // the oldest item goes
                }
            } else if at != len {
                shift_within(list, at..len, at + 1); // newer items shift up
            }
            at as c_int
        }
        Ordering::Equal => {
            if len == JUMPLISTSIZE as usize {
                -1 // older than the whole list
            } else {
                if len > 0 {
                    shift_within(list, 0..len, 1);
                }
                0
            }
        }
    }
}

/// `slice::copy_within` without the `Copy` bound: the elements are shallow-
/// cloned in whichever direction keeps the source intact until it is read,
/// which is what a `memmove` of the same range does.
fn shift_within<T: Clone>(list: &mut [T], src: core::ops::Range<usize>, dest: usize) {
    if dest <= src.start {
        for (n, at) in src.enumerate() {
            list[dest + n] = list[at].clone();
        }
    } else {
        for (n, at) in src.enumerate().rev() {
            list[dest + n] = list[at].clone();
        }
    }
}

/// Put `entry` into a jump or change list kept oldest-first, dropping the
/// oldest item if it is full. `same` recognises an entry the list already
/// holds, which is then not inserted at all.
unsafe fn insert_mark_list(
    list: &mut [ShadaEntry],
    size: &mut size_t,
    mut entry: ShadaEntry,
    same: impl Fn(&ShadaEntry) -> bool,
) {
    unsafe {
        // Walk back to the first entry no newer than this one.
        let mut i = *size as c_int;
        while i > 0 {
            let existing = list[i as usize - 1];
            if existing.timestamp <= entry.timestamp {
                if same(&existing) {
                    i = -1;
                }
                break;
            }
            i -= 1;
        }
        if i > 0 && *size == JUMPLISTSIZE as size_t {
            shada_free_shada_entry(&raw mut list[0]);
        }
        let i = marklist_insert(list, *size as c_int, i);
        if i == -1 {
            shada_free_shada_entry(&raw mut entry);
            return;
        }
        list[i as usize] = entry;
        if *size < JUMPLISTSIZE as size_t {
            *size += 1;
        }
    }
}

/// Keep the newer of the entry already in `slot` and the one just read,
/// freeing the other. A tie goes to the one already there, which is the
/// running Nvim's.
unsafe fn keep_newer(slot: *mut ShadaEntry, mut entry: ShadaEntry) {
    unsafe {
        if (*slot).type_0 != kSDItemMissing {
            if (*slot).timestamp >= entry.timestamp {
                shada_free_shada_entry(&raw mut entry);
                return;
            }
            shada_free_shada_entry(slot);
        }
        *slot = entry;
    }
}

/// Read the existing ShaDa file and merge it into `wms`.
///
/// Entries this Nvim has nowhere to put — an unknown type, a history it
/// does not keep, a register or mark name it does not know — go straight to
/// `packer`, so that writing the file does not lose them.
pub(crate) unsafe fn shada_read_when_writing(
    sd_reader: *mut FileDescriptor,
    srni_flags: c_uint,
    max_kbyte: size_t,
    wms: *mut WriteMergerState,
    packer: *mut PackerBuffer,
) -> ShaDaWriteResult {
    unsafe {
        let mut ret = kSDWriteSuccessful;
        loop {
            let mut entry: ShadaEntry = core::mem::zeroed();
            match shada_read_next_item(sd_reader, &raw mut entry, srni_flags, max_kbyte) {
                kSDReadStatusSuccess => {}
                kSDReadStatusFinished => return ret,
                kSDReadStatusMalformed => continue,
                // Whatever has been merged so far is still worth writing.
                kSDReadStatusNotShaDa => return kSDWriteReadNotShada,
                _ => return ret,
            }

            match entry.type_0 {
                kSDItemMissing => {}
                kSDItemHeader | kSDItemBufferList => {
                    unreachable!("shada: entry type {} is never merged", entry.type_0)
                }
                kSDItemUnknown => {
                    ret = shada_pack_entry(packer, entry, 0);
                    shada_free_shada_entry(&raw mut entry);
                }
                kSDItemSearchPattern => {
                    let slot = if entry.data.search_pattern.is_substitute_pattern {
                        &raw mut (*wms).sub_search_pattern
                    } else {
                        &raw mut (*wms).search_pattern
                    };
                    keep_newer(slot, entry);
                }
                kSDItemSubString => keep_newer(&raw mut (*wms).replacement, entry),
                kSDItemHistoryEntry => ret = merge_history(wms, entry, packer, ret),
                kSDItemRegister => {
                    let idx = op_reg_index(entry.data.reg.name as c_int);
                    if idx < 0 {
                        ret = shada_pack_entry(packer, entry, 0);
                        shada_free_shada_entry(&raw mut entry);
                    } else {
                        keep_newer(&raw mut (*wms).registers[idx as usize], entry);
                    }
                }
                kSDItemVariable => {
                    // A variable this session has already written wins.
                    if !set_has_cstr_t(&raw mut (*wms).dumped_variables, entry.data.global_var.name)
                    {
                        ret = shada_pack_entry(packer, entry, 0);
                    }
                    shada_free_shada_entry(&raw mut entry);
                }
                kSDItemGlobalMark => ret = merge_global_mark(wms, entry, packer, ret),
                kSDItemChange | kSDItemLocalMark => merge_file_mark(wms, entry),
                kSDItemJump => {
                    let (mark, fname) = (entry.data.filemark.mark, entry.data.filemark.fname);
                    insert_mark_list(
                        &mut (*wms).jumps,
                        &mut (*wms).jumps_size,
                        entry,
                        |existing| {
                            marks_equal(existing.data.filemark.mark, mark)
                                && strcmp(existing.data.filemark.fname, fname) == 0
                        },
                    );
                }
                _ => {}
            }
        }
    }
}

/// One history entry from the file. A history this Nvim does not have goes
/// straight through; one it has but is keeping nothing of is dropped.
unsafe fn merge_history(
    wms: *mut WriteMergerState,
    mut entry: ShadaEntry,
    packer: *mut PackerBuffer,
    ret: ShaDaWriteResult,
) -> ShaDaWriteResult {
    unsafe {
        let histtype = entry.data.history_item.histtype as c_uint;
        if histtype >= HIST_COUNT {
            let ret = shada_pack_entry(packer, entry, 0);
            shada_free_shada_entry(&raw mut entry);
            return ret;
        }
        let hms = &raw mut (*wms).hms[histtype as usize];
        if (*hms).hmll.size != 0 {
            hms_insert(hms, entry, true);
        } else {
            shada_free_shada_entry(&raw mut entry);
        }
        ret
    }
}

/// One global mark from the file.
///
/// A numbered mark has no name to match on — the ten of them are simply the
/// ten most recent, so it is placed by timestamp. A lettered mark goes in
/// its own slot, and has to beat whatever this Nvim holds for that letter.
unsafe fn merge_global_mark(
    wms: *mut WriteMergerState,
    mut entry: ShadaEntry,
    packer: *mut PackerBuffer,
    ret: ShaDaWriteResult,
) -> ShaDaWriteResult {
    unsafe {
        if ascii_isdigit(entry.data.filemark.name as c_int) {
            merge_numbered_mark(wms, entry);
            return ret;
        }

        let idx = mark_global_index(entry.data.filemark.name);
        if idx < 0 {
            let ret = shada_pack_entry(packer, entry, 0);
            shada_free_shada_entry(&raw mut entry);
            return ret;
        }
        let slot = if idx < 26 {
            &raw mut (*wms).global_marks[idx as usize]
        } else {
            &raw mut (*wms).numbered_marks[idx as usize - 26]
        };
        // Nothing has claimed the slot yet, so what the file entry has to
        // beat is the mark this Nvim holds.
        if (*slot).type_0 == kSDItemMissing && global_mark_timestamp(idx) >= entry.timestamp {
            shada_free_shada_entry(&raw mut entry);
            return ret;
        }
        keep_newer(slot, entry);
        ret
    }
}

/// A numbered global mark: kept in a list of the ten most recent, with the
/// mark names ignored entirely.
unsafe fn merge_numbered_mark(wms: *mut WriteMergerState, mut entry: ShadaEntry) {
    unsafe {
        let marks = &(*wms).numbered_marks;
        for i in (1..=marks.len()).rev() {
            let existing = marks[i - 1];
            if existing.type_0 != kSDItemGlobalMark {
                continue;
            }
            // The same mark written twice: keep the one already here.
            if existing.timestamp == entry.timestamp
                && existing.additional_data.is_null()
                && entry.additional_data.is_null()
                && marks_equal(existing.data.filemark.mark, entry.data.filemark.mark)
                && strcmp(existing.data.filemark.fname, entry.data.filemark.fname) == 0
            {
                shada_free_shada_entry(&raw mut entry);
                return;
            }
            if existing.timestamp >= entry.timestamp {
                if i < marks.len() {
                    replace_numbered_mark(wms, i, entry);
                } else {
                    // Older than all ten of them.
                    shada_free_shada_entry(&raw mut entry);
                }
                return;
            }
        }
        replace_numbered_mark(wms, 0, entry);
    }
}

/// A buffer-local mark or change-list entry from the file, filed under the
/// name of the file it belongs to.
unsafe fn merge_file_mark(wms: *mut WriteMergerState, mut entry: ShadaEntry) {
    unsafe {
        let fname = entry.data.filemark.fname;
        if shada_removable(fname) {
            shada_free_shada_entry(&raw mut entry);
            return;
        }

        let mut key: *mut cstr_t = core::ptr::null_mut();
        let mut new_item = false;
        let val = map_put_ref_cstr_t_ptr_t(
            &raw mut (*wms).file_marks,
            fname,
            &raw mut key,
            &raw mut new_item,
        );
        if new_item {
            *key = xstrdup(fname);
        }
        if (*val).is_null() {
            *val = xcalloc(1, size_of::<FileMarks>());
        }
        let filemarks = (*val).cast::<FileMarks>();
        if entry.timestamp > (*filemarks).greatest_timestamp {
            (*filemarks).greatest_timestamp = entry.timestamp;
        }

        if entry.type_0 == kSDItemChange {
            let mark = entry.data.filemark.mark;
            insert_mark_list(
                &mut (*filemarks).changes,
                &mut (*filemarks).changes_size,
                entry,
                |existing| marks_equal(existing.data.filemark.mark, mark),
            );
            return;
        }

        let idx = mark_local_index(entry.data.filemark.name);
        if idx < 0 {
            // A mark name this Nvim does not know: keep it to write back.
            (*filemarks).additional_marks_size += 1;
            (*filemarks).additional_marks = xrealloc(
                (*filemarks).additional_marks.cast::<c_void>(),
                (*filemarks).additional_marks_size * size_of::<ShadaEntry>(),
            )
            .cast::<ShadaEntry>();
            *(*filemarks)
                .additional_marks
                .add((*filemarks).additional_marks_size - 1) = entry;
            return;
        }

        let slot = &raw mut (*filemarks).marks[idx as usize];
        if (*slot).type_0 != kSDItemMissing {
            if (*slot).timestamp >= entry.timestamp {
                shada_free_shada_entry(&raw mut entry);
                return;
            }
            if (*slot).can_free_entry {
                // The map's key may be the very string about to be freed.
                if *key == (*slot).data.filemark.fname {
                    *key = entry.data.filemark.fname;
                }
                shada_free_shada_entry(slot);
            }
        } else if beaten_by_a_loaded_buffer(&entry) {
            shada_free_shada_entry(&raw mut entry);
            return;
        }
        *slot = entry;
    }
}

/// Whether a buffer Nvim has open on this file already holds a newer value
/// for the mark. Nothing has claimed the slot, so this is the comparison
/// [`keep_newer`] would otherwise make.
unsafe fn beaten_by_a_loaded_buffer(entry: &ShadaEntry) -> bool {
    unsafe {
        let mut buf = firstbuf.get();
        while !buf.is_null() {
            if !(*buf).b_ffname.is_null()
                && path_fnamecmp(entry.data.filemark.fname, (*buf).b_ffname) == 0
            {
                let mut fm: fmark_T = fmark_T::UNSET;
                mark_get(
                    buf,
                    curwin.get(),
                    &raw mut fm,
                    kMarkBufLocal,
                    entry.data.filemark.name as c_int,
                );
                if fm.timestamp >= entry.timestamp {
                    return true;
                }
            }
            buf = (*buf).b_next;
        }
        false
    }
}
