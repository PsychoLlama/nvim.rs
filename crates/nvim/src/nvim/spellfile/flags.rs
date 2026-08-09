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

use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::charset::getdigits_int;
use crate::src::nvim::hashtab::{hash_add, hash_clear, hash_find, hash_removed};
use crate::src::nvim::mbyte::mb_ptr2char_adv;
use crate::src::nvim::memory::{xfree, xmemcpyz};
use crate::src::nvim::os::libc::{gettext, memmove, strcat, strcpy, strlen};
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::types::{hashitem_T, size_t, uint8_t};

use super::{
    AFT_CAPLONG, AFT_CHAR, AFT_LONG, AFT_NUM, NUL, ZERO_FLAG, affentry_T, afffile_T, affheader_T,
    compitem_T, e_affname, spellinfo_T, vim_regfree,
};

/// Decode one flag and advance `pp` past it. Returns 0 when there is none.
///
/// # Safety
///
/// `pp` must point at a pointer into a NUL-terminated string.
pub unsafe fn get_affitem(flagtype: c_int, pp: *mut *mut c_char) -> c_uint {
    // SAFETY: the caller promises the string; each branch advances `pp` by
    // at most what it read.
    unsafe {
        if flagtype == AFT_NUM {
            if !ascii_isdigit(**pp as c_int) {
                // Not a number at all; step over the offending byte so the
                // caller makes progress.
                *pp = (*pp).add(1);
                return 0;
            }
            let mut res = getdigits_int(pp, true, 0);
            if res == 0 {
                // Zero would read as "no flag", so it gets its own value.
                res = ZERO_FLAG;
            }
            return res as c_uint;
        }

        let mut res = mb_ptr2char_adv(pp.cast::<*const c_char>());
        // Two-character flags: always for LONG, and for CAPLONG only when
        // the first character is upper case ASCII.
        if flagtype == AFT_LONG
            || (flagtype == AFT_CAPLONG && res >= b'A' as c_int && res <= b'Z' as c_int)
        {
            if **pp as c_int == NUL {
                return 0;
            }
            res = mb_ptr2char_adv(pp.cast::<*const c_char>()) + (res << 16);
        }
        res as c_uint
    }
}

/// Decode an item that should hold exactly one flag, complaining if it
/// does not.
///
/// # Safety
///
/// `item` and `fname` must be NUL-terminated.
pub unsafe fn affitem2flag(
    flagtype: c_int,
    item: *mut c_char,
    fname: *mut c_char,
    lnum: c_int,
) -> c_uint {
    // SAFETY: the caller promises the strings.
    unsafe {
        let mut p = item;
        let res = get_affitem(flagtype, &raw mut p);
        if res == 0 {
            let msg = if flagtype == AFT_NUM {
                c"Flag is not a number in %s line %d: %s"
            } else {
                c"Illegal flag in %s line %d: %s"
            };
            smsg_c!(0, gettext(msg.as_ptr()), fname, lnum, item);
        }
        // Anything left over means the item was more than one flag.
        if *p as c_int != NUL {
            smsg_c!(0, gettext(e_affname.get()), fname, lnum, item);
            return 0;
        }
        res
    }
}

