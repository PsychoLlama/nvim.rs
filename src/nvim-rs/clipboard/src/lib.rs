//! Clipboard integration for Neovim
//!
//! This crate provides Rust implementations for clipboard operations,
//! including system clipboard interaction, selection types, and provider
//! abstraction.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::missing_panics_doc)]

pub mod async_ops;
pub mod provider;
pub mod selection;

use std::ffi::{c_int, c_uint};

// Re-export key types
pub use async_ops::{ClipboardOperation, ClipboardRequest, ClipboardResult};
pub use provider::{ClipboardProvider, ProviderCapabilities, ProviderStatus};
pub use selection::{ClipboardData, SelectionType};

// =============================================================================
// Opaque Handle Types
// =============================================================================

/// Opaque handle for yankreg_T pointers from C
type YankregHandle = std::ffi::c_void;

// =============================================================================
// Clipboard Flags
// =============================================================================

/// kOptCbFlagUnnamed (from generated option_vars.h)
const CB_FLAG_UNNAMED: c_uint = 0x01;
/// kOptCbFlagUnnamedplus (from generated option_vars.h)
const CB_FLAG_UNNAMEDPLUS: c_uint = 0x02;

/// Clipboard flags (maps to cb_flags in C)
pub const CB_UNNAMED: u32 = 0x0001;
pub const CB_UNNAMEDPLUS: u32 = 0x0002;

/// Register array index for '*' (from register_defs.h)
const STAR_REG_INDEX: c_int = 37;
/// Register array index for '+' (from register_defs.h)
const PLUS_REG_INDEX: c_int = 38;

/// Clipboard flags wrapper
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ClipboardFlags {
    flags: u32,
}

impl ClipboardFlags {
    /// Create with no flags
    pub const fn none() -> Self {
        Self { flags: 0 }
    }

    /// Create from raw value
    pub const fn from_raw(flags: u32) -> Self {
        Self { flags }
    }

    /// Get raw value
    pub const fn as_raw(self) -> u32 {
        self.flags
    }

    /// Check if unnamed is set
    pub const fn has_unnamed(self) -> bool {
        (self.flags & CB_UNNAMED) != 0
    }

    /// Check if unnamedplus is set
    pub const fn has_unnamedplus(self) -> bool {
        (self.flags & CB_UNNAMEDPLUS) != 0
    }

    /// Check if any clipboard register is enabled
    pub const fn has_any(self) -> bool {
        self.has_unnamed() || self.has_unnamedplus()
    }

    /// Set unnamed flag
    pub fn set_unnamed(&mut self, value: bool) {
        if value {
            self.flags |= CB_UNNAMED;
        } else {
            self.flags &= !CB_UNNAMED;
        }
    }

    /// Set unnamedplus flag
    pub fn set_unnamedplus(&mut self, value: bool) {
        if value {
            self.flags |= CB_UNNAMEDPLUS;
        } else {
            self.flags &= !CB_UNNAMEDPLUS;
        }
    }
}

// =============================================================================
// Batch State
// =============================================================================

/// State for batch clipboard operations
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BatchState {
    /// Batch change count (nested level)
    pub count: c_int,
    /// Whether clipboard updates are delayed
    pub delay_update: bool,
    /// Whether clipboard needs update after batch
    pub needs_update: bool,
}

impl Default for BatchState {
    fn default() -> Self {
        Self {
            count: 0,
            delay_update: false,
            needs_update: false,
        }
    }
}

impl BatchState {
    /// Create new empty batch state
    pub const fn new() -> Self {
        Self {
            count: 0,
            delay_update: false,
            needs_update: false,
        }
    }

    /// Check if we're in a batch
    pub const fn in_batch(&self) -> bool {
        self.count > 0
    }

    /// Start a batch operation
    pub fn start_batch(&mut self) -> bool {
        self.count += 1;
        if self.count == 1 {
            self.delay_update = true;
            true // First level
        } else {
            false // Nested
        }
    }

    /// End a batch operation, returns true if batch is complete
    pub fn end_batch(&mut self) -> bool {
        if self.count > 0 {
            self.count -= 1;
        }
        if self.count == 0 {
            self.delay_update = false;
            true // Batch complete
        } else {
            false // Still in nested batch
        }
    }

    /// Mark that clipboard needs update
    pub fn mark_needs_update(&mut self) {
        if self.delay_update {
            self.needs_update = true;
        }
    }

    /// Clear needs update flag
    pub fn clear_needs_update(&mut self) {
        self.needs_update = false;
    }

