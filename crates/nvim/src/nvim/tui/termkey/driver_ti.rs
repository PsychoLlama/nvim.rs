//! libtermkey's terminfo key driver.
//!
//! At start-up it asks the terminal's description (and nvim's override hook)
//! for every key capability it knows about, and builds a trie of the escape
//! sequences those capabilities named. Input is then matched against the trie
//! before the generic CSI driver gets a look.
//!
//! Ported from libtermkey, Copyright (c) 2007-2011 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libtermkey-LICENSE.txt.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::tui::terminfo::caps::{KEYS, MAX_FUNCTION_KEY, key_slot};
use crate::src::nvim::tui::termkey::termkey::{
    TERMKEY_KEYMOD_SHIFT, TERMKEY_RES_AGAIN, TERMKEY_RES_KEY, TERMKEY_RES_NONE,
    TERMKEY_SYM_BACKSPACE, TERMKEY_SYM_BEGIN, TERMKEY_SYM_CLEAR, TERMKEY_SYM_DELETE,
    TERMKEY_SYM_END, TERMKEY_SYM_FIND, TERMKEY_SYM_HOME, TERMKEY_SYM_INSERT, TERMKEY_SYM_LEFT,
    TERMKEY_SYM_PAGEDOWN, TERMKEY_SYM_PAGEUP, TERMKEY_SYM_RIGHT, TERMKEY_SYM_SELECT,
    TERMKEY_SYM_SUSPEND, TERMKEY_SYM_TAB, TERMKEY_SYM_UNDO, TERMKEY_TYPE_FUNCTION,
    TERMKEY_TYPE_KEYSYM, Tk,
};
use crate::src::nvim::tui::termkey::trie::{KeyTrie, Lookup};
use crate::src::nvim::types::{
    TermKeyKey, TermKeyResult, TermKeySym, TermKeyType, TerminfoEntry, keyinfo, size_t,
};
use core::ffi::{CStr, c_char, c_int, c_void};
use std::ffi::CString;

/// One of the named (non-`key_fN`) capabilities the driver looks up.
struct NamedKey {
    /// Which of `tui::terminfo::caps::KEYS` this is. That index is also the
    /// slot in `TerminfoEntry::keys`, and it carries the capability names the
    /// getstr hook is asked for, so the two tables cannot drift apart.
    slot: usize,
    kind: TermKeyType,
    sym: TermKeySym,
    /// Modifiers the capability implies on its own (only shift, for `key_btab`).
    mods: c_int,
}

const fn named(slot: usize, sym: TermKeySym) -> NamedKey {
    NamedKey {
        slot,
        kind: TERMKEY_TYPE_KEYSYM,
        sym,
        mods: 0,
    }
}

/// Registration order matters: the first sequence to claim a byte string wins,
/// so this list keeps upstream's, alphabetical by capability.
static NAMED_KEYS: [NamedKey; 16] = [
    named(key_slot::BACKSPACE, TERMKEY_SYM_BACKSPACE),
    named(key_slot::BEG, TERMKEY_SYM_BEGIN),
    NamedKey {
        slot: key_slot::BTAB,
        kind: TERMKEY_TYPE_KEYSYM,
        sym: TERMKEY_SYM_TAB,
        mods: TERMKEY_KEYMOD_SHIFT as c_int,
    },
    named(key_slot::CLEAR, TERMKEY_SYM_CLEAR),
    named(key_slot::DC, TERMKEY_SYM_DELETE),
    named(key_slot::END, TERMKEY_SYM_END),
    named(key_slot::FIND, TERMKEY_SYM_FIND),
    named(key_slot::HOME, TERMKEY_SYM_HOME),
    named(key_slot::IC, TERMKEY_SYM_INSERT),
    named(key_slot::LEFT, TERMKEY_SYM_LEFT),
    named(key_slot::NPAGE, TERMKEY_SYM_PAGEDOWN),
    named(key_slot::PPAGE, TERMKEY_SYM_PAGEUP),
    named(key_slot::RIGHT, TERMKEY_SYM_RIGHT),
    named(key_slot::SELECT, TERMKEY_SYM_SELECT),
    named(key_slot::SUSPEND, TERMKEY_SYM_SUSPEND),
    named(key_slot::UNDO, TERMKEY_SYM_UNDO),
];

/// The driver's per-`TermKey` state. Reached through `TermKey::ti` as an opaque
/// pointer, because `TermKey` is still a `repr(C)` type the unit specs poke at
/// and this one is not.
pub struct TerminfoDriver {
    entry: *mut TerminfoEntry,
    /// Built on the first start; `None` until then.
    keys: Option<KeyTrie>,
}

/// Allocate driver state for a terminal description, which may be null when
/// nothing is known about the terminal — the getstr hook can still supply
/// sequences.
pub fn new_driver(entry: *mut TerminfoEntry) -> *mut c_void {
    Box::into_raw(Box::new(TerminfoDriver { entry, keys: None })) as *mut c_void
}

pub unsafe fn free_driver(info: *mut c_void) {
    // SAFETY: the box `new_driver` leaked, reclaimed exactly once — the reader
    // that owns it is destroyed once.
    drop(unsafe { Box::from_raw(info as *mut TerminfoDriver) });
}

/// The driver state hanging off a reader.
///
/// `TermKey::ti` is the box `new_driver` leaked. The reader owns it for its
/// whole life and nothing else reaches it, so handing out a reference rests on
/// exactly the promise `Tk` already carries.
fn driver_of<'a>(tk: Tk) -> &'a mut TerminfoDriver {
    // SAFETY: as the doc comment says.
    unsafe { &mut *(tk.ti as *mut TerminfoDriver) }
}

