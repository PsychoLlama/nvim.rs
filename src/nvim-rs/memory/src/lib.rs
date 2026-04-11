//! Memory allocation bridge for Neovim C-to-Rust migration
//!
//! This module provides safe Rust wrappers around Neovim's memory allocation
//! functions (`xmalloc`, `xfree`, etc.). This enables Rust code to allocate
//! memory that can be safely passed to and from C code.
//!
//! # Why this is needed
//!
//! Neovim uses its own memory allocators that:
//! - Never return NULL (they exit on OOM)
//! - Integrate with Neovim's memory management (e.g., `try_to_free_memory`)
//! - Are used throughout the C codebase
//!
//! During the migration, Rust code must use these allocators when:
//! - Returning allocated memory to C code
//! - Receiving memory from C code that Rust must later free
//! - Allocating memory that will be stored in C data structures
//!
//! # Safety
//!
//! The wrapper types in this module (`NvimBox`, `NvimString`) ensure that
//! memory allocated with nvim's allocator is freed with nvim's allocator.
//!
//! # Panics
//!
//! Functions in this module will panic if nvim's allocator returns NULL,
//! which should never happen in practice (nvim exits on OOM).

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::missing_const_for_fn)] // Many functions can't be const due to FFI
#![allow(clippy::must_use_candidate)] // FFI-focused crate, not all return values need must_use
#![allow(clippy::missing_panics_doc)] // Panics only on NULL from allocator (shouldn't happen)
#![allow(clippy::cast_ptr_alignment)] // Arena blocks are xmalloc-aligned; casts to ConsumedBlk are safe

use std::alloc::Layout;
use std::ffi::{c_char, c_void, CStr};
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

// =============================================================================
// FFI Bindings to nvim's memory allocator
// =============================================================================

extern "C" {
    /// Allocate `size` bytes. Never returns NULL (exits on OOM).
    pub fn xmalloc(size: usize) -> *mut c_void;

    /// Allocate `count * size` bytes, zeroed. Never returns NULL.
    pub fn xcalloc(count: usize, size: usize) -> *mut c_void;

    /// Reallocate memory. Never returns NULL.
    pub fn xrealloc(ptr: *mut c_void, size: usize) -> *mut c_void;

    /// Allocate `size + 1` bytes, with the last byte zeroed (for strings).
    pub fn xmallocz(size: usize) -> *mut c_void;

    /// Free memory allocated by xmalloc/xcalloc/xrealloc/xmallocz.
    pub fn xfree(ptr: *mut c_void);

    /// Duplicate a string using xmallocz.
    pub fn xstrdup(str: *const c_char) -> *mut c_char;

    /// Duplicate at most `len` bytes of a string.
    pub fn xstrndup(str: *const c_char, len: usize) -> *mut c_char;

    /// Duplicate memory using xmalloc.
    pub fn xmemdup(data: *const c_void, len: usize) -> *mut c_void;

    /// Duplicate memory with zero termination.
    pub fn xmemdupz(data: *const c_void, len: usize) -> *mut c_void;

    /// Align an offset to the arena alignment boundary (implemented in memutil crate).
    fn arena_align_offset(off: u64) -> usize;

    /// Increment the global `arena_alloc_count` by 1.
    fn nvim_inc_arena_alloc_count();
}

// =============================================================================
// Arena Allocator
// =============================================================================

const ARENA_BLOCK_SIZE: usize = 4096;
const REUSE_MAX: usize = 4;

/// Mirror of C's `struct consumed_blk` -- a singly linked list of used blocks.
#[repr(C)]
pub(crate) struct ConsumedBlk {
    pub(crate) prev: *mut ConsumedBlk,
}

/// Mirror of C's `Arena` struct.
#[repr(C)]
pub struct Arena {
    cur_blk: *mut c_char,
    pos: usize,
    size: usize,
}

/// Opaque handle returned by `arena_finish`, used to free arena memory later.
///
/// In C this is `typedef struct consumed_blk *ArenaMem;`.
/// Exposed as a raw void pointer to avoid leaking the private `ConsumedBlk` type.
pub type ArenaMem = *mut c_void;

