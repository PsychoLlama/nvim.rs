//! Buffer lifecycle management helpers
//!
//! This module provides Rust implementations for buffer lifecycle operations
//! including creation validation, cleanup preparation, and state transitions.

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::missing_const_for_fn)]
#![allow(dead_code)]

use nvim_ex_cmds_types::ExArg;
use std::ffi::{c_char, c_int, c_uint, c_void};

use crate::{
    buf_struct::{buf_mut, buf_ref},
    errors, win_mut_raw, win_ref_raw, BufHandle,
};

// =============================================================================
// External C Statics
// =============================================================================

extern "C" {
    static mut jop_flags: c_uint;
    static mut updating_screen: bool;
}

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    fn nvim_get_firstbuf() -> BufHandle;
    static mut lastbuf: *mut std::ffi::c_void;
    fn nvim_get_curbuf() -> BufHandle;
    fn nvim_get_curwin() -> *mut c_void;
    // Jump list accessors (implemented in mark.c)
    fn nvim_mark_win_get_jumplistlen(win: *mut c_void) -> c_int;
    fn nvim_mark_win_set_jumplistidx(win: *mut c_void, idx: c_int);
    fn nvim_mark_win_get_jumplistidx(win: *mut c_void) -> c_int;
    fn nvim_mark_win_get_jumplist_fnum(win: *mut c_void, idx: c_int) -> c_int;
    fn mark_jumplist_forget_file(win: *mut c_void, fnum: c_int);

    // au_new_curbuf accessors (implemented in ex_cmds_shim.c)
    fn nvim_ecmd_au_new_curbuf_valid() -> c_int;
    fn nvim_ecmd_au_new_curbuf_get_buf() -> BufHandle;

    // bt_quickfix (implemented in Rust, re-exported)
    fn rs_bt_quickfix(buf: BufHandle) -> bool;

    // buflist_findnr (implemented in Rust, re-exported)
    fn rs_buflist_findnr(nr: c_int) -> BufHandle;

    // Validation helpers for do_buffer_ext (implemented in ex_cmds_shim.c)
    fn check_can_set_curbuf_forceit(forceit: c_int) -> bool;
    fn nvim_ecmd_emsg_closing_buffer();

    // can_unload_buffer accessors
    /// Get the first window in the current tab (`firstwin`).
    fn nvim_get_firstwin() -> *mut c_void;
    /// Get `wp->w_next` for current-tab iteration.
    fn nvim_win_get_next_in_tab(wp: *mut c_void) -> *mut c_void;

    // buf_ensure_loaded accessor (compound: aucmd_prepbuf + open_buffer + aucmd_restbuf)
    fn nvim_buf_aucmd_open_buffer(buf: BufHandle) -> c_int;

    // buf_open_scratch accessors
    fn apply_autocmds(
        event: c_int,
        fname: *const c_char,
        fname_io: *const c_char,
        force: bool,
        buf: BufHandle,
    ) -> bool;
    fn nvim_set_buf_opts_scratch();
}

// Event constants from auevents_enum.generated.h
const EVENT_BUFFILEPRE: c_int = 5;
const EVENT_BUFFILEPOST: c_int = 4;

// kOptJopFlagClean flag value (from option_vars.generated.h)
const K_OPT_JOP_FLAG_CLEAN: c_uint = 0x04;

// do_buffer_ext start values (from buffer.h dobuf_start_values)
const DOBUF_CURRENT: c_int = 0;
const DOBUF_FIRST: c_int = 1;
const DOBUF_LAST: c_int = 2;
const DOBUF_MOD: c_int = 3;

// do_buffer_ext flags (from buffer.h dobuf_flags_value)
const DOBUF_FORCEIT: c_int = 1;
const DOBUF_SKIPHELP: c_int = 4;

// do_buffer_ext action values (from buffer.h dobuf_action_values)
const DOBUF_GOTO: c_int = 0;
const DOBUF_SPLIT: c_int = 1;

// Direction constants (from vim_defs.h)
const FORWARD: c_int = 1;
// BACKWARD = -1 but not used directly (we compare != FORWARD)

// =============================================================================
// Buffer Flags (from buffer_defs.h)
// =============================================================================

/// Buffer flags matching `buffer_defs.h`
pub mod buf_flags {
    use std::ffi::c_int;

    /// Buffer needs read-only check
    pub const BF_CHECK_RO: c_int = 0x02;
    /// Buffer was never loaded
    pub const BF_NEVERLOADED: c_int = 0x04;
    /// Buffer has read error
    pub const BF_READERR: c_int = 0x40;
    /// Dummy buffer for internal use
    pub const BF_DUMMY: c_int = 0x80;
}

/// Buffer action types for `close_buffer`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BufferAction {
    /// Don't unload the buffer
    #[default]
    None = 0,
    /// Unload the buffer
    Unload = 1,
    /// Delete the buffer
    Delete = 2,
    /// Wipe the buffer completely
    Wipe = 3,
}

impl BufferAction {
    /// Create from raw integer (matching `DOBUF_*` constants)
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Unload,
            2 => Self::Delete,
            3 => Self::Wipe,
            _ => Self::None,
        }
    }

    /// Convert to raw integer
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Check if this action unloads the buffer
    #[must_use]
    pub const fn unloads(self) -> bool {
        !matches!(self, Self::None)
    }

    /// Check if this action deletes the buffer
    #[must_use]
    pub const fn deletes(self) -> bool {
        matches!(self, Self::Delete | Self::Wipe)
    }

    /// Check if this action wipes the buffer
    #[must_use]
    pub const fn wipes(self) -> bool {
        matches!(self, Self::Wipe)
    }
}

// =============================================================================
// Buffer Lifecycle State
// =============================================================================

/// Current lifecycle state of a buffer
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LifecycleState {
    /// Buffer has never been loaded (`BF_NEVERLOADED` set)
    #[default]
    NeverLoaded = 0,
    /// Buffer is not loaded (`ml_mfp` is NULL)
    NotLoaded = 1,
    /// Buffer is loaded but not displayed (`b_nwindows` == 0)
    Hidden = 2,
    /// Buffer is loaded and displayed in a window
    Normal = 3,
}

impl LifecycleState {
    /// Check if buffer is loaded (in memory)
    #[must_use]
    pub const fn is_loaded(self) -> bool {
        matches!(self, Self::Hidden | Self::Normal)
    }

    /// Check if buffer is visible in a window
    #[must_use]
    pub const fn is_visible(self) -> bool {
        matches!(self, Self::Normal)
    }

    /// Convert to raw integer
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Create from raw integer
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::NotLoaded,
            2 => Self::Hidden,
            3 => Self::Normal,
            _ => Self::NeverLoaded,
        }
    }
}

/// Get the lifecycle state of a buffer.
///
/// # Safety
///
/// Calls external C functions. `buf` must be valid.
#[must_use]
pub unsafe fn get_lifecycle_state(buf: BufHandle) -> LifecycleState {
    if buf.is_null() {
        return LifecycleState::NeverLoaded;
    }

    let b = buf_ref(buf);
    if (b.b_flags & buf_flags::BF_NEVERLOADED) != 0 {
        return LifecycleState::NeverLoaded;
    }

    if b.ml_mfp_is_null() {
        return LifecycleState::NotLoaded;
    }

    if b.b_nwindows == 0 {
        LifecycleState::Hidden
    } else {
        LifecycleState::Normal
    }
}

// =============================================================================
// Buffer Close Preparation
// =============================================================================

/// Result of checking if a buffer can be unloaded
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct UnloadCheck {
    /// Whether the buffer can be unloaded
    pub can_unload: bool,
    /// Whether the buffer is locked
    pub is_locked: bool,
    /// Whether the buffer is locked for split
    pub is_locked_split: bool,
    /// Whether the buffer is in use by a terminal
    pub has_terminal: bool,
    /// Number of windows displaying this buffer
    pub nwindows: c_int,
}

/// Check if a buffer can be unloaded.
///
/// # Safety
///
/// Calls external C functions. `buf` must be valid.
#[must_use]
pub unsafe fn check_can_unload(buf: BufHandle) -> UnloadCheck {
    if buf.is_null() {
        return UnloadCheck::default();
    }

    let b = buf_ref(buf);
    let is_locked = b.b_locked > 0;
    let is_locked_split = b.b_locked_split > 0;
    let has_terminal = !b.terminal.is_null();
    let nwindows = b.b_nwindows;

    // Buffer can be unloaded if not locked
    let can_unload = !is_locked;

    UnloadCheck {
        can_unload,
        is_locked,
        is_locked_split,
        has_terminal,
        nwindows,
    }
}

/// Determine the effective action for closing a buffer based on bufhidden.
///
/// # Safety
///
/// Calls external C functions. `buf` must be valid.
#[must_use]
pub unsafe fn effective_close_action(buf: BufHandle, requested: BufferAction) -> BufferAction {
    if buf.is_null() {
        return requested;
    }

    // Terminal buffers can only be wiped
    if !buf_ref(buf).terminal.is_null()
        && (requested.unloads() || requested.deletes() || requested.wipes())
    {
        return BufferAction::Wipe;
    }

    // Check bufhidden option
    let bh = buf_ref(buf).bufhidden_char0();
    match bh {
        b'd' => {
            // bufhidden=delete
            BufferAction::Delete
        }
        b'w' => {
            // bufhidden=wipe
            BufferAction::Wipe
        }
        b'u' => {
            // bufhidden=unload
            if requested.deletes() || requested.wipes() {
                requested
            } else {
                BufferAction::Unload
            }
        }
        _ => requested,
    }
}

// =============================================================================
// Buffer List Operations
// =============================================================================

/// Check if buffer has a file name.
///
/// # Safety
///
/// Calls external C functions. `buf` must be valid.
#[must_use]
pub unsafe fn has_filename(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    !buf_ref(buf).b_ffname.is_null()
}

/// Check if buffer is modified.
///
/// # Safety
///
/// Calls external C functions. `buf` must be valid.
#[must_use]
pub unsafe fn is_modified(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    buf_ref(buf).b_changed != 0
}

/// Check if buffer is a dummy buffer.
///
/// # Safety
///
/// Calls external C functions. `buf` must be valid.
#[must_use]
pub unsafe fn is_dummy(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    (buf_ref(buf).b_flags & buf_flags::BF_DUMMY) != 0
}

