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

use crate::cstr;
use crate::message_fmt::c_str;
use crate::smsg;
use core::ffi::{c_char, c_int};

use crate::main::curwin;
use crate::mbyte::{char_at, char_len, mb_ptr2char_adv, utfc_ptr2len};
use crate::spell::spell_casefold;
use crate::strings::vim_strchr;
use crate::types::{NUL, RepItem};
use ::libc::{strcat, strcpy};

use super::aff::{AffState, is_digit_byte};
use super::{MAXWLEN, spellinfo_T};

/// Append `KEYWORD value` to the text `:spellinfo` shows.
///
/// # Safety
///
/// As [`handle_line`].
pub(super) unsafe fn append_info(spin: &mut spellinfo_T, items: &[*mut c_char]) {
    // SAFETY: the buffer is sized for the old text, a newline, both items
    // and a space, plus the terminator.
    let old = if spin.si_info.is_null() {
        0
    } else {
        // SAFETY: `si_info` is a NUL-terminated arena string.
        unsafe { cstr::bytes_at(spin.si_info) }.len()
    };
    let len = old
        + unsafe { cstr::bytes_at(items[0]) }.len()
        + unsafe { cstr::bytes_at(items[1]) }.len()
        + 3;
    let p = spin.si_arena.alloc_bytes(len, false);
    // SAFETY: `p` is `len` bytes, which is what the pieces below need.
    unsafe {
        if !spin.si_info.is_null() {
            strcpy(p, spin.si_info);
            strcat(p, c"\n".as_ptr());
        }
        strcat(p, items[0]);
        strcat(p, c" ".as_ptr());
        strcat(p, items[1]);
    }
    spin.si_info = p;
}

/// `CHECKCOMPOUNDPATTERN`: a pair of strings that may not meet at a
/// compound join, recorded once.
///
/// # Safety
///
/// As [`handle_line`].
pub(super) unsafe fn add_comppat(spin: &mut spellinfo_T, items: &[*mut c_char]) {
    // SAFETY: the caller promises the items.
    let (a, b) = unsafe { (cstr::bytes_at(items[1]), cstr::bytes_at(items[2])) };
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
    spin: &mut spellinfo_T,
    st: &AffState,
    items: &[*mut c_char],
    fname: *mut c_char,
    lnum: c_int,
) {
    // SAFETY: the caller promises the items; the substitution is in place
    // and replaces one byte with one byte.
    if items.len() > 3 && unsafe { *items[3] } as c_int != b'#' as c_int {
        // SAFETY: the affix file's name and the trailing item.
        let (file, item) = unsafe { (c_str(fname), c_str(items[3])) };
        smsg!(0, "Trailing text in {file} line {lnum}: {item}");
    }
    // "REPSAL" has an S where "REP" has its terminator.
    let is_sal = unsafe { *items[0].add(3) } as c_int == b'S' as c_int;
    if !(if is_sal { st.do_repsal } else { st.do_rep }) {
        return;
    }
    for &item in &items[1..3] {
        let mut p = item;
        while unsafe { *p } as c_int != NUL {
            if unsafe { *p } as c_int == b'_' as c_int {
                unsafe { *p = b' ' as c_char };
            }
            p = unsafe { p.add(utfc_ptr2len(p) as usize) };
        }
    }
    let out = if is_sal {
        &mut spin.si_repsal
    } else {
        &mut spin.si_rep
    };
    // SAFETY: the caller promises the items.
    unsafe { add_fromto(out, items[1], items[2]) };
}

/// `MAP`: a group of characters that count as near-equivalent.
///
/// # Safety
///
/// As [`handle_line`].
pub(super) unsafe fn handle_map(
    spin: &mut spellinfo_T,
    st: &mut AffState,
    items: &[*mut c_char],
    fname: *mut c_char,
    lnum: c_int,
) {
    // SAFETY: the caller promises the items.
    if !st.found_map {
        // The first MAP line is the number of groups.
        st.found_map = true;
        if !unsafe { is_digit_byte(*items[1]) } {
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
    let mut p = items[1];
    while unsafe { *p } as c_int != NUL {
        let c = unsafe { mb_ptr2char_adv((&raw mut p).cast::<*const c_char>()) };
        // The groups collected so far are bytes rather than a C string now,
        // so the membership test decodes them instead of `vim_strchr`.
        if chars_of(&spin.si_map).any(|seen| seen == c) || !unsafe { vim_strchr(p, c) }.is_null() {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let fname = unsafe { c_str(fname) };
            smsg!(0, "Duplicate character in MAP in {fname} line {}", lnum);
        }
    }
    // SAFETY: the caller promises the item.
    spin.si_map
        .extend_from_slice(unsafe { cstr::bytes_at(items[1]) });
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
pub(super) unsafe fn handle_sal(spin: &mut spellinfo_T, items: &[*mut c_char]) {
    // SAFETY: the caller promises the items.
    // SAFETY: the caller promises the items.
    let name = unsafe { cstr::bytes_at(items[1]) };
    let slot = match name {
        b"followup" => Some(&mut spin.si_followup),
        b"collapse_result" => Some(&mut spin.si_collapse),
        b"remove_accents" => Some(&mut spin.si_rem_accents),
        _ => None,
    };
    if let Some(slot) = slot {
        // SAFETY: as above.
        *slot = unsafe { sal_to_bool(items[2]) } as c_int;
        return;
    }
    // "_" means the rule deletes what it matched.
    let to = if unsafe { cstr::eq_bytes(items[2], b"_") } {
        c"".as_ptr().cast_mut()
    } else {
        items[2]
    };
    unsafe { add_fromto(&mut spin.si_sal, items[1], to) };
}

/// Add a case-folded from/to pair to one of the substitution tables.
///
/// # Safety
///
/// `from` and `to` must be NUL-terminated.
pub(super) unsafe fn add_fromto(out: &mut Vec<RepItem>, from: *mut c_char, to: *mut c_char) {
    // SAFETY: the caller promises the strings; `word` is MAXWLEN, the
    // bound `spell_casefold` is given.
    let folded = |s: *mut c_char| -> Box<[u8]> {
        let mut word: [c_char; MAXWLEN] = [0; MAXWLEN];
        let (win, buf) = (curwin.get(), word.as_mut_ptr());
        let len = unsafe { cstr::bytes_at(s) }.len() as c_int;
        let _ = unsafe { spell_casefold(win, s, len, buf, MAXWLEN as c_int) };
        unsafe { cstr::bytes_at(word.as_ptr()) }.into()
    };
    out.push(RepItem {
        from: folded(from),
        to: folded(to),
    });
}

/// `1` and `true` are the affirmative values a `SAL` setting takes.
///
/// # Safety
///
/// `s` must be NUL-terminated.
pub(super) unsafe fn sal_to_bool(s: *mut c_char) -> bool {
    // SAFETY: the caller promises the string.
    unsafe { cstr::eq_bytes(s, b"1") || cstr::eq_bytes(s, b"true") }
}
