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

use crate::cstr;
use crate::message_fmt::c_str;
use crate::smsg;
use core::ffi::{c_char, c_int};

use crate::garray::ga_append_via_ptr;
use crate::hashtab::{hash_add, hash_find, hash_removed};
use crate::mbyte::{mb_toupper, utf_head_off, utf_ptr2char, utfc_ptr2len};
use crate::memory::xstrlcpy;
use crate::os::cshim::snprintf;
use crate::spell::{onecap_copy, spelltab_upper};
use crate::strings::{has_non_ascii, vim_strchr};
use crate::types::{NUL, hashitem_T, hashtab_T, size_t};
use ::libc::{atoi, strcpy, strlen};

use super::aff::{AffState, str_equal};
use super::flags::{aff_process_flags, affitem2flag, check_renumber};
use super::wordtree::tree_add_word;
use super::{
    AH_KEY_LEN, MAXLINELEN, PFX_FLAGS, WFP_COMPFORBID, WFP_COMPPERMIT, WFP_NC, WFP_UP, affentry_T,
    afffile_T, affheader_T, spellinfo_T, vim_regcomp, vim_regfree,
};
use crate::regexp::{RE_MAGIC, RE_STRICT, RE_STRING};

/// The header line of a `PFX`/`SFX` block. Returns false to stop reading.
///
/// # Safety
///
/// As [`handle_line`].
pub(super) unsafe fn handle_affix_header(
    spin: *mut spellinfo_T,
    aff: *mut afffile_T,
    st: &mut AffState,
    items: &[*mut c_char],
    fname: *mut c_char,
    lnum: c_int,
) -> bool {
    // SAFETY: the caller promises the items; `key` is AH_KEY_LEN and
    // `xstrlcpy` is given that bound.
    let is_prefix = unsafe { *items[0] } as c_int == b'P' as c_int;
    let tp: *mut hashtab_T = if is_prefix {
        unsafe { &raw mut (*aff).af_pref }
    } else {
        unsafe { &raw mut (*aff).af_suff }
    };

    let mut key: [c_char; 17] = [0; 17];
    unsafe { xstrlcpy(key.as_mut_ptr(), items[1], AH_KEY_LEN as size_t) };
    let hi: *mut hashitem_T = unsafe { hash_find(tp, key.as_mut_ptr()) };
    let combines = unsafe { *items[2] } as c_int == b'Y' as c_int;

    if !unsafe { (*hi).hi_key }.is_null()
        && unsafe { (*hi).hi_key } != (&raw const hash_removed).cast_mut().cast()
    {
        // A continued block for an affix already defined.
        st.cur_aff = unsafe { affheader_T::of_key((*hi).hi_key) };
        if (unsafe { (*st.cur_aff).ah_combine } != 0) != combines {
            // SAFETY: the affix file's name and the item, NUL-terminated.
            let (file, item) = unsafe { (c_str(fname), c_str(items[1])) };
            smsg!(
                0,
                "Different combining flag in continued affix block in {file} line {lnum}: {item}"
            );
        }
        if unsafe { (*st.cur_aff).ah_follows } == 0 {
            // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
            let (fname, arg2) = unsafe { (c_str(fname), c_str(items[1])) };
            smsg!(0, "Duplicate affix in {fname} line {}: {arg2}", lnum);
        }
    } else {
        st.cur_aff = unsafe { (*spin).si_arena.alloc::<affheader_T>() };
        unsafe { (*st.cur_aff).ah_flag = affitem2flag((*aff).af_flagtype, items[1], fname, lnum) };
        // An unusable name is fatal: the key would not fit, or the
        // flag could not be read.
        if unsafe { (*st.cur_aff).ah_flag } == 0
            || unsafe { strlen(items[1]) } >= AH_KEY_LEN as size_t
        {
            return false;
        }
        let clashes = [
            unsafe { (*aff).af_bad },
            unsafe { (*aff).af_rare },
            unsafe { (*aff).af_keepcase },
            unsafe { (*aff).af_needaffix },
            unsafe { (*aff).af_circumfix },
            unsafe { (*aff).af_nosuggest },
            unsafe { (*aff).af_needcomp },
            unsafe { (*aff).af_comproot },
        ];
        if clashes.contains(&unsafe { (*st.cur_aff).ah_flag }) {
            // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
            let (fname, arg2) = unsafe { (c_str(fname), c_str(items[1])) };
            smsg!(
                0,
                "Affix also used for BAD/RARE/KEEPCASE/NEEDAFFIX/NEEDCOMPOUND/NOSUGGEST in {fname} line {}: {arg2}",
                lnum
            );
        }
        unsafe { strcpy(affheader_T::key(st.cur_aff), items[1]) };
        let _ = unsafe { hash_add(tp, affheader_T::key(st.cur_aff)) };
        unsafe { (*st.cur_aff).ah_combine = combines as c_int };
    }

    // An "S" after the count says another block for this affix follows.
    let mut lasti = 4;
    if items.len() > lasti && unsafe { cstr::bytes_at(items[lasti]) == b"S" } {
        lasti += 1;
        unsafe { (*st.cur_aff).ah_follows = 1 };
    } else {
        unsafe { (*st.cur_aff).ah_follows = 0 };
    }
    if items.len() > lasti
        && !unsafe { (*aff).af_ignoreextra }
        && unsafe { *items[lasti] } as c_int != b'#' as c_int
    {
        // SAFETY: the affix file's name and the trailing item.
        let (file, item) = unsafe { (c_str(fname), c_str(items[lasti])) };
        smsg!(0, "Trailing text in {file} line {lnum}: {item}");
    }
    if unsafe { cstr::bytes_at(items[2]) != b"Y" } && unsafe { cstr::bytes_at(items[2]) != b"N" } {
        // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
        let (fname, arg2) = unsafe { (c_str(fname), c_str(items[2])) };
        smsg!(0, "Expected Y or N in {fname} line {}: {arg2}", lnum);
    }

    if is_prefix && unsafe { (*aff).af_pfxpostpone } != 0 {
        if unsafe { (*st.cur_aff).ah_newID } == 0 {
            unsafe { check_renumber(spin) };
            unsafe { (*spin).si_newprefID += 1 };
            unsafe { (*st.cur_aff).ah_newID = (*spin).si_newprefID };
            // Nothing has used the id yet; it is given back at the end
            // of the block if nothing does.
            st.did_postpone_prefix = false;
        } else {
            st.did_postpone_prefix = true;
        }
    }
    st.aff_todo = unsafe { atoi(items[3]) };
    true
}

