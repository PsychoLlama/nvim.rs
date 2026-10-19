//! Merging what is already in the ShaDa file with what Nvim holds.
//!
//! Writing a ShaDa file is not writing this session's state over the old
//! one: it is a merge. Entries this Nvim has no opinion about are copied
//! straight through, and the ones it does have an opinion about are decided
//! by timestamp — the newer of the two wins, and where there is room for
//! several (histories, jumps, changes) the newest N are kept.
//!
//! Two structures do that. [`HistoryRing`] is a fixed-size list of history
//! entries in timestamp order, linked through slot indices so an entry can be
//! dropped from the middle in constant time; [`HistoryMergerState`] wraps one
//! per history type with what of Nvim's own history is still to be folded
//! in. File marks and jumps use plain arrays kept sorted by timestamp, which
//! [`marklist_insert`] makes room in.
//!
//! [`read_when_writing`] is the pass over the existing file that feeds all
//! of them.

#![forbid(unsafe_code)]

use crate::ascii::ascii_isdigit;
use crate::cmdhist::{
    HistRestored, HistShadaEntry, hist_shada_replace, hist_shada_take, hist_shada_view,
};
use crate::mark::{global_mark_timestamp, shada_local_mark_timestamp};
use crate::path::path_fnamecmp;
use crate::register::{op_reg_index, shada_cleared_register_timestamp};
use crate::registry::{IdMap, id_map};
use crate::search::shada_cleared_pattern_timestamp;
use crate::types::{Pos, Timestamp};
use crate::winlayer::{Win, buffers};
use core::cmp::Ordering;
use core::ffi::{CStr, c_int};
use std::collections::VecDeque;
use std::rc::Rc;

use super::collect::replace_numbered_mark;
use super::file::shada_removable;
use super::pack::{Packer, pack_entry};
use super::unpack::Source;
use super::{
    FileMarks, HIST_COUNT, HistoryMergerState, JUMPLISTSIZE, ReadKinds, ReadStatus, ShadaEntry,
    ShadaEntryData, ShadaHistoryItem, WriteMergerState, WriteStatus, mark_global_index,
    mark_local_index,
};

/// One slot of a [`HistoryRing`]: an entry and its neighbours in timestamp
/// order.
pub(crate) struct RingSlot {
    pub(crate) data: ShadaEntry,
    next: Option<usize>,
    prev: Option<usize>,
}

/// At most `size` history entries, oldest first, linked through slot
/// indices.
///
/// Slots are handed out from the front until `size` have been; after that a
/// removal leaves exactly one hole, which `hole` remembers and the next
/// insertion fills. Removing the slot handed out last gives it back instead.
pub(crate) struct HistoryRing {
    slots: Vec<RingSlot>,
    first: Option<usize>,
    last: Option<usize>,
    hole: Option<usize>,
    size: usize,
    num_entries: usize,
    /// The slot holding each history text, keyed by the slot's own shared
    /// text. Asked, never walked.
    contained_entries: IdMap<Rc<CStr>, usize>,
}

impl HistoryRing {
    /// A ring that holds at most `size` entries.
    pub(crate) fn new(size: usize) -> HistoryRing {
        HistoryRing {
            slots: Vec::with_capacity(size),
            first: None,
            last: None,
            hole: None,
            size,
            num_entries: 0,
            contained_entries: id_map(),
        }
    }

    /// The text an entry is filed under.
    fn key(entry: &ShadaEntry) -> &Rc<CStr> {
        &entry.data.history().text
    }

    /// Take the entry in slot `at` out of the ring, dropping it.
    fn remove(&mut self, at: usize) {
        let removed = self
            .contained_entries
            .remove(Self::key(&self.slots[at].data));
        debug_assert!(removed.is_some(), "shada: ring entry was not in the map");

        let (next, prev) = (self.slots[at].next, self.slots[at].prev);
        match next {
            None => self.last = prev,
            Some(next) => self.slots[next].prev = prev,
        }
        match prev {
            None => self.first = next,
            Some(prev) => self.slots[prev].next = next,
        }
        self.num_entries -= 1;

        if at == self.slots.len() - 1 {
            self.slots.pop();
        } else {
            debug_assert!(self.hole.is_none(), "shada: two holes in the ring");
            self.hole = Some(at);
            self.slots[at].data = ShadaEntry::MISSING;
        }
    }

