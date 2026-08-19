//! Reading a file name out of the buffer's text.
//!
//! [`file_name_in_line`] is what `gf` and its neighbours use: it finds the
//! run of `'isfname'` characters around a column, allows the extra
//! characters a URL needs, drops trailing punctuation, and picks up a
//! trailing `" line 99"`. [`find_file_name_in_path`] then looks the name up
//! along `'path'`, applying `'includeexpr'` when asked to or when the plain
//! lookup failed.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::semsg_c;
use crate::types::{FAIL, VV_FNAME};
use core::ffi::{c_char, c_int, c_long};
use core::ptr;
use std::ffi::CStr;

/// The file name at the cursor, or the Visual selection when there is one.
///
/// Returns the name in allocated memory, NULL for failure.
pub unsafe fn grab_file_name(count: c_int, file_lnum: *mut linenr_T) -> *mut c_char {
    unsafe {
        let options = (FNAME_MESS | FNAME_EXP | FNAME_REL | FNAME_UNESC) as c_int;
        if !VIsual_active.get() {
            return file_name_at_cursor(options | FNAME_HYP as c_int, count, file_lnum);
        }

        let mut len: size_t = 0;
        let mut ptr: *mut c_char = ptr::null_mut();
        if get_visual_text(ptr::null_mut::<cmdarg_T>(), &raw mut ptr, &raw mut len) as c_int == FAIL
        {
            return ptr::null_mut();
        }
        // Only recognize ":123" here.
        if !file_lnum.is_null()
            && *ptr.add(len) == b':' as c_char
            && (*ptr.add(len + 1) as u8).is_ascii_digit()
        {
            let mut p = ptr.add(len + 1);
            *file_lnum = getdigits_int32(&raw mut p, false, 0) as linenr_T;
        }
        find_file_name_in_path(ptr, len, options, count as c_long, (*curbuf.get()).b_ffname)
    }
}

/// The file name under or after the cursor.
///
/// `'path'` is searched when the name is not absolute. The string returned
/// has been allocated and should be freed by the caller; NULL is returned
/// if the file name or the file is not found.
///
/// options:
/// - `FNAME_MESS`  give error messages
/// - `FNAME_EXP`   expand to path
/// - `FNAME_HYP`   check for hypertext link
/// - `FNAME_INCL`  apply `'includeexpr'`
pub unsafe fn file_name_at_cursor(
    options: c_int,
    count: c_int,
    file_lnum: *mut linenr_T,
) -> *mut c_char {
    unsafe {
        file_name_in_line(
            get_cursor_line_ptr(),
            (*curwin.get()).w_cursor.col as c_int,
            options,
            count,
            (*curbuf.get()).b_ffname,
            file_lnum,
        )
    }
}

/// The start of the file name around `line[col]`, or NULL when the rest of
/// the line holds no `'isfname'` character at all.
///
/// Goes one character back to the `":"` before `"//"`, or to the drive letter
/// before `":\"`, even when `":"` is not in `'isfname'`.
unsafe fn name_start(line: *mut c_char, col: c_int, options: c_int) -> *mut c_char {
    unsafe {
        // Search forward for what could be the start of a file name.
        let mut ptr = line.offset(col as isize);
        while *ptr != 0 && !vim_isfilec(*ptr as u8 as c_int) {
            ptr = ptr.offset(utfc_ptr2len(ptr) as isize);
        }
        if *ptr == 0 {
            return ptr::null_mut();
        }

        // Search backward for the first character of the file name.
        while ptr > line {
            let head_off = utf_head_off(line, ptr.sub(1)) as usize;
            if head_off > 0 {
                ptr = ptr.sub(head_off + 1);
            } else if vim_isfilec(*ptr.sub(1) as u8 as c_int)
                || (options & FNAME_HYP as c_int != 0 && path_is_url(ptr.sub(1)) != 0)
            {
                ptr = ptr.sub(1);
            } else {
                break;
            }
        }
        ptr
    }
}

