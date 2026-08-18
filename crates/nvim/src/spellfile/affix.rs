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

use crate::smsg_c;
use core::ffi::{c_char, c_int};

use crate::garray::ga_append_via_ptr;
use crate::hashtab::{hash_add, hash_find, hash_removed};
use crate::mbyte::{mb_toupper, utf_head_off, utf_ptr2char, utfc_ptr2len};
use crate::memory::xstrlcpy;
use crate::os::cshim::{gettext, snprintf};
use crate::spell::{onecap_copy, spelltab};
use crate::strings::{has_non_ascii, vim_strchr};
use crate::types::{hashitem_T, hashtab_T, size_t};
use ::libc::{atoi, strcmp, strcpy, strlen};

use super::aff::{AffState, str_equal};
use super::flags::{aff_process_flags, affitem2flag, check_renumber};
use super::wordtree::tree_add_word;
use super::{
    AH_KEY_LEN, MAXLINELEN, NUL, PFX_FLAGS, WFP_COMPFORBID, WFP_COMPPERMIT, WFP_NC, WFP_UP,
    affentry_T, afffile_T, affheader_T, e_afftrailing, spellinfo_T, vim_regcomp, vim_regfree,
};
use crate::regexp::{RE_MAGIC, RE_STRICT, RE_STRING};

/// The header line of a `PFX`/`SFX` block. Returns false to stop reading.
///
/// # Safety
///
/// As [`handle_line`].
pub unsafe fn handle_affix_header(
    spin: *mut spellinfo_T,
    aff: *mut afffile_T,
    st: &mut AffState,
    items: &[*mut c_char],
    fname: *mut c_char,
    lnum: c_int,
) -> bool {
    // SAFETY: the caller promises the items; `key` is AH_KEY_LEN and
    // `xstrlcpy` is given that bound.
    unsafe {
        let is_prefix = *items[0] as c_int == b'P' as c_int;
        let tp: *mut hashtab_T = if is_prefix {
            &raw mut (*aff).af_pref
        } else {
            &raw mut (*aff).af_suff
        };

        let mut key: [c_char; 17] = [0; 17];
        xstrlcpy(key.as_mut_ptr(), items[1], AH_KEY_LEN as size_t);
        let hi: *mut hashitem_T = hash_find(tp, key.as_mut_ptr());
        let combines = *items[2] as c_int == b'Y' as c_int;

        if !(*hi).hi_key.is_null() && (*hi).hi_key != (&raw const hash_removed).cast_mut().cast() {
            // A continued block for an affix already defined.
            st.cur_aff = (*hi).hi_key.cast::<affheader_T>();
            if ((*st.cur_aff).ah_combine != 0) != combines {
                smsg_c!(
                    0,
                    gettext(
                        c"Different combining flag in continued affix block in %s line %d: %s"
                            .as_ptr(),
                    ),
                    fname,
                    lnum,
                    items[1],
                );
            }
            if (*st.cur_aff).ah_follows == 0 {
                smsg_c!(
                    0,
                    gettext(c"Duplicate affix in %s line %d: %s".as_ptr()),
                    fname,
                    lnum,
                    items[1],
                );
            }
        } else {
            st.cur_aff = (*spin).si_arena.alloc::<affheader_T>();
            (*st.cur_aff).ah_flag = affitem2flag((*aff).af_flagtype, items[1], fname, lnum);
            // An unusable name is fatal: the key would not fit, or the
            // flag could not be read.
            if (*st.cur_aff).ah_flag == 0 || strlen(items[1]) >= AH_KEY_LEN as size_t {
                return false;
            }
            let clashes = [
                (*aff).af_bad,
                (*aff).af_rare,
                (*aff).af_keepcase,
                (*aff).af_needaffix,
                (*aff).af_circumfix,
                (*aff).af_nosuggest,
                (*aff).af_needcomp,
                (*aff).af_comproot,
            ];
            if clashes.contains(&(*st.cur_aff).ah_flag) {
                smsg_c!(
                    0,
                    gettext(
                        c"Affix also used for BAD/RARE/KEEPCASE/NEEDAFFIX/NEEDCOMPOUND/NOSUGGEST in %s line %d: %s"
                            .as_ptr(),
                    ),
                    fname,
                    lnum,
                    items[1],
                );
            }
            strcpy((&raw mut (*st.cur_aff).ah_key).cast::<c_char>(), items[1]);
            hash_add(tp, (&raw mut (*st.cur_aff).ah_key).cast::<c_char>());
            (*st.cur_aff).ah_combine = combines as c_int;
        }

        // An "S" after the count says another block for this affix follows.
        let mut lasti = 4;
        if items.len() > lasti && strcmp(items[lasti], c"S".as_ptr()) == 0 {
            lasti += 1;
            (*st.cur_aff).ah_follows = 1;
        } else {
            (*st.cur_aff).ah_follows = 0;
        }
        if items.len() > lasti && !(*aff).af_ignoreextra && *items[lasti] as c_int != b'#' as c_int
        {
            smsg_c!(0, gettext(e_afftrailing.get()), fname, lnum, items[lasti]);
        }
        if strcmp(items[2], c"Y".as_ptr()) != 0 && strcmp(items[2], c"N".as_ptr()) != 0 {
            smsg_c!(
                0,
                gettext(c"Expected Y or N in %s line %d: %s".as_ptr()),
                fname,
                lnum,
                items[2],
            );
        }

        if is_prefix && (*aff).af_pfxpostpone != 0 {
            if (*st.cur_aff).ah_newID == 0 {
                check_renumber(spin);
                (*spin).si_newprefID += 1;
                (*st.cur_aff).ah_newID = (*spin).si_newprefID;
                // Nothing has used the id yet; it is given back at the end
                // of the block if nothing does.
                st.did_postpone_prefix = false;
            } else {
                st.did_postpone_prefix = true;
            }
        }
        st.aff_todo = atoi(items[3]);
        true
    }
}

