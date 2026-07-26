//! libtermkey's CSI driver: the escape sequences every terminal shares, as
//! opposed to the ones its terminfo description names.
//!
//! It handles control sequences (`CSI ... final`), single-shift-three
//! (`SS3 x`), and the control strings — DCS, OSC and APC — whose payload the
//! consumer retrieves with `termkey_interpret_string`.

use crate::src::nvim::memory::{xfree, xmalloc};
use crate::src::nvim::tui::termkey::csi::{self, CSI_MAX_PARAMS, CsiSeq};
use crate::src::nvim::tui::termkey::keytables::{
    CSI_FINAL_BASE, CSI_FUNC_COUNT, CSI_FUNCS, CSI_SS3, SS3, SS3_KEYPAD_ALT,
};
use crate::src::nvim::tui::termkey::report::{self, Payload};
use crate::src::nvim::tui::termkey::termkey::{
    TERMKEY_EVENT_UNKNOWN, TERMKEY_FLAG_CONVERTKP, TERMKEY_KEYMOD_ALT, TERMKEY_RES_AGAIN,
    TERMKEY_RES_KEY, TERMKEY_RES_NONE, TERMKEY_SYM_UNKNOWN, TERMKEY_TYPE_APC, TERMKEY_TYPE_DCS,
    TERMKEY_TYPE_KEYSYM, TERMKEY_TYPE_MODEREPORT, TERMKEY_TYPE_MOUSE, TERMKEY_TYPE_OSC,
    TERMKEY_TYPE_POSITION, TERMKEY_TYPE_UNICODE, TERMKEY_TYPE_UNKNOWN_CSI, emit_codepoint,
    peekkey_mouse,
};
pub use crate::src::nvim::types::{
    TermKey, TermKeyCsi, TermKeyCsiParam, TermKeyKey, TermKeyKey_code, TermKeyMouseEvent,
    TermKeyResult, keyinfo, size_t,
};
use core::ffi::{c_char, c_int, c_uint, c_void};

/// Parameters resolved against the input buffer. A `None` entry is an omitted
/// parameter, which reads as -1.
type Params<'a> = [Option<&'a [u8]>; CSI_MAX_PARAMS];

/// Sequence introducers and terminators, in both their seven-bit (an escape
/// then a character) and eight-bit (a single C1 byte) spellings.
///
/// Nested so these short, generic names stay out of the flat namespace the unit
/// tests' generated C declarations share.
mod byte {
    pub const ESC: u8 = 0x1b;
    pub const BEL: u8 = 0x07;
    pub const SS3_7BIT: u8 = b'O';
    pub const CSI_7BIT: u8 = b'[';
    pub const DCS_7BIT: u8 = b'P';
    pub const OSC_7BIT: u8 = b']';
    pub const APC_7BIT: u8 = b'_';
    pub const SS3_8BIT: u8 = 0x8f;
    pub const DCS_8BIT: u8 = 0x90;
    pub const CSI_8BIT: u8 = 0x9b;
    pub const STRING_TERMINATOR_8BIT: u8 = 0x9c;
    pub const OSC_8BIT: u8 = 0x9d;
}
use byte::{
    APC_7BIT, BEL, CSI_7BIT, CSI_8BIT, DCS_7BIT, DCS_8BIT, ESC, OSC_7BIT, OSC_8BIT, SS3_7BIT,
    SS3_8BIT, STRING_TERMINATOR_8BIT,
};

pub unsafe fn new_driver() -> *mut TermKeyCsi {
    // The struct is C-shaped and freed with xfree, so it is filled by hand.
    let csi = xmalloc(size_of::<TermKeyCsi>()) as *mut TermKeyCsi;
    (*csi).saved_string_id = 0;
    (*csi).saved_string = core::ptr::null_mut();
    csi
}

pub unsafe fn free_driver(csi: *mut TermKeyCsi) {
    if !(*csi).saved_string.is_null() {
        xfree((*csi).saved_string as *mut c_void);
    }
    xfree(csi as *mut c_void);
}

/// The value of a CSI parameter, ignoring any sub-parameters. An omitted
/// parameter reads as -1.
pub unsafe fn csi_param_value(param: TermKeyCsiParam) -> c_int {
    csi::param_groups(param_bytes(param)).0
}

