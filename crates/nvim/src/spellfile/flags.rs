//! Affix flags: how a `.aff` file spells them, and what they turn into.
//!
//! A Hunspell affix file gives every affix, and several special properties,
//! a *flag*. A `.dic` entry then lists the flags its word carries. How a
//! flag is written depends on the file's `FLAG` line:
//!
//! - `AFT_CHAR` (the default) — one character per flag, run together.
//! - `AFT_LONG` — two characters per flag.
//! - `AFT_CAPLONG` — two characters when the first is `A`-`Z`, else one.
//! - `AFT_NUM` — a decimal number per flag, comma separated.
//!
//! [`get_affitem`] decodes one flag and advances past it; [`affitem2flag`]
//! decodes a whole item that should be exactly one flag;
//! [`flag_in_afflist`] answers whether a list contains a given flag without
//! decoding it into anything.
//!
//! A flag's value is the character, or for the two-character forms the
//! second character plus the first shifted left sixteen. Zero means "no
//! flag", so a numeric flag of literally zero is stored as [`ZERO_FLAG`]
//! instead.
//!
//! # Compound ids
//!
//! Compound flags do not survive into the `.spl` as flags. Each distinct
//! one is given a small id by [`process_compflags`], and the compound
//! *pattern* is rewritten in terms of those ids. Ids are handed out
//! downwards from 255 while postponed prefix ids go upwards from 0, and
//! [`check_renumber`] jumps both apart when they are about to collide, so
//! the two kinds never share a value.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::smsg_c;
use core::ffi::{c_char, c_int, c_uint};

use crate::ascii::ascii_isdigit;
use crate::charset::getdigits_int;
use crate::hashtab::{hash_add, hash_clear, hash_find, hash_removed};
use crate::mbyte::mb_ptr2char_adv;
use crate::memory::{xfree, xmemcpyz};
use crate::os::cshim::{gettext, gettext_ptr, memmove};
use crate::strings::vim_strchr;
use crate::types::{NUL, hashitem_T, size_t, uint8_t};
use ::libc::{strcat, strcpy, strlen};

use super::{
    AFT_CAPLONG, AFT_CHAR, AFT_LONG, AFT_NUM, ZERO_FLAG, affentry_T, afffile_T, affheader_T,
    compitem_T, e_affname, spellinfo_T, vim_regfree,
};

/// Decode one flag and advance `pp` past it. Returns 0 when there is none.
///
/// # Safety
///
/// `pp` must point at a pointer into a NUL-terminated string.
pub(super) unsafe fn get_affitem(flagtype: c_int, pp: *mut *mut c_char) -> c_uint {
    // SAFETY: the caller promises the string; each branch advances `pp` by
    // at most what it read.
    if flagtype == AFT_NUM {
        if !ascii_isdigit(unsafe { **pp } as c_int) {
            // Not a number at all; step over the offending byte so the
            // caller makes progress.
            unsafe { *pp = (*pp).add(1) };
            return 0;
        }
        let mut res = unsafe { getdigits_int(pp, true, 0) };
        if res == 0 {
            // Zero would read as "no flag", so it gets its own value.
            res = ZERO_FLAG;
        }
        return res as c_uint;
    }

    let mut res = unsafe { mb_ptr2char_adv(pp.cast::<*const c_char>()) };
    // Two-character flags: always for LONG, and for CAPLONG only when
    // the first character is upper case ASCII.
    if flagtype == AFT_LONG
        || (flagtype == AFT_CAPLONG && res >= b'A' as c_int && res <= b'Z' as c_int)
    {
        if unsafe { **pp } as c_int == NUL {
            return 0;
        }
        res = unsafe { mb_ptr2char_adv(pp.cast::<*const c_char>()) } + (res << 16);
    }
    res as c_uint
}

/// Decode an item that should hold exactly one flag, complaining if it
/// does not.
///
/// # Safety
///
/// `item` and `fname` must be NUL-terminated.
pub(super) unsafe fn affitem2flag(
    flagtype: c_int,
    item: *mut c_char,
    fname: *mut c_char,
    lnum: c_int,
) -> c_uint {
    // SAFETY: the caller promises the strings.
    let mut p = item;
    let res = unsafe { get_affitem(flagtype, &raw mut p) };
    if res == 0 {
        let msg = if flagtype == AFT_NUM {
            c"Flag is not a number in %s line %d: %s"
        } else {
            c"Illegal flag in %s line %d: %s"
        };
        unsafe { smsg_c!(0, gettext(msg).as_ptr(), fname, lnum, item) };
    }
    // Anything left over means the item was more than one flag.
    if unsafe { *p } as c_int != NUL {
        unsafe { smsg_c!(0, gettext_ptr(e_affname.get()), fname, lnum, item) };
        return 0;
    }
    res
}

