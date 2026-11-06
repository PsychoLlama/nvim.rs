//! libtermkey's terminfo key driver.
//!
//! At start-up it asks the terminal's description (and nvim's override hook)
//! for every key capability it knows about, and builds a trie of the escape
//! sequences those capabilities named. Input is then matched against the trie
//! before the generic CSI driver gets a look.
//!
//! Ported from libtermkey, Copyright (c) 2007-2011 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libtermkey-LICENSE.txt.

#![forbid(unsafe_code)]

use crate::narrow::len_as_int;
use crate::tui::terminfo::caps::{KEYS, MAX_FUNCTION_KEY, key_slot};
use crate::tui::termkey::termkey::{
    TERMKEY_KEYMOD_SHIFT, TERMKEY_RES_AGAIN, TERMKEY_RES_KEY, TERMKEY_RES_NONE,
    TERMKEY_SYM_BACKSPACE, TERMKEY_SYM_BEGIN, TERMKEY_SYM_CLEAR, TERMKEY_SYM_DELETE,
    TERMKEY_SYM_END, TERMKEY_SYM_FIND, TERMKEY_SYM_HOME, TERMKEY_SYM_INSERT, TERMKEY_SYM_LEFT,
    TERMKEY_SYM_PAGEDOWN, TERMKEY_SYM_PAGEUP, TERMKEY_SYM_RIGHT, TERMKEY_SYM_SELECT,
    TERMKEY_SYM_SUSPEND, TERMKEY_SYM_TAB, TERMKEY_SYM_UNDO, TERMKEY_TYPE_FUNCTION,
    TERMKEY_TYPE_KEYSYM, TermKey,
};
use crate::tui::termkey::trie::{KeyTrie, Lookup};
use crate::types::{
    KeyCode, TermKeyKey, TermKeyResult, TermKeySym, TermKeyType, TerminfoEntry, keyinfo,
};
use core::ffi::{CStr, c_int};
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
        mods: TERMKEY_KEYMOD_SHIFT,
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

/// The driver's per-reader state: the key trie, built on the first start.
#[derive(Default)]
pub struct TerminfoDriver {
    keys: Option<KeyTrie>,
}

/// Read one capability, giving nvim's hook the final say.
///
/// Returns the escape sequence, or `None` when the capability is absent. An
/// empty string is no sequence to match against.
fn capability(tk: &TermKey, from_entry: Option<&[u8]>, name: &CStr) -> Option<Vec<u8>> {
    let value = match tk.terminfo_hook() {
        Some(hook) => hook(name, from_entry),
        None => from_entry.map(<[u8]>::to_vec),
    };
    value.filter(|bytes| !bytes.is_empty())
}

/// Register a capability's sequence if the terminal has one. Reports whether it
/// did, which is how the caller decides whether to bother with the shifted
/// variant.
fn register(
    tk: &TermKey,
    trie: &mut KeyTrie,
    from_entry: Option<&[u8]>,
    name: &CStr,
    info: keyinfo,
) -> bool {
    match capability(tk, from_entry, name) {
        Some(seq) => {
            trie.insert(&seq, info);
            true
        }
        None => false,
    }
}

/// Build the trie from the terminal's description, `None` when nothing is
/// known about the terminal -- the hook can still supply sequences.
fn load(tk: &TermKey, entry: Option<&TerminfoEntry>) -> KeyTrie {
    let mut trie = KeyTrie::default();
    for key in &NAMED_KEYS {
        let cap = &KEYS[key.slot];
        let (plain, shifted) = match entry {
            Some(entry) => (
                entry.keys[key.slot][0].as_deref(),
                entry.keys[key.slot][1].as_deref(),
            ),
            None => (None, None),
        };
        let loaded = register(
            tk,
            &mut trie,
            plain,
            cap.name,
            keyinfo {
                kind: key.kind,
                sym: key.sym,
                modifier_mask: key.mods,
                modifier_set: key.mods,
            },
        );
        // Upstream only reaches for the shifted variant when the unshifted one
        // was present, so a terminal describing only `key_sfoo` gets neither.
        if loaded {
            let mods = key.mods | TERMKEY_KEYMOD_SHIFT;
            register(
                tk,
                &mut trie,
                shifted,
                cap.shifted_name,
                keyinfo {
                    kind: key.kind,
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
            Some(entry) => entry.f_keys[number - 1].as_deref(),
            None => None,
        };
        let name = CString::new(format!("key_f{number}")).expect("no interior NUL");
        let loaded = register(
            tk,
            &mut trie,
            from_entry,
            &name,
            keyinfo {
                kind: TERMKEY_TYPE_FUNCTION,
                sym: len_as_int(number),
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
/// nvim feeds termkey through `push_bytes` and owns terminal output itself.
/// Both paths were unreachable and are gone, along with the stop hook, which
/// then had nothing left to do.
pub fn load_keys(tk: &mut TermKey, entry: Option<&TerminfoEntry>) {
    if tk.ti.keys.is_none() {
        let keys = load(tk, entry);
        tk.ti.keys = Some(keys);
    }
}

/// Match the head of the input buffer against the terminal's key sequences.
pub fn peek_key(
    tk: &TermKey,
    key: &mut TermKeyKey,
    force: bool,
    nbytep: &mut usize,
) -> TermKeyResult {
    if tk.buffcount == 0 {
        return TERMKEY_RES_NONE;
    }
    let found = tk
        .ti
        .keys
        .as_ref()
        .map_or(Lookup::None, |t| t.lookup(tk.buffered()));
    match found {
        // Every capability this driver loads names a symbol or a function key,
        // so upstream's branch handing a TERMKEY_TYPE_MOUSE hit to the mouse
        // decoder could never run and is gone.
        Lookup::Key { info, consumed } => {
            key.kind = info.kind;
            key.code = KeyCode::scalar(info.sym);
            key.modifiers = info.modifier_set;
            *nbytep = consumed;
            TERMKEY_RES_KEY
        }
        // More bytes could still complete a sequence, unless the caller has
        // given up waiting for them.
        Lookup::Partial if !force => TERMKEY_RES_AGAIN,
        _ => TERMKEY_RES_NONE,
    }
}
