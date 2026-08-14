//! The escape-sequence parser: a byte stream in, parser events out.
//!
//! Bytes arrive from the child process and leave as calls into the
//! `VTermParserCallbacks` the consumer installed — text, control characters,
//! escape sequences, control sequences, and the fragments of the
//! string-carrying sequences (OSC, DCS, APC, PM, SOS). The only state kept
//! between calls is the small block in `VTerm::parser`, so a sequence split
//! across two writes resumes where it left off.
//!
//! That block is reached through [`Parser`], whose *construction* carries the
//! promise that the terminal and the callback table installed in it are live.
//! The state machine built on top of it — accumulating a sequence's leader,
//! parameters and payload, and reporting each completed event — is then
//! ordinary checked code.
//!
//! Ported from libvterm, Copyright (c) 2008 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libvterm-LICENSE.txt.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::types::{
    VTerm, VTerm_parser, VTerm_parser_v_csi, VTerm_parser_v_dcs, VTerm_parser_v_osc,
    VTermParserCallbacks, VTermParserState, VTermStringFragment, VTermTerminator, size_t,
};
use crate::src::nvim::vterm::vterm::{VTERM_TERMINATOR_BEL, VTERM_TERMINATOR_ST};
use core::ffi::{c_char, c_int, c_long, c_void};
use core::ops::{Deref, DerefMut};
use core::slice;

/// Intermediate bytes held for an escape or control sequence.
const INTERMED_MAX: usize = 16;
/// Private-marker bytes held ahead of a control sequence's parameters.
const CSI_LEADER_MAX: usize = 16;
/// Parameters a control sequence can carry.
const CSI_ARGS_MAX: usize = 32;

/// The value stored for a parameter the sender left empty.
const CSI_ARG_MISSING: c_long = 0x7fff_ffff;
/// Set on a parameter that was separated from the next by `:` rather than `;`.
const CSI_ARG_FLAG_MORE: c_long = 0x8000_0000u32 as c_long;

/// Where the parser is in a sequence.
///
/// The discriminants are the values stored in `VTerm::parser::state`, and
/// their order matters: everything from `OscCommand` up is accumulating a
/// string, which is what [`ParserState::is_string`] tests.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(u32)]
enum ParserState {
    Normal = 0,
    CsiLeader = 1,
    CsiArgs = 2,
    CsiIntermed = 3,
    DcsCommand = 4,
    OscCommand = 5,
    Osc = 6,
    Dcs = 7,
    Apc = 8,
    Pm = 9,
    Sos = 10,
}

impl ParserState {
    fn from_raw(raw: VTermParserState) -> Self {
        match raw {
            1 => ParserState::CsiLeader,
            2 => ParserState::CsiArgs,
            3 => ParserState::CsiIntermed,
            4 => ParserState::DcsCommand,
            5 => ParserState::OscCommand,
            6 => ParserState::Osc,
            7 => ParserState::Dcs,
            8 => ParserState::Apc,
            9 => ParserState::Pm,
            10 => ParserState::Sos,
            _ => ParserState::Normal,
        }
    }

    fn raw(self) -> VTermParserState {
        self as VTermParserState
    }

    /// True while the parser is collecting the payload of a string-carrying
    /// sequence, which changes how control characters and ESC are handled.
    fn is_string(self) -> bool {
        self >= ParserState::OscCommand
    }
}

/// Intermediate bytes are the 0x20-0x2f range, which can appear in both
/// escape and control sequences before the final byte.
fn is_intermed(c: u8) -> bool {
    (0x20..=0x2f).contains(&c)
}

/// Something the parser recognised, on its way to the consumer's callback.
enum ParserEvent<'a> {
    /// A control character the parser does not act on itself.
    Control(u8),
    /// A complete control sequence; the byte is its final character, and the
    /// leader, parameters and intermediates are in `VTerm::parser`.
    Csi(u8),
    /// A complete escape sequence; the byte is its final character.
    Escape(u8),
    /// A piece of the payload of the string sequence currently open.
    StringFragment {
        bytes: &'a [u8],
        /// The terminator has been seen; this is the last piece.
        last: bool,
        terminator: VTermTerminator,
    },
    /// Ordinary text; the callback reports how much of it it consumed.
    Text(&'a [u8]),
}