/// How many bytes of `ptr` belong to the file name that starts there.
///
/// `":"`, `"?"`, `"&"` and `"="` join the name once a `type://` prefix has
/// been seen, so that `http://google.com:8080?q=this&that=ok` comes out
/// whole. `"\ "` is an escaped space and counts as two.
unsafe fn name_length(ptr: *const c_char, options: c_int) -> usize {
    unsafe {
        let hyp = options & FNAME_HYP as c_int != 0;
        // TODO(justinmk): Check for driveletter "x:/" at start, regardless of
        // 'isfname'.
        let mut len = if path_has_drive_letter(ptr, strlen(ptr)) {
            2
        } else {
            0
        };
        let mut in_type = true;
        let mut is_url = false;
        loop {
            let at = |i: usize| *ptr.add(i) as u8;
            let escaped_space = at(len) == b'\\' && at(len + 1) == b' ';
            if !(vim_isfilec(at(len) as c_int)
                || escaped_space
                || (hyp && path_is_url(ptr.add(len)) != 0)
                || (is_url && !vim_strchr(c":?&=".as_ptr(), at(len) as c_int).is_null()))
            {
                break;
            }
            if at(len).is_ascii_alphabetic() {
                if in_type && path_is_url(ptr.add(len + 1)) != 0 {
                    is_url = true;
                }
            } else {
                in_type = false;
            }
            if escaped_space {
                len += 1; // skip over the "\" in "\ "
            }
            len += utfc_ptr2len(ptr.add(len)) as usize;
        }

        // If there is trailing punctuation, remove it. But don't remove "..",
        // which could be a directory name.
        if len > 2
            && !vim_strchr(c".,:;!".as_ptr(), *ptr.add(len - 1) as u8 as c_int).is_null()
            && *ptr.add(len - 2) != b'.' as c_char
        {
            len -= 1;
        }
        len
    }
}

/// The line number written after a file name, as `" line 99"` or after any
/// single separator character. Both the English spelling and the translated
/// one are accepted, as `last_set_msg()` writes the latter.
unsafe fn trailing_line_number(after_name: *const c_char) -> Option<c_long> {
    unsafe {
        let english = c" line ";
        let localized = CStr::from_ptr(gettext(line_msg.ptr().cast::<c_char>()));

        let mut p = after_name.cast_mut();
        if strncmp(p, english.as_ptr(), english.count_bytes()) == 0 {
            p = p.add(english.count_bytes());
        } else if strncmp(p, localized.as_ptr(), localized.count_bytes()) == 0 {
            p = p.add(localized.count_bytes());
        } else {
            p = skipwhite(p);
        }

        if *p == 0 {
            return None;
        }
        if !(*p as u8).is_ascii_digit() {
            p = p.add(1); // skip the separator
        }
        p = skipwhite(p);
        (*p as u8)
            .is_ascii_digit()
            .then(|| getdigits_long(&raw mut p, false, 0))
    }
}

/// The name of the file under or after `line[col]`, looked up in `'path'`.
///
/// @param rel_fname  file we are searching relative to
/// @param file_lnum  line number after the file name
///
/// Otherwise like [`file_name_at_cursor`].
pub unsafe fn file_name_in_line(
    line: *mut c_char,
    col: c_int,
    options: c_int,
    count: c_int,
    rel_fname: *mut c_char,
    file_lnum: *mut linenr_T,
) -> *mut c_char {
    unsafe {
        let ptr = name_start(line, col, options);
        if ptr.is_null() {
            if options & FNAME_MESS as c_int != 0 {
                emsg(gettext(c"E446: No file name under cursor".as_ptr()));
            }
            return ptr::null_mut();
        }

        let len = name_length(ptr, options);
        if !file_lnum.is_null()
            && let Some(lnum) = trailing_line_number(ptr.add(len))
        {
            *file_lnum = lnum as linenr_T;
        }

        find_file_name_in_path(ptr, len, options, count as c_long, rel_fname)
    }
}

