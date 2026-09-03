//! The optional sections of a `.spl` file.
//!
//! Each function here reads one section's payload and returns `0` or one of
//! the `SP_*` codes from [`read`](super::read). They are called from
//! [`read_section`](super::read) once the section id, flags and length have
//! been consumed.
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
use core::ffi::{c_char, c_int, c_uint};

use crate::fileio::{get2c, read_string};
use crate::hashtab::{hash_add_item, hash_hash, hash_lookup, hash_reset};
use crate::mbyte::{
    char_at, char_len, mb_cptr2char_adv, mb_ptr2char_adv, utf_char2bytes, utf_char2len,
};
use crate::memory::{xcalloc, xfree, xmalloc};
use crate::message::emsg;
use crate::os::cshim::{getc, gettext};
use crate::spell::{ascii_spell_chartab, byte_in_str, count_common_word};
use crate::strings::vim_strchr;
use crate::types::{
    FILE, NUL, RepItem, hash_T, int16_t, regprog_T, salfirst_T, salitem_T, size_t, slang_T, uint8_t,
};
use ::libc::ungetc;

use super::read::read_nonnul_bytes;
use super::{
    CF_UPPER, CF_WORD, COMPOUND_MAX_LEN, EOF, MAXREGIONS, MAXWLEN, SAL_COLLAPSE, SAL_F0LLOWUP,
    SAL_REM_ACCENTS, SP_FORMERROR, SP_OTHERERROR, SP_TRUNCERROR, e_duplicate_char_in_map_entry,
    set_spell_finish, vim_regcomp,
};
use crate::regexp::{RE_MAGIC, RE_STRICT, RE_STRING};

/// Read a length-prefixed string, with the length in `cnt_bytes` bytes.
///
/// `cntp` receives the length, or a negative `SP_*` on failure — the length
/// is how the caller both sizes the string and learns what went wrong, so a
/// null return with a non-negative count just means "empty".
///
/// # Safety
///
/// `fd` must be open and `cntp` writable.
pub(super) unsafe fn read_cnt_string(
    fd: *mut FILE,
    cnt_bytes: c_int,
    cntp: *mut c_int,
) -> *mut c_char {
    // SAFETY: the caller promises the file and the out-pointer.
    let mut cnt: c_int = 0;
    for _ in 0..cnt_bytes {
        let c = unsafe { getc(fd) };
        if c == EOF {
            unsafe { *cntp = SP_TRUNCERROR };
            return core::ptr::null_mut();
        }
        cnt = ((cnt as c_uint) << 8).wrapping_add(c as c_uint) as c_int;
    }
    unsafe { *cntp = cnt };
    if cnt == 0 {
        return core::ptr::null_mut();
    }
    let str = unsafe { read_string(fd, cnt as size_t) };
    if str.is_null() {
        unsafe { *cntp = SP_OTHERERROR };
    }
    str
}

/// `SN_REGION`: two letters per region, at most [`MAXREGIONS`] of them.
///
/// # Safety
///
/// `fd` must be positioned at the payload and `lp` be live.
pub(super) unsafe fn read_region_section(fd: *mut FILE, lp: *mut slang_T, len: c_int) -> c_int {
    // SAFETY: the length check below keeps the read inside `sl_regions`,
    // which holds MAXREGIONS * 2 letters plus a terminator.
    if len > MAXREGIONS as c_int * 2 {
        return SP_FORMERROR;
    }
    let buf = (unsafe { &raw mut (*lp).sl_regions }).cast::<c_char>();
    if let Err(e) = unsafe { read_nonnul_bytes(fd, buf, len as usize) } {
        return e;
    }
    unsafe { (*lp).sl_regions[len as usize] = NUL as c_char };
    0
}

/// `SN_CHARFLAGS`: a flags byte per high character, then their folded
/// forms. Either both parts are present or the section is malformed.
///
/// # Safety
///
/// `fd` must be positioned at the payload.
pub(super) unsafe fn read_charflags_section(fd: *mut FILE) -> c_int {
    // SAFETY: `fd` is open; both strings are owned here and freed here.
    let mut flagslen: c_int = 0;
    let flags = unsafe { read_cnt_string(fd, 1, &raw mut flagslen) };
    if flagslen < 0 {
        return flagslen;
    }
    let mut follen: c_int = 0;
    let fol = unsafe { read_cnt_string(fd, 2, &raw mut follen) };
    if follen < 0 {
        unsafe { xfree(flags.cast()) };
        return follen;
    }
    if !flags.is_null() && !fol.is_null() {
        unsafe { set_spell_charflags(flags, flagslen, fol) };
    }
    unsafe { xfree(flags.cast()) };
    unsafe { xfree(fol.cast()) };
    if flags.is_null() != fol.is_null() {
        return SP_FORMERROR;
    }
    0
}

