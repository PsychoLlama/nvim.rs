//! Reading OS input, and the buffer it lands in.
//!
//! # Boundary
//!
//! stdin is read by libuv through [`RStream`], which calls
//! [`input_read_cb`] with whatever arrived; everything else here is bytes
//! moving between that callback, a fixed buffer, and `getchar.c`'s typeahead.
//!
//! The buffer is a plain array plus a read and a write offset into it. It is
//! never a ring: [`input_enqueue_raw`] compacts what is unread back to the
//! front when it needs the room, which is why the two offsets only ever move
//! forward between compactions.
//!
//! This file also owns the CursorHold timer (there is nowhere better for it
//! yet — upstream's TODO wants it to become a `state_check` timer) and the
//! `<`*col*`,`*row*`>` suffix that a mouse key sequence may carry.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `os/` row in docs/perimeter.md.
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::autocmd::fire_autocmds;
use crate::autocmd::state::did_cursorhold;
use crate::autocmd::trigger_cursorhold;
use crate::cstr;
use crate::event::libuv::uv_guess_handle;
use crate::event::r#loop::{loop_poll_events, process_events_until};
use crate::event::multiqueue::{ChildQueue, queue_in};
use crate::event::multiqueue::{multiqueue_empty, multiqueue_process_events, multiqueue_put_event};
use crate::event::rstream::{rstream_init_fd, rstream_may_close, rstream_start, rstream_stop};
use crate::getchar::state::{ctrl_c_interrupts, got_int, mapped_ctrl_c, typebuf_was_filled};
use crate::getchar::{before_blocking, typebuf_changed};
use crate::global_cell::GlobalCell;
use crate::keycodes::{
    Ctrl_C, FSK_KEYCODE, K_SPECIAL, KE_EVENT, KE_FILLER, KE_LEFTMOUSE, KE_MIDDLEMOUSE,
    KE_MOUSEDOWN, KE_MOUSEMOVE, KE_MOUSERIGHT, KE_RIGHTMOUSE, KE_RIGHTRELEASE, KE_X1MOUSE,
    KE_X2MOUSE, KE_X2RELEASE, KS_EXTRA, KS_MODIFIER, KS_SPECIAL,
};
use crate::keycodes::{ModMask, special_key_at, special_key_bytes};
use crate::log::{LOGLVL_DBG, logmsg};
use crate::message_fmt::c_str;
use crate::mouse::state::{mouse_col, mouse_grid, mouse_row};
use crate::msgpack_rpc::channel::ch_before_blocking_events;
use crate::option::vars::{p_mouset, p_ut};
use crate::os::cshim::gettext;
use crate::os::time::os_hrtime;
use crate::profile::do_profiling;
use crate::profile::{prof_input_end, prof_input_start};
use crate::startup::{getout, main_loop, preserve_exit, silent_mode, used_stdin};
use crate::state::mode::State;
use crate::state::{MODE_INSERT, get_real_state};
use crate::types::AutoEvent;
use crate::types::libc::STDIN_FILENO;
use crate::types::{
    Event, KeyExtra, MultiQueue, RStream, Stream, String_0, size_t, uint8_t, uint64_t,
    uv_handle_type,
};
use crate::ui::state::{Columns, Rows, current_ui};
use crate::winlayer::Buf;
use core::ffi::{c_char, c_int, c_uint, c_void};
use core::mem::MaybeUninit;
use core::ptr;

const UV_TTY: uv_handle_type = 14;
const PROF_YES: c_int = 1;
/// The longest byte sequence one key can become: `K_SPECIAL KS_MODIFIER mod`
/// plus `K_SPECIAL KS_EXTRA code`.
const MAX_KEY_CODE_LEN: usize = 6;
const READ_BUFFER_SIZE: usize = 0xfff;
const INPUT_BUFFER_SIZE: usize = READ_BUFFER_SIZE * 4 + MAX_KEY_CODE_LEN;

/// Upstream writes `{ .s.closed = true }`, which C defines as zero-filling
/// every other field — eighty lines of `NULL`/`None`/`0` if transcribed.
///
/// SAFETY: every field of an [`RStream`] is a raw pointer, an integer, a
/// `bool`, an `Option<fn>` or a `#[repr(C)]` union of those, and the all-zero
/// bit pattern is a valid value of each (null, 0, false, and `None` by the
/// null-pointer optimisation).
const ZEROED_RSTREAM: RStream = unsafe { MaybeUninit::zeroed().assume_init() };

/// stdin. Starts closed, because nothing is read until a UI attaches.
static read_stream: GlobalCell<RStream> = GlobalCell::new(RStream {
    s: Stream {
        closed: true,
        ..ZEROED_RSTREAM.s
    },
    ..ZEROED_RSTREAM
});

