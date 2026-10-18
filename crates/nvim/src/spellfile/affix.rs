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

#![forbid(unsafe_code)]

use crate::message_fmt::msg_cstr;
use crate::smsg;
use core::cell::RefCell;
use core::ffi::{CStr, c_int};

use crate::mbyte::{c_cluster_len, char_at, head_off, mb_toupper};
use crate::spell::{onecap_copy, spelltab_upper};
use crate::strings::{any_non_ascii, find_char};

use super::aff::{AffState, item_number};
use super::flags::{aff_process_flags, affitem2flag, check_renumber};
use super::wordtree::{AFF_ENTRY_BYTES, AFF_HEADER_BYTES, tree_add_word};
use super::{
    AH_KEY_LEN, AffEntry, AffFile, AffHeader, AffixKind, MAXLINELEN, PFX_FLAGS, SpellInfo,
    WFP_COMPFORBID, WFP_COMPPERMIT, WFP_NC, WFP_UP, vim_regcomp,
};
use crate::regexp::{OwnedProg, RE_MAGIC, RE_STRICT, RE_STRING};

/// Compile an affix condition: `pattern` is `^%s` or `%s$` around it,
/// cut where upstream's `MAXLINELEN` buffer cut it.
fn compile_condition(before: &[u8], cond: &[u8], after: &[u8], flags: c_int) -> Option<OwnedProg> {
    let mut pattern = Vec::with_capacity(before.len() + cond.len() + after.len());
    pattern.extend_from_slice(before);
    pattern.extend_from_slice(cond);
    pattern.extend_from_slice(after);
    pattern.truncate(MAXLINELEN as usize - 1);
    crate::cstr::with_terminated(&pattern, |pattern| vim_regcomp(pattern, flags))
}

