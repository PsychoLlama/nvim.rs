//! Decoration provider callback infrastructure
//!
//! This module provides FFI functions for invoking decoration provider callbacks
//! and building argument lists for each callback type.
//!
//! # Callback Types
//!
//! Decoration providers support several callback types:
//! - `start`: Called at the beginning of a redraw cycle
//! - `buf`: Called when a buffer is about to be redrawn
//! - `win`: Called when a window is about to be redrawn
//! - `line`: Called for each line being drawn
//! - `range`: Called for a range of lines (used for virt_lines, conceal, etc.)
//! - `end`: Called at the end of a redraw cycle
//! - `spell`: Called for spell checking navigation
//! - `conceal_line`: Called for concealed line rendering
//!
//! # Argument Building
//!
//! Each callback type has specific arguments that are passed to the Lua callback:
//! - `start`: no arguments (returns bool for active state)
//! - `buf`: (bufnr)
//! - `win`: (winid, bufnr, toprow, botrow)
//! - `line`: (winid, bufnr, row)
//! - `range`: (winid, bufnr, start_row, start_col, end_row, end_col)
//! - `end`: no arguments
//! - `spell`: (winid, start_row, start_col, end_row, end_col)
//! - `conceal_line`: (winid, row)

use std::ffi::c_int;

use crate::types::{BufHandle, DecorProviderHandle, WinHandle};

// =============================================================================
// C accessor declarations for callback invocation
// =============================================================================

// Callback reference accessor functions are now implemented in accessors.rs
// (Phase 5 migration). Use the accessors module functions directly.
use crate::accessors::{
    nvim_decor_provider_get_conceal_line_rs as nvim_decor_provider_get_conceal_line,
    nvim_decor_provider_get_redraw_buf_rs as nvim_decor_provider_get_redraw_buf,
    nvim_decor_provider_get_redraw_end_rs as nvim_decor_provider_get_redraw_end,
    nvim_decor_provider_get_redraw_line_rs as nvim_decor_provider_get_redraw_line,
    nvim_decor_provider_get_redraw_range_rs as nvim_decor_provider_get_redraw_range,
    nvim_decor_provider_get_redraw_start_rs as nvim_decor_provider_get_redraw_start,
    nvim_decor_provider_get_redraw_win_rs as nvim_decor_provider_get_redraw_win,
    nvim_decor_provider_get_spell_nav_rs as nvim_decor_provider_get_spell_nav,
};

extern "C" {
    // Handle accessors for argument building
    fn nvim_win_get_handle(win: WinHandle) -> c_int;
    fn nvim_buf_get_handle(buf: BufHandle) -> c_int;
}

// =============================================================================
// Callback argument structures
// =============================================================================

/// Arguments for the "buf" callback
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BufCallbackArgs {
    pub bufnr: c_int,
}

/// Arguments for the "win" callback
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WinCallbackArgs {
    pub winid: c_int,
    pub bufnr: c_int,
    pub toprow: c_int,
    pub botrow: c_int,
}

/// Arguments for the "line" callback
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LineCallbackArgs {
    pub winid: c_int,
    pub bufnr: c_int,
    pub row: c_int,
}

/// Arguments for the "range" callback
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RangeCallbackArgs {
    pub winid: c_int,
    pub bufnr: c_int,
    pub start_row: c_int,
    pub start_col: c_int,
    pub end_row: c_int,
    pub end_col: c_int,
}

/// Arguments for the "spell" callback
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SpellCallbackArgs {
    pub winid: c_int,
    pub start_row: c_int,
    pub start_col: c_int,
    pub end_row: c_int,
    pub end_col: c_int,
}

/// Arguments for the "conceal_line" callback
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ConcealLineCallbackArgs {
    pub winid: c_int,
    pub row: c_int,
}

// =============================================================================
// Callback check functions
// =============================================================================

