//! The optional sections of a `.spl` file.
//!
//! Each function here reads one section's payload from an [`Spl`] and
//! answers `Ok(())` or one of the three [`SpellReadError`]s. They are called
//! from [`read_section`](super::read) once the section id, flags and length
//! have been consumed.
//!
//! All of them are reading attacker-controllable bytes, so counts are
//! bounded before they are used to allocate or index, and text that will be
//! treated as a C string is rejected if it contains a NUL.
//!
//! # What the sections carry
//!
//! - `REGION` — the two-letter names of up to [`MAXREGIONS`] regions.
//! - `CHARFLAGS` — which high bytes are word characters and how they fold.
//! - `PREFCOND` — one regexp per prefix id, matched against what precedes.
//! - `REP` / `REPSAL` — from/to pairs for suggesting a replacement.
//! - `SAL` — the sound-folding rules, or `SOFO` as a simpler alternative.
//! - `WORDS` — words common enough to score better as suggestions.
//! - `MAP` — characters that count as near-equivalent when scoring.
//! - `COMPOUND` — the rules for joining words together.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use core::ffi::{c_char, c_int};

use crate::hashtab::{hash_add_item, hash_hash, hash_lookup, hash_reset};
use crate::mbyte::{char_at, char_len, encode_char, mb_ptr2char_adv, utf_char2len};
use crate::memory::handoff::owned_cstr;
use crate::memory::xfree;
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::spell::{ascii_spell_chartab, count_common_word};
use crate::strings::vim_strchr;
use crate::types::{
    NUL, RepItem, hash_T, int16_t, regprog_T, salfirst_T, salitem_T, slang_T, uint8_t,
};

use super::spl::{SpellReadError, Spl, SplResult, trim_nul};
use super::{
    CF_UPPER, CF_WORD, COMPOUND_MAX_LEN, MAXREGIONS, MAXWLEN, SAL_COLLAPSE, SAL_F0LLOWUP,
    SAL_REM_ACCENTS, e_duplicate_char_in_map_entry, set_spell_finish, vim_regcomp,
};
use crate::regexp::{RE_MAGIC, RE_STRICT, RE_STRING};

/// `SN_REGION`: two letters per region, at most [`MAXREGIONS`] of them.
pub(super) fn read_region_section(spl: &mut Spl, lp: &mut slang_T, len: c_int) -> SplResult<()> {
    // `sl_regions` holds MAXREGIONS * 2 letters plus a terminator.
    if len > MAXREGIONS as c_int * 2 {
        return Err(SpellReadError::Format);
    }
    let len = len as usize;
    let bytes = spl.read_nonnul_bytes(len)?;
    for (slot, &b) in lp.sl_regions.iter_mut().zip(bytes.iter()) {
        *slot = b.cast_signed();
    }
    lp.sl_regions[len] = NUL as c_char;
    Ok(())
}

/// `SN_CHARFLAGS`: a flags byte per high character, then their folded
/// forms. Either both parts are present or the section is malformed.
pub(super) fn read_charflags_section(spl: &mut Spl) -> SplResult<()> {
    let flags = spl.read_cnt_string(1)?;
    let fol = spl.read_cnt_string(2)?;
    if !flags.is_empty() && !fol.is_empty() {
        set_spell_charflags(&flags, &fol);
    }
    if flags.is_empty() != fol.is_empty() {
        return Err(SpellReadError::Format);
    }
    Ok(())
}

