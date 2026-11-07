//! libtermkey's CSI driver: the escape sequences every terminal shares, as
//! opposed to the ones its terminfo description names.
//!
//! It handles control sequences (`CSI ... final`), single-shift-three
//! (`SS3 x`), and the control strings — DCS, OSC and APC — whose payload the
//! consumer retrieves with `TermKey::interpret_string`.
//!
//! Ported from libtermkey, Copyright (c) 2007-2011 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libtermkey-LICENSE.txt.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::narrow::as_index;
use crate::narrow::const_len_as_int;
use crate::tui::termkey::csi::{self, CSI_MAX_PARAMS, CsiSeq};
use crate::tui::termkey::keytables::{
    CSI_FINAL_BASE, CSI_FUNC_COUNT, CSI_FUNCS, CSI_SS3, SS3, SS3_KEYPAD_ALT,
};
use crate::tui::termkey::report::{self, Payload};
use crate::tui::termkey::termkey::{
    TERMKEY_EVENT_UNKNOWN, TERMKEY_FLAG_CONVERTKP, TERMKEY_KEYMOD_ALT, TERMKEY_RES_AGAIN,
    TERMKEY_RES_KEY, TERMKEY_RES_NONE, TERMKEY_SYM_UNKNOWN, TERMKEY_TYPE_APC, TERMKEY_TYPE_DCS,
    TERMKEY_TYPE_KEYSYM, TERMKEY_TYPE_MODEREPORT, TERMKEY_TYPE_MOUSE, TERMKEY_TYPE_OSC,
    TERMKEY_TYPE_POSITION, TERMKEY_TYPE_UNICODE, TERMKEY_TYPE_UNKNOWN_CSI, TermKey, emit_codepoint,
    peekkey_mouse,
};
use crate::tui::termkey::types::{KeyCode, TermKeyKey, TermKeyResult};
use core::ffi::c_int;

/// Parameters resolved against the input buffer. A `None` entry is an omitted
/// parameter, which reads as -1.
type Params<'a> = [Option<&'a [u8]>; CSI_MAX_PARAMS];

/// Sequence introducers and terminators, in both their seven-bit (an escape
/// then a character) and eight-bit (a single C1 byte) spellings.
///
/// Nested so these short, generic names stay out of the flat namespace the unit
/// tests' generated C declarations share.
mod byte {
    pub(super) const ESC: u8 = 0x1b;
    pub(super) const BEL: u8 = 0x07;
    pub(super) const SS3_7BIT: u8 = b'O';
    pub(super) const CSI_7BIT: u8 = b'[';
    pub(super) const DCS_7BIT: u8 = b'P';
    pub(super) const OSC_7BIT: u8 = b']';
    pub(super) const APC_7BIT: u8 = b'_';
    pub(super) const SS3_8BIT: u8 = 0x8f;
    pub(super) const DCS_8BIT: u8 = 0x90;
    pub(super) const CSI_8BIT: u8 = 0x9b;
    pub(super) const STRING_TERMINATOR_8BIT: u8 = 0x9c;
    pub(super) const OSC_8BIT: u8 = 0x9d;
}
use byte::{
    APC_7BIT, BEL, CSI_7BIT, CSI_8BIT, DCS_7BIT, DCS_8BIT, ESC, OSC_7BIT, OSC_8BIT, SS3_7BIT,
    SS3_8BIT, STRING_TERMINATOR_8BIT,
};

/// The driver's per-reader state: the last control string's payload, and
/// the serial number that ties it to the key that reported it.
#[derive(Default)]
pub struct CsiDriver {
    pub(super) saved_string_id: c_int,
    pub(super) saved_string: Vec<u8>,
}

