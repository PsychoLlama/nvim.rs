//! Accessor functions for DecorProvider fields
//!
//! This module provides FFI wrapper functions to access DecorProvider struct
//! fields from Rust. The actual struct is owned by C, and Rust uses accessor
//! functions to read/write fields.
//!
//! # Architecture
//!
//! DecorProvider fields are accessed via C accessor functions defined in
//! `decoration_provider.c`. This opaque handle pattern allows:
//! - C to own the provider storage (kvec_t)
//! - Rust to safely read/write fields through typed functions
//! - Future migration of accessors to Rust as needed

use std::ffi::{c_int, c_void};

use crate::constants::{DECOR_PROVIDER_DISABLED, LUA_NOREF};
use crate::lifecycle::get_decor_provider;
use crate::types::DecorProviderHandle;

// =============================================================================
// C Pointer-based Field Accessors (defined in decoration_provider.c)
// =============================================================================

extern "C" {
    fn nvim_decor_provider_ptr_get_hl_valid(p: *mut c_void) -> c_int;
    fn nvim_decor_provider_ptr_get_hl_cached(p: *mut c_void) -> bool;
    fn nvim_decor_provider_ptr_set_hl_cached(p: *mut c_void, val: bool);
    fn nvim_decor_provider_ptr_has_hl_def(p: *mut c_void) -> bool;
    fn nvim_decor_provider_ptr_get_hl_valid_and_clear_cached(p: *mut c_void) -> c_int;
    fn nvim_decor_provider_ptr_get_redraw_start(p: *mut c_void) -> c_int;
    fn nvim_decor_provider_ptr_get_redraw_buf(p: *mut c_void) -> c_int;
    fn nvim_decor_provider_ptr_get_redraw_win(p: *mut c_void) -> c_int;
    fn nvim_decor_provider_ptr_get_redraw_line(p: *mut c_void) -> c_int;
    fn nvim_decor_provider_ptr_get_redraw_range(p: *mut c_void) -> c_int;
    fn nvim_decor_provider_ptr_get_redraw_end(p: *mut c_void) -> c_int;
    fn nvim_decor_provider_ptr_get_spell_nav(p: *mut c_void) -> c_int;
    fn nvim_decor_provider_ptr_get_conceal_line(p: *mut c_void) -> c_int;
}

// =============================================================================
// Namespace-based Exported Functions (Phase 5 migration)
// =============================================================================

/// Get hl_valid for a namespace. Returns -1 if provider doesn't exist.
///
/// Migrated from C `nvim_decor_provider_get_hl_valid`.
#[unsafe(export_name = "nvim_decor_provider_get_hl_valid")]
pub unsafe extern "C" fn nvim_decor_provider_get_hl_valid_rs(ns_id: c_int) -> c_int {
    let p = unsafe { get_decor_provider(ns_id, false) };
    if p.is_null() {
        -1
    } else {
        unsafe { nvim_decor_provider_ptr_get_hl_valid(p) }
    }
}

/// Get hl_cached for a namespace. Returns false if provider doesn't exist.
///
/// Migrated from C `nvim_decor_provider_get_hl_cached`.
#[unsafe(export_name = "nvim_decor_provider_get_hl_cached")]
pub unsafe extern "C" fn nvim_decor_provider_get_hl_cached_rs(ns_id: c_int) -> bool {
    let p = unsafe { get_decor_provider(ns_id, false) };
    if p.is_null() {
        false
    } else {
        unsafe { nvim_decor_provider_ptr_get_hl_cached(p) }
    }
}

/// Set hl_cached for a namespace. Creates provider if force=true.
///
/// Migrated from C `nvim_decor_provider_set_hl_cached`.
#[unsafe(export_name = "nvim_decor_provider_set_hl_cached")]
pub unsafe extern "C" fn nvim_decor_provider_set_hl_cached_rs(
    ns_id: c_int,
    cached: bool,
    force: bool,
) {
    let p = unsafe { get_decor_provider(ns_id, force) };
    if !p.is_null() {
        unsafe { nvim_decor_provider_ptr_set_hl_cached(p, cached) };
    }
}

