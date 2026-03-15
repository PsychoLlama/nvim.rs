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
/// Opaque handle to `qf_info_T` (mutable)
type QfInfoHandleMut = *mut c_void;
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

// MAXPATHL = 4096 on Linux, but we use the C accessor to get the value
const MAXPATHL: usize = 4096;

extern "C" {
    // Existing accessors (QfInfoHandle = *const c_void in lib.rs)
    fn nvim_qf_get_curlist(qi: *const c_void) -> *const c_void;
    fn nvim_qf_get_id(qfl: *const c_void) -> u32;

    // Phase 3 accessors
    fn nvim_path_try_shorten_fname(full_fname: *const c_char) -> *mut c_char;
    fn nvim_msg_start();
    fn msg_strtrunc(s: *const c_char, force: c_int) -> *mut c_char;
    fn msg_outtrans(str: *const c_char, hl_id: c_int, hist: bool) -> c_int;
    fn msg_clr_eos();
    static mut msg_nowait: bool;
    static mut msg_col: c_int;
    static mut msg_didout: bool;
    // (nvim_msg_strtrunc_free, nvim_msg_outtrans, nvim_msg_set_nowait,
    //  nvim_msg_set_col_zero, nvim_msg_set_didout_false deleted)
    fn nvim_ui_flush();
    // nvim_buf_has_ml_mfp: defined in memline_shim.c as int(buf_T*); returns 0/1
    fn nvim_buf_has_ml_mfp(buf: BufHandle) -> c_int;
    fn nvim_buf_get_mfp_fname(buf: BufHandle) -> *const c_char;
    fn nvim_buf_get_bh_first_char(buf: BufHandle) -> c_char;
    fn nvim_cmdmod_has_cmod_hide() -> bool;
    fn nvim_buf_clear_bf_dummy(buf: BufHandle);
    fn nvim_wipe_dummy_buffer(buf: BufHandle, dirname_start: *mut c_char);
    fn nvim_unload_dummy_buffer(buf: BufHandle, dirname_start: *mut c_char);
    fn nvim_load_dummy_buf(
        fname: *const c_char,
        dirname_start: *mut c_char,
        dirname_now: *mut c_char,
    ) -> BufHandle;
    // nvim_buflist_findname_exp: defined in window_shim.c as buf_T*(const char*)
    fn nvim_buflist_findname_exp(fname: *const c_char) -> BufHandle;
    fn nvim_apply_filetype_autocmds_and_modelines(buf: BufHandle);
    fn nvim_ex_cd_arg(arg: *mut c_char, is_lcd: bool);
    fn smsg(hl_id: c_int, fmt: *const c_char, ...) -> c_int;
    fn emsg(msg: *const std::ffi::c_char) -> bool;
    // (nvim_vgr_smsg_cannot_open, nvim_qf_emsg_ll_changed deleted: use smsg/emsg directly)
    fn nvim_os_dirname(buf: *mut c_char, size: c_int);
    // jump
    fn rs_qf_jump_newwin(qi: *mut c_void, dir: c_int, errornr: c_int, forceit: c_int, newwin: bool);
    // qf_list ops (for vgr_qflist_valid inline)
    fn rs_qf_new_list(qi: *mut c_void, title: *const c_char);
    fn rs_qf_restore_list(qi: *mut c_void, save_qfid: u32) -> c_int;
    fn rs_qflist_valid(wp: WinHandle, qf_id: u32) -> bool;
    // curbuf accessor
    fn nvim_qf_get_curbuf() -> BufHandle;
    fn nvim_qf_curbuf_is(buf: *const c_void) -> bool;
    // xstrdup / xfree (for target_dir)
    fn nvim_xstrdup(s: *const c_char) -> *mut c_char;
}