/// Read a parameter's modifier field: terminals send the modifier mask plus
/// one, and may append the key event as a sub-parameter.
///
/// `None` means the sequence should be rejected because it named an event this
/// build does not know.
fn modifiers_and_event(key: &mut TermKeyKey, param: Option<&[u8]>) -> Option<()> {
    let Some(bytes) = param else {
        key.modifiers = 0;
        return Some(());
    };
    let (value, subparam) = csi::param_groups(Some(bytes));
    if let Some(subparam) = subparam {
        key.event = csi::parse_key_event(subparam);
        if key.event == TERMKEY_EVENT_UNKNOWN {
            return None;
        }
    }
    key.modifiers = value - 1;
    Some(())
}

/// `CSI ... A` and the SS3 commands that share their final bytes.
///
/// A private-use introducer or intermediate byte disqualifies the sequence:
/// upstream indexed its table with the *whole* packed command, so `CSI < A`
/// read thousands of entries past the end of a 64-entry array. Reporting it as
/// an unknown CSI is what the consumer already does with every other sequence
/// this driver does not claim.
fn handle_csi_ss3(key: &mut TermKeyKey, command: u32, params: &Params) -> TermKeyResult {
    let Ok(final_byte) = u8::try_from(command) else {
        return TERMKEY_RES_NONE;
    };
    if modifiers_and_event(key, params[1]).is_none() {
        return TERMKEY_RES_NONE;
    }
    let info = CSI_SS3[usize::from(final_byte - CSI_FINAL_BASE)];
    key.kind = info.kind;
    key.code = KeyCode::scalar(info.sym);
    key.modifiers &= !info.modifier_mask;
    key.modifiers |= info.modifier_set;
    if info.sym == TERMKEY_SYM_UNKNOWN {
        TERMKEY_RES_NONE
    } else {
        TERMKEY_RES_KEY
    }
}

/// `CSI N ~`, the numbered keys.
fn handle_csi_func(
    tk: &TermKey,
    key: &mut TermKeyKey,
    nparams: usize,
    params: &Params,
) -> TermKeyResult {
    if nparams == 0 {
        return TERMKEY_RES_NONE;
    }
    if modifiers_and_event(key, params[1]).is_none() {
        return TERMKEY_RES_NONE;
    }
    key.kind = TERMKEY_TYPE_KEYSYM;
    let number = csi::param_groups(params[0]).0;
    if number == 27 && params[2].is_some() {
        // `CSI 27 ; mod ; codepoint ~` names a plain character with modifiers.
        let modifiers = key.modifiers;
        emit_codepoint(tk, csi::param_groups(params[2]).0, key);
        key.modifiers |= modifiers;
    } else if (0..const_len_as_int(CSI_FUNC_COUNT)).contains(&number) {
        let info = CSI_FUNCS[as_index(number)];
        key.kind = info.kind;
        key.code = KeyCode::scalar(info.sym);
        key.modifiers &= !info.modifier_mask;
        key.modifiers |= info.modifier_set;
    } else {
        key.code = KeyCode::scalar(TERMKEY_SYM_UNKNOWN);
    }
    if key.code.sym() == TERMKEY_SYM_UNKNOWN {
        TERMKEY_RES_NONE
    } else {
        TERMKEY_RES_KEY
    }
}

/// `CSI codepoint ; mod u`, the unambiguous encoding for any key.
fn handle_csi_u(
    tk: &TermKey,
    key: &mut TermKeyKey,
    command: u32,
    params: &Params,
) -> TermKeyResult {
    // Upstream matched the whole packed command, so a private-use introducer
    // takes the sequence out of this encoding entirely.
    if command != u32::from(b'u') {
        return TERMKEY_RES_NONE;
    }
    if modifiers_and_event(key, params[1]).is_none() {
        return TERMKEY_RES_NONE;
    }
    let modifiers = key.modifiers;
    key.kind = TERMKEY_TYPE_KEYSYM;
    emit_codepoint(tk, csi::param_groups(params[0]).0, key);
    key.modifiers |= modifiers;
    TERMKEY_RES_KEY
}