/// The header line of a `PFX`/`SFX` block. Returns false to stop reading.
pub(super) fn handle_affix_header(
    spin: &mut SpellInfo,
    aff: &mut AffFile,
    st: &mut AffState,
    items: &[&CStr],
    fname: &CStr,
    lnum: c_int,
) -> bool {
    let kind = if items[0].to_bytes().starts_with(b"P") {
        AffixKind::Prefix
    } else {
        AffixKind::Suffix
    };
    let name = items[1].to_bytes();
    // The table keys on the name cut to the key's room.
    let key = &name[..name.len().min(AH_KEY_LEN as usize - 1)];
    let combines = items[2].to_bytes().starts_with(b"Y");

    let at = match aff.affixes(kind).find(key) {
        Some(at) => {
            // A continued block for an affix already defined.
            let ah = aff.affixes(kind).get(at);
            if (ah.ah_combine != 0) != combines {
                let (file, item) = (msg_cstr(fname), msg_cstr(items[1]));
                smsg!(
                    0,
                    "Different combining flag in continued affix block in {file} line {lnum}: {item}"
                );
            }
            if ah.ah_follows == 0 {
                let (fname, arg2) = (msg_cstr(fname), msg_cstr(items[1]));
                smsg!(0, "Duplicate affix in {fname} line {}: {arg2}", lnum);
            }
            at
        }
        None => {
            spin.si_arena.charge(AFF_HEADER_BYTES, true);
            let flag = affitem2flag(aff.af_flagtype, name, fname, lnum);
            // An unusable name is fatal: the key would not fit, or the
            // flag could not be read.
            if flag == 0 || name.len() >= AH_KEY_LEN as usize {
                st.cur_aff = None;
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
            if clashes.contains(&flag) {
                let (fname, arg2) = (msg_cstr(fname), msg_cstr(items[1]));
                smsg!(
                    0,
                    "Affix also used for BAD/RARE/KEEPCASE/NEEDAFFIX/NEEDCOMPOUND/NOSUGGEST in {fname} line {}: {arg2}",
                    lnum
                );
            }
            aff.affixes_mut(kind).insert(AffHeader {
                ah_key: name.to_vec(),
                ah_flag: flag,
                ah_new_id: 0,
                ah_combine: c_int::from(combines),
                ah_follows: 0,
                ah_entries: Vec::new(),
            })
        }
    };
    st.cur_aff = Some((kind, at));

    // An "S" after the count says another block for this affix follows.
    let mut lasti = 4;
    let follows = items.len() > lasti && items[lasti] == c"S";
    if follows {
        lasti += 1;
    }
    aff.affixes_mut(kind).get_mut(at).ah_follows = c_int::from(follows);
    if items.len() > lasti && !aff.af_ignoreextra && !items[lasti].to_bytes().starts_with(b"#") {
        let (file, item) = (msg_cstr(fname), msg_cstr(items[lasti]));
        smsg!(0, "Trailing text in {file} line {lnum}: {item}");
    }
    if items[2] != c"Y" && items[2] != c"N" {
        let (fname, arg2) = (msg_cstr(fname), msg_cstr(items[2]));
        smsg!(0, "Expected Y or N in {fname} line {}: {arg2}", lnum);
    }

    if kind == AffixKind::Prefix && aff.af_pfxpostpone != 0 {
        if aff.af_pref.get(at).ah_new_id == 0 {
            check_renumber(spin);
            spin.si_newpref_id += 1;
            aff.af_pref.get_mut(at).ah_new_id = spin.si_newpref_id;
            // Nothing has used the id yet; it is given back at the end
            // of the block if nothing does.
            st.did_postpone_prefix = false;
        } else {
            st.did_postpone_prefix = true;
        }
    }
    st.aff_todo = item_number(items[3]);
    true
}

/// One entry of a `PFX`/`SFX` block.
pub(super) fn handle_affix_entry(
    spin: &mut SpellInfo,
    aff: &mut AffFile,
    st: &mut AffState,
    items: &[&CStr],
    fname: &CStr,
    lnum: c_int,
) {
    let lasti = 5;
    // A lone "-" is Hunspell's morphological field separator.
    if items.len() > lasti
        && !items[lasti].to_bytes().starts_with(b"#")
        && (items[lasti] != c"-" || items.len() != lasti + 1)
    {
        let (file, item) = (msg_cstr(fname), msg_cstr(items[lasti]));
        smsg!(0, "Trailing text in {file} line {lnum}: {item}");
    }
    st.aff_todo -= 1;
    let (kind, at) = st.cur_aff.expect("an entry follows its block's header");

    spin.si_arena.charge(AFF_ENTRY_BYTES, true);
    let mut entry = AffEntry {
        ae_chop: None,
        ae_add: None,
        ae_flags: None,
        ae_cond: None,
        ae_prog: RefCell::new(None),
        ae_compforbid: false,
        ae_comppermit: false,
    };
    if items[2] != c"0" {
        let chop = items[2].to_bytes();
        spin.si_arena.charge_str(chop);
        entry.ae_chop = Some(chop.to_vec());
    }
    if items[3] != c"0" {
        let mut add = items[3].to_bytes().to_vec();
        spin.si_arena.charge_str(&add);
        // Flags the added form itself carries follow a "/".
        if let Some(slash) = find_char(&add, c_int::from(b'/')) {
            entry.ae_flags = Some(add[slash + 1..].to_vec());
            add.truncate(slash);
            entry.ae_add = Some(add);
            aff_process_flags(aff, &mut entry);
        } else {
            entry.ae_add = Some(add);
        }
    }

    // With 'ascii' set, an affix that needs more than ASCII is dropped.
    if spin.si_ascii != 0
        && (any_non_ascii(entry.ae_chop.as_deref().unwrap_or_default())
            || any_non_ascii(entry.ae_add.as_deref().unwrap_or_default()))
    {
        return;
    }

    let is_prefix = kind == AffixKind::Prefix;
    if items[4] != c"." {
        let cond = items[4].to_bytes();
        spin.si_arena.charge_str(cond);
        entry.ae_cond = Some(cond.to_vec());
        // A prefix condition anchors at the start, a suffix at the end.
        let (before, after): (&[u8], &[u8]) = if is_prefix { (b"^", b"") } else { (b"", b"$") };
        let prog = compile_condition(before, cond, after, RE_MAGIC + RE_STRING + RE_STRICT);
        if prog.is_none() {
            let (fname, arg2) = (msg_cstr(fname), msg_cstr(items[4]));
            smsg!(0, "Broken condition in {fname} line {}: {arg2}", lnum);
        }
        entry.ae_prog = RefCell::new(prog);
    }

    if is_prefix && aff.af_pfxpostpone != 0 && entry.ae_flags.is_none() {
        postpone_prefix(spin, aff, st, &mut entry, items);
    }
    aff.affixes_mut(kind).get_mut(at).ah_entries.push(entry);
}

/// File a prefix in the prefix tree instead of expanding it into words.
pub(super) fn postpone_prefix(
    spin: &mut SpellInfo,
    aff: &mut AffFile,
    st: &mut AffState,
    entry: &mut AffEntry,
    items: &[&CStr],
) {
    // A prefix that chops one letter and adds the same letter upper
    // cased is really a capitalisation rule; record it as one so the
    // checker can apply it without a chop.
    let mut upper = false;
    if let (Some(chop), Some(add)) = (&entry.ae_chop, &mut entry.ae_add)
        && c_cluster_len(chop) == chop.len()
    {
        let c = char_at(chop);
        let c_up = if c >= 128 {
            mb_toupper(c)
        } else {
            c_int::from(spelltab_upper(c as usize))
        };
        if c_up != c
            && entry
                .ae_cond
                .as_deref()
                .is_none_or(|cond| char_at(cond) == c)
        {
            // Step back to the last character of what is added.
            let last = add
                .len()
                .checked_sub(1)
                .map_or(0, |end| end - head_off(add, end));
            if char_at(&add[last..]) == c_up {
                upper = true;
                add.truncate(last);
                entry.ae_chop = None;
                if entry.ae_cond.is_some() {
                    // The condition has to match the capitalised form.
                    let mut buf = [0u8; MAXLINELEN as usize];
                    onecap_copy(items[4].to_bytes(), &mut buf, true);
                    let cond = crate::cstr::until_nul(&buf).to_vec();
                    spin.si_arena.charge_str(&cond);
                    *entry.ae_prog.get_mut() =
                        compile_condition(b"^", &cond, b"", RE_MAGIC + RE_STRING);
                    entry.ae_cond = Some(cond);
                }
            }
        }
    }

    // Only a prefix with nothing to chop can be applied at match time.
    if entry.ae_chop.is_none() {
        file_postponed_prefix(spin, aff, st, entry, upper);
    }

    // Nothing in the block was postponed after all; give the id back.
    if st.aff_todo == 0 && !st.did_postpone_prefix {
        spin.si_newpref_id -= 1;
        let (kind, at) = st.cur_aff.expect("an entry follows its block's header");
        aff.affixes_mut(kind).get_mut(at).ah_new_id = 0;
    }
}

/// Put one postponed prefix into the prefix tree.
pub(super) fn file_postponed_prefix(
    spin: &mut SpellInfo,
    aff: &AffFile,
    st: &mut AffState,
    entry: &AffEntry,
    upper: bool,
) {
    // Conditions are shared: the tree stores an index into si_prefcond.
    let want: Option<&[u8]> = entry.ae_cond.as_deref();
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

    let added = entry.ae_add.as_deref().unwrap_or_default();
    let (kind, at) = st.cur_aff.expect("an entry follows its block's header");
    let ah = aff.affixes(kind).get(at);
    let mut n = PFX_FLAGS;
    if ah.ah_combine == 0 {
        n |= WFP_NC as c_int;
    }
    if upper {
        n |= WFP_UP as c_int;
    }
    if entry.ae_comppermit {
        n |= WFP_COMPPERMIT as c_int;
    }
    if entry.ae_compforbid {
        n |= WFP_COMPFORBID as c_int;
    }
    let prefroot = spin.si_prefroot.expect("mkspell allocates the roots first");
    let new_id = ah.ah_new_id;
    tree_add_word(spin, added, prefroot, n, idx, new_id);
    st.did_postpone_prefix = true;
}
