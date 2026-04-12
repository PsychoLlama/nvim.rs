//! UI client event handlers migrated from C
//!
//! Phase 1 and Phase 2 functions.

use std::ffi::{c_char, c_void};

use nvim_api::{Array, Object, ObjectType};

// Opaque type handle for Error
type Error = c_void;

// ObjectType constants (matching C kObjectType* enum)
const K_OBJECT_TYPE_INTEGER: i32 = ObjectType::Integer as i32;
const K_OBJECT_TYPE_STRING: i32 = ObjectType::String as i32;

// C accessors declared in ui_client.c
extern "C" {
    fn nvim_uic_rpc_send_detach();
    fn nvim_uic_set_attached(value: bool);
    fn nvim_uic_tui_is_stopped() -> bool;
    fn nvim_uic_tui_stop();
    fn nvim_uic_api_set_error_validation(err: *mut Error, msg: *const c_char);
    fn nvim_uic_set_error_exit(value: i32);
    fn nvim_uic_set_channel_id(value: u64);
    fn nvim_uic_queue_channel_connect(server_addr: *mut c_char);
    fn nvim_uic_tui_grid_resize(grid: i64, width: i64, height: i64);
    fn nvim_uic_grid_line_buf_realloc(new_size: usize);
    fn nvim_uic_get_grid_line_buf_size() -> usize;
    fn nvim_uic_save_restart_args(args: Array);
}

// ============================================================================
// Phase 1: Simple stub functions
// ============================================================================

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

// ============================================================================
// Phase 2: Event handler functions
// ============================================================================

/// Handle `error_exit` UI event: set the client error exit code.
///
/// Replaces C `ui_client_event_error_exit`.
///
/// # Safety
/// `args` must be a valid C `Array` passed from the event dispatch system.
#[unsafe(export_name = "ui_client_event_error_exit")]
pub unsafe extern "C" fn rs_ui_client_event_error_exit(args: Array) {
    unsafe {
        if args.size < 1 || (*args.items).obj_type != K_OBJECT_TYPE_INTEGER {
            // ELOG is not available from Rust here; silently return on bad args.
            return;
        }
        let exit_code = i32::try_from((*args.items).data.integer).unwrap_or(0);
        nvim_uic_set_error_exit(exit_code);
    }
}

/// Handle 'connect' UI event: queue a channel connect and update channel ID.
///
/// Replaces C `ui_client_event_connect`.
///
/// # Safety
/// `args` must be a valid C `Array` from the event dispatch system.
#[unsafe(export_name = "ui_client_event_connect")]
pub unsafe extern "C" fn rs_ui_client_event_connect(args: Array) {
    unsafe {
        if args.size < 1 || (*args.items).obj_type != K_OBJECT_TYPE_STRING {
            return;
        }
        let server_addr = (*args.items).data.string.data;
        nvim_uic_queue_channel_connect(server_addr);
        // Set a dummy channel ID to prevent client exit when server detaches.
        nvim_uic_set_channel_id(u64::MAX);
    }
}

/// Handle 'restart' UI event: save restart arguments for later use.
///
/// Replaces C `ui_client_event_restart`.
///
/// # Safety
/// `args` must be a valid C `Array` from the event dispatch system.
#[unsafe(export_name = "ui_client_event_restart")]
pub unsafe extern "C" fn rs_ui_client_event_restart(args: Array) {
    // NB: don't send nvim_ui_detach to server, as it may have already exited.
    // Save the arguments for ui_client_may_restart_server() later.
    unsafe {
        nvim_uic_save_restart_args(args);
    }
}

/// Handle `grid_resize` UI event: resize the TUI grid and reallocate buffers.
///
/// Replaces C `ui_client_event_grid_resize`.
///
/// # Safety
/// `args` must be a valid C `Array` from the event dispatch system.
#[unsafe(export_name = "ui_client_event_grid_resize")]
pub unsafe extern "C" fn rs_ui_client_event_grid_resize(args: Array) {
    unsafe {
        if args.size < 3
            || (*args.items).obj_type != K_OBJECT_TYPE_INTEGER
            || (*args.items.add(1)).obj_type != K_OBJECT_TYPE_INTEGER
            || (*args.items.add(2)).obj_type != K_OBJECT_TYPE_INTEGER
        {
            return;
        }
        let grid = (*args.items).data.integer;
        let width = (*args.items.add(1)).data.integer;
        let height = (*args.items.add(2)).data.integer;
        nvim_uic_tui_grid_resize(grid, width, height);

        let width_usize = usize::try_from(width).unwrap_or(0);
        if nvim_uic_get_grid_line_buf_size() < width_usize {
            nvim_uic_grid_line_buf_realloc(width_usize);
        }
    }
}
