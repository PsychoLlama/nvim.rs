//! Scheduled message utilities
//!
//! Provides helpers for deferred message handling - messages that are
//! scheduled to be displayed later via the event loop. This is used
//! when errors occur in contexts where immediate display is not safe.

use std::ffi::{c_char, c_int, c_void};

// ============================================================================
// Constants
// ============================================================================

/// Maximum size for scheduled message buffer (matches IOSIZE)
pub const SCHEDULED_MSG_BUFSIZE: c_int = 8192;

/// Scheduled message type: normal error
pub const SCHED_MSG_ERROR: c_int = 0;

/// Scheduled message type: multiline error
pub const SCHED_MSG_MULTILINE: c_int = 1;

// ============================================================================
// Scheduled Message State
// ============================================================================

// C function declarations
extern "C" {
    /// Check if we're in a fast callback context (unsafe for direct messages)
    fn nvim_get_in_fast_callback() -> c_int;
    /// Call emsg(s) — error message display.
    fn emsg(s: *const c_char) -> bool;
    /// Call emsg_multiline(s, kind, hl, true) — multiline error display.
    fn emsg_multiline(s: *const c_char, kind: *const c_char, hl: c_int, crlf: bool) -> bool;
    /// Free memory allocated with xstrdup/xmalloc.
    fn xfree(ptr: *mut c_void);
}

/// Check if we're in a fast callback context.
///
/// When in a fast callback, messages should be scheduled rather than
/// displayed immediately to avoid reentrancy issues.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_in_fast_callback() -> c_int {
    nvim_get_in_fast_callback()
}

/// Check if messages should be scheduled (deferred).
///
/// Returns true if the current context requires message scheduling
/// rather than immediate display.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_should_schedule_msg() -> c_int {
    c_int::from(nvim_get_in_fast_callback() != 0)
}

// ============================================================================
// Message Scheduling Helpers
// ============================================================================

/// Calculate buffer size needed for a scheduled message.
///
/// Returns the minimum of the requested size and SCHEDULED_MSG_BUFSIZE.
#[no_mangle]
pub const extern "C" fn rs_sched_msg_bufsize(requested: c_int) -> c_int {
    if requested <= 0 {
        SCHEDULED_MSG_BUFSIZE
    } else if requested < SCHEDULED_MSG_BUFSIZE {
        requested
    } else {
        SCHEDULED_MSG_BUFSIZE
    }
}

/// Check if a scheduled message type is valid.
#[no_mangle]
pub const extern "C" fn rs_is_valid_sched_msg_type(msg_type: c_int) -> c_int {
    (msg_type == SCHED_MSG_ERROR || msg_type == SCHED_MSG_MULTILINE) as c_int
}

/// Check if scheduled message type is multiline.
#[no_mangle]
pub const extern "C" fn rs_is_sched_msg_multiline(msg_type: c_int) -> c_int {
    (msg_type == SCHED_MSG_MULTILINE) as c_int
}

// ============================================================================
// Deferred Context State
// ============================================================================

/// Deferred execution context flags.
///
/// These indicate what kind of deferred context we're in.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeferredContext {
    /// In UI event handler
    pub in_ui_handler: bool,
    /// In decoration provider
    pub in_decor_provider: bool,
    /// In fast callback
    pub in_fast_callback: bool,
}

impl DeferredContext {
    /// Create a new empty context.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            in_ui_handler: false,
            in_decor_provider: false,
            in_fast_callback: false,
        }
    }

    /// Check if any deferred flag is set.
    #[must_use]
    pub const fn any_deferred(&self) -> bool {
        self.in_ui_handler || self.in_decor_provider || self.in_fast_callback
    }

    /// Check if messages should be scheduled.
    #[must_use]
    pub const fn should_schedule(&self) -> bool {
        self.any_deferred()
    }
}

impl Default for DeferredContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Get deferred context flags as packed integer.
///
/// Bit 0: in_ui_handler
/// Bit 1: in_decor_provider
/// Bit 2: in_fast_callback
#[no_mangle]
pub const extern "C" fn rs_deferred_context_pack(ctx: DeferredContext) -> c_int {
    let mut flags = 0;
    if ctx.in_ui_handler {
        flags |= 1;
    }
    if ctx.in_decor_provider {
        flags |= 2;
    }
    if ctx.in_fast_callback {
        flags |= 4;
    }
    flags
}