/// Reuse pool: at most `REUSE_MAX` blocks are kept for reuse.
static mut ARENA_REUSE_BLK: *mut ConsumedBlk = std::ptr::null_mut();
static mut ARENA_REUSE_BLK_COUNT: usize = 0;

/// Free all cached reuse blocks.
///
/// # Safety
/// Mutates process-global state. Must not be called concurrently.
#[export_name = "arena_free_reuse_blks"]
pub unsafe extern "C" fn rs_arena_free_reuse_blks() {
    while ARENA_REUSE_BLK_COUNT > 0 {
        let blk = ARENA_REUSE_BLK;
        ARENA_REUSE_BLK = (*blk).prev;
        xfree(blk.cast());
        ARENA_REUSE_BLK_COUNT -= 1;
    }
}

/// Finish the allocations in an arena.
///
/// Returns an opaque `ArenaMem` handle that can later be passed to
/// `arena_mem_free` to release the memory.
///
/// # Safety
/// `arena` must be a valid pointer to an initialized `Arena`.
#[export_name = "arena_finish"]
pub unsafe extern "C" fn rs_arena_finish(arena: *mut Arena) -> ArenaMem {
    let res = (*arena).cur_blk.cast::<c_void>();
    // Reset to ARENA_EMPTY
    (*arena).cur_blk = std::ptr::null_mut();
    (*arena).pos = 0;
    (*arena).size = 0;
    res
}

/// Allocate a block of `ARENA_BLOCK_SIZE` bytes, reusing a cached block if
/// available. Must be freed with `free_block`.
///
/// # Safety
/// Mutates process-global reuse pool state. Must not be called concurrently.
#[export_name = "alloc_block"]
pub unsafe extern "C" fn rs_alloc_block() -> *mut c_void {
    if ARENA_REUSE_BLK_COUNT > 0 {
        let retval = ARENA_REUSE_BLK.cast::<c_void>();
        ARENA_REUSE_BLK = (*ARENA_REUSE_BLK).prev;
        ARENA_REUSE_BLK_COUNT -= 1;
        retval
    } else {
        nvim_inc_arena_alloc_count();
        xmalloc(ARENA_BLOCK_SIZE)
    }
}

/// Allocate a new block and link it into the arena.
///
/// # Safety
/// `arena` must be a valid pointer to an initialized `Arena`.
#[export_name = "arena_alloc_block"]
pub unsafe extern "C" fn rs_arena_alloc_block(arena: *mut Arena) {
    let prev_blk = (*arena).cur_blk.cast::<ConsumedBlk>();
    (*arena).cur_blk = rs_alloc_block().cast();
    (*arena).pos = 0;
    (*arena).size = ARENA_BLOCK_SIZE;
    let blk = rs_arena_alloc(arena, std::mem::size_of::<ConsumedBlk>(), true).cast::<ConsumedBlk>();
    (*blk).prev = prev_blk;
}

