//! The xmalloc allocation family, memory/string helpers, and the block
//! pool: safe cores + C-ABI shims.
//!
//! [`alloc_log`] is how a test asserts an exact allocation sequence: the four
//! `seam_*` wrappers below are this module's only calls into the platform
//! allocator, and each one logs what it asked for while a recorder is armed.
//! They replaced the `mem_malloc`/`mem_free`/`mem_calloc`/`mem_realloc`
//! function pointers the LuaJIT unit suite used to rebind at runtime, which
//! is why no Rust container may replace an `xmalloc` here and no call may be
//! elided: the *sequence* is observable.
//!
//! Copy helpers whose C originals used `memmove` (`xstrlcat`, which the unit
//! suite calls with `src` pointing into `dst`) keep raw `ptr::copy`; slices
//! must never alias. Helpers that scan for a NUL only construct slices up to
//! the terminator, because the C originals never read past it and the
//! allocation may end there.
//!
//! # Boundary
//!
//! Five exports (`xmalloc`/`xcalloc`/`xfree`/`xstrdup`/`xmemdupz`) are still
//! pinned by the ABI ledger and named from `test/unit`. Their signatures are
//! observable, and so is the allocation sequence — through [`alloc_log`], for
//! every caller in the crate.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `memory/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use crate::global_cell::GlobalCell;
use crate::mbyte::Codepoint;
use crate::memory::alloc_log::AllocEvent;
use crate::semsg;
use core::ffi::{CStr, c_char, c_ulong, c_void};
use core::ptr;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::memfile::mf_release_all;
use crate::message::E_OUTOFMEM;
use crate::message::clear_sb_text;
use crate::message::state::EMSG_SILENT;
use crate::os::libc::size_t;
use crate::startup::{fallibly, preserve_exit};
use ::libc::{calloc, free, malloc, realloc};

/// Ask the editor to give memory back: drop the scrollback, release
/// memfile blocks, hand the block pool's spare blocks to the allocator. Runs at
/// most once at a time, since everything it calls may allocate.
///
/// # Safety
///
/// It frees the message scrollback, every memfile's cached blocks and the
/// block pool's spare blocks, so no reference into any of them may be live
/// across the call; the allocation paths that call it hold none. A memfile
/// whose memline is lent -- to a walk, or a line of it as a `LineText` -- is
/// passed over, so those may be. Main thread only: those are main-thread
/// state.
unsafe fn try_to_free_memory() {
    static TRYING_TO_FREE: AtomicBool = AtomicBool::new(false);
    if TRYING_TO_FREE.swap(true, Ordering::Relaxed) {
        return;
    }
    clear_sb_text(true);
    mf_release_all();
    free_reuse_blks();
    TRYING_TO_FREE.store(false, Ordering::Relaxed);
}

/// Report an allocation failure, once per process.
fn do_outofmem_msg(size: usize) {
    if DID_OUTOFMEM_MSG.get() {
        return;
    }
    // Message queueing would fail the allocation again; report loudly, once.
    EMSG_SILENT.set(0);
    DID_OUTOFMEM_MSG.set(true);
    // A `size_t` that would not fit a `%lu` is not a size any allocator was
    // ever going to serve, so saturating beats panicking on the OOM path.
    let size = c_ulong::try_from(size).unwrap_or(c_ulong::MAX);
    semsg!("E342: Out of memory!  (allocating {} bytes)", size);
}

/// The four calls below are the whole of this module's contact with the
/// platform allocator, and the only place [`alloc_log`] observes. Keeping
/// them in one place is what lets a test assert an exact allocation sequence
/// without the recorder being spread through the file.
///
/// # Safety
///
/// As [`try_malloc`].
unsafe fn seam_malloc(size: usize) -> *mut c_void {
    // SAFETY: the platform allocator.
    let ret = unsafe { malloc(size) };
    alloc_log::record(AllocEvent::Malloc { size, ret });
    ret
}

/// As [`seam_malloc`], for `calloc`.
///
/// # Safety
///
/// As [`try_malloc`].
unsafe fn seam_calloc(count: usize, size: usize) -> *mut c_void {
    // SAFETY: the platform allocator.
    let ret = unsafe { calloc(count, size) };
    alloc_log::record(AllocEvent::Calloc { count, size, ret });
    ret
}

