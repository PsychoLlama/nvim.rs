//! libtermkey's core: the input buffer, the two drivers, and the translation
//! of a byte run into a key.
//!
//! Ported from libtermkey, Copyright (c) 2007-2011 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libtermkey-LICENSE.txt.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::memory::{xfree, xmalloc, xrealloc};
use crate::src::nvim::tui::termkey::driver_csi;
use crate::src::nvim::tui::termkey::driver_ti;
use crate::src::nvim::tui::termkey::format::{self, KeyBody};
use crate::src::nvim::tui::termkey::keynames;
use crate::src::nvim::tui::termkey::report;
use crate::src::nvim::tui::termkey::utf8::{self, Decoded, UNICODE_INVALID};
use crate::src::nvim::types::{
    TermKey, TermKey_Terminfo_Getstr_Hook, TermKeyEvent, TermKeyFormat, TermKeyKey,
    TermKeyKey_code, TermKeyMouseEvent, TermKeyResult, TermKeySym, TermKeyType, TerminfoEntry,
    size_t,
};
use core::ffi::{CStr, c_char, c_int, c_uchar, c_void};
use core::ops::{Deref, DerefMut};

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

pub type TermKeyModifier = ::core::ffi::c_uint;
pub const TERMKEY_KEYMOD_SHIFT: TermKeyModifier = 1;
pub const TERMKEY_KEYMOD_ALT: TermKeyModifier = 2;
pub const TERMKEY_KEYMOD_CTRL: TermKeyModifier = 4;

pub type TermKeyFlag = ::core::ffi::c_uint;
pub const TERMKEY_FLAG_NOINTERPRET: TermKeyFlag = 1;
pub const TERMKEY_FLAG_CONVERTKP: TermKeyFlag = 2;
pub const TERMKEY_FLAG_UTF8: TermKeyFlag = 8;
pub const TERMKEY_FLAG_SPACESYMBOL: TermKeyFlag = 32;
pub const TERMKEY_FLAG_NOSTART: TermKeyFlag = 256;
pub const TERMKEY_FLAG_KEEPC0: TermKeyFlag = 512;

pub type TermKeyCanon = ::core::ffi::c_uint;
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

/// How much input is held before `termkey_push_bytes` starts refusing it.
const TERMKEY_DEFAULT_BUFFER_SIZE: size_t = 256;

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

/// A live key reader — the C ABI's `TermKey *`, with the promise that reaching
/// it is sound.
///
/// Constructing one is the unsafe operation; everything after it is checked
/// code, which is why this file spends a line per entry point rather than one
/// per `(*tk)`.
#[derive(Copy, Clone)]
pub struct Tk(*mut TermKey);

impl Tk {
    /// # Safety
    ///
    /// `tk` must point at a live `TermKey` that outlives the wrapper and that
    /// nothing else reads or writes meanwhile, whose `buffer` addresses
    /// `buffsize` writable bytes, and whose `buffstart + buffcount` stays
    /// inside them — the invariant every function here relies on and restores.
    pub unsafe fn of(tk: *mut TermKey) -> Self {
        Self(tk)
    }

    /// The reader as the C ABI spells it, for the entry points that hand it
    /// back and for the calls that have not been narrowed yet.
    pub fn raw(self) -> *mut TermKey {
        self.0
    }

    /// The input that has been pushed in and not yet read out.
    pub fn buffered(&self) -> &[u8] {
        // SAFETY: `buffstart + buffcount` is inside the buffer, by `Tk::of`.
        unsafe { core::slice::from_raw_parts(self.buffer.add(self.buffstart), self.buffcount) }
    }

    /// The whole buffer, for the two compactions that slide bytes inside it.
    fn whole_buffer(&mut self) -> &mut [u8] {
        // SAFETY: the buffer addresses `buffsize` writable bytes, by `Tk::of`.
        unsafe { core::slice::from_raw_parts_mut(self.buffer, self.buffsize) }
    }

    /// Consume `count` bytes of input.
    fn eat_bytes(&mut self, count: size_t) {
        if count >= self.buffcount {
            self.buffstart = 0;
            self.buffcount = 0;
        } else {
            self.buffstart += count;
            self.buffcount -= count;
        }
    }
}

