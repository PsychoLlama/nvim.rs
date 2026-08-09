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
//
// Everything here takes `&mut TUIData` and is safe on the strength of that
// type's invariant (see `TUIData`): the handles, staging buffer and terminfo
// entry it holds are the ones `tui_start` set up. What stays `unsafe` is
// what trusts something the *caller* supplies instead -- a pointer and a
// length, or a scratch buffer.

use crate::src::nvim::event::libuv::{uv_run, uv_strerror, uv_write};
use crate::src::nvim::log::{LOGLVL_ERR, logmsg_c};
use crate::src::nvim::os::libc::fwrite;
use crate::src::nvim::tui::terminfo::caps::{
    TerminfoDef, kTerm_cursor_invisible, kTerm_cursor_normal, kTermCount,
};
use crate::src::nvim::tui::terminfo::terminfo_fmt;
use crate::src::nvim::types::{
    BUF_SIZE, TPVAR, TUIData, size_t, uv_buf_t, uv_stream_t, uv_write_t,
};
use core::ffi::{CStr, c_char, c_int};

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
/// is written directly rather than copied in, since staging it could never
/// work however much room were made.
pub fn out(tui: &mut TUIData, bytes: &[u8]) {
    if bytes.len() > tui.staging.room() {
        flush(tui);
        if bytes.len() > BUF_SIZE {
            write_out(tui, Some(bytes));
            return;
        }
    }
    tui.staging.push(bytes);
}

/// Stage a NUL-terminated string, doing nothing when there is none.
pub fn out_cstr(tui: &mut TUIData, s: Option<&CStr>) {
    if let Some(s) = s {
        out(tui, s.to_bytes());
    }
}

/// Stage `len` bytes starting at `ptr`.
///
/// For the callers that hold a pointer and a length rather than a slice:
/// an API `String_0`, the URL builder's buffer, a `strlen`-measured cell.
///
/// # Safety
/// `ptr` must be valid for reads of `len` bytes.
pub unsafe fn out_raw(tui: &mut TUIData, ptr: *const c_char, len: usize) {
    // SAFETY: the caller guarantees the pointer and the length; a length of
    // zero says nothing about the pointer, so it is answered here.
    let bytes = if len == 0 {
        &[][..]
    } else {
        unsafe { core::slice::from_raw_parts(ptr.cast::<u8>(), len) }
    };
    out(tui, bytes);
}

/// Format up to three integers into `fmt` and stage the result.
///
/// This replaces a `printf` variadic whose callers pass small integers;
/// `fmt` is applied by [`core::fmt`], so the format string is checked at
/// compile time rather than trusted at runtime.
pub fn out_fmt(tui: &mut TUIData, args: core::fmt::Arguments<'_>) {
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
        out(tui, &scratch.buf[..scratch.len]);
    }
}

// -------------------------------------------------------------- capabilities

/// Stage capability `what`, which takes no parameters.
pub fn terminfo_out(tui: &mut TUIData, what: TerminfoDef) {
    let mut params = NO_PARAMS;
    terminfo_print(tui, what, &mut params);
}

/// Stage `count` copies of `byte`.
///
/// Flushing as often as it takes, so a run longer than the staging buffer is
/// written in whole buffers rather than refused.
pub fn out_repeat(tui: &mut TUIData, byte: u8, count: usize) {
    let mut left = count;
    loop {
        left -= tui.staging.fill(byte, left);
        if left == 0 {
            return;
        }
        flush(tui);
    }
}

/// Stage capability `what` with numeric parameters.
///
/// terminfo's parameter stack holds nine; anything the caller does not give
/// is zero, which is what a capability that ignores a parameter expects to
/// find.
///
pub fn terminfo_print_nums(tui: &mut TUIData, what: TerminfoDef, nums: &[c_int]) {
    let mut params = NO_PARAMS;
    for (slot, &n) in params.iter_mut().zip(nums) {
        slot.num = n as core::ffi::c_long;
    }
    terminfo_print(tui, what, &mut params);
}

/// Stage capability `what` with a single string parameter.
///
/// The parameter is read during the expansion and not kept, which is why a
/// borrow is enough.
pub fn terminfo_print_str(tui: &mut TUIData, what: TerminfoDef, s: &CStr) {
    let mut params = NO_PARAMS;
    params[0].string = s.as_ptr().cast_mut();
    terminfo_print(tui, what, &mut params);
}

