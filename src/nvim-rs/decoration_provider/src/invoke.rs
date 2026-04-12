//! Provider invocation functions
//!
//! This module implements the iteration loops that invoke decoration provider
//! callbacks during redraw, migrated from `decoration_provider.c`.
//!
//! # Migrated functions
//!
//! Phase 3 (simple loops):
//! - `decor_providers_invoke_spell`
//! - `decor_providers_invoke_conceal_line`
//! - `decor_providers_invoke_buf`
//! - `decor_providers_invoke_end`
//! - `decor_providers_start`
//!
//! Phase 4 (complex loops):
//! - `decor_providers_invoke_win`
//! - `decor_providers_invoke_line`
//! - `decor_providers_invoke_range`
//!
//! # Safety contract
//!
//! The `decor_providers` kvec lives in C. Every call to a `*_invoke_*_c`
//! wrapper may trigger Lua code that registers new providers, reallocating
//! the vector. All iteration is therefore index-based: we never hold a pointer
//! into the vector across a C call.

use std::ffi::c_int;

use crate::constants::{DECOR_PROVIDER_ACTIVE, DECOR_PROVIDER_DISABLED, LUA_NOREF};
use crate::types::WinHandle;

// =============================================================================
// C Accessor Declarations
// =============================================================================

extern "C" {
    // kvec iteration
    fn nvim_decor_providers_size() -> usize;
    fn nvim_decor_providers_get_state(i: usize) -> c_int;
    fn nvim_decor_providers_set_state(i: usize, state: c_int);

    // Index-based callback ref accessors
    fn nvim_decor_providers_get_redraw_start(i: usize) -> c_int;
    fn nvim_decor_providers_get_redraw_buf(i: usize) -> c_int;
    fn nvim_decor_providers_get_redraw_end(i: usize) -> c_int;
    fn nvim_decor_providers_get_spell_nav(i: usize) -> c_int;
    fn nvim_decor_providers_get_conceal_line(i: usize) -> c_int;
    fn nvim_decor_providers_get_redraw_win(i: usize) -> c_int;
    fn nvim_decor_providers_get_redraw_line(i: usize) -> c_int;
    fn nvim_decor_providers_get_redraw_range(i: usize) -> c_int;

    // Win skip state
    fn nvim_decor_providers_get_win_skip_row(i: usize) -> c_int;
    fn nvim_decor_providers_get_win_skip_col(i: usize) -> c_int;
    fn nvim_decor_providers_set_win_skip(i: usize, row: c_int, col: c_int);
    fn nvim_decor_providers_clear_win_skip(i: usize);

    // Globals
    fn nvim_decor_get_display_tick() -> u64;

    // Window accessors
    fn nvim_win_get_handle(wp: WinHandle) -> c_int;
    fn nvim_win_get_buf_handle(wp: WinHandle) -> c_int;
    fn nvim_win_get_buf_marktree_n_keys(wp: WinHandle) -> usize;
    fn nvim_win_get_topline(wp: WinHandle) -> c_int;
    fn nvim_win_get_botline(wp: WinHandle) -> c_int;
    fn nvim_win_get_buf_line_count(wp: WinHandle) -> c_int;

    // Helpers
    fn nvim_decor_check_to_be_deleted();
    fn nvim_decor_state_set_running(val: bool);
    fn nvim_decor_state_assert_clean();
    fn nvim_decor_validate_botline(wp: WinHandle);
    fn nvim_decor_hl_check_ns();

    // Per-callback C invocation wrappers (build the Array in C, call invoke_c)
    fn nvim_decor_cb_spell(
        provider_idx: c_int,
        luaref: c_int,
        win_handle: c_int,
        buf_handle: c_int,
        start_row: c_int,
        start_col: c_int,
        end_row: c_int,
        end_col: c_int,
    );

    fn nvim_decor_cb_conceal_line(
        provider_idx: c_int,
        luaref: c_int,
        win_handle: c_int,
        buf_handle: c_int,
        row: c_int,
    );

    fn nvim_decor_cb_buf(provider_idx: c_int, luaref: c_int, buf_handle: c_int, display_tick: u64);

    fn nvim_decor_cb_end(provider_idx: c_int, luaref: c_int, display_tick: u64);

    fn nvim_decor_cb_start(provider_idx: c_int, luaref: c_int, display_tick: u64) -> bool;

    fn nvim_decor_cb_win(
        provider_idx: c_int,
        luaref: c_int,
        win_handle: c_int,
        buf_handle: c_int,
        topline: c_int,
        botline: c_int,
    ) -> bool;

    fn nvim_decor_cb_line(
        provider_idx: c_int,
        luaref: c_int,
        win_handle: c_int,
        buf_handle: c_int,
        row: c_int,
    ) -> bool;

    fn nvim_decor_provider_invoke_range_c(
        provider_idx: c_int,
        luaref: c_int,
        winid: c_int,
        bufid: c_int,
        start_row: c_int,
        start_col: c_int,
        end_row: c_int,
        end_col: c_int,
    ) -> DecorRangeInvokeResult;
}