/// Mouse reports: `CSI Cb ; Cx ; Cy M` (rxvt) and `CSI < Cb ; Cx ; Cy M/m`
/// (SGR). The X10 form, whose payload is three raw bytes rather than decimal
/// parameters, is decoded by `peekkey_mouse` instead.
fn handle_csi_mouse(
    key: &mut TermKeyKey,
    command: u32,
    nparams: usize,
    params: &Params,
) -> TermKeyResult {
    let final_byte = (command & 0xff) as u8;
    // Everything above the final byte, so an intermediate byte disqualifies the
    // sequence just as an unrecognised introducer does.
    let introducer = command >> 8;
    if final_byte != b'M' && final_byte != b'm' {
        return TERMKEY_RES_NONE;
    }
    if nparams < 3 {
        return TERMKEY_RES_NONE;
    }
    if introducer != 0 && introducer != u32::from(b'<') {
        return TERMKEY_RES_NONE;
    }
    key.kind = TERMKEY_TYPE_MOUSE;
    let mut payload: Payload = [0; 4];
    payload[0] = report::low_byte(csi::param_groups(params[0]).0);
    // Bits 2-4 of the button code are the modifiers; lift them out of it.
    key.modifiers = (c_int::from(payload[0]) & 0x1c) >> 2;
    payload[0] &= !0x1c;
    report::pack_position(
        &mut payload,
        csi::param_groups(params[1]).0,
        csi::param_groups(params[2]).0,
    );
    if introducer == u32::from(b'<') && final_byte == b'm' {
        report::mark_sgr_release(&mut payload);
    }
    key.code = KeyCode::report(payload);
    TERMKEY_RES_KEY
}

/// `CSI ? line ; col R`, the cursor position report. A plain `CSI R` is much
/// more likely to be F3, so it falls through to the final-byte tables.
fn handle_csi_position(
    key: &mut TermKeyKey,
    command: u32,
    nparams: usize,
    params: &Params,
) -> TermKeyResult {
    if command != u32::from(b'?') << 8 | u32::from(b'R') {
        return handle_csi_ss3(key, command, params);
    }
    if nparams < 2 {
        return TERMKEY_RES_NONE;
    }
    key.kind = TERMKEY_TYPE_POSITION;
    let mut payload: Payload = [0; 4];
    report::pack_position(
        &mut payload,
        csi::param_groups(params[1]).0,
        csi::param_groups(params[0]).0,
    );
    key.code = KeyCode::report(payload);
    TERMKEY_RES_KEY
}

/// `CSI [?] mode ; value $ y`, the DECRPM mode report.
fn handle_csi_mode(
    key: &mut TermKeyKey,
    command: u32,
    nparams: usize,
    params: &Params,
) -> TermKeyResult {
    let initial = (command >> 8 & 0xff).cast_signed();
    if command >> 16 != u32::from(b'$') || command & 0xff != u32::from(b'y') {
        return TERMKEY_RES_NONE;
    }
    if initial != 0 && initial != c_int::from(b'?') {
        return TERMKEY_RES_NONE;
    }
    if nparams < 2 {
        return TERMKEY_RES_NONE;
    }
    key.kind = TERMKEY_TYPE_MODEREPORT;
    let mut payload: Payload = [0; 4];
    report::pack_mode(
        &mut payload,
        initial,
        csi::param_groups(params[0]).0,
        csi::param_groups(params[1]).0,
    );
    key.code = KeyCode::report(payload);
    TERMKEY_RES_KEY
}

/// Run `f` with the buffer advanced past `consumed` bytes, then put it back and
/// fold those bytes into the count `f` reported.
fn with_buffer_advanced(
    tk: &mut TermKey,
    consumed: usize,
    nbytep: &mut usize,
    f: impl FnOnce(&TermKey, &mut usize) -> TermKeyResult,
) -> TermKeyResult {
    tk.buffstart += consumed;
    tk.buffcount -= consumed;
    let result = f(tk, nbytep);
    tk.buffstart -= consumed;
    tk.buffcount += consumed;
    if result == TERMKEY_RES_KEY {
        *nbytep += consumed;
    }
    result
}