/// Main arena allocation function.
///
/// If `arena` is NULL, falls back to a global `xmalloc` call (caller must free).
/// If `size` is zero, returns a non-null but non-unique pointer.
///
/// # Safety
/// `arena` must be NULL or a valid pointer to an initialized `Arena`.
#[export_name = "arena_alloc"]
pub unsafe extern "C" fn rs_arena_alloc(
    arena: *mut Arena,
    size: usize,
    align: bool,
) -> *mut c_void {
    if arena.is_null() {
        return xmalloc(size);
    }
    if (*arena).cur_blk.is_null() {
        rs_arena_alloc_block(arena);
    }
    let alloc_pos = if align {
        arena_align_offset((*arena).pos as u64)
    } else {
        (*arena).pos
    };
    if alloc_pos + size > (*arena).size {
        // Threshold: more than half of (ARENA_BLOCK_SIZE - header) -> oversized block
        let threshold = (ARENA_BLOCK_SIZE - std::mem::size_of::<ConsumedBlk>()) >> 1;
        if size > threshold {
            nvim_inc_arena_alloc_count();
            let hdr_size = std::mem::size_of::<ConsumedBlk>();
            let aligned_hdr_size = if align {
                arena_align_offset(hdr_size as u64)
            } else {
                hdr_size
            };
            let alloc: *mut c_char = xmalloc(size + aligned_hdr_size).cast();

            // Insert oversized block behind current block in the linked list.
            // cur_blk always stays as the normal ARENA_BLOCK_SIZE block.
            let cur_blk = (*arena).cur_blk.cast::<ConsumedBlk>();
            let fix_blk = alloc.cast::<ConsumedBlk>();
            (*fix_blk).prev = (*cur_blk).prev;
            (*cur_blk).prev = fix_blk;

            return alloc.add(aligned_hdr_size).cast();
        }
        // Not oversized: allocate a new normal block
        rs_arena_alloc_block(arena);
        let new_alloc_pos = if align {
            arena_align_offset((*arena).pos as u64)
        } else {
            (*arena).pos
        };
        let mem: *mut c_char = (*arena).cur_blk.add(new_alloc_pos);
        (*arena).pos = new_alloc_pos + size;
        return mem.cast();
    }

    let mem: *mut c_char = (*arena).cur_blk.add(alloc_pos);
    (*arena).pos = alloc_pos + size;
    mem.cast()
}

/// Free or cache a block for reuse.
///
/// # Safety
/// `block` must have been allocated by `alloc_block` (i.e., `ARENA_BLOCK_SIZE` bytes).
#[export_name = "free_block"]
pub unsafe extern "C" fn rs_free_block(block: *mut c_void) {
    if ARENA_REUSE_BLK_COUNT < REUSE_MAX {
        let reuse_blk = block.cast::<ConsumedBlk>();
        (*reuse_blk).prev = ARENA_REUSE_BLK;
        ARENA_REUSE_BLK = reuse_blk;
        ARENA_REUSE_BLK_COUNT += 1;
    } else {
        xfree(block);
    }
}

/// Free all blocks in an arena chain previously obtained from `arena_finish`.
///
/// # Safety
/// `mem` must be a valid `ArenaMem` handle (or NULL).
#[export_name = "arena_mem_free"]
pub unsafe extern "C" fn rs_arena_mem_free(mem: ArenaMem) {
    let mut b = mem.cast::<ConsumedBlk>();
    // The first block is always a normal ARENA_BLOCK_SIZE block; put it in reuse pool.
    if !b.is_null() {
        let reuse_blk = b;
        b = (*b).prev;
        rs_free_block(reuse_blk.cast());
    }
    // Remaining blocks may be oversized; always free them directly.
    while !b.is_null() {
        let prev = (*b).prev;
        xfree(b.cast());
        b = prev;
    }
}

/// Allocate `size` bytes from `arena`, zero-terminating the result.
///
/// # Safety
/// `arena` must be NULL or a valid pointer to an initialized `Arena`.
#[export_name = "arena_allocz"]
pub unsafe extern "C" fn rs_arena_allocz(arena: *mut Arena, size: usize) -> *mut c_char {
    let mem = rs_arena_alloc(arena, size + 1, false).cast::<c_char>();
    *mem.add(size) = 0;
    mem
}

/// Duplicate `size` bytes from `buf` into `arena`, zero-terminating the result.
///
/// # Safety
/// `arena` must be NULL or valid; `buf` must be valid for `size` bytes.
#[export_name = "arena_memdupz"]
pub unsafe extern "C" fn rs_arena_memdupz(
    arena: *mut Arena,
    buf: *const c_char,
    size: usize,
) -> *mut c_char {
    let mem = rs_arena_allocz(arena, size);
    std::ptr::copy_nonoverlapping(buf, mem, size);
    mem
}

/// Duplicate a NUL-terminated string into `arena`.
///
/// # Safety
/// `arena` must be NULL or valid; `str` must be a valid NUL-terminated C string.
#[export_name = "arena_strdup"]
pub unsafe extern "C" fn rs_arena_strdup(arena: *mut Arena, str: *const c_char) -> *mut c_char {
    let len = libc_strlen(str);
    rs_arena_memdupz(arena, str, len)
}

