//! [`Ring`], the fixed-capacity history ring, and the entries it holds.
//!
//! Pure data structure: the editor-state coupling (the 'history' option,
//! `maptick`, timestamps, the shada boundary) all lives in the parent
//! module, which is why this half can be checked by Miri.
//!
//! Raw slot indexes are part of the contract — cmdline navigation keeps one
//! across keystrokes — so vacant slots left by a resize or a deletion behave
//! exactly as they did in the C arrays.

#![forbid(unsafe_code)]

use super::ExtraData;
use crate::types::Timestamp;
use core::ffi::{c_char, c_int};
use std::ffi::CString;

/// One history entry.
pub struct HistEntry {
    pub(super) number: c_int,
    pub(super) text: CString,
    pub(super) sep: u8,
    pub(super) timestamp: Timestamp,
    pub(super) extra: ExtraData,
}

impl HistEntry {
    /// Sequence number shown by `:history` and returned by `histnr()`.
    pub fn number(&self) -> c_int {
        self.number
    }

    /// Entry text without terminator.
    pub fn text(&self) -> &[u8] {
        self.text.as_bytes()
    }

    /// Separator character (search history only; NUL elsewhere).
    pub fn sep(&self) -> u8 {
        self.sep
    }

    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    pub(super) fn c_ptr(&self) -> *const c_char {
        self.text.as_ptr()
    }
}

/// Truncate at the first NUL; history entries are C strings and can never
/// contain one.
pub(super) fn to_cstring(bytes: &[u8]) -> CString {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    CString::new(&bytes[..end]).expect("no interior NUL before `end`")
}

/// A fixed-capacity history ring. Pure data structure: all editor-state
/// coupling (the 'history' option, maptick, timestamps) lives in the
/// module-level functions.
///
/// `idx` is the raw slot of the newest entry, `-1` when empty. Entries sit
/// contiguously behind `idx` (wrapping); vacant slots elsewhere are normal
/// after resizes and deletions.
pub struct Ring {
    pub(super) entries: Vec<Option<HistEntry>>,
    pub(super) idx: c_int,
    pub(super) num: c_int,
}

pub(super) const EMPTY_RING: Ring = Ring {
    entries: Vec::new(),
    idx: -1,
    num: 0,
};

impl Ring {
    pub fn new(len: usize) -> Ring {
        Ring {
            entries: (0..len).map(|_| None).collect(),
            idx: -1,
            num: 0,
        }
    }