/// `SN_PREFCOND`: one condition per prefix id, compiled to a regexp
/// anchored at the start of what precedes the prefix.
///
/// # Safety
///
/// `fd` must be positioned at the payload and `lp` be live.
pub(super) unsafe fn read_prefcond_section(fd: *mut FILE, lp: *mut slang_T) -> c_int {
    // SAFETY: `buf` is MAXWLEN + 1 and `n` is bounded below by MAXWLEN, so
    // the caret, the payload and the terminator all fit.
    let cnt = unsafe { get2c(fd) };
    if cnt <= 0 {
        return SP_FORMERROR;
    }
    unsafe {
        (*lp).sl_prefprog =
            xcalloc(cnt as size_t, size_of::<*mut regprog_T>()).cast::<*mut regprog_T>()
    };
    unsafe { (*lp).sl_prefixcnt = cnt };

    for i in 0..cnt {
        let n = unsafe { getc(fd) };
        if n < 0 || n >= MAXWLEN as c_int {
            return SP_FORMERROR;
        }
        if n == 0 {
            continue;
        }
        let mut buf: [c_char; MAXWLEN + 1] = [0; MAXWLEN + 1];
        buf[0] = b'^' as c_char;
        if let Err(e) = unsafe { read_nonnul_bytes(fd, buf.as_mut_ptr().add(1), n as usize) } {
            return e;
        }
        buf[(n + 1) as usize] = NUL as c_char;
        unsafe {
            *(*lp).sl_prefprog.offset(i as isize) =
                vim_regcomp(buf.as_mut_ptr(), RE_MAGIC | RE_STRING)
        };
    }
    0
}

/// `SN_REP` and `SN_REPSAL`: from/to pairs, plus an index of where the
/// entries for each leading byte start.
///
/// # Safety
///
/// `fd` must be positioned at the payload; `gap` must be an initialised
/// `fromto_T` array and `first` a 256-entry table.
pub(super) unsafe fn read_rep_section(
    fd: *mut FILE,
    out: &mut Vec<RepItem>,
    first: &mut [int16_t; 256],
) -> c_int {
    // SAFETY: the caller promises the stream; each string below is a
    // NUL-terminated answer from `read_cnt_string`, copied before it is
    // freed.
    let cnt = unsafe { get2c(fd) };
    if cnt < 0 {
        return SP_TRUNCERROR;
    }

    let mut items: Vec<RepItem> = Vec::with_capacity(cnt as usize);
    for _ in 0..cnt {
        let mut c: c_int = 0;
        let from = unsafe { read_cnt_string(fd, 1, &raw mut c) };
        if c < 0 {
            return c;
        }
        if c == 0 {
            return SP_FORMERROR;
        }
        let from = unsafe { owned_string(from) };

        let mut c: c_int = 0;
        let to = unsafe { read_cnt_string(fd, 1, &raw mut c) };
        if c <= 0 {
            return if c < 0 { c } else { SP_FORMERROR };
        }
        items.push(RepItem {
            from,
            to: unsafe { owned_string(to) },
        });
    }

    // Entries arrive sorted, so the first index per leading byte is
    // all the search needs.
    first.fill(-1);
    for (i, item) in items.iter().enumerate() {
        let lead = usize::from(item.from[0]);
        if first[lead] == -1 {
            first[lead] = i as int16_t;
        }
    }
    *out = items;
    0
}

/// Take a `read_cnt_string` answer over as owned bytes and free it.
///
/// # Safety
///
/// `p` must be a NUL-terminated string `read_cnt_string` allocated, or null.
unsafe fn owned_string(p: *mut c_char) -> Box<[u8]> {
    if p.is_null() {
        return Box::default();
    }
    // SAFETY: the caller promises the string.
    let bytes = unsafe { cstr::bytes_at(p) }.to_vec().into_boxed_slice();
    unsafe { xfree(p.cast()) };
    bytes
}