    /// Save and reset batch state
    pub fn save_and_reset(&mut self) -> c_int {
        let saved = self.count;
        self.count = 0;
        self.delay_update = false;
        saved
    }

    /// Restore batch state
    pub fn restore(&mut self, saved_count: c_int) {
        self.count = saved_count;
        if self.count > 0 {
            self.delay_update = true;
        }
    }
}

// =============================================================================
// Register Name
// =============================================================================

/// Character code for '*' register
pub const STAR_REGISTER: c_int = b'*' as c_int;
/// Character code for '+' register
pub const PLUS_REGISTER: c_int = b'+' as c_int;
/// Character code for '"' register
pub const UNNAMED_REGISTER: c_int = b'"' as c_int;
/// NUL register (unnamed operation)
pub const NUL_REGISTER: c_int = 0;

/// Check if a register name is a clipboard register
pub const fn is_clipboard_register(name: c_int) -> bool {
    name == STAR_REGISTER || name == PLUS_REGISTER
}

/// Clipboard register type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClipboardRegister {
    /// Not a clipboard register
    #[default]
    None = 0,
    /// PRIMARY selection (*)
    Star = 1,
    /// CLIPBOARD selection (+)
    Plus = 2,
}

impl ClipboardRegister {
    /// Create from register name
    pub const fn from_name(name: c_int) -> Self {
        match name {
            0x2A => Self::Star, // b'*'
            0x2B => Self::Plus, // b'+'
            _ => Self::None,
        }
    }

    /// Convert to register name
    pub const fn to_name(self) -> c_int {
        match self {
            Self::None => 0,
            Self::Star => STAR_REGISTER,
            Self::Plus => PLUS_REGISTER,
        }
    }

    /// Check if this is a valid clipboard register
    pub const fn is_valid(self) -> bool {
        !matches!(self, Self::None)
    }
}

// =============================================================================
// Adjust Result
// =============================================================================

/// Result of adjusting clipboard name
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AdjustResult {
    /// Whether a clipboard register should be used
    pub use_clipboard: bool,
    /// The adjusted register name
    pub name: c_int,
    /// Which clipboard register to use
    pub register: ClipboardRegister,
    /// Whether provider is available
    pub provider_available: bool,
}

impl Default for AdjustResult {
    fn default() -> Self {
        Self {
            use_clipboard: false,
            name: 0,
            register: ClipboardRegister::None,
            provider_available: false,
        }
    }
}

impl AdjustResult {
    /// Create a "no clipboard" result
    pub const fn no_clipboard() -> Self {
        Self {
            use_clipboard: false,
            name: 0,
            register: ClipboardRegister::None,
            provider_available: false,
        }
    }

    /// Create a "use clipboard" result
    pub const fn use_clipboard(name: c_int, register: ClipboardRegister) -> Self {
        Self {
            use_clipboard: true,
            name,
            register,
            provider_available: true,
        }
    }

    /// Create a "no provider" result
    pub const fn no_provider() -> Self {
        Self {
            use_clipboard: false,
            name: 0,
            register: ClipboardRegister::None,
            provider_available: false,
        }
    }
}

// =============================================================================
// Module State (replaces C static variables)
// =============================================================================

struct ClipboardModuleState {
    batch: BatchState,
    didwarn: bool,
}

impl ClipboardModuleState {
    const fn new() -> Self {
        Self {
            batch: BatchState::new(),
            didwarn: false,
        }
    }
}

static mut CLIPBOARD_STATE: ClipboardModuleState = ClipboardModuleState::new();

// =============================================================================
// C Accessor Declarations (extern "C")
// =============================================================================

extern "C" {
    /// Get cb_flags option value
    fn nvim_option_get_cb_flags() -> c_uint;
    /// Check if clipboard provider is available
    fn nvim_clipboard_eval_has_provider() -> bool;
    /// Show a non-error message
    fn nvim_clipboard_msg(s: *const std::ffi::c_char);
    /// Check if output is being redirected
    fn nvim_clipboard_redirecting() -> bool;
    /// Get register by array index (0-38)
    fn get_y_register(reg: c_int) -> *mut YankregHandle;
    /// Get previous yank register
    fn get_y_previous() -> *mut YankregHandle;
    /// Free a register's contents
    fn nvim_free_register(reg: *mut YankregHandle);
    /// Provider get: calls eval_call_provider("clipboard","get",...) and populates reg
    fn nvim_clipboard_provider_get(name: c_int, reg: *mut YankregHandle) -> bool;
    /// Provider set: builds list from reg and calls eval_call_provider("clipboard","set",...)
    fn nvim_clipboard_provider_set(name: c_int, reg: *mut YankregHandle);
    /// Update register width after populating y_array
    fn update_yankreg_width(reg: *mut YankregHandle);
}

