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

#![forbid(unsafe_code)]

use crate::message_fmt::{msg_bytes, msg_cstr};
use crate::smsg;
use crate::strings::{find_char, has_char};
use core::ffi::{CStr, c_int, c_uint};

use crate::charset::getdigits_int_at;
use crate::cstr::byte_at;
use crate::mbyte::{c_cluster_len, char_at};
use crate::types::NUL;

use super::wordtree::COMP_ITEM_BYTES;
use super::{
    AFT_CAPLONG, AFT_CHAR, AFT_LONG, AFT_NUM, AffEntry, AffFile, CompItem, SpellInfo, ZERO_FLAG,
};

/// `text` from `at` on, empty past its end: what a C walk saw there.
fn rest(text: &[u8], at: usize) -> &[u8] {
    text.get(at..).unwrap_or_default()
}

/// Decode one flag of `text` at `*at` and advance past it. Returns 0 when
/// there is none.
///
/// `text` is read as the C string it was: past its end, or at a NUL, is
/// the terminator.
pub(super) fn get_affitem(flagtype: c_int, text: &[u8], at: &mut usize) -> c_uint {
    if flagtype == AFT_NUM {
        if !byte_at(text, *at).is_ascii_digit() {
            // Not a number at all; step over the offending byte so the
            // caller makes progress.
            *at += 1;
            return 0;
        }
        let (mut res, past) = getdigits_int_at(text, *at, true, 0);
        *at = past;
        if res == 0 {
            // Zero would read as "no flag", so it gets its own value.
            res = ZERO_FLAG;
        }
        return res as c_uint;
    }

    let mut res = char_at(rest(text, *at));
    *at += c_cluster_len(rest(text, *at));
    // Two-character flags: always for LONG, and for CAPLONG only when
    // the first character is upper case ASCII.
    if flagtype == AFT_LONG
        || (flagtype == AFT_CAPLONG && res >= b'A' as c_int && res <= b'Z' as c_int)
    {
        if byte_at(text, *at) == NUL as u8 {
            return 0;
        }
        res = char_at(rest(text, *at)) + (res << 16);
        *at += c_cluster_len(rest(text, *at));
    }
    res as c_uint
}

/// Decode an item that should hold exactly one flag, complaining if it
/// does not.
pub(super) fn affitem2flag(flagtype: c_int, item: &[u8], fname: &CStr, lnum: c_int) -> c_uint {
    let mut at = 0;
    let res = get_affitem(flagtype, item, &mut at);
    if res == 0 {
        let (file, shown) = (msg_cstr(fname), msg_bytes(item));
        if flagtype == AFT_NUM {
            smsg!(0, "Flag is not a number in {file} line {lnum}: {shown}");
        } else {
            smsg!(0, "Illegal flag in {file} line {lnum}: {shown}");
        }
    }
    // Anything left over means the item was more than one flag.
    if byte_at(item, at) != NUL as u8 {
        let (file, shown) = (msg_cstr(fname), msg_bytes(item));
        smsg!(0, "Affix name too long in {file} line {lnum}: {shown}");
        return 0;
    }
    res
}

/// Is `flag` one of the flags in `afflist`?
pub(super) fn flag_in_afflist(flagtype: c_int, afflist: &[u8], flag: c_uint) -> bool {
    match flagtype {
        AFT_CHAR => find_char(afflist, flag as c_int).is_some(),
        AFT_LONG | AFT_CAPLONG => {
            let mut p = 0;
            while byte_at(afflist, p) != NUL as u8 {
                let mut n = char_at(rest(afflist, p)) as c_uint;
                p += c_cluster_len(rest(afflist, p));
                if (flagtype == AFT_LONG || (n >= b'A' as c_uint && n <= b'Z' as c_uint))
                    && byte_at(afflist, p) != NUL as u8
                {
                    n = (char_at(rest(afflist, p)) as c_uint).wrapping_add(n << 16);
                    p += c_cluster_len(rest(afflist, p));
                }
                if n == flag {
                    return true;
                }
            }
            false
        }
        AFT_NUM => {
            let mut p = 0;
            while byte_at(afflist, p) != NUL as u8 {
                let (digits, past) = getdigits_int_at(afflist, p, true, 0);
                p = past;
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
                if byte_at(afflist, p) != NUL as u8 {
                    p += 1;
                }
            }
            false
        }
        _ => false,
    }
}

