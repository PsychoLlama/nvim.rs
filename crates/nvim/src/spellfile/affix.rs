//! `PFX` and `SFX` blocks of a `.aff` file.
//!
//! A block opens with a header naming the affix, saying whether it may
//! combine with an affix at the other end of the word, and how many entries
//! follow. Each entry gives what to chop off the stem, what to add, and the
//! condition the stem must satisfy for the affix to apply.
//!
//! # Postponed prefixes
//!
//! `PFXPOSTPONE` asks for prefixes to be applied when a word is checked
//! rather than expanded into the word list, which keeps the `.spl` far
//! smaller. Only a prefix that chops nothing can work that way, so
//! [`postpone_prefix`] first tries to turn a chop-one-add-one prefix into a
//! capitalisation rule, which chops nothing; whatever is left over with a
//! chop is expanded normally after all.
//!
//! Each postponed prefix takes an id, handed out when its block opens. If
//! no entry in the block turns out to be postponable the id is given back,
//! since ids are a scarce single byte shared with compound flags.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use crate::message_fmt::msg_cstr;
use crate::smsg;
use core::ffi::{CStr, c_char, c_int};

use crate::hashtab::{hash_add, hash_find};
use crate::mbyte::{mb_toupper, utf_head_off, utf_ptr2char, utfc_ptr2len};
use crate::memory::xstrlcpy;
use crate::os::cshim::snprintf;
use crate::spell::{onecap_copy, spelltab_upper};
use crate::strings::{has_non_ascii, vim_strchr};
use crate::types::{HashTab, NUL, size_t};
use ::libc::{atoi, strcpy};

use super::aff::{AffState, item_ptr};
use super::flags::{aff_process_flags, affitem2flag, check_renumber};
use super::wordtree::tree_add_word;
use super::{
    AH_KEY_LEN, AffEntry, AffFile, AffHeader, MAXLINELEN, PFX_FLAGS, SpellInfo, WFP_COMPFORBID,
    WFP_COMPPERMIT, WFP_NC, WFP_UP, vim_regcomp, vim_regfree,
};
use crate::regexp::{RE_MAGIC, RE_STRICT, RE_STRING};

/// The header line of a `PFX`/`SFX` block. Returns false to stop reading.
///
/// # Safety
///
/// As [`handle_line`].
pub(super) fn handle_affix_header(
    spin: &mut SpellInfo,
    aff: &mut AffFile,
    st: &mut AffState,
    items: &[&CStr],
    fname: &CStr,
    lnum: c_int,
) -> bool {
    // SAFETY: `key` is AH_KEY_LEN and `xstrlcpy` is given that bound.
    let is_prefix = items[0].to_bytes().starts_with(b"P");
    let tp: *mut HashTab = if is_prefix {
        &raw mut aff.af_pref
    } else {
        &raw mut aff.af_suff
    };

    let mut key: [c_char; 17] = [0; 17];
    unsafe { xstrlcpy(key.as_mut_ptr(), item_ptr(items[1]), AH_KEY_LEN as size_t) };
    let hi = unsafe { hash_find(tp, key.as_mut_ptr()) };
    let combines = items[2].to_bytes().starts_with(b"Y");

    if hi.is_kept() {
        // A continued block for an affix already defined.
        st.cur_aff = unsafe { AffHeader::of_key(hi.hi_key) };
        if (unsafe { (*st.cur_aff).ah_combine } != 0) != combines {
            // SAFETY: the affix file's name, NUL-terminated.
            let (file, item) = (msg_cstr(fname), msg_cstr(items[1]));
            smsg!(
                0,
                "Different combining flag in continued affix block in {file} line {lnum}: {item}"
            );
        }
        if unsafe { (*st.cur_aff).ah_follows } == 0 {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let (fname, arg2) = (msg_cstr(fname), msg_cstr(items[1]));
            smsg!(0, "Duplicate affix in {fname} line {}: {arg2}", lnum);
        }
    } else {
        st.cur_aff = spin.si_arena.alloc::<AffHeader>();
        unsafe {
            (*st.cur_aff).ah_flag = affitem2flag(aff.af_flagtype, item_ptr(items[1]), fname, lnum)
        };
        // An unusable name is fatal: the key would not fit, or the
        // flag could not be read.
        if unsafe { (*st.cur_aff).ah_flag } == 0
            || items[1].to_bytes().len() >= AH_KEY_LEN as size_t
        {
            return false;
        }
        let clashes = [
            aff.af_bad,
            aff.af_rare,
            aff.af_keepcase,
            aff.af_needaffix,
            aff.af_circumfix,
            aff.af_nosuggest,
            aff.af_needcomp,
            aff.af_comproot,
        ];
        if clashes.contains(&unsafe { (*st.cur_aff).ah_flag }) {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let (fname, arg2) = (msg_cstr(fname), msg_cstr(items[1]));
            smsg!(
                0,
                "Affix also used for BAD/RARE/KEEPCASE/NEEDAFFIX/NEEDCOMPOUND/NOSUGGEST in {fname} line {}: {arg2}",
                lnum
            );
        }
        unsafe { strcpy(AffHeader::key(st.cur_aff), item_ptr(items[1])) };
        let _ = unsafe { hash_add(tp, AffHeader::key(st.cur_aff)) };
        unsafe { (*st.cur_aff).ah_combine = combines as c_int };
    }

    // An "S" after the count says another block for this affix follows.
    let mut lasti = 4;
    if items.len() > lasti && items[lasti] == c"S" {
        lasti += 1;
        unsafe { (*st.cur_aff).ah_follows = 1 };
    } else {
        unsafe { (*st.cur_aff).ah_follows = 0 };
    }
    if items.len() > lasti && !aff.af_ignoreextra && !items[lasti].to_bytes().starts_with(b"#") {
        // SAFETY: the affix file's name, NUL-terminated.
        let (file, item) = (msg_cstr(fname), msg_cstr(items[lasti]));
        smsg!(0, "Trailing text in {file} line {lnum}: {item}");
    }
    if items[2] != c"Y" && items[2] != c"N" {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let (fname, arg2) = (msg_cstr(fname), msg_cstr(items[2]));
        smsg!(0, "Expected Y or N in {fname} line {}: {arg2}", lnum);
    }

    if is_prefix && aff.af_pfxpostpone != 0 {
        if unsafe { (*st.cur_aff).ah_new_id } == 0 {
            check_renumber(spin);
            spin.si_newpref_id += 1;
            unsafe { (*st.cur_aff).ah_new_id = spin.si_newpref_id };
            // Nothing has used the id yet; it is given back at the end
            // of the block if nothing does.
            st.did_postpone_prefix = false;
        } else {
            st.did_postpone_prefix = true;
        }
    }
    // SAFETY: an item, which is a live NUL-terminated string.
    st.aff_todo = unsafe { atoi(item_ptr(items[3])) };
    true
}

