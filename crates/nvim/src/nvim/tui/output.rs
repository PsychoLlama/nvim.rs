//! The TUI's write path.
//!
//! Everything the TUI emits is staged in one buffer and leaves in a single
//! `uv_write` per flush, wrapped in whatever this terminal needs around a
//! screen update: synchronised-output brackets when it has them, and a
//! cursor hide/show pair when it does not, so a repaint is not watched
//! happening a cell at a time.
//!
//! The wrappers are why a flush writes three buffers rather than one -- a
//! prologue, the staged bytes, an epilogue -- and why [`flush_buf`] is the
//! only function here that talks to libuv.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::event::libuv::{uv_run, uv_strerror, uv_write};
use crate::src::nvim::log::logmsg;
use crate::src::nvim::os::libc::fwrite;
use crate::src::nvim::tui::terminfo::caps::{
    TerminfoDef, kTerm_cursor_invisible, kTerm_cursor_normal, kTermCount,
};
use crate::src::nvim::tui::terminfo::terminfo_fmt;
use crate::src::nvim::tui::tui::TUIData;
use crate::src::nvim::types::{TPVAR, size_t, uv_buf_t, uv_stream_t, uv_write_t};
use core::ffi::{CStr, c_char, c_int};

/// The staging buffer's size. A flush is one `uv_write`, so this also caps
/// how much can be written without a syscall.
pub const BUF_SIZE: usize = 65535;

/// How much room a parameterised capability is assumed to need. If less than
/// this is left in the buffer, flush first rather than risk a short write.
pub const TERMINFO_SEQ_LIMIT: usize = 128;

/// The prologue/epilogue scratch buffers. Large enough for a
/// synchronised-output bracket plus a cursor visibility change.
const WRAP_BUF: usize = 32;

/// DECSET/DECRST 2026: begin and end a synchronised update, so the terminal
/// paints the whole frame at once instead of as it arrives.
const SYNC_START: &CStr = c"\x1b[?2026h";
const SYNC_END: &CStr = c"\x1b[?2026l";

/// terminfo's parameter stack, all nine slots empty. Capabilities that take
/// no parameters still have to be handed one of these.
const NO_PARAMS: [TPVAR; 9] = [TPVAR {
    num: 0,
    string: core::ptr::null_mut(),
}; 9];

/// Should the cursor be hidden right now? It is, while the editor is busy
/// and whenever the editor has asked for it.
fn should_invisible(tui: &TUIData) -> bool {
    tui.busy || tui.want_invisible
}

// ------------------------------------------------------------------ staging

/// Stage `bytes` for the next flush.
///
/// A write that does not fit flushes first; one larger than the whole buffer
/// is handed to the flush directly rather than copied, which is what
/// `buf_to_flush` exists for.
///
/// # Safety
/// `tui` must point to a live `TUIData`.
pub unsafe fn out(tui: *mut TUIData, bytes: &[u8]) {
    unsafe {
        let len = bytes.len();
        if len > BUF_SIZE - (*tui).bufpos {
            flush_buf(tui);
            if len > BUF_SIZE {
                (*tui).buf_to_flush = bytes.as_ptr().cast_mut().cast();
                (*tui).bufpos = len;
                flush_buf(tui);
                return;
            }
        }
        let dst = (&raw mut (*tui).buf).cast::<u8>().add((*tui).bufpos);
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), dst, len);
        (*tui).bufpos += len;
    }
}

/// Stage a NUL-terminated string, doing nothing when there is none.
///
/// # Safety
/// `tui` must point to a live `TUIData`.
pub unsafe fn out_cstr(tui: *mut TUIData, s: Option<&CStr>) {
    if let Some(s) = s {
        unsafe { out(tui, s.to_bytes()) };
    }
}

/// Stage `len` bytes starting at `ptr`.
///
/// For the callers that hold a pointer and a length rather than a slice:
/// an API `String_0`, the URL builder's buffer, a `strlen`-measured cell.
///
/// # Safety
/// `ptr` must be valid for reads of `len` bytes.
pub unsafe fn out_raw(tui: *mut TUIData, ptr: *const c_char, len: usize) {
    unsafe { out(tui, core::slice::from_raw_parts(ptr.cast::<u8>(), len)) };
}

/// Stage a raw capability pointer, doing nothing when it is null.
///
/// # Safety
/// `tui` must be live, and `str` either null or NUL-terminated.
pub unsafe fn out_ptr(tui: *mut TUIData, str: *const c_char) {
    if !str.is_null() {
        unsafe { out(tui, CStr::from_ptr(str).to_bytes()) };
    }
}