// =============================================================================
// Core Logic
// =============================================================================

const MSG_NO_CLIP: &[u8] = b"clipboard: No provider. Try \":checkhealth\" or \":h clipboard\".\0";

/// Internal implementation of adjust_clipboard_name.
///
/// Checks whether `name` refers to a clipboard register (explicit `*`/`+`
/// or implicit via clipboard=unnamed[plus]) and returns the yankreg_T pointer
/// if so, or null if not.
///
/// # Safety
/// `name` must be a valid pointer.
unsafe fn adjust_clipboard_name_impl(
    name: *mut c_int,
    quiet: bool,
    writing: bool,
) -> *mut YankregHandle {
    let s = &raw mut CLIPBOARD_STATE;
    let n = unsafe { *name };
    let cb_flags = unsafe { nvim_option_get_cb_flags() };

    let explicit_cb_reg = n == b'*' as c_int || n == b'+' as c_int;
    let implicit_cb_reg = n == 0 && (cb_flags & (CB_FLAG_UNNAMED | CB_FLAG_UNNAMEDPLUS)) != 0;

    if !explicit_cb_reg && !implicit_cb_reg {
        return std::ptr::null_mut();
    }

    if !unsafe { nvim_clipboard_eval_has_provider() } {
        let batch_count = unsafe { (*s).batch.count };
        let didwarn = unsafe { (*s).didwarn };
        if batch_count <= 1
            && !quiet
            && (!didwarn || (explicit_cb_reg && !unsafe { nvim_clipboard_redirecting() }))
        {
            unsafe { (*s).didwarn = true };
            // Use msg() not emsg() — interrupting :redir causes a weird state
            unsafe { nvim_clipboard_msg(MSG_NO_CLIP.as_ptr().cast()) };
        }
        return std::ptr::null_mut();
    }

    if explicit_cb_reg {
        let reg_idx = if n == b'*' as c_int {
            STAR_REG_INDEX
        } else {
            PLUS_REG_INDEX
        };
        let target = unsafe { get_y_register(reg_idx) };
        if writing
            && (cb_flags
                & if n == b'*' as c_int {
                    CB_FLAG_UNNAMED
                } else {
                    CB_FLAG_UNNAMEDPLUS
                })
                != 0
        {
            unsafe { (*s).batch.needs_update = false };
        }
        target
    } else {
        // Unnamed register: "implicit" clipboard
        let delay_update = unsafe { (*s).batch.delay_update };
        let needs_update = unsafe { (*s).batch.needs_update };
        if writing && delay_update {
            unsafe { (*s).batch.needs_update = true };
            return std::ptr::null_mut();
        } else if !writing && needs_update {
            return std::ptr::null_mut();
        }

        if cb_flags & CB_FLAG_UNNAMEDPLUS != 0 {
            unsafe {
                *name = if cb_flags & CB_FLAG_UNNAMED != 0 && writing {
                    b'"' as c_int
                } else {
                    b'+' as c_int
                };
            }
            unsafe { get_y_register(PLUS_REG_INDEX) }
        } else {
            unsafe { *name = b'*' as c_int };
            unsafe { get_y_register(STAR_REG_INDEX) }
        }
    }
}

/// Flush deferred clipboard set if needed
unsafe fn flush_clipboard_if_needed() {
    let s = &raw mut CLIPBOARD_STATE;
    if unsafe { (*s).batch.needs_update } {
        unsafe { (*s).batch.needs_update = false };
        // unnamed ("implicit" clipboard)
        let prev = unsafe { get_y_previous() };
        unsafe { rs_set_clipboard(0, prev) };
    }
}

// =============================================================================
// FFI Exports — Main clipboard functions
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_adjust_clipboard_name(
    name: *mut c_int,
    quiet: bool,
    writing: bool,
) -> *mut YankregHandle {
    unsafe { adjust_clipboard_name_impl(name, quiet, writing) }
}

#[no_mangle]
pub unsafe extern "C" fn rs_get_clipboard(
    name: c_int,
    target: *mut *mut YankregHandle,
    quiet: bool,
) -> bool {
    let mut name = name;
    let reg = unsafe { adjust_clipboard_name_impl(&raw mut name, quiet, false) };
    if reg.is_null() {
        return false;
    }
    unsafe { nvim_free_register(reg) };

    let ok = unsafe { nvim_clipboard_provider_get(name, reg) };
    if ok {
        unsafe { update_yankreg_width(reg) };
    }
    unsafe { *target = reg };
    ok
}

