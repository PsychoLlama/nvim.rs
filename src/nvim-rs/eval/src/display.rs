//! Display utilities for VimL evaluation.
//!
//! This module provides `last_set_msg`, which displays the script name and
//! line where a setting was last changed. Equivalent to the C `last_set_msg`
//! in `eval_shim.c`.

use std::ffi::{c_char, c_int};

// =============================================================================
// C Accessor Extern Declarations
// =============================================================================

extern "C" {
    // get_scriptname delegate (in runtime_ffi.c)
    fn nvim_rt_get_scriptname(sc_sid: c_int, sc_chan: u64, should_free: *mut bool) -> *mut c_char;

    // script_is_lua (implemented in runtime/src/script.rs, exposed as rs_script_is_lua)
    fn script_is_lua(sid: c_int) -> bool;

    // verbose helpers (in message.c)
    fn verbose_enter();
    fn verbose_leave();

    // message output
    fn msg_puts(s: *const c_char);
    fn msg_outnum(n: c_int);

    // gettext / translation
    fn nvim_gettext(s: *const c_char) -> *const c_char;

    // memory deallocation
    fn xfree(ptr: *mut std::ffi::c_void);
}

// =============================================================================
// String constants (matching C `line_msg` and other literals)
// =============================================================================

/// "\n\tLast set from " -- matches C _(...)
const MSG_LAST_SET_FROM: &[u8] = b"\n\tLast set from \0";

/// " line " -- matches C line_msg constant
const MSG_LINE: &[u8] = b" line \0";

/// " (run Nvim with -V1 for more details)" -- lua script hint
const MSG_LUA_HINT: &[u8] = b" (run Nvim with -V1 for more details)\0";

// =============================================================================
// Phase 1: last_set_msg
// =============================================================================

/// Display the script name where an item was last set.
///
/// Should only be invoked when 'verbose' is non-zero. Takes the two relevant
/// fields of `sctx_T` as scalars to avoid passing the full struct over FFI.
///
/// # Safety
///
/// Calls multiple C functions that access global state.
#[no_mangle]
pub unsafe extern "C" fn rs_last_set_msg(sc_sid: c_int, sc_lnum: c_int, sc_chan: u64) {
    if sc_sid == 0 {
        return;
    }

    let mut should_free: bool = false;
    let p = nvim_rt_get_scriptname(sc_sid, sc_chan, &raw mut should_free);

    verbose_enter();

    // "\n\tLast set from "
    let header = nvim_gettext(MSG_LAST_SET_FROM.as_ptr().cast::<c_char>());
    msg_puts(header);
    msg_puts(p);

    if sc_lnum > 0 {
        // " line "
        let line_str = nvim_gettext(MSG_LINE.as_ptr().cast::<c_char>());
        msg_puts(line_str);
        msg_outnum(sc_lnum);
    } else if script_is_lua(sc_sid) {
        // " (run Nvim with -V1 for more details)"
        let hint = nvim_gettext(MSG_LUA_HINT.as_ptr().cast::<c_char>());
        msg_puts(hint);
    }

    if should_free {
        xfree(p.cast::<std::ffi::c_void>());
    }

    verbose_leave();
}
