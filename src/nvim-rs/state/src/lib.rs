//! State management for Neovim Rust components
//!
//! This crate provides state management infrastructure for Neovim,
//! including global state, mode tracking, cursor state, and screen state.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)] // FFI functions cannot be const

pub mod cursor;
pub mod global;
pub mod mode;
pub mod screen;

use std::ffi::{c_int, c_void};
use std::sync::atomic::{AtomicBool, Ordering};

// =============================================================================
// State Change Types
// =============================================================================

/// Types of state changes.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StateChangeType {
    /// No change
    #[default]
    None = 0,
    /// Mode changed
    Mode = 1,
    /// Cursor moved
    Cursor = 2,
    /// Buffer changed
    Buffer = 3,
    /// Window changed
    Window = 4,
    /// Screen needs redraw
    Screen = 5,
    /// Option changed
    Option = 6,
    /// Multiple changes
    Multiple = 7,
}

impl StateChangeType {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Mode,
            2 => Self::Cursor,
            3 => Self::Buffer,
            4 => Self::Window,
            5 => Self::Screen,
            6 => Self::Option,
            7 => Self::Multiple,
            _ => Self::None,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if this is a significant change.
    #[must_use]
    pub const fn is_significant(self) -> bool {
        !matches!(self, Self::None)
    }
}

/// FFI: Check if state change is significant.
#[no_mangle]
pub extern "C" fn rs_state_change_is_significant(change: c_int) -> c_int {
    c_int::from(StateChangeType::from_c_int(change).is_significant())
}

// =============================================================================
// State Flags
// =============================================================================

/// Flags for tracking various state conditions.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct StateFlags {
    /// In a mapping
    pub in_mapping: bool,
    /// In a script
    pub in_script: bool,
    /// In an expression
    pub in_expression: bool,
    /// In a function call
    pub in_function: bool,
    /// In a try block
    pub in_try: bool,
    /// Exception pending
    pub exception_pending: bool,
    /// Got an interrupt
    pub got_interrupt: bool,
    /// Need to check for interrupts
    pub need_check_int: bool,
}

impl StateFlags {
    /// Create new state flags.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            in_mapping: false,
            in_script: false,
            in_expression: false,
            in_function: false,
            in_try: false,
            exception_pending: false,
            got_interrupt: false,
            need_check_int: false,
        }
    }

    /// Check if in any execution context.
    #[must_use]
    pub const fn in_execution(&self) -> bool {
        self.in_mapping || self.in_script || self.in_function
    }

    /// Check if should stop execution.
    #[must_use]
    pub const fn should_stop(&self) -> bool {
        self.got_interrupt || self.exception_pending
    }
}

/// FFI: Create state flags.
#[no_mangle]
pub extern "C" fn rs_state_flags_new() -> StateFlags {
    StateFlags::new()
}

/// FFI: Check if in execution.
///
/// # Safety
/// `flags` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_state_in_execution(flags: *const StateFlags) -> c_int {
    if flags.is_null() {
        return 0;
    }
    c_int::from((*flags).in_execution())
}

/// FFI: Check if should stop.
///
/// # Safety
/// `flags` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_state_should_stop(flags: *const StateFlags) -> c_int {
    if flags.is_null() {
        return 0;
    }
    c_int::from((*flags).should_stop())
}

// =============================================================================
// State Snapshot
// =============================================================================

/// Snapshot of state at a point in time.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct StateSnapshot {
    /// Sequence number
    pub seq: u64,
    /// Current mode
    pub mode: c_int,
    /// Current buffer ID
    pub buffer_id: c_int,
    /// Current window ID
    pub window_id: c_int,
    /// Cursor line
    pub cursor_line: c_int,
    /// Cursor column
    pub cursor_col: c_int,
    /// Timestamp (ms since epoch)
    pub timestamp_ms: i64,
}

impl StateSnapshot {
    /// Create new snapshot.
    #[must_use]
    pub const fn new(seq: u64) -> Self {
        Self {
            seq,
            mode: 0,
            buffer_id: 0,
            window_id: 0,
            cursor_line: 1,
            cursor_col: 0,
            timestamp_ms: 0,
        }
    }

    /// Check if snapshot is valid.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.buffer_id > 0 && self.window_id > 0
    }

    /// Check if cursor changed from another snapshot.
    #[must_use]
    pub const fn cursor_changed(&self, other: &Self) -> bool {
        self.cursor_line != other.cursor_line || self.cursor_col != other.cursor_col
    }
}

/// FFI: Create snapshot.
#[no_mangle]
pub extern "C" fn rs_state_snapshot_new(seq: u64) -> StateSnapshot {
    StateSnapshot::new(seq)
}

/// FFI: Check if snapshot is valid.
///
/// # Safety
/// `snap` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_state_snapshot_is_valid(snap: *const StateSnapshot) -> c_int {
    if snap.is_null() {
        return 0;
    }
    c_int::from((*snap).is_valid())
}

// =============================================================================
// Phase 1: virtual_active, SafeState functions
// =============================================================================

// kOptVeFlags values (from option_vars.generated.h)
const K_OPT_VE_FLAG_ALL: u32 = 0x04;
const K_OPT_VE_FLAG_BLOCK: u32 = 0x05;
const K_OPT_VE_FLAG_INSERT: u32 = 0x06;

