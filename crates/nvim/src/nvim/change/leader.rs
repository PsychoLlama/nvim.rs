//! Measuring a 'comments' leader on a line.
//!
//! `get_leader_len` answers how many bytes at the start of a line are a comment
//! leader, and `get_last_leader_offset` where the *last* leader on a line
//! begins -- the one a trailing `//` comment starts at.  Both walk 'comments'
//! item by item and both write the item's flags through an out-parameter, which
//! callers read *after a failure* as well as after a match: see the note at
//! `get_leader_len`.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn get_leader_len(
    mut line: *mut ::core::ffi::c_char,
    mut flags: *mut *mut ::core::ffi::c_char,
    mut backward: bool,
    mut include_space: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut j: ::core::ffi::c_int = 0;
        let mut got_com: bool = false;
        let mut part_buf: [::core::ffi::c_char; 50] = [0; 50];
        let mut string: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut middle_match_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut saved_flags: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while ascii_iswhite(*line.offset(i as isize) as ::core::ffi::c_int) {
            i += 1;
        }
        while *line.offset(i as isize) as ::core::ffi::c_int != NUL {
            let mut found_one: bool = false;
            let mut list: *mut ::core::ffi::c_char = (*curbuf.get()).b_p_com;
            while *list != 0 {
                if !got_com && !flags.is_null() {
                    *flags = list;
                }
                let mut prev_list: *mut ::core::ffi::c_char = list;
                copy_option_part(
                    &raw mut list,
                    &raw mut part_buf as *mut ::core::ffi::c_char,
                    COM_MAX_LEN as size_t,
                    c",".as_ptr() as *mut ::core::ffi::c_char,
                );
                string = vim_strchr(
                    &raw mut part_buf as *mut ::core::ffi::c_char,
                    ':' as ::core::ffi::c_int,
                );
                if !string.is_null() {
                    let c2rust_fresh4 = string;
                    string = string.offset(1);
                    *c2rust_fresh4 = NUL as ::core::ffi::c_char;
                    if middle_match_len != 0 as ::core::ffi::c_int
                        && vim_strchr(&raw mut part_buf as *mut ::core::ffi::c_char, COM_MIDDLE)
                            .is_null()
                        && vim_strchr(&raw mut part_buf as *mut ::core::ffi::c_char, COM_END)
                            .is_null()
                    {
                        break;
                    }
                    if got_com as ::core::ffi::c_int != 0
                        && vim_strchr(&raw mut part_buf as *mut ::core::ffi::c_char, COM_NEST)
                            .is_null()
                    {
                        continue;
                    }
                    if backward as ::core::ffi::c_int != 0
                        && !vim_strchr(&raw mut part_buf as *mut ::core::ffi::c_char, COM_NOBACK)
                            .is_null()
                    {
                        continue;
                    }
                    if ascii_iswhite(
                        *string.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ) {
                        if i == 0 as ::core::ffi::c_int
                            || !ascii_iswhite(*line.offset((i - 1 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_int)
                        {
                            continue;
                        } else {
                            while ascii_iswhite(*string.offset(0 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int)
                            {
                                string = string.offset(1);
                            }
                        }
                    }
                    j = 0 as ::core::ffi::c_int;
                    while *string.offset(j as isize) as ::core::ffi::c_int != NUL
                        && *string.offset(j as isize) as ::core::ffi::c_int
                            == *line.offset((i + j) as isize) as ::core::ffi::c_int
                    {
                        j += 1;
                    }
                    if *string.offset(j as isize) as ::core::ffi::c_int != NUL {
                        continue;
                    }
                    if !vim_strchr(&raw mut part_buf as *mut ::core::ffi::c_char, COM_BLANK)
                        .is_null()
                        && !ascii_iswhite(*line.offset((i + j) as isize) as ::core::ffi::c_int)
                        && *line.offset((i + j) as isize) as ::core::ffi::c_int != NUL
                    {
                        continue;
                    }
                    if !vim_strchr(&raw mut part_buf as *mut ::core::ffi::c_char, COM_MIDDLE)
                        .is_null()
                    {
                        if middle_match_len == 0 as ::core::ffi::c_int {
                            middle_match_len = j;
                            saved_flags = prev_list;
                        }
                    } else {
                        if middle_match_len != 0 as ::core::ffi::c_int && j > middle_match_len {
                            middle_match_len = 0 as ::core::ffi::c_int;
                        }
                        if middle_match_len == 0 as ::core::ffi::c_int {
                            i += j;
                        }
                        found_one = true;
                        break;
                    }
                }
            }
            if middle_match_len != 0 as ::core::ffi::c_int {
                if !got_com && !flags.is_null() {
                    *flags = saved_flags;
                }
                i += middle_match_len;
                found_one = true;
            }
            if !found_one {
                break;
            }
            result = i;
            while ascii_iswhite(*line.offset(i as isize) as ::core::ffi::c_int) {
                i += 1;
            }
            if include_space {
                result = i;
            }
            got_com = true;
            if vim_strchr(&raw mut part_buf as *mut ::core::ffi::c_char, COM_NEST).is_null() {
                break;
            }
        }
        return result;
    }
}

pub unsafe extern "C" fn get_last_leader_offset(
    mut line: *mut ::core::ffi::c_char,
    mut flags: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut result: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut j: ::core::ffi::c_int = 0;
        let mut lower_check_bound: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut com_leader: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut com_flags: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut part_buf: [::core::ffi::c_char; 50] = [0; 50];
        let mut i: ::core::ffi::c_int = strlen(line) as ::core::ffi::c_int;
        loop {
            i -= 1;
            if i < lower_check_bound {
                break;
            }
            let mut found_one: bool = false;
            let mut list: *mut ::core::ffi::c_char = (*curbuf.get()).b_p_com;
            while *list != 0 {
                let mut flags_save: *mut ::core::ffi::c_char = list;
                copy_option_part(
                    &raw mut list,
                    &raw mut part_buf as *mut ::core::ffi::c_char,
                    COM_MAX_LEN as size_t,
                    c",".as_ptr() as *mut ::core::ffi::c_char,
                );
                let mut string: *mut ::core::ffi::c_char = vim_strchr(
                    &raw mut part_buf as *mut ::core::ffi::c_char,
                    ':' as ::core::ffi::c_int,
                );
                if string.is_null() {
                    continue;
                }
                let c2rust_fresh5 = string;
                string = string.offset(1);
                *c2rust_fresh5 = NUL as ::core::ffi::c_char;
                com_leader = string;
                if ascii_iswhite(
                    *string.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                ) {
                    if i == 0 as ::core::ffi::c_int
                        || !ascii_iswhite(*line.offset((i - 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int)
                    {
                        continue;
                    }
                    while ascii_iswhite(*string as ::core::ffi::c_int) {
                        string = string.offset(1);
                    }
                }
                j = 0 as ::core::ffi::c_int;
                while *string.offset(j as isize) as ::core::ffi::c_int != NUL
                    && *string.offset(j as isize) as ::core::ffi::c_int
                        == *line.offset((i + j) as isize) as ::core::ffi::c_int
                {
                    j += 1;
                }
                if *string.offset(j as isize) as ::core::ffi::c_int != NUL {
                    continue;
                }
                if !vim_strchr(&raw mut part_buf as *mut ::core::ffi::c_char, COM_BLANK).is_null()
                    && !ascii_iswhite(*line.offset((i + j) as isize) as ::core::ffi::c_int)
                    && *line.offset((i + j) as isize) as ::core::ffi::c_int != NUL
                {
                    continue;
                }
                if !vim_strchr(&raw mut part_buf as *mut ::core::ffi::c_char, COM_MIDDLE).is_null()
                {
                    j = 0 as ::core::ffi::c_int;
                    while j <= i
                        && ascii_iswhite(*line.offset(j as isize) as ::core::ffi::c_int)
                            as ::core::ffi::c_int
                            != 0
                    {
                        j += 1;
                    }
                    if j < i {
                        continue;
                    }
                }
                found_one = true;
                if !flags.is_null() {
                    *flags = flags_save;
                }
                com_flags = flags_save;
                break;
            }
            if !found_one {
                continue;
            }
            let mut part_buf2: [::core::ffi::c_char; 50] = [0; 50];
            result = i;
            if !vim_strchr(&raw mut part_buf as *mut ::core::ffi::c_char, COM_NEST).is_null() {
                continue;
            }
            lower_check_bound = i;
            while ascii_iswhite(*com_leader as ::core::ffi::c_int) {
                com_leader = com_leader.offset(1);
            }
            let mut len1: ::core::ffi::c_int = strlen(com_leader) as ::core::ffi::c_int;
            let mut list_0: *mut ::core::ffi::c_char = (*curbuf.get()).b_p_com;
            while *list_0 != 0 {
                let mut flags_save_0: *mut ::core::ffi::c_char = list_0;
                copy_option_part(
                    &raw mut list_0,
                    &raw mut part_buf2 as *mut ::core::ffi::c_char,
                    COM_MAX_LEN as size_t,
                    c",".as_ptr() as *mut ::core::ffi::c_char,
                );
                if flags_save_0 == com_flags {
                    continue;
                }
                let mut string_0: *mut ::core::ffi::c_char = vim_strchr(
                    &raw mut part_buf2 as *mut ::core::ffi::c_char,
                    ':' as ::core::ffi::c_int,
                );
                string_0 = string_0.offset(1);
                while ascii_iswhite(*string_0 as ::core::ffi::c_int) {
                    string_0 = string_0.offset(1);
                }
                let mut len2: ::core::ffi::c_int = strlen(string_0) as ::core::ffi::c_int;
                if len2 == 0 as ::core::ffi::c_int {
                    continue;
                }
                let mut off: ::core::ffi::c_int = if len2 > i { i } else { len2 };
                while off > 0 as ::core::ffi::c_int && off + len1 > len2 {
                    off -= 1;
                    if strncmp(
                        string_0.offset(off as isize),
                        com_leader,
                        (len2 - off) as size_t,
                    ) == 0
                    {
                        lower_check_bound = if lower_check_bound < i - off {
                            lower_check_bound
                        } else {
                            i - off
                        };
                    }
                }
            }
        }
        return result;
    }
}
