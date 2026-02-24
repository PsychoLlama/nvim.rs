//! Job helper functions migrated from eval_shim.c (Phase 2, eval_shim pass 8).
//!
//! Implements `common_job_callbacks` and `find_job`.

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Opaque handle types
// =============================================================================

/// Opaque pointer to dict_T.
type DictHandle = *mut c_void;
/// Opaque pointer to Callback.
type CallbackHandle = *mut c_void;
/// Opaque pointer to CallbackReader.
type CallbackReaderHandle = *mut c_void;
/// Opaque pointer to Channel.
type ChannelHandle = *mut c_void;

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
    fn nvim_cbr_get_cb_ptr(reader: CallbackReaderHandle) -> CallbackHandle;
    fn nvim_cbr_set_buffered(reader: CallbackReaderHandle, buffered: c_int);
    fn nvim_cbr_set_self(reader: CallbackReaderHandle, dict: DictHandle);
    fn nvim_callback_reader_free(reader: CallbackReaderHandle);

    // -- Callback type accessor --
    fn nvim_eval_cb_get_type(cb: CallbackHandle) -> c_int;

    // -- Callback free --
    fn nvim_callback_free(cb: CallbackHandle);

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
#[no_mangle]
pub unsafe extern "C" fn rs_common_job_callbacks(
    vopts: DictHandle,
    on_stdout: CallbackReaderHandle,
    on_stderr: CallbackReaderHandle,
    on_exit: CallbackHandle,
) -> bool {
    let stdout_cb = nvim_cbr_get_cb_ptr(on_stdout);
    let stderr_cb = nvim_cbr_get_cb_ptr(on_stderr);

    if nvim_tv_dict_get_callback(vopts, c"on_stdout".as_ptr(), 9, stdout_cb)
        && nvim_tv_dict_get_callback(vopts, c"on_stderr".as_ptr(), 9, stderr_cb)
        && nvim_tv_dict_get_callback(vopts, c"on_exit".as_ptr(), 7, on_exit)
    {
        let stdout_buffered = nvim_tv_dict_get_number(vopts, c"stdout_buffered".as_ptr()) != 0;
        let stderr_buffered = nvim_tv_dict_get_number(vopts, c"stderr_buffered".as_ptr()) != 0;

        nvim_cbr_set_buffered(on_stdout, stdout_buffered as c_int);
        nvim_cbr_set_buffered(on_stderr, stderr_buffered as c_int);

        if stdout_buffered && nvim_eval_cb_get_type(stdout_cb) == K_CALLBACK_NONE {
            nvim_cbr_set_self(on_stdout, vopts);
        }
        if stderr_buffered && nvim_eval_cb_get_type(stderr_cb) == K_CALLBACK_NONE {
            nvim_cbr_set_self(on_stderr, vopts);
        }

        nvim_dict_refcount_inc(vopts);
        return true;
    }

    nvim_callback_reader_free(on_stdout);
    nvim_callback_reader_free(on_stderr);
    nvim_callback_free(on_exit);
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
#[no_mangle]
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
