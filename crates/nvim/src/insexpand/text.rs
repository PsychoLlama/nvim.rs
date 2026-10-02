//! Word, line and case handling over the text a match is built from.
//!
//! [`ins_compl_add_infercase`] is `'infercase'`: it re-cases a match to match
//! what the user typed.  [`find_common_prefix`] computes the longest common
//! prefix `'longest'` inserts, and the `find_word_*` / `find_line_end`
//! helpers are the scans every buffer source walks with.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::cstr;
use crate::mbyte::cluster_len;
use crate::memory::handoff::owned_cstr;
use crate::types::{IOSIZE, NUL};
use crate::winlayer::buffers;
use crate::winlayer::{Buf, Win};
use core::slice;

/// The completed text with the case of the originally typed text inferred.
///
/// The answer is `out` unless it did not fit, in which case `tofree` is set
/// to the allocation the answer lives in.
///
/// # Safety
///
/// `str` must point at a NUL-terminated string. `tofree` must point at a
/// writable `*mut c_char` slot the caller owns for the call.
unsafe fn ins_compl_infercase_gettext(
    str: *const c_char,
    char_len: c_int,
    compl_char_len: c_int,
    min_len: c_int,
    out: &mut [c_char; IOSIZE as usize],
    tofree: *mut *mut c_char,
) -> *mut c_char {
    // The completion as wide characters, so the case rules below can
    // rewrite it in place.
    let mut wca: Vec<c_int> = Vec::with_capacity(char_len as usize);
    let mut p = str;
    for _ in 0..char_len {
        wca.push(unsafe { mb_ptr2char_adv(&raw mut p) });
    }

    // Rule 1: were any chars converted to lower?
    let mut has_lower = false;
    let mut p = compl_orig_text().data() as *const c_char;
    for i in 0..min_len {
        let c = unsafe { mb_ptr2char_adv(&raw mut p) };
        if mb_islower(c) {
            has_lower = true;
            if mb_isupper(wca[i as usize]) {
                // Rule 1 is satisfied.
                for w in &mut wca[compl_char_len.min(char_len) as usize..] {
                    *w = mb_tolower(*w);
                }
                break;
            }
        }
    }

    // Rule 2: no lower case, 2nd consecutive letter converted to upper case.
    if !has_lower {
        let mut was_letter = false;
        let mut p = compl_orig_text().data() as *const c_char;
        for i in 0..min_len {
            let c = unsafe { mb_ptr2char_adv(&raw mut p) };
            if was_letter && mb_isupper(c) && mb_islower(wca[i as usize]) {
                // Rule 2 is satisfied.
                for w in &mut wca[compl_char_len.min(char_len) as usize..] {
                    *w = mb_toupper(*w);
                }
                break;
            }
            was_letter = mb_islower(c) || mb_isupper(c);
        }
    }

    // Copy the original case of the part we typed.
    let mut p = compl_orig_text().data() as *const c_char;
    for w in wca.iter_mut().take(min_len as usize) {
        let c = unsafe { mb_ptr2char_adv(&raw mut p) };
        if mb_islower(c) {
            *w = mb_tolower(*w);
        } else if mb_isupper(c) {
            *w = mb_toupper(*w);
        }
    }

    // Encode the wide characters back. `out` is used until a character
    // would come within six bytes of its end (five for the widest
    // sequence, one for the NUL), at which point everything written so far
    // moves into an owned buffer and the rest is appended there.
    let iobuff = out.as_mut_ptr();
    let mut spilled: Option<Vec<u8>> = None;
    let mut out = iobuff;
    let mut i = 0;
    while i < char_len {
        if let Some(buf) = spilled.as_mut() {
            // Room for the widest sequence, then cut back to what was
            // written -- the shape `ga_grow(10)` plus `ga_len +=` had.
            let at = buf.len();
            buf.resize(at + 10, 0);
            // SAFETY: `utf_char2bytes` writes at most six bytes, and ten
            // were just made available at `at`.
            let n = unsafe { utf_char2bytes(wca[i as usize], buf.as_mut_ptr().add(at).cast()) };
            buf.truncate(at + n as usize);
            i += 1;
        } else if unsafe { out.offset_from(iobuff) } + 6 >= IOSIZE as isize {
            // Add the character in the next round.
            // SAFETY: `iobuff` holds the bytes written so far.
            let used = unsafe { out.offset_from(iobuff) } as usize;
            spilled = Some(unsafe { slice::from_raw_parts(iobuff.cast::<u8>(), used) }.to_vec());
        } else {
            out = unsafe { out.offset(utf_char2bytes(wca[i as usize], out) as isize) };
            i += 1;
        }
    }

    if let Some(buf) = spilled {
        let owned = owned_cstr(buf);
        unsafe { *tofree = owned };
        return owned;
    }
    unsafe { *out = NUL as c_char };
    iobuff
}

