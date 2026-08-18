//! The state machine's C boundary: its lifetime, the parser callbacks it
//! installs, and the entry points the host calls.
//!
//! Everything here works in raw pointers, because everything here is either
//! an `extern "C"` function the host or the parser calls, or the allocation
//! that has to run before a [`VTermState`] exists at all. Once there is a
//! state machine to speak of, the work moves to the safe methods the parent
//! module defines on it, so each function below spends one region on reaching
//! the state and then stops doing pointer work.
//!
//! Two obligations recur. The `user` pointer the parser hands back is the
//! state machine it was given in [`vterm_obtain_state`], which outlives the
//! parser it is installed in. And an unrecognised sequence goes out to a
//! *consumer's* fallback, which may re-enter the terminal, so every one of
//! them is reached with the table read out and nothing borrowed.
//!
//! Ported from libvterm, Copyright (c) 2008 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libvterm-LICENSE.txt.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_long, c_uint, c_void};
use core::slice;

use crate::global_cell::GlobalCell;
use crate::types::{
    VTerm, VTermKeyEncodingFlags, VTermLineInfo, VTermParserCallbacks, VTermProp, VTermRect,
    VTermSelectionCallbacks, VTermState, VTermStateCallbacks, VTermStateFallbacks,
    VTermStringFragment, VTermValue, size_t, uint8_t,
};
use crate::vterm::encoding::{ENC_UTF8, vterm_lookup_encoding};
use crate::vterm::parser::vterm_parser_set_callbacks;
use crate::vterm::pen::init_pen;
use crate::vterm::screen::{BUFIDX_ALTSCREEN, BUFIDX_PRIMARY};
use crate::vterm::vterm::{
    MOUSE_X10, VTERM_PROP_ALTSCREEN, VTERM_PROP_CURSORBLINK, VTERM_PROP_CURSORSHAPE,
    VTERM_PROP_CURSORVISIBLE, VTERM_PROP_FOCUSREPORT, VTERM_PROP_ICONNAME, VTERM_PROP_MOUSE,
    VTERM_PROP_MOUSE_DRAG, VTERM_PROP_MOUSE_MOVE, VTERM_PROP_REVERSE, VTERM_PROP_SYNCOUTPUT,
    VTERM_PROP_THEMEUPDATES, VTERM_PROP_TITLE, vterm_alloc, vterm_dealloc,
};
use crate::vterm::{csi, dcs, mode, selection, text};

use super::{MOUSE_WANT_CLICK, MOUSE_WANT_DRAG, MOUSE_WANT_MOVE, tabstop_bytes};

/// The state machine a parser callback's `user` pointer stands for.
///
/// # Safety
///
/// `user` must be the pointer [`vterm_obtain_state`] installed, and no other
/// reference to that state may be live.
unsafe fn state_of<'a>(user: *mut c_void) -> &'a mut VTermState {
    // SAFETY: the caller's promise.
    unsafe { &mut *user.cast::<VTermState>() }
}

/// A leader or intermediate arrives as a NUL-terminated string, but only a
/// single-byte one means anything here, so two bytes decide it.
///
/// # Safety
///
/// `bytes` must be null or point at a NUL-terminated string.
unsafe fn two_bytes(bytes: *const c_char) -> [u8; 2] {
    // SAFETY: a non-null, non-empty string has a first byte, and a second one
    // that is at worst the terminator.
    if bytes.is_null() || unsafe { *bytes } == 0 {
        [0, 0]
    } else {
        unsafe { [*bytes as u8, *bytes.offset(1) as u8] }
    }
}

// ------------------------------------------------------------------ lifetime