/// `SN_PREFCOND`: one condition per prefix id, compiled to a regexp
/// anchored at the start of what precedes the prefix.
///
/// # Safety
///
/// `lp` must be a language whose `sl_prefprog` is free to be replaced.
pub(super) unsafe fn read_prefcond_section(spl: &mut Spl, lp: &mut slang_T) -> SplResult<()> {
    // Both counts below take the end of the file as `-1` and let the range
    // test reject it: a truncated `SN_PREFCOND` is a *format* error, which
    // `test_spellfile.vim` pins.
    let cnt = spl.get2c().unwrap_or(-1);
    if cnt <= 0 {
        return Err(SpellReadError::Format);
    }
    let cnt = cnt as usize;
    let mut progs: Vec<*mut regprog_T> = vec![core::ptr::null_mut(); cnt];

    for slot in &mut progs {
        let n = spl.getc().map_or(-1, c_int::from);
        if n < 0 || n >= MAXWLEN as c_int {
            return Err(SpellReadError::Format);
        }
        if n == 0 {
            continue;
        }
        let n = n as usize;
        // The condition matches what precedes the prefix, from its start.
        let mut pat = Vec::with_capacity(n + 2);
        pat.push(b'^');
        pat.extend_from_slice(&spl.read_nonnul_bytes(n)?);
        pat.push(NUL as u8);
        // SAFETY: `pat` is NUL-terminated and outlives the call.
        *slot = unsafe { vim_regcomp(pat.as_mut_ptr().cast::<c_char>(), RE_MAGIC | RE_STRING) };
    }

    lp.sl_prefixcnt = cnt as c_int;
    lp.sl_prefprog = Box::into_raw(progs.into_boxed_slice()).cast::<*mut regprog_T>();
    Ok(())
}

/// `SN_REP` and `SN_REPSAL`: from/to pairs, plus an index of where the
/// entries for each leading byte start.
pub(super) fn read_rep_section(
    spl: &mut Spl,
    out: &mut Vec<RepItem>,
    first: &mut [int16_t; 256],
) -> SplResult<()> {
    let cnt = spl.get2c()?;

    let mut items: Vec<RepItem> = Vec::with_capacity(cnt as usize);
    for _ in 0..cnt {
        let from = spl.read_cnt_string(1)?;
        if from.is_empty() {
            return Err(SpellReadError::Format);
        }
        let to = spl.read_cnt_string(1)?;
        if to.is_empty() {
            return Err(SpellReadError::Format);
        }
        items.push(RepItem {
            from: trim_nul(&from).into(),
            to: trim_nul(&to).into(),
        });
    }

    // Entries arrive sorted, so the first index per leading byte is
    // all the search needs. A `from` that begins with a NUL is empty
    // once trimmed and indexes slot zero, which is where the C string
    // it used to be put it too.
    first.fill(-1);
    for (i, item) in items.iter().enumerate() {
        let lead = usize::from(item.from.first().copied().unwrap_or(0));
        if first[lead] == -1 {
            first[lead] = i as int16_t;
        }
    }
    *out = items;
    Ok(())
}

