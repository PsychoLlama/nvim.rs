//! Word, line and case handling over the text a match is built from.
//!
//! [`ins_compl_add_infercase`] is `'infercase'`: it re-cases a match to match
//! what the user typed.  [`find_common_prefix`] computes the longest common
//! prefix `'longest'` inserts, and the `find_word_*` / `find_line_end`
//! helpers are the scans every buffer source walks with.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn ins_compl_infercase_gettext(
    mut str: *const ::core::ffi::c_char,
    mut char_len: ::core::ffi::c_int,
    mut compl_char_len: ::core::ffi::c_int,
    mut min_len: ::core::ffi::c_int,
    mut tofree: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut has_lower: bool = false_0 != 0;
        let wca: *mut ::core::ffi::c_int = xmalloc(
            (char_len as size_t).wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>()),
        ) as *mut ::core::ffi::c_int;
        let mut p: *const ::core::ffi::c_char = str;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < char_len {
            *wca.offset(i as isize) = mb_ptr2char_adv(&raw mut p);
            i += 1;
        }
        let mut p_0: *const ::core::ffi::c_char = (*compl_orig_text.ptr()).data;
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < min_len {
            let c: ::core::ffi::c_int = mb_ptr2char_adv(&raw mut p_0);
            if mb_islower(c) {
                has_lower = true_0 != 0;
                if mb_isupper(*wca.offset(i_0 as isize)) {
                    i_0 = compl_char_len;
                    while i_0 < char_len {
                        *wca.offset(i_0 as isize) = mb_tolower(*wca.offset(i_0 as isize));
                        i_0 += 1;
                    }
                    break;
                }
            }
            i_0 += 1;
        }
        if !has_lower {
            let mut was_letter: bool = false_0 != 0;
            let mut p_1: *const ::core::ffi::c_char = (*compl_orig_text.ptr()).data;
            let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_1 < min_len {
                let c_0: ::core::ffi::c_int = mb_ptr2char_adv(&raw mut p_1);
                if was_letter as ::core::ffi::c_int != 0
                    && mb_isupper(c_0) as ::core::ffi::c_int != 0
                    && mb_islower(*wca.offset(i_1 as isize)) as ::core::ffi::c_int != 0
                {
                    i_1 = compl_char_len;
                    while i_1 < char_len {
                        *wca.offset(i_1 as isize) = mb_toupper(*wca.offset(i_1 as isize));
                        i_1 += 1;
                    }
                    break;
                } else {
                    was_letter = mb_islower(c_0) as ::core::ffi::c_int != 0
                        || mb_isupper(c_0) as ::core::ffi::c_int != 0;
                    i_1 += 1;
                }
            }
        }
        let mut p_2: *const ::core::ffi::c_char = (*compl_orig_text.ptr()).data;
        let mut i_2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_2 < min_len {
            let c_1: ::core::ffi::c_int = mb_ptr2char_adv(&raw mut p_2);
            if mb_islower(c_1) {
                *wca.offset(i_2 as isize) = mb_tolower(*wca.offset(i_2 as isize));
            } else if mb_isupper(c_1) {
                *wca.offset(i_2 as isize) = mb_toupper(*wca.offset(i_2 as isize));
            }
            i_2 += 1;
        }
        let mut gap: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        let mut p_3: *mut ::core::ffi::c_char = IObuff.ptr() as *mut ::core::ffi::c_char;
        let mut i_3: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        ga_init(
            &raw mut gap,
            1 as ::core::ffi::c_int,
            500 as ::core::ffi::c_int,
        );
        while i_3 < char_len {
            if !gap.ga_data.is_null() {
                ga_grow(&raw mut gap, 10 as ::core::ffi::c_int);
                '_c2rust_label: {
                    if !gap.ga_data.is_null() {
                    } else {
                        __assert_fail(
                        b"gap.ga_data != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/insexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        771 as ::core::ffi::c_uint,
                        b"char *ins_compl_infercase_gettext(const char *, int, int, int, char **)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                    }
                };
                p_3 = (gap.ga_data as *mut ::core::ffi::c_char).offset(gap.ga_len as isize);
                let c2rust_fresh0 = i_3;
                i_3 = i_3 + 1;
                gap.ga_len += utf_char2bytes(*wca.offset(c2rust_fresh0 as isize), p_3);
            } else if p_3.offset_from(IObuff.ptr() as *mut ::core::ffi::c_char) + 6 as isize
                >= IOSIZE as isize
            {
                ga_grow(&raw mut gap, IOSIZE);
                *p_3 = NUL as ::core::ffi::c_char;
                strcpy(
                    gap.ga_data as *mut ::core::ffi::c_char,
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                );
                gap.ga_len =
                    p_3.offset_from(IObuff.ptr() as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
            } else {
                let c2rust_fresh1 = i_3;
                i_3 = i_3 + 1;
                p_3 = p_3.offset(utf_char2bytes(*wca.offset(c2rust_fresh1 as isize), p_3) as isize);
            }
        }
        xfree(wca as *mut ::core::ffi::c_void);
        if !gap.ga_data.is_null() {
            *tofree = gap.ga_data as *mut ::core::ffi::c_char;
            return gap.ga_data as *mut ::core::ffi::c_char;
        }
        *p_3 = NUL as ::core::ffi::c_char;
        return IObuff.ptr() as *mut ::core::ffi::c_char;
    }
}

