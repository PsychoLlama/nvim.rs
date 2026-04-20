//! Inline block-level helpers for pointer blocks and data blocks.
//!
//! This module replaces the 28 C `nvim_pp_*`/`nvim_dp_*` round-trip accessor
//! functions with direct Rust implementations that operate on the in-memory
//! `PointerBlockHeader` + `pb_pointer[]` / `DataBlockHeader` + `db_index[]`
//! layouts defined in `types.rs`.
//!
//! # Memory layout assumptions
//!
//! - `pb_pointer[]` starts immediately after `PointerBlockHeader` (8 bytes).
//!   The C `PointerBlock` struct has the flexible array `pb_pointer[]` at that
//!   offset with no additional padding (validated by `test_layout_compat` in
//!   types.rs).
//!
//! - `db_index[]` starts immediately after `DataBlockHeader` (24 bytes).
//!   The C `DataBlock` struct has the flexible array `db_index[]` at that offset
//!   with no additional padding (same test).

// Index parameters are C `int` (i32) but used as array offsets; callers
// guarantee non-negative values, so sign-loss casts are safe here.
#![allow(clippy::cast_sign_loss)]

use std::ffi::{c_char, c_int, c_uint, c_void};

use nvim_buffer::buf_struct::BufStruct;

use crate::types::{
    DataBlockHeader, PointerBlockHeader, PointerEntry, DATA_BLOCK_HEADER_SIZE, DB_INDEX_MASK,
};

// =============================================================================
// Pointer-block helpers
// =============================================================================

/// Return a mutable pointer to `pb_pointer[0]`.
///
/// # Safety
/// `pp` must point to a valid `PointerBlock` (header + entries).
#[inline]
pub(crate) unsafe fn pb_pointer_ptr(pp: *mut c_void) -> *mut PointerEntry {
    #[allow(clippy::cast_ptr_alignment)]
    pp.cast::<u8>()
        .add(std::mem::size_of::<PointerBlockHeader>())
        .cast::<PointerEntry>()
}

/// Return a const pointer to `pb_pointer[0]`.
///
/// # Safety
/// `pp` must point to a valid `PointerBlock` (header + entries).
#[inline]
unsafe fn pb_pointer_ptr_const(pp: *const c_void) -> *const PointerEntry {
    #[allow(clippy::cast_ptr_alignment)]
    pp.cast::<u8>()
        .add(std::mem::size_of::<PointerBlockHeader>())
        .cast::<PointerEntry>()
}

/// Read `pb_id` from a pointer block.
///
/// # Safety
/// `pp` must point to a valid pointer block.
#[inline]
pub(crate) unsafe fn pp_get_id(pp: *const c_void) -> u16 {
    (*pp.cast::<PointerBlockHeader>()).pb_id
}

/// Read `pb_count` from a pointer block.
///
/// # Safety
/// `pp` must point to a valid pointer block.
#[inline]
pub(crate) unsafe fn pp_get_count(pp: *const c_void) -> u16 {
    (*pp.cast::<PointerBlockHeader>()).pb_count
}

/// Read `pb_count_max` from a pointer block.
///
/// # Safety
/// `pp` must point to a valid pointer block.
#[inline]
pub(crate) unsafe fn pp_get_count_max(pp: *const c_void) -> u16 {
    (*pp.cast::<PointerBlockHeader>()).pb_count_max
}

/// Write `pb_count`.
///
/// # Safety
/// `pp` must point to a valid pointer block.
#[inline]
pub(crate) unsafe fn pp_set_count(pp: *mut c_void, val: u16) {
    (*pp.cast::<PointerBlockHeader>()).pb_count = val;
}

/// Write `pb_count_max`.
///
/// # Safety
/// `pp` must point to a valid pointer block.
#[inline]
pub(crate) unsafe fn pp_set_count_max(pp: *mut c_void, val: u16) {
    (*pp.cast::<PointerBlockHeader>()).pb_count_max = val;
}

