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

#![forbid(unsafe_code)]

use crate::cstr;
use crate::semsg;
use crate::smsg;
use crate::spell::WordFlags;
use crate::tr;
use core::ffi::{CStr, c_int};
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::File;
use std::os::unix::ffi::OsStrExt;

use crate::cstr::byte_at;
use crate::fileio::Fgets;
use crate::getchar::state::got_int;
use crate::mbyte::{c_cluster_len, char_count, head_off};
use crate::message::state::{msg_col, msg_didout};
use crate::message::{msg_clr_eos, msg_display_elided, msg_start};
use crate::message_fmt::{msg_bytes, msg_cstr};
use crate::option::vars::p_verbose;
use crate::os::input::line_breakcheck;
use crate::os::time::os_time;
use crate::strings::any_non_ascii;
use crate::types::{CONV_NONE, Failed, NUL, Timestamp};
use crate::ui::ui_flush;

use super::flags::{flag_in_afflist, get_affitem};
use super::wordtree::store_word;
use super::{
    AFT_NUM, AffEntry, AffFile, AffixKind, CONDIT_AFF, CONDIT_CFIX, CONDIT_COMB, CONDIT_SUF,
    MAXLINELEN, MAXWLEN, SpellInfo, spell_message_fmt, vim_regexec_prog,
};

/// The size of upstream's buffer for the progress line.
const PROGRESS_ROOM: usize = 754;

