//! Helpgrep search implementation.
//!
//! Migrated from `hgr_search_file`, `hgr_search_files_in_dir`, and
//! `hgr_search_in_rtp` in `quickfix_shim.c` (Phase 1).

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::{c_char, c_int, c_void, CStr};

// =============================================================================
// Type aliases
// =============================================================================

type LinenrT = i32;
/// Opaque handle to `qf_list_T`
type QfListHandleMut = *mut c_void;
/// Opaque handle to `regmatch_T`
type RegmatchHandle = *mut c_void;
/// Opaque FILE* handle
type FileHandle = *mut c_void;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    // File I/O
    fn nvim_hgr_os_fopen(fname: *const c_char) -> FileHandle;
    fn nvim_hgr_vim_fgets(buf: *mut c_char, size: c_int, fd: FileHandle) -> bool;
    fn nvim_hgr_fclose(fd: FileHandle);

    // Regex
    fn nvim_hgr_vim_regexec(rmp: RegmatchHandle, line: *mut c_char) -> bool;
    fn nvim_hgr_regmatch_startp(rmp: RegmatchHandle) -> *mut c_char;
    fn nvim_hgr_regmatch_endp(rmp: RegmatchHandle) -> *mut c_char;

    // Globals
    fn nvim_hgr_get_IObuff() -> *mut c_char;
    fn nvim_hgr_get_IOSIZE() -> c_int;
    fn nvim_hgr_get_got_int() -> c_int;
    fn nvim_hgr_set_got_int(val: c_int);
    fn nvim_hgr_line_breakcheck();

    // Wildcard expansion
    fn nvim_hgr_gen_expand_wildcards(
        dirname: *mut c_char,
        fcount_out: *mut c_int,
        fnames_out: *mut *mut *mut c_char,
    ) -> c_int;
    fn nvim_hgr_free_wild(fcount: c_int, fnames: *mut *mut c_char);
    fn nvim_hgr_fname_at(fnames: *mut *mut c_char, idx: c_int) -> *mut c_char;
    fn nvim_hgr_add_pathsep(dirname: *mut c_char);
    fn nvim_hgr_strcat_doc_glob(dirname: *mut c_char);
    fn nvim_hgr_STRNICMP(a: *const c_char, b: *const c_char, n: c_int) -> c_int;

    // Option part / p_rtp iteration
    fn nvim_hgr_get_p_rtp() -> *mut c_char;
    fn nvim_hgr_copy_option_part(pp: *mut *mut c_char, buf: *mut c_char, maxlen: c_int);
    fn nvim_hgr_get_MAXPATHL() -> c_int;
    fn nvim_hgr_get_NameBuff() -> *mut c_char;

    // Quickfix entry addition
    fn rs_qf_add_entry(
        qfl: QfListHandleMut,
        dir: *mut c_char,
        fname: *const c_char,
        module: *const c_char,
        bufnum: c_int,
        mesg: *const c_char,
        lnum: LinenrT,
        end_lnum: LinenrT,
        col: c_int,
        end_col: c_int,
        vis_col: c_char,
        pattern: *const c_char,
        nr: c_int,
        type_char: c_char,
        user_data: *const c_void,
        valid: c_char,
    ) -> c_int;
}

// from quickfix.h
const QF_FAIL: c_int = 0;
const OK: c_int = 1;

