//! Comment leader parsing functions.
//!
//! This module provides functions for parsing comment leaders from lines,
//! using the 'comments' option to identify comment prefixes.

use std::ffi::{c_char, c_int};

// =============================================================================
// Comment Flags (from option_vars.h)
// =============================================================================

/// Comment leader: start of three-piece comment.
pub const COM_START: c_char = b's' as c_char;

/// Comment leader: middle of three-piece comment.
pub const COM_MIDDLE: c_char = b'm' as c_char;

/// Comment leader: end of three-piece comment.
pub const COM_END: c_char = b'e' as c_char;

/// Comment leader: nest within comment.
pub const COM_NEST: c_char = b'n' as c_char;

/// Comment leader: blank after leader.
pub const COM_BLANK: c_char = b'b' as c_char;

/// Comment leader: don't use for "O" command.
pub const COM_NOBACK: c_char = b'O' as c_char;

/// Comment leader: first line only.
pub const COM_FIRST: c_char = b'f' as c_char;

/// Comment leader: left adjust.
pub const COM_LEFT: c_char = b'l' as c_char;

/// Comment leader: right adjust.
pub const COM_RIGHT: c_char = b'r' as c_char;

/// Comment leader: auto-end for comment.
pub const COM_AUTO_END: c_char = b'x' as c_char;

/// Maximum length of a comment leader string.
pub const COM_MAX_LEN: usize = 256;

// =============================================================================
// C Accessor Functions (extern declarations)
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Buffer option accessor
    fn nvim_curbuf_get_b_p_com() -> *mut c_char;

    // String functions
    fn nvim_change_copy_option_part(
        src: *mut *mut c_char,
        dest: *mut c_char,
        maxlen: c_int,
        sep: *const c_char,
    ) -> usize;
    fn nvim_vim_strchr(s: *const c_char, c: c_int) -> *mut c_char;
    fn nvim_change_ascii_iswhite(c: c_int) -> bool;
}

// =============================================================================
// Comment Leader Functions
// =============================================================================

