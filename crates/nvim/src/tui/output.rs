//! The TUI's write path.
//!
//! Everything the TUI emits is staged in one buffer and leaves in a single
//! `uv_write` per flush, wrapped in whatever this terminal needs around a
//! screen update: synchronised-output brackets when it has them, and a
//! cursor hide/show pair when it does not, so a repaint is not watched
//! happening a cell at a time.
//!
//! The wrappers are why a flush writes three buffers rather than one -- a
//! prologue, the staged bytes, an epilogue -- and why [`flush`] is the
//! only function here that talks to libuv.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `tui/{tui,input,output,events}.rs` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
//
// Everything here takes `&mut TUIData` and is safe on the strength of that
// type's invariant (see `TUIData`): the handles, staging buffer and terminfo
// entry it holds are the ones `tui_start` set up. What stays `unsafe` is the
// one `uv_write` a flush makes.

use crate::event::libuv::{uv_run, uv_write};
use crate::event::uv::{uv_buf_t, uv_stream_t, uv_write_t};
use crate::log::{LOGLVL_ERR, logmsg};
use crate::message_fmt::msg_cstr;
use crate::os::fs::os_strerror;
use crate::tui::state::{BUF_SIZE, TUIData};
use crate::tui::terminfo::caps::{
    TerminfoDef, kTerm_cursor_invisible, kTerm_cursor_normal, kTermCount,
};
use crate::tui::terminfo::param::Param;
use crate::tui::terminfo::terminfo_fmt;
use core::ffi::{CStr, c_int};
use std::io::Write;

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
const NO_PARAMS: [Param<'static>; 9] = [Param {
    num: 0,
    string: None,
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
        slot.num = i64::from(n);
    }
    terminfo_print(tui, what, &mut params);
}

/// Stage capability `what` with a single string parameter.
///
/// The parameter is read during the expansion and not kept, which is why a
/// borrow is enough.
pub fn terminfo_print_str(tui: &mut TUIData, what: TerminfoDef, s: &CStr) {
    let mut params = NO_PARAMS;
    params[0].string = Some(s.to_bytes());
    terminfo_print(tui, what, &mut params);
}

