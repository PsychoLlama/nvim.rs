//! The state machine's C boundary: its lifetime, the parser callbacks it
//! installs, and the entry points the host calls.
//!
//! Everything here works in raw pointers, because everything here is either
//! an `extern "C"` function the host or the parser calls, or the allocation
//! that has to run before a [`VTermState`] exists at all. Once there is a
//! state machine to speak of, the work moves to the safe methods the parent
//! module defines on it.
//!
//! Ported from libvterm, Copyright (c) 2008 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libvterm-LICENSE.txt.

use core::ffi::{c_char, c_int, c_long, c_uint, c_void};
use core::slice;

use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::types::{
    VTerm, VTermKeyEncodingFlags, VTermLineInfo, VTermParserCallbacks, VTermProp, VTermRect,
    VTermSelectionCallbacks, VTermState, VTermStateCallbacks, VTermStateFallbacks,
    VTermStringFragment, VTermValue, size_t, uint8_t,
};
use crate::src::nvim::vterm::encoding::{ENC_UTF8, vterm_lookup_encoding};
use crate::src::nvim::vterm::parser::vterm_parser_set_callbacks;
use crate::src::nvim::vterm::pen::init_pen;
use crate::src::nvim::vterm::screen::{BUFIDX_ALTSCREEN, BUFIDX_PRIMARY};
use crate::src::nvim::vterm::vterm::{
    MOUSE_X10, VTERM_PROP_ALTSCREEN, VTERM_PROP_CURSORBLINK, VTERM_PROP_CURSORSHAPE,
    VTERM_PROP_CURSORVISIBLE, VTERM_PROP_FOCUSREPORT, VTERM_PROP_ICONNAME, VTERM_PROP_MOUSE,
    VTERM_PROP_MOUSE_DRAG, VTERM_PROP_MOUSE_MOVE, VTERM_PROP_REVERSE, VTERM_PROP_SYNCOUTPUT,
    VTERM_PROP_THEMEUPDATES, VTERM_PROP_TITLE, vterm_alloc, vterm_dealloc,
};
use crate::src::nvim::vterm::{csi, dcs, mode, selection, text};

use super::{MOUSE_WANT_CLICK, MOUSE_WANT_DRAG, MOUSE_WANT_MOVE, tabstop_bytes};

/// Allocates a state machine for `vt` and gives it its power-on settings.
unsafe fn vterm_state_new(vt: *mut VTerm) -> *mut VTermState {
    let state = vterm_alloc(::core::mem::size_of::<VTermState>()) as *mut VTermState;

    (*state).vt = vt;
    (*state).rows = (*vt).rows;
    (*state).cols = (*vt).cols;
    (*state).mouse_col = 0;
    (*state).mouse_row = 0;
    (*state).mouse_buttons = 0;
    (*state).mouse_protocol = MOUSE_X10;
    (*state).callbacks = core::ptr::null();
    (*state).cbdata = core::ptr::null_mut();
    (*state).selection.callbacks = core::ptr::null();
    (*state).selection.user = core::ptr::null_mut();
    (*state).selection.buffer = core::ptr::null_mut();

    init_pen(&mut *state);
    (*state).bold_is_highbright = 0;
    (*state).combine_pos.row = -1;

    (*state).tabstops = vterm_alloc(tabstop_bytes((*state).cols)).cast::<uint8_t>();
    let marks = ((*state).rows.max(0) as size_t) * ::core::mem::size_of::<VTermLineInfo>();
    // TODO(vterm): the altscreen's marks could wait until it is switched on.
    (*state).lineinfos[BUFIDX_PRIMARY] = vterm_alloc(marks).cast::<VTermLineInfo>();
    (*state).lineinfos[BUFIDX_ALTSCREEN] = vterm_alloc(marks).cast::<VTermLineInfo>();
    (*state).lineinfo = (*state).lineinfos[BUFIDX_PRIMARY];

    let utf8 = vterm_lookup_encoding(ENC_UTF8, b'u' as c_char);
    (*state).encoding_utf8.enc = utf8;
    if let Some(init) = (*utf8).init {
        init(
            utf8,
            (&raw mut (*state).encoding_utf8.data).cast::<c_void>(),
        );
    }

    for stack in &mut (*state).key_encoding_stacks {
        stack.items = [VTermKeyEncodingFlags {
            disambiguate_report_events_report_alternate_report_all_keys_report_associated: [0; 1],
        }; 16];
        stack.size = 1;
    }

    state
}

pub unsafe extern "C" fn vterm_state_free(state: *mut VTermState) {
    vterm_dealloc((*state).tabstops.cast::<c_void>());
    vterm_dealloc((*state).lineinfos[BUFIDX_PRIMARY].cast::<c_void>());
    if !(*state).lineinfos[BUFIDX_ALTSCREEN].is_null() {
        vterm_dealloc((*state).lineinfos[BUFIDX_ALTSCREEN].cast::<c_void>());
    }
    vterm_dealloc(state.cast::<c_void>());
}