#[no_mangle]
pub unsafe extern "C" fn rs_set_clipboard(name: c_int, reg: *mut YankregHandle) {
    let mut name = name;
    let target = unsafe { adjust_clipboard_name_impl(&raw mut name, false, true) };
    if target.is_null() {
        return;
    }
    unsafe { nvim_clipboard_provider_set(name, reg) };
}

#[no_mangle]
pub unsafe extern "C" fn rs_start_batch_changes() {
    let s = &raw mut CLIPBOARD_STATE;
    unsafe {
        (*s).batch.count += 1;
        if (*s).batch.count > 1 {
            return;
        }
        (*s).batch.delay_update = true;
    }
}

#[no_mangle]
pub unsafe extern "C" fn rs_end_batch_changes() {
    let s = &raw mut CLIPBOARD_STATE;
    unsafe {
        (*s).batch.count -= 1;
        if (*s).batch.count > 0 {
            return;
        }
        (*s).batch.delay_update = false;
    }
    unsafe { flush_clipboard_if_needed() };
}

#[no_mangle]
pub unsafe extern "C" fn rs_save_batch_count() -> c_int {
    let s = &raw mut CLIPBOARD_STATE;
    let save_count = unsafe { (*s).batch.count };
    unsafe {
        (*s).batch.count = 0;
        (*s).batch.delay_update = false;
    }
    unsafe { flush_clipboard_if_needed() };
    save_count
}

#[no_mangle]
pub unsafe extern "C" fn rs_restore_batch_count(save_count: c_int) {
    let s = &raw mut CLIPBOARD_STATE;
    unsafe {
        assert!((*s).batch.count == 0);
        (*s).batch.count = save_count;
        if (*s).batch.count > 0 {
            (*s).batch.delay_update = true;
        }
    }
}

// =============================================================================
// FFI Exports — Utility (existing)
// =============================================================================

/// FFI export: Check if register is clipboard
#[no_mangle]
pub extern "C" fn rs_clipboard_is_clipboard_register(name: c_int) -> c_int {
    c_int::from(is_clipboard_register(name))
}

/// FFI export: Get clipboard register from name
#[no_mangle]
pub extern "C" fn rs_clipboard_register_from_name(name: c_int) -> ClipboardRegister {
    ClipboardRegister::from_name(name)
}

/// FFI export: Check if flags has unnamed
#[no_mangle]
pub extern "C" fn rs_clipboard_flags_has_unnamed(flags: u32) -> c_int {
    c_int::from(ClipboardFlags::from_raw(flags).has_unnamed())
}

/// FFI export: Check if flags has unnamedplus
#[no_mangle]
pub extern "C" fn rs_clipboard_flags_has_unnamedplus(flags: u32) -> c_int {
    c_int::from(ClipboardFlags::from_raw(flags).has_unnamedplus())
}

/// FFI export: Create new batch state
#[no_mangle]
pub extern "C" fn rs_clipboard_batch_new() -> BatchState {
    BatchState::new()
}

/// FFI export: Start batch operation
#[no_mangle]
pub extern "C" fn rs_clipboard_batch_start(state: *mut BatchState) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*state).start_batch() })
}

/// FFI export: End batch operation
#[no_mangle]
pub extern "C" fn rs_clipboard_batch_end(state: *mut BatchState) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*state).end_batch() })
}

/// FFI export: Check if in batch
#[no_mangle]
pub extern "C" fn rs_clipboard_batch_in_batch(state: *const BatchState) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*state).in_batch() })
}

/// FFI export: Check if needs update
#[no_mangle]
pub extern "C" fn rs_clipboard_batch_needs_update(state: *const BatchState) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*state).needs_update })
}

/// FFI export: Mark needs update
#[no_mangle]
pub extern "C" fn rs_clipboard_batch_mark_update(state: *mut BatchState) {
    if !state.is_null() {
        unsafe { (*state).mark_needs_update() }
    }
}

/// FFI export: Clear needs update
#[no_mangle]
pub extern "C" fn rs_clipboard_batch_clear_update(state: *mut BatchState) {
    if !state.is_null() {
        unsafe { (*state).clear_needs_update() }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::borrow_as_ptr)]