/// `SN_SAL`: the sound-folding rules.
///
/// Each rule arrives as one blob holding a lead string, an optional `(...)`
/// set of alternatives for the character after it, and the rule characters,
/// laid out end to end — which is why the parsing is one pass that cuts the
/// blob into three.
pub(super) fn read_sal_section(spl: &mut Spl, slang: &mut slang_T) -> SplResult<()> {
    slang.sl_sofo = false;

    // A missing flags byte reads as -1, which sets all three.
    let flags = spl.getc().map_or(-1, c_int::from);
    if flags & SAL_F0LLOWUP as c_int != 0 {
        slang.sl_followup = true;
    }
    if flags & SAL_COLLAPSE as c_int != 0 {
        slang.sl_collapse = true;
    }
    if flags & SAL_REM_ACCENTS as c_int != 0 {
        slang.sl_rem_accents = true;
    }

    let cnt = spl.get2c()?;
    let mut rules: Vec<salitem_T> = Vec::with_capacity(cnt as usize + 1);

    while (rules.len() as c_int) < cnt {
        // The whole item arrives as `ccnt` bytes: the lead, then an
        // optional "(abc)" set, then the flag characters.
        let ccnt = usize::from(spl.byte()?);
        let item = spl.read_bytes(ccnt)?;

        // The lead is everything up to the first rule character.
        let lead_len = item
            .iter()
            .position(|b| b"0123456789(-<^$".contains(b))
            .unwrap_or(item.len());
        // A NUL inside the item ends whichever string it lands in, as it
        // did when the three were one C string.
        let lead = trim_nul(&item[..lead_len]);

        // An optional "(abc)" set of characters any of which may follow
        // the lead; whatever is left after it is the rule.
        let (oneof, rest) = if item.get(lead_len) == Some(&b'(') {
            let close = item[lead_len + 1..]
                .iter()
                .position(|&b| b == b')')
                .map(|at| lead_len + 1 + at);
            match close {
                Some(at) => (
                    Some(&item[lead_len + 1..at]),
                    &item[(at + 1).min(item.len())..],
                ),
                None => (Some(&item[lead_len + 1..]), &item[item.len()..]),
            }
        } else {
            (None, &item[lead_len..])
        };

        // The tail past the first flag character is where the file may
        // not put a NUL: the flags are a string.
        if rest.len() > 1 && rest[1..].contains(&0) {
            return Err(SpellReadError::Format);
        }
        let mut sm_rules = rest.to_vec();
        sm_rules.push(NUL as u8);

        let to = spl.read_cnt_string(1)?;
        let sm_to_w = if to.is_empty() {
            None
        } else {
            Some(bytes2wide(trim_nul(&to)))
        };

        let sm_lead_w = bytes2wide(lead);
        rules.push(salitem_T {
            sm_leadlen: sm_lead_w.len() as c_int - 1,
            sm_lead_w,
            sm_oneof_w: oneof.map(|set| bytes2wide(trim_nul(set))),
            sm_rules: sm_rules.into_boxed_slice(),
            sm_to_w,
        });
    }

    if !rules.is_empty() {
        // A final empty rule, so the search always has one to stop on.
        rules.push(salitem_T {
            sm_lead_w: Box::new([NUL]),
            sm_leadlen: 0,
            sm_oneof_w: None,
            sm_rules: Box::new([NUL as u8]),
            sm_to_w: None,
        });
    }
    slang.sl_sal = rules;

    set_sal_first(slang);
    Ok(())
}

/// `SN_WORDS`: NUL-separated common words.
///
/// # Safety
///
/// `lp` must be a language whose `sl_wordcount` table is initialised.
pub(super) unsafe fn read_words_section(
    spl: &mut Spl,
    lp: &mut slang_T,
    len: c_int,
) -> SplResult<()> {
    let mut done = 0;
    let mut word: Vec<u8> = Vec::with_capacity(MAXWLEN);
    while done < len {
        word.clear();
        loop {
            let c = spl.byte()?;
            if c == NUL as u8 {
                break;
            }
            // One slot is kept for the terminator the word is counted with.
            if word.len() == MAXWLEN - 1 {
                return Err(SpellReadError::Format);
            }
            word.push(c);
        }
        done += word.len() as c_int + 1;
        word.push(NUL as u8);
        // SAFETY: `word` is NUL-terminated and outlives the call.
        unsafe { count_common_word(lp, word.as_mut_ptr().cast::<c_char>(), -1, 10) };
    }
    Ok(())
}

/// `SN_SOFO`: a from/to character mapping used instead of `SAL` rules.
pub(super) fn read_sofo_section(spl: &mut Spl, slang: &mut slang_T) -> SplResult<()> {
    slang.sl_sofo = true;

    let from = spl.read_cnt_string(2)?;
    let to = spl.read_cnt_string(2)?;

    // Both or neither; one alone cannot be a mapping.
    if !from.is_empty() && !to.is_empty() {
        set_sofo(slang, trim_nul(&from), trim_nul(&to))
    } else if from.is_empty() != to.is_empty() {
        Err(SpellReadError::Format)
    } else {
        Ok(())
    }
}

/// Is `c` in `flags`, which is a NUL-terminated run of bytes?
///
/// The flag sets are built as they are read and asked about in the same
/// loop, so this answers over the part built so far — and stops at a NUL
/// the file put there, exactly as the C string search it replaces did.
fn byte_in_flags(flags: &[u8], c: u8) -> bool {
    if c == NUL as u8 {
        return false;
    }
    for &b in flags {
        if b == NUL as u8 {
            return false;
        }
        if b == c {
            return true;
        }
    }
    false
}

