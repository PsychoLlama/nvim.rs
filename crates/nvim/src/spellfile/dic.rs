//! Reading the word lists `:mkspell` builds a `.spl` from.
//!
//! Two shapes are accepted. A Hunspell pair — `xx.aff` describing the
//! affixes and `xx.dic` listing stems with the affix flags each takes — is
//! read by [`spell_read_dic`]. A plain word list, one word per line with
//! optional `/` flags, is read by [`spell_read_wordfile`].
//!
//! # Expanding affixes
//!
//! A `.dic` line names a stem and the affixes it accepts. [`store_aff_word`]
//! applies them: for every affix whose flag the word carries and whose
//! condition its letters satisfy, it chops and adds what the affix says and
//! stores the result as a word in its own right, then recurses so a suffix
//! can sit on top of a prefix.
//!
//! `condit` carries what is allowed at each step:
//!
//! - `CONDIT_SUF` — a suffix may still be added.
//! - `CONDIT_COMB` — only affixes marked combinable may be added, which is
//!   how `Y`/`N` on the affix header is enforced one level down.
//! - `CONDIT_CFIX` / `CONDIT_AFF` — together they implement `CIRCUMFIX`:
//!   an affix marked with the circumfix flag only counts when its partner
//!   at the other end of the word is present too.
//!
//! # Postponed prefixes
//!
//! With `PFXPOSTPONE`, prefixes are not expanded into the word list at all;
//! each is given an id and the word records which ids it accepts, so the
//! checker applies them at match time instead. [`get_pfxlist`] collects
//! those ids, and [`get_compflags`] does the same for compound flags. Both
//! append to one buffer, prefix ids first.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::{semsg_c, smsg_c};
use core::ffi::{c_char, c_int};

use crate::ascii::ascii_isdigit;
use crate::charset::skipwhite;
use crate::fileio::vim_fgets;
use crate::hashtab::{
    hash_add_item, hash_clear, hash_find, hash_hash, hash_init, hash_lookup, hash_removed,
};
use crate::main::{IObuff, e_notopen, got_int, msg_col, msg_didout, p_verbose};
use crate::mbyte::{mb_charlen, string_convert, utf_head_off, utfc_ptr2len};
use crate::memory::{xfree, xmemcpyz, xstrlcat, xstrlcpy};
use crate::message::{msg_clr_eos, msg_outtrans_long, msg_start};
use crate::os::cshim::{gettext, memmove};
use crate::os::fs::os_fopen;
use crate::os::input::line_breakcheck;
use crate::os::time::os_time;
use crate::strings::{has_non_ascii, vim_snprintf};
use crate::types::{
    CONV_NONE, NUL, Timestamp, colnr_T, hash_T, hashitem_T, hashtab_T, size_t, uint8_t,
};
use crate::ui::ui_flush;
use ::libc::{fclose, strlen};

use super::flags::{flag_in_afflist, get_affitem};
use super::wordtree::store_word;
use super::{
    AFT_NUM, CONDIT_AFF, CONDIT_CFIX, CONDIT_COMB, CONDIT_SUF, FAIL, IOSIZE, MAXLINELEN, MAXWLEN,
    OK, WF_BANNED, WF_COMPROOT, WF_FIXCAP, WF_HAS_AFF, WF_KEEPCAP, WF_NEEDCOMP, WF_NOCOMPAFT,
    WF_NOCOMPBEF, WF_NOSUGGEST, WF_RARE, affentry_T, afffile_T, affheader_T, compitem_T,
    spell_message, spellinfo_T, vim_regexec_prog,
};

