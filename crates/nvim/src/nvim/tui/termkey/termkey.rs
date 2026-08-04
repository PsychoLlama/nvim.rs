//! libtermkey's core: the input buffer, the two drivers, and the translation
//! of a byte run into a key.

use crate::src::nvim::memory::{xfree, xmalloc, xrealloc};
use crate::src::nvim::tui::termkey::driver_csi::{
    self, termkey_interpret_modereport, termkey_interpret_mouse,
};
use crate::src::nvim::tui::termkey::driver_ti;
use crate::src::nvim::tui::termkey::format::{self, KeyBody};
use crate::src::nvim::tui::termkey::keynames;
use crate::src::nvim::tui::termkey::report;
use crate::src::nvim::tui::termkey::utf8::{self, Decoded, UNICODE_INVALID};
use crate::src::nvim::types::{
    TermKey, TermKey_Terminfo_Getstr_Hook, TermKeyCsi, TermKeyEvent, TermKeyFormat, TermKeyKey,
    TermKeyKey_code, TermKeyMouseEvent, TermKeyResult, TermKeySym, TermKeyType, TerminfoEntry,
    size_t,
};
use core::ffi::{CStr, c_char, c_int, c_uchar, c_void};

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

/// Create a key reader. `term` is the terminal's description, which may be null
/// when nothing is known about it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_new_abstract(
    term: *mut TerminfoEntry,
    flags: c_int,
) -> *mut TermKey {
    let tk: *mut TermKey = xmalloc(size_of::<TermKey>()) as *mut TermKey;
    (*tk).canonflags = 0;
    (*tk).buffsize = TERMKEY_DEFAULT_BUFFER_SIZE;
    (*tk).buffer = xmalloc((*tk).buffsize) as *mut c_uchar;
    (*tk).buffstart = 0;
    (*tk).buffcount = 0;
    (*tk).hightide = 0;
    (*tk).ti_getstr_hook = None;
    (*tk).ti_getstr_hook_data = core::ptr::null_mut();
    (*tk).is_started = 0;
    (*tk).ti = driver_ti::new_driver(term);
    (*tk).csi = driver_csi::new_driver();
    termkey_set_flags(tk, flags);
    if flags & TERMKEY_FLAG_NOSTART as c_int == 0 {
        termkey_start(tk);
    }
    tk
}