/// Is `flag` one of the flags in `afflist`?
///
/// # Safety
///
/// `afflist` must be NUL-terminated.
pub unsafe fn flag_in_afflist(flagtype: c_int, afflist: *mut c_char, flag: c_uint) -> bool {
    // SAFETY: the caller promises the string; every walk stops at its NUL.
    unsafe {
        match flagtype {
            AFT_CHAR => !vim_strchr(afflist, flag as c_int).is_null(),
            AFT_LONG | AFT_CAPLONG => {
                let mut p = afflist;
                while *p as c_int != NUL {
                    let mut n = mb_ptr2char_adv((&raw mut p).cast::<*const c_char>()) as c_uint;
                    if (flagtype == AFT_LONG || (n >= b'A' as c_uint && n <= b'Z' as c_uint))
                        && *p as c_int != NUL
                    {
                        n = (mb_ptr2char_adv((&raw mut p).cast::<*const c_char>()) as c_uint)
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
                while *p as c_int != NUL {
                    let digits = getdigits_int(&raw mut p, true, 0);
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
                    if *p as c_int != NUL {
                        p = p.add(1);
                    }
                }
                false
            }
            _ => false,
        }
    }
}

/// Strip `COMPOUNDPERMITFLAG` and `COMPOUNDFORBIDFLAG` out of an affix
/// entry's own flags, recording them as booleans on the entry instead.
///
/// # Safety
///
/// `entry` and `affile` must be live, and `ae_flags` NUL-terminated.
pub unsafe fn aff_process_flags(affile: *mut afffile_T, entry: *mut affentry_T) {
    // SAFETY: the caller promises both; the memmove closes a gap inside one
    // string, so source and destination share an allocation.
    unsafe {
        if (*entry).ae_flags.is_null()
            || ((*affile).af_compforbid == 0 && (*affile).af_comppermit == 0)
        {
            return;
        }
        let mut p = (*entry).ae_flags;
        while *p as c_int != NUL {
            let prevp = p;
            let flag = get_affitem((*affile).af_flagtype, &raw mut p);
            if flag == (*affile).af_comppermit || flag == (*affile).af_compforbid {
                // Remove the flag from the list and stay put, so the next
                // flag is read from where this one was.
                memmove(prevp.cast(), p.cast(), strlen(p) + 1);
                p = prevp;
                if flag == (*affile).af_comppermit {
                    (*entry).ae_comppermit = 1;
                } else {
                    (*entry).ae_compforbid = 1;
                }
            }
            if (*affile).af_flagtype == AFT_NUM && *p as c_int == b',' as c_int {
                p = p.add(1);
            }
        }
        if *(*entry).ae_flags as c_int == NUL {
            (*entry).ae_flags = core::ptr::null_mut();
        }
    }
}

/// Rewrite the compound pattern in terms of one-byte ids, appending it to
/// whatever a previous `.aff` file already contributed.
///
/// # Safety
///
/// `compflags` must be NUL-terminated and `aff` live.
pub unsafe fn process_compflags(
    spin: *mut spellinfo_T,
    aff: *mut afffile_T,
    compflags: *mut c_char,
) {
    // SAFETY: the destination is sized for the old pattern, a separator and
    // the new one, and each flag turns into at most one byte.
    unsafe {
        let mut len = strlen(compflags) + 1;
        if !(*spin).si_compflags.is_null() {
            len += strlen((*spin).si_compflags) + 1;
        }
        let dest = (*spin).si_arena.alloc_bytes(len, false);
        if !(*spin).si_compflags.is_null() {
            strcpy(dest, (*spin).si_compflags);
            strcat(dest, c"/".as_ptr());
        }
        (*spin).si_compflags = dest;

        let mut tp = dest.cast::<uint8_t>().add(strlen(dest));
        let mut p = compflags;
        let mut key: [c_char; 17] = [0; 17];
        while *p as c_int != NUL {
            // Pattern punctuation passes straight through.
            if !vim_strchr(c"/?*+[]".as_ptr(), *p as uint8_t as c_int).is_null() {
                *tp = *p as uint8_t;
                tp = tp.add(1);
                p = p.add(1);
                continue;
            }

            let prevp = p;
            let flag = get_affitem((*aff).af_flagtype, &raw mut p);
            if flag != 0 {
                xmemcpyz(
                    key.as_mut_ptr().cast(),
                    prevp.cast(),
                    p.offset_from(prevp) as size_t,
                );
                let hi: *mut hashitem_T = hash_find(&raw mut (*aff).af_comp, key.as_mut_ptr());
                let id = if !(*hi).hi_key.is_null()
                    && (*hi).hi_key != (&raw const hash_removed).cast_mut().cast()
                {
                    (*(*hi).hi_key.cast::<compitem_T>()).ci_newID
                } else {
                    let ci = (*spin).si_arena.alloc::<compitem_T>();
                    strcpy((&raw mut (*ci).ci_key).cast::<c_char>(), key.as_mut_ptr());
                    (*ci).ci_flag = flag;
                    // Ids count downwards, skipping any byte that would be
                    // meaningful in the pattern this becomes.
                    let id = loop {
                        check_renumber(spin);
                        let id = (*spin).si_newcompID;
                        (*spin).si_newcompID -= 1;
                        if vim_strchr(c"/?*+[]\\-^".as_ptr(), id).is_null() {
                            break id;
                        }
                    };
                    (*ci).ci_newID = id;
                    hash_add(
                        &raw mut (*aff).af_comp,
                        (&raw mut (*ci).ci_key).cast::<c_char>(),
                    );
                    id
                };
                *tp = id as uint8_t;
                tp = tp.add(1);
            }
            if (*aff).af_flagtype == AFT_NUM && *p as c_int == b',' as c_int {
                p = p.add(1);
            }
        }
        *tp = NUL as uint8_t;
    }
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
pub unsafe fn check_renumber(spin: *mut spellinfo_T) {
    // SAFETY: the caller promises `spin`.
    unsafe {
        if (*spin).si_newprefID == (*spin).si_newcompID && (*spin).si_newcompID < 128 {
            (*spin).si_newprefID = 127;
            (*spin).si_newcompID = 255;
        }
    }
}

/// Release what an affix file owns outside the arena: the encoding name and
/// every compiled condition.
///
/// # Safety
///
/// `aff` must be a live affix file that is not used again.
pub unsafe fn spell_free_aff(aff: *mut afffile_T) {
    // SAFETY: the caller promises the affix file; the regexps below are the
    // only heap allocations the entries own.
    unsafe {
        xfree((*aff).af_enc.cast());

        for ht in [&raw mut (*aff).af_pref, &raw mut (*aff).af_suff] {
            let mut todo = (*ht).ht_used as c_int;
            let mut hi: *mut hashitem_T = (*ht).ht_array;
            while todo > 0 {
                if !(*hi).hi_key.is_null()
                    && (*hi).hi_key != (&raw const hash_removed).cast_mut().cast()
                {
                    todo -= 1;
                    let ah = (*hi).hi_key.cast::<affheader_T>();
                    let mut ae = (*ah).ah_first;
                    while !ae.is_null() {
                        vim_regfree((*ae).ae_prog);
                        ae = (*ae).ae_next;
                    }
                }
                hi = hi.add(1);
            }
        }

        hash_clear(&raw mut (*aff).af_pref);
        hash_clear(&raw mut (*aff).af_suff);
        hash_clear(&raw mut (*aff).af_comp);
    }
}
