//! The xmalloc allocation family, memory/string helpers, and the arena
//! allocator: safe cores + C-ABI shims.
//!
//! The `mem_malloc`/`mem_free`/`mem_calloc`/`mem_realloc` function pointers
//! are a load-bearing seam: the unit suite rebinds them at runtime to LuaJIT
//! callbacks so specs can assert on exact allocation sequences. Every heap
//! byte this module hands out therefore still flows through them — no Rust
//! container replaces an `xmalloc` here.
//!
//! Copy helpers whose C originals used `memmove` (`xstrlcat`, which the unit
//! suite calls with `src` pointing into `dst`) keep raw `ptr::copy`; slices
//! must never alias. Helpers that scan for a NUL only construct slices up to
//! the terminator, because the C originals never read past it and the
//! allocation may end there.
//!
//! # Boundary
//!
//! The ten exports (`mem_malloc`/`mem_free`/`mem_calloc`/`mem_realloc` and
//! `xmalloc`/`xcalloc`/`xfree`/`xstrdup`/`xmemdup`/`xmemdupz`) are pinned by
//! the ABI ledger and read by `test/unit`'s allocator seam. Their signatures
//! and their *allocation sequence* are both observable, so nothing here may
//! be reorganised into a Rust container, and no call may be elided.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::global_cell::SharedCell;
use crate::semsg_c;
use core::ffi::{CStr, c_char, c_int, c_long, c_ulong, c_void};
use core::{ptr, slice};

use crate::main::{did_outofmem_msg, e_outofmem, emsg_silent, preserve_exit};
use crate::memfile::mf_release_all;
use crate::message::clear_sb_text;
use crate::os::cshim::gettext;
use ::libc::{calloc, free, malloc, realloc};

pub type MemMalloc = Option<unsafe extern "C" fn(usize) -> *mut c_void>;
pub type MemFree = Option<unsafe extern "C" fn(*mut c_void)>;
pub type MemCalloc = Option<unsafe extern "C" fn(usize, usize) -> *mut c_void>;
pub type MemRealloc = Option<unsafe extern "C" fn(*mut c_void, usize) -> *mut c_void>;

#[unsafe(no_mangle)]
pub static mem_malloc: SharedCell<MemMalloc> = SharedCell::new(Some(malloc));
#[unsafe(no_mangle)]
pub static mem_free: SharedCell<MemFree> = SharedCell::new(Some(free));
#[unsafe(no_mangle)]
pub static mem_calloc: SharedCell<MemCalloc> = SharedCell::new(Some(calloc));
#[unsafe(no_mangle)]
pub static mem_realloc: SharedCell<MemRealloc> = SharedCell::new(Some(realloc));

/// Ask the editor to give memory back: drop the scrollback, release
/// memfile blocks, hand the arena's spare blocks to the allocator. Runs at
/// most once at a time, since everything it calls may allocate.
///
/// # Safety
///
/// Runs on the main thread with the editor in a state where the scrollback
/// and the memfiles may be dropped -- i.e. from inside an allocation.
unsafe fn try_to_free_memory() {
    static trying_to_free: SharedCell<bool> = SharedCell::new(false);
    if trying_to_free.get() {
        return;
    }
    trying_to_free.set(true);
    // SAFETY: the caller's promise about the editor's state.
    unsafe {
        clear_sb_text(true);
        mf_release_all();
        arena::free_reuse_blks();
    }
    trying_to_free.set(false);
}

/// Report an allocation failure, once per process.
///
/// # Safety
///
/// Runs on the main thread; `semsg` reaches the editor's message layer.
unsafe fn do_outofmem_msg(size: usize) {
    if did_outofmem_msg.get() {
        return;
    }
    // Message queueing would fail the allocation again; report loudly, once.
    emsg_silent.set(0);
    did_outofmem_msg.set(true);
    // A `size_t` that would not fit a `%lu` is not a size any allocator was
    // ever going to serve, so saturating beats panicking on the OOM path.
    let size = c_ulong::try_from(size).unwrap_or(c_ulong::MAX);
    let fmt = c"E342: Out of memory!  (allocating %lu bytes)";
    // SAFETY: `gettext` answers a NUL-terminated string, and `%lu` spends
    // exactly the `c_ulong` that follows it.
    unsafe { semsg_c!(gettext(fmt.as_ptr()), size) };
}

