//! The tables a `.aff` file contributes to the language being built.
//!
//! These are the keywords whose payload is data rather than a setting: the
//! `REP` and `REPSAL` replacement pairs, the `SAL` sound-folding rules,
//! the `MAP` groups of near-equivalent characters, the
//! `CHECKCOMPOUNDPATTERN` pairs, and the free text `:spellinfo` shows.
//!
//! Several of them are only taken from the *first* `.aff` file of a run
//! that has them — a second file's `REP` table would otherwise be appended
//! to the first's rather than replacing it. `AffState` carries those
//! decisions as its `do_*` fields.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use crate::message_fmt::{c_str, msg_cstr};
use crate::smsg;
use crate::strings::find_char;
use crate::winlayer::Win;
use core::ffi::{CStr, c_char, c_int};
use std::ffi::CString;

use crate::mbyte::{char_at, char_len, cluster_len};
use crate::spell::spell_casefold;
use crate::types::RepItem;
use ::libc::{strcat, strcpy};

use super::aff::{AffState, first_byte, is_digit_byte, item_ptr};
use super::{MAXWLEN, SpellInfo};

/// Append `KEYWORD value` to the text `:spellinfo` shows.
///
/// # Safety
///
/// As [`handle_line`].
pub(super) unsafe fn append_info(spin: &mut SpellInfo, items: &[&CStr]) {
    let old = if spin.si_info.is_null() {
        0
    } else {
        // SAFETY: `si_info` is a NUL-terminated arena string.
        unsafe { cstr::bytes_at(spin.si_info) }.len()
    };
    let len = old + items[0].to_bytes().len() + items[1].to_bytes().len() + 3;
    let p = spin.si_arena.alloc_bytes(len, false);
    // SAFETY: `p` is `len` bytes, which is the old text, a newline, both
    // items and a space, plus the terminator.
    unsafe {
        if !spin.si_info.is_null() {
            strcpy(p, spin.si_info);
            strcat(p, c"\n".as_ptr());
        }
        strcat(p, item_ptr(items[0]));
        strcat(p, c" ".as_ptr());
        strcat(p, item_ptr(items[1]));
    }
    spin.si_info = p;
}

/// `CHECKCOMPOUNDPATTERN`: a pair of strings that may not meet at a
/// compound join, recorded once.
pub(super) fn add_comppat(spin: &mut SpellInfo, items: &[&CStr]) {
    let (a, b) = (items[1].to_bytes(), items[2].to_bytes());
    let pats = &mut spin.si_comppat;
    let known = pats
        .as_chunks::<2>()
        .0
        .iter()
        .any(|pair| &*pair[0] == a && &*pair[1] == b);
    if !known {
        pats.push(a.into());
        pats.push(b.into());
    }
}

/// A `REP`/`REPSAL` pair. `_` stands for a space in both halves.
///
/// # Safety
///
/// As [`handle_line`].
pub(super) unsafe fn add_rep_entry(
    spin: &mut SpellInfo,
    st: &AffState,
    items: &[&CStr],
    fname: *mut c_char,
    lnum: c_int,
) {
    if items.len() > 3 && !items[3].to_bytes().starts_with(b"#") {
        // SAFETY: the affix file's name, NUL-terminated.
        let (file, item) = (unsafe { c_str(fname) }, msg_cstr(items[3]));
        smsg!(0, "Trailing text in {file} line {lnum}: {item}");
    }
    // "REPSAL" has an S where "REP" has its terminator.
    let is_sal = items[0].to_bytes().get(3) == Some(&b'S');
    if !(if is_sal { st.do_repsal } else { st.do_rep }) {
        return;
    }
    // Both halves take a space wherever they spell one as `_`. Upstream
    // rewrites the item in place, stepping character by character so that a
    // `_` inside a multibyte character would be left alone -- which no byte
    // of one can be, since a continuation byte is never ASCII, so this is
    // the same substitution over a copy of the item.
    let unescaped = |item: &CStr| -> CString {
        let mut bytes = item.to_bytes().to_vec();
        for byte in &mut bytes {
            if *byte == b'_' {
                *byte = b' ';
            }
        }
        CString::new(bytes).expect("an item holds no NUL")
    };
    let (from, to) = (unescaped(items[1]), unescaped(items[2]));
    let out = if is_sal {
        &mut spin.si_repsal
    } else {
        &mut spin.si_rep
    };
    // SAFETY: two NUL-terminated strings of this call's own.
    unsafe { add_fromto(out, &from, &to) };
}

