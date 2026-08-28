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
use crate::guard::Script;
use crate::normal::visual_active;
use crate::semsg_c;
use crate::types::{FAIL, OptionSetFlags, Vv};
use core::ffi::{c_char, c_int, c_long};
use core::ptr;
use std::ffi::CStr;

/// The file name at the cursor, or the Visual selection when there is one.
///
/// Returns the name in allocated memory, NULL for failure.
pub(crate) unsafe fn grab_file_name(count: c_int, file_lnum: *mut linenr_T) -> *mut c_char {
    let options = FileNameOpts::MESS | FileNameOpts::EXP | FileNameOpts::REL | FileNameOpts::UNESC;
    if !visual_active() {
        return unsafe { file_name_at_cursor(options | FileNameOpts::HYP, count, file_lnum) };
    }

    let mut len: size_t = 0;
    let mut ptr: *mut c_char = ptr::null_mut();
    if unsafe { get_visual_text(ptr::null_mut::<cmdarg_T>(), &raw mut ptr, &raw mut len) } as c_int
        == FAIL
    {
        return ptr::null_mut();
    }
    // Only recognize ":123" here.
    if !file_lnum.is_null()
        && unsafe { *ptr.add(len) } == b':' as c_char
        && (unsafe { *ptr.add(len + 1) } as u8).is_ascii_digit()
    {
        let mut p = unsafe { ptr.add(len + 1) };
        unsafe { *file_lnum = getdigits_int32(&raw mut p, false, 0) as linenr_T };
    }
    unsafe { find_file_name_in_path(ptr, len, options, count as c_long, (*curbuf.get()).b_ffname) }
}