/// The terminal's state machine, created and wired to the parser on first ask.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_obtain_state(vt: *mut VTerm) -> *mut VTermState {
    if !(*vt).state.is_null() {
        return (*vt).state;
    }
    let state = vterm_state_new(vt);
    (*vt).state = state;
    vterm_parser_set_callbacks(vt, PARSER_CALLBACKS.ptr(), state.cast::<c_void>());
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
    let state = &mut *(user as *mut VTermState);

    // A high bit selects the right-hand set, a single shift overrides both,
    // and any slot designated UTF-8 collapses onto the one shared decoder so
    // that a multi-byte sequence can span calls.
    let lead = if len == 0 { 0 } else { *bytes as u8 };
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

    let vt = state.vt;
    let codepoints = (*vt).tmpbuffer.cast::<u32>();
    let maxpoints = (*vt).tmpbuffer_len / ::core::mem::size_of::<u32>();
    let mut npoints: c_int = 0;
    let mut eaten: size_t = 0;
    let enc = (*encoding).enc;
    let decode = (*enc).decode.expect("character set without a decoder");
    decode(
        enc,
        (&raw mut (*encoding).data).cast::<c_void>(),
        codepoints,
        &raw mut npoints,
        if state.gsingle_set != 0 {
            1
        } else {
            maxpoints as c_int
        },
        bytes,
        &raw mut eaten,
        len,
    );

    // A stateful set may not have seen enough bytes for even one codepoint.
    if npoints == 0 {
        return eaten as c_int;
    }
    state.gsingle_set = 0;
    text::print(state, slice::from_raw_parts(codepoints, npoints as usize));
    eaten as c_int
}

unsafe extern "C" fn on_control(control: uint8_t, user: *mut c_void) -> c_int {
    let state = &mut *(user as *mut VTermState);
    if text::control(state, control) {
        return 1;
    }
    match state.fallback_table().and_then(|f| f.control) {
        Some(f) => (f(control, state.fbdata) != 0) as c_int,
        None => 0,
    }
}

unsafe extern "C" fn on_escape(bytes: *const c_char, len: size_t, user: *mut c_void) -> c_int {
    let state = &mut *(user as *mut VTermState);
    text::escape(state, slice::from_raw_parts(bytes.cast::<u8>(), len))
}

unsafe extern "C" fn on_csi(
    leader: *const c_char,
    args: *const c_long,
    argcount: c_int,
    intermed: *const c_char,
    command: c_char,
    user: *mut c_void,
) -> c_int {
    // A leader or intermediate arrives as a NUL-terminated string, but only a
    // single-byte one means anything here, so two bytes decide it.
    let leader_bytes = if leader.is_null() || *leader == 0 {
        [0, 0]
    } else {
        [*leader as u8, *leader.offset(1) as u8]
    };
    let intermed_bytes = if intermed.is_null() || *intermed == 0 {
        [0, 0]
    } else {
        [*intermed as u8, *intermed.offset(1) as u8]
    };
    let state = &mut *(user as *mut VTermState);
    let outcome = csi::dispatch(
        state,
        leader_bytes,
        slice::from_raw_parts(args, argcount.max(0) as usize),
        intermed_bytes,
        command as u8,
    );
    match outcome {
        csi::Outcome::Handled => 1,
        csi::Outcome::Ignored => 0,
        csi::Outcome::Unrecognised => {
            let fbdata = state.fbdata;
            match state.fallback_table().and_then(|f| f.csi) {
                Some(f) => (f(leader, args, argcount, intermed, command, fbdata) != 0) as c_int,
                None => 0,
            }
        }
    }
}

/// Upstream offers every OSC to the fallback, even the ones it handled
/// itself, and reports only what the fallback made of it.
unsafe extern "C" fn on_osc(command: c_int, frag: VTermStringFragment, user: *mut c_void) -> c_int {
    let state = &mut *(user as *mut VTermState);
    selection::osc(state, command, frag);
    match state.fallback_table().and_then(|f| f.osc) {
        Some(f) => (f(command, frag, state.fbdata) != 0) as c_int,
        None => 0,
    }
}

unsafe extern "C" fn on_dcs(
    command: *const c_char,
    commandlen: size_t,
    frag: VTermStringFragment,
    user: *mut c_void,
) -> c_int {
    let state = &mut *(user as *mut VTermState);
    let name = slice::from_raw_parts(command.cast::<u8>(), commandlen);
    if dcs::device_control(state, name, frag) {
        return 1;
    }
    let fbdata = state.fbdata;
    match state.fallback_table().and_then(|f| f.dcs) {
        Some(f) => (f(command, commandlen, frag, fbdata) != 0) as c_int,
        None => 0,
    }
}