/// Read one capability, giving nvim's hook the final say.
///
/// Returns the escape sequence, or `None` when the capability is absent.
/// Terminfo spells absence two ways — a null pointer and the historical
/// `(char *)-1` — and the hook may return either; an empty string is no
/// sequence to match against.
fn capability<'a>(tk: Tk, from_entry: *const c_char, name: &CStr) -> Option<&'a [u8]> {
    let mut value = from_entry;
    if let Some(hook) = tk.ti_getstr_hook {
        // SAFETY: the hook nvim installed with `termkey_hook_terminfo_getstr`,
        // called with a capability name and its own data, as it expects.
        value = unsafe { hook(name.as_ptr(), value, tk.ti_getstr_hook_data) };
    }
    if value.is_null() || value.addr() == usize::MAX {
        return None;
    }
    // SAFETY: a terminfo capability is a NUL-terminated string, whether it came
    // from the terminal's description or from nvim's hook, and both outlive the
    // trie built out of them.
    let bytes = unsafe { CStr::from_ptr(value) }.to_bytes();
    (!bytes.is_empty()).then_some(bytes)
}

/// Register a capability's sequence if the terminal has one. Reports whether it
/// did, which is how the caller decides whether to bother with the shifted
/// variant.
fn register(
    tk: Tk,
    trie: &mut KeyTrie,
    from_entry: *const c_char,
    name: &CStr,
    info: keyinfo,
) -> bool {
    match capability(tk, from_entry, name) {
        Some(seq) => {
            trie.insert(seq, info);
            true
        }
        None => false,
    }
}

/// Build the trie from the terminal's description.
fn load(tk: Tk, entry: *mut TerminfoEntry) -> KeyTrie {
    let mut trie = KeyTrie::default();
    // SAFETY: the description nvim handed to `termkey_new_abstract`, which
    // outlives the reader; null when nothing was known about the terminal.
    let entry = unsafe { entry.as_ref() };
    for key in &NAMED_KEYS {
        let cap = &KEYS[key.slot];
        let (plain, shifted) = match entry {
            Some(entry) => (entry.keys[key.slot][0], entry.keys[key.slot][1]),
            None => (core::ptr::null(), core::ptr::null()),
        };
        let loaded = register(
            tk,
            &mut trie,
            plain,
            cap.name,
            keyinfo {
                type_0: key.kind,
                sym: key.sym,
                modifier_mask: key.mods,
                modifier_set: key.mods,
            },
        );
        // Upstream only reaches for the shifted variant when the unshifted one
        // was present, so a terminal describing only `key_sfoo` gets neither.
        if loaded {
            let mods = key.mods | TERMKEY_KEYMOD_SHIFT as c_int;
            register(
                tk,
                &mut trie,
                shifted,
                cap.shifted_name,
                keyinfo {
                    type_0: key.kind,
                    sym: key.sym,
                    modifier_mask: mods,
                    modifier_set: mods,
                },
            );
        }
    }
    // Function keys are numbered, not symbolic: `keyinfo::sym` carries the
    // number when the type is TERMKEY_TYPE_FUNCTION. The scan stops at the
    // first gap, so a terminal without `key_f1` gets no function keys at all.
    for number in 1..=MAX_FUNCTION_KEY {
        let from_entry = match entry {
            Some(entry) => entry.f_keys[number - 1],
            None => core::ptr::null(),
        };
        let name = CString::new(format!("key_f{number}")).expect("no interior NUL");
        let loaded = register(
            tk,
            &mut trie,
            from_entry,
            &name,
            keyinfo {
                type_0: TERMKEY_TYPE_FUNCTION,
                sym: number as TermKeySym,
                modifier_mask: 0,
                modifier_set: 0,
            },
        );
        if !loaded {
            break;
        }
    }
    trie
}

/// Build the key trie if this is the first start.
///
/// Upstream also wrote `keypad_xmit` to the terminal here and `keypad_local` on
/// stop, but both were guarded on `TermKey::fd`, which this tree never sets:
/// nvim feeds termkey through `termkey_push_bytes` and owns terminal output
/// itself. Both paths were unreachable and are gone, along with the stop hook,
/// which then had nothing left to do.
pub fn load_keys(tk: Tk) {
    let driver = driver_of(tk);
    if driver.keys.is_none() {
        let entry = driver.entry;
        driver.keys = Some(load(tk, entry));
    }
}

/// Match the head of the input buffer against the terminal's key sequences.
pub fn peek_key(tk: Tk, key: &mut TermKeyKey, force: c_int, nbytep: &mut size_t) -> TermKeyResult {
    if tk.buffcount == 0 {
        return TERMKEY_RES_NONE;
    }
    let bytes = tk.buffered();
    let found = driver_of(tk)
        .keys
        .as_ref()
        .map_or(Lookup::None, |t| t.lookup(bytes));
    match found {
        // Every capability this driver loads names a symbol or a function key,
        // so upstream's branch handing a TERMKEY_TYPE_MOUSE hit to the mouse
        // decoder could never run and is gone.
        Lookup::Key { info, consumed } => {
            key.type_0 = info.type_0;
            key.code.sym = info.sym;
            key.modifiers = info.modifier_set;
            *nbytep = consumed;
            TERMKEY_RES_KEY
        }
        // More bytes could still complete a sequence, unless the caller has
        // given up waiting for them.
        Lookup::Partial if force == 0 => TERMKEY_RES_AGAIN,
        _ => TERMKEY_RES_NONE,
    }
}
