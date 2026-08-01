//! Reading a file name out of the buffer's text.
//!
//! [`file_name_in_line`] is what `gf` and its neighbours use: it finds the
//! run of `'isfname'` characters around a column, allows the extra
//! characters a URL needs, drops trailing punctuation, and picks up a
//! trailing `" line 99"`. [`find_file_name_in_path`] then looks the name up
//! along `'path'`, applying `'includeexpr'` when asked to or when the plain
//! lookup failed.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn grab_file_name(
    mut count: ::core::ffi::c_int,
    mut file_lnum: *mut linenr_T,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut options: ::core::ffi::c_int = FNAME_MESS as ::core::ffi::c_int
            | FNAME_EXP as ::core::ffi::c_int
            | FNAME_REL as ::core::ffi::c_int
            | FNAME_UNESC as ::core::ffi::c_int;
        if VIsual_active.get() {
            let mut len: size_t = 0;
            let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            if get_visual_text(
                ::core::ptr::null_mut::<cmdarg_T>(),
                &raw mut ptr,
                &raw mut len,
            ) as ::core::ffi::c_int
                == FAIL
            {
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            if !file_lnum.is_null()
                && *ptr.offset(len as isize) as ::core::ffi::c_int == ':' as ::core::ffi::c_int
                && *(*__ctype_b_loc()).offset(*ptr.offset(len.wrapping_add(1 as size_t) as isize)
                    as uint8_t as ::core::ffi::c_int
                    as isize) as ::core::ffi::c_int
                    & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    != 0
            {
                let mut p: *mut ::core::ffi::c_char = ptr
                    .offset(len as isize)
                    .offset(1 as ::core::ffi::c_int as isize);
                *file_lnum = getdigits_int32(&raw mut p, false_0 != 0, 0 as int32_t) as linenr_T;
            }
            return find_file_name_in_path(
                ptr,
                len,
                options,
                count as ::core::ffi::c_long,
                (*curbuf.get()).b_ffname,
            );
        }
        return file_name_at_cursor(options | FNAME_HYP as ::core::ffi::c_int, count, file_lnum);
    }
}

pub unsafe extern "C" fn file_name_at_cursor(
    mut options: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut file_lnum: *mut linenr_T,
) -> *mut ::core::ffi::c_char {
    unsafe {
        return file_name_in_line(
            get_cursor_line_ptr(),
            (*curwin.get()).w_cursor.col as ::core::ffi::c_int,
            options,
            count,
            (*curbuf.get()).b_ffname,
            file_lnum,
        );
    }
}

