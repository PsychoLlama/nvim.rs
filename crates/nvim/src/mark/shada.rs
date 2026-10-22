//! The surface the shada reader and writer drive, over owned values.
//!
//! The stores keep their marks as `FileMark`/`XFileMark` records that own
//! their file name and ShaDa extra data, and so do ShaDa's entries: the
//! readers here answer copies -- [`shada_global_marks`],
//! [`shada_buffer_marks`], [`shada_jumps`] and [`shada_changes`] -- and the
//! setters and inserters take a [`ShadaMark`] by value and move it into the
//! store's record.
//!
//! The two setters are the merge half. Neither overwrites a record that is
//! *newer* than the one being merged in, which is what `update` selects; the
//! comparison is on the timestamp every store carries and nothing else prints.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::memory::{ThinCString, XString};
use crate::os::time::os_time;
use crate::shada::{ShadaExtra, marklist_insert};
use crate::types::{FileMark, FileMarkView, Pos, Timestamp, XFileMark};
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int};

use super::lookup::mark_global_index;
use super::store::{LocalMark, global_mark, global_mark_names, mark_name, slot};
use super::*;

/// One mark as ShaDa reads and writes it: where it is, in which buffer (or
/// file, when no buffer holds it), when it was set, and the extra data the
/// file gave it.
pub(crate) struct ShadaMark {
    pub(crate) mark: Pos,
    /// The buffer's number; 0 when the mark names a file instead.
    pub(crate) fnum: c_int,
    pub(crate) timestamp: Timestamp,
    /// The file a mark with no buffer names.
    pub(crate) fname: Option<XString>,
    pub(crate) extra: Option<ShadaExtra>,
}

impl ShadaMark {
    /// A copy of a store's record.
    fn of(fm: &FileMark, fname: Option<XString>) -> ShadaMark {
        ShadaMark {
            mark: fm.mark,
            fnum: fm.fnum,
            timestamp: fm.timestamp,
            fname,
            extra: fm.extra.as_deref().cloned(),
        }
    }

    /// The store's record for it, taking the extra data over.
    fn into_fmark(self) -> (FileMark, Option<XString>) {
        let fm = FileMark {
            mark: self.mark,
            fnum: self.fnum,
            timestamp: self.timestamp,
            // Nothing is known about where the window was scrolled to.
            view: FileMarkView::NONE,
            extra: self.extra.map(Box::new),
        };
        (fm, self.fname)
    }

    /// The store's record with a file name, taking both over.
    fn into_xfmark(self) -> XFileMark {
        let (fmark, fname) = self.into_fmark();
        XFileMark {
            fmark,
            fname: fname.map(ThinCString::from),
        }
    }
}

/// Every set global mark with its name, `'A`-`'Z` then `'0`-`'9`, copied.
pub(crate) fn shada_global_marks() -> Vec<(c_char, ShadaMark)> {
    let mut found = Vec::new();
    for (at, name) in global_mark_names() {
        let xfm = global_mark(at);
        if !xfm.fmark.is_set() {
            continue;
        }
        let fname = xfm.file_name().map(XString::from_cstr);
        found.push((name, ShadaMark::of(&xfm.fmark, fname)));
    }
    found
}

/// Every set mark of `buffer` with its name, `'"`, `'^`, `'.` then
/// `'a`-`'z`, copied.
pub(crate) fn shada_buffer_marks(buffer: Buf) -> Vec<(c_char, ShadaMark)> {
    let special = [
        (b'"', &buffer.b_last_cursor),
        (b'^', &buffer.b_last_insert),
        (b'.', &buffer.b_last_change),
    ];
    let named = buffer
        .b_namedm
        .iter()
        .zip(b'a'..=b'z')
        .map(|(mark, name)| (name, mark));
    special
        .into_iter()
        .chain(named)
        .filter(|(_, mark)| mark.is_set())
        .map(|(name, mark)| (mark_name(c_int::from(name)), ShadaMark::of(mark, None)))
        .collect()
}

/// The jump list of `win`, oldest first, copied.
pub(crate) fn shada_jumps(win: Win) -> Vec<ShadaMark> {
    let len = slot(win.w_jumplistlen);
    win.w_jumplist[..len]
        .iter()
        .map(|jump| ShadaMark::of(&jump.fmark, jump.file_name().map(XString::from_cstr)))
        .collect()
}

/// The change list of `buffer`, oldest first, copied.
pub(crate) fn shada_changes(buffer: Buf) -> Vec<ShadaMark> {
    let len = slot(buffer.b_changelistlen);
    buffer.b_changelist[..len]
        .iter()
        .map(|change| ShadaMark::of(change, None))
        .collect()
}