fn peek_csi(
    tk: &mut TermKey,
    intro_len: usize,
    key: &mut TermKeyKey,
    force: bool,
    nbytep: &mut usize,
) -> TermKeyResult {
    let bytes = tk.buffered();
    let Some(seq) = csi::parse(bytes, intro_len) else {
        if !force {
            return TERMKEY_RES_AGAIN;
        }
        // Out of patience: the introducer was just Alt-[ after all.
        emit_codepoint(tk, c_int::from(CSI_7BIT), key);
        key.modifiers |= TERMKEY_KEYMOD_ALT;
        *nbytep = intro_len;
        return TERMKEY_RES_KEY;
    };
    // `CSI M` with fewer than three parameters is the X10 mouse protocol, whose
    // three payload bytes follow the sequence rather than sitting inside it.
    if seq.command == u32::from(b'M') && seq.nparams < 3 {
        let peek = |tk: &TermKey, nbytep: &mut usize| peekkey_mouse(tk, key, nbytep);
        return with_buffer_advanced(tk, seq.len, nbytep, peek);
    }

    let mut params: Params = [None; CSI_MAX_PARAMS];
    for (slot, param) in params.iter_mut().zip(seq.params.iter()) {
        *slot = param.map(|(start, end)| &bytes[start..end]);
    }
    let result = dispatch(tk, key, &seq, &params);
    if result == TERMKEY_RES_NONE {
        // Nothing here recognises it. Report it whole, consuming only the
        // introducer, and let the consumer re-parse the rest before the
        // next `peekkey` discards it.
        key.kind = TERMKEY_TYPE_UNKNOWN_CSI;
        key.code = KeyCode::scalar(seq.command.cast_signed());
        key.modifiers = 0;
        tk.hightide = seq.len - intro_len;
        *nbytep = intro_len;
        return TERMKEY_RES_KEY;
    }
    *nbytep = seq.len;
    result
}

/// Pick the handler for a sequence's final byte.
///
/// Upstream kept a 64-slot table of function pointers filled at start-up. The
/// entries were fixed, and `R` was deliberately overwritten after the
/// final-byte tables had claimed it — which is why the position handler falls
/// back to them.
fn dispatch(tk: &TermKey, key: &mut TermKeyKey, seq: &CsiSeq, params: &Params) -> TermKeyResult {
    match (seq.command & 0xff) as u8 {
        b'u' => handle_csi_u(tk, key, seq.command, params),
        b'M' | b'm' => handle_csi_mouse(key, seq.command, seq.nparams, params),
        b'R' => handle_csi_position(key, seq.command, seq.nparams, params),
        b'y' => handle_csi_mode(key, seq.command, seq.nparams, params),
        b'~' => handle_csi_func(tk, key, seq.nparams, params),
        // `csi::parse` only ever stops on a byte in the final-byte range, so
        // the index is in bounds.
        final_byte
            if CSI_SS3[(final_byte - CSI_FINAL_BASE) as usize].sym != TERMKEY_SYM_UNKNOWN =>
        {
            handle_csi_ss3(key, seq.command, params)
        }
        _ => TERMKEY_RES_NONE,
    }
}