pub unsafe extern "C" fn file_name_in_line(
    mut line: *mut ::core::ffi::c_char,
    mut col: ::core::ffi::c_int,
    mut options: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut rel_fname: *mut ::core::ffi::c_char,
    mut file_lnum: *mut linenr_T,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut ptr: *mut ::core::ffi::c_char = line.offset(col as isize);
        while *ptr as ::core::ffi::c_int != NUL
            && !vim_isfilec(*ptr as uint8_t as ::core::ffi::c_int)
        {
            ptr = ptr.offset(utfc_ptr2len(ptr) as isize);
        }
        if *ptr as ::core::ffi::c_int == NUL {
            if options & FNAME_MESS as ::core::ffi::c_int != 0 {
                emsg(gettext(
                    b"E446: No file name under cursor\0".as_ptr() as *const ::core::ffi::c_char
                ));
            }
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut len: size_t = 0;
        let mut in_type: bool = true_0 != 0;
        let mut is_url: bool = false_0 != 0;
        while ptr > line {
            len = utf_head_off(line, ptr.offset(-(1 as ::core::ffi::c_int as isize))) as size_t;
            if len > 0 as size_t {
                ptr = ptr.offset(-(len.wrapping_add(1 as size_t) as isize));
            } else {
                if !(vim_isfilec(
                    *ptr.offset(-1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                ) as ::core::ffi::c_int
                    != 0
                    || options & FNAME_HYP as ::core::ffi::c_int != 0
                        && path_is_url(ptr.offset(-(1 as ::core::ffi::c_int as isize))) != 0)
                {
                    break;
                }
                ptr = ptr.offset(-1);
            }
        }
        len = (if path_has_drive_letter(ptr, strlen(ptr)) as ::core::ffi::c_int != 0 {
            2 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as size_t;
        while vim_isfilec(*ptr.offset(len as isize) as uint8_t as ::core::ffi::c_int)
            as ::core::ffi::c_int
            != 0
            || *ptr.offset(len as isize) as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                && *ptr.offset(len.wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
                    == ' ' as ::core::ffi::c_int
            || options & FNAME_HYP as ::core::ffi::c_int != 0
                && path_is_url(ptr.offset(len as isize)) != 0
            || is_url as ::core::ffi::c_int != 0
                && !vim_strchr(
                    b":?&=\0".as_ptr() as *const ::core::ffi::c_char,
                    *ptr.offset(len as isize) as uint8_t as ::core::ffi::c_int,
                )
                .is_null()
        {
            if *ptr.offset(len as isize) as ::core::ffi::c_int >= 'A' as ::core::ffi::c_int
                && *ptr.offset(len as isize) as ::core::ffi::c_int <= 'Z' as ::core::ffi::c_int
                || *ptr.offset(len as isize) as ::core::ffi::c_int >= 'a' as ::core::ffi::c_int
                    && *ptr.offset(len as isize) as ::core::ffi::c_int <= 'z' as ::core::ffi::c_int
            {
                if in_type as ::core::ffi::c_int != 0
                    && path_is_url(
                        ptr.offset(len as isize)
                            .offset(1 as ::core::ffi::c_int as isize),
                    ) != 0
                {
                    is_url = true_0 != 0;
                }
            } else {
                in_type = false_0 != 0;
            }
            if *ptr.offset(len as isize) as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                && *ptr.offset(len.wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
                    == ' ' as ::core::ffi::c_int
            {
                len = len.wrapping_add(1);
            }
            len = len.wrapping_add(utfc_ptr2len(ptr.offset(len as isize)) as size_t);
        }
        if len > 2 as size_t
            && !vim_strchr(
                b".,:;!\0".as_ptr() as *const ::core::ffi::c_char,
                *ptr.offset(len.wrapping_sub(1 as size_t) as isize) as uint8_t
                    as ::core::ffi::c_int,
            )
            .is_null()
            && *ptr.offset(len.wrapping_sub(2 as size_t) as isize) as ::core::ffi::c_int
                != '.' as ::core::ffi::c_int
        {
            len = len.wrapping_sub(1);
        }
        if !file_lnum.is_null() {
            let mut match_text: *const ::core::ffi::c_char =
                b" line \0".as_ptr() as *const ::core::ffi::c_char;
            let mut match_textlen: size_t = 6 as size_t;
            let mut p: *mut ::core::ffi::c_char = ptr.offset(len as isize);
            if strncmp(p, match_text, match_textlen) == 0 as ::core::ffi::c_int {
                p = p.offset(match_textlen as isize);
            } else {
                match_text = gettext(&raw const line_msg as *const ::core::ffi::c_char);
                match_textlen = strlen(match_text);
                if strncmp(p, match_text, match_textlen) == 0 as ::core::ffi::c_int {
                    p = p.offset(match_textlen as isize);
                } else {
                    p = skipwhite(p);
                }
            }
            if *p as ::core::ffi::c_int != NUL {
                if *(*__ctype_b_loc()).offset(*p as uint8_t as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    == 0
                {
                    p = p.offset(1);
                }
                p = skipwhite(p);
                if *(*__ctype_b_loc()).offset(*p as uint8_t as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    != 0
                {
                    *file_lnum = getdigits_long(&raw mut p, false_0 != 0, 0 as ::core::ffi::c_long)
                        as linenr_T;
                }
            }
        }
        return find_file_name_in_path(ptr, len, options, count as ::core::ffi::c_long, rel_fname);
    }
}

pub(crate) unsafe extern "C" fn eval_includeexpr(
    ptr: *const ::core::ffi::c_char,
    len: size_t,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let save_sctx: sctx_T = current_sctx.get();
        set_vim_var_string(VV_FNAME, ptr, len as ptrdiff_t);
        current_sctx
            .set((*curbuf.get()).b_p_script_ctx[kBufOptIncludeexpr as ::core::ffi::c_int as usize]);
        let mut res: *mut ::core::ffi::c_char = eval_to_string_safe(
            (*curbuf.get()).b_p_inex,
            was_set_insecurely(
                curwin.get(),
                kOptIncludeexpr,
                OPT_LOCAL as ::core::ffi::c_int,
            ),
            true_0 != 0,
        );
        set_vim_var_string(
            VV_FNAME,
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as ptrdiff_t,
        );
        current_sctx.set(save_sctx);
        return res;
    }
}

pub unsafe extern "C" fn find_file_name_in_path(
    mut ptr: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut options: ::core::ffi::c_int,
    mut count: ::core::ffi::c_long,
    mut rel_fname: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut file_name: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if len == 0 as size_t {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if options & FNAME_HYP as ::core::ffi::c_int != 0
            && len > 6 as size_t
            && strncmp(
                ptr,
                b"file:/\0".as_ptr() as *const ::core::ffi::c_char,
                6 as size_t,
            ) == 0 as ::core::ffi::c_int
            && !vim_ispathsep(*ptr.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        {
            let mut off: size_t = (if path_has_drive_letter(
                ptr.offset(6 as ::core::ffi::c_int as isize),
                len.wrapping_sub(6 as size_t),
            ) as ::core::ffi::c_int
                != 0
            {
                6 as ::core::ffi::c_int
            } else {
                5 as ::core::ffi::c_int
            }) as size_t;
            ptr = ptr.offset(off as isize);
            len = len.wrapping_sub(off);
        }
        if options & FNAME_INCL as ::core::ffi::c_int != 0
            && *(*curbuf.get()).b_p_inex as ::core::ffi::c_int != NUL
        {
            tofree = eval_includeexpr(ptr, len);
            if !tofree.is_null() {
                ptr = tofree;
                len = strlen(ptr);
            }
        }
        if options & FNAME_EXP as ::core::ffi::c_int != 0 {
            let mut file_to_find: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut search_ctx: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            file_name = find_file_in_path(
                ptr,
                len,
                options & !(FNAME_MESS as ::core::ffi::c_int),
                true_0,
                rel_fname,
                &raw mut file_to_find,
                &raw mut search_ctx,
            );
            if file_name.is_null()
                && options & FNAME_INCL as ::core::ffi::c_int == 0
                && *(*curbuf.get()).b_p_inex as ::core::ffi::c_int != NUL
            {
                tofree = eval_includeexpr(ptr, len);
                if !tofree.is_null() {
                    ptr = tofree;
                    len = strlen(ptr);
                    file_name = find_file_in_path(
                        ptr,
                        len,
                        options & !(FNAME_MESS as ::core::ffi::c_int),
                        true_0,
                        rel_fname,
                        &raw mut file_to_find,
                        &raw mut search_ctx,
                    );
                }
            }
            if file_name.is_null() && options & FNAME_MESS as ::core::ffi::c_int != 0 {
                let mut c: ::core::ffi::c_char = *ptr.offset(len as isize);
                *ptr.offset(len as isize) = NUL as ::core::ffi::c_char;
                semsg(
                    gettext(b"E447: Can't find file \"%s\" in path\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    ptr,
                );
                *ptr.offset(len as isize) = c;
            }
            while !file_name.is_null() && {
                count -= 1;
                count > 0 as ::core::ffi::c_long
            } {
                xfree(file_name as *mut ::core::ffi::c_void);
                file_name = find_file_in_path(
                    ptr,
                    len,
                    options,
                    false_0,
                    rel_fname,
                    &raw mut file_to_find,
                    &raw mut search_ctx,
                );
            }
            xfree(file_to_find as *mut ::core::ffi::c_void);
            vim_findfile_cleanup(search_ctx as *mut ::core::ffi::c_void);
        } else {
            file_name = xstrnsave(ptr, len);
        }
        xfree(tofree as *mut ::core::ffi::c_void);
        return file_name;
    }
}