/// Check if buffer was never loaded.
///
/// # Safety
///
/// Calls external C functions. `buf` must be valid.
#[must_use]
pub unsafe fn is_never_loaded(buf: BufHandle) -> bool {
    if buf.is_null() {
        return true;
    }
    (buf_ref(buf).b_flags & buf_flags::BF_NEVERLOADED) != 0
}

/// Check if buffer is last in window (nwindows == 1).
///
/// # Safety
///
/// Calls external C functions. `buf` must be valid.
#[must_use]
pub unsafe fn is_last_in_window(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    buf_ref(buf).b_nwindows == 1
}

/// Check if this is the current buffer.
///
/// # Safety
///
/// Calls external C functions. `buf` must be valid.
#[must_use]
pub unsafe fn is_curbuf(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    buf == nvim_get_curbuf()
}

// =============================================================================
// Buffer Position in List
// =============================================================================

/// Information about a buffer's position for lifecycle operations.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct LifecyclePosition {
    /// File number
    pub fnum: c_int,
    /// Lifecycle state
    pub state: LifecycleState,
    /// Number of windows
    pub nwindows: c_int,
    /// Whether buffer has filename
    pub has_filename: bool,
    /// Whether buffer is modified
    pub is_modified: bool,
    /// Whether buffer is current
    pub is_current: bool,
    /// Whether buffer is dummy
    pub is_dummy: bool,
}

/// Get lifecycle position info for a buffer.
///
/// # Safety
///
/// Calls external C functions. `buf` must be valid.
#[must_use]
pub unsafe fn get_lifecycle_position(buf: BufHandle) -> LifecyclePosition {
    if buf.is_null() {
        return LifecyclePosition::default();
    }

    let b = buf_ref(buf);
    LifecyclePosition {
        fnum: b.handle,
        state: get_lifecycle_state(buf),
        nwindows: b.b_nwindows,
        has_filename: has_filename(buf),
        is_modified: is_modified(buf),
        is_current: is_curbuf(buf),
        is_dummy: is_dummy(buf),
    }
}

// =============================================================================
// Find Next Buffer When Deleting Current Buffer
// =============================================================================

/// Find the best buffer to switch to when deleting `curbuf` (`buf_fnum`).
///
/// Mirrors the search logic in `do_buffer_ext` lines 2280-2386:
/// 1. Use `au_new_curbuf` if valid.
/// 2. Search the jump list (most-recently-visited loaded buffer).
/// 3. Walk forward then backward through the buffer list.
/// 4. Fall back to any listed non-quickfix buffer.
/// 5. Last resort: adjacent buffer.
///
/// Returns NULL if no suitable buffer was found (caller must call `empty_curbuf`).
///
/// `update_jumplist` is set to 0 when the jump list search found a buffer via
/// `kOptJopFlagClean` (telling `set_curbuf` not to add another jump entry).
///
/// # Safety
///
/// Must be called on the main thread with valid Neovim state.
#[allow(clippy::too_many_lines)]
pub unsafe fn find_buffer_for_delete(buf_fnum: c_int, update_jumplist: *mut c_int) -> BufHandle {
    let curbuf = nvim_get_curbuf();
    let curwin = nvim_get_curwin();
    let jop_clean = (jop_flags & K_OPT_JOP_FLAG_CLEAN) != 0;

    // 1. Use au_new_curbuf if set and still valid.
    if !nvim_ecmd_au_new_curbuf_get_buf().is_null() && nvim_ecmd_au_new_curbuf_valid() != 0 {
        return nvim_ecmd_au_new_curbuf_get_buf();
    }

    let mut result: BufHandle = BufHandle(std::ptr::null_mut());
    let mut bp: BufHandle = BufHandle(std::ptr::null_mut()); // fallback: first unloaded candidate

    // 2. Search the jump list for a recently-visited loaded buffer.
    let mut jumplistlen = nvim_mark_win_get_jumplistlen(curwin);
    if jumplistlen > 0 {
        if jop_clean {
            mark_jumplist_forget_file(curwin, buf_fnum);
            jumplistlen = nvim_mark_win_get_jumplistlen(curwin);
        }

        if jumplistlen > 0 {
            let jumplistidx = nvim_mark_win_get_jumplistidx(curwin);

            let mut jumpidx = if jop_clean {
                // If idx == len, the current pos was not yet added; go to last entry.
                if jumplistidx == jumplistlen {
                    let new_idx = jumplistlen - 1;
                    nvim_mark_win_set_jumplistidx(curwin, new_idx);
                    new_idx
                } else {
                    jumplistidx
                }
            } else {
                let idx = jumplistidx - 1;
                if idx < 0 {
                    jumplistlen - 1
                } else {
                    idx
                }
            };

            let forward = jumpidx;
            loop {
                if !jop_clean && jumpidx == jumplistidx {
                    break;
                }

                let fnum = nvim_mark_win_get_jumplist_fnum(curwin, jumpidx);
                let candidate = rs_buflist_findnr(fnum);

                let mut found = false;
                if !candidate.is_null() {
                    // Skip curbuf, unlisted, and quickfix buffers.
                    if candidate == curbuf
                        || buf_ref(candidate).b_p_bl == 0
                        || rs_bt_quickfix(candidate)
                    {
                        // Not suitable
                    } else if buf_ref(candidate).ml_mfp_is_null() {
                        // Unloaded: remember as fallback
                        if bp.is_null() {
                            bp = candidate;
                        }
                    } else {
                        // Found a valid loaded buffer
                        result = candidate;
                        if jop_clean {
                            nvim_mark_win_set_jumplistidx(curwin, jumpidx);
                            if !update_jumplist.is_null() {
                                *update_jumplist = 0;
                            }
                        }
                        found = true;
                    }
                }

                if found {
                    return result;
                }

                // Advance to older jump list entry
                if jumpidx == 0 && jumplistidx == jumplistlen {
                    break;
                }
                jumpidx -= 1;
                if jumpidx < 0 {
                    jumpidx = jumplistlen - 1;
                }
                if jumpidx == forward {
                    break; // list exhausted
                }
            }
        }
    }

    // 3. Walk forward then backward through buffer list.
    if result.is_null() {
        let mut forward = true;
        let mut buf = BufHandle::from_ptr(buf_ref(curbuf).b_next);
        loop {
            if buf.is_null() {
                if !forward {
                    break; // tried both directions
                }
                buf = BufHandle::from_ptr(buf_ref(curbuf).b_prev);
                forward = false;
                continue;
            }
            // Prefer same help-buffer type, listed, non-quickfix
            if buf_ref(buf).b_help == buf_ref(curbuf).b_help
                && buf_ref(buf).b_p_bl != 0
                && !rs_bt_quickfix(buf)
            {
                if !buf_ref(buf).ml_mfp_is_null() {
                    result = buf;
                    break;
                }
                if bp.is_null() {
                    bp = buf;
                }
            }
            buf = if forward {
                BufHandle::from_ptr(buf_ref(buf).b_next)
            } else {
                BufHandle::from_ptr(buf_ref(buf).b_prev)
            };
        }
    }

    // 4. Fall back to any unloaded buffer found during search.
    if result.is_null() {
        result = bp;
    }

    // 5. Fall back to any listed non-quickfix buffer.
    if result.is_null() {
        let mut buf = nvim_get_firstbuf();
        while !buf.is_null() {
            if buf_ref(buf).b_p_bl != 0 && buf != curbuf && !rs_bt_quickfix(buf) {
                result = buf;
                break;
            }
            buf = BufHandle::from_ptr(buf_ref(buf).b_next);
        }
    }

    // 6. Last resort: adjacent buffer (even if quickfix).
    if result.is_null() {
        let next = BufHandle::from_ptr(buf_ref(curbuf).b_next);
        let prev = BufHandle::from_ptr(buf_ref(curbuf).b_prev);
        result = if !next.is_null() && !rs_bt_quickfix(next) {
            next
        } else if !prev.is_null() && !rs_bt_quickfix(prev) {
            prev
        } else {
            // absolute last resort: any adjacent
            if next.is_null() {
                prev
            } else {
                next
            }
        };
    }

    result
}

/// Advance `buf` one step forward or backward, wrapping at list ends.
unsafe fn nav_step(buf: BufHandle, dir: c_int) -> BufHandle {
    if dir == FORWARD {
        let next = BufHandle::from_ptr(buf_ref(buf).b_next);
        if next.is_null() {
            nvim_get_firstbuf()
        } else {
            next
        }
    } else {
        let prev = BufHandle::from_ptr(buf_ref(buf).b_prev);
        if prev.is_null() {
            BufHandle::from_ptr(lastbuf)
        } else {
            prev
        }
    }
}