/// The parser block of a live terminal, and the consumer's callback table.
///
/// Constructing one is where this file's unsafety collects: the promise that
/// the terminal behind the pointer and the table installed in it stay live is
/// made once, by [`Parser::of`], and every read and write of the block after
/// that is plain field access through [`Deref`].
///
/// A consumer callback is free to re-enter the terminal, and the calls out to
/// one hand it pointers *into* the parser block, so no borrow of the block
/// may be held across a report — which is what `&mut self` on
/// [`Parser::dispatch`] enforces.
struct Parser {
    vt: *mut VTerm,
}

impl Deref for Parser {
    type Target = VTerm_parser;

    fn deref(&self) -> &VTerm_parser {
        // SAFETY: the wrapper promised the terminal stays live, and it is the
        // only one over this terminal, so no other reference to the block is
        // outstanding.
        unsafe { &(*self.vt).parser }
    }
}

impl DerefMut for Parser {
    fn deref_mut(&mut self) -> &mut VTerm_parser {
        // SAFETY: as for `deref`, and `&mut self` rules out a second borrow
        // through this wrapper.
        unsafe { &mut (*self.vt).parser }
    }
}

impl Parser {
    /// The parser of the terminal `vt` points at.
    ///
    /// # Safety
    ///
    /// `vt` must point at a live terminal for as long as the wrapper is used,
    /// and the callback table and data pointer installed through
    /// [`vterm_parser_set_callbacks`] must stay valid for that long. No other
    /// wrapper over the same terminal may exist at the same time.
    unsafe fn of(vt: *mut VTerm) -> Self {
        Parser { vt }
    }

    /// The consumer's callback table, if it installed one.
    fn handlers(&self) -> Option<&VTermParserCallbacks> {
        // SAFETY: constructing the wrapper promised the installed table is
        // live, so the pointer is either null or a readable table.
        unsafe { self.callbacks.as_ref() }
    }

    /// Whether an 8-bit C1 control byte is recognised as one. In UTF-8 mode
    /// those bytes are continuation bytes and are left to the text callback.
    fn c1_allowed(&self) -> bool {
        // SAFETY: the wrapper promised the terminal stays live.
        unsafe { (*self.vt).mode.utf8() == 0 }
    }

    /// The control-sequence arm of the per-sequence union, live from
    /// `CsiLeader` until the sequence ends.
    ///
    /// Every arm of the union is integers, and the terminal was zeroed when
    /// it was allocated, so reading the arm the parser is not in reads stale
    /// values rather than uninitialised ones; which arm is *meaningful* is
    /// what [`VTerm_parser::state`] says.
    fn csi(&self) -> &VTerm_parser_v_csi {
        // SAFETY: the arm is initialised whichever one was written last.
        unsafe { &self.v.csi }
    }

    fn csi_mut(&mut self) -> &mut VTerm_parser_v_csi {
        // SAFETY: as for `csi`.
        unsafe { &mut self.v.csi }
    }

    /// The OSC arm, live from `OscCommand` until the string ends.
    fn osc(&self) -> &VTerm_parser_v_osc {
        // SAFETY: as for `csi`.
        unsafe { &self.v.osc }
    }

    fn osc_mut(&mut self) -> &mut VTerm_parser_v_osc {
        // SAFETY: as for `csi`.
        unsafe { &mut self.v.osc }
    }

    /// The DCS arm, live from `DcsCommand` until the string ends.
    fn dcs(&self) -> &VTerm_parser_v_dcs {
        // SAFETY: as for `csi`.
        unsafe { &self.v.dcs }
    }

    fn dcs_mut(&mut self) -> &mut VTerm_parser_v_dcs {
        // SAFETY: as for `csi`.
        unsafe { &mut self.v.dcs }
    }

    /// Collects one intermediate byte, if there is room; the last slot is
    /// reserved for the NUL the consumer's escape callback is handed.
    fn push_intermed(&mut self, c: u8) {
        let len = self.intermedlen as usize;
        if len < INTERMED_MAX - 1 {
            self.intermed[len] = c as c_char;
            self.intermedlen += 1;
        }
    }

    /// NUL-terminates the intermediates ahead of reporting a sequence.
    fn terminate_intermed(&mut self) {
        let len = self.intermedlen as usize;
        self.intermed[len] = 0;
    }

