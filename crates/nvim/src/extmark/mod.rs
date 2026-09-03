//! Extmarks: positions in a buffer that move with the text.
//!
//! Carved by the stage:
//!
//! | child | what |
//! | --- | --- |
//! | [`set`] | `extmark_set()` -- placing a mark |
//! | [`del`] | removing one mark, or a namespace's worth |
//! | [`get`] | reading marks back, and freeing them all |
//! | [`undo`] | recording and replaying a change's effect on marks |
//! | [`splice`] | moving marks when the text moves |
//!
//! What stays here is the `kExtmark*` operation and undo-object alphabet the
//! five share, the empty-container initialisers the marktree and the
//! namespace id maps are built from, and **the safe forms of everything this
//! family calls out to**.
//!
//! That last block is the whole shape of the rewrite. Every mark this module
//! places lives in a `MarkTree`, and `marktree.c` is not ported: its entry
//! points are still `unsafe extern "C"` over `*mut MarkTree` and
//! `*mut MarkTreeIter`, so an obligation has to be discharged somewhere. It
//! is discharged **once per entry point** here rather than once per call
//! site (there were 117 raw casts before), and the wrappers are *safe*
//! functions, because everything any of them needs is a live buffer, its own
//! marktree and an iterator -- which [`Buf`], `&mut MarkTree` and
//! `&mut MarkTreeIter` already carry. With those in hand the five children
//! are ordinary checked code, and [`splice`] has no `unsafe` at all.
//!
//! `buf->b_marktree` and `buf->b_extmark_ns` are *fields of `buf_T`*, so
//! taking `&mut` to either is safe once the pointer is a [`Buf`] -- the same
//! borrow-the-field lever `buffer_updates.rs` uses on its two kvecs. Each
//! borrow is momentary by construction: it lasts for the call that asked for
//! it and no longer, which matters because a decoration provider can re-enter
//! the editor while marks are being read.
//!
//! Original: `src/nvim/extmark.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_int, c_void};
use core::ptr;

use crate::buffer_updates::buf_updates_send_splice;
use crate::decoration::{
    SignCountHalf, buf_decor_remove, buf_put_decor, buf_signcols_count_range, decor_free,
    decor_redraw, decor_state_invalidate, decor_type_flags,
};
use crate::main::curbuf_splice_pending;
use crate::marktree::{
    marktree_clear, marktree_del_itr, marktree_get_alt, marktree_get_altpos, marktree_itr_current,
    marktree_itr_get, marktree_itr_get_ext, marktree_itr_get_overlap, marktree_itr_next,
    marktree_itr_step_overlap, marktree_lookup, marktree_lookup_ns, marktree_move,
    marktree_move_region, marktree_put, marktree_revise_meta, marktree_splice,
};
use crate::memline::ml_find_line_or_offset;
use crate::memory::xrealloc;
use crate::types::buffer::ExtmarkNs;
use crate::types::{
    DecorInline, ExtmarkInfoArray, ExtmarkOp, ExtmarkSplice, ExtmarkType, ExtmarkUndoObject, MTKey,
    MTPair, MTPos, MarkTree, MarkTreeIter, UndoObjectType, bcount_t, buf_T, colnr_T,
    extmark_undo_vec_t, int32_t, linenr_T, size_t, u_header_T, uint16_t, uint32_t, uint64_t,
};
use crate::undo::u_force_get_undo_header;
use crate::winlayer::Buf;

// The carve of the transpiled module; see each child's docs.
mod del;
mod get;
mod set;
mod splice;
mod undo;

pub use self::del::*;
pub use self::get::*;
pub use self::set::*;
pub use self::undo::*;

pub const kExtmarkSavePos: UndoObjectType = 3;
pub const kExtmarkMove: UndoObjectType = 1;
pub const kExtmarkSplice: UndoObjectType = 0;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
pub const kExtmarkNone: ExtmarkType = 1;
pub const KV_INITIAL_VALUE: ExtmarkInfoArray = ExtmarkInfoArray {
    size: 0,
    capacity: 0,
    items: ptr::null_mut(),
};
// ---------------------------------------------------------------------------
// The two containers hanging off a buffer

impl Buf {
    /// `buf->b_marktree`, where this buffer's marks live.
    fn marktree(&mut self) -> &mut MarkTree {
        &mut self.b_marktree
    }

