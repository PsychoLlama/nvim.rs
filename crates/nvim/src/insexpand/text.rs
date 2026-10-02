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
use crate::mbyte::{char_at, class_in, cluster_len, encode_char};
use crate::types::NUL;
use crate::winlayer::buffers;
use crate::winlayer::{Buf, Win};

/// The characters of `text`, one per cluster, as `mb_ptr2char_adv` walks
/// them.
fn chars_of(text: &[u8]) -> Vec<c_int> {
    let mut chars = Vec::new();
    let mut at = 0;
    while at < text.len() {
        chars.push(char_at(&text[at..]));
        at += cluster_len(&text[at..]);
    }
    chars
}

/// `text` with the case of the originally typed text inferred: what case you
/// probably wanted the rest of the word in.
fn infercase_text(text: &[u8], typed: &[u8]) -> Vec<u8> {
    // The completion as characters, so the case rules below can rewrite it
    // in place.
    let mut wca = chars_of(text);
    let typed = chars_of(typed);
    let (char_len, compl_char_len) = (wca.len(), typed.len());
    // `char_len` may be smaller than `compl_char_len` when using a
    // thesaurus: only the minimum is compared.
    let min_len = char_len.min(compl_char_len);
    let rest = compl_char_len.min(char_len);

    // Rule 1: were any chars converted to lower?
    let mut has_lower = false;
    for i in 0..min_len {
        if mb_islower(typed[i]) {
            has_lower = true;
            if mb_isupper(wca[i]) {
                // Rule 1 is satisfied.
                for w in &mut wca[rest..] {
                    *w = mb_tolower(*w);
                }
                break;
            }
        }
    }

    // Rule 2: no lower case, 2nd consecutive letter converted to upper case.
    if !has_lower {
        let mut was_letter = false;
        for i in 0..min_len {
            let c = typed[i];
            if was_letter && mb_isupper(c) && mb_islower(wca[i]) {
                // Rule 2 is satisfied.
                for w in &mut wca[rest..] {
                    *w = mb_toupper(*w);
                }
                break;
            }
            was_letter = mb_islower(c) || mb_isupper(c);
        }
    }

    // Copy the original case of the part we typed.
    for (w, &c) in wca.iter_mut().zip(&typed).take(min_len) {
        if mb_islower(c) {
            *w = mb_tolower(*w);
        } else if mb_isupper(c) {
            *w = mb_toupper(*w);
        }
    }

    let mut out = Vec::with_capacity(text.len());
    let mut buf = [0u8; 6];
    for &w in &wca {
        let n = encode_char(w, &mut buf);
        out.extend_from_slice(&buf[..n]);
    }
    out
}

/// [`ins_compl_add`], but with `'ignorecase'` and `'infercase'` set the case of
/// the originally typed text is kept and the case of the rest is inferred —
/// i.e. this works out what case you probably wanted the rest of the word in.
///
/// The match is the first `len` bytes of `rest`, which runs on to the end of
/// the string the match was found in: upstream re-cases all of that and then
/// takes `len` bytes, so a case change that alters a byte length shifts
/// which bytes those are, and this does the same. `cont_s_ipos` says the next
/// `CTRL-X <>` sets the initial position.
pub fn ins_compl_add_infercase(
    rest: &[u8],
    len: usize,
    icase: bool,
    fname: Option<&CStr>,
    dir: Direction,
    cont_s_ipos: bool,
    score: c_int,
) -> c_int {
    let recased;
    let mut text = &rest[..len.min(rest.len())];
    if p_ic() && Buf::current().b_p_inf != 0 && len > 0 {
        let typed = compl_orig_text().to_vec();
        recased = infercase_text(rest, &typed);
        text = &recased[..len.min(recased.len())];
    }

    let mut flags = 0;
    if cont_s_ipos {
        flags |= CP_CONT_S_IPOS;
    }
    if icase {
        flags |= CP_ICASE;
    }
    ins_compl_add(text, fname, NO_EXTRA, None, dir, flags, false, NO_HL, score)
}

/// The offset of the first character of the next word in `text` from `at`,
/// stopping at the end or a line break -- [`find_word_start`] over a slice.
pub(crate) fn word_start(text: &[u8], mut at: usize) -> usize {
    let buffer = Buf::current();
    while at < text.len() && text[at] != b'\n' && class_in(&text[at..], buffer) <= 1 {
        at += cluster_len(&text[at..]);
    }
    at
}

/// The offset just after the word `text[at..]` starts inside of --
/// [`find_word_end`] over a slice.
pub(crate) fn word_end(text: &[u8], mut at: usize) -> usize {
    let buffer = Buf::current();
    let start_class = class_in(&text[at..], buffer);
    if start_class > 1 {
        while at < text.len() {
            at += cluster_len(&text[at..]);
            if class_in(&text[at..], buffer) != start_class {
                break;
            }
        }
    }
    at
}

/// The length of `text` without the CRs and NLs at its end --
/// [`find_line_end`] over a slice.
pub(crate) fn line_end(text: &[u8]) -> usize {
    let mut end = text.len();
    while end > 0 && matches!(text[end - 1], b'\r' | b'\n') {
        end -= 1;
    }
    end
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
        // A match from rows rebuilt since it was added counts against no
        // source.
        let cur_source = compl.with(ComplItem::cpt_source);
        if let Some(source) = cur_source
            && let Some(count) = match_count.get_mut(source)
        {
            *count += 1;
            let max_matches = cpt_sources().row(source as c_int).cs_max_matches;
            if max_matches > 0 && *count > max_matches {
                match_limit_exceeded = true;
            }
        }

        let from_curbuf = cur_source.is_some_and(|source| {
            cpt_sources().row(source as c_int).cs_flag as c_int == '.' as c_int
        });
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