/// Inline of `vgr_display_fname`: display filename in the status area during vimgrep.
unsafe fn vgr_display_fname_rust(fname: *mut c_char) {
    nvim_msg_start();
    let p = msg_strtrunc(fname, 1);
    if p.is_null() {
        msg_outtrans(fname, 0, false);
    } else {
        msg_outtrans(p, 0, false);
        nvim_xfree(p.cast());
    }
    msg_clr_eos();
    msg_didout = false;
    msg_nowait = true;
    msg_col = 0;
    nvim_ui_flush();
}

/// Inline of `existing_swapfile`: check if `buf` has a non-`.swp` swap file.
unsafe fn existing_swapfile_rust(buf: BufHandle) -> bool {
    let fname = nvim_buf_get_mfp_fname(buf);
    if fname.is_null() {
        return false;
    }
    let s = std::ffi::CStr::from_ptr(fname).to_bytes();
    let len = s.len();
    if len < 2 {
        return false;
    }
    s[len - 1] != b'p' || s[len - 2] != b'w'
}

/// Inline of `vgr_qflist_valid`: verify the quickfix list survived an autocmd.
unsafe fn vgr_qflist_valid_rust(
    wp: WinHandle,
    qi: *mut c_void,
    qfid: u32,
    title: *const c_char,
) -> bool {
    if !rs_qflist_valid(wp, qfid) {
        if !wp.is_null() {
            emsg(c"E926: Current location list was changed".as_ptr());
            return false;
        }
        rs_qf_new_list(qi, title);
        return true;
    }
    rs_qf_restore_list(qi, qfid) != FAIL
}

/// Inline of `nvim_vgr_handle_dummy_buf`: post-search dummy buffer disposition.
#[allow(clippy::too_many_arguments)]
unsafe fn handle_dummy_buf_rust(
    buf: BufHandle,
    found_match: bool,
    duplicate_name: bool,
    flags: c_int,
    dirname_start: *mut c_char,
    dirname_now: *mut c_char,
    first_match_buf: *mut BufHandle,
    target_dir: *mut *mut c_char,
) {
    if found_match && (*first_match_buf).is_null() {
        *first_match_buf = buf;
    }

    if duplicate_name {
        nvim_wipe_dummy_buffer(buf, dirname_start);
        return;
    }

    let bh0 = nvim_buf_get_bh_first_char(buf) as u8;
    if !nvim_cmdmod_has_cmod_hide() || bh0 == b'u' || bh0 == b'w' || bh0 == b'd' {
        if !found_match {
            nvim_wipe_dummy_buffer(buf, dirname_start);
            return;
        }
        if buf != *first_match_buf || (flags & VGR_NOJUMP) != 0 || existing_swapfile_rust(buf) {
            nvim_unload_dummy_buffer(buf, dirname_start);
            nvim_buf_clear_bf_dummy(buf);
            return;
        }
    }

    // Buffer is kept loaded.
    nvim_buf_clear_bf_dummy(buf);

    if buf == *first_match_buf
        && (*target_dir).is_null()
        && !libc_strcmp_ne(dirname_start, dirname_now)
    {
        *target_dir = nvim_xstrdup(dirname_now);
    }

    nvim_apply_filetype_autocmds_and_modelines(buf);
}

