//! The surface the shada reader and writer drive, over owned values.
//!
//! The stores keep their marks as `FileMark`/`XFileMark` records whose file
//! name and ShaDa extra data are raw allocations the store owns. ShaDa's
//! entries own theirs, so everything here copies on the way out and takes
//! over on the way in: [`shada_global_marks`], [`shada_buffer_marks`],
//! [`shada_jumps`] and [`shada_changes`] answer copies, and the setters and
//! inserters take a [`ShadaMark`] by value and turn it into the store's
//! record. The raw end of the extra data -- `AdditionalData`, an `xmalloc`
//! block of a count, a length and the bytes -- is [`extra_into_raw`]
//! and [`extra_copy_raw`]; the file marks are the last store that keeps it.
//!
//! The two setters are the merge half. Neither overwrites a record that is
//! *newer* than the one being merged in, which is what `update` selects; the
//! comparison is on the timestamp every store carries and nothing else prints.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::memory::{XString, xmalloc};
use crate::os::time::os_time;
use crate::shada::{ShadaExtra, marklist_insert};
use crate::types::{AdditionalData, FileMark, FileMarkView, Pos, Timestamp, XFileMark};
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int};
use core::ptr;

use super::lookup::mark_global_index;
use super::store::{Fmark, GlobalMarks, mark_name};
use super::*;
use crate::ascii::ascii_islower;

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
            // SAFETY: a record in place owns its extra data, null or a live
            // block, which is only read here.
            extra: unsafe { extra_copy_raw(fm.additional_data) },
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
            additional_data: extra_into_raw(self.extra),
        };
        (fm, self.fname)
    }

    /// The store's record with a file name, taking both over.
    fn into_xfmark(self) -> XFileMark {
        let (fmark, fname) = self.into_fmark();
        XFileMark {
            fmark,
            fname: fname.map_or(ptr::null_mut(), XString::into_raw),
        }
    }
}

/// The extra data as the stores hold it: an `xmalloc` block, or null for
/// none. The block is the caller's to free.
fn extra_into_raw(extra: Option<ShadaExtra>) -> *mut AdditionalData {
    let Some(extra) = extra else {
        return ptr::null_mut();
    };
    let bytes = extra.bytes();
    let header = AdditionalData {
        nitems: extra.items(),
        // One entry's extra data cannot be 4 GiB long: saturating is the safe
        // reading of upstream's bare `(uint32_t)`, and unreachable either way.
        nbytes: u32::try_from(bytes.len()).unwrap_or(u32::MAX),
        data: [],
    };
    // SAFETY: a fresh block of the header's size plus the bytes, both
    // written before anything reads it.
    unsafe {
        let block = xmalloc(size_of::<AdditionalData>() + bytes.len()).cast::<AdditionalData>();
        block.write(header);
        (&raw mut (*block).data)
            .cast::<u8>()
            .copy_from_nonoverlapping(bytes.as_ptr(), bytes.len());
        block
    }
}

/// A copy of the extra data a store holds, which keeps its block.
///
/// # Safety
/// `raw` is null or a live block [`extra_into_raw`] (or the shada reader
/// before it) made.
unsafe fn extra_copy_raw(raw: *const AdditionalData) -> Option<ShadaExtra> {
    if raw.is_null() {
        return None;
    }
    // SAFETY: the caller's live block: a header and `nbytes` bytes after it.
    unsafe {
        let bytes = core::slice::from_raw_parts(
            (&raw const (*raw).data).cast::<u8>(),
            (*raw).nbytes as usize,
        );
        Some(ShadaExtra::from_parts((*raw).nitems, bytes.to_vec()))
    }
}

/// Every set global mark with its name, `'A`-`'Z` then `'0`-`'9`, copied.
pub(crate) fn shada_global_marks() -> Vec<(c_char, ShadaMark)> {
    let mut found = Vec::new();
    for (at, slot) in GlobalMarks::indexed() {
        if !slot.fmark().is_set() {
            continue;
        }
        // `'A`-`'Z` occupy `0..NMARKS` and `'0`-`'9` the rest.
        let name = mark_name(if at < NMARKS {
            'A' as c_int + at
        } else {
            '0' as c_int + at - NMARKS
        });
        let xfm = slot.read();
        let fname = xfm.file_name().map(XString::from_cstr);
        found.push((name, ShadaMark::of(&xfm.fmark, fname)));
    }
    found
}