/// Read a `.dic` file: a count line, then one stem per line with the affix
/// flags it takes after a `/`.
///
/// # Safety
///
/// `fname` must be a NUL-terminated path and `affile` the affix file that
/// goes with it.
pub unsafe fn spell_read_dic(
    spin: *mut spellinfo_T,
    fname: *mut c_char,
    affile: *mut afffile_T,
) -> c_int {
    // SAFETY: the caller promises the path and the affix file; every buffer
    // below is sized for what is written into it.
    unsafe {
        let fd = os_fopen(fname, c"r".as_ptr());
        if fd.is_null() {
            semsg_c!(gettext((&raw const e_notopen).cast()), fname);
            return FAIL;
        }

        let mut ht: hashtab_T = core::mem::zeroed();
        hash_init(&raw mut ht);
        vim_snprintf(
            IObuff.ptr().cast::<c_char>(),
            IOSIZE as size_t,
            gettext(c"Reading dictionary file %s...".as_ptr()),
            fname,
        );
        spell_message(&*spin, IObuff.ptr().cast::<c_char>());

        // Force the first progress message.
        (*spin).si_msg_count = 999999;

        let mut line: [c_char; MAXLINELEN as usize] = [0; MAXLINELEN as usize];
        // The first line is a word count, which is only checked, not used.
        if vim_fgets(line.as_mut_ptr(), MAXLINELEN, fd)
            || !ascii_isdigit(*skipwhite(line.as_mut_ptr()) as c_int)
        {
            semsg_c!(gettext(c"E760: No word count in %s".as_ptr()), fname);
        }

        let mut store_afflist: [c_char; MAXWLEN] = [0; MAXWLEN];
        let mut message: [c_char; 754] = [0; 754];
        let mut lnum: c_int = 1;
        let mut non_ascii = 0;
        let mut duplicate = 0;
        let mut retval = OK;
        let mut last_msg_time: Timestamp = 0;

        while !vim_fgets(line.as_mut_ptr(), MAXLINELEN, fd) && !got_int.get() {
            line_breakcheck();
            lnum += 1;
            if line[0] as c_int == b'#' as c_int || line[0] as c_int == b'/' as c_int {
                continue;
            }
            // Trim trailing white space, and skip the line if nothing is
            // left.
            let mut l = strlen(line.as_ptr()) as usize;
            while l > 0 && line[l - 1] as uint8_t as c_int <= b' ' as c_int {
                l -= 1;
            }
            if l == 0 {
                continue;
            }
            line[l] = NUL as c_char;

            let mut pc: *mut c_char = core::ptr::null_mut();
            let w = if (*spin).si_conv.vc_type != CONV_NONE {
                pc = string_convert(
                    &raw mut (*spin).si_conv,
                    line.as_mut_ptr(),
                    core::ptr::null_mut(),
                );
                if pc.is_null() {
                    smsg_c!(
                        0,
                        gettext(c"Conversion failure for word in %s line %d: %s".as_ptr()),
                        fname,
                        lnum,
                        line.as_mut_ptr(),
                    );
                    continue;
                }
                pc
            } else {
                line.as_mut_ptr()
            };

            // Split the stem from its affix flags. An escaped "\\" or "\/"
            // is a literal, and collapses here.
            let mut afflist: *mut c_char = core::ptr::null_mut();
            let mut p = w;
            while *p as c_int != NUL {
                if *p as c_int == b'\\' as c_int
                    && (*p.add(1) as c_int == b'\\' as c_int || *p.add(1) as c_int == b'/' as c_int)
                {
                    memmove(p.cast(), p.add(1).cast(), strlen(p.add(1)) + 1);
                } else if *p as c_int == b'/' as c_int {
                    *p = NUL as c_char;
                    afflist = p.add(1);
                    break;
                }
                p = p.add(utfc_ptr2len(p) as usize);
            }

            if (*spin).si_ascii != 0 && has_non_ascii(w) {
                non_ascii += 1;
                xfree(pc.cast());
                continue;
            }

            // Progress, at most once a second.
            if (*spin).si_verbose != 0 && (*spin).si_msg_count > 10000 {
                (*spin).si_msg_count = 0;
                if os_time() > last_msg_time {
                    last_msg_time = os_time();
                    vim_snprintf(
                        message.as_mut_ptr(),
                        core::mem::size_of_val(&message),
                        gettext(c"line %6d, word %6d - %s".as_ptr()),
                        lnum,
                        (*spin).si_foldwcount + (*spin).si_keepwcount,
                        w,
                    );
                    msg_start();
                    msg_outtrans_long(message.as_mut_ptr(), 0);
                    msg_clr_eos();
                    msg_didout.set(false);
                    msg_col.set(0);
                    ui_flush();
                }
            }

            // The word is kept in the arena so the duplicate table can
            // point at it for the rest of the run.
            let dw = (*spin).si_arena.save_str(w);
            if dw.is_null() {
                retval = FAIL;
                xfree(pc.cast());
                break;
            }

            let hash: hash_T = hash_hash(dw);
            let hi: *mut hashitem_T = hash_lookup(&raw mut ht, dw, strlen(dw), hash);
            if (*hi).hi_key.is_null() || (*hi).hi_key == (&raw const hash_removed).cast_mut().cast()
            {
                hash_add_item(&raw mut ht, hi, dw, hash);
            } else {
                // Report every duplicate when 'verbose' is on, otherwise
                // just the first, plus a count at the end.
                if p_verbose.get() > 0 {
                    smsg_c!(
                        0,
                        gettext(c"Duplicate word in %s line %d: %s".as_ptr()),
                        fname,
                        lnum,
                        dw,
                    );
                } else if duplicate == 0 {
                    smsg_c!(
                        0,
                        gettext(c"First duplicate word in %s line %d: %s".as_ptr()),
                        fname,
                        lnum,
                        dw,
                    );
                }
                duplicate += 1;
            }

            let mut flags = 0;
            store_afflist[0] = NUL as c_char;
            let mut pfxlen = 0;
            let mut need_affix = false;
            if !afflist.is_null() {
                flags |= get_affix_flags(affile, afflist);
                if (*affile).af_needaffix != 0
                    && flag_in_afflist((*affile).af_flagtype, afflist, (*affile).af_needaffix)
                {
                    need_affix = true;
                }
                if (*affile).af_pfxpostpone != 0 {
                    pfxlen = get_pfxlist(affile, afflist, store_afflist.as_mut_ptr());
                }
                if !(*spin).si_compflags.is_null() {
                    get_compflags(
                        affile,
                        afflist,
                        store_afflist.as_mut_ptr().offset(pfxlen as isize),
                    );
                }
            }

            let region = (*spin).si_region;
            if store_word(
                &mut *spin,
                dw,
                flags,
                region,
                store_afflist.as_ptr(),
                need_affix,
            ) == FAIL
            {
                retval = FAIL;
            }

            if !afflist.is_null() {
                // Suffixes first, so a prefix can then be put on a word
                // that already has one; then prefixes on the bare stem.
                if store_aff_word(
                    spin,
                    dw,
                    afflist,
                    affile,
                    &raw mut (*affile).af_suff,
                    &raw mut (*affile).af_pref,
                    CONDIT_SUF,
                    flags,
                    store_afflist.as_mut_ptr(),
                    pfxlen,
                ) == FAIL
                {
                    retval = FAIL;
                }
                if store_aff_word(
                    spin,
                    dw,
                    afflist,
                    affile,
                    &raw mut (*affile).af_pref,
                    core::ptr::null_mut(),
                    CONDIT_SUF,
                    flags,
                    store_afflist.as_mut_ptr(),
                    pfxlen,
                ) == FAIL
                {
                    retval = FAIL;
                }
            }
            xfree(pc.cast());
        }

        if duplicate > 0 {
            smsg_c!(
                0,
                gettext(c"%d duplicate word(s) in %s".as_ptr()),
                duplicate,
                fname,
            );
        }
        if (*spin).si_ascii != 0 && non_ascii > 0 {
            smsg_c!(
                0,
                gettext(c"Ignored %d word(s) with non-ASCII characters in %s".as_ptr()),
                non_ascii,
                fname,
            );
        }
        hash_clear(&raw mut ht);
        fclose(fd);
        retval
    }
}