// APC, PM and SOS carry nothing this terminal understands, so each only
// reaches for its own fallback.
unsafe extern "C" fn on_apc(frag: VTermStringFragment, user: *mut c_void) -> c_int {
    let state = &mut *(user as *mut VTermState);
    match state.fallback_table().and_then(|f| f.apc) {
        Some(f) => (f(frag, state.fbdata) != 0) as c_int,
        None => 0,
    }
}

unsafe extern "C" fn on_pm(frag: VTermStringFragment, user: *mut c_void) -> c_int {
    let state = &mut *(user as *mut VTermState);
    match state.fallback_table().and_then(|f| f.pm) {
        Some(f) => (f(frag, state.fbdata) != 0) as c_int,
        None => 0,
    }
}

unsafe extern "C" fn on_sos(frag: VTermStringFragment, user: *mut c_void) -> c_int {
    let state = &mut *(user as *mut VTermState);
    match state.fallback_table().and_then(|f| f.sos) {
        Some(f) => (f(frag, state.fbdata) != 0) as c_int,
        None => 0,
    }
}

unsafe extern "C" fn on_resize(rows: c_int, cols: c_int, user: *mut c_void) -> c_int {
    let state = &mut *(user as *mut VTermState);
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
    mode::reset(&mut *state, hard != 0);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_state_set_callbacks(
    state: *mut VTermState,
    callbacks: *const VTermStateCallbacks,
    user: *mut c_void,
) {
    let installed = !callbacks.is_null();
    (*state).callbacks = callbacks;
    (*state).cbdata = if installed {
        user
    } else {
        core::ptr::null_mut()
    };
    if installed {
        (*state).init_consumer_pen();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_state_set_unrecognised_fallbacks(
    state: *mut VTermState,
    fallbacks: *const VTermStateFallbacks,
    user: *mut c_void,
) {
    (*state).fallbacks = fallbacks;
    (*state).fbdata = if fallbacks.is_null() {
        core::ptr::null_mut()
    } else {
        user
    };
}

/// Applies a terminal property, offering it to the consumer first, which may
/// refuse it. Refusal matters most for the alternate screen: the state must
/// not believe it switched if the screen did not.
pub unsafe extern "C" fn vterm_state_set_termprop(
    state: *mut VTermState,
    prop: VTermProp,
    val: *mut VTermValue,
) -> c_int {
    let state = &mut *state;
    let cbdata = state.cbdata;
    if let Some(f) = state.callback_table().and_then(|c| c.settermprop)
        && f(prop, val, cbdata) == 0
    {
        return 0;
    }

    match prop {
        // Titles are passed straight through, never stored.
        VTERM_PROP_TITLE | VTERM_PROP_ICONNAME => {}
        VTERM_PROP_CURSORVISIBLE => state.mode.set_cursor_visible((*val).boolean as c_uint),
        VTERM_PROP_CURSORBLINK => state.mode.set_cursor_blink((*val).boolean as c_uint),
        VTERM_PROP_CURSORSHAPE => state.mode.set_cursor_shape((*val).number as c_uint),
        VTERM_PROP_REVERSE => state.mode.set_screen((*val).boolean as c_uint),
        VTERM_PROP_ALTSCREEN => {
            state.mode.set_alt_screen((*val).boolean as c_uint);
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
            let level = (*val).number;
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
        VTERM_PROP_FOCUSREPORT => state.mode.set_report_focus((*val).boolean as c_uint),
        VTERM_PROP_THEMEUPDATES => state.mode.set_theme_updates((*val).boolean as c_uint),
        VTERM_PROP_SYNCOUTPUT => state.mode.set_synchronized_output((*val).boolean as c_uint),
        _ => return 0,
    }
    1
}

/// Reports that the terminal window gained focus, if the host asked to hear.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_state_focus_in(state: *mut VTermState) {
    (*state).report_focus(b'I');
}

/// Reports that the terminal window lost focus, if the host asked to hear.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_state_focus_out(state: *mut VTermState) {
    (*state).report_focus(b'O');
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_state_get_lineinfo(
    state: *const VTermState,
    row: c_int,
) -> *const VTermLineInfo {
    (*state).lineinfo.offset(row as isize)
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
    let buffer = if buflen != 0 && buffer.is_null() {
        vterm_alloc(buflen).cast::<c_char>()
    } else {
        buffer
    };
    (*state).selection.callbacks = callbacks;
    (*state).selection.user = user;
    (*state).selection.buffer = buffer;
    (*state).selection.buflen = buflen;
}