/// `SN_SAL`: the sound-folding rules.
///
/// Each rule is one blob holding a lead string, an optional `(...)` set of
/// alternatives for the character after it, and the rule characters — laid
/// out end to end in one allocation with NULs between, which is why the
/// parsing is a single pass with a moving write pointer.
///
/// # Safety
///
/// `fd` must be positioned at the payload and `slang` be live.
pub(super) unsafe fn read_sal_section(fd: *mut FILE, slang: *mut slang_T) -> c_int {
    // SAFETY: each entry's buffer is `ccnt + 2` bytes, and the writes below
    // add at most `ccnt` characters plus two terminators.
    unsafe { (*slang).sl_sofo = false };

    let flags = unsafe { getc(fd) };
    if flags & SAL_F0LLOWUP as c_int != 0 {
        unsafe { (*slang).sl_followup = true };
    }
    if flags & SAL_COLLAPSE as c_int != 0 {
        unsafe { (*slang).sl_collapse = true };
    }
    if flags & SAL_REM_ACCENTS as c_int != 0 {
        unsafe { (*slang).sl_rem_accents = true };
    }

    let cnt = unsafe { get2c(fd) };
    if cnt < 0 {
        return SP_TRUNCERROR;
    }
    let mut rules: Vec<salitem_T> = Vec::with_capacity(cnt as usize + 1);

    while (rules.len() as c_int) < cnt {
        let ccnt = unsafe { getc(fd) };
        if ccnt < 0 {
            return SP_TRUNCERROR;
        }
        // The whole item arrives as `ccnt` bytes: the lead, then an
        // optional "(abc)" set, then the flag characters.
        let mut item = Vec::with_capacity(ccnt as usize);
        for _ in 0..ccnt {
            let c = unsafe { getc(fd) };
            if c < 0 {
                return SP_TRUNCERROR;
            }
            item.push(c as u8);
        }

        // The lead is everything up to the first rule character.
        let lead_len = item
            .iter()
            .position(|b| b"0123456789(-<^$".contains(b))
            .unwrap_or(item.len());
        // A NUL inside the item ends whichever string it lands in, as it
        // did when the three were one C string.
        let lead = &item[..lead_len];
        let lead = &lead[..lead.iter().position(|&b| b == 0).unwrap_or(lead.len())];

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
            return SP_FORMERROR;
        }
        let mut sm_rules = rest.to_vec();
        sm_rules.push(NUL as u8);

        let mut ccnt = 0;
        // SAFETY: `fd` is positioned at the replacement's length byte.
        let to = unsafe { read_cnt_string(fd, 1, &raw mut ccnt) };
        if ccnt < 0 {
            return ccnt;
        }
        // SAFETY: `read_cnt_string` answers a NUL-terminated string or
        // null; the copy is taken before it is freed.
        let sm_to_w = if to.is_null() {
            None
        } else {
            let wide = unsafe { str2wide(to) };
            unsafe { xfree(to.cast()) };
            Some(wide)
        };

        let sm_lead_w = bytes2wide(lead);
        rules.push(salitem_T {
            sm_leadlen: sm_lead_w.len() as c_int - 1,
            sm_lead_w,
            sm_oneof_w: oneof.map(|set| {
                bytes2wide(&set[..set.iter().position(|&b| b == 0).unwrap_or(set.len())])
            }),
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
    // SAFETY: the caller's language.
    unsafe { (*slang).sl_sal = rules };

    unsafe { set_sal_first(slang) };
    0
}

/// `SN_WORDS`: NUL-separated common words.
///
/// # Safety
///
/// `fd` must be positioned at the payload and `lp` be live.
pub(super) unsafe fn read_words_section(fd: *mut FILE, lp: *mut slang_T, len: c_int) -> c_int {
    // SAFETY: `word` is MAXWLEN and the loop refuses to fill its last slot.
    let mut word: [uint8_t; MAXWLEN] = [0; MAXWLEN];
    let mut done = 0;
    while done < len {
        let mut i = 0;
        loop {
            let c = unsafe { getc(fd) };
            if c == EOF {
                return SP_TRUNCERROR;
            }
            word[i as usize] = c as uint8_t;
            if word[i as usize] as c_int == NUL {
                break;
            }
            if i == MAXWLEN as c_int - 1 {
                return SP_FORMERROR;
            }
            i += 1;
        }
        unsafe { count_common_word(lp, word.as_mut_ptr().cast::<c_char>(), -1, 10) };
        done += i + 1;
    }
    0
}

/// `SN_SOFO`: a from/to character mapping used instead of `SAL` rules.
///
/// # Safety
///
/// `fd` must be positioned at the payload and `slang` be live.
pub(super) unsafe fn read_sofo_section(fd: *mut FILE, slang: *mut slang_T) -> c_int {
    // SAFETY: both strings are owned here and freed here.
    unsafe { (*slang).sl_sofo = true };

    let mut cnt: c_int = 0;
    let from = unsafe { read_cnt_string(fd, 2, &raw mut cnt) };
    if cnt < 0 {
        return cnt;
    }
    let to = unsafe { read_cnt_string(fd, 2, &raw mut cnt) };
    if cnt < 0 {
        unsafe { xfree(from.cast()) };
        return cnt;
    }

    // Both or neither; one alone cannot be a mapping.
    let res = if !from.is_null() && !to.is_null() {
        unsafe { set_sofo(slang, from, to) }
    } else if !from.is_null() || !to.is_null() {
        SP_FORMERROR
    } else {
        0
    };
    unsafe { xfree(from.cast()) };
    unsafe { xfree(to.cast()) };
    res
}

/// `SN_COMPOUND`: the limits on joining words, the flags that say which
/// words may join, and the pattern that checks a candidate compound.
///
/// The flag string is turned into a regexp as it is read: each flag becomes
/// a branch, `/` separates the parts a compound may be built from.
///
/// # Safety
///
/// `fd` must be positioned at the payload and `slang` be live.
pub(super) unsafe fn read_compound(fd: *mut FILE, slang: *mut slang_T, len: c_int) -> c_int {
    // SAFETY: `pat` is sized from `todo` for the worst case — two bytes per
    // flag for the escaped forms, plus the fixed wrapper — and the flag
    // buffers are `todo + 1`, which is one per flag plus a terminator.
    let mut todo = len;
    if todo < 2 {
        return SP_FORMERROR;
    }

    todo -= 1;
    let mut c = unsafe { getc(fd) };
    unsafe { (*slang).sl_compmax = if c < 2 { MAXWLEN as c_int } else { c } };
    todo -= 1;
    c = unsafe { getc(fd) };
    unsafe { (*slang).sl_compminlen = if c < 1 { 0 } else { c } };
    todo -= 1;
    c = unsafe { getc(fd) };
    unsafe { (*slang).sl_compsylmax = if c < 1 { MAXWLEN as c_int } else { c } };

    // A zero here marks the newer layout, which adds the options byte
    // and the CHECKCOMPOUNDPATTERN list; anything else is a flag of
    // the old layout and gets pushed back.
    c = unsafe { getc(fd) };
    if c != 0 {
        unsafe { ungetc(c, fd) };
    } else {
        todo -= 1;
        unsafe { (*slang).sl_compoptions = getc(fd) };
        todo -= 1;

        let cnt = unsafe { get2c(fd) };
        if cnt < 0 {
            return SP_TRUNCERROR;
        }
        todo -= 2;
        let mut pats: Vec<Box<[u8]>> = Vec::with_capacity(cnt as usize);
        for _ in 0..cnt {
            let mut n: c_int = 0;
            // SAFETY: `fd` is positioned at the pattern's length byte;
            // the answer is a NUL-terminated string, or null for an
            // empty one, and is copied before it is freed.
            let p = unsafe { read_cnt_string(fd, 1, &raw mut n) };
            if n < 0 {
                return n;
            }
            pats.push(if p.is_null() {
                Box::default()
            } else {
                let bytes = unsafe { cstr::bytes_at(p) }.to_vec().into_boxed_slice();
                unsafe { xfree(p.cast()) };
                bytes
            });
            todo -= n + 1;
        }
        // SAFETY: the caller's language.
        unsafe { (*slang).sl_comppat = pats };
    }

    if todo < 0 {
        return SP_FORMERROR;
    }
    if todo as size_t > COMPOUND_MAX_LEN as size_t {
        return SP_FORMERROR;
    }

    // Worst case per flag: a backslash and up to four UTF-8 bytes.
    let patsize = (todo as size_t) * 2 + 7 + (todo as size_t) * 2;
    let flagsize = (todo as size_t) + 1;
    let pat = unsafe { xmalloc(patsize) }.cast::<c_char>();

    let mut cp = unsafe { xmalloc(flagsize) }.cast::<uint8_t>();
    unsafe { (*slang).sl_compstartflags = cp };
    unsafe { *cp = NUL as uint8_t };
    let mut ap = unsafe { xmalloc(flagsize) }.cast::<uint8_t>();
    unsafe { (*slang).sl_compallflags = ap };
    unsafe { *ap = NUL as uint8_t };
    let mut crp = unsafe { xmalloc(flagsize) }.cast::<uint8_t>();
    unsafe { (*slang).sl_comprules = crp };

    let mut pp = pat;
    for ch in *b"^\\(" {
        unsafe { *pp = ch as c_char };
        pp = unsafe { pp.add(1) };
    }

    // `atstart` is 1 while the next flag would begin a compound, and 2
    // inside a `[...]` set at that position.
    let mut atstart = 1;
    while todo > 0 {
        todo -= 1;
        c = unsafe { getc(fd) };
        if c == EOF {
            unsafe { xfree(pat.cast()) };
            return SP_TRUNCERROR;
        }

        // Collect the set of all flags, and the set that may start a
        // compound, skipping the regexp punctuation.
        if unsafe { vim_strchr(c"?*+[]/".as_ptr(), c) }.is_null()
            && !unsafe { byte_in_str((*slang).sl_compallflags, c) }
        {
            unsafe { *ap = c as uint8_t };
            ap = unsafe { ap.add(1) };
            unsafe { *ap = NUL as uint8_t };
        }
        if atstart != 0 {
            if c == b'[' as c_int {
                atstart = 2;
            } else if c == b']' as c_int {
                atstart = 0;
            } else {
                if !unsafe { byte_in_str((*slang).sl_compstartflags, c) } {
                    unsafe { *cp = c as uint8_t };
                    cp = unsafe { cp.add(1) };
                    unsafe { *cp = NUL as uint8_t };
                }
                if atstart == 1 {
                    atstart = 0;
                }
            }
        }

        // The rules string is only kept while the pattern stays a
        // plain sequence; any repetition makes it meaningless.
        if !crp.is_null() {
            if c == b'?' as c_int || c == b'+' as c_int || c == b'*' as c_int {
                unsafe { xfree((*slang).sl_comprules.cast()) };
                unsafe { (*slang).sl_comprules = core::ptr::null_mut() };
                crp = core::ptr::null_mut();
            } else {
                unsafe { *crp = c as uint8_t };
                crp = unsafe { crp.add(1) };
            }
        }

        if c == b'/' as c_int {
            unsafe { *pp = b'\\' as c_char };
            unsafe { *pp.add(1) = b'|' as c_char };
            pp = unsafe { pp.add(2) };
            atstart = 1;
        } else {
            if c == b'?' as c_int || c == b'+' as c_int || c == b'~' as c_int {
                unsafe { *pp = b'\\' as c_char };
                pp = unsafe { pp.add(1) };
            }
            pp = unsafe { pp.offset(utf_char2bytes(c, pp) as isize) };
        }
    }

    for ch in *b"\\)$" {
        unsafe { *pp = ch as c_char };
        pp = unsafe { pp.add(1) };
    }
    unsafe { *pp = NUL as c_char };
    if !crp.is_null() {
        unsafe { *crp = NUL as uint8_t };
    }

    unsafe { (*slang).sl_compprog = vim_regcomp(pat, RE_MAGIC + RE_STRING + RE_STRICT) };
    unsafe { xfree(pat.cast()) };
    if unsafe { (*slang).sl_compprog }.is_null() {
        return SP_FORMERROR;
    }
    0
}

/// Turn a `SOFOFROM`/`SOFOTO` pair into the lookup the sound folder uses.
///
/// Characters below 256 map directly through `sl_sal_first`. Above that,
/// the low byte selects a list of from/to pairs, terminated by a zero.
///
/// # Safety
///
/// `from` and `to` must be NUL-terminated strings.
unsafe fn set_sofo(lp: *mut slang_T, from: *const c_char, to: *const c_char) -> c_int {
    // SAFETY: the caller promises the strings.
    let (from, to) = unsafe { (str2wide(from), str2wide(to)) };
    // The two strings must describe the same number of characters.
    if from.len() != to.len() {
        return SP_FORMERROR;
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

    // SAFETY: the caller's language.
    let first = unsafe { &mut (*lp).sl_sal_first };
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
    // SAFETY: as above.
    unsafe { (*lp).sl_sofo_map = core::mem::take(&mut map) };
    0
}

/// Index the `SAL` rules by the low byte of their first character, and
/// gather the rules that share one so the search can stop at the first
/// mismatch.
///
/// # Safety
///
/// `lp` must hold a filled `sl_sal`.
unsafe fn set_sal_first(lp: *mut slang_T) {
    // SAFETY: the caller's language.
    let sfirst = unsafe { &mut (*lp).sl_sal_first };
    sfirst.fill(-1);
    let rules = unsafe { &mut (*lp).sl_sal };

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

/// Copy a string into a NUL-terminated array of characters.
///
/// # Safety
///
/// `s` must be a NUL-terminated string.
unsafe fn str2wide(s: *const c_char) -> Box<[c_int]> {
    // SAFETY: the caller promises a NUL-terminated string.
    bytes2wide(unsafe { cstr::bytes_at(s) })
}

/// The same, for bytes that carry their own end.
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
/// `map` must be a NUL-terminated string.
pub(super) unsafe fn set_map_str(lp: *mut slang_T, map: *const c_char) {
    // SAFETY: the caller promises the string; each hash key is its own
    // allocation, owned by the table.
    if unsafe { *map } as c_int == NUL {
        unsafe { (*lp).sl_has_map = false };
        return;
    }
    unsafe { (*lp).sl_has_map = true };

    for i in 0..256 {
        unsafe { (*lp).sl_map_array[i] = 0 };
    }
    // SAFETY: the caller's language, whose map table `slang_alloc` set up.
    hash_reset(unsafe { &mut (*lp).sl_map_hash });

    // The first character of a group represents the whole group.
    let mut headc = 0;
    let mut p = map;
    while unsafe { *p } as c_int != NUL {
        let c = unsafe { mb_cptr2char_adv(&raw mut p) };
        if c == b'/' as c_int {
            headc = 0;
            continue;
        }
        if headc == 0 {
            headc = c;
        }
        if c < 256 {
            unsafe { (*lp).sl_map_array[c as usize] = headc };
            continue;
        }

        // Key: the character, a NUL, then its group's head.
        let cl = utf_char2len(c);
        let headcl = utf_char2len(headc);
        let b = unsafe { xmalloc((cl + headcl) as size_t + 2) }.cast::<c_char>();
        unsafe { utf_char2bytes(c, b) };
        unsafe { *b.offset(cl as isize) = NUL as c_char };
        unsafe { utf_char2bytes(headc, b.offset(cl as isize).add(1)) };
        unsafe { *b.offset((cl + 1 + headcl) as isize) = NUL as c_char };

        let hash: hash_T = unsafe { hash_hash(b) };
        let hi =
            unsafe { hash_lookup(&raw mut (*lp).sl_map_hash, b, cstr::bytes_at(b).len(), hash) };
        if !hi.is_kept() {
            unsafe { hash_add_item(&raw mut (*lp).sl_map_hash, hi, b, hash) };
        } else {
            emsg(gettext(e_duplicate_char_in_map_entry));
            unsafe { xfree(b.cast()) };
        }
    }
}

/// Build the word-character table from a `CHARFLAGS` section and install
/// it, if it agrees with what is already there.
///
/// # Safety
///
/// `flags_in` must hold `cnt` bytes and `fol` be a NUL-terminated string.
unsafe fn set_spell_charflags(flags_in: *const c_char, cnt: c_int, fol: *const c_char) {
    // SAFETY: the caller promises the buffers; the loop reads at most
    // `cnt` flag bytes and stops at `fol`'s terminator.
    let flags = flags_in.cast::<uint8_t>();
    let mut new_st = ascii_spell_chartab();

    // Only the high half is described; the low half is fixed.
    let mut p = fol;
    for i in 0..128 {
        if i < cnt {
            let f = unsafe { *flags.offset(i as isize) } as c_int;
            new_st.st_isw[(i + 128) as usize] = f & CF_WORD as c_int != 0;
            new_st.st_isu[(i + 128) as usize] = f & CF_UPPER as c_int != 0;
        }
        if unsafe { *p } as c_int != NUL {
            let c = unsafe { mb_ptr2char_adv(&raw mut p) };
            new_st.st_fold[(i + 128) as usize] = c as uint8_t;
            // Record the reverse mapping too, when it fits.
            if i + 128 != c && new_st.st_isu[(i + 128) as usize] && c < 256 {
                new_st.st_upper[c as usize] = (i + 128) as uint8_t;
            }
        }
    }
    let _ = set_spell_finish(&new_st);
}
