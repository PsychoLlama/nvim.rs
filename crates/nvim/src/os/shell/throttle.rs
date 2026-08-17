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

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::highlight_group::{HLF_SE, HLF_SO};
use crate::mbyte::{utf8len_tab_zero, utfc_ptr2len_len};
use crate::memory::{xfree, xmemdupz};
use crate::message::{msg_ext_set_append, msg_ext_set_kind, msg_multiline, msg_putchar};
use crate::os::time::os_hrtime;
use crate::types::libc::{STDERR_FILENO, STDOUT_FILENO};
use crate::types::ui::kUIMessages;
use crate::types::{Event, RStream, String_0, intptr_t, uint64_t};
use crate::ui::{ui_flush, ui_has};

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
    // SAFETY: static messages, none of which is a format string.
    unsafe {
        if visit.get() == 1 {
            msg_puts(c"...\n".as_ptr());
        }
        // Put the cursor back at the start of the line either side.
        msg_putchar('\r' as c_int);
        msg_puts(pulse.as_ptr());
        msg_putchar('\r' as c_int);
        ui_flush();
    }
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

/// Draw one chunk, on the fast-events queue when a UI is handling messages.
///
/// # Safety
/// An `argv_callback`: `argv[0]` must be an owned allocation of `argv[1]`
/// bytes, which this takes over and frees, and `argv[2]` a file descriptor.
unsafe extern "C" fn out_data_event(argv: *mut *mut c_void) {
    // SAFETY: the caller's contract.
    unsafe {
        let text = *argv.add(0) as *mut c_char;
        let count = (*argv.add(1)).addr();
        let fd = (*argv.add(2)).addr() as c_int;

        let is_stderr = fd == STDERR_FILENO;
        let hl = if is_stderr { HLF_SE } else { HLF_SO } as c_int;
        msg_ext_set_kind(if is_stderr {
            c"shell_err".as_ptr()
        } else {
            c"shell_out".as_ptr()
        });
        msg_ext_set_append(true);
        let mut need_clear = true;
        msg_multiline(
            String_0 {
                data: text,
                size: count,
            },
            hl,
            false,
            false,
            &raw mut need_clear,
        );
        xfree(text.cast());
        ui_flush();
    }
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
        // SAFETY: `output[at..*count]` is in bounds, and `utfc_ptr2len_len`
        // is bounded by the length it is handed.
        let step = unsafe {
            if output[at] == 0 {
                1
            } else {
                utfc_ptr2len_len(
                    output[at..].as_ptr().cast::<c_char>(),
                    (*count - at) as c_int,
                ) as usize
            }
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
    // SAFETY: `output[..*count]` is in bounds; `str` is an owned allocation
    // that `out_data_event` takes over, whichever way it is reached.
    unsafe {
        let text = xmemdupz(output.as_ptr().cast(), *count) as *mut c_char;
        let mut argv: [*mut c_void; 3] = [
            text.cast(),
            ptr::with_exposed_provenance_mut(*count),
            ptr::with_exposed_provenance_mut(fd as intptr_t as usize),
        ];
        if ui_has(kUIMessages) {
            multiqueue_put_event(
                (*main_loop.ptr()).fast_events,
                Event::new(Some(out_data_event), [argv[0], argv[1], argv[2]]),
            );
        } else {
            out_data_event(argv.as_mut_ptr());
        }
    }
}

/// The `stream_read_cb` `do_os_system` uses when the output goes to the
/// screen rather than to the caller.
///
/// # Safety
/// An `stream_read_cb`: `ptr` must be readable for `count` bytes and `stream`
/// live.
pub(crate) unsafe extern "C" fn out_data_cb(
    stream: *mut RStream,
    ptr: *const c_char,
    mut count: size_t,
    _data: *mut c_void,
    eof: bool,
) -> size_t {
    if count == 0 {
        return count;
    }
    // SAFETY: the caller's contract.
    let data = unsafe { core::slice::from_raw_parts(ptr.cast::<u8>(), count) };
    if out_data_decide_throttle(count) {
        // Above the threshold: save it, and show it later if it turns out to
        // have been the last chunk.
        out_data_ring(Ring::Save(data));
    } else {
        // SAFETY: `stream` is live.
        out_data_append_to_screen(data, &mut count, unsafe { (*stream).s.fd }, eof);
    }
    count
}