fn peek_ss3(
    tk: &TermKey,
    intro_len: usize,
    key: &mut TermKeyKey,
    force: bool,
    nbytep: &mut usize,
) -> TermKeyResult {
    let bytes = tk.buffered();
    if bytes.len() < intro_len + 1 {
        if !force {
            return TERMKEY_RES_AGAIN;
        }
        // Out of patience: the introducer was just Alt-O after all.
        let len = bytes.len();
        emit_codepoint(tk, c_int::from(SS3_7BIT), key);
        key.modifiers |= TERMKEY_KEYMOD_ALT;
        *nbytep = len;
        return TERMKEY_RES_KEY;
    }
    let command = bytes[intro_len];
    if !(0x40..0x80).contains(&command) {
        return TERMKEY_RES_NONE;
    }
    let slot = (command - CSI_FINAL_BASE) as usize;
    let mut info = CSI_SS3[slot];
    if info.sym == TERMKEY_SYM_UNKNOWN {
        if tk.flags & TERMKEY_FLAG_CONVERTKP != 0 && SS3_KEYPAD_ALT[slot] != 0 {
            // The consumer wants keypad keys as the characters they stand for.
            let codepoint = c_int::from(SS3_KEYPAD_ALT[slot]);
            key.kind = TERMKEY_TYPE_UNICODE;
            key.code = KeyCode::scalar(codepoint);
            key.modifiers = 0;
            key.utf8[0] = SS3_KEYPAD_ALT[slot];
            key.utf8[1] = 0;
            *nbytep = intro_len + 1;
            return TERMKEY_RES_KEY;
        }
        info = SS3[slot];
    }
    if info.sym == TERMKEY_SYM_UNKNOWN {
        return TERMKEY_RES_NONE;
    }
    key.kind = info.kind;
    key.code = KeyCode::scalar(info.sym);
    key.modifiers = info.modifier_set;
    *nbytep = intro_len + 1;
    TERMKEY_RES_KEY
}

/// A control string — DCS, OSC or APC — whose payload runs to a BEL or a string
/// terminator. The payload is kept aside for `TermKey::interpret_string`, and
/// the key carries a serial number so a stale key cannot read a newer payload.
fn peek_control_string(
    tk: &mut TermKey,
    intro_len: usize,
    key: &mut TermKeyKey,
    nbytep: &mut usize,
) -> TermKeyResult {
    let bytes = tk.buffered();
    let Some(end) = (intro_len..bytes.len()).find(|&i| {
        bytes[i] == BEL
            || bytes[i] == STRING_TERMINATOR_8BIT
            || (bytes[i] == ESC && bytes.get(i + 1) == Some(&b'\\'))
    }) else {
        return TERMKEY_RES_AGAIN;
    };
    // A two-byte `ESC \` terminator takes one more byte than a one-byte one.
    *nbytep = end + 1 + usize::from(bytes[end] == ESC);

    // The introducer's low five bits distinguish the three string kinds, in
    // both their seven- and eight-bit spellings.
    key.kind = match bytes[intro_len - 1] & 0x1f {
        0x10 => TERMKEY_TYPE_DCS,
        0x1d => TERMKEY_TYPE_OSC,
        0x1f => TERMKEY_TYPE_APC,
        other => unreachable!("control string introducer {other:#x}"),
    };
    let payload = bytes[intro_len..end].to_vec();
    tk.csi.saved_string = payload;
    tk.csi.saved_string_id += 1;
    key.code = KeyCode::scalar(tk.csi.saved_string_id);
    key.modifiers = 0;
    TERMKEY_RES_KEY
}

pub fn peek_key(
    tk: &mut TermKey,
    key: &mut TermKeyKey,
    force: bool,
    nbytep: &mut usize,
) -> TermKeyResult {
    if tk.buffcount == 0 {
        return TERMKEY_RES_NONE;
    }
    let bytes = tk.buffered();
    match bytes[0] {
        ESC => match bytes.get(1) {
            // Not enough to tell yet. Upstream reports NONE rather than AGAIN
            // and leaves the waiting to `peekkey_simple`, which handles a lone
            // escape as Alt-<next key>.
            None => TERMKEY_RES_NONE,
            Some(&SS3_7BIT) => peek_ss3(tk, 2, key, force, nbytep),
            Some(&CSI_7BIT) => peek_csi(tk, 2, key, force, nbytep),
            Some(&DCS_7BIT | &OSC_7BIT | &APC_7BIT) => peek_control_string(tk, 2, key, nbytep),
            _ => TERMKEY_RES_NONE,
        },
        SS3_8BIT => peek_ss3(tk, 1, key, force, nbytep),
        CSI_8BIT => peek_csi(tk, 1, key, force, nbytep),
        DCS_8BIT | OSC_8BIT => peek_control_string(tk, 1, key, nbytep),
        _ => TERMKEY_RES_NONE,
    }
}