/// Minimal `strlen` for use in `arena_strdup` without a libc dependency.
unsafe fn libc_strlen(s: *const c_char) -> usize {
    let mut len = 0usize;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

// =============================================================================
// Mergesort for doubly linked lists
// =============================================================================

type MergeSortGetFunc = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
type MergeSortSetFunc = unsafe extern "C" fn(*mut c_void, *mut c_void);
type MergeSortCompareFunc = unsafe extern "C" fn(*const c_void, *const c_void) -> std::ffi::c_int;

/// Iterative merge sort for doubly linked list. O(NlogN) worst case, stable.
///
/// The list is divided into blocks of increasing size (1, 2, 4, 8, ...).
/// Each pair of blocks is merged in sorted order.
/// Merged blocks are reconnected to build the sorted list.
///
/// # Safety
/// All function pointer arguments must be non-null and valid C function pointers.
/// `head` must be NULL or a valid node pointer.
#[export_name = "mergesort_list"]
pub unsafe extern "C" fn rs_mergesort_list(
    mut head: *mut c_void,
    get_next: MergeSortGetFunc,
    set_next: MergeSortSetFunc,
    get_prev: MergeSortGetFunc,
    set_prev: MergeSortSetFunc,
    compare: MergeSortCompareFunc,
) -> *mut c_void {
    if head.is_null() || get_next(head).is_null() {
        return head;
    }

    // Count length
    let mut n = 0i32;
    let mut curr = head;
    while !curr.is_null() {
        n += 1;
        curr = get_next(curr);
    }

    let mut size = 1i32;
    while size < n {
        let mut new_head: *mut c_void = std::ptr::null_mut();
        let mut tail: *mut c_void = std::ptr::null_mut();
        curr = head;

        while !curr.is_null() {
            // Split two runs
            let left = curr;
            let mut right = left;
            for _ in 0..size {
                if right.is_null() {
                    break;
                }
                right = get_next(right);
            }

            let mut next = right;
            for _ in 0..size {
                if next.is_null() {
                    break;
                }
                next = get_next(next);
            }

            // Break links between left run and right run
            let l_end = if right.is_null() {
                std::ptr::null_mut()
            } else {
                get_prev(right)
            };
            if !l_end.is_null() {
                set_next(l_end, std::ptr::null_mut());
            }
            if !right.is_null() {
                set_prev(right, std::ptr::null_mut());
            }

            // Break links between right run and next chunk
            let r_end = if next.is_null() {
                std::ptr::null_mut()
            } else {
                get_prev(next)
            };
            if !r_end.is_null() {
                set_next(r_end, std::ptr::null_mut());
            }
            if !next.is_null() {
                set_prev(next, std::ptr::null_mut());
            }

            // Merge the two runs
            let mut merged: *mut c_void = std::ptr::null_mut();
            let mut merged_tail: *mut c_void = std::ptr::null_mut();
            let mut left_cur = left;
            let mut right_cur = right;

            while !left_cur.is_null() || !right_cur.is_null() {
                // Choose the next element: left if right is done or left <= right
                let take_left = !left_cur.is_null()
                    && (right_cur.is_null() || compare(left_cur, right_cur) <= 0);
                let chosen = if take_left {
                    let c = left_cur;
                    left_cur = get_next(left_cur);
                    c
                } else {
                    let c = right_cur;
                    right_cur = get_next(right_cur);
                    c
                };

                if merged_tail.is_null() {
                    merged = chosen;
                    merged_tail = chosen;
                    set_prev(chosen, std::ptr::null_mut());
                } else {
                    set_next(merged_tail, chosen);
                    set_prev(chosen, merged_tail);
                    merged_tail = chosen;
                }
            }

            // Connect merged run to the accumulating full list
            if new_head.is_null() {
                new_head = merged;
            } else {
                set_next(tail, merged);
                set_prev(merged, tail);
            }

            // Advance tail to end of merged run
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

// =============================================================================
// NvimBox - A Box-like type that uses nvim's allocator
// =============================================================================

/// A smart pointer that allocates using nvim's `xmalloc` and frees with `xfree`.
///
/// Similar to `Box<T>`, but uses Neovim's memory allocator. This is essential
/// when passing allocated memory to C code or receiving memory from C code.
///
/// # Example
///
/// ```ignore
/// let boxed = NvimBox::new(42u32);
/// assert_eq!(*boxed, 42);
/// ```
pub struct NvimBox<T> {
    ptr: NonNull<T>,
}

impl<T> NvimBox<T> {
    /// Allocate and initialize a new `NvimBox`.
    pub fn new(value: T) -> Self {
        let layout = Layout::new::<T>();
        if layout.size() == 0 {
            // For ZST, use a dangling pointer
            return Self {
                ptr: NonNull::dangling(),
            };
        }

        // SAFETY: xmalloc never returns NULL
        let raw = unsafe { xmalloc(layout.size()) };
        let ptr = NonNull::new(raw.cast::<T>()).expect("xmalloc returned NULL");

        // SAFETY: ptr is valid and properly aligned
        unsafe {
            ptr.as_ptr().write(value);
        }

        Self { ptr }
    }

    /// Create from a raw pointer allocated with `xmalloc`.
    ///
    /// # Safety
    ///
    /// - `ptr` must have been allocated with `xmalloc` (or similar nvim allocator)
    /// - `ptr` must be valid and properly initialized
    /// - The caller transfers ownership to `NvimBox`
    pub unsafe fn from_raw(ptr: *mut T) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr })
    }

    /// Consume the box and return the raw pointer.
    ///
    /// The caller is responsible for freeing with `xfree`.
    pub fn into_raw(self) -> *mut T {
        let ptr = self.ptr.as_ptr();
        std::mem::forget(self);
        ptr
    }

    /// Get a raw pointer without consuming the box.
    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    /// Get a mutable raw pointer without consuming the box.
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }
}