/// Is `flag` one of the flags in `afflist`?
///
/// # Safety
///
/// `afflist` must be NUL-terminated.
pub(super) unsafe fn flag_in_afflist(flagtype: c_int, afflist: *mut c_char, flag: c_uint) -> bool {
    // SAFETY: the caller promises the string; every walk stops at its NUL.
    match flagtype {
        AFT_CHAR => !unsafe { vim_strchr(afflist, flag as c_int) }.is_null(),
        AFT_LONG | AFT_CAPLONG => {
            let mut p = afflist;
            while unsafe { *p } as c_int != NUL {
                let mut n =
                    unsafe { mb_ptr2char_adv((&raw mut p).cast::<*const c_char>()) } as c_uint;
                if (flagtype == AFT_LONG || (n >= b'A' as c_uint && n <= b'Z' as c_uint))
                    && unsafe { *p } as c_int != NUL
                {
                    n = (unsafe { mb_ptr2char_adv((&raw mut p).cast::<*const c_char>()) }
                        as c_uint)
                        .wrapping_add(n << 16);
                }
                if n == flag {
                    return true;
                }
            }
            false
        }
        AFT_NUM => {
            let mut p = afflist;
            while unsafe { *p } as c_int != NUL {
                let digits = unsafe { getdigits_int(&raw mut p, true, 0) };
                debug_assert!(digits >= 0);
                let mut n = digits as c_uint;
                if n == 0 {
                    n = ZERO_FLAG as c_uint;
                }
                if n == flag {
                    return true;
                }
                // Step over the comma, or the byte that was not a
                // digit, so the walk terminates.
                if unsafe { *p } as c_int != NUL {
                    p = unsafe { p.add(1) };
                }
            }
            false
        }
        _ => false,
    }
}

/// Strip `COMPOUNDPERMITFLAG` and `COMPOUNDFORBIDFLAG` out of an affix
/// entry's own flags, recording them as booleans on the entry instead.
///
/// # Safety
///
/// `entry` and `affile` must be live, and `ae_flags` NUL-terminated.
pub(super) unsafe fn aff_process_flags(affile: *mut afffile_T, entry: *mut affentry_T) {
    // SAFETY: the caller promises both; the memmove closes a gap inside one
    // string, so source and destination share an allocation.
    if unsafe { (*entry).ae_flags }.is_null()
        || (unsafe { (*affile).af_compforbid } == 0 && unsafe { (*affile).af_comppermit } == 0)
    {
        return;
    }
    let mut p = unsafe { (*entry).ae_flags };
    while unsafe { *p } as c_int != NUL {
        let prevp = p;
        let flag = unsafe { get_affitem((*affile).af_flagtype, &raw mut p) };
        if flag == unsafe { (*affile).af_comppermit } || flag == unsafe { (*affile).af_compforbid }
        {
            // Remove the flag from the list and stay put, so the next
            // flag is read from where this one was.
            unsafe { memmove(prevp.cast(), p.cast(), strlen(p) + 1) };
            p = prevp;
            if flag == unsafe { (*affile).af_comppermit } {
                unsafe { (*entry).ae_comppermit = 1 };
            } else {
                unsafe { (*entry).ae_compforbid = 1 };
            }
        }
        if unsafe { (*affile).af_flagtype } == AFT_NUM && unsafe { *p } as c_int == b',' as c_int {
            p = unsafe { p.add(1) };
        }
    }
    if unsafe { *(*entry).ae_flags } as c_int == NUL {
        unsafe { (*entry).ae_flags = core::ptr::null_mut() };
    }
}

