//! libtermkey's core: the input buffer, the two drivers, and the translation
//! of a byte run into a key.
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
use crate::tui::terminfo::entry::TerminfoEntry;
use crate::tui::termkey::csi::{self, CSI_MAX_PARAMS};
use crate::tui::termkey::driver_csi::CsiDriver;
use crate::tui::termkey::driver_ti::TerminfoDriver;
use crate::tui::termkey::format::{self, KeyBody};
use crate::tui::termkey::types::{
    KeyCode, TermKeyEvent, TermKeyFormat, TermKeyKey, TermKeyMouseEvent, TermKeyResult, TermKeySym,
    TermKeyType,
};
use crate::tui::termkey::utf8::{self, Decoded, UNICODE_INVALID};
use crate::tui::termkey::{driver_csi, driver_ti, report};
use core::ffi::{CStr, c_int};

pub const TERMKEY_EVENT_UNKNOWN: TermKeyEvent = 0;
pub const TERMKEY_EVENT_PRESS: TermKeyEvent = 1;
pub const TERMKEY_EVENT_REPEAT: TermKeyEvent = 2;
pub const TERMKEY_EVENT_RELEASE: TermKeyEvent = 3;

pub const TERMKEY_SYM_UNKNOWN: TermKeySym = -1;
pub const TERMKEY_SYM_NONE: TermKeySym = 0;
pub const TERMKEY_SYM_BACKSPACE: TermKeySym = 1;
pub const TERMKEY_SYM_TAB: TermKeySym = 2;
pub const TERMKEY_SYM_ENTER: TermKeySym = 3;
pub const TERMKEY_SYM_ESCAPE: TermKeySym = 4;
pub const TERMKEY_SYM_SPACE: TermKeySym = 5;
pub const TERMKEY_SYM_DEL: TermKeySym = 6;
pub const TERMKEY_SYM_UP: TermKeySym = 7;
pub const TERMKEY_SYM_DOWN: TermKeySym = 8;
pub const TERMKEY_SYM_LEFT: TermKeySym = 9;
pub const TERMKEY_SYM_RIGHT: TermKeySym = 10;
pub const TERMKEY_SYM_BEGIN: TermKeySym = 11;
pub const TERMKEY_SYM_FIND: TermKeySym = 12;
pub const TERMKEY_SYM_INSERT: TermKeySym = 13;
pub const TERMKEY_SYM_DELETE: TermKeySym = 14;
pub const TERMKEY_SYM_SELECT: TermKeySym = 15;
pub const TERMKEY_SYM_PAGEUP: TermKeySym = 16;
pub const TERMKEY_SYM_PAGEDOWN: TermKeySym = 17;
pub const TERMKEY_SYM_HOME: TermKeySym = 18;
pub const TERMKEY_SYM_END: TermKeySym = 19;
pub const TERMKEY_SYM_CLEAR: TermKeySym = 21;
pub const TERMKEY_SYM_SUSPEND: TermKeySym = 40;
pub const TERMKEY_SYM_UNDO: TermKeySym = 41;
pub const TERMKEY_SYM_KP0: TermKeySym = 42;
pub const TERMKEY_SYM_KP1: TermKeySym = 43;
pub const TERMKEY_SYM_KP2: TermKeySym = 44;
pub const TERMKEY_SYM_KP3: TermKeySym = 45;
pub const TERMKEY_SYM_KP4: TermKeySym = 46;
pub const TERMKEY_SYM_KP5: TermKeySym = 47;
pub const TERMKEY_SYM_KP6: TermKeySym = 48;
pub const TERMKEY_SYM_KP7: TermKeySym = 49;
pub const TERMKEY_SYM_KP8: TermKeySym = 50;
pub const TERMKEY_SYM_KP9: TermKeySym = 51;
pub const TERMKEY_SYM_KPENTER: TermKeySym = 52;
pub const TERMKEY_SYM_KPPLUS: TermKeySym = 53;
pub const TERMKEY_SYM_KPMINUS: TermKeySym = 54;
pub const TERMKEY_SYM_KPMULT: TermKeySym = 55;
pub const TERMKEY_SYM_KPDIV: TermKeySym = 56;
pub const TERMKEY_SYM_KPCOMMA: TermKeySym = 57;
pub const TERMKEY_SYM_KPPERIOD: TermKeySym = 58;
pub const TERMKEY_SYM_KPEQUALS: TermKeySym = 59;
pub const TERMKEY_TYPE_UNKNOWN_CSI: TermKeyType = -1;
pub const TERMKEY_TYPE_UNICODE: TermKeyType = 0;
pub const TERMKEY_TYPE_FUNCTION: TermKeyType = 1;
pub const TERMKEY_TYPE_KEYSYM: TermKeyType = 2;
pub const TERMKEY_TYPE_MOUSE: TermKeyType = 3;
pub const TERMKEY_TYPE_POSITION: TermKeyType = 4;
pub const TERMKEY_TYPE_MODEREPORT: TermKeyType = 5;
pub const TERMKEY_TYPE_DCS: TermKeyType = 6;
pub const TERMKEY_TYPE_OSC: TermKeyType = 7;
pub const TERMKEY_TYPE_APC: TermKeyType = 8;