/// Check if a provider has a valid redraw_start callback
#[no_mangle]
pub extern "C" fn rs_decor_provider_has_start_callback(provider: DecorProviderHandle) -> bool {
    let ref_id = unsafe { nvim_decor_provider_get_redraw_start(provider) };
    ref_id != crate::constants::LUA_NOREF
}

/// Check if a provider has a valid redraw_buf callback
#[no_mangle]
pub extern "C" fn rs_decor_provider_has_buf_callback(provider: DecorProviderHandle) -> bool {
    let ref_id = unsafe { nvim_decor_provider_get_redraw_buf(provider) };
    ref_id != crate::constants::LUA_NOREF
}

/// Check if a provider has a valid redraw_win callback
#[no_mangle]
pub extern "C" fn rs_decor_provider_has_win_callback(provider: DecorProviderHandle) -> bool {
    let ref_id = unsafe { nvim_decor_provider_get_redraw_win(provider) };
    ref_id != crate::constants::LUA_NOREF
}

/// Check if a provider has a valid redraw_line callback
#[no_mangle]
pub extern "C" fn rs_decor_provider_has_line_callback(provider: DecorProviderHandle) -> bool {
    let ref_id = unsafe { nvim_decor_provider_get_redraw_line(provider) };
    ref_id != crate::constants::LUA_NOREF
}

/// Check if a provider has a valid redraw_range callback
#[no_mangle]
pub extern "C" fn rs_decor_provider_has_range_callback(provider: DecorProviderHandle) -> bool {
    let ref_id = unsafe { nvim_decor_provider_get_redraw_range(provider) };
    ref_id != crate::constants::LUA_NOREF
}

/// Check if a provider has a valid redraw_end callback
#[no_mangle]
pub extern "C" fn rs_decor_provider_has_end_callback(provider: DecorProviderHandle) -> bool {
    let ref_id = unsafe { nvim_decor_provider_get_redraw_end(provider) };
    ref_id != crate::constants::LUA_NOREF
}

/// Check if a provider has a valid spell_nav callback
#[no_mangle]
pub extern "C" fn rs_decor_provider_has_spell_callback(provider: DecorProviderHandle) -> bool {
    let ref_id = unsafe { nvim_decor_provider_get_spell_nav(provider) };
    ref_id != crate::constants::LUA_NOREF
}

/// Check if a provider has a valid conceal_line callback
#[no_mangle]
pub extern "C" fn rs_decor_provider_has_conceal_callback(provider: DecorProviderHandle) -> bool {
    let ref_id = unsafe { nvim_decor_provider_get_conceal_line(provider) };
    ref_id != crate::constants::LUA_NOREF
}

// =============================================================================
// Argument building functions
// =============================================================================

/// Build arguments for a "buf" callback
#[no_mangle]
pub extern "C" fn rs_build_buf_callback_args(buf: BufHandle) -> BufCallbackArgs {
    BufCallbackArgs {
        bufnr: unsafe { nvim_buf_get_handle(buf) },
    }
}

/// Build arguments for a "win" callback
#[no_mangle]
pub extern "C" fn rs_build_win_callback_args(
    win: WinHandle,
    buf: BufHandle,
    toprow: c_int,
    botrow: c_int,
) -> WinCallbackArgs {
    WinCallbackArgs {
        winid: unsafe { nvim_win_get_handle(win) },
        bufnr: unsafe { nvim_buf_get_handle(buf) },
        toprow,
        botrow,
    }
}

/// Build arguments for a "line" callback
#[no_mangle]
pub extern "C" fn rs_build_line_callback_args(
    win: WinHandle,
    buf: BufHandle,
    row: c_int,
) -> LineCallbackArgs {
    LineCallbackArgs {
        winid: unsafe { nvim_win_get_handle(win) },
        bufnr: unsafe { nvim_buf_get_handle(buf) },
        row,
    }
}