/// Find and validate the target buffer for a `do_buffer_ext` call.
///
/// Implements the navigation and pre-action validation logic from `do_buffer_ext`.
/// Emits the appropriate error message on failure and returns a null handle.
/// Returns the target buffer handle on success.
///
/// # Arguments
/// - `action`: `DOBUF_GOTO`, `DOBUF_SPLIT`, `DOBUF_UNLOAD`, `DOBUF_DEL`, or `DOBUF_WIPE`
/// - `start`: `DOBUF_FIRST`, `DOBUF_LAST`, `DOBUF_MOD`, or `DOBUF_CURRENT`
/// - `dir`: `FORWARD` (1) or `BACKWARD` (-1)
/// - `count`: buffer number (for `DOBUF_FIRST`) or number of steps
/// - `flags`: `DOBUF_FORCEIT` and/or `DOBUF_SKIPHELP`
/// - `unload`: whether the action unloads the buffer
///
/// # Safety
/// Caller must ensure C runtime is live and globals are valid.
unsafe fn find_and_validate_buffer(
    action: c_int,
    start: c_int,
    dir: c_int,
    count: c_int,
    flags: c_int,
    unload: bool,
) -> BufHandle {
    let null = BufHandle(std::ptr::null_mut());
    // Determine starting buffer.
    let mut buf: BufHandle = match start {
        DOBUF_FIRST => nvim_get_firstbuf(),
        DOBUF_LAST => BufHandle::from_ptr(lastbuf),
        _ => nvim_get_curbuf(),
    };
    let curbuf = nvim_get_curbuf();

    if start == DOBUF_MOD {
        // Find next modified buffer (wraps around).
        let mut remaining = count;
        while remaining > 0 {
            loop {
                let next = BufHandle::from_ptr(buf_ref(buf).b_next);
                buf = if next.is_null() {
                    nvim_get_firstbuf()
                } else {
                    next
                };
                if buf == curbuf || buf_ref(buf).b_changed != 0 {
                    break;
                }
            }
            remaining -= 1;
        }
        if buf_ref(buf).b_changed == 0 {
            errors::emsg_e84();
            return null;
        }
    } else if start == DOBUF_FIRST && count != 0 {
        // Find buffer by number.
        while !buf.is_null() && buf_ref(buf).handle != count {
            buf = BufHandle::from_ptr(buf_ref(buf).b_next);
        }
    } else {
        // Navigate count steps forward/backward through listed buffers.
        let help_only = (flags & DOBUF_SKIPHELP) != 0 && buf_ref(buf).b_help != 0;
        let mut bp = null;
        let mut remaining = count;
        // Mirrors the C while-loop in do_buffer_ext.
        while remaining > 0
            || (bp != buf
                && !unload
                && !(if help_only {
                    buf_ref(buf).b_help != 0
                } else {
                    buf_ref(buf).b_p_bl != 0
                }))
        {
            if bp.is_null() {
                bp = buf;
            }
            buf = nav_step(buf, dir);
            if unload
                || (if help_only {
                    buf_ref(buf).b_help != 0
                } else {
                    buf_ref(buf).b_p_bl != 0
                        && ((flags & DOBUF_SKIPHELP) == 0 || buf_ref(buf).b_help == 0)
                })
            {
                remaining -= 1;
                bp = null;
            }
            if bp == buf {
                errors::emsg_e85();
                return null;
            }
        }
    }

    // Could not find a buffer.
    if buf.is_null() {
        if start == DOBUF_FIRST {
            if !unload {
                errors::semsg_e_nobufnr(i64::from(count));
            }
        } else if dir == FORWARD {
            errors::emsg_e87();
        } else {
            errors::emsg_e88();
        }
        return null;
    }

    // Pre-action validation.
    if action == DOBUF_GOTO && buf != curbuf {
        let forceit = c_int::from((flags & DOBUF_FORCEIT) != 0);
        if !check_can_set_curbuf_forceit(forceit) {
            return null;
        }
        if buf_ref(buf).b_locked_split != 0 {
            nvim_ecmd_emsg_closing_buffer();
            return null;
        }
    }

    if (action == DOBUF_GOTO || action == DOBUF_SPLIT)
        && (buf_ref(buf).b_flags & buf_flags::BF_DUMMY) != 0
    {
        errors::semsg_e_nobufnr(i64::from(count));
        return null;
    }

    buf
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Get buffer lifecycle state.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_lifecycle_state(buf: BufHandle) -> c_int {
    get_lifecycle_state(buf).to_raw()
}

/// Check if buffer can be unloaded.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_can_unload(buf: BufHandle) -> c_int {
    c_int::from(check_can_unload(buf).can_unload)
}

/// Get effective close action for buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_effective_action(buf: BufHandle, action: c_int) -> c_int {
    effective_close_action(buf, BufferAction::from_raw(action)).to_raw()
}

/// Check if buffer has filename.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_has_filename(buf: BufHandle) -> c_int {
    c_int::from(has_filename(buf))
}

/// Check if buffer is modified.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_is_modified(buf: BufHandle) -> c_int {
    c_int::from(is_modified(buf))
}

/// Check if buffer is dummy.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_is_dummy(buf: BufHandle) -> c_int {
    c_int::from(is_dummy(buf))
}

/// Check if buffer was never loaded.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_is_never_loaded(buf: BufHandle) -> c_int {
    c_int::from(is_never_loaded(buf))
}

/// Check if buffer is last in window.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_is_last_in_window(buf: BufHandle) -> c_int {
    c_int::from(is_last_in_window(buf))
}

/// Check if buffer is curbuf.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_is_curbuf(buf: BufHandle) -> c_int {
    c_int::from(is_curbuf(buf))
}

/// Get buffer nwindows count.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_nwindows(buf: BufHandle) -> c_int {
    if buf.is_null() {
        return 0;
    }
    buf_ref(buf).b_nwindows
}

/// Find the best buffer to switch to when deleting the current buffer.
///
/// `buf_fnum` is the file number of the buffer being deleted.
/// `update_jumplist` is set to 0 when `jop_clean` caused a jump-list selection
/// (so caller should pass `false` to `set_curbuf`).
/// Returns NULL if no suitable buffer was found (caller should call `empty_curbuf`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_find_buffer_for_delete(
    buf_fnum: c_int,
    update_jumplist: *mut c_int,
) -> BufHandle {
    find_buffer_for_delete(buf_fnum, update_jumplist)
}

/// Find and validate the target buffer for a `do_buffer_ext` call.
///
/// Implements the navigation logic and pre-action validation from `do_buffer_ext`.
/// Emits the appropriate error message and returns null on failure.
/// On success returns the target buffer handle.
///
/// Called from C `do_buffer_ext` to replace the navigation block.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_find_and_validate_buffer(
    action: c_int,
    start: c_int,
    dir: c_int,
    count: c_int,
    flags: c_int,
    unload: c_int,
) -> BufHandle {
    find_and_validate_buffer(action, start, dir, count, flags, unload != 0)
}

// =============================================================================
// goto_buffer: handle ATTENTION dialog for buffer switching
// =============================================================================

// `CMD_index` values for buffer navigation commands.
// These must match the C enum in `ex_cmds_defs.h` exactly.
// Verified by `_Static_assert` in `buffer.c`.
const CMD_BNEXT: c_int = 30;
const CMD_SBNEXT: c_int = 393;
const CMD_BNEXT_REVERSE: c_int = 21; // CMD_bNext
const CMD_BPREVIOUS: c_int = 32;
const CMD_SBNEXT_REVERSE: c_int = 388; // CMD_sbNext
const CMD_SBPREVIOUS: c_int = 394;

// Opaque representation of `cleanup_T` (16 bytes, 8-byte aligned).
// Layout must match C struct: `{ int pending; [padding 4]; void *exception; }`
#[repr(C, align(8))]
struct CleanupT {
    _data: [u8; 16],
}

extern "C" {
    static mut swap_exists_action: c_int;
    static mut swap_exists_did_quit: bool;

    fn do_buffer_ext(action: c_int, start: c_int, dir: c_int, count: c_int, flags: c_int) -> c_int;
    fn nvim_eap_get_forceit(eap: *const ExArg) -> bool;
    fn win_close(win: *mut c_void, free_buf: bool, free_tabpage: bool) -> c_int;
    fn enter_cleanup(csp: *mut CleanupT);
    fn leave_cleanup(csp: *mut CleanupT);
}

// =============================================================================
// handle_swap_exists: handle the ATTENTION dialog result
// =============================================================================

extern "C" {
    // SEA_RECOVER = 3 (from globals.h)
    // buflist_new(ffname, sfname, lnum, flags) -> buf_T*
    #[link_name = "buflist_new"]
    fn lifecycle_buflist_new(
        ffname_arg: *const c_char,
        sfname_arg: *const c_char,
        lnum: c_int,
        flags: c_int,
    ) -> BufHandle;
    fn block_autocmds();
    fn unblock_autocmds();
    fn ml_recover(checkext: bool);
    fn do_modelines(flags: c_int);
    fn msg_puts(s: *const c_char);
    static mut msg_scroll: c_int;
    static mut msg_row: c_int;
    static mut cmdline_row: c_int;
}

// SEA_RECOVER constant (from globals.h)
const SEA_RECOVER: c_int = 3;
// BLN_* constants (from buffer.h)
const BLN_CURBUF: c_int = 1;
const BLN_LISTED: c_int = 2;
// DOBUF_UNLOAD action
const DOBUF_UNLOAD_ACTION: c_int = 2;

/// Handle the situation of `swap_exists_action` being set.
///
/// It is allowed for `old_curbuf` to be NULL or invalid.
///
/// Rust port of C `handle_swap_exists()`.
///
/// # Safety
/// Accesses global Neovim state. Must be called on the main thread.
#[unsafe(export_name = "handle_swap_exists")]
pub unsafe extern "C" fn rs_handle_swap_exists(old_curbuf: *mut c_void) {
    let old_tw = buf_ref(nvim_get_curbuf()).b_p_tw;

    if swap_exists_action == SEA_QUIT {
        // Reset the error/interrupt/exception state here so that
        // aborting() returns false when closing a buffer.
        let mut cs = CleanupT { _data: [0u8; 16] };
        enter_cleanup(&raw mut cs);

        // User selected Quit at ATTENTION prompt.  Go back to previous
        // buffer.  If that buffer is gone or the same as the current one,
        // open a new, empty buffer.
        swap_exists_action = SEA_NONE; // don't want it again
        swap_exists_did_quit = true;
        close_buffer(
            nvim_get_curwin(),
            nvim_get_curbuf(),
            DOBUF_UNLOAD_ACTION,
            false,
            false,
        );

        let old_curbuf_ref = old_curbuf.cast::<crate::misc::BufRef>();
        let buf = if old_curbuf_ref.is_null()
            || !bufref_valid(old_curbuf_ref)
            || (*old_curbuf_ref).br_buf == nvim_get_curbuf().as_ptr()
        {
            // Block autocommands here because curwin->w_buffer is NULL.
            block_autocmds();
            let b = lifecycle_buflist_new(
                std::ptr::null(),
                std::ptr::null(),
                1,
                BLN_CURBUF | BLN_LISTED,
            );
            unblock_autocmds();
            b
        } else {
            BufHandle((*old_curbuf_ref).br_buf)
        };

        if !buf.is_null() {
            enter_buffer(buf);
            if old_tw != buf_ref(nvim_get_curbuf()).b_p_tw {
                check_colorcolumn(std::ptr::null(), nvim_get_curwin());
            }
        }
        // If old_curbuf is NULL we are in big trouble here...

        // Restore the error/interrupt/exception state if not discarded by a
        // new aborting error, interrupt, or uncaught exception.
        leave_cleanup(&raw mut cs);
    } else if swap_exists_action == SEA_RECOVER {
        // Reset the error/interrupt/exception state here so that
        // aborting() returns false when closing a buffer.
        let mut cs = CleanupT { _data: [0u8; 16] };
        enter_cleanup(&raw mut cs);

        // User selected Recover at ATTENTION prompt.
        msg_scroll = 1;
        ml_recover(false);
        msg_puts(c"\n".as_ptr()); // don't overwrite the last message
        cmdline_row = msg_row;
        do_modelines(0);

        // Restore the error/interrupt/exception state if not discarded by a
        // new aborting error, interrupt, or uncaught exception.
        leave_cleanup(&raw mut cs);
    }
    swap_exists_action = SEA_NONE;
}

