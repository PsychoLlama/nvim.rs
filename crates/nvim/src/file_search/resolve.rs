//! Looking a name up along `'path'` or `'cdpath'`.
//!
//! [`find_file_in_path_option`] is the loop over the option's entries: it
//! expands environment variables in the name, decides whether the name is
//! absolute enough to skip the option entirely, and otherwise drives
//! [`vim_findfile`](super::vim_findfile) once per entry, remembering where
//! it got to so that a repeat call answers the next match.
//! `'suffixesadd'` is tried at every candidate.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cstr;
use crate::message_fmt::{c_str, emsg_text};
use crate::path::buffer_path;
use crate::tr_c;
use crate::types::MAXPATHL;
use core::ffi::{c_char, c_int};
use core::{ptr, slice};

/// Find the file `ptr[len]` in `'path'`. Also finds directory names.
///
/// On the first call set `first` to true to initialize the search, false for
/// repeating calls. Repeating calls return other files called `ptr[len]`
/// from the path; only on the first call are `ptr` and `len` used.
///
/// If nothing is found on the first call, `FileNameOpts::MESS` issues
/// `Can't find file "<file>" in path`; on repeating calls,
/// `No more file "<file>" found in path`.
///
/// @param ptr  file name
/// @param len  length of file name
/// @param first  use count'th matching file name
/// @param rel_fname  file name searching relative to
/// @param[in,out] file_to_find  modified copy of file name
/// @param[in,out] search_ctx  state of the search
///
/// @return  an allocated string for the file name. NULL for error.
pub(crate) unsafe fn find_file_in_path(
    ptr: *mut c_char,
    len: size_t,
    options: FileNameOpts,
    first: bool,
    rel_fname: *mut c_char,
    file_to_find: *mut *mut c_char,
    search_ctx: *mut *mut c_char,
) -> *mut c_char {
    unsafe {
        find_file_in_path_option(
            ptr,
            len,
            options,
            first,
            buffer_path(),
            FINDFILE_BOTH as c_int,
            rel_fname,
            (*curbuf.get()).b_p_sua,
            file_to_find,
            search_ctx,
        )
    }
}

/// Find the directory name `ptr[len]` in `'cdpath'`.
///
/// options:
/// - `FileNameOpts::MESS`   give error message when not found
/// - `FileNameOpts::UNESC`  unescape backslashes
///
/// @param ptr  file name
/// @param len  length of file name
/// @param rel_fname  file name searching relative to
/// @param[in,out] file_to_find  modified copy of file name
/// @param[in,out] search_ctx  state of the search
///
/// @return  an allocated string for the file name. NULL for error.
pub(crate) unsafe fn find_directory_in_path(
    ptr: *mut c_char,
    len: size_t,
    options: FileNameOpts,
    rel_fname: *mut c_char,
    file_to_find: *mut *mut c_char,
    search_ctx: *mut *mut c_char,
) -> *mut c_char {
    unsafe {
        find_file_in_path_option(
            ptr,
            len,
            options,
            true,
            p_cdpath.get(),
            FINDFILE_DIR as c_int,
            rel_fname,
            c"".as_ptr().cast_mut(),
            file_to_find,
            search_ctx,
        )
    }
}

/// Replace `*file_to_find` with `ptr[len]`, environment variables expanded.
///
/// With `FileNameOpts::UNESC` every `"\ "` in the result becomes a plain space, so
/// that a name escaped for the command line reaches the file system whole.
unsafe fn prepare_name(
    ptr: *mut c_char,
    len: size_t,
    options: FileNameOpts,
    file_to_find: *mut *mut c_char,
) {
    let mut expanded = [0 as c_char; MAXPATHL as usize];
    // expand_env_esc wants a NUL-terminated name, and the caller's is a
    // slice of a longer line.
    let save_char = unsafe { *ptr.add(len) };
    unsafe { *ptr.add(len) = 0 };
    let name_buff = expanded.as_mut_ptr();
    let written = unsafe {
        expand_env_esc(
            ptr,
            name_buff,
            MAXPATHL as c_int,
            false,
            true,
            ptr::null_mut(),
        )
    };
    unsafe { *ptr.add(len) = save_char };

    unsafe { xfree((*file_to_find).cast()) };
    let name_buff = unsafe { slice::from_raw_parts(name_buff.cast::<u8>(), written) };
    let mut name = name_buff.to_vec();
    if options.has(FileNameOpts::UNESC) {
        // Change all "\ " to " ".
        let mut out = 0;
        let mut at = 0;
        while at < name.len() {
            if name[at] == b'\\' && name.get(at + 1) == Some(&b' ') {
                at += 1;
            }
            name[out] = name[at];
            out += 1;
            at += 1;
        }
        name.truncate(out);
    }
    unsafe { *file_to_find = xmemdupz(name.as_ptr().cast(), name.len()).cast() };
}