/// [`ins_compl_add`], but with `'ignorecase'` and `'infercase'` set the case of
/// the originally typed text is kept and the case of the rest is inferred —
/// i.e. this works out what case you probably wanted the rest of the word in.
///
/// `cont_s_ipos` says the next `CTRL-X <>` sets the initial position.
///
/// # Safety
///
/// `str_arg` must point at a NUL-terminated string whose first `len` bytes
/// (or all of it, if shorter) are the match. `fname` must be null or point at
/// a NUL-terminated string.
pub unsafe fn ins_compl_add_infercase(
    str_arg: *mut c_char,
    len: c_int,
    icase: bool,
    fname: *mut c_char,
    dir: Direction,
    cont_s_ipos: bool,
    score: c_int,
) -> c_int {
    // Where `'infercase'` re-cases the match; upstream shares `IObuff`.
    let mut recased = [0 as c_char; IOSIZE as usize];
    let mut str = str_arg;
    let mut tofree: *mut c_char = ptr::null_mut();
    // C's MB_PTR_ADV: step one (possibly composed) character.
    let char_count = |mut p: *const c_char| {
        let mut n = 0;
        while unsafe { *p } as c_int != NUL {
            p = unsafe { p.offset(utfc_ptr2len(p.cast_mut()) as isize) };
            n += 1;
        }
        n
    };

    if p_ic() && Buf::current().b_p_inf != 0 && len > 0 {
        let char_len = char_count(str);
        let compl_char_len = char_count(compl_orig_text().data());
        // "char_len" may be smaller than "compl_char_len" when using
        // thesaurus, only use the minimum when comparing.
        let min_len = char_len.min(compl_char_len);
        let free = &raw mut tofree;
        // SAFETY: `str` is `char_len` characters, `recased` is this frame's
        // scratch buffer and `free` its own local.
        str = unsafe {
            ins_compl_infercase_gettext(str, char_len, compl_char_len, min_len, &mut recased, free)
        };
    }

    let mut flags = 0;
    if cont_s_ipos {
        flags |= CP_CONT_S_IPOS;
    }
    if icase {
        flags |= CP_ICASE;
    }

    // SAFETY: `str` is NUL-terminated (a line, a word copied into a
    // terminated buffer, or the re-cased copy) and the scan stops at its
    // terminator, which is also where the match is cut; `fname` is null or a
    // NUL-terminated name.
    let (text, fname) = unsafe {
        (
            cstr::prefix_at(str, len.max(0) as usize),
            (!fname.is_null()).then(|| cstr::at(fname)),
        )
    };
    let res = ins_compl_add(text, fname, NO_EXTRA, None, dir, flags, false, NO_HL, score);
    unsafe { xfree(tofree.cast::<c_void>()) };
    res
}

/// The first character of the next word, stopping at a NUL.
///
/// # Safety
///
/// `text` must point at a NUL-terminated string, unaliased for the call.
pub unsafe fn find_word_start(mut text: *mut c_char) -> *mut c_char {
    while unsafe { *text } as c_int != NUL
        && unsafe { *text } as c_int != '\n' as c_int
        && unsafe { mb_get_class(text) } <= 1
    {
        text = unsafe { text.offset(utfc_ptr2len(text) as isize) };
    }
    text
}

/// Just after the word `text` points inside of.
///
/// # Safety
///
/// `text` must point at a NUL-terminated string, unaliased for the call.
pub unsafe fn find_word_end(mut text: *mut c_char) -> *mut c_char {
    let start_class = unsafe { mb_get_class(text) };
    if start_class > 1 {
        while unsafe { *text } as c_int != NUL {
            text = unsafe { text.offset(utfc_ptr2len(text) as isize) };
            if unsafe { mb_get_class(text) } != start_class {
                break;
            }
        }
    }
    text
}