pub const TERMKEY_RES_NONE: TermKeyResult = 0;
pub const TERMKEY_RES_KEY: TermKeyResult = 1;
pub const TERMKEY_RES_EOF: TermKeyResult = 2;
pub const TERMKEY_RES_AGAIN: TermKeyResult = 3;
pub const TERMKEY_RES_ERROR: TermKeyResult = 4;

pub const TERMKEY_MOUSE_UNKNOWN: TermKeyMouseEvent = 0;
pub const TERMKEY_MOUSE_PRESS: TermKeyMouseEvent = 1;
pub const TERMKEY_MOUSE_DRAG: TermKeyMouseEvent = 2;
pub const TERMKEY_MOUSE_RELEASE: TermKeyMouseEvent = 3;

pub type TermKeyModifier = ::core::ffi::c_int;
pub const TERMKEY_KEYMOD_SHIFT: TermKeyModifier = 1;
pub const TERMKEY_KEYMOD_ALT: TermKeyModifier = 2;
pub const TERMKEY_KEYMOD_CTRL: TermKeyModifier = 4;

pub type TermKeyFlag = ::core::ffi::c_int;
pub const TERMKEY_FLAG_NOINTERPRET: TermKeyFlag = 1;
pub const TERMKEY_FLAG_CONVERTKP: TermKeyFlag = 2;
pub const TERMKEY_FLAG_UTF8: TermKeyFlag = 8;
pub const TERMKEY_FLAG_SPACESYMBOL: TermKeyFlag = 32;
pub const TERMKEY_FLAG_NOSTART: TermKeyFlag = 256;
pub const TERMKEY_FLAG_KEEPC0: TermKeyFlag = 512;

pub type TermKeyCanon = ::core::ffi::c_int;
pub const TERMKEY_CANON_SPACESYMBOL: TermKeyCanon = 1;
pub const TERMKEY_CANON_DELBS: TermKeyCanon = 2;

pub const TERMKEY_FORMAT_LONGMOD: TermKeyFormat = 1;
pub const TERMKEY_FORMAT_CARETCTRL: TermKeyFormat = 2;
pub const TERMKEY_FORMAT_ALTISMETA: TermKeyFormat = 4;
pub const TERMKEY_FORMAT_WRAPBRACKET: TermKeyFormat = 8;
pub const TERMKEY_FORMAT_SPACEMOD: TermKeyFormat = 16;
pub const TERMKEY_FORMAT_LOWERMOD: TermKeyFormat = 32;
pub const TERMKEY_FORMAT_LOWERSPACE: TermKeyFormat = 64;
pub const TERMKEY_FORMAT_MOUSE_POS: TermKeyFormat = 256;

