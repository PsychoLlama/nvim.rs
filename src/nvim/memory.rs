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

use crate::src::nvim::global_cell::{GlobalCell, SharedCell};
use core::ffi::{c_char, c_int, c_long, c_void, CStr};
use core::ptr;
use core::slice;

extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn calloc(nmemb: usize, size: usize) -> *mut c_void;
    fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
    static mut arena_alloc_count: usize;
    static e_outofmem: [c_char; 0];
    fn gettext(msgid: *const c_char) -> *mut c_char;
    static mut emsg_silent: c_int;
    static mut did_outofmem_msg: bool;
    fn preserve_exit(errmsg: *const c_char) -> !;
    fn mf_release_all() -> bool;
    fn semsg(fmt: *const c_char, ...) -> bool;
    fn clear_sb_text(all: bool);
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct consumed_blk {
    pub prev: *mut consumed_blk,
}

pub type ArenaMem = *mut consumed_blk;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Arena {
    pub cur_blk: *mut c_char,
    pub pos: usize,
    pub size: usize,
}

pub type MemMalloc = Option<unsafe extern "C" fn(usize) -> *mut c_void>;
pub type MemFree = Option<unsafe extern "C" fn(*mut c_void)>;
pub type MemCalloc = Option<unsafe extern "C" fn(usize, usize) -> *mut c_void>;
pub type MemRealloc = Option<unsafe extern "C" fn(*mut c_void, usize) -> *mut c_void>;

#[no_mangle]
pub static mem_malloc: SharedCell<MemMalloc> = SharedCell::new(Some(malloc));
#[no_mangle]
pub static mem_free: SharedCell<MemFree> = SharedCell::new(Some(free));
#[no_mangle]
pub static mem_calloc: SharedCell<MemCalloc> = SharedCell::new(Some(calloc));
#[no_mangle]
pub static mem_realloc: SharedCell<MemRealloc> = SharedCell::new(Some(realloc));

unsafe fn try_to_free_memory() {
    static trying_to_free: SharedCell<bool> = SharedCell::new(false);
    if trying_to_free.get() {
        return;
    }
    trying_to_free.set(true);
    clear_sb_text(true);
    mf_release_all();
    arena_free_reuse_blks();
    trying_to_free.set(false);
}

unsafe fn do_outofmem_msg(size: usize) {
    if did_outofmem_msg {
        return;
    }
    // Message queueing would fail the allocation again; report loudly, once.
    emsg_silent = 0;
    did_outofmem_msg = true;
    semsg(
        gettext(b"E342: Out of memory!  (allocating %lu bytes)\0".as_ptr() as *const c_char),
        size as u64,
    );
}