/// Bytes read from the OS and not yet handed to the typeahead.
///
/// `input_buffer[read_pos..write_pos]` is the unread run; see the module docs
/// for why those are offsets and not pointers.
static input_buffer: GlobalCell<[u8; INPUT_BUFFER_SIZE]> = GlobalCell::new([0; INPUT_BUFFER_SIZE]);
static input_read_pos: GlobalCell<usize> = GlobalCell::new(0);
static input_write_pos: GlobalCell<usize> = GlobalCell::new(0);

static input_eof: GlobalCell<bool> = GlobalCell::new(false);
static blocking: GlobalCell<bool> = GlobalCell::new(false);
/// Time already spent waiting for a CursorHold, and the `tb_change_cnt` that
/// wait started under — a change to the typeahead restarts the clock.
static cursorhold_time: GlobalCell<c_int> = GlobalCell::new(0);
static cursorhold_tb_change_cnt: GlobalCell<c_int> = GlobalCell::new(0);

/// Start reading stdin.
pub fn input_start() {
    // SAFETY: `read_stream` is this module's own static and libuv only ever
    // touches it from the main thread; `input_read_cb` has the signature
    // `rstream_start` demands and takes no data pointer.
    unsafe {
        if !(*read_stream.ptr()).s.closed {
            return;
        }
        used_stdin.set(true);
        rstream_init_fd(main_loop.ptr(), read_stream.ptr(), STDIN_FILENO);
        rstream_start(read_stream.ptr(), Some(input_read_cb), ptr::null_mut());
    }
}

/// Stop reading stdin.
pub fn input_stop() {
    // SAFETY: as `input_start`.
    unsafe {
        if (*read_stream.ptr()).s.closed {
            return;
        }
        rstream_stop(read_stream.ptr());
        rstream_may_close(read_stream.ptr());
    }
}

/// The queued CursorHold, fired once the event loop gets to it.
///
/// # Safety
/// An `argv_callback`; reads no argument.
unsafe extern "C" fn cursorhold_event(_argv: *mut *mut c_void) {
    let event = if State.get() & MODE_INSERT != 0 {
        AutoEvent::CursorHoldI
    } else {
        AutoEvent::CursorHold
    } as AutoEvent;
    let __hoisted_0 = Buf::current_or_none();
    fire_autocmds(event, false, __hoisted_0);
    did_cursorhold.set(true);
}

fn create_cursorhold_event(events_enabled: bool) {
    // SAFETY: `main_loop.events` is the process's event queue.
    unsafe {
        // If events are enabled and the queue has any items, this should not
        // have been reached — `inbuf_poll` would have answered `Ready`.
        debug_assert!(!events_enabled || multiqueue_empty(main_events()));
        multiqueue_put_event(main_events(), Event::new(Some(cursorhold_event), []));
    }
}

fn reset_cursorhold_wait(tb_change_cnt: c_int) {
    cursorhold_time.set(0);
    cursorhold_tb_change_cnt.set(tb_change_cnt);
}

/// Move up to `maxlen` buffered bytes into `buf`, or 0 if there are none.
///
/// # Safety
/// `buf` must be writable for `maxlen` bytes.
unsafe fn try_read(buf: *mut uint8_t, maxlen: c_int, tb_change_cnt: c_int) -> Option<c_int> {
    if maxlen == 0 || input_available() == 0 {
        return None;
    }
    reset_cursorhold_wait(tb_change_cnt);
    debug_assert!(maxlen >= 0);
    let to_read = (maxlen as usize).min(input_available() as usize);
    let from = input_read_pos.get();
    input_buffer.with(|input| {
        // SAFETY: the caller's contract, and `to_read` is bounded by both
        // `maxlen` and the unread run, so the source range is in bounds.
        unsafe { ptr::copy_nonoverlapping(input[from..].as_ptr(), buf, to_read) };
    });
    input_read_pos.set(from + to_read);
    // Safe because INPUT_BUFFER_SIZE fits in an int.
    Some(to_read as c_int)
}