/// How much input is held before [`TermKey::push_bytes`] starts refusing it.
const TERMKEY_DEFAULT_BUFFER_SIZE: usize = 256;

/// The symbol each C0 control byte stands for, or `TERMKEY_SYM_NONE` where it
/// has none and the Ctrl-letter reading applies instead.
///
/// Upstream registered these three at construction time into a per-`TermKey`
/// array, with room for a modifier mask and set that every registration left at
/// zero and nothing else ever wrote.
static C0_SYMS: [TermKeySym; 32] = {
    let mut table = [TERMKEY_SYM_NONE; 32];
    table[0x09] = TERMKEY_SYM_TAB;
    table[0x0d] = TERMKEY_SYM_ENTER;
    table[0x1b] = TERMKEY_SYM_ESCAPE;
    table
};

/// nvim's say over a key capability the terminfo driver loads: given the
/// capability's name and what the terminal's description holds for it, the
/// sequence to register instead, or `None` for none.
pub type TerminfoHook = Box<dyn Fn(&CStr, Option<&[u8]>) -> Option<Vec<u8>>>;

/// A key reader: bytes go in through [`TermKey::push_bytes`], keys come out
/// of [`TermKey::getkey`].
pub struct TermKey {
    pub flags: c_int,
    pub canonflags: c_int,
    /// The input buffer, its whole length usable.
    buffer: Vec<u8>,
    /// Where the unread input starts, and how much of it there is. The pair
    /// walks forward through `buffer` until a read compacts it.
    pub(super) buffstart: usize,
    pub(super) buffcount: usize,
    /// Bytes of an unrecognised control sequence held back so the consumer can
    /// re-read them, and discarded at the start of the next read.
    pub(super) hightide: usize,
    ti_getstr_hook: Option<TerminfoHook>,
    pub is_started: bool,
    /// The terminfo driver's state.
    pub(super) ti: TerminfoDriver,
    /// The CSI driver's state.
    pub(super) csi: CsiDriver,
}

/// An unrecognised control sequence, re-read: its parameters (`None` for an
/// omitted one) and its packed command -- the final byte, with the
/// private-use introducer in bits 8-15 and the intermediate byte in bits
/// 16-23.
pub struct UnknownCsi<'a> {
    params: [Option<&'a [u8]>; CSI_MAX_PARAMS],
    nparams: usize,
    pub command: u32,
}

impl<'a> UnknownCsi<'a> {
    /// The parameters the sequence carried.
    pub fn params(&self) -> &[Option<&'a [u8]>] {
        &self.params[..self.nparams]
    }
}

/// The value of a CSI parameter, ignoring any sub-parameters. An omitted
/// parameter reads as -1.
pub fn csi_param_value(param: Option<&[u8]>) -> c_int {
    csi::param_groups(param).0
}

impl TermKey {
    /// A key reader. `term` is the terminal's description, `None` when
    /// nothing is known about it; it is read when the reader starts, which
    /// is now unless `flags` holds `TERMKEY_FLAG_NOSTART`.
    pub fn new(term: Option<&TerminfoEntry>, flags: c_int) -> Self {
        let mut tk = Self {
            flags: 0,
            canonflags: 0,
            buffer: vec![0; TERMKEY_DEFAULT_BUFFER_SIZE],
            buffstart: 0,
            buffcount: 0,
            hightide: 0,
            ti_getstr_hook: None,
            is_started: false,
            ti: TerminfoDriver::default(),
            csi: CsiDriver::default(),
        };
        tk.set_flags(flags);
        if flags & TERMKEY_FLAG_NOSTART == 0 {
            tk.start(term);
        }
        tk
    }

    /// Install nvim's override for terminfo capability lookups, so it can
    /// supply key sequences the terminal's description does not name. It is
    /// consulted when the reader starts.
    pub fn set_terminfo_hook(&mut self, hook: TerminfoHook) {
        self.ti_getstr_hook = Some(hook);
    }

