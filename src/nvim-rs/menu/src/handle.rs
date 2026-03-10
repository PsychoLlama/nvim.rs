//! Typed handle for the `vimmenu_T` struct.
//!
//! `VimMenuHandle` is a thin newtype over `*mut VimMenu` so that field
//! access goes directly through the `#[repr(C)]` struct rather than
//! bouncing through C accessor functions.

use std::ffi::{c_char, c_int};

use crate::vim_menu::VimMenu;

/// Handle to a Neovim menu (`vimmenu_T*`).
///
/// This is a typed pointer – Rust code may dereference it directly
/// because `VimMenu` is `#[repr(C)]` and mirrors `vimmenu_T` exactly.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VimMenuHandle(*mut VimMenu);

impl VimMenuHandle {
    /// Create a new menu handle from a raw `VimMenu` pointer.
    ///
    /// # Safety
    /// The pointer must be a valid `vimmenu_T*` or null.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *mut VimMenu) -> Self {
        Self(ptr)
    }

    /// Create a null handle.
    #[inline]
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Create a null handle (const alias for use in static initializers).
    #[inline]
    #[must_use]
    pub const fn null_const() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Get the raw `VimMenu` pointer.
    #[inline]
    #[must_use]
    pub const fn as_ptr(self) -> *mut VimMenu {
        self.0
    }

    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    // ========================================================================
    // Field accessors – direct field dereference via #[repr(C)] VimMenu
    // ========================================================================

    /// Get the menu's modes field.
    #[inline]
    pub fn modes(self) -> c_int {
        if self.is_null() {
            return 0;
        }
        unsafe { (*self.0).modes }
    }

    /// Get the menu's enabled field.
    #[inline]
    pub fn enabled(self) -> c_int {
        if self.is_null() {
            return 0;
        }
        unsafe { (*self.0).enabled }
    }

    /// Get the menu's name field.
    #[inline]
    pub fn name(self) -> *const c_char {
        if self.is_null() {
            return std::ptr::null();
        }
        unsafe { (*self.0).name }
    }

    /// Get the menu's dname (display name) field.
    #[inline]
    pub fn dname(self) -> *const c_char {
        if self.is_null() {
            return std::ptr::null();
        }
        unsafe { (*self.0).dname }
    }

    /// Get the menu's en_name (English name) field.
    #[inline]
    pub fn en_name(self) -> *const c_char {
        if self.is_null() {
            return std::ptr::null();
        }
        unsafe { (*self.0).en_name }
    }

    /// Get the menu's en_dname (English display name) field.
    #[inline]
    pub fn en_dname(self) -> *const c_char {
        if self.is_null() {
            return std::ptr::null();
        }
        unsafe { (*self.0).en_dname }
    }

    /// Get the menu's priority field.
    #[inline]
    pub fn priority(self) -> c_int {
        if self.is_null() {
            return 0;
        }
        unsafe { (*self.0).priority }
    }

    /// Get the menu's mnemonic field.
    #[inline]
    pub fn mnemonic(self) -> c_int {
        if self.is_null() {
            return 0;
        }
        unsafe { (*self.0).mnemonic }
    }

    /// Get the menu's actext (accelerator text) field.
    #[inline]
    pub fn actext(self) -> *const c_char {
        if self.is_null() {
            return std::ptr::null();
        }
        unsafe { (*self.0).actext }
    }

    /// Get the menu's children field.
    #[inline]
    pub fn children(self) -> VimMenuHandle {
        if self.is_null() {
            return VimMenuHandle::null();
        }
        VimMenuHandle(unsafe { (*self.0).children })
    }

    /// Get the menu's parent field.
    #[inline]
    pub fn parent(self) -> VimMenuHandle {
        if self.is_null() {
            return VimMenuHandle::null();
        }
        VimMenuHandle(unsafe { (*self.0).parent })
    }

    /// Get the menu's next sibling field.
    #[inline]
    pub fn next(self) -> VimMenuHandle {
        if self.is_null() {
            return VimMenuHandle::null();
        }
        VimMenuHandle(unsafe { (*self.0).next })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_handle() {
        let handle = VimMenuHandle::null();
        assert!(handle.is_null());
        assert_eq!(handle.as_ptr(), std::ptr::null_mut());
    }

    #[test]
    fn test_null_handle_field_defaults() {
        let handle = VimMenuHandle::null();
        assert_eq!(handle.modes(), 0);
        assert_eq!(handle.enabled(), 0);
        assert!(handle.name().is_null());
        assert!(handle.dname().is_null());
        assert!(handle.en_name().is_null());
        assert!(handle.en_dname().is_null());
        assert_eq!(handle.priority(), 0);
        assert!(handle.actext().is_null());
        assert!(handle.children().is_null());
        assert!(handle.parent().is_null());
        assert!(handle.next().is_null());
    }
}