/// `SN_COMPOUND`: the limits on joining words, the flags that say which
/// words may join, and the pattern that checks a candidate compound.
///
/// The flag string is turned into a regexp as it is read: each flag becomes
/// a branch, `/` separates the parts a compound may be built from.
///
/// # Safety
///
/// `slang` must be a language whose compound fields are free to be replaced.
pub(super) unsafe fn read_compound(
    spl: &mut Spl,
    slang: &mut slang_T,
    len: c_int,
) -> SplResult<()> {
    let mut todo = len;
    if todo < 2 {
        return Err(SpellReadError::Format);
    }

    todo -= 1;
    let mut c = spl.getc().map_or(-1, c_int::from);
    slang.sl_compmax = if c < 2 { MAXWLEN as c_int } else { c };
    todo -= 1;
    c = spl.getc().map_or(-1, c_int::from);
    slang.sl_compminlen = if c < 1 { 0 } else { c };
    todo -= 1;
    c = spl.getc().map_or(-1, c_int::from);
    slang.sl_compsylmax = if c < 1 { MAXWLEN as c_int } else { c };

    // A zero here marks the newer layout, which adds the options byte
    // and the CHECKCOMPOUNDPATTERN list; anything else is a flag of
    // the old layout and gets pushed back.
    match spl.getc() {
        Some(0) => {
            todo -= 1;
            slang.sl_compoptions = spl.getc().map_or(-1, c_int::from);
            todo -= 1;

            let cnt = spl.get2c()?;
            todo -= 2;
            let mut pats: Vec<Box<[u8]>> = Vec::with_capacity(cnt as usize);
            for _ in 0..cnt {
                let p = spl.read_cnt_string(1)?;
                todo -= p.len() as c_int + 1;
                pats.push(trim_nul(&p).into());
            }
            slang.sl_comppat = pats;
        }
        Some(b) => spl.unget(b),
        // The end of the file leaves the old layout in force, as an
        // `ungetc(EOF)` did; the flag loop below then finds nothing.
        None => {}
    }

    if todo < 0 {
        return Err(SpellReadError::Format);
    }
    if todo as usize > COMPOUND_MAX_LEN as usize {
        return Err(SpellReadError::Format);
    }

    // The pattern is `^\( … \)$` with every flag as a branch.
    let mut pat: Vec<u8> = Vec::with_capacity(todo as usize * 4 + 8);
    pat.extend_from_slice(b"^\\(");
    // Every flag, and the ones that may start a compound.
    let mut all: Vec<u8> = Vec::new();
    let mut start: Vec<u8> = Vec::new();
    // The flags in order, kept only while the pattern stays a plain
    // sequence; any repetition makes it meaningless.
    let mut rules: Option<Vec<u8>> = Some(Vec::new());

    // `atstart` is 1 while the next flag would begin a compound, and 2
    // inside a `[...]` set at that position.
    let mut atstart = 1;
    while todo > 0 {
        todo -= 1;
        let Some(b) = spl.getc() else {
            return Err(SpellReadError::Trunc);
        };
        let c = c_int::from(b);

        // Collect the set of all flags, and the set that may start a
        // compound, skipping the regexp punctuation.
        // SAFETY: a literal, NUL-terminated string.
        if unsafe { vim_strchr(c"?*+[]/".as_ptr(), c) }.is_null() && !byte_in_flags(&all, b) {
            all.push(b);
        }
        if atstart != 0 {
            if b == b'[' {
                atstart = 2;
            } else if b == b']' {
                atstart = 0;
            } else {
                if !byte_in_flags(&start, b) {
                    start.push(b);
                }
                if atstart == 1 {
                    atstart = 0;
                }
            }
        }

        if let Some(kept) = rules.as_mut() {
            if b == b'?' || b == b'+' || b == b'*' {
                rules = None;
            } else {
                kept.push(b);
            }
        }

        if b == b'/' {
            pat.extend_from_slice(b"\\|");
            atstart = 1;
        } else {
            if b == b'?' || b == b'+' || b == b'~' {
                pat.push(b'\\');
            }
            let at = pat.len();
            pat.resize(at + utf_char2len(c) as usize, 0);
            encode_char(c, &mut pat[at..]);
        }
    }

    pat.extend_from_slice(b"\\)$");
    pat.push(NUL as u8);

    slang.sl_compallflags = owned_cstr(all).cast::<uint8_t>();
    slang.sl_compstartflags = owned_cstr(start).cast::<uint8_t>();
    // SAFETY: whatever a previous section left there is this language's.
    unsafe { xfree(slang.sl_comprules.cast()) };
    slang.sl_comprules = match rules {
        Some(kept) => owned_cstr(kept).cast::<uint8_t>(),
        None => core::ptr::null_mut(),
    };

    // SAFETY: `pat` is NUL-terminated and outlives the call.
    slang.sl_compprog = unsafe {
        vim_regcomp(
            pat.as_mut_ptr().cast::<c_char>(),
            RE_MAGIC + RE_STRING + RE_STRICT,
        )
    };
    if slang.sl_compprog.is_null() {
        return Err(SpellReadError::Format);
    }
    Ok(())
}

