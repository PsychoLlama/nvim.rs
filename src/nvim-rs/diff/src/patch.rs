//! `:diffpatch` command implementation
//!
//! This module provides the Rust implementation of the `:diffpatch` ex command,
//! which creates a new version of a file from the current buffer and a diff file.

use std::ffi::c_char;
use std::os::raw::c_int;

use crate::buffer::WinHandle;

/// Result constants.
const OK: c_int = 1;
const FAIL: c_int = 0;

/// DIFF_VERTICAL flag value (must match C).
const DIFF_VERTICAL: c_int = 0x080;
/// WSP_VERT window-split flag (must match C window.h).
const WSP_VERT: c_int = 0x02;

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    // Temp file management
    fn nvim_diff_vim_tempname() -> *mut c_char;

    // Buffer write to file
    fn nvim_diff_buf_write_curbuf(fname: *const c_char) -> c_int;

    // Path utilities
    fn nvim_diff_FullName_save(fname: *const c_char) -> *mut c_char;
    fn nvim_diff_vim_strsave_shellescape(s: *const c_char) -> *mut c_char;

    // Directory management (UNIX)
    fn nvim_diff_os_dirname(buf: *mut c_char, size: c_int) -> c_int;
    fn nvim_diff_os_chdir(dir: *const c_char) -> c_int;
    fn nvim_diff_vim_gettempdir() -> *const c_char;
    fn nvim_diff_shorten_fnames();

    // Patch application
    fn nvim_diff_is_patchexpr_set() -> bool;
    fn nvim_diff_eval_patch(orig: *const c_char, diff: *const c_char, out: *const c_char);
    fn nvim_diff_call_shell_filter(cmd: *const c_char);

    // File operations
    fn nvim_diff_os_fileinfo_size(fname: *const c_char, size_out: *mut u64) -> bool;
    fn nvim_diff_os_remove(fname: *const c_char);

    // Memory
    fn nvim_diff_xfree(p: *mut std::ffi::c_void);
    fn nvim_diff_xstrnsave(s: *const c_char, len: usize) -> *mut c_char;
    fn nvim_diff_get_MAXPATHL() -> c_int;
    fn nvim_diff_xmalloc(size: usize) -> *mut c_char;
    fn nvim_diff_vim_snprintf_patch(
        buf: *mut c_char,
        buflen: usize,
        tmp_new: *const c_char,
        tmp_orig: *const c_char,
        esc_name: *const c_char,
    );
    fn nvim_diff_strlen(s: *const c_char) -> usize;

    // Curbuf info
    fn nvim_diff_get_curbuf_fname() -> *const c_char;

    // Error messages
    fn nvim_diff_emsg_e816();
    fn nvim_diff_emsg_prev_dir();

    // Window / ex command
    fn nvim_diff_get_diff_flags() -> c_int;
    fn rs_win_split(size: c_int, flags: c_int) -> c_int;
    fn nvim_diff_get_curwin() -> WinHandle;
    fn nvim_diff_set_cmdmod_tab_zero();
    fn nvim_eap_set_cmdidx(eap: *mut std::ffi::c_void, idx: c_int);
    fn nvim_eap_set_arg(eap: *mut std::ffi::c_void, arg: *mut c_char);
    fn nvim_eap_get_arg(eap: *const std::ffi::c_void) -> *mut c_char;
    fn nvim_diff_do_exedit_with_old_curwin(eap: *mut std::ffi::c_void, old_curwin: WinHandle);
    fn nvim_diff_get_CMD_split() -> c_int;
    fn rs_win_valid(wp: WinHandle) -> c_int;
    fn rs_diff_win_options(wp: WinHandle, addbuf: bool);
    fn nvim_diff_ex_file(eap: *mut std::ffi::c_void);
    fn nvim_diff_augroup_exists_filetypedetect() -> bool;
    fn nvim_diff_do_cmdline_cmd(cmd: *const c_char);
}

// =============================================================================
// Internal implementation
// =============================================================================