/// The `mem_*` seam's four function pointers, unwrapped. They are only ever
/// `None` if a spec rebound one to nil, which would be a broken spec.
macro_rules! mem_fn {
    ($cell:ident) => {
        $cell.get().expect("non-null function pointer")
    };
}

/// `malloc`, retried once after asking the editor for memory back. NULL when
/// even that did not help.
///
/// # Safety
///
/// Main thread, mid-allocation (see [`try_to_free_memory`]).
pub unsafe fn try_malloc(size: usize) -> *mut c_void {
    // A zero-byte malloc may answer NULL, which every caller here reads as
    // failure; upstream rounds up for the same reason.
    let allocated_size = size.max(1);
    // SAFETY: the seam's `malloc`, and the retry after freeing.
    unsafe {
        let ret = mem_fn!(mem_malloc)(allocated_size);
        if !ret.is_null() {
            return ret;
        }
        try_to_free_memory();
        mem_fn!(mem_malloc)(allocated_size)
    }
}

/// [`try_malloc`], reporting the failure to the user.
///
/// # Safety
///
/// As [`try_malloc`].
pub unsafe fn verbose_try_malloc(size: usize) -> *mut c_void {
    // SAFETY: the caller's promise.
    unsafe {
        let ret = try_malloc(size);
        if ret.is_null() {
            do_outofmem_msg(size);
        }
        ret
    }
}

/// `malloc` that never fails: an allocation that cannot be served ends the
/// process through `preserve_exit`, which writes the swapfiles out first.
///
/// # Safety
///
/// As [`try_malloc`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmalloc(size: usize) -> *mut c_void {
    // SAFETY: the caller's promise; `e_outofmem` is a NUL-terminated static.
    unsafe {
        let ret = try_malloc(size);
        if ret.is_null() {
            preserve_exit((&raw const e_outofmem).cast::<c_char>());
        }
        ret
    }
}

/// `free`, through the seam. A null pointer is a no-op, as `free`'s is.
///
/// # Safety
///
/// `ptr` is null or an allocation from this module's family, not yet freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xfree(ptr: *mut c_void) {
    // SAFETY: the caller's allocation, handed back to the seam that made it.
    unsafe { mem_fn!(mem_free)(ptr) };
}

/// `calloc` that never fails. The zeroing is load-bearing: callers all over
/// the tree treat a fresh allocation as initialized.
///
/// # Safety
///
/// As [`try_malloc`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xcalloc(count: usize, size: usize) -> *mut c_void {
    // As in `try_malloc`: a zero-sized request must still answer a pointer.
    let (allocated_count, allocated_size) = if count != 0 && size != 0 {
        (count, size)
    } else {
        (1, 1)
    };
    // SAFETY: the seam's `calloc`, the retry after freeing, and a
    // NUL-terminated static for the exit message.
    unsafe {
        let ret = mem_fn!(mem_calloc)(allocated_count, allocated_size);
        if !ret.is_null() {
            return ret;
        }
        try_to_free_memory();
        let ret = mem_fn!(mem_calloc)(allocated_count, allocated_size);
        if ret.is_null() {
            preserve_exit((&raw const e_outofmem).cast::<c_char>());
        }
        ret
    }
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
    unsafe {
        let ret = mem_fn!(mem_realloc)(ptr, allocated_size);
        if !ret.is_null() {
            return ret;
        }
        try_to_free_memory();
        let ret = mem_fn!(mem_realloc)(ptr, allocated_size);
        if ret.is_null() {
            preserve_exit((&raw const e_outofmem).cast::<c_char>());
        }
        ret
    }
}

