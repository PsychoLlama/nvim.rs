#![forbid(unsafe_code)]

//! The names of the symbolic keys.
//!
//! Ported from libtermkey, Copyright (c) 2007-2011 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libtermkey-LICENSE.txt.

use crate::types::TermKeySym;
use core::ffi::CStr;

/// Every symbol's name, indexed by the symbol. Upstream built the same list at
/// construction time into a heap array that only ever held these entries.
///
/// One deviation: it registered "NONE" through the path meant for *new* symbols
/// a consumer invents, which allocates the next free number rather than using
/// the one given. "NONE" landed at index 64 with 0 and 60-63 left null, so
/// `termkey_get_keyname(TERMKEY_SYM_NONE)` returned a null pointer that
/// `termkey_strfkey` would have printed as "(null)". Here the name belongs to
/// the symbol it names.
pub static KEY_NAMES: [&CStr; 60] = [
    c"NONE",
    c"Backspace",
    c"Tab",
    c"Enter",
    c"Escape",
    c"Space",
    c"DEL",
    c"Up",
    c"Down",
    c"Left",
    c"Right",
    c"Begin",
    c"Find",
    c"Insert",
    c"Delete",
    c"Select",
    c"PageUp",
    c"PageDown",
    c"Home",
    c"End",
    c"Cancel",
    c"Clear",
    c"Close",
    c"Command",
    c"Copy",
    c"Exit",
    c"Help",
    c"Mark",
    c"Message",
    c"Move",
    c"Open",
    c"Options",
    c"Print",
    c"Redo",
    c"Reference",
    c"Refresh",
    c"Replace",
    c"Restart",
    c"Resume",
    c"Save",
    c"Suspend",
    c"Undo",
    c"KP0",
    c"KP1",
    c"KP2",
    c"KP3",
    c"KP4",
    c"KP5",
    c"KP6",
    c"KP7",
    c"KP8",
    c"KP9",
    c"KPEnter",
    c"KPPlus",
    c"KPMinus",
    c"KPMult",
    c"KPDiv",
    c"KPComma",
    c"KPPeriod",
    c"KPEquals",
];

/// What every symbol outside the table is called.
pub const UNKNOWN_NAME: &CStr = c"UNKNOWN";

pub fn name(sym: TermKeySym) -> &'static CStr {
    usize::try_from(sym)
        .ok()
        .and_then(|sym| KEY_NAMES.get(sym).copied())
        .unwrap_or(UNKNOWN_NAME)
}

/// A name as text. The names are all ASCII, so this never has to decide what
/// to do about a name that is not.
pub fn text(name: &CStr) -> &str {
    name.to_str().unwrap_or_default()
}

/// Rewrite a camel-cased name as lower case with spaces: "PageUp" becomes
/// "page up". A capital only starts a new word when a lower-case letter came
/// before it, so "KP0" stays one word and "KPEnter" becomes "kpenter".
pub fn spaced_lowercase(name: &CStr) -> String {
    let mut out = String::with_capacity(name.count_bytes());
    let mut prev_lower = false;
    for ch in text(name).chars() {
        if ch.is_ascii_uppercase() && prev_lower {
            out.push(' ');
        }
        prev_lower = ch.is_ascii_lowercase();
        out.push(ch.to_ascii_lowercase());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_symbol_is_named_at_its_own_number() {
        assert_eq!(name(0), c"NONE");
        assert_eq!(name(5), c"Space");
        assert_eq!(name(7), c"Up");
        assert_eq!(name(59), c"KPEquals");
    }

    #[test]
    fn symbols_outside_the_table_are_unknown() {
        assert_eq!(name(-1), UNKNOWN_NAME);
        assert_eq!(name(60), UNKNOWN_NAME);
        assert_eq!(name(9999), UNKNOWN_NAME);
    }

    #[test]
    fn every_name_is_distinct_and_none_is_a_prefix_of_another() {
        let mut sorted = KEY_NAMES.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), KEY_NAMES.len(), "two symbols share a name");
        // Upstream resolved a name by taking the first entry that prefixed the
        // text, so a name that prefixed a later one would have shadowed it.
        // Nothing depends on that any more, but a table where it could happen
        // is a table where two spellings mean one key.
        for (sym, name) in KEY_NAMES.iter().enumerate() {
            let shadowed = KEY_NAMES
                .iter()
                .position(|other| other.to_bytes().starts_with(name.to_bytes()));
            assert_eq!(shadowed, Some(sym), "{name:?} prefixes an earlier name");
        }
    }

    #[test]
    fn camel_case_becomes_lower_case_words() {
        assert_eq!(spaced_lowercase(c"PageUp"), "page up");
        assert_eq!(spaced_lowercase(c"Up"), "up");
        assert_eq!(spaced_lowercase(c"KP0"), "kp0");
        assert_eq!(spaced_lowercase(c"KPEnter"), "kpenter");
        assert_eq!(spaced_lowercase(c"DEL"), "del");
    }
}