/// One entry of a `PFX`/`SFX` block.
///
/// # Safety
///
/// As [`handle_line`].
pub(super) fn handle_affix_entry(
    spin: &mut SpellInfo,
    aff: &mut AffFile,
    st: &mut AffState,
    items: &[&CStr],
    fname: &CStr,
    lnum: c_int,
) {
    // SAFETY: `buf` is MAXLINELEN, which is the bound the snprintf calls
    // are given.
    let lasti = 5;
    // A lone "-" is Hunspell's morphological field separator.
    if items.len() > lasti
        && !items[lasti].to_bytes().starts_with(b"#")
        && (items[lasti] != c"-" || items.len() != lasti + 1)
    {
        // SAFETY: the affix file's name, NUL-terminated.
        let (file, item) = (msg_cstr(fname), msg_cstr(items[lasti]));
        smsg!(0, "Trailing text in {file} line {lnum}: {item}");
    }
    st.aff_todo -= 1;

    let entry = spin.si_arena.alloc::<AffEntry>();
    if items[2] != c"0" {
        unsafe { (*entry).ae_chop = spin.si_arena.save_str(item_ptr(items[2])) };
    }
    if items[3] != c"0" {
        unsafe { (*entry).ae_add = spin.si_arena.save_str(item_ptr(items[3])) };
        // Flags the added form itself carries follow a "/".
        unsafe { (*entry).ae_flags = vim_strchr((*entry).ae_add, b'/' as c_int) };
        if !unsafe { (*entry).ae_flags }.is_null() {
            unsafe { *(*entry).ae_flags = NUL as c_char };
            unsafe { (*entry).ae_flags = (*entry).ae_flags.add(1) };
            unsafe { aff_process_flags(aff, entry) };
        }
    }

    // With 'ascii' set, an affix that needs more than ASCII is dropped.
    if spin.si_ascii != 0
        && (unsafe { has_non_ascii((*entry).ae_chop) } || unsafe { has_non_ascii((*entry).ae_add) })
    {
        return;
    }

    unsafe { (*entry).ae_next = (*st.cur_aff).ah_first };
    unsafe { (*st.cur_aff).ah_first = entry };

    let is_prefix = items[0].to_bytes().starts_with(b"P");
    if items[4] != c"." {
        unsafe { (*entry).ae_cond = spin.si_arena.save_str(item_ptr(items[4])) };
        let mut buf: [c_char; MAXLINELEN as usize] = [0; MAXLINELEN as usize];
        // A prefix condition anchors at the start, a suffix at the end.
        let pattern = if is_prefix { c"^%s" } else { c"%s$" };
        let (out, room) = (buf.as_mut_ptr(), size_of_val(&buf));
        unsafe { snprintf(out, room, pattern.as_ptr(), item_ptr(items[4])) };
        unsafe {
            (*entry).ae_prog =
                vim_regcomp(cstr::at(buf.as_mut_ptr()), RE_MAGIC + RE_STRING + RE_STRICT)
        };
        if unsafe { (*entry).ae_prog }.is_null() {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let (fname, arg2) = (msg_cstr(fname), msg_cstr(items[4]));
            smsg!(0, "Broken condition in {fname} line {}: {arg2}", lnum);
        }
    }

    if is_prefix && aff.af_pfxpostpone != 0 && unsafe { (*entry).ae_flags }.is_null() {
        postpone_prefix(spin, st, entry, items);
    }
}