/// As [`seam_malloc`], for `realloc`.
///
/// # Safety
///
/// `ptr` is null or an allocation from this module's family; otherwise as
/// [`try_malloc`].
unsafe fn seam_realloc(ptr: *mut c_void, size: usize) -> *mut c_void {
    // SAFETY: the caller's allocation, handed back to the allocator that
    // made it.
    let ret = unsafe { realloc(ptr, size) };
    alloc_log::record(AllocEvent::Realloc { ptr, size, ret });
    ret
}

/// As [`seam_malloc`], for `free`.
///
/// # Safety
///
/// `ptr` is null or an allocation from this module's family, not yet freed.
unsafe fn seam_free(ptr: *mut c_void) {
    // SAFETY: the caller's allocation, handed back to the allocator that
    // made it.
    unsafe { free(ptr) };
    alloc_log::record(AllocEvent::Free { ptr });
}

/// `malloc`, retried once after asking the editor for memory back. NULL when
/// even that did not help.
///
/// # Safety
///
/// As [`try_to_free_memory`], which it calls when the first attempt fails:
/// no reference into the scrollback, a memfile's cached blocks or the block
/// pool's spare blocks may be live across the call. A non-null answer is an
/// allocation of at least `size` bytes the caller owns and frees with
/// [`xfree`].
pub unsafe fn try_malloc(size: usize) -> *mut c_void {
    // A zero-byte malloc may answer NULL, which every caller here reads as
    // failure; upstream rounds up for the same reason.
    let allocated_size = size.max(1);
    // SAFETY: the platform allocator, and the retry after freeing.
    let ret = unsafe { seam_malloc(allocated_size) };
    if !ret.is_null() {
        return ret;
    }
    unsafe { try_to_free_memory() };
    unsafe { seam_malloc(allocated_size) }
}

/// Where the first `needle` in `hay` is: the C library's `memchr`, whose
/// vectorised search a reader scanning every byte of a file needs -- core's
/// costs several times as much per byte.
pub(crate) fn memchr_in(hay: &[u8], needle: u8) -> Option<usize> {
    // An empty slice's pointer is dangling, and `memchr` is handed a pointer
    // it may dereference whatever the length (Miri holds it to that).
    if hay.is_empty() {
        return None;
    }
    // SAFETY: `hay` is readable for its length; the answer is null or a
    // pointer into it.
    let needle = ::core::ffi::c_int::from(needle);
    let found = unsafe { ::libc::memchr(hay.as_ptr().cast(), needle, hay.len()) };
    (!found.is_null()).then(|| found.addr() - hay.as_ptr().addr())
}

/// Where the first `needle` in the NUL-terminated `hay` is, before its first
/// NUL: the C library's `strchr`, which finds the needle and the terminator
/// in one vectorised pass.
///
/// # Panics
/// If `hay` does not end in a NUL -- which is what keeps the search inside
/// it.
pub(crate) fn strchr_in(hay: &[u8], needle: u8) -> Option<usize> {
    assert_eq!(hay.last(), Some(&0), "strchr wants a terminated string");
    if needle == 0 {
        return None;
    }
    if needle >= 0x80 {
        // A codepoint past ASCII is searched for as its UTF-8 sequence, as
        // upstream's `vim_strchr` did; only the ASCII search is libc's.
        let text = &hay[..strlen_in(hay)];
        return crate::strings::find_char(text, Codepoint::from(needle));
    }
    // SAFETY: `hay` holds a NUL, which the search stops at; the answer is
    // null or a pointer into `hay` before it.
    let found =
        unsafe { crate::os::cshim::strchr(hay.as_ptr().cast(), ::core::ffi::c_int::from(needle)) };
    (!found.is_null()).then(|| found.addr() - hay.as_ptr().addr())
}

/// How long the NUL-terminated `hay` is, its NUL left out: the C library's
/// `strlen`.
///
/// # Panics
/// If `hay` does not end in a NUL.
pub(crate) fn strlen_in(hay: &[u8]) -> usize {
    assert_eq!(hay.last(), Some(&0), "strlen wants a terminated string");
    // SAFETY: `hay` holds a NUL, which the count stops at.
    unsafe { CStr::from_ptr(hay.as_ptr().cast()) }.count_bytes()
}

