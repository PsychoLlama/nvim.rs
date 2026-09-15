//! The message scrollback, which `g<` and the pager page through.
//!
//! Every line [`crate::message::msg_bytes_to_grid`] emits is also
//! copied into a linked list of [`MsgChunk`] chunks ([`store_sb_text`]), so
//! that the pager can scroll backwards past what the screen still holds.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::cstr;
use core::ffi::{c_char, c_int, c_uint};
use core::{mem, ptr};

/// The most recently displayed chunk of message text.
pub(crate) static last_msgchunk: GlobalCell<*mut MsgChunk> = GlobalCell::new(ptr::null_mut());

/// Whether, and how much of, the scrollback to drop before the next message.
static do_clear_sb_text: GlobalCell<ScrollbackClear> = GlobalCell::new(SB_CLEAR_NONE);

/// A chunk's text lives in the same allocation, right after the header.
///
/// # Safety
/// `mp` must point at a chunk allocated by [`store_sb_text`].
unsafe fn sb_text(mp: *mut MsgChunk) -> *mut c_char {
    // Not `(*mp).sb_text.as_mut_ptr()`: the field is a zero-length array, so
    // the autoref covers no bytes and the pointer carries no provenance for
    // the text that follows it.
    unsafe { (&raw mut (*mp).sb_text).cast() }
}

/// Remember `run` for scrolling back over later.
///
/// `finish` marks the chunk as ending its screen line. `sb_col` is the column
/// the run started at, so the pager can put it back where it was; it is reset
/// here, because the next run starts at the left margin of whatever comes
/// after this one.
pub(crate) fn store_sb_text(run: &[u8], hl_id: c_int, sb_col: &mut c_int, finish: bool) {
    let mut run = run;
    if do_clear_sb_text.get() == SB_CLEAR_ALL || do_clear_sb_text.get() == SB_CLEAR_CMDLINE_DONE {
        clear_sb_text(do_clear_sb_text.get() == SB_CLEAR_ALL);
        msg_sb_eol(); // prevent messages from overlapping
        if do_clear_sb_text.get() == SB_CLEAR_CMDLINE_DONE && run.first() == Some(&b'\n') {
            run = &run[1..];
        }
        do_clear_sb_text.set(SB_CLEAR_NONE);
    }

    if !run.is_empty() {
        let len = run.len();
        // SAFETY: the chunk's text lives in the same allocation, right after
        // the header, which is why the size is asked for that way.
        let mp: *mut MsgChunk =
            unsafe { xmalloc(mem::offset_of!(MsgChunk, sb_text) + len + 1) }.cast();
        // SAFETY: the allocation is live and holds a header and `len + 1`
        // bytes of text.
        unsafe {
            (*mp).sb_eol = c_char::from(finish);
            (*mp).sb_msg_col = *sb_col;
            (*mp).sb_hl_id = hl_id;
            ptr::copy_nonoverlapping(run.as_ptr().cast::<c_char>(), sb_text(mp), len);
            *sb_text(mp).add(len) = 0;

            (*mp).sb_prev = last_msgchunk.get();
            (*mp).sb_next = ptr::null_mut();
        }
        if !last_msgchunk.get().is_null() {
            // SAFETY: the list's tail is a live chunk.
            unsafe { (*last_msgchunk.get()).sb_next = mp };
        }
        last_msgchunk.set(mp);
    } else if finish && !last_msgchunk.get().is_null() {
        // SAFETY: as above.
        unsafe { (*last_msgchunk.get()).sb_eol = 1 };
    }

    *sb_col = 0;
}

/// Finished showing messages: clear the scroll-back text on the next one.
pub fn may_clear_sb_text() {
    msg_ext_ui_flush(); // ensure messages until now are emitted
    do_clear_sb_text.set(SB_CLEAR_ALL);
    do_clear_hist_temp.set(true);
}

/// Starting to edit the command line: do not clear messages now.
pub fn sb_text_start_cmdline() {
    if do_clear_sb_text.get() == SB_CLEAR_CMDLINE_BUSY {
        // A recursive command line: the outer one need not be remembered,
        // it will be redrawn when this level returns.
        sb_text_restart_cmdline();
    } else {
        msg_sb_eol();
        do_clear_sb_text.set(SB_CLEAR_CMDLINE_BUSY);
    }
}