/// Turn a `SOFOFROM`/`SOFOTO` pair into the lookup the sound folder uses.
///
/// Characters below 256 map directly through `sl_sal_first`. Above that,
/// the low byte selects a list of from/to pairs, terminated by a zero.
fn set_sofo(lp: &mut slang_T, from: &[u8], to: &[u8]) -> SplResult<()> {
    let (from, to) = (bytes2wide(from), bytes2wide(to));
    // The two strings must describe the same number of characters.
    if from.len() != to.len() {
        return Err(SpellReadError::Format);
    }

    // Characters below 256 map straight through `sl_sal_first`. Wider ones
    // go in the list their low byte selects, as `from, to` pairs ending in
    // a zero.
    let mut sizes = [0usize; 256];
    for &c in from.iter().take(from.len() - 1) {
        if c >= 256 {
            sizes[(c & 0xff) as usize] += 1;
        }
    }
    let mut map: Vec<Box<[c_int]>> = sizes
        .iter()
        .map(|&n| {
            if n == 0 {
                Box::default()
            } else {
                vec![NUL; n * 2 + 1].into_boxed_slice()
            }
        })
        .collect();
    let mut filled = [0usize; 256];

    let first = &mut lp.sl_sal_first;
    first.fill(0);
    for (&c, &to_c) in from.iter().zip(to.iter()).take(from.len() - 1) {
        if c >= 256 {
            let low = (c & 0xff) as usize;
            let at = filled[low];
            map[low][at] = c;
            map[low][at + 1] = to_c;
            map[low][at + 2] = NUL;
            filled[low] += 2;
        } else {
            first[c as usize] = to_c as salfirst_T;
        }
    }
    lp.sl_sofo_map = map;
    Ok(())
}

/// Index the `SAL` rules by the low byte of their first character, and
/// gather the rules that share one so the search can stop at the first
/// mismatch.
fn set_sal_first(lp: &mut slang_T) {
    lp.sl_sal_first.fill(-1);
    let rules = &mut lp.sl_sal;
    let sfirst = &mut lp.sl_sal_first;

    let mut i = 0;
    while i < rules.len() {
        let c = (rules[i].sm_lead_w[0] & 0xff) as usize;
        if sfirst[c] == -1 {
            sfirst[c] = i as salfirst_T;
            // Skip the run that is already together.
            while i + 1 < rules.len() && (rules[i + 1].sm_lead_w[0] & 0xff) as usize == c {
                i += 1;
            }
            // Pull any later rule with the same low byte up to join it.
            let mut n = 1;
            while i + n < rules.len() {
                if (rules[i + n].sm_lead_w[0] & 0xff) as usize == c {
                    i += 1;
                    n -= 1;
                    rules[i..=i + n].rotate_right(1);
                }
                n += 1;
            }
        }
        i += 1;
    }
}

