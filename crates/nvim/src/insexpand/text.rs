//! Word, line and case handling over the text a match is built from.
//!
//! [`ins_compl_add_infercase`] is `'infercase'`: it re-cases a match to match
//! what the user typed.  [`find_common_prefix`] computes the longest common
//! prefix `'longest'` inserts, and the `find_word_*` / `find_line_end`
//! helpers are the scans every buffer source walks with.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cstr;
use crate::types::{IOSIZE, NUL};
use crate::winlayer::buffers;
use crate::winlayer::{Buf, Win};

/// The completed text with the case of the originally typed text inferred.
///
/// The answer is `out` unless it did not fit, in which case `tofree` is set
/// to the allocation the answer lives in.
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
    // moves into a growarray and the rest is appended there.
    let iobuff = out.as_mut_ptr();
    let mut gap = GARRAY_T_INIT;
    let mut out = iobuff;
    let mut i = 0;
    unsafe { ga_init(&raw mut gap, 1, 500) };
    while i < char_len {
        if !gap.ga_data.is_null() {
            unsafe { ga_grow(&raw mut gap, 10) };
            debug_assert!(!gap.ga_data.is_null());
            out = unsafe { gap.ga_data.cast::<c_char>().offset(gap.ga_len as isize) };
            gap.ga_len += unsafe { utf_char2bytes(wca[i as usize], out) };
            i += 1;
        } else if unsafe { out.offset_from(iobuff) } + 6 >= IOSIZE as isize {
            // Add the character in the next round.
            unsafe { ga_grow(&raw mut gap, IOSIZE) };
            unsafe { *out = NUL as c_char };
            unsafe { strcpy(gap.ga_data.cast::<c_char>(), iobuff) };
            gap.ga_len = unsafe { out.offset_from(iobuff) } as c_int;
        } else {
            out = unsafe { out.offset(utf_char2bytes(wca[i as usize], out) as isize) };
            i += 1;
        }
    }

    if !gap.ga_data.is_null() {
        unsafe { *tofree = gap.ga_data.cast::<c_char>() };
        return gap.ga_data.cast::<c_char>();
    }
    unsafe { *out = NUL as c_char };
    iobuff
}

/// [`ins_compl_add`], but with `'ignorecase'` and `'infercase'` set the case of
/// the originally typed text is kept and the case of the rest is inferred —
/// i.e. this works out what case you probably wanted the rest of the word in.
///
/// `cont_s_ipos` says the next `CTRL-X <>` sets the initial position.
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

    if p_ic.get() != 0 && cur_buf().b_p_inf != 0 && len > 0 {
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

    let (no_cptext, no_hl) = (ptr::null(), ptr::null());
    let no_data = ptr::null_mut();
    // SAFETY: `str` is `len` readable bytes and `fname` null or a
    // NUL-terminated name; there is no `cptext`, user data or highlight pair.
    let res = unsafe {
        ins_compl_add(
            str, len, fname, no_cptext, false, no_data, dir, flags, false, no_hl, score,
        )
    };
    unsafe { xfree(tofree.cast::<c_void>()) };
    res
}

/// The first character of the next word, stopping at a NUL.
pub unsafe fn find_word_start(mut ptr: *mut c_char) -> *mut c_char {
    while unsafe { *ptr } as c_int != NUL
        && unsafe { *ptr } as c_int != '\n' as c_int
        && unsafe { mb_get_class(ptr) } <= 1
    {
        ptr = unsafe { ptr.offset(utfc_ptr2len(ptr) as isize) };
    }
    ptr
}

/// Just after the word `ptr` points inside of.
pub unsafe fn find_word_end(mut ptr: *mut c_char) -> *mut c_char {
    let start_class = unsafe { mb_get_class(ptr) };
    if start_class > 1 {
        while unsafe { *ptr } as c_int != NUL {
            ptr = unsafe { ptr.offset(utfc_ptr2len(ptr) as isize) };
            if unsafe { mb_get_class(ptr) } != start_class {
                break;
            }
        }
    }
    ptr
}

