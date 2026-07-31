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

use core::ffi::{c_char, c_int, c_uint};

use crate::src::nvim::fileio::{get2c, read_string};
use crate::src::nvim::garray::{ga_grow, ga_init};
use crate::src::nvim::hashtab::{hash_add_item, hash_hash, hash_init, hash_lookup, hash_removed};
use crate::src::nvim::mbyte::{
    mb_charlen, mb_cptr2char_adv, mb_ptr2char_adv, utf_char2bytes, utf_char2len, utf_ptr2len,
};
use crate::src::nvim::memory::{xcalloc, xfree, xmalloc};
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::libc::{getc, gettext, memset, strlen, ungetc};
use crate::src::nvim::spell::{byte_in_str, clear_spell_chartab, count_common_word};
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::types::{
    FILE, fromto_T, garray_T, hash_T, hashitem_T, int16_t, regprog_T, salfirst_T, salitem_T,
    size_t, slang_T, spelltab_T, uint8_t,
};

use super::read::read_nonnul_bytes;
use super::{
    CF_UPPER, CF_WORD, COMPOUND_MAX_LEN, EOF, MAXREGIONS, MAXWLEN, NUL, RE_MAGIC, RE_STRICT,
    RE_STRING, SAL_COLLAPSE, SAL_F0LLOWUP, SAL_REM_ACCENTS, SP_FORMERROR, SP_OTHERERROR,
    SP_TRUNCERROR, e_duplicate_char_in_map_entry, set_spell_finish, vim_regcomp,
};

/// Read a length-prefixed string, with the length in `cnt_bytes` bytes.
///
/// `cntp` receives the length, or a negative `SP_*` on failure — the length
/// is how the caller both sizes the string and learns what went wrong, so a
/// null return with a non-negative count just means "empty".
///
/// # Safety
///
/// `fd` must be open and `cntp` writable.
pub unsafe fn read_cnt_string(fd: *mut FILE, cnt_bytes: c_int, cntp: *mut c_int) -> *mut c_char {
    // SAFETY: the caller promises the file and the out-pointer.
    unsafe {
        let mut cnt: c_int = 0;
        for _ in 0..cnt_bytes {
            let c = getc(fd);
            if c == EOF {
                *cntp = SP_TRUNCERROR;
                return core::ptr::null_mut();
            }
            cnt = ((cnt as c_uint) << 8).wrapping_add(c as c_uint) as c_int;
        }
        *cntp = cnt;
        if cnt == 0 {
            return core::ptr::null_mut();
        }
        let str = read_string(fd, cnt as size_t);
        if str.is_null() {
            *cntp = SP_OTHERERROR;
        }
        str
    }
}

/// `SN_REGION`: two letters per region, at most [`MAXREGIONS`] of them.
///
/// # Safety
///
/// `fd` must be positioned at the payload and `lp` be live.
pub unsafe fn read_region_section(fd: *mut FILE, lp: *mut slang_T, len: c_int) -> c_int {
    // SAFETY: the length check below keeps the read inside `sl_regions`,
    // which holds MAXREGIONS * 2 letters plus a terminator.
    unsafe {
        if len > MAXREGIONS as c_int * 2 {
            return SP_FORMERROR;
        }
        let buf = (&raw mut (*lp).sl_regions).cast::<c_char>();
        if let Err(e) = read_nonnul_bytes(fd, buf, len as usize) {
            return e;
        }
        (*lp).sl_regions[len as usize] = NUL as c_char;
        0
    }
}

/// `SN_CHARFLAGS`: a flags byte per high character, then their folded
/// forms. Either both parts are present or the section is malformed.
///
/// # Safety
///
/// `fd` must be positioned at the payload.
pub unsafe fn read_charflags_section(fd: *mut FILE) -> c_int {
    // SAFETY: `fd` is open; both strings are owned here and freed here.
    unsafe {
        let mut flagslen: c_int = 0;
        let flags = read_cnt_string(fd, 1, &raw mut flagslen);
        if flagslen < 0 {
            return flagslen;
        }
        let mut follen: c_int = 0;
        let fol = read_cnt_string(fd, 2, &raw mut follen);
        if follen < 0 {
            xfree(flags.cast());
            return follen;
        }
        if !flags.is_null() && !fol.is_null() {
            set_spell_charflags(flags, flagslen, fol);
        }
        xfree(flags.cast());
        xfree(fol.cast());
        if flags.is_null() != fol.is_null() {
            return SP_FORMERROR;
        }
        0
    }
}