/// Rewrite the compound pattern in terms of one-byte ids, appending it to
/// whatever a previous `.aff` file already contributed.
///
/// # Safety
///
/// `compflags` must be NUL-terminated and `aff` live.
pub(super) unsafe fn process_compflags(
    spin: *mut spellinfo_T,
    aff: *mut afffile_T,
    compflags: *mut c_char,
) {
    // SAFETY: the destination is sized for the old pattern, a separator and
    // the new one, and each flag turns into at most one byte.
    let mut len = unsafe { strlen(compflags) } + 1;
    if !unsafe { (*spin).si_compflags }.is_null() {
        len += unsafe { strlen((*spin).si_compflags) } + 1;
    }
    let dest = unsafe { (*spin).si_arena.alloc_bytes(len, false) };
    if !unsafe { (*spin).si_compflags }.is_null() {
        unsafe { strcpy(dest, (*spin).si_compflags) };
        unsafe { strcat(dest, c"/".as_ptr()) };
    }
    unsafe { (*spin).si_compflags = dest };

    let mut tp = unsafe { dest.cast::<uint8_t>().add(strlen(dest)) };
    let mut p = compflags;
    let mut key: [c_char; 17] = [0; 17];
    while unsafe { *p } as c_int != NUL {
        // Pattern punctuation passes straight through.
        if !unsafe { vim_strchr(c"/?*+[]".as_ptr(), *p as uint8_t as c_int) }.is_null() {
            unsafe { *tp = *p as uint8_t };
            tp = unsafe { tp.add(1) };
            p = unsafe { p.add(1) };
            continue;
        }

        let prevp = p;
        let flag = unsafe { get_affitem((*aff).af_flagtype, &raw mut p) };
        if flag != 0 {
            unsafe {
                xmemcpyz(
                    key.as_mut_ptr().cast(),
                    prevp.cast(),
                    p.offset_from(prevp) as size_t,
                )
            };
            let hi: *mut hashitem_T =
                unsafe { hash_find(&raw mut (*aff).af_comp, key.as_mut_ptr()) };
            let id = if !unsafe { (*hi).hi_key }.is_null()
                && unsafe { (*hi).hi_key } != (&raw const hash_removed).cast_mut().cast()
            {
                unsafe { (*compitem_T::of_key((*hi).hi_key)).ci_newID }
            } else {
                let ci = unsafe { (*spin).si_arena.alloc::<compitem_T>() };
                unsafe { strcpy(compitem_T::key(ci), key.as_mut_ptr()) };
                unsafe { (*ci).ci_flag = flag };
                // Ids count downwards, skipping any byte that would be
                // meaningful in the pattern this becomes.
                let id = loop {
                    unsafe { check_renumber(spin) };
                    let id = unsafe { (*spin).si_newcompID };
                    unsafe { (*spin).si_newcompID -= 1 };
                    if unsafe { vim_strchr(c"/?*+[]\\-^".as_ptr(), id) }.is_null() {
                        break id;
                    }
                };
                unsafe { (*ci).ci_newID = id };
                unsafe { hash_add(&raw mut (*aff).af_comp, compitem_T::key(ci)) };
                id
            };
            unsafe { *tp = id as uint8_t };
            tp = unsafe { tp.add(1) };
        }
        if unsafe { (*aff).af_flagtype } == AFT_NUM && unsafe { *p } as c_int == b',' as c_int {
            p = unsafe { p.add(1) };
        }
    }
    unsafe { *tp = NUL as uint8_t };
}

/// Move the prefix and compound id counters apart before they meet.
///
/// Prefix ids count up from zero and compound ids down from 255. When both
/// reach the same value in the lower half, the split is redrawn at 127/255
/// so each kind keeps a range to itself.
///
/// # Safety
///
/// `spin` must be live.
pub(super) unsafe fn check_renumber(spin: *mut spellinfo_T) {
    // SAFETY: the caller promises `spin`.
    if unsafe { (*spin).si_newprefID } == unsafe { (*spin).si_newcompID }
        && unsafe { (*spin).si_newcompID } < 128
    {
        unsafe { (*spin).si_newprefID = 127 };
        unsafe { (*spin).si_newcompID = 255 };
    }
}

/// Release what an affix file owns outside the arena: the encoding name and
/// every compiled condition.
///
/// # Safety
///
/// `aff` must be a live affix file that is not used again.
pub(super) unsafe fn spell_free_aff(aff: *mut afffile_T) {
    // SAFETY: the caller promises the affix file; the regexps below are the
    // only heap allocations the entries own.
    unsafe { xfree((*aff).af_enc.cast()) };

    for ht in [unsafe { &raw mut (*aff).af_pref }, unsafe {
        &raw mut (*aff).af_suff
    }] {
        let mut todo = unsafe { (*ht).ht_used } as c_int;
        let mut hi: *mut hashitem_T = unsafe { (*ht).ht_array };
        while todo > 0 {
            if !unsafe { (*hi).hi_key }.is_null()
                && unsafe { (*hi).hi_key } != (&raw const hash_removed).cast_mut().cast()
            {
                todo -= 1;
                let ah = unsafe { affheader_T::of_key((*hi).hi_key) };
                let mut ae = unsafe { (*ah).ah_first };
                while !ae.is_null() {
                    unsafe { vim_regfree((*ae).ae_prog) };
                    ae = unsafe { (*ae).ae_next };
                }
            }
            hi = unsafe { hi.add(1) };
        }
    }

    unsafe { hash_clear(&raw mut (*aff).af_pref) };
    unsafe { hash_clear(&raw mut (*aff).af_suff) };
    unsafe { hash_clear(&raw mut (*aff).af_comp) };
}