    /// The installed terminfo hook, if any.
    pub(super) fn terminfo_hook(&self) -> Option<&TerminfoHook> {
        self.ti_getstr_hook.as_ref()
    }

    /// Begin reading keys, building the terminfo driver's key table from
    /// `term` the first time. Upstream also put the terminal into raw mode
    /// here and restored it on stop, but that was guarded on a file
    /// descriptor this tree never gives it -- nvim owns the terminal and
    /// feeds bytes in by hand.
    pub fn start(&mut self, term: Option<&TerminfoEntry>) {
        if self.is_started {
            return;
        }
        driver_ti::load_keys(self, term);
        self.is_started = true;
    }

    fn set_flags(&mut self, newflags: c_int) {
        self.flags = newflags;
        // The two spellings of "a space is a symbol, not a character" are
        // kept in step in both directions.
        if self.flags & TERMKEY_FLAG_SPACESYMBOL != 0 {
            self.canonflags |= TERMKEY_CANON_SPACESYMBOL;
        } else {
            self.canonflags &= !(TERMKEY_CANON_SPACESYMBOL);
        }
    }

    pub fn canonflags(&self) -> c_int {
        self.canonflags
    }

    pub fn set_canonflags(&mut self, flags: c_int) {
        self.canonflags = flags;
        if self.canonflags & TERMKEY_CANON_SPACESYMBOL != 0 {
            self.flags |= TERMKEY_FLAG_SPACESYMBOL;
        } else {
            self.flags &= !(TERMKEY_FLAG_SPACESYMBOL);
        }
    }

    /// How many bytes the input buffer holds.
    pub fn buffer_size(&self) -> usize {
        self.buffer.len()
    }

    /// Resize the input buffer, keeping what is in it. Shrinking it below
    /// what is buffered is the caller's mistake, as upstream's `realloc` was.
    pub fn set_buffer_size(&mut self, size: usize) {
        self.buffer.resize(size, 0);
    }

    /// How much more input the buffer can take.
    pub fn buffer_remaining(&self) -> usize {
        self.buffer.len() - self.buffcount
    }

    /// The input that has been pushed in and not yet read out.
    pub(super) fn buffered(&self) -> &[u8] {
        &self.buffer[self.buffstart..self.buffstart + self.buffcount]
    }

    /// Consume `count` bytes of input.
    fn eat_bytes(&mut self, count: usize) {
        if count >= self.buffcount {
            self.buffstart = 0;
            self.buffcount = 0;
        } else {
            self.buffstart += count;
            self.buffcount -= count;
        }
    }

    /// Hand more input to the reader. Returns how much of it was taken, or
    /// `None` when the buffer was already full.
    pub fn push_bytes(&mut self, bytes: &[u8]) -> Option<usize> {
        if self.buffstart != 0 {
            // Slide what is left of the previous read back to the front.
            let (start, count) = (self.buffstart, self.buffcount);
            self.buffer.copy_within(start..start + count, 0);
            self.buffstart = 0;
        }
        if self.buffcount >= self.buffer.len() {
            return None;
        }
        let taken = bytes.len().min(self.buffer.len() - self.buffcount);
        let count = self.buffcount;
        self.buffer[count..count + taken].copy_from_slice(&bytes[..taken]);
        self.buffcount += taken;
        Some(taken)
    }

    /// Read the next key into `key`, consuming it. `TERMKEY_RES_AGAIN` means
    /// a sequence could still grow; `key` is then filled in with what it
    /// spells so far, so a caller that gives up waiting has something to
    /// report, and the bytes stay in the buffer.
    pub fn getkey(&mut self, key: &mut TermKeyKey) -> TermKeyResult {
        let mut nbytes = 0;
        let ret = peekkey(self, key, false, &mut nbytes);
        if ret == TERMKEY_RES_KEY {
            self.eat_bytes(nbytes);
        }
        if ret == TERMKEY_RES_AGAIN {
            peekkey(self, key, true, &mut nbytes);
        }
        ret
    }

