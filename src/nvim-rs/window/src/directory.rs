//! Window directory management (`win_fix_current_dir`).
//!
//! This module provides `rs_win_fix_current_dir`, the Rust replacement for
//! `win_fix_current_dir()` in `src/nvim/window_shim.c`.
//!
//! The function adjusts the current working directory when switching windows,
//! respecting window-local (`:lcd`), tab-local (`:tcd`), and global dirs.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use crate::win_struct::win_ref;
use crate::TabpageHandle;
use crate::WinHandle;
use std::ffi::{c_char, c_int};

// MAXPATHL on Linux is 4096
const MAXPATHL: usize = 4096;

// os_dirname return codes
const OK: c_int = 1;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    fn nvim_get_curwin() -> WinHandle;
    fn nvim_get_curtab() -> TabpageHandle;
    // globaldir global
    fn nvim_get_globaldir() -> *const c_char;
    // set globaldir = xstrdup(s)
    fn nvim_set_globaldir_from_str(s: *const c_char);
    // XFREE_CLEAR(globaldir)
    fn nvim_clear_globaldir();
    // os_dirname(buf, MAXPATHL) -> OK or FAIL
    fn nvim_os_dirname_maxpathl(buf: *mut c_char) -> c_int;
    // os_chdir(dir) -> 0 on success
    #[link_name = "os_chdir"]
    fn nvim_os_chdir(dir: *const c_char) -> c_int;
    // pathcmp(a, b, -1) -> 0 if equal
    #[link_name = "pathcmp"]
    fn nvim_pathcmp_impl(a: *const c_char, b: *const c_char, maxlen: c_int) -> c_int;
    // p_acd option
    fn nvim_get_p_acd() -> c_int;
    // last_chdir_reason = NULL
    fn nvim_set_last_chdir_reason_null();
    // shorten_fnames(true)
    #[link_name = "shorten_fnames"]
    fn nvim_shorten_fnames_force(force: c_int);
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
        let win_localdir = win_ref(nvim_get_curwin()).w_localdir.cast_const();
        let new_dir = if win_localdir.is_null() {
            nvim_get_curtab().as_tabpage_ref().tp_localdir.cast_const()
        } else {
            win_localdir
        };

        let mut cwd = [0u8; MAXPATHL];
        let cwd_ptr = cwd.as_mut_ptr().cast::<c_char>();
        if nvim_os_dirname_maxpathl(cwd_ptr) != OK {
            cwd[0] = 0;
        }

        let globaldir = nvim_get_globaldir();

        if new_dir.is_null() {
            if !globaldir.is_null() {
                // Window doesn't have a local directory and we are not in the global
                // directory: Change to the global directory.
                let dir_differs = nvim_pathcmp_impl(globaldir, cwd_ptr, -1) != 0;
                let p_acd = nvim_get_p_acd() != 0;
                if !p_acd && dir_differs {
                    nvim_do_autocmd_dirchanged_global(globaldir, 1);
                }
                if nvim_os_chdir(globaldir) == 0 && !p_acd && dir_differs {
                    nvim_do_autocmd_dirchanged_global(globaldir, 0);
                }
                nvim_clear_globaldir();
                nvim_set_last_chdir_reason_null();
                nvim_shorten_fnames_force(1);
            }
        } else {
            // Window/tab has a local directory: Save current directory as global
            // (unless that was done already) and change to the local directory.
            if globaldir.is_null() && cwd[0] != 0 {
                nvim_set_globaldir_from_str(cwd_ptr);
            }

            let dir_differs = nvim_pathcmp_impl(new_dir, cwd_ptr, -1) != 0;
            let p_acd = nvim_get_p_acd() != 0;

            // localdir=1 means window scope, localdir=0 means tabpage scope.
            let is_window_local = c_int::from(!win_localdir.is_null());

            if !p_acd && dir_differs {
                nvim_do_autocmd_dirchanged_win(new_dir, is_window_local, 1);
            }
            if nvim_os_chdir(new_dir) == 0 && !p_acd && dir_differs {
                nvim_do_autocmd_dirchanged_win(new_dir, is_window_local, 0);
            }
            nvim_set_last_chdir_reason_null();
            nvim_shorten_fnames_force(1);
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

/// C export: `win_fix_current_dir` — eliminates the C thin wrapper.
///
/// # Safety
/// Called from C via FFI.
#[unsafe(export_name = "win_fix_current_dir")]
pub unsafe extern "C" fn win_fix_current_dir() {
    win_fix_current_dir_impl();
}