pub unsafe extern "C" fn ins_compl_add_infercase(
    mut str_arg: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut icase: bool,
    mut fname: *mut ::core::ffi::c_char,
    mut dir: Direction,
    mut cont_s_ipos: bool,
    mut score: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut str: *mut ::core::ffi::c_char = str_arg;
        let mut char_len: ::core::ffi::c_int = 0;
        let mut compl_char_len: ::core::ffi::c_int = 0;
        let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if p_ic.get() != 0 && (*curbuf.get()).b_p_inf != 0 && len > 0 as ::core::ffi::c_int {
            let mut p: *const ::core::ffi::c_char = str;
            char_len = 0 as ::core::ffi::c_int;
            while *p as ::core::ffi::c_int != NUL {
                p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
                char_len += 1;
            }
            let mut p_0: *const ::core::ffi::c_char = (*compl_orig_text.ptr()).data;
            compl_char_len = 0 as ::core::ffi::c_int;
            while *p_0 as ::core::ffi::c_int != NUL {
                p_0 = p_0.offset(utfc_ptr2len(p_0 as *mut ::core::ffi::c_char) as isize);
                compl_char_len += 1;
            }
            let mut min_len: ::core::ffi::c_int = if char_len < compl_char_len {
                char_len
            } else {
                compl_char_len
            };
            str = ins_compl_infercase_gettext(
                str,
                char_len,
                compl_char_len,
                min_len,
                &raw mut tofree,
            );
        }
        if cont_s_ipos {
            flags |= CP_CONT_S_IPOS;
        }
        if icase {
            flags |= CP_ICASE;
        }
        let mut res: ::core::ffi::c_int = ins_compl_add(
            str,
            len,
            fname,
            ::core::ptr::null::<*mut ::core::ffi::c_char>(),
            false_0 != 0,
            ::core::ptr::null_mut::<typval_T>(),
            dir,
            flags,
            false_0 != 0,
            ::core::ptr::null::<::core::ffi::c_int>(),
            score,
        );
        xfree(tofree as *mut ::core::ffi::c_void);
        return res;
    }
}

pub unsafe extern "C" fn find_word_start(
    mut ptr: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        while *ptr as ::core::ffi::c_int != NUL
            && *ptr as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
            && mb_get_class(ptr) <= 1 as ::core::ffi::c_int
        {
            ptr = ptr.offset(utfc_ptr2len(ptr) as isize);
        }
        return ptr;
    }
}

pub unsafe extern "C" fn find_word_end(
    mut ptr: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let start_class: ::core::ffi::c_int = mb_get_class(ptr);
        if start_class > 1 as ::core::ffi::c_int {
            while *ptr as ::core::ffi::c_int != NUL {
                ptr = ptr.offset(utfc_ptr2len(ptr) as isize);
                if mb_get_class(ptr) != start_class {
                    break;
                }
            }
        }
        return ptr;
    }
}

pub unsafe extern "C" fn find_line_end(
    mut ptr: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut s: *mut ::core::ffi::c_char = ptr.offset(strlen(ptr) as isize);
        while s > ptr
            && (*s.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == CAR
                || *s.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NL)
        {
            s = s.offset(-1);
        }
        return s;
    }
}

pub(crate) unsafe extern "C" fn get_next_bufname_token() {
    unsafe {
        let mut b: *mut buf_T = firstbuf.get();
        while !b.is_null() {
            if (*b).b_p_bl != 0 && !(*b).b_sfname.is_null() {
                let mut tail: *mut ::core::ffi::c_char = path_tail((*b).b_sfname);
                if strncmp(
                    tail,
                    (*compl_orig_text.ptr()).data,
                    (*compl_orig_text.ptr()).size,
                ) == 0 as ::core::ffi::c_int
                {
                    ins_compl_add(
                        tail,
                        strlen(tail) as ::core::ffi::c_int,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        ::core::ptr::null::<*mut ::core::ffi::c_char>(),
                        false_0 != 0,
                        ::core::ptr::null_mut::<typval_T>(),
                        kDirectionNotSet,
                        if p_ic.get() != 0 {
                            CP_ICASE
                        } else {
                            0 as ::core::ffi::c_int
                        },
                        false_0 != 0,
                        ::core::ptr::null::<::core::ffi::c_int>(),
                        FUZZY_SCORE_NONE,
                    );
                }
            }
            b = (*b).b_next;
        }
    }
}