/// [`xmalloc`] of `size + 1` bytes with the extra one set to NUL: room for
/// `size` bytes of payload plus a terminator.
///
/// # Safety
///
/// As [`try_malloc`].
pub unsafe fn xmallocz(size: usize) -> *mut c_void {
    let total_size = size.wrapping_add(1);
    // SAFETY: a NUL-terminated string from `gettext`, then `size + 1` bytes
    // of fresh allocation whose last byte is the one being written.
    unsafe {
        if total_size < size {
            let too_big = c"Nvim: Data too large to fit into virtual memory space\n";
            preserve_exit(gettext(too_big.as_ptr()));
        }
        let ret = xmalloc(total_size);
        *ret.cast::<u8>().add(size) = 0;
        ret
    }
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
    // SAFETY: the caller's promise. `copy_nonoverlapping` accepts a zero
    // length with any pointers, so no guard is needed.
    unsafe { ptr::copy_nonoverlapping(src.cast::<u8>(), dst.cast::<u8>(), len) };
}

/// Duplicate `len` bytes into a fresh NUL-terminated allocation.
///
/// # Safety
///
/// `data` is readable for `len` bytes; otherwise as [`try_malloc`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmemdupz(data: *const c_void, len: usize) -> *mut c_void {
    // SAFETY: `xmallocz` answers `len + 1` fresh bytes, which cannot
    // overlap the caller's `len` readable ones.
    unsafe {
        let ret = xmallocz(len);
        copy_bytes(ret, data, len);
        ret
    }
}

/// `memcpy` that terminates the copy: `len` bytes plus a NUL.
///
/// # Safety
///
/// `src` is readable for `len` bytes, `dst` writable for `len + 1`, and they
/// do not overlap.
pub unsafe fn xmemcpyz(dst: *mut c_void, src: *const c_void, len: usize) -> *mut c_void {
    // SAFETY: the caller's promise, terminator byte included.
    unsafe {
        copy_bytes(dst, src, len);
        *dst.cast::<u8>().add(len) = 0;
    }
    dst
}

/// Position of `c` in `haystack`, or `haystack.len()` when absent.
fn find_or_end(haystack: &[u8], c: u8) -> usize {
    haystack
        .iter()
        .position(|&b| b == c)
        .unwrap_or(haystack.len())
}

/// Substitute every `from` byte with `to`.
fn replace_bytes(s: &mut [u8], from: u8, to: u8) {
    for b in s {
        if *b == from {
            *b = to;
        }
    }
}

/// Number of `c` bytes in `s`.
fn count_byte(s: &[u8], c: u8) -> usize {
    s.iter().filter(|&&b| b == c).count()
}

/// Like `strchr`, but absent characters yield the terminator instead of
/// NULL.
///
/// # Safety
///
/// `str` is NUL-terminated.
pub unsafe fn xstrchrnul(str: *const c_char, c: c_char) -> *mut c_char {
    // SAFETY: the caller's NUL-terminated string; the answer is at most the
    // terminator's own address.
    let off = find_or_end(unsafe { cbytes(str) }, c.cast_unsigned());
    unsafe { str.add(off) }.cast_mut()
}

/// Like `memchr`, but absent characters yield `addr + size` instead of
/// NULL.
///
/// # Safety
///
/// `addr` is readable for `size` bytes.
pub unsafe fn xmemscan(addr: *const c_void, c: c_char, size: usize) -> *mut c_void {
    // SAFETY: the caller's `size` readable bytes; the answer is at most one
    // past the end, which is a legal address to form.
    let hay = unsafe { slice::from_raw_parts(addr.cast::<u8>(), size) };
    unsafe { addr.cast::<u8>().add(find_or_end(hay, c.cast_unsigned())) }
        .cast_mut()
        .cast::<c_void>()
}

/// Replace every `c` byte of a C string with `x`.
///
/// # Safety
///
/// `str` is a NUL-terminated, writable string.
pub unsafe fn strchrsub(str: *mut c_char, c: c_char, x: c_char) {
    debug_assert!(c != 0, "c != NUL");
    // SAFETY: the caller's writable string, up to but not past its NUL.
    let len = unsafe { cbytes(str) }.len();
    let s = unsafe { slice::from_raw_parts_mut(str.cast::<u8>(), len) };
    replace_bytes(s, c.cast_unsigned(), x.cast_unsigned());
}