/// Turn the affix flags a word carries into the `WF_*` bits stored with it.
///
/// # Safety
///
/// `afflist` must be a NUL-terminated flag list and `affile` live.
unsafe fn get_affix_flags(affile: *mut afffile_T, afflist: *mut c_char) -> c_int {
    // SAFETY: the caller promises both.
    unsafe {
        let flagtype = (*affile).af_flagtype;
        let mut flags = 0;
        for (declared, bits) in [
            (
                (*affile).af_keepcase,
                WF_KEEPCAP as c_int | WF_FIXCAP as c_int,
            ),
            ((*affile).af_rare, WF_RARE as c_int),
            ((*affile).af_bad, WF_BANNED as c_int),
            ((*affile).af_needcomp, WF_NEEDCOMP as c_int),
            ((*affile).af_comproot, WF_COMPROOT as c_int),
            ((*affile).af_nosuggest, WF_NOSUGGEST as c_int),
        ] {
            // A flag of zero means the affix file never declared it.
            if declared != 0 && flag_in_afflist(flagtype, afflist, declared) {
                flags |= bits;
            }
        }
        flags
    }
}

/// Collect the ids of the postponed prefixes a word accepts, and return how
/// many there were.
///
/// # Safety
///
/// `store_afflist` must have room for one byte per flag in `afflist` plus a
/// terminator.
unsafe fn get_pfxlist(
    affile: *mut afffile_T,
    afflist: *mut c_char,
    store_afflist: *mut c_char,
) -> c_int {
    // SAFETY: the caller promises the output buffer; `key` is AH_KEY_LEN
    // and `get_affitem` never advances past a flag that long.
    unsafe {
        let mut cnt = 0;
        let mut key: [c_char; 17] = [0; 17];
        let mut p = afflist;
        while *p as c_int != NUL {
            let prevp = p;
            if get_affitem((*affile).af_flagtype, &raw mut p) != 0 {
                xmemcpyz(
                    key.as_mut_ptr().cast(),
                    prevp.cast(),
                    p.offset_from(prevp) as size_t,
                );
                let hi = hash_find(&raw mut (*affile).af_pref, key.as_mut_ptr());
                if !(*hi).hi_key.is_null()
                    && (*hi).hi_key != (&raw const hash_removed).cast_mut().cast()
                {
                    // Only prefixes that were actually postponed have an
                    // id; the rest were expanded into the word list.
                    let id = (*(*hi).hi_key.cast::<affheader_T>()).ah_newID;
                    if id != 0 {
                        *store_afflist.offset(cnt as isize) = id as uint8_t as c_char;
                        cnt += 1;
                    }
                }
            }
            if (*affile).af_flagtype == AFT_NUM && *p as c_int == b',' as c_int {
                p = p.add(1);
            }
        }
        *store_afflist.offset(cnt as isize) = NUL as c_char;
        cnt
    }
}

