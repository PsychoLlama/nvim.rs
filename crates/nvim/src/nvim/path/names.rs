//! Taking a file name apart, and putting one together.
//!
//! Everything here is about the *text* of a name: which byte the last
//! component starts at ([`path_tail`] and its neighbours), whether a byte is
//! a path separator, how to join a directory to a name ([`concat_fnames`]),
//! and whether a name is really a URL ([`path_with_url`]) or an absolute
//! path. Nothing here touches the file system — except
//! [`dir_of_file_exists`], which is here because it is a question about a
//! name's directory part.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_tail(
    mut fname: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        if fname.is_null() {
            return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        let mut tail: *const ::core::ffi::c_char = get_past_head(fname);
        let mut p: *const ::core::ffi::c_char = tail;
        while *p as ::core::ffi::c_int != NUL {
            if vim_ispathsep_nocolon(*p as ::core::ffi::c_int) {
                tail = p.offset(1 as ::core::ffi::c_int as isize);
            }
            p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
        }
        return tail as *mut ::core::ffi::c_char;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_tail_with_sep(
    mut fname: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut past_head: *mut ::core::ffi::c_char = get_past_head(fname);
        let mut tail: *mut ::core::ffi::c_char = path_tail(fname);
        while tail > past_head && after_pathsep(fname, tail) != 0 {
            tail = tail.offset(-1);
        }
        return tail;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn invocation_path_tail(
    mut invocation: *const ::core::ffi::c_char,
    mut len: *mut size_t,
) -> *const ::core::ffi::c_char {
    unsafe {
        let mut tail: *const ::core::ffi::c_char = get_past_head(invocation);
        let mut p: *const ::core::ffi::c_char = tail;
        while *p as ::core::ffi::c_int != NUL
            && *p as ::core::ffi::c_int != ' ' as ::core::ffi::c_int
        {
            let mut was_sep: bool = vim_ispathsep_nocolon(*p as ::core::ffi::c_int);
            p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
            if was_sep {
                tail = p;
            }
        }
        if !len.is_null() {
            *len = p.offset_from(tail) as size_t;
        }
        return tail;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_next_component(
    mut fname: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    unsafe {
        while *fname as ::core::ffi::c_int != NUL && !vim_ispathsep(*fname as ::core::ffi::c_int) {
            fname = fname.offset(utfc_ptr2len(fname as *mut ::core::ffi::c_char) as isize);
        }
        if *fname as ::core::ffi::c_int != NUL {
            fname = fname.offset(1);
        }
        return fname;
    }
}

pub unsafe extern "C" fn path_head_length() -> ::core::ffi::c_int {
    return 1 as ::core::ffi::c_int;
}

pub unsafe extern "C" fn is_path_head(mut path: *const ::core::ffi::c_char) -> bool {
    unsafe {
        return vim_ispathsep(*path as ::core::ffi::c_int);
    }
}

pub unsafe extern "C" fn get_past_head(
    mut path: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut retval: *const ::core::ffi::c_char = path;
        while vim_ispathsep(*retval as ::core::ffi::c_int) {
            retval = retval.offset(1);
        }
        return retval as *mut ::core::ffi::c_char;
    }
}

pub unsafe extern "C" fn vim_ispathsep(mut c: ::core::ffi::c_int) -> bool {
    return c == '/' as ::core::ffi::c_int;
}

pub unsafe extern "C" fn vim_ispathsep_nocolon(mut c: ::core::ffi::c_int) -> bool {
    unsafe {
        return vim_ispathsep(c);
    }
}

pub unsafe extern "C" fn vim_ispathlistsep(mut c: ::core::ffi::c_int) -> bool {
    return c == ':' as ::core::ffi::c_int;
}

pub unsafe extern "C" fn dir_of_file_exists(mut fname: *mut ::core::ffi::c_char) -> bool {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = path_tail_with_sep(fname);
        if p == fname {
            return true_0 != 0;
        }
        let mut c: ::core::ffi::c_char = *p;
        *p = NUL as ::core::ffi::c_char;
        let mut retval: bool = os_isdir(fname);
        *p = c;
        return retval;
    }
}

pub unsafe extern "C" fn path_fnamecmp(
    mut fname1: *const ::core::ffi::c_char,
    mut fname2: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        return mb_strcmp_ic(p_fic.get() != 0, fname1, fname2);
    }
}

pub unsafe extern "C" fn path_fnamencmp(
    fname1: *const ::core::ffi::c_char,
    fname2: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    unsafe {
        if p_fic.get() != 0 {
            return mb_strnicmp(fname1, fname2, len);
        }
        return strncmp(fname1, fname2, len);
    }
}

#[inline]
pub(crate) unsafe extern "C" fn do_concat_fnames(
    mut fname1: *mut ::core::ffi::c_char,
    len1: size_t,
    mut fname2: *const ::core::ffi::c_char,
    len2: size_t,
    sep: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        if sep as ::core::ffi::c_int != 0
            && *fname1 as ::core::ffi::c_int != 0
            && after_pathsep(fname1, fname1.offset(len1 as isize)) == 0
        {
            *fname1.offset(len1 as isize) = PATHSEP as ::core::ffi::c_char;
            memmove(
                fname1
                    .offset(len1 as isize)
                    .offset(1 as ::core::ffi::c_int as isize)
                    as *mut ::core::ffi::c_void,
                fname2 as *const ::core::ffi::c_void,
                len2.wrapping_add(1 as size_t),
            );
        } else {
            memmove(
                fname1.offset(len1 as isize) as *mut ::core::ffi::c_void,
                fname2 as *const ::core::ffi::c_void,
                len2.wrapping_add(1 as size_t),
            );
        }
        return fname1;
    }
}