/// Sort `items` with the C library's `qsort`, ordered by `cmp`.
///
/// For the callers whose output shows the order `qsort` leaves equal
/// elements in -- which a Rust sort, stable or not, does not reproduce.
/// glibc's `qsort` is its `qsort_r` with no context, so the order is the
/// same; the context is how the comparison reaches `cmp`.
pub(crate) fn qsort_by<T: Copy, F: FnMut(&T, &T) -> core::cmp::Ordering>(
    items: &mut [T],
    mut cmp: F,
) {
    /// # Safety
    ///
    /// As `qsort_r`'s comparison: `a` and `b` point at two `T`s of the slice
    /// being sorted and `context` at the `F` it was given.
    unsafe extern "C" fn compare<T, F: FnMut(&T, &T) -> core::cmp::Ordering>(
        a: *const c_void,
        b: *const c_void,
        context: *mut c_void,
    ) -> ::core::ffi::c_int {
        // SAFETY: `qsort_r` hands back two elements of the slice and the
        // context it was given, which is the closure below.
        let (a, b, cmp) = unsafe { (&*a.cast::<T>(), &*b.cast::<T>(), &mut *context.cast::<F>()) };
        cmp(a, b) as ::core::ffi::c_int
    }
    if items.len() < 2 {
        return;
    }
    let (base, len, width) = (
        items.as_mut_ptr().cast::<c_void>(),
        items.len(),
        size_of::<T>(),
    );
    // SAFETY: `len` elements of `width` bytes at `base`, which `qsort_r`
    // moves as bytes -- `T: Copy` makes that a valid move -- and the
    // comparison over them with its context.
    unsafe {
        ::libc::qsort_r(
            base,
            len,
            width,
            Some(compare::<T, F>),
            (&raw mut cmp).cast(),
        )
    };
}

/// Grow `bytes` to `len` bytes, zero-filling the new ones, the way
/// upstream's `verbose_try_malloc` allocates: retried once after asking the editor
/// for memory back, and reported to the user when even that did not help.
/// False, with `bytes` unchanged, when the memory could not be had.
pub(crate) fn try_grow_zeroed(bytes: &mut Vec<u8>, len: usize) -> bool {
    let more = len.saturating_sub(bytes.len());
    if !try_reserve_items(bytes, more) {
        return false;
    }
    bytes.resize(len, 0);
    true
}

/// Make room in `items` for `more` items the way upstream's
/// `verbose_try_malloc` allocates: retried once after asking the editor for
/// memory back, and reported to the user (`E342`) when even that did not
/// help. False, with `items` unchanged, when the memory could not be had;
/// the request never ends the process.
pub(crate) fn try_reserve_items<T>(items: &mut Vec<T>, more: usize) -> bool {
    if fallibly(|| items.try_reserve_exact(more)).is_ok() {
        return true;
    }
    // SAFETY: an allocation is what is under way, on the main thread.
    unsafe { try_to_free_memory() };
    if fallibly(|| items.try_reserve_exact(more)).is_ok() {
        return true;
    }
    let bytes = items
        .len()
        .saturating_add(more)
        .saturating_mul(size_of::<T>().max(1));
    do_outofmem_msg(bytes);
    false
}

/// `malloc` that never fails: an allocation that cannot be served ends the
/// process through `preserve_exit`, which writes the swapfiles out first.
///
/// Carries the C ABI because tree-sitter takes it as an allocator hook
/// (`ts_set_allocator`), as [`xcalloc`], [`xrealloc`] and [`xfree`] do.
///
/// # Safety
///
/// As [`try_malloc`].
pub unsafe extern "C" fn xmalloc(size: usize) -> *mut c_void {
    // SAFETY: the caller's promise; `E_OUTOFMEM` is a NUL-terminated static.
    unsafe {
        let ret = try_malloc(size);
        if ret.is_null() {
            preserve_exit(Some(E_OUTOFMEM));
        }
        ret
    }
}