/// Compare two C strings; returns `true` if they differ.
#[allow(clippy::missing_const_for_fn)]
unsafe fn libc_strcmp_ne(a: *const c_char, b: *const c_char) -> bool {
    // Manual byte comparison to avoid pulling in libc crate
    let mut pa = a;
    let mut pb = b;
    loop {
        let ca = *pa;
        let cb = *pb;
        if ca != cb {
            return true;
        }
        if ca == 0 {
            return false;
        }
        pa = pa.add(1);
        pb = pb.add(1);
    }
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

    // Allocate directory name buffers (replaces nvim_vgr_alloc_dirnames)
    let mut dirname_start = vec![0u8; MAXPATHL];
    let mut dirname_now = vec![0u8; MAXPATHL];
    nvim_os_dirname(
        dirname_start.as_mut_ptr().cast::<c_char>(),
        MAXPATHL as c_int,
    );

    let mut seconds: u64 = 0;
    let mut status = FAIL;

    'theend: {
        for fi in 0..fcount {
            if nvim_get_got_int() != 0 || *tomatch <= 0 {
                break;
            }

            let fname = nvim_path_try_shorten_fname(*fnames.add(fi as usize));

            // Display fname once per second (replaces nvim_vgr_time_now)
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            if now > seconds {
                seconds = now;
                vgr_display_fname_rust(fname);
            }

            // Find buffer (replaces nvim_vgr_find_buf)
            let found_buf = nvim_buflist_findname_exp(*fnames.add(fi as usize));
            let has_mfp = !found_buf.is_null() && nvim_buf_has_ml_mfp(found_buf) != 0;
            let mut buf = found_buf;
            let using_dummy;
            let mut duplicate_name = false;
            if buf.is_null() || !has_mfp {
                duplicate_name = !buf.is_null();
                using_dummy = true;
                *redraw_for_dummy = true;
                buf = nvim_load_dummy_buf(
                    fname,
                    dirname_start.as_mut_ptr().cast::<c_char>(),
                    dirname_now.as_mut_ptr().cast::<c_char>(),
                );
            } else {
                using_dummy = false;
            }

            // Check whether the quickfix list is still valid (inlines vgr_qflist_valid).
            if !vgr_qflist_valid_rust(wp, qi, save_qfid, qf_title) {
                break 'theend;
            }

            save_qfid = nvim_qf_get_id(nvim_qf_get_curlist(qi));

            if buf.is_null() {
                if nvim_get_got_int() == 0 {
                    smsg(0, c"Cannot open file \"%s\"".as_ptr(), fname);
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
                    handle_dummy_buf_rust(
                        buf,
                        found_match,
                        duplicate_name,
                        flags,
                        dirname_start.as_mut_ptr().cast::<c_char>(),
                        dirname_now.as_mut_ptr().cast::<c_char>(),
                        first_match_buf,
                        target_dir,
                    );
                }
            }
        }
        status = OK;
    }

    // dirname_start/dirname_now freed by Rust Vec drop
    status
}

// =============================================================================
// ex_vimgrep migration
// =============================================================================

/// Opaque handle to `exarg_T`
type EapHandle = *mut c_void;

// CMD constants for vimgrep command variants (verified by _Static_assert in quickfix_shim.c)
const CMD_GREPADD: c_int = 173;
const CMD_LGREPADD: c_int = 240;
const CMD_VIMGREPADD: c_int = 510;
const CMD_LVIMGREPADD: c_int = 268;

// VGR_NOJUMP flag constant (verified by _Static_assert in quickfix_shim.c)
const VGR_NOJUMP: c_int = 2;