/// Which event queue [`input_get`] processes while it waits.
#[derive(Clone, Copy)]
pub(crate) enum InputEvents {
    /// None: only input wakes the wait.
    None,
    /// The main loop's.
    MainLoop,
    /// A child queue, for as long as its cell holds one.
    Queue(&'static GlobalCell<Option<ChildQueue>>),
}

impl InputEvents {
    /// The queue as the poll takes it: null, or a live queue.
    fn queue(self) -> *mut MultiQueue {
        match self {
            InputEvents::None => ptr::null_mut(),
            // SAFETY: `main_loop` is the process's event loop, always live.
            InputEvents::MainLoop => main_events(),
            InputEvents::Queue(cell) => queue_in(cell),
        }
    }
}

/// The main loop's event queue.
fn main_events() -> *mut MultiQueue {
    // SAFETY: `main_loop` is the process's event loop, always live.
    unsafe { (*main_loop.ptr()).events }
}

/// Whether the main loop's event queue holds anything.
pub(crate) fn main_loop_has_events() -> bool {
    // SAFETY: the main loop's queue, always live.
    unsafe { !multiqueue_empty(main_events()) }
}

/// Read OS input into `buf`, consuming pending events while waiting (when
/// `ms != 0`), and answer how many bytes came.
///
/// Consumes available OS input and pending events, manages CursorHold, and
/// handles EOF. Originally based on Vim's `mch_inchar`. An empty `buf` only
/// waits.
///
/// `ms` is a timeout in milliseconds: -1 waits indefinitely, 0 does not wait.
/// `tb_change_cnt` is how typeahead changes are detected, and `events` the
/// queue to process.
pub(crate) fn input_get(
    buf: &mut [u8],
    ms: c_int,
    tb_change_cnt: c_int,
    events: InputEvents,
) -> usize {
    let maxlen = c_int::try_from(buf.len()).unwrap_or(c_int::MAX);
    let (buf, events) = (buf.as_mut_ptr(), events.queue());
    // SAFETY: `buf` is the slice's own `maxlen` writable bytes, and `events`
    // null or a live queue.
    let got = unsafe { input_get_raw(buf, maxlen, ms, tb_change_cnt, events) };
    usize::try_from(got).unwrap_or(0)
}

/// [`input_get`] over the pointer and the length.
///
/// # Safety
/// `buf` must be writable for `maxlen` bytes, and `events` NULL or a live
/// queue.
unsafe fn input_get_raw(
    buf: *mut uint8_t,
    maxlen: c_int,
    ms: c_int,
    tb_change_cnt: c_int,
    events: *mut MultiQueue,
) -> c_int {
    // Needed so that feeding typeahead over RPC can prevent CursorHold.
    if tb_change_cnt != cursorhold_tb_change_cnt.get() {
        reset_cursorhold_wait(tb_change_cnt);
    }

    // SAFETY: the caller's contract on `buf` and `events`; `curbuf`,
    // `read_stream` and `main_loop` are always-live globals, and the
    // getchar/autocmd entry points below take no pointer of ours.
    unsafe {
        if let Some(n) = try_read(buf, maxlen, tb_change_cnt) {
            return n;
        }

        // No risk of a UI flood, so disable CTRL-C "interrupt" behaviour if
        // it is mapped.
        if (mapped_ctrl_c.get() | Buf::current().b_mapped_ctrl_c) & get_real_state() != 0 {
            ctrl_c_interrupts.set(false);
        }

        let mut result;
        if ms >= 0 {
            result = inbuf_poll(ms, events);
            if result == InputAvail::Empty {
                return 0;
            }
        } else {
            let wait_start = os_hrtime();
            cursorhold_time.set(cursorhold_time.get().min(p_ut() as c_int));
            result = inbuf_poll(p_ut() as c_int - cursorhold_time.get(), events);
            if result == InputAvail::Empty {
                if (*read_stream.ptr()).s.closed && silent_mode.get() {
                    // Drained event loop and initial input; exit `-es`/`-Es`.
                    read_error_exit();
                }
                reset_cursorhold_wait(tb_change_cnt);
                if trigger_cursorhold() && !typebuf_changed(tb_change_cnt) {
                    create_cursorhold_event(events == main_events());
                } else {
                    before_blocking();
                    result = inbuf_poll(-1, events);
                }
            } else {
                let waited = os_hrtime().wrapping_sub(wait_start) / 1_000_000;
                cursorhold_time.set(cursorhold_time.get().wrapping_add(waited as c_int));
            }
        }

        ctrl_c_interrupts.set(true);

        // If input went straight into the typeahead buffer, bail out here.
        if typebuf_changed(tb_change_cnt) {
            return 0;
        }

        if let Some(n) = try_read(buf, maxlen, tb_change_cnt) {
            return n;
        }

        // With events pending, hand back the keys directly.
        if maxlen != 0 && pending_events(events) {
            return push_event_key(buf, maxlen);
        }

        if result == InputAvail::Eof && ms != 0 {
            read_error_exit();
        }
    }
    0
}

/// Whether a character is available for reading.
pub fn os_char_avail() -> bool {
    inbuf_poll(0, ptr::null_mut()) == InputAvail::Ready
}

/// Poll for fast events; sets `got_int` if CTRL-C was typed.
///
/// This runs a full libuv loop iteration, which is expensive — prefer
/// [`line_breakcheck`] in a busy inner loop. The caller must at least check
/// `got_int` before calling again, and often wants `input_available()` too,
/// to throttle idle processing while there is user input waiting.
pub(crate) fn os_breakcheck() {
    if got_int.get() {
        return;
    }
    let uv_loop = main_loop.ptr();
    #[cfg(test)]
    if !test_loop_up(uv_loop) {
        return;
    }
    // SAFETY: `main_loop` is the process's event loop, always live.
    unsafe { loop_poll_events(uv_loop, 0) };
}

/// Bring the event loop up the first time a lib test polls it: nothing ran
/// startup there, and a directory walk breaks out to check for CTRL-C.
/// Under Miri there is no libuv to bring up, and nothing to poll.
#[cfg(test)]
fn test_loop_up(uv_loop: *mut crate::types::Loop) -> bool {
    if cfg!(miri) {
        return false;
    }
    static UP: std::sync::Once = std::sync::Once::new();
    // SAFETY: the process's loop storage, which does not move, initialised
    // once and never closed.
    UP.call_once(|| unsafe { crate::event::r#loop::loop_init(uv_loop) });
    true
}

const BREAKCHECK_SKIP: c_int = 1000;
static breakcheck_count: GlobalCell<c_int> = GlobalCell::new(0);

/// Start the [`fast_breakcheck`] count over, so a lib test that adds matches
/// never reaches the event loop it has not got.
#[cfg(test)]
pub(crate) fn reset_breakcheck_count() {
    breakcheck_count.set(0);
}

/// [`os_breakcheck`], but only once every `every` calls.
fn breakcheck_every(every: c_int) {
    let count = breakcheck_count.get() + 1;
    if count >= every {
        breakcheck_count.set(0);
        os_breakcheck();
    } else {
        breakcheck_count.set(count);
    }
}

/// Check for CTRL-C, but only once in a while.
///
/// Use this rather than [`os_breakcheck`] in anything that runs per line of a
/// file: `os_breakcheck` makes system calls and is far too slow to do that
/// often.
pub fn line_breakcheck() {
    breakcheck_every(BREAKCHECK_SKIP);
}

/// [`line_breakcheck`], checking ten times less often.
pub fn fast_breakcheck() {
    breakcheck_every(BREAKCHECK_SKIP * 10);
}

/// [`line_breakcheck`], checking a hundred times less often.
pub fn veryfast_breakcheck() {
    breakcheck_every(BREAKCHECK_SKIP * 100);
}

/// Whether file descriptor `fd` refers to a terminal.
pub fn os_isatty(fd: c_int) -> bool {
    // SAFETY: libuv classifies a descriptor without dereferencing anything.
    unsafe { uv_guess_handle(fd) == UV_TTY }
}

/// Wait until everything written to the terminal on `fd` has gone out --
/// `tcdrain()`. Nothing to do for a descriptor that is not a terminal.
pub(crate) fn tty_drain(fd: c_int) {
    // SAFETY: `tcdrain` takes a descriptor and dereferences nothing.
    unsafe { ::libc::tcdrain(fd) };
}

/// How many bytes are buffered and unread.
pub fn input_available() -> size_t {
    (input_write_pos.get() - input_read_pos.get()) as size_t
}

/// How much room is left at the end of the buffer.
fn input_space() -> usize {
    INPUT_BUFFER_SIZE - input_write_pos.get()
}

/// Append `data` to the input buffer, dropping whatever does not fit.
pub fn input_enqueue_raw(data: &[u8]) {
    input_buffer.with_mut(|input| {
        let (read, write) = (input_read_pos.get(), input_write_pos.get());
        // Reclaim the room already consumed by moving the unread run to the
        // front. The buffer is a window, not a ring.
        let write = if read > 0 {
            input.copy_within(read..write, 0);
            input_read_pos.set(0);
            write - read
        } else {
            write
        };
        let to_write = data.len().min(INPUT_BUFFER_SIZE - write);
        input[write..write + to_write].copy_from_slice(&data[..to_write]);
        input_write_pos.set(write + to_write);
    });
}

/// A `<x>` form takes at least one character and produces at most nineteen
/// (one plus five times three for the character, three for a modifier).
const MAX_TRANS_SPECIAL: usize = 19;

/// Feed `keys` — key *notation*, not raw bytes — from channel `chan_id`.
///
/// Returns how many bytes of `keys` were consumed; an incomplete trailing
/// `<...>` is left for the next call.
pub fn input_enqueue(chan_id: uint64_t, keys: String_0) -> size_t {
    current_ui.set(chan_id);

    // SAFETY: a `String` owns `len()` bytes and the NUL after them. Every
    // pointer below is derived from
    // `keys.data` and the loop never lets `ptr` past `end`, except for the
    // one-past-the-end read of the `'<'` skip — which lands on the NUL that
    // terminates every `String` nvim builds, exactly as upstream's does.
    // `buf` is sized for the longest sequence `trans_special` can produce,
    // and no `did_simplify` answer is wanted: simplification happens later,
    // in the typeahead.
    unsafe {
        let mut ptr = keys.data() as *const c_char;
        let end = ptr.add(keys.len());

        while input_space() >= MAX_TRANS_SPECIAL && ptr < end {
            let mut buf = [0u8; MAX_TRANS_SPECIAL];
            let text = cstr::slice_at(ptr, end.offset_from(ptr) as size_t);
            let (used, new_size) = trans_key(text, &mut buf);
            ptr = ptr.add(used);

            if new_size > 0 {
                let new_size = handle_mouse_event(&mut ptr, end, &mut buf, new_size);
                if new_size > 0 {
                    input_enqueue_raw(&buf[..new_size as usize]);
                }
                continue;
            }

            let byte = *ptr as u8;
            if byte == b'<' {
                // An invalid or incomplete key sequence: skip to the next '>'.
                let old_ptr = ptr;
                loop {
                    ptr = ptr.add(1);
                    if ptr >= end || *ptr == b'>' as c_char {
                        break;
                    }
                }
                if *ptr != b'>' as c_char {
                    // Incomplete: hand it back unconsumed.
                    ptr = old_ptr;
                    break;
                }
                ptr = ptr.add(1);
                continue;
            }

            // Copy the character across, escaping K_SPECIAL.
            if byte as c_int == K_SPECIAL {
                input_enqueue_raw(&[K_SPECIAL as u8, KS_SPECIAL as u8, KE_FILLER as u8]);
            } else {
                input_enqueue_raw(&[byte]);
            }
            ptr = ptr.add(1);
        }

        let consumed = ptr.offset_from(keys.data() as *const c_char) as size_t;
        process_ctrl_c();
        consumed
    }
}

/// How many clicks in a row, and where the last one was.
static orig_num_clicks: GlobalCell<c_int> = GlobalCell::new(0);
static orig_mouse_code: GlobalCell<c_int> = GlobalCell::new(0);
static orig_mouse_grid: GlobalCell<c_int> = GlobalCell::new(0);
static orig_mouse_col: GlobalCell<c_int> = GlobalCell::new(0);
static orig_mouse_row: GlobalCell<c_int> = GlobalCell::new(0);
/// When the previous click was, in nanoseconds.
static orig_mouse_time: GlobalCell<uint64_t> = GlobalCell::new(0);

/// The `MOD_MASK_*CLICK` modifier this mouse event carries, or `None` when the
/// event should be dropped entirely — which happens for a mouse *move* that
/// did not move.
fn check_multiclick(code: c_int, grid: c_int, row: c_int, col: c_int) -> Option<uint8_t> {
    if (KE_MOUSEDOWN as c_int..=KE_MOUSERIGHT as c_int).contains(&code) {
        return Some(0);
    }

    let no_move =
        orig_mouse_grid.get() == grid && orig_mouse_col.get() == col && orig_mouse_row.get() == row;

    if code == KE_MOUSEMOVE as c_int {
        if no_move {
            return None;
        }
    } else if [
        KE_LEFTMOUSE,
        KE_RIGHTMOUSE,
        KE_MIDDLEMOUSE,
        KE_X1MOUSE,
        KE_X2MOUSE,
    ]
    .contains(&(code as KeyExtra))
    {
        // For a click event the run length is updated; a drag or a release
        // keeps whatever the click before it established.
        let mouse_time = os_hrtime();
        let timediff = mouse_time.wrapping_sub(orig_mouse_time.get());
        // 'mousetime' is in milliseconds, `os_hrtime` in nanoseconds.
        let mouset = (p_mouset() as uint64_t).wrapping_mul(1_000_000);
        let same_click = code == orig_mouse_code.get()
            && no_move
            && timediff < mouset
            && orig_num_clicks.get() != 4;
        orig_num_clicks.set(if same_click {
            orig_num_clicks.get() + 1
        } else {
            1
        });
        orig_mouse_code.set(code);
        orig_mouse_time.set(mouse_time);
    }

    orig_mouse_grid.set(grid);
    orig_mouse_col.set(col);
    orig_mouse_row.set(row);

    if code == KE_MOUSEMOVE as c_int {
        return Some(0);
    }
    Some(match orig_num_clicks.get() {
        2 => ModMask::TWO_CLICK.bits() as uint8_t,
        3 => ModMask::THREE_CLICK.bits() as uint8_t,
        4 => ModMask::FOUR_CLICK.bits() as uint8_t,
        _ => 0,
    })
}

/// The `<`*col*`,`*row*`>` suffix a mouse key sequence may carry, and how many
/// bytes it took — upstream's `sscanf(*ptr, "<%d,%d>%n", …)`, with `%d`'s
/// leading whitespace and optional sign.
///
/// `None` means the format did not match, which upstream distinguishes only
/// by `%n` never being reached.
fn scan_mouse_pos(s: &[u8]) -> Option<(c_int, c_int, usize)> {
    /// `%d`: whitespace, an optional sign, then at least one digit. Saturates
    /// rather than wrapping, which is the one place this is *not* `sscanf`:
    /// C leaves an out-of-range conversion undefined.
    fn scan_int(s: &[u8]) -> Option<(c_int, usize)> {
        let mut at = 0;
        while s.get(at).is_some_and(u8::is_ascii_whitespace) {
            at += 1;
        }
        let negative = match s.get(at) {
            Some(b'-') => {
                at += 1;
                true
            }
            Some(b'+') => {
                at += 1;
                false
            }
            _ => false,
        };
        let digits = at;
        let mut value: i64 = 0;
        while let Some(d) = s.get(at).filter(|b| b.is_ascii_digit()) {
            value = value.saturating_mul(10).saturating_add((d - b'0') as i64);
            at += 1;
        }
        if at == digits {
            return None;
        }
        let value = if negative { -value } else { value };
        Some((
            value.clamp(c_int::MIN as i64, c_int::MAX as i64) as c_int,
            at,
        ))
    }

    let after_lt = s.strip_prefix(b"<")?;
    let (col, col_len) = scan_int(after_lt)?;
    let after_comma = after_lt[col_len..].strip_prefix(b",")?;
    let (row, row_len) = scan_int(after_comma)?;
    // What `%n` records: how far into `s` — not into whatever the last step
    // stripped — the whole format reached.
    let rest = after_comma[row_len..].strip_prefix(b">")?;
    Some((col, row, s.len() - rest.len()))
}

/// Extract a mouse event's row and column, and detect multiple clicks.
///
/// Answers the new length of `buf`, which is zero when the event is to be
/// dropped.
///
/// # Safety
/// `*ptr` must be inside a run ending at `end`.
unsafe fn handle_mouse_event(
    cursor: &mut *const c_char,
    end: *const c_char,
    buf: &mut [u8; MAX_TRANS_SPECIAL],
    bufsize: c_uint,
) -> c_uint {
    // A modifier prefix, if there is one, pushes the event three bytes along.
    let (mouse_code, kind) = match bufsize {
        3 => (buf[2] as c_int, buf[1] as c_int),
        6 => (buf[5] as c_int, buf[4] as c_int),
        _ => (0, 0),
    };

    let is_mouse = (KE_LEFTMOUSE as c_int..=KE_RIGHTRELEASE as c_int).contains(&mouse_code)
        || (KE_X1MOUSE as c_int..=KE_X2RELEASE as c_int).contains(&mouse_code)
        || (KE_MOUSEDOWN as c_int..=KE_MOUSERIGHT as c_int).contains(&mouse_code)
        || mouse_code == KE_MOUSEMOVE as c_int;
    if kind != KS_EXTRA || !is_mouse {
        return bufsize;
    }

    // A `<col,row>` sequence can follow, and sets the mouse_row/mouse_col
    // globals. That is ugly, but it is how the rest of the code expects to
    // find mouse coordinates.
    // SAFETY: the caller's contract puts `*ptr` inside the run ending at
    // `end`.
    let rest = unsafe {
        core::slice::from_raw_parts(cursor.cast::<u8>(), end.offset_from(*cursor) as usize)
    };
    if let Some((col, row, advance)) = scan_mouse_pos(rest) {
        if col >= 0 && row >= 0 {
            // Some terminals report positions off the screen.
            mouse_grid.set(0);
            mouse_row.set(row.min(Rows.get() - 1));
            mouse_col.set(col.min(Columns.get() - 1));
        }
        // SAFETY: `advance` counts bytes of `rest`, so it stays within `end`.
        *cursor = unsafe { cursor.add(advance) };
    }

    let Some(modifiers) = check_multiclick(
        mouse_code,
        mouse_grid.get(),
        mouse_row.get(),
        mouse_col.get(),
    ) else {
        return 0;
    };

    if modifiers == 0 {
        return bufsize;
    }
    if buf[1] as c_int == KS_MODIFIER {
        buf[2] |= modifiers;
        return bufsize;
    }
    // No modifiers in the buffer yet: shift the event three bytes along and
    // write the modifier sequence in front of it.
    buf.copy_within(0..3, 3);
    buf[0] = K_SPECIAL as u8;
    buf[1] = KS_MODIFIER as u8;
    buf[2] = modifiers;
    bufsize + 3
}

/// Feed a mouse event that arrived already decoded — over RPC, rather than as
/// key notation.
pub fn input_enqueue_mouse(code: c_int, modifier: uint8_t, grid: c_int, row: c_int, col: c_int) {
    let Some(clicks) = check_multiclick(code, grid, row, col) else {
        return;
    };
    let modifier = modifier | clicks;

    let mut buf = [0u8; 6];
    let at = if modifier != 0 {
        buf[..3].copy_from_slice(&[K_SPECIAL as u8, KS_MODIFIER as u8, modifier]);
        3
    } else {
        0
    };
    buf[at..at + 3].copy_from_slice(&[K_SPECIAL as u8, KS_EXTRA as u8, code as u8]);

    mouse_grid.set(grid);
    mouse_row.set(row);
    mouse_col.set(col);

    input_enqueue_raw(&buf[..at + 3]);
}

/// What [`inbuf_poll`] found.
///
/// The third value is EOF, not "unknown", so this is an enumeration and not
/// a tri-state boolean.
#[derive(Copy, Clone, PartialEq, Eq)]
enum InputAvail {
    /// Input or events are available.
    Ready,
    /// Neither, within the timeout.
    Empty,
    /// The input stream has reached end of file.
    Eof,
}

/// Whether the main loop is blocked waiting for input.
pub fn input_blocking() -> bool {
    blocking.get()
}

/// Check for (but do not read) available input, consuming `main_loop.events`
/// while waiting.
///
/// `ms` is a timeout in milliseconds; -1 waits indefinitely and 0 does not
/// wait. `events` is an optional queue to check for pending events.
fn inbuf_poll(ms: c_int, events: *mut MultiQueue) -> InputAvail {
    // SAFETY: `events` is NULL or a live queue, and `main_loop` is the
    // process's event loop.
    unsafe {
        if os_input_ready(events) {
            return InputAvail::Ready;
        }

        if do_profiling.get() == PROF_YES && ms != 0 {
            prof_input_start();
        }

        if (ms == -1 || ms > 0) && events != main_events() && !input_eof.get() {
            // The pending input provoked a blocking wait. Do special events
            // now. #6247
            blocking.set(true);
            multiqueue_process_events(queue_in(&ch_before_blocking_events));
        }
        logmsg!(
            LOGLVL_DBG,
            c"inbuf_poll",
            514,
            "blocking... events={}",
            c_str(if events.is_null() {
                c"false".as_ptr()
            } else {
                c"true".as_ptr()
            })
        );
        // Upstream polls with a NULL queue here, so the macro's "drain this
        // queue instead" branch is dead: `events` is only read by
        // `os_input_ready`.
        process_events_until(main_loop.ptr(), ptr::null_mut(), ms as i64, || {
            os_input_ready(events) || input_eof.get()
        });
        blocking.set(false);

        if do_profiling.get() == PROF_YES && ms != 0 {
            prof_input_end();
        }

        if os_input_ready(events) {
            InputAvail::Ready
        } else if input_eof.get() {
            InputAvail::Eof
        } else {
            InputAvail::Empty
        }
    }
}

/// libuv's read callback: everything that arrives on stdin lands here.
///
/// # Safety
/// An `stream_read_cb`; `buf` must be readable for `count` bytes.
unsafe fn input_read_cb(
    _stream: *mut RStream,
    buf: *const c_char,
    count: size_t,
    _data: *mut c_void,
    at_eof: bool,
) -> size_t {
    if at_eof {
        input_eof.set(true);
    }
    // The stream's own buffer is smaller than ours, so a read always fits.
    debug_assert!(input_space() >= count);
    // SAFETY: the caller's contract -- `buf` is `count` readable bytes.
    input_enqueue_raw(unsafe { core::slice::from_raw_parts(buf.cast::<u8>(), count) });
    count
}

/// Reverse-search the buffered input for a CTRL-C, and discard everything
/// typed before it.
fn process_ctrl_c() {
    if !ctrl_c_interrupts.get() {
        return;
    }
    let read = input_read_pos.get();
    let write = input_write_pos.get();
    let Some(at) = input_buffer.with_mut(|input| {
        let unread = &mut input[read..write];
        let at = (0..unread.len()).rev().find(|&i| {
            unread[i] == Ctrl_C as u8
                || (unread[i] == b'C'
                    && i >= 3
                    && unread[i - 3] == K_SPECIAL as u8
                    && unread[i - 2] == KS_MODIFIER as u8
                    && unread[i - 1] == ModMask::CTRL.bits() as u8)
        })?;
        unread[at] = Ctrl_C as u8;
        Some(at)
    }) else {
        return;
    };
    got_int.set(true);
    if at > 0 {
        // Drop the unprocessed typeahead in front of the CTRL-C.
        input_read_pos.set(read + at);
    }
}

/// Push bytes of the `KE_EVENT` key sequence, a partial one at a time when
/// `maxlen < 3`.
///
/// # Safety
/// `buf` must be writable for `maxlen` bytes, which must be at least one.
unsafe fn push_event_key(buf: *mut uint8_t, maxlen: c_int) -> c_int {
    const KEY: [uint8_t; 3] = [
        K_SPECIAL as uint8_t,
        KS_EXTRA as uint8_t,
        KE_EVENT as uint8_t,
    ];
    static key_idx: GlobalCell<usize> = GlobalCell::new(0);

    let mut buf_idx = 0;
    loop {
        // SAFETY: the caller's contract, and `buf_idx < maxlen` below.
        unsafe { *buf.add(buf_idx) = KEY[key_idx.get()] };
        key_idx.set((key_idx.get() + 1) % KEY.len());
        buf_idx += 1;
        if key_idx.get() == 0 || buf_idx >= maxlen as usize {
            break;
        }
    }
    buf_idx as c_int
}

/// Whether there is input waiting, in the typeahead, the buffer or `events`.
///
/// # Safety
/// `events` must be NULL or a live queue.
pub unsafe fn os_input_ready(events: *mut MultiQueue) -> bool {
    typebuf_was_filled.get()          // an API call filled the typeahead
        || input_available() != 0     // the input buffer holds something
        || unsafe { pending_events(events) } // events must be processed
}

/// Exit because of an input read error.
fn read_error_exit() -> ! {
    if silent_mode.get() {
        // The normal way out for `nvim -es`.
        getout(0);
    }
    preserve_exit(Some(gettext(c"Nvim: Error reading input, exiting...\n")))
}

/// # Safety
/// `events` must be NULL or a live queue.
unsafe fn pending_events(events: *mut MultiQueue) -> bool {
    // SAFETY: the caller's contract.
    !events.is_null() && unsafe { !multiqueue_empty(events) }
}

/// The `<>` key name `text` starts with, encoded into `buf`: how many bytes
/// of `text` it took, and how many of `buf` its encoding fills -- both zero
/// when `text` does not start with one.
fn trans_key(text: &[u8], buf: &mut [u8; MAX_TRANS_SPECIAL]) -> (usize, c_uint) {
    match special_key_at(text, FSK_KEYCODE, None) {
        Some((key, modifiers, used)) => {
            let encoded = special_key_bytes(key, modifiers, true);
            let encoded = encoded.as_bytes();
            buf[..encoded.len()].copy_from_slice(encoded);
            (used, c_uint::try_from(encoded.len()).unwrap_or(0))
        }
        None => (0, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mouse_position_suffix_is_col_then_row() {
        assert_eq!(scan_mouse_pos(b"<10,20>rest"), Some((10, 20, 7)));
        assert_eq!(scan_mouse_pos(b"<0,0>"), Some((0, 0, 5)));
        assert_eq!(scan_mouse_pos(b"<-1,-2>"), Some((-1, -2, 7)));
    }

    #[test]
    fn a_mouse_position_suffix_takes_percent_d_verbatim() {
        // `%d` skips leading whitespace and accepts a sign, so `sscanf` does
        // too, however unlikely a terminal is to send it.
        assert_eq!(scan_mouse_pos(b"< 10, +20>"), Some((10, 20, 10)));
        // Out of range saturates rather than wrapping.
        assert_eq!(
            scan_mouse_pos(b"<99999999999,0>"),
            Some((c_int::MAX, 0, 15))
        );
    }

    #[test]
    fn anything_else_is_not_a_mouse_position() {
        assert_eq!(scan_mouse_pos(b""), None);
        assert_eq!(scan_mouse_pos(b"<10,20"), None);
        assert_eq!(scan_mouse_pos(b"<10;20>"), None);
        assert_eq!(scan_mouse_pos(b"<,20>"), None);
        assert_eq!(scan_mouse_pos(b"10,20>"), None);
    }
}