/// Search for a pattern in a help file and add matches to the quickfix list.
///
/// Migrated from `hgr_search_file` in `quickfix_shim.c`.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T`
/// - `fname` must be a valid C string
/// - `p_regmatch` must be a valid pointer to a `regmatch_T`
unsafe fn hgr_search_file(qfl: QfListHandleMut, fname: *const c_char, p_regmatch: RegmatchHandle) {
    let fd = nvim_hgr_os_fopen(fname);
    if fd.is_null() {
        return;
    }

    let iobuff = nvim_hgr_get_IObuff();
    let iosize = nvim_hgr_get_IOSIZE();

    let mut lnum: LinenrT = 1;
    while !nvim_hgr_vim_fgets(iobuff, iosize, fd) && nvim_hgr_get_got_int() == 0 {
        let line = iobuff;

        if nvim_hgr_vim_regexec(p_regmatch, line) {
            // Compute byte length of line
            let mut l = CStr::from_ptr(line).to_bytes().len() as c_int;

            // Remove trailing CR, LF, spaces, etc.
            while l > 0 && *line.add(l as usize - 1) <= b' ' as c_char {
                l -= 1;
                *line.add(l as usize) = 0;
            }

            let startp = nvim_hgr_regmatch_startp(p_regmatch);
            let endp = nvim_hgr_regmatch_endp(p_regmatch);
            let col = (startp.offset_from(line) as c_int) + 1;
            let end_col = (endp.offset_from(line) as c_int) + 1;

            if rs_qf_add_entry(
                qfl,
                std::ptr::null_mut(), // dir
                fname,
                std::ptr::null(), // module
                0,
                line.cast_const(),
                lnum,
                0,
                col,
                end_col,
                c_char::from(false), // vis_col
                std::ptr::null(),    // pattern
                0,
                1i8,                // type = 1 (helpgrep)
                std::ptr::null(),   // user_data
                c_char::from(true), // valid
            ) == QF_FAIL
            {
                nvim_hgr_set_got_int(1);
                break;
            }
        }
        lnum += 1;
        nvim_hgr_line_breakcheck();
    }
    nvim_hgr_fclose(fd);
}

/// Search for a pattern in all help files in the doc directory under `dirname`.
///
/// Migrated from `hgr_search_files_in_dir` in `quickfix_shim.c`.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T`
/// - `dirname` must be a valid, writable C string buffer (MAXPATHL bytes)
/// - `p_regmatch` must be a valid pointer to a `regmatch_T`
/// - `lang` must be a valid C string or NULL
unsafe fn hgr_search_files_in_dir(
    qfl: QfListHandleMut,
    dirname: *mut c_char,
    p_regmatch: RegmatchHandle,
    lang: *const c_char,
) {
    // Append path separator and doc glob pattern
    nvim_hgr_add_pathsep(dirname);
    nvim_hgr_strcat_doc_glob(dirname);

    let mut fcount: c_int = 0;
    let mut fnames: *mut *mut c_char = std::ptr::null_mut();

    if nvim_hgr_gen_expand_wildcards(dirname, &raw mut fcount, &raw mut fnames) == OK && fcount > 0
    {
        let mut fi = 0;
        while fi < fcount && nvim_hgr_get_got_int() == 0 {
            let fname = nvim_hgr_fname_at(fnames, fi);

            // Skip files for a different language.
            if !lang.is_null() {
                let fname_len = CStr::from_ptr(fname).to_bytes().len() as c_int;
                let ext_offset = fname_len - 3;
                if ext_offset < 0 {
                    fi += 1;
                    continue;
                }
                let ext_ptr = fname.add(ext_offset as usize);

                // Skip if lang doesn't match last 2 chars of extension
                // (e.g. "fr" in "tutor.frx" or "tutor.fr.txt")
                // Exception: lang="en" matches ".txt" extension
                let lang_matches_ext = nvim_hgr_STRNICMP(lang, ext_ptr, 2) == 0;
                let is_en = nvim_hgr_STRNICMP(lang, c"en".as_ptr(), 2) == 0;
                let is_txt = nvim_hgr_STRNICMP(c"txt".as_ptr(), ext_ptr, 3) == 0;

                if !lang_matches_ext && (!is_en || !is_txt) {
                    fi += 1;
                    continue;
                }
            }

            hgr_search_file(qfl, fname.cast_const(), p_regmatch);
            fi += 1;
        }
        nvim_hgr_free_wild(fcount, fnames);
    }
}

/// Search for a pattern in all help files in 'runtimepath' and add matches
/// to the quickfix list.
///
/// Migrated from `hgr_search_in_rtp` in `quickfix_shim.c`.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T`
/// - `p_regmatch` must be a valid pointer to a `regmatch_T`
/// - `lang` must be a valid C string or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_hgr_search_in_rtp(
    qfl: QfListHandleMut,
    p_regmatch: RegmatchHandle,
    lang: *const c_char,
) {
    let maxpathl = nvim_hgr_get_MAXPATHL();
    let name_buff = nvim_hgr_get_NameBuff();

    // Go through all directories in 'runtimepath'
    let mut p = nvim_hgr_get_p_rtp();
    while *p != 0 && nvim_hgr_get_got_int() == 0 {
        nvim_hgr_copy_option_part(&raw mut p, name_buff, maxpathl);
        hgr_search_files_in_dir(qfl, name_buff, p_regmatch, lang);
    }
}
