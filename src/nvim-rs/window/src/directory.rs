//! Window directory management (`win_fix_current_dir`).
//!
//! This module provides `rs_win_fix_current_dir`, the Rust replacement for
//! `win_fix_current_dir()` in `src/nvim/window_shim.c`.
//!
//! The function adjusts the current working directory when switching windows,
//! respecting window-local (`:lcd`), tab-local (`:tcd`), and global dirs.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::ffi::{c_char, c_int};

// MAXPATHL on Linux is 4096
const MAXPATHL: usize = 4096;

// os_dirname return codes
const OK: c_int = 1;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // curwin->w_localdir (or NULL)
    fn nvim_curwin_get_localdir() -> *const c_char;
    // curtab->tp_localdir (or NULL)
    fn nvim_curtab_get_localdir() -> *const c_char;
    // globaldir global
    fn nvim_get_globaldir() -> *const c_char;
    // set globaldir = xstrdup(s)
    fn nvim_set_globaldir_from_str(s: *const c_char);
    // XFREE_CLEAR(globaldir)
    fn nvim_clear_globaldir();
    // os_dirname(buf, MAXPATHL) -> OK or FAIL
    fn nvim_os_dirname_maxpathl(buf: *mut c_char) -> c_int;
    // os_chdir(dir) -> 0 on success
    fn nvim_os_chdir(dir: *const c_char) -> c_int;
    // pathcmp(a, b, -1) -> 0 if equal
    fn nvim_pathcmp_unlen(a: *const c_char, b: *const c_char) -> c_int;
    // p_acd option
    fn nvim_get_p_acd() -> c_int;
    // last_chdir_reason = NULL
    fn nvim_set_last_chdir_reason_null();
    // shorten_fnames(true)
    fn nvim_shorten_fnames_force();
    // do_autocmd_dirchanged for window/tabpage scope
    fn nvim_do_autocmd_dirchanged_win(new_dir: *const c_char, localdir: c_int, pre: c_int);
    // do_autocmd_dirchanged for global scope
    fn nvim_do_autocmd_dirchanged_global(new_dir: *const c_char, pre: c_int);
}

// =============================================================================
// Implementation
// =============================================================================

/// Used after making another window the current one: change directory if needed.
///
/// Rust port of C `win_fix_current_dir()`.
///
/// # Safety
///
/// Accesses global Neovim state through C accessor functions.
fn win_fix_current_dir_impl() {
    unsafe {
        // New directory is either the local directory of the window, tab or NULL.
        let win_localdir = nvim_curwin_get_localdir();
        let new_dir = if !win_localdir.is_null() {
            win_localdir
        } else {
            nvim_curtab_get_localdir()
        };

        let mut cwd = [0u8; MAXPATHL];
        let cwd_ptr = cwd.as_mut_ptr().cast::<c_char>();
        if nvim_os_dirname_maxpathl(cwd_ptr) != OK {
            cwd[0] = 0;
        }

        if !new_dir.is_null() {
            // Window/tab has a local directory: Save current directory as global
            // (unless that was done already) and change to the local directory.
            let globaldir = nvim_get_globaldir();
            if globaldir.is_null() && cwd[0] != 0 {
                nvim_set_globaldir_from_str(cwd_ptr);
            }

            let dir_differs = nvim_pathcmp_unlen(new_dir, cwd_ptr) != 0;
            let p_acd = nvim_get_p_acd() != 0;

            // localdir=1 means window scope, localdir=0 means tabpage scope.
            let is_window_local = if win_localdir.is_null() { 0 } else { 1 };

            if !p_acd && dir_differs {
                nvim_do_autocmd_dirchanged_win(new_dir, is_window_local, 1);
            }
            if nvim_os_chdir(new_dir) == 0 {
                if !p_acd && dir_differs {
                    nvim_do_autocmd_dirchanged_win(new_dir, is_window_local, 0);
                }
            }
            nvim_set_last_chdir_reason_null();
            nvim_shorten_fnames_force();
        } else {
            let globaldir = nvim_get_globaldir();
            if !globaldir.is_null() {
                // Window doesn't have a local directory and we are not in the global
                // directory: Change to the global directory.
                let dir_differs = nvim_pathcmp_unlen(globaldir, cwd_ptr) != 0;
                let p_acd = nvim_get_p_acd() != 0;
                if !p_acd && dir_differs {
                    nvim_do_autocmd_dirchanged_global(globaldir, 1);
                }
                if nvim_os_chdir(globaldir) == 0 {
                    if !p_acd && dir_differs {
                        nvim_do_autocmd_dirchanged_global(globaldir, 0);
                    }
                }
                nvim_clear_globaldir();
                nvim_set_last_chdir_reason_null();
                nvim_shorten_fnames_force();
            }
        }
    }
}

// =============================================================================
// FFI Export
// =============================================================================

/// FFI wrapper for `win_fix_current_dir`.
///
/// Called from C thin wrapper `void win_fix_current_dir()` and from
/// `rs_win_enter_ext` (Phase 1).
///
/// # Safety
///
/// Called from C via FFI.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_fix_current_dir() {
    win_fix_current_dir_impl();
}
