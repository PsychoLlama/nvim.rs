//! Getting a shell command's output onto the screen without drowning in it.
//!
//! `:!` is synchronous, so a command that produces megabytes would spend all
//! of its time in the UI and take CTRL-C with it. Three pieces answer that:
//! [`out_data_decide_throttle`] decides when to stop drawing and pulses a
//! `...` instead, [`out_data_ring`] keeps the last half-threshold of what was
//! skipped so the *end* of the output is still shown, and
//! [`out_data_append_to_screen`] is what actually draws — holding back an
//! incomplete UTF-8 sequence at the end of a chunk until the next one
//! arrives.
//!
//! Vim needs none of this: its `:!` runs the child on a tty in cooked mode,
//! so CTRL-C is caught by the terminal and the child can page itself. Nvim
//! uses pipes.

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

use crate::event_queue::put_job_on_fast_events;
use crate::global_cell::GlobalCell;
use crate::highlight_group::{HLF_SE, HLF_SO};
use crate::mbyte::{cluster_len, utf8len_tab_zero};
use crate::message::{msg_ext_set_append, msg_ext_set_kind, msg_multiline, msg_putchar, msg_str};
use crate::os::libc::uint64_t;
use crate::os::libc::{STDERR_FILENO, STDOUT_FILENO};
use crate::os::time::os_hrtime;
use crate::ui::state::kUIMessages;
use crate::ui::{ui_flush, ui_has};
use core::ffi::c_int;

/// One second, in nanoseconds.
const NS_1_SECOND: uint64_t = 1_000_000_000;
/// 10KB — "a few screenfuls" of data.
const OUT_DATA_THRESHOLD: usize = 1024 * 10;
/// How much skipped output [`out_data_ring`] keeps to show at the end.
const MAX_CHUNK_SIZE: usize = OUT_DATA_THRESHOLD / 2;

/// Start of the current throttle.
static started: GlobalCell<uint64_t> = GlobalCell::new(0);
/// Bytes seen since the last throttle.
static received: GlobalCell<usize> = GlobalCell::new(0);
/// "Pulse" count of the current throttle.
static visit: GlobalCell<usize> = GlobalCell::new(0);

/// Track output for the running shell command, and pulse a `...` while output
/// is being skipped.
///
/// `size` is the length of the chunk just received. **`size == 0` is the reset
/// call**: it clears the state and answers the *previous* decision, which is
/// how `do_os_system` learns that the last chunk was skipped and has to be
/// replayed.
///
/// Answers whether this chunk should be skipped.
pub(crate) fn out_data_decide_throttle(size: usize) -> bool {
    if size == 0 {
        let previous_decision = visit.get() > 0;
        started.set(0);
        received.set(0);
        visit.set(0);
        return previous_decision;
    }

    received.set(received.get() + size);
    if received.get() < OUT_DATA_THRESHOLD
        // Show at least the first chunk, however big it is.
        || (started.get() == 0 && received.get() < size + 1000)
    {
        return false;
    } else if visit.get() == 0 {
        started.set(os_hrtime());
    } else {
        let since = os_hrtime() - started.get();
        if since < visit.get() as uint64_t * (NS_1_SECOND / 10) {
            return true;
        }
        if since > 3 * NS_1_SECOND {
            received.set(0);
            visit.set(0);
            return false;
        }
    }

    visit.set(visit.get() + 1);
    // Pulse "..." at the bottom of the screen.
    let tick = visit.get() % 4;
    let pulse = match tick {
        0 => c"   ",
        1 => c".  ",
        2 => c".. ",
        _ => c"...",
    };
    if visit.get() == 1 {
        msg_str(c"...\n");
    }
    // Put the cursor back at the start of the line either side.
    msg_putchar('\r' as c_int);
    msg_str(pulse);
    msg_putchar('\r' as c_int);
    ui_flush();
    true
}

/// The tail of the output that was skipped, kept so the *end* of a throttled
/// command is still shown.
static last_skipped: GlobalCell<[u8; MAX_CHUNK_SIZE]> = GlobalCell::new([0; MAX_CHUNK_SIZE]);
static last_skipped_len: GlobalCell<usize> = GlobalCell::new(0);

