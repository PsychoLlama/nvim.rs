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

use crate::fileio::{get2c, read_string};
use crate::garray::{ga_grow, ga_init};
use crate::hashtab::{hash_add_item, hash_hash, hash_init, hash_lookup, hash_removed};
use crate::mbyte::{
    mb_charlen, mb_cptr2char_adv, mb_ptr2char_adv, utf_char2bytes, utf_char2len, utf_ptr2len,
};
use crate::memory::{xcalloc, xfree, xmalloc};
use crate::message::emsg;
use crate::os::cshim::{getc, gettext};
use crate::spell::{ascii_spell_chartab, byte_in_str, count_common_word};
use crate::strings::vim_strchr;
use crate::types::{
    FILE, NUL, fromto_T, garray_T, hash_T, hashitem_T, int16_t, regprog_T, salfirst_T, salitem_T,
    size_t, slang_T, uint8_t,
};
use ::libc::{memset, strlen, ungetc};

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
    gap: *mut garray_T,
    first: *mut int16_t,
) -> c_int {
    // SAFETY: the caller promises the array and the table; `ga_grow` makes
    // room for `cnt` entries before any is written.
    let cnt = unsafe { get2c(fd) };
    if cnt < 0 {
        return SP_TRUNCERROR;
    }
    unsafe { ga_grow(gap, cnt) };

    while unsafe { (*gap).ga_len } < cnt {
        let ftp = unsafe {
            (*gap)
                .ga_data
                .cast::<fromto_T>()
                .offset((*gap).ga_len as isize)
        };
        let mut c: c_int = 0;
        unsafe { (*ftp).ft_from = read_cnt_string(fd, 1, &raw mut c) };
        if c < 0 {
            return c;
        }
        if c == 0 {
            return SP_FORMERROR;
        }
        unsafe { (*ftp).ft_to = read_cnt_string(fd, 1, &raw mut c) };
        if c <= 0 {
            unsafe { xfree((*ftp).ft_from.cast()) };
            return if c < 0 { c } else { SP_FORMERROR };
        }
        unsafe { (*gap).ga_len += 1 };
    }

    // Entries arrive sorted, so the first index per leading byte is
    // all the search needs.
    for i in 0..256 {
        unsafe { *first.offset(i) = -1 };
    }
    for i in 0..unsafe { (*gap).ga_len } {
        let ftp = unsafe { (*gap).ga_data.cast::<fromto_T>().offset(i as isize) };
        let lead = unsafe { *(*ftp).ft_from } as uint8_t as isize;
        if unsafe { *first.offset(lead) } == -1 {
            unsafe { *first.offset(lead) = i as int16_t };
        }
    }
    0
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
    let gap = unsafe { &raw mut (*slang).sl_sal };
    unsafe { ga_init(gap, size_of::<salitem_T>() as c_int, 10) };
    // One spare for the terminating entry appended below.
    unsafe { ga_grow(gap, cnt + 1) };

    while unsafe { (*gap).ga_len } < cnt {
        let smp = unsafe {
            (*gap)
                .ga_data
                .cast::<salitem_T>()
                .offset((*gap).ga_len as isize)
        };
        let mut ccnt = unsafe { getc(fd) };
        if ccnt < 0 {
            return SP_TRUNCERROR;
        }
        let mut p = unsafe { xmalloc((ccnt as size_t) + 2) }.cast::<c_char>();
        unsafe { (*smp).sm_lead = p };

        // The lead: everything up to the first rule character.
        let mut c = NUL;
        let mut i = 0;
        while i < ccnt {
            c = unsafe { getc(fd) };
            if !unsafe { vim_strchr(c"0123456789(-<^$".as_ptr(), c) }.is_null() {
                break;
            }
            unsafe { *p = c as uint8_t as c_char };
            p = unsafe { p.add(1) };
            i += 1;
        }
        unsafe { (*smp).sm_leadlen = p.offset_from((*smp).sm_lead) as c_int };
        unsafe { *p = NUL as c_char };
        p = unsafe { p.add(1) };

        // An optional "(abc)" set of characters any of which may
        // follow the lead.
        if c == b'(' as c_int {
            unsafe { (*smp).sm_oneof = p };
            i += 1;
            while i < ccnt {
                c = unsafe { getc(fd) };
                if c == b')' as c_int {
                    break;
                }
                unsafe { *p = c as uint8_t as c_char };
                p = unsafe { p.add(1) };
                i += 1;
            }
            unsafe { *p = NUL as c_char };
            p = unsafe { p.add(1) };
            i += 1;
            if i < ccnt {
                c = unsafe { getc(fd) };
            }
        } else {
            unsafe { (*smp).sm_oneof = core::ptr::null_mut() };
        }

        // Whatever is left is the rule.
        unsafe { (*smp).sm_rules = p };
        if i < ccnt {
            unsafe { *p = c as uint8_t as c_char };
            p = unsafe { p.add(1) };
        }
        i += 1;
        if i < ccnt {
            if let Err(e) = unsafe { read_nonnul_bytes(fd, p, (ccnt - i) as usize) } {
                unsafe { xfree((*smp).sm_lead.cast()) };
                return e;
            }
            p = unsafe { p.offset((ccnt - i) as isize) };
        }
        unsafe { *p = NUL as c_char };

        unsafe { (*smp).sm_to = read_cnt_string(fd, 1, &raw mut ccnt) };
        if ccnt < 0 {
            unsafe { xfree((*smp).sm_lead.cast()) };
            return ccnt;
        }

        // Wide copies, since sound folding works in characters.
        unsafe { (*smp).sm_lead_w = mb_str2wide((*smp).sm_lead) };
        unsafe { (*smp).sm_leadlen = mb_charlen((*smp).sm_lead) };
        let oneof = unsafe { (*smp).sm_oneof };
        let wide = if oneof.is_null() {
            core::ptr::null_mut()
        } else {
            unsafe { mb_str2wide(oneof) }
        };
        unsafe { (*smp).sm_oneof_w = wide };
        let to = unsafe { (*smp).sm_to };
        let wide = if to.is_null() {
            core::ptr::null_mut()
        } else {
            unsafe { mb_str2wide(to) }
        };
        unsafe { (*smp).sm_to_w = wide };
        unsafe { (*gap).ga_len += 1 };
    }

    if unsafe { (*gap).ga_len } > 0 {
        // A final empty rule, so the search always has one to stop on.
        let smp = unsafe {
            (*gap)
                .ga_data
                .cast::<salitem_T>()
                .offset((*gap).ga_len as isize)
        };
        let p = unsafe { xmalloc(1) }.cast::<c_char>();
        unsafe { *p = NUL as c_char };
        unsafe { (*smp).sm_lead = p };
        unsafe { (*smp).sm_lead_w = mb_str2wide(p) };
        unsafe { (*smp).sm_leadlen = 0 };
        unsafe { (*smp).sm_oneof = core::ptr::null_mut() };
        unsafe { (*smp).sm_oneof_w = core::ptr::null_mut() };
        unsafe { (*smp).sm_rules = p };
        unsafe { (*smp).sm_to = core::ptr::null_mut() };
        unsafe { (*smp).sm_to_w = core::ptr::null_mut() };
        unsafe { (*gap).ga_len += 1 };
    }

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

        let gap = unsafe { &raw mut (*slang).sl_comppat };
        let mut cnt = unsafe { get2c(fd) };
        if cnt < 0 {
            return SP_TRUNCERROR;
        }
        todo -= 2;
        unsafe { ga_init(gap, size_of::<*mut c_char>() as c_int, cnt) };
        unsafe { ga_grow(gap, cnt) };
        while cnt > 0 {
            cnt -= 1;
            let slot = unsafe {
                (*gap)
                    .ga_data
                    .cast::<*mut c_char>()
                    .offset((*gap).ga_len as isize)
            };
            unsafe { (*gap).ga_len += 1 };
            let mut n: c_int = 0;
            unsafe { *slot = read_cnt_string(fd, 1, &raw mut n) };
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
    // SAFETY: the caller promises the strings; the second pass writes
    // exactly as many pairs as the first pass counted.
    let gap = unsafe { &raw mut (*lp).sl_sal };
    unsafe { ga_init(gap, size_of::<*mut c_int>() as c_int, 1) };
    unsafe { ga_grow(gap, 256) };
    unsafe { memset((*gap).ga_data, 0, size_of::<*mut c_int>() * 256) };
    unsafe { (*gap).ga_len = 256 };

    // First pass: how many high characters share each low byte.
    let mut p = from;
    let mut s = to;
    while unsafe { *p } as c_int != NUL && unsafe { *s } as c_int != NUL {
        let c = unsafe { mb_cptr2char_adv(&raw mut p) };
        s = unsafe { s.offset(utf_ptr2len(s) as isize) };
        if c >= 256 {
            unsafe { (*lp).sl_sal_first[(c & 0xff) as usize] += 1 };
        }
    }
    // The two strings must describe the same number of characters.
    if unsafe { *p } as c_int != NUL || unsafe { *s } as c_int != NUL {
        return SP_FORMERROR;
    }

    for i in 0..256 {
        if unsafe { (*lp).sl_sal_first[i] } > 0 {
            // Room for each pair plus a zero terminator.
            let n = unsafe { (*lp).sl_sal_first[i] } as size_t * 2 + 1;
            let list = unsafe { xmalloc(size_of::<c_int>() * n) }.cast::<c_int>();
            unsafe { *(*gap).ga_data.cast::<*mut c_int>().add(i) = list };
            unsafe { *list = 0 };
        }
    }
    let first = unsafe { &raw mut (*lp).sl_sal_first }.cast();
    unsafe { memset(first, 0, size_of::<salfirst_T>() * 256) };

    // Second pass: fill the lists and the direct table.
    let mut p = from;
    let mut s = to;
    while unsafe { *p } as c_int != NUL && unsafe { *s } as c_int != NUL {
        let c = unsafe { mb_cptr2char_adv(&raw mut p) };
        let to_c = unsafe { mb_cptr2char_adv(&raw mut s) };
        if c >= 256 {
            let lists = unsafe { (*gap).ga_data.cast::<*mut c_int>() };
            let mut inp = unsafe { *lists.offset((c & 0xff) as isize) };
            while unsafe { *inp } != 0 {
                inp = unsafe { inp.add(1) };
            }
            unsafe { *inp = c };
            unsafe { *inp.add(1) = to_c };
            unsafe { *inp.add(2) = NUL };
        } else {
            unsafe { (*lp).sl_sal_first[c as usize] = to_c as salfirst_T };
        }
    }
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
    // SAFETY: `sl_sal` holds `ga_len` items with wide lead strings.
    let gap = unsafe { &raw mut (*lp).sl_sal };
    let sfirst = (unsafe { &raw mut (*lp).sl_sal_first }).cast::<salfirst_T>();
    for i in 0..256 {
        unsafe { *sfirst.offset(i) = -1 as salfirst_T };
    }

    let smp = unsafe { (*gap).ga_data }.cast::<salitem_T>();
    let mut i = 0;
    while i < unsafe { (*gap).ga_len } {
        let c = unsafe { *(*smp.offset(i as isize)).sm_lead_w } & 0xff;
        if unsafe { *sfirst.offset(c as isize) } == -1 {
            unsafe { *sfirst.offset(c as isize) = i as salfirst_T };
            // Skip the run that is already together.
            while i + 1 < unsafe { (*gap).ga_len }
                && unsafe { *(*smp.offset((i + 1) as isize)).sm_lead_w } & 0xff == c
            {
                i += 1;
            }
            // Pull any later rule with the same low byte up to join it.
            let mut n = 1;
            while i + n < unsafe { (*gap).ga_len } {
                if unsafe { *(*smp.offset((i + n) as isize)).sm_lead_w } & 0xff == c {
                    i += 1;
                    n -= 1;
                    let tsal = unsafe { *smp.offset((i + n) as isize) };
                    let from = unsafe { smp.offset(i as isize) };
                    unsafe { core::ptr::copy(from, from.add(1), n as usize) };
                    unsafe { *smp.offset(i as isize) = tsal };
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
unsafe fn mb_str2wide(s: *const c_char) -> *mut c_int {
    // SAFETY: the array is sized from the string's character count.
    let res =
        unsafe { xmalloc((mb_charlen(s) as size_t + 1) * size_of::<c_int>()) }.cast::<c_int>();
    let mut i = 0;
    let mut p = s;
    while unsafe { *p } as c_int != NUL {
        unsafe { *res.offset(i) = mb_ptr2char_adv(&raw mut p) };
        i += 1;
    }
    unsafe { *res.offset(i) = NUL };
    res
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
    unsafe { hash_init(&raw mut (*lp).sl_map_hash) };

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
        let hi: *mut hashitem_T =
            unsafe { hash_lookup(&raw mut (*lp).sl_map_hash, b, strlen(b), hash) };
        if unsafe { (*hi).hi_key }.is_null()
            || unsafe { (*hi).hi_key } == (&raw const hash_removed).cast_mut().cast()
        {
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