    /// `buf->b_extmark_ns`: namespace id to the highest mark id handed out in
    /// it.
    fn extmark_ns(&mut self) -> &mut ExtmarkNs {
        &mut self.b_extmark_ns
    }
}

/// The buffer the editor is working in -- `curbuf`.
fn current_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

// ---------------------------------------------------------------------------
// `marktree.h`, in safe form
//
// One wrapper per entry point. Each needs a live tree and, where it takes
// one, an iterator; `&mut MarkTree` and `&mut MarkTreeIter` are exactly those
// promises, so every wrapper below is safe and its `unsafe` is the call
// itself. An `Option<&mut MarkTreeIter>` is the C's "or NULL" argument.

/// The iterator's raw key -- `mt_itr_rawkey(itr)`, `itr->x->key[itr->i]`.
///
/// Only valid while the iterator is positioned on a key, which every caller
/// here has just established with a lookup or a `marktree_itr_current`.
fn itr_rawkey(itr: &mut MarkTreeIter) -> &mut MTKey {
    // SAFETY: a positioned iterator points at a live node, and `i` is the
    // slot within it the tree itself put there.
    unsafe { &mut (*itr.x).key[itr.i as usize] }
}

fn tree_put(tree: &mut MarkTree, key: MTKey, end_row: c_int, end_col: c_int, end_right: bool) {
    // SAFETY: a live tree, and a key the caller has filled in.
    unsafe { marktree_put(tree, key, end_row, end_col, end_right) }
}

fn tree_del_itr(tree: &mut MarkTree, itr: &mut MarkTreeIter, rev: bool) -> uint64_t {
    // SAFETY: a live tree and an iterator positioned in it.
    unsafe { marktree_del_itr(tree, itr, rev) }
}

fn tree_lookup(tree: &mut MarkTree, id: uint64_t, itr: Option<&mut MarkTreeIter>) -> MTKey {
    // SAFETY: a live tree; the iterator is optional and is written, not read.
    unsafe { marktree_lookup(tree, id, itr) }
}

fn tree_lookup_ns(
    tree: &mut MarkTree,
    ns: uint32_t,
    id: uint32_t,
    end: bool,
    itr: Option<&mut MarkTreeIter>,
) -> MTKey {
    // SAFETY: as [`tree_lookup`].
    unsafe { marktree_lookup_ns(tree, ns, id, end, itr) }
}

fn tree_get_alt(tree: &mut MarkTree, mark: MTKey, itr: Option<&mut MarkTreeIter>) -> MTKey {
    // SAFETY: as [`tree_lookup`]; `mark` is a key read out of this tree.
    unsafe { marktree_get_alt(tree, mark, itr) }
}

fn tree_get_altpos(tree: &mut MarkTree, mark: MTKey, itr: Option<&mut MarkTreeIter>) -> MTPos {
    // SAFETY: as [`tree_get_alt`].
    unsafe { marktree_get_altpos(tree, mark, itr) }
}

fn tree_move(tree: &mut MarkTree, itr: &mut MarkTreeIter, row: c_int, col: c_int) {
    // SAFETY: a live tree and an iterator positioned in it.
    unsafe { marktree_move(tree, itr, row, col) }
}

fn tree_revise_meta(tree: &mut MarkTree, itr: &mut MarkTreeIter, old_key: MTKey) {
    // SAFETY: a live tree and an iterator positioned in it; `old_key` is the
    // key as it read before the caller edited its flags in place.
    unsafe { marktree_revise_meta(tree, itr, old_key) }
}

fn tree_clear(tree: &mut MarkTree) {
    // SAFETY: a live tree, whose nodes this frees and whose root it resets.
    unsafe { marktree_clear(tree) }
}

fn tree_splice(
    tree: &mut MarkTree,
    start: MTPos,
    old_row: c_int,
    old_col: c_int,
    new_row: c_int,
    new_col: c_int,
) -> bool {
    // SAFETY: a live tree; the extents are plain numbers.
    unsafe {
        marktree_splice(
            tree, start.row, start.col, old_row, old_col, new_row, new_col,
        )
    }
}