/// Read a `.dic` file: a count line, then one stem per line with the affix
/// flags it takes after a `/`.
pub(super) fn spell_read_dic(
    spin: &mut SpellInfo,
    fname: &CStr,
    affile: &AffFile,
) -> Result<(), Failed> {
    let Ok(file) = File::open(OsStr::from_bytes(fname.to_bytes())) else {
        let fname = msg_cstr(fname);
        semsg!("E484: Can't open file {fname}");
        return Err(Failed);
    };
    let mut fd = Fgets::new(file);

    // The words seen so far, to report duplicates.
    let mut seen: HashSet<Vec<u8>> = HashSet::new();
    let name = fname.to_string_lossy();
    spell_message_fmt(&*spin, format_args!("Reading dictionary file {name}..."));

    // Force the first progress message.
    spin.si_msg_count = 999999;

    let mut raw: Vec<u8> = Vec::new();
    // The first line is a word count, which is only checked, not used.
    let counted = fd.line_capped(&mut raw, MAXLINELEN as usize) && {
        let line = cstr::until_nul(&raw);
        byte_at(line, crate::charset::skip::white(line)).is_ascii_digit()
    };
    if !counted {
        let fname = msg_cstr(fname);
        semsg!("E760: No word count in {fname}");
    }

    // Kept across lines, as upstream's was: past the terminator it holds
    // whatever an earlier line left, which the affix walk can read.
    let mut store_afflist = [0u8; MAXWLEN];
    let mut lnum: c_int = 1;
    let mut non_ascii = 0;
    let mut duplicate = 0;
    let mut retval = Ok(());
    let mut last_msg_time: Timestamp = 0;

    while fd.line_capped(&mut raw, MAXLINELEN as usize) && !got_int.get() {
        line_breakcheck();
        lnum += 1;
        let read = cstr::until_nul(&raw);
        if matches!(read.first(), Some(b'#' | b'/')) {
            continue;
        }
        // Trim trailing white space, and skip the line if nothing is
        // left.
        let l = read.iter().rposition(|&b| b > b' ').map_or(0, |at| at + 1);
        if l == 0 {
            continue;
        }
        let line = &read[..l];

        let mut w: Vec<u8> = if spin.si_conv.vc_type != CONV_NONE {
            let Some(converted) = spin.si_conv.convert(line) else {
                let (fname, line) = (msg_cstr(fname), msg_bytes(line));
                smsg!(
                    0,
                    "Conversion failure for word in {fname} line {}: {line}",
                    lnum
                );
                continue;
            };
            cstr::until_nul(&converted).to_vec()
        } else {
            line.to_vec()
        };

        // Split the stem from its affix flags. An escaped "\\" or "\/"
        // is a literal, and collapses here.
        let mut afflist: Option<Vec<u8>> = None;
        let mut p = 0;
        while p < w.len() {
            if w[p] == b'\\' && matches!(byte_at(&w, p + 1), b'\\' | b'/') {
                w.remove(p);
            } else if w[p] == b'/' {
                afflist = Some(w[p + 1..].to_vec());
                w.truncate(p);
                break;
            }
            p += c_cluster_len(&w[p..]);
        }

        if spin.si_ascii != 0 && any_non_ascii(&w) {
            non_ascii += 1;
            continue;
        }

        // Progress, at most once a second.
        if spin.si_verbose != 0 && spin.si_msg_count > 10000 {
            spin.si_msg_count = 0;
            if os_time() > last_msg_time {
                last_msg_time = os_time();
                let count = spin.si_foldwcount + spin.si_keepwcount;
                let shown = msg_bytes(&w);
                let mut message = tr!("line {:6}, word {:6} - {}", lnum, count, shown).into_bytes();
                message.truncate(PROGRESS_ROOM - 1);
                msg_start();
                msg_display_elided(&cstr::owned(cstr::until_nul(&message)), 0);
                msg_clr_eos();
                msg_didout.set(false);
                msg_col.set(0);
                ui_flush();
            }
        }

        // Upstream kept the word in the arena so its duplicate table could
        // point at it for the rest of the run.
        spin.si_arena.charge_str(&w);
        if !seen.insert(w.clone()) {
            // Report every duplicate when 'verbose' is on, otherwise
            // just the first, plus a count at the end.
            if p_verbose() > 0 {
                let (fname, dw) = (msg_cstr(fname), msg_bytes(&w));
                smsg!(0, "Duplicate word in {fname} line {}: {dw}", lnum);
            } else if duplicate == 0 {
                let (fname, dw) = (msg_cstr(fname), msg_bytes(&w));
                smsg!(0, "First duplicate word in {fname} line {}: {dw}", lnum);
            }
            duplicate += 1;
        }

        let mut flags = WordFlags::NONE;
        store_afflist[0] = NUL as u8;
        let mut pfxlen = 0;
        let mut need_affix = false;
        if let Some(afflist) = &afflist {
            flags |= get_affix_flags(affile, afflist);
            if affile.af_needaffix != 0
                && flag_in_afflist(affile.af_flagtype, afflist, affile.af_needaffix)
            {
                need_affix = true;
            }
            if affile.af_pfxpostpone != 0 {
                pfxlen = get_pfxlist(affile, afflist, &mut store_afflist);
            }
            if spin.si_compflags.is_some() {
                get_compflags(affile, afflist, &mut store_afflist[pfxlen..]);
            }
        }

        let region = spin.si_region;
        if store_word(&mut *spin, &w, flags, region, &store_afflist, need_affix).is_err() {
            retval = Err(Failed);
        }

        if let Some(afflist) = &afflist {
            w.push(NUL as u8);
            let word = CStr::from_bytes_with_nul(&w).expect("the stem holds no NUL");
            // Suffixes first, so a prefix can then be put on a word
            // that already has one; then prefixes on the bare stem.
            let mut call = AffWord {
                word,
                afflist,
                ht: AffixKind::Suffix,
                xht: Some(AffixKind::Prefix),
                condit: CONDIT_SUF,
                flags,
                pfxlist: &store_afflist,
                pfxlen,
            };
            if store_aff_word(spin, affile, call).is_err() {
                retval = Err(Failed);
            }
            call.ht = AffixKind::Prefix;
            call.xht = None;
            if store_aff_word(spin, affile, call).is_err() {
                retval = Err(Failed);
            }
        }
    }

    if duplicate > 0 {
        let fname = msg_cstr(fname);
        smsg!(0, "{} duplicate word(s) in {fname}", duplicate);
    }
    if spin.si_ascii != 0 && non_ascii > 0 {
        let fname = msg_cstr(fname);
        smsg!(
            0,
            "Ignored {} word(s) with non-ASCII characters in {fname}",
            non_ascii
        );
    }
    retval
}

/// Turn the affix flags a word carries into the `WF_*` bits stored with it.
fn get_affix_flags(affile: &AffFile, afflist: &[u8]) -> WordFlags {
    let flagtype = affile.af_flagtype;
    let mut flags = WordFlags::NONE;
    for (declared, bits) in [
        (affile.af_keepcase, WordFlags::KEEPCAP | WordFlags::FIXCAP),
        (affile.af_rare, WordFlags::RARE),
        (affile.af_bad, WordFlags::BANNED),
        (affile.af_needcomp, WordFlags::NEEDCOMP),
        (affile.af_comproot, WordFlags::COMPROOT),
        (affile.af_nosuggest, WordFlags::NOSUGGEST),
    ] {
        // A flag of zero means the affix file never declared it.
        if declared != 0 && flag_in_afflist(flagtype, afflist, declared) {
            flags |= bits;
        }
    }
    flags
}