/// Collect the compound ids a word accepts.
///
/// # Safety
///
/// As [`get_pfxlist`].
unsafe fn get_compflags(affile: *mut afffile_T, afflist: *mut c_char, store_afflist: *mut c_char) {
    // SAFETY: as above.
    unsafe {
        let mut cnt = 0;
        let mut key: [c_char; 17] = [0; 17];
        let mut p = afflist;
        while *p as c_int != NUL {
            let prevp = p;
            if get_affitem((*affile).af_flagtype, &raw mut p) != 0 {
                xmemcpyz(
                    key.as_mut_ptr().cast(),
                    prevp.cast(),
                    p.offset_from(prevp) as size_t,
                );
                let hi = hash_find(&raw mut (*affile).af_comp, key.as_mut_ptr());
                if !(*hi).hi_key.is_null()
                    && (*hi).hi_key != (&raw const hash_removed).cast_mut().cast()
                {
                    *store_afflist.offset(cnt as isize) =
                        (*(*hi).hi_key.cast::<compitem_T>()).ci_newID as uint8_t as c_char;
                    cnt += 1;
                }
            }
            if (*affile).af_flagtype == AFT_NUM && *p as c_int == b',' as c_int {
                p = p.add(1);
            }
        }
        *store_afflist.offset(cnt as isize) = NUL as c_char;
    }
}