fn tree_move_region(
    tree: &mut MarkTree,
    start: MTPos,
    extent_row: c_int,
    extent_col: colnr_T,
    new_row: c_int,
    new_col: colnr_T,
) {
    marktree_move_region(
        tree, start.row, start.col, extent_row, extent_col, new_row, new_col,
    )
}

fn itr_get(tree: &mut MarkTree, row: int32_t, col: c_int, itr: &mut MarkTreeIter) -> bool {
    // SAFETY: a live tree and an iterator this positions in it.
    unsafe { marktree_itr_get(tree, row, col, itr) }
}

/// `marktree_itr_get_ext` with the two arguments this family never varies:
/// no `oldbase` out-parameter and no metadata filter.
fn itr_get_ext(tree: &mut MarkTree, p: MTPos, itr: &mut MarkTreeIter) -> bool {
    // SAFETY: as [`itr_get`]; both optional out-parameters are NULL, which
    // the callee tests for.
    unsafe { marktree_itr_get_ext(tree, p, itr, false, false, None, None) }
}

fn itr_get_overlap(tree: &mut MarkTree, row: c_int, col: c_int, itr: &mut MarkTreeIter) -> bool {
    // SAFETY: as [`itr_get`].
    unsafe { marktree_itr_get_overlap(tree, row, col, itr) }
}

fn itr_step_overlap(tree: &mut MarkTree, itr: &mut MarkTreeIter, pair: &mut MTPair) -> bool {
    // SAFETY: a live tree, an iterator `marktree_itr_get_overlap` positioned,
    // and a pair the callee writes.
    unsafe { marktree_itr_step_overlap(tree, itr, pair) }
}

/// The key the iterator is on, or `MT_INVALID_KEY` (row -1) past the end.
fn itr_current(itr: &mut MarkTreeIter) -> MTKey {
    // SAFETY: the callee tests `itr->x` for NULL itself.
    unsafe { marktree_itr_current(itr) }
}

fn itr_next(tree: &mut MarkTree, itr: &mut MarkTreeIter) -> bool {
    // SAFETY: a live tree and an iterator positioned in it.
    unsafe { marktree_itr_next(tree, itr) }
}

// ---------------------------------------------------------------------------
// The other calls out of the family
//
// Every one of these wants a live buffer, which is [`Buf`]'s promise, so
// these wrappers are safe too.

fn decor_remove(buf: Buf, row1: c_int, row2: c_int, col1: c_int, decor: DecorInline, free: bool) {
    // SAFETY: a live buffer and a decoration read out of one of its marks.
    unsafe { buf_decor_remove(buf.raw(), row1, row2, col1, decor, free) }
}

fn put_decor(buf: Buf, decor: DecorInline, row: c_int, row2: c_int) {
    // SAFETY: as [`decor_remove`].
    unsafe { buf_put_decor(buf.raw(), decor, row, row2) }
}

fn redraw_decor(buf: Buf, row1: c_int, row2: c_int, col1: c_int, decor: DecorInline) {
    // SAFETY: as [`decor_remove`].
    unsafe { decor_redraw(buf.raw(), row1, row2, col1, decor) }
}

fn free_decor(decor: DecorInline) {
    // SAFETY: an inline decoration, or an index into the decoration arena
    // that this releases exactly once -- the caller's business, not ours.
    unsafe { decor_free(decor) }
}

fn type_flags(decor: DecorInline) -> uint16_t {
    // SAFETY: as [`free_decor`], and this only reads.
    unsafe { decor_type_flags(decor) }
}

fn invalidate_decor_state(buf: Buf) {
    // SAFETY: a live buffer.
    unsafe { decor_state_invalidate(buf.raw()) }
}

fn signcols_count_range(buf: Buf, row1: c_int, row2: c_int, add: c_int, half: SignCountHalf) {
    // SAFETY: a live buffer, whose own marktree this walks.
    unsafe { buf_signcols_count_range(buf.raw(), row1, row2, add, half) }
}

/// The highest extmark id handed out in namespace `key`, registering the
/// namespace at 0 if it was not there -- upstream's `map_put_ref` followed by
/// a read of the slot.
fn ns_counter(map: &mut ExtmarkNs, key: uint32_t) -> uint32_t {
    *map.entry(key).or_insert(0)
}