/// One entry of a `PFX`/`SFX` block.
///
/// # Safety
///
/// As [`handle_line`].
pub(super) unsafe fn handle_affix_entry(
    spin: *mut spellinfo_T,
    aff: *mut afffile_T,
    st: &mut AffState,
    items: &[*mut c_char],
    fname: *mut c_char,
    lnum: c_int,
) {
    // SAFETY: the caller promises the items; `buf` is MAXLINELEN, which is
    // the bound the snprintf calls are given.
    let lasti = 5;
    // A lone "-" is Hunspell's morphological field separator.
    if items.len() > lasti
        && unsafe { *items[lasti] } as c_int != b'#' as c_int
        && (unsafe { cstr::bytes_at(items[lasti]) != b"-" } || items.len() != lasti + 1)
    {
        // SAFETY: the affix file's name and the trailing item.
        let (file, item) = unsafe { (c_str(fname), c_str(items[lasti])) };
        smsg!(0, "Trailing text in {file} line {lnum}: {item}");
    }
    st.aff_todo -= 1;

    let entry = unsafe { (*spin).si_arena.alloc::<affentry_T>() };
    if unsafe { cstr::bytes_at(items[2]) != b"0" } {
        unsafe { (*entry).ae_chop = (*spin).si_arena.save_str(items[2]) };
    }
    if unsafe { cstr::bytes_at(items[3]) != b"0" } {
        unsafe { (*entry).ae_add = (*spin).si_arena.save_str(items[3]) };
        // Flags the added form itself carries follow a "/".
        unsafe { (*entry).ae_flags = vim_strchr((*entry).ae_add, b'/' as c_int) };
        if !unsafe { (*entry).ae_flags }.is_null() {
            unsafe { *(*entry).ae_flags = NUL as c_char };
            unsafe { (*entry).ae_flags = (*entry).ae_flags.add(1) };
            unsafe { aff_process_flags(aff, entry) };
        }
    }

    // With 'ascii' set, an affix that needs more than ASCII is dropped.
    if unsafe { (*spin).si_ascii } != 0
        && (unsafe { has_non_ascii((*entry).ae_chop) } || unsafe { has_non_ascii((*entry).ae_add) })
    {
        return;
    }

    unsafe { (*entry).ae_next = (*st.cur_aff).ah_first };
    unsafe { (*st.cur_aff).ah_first = entry };

    let is_prefix = unsafe { *items[0] } as c_int == b'P' as c_int;
    if unsafe { cstr::bytes_at(items[4]) != b"." } {
        unsafe { (*entry).ae_cond = (*spin).si_arena.save_str(items[4]) };
        let mut buf: [c_char; MAXLINELEN as usize] = [0; MAXLINELEN as usize];
        // A prefix condition anchors at the start, a suffix at the end.
        let pattern = if is_prefix { c"^%s" } else { c"%s$" };
        let (out, room) = (buf.as_mut_ptr(), size_of_val(&buf));
        unsafe { snprintf(out, room, pattern.as_ptr(), items[4]) };
        unsafe {
            (*entry).ae_prog = vim_regcomp(buf.as_mut_ptr(), RE_MAGIC + RE_STRING + RE_STRICT)
        };
        if unsafe { (*entry).ae_prog }.is_null() {
            // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
            let (fname, arg2) = unsafe { (c_str(fname), c_str(items[4])) };
            smsg!(0, "Broken condition in {fname} line {}: {arg2}", lnum);
        }
    }

    if is_prefix && unsafe { (*aff).af_pfxpostpone } != 0 && unsafe { (*entry).ae_flags }.is_null()
    {
        unsafe { postpone_prefix(spin, st, entry, items) };
    }
}