/// Expand capability `what` against `params` and stage the result.
///
/// The expansion writes straight into the staging buffer, so it is tried
/// twice: once where the buffer stands, and -- if there was not obviously
/// room -- again after a flush. `terminfo_fmt` consumes the parameter stack,
/// which is why the first attempt gets a copy.
///
fn terminfo_print(tui: &mut TUIData, what: TerminfoDef, params: &mut [Param<'_>; 9]) {
    assert!(what < kTermCount, "capability {what} out of range");
    if tui.ti.def(what).is_none_or(<[u8]>::is_empty) {
        return;
    }
    if tui.staging.room() > TERMINFO_SEQ_LIMIT {
        let mut copy = *params;
        let len = expand_staged(tui, what, &mut copy);
        if len > 0 {
            tui.staging.commit(len);
            return;
        }
    }
    flush(tui);
    let len = expand_staged(tui, what, params);
    if len > 0 {
        tui.staging.commit(len);
    }
}

/// Expand capability `what` into the staging buffer's spare room, answering
/// how much it wrote; [`Staging::commit`](crate::tui::state::Staging::commit)
/// stages it.
fn expand_staged(tui: &mut TUIData, what: TerminfoDef, params: &mut [Param<'_>; 9]) -> usize {
    let TUIData { ti, staging, .. } = tui;
    terminfo_fmt(staging.spare(), ti.def(what).unwrap_or_default(), params)
}

/// Expand a no-parameter capability into `buf`, returning how much it used.
fn fmt_into(capability: &[u8], buf: &mut [u8]) -> usize {
    let mut params = NO_PARAMS;
    terminfo_fmt(buf, capability, &mut params)
}

// ------------------------------------------------------------------ flushing

/// Build the bytes that precede a flush: open a synchronised update if the
/// terminal has one, and otherwise hide the cursor for the duration.
fn flush_buf_start(tui: &mut TUIData, buf: &mut [u8]) -> usize {
    if tui.sync_output && tui.has_sync_mode {
        let bytes = SYNC_START.to_bytes();
        buf[..bytes.len()].copy_from_slice(bytes);
        return bytes.len();
    }
    if !tui.is_invisible {
        tui.is_invisible = true;
        if let Some(seq) = tui.ti.def(kTerm_cursor_invisible) {
            return fmt_into(seq, buf);
        }
    }
    0
}

/// Build the bytes that follow a flush: close the synchronised update, then
/// bring the cursor back to whatever visibility the editor last asked for.
fn flush_buf_end(tui: &mut TUIData, buf: &mut [u8]) -> usize {
    let mut offset = 0;
    if tui.sync_output && tui.has_sync_mode {
        let bytes = SYNC_END.to_bytes();
        buf[..bytes.len()].copy_from_slice(bytes);
        offset += bytes.len();
    }
    let want_hidden = should_invisible(tui);
    let slot = if tui.is_invisible && !want_hidden {
        tui.is_invisible = false;
        Some(kTerm_cursor_normal)
    } else if !tui.is_invisible && want_hidden {
        tui.is_invisible = true;
        Some(kTerm_cursor_invisible)
    } else {
        None
    };
    if let Some(seq) = slot.and_then(|slot| tui.ti.def(slot)) {
        offset += fmt_into(seq, &mut buf[offset..]);
    }
    offset
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
    if oversized.is_none() && tui.staging.is_empty() && tui.is_invisible == should_invisible(tui) {
        return;
    }

    let mut pre = [0u8; WRAP_BUF];
    let mut post = [0u8; WRAP_BUF];
    // The wrappers first: both decide what to send from the cursor state,
    // and both change it.
    let pre_len = flush_buf_start(tui, &mut pre);
    let post_len = flush_buf_end(tui, &mut post);

    if let Some(file) = tui.screenshot.as_mut() {
        let body = oversized.unwrap_or_else(|| tui.staging.as_bytes());
        // A screenshot is a test's record; a short write shows up there.
        let _ = file
            .write_all(&pre[..pre_len])
            .and_then(|()| file.write_all(body))
            .and_then(|()| file.write_all(&post[..post_len]));
    } else {
        let (body, body_len) = match oversized {
            Some(bytes) => (bytes.as_ptr().cast_mut().cast(), bytes.len()),
            None => tui.staging.staged(),
        };
        let bufs = [
            uv_buf_t {
                base: pre.as_mut_ptr().cast(),
                len: pre_len,
            },
            uv_buf_t {
                base: body,
                len: body_len,
            },
            uv_buf_t {
                base: post.as_mut_ptr().cast(),
                len: post_len,
            },
        ];
        // SAFETY: the output handle is this TUI's own, open since
        // `tui_start`; the three buffers are this frame's scratch, the
        // staging buffer and the caller's slice, all of which outlive the
        // write, which the loop below runs to completion. The request is
        // zeroed rather than filled field by field: `uv_write` fills it in,
        // and every field starts as null or zero.
        unsafe {
            let mut req: uv_write_t = core::mem::zeroed();
            let ret = uv_write(
                &raw mut req,
                (&raw mut tui.output_handle).cast::<uv_stream_t>(),
                bufs.as_ptr(),
                bufs.len().try_into().expect("`bufs` holds three buffers"),
                None,
            );
            if ret != 0 {
                logmsg!(
                    LOGLVL_ERR,
                    c"flush_buf",
                    0,
                    "uv_write failed: {}",
                    msg_cstr(os_strerror(ret))
                );
            }
            // The write loop is private to the TUI and runs to completion
            // here, which is what makes the flush synchronous.
            uv_run(&raw mut tui.write_loop, UV_RUN_DEFAULT);
        }
    }

    tui.staging.clear();
}

const UV_RUN_DEFAULT: core::ffi::c_uint = 0;
