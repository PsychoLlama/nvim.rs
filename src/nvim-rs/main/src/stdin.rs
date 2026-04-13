//! Stdin reading
//!
//! Implements `rs_read_stdin` replacing the static C function in main.c.

use nvim_buffer::buf_struct::BufStruct;
use std::ffi::{c_char, c_int};

// Buffer list flags
const BLN_LISTED: c_int = 2;

// Readfile flags
const READ_NEW: c_int = 0x01;
const READ_STDIN: c_int = 0x04;

// SEA_DIALOG = 1 (from globals.h)
const SEA_DIALOG: c_int = 1;

// MAXLNUM = 0x7fffffff (from pos_defs.h)
const MAXLNUM: i32 = 0x7fff_ffff;

// IOSIZE = 1025 (from globals.h)
const IOSIZE: usize = 1025;

unsafe extern "C" {
    fn buflist_new(
        ffname: *mut c_char,
        sfname: *mut c_char,
        lnum: i32,
        flags: c_int,
    ) -> *mut std::ffi::c_void; // buf_T*

    fn set_curbuf(buf: *mut std::ffi::c_void, action: c_int, update_jumplist: bool);
    fn readfile(
        fname: *mut c_char,
        sfname: *mut c_char,
        from: i32,
        lines_to_skip: i32,
        lines_to_read: i32,
        eap: *mut std::ffi::c_void, // exarg_T*
        flags: c_int,
        silent: bool,
    ) -> c_int;
    fn open_buffer(read_stdin: bool, eap: *mut std::ffi::c_void, flags_arg: c_int) -> c_int;
    fn rs_set_buflisted(on: c_int);
    fn do_cmdline_cmd(cmd: *const c_char) -> c_int;
    fn vim_snprintf(buf: *mut c_char, len: usize, fmt: *const c_char, ...) -> c_int;
    fn semsg(fmt: *const c_char, ...);
    fn rs_check_swap_exists_action();

    fn nvim_curbuf_get_handle() -> c_int;
    fn nvim_curbuf_has_ffname() -> c_int;
    fn nvim_buf_is_empty(buf: *mut std::ffi::c_void) -> c_int;
    fn nvim_curbuf_b_next_null() -> c_int;
    // Returns current curbuf pointer (opaque)
    fn nvim_get_curbuf() -> *mut std::ffi::c_void;

    static mut swap_exists_action: c_int;
    static mut no_wait_return: c_int;
    static mut msg_didany: bool;
}

/// Read text from stdin into the current buffer.
///
/// # Safety
/// Must be called from the main thread during startup, after curbuf is set.
#[no_mangle]
pub unsafe extern "C" fn rs_read_stdin() {
    swap_exists_action = SEA_DIALOG;
    no_wait_return = true as c_int;
    let save_msg_didany = msg_didany;

    if nvim_curbuf_has_ffname() != 0 {
        // curbuf is already opened for a file, create a new buffer for stdin. #35269
        let stdin_buf = buflist_new(std::ptr::null_mut(), std::ptr::null_mut(), 0, BLN_LISTED);
        if stdin_buf.is_null() {
            semsg(c"Failed to create buffer for stdin".as_ptr());
            return;
        }

        // remember the current buffer number so we can go back to it
        let initial_buf_handle = nvim_curbuf_get_handle();

        // set the buffer we just created as curbuf so we can read stdin into it
        set_curbuf(stdin_buf, 0, false);
        readfile(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
            0,
            MAXLNUM,
            std::ptr::null_mut(),
            READ_NEW + READ_STDIN,
            true,
        );

        // remember stdin_buf_handle so we can close it if stdin_buf ends up empty
        let stdin_buf_handle = (*stdin_buf.cast::<BufStruct>()).handle;
        let curbuf = nvim_get_curbuf();
        let stdin_buf_empty = nvim_buf_is_empty(curbuf) != 0;

        // switch back to the original starting buffer
        let mut buf = [0i8; IOSIZE];
        vim_snprintf(
            buf.as_mut_ptr(),
            IOSIZE,
            c"silent! buffer %d".as_ptr(),
            initial_buf_handle,
        );
        do_cmdline_cmd(buf.as_ptr());

        if stdin_buf_empty {
            // stdin buffer may be first or last ("echo foo | nvim file1 -"). #35269
            // only wipe buffer after having switched to original starting buffer. #35681
            vim_snprintf(
                buf.as_mut_ptr(),
                IOSIZE,
                c"silent! bwipeout! %d".as_ptr(),
                stdin_buf_handle,
            );
            do_cmdline_cmd(buf.as_ptr());
        }
    } else {
        // stdin buffer is first so we can just use curbuf
        rs_set_buflisted(1);
        // Create memfile and read from stdin.
        open_buffer(true, std::ptr::null_mut(), 0);
        // stdin was empty so we should wipe it (e.g. "echo file1 | xargs nvim"). #8561
        let curbuf = nvim_get_curbuf();
        if nvim_buf_is_empty(curbuf) != 0 && nvim_curbuf_b_next_null() == 0 {
            do_cmdline_cmd(c"silent! bnext".as_ptr());
            do_cmdline_cmd(c"silent! bwipeout 1".as_ptr());
        }
    }

    no_wait_return = false as c_int;
    msg_didany = save_msg_didany;
    // TIME_MSG("reading stdin") is a no-op in Rust
    rs_check_swap_exists_action();
}