impl Deref for Tk {
    type Target = TermKey;

    fn deref(&self) -> &TermKey {
        // SAFETY: the reader is live, by `Tk::of`.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Tk {
    fn deref_mut(&mut self) -> &mut TermKey {
        // SAFETY: the reader is live and unaliased, by `Tk::of`.
        unsafe { &mut *self.0 }
    }
}

/// The arms of a key's `code` union.
///
/// The three scalar arms — a character's codepoint, a function key's number
/// and a symbol — are one `c_int` at offset 0, and the report payload covers
/// the same four bytes, so every arm is written whenever any is. Reading the
/// arm a key is not in therefore reads a stale value, never an uninitialised
/// one; which arm is *meaningful* is what `type_0` says.
pub trait KeyCode {
    /// The character a `TERMKEY_TYPE_UNICODE` key stands for.
    fn codepoint(&self) -> c_int;
    /// The number of a `TERMKEY_TYPE_FUNCTION` key, or the serial number of a
    /// control string or an unrecognised control sequence.
    fn number(&self) -> c_int;
    /// The symbol of a `TERMKEY_TYPE_KEYSYM` key.
    fn sym(&self) -> TermKeySym;
    /// The four packed bytes of a mouse, position or mode report.
    fn report(&self) -> report::Payload;
}

impl KeyCode for TermKeyKey {
    fn codepoint(&self) -> c_int {
        // SAFETY: the scalar arms are one `c_int` at offset 0; see the trait.
        unsafe { self.code.codepoint }
    }

    fn number(&self) -> c_int {
        self.codepoint()
    }

    fn sym(&self) -> TermKeySym {
        self.codepoint()
    }

    fn report(&self) -> report::Payload {
        // SAFETY: the payload covers the same four bytes; see the trait.
        unsafe { self.code.mouse }
    }
}

/// A key's `utf8` field, up to its terminator.
///
/// Bounded, unlike upstream's `%s`: a key the consumer built itself need not
/// have terminated the field.
fn utf8_bytes(field: &[c_char; 7]) -> &[u8] {
    let len = field
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(field.len());
    // SAFETY: `c_char` and `u8` have the same size and alignment and every bit
    // pattern is valid for both, so the prefix reads as bytes unchanged.
    unsafe { core::slice::from_raw_parts(field.as_ptr() as *const u8, len) }
}

/// Create a key reader. `term` is the terminal's description, which may be null
/// when nothing is known about it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_new_abstract(
    term: *mut TerminfoEntry,
    flags: c_int,
) -> *mut TermKey {
    // SAFETY: xmalloc returns a fresh allocation of the size asked for; it
    // aborts rather than returning null.
    let raw = unsafe { xmalloc(size_of::<TermKey>()) } as *mut TermKey;
    // SAFETY: as above.
    let buffer = unsafe { xmalloc(TERMKEY_DEFAULT_BUFFER_SIZE) } as *mut c_uchar;
    let fresh = TermKey {
        flags: 0,
        canonflags: 0,
        buffer,
        buffstart: 0,
        buffcount: 0,
        buffsize: TERMKEY_DEFAULT_BUFFER_SIZE,
        hightide: 0,
        ti_getstr_hook: None,
        ti_getstr_hook_data: core::ptr::null_mut(),
        is_started: 0,
        ti: driver_ti::new_driver(term),
        csi: driver_csi::new_driver(),
    };
    // SAFETY: `raw` is a fresh allocation of exactly one `TermKey`.
    unsafe { raw.write(fresh) };
    // SAFETY: it now holds a live reader whose buffer addresses
    // `TERMKEY_DEFAULT_BUFFER_SIZE` bytes with nothing pushed into it yet.
    let tk = unsafe { Tk::of(raw) };
    set_flags(tk, flags);
    if flags & TERMKEY_FLAG_NOSTART as c_int == 0 {
        start(tk);
    }
    raw
}