/// Increment `pb_count`, returning the new value.
///
/// # Safety
/// `pp` must point to a valid pointer block.
#[inline]
pub(crate) unsafe fn pp_inc_count(pp: *mut c_void) -> u16 {
    let h = pp.cast::<PointerBlockHeader>();
    (*h).pb_count += 1;
    (*h).pb_count
}

/// Decrement `pb_count`, returning the new value as `c_int`.
///
/// # Safety
/// `pp` must point to a valid pointer block.
#[inline]
pub(crate) unsafe fn pp_dec_count(pp: *mut c_void) -> c_int {
    let h = pp.cast::<PointerBlockHeader>();
    (*h).pb_count -= 1;
    c_int::from((*h).pb_count)
}

/// Calculate `pb_count_max` for the given memfile page size.
///
/// Replaces the C `PB_COUNT_MAX` macro / `nvim_pp_count_max_for_mfp`.
///
/// # Safety
/// `mfp` must be a valid `*mut memfile_T` pointer.
#[inline]
pub(crate) unsafe fn pp_count_max_for_mfp(mfp: *mut c_void) -> u16 {
    let page_size = nvim_mf_get_page_size(mfp) as usize;
    crate::types::pb_count_max(page_size)
}

extern "C" {
    pub(crate) fn nvim_mf_get_page_size(mfp: *mut c_void) -> c_uint;
}

// ---- pb_pointer[idx] field accessors ----------------------------------------

/// Read `pb_pointer[idx].pe_bnum`.
///
/// # Safety
/// `pp` must point to a valid pointer block with at least `idx + 1` entries.
#[inline]
pub(crate) unsafe fn pp_pe_get_bnum(pp: *const c_void, idx: c_int) -> i64 {
    (*pb_pointer_ptr_const(pp).add(idx as usize)).pe_bnum
}

/// Read `pb_pointer[idx].pe_line_count`.
///
/// # Safety
/// `pp` must point to a valid pointer block with at least `idx + 1` entries.
#[inline]
pub(crate) unsafe fn pp_pe_get_line_count(pp: *const c_void, idx: c_int) -> i64 {
    i64::from((*pb_pointer_ptr_const(pp).add(idx as usize)).pe_line_count)
}

/// Read `pb_pointer[idx].pe_page_count` (signed).
///
/// # Safety
/// `pp` must point to a valid pointer block with at least `idx + 1` entries.
#[inline]
pub(crate) unsafe fn pp_pe_get_page_count(pp: *const c_void, idx: c_int) -> c_int {
    (*pb_pointer_ptr_const(pp).add(idx as usize)).pe_page_count
}

/// Read `pb_pointer[idx].pe_page_count` (unsigned).
///
/// # Safety
/// `pp` must point to a valid pointer block with at least `idx + 1` entries.
#[inline]
pub(crate) unsafe fn pp_pe_get_page_count_uint(pp: *const c_void, idx: c_int) -> c_uint {
    (*pb_pointer_ptr_const(pp).add(idx as usize)).pe_page_count as c_uint
}

/// Read `pb_pointer[idx].pe_old_lnum`.
///
/// # Safety
/// `pp` must point to a valid pointer block with at least `idx + 1` entries.
#[inline]
pub(crate) unsafe fn pp_pe_get_old_lnum(pp: *const c_void, idx: c_int) -> i64 {
    i64::from((*pb_pointer_ptr_const(pp).add(idx as usize)).pe_old_lnum)
}

/// Write `pb_pointer[idx].pe_bnum`.
///
/// # Safety
/// `pp` must point to a valid pointer block with at least `idx + 1` entries.
#[inline]
pub(crate) unsafe fn pp_pe_set_bnum(pp: *mut c_void, idx: c_int, bnum: i64) {
    (*pb_pointer_ptr(pp).add(idx as usize)).pe_bnum = bnum;
}

