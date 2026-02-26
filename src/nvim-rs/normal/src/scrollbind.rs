//! Scroll-binding: `do_check_scrollbind` and `check_scrollbind` migrated to Rust.
//!
//! Phase 3 of the normal_shim.c migration. The window iteration loop body
//! stays in C (via `nvim_scrollbind_sync_windows`) because it temporarily
//! swaps the `curwin`/`curbuf` globals, which is unsafe to replicate in Rust.

use std::ffi::c_int;
use std::sync::atomic::{AtomicI32, Ordering};

use crate::{BufHandle, WinHandle};

// Static variables corresponding to the C `static` locals in do_check_scrollbind.
// Using AtomicI32 for vtopline/leftcol; raw pointers stored as usize for win/buf handles.
// All accesses are from a single thread (Neovim is single-threaded for these calls).
static OLD_CURWIN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static OLD_VTOPLINE: AtomicI32 = AtomicI32::new(0);
static OLD_BUF: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static OLD_LEFTCOL: AtomicI32 = AtomicI32::new(0);

extern "C" {
    fn nvim_get_did_syncbind() -> bool;
    fn nvim_set_did_syncbind(val: bool);
    fn nvim_curwin_get_p_scb() -> bool;
    fn nvim_get_curwin() -> WinHandle;
    fn nvim_get_curbuf() -> BufHandle;
    fn nvim_curwin_eq(wp: WinHandle) -> bool;
    fn nvim_curwin_buf_eq(buf: BufHandle) -> bool;
    fn nvim_curwin_get_w_p_diff() -> bool;
    fn nvim_get_curwin_w_leftcol() -> c_int;
    fn nvim_curwin_get_w_scbind_pos() -> c_int;
    fn nvim_curwin_set_w_scbind_pos(val: c_int);
    fn nvim_vim_strchr_p_sbo(c: c_int) -> bool;
    fn nvim_scrollbind_sync_windows(
        old_curwin: WinHandle,
        vtopline_diff: c_int,
        tgt_leftcol: c_int,
        want_ver: bool,
        want_hor: bool,
    );
}

/// When `check` is false, prepare for commands that scroll the window.
/// When `check` is true, take care of scroll-binding after the window has scrolled.
///
/// Rust replacement for the C `do_check_scrollbind` function. Manages the four
/// function-static variables that track the previous window state.
///
/// # Safety
/// Caller must ensure curwin/curbuf globals are valid.
#[no_mangle]
pub unsafe extern "C" fn rs_do_check_scrollbind(check: bool) {
    let vtopline = crate::rs_get_vtopline(nvim_get_curwin());

    if check && nvim_curwin_get_p_scb() {
        let old_curwin = OLD_CURWIN.load(Ordering::Relaxed) as WinHandle;
        let old_buf = OLD_BUF.load(Ordering::Relaxed) as BufHandle;
        let old_vtopline = OLD_VTOPLINE.load(Ordering::Relaxed);
        let old_leftcol = OLD_LEFTCOL.load(Ordering::Relaxed);

        if nvim_get_did_syncbind() {
            // ":syncbind" was just used: reset values, don't scroll.
            nvim_set_did_syncbind(false);
        } else if nvim_curwin_eq(old_curwin) {
            // Same window: sync if buffer or diff matches and position changed.
            if (nvim_curwin_buf_eq(old_buf) || nvim_curwin_get_w_p_diff())
                && (vtopline != old_vtopline || nvim_get_curwin_w_leftcol() != old_leftcol)
            {
                rs_check_scrollbind(
                    vtopline - old_vtopline,
                    nvim_get_curwin_w_leftcol() - old_leftcol,
                );
            }
        } else if nvim_vim_strchr_p_sbo('j' as c_int) {
            // Window switch: resync relative position if 'j' flag set in 'scrollopt'.
            rs_check_scrollbind(vtopline - nvim_curwin_get_w_scbind_pos(), 0);
        }
        nvim_curwin_set_w_scbind_pos(vtopline);
    }

    // Update static state.
    OLD_CURWIN.store(nvim_get_curwin() as usize, Ordering::Relaxed);
    OLD_VTOPLINE.store(vtopline, Ordering::Relaxed);
    OLD_BUF.store(nvim_get_curbuf() as usize, Ordering::Relaxed);
    OLD_LEFTCOL.store(nvim_get_curwin_w_leftcol(), Ordering::Relaxed);
}

/// Synchronize scroll-bound windows based on scroll delta.
///
/// Computes `want_ver`/`want_hor` from 'scrollopt', then delegates window
/// iteration (with curwin/curbuf swapping) to `nvim_scrollbind_sync_windows`.
///
/// # Safety
/// Caller must ensure curwin/curbuf globals and all window pointers are valid.
#[no_mangle]
pub unsafe extern "C" fn rs_check_scrollbind(vtopline_diff: c_int, leftcol_diff: c_int) {
    let old_curwin = nvim_get_curwin();
    let tgt_leftcol = nvim_get_curwin_w_leftcol();

    // Check 'scrollopt' for vertical and horizontal scroll flags.
    let want_ver =
        nvim_curwin_get_w_p_diff() || (nvim_vim_strchr_p_sbo('v' as c_int) && vtopline_diff != 0);
    let want_hor = nvim_vim_strchr_p_sbo('h' as c_int) && (leftcol_diff != 0 || vtopline_diff != 0);

    // Delegate window iteration (curwin/curbuf swapping) to C compound accessor.
    nvim_scrollbind_sync_windows(old_curwin, vtopline_diff, tgt_leftcol, want_ver, want_hor);
}