pub(crate) unsafe extern "C" fn strip_caret_numbers_in_place(mut str: *mut ::core::ffi::c_char) {
    unsafe {
        let mut read: *mut ::core::ffi::c_char = str;
        let mut write: *mut ::core::ffi::c_char = str;
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if str.is_null() {
            return;
        }
        while *read != 0 {
            if *read as ::core::ffi::c_int == '^' as ::core::ffi::c_int {
                p = read.offset(1 as ::core::ffi::c_int as isize);
                while ascii_isdigit(*p as ::core::ffi::c_int) {
                    p = p.offset(1);
                }
                if (*p as ::core::ffi::c_int == ',' as ::core::ffi::c_int
                    || *p as ::core::ffi::c_int == '\0' as ::core::ffi::c_int)
                    && p != read.offset(1 as ::core::ffi::c_int as isize)
                {
                    read = p;
                } else {
                    let c2rust_fresh5 = read;
                    read = read.offset(1);
                    let c2rust_fresh6 = write;
                    write = write.offset(1);
                    *c2rust_fresh6 = *c2rust_fresh5;
                }
            } else {
                let c2rust_fresh7 = read;
                read = read.offset(1);
                let c2rust_fresh8 = write;
                write = write.offset(1);
                *c2rust_fresh8 = *c2rust_fresh7;
            }
        }
        *write = '\0' as ::core::ffi::c_char;
    }
}

