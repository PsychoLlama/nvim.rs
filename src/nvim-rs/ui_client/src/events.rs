//! UI client event handlers migrated from C
//!
//! Phase 1: Simple stub functions and state accessors.

use std::ffi::{c_char, c_void};

use nvim_api::{Array, Object};

// Opaque type handle for Error
type Error = c_void;

// C accessors declared in ui_client.c
extern "C" {
    fn nvim_uic_rpc_send_detach();
    fn nvim_uic_set_attached(value: bool);
    fn nvim_uic_tui_is_stopped() -> bool;
    fn nvim_uic_tui_stop();
    fn nvim_uic_api_set_error_validation(err: *mut Error, msg: *const c_char);
}

/// Send detach event and mark UI client as detached.
///
/// Replaces C `ui_client_detach`.
///
/// # Safety
/// Must be called from C; relies on internal C state for the RPC channel.
#[unsafe(export_name = "ui_client_detach")]
pub unsafe extern "C" fn rs_ui_client_detach() {
    unsafe {
        nvim_uic_rpc_send_detach();
        nvim_uic_set_attached(false);
    }
}

/// Stop the TUI if it is still running, and mark UI client as detached.
///
/// Replaces C `ui_client_stop`.
///
/// # Safety
/// Must be called from C; accesses TUI state via C accessors.
#[unsafe(export_name = "ui_client_stop")]
pub unsafe extern "C" fn rs_ui_client_stop() {
    unsafe {
        nvim_uic_set_attached(false);
        if !nvim_uic_tui_is_stopped() {
            nvim_uic_tui_stop();
        }
    }
}

/// Abort stub for `grid_line` events -- this path is unreachable in normal operation.
///
/// Replaces C `ui_client_event_grid_line`.
///
/// # Safety
/// Calls `abort()` unconditionally; this function is unreachable.
#[unsafe(export_name = "ui_client_event_grid_line")]
pub unsafe extern "C" fn rs_ui_client_event_grid_line(_args: Array) -> ! {
    std::process::abort();
}

/// Placeholder for _sync_ requests with 'redraw' method name.
///
/// Async 'redraw' events are handled directly in `msgpack_rpc/unpacker.c`.
/// This handler is called only when 'redraw' is sent as a synchronous request,
/// which is invalid -- return a validation error.
///
/// Replaces C `handle_ui_client_redraw`.
///
/// # Safety
/// `error` must be a valid pointer to a C `Error` struct, or null.
#[unsafe(export_name = "handle_ui_client_redraw")]
pub unsafe extern "C" fn rs_handle_ui_client_redraw(
    _channel_id: u64,
    _args: Array,
    _arena: *mut c_void,
    error: *mut Error,
) -> Object {
    unsafe {
        nvim_uic_api_set_error_validation(error, c"'redraw' cannot be sent as a request".as_ptr());
    }
    Object::nil()
}