#[allow(clippy::cast_lossless)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_flags() {
        let flags = ClipboardFlags::none();
        assert!(!flags.has_unnamed());
        assert!(!flags.has_unnamedplus());

        let flags = ClipboardFlags::from_raw(CB_UNNAMED);
        assert!(flags.has_unnamed());
        assert!(!flags.has_unnamedplus());

        let flags = ClipboardFlags::from_raw(CB_UNNAMED | CB_UNNAMEDPLUS);
        assert!(flags.has_any());
    }

    #[test]
    fn test_clipboard_flags_set() {
        let mut flags = ClipboardFlags::none();
        flags.set_unnamed(true);
        assert!(flags.has_unnamed());

        flags.set_unnamedplus(true);
        assert!(flags.has_unnamedplus());

        flags.set_unnamed(false);
        assert!(!flags.has_unnamed());
    }

    #[test]
    fn test_batch_state() {
        let mut batch = BatchState::new();
        assert!(!batch.in_batch());

        assert!(batch.start_batch());
        assert!(batch.in_batch());
        assert!(batch.delay_update);

        assert!(!batch.start_batch()); // Nested
        assert!(batch.in_batch());

        assert!(!batch.end_batch()); // Still nested
        assert!(batch.in_batch());

        assert!(batch.end_batch()); // Complete
        assert!(!batch.in_batch());
    }

    #[test]
    fn test_batch_needs_update() {
        let mut batch = BatchState::new();
        batch.start_batch();
        batch.mark_needs_update();
        assert!(batch.needs_update);

        batch.clear_needs_update();
        assert!(!batch.needs_update);
    }

    #[test]
    fn test_batch_save_restore() {
        let mut batch = BatchState::new();
        batch.start_batch();
        batch.start_batch();
        assert_eq!(batch.count, 2);

        let saved = batch.save_and_reset();
        assert_eq!(saved, 2);
        assert_eq!(batch.count, 0);
        assert!(!batch.delay_update);

        batch.restore(saved);
        assert_eq!(batch.count, 2);
        assert!(batch.delay_update);
    }

    #[test]
    fn test_clipboard_register() {
        assert!(is_clipboard_register(STAR_REGISTER));
        assert!(is_clipboard_register(PLUS_REGISTER));
        assert!(!is_clipboard_register(UNNAMED_REGISTER));

        assert_eq!(
            ClipboardRegister::from_name(STAR_REGISTER),
            ClipboardRegister::Star
        );
        assert_eq!(
            ClipboardRegister::from_name(PLUS_REGISTER),
            ClipboardRegister::Plus
        );
        assert_eq!(
            ClipboardRegister::from_name(b'a' as c_int),
            ClipboardRegister::None
        );

        assert!(ClipboardRegister::Star.is_valid());
        assert!(!ClipboardRegister::None.is_valid());
    }

    #[test]
    fn test_adjust_result() {
        let no_clip = AdjustResult::no_clipboard();
        assert!(!no_clip.use_clipboard);

        let use_clip = AdjustResult::use_clipboard(STAR_REGISTER, ClipboardRegister::Star);
        assert!(use_clip.use_clipboard);
        assert_eq!(use_clip.name, STAR_REGISTER);
    }

    #[test]
    fn test_ffi_exports() {
        assert_eq!(rs_clipboard_is_clipboard_register(STAR_REGISTER), 1);
        assert_eq!(rs_clipboard_is_clipboard_register(UNNAMED_REGISTER), 0);

        assert_eq!(rs_clipboard_flags_has_unnamed(CB_UNNAMED), 1);
        assert_eq!(rs_clipboard_flags_has_unnamedplus(CB_UNNAMED), 0);

        let mut batch = rs_clipboard_batch_new();
        assert_eq!(rs_clipboard_batch_in_batch(&batch), 0);

        rs_clipboard_batch_start(&mut batch);
        assert_eq!(rs_clipboard_batch_in_batch(&batch), 1);
    }

    #[test]
    fn test_register_index_constants() {
        // Verify that our register index constants match the character-to-index mapping
        assert_eq!(STAR_REG_INDEX, 37);
        assert_eq!(PLUS_REG_INDEX, 38);
        // Character constants
        assert_eq!(STAR_REGISTER, 42); // b'*'
        assert_eq!(PLUS_REGISTER, 43); // b'+'
    }

    #[test]
    fn test_cb_flag_constants() {
        assert_eq!(CB_FLAG_UNNAMED, 0x01);
        assert_eq!(CB_FLAG_UNNAMEDPLUS, 0x02);
    }
}