/// Store `value` as namespace `key`'s highest id.
fn ns_set_counter(map: &mut ExtmarkNs, key: uint32_t, value: uint32_t) {
    map.insert(key, value);
}

/// Whether the buffer holds, or has held, an extmark in namespace `key`.
fn ns_has(map: &ExtmarkNs, key: uint32_t) -> bool {
    map.contains_key(&key)
}

/// `map_del(uint32_t, uint32_t)`.
fn ns_del(map: &mut ExtmarkNs, key: uint32_t) {
    map.remove(&key);
}

/// Forget every namespace. The table is a `buf_T` field, so it keeps its
/// allocation -- which is what upstream's `map_destroy` plus the `MAP_INIT`
/// it always wrote over it amounted to.
fn ns_destroy(map: &mut ExtmarkNs) {
    map.clear();
}

/// `ml_find_line_or_offset(buf, lnum, NULL, true)`: the byte offset of a
/// line, counted with the file format's line endings ignored.
fn line_offset(buf: Buf, lnum: linenr_T) -> c_int {
    // SAFETY: a live buffer; the `offp` out-parameter is NULL, which the
    // callee tests for.
    unsafe { ml_find_line_or_offset(buf.raw(), lnum, ptr::null_mut(), true) }
}

/// `u_force_get_undo_header(buf)`, and then the extmark list on it. NULL when
/// the change is not undoable.
fn undo_marks(buf: Buf) -> *mut extmark_undo_vec_t {
    // SAFETY: a live buffer.
    let uhp: *mut u_header_T = unsafe { u_force_get_undo_header(buf.raw()) };
    if uhp.is_null() {
        return ptr::null_mut();
    }
    // SAFETY: a non-NULL undo header the editor owns; the list is a field of
    // it, and the borrow it hands back is the caller's to keep momentary.
    unsafe { &raw mut (*uhp).uh_extmark }
}

/// `curbuf_splice_pending`: a `:global` or similar is batching its own
/// splices and [`extmark_adjust`] must stay out of the way.
fn splice_pending() -> bool {
    curbuf_splice_pending.get() != 0
}

/// One (row, col, byte) triple of a splice -- its start, the extent removed
/// or the extent inserted, in the order `buf_updates_send_splice` takes them.
#[derive(Clone, Copy, Default)]
pub(crate) struct Extent {
    pub row: c_int,
    pub col: colnr_T,
    pub byte: bcount_t,
}

/// `buf_updates_send_splice`: the `on_bytes` half of every change.
fn send_splice(buf: Buf, start: Extent, old: Extent, new: Extent) {
    // SAFETY: a live buffer. This re-enters the editor through the update
    // callbacks, which is why no borrow of the buffer spans the call.
    unsafe {
        buf_updates_send_splice(
            buf.raw(),
            start.row,
            start.col,
            start.byte,
            old.row,
            old.col,
            old.byte,
            new.row,
            new.col,
            new.byte,
        );
    }
}

// ---------------------------------------------------------------------------
// klib's `kvec_t`

/// `kv_push`, over a vector's three fields.
///
/// The family pushes to two of them -- [`get`]'s answer array and the undo
/// header's `uh_extmark` -- and does nothing else with either, so this is the
/// whole of klib's growable vector that is needed here.
fn kv_push<T>(size: &mut size_t, capacity: &mut size_t, items: &mut *mut T, value: T) {
    if *size == *capacity {
        *capacity = if *capacity != 0 { *capacity << 1 } else { 8 };
        let bytes = size_of::<T>() * *capacity;
        let old = items.cast::<c_void>();
        // SAFETY: `items` is NULL or this vector's own allocation, and the
        // new size counts the same element type.
        *items = unsafe { xrealloc(old, bytes) }.cast::<T>();
    }
    let end = *size;
    *size = end + 1;
    // SAFETY: slot `end` is inside the allocation just made big enough.
    unsafe { *items.add(end) = value };
}

/// `kv_push(*uvp, undo)`, onto an undo header's extmark list.
fn push_undo(uvp: *mut extmark_undo_vec_t, undo: ExtmarkUndoObject) {
    // SAFETY: the caller's promise -- an undo header's own list, which
    // [`undo_marks`] answered or the caller was handed.
    let uvp = unsafe { &mut *uvp };
    kv_push(&mut uvp.size, &mut uvp.capacity, &mut uvp.items, undo);
}