/// Replace every `c` byte of a `len`-byte buffer with `x`.
///
/// # Safety
///
/// `data` is writable for `len` bytes.
pub unsafe fn memchrsub(data: *mut c_void, c: c_char, x: c_char, len: usize) {
    if len != 0 {
        // SAFETY: the caller's `len` writable bytes.
        let s = unsafe { slice::from_raw_parts_mut(data.cast::<u8>(), len) };
        replace_bytes(s, c.cast_unsigned(), x.cast_unsigned());
    }
}

/// How many `c` bytes the first `len` bytes of `data` hold.
///
/// # Safety
///
/// `data` is readable for `len` bytes.
pub unsafe fn memcnt(data: *const c_void, c: c_char, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    // SAFETY: the caller's `len` readable bytes.
    let hay = unsafe { slice::from_raw_parts(data.cast::<u8>(), len) };
    count_byte(hay, c.cast_unsigned())
}

/// `strnlen`: bytes before the terminator, reading at most `maxlen` bytes.
///
/// # Safety
///
/// `s` is readable up to its terminator or `maxlen` bytes, whichever comes
/// first.
unsafe fn strnlen(s: *const c_char, maxlen: usize) -> usize {
    let mut n = 0;
    // SAFETY: the caller's promise; the read stops at the first NUL, so it
    // never runs past a shorter string than `maxlen`.
    while n < maxlen && unsafe { *s.add(n) } != 0 {
        n += 1;
    }
    n
}

/// `strcpy` returning a pointer to the written terminator rather than
/// `dst`.
///
/// # Safety
///
/// `src` is NUL-terminated and `dst` is writable for its whole length plus
/// the terminator; the two do not overlap.
pub unsafe fn xstpcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char {
    // SAFETY: the caller's promise, terminator included.
    let len = unsafe { cbytes(src) }.len() + 1;
    unsafe { copy_bytes(dst.cast::<c_void>(), src.cast::<c_void>(), len) };
    unsafe { dst.add(len - 1) }
}

/// BSD `strlcpy`: bounded copy that always terminates (when `dsize > 0`)
/// and returns the untruncated source length.
///
/// # Safety
///
/// `src` is NUL-terminated, `dst` is writable for `dsize` bytes, and the two
/// do not overlap.
pub unsafe fn xstrlcpy(dst: *mut c_char, src: *const c_char, dsize: usize) -> usize {
    // SAFETY: the caller's promise. At most `dsize - 1` payload bytes are
    // written, plus the terminator at `dsize - 1` or earlier.
    let slen = unsafe { cbytes(src) }.len();
    if dsize != 0 {
        let len = slen.min(dsize - 1);
        unsafe { copy_bytes(dst.cast::<c_void>(), src.cast::<c_void>(), len) };
        unsafe { *dst.add(len) = 0 };
    }
    slen
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xstrdup(str: *const c_char) -> *mut c_char {
    // SAFETY: the caller's NUL-terminated string.
    let len = unsafe { cbytes(str) }.len();
    unsafe { xmemdupz(str.cast::<c_void>(), len) }.cast::<c_char>()
}

/// `memrchr`: last occurrence of `c` in the first `len` bytes, or NULL.
///
/// # Safety
///
/// `src` is readable for `len` bytes.
pub unsafe fn xmemrchr(src: *const c_void, c: u8, len: usize) -> *mut c_void {
    if len == 0 {
        return ptr::null_mut();
    }
    // SAFETY: the caller's `len` readable bytes; the answer is an index
    // into them.
    let hay = unsafe { slice::from_raw_parts(src.cast::<u8>(), len) };
    match hay.iter().rposition(|&b| b == c) {
        Some(i) => unsafe { src.cast::<u8>().add(i) }
            .cast_mut()
            .cast::<c_void>(),
        None => ptr::null_mut(),
    }
}

/// `strndup`: duplicate at most `len` bytes (stopping at a terminator),
/// always NUL-terminating the copy.
///
/// # Safety
///
/// `str` is readable up to its terminator or `len` bytes, whichever comes
/// first; otherwise as [`try_malloc`].
pub unsafe fn xstrndup(str: *const c_char, len: usize) -> *mut c_char {
    // SAFETY: the caller's promise; `strnlen` never reports more than it
    // was allowed to read.
    unsafe { xmemdupz(str.cast::<c_void>(), strnlen(str, len)) }.cast::<c_char>()
}

/// Duplicate `len` bytes into a fresh allocation, without a terminator.
///
/// # Safety
///
/// `data` is readable for `len` bytes; otherwise as [`try_malloc`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmemdup(data: *const c_void, len: usize) -> *mut c_void {
    // SAFETY: `xmalloc` answers `len` fresh bytes, which cannot overlap the
    // caller's `len` readable ones.
    unsafe {
        let ret = xmalloc(len);
        copy_bytes(ret, data, len);
        ret
    }
}