/// Returns the length in bytes of the comment leader prefix.
///
/// If this string is not a comment then 0 is returned.
/// When "flags" is not NULL, it is set to point to the flags of the recognized
/// comment leader.
/// "backward" must be true for the "O" command.
/// If "include_space" is set, include trailing whitespace while calculating
/// the length.
fn get_leader_len_impl(
    line: *const c_char,
    flags: *mut *mut c_char,
    backward: bool,
    include_space: bool,
) -> c_int {
    // SAFETY: All operations are safe FFI calls
    unsafe {
        if line.is_null() {
            return 0;
        }

        let mut got_com = false;
        let mut part_buf = [0i8; COM_MAX_LEN];
        let mut middle_match_len = 0;
        let mut saved_flags: *mut c_char = std::ptr::null_mut();

        let mut result = 0i32;
        let mut i = 0i32;

        // leading white space is ignored
        while nvim_change_ascii_iswhite(*line.offset(i as isize) as c_int) {
            i += 1;
        }

        // Repeat to match several nested comment strings.
        while *line.offset(i as isize) != 0 {
            // scan through the 'comments' option for a match
            let mut found_one = false;
            let mut list = nvim_curbuf_get_b_p_com();

            while *list != 0 {
                // Get one option part into part_buf[].  Advance "list" to next one.
                if !got_com && !flags.is_null() {
                    *flags = list; // remember where flags started
                }
                let prev_list = list;
                nvim_change_copy_option_part(
                    &mut list,
                    part_buf.as_mut_ptr(),
                    COM_MAX_LEN as c_int,
                    c",".as_ptr(),
                );

                let string = nvim_vim_strchr(part_buf.as_ptr(), b':' as c_int);
                if string.is_null() {
                    // missing ':', ignore this part
                    continue;
                }
                *string = 0; // isolate flags from string
                let mut string = string.add(1);

                // If we found a middle match previously, use that match when this
                // is not a middle or end.
                if middle_match_len != 0
                    && nvim_vim_strchr(part_buf.as_ptr(), COM_MIDDLE as c_int).is_null()
                    && nvim_vim_strchr(part_buf.as_ptr(), COM_END as c_int).is_null()
                {
                    break;
                }

                // When we already found a nested comment, only accept further nested comments.
                if got_com && nvim_vim_strchr(part_buf.as_ptr(), COM_NEST as c_int).is_null() {
                    continue;
                }

                // When 'O' flag present and using "O" command skip this one.
                if backward && !nvim_vim_strchr(part_buf.as_ptr(), COM_NOBACK as c_int).is_null() {
                    continue;
                }

                // Line contents and string must match.
                // When string starts with white space, must have some white space.
                if nvim_change_ascii_iswhite(*string as c_int) {
                    if i == 0 || !nvim_change_ascii_iswhite(*line.offset((i - 1) as isize) as c_int)
                    {
                        continue; // missing white space
                    }
                    while nvim_change_ascii_iswhite(*string as c_int) {
                        string = string.add(1);
                    }
                }

                let mut j = 0;
                while *string.add(j) != 0 && *string.add(j) == *line.offset((i + j as i32) as isize)
                {
                    j += 1;
                }
                if *string.add(j) != 0 {
                    continue; // string doesn't match
                }

                // When 'b' flag used, there must be white space or end-of-line after the string.
                if !nvim_vim_strchr(part_buf.as_ptr(), COM_BLANK as c_int).is_null()
                    && !nvim_change_ascii_iswhite(*line.offset((i + j as i32) as isize) as c_int)
                    && *line.offset((i + j as i32) as isize) != 0
                {
                    continue;
                }

                // We have found a match.
                if !nvim_vim_strchr(part_buf.as_ptr(), COM_MIDDLE as c_int).is_null() {
                    if middle_match_len == 0 {
                        middle_match_len = j as i32;
                        saved_flags = prev_list;
                    }
                    continue;
                }

                if middle_match_len != 0 && (j as i32) > middle_match_len {
                    // Use this match instead of the middle match.
                    middle_match_len = 0;
                }

                if middle_match_len == 0 {
                    i += j as i32;
                }
                found_one = true;
                break;
            }

            if middle_match_len != 0 {
                // Use the previously found middle match.
                if !got_com && !flags.is_null() {
                    *flags = saved_flags;
                }
                i += middle_match_len;
                found_one = true;
            }

            // No match found, stop scanning.
            if !found_one {
                break;
            }

            result = i;

            // Include any trailing white space.
            while nvim_change_ascii_iswhite(*line.offset(i as isize) as c_int) {
                i += 1;
            }

            if include_space {
                result = i;
            }

            // If this comment doesn't nest, stop here.
            got_com = true;
            if nvim_vim_strchr(part_buf.as_ptr(), COM_NEST as c_int).is_null() {
                break;
            }
        }

        result
    }
}

/// FFI wrapper for `get_leader_len`.
///
/// Returns the length in bytes of the comment leader prefix.
#[export_name = "get_leader_len"]
pub extern "C" fn rs_get_leader_len(
    line: *const c_char,
    flags: *mut *mut c_char,
    backward: bool,
    include_space: bool,
) -> c_int {
    get_leader_len_impl(line, flags, backward, include_space)
}