impl<T> Deref for NvimBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: ptr is valid and initialized
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> DerefMut for NvimBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: ptr is valid and initialized, we have exclusive access
        unsafe { self.ptr.as_mut() }
    }
}

impl<T> Drop for NvimBox<T> {
    fn drop(&mut self) {
        let layout = Layout::new::<T>();
        // SAFETY: ptr was allocated with xmalloc
        unsafe {
            std::ptr::drop_in_place(self.ptr.as_ptr());
            if layout.size() != 0 {
                xfree(self.ptr.as_ptr().cast());
            }
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for NvimBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: Clone> Clone for NvimBox<T> {
    fn clone(&self) -> Self {
        Self::new((**self).clone())
    }
}

// SAFETY: NvimBox<T> is Send if T is Send
unsafe impl<T: Send> Send for NvimBox<T> {}

// SAFETY: NvimBox<T> is Sync if T is Sync
unsafe impl<T: Sync> Sync for NvimBox<T> {}

// =============================================================================
// NvimString - A C string allocated with nvim's allocator
// =============================================================================

/// A C-compatible string allocated with nvim's `xmallocz`.
///
/// This type ensures proper memory management when working with strings
/// that need to be passed to or from C code.
pub struct NvimString {
    ptr: NonNull<c_char>,
    len: usize,
}

impl NvimString {
    /// Create a new `NvimString` from a Rust string slice.
    pub fn new(s: &str) -> Self {
        let len = s.len();
        // SAFETY: xmallocz allocates len+1 bytes and zeros the last byte
        let ptr = unsafe { xmallocz(len) };
        let ptr = NonNull::new(ptr.cast::<c_char>()).expect("xmallocz returned NULL");

        // Copy the string data
        // SAFETY: ptr is valid for len bytes, s is valid for len bytes
        unsafe {
            std::ptr::copy_nonoverlapping(s.as_ptr(), ptr.as_ptr().cast(), len);
        }

        Self { ptr, len }
    }

    /// Create from a raw C string pointer allocated with nvim's allocator.
    ///
    /// # Safety
    ///
    /// - `ptr` must have been allocated with `xmalloc`/`xmallocz`/`xstrdup`
    /// - `ptr` must be a valid NUL-terminated C string
    /// - The caller transfers ownership to `NvimString`
    pub unsafe fn from_raw(ptr: *mut c_char) -> Option<Self> {
        let ptr = NonNull::new(ptr)?;
        // SAFETY: caller guarantees ptr is a valid C string
        let len = unsafe { CStr::from_ptr(ptr.as_ptr()).to_bytes().len() };
        Some(Self { ptr, len })
    }

    /// Consume the string and return the raw pointer.
    ///
    /// The caller is responsible for freeing with `xfree`.
    pub fn into_raw(self) -> *mut c_char {
        let ptr = self.ptr.as_ptr();
        std::mem::forget(self);
        ptr
    }

    /// Get the raw C string pointer.
    pub fn as_ptr(&self) -> *const c_char {
        self.ptr.as_ptr()
    }

    /// Get the length in bytes (not including the NUL terminator).
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if the string is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the string as a byte slice (not including NUL terminator).
    pub fn as_bytes(&self) -> &[u8] {
        // SAFETY: ptr is valid for len bytes
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr().cast(), self.len) }
    }

    /// Try to convert to a Rust string slice.
    ///
    /// Returns `None` if the string is not valid UTF-8.
    pub fn to_str(&self) -> Option<&str> {
        std::str::from_utf8(self.as_bytes()).ok()
    }
}

impl Drop for NvimString {
    fn drop(&mut self) {
        // SAFETY: ptr was allocated with xmallocz
        unsafe {
            xfree(self.ptr.as_ptr().cast());
        }
    }
}

impl Clone for NvimString {
    fn clone(&self) -> Self {
        // SAFETY: xstrndup is safe to call with valid pointer and length
        let ptr = unsafe { xstrndup(self.ptr.as_ptr(), self.len) };
        let ptr = NonNull::new(ptr).expect("xstrndup returned NULL");
        Self { ptr, len: self.len }
    }
}

impl fmt::Debug for NvimString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.to_str() {
            Some(s) => write!(f, "NvimString({s:?})"),
            None => write!(f, "NvimString({:?})", self.as_bytes()),
        }
    }
}

