//! Reading a file name out of the buffer's text.
//!
//! [`file_name_in_line`] is what `gf` and its neighbours use: it finds the
//! run of `'isfname'` characters around a column, allows the extra
//! characters a URL needs, drops trailing punctuation, and picks up a
//! trailing `" line 99"`. [`find_file_name_in_path`] then looks the name up
//! along `'path'`, applying `'includeexpr'` when asked to or when the plain
//! lookup failed.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::cstr;
use crate::guard::Script;
use crate::message_fmt::c_str;
use crate::normal::visual_active;
use crate::semsg;
use crate::strings::has_char;
use crate::types::{FAIL, OptionSetFlags, Vv};
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int, c_long};
use core::ptr;
use std::ffi::CStr;

/// The file name at the cursor, or the Visual selection when there is one.
///
/// Returns the name in allocated memory, NULL for failure.
///
/// # Safety
///
/// `file_lnum` must point at a writable line number the caller owns.
pub(crate) unsafe fn grab_file_name(count: c_int, file_lnum: *mut LineNr) -> *mut c_char {
    let options = FileNameOpts::MESS | FileNameOpts::EXP | FileNameOpts::REL | FileNameOpts::UNESC;
    if !visual_active() {
        return unsafe { file_name_at_cursor(options | FileNameOpts::HYP, count, file_lnum) };
    }

    let mut len: size_t = 0;
    let mut ptr: *mut c_char = ptr::null_mut();
    if unsafe { get_visual_text(None, &raw mut ptr, &raw mut len) } as c_int == FAIL {
        return ptr::null_mut();
    }
    // Only recognize ":123" here.
    if !file_lnum.is_null()
        && unsafe { *ptr.add(len) } == b':' as c_char
        && (unsafe { *ptr.add(len + 1) } as u8).is_ascii_digit()
    {
        let mut p = unsafe { ptr.add(len + 1) };
        unsafe { *file_lnum = getdigits_int32(&raw mut p, false, 0) as LineNr };
    }
    unsafe { find_file_name_in_path(ptr, len, options, count as c_long, Buf::current().b_ffname) }
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
///
/// # Safety
///
/// `file_lnum` must point at a writable line number the caller owns.
pub(crate) unsafe fn file_name_at_cursor(
    options: FileNameOpts,
    count: c_int,
    file_lnum: *mut LineNr,
) -> *mut c_char {
    unsafe {
        file_name_in_line(
            get_cursor_line_ptr(),
            Win::current().w_cursor.col as c_int,
            options,
            count,
            Buf::current().b_ffname,
            file_lnum,
        )
    }
}

/// The start of the file name around `line[col]`, or NULL when the rest of
/// the line holds no `'isfname'` character at all.
///
/// Goes one character back to the `":"` before `"//"`, or to the drive letter
/// before `":\"`, even when `":"` is not in `'isfname'`.
///
/// # Safety
///
/// `line` must point at a NUL-terminated string, unaliased for the call.
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
            || (options.has(FileNameOpts::HYP) && unsafe { path_is_url(cstr::at(ptr.sub(1))) } != 0)
        {
            ptr = unsafe { ptr.sub(1) };
        } else {
            break;
        }
    }
    ptr
}

/// How many bytes of `name` belong to the file name that starts there.
///
/// `":"`, `"?"`, `"&"` and `"="` join the name once a `type://` prefix has
/// been seen, so that `http://google.com:8080?q=this&that=ok` comes out
/// whole. `"\ "` is an escaped space and counts as two.
///
/// # Safety
///
/// `name` must point at a NUL-terminated string.
unsafe fn name_length(name: *const c_char, options: FileNameOpts) -> usize {
    let hyp = options.has(FileNameOpts::HYP);
    // TODO(justinmk): Check for driveletter "x:/" at start, regardless of
    // 'isfname'.
    let mut len = if path_has_drive_letter(unsafe { cstr::bytes_at(name) }) {
        2
    } else {
        0
    };
    let mut in_type = true;
    let mut is_url = false;
    loop {
        let at = |i: usize| unsafe { *name.add(i) } as u8;
        let escaped_space = at(len) == b'\\' && at(len + 1) == b' ';
        if !(vim_isfilec(at(len) as c_int)
            || escaped_space
            || (hyp && unsafe { path_is_url(cstr::at(name.add(len))) } != 0)
            || (is_url && has_char(c":?&=", at(len) as c_int)))
        {
            break;
        }
        if at(len).is_ascii_alphabetic() {
            if in_type && unsafe { path_is_url(cstr::at(name.add(len + 1))) } != 0 {
                is_url = true;
            }
        } else {
            in_type = false;
        }
        if escaped_space {
            len += 1; // skip over the "\" in "\ "
        }
        len += unsafe { utfc_ptr2len(name.add(len)) } as usize;
    }

    // If there is trailing punctuation, remove it. But don't remove "..",
    // which could be a directory name.
    if len > 2
        && has_char(c".,:;!", unsafe { *name.add(len - 1) } as u8 as c_int)
        && unsafe { *name.add(len - 2) } != b'.' as c_char
    {
        len -= 1;
    }
    len
}