/// `strncmp(a, b, n) == 0`, phrased over whole C strings: the length-`n`
/// prefixes match, where a terminator before `n` ends both sides.
fn eq_upto(a: &[u8], b: &[u8], n: usize) -> bool {
    a[..a.len().min(n)] == b[..b.len().min(n)]
}

/// `strcmp` equality where NULL only equals NULL.
///
/// # Safety
///
/// `a` and `b` are null or NUL-terminated.
pub unsafe fn strequal(a: *const c_char, b: *const c_char) -> bool {
    if a.is_null() || b.is_null() {
        return a.is_null() && b.is_null();
    }
    // SAFETY: neither is null, so both are the caller's C strings.
    unsafe { cbytes(a) == cbytes(b) }
}

/// `strncmp` equality where NULL only equals NULL.
///
/// # Safety
///
/// `a` and `b` are null or NUL-terminated.
pub unsafe fn strnequal(a: *const c_char, b: *const c_char, n: usize) -> bool {
    if a.is_null() || b.is_null() {
        return a.is_null() && b.is_null();
    }
    // SAFETY: neither is null, so both are the caller's C strings.
    unsafe { eq_upto(cbytes(a), cbytes(b), n) }
}

/// Big-endian encoding of a timestamp, as shada writes it.
///
/// # Safety
///
/// `buf` is writable for 8 bytes.
pub unsafe fn time_to_bytes(time_: c_long, buf: *mut u8) {
    // SAFETY: the caller's 8 writable bytes.
    unsafe { *buf.cast::<[u8; 8]>() = time_.cast_unsigned().to_be_bytes() };
}

pub type MergeSortGetFunc = Option<unsafe fn(*mut c_void) -> *mut c_void>;
pub type MergeSortSetFunc = Option<unsafe fn(*mut c_void, *mut c_void)>;
pub type MergeSortCompareFunc = Option<unsafe fn(*const c_void, *const c_void) -> c_int>;