/// Just after the line, omitting the CR and NL at its end.
pub unsafe fn find_line_end(ptr: *mut c_char) -> *mut c_char {
    let mut s = unsafe { ptr.add(cstr::bytes_at(ptr).len()) };
    while s > ptr && matches!(unsafe { *s.offset(-1) } as c_int, c if c == CAR || c == NL) {
        s = unsafe { s.offset(-1) };
    }
    s
}

/// Add every listed buffer's file name that starts with what was typed.
pub(crate) unsafe fn get_next_bufname_token() {
    for b in buffers() {
        if b.b_p_bl == 0 || b.b_sfname.is_null() {
            continue;
        }
        // SAFETY: a live buffer from the editor's own list, whose short name
        // is a NUL-terminated string.
        let tail = unsafe { path_tail(b.b_sfname) };
        let orig = compl_orig_text().value();
        if unsafe { cstr::prefix_eq(tail, orig.data(), orig.len()) } {
            let flags = if p_ic.get() != 0 { CP_ICASE } else { 0 };
            let (no_name, no_data) = (ptr::null_mut(), ptr::null_mut());
            let (no_cptext, no_hl) = (ptr::null(), ptr::null());
            let (dir, score) = (kDirectionNotSet, FUZZY_SCORE_NONE);
            // SAFETY: `tail` is a NUL-terminated buffer name, and there is
            // no `cptext`, user data or highlight pair.
            unsafe {
                let len = cstr::bytes_at(tail).len() as c_int;
                let no_dup = false;
                ins_compl_add(
                    tail, len, no_name, no_cptext, false, no_data, dir, flags, no_dup, no_hl, score,
                )
            };
        }
    }
}

/// Strip carets followed by numbers — the `'complete'` `^N` max-matches
/// suffix — in place.
pub(crate) unsafe fn strip_caret_numbers_in_place(str: *mut c_char) {
    if str.is_null() {
        return;
    }
    let mut read = str;
    let mut write = str;
    while unsafe { *read } != 0 {
        if unsafe { *read } as c_int == '^' as c_int {
            let mut p = unsafe { read.offset(1) };
            while ascii_isdigit(unsafe { *p } as c_int) {
                p = unsafe { p.offset(1) };
            }
            // A caret with at least one digit after it and nothing but the
            // next source's separator beyond: drop the whole run.
            if (unsafe { *p } as c_int == ',' as c_int || unsafe { *p } as c_int == '\0' as c_int)
                && p != unsafe { read.offset(1) }
            {
                read = p;
                continue;
            }
        }
        unsafe { *write = *read };
        write = unsafe { write.offset(1) };
        read = unsafe { read.offset(1) };
    }
    unsafe { *write = '\0' as c_char };
}