/// `free`, through the seam. A null pointer is a no-op, as `free`'s is.
///
/// Carries the C ABI as tree-sitter's allocator hook; see [`xmalloc`].
///
/// # Safety
///
/// `ptr` is null or an allocation from this module's family, not yet freed.
pub unsafe extern "C" fn xfree(ptr: *mut c_void) {
    // SAFETY: the caller's allocation, handed back to the seam that made it.
    unsafe { seam_free(ptr) };
}

/// `calloc` that never fails. The zeroing is load-bearing: callers all over
/// the tree treat a fresh allocation as initialized.
///
/// Carries the C ABI as tree-sitter's allocator hook; see [`xmalloc`].
///
/// # Safety
///
/// As [`try_malloc`].
pub unsafe extern "C" fn xcalloc(count: usize, size: usize) -> *mut c_void {
    // As in `try_malloc`: a zero-sized request must still answer a pointer.
    let (allocated_count, allocated_size) = if count != 0 && size != 0 {
        (count, size)
    } else {
        (1, 1)
    };
    // SAFETY: the platform allocator.
    let ret = unsafe { seam_calloc(allocated_count, allocated_size) };
    if !ret.is_null() {
        return ret;
    }
    // SAFETY: the caller's promise.
    unsafe { xcalloc_retry(allocated_count, allocated_size) }
}

/// [`xcalloc`] after the allocator said no: free what can be freed, try
/// once more, and exit if that fails too.
///
/// Out of line so `xcalloc` stays small enough to inline: with the retry
/// in it, the allocator seam folded in and every list and dict allocation
/// paid a call (evalbench +0.3 %).
///
/// # Safety
///
/// As [`try_malloc`].
#[cold]
#[inline(never)]
unsafe fn xcalloc_retry(count: usize, size: usize) -> *mut c_void {
    // SAFETY: the retry after freeing, and a NUL-terminated static for the
    // exit message.
    unsafe { try_to_free_memory() };
    let ret = unsafe { seam_calloc(count, size) };
    if ret.is_null() {
        preserve_exit(Some(E_OUTOFMEM));
    }
    ret
}

/// `realloc` that never fails.
///
/// # Safety
///
/// `ptr` is null or an allocation from this module's family; the caller drops
/// it in favour of the result. Otherwise as [`try_malloc`].
pub unsafe extern "C" fn xrealloc(ptr: *mut c_void, size: usize) -> *mut c_void {
    let allocated_size = size.max(1);
    // SAFETY: the caller's allocation, handed to the seam that made it, plus
    // the retry after freeing.
    let ret = unsafe { seam_realloc(ptr, allocated_size) };
    if !ret.is_null() {
        return ret;
    }
    unsafe { try_to_free_memory() };
    let ret = unsafe { seam_realloc(ptr, allocated_size) };
    if ret.is_null() {
        preserve_exit(Some(E_OUTOFMEM));
    }
    ret
}

/// A C string's bytes, terminator excluded.
///
/// # Safety
///
/// `s` is NUL-terminated and stays valid for as long as the result is used.
unsafe fn cbytes<'a>(s: *const c_char) -> &'a [u8] {
    // SAFETY: the caller's promise.
    unsafe { CStr::from_ptr(s) }.to_bytes()
}

/// Copy `len` bytes. The two regions must not overlap -- every caller here
/// is a fresh allocation or a distinct buffer; `xstrlcat` is the one that
/// may alias, and it uses `ptr::copy`.
///
/// # Safety
///
/// `src` is readable and `dst` writable for `len` bytes, and they do not
/// overlap.
unsafe fn copy_bytes(dst: *mut c_void, src: *const c_void, len: usize) {
    // The guard is not hygiene: a zero-length `copy_nonoverlapping` still
    // demands non-null pointers, and several callers here pass a null
    // source with a zero length.
    if len != 0 {
        // SAFETY: the caller's promise.
        unsafe { ptr::copy_nonoverlapping(src.cast::<u8>(), dst.cast::<u8>(), len) };
    }
}