    /// [`TermKey::getkey`], taking a sequence that could still grow as
    /// complete.
    pub fn getkey_force(&mut self, key: &mut TermKeyKey) -> TermKeyResult {
        let mut nbytes = 0;
        let ret = peekkey(self, key, true, &mut nbytes);
        if ret == TERMKEY_RES_KEY {
            self.eat_bytes(nbytes);
        }
        ret
    }

    /// The payload of the last DCS, OSC or APC string, if `key` is still the
    /// one that reported it: up to its first NUL, as upstream's C string
    /// ended there. Borrowed from the reader, so it is read before the next
    /// control string replaces it.
    pub fn interpret_string(&self, key: &TermKeyKey) -> Option<&[u8]> {
        let kind = key.kind;
        if kind != TERMKEY_TYPE_DCS && kind != TERMKEY_TYPE_OSC && kind != TERMKEY_TYPE_APC {
            return None;
        }
        // Each string gets a serial number, so a key held past the next one
        // cannot read a payload that is no longer its own.
        if self.csi.saved_string_id != key.code.number() {
            return None;
        }
        let saved = &self.csi.saved_string;
        Some(saved.split(|&byte| byte == 0).next().unwrap_or(saved))
    }

    /// Re-parse the control sequence a `TERMKEY_TYPE_UNKNOWN_CSI` key stands
    /// for, so the consumer can decide what to make of it.
    ///
    /// The sequence is still sitting at the head of the buffer: `peekkey`
    /// consumed only its introducer and parked the rest under `hightide`,
    /// which it discards on the next read. `None` once it has, or for any
    /// other key.
    pub fn interpret_csi(&self, key: &TermKeyKey) -> Option<UnknownCsi<'_>> {
        if self.hightide == 0 || key.kind != TERMKEY_TYPE_UNKNOWN_CSI {
            return None;
        }
        let bytes = self.buffered();
        let seq = csi::parse(bytes, 0)?;
        let mut params = [None; CSI_MAX_PARAMS];
        for (slot, param) in params.iter_mut().zip(&seq.params[..seq.nparams]) {
            *slot = param.map(|(start, end)| &bytes[start..end]);
        }
        Some(UnknownCsi {
            params,
            nparams: seq.nparams,
            command: seq.command,
        })
    }
}

/// A mouse key's (event, button, line, column), or `None` for any other key.
/// A key whose report names no event has button 0.
pub fn interpret_mouse(key: &TermKeyKey) -> Option<(TermKeyMouseEvent, c_int, c_int, c_int)> {
    if key.kind != TERMKEY_TYPE_MOUSE {
        return None;
    }
    let payload = key.code.payload();
    let (line, col) = report::unpack_position(&payload);
    let (event, button) = report::decode_mouse(&payload);
    Some((event, button, line, col))
}

/// A mode report's (introducer, mode, value), or `None` for any other key.
pub fn interpret_modereport(key: &TermKeyKey) -> Option<(c_int, c_int, c_int)> {
    (key.kind == TERMKEY_TYPE_MODEREPORT).then(|| report::unpack_mode(&key.code.payload()))
}

/// A key's `utf8` field, up to its terminator.
///
/// Bounded, unlike upstream's `%s`: a key the consumer built itself need not
/// have terminated the field.
fn utf8_bytes(field: &[u8; 7]) -> &[u8] {
    let len = field
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(field.len());
    &field[..len]
}

/// Write a codepoint's UTF-8 into a key's `utf8` field, which is exactly wide
/// enough for the longest encoding plus its terminator.
fn fill_utf8(codepoint: c_int, out: &mut [u8; 7]) {
    let (bytes, len) = utf8::encode(codepoint);
    out[..len].copy_from_slice(&bytes[..len]);
    out[len] = 0;
}