/// File a prefix in the prefix tree instead of expanding it into words.
///
/// # Safety
///
/// As [`handle_affix_entry`].
pub(super) unsafe fn postpone_prefix(
    spin: *mut spellinfo_T,
    st: &mut AffState,
    entry: *mut affentry_T,
    items: &[*mut c_char],
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
            let mut p = unsafe { (*entry).ae_add.add(strlen((*entry).ae_add)) };
            p = unsafe { p.offset(-((utf_head_off((*entry).ae_add, p.sub(1)) + 1) as isize)) };
            if unsafe { utf_ptr2char(p) } == c_up {
                upper = true;
                unsafe { (*entry).ae_chop = core::ptr::null_mut() };
                unsafe { *p = NUL as c_char };
                if !unsafe { (*entry).ae_cond }.is_null() {
                    // The condition has to match the capitalised form.
                    let mut buf: [c_char; MAXLINELEN as usize] = [0; MAXLINELEN as usize];
                    unsafe { onecap_copy(items[4], buf.as_mut_ptr(), true) };
                    unsafe { (*entry).ae_cond = (*spin).si_arena.save_str(buf.as_mut_ptr()) };
                    if !unsafe { (*entry).ae_cond }.is_null() {
                        let out = buf.as_mut_ptr();
                        let cond = unsafe { (*entry).ae_cond };
                        unsafe { snprintf(out, MAXLINELEN as size_t, c"^%s".as_ptr(), cond) };
                        unsafe { vim_regfree((*entry).ae_prog) };
                        unsafe {
                            (*entry).ae_prog = vim_regcomp(buf.as_mut_ptr(), RE_MAGIC + RE_STRING)
                        };
                    }
                }
            }
        }
    }

    // Only a prefix with nothing to chop can be applied at match time.
    if unsafe { (*entry).ae_chop }.is_null() {
        unsafe { file_postponed_prefix(spin, st, entry, upper) };
    }

    // Nothing in the block was postponed after all; give the id back.
    if st.aff_todo == 0 && !st.did_postpone_prefix {
        unsafe { (*spin).si_newprefID -= 1 };
        unsafe { (*st.cur_aff).ah_newID = 0 };
    }
}

/// Put one postponed prefix into the prefix tree.
///
/// # Safety
///
/// As [`postpone_prefix`].
pub(super) unsafe fn file_postponed_prefix(
    spin: *mut spellinfo_T,
    st: &mut AffState,
    entry: *mut affentry_T,
    upper: bool,
) {
    // SAFETY: the caller promises the entry.
    // Conditions are shared: the tree stores an index into si_prefcond.
    let mut idx = unsafe { (*spin).si_prefcond.ga_len } - 1;
    while idx >= 0 {
        let conds = unsafe { (*spin).si_prefcond.ga_data.cast::<*mut c_char>() };
        let p = unsafe { *conds.offset(idx as isize) };
        if unsafe { str_equal(p, (*entry).ae_cond) } {
            break;
        }
        idx -= 1;
    }
    if idx < 0 {
        idx = unsafe { (*spin).si_prefcond.ga_len };
        let pp =
            unsafe { ga_append_via_ptr(&raw mut (*spin).si_prefcond, size_of::<*mut c_char>()) }
                .cast::<*mut c_char>();
        let cond = unsafe { (*entry).ae_cond };
        let saved = if cond.is_null() {
            core::ptr::null_mut()
        } else {
            unsafe { (*spin).si_arena.save_str(cond) }
        };
        unsafe { *pp = saved };
    }

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
    let prefroot = unsafe { (*spin).si_prefroot };
    let newID = unsafe { (*st.cur_aff).ah_newID };
    let _ = unsafe { tree_add_word(&mut *spin, added, prefroot, n, idx, newID) };
    st.did_postpone_prefix = true;
}