/// Allocates a state machine for `vt` and gives it its power-on settings.
///
/// # Safety
///
/// `vt` must be a live terminal that has no state machine yet.
unsafe fn vterm_state_new(vt: *mut VTerm) -> *mut VTermState {
    // SAFETY: the caller's promise.
    let (rows, cols) = unsafe { ((*vt).rows, (*vt).cols) };
    // SAFETY: `vterm_alloc` answers zeroed storage of the size asked for, and
    // a zeroed `VTermState` is a valid one -- every field is a scalar, a
    // pointer or an array of those.
    let allocation = unsafe { vterm_alloc(size_of::<VTermState>()) }.cast::<VTermState>();
    // SAFETY: as above, and nothing else refers to the fresh allocation.
    let state = unsafe { &mut *allocation };

    state.vt = vt;
    state.rows = rows;
    state.cols = cols;
    state.mouse_col = 0;
    state.mouse_row = 0;
    state.mouse_buttons = 0;
    state.mouse_protocol = MOUSE_X10;
    state.callbacks = core::ptr::null();
    state.cbdata = core::ptr::null_mut();
    state.selection.callbacks = core::ptr::null();
    state.selection.user = core::ptr::null_mut();
    state.selection.buffer = core::ptr::null_mut();

    init_pen(state);
    state.bold_is_highbright = 0;
    state.combine_pos.row = -1;

    // The tab stops and the two line-mark arrays live outside the struct and
    // belong to it from here until `vterm_state_free`.
    // SAFETY: a fresh allocation of the size the state will index by.
    state.tabstops = unsafe { vterm_alloc(tabstop_bytes(cols)) }.cast::<uint8_t>();
    let marks = (rows.max(0) as size_t) * size_of::<VTermLineInfo>();
    // TODO(vterm): the altscreen's marks could wait until it is switched on.
    // SAFETY: as above, one array of `rows` marks per buffer.
    let (primary, altscreen) = unsafe { (vterm_alloc(marks), vterm_alloc(marks)) };
    state.lineinfos[BUFIDX_PRIMARY] = primary.cast::<VTermLineInfo>();
    state.lineinfos[BUFIDX_ALTSCREEN] = altscreen.cast::<VTermLineInfo>();
    state.lineinfo = state.lineinfos[BUFIDX_PRIMARY];

    let utf8 = vterm_lookup_encoding(ENC_UTF8, b'u' as c_char);
    state.encoding_utf8.enc = utf8;
    let scratch = (&raw mut state.encoding_utf8.data).cast::<c_void>();
    // SAFETY: `vterm_lookup_encoding` answers one of the encoding module's
    // static tables, and `scratch` is the instance's own inline data area.
    if let Some(init) = unsafe { (*utf8).init } {
        unsafe { init(utf8, scratch) };
    }

    for stack in &mut state.key_encoding_stacks {
        stack.items = [VTermKeyEncodingFlags {
            disambiguate_report_events_report_alternate_report_all_keys_report_associated: [0; 1],
        }; 16];
        stack.size = 1;
    }

    allocation
}

/// Releases the state machine and the three allocations it owns.
///
/// # Safety
///
/// `state` must have come from [`vterm_state_new`] and must not be used
/// again.
pub unsafe fn vterm_state_free(state: *mut VTermState) {
    // SAFETY: the caller's promise.
    let (tabstops, lineinfos) = unsafe { ((*state).tabstops, (*state).lineinfos) };
    // SAFETY: each was allocated in `vterm_state_new` and is reachable from
    // nowhere else; the altscreen's may have been dropped by a resize.
    unsafe { vterm_dealloc(tabstops.cast::<c_void>()) };
    unsafe { vterm_dealloc(lineinfos[BUFIDX_PRIMARY].cast::<c_void>()) };
    if !lineinfos[BUFIDX_ALTSCREEN].is_null() {
        unsafe { vterm_dealloc(lineinfos[BUFIDX_ALTSCREEN].cast::<c_void>()) };
    }
    unsafe { vterm_dealloc(state.cast::<c_void>()) };
}

/// The terminal's state machine, created and wired to the parser on first ask.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_obtain_state(vt: *mut VTerm) -> *mut VTermState {
    // SAFETY: the caller promises `vt` is a live terminal. Nothing is
    // borrowed across `vterm_state_new`, which reads the terminal itself.
    let existing = unsafe { (*vt).state };
    if !existing.is_null() {
        return existing;
    }
    // SAFETY: the null just ruled out is the "has no state yet" the
    // constructor asks for.
    let state = unsafe { vterm_state_new(vt) };
    // SAFETY: the terminal, its parser and this state are freed together, so
    // the callbacks and their `user` pointer outlive the installation.
    unsafe { (*vt).state = state };
    unsafe { vterm_parser_set_callbacks(vt, PARSER_CALLBACKS.ptr(), state.cast::<c_void>()) };
    state
}