pub unsafe extern "C" fn concat_fnames(
    mut fname1: *const ::core::ffi::c_char,
    mut fname2: *const ::core::ffi::c_char,
    mut sep: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let len1: size_t = strlen(fname1);
        let len2: size_t = strlen(fname2);
        let mut dest: *mut ::core::ffi::c_char =
            xmalloc(len1.wrapping_add(len2).wrapping_add(3 as size_t)) as *mut ::core::ffi::c_char;
        memmove(
            dest as *mut ::core::ffi::c_void,
            fname1 as *const ::core::ffi::c_void,
            len1.wrapping_add(1 as size_t),
        );
        return do_concat_fnames(dest, len1, fname2, len2, sep);
    }
}

pub unsafe extern "C" fn concat_fnames_realloc(
    mut fname1: *mut ::core::ffi::c_char,
    mut fname2: *const ::core::ffi::c_char,
    mut sep: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let len1: size_t = strlen(fname1);
        let len2: size_t = strlen(fname2);
        return do_concat_fnames(
            xrealloc(
                fname1 as *mut ::core::ffi::c_void,
                len1.wrapping_add(len2).wrapping_add(3 as size_t),
            ) as *mut ::core::ffi::c_char,
            len1,
            fname2,
            len2,
            sep,
        );
    }
}

pub unsafe extern "C" fn add_pathsep(mut p: *mut ::core::ffi::c_char) -> bool {
    unsafe {
        let len: size_t = strlen(p);
        if *p as ::core::ffi::c_int != NUL && after_pathsep(p, p.offset(len as isize)) == 0 {
            let pathsep_len: size_t = ::core::mem::size_of::<[::core::ffi::c_char; 2]>();
            if len > (MAXPATHL as size_t).wrapping_sub(pathsep_len) {
                return false_0 != 0;
            }
            memcpy(
                p.offset(len as isize) as *mut ::core::ffi::c_void,
                PATHSEPSTR.as_ptr() as *const ::core::ffi::c_void,
                pathsep_len,
            );
        }
        return true_0 != 0;
    }
}

pub unsafe extern "C" fn path_has_drive_letter(
    mut p: *const ::core::ffi::c_char,
    mut path_len: size_t,
) -> bool {
    unsafe {
        return path_len >= 2 as size_t
            && (*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                >= 'A' as ::core::ffi::c_uint
                && *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                    <= 'Z' as ::core::ffi::c_uint
                || *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                    >= 'a' as ::core::ffi::c_uint
                    && *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                        <= 'z' as ::core::ffi::c_uint)
            && (*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == ':' as ::core::ffi::c_int
                || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '|' as ::core::ffi::c_int)
            && (path_len == 2 as size_t
                || (*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '/' as ::core::ffi::c_int) as ::core::ffi::c_int
                    | (*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\\' as ::core::ffi::c_int)
                        as ::core::ffi::c_int
                    | (*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '?' as ::core::ffi::c_int) as ::core::ffi::c_int
                    | (*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '#' as ::core::ffi::c_int) as ::core::ffi::c_int
                    != 0);
    }
}

pub unsafe extern "C" fn path_is_url(mut p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        if strncmp(
            p,
            b":/\0".as_ptr() as *const ::core::ffi::c_char,
            2 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            return URL_SLASH as ::core::ffi::c_int;
        } else if strncmp(
            p,
            b":\\\\\0".as_ptr() as *const ::core::ffi::c_char,
            3 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            return URL_BACKSLASH as ::core::ffi::c_int;
        }
        return 0 as ::core::ffi::c_int;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_with_url(
    mut fname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        if !(*fname as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && *fname as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || *fname as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && *fname as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint)
        {
            return 0 as ::core::ffi::c_int;
        }
        if path_has_drive_letter(fname, strlen(fname)) {
            return 0 as ::core::ffi::c_int;
        }
        p = fname.offset(1 as ::core::ffi::c_int as isize);
        while *p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
            || ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
            || *p as ::core::ffi::c_int == '+' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == '.' as ::core::ffi::c_int
        {
            p = p.offset(1);
        }
        if *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '+' as ::core::ffi::c_int
            || *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '-' as ::core::ffi::c_int
            || *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
        {
            return 0 as ::core::ffi::c_int;
        }
        return path_is_url(p);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_with_extension(
    mut path: *const ::core::ffi::c_char,
    mut extension: *const ::core::ffi::c_char,
) -> bool {
    unsafe {
        let mut last_dot: *const ::core::ffi::c_char = strrchr(path, '.' as ::core::ffi::c_int);
        if last_dot.is_null() {
            return false_0 != 0;
        }
        return mb_strcmp_ic(
            p_fic.get() != 0,
            last_dot.offset(1 as ::core::ffi::c_int as isize),
            extension,
        ) == 0 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn vim_isAbsName(mut name: *const ::core::ffi::c_char) -> bool {
    unsafe {
        return path_with_url(name) != 0 as ::core::ffi::c_int
            || path_is_absolute(name) as ::core::ffi::c_int != 0;
    }
}

pub unsafe extern "C" fn after_pathsep(
    mut b: *const ::core::ffi::c_char,
    mut p: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        return (p > b
            && vim_ispathsep(*p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
            && utf_head_off(b, p.offset(-(1 as ::core::ffi::c_int as isize)))
                == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_is_absolute(mut fname: *const ::core::ffi::c_char) -> bool {
    unsafe {
        return *fname as ::core::ffi::c_int == '/' as ::core::ffi::c_int
            || *fname as ::core::ffi::c_int == '~' as ::core::ffi::c_int;
    }
}