/// Matches `DecorRangeInvokeResult` in `decoration_provider.h`.
#[repr(C)]
struct DecorRangeInvokeResult {
    ok: bool,
    stop_win: bool,
    has_skip: bool,
    skip_row: c_int,
    skip_col: c_int,
}

// =============================================================================
// Phase 3: Simple invoke functions
// =============================================================================

/// Invoke spell callbacks on all providers that are not fully disabled.
///
/// Migrated from C `decor_providers_invoke_spell`.
#[unsafe(export_name = "decor_providers_invoke_spell")]
pub unsafe extern "C" fn decor_providers_invoke_spell(
    wp: WinHandle,
    start_row: c_int,
    start_col: c_int,
    end_row: c_int,
    end_col: c_int,
) {
    let len = unsafe { nvim_decor_providers_size() };
    let win_handle = unsafe { nvim_win_get_handle(wp) };
    let buf_handle = unsafe { nvim_win_get_buf_handle(wp) };
    for i in 0..len {
        let state = unsafe { nvim_decor_providers_get_state(i) };
        let spell_nav = unsafe { nvim_decor_providers_get_spell_nav(i) };
        if state != DECOR_PROVIDER_DISABLED && spell_nav != LUA_NOREF {
            unsafe {
                nvim_decor_cb_spell(
                    i as c_int, spell_nav, win_handle, buf_handle, start_row, start_col, end_row,
                    end_col,
                );
            }
        }
    }
}

/// Invoke conceal_line callbacks; returns whether any new marks were placed.
///
/// Migrated from C `decor_providers_invoke_conceal_line`.
#[unsafe(export_name = "decor_providers_invoke_conceal_line")]
pub unsafe extern "C" fn decor_providers_invoke_conceal_line(wp: WinHandle, row: c_int) -> bool {
    let keys_before = unsafe { nvim_win_get_buf_marktree_n_keys(wp) };
    let len = unsafe { nvim_decor_providers_size() };
    let win_handle = unsafe { nvim_win_get_handle(wp) };
    let buf_handle = unsafe { nvim_win_get_buf_handle(wp) };
    for i in 0..len {
        let state = unsafe { nvim_decor_providers_get_state(i) };
        let conceal_line = unsafe { nvim_decor_providers_get_conceal_line(i) };
        if state != DECOR_PROVIDER_DISABLED && conceal_line != LUA_NOREF {
            unsafe {
                nvim_decor_cb_conceal_line(i as c_int, conceal_line, win_handle, buf_handle, row);
            }
        }
    }
    let keys_after = unsafe { nvim_win_get_buf_marktree_n_keys(wp) };
    keys_after > keys_before
}