/// Every set mark of `buffer` with its name, `'"`, `'^`, `'.` then
/// `'a`-`'z`, copied.
pub(crate) fn shada_buffer_marks(buffer: Buf) -> Vec<(c_char, ShadaMark)> {
    let special = [
        (b'"', buffer.last_cursor()),
        (b'^', buffer.last_insert()),
        (b'.', buffer.last_change()),
    ];
    let named = (0..NMARKS).map(|i| {
        let name = u8::try_from(i).map_or(0, |i| b'a' + i);
        (name, buffer.named_mark(i))
    });
    special
        .into_iter()
        .chain(named)
        .filter(|(_, mark)| mark.is_set())
        .map(|(name, mark)| {
            (
                mark_name(c_int::from(name)),
                ShadaMark::of(&mark.read(), None),
            )
        })
        .collect()
}

/// The jump list of `win`, oldest first, copied.
pub(crate) fn shada_jumps(win: Win) -> Vec<ShadaMark> {
    win.jumps()
        .map(|jump| {
            let xfm = jump.read();
            let fname = xfm.file_name().map(XString::from_cstr);
            ShadaMark::of(&xfm.fmark, fname)
        })
        .collect()
}

/// The change list of `buffer`, oldest first, copied.
pub(crate) fn shada_changes(buffer: Buf) -> Vec<ShadaMark> {
    buffer
        .changes()
        .map(|change| ShadaMark::of(&change.read(), None))
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
    let tgt = GlobalMarks::at(idx);
    // A merge never overwrites a record the editor made more recently than
    // the one the shada file carries; a plain set (`update` false) always does.
    if update && fm.timestamp <= tgt.fmark().timestamp() {
        return false;
    }
    if tgt.fmark().is_set() {
        // SAFETY: the slot's allocations are the table's to free, and it is
        // about to be overwritten.
        unsafe { free_xfmark(tgt.read()) };
    }
    tgt.write(fm.into_xfmark());
    true
}

/// The store `name` names in `buffer`, if it is one shada sets.
fn local_store(name: c_char, buffer: Buf) -> Option<Fmark> {
    let name = c_int::from(name);
    Some(if ascii_islower(name) {
        buffer.named_mark(name - 'a' as c_int)
    } else if name == '"' as c_int {
        buffer.last_cursor()
    } else if name == '^' as c_int {
        buffer.last_insert()
    } else if name == ':' as c_int {
        buffer.prompt_start()
    } else if name == '.' as c_int {
        buffer.last_change()
    } else {
        return None;
    })
}

/// Set `buffer`'s mark `name` to `fm`, which the store takes over; answers
/// whether it did. `update` as [`shada_set_global_mark`].
pub(crate) fn shada_set_local_mark(name: c_char, buffer: Buf, fm: ShadaMark, update: bool) -> bool {
    let Some(tgt) = local_store(name, buffer) else {
        return false;
    };
    if update && fm.timestamp <= tgt.timestamp() {
        return false;
    }
    if tgt.is_set() {
        // SAFETY: the store's allocation is the buffer's to free, and it is
        // about to be overwritten.
        unsafe { free_fmark(tgt.read()) };
    }
    tgt.write(fm.into_fmark().0);
    true
}

/// When `buffer`'s mark `name` was set, as [`mark_get`] resolves it; 0 for
/// one it does not.
pub(crate) fn shada_local_mark_timestamp(buffer: Buf, win: Win, name: c_int) -> Timestamp {
    let mut fm = FileMark::UNSET;
    // SAFETY: `fm` is this frame's record and outlives the call; only its
    // timestamp is read back.
    unsafe { mark_get(buffer, win, &raw mut fm, kMarkBufLocal, name) };
    fm.timestamp
}

/// The `"` mark the buffer list restores: `pos`, stamped now, releasing
/// what the store held.
pub(crate) fn shada_restore_last_cursor(buffer: Buf, pos: Pos) {
    let store = buffer.last_cursor();
    // SAFETY: the record in place owns its extra data.
    unsafe { free_fmark(store.read()) };
    store.write(FileMark {
        mark: pos,
        fnum: 0,
        timestamp: os_time(),
        view: FileMarkView::NONE,
        additional_data: ptr::null_mut(),
    });
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
    if i > 0 && win.w_jumplistlen == JUMPLISTSIZE {
        // SAFETY: the oldest jump is about to be overwritten, so what it
        // holds is this call's to release.
        unsafe { free_xfmark(win.w_jumplist[0].clone()) };
    }
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
    if i > 0 && buffer.b_changelistlen == JUMPLISTSIZE {
        // SAFETY: the oldest change is about to be overwritten, so what it
        // holds is this call's to release.
        unsafe { free_fmark(buffer.b_changelist[0].clone()) };
    }
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
    GlobalMarks::at(idx).read().fmark.timestamp
}