/// Release the reader and everything it owns; the pointer dangles afterwards.
fn free(mut tk: Tk) {
    let buffer = tk.buffer;
    // SAFETY: allocated by `termkey_new_abstract` and freed exactly once — a
    // reader is destroyed once, and the field is cleared here.
    unsafe { xfree(buffer as *mut c_void) };
    tk.buffer = core::ptr::null_mut();
    let (ti, csi, raw) = (tk.ti, tk.csi, tk.raw());
    // SAFETY: the two drivers and the reader itself, allocated with it and
    // freed exactly once. Nothing reads `tk` after this.
    unsafe {
        driver_ti::free_driver(ti);
        driver_csi::free_driver(csi);
        xfree(raw as *mut c_void);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_destroy(tk: *mut TermKey) {
    // SAFETY: the caller's reader, as at every entry point here.
    let tk = unsafe { Tk::of(tk) };
    if tk.is_started != 0 {
        stop(tk);
    }
    free(tk);
}

/// Install nvim's override for terminfo capability lookups, so it can supply
/// key sequences the terminal's description does not name.
pub unsafe fn termkey_hook_terminfo_getstr(
    tk: *mut TermKey,
    hookfn: Option<TermKey_Terminfo_Getstr_Hook>,
    data: *mut c_void,
) {
    // SAFETY: the caller's reader, as at every entry point here.
    let mut tk = unsafe { Tk::of(tk) };
    tk.ti_getstr_hook = hookfn;
    tk.ti_getstr_hook_data = data;
}

/// Begin reading keys. Upstream also put the terminal into raw mode here and
/// restored it on stop, but that was guarded on a file descriptor this tree
/// never gives it — nvim owns the terminal and feeds bytes in by hand.
fn start(mut tk: Tk) -> c_int {
    if tk.is_started != 0 {
        return 1;
    }
    driver_ti::load_keys(tk);
    tk.is_started = 1;
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_start(tk: *mut TermKey) -> c_int {
    // SAFETY: the caller's reader, as at every entry point here.
    start(unsafe { Tk::of(tk) })
}

fn stop(mut tk: Tk) -> c_int {
    tk.is_started = 0;
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_stop(tk: *mut TermKey) -> c_int {
    // SAFETY: the caller's reader, as at every entry point here.
    stop(unsafe { Tk::of(tk) })
}

fn set_flags(mut tk: Tk, newflags: c_int) {
    tk.flags = newflags;
    // The two spellings of "a space is a symbol, not a character" are kept in
    // step in both directions.
    if tk.flags & TERMKEY_FLAG_SPACESYMBOL as c_int != 0 {
        tk.canonflags |= TERMKEY_CANON_SPACESYMBOL as c_int;
    } else {
        tk.canonflags &= !(TERMKEY_CANON_SPACESYMBOL as c_int);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_set_flags(tk: *mut TermKey, newflags: c_int) {
    // SAFETY: the caller's reader, as at every entry point here.
    set_flags(unsafe { Tk::of(tk) }, newflags);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_get_canonflags(tk: *mut TermKey) -> c_int {
    // SAFETY: the caller's reader, as at every entry point here.
    unsafe { Tk::of(tk) }.canonflags
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_set_canonflags(tk: *mut TermKey, flags: c_int) {
    // SAFETY: the caller's reader, as at every entry point here.
    let mut tk = unsafe { Tk::of(tk) };
    tk.canonflags = flags;
    if tk.canonflags & TERMKEY_CANON_SPACESYMBOL as c_int != 0 {
        tk.flags |= TERMKEY_FLAG_SPACESYMBOL as c_int;
    } else {
        tk.flags &= !(TERMKEY_FLAG_SPACESYMBOL as c_int);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_get_buffer_size(tk: *mut TermKey) -> size_t {
    // SAFETY: the caller's reader, as at every entry point here.
    unsafe { Tk::of(tk) }.buffsize
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_set_buffer_size(tk: *mut TermKey, size: size_t) -> c_int {
    // SAFETY: the caller's reader, as at every entry point here.
    let mut tk = unsafe { Tk::of(tk) };
    let buffer = tk.buffer;
    // SAFETY: the buffer this reader allocated, resized in place; xrealloc
    // aborts rather than returning null.
    tk.buffer = unsafe { xrealloc(buffer as *mut c_void, size) } as *mut c_uchar;
    tk.buffsize = size;
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_get_buffer_remaining(tk: *mut TermKey) -> size_t {
    // SAFETY: the caller's reader, as at every entry point here.
    let tk = unsafe { Tk::of(tk) };
    tk.buffsize - tk.buffcount
}

/// Hand more input to the reader. Returns how much of it was taken, or `-1` as
/// a `size_t` when the buffer was already full.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_push_bytes(
    tk: *mut TermKey,
    bytes: *const c_char,
    len: size_t,
) -> size_t {
    // SAFETY: the caller's reader, as at every entry point here.
    let mut tk = unsafe { Tk::of(tk) };
    // SAFETY: the caller's input, `len` bytes of it.
    let bytes = unsafe { core::slice::from_raw_parts(bytes as *const u8, len) };
    if tk.buffstart != 0 {
        // Slide what is left of the previous read back to the front.
        let (start, count) = (tk.buffstart, tk.buffcount);
        tk.whole_buffer().copy_within(start..start + count, 0);
        tk.buffstart = 0;
    }
    if tk.buffcount >= tk.buffsize {
        return -1i32 as size_t;
    }
    let taken = len.min(tk.buffsize - tk.buffcount);
    let count = tk.buffcount;
    tk.whole_buffer()[count..count + taken].copy_from_slice(&bytes[..taken]);
    tk.buffcount += taken;
    taken
}

/// Write a codepoint's UTF-8 into a key's `utf8` field, which is exactly wide
/// enough for the longest encoding plus its terminator.
fn fill_utf8(codepoint: c_int, out: &mut [c_char; 7]) {
    let (bytes, len) = utf8::encode(codepoint);
    for (slot, byte) in out.iter_mut().zip(bytes[..len].iter()) {
        *slot = *byte as c_char;
    }
    out[len] = 0;
}

/// Turn a codepoint into a key, applying the C0 and C1 readings.
pub fn emit_codepoint(tk: Tk, codepoint: c_int, key: &mut TermKeyKey) {
    let flags = tk.flags;
    let interpret = flags & TERMKEY_FLAG_NOINTERPRET as c_int == 0;
    if codepoint == 0 {
        // NUL is Ctrl-Space, which has no character of its own.
        key.type_0 = TERMKEY_TYPE_KEYSYM;
        key.code = TermKeyKey_code {
            sym: TERMKEY_SYM_SPACE,
        };
        key.modifiers = TERMKEY_KEYMOD_CTRL as c_int;
    } else if codepoint < 0x20 && flags & TERMKEY_FLAG_KEEPC0 as c_int == 0 {
        let sym = if interpret {
            C0_SYMS[codepoint as usize]
        } else {
            TERMKEY_SYM_NONE
        };
        if sym == TERMKEY_SYM_NONE {
            // Ctrl-letter: C0 codes map onto '@' through '_', but the
            // lower-case letter is the friendlier name for A-Z.
            key.type_0 = TERMKEY_TYPE_UNICODE;
            let base = if (b'A' as c_int..=b'Z' as c_int).contains(&(codepoint + 0x40)) {
                0x60
            } else {
                0x40
            };
            key.code = TermKeyKey_code {
                codepoint: codepoint + base,
            };
            key.modifiers = TERMKEY_KEYMOD_CTRL as c_int;
        } else {
            key.type_0 = TERMKEY_TYPE_KEYSYM;
            key.code = TermKeyKey_code { sym };
            key.modifiers = 0;
        }
    } else if codepoint == 0x7f && interpret {
        key.type_0 = TERMKEY_TYPE_KEYSYM;
        key.code = TermKeyKey_code {
            sym: TERMKEY_SYM_DEL,
        };
        key.modifiers = 0;
    } else if (0x80..0xa0).contains(&codepoint) {
        // The C1 controls are Alt-Ctrl-letter.
        key.type_0 = TERMKEY_TYPE_UNICODE;
        key.code = TermKeyKey_code {
            codepoint: codepoint - 0x40,
        };
        key.modifiers = (TERMKEY_KEYMOD_CTRL | TERMKEY_KEYMOD_ALT) as c_int;
    } else {
        key.type_0 = TERMKEY_TYPE_UNICODE;
        key.code = TermKeyKey_code { codepoint };
        key.modifiers = 0;
    }
    canonicalise(tk, key);
    if key.type_0 == TERMKEY_TYPE_UNICODE {
        fill_utf8(key.codepoint(), &mut key.utf8);
    }
}

/// Apply the consumer's preferences about which of two equivalent spellings a
/// key gets: space as a symbol or a character, and DEL as backspace.
fn canonicalise(tk: Tk, key: &mut TermKeyKey) {
    let flags = tk.canonflags;
    if flags & TERMKEY_CANON_SPACESYMBOL as c_int != 0 {
        if key.type_0 == TERMKEY_TYPE_UNICODE && key.codepoint() == 0x20 {
            key.type_0 = TERMKEY_TYPE_KEYSYM;
            key.code = TermKeyKey_code {
                sym: TERMKEY_SYM_SPACE,
            };
        }
    } else if key.type_0 == TERMKEY_TYPE_KEYSYM && key.sym() == TERMKEY_SYM_SPACE {
        key.type_0 = TERMKEY_TYPE_UNICODE;
        key.code = TermKeyKey_code { codepoint: 0x20 };
        fill_utf8(0x20, &mut key.utf8);
    }
    if flags & TERMKEY_CANON_DELBS as c_int != 0
        && key.type_0 == TERMKEY_TYPE_KEYSYM
        && key.sym() == TERMKEY_SYM_DEL
    {
        key.code = TermKeyKey_code {
            sym: TERMKEY_SYM_BACKSPACE,
        };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_canonicalise(tk: *mut TermKey, key: *mut TermKeyKey) {
    // SAFETY: the caller's reader and the key it is asking about.
    let (tk, key) = unsafe { (Tk::of(tk), &mut *key) };
    canonicalise(tk, key);
}

/// Read the next key without consuming it. `force` means "decide now": a
/// sequence that could still grow is taken as complete instead of waiting.
fn peekkey(mut tk: Tk, key: &mut TermKeyKey, force: c_int, nbytep: &mut size_t) -> TermKeyResult {
    if tk.is_started == 0 {
        return TERMKEY_RES_ERROR;
    }
    key.event = TERMKEY_EVENT_PRESS;
    if tk.hightide != 0 {
        // The tail of an unrecognised control sequence, held back so the
        // consumer could re-read it with `termkey_interpret_csi`.
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
                let halfsize = tk.buffsize / 2;
                if tk.buffstart > halfsize {
                    tk.whole_buffer().copy_within(halfsize..halfsize * 2, 0);
                    tk.buffstart -= halfsize;
                }
                return ret;
            }
            TERMKEY_RES_EOF | TERMKEY_RES_ERROR => return ret,
            TERMKEY_RES_AGAIN => again |= force == 0,
            _ => {}
        }
    }
    if again {
        return TERMKEY_RES_AGAIN;
    }
    peekkey_simple(tk, key, force, nbytep)
}

/// Whatever neither driver claimed: a plain character, or an escape prefix that
/// makes the key after it an Alt- key.
fn peekkey_simple(
    mut tk: Tk,
    key: &mut TermKeyKey,
    force: c_int,
    nbytep: &mut size_t,
) -> TermKeyResult {
    if tk.buffcount == 0 {
        return TERMKEY_RES_NONE;
    }
    let first = tk.buffered()[0];
    if first == 0x1b {
        if tk.buffcount == 1 {
            if force == 0 {
                return TERMKEY_RES_AGAIN;
            }
            // Nothing followed it, so it was the escape key itself.
            emit_codepoint(tk, first as c_int, key);
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
            key.modifiers |= TERMKEY_KEYMOD_ALT as c_int;
            *nbytep += 1;
        }
        return result;
    }
    if first < 0xa0 {
        emit_codepoint(tk, first as c_int, key);
        *nbytep = 1;
        return TERMKEY_RES_KEY;
    }
    if tk.flags & TERMKEY_FLAG_UTF8 as c_int != 0 {
        let bytes = tk.buffered();
        let (codepoint, len, result) = match utf8::decode(bytes) {
            Decoded::Char { codepoint, len } => (codepoint, len, TERMKEY_RES_KEY),
            Decoded::Incomplete if force != 0 => {
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
        key.type_0 = TERMKEY_TYPE_UNICODE;
        key.modifiers = 0;
        emit_codepoint(tk, codepoint, key);
        return result;
    }
    // No UTF-8: every byte is its own character.
    key.type_0 = TERMKEY_TYPE_UNICODE;
    key.code = TermKeyKey_code {
        codepoint: first as c_int,
    };
    key.modifiers = 0;
    key.utf8[0] = first as c_char;
    key.utf8[1] = 0;
    *nbytep = 1;
    TERMKEY_RES_KEY
}

/// Decode an X10 mouse report: three bytes of button-and-modifiers, column and
/// line, each offset by a space so they stay printable.
pub fn peekkey_mouse(tk: Tk, key: &mut TermKeyKey, nbytep: &mut size_t) -> TermKeyResult {
    if tk.buffcount < 3 {
        return TERMKEY_RES_AGAIN;
    }
    key.type_0 = TERMKEY_TYPE_MOUSE;
    let mut payload: report::Payload = [0; 4];
    for (slot, byte) in payload.iter_mut().zip(&tk.buffered()[..3]) {
        *slot = (*byte as c_char as c_int - 0x20) as c_char;
    }
    // Bits 2-4 of the button code are the modifiers; lift them out of it.
    key.modifiers = (payload[0] as c_int & 0x1c) >> 2;
    payload[0] = (payload[0] as c_int & !0x1c) as c_char;
    key.code = TermKeyKey_code { mouse: payload };
    *nbytep = 3;
    TERMKEY_RES_KEY
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_getkey(tk: *mut TermKey, key: *mut TermKeyKey) -> TermKeyResult {
    // SAFETY: the caller's reader and the key it is reading into.
    let (mut tk, key) = unsafe { (Tk::of(tk), &mut *key) };
    let mut nbytes: size_t = 0;
    let ret = peekkey(tk, key, 0, &mut nbytes);
    if ret == TERMKEY_RES_KEY {
        tk.eat_bytes(nbytes);
    }
    if ret == TERMKEY_RES_AGAIN {
        // Fill the key in anyway, so a caller that gives up waiting has
        // something to report. The bytes stay in the buffer.
        peekkey(tk, key, 1, &mut nbytes);
    }
    ret
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_getkey_force(
    tk: *mut TermKey,
    key: *mut TermKeyKey,
) -> TermKeyResult {
    // SAFETY: the caller's reader and the key it is reading into.
    let (mut tk, key) = unsafe { (Tk::of(tk), &mut *key) };
    let mut nbytes: size_t = 0;
    let ret = peekkey(tk, key, 1, &mut nbytes);
    if ret == TERMKEY_RES_KEY {
        tk.eat_bytes(nbytes);
    }
    ret
}

/// The name of a symbolic key, or "UNKNOWN".
#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_get_keyname(_tk: *mut TermKey, sym: TermKeySym) -> *const c_char {
    keynames::name(sym).as_ptr()
}

/// Find the symbol named at the head of `str`, and where its name ends.
///
/// Returns null when nothing matches. On a match `*symp` is the symbol and the
/// return value points at the rest of `str`, so "DownMore" yields Down and
/// "More".
#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_lookup_keyname(
    _tk: *mut TermKey,
    text: *const c_char,
    symp: *mut TermKeySym,
) -> *const c_char {
    // SAFETY: the caller's NUL-terminated name.
    let name = unsafe { CStr::from_ptr(text) }.to_bytes();
    match keynames::lookup(name) {
        Some((sym, len)) => {
            // SAFETY: the caller's out-parameter, and `len` bytes of the name
            // matched, so `text + len` is inside the same string.
            unsafe {
                *symp = sym;
                text.add(len)
            }
        }
        None => core::ptr::null(),
    }
}

/// Render a key as text, in the manner of `snprintf`: at most `len - 1` bytes
/// plus a terminator are written, and the return value is the length the whole
/// rendering would have taken.
///
/// Upstream built the text with a series of `snprintf`s into `buffer + pos`
/// with a remaining size of `len - pos`. Once `pos` passed `len` — a key whose
/// modifiers alone overflowed the buffer, which `TERMKEY_FORMAT_MOUSE_POS`
/// makes easy — that subtraction wrapped to a size_t of about 2^64 and the next
/// write ran off the end of the caller's buffer. Rendering once and copying
/// what fits has no such edge.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_strfkey(
    _tk: *mut TermKey,
    buffer: *mut c_char,
    len: size_t,
    key: *mut TermKeyKey,
    format: TermKeyFormat,
) -> size_t {
    // SAFETY: the caller's key.
    let key = unsafe { &*key };
    // A key whose UTF-8 was never filled in — one the consumer built itself —
    // gets it derived from the codepoint.
    let mut encoded = [0 as c_char; 7];
    let utf8: &[u8] = if key.type_0 == TERMKEY_TYPE_UNICODE {
        if key.utf8[0] == 0 {
            fill_utf8(key.codepoint(), &mut encoded);
            utf8_bytes(&encoded)
        } else {
            utf8_bytes(&key.utf8)
        }
    } else {
        b""
    };

    let body = match key.type_0 {
        TERMKEY_TYPE_UNICODE => KeyBody::Unicode {
            codepoint: key.codepoint(),
            utf8,
        },
        TERMKEY_TYPE_KEYSYM => KeyBody::Sym(key.sym()),
        TERMKEY_TYPE_FUNCTION => KeyBody::Function(key.number()),
        TERMKEY_TYPE_MOUSE => {
            // What `termkey_interpret_mouse` answers, without the four
            // out-parameters: every one of them is wanted here.
            let payload = key.report();
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
            // Likewise `termkey_interpret_modereport`.
            let (initial, mode, value) = report::unpack_mode(&key.report());
            KeyBody::Mode {
                initial,
                mode,
                value,
            }
        }
        TERMKEY_TYPE_DCS => KeyBody::Dcs,
        TERMKEY_TYPE_OSC => KeyBody::Osc,
        TERMKEY_TYPE_APC => KeyBody::Apc,
        TERMKEY_TYPE_UNKNOWN_CSI => KeyBody::UnknownCsi(key.number()),
        _ => KeyBody::Unrecognised,
    };

    let text = format::render(&body, key.modifiers, format);
    if len > 0 {
        let written = text.len().min(len - 1);
        // SAFETY: the caller's buffer holds `len` bytes and `written < len`.
        unsafe {
            core::ptr::copy_nonoverlapping(text.as_ptr(), buffer as *mut u8, written);
            *buffer.add(written) = 0;
        }
    }
    text.len()
}

/// The payload of the last DCS, OSC or APC string, if `key` is still the one
/// that reported it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_interpret_string(
    tk: *mut TermKey,
    key: *const TermKeyKey,
    strp: *mut *const c_char,
) -> TermKeyResult {
    // SAFETY: the caller's reader and the key it is asking about.
    let (tk, key) = unsafe { (Tk::of(tk), &*key) };
    let kind = key.type_0;
    if kind != TERMKEY_TYPE_DCS && kind != TERMKEY_TYPE_OSC && kind != TERMKEY_TYPE_APC {
        return TERMKEY_RES_NONE;
    }
    // SAFETY: the CSI driver's state, allocated with this reader and live for
    // as long as it is.
    let csi = unsafe { &*tk.csi };
    // Each string gets a serial number, so a key held past the next one cannot
    // read a payload that is no longer its own.
    if csi.saved_string_id != key.number() {
        return TERMKEY_RES_NONE;
    }
    // SAFETY: the caller's out-parameter.
    unsafe { *strp = csi.saved_string };
    TERMKEY_RES_KEY
}