/// What [`out_data_ring`] is being asked to do. Upstream spells all three as
/// one function taking a NULL pointer and a magic size.
pub(crate) enum Ring<'a> {
    /// Forget everything saved.
    Reset,
    /// Draw what is saved — the last chunk, replayed once the command ends.
    Print,
    /// Keep the tail of this chunk.
    Save(&'a [u8]),
}

/// The quasi-ring-buffer of skipped output.
pub(crate) fn out_data_ring(what: Ring) {
    match what {
        Ring::Reset => last_skipped_len.set(0),
        Ring::Print => {
            let len = last_skipped_len.get();
            last_skipped.with(|saved| {
                let mut count = len;
                out_data_append_to_screen(&saved[..len], &mut count, STDOUT_FILENO, true);
            });
        }
        Ring::Save(output) if output.len() >= MAX_CHUNK_SIZE => {
            // Only the tail fits, so only the tail is kept.
            let start = output.len() - MAX_CHUNK_SIZE;
            last_skipped.with_mut(|saved| saved.copy_from_slice(&output[start..]));
            last_skipped_len.set(MAX_CHUNK_SIZE);
        }
        Ring::Save(output) if !output.is_empty() => {
            let len = last_skipped_len.get();
            // How much of the old data still fits in front of the new.
            let keep_len = len.min(MAX_CHUNK_SIZE - output.len());
            let keep_start = len - keep_len;
            last_skipped.with_mut(|saved| {
                saved.copy_within(keep_start..keep_start + keep_len, 0);
                saved[keep_len..keep_len + output.len()].copy_from_slice(output);
            });
            last_skipped_len.set(keep_len + output.len());
        }
        Ring::Save(_) => {}
    }
}

/// Draw one chunk of output that came on `fd`.
fn draw_chunk(text: &[u8], fd: c_int) {
    let is_stderr = fd == STDERR_FILENO;
    let hl = if is_stderr { HLF_SE } else { HLF_SO } as c_int;
    msg_ext_set_kind(if is_stderr {
        c"shell_err"
    } else {
        c"shell_out"
    });
    msg_ext_set_append(true);
    let mut need_clear = true;
    msg_multiline(text, hl, false, false, &mut need_clear);
    ui_flush();
}

/// Append `output` to the last screen line, holding back a trailing
/// incomplete UTF-8 sequence by lowering `count`.
///
/// This is deliberately not exact: a continuation byte that is already
/// invalid is still buffered, and characters are not composed across a chunk
/// boundary. Both are corrected when this moves to the vterm implementation.
pub(crate) fn out_data_append_to_screen(output: &[u8], count: &mut usize, fd: c_int, eof: bool) {
    let mut at = 0;
    while at < *count {
        let step = if output[at] == 0 {
            1
        } else {
            cluster_len(&output[at..*count])
        };
        if !eof && step == 1 && utf8len_tab_zero[output[at] as usize] as usize > *count - at {
            // An incomplete sequence at the end: leave it for the next chunk.
            *count = at;
            break;
        }
        at += step;
    }

    // Done after `uv_run` to avoid recursing into a `vim.ui_attach()`
    // msg_show callback. #38664
    let text = output[..*count].to_vec();
    if ui_has(kUIMessages) {
        put_job_on_fast_events(move || draw_chunk(&text, fd));
    } else {
        draw_chunk(&text, fd);
    }
}

/// What `do_os_system` does with a chunk of output when it goes to the
/// screen rather than to the caller: draw it, or -- above the threshold --
/// save it, to show later if it turns out to have been the last chunk.
/// Answers how many bytes were consumed; an incomplete UTF-8 sequence at the
/// end is left for the next chunk unless `eof`.
pub(crate) fn out_data(data: &[u8], fd: c_int, eof: bool) -> usize {
    let mut count = data.len();
    if count == 0 {
        return count;
    }
    if out_data_decide_throttle(count) {
        out_data_ring(Ring::Save(data));
    } else {
        out_data_append_to_screen(data, &mut count, fd, eof);
    }
    count
}