/// Write `byte` at `at` of an id list, where upstream's array had room.
fn put(list: &mut [u8], at: usize, byte: u8) {
    if let Some(slot) = list.get_mut(at) {
        *slot = byte;
    }
}

/// Collect the ids of the postponed prefixes a word accepts into `out`,
/// terminated, and return how many there were.
fn get_pfxlist(affile: &AffFile, afflist: &[u8], out: &mut [u8]) -> usize {
    let mut cnt = 0;
    let mut p = 0;
    while byte_at(afflist, p) != NUL as u8 {
        let prevp = p;
        if get_affitem(affile.af_flagtype, afflist, &mut p) != 0
            && let Some(at) = affile.af_pref.find(&afflist[prevp..p.min(afflist.len())])
        {
            // Only prefixes that were actually postponed have an id;
            // the rest were expanded into the word list.
            let id = affile.af_pref.get(at).ah_new_id;
            if id != 0 {
                put(out, cnt, id as u8);
                cnt += 1;
            }
        }
        if affile.af_flagtype == AFT_NUM && byte_at(afflist, p) == b',' {
            p += 1;
        }
    }
    put(out, cnt, NUL as u8);
    cnt
}

/// Collect the compound ids a word accepts into `out`, terminated.
fn get_compflags(affile: &AffFile, afflist: &[u8], out: &mut [u8]) {
    let mut cnt = 0;
    let mut p = 0;
    while byte_at(afflist, p) != NUL as u8 {
        let prevp = p;
        if get_affitem(affile.af_flagtype, afflist, &mut p) != 0
            && let Some(at) = affile.af_comp.find(&afflist[prevp..p.min(afflist.len())])
        {
            put(out, cnt, affile.af_comp.get(at).ci_new_id as u8);
            cnt += 1;
        }
        if affile.af_flagtype == AFT_NUM && byte_at(afflist, p) == b',' {
            p += 1;
        }
    }
    put(out, cnt, NUL as u8);
}

/// One call's worth of [`store_aff_word`] arguments.
#[derive(Copy, Clone)]
struct AffWord<'a> {
    /// The stem to affix, and the flags it declared.
    word: &'a CStr,
    afflist: &'a [u8],
    /// The affix table to apply, and -- on a suffix pass -- the prefix
    /// table a combinable suffix may take a prefix from. `xht` being unset
    /// is also what tells the body it is adding a prefix.
    ht: AffixKind,
    xht: Option<AffixKind>,
    /// Which conditions the affix has to meet, and the word's `WF_*` flags.
    condit: c_int,
    flags: WordFlags,
    /// The affix ids collected so far, and how many of them are prefix
    /// ids. The slice runs to the end of the array upstream's pointer
    /// pointed into, so past the terminator it reads what that array held.
    pfxlist: &'a [u8],
    pfxlen: usize,
}

/// Where a pass's id list lives: the caller's, or one of this pass's two
/// arrays, from an offset.
#[derive(Copy, Clone)]
enum IdList {
    Caller(usize),
    Store(usize),
    Forbid(usize),
}

impl IdList {
    /// The list, from its offset to the end of its array.
    fn read<'a>(self, caller: &'a [u8], store: &'a [u8], forbid: &'a [u8]) -> &'a [u8] {
        let (list, at) = match self {
            IdList::Caller(at) => (caller, at),
            IdList::Store(at) => (store, at),
            IdList::Forbid(at) => (forbid, at),
        };
        list.get(at..).unwrap_or_default()
    }

    fn skip(self, n: usize) -> Self {
        match self {
            IdList::Caller(at) => IdList::Caller(at + n),
            IdList::Store(at) => IdList::Store(at + n),
            IdList::Forbid(at) => IdList::Forbid(at + n),
        }
    }
}

