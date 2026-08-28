//! The message scrollback, which `g<` and the pager page through.
//!
//! Every line [`crate::message::msg_puts_display`] emits is also
//! copied into a linked list of [`msgchunk_T`] chunks ([`store_sb_text`]), so
//! that the pager can scroll backwards past what the screen still holds.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use core::ffi::{c_char, c_int, c_uint};
use core::{mem, ptr};

/// The most recently displayed chunk of message text.
pub(crate) static last_msgchunk: GlobalCell<*mut msgchunk_T> = GlobalCell::new(ptr::null_mut());

/// Whether, and how much of, the scrollback to drop before the next message.
static do_clear_sb_text: GlobalCell<sb_clear_T> = GlobalCell::new(SB_CLEAR_NONE);

/// A chunk's text lives in the same allocation, right after the header.
///
/// # Safety
/// `mp` must point at a chunk allocated by [`store_sb_text`].
unsafe fn sb_text(mp: *mut msgchunk_T) -> *mut c_char {
    // Not `(*mp).sb_text.as_mut_ptr()`: the field is a zero-length array, so
    // the autoref covers no bytes and the pointer carries no provenance for
    // the text that follows it.
    unsafe { (&raw mut (*mp).sb_text).cast() }
}

/// Remember `*sb_str ..= s` for scrolling back over later.
///
/// `finish` marks the chunk as ending its screen line. `sb_col` is the column
/// the run started at, so the pager can put it back where it was.
///
/// # Safety
/// `*sb_str` and `s` must point into the same readable buffer, with `s` at or
/// after `*sb_str`.
pub(crate) unsafe fn store_sb_text(
    sb_str: *mut *const c_char,
    s: *const c_char,
    hl_id: c_int,
    sb_col: *mut c_int,
    finish: c_int,
) {
    if do_clear_sb_text.get() == SB_CLEAR_ALL || do_clear_sb_text.get() == SB_CLEAR_CMDLINE_DONE {
        unsafe { clear_sb_text(do_clear_sb_text.get() == SB_CLEAR_ALL) };
        unsafe { msg_sb_eol() }; // prevent messages from overlapping
        if do_clear_sb_text.get() == SB_CLEAR_CMDLINE_DONE
            && s > unsafe { *sb_str }
            && unsafe { **sb_str } == b'\n' as c_char
        {
            unsafe { *sb_str = (*sb_str).add(1) };
        }
        do_clear_sb_text.set(SB_CLEAR_NONE);
    }

    if s > unsafe { *sb_str } {
        let len = unsafe { s.offset_from(*sb_str) as size_t };
        let mp: *mut msgchunk_T =
            unsafe { xmalloc(mem::offset_of!(msgchunk_T, sb_text) + len + 1) }.cast();
        unsafe { (*mp).sb_eol = finish as c_char };
        unsafe { (*mp).sb_msg_col = *sb_col };
        unsafe { (*mp).sb_hl_id = hl_id };
        unsafe { ptr::copy_nonoverlapping(*sb_str, sb_text(mp), len) };
        unsafe { *sb_text(mp).add(len) = 0 };

        unsafe { (*mp).sb_prev = last_msgchunk.get() };
        unsafe { (*mp).sb_next = ptr::null_mut() };
        if !last_msgchunk.get().is_null() {
            unsafe { (*last_msgchunk.get()).sb_next = mp };
        }
        last_msgchunk.set(mp);
    } else if finish != 0 && !last_msgchunk.get().is_null() {
        unsafe { (*last_msgchunk.get()).sb_eol = 1 };
    }

    unsafe { *sb_str = s };
    unsafe { *sb_col = 0 };
}

/// Finished showing messages: clear the scroll-back text on the next one.
pub unsafe fn may_clear_sb_text() {
    unsafe { msg_ext_ui_flush() }; // ensure messages until now are emitted
    do_clear_sb_text.set(SB_CLEAR_ALL);
    do_clear_hist_temp.set(true);
}

/// Starting to edit the command line: do not clear messages now.
pub unsafe fn sb_text_start_cmdline() {
    if do_clear_sb_text.get() == SB_CLEAR_CMDLINE_BUSY {
        // A recursive command line: the outer one need not be remembered,
        // it will be redrawn when this level returns.
        unsafe { sb_text_restart_cmdline() };
    } else {
        unsafe { msg_sb_eol() };
        do_clear_sb_text.set(SB_CLEAR_CMDLINE_BUSY);
    }
}

/// Redrawing the command line: drop the last unfinished line.
pub unsafe fn sb_text_restart_cmdline() {
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
pub unsafe fn sb_text_end_cmdline() {
    do_clear_sb_text.set(SB_CLEAR_CMDLINE_DONE);
}

/// Forget the remembered text. With `all` false the last screen line is kept.
pub unsafe fn clear_sb_text(all: bool) {
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
pub unsafe fn show_sb_text() {
    if ui_has(kUIMessages) {
        let mut ea = exarg_T {
            arg: c"".as_ptr().cast_mut(),
            skip: 1,
            ..exarg_T::default()
        };
        unsafe { ex_messages(&raw mut ea) };
        return;
    }
    // Only show something when there is more than one line: a command
    // with no output would otherwise leave one line looking odd.
    let mp = unsafe { msg_sb_start(last_msgchunk.get()) };
    if mp.is_null() || unsafe { (*mp).sb_prev }.is_null() {
        unsafe { vim_beep(kOptBoFlagMess as c_uint) };
    } else {
        unsafe { do_more_prompt(b'G' as c_int) };
        unsafe { wait_return(0) };
    }
}

/// Walk back to the chunk that starts the screen line `mps` is part of.
pub(crate) unsafe fn msg_sb_start(mps: *mut msgchunk_T) -> *mut msgchunk_T {
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
pub unsafe fn msg_sb_eol() {
    if !last_msgchunk.get().is_null() {
        unsafe { (*last_msgchunk.get()).sb_eol = 1 };
    }
}

/// Redisplay one remembered screen line at `row`, answering the chunk the
/// next line starts at (null at the end of the list).
pub(crate) unsafe fn disp_sb_line(row: c_int, smp: *mut msgchunk_T) -> *mut msgchunk_T {
    let mut mp = smp;
    loop {
        msg_row.set(row);
        msg_col.set(unsafe { (*mp).sb_msg_col });
        unsafe { msg_puts_display(sb_text(mp), -1, (*mp).sb_hl_id, true) };
        if unsafe { (*mp).sb_eol } != 0 || unsafe { (*mp).sb_next }.is_null() {
            break;
        }
        mp = unsafe { (*mp).sb_next };
    }
    unsafe { (*mp).sb_next }
}
