//! Job helper functions migrated from eval_shim.c (Phase 2, eval_shim pass 8).
//!
//! Implements `common_job_callbacks` and `find_job`.

use std::ffi::{c_char, c_int, c_void};

use super::typval::{CallbackReaderT, CallbackT};

// =============================================================================
// Opaque handle types
// =============================================================================

/// Opaque pointer to dict_T.
type DictHandle = *mut c_void;
/// Opaque pointer to Channel.
type ChannelHandle = *mut c_void;

type CallbackHandle = *mut CallbackT;

// =============================================================================
// C Extern Declarations
// =============================================================================

extern "C" {
    // -- tv_dict_get_callback wrapper --
    #[link_name = "tv_dict_get_callback"]
    fn nvim_tv_dict_get_callback(
        dict: DictHandle,
        key: *const c_char,
        key_len: isize,
        result: CallbackHandle,
    ) -> bool;

    // -- tv_dict_get_number wrapper --
    #[link_name = "tv_dict_get_number"]
    fn nvim_tv_dict_get_number(dict: DictHandle, key: *const c_char) -> i64;

    #[link_name = "callback_reader_free"]
    fn nvim_callback_reader_free(reader: *mut CallbackReaderT);

    // -- Callback free (now defined in Rust eval_exec/callback.rs, takes *mut c_void) --
    fn nvim_callback_free(cb: *mut c_void);

    // -- Dict refcount --
    fn nvim_dict_refcount_inc(dict: DictHandle);

    // -- Channel accessors --
    fn nvim_find_channel(id: u64) -> ChannelHandle;
    fn nvim_channel_is_valid_job(chan: ChannelHandle) -> c_int;
    fn nvim_channel_is_not_proc(chan: ChannelHandle) -> c_int;

    // -- Error messages: now in crate::errors --
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
    on_stdout: *mut CallbackReaderT,
    on_stderr: *mut CallbackReaderT,
    on_exit: CallbackHandle,
) -> bool {
    let stdout_cb = &raw mut (*on_stdout).cb;
    let stderr_cb = &raw mut (*on_stderr).cb;

    if nvim_tv_dict_get_callback(vopts, c"on_stdout".as_ptr(), 9, stdout_cb)
        && nvim_tv_dict_get_callback(vopts, c"on_stderr".as_ptr(), 9, stderr_cb)
        && nvim_tv_dict_get_callback(vopts, c"on_exit".as_ptr(), 7, on_exit)
    {
        let stdout_buffered = nvim_tv_dict_get_number(vopts, c"stdout_buffered".as_ptr()) != 0;
        let stderr_buffered = nvim_tv_dict_get_number(vopts, c"stderr_buffered".as_ptr()) != 0;

        (*on_stdout).buffered = stdout_buffered;
        (*on_stderr).buffered = stderr_buffered;

        // Direct field access: replaces nvim_eval_cb_get_type
        if stdout_buffered && (*stdout_cb).cb_type == K_CALLBACK_NONE {
            (*on_stdout).self_ = vopts;
        }
        if stderr_buffered && (*stderr_cb).cb_type == K_CALLBACK_NONE {
            (*on_stderr).self_ = vopts;
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
                crate::errors::emsg_invchanjob();
            } else {
                crate::errors::emsg_invchan();
            }
        }
        return std::ptr::null_mut();
    }
    data
}