#[allow(clashing_extern_declarations)]
extern "C" {
    // eap accessors
    fn nvim_eap_get_cmdlinep_deref_make(eap: EapHandle) -> *mut c_char;
    fn nvim_eap_get_addr_count(eap: EapHandle) -> c_int;
    // linenr_T = int32_t = c_int; commands.rs incorrectly declares as i64
    fn nvim_eap_get_line2(eap: EapHandle) -> c_int;
    fn nvim_eap_get_arg(eap: EapHandle) -> *mut c_char;
    fn nvim_eap_get_cmdidx(eap: EapHandle) -> c_int;
    fn nvim_eap_get_forceit(eap: EapHandle) -> bool;

    // Pattern parsing
    fn rs_skip_vimgrep_pat(p: *mut c_char, s: *mut *mut c_char, flags: *mut c_int) -> *mut c_char;

    // qf_cmdtitle: build ":cmdline" title string into buf
    fn rs_qf_cmdtitle(cmd: *const c_char, buf: *mut c_char, bufsz: usize) -> usize;

    // Stack / list management
    fn rs_qf_cmd_get_or_alloc_stack(eap: EapHandle, pwinp: *mut WinHandle) -> QfInfoHandleMut;
    fn rs_qf_stack_empty(qi: *const c_void) -> bool;
    // rs_qf_new_list, rs_qflist_valid, rs_qf_restore_list: declared in process_files extern block above
    fn rs_qf_list_empty(qfl: *const c_void) -> bool;
    fn rs_qf_update_buffer(qi: QfInfoHandleMut, old_last: *const c_void);
    fn rs_foldUpdateAll(win: WinHandle);

    // regmmatch_T heap management (Phase 1 new accessors)
    fn nvim_vgr_regcomp_init(pat: *const c_char) -> RegmmatchHandle;
    fn nvim_vgr_regmatch_free(rm: RegmmatchHandle);

    // get_arglist_exp wrapper (Phase 1)
    fn nvim_vgr_get_arglist_exp(
        p: *const c_char,
        fcount_out: *mut c_int,
        fnames_out: *mut *mut *mut c_char,
    ) -> c_int;
    fn nvim_vgr_free_wild_raw(fcount: c_int, fnames: *mut *mut c_char);

    // Error message wrappers for parse
    // emsg declared in the second extern block
    // (nvim_vgr_emsg_invalpat, nvim_vgr_emsg_no_filename, nvim_vgr_emsg_nomatch deleted: use emsg directly)
    fn skipwhite(s: *const c_char) -> *const c_char;

    // xstrdup / xfree
    fn nvim_xfree(ptr: *mut c_void);
    fn xstrdup(s: *const c_char) -> *mut c_char;

    // Pre-check inlining (Phase 2)
    fn nvim_check_can_set_curbuf_forceit(forceit: c_int) -> bool;
    fn rs_vgr_get_auname(cmdidx: c_int) -> *const c_char;
    fn nvim_apply_autocmds_quickfixcmdpre(au_name: *const c_char) -> bool;
    fn nvim_apply_autocmds_quickfixcmdpost(au_name: *const c_char) -> bool;
    fn nvim_aborting() -> bool;

    // Finalize inlining (Phase 2)
    fn nvim_qf_get_curlist_mut(qi: QfInfoHandleMut) -> *mut c_void;
    fn nvim_qf_get_start(qfl: *const c_void) -> *const c_void;
    fn nvim_qf_set_nonevalid(qfl: *mut c_void, nonevalid: bool);
    fn nvim_qf_set_ptr(qfl: *mut c_void, ptr: *const c_void);
    fn nvim_qf_set_index(qfl: *mut c_void, idx: c_int);

    fn semsg(fmt: *const std::ffi::c_char, ...) -> bool;
    // (nvim_semsg_nomatch2 deleted: use semsg directly)

    // busy counter
    #[link_name = "rs_incr_quickfix_busy"]
    fn nvim_incr_quickfix_busy();
    #[link_name = "rs_decr_quickfix_busy"]
    fn nvim_decr_quickfix_busy();
}

/// Native Rust representation of the `:vimgrep` argument state.
///
/// Replaces the C `vgr_args_T` heap allocation from Phase 1.
struct VgrArgs {
    tomatch: c_int,
    /// Borrowed pointer into `eap->arg`; NOT owned.
    spat: *mut c_char,
    flags: c_int,
    /// Owned by C's `get_arglist_exp`; freed by `FreeWild` via `nvim_vgr_free_wild_raw`.
    fnames: *mut *mut c_char,
    fcount: c_int,
    /// Heap-allocated `regmmatch_T`; freed by `nvim_vgr_regmatch_free`.
    regmatch: RegmmatchHandle,
    /// Owned (`xstrdup`'d) quickfix list title.
    qf_title: *mut c_char,
}

