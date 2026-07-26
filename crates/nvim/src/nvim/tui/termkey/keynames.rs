#![forbid(unsafe_code)]

//! The names of the symbolic keys.

use crate::src::nvim::types::TermKeySym;
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

/// Find the symbol whose name starts `text`, and how many bytes it took.
///
/// The first symbol in numeric order whose name is a prefix of `text` wins, so
/// "DownMore" is Down followed by "More".
pub fn lookup(text: &[u8]) -> Option<(TermKeySym, usize)> {
    KEY_NAMES
        .iter()
        .position(|name| text.starts_with(name.to_bytes()))
        .map(|sym| (sym as TermKeySym, KEY_NAMES[sym].count_bytes()))
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
    fn lookup_takes_the_longest_leading_name_it_knows() {
        assert_eq!(lookup(b"Space"), Some((5, 5)));
        assert_eq!(lookup(b"Up"), Some((7, 2)));
        assert_eq!(lookup(b"DownMore"), Some((8, 4)));
        assert_eq!(lookup(b"SomeUnknownKey"), None);
    }

    #[test]
    fn names_are_unique_and_in_symbol_order() {
        let mut sorted = KEY_NAMES.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), KEY_NAMES.len());
        for (sym, name) in KEY_NAMES.iter().enumerate() {
            assert_eq!(lookup(name.to_bytes()).map(|found| found.0), {
                // A name that is a prefix of an earlier name resolves to that
                // one instead; none is, so every name finds itself.
                Some(sym as TermKeySym)
            });
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