/// Is `name` `"."`, `".."`, or something below one of them? Such a name is
/// meant relative to the current directory and never looked for in `'path'`.
unsafe fn rel_to_curdir(name: *const c_char) -> bool {
    let at = |i: usize| unsafe { *name.add(i) } as u8;
    let ends_component = |i: usize| at(i) == 0 || vim_ispathsep(at(i) as c_int);
    at(0) == b'.' && (ends_component(1) || (at(1) == b'.' && ends_component(2)))
}

/// Try `name`, then `name` with each part of `'suffixesadd'` appended, and
/// answer the first that exists and is of the wanted kind.
///
/// The candidate is built in `name_buff`, which already holds the name for
/// `namelen` bytes.
unsafe fn try_suffixes(
    name_buff: &mut [c_char; MAXPATHL as usize],
    namelen: size_t,
    find_what: c_int,
    suffixes: *mut c_char,
) -> *mut c_char {
    let name_buff = name_buff.as_mut_ptr();
    let mut len = namelen;
    let mut suffix = suffixes;
    loop {
        if unsafe { os_path_exists(name_buff) }
            && (find_what == FINDFILE_BOTH as c_int
                || (find_what == FINDFILE_DIR as c_int) == unsafe { os_isdir(name_buff) })
        {
            return unsafe { xmemdupz(name_buff.cast(), len) }.cast();
        }
        if unsafe { *suffix } == 0 {
            return ptr::null_mut();
        }
        debug_assert!(namelen <= MAXPATHL as usize);
        // `copy_option_part` answers what it wrote, which is at most
        // MAXPATHL - namelen - 1.
        len = namelen
            + unsafe {
                copy_option_part(
                    &raw mut suffix,
                    name_buff.add(namelen),
                    MAXPATHL as usize - namelen,
                    c",".as_ptr().cast_mut(),
                )
            };
    }
}

/// Look for a name that needs no `'path'`: absolute, or relative to the
/// current directory.
///
/// `FileNameOpts::REL` asks for the directory of `rel_fname` to be tried first; the
/// current directory is the second and last try.
unsafe fn find_without_path(
    file_to_find: *const c_char,
    file_to_findlen: size_t,
    options: FileNameOpts,
    find_what: c_int,
    rel_fname: *const c_char,
    suffixes: *mut c_char,
) -> *mut c_char {
    // The candidate being tried. Upstream shares `NameBuff` between this
    // and `try_suffixes`, which appends to it.
    let mut candidate = [0 as c_char; MAXPATHL as usize];
    if unsafe { path_with_url(file_to_find) } != 0 {
        return unsafe { xmemdupz(file_to_find.cast(), file_to_findlen) }.cast();
    }

    let name_buff = candidate.as_mut_ptr();
    let rel_fnamelen = if rel_fname.is_null() {
        0
    } else {
        unsafe { cstr::bytes_at(rel_fname) }.len()
    };
    let relative = unsafe { rel_to_curdir(file_to_find) }
        && options.has(FileNameOpts::REL)
        && !rel_fname.is_null()
        && rel_fnamelen + file_to_findlen < MAXPATHL as usize;

    // Run 1 is relative to "rel_fname"'s directory, run 2 is relative to
    // the current directory. Only run 2 happens when the first does not
    // apply.
    for run in 1..=2 {
        let len = if run == 1 && relative {
            let len = unsafe {
                vim_snprintf(
                    name_buff,
                    MAXPATHL as usize,
                    c"%.*s%s".as_ptr(),
                    path_tail(rel_fname.cast_mut()).offset_from(rel_fname) as c_int,
                    rel_fname,
                    file_to_find,
                )
            } as size_t;
            debug_assert!(len < MAXPATHL as usize);
            len
        } else if run == 1 {
            continue;
        } else {
            unsafe { strcpy(name_buff, file_to_find) };
            file_to_findlen
        };

        let found = unsafe { try_suffixes(&mut candidate, len, find_what, suffixes) };
        if !found.is_null() {
            return found;
        }
    }
    ptr::null_mut()
}

