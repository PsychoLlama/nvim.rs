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

use crate::smsg_c;
use core::ffi::{CStr, c_char, c_int};

use crate::garray::{ga_append, ga_append_via_ptr, ga_concat, ga_grow};
use crate::main::curwin;
use crate::mbyte::{mb_ptr2char_adv, utfc_ptr2len};
use crate::os::libc::{gettext, strcat, strcmp, strcpy, strlen};
use crate::spell::spell_casefold;
use crate::strings::vim_strchr;
use crate::types::{fromto_T, garray_T};

use super::aff::{AffState, is_digit_byte};
use super::{MAXWLEN, NUL, e_afftrailing, spellinfo_T};

/// Append `KEYWORD value` to the text `:spellinfo` shows.
///
/// # Safety
///
/// As [`handle_line`].
pub unsafe fn append_info(spin: *mut spellinfo_T, items: &[*mut c_char]) {
    // SAFETY: the buffer is sized for the old text, a newline, both items
    // and a space, plus the terminator.
    unsafe {
        let old = if (*spin).si_info.is_null() {
            0
        } else {
            strlen((*spin).si_info)
        };
        let len = old + strlen(items[0]) + strlen(items[1]) + 3;
        let p = (*spin).si_arena.alloc_bytes(len, false);
        if !(*spin).si_info.is_null() {
            strcpy(p, (*spin).si_info);
            strcat(p, c"\n".as_ptr());
        }
        strcat(p, items[0]);
        strcat(p, c" ".as_ptr());
        strcat(p, items[1]);
        (*spin).si_info = p;
    }
}

/// `CHECKCOMPOUNDPATTERN`: a pair of strings that may not meet at a
/// compound join, recorded once.
///
/// # Safety
///
/// As [`handle_line`].
pub unsafe fn add_comppat(spin: *mut spellinfo_T, items: &[*mut c_char]) {
    // SAFETY: `ga_grow(2)` makes room for the pair appended below.
    unsafe {
        let gap = &raw mut (*spin).si_comppat;
        let mut i = 0;
        while i < (*gap).ga_len - 1 {
            let entries = (*gap).ga_data.cast::<*mut c_char>();
            if strcmp(*entries.offset(i as isize), items[1]) == 0
                && strcmp(*entries.offset(i as isize + 1), items[2]) == 0
            {
                break;
            }
            i += 2;
        }
        if i >= (*gap).ga_len {
            ga_grow(gap, 2);
            for item in &items[1..3] {
                let entries = (*gap).ga_data.cast::<*mut c_char>();
                *entries.offset((*gap).ga_len as isize) = (*spin).si_arena.save_str(*item);
                (*gap).ga_len += 1;
            }
        }
    }
}

/// A `REP`/`REPSAL` pair. `_` stands for a space in both halves.
///
/// # Safety
///
/// As [`handle_line`].
pub unsafe fn add_rep_entry(
    spin: *mut spellinfo_T,
    st: &AffState,
    items: &[*mut c_char],
    fname: *mut c_char,
    lnum: c_int,
) {
    // SAFETY: the caller promises the items; the substitution is in place
    // and replaces one byte with one byte.
    unsafe {
        if items.len() > 3 && *items[3] as c_int != b'#' as c_int {
            smsg_c!(0, gettext(e_afftrailing.get()), fname, lnum, items[3]);
        }
        // "REPSAL" has an S where "REP" has its terminator.
        let is_sal = *items[0].add(3) as c_int == b'S' as c_int;
        if !(if is_sal { st.do_repsal } else { st.do_rep }) {
            return;
        }
        for &item in &items[1..3] {
            let mut p = item;
            while *p as c_int != NUL {
                if *p as c_int == b'_' as c_int {
                    *p = b' ' as c_char;
                }
                p = p.add(utfc_ptr2len(p) as usize);
            }
        }
        let gap = if is_sal {
            &raw mut (*spin).si_repsal
        } else {
            &raw mut (*spin).si_rep
        };
        add_fromto(spin, gap, items[1], items[2]);
    }
}