/// `SN_PREFCOND`: one condition per prefix id, compiled to a regexp
/// anchored at the start of what precedes the prefix.
///
/// # Safety
///
/// `fd` must be positioned at the payload and `lp` be live.
pub unsafe fn read_prefcond_section(fd: *mut FILE, lp: *mut slang_T) -> c_int {
    // SAFETY: `buf` is MAXWLEN + 1 and `n` is bounded below by MAXWLEN, so
    // the caret, the payload and the terminator all fit.
    unsafe {
        let cnt = get2c(fd);
        if cnt <= 0 {
            return SP_FORMERROR;
        }
        (*lp).sl_prefprog =
            xcalloc(cnt as size_t, core::mem::size_of::<*mut regprog_T>()).cast::<*mut regprog_T>();
        (*lp).sl_prefixcnt = cnt;

        for i in 0..cnt {
            let n = getc(fd);
            if n < 0 || n >= MAXWLEN as c_int {
                return SP_FORMERROR;
            }
            if n == 0 {
                continue;
            }
            let mut buf: [c_char; MAXWLEN + 1] = [0; MAXWLEN + 1];
            buf[0] = b'^' as c_char;
            if let Err(e) = read_nonnul_bytes(fd, buf.as_mut_ptr().add(1), n as usize) {
                return e;
            }
            buf[(n + 1) as usize] = NUL as c_char;
            *(*lp).sl_prefprog.offset(i as isize) =
                vim_regcomp(buf.as_mut_ptr(), RE_MAGIC | RE_STRING);
        }
        0
    }
}