/// File a prefix in the prefix tree instead of expanding it into words.
///
/// # Safety
///
/// As [`handle_affix_entry`].
pub(super) fn postpone_prefix(
    spin: &mut SpellInfo,
    st: &mut AffState,
    entry: *mut AffEntry,
    items: &[&CStr],
) {
    // SAFETY: the caller promises the entry and the items.
    // A prefix that chops one letter and adds the same letter upper
    // cased is really a capitalisation rule; record it as one so the
    // checker can apply it without a chop.
    let mut upper = false;
    if !unsafe { (*entry).ae_chop }.is_null()
        && !unsafe { (*entry).ae_add }.is_null()
        && unsafe {
            *(*entry)
                .ae_chop
                .offset(utfc_ptr2len((*entry).ae_chop) as isize)
        } as c_int
            == NUL
    {
        let c = unsafe { utf_ptr2char((*entry).ae_chop) };
        let c_up = if c >= 128 {
            mb_toupper(c)
        } else {
            spelltab_upper(c as usize) as c_int
        };
        if c_up != c
            && (unsafe { (*entry).ae_cond }.is_null()
                || unsafe { utf_ptr2char((*entry).ae_cond) } == c)
        {
            // Step back to the last character of what is added.
            let mut p = unsafe { (*entry).ae_add.add(cstr::bytes_at((*entry).ae_add).len()) };
            p = unsafe { p.offset(-((utf_head_off((*entry).ae_add, p.sub(1)) + 1) as isize)) };
            if unsafe { utf_ptr2char(p) } == c_up {
                upper = true;
                unsafe { (*entry).ae_chop = core::ptr::null_mut() };
                unsafe { *p = NUL as c_char };
                if !unsafe { (*entry).ae_cond }.is_null() {
                    // The condition has to match the capitalised form.
                    let mut buf: [c_char; MAXLINELEN as usize] = [0; MAXLINELEN as usize];
                    unsafe { onecap_copy(item_ptr(items[4]), buf.as_mut_ptr(), true) };
                    unsafe { (*entry).ae_cond = spin.si_arena.save_str(buf.as_mut_ptr()) };
                    if !unsafe { (*entry).ae_cond }.is_null() {
                        let out = buf.as_mut_ptr();
                        let cond = unsafe { (*entry).ae_cond };
                        unsafe { snprintf(out, MAXLINELEN as size_t, c"^%s".as_ptr(), cond) };
                        unsafe { vim_regfree((*entry).ae_prog) };
                        unsafe {
                            (*entry).ae_prog =
                                vim_regcomp(cstr::at(buf.as_mut_ptr()), RE_MAGIC + RE_STRING)
                        };
                    }
                }
            }
        }
    }

    // Only a prefix with nothing to chop can be applied at match time.
    if unsafe { (*entry).ae_chop }.is_null() {
        file_postponed_prefix(spin, st, entry, upper);
    }

    // Nothing in the block was postponed after all; give the id back.
    if st.aff_todo == 0 && !st.did_postpone_prefix {
        spin.si_newpref_id -= 1;
        unsafe { (*st.cur_aff).ah_new_id = 0 };
    }
}

/// Put one postponed prefix into the prefix tree.
///
/// # Safety
///
/// As [`postpone_prefix`].
pub(super) fn file_postponed_prefix(
    spin: &mut SpellInfo,
    st: &mut AffState,
    entry: *mut AffEntry,
    upper: bool,
) {
    // SAFETY: the caller promises the entry.
    // Conditions are shared: the tree stores an index into si_prefcond.
    // SAFETY: the caller promises the entry's condition string.
    let cond = unsafe { (*entry).ae_cond };
    let want: Option<&[u8]> = if cond.is_null() {
        None
    } else {
        // SAFETY: as above.
        Some(unsafe { cstr::bytes_at(cond) })
    };
    let found = spin
        .si_prefcond
        .iter()
        .rposition(|held| held.as_deref() == want);
    let idx = match found {
        Some(at) => at as c_int,
        None => {
            spin.si_prefcond.push(want.map(Into::into));
            spin.si_prefcond.len() as c_int - 1
        }
    };

    let added = if unsafe { (*entry).ae_add }.is_null() {
        c"".as_ptr().cast_mut()
    } else {
        unsafe { (*entry).ae_add }
    };
    let mut n = PFX_FLAGS;
    if unsafe { (*st.cur_aff).ah_combine } == 0 {
        n |= WFP_NC as c_int;
    }
    if upper {
        n |= WFP_UP as c_int;
    }
    if unsafe { (*entry).ae_comppermit } != 0 {
        n |= WFP_COMPPERMIT as c_int;
    }
    if unsafe { (*entry).ae_compforbid } != 0 {
        n |= WFP_COMPFORBID as c_int;
    }
    let prefroot = spin.si_prefroot;
    let new_id = unsafe { (*st.cur_aff).ah_new_id };
    let _ = unsafe { tree_add_word(&mut *spin, added, prefroot, n, idx, new_id) };
    st.did_postpone_prefix = true;
}
