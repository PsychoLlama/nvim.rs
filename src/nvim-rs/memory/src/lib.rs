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
