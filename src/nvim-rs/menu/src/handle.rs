//! Opaque handle types for menu structures.
//!
//! This module provides the `VimMenuHandle` type, which is an opaque pointer
//! to a `vimmenu_T` structure in C. All field access is done through C
//! accessor functions declared in this module.

use std::ffi::{c_char, c_int, c_void};

/// Opaque handle to a Neovim menu (`vimmenu_T*`).
///
/// This is an opaque pointer type - Rust code should not attempt to
/// dereference or inspect the contents. All field access is done
/// through C accessor functions.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VimMenuHandle(*mut c_void);

impl VimMenuHandle {
    /// Create a new menu handle from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid `vimmenu_T*` or null.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *mut c_void) -> Self {
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

    /// Get the raw pointer.
    #[inline]
    #[must_use]
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }

    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    // ========================================================================
    // Field accessors using C functions
    // ========================================================================

    /// Get the menu's modes field.
    #[inline]
    pub fn modes(self) -> c_int {
        if self.is_null() {
            return 0;
        }
        unsafe { nvim_menu_get_modes(self) }
    }

    /// Get the menu's enabled field.
    #[inline]
    pub fn enabled(self) -> c_int {
        if self.is_null() {
            return 0;
        }
        unsafe { nvim_menu_get_enabled(self) }
    }

    /// Get the menu's name field.
    #[inline]
    pub fn name(self) -> *const c_char {
        if self.is_null() {
            return std::ptr::null();
        }
        unsafe { nvim_menu_get_name(self) }
    }

    /// Get the menu's dname (display name) field.
    #[inline]
    pub fn dname(self) -> *const c_char {
        if self.is_null() {
            return std::ptr::null();
        }
        unsafe { nvim_menu_get_dname(self) }
    }

    /// Get the menu's en_name (English name) field.
    #[inline]
    pub fn en_name(self) -> *const c_char {
        if self.is_null() {
            return std::ptr::null();
        }
        unsafe { nvim_menu_get_en_name(self) }
    }

    /// Get the menu's en_dname (English display name) field.
    #[inline]
    pub fn en_dname(self) -> *const c_char {
        if self.is_null() {
            return std::ptr::null();
        }
        unsafe { nvim_menu_get_en_dname(self) }
    }

    /// Get the menu's priority field.
    #[inline]
    pub fn priority(self) -> c_int {
        if self.is_null() {
            return 0;
        }
        unsafe { nvim_menu_get_priority(self) }
    }

    /// Get the menu's children field.
    #[inline]
    pub fn children(self) -> VimMenuHandle {
        if self.is_null() {
            return VimMenuHandle::null();
        }
        unsafe { nvim_menu_get_children(self) }
    }

    /// Get the menu's parent field.
    #[inline]
    pub fn parent(self) -> VimMenuHandle {
        if self.is_null() {
            return VimMenuHandle::null();
        }
        unsafe { nvim_menu_get_parent(self) }
    }

    /// Get the menu's next sibling field.
    #[inline]
    pub fn next(self) -> VimMenuHandle {
        if self.is_null() {
            return VimMenuHandle::null();
        }
        unsafe { nvim_menu_get_next(self) }
    }

    /// Get the menu's mnemonic field.
    #[inline]
    pub fn mnemonic(self) -> c_int {
        if self.is_null() {
            return 0;
        }
        unsafe { nvim_menu_get_mnemonic(self) }
    }

    /// Get the menu's actext (accelerator text) field.
    #[inline]
    pub fn actext(self) -> *const c_char {
        if self.is_null() {
            return std::ptr::null();
        }
        unsafe { nvim_menu_get_actext(self) }
    }
}

// ============================================================================
// C accessor function declarations
// ============================================================================

extern "C" {
    /// Get the modes field from a menu.
    fn nvim_menu_get_modes(menu: VimMenuHandle) -> c_int;

    /// Get the enabled field from a menu.
    fn nvim_menu_get_enabled(menu: VimMenuHandle) -> c_int;

    /// Get the name field from a menu.
    fn nvim_menu_get_name(menu: VimMenuHandle) -> *const c_char;

    /// Get the dname (display name) field from a menu.
    fn nvim_menu_get_dname(menu: VimMenuHandle) -> *const c_char;

    /// Get the en_name (English name) field from a menu.
    fn nvim_menu_get_en_name(menu: VimMenuHandle) -> *const c_char;

    /// Get the en_dname (English display name) field from a menu.
    fn nvim_menu_get_en_dname(menu: VimMenuHandle) -> *const c_char;

    /// Get the priority field from a menu.
    fn nvim_menu_get_priority(menu: VimMenuHandle) -> c_int;

    /// Get the children field from a menu.
    fn nvim_menu_get_children(menu: VimMenuHandle) -> VimMenuHandle;

    /// Get the parent field from a menu.
    fn nvim_menu_get_parent(menu: VimMenuHandle) -> VimMenuHandle;

    /// Get the next sibling field from a menu.
    fn nvim_menu_get_next(menu: VimMenuHandle) -> VimMenuHandle;

    /// Get the mnemonic field from a menu.
    fn nvim_menu_get_mnemonic(menu: VimMenuHandle) -> c_int;

    /// Get the actext (accelerator text) field from a menu.
    fn nvim_menu_get_actext(menu: VimMenuHandle) -> *const c_char;
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

    // Note: Accessor tests require linking with the C library,
    // so they are tested via integration tests, not unit tests.
    // The accessor methods have null checks and return safe defaults for null handles.

    #[test]
    fn test_from_ptr() {
        let dummy: c_int = 42;
        let ptr = &dummy as *const c_int as *mut c_void;
        let handle = unsafe { VimMenuHandle::from_ptr(ptr) };
        assert!(!handle.is_null());
        assert_eq!(handle.as_ptr(), ptr);
    }
}