/// Turn a codepoint into a key, applying the C0 and C1 readings.
pub fn emit_codepoint(tk: &TermKey, codepoint: c_int, key: &mut TermKeyKey) {
    let flags = tk.flags;
    let interpret = flags & TERMKEY_FLAG_NOINTERPRET == 0;
    if codepoint == 0 {
        // NUL is Ctrl-Space, which has no character of its own.
        key.kind = TERMKEY_TYPE_KEYSYM;
        key.code = KeyCode::scalar(TERMKEY_SYM_SPACE);
        key.modifiers = TERMKEY_KEYMOD_CTRL;
    } else if codepoint < 0x20 && flags & TERMKEY_FLAG_KEEPC0 == 0 {
        let sym = if interpret {
            C0_SYMS[as_index(codepoint)]
        } else {
            TERMKEY_SYM_NONE
        };
        if sym == TERMKEY_SYM_NONE {
            // Ctrl-letter: C0 codes map onto '@' through '_', but the
            // lower-case letter is the friendlier name for A-Z.
            key.kind = TERMKEY_TYPE_UNICODE;
            let base = if (c_int::from(b'A')..=c_int::from(b'Z')).contains(&(codepoint + 0x40)) {
                0x60
            } else {
                0x40
            };
            key.code = KeyCode::scalar(codepoint + base);
            key.modifiers = TERMKEY_KEYMOD_CTRL;
        } else {
            key.kind = TERMKEY_TYPE_KEYSYM;
            key.code = KeyCode::scalar(sym);
            key.modifiers = 0;
        }
    } else if codepoint == 0x7f && interpret {
        key.kind = TERMKEY_TYPE_KEYSYM;
        key.code = KeyCode::scalar(TERMKEY_SYM_DEL);
        key.modifiers = 0;
    } else if (0x80..0xa0).contains(&codepoint) {
        // The C1 controls are Alt-Ctrl-letter.
        key.kind = TERMKEY_TYPE_UNICODE;
        key.code = KeyCode::scalar(codepoint - 0x40);
        key.modifiers = (TERMKEY_KEYMOD_CTRL | TERMKEY_KEYMOD_ALT) as c_int;
    } else {
        key.kind = TERMKEY_TYPE_UNICODE;
        key.code = KeyCode::scalar(codepoint);
        key.modifiers = 0;
    }
    canonicalise(tk, key);
    if key.kind == TERMKEY_TYPE_UNICODE {
        fill_utf8(key.code.codepoint(), &mut key.utf8);
    }
}

/// Apply the consumer's preferences about which of two equivalent spellings a
/// key gets: space as a symbol or a character, and DEL as backspace.
fn canonicalise(tk: &TermKey, key: &mut TermKeyKey) {
    let flags = tk.canonflags;
    if flags & TERMKEY_CANON_SPACESYMBOL != 0 {
        if key.kind == TERMKEY_TYPE_UNICODE && key.code.codepoint() == 0x20 {
            key.kind = TERMKEY_TYPE_KEYSYM;
            key.code = KeyCode::scalar(TERMKEY_SYM_SPACE);
        }
    } else if key.kind == TERMKEY_TYPE_KEYSYM && key.code.sym() == TERMKEY_SYM_SPACE {
        key.kind = TERMKEY_TYPE_UNICODE;
        key.code = KeyCode::scalar(0x20);
        fill_utf8(0x20, &mut key.utf8);
    }
    if flags & TERMKEY_CANON_DELBS != 0
        && key.kind == TERMKEY_TYPE_KEYSYM
        && key.code.sym() == TERMKEY_SYM_DEL
    {
        key.code = KeyCode::scalar(TERMKEY_SYM_BACKSPACE);
    }
}