/// Bottom-up mergesort over an intrusive doubly-linked list, generic via
/// accessor callbacks. All list knowledge lives behind the callbacks, so
/// this stays a pointer-shuffling shim.
///
/// # Safety
///
/// All five accessors are `Some`, `head` is null or the first element of a
/// list they understand, and they are the *only* thing that reads or writes
/// an element while the sort runs.
pub unsafe fn mergesort_list(
    mut head: *mut c_void,
    get_next: MergeSortGetFunc,
    set_next: MergeSortSetFunc,
    get_prev: MergeSortGetFunc,
    set_prev: MergeSortSetFunc,
    compare: MergeSortCompareFunc,
) -> *mut c_void {
    let next_of = get_next.expect("non-null function pointer");
    let link_next = set_next.expect("non-null function pointer");
    let prev_of = get_prev.expect("non-null function pointer");
    let link_prev = set_prev.expect("non-null function pointer");
    let cmp = compare.expect("non-null function pointer");

    // The obligation is discharged once, here: every element the body below
    // hands an accessor came out of another accessor, i.e. out of the
    // caller's own list. Wrapping each in a closure makes the sort itself --
    // which is where a bug would live -- ordinary safe Rust.
    // SAFETY: the caller's accessors over the caller's list.
    let get_next = |p: *mut c_void| unsafe { next_of(p) };
    let set_next = |p: *mut c_void, q: *mut c_void| unsafe { link_next(p, q) };
    let get_prev = |p: *mut c_void| unsafe { prev_of(p) };
    let set_prev = |p: *mut c_void, q: *mut c_void| unsafe { link_prev(p, q) };
    let compare = |a: *const c_void, b: *const c_void| unsafe { cmp(a, b) };

    if head.is_null() || get_next(head).is_null() {
        return head;
    }
    let mut n = 0;
    let mut curr = head;
    while !curr.is_null() {
        n += 1;
        curr = get_next(curr);
    }

    let mut size = 1;
    while size < n {
        let mut new_head: *mut c_void = ptr::null_mut();
        let mut tail: *mut c_void = ptr::null_mut();
        curr = head;
        while !curr.is_null() {
            // Split off two size-length runs starting at curr.
            let mut left = curr;
            let mut right = left;
            let mut i = 0;
            while i < size && !right.is_null() {
                right = get_next(right);
                i += 1;
            }
            let mut next = right;
            let mut i = 0;
            while i < size && !next.is_null() {
                next = get_next(next);
                i += 1;
            }
            let l_end = if !right.is_null() {
                get_prev(right)
            } else {
                ptr::null_mut()
            };
            if !l_end.is_null() {
                set_next(l_end, ptr::null_mut());
            }
            if !right.is_null() {
                set_prev(right, ptr::null_mut());
            }
            let r_end = if !next.is_null() {
                get_prev(next)
            } else {
                ptr::null_mut()
            };
            if !r_end.is_null() {
                set_next(r_end, ptr::null_mut());
            }
            if !next.is_null() {
                set_prev(next, ptr::null_mut());
            }

            // Merge the two runs.
            let mut merged: *mut c_void = ptr::null_mut();
            let mut merged_tail: *mut c_void = ptr::null_mut();
            while !left.is_null() || !right.is_null() {
                // The left run wins a tie, which is what makes the sort
                // stable. `compare` only sees two live elements.
                let chosen = if !left.is_null() && (right.is_null() || compare(left, right) <= 0) {
                    let taken = left;
                    left = get_next(left);
                    taken
                } else {
                    let taken = right;
                    right = get_next(right);
                    taken
                };
                if !merged_tail.is_null() {
                    set_next(merged_tail, chosen);
                    set_prev(chosen, merged_tail);
                    merged_tail = chosen;
                } else {
                    merged_tail = chosen;
                    merged = merged_tail;
                    set_prev(chosen, ptr::null_mut());
                }
            }

            // Append the merged run to the output list.
            if new_head.is_null() {
                new_head = merged;
            } else {
                set_next(tail, merged);
                set_prev(merged, tail);
            }
            while !get_next(merged_tail).is_null() {
                merged_tail = get_next(merged_tail);
            }
            tail = merged_tail;
            curr = next;
        }
        head = new_head;
        size *= 2;
    }
    head
}

/// The arena allocator, re-exported: every caller in the tree spells it
/// `crate::memory::arena_*`, and it is the same allocation family.
pub mod arena;
pub use arena::*;

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

    #[test]
    fn eq_upto_matches_strncmp_semantics() {
        assert!(eq_upto(b"abc", b"abc", 10)); // shared terminator before n
        assert!(eq_upto(b"abcX", b"abcY", 3)); // differences past n invisible
        assert!(!eq_upto(b"abc", b"abcd", 4)); // terminator vs 'd'
        assert!(eq_upto(b"", b"", 5));
        assert!(eq_upto(b"xyz", b"abc", 0)); // n = 0 compares nothing
    }

    #[test]
    fn find_replace_count_over_slices() {
        assert_eq!(find_or_end(b"hello", b'l'), 2);
        assert_eq!(find_or_end(b"hello", b'z'), 5);
        let mut buf = *b"a.b.c";
        replace_bytes(&mut buf, b'.', b'-');
        assert_eq!(&buf, b"a-b-c");
        assert_eq!(count_byte(b"a-b-c", b'-'), 2);
        assert_eq!(count_byte(b"", b'-'), 0);
    }
}