/// The `ExtmarkSplice` of the list's last entry, if it has one and it is a
/// splice -- `&kv_A(v, kv_size(v) - 1).data.splice`, guarded as upstream
/// guards it. The borrow is unbounded and must not outlive the header: the
/// one caller drops it before the next push.
fn last_splice<'a>(uvp: *mut extmark_undo_vec_t) -> Option<&'a mut ExtmarkSplice> {
    // SAFETY: as [`push_undo`].
    let uvp = unsafe { &mut *uvp };
    if uvp.size == 0 {
        return None;
    }
    // SAFETY: `size - 1` is the last slot pushed, and the vector's element
    // type is what was pushed into it.
    let item = unsafe { &mut *uvp.items.add(uvp.size - 1) };
    match item {
        ExtmarkUndoObject::Splice(splice) => Some(splice),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// The `splice` stage's entry points
//
// Their bodies are in [`splice`], which forbids `unsafe` -- so the one line
// each that turns the caller's `buf_T *` into a [`Buf`] lives here, in the
// file already spending the row.

/// Adjust extmark rows for inserted or deleted rows; columns stay fixed.
pub unsafe fn extmark_adjust(
    buf: *mut buf_T,
    line1: linenr_T,
    line2: linenr_T,
    amount: linenr_T,
    amount_after: linenr_T,
    undo: ExtmarkOp,
) {
    // SAFETY: the caller's promise -- a live buffer.
    let buf = unsafe { Buf::new(buf) };
    splice::adjust(buf, line1, line2, amount, amount_after, undo);
}

/// Adjust extmarks after a text edit, and emit the `on_bytes` event
/// (`:h api-buffer-updates`).
///
/// `old_col` and `new_col` encode an offset from `start_col` when the
/// matching row extent is 0, and the end column of the region otherwise.
pub unsafe fn extmark_splice(
    buf: *mut buf_T,
    start_row: c_int,
    start_col: colnr_T,
    old_row: c_int,
    old_col: colnr_T,
    old_byte: bcount_t,
    new_row: c_int,
    new_col: colnr_T,
    new_byte: bcount_t,
    undo: ExtmarkOp,
) {
    let old = Extent {
        row: old_row,
        col: old_col,
        byte: old_byte,
    };
    let new = Extent {
        row: new_row,
        col: new_col,
        byte: new_byte,
    };
    // SAFETY: the caller's promise -- a live buffer.
    let buf = unsafe { Buf::new(buf) };
    splice::splice(buf, start_row, start_col, old, new, undo);
}

/// The single-line shorthand: the column delta is both the column count and
/// the byte count.
pub unsafe fn extmark_splice_cols(
    buf: *mut buf_T,
    start_row: c_int,
    start_col: colnr_T,
    old_col: colnr_T,
    new_col: colnr_T,
    undo: ExtmarkOp,
) {
    let old = Extent {
        row: 0,
        col: old_col,
        byte: old_col as bcount_t,
    };
    let new = Extent {
        row: 0,
        col: new_col,
        byte: new_col as bcount_t,
    };
    // SAFETY: the caller's promise -- a live buffer.
    let buf = unsafe { Buf::new(buf) };
    splice::splice(buf, start_row, start_col, old, new, undo);
}

/// Text removed from one place and inserted at another, as `:move` does it.
pub unsafe fn extmark_move_region(
    buf: *mut buf_T,
    start_row: c_int,
    start_col: colnr_T,
    start_byte: bcount_t,
    extent_row: c_int,
    extent_col: colnr_T,
    extent_byte: bcount_t,
    new_row: c_int,
    new_col: colnr_T,
    new_byte: bcount_t,
    undo: ExtmarkOp,
) {
    let start = Extent {
        row: start_row,
        col: start_col,
        byte: start_byte,
    };
    let extent = Extent {
        row: extent_row,
        col: extent_col,
        byte: extent_byte,
    };
    let new = Extent {
        row: new_row,
        col: new_col,
        byte: new_byte,
    };
    // SAFETY: the caller's promise -- a live buffer.
    let buf = unsafe { Buf::new(buf) };
    splice::move_region(buf, start, extent, new, undo);
}