#[no_mangle]
pub unsafe extern "C" fn try_malloc(size: usize) -> *mut c_void {
    let allocated_size = size.max(1);
    let mut ret = (*mem_malloc.ptr()).expect("non-null function pointer")(allocated_size);
    if ret.is_null() {
        try_to_free_memory();
        ret = (*mem_malloc.ptr()).expect("non-null function pointer")(allocated_size);
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn verbose_try_malloc(size: usize) -> *mut c_void {
    let ret = try_malloc(size);
    if ret.is_null() {
        do_outofmem_msg(size);
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn xmalloc(size: usize) -> *mut c_void {
    let ret = try_malloc(size);
    if ret.is_null() {
        preserve_exit(&raw const e_outofmem as *const c_char);
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn xfree(ptr: *mut c_void) {
    (*mem_free.ptr()).expect("non-null function pointer")(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn xcalloc(count: usize, size: usize) -> *mut c_void {
    let (allocated_count, allocated_size) = if count != 0 && size != 0 {
        (count, size)
    } else {
        (1, 1)
    };
    let mut ret =
        (*mem_calloc.ptr()).expect("non-null function pointer")(allocated_count, allocated_size);
    if ret.is_null() {
        try_to_free_memory();
        ret = (*mem_calloc.ptr()).expect("non-null function pointer")(
            allocated_count,
            allocated_size,
        );
        if ret.is_null() {
            preserve_exit(&raw const e_outofmem as *const c_char);
        }
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn xrealloc(ptr: *mut c_void, size: usize) -> *mut c_void {
    let allocated_size = size.max(1);
    let mut ret = (*mem_realloc.ptr()).expect("non-null function pointer")(ptr, allocated_size);
    if ret.is_null() {
        try_to_free_memory();
        ret = (*mem_realloc.ptr()).expect("non-null function pointer")(ptr, allocated_size);
        if ret.is_null() {
            preserve_exit(&raw const e_outofmem as *const c_char);
        }
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn xmallocz(size: usize) -> *mut c_void {
    let total_size = size.wrapping_add(1);
    if total_size < size {
        preserve_exit(gettext(
            b"Nvim: Data too large to fit into virtual memory space\n\0".as_ptr() as *const c_char,
        ));
    }
    let ret = xmalloc(total_size);
    *(ret as *mut u8).add(size) = 0;
    ret
}

#[no_mangle]
pub unsafe extern "C" fn xmemdupz(data: *const c_void, len: usize) -> *mut c_void {
    let ret = xmallocz(len);
    if len != 0 {
        slice::from_raw_parts_mut(ret as *mut u8, len)
            .copy_from_slice(slice::from_raw_parts(data as *const u8, len));
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn xmemcpyz(dst: *mut c_void, src: *const c_void, len: usize) -> *mut c_void {
    if len != 0 {
        slice::from_raw_parts_mut(dst as *mut u8, len)
            .copy_from_slice(slice::from_raw_parts(src as *const u8, len));
    }
    *(dst as *mut u8).add(len) = 0;
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
#[no_mangle]
pub unsafe extern "C" fn xstrchrnul(str: *const c_char, c: c_char) -> *mut c_char {
    let bytes = CStr::from_ptr(str).to_bytes();
    str.add(find_or_end(bytes, c as u8)) as *mut c_char
}

/// Like `memchr`, but absent characters yield `addr + size` instead of
/// NULL.
#[no_mangle]
pub unsafe extern "C" fn xmemscan(addr: *const c_void, c: c_char, size: usize) -> *mut c_void {
    let hay = slice::from_raw_parts(addr as *const u8, size);
    (addr as *mut u8).add(find_or_end(hay, c as u8)) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn strchrsub(str: *mut c_char, c: c_char, x: c_char) {
    assert!(c != 0, "c != NUL");
    let len = CStr::from_ptr(str).to_bytes().len();
    replace_bytes(
        slice::from_raw_parts_mut(str as *mut u8, len),
        c as u8,
        x as u8,
    );
}

#[no_mangle]
pub unsafe extern "C" fn memchrsub(data: *mut c_void, c: c_char, x: c_char, len: usize) {
    if len != 0 {
        replace_bytes(
            slice::from_raw_parts_mut(data as *mut u8, len),
            c as u8,
            x as u8,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn strcnt(str: *const c_char, c: c_char) -> usize {
    assert!(c != 0, "c != 0");
    count_byte(CStr::from_ptr(str).to_bytes(), c as u8)
}

#[no_mangle]
pub unsafe extern "C" fn memcnt(data: *const c_void, c: c_char, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    count_byte(slice::from_raw_parts(data as *const u8, len), c as u8)
}

/// `strnlen`: bytes before the terminator, reading at most `maxlen` bytes.
unsafe fn strnlen(s: *const c_char, maxlen: usize) -> usize {
    let mut n = 0;
    while n < maxlen && *s.add(n) != 0 {
        n += 1;
    }
    n
}

/// `strcpy` returning a pointer to the written terminator rather than
/// `dst`.
#[no_mangle]
pub unsafe extern "C" fn xstpcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char {
    let bytes = CStr::from_ptr(src).to_bytes_with_nul();
    slice::from_raw_parts_mut(dst as *mut u8, bytes.len()).copy_from_slice(bytes);
    dst.add(bytes.len() - 1)
}

/// `stpncpy`: copy at most `maxlen` bytes, zero-filling any remainder, and
/// return where the terminator went (or `dst + maxlen` if none fit).
#[no_mangle]
pub unsafe extern "C" fn xstpncpy(
    dst: *mut c_char,
    src: *const c_char,
    maxlen: usize,
) -> *mut c_char {
    let srclen = strnlen(src, maxlen);
    let out = slice::from_raw_parts_mut(dst as *mut u8, maxlen);
    out[..srclen].copy_from_slice(slice::from_raw_parts(src as *const u8, srclen));
    if srclen < maxlen {
        out[srclen..].fill(0);
        dst.add(srclen)
    } else {
        dst.add(maxlen)
    }
}

/// BSD `strlcpy`: bounded copy that always terminates (when `dsize > 0`)
/// and returns the untruncated source length.
#[no_mangle]
pub unsafe extern "C" fn xstrlcpy(dst: *mut c_char, src: *const c_char, dsize: usize) -> usize {
    let slen = CStr::from_ptr(src).to_bytes().len();
    if dsize != 0 {
        let len = slen.min(dsize - 1);
        let out = slice::from_raw_parts_mut(dst as *mut u8, len + 1);
        if len != 0 {
            out[..len].copy_from_slice(slice::from_raw_parts(src as *const u8, len));
        }
        out[len] = 0;
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
#[no_mangle]
pub unsafe extern "C" fn xstrlcat(dst: *mut c_char, src: *const c_char, dsize: usize) -> usize {
    assert!(dsize > 0, "dsize > 0");
    let dlen = CStr::from_ptr(dst).to_bytes().len();
    assert!(dlen < dsize, "dlen < dsize");
    let slen = CStr::from_ptr(src).to_bytes().len();
    let (copy_len, nul_at) = lcat_copy(dlen, slen, dsize);
    ptr::copy(src, dst.add(dlen), copy_len);
    if let Some(nul) = nul_at {
        *dst.add(nul) = 0;
    }
    slen + dlen
}

#[no_mangle]
pub unsafe extern "C" fn xstrdup(str: *const c_char) -> *mut c_char {
    xmemdupz(str as *const c_void, CStr::from_ptr(str).to_bytes().len()) as *mut c_char
}

/// `xstrdup` that maps NULL to an allocated empty string.
#[no_mangle]
pub unsafe extern "C" fn xstrdupnul(str: *const c_char) -> *mut c_char {
    if str.is_null() {
        return xmallocz(0) as *mut c_char;
    }
    xstrdup(str)
}

/// `memrchr`: last occurrence of `c` in the first `len` bytes, or NULL.
#[no_mangle]
pub unsafe extern "C" fn xmemrchr(src: *const c_void, c: u8, len: usize) -> *mut c_void {
    if len == 0 {
        return ptr::null_mut();
    }
    let hay = slice::from_raw_parts(src as *const u8, len);
    match hay.iter().rposition(|&b| b == c) {
        Some(i) => (src as *mut u8).add(i) as *mut c_void,
        None => ptr::null_mut(),
    }
}

/// `strndup`: duplicate at most `len` bytes (stopping at a terminator),
/// always NUL-terminating the copy.
#[no_mangle]
pub unsafe extern "C" fn xstrndup(str: *const c_char, len: usize) -> *mut c_char {
    xmemdupz(str as *const c_void, strnlen(str, len)) as *mut c_char
}

#[no_mangle]
pub unsafe extern "C" fn xmemdup(data: *const c_void, len: usize) -> *mut c_void {
    let ret = xmalloc(len);
    if len != 0 {
        slice::from_raw_parts_mut(ret as *mut u8, len)
            .copy_from_slice(slice::from_raw_parts(data as *const u8, len));
    }
    ret
}

/// `strncmp(a, b, n) == 0`, phrased over whole C strings: the length-`n`
/// prefixes match, where a terminator before `n` ends both sides.
fn eq_upto(a: &[u8], b: &[u8], n: usize) -> bool {
    a[..a.len().min(n)] == b[..b.len().min(n)]
}

/// `strcmp` equality where NULL only equals NULL.
#[no_mangle]
pub unsafe extern "C" fn strequal(a: *const c_char, b: *const c_char) -> bool {
    if a.is_null() || b.is_null() {
        return a.is_null() && b.is_null();
    }
    CStr::from_ptr(a).to_bytes() == CStr::from_ptr(b).to_bytes()
}

/// `strncmp` equality where NULL only equals NULL.
#[no_mangle]
pub unsafe extern "C" fn strnequal(a: *const c_char, b: *const c_char, n: usize) -> bool {
    if a.is_null() || b.is_null() {
        return a.is_null() && b.is_null();
    }
    eq_upto(
        CStr::from_ptr(a).to_bytes(),
        CStr::from_ptr(b).to_bytes(),
        n,
    )
}

/// Big-endian encoding of a timestamp, as shada writes it.
#[no_mangle]
pub unsafe extern "C" fn time_to_bytes(time_: c_long, buf: *mut u8) {
    slice::from_raw_parts_mut(buf, 8).copy_from_slice(&(time_ as u64).to_be_bytes());
}

pub type MergeSortGetFunc = Option<unsafe extern "C" fn(*mut c_void) -> *mut c_void>;
pub type MergeSortSetFunc = Option<unsafe extern "C" fn(*mut c_void, *mut c_void)>;
pub type MergeSortCompareFunc = Option<unsafe extern "C" fn(*const c_void, *const c_void) -> c_int>;

/// Bottom-up mergesort over an intrusive doubly-linked list, generic via
/// accessor callbacks. All list knowledge lives behind the callbacks, so
/// this stays a pointer-shuffling shim.
#[no_mangle]
pub unsafe extern "C" fn mergesort_list(
    mut head: *mut c_void,
    get_next: MergeSortGetFunc,
    set_next: MergeSortSetFunc,
    get_prev: MergeSortGetFunc,
    set_prev: MergeSortSetFunc,
    compare: MergeSortCompareFunc,
) -> *mut c_void {
    let get_next = get_next.expect("non-null function pointer");
    let set_next = set_next.expect("non-null function pointer");
    let get_prev = get_prev.expect("non-null function pointer");
    let set_prev = set_prev.expect("non-null function pointer");
    let compare = compare.expect("non-null function pointer");

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
                let chosen;
                if left.is_null() {
                    chosen = right;
                    right = get_next(right);
                } else if right.is_null() {
                    chosen = left;
                    left = get_next(left);
                } else if compare(left, right) <= 0 {
                    chosen = left;
                    left = get_next(left);
                } else {
                    chosen = right;
                    right = get_next(right);
                }
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

pub const ARENA_BLOCK_SIZE: usize = 4096;
const REUSE_MAX: usize = 4;

pub const ARENA_EMPTY: Arena = Arena {
    cur_blk: ptr::null_mut(),
    pos: 0,
    size: 0,
};

static arena_reuse_blk: GlobalCell<*mut consumed_blk> = GlobalCell::new(ptr::null_mut());
static arena_reuse_blk_count: GlobalCell<usize> = GlobalCell::new(0);

unsafe fn arena_free_reuse_blks() {
    while arena_reuse_blk_count.get() > 0 {
        let blk = arena_reuse_blk.get();
        arena_reuse_blk.set((*arena_reuse_blk.get()).prev);
        xfree(blk as *mut c_void);
        (*arena_reuse_blk_count.ptr()) -= 1;
    }
}

/// Detach the arena's chain of consumed blocks for a later
/// `arena_mem_free`, leaving the arena empty.
#[no_mangle]
pub unsafe extern "C" fn arena_finish(arena: *mut Arena) -> ArenaMem {
    let res = (*arena).cur_blk as *mut consumed_blk;
    *arena = ARENA_EMPTY;
    res
}

#[no_mangle]
pub unsafe extern "C" fn alloc_block() -> *mut c_void {
    if arena_reuse_blk_count.get() > 0 {
        let retval = arena_reuse_blk.get() as *mut c_void;
        arena_reuse_blk.set((*arena_reuse_blk.get()).prev);
        (*arena_reuse_blk_count.ptr()) -= 1;
        retval
    } else {
        arena_alloc_count = arena_alloc_count.wrapping_add(1);
        xmalloc(ARENA_BLOCK_SIZE)
    }
}

#[no_mangle]
pub unsafe extern "C" fn arena_alloc_block(arena: *mut Arena) {
    let prev_blk = (*arena).cur_blk as *mut consumed_blk;
    (*arena).cur_blk = alloc_block() as *mut c_char;
    (*arena).pos = 0;
    (*arena).size = ARENA_BLOCK_SIZE;
    // The block's first bytes link to the previous block.
    let blk = arena_alloc(arena, core::mem::size_of::<consumed_blk>(), true) as *mut consumed_blk;
    (*blk).prev = prev_blk;
}

pub const ARENA_ALIGN: usize = {
    let ptr_size = core::mem::size_of::<*mut c_void>();
    let double_size = core::mem::size_of::<f64>();
    if ptr_size > double_size {
        ptr_size
    } else {
        double_size
    }
};

/// Round `off` up to the arena's allocation alignment.
fn align_offset(off: usize) -> usize {
    (off.wrapping_add(ARENA_ALIGN - 1)) & !(ARENA_ALIGN - 1)
}

/// Allocations that would waste more than half a block get their own
/// exactly-sized block instead.
fn is_oversize(size: usize) -> bool {
    size > (ARENA_BLOCK_SIZE - core::mem::size_of::<consumed_blk>()) / 2
}

#[no_mangle]
pub unsafe extern "C" fn arena_alloc(arena: *mut Arena, size: usize, align: bool) -> *mut c_void {
    if arena.is_null() {
        return xmalloc(size);
    }
    if (*arena).cur_blk.is_null() {
        arena_alloc_block(arena);
    }
    let mut alloc_pos = if align {
        align_offset((*arena).pos)
    } else {
        (*arena).pos
    };
    if alloc_pos.wrapping_add(size) > (*arena).size {
        if is_oversize(size) {
            // Chain an exactly-sized block *behind* the current one, so the
            // current block's remaining space stays usable.
            arena_alloc_count = arena_alloc_count.wrapping_add(1);
            let hdr_size = core::mem::size_of::<consumed_blk>();
            let aligned_hdr_size = if align {
                align_offset(hdr_size)
            } else {
                hdr_size
            };
            let alloc = xmalloc(size.wrapping_add(aligned_hdr_size)) as *mut c_char;
            let cur_blk = (*arena).cur_blk as *mut consumed_blk;
            let fix_blk = alloc as *mut consumed_blk;
            (*fix_blk).prev = (*cur_blk).prev;
            (*cur_blk).prev = fix_blk;
            return alloc.add(aligned_hdr_size) as *mut c_void;
        }
        arena_alloc_block(arena);
        alloc_pos = if align {
            align_offset((*arena).pos)
        } else {
            (*arena).pos
        };
    }
    let mem = (*arena).cur_blk.add(alloc_pos);
    (*arena).pos = alloc_pos.wrapping_add(size);
    mem as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn free_block(block: *mut c_void) {
    if arena_reuse_blk_count.get() < REUSE_MAX {
        let reuse_blk = block as *mut consumed_blk;
        (*reuse_blk).prev = arena_reuse_blk.get();
        arena_reuse_blk.set(reuse_blk);
        (*arena_reuse_blk_count.ptr()) += 1;
    } else {
        xfree(block);
    }
}

#[no_mangle]
pub unsafe extern "C" fn arena_mem_free(mem: ArenaMem) {
    let mut b = mem;
    // The first block may be reused; the rest of the chain is freed.
    if !b.is_null() {
        let reuse_blk = b;
        b = (*b).prev;
        free_block(reuse_blk as *mut c_void);
    }
    while !b.is_null() {
        let prev = (*b).prev;
        xfree(b as *mut c_void);
        b = prev;
    }
}

#[no_mangle]
pub unsafe extern "C" fn arena_allocz(arena: *mut Arena, size: usize) -> *mut c_char {
    let mem = arena_alloc(arena, size.wrapping_add(1), false) as *mut c_char;
    *mem.add(size) = 0;
    mem
}

#[no_mangle]
pub unsafe extern "C" fn arena_memdupz(
    arena: *mut Arena,
    buf: *const c_char,
    size: usize,
) -> *mut c_char {
    let mem = arena_allocz(arena, size);
    if size != 0 {
        slice::from_raw_parts_mut(mem as *mut u8, size)
            .copy_from_slice(slice::from_raw_parts(buf as *const u8, size));
    }
    mem
}

#[no_mangle]
pub unsafe extern "C" fn arena_strdup(arena: *mut Arena, str: *const c_char) -> *mut c_char {
    arena_memdupz(arena, str, CStr::from_ptr(str).to_bytes().len())
}

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

    #[test]
    fn arena_alignment_rounds_up_to_the_platform_align() {
        assert_eq!(align_offset(0), 0);
        assert_eq!(align_offset(1), ARENA_ALIGN);
        assert_eq!(align_offset(ARENA_ALIGN), ARENA_ALIGN);
        assert_eq!(align_offset(ARENA_ALIGN + 1), 2 * ARENA_ALIGN);
    }

    #[test]
    fn oversize_threshold_is_half_a_block_minus_header() {
        let threshold = (ARENA_BLOCK_SIZE - core::mem::size_of::<consumed_blk>()) / 2;
        assert!(!is_oversize(threshold));
        assert!(is_oversize(threshold + 1));
    }
}