/// Format up to three integers into `fmt` and stage the result.
///
/// This replaces a `printf` variadic whose only two callers pass small
/// integers; `fmt` is applied by [`core::fmt`], so the format string is
/// checked at compile time rather than trusted at runtime.
///
/// # Safety
/// `tui` must point to a live `TUIData`.
pub unsafe fn out_fmt(tui: *mut TUIData, args: core::fmt::Arguments<'_>) {
    use core::fmt::Write;

    /// Enough for any escape sequence these callers build: the longest is a
    /// 24-bit underline colour, at 22 bytes.
    struct Scratch {
        buf: [u8; 64],
        len: usize,
    }
    impl Write for Scratch {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let end = self.len + s.len();
            if end > self.buf.len() {
                return Err(core::fmt::Error);
            }
            self.buf[self.len..end].copy_from_slice(s.as_bytes());
            self.len = end;
            Ok(())
        }
    }

    let mut scratch = Scratch {
        buf: [0; 64],
        len: 0,
    };
    // A sequence that does not fit is dropped rather than truncated: half an
    // escape sequence is worse on the wire than none.
    if scratch.write_fmt(args).is_ok() {
        unsafe { out(tui, &scratch.buf[..scratch.len]) };
    }
}

// -------------------------------------------------------------- capabilities

/// Stage capability `what`, which takes no parameters.
///
/// # Safety
/// `tui` must point to a live `TUIData`.
pub unsafe fn terminfo_out(tui: *mut TUIData, what: TerminfoDef) {
    let mut params = NO_PARAMS;
    unsafe { terminfo_print(tui, what, &mut params) };
}

/// Stage capability `what` with up to three numeric parameters.
///
/// # Safety
/// `tui` must point to a live `TUIData`.
pub unsafe fn terminfo_print_num(tui: *mut TUIData, what: TerminfoDef, nums: [c_int; 3]) {
    let mut params = NO_PARAMS;
    for (slot, n) in params.iter_mut().zip(nums) {
        slot.num = n as core::ffi::c_long;
    }
    unsafe { terminfo_print(tui, what, &mut params) };
}

/// Expand capability `what` against `params` and stage the result.
///
/// The expansion writes straight into the staging buffer, so it is tried
/// twice: once where the buffer stands, and -- if there was not obviously
/// room -- again after a flush. `terminfo_fmt` consumes the parameter stack,
/// which is why the first attempt gets a copy.
///
/// # Safety
/// `tui` must point to a live `TUIData`.
pub unsafe fn terminfo_print(tui: *mut TUIData, what: TerminfoDef, params: &mut [TPVAR; 9]) {
    assert!(what < kTermCount, "capability {what} out of range");
    unsafe {
        let str = (*tui).ti.defs[what as usize];
        if str.is_null() || *str == 0 {
            return;
        }
        let expand = |tui: *mut TUIData, params: *mut TPVAR| -> size_t {
            let base = (&raw mut (*tui).buf).cast::<c_char>();
            terminfo_fmt(base.add((*tui).bufpos), base.add(BUF_SIZE), str, params)
        };
        if BUF_SIZE - (*tui).bufpos > TERMINFO_SEQ_LIMIT {
            let mut copy = *params;
            let len = expand(tui, copy.as_mut_ptr());
            if len > 0 {
                (*tui).bufpos += len;
                return;
            }
        }
        flush_buf(tui);
        let len = expand(tui, params.as_mut_ptr());
        if len > 0 {
            (*tui).bufpos += len;
        }
    }
}

/// Expand a no-parameter capability into `buf`, returning how much it used.
///
/// # Safety
/// `buf` must be valid for `len` bytes.
unsafe fn fmt_into(str: *const c_char, buf: *mut c_char, len: usize) -> size_t {
    let mut params = NO_PARAMS;
    unsafe { terminfo_fmt(buf, buf.add(len), str, params.as_mut_ptr()) }
}

// ------------------------------------------------------------------ flushing

