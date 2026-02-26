//! Display utilities for VimL evaluation.
//!
//! This module provides `last_set_msg`, which displays the script name and
//! line where a setting was last changed. Equivalent to the C `last_set_msg`
//! in `eval_shim.c`.
//!
//! Phase 12 update: changed from rs_last_set_msg(sc_sid, sc_lnum, sc_chan) scalars
//! to last_set_msg(SctxT) by-value struct, eliminating the C wrapper.

use std::ffi::{c_char, c_int};

// =============================================================================
// SctxT: #[repr(C)] mirror of C's sctx_T struct
//
// C definition (eval/typval_defs.h lines 279-290):
//   typedef struct {
//     scid_T sc_sid;     // int, offset 0
//     int sc_seq;        // int, offset 4
//     linenr_T sc_lnum;  // int32_t, offset 8
//     // 4 bytes implicit padding for sc_chan alignment
//     uint64_t sc_chan;  // offset 16
//   } sctx_T;            // sizeof = 24
//
// Validated by _Static_assert(sizeof(sctx_T) == 24) in eval_shim.c.
// =============================================================================

/// Rust mirror of C `sctx_T` (24 bytes).
#[repr(C)]
pub struct SctxT {
    /// Script ID (scid_T = int).
    pub sc_sid: c_int,
    /// Sourcing sequence number.
    pub sc_seq: c_int,
    /// Line number in script (linenr_T = int32_t).
    pub sc_lnum: i32,
    // 4 bytes implicit padding here (repr(C) aligns sc_chan to 8 bytes)
    /// Channel ID (only used when sc_sid is SID_API_CLIENT).
    pub sc_chan: u64,
}

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
// Phase 12: last_set_msg (was rs_last_set_msg with scalars, now by-value SctxT)
// =============================================================================

/// Display the script name where an item was last set.
///
/// Should only be invoked when 'verbose' is non-zero.
///
/// Exported as `last_set_msg` (replaces C wrapper in eval_shim.c).
/// Accepts `sctx_T` by value via `SctxT` (`#[repr(C)]` struct).
///
/// # Safety
///
/// Calls multiple C functions that access global state.
#[export_name = "last_set_msg"]
pub unsafe extern "C" fn rs_last_set_msg(script_ctx: SctxT) {
    let sc_sid = script_ctx.sc_sid;
    let sc_lnum = script_ctx.sc_lnum;
    let sc_chan = script_ctx.sc_chan;

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
