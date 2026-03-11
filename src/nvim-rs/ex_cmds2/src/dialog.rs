//! Dialog functions for buffer change prompts
//!
//! Ports of `dialog_changed` and `dialog_close_terminal`.

use std::ffi::{c_char, c_int};

use crate::autowrite_impl::BufHandle;
use crate::DIALOG_MSG_SIZE;

const OK: c_int = 1;
const VIM_YES: c_int = 2;
const VIM_NO: c_int = 3;
const VIM_ALL: c_int = 5;
const VIM_DISCARDALL: c_int = 6;
const VIM_QUESTION: c_int = 4;

extern "C" {
    // --- already declared in autowrite_impl ---
    fn nvim_ex2_buf_get_fname(buf: *mut BufHandle) -> *const c_char;
    fn nvim_ex2_buf_get_ffname(buf: *mut BufHandle) -> *const c_char;
    fn nvim_ex2_buf_get_b_p_ro(buf: *mut BufHandle) -> c_int;
    fn nvim_ex2_bufIsChanged(buf: *mut BufHandle) -> bool;
    fn nvim_ex2_get_firstbuf() -> *mut BufHandle;
    fn nvim_ex2_buf_next(buf: *mut BufHandle) -> *mut BufHandle;
    fn nvim_ex2_bufref_create(buf: *mut BufHandle) -> *mut crate::autowrite_impl::BufrefHandle;
    fn nvim_ex2_bufref_valid(br: *mut crate::autowrite_impl::BufrefHandle) -> bool;
    fn nvim_ex2_bufref_free(br: *mut crate::autowrite_impl::BufrefHandle);
    fn nvim_ex2_gettext(s: *const c_char) -> *const c_char;

    // --- dialog-specific accessors ---
    fn nvim_ex2_buf_get_fnum(buf: *mut BufHandle) -> c_int;
    #[link_name = "dialog_msg"]
    fn nvim_ex2_dialog_msg(buff: *mut c_char, format: *const c_char, fname: *const c_char);
    #[link_name = "vim_dialog_yesnocancel"]
    fn nvim_ex2_vim_dialog_yesnocancel(
        typ: c_int,
        title: *const c_char,
        message: *const c_char,
        dflt: c_int,
    ) -> c_int;
    #[link_name = "vim_dialog_yesnoallcancel"]
    fn nvim_ex2_vim_dialog_yesnoallcancel(
        typ: c_int,
        title: *const c_char,
        message: *const c_char,
        dflt: c_int,
    ) -> c_int;
    // nvim_ex2_check_overwrite: special wrapper (stack-local exarg_T), kept
    fn nvim_ex2_check_overwrite(
        buf: *mut BufHandle,
        fname: *const c_char,
        ffname: *const c_char,
    ) -> c_int;
    #[link_name = "unchanged"]
    fn nvim_ex2_unchanged(buf: *mut BufHandle, ff: bool, always_inc_changedtick: bool);
    #[link_name = "buf_set_name"]
    fn nvim_ex2_buf_set_name(fnum: c_int, name: *mut c_char);
    fn nvim_ex2_buf_clear_names(buf: *mut BufHandle);
    fn nvim_ex2_buf_set_fname_null(buf: *mut BufHandle);
}

// Rust-internal: write all lines in a buffer
unsafe fn buf_write_all(buf: *mut BufHandle, forceit: bool) -> c_int {
    // Call the Rust implementation exported as "buf_write_all"
    extern "C" {
        #[link_name = "buf_write_all"]
        fn buf_write_all_fn(buf: *mut BufHandle, forceit: bool) -> c_int;
    }
    unsafe { buf_write_all_fn(buf, forceit) }
}

/// Port of `dialog_close_terminal`
#[export_name = "dialog_close_terminal"]
pub unsafe extern "C" fn rs_dialog_close_terminal(buf: *mut BufHandle) -> bool {
    let mut buff = [0u8; DIALOG_MSG_SIZE];

    let fname = unsafe { nvim_ex2_buf_get_fname(buf) };
    let display_name = if fname.is_null() {
        b"?\0".as_ptr().cast()
    } else {
        fname
    };

    unsafe {
        nvim_ex2_dialog_msg(
            buff.as_mut_ptr().cast(),
            nvim_ex2_gettext(b"Close \"%s\"?\0".as_ptr().cast()),
            display_name,
        );
    }

    let ret = unsafe {
        nvim_ex2_vim_dialog_yesnocancel(VIM_QUESTION, std::ptr::null(), buff.as_ptr().cast(), 1)
    };

    ret == VIM_YES
}