/// Invoke buf callbacks on all active providers.
///
/// Migrated from C `decor_providers_invoke_buf`.
///
/// # Safety
/// `buf` must be a valid `buf_T*`.
#[unsafe(export_name = "decor_providers_invoke_buf")]
pub unsafe extern "C" fn decor_providers_invoke_buf(buf: crate::types::BufHandle) {
    // Use nvim_buf_get_handle (from buffer_shim.c) to extract the buffer handle.
    extern "C" {
        fn nvim_buf_get_handle(buf: crate::types::BufHandle) -> c_int;
    }
    let len = unsafe { nvim_decor_providers_size() };
    let buf_handle = unsafe { nvim_buf_get_handle(buf) };
    let display_tick = unsafe { nvim_decor_get_display_tick() };
    for i in 0..len {
        let state = unsafe { nvim_decor_providers_get_state(i) };
        let redraw_buf = unsafe { nvim_decor_providers_get_redraw_buf(i) };
        if state == DECOR_PROVIDER_ACTIVE && redraw_buf != LUA_NOREF {
            unsafe { nvim_decor_cb_buf(i as c_int, redraw_buf, buf_handle, display_tick) };
        }
    }
}

/// Invoke end callbacks on all providers that are not fully disabled,
/// then call `decor_check_to_be_deleted`.
///
/// Migrated from C `decor_providers_invoke_end`.
#[unsafe(export_name = "decor_providers_invoke_end")]
pub unsafe extern "C" fn decor_providers_invoke_end() {
    let len = unsafe { nvim_decor_providers_size() };
    let display_tick = unsafe { nvim_decor_get_display_tick() };
    for i in 0..len {
        let state = unsafe { nvim_decor_providers_get_state(i) };
        let redraw_end = unsafe { nvim_decor_providers_get_redraw_end(i) };
        if state != DECOR_PROVIDER_DISABLED && redraw_end != LUA_NOREF {
            unsafe { nvim_decor_cb_end(i as c_int, redraw_end, display_tick) };
        }
    }
    unsafe { nvim_decor_check_to_be_deleted() };
}

/// Invoke start callbacks on all providers; update state based on result.
///
/// Migrated from C `decor_providers_start`.
#[unsafe(export_name = "decor_providers_start")]
pub unsafe extern "C" fn decor_providers_start() {
    let len = unsafe { nvim_decor_providers_size() };
    let display_tick = unsafe { nvim_decor_get_display_tick() };
    for i in 0..len {
        let state = unsafe { nvim_decor_providers_get_state(i) };
        let redraw_start = unsafe { nvim_decor_providers_get_redraw_start(i) };
        if state != DECOR_PROVIDER_DISABLED && redraw_start != LUA_NOREF {
            let active = unsafe { nvim_decor_cb_start(i as c_int, redraw_start, display_tick) };
            let new_state = if active {
                DECOR_PROVIDER_ACTIVE
            } else {
                crate::constants::DECOR_PROVIDER_REDRAW_DISABLED
            };
            unsafe { nvim_decor_providers_set_state(i, new_state) };
        } else if state != DECOR_PROVIDER_DISABLED {
            unsafe { nvim_decor_providers_set_state(i, DECOR_PROVIDER_ACTIVE) };
        }
    }
}

// =============================================================================
// Phase 4: Complex invoke functions
// =============================================================================

/// Invoke win callbacks on all providers; reset WinDisabled state at start
/// of each window.
///
/// Migrated from C `decor_providers_invoke_win`.
#[unsafe(export_name = "decor_providers_invoke_win")]
pub unsafe extern "C" fn decor_providers_invoke_win(wp: WinHandle) {
    unsafe { nvim_decor_state_assert_clean() };

    let len = unsafe { nvim_decor_providers_size() };
    if len > 0 {
        unsafe { nvim_decor_validate_botline(wp) };
    }

    let topline = unsafe { nvim_win_get_topline(wp) };
    let botline_raw = unsafe { nvim_win_get_botline(wp) };
    let buf_line_count = unsafe { nvim_win_get_buf_line_count(wp) };
    // MIN(wp->w_botline, wp->w_buffer->b_ml.ml_line_count)
    let botline = botline_raw.min(buf_line_count);
    let win_handle = unsafe { nvim_win_get_handle(wp) };
    let buf_handle = unsafe { nvim_win_get_buf_handle(wp) };

    for i in 0..len {
        let state = unsafe { nvim_decor_providers_get_state(i) };

        // Reset WinDisabled → Active at start of new window
        let state = if state == crate::constants::DECOR_PROVIDER_WIN_DISABLED {
            unsafe { nvim_decor_providers_set_state(i, DECOR_PROVIDER_ACTIVE) };
            DECOR_PROVIDER_ACTIVE
        } else {
            state
        };

        // Clear win_skip (matches C: p->win_skip_row = 0; p->win_skip_col = 0;)
        unsafe { nvim_decor_providers_clear_win_skip(i) };

        let redraw_win = unsafe { nvim_decor_providers_get_redraw_win(i) };
        if state == DECOR_PROVIDER_ACTIVE && redraw_win != LUA_NOREF {
            // C passes topline-1 and botline-1 (0-based rows)
            let ok = unsafe {
                nvim_decor_cb_win(
                    i as c_int,
                    redraw_win,
                    win_handle,
                    buf_handle,
                    topline - 1,
                    botline - 1,
                )
            };
            if !ok {
                unsafe {
                    nvim_decor_providers_set_state(i, crate::constants::DECOR_PROVIDER_WIN_DISABLED)
                };
            }
        }
    }
}