/// Copy bytes into a NUL-terminated array of characters.
fn bytes2wide(bytes: &[u8]) -> Box<[c_int]> {
    let mut out = Vec::with_capacity(bytes.len() + 1);
    let mut at = 0;
    while at < bytes.len() {
        out.push(char_at(&bytes[at..]));
        at += char_len(&bytes[at..]);
    }
    out.push(NUL);
    out.into_boxed_slice()
}

/// `SN_MAP`: `/`-separated groups whose members count as near-equivalent
/// when scoring a suggestion.
///
/// Members below 256 go in a direct table; above that, a hash table keyed
/// on the pair of characters answers "are these two in the same group".
///
/// # Safety
///
/// `lp`'s `sl_map_hash` must be an initialised table; every key added to it
/// is an allocation the table then owns.
pub(super) unsafe fn set_map_str(lp: &mut slang_T, map: &[u8]) {
    if map.is_empty() {
        lp.sl_has_map = false;
        return;
    }
    lp.sl_has_map = true;

    lp.sl_map_array.fill(0);
    hash_reset(&mut lp.sl_map_hash);

    // The first character of a group represents the whole group.
    let mut headc = 0;
    let mut at = 0;
    while at < map.len() {
        let c = char_at(&map[at..]);
        at += char_len(&map[at..]);
        if c == b'/' as c_int {
            headc = 0;
            continue;
        }
        if headc == 0 {
            headc = c;
        }
        if c < 256 {
            lp.sl_map_array[c as usize] = headc;
            continue;
        }

        // Key: the character, a NUL, then its group's head.
        let mut key = vec![0u8; (utf_char2len(c) + utf_char2len(headc)) as usize + 1];
        let cl = encode_char(c, &mut key);
        encode_char(headc, &mut key[cl + 1..]);
        let b = owned_cstr(key);

        // SAFETY: `b` is a NUL-terminated allocation the table takes over
        // when it is kept, and this frame frees when it is not.
        unsafe {
            let hash: hash_T = hash_hash(b);
            let hi = hash_lookup(&raw mut lp.sl_map_hash, b, cstr::bytes_at(b).len(), hash);
            if hi.is_kept() {
                emsg(gettext(e_duplicate_char_in_map_entry));
                xfree(b.cast());
            } else {
                hash_add_item(&raw mut lp.sl_map_hash, hi, b, hash);
            }
        }
    }
}

/// Build the word-character table from a `CHARFLAGS` section and install
/// it, if it agrees with what is already there.
fn set_spell_charflags(flags: &[u8], fol: &[u8]) {
    let mut new_st = ascii_spell_chartab();

    // `fol`'s walk is over composing characters as well as base ones, and
    // the codec's slice forms do not cover that pairing, so it stays a
    // pointer walk over a copy this frame owns and terminates.
    let mut folded = fol.to_vec();
    folded.push(NUL as u8);
    let mut p = folded.as_ptr().cast::<c_char>();

    // Only the high half is described; the low half is fixed.
    for i in 0..128usize {
        if let Some(&f) = flags.get(i) {
            new_st.st_isw[i + 128] = f & CF_WORD as u8 != 0;
            new_st.st_isu[i + 128] = f & CF_UPPER as u8 != 0;
        }
        // SAFETY: `folded` is NUL-terminated, and the walk stops there.
        if unsafe { *p } as c_int != NUL {
            // SAFETY: as above.
            let c = unsafe { mb_ptr2char_adv(&raw mut p) };
            new_st.st_fold[i + 128] = c as uint8_t;
            // Record the reverse mapping too, when it fits.
            if i as c_int + 128 != c && new_st.st_isu[i + 128] && c < 256 {
                new_st.st_upper[c as usize] = (i + 128) as uint8_t;
            }
        }
    }
    let _ = set_spell_finish(&new_st);
}
