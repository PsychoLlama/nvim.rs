//! Vimgrep functions for searching patterns across buffers.
//!
//! This module implements the core vimgrep matching logic, migrated from
//! `quickfix.c` `vgr_match_buflines`.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::{c_char, c_int, c_void, CStr};

// =============================================================================
// Constants (verified with _Static_assert in quickfix.c)
// =============================================================================

const VGR_GLOBAL: c_int = 1;
const VGR_FUZZY: c_int = 4;
const FUZZY_MATCH_MAX_LEN: usize = 1024;
const QF_FAIL: c_int = 0;

// =============================================================================
// Type aliases
// =============================================================================

type LinenrT = i32;
type ColnrT = c_int;

/// Opaque handle to `qf_list_T`
type QfListHandleMut = *mut c_void;
/// Opaque handle to `buf_T`
type BufHandle = *mut c_void;
/// Opaque handle to `win_T`
type WinHandle = *mut c_void;
/// Opaque handle to `regmmatch_T`
type RegmmatchHandle = *mut c_void;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    // Buffer accessors
    fn nvim_buf_get_line_count(buf: BufHandle) -> LinenrT;
    fn nvim_buf_get_fnum(buf: BufHandle) -> c_int;
    fn nvim_ml_get_buf(buf: BufHandle, lnum: LinenrT) -> *mut c_char;
    fn nvim_ml_get_buf_len(buf: BufHandle, lnum: LinenrT) -> ColnrT;

    // Regex accessors
    fn nvim_vim_regexec_multi(
        rm: RegmmatchHandle,
        win: WinHandle,
        buf: BufHandle,
        lnum: LinenrT,
        col: ColnrT,
    ) -> c_int;
    fn nvim_regmatch_startpos_lnum(rm: RegmmatchHandle, idx: c_int) -> LinenrT;
    fn nvim_regmatch_startpos_col(rm: RegmmatchHandle, idx: c_int) -> ColnrT;
    fn nvim_regmatch_endpos_lnum(rm: RegmmatchHandle, idx: c_int) -> LinenrT;
    fn nvim_regmatch_endpos_col(rm: RegmmatchHandle, idx: c_int) -> ColnrT;

    // Fuzzy match
    fn nvim_fuzzy_match(
        str: *const c_char,
        pat: *const c_char,
        matchseq: bool,
        score: *mut c_int,
        matches: *mut u32,
        max_matches: c_int,
    ) -> c_int;

    // Globals
    fn nvim_get_got_int() -> c_int;
    fn nvim_set_got_int(val: c_int);
    fn nvim_get_curwin() -> WinHandle;
    fn nvim_line_breakcheck();

    // Quickfix entry addition (already in Rust, but we call it via FFI
    // since it's in a different module scope within the same crate)
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

// =============================================================================
// Exported functions
// =============================================================================

/// Search for a pattern in all the lines in a buffer and add the matching lines
/// to a quickfix list.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T`
/// - `fname` must be a valid C string or NULL
/// - `buf` must be a valid pointer to a `buf_T`
/// - `spat` must be a valid C string
/// - `regmatch` must be a valid pointer to a `regmmatch_T`
/// - `tomatch` must be a valid pointer to an int
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_vgr_match_buflines(
    qfl: QfListHandleMut,
    fname: *const c_char,
    buf: BufHandle,
    spat: *const c_char,
    regmatch: RegmmatchHandle,
    tomatch: *mut c_int,
    duplicate_name: c_int,
    flags: c_int,
) -> bool {
    let mut found_match = false;

    // Calculate pattern length for fuzzy matching
    let spat_len = CStr::from_ptr(spat).to_bytes().len();
    let pat_len = spat_len.min(FUZZY_MATCH_MAX_LEN);

    let line_count = nvim_buf_get_line_count(buf);
    let buf_fnum = nvim_buf_get_fnum(buf);
    let bufnum = if duplicate_name != 0 { 0 } else { buf_fnum };

    let mut lnum: LinenrT = 1;
    while lnum <= line_count && *tomatch > 0 {
        let mut col: ColnrT = 0;

        if (flags & VGR_FUZZY) == 0 {
            found_match |= vgr_regex_match(
                qfl, fname, buf, regmatch, bufnum, lnum, &mut col, tomatch, flags,
            );
        } else {
            found_match |= vgr_fuzzy_match(
                qfl, fname, buf, spat, bufnum, lnum, &mut col, tomatch, flags, pat_len,
            );
        }

        nvim_line_breakcheck();
        if nvim_get_got_int() != 0 {
            break;
        }

        lnum += 1;
    }

    found_match
}