/// Get hl_valid and set hl_cached=false atomically. Creates provider if needed.
///
/// Migrated from C `nvim_decor_provider_hl_def_prepare`.
#[unsafe(export_name = "nvim_decor_provider_hl_def_prepare")]
pub unsafe extern "C" fn nvim_decor_provider_hl_def_prepare_rs(ns_id: c_int) -> c_int {
    let p = unsafe { get_decor_provider(ns_id, true) };
    // get_decor_provider with force=true always returns non-null
    unsafe { nvim_decor_provider_ptr_get_hl_valid_and_clear_cached(p) }
}

/// Check if namespace has a hl_def callback defined.
///
/// Migrated from C `nvim_decor_provider_has_hl_def`.
#[unsafe(export_name = "nvim_decor_provider_has_hl_def")]
pub unsafe extern "C" fn nvim_decor_provider_has_hl_def_rs(ns_id: c_int) -> bool {
    let p = unsafe { get_decor_provider(ns_id, false) };
    if p.is_null() {
        false
    } else {
        unsafe { nvim_decor_provider_ptr_has_hl_def(p) }
    }
}

// =============================================================================
// Pointer-based Exported Functions (Phase 5 migration)
// =============================================================================

/// Get redraw_start callback ref from provider pointer.
///
/// Migrated from C `nvim_decor_provider_get_redraw_start`.
#[unsafe(export_name = "nvim_decor_provider_get_redraw_start")]
pub unsafe extern "C" fn nvim_decor_provider_get_redraw_start_rs(
    provider: DecorProviderHandle,
) -> c_int {
    unsafe { nvim_decor_provider_ptr_get_redraw_start(provider.as_ptr()) }
}

/// Get redraw_buf callback ref from provider pointer.
///
/// Migrated from C `nvim_decor_provider_get_redraw_buf`.
#[unsafe(export_name = "nvim_decor_provider_get_redraw_buf")]
pub unsafe extern "C" fn nvim_decor_provider_get_redraw_buf_rs(
    provider: DecorProviderHandle,
) -> c_int {
    unsafe { nvim_decor_provider_ptr_get_redraw_buf(provider.as_ptr()) }
}

/// Get redraw_win callback ref from provider pointer.
///
/// Migrated from C `nvim_decor_provider_get_redraw_win`.
#[unsafe(export_name = "nvim_decor_provider_get_redraw_win")]
pub unsafe extern "C" fn nvim_decor_provider_get_redraw_win_rs(
    provider: DecorProviderHandle,
) -> c_int {
    unsafe { nvim_decor_provider_ptr_get_redraw_win(provider.as_ptr()) }
}

/// Get redraw_line callback ref from provider pointer.
///
/// Migrated from C `nvim_decor_provider_get_redraw_line`.
#[unsafe(export_name = "nvim_decor_provider_get_redraw_line")]
pub unsafe extern "C" fn nvim_decor_provider_get_redraw_line_rs(
    provider: DecorProviderHandle,
) -> c_int {
    unsafe { nvim_decor_provider_ptr_get_redraw_line(provider.as_ptr()) }
}

/// Get redraw_range callback ref from provider pointer.
///
/// Migrated from C `nvim_decor_provider_get_redraw_range`.
#[unsafe(export_name = "nvim_decor_provider_get_redraw_range")]
pub unsafe extern "C" fn nvim_decor_provider_get_redraw_range_rs(
    provider: DecorProviderHandle,
) -> c_int {
    unsafe { nvim_decor_provider_ptr_get_redraw_range(provider.as_ptr()) }
}

/// Get redraw_end callback ref from provider pointer.
///
/// Migrated from C `nvim_decor_provider_get_redraw_end`.
#[unsafe(export_name = "nvim_decor_provider_get_redraw_end")]
pub unsafe extern "C" fn nvim_decor_provider_get_redraw_end_rs(
    provider: DecorProviderHandle,
) -> c_int {
    unsafe { nvim_decor_provider_ptr_get_redraw_end(provider.as_ptr()) }
}

/// Get spell_nav callback ref from provider pointer.
///
/// Migrated from C `nvim_decor_provider_get_spell_nav`.
#[unsafe(export_name = "nvim_decor_provider_get_spell_nav")]
pub unsafe extern "C" fn nvim_decor_provider_get_spell_nav_rs(
    provider: DecorProviderHandle,
) -> c_int {
    unsafe { nvim_decor_provider_ptr_get_spell_nav(provider.as_ptr()) }
}

