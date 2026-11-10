//! The escape-sequence parser: a byte stream in, parser events out.
//!
//! Bytes arrive from the child process and leave as calls into the
//! [`ParserHandler`] the terminal reports to — text, control characters,
//! escape sequences, control sequences, and the fragments of the
//! string-carrying sequences (OSC, DCS, APC, PM, SOS). The only state kept
//! between calls is the small block in `VTerm::parser`, so a sequence split
//! across two writes resumes where it left off.
//!
//! Ported from libvterm, Copyright (c) 2008 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libvterm-LICENSE.txt.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::deref::deref_to_field;
use crate::narrow::as_index;
use crate::narrow::const_len_as_int;
use crate::vterm::host::{Csi, Fragment, ParserHandler};
use crate::vterm::internal::VTermParserState;
use crate::vterm::types::{VTerm_parser, VTermTerminator};
use crate::vterm::vterm::{VTERM_TERMINATOR_BEL, VTERM_TERMINATOR_ST};
use core::ffi::{CStr, c_int, c_long, c_uint};

/// Intermediate bytes held for an escape or control sequence.
const INTERMED_MAX: usize = 16;
/// Private-marker bytes held ahead of a control sequence's parameters.
const CSI_LEADER_MAX: usize = 16;
/// Parameters a control sequence can carry.
const CSI_ARGS_MAX: usize = 32;

/// The value stored for a parameter the sender left empty.
pub(crate) const CSI_ARG_MISSING: c_long = 2147483647;
/// Set on a parameter that is followed by a sub-parameter, i.e. one written
/// `38:5:1` rather than `38;5;1`: separated from the next by `:`, not `;`.
pub(crate) const CSI_ARG_FLAG_MORE: c_long = 1 << 31;
/// The value bits of a parameter, below the flag. Spelled as an expression
/// ffigen does not evaluate, so the unit fixtures' `CSI_ARG_MASK` macro
/// (`shim.h`) is the only C spelling.
pub(crate) const CSI_ARG_MASK: c_uint = !((1 as c_uint) << 31);

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
#[derive(Debug)]
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

/// The parser block of a terminal and the consumer it reports to, for the
/// length of one write.
#[derive(Debug)]
struct Parser<'a, H: ParserHandler + ?Sized> {
    block: &'a mut VTerm_parser,
    handler: &'a mut H,
    /// Whether the input is decoded as UTF-8, in which case 8-bit C1 control
    /// bytes are continuation bytes and are left to the text callback.
    utf8: bool,
}