/// Read the next key without consuming it. `force` means "decide now": a
/// sequence that could still grow is taken as complete instead of waiting.
fn peekkey(
    tk: &mut TermKey,
    key: &mut TermKeyKey,
    force: bool,
    nbytep: &mut usize,
) -> TermKeyResult {
    if !tk.is_started {
        return TERMKEY_RES_ERROR;
    }
    key.event = TERMKEY_EVENT_PRESS;
    if tk.hightide != 0 {
        // The tail of an unrecognised control sequence, held back so the
        // consumer could re-read it with `interpret_csi`.
        tk.buffstart += tk.hightide;
        tk.buffcount -= tk.hightide;
        tk.hightide = 0;
    }
    let mut again = false;
    // The terminfo driver has first refusal (its sequences come from the
    // terminal's own description), then the CSI driver's generic parsing.
    for probe in 0..2 {
        let ret = if probe == 0 {
            driver_ti::peek_key(tk, key, force, nbytep)
        } else {
            driver_csi::peek_key(tk, key, force, nbytep)
        };
        match ret {
            TERMKEY_RES_KEY => {
                // Reclaim the front half of the buffer once reads have walked
                // past its midpoint, so a long run does not push buffstart off
                // the end. Only worth doing when a key was actually consumed.
                let halfsize = tk.buffer.len() / 2;
                if tk.buffstart > halfsize {
                    tk.buffer.copy_within(halfsize..halfsize * 2, 0);
                    tk.buffstart -= halfsize;
                }
                return ret;
            }
            TERMKEY_RES_EOF | TERMKEY_RES_ERROR => return ret,
            TERMKEY_RES_AGAIN => again |= !force,
            _ => {}
        }
    }
    if again {
        return TERMKEY_RES_AGAIN;
    }
    peekkey_simple(tk, key, force, nbytep)
}

/// Whatever neither driver claimed: a plain character, or an escape prefix that
/// makes the key after it an Alt- key (unless that key is a repeat or release).
fn peekkey_simple(
    tk: &mut TermKey,
    key: &mut TermKeyKey,
    force: bool,
    nbytep: &mut usize,
) -> TermKeyResult {
    if tk.buffcount == 0 {
        return TERMKEY_RES_NONE;
    }
    let first = tk.buffered()[0];
    if first == 0x1b {
        if tk.buffcount == 1 {
            if !force {
                return TERMKEY_RES_AGAIN;
            }
            // Nothing followed it, so it was the escape key itself.
            emit_codepoint(tk, c_int::from(first), key);
            *nbytep = 1;
            return TERMKEY_RES_KEY;
        }
        // Read what follows as its own key, then add the Alt modifier.
        tk.buffstart += 1;
        tk.buffcount -= 1;
        let result = peekkey(tk, key, force, nbytep);
        tk.buffstart -= 1;
        tk.buffcount += 1;
        if result == TERMKEY_RES_KEY {
            if key.event == TERMKEY_EVENT_REPEAT || key.event == TERMKEY_EVENT_RELEASE {
                // A repeat or release carries its modifiers in its own
                // escape sequence, so the escape before it cannot be its Alt:
                // it was the escape key, pressed on its own.
                key.event = TERMKEY_EVENT_PRESS;
                emit_codepoint(tk, c_int::from(first), key);
                *nbytep = 1;
                return TERMKEY_RES_KEY;
            }
            key.modifiers |= TERMKEY_KEYMOD_ALT;
            *nbytep += 1;
        }
        return result;
    }
    if first < 0xa0 {
        emit_codepoint(tk, c_int::from(first), key);
        *nbytep = 1;
        return TERMKEY_RES_KEY;
    }
    if tk.flags & TERMKEY_FLAG_UTF8 != 0 {
        let bytes = tk.buffered();
        let (codepoint, len, result) = match utf8::decode(bytes) {
            Decoded::Char { codepoint, len } => (codepoint, len, TERMKEY_RES_KEY),
            Decoded::Incomplete if force => {
                // Out of patience: take what there is as one bad character.
                (UNICODE_INVALID, bytes.len(), TERMKEY_RES_KEY)
            }
            Decoded::Incomplete => (UNICODE_INVALID, 0, TERMKEY_RES_AGAIN),
        };
        if result == TERMKEY_RES_KEY {
            *nbytep = len;
        }
        // Upstream fills the key in even when it is about to report AGAIN, and
        // the caller re-reads with force set rather than trusting it.
        key.kind = TERMKEY_TYPE_UNICODE;
        key.modifiers = 0;
        emit_codepoint(tk, codepoint, key);
        return result;
    }
    // No UTF-8: every byte is its own character.
    key.kind = TERMKEY_TYPE_UNICODE;
    key.code = KeyCode::scalar(c_int::from(first));
    key.modifiers = 0;
    key.utf8[0] = first;
    key.utf8[1] = 0;
    *nbytep = 1;
    TERMKEY_RES_KEY
}