    /// Put `data` into the ring just after `after` (or at the front when
    /// that is `None`). A full ring drops its oldest entry first.
    fn insert(&mut self, mut after: Option<usize>, data: ShadaEntry) {
        if self.num_entries == self.size {
            let first = self.first.expect("shada: a full ring is not empty");
            // The entry being inserted after may be the one about to go.
            if after == Some(first) {
                after = None;
            }
            self.remove(first);
        }

        let key = Rc::clone(Self::key(&data));
        let slot = RingSlot {
            data,
            next: None,
            prev: after,
        };
        let target = match self.hole.take() {
            Some(hole) => {
                debug_assert_eq!(self.slots.len() - 1, self.num_entries);
                self.slots[hole] = slot;
                hole
            }
            None => {
                debug_assert_eq!(self.slots.len(), self.num_entries);
                self.slots.push(slot);
                self.slots.len() - 1
            }
        };
        // Only a key that was not there takes the new slot.
        self.contained_entries.entry(key).or_insert(target);
        self.num_entries += 1;

        let next = match after {
            None => self.first.replace(target),
            Some(after) => self.slots[after].next.replace(target),
        };
        self.slots[target].next = next;
        match next {
            None => self.last = Some(target),
            Some(next) => self.slots[next].prev = Some(target),
        }
    }

    /// The entries, oldest first.
    pub(crate) fn iter(&self) -> impl Iterator<Item = &ShadaEntry> {
        let mut cur = self.first;
        core::iter::from_fn(move || {
            let at = cur?;
            cur = self.slots[at].next;
            Some(&self.slots[at].data)
        })
    }

    /// The entries, oldest first, taken out of the ring.
    pub(crate) fn drain(&mut self) -> Vec<ShadaEntry> {
        let mut out = Vec::with_capacity(self.num_entries);
        let mut cur = self.first;
        while let Some(at) = cur {
            cur = self.slots[at].next;
            out.push(core::mem::replace(
                &mut self.slots[at].data,
                ShadaEntry::MISSING,
            ));
        }
        self.slots.clear();
        self.first = None;
        self.last = None;
        self.hole = None;
        self.num_entries = 0;
        self.contained_entries.clear();
        out
    }

    /// Whether this ring keeps anything at all.
    pub(crate) fn keeps_nothing(&self) -> bool {
        self.size == 0
    }
}

impl HistoryMergerState {
    /// A merger for one history type, holding at most `num_elements`, with a
    /// snapshot of Nvim's own history for it to fold in: taken out of the
    /// history table when `reading` (the merger will put the merged result
    /// back), copied when writing.
    pub(crate) fn new(history_type: u8, num_elements: usize, reading: bool) -> HistoryMergerState {
        let entries = if reading {
            hist_shada_take(c_int::from(history_type))
        } else {
            hist_shada_view(c_int::from(history_type))
        };
        let pending = entries
            .into_iter()
            .map(|he: HistShadaEntry| ShadaEntry {
                timestamp: he.timestamp,
                data: ShadaEntryData::HistoryEntry(ShadaHistoryItem {
                    histtype: history_type,
                    text: he.text,
                    sep: he.sep,
                }),
                extra: he.extra,
            })
            .collect::<VecDeque<_>>();
        let mut hmll = HistoryRing::new(num_elements);
        // Nvim's own lines all go in; the file's may add some.
        hmll.contained_entries
            .reserve(num_elements.min(pending.len()));
        HistoryMergerState {
            hmll,
            pending,
            history_type,
        }
    }