/// Apply every affix in `ht` that `word` accepts, store each result, and
/// recurse for the affixes that may sit on top.
fn store_aff_word(spin: &mut SpellInfo, affile: &AffFile, call: AffWord<'_>) -> Result<(), Failed> {
    let AffWord {
        word,
        afflist,
        ht,
        xht,
        condit,
        flags,
        pfxlist,
        pfxlen,
    } = call;
    let mut newword = [0u8; MAXWLEN];
    let mut store_afflist = [0u8; MAXWLEN];
    let mut pfx_pfxlist = [0u8; MAXWLEN];
    let mut retval = Ok(());
    let wordlen = word.count_bytes();

    for ah in affile.affixes(ht).iter() {
        if retval.is_err() {
            break;
        }
        if (condit & CONDIT_COMB != 0 && ah.ah_combine == 0)
            || !flag_in_afflist(affile.af_flagtype, afflist, ah.ah_flag)
        {
            continue;
        }
        for ae in ah.ah_entries.iter().rev() {
            if !affix_applies(affile, ae, xht, word, wordlen, condit) {
                continue;
            }
            build_affixed_word(&mut newword, word.to_bytes(), ae, xht.is_none());

            let mut use_flags = flags;
            let mut use_list = IdList::Caller(0);
            let mut use_pfxlen = pfxlen;
            let mut need_affix = false;
            let mut use_condit = condit | CONDIT_COMB | CONDIT_AFF;

            if let Some(ae_flags) = &ae.ae_flags {
                use_flags |= get_affix_flags(affile, ae_flags);
                if affile.af_needaffix != 0
                    && flag_in_afflist(affile.af_flagtype, ae_flags, affile.af_needaffix)
                {
                    need_affix = true;
                }
                if affile.af_circumfix != 0
                    && flag_in_afflist(affile.af_flagtype, ae_flags, affile.af_circumfix)
                {
                    use_condit |= CONDIT_CFIX;
                    // The first half of a circumfix is not a word on its
                    // own.
                    if condit & CONDIT_CFIX == 0 {
                        need_affix = true;
                    }
                }
                if affile.af_pfxpostpone != 0 || spin.si_compflags.is_some() {
                    use_pfxlen = if affile.af_pfxpostpone != 0 {
                        get_pfxlist(affile, ae_flags, &mut store_afflist)
                    } else {
                        0
                    };
                    use_list = IdList::Store(0);
                    let list = &mut store_afflist;

                    // Merge in the ids the word already had, skipping any
                    // this affix repeats.
                    for i in 0..pfxlen {
                        let want = byte_at(pfxlist, i);
                        if !list[..use_pfxlen.min(MAXWLEN)].contains(&want) {
                            put(list, use_pfxlen, want);
                            use_pfxlen += 1;
                        }
                    }

                    // The compound ids follow the prefix ids.
                    if spin.si_compflags.is_some() {
                        get_compflags(
                            affile,
                            ae_flags,
                            list.get_mut(use_pfxlen..).unwrap_or_default(),
                        );
                    } else {
                        put(list, use_pfxlen, NUL as u8);
                    }
                    let mut i = pfxlen;
                    while byte_at(pfxlist, i) != NUL as u8 {
                        let want = byte_at(pfxlist, i);
                        let mut j = use_pfxlen;
                        while byte_at(list, j) != NUL as u8 {
                            if want == list[j] {
                                break;
                            }
                            j += 1;
                        }
                        if byte_at(list, j) == NUL as u8 {
                            put(list, j, want);
                            put(list, j + 1, NUL as u8);
                        }
                        i += 1;
                    }
                }
            }

            // An affix that forbids compounding gets its own copy of the
            // prefix ids, so the compound ids are dropped from it.
            if ae.ae_compforbid {
                let mut copy = [0u8; MAXWLEN];
                let n = use_pfxlen.min(MAXWLEN);
                let from = use_list.read(pfxlist, &store_afflist, &pfx_pfxlist);
                for (k, byte) in copy[..n].iter_mut().enumerate() {
                    *byte = byte_at(from, k);
                }
                pfx_pfxlist[..n].copy_from_slice(&copy[..n]);
                put(&mut pfx_pfxlist, use_pfxlen, NUL as u8);
                use_list = IdList::Forbid(0);
            }

            let has_prefixes = spin
                .si_prefroot
                .is_some_and(|root| spin.node(root).wn_sibling.is_some());
            if has_prefixes {
                use_flags |= WordFlags::HAS_AFF;
                // A non-combinable affix keeps only the compound ids, not
                // the prefix ids.
                if ah.ah_combine == 0 {
                    use_list = use_list.skip(use_pfxlen);
                }
            }
            if spin.si_compflags.is_some() && !ae.ae_comppermit {
                use_flags |= if xht.is_none() {
                    WordFlags::NOCOMPBEF
                } else {
                    WordFlags::NOCOMPAFT
                };
            }

            let use_pfxlist = use_list.read(pfxlist, &store_afflist, &pfx_pfxlist);
            let newword = CStr::from_bytes_until_nul(&newword).expect("the word is terminated");
            let region = spin.si_region;
            if store_word(
                &mut *spin,
                newword.to_bytes(),
                use_flags,
                region,
                use_pfxlist,
                need_affix,
            )
            .is_err()
            {
                retval = Err(Failed);
            }

            // A suffix may follow, if this affix allows one.
            if condit & CONDIT_SUF != 0
                && let Some(ae_flags) = &ae.ae_flags
            {
                let deeper = use_condit & if xht.is_none() { !0 } else { !CONDIT_SUF };
                let deep = AffWord {
                    word: newword,
                    afflist: ae_flags,
                    ht: AffixKind::Suffix,
                    xht,
                    condit: deeper,
                    flags: use_flags,
                    pfxlist: use_pfxlist,
                    pfxlen,
                };
                if store_aff_word(spin, affile, deep).is_err() {
                    retval = Err(Failed);
                }
            }

            // And a prefix, when this is the suffix pass and the suffix
            // is combinable.
            if let Some(other) = xht
                && ah.ah_combine != 0
            {
                let mut cross = AffWord {
                    word: newword,
                    afflist,
                    ht: other,
                    xht: None,
                    condit: use_condit,
                    flags: use_flags,
                    pfxlist: use_pfxlist,
                    pfxlen,
                };
                let a = store_aff_word(spin, affile, cross);
                let b = ae.ae_flags.as_deref().map(|flags| {
                    cross.afflist = flags;
                    store_aff_word(spin, affile, cross)
                });
                if a.is_err() || b.is_some_and(|b| b.is_err()) {
                    retval = Err(Failed);
                }
            }
        }
    }
    retval
}

