//! Looking a name up along `'path'` or `'cdpath'`.
//!
//! [`find_file_in_path_option`] is the loop over the option's entries: it
//! expands environment variables in the name, decides whether the name is
//! absolute enough to skip the option entirely, and otherwise drives
//! [`vim_findfile`](super::vim_findfile) once per entry, remembering where
//! it got to so that a repeat call answers the next match.
//! `'suffixesadd'` is tried at every candidate.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::path::buffer_path;
use core::ffi::{c_char, c_int};
use core::ptr;
use core::slice;

/// Find the file `ptr[len]` in `'path'`. Also finds directory names.
///
/// On the first call set `first` to true to initialize the search, false for
/// repeating calls. Repeating calls return other files called `ptr[len]`
/// from the path; only on the first call are `ptr` and `len` used.
///
/// If nothing is found on the first call, `FNAME_MESS` issues
/// `Can't find file "<file>" in path`; on repeating calls,
/// `No more file "<file>" found in path`.
///
/// Uses `NameBuff`.
///
/// @param ptr  file name
/// @param len  length of file name
/// @param first  use count'th matching file name
/// @param rel_fname  file name searching relative to
/// @param[in,out] file_to_find  modified copy of file name
/// @param[in,out] search_ctx  state of the search
///
/// @return  an allocated string for the file name. NULL for error.
pub unsafe extern "C" fn find_file_in_path(
    ptr: *mut c_char,
    len: size_t,
    options: c_int,
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
/// - `FNAME_MESS`   give error message when not found
/// - `FNAME_UNESC`  unescape backslashes
///
/// Uses `NameBuff`.
///
/// @param ptr  file name
/// @param len  length of file name
/// @param rel_fname  file name searching relative to
/// @param[in,out] file_to_find  modified copy of file name
/// @param[in,out] search_ctx  state of the search
///
/// @return  an allocated string for the file name. NULL for error.
pub unsafe extern "C" fn find_directory_in_path(
    ptr: *mut c_char,
    len: size_t,
    options: c_int,
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
/// With `FNAME_UNESC` every `"\ "` in the result becomes a plain space, so
/// that a name escaped for the command line reaches the file system whole.
unsafe fn prepare_name(
    ptr: *mut c_char,
    len: size_t,
    options: c_int,
    file_to_find: *mut *mut c_char,
) {
    unsafe {
        // expand_env_esc wants a NUL-terminated name, and the caller's is a
        // slice of a longer line.
        let save_char = *ptr.add(len);
        *ptr.add(len) = 0;
        let name_buff = NameBuff.ptr().cast::<c_char>();
        let written = expand_env_esc(
            ptr,
            name_buff,
            MAXPATHL as c_int,
            false,
            true,
            ptr::null_mut(),
        );
        *ptr.add(len) = save_char;

        xfree((*file_to_find).cast());
        let name_buff = slice::from_raw_parts(name_buff.cast::<u8>(), written);
        let mut name = name_buff.to_vec();
        if options & FNAME_UNESC as c_int != 0 {
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
        *file_to_find = xmemdupz(name.as_ptr().cast(), name.len()).cast();
    }
}

/// Is `name` `"."`, `".."`, or something below one of them? Such a name is
/// meant relative to the current directory and never looked for in `'path'`.
unsafe fn rel_to_curdir(name: *const c_char) -> bool {
    unsafe {
        let at = |i: usize| *name.add(i) as u8;
        let ends_component = |i: usize| at(i) == 0 || vim_ispathsep(at(i) as c_int);
        at(0) == b'.' && (ends_component(1) || (at(1) == b'.' && ends_component(2)))
    }
}

/// Try `name`, then `name` with each part of `'suffixesadd'` appended, and
/// answer the first that exists and is of the wanted kind.
///
/// The candidate is built in `NameBuff`, which already holds `name` for
/// `namelen` bytes.
unsafe fn try_suffixes(namelen: size_t, find_what: c_int, suffixes: *mut c_char) -> *mut c_char {
    unsafe {
        let name_buff = NameBuff.ptr().cast::<c_char>();
        let mut len = namelen;
        let mut suffix = suffixes;
        loop {
            if os_path_exists(name_buff)
                && (find_what == FINDFILE_BOTH as c_int
                    || (find_what == FINDFILE_DIR as c_int) == os_isdir(name_buff))
            {
                return xmemdupz(name_buff.cast(), len).cast();
            }
            if *suffix == 0 {
                return ptr::null_mut();
            }
            debug_assert!(namelen <= MAXPATHL);
            // `copy_option_part` answers what it wrote, which is at most
            // MAXPATHL - namelen - 1.
            len = namelen
                + copy_option_part(
                    &raw mut suffix,
                    name_buff.add(namelen),
                    MAXPATHL - namelen,
                    c",".as_ptr().cast_mut(),
                );
        }
    }
}

/// Look for a name that needs no `'path'`: absolute, or relative to the
/// current directory.
///
/// `FNAME_REL` asks for the directory of `rel_fname` to be tried first; the
/// current directory is the second and last try.
unsafe fn find_without_path(
    file_to_find: *const c_char,
    file_to_findlen: size_t,
    options: c_int,
    find_what: c_int,
    rel_fname: *const c_char,
    suffixes: *mut c_char,
) -> *mut c_char {
    unsafe {
        if path_with_url(file_to_find) != 0 {
            return xmemdupz(file_to_find.cast(), file_to_findlen).cast();
        }

        let name_buff = NameBuff.ptr().cast::<c_char>();
        let rel_fnamelen = if rel_fname.is_null() {
            0
        } else {
            strlen(rel_fname)
        };
        let relative = rel_to_curdir(file_to_find)
            && options & FNAME_REL as c_int != 0
            && !rel_fname.is_null()
            && rel_fnamelen + file_to_findlen < MAXPATHL;

        // Run 1 is relative to "rel_fname"'s directory, run 2 is relative to
        // the current directory. Only run 2 happens when the first does not
        // apply.
        for run in 1..=2 {
            let len = if run == 1 && relative {
                let len = vim_snprintf(
                    name_buff,
                    MAXPATHL,
                    c"%.*s%s".as_ptr(),
                    path_tail(rel_fname.cast_mut()).offset_from(rel_fname) as c_int,
                    rel_fname,
                    file_to_find,
                ) as size_t;
                debug_assert!(len < MAXPATHL);
                len
            } else if run == 1 {
                continue;
            } else {
                strcpy(name_buff, file_to_find);
                file_to_findlen
            };

            let found = try_suffixes(len, find_what, suffixes);
            if !found.is_null() {
                return found;
            }
        }
        ptr::null_mut()
    }
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
    search_ctx: *mut *mut ff_search_ctx_T,
) -> *mut c_char {
    unsafe {
        // Where the last call had got to in the option.
        static DIR: GlobalCell<*mut c_char> = GlobalCell::new(ptr::null_mut());
        // Whether `*search_ctx` is a context a `vim_findfile` may resume.
        static INITIALIZED: GlobalCell<bool> = GlobalCell::new(false);

        if first {
            // vim_findfile_free_visited can handle a possible NULL pointer
            vim_findfile_free_visited((*search_ctx).cast());
            DIR.set(path_option);
            INITIALIZED.set(false);
        }

        loop {
            if INITIALIZED.get() {
                let file_name = vim_findfile((*search_ctx).cast());
                if !file_name.is_null() {
                    return file_name;
                }
                INITIALIZED.set(false);
                continue;
            }

            if DIR.get().is_null() || *DIR.get() == 0 {
                // We searched all paths of the option, now we can free the
                // search context.
                vim_findfile_cleanup((*search_ctx).cast());
                *search_ctx = ptr::null_mut();
                return ptr::null_mut();
            }

            let mut buf = vec![0 as c_char; MAXPATHL];
            copy_option_part(
                DIR.ptr(),
                buf.as_mut_ptr(),
                MAXPATHL,
                c" ,".as_ptr().cast_mut(),
            );
            // Splits the entry at an unescaped ';', leaving the entry in
            // `buf` and answering the stop directories after it.
            let stopdirs = vim_findfile_stopdir(buf.as_mut_ptr());
            *search_ctx = vim_findfile_init(
                buf.as_mut_ptr(),
                file_to_find.cast_mut(),
                file_to_findlen,
                stopdirs,
                100,
                false_0,
                find_what,
                (*search_ctx).cast(),
                false_0,
                rel_fname,
            )
            .cast();
            if !(*search_ctx).is_null() {
                INITIALIZED.set(true);
            }
        }
    }
}

/// Say that `file_to_find` is not there, in the wording the caller earned:
/// a first call has not found it at all, a repeat call has run out.
unsafe fn report_missing(first: bool, find_what: c_int, file_to_find: *const c_char) {
    unsafe {
        let message = match (first, find_what == FINDFILE_DIR as c_int) {
            (true, true) => e_cant_find_directory_str_in_cdpath.ptr().cast::<c_char>(),
            (true, false) => e_cant_find_file_str_in_path.ptr().cast::<c_char>(),
            (false, true) => e_no_more_directory_str_found_in_cdpath
                .ptr()
                .cast::<c_char>(),
            (false, false) => e_no_more_file_str_found_in_path.ptr().cast::<c_char>(),
        };
        semsg(gettext(message), file_to_find);
    }
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
pub unsafe extern "C" fn find_file_in_path_option(
    ptr: *mut c_char,
    len: size_t,
    options: c_int,
    first: bool,
    path_option: *mut c_char,
    find_what: c_int,
    rel_fname: *mut c_char,
    suffixes: *mut c_char,
    file_to_find: *mut *mut c_char,
    search_ctx_arg: *mut *mut c_char,
) -> *mut c_char {
    unsafe {
        let search_ctx = search_ctx_arg.cast::<*mut ff_search_ctx_T>();
        // Do not attempt to search "relative" to a URL. #6009
        let rel_fname = if !rel_fname.is_null() && path_with_url(rel_fname) != 0 {
            ptr::null_mut()
        } else {
            rel_fname
        };

        if first {
            if len == 0 {
                return ptr::null_mut();
            }
            prepare_name(ptr, len, options, file_to_find);
        }
        let name = *file_to_find;
        let namelen = strlen(name);

        // "..", "../path", "." and "./path" mean the current directory just
        // as an absolute name means itself: neither uses `path_option`.
        let file_name = if vim_isAbsName(name) || rel_to_curdir(name) {
            // If this is not a first call, return NULL: we already returned
            // a filename on the first call.
            if first {
                find_without_path(name, namelen, options, find_what, rel_fname, suffixes)
            } else {
                ptr::null_mut()
            }
        } else {
            find_along_option(
                first,
                path_option,
                find_what,
                rel_fname,
                name,
                namelen,
                search_ctx,
            )
        };

        if file_name.is_null() && options & FNAME_MESS as c_int != 0 {
            report_missing(first, find_what, name);
        }
        file_name
    }
}