/// Just after the line, omitting the CR and NL at its end.
///
/// # Safety
///
/// `text` must point at a NUL-terminated string, unaliased for the call.
pub unsafe fn find_line_end(text: *mut c_char) -> *mut c_char {
    let mut s = unsafe { text.add(cstr::bytes_at(text).len()) };
    while s > text && matches!(unsafe { *s.offset(-1) } as c_int, c if c == CAR || c == NL) {
        s = unsafe { s.offset(-1) };
    }
    s
}

/// Add every listed buffer's file name that starts with what was typed.
pub(crate) fn get_next_bufname_token() {
    for b in buffers() {
        if b.b_p_bl == 0 || b.name.short().is_none() {
            continue;
        }
        // SAFETY: a live buffer from the editor's own list, whose short name
        // is a NUL-terminated string.
        let tail = unsafe { path_tail(b.name.short_ptr()) };
        let (orig_data, orig_len) = compl_orig_text().parts();
        if unsafe { cstr::prefix_eq(tail, orig_data, orig_len) } {
            let flags = if p_ic() { CP_ICASE } else { 0 };
            let (dir, score) = (kDirectionNotSet, FUZZY_SCORE_NONE);
            // SAFETY: `tail` is a NUL-terminated buffer name.
            let tail = unsafe { cstr::bytes_at(tail) };
            ins_compl_add(tail, None, NO_EXTRA, None, dir, flags, false, NO_HL, score);
        }
    }
}

/// `option` without the carets followed by numbers — the `'complete'` `^N`
/// max-matches suffix — that end an entry.
pub(crate) fn strip_caret_numbers(option: &[u8]) -> XString {
    let mut out = Vec::with_capacity(option.len());
    let mut read = 0;
    while read < option.len() {
        if option[read] == b'^' {
            let digits = option[read + 1..]
                .iter()
                .take_while(|b| b.is_ascii_digit())
                .count();
            let after = read + 1 + digits;
            // A caret with at least one digit after it and nothing but the
            // next source's separator beyond: drop the whole run.
            if digits > 0 && matches!(option.get(after), None | Some(b',')) {
                read = after;
                continue;
            }
        }
        out.push(option[read]);
        read += 1;
    }
    XString::from_bytes(&out)
}

/// The longest common prefix among the current matches: the match it was
/// taken from (whose text runs on past it) and the prefix's length; `None`
/// when there is none longer than the leader.
///
/// With `curbuf_only` only matches from the `'complete'` `.` source count.
pub(crate) fn find_common_prefix(curbuf_only: bool) -> Option<(XString, usize)> {
    if cpt_sources().is_unset() {
        return None;
    }

    let mut match_count: Vec<c_int> = vec![0; cpt_sources().len()];
    clear_adjusted_leader();
    let typed = ins_compl_leader_str().to_vec();

    let mut first: Option<XString> = None;
    let mut len = 0;
    for compl in matches_from(first_match()) {
        let leader = get_leader_for_startcol(compl, true);

        // Apply 'smartcase' behavior during normal mode.
        if ctrl_x_mode_normal()
            && !p_inf()
            && !leader.is_unset()
            // SAFETY: the leader is a NUL-terminated string.
            && !unsafe { ignorecase(leader.data()) }
        {
            compl.update(|m| m.flags &= !CP_ICASE);
        }

        let displayed =
            !compl.is_original() && (leader.is_unset() || ins_compl_equal(compl, leader));
        if !displayed {
            continue;
        }
        // Limit the number of items from each source if max_items is set.
        let mut match_limit_exceeded = false;
        let cur_source = compl.with(|m| m.cpt_source_idx);
        if cur_source != -1 {
            match_count[cur_source as usize] += 1;
            let max_matches = cpt_sources().row(cur_source).cs_max_matches;
            if max_matches > 0 && match_count[cur_source as usize] > max_matches {
                match_limit_exceeded = true;
            }
        }

        let from_curbuf =
            cur_source != -1 && cpt_sources().row(cur_source).cs_flag as c_int == '.' as c_int;
        if match_limit_exceeded || (curbuf_only && !from_curbuf) {
            continue;
        }
        match &first {
            None => {
                let text = compl.text_copy();
                if text.starts_with(&typed) {
                    len = text.len();
                    first = Some(text);
                }
            }
            Some(prefix) => {
                // Shorten the prefix to what this match still agrees on.
                len = compl.with(|m| shared_prefix_len(prefix, len, &m.text));
                if len == 0 {
                    break;
                }
            }
        }
    }

    if len <= typed.len() {
        return None;
    }
    let first = first.expect("a prefix longer than the leader came from a match");
    // Avoid inserting text that duplicates the text already after the cursor.
    if len == first.len() {
        let line = get_cursor_line_ptr();
        // SAFETY: the cursor column is inside the cursor line.
        let p = unsafe { line.offset(Win::current().w_cursor.col as isize) };
        if !p.is_null() && !ascii_iswhite_or_nul(unsafe { *p } as c_int) {
            // SAFETY: `find_word_end` answers a pointer into the same line.
            let text_len = unsafe { find_word_end(p).offset_from(p) } as usize;
            // SAFETY: `p` has `text_len` bytes of the word.
            let word = unsafe { cstr::slice_at(p, text_len) };
            if text_len > 0
                && text_len < len - typed.len()
                && first[len - text_len..].starts_with(word)
            {
                len -= text_len;
            }
        }
    }
    Some((first, len))
}