/// Build the bytes that precede a flush: open a synchronised update if the
/// terminal has one, and otherwise hide the cursor for the duration.
///
/// # Safety
/// `tui` must be live and `buf` valid for `len` bytes.
unsafe fn flush_buf_start(tui: *mut TUIData, buf: *mut c_char, len: usize) -> size_t {
    unsafe {
        if (*tui).sync_output && (*tui).has_sync_mode {
            let bytes = SYNC_START.to_bytes();
            core::ptr::copy_nonoverlapping(bytes.as_ptr().cast::<c_char>(), buf, bytes.len());
            return bytes.len();
        }
        if !(*tui).is_invisible {
            (*tui).is_invisible = true;
            let str = (*tui).ti.defs[kTerm_cursor_invisible as usize];
            if !str.is_null() {
                return fmt_into(str, buf, len);
            }
        }
        0
    }
}

/// Build the bytes that follow a flush: close the synchronised update, then
/// bring the cursor back to whatever visibility the editor last asked for.
///
/// # Safety
/// `tui` must be live and `buf` valid for `len` bytes.
unsafe fn flush_buf_end(tui: *mut TUIData, buf: *mut c_char, len: usize) -> size_t {
    unsafe {
        let mut offset = 0;
        if (*tui).sync_output && (*tui).has_sync_mode {
            let bytes = SYNC_END.to_bytes();
            core::ptr::copy_nonoverlapping(bytes.as_ptr().cast::<c_char>(), buf, bytes.len());
            offset += bytes.len();
        }
        let want_hidden = should_invisible(&*tui);
        let str = if (*tui).is_invisible && !want_hidden {
            (*tui).is_invisible = false;
            (*tui).ti.defs[kTerm_cursor_normal as usize]
        } else if !(*tui).is_invisible && want_hidden {
            (*tui).is_invisible = true;
            (*tui).ti.defs[kTerm_cursor_invisible as usize]
        } else {
            core::ptr::null()
        };
        if !str.is_null() {
            offset += fmt_into(str, buf.add(offset), len - offset);
        }
        offset
    }
}

/// Write everything staged, wrapped in its prologue and epilogue.
///
/// Nothing is written when there is nothing staged *and* the cursor is
/// already in the right state -- the second half matters because a flush
/// with an empty buffer is still how a visibility change reaches the
/// terminal.
///
/// # Safety
/// `tui` must point to a live `TUIData`.
pub unsafe fn flush_buf(tui: *mut TUIData) {
    unsafe {
        if (*tui).bufpos == 0 && (*tui).is_invisible == should_invisible(&*tui) {
            return;
        }

        let mut pre = [0 as c_char; WRAP_BUF];
        let mut post = [0 as c_char; WRAP_BUF];
        let bufs = [
            uv_buf_t {
                base: pre.as_mut_ptr(),
                len: flush_buf_start(tui, pre.as_mut_ptr(), WRAP_BUF),
            },
            uv_buf_t {
                // An oversized write was handed straight to us and is not in
                // the staging buffer at all.
                base: if (*tui).buf_to_flush.is_null() {
                    (&raw mut (*tui).buf).cast::<c_char>()
                } else {
                    (*tui).buf_to_flush
                },
                len: (*tui).bufpos,
            },
            uv_buf_t {
                base: post.as_mut_ptr(),
                len: flush_buf_end(tui, post.as_mut_ptr(), WRAP_BUF),
            },
        ];

        if !(*tui).screenshot.is_null() {
            for b in &bufs {
                fwrite(b.base.cast(), b.len, 1, (*tui).screenshot);
            }
        } else {
            // Zeroed rather than field-by-field: `uv_write` fills the
            // request in, and every field starts as null or zero.
            let mut req: uv_write_t = core::mem::zeroed();
            let ret = uv_write(
                &raw mut req,
                (&raw mut (*tui).output_handle).cast::<uv_stream_t>(),
                bufs.as_ptr(),
                bufs.len() as core::ffi::c_uint,
                None,
            );
            if ret != 0 {
                logmsg(
                    LOGLVL_ERR,
                    core::ptr::null(),
                    c"flush_buf".as_ptr(),
                    0,
                    true,
                    c"uv_write failed: %s".as_ptr(),
                    uv_strerror(ret),
                );
            }
            // The write loop is private to the TUI and runs to completion
            // here, which is what makes the flush synchronous.
            uv_run(&raw mut (*tui).write_loop, UV_RUN_DEFAULT);
        }

        (*tui).buf_to_flush = core::ptr::null_mut();
        (*tui).bufpos = 0;
    }
}

const LOGLVL_ERR: c_int = 4;
const UV_RUN_DEFAULT: core::ffi::c_uint = 0;