/// Redrawing the command line: drop the last unfinished line.
pub fn sb_text_restart_cmdline() {
    // Needed when returning from a nested command line.
    do_clear_sb_text.set(SB_CLEAR_CMDLINE_BUSY);
    if last_msgchunk.get().is_null() || unsafe { (*last_msgchunk.get()).sb_eol } != 0 {
        // No unfinished line: don't clear anything.
        return;
    }

    let mut tofree = unsafe { msg_sb_start(last_msgchunk.get()) };
    last_msgchunk.set(unsafe { (*tofree).sb_prev });
    if !last_msgchunk.get().is_null() {
        unsafe { (*last_msgchunk.get()).sb_next = ptr::null_mut() };
    }
    while !tofree.is_null() {
        let next = unsafe { (*tofree).sb_next };
        unsafe { xfree(tofree.cast()) };
        tofree = next;
    }
}

/// Finished editing the command line: clear the old lines, but the last one
/// only later.
pub fn sb_text_end_cmdline() {
    do_clear_sb_text.set(SB_CLEAR_CMDLINE_DONE);
}

/// Forget the remembered text. With `all` false the last screen line is kept.
pub fn clear_sb_text(all: bool) {
    // The slot holding the newest chunk to drop: either the list head, or
    // the `sb_prev` of the line that is being kept.
    let lastp = if all {
        last_msgchunk.ptr()
    } else {
        if last_msgchunk.get().is_null() {
            return;
        }
        unsafe { &raw mut (*msg_sb_start(last_msgchunk.get())).sb_prev }
    };
    while !unsafe { (*lastp).is_null() } {
        let prev = unsafe { (**lastp).sb_prev };
        unsafe { xfree((*lastp).cast()) };
        unsafe { *lastp = prev };
    }
}

/// The `g<` command.
pub fn show_sb_text() {
    if ui_has(kUIMessages) {
        let mut ea = ExArg {
            arg: c"".as_ptr().cast_mut(),
            skip: 1,
            ..ExArg::default()
        };
        ex_messages(&mut ea);
        return;
    }
    // Only show something when there is more than one line: a command
    // with no output would otherwise leave one line looking odd.
    let mp = unsafe { msg_sb_start(last_msgchunk.get()) };
    if mp.is_null() || unsafe { (*mp).sb_prev }.is_null() {
        vim_beep(kOptBoFlagMess as c_uint);
    } else {
        do_more_prompt(c_int::from(b'G'));
        wait_return(0);
    }
}

/// Walk back to the chunk that starts the screen line `mps` is part of.
///
/// # Safety
///
/// `mps` must point at a live `MsgChunk`, unaliased for the call.
pub(crate) unsafe fn msg_sb_start(mps: *mut MsgChunk) -> *mut MsgChunk {
    let mut mp = mps;
    while !mp.is_null()
        && !unsafe { (*mp).sb_prev }.is_null()
        && unsafe { (*(*mp).sb_prev).sb_eol } == 0
    {
        mp = unsafe { (*mp).sb_prev };
    }
    mp
}

/// Mark the last chunk as finishing its screen line.
pub fn msg_sb_eol() {
    if !last_msgchunk.get().is_null() {
        unsafe { (*last_msgchunk.get()).sb_eol = 1 };
    }
}

/// Redisplay one remembered screen line at `row`, answering the chunk the
/// next line starts at (null at the end of the list).
///
/// # Safety
///
/// `smp` must point at a live `MsgChunk`, unaliased for the call.
pub(crate) unsafe fn disp_sb_line(row: c_int, smp: *mut MsgChunk) -> *mut MsgChunk {
    let mut mp = smp;
    loop {
        msg_row.set(row);
        msg_col.set(unsafe { (*mp).sb_msg_col });
        msg_bytes_to_grid(
            unsafe { cstr::bytes_at(sb_text(mp)) },
            unsafe { (*mp).sb_hl_id },
            true,
        );
        if unsafe { (*mp).sb_eol } != 0 || unsafe { (*mp).sb_next }.is_null() {
            break;
        }
        mp = unsafe { (*mp).sb_next };
    }
    unsafe { (*mp).sb_next }
}