/// Build arguments for a "range" callback
#[no_mangle]
pub extern "C" fn rs_build_range_callback_args(
    win: WinHandle,
    buf: BufHandle,
    start_row: c_int,
    start_col: c_int,
    end_row: c_int,
    end_col: c_int,
) -> RangeCallbackArgs {
    RangeCallbackArgs {
        winid: unsafe { nvim_win_get_handle(win) },
        bufnr: unsafe { nvim_buf_get_handle(buf) },
        start_row,
        start_col,
        end_row,
        end_col,
    }
}

/// Build arguments for a "spell" callback
#[no_mangle]
pub extern "C" fn rs_build_spell_callback_args(
    win: WinHandle,
    start_row: c_int,
    start_col: c_int,
    end_row: c_int,
    end_col: c_int,
) -> SpellCallbackArgs {
    SpellCallbackArgs {
        winid: unsafe { nvim_win_get_handle(win) },
        start_row,
        start_col,
        end_row,
        end_col,
    }
}

/// Build arguments for a "conceal_line" callback
#[no_mangle]
pub extern "C" fn rs_build_conceal_line_callback_args(
    win: WinHandle,
    row: c_int,
) -> ConcealLineCallbackArgs {
    ConcealLineCallbackArgs {
        winid: unsafe { nvim_win_get_handle(win) },
        row,
    }
}

// =============================================================================
// Callback invocation helpers
// =============================================================================

/// Check if any callback should be invoked for this provider
#[no_mangle]
pub extern "C" fn rs_decor_provider_has_any_callback(provider: DecorProviderHandle) -> bool {
    rs_decor_provider_has_start_callback(provider)
        || rs_decor_provider_has_buf_callback(provider)
        || rs_decor_provider_has_win_callback(provider)
        || rs_decor_provider_has_line_callback(provider)
        || rs_decor_provider_has_range_callback(provider)
        || rs_decor_provider_has_end_callback(provider)
}

/// Count how many callbacks a provider has registered
#[no_mangle]
pub extern "C" fn rs_decor_provider_callback_count(provider: DecorProviderHandle) -> c_int {
    let mut count = 0;
    if rs_decor_provider_has_start_callback(provider) {
        count += 1;
    }
    if rs_decor_provider_has_buf_callback(provider) {
        count += 1;
    }
    if rs_decor_provider_has_win_callback(provider) {
        count += 1;
    }
    if rs_decor_provider_has_line_callback(provider) {
        count += 1;
    }
    if rs_decor_provider_has_range_callback(provider) {
        count += 1;
    }
    if rs_decor_provider_has_end_callback(provider) {
        count += 1;
    }
    if rs_decor_provider_has_spell_callback(provider) {
        count += 1;
    }
    if rs_decor_provider_has_conceal_callback(provider) {
        count += 1;
    }
    count
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buf_callback_args() {
        // Test that the struct is properly sized for C interop
        assert_eq!(
            std::mem::size_of::<BufCallbackArgs>(),
            std::mem::size_of::<c_int>()
        );
    }

    #[test]
    fn test_win_callback_args() {
        // Test that the struct layout is correct
        assert_eq!(
            std::mem::size_of::<WinCallbackArgs>(),
            std::mem::size_of::<c_int>() * 4
        );
    }

    #[test]
    fn test_line_callback_args() {
        // Test that the struct layout is correct
        assert_eq!(
            std::mem::size_of::<LineCallbackArgs>(),
            std::mem::size_of::<c_int>() * 3
        );
    }

    #[test]
    fn test_range_callback_args() {
        // Test that the struct layout is correct
        assert_eq!(
            std::mem::size_of::<RangeCallbackArgs>(),
            std::mem::size_of::<c_int>() * 6
        );
    }

    #[test]
    fn test_spell_callback_args() {
        // Test that the struct layout is correct
        assert_eq!(
            std::mem::size_of::<SpellCallbackArgs>(),
            std::mem::size_of::<c_int>() * 5
        );
    }

    #[test]
    fn test_conceal_line_callback_args() {
        // Test that the struct layout is correct
        assert_eq!(
            std::mem::size_of::<ConcealLineCallbackArgs>(),
            std::mem::size_of::<c_int>() * 2
        );
    }
}