/// Apply every affix in `ht` that `word` accepts, store each result, and
/// recurse for the affixes that may sit on top.
///
/// `xht` is the *other* table — suffixes when this pass is doing prefixes —
/// and its being non-null is also what tells the body it is adding a prefix
/// rather than a suffix.
///
/// # Safety
///
/// `word` and `afflist` must be NUL-terminated; `ht` and `affile` live;
/// `pfxlist`, when given, must have room past `pfxlen` for more ids.
#[allow(clippy::too_many_arguments)]
pub unsafe fn store_aff_word(
    spin: *mut spellinfo_T,
    word: *mut c_char,
    afflist: *mut c_char,
    affile: *mut afffile_T,
    ht: *mut hashtab_T,
    xht: *mut hashtab_T,
    condit: c_int,
    flags: c_int,
    pfxlist: *mut c_char,
    pfxlen: c_int,
) -> c_int {
    // SAFETY: the caller promises the strings and tables; `newword` is
    // MAXWLEN and every write to it goes through xstrlcpy/xstrlcat with
    // that bound.
    unsafe {
        let mut newword: [c_char; MAXWLEN] = [0; MAXWLEN];
        let mut store_afflist: [c_char; MAXWLEN] = [0; MAXWLEN];
        let mut pfx_pfxlist: [c_char; MAXWLEN] = [0; MAXWLEN];
        let mut retval = OK;
        let wordlen = strlen(word);

        let mut todo = (*ht).ht_used as c_int;
        let mut hi = (*ht).ht_array;
        while todo > 0 && retval == OK {
            if (*hi).hi_key.is_null() || (*hi).hi_key == (&raw const hash_removed).cast_mut().cast()
            {
                hi = hi.add(1);
                continue;
            }
            todo -= 1;
            let ah = (*hi).hi_key.cast::<affheader_T>();

            if (condit & CONDIT_COMB == 0 || (*ah).ah_combine != 0)
                && flag_in_afflist((*affile).af_flagtype, afflist, (*ah).ah_flag)
            {
                let mut ae = (*ah).ah_first;
                while !ae.is_null() {
                    if affix_applies(affile, ae, xht, word, wordlen, condit) {
                        build_affixed_word(&mut newword, word, ae, xht);

                        let mut use_flags = flags;
                        let mut use_pfxlist = pfxlist;
                        let mut use_pfxlen = pfxlen;
                        let mut need_affix = false;
                        let mut use_condit = condit | CONDIT_COMB | CONDIT_AFF;

                        if !(*ae).ae_flags.is_null() {
                            use_flags |= get_affix_flags(affile, (*ae).ae_flags);
                            if (*affile).af_needaffix != 0
                                && flag_in_afflist(
                                    (*affile).af_flagtype,
                                    (*ae).ae_flags,
                                    (*affile).af_needaffix,
                                )
                            {
                                need_affix = true;
                            }
                            if (*affile).af_circumfix != 0
                                && flag_in_afflist(
                                    (*affile).af_flagtype,
                                    (*ae).ae_flags,
                                    (*affile).af_circumfix,
                                )
                            {
                                use_condit |= CONDIT_CFIX;
                                // The first half of a circumfix is not a
                                // word on its own.
                                if condit & CONDIT_CFIX == 0 {
                                    need_affix = true;
                                }
                            }
                            if (*affile).af_pfxpostpone != 0 || !(*spin).si_compflags.is_null() {
                                use_pfxlen = if (*affile).af_pfxpostpone != 0 {
                                    get_pfxlist(affile, (*ae).ae_flags, store_afflist.as_mut_ptr())
                                } else {
                                    0
                                };
                                use_pfxlist = store_afflist.as_mut_ptr();

                                // Merge in the ids the word already had,
                                // skipping any this affix repeats.
                                for i in 0..pfxlen {
                                    let want = *pfxlist.offset(i as isize);
                                    let mut j = 0;
                                    while j < use_pfxlen {
                                        if want == *use_pfxlist.offset(j as isize) {
                                            break;
                                        }
                                        j += 1;
                                    }
                                    if j == use_pfxlen {
                                        *use_pfxlist.offset(use_pfxlen as isize) = want;
                                        use_pfxlen += 1;
                                    }
                                }

                                // The compound ids follow the prefix ids.
                                if !(*spin).si_compflags.is_null() {
                                    get_compflags(
                                        affile,
                                        (*ae).ae_flags,
                                        use_pfxlist.offset(use_pfxlen as isize),
                                    );
                                } else {
                                    *use_pfxlist.offset(use_pfxlen as isize) = NUL as c_char;
                                }
                                let mut i = pfxlen;
                                while *pfxlist.offset(i as isize) as c_int != NUL {
                                    let want = *pfxlist.offset(i as isize);
                                    let mut j = use_pfxlen;
                                    while *use_pfxlist.offset(j as isize) as c_int != NUL {
                                        if want == *use_pfxlist.offset(j as isize) {
                                            break;
                                        }
                                        j += 1;
                                    }
                                    if *use_pfxlist.offset(j as isize) as c_int == NUL {
                                        *use_pfxlist.offset(j as isize) = want;
                                        *use_pfxlist.offset(j as isize + 1) = NUL as c_char;
                                    }
                                    i += 1;
                                }
                            }
                        }

                        // An affix that forbids compounding gets its own
                        // copy of the list, so truncating it below does
                        // not disturb the caller's.
                        if !use_pfxlist.is_null() && (*ae).ae_compforbid as c_int != 0 {
                            xmemcpyz(
                                pfx_pfxlist.as_mut_ptr().cast(),
                                use_pfxlist.cast(),
                                use_pfxlen as size_t,
                            );
                            use_pfxlist = pfx_pfxlist.as_mut_ptr();
                        }

                        if !(*spin).si_prefroot.is_null()
                            && !(*(*spin).si_prefroot).wn_sibling.is_null()
                        {
                            use_flags |= WF_HAS_AFF as c_int;
                            // A non-combinable affix keeps only the
                            // compound ids, not the prefix ids.
                            if (*ah).ah_combine == 0 && !use_pfxlist.is_null() {
                                use_pfxlist = use_pfxlist.offset(use_pfxlen as isize);
                            }
                        }
                        if !(*spin).si_compflags.is_null() && (*ae).ae_comppermit == 0 {
                            use_flags |= if xht.is_null() {
                                WF_NOCOMPBEF as c_int
                            } else {
                                WF_NOCOMPAFT as c_int
                            };
                        }

                        let region = (*spin).si_region;
                        if store_word(
                            &mut *spin,
                            newword.as_mut_ptr(),
                            use_flags,
                            region,
                            use_pfxlist,
                            need_affix,
                        ) == FAIL
                        {
                            retval = FAIL;
                        }

                        // A suffix may follow, if this affix allows one.
                        if condit & CONDIT_SUF != 0 && !(*ae).ae_flags.is_null() {
                            let deeper = use_condit & if xht.is_null() { !0 } else { !CONDIT_SUF };
                            if store_aff_word(
                                spin,
                                newword.as_mut_ptr(),
                                (*ae).ae_flags,
                                affile,
                                &raw mut (*affile).af_suff,
                                xht,
                                deeper,
                                use_flags,
                                use_pfxlist,
                                pfxlen,
                            ) == FAIL
                            {
                                retval = FAIL;
                            }
                        }

                        // And a prefix, when this is the suffix pass and
                        // the suffix is combinable.
                        if !xht.is_null() && (*ah).ah_combine != 0 {
                            let a = store_aff_word(
                                spin,
                                newword.as_mut_ptr(),
                                afflist,
                                affile,
                                xht,
                                core::ptr::null_mut(),
                                use_condit,
                                use_flags,
                                use_pfxlist,
                                pfxlen,
                            );
                            if a == FAIL
                                || (!(*ae).ae_flags.is_null()
                                    && store_aff_word(
                                        spin,
                                        newword.as_mut_ptr(),
                                        (*ae).ae_flags,
                                        affile,
                                        xht,
                                        core::ptr::null_mut(),
                                        use_condit,
                                        use_flags,
                                        use_pfxlist,
                                        pfxlen,
                                    ) == FAIL)
                            {
                                retval = FAIL;
                            }
                        }
                    }
                    ae = (*ae).ae_next;
                }
            }
            hi = hi.add(1);
        }
        retval
    }
}