// ------------------------------------------------------- the parser's events

static PARSER_CALLBACKS: GlobalCell<VTermParserCallbacks> = GlobalCell::new(VTermParserCallbacks {
    text: Some(on_text),
    control: Some(on_control),
    escape: Some(on_escape),
    csi: Some(on_csi),
    osc: Some(on_osc),
    dcs: Some(on_dcs),
    apc: Some(on_apc),
    pm: Some(on_pm),
    sos: Some(on_sos),
    resize: Some(on_resize),
});

/// Decodes a run of input bytes through the live character set and prints the
/// graphemes it yields. Returns how many input bytes were consumed, which is
/// short of `len` when the run ends part-way through a sequence.
unsafe extern "C" fn on_text(bytes: *const c_char, len: size_t, user: *mut c_void) -> c_int {
    // SAFETY: the parser hands back what `vterm_obtain_state` installed.
    let state = unsafe { state_of(user) };

    // A high bit selects the right-hand set, a single shift overrides both,
    // and any slot designated UTF-8 collapses onto the one shared decoder so
    // that a multi-byte sequence can span calls.
    // SAFETY: `bytes` points at `len` readable bytes, so a non-empty run has
    // a first one.
    let lead = if len == 0 {
        0
    } else {
        (unsafe { *bytes }) as u8
    };
    let slot = if state.gsingle_set != 0 {
        state.gsingle_set as usize
    } else if lead & 0x80 == 0 {
        state.gl_set as usize
    } else if state.utf8() {
        usize::MAX
    } else {
        state.gr_set as usize
    };
    let shared = match state.encoding.get(slot) {
        Some(instance) => instance.enc == state.encoding_utf8.enc,
        None => true,
    };
    let encoding = if shared {
        &raw mut state.encoding_utf8
    } else {
        &raw mut state.encoding[slot]
    };

    // The decoder writes codepoints into the terminal's scratch buffer, which
    // is its own allocation, so the state stays free to be written meanwhile.
    // SAFETY: `state.vt` is the terminal this state was made for, and
    // `encoding` points into the state's own array of instances.
    let (scratch, scratch_len) = unsafe { ((*state.vt).tmpbuffer, (*state.vt).tmpbuffer_len) };
    let enc = unsafe { (*encoding).enc };
    let data = unsafe { &raw mut (*encoding).data }.cast::<c_void>();
    let points = scratch.cast::<u32>();
    let room = if state.gsingle_set != 0 {
        1
    } else {
        (scratch_len / size_of::<u32>()) as c_int
    };
    let mut got: c_int = 0;
    let mut eaten: size_t = 0;
    // The two out-scalars go in as pointers so that the call fits one line.
    let (n_out, eaten_out) = (&raw mut got, &raw mut eaten);
    // SAFETY: `enc` is one of the encoding module's static tables, all of
    // which have a decoder.
    let decode = unsafe { (*enc).decode }.expect("character set without a decoder");
    // SAFETY: `points`/`room` is the terminal's scratch, `bytes`/`len` the
    // caller's run, `data` the instance's own scratch, and the two counters
    // are locals.
    unsafe { decode(enc, data, points, n_out, room, bytes, eaten_out, len) };

    // A stateful set may not have seen enough bytes for even one codepoint.
    if got == 0 {
        return eaten as c_int;
    }
    state.gsingle_set = 0;
    // SAFETY: the decoder reports having written `got` codepoints.
    let printed = unsafe { slice::from_raw_parts(points, got as usize) };
    text::print(state, printed);
    eaten as c_int
}

