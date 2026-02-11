//! Find patterns in included files: `find_pattern_in_path()` and helpers.
//!
//! Migrated from search.c. Powers `[I`, `[i`, `[D`, `[d`, `CTRL-W i`, etc.
//!
//! Due to the extremely wide dependency surface (~40 external calls spanning
//! regexp, file I/O, completion API, window management, and messaging), this
//! module delegates most work to batch C helper functions. The Rust code
//! handles the top-level lifecycle (init / run / cleanup).

use std::ffi::{c_char, c_int, c_void};

type LinenrT = i32;

/// Opaque handle for the SearchedFile stack + regex state managed in C.
type FpipHandle = *mut c_void;

/// Result from the C-side initialization batch helper.
#[repr(C)]
struct FpipInitResult {
    /// Handle to the opaque state (NULL on failure).
    handle: FpipHandle,
    /// Whether initialization succeeded.
    ok: c_int,
}

extern "C" {
    /// Initialize the fpip state: compile regexps, allocate file stack,
    /// set up the first line. Returns opaque handle.
    fn nvim_fpip_init(
        ptr: *const c_char,
        dir: c_int,
        len: usize,
        whole: c_int,
        skip_comments: c_int,
        typ: c_int,
        count: c_int,
        action: c_int,
        start_lnum: LinenrT,
        end_lnum: LinenrT,
        forceit: c_int,
        silent: c_int,
    ) -> FpipInitResult;

    /// Execute the full main loop of find_pattern_in_path.
    fn nvim_fpip_run(handle: FpipHandle);

    /// Clean up: close files, free regexps, free the state handle.
    fn nvim_fpip_cleanup(handle: FpipHandle);
}

/// Rust implementation of find_pattern_in_path().
///
/// Due to the extremely wide dependency surface, this function delegates
/// the entire operation to C batch helpers that manage the regexp state,
/// file stack, and all the action-specific logic.
///
/// # Safety
/// All pointer arguments must be valid for the duration of the call.
#[no_mangle]
pub unsafe extern "C" fn rs_find_pattern_in_path(
    ptr: *const c_char,
    dir: c_int,
    len: usize,
    whole: c_int,
    skip_comments: c_int,
    typ: c_int,
    count: c_int,
    action: c_int,
    start_lnum: LinenrT,
    end_lnum: LinenrT,
    forceit: c_int,
    silent: c_int,
) {
    let init = nvim_fpip_init(
        ptr,
        dir,
        len,
        whole,
        skip_comments,
        typ,
        count,
        action,
        start_lnum,
        end_lnum,
        forceit,
        silent,
    );

    if init.ok == 0 || init.handle.is_null() {
        // Init failed (e.g., regex compilation error)
        if !init.handle.is_null() {
            nvim_fpip_cleanup(init.handle);
        }
        return;
    }

    // Run the main search loop
    nvim_fpip_run(init.handle);

    // Cleanup
    nvim_fpip_cleanup(init.handle);
}