    /// Insert one history entry, keeping the ring ordered by timestamp.
    ///
    /// An entry whose text is already in the ring replaces it only if it is
    /// newer — or, for one of Nvim's own entries, if it is as new: a tie goes
    /// to the running session.
    ///
    /// `from_file` says the entry came from the file rather than from Nvim's
    /// own history, in which case everything of Nvim's that is older is
    /// folded in first, so that the two sequences interleave by timestamp.
    pub(crate) fn insert(&mut self, entry: ShadaEntry, from_file: bool) {
        if from_file {
            while self
                .pending
                .front()
                .is_some_and(|next| next.timestamp < entry.timestamp)
            {
                let next = self.pending.pop_front().expect("the entry just looked at");
                self.insert(next, false);
            }
        }

        let ring = &mut self.hmll;
        if let Some(&existing) = ring.contained_entries.get(&**HistoryRing::key(&entry)) {
            let held = ring.slots[existing].data.timestamp;
            if entry.timestamp > held {
                ring.remove(existing);
            } else {
                if !from_file && entry.timestamp == held {
                    // The text is the same, so the map's key still names it.
                    ring.slots[existing].data = entry;
                }
                return;
            }
        }

        // Walk back from the newest to the first entry no newer than this
        // one; that is what the new entry goes after.
        let mut after = ring.last;
        while let Some(at) = after
            && ring.slots[at].data.timestamp > entry.timestamp
        {
            after = ring.slots[at].prev;
        }
        ring.insert(after, entry);
    }

    /// Fold in everything of Nvim's own history that is left.
    pub(crate) fn insert_whole_neovim_history(&mut self) {
        while let Some(next) = self.pending.pop_front() {
            self.insert(next, false);
        }
    }

    /// Make the merged ring Nvim's history for this type.
    pub(crate) fn into_history(mut self) {
        let merged: Vec<HistRestored> = self
            .hmll
            .drain()
            .into_iter()
            .map(|entry| {
                let ShadaEntryData::HistoryEntry(item) = entry.data else {
                    unreachable!("shada: the history ring holds history entries");
                };
                HistRestored {
                    text: item.text,
                    sep: item.sep,
                    timestamp: entry.timestamp,
                    extra: entry.extra,
                }
            })
            .collect();
        hist_shada_replace(c_int::from(self.history_type), merged);
        self.pending = VecDeque::new();
    }
}

/// Whether two marks are at the same place.
pub(crate) fn marks_equal(a: Pos, b: Pos) -> bool {
    a.lnum == b.lnum && a.col == b.col
}