/// Return the offset at which the last comment in line starts.
///
/// If there is no comment in the whole line, -1 is returned.
/// When "flags" is not null, it is set to point to the flags describing the
/// recognized comment leader.
fn get_last_leader_offset_impl(line: *const c_char, flags: *mut *mut c_char) -> c_int {
    // SAFETY: All operations are safe FFI calls
    unsafe {
        if line.is_null() {
            return -1;
        }

        let mut result = -1i32;
        let mut lower_check_bound = 0i32;
        let mut com_flags: *mut c_char = std::ptr::null_mut();
        let mut part_buf = [0i8; COM_MAX_LEN];

        // Get line length
        let line_len = libc::strlen(line) as i32;
        let mut i = line_len;

        // Repeat to match several nested comment strings.
        i -= 1;
        while i >= lower_check_bound {
            // scan through the 'comments' option for a match
            let mut found_one = false;
            let mut list = nvim_curbuf_get_b_p_com();

            while *list != 0 {
                let flags_save = list;

                nvim_change_copy_option_part(
                    &mut list,
                    part_buf.as_mut_ptr(),
                    COM_MAX_LEN as c_int,
                    c",".as_ptr(),
                );

                let mut string = nvim_vim_strchr(part_buf.as_ptr(), b':' as c_int);
                if string.is_null() {
                    continue;
                }
                *string = 0;
                string = string.add(1);
                let _com_leader = string;

                // Line contents and string must match.
                if nvim_change_ascii_iswhite(*string as c_int) {
                    if i == 0 || !nvim_change_ascii_iswhite(*line.offset((i - 1) as isize) as c_int)
                    {
                        continue;
                    }
                    while nvim_change_ascii_iswhite(*string as c_int) {
                        string = string.add(1);
                    }
                }

                let mut j = 0;
                while *string.add(j) != 0 && *string.add(j) == *line.offset((i + j as i32) as isize)
                {
                    j += 1;
                }
                if *string.add(j) != 0 {
                    continue;
                }

                // When 'b' flag used, there must be white space or end-of-line after the string.
                if !nvim_vim_strchr(part_buf.as_ptr(), COM_BLANK as c_int).is_null()
                    && !nvim_change_ascii_iswhite(*line.offset((i + j as i32) as isize) as c_int)
                    && *line.offset((i + j as i32) as isize) != 0
                {
                    continue;
                }

                // For a middlepart comment, only consider it to match if
                // everything before the current position in the line is whitespace.
                if !nvim_vim_strchr(part_buf.as_ptr(), COM_MIDDLE as c_int).is_null() {
                    let mut k = 0;
                    while k <= i && nvim_change_ascii_iswhite(*line.offset(k as isize) as c_int) {
                        k += 1;
                    }
                    if k < i {
                        continue;
                    }
                }

                // We have found a match.
                found_one = true;
                if !flags.is_null() {
                    *flags = flags_save;
                }
                com_flags = flags_save;
                break;
            }

            if found_one {
                let mut part_buf2 = [0i8; COM_MAX_LEN];
                result = i;

                // If this comment nests, continue searching.
                if !nvim_vim_strchr(part_buf.as_ptr(), COM_NEST as c_int).is_null() {
                    i -= 1;
                    continue;
                }

                lower_check_bound = i;

                // Let's verify whether the comment leader found is a substring
                // of other comment leaders.
                let mut com_leader = nvim_vim_strchr(part_buf.as_ptr(), b':' as c_int);
                if !com_leader.is_null() {
                    com_leader = com_leader.add(1);
                    while nvim_change_ascii_iswhite(*com_leader as c_int) {
                        com_leader = com_leader.add(1);
                    }
                    let len1 = libc::strlen(com_leader) as i32;

                    let mut list = nvim_curbuf_get_b_p_com();
                    while *list != 0 {
                        let flags_save = list;
                        nvim_change_copy_option_part(
                            &mut list,
                            part_buf2.as_mut_ptr(),
                            COM_MAX_LEN as c_int,
                            c",".as_ptr(),
                        );
                        if flags_save == com_flags {
                            continue;
                        }
                        let mut string = nvim_vim_strchr(part_buf2.as_ptr(), b':' as c_int);
                        if string.is_null() {
                            continue;
                        }
                        string = string.add(1);
                        while nvim_change_ascii_iswhite(*string as c_int) {
                            string = string.add(1);
                        }
                        let len2 = libc::strlen(string) as i32;
                        if len2 == 0 {
                            continue;
                        }

                        // Now we have to verify whether string ends with a substring
                        // beginning the com_leader.
                        let mut off = if len2 > i { i } else { len2 };
                        while off > 0 && off + len1 > len2 {
                            off -= 1;
                            if libc::strncmp(
                                string.add(off as usize),
                                com_leader,
                                (len2 - off) as usize,
                            ) == 0
                            {
                                let new_bound = i - off;
                                if new_bound < lower_check_bound {
                                    lower_check_bound = new_bound;
                                }
                            }
                        }
                    }
                }
            }
            i -= 1;
        }

        result
    }
}

/// FFI wrapper for `get_last_leader_offset`.
///
/// Return the offset at which the last comment in line starts.
/// If there is no comment in the whole line, -1 is returned.
#[export_name = "get_last_leader_offset"]
pub extern "C" fn rs_get_last_leader_offset(line: *const c_char, flags: *mut *mut c_char) -> c_int {
    get_last_leader_offset_impl(line, flags)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_constants() {
        assert_eq!(COM_START, b's' as c_char);
        assert_eq!(COM_MIDDLE, b'm' as c_char);
        assert_eq!(COM_END, b'e' as c_char);
        assert_eq!(COM_NEST, b'n' as c_char);
        assert_eq!(COM_BLANK, b'b' as c_char);
        assert_eq!(COM_MAX_LEN, 256);
    }
}