/// Add `count` to `pb_pointer[idx].pe_line_count`.
///
/// # Safety
/// `pp` must point to a valid pointer block with at least `idx + 1` entries.
#[inline]
pub(crate) unsafe fn pp_pe_linecount_add(pp: *mut c_void, idx: c_int, count: c_int) {
    (*pb_pointer_ptr(pp).add(idx as usize)).pe_line_count += count;
}

/// Increment `pb_pointer[idx].pe_line_count`.
///
/// # Safety
/// `pp` must point to a valid pointer block with at least `idx + 1` entries.
#[inline]
pub(crate) unsafe fn pp_pe_inc_line_count(pp: *mut c_void, idx: c_int) {
    (*pb_pointer_ptr(pp).add(idx as usize)).pe_line_count += 1;
}

/// Decrement `pb_pointer[idx].pe_line_count`.
///
/// # Safety
/// `pp` must point to a valid pointer block with at least `idx + 1` entries.
#[inline]
pub(crate) unsafe fn pp_pe_dec_line_count(pp: *mut c_void, idx: c_int) {
    (*pb_pointer_ptr(pp).add(idx as usize)).pe_line_count -= 1;
}

/// `memmove` pointer-entry array elements within the same block.
///
/// Moves `count` entries starting at `src_idx` to `dst_idx`.
///
/// # Safety
/// `pp` must point to a valid pointer block; src/dst ranges must be in bounds.
#[inline]
pub(crate) unsafe fn pp_pe_memmove(pp: *mut c_void, dst_idx: c_int, src_idx: c_int, count: c_int) {
    let arr = pb_pointer_ptr(pp);
    std::ptr::copy(
        arr.add(src_idx as usize),
        arr.add(dst_idx as usize),
        count as usize,
    );
}

// =============================================================================
// Data-block helpers
// =============================================================================

/// Return a const pointer to `db_index[0]`.
///
/// # Safety
/// `dp` must point to a valid `DataBlock` (header + index array).
#[inline]
unsafe fn db_index_ptr_const(dp: *const c_void) -> *const u32 {
    #[allow(clippy::cast_ptr_alignment)]
    dp.cast::<u8>().add(DATA_BLOCK_HEADER_SIZE).cast::<u32>()
}

/// Read `db_id` from a data block.
///
/// # Safety
/// `dp` must point to a valid data block.
#[inline]
pub(crate) unsafe fn dp_get_id(dp: *const c_void) -> u16 {
    (*dp.cast::<DataBlockHeader>()).db_id
}

/// Read `db_txt_end`.
///
/// # Safety
/// `dp` must point to a valid data block.
#[inline]
pub(crate) unsafe fn dp_get_txt_end(dp: *const c_void) -> c_uint {
    (*dp.cast::<DataBlockHeader>()).db_txt_end
}

/// Write `db_txt_end`.
///
/// # Safety
/// `dp` must point to a valid data block.
#[inline]
pub(crate) unsafe fn dp_set_txt_end(dp: *mut c_void, val: c_uint) {
    (*dp.cast::<DataBlockHeader>()).db_txt_end = val;
}

/// Read `db_line_count`.
///
/// # Safety
/// `dp` must point to a valid data block.
#[inline]
pub(crate) unsafe fn dp_get_line_count(dp: *const c_void) -> i64 {
    (*dp.cast::<DataBlockHeader>()).db_line_count
}

/// Read `db_index[i] & DB_INDEX_MASK`.
///
/// # Safety
/// `dp` must point to a valid data block with at least `i + 1` index entries.
#[inline]
pub(crate) unsafe fn dp_get_index_masked(dp: *const c_void, i: c_int) -> c_uint {
    (*db_index_ptr_const(dp).add(i as usize)) & DB_INDEX_MASK
}