/// Make room in `list` for an item that belongs just before index `i` (or
/// after the last item when `i == len`), and answer where to put it.
///
/// Higher indices are newer. A full list drops its oldest item to make
/// room; if the new item is older than everything in a full list there is
/// nowhere for it to go and this answers −1. So does an `i` of −1, which is
/// how a caller says the item is one the list already holds.
///
/// The items are *moved* within the list, not duplicated: what is left in
/// the slot answered is whatever the caller is about to write over.
pub(crate) fn marklist_insert<T>(list: &mut [T], len: c_int, i: c_int) -> c_int {
    let len = len as usize;
    match i.cmp(&0) {
        Ordering::Less => -1,
        Ordering::Greater => {
            let mut at = i as usize;
            if len == JUMPLISTSIZE {
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
            if len == JUMPLISTSIZE {
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

/// `slice::copy_within` without the `Copy` bound: the elements are swapped
/// in whichever direction keeps the source intact until it is read, which
/// is what a `memmove` of the same range does.
pub(crate) fn shift_within<T>(list: &mut [T], src: core::ops::Range<usize>, dest: usize) {
    if dest <= src.start {
        for (n, at) in src.enumerate() {
            list.swap(dest + n, at);
        }
    } else {
        for (n, at) in src.enumerate().rev() {
            list.swap(dest + n, at);
        }
    }
}

/// Put `entry` into a jump or change list kept oldest-first, dropping the
/// oldest item if it is full. `same` recognises an entry the list already
/// holds, which is then not inserted at all.
fn insert_mark_list(
    list: &mut [ShadaEntry],
    size: &mut usize,
    entry: ShadaEntry,
    same: impl Fn(&ShadaEntry) -> bool,
) {
    // Walk back to the first entry no newer than this one.
    let mut i = *size as c_int;
    while i > 0 {
        let existing = &list[i as usize - 1];
        if existing.timestamp <= entry.timestamp {
            if same(existing) {
                i = -1;
            }
            break;
        }
        i -= 1;
    }
    let i = marklist_insert(list, *size as c_int, i);
    if i == -1 {
        return;
    }
    list[i as usize] = entry;
    if *size < JUMPLISTSIZE {
        *size += 1;
    }
}

/// Keep the newer of the entry already in `slot` and the one just read,
/// dropping the other. A tie goes to the one already there, which is the
/// running Nvim's.
fn keep_newer(slot: &mut ShadaEntry, entry: ShadaEntry) {
    if !slot.is_missing() && slot.timestamp >= entry.timestamp {
        return;
    }
    *slot = entry;
}

/// Read the existing ShaDa file and merge it into `wms`.
///
/// Entries this Nvim has nowhere to put — an unknown type, a history it
/// does not keep, a register or mark name it does not know — go straight to
/// `out`, so that writing the file does not lose them.
pub(crate) fn read_when_writing(
    source: &mut Source,
    kinds: ReadKinds,
    max_kbyte: usize,
    wms: &mut WriteMergerState,
    out: &mut Packer,
) -> WriteStatus {
    let mut ret = WriteStatus::Successful;
    loop {
        let entry = match source.next_entry(kinds, max_kbyte) {
            Ok(entry) => entry,
            Err(ReadStatus::Finished) => return ret,
            Err(ReadStatus::Malformed) => continue,
            // Whatever has been merged so far is still worth writing.
            Err(ReadStatus::NotShada) => return WriteStatus::ReadNotShada,
            Err(_) => return ret,
        };

        match &entry.data {
            ShadaEntryData::Missing => {}
            ShadaEntryData::Header(_) | ShadaEntryData::BufferList(_) => {
                unreachable!("shada: entry type {} is never merged", entry.kind())
            }
            ShadaEntryData::Unknown(_) => ret = pack_entry(out, &entry, 0),
            ShadaEntryData::SearchPattern(pattern) => {
                let is_substitute = pattern.is_substitute_pattern.unwrap_or(false);
                let slot = if is_substitute {
                    &mut wms.sub_search_pattern
                } else {
                    &mut wms.search_pattern
                };
                // A pattern this session cleared since the file was written
                // stays cleared.
                if slot.is_missing()
                    && shada_cleared_pattern_timestamp(is_substitute)
                        .is_some_and(|cleared| cleared >= entry.timestamp)
                {
                    continue;
                }
                keep_newer(slot, entry);
            }
            ShadaEntryData::SubString(_) => keep_newer(&mut wms.replacement, entry),
            ShadaEntryData::HistoryEntry(item) => {
                let histtype = usize::from(item.histtype);
                if histtype >= HIST_COUNT {
                    ret = pack_entry(out, &entry, 0);
                } else if let Some(hms) = &mut wms.hms[histtype]
                    && !hms.hmll.keeps_nothing()
                {
                    hms.insert(entry, true);
                }
            }
            ShadaEntryData::Register(reg) => {
                match usize::try_from(op_reg_index(c_int::from(reg.name))) {
                    // Likewise a register this session emptied.
                    Ok(idx)
                        if wms.registers[idx].is_missing()
                            && shada_cleared_register_timestamp(reg.name)
                                .is_some_and(|cleared| cleared >= entry.timestamp) => {}
                    Ok(idx) => keep_newer(&mut wms.registers[idx], entry),
                    Err(_) => ret = pack_entry(out, &entry, 0),
                }
            }
            ShadaEntryData::Variable(var) => {
                // A variable this session has already written wins.
                if !wms.dumped_variables.contains(var.name.as_cstr().to_bytes()) {
                    ret = pack_entry(out, &entry, 0);
                }
            }
            ShadaEntryData::GlobalMark(_) => {
                if let Some(status) = merge_global_mark(wms, entry, out) {
                    ret = status;
                }
            }
            ShadaEntryData::Change(_) | ShadaEntryData::LocalMark(_) => {
                merge_file_mark(wms, entry);
            }
            ShadaEntryData::Jump(jump) => {
                let mark = jump.mark;
                let fname = jump.fname.as_cstr().to_bytes().to_vec();
                insert_mark_list(&mut wms.jumps, &mut wms.jumps_size, entry, |existing| {
                    let held = existing.data.filemark();
                    marks_equal(held.mark, mark) && held.fname.as_cstr().to_bytes() == fname
                });
            }
        }
    }
}

/// One global mark from the file: `Some` with the status of writing it
/// straight through, when it has a name this Nvim does not know.
///
/// A numbered mark has no name to match on — the ten of them are simply the
/// ten most recent, so it is placed by timestamp. A lettered mark goes in
/// its own slot, and has to beat whatever this Nvim holds for that letter.
fn merge_global_mark(
    wms: &mut WriteMergerState,
    entry: ShadaEntry,
    out: &mut Packer,
) -> Option<WriteStatus> {
    let name = entry.data.filemark().name;
    if ascii_isdigit(c_int::from(name)) {
        merge_numbered_mark(wms, entry);
        return None;
    }
    let Some(idx) = mark_global_index(name) else {
        return Some(pack_entry(out, &entry, 0));
    };
    let slot = if idx < 26 {
        &mut wms.global_marks[idx]
    } else {
        &mut wms.numbered_marks[idx - 26]
    };
    // Nothing has claimed the slot yet, so what the file entry has to beat
    // is the mark this Nvim holds.
    if slot.is_missing() && global_mark_timestamp(idx as c_int) >= entry.timestamp {
        return None;
    }
    keep_newer(slot, entry);
    None
}

/// A numbered global mark: kept in a list of the ten most recent, with the
/// mark names ignored entirely.
fn merge_numbered_mark(wms: &mut WriteMergerState, entry: ShadaEntry) {
    let len = wms.numbered_marks.len();
    for i in (1..=len).rev() {
        let existing = &wms.numbered_marks[i - 1];
        if !matches!(existing.data, ShadaEntryData::GlobalMark(_)) {
            continue;
        }
        // The same mark written twice: keep the one already here.
        let (held, read) = (existing.data.filemark(), entry.data.filemark());
        if existing.timestamp == entry.timestamp
            && existing.extra.is_none()
            && entry.extra.is_none()
            && marks_equal(held.mark, read.mark)
            && held.fname.as_cstr() == read.fname.as_cstr()
        {
            return;
        }
        if existing.timestamp >= entry.timestamp {
            if i < len {
                replace_numbered_mark(wms, i, entry);
            }
            // Otherwise it is older than all ten of them.
            return;
        }
    }
    replace_numbered_mark(wms, 0, entry);
}

/// A buffer-local mark or change-list entry from the file, filed under the
/// name of the file it belongs to.
fn merge_file_mark(wms: &mut WriteMergerState, entry: ShadaEntry) {
    let fname = entry.data.filemark().fname.clone();
    if shada_removable(fname.as_cstr()) {
        return;
    }

    let filemarks: &mut FileMarks = wms.file_marks_for(fname.as_cstr().to_bytes());
    if entry.timestamp > filemarks.greatest_timestamp {
        filemarks.greatest_timestamp = entry.timestamp;
    }

    if let ShadaEntryData::Change(change) = &entry.data {
        let mark = change.mark;
        insert_mark_list(
            &mut filemarks.changes,
            &mut filemarks.changes_size,
            entry,
            |existing| marks_equal(existing.data.filemark().mark, mark),
        );
        return;
    }

    let Some(idx) = mark_local_index(entry.data.filemark().name) else {
        // A mark name this Nvim does not know: keep it to write back.
        filemarks.additional_marks.push(entry);
        return;
    };
    let slot = &mut filemarks.marks[idx];
    if !slot.is_missing() {
        if slot.timestamp >= entry.timestamp {
            return;
        }
    } else if beaten_by_a_loaded_buffer(&entry) {
        return;
    }
    *slot = entry;
}

/// Whether a buffer Nvim has open on this file already holds a newer value
/// for the mark. Nothing has claimed the slot, so this is the comparison
/// [`keep_newer`] would otherwise make.
fn beaten_by_a_loaded_buffer(entry: &ShadaEntry) -> bool {
    let mark = entry.data.filemark();
    let name = mark.fname.as_cstr().to_bytes();
    for buf in buffers() {
        let Some(full) = buf.name.full() else {
            continue;
        };
        if path_fnamecmp(name, full.to_bytes()) == 0
            && shada_local_mark_timestamp(buf, Win::current(), c_int::from(mark.name))
                >= entry.timestamp
        {
            return true;
        }
    }
    false
}

/// Order one file's marks by timestamp, newest first — the reverse of the
/// usual sense, so that the files with the freshest marks come first when
/// only so many of them fit.
pub(crate) fn compare_file_marks(a: Timestamp, b: Timestamp) -> Ordering {
    b.cmp(&a)
}