impl fmt::Display for NvimString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.to_str() {
            Some(s) => write!(f, "{s}"),
            None => write!(f, "{:?}", self.as_bytes()),
        }
    }
}

impl From<&str> for NvimString {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for NvimString {
    fn from(s: String) -> Self {
        Self::new(&s)
    }
}

// SAFETY: NvimString contains no interior mutability and the pointer is valid
unsafe impl Send for NvimString {}
unsafe impl Sync for NvimString {}

// =============================================================================
// NvimVec - A Vec-like type that uses nvim's allocator
// =============================================================================

/// A dynamically sized array using nvim's allocator.
///
/// This is similar to `Vec<T>` but uses `xmalloc`/`xrealloc`/`xfree`
/// for memory management.
pub struct NvimVec<T> {
    ptr: NonNull<T>,
    len: usize,
    capacity: usize,
}

impl<T> NvimVec<T> {
    /// Create a new empty `NvimVec`.
    pub fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            len: 0,
            capacity: 0,
        }
    }

    /// Create a new `NvimVec` with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity == 0 || std::mem::size_of::<T>() == 0 {
            return Self::new();
        }

        let size = capacity
            .checked_mul(std::mem::size_of::<T>())
            .expect("capacity overflow");
        let ptr = unsafe { xmalloc(size) };
        let ptr = NonNull::new(ptr.cast::<T>()).expect("xmalloc returned NULL");

        Self {
            ptr,
            len: 0,
            capacity,
        }
    }

    /// Get the length of the vector.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if the vector is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the capacity of the vector.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Push an element to the end of the vector.
    pub fn push(&mut self, value: T) {
        if self.len == self.capacity {
            self.grow();
        }

        // SAFETY: we just ensured there's space
        unsafe {
            self.ptr.as_ptr().add(self.len).write(value);
        }
        self.len += 1;
    }

    /// Pop an element from the end of the vector.
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;
        // SAFETY: len was > 0, so this index is valid
        Some(unsafe { self.ptr.as_ptr().add(self.len).read() })
    }

    /// Get a slice of the vector's elements.
    pub fn as_slice(&self) -> &[T] {
        if self.len == 0 {
            return &[];
        }
        // SAFETY: ptr is valid for len elements
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    /// Get a mutable slice of the vector's elements.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        if self.len == 0 {
            return &mut [];
        }
        // SAFETY: ptr is valid for len elements, we have exclusive access
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    /// Clear the vector, dropping all elements.
    pub fn clear(&mut self) {
        // Drop all elements
        while self.pop().is_some() {}
    }

    /// Get a raw pointer to the vector's buffer.
    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    /// Get a mutable raw pointer to the vector's buffer.
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }

    fn grow(&mut self) {
        let new_capacity = if self.capacity == 0 {
            4
        } else {
            self.capacity.checked_mul(2).expect("capacity overflow")
        };

        let elem_size = std::mem::size_of::<T>();
        if elem_size == 0 {
            // For ZST, just update capacity
            self.capacity = new_capacity;
            return;
        }

        let new_size = new_capacity
            .checked_mul(elem_size)
            .expect("capacity overflow");

        let new_ptr = if self.capacity == 0 {
            unsafe { xmalloc(new_size) }
        } else {
            unsafe { xrealloc(self.ptr.as_ptr().cast(), new_size) }
        };

        self.ptr = NonNull::new(new_ptr.cast::<T>()).expect("xmalloc/xrealloc returned NULL");
        self.capacity = new_capacity;
    }
}

