//! Buffer lifecycle management helpers
//!
//! This module provides Rust implementations for buffer lifecycle operations
//! including creation validation, cleanup preparation, and state transitions.

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_safety_doc)]
#![allow(dead_code)]

use std::ffi::{c_char, c_int, c_uint, c_void};

use crate::BufHandle;

// =============================================================================
// External C Statics
// =============================================================================

extern "C" {
    static mut jop_flags: c_uint;
}

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    fn nvim_get_firstbuf() -> BufHandle;
    fn nvim_get_lastbuf() -> BufHandle;
    fn nvim_get_curbuf() -> BufHandle;
    fn nvim_get_curwin() -> *mut c_void;
    fn nvim_buf_get_prev(buf: BufHandle) -> BufHandle;
    fn nvim_buf_get_next(buf: BufHandle) -> BufHandle;
    fn nvim_buf_get_b_next(buf: BufHandle) -> BufHandle;
    fn nvim_buf_get_fnum(buf: BufHandle) -> c_int;
    fn nvim_buf_get_ffname(buf: BufHandle) -> *const c_char;
    fn nvim_buf_get_nwindows(buf: BufHandle) -> c_int;
    fn nvim_buf_get_locked(buf: BufHandle) -> c_int;
    fn nvim_buf_get_locked_split(buf: BufHandle) -> c_int;
    fn nvim_buf_get_flags(buf: BufHandle) -> c_int;
    fn nvim_buf_get_terminal(buf: BufHandle) -> c_int;
    fn nvim_buf_get_bufhidden(buf: BufHandle) -> c_char;
    fn nvim_buf_get_changed(buf: BufHandle) -> c_int;
    fn nvim_buf_get_ml_mfp(buf: BufHandle) -> *mut c_void;
    fn nvim_buf_get_b_p_bl(buf: BufHandle) -> c_int;
    fn nvim_buf_get_b_help(buf: BufHandle) -> c_int;
    fn nvim_buf_has_memfile(buf: BufHandle) -> c_int;

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

    // Error message emitters for do_buffer_ext navigation (implemented in buffer.c)
    fn nvim_emsg_e84();
    fn nvim_emsg_e85();
    fn nvim_emsg_e87();
    fn nvim_emsg_e88();
    fn nvim_semsg_e_nobufnr(count: i64);

    // Validation helpers for do_buffer_ext (implemented in ex_cmds_shim.c)
    fn nvim_excmds_check_can_set_curbuf_forceit(forceit: c_int) -> c_int;
    fn nvim_ecmd_emsg_closing_buffer();
}

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

    let flags = nvim_buf_get_flags(buf);
    if (flags & buf_flags::BF_NEVERLOADED) != 0 {
        return LifecycleState::NeverLoaded;
    }

    let ml_mfp = nvim_buf_get_ml_mfp(buf);
    if ml_mfp.is_null() {
        return LifecycleState::NotLoaded;
    }

    let nwindows = nvim_buf_get_nwindows(buf);
    if nwindows == 0 {
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

    let is_locked = nvim_buf_get_locked(buf) > 0;
    let is_locked_split = nvim_buf_get_locked_split(buf) > 0;
    let has_terminal = nvim_buf_get_terminal(buf) != 0;
    let nwindows = nvim_buf_get_nwindows(buf);

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
    if nvim_buf_get_terminal(buf) != 0
        && (requested.unloads() || requested.deletes() || requested.wipes())
    {
        return BufferAction::Wipe;
    }

    // Check bufhidden option
    let bh = nvim_buf_get_bufhidden(buf) as u8;
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
    !nvim_buf_get_ffname(buf).is_null()
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
    nvim_buf_get_changed(buf) != 0
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
    (nvim_buf_get_flags(buf) & buf_flags::BF_DUMMY) != 0
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
    (nvim_buf_get_flags(buf) & buf_flags::BF_NEVERLOADED) != 0
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
    nvim_buf_get_nwindows(buf) == 1
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

    LifecyclePosition {
        fnum: nvim_buf_get_fnum(buf),
        state: get_lifecycle_state(buf),
        nwindows: nvim_buf_get_nwindows(buf),
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
                        || nvim_buf_get_b_p_bl(candidate) == 0
                        || rs_bt_quickfix(candidate)
                    {
                        // Not suitable
                    } else if nvim_buf_has_memfile(candidate) == 0 {
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
        let mut buf = nvim_buf_get_b_next(curbuf);
        loop {
            if buf.is_null() {
                if !forward {
                    break; // tried both directions
                }
                buf = nvim_buf_get_prev(curbuf);
                forward = false;
                continue;
            }
            // Prefer same help-buffer type, listed, non-quickfix
            if nvim_buf_get_b_help(buf) == nvim_buf_get_b_help(curbuf)
                && nvim_buf_get_b_p_bl(buf) != 0
                && !rs_bt_quickfix(buf)
            {
                if nvim_buf_has_memfile(buf) != 0 {
                    result = buf;
                    break;
                }
                if bp.is_null() {
                    bp = buf;
                }
            }
            buf = if forward {
                nvim_buf_get_b_next(buf)
            } else {
                nvim_buf_get_prev(buf)
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
            if nvim_buf_get_b_p_bl(buf) != 0 && buf != curbuf && !rs_bt_quickfix(buf) {
                result = buf;
                break;
            }
            buf = nvim_buf_get_b_next(buf);
        }
    }

    // 6. Last resort: adjacent buffer (even if quickfix).
    if result.is_null() {
        let next = nvim_buf_get_b_next(curbuf);
        let prev = nvim_buf_get_prev(curbuf);
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
        let next = nvim_buf_get_b_next(buf);
        if next.is_null() {
            nvim_get_firstbuf()
        } else {
            next
        }
    } else {
        let prev = nvim_buf_get_prev(buf);
        if prev.is_null() {
            nvim_get_lastbuf()
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
        DOBUF_LAST => nvim_get_lastbuf(),
        _ => nvim_get_curbuf(),
    };
    let curbuf = nvim_get_curbuf();

    if start == DOBUF_MOD {
        // Find next modified buffer (wraps around).
        let mut remaining = count;
        while remaining > 0 {
            loop {
                let next = nvim_buf_get_b_next(buf);
                buf = if next.is_null() {
                    nvim_get_firstbuf()
                } else {
                    next
                };
                if buf == curbuf || nvim_buf_get_changed(buf) != 0 {
                    break;
                }
            }
            remaining -= 1;
        }
        if nvim_buf_get_changed(buf) == 0 {
            nvim_emsg_e84();
            return null;
        }
    } else if start == DOBUF_FIRST && count != 0 {
        // Find buffer by number.
        while !buf.is_null() && nvim_buf_get_fnum(buf) != count {
            buf = nvim_buf_get_b_next(buf);
        }
    } else {
        // Navigate count steps forward/backward through listed buffers.
        let help_only = (flags & DOBUF_SKIPHELP) != 0 && nvim_buf_get_b_help(buf) != 0;
        let mut bp = null;
        let mut remaining = count;
        // Mirrors the C while-loop in do_buffer_ext.
        while remaining > 0
            || (bp != buf
                && !unload
                && !(if help_only {
                    nvim_buf_get_b_help(buf) != 0
                } else {
                    nvim_buf_get_b_p_bl(buf) != 0
                }))
        {
            if bp.is_null() {
                bp = buf;
            }
            buf = nav_step(buf, dir);
            if unload
                || (if help_only {
                    nvim_buf_get_b_help(buf) != 0
                } else {
                    nvim_buf_get_b_p_bl(buf) != 0
                        && ((flags & DOBUF_SKIPHELP) == 0 || nvim_buf_get_b_help(buf) == 0)
                })
            {
                remaining -= 1;
                bp = null;
            }
            if bp == buf {
                nvim_emsg_e85();
                return null;
            }
        }
    }

    // Could not find a buffer.
    if buf.is_null() {
        if start == DOBUF_FIRST {
            if !unload {
                nvim_semsg_e_nobufnr(i64::from(count));
            }
        } else if dir == FORWARD {
            nvim_emsg_e87();
        } else {
            nvim_emsg_e88();
        }
        return null;
    }

    // Pre-action validation.
    if action == DOBUF_GOTO && buf != curbuf {
        let forceit = c_int::from((flags & DOBUF_FORCEIT) != 0);
        if nvim_excmds_check_can_set_curbuf_forceit(forceit) == 0 {
            return null;
        }
        if nvim_buf_get_locked_split(buf) != 0 {
            nvim_ecmd_emsg_closing_buffer();
            return null;
        }
    }

    if (action == DOBUF_GOTO || action == DOBUF_SPLIT)
        && (nvim_buf_get_flags(buf) & buf_flags::BF_DUMMY) != 0
    {
        nvim_semsg_e_nobufnr(i64::from(count));
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
    nvim_buf_get_nwindows(buf)
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