/// How many of the first `len` bytes of `prefix` `text` shares: upstream's
/// walk, which counts the bytes of each base character but steps a whole
/// cluster at a time.
fn shared_prefix_len(prefix: &[u8], len: usize, text: &[u8]) -> usize {
    // C's MB_BYTE2LEN: bytes in the sequence this byte starts.
    let byte2len = |b: u8| usize::from(utf8len_tab[usize::from(b)]);
    let (mut j, mut s1, mut s2) = (0, 0, 0);
    while j < len && s1 < prefix.len() && s2 < text.len() {
        let n = byte2len(prefix[s1]);
        let same = n == byte2len(text[s2])
            && matches!(
                (prefix.get(s1..s1 + n), text.get(s2..s2 + n)),
                (Some(a), Some(b)) if a == b
            );
        if !same {
            break;
        }
        j += n;
        s1 += cluster_len(&prefix[s1..]);
        s2 += cluster_len(&text[s2..]);
    }
    j
}

/// Look in the first `len` characters of `src` for search metacharacters.
///
/// When `dest` is not null they are copied there, quoting the metacharacters
/// with a backslash, and `dest` is NUL terminated. Answers the length `dest`
/// needs either way.
///
/// # Safety
///
/// `dest` must point at a NUL-terminated string, unaliased for the call.
/// `src` must point at a NUL-terminated string, unaliased for the call.
pub(crate) unsafe fn quote_meta(
    mut dest: *mut c_char,
    mut src: *mut c_char,
    mut len: c_int,
) -> c_uint {
    let mut m = len as c_uint + 1; // one extra for the NUL
    loop {
        len -= 1;
        if len < 0 {
            break;
        }
        // C's switch falls through label by label, each guard `break`ing
        // out of it (no quoting) or dropping into the next label's guard:
        // `.`/`*`/`[` test dictionary-or-thesaurus and then 'magic', `~`
        // tests 'magic' and then dictionary-or-thesaurus, `\` only the
        // former, and `^`/`$` neither. Both queries read a global, so the
        // two orders agree.
        let dict_or_thesaurus = || ctrl_x_mode_dictionary() || ctrl_x_mode_thesaurus();
        let quote = match unsafe { *src } as u8 {
            b'.' | b'*' | b'[' | b'~' => magic_isset() && !dict_or_thesaurus(),
            b'\\' => !dict_or_thesaurus(),
            // Currently `^` is not needed.
            b'^' | b'$' => true,
            _ => false,
        };
        if quote {
            m += 1;
            if !dest.is_null() {
                unsafe { *dest = '\\' as c_char };
                dest = unsafe { dest.offset(1) };
            }
        }
        if !dest.is_null() {
            unsafe { *dest = *src };
            dest = unsafe { dest.offset(1) };
        }
        // Copy the remaining bytes of a multibyte character.
        let mb_len = unsafe { utfc_ptr2len(src) } - 1;
        if mb_len > 0 && len >= mb_len {
            for _ in 0..mb_len {
                len -= 1;
                src = unsafe { src.offset(1) };
                if !dest.is_null() {
                    unsafe { *dest = *src };
                    dest = unsafe { dest.offset(1) };
                }
            }
        }
        src = unsafe { src.offset(1) };
    }
    if !dest.is_null() {
        unsafe { *dest = NUL as c_char };
    }
    m
}
