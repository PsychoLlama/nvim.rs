//! do_cmdline: getline_equal and getline_cookie helpers.
//!
//! `do_cmdline` itself remains in C (Phase 5 deferred - too complex for single-pass port).
//! `getline_equal` and `getline_cookie` are implemented here in Rust.

use std::ffi::c_void;

use crate::do_one_cmd::LineGetter;

/// Compare two `LineGetter` (Option<fn>) values by raw address.
fn linegetter_eq(a: LineGetter, b: LineGetter) -> bool {
    match (a, b) {
        (Some(fa), Some(fb)) => std::ptr::fn_addr_eq(fa, fb),
        (None, None) => true,
        _ => false,
    }
}

// =============================================================================
// FFI declarations (minimal - only what getline_equal/getline_cookie need)
// =============================================================================

extern "C" {
    fn nvim_docmd_get_loop_line_ptr() -> LineGetter;
    fn nvim_docmd_loop_cookie_get_lc_getline(lc: *mut c_void) -> LineGetter;
    fn nvim_docmd_loop_cookie_get_cookie(lc: *mut c_void) -> *mut c_void;
}

// =============================================================================
// getline_equal / getline_cookie
// =============================================================================

/// If `fgetline` is `get_loop_line()`, return true if the getline it uses
/// equals `func`. Otherwise return true when `fgetline` equals `func`.
///
/// # Safety
///
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn getline_equal(
    fgetline: LineGetter,
    cookie: *mut c_void,
    func: LineGetter,
) -> bool {
    let loop_line_ptr = nvim_docmd_get_loop_line_ptr();
    let mut gp = fgetline;
    let mut cp = cookie;

    // When fgetline is get_loop_line() use the cookie to find the function
    // that's originally used to obtain the lines.
    while linegetter_eq(gp, loop_line_ptr) {
        let new_gp = nvim_docmd_loop_cookie_get_lc_getline(cp);
        let new_cp = nvim_docmd_loop_cookie_get_cookie(cp);
        gp = new_gp;
        cp = new_cp;
    }
    linegetter_eq(gp, func)
}

/// If `fgetline` is `get_loop_line()`, return the cookie used by the original
/// getline function. Otherwise return `cookie`.
///
/// # Safety
///
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn getline_cookie(fgetline: LineGetter, cookie: *mut c_void) -> *mut c_void {
    let loop_line_ptr = nvim_docmd_get_loop_line_ptr();
    let mut gp = fgetline;
    let mut cp = cookie;

    while linegetter_eq(gp, loop_line_ptr) {
        let new_gp = nvim_docmd_loop_cookie_get_lc_getline(cp);
        let new_cp = nvim_docmd_loop_cookie_get_cookie(cp);
        gp = new_gp;
        cp = new_cp;
    }
    cp
}