impl<T> Default for NvimVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for NvimVec<T> {
    fn drop(&mut self) {
        // Drop all elements
        self.clear();

        // Free the buffer
        if self.capacity != 0 && std::mem::size_of::<T>() != 0 {
            unsafe {
                xfree(self.ptr.as_ptr().cast());
            }
        }
    }
}

impl<T> Deref for NvimVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> DerefMut for NvimVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T: Clone> Clone for NvimVec<T> {
    fn clone(&self) -> Self {
        let mut new_vec = Self::with_capacity(self.len);
        for item in self.as_slice() {
            new_vec.push(item.clone());
        }
        new_vec
    }
}

impl<T: fmt::Debug> fmt::Debug for NvimVec<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_slice(), f)
    }
}

// SAFETY: NvimVec<T> is Send if T is Send
unsafe impl<T: Send> Send for NvimVec<T> {}

// SAFETY: NvimVec<T> is Sync if T is Sync
unsafe impl<T: Sync> Sync for NvimVec<T> {}

// =============================================================================
// Convenience functions for C interop
// =============================================================================

/// Allocate memory for a value of type T and return the raw pointer.
///
/// The caller is responsible for freeing with `xfree`.
///
/// # Safety
///
/// The returned pointer must be properly initialized before use.
pub unsafe fn alloc_uninit<T>() -> *mut T {
    let layout = Layout::new::<T>();
    if layout.size() == 0 {
        return NonNull::<T>::dangling().as_ptr();
    }
    unsafe { xmalloc(layout.size()).cast() }
}

/// Allocate a zeroed value of type T and return the raw pointer.
///
/// The caller is responsible for freeing with `xfree`.
pub fn alloc_zeroed<T>() -> *mut T {
    let layout = Layout::new::<T>();
    if layout.size() == 0 {
        return NonNull::<T>::dangling().as_ptr();
    }
    unsafe { xcalloc(1, layout.size()).cast() }
}

/// Free a pointer that was allocated with nvim's allocator.
///
/// # Safety
///
/// - `ptr` must have been allocated with `xmalloc`/`xcalloc`/etc.
/// - `ptr` must not be used after this call
pub unsafe fn free<T>(ptr: *mut T) {
    if !ptr.is_null() && std::mem::size_of::<T>() != 0 {
        unsafe { xfree(ptr.cast()) };
    }
}

// =============================================================================
// Tests (only run when linking with actual nvim, not in isolation)
// =============================================================================

#[cfg(test)]
mod tests {
    // These tests cannot run without the nvim allocator linked.
    // They serve as documentation and can be run with integration tests.

    #[test]
    fn test_module_compiles() {
        // Just verify the module compiles correctly
        // (actual functionality requires linking with nvim)
    }
}
