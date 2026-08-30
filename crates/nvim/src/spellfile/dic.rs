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

use crate::cstr;
use crate::semsg;
use crate::smsg;
use core::ffi::{CStr, c_char, c_int};

use crate::ascii::ascii_isdigit;
use crate::charset::skipwhite;
use crate::fileio::vim_fgets;
use crate::hashtab::{
    hash_add_item, hash_clear, hash_find, hash_hash, hash_init, hash_lookup, hash_removed,
};
use crate::main::{got_int, msg_col, msg_didout, p_verbose};
use crate::mbyte::{mb_charlen, string_convert, utf_head_off, utfc_ptr2len};
use crate::memory::{xfree, xmemcpyz, xstrlcat, xstrlcpy};
use crate::message::{msg_clr_eos, msg_outtrans_long, msg_start};
use crate::message_fmt::c_str;
use crate::os::cshim::gettext;
use crate::os::fs::os_fopen;
use crate::os::input::line_breakcheck;
use crate::os::time::os_time;
use crate::strings::{has_non_ascii, vim_snprintf};
use crate::types::{
    CONV_NONE, Failed, NUL, Timestamp, colnr_T, hash_T, hashitem_T, hashtab_T, size_t, uint8_t,
};
use crate::ui::ui_flush;
use ::libc::fclose;

use super::flags::{flag_in_afflist, get_affitem};
use super::wordtree::store_word;
use super::{
    AFT_NUM, CONDIT_AFF, CONDIT_CFIX, CONDIT_COMB, CONDIT_SUF, MAXLINELEN, MAXWLEN, WF_BANNED,
    WF_COMPROOT, WF_FIXCAP, WF_HAS_AFF, WF_KEEPCAP, WF_NEEDCOMP, WF_NOCOMPAFT, WF_NOCOMPBEF,
    WF_NOSUGGEST, WF_RARE, affentry_T, afffile_T, affheader_T, compitem_T, spell_message_fmt,
    spellinfo_T, vim_regexec_prog,
};