/// The file name under or after the cursor.
///
/// `'path'` is searched when the name is not absolute. The string returned
/// has been allocated and should be freed by the caller; NULL is returned
/// if the file name or the file is not found.
///
/// options:
/// - `FileNameOpts::MESS`  give error messages
/// - `FileNameOpts::EXP`   expand to path
/// - `FileNameOpts::HYP`   check for hypertext link
/// - `FileNameOpts::INCL`  apply `'includeexpr'`
pub(crate) unsafe fn file_name_at_cursor(
    options: FileNameOpts,
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
unsafe fn name_start(line: *mut c_char, col: c_int, options: FileNameOpts) -> *mut c_char {
    // Search forward for what could be the start of a file name.
    let mut ptr = unsafe { line.offset(col as isize) };
    while unsafe { *ptr } != 0 && !unsafe { vim_isfilec(*ptr as u8 as c_int) } {
        ptr = unsafe { ptr.offset(utfc_ptr2len(ptr) as isize) };
    }
    if unsafe { *ptr } == 0 {
        return ptr::null_mut();
    }

    // Search backward for the first character of the file name.
    while ptr > line {
        let head_off = unsafe { utf_head_off(line, ptr.sub(1)) } as usize;
        if head_off > 0 {
            ptr = unsafe { ptr.sub(head_off + 1) };
        } else if unsafe { vim_isfilec(*ptr.sub(1) as u8 as c_int) }
            || (options.has(FileNameOpts::HYP) && unsafe { path_is_url(ptr.sub(1)) } != 0)
        {
            ptr = unsafe { ptr.sub(1) };
        } else {
            break;
        }
    }
    ptr
}

/// How many bytes of `ptr` belong to the file name that starts there.
///
/// `":"`, `"?"`, `"&"` and `"="` join the name once a `type://` prefix has
/// been seen, so that `http://google.com:8080?q=this&that=ok` comes out
/// whole. `"\ "` is an escaped space and counts as two.
unsafe fn name_length(ptr: *const c_char, options: FileNameOpts) -> usize {
    let hyp = options.has(FileNameOpts::HYP);
    // TODO(justinmk): Check for driveletter "x:/" at start, regardless of
    // 'isfname'.
    let mut len = if unsafe { path_has_drive_letter(ptr, strlen(ptr)) } {
        2
    } else {
        0
    };
    let mut in_type = true;
    let mut is_url = false;
    loop {
        let at = |i: usize| unsafe { *ptr.add(i) } as u8;
        let escaped_space = at(len) == b'\\' && at(len + 1) == b' ';
        if !(unsafe { vim_isfilec(at(len) as c_int) }
            || escaped_space
            || (hyp && unsafe { path_is_url(ptr.add(len)) } != 0)
            || (is_url && !unsafe { vim_strchr(c":?&=".as_ptr(), at(len) as c_int) }.is_null()))
        {
            break;
        }
        if at(len).is_ascii_alphabetic() {
            if in_type && unsafe { path_is_url(ptr.add(len + 1)) } != 0 {
                is_url = true;
            }
        } else {
            in_type = false;
        }
        if escaped_space {
            len += 1; // skip over the "\" in "\ "
        }
        len += unsafe { utfc_ptr2len(ptr.add(len)) } as usize;
    }

    // If there is trailing punctuation, remove it. But don't remove "..",
    // which could be a directory name.
    if len > 2
        && !unsafe { vim_strchr(c".,:;!".as_ptr(), *ptr.add(len - 1) as u8 as c_int) }.is_null()
        && unsafe { *ptr.add(len - 2) } != b'.' as c_char
    {
        len -= 1;
    }
    len
}

/// The line number written after a file name, as `" line 99"` or after any
/// single separator character. Both the English spelling and the translated
/// one are accepted, as `last_set_msg()` writes the latter.
unsafe fn trailing_line_number(after_name: *const c_char) -> Option<c_long> {
    let english = c" line ";
    let localized = unsafe { CStr::from_ptr(gettext(line_msg.as_ptr())) };

    let mut p = after_name.cast_mut();
    if unsafe { strncmp(p, english.as_ptr(), english.count_bytes()) } == 0 {
        p = unsafe { p.add(english.count_bytes()) };
    } else if unsafe { strncmp(p, localized.as_ptr(), localized.count_bytes()) } == 0 {
        p = unsafe { p.add(localized.count_bytes()) };
    } else {
        p = unsafe { skipwhite(p) };
    }

    if unsafe { *p } == 0 {
        return None;
    }
    if !(unsafe { *p } as u8).is_ascii_digit() {
        p = unsafe { p.add(1) }; // skip the separator
    }
    p = unsafe { skipwhite(p) };
    (unsafe { *p } as u8)
        .is_ascii_digit()
        .then(|| unsafe { getdigits_long(&raw mut p, false, 0) })
}

/// The name of the file under or after `line[col]`, looked up in `'path'`.
///
/// @param rel_fname  file we are searching relative to
/// @param file_lnum  line number after the file name
///
/// Otherwise like [`file_name_at_cursor`].
pub(crate) unsafe fn file_name_in_line(
    line: *mut c_char,
    col: c_int,
    options: FileNameOpts,
    count: c_int,
    rel_fname: *mut c_char,
    file_lnum: *mut linenr_T,
) -> *mut c_char {
    let ptr = unsafe { name_start(line, col, options) };
    if ptr.is_null() {
        if options.has(FileNameOpts::MESS) {
            unsafe { emsg(gettext(c"E446: No file name under cursor".as_ptr())) };
        }
        return ptr::null_mut();
    }

    let len = unsafe { name_length(ptr, options) };
    if !file_lnum.is_null()
        && let Some(lnum) = unsafe { trailing_line_number(ptr.add(len)) }
    {
        unsafe { *file_lnum = lnum as linenr_T };
    }

    unsafe { find_file_name_in_path(ptr, len, options, count as c_long, rel_fname) }
}

/// Run `'includeexpr'` over `ptr[len]`, with the name in `v:fname`.
pub(crate) unsafe fn eval_includeexpr(ptr: *const c_char, len: size_t) -> *mut c_char {
    unsafe { set_vim_var_string(Vv::Fname, ptr, len as ptrdiff_t) };
    // Errors go against the script that set `'includeexpr'`.
    let script_ctx =
        Script::context(unsafe { (*curbuf.get()).b_p_script_ctx[kBufOptIncludeexpr as usize] });

    let res = unsafe {
        eval_to_string_safe(
            (*curbuf.get()).b_p_inex,
            was_set_insecurely(curwin.get(), kOptIncludeexpr, OptionSetFlags::LOCAL),
            true,
        )
    };

    unsafe { set_vim_var_string(Vv::Fname, ptr::null(), 0) };
    drop(script_ctx);
    res
}

/// The name of the file `ptr[len]` in `'path'`.
///
/// Otherwise like [`file_name_at_cursor`].
///
/// @param rel_fname  file we are searching relative to
pub(crate) unsafe fn find_file_name_in_path(
    ptr: *mut c_char,
    len: size_t,
    options: FileNameOpts,
    count: c_long,
    rel_fname: *mut c_char,
) -> *mut c_char {
    let mut ptr = ptr;
    let mut len = len;
    let mut count = count;
    if len == 0 {
        return ptr::null_mut();
    }

    // "file:/name" and "file://name" both name "/name"; a drive letter
    // after "file:/" keeps the slash.
    if options.has(FileNameOpts::HYP)
        && len > 6
        && unsafe { strncmp(ptr, c"file:/".as_ptr(), 6) } == 0
        && !vim_ispathsep(unsafe { *ptr.add(6) } as c_int)
    {
        let off = if unsafe { path_has_drive_letter(ptr.add(6), len - 6) } {
            6
        } else {
            5
        };
        ptr = unsafe { ptr.add(off) };
        len -= off;
    }

    let mut tofree: *mut c_char = ptr::null_mut();
    if options.has(FileNameOpts::INCL) && unsafe { *(*curbuf.get()).b_p_inex } != 0 {
        tofree = unsafe { eval_includeexpr(ptr, len) };
        if !tofree.is_null() {
            ptr = tofree;
            len = unsafe { strlen(ptr) };
        }
    }

    let mut file_name: *mut c_char = ptr::null_mut();
    if options.has(FileNameOpts::EXP) {
        let mut file_to_find: *mut c_char = ptr::null_mut();
        let mut search_ctx: *mut c_char = ptr::null_mut();
        let quiet = options.without(FileNameOpts::MESS);
        let mut look = |ptr, len, first| unsafe {
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
            && !options.has(FileNameOpts::INCL)
            && unsafe { *(*curbuf.get()).b_p_inex } != 0
        {
            tofree = unsafe { eval_includeexpr(ptr, len) };
            if !tofree.is_null() {
                ptr = tofree;
                len = unsafe { strlen(ptr) };
                file_name = look(ptr, len, true);
            }
        }
        if file_name.is_null() && options.has(FileNameOpts::MESS) {
            let c = unsafe { *ptr.add(len) };
            unsafe { *ptr.add(len) = 0 };
            unsafe {
                semsg_c!(
                    gettext(c"E447: Can't find file \"%s\" in path".as_ptr()),
                    ptr,
                )
            };
            unsafe { *ptr.add(len) = c };
        }

        // Repeat finding the file "count" times. This matters when it
        // appears several times in the path.
        //
        // Note the repeats pass `options` unmasked, so FileNameOpts::MESS reaches
        // find_file_in_path and its "No more file" message. Upstream.
        while !file_name.is_null() && {
            count -= 1;
            count > 0
        } {
            unsafe { xfree(file_name.cast()) };
            file_name = unsafe {
                find_file_in_path(
                    ptr,
                    len,
                    options,
                    false,
                    rel_fname,
                    &raw mut file_to_find,
                    &raw mut search_ctx,
                )
            };
        }

        unsafe { xfree(file_to_find.cast()) };
        unsafe { vim_findfile_cleanup(search_ctx.cast()) };
    } else {
        file_name = unsafe { xstrnsave(ptr, len) };
    }

    unsafe { xfree(tofree.cast()) };
    file_name
}