/// Handle regex matching for a single line.
#[allow(clippy::too_many_arguments)]
unsafe fn vgr_regex_match(
    qfl: QfListHandleMut,
    fname: *const c_char,
    buf: BufHandle,
    regmatch: RegmmatchHandle,
    bufnum: c_int,
    lnum: LinenrT,
    col: &mut ColnrT,
    tomatch: *mut c_int,
    flags: c_int,
) -> bool {
    let mut found_match = false;
    let curwin = nvim_get_curwin();

    while nvim_vim_regexec_multi(regmatch, curwin, buf, lnum, *col) > 0 {
        let start_lnum = nvim_regmatch_startpos_lnum(regmatch, 0);
        let start_col = nvim_regmatch_startpos_col(regmatch, 0);
        let end_lnum = nvim_regmatch_endpos_lnum(regmatch, 0);
        let end_col = nvim_regmatch_endpos_col(regmatch, 0);

        let mesg = nvim_ml_get_buf(buf, start_lnum + lnum);

        if rs_qf_add_entry(
            qfl,
            std::ptr::null_mut(), // dir
            fname,
            std::ptr::null(), // module
            bufnum,
            mesg,
            start_lnum + lnum,
            end_lnum + lnum,
            start_col + 1,
            end_col + 1,
            0,                // vis_col = false
            std::ptr::null(), // pattern
            0,                // nr
            0,                // type
            std::ptr::null(), // user_data
            1,                // valid = true
        ) == QF_FAIL
        {
            nvim_set_got_int(1);
            break;
        }
        found_match = true;
        *tomatch -= 1;
        if *tomatch == 0 {
            break;
        }
        if (flags & VGR_GLOBAL) == 0 || end_lnum > 0 {
            break;
        }
        *col = end_col + ColnrT::from(*col == end_col);
        if *col > nvim_ml_get_buf_len(buf, lnum) {
            break;
        }
    }
    found_match
}

/// Handle fuzzy matching for a single line.
#[allow(clippy::too_many_arguments)]
unsafe fn vgr_fuzzy_match(
    qfl: QfListHandleMut,
    fname: *const c_char,
    buf: BufHandle,
    spat: *const c_char,
    bufnum: c_int,
    lnum: LinenrT,
    col: &mut ColnrT,
    tomatch: *mut c_int,
    flags: c_int,
    pat_len: usize,
) -> bool {
    let mut found_match = false;
    let str = nvim_ml_get_buf(buf, lnum);
    let linelen = nvim_ml_get_buf_len(buf, lnum);
    let mut score: c_int = 0;
    let mut matches = [0u32; FUZZY_MATCH_MAX_LEN];
    let sz = FUZZY_MATCH_MAX_LEN as c_int;

    while nvim_fuzzy_match(
        str.add(*col as usize),
        spat,
        false,
        &raw mut score,
        matches.as_mut_ptr(),
        sz,
    ) > 0
    {
        if rs_qf_add_entry(
            qfl,
            std::ptr::null_mut(), // dir
            fname,
            std::ptr::null(), // module
            bufnum,
            str,
            lnum,
            0,                              // end_lnum
            matches[0] as c_int + *col + 1, // col
            0,                              // end_col
            0,                              // vis_col = false
            std::ptr::null(),               // pattern
            0,                              // nr
            0,                              // type
            std::ptr::null(),               // user_data
            1,                              // valid = true
        ) == QF_FAIL
        {
            nvim_set_got_int(1);
            break;
        }
        found_match = true;
        *tomatch -= 1;
        if *tomatch == 0 {
            break;
        }
        if (flags & VGR_GLOBAL) == 0 {
            break;
        }
        *col = matches[pat_len - 1] as ColnrT + *col + 1;
        if *col > linelen {
            break;
        }
    }
    found_match
}

// =============================================================================
// vgr_process_files migration
// =============================================================================

const OK: c_int = 1;
const FAIL: c_int = 0;