/// The longest common prefix among the current matches, with `prefix_len` set
/// to its length; null when there is none longer than the leader.
///
/// With `curbuf_only` only matches from the `'complete'` `.` source count.
pub(crate) unsafe fn find_common_prefix(prefix_len: *mut size_t, curbuf_only: bool) -> *mut c_char {
    if cpt_sources().is_unset() {
        return ptr::null_mut();
    }

    // C's MB_BYTE2LEN: bytes in the sequence this byte starts.
    let byte2len = |b: c_char| utf8len_tab[b as u8 as usize] as c_int;

    let mut match_count: Vec<c_int> = vec![0; cpt_sources().rows().len()];
    clear_adjusted_leader();

    let mut first: *mut c_char = ptr::null_mut();
    let mut len: c_int = -1;
    for mut compl in matches_from(first_match()) {
        // SAFETY: `compl` is a live node of the match list.
        let leader = unsafe { get_leader_for_startcol(compl, true) };

        // Apply 'smartcase' behavior during normal mode.
        if ctrl_x_mode_normal()
            && p_inf.get() == 0
            && !leader.data().is_null()
            // SAFETY: the leader is a NUL-terminated string.
            && unsafe { ignorecase(leader.data()) } == 0
        {
            compl.cp_flags &= !CP_ICASE;
        }

        // SAFETY: the leader is readable for its own length.
        let displayed = !compl.is_original()
            && (leader.data().is_null()
                || unsafe { ins_compl_equal(compl, leader.data(), leader.len()) });
        if displayed {
            // Limit the number of items from each source if max_items is set.
            let mut match_limit_exceeded = false;
            let cur_source = compl.cp_cpt_source_idx;
            if cur_source != -1 {
                match_count[cur_source as usize] += 1;
                let max_matches = cpt_sources().row(cur_source).cs_max_matches;
                if max_matches > 0 && match_count[cur_source as usize] > max_matches {
                    match_limit_exceeded = true;
                }
            }

            let from_curbuf =
                cur_source != -1 && cpt_sources().row(cur_source).cs_flag as c_int == '.' as c_int;
            if !match_limit_exceeded && (!curbuf_only || from_curbuf) {
                let text = compl.cp_str.data();
                // SAFETY: the leader is readable for its own length and a
                // match's text is NUL-terminated.
                let starts_with_leader =
                    unsafe { cstr::prefix_eq(ins_compl_leader(), text, ins_compl_leader_len()) };
                if first.is_null() && starts_with_leader {
                    first = text;
                    // SAFETY: as above.
                    len = unsafe { cstr::bytes_at(first) }.len() as c_int;
                } else if !first.is_null() {
                    // Shorten the prefix to what this match still agrees on.
                    let mut j: c_int = 0; // count in bytes
                    let mut s1 = first;
                    let mut s2 = text;
                    while j < len {
                        // SAFETY: both walk NUL-terminated strings and stop
                        // at the first NUL.
                        let (b1, b2) = unsafe { (*s1, *s2) };
                        if b1 as c_int == NUL || b2 as c_int == NUL {
                            break;
                        }
                        // SAFETY: as above -- `byte2len(b1)` bytes of a
                        // character that starts at both pointers.
                        let n = byte2len(b1) as size_t;
                        let differ = byte2len(b1) != byte2len(b2)
                            || unsafe { cstr::slice_at(s1, n) != cstr::slice_at(s2, n) };
                        if differ {
                            break;
                        }
                        j += byte2len(b1);
                        // SAFETY: as above -- the step lands on the next
                        // character or on the NUL.
                        s1 = unsafe { s1.offset(utfc_ptr2len(s1) as isize) };
                        // SAFETY: as above.
                        s2 = unsafe { s2.offset(utfc_ptr2len(s2) as isize) };
                    }
                    len = j;
                    if len == 0 {
                        break;
                    }
                }
            }
        }
    }

    if len <= ins_compl_leader_len() as c_int {
        return ptr::null_mut();
    }
    debug_assert!(!first.is_null());
    // Avoid inserting text that duplicates the text already after the cursor.
    if len == unsafe { cstr::bytes_at(first) }.len() as c_int {
        let p = unsafe { get_cursor_line_ptr().offset(cur_win().w_cursor.col as isize) };
        if !p.is_null() && !ascii_iswhite_or_nul(unsafe { *p } as c_int) {
            // SAFETY: `find_word_end` answers a pointer into the same line.
            let text_len = unsafe { find_word_end(p).offset_from(p) } as c_int;
            if text_len > 0
                && text_len < len - ins_compl_leader_len() as c_int
                && unsafe {
                    cstr::prefix_at(first.offset((len - text_len) as isize), text_len as size_t)
                        == cstr::prefix_at(p, text_len as size_t)
                }
            {
                len -= text_len;
            }
        }
    }
    unsafe { *prefix_len = len as size_t };
    first
}

/// Look in the first `len` characters of `src` for search metacharacters.
///
/// When `dest` is not null they are copied there, quoting the metacharacters
/// with a backslash, and `dest` is NUL terminated. Answers the length `dest`
/// needs either way.
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

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