    /// Ring capacity (the `hislen` of the C implementation).
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.idx < 0
    }

    /// Raw slot index of the newest entry, `-1` when empty.
    pub fn newest_idx(&self) -> c_int {
        self.idx
    }

    /// Sequence number of the newest entry; `-1` when the ring is empty,
    /// `0` when `idx` points at a vacant slot (possible after deleting the
    /// newest entry — a C quirk `histnr()` exposes).
    pub fn newest_number(&self) -> c_int {
        if self.idx < 0 {
            return -1;
        }
        self.number_at(self.idx)
    }

    /// Entry at raw slot `idx`, if occupied.
    pub fn get(&self, idx: c_int) -> Option<&HistEntry> {
        let idx = usize::try_from(idx).ok()?;
        self.entries.get(idx)?.as_ref()
    }

    /// Sequence number at a raw slot; vacant slots read as 0, exactly like
    /// the zero-filled C array.
    pub(super) fn number_at(&self, idx: c_int) -> c_int {
        self.get(idx).map_or(0, |e| e.number)
    }

    /// Resize to `newlen` slots, keeping the newest entries. Mirrors the C
    /// `init_history` layout: kept entries are compacted to the front in
    /// age order (any leading vacancy from a not-full ring included), and
    /// `idx` lands on `min(newlen, oldlen) - 1`.
    pub fn resize(&mut self, newlen: usize) {
        let oldlen = self.entries.len() as c_int;
        let newlen_i = newlen as c_int;
        if newlen_i == oldlen {
            return;
        }
        let mut temp: Vec<Option<HistEntry>> = (0..newlen).map(|_| None).collect();
        let j = self.idx;
        if j >= 0 {
            // l1: slots [0..=j] to keep; l2: kept slots wrapped at the end.
            let l1 = (j + 1).min(newlen_i);
            let l2 = newlen_i.min(oldlen) - l1;
            let i1 = j + 1 - l1;
            let i2 = l1.max(oldlen - newlen_i + l1);
            for k in 0..l2 {
                temp[k as usize] = self.entries[(i2 + k) as usize].take();
            }
            for k in 0..l1 {
                temp[(l2 + k) as usize] = self.entries[(i1 + k) as usize].take();
            }
        }
        self.idx = if j < 0 { -1 } else { newlen_i.min(oldlen) - 1 };
        self.entries = temp;
    }

    /// Append an entry, overwriting the oldest slot when full.
    pub fn add(&mut self, text: &[u8], sep: u8, now: Timestamp) {
        self.idx += 1;
        if self.idx == self.entries.len() as c_int {
            self.idx = 0;
        }
        self.num += 1;
        self.entries[self.idx as usize] = Some(HistEntry {
            number: self.num,
            text: to_cstring(text),
            sep,
            timestamp: now,
            extra: ExtraData::NONE,
        });
    }

    /// If `text` is already in the ring (searching newest to oldest,
    /// stopping at the first vacant slot), move it to the front with a
    /// fresh number and timestamp and report `true`. `sep` must also match
    /// when given (search history distinguishes `/` from `?` entries).
    pub fn move_to_front(&mut self, text: &[u8], sep: Option<u8>, now: Timestamp) -> bool {
        if self.idx < 0 {
            return false;
        }
        let hislen = self.entries.len() as c_int;
        let start = self.idx;
        let mut i = start;
        let found = loop {
            match self.entries[i as usize].as_ref() {
                None => return false,
                Some(e) => {
                    if e.text.as_bytes() == text && sep.is_none_or(|s| s == e.sep) {
                        break i;
                    }
                }
            }
            i -= 1;
            if i < 0 {
                i = hislen - 1;
            }
            if i == start {
                return false;
            }
        };
        let mut entry = self.entries[found as usize].take().expect("slot occupied");
        let mut i = found;
        while i != start {
            let next = (i + 1) % hislen;
            self.entries[i as usize] = self.entries[next as usize].take();
            i = next;
        }
        self.num += 1;
        entry.number = self.num;
        entry.timestamp = now;
        entry.extra = ExtraData::NONE;
        self.entries[start as usize] = Some(entry);
        true
    }

    /// Remove the newest entry, stepping `idx` back (used when a search
    /// from a mapping replaces the previous search from the same mapping).
    pub fn drop_newest(&mut self) {
        self.entries[self.idx as usize] = None;
        self.num -= 1;
        self.idx -= 1;
        if self.idx < 0 {
            self.idx = self.entries.len() as c_int - 1;
        }
    }

    /// Map a history number to a raw slot index: positive `num` finds the
    /// entry with that sequence number, negative counts back from the
    /// newest (-1 = newest). Returns -1 if there is no such entry.
    pub fn calc_idx(&self, num: c_int) -> c_int {
        let hislen = self.entries.len() as c_int;
        let mut i = self.idx;
        if hislen == 0 || i < 0 || num == 0 {
            return -1;
        }
        if num > 0 {
            let mut wrapped = false;
            while self.number_at(i) > num {
                i -= 1;
                if i >= 0 {
                    continue;
                }
                if wrapped {
                    break;
                }
                i += hislen;
                wrapped = true;
            }
            if i >= 0 && self.number_at(i) == num && self.get(i).is_some() {
                return i;
            }
        } else if -i64::from(num) <= i64::from(hislen) {
            i += num + 1;
            if i < 0 {
                i += hislen;
            }
            if self.get(i).is_some() {
                return i;
            }
        }
        -1
    }

    /// Drop every entry and reset numbering.
    pub fn clear(&mut self) {
        for slot in &mut self.entries {
            *slot = None;
        }
        self.idx = -1;
        self.num = 0;
    }

    /// Delete every entry `matches` accepts, compacting survivors toward
    /// the newest slot (entries keep their numbers). Returns whether
    /// anything matched.
    pub fn delete_matching(&mut self, mut matches: impl FnMut(&HistEntry) -> bool) -> bool {
        if self.idx < 0 {
            return false;
        }
        let hislen = self.entries.len() as c_int;
        let idx = self.idx;
        let mut found = false;
        let mut i = idx;
        let mut last = idx;
        loop {
            let matched = match self.entries[i as usize].as_ref() {
                None => break,
                Some(e) => matches(e),
            };
            if matched {
                found = true;
                self.entries[i as usize] = None;
            } else {
                if i != last {
                    self.entries[last as usize] = self.entries[i as usize].take();
                }
                last -= 1;
                if last < 0 {
                    last += hislen;
                }
            }
            i -= 1;
            if i < 0 {
                i += hislen;
            }
            if i == idx {
                break;
            }
        }
        if self.entries[idx as usize].is_none() {
            self.idx = -1;
        }
        found
    }

    /// Delete the entry at raw slot `i`, shifting newer entries down and
    /// stepping `idx` back one slot (which may leave it on a vacant slot —
    /// the C behavior when the newest entry is deleted).
    pub fn delete_at(&mut self, mut i: c_int) {
        let hislen = self.entries.len() as c_int;
        let idx = self.idx;
        self.entries[i as usize] = None;
        while i != idx {
            let j = (i + 1) % hislen;
            self.entries[i as usize] = self.entries[j as usize].take();
            i = j;
        }
        self.idx = if idx > 0 { idx - 1 } else { idx - 1 + hislen };
    }

    /// Raw slot indexes of the live entries, oldest first: the first
    /// occupied slot after `idx`, forward (wrapping) until a vacant slot
    /// or until `idx` itself has been yielded.
    pub(super) fn oldest_first_indices(&self) -> Vec<c_int> {
        let mut out = Vec::new();
        if self.idx < 0 {
            return out;
        }
        let hislen = self.entries.len() as c_int;
        let idx = self.idx;
        let mut p = idx;
        loop {
            p = (p + 1) % hislen;
            if self.entries[p as usize].is_some() {
                break;
            }
            if p == idx {
                return out;
            }
        }
        loop {
            if self.entries[p as usize].is_none() {
                break;
            }
            out.push(p);
            if p == idx {
                break;
            }
            p = (p + 1) % hislen;
        }
        out
    }
}