unsafe extern "C" fn on_control(control: uint8_t, user: *mut c_void) -> c_int {
    // SAFETY: the parser hands back what `vterm_obtain_state` installed.
    let state = unsafe { state_of(user) };
    if text::control(state, control) {
        return 1;
    }
    let (fallback, fbdata) = (state.fallback(|f| f.control), state.fbdata);
    match fallback {
        // SAFETY: the consumer's own callback, reached with nothing borrowed.
        Some(f) => (unsafe { f(control, fbdata) } != 0) as c_int,
        None => 0,
    }
}

unsafe extern "C" fn on_escape(bytes: *const c_char, len: size_t, user: *mut c_void) -> c_int {
    // SAFETY: the parser hands back what `vterm_obtain_state` installed, and
    // `bytes`/`len` is the sequence it gathered.
    let state = unsafe { state_of(user) };
    let sequence = unsafe { slice::from_raw_parts(bytes.cast::<u8>(), len) };
    text::escape(state, sequence)
}

unsafe extern "C" fn on_csi(
    leader: *const c_char,
    args: *const c_long,
    argcount: c_int,
    intermed: *const c_char,
    command: c_char,
    user: *mut c_void,
) -> c_int {
    // SAFETY: the parser hands back what `vterm_obtain_state` installed, and
    // both strings are its own NUL-terminated buffers.
    let (leader_bytes, intermed_bytes) = unsafe { (two_bytes(leader), two_bytes(intermed)) };
    let state = unsafe { state_of(user) };
    // SAFETY: `args` points at `argcount` parameters the parser collected.
    let parameters = unsafe { slice::from_raw_parts(args, argcount.max(0) as usize) };
    let outcome = csi::dispatch(
        state,
        leader_bytes,
        parameters,
        intermed_bytes,
        command as u8,
    );
    match outcome {
        csi::Outcome::Handled => 1,
        csi::Outcome::Ignored => 0,
        csi::Outcome::Unrecognised => {
            let (fallback, fbdata) = (state.fallback(|f| f.csi), state.fbdata);
            match fallback {
                // SAFETY: the consumer's own callback, reached unborrowed;
                // the two strings are the parser's and outlive the report.
                Some(f) => {
                    (unsafe { f(leader, args, argcount, intermed, command, fbdata) } != 0) as c_int
                }
                None => 0,
            }
        }
    }
}

/// Upstream offers every OSC to the fallback, even the ones it handled
/// itself, and reports only what the fallback made of it.
unsafe extern "C" fn on_osc(command: c_int, frag: VTermStringFragment, user: *mut c_void) -> c_int {
    // SAFETY: the parser hands back what `vterm_obtain_state` installed.
    let state = unsafe { state_of(user) };
    selection::osc(state, command, frag);
    let (fallback, fbdata) = (state.fallback(|f| f.osc), state.fbdata);
    match fallback {
        // SAFETY: the consumer's own callback, reached with nothing borrowed.
        Some(f) => (unsafe { f(command, frag, fbdata) } != 0) as c_int,
        None => 0,
    }
}

unsafe extern "C" fn on_dcs(
    command: *const c_char,
    commandlen: size_t,
    frag: VTermStringFragment,
    user: *mut c_void,
) -> c_int {
    // SAFETY: the parser hands back what `vterm_obtain_state` installed, and
    // `command`/`commandlen` is the name it gathered.
    let state = unsafe { state_of(user) };
    let name = unsafe { slice::from_raw_parts(command.cast::<u8>(), commandlen) };
    if dcs::device_control(state, name, frag) {
        return 1;
    }
    let (fallback, fbdata) = (state.fallback(|f| f.dcs), state.fbdata);
    match fallback {
        // SAFETY: the consumer's own callback, reached unborrowed; the name
        // is the parser's and outlives the report.
        Some(f) => (unsafe { f(command, commandlen, frag, fbdata) } != 0) as c_int,
        None => 0,
    }
}

// APC, PM and SOS carry nothing this terminal understands, so each only
// reaches for its own fallback.
unsafe extern "C" fn on_apc(frag: VTermStringFragment, user: *mut c_void) -> c_int {
    // SAFETY: the parser hands back what `vterm_obtain_state` installed.
    let state = unsafe { state_of(user) };
    let (fallback, fbdata) = (state.fallback(|f| f.apc), state.fbdata);
    match fallback {
        // SAFETY: the consumer's own callback, reached with nothing borrowed.
        Some(f) => (unsafe { f(frag, fbdata) } != 0) as c_int,
        None => 0,
    }
}