/// Strip `COMPOUNDPERMITFLAG` and `COMPOUNDFORBIDFLAG` out of an affix
/// entry's own flags, recording them as booleans on the entry instead.
pub(super) fn aff_process_flags(affile: &AffFile, entry: &mut AffEntry) {
    if affile.af_compforbid == 0 && affile.af_comppermit == 0 {
        return;
    }
    let Some(flags) = entry.ae_flags.as_mut() else {
        return;
    };
    let mut p = 0;
    while byte_at(flags, p) != NUL as u8 {
        let prevp = p;
        let flag = get_affitem(affile.af_flagtype, flags, &mut p);
        if flag == affile.af_comppermit || flag == affile.af_compforbid {
            // Remove the flag from the list and stay put, so the next
            // flag is read from where this one was.
            flags.drain(prevp..p.min(flags.len()));
            p = prevp;
            if flag == affile.af_comppermit {
                entry.ae_comppermit = true;
            } else {
                entry.ae_compforbid = true;
            }
        }
        if affile.af_flagtype == AFT_NUM && byte_at(flags, p) == b',' {
            p += 1;
        }
    }
    if flags.first().is_none_or(|&b| b == NUL as u8) {
        entry.ae_flags = None;
    }
}

/// Rewrite the compound pattern in terms of one-byte ids, appending it to
/// whatever a previous `.aff` file already contributed.
pub(super) fn process_compflags(spin: &mut SpellInfo, aff: &mut AffFile, compflags: &[u8]) {
    // What upstream allocated: the old pattern, a separator and the new one,
    // where each flag turns into at most one byte.
    let mut len = compflags.len() + 1;
    if let Some(old) = &spin.si_compflags {
        len += old.len() + 1;
    }
    spin.si_arena.charge(len, false);
    let mut dest = Vec::with_capacity(len);
    if let Some(old) = &spin.si_compflags {
        dest.extend_from_slice(old);
        dest.push(b'/');
    }

    let mut p = 0;
    while p < compflags.len() {
        // Pattern punctuation passes straight through.
        if has_char(c"/?*+[]", c_int::from(compflags[p])) {
            dest.push(compflags[p]);
            p += 1;
            continue;
        }

        let prevp = p;
        let flag = get_affitem(aff.af_flagtype, compflags, &mut p);
        if flag != 0 {
            let key = &compflags[prevp..p.min(compflags.len())];
            let id = match aff.af_comp.find(key) {
                Some(at) => aff.af_comp.get(at).ci_new_id,
                None => {
                    spin.si_arena.charge(COMP_ITEM_BYTES, true);
                    // Ids count downwards, skipping any byte that would be
                    // meaningful in the pattern this becomes.
                    let id = loop {
                        check_renumber(spin);
                        let id = spin.si_newcomp_id;
                        spin.si_newcomp_id -= 1;
                        if !has_char(c"/?*+[]\\-^", id) {
                            break id;
                        }
                    };
                    aff.af_comp.insert(CompItem {
                        ci_key: key.to_vec(),
                        ci_new_id: id,
                    });
                    id
                }
            };
            dest.push(id as u8);
        }
        if aff.af_flagtype == AFT_NUM && byte_at(compflags, p) == b',' {
            p += 1;
        }
    }
    spin.si_compflags = Some(dest);
}

/// Move the prefix and compound id counters apart before they meet.
///
/// Prefix ids count up from zero and compound ids down from 255. When both
/// reach the same value in the lower half, the split is redrawn at 127/255
/// so each kind keeps a range to itself.
pub(super) fn check_renumber(spin: &mut SpellInfo) {
    if spin.si_newpref_id == spin.si_newcomp_id && spin.si_newcomp_id < 128 {
        spin.si_newpref_id = 127;
        spin.si_newcomp_id = 255;
    }
}