pub(crate) unsafe extern "C" fn find_common_prefix(
    mut prefix_len: *mut size_t,
    mut curbuf_only: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut is_cpt_completion: bool = !(*cpt_sources_array.ptr()).is_null();
        if !is_cpt_completion {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut match_count: *mut ::core::ffi::c_int = xcalloc(
            cpt_sources_count.get() as size_t,
            ::core::mem::size_of::<::core::ffi::c_int>(),
        ) as *mut ::core::ffi::c_int;
        get_leader_for_startcol(::core::ptr::null_mut::<compl_T>(), true_0 != 0);
        let mut compl: *mut compl_T = compl_first_match.get();
        let mut first: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut len: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        loop {
            let mut leader: *mut String_0 = get_leader_for_startcol(compl, true_0 != 0);
            if ctrl_x_mode_normal() as ::core::ffi::c_int != 0
                && p_inf.get() == 0
                && !(*leader).data.is_null()
                && ignorecase((*leader).data) == 0
            {
                (*compl).cp_flags &= !(CP_ICASE);
            }
            if !match_at_original_text(compl)
                && ((*leader).data.is_null()
                    || ins_compl_equal(compl, (*leader).data, (*leader).size) as ::core::ffi::c_int
                        != 0)
            {
                let mut match_limit_exceeded: bool = false_0 != 0;
                let mut cur_source: ::core::ffi::c_int = (*compl).cp_cpt_source_idx;
                if cur_source != -1 as ::core::ffi::c_int {
                    *match_count.offset(cur_source as isize) += 1;
                    let mut max_matches: ::core::ffi::c_int =
                        (*(*cpt_sources_array.ptr()).offset(cur_source as isize)).cs_max_matches;
                    if max_matches > 0 as ::core::ffi::c_int
                        && *match_count.offset(cur_source as isize) > max_matches
                    {
                        match_limit_exceeded = true_0 != 0;
                    }
                }
                if !match_limit_exceeded
                    && (!curbuf_only
                        || cur_source != -1 as ::core::ffi::c_int
                            && (*(*cpt_sources_array.ptr()).offset(cur_source as isize)).cs_flag
                                as ::core::ffi::c_int
                                == '.' as ::core::ffi::c_int)
                {
                    if first.is_null()
                        && strncmp(
                            ins_compl_leader(),
                            (*compl).cp_str.data,
                            ins_compl_leader_len(),
                        ) == 0 as ::core::ffi::c_int
                    {
                        first = (*compl).cp_str.data;
                        len = strlen(first) as ::core::ffi::c_int;
                    } else if !first.is_null() {
                        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        let mut s1: *mut ::core::ffi::c_char = first;
                        let mut s2: *mut ::core::ffi::c_char = (*compl).cp_str.data;
                        while j < len
                            && *s1 as ::core::ffi::c_int != NUL
                            && *s2 as ::core::ffi::c_int != NUL
                        {
                            if (*utf8len_tab.ptr())[*s1 as uint8_t as usize] as ::core::ffi::c_int
                                != (*utf8len_tab.ptr())[*s2 as uint8_t as usize]
                                    as ::core::ffi::c_int
                                || memcmp(
                                    s1 as *const ::core::ffi::c_void,
                                    s2 as *const ::core::ffi::c_void,
                                    (*utf8len_tab.ptr())[*s1 as uint8_t as usize] as size_t,
                                ) != 0 as ::core::ffi::c_int
                            {
                                break;
                            }
                            j +=
                                (*utf8len_tab.ptr())[*s1 as uint8_t as usize] as ::core::ffi::c_int;
                            s1 = s1.offset(utfc_ptr2len(s1) as isize);
                            s2 = s2.offset(utfc_ptr2len(s2) as isize);
                        }
                        len = j;
                        if len == 0 as ::core::ffi::c_int {
                            break;
                        }
                    }
                }
            }
            compl = (*compl).cp_next;
            if !(!compl.is_null() && !is_first_match(compl)) {
                break;
            }
        }
        xfree(match_count as *mut ::core::ffi::c_void);
        if len > ins_compl_leader_len() as ::core::ffi::c_int {
            '_c2rust_label: {
                if !first.is_null() {
                } else {
                    __assert_fail(
                        b"first != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/insexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        5085 as ::core::ffi::c_uint,
                        b"char *find_common_prefix(size_t *, _Bool)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            if len == strlen(first) as ::core::ffi::c_int {
                let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
                let mut p: *mut ::core::ffi::c_char =
                    line.offset((*curwin.get()).w_cursor.col as isize);
                if !p.is_null() && !ascii_iswhite_or_nul(*p as ::core::ffi::c_int) {
                    let mut end: *mut ::core::ffi::c_char = find_word_end(p);
                    let mut text_len: ::core::ffi::c_int = end.offset_from(p) as ::core::ffi::c_int;
                    if text_len > 0 as ::core::ffi::c_int
                        && text_len < len - ins_compl_leader_len() as ::core::ffi::c_int
                        && strncmp(
                            first.offset(len as isize).offset(-(text_len as isize)),
                            p,
                            text_len as size_t,
                        ) == 0 as ::core::ffi::c_int
                    {
                        len -= text_len;
                    }
                }
            }
            *prefix_len = len as size_t;
            return first;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}

pub(crate) unsafe extern "C" fn quote_meta(
    mut dest: *mut ::core::ffi::c_char,
    mut src: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) -> ::core::ffi::c_uint {
    unsafe {
        let mut m: ::core::ffi::c_uint =
            (len as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint);
        loop {
            len -= 1;
            if len < 0 as ::core::ffi::c_int {
                break;
            }
            's_85: {
                'c_56947: {
                    'c_56925: {
                        match *src as ::core::ffi::c_int {
                            46 | 42 | 91 => {
                                if ctrl_x_mode_dictionary() as ::core::ffi::c_int != 0
                                    || ctrl_x_mode_thesaurus() as ::core::ffi::c_int != 0
                                {
                                    break 's_85;
                                }
                            }
                            126 => {}
                            92 => {
                                break 'c_56925;
                            }
                            94 | 36 => {
                                break 'c_56947;
                            }
                            _ => {
                                break 's_85;
                            }
                        }
                        if !magic_isset() {
                            break 's_85;
                        }
                    }
                    if ctrl_x_mode_dictionary() as ::core::ffi::c_int != 0
                        || ctrl_x_mode_thesaurus() as ::core::ffi::c_int != 0
                    {
                        break 's_85;
                    }
                }
                m = m.wrapping_add(1);
                if !dest.is_null() {
                    let c2rust_fresh9 = dest;
                    dest = dest.offset(1);
                    *c2rust_fresh9 = '\\' as ::core::ffi::c_char;
                }
            }
            if !dest.is_null() {
                let c2rust_fresh10 = dest;
                dest = dest.offset(1);
                *c2rust_fresh10 = *src;
            }
            let mb_len: ::core::ffi::c_int = utfc_ptr2len(src) - 1 as ::core::ffi::c_int;
            if mb_len > 0 as ::core::ffi::c_int && len >= mb_len {
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < mb_len {
                    len -= 1;
                    src = src.offset(1);
                    if !dest.is_null() {
                        let c2rust_fresh11 = dest;
                        dest = dest.offset(1);
                        *c2rust_fresh11 = *src;
                    }
                    i += 1;
                }
            }
            src = src.offset(1);
        }
        if !dest.is_null() {
            *dest = NUL as ::core::ffi::c_char;
        }
        return m;
    }
}
