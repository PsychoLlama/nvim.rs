//! Remote subcommand handling
//!
//! Implements `rs_remote_request` replacing the static C function in main.c.

use crate::setup::MparmT;
use std::ffi::{c_char, c_int};

// Values for window_layout (must match C enums in main.c)
const WIN_TABS: c_int = 3;

// TriState kTrue value
const K_TRUE: c_int = 1;

unsafe extern "C" {
    fn rs_server_connect(server_addr: *mut c_char, errmsg: *mut *const c_char) -> u64;
    // C helper: builds API types, calls vim._cs_remote.
    // Calls os_exit on any error.
    // Returns packed i64: high32=should_exit, low32=tabbed (-1/0/1 each).
    fn nvim_call_cs_remote(
        chan: u64,
        server_addr: *const c_char,
        connect_error: *const c_char,
        argc: c_int,
        argv: *mut *mut c_char,
        remote_args: c_int,
    ) -> i64;
    fn os_exit(r: c_int) -> !;
    fn os_getenv_noalloc(name: *const c_char) -> *const c_char;
    fn strequal(a: *const c_char, b: *const c_char) -> bool;
    fn fprintf(stream: *mut std::ffi::c_void, fmt: *const c_char, ...) -> c_int;

    static stderr: *mut std::ffi::c_void;
    static mut ui_client_channel_id: u64;
}

/// Handle remote subcommands.
///
/// # Safety
/// All pointer arguments must be valid for the duration of the call.
#[no_mangle]
pub unsafe extern "C" fn rs_remote_request(
    params: *mut MparmT,
    remote_args: c_int,
    server_addr: *mut c_char,
    argc: c_int,
    argv: *mut *mut c_char,
    ui_only: bool,
) {
    let is_ui = strequal(*argv.add(remote_args as usize), c"--remote-ui".as_ptr());

    if ui_only && !is_ui {
        // TODO(bfredl): this implies always starting the TUI.
        return;
    }

    let mut connect_error: *const c_char = std::ptr::null();
    let chan = rs_server_connect(server_addr, &mut connect_error);

    if is_ui {
        if chan == 0 {
            fprintf(
                stderr,
                c"Remote ui failed to start: %s\n".as_ptr(),
                connect_error,
            );
            os_exit(1);
        } else {
            let nvim_env = os_getenv_noalloc(c"NVIM".as_ptr());
            if strequal(server_addr, nvim_env) {
                fprintf(
                    stderr,
                    c"%s".as_ptr(),
                    c"Cannot attach UI of :terminal child to its parent. ".as_ptr(),
                );
                fprintf(
                    stderr,
                    c"%s\n".as_ptr(),
                    c"(Unset $NVIM to skip this check)".as_ptr(),
                );
                os_exit(1);
            }
        }
        ui_client_channel_id = chan;
        return;
    }

    // Delegate Lua call and result parsing to C helper (avoids API type complexity in Rust)
    let packed = nvim_call_cs_remote(chan, server_addr, connect_error, argc, argv, remote_args);
    // Unpack: high32=should_exit, low32=tabbed (-1/0/1 each)
    let should_exit = (packed >> 32) as c_int;
    let tabbed = (packed & 0xFFFF_FFFF) as i32 as c_int;

    if should_exit == K_TRUE {
        os_exit(0);
    }
    if tabbed == K_TRUE {
        let p = &mut *params;
        p.window_count = argc - remote_args - 1;
        p.window_layout = WIN_TABS;
    }
}