/// Drive `vim_findfile` over the entries of `'path'` or `'cdpath'`, one
/// entry at a time, answering the next match each call.
///
/// The position in the option and the half-finished search context are the
/// state a repeating call resumes from — upstream keeps them in statics, so
/// there is one such walk in the whole editor.
unsafe fn find_along_option(
    first: bool,
    path_option: *mut c_char,
    find_what: c_int,
    rel_fname: *mut c_char,
    file_to_find: *const c_char,
    file_to_findlen: size_t,
    search_ctx: *mut *mut FindContext,
) -> *mut c_char {
    // Where the last call had got to in the option.
    static DIR: GlobalCell<*mut c_char> = GlobalCell::new(ptr::null_mut());
    // Whether `*search_ctx` is a context a `vim_findfile` may resume.
    static INITIALIZED: GlobalCell<bool> = GlobalCell::new(false);

    if first {
        // vim_findfile_free_visited can handle a possible NULL pointer
        unsafe { vim_findfile_free_visited((*search_ctx).cast()) };
        DIR.set(path_option);
        INITIALIZED.set(false);
    }

    loop {
        if INITIALIZED.get() {
            let file_name = unsafe { vim_findfile((*search_ctx).cast()) };
            if !file_name.is_null() {
                return file_name;
            }
            INITIALIZED.set(false);
            continue;
        }

        if DIR.get().is_null() || unsafe { *DIR.get() } == 0 {
            // We searched all paths of the option, now we can free the
            // search context.
            unsafe { vim_findfile_cleanup((*search_ctx).cast()) };
            unsafe { *search_ctx = ptr::null_mut() };
            return ptr::null_mut();
        }

        let mut buf = vec![0 as c_char; MAXPATHL as usize];
        let mut dir = DIR.get();
        unsafe {
            copy_option_part(
                &raw mut dir,
                buf.as_mut_ptr(),
                MAXPATHL as usize,
                c" ,".as_ptr().cast_mut(),
            )
        };
        DIR.set(dir);
        // Splits the entry at an unescaped ';', leaving the entry in
        // `buf` and answering the stop directories after it.
        let stopdirs = unsafe { vim_findfile_stopdir(buf.as_mut_ptr()) };
        unsafe {
            *search_ctx = vim_findfile_init(
                buf.as_mut_ptr(),
                file_to_find.cast_mut(),
                file_to_findlen,
                stopdirs,
                100,
                false,
                find_what,
                (*search_ctx).cast(),
                false,
                rel_fname,
            )
            .cast()
        };
        if !unsafe { *search_ctx }.is_null() {
            INITIALIZED.set(true);
        }
    }
}

/// Say that `file_to_find` is not there, in the wording the caller earned:
/// a first call has not found it at all, a repeat call has run out.
unsafe fn report_missing(first: bool, find_what: c_int, file_to_find: *const c_char) {
    let message = match (first, find_what == FINDFILE_DIR as c_int) {
        (true, true) => e_cant_find_directory_str_in_cdpath,
        (true, false) => e_cant_find_file_str_in_path,
        (false, true) => e_no_more_directory_str_found_in_cdpath,
        (false, false) => e_no_more_file_str_found_in_path,
    };
    // SAFETY: the caller's NUL-terminated name.
    let file_to_find = unsafe { c_str(file_to_find) };
    emsg_text(tr_c!(message, file_to_find));
}

/// @param ptr  file name
/// @param len  length of file name
/// @param first  use count'th matching file name
/// @param path_option  `'path'` or `'cdpath'`
/// @param find_what  `FINDFILE_FILE`, `_DIR` or `_BOTH`
/// @param rel_fname  file name we are looking relative to
/// @param suffixes  list of suffixes, `'suffixesadd'` option
/// @param[in,out] file_to_find  modified copy of file name
/// @param[in,out] search_ctx_arg  state of the search
pub(crate) unsafe fn find_file_in_path_option(
    ptr: *mut c_char,
    len: size_t,
    options: FileNameOpts,
    first: bool,
    path_option: *mut c_char,
    find_what: c_int,
    rel_fname: *mut c_char,
    suffixes: *mut c_char,
    file_to_find: *mut *mut c_char,
    search_ctx_arg: *mut *mut c_char,
) -> *mut c_char {
    let search_ctx = search_ctx_arg.cast::<*mut FindContext>();
    // Do not attempt to search "relative" to a URL. #6009
    let rel_fname = if !rel_fname.is_null() && unsafe { path_with_url(rel_fname) } != 0 {
        ptr::null_mut()
    } else {
        rel_fname
    };

    if first {
        if len == 0 {
            return ptr::null_mut();
        }
        unsafe { prepare_name(ptr, len, options, file_to_find) };
    }
    let name = unsafe { *file_to_find };
    let namelen = unsafe { cstr::bytes_at(name) }.len();

    // "..", "../path", "." and "./path" mean the current directory just
    // as an absolute name means itself: neither uses `path_option`.
    let file_name = if unsafe { vim_is_abs_name(name) } || unsafe { rel_to_curdir(name) } {
        // If this is not a first call, return NULL: we already returned
        // a filename on the first call.
        if first {
            unsafe { find_without_path(name, namelen, options, find_what, rel_fname, suffixes) }
        } else {
            ptr::null_mut()
        }
    } else {
        unsafe {
            find_along_option(
                first,
                path_option,
                find_what,
                rel_fname,
                name,
                namelen,
                search_ctx,
            )
        }
    };

    if file_name.is_null() && options.has(FileNameOpts::MESS) {
        unsafe { report_missing(first, find_what, name) };
    }
    file_name
}