/// Read a `.dic` file: a count line, then one stem per line with the affix
/// flags it takes after a `/`.
///
/// # Safety
///
/// `fname` must be a NUL-terminated path and `affile` the affix file that
/// goes with it.
pub(super) unsafe fn spell_read_dic(
    spin: *mut spellinfo_T,
    fname: *mut c_char,
    affile: *mut afffile_T,
) -> Result<(), Failed> {
    // SAFETY: the caller promises the path and the affix file; every buffer
    // below is sized for what is written into it.
    let fd = unsafe { os_fopen(fname, c"r".as_ptr()) };
    if fd.is_null() {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let fname = unsafe { c_str(fname) };
        semsg!("E484: Can't open file {fname}");
        return Err(Failed);
    }

    let mut ht: hashtab_T = unsafe { core::mem::zeroed() };
    unsafe { hash_init(&raw mut ht) };
    let name = unsafe { CStr::from_ptr(fname) }.to_string_lossy();
    spell_message_fmt(
        unsafe { &*spin },
        format_args!("Reading dictionary file {name}..."),
    );

    // Force the first progress message.
    unsafe { (*spin).si_msg_count = 999999 };

    let mut line: [c_char; MAXLINELEN as usize] = [0; MAXLINELEN as usize];
    // The first line is a word count, which is only checked, not used.
    if unsafe { vim_fgets(line.as_mut_ptr(), MAXLINELEN, fd) }
        || !ascii_isdigit(unsafe { *skipwhite(line.as_mut_ptr()) } as c_int)
    {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let fname = unsafe { c_str(fname) };
        semsg!("E760: No word count in {fname}");
    }

    let mut store_afflist: [c_char; MAXWLEN] = [0; MAXWLEN];
    let mut message: [c_char; 754] = [0; 754];
    let mut lnum: c_int = 1;
    let mut non_ascii = 0;
    let mut duplicate = 0;
    let mut retval = Ok(());
    let mut last_msg_time: Timestamp = 0;

    while !unsafe { vim_fgets(line.as_mut_ptr(), MAXLINELEN, fd) } && !got_int.get() {
        line_breakcheck();
        lnum += 1;
        if line[0] as c_int == b'#' as c_int || line[0] as c_int == b'/' as c_int {
            continue;
        }
        // Trim trailing white space, and skip the line if nothing is
        // left.
        let mut l = unsafe { cstr::bytes_at(line.as_ptr()) }.len();
        while l > 0 && line[l - 1] as uint8_t as c_int <= b' ' as c_int {
            l -= 1;
        }
        if l == 0 {
            continue;
        }
        line[l] = NUL as c_char;

        let mut pc: *mut c_char = core::ptr::null_mut();
        let w = if unsafe { (*spin).si_conv.vc_type } != CONV_NONE {
            let conv = unsafe { &raw mut (*spin).si_conv };
            pc = unsafe { string_convert(conv, line.as_mut_ptr(), core::ptr::null_mut()) };
            if pc.is_null() {
                // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
                let (fname, line) = unsafe { (c_str(fname), c_str(line.as_mut_ptr())) };
                smsg!(
                    0,
                    "Conversion failure for word in {fname} line {}: {line}",
                    lnum
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
        while unsafe { *p } as c_int != NUL {
            if unsafe { *p } as c_int == b'\\' as c_int
                && (unsafe { *p.add(1) } as c_int == b'\\' as c_int
                    || unsafe { *p.add(1) } as c_int == b'/' as c_int)
            {
                let n_len = unsafe { cstr::bytes_at(p.add(1)) }.len();
                unsafe { p.cast::<u8>().copy_from(p.add(1).cast(), n_len + 1) };
            } else if unsafe { *p } as c_int == b'/' as c_int {
                unsafe { *p = NUL as c_char };
                afflist = unsafe { p.add(1) };
                break;
            }
            p = unsafe { p.add(utfc_ptr2len(p) as usize) };
        }

        if unsafe { (*spin).si_ascii } != 0 && unsafe { has_non_ascii(w) } {
            non_ascii += 1;
            unsafe { xfree(pc.cast()) };
            continue;
        }

        // Progress, at most once a second.
        if unsafe { (*spin).si_verbose } != 0 && unsafe { (*spin).si_msg_count } > 10000 {
            unsafe { (*spin).si_msg_count = 0 };
            if os_time() > last_msg_time {
                last_msg_time = os_time();
                let (buf, room) = (message.as_mut_ptr(), size_of_val(&message));
                let fmt = gettext(c"line %6d, word %6d - %s");
                let count = unsafe { (*spin).si_foldwcount + (*spin).si_keepwcount };
                unsafe { vim_snprintf(buf, room, fmt.as_ptr(), lnum, count, w) };
                unsafe { msg_start() };
                unsafe { msg_outtrans_long(message.as_mut_ptr(), 0) };
                unsafe { msg_clr_eos() };
                msg_didout.set(false);
                msg_col.set(0);
                unsafe { ui_flush() };
            }
        }

        // The word is kept in the arena so the duplicate table can
        // point at it for the rest of the run.
        let dw = unsafe { (*spin).si_arena.save_str(w) };
        if dw.is_null() {
            retval = Err(Failed);
            unsafe { xfree(pc.cast()) };
            break;
        }

        let hash: hash_T = unsafe { hash_hash(dw) };
        let hi: *mut hashitem_T =
            unsafe { hash_lookup(&raw mut ht, dw, cstr::bytes_at(dw).len(), hash) };
        if unsafe { (*hi).hi_key }.is_null()
            || unsafe { (*hi).hi_key } == (&raw const hash_removed).cast_mut().cast()
        {
            unsafe { hash_add_item(&raw mut ht, hi, dw, hash) };
        } else {
            // Report every duplicate when 'verbose' is on, otherwise
            // just the first, plus a count at the end.
            if p_verbose.get() > 0 {
                // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
                let (fname, dw) = unsafe { (c_str(fname), c_str(dw)) };
                smsg!(0, "Duplicate word in {fname} line {}: {dw}", lnum);
            } else if duplicate == 0 {
                // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
                let (fname, dw) = unsafe { (c_str(fname), c_str(dw)) };
                smsg!(0, "First duplicate word in {fname} line {}: {dw}", lnum);
            }
            duplicate += 1;
        }

        let mut flags = 0;
        store_afflist[0] = NUL as c_char;
        let mut pfxlen = 0;
        let mut need_affix = false;
        if !afflist.is_null() {
            flags |= unsafe { get_affix_flags(affile, afflist) };
            if unsafe { (*affile).af_needaffix } != 0
                && unsafe {
                    flag_in_afflist((*affile).af_flagtype, afflist, (*affile).af_needaffix)
                }
            {
                need_affix = true;
            }
            if unsafe { (*affile).af_pfxpostpone } != 0 {
                pfxlen = unsafe { get_pfxlist(affile, afflist, store_afflist.as_mut_ptr()) };
            }
            if !unsafe { (*spin).si_compflags }.is_null() {
                let at = unsafe { store_afflist.as_mut_ptr().offset(pfxlen as isize) };
                unsafe { get_compflags(affile, afflist, at) };
            }
        }

        let region = unsafe { (*spin).si_region };
        let afx = store_afflist.as_ptr();
        let stored = unsafe { store_word(&mut *spin, dw, flags, region, afx, need_affix) };
        if stored.is_err() {
            retval = Err(Failed);
        }

        if !afflist.is_null() {
            // Suffixes first, so a prefix can then be put on a word
            // that already has one; then prefixes on the bare stem.
            let suff = unsafe { &raw mut (*affile).af_suff };
            let pref = unsafe { &raw mut (*affile).af_pref };
            let mut call = AffWord {
                spin,
                word: dw,
                afflist,
                affile,
                ht: suff,
                xht: pref,
                condit: CONDIT_SUF,
                flags,
                pfxlist: store_afflist.as_mut_ptr(),
                pfxlen,
            };
            if unsafe { store_aff_word(call) }.is_err() {
                retval = Err(Failed);
            }
            call.ht = pref;
            call.xht = core::ptr::null_mut();
            if unsafe { store_aff_word(call) }.is_err() {
                retval = Err(Failed);
            }
        }
        unsafe { xfree(pc.cast()) };
    }

    if duplicate > 0 {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let fname = unsafe { c_str(fname) };
        smsg!(0, "{} duplicate word(s) in {fname}", duplicate);
    }
    if unsafe { (*spin).si_ascii } != 0 && non_ascii > 0 {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let fname = unsafe { c_str(fname) };
        smsg!(
            0,
            "Ignored {} word(s) with non-ASCII characters in {fname}",
            non_ascii
        );
    }
    unsafe { hash_clear(&raw mut ht) };
    unsafe { fclose(fd) };
    retval
}

/// Turn the affix flags a word carries into the `WF_*` bits stored with it.
///
/// # Safety
///
/// `afflist` must be a NUL-terminated flag list and `affile` live.
unsafe fn get_affix_flags(affile: *mut afffile_T, afflist: *mut c_char) -> c_int {
    // SAFETY: the caller promises both.
    let flagtype = unsafe { (*affile).af_flagtype };
    let mut flags = 0;
    for (declared, bits) in [
        (
            unsafe { (*affile).af_keepcase },
            WF_KEEPCAP as c_int | WF_FIXCAP as c_int,
        ),
        (unsafe { (*affile).af_rare }, WF_RARE as c_int),
        (unsafe { (*affile).af_bad }, WF_BANNED as c_int),
        (unsafe { (*affile).af_needcomp }, WF_NEEDCOMP as c_int),
        (unsafe { (*affile).af_comproot }, WF_COMPROOT as c_int),
        (unsafe { (*affile).af_nosuggest }, WF_NOSUGGEST as c_int),
    ] {
        // A flag of zero means the affix file never declared it.
        if declared != 0 && unsafe { flag_in_afflist(flagtype, afflist, declared) } {
            flags |= bits;
        }
    }
    flags
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
    let mut cnt = 0;
    let mut key: [c_char; 17] = [0; 17];
    let mut p = afflist;
    while unsafe { *p } as c_int != NUL {
        let prevp = p;
        if unsafe { get_affitem((*affile).af_flagtype, &raw mut p) } != 0 {
            let len = unsafe { p.offset_from(prevp) } as size_t;
            unsafe { xmemcpyz(key.as_mut_ptr().cast(), prevp.cast(), len) };
            let hi = unsafe { hash_find(&raw mut (*affile).af_pref, key.as_mut_ptr()) };
            if !unsafe { (*hi).hi_key }.is_null()
                && unsafe { (*hi).hi_key } != (&raw const hash_removed).cast_mut().cast()
            {
                // Only prefixes that were actually postponed have an
                // id; the rest were expanded into the word list.
                let id = unsafe { (*affheader_T::of_key((*hi).hi_key)).ah_newID };
                if id != 0 {
                    unsafe { *store_afflist.offset(cnt as isize) = id as uint8_t as c_char };
                    cnt += 1;
                }
            }
        }
        if unsafe { (*affile).af_flagtype } == AFT_NUM && unsafe { *p } as c_int == b',' as c_int {
            p = unsafe { p.add(1) };
        }
    }
    unsafe { *store_afflist.offset(cnt as isize) = NUL as c_char };
    cnt
}

/// Collect the compound ids a word accepts.
///
/// # Safety
///
/// As [`get_pfxlist`].
unsafe fn get_compflags(affile: *mut afffile_T, afflist: *mut c_char, store_afflist: *mut c_char) {
    // SAFETY: as above.
    let mut cnt = 0;
    let mut key: [c_char; 17] = [0; 17];
    let mut p = afflist;
    while unsafe { *p } as c_int != NUL {
        let prevp = p;
        if unsafe { get_affitem((*affile).af_flagtype, &raw mut p) } != 0 {
            let len = unsafe { p.offset_from(prevp) } as size_t;
            unsafe { xmemcpyz(key.as_mut_ptr().cast(), prevp.cast(), len) };
            let hi = unsafe { hash_find(&raw mut (*affile).af_comp, key.as_mut_ptr()) };
            if !unsafe { (*hi).hi_key }.is_null()
                && unsafe { (*hi).hi_key } != (&raw const hash_removed).cast_mut().cast()
            {
                unsafe {
                    *store_afflist.offset(cnt as isize) =
                        (*compitem_T::of_key((*hi).hi_key)).ci_newID as uint8_t as c_char
                };
                cnt += 1;
            }
        }
        if unsafe { (*affile).af_flagtype } == AFT_NUM && unsafe { *p } as c_int == b',' as c_int {
            p = unsafe { p.add(1) };
        }
    }
    unsafe { *store_afflist.offset(cnt as isize) = NUL as c_char };
}

/// One call's worth of [`store_aff_word`] arguments.
///
/// The C carried these ten positionally and the transpiled call was nine
/// lines of `unsafe` region apiece. Naming them puts the call back on one
/// line and says at each site which table is being applied and which is
/// tried on top of it.
#[derive(Copy, Clone)]
pub(super) struct AffWord {
    /// The word list being built.
    pub spin: *mut spellinfo_T,
    /// The stem to affix, and the flags it declared.
    pub word: *mut c_char,
    pub afflist: *mut c_char,
    /// The `.aff` file the affixes came from.
    pub affile: *mut afffile_T,
    /// The affix table to apply, and the *other* one -- suffixes when this
    /// pass is doing prefixes. `xht` being non-null is also what tells the
    /// body it is adding a prefix rather than a suffix.
    pub ht: *mut hashtab_T,
    pub xht: *mut hashtab_T,
    /// Which conditions the affix has to meet, and the word's `WF_*` flags.
    pub condit: c_int,
    pub flags: c_int,
    /// The affix ids collected so far, and how many there are.
    pub pfxlist: *mut c_char,
    pub pfxlen: c_int,
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
pub(super) unsafe fn store_aff_word(call: AffWord) -> Result<(), Failed> {
    let AffWord {
        spin,
        word,
        afflist,
        affile,
        ht,
        xht,
        condit,
        flags,
        pfxlist,
        pfxlen,
    } = call;
    // SAFETY: the caller promises the strings and tables; `newword` is
    // MAXWLEN and every write to it goes through xstrlcpy/xstrlcat with
    // that bound.
    let mut newword: [c_char; MAXWLEN] = [0; MAXWLEN];
    let mut store_afflist: [c_char; MAXWLEN] = [0; MAXWLEN];
    let mut pfx_pfxlist: [c_char; MAXWLEN] = [0; MAXWLEN];
    let mut retval = Ok(());
    let wordlen = unsafe { cstr::bytes_at(word) }.len();

    let mut todo = unsafe { (*ht).ht_used } as c_int;
    let mut hi = unsafe { (*ht).ht_array };
    while todo > 0 && retval.is_ok() {
        if unsafe { (*hi).hi_key }.is_null()
            || unsafe { (*hi).hi_key } == (&raw const hash_removed).cast_mut().cast()
        {
            hi = unsafe { hi.add(1) };
            continue;
        }
        todo -= 1;
        let ah = unsafe { affheader_T::of_key((*hi).hi_key) };

        if (condit & CONDIT_COMB == 0 || unsafe { (*ah).ah_combine } != 0)
            && unsafe { flag_in_afflist((*affile).af_flagtype, afflist, (*ah).ah_flag) }
        {
            let mut ae = unsafe { (*ah).ah_first };
            while !ae.is_null() {
                if unsafe { affix_applies(affile, ae, xht, word, wordlen, condit) } {
                    unsafe { build_affixed_word(&mut newword, word, ae, xht) };

                    let mut use_flags = flags;
                    let mut use_pfxlist = pfxlist;
                    let mut use_pfxlen = pfxlen;
                    let mut need_affix = false;
                    let mut use_condit = condit | CONDIT_COMB | CONDIT_AFF;

                    if !unsafe { (*ae).ae_flags }.is_null() {
                        use_flags |= unsafe { get_affix_flags(affile, (*ae).ae_flags) };
                        if unsafe { (*affile).af_needaffix } != 0
                            && unsafe {
                                flag_in_afflist(
                                    (*affile).af_flagtype,
                                    (*ae).ae_flags,
                                    (*affile).af_needaffix,
                                )
                            }
                        {
                            need_affix = true;
                        }
                        if unsafe { (*affile).af_circumfix } != 0
                            && unsafe {
                                flag_in_afflist(
                                    (*affile).af_flagtype,
                                    (*ae).ae_flags,
                                    (*affile).af_circumfix,
                                )
                            }
                        {
                            use_condit |= CONDIT_CFIX;
                            // The first half of a circumfix is not a
                            // word on its own.
                            if condit & CONDIT_CFIX == 0 {
                                need_affix = true;
                            }
                        }
                        if unsafe { (*affile).af_pfxpostpone } != 0
                            || !unsafe { (*spin).si_compflags }.is_null()
                        {
                            let ae_flags = unsafe { (*ae).ae_flags };
                            let afx = store_afflist.as_mut_ptr();
                            use_pfxlen = if unsafe { (*affile).af_pfxpostpone } != 0 {
                                unsafe { get_pfxlist(affile, ae_flags, afx) }
                            } else {
                                0
                            };
                            use_pfxlist = store_afflist.as_mut_ptr();

                            // Merge in the ids the word already had,
                            // skipping any this affix repeats.
                            for i in 0..pfxlen {
                                let want = unsafe { *pfxlist.offset(i as isize) };
                                let mut j = 0;
                                while j < use_pfxlen {
                                    if want == unsafe { *use_pfxlist.offset(j as isize) } {
                                        break;
                                    }
                                    j += 1;
                                }
                                if j == use_pfxlen {
                                    unsafe { *use_pfxlist.offset(use_pfxlen as isize) = want };
                                    use_pfxlen += 1;
                                }
                            }

                            // The compound ids follow the prefix ids.
                            if !unsafe { (*spin).si_compflags }.is_null() {
                                let flags = unsafe { (*ae).ae_flags };
                                let at = unsafe { use_pfxlist.offset(use_pfxlen as isize) };
                                unsafe { get_compflags(affile, flags, at) };
                            } else {
                                unsafe { *use_pfxlist.offset(use_pfxlen as isize) = NUL as c_char };
                            }
                            let mut i = pfxlen;
                            while unsafe { *pfxlist.offset(i as isize) } as c_int != NUL {
                                let want = unsafe { *pfxlist.offset(i as isize) };
                                let mut j = use_pfxlen;
                                while unsafe { *use_pfxlist.offset(j as isize) } as c_int != NUL {
                                    if want == unsafe { *use_pfxlist.offset(j as isize) } {
                                        break;
                                    }
                                    j += 1;
                                }
                                if unsafe { *use_pfxlist.offset(j as isize) } as c_int == NUL {
                                    unsafe { *use_pfxlist.offset(j as isize) = want };
                                    unsafe { *use_pfxlist.offset(j as isize + 1) = NUL as c_char };
                                }
                                i += 1;
                            }
                        }
                    }

                    // An affix that forbids compounding gets its own
                    // copy of the list, so truncating it below does
                    // not disturb the caller's.
                    if !use_pfxlist.is_null() && unsafe { (*ae).ae_compforbid } as c_int != 0 {
                        let to = pfx_pfxlist.as_mut_ptr().cast();
                        unsafe { xmemcpyz(to, use_pfxlist.cast(), use_pfxlen as size_t) };
                        use_pfxlist = pfx_pfxlist.as_mut_ptr();
                    }

                    if !unsafe { (*spin).si_prefroot }.is_null()
                        && !unsafe { (*(*spin).si_prefroot).wn_sibling }.is_null()
                    {
                        use_flags |= WF_HAS_AFF as c_int;
                        // A non-combinable affix keeps only the
                        // compound ids, not the prefix ids.
                        if unsafe { (*ah).ah_combine } == 0 && !use_pfxlist.is_null() {
                            use_pfxlist = unsafe { use_pfxlist.offset(use_pfxlen as isize) };
                        }
                    }
                    if !unsafe { (*spin).si_compflags }.is_null()
                        && unsafe { (*ae).ae_comppermit } == 0
                    {
                        use_flags |= if xht.is_null() {
                            WF_NOCOMPBEF as c_int
                        } else {
                            WF_NOCOMPAFT as c_int
                        };
                    }

                    let region = unsafe { (*spin).si_region };
                    let nw = newword.as_mut_ptr();
                    let stored = unsafe {
                        store_word(&mut *spin, nw, use_flags, region, use_pfxlist, need_affix)
                    };
                    if stored.is_err() {
                        retval = Err(Failed);
                    }

                    // A suffix may follow, if this affix allows one.
                    if condit & CONDIT_SUF != 0 && !unsafe { (*ae).ae_flags }.is_null() {
                        let deeper = use_condit & if xht.is_null() { !0 } else { !CONDIT_SUF };
                        let deep = AffWord {
                            spin,
                            word: newword.as_mut_ptr(),
                            afflist: unsafe { (*ae).ae_flags },
                            affile,
                            ht: unsafe { &raw mut (*affile).af_suff },
                            xht,
                            condit: deeper,
                            flags: use_flags,
                            pfxlist: use_pfxlist,
                            pfxlen,
                        };
                        if unsafe { store_aff_word(deep) }.is_err() {
                            retval = Err(Failed);
                        }
                    }

                    // And a prefix, when this is the suffix pass and
                    // the suffix is combinable.
                    if !xht.is_null() && unsafe { (*ah).ah_combine } != 0 {
                        let mut cross = AffWord {
                            spin,
                            word: newword.as_mut_ptr(),
                            afflist,
                            affile,
                            ht: xht,
                            xht: core::ptr::null_mut(),
                            condit: use_condit,
                            flags: use_flags,
                            pfxlist: use_pfxlist,
                            pfxlen,
                        };
                        let a = unsafe { store_aff_word(cross) };
                        cross.afflist = unsafe { (*ae).ae_flags };
                        if a.is_err()
                            || (!cross.afflist.is_null()
                                && unsafe { store_aff_word(cross) }.is_err())
                        {
                            retval = Err(Failed);
                        }
                    }
                }
                ae = unsafe { (*ae).ae_next };
            }
        }
        hi = unsafe { hi.add(1) };
    }
    retval
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
    // A postponed prefix with nothing to chop, nothing to add and no
    // flags is handled at match time instead.
    if xht.is_null()
        && unsafe { (*affile).af_pfxpostpone } != 0
        && unsafe { (*ae).ae_chop }.is_null()
        && unsafe { (*ae).ae_flags }.is_null()
    {
        return false;
    }
    // There has to be something left after chopping.
    if !unsafe { (*ae).ae_chop }.is_null()
        && unsafe { cstr::bytes_at((*ae).ae_chop) }.len() >= wordlen
    {
        return false;
    }
    // The affix's condition must match the word.
    if !unsafe { (*ae).ae_prog }.is_null()
        && !unsafe { vim_regexec_prog(&raw mut (*ae).ae_prog, false, word, 0 as colnr_T) }
    {
        return false;
    }
    // A circumfix affix only counts once its partner is present, and
    // a non-circumfix one only while no partner is expected.
    let at_cfix = condit & CONDIT_CFIX == 0;
    let is_plain = condit & CONDIT_AFF == 0
        || unsafe { (*ae).ae_flags }.is_null()
        || !unsafe {
            flag_in_afflist(
                (*affile).af_flagtype,
                (*ae).ae_flags,
                (*affile).af_circumfix,
            )
        };
    at_cfix == is_plain
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
    let cap = MAXWLEN as size_t;
    if xht.is_null() {
        // A prefix: chop from the front, then put the addition before
        // what is left.
        if unsafe { (*ae).ae_add }.is_null() {
            newword[0] = NUL as c_char;
        } else {
            unsafe { xstrlcpy(newword.as_mut_ptr(), (*ae).ae_add, cap) };
        }
        let mut p = word;
        if !unsafe { (*ae).ae_chop }.is_null() {
            for _ in 0..unsafe { mb_charlen((*ae).ae_chop) } {
                p = unsafe { p.add(utfc_ptr2len(p) as usize) };
            }
        }
        unsafe { xstrlcat(newword.as_mut_ptr(), p, cap) };
        return;
    }

    // A suffix: chop from the end, then append.
    unsafe { xstrlcpy(newword.as_mut_ptr(), word, cap) };
    if !unsafe { (*ae).ae_chop }.is_null() {
        let ptr_len = unsafe { cstr::bytes_at(newword.as_ptr()) }.len();
        let mut p = unsafe { newword.as_mut_ptr().add(ptr_len) };
        for _ in 0..unsafe { mb_charlen((*ae).ae_chop) } {
            // Step back one whole character.
            p = unsafe { p.offset(-((utf_head_off(newword.as_mut_ptr(), p.sub(1)) + 1) as isize)) };
        }
        unsafe { *p = NUL as c_char };
    }
    if !unsafe { (*ae).ae_add }.is_null() {
        unsafe { xstrlcat(newword.as_mut_ptr(), (*ae).ae_add, cap) };
    }
}