/// Does this affix entry apply to `word`?
///
/// # Safety
///
/// `ae` must be a live entry and `word` NUL-terminated.
unsafe fn affix_applies(
    affile: *mut afffile_T,
    ae: *mut affentry_T,
    xht: *mut hashtab_T,
    word: *mut c_char,
    wordlen: usize,
    condit: c_int,
) -> bool {
    // SAFETY: the caller promises the entry and the word.
    unsafe {
        // A postponed prefix with nothing to chop, nothing to add and no
        // flags is handled at match time instead.
        if xht.is_null()
            && (*affile).af_pfxpostpone != 0
            && (*ae).ae_chop.is_null()
            && (*ae).ae_flags.is_null()
        {
            return false;
        }
        // There has to be something left after chopping.
        if !(*ae).ae_chop.is_null() && strlen((*ae).ae_chop) >= wordlen {
            return false;
        }
        // The affix's condition must match the word.
        if !(*ae).ae_prog.is_null()
            && !vim_regexec_prog(&raw mut (*ae).ae_prog, false, word, 0 as colnr_T)
        {
            return false;
        }
        // A circumfix affix only counts once its partner is present, and
        // a non-circumfix one only while no partner is expected.
        let at_cfix = condit & CONDIT_CFIX == 0;
        let is_plain = condit & CONDIT_AFF == 0
            || (*ae).ae_flags.is_null()
            || !flag_in_afflist(
                (*affile).af_flagtype,
                (*ae).ae_flags,
                (*affile).af_circumfix,
            );
        at_cfix == is_plain
    }
}