deref_to_field!(impl[H: ParserHandler + ?Sized] Parser<'_, H> => mut VTerm_parser, block);

impl<H: ParserHandler + ?Sized> Parser<'_, H> {
    /// Whether an 8-bit C1 control byte is recognised as one.
    fn c1_allowed(&self) -> bool {
        !self.utf8
    }

    /// Collects one intermediate byte, if there is room; the last slot is
    /// reserved for the NUL the consumer's escape callback is handed.
    fn push_intermed(&mut self, c: u8) {
        let len = as_index(self.intermedlen);
        if len < INTERMED_MAX - 1 {
            self.intermed[len] = c;
            self.intermedlen += 1;
        }
    }

    /// NUL-terminates the intermediates ahead of reporting a sequence.
    fn terminate_intermed(&mut self) {
        let len = as_index(self.intermedlen);
        self.intermed[len] = 0;
    }

    /// Hands `event` to the consumer.
    ///
    /// Returns how many bytes a [`ParserEvent::Text`] was consumed by; zero
    /// for every other event.
    fn dispatch(&mut self, event: ParserEvent) -> usize {
        let Parser { block, handler, .. } = self;
        match event {
            ParserEvent::Control(control) => {
                let _ = handler.control(control);
            }
            ParserEvent::Csi(command) => {
                let csi = &block.v;
                // A missing leader or intermediate is reported as absent,
                // not as an empty string.
                let leader = (csi.leaderlen != 0)
                    .then(|| CStr::from_bytes_until_nul(&csi.leader).ok())
                    .flatten();
                let intermed = (block.intermedlen != 0)
                    .then(|| CStr::from_bytes_until_nul(&block.intermed).ok())
                    .flatten();
                let argc = as_index(csi.argi.max(0)).min(csi.args.len());
                let csi = Csi {
                    leader,
                    args: &csi.args[..argc],
                    intermed,
                    command,
                };
                let _ = handler.csi(&csi);
            }
            ParserEvent::Escape(command) => {
                // The consumer sees the intermediates followed by the final
                // byte, NUL-terminated.
                let mut seq = [0u8; INTERMED_MAX + 1];
                let intermedlen = as_index(block.intermedlen.max(0)).min(seq.len() - 1);
                seq[..intermedlen].copy_from_slice(&block.intermed[..intermedlen]);
                seq[intermedlen] = command;
                let _ = handler.escape(&seq[..=intermedlen]);
            }
            ParserEvent::StringFragment {
                bytes,
                last,
                terminator,
            } => {
                let frag = Fragment::new(bytes, block.string_initial, last, terminator);
                match ParserState::from_raw(block.state) {
                    ParserState::Osc => {
                        let _ = handler.osc(block.v.leaderlen, frag);
                    }
                    ParserState::Dcs => {
                        let len = as_index(block.v.leaderlen.max(0)).min(block.v.leader.len());
                        let _ = handler.dcs(&block.v.leader[..len], frag);
                    }
                    ParserState::Apc => {
                        let _ = handler.apc(frag);
                    }
                    ParserState::Pm => {
                        let _ = handler.pm(frag);
                    }
                    ParserState::Sos => {
                        let _ = handler.sos(frag);
                    }
                    // Nothing is open yet: the sequence's own command bytes
                    // are not a fragment of its payload.
                    _ => return 0,
                }
                // Only the first fragment of a string is the initial one.
                block.string_initial = false;
            }
            ParserEvent::Text(text) => {
                if let Some(eaten) = handler.text(text) {
                    return as_index(eaten.max(0));
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

/// Where one write is in its input.
#[derive(Debug)]
struct Cursor<'i> {
    input: &'i [u8],
    /// The byte being parsed.
    pos: usize,
    /// Index into `input` where the payload of the open string sequence
    /// begins; `None` while no payload has started.
    string_start: Option<usize>,
}

impl<'i> Cursor<'i> {
    /// The payload accumulated for the open string sequence, up to the
    /// byte being parsed.
    fn pending(&self) -> &'i [u8] {
        pending(self.input, self.string_start, self.pos)
    }

    /// The payload starts after the byte being parsed.
    fn start_string_after(&mut self) {
        self.string_start = Some(self.pos + 1);
    }
}

impl<H: ParserHandler + ?Sized> Parser<'_, H> {
    /// Acts on `c` if it is one of the bytes that mean the same wherever
    /// the parser is — filler, cancel, ESC, a C0 control — and answers
    /// whether it was.
    fn filler_or_control(&mut self, cur: &mut Cursor<'_>, c: u8, state: ParserState) -> bool {
        // NUL and DEL are filler; they interrupt a string but never end it.
        if c == 0x00 || c == 0x7f {
            if state.is_string() {
                self.dispatch(ParserEvent::StringFragment {
                    bytes: cur.pending(),
                    last: false,
                    terminator: VTERM_TERMINATOR_ST,
                });
                cur.start_string_after();
            }
            if self.emit_nul {
                self.dispatch(ParserEvent::Control(c));
            }
            return true;
        }
        // CAN and SUB abandon whatever was in progress.
        if c == 0x18 || c == 0x1a {
            self.set_in_esc(false);
            self.state = ParserState::Normal.raw();
            cur.string_start = None;
            if self.emit_nul {
                self.dispatch(ParserEvent::Control(c));
            }
            return true;
        }
        if c == 0x1b {
            self.intermedlen = 0;
            if !state.is_string() {
                self.state = ParserState::Normal.raw();
            }
            self.set_in_esc(true);
            return true;
        }
        // Inside a string BEL stands in for ST, so it is handled with the
        // terminators rather than here with the controls.
        if !(c == 0x07 && state.is_string()) && c < 0x20 {
            // SOS is defined to swallow every other control character.
            if state == ParserState::Sos {
                return true;
            }
            if state.is_string() {
                self.dispatch(ParserEvent::StringFragment {
                    bytes: cur.pending(),
                    last: false,
                    terminator: VTERM_TERMINATOR_ST,
                });
            }
            self.dispatch(ParserEvent::Control(c));
            if ParserState::from_raw(self.state).is_string() {
                cur.start_string_after();
            }
            return true;
        }
        false
    }

    /// Feeds `c` to the sequence in progress, or to the text when there is
    /// none.
    fn sequence_byte(&mut self, cur: &mut Cursor<'_>, mut c: u8, mut state: ParserState) {
        let mut c1_allowed = self.c1_allowed();
        let mut string_len = cur
            .string_start
            .map_or(0, |start| cur.pos.saturating_sub(start));

        if self.in_esc() {
            // `ESC X` is the 7-bit spelling of the C1 control X + 0x40.
            // Inside a string only `ESC \` (ST) is recognised, so that a
            // stray ESC can't be mistaken for a new sequence.
            if self.intermedlen == 0
                && (0x40..0x60).contains(&c)
                && (!state.is_string() || c == 0x5c)
            {
                c += 0x40;
                c1_allowed = true;
                // The ESC itself is not part of the payload.
                string_len = string_len.saturating_sub(1);
                self.set_in_esc(false);
            } else {
                cur.string_start = None;
                state = ParserState::Normal;
                self.state = state.raw();
            }
        }

        // The CSI sections fall through to each other and an OSC command
        // falls through to the string collector: a section answers the one
        // that takes the byte next, if any.
        let mut section = Some(state);
        while let Some(current) = section {
            section = match current {
                ParserState::CsiLeader => self.csi_leader(c),
                ParserState::CsiArgs => self.csi_args(c),
                ParserState::CsiIntermed => {
                    self.csi_intermed(cur, c);
                    None
                }
                ParserState::OscCommand => self.osc_command(cur, c, &mut string_len),
                ParserState::DcsCommand => {
                    self.dcs_command(cur, c);
                    None
                }
                ParserState::Osc
                | ParserState::Dcs
                | ParserState::Apc
                | ParserState::Pm
                | ParserState::Sos => {
                    self.string_byte(cur, c, c1_allowed, string_len);
                    None
                }
                ParserState::Normal => {
                    self.normal_byte(cur, c, c1_allowed);
                    None
                }
            };
        }
    }

    /// A control sequence's private-use markers, 0x3c to 0x3f. Anything
    /// else ends them and starts the parameters.
    fn csi_leader(&mut self, c: u8) -> Option<ParserState> {
        let csi = &mut self.v;
        if (0x3c..=0x3f).contains(&c) {
            if as_index(csi.leaderlen) < CSI_LEADER_MAX - 1 {
                csi.leader[as_index(csi.leaderlen)] = c;
                csi.leaderlen += 1;
            }
            return None;
        }
        csi.leader[as_index(csi.leaderlen)] = 0;
        csi.argi = 0;
        csi.args[0] = CSI_ARG_MISSING;
        self.state = ParserState::CsiArgs.raw();
        Some(ParserState::CsiArgs)
    }

    /// A control sequence's parameters: digits and separators. Anything
    /// else ends them and starts the intermediates.
    fn csi_args(&mut self, mut c: u8) -> Option<ParserState> {
        let csi = &mut self.v;
        let argi = as_index(csi.argi);
        if c.is_ascii_digit() {
            if csi.args[argi] == CSI_ARG_MISSING {
                csi.args[argi] = 0;
            }
            // A sender can run the digits past what a `long` holds; wrap
            // rather than trap, as the C did.
            csi.args[argi] = csi.args[argi]
                .wrapping_mul(10)
                .wrapping_add(c_long::from(c - b'0'));
            return None;
        }
        // A colon separates sub-parameters of one parameter; it is
        // otherwise a semicolon with a flag set.
        if c == b':' {
            csi.args[argi] |= CSI_ARG_FLAG_MORE;
            c = b';';
        }
        // Upstream advanced `argi` unbounded, so a sequence with 32 or more
        // parameters ran off the end of `args`. Parameters past the last
        // slot now overwrite it instead.
        if c == b';' {
            if argi + 1 < CSI_ARGS_MAX {
                csi.argi += 1;
            }
            csi.args[as_index(csi.argi)] = CSI_ARG_MISSING;
            return None;
        }
        csi.argi = (csi.argi + 1).min(const_len_as_int(CSI_ARGS_MAX));
        self.intermedlen = 0;
        self.state = ParserState::CsiIntermed.raw();
        Some(ParserState::CsiIntermed)
    }

    /// A control sequence's intermediates, then the byte that ends it.
    fn csi_intermed(&mut self, cur: &mut Cursor<'_>, c: u8) {
        if is_intermed(c) {
            self.push_intermed(c);
            return;
        }
        // ESC cancels the sequence; a final byte completes it; anything
        // else was malformed. All three end it.
        if c != 0x1b && (0x40..=0x7e).contains(&c) {
            self.terminate_intermed();
            self.dispatch(ParserEvent::Csi(c));
        }
        self.state = ParserState::Normal.raw();
        cur.string_start = None;
    }

    /// An OSC's command number, up to the `;` that starts its payload. A
    /// payload with no number starts at once, in the string collector.
    fn osc_command(
        &mut self,
        cur: &mut Cursor<'_>,
        c: u8,
        string_len: &mut usize,
    ) -> Option<ParserState> {
        // An OSC keeps its command number where a control sequence keeps
        // its leader's length.
        let command = &mut self.v.leaderlen;
        if c.is_ascii_digit() {
            let base = if *command == -1 {
                0
            } else {
                command.wrapping_mul(10)
            };
            *command = base.wrapping_add(c_int::from(c - b'0'));
            return None;
        }
        if c == b';' {
            self.state = ParserState::Osc.raw();
            cur.start_string_after();
            return None;
        }
        // No command digits at all: the payload starts here.
        cur.string_start = Some(cur.pos);
        *string_len = 0;
        self.state = ParserState::Osc.raw();
        Some(ParserState::Osc)
    }

    /// A DCS's command bytes, up to the final byte that starts its payload.
    fn dcs_command(&mut self, cur: &mut Cursor<'_>, c: u8) {
        // A DCS keeps its command bytes where a control sequence keeps its
        // leader.
        let dcs = &mut self.v;
        if as_index(dcs.leaderlen) < CSI_LEADER_MAX {
            dcs.leader[as_index(dcs.leaderlen)] = c;
            dcs.leaderlen += 1;
        }
        if (0x40..=0x7e).contains(&c) {
            cur.start_string_after();
            self.state = ParserState::Dcs.raw();
        }
    }

    /// A byte of a string sequence's payload: only a terminator does
    /// anything, handing over the payload as its last fragment.
    fn string_byte(&mut self, cur: &mut Cursor<'_>, c: u8, c1_allowed: bool, string_len: usize) {
        if c == 0x07 || (c1_allowed && c == 0x9c) {
            let start = cur.string_start.unwrap_or(cur.pos);
            self.dispatch(ParserEvent::StringFragment {
                bytes: &cur.input[start..(start + string_len).min(cur.input.len())],
                last: true,
                terminator: if c == 0x07 {
                    VTERM_TERMINATOR_BEL
                } else {
                    VTERM_TERMINATOR_ST
                },
            });
            self.state = ParserState::Normal.raw();
            cur.string_start = None;
        }
    }

    /// A byte outside any sequence: an escape sequence's intermediates and
    /// final byte, a C1 control, or text.
    fn normal_byte(&mut self, cur: &mut Cursor<'_>, c: u8, c1_allowed: bool) {
        if self.in_esc() {
            if is_intermed(c) {
                self.push_intermed(c);
            } else if (0x30..0x7f).contains(&c) {
                self.dispatch(ParserEvent::Escape(c));
                self.set_in_esc(false);
                self.state = ParserState::Normal.raw();
                cur.string_start = None;
            }
            return;
        }
        if c1_allowed && (0x80..0xa0).contains(&c) {
            self.c1_control(cur, c);
            return;
        }
        // Plain text. The callback takes as much as it wants; if it takes
        // nothing, force a byte of progress.
        let eaten = self
            .dispatch(ParserEvent::Text(&cur.input[cur.pos..]))
            .max(1);
        cur.pos += eaten - 1;
    }

    /// An 8-bit C1 control: the introducers open their sequence, the rest
    /// go to the consumer.
    fn c1_control(&mut self, cur: &mut Cursor<'_>, c: u8) {
        match c {
            // DCS
            0x90 => {
                self.string_initial = true;
                self.v.leaderlen = 0;
                self.state = ParserState::DcsCommand.raw();
                cur.string_start = None;
            }
            // SOS
            0x98 => {
                self.string_initial = true;
                self.state = ParserState::Sos.raw();
                cur.start_string_after();
            }
            // CSI
            0x9b => {
                self.v.leaderlen = 0;
                self.state = ParserState::CsiLeader.raw();
                cur.string_start = None;
            }
            // OSC
            0x9d => {
                self.v.leaderlen = -1;
                self.string_initial = true;
                self.state = ParserState::OscCommand.raw();
                cur.string_start = None;
            }
            // PM
            0x9e => {
                self.string_initial = true;
                self.state = ParserState::Pm.raw();
                cur.start_string_after();
            }
            // APC
            0x9f => {
                self.string_initial = true;
                self.state = ParserState::Apc.raw();
                cur.start_string_after();
            }
            _ => {
                self.dispatch(ParserEvent::Control(c));
            }
        }
    }

    /// Hands over whatever payload the write ended in the middle of.
    fn flush_pending(&mut self, cur: &Cursor<'_>) {
        let Some(start) = cur.string_start else {
            return;
        };
        let mut string_len = cur.pos.min(cur.input.len()).saturating_sub(start);
        if string_len > 0 {
            // A trailing ESC may yet turn out to be the ST that ends the
            // string, so it is not part of the payload.
            if self.in_esc() {
                string_len -= 1;
            }
            self.dispatch(ParserEvent::StringFragment {
                bytes: &cur.input[start..start + string_len],
                last: false,
                terminator: VTERM_TERMINATOR_ST,
            });
        }
    }
}

/// Parses `input`, the next bytes of the child's output, reporting what it
/// recognises to `handler`. A sequence `input` ends in the middle of resumes
/// on the next call.
pub(crate) fn write<H: ParserHandler + ?Sized>(
    block: &mut VTerm_parser,
    utf8: bool,
    input: &[u8],
    handler: &mut H,
) {
    let mut parser = Parser {
        block,
        handler,
        utf8,
    };
    // A string sequence left open by an earlier call resumes at byte zero.
    let mut cur = Cursor {
        input,
        pos: 0,
        string_start: ParserState::from_raw(parser.state)
            .is_string()
            .then_some(0usize),
    };

    while cur.pos < input.len() {
        let c = input[cur.pos];
        let state = ParserState::from_raw(parser.state);
        if !parser.filler_or_control(&mut cur, c, state) {
            parser.sequence_byte(&mut cur, c, state);
        }
        cur.pos += 1;
    }

    parser.flush_pending(&cur);
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