/// `MAP`: a group of characters that count as near-equivalent.
///
/// # Safety
///
/// As [`handle_line`].
pub(super) unsafe fn handle_map(
    spin: &mut SpellInfo,
    st: &mut AffState,
    items: &[&CStr],
    fname: *mut c_char,
    lnum: c_int,
) {
    if !st.found_map {
        // The first MAP line is the number of groups.
        st.found_map = true;
        // SAFETY: reading the locale table.
        if !unsafe { is_digit_byte(first_byte(items[1]) as c_char) } {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let fname = unsafe { c_str(fname) };
            smsg!(0, "Expected MAP count in {fname} line {}", lnum);
        }
        return;
    }
    if !st.do_mapline {
        return;
    }

    // A character may only appear in one group, and only once in it.
    let group = items[1].to_bytes();
    let mut at = 0;
    while at < group.len() {
        let c = char_at(&group[at..]);
        at += cluster_len(&group[at..]);
        if chars_of(&spin.si_map).any(|seen| seen == c) || find_char(&group[at..], c).is_some() {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let fname = unsafe { c_str(fname) };
            smsg!(0, "Duplicate character in MAP in {fname} line {}", lnum);
        }
    }
    spin.si_map.extend_from_slice(group);
    spin.si_map.push(b'/');
}

/// The characters `bytes` spells, in order.
fn chars_of(bytes: &[u8]) -> impl Iterator<Item = c_int> + '_ {
    let mut at = 0;
    core::iter::from_fn(move || {
        let rest = bytes.get(at..).filter(|r| !r.is_empty())?;
        at += char_len(rest);
        Some(char_at(rest))
    })
}

/// `SAL`: either a sound-folding setting or one folding rule.
///
/// # Safety
///
/// As [`handle_line`].
pub(super) unsafe fn handle_sal(spin: &mut SpellInfo, items: &[&CStr]) {
    let slot = match items[1].to_bytes() {
        b"followup" => Some(&mut spin.si_followup),
        b"collapse_result" => Some(&mut spin.si_collapse),
        b"remove_accents" => Some(&mut spin.si_rem_accents),
        _ => None,
    };
    if let Some(slot) = slot {
        *slot = sal_to_bool(items[2]) as c_int;
        return;
    }
    // "_" means the rule deletes what it matched.
    let to = if items[2] == c"_" { c"" } else { items[2] };
    // SAFETY: the items are live NUL-terminated strings.
    unsafe { add_fromto(&mut spin.si_sal, items[1], to) };
}

/// Add a case-folded from/to pair to one of the substitution tables.
///
/// # Safety
///
/// Main thread; the current window must be live.
pub(super) unsafe fn add_fromto(out: &mut Vec<RepItem>, from: &CStr, to: &CStr) {
    // SAFETY: the caller promises the strings; `word` is MAXWLEN, the
    // bound `spell_casefold` is given.
    let folded = |s: &CStr| -> Box<[u8]> {
        let mut word: [c_char; MAXWLEN] = [0; MAXWLEN];
        let (win, buf) = (Win::current(), word.as_mut_ptr());
        let len = s.to_bytes().len() as c_int;
        let _ = unsafe { spell_casefold(win, s.as_ptr(), len, buf, MAXWLEN as c_int) };
        unsafe { cstr::bytes_at(word.as_ptr()) }.into()
    };
    out.push(RepItem {
        from: folded(from),
        to: folded(to),
    });
}

/// `1` and `true` are the affirmative values a `SAL` setting takes.
pub(super) fn sal_to_bool(s: &CStr) -> bool {
    s == c"1" || s == c"true"
}