/// Duplicate `len` bytes into a fresh NUL-terminated allocation.
///
/// # Safety
///
/// `data` is readable for `len` bytes; otherwise as [`try_malloc`].
pub unsafe fn xmemdupz(data: *const c_void, len: usize) -> *mut c_void {
    // `len` readable bytes put it at most `isize::MAX`, so the terminator's
    // byte cannot overflow the size.
    // SAFETY: `xmalloc` answers `len + 1` fresh bytes, which cannot overlap
    // the caller's `len` readable ones; the last is the terminator.
    unsafe {
        let ret = xmalloc(len + 1);
        copy_bytes(ret, data, len);
        *ret.cast::<u8>().add(len) = 0;
        ret
    }
}

/// How many source bytes `xstrlcat` copies when appending `slen` bytes at
/// offset `dlen` of a `dsize` buffer, and where the terminator must be
/// written when truncating (the untruncated copy brings its own).
fn lcat_copy(dlen: usize, slen: usize, dsize: usize) -> (usize, Option<usize>) {
    if slen > dsize - dlen - 1 {
        (dsize - dlen - 1, Some(dsize - 1))
    } else {
        (slen + 1, None)
    }
}

/// BSD `strlcat`. The unit suite calls this with `src` pointing into `dst`,
/// so the copy stays a raw memmove.
///
/// # Safety
///
/// `dst` is a NUL-terminated string writable for `dsize` bytes, `src` is
/// NUL-terminated, and `dsize > 0`. The two *may* overlap.
pub unsafe fn xstrlcat(dst: *mut c_char, src: *const c_char, dsize: usize) -> usize {
    debug_assert!(dsize > 0, "dsize > 0");
    // SAFETY: the caller's two NUL-terminated strings. `lcat_copy` bounds
    // the write to `dsize` bytes from `dst`, and the copy may alias.
    let dlen = unsafe { cbytes(dst) }.len();
    debug_assert!(dlen < dsize, "dlen < dsize");
    let slen = unsafe { cbytes(src) }.len();
    let (copy_len, nul_at) = lcat_copy(dlen, slen, dsize);
    unsafe { ptr::copy(src, dst.add(dlen), copy_len) };
    if let Some(nul) = nul_at {
        unsafe { *dst.add(nul) = 0 };
    }
    slen + dlen
}

/// `strdup`, never failing.
///
/// # Safety
///
/// `str` is NUL-terminated; otherwise as [`try_malloc`].
pub unsafe fn xstrdup(str: *const c_char) -> *mut c_char {
    // SAFETY: the caller's NUL-terminated string.
    let len = unsafe { cbytes(str) }.len();
    unsafe { xmemdupz(str.cast::<c_void>(), len) }.cast::<c_char>()
}

/// Duplicate `len` bytes into a fresh allocation, without a terminator.
///
/// # Safety
///
/// `data` is readable for `len` bytes; otherwise as [`try_malloc`].
pub unsafe fn xmemdup(data: *const c_void, len: usize) -> *mut c_void {
    // SAFETY: `xmalloc` answers `len` fresh bytes, which cannot overlap the
    // caller's `len` readable ones.
    unsafe {
        let ret = xmalloc(len);
        copy_bytes(ret, data, len);
        ret
    }
}

pub mod alloc_log;
pub mod blocks;
pub(crate) mod claimed;
pub(crate) mod cstr_vec;
pub mod thin_cstring;
pub mod xstring;
/// The block pool, re-exported: it is the same allocation family.
pub use blocks::*;
pub use thin_cstring::ThinCString;
pub use xstring::XString;

pub(crate) static ARENA_ALLOC_COUNT: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
pub(crate) static DID_OUTOFMEM_MSG: GlobalCell<bool> = GlobalCell::new(false);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lcat_copies_source_and_terminator_when_it_fits() {
        // "b" (slen 1) onto "a" (dlen 1) in 3 bytes: copies "b\0".
        assert_eq!(lcat_copy(1, 1, 3), (2, None));
    }

    #[test]
    fn lcat_truncates_to_the_buffer_and_terminates_it() {
        // "defgi" onto "ABC" in 6 bytes: 2 payload bytes, NUL at index 5.
        assert_eq!(lcat_copy(3, 5, 6), (2, Some(5)));
        // No room at all: zero bytes, still re-terminates.
        assert_eq!(lcat_copy(3, 5, 4), (0, Some(3)));
    }
}