unsafe extern "C" fn on_pm(frag: VTermStringFragment, user: *mut c_void) -> c_int {
    // SAFETY: the parser hands back what `vterm_obtain_state` installed.
    let state = unsafe { state_of(user) };
    let (fallback, fbdata) = (state.fallback(|f| f.pm), state.fbdata);
    match fallback {
        // SAFETY: the consumer's own callback, reached with nothing borrowed.
        Some(f) => (unsafe { f(frag, fbdata) } != 0) as c_int,
        None => 0,
    }
}

unsafe extern "C" fn on_sos(frag: VTermStringFragment, user: *mut c_void) -> c_int {
    // SAFETY: the parser hands back what `vterm_obtain_state` installed.
    let state = unsafe { state_of(user) };
    let (fallback, fbdata) = (state.fallback(|f| f.sos), state.fbdata);
    match fallback {
        // SAFETY: the consumer's own callback, reached with nothing borrowed.
        Some(f) => (unsafe { f(frag, fbdata) } != 0) as c_int,
        None => 0,
    }
}

unsafe extern "C" fn on_resize(rows: c_int, cols: c_int, user: *mut c_void) -> c_int {
    // SAFETY: the parser hands back what `vterm_obtain_state` installed.
    let state = unsafe { state_of(user) };
    let oldpos = state.pos;

    if cols != state.cols {
        state.resize_tabstops(cols);
    }
    state.rows = rows;
    state.cols = cols;
    if state.scrollregion_bottom > -1 {
        state.scrollregion_bottom = state.scrollregion_bottom.min(rows);
    }
    if state.scrollregion_right > -1 {
        state.scrollregion_right = state.scrollregion_right.min(cols);
    }

    // Upstream reallocated the line marks itself when no consumer took the
    // resize, but guarded that on a row count it had already overwritten, so
    // the branch never ran; the screen module is the only consumer there is,
    // and it always takes the resize.
    state.resize_consumer(rows, cols);
    state.lineinfo = state.lineinfos[state.active_buffer()];

    if state.at_phantom != 0 && state.pos.col < cols - 1 {
        state.at_phantom = 0;
        state.pos.col += 1;
    }
    state.pos.row = state.pos.row.clamp(0, (rows - 1).max(0));
    state.pos.col = state.pos.col.clamp(0, (cols - 1).max(0));
    state.update_cursor(oldpos, true);
    1
}