/// Inner body of `:diffpatch`.  Returns true on success, false on early error.
/// Cleanup of `tmp_orig`/`tmp_new` is handled by the caller (`ex_diffpatch_impl`).
#[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
unsafe fn ex_diffpatch_body(
    eap: *mut std::ffi::c_void,
    old_curwin: WinHandle,
    tmp_orig: *mut c_char,
    tmp_new: *mut c_char,
) -> bool {
    if tmp_orig.is_null() || tmp_new.is_null() {
        return false;
    }

    // Write the current buffer to tmp_orig.
    if nvim_diff_buf_write_curbuf(tmp_orig.cast_const()) == FAIL {
        return false;
    }

    // Get the absolute path of the patchfile.
    let eap_arg = nvim_eap_get_arg(eap);
    let fullname = nvim_diff_FullName_save(eap_arg);
    let diff_path: *const c_char = if fullname.is_null() {
        eap_arg
    } else {
        fullname
    };

    // Shell-escape the patchfile name.
    let esc_name = nvim_diff_vim_strsave_shellescape(diff_path);
    if esc_name.is_null() {
        nvim_diff_xfree(fullname.cast());
        return false;
    }

    // Allocate command buffer.
    let buflen = nvim_diff_strlen(tmp_orig.cast_const())
        + nvim_diff_strlen(esc_name.cast_const())
        + nvim_diff_strlen(tmp_new.cast_const())
        + 16;
    let cmdbuf = nvim_diff_xmalloc(buflen);

    // Save current directory and chdir to temp dir.
    let maxpathl = nvim_diff_get_MAXPATHL();
    // maxpathl is MAXPATHL (typically 4096 on Linux), always non-negative.
    let dirbuf_size = (maxpathl + 1).unsigned_abs() as usize;
    let dirbuf = nvim_diff_xmalloc(dirbuf_size);
    let dir_changed = if !dirbuf.is_null()
        && nvim_diff_os_dirname(dirbuf, maxpathl + 1) == OK
        && nvim_diff_os_chdir(dirbuf.cast_const()) == 0
    {
        let tempdir = nvim_diff_vim_gettempdir();
        let tempdir_ptr: *const c_char = if tempdir.is_null() || *tempdir == 0 {
            c"/tmp".as_ptr()
        } else {
            tempdir
        };
        nvim_diff_os_chdir(tempdir_ptr);
        nvim_diff_shorten_fnames();
        true
    } else {
        false
    };

    // Apply the patch.
    if nvim_diff_is_patchexpr_set() {
        nvim_diff_eval_patch(tmp_orig.cast_const(), diff_path, tmp_new.cast_const());
    } else if !cmdbuf.is_null() {
        nvim_diff_vim_snprintf_patch(
            cmdbuf,
            buflen,
            tmp_new.cast_const(),
            tmp_orig.cast_const(),
            esc_name.cast_const(),
        );
        nvim_diff_call_shell_filter(cmdbuf.cast_const());
    }

    // Restore directory.
    if dir_changed && !dirbuf.is_null() {
        if nvim_diff_os_chdir(dirbuf.cast_const()) != 0 {
            nvim_diff_emsg_prev_dir();
        }
        nvim_diff_shorten_fnames();
    }
    nvim_diff_xfree(dirbuf.cast());
    nvim_diff_xfree(cmdbuf.cast());
    nvim_diff_xfree(esc_name.cast());
    nvim_diff_xfree(fullname.cast());

    // Delete .orig and .rej files created by patch.
    let tmp_new_len = nvim_diff_strlen(tmp_new.cast_const());
    let suffixed_len = tmp_new_len + 8;
    let suffixed = nvim_diff_xmalloc(suffixed_len);
    if !suffixed.is_null() {
        // Build "tmp_new.orig" and remove it.
        std::ptr::copy_nonoverlapping(tmp_new.cast_const(), suffixed, tmp_new_len);
        let orig_suffix: &[u8] = b".orig\0";
        std::ptr::copy_nonoverlapping(
            orig_suffix.as_ptr().cast::<c_char>(),
            suffixed.add(tmp_new_len),
            orig_suffix.len(),
        );
        nvim_diff_os_remove(suffixed.cast_const());

        // Build "tmp_new.rej" and remove it.
        let rej_suffix: &[u8] = b".rej\0";
        std::ptr::copy_nonoverlapping(
            rej_suffix.as_ptr().cast::<c_char>(),
            suffixed.add(tmp_new_len),
            rej_suffix.len(),
        );
        nvim_diff_os_remove(suffixed.cast_const());
        nvim_diff_xfree(suffixed.cast());
    }

    // Only continue if the output file was created and is non-empty.
    let mut filesize: u64 = 0;
    let info_ok =
        nvim_diff_os_fileinfo_size(tmp_new.cast_const(), std::ptr::addr_of_mut!(filesize));
    if !info_ok || filesize == 0 {
        nvim_diff_emsg_e816();
        return false;
    }

    // Build the new buffer name (curbuf->b_fname + ".new").
    let curbuf_fname = nvim_diff_get_curbuf_fname();
    let newname: *mut c_char = if !curbuf_fname.is_null() && *curbuf_fname != 0 {
        let fname_len = nvim_diff_strlen(curbuf_fname);
        let p = nvim_diff_xstrnsave(curbuf_fname, fname_len + 4);
        if !p.is_null() {
            let new_suffix: &[u8] = b".new\0";
            std::ptr::copy_nonoverlapping(
                new_suffix.as_ptr().cast::<c_char>(),
                p.add(fname_len),
                new_suffix.len(),
            );
        }
        p
    } else {
        std::ptr::null_mut()
    };

    // Don't use a new tab page; each tab page has its own diffs.
    nvim_diff_set_cmdmod_tab_zero();

    let split_flags = if nvim_diff_get_diff_flags() & DIFF_VERTICAL != 0 {
        WSP_VERT
    } else {
        0
    };

    if rs_win_split(0, split_flags) != FAIL {
        // Pretend it was a ":split fname" command.
        let cmd_split = nvim_diff_get_CMD_split();
        nvim_eap_set_cmdidx(eap, cmd_split);
        nvim_eap_set_arg(eap, tmp_new);
        nvim_diff_do_exedit_with_old_curwin(eap, old_curwin);

        // Check that split worked.
        let curwin = nvim_diff_get_curwin();
        if curwin != old_curwin && rs_win_valid(old_curwin) != 0 {
            // Set 'diff', 'scrollbind' on and 'wrap' off for both windows.
            rs_diff_win_options(curwin, true);
            rs_diff_win_options(old_curwin, true);

            if !newname.is_null() {
                // Do ":file filename.new" on the patched buffer.
                nvim_eap_set_arg(eap, newname);
                nvim_diff_ex_file(eap);

                // Do filetype detection with the new name.
                if nvim_diff_augroup_exists_filetypedetect() {
                    nvim_diff_do_cmdline_cmd(c":doau filetypedetect BufRead".as_ptr());
                }
            }
        }
    }

    nvim_diff_xfree(newname.cast());
    true
}

