//! The iterator and setter surface the shada reader and writer drive.
//!
//! Both iterators are C-shaped: the caller passes null to start and gets back
//! an opaque token that is really the address of the record just answered.
//! That token is what a subsequent call resumes from, which is why nothing may
//! edit a store while an iteration is in progress — the position is the
//! pointer, not an index the container could re-derive.
//!
//! The two setters are the merge half. Neither overwrites a record that is
//! *newer* than the one being merged in, which is what `update` selects; the
//! comparison is on the timestamp every store carries and nothing else prints.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::winlayer::Buf;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use super::lookup::*;
use super::store::{Fmark, GlobalMarks, NUL_BYTE, mark_name};
use super::*;
use crate::ascii::ascii_islower;

/// Iterate over global marks
///
/// @warning No mark-editing functions must be called while iteration is in
///          progress.
///
/// `iter` — Iterator. Pass NULL to start iteration.
/// `name` — Mark name.
/// `fm` — Mark definition.
///
/// Returns pointer that needs to be passed to next `mark_global_iter` call or
///         NULL if iteration is over.
///
/// # Safety
/// `name` and `fm` must point at live, writable storage, and `iter` must be
/// null or a value a previous call answered.
pub unsafe fn mark_global_iter(
    iter: *const c_void,
    name: *mut c_char,
    fm: *mut xfmark_T,
) -> *const c_void {
    // SAFETY: the caller promised writable out-parameters.
    unsafe { *name = NUL_BYTE };
    // The token is the address of a slot; turn it back into its index.
    let from = if iter.is_null() {
        0
    } else {
        GlobalMarks::index_of(iter.cast())
    };
    let Some(at) = set_global_at_or_after(from) else {
        return ptr::null();
    };
    // `'A`-`'Z` occupy `0..NMARKS` and `'0`-`'9` the rest — the inverse of
    // `mark_global_index`, and the fourth place the formula is written out.
    // SAFETY: as above.
    unsafe {
        *name = mark_name(if at < NMARKS {
            'A' as c_int + at
        } else {
            '0' as c_int + at - NMARKS
        });
        *fm = GlobalMarks::at(at).read();
    }
    match set_global_at_or_after(at + 1) {
        Some(next) => GlobalMarks::at(next).raw().cast(),
        None => ptr::null(),
    }
}

/// The first global slot at or after `from` that holds a mark, if any.
fn set_global_at_or_after(from: c_int) -> Option<c_int> {
    (from..NGLOBALMARKS).find(|&i| GlobalMarks::at(i).fmark().is_set())
}

/// The buffer mark after the one called `mark_name`, and its name.
///
/// The order is `'"`, `'^`, `'.`, then `'a`-`'z`; `NUL` starts it and `'z`
/// ends it. `mark_name` is both the cursor and the answer, because the
/// iterator token the caller holds is an address and the three tick marks are
/// separate fields rather than entries of an array.
///
/// # Safety
/// `buf` must be a live buffer and `mark_name` must point at live, writable
/// storage holding one of the names above.
#[inline]
pub(super) unsafe fn next_buffer_mark(buf: *const buf_T, mark_name: *mut c_char) -> *const fmark_T {
    // SAFETY: the caller promised a live buffer and a live cursor.
    let buf = unsafe { Buf::new(buf.cast_mut()) };
    // SAFETY: as above.
    let here = unsafe { *mark_name };
    let (next, mark): (c_char, Fmark) = match c_int::from(here) {
        NUL => ('"' as c_char, buf.last_cursor()),
        34 => ('^' as c_char, buf.last_insert()),
        94 => ('.' as c_char, buf.last_change()),
        46 => ('a' as c_char, buf.named_mark(0)),
        122 => return ptr::null(),
        _ => {
            let next = here + 1;
            (next, buf.named_mark(c_int::from(next) - 'a' as c_int))
        }
    };
    // SAFETY: as above.
    unsafe { *mark_name = next };
    mark.raw()
}