extern "C" {
    // Existing accessors (QfInfoHandle = *const c_void in lib.rs)
    fn nvim_qf_get_curlist(qi: *const c_void) -> *const c_void;
    fn nvim_qf_get_id(qfl: *const c_void) -> u32;

    // New Phase 6 wrappers
    fn nvim_vgr_alloc_dirnames(start_out: *mut *mut c_char, now_out: *mut *mut c_char);
    fn nvim_vgr_free_dirnames(start: *mut c_char, now: *mut c_char);
    fn nvim_vgr_shorten_fname(full_fname: *const c_char) -> *mut c_char;
    fn nvim_vgr_display_fname_wrapper(fname: *const c_char);
    fn nvim_vgr_find_buf(fname: *const c_char, has_mfp: *mut bool) -> BufHandle;
    fn nvim_vgr_load_dummy_buf_wrapper(
        fname: *const c_char,
        dirname_start: *mut c_char,
        dirname_now: *mut c_char,
    ) -> BufHandle;
    fn nvim_vgr_qflist_valid_wrapper(
        wp: WinHandle,
        qi: *mut c_void,
        qfid: u32,
        title: *const c_char,
    ) -> bool;
    fn nvim_vgr_smsg_cannot_open(fname: *const c_char);
    fn nvim_vgr_time_now() -> i64;
    fn nvim_vgr_handle_dummy_buf(
        buf: BufHandle,
        found_match: bool,
        duplicate_name: bool,
        flags: c_int,
        dirname_start: *mut c_char,
        dirname_now: *mut c_char,
        first_match_buf: *mut BufHandle,
        target_dir: *mut *mut c_char,
    );
}

/// Process a list of files for vimgrep pattern matching.
///
/// # Safety
///
/// All pointer parameters must be valid.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn rs_vgr_process_files(
    wp: WinHandle,
    qi: *mut c_void,
    fcount: c_int,
    fnames: *const *const c_char,
    spat: *const c_char,
    regmatch: RegmmatchHandle,
    tomatch: *mut c_int,
    flags: c_int,
    qf_title: *const c_char,
    redraw_for_dummy: *mut bool,
    first_match_buf: *mut BufHandle,
    target_dir: *mut *mut c_char,
) -> c_int {
    let mut save_qfid = nvim_qf_get_id(nvim_qf_get_curlist(qi));

    let mut dirname_start: *mut c_char = std::ptr::null_mut();
    let mut dirname_now: *mut c_char = std::ptr::null_mut();
    nvim_vgr_alloc_dirnames(&raw mut dirname_start, &raw mut dirname_now);

    let mut seconds: i64 = 0;
    let mut status = FAIL;

    'theend: {
        for fi in 0..fcount {
            if nvim_get_got_int() != 0 || *tomatch <= 0 {
                break;
            }

            let fname = nvim_vgr_shorten_fname(*fnames.add(fi as usize));
            let now = nvim_vgr_time_now();
            if now > seconds {
                seconds = now;
                nvim_vgr_display_fname_wrapper(fname);
            }

            let mut has_mfp = false;
            let mut buf = nvim_vgr_find_buf(*fnames.add(fi as usize), &raw mut has_mfp);
            let using_dummy;
            let mut duplicate_name = false;
            if buf.is_null() || !has_mfp {
                duplicate_name = !buf.is_null();
                using_dummy = true;
                *redraw_for_dummy = true;
                buf = nvim_vgr_load_dummy_buf_wrapper(fname, dirname_start, dirname_now);
            } else {
                using_dummy = false;
            }

            // Check whether the quickfix list is still valid.
            if !nvim_vgr_qflist_valid_wrapper(wp, qi, save_qfid, qf_title) {
                break 'theend;
            }

            save_qfid = nvim_qf_get_id(nvim_qf_get_curlist(qi));

            if buf.is_null() {
                if nvim_get_got_int() == 0 {
                    nvim_vgr_smsg_cannot_open(fname);
                }
            } else {
                let found_match = rs_vgr_match_buflines(
                    nvim_qf_get_curlist(qi).cast_mut(),
                    fname,
                    buf,
                    spat,
                    regmatch,
                    tomatch,
                    c_int::from(duplicate_name),
                    flags,
                );

                if using_dummy {
                    nvim_vgr_handle_dummy_buf(
                        buf,
                        found_match,
                        duplicate_name,
                        flags,
                        dirname_start,
                        dirname_now,
                        first_match_buf,
                        target_dir,
                    );
                }
            }
        }
        status = OK;
    }

    nvim_vgr_free_dirnames(dirname_start, dirname_now);
    status
}