/// Core dispatcher for `:diffpatch`.
///
/// Allocates temp files, delegates to `ex_diffpatch_body`, then always removes
/// the temp files (matching the C `goto theend` cleanup pattern).
///
/// # Safety
/// `eap` must be a valid pointer to an `exarg_T`.
unsafe fn ex_diffpatch_impl(eap: *mut std::ffi::c_void, old_curwin: WinHandle) {
    let tmp_orig = nvim_diff_vim_tempname();
    let tmp_new = nvim_diff_vim_tempname();

    ex_diffpatch_body(eap, old_curwin, tmp_orig, tmp_new);

    // Always clean up temp files (matches C `goto theend` behavior).
    if !tmp_orig.is_null() {
        nvim_diff_os_remove(tmp_orig.cast_const());
    }
    nvim_diff_xfree(tmp_orig.cast());

    if !tmp_new.is_null() {
        nvim_diff_os_remove(tmp_new.cast_const());
    }
    nvim_diff_xfree(tmp_new.cast());
}

// =============================================================================
// Public FFI entry point
// =============================================================================

/// `:diffpatch` command FFI entry point.
///
/// # Safety
/// `eap` must be a valid pointer to an `exarg_T`.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_diffpatch(eap: *mut std::ffi::c_void) {
    let old_curwin = nvim_diff_get_curwin();
    ex_diffpatch_impl(eap, old_curwin);
}