/// Invoke line callbacks on all active providers; set running flag.
///
/// Migrated from C `decor_providers_invoke_line`.
#[unsafe(export_name = "decor_providers_invoke_line")]
pub unsafe extern "C" fn decor_providers_invoke_line(wp: WinHandle, row: c_int) {
    unsafe { nvim_decor_state_set_running(true) };
    let len = unsafe { nvim_decor_providers_size() };
    let win_handle = unsafe { nvim_win_get_handle(wp) };
    let buf_handle = unsafe { nvim_win_get_buf_handle(wp) };
    for i in 0..len {
        let state = unsafe { nvim_decor_providers_get_state(i) };
        let redraw_line = unsafe { nvim_decor_providers_get_redraw_line(i) };
        if state == DECOR_PROVIDER_ACTIVE && redraw_line != LUA_NOREF {
            let ok =
                unsafe { nvim_decor_cb_line(i as c_int, redraw_line, win_handle, buf_handle, row) };
            if !ok {
                // return false or error: skip rest of this window
                unsafe {
                    nvim_decor_providers_set_state(i, crate::constants::DECOR_PROVIDER_WIN_DISABLED)
                };
            }
            unsafe { nvim_decor_hl_check_ns() };
        }
    }
    unsafe { nvim_decor_state_set_running(false) };
}

/// Invoke range callbacks on all active providers with win_skip optimization.
///
/// Migrated from C `decor_providers_invoke_range`.
#[unsafe(export_name = "decor_providers_invoke_range")]
pub unsafe extern "C" fn decor_providers_invoke_range(
    wp: WinHandle,
    start_row: c_int,
    start_col: c_int,
    end_row: c_int,
    end_col: c_int,
) {
    unsafe { nvim_decor_state_set_running(true) };
    let len = unsafe { nvim_decor_providers_size() };
    let win_handle = unsafe { nvim_win_get_handle(wp) };
    let buf_handle = unsafe { nvim_win_get_buf_handle(wp) };
    for i in 0..len {
        let state = unsafe { nvim_decor_providers_get_state(i) };
        let redraw_range = unsafe { nvim_decor_providers_get_redraw_range(i) };
        if state == DECOR_PROVIDER_ACTIVE && redraw_range != LUA_NOREF {
            let skip_row = unsafe { nvim_decor_providers_get_win_skip_row(i) };
            let skip_col = unsafe { nvim_decor_providers_get_win_skip_col(i) };
            if skip_row > end_row || (skip_row == end_row && skip_col >= end_col) {
                continue;
            }

            let result = unsafe {
                nvim_decor_provider_invoke_range_c(
                    i as c_int,
                    redraw_range,
                    win_handle,
                    buf_handle,
                    start_row,
                    start_col,
                    end_row,
                    end_col,
                )
            };

            if result.stop_win {
                unsafe {
                    nvim_decor_providers_set_state(i, crate::constants::DECOR_PROVIDER_WIN_DISABLED)
                };
            } else if result.has_skip {
                unsafe { nvim_decor_providers_set_win_skip(i, result.skip_row, result.skip_col) };
            }

            unsafe { nvim_decor_hl_check_ns() };
        }
    }
    unsafe { nvim_decor_state_set_running(false) };
}