/// The line number written after a file name, as `" line 99"` or after any
/// single separator character. Both the English spelling and the translated
/// one are accepted, as `last_set_msg()` writes the latter.
///
/// # Safety
///
/// `after_name` must point at a NUL-terminated string.
unsafe fn trailing_line_number(after_name: *const c_char) -> Option<c_long> {
    let english = c" line ";
    let localized = unsafe { CStr::from_ptr(gettext(line_msg).as_ptr()) };

    let mut p = after_name.cast_mut();
    if unsafe { cstr::prefix_eq(p, english.as_ptr(), english.count_bytes()) } {
        p = unsafe { p.add(english.count_bytes()) };
    } else if unsafe { cstr::prefix_eq(p, localized.as_ptr(), localized.count_bytes()) } {
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
///
/// # Safety
///
/// `line` must point at a NUL-terminated string, unaliased for the call.
/// `rel_fname` must point at a NUL-terminated string, unaliased for the call.
/// `file_lnum` must point at a writable line number the caller owns.
pub(crate) unsafe fn file_name_in_line(
    line: *mut c_char,
    col: c_int,
    options: FileNameOpts,
    count: c_int,
    rel_fname: *mut c_char,
    file_lnum: *mut LineNr,
) -> *mut c_char {
    let ptr = unsafe { name_start(line, col, options) };
    if ptr.is_null() {
        if options.has(FileNameOpts::MESS) {
            emsg(gettext(c"E446: No file name under cursor"));
        }
        return ptr::null_mut();
    }

    let len = unsafe { name_length(ptr, options) };
    if !file_lnum.is_null()
        && let Some(lnum) = unsafe { trailing_line_number(ptr.add(len)) }
    {
        unsafe { *file_lnum = lnum as LineNr };
    }

    unsafe { find_file_name_in_path(ptr, len, options, count as c_long, rel_fname) }
}

/// Run `'includeexpr'` over `ptr[len]`, with the name in `v:fname`.
///
/// # Safety
///
/// `name` must point at `len` readable bytes.
pub(crate) unsafe fn eval_includeexpr(name: *const c_char, len: size_t) -> *mut c_char {
    unsafe { set_vim_var_string(Vv::Fname, name, len as ptrdiff_t) };
    // Errors go against the script that set `'includeexpr'`.
    let script_ctx = Script::context(Buf::current().b_p_script_ctx[kBufOptIncludeexpr as usize]);

    let res = unsafe {
        eval_to_string_safe(
            Buf::current().b_p_inex,
            was_set_insecurely(Win::current(), kOptIncludeexpr, OptionSetFlags::LOCAL),
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
///
/// # Safety
///
/// `name` must point at `len` bytes the caller owns, readable and writable,
/// unaliased for the call. `rel_fname` must point at a NUL-terminated string,
/// unaliased for the call.
pub(crate) unsafe fn find_file_name_in_path(
    name: *mut c_char,
    len: size_t,
    options: FileNameOpts,
    count: c_long,
    rel_fname: *mut c_char,
) -> *mut c_char {
    let mut name = name;
    let mut len = len;
    let mut count = count;
    if len == 0 {
        return ptr::null_mut();
    }

    // "file:/name" and "file://name" both name "/name"; a drive letter
    // after "file:/" keeps the slash.
    if options.has(FileNameOpts::HYP)
        && len > 6
        && unsafe { cstr::starts_with(name, b"file:/") }
        && !vim_ispathsep(unsafe { *name.add(6) } as c_int)
    {
        let off = if path_has_drive_letter(unsafe { &cstr::bytes_at(name.add(6))[..len - 6] }) {
            6
        } else {
            5
        };
        name = unsafe { name.add(off) };
        len -= off;
    }

    let mut tofree: *mut c_char = ptr::null_mut();
    if options.has(FileNameOpts::INCL) && unsafe { *Buf::current().b_p_inex } != 0 {
        tofree = unsafe { eval_includeexpr(name, len) };
        if !tofree.is_null() {
            name = tofree;
            len = unsafe { cstr::bytes_at(name) }.len();
        }
    }

    let mut file_name: *mut c_char;
    if options.has(FileNameOpts::EXP) {
        let mut file_to_find: *mut c_char = ptr::null_mut();
        let mut search_ctx: *mut c_char = ptr::null_mut();
        let quiet = options.without(FileNameOpts::MESS);
        let mut look = |name, len, first| unsafe {
            find_file_in_path(
                name,
                len,
                quiet,
                first,
                rel_fname,
                &raw mut file_to_find,
                &raw mut search_ctx,
            )
        };
        file_name = look(name, len, true);

        // If the file could not be found in a normal way, try applying
        // 'includeexpr' (unless done already).
        if file_name.is_null()
            && !options.has(FileNameOpts::INCL)
            && unsafe { *Buf::current().b_p_inex } != 0
        {
            tofree = unsafe { eval_includeexpr(name, len) };
            if !tofree.is_null() {
                name = tofree;
                len = unsafe { cstr::bytes_at(name) }.len();
                file_name = look(name, len, true);
            }
        }
        if file_name.is_null() && options.has(FileNameOpts::MESS) {
            let c = unsafe { *name.add(len) };
            unsafe { *name.add(len) = 0 };
            // SAFETY: the byte past the name was just replaced by a NUL.
            let shown = unsafe { c_str(name) };
            semsg!("E447: Can't find file \"{shown}\" in path");
            unsafe { *name.add(len) = c };
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
                    name,
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
        file_name = unsafe { xstrnsave(name, len) };
    }

    unsafe { xfree(tofree.cast()) };
    file_name
}