// Mode constants (from state_defs.h)
const MODE_TERMINAL: c_int = 0x80;
const MODE_INSERT: c_int = 0x10;

// TriState values (kNone = -1, kFalse = 0, kTrue = 1)
const K_NONE: c_int = -1;

// Ctrl_V = 22 (from ascii_defs.h)
const CTRL_V: c_int = 22;

// WinHandle type alias
type WinHandle = *mut c_void;

extern "C" {
    /// Current editor state flags.
    static mut State: c_int;
    /// `VIsual_active` global (visual mode active).
    static mut VIsual_active: bool;
    /// Get `VIsual_mode` global.
    fn nvim_get_VIsual_mode() -> c_int;
    /// Get `virtual_op` `TriState`.
    fn nvim_get_virtual_op() -> c_int;
    /// Get `ve_flags` for a window.
    #[link_name = "get_ve_flags"]
    fn nvim_state_get_ve_flags(wp: WinHandle) -> c_uint;
    /// Get `typebuf.tb_len`.
    fn nvim_get_typebuf_len() -> c_int;
    /// Check if stuff is empty.
    fn nvim_stuff_empty() -> c_int;
    /// Check if using a script.
    fn nvim_using_script() -> c_int;
    /// Get `global_busy`.
    fn nvim_get_global_busy() -> bool;
    /// Get `debug_mode`.
    fn nvim_is_debug_mode() -> c_int;
    /// Apply `SafeState` autocommand.
    fn nvim_apply_autocmds_safestate();
    // Log message (DLOG macro equivalent -- ignored in Rust for now).
    // (no-op in Rust migration)
}

use std::ffi::c_uint;

/// Rust-owned `was_safe` static (replaces C static in state.c).
static WAS_SAFE: AtomicBool = AtomicBool::new(false);

/// Return whether currently it is safe (no typeahead, not in script, etc.).
///
/// # Safety
/// Calls C accessor functions.
unsafe fn is_safe_now() -> bool {
    nvim_stuff_empty() != 0
        && nvim_get_typebuf_len() == 0
        && nvim_using_script() == 0
        && !nvim_get_global_busy()
        && nvim_is_debug_mode() == 0
}

/// Return true if in the current mode we need to use virtual editing.
///
/// # Safety
/// Reads C globals via accessor functions.
#[unsafe(export_name = "virtual_active")]
pub unsafe extern "C" fn rs_virtual_active(wp: WinHandle) -> bool {
    // While an operator is being executed we return virtual_op, because
    // VIsual_active has already been reset.
    let virtual_op = nvim_get_virtual_op();
    if virtual_op != K_NONE {
        return virtual_op != 0;
    }

    // In Terminal mode the cursor can be positioned anywhere.
    if (State & MODE_TERMINAL) != 0 {
        return true;
    }

    let cur_ve_flags = nvim_state_get_ve_flags(wp);

    cur_ve_flags == K_OPT_VE_FLAG_ALL
        || ((cur_ve_flags & K_OPT_VE_FLAG_BLOCK) != 0
            && VIsual_active
            && nvim_get_VIsual_mode() == CTRL_V)
        || ((cur_ve_flags & K_OPT_VE_FLAG_INSERT) != 0 && (State & MODE_INSERT) != 0)
}

/// Trigger `SafeState` autocmd if currently in a safe state.
///
/// # Safety
/// Calls C autocmd/state functions.
#[unsafe(export_name = "may_trigger_safestate")]
pub unsafe extern "C" fn rs_may_trigger_safestate(safe: bool) {
    let is_safe = safe && is_safe_now();
    // (DLOG for state changes omitted -- Rust migration)
    if is_safe {
        nvim_apply_autocmds_safestate();
    }
    WAS_SAFE.store(is_safe, Ordering::Relaxed);
}

/// Reset the `was_safe` flag (something changed making state unsafe).
///
/// # Safety
/// Modifies Rust-owned atomic.
#[unsafe(export_name = "state_no_longer_safe")]
pub unsafe extern "C" fn rs_state_no_longer_safe(_reason: *const std::ffi::c_char) {
    // (DLOG omitted)
    WAS_SAFE.store(false, Ordering::Relaxed);
}

/// C-callable accessor for `was_safe` (for event crate compatibility).
#[no_mangle]
pub extern "C" fn nvim_state_get_was_safe() -> c_int {
    c_int::from(WAS_SAFE.load(Ordering::Relaxed))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_change_type() {
        assert!(!StateChangeType::None.is_significant());
        assert!(StateChangeType::Mode.is_significant());
        assert!(StateChangeType::Cursor.is_significant());
    }

    #[test]
    fn test_state_flags() {
        let mut flags = StateFlags::new();
        assert!(!flags.in_execution());
        assert!(!flags.should_stop());

        flags.in_mapping = true;
        assert!(flags.in_execution());

        flags.got_interrupt = true;
        assert!(flags.should_stop());
    }

    #[test]
    fn test_state_snapshot() {
        let snap = StateSnapshot::new(1);
        assert!(!snap.is_valid());

        let mut snap = snap;
        snap.buffer_id = 1;
        snap.window_id = 1;
        assert!(snap.is_valid());
    }
}