    /// Hands `event` to the callback registered for it, if there is one.
    ///
    /// Returns how many bytes a [`ParserEvent::Text`] was consumed by; zero
    /// for every other event.
    fn dispatch(&mut self, event: ParserEvent) -> usize {
        let cbdata = self.cbdata;

        match event {
            ParserEvent::Control(control) => {
                if let Some(f) = self.handlers().and_then(|c| c.control) {
                    // SAFETY: the consumer's own callback, taking the control
                    // byte and the data it registered with.
                    unsafe { f(control, cbdata) };
                }
            }
            ParserEvent::Csi(command) => {
                if let Some(f) = self.handlers().and_then(|c| c.csi) {
                    let csi = self.csi();
                    // A missing leader or intermediate is reported as NULL,
                    // not as an empty string.
                    let leader = if csi.leaderlen != 0 {
                        csi.leader.as_ptr()
                    } else {
                        core::ptr::null()
                    };
                    let args = csi.args.as_ptr();
                    let argi = csi.argi;
                    let intermed = if self.intermedlen != 0 {
                        self.intermed.as_ptr()
                    } else {
                        core::ptr::null()
                    };
                    let command = command as c_char;
                    // SAFETY: the consumer's own callback. The leader,
                    // parameter and intermediate pointers are into the parser
                    // block, which outlives the call, and nothing here holds
                    // a borrow of that block across it.
                    unsafe { f(leader, args, argi, intermed, command, cbdata) };
                }
            }
            ParserEvent::Escape(command) => {
                // The consumer sees the intermediates followed by the final
                // byte, NUL-terminated.
                let mut seq = [0 as c_char; INTERMED_MAX + 1];
                let intermedlen = (self.intermedlen.max(0) as usize).min(seq.len() - 1);
                seq[..intermedlen].copy_from_slice(&self.intermed[..intermedlen]);
                seq[intermedlen] = command as c_char;
                if let Some(f) = self.handlers().and_then(|c| c.escape) {
                    // SAFETY: the consumer's own callback, taking `seq`,
                    // which outlives the call, and its own data.
                    unsafe { f(seq.as_ptr(), intermedlen + 1, cbdata) };
                }
            }
            ParserEvent::StringFragment {
                bytes,
                last,
                terminator,
            } => {
                let mut frag = VTermStringFragment {
                    str: bytes.as_ptr().cast::<c_char>(),
                    len_initial_final_0: [0; 4],
                    terminator,
                };
                frag.set_len(bytes.len());
                frag.set_initial(self.string_initial);
                frag.set_final_0(last);

                match ParserState::from_raw(self.state) {
                    ParserState::Osc => {
                        if let Some(f) = self.handlers().and_then(|c| c.osc) {
                            let command = self.osc().command;
                            // SAFETY: the consumer's own callback, taking the
                            // fragment by value and its own data.
                            unsafe { f(command, frag, cbdata) };
                        }
                    }
                    ParserState::Dcs => {
                        if let Some(f) = self.handlers().and_then(|c| c.dcs) {
                            let dcs = self.dcs();
                            let command = dcs.command.as_ptr();
                            let commandlen = dcs.commandlen.max(0) as size_t;
                            // SAFETY: the consumer's own callback. The
                            // command pointer is into the parser block, which
                            // outlives the call.
                            unsafe { f(command, commandlen, frag, cbdata) };
                        }
                    }
                    ParserState::Apc => {
                        if let Some(f) = self.handlers().and_then(|c| c.apc) {
                            // SAFETY: the consumer's own callback.
                            unsafe { f(frag, cbdata) };
                        }
                    }
                    ParserState::Pm => {
                        if let Some(f) = self.handlers().and_then(|c| c.pm) {
                            // SAFETY: the consumer's own callback.
                            unsafe { f(frag, cbdata) };
                        }
                    }
                    ParserState::Sos => {
                        if let Some(f) = self.handlers().and_then(|c| c.sos) {
                            // SAFETY: the consumer's own callback.
                            unsafe { f(frag, cbdata) };
                        }
                    }
                    // Nothing is open yet: the sequence's own command bytes
                    // are not a fragment of its payload.
                    _ => return 0,
                }
                // Only the first fragment of a string is the initial one.
                self.string_initial = false;
            }
            ParserEvent::Text(text) => {
                if let Some(f) = self.handlers().and_then(|c| c.text) {
                    let bytes = text.as_ptr().cast::<c_char>();
                    // SAFETY: the consumer's own callback, taking the text
                    // this write is still holding and its own data.
                    let eaten = unsafe { f(bytes, text.len(), cbdata) };
                    return eaten.max(0) as usize;
                }
            }
        }
        0
    }
}