/// `MAP`: a group of characters that count as near-equivalent.
///
/// # Safety
///
/// As [`handle_line`].
pub unsafe fn handle_map(
    spin: *mut spellinfo_T,
    st: &mut AffState,
    items: &[*mut c_char],
    fname: *mut c_char,
    lnum: c_int,
) {
    // SAFETY: the caller promises the items.
    unsafe {
        if !st.found_map {
            // The first MAP line is the number of groups.
            st.found_map = true;
            if !is_digit_byte(*items[1]) {
                smsg_c!(
                    0,
                    gettext(c"Expected MAP count in %s line %d".as_ptr()),
                    fname,
                    lnum,
                );
            }
            return;
        }
        if !st.do_mapline {
            return;
        }

        // A character may only appear in one group, and only once in it.
        let mut p = items[1];
        while *p as c_int != NUL {
            let c = mb_ptr2char_adv((&raw mut p).cast::<*const c_char>());
            if ((*spin).si_map.ga_len > 0
                && !vim_strchr((*spin).si_map.ga_data.cast::<c_char>(), c).is_null())
                || !vim_strchr(p, c).is_null()
            {
                smsg_c!(
                    0,
                    gettext(c"Duplicate character in MAP in %s line %d".as_ptr()),
                    fname,
                    lnum,
                );
            }
        }
        ga_concat(&raw mut (*spin).si_map, items[1]);
        ga_append(&raw mut (*spin).si_map, b'/');
    }
}

/// `SAL`: either a sound-folding setting or one folding rule.
///
/// # Safety
///
/// As [`handle_line`].
pub unsafe fn handle_sal(spin: *mut spellinfo_T, items: &[*mut c_char]) {
    // SAFETY: the caller promises the items.
    unsafe {
        let settings: [(&CStr, *mut c_int); 3] = [
            (c"followup", &raw mut (*spin).si_followup),
            (c"collapse_result", &raw mut (*spin).si_collapse),
            (c"remove_accents", &raw mut (*spin).si_rem_accents),
        ];
        for (name, slot) in settings {
            if strcmp(items[1], name.as_ptr()) == 0 {
                *slot = sal_to_bool(items[2]) as c_int;
                return;
            }
        }
        // "_" means the rule deletes what it matched.
        let to = if strcmp(items[2], c"_".as_ptr()) == 0 {
            c"".as_ptr().cast_mut()
        } else {
            items[2]
        };
        add_fromto(spin, &raw mut (*spin).si_sal, items[1], to);
    }
}

/// Add a case-folded from/to pair to one of the substitution tables.
///
/// # Safety
///
/// `from` and `to` must be NUL-terminated and `gap` a `fromto_T` array.
pub unsafe fn add_fromto(
    spin: *mut spellinfo_T,
    gap: *mut garray_T,
    from: *mut c_char,
    to: *mut c_char,
) {
    // SAFETY: `word` is MAXWLEN, the bound spell_casefold is given.
    unsafe {
        let ftp = ga_append_via_ptr(gap, core::mem::size_of::<fromto_T>()).cast::<fromto_T>();
        let mut word: [c_char; MAXWLEN] = [0; MAXWLEN];

        spell_casefold(
            curwin.get(),
            from,
            strlen(from) as c_int,
            word.as_mut_ptr(),
            MAXWLEN as c_int,
        );
        (*ftp).ft_from = (*spin).si_arena.save_str(word.as_mut_ptr());
        spell_casefold(
            curwin.get(),
            to,
            strlen(to) as c_int,
            word.as_mut_ptr(),
            MAXWLEN as c_int,
        );
        (*ftp).ft_to = (*spin).si_arena.save_str(word.as_mut_ptr());
    }
}

/// `1` and `true` are the affirmative values a `SAL` setting takes.
///
/// # Safety
///
/// `s` must be NUL-terminated.
pub unsafe fn sal_to_bool(s: *mut c_char) -> bool {
    // SAFETY: the caller promises the string.
    unsafe { strcmp(s, c"1".as_ptr()) == 0 || strcmp(s, c"true".as_ptr()) == 0 }
}
