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
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::shada::ShadaExtra;
use crate::types::Timestamp;
use core::ffi::{CStr, c_int};
use std::ffi::CString;
use std::rc::Rc;

/// One history entry.
pub struct HistEntry {
    pub(super) number: c_int,
    /// Shared, so that the ShaDa writer can hold the text without a copy.
    pub(super) text: Rc<CStr>,
    pub(super) sep: u8,
    pub(super) timestamp: Timestamp,
    pub(super) extra: Option<ShadaExtra>,
}

impl HistEntry {
    /// Sequence number shown by `:history` and returned by `histnr()`.
    pub fn number(&self) -> c_int {
        self.number
    }

    /// Entry text without terminator.
    pub fn text(&self) -> &[u8] {
        self.text.to_bytes()
    }

    /// Separator character (search history only; NUL elsewhere).
    pub fn sep(&self) -> u8 {
        self.sep
    }

    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    /// The text as the C string it is stored as.
    pub(super) fn cstr(&self) -> &CStr {
        &self.text
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
/// `idx` is the raw slot of the newest entry, `None` when empty. Entries
/// sit contiguously behind `idx` (wrapping); vacant slots elsewhere are
/// normal after resizes and deletions.
pub struct Ring {
    pub(super) entries: Vec<Option<HistEntry>>,
    pub(super) idx: Option<usize>,
    pub(super) num: c_int,
}

pub(super) const EMPTY_RING: Ring = Ring {
    entries: Vec::new(),
    idx: None,
    num: 0,
};

impl Ring {
    pub fn new(len: usize) -> Ring {
        Ring {
            entries: (0..len).map(|_| None).collect(),
            idx: None,
            num: 0,
        }
    }

    /// Ring capacity (the `hislen` of the C implementation).
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.idx.is_none()
    }

    /// Raw slot index of the newest entry, `None` when empty.
    pub fn newest_idx(&self) -> Option<usize> {
        self.idx
    }

    /// Sequence number of the newest entry; `None` when the ring is empty,
    /// `0` when `idx` points at a vacant slot (possible after deleting the
    /// newest entry — a C quirk `histnr()` exposes).
    pub fn newest_number(&self) -> Option<c_int> {
        self.idx.map(|idx| self.number_at(idx))
    }

    /// Entry at raw slot `idx`, if occupied.
    pub fn get(&self, idx: usize) -> Option<&HistEntry> {
        self.entries.get(idx)?.as_ref()
    }

    /// Sequence number at a raw slot; vacant slots read as 0, exactly like
    /// the zero-filled C array.
    pub(super) fn number_at(&self, idx: usize) -> c_int {
        self.get(idx).map_or(0, |e| e.number)
    }

    /// The slot before `i`, wrapping.
    fn before(&self, i: usize) -> usize {
        i.checked_sub(1).unwrap_or(self.entries.len() - 1)
    }

    /// The slot after `i`, wrapping.
    fn after(&self, i: usize) -> usize {
        (i + 1) % self.entries.len()
    }

    /// Resize to `newlen` slots, keeping the newest entries. Mirrors the C
    /// `init_history` layout: kept entries are compacted to the front in
    /// age order (any leading vacancy from a not-full ring included), and
    /// `idx` lands on `min(newlen, oldlen) - 1`.
    pub fn resize(&mut self, newlen: usize) {
        let oldlen = self.entries.len();
        if newlen == oldlen {
            return;
        }
        let mut temp: Vec<Option<HistEntry>> = (0..newlen).map(|_| None).collect();
        let kept = newlen.min(oldlen);
        if let Some(j) = self.idx {
            // l1: slots [0..=j] to keep; l2: kept slots wrapped at the end.
            let l1 = (j + 1).min(newlen);
            let l2 = kept - l1;
            let i1 = j + 1 - l1;
            let i2 = l1 + oldlen.saturating_sub(newlen);
            let (wrapped, front) = temp.split_at_mut(l2);
            for (slot, kept) in wrapped.iter_mut().zip(&mut self.entries[i2..i2 + l2]) {
                *slot = kept.take();
            }
            for (slot, kept) in front[..l1].iter_mut().zip(&mut self.entries[i1..i1 + l1]) {
                *slot = kept.take();
            }
        }
        self.idx = self.idx.and_then(|_| kept.checked_sub(1));
        self.entries = temp;
    }

    /// Append an entry, overwriting the oldest slot when full.
    pub fn add(&mut self, text: &[u8], sep: u8, now: Timestamp) {
        let idx = self.idx.map_or(0, |idx| self.after(idx));
        self.idx = Some(idx);
        self.num += 1;
        self.entries[idx] = Some(HistEntry {
            number: self.num,
            text: Rc::from(to_cstring(text)),
            sep,
            timestamp: now,
            extra: None,
        });
    }

    /// If `text` is already in the ring (searching newest to oldest,
    /// stopping at the first vacant slot), move it to the front with a
    /// fresh number and timestamp and report `true`. `sep` must also match
    /// when given (search history distinguishes `/` from `?` entries).
    pub fn move_to_front(&mut self, text: &[u8], sep: Option<u8>, now: Timestamp) -> bool {
        let Some(start) = self.idx else {
            return false;
        };
        let mut i = start;
        let found = loop {
            match self.entries[i].as_ref() {
                None => return false,
                Some(e) => {
                    if e.text.to_bytes() == text && sep.is_none_or(|s| s == e.sep) {
                        break i;
                    }
                }
            }
            i = self.before(i);
            if i == start {
                return false;
            }
        };
        let mut entry = self.entries[found].take().expect("slot occupied");
        let mut i = found;
        while i != start {
            let next = self.after(i);
            self.entries[i] = self.entries[next].take();
            i = next;
        }
        self.num += 1;
        entry.number = self.num;
        entry.timestamp = now;
        entry.extra = None;
        self.entries[start] = Some(entry);
        true
    }

    /// Remove the newest entry, stepping `idx` back (used when a search
    /// from a mapping replaces the previous search from the same mapping).
    ///
    /// # Panics
    /// On an empty ring: the caller has seen an entry.
    pub fn drop_newest(&mut self) {
        let idx = self.idx.expect("a ring with an entry to drop");
        self.entries[idx] = None;
        self.num -= 1;
        self.idx = Some(self.before(idx));
    }

    /// Map a history number to a raw slot index: positive `num` finds the
    /// entry with that sequence number, negative counts back from the
    /// newest (-1 = newest). `None` if there is no such entry.
    pub fn calc_idx(&self, num: c_int) -> Option<usize> {
        let hislen = self.entries.len();
        let start = self.idx?;
        if hislen == 0 || num == 0 {
            return None;
        }
        if num > 0 {
            // Walk back from the newest while the numbers are too high,
            // wrapping once.
            let mut i = Some(start);
            let mut wrapped = false;
            while let Some(at) = i
                && self.number_at(at) > num
            {
                i = match at.checked_sub(1) {
                    Some(prev) => Some(prev),
                    None if wrapped => None,
                    None => {
                        wrapped = true;
                        Some(hislen - 1)
                    }
                };
            }
            i.filter(|&at| self.number_at(at) == num && self.get(at).is_some())
        } else {
            let back = usize::try_from(num.unsigned_abs()).ok()?;
            if back > hislen {
                return None;
            }
            // `start + 1 - back`, wrapping: `back` is at most `hislen`.
            let at = (start + 1 + hislen - back) % hislen;
            self.get(at).is_some().then_some(at)
        }
    }

    /// Drop every entry and reset numbering.
    pub fn clear(&mut self) {
        for slot in &mut self.entries {
            *slot = None;
        }
        self.idx = None;
        self.num = 0;
    }

    /// Delete every entry `matches` accepts, compacting survivors toward
    /// the newest slot (entries keep their numbers). Returns whether
    /// anything matched.
    pub fn delete_matching(&mut self, mut matches: impl FnMut(&HistEntry) -> bool) -> bool {
        let Some(idx) = self.idx else {
            return false;
        };
        let mut found = false;
        let mut i = idx;
        let mut last = idx;
        loop {
            let matched = match self.entries[i].as_ref() {
                None => break,
                Some(e) => matches(e),
            };
            if matched {
                found = true;
                self.entries[i] = None;
            } else {
                if i != last {
                    self.entries[last] = self.entries[i].take();
                }
                last = self.before(last);
            }
            i = self.before(i);
            if i == idx {
                break;
            }
        }
        if self.entries[idx].is_none() {
            self.idx = None;
        }
        found
    }

    /// Delete the entry at raw slot `i`, shifting newer entries down and
    /// stepping `idx` back one slot (which may leave it on a vacant slot —
    /// the C behavior when the newest entry is deleted).
    ///
    /// # Panics
    /// On an empty ring: `i` is a slot [`Ring::calc_idx`] found.
    pub fn delete_at(&mut self, mut i: usize) {
        let idx = self.idx.expect("a ring with an entry to delete");
        self.entries[i] = None;
        while i != idx {
            let j = self.after(i);
            self.entries[i] = self.entries[j].take();
            i = j;
        }
        self.idx = Some(self.before(idx));
    }

    /// Raw slot indexes of the live entries, oldest first: the first
    /// occupied slot after `idx`, forward (wrapping) until a vacant slot
    /// or until `idx` itself has been yielded.
    pub(super) fn oldest_first_indices(&self) -> Vec<usize> {
        let mut out = Vec::new();
        let Some(idx) = self.idx else {
            return out;
        };
        let mut p = idx;
        loop {
            p = self.after(p);
            if self.entries[p].is_some() {
                break;
            }
            if p == idx {
                return out;
            }
        }
        loop {
            if self.entries[p].is_none() {
                break;
            }
            out.push(p);
            if p == idx {
                break;
            }
            p = self.after(p);
        }
        out
    }
}