/// Does this affix entry apply to `word`?
fn affix_applies(
    affile: &AffFile,
    ae: &AffEntry,
    xht: Option<AffixKind>,
    word: &CStr,
    wordlen: usize,
    condit: c_int,
) -> bool {
    // A postponed prefix with nothing to chop, nothing to add and no
    // flags is handled at match time instead.
    if xht.is_none() && affile.af_pfxpostpone != 0 && ae.ae_chop.is_none() && ae.ae_flags.is_none()
    {
        return false;
    }
    // There has to be something left after chopping.
    if ae
        .ae_chop
        .as_ref()
        .is_some_and(|chop| chop.len() >= wordlen)
    {
        return false;
    }
    // The affix's condition must match the word.
    {
        let mut prog = ae.ae_prog.borrow_mut();
        if prog.is_some() && !vim_regexec_prog(&mut prog, false, word, 0) {
            return false;
        }
    }
    // A circumfix affix only counts once its partner is present, and
    // a non-circumfix one only while no partner is expected.
    let at_cfix = condit & CONDIT_CFIX == 0;
    let is_plain = condit & CONDIT_AFF == 0
        || ae
            .ae_flags
            .as_ref()
            .is_none_or(|flags| !flag_in_afflist(affile.af_flagtype, flags, affile.af_circumfix));
    at_cfix == is_plain
}

/// `xstrlcpy` into a word buffer: as much of `src` as fits, terminated.
fn copy_into(dst: &mut [u8; MAXWLEN], src: &[u8]) {
    let n = src.len().min(MAXWLEN - 1);
    dst[..n].copy_from_slice(&src[..n]);
    dst[n] = NUL as u8;
}

/// `xstrlcat` into a word buffer: as much of `src` as fits after what is
/// there, terminated.
fn append_into(dst: &mut [u8; MAXWLEN], src: &[u8]) {
    let len = cstr::until_nul(dst).len();
    if len >= MAXWLEN - 1 {
        return;
    }
    let n = src.len().min(MAXWLEN - 1 - len);
    dst[len..len + n].copy_from_slice(&src[..n]);
    dst[len + n] = NUL as u8;
}

/// Chop and add what one affix says, into `newword`.
fn build_affixed_word(newword: &mut [u8; MAXWLEN], word: &[u8], ae: &AffEntry, prefix: bool) {
    let chop_chars = ae.ae_chop.as_deref().map_or(0, char_count);
    if prefix {
        // A prefix: chop from the front, then put the addition before
        // what is left.
        copy_into(newword, ae.ae_add.as_deref().unwrap_or_default());
        let mut p = 0;
        for _ in 0..chop_chars {
            p += c_cluster_len(word.get(p..).unwrap_or_default());
        }
        append_into(newword, word.get(p..).unwrap_or_default());
        return;
    }

    // A suffix: chop from the end, then append.
    copy_into(newword, word);
    if ae.ae_chop.is_some() {
        let len = cstr::until_nul(newword).len();
        let mut p = len;
        for _ in 0..chop_chars {
            // Step back one whole character.
            if p == 0 {
                break;
            }
            p -= head_off(&newword[..len], p - 1) + 1;
        }
        newword[p] = NUL as u8;
    }
    if let Some(add) = &ae.ae_add {
        append_into(newword, add);
    }
}