impl VgrArgs {
    /// Parse `:vimgrep` arguments from `eap`.
    ///
    /// Returns `Ok(Self)` on success, `Err(())` if an error was emitted.
    ///
    /// # Safety
    ///
    /// `eap` must be a valid pointer to an `exarg_T`.
    unsafe fn parse(eap: EapHandle) -> Result<Self, ()> {
        // Build qf_title from cmdlinep
        let cmdline = nvim_eap_get_cmdlinep_deref_make(eap);
        let mut title_buf = [0u8; 1025]; // IOSIZE
        rs_qf_cmdtitle(
            cmdline,
            title_buf.as_mut_ptr().cast::<c_char>(),
            title_buf.len(),
        );
        let qf_title = xstrdup(title_buf.as_ptr().cast::<c_char>());

        // tomatch: line2 if addr_count > 0, else MAXLNUM
        let tomatch: c_int = if nvim_eap_get_addr_count(eap) > 0 {
            nvim_eap_get_line2(eap)
        } else {
            0x7fff_ffff // MAXLNUM
        };

        // Parse pattern and flags from eap->arg
        let arg = nvim_eap_get_arg(eap);
        let mut spat: *mut c_char = std::ptr::null_mut();
        let mut flags: c_int = 0;
        let p = rs_skip_vimgrep_pat(arg, &raw mut spat, &raw mut flags);
        if p.is_null() {
            emsg(c"E682: Invalid search pattern or delimiter".as_ptr());
            nvim_xfree(qf_title.cast());
            return Err(());
        }

        // Compile regex from spat
        let regmatch = nvim_vgr_regcomp_init(spat);
        if regmatch.is_null() {
            // error already emitted inside nvim_vgr_regcomp_init
            nvim_xfree(qf_title.cast());
            return Err(());
        }

        // Check that file name is present after pattern
        let p = skipwhite(p);
        if *p == 0 {
            emsg(c"E683: File name missing or invalid pattern".as_ptr());
            nvim_vgr_regmatch_free(regmatch);
            nvim_xfree(qf_title.cast());
            return Err(());
        }

        // Expand file list
        let mut fcount: c_int = 0;
        let mut fnames: *mut *mut c_char = std::ptr::null_mut();
        if nvim_vgr_get_arglist_exp(p, &raw mut fcount, &raw mut fnames) == FAIL || fcount == 0 {
            emsg(c"E479: No match".as_ptr());
            nvim_vgr_regmatch_free(regmatch);
            nvim_xfree(qf_title.cast());
            return Err(());
        }

        Ok(Self {
            tomatch,
            spat,
            flags,
            fnames,
            fcount,
            regmatch,
            qf_title,
        })
    }

    /// Free the file list (`FreeWild`). Safe to call multiple times.
    unsafe fn free_wild(&mut self) {
        if !self.fnames.is_null() {
            nvim_vgr_free_wild_raw(self.fcount, self.fnames);
            self.fnames = std::ptr::null_mut();
            self.fcount = 0;
        }
    }

    /// Free `regmatch` and `qf_title` (but not `fnames` - call `free_wild` separately).
    /// Safe to call after `free_wild` has already been called.
    unsafe fn cleanup(&mut self) {
        if !self.regmatch.is_null() {
            nvim_vgr_regmatch_free(self.regmatch);
            self.regmatch = std::ptr::null_mut();
        }
        if !self.qf_title.is_null() {
            nvim_xfree(self.qf_title.cast());
            self.qf_title = std::ptr::null_mut();
        }
        // Also free fnames if not yet freed
        self.free_wild();
    }
}