/// Get conceal_line callback ref from provider pointer.
///
/// Migrated from C `nvim_decor_provider_get_conceal_line`.
#[unsafe(export_name = "nvim_decor_provider_get_conceal_line")]
pub unsafe extern "C" fn nvim_decor_provider_get_conceal_line_rs(
    provider: DecorProviderHandle,
) -> c_int {
    unsafe { nvim_decor_provider_ptr_get_conceal_line(provider.as_ptr()) }
}

// =============================================================================
// Rust-internal accessor wrappers
// =============================================================================

/// Get hl_valid for a namespace (Rust-internal helper).
/// Returns -1 if provider doesn't exist.
pub fn get_hl_valid(ns_id: c_int) -> c_int {
    unsafe { nvim_decor_provider_get_hl_valid_rs(ns_id) }
}

/// Get hl_cached for a namespace (Rust-internal helper).
/// Returns false if provider doesn't exist.
pub fn get_hl_cached(ns_id: c_int) -> bool {
    unsafe { nvim_decor_provider_get_hl_cached_rs(ns_id) }
}

/// Set hl_cached for a namespace (Rust-internal helper).
/// Creates provider if force=true.
pub fn set_hl_cached(ns_id: c_int, cached: bool, force: bool) {
    unsafe { nvim_decor_provider_set_hl_cached_rs(ns_id, cached, force) };
}

/// Get hl_valid and set hl_cached=false atomically (Rust-internal helper).
/// Creates provider if needed.
pub fn hl_def_prepare(ns_id: c_int) -> c_int {
    unsafe { nvim_decor_provider_hl_def_prepare_rs(ns_id) }
}

/// Check if namespace has a hl_def callback defined (Rust-internal helper).
pub fn has_hl_def(ns_id: c_int) -> bool {
    unsafe { nvim_decor_provider_has_hl_def_rs(ns_id) }
}

// =============================================================================
// Provider Reference Information
// =============================================================================

/// Information about a provider's LuaRef callback state.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ProviderRefInfo {
    /// Whether redraw_start callback is set.
    pub has_start: bool,
    /// Whether redraw_buf callback is set.
    pub has_buf: bool,
    /// Whether redraw_win callback is set.
    pub has_win: bool,
    /// Whether redraw_line callback is set.
    pub has_line: bool,
    /// Whether redraw_range callback is set.
    pub has_range: bool,
    /// Whether redraw_end callback is set.
    pub has_end: bool,
    /// Whether hl_def callback is set.
    pub has_hl_def: bool,
    /// Whether spell_nav callback is set.
    pub has_spell_nav: bool,
    /// Whether conceal_line callback is set.
    pub has_conceal_line: bool,
}

impl ProviderRefInfo {
    /// Create empty info (no callbacks set).
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            has_start: false,
            has_buf: false,
            has_win: false,
            has_line: false,
            has_range: false,
            has_end: false,
            has_hl_def: false,
            has_spell_nav: false,
            has_conceal_line: false,
        }
    }

    /// Check if any redraw callback is set.
    #[must_use]
    pub const fn has_any_redraw(&self) -> bool {
        self.has_start
            || self.has_buf
            || self.has_win
            || self.has_line
            || self.has_range
            || self.has_end
    }

    /// Check if any callback is set.
    #[must_use]
    pub const fn has_any(&self) -> bool {
        self.has_any_redraw() || self.has_hl_def || self.has_spell_nav || self.has_conceal_line
    }
}

// =============================================================================
// Provider Skip State
// =============================================================================

/// Skip state for a provider in current window.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ProviderSkipState {
    /// Row to skip to.
    pub skip_row: c_int,
    /// Column to skip to.
    pub skip_col: c_int,
}

impl ProviderSkipState {
    /// Create zero skip state.
    #[must_use]
    pub const fn zero() -> Self {
        Self {
            skip_row: 0,
            skip_col: 0,
        }
    }

    /// Check if should skip the given position.
    #[must_use]
    pub const fn should_skip(&self, end_row: c_int, end_col: c_int) -> bool {
        self.skip_row > end_row || (self.skip_row == end_row && self.skip_col >= end_col)
    }

    /// Update skip position from callback result.
    pub fn update(&mut self, row: c_int, col: c_int) {
        self.skip_row = row;
        self.skip_col = col;
    }