/// Iterate over buffer marks
///
/// @warning No mark-editing functions must be called while iteration is in
///          progress.
///
/// `iter` — Iterator. Pass NULL to start iteration.
/// `buf` — Buffer.
/// `name` — Mark name.
/// `fm` — Mark definition.
///
/// Returns pointer that needs to be passed to next `mark_buffer_iter` call or
///         NULL if iteration is over.
///
/// # Safety
/// `buf` must be a live buffer, `name` and `fm` must point at live, writable
/// storage, and `iter` must be null or a value a previous call answered for
/// the same buffer.
pub unsafe fn mark_buffer_iter(
    iter: *const c_void,
    buf: *const buf_T,
    name: *mut c_char,
    fm: *mut fmark_T,
) -> *const c_void {
    // SAFETY: the caller promised a live buffer and writable out-parameters.
    let bufh = unsafe { Buf::new(buf.cast_mut()) };
    // SAFETY: as above.
    unsafe { *name = NUL_BYTE };
    // Turn the token back into the name it stands for. The last arm reads
    // "how far into `b_namedm` this is, as a letter": upstream spells it as
    // an `offset('a')` before the `offset_from`, which is the same sum.
    let mut at: c_char = if iter.is_null() {
        NUL_BYTE
    } else if ptr::eq(iter.cast(), bufh.last_cursor().raw()) {
        '"' as c_char
    } else if ptr::eq(iter.cast(), bufh.last_insert().raw()) {
        '^' as c_char
    } else if ptr::eq(iter.cast(), bufh.last_change().raw()) {
        '.' as c_char
    } else {
        let base = bufh.named_mark(0).raw().addr();
        let bytes = iter.cast::<fmark_T>().addr().wrapping_sub(base);
        let idx = bytes.wrapping_div(size_of::<fmark_T>());
        mark_name(c_int::try_from(idx).unwrap_or(0) + 'a' as c_int)
    };
    // SAFETY: `buf` is live and `mark_name` is on this stack.
    let mut iter_mark = unsafe { next_buffer_mark(buf, &raw mut at) };
    while !iter_mark.is_null() {
        // SAFETY: every non-null answer names a live record of `buf`.
        if unsafe { Fmark::new(iter_mark.cast_mut()) }.is_set() {
            break;
        }
        // SAFETY: as above.
        iter_mark = unsafe { next_buffer_mark(buf, &raw mut at) };
    }
    if iter_mark.is_null() {
        return ptr::null();
    }
    // SAFETY: writable out-parameters, and a live record.
    unsafe {
        *name = at;
        *fm = (*iter_mark).clone();
    }
    iter_mark.cast()
}

/// Set global mark
///
/// `name` — Mark name.
/// `fm` — Mark to be set.
/// `update` — If true then only set global mark if it was created
///                     later then existing one.
///
/// Returns true on success, false on failure.
///
/// # Safety
/// The editor's globals must be live, and `fm`'s allocations must be handed
/// over to the table.
pub unsafe fn mark_set_global(name: c_char, fm: xfmark_T, update: bool) -> bool {
    let idx = mark_global_index(name);
    if idx == -1 {
        return false;
    }
    let tgt = GlobalMarks::at(idx);
    // A merge never overwrites a record the editor made more recently than
    // the one the shada file carries; a plain set (`update` false) always does.
    if update && fm.fmark.timestamp <= tgt.fmark().timestamp() {
        return false;
    }
    if tgt.fmark().is_set() {
        // SAFETY: the slot's allocations are the table's to free, and it is
        // about to be overwritten.
        unsafe { free_xfmark(tgt.read()) };
    }
    tgt.write(fm);
    true
}

/// Set local mark
///
/// `name` — Mark name.
/// `buf` — Pointer to the buffer to set mark in.
/// `fm` — Mark to be set.
/// `update` — If true then only set global mark if it was created
///                     later then existing one.
///
/// Returns true on success, false on failure.
///
/// # Safety
/// `buf` must be a live buffer, and `fm`'s allocations must be handed over to
/// the store.
pub unsafe fn mark_set_local(name: c_char, buf: *mut buf_T, fm: fmark_T, update: bool) -> bool {
    // SAFETY: the caller promised a live buffer.
    let bufh = unsafe { Buf::new(buf) };
    let name = c_int::from(name);
    let tgt: Fmark = if ascii_islower(name) {
        bufh.named_mark(name - 'a' as c_int)
    } else if name == '"' as c_int {
        bufh.last_cursor()
    } else if name == '^' as c_int {
        bufh.last_insert()
    } else if name == ':' as c_int {
        bufh.prompt_start()
    } else if name == '.' as c_int {
        bufh.last_change()
    } else {
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
    tgt.write(fm);
    true
}

/// The timestamp of global mark `idx` — `'A`-`'Z` then `'0`-`'9`.
///
/// The ShaDa merge asks for it to decide whether a file entry is newer than
/// the mark this Nvim already holds; it is the only thing outside `mark/`
/// that reads the global table directly.
pub(crate) fn global_mark_timestamp(idx: c_int) -> Timestamp {
    GlobalMarks::at(idx).read().fmark.timestamp
}