/// The payload accumulated for the open string sequence, up to `pos`.
///
/// `start` is `None` before any payload has been seen — during an OSC's
/// command digits, say — where upstream passed a dangling pointer to a
/// fragment callback that discards it anyway.
fn pending(input: &[u8], start: Option<usize>, pos: usize) -> &[u8] {
    match start {
        Some(start) if start <= pos => &input[start..pos.min(input.len())],
        _ => &[],
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_input_write(
    vt: *mut VTerm,
    bytes: *const c_char,
    len: size_t,
) -> size_t {
    // SAFETY: the caller promises `bytes` points at `len` readable bytes that
    // stay put for the call.
    let input = unsafe { slice::from_raw_parts(bytes.cast::<u8>(), len) };
    // SAFETY: the caller promises a live terminal, and the callback table it
    // was given through `vterm_parser_set_callbacks` is still installed.
    let mut parser = unsafe { Parser::of(vt) };

    // Index into `input` where the payload of the open string sequence
    // begins. A sequence left open by an earlier call resumes at byte zero.
    let mut string_start = ParserState::from_raw(parser.state)
        .is_string()
        .then_some(0usize);
    let mut pos = 0usize;

    while pos < input.len() {
        let mut c = input[pos];
        let mut c1_allowed = parser.c1_allowed();
        let mut state = ParserState::from_raw(parser.state);

        'byte: {
            // NUL and DEL are filler; they interrupt a string but never end it.
            if c == 0x00 || c == 0x7f {
                if state.is_string() {
                    parser.dispatch(ParserEvent::StringFragment {
                        bytes: pending(input, string_start, pos),
                        last: false,
                        terminator: VTERM_TERMINATOR_ST,
                    });
                    string_start = Some(pos + 1);
                }
                if parser.emit_nul {
                    parser.dispatch(ParserEvent::Control(c));
                }
                break 'byte;
            }
            // CAN and SUB abandon whatever was in progress.
            if c == 0x18 || c == 0x1a {
                parser.set_in_esc(false);
                parser.state = ParserState::Normal.raw();
                string_start = None;
                if parser.emit_nul {
                    parser.dispatch(ParserEvent::Control(c));
                }
                break 'byte;
            }
            if c == 0x1b {
                parser.intermedlen = 0;
                if !state.is_string() {
                    parser.state = ParserState::Normal.raw();
                }
                parser.set_in_esc(true);
                break 'byte;
            }
            // Inside a string BEL stands in for ST, so it is handled below
            // with the terminators rather than here with the controls.
            if !(c == 0x07 && state.is_string()) && c < 0x20 {
                // SOS is defined to swallow every other control character.
                if state == ParserState::Sos {
                    break 'byte;
                }
                if state.is_string() {
                    parser.dispatch(ParserEvent::StringFragment {
                        bytes: pending(input, string_start, pos),
                        last: false,
                        terminator: VTERM_TERMINATOR_ST,
                    });
                }
                parser.dispatch(ParserEvent::Control(c));
                if ParserState::from_raw(parser.state).is_string() {
                    string_start = Some(pos + 1);
                }
                break 'byte;
            }

            let mut string_len = string_start.map_or(0, |start| pos.saturating_sub(start));

            if parser.in_esc() {
                // `ESC X` is the 7-bit spelling of the C1 control X + 0x40.
                // Inside a string only `ESC \` (ST) is recognised, so that a
                // stray ESC can't be mistaken for a new sequence.
                if parser.intermedlen == 0
                    && (0x40..0x60).contains(&c)
                    && (!state.is_string() || c == 0x5c)
                {
                    c += 0x40;
                    c1_allowed = true;
                    // The ESC itself is not part of the payload.
                    string_len = string_len.saturating_sub(1);
                    parser.set_in_esc(false);
                } else {
                    string_start = None;
                    state = ParserState::Normal;
                    parser.state = state.raw();
                }
            }

            // The CSI sections fall through to each other and an OSC command
            // falls through to the string collector, so which section runs is
            // a loop variable rather than a match arm.
            let mut section = state;
            loop {
                match section {
                    ParserState::CsiLeader => {
                        // Private-use markers, 0x3c to 0x3f.
                        if (0x3c..=0x3f).contains(&c) {
                            let csi = parser.csi_mut();
                            if (csi.leaderlen as usize) < CSI_LEADER_MAX - 1 {
                                csi.leader[csi.leaderlen as usize] = c as c_char;
                                csi.leaderlen += 1;
                            }
                            break;
                        }
                        let csi = parser.csi_mut();
                        csi.leader[csi.leaderlen as usize] = 0;
                        csi.argi = 0;
                        csi.args[0] = CSI_ARG_MISSING;
                        parser.state = ParserState::CsiArgs.raw();
                        section = ParserState::CsiArgs;
                    }
                    ParserState::CsiArgs => {
                        let csi = parser.csi_mut();
                        let argi = csi.argi as usize;
                        if c.is_ascii_digit() {
                            if csi.args[argi] == CSI_ARG_MISSING {
                                csi.args[argi] = 0;
                            }
                            // A sender can run the digits past what a `long`
                            // holds; wrap rather than trap, as the C did.
                            csi.args[argi] = csi.args[argi]
                                .wrapping_mul(10)
                                .wrapping_add(c_long::from(c - b'0'));
                            break;
                        }
                        // A colon separates sub-parameters of one parameter;
                        // it is otherwise a semicolon with a flag set.
                        if c == b':' {
                            csi.args[argi] |= CSI_ARG_FLAG_MORE;
                            c = b';';
                        }
                        // Upstream advanced `argi` unbounded, so a sequence
                        // with 32 or more parameters ran off the end of
                        // `args`. Parameters past the last slot now overwrite
                        // it instead.
                        if c == b';' {
                            if argi + 1 < CSI_ARGS_MAX {
                                csi.argi += 1;
                            }
                            csi.args[csi.argi as usize] = CSI_ARG_MISSING;
                            break;
                        }
                        csi.argi = (csi.argi + 1).min(CSI_ARGS_MAX as c_int);
                        parser.intermedlen = 0;
                        parser.state = ParserState::CsiIntermed.raw();
                        section = ParserState::CsiIntermed;
                    }
                    ParserState::CsiIntermed => {
                        if is_intermed(c) {
                            parser.push_intermed(c);
                            break;
                        }
                        // ESC cancels the sequence; a final byte completes it;
                        // anything else was malformed. All three end it.
                        if c != 0x1b && (0x40..=0x7e).contains(&c) {
                            parser.terminate_intermed();
                            parser.dispatch(ParserEvent::Csi(c));
                        }
                        parser.state = ParserState::Normal.raw();
                        string_start = None;
                        break;
                    }
                    ParserState::OscCommand => {
                        let osc = parser.osc_mut();
                        if c.is_ascii_digit() {
                            let base = if osc.command == -1 {
                                0
                            } else {
                                osc.command.wrapping_mul(10)
                            };
                            osc.command = base.wrapping_add(c_int::from(c - b'0'));
                            break;
                        }
                        if c == b';' {
                            parser.state = ParserState::Osc.raw();
                            string_start = Some(pos + 1);
                            break;
                        }
                        // No command digits at all: the payload starts here.
                        string_start = Some(pos);
                        string_len = 0;
                        parser.state = ParserState::Osc.raw();
                        section = ParserState::Osc;
                    }
                    ParserState::DcsCommand => {
                        let dcs = parser.dcs_mut();
                        if (dcs.commandlen as usize) < CSI_LEADER_MAX {
                            dcs.command[dcs.commandlen as usize] = c as c_char;
                            dcs.commandlen += 1;
                        }
                        if (0x40..=0x7e).contains(&c) {
                            string_start = Some(pos + 1);
                            parser.state = ParserState::Dcs.raw();
                        }
                        break;
                    }
                    ParserState::Osc
                    | ParserState::Dcs
                    | ParserState::Apc
                    | ParserState::Pm
                    | ParserState::Sos => {
                        if c == 0x07 || (c1_allowed && c == 0x9c) {
                            let start = string_start.unwrap_or(pos);
                            parser.dispatch(ParserEvent::StringFragment {
                                bytes: &input[start..(start + string_len).min(input.len())],
                                last: true,
                                terminator: if c == 0x07 {
                                    VTERM_TERMINATOR_BEL
                                } else {
                                    VTERM_TERMINATOR_ST
                                },
                            });
                            parser.state = ParserState::Normal.raw();
                            string_start = None;
                        }
                        break;
                    }
                    ParserState::Normal => {
                        if parser.in_esc() {
                            if is_intermed(c) {
                                parser.push_intermed(c);
                            } else if (0x30..0x7f).contains(&c) {
                                parser.dispatch(ParserEvent::Escape(c));
                                parser.set_in_esc(false);
                                parser.state = ParserState::Normal.raw();
                                string_start = None;
                            }
                            break;
                        }
                        if c1_allowed && (0x80..0xa0).contains(&c) {
                            match c {
                                // DCS
                                0x90 => {
                                    parser.string_initial = true;
                                    parser.dcs_mut().commandlen = 0;
                                    parser.state = ParserState::DcsCommand.raw();
                                    string_start = None;
                                }
                                // SOS
                                0x98 => {
                                    parser.string_initial = true;
                                    parser.state = ParserState::Sos.raw();
                                    string_start = Some(pos + 1);
                                }
                                // CSI
                                0x9b => {
                                    parser.csi_mut().leaderlen = 0;
                                    parser.state = ParserState::CsiLeader.raw();
                                    string_start = None;
                                }
                                // OSC
                                0x9d => {
                                    parser.osc_mut().command = -1;
                                    parser.string_initial = true;
                                    parser.state = ParserState::OscCommand.raw();
                                    string_start = None;
                                }
                                // PM
                                0x9e => {
                                    parser.string_initial = true;
                                    parser.state = ParserState::Pm.raw();
                                    string_start = Some(pos + 1);
                                }
                                // APC
                                0x9f => {
                                    parser.string_initial = true;
                                    parser.state = ParserState::Apc.raw();
                                    string_start = Some(pos + 1);
                                }
                                _ => {
                                    parser.dispatch(ParserEvent::Control(c));
                                }
                            }
                            break;
                        }
                        // Plain text. The callback takes as much as it wants;
                        // if it takes nothing, force a byte of progress.
                        let eaten = parser.dispatch(ParserEvent::Text(&input[pos..])).max(1);
                        pos += eaten - 1;
                        break;
                    }
                }
            }
        }
        pos += 1;
    }

    // Hand over whatever payload this write ended in the middle of.
    if let Some(start) = string_start {
        let mut string_len = pos.min(input.len()).saturating_sub(start);
        if string_len > 0 {
            // A trailing ESC may yet turn out to be the ST that ends the
            // string, so it is not part of the payload.
            if parser.in_esc() {
                string_len -= 1;
            }
            parser.dispatch(ParserEvent::StringFragment {
                bytes: &input[start..start + string_len],
                last: false,
                terminator: VTERM_TERMINATOR_ST,
            });
        }
    }

    len
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_parser_set_callbacks(
    vt: *mut VTerm,
    callbacks: *const VTermParserCallbacks,
    user: *mut c_void,
) {
    // SAFETY: the caller promises a live terminal. The table being installed
    // is the one the wrapper's promise will then be about.
    let mut parser = unsafe { Parser::of(vt) };
    parser.callbacks = callbacks;
    parser.cbdata = user;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_states_are_the_ones_that_collect_a_payload() {
        assert!(!ParserState::Normal.is_string());
        assert!(!ParserState::CsiLeader.is_string());
        assert!(!ParserState::CsiIntermed.is_string());
        // The DCS command bytes come before the payload.
        assert!(!ParserState::DcsCommand.is_string());
        for state in [
            ParserState::OscCommand,
            ParserState::Osc,
            ParserState::Dcs,
            ParserState::Apc,
            ParserState::Pm,
            ParserState::Sos,
        ] {
            assert!(state.is_string(), "{state:?}");
        }
    }

    #[test]
    fn states_round_trip_through_their_stored_value() {
        for state in [
            ParserState::Normal,
            ParserState::CsiLeader,
            ParserState::CsiArgs,
            ParserState::CsiIntermed,
            ParserState::DcsCommand,
            ParserState::OscCommand,
            ParserState::Osc,
            ParserState::Dcs,
            ParserState::Apc,
            ParserState::Pm,
            ParserState::Sos,
        ] {
            assert_eq!(ParserState::from_raw(state as VTermParserState), state);
        }
        // Anything the field could not have held reads back as Normal.
        assert_eq!(ParserState::from_raw(11), ParserState::Normal);
    }

    #[test]
    fn intermediates_are_the_punctuation_range() {
        assert!(!is_intermed(0x1f));
        assert!(is_intermed(0x20));
        assert!(is_intermed(b'!'));
        assert!(is_intermed(0x2f));
        assert!(!is_intermed(0x30));
    }

    #[test]
    fn pending_payload_is_empty_until_one_starts() {
        let input = b"abcdef";
        assert_eq!(pending(input, None, 3), b"");
        assert_eq!(pending(input, Some(1), 3), b"bc");
        assert_eq!(pending(input, Some(3), 3), b"");
        // A payload marked as starting past the end of this write.
        assert_eq!(pending(input, Some(6), 6), b"");
        assert_eq!(pending(input, Some(4), 99), b"ef");
    }
}