/// One entry of a `PFX`/`SFX` block.
///
/// # Safety
///
/// As [`handle_line`].
pub unsafe fn handle_affix_entry(
    spin: *mut spellinfo_T,
    aff: *mut afffile_T,
    st: &mut AffState,
    items: &[*mut c_char],
    fname: *mut c_char,
    lnum: c_int,
) {
    // SAFETY: the caller promises the items; `buf` is MAXLINELEN, which is
    // the bound the snprintf calls are given.
    unsafe {
        let lasti = 5;
        // A lone "-" is Hunspell's morphological field separator.
        if items.len() > lasti
            && *items[lasti] as c_int != b'#' as c_int
            && (strcmp(items[lasti], c"-".as_ptr()) != 0 || items.len() != lasti + 1)
        {
            smsg_c!(0, gettext(e_afftrailing.get()), fname, lnum, items[lasti]);
        }
        st.aff_todo -= 1;

        let entry = (*spin).si_arena.alloc::<affentry_T>();
        if strcmp(items[2], c"0".as_ptr()) != 0 {
            (*entry).ae_chop = (*spin).si_arena.save_str(items[2]);
        }
        if strcmp(items[3], c"0".as_ptr()) != 0 {
            (*entry).ae_add = (*spin).si_arena.save_str(items[3]);
            // Flags the added form itself carries follow a "/".
            (*entry).ae_flags = vim_strchr((*entry).ae_add, b'/' as c_int);
            if !(*entry).ae_flags.is_null() {
                *(*entry).ae_flags = NUL as c_char;
                (*entry).ae_flags = (*entry).ae_flags.add(1);
                aff_process_flags(aff, entry);
            }
        }

        // With 'ascii' set, an affix that needs more than ASCII is dropped.
        if (*spin).si_ascii != 0
            && (has_non_ascii((*entry).ae_chop) || has_non_ascii((*entry).ae_add))
        {
            return;
        }

        (*entry).ae_next = (*st.cur_aff).ah_first;
        (*st.cur_aff).ah_first = entry;

        let is_prefix = *items[0] as c_int == b'P' as c_int;
        if strcmp(items[4], c".".as_ptr()) != 0 {
            (*entry).ae_cond = (*spin).si_arena.save_str(items[4]);
            let mut buf: [c_char; MAXLINELEN as usize] = [0; MAXLINELEN as usize];
            // A prefix condition anchors at the start, a suffix at the end.
            let pattern = if is_prefix { c"^%s" } else { c"%s$" };
            snprintf(
                buf.as_mut_ptr(),
                core::mem::size_of_val(&buf),
                pattern.as_ptr(),
                items[4],
            );
            (*entry).ae_prog = vim_regcomp(buf.as_mut_ptr(), RE_MAGIC + RE_STRING + RE_STRICT);
            if (*entry).ae_prog.is_null() {
                smsg_c!(
                    0,
                    gettext(c"Broken condition in %s line %d: %s".as_ptr()),
                    fname,
                    lnum,
                    items[4],
                );
            }
        }

        if is_prefix && (*aff).af_pfxpostpone != 0 && (*entry).ae_flags.is_null() {
            postpone_prefix(spin, st, entry, items);
        }
    }
}