/// `:vimgrep`, `:vimgrepadd`, `:lvimgrep`, `:lvimgrepadd` command.
///
/// # Safety
///
/// `eap` must be a valid pointer to an `exarg_T`.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_vimgrep(eap: EapHandle) {
    // Phase 2: inline nvim_vgr_pre_check
    let forceit = nvim_eap_get_forceit(eap);
    if !nvim_check_can_set_curbuf_forceit(c_int::from(forceit)) {
        return;
    }
    let cmdidx = nvim_eap_get_cmdidx(eap);
    let au_name = rs_vgr_get_auname(cmdidx);
    if !au_name.is_null() && nvim_apply_autocmds_quickfixcmdpre(au_name) && nvim_aborting() {
        return;
    }

    // Get or allocate the quickfix stack
    let mut wp: WinHandle = std::ptr::null_mut();
    let qi = rs_qf_cmd_get_or_alloc_stack(eap, &raw mut wp);

    // Parse arguments into native Rust struct
    let Ok(mut args) = VgrArgs::parse(eap) else {
        return;
    };

    // Create new list unless this is an "add" command with an existing stack
    if (cmdidx != CMD_GREPADD
        && cmdidx != CMD_LGREPADD
        && cmdidx != CMD_VIMGREPADD
        && cmdidx != CMD_LVIMGREPADD)
        || rs_qf_stack_empty(qi)
    {
        rs_qf_new_list(qi, args.qf_title);
    }

    let mut target_dir: *mut c_char = std::ptr::null_mut();

    nvim_incr_quickfix_busy();

    let mut tomatch = args.tomatch;
    let mut redraw_for_dummy = false;
    let mut first_match_buf: BufHandle = std::ptr::null_mut();
    let status = rs_vgr_process_files(
        wp,
        qi,
        args.fcount,
        args.fnames.cast(),
        args.spat,
        args.regmatch,
        &raw mut tomatch,
        args.flags,
        args.qf_title,
        &raw mut redraw_for_dummy,
        &raw mut first_match_buf,
        &raw mut target_dir,
    );

    if status != OK {
        args.free_wild();
        nvim_decr_quickfix_busy();
        args.cleanup();
        nvim_xfree(target_dir.cast());
        return;
    }

    args.free_wild();

    // Phase 2: inline nvim_vgr_finalize_list
    {
        let qfl = nvim_qf_get_curlist_mut(qi);
        nvim_qf_set_nonevalid(qfl, false);
        let start = nvim_qf_get_start(qfl);
        nvim_qf_set_ptr(qfl, start);
        nvim_qf_set_index(qfl, 1);
        crate::rs_qf_incr_changedtick(qfl.cast());
        rs_qf_update_buffer(qi, std::ptr::null());
    }

    // Remember current qfid for validation after autocmd
    let save_qfid = nvim_qf_get_id(nvim_qf_get_curlist(qi));

    // Phase 2: inline nvim_vgr_post_autocmd
    {
        let au_name_post = rs_vgr_get_auname(cmdidx);
        if !au_name_post.is_null() {
            nvim_apply_autocmds_quickfixcmdpost(au_name_post);
        }
    }

    // Phase 2: inline nvim_vgr_list_still_valid
    if !rs_qflist_valid(wp, save_qfid) || rs_qf_restore_list(qi, save_qfid) == FAIL {
        nvim_decr_quickfix_busy();
        args.cleanup();
        nvim_xfree(target_dir.cast());
        return;
    }

    // Save fields before cleanup (jump needs flags and spat)
    let flags = args.flags;
    let spat = args.spat;

    // Phase 3: inline vgr_jump_or_nomatch + vgr_jump_to_match
    {
        let qfl = nvim_qf_get_curlist(qi);
        if rs_qf_list_empty(qfl) {
            semsg(c"E480: No match: %s".as_ptr(), spat);
        } else if (flags & VGR_NOJUMP) == 0 {
            // Inline vgr_jump_to_match:
            let buf_before = nvim_qf_get_curbuf();
            rs_qf_jump_newwin(qi, 0, 0, c_int::from(forceit), false);
            if !nvim_qf_curbuf_is(buf_before) {
                // Jumped to another buffer; redraw already handled.
                redraw_for_dummy = false;
            }
            // Jump to the directory used after loading the buffer.
            if nvim_qf_curbuf_is(first_match_buf) && !target_dir.is_null() {
                nvim_ex_cd_arg(target_dir, true);
            }
        }
    }

    nvim_decr_quickfix_busy();

    // Phase 2: inline nvim_vgr_foldUpdateAll_curwin
    if redraw_for_dummy {
        rs_foldUpdateAll(nvim_get_curwin());
    }

    // theend cleanup
    args.cleanup();
    nvim_xfree(target_dir.cast());
}