/// Set global mark `name` to `fm`, which the table takes over; answers
/// whether it did.
///
/// With `update`, a mark is only set if it was made later than the one
/// already there.
pub(crate) fn shada_set_global_mark(name: c_char, fm: ShadaMark, update: bool) -> bool {
    let idx = mark_global_index(name);
    if idx == -1 {
        return false;
    }
    let idx = slot(idx);
    // A merge never overwrites a record the editor made more recently than
    // the one the shada file carries; a plain set (`update` false) always does.
    if update && fm.timestamp <= namedfm.with(|table| table[idx].fmark.timestamp) {
        return false;
    }
    store::set_global_mark(idx, fm.into_xfmark());
    true
}

/// Set `buffer`'s mark `name` to `fm`, which the store takes over; answers
/// whether it did. `update` as [`shada_set_global_mark`].
pub(crate) fn shada_set_local_mark(name: c_char, buffer: Buf, fm: ShadaMark, update: bool) -> bool {
    let Some(store) = LocalMark::named(c_int::from(name)) else {
        return false;
    };
    store.with(buffer, |target| {
        if update && fm.timestamp <= target.timestamp {
            return false;
        }
        *target = fm.into_fmark().0;
        true
    })
}

/// When `buffer`'s mark `name` was set, as [`mark_get`] resolves it; 0 for
/// one it does not.
pub(crate) fn shada_local_mark_timestamp(buffer: Buf, win: Win, name: c_int) -> Timestamp {
    mark_get(buffer, win, kMarkBufLocal, name).map_or(0, |fm| fm.timestamp)
}

/// The `"` mark the buffer list restores: `pos`, stamped now, releasing
/// what the store held.
pub(crate) fn shada_restore_last_cursor(mut buffer: Buf, pos: Pos) {
    buffer.b_last_cursor = FileMark {
        mark: pos,
        fnum: 0,
        timestamp: os_time(),
        view: FileMarkView::NONE,
        extra: None,
    };
}

/// Put a jump read from a file into the current window's jump list, which
/// is kept oldest first.
///
/// A jump the list already holds — same position, same file — is dropped
/// rather than inserted twice, and so is one older than a list that is
/// already full. A jump with a file name (no loaded buffer holds the file)
/// is compared by name, one without by buffer number.
pub(crate) fn shada_insert_jump(fm: ShadaMark) {
    let mut win = Win::current();
    let mut i = win.w_jumplistlen;
    while i > 0 {
        let existing = &win.w_jumplist[list_index(i - 1)];
        if existing.fmark.timestamp <= fm.timestamp {
            let same_file = match &fm.fname {
                Some(fname) => existing.file_name() == Some(fname.as_cstr()),
                None => fm.fnum == existing.fmark.fnum,
            };
            if marks_equal(existing.fmark.mark, fm.mark) && same_file {
                i = -1;
            }
            break;
        }
        i -= 1;
    }
    // A full list loses its oldest jump: the insertion below moves it to
    // the slot it then fills, which releases it.
    let len = win.w_jumplistlen;
    let i = marklist_insert(&mut win.w_jumplist, len, i);
    if i == -1 {
        return;
    }
    win.w_jumplist[list_index(i)] = fm.into_xfmark();
    if win.w_jumplistlen < JUMPLISTSIZE {
        win.w_jumplistlen += 1;
    }
    // Keep the cursor into the list pointing at the same jump.
    if win.w_jumplistidx >= i && win.w_jumplistidx < win.w_jumplistlen {
        win.w_jumplistidx += 1;
    }
}

/// [`shada_insert_jump`] for a buffer's change list, which needs no file
/// name to compare on because every entry in it is in this buffer.
pub(crate) fn shada_insert_change(mut buffer: Buf, fm: ShadaMark) {
    let mut i = buffer.b_changelistlen;
    while i > 0 {
        let existing = &buffer.b_changelist[list_index(i - 1)];
        if existing.timestamp <= fm.timestamp {
            if marks_equal(existing.mark, fm.mark) {
                i = -1;
            }
            break;
        }
        i -= 1;
    }
    // A full list loses its oldest change the same way.
    let len = buffer.b_changelistlen;
    let i = marklist_insert(&mut buffer.b_changelist, len, i);
    if i == -1 {
        return;
    }
    buffer.b_changelist[list_index(i)] = fm.into_fmark().0;
    if buffer.b_changelistlen < JUMPLISTSIZE {
        buffer.b_changelistlen += 1;
    }
}

/// A position in a jump or change list, which the walks above keep as a
/// `c_int` that is only negative as a "not at all" answer checked first.
fn list_index(i: c_int) -> usize {
    usize::try_from(i).expect("a position in the list")
}

/// Whether two marks are at the same place.
fn marks_equal(a: Pos, b: Pos) -> bool {
    a.lnum == b.lnum && a.col == b.col
}

/// The timestamp of global mark `idx` — `'A`-`'Z` then `'0`-`'9`.
///
/// The ShaDa merge asks for it to decide whether a file entry is newer than
/// the mark this Nvim already holds; it is the only thing outside `mark/`
/// that reads the global table directly.
pub(crate) fn global_mark_timestamp(idx: c_int) -> Timestamp {
    let idx = slot(idx);
    namedfm.with(|table| table[idx].fmark.timestamp)
}