/// File a prefix in the prefix tree instead of expanding it into words.
///
/// # Safety
///
/// As [`handle_affix_entry`].
pub unsafe fn postpone_prefix(
    spin: *mut spellinfo_T,
    st: &mut AffState,
    entry: *mut affentry_T,
    items: &[*mut c_char],
) {
    // SAFETY: the caller promises the entry and the items.
    unsafe {
        // A prefix that chops one letter and adds the same letter upper
        // cased is really a capitalisation rule; record it as one so the
        // checker can apply it without a chop.
        let mut upper = false;
        if !(*entry).ae_chop.is_null()
            && !(*entry).ae_add.is_null()
            && *(*entry)
                .ae_chop
                .offset(utfc_ptr2len((*entry).ae_chop) as isize) as c_int
                == NUL
        {
            let c = utf_ptr2char((*entry).ae_chop);
            let c_up = if c >= 128 {
                mb_toupper(c)
            } else {
                (*spelltab.ptr()).st_upper[c as usize] as c_int
            };
            if c_up != c && ((*entry).ae_cond.is_null() || utf_ptr2char((*entry).ae_cond) == c) {
                // Step back to the last character of what is added.
                let mut p = (*entry).ae_add.add(strlen((*entry).ae_add));
                p = p.offset(-((utf_head_off((*entry).ae_add, p.sub(1)) + 1) as isize));
                if utf_ptr2char(p) == c_up {
                    upper = true;
                    (*entry).ae_chop = core::ptr::null_mut();
                    *p = NUL as c_char;
                    if !(*entry).ae_cond.is_null() {
                        // The condition has to match the capitalised form.
                        let mut buf: [c_char; MAXLINELEN as usize] = [0; MAXLINELEN as usize];
                        onecap_copy(items[4], buf.as_mut_ptr(), true);
                        (*entry).ae_cond = (*spin).si_arena.save_str(buf.as_mut_ptr());
                        if !(*entry).ae_cond.is_null() {
                            snprintf(
                                buf.as_mut_ptr(),
                                MAXLINELEN as size_t,
                                c"^%s".as_ptr(),
                                (*entry).ae_cond,
                            );
                            vim_regfree((*entry).ae_prog);
                            (*entry).ae_prog = vim_regcomp(buf.as_mut_ptr(), RE_MAGIC + RE_STRING);
                        }
                    }
                }
            }
        }

        // Only a prefix with nothing to chop can be applied at match time.
        if (*entry).ae_chop.is_null() {
            file_postponed_prefix(spin, st, entry, upper);
        }

        // Nothing in the block was postponed after all; give the id back.
        if st.aff_todo == 0 && !st.did_postpone_prefix {
            (*spin).si_newprefID -= 1;
            (*st.cur_aff).ah_newID = 0;
        }
    }
}

/// Put one postponed prefix into the prefix tree.
///
/// # Safety
///
/// As [`postpone_prefix`].
pub unsafe fn file_postponed_prefix(
    spin: *mut spellinfo_T,
    st: &mut AffState,
    entry: *mut affentry_T,
    upper: bool,
) {
    // SAFETY: the caller promises the entry.
    unsafe {
        // Conditions are shared: the tree stores an index into si_prefcond.
        let mut idx = (*spin).si_prefcond.ga_len - 1;
        while idx >= 0 {
            let p = *(*spin)
                .si_prefcond
                .ga_data
                .cast::<*mut c_char>()
                .offset(idx as isize);
            if str_equal(p, (*entry).ae_cond) {
                break;
            }
            idx -= 1;
        }
        if idx < 0 {
            idx = (*spin).si_prefcond.ga_len;
            let pp = ga_append_via_ptr(
                &raw mut (*spin).si_prefcond,
                core::mem::size_of::<*mut c_char>(),
            )
            .cast::<*mut c_char>();
            *pp = if (*entry).ae_cond.is_null() {
                core::ptr::null_mut()
            } else {
                (*spin).si_arena.save_str((*entry).ae_cond)
            };
        }

        let added = if (*entry).ae_add.is_null() {
            c"".as_ptr().cast_mut()
        } else {
            (*entry).ae_add
        };
        let mut n = PFX_FLAGS;
        if (*st.cur_aff).ah_combine == 0 {
            n |= WFP_NC as c_int;
        }
        if upper {
            n |= WFP_UP as c_int;
        }
        if (*entry).ae_comppermit != 0 {
            n |= WFP_COMPPERMIT as c_int;
        }
        if (*entry).ae_compforbid != 0 {
            n |= WFP_COMPFORBID as c_int;
        }
        let prefroot = (*spin).si_prefroot;
        let newID = (*st.cur_aff).ah_newID;
        tree_add_word(&mut *spin, added, prefroot, n, idx, newID);
        st.did_postpone_prefix = true;
    }
}