unsafe fn param_bytes<'a>(param: TermKeyCsiParam) -> Option<&'a [u8]> {
    if param.param.is_null() {
        None
    } else {
        Some(core::slice::from_raw_parts(param.param, param.length))
    }
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
    if command > 0xff {
        return TERMKEY_RES_NONE;
    }
    if modifiers_and_event(key, params[1]).is_none() {
        return TERMKEY_RES_NONE;
    }
    let info = CSI_SS3[(command as u8 - CSI_FINAL_BASE) as usize];
    key.type_0 = info.type_0;
    key.code.sym = info.sym;
    key.modifiers &= !info.modifier_mask;
    key.modifiers |= info.modifier_set;
    if info.sym == TERMKEY_SYM_UNKNOWN {
        TERMKEY_RES_NONE
    } else {
        TERMKEY_RES_KEY
    }
}

/// `CSI N ~`, the numbered keys.
unsafe fn handle_csi_func(
    tk: *mut TermKey,
    key: *mut TermKeyKey,
    nparams: usize,
    params: &Params,
) -> TermKeyResult {
    if nparams == 0 {
        return TERMKEY_RES_NONE;
    }
    if modifiers_and_event(&mut *key, params[1]).is_none() {
        return TERMKEY_RES_NONE;
    }
    (*key).type_0 = TERMKEY_TYPE_KEYSYM;
    let number = csi::param_groups(params[0]).0;
    if number == 27 && params[2].is_some() {
        // `CSI 27 ; mod ; codepoint ~` names a plain character with modifiers.
        let modifiers = (*key).modifiers;
        emit_codepoint(tk, csi::param_groups(params[2]).0, key);
        (*key).modifiers |= modifiers;
    } else if (0..CSI_FUNC_COUNT as c_int).contains(&number) {
        let info = CSI_FUNCS[number as usize];
        (*key).type_0 = info.type_0;
        (*key).code.sym = info.sym;
        (*key).modifiers &= !info.modifier_mask;
        (*key).modifiers |= info.modifier_set;
    } else {
        (*key).code.sym = TERMKEY_SYM_UNKNOWN;
    }
    if (*key).code.sym == TERMKEY_SYM_UNKNOWN {
        TERMKEY_RES_NONE
    } else {
        TERMKEY_RES_KEY
    }
}

/// `CSI codepoint ; mod u`, the unambiguous encoding for any key.
unsafe fn handle_csi_u(
    tk: *mut TermKey,
    key: *mut TermKeyKey,
    command: u32,
    params: &Params,
) -> TermKeyResult {
    // Upstream matched the whole packed command, so a private-use introducer
    // takes the sequence out of this encoding entirely.
    if command != b'u' as u32 {
        return TERMKEY_RES_NONE;
    }
    if modifiers_and_event(&mut *key, params[1]).is_none() {
        return TERMKEY_RES_NONE;
    }
    let modifiers = (*key).modifiers;
    (*key).type_0 = TERMKEY_TYPE_KEYSYM;
    emit_codepoint(tk, csi::param_groups(params[0]).0, key);
    (*key).modifiers |= modifiers;
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
    if introducer != 0 && introducer != b'<' as u32 {
        return TERMKEY_RES_NONE;
    }
    key.type_0 = TERMKEY_TYPE_MOUSE;
    let mut payload: Payload = [0; 4];
    payload[0] = csi::param_groups(params[0]).0 as c_char;
    // Bits 2-4 of the button code are the modifiers; lift them out of it.
    key.modifiers = (payload[0] as c_int & 0x1c) >> 2;
    payload[0] = (payload[0] as c_int & !0x1c) as c_char;
    report::pack_position(
        &mut payload,
        csi::param_groups(params[1]).0,
        csi::param_groups(params[2]).0,
    );
    if introducer == b'<' as u32 && final_byte == b'm' {
        report::mark_sgr_release(&mut payload);
    }
    key.code = TermKeyKey_code { mouse: payload };
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
    if command != (b'?' as u32) << 8 | b'R' as u32 {
        return handle_csi_ss3(key, command, params);
    }
    if nparams < 2 {
        return TERMKEY_RES_NONE;
    }
    key.type_0 = TERMKEY_TYPE_POSITION;
    let mut payload: Payload = [0; 4];
    report::pack_position(
        &mut payload,
        csi::param_groups(params[1]).0,
        csi::param_groups(params[0]).0,
    );
    key.code = TermKeyKey_code { mouse: payload };
    TERMKEY_RES_KEY
}