/// Run `'includeexpr'` over `ptr[len]`, with the name in `v:fname`.
pub(crate) unsafe fn eval_includeexpr(ptr: *const c_char, len: size_t) -> *mut c_char {
    unsafe {
        let save_sctx = current_sctx.get();
        set_vim_var_string(VV_FNAME, ptr, len as ptrdiff_t);
        current_sctx.set((*curbuf.get()).b_p_script_ctx[kBufOptIncludeexpr as usize]);

        let res = eval_to_string_safe(
            (*curbuf.get()).b_p_inex,
            was_set_insecurely(curwin.get(), kOptIncludeexpr, OPT_LOCAL as c_int),
            true,
        );

        set_vim_var_string(VV_FNAME, ptr::null(), 0);
        current_sctx.set(save_sctx);
        res
    }
}

/// The name of the file `ptr[len]` in `'path'`.
///
/// Otherwise like [`file_name_at_cursor`].
///
/// @param rel_fname  file we are searching relative to
pub unsafe fn find_file_name_in_path(
    ptr: *mut c_char,
    len: size_t,
    options: c_int,
    count: c_long,
    rel_fname: *mut c_char,
) -> *mut c_char {
    unsafe {
        let mut ptr = ptr;
        let mut len = len;
        let mut count = count;
        if len == 0 {
            return ptr::null_mut();
        }

        // "file:/name" and "file://name" both name "/name"; a drive letter
        // after "file:/" keeps the slash.
        if options & FNAME_HYP as c_int != 0
            && len > 6
            && strncmp(ptr, c"file:/".as_ptr(), 6) == 0
            && !vim_ispathsep(*ptr.add(6) as c_int)
        {
            let off = if path_has_drive_letter(ptr.add(6), len - 6) {
                6
            } else {
                5
            };
            ptr = ptr.add(off);
            len -= off;
        }

        let mut tofree: *mut c_char = ptr::null_mut();
        if options & FNAME_INCL as c_int != 0 && *(*curbuf.get()).b_p_inex != 0 {
            tofree = eval_includeexpr(ptr, len);
            if !tofree.is_null() {
                ptr = tofree;
                len = strlen(ptr);
            }
        }

        let mut file_name: *mut c_char = ptr::null_mut();
        if options & FNAME_EXP as c_int != 0 {
            let mut file_to_find: *mut c_char = ptr::null_mut();
            let mut search_ctx: *mut c_char = ptr::null_mut();
            let quiet = options & !(FNAME_MESS as c_int);
            let mut look = |ptr, len, first| {
                find_file_in_path(
                    ptr,
                    len,
                    quiet,
                    first,
                    rel_fname,
                    &raw mut file_to_find,
                    &raw mut search_ctx,
                )
            };
            file_name = look(ptr, len, true);

            // If the file could not be found in a normal way, try applying
            // 'includeexpr' (unless done already).
            if file_name.is_null()
                && options & FNAME_INCL as c_int == 0
                && *(*curbuf.get()).b_p_inex != 0
            {
                tofree = eval_includeexpr(ptr, len);
                if !tofree.is_null() {
                    ptr = tofree;
                    len = strlen(ptr);
                    file_name = look(ptr, len, true);
                }
            }
            if file_name.is_null() && options & FNAME_MESS as c_int != 0 {
                let c = *ptr.add(len);
                *ptr.add(len) = 0;
                semsg_c!(
                    gettext(c"E447: Can't find file \"%s\" in path".as_ptr()),
                    ptr,
                );
                *ptr.add(len) = c;
            }

            // Repeat finding the file "count" times. This matters when it
            // appears several times in the path.
            //
            // Note the repeats pass `options` unmasked, so FNAME_MESS reaches
            // find_file_in_path and its "No more file" message. Upstream.
            while !file_name.is_null() && {
                count -= 1;
                count > 0
            } {
                xfree(file_name.cast());
                file_name = find_file_in_path(
                    ptr,
                    len,
                    options,
                    false,
                    rel_fname,
                    &raw mut file_to_find,
                    &raw mut search_ctx,
                );
            }

            xfree(file_to_find.cast());
            vim_findfile_cleanup(search_ctx.cast());
        } else {
            file_name = xstrnsave(ptr, len);
        }

        xfree(tofree.cast());
        file_name
    }
}