    /// Reset skip state.
    pub fn reset(&mut self) {
        self.skip_row = 0;
        self.skip_col = 0;
    }
}

// =============================================================================
// Provider Highlight State
// =============================================================================

/// Highlight state for a provider.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ProviderHlState {
    /// Current hl_valid value.
    pub hl_valid: c_int,
    /// Whether hl is cached.
    pub hl_cached: bool,
}

impl ProviderHlState {
    /// Create new highlight state.
    #[must_use]
    pub const fn new(hl_valid: c_int, hl_cached: bool) -> Self {
        Self {
            hl_valid,
            hl_cached,
        }
    }

    /// Create default state (invalid, not cached).
    #[must_use]
    pub const fn default_state() -> Self {
        Self {
            hl_valid: -1,
            hl_cached: false,
        }
    }

    /// Mark as cached.
    pub fn mark_cached(&mut self) {
        self.hl_cached = true;
    }

    /// Invalidate cache.
    pub fn invalidate(&mut self) {
        self.hl_cached = false;
    }

    /// Check if needs revalidation.
    #[must_use]
    pub const fn needs_revalidation(&self) -> bool {
        !self.hl_cached
    }
}

// =============================================================================
// FFI Exports - Namespace-based Accessors
// =============================================================================

/// FFI: Get hl_valid for namespace.
#[no_mangle]
pub extern "C" fn rs_decor_provider_get_hl_valid(ns_id: c_int) -> c_int {
    get_hl_valid(ns_id)
}

/// FFI: Get hl_cached for namespace.
#[no_mangle]
pub extern "C" fn rs_decor_provider_get_hl_cached(ns_id: c_int) -> bool {
    get_hl_cached(ns_id)
}

/// FFI: Set hl_cached for namespace.
#[no_mangle]
pub extern "C" fn rs_decor_provider_set_hl_cached(ns_id: c_int, cached: bool, force: bool) {
    set_hl_cached(ns_id, cached, force);
}

/// FFI: Prepare hl_def (get hl_valid, set hl_cached=false).
#[no_mangle]
pub extern "C" fn rs_decor_provider_hl_def_prepare(ns_id: c_int) -> c_int {
    hl_def_prepare(ns_id)
}

/// FFI: Check if namespace has hl_def callback.
#[no_mangle]
pub extern "C" fn rs_decor_provider_has_hl_def(ns_id: c_int) -> bool {
    has_hl_def(ns_id)
}

// =============================================================================
// FFI Exports - Handle Operations
// =============================================================================

/// FFI: Check if handle is null.
#[no_mangle]
pub extern "C" fn rs_decor_provider_handle_is_null(handle: DecorProviderHandle) -> bool {
    handle.is_null()
}

// =============================================================================
// FFI Exports - ProviderRefInfo
// =============================================================================

/// FFI: Create empty ProviderRefInfo.
#[no_mangle]
pub extern "C" fn rs_provider_ref_info_empty() -> ProviderRefInfo {
    ProviderRefInfo::empty()
}

/// FFI: Check if any redraw callback is set.
#[no_mangle]
pub extern "C" fn rs_provider_ref_info_has_any_redraw(info: ProviderRefInfo) -> bool {
    info.has_any_redraw()
}

/// FFI: Check if any callback is set.
#[no_mangle]
pub extern "C" fn rs_provider_ref_info_has_any(info: ProviderRefInfo) -> bool {
    info.has_any()
}

/// FFI: Set has_start field.
/// # Safety
/// info must be a valid non-null pointer to ProviderRefInfo.
#[no_mangle]
pub unsafe extern "C" fn rs_provider_ref_info_set_start(info: *mut ProviderRefInfo, has: bool) {
    if !info.is_null() {
        (*info).has_start = has;
    }
}

/// FFI: Set has_buf field.
/// # Safety
/// info must be a valid non-null pointer to ProviderRefInfo.
#[no_mangle]
pub unsafe extern "C" fn rs_provider_ref_info_set_buf(info: *mut ProviderRefInfo, has: bool) {
    if !info.is_null() {
        (*info).has_buf = has;
    }
}