/// `CSI [?] mode ; value $ y`, the DECRPM mode report.
fn handle_csi_mode(
    key: &mut TermKeyKey,
    command: u32,
    nparams: usize,
    params: &Params,
) -> TermKeyResult {
    let initial = (command >> 8 & 0xff) as c_int;
    if command >> 16 != b'$' as u32 || command & 0xff != b'y' as u32 {
        return TERMKEY_RES_NONE;
    }
    if initial != 0 && initial != b'?' as c_int {
        return TERMKEY_RES_NONE;
    }
    if nparams < 2 {
        return TERMKEY_RES_NONE;
    }
    key.type_0 = TERMKEY_TYPE_MODEREPORT;
    let mut payload: Payload = [0; 4];
    report::pack_mode(
        &mut payload,
        initial,
        csi::param_groups(params[0]).0,
        csi::param_groups(params[1]).0,
    );
    key.code = TermKeyKey_code { mouse: payload };
    TERMKEY_RES_KEY
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_interpret_mouse(
    _tk: *mut TermKey,
    key: *const TermKeyKey,
    event: *mut TermKeyMouseEvent,
    button: *mut c_int,
    line: *mut c_int,
    col: *mut c_int,
) -> TermKeyResult {
    if (*key).type_0 != TERMKEY_TYPE_MOUSE {
        return TERMKEY_RES_NONE;
    }
    let payload: &Payload = &(*key).code.mouse;
    let (packed_line, packed_col) = report::unpack_position(payload);
    if !line.is_null() {
        *line = packed_line;
    }
    if !col.is_null() {
        *col = packed_col;
    }
    // Upstream zeroes the button before deciding whether it has an event to
    // report, so a caller asking only for the position gets button 0.
    if !button.is_null() {
        *button = 0;
    }
    if event.is_null() {
        return TERMKEY_RES_KEY;
    }
    let (decoded_event, decoded_button) = report::decode_mouse(payload);
    *event = decoded_event;
    if !button.is_null() {
        *button = decoded_button;
    }
    TERMKEY_RES_KEY
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_interpret_position(
    _tk: *mut TermKey,
    key: *const TermKeyKey,
    line: *mut c_int,
    col: *mut c_int,
) -> TermKeyResult {
    if (*key).type_0 != TERMKEY_TYPE_POSITION {
        return TERMKEY_RES_NONE;
    }
    let (packed_line, packed_col) = report::unpack_position(&(*key).code.mouse);
    if !line.is_null() {
        *line = packed_line;
    }
    if !col.is_null() {
        *col = packed_col;
    }
    TERMKEY_RES_KEY
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_interpret_modereport(
    _tk: *mut TermKey,
    key: *const TermKeyKey,
    initial: *mut c_int,
    mode: *mut c_int,
    value: *mut c_int,
) -> TermKeyResult {
    if (*key).type_0 != TERMKEY_TYPE_MODEREPORT {
        return TERMKEY_RES_NONE;
    }
    let (packed_initial, packed_mode, packed_value) = report::unpack_mode(&(*key).code.mouse);
    if !initial.is_null() {
        *initial = packed_initial;
    }
    if !mode.is_null() {
        *mode = packed_mode;
    }
    if !value.is_null() {
        *value = packed_value;
    }
    TERMKEY_RES_KEY
}

/// Re-parse the control sequence a TERMKEY_TYPE_UNKNOWN_CSI key stands for, so
/// the consumer can decide what to make of it.
///
/// The sequence is still sitting at the head of the buffer: `peekkey` consumed
/// only its introducer and parked the rest under `TermKey::hightide`, which it
/// discards on the next call. `params` must have room for
/// `csi::CSI_MAX_PARAMS`, which upstream assumed rather than checked — it
/// ignores the count the caller passes in and only writes it back.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_interpret_csi(
    tk: *mut TermKey,
    key: *const TermKeyKey,
    params: *mut TermKeyCsiParam,
    nparams: *mut size_t,
    cmd: *mut c_uint,
) -> TermKeyResult {
    if (*tk).hightide == 0 || (*key).type_0 != TERMKEY_TYPE_UNKNOWN_CSI {
        return TERMKEY_RES_NONE;
    }
    let bytes = buffered(tk);
    let Some(seq) = csi::parse(bytes, 0) else {
        return TERMKEY_RES_AGAIN;
    };
    for (i, param) in seq.params[..seq.nparams].iter().enumerate() {
        *params.add(i) = match *param {
            Some((start, end)) => TermKeyCsiParam {
                param: bytes.as_ptr().add(start),
                length: end - start,
            },
            None => TermKeyCsiParam {
                param: core::ptr::null(),
                length: 0,
            },
        };
    }
    *nparams = seq.nparams;
    *cmd = seq.command;
    TERMKEY_RES_KEY
}

/// The unconsumed input, as a slice.
unsafe fn buffered<'a>(tk: *mut TermKey) -> &'a [u8] {
    core::slice::from_raw_parts((*tk).buffer.add((*tk).buffstart), (*tk).buffcount)
}

/// Run `f` with the buffer advanced past `consumed` bytes, then put it back and
/// fold those bytes into the count `f` reported.
unsafe fn with_buffer_advanced(
    tk: *mut TermKey,
    consumed: usize,
    nbytep: *mut size_t,
    f: impl FnOnce(*mut TermKey) -> TermKeyResult,
) -> TermKeyResult {
    (*tk).buffstart += consumed;
    (*tk).buffcount -= consumed;
    let result = f(tk);
    (*tk).buffstart -= consumed;
    (*tk).buffcount += consumed;
    if result == TERMKEY_RES_KEY {
        *nbytep += consumed;
    }
    result
}

unsafe fn peek_csi(
    tk: *mut TermKey,
    intro_len: usize,
    key: *mut TermKeyKey,
    force: c_int,
    nbytep: *mut size_t,
) -> TermKeyResult {
    let bytes = buffered(tk);
    let Some(seq) = csi::parse(bytes, intro_len) else {
        if force == 0 {
            return TERMKEY_RES_AGAIN;
        }
        // Out of patience: the introducer was just Alt-[ after all.
        emit_codepoint(tk, CSI_7BIT as c_int, key);
        (*key).modifiers |= TERMKEY_KEYMOD_ALT as c_int;
        *nbytep = intro_len;
        return TERMKEY_RES_KEY;
    };
    // `CSI M` with fewer than three parameters is the X10 mouse protocol, whose
    // three payload bytes follow the sequence rather than sitting inside it.
    if seq.command == b'M' as u32 && seq.nparams < 3 {
        return with_buffer_advanced(tk, seq.len, nbytep, |tk| peekkey_mouse(tk, key, nbytep));
    }

    let mut params: Params = [None; CSI_MAX_PARAMS];
    for (slot, param) in params.iter_mut().zip(seq.params.iter()) {
        *slot = param.map(|(start, end)| &bytes[start..end]);
    }
    let result = dispatch(tk, key, &seq, &params);
    if result == TERMKEY_RES_NONE {
        // Nothing here recognises it. Report it whole, consuming only the
        // introducer, and let the consumer re-parse the rest with
        // `termkey_interpret_csi` before the next `peekkey` discards it.
        (*key).type_0 = TERMKEY_TYPE_UNKNOWN_CSI;
        (*key).code.number = seq.command as c_int;
        (*key).modifiers = 0;
        (*tk).hightide = seq.len - intro_len;
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
unsafe fn dispatch(
    tk: *mut TermKey,
    key: *mut TermKeyKey,
    seq: &CsiSeq,
    params: &Params,
) -> TermKeyResult {
    match (seq.command & 0xff) as u8 {
        b'u' => handle_csi_u(tk, key, seq.command, params),
        b'M' | b'm' => handle_csi_mouse(&mut *key, seq.command, seq.nparams, params),
        b'R' => handle_csi_position(&mut *key, seq.command, seq.nparams, params),
        b'y' => handle_csi_mode(&mut *key, seq.command, seq.nparams, params),
        b'~' => handle_csi_func(tk, key, seq.nparams, params),
        // `csi::parse` only ever stops on a byte in the final-byte range, so
        // the index is in bounds.
        final_byte
            if CSI_SS3[(final_byte - CSI_FINAL_BASE) as usize].sym != TERMKEY_SYM_UNKNOWN =>
        {
            handle_csi_ss3(&mut *key, seq.command, params)
        }
        _ => TERMKEY_RES_NONE,
    }
}

unsafe fn peek_ss3(
    tk: *mut TermKey,
    intro_len: usize,
    key: *mut TermKeyKey,
    force: c_int,
    nbytep: *mut size_t,
) -> TermKeyResult {
    let bytes = buffered(tk);
    if bytes.len() < intro_len + 1 {
        if force == 0 {
            return TERMKEY_RES_AGAIN;
        }
        // Out of patience: the introducer was just Alt-O after all.
        emit_codepoint(tk, SS3_7BIT as c_int, key);
        (*key).modifiers |= TERMKEY_KEYMOD_ALT as c_int;
        *nbytep = bytes.len();
        return TERMKEY_RES_KEY;
    }
    let command = bytes[intro_len];
    if !(0x40..0x80).contains(&command) {
        return TERMKEY_RES_NONE;
    }
    let slot = (command - CSI_FINAL_BASE) as usize;
    let mut info = CSI_SS3[slot];
    if info.sym == TERMKEY_SYM_UNKNOWN {
        if (*tk).flags & TERMKEY_FLAG_CONVERTKP as c_int != 0 && SS3_KEYPAD_ALT[slot] != 0 {
            // The consumer wants keypad keys as the characters they stand for.
            (*key).type_0 = TERMKEY_TYPE_UNICODE;
            (*key).code.codepoint = SS3_KEYPAD_ALT[slot] as u8 as c_int;
            (*key).modifiers = 0;
            (*key).utf8[0] = (*key).code.codepoint as c_char;
            (*key).utf8[1] = 0;
            *nbytep = intro_len + 1;
            return TERMKEY_RES_KEY;
        }
        info = SS3[slot];
    }
    if info.sym == TERMKEY_SYM_UNKNOWN {
        return TERMKEY_RES_NONE;
    }
    (*key).type_0 = info.type_0;
    (*key).code.sym = info.sym;
    (*key).modifiers = info.modifier_set;
    *nbytep = intro_len + 1;
    TERMKEY_RES_KEY
}

/// A control string — DCS, OSC or APC — whose payload runs to a BEL or a string
/// terminator. The payload is kept aside for `termkey_interpret_string`, and the
/// key carries a serial number so a stale key cannot read a newer payload.
unsafe fn peek_control_string(
    tk: *mut TermKey,
    csi: *mut TermKeyCsi,
    intro_len: usize,
    key: *mut TermKeyKey,
    nbytep: *mut size_t,
) -> TermKeyResult {
    let bytes = buffered(tk);
    let Some(end) = (intro_len..bytes.len()).find(|&i| {
        bytes[i] == BEL
            || bytes[i] == STRING_TERMINATOR_8BIT
            || (bytes[i] == ESC && bytes.get(i + 1) == Some(&b'\\'))
    }) else {
        return TERMKEY_RES_AGAIN;
    };
    // A two-byte `ESC \` terminator takes one more byte than a one-byte one.
    *nbytep = end + 1 + usize::from(bytes[end] == ESC);

    let payload = &bytes[intro_len..end];
    if !(*csi).saved_string.is_null() {
        xfree((*csi).saved_string as *mut c_void);
    }
    let saved = xmalloc(payload.len() + 1) as *mut u8;
    core::ptr::copy_nonoverlapping(payload.as_ptr(), saved, payload.len());
    *saved.add(payload.len()) = 0;
    (*csi).saved_string = saved as *mut c_char;
    (*csi).saved_string_id += 1;

    // The introducer's low five bits distinguish the three string kinds, in
    // both their seven- and eight-bit spellings.
    (*key).type_0 = match bytes[intro_len - 1] & 0x1f {
        0x10 => TERMKEY_TYPE_DCS,
        0x1d => TERMKEY_TYPE_OSC,
        0x1f => TERMKEY_TYPE_APC,
        other => unreachable!("control string introducer {other:#x}"),
    };
    (*key).code.number = (*csi).saved_string_id;
    (*key).modifiers = 0;
    TERMKEY_RES_KEY
}

pub unsafe fn peek_key(
    tk: *mut TermKey,
    csi: *mut TermKeyCsi,
    key: *mut TermKeyKey,
    force: c_int,
    nbytep: *mut size_t,
) -> TermKeyResult {
    if (*tk).buffcount == 0 {
        return TERMKEY_RES_NONE;
    }
    let bytes = buffered(tk);
    match bytes[0] {
        ESC => match bytes.get(1) {
            // Not enough to tell yet. Upstream reports NONE rather than AGAIN
            // and leaves the waiting to `peekkey_simple`, which handles a lone
            // escape as Alt-<next key>.
            None => TERMKEY_RES_NONE,
            Some(&SS3_7BIT) => peek_ss3(tk, 2, key, force, nbytep),
            Some(&CSI_7BIT) => peek_csi(tk, 2, key, force, nbytep),
            Some(&DCS_7BIT | &OSC_7BIT | &APC_7BIT) => peek_control_string(tk, csi, 2, key, nbytep),
            _ => TERMKEY_RES_NONE,
        },
        SS3_8BIT => peek_ss3(tk, 1, key, force, nbytep),
        CSI_8BIT => peek_csi(tk, 1, key, force, nbytep),
        DCS_8BIT | OSC_8BIT => peek_control_string(tk, csi, 1, key, nbytep),
        _ => TERMKEY_RES_NONE,
    }
}
