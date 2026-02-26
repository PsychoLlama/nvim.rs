//! Job helper functions migrated from eval_shim.c (Phase 2, eval_shim pass 8).
//!
//! Implements `common_job_callbacks` and `find_job`.

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Opaque handle types
// =============================================================================

/// Opaque pointer to dict_T.
type DictHandle = *mut c_void;
/// Opaque pointer to CallbackReader.
type CallbackReaderHandle = *mut c_void;
/// Opaque pointer to Channel.
type ChannelHandle = *mut c_void;

// CallbackT: Rust mirror of C Callback struct (16 bytes).
// Layout validated by _Static_assert in eval_shim.c.
#[repr(C)]
pub union CallbackData {
    pub funcref: *mut std::ffi::c_char,
    pub partial: *mut c_void,
    pub luaref: std::ffi::c_int,
}

#[repr(C)]
pub struct CallbackT {
    data: CallbackData,
    cb_type: std::ffi::c_int,
    // 4 bytes trailing padding
}

type CallbackHandle = *mut CallbackT;

// =============================================================================
// C Extern Declarations
// =============================================================================

extern "C" {
    // -- tv_dict_get_callback wrapper --
    fn nvim_tv_dict_get_callback(
        dict: DictHandle,
        key: *const c_char,
        key_len: isize,
        result: CallbackHandle,
    ) -> bool;

    // -- tv_dict_get_number wrapper --
    fn nvim_tv_dict_get_number(dict: DictHandle, key: *const c_char) -> i64;

    // -- CallbackReader accessors --
    fn nvim_eval_cbr_get_cb(reader: CallbackReaderHandle) -> CallbackHandle;
    fn nvim_cbr_set_buffered(reader: CallbackReaderHandle, buffered: c_int);
    fn nvim_cbr_set_self(reader: CallbackReaderHandle, dict: DictHandle);
    fn nvim_callback_reader_free(reader: CallbackReaderHandle);

    // -- Callback free (now defined in Rust eval_exec/callback.rs, takes *mut c_void) --
    fn nvim_callback_free(cb: *mut c_void);

    // -- Dict refcount --
    fn nvim_dict_refcount_inc(dict: DictHandle);

    // -- Channel accessors --
    fn nvim_find_channel(id: u64) -> ChannelHandle;
    fn nvim_channel_is_valid_job(chan: ChannelHandle) -> c_int;
    fn nvim_channel_is_not_proc(chan: ChannelHandle) -> c_int;

    // -- Error messages --
    fn nvim_emsg_invchan();
    fn nvim_emsg_invchanjob();
}

// kCallbackNone == 0 (verified by _Static_assert in eval_shim.c)
const K_CALLBACK_NONE: c_int = 0;
// Note: callback_depth check removed -- use direct CallbackT field access

// =============================================================================
// Phase 2: common_job_callbacks
// =============================================================================

/// Extract job callbacks (on_stdout, on_stderr, on_exit) from an options dict.
///
/// Returns true on success, false on failure (callbacks are freed on failure).
///
/// # Safety
/// - `vopts` must be a valid dict_T pointer.
/// - `on_stdout`, `on_stderr` must be valid CallbackReader pointers.
/// - `on_exit` must be a valid Callback pointer.
#[export_name = "common_job_callbacks"]
pub unsafe extern "C" fn rs_common_job_callbacks(
    vopts: DictHandle,
    on_stdout: CallbackReaderHandle,
    on_stderr: CallbackReaderHandle,
    on_exit: CallbackHandle,
) -> bool {
    let stdout_cb = nvim_eval_cbr_get_cb(on_stdout);
    let stderr_cb = nvim_eval_cbr_get_cb(on_stderr);

    if nvim_tv_dict_get_callback(vopts, c"on_stdout".as_ptr(), 9, stdout_cb)
        && nvim_tv_dict_get_callback(vopts, c"on_stderr".as_ptr(), 9, stderr_cb)
        && nvim_tv_dict_get_callback(vopts, c"on_exit".as_ptr(), 7, on_exit)
    {
        let stdout_buffered = nvim_tv_dict_get_number(vopts, c"stdout_buffered".as_ptr()) != 0;
        let stderr_buffered = nvim_tv_dict_get_number(vopts, c"stderr_buffered".as_ptr()) != 0;

        nvim_cbr_set_buffered(on_stdout, c_int::from(stdout_buffered));
        nvim_cbr_set_buffered(on_stderr, c_int::from(stderr_buffered));

        // Direct field access: replaces nvim_eval_cb_get_type
        if stdout_buffered && (*stdout_cb).cb_type == K_CALLBACK_NONE {
            nvim_cbr_set_self(on_stdout, vopts);
        }
        if stderr_buffered && (*stderr_cb).cb_type == K_CALLBACK_NONE {
            nvim_cbr_set_self(on_stderr, vopts);
        }

        nvim_dict_refcount_inc(vopts);
        return true;
    }

    nvim_callback_reader_free(on_stdout);
    nvim_callback_reader_free(on_stderr);
    nvim_callback_free(on_exit.cast::<c_void>());
    false
}

// =============================================================================
// Phase 2: find_job
// =============================================================================

/// Find a job Channel by ID, verifying it is a running proc stream.
///
/// Returns the Channel pointer on success, NULL on failure.
/// If `show_error` is true, emits an appropriate error message on failure.
///
/// # Safety
/// Safe to call from C.
#[must_use]
#[export_name = "find_job"]
pub unsafe extern "C" fn rs_find_job(id: u64, show_error: bool) -> ChannelHandle {
    let data = nvim_find_channel(id);
    if nvim_channel_is_valid_job(data) == 0 {
        if show_error {
            if nvim_channel_is_not_proc(data) != 0 {
                nvim_emsg_invchanjob();
            } else {
                nvim_emsg_invchan();
            }
        }
        return std::ptr::null_mut();
    }
    data
}