/// Decode an X10 mouse report: three bytes of button-and-modifiers, column and
/// line, each offset by a space so they stay printable.
pub fn peekkey_mouse(tk: &TermKey, key: &mut TermKeyKey, nbytep: &mut usize) -> TermKeyResult {
    if tk.buffcount < 3 {
        return TERMKEY_RES_AGAIN;
    }
    key.kind = TERMKEY_TYPE_MOUSE;
    let mut payload: report::Payload = [0; 4];
    for (slot, byte) in payload.iter_mut().zip(&tk.buffered()[..3]) {
        *slot = byte.wrapping_sub(0x20);
    }
    // Bits 2-4 of the button code are the modifiers; lift them out of it.
    key.modifiers = (c_int::from(payload[0]) & 0x1c) >> 2;
    payload[0] &= !0x1c;
    key.code = KeyCode::report(payload);
    *nbytep = 3;
    TERMKEY_RES_KEY
}

/// Render a key as text: what upstream's `termkey_strfkey` wrote, whole.
///
/// Upstream built the text with a series of `snprintf`s into `buffer + pos`
/// with a remaining size of `len - pos`, which wrapped once `pos` passed
/// `len`; rendering into a growable buffer has no such edge.
pub fn strfkey(key: &TermKeyKey, format: TermKeyFormat) -> Vec<u8> {
    // A key whose UTF-8 was never filled in -- one the consumer built itself
    // -- gets it derived from the codepoint.
    let mut encoded = [0u8; 7];
    let utf8: &[u8] = if key.kind == TERMKEY_TYPE_UNICODE {
        if key.utf8[0] == 0 {
            fill_utf8(key.code.codepoint(), &mut encoded);
            utf8_bytes(&encoded)
        } else {
            utf8_bytes(&key.utf8)
        }
    } else {
        b""
    };

    let body = match key.kind {
        TERMKEY_TYPE_UNICODE => KeyBody::Unicode {
            codepoint: key.code.codepoint(),
            utf8,
        },
        TERMKEY_TYPE_KEYSYM => KeyBody::Sym(key.code.sym()),
        TERMKEY_TYPE_FUNCTION => KeyBody::Function(key.code.number()),
        TERMKEY_TYPE_MOUSE => {
            let payload = key.code.payload();
            let (line, col) = report::unpack_position(&payload);
            let (event, button) = report::decode_mouse(&payload);
            KeyBody::Mouse {
                event,
                button,
                line,
                col,
            }
        }
        TERMKEY_TYPE_POSITION => KeyBody::Position,
        TERMKEY_TYPE_MODEREPORT => {
            let (initial, mode, value) = report::unpack_mode(&key.code.payload());
            KeyBody::Mode {
                initial,
                mode,
                value,
            }
        }
        TERMKEY_TYPE_DCS => KeyBody::Dcs,
        TERMKEY_TYPE_OSC => KeyBody::Osc,
        TERMKEY_TYPE_APC => KeyBody::Apc,
        TERMKEY_TYPE_UNKNOWN_CSI => KeyBody::UnknownCsi(key.code.number()),
        _ => KeyBody::Unrecognised,
    };

    format::render(&body, key.modifiers, format)
}