// SEA_* constants (from globals.h)
const SEA_NONE: c_int = 0;
const SEA_DIALOG: c_int = 1;
const SEA_QUIT: c_int = 2;

/// Go to another buffer. Handles the result of the ATTENTION dialog.
///
/// Rust port of C `goto_buffer()`.
///
/// # Safety
/// Accesses global Neovim state. `eap` must be a valid `exarg_T*`.
#[no_mangle]
pub unsafe extern "C" fn goto_buffer(eap: *const ExArg, start: c_int, dir: c_int, count: c_int) {
    let save_sea = swap_exists_action;

    let cmdidx = (*eap).cmdidx;
    let skip_help_buf = matches!(
        cmdidx,
        CMD_BNEXT
            | CMD_SBNEXT
            | CMD_BNEXT_REVERSE
            | CMD_BPREVIOUS
            | CMD_SBNEXT_REVERSE
            | CMD_SBPREVIOUS
    );

    let mut old_curbuf = crate::misc::BufRef {
        br_buf: std::ptr::null_mut(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    let curbuf = nvim_get_curbuf();
    crate::misc::set_bufref(&raw mut old_curbuf, curbuf);

    if swap_exists_action == SEA_NONE {
        swap_exists_action = SEA_DIALOG;
    }

    let cmd_ptr = (*eap).cmd;
    let is_split = !cmd_ptr.is_null() && *cmd_ptr == b's' as c_char;
    let action = if is_split { DOBUF_SPLIT } else { DOBUF_GOTO };
    let flags = (if nvim_eap_get_forceit(eap) {
        DOBUF_FORCEIT
    } else {
        0
    }) | (if skip_help_buf { DOBUF_SKIPHELP } else { 0 });

    let _ = do_buffer_ext(action, start, dir, count, flags);

    if swap_exists_action == SEA_QUIT && is_split {
        let mut cs = CleanupT { _data: [0u8; 16] };
        // Reset the error/interrupt/exception state here so that
        // aborting() returns false when closing a window.
        enter_cleanup(&raw mut cs);
        // Quitting means closing the split window, nothing else.
        win_close(nvim_get_curwin(), true, false);
        swap_exists_action = save_sea;
        swap_exists_did_quit = true;
        // Restore the error/interrupt/exception state if not discarded by a
        // new aborting error, interrupt, or uncaught exception.
        leave_cleanup(&raw mut cs);
    } else {
        rs_handle_swap_exists((&raw mut old_curbuf).cast::<c_void>());
    }
}

// =============================================================================
// empty_curbuf: make current buffer empty when last buffer is wiped
// =============================================================================

extern "C" {
    static mut need_fileinfo: bool;

    fn nvim_ses_get_firstwin() -> *mut c_void;
    fn nvim_win_get_buffer(wp: *mut c_void) -> BufHandle;

    // close_windows(buf, keep_curwin) -- implemented in Rust window crate
    fn close_windows(buf: BufHandle, keep_curwin: c_int);
    fn setpcmark();
    // do_ecmd(fnum, ffname, sfname, eap, newlnum, flags, oldwin)
    fn do_ecmd(
        fnum: c_int,
        ffname: *const c_char,
        sfname: *const c_char,
        eap: *mut ExArg,
        newlnum: c_int,
        flags: c_int,
        oldwin: *mut c_void,
    ) -> c_int;
    // close_buffer(win, buf, action, abort_if_last, ignore_abort)
    fn close_buffer(
        win: *mut c_void,
        buf: BufHandle,
        action: c_int,
        abort_if_last: bool,
        ignore_abort: bool,
    ) -> bool;

    // bufref_valid and set_bufref -- exported from misc.rs
    fn bufref_valid(bufref: *mut crate::misc::BufRef) -> bool;
}

// ECMD newlnum constants (from ex_cmds.h)
const ECMD_ONE: c_int = 1;
// ECMD flags
const ECMD_FORCEIT: c_int = 0x08;
const ECMD_HIDE: c_int = 0x01;

// FAIL/OK from vim_defs.h
const FAIL: c_int = 0;
const OK: c_int = 1;
const DOBUF_UNLOAD_VAL: c_int = 2; // matches DOBUF_UNLOAD

/// Make the current buffer empty.
/// Used when it is wiped out and it's the last buffer.
///
/// Rust port of C `empty_curbuf()`.
///
/// # Safety
/// Accesses global Neovim state. Must be called on the main thread.
#[no_mangle]
pub unsafe extern "C" fn empty_curbuf(close_others: bool, forceit: c_int, action: c_int) -> c_int {
    if action == DOBUF_UNLOAD_VAL {
        crate::errors::emsg_e90();
        return FAIL;
    }

    let buf = nvim_get_curbuf();
    let mut bufref = crate::misc::BufRef {
        br_buf: buf.as_ptr(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    crate::misc::set_bufref(&raw mut bufref, buf);

    if close_others {
        let curwin = nvim_get_curwin();
        let can_close_all_others = if win_ref_raw(curwin).w_floating {
            // Check if there is any non-floating window with a different buffer.
            let mut found = false;
            let mut wp = nvim_ses_get_firstwin();
            while !wp.is_null() {
                if win_ref_raw(wp).w_floating {
                    break;
                }
                if nvim_win_get_buffer(wp) != buf {
                    found = true;
                    break;
                }
                wp = win_ref_raw(wp).w_next.as_ptr();
            }
            found
        } else {
            true
        };
        close_windows(buf, c_int::from(can_close_all_others));
    }

    setpcmark();
    let curwin = nvim_get_curwin();
    let ecmd_flags = if forceit != 0 { ECMD_FORCEIT } else { 0 };
    let retval = do_ecmd(
        0,
        std::ptr::null(),
        std::ptr::null(),
        std::ptr::null_mut(),
        ECMD_ONE,
        ecmd_flags,
        curwin,
    );

    // do_ecmd() may create a new buffer; delete the old one if still valid
    // and no longer in any window.
    let curbuf_after = nvim_get_curbuf();
    if buf != curbuf_after && bufref_valid(&raw mut bufref) && buf_ref(buf).b_nwindows == 0 {
        close_buffer(std::ptr::null_mut(), buf, action, false, false);
    }

    if !close_others {
        need_fileinfo = false;
    }

    retval
}

// =============================================================================
// can_unload_buffer
// =============================================================================

/// Return true when buffer `buf` can be unloaded.
///
/// Gives an error message and returns false when the buffer is locked or the
/// screen is being redrawn and the buffer is in a window.
///
/// # Safety
///
/// Must be called on the Neovim main thread with valid state.
#[must_use]
#[unsafe(export_name = "can_unload_buffer")]
pub unsafe extern "C" fn rs_can_unload_buffer(buf: BufHandle) -> bool {
    if buf_ref(buf).b_locked != 0 {
        errors::emsg_e937_buf_in_use(buf);
        return false;
    }

    if updating_screen {
        let mut wp = nvim_get_firstwin();
        while !wp.is_null() {
            if nvim_win_get_buffer(wp) == buf {
                errors::emsg_e937_buf_in_use(buf);
                return false;
            }
            wp = nvim_win_get_next_in_tab(wp);
        }
    }

    true
}

/// Ensure buffer `buf` is loaded into memory.
///
/// If the buffer already has an open memfile (`b_ml.ml_mfp != NULL`), this is
/// a no-op and returns `true`.  Otherwise it makes the buffer temporarily
/// current via `aucmd_prepbuf`, calls `open_buffer`, and restores state.
///
/// Returns `true` when the buffer is successfully loaded, `false` on failure.
///
/// Mirrors C `buf_ensure_loaded`.
///
/// # Safety
///
/// Must be called on the main thread with valid Neovim state.
#[must_use]
#[unsafe(export_name = "buf_ensure_loaded")]
pub unsafe extern "C" fn rs_buf_ensure_loaded(buf: BufHandle) -> bool {
    if !buf_ref(buf).ml_mfp_is_null() {
        return true; // already loaded
    }
    nvim_buf_aucmd_open_buffer(buf) != 0
}

// =============================================================================
// buf_open_scratch
// =============================================================================

/// Create or switch to a scratch buffer.
///
/// Sets `buftype=nofile`, `bufhidden=hide`, `noswapfile`, and resets bindings.
/// If `bufname` is non-null, applies `BufFilePre`/`BufFilePost` autocmds and
/// calls `setfname` to rename the buffer.
///
/// Mirrors C `buf_open_scratch`.
///
/// # Safety
///
/// Must be called on the main thread with valid Neovim state.
#[unsafe(export_name = "buf_open_scratch")]
pub unsafe extern "C" fn rs_buf_open_scratch(bufnr: c_int, bufname: *mut c_char) -> c_int {
    const FAIL: c_int = 0;
    const OK: c_int = 1;

    // do_ecmd(bufnr, NULL, NULL, NULL, ECMD_ONE, ECMD_HIDE, NULL) != FAIL
    if do_ecmd(
        bufnr,
        std::ptr::null(),
        std::ptr::null(),
        std::ptr::null_mut(),
        ECMD_ONE,
        ECMD_HIDE,
        std::ptr::null_mut(),
    ) == FAIL
    {
        return FAIL;
    }

    if !bufname.is_null() {
        let curbuf = nvim_get_curbuf();
        apply_autocmds(
            EVENT_BUFFILEPRE,
            std::ptr::null(),
            std::ptr::null(),
            false,
            curbuf,
        );
        crate::filename::rs_setfname(curbuf, bufname, std::ptr::null_mut(), true);
        apply_autocmds(
            EVENT_BUFFILEPOST,
            std::ptr::null(),
            std::ptr::null(),
            false,
            nvim_get_curbuf(),
        );
    }

    nvim_set_buf_opts_scratch();
    OK
}

// =============================================================================
// set_curbuf: switch current buffer
// =============================================================================

extern "C" {
    // Phase 3: set_curbuf helpers
    fn nvim_get_cmdmod_cmod_flags() -> c_int;
    fn nvim_excmds_set_curwin_alt_fnum(fnum: c_int);
    static mut VIsual_reselect: c_int;
    fn reset_synblock(win: *mut c_void);
    static mut State: c_int;
    fn nvim_u_sync(force: bool);
    fn bufIsChanged(buf: BufHandle) -> bool;
    fn enter_buffer(buf: BufHandle);
    fn check_colorcolumn(cc: *const c_char, win: *mut c_void) -> *const c_char;
    fn nvim_buf_terminal_check_size(buf: BufHandle) -> c_int;
    fn nvim_curwin_buffer_is_null() -> c_int;
    fn nvim_curwin_buffer_is_buf(buf: BufHandle) -> c_int;
    fn aborting() -> c_int;
    fn rs_win_valid(win: *mut c_void) -> c_int;
    fn rs_get_last_winid() -> c_int;
    fn nvim_set_curwin(win: *mut c_void);
}

// CMOD_KEEPALT flag from ex_cmds_defs.h
const CMOD_KEEPALT: c_int = 0x0100;
// MODE_INSERT from state_defs.h
const MODE_INSERT: c_int = 0x10;
// DOBUF action values
const DOBUF_UNLOAD: c_int = 2;
const DOBUF_DEL: c_int = 3;
const DOBUF_WIPE: c_int = 4;
// auevents_enum
const EVENT_BUFLEAVE: c_int = 7;

/// Switch the current buffer to `buf`.
///
/// Fires `BufLeave` autocommands, closes the previous buffer (if needed),
/// and calls `enter_buffer` for the new buffer.
///
/// # Safety
/// Must be called on the Neovim main thread with valid state.
#[unsafe(export_name = "set_curbuf")]
pub unsafe extern "C" fn rs_set_curbuf(buf: BufHandle, action: c_int, update_jumplist: bool) {
    let unload = action == DOBUF_UNLOAD || action == DOBUF_DEL || action == DOBUF_WIPE;
    let old_tw = buf_ref(nvim_get_curbuf()).b_p_tw;
    let last_winid = rs_get_last_winid();

    if update_jumplist {
        setpcmark();
    }

    if (nvim_get_cmdmod_cmod_flags() & CMOD_KEEPALT) == 0 {
        nvim_excmds_set_curwin_alt_fnum(buf_ref(nvim_get_curbuf()).handle);
    }
    crate::rs_buflist_altfpos(crate::WinHandle(nvim_get_curwin()));

    VIsual_reselect = 0;

    let prevbuf = nvim_get_curbuf();
    let mut prevbufref = crate::misc::BufRef {
        br_buf: std::ptr::null_mut(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    let mut newbufref = crate::misc::BufRef {
        br_buf: std::ptr::null_mut(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    crate::misc::set_bufref(&raw mut prevbufref, prevbuf);
    crate::misc::set_bufref(&raw mut newbufref, buf);

    let curbuf = nvim_get_curbuf();
    if !apply_autocmds(
        EVENT_BUFLEAVE,
        std::ptr::null(),
        std::ptr::null(),
        false,
        curbuf,
    ) || (bufref_valid(&raw mut prevbufref)
        && bufref_valid(&raw mut newbufref)
        && aborting() == 0)
    {
        if nvim_curwin_buffer_is_buf(prevbuf) != 0 {
            reset_synblock(nvim_get_curwin());
        }
        // Close windows if unloading or if alt-bufhidden triggers it.
        let bh = buf_ref(prevbuf).bufhidden_char0();
        let new_lastwinid = rs_get_last_winid();
        if unload || (new_lastwinid != last_winid && (bh == b'w' || bh == b'd' || bh == b'u')) {
            close_windows(prevbuf, 0);
        }
        if bufref_valid(&raw mut prevbufref) && aborting() == 0 {
            let previouswin = nvim_get_curwin();

            // Do not sync when in Insert mode and buffer is open in another window.
            if prevbuf == nvim_get_curbuf()
                && ((State & MODE_INSERT) == 0 || buf_ref(nvim_get_curbuf()).b_nwindows <= 1)
            {
                nvim_u_sync(false);
            }
            let close_win = if nvim_curwin_buffer_is_buf(prevbuf) != 0 {
                nvim_get_curwin()
            } else {
                std::ptr::null_mut()
            };
            let close_action = if unload {
                action
            } else if action == DOBUF_GOTO && !crate::rs_buf_hide(prevbuf) && !bufIsChanged(prevbuf)
            {
                DOBUF_UNLOAD
            } else {
                0
            };
            close_buffer(close_win, prevbuf, close_action, false, false);
            if nvim_get_curwin() != previouswin && rs_win_valid(previouswin) != 0 {
                nvim_set_curwin(previouswin);
            }
        }
    }

    let valid = crate::rs_buf_valid(buf) != 0;
    if (valid && buf != nvim_get_curbuf() && aborting() == 0) || nvim_curwin_buffer_is_null() != 0 {
        let cur = nvim_get_curbuf();
        if !cur.is_null() && prevbuf != cur {
            buf_mut(cur).b_nwindows -= 1;
        }
        enter_buffer(if valid {
            buf
        } else {
            BufHandle::from_ptr(lastbuf)
        });
        if old_tw != buf_ref(nvim_get_curbuf()).b_p_tw {
            check_colorcolumn(std::ptr::null(), nvim_get_curwin());
        }
    }

    if bufref_valid(&raw mut prevbufref) {
        nvim_buf_terminal_check_size(prevbuf);
    }
}

// =============================================================================
// do_buffer_ext: buffer list commands
// =============================================================================

extern "C" {
    fn dialog_changed(buf: BufHandle, checkall: bool);
    fn dialog_close_terminal(buf: BufHandle) -> bool;
    fn can_abandon(buf: BufHandle, forceit: c_int) -> bool;
    fn can_unload_buffer(buf: BufHandle) -> bool;
    fn win_split(size: c_int, flags: c_int) -> c_int;
    fn nvim_get_p_confirm() -> c_int;
    fn nvim_get_p_write() -> c_int;
    fn nvim_get_VIsual_active() -> c_int;
    fn nvim_reset_binding_curwin();
    fn swbuf_goto_win_with_buf(buf: BufHandle) -> *mut c_void;
    fn is_aucmd_win(wp: *mut c_void) -> c_int;
    fn rs_win_locked(wp: *mut c_void) -> c_int;
    fn rs_last_window(win: *mut c_void) -> c_int;
    fn nvim_buf_terminal_running(buf: BufHandle) -> c_int;
    fn nvim_get_lastwin() -> *mut c_void;
    fn end_visual_mode();
    fn semsg(fmt: *const c_char, ...);
}

// CMOD_CONFIRM from ex_cmds_defs.h
const CMOD_CONFIRM: c_int = 0x0080;

/// Implementation of the commands for the buffer list.
///
/// # Safety
/// Must be called on the Neovim main thread with valid state.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_do_buffer_ext(
    action: c_int,
    start: c_int,
    dir: c_int,
    count: c_int,
    flags: c_int,
) -> c_int {
    let unload = action == DOBUF_UNLOAD || action == DOBUF_DEL || action == DOBUF_WIPE;
    let mut update_jumplist = true;

    // Find and validate target buffer (navigation + pre-action checks).
    let mut buf =
        rs_find_and_validate_buffer(action, start, dir, count, flags, c_int::from(unload));
    if buf.is_null() {
        return FAIL;
    }

    // delete buffer "buf" from memory and/or the list
    if unload {
        if !can_unload_buffer(buf) {
            return FAIL;
        }
        let mut bufref = crate::misc::BufRef {
            br_buf: std::ptr::null_mut(),
            br_fnum: 0,
            br_buf_free_count: 0,
        };
        crate::misc::set_bufref(&raw mut bufref, buf);

        // When unloading or deleting a buffer that's already unloaded and
        // unlisted: fail silently.
        if action != DOBUF_WIPE && buf_ref(buf).ml_mfp_is_null() && buf_ref(buf).b_p_bl == 0 {
            return FAIL;
        }

        if (flags & DOBUF_FORCEIT) == 0 && bufIsChanged(buf) {
            let p_confirm = nvim_get_p_confirm() != 0;
            let cmod_confirm = (nvim_get_cmdmod_cmod_flags() & CMOD_CONFIRM) != 0;
            let p_write = nvim_get_p_write() != 0;
            if (p_confirm || cmod_confirm) && p_write {
                dialog_changed(buf, false);
                if !bufref_valid(&raw mut bufref) {
                    return FAIL;
                }
                if bufIsChanged(buf) {
                    return FAIL;
                }
            } else {
                // "E89: No write since last change for buffer %ld (add ! to override)"
                semsg(
                    c"E89: No write since last change for buffer %ld (add ! to override)".as_ptr(),
                    i64::from(buf_ref(buf).handle),
                );
                return FAIL;
            }
        }

        if (flags & DOBUF_FORCEIT) == 0 && nvim_buf_terminal_running(buf) != 0 {
            let p_confirm = nvim_get_p_confirm() != 0;
            let cmod_confirm = (nvim_get_cmdmod_cmod_flags() & CMOD_CONFIRM) != 0;
            if p_confirm || cmod_confirm {
                if !dialog_close_terminal(buf) {
                    return FAIL;
                }
            } else {
                semsg(
                    c"E89: %s will be killed (add ! to override)".as_ptr(),
                    buf_ref(buf).b_fname,
                );
                return FAIL;
            }
        }

        let buf_fnum = buf_ref(buf).handle;

        // When closing the current buffer stop Visual mode.
        if buf == nvim_get_curbuf() && nvim_get_VIsual_active() != 0 {
            end_visual_mode();
        }

        // If deleting the last (listed) buffer, make it empty.
        let mut bp: BufHandle = BufHandle(std::ptr::null_mut());
        let mut iter = nvim_get_firstbuf();
        while !iter.is_null() {
            if buf_ref(iter).b_p_bl != 0 && iter != buf {
                bp = iter;
                break;
            }
            iter = BufHandle::from_ptr(buf_ref(iter).b_next);
        }
        if bp.is_null() && buf == nvim_get_curbuf() {
            return empty_curbuf(true, flags & DOBUF_FORCEIT, action);
        }

        // If the deleted buffer is the current one, close the current window
        // (unless it's the only non-floating window).
        let mut curwin = nvim_get_curwin();
        while buf == nvim_get_curbuf()
            && rs_win_locked(curwin) == 0
            && buf_ref(nvim_get_curbuf()).b_locked == 0
            && (is_aucmd_win(nvim_get_lastwin()) != 0 || rs_last_window(curwin) == 0)
        {
            if win_close(curwin, false, false) == FAIL {
                break;
            }
            curwin = nvim_get_curwin();
        }

        // If the buffer to be deleted is not the current one, delete it here.
        if buf != nvim_get_curbuf() {
            if (jop_flags & K_OPT_JOP_FLAG_CLEAN) != 0 {
                mark_jumplist_forget_file(nvim_get_curwin(), buf_fnum);
            }
            close_windows(buf, 0);
            if buf != nvim_get_curbuf()
                && bufref_valid(&raw mut bufref)
                && buf_ref(buf).b_nwindows <= 0
            {
                close_buffer(std::ptr::null_mut(), buf, action, false, false);
            }
            return OK;
        }

        // Deleting the current buffer: Need to find another buffer to go to.
        let mut update_jumplist_int: c_int = i32::from(update_jumplist);
        buf = rs_find_buffer_for_delete(buf_fnum, &raw mut update_jumplist_int);
        update_jumplist = update_jumplist_int != 0;
    }

    if buf.is_null() {
        return empty_curbuf(false, flags & DOBUF_FORCEIT, action);
    }

    // make "buf" the current buffer
    if action == DOBUF_SPLIT {
        if !swbuf_goto_win_with_buf(buf).is_null() {
            return OK;
        }
        if win_split(0, 0) == FAIL {
            return FAIL;
        }
    }

    // go to current buffer - nothing to do
    if buf == nvim_get_curbuf() {
        return OK;
    }

    // Check if the current buffer may be abandoned.
    if action == DOBUF_GOTO && !can_abandon(nvim_get_curbuf(), flags & DOBUF_FORCEIT) {
        let p_confirm = nvim_get_p_confirm() != 0;
        let cmod_confirm = (nvim_get_cmdmod_cmod_flags() & CMOD_CONFIRM) != 0;
        let p_write = nvim_get_p_write() != 0;
        if (p_confirm || cmod_confirm) && p_write {
            let mut bufref = crate::misc::BufRef {
                br_buf: std::ptr::null_mut(),
                br_fnum: 0,
                br_buf_free_count: 0,
            };
            crate::misc::set_bufref(&raw mut bufref, buf);
            dialog_changed(nvim_get_curbuf(), false);
            if !bufref_valid(&raw mut bufref) {
                return FAIL;
            }
        }
        if bufIsChanged(nvim_get_curbuf()) {
            crate::misc::no_write_message();
            return FAIL;
        }
    }

    // Go to the other buffer.
    rs_set_curbuf(buf, action, update_jumplist);

    if action == DOBUF_SPLIT {
        nvim_reset_binding_curwin();
    }

    if aborting() != 0 {
        return FAIL;
    }

    OK
}

// =============================================================================
// enter_buffer + buflist_getfpos
// =============================================================================

extern "C" {
    fn nvim_curwin_set_buffer2(buf: BufHandle);
    fn nvim_set_curbuf_ptr(buf: BufHandle);
    fn buf_copy_options(buf: BufHandle, flags: c_int);
    fn get_winopts(buf: BufHandle);
    fn rs_clearFolding(win: *mut c_void);
    fn rs_foldUpdateAll(win: *mut c_void);
    fn rs_diff_buf_add(buf: BufHandle);
    fn nvim_ecmd_curwin_set_ws_to_buf(buf: BufHandle);
    #[link_name = "set_topline"]
    fn nvim_set_topline(wp: *mut c_void, lnum: c_int);
    fn nvim_ecmd_curwin_set_coladd_curswant();
    fn nvim_excmds_buf_ft_is_empty(buf: BufHandle) -> c_int;
    fn shortmess(x: c_int) -> bool;
    fn nvim_ecmd_curbuf_set_did_filetype(val: c_int);
    fn open_buffer(read_stdin: bool, eap: *mut ExArg, flags: c_int) -> c_int;
    static msg_silent: c_int;
    fn buf_check_timestamp(buf: BufHandle);
    fn inindent(extra: c_int) -> c_int;
    fn maketitle();
    fn scroll_cursor_halfway(win: *mut c_void, atend: bool, prefer_above: bool);
    fn nvim_ecmd_curbuf_set_last_used();
    fn parse_spelllang(win: *mut c_void) -> *mut c_char;
    fn redraw_later(win: *mut c_void, upd: c_int);
    fn nvim_get_p_sol() -> c_int;
    fn check_cursor_lnum(win: *mut c_void);
    fn check_cursor_col(win: *mut c_void);
    fn buflist_findfmark(buf: BufHandle) -> *mut c_void;
    fn nvim_fmark_get_lnum(fm: *const c_void) -> c_int;
    fn nvim_fmark_get_col(fm: *const c_void) -> c_int;
    fn nvim_curwin_set_cursor(lnum: c_int, col: c_int);
    fn keymap_init() -> *mut c_char;
    fn nvim_ecmd_curbuf_get_kmap_state() -> c_int;
    fn check_arg_idx(win: *mut c_void);
    fn nvim_get_entered_free_all_mem() -> c_int;
    fn nvim_win_get_b_p_spl(win: *const c_void) -> *const c_char;
    fn nvim_curwin_set_cursor_lnum(lnum: c_int);
    fn nvim_curwin_set_cursor_col(col: c_int);
    fn nvim_curwin_set_cursor_coladd(coladd: c_int);
    fn nvim_curwin_set_w_set_curswant(val: bool);
}

// BCO_* constants (from option.h)
const BCO_ENTER: c_int = 1;
const BCO_NOHELP: c_int = 4;
// SHM_FILEINFO = 'F' (from option_vars.h)
const SHM_FILEINFO: c_int = b'F' as c_int;
// EVENT constants (from auevents_enum.generated.h)
const EVENT_BUFENTER: c_int = 3;
const EVENT_BUFWINENTER: c_int = 16;
// UPD_NOT_VALID (from drawscreen.h)
const UPD_NOT_VALID: c_int = 40;
// KEYMAP_INIT (from buffer_defs.h)
const KEYMAP_INIT: c_int = 1;

/// Go to the last known line number for the current buffer.
///
/// Port of C `buflist_getfpos()` (static, only called by `enter_buffer`).
///
/// # Safety
/// Accesses global Neovim state. Must be called on the main thread.
unsafe fn buflist_getfpos() {
    let curbuf = nvim_get_curbuf();
    let curwin = nvim_get_curwin();
    let fm = buflist_findfmark(curbuf);
    let fmark_lnum = nvim_fmark_get_lnum(fm);
    let fmark_col = nvim_fmark_get_col(fm);

    // Set lnum and validate it (check_cursor_lnum may clamp it).
    nvim_curwin_set_cursor_lnum(fmark_lnum);
    check_cursor_lnum(curwin);

    if nvim_get_p_sol() != 0 {
        // 'startofline' set: move to col 0.
        nvim_curwin_set_cursor_col(0);
    } else {
        // Restore saved column, validate it, clear coladd, set curswant.
        nvim_curwin_set_cursor_col(fmark_col);
        check_cursor_col(curwin);
        nvim_curwin_set_cursor_coladd(0);
        nvim_curwin_set_w_set_curswant(true);
    }
}

/// Enter a new current buffer.
///
/// Old `curbuf` must have been abandoned already. This also means `curbuf`
/// may point to freed memory when this function starts.
///
/// Port of C `enter_buffer()`.
///
/// # Safety
/// Accesses global Neovim state. Must be called on the main thread with valid
/// `buf`.
#[unsafe(export_name = "enter_buffer")]
pub unsafe extern "C" fn rs_enter_buffer(buf: BufHandle) {
    let curwin = nvim_get_curwin();

    // When closing the current buffer stop Visual mode.
    if nvim_get_VIsual_active() != 0 && nvim_get_entered_free_all_mem() == 0 {
        end_visual_mode();
    }

    // Get the buffer in the current window.
    nvim_curwin_set_buffer2(buf);
    nvim_set_curbuf_ptr(buf);
    let nwindows = buf_ref(buf).b_nwindows;
    buf_mut(buf).b_nwindows = nwindows + 1;

    // Copy buffer and window local option values.  Not for a help buffer.
    buf_copy_options(buf, BCO_ENTER | BCO_NOHELP);
    if buf_ref(buf).b_help == 0 {
        get_winopts(buf);
    } else {
        // Remove all folds in the window.
        rs_clearFolding(curwin);
    }
    rs_foldUpdateAll(curwin); // update folds (later).

    if win_ref_raw(curwin).w_p_diff() != 0 {
        rs_diff_buf_add(buf);
    }

    nvim_ecmd_curwin_set_ws_to_buf(buf);

    // Cursor on first line by default.
    nvim_curwin_set_cursor(1, 0);
    nvim_ecmd_curwin_set_coladd_curswant();
    win_mut_raw(curwin).w_topline_was_set = 0;

    // mark cursor position as being invalid
    win_mut_raw(curwin).w_valid = 0;

    // Make sure the buffer is loaded.
    if buf_ref(buf).ml_mfp_is_null() {
        // need to load the file
        // If there is no filetype, allow for detecting one.  Esp. useful for
        // ":ball" used in an autocommand.  If there already is a filetype we
        // might prefer to keep it.
        if nvim_excmds_buf_ft_is_empty(buf) != 0 {
            nvim_ecmd_curbuf_set_did_filetype(0);
        }

        open_buffer(false, std::ptr::null_mut(), 0);
    } else {
        if msg_silent == 0 && !shortmess(SHM_FILEINFO) {
            need_fileinfo = true; // display file info after redraw
        }
        // check if file changed
        buf_check_timestamp(buf);

        nvim_set_topline(curwin, 1);
        win_mut_raw(curwin).w_topfill = 0;
        apply_autocmds(
            EVENT_BUFENTER,
            std::ptr::null(),
            std::ptr::null(),
            false,
            buf,
        );
        apply_autocmds(
            EVENT_BUFWINENTER,
            std::ptr::null(),
            std::ptr::null(),
            false,
            buf,
        );
    }

    // If autocommands did not change the cursor position, restore cursor lnum
    // and possibly cursor col.
    let curwin = nvim_get_curwin(); // re-acquire after autocommands
    let curbuf = nvim_get_curbuf();
    if win_ref_raw(curwin).w_cursor.lnum == 1 && inindent(0) != 0 {
        buflist_getfpos();
    }

    check_arg_idx(curwin); // check for valid arg_idx
    maketitle();
    // when autocmds didn't change it
    if win_ref_raw(curwin).w_topline == 1 && win_ref_raw(curwin).w_topline_was_set == 0 {
        scroll_cursor_halfway(curwin, false, false); // redisplay at correct position
    }

    // Change directories when the 'acd' option is set.
    crate::misc::do_autochdir();

    if (nvim_ecmd_curbuf_get_kmap_state() & KEYMAP_INIT) != 0 {
        keymap_init();
    }
    // May need to set the spell language.  Can only do this after the buffer
    // has been properly setup.
    // May need to set the spell language -- only if !b_help, w_p_spell, and spl is non-empty.
    let spl = nvim_win_get_b_p_spl(curwin);
    if buf_ref(curbuf).b_help == 0
        && win_ref_raw(curwin).w_p_spell() != 0
        && !spl.is_null()
        && *spl != 0
    {
        parse_spelllang(curwin);
    }
    nvim_ecmd_curbuf_set_last_used();

    if !buf_ref(curbuf).terminal.is_null() {
        nvim_buf_terminal_check_size(curbuf);
    }

    redraw_later(curwin, UPD_NOT_VALID);
}

// =============================================================================
// ex_buffer_all: :ball / :sball / :unhide / :sunhide
// =============================================================================

extern "C" {
    // eap accessors (ex_docmd.c)

    // tabpage traversal (window globals)
    fn nvim_get_curtab() -> *mut c_void;
    fn nvim_get_first_tabpage() -> *mut c_void;
    fn nvim_al_tp_get_next(tp: *mut c_void) -> *mut c_void;
    fn goto_tabpage_tp(tp: *mut c_void, trigger_enter: bool, trigger_leave: bool);

    // reset VIsual mode (normal/src/lib.rs)
    fn rs_reset_VIsual_and_resel();

    // window list traversal and properties (win_struct.rs)

    // globals (arglist shim)
    fn nvim_al_ONE_WINDOW() -> c_int;
    fn nvim_al_is_aucmd_win(wp: *mut c_void) -> c_int;
    fn nvim_al_win_enter(wp: *mut c_void, undo_sync: c_int);
    fn nvim_al_win_move_after(wp: *mut c_void, after: *mut c_void);
    fn nvim_al_get_autocmd_no_enter() -> c_int;
    fn nvim_al_set_autocmd_no_enter(val: c_int);
    fn nvim_al_get_autocmd_no_leave() -> c_int;
    fn nvim_al_set_autocmd_no_leave(val: c_int);
    #[link_name = "rs_lastwin_nofloating"]
    fn nvim_al_lastwin_nofloating() -> *mut c_void;
    #[link_name = "rs_tabpage_index"]
    fn nvim_al_tabpage_index(tp: *mut c_void) -> c_int;
    fn nvim_al_get_p_tpm() -> c_int;
    fn nvim_al_get_p_ea() -> c_int;
    fn nvim_al_set_p_ea(val: c_int);
    fn nvim_al_set_cmdmod_cmod_tab(val: c_int);
    fn nvim_al_autowrite(buf: BufHandle, eap_forceit: c_int) -> c_int;
    fn nvim_al_get_Columns() -> c_int;

    // Direct cmdmod access
    static mut cmdmod: nvim_ex_cmds_types::CmdMod;

    // screen geometry (window globals)
    fn nvim_get_rows_avail() -> c_int;

    // global interruption
    fn os_breakcheck();
    fn vgetc() -> c_int;
    #[link_name = "got_int"]
    static mut ex_buffer_got_int: bool;

    // buf_hide (Rust-exported from buffer lib.rs)
    fn buf_hide(buf: BufHandle) -> bool;
}

// CMD_sunhide and CMD_unhide values from ex_cmds_enum.generated.h
const CMD_SUNHIDE: c_int = 437;
const CMD_UNHIDE: c_int = 495;

// WSP_* flags from window.h
const WSP_ROOM: c_int = 0x01;
const WSP_VERT: c_int = 0x02;
const WSP_BELOW: c_int = 0x40;

/// Open a window for a number of buffers.
///
/// Implements `:ball`, `:sball`, `:unhide`, `:sunhide`.
///
/// Port of C `ex_buffer_all()`.
///
/// # Safety
/// Accesses global Neovim state. Must be called on the main thread.
#[unsafe(export_name = "ex_buffer_all")]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_ex_buffer_all(eap: *mut ExArg) {
    let mut split_ret = OK;
    let mut open_wins: c_int = 0;
    let had_tab = cmdmod.cmod_tab;

    // Maximum number of windows to open.
    let count: c_int = if (*eap).addr_count == 0 {
        9999 // make as many windows as possible
    } else {
        (*eap).line2
    };

    // When true also load inactive buffers.
    let cmdidx = (*eap).cmdidx;
    let all = cmdidx != CMD_UNHIDE && cmdidx != CMD_SUNHIDE;

    // Stop Visual mode; cursor and "VIsual" may be invalid after switching buffers.
    rs_reset_VIsual_and_resel();

    setpcmark();

    // Close superfluous windows (two windows for same buffer or not full-width).
    if had_tab > 0 {
        goto_tabpage_tp(nvim_get_first_tabpage(), true, true);
    }
    'close_loop: loop {
        let mut tpnext = nvim_al_tp_get_next(nvim_get_curtab());

        // Try to close floating windows first.
        let lastwin = nvim_get_lastwin();
        let firstwin = nvim_get_firstwin();
        let mut wp = if win_ref_raw(lastwin).w_floating {
            lastwin
        } else {
            firstwin
        };
        while !wp.is_null() {
            // Compute wpnext
            let wpnext = if win_ref_raw(wp).w_floating {
                let prev = win_ref_raw(wp).w_prev.as_ptr();
                if !prev.is_null() && win_ref_raw(prev).w_floating {
                    prev
                } else {
                    nvim_get_firstwin()
                }
            } else {
                let next = win_ref_raw(wp).w_next.as_ptr();
                if next.is_null() || win_ref_raw(next).w_floating {
                    std::ptr::null_mut()
                } else {
                    next
                }
            };

            let wp_buf = nvim_win_get_buffer(wp);
            let cmod_split = cmdmod.cmod_split;
            let rows_avail = nvim_get_rows_avail();
            let height_too_small = if (cmod_split & WSP_VERT) != 0 {
                win_ref_raw(wp).w_height
                    + win_ref_raw(wp).w_hsep_height
                    + win_ref_raw(wp).w_status_height
                    < rows_avail
            } else {
                win_ref_raw(wp).w_width != nvim_al_get_Columns()
            };

            let should_close = (buf_ref(wp_buf).b_nwindows > 1
                || win_ref_raw(wp).w_floating
                || height_too_small
                || (had_tab > 0 && wp != nvim_get_firstwin()))
                && nvim_al_ONE_WINDOW() == 0
                && rs_win_locked(nvim_get_curwin()) == 0
                && buf_ref(wp_buf).b_locked == 0
                && nvim_al_is_aucmd_win(wp) == 0;

            if should_close {
                if win_close(wp, false, false) == FAIL {
                    break;
                }
                // Autocommand may change windows: start all over.
                let lastwin2 = nvim_get_lastwin();
                wp = if win_ref_raw(lastwin2).w_floating {
                    lastwin2
                } else {
                    nvim_get_firstwin()
                };
                tpnext = nvim_get_first_tabpage();
                open_wins = 0;
            } else {
                open_wins += 1;
                wp = wpnext;
            }
        }

        // Without the ":tab" modifier only do the current tab page.
        if had_tab == 0 || tpnext.is_null() {
            break 'close_loop;
        }
        goto_tabpage_tp(tpnext, true, true);
    }

    // Go through the buffer list. When a buffer doesn't have a window yet,
    // open one. Otherwise move the window to the right position.
    // Don't execute Win/Buf Enter/Leave autocommands here.
    let ane = nvim_al_get_autocmd_no_enter();
    nvim_al_set_autocmd_no_enter(ane + 1);
    // lastwin may be aucmd_win
    nvim_al_win_enter(nvim_al_lastwin_nofloating(), 0);
    let anl = nvim_al_get_autocmd_no_leave();
    nvim_al_set_autocmd_no_leave(anl + 1);

    let mut buf = nvim_get_firstbuf();
    while !buf.is_null() && open_wins < count {
        // Check if this buffer needs a window.
        if (!all && buf_ref(buf).ml_mfp_is_null()) || buf_ref(buf).b_p_bl == 0 {
            buf = BufHandle::from_ptr(buf_ref(buf).b_next);
            continue;
        }

        // Find or skip existing window for this buffer.
        let wp: *mut c_void = if had_tab != 0 {
            // With ":tab" modifier don't move the window.
            if buf_ref(buf).b_nwindows > 0 {
                nvim_get_lastwin() // buffer has a window, skip it
            } else {
                std::ptr::null_mut()
            }
        } else {
            // Check if this buffer already has a window.
            let mut found: *mut c_void = std::ptr::null_mut();
            let mut w = nvim_get_firstwin();
            while !w.is_null() {
                if !win_ref_raw(w).w_floating && nvim_win_get_buffer(w) == buf {
                    found = w;
                    break;
                }
                w = win_ref_raw(w).w_next.as_ptr();
            }
            // If the buffer already has a window, move it.
            if !found.is_null() {
                nvim_al_win_move_after(found, nvim_get_curwin());
            }
            found
        };

        if wp.is_null() && split_ret == OK {
            let mut bufref = crate::misc::BufRef {
                br_buf: std::ptr::null_mut(),
                br_fnum: 0,
                br_buf_free_count: 0,
            };
            crate::misc::set_bufref(&raw mut bufref, buf);

            // Split the window and put the buffer in it.
            let p_ea_save = nvim_al_get_p_ea();
            nvim_al_set_p_ea(1); // use space from all windows
            split_ret = win_split(0, WSP_ROOM | WSP_BELOW);
            open_wins += 1;
            nvim_al_set_p_ea(p_ea_save);
            if split_ret == FAIL {
                buf = BufHandle::from_ptr(buf_ref(buf).b_next);
                continue;
            }

            // Open the buffer in this window.
            swap_exists_action = SEA_DIALOG;
            rs_set_curbuf(buf, DOBUF_GOTO, (jop_flags & K_OPT_JOP_FLAG_CLEAN) == 0);
            if !bufref_valid(&raw mut bufref) {
                // Autocommands deleted the buffer.
                swap_exists_action = SEA_NONE;
                break;
            }
            if swap_exists_action == SEA_QUIT {
                let mut cs = CleanupT { _data: [0u8; 16] };
                // Reset error/interrupt/exception state so aborting() is false.
                enter_cleanup(&raw mut cs);

                // User selected Quit: close this window.
                win_close(nvim_get_curwin(), true, false);
                open_wins -= 1;
                swap_exists_action = SEA_NONE;
                swap_exists_did_quit = true;

                // Restore error/interrupt/exception state.
                leave_cleanup(&raw mut cs);
            } else {
                rs_handle_swap_exists(std::ptr::null_mut());
            }
        }

        os_breakcheck();
        if ex_buffer_got_int {
            let _ = vgetc(); // only break file loading, not the rest
            break;
        }
        // Autocommands deleted buffer or aborted script processing.
        if aborting() != 0 {
            break;
        }
        // When ":tab" was used, open new tab for each new window repeatedly.
        if had_tab > 0 && nvim_al_tabpage_index(std::ptr::null_mut()) <= nvim_al_get_p_tpm() {
            nvim_al_set_cmdmod_cmod_tab(9999);
        }

        buf = BufHandle::from_ptr(buf_ref(buf).b_next);
    }

    let ane = nvim_al_get_autocmd_no_enter();
    nvim_al_set_autocmd_no_enter(ane - 1);
    nvim_al_win_enter(nvim_get_firstwin(), 0); // back to first window
    let anl = nvim_al_get_autocmd_no_leave();
    nvim_al_set_autocmd_no_leave(anl - 1);

    // Close superfluous windows.
    let mut wp = nvim_get_lastwin();
    while open_wins > count {
        let wp_buf = nvim_win_get_buffer(wp);
        let r = (buf_hide(wp_buf) || !bufIsChanged(wp_buf) || nvim_al_autowrite(wp_buf, 0) == OK)
            && nvim_al_is_aucmd_win(wp) == 0;
        if rs_win_valid(wp) == 0 {
            // BufWrite autocommands made window invalid: start over.
            wp = nvim_get_lastwin();
        } else if r {
            win_close(wp, !buf_hide(wp_buf), false);
            open_wins -= 1;
            wp = nvim_get_lastwin();
        } else {
            wp = win_ref_raw(wp).w_prev.as_ptr();
            if wp.is_null() {
                break;
            }
        }
    }
}

// =============================================================================
// open_buffer (Phase N migration)
// =============================================================================

extern "C" {
    fn nvim_curbuf_mf_set_nosync();
    fn nvim_curbuf_mf_unset_nosync();
    fn nvim_open_buffer_setup_bufref(old_curbuf: *mut crate::misc::BufRef);
    fn nvim_open_buffer_read_file(
        eap: *mut ExArg,
        flags: c_int,
        silent: c_int,
        read_fifo_out: *mut c_int,
    ) -> c_int;
    fn nvim_open_buffer_read_stdin(eap: *mut ExArg, flags: c_int, silent: c_int) -> c_int;
    fn nvim_curbuf_init_first_load();
    fn nvim_open_buffer_set_changed(retval: c_int, read_stdin: c_int, read_fifo: c_int);
    fn nvim_curwin_init_topline();
    fn nvim_open_buffer_bufenter(retval: *mut c_int);
    fn nvim_open_buffer_post_autocmd(
        old_curbuf: *mut crate::misc::BufRef,
        flags: c_int,
        retval: *mut c_int,
    );
    fn rs_bt_nofileread(buf: BufHandle) -> bool;
    fn rs_foldUpdateAll_curwin();
    /// Open memfile for buffer. Returns FAIL (0) on failure, OK (1) on success.
    fn ml_open(buf: BufHandle) -> c_int;
    /// Exit with the given exit value.
    fn getout(exitval: c_int) -> !;
}

extern "C" {
    /// readonlymode: open file as read-only.
    static mut readonlymode: bool;
    /// `v_dying`: non-zero while exiting.
    static mut v_dying: c_int;
}

// READ_NOFILE flag: do not read a file, do trigger BufReadCmd
const READ_NOFILE: c_int = 0x100;

/// `BF_NEVERLOADED` constant for the `ml_open` init path (mirrors `buf_flags::BF_NEVERLOADED`).
const BF_NEVERLOADED_ML: c_int = 0x04;

/// Open memfile for the current buffer, handling the fallback path on failure.
///
/// Rust equivalent of `nvim_open_buffer_ml_init()` in `buffer_shim.c`.
///
/// Returns 1 if `ml_open` succeeded (proceed normally), 0 if it failed
/// (FAIL should be returned by `open_buffer`).
///
/// # Safety
/// Must be called from `open_buffer` with valid global state.
unsafe fn open_buffer_ml_init_impl(old_tw: i64) -> c_int {
    let curbuf = nvim_get_curbuf();

    // Set readonly flag when BF_NEVERLOADED is being reset.
    if readonlymode
        && !buf_ref(curbuf).b_ffname.is_null()
        && (buf_ref(curbuf).b_flags & BF_NEVERLOADED_ML) != 0
    {
        buf_mut(curbuf).b_p_ro = 1;
    }

    // ml_open returns FAIL (0) on failure, OK (1) on success.
    if ml_open(curbuf) != 0 {
        return 1; // success
    }

    // There MUST be a memfile, otherwise we can't do anything.
    // If we can't create one for the current buffer, take another buffer.
    close_buffer(std::ptr::null_mut(), curbuf, 0, false, false);

    // Set curbuf to NULL and search for a buffer that has a memfile.
    nvim_set_curbuf_ptr(BufHandle::from_ptr(std::ptr::null_mut()));

    let mut found = BufHandle::from_ptr(std::ptr::null_mut());
    let mut buf = nvim_get_firstbuf();
    while !buf.is_null() {
        if !buf_ref(buf).ml_mfp_is_null() {
            found = buf;
            break;
        }
        buf = BufHandle::from_ptr(buf_ref(buf).b_next);
    }

    if found.is_null() {
        errors::emsg_e82_no_buffer_exiting();
        v_dying = 2;
        getout(2);
    }

    nvim_set_curbuf_ptr(found);
    let curbuf = found;
    errors::emsg_e83_using_other_buffer();
    enter_buffer(curbuf);
    if old_tw != buf_ref(curbuf).b_p_tw {
        check_colorcolumn(std::ptr::null(), nvim_get_curwin());
    }
    0 // failure: caller should return FAIL
}

/// Open current buffer: open the memfile and read the file into memory.
///
/// Port of C `open_buffer`.
///
/// - `read_stdin`: read file from stdin
/// - `eap`: for forced `ff` and `fenc` or NULL
/// - `flags_arg`: extra flags for `readfile`
///
/// Returns FAIL (0) for failure, OK (1) otherwise.
///
/// # Safety
/// Accesses global Neovim state. Must be called on the main thread.
#[unsafe(export_name = "open_buffer")]
pub unsafe extern "C" fn rs_open_buffer(
    read_stdin: bool,
    eap: *mut ExArg,
    flags_arg: c_int,
) -> c_int {
    const OK: c_int = 1;
    const FAIL: c_int = 0;

    let mut flags = flags_arg;
    let mut retval = OK;
    let silent = shortmess(SHM_FILEINFO);
    let silent_int: c_int = c_int::from(silent);

    // Save old text-width before ml_open attempt (needed for fallback path).
    let old_tw = buf_ref(nvim_get_curbuf()).b_p_tw;

    // Handle readonlymode + try ml_open. Returns 0 (FAIL) if we should bail.
    if open_buffer_ml_init_impl(old_tw) == 0 {
        return FAIL;
    }

    // Do not sync this buffer yet; may first want to read the file.
    nvim_curbuf_mf_set_nosync();

    // Save bufref so we can detect if autocommands changed curbuf.
    let mut old_curbuf = crate::misc::BufRef {
        br_buf: std::ptr::null_mut(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    nvim_open_buffer_setup_bufref(&raw mut old_curbuf);

    // A buffer without an actual file should not use the buffer name to read.
    let curbuf = nvim_get_curbuf();
    if rs_bt_nofileread(curbuf) {
        flags |= READ_NOFILE;
    }

    // Read the file or stdin.
    let mut read_fifo: c_int = 0;
    if !buf_ref(nvim_get_curbuf()).b_ffname.is_null() {
        retval = nvim_open_buffer_read_file(eap, flags, silent_int, &raw mut read_fifo);
    } else if read_stdin {
        retval = nvim_open_buffer_read_stdin(eap, flags, silent_int);
    }
    // If neither, retval stays OK and read_fifo stays 0.

    // Can now sync buffer; handle first-load; set changed state.
    nvim_curbuf_mf_unset_nosync();
    nvim_curbuf_init_first_load();
    nvim_open_buffer_set_changed(retval, c_int::from(read_stdin), read_fifo);
    rs_foldUpdateAll_curwin();
    nvim_curwin_init_topline();
    nvim_open_buffer_bufenter(&raw mut retval);
    if retval == FAIL {
        return retval;
    }
    nvim_open_buffer_post_autocmd(&raw mut old_curbuf, flags, &raw mut retval);

    retval
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_action() {
        assert!(!BufferAction::None.unloads());
        assert!(BufferAction::Unload.unloads());
        assert!(BufferAction::Delete.unloads());
        assert!(BufferAction::Wipe.unloads());

        assert!(!BufferAction::None.deletes());
        assert!(!BufferAction::Unload.deletes());
        assert!(BufferAction::Delete.deletes());
        assert!(BufferAction::Wipe.deletes());

        assert!(!BufferAction::None.wipes());
        assert!(!BufferAction::Unload.wipes());
        assert!(!BufferAction::Delete.wipes());
        assert!(BufferAction::Wipe.wipes());
    }

    #[test]
    fn test_lifecycle_state() {
        assert!(!LifecycleState::NeverLoaded.is_loaded());
        assert!(!LifecycleState::NotLoaded.is_loaded());
        assert!(LifecycleState::Hidden.is_loaded());
        assert!(LifecycleState::Normal.is_loaded());

        assert!(!LifecycleState::NeverLoaded.is_visible());
        assert!(!LifecycleState::NotLoaded.is_visible());
        assert!(!LifecycleState::Hidden.is_visible());
        assert!(LifecycleState::Normal.is_visible());
    }

    #[test]
    fn test_buffer_action_roundtrip() {
        for i in 0..4 {
            let action = BufferAction::from_raw(i);
            assert_eq!(action.to_raw(), i);
        }
    }

    #[test]
    fn test_lifecycle_state_roundtrip() {
        for i in 0..4 {
            let state = LifecycleState::from_raw(i);
            assert_eq!(state.to_raw(), i);
        }
    }

    #[test]
    fn test_unload_check_default() {
        let check = UnloadCheck::default();
        assert!(!check.can_unload);
        assert!(!check.is_locked);
        assert!(!check.is_locked_split);
        assert!(!check.has_terminal);
        assert_eq!(check.nwindows, 0);
    }
}