/// Port of `dialog_changed`
#[export_name = "dialog_changed"]
pub unsafe extern "C" fn rs_dialog_changed(buf: *mut BufHandle, checkall: bool) {
    let mut buff = [0u8; DIALOG_MSG_SIZE];

    unsafe {
        nvim_ex2_dialog_msg(
            buff.as_mut_ptr().cast(),
            nvim_ex2_gettext(b"Save changes to \"%s\"?\0".as_ptr().cast()),
            nvim_ex2_buf_get_fname(buf),
        );
    }

    let ret = if checkall {
        unsafe {
            nvim_ex2_vim_dialog_yesnoallcancel(
                VIM_QUESTION,
                std::ptr::null(),
                buff.as_ptr().cast(),
                1,
            )
        }
    } else {
        unsafe {
            nvim_ex2_vim_dialog_yesnocancel(VIM_QUESTION, std::ptr::null(), buff.as_ptr().cast(), 1)
        }
    };

    if ret == VIM_YES {
        let empty_bufname = unsafe { nvim_ex2_buf_get_fname(buf) }.is_null();
        if empty_bufname {
            let fnum = unsafe { nvim_ex2_buf_get_fnum(buf) };
            unsafe { nvim_ex2_buf_set_name(fnum, b"Untitled\0".as_ptr().cast_mut().cast()) };
        }

        let fname = unsafe { nvim_ex2_buf_get_fname(buf) };
        let ffname = unsafe { nvim_ex2_buf_get_ffname(buf) };

        if unsafe { nvim_ex2_check_overwrite(buf, fname, ffname) } == OK
            && unsafe { buf_write_all(buf, false) } == OK
        {
            return;
        }

        // restore to empty when write failed
        if empty_bufname {
            unsafe { nvim_ex2_buf_set_fname_null(buf) };
            unsafe { nvim_ex2_buf_clear_names(buf) };
            unsafe { nvim_ex2_unchanged(buf, true, false) };
        }
    } else if ret == VIM_NO {
        unsafe { nvim_ex2_unchanged(buf, true, false) };
    } else if ret == VIM_ALL {
        // Write all modified files that can be written.
        // Skip readonly buffers, these need to be confirmed individually.
        let mut buf2 = unsafe { nvim_ex2_get_firstbuf() };
        while !buf2.is_null() {
            if unsafe { nvim_ex2_bufIsChanged(buf2) }
                && !unsafe { nvim_ex2_buf_get_ffname(buf2) }.is_null()
                && unsafe { nvim_ex2_buf_get_b_p_ro(buf2) } == 0
            {
                let bufref = unsafe { nvim_ex2_bufref_create(buf2) };

                let fname2 = unsafe { nvim_ex2_buf_get_fname(buf2) };
                let ffname2 = unsafe { nvim_ex2_buf_get_ffname(buf2) };

                if !fname2.is_null()
                    && unsafe { nvim_ex2_check_overwrite(buf2, fname2, ffname2) } == OK
                {
                    unsafe { buf_write_all(buf2, false) };
                }
                // an autocommand may have deleted the buffer
                if !unsafe { nvim_ex2_bufref_valid(bufref) } {
                    buf2 = unsafe { nvim_ex2_get_firstbuf() };
                }
                unsafe { nvim_ex2_bufref_free(bufref) };
            }
            buf2 = unsafe { nvim_ex2_buf_next(buf2) };
        }
    } else if ret == VIM_DISCARDALL {
        // mark all buffers as unchanged
        let mut buf2 = unsafe { nvim_ex2_get_firstbuf() };
        while !buf2.is_null() {
            unsafe { nvim_ex2_unchanged(buf2, true, false) };
            buf2 = unsafe { nvim_ex2_buf_next(buf2) };
        }
    }
}