/// Chop and add what one affix says, into `newword`.
///
/// # Safety
///
/// `word` must be NUL-terminated and `ae` a live entry.
unsafe fn build_affixed_word(
    newword: &mut [c_char; MAXWLEN],
    word: *mut c_char,
    ae: *mut affentry_T,
    xht: *mut hashtab_T,
) {
    // SAFETY: every write is bounded by MAXWLEN, the array's size.
    unsafe {
        let cap = MAXWLEN as size_t;
        if xht.is_null() {
            // A prefix: chop from the front, then put the addition before
            // what is left.
            if (*ae).ae_add.is_null() {
                newword[0] = NUL as c_char;
            } else {
                xstrlcpy(newword.as_mut_ptr(), (*ae).ae_add, cap);
            }
            let mut p = word;
            if !(*ae).ae_chop.is_null() {
                for _ in 0..mb_charlen((*ae).ae_chop) {
                    p = p.add(utfc_ptr2len(p) as usize);
                }
            }
            xstrlcat(newword.as_mut_ptr(), p, cap);
            return;
        }

        // A suffix: chop from the end, then append.
        xstrlcpy(newword.as_mut_ptr(), word, cap);
        if !(*ae).ae_chop.is_null() {
            let mut p = newword.as_mut_ptr().add(strlen(newword.as_ptr()));
            for _ in 0..mb_charlen((*ae).ae_chop) {
                // Step back one whole character.
                p = p.offset(-((utf_head_off(newword.as_mut_ptr(), p.sub(1)) + 1) as isize));
            }
            *p = NUL as c_char;
        }
        if !(*ae).ae_add.is_null() {
            xstrlcat(newword.as_mut_ptr(), (*ae).ae_add, cap);
        }
    }
}