pub unsafe fn termkey_free(tk: *mut TermKey) {
    xfree((*tk).buffer as *mut c_void);
    (*tk).buffer = core::ptr::null_mut();
    driver_ti::free_driver((*tk).ti);
    driver_csi::free_driver((*tk).csi);
    xfree(tk as *mut c_void);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_destroy(tk: *mut TermKey) {
    if (*tk).is_started != 0 {
        termkey_stop(tk);
    }
    termkey_free(tk);
}

/// Install nvim's override for terminfo capability lookups, so it can supply
/// key sequences the terminal's description does not name.
pub unsafe fn termkey_hook_terminfo_getstr(
    tk: *mut TermKey,
    hookfn: Option<TermKey_Terminfo_Getstr_Hook>,
    data: *mut c_void,
) {
    (*tk).ti_getstr_hook = hookfn;
    (*tk).ti_getstr_hook_data = data;
}

/// Begin reading keys. Upstream also put the terminal into raw mode here and
/// restored it on stop, but that was guarded on a file descriptor this tree
/// never gives it — nvim owns the terminal and feeds bytes in by hand.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_start(tk: *mut TermKey) -> c_int {
    if (*tk).is_started != 0 {
        return 1;
    }
    driver_ti::load_keys(tk);
    (*tk).is_started = 1;
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_stop(tk: *mut TermKey) -> c_int {
    (*tk).is_started = 0;
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_set_flags(tk: *mut TermKey, newflags: c_int) {
    (*tk).flags = newflags;
    // The two spellings of "a space is a symbol, not a character" are kept in
    // step in both directions.
    if (*tk).flags & TERMKEY_FLAG_SPACESYMBOL as c_int != 0 {
        (*tk).canonflags |= TERMKEY_CANON_SPACESYMBOL as c_int;
    } else {
        (*tk).canonflags &= !(TERMKEY_CANON_SPACESYMBOL as c_int);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_get_canonflags(tk: *mut TermKey) -> c_int {
    (*tk).canonflags
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_set_canonflags(tk: *mut TermKey, flags: c_int) {
    (*tk).canonflags = flags;
    if (*tk).canonflags & TERMKEY_CANON_SPACESYMBOL as c_int != 0 {
        (*tk).flags |= TERMKEY_FLAG_SPACESYMBOL as c_int;
    } else {
        (*tk).flags &= !(TERMKEY_FLAG_SPACESYMBOL as c_int);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_get_buffer_size(tk: *mut TermKey) -> size_t {
    (*tk).buffsize
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_set_buffer_size(tk: *mut TermKey, size: size_t) -> c_int {
    (*tk).buffer = xrealloc((*tk).buffer as *mut c_void, size) as *mut c_uchar;
    (*tk).buffsize = size;
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_get_buffer_remaining(tk: *mut TermKey) -> size_t {
    (*tk).buffsize - (*tk).buffcount
}

/// Hand more input to the reader. Returns how much of it was taken, or `-1` as
/// a `size_t` when the buffer was already full.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_push_bytes(
    tk: *mut TermKey,
    bytes: *const c_char,
    len: size_t,
) -> size_t {
    if (*tk).buffstart != 0 {
        // Slide what is left of the previous read back to the front.
        core::ptr::copy(
            (*tk).buffer.add((*tk).buffstart),
            (*tk).buffer,
            (*tk).buffcount,
        );
        (*tk).buffstart = 0;
    }
    if (*tk).buffcount >= (*tk).buffsize {
        return -1i32 as size_t;
    }
    let len = len.min((*tk).buffsize - (*tk).buffcount);
    core::ptr::copy_nonoverlapping(bytes as *const u8, (*tk).buffer.add((*tk).buffcount), len);
    (*tk).buffcount += len;
    len
}

unsafe fn eat_bytes(tk: *mut TermKey, count: size_t) {
    if count >= (*tk).buffcount {
        (*tk).buffstart = 0;
        (*tk).buffcount = 0;
    } else {
        (*tk).buffstart += count;
        (*tk).buffcount -= count;
    }
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
pub unsafe fn emit_codepoint(tk: *mut TermKey, codepoint: c_int, key: *mut TermKeyKey) {
    let flags = (*tk).flags;
    let interpret = flags & TERMKEY_FLAG_NOINTERPRET as c_int == 0;
    if codepoint == 0 {
        // NUL is Ctrl-Space, which has no character of its own.
        (*key).type_0 = TERMKEY_TYPE_KEYSYM;
        (*key).code = TermKeyKey_code {
            sym: TERMKEY_SYM_SPACE,
        };
        (*key).modifiers = TERMKEY_KEYMOD_CTRL as c_int;
    } else if codepoint < 0x20 && flags & TERMKEY_FLAG_KEEPC0 as c_int == 0 {
        let sym = if interpret {
            C0_SYMS[codepoint as usize]
        } else {
            TERMKEY_SYM_NONE
        };
        if sym == TERMKEY_SYM_NONE {
            // Ctrl-letter: C0 codes map onto '@' through '_', but the
            // lower-case letter is the friendlier name for A-Z.
            (*key).type_0 = TERMKEY_TYPE_UNICODE;
            let base = if (b'A' as c_int..=b'Z' as c_int).contains(&(codepoint + 0x40)) {
                0x60
            } else {
                0x40
            };
            (*key).code = TermKeyKey_code {
                codepoint: codepoint + base,
            };
            (*key).modifiers = TERMKEY_KEYMOD_CTRL as c_int;
        } else {
            (*key).type_0 = TERMKEY_TYPE_KEYSYM;
            (*key).code = TermKeyKey_code { sym };
            (*key).modifiers = 0;
        }
    } else if codepoint == 0x7f && interpret {
        (*key).type_0 = TERMKEY_TYPE_KEYSYM;
        (*key).code = TermKeyKey_code {
            sym: TERMKEY_SYM_DEL,
        };
        (*key).modifiers = 0;
    } else if (0x80..0xa0).contains(&codepoint) {
        // The C1 controls are Alt-Ctrl-letter.
        (*key).type_0 = TERMKEY_TYPE_UNICODE;
        (*key).code = TermKeyKey_code {
            codepoint: codepoint - 0x40,
        };
        (*key).modifiers = (TERMKEY_KEYMOD_CTRL | TERMKEY_KEYMOD_ALT) as c_int;
    } else {
        (*key).type_0 = TERMKEY_TYPE_UNICODE;
        (*key).code = TermKeyKey_code { codepoint };
        (*key).modifiers = 0;
    }
    termkey_canonicalise(tk, key);
    if (*key).type_0 == TERMKEY_TYPE_UNICODE {
        fill_utf8((*key).code.codepoint, &mut (*key).utf8);
    }
}

/// Apply the consumer's preferences about which of two equivalent spellings a
/// key gets: space as a symbol or a character, and DEL as backspace.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_canonicalise(tk: *mut TermKey, key: *mut TermKeyKey) {
    let flags = (*tk).canonflags;
    if flags & TERMKEY_CANON_SPACESYMBOL as c_int != 0 {
        if (*key).type_0 == TERMKEY_TYPE_UNICODE && (*key).code.codepoint == 0x20 {
            (*key).type_0 = TERMKEY_TYPE_KEYSYM;
            (*key).code = TermKeyKey_code {
                sym: TERMKEY_SYM_SPACE,
            };
        }
    } else if (*key).type_0 == TERMKEY_TYPE_KEYSYM && (*key).code.sym == TERMKEY_SYM_SPACE {
        (*key).type_0 = TERMKEY_TYPE_UNICODE;
        (*key).code = TermKeyKey_code { codepoint: 0x20 };
        fill_utf8(0x20, &mut (*key).utf8);
    }
    if flags & TERMKEY_CANON_DELBS as c_int != 0
        && (*key).type_0 == TERMKEY_TYPE_KEYSYM
        && (*key).code.sym == TERMKEY_SYM_DEL
    {
        (*key).code = TermKeyKey_code {
            sym: TERMKEY_SYM_BACKSPACE,
        };
    }
}

/// Read the next key without consuming it. `force` means "decide now": a
/// sequence that could still grow is taken as complete instead of waiting.
unsafe fn peekkey(
    tk: *mut TermKey,
    key: *mut TermKeyKey,
    force: c_int,
    nbytep: *mut size_t,
) -> TermKeyResult {
    if (*tk).is_started == 0 {
        return TERMKEY_RES_ERROR;
    }
    (*key).event = TERMKEY_EVENT_PRESS;
    if (*tk).hightide != 0 {
        // The tail of an unrecognised control sequence, held back so the
        // consumer could re-read it with `termkey_interpret_csi`.
        (*tk).buffstart += (*tk).hightide;
        (*tk).buffcount -= (*tk).hightide;
        (*tk).hightide = 0;
    }
    let mut again = false;
    // The terminfo driver has first refusal (its sequences come from the
    // terminal's own description), then the CSI driver's generic parsing.
    for probe in 0..2 {
        let ret = if probe == 0 {
            driver_ti::peek_key(tk, key, force, nbytep)
        } else {
            driver_csi::peek_key(tk, (*tk).csi, key, force, nbytep)
        };
        match ret {
            TERMKEY_RES_KEY => {
                // Reclaim the front half of the buffer once reads have walked
                // past its midpoint, so a long run does not push buffstart off
                // the end. Only worth doing when a key was actually consumed.
                let halfsize = (*tk).buffsize / 2;
                if (*tk).buffstart > halfsize {
                    core::ptr::copy_nonoverlapping(
                        (*tk).buffer.add(halfsize),
                        (*tk).buffer,
                        halfsize,
                    );
                    (*tk).buffstart -= halfsize;
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
unsafe fn peekkey_simple(
    tk: *mut TermKey,
    key: *mut TermKeyKey,
    force: c_int,
    nbytep: *mut size_t,
) -> TermKeyResult {
    if (*tk).buffcount == 0 {
        return TERMKEY_RES_NONE;
    }
    let first = *(*tk).buffer.add((*tk).buffstart);
    if first == 0x1b {
        if (*tk).buffcount == 1 {
            if force == 0 {
                return TERMKEY_RES_AGAIN;
            }
            // Nothing followed it, so it was the escape key itself.
            emit_codepoint(tk, first as c_int, key);
            *nbytep = 1;
            return TERMKEY_RES_KEY;
        }
        // Read what follows as its own key, then add the Alt modifier.
        (*tk).buffstart += 1;
        (*tk).buffcount -= 1;
        let result = peekkey(tk, key, force, nbytep);
        (*tk).buffstart -= 1;
        (*tk).buffcount += 1;
        if result == TERMKEY_RES_KEY {
            (*key).modifiers |= TERMKEY_KEYMOD_ALT as c_int;
            *nbytep += 1;
        }
        return result;
    }
    if first < 0xa0 {
        emit_codepoint(tk, first as c_int, key);
        *nbytep = 1;
        return TERMKEY_RES_KEY;
    }
    if (*tk).flags & TERMKEY_FLAG_UTF8 as c_int != 0 {
        let bytes = core::slice::from_raw_parts((*tk).buffer.add((*tk).buffstart), (*tk).buffcount);
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
        (*key).type_0 = TERMKEY_TYPE_UNICODE;
        (*key).modifiers = 0;
        emit_codepoint(tk, codepoint, key);
        return result;
    }
    // No UTF-8: every byte is its own character.
    (*key).type_0 = TERMKEY_TYPE_UNICODE;
    (*key).code = TermKeyKey_code {
        codepoint: first as c_int,
    };
    (*key).modifiers = 0;
    (*key).utf8[0] = first as c_char;
    (*key).utf8[1] = 0;
    *nbytep = 1;
    TERMKEY_RES_KEY
}

/// Decode an X10 mouse report: three bytes of button-and-modifiers, column and
/// line, each offset by a space so they stay printable.
pub unsafe fn peekkey_mouse(
    tk: *mut TermKey,
    key: *mut TermKeyKey,
    nbytep: *mut size_t,
) -> TermKeyResult {
    if (*tk).buffcount < 3 {
        return TERMKEY_RES_AGAIN;
    }
    let bytes = core::slice::from_raw_parts((*tk).buffer.add((*tk).buffstart), 3);
    (*key).type_0 = TERMKEY_TYPE_MOUSE;
    let mut payload: report::Payload = [0; 4];
    for (slot, byte) in payload.iter_mut().zip(bytes) {
        *slot = (*byte as c_char as c_int - 0x20) as c_char;
    }
    // Bits 2-4 of the button code are the modifiers; lift them out of it.
    (*key).modifiers = (payload[0] as c_int & 0x1c) >> 2;
    payload[0] = (payload[0] as c_int & !0x1c) as c_char;
    (*key).code = TermKeyKey_code { mouse: payload };
    *nbytep = 3;
    TERMKEY_RES_KEY
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_getkey(tk: *mut TermKey, key: *mut TermKeyKey) -> TermKeyResult {
    let mut nbytes: size_t = 0;
    let ret = peekkey(tk, key, 0, &raw mut nbytes);
    if ret == TERMKEY_RES_KEY {
        eat_bytes(tk, nbytes);
    }
    if ret == TERMKEY_RES_AGAIN {
        // Fill the key in anyway, so a caller that gives up waiting has
        // something to report. The bytes stay in the buffer.
        peekkey(tk, key, 1, &raw mut nbytes);
    }
    ret
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn termkey_getkey_force(
    tk: *mut TermKey,
    key: *mut TermKeyKey,
) -> TermKeyResult {
    let mut nbytes: size_t = 0;
    let ret = peekkey(tk, key, 1, &raw mut nbytes);
    if ret == TERMKEY_RES_KEY {
        eat_bytes(tk, nbytes);
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
    match keynames::lookup(CStr::from_ptr(text).to_bytes()) {
        Some((sym, len)) => {
            *symp = sym;
            text.add(len)
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
    tk: *mut TermKey,
    buffer: *mut c_char,
    len: size_t,
    key: *mut TermKeyKey,
    format: TermKeyFormat,
) -> size_t {
    // A key whose UTF-8 was never filled in — one the consumer built itself —
    // gets it derived from the codepoint.
    let mut encoded = [0 as c_char; 7];
    let utf8: &[u8] = if (*key).type_0 == TERMKEY_TYPE_UNICODE {
        let source = if (*key).utf8[0] == 0 {
            fill_utf8((*key).code.codepoint, &mut encoded);
            &encoded
        } else {
            &(*key).utf8
        };
        let raw = &*(source as *const [c_char; 7] as *const [u8; 7]);
        // Bounded, unlike upstream's `%s`: a key the consumer built itself need
        // not have terminated the field.
        &raw[..raw.iter().position(|&byte| byte == 0).unwrap_or(raw.len())]
    } else {
        b""
    };

    let mut mouse_event: TermKeyMouseEvent = TERMKEY_MOUSE_UNKNOWN;
    let (mut button, mut line, mut col) = (0, 0, 0);
    let (mut initial, mut mode, mut value) = (0, 0, 0);
    let body = match (*key).type_0 {
        TERMKEY_TYPE_UNICODE => KeyBody::Unicode {
            codepoint: (*key).code.codepoint,
            utf8,
        },
        TERMKEY_TYPE_KEYSYM => KeyBody::Sym((*key).code.sym),
        TERMKEY_TYPE_FUNCTION => KeyBody::Function((*key).code.number),
        TERMKEY_TYPE_MOUSE => {
            termkey_interpret_mouse(
                tk,
                key,
                &raw mut mouse_event,
                &raw mut button,
                &raw mut line,
                &raw mut col,
            );
            KeyBody::Mouse {
                event: mouse_event,
                button,
                line,
                col,
            }
        }
        TERMKEY_TYPE_POSITION => KeyBody::Position,
        TERMKEY_TYPE_MODEREPORT => {
            termkey_interpret_modereport(tk, key, &raw mut initial, &raw mut mode, &raw mut value);
            KeyBody::Mode {
                initial,
                mode,
                value,
            }
        }
        TERMKEY_TYPE_DCS => KeyBody::Dcs,
        TERMKEY_TYPE_OSC => KeyBody::Osc,
        TERMKEY_TYPE_APC => KeyBody::Apc,
        TERMKEY_TYPE_UNKNOWN_CSI => KeyBody::UnknownCsi((*key).code.number),
        _ => KeyBody::Unrecognised,
    };

    let text = format::render(&body, (*key).modifiers, format);
    if len > 0 {
        let written = text.len().min(len - 1);
        core::ptr::copy_nonoverlapping(text.as_ptr(), buffer as *mut u8, written);
        *buffer.add(written) = 0;
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
    let kind = (*key).type_0;
    if kind != TERMKEY_TYPE_DCS && kind != TERMKEY_TYPE_OSC && kind != TERMKEY_TYPE_APC {
        return TERMKEY_RES_NONE;
    }
    let csi: *mut TermKeyCsi = (*tk).csi;
    // Each string gets a serial number, so a key held past the next one cannot
    // read a payload that is no longer its own.
    if (*csi).saved_string_id != (*key).code.number {
        return TERMKEY_RES_NONE;
    }
    *strp = (*csi).saved_string;
    TERMKEY_RES_KEY
}
