//! Writing messages, and the retired subscription API.
//!
//! `write_msg` is the line-buffered writer behind `nvim_out_write` and the two
//! `nvim_err_write` spellings: it accumulates until a newline, so a client may
//! send a message in pieces.  `nvim_notify` is the message-with-a-level shim.
//! `nvim_subscribe`/`nvim_unsubscribe` are empty: the broadcast events they
//! filtered no longer exist.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::*;
use crate::guard::Suppress;
use crate::message::{emsg, msg};
use crate::types::builders::ArrayBuf;
use core::ffi::CStr;

pub fn nvim_subscribe(_channel_id: uint64_t, _event: String_0) {}

pub fn nvim_unsubscribe(_channel_id: uint64_t, _event: String_0) {}

/// The line `nvim_out_write` is accumulating, up to its newline.
static OUT_LINE: GlobalCell<Vec<u8>> = GlobalCell::new(Vec::new());
/// The same for the two `nvim_err_write` spellings.
static ERR_LINE: GlobalCell<Vec<u8>> = GlobalCell::new(Vec::new());

fn write_msg(message: String_0, to_err: bool, writeln: bool) {
    let line_buf = if to_err { &ERR_LINE } else { &OUT_LINE };
    let push = |byte: u8| line_buf.with_mut(|line| line.push(byte));
    // Publish the line the buffer holds and empty it. The line is out of
    // its cell while it is shown: `msg`/`emsg` can re-enter this function.
    let flush = || {
        let mut line = line_buf.take();
        line.push(0);
        let text = CStr::from_bytes_until_nul(&line).expect("the line was just terminated");
        if to_err {
            emsg(text);
        } else {
            msg(text, 0);
        }
        if msg_silent.get() == 0 {
            msg_didout.set(true);
        }
        // Back to its minimum size; whatever a re-entered write left in the
        // cell meanwhile is dropped, as upstream's `size = 0` dropped it.
        line.clear();
        line.shrink_to(LINE_BUFFER_MIN_SIZE as usize);
        line_buf.set(line);
    };

    let no_prompt = Suppress::wait_return();
    for &byte in message.as_bytes() {
        if got_int.get() {
            break;
        }
        match byte {
            b'\n' => flush(),
            // A NUL in the text stands for a newline, as it does everywhere
            // a buffer line is passed as a C string.
            0 => push(b'\n'),
            _ => push(byte),
        }
    }
    if writeln {
        flush();
    }
    drop(no_prompt);
    msg_end();
}

pub fn nvim_out_write(str: String_0) {
    write_msg(str, false, false);
}

pub fn nvim_err_write(str: String_0) {
    write_msg(str, true, false);
}

pub fn nvim_err_writeln(str: String_0) {
    write_msg(str, true, true);
}

/// Hand the message to `vim.notify`.
pub fn nvim_notify(msg_0: String_0, log_level: Integer, opts: ApiDict) -> Result<Object, Error> {
    let mut args = ArrayBuf::<3>::new();
    args.push(Object::string(msg_0));
    args.push(Object::integer(log_level));
    args.push(Object::dict(opts));
    nlua_exec(b"return vim.notify(...)", None, args.array(), kRetObject)
}