/// FFI: Set has_win field.
/// # Safety
/// info must be a valid non-null pointer to ProviderRefInfo.
#[no_mangle]
pub unsafe extern "C" fn rs_provider_ref_info_set_win(info: *mut ProviderRefInfo, has: bool) {
    if !info.is_null() {
        (*info).has_win = has;
    }
}

/// FFI: Set has_line field.
/// # Safety
/// info must be a valid non-null pointer to ProviderRefInfo.
#[no_mangle]
pub unsafe extern "C" fn rs_provider_ref_info_set_line(info: *mut ProviderRefInfo, has: bool) {
    if !info.is_null() {
        (*info).has_line = has;
    }
}

/// FFI: Set has_range field.
/// # Safety
/// info must be a valid non-null pointer to ProviderRefInfo.
#[no_mangle]
pub unsafe extern "C" fn rs_provider_ref_info_set_range(info: *mut ProviderRefInfo, has: bool) {
    if !info.is_null() {
        (*info).has_range = has;
    }
}

/// FFI: Set has_end field.
/// # Safety
/// info must be a valid non-null pointer to ProviderRefInfo.
#[no_mangle]
pub unsafe extern "C" fn rs_provider_ref_info_set_end(info: *mut ProviderRefInfo, has: bool) {
    if !info.is_null() {
        (*info).has_end = has;
    }
}

// =============================================================================
// FFI Exports - ProviderSkipState
// =============================================================================

/// FFI: Create zero skip state.
#[no_mangle]
pub extern "C" fn rs_provider_skip_state_zero() -> ProviderSkipState {
    ProviderSkipState::zero()
}

/// FFI: Check if should skip position.
#[no_mangle]
pub extern "C" fn rs_provider_skip_state_should_skip(
    state: ProviderSkipState,
    end_row: c_int,
    end_col: c_int,
) -> bool {
    state.should_skip(end_row, end_col)
}

/// FFI: Update skip position.
/// # Safety
/// state must be a valid non-null pointer to ProviderSkipState.
#[no_mangle]
pub unsafe extern "C" fn rs_provider_skip_state_update(
    state: *mut ProviderSkipState,
    row: c_int,
    col: c_int,
) {
    if !state.is_null() {
        (*state).update(row, col);
    }
}

/// FFI: Reset skip state.
/// # Safety
/// state must be a valid non-null pointer to ProviderSkipState.
#[no_mangle]
pub unsafe extern "C" fn rs_provider_skip_state_reset(state: *mut ProviderSkipState) {
    if !state.is_null() {
        (*state).reset();
    }
}

// =============================================================================
// FFI Exports - ProviderHlState
// =============================================================================

/// FFI: Create new highlight state.
#[no_mangle]
pub extern "C" fn rs_provider_hl_state_new(hl_valid: c_int, hl_cached: bool) -> ProviderHlState {
    ProviderHlState::new(hl_valid, hl_cached)
}

/// FFI: Create default highlight state.
#[no_mangle]
pub extern "C" fn rs_provider_hl_state_default() -> ProviderHlState {
    ProviderHlState::default_state()
}

/// FFI: Mark as cached.
/// # Safety
/// state must be a valid non-null pointer to ProviderHlState.
#[no_mangle]
pub unsafe extern "C" fn rs_provider_hl_state_mark_cached(state: *mut ProviderHlState) {
    if !state.is_null() {
        (*state).mark_cached();
    }
}

/// FFI: Invalidate cache.
/// # Safety
/// state must be a valid non-null pointer to ProviderHlState.
#[no_mangle]
pub unsafe extern "C" fn rs_provider_hl_state_invalidate(state: *mut ProviderHlState) {
    if !state.is_null() {
        (*state).invalidate();
    }
}

/// FFI: Check if needs revalidation.
#[no_mangle]
pub extern "C" fn rs_provider_hl_state_needs_revalidation(state: ProviderHlState) -> bool {
    state.needs_revalidation()
}

// =============================================================================
// FFI Exports - Utility Functions
// =============================================================================

/// Check if a LuaRef value represents a valid callback (not NOREF).
#[no_mangle]
pub extern "C" fn rs_decor_provider_ref_is_valid(lua_ref: c_int) -> bool {
    lua_ref != LUA_NOREF
}

/// Check if provider state allows callbacks.
#[no_mangle]
pub extern "C" fn rs_decor_provider_state_allows_callbacks(state: c_int) -> bool {
    state != DECOR_PROVIDER_DISABLED
}