// ---------------------------------------------------------------- public API

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_state_reset(state: *mut VTermState, hard: c_int) {
    // SAFETY: the caller promises `state` is a live state machine.
    mode::reset(unsafe { &mut *state }, hard != 0);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_state_set_callbacks(
    state: *mut VTermState,
    callbacks: *const VTermStateCallbacks,
    user: *mut c_void,
) {
    // SAFETY: the caller promises `state` is a live state machine, and that
    // the table and its `user` pointer outlive their installation.
    let state = unsafe { &mut *state };
    let installed = !callbacks.is_null();
    state.callbacks = callbacks;
    state.cbdata = if installed {
        user
    } else {
        core::ptr::null_mut()
    };
    if installed {
        state.init_consumer_pen();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_state_set_unrecognised_fallbacks(
    state: *mut VTermState,
    fallbacks: *const VTermStateFallbacks,
    user: *mut c_void,
) {
    // SAFETY: as `vterm_state_set_callbacks` above.
    let state = unsafe { &mut *state };
    state.fallbacks = fallbacks;
    state.fbdata = if fallbacks.is_null() {
        core::ptr::null_mut()
    } else {
        user
    };
}

/// Applies a terminal property, offering it to the consumer first, which may
/// refuse it. Refusal matters most for the alternate screen: the state must
/// not believe it switched if the screen did not.
pub unsafe fn vterm_state_set_termprop(
    state: *mut VTermState,
    prop: VTermProp,
    val: *mut VTermValue,
) -> c_int {
    // SAFETY: the caller promises `state` is a live state machine and that
    // `val` holds the arm `prop` calls for. Each arm is read through one of
    // the two accessors, never both, so nothing reads past what was written.
    let state = unsafe { &mut *state };
    let boolean = || unsafe { (*val).boolean } as c_uint;
    let number = || unsafe { (*val).number };

    let (consumer, cbdata) = (state.consumer(|c| c.settermprop), state.cbdata);
    // SAFETY: the consumer's own callback, reached with nothing borrowed;
    // `val` is the caller's and outlives the offer.
    if let Some(f) = consumer
        && unsafe { f(prop, val, cbdata) } == 0
    {
        return 0;
    }

    match prop {
        // Titles are passed straight through, never stored.
        VTERM_PROP_TITLE | VTERM_PROP_ICONNAME => {}
        VTERM_PROP_CURSORVISIBLE => state.mode.set_cursor_visible(boolean()),
        VTERM_PROP_CURSORBLINK => state.mode.set_cursor_blink(boolean()),
        VTERM_PROP_CURSORSHAPE => state.mode.set_cursor_shape(number() as c_uint),
        VTERM_PROP_REVERSE => state.mode.set_screen(boolean()),
        VTERM_PROP_ALTSCREEN => {
            state.mode.set_alt_screen(boolean());
            state.lineinfo = state.lineinfos[state.active_buffer()];
            if state.mode.alt_screen() != 0 {
                let rect = VTermRect {
                    start_row: 0,
                    end_row: state.rows,
                    start_col: 0,
                    end_col: state.cols,
                };
                state.erase(rect, false);
            }
        }
        VTERM_PROP_MOUSE => {
            let level = number();
            state.mouse_flags = 0;
            if level != 0 {
                state.mouse_flags |= MOUSE_WANT_CLICK;
            }
            if level == VTERM_PROP_MOUSE_DRAG {
                state.mouse_flags |= MOUSE_WANT_DRAG;
            }
            if level == VTERM_PROP_MOUSE_MOVE {
                state.mouse_flags |= MOUSE_WANT_MOVE;
            }
        }
        VTERM_PROP_FOCUSREPORT => state.mode.set_report_focus(boolean()),
        VTERM_PROP_THEMEUPDATES => state.mode.set_theme_updates(boolean()),
        VTERM_PROP_SYNCOUTPUT => state.mode.set_synchronized_output(boolean()),
        _ => return 0,
    }
    1
}

/// Reports that the terminal window gained focus, if the host asked to hear.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_state_focus_in(state: *mut VTermState) {
    // SAFETY: the caller promises `state` is a live state machine.
    unsafe { &mut *state }.report_focus(b'I');
}

/// Reports that the terminal window lost focus, if the host asked to hear.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_state_focus_out(state: *mut VTermState) {
    // SAFETY: the caller promises `state` is a live state machine.
    unsafe { &mut *state }.report_focus(b'O');
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_state_get_lineinfo(
    state: *const VTermState,
    row: c_int,
) -> *const VTermLineInfo {
    // SAFETY: the caller promises `state` is a live state machine and that
    // `row` is one of its rows.
    unsafe { (*state).lineinfo.offset(row as isize) }
}

/// Installs the consumer's selection handling, allocating the staging buffer
/// the decoder chunks through when the consumer did not supply one.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_state_set_selection_callbacks(
    state: *mut VTermState,
    callbacks: *const VTermSelectionCallbacks,
    user: *mut c_void,
    buffer: *mut c_char,
    buflen: size_t,
) {
    // SAFETY: a fresh allocation of the length the decoder will chunk through.
    let buffer = if buflen != 0 && buffer.is_null() {
        unsafe { vterm_alloc(buflen) }.cast::<c_char>()
    } else {
        buffer
    };
    // SAFETY: the caller promises `state` is a live state machine, and that
    // the table, its `user` pointer and any buffer it supplied outlive their
    // installation.
    let state = unsafe { &mut *state };
    state.selection.callbacks = callbacks;
    state.selection.user = user;
    state.selection.buffer = buffer;
    state.selection.buflen = buflen;
}