/// Unpack integer to deferred context flags.
#[no_mangle]
pub const extern "C" fn rs_deferred_context_unpack(flags: c_int) -> DeferredContext {
    DeferredContext {
        in_ui_handler: (flags & 1) != 0,
        in_decor_provider: (flags & 2) != 0,
        in_fast_callback: (flags & 4) != 0,
    }
}

/// Check if packed context indicates deferral needed.
#[no_mangle]
pub const extern "C" fn rs_deferred_context_any(flags: c_int) -> c_int {
    (flags != 0) as c_int
}

// ============================================================================
// Phase 3: msg_semsg_event / msg_semsg_multiline_event — migrated from message.c
// ============================================================================

/// HLF_E constant (error highlight group index).
/// Enum value 6: HLF_NONE=0, HLF_8=1, HLF_EOB=2, HLF_TERM=3, HLF_AT=4, HLF_D=5, HLF_E=6.
const HLF_E: c_int = 6;

/// Event callback: display a deferred error message and free the string.
///
/// Replaces C static `msg_semsg_event(void **argv)`.
///
/// # Safety
/// `argv[0]` must be a valid heap-allocated C string (`xstrdup` result).
#[no_mangle]
pub unsafe extern "C" fn rs_msg_semsg_event(argv: *mut *mut c_void) {
    let s = (*argv).cast::<c_char>();
    let _ = emsg(s);
    xfree((*argv).cast::<c_void>());
}

/// Event callback: display a deferred multiline error message and free the string.
///
/// Replaces C static `msg_semsg_multiline_event(void **argv)`.
///
/// # Safety
/// `argv[0]` must be a valid heap-allocated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_semsg_multiline_event(argv: *mut *mut c_void) {
    let s = (*argv).cast::<c_char>();
    emsg_multiline(s, c"emsg".as_ptr(), HLF_E, true);
    xfree((*argv).cast::<c_void>());
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Expected buffer size as c_int (8192 fits in i32)
    const EXPECTED_BUFSIZE: c_int = 8192;

    #[test]
    fn test_sched_msg_bufsize() {
        assert_eq!(rs_sched_msg_bufsize(0), EXPECTED_BUFSIZE);
        assert_eq!(rs_sched_msg_bufsize(-1), EXPECTED_BUFSIZE);
        assert_eq!(rs_sched_msg_bufsize(100), 100);
        assert_eq!(rs_sched_msg_bufsize(10000), EXPECTED_BUFSIZE);
    }

    #[test]
    fn test_sched_msg_type() {
        assert_eq!(rs_is_valid_sched_msg_type(SCHED_MSG_ERROR), 1);
        assert_eq!(rs_is_valid_sched_msg_type(SCHED_MSG_MULTILINE), 1);
        assert_eq!(rs_is_valid_sched_msg_type(99), 0);

        assert_eq!(rs_is_sched_msg_multiline(SCHED_MSG_ERROR), 0);
        assert_eq!(rs_is_sched_msg_multiline(SCHED_MSG_MULTILINE), 1);
    }

    #[test]
    fn test_deferred_context() {
        let ctx = DeferredContext::new();
        assert!(!ctx.any_deferred());
        assert!(!ctx.should_schedule());

        let ctx = DeferredContext {
            in_ui_handler: true,
            in_decor_provider: false,
            in_fast_callback: false,
        };
        assert!(ctx.any_deferred());
        assert!(ctx.should_schedule());
    }

    #[test]
    fn test_deferred_context_pack_unpack() {
        let ctx = DeferredContext {
            in_ui_handler: true,
            in_decor_provider: false,
            in_fast_callback: true,
        };
        let packed = rs_deferred_context_pack(ctx);
        assert_eq!(packed, 5); // bits 0 and 2

        let unpacked = rs_deferred_context_unpack(packed);
        assert_eq!(unpacked, ctx);
    }

    #[test]
    fn test_deferred_context_any() {
        assert_eq!(rs_deferred_context_any(0), 0);
        assert_eq!(rs_deferred_context_any(1), 1);
        assert_eq!(rs_deferred_context_any(7), 1);
    }
}