/// Combine state and ref check for callback invocation.
#[no_mangle]
pub extern "C" fn rs_decor_provider_should_invoke(
    state: c_int,
    lua_ref: c_int,
    require_active: bool,
) -> bool {
    let state_ok = if require_active {
        state == crate::constants::DECOR_PROVIDER_ACTIVE
    } else {
        state != DECOR_PROVIDER_DISABLED
    };
    state_ok && lua_ref != LUA_NOREF
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_ref_info() {
        let empty = ProviderRefInfo::empty();
        assert!(!empty.has_any());
        assert!(!empty.has_any_redraw());

        let mut info = ProviderRefInfo::empty();
        info.has_start = true;
        assert!(info.has_any());
        assert!(info.has_any_redraw());

        let mut info2 = ProviderRefInfo::empty();
        info2.has_hl_def = true;
        assert!(info2.has_any());
        assert!(!info2.has_any_redraw());
    }

    #[test]
    fn test_provider_skip_state() {
        let skip = ProviderSkipState::zero();
        assert_eq!(skip.skip_row, 0);
        assert_eq!(skip.skip_col, 0);

        // Position (0, 0) should not skip (5, 5)
        assert!(!skip.should_skip(5, 5));

        let mut skip = ProviderSkipState {
            skip_row: 5,
            skip_col: 5,
        };
        // Skip (5, 5) should skip (5, 5) exactly
        assert!(skip.should_skip(5, 5));
        // Skip (5, 5) should skip (5, 3)
        assert!(skip.should_skip(5, 3));
        // Skip (5, 5) should not skip (5, 10)
        assert!(!skip.should_skip(5, 10));
        // Skip (5, 5) should not skip (10, 0)
        assert!(!skip.should_skip(10, 0));

        skip.update(10, 10);
        assert_eq!(skip.skip_row, 10);
        assert_eq!(skip.skip_col, 10);

        skip.reset();
        assert_eq!(skip.skip_row, 0);
        assert_eq!(skip.skip_col, 0);
    }

    #[test]
    fn test_provider_hl_state() {
        let state = ProviderHlState::default_state();
        assert_eq!(state.hl_valid, -1);
        assert!(!state.hl_cached);
        assert!(state.needs_revalidation());

        let state = ProviderHlState::new(5, true);
        assert_eq!(state.hl_valid, 5);
        assert!(state.hl_cached);
        assert!(!state.needs_revalidation());

        let mut state = ProviderHlState::new(3, false);
        state.mark_cached();
        assert!(state.hl_cached);

        state.invalidate();
        assert!(!state.hl_cached);
    }

    #[test]
    fn test_ref_is_valid() {
        assert!(!rs_decor_provider_ref_is_valid(LUA_NOREF));
        assert!(rs_decor_provider_ref_is_valid(0));
        assert!(rs_decor_provider_ref_is_valid(1));
        assert!(rs_decor_provider_ref_is_valid(100));
    }

    #[test]
    fn test_should_invoke() {
        use crate::constants::{
            DECOR_PROVIDER_ACTIVE, DECOR_PROVIDER_DISABLED, DECOR_PROVIDER_REDRAW_DISABLED,
        };

        // Active state, valid ref, require active = should invoke
        assert!(rs_decor_provider_should_invoke(
            DECOR_PROVIDER_ACTIVE,
            1,
            true
        ));

        // Active state, invalid ref = should not invoke
        assert!(!rs_decor_provider_should_invoke(
            DECOR_PROVIDER_ACTIVE,
            LUA_NOREF,
            true
        ));

        // Disabled state = should not invoke
        assert!(!rs_decor_provider_should_invoke(
            DECOR_PROVIDER_DISABLED,
            1,
            true
        ));
        assert!(!rs_decor_provider_should_invoke(
            DECOR_PROVIDER_DISABLED,
            1,
            false
        ));

        // Redraw disabled, require active = should not invoke
        assert!(!rs_decor_provider_should_invoke(
            DECOR_PROVIDER_REDRAW_DISABLED,
            1,
            true
        ));

        // Redraw disabled, not require active = should invoke
        assert!(rs_decor_provider_should_invoke(
            DECOR_PROVIDER_REDRAW_DISABLED,
            1,
            false
        ));
    }
}