/// `SN_REP` and `SN_REPSAL`: from/to pairs, plus an index of where the
/// entries for each leading byte start.
///
/// # Safety
///
/// `fd` must be positioned at the payload; `gap` must be an initialised
/// `fromto_T` array and `first` a 256-entry table.
pub unsafe fn read_rep_section(fd: *mut FILE, gap: *mut garray_T, first: *mut int16_t) -> c_int {
    // SAFETY: the caller promises the array and the table; `ga_grow` makes
    // room for `cnt` entries before any is written.
    unsafe {
        let cnt = get2c(fd);
        if cnt < 0 {
            return SP_TRUNCERROR;
        }
        ga_grow(gap, cnt);

        while (*gap).ga_len < cnt {
            let ftp = (*gap)
                .ga_data
                .cast::<fromto_T>()
                .offset((*gap).ga_len as isize);
            let mut c: c_int = 0;
            (*ftp).ft_from = read_cnt_string(fd, 1, &raw mut c);
            if c < 0 {
                return c;
            }
            if c == 0 {
                return SP_FORMERROR;
            }
            (*ftp).ft_to = read_cnt_string(fd, 1, &raw mut c);
            if c <= 0 {
                xfree((*ftp).ft_from.cast());
                return if c < 0 { c } else { SP_FORMERROR };
            }
            (*gap).ga_len += 1;
        }

        // Entries arrive sorted, so the first index per leading byte is
        // all the search needs.
        for i in 0..256 {
            *first.offset(i) = -1;
        }
        for i in 0..(*gap).ga_len {
            let ftp = (*gap).ga_data.cast::<fromto_T>().offset(i as isize);
            let lead = *(*ftp).ft_from as uint8_t as isize;
            if *first.offset(lead) == -1 {
                *first.offset(lead) = i as int16_t;
            }
        }
        0
    }
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
pub unsafe fn read_sal_section(fd: *mut FILE, slang: *mut slang_T) -> c_int {
    // SAFETY: each entry's buffer is `ccnt + 2` bytes, and the writes below
    // add at most `ccnt` characters plus two terminators.
    unsafe {
        (*slang).sl_sofo = false;

        let flags = getc(fd);
        if flags & SAL_F0LLOWUP as c_int != 0 {
            (*slang).sl_followup = true;
        }
        if flags & SAL_COLLAPSE as c_int != 0 {
            (*slang).sl_collapse = true;
        }
        if flags & SAL_REM_ACCENTS as c_int != 0 {
            (*slang).sl_rem_accents = true;
        }

        let cnt = get2c(fd);
        if cnt < 0 {
            return SP_TRUNCERROR;
        }
        let gap = &raw mut (*slang).sl_sal;
        ga_init(gap, core::mem::size_of::<salitem_T>() as c_int, 10);
        // One spare for the terminating entry appended below.
        ga_grow(gap, cnt + 1);

        while (*gap).ga_len < cnt {
            let smp = (*gap)
                .ga_data
                .cast::<salitem_T>()
                .offset((*gap).ga_len as isize);
            let mut ccnt = getc(fd);
            if ccnt < 0 {
                return SP_TRUNCERROR;
            }
            let mut p = xmalloc((ccnt as size_t) + 2).cast::<c_char>();
            (*smp).sm_lead = p;

            // The lead: everything up to the first rule character.
            let mut c = NUL;
            let mut i = 0;
            while i < ccnt {
                c = getc(fd);
                if !vim_strchr(c"0123456789(-<^$".as_ptr(), c).is_null() {
                    break;
                }
                *p = c as uint8_t as c_char;
                p = p.add(1);
                i += 1;
            }
            (*smp).sm_leadlen = p.offset_from((*smp).sm_lead) as c_int;
            *p = NUL as c_char;
            p = p.add(1);

            // An optional "(abc)" set of characters any of which may
            // follow the lead.
            if c == b'(' as c_int {
                (*smp).sm_oneof = p;
                i += 1;
                while i < ccnt {
                    c = getc(fd);
                    if c == b')' as c_int {
                        break;
                    }
                    *p = c as uint8_t as c_char;
                    p = p.add(1);
                    i += 1;
                }
                *p = NUL as c_char;
                p = p.add(1);
                i += 1;
                if i < ccnt {
                    c = getc(fd);
                }
            } else {
                (*smp).sm_oneof = core::ptr::null_mut();
            }

            // Whatever is left is the rule.
            (*smp).sm_rules = p;
            if i < ccnt {
                *p = c as uint8_t as c_char;
                p = p.add(1);
            }
            i += 1;
            if i < ccnt {
                if let Err(e) = read_nonnul_bytes(fd, p, (ccnt - i) as usize) {
                    xfree((*smp).sm_lead.cast());
                    return e;
                }
                p = p.offset((ccnt - i) as isize);
            }
            *p = NUL as c_char;

            (*smp).sm_to = read_cnt_string(fd, 1, &raw mut ccnt);
            if ccnt < 0 {
                xfree((*smp).sm_lead.cast());
                return ccnt;
            }

            // Wide copies, since sound folding works in characters.
            (*smp).sm_lead_w = mb_str2wide((*smp).sm_lead);
            (*smp).sm_leadlen = mb_charlen((*smp).sm_lead);
            (*smp).sm_oneof_w = if (*smp).sm_oneof.is_null() {
                core::ptr::null_mut()
            } else {
                mb_str2wide((*smp).sm_oneof)
            };
            (*smp).sm_to_w = if (*smp).sm_to.is_null() {
                core::ptr::null_mut()
            } else {
                mb_str2wide((*smp).sm_to)
            };
            (*gap).ga_len += 1;
        }

        if (*gap).ga_len > 0 {
            // A final empty rule, so the search always has one to stop on.
            let smp = (*gap)
                .ga_data
                .cast::<salitem_T>()
                .offset((*gap).ga_len as isize);
            let p = xmalloc(1).cast::<c_char>();
            *p = NUL as c_char;
            (*smp).sm_lead = p;
            (*smp).sm_lead_w = mb_str2wide(p);
            (*smp).sm_leadlen = 0;
            (*smp).sm_oneof = core::ptr::null_mut();
            (*smp).sm_oneof_w = core::ptr::null_mut();
            (*smp).sm_rules = p;
            (*smp).sm_to = core::ptr::null_mut();
            (*smp).sm_to_w = core::ptr::null_mut();
            (*gap).ga_len += 1;
        }

        set_sal_first(slang);
        0
    }
}

/// `SN_WORDS`: NUL-separated common words.
///
/// # Safety
///
/// `fd` must be positioned at the payload and `lp` be live.
pub unsafe fn read_words_section(fd: *mut FILE, lp: *mut slang_T, len: c_int) -> c_int {
    // SAFETY: `word` is MAXWLEN and the loop refuses to fill its last slot.
    unsafe {
        let mut word: [uint8_t; MAXWLEN] = [0; MAXWLEN];
        let mut done = 0;
        while done < len {
            let mut i = 0;
            loop {
                let c = getc(fd);
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
            count_common_word(lp, word.as_mut_ptr().cast::<c_char>(), -1, 10);
            done += i + 1;
        }
        0
    }
}

/// `SN_SOFO`: a from/to character mapping used instead of `SAL` rules.
///
/// # Safety
///
/// `fd` must be positioned at the payload and `slang` be live.
pub unsafe fn read_sofo_section(fd: *mut FILE, slang: *mut slang_T) -> c_int {
    // SAFETY: both strings are owned here and freed here.
    unsafe {
        (*slang).sl_sofo = true;

        let mut cnt: c_int = 0;
        let from = read_cnt_string(fd, 2, &raw mut cnt);
        if cnt < 0 {
            return cnt;
        }
        let to = read_cnt_string(fd, 2, &raw mut cnt);
        if cnt < 0 {
            xfree(from.cast());
            return cnt;
        }

        // Both or neither; one alone cannot be a mapping.
        let res = if !from.is_null() && !to.is_null() {
            set_sofo(slang, from, to)
        } else if !from.is_null() || !to.is_null() {
            SP_FORMERROR
        } else {
            0
        };
        xfree(from.cast());
        xfree(to.cast());
        res
    }
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
pub unsafe fn read_compound(fd: *mut FILE, slang: *mut slang_T, len: c_int) -> c_int {
    // SAFETY: `pat` is sized from `todo` for the worst case — two bytes per
    // flag for the escaped forms, plus the fixed wrapper — and the flag
    // buffers are `todo + 1`, which is one per flag plus a terminator.
    unsafe {
        let mut todo = len;
        if todo < 2 {
            return SP_FORMERROR;
        }

        todo -= 1;
        let mut c = getc(fd);
        (*slang).sl_compmax = if c < 2 { MAXWLEN as c_int } else { c };
        todo -= 1;
        c = getc(fd);
        (*slang).sl_compminlen = if c < 1 { 0 } else { c };
        todo -= 1;
        c = getc(fd);
        (*slang).sl_compsylmax = if c < 1 { MAXWLEN as c_int } else { c };

        // A zero here marks the newer layout, which adds the options byte
        // and the CHECKCOMPOUNDPATTERN list; anything else is a flag of
        // the old layout and gets pushed back.
        c = getc(fd);
        if c != 0 {
            ungetc(c, fd);
        } else {
            todo -= 1;
            (*slang).sl_compoptions = getc(fd);
            todo -= 1;

            let gap = &raw mut (*slang).sl_comppat;
            let mut cnt = get2c(fd);
            if cnt < 0 {
                return SP_TRUNCERROR;
            }
            todo -= 2;
            ga_init(gap, core::mem::size_of::<*mut c_char>() as c_int, cnt);
            ga_grow(gap, cnt);
            while cnt > 0 {
                cnt -= 1;
                let slot = (*gap)
                    .ga_data
                    .cast::<*mut c_char>()
                    .offset((*gap).ga_len as isize);
                (*gap).ga_len += 1;
                let mut n: c_int = 0;
                *slot = read_cnt_string(fd, 1, &raw mut n);
                if n < 0 {
                    return n;
                }
                todo -= n + 1;
            }
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
        let pat = xmalloc(patsize).cast::<c_char>();

        let mut cp = xmalloc(flagsize).cast::<uint8_t>();
        (*slang).sl_compstartflags = cp;
        *cp = NUL as uint8_t;
        let mut ap = xmalloc(flagsize).cast::<uint8_t>();
        (*slang).sl_compallflags = ap;
        *ap = NUL as uint8_t;
        let mut crp = xmalloc(flagsize).cast::<uint8_t>();
        (*slang).sl_comprules = crp;

        let mut pp = pat;
        for ch in [b'^', b'\\', b'('] {
            *pp = ch as c_char;
            pp = pp.add(1);
        }

        // `atstart` is 1 while the next flag would begin a compound, and 2
        // inside a `[...]` set at that position.
        let mut atstart = 1;
        while todo > 0 {
            todo -= 1;
            c = getc(fd);
            if c == EOF {
                xfree(pat.cast());
                return SP_TRUNCERROR;
            }

            // Collect the set of all flags, and the set that may start a
            // compound, skipping the regexp punctuation.
            if vim_strchr(c"?*+[]/".as_ptr(), c).is_null()
                && !byte_in_str((*slang).sl_compallflags, c)
            {
                *ap = c as uint8_t;
                ap = ap.add(1);
                *ap = NUL as uint8_t;
            }
            if atstart != 0 {
                if c == b'[' as c_int {
                    atstart = 2;
                } else if c == b']' as c_int {
                    atstart = 0;
                } else {
                    if !byte_in_str((*slang).sl_compstartflags, c) {
                        *cp = c as uint8_t;
                        cp = cp.add(1);
                        *cp = NUL as uint8_t;
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
                    xfree((*slang).sl_comprules.cast());
                    (*slang).sl_comprules = core::ptr::null_mut();
                    crp = core::ptr::null_mut();
                } else {
                    *crp = c as uint8_t;
                    crp = crp.add(1);
                }
            }

            if c == b'/' as c_int {
                *pp = b'\\' as c_char;
                *pp.add(1) = b'|' as c_char;
                pp = pp.add(2);
                atstart = 1;
            } else {
                if c == b'?' as c_int || c == b'+' as c_int || c == b'~' as c_int {
                    *pp = b'\\' as c_char;
                    pp = pp.add(1);
                }
                pp = pp.offset(utf_char2bytes(c, pp) as isize);
            }
        }

        for ch in [b'\\', b')', b'$'] {
            *pp = ch as c_char;
            pp = pp.add(1);
        }
        *pp = NUL as c_char;
        if !crp.is_null() {
            *crp = NUL as uint8_t;
        }

        (*slang).sl_compprog = vim_regcomp(pat, RE_MAGIC + RE_STRING + RE_STRICT);
        xfree(pat.cast());
        if (*slang).sl_compprog.is_null() {
            return SP_FORMERROR;
        }
        0
    }
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
    // SAFETY: the caller promises the strings; the second pass writes
    // exactly as many pairs as the first pass counted.
    unsafe {
        let gap = &raw mut (*lp).sl_sal;
        ga_init(gap, core::mem::size_of::<*mut c_int>() as c_int, 1);
        ga_grow(gap, 256);
        memset((*gap).ga_data, 0, core::mem::size_of::<*mut c_int>() * 256);
        (*gap).ga_len = 256;

        // First pass: how many high characters share each low byte.
        let mut p = from;
        let mut s = to;
        while *p as c_int != NUL && *s as c_int != NUL {
            let c = mb_cptr2char_adv(&raw mut p);
            s = s.offset(utf_ptr2len(s) as isize);
            if c >= 256 {
                (*lp).sl_sal_first[(c & 0xff) as usize] += 1;
            }
        }
        // The two strings must describe the same number of characters.
        if *p as c_int != NUL || *s as c_int != NUL {
            return SP_FORMERROR;
        }

        for i in 0..256 {
            if (*lp).sl_sal_first[i] > 0 {
                // Room for each pair plus a zero terminator.
                let n = (*lp).sl_sal_first[i] as size_t * 2 + 1;
                let list = xmalloc(core::mem::size_of::<c_int>() * n).cast::<c_int>();
                *(*gap).ga_data.cast::<*mut c_int>().add(i) = list;
                *list = 0;
            }
        }
        memset(
            (&raw mut (*lp).sl_sal_first).cast(),
            0,
            core::mem::size_of::<salfirst_T>() * 256,
        );

        // Second pass: fill the lists and the direct table.
        let mut p = from;
        let mut s = to;
        while *p as c_int != NUL && *s as c_int != NUL {
            let c = mb_cptr2char_adv(&raw mut p);
            let to_c = mb_cptr2char_adv(&raw mut s);
            if c >= 256 {
                let mut inp = *(*gap)
                    .ga_data
                    .cast::<*mut c_int>()
                    .offset((c & 0xff) as isize);
                while *inp != 0 {
                    inp = inp.add(1);
                }
                *inp = c;
                *inp.add(1) = to_c;
                *inp.add(2) = NUL;
            } else {
                (*lp).sl_sal_first[c as usize] = to_c as salfirst_T;
            }
        }
        0
    }
}

/// Index the `SAL` rules by the low byte of their first character, and
/// gather the rules that share one so the search can stop at the first
/// mismatch.
///
/// # Safety
///
/// `lp` must hold a filled `sl_sal`.
unsafe fn set_sal_first(lp: *mut slang_T) {
    // SAFETY: `sl_sal` holds `ga_len` items with wide lead strings.
    unsafe {
        let gap = &raw mut (*lp).sl_sal;
        let sfirst = (&raw mut (*lp).sl_sal_first).cast::<salfirst_T>();
        for i in 0..256 {
            *sfirst.offset(i) = -1 as salfirst_T;
        }

        let smp = (*gap).ga_data.cast::<salitem_T>();
        let mut i = 0;
        while i < (*gap).ga_len {
            let c = *(*smp.offset(i as isize)).sm_lead_w & 0xff;
            if *sfirst.offset(c as isize) == -1 {
                *sfirst.offset(c as isize) = i as salfirst_T;
                // Skip the run that is already together.
                while i + 1 < (*gap).ga_len
                    && *(*smp.offset((i + 1) as isize)).sm_lead_w & 0xff == c
                {
                    i += 1;
                }
                // Pull any later rule with the same low byte up to join it.
                let mut n = 1;
                while i + n < (*gap).ga_len {
                    if *(*smp.offset((i + n) as isize)).sm_lead_w & 0xff == c {
                        i += 1;
                        n -= 1;
                        let tsal = *smp.offset((i + n) as isize);
                        core::ptr::copy(
                            smp.offset(i as isize),
                            smp.offset(i as isize).add(1),
                            n as usize,
                        );
                        *smp.offset(i as isize) = tsal;
                    }
                    n += 1;
                }
            }
            i += 1;
        }
    }
}

/// Copy a string into a NUL-terminated array of characters.
///
/// # Safety
///
/// `s` must be a NUL-terminated string.
unsafe fn mb_str2wide(s: *const c_char) -> *mut c_int {
    // SAFETY: the array is sized from the string's character count.
    unsafe {
        let res =
            xmalloc((mb_charlen(s) as size_t + 1) * core::mem::size_of::<c_int>()).cast::<c_int>();
        let mut i = 0;
        let mut p = s;
        while *p as c_int != NUL {
            *res.offset(i) = mb_ptr2char_adv(&raw mut p);
            i += 1;
        }
        *res.offset(i) = NUL;
        res
    }
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
pub unsafe fn set_map_str(lp: *mut slang_T, map: *const c_char) {
    // SAFETY: the caller promises the string; each hash key is its own
    // allocation, owned by the table.
    unsafe {
        if *map as c_int == NUL {
            (*lp).sl_has_map = false;
            return;
        }
        (*lp).sl_has_map = true;

        for i in 0..256 {
            (*lp).sl_map_array[i] = 0;
        }
        hash_init(&raw mut (*lp).sl_map_hash);

        // The first character of a group represents the whole group.
        let mut headc = 0;
        let mut p = map;
        while *p as c_int != NUL {
            let c = mb_cptr2char_adv(&raw mut p);
            if c == b'/' as c_int {
                headc = 0;
                continue;
            }
            if headc == 0 {
                headc = c;
            }
            if c < 256 {
                (*lp).sl_map_array[c as usize] = headc;
                continue;
            }

            // Key: the character, a NUL, then its group's head.
            let cl = utf_char2len(c);
            let headcl = utf_char2len(headc);
            let b = xmalloc((cl + headcl) as size_t + 2).cast::<c_char>();
            utf_char2bytes(c, b);
            *b.offset(cl as isize) = NUL as c_char;
            utf_char2bytes(headc, b.offset(cl as isize).add(1));
            *b.offset((cl + 1 + headcl) as isize) = NUL as c_char;

            let hash: hash_T = hash_hash(b);
            let hi: *mut hashitem_T = hash_lookup(&raw mut (*lp).sl_map_hash, b, strlen(b), hash);
            if (*hi).hi_key.is_null() || (*hi).hi_key == (&raw const hash_removed).cast_mut().cast()
            {
                hash_add_item(&raw mut (*lp).sl_map_hash, hi, b, hash);
            } else {
                emsg(gettext(
                    e_duplicate_char_in_map_entry.ptr().cast::<c_char>(),
                ));
                xfree(b.cast());
            }
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
    unsafe {
        let flags = flags_in.cast::<uint8_t>();
        let mut new_st: spelltab_T = core::mem::zeroed();
        clear_spell_chartab(&raw mut new_st);

        // Only the high half is described; the low half is fixed.
        let mut p = fol;
        for i in 0..128 {
            if i < cnt {
                let f = *flags.offset(i as isize) as c_int;
                new_st.st_isw[(i + 128) as usize] = f & CF_WORD as c_int != 0;
                new_st.st_isu[(i + 128) as usize] = f & CF_UPPER as c_int != 0;
            }
            if *p as c_int != NUL {
                let c = mb_ptr2char_adv(&raw mut p);
                new_st.st_fold[(i + 128) as usize] = c as uint8_t;
                // Record the reverse mapping too, when it fits.
                if i + 128 != c && new_st.st_isu[(i + 128) as usize] && c < 256 {
                    new_st.st_upper[c as usize] = (i + 128) as uint8_t;
                }
            }
        }
        set_spell_finish(&raw mut new_st);
    }
}