/// Expand capability `what` against `params` and stage the result.
///
/// The expansion writes straight into the staging buffer, so it is tried
/// twice: once where the buffer stands, and -- if there was not obviously
/// room -- again after a flush. `terminfo_fmt` consumes the parameter stack,
/// which is why the first attempt gets a copy.
///
fn terminfo_print(tui: &mut TUIData, what: TerminfoDef, params: &mut [TPVAR; 9]) {
    assert!(what < kTermCount, "capability {what} out of range");
    // SAFETY: the capability strings belong to this TUI's terminfo entry,
    // and the expansion is bounded by the staging buffer's own end.
    unsafe {
        let str = tui.ti.defs[what as usize];
        if str.is_null() || *str == 0 {
            return;
        }
        let expand = |tui: &mut TUIData, params: *mut TPVAR| -> size_t {
            let spare = tui.staging.spare();
            let start = spare.as_mut_ptr().cast::<c_char>();
            terminfo_fmt(start, start.add(spare.len()), str, params)
        };
        if tui.staging.room() > TERMINFO_SEQ_LIMIT {
            let mut copy = *params;
            let len = expand(tui, copy.as_mut_ptr());
            if len > 0 {
                tui.staging.commit(len);
                return;
            }
        }
        flush(tui);
        let len = expand(tui, params.as_mut_ptr());
        if len > 0 {
            tui.staging.commit(len);
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
unsafe fn flush_buf_start(tui: &mut TUIData, buf: *mut c_char, len: usize) -> size_t {
    // SAFETY: the caller guarantees `buf`; the capability string comes from
    // this terminal's own entry.
    unsafe {
        if tui.sync_output && tui.has_sync_mode {
            let bytes = SYNC_START.to_bytes();
            core::ptr::copy_nonoverlapping(bytes.as_ptr().cast::<c_char>(), buf, bytes.len());
            return bytes.len();
        }
        if !tui.is_invisible {
            tui.is_invisible = true;
            let str = tui.ti.defs[kTerm_cursor_invisible as usize];
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
unsafe fn flush_buf_end(tui: &mut TUIData, buf: *mut c_char, len: usize) -> size_t {
    // SAFETY: the caller guarantees `buf`; the capability strings come from
    // this terminal's own entry.
    unsafe {
        let mut offset = 0;
        if tui.sync_output && tui.has_sync_mode {
            let bytes = SYNC_END.to_bytes();
            core::ptr::copy_nonoverlapping(bytes.as_ptr().cast::<c_char>(), buf, bytes.len());
            offset += bytes.len();
        }
        let want_hidden = should_invisible(tui);
        let str = if tui.is_invisible && !want_hidden {
            tui.is_invisible = false;
            tui.ti.defs[kTerm_cursor_normal as usize]
        } else if !tui.is_invisible && want_hidden {
            tui.is_invisible = true;
            tui.ti.defs[kTerm_cursor_invisible as usize]
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
pub fn flush(tui: &mut TUIData) {
    write_out(tui, None);
}

/// [`flush`], writing `oversized` in place of what is staged.
///
/// Nothing is staged when there is an oversized write: it got here through
/// [`out`], which flushes before handing one over.
fn write_out(tui: &mut TUIData, oversized: Option<&[u8]>) {
    // SAFETY: the handles and screenshot file are this TUI's own, the staged
    // bytes are its buffer's, an oversized write is the caller's slice and
    // outlives this call, and the two scratch buffers are on this frame.
    unsafe {
        if oversized.is_none()
            && tui.staging.is_empty()
            && tui.is_invisible == should_invisible(tui)
        {
            return;
        }

        let mut pre = [0 as c_char; WRAP_BUF];
        let mut post = [0 as c_char; WRAP_BUF];
        // The wrappers first: both decide what to send from the cursor
        // state, and both change it.
        let pre_len = flush_buf_start(tui, pre.as_mut_ptr(), WRAP_BUF);
        let post_len = flush_buf_end(tui, post.as_mut_ptr(), WRAP_BUF);
        let (body, body_len) = match oversized {
            Some(bytes) => (bytes.as_ptr().cast_mut().cast::<c_char>(), bytes.len()),
            None => tui.staging.staged(),
        };
        let bufs = [
            uv_buf_t {
                base: pre.as_mut_ptr(),
                len: pre_len,
            },
            uv_buf_t {
                base: body,
                len: body_len,
            },
            uv_buf_t {
                base: post.as_mut_ptr(),
                len: post_len,
            },
        ];

        if !tui.screenshot.is_null() {
            for b in &bufs {
                fwrite(b.base.cast(), b.len, 1, tui.screenshot);
            }
        } else {
            // Zeroed rather than field-by-field: `uv_write` fills the
            // request in, and every field starts as null or zero.
            let mut req: uv_write_t = core::mem::zeroed();
            let ret = uv_write(
                &raw mut req,
                (&raw mut tui.output_handle).cast::<uv_stream_t>(),
                bufs.as_ptr(),
                bufs.len() as core::ffi::c_uint,
                None,
            );
            if ret != 0 {
                logmsg_c!(
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
            uv_run(&raw mut tui.write_loop, UV_RUN_DEFAULT);
        }

        tui.staging.clear();
    }
}

const UV_RUN_DEFAULT: core::ffi::c_uint = 0;