/// Check if `db_index[i]` address overruns the text area.
///
/// Returns `1` if `&db_index[i] >= (char *)dp + db_txt_start`, else `0`.
///
/// # Safety
/// `dp` must point to a valid data block.
#[inline]
pub(crate) unsafe fn dp_index_overruns_txt(dp: *const c_void, i: c_int) -> c_int {
    let h = dp.cast::<DataBlockHeader>();
    let index_addr = db_index_ptr_const(dp).add(i as usize).cast::<c_char>();
    let txt_start_addr = dp.cast::<c_char>().add((*h).db_txt_start as usize);
    c_int::from(index_addr >= txt_start_addr)
}

/// Return a pointer to text at `offset` bytes from the block start.
///
/// # Safety
/// `dp` must point to a valid data block; `offset` must be within the block.
#[inline]
pub(crate) unsafe fn dp_get_txt_ptr(dp: *const c_void, offset: c_uint) -> *const c_char {
    dp.cast::<c_char>().add(offset as usize)
}

/// Write NUL to `db_txt_end - 1` (terminate text area).
///
/// # Safety
/// `dp` must point to a valid data block whose `db_txt_end` is within bounds.
#[inline]
pub(crate) unsafe fn dp_write_nul_at_txt_end(dp: *mut c_void) {
    let h = dp.cast::<DataBlockHeader>();
    let off = (*h).db_txt_end as usize - 1;
    *dp.cast::<c_char>().add(off) = 0;
}

// =============================================================================
// Init functions (Phase 2)
// =============================================================================

/// Initialize the root pointer block (block 1) with one entry pointing to block 2.
///
/// # Safety
/// `pp` must point to a writable, fully-allocated pointer block.
pub(crate) unsafe fn pp_init_root(pp: *mut c_void) {
    let h = pp.cast::<PointerBlockHeader>();
    (*h).pb_count = 1;
    let entry = pb_pointer_ptr(pp);
    (*entry).pe_bnum = 2;
    (*entry).pe_page_count = 1;
    (*entry).pe_old_lnum = 1;
    (*entry).pe_line_count = 1; // line count after insertion
}

/// Initialize the first data block (block 2) with one empty line.
///
/// # Safety
/// `dp` must point to a writable, fully-allocated data block whose header
/// fields (`db_txt_start`, `db_txt_end`, `db_free`) are already initialized.
pub(crate) unsafe fn dp_init_empty_line(dp: *mut c_void) {
    let h = dp.cast::<DataBlockHeader>();
    // Move text start back one byte for the NUL-terminated empty line
    (*h).db_txt_start -= 1;
    // Account for one byte of text plus one index entry.
    // INDEX_SIZE is always 4 (sizeof(unsigned) on all supported platforms).
    (*h).db_free -= 5;
    (*h).db_line_count = 1;
    // Write the NUL byte at db_txt_start
    *dp.cast::<c_char>().add((*h).db_txt_start as usize) = 0;
    // Set db_index[0] to the offset of that NUL.
    // SAFETY: DATA_BLOCK_HEADER_SIZE (24) is a multiple of 4, so the pointer
    // is aligned for u32.
    #[allow(clippy::cast_ptr_alignment)]
    let idx_ptr = dp.cast::<u8>().add(DATA_BLOCK_HEADER_SIZE).cast::<u32>();
    *idx_ptr = (*h).db_txt_start;
}

use crate::types::BufHandle;

/// Initialize the memline fields in a `buf_T` to empty/zero state.
///
/// Replaces `nvim_buf_init_ml_empty`.
///
/// # Safety
/// `buf` must point to a valid `buf_T`.
pub(crate) unsafe fn buf_init_ml_empty(buf: *mut BufHandle) {
    let bs = buf.cast::<BufStruct>();
    (*bs).ml_stack_size = 0;
    (*bs).ml_stack = std::ptr::null_mut();
    (*bs).ml_stack_top = 0;
    (*bs).ml_locked = std::ptr::null_mut();
    (*bs).ml_line_lnum = 0;
    (*bs).ml_line_offset = 0;
    (*bs).ml_chunksize = std::ptr::null_mut();
    (*bs).ml_usedchunks = 0;
}

extern "C" {}
