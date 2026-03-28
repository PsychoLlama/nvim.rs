//! Ex command implementations: `ex_make`, `ex_cfile`, `ex_cbuffer`, `ex_cexpr`.
//! Also includes helpers: `make_get_fullcmd`, `get_mef_name`.
//! And wrapper implementations: `copy_loclist_stack`.
//!
//! Phase 7 of the quickfix C-to-Rust migration.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_safety_doc)]
#![allow(clashing_extern_declarations)]

use std::ffi::{c_char, c_int, c_void, CStr, CString};

// =============================================================================
// Type aliases
// =============================================================================

type EapHandle = *mut c_void;
type QfInfoHandleMut = *mut c_void;
type WinHandle = *mut c_void;
type BufHandle = *mut c_void;
type LinenrT = i32;

// CMD_* constants (validated by _Static_assert in quickfix_shim.c)
const CMD_MAKE: c_int = 273;
const CMD_LMAKE: c_int = 248;
const CMD_GREPADD: c_int = 173;
const CMD_LGREPADD: c_int = 240;

const CMD_CADDFILE: c_int = 51;
const CMD_LADDFILE: c_int = 218;
const CMD_CFILE: c_int = 65;
const CMD_LFILE: c_int = 233;

const CMD_CADDBUFFER: c_int = 49;
const CMD_LADDBUFFER: c_int = 217;
const CMD_CBUFFER: c_int = 55;
const CMD_LBUFFER: c_int = 221;

const CMD_CADDEXPR: c_int = 50;
const CMD_LADDEXPR: c_int = 216;
const CMD_CEXPR: c_int = 64;
const CMD_LEXPR: c_int = 232;

// typval v_type values (from eval/typval_defs.h VAR_*)
const VAR_STRING: c_int = 1;

// Autocmd event constants (from auevents_enum.generated.h, validated by _Static_assert)
const EVENT_QUICKFIXCMDPRE: c_int = 89;
const EVENT_QUICKFIXCMDPOST: c_int = 88;

// =============================================================================
// External C function declarations
// =============================================================================

extern "C" {
    // Option globals (direct access)
    static p_shq: *const c_char;
    static p_sp: *const c_char;
    static p_mef: *const c_char;
    static p_efm: *const c_char;
    static p_menc: *const c_char;
    static p_gefm: *const c_char;
    static p_ef: *const c_char;

    // curbuf option accessors (struct field access - retained as opaque accessors)
    fn nvim_curbuf_get_b_p_menc() -> *const c_char;
    fn nvim_curbuf_get_b_p_gefm() -> *const c_char;

    // Shell/message helpers
    fn append_redir(buf: *mut c_char, buflen: usize, opt: *const c_char, name: *const c_char);
    static mut msg_col: c_int;
    static mut msg_didout: bool;
    #[link_name = "msg_start"]
    fn nvim_msg_start();
    fn msg_puts(s: *const c_char);
    fn msg_outtrans(str: *const c_char, hl_id: c_int, hist: bool) -> c_int;
    // (nvim_msg_puts_colon_bang, nvim_msg_outtrans_cmd deleted: use msg_puts/msg_outtrans directly)

    // autowrite, shell, remove
    fn autowrite_all();
    fn do_shell(cmd: *const c_char, flags: c_int);
    #[link_name = "os_remove"]
    fn nvim_os_remove(path: *const c_char) -> c_int;

    // OS helpers for get_mef_name
    fn os_get_pid() -> i64;
    fn nvim_os_fileinfo_link_exists(name: *const c_char) -> bool;
    fn emsg(msg: *const std::ffi::c_char) -> bool;
    // (nvim_emsg_notmp deleted: use emsg directly)
    fn vim_tempname() -> *mut c_char;

    // Memory
    fn nvim_xmalloc(size: usize) -> *mut c_void;
    fn nvim_xfree(ptr: *mut c_void);
    fn nvim_xstrdup(s: *const c_char) -> *mut c_char;

    // qf_cmdtitle / eap accessors
    fn nvim_eap_get_cmdlinep_deref_make(eap: EapHandle) -> *mut c_char;
    fn nvim_eap_get_cmdidx(eap: EapHandle) -> c_int;
    fn nvim_eap_get_forceit(eap: EapHandle) -> bool;
    fn nvim_eap_get_arg(eap: EapHandle) -> *mut c_char;
    fn nvim_eap_get_addr_count(eap: EapHandle) -> c_int;
    fn nvim_eap_get_line1(eap: EapHandle) -> LinenrT;
    fn nvim_eap_get_line2(eap: EapHandle) -> LinenrT;

    // rs_qf_cmdtitle
    fn rs_qf_cmdtitle(cmd: *const c_char, buf: *mut c_char, bufsz: usize) -> usize;

    // Quickfix init / jump / valid
    fn rs_qf_init_ext(
        qi: QfInfoHandleMut,
        qf_idx: c_int,
        efile: *const c_char,
        buf: BufHandle,
        tv: *mut c_void,
        errorformat: *mut c_char,
        newlist: bool,
        lnumfirst: LinenrT,
        lnumlast: LinenrT,
        qf_title: *const c_char,
        enc: *mut c_char,
    ) -> c_int;
    fn rs_qf_jump_first(qi: QfInfoHandleMut, save_qfid: u32, forceit: c_int);
    fn rs_qflist_valid(wp: WinHandle, qf_id: u32) -> bool;
    fn rs_qf_stack_empty(qi: *const c_void) -> bool;
    #[link_name = "grep_internal"]
    fn rs_grep_internal(cmdidx: c_int) -> c_int;
    fn rs_ex_vimgrep(eap: EapHandle);
    fn rs_make_get_auname(cmdidx: c_int) -> *const c_char;
    fn rs_cfile_get_auname(cmdidx: c_int) -> *const c_char;
    fn rs_cbuffer_get_auname(cmdidx: c_int) -> *const c_char;
    fn rs_cexpr_get_auname(cmdidx: c_int) -> *const c_char;

    // Stack management
    fn rs_qf_cmd_get_or_alloc_stack(eap: EapHandle, pwinp: *mut WinHandle) -> QfInfoHandleMut;
    fn rs_ll_get_or_alloc_list(wp: WinHandle) -> QfInfoHandleMut;
    fn nvim_get_ql_info() -> QfInfoHandleMut;
    fn nvim_get_curwin() -> WinHandle;
    fn is_loclist_cmd(cmdidx: c_int) -> bool;

    // List management
    fn nvim_qf_get_curlist_mut(qi: QfInfoHandleMut) -> *mut c_void;
    #[link_name = "rs_incr_quickfix_busy"]
    fn nvim_incr_quickfix_busy();
    #[link_name = "rs_decr_quickfix_busy"]
    fn nvim_decr_quickfix_busy();

    // curlist accessors
    fn nvim_qf_get_curlist_idx(qi: QfInfoHandleMut) -> c_int;
    fn nvim_qf_get_id(qfl: *const c_void) -> u32;

    // set_option_direct for cfile
    fn nvim_set_option_direct_ef(val: *const c_char);

    // buf accessors for cbuffer_process_args
    fn nvim_buf_has_ml_mfp_void(buf: *const c_void) -> bool;
    fn nvim_buf_get_ml_line_count_void(buf: *const c_void) -> LinenrT;
    fn nvim_buf_get_sfname_void(buf: *const c_void) -> *const c_char;
    fn nvim_buflist_findnr_ptr(nr: c_int) -> *mut c_void;
    static mut curbuf: *mut c_void;
    fn skipdigits(s: *const c_char) -> *mut c_char;
    // (nvim_emsg_invarg, nvim_emsg_buf_not_loaded, nvim_emsg_invrange deleted: use emsg directly)
    // emsg declared earlier in this file
    fn nvim_eap_set_line1(eap: EapHandle, lnum: LinenrT);
    fn nvim_eap_set_line2(eap: EapHandle, lnum: LinenrT);

    // eval_expr / tv_free for ex_cexpr
    fn nvim_eval_expr(arg: *const c_char, eap: EapHandle) -> *mut c_void;
    fn nvim_tv_get_type_void(tv: *const c_void) -> c_int;
    fn nvim_tv_get_vval_string(tv: *const c_void) -> *const c_char;
    fn nvim_tv_is_list(tv: *const c_void) -> bool;
    fn nvim_tv_free_void(tv: *mut c_void);
    // (nvim_emsg_e777 deleted: use emsg directly)
    // emsg declared earlier in this file

    // autocmd wrappers → deleted; Rust inlines apply_autocmds + aborting directly
    fn apply_autocmds(
        event: c_int,
        fname: *mut c_char,
        fname_io: *mut c_char,
        force: bool,
        buf: *mut c_void,
    ) -> bool;
    fn aborting() -> bool;
    fn nvim_qf_buf_get_fname(buf: *const c_void) -> *const c_char;

    // IObuff helpers for cbuffer
    fn nvim_qf_snprintf_iobuff(title: *const c_char, sfname: *const c_char);
    static IObuff: *mut c_char;

    // GET_LOC_LIST (same as nvim_win_get_loclist in lib.rs)
    fn nvim_win_get_loclist(wp: *const c_void) -> *const c_void;

    // copy_loclist_stack helpers
    fn nvim_win_get_llist_or_ref(from_win: *const c_void) -> *mut c_void;
    fn nvim_win_set_llist(to_win: *mut c_void, qi: *mut c_void);
    // nvim_win_get_p_lhi already declared in lifecycle.rs (returns c_int)
    fn nvim_qf_get_listcount(qi: *const c_void) -> c_int;
    fn nvim_qf_set_listcount(qi: *mut c_void, n: c_int);
    // nvim_qf_get_curlist_idx already declared above (line 141)
    fn nvim_qf_set_curlist_idx(qi: *mut c_void, n: c_int);
    #[link_name = "nvim_qf_get_list_at"]
    fn nvim_qi_get_list_qi(qi: *mut c_void, idx: c_int) -> *mut c_void;
    fn nvim_qf_get_maxcount(qi: *const c_void) -> c_int;
    fn nvim_qf_free_all_win(to_win: *mut c_void);
    fn nvim_win_set_p_lhi(win: *mut c_void, v: c_int);
    fn nvim_win_get_p_lhi(win: *const c_void) -> c_int;

    // qf_alloc_stack (via lifecycle.rs export)
    fn rs_qf_alloc_stack(qfl_type: c_int, maxcount: c_int) -> *mut c_void;

    // copy_loclist per-list
    fn rs_copy_loclist(from: *mut c_void, to: *mut c_void) -> c_int;

    // libc
    fn strlen(s: *const c_char) -> usize;
    fn snprintf(buf: *mut c_char, size: usize, fmt: *const c_char, ...) -> c_int;
    fn atoi(s: *const c_char) -> c_int;
}

// =============================================================================
// Constants
// =============================================================================

const OK: c_int = 1;
const FAIL: c_int = 0;

// qf_info_T.qfl_type == QFLT_LOCATION for location lists
const QFLT_LOCATION: c_int = 1;

// =============================================================================
// Helper: build qf_cmdtitle string into a local buffer
// =============================================================================

/// Write `:cmd` into a stack-allocated buffer and return pointer to it.
/// The returned pointer points into `buf`, which must outlive the use.
unsafe fn make_cmdtitle(cmd: *const c_char, buf: &mut [u8]) -> *const c_char {
    rs_qf_cmdtitle(cmd, buf.as_mut_ptr().cast::<c_char>(), buf.len());
    buf.as_ptr().cast::<c_char>()
}

// =============================================================================
// Phase 1: get_mef_name — generate error file name from 'makeef' or tempfile
// =============================================================================

/// Static state for `get_mef_name` (matches the C `static int start / off`).
static MEF_START: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(-1);
static MEF_OFF: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

/// Return the name for the errorfile, in allocated memory (xmalloc).
/// Find a new unique name when 'makeef' contains "##".
/// Returns NULL on error.
///
/// # Safety
///
/// Calls many C functions. Must only be called from the main nvim thread.
#[no_mangle]
#[allow(clippy::missing_panics_doc)] // CString::new on integer-only string cannot fail
pub unsafe extern "C" fn rs_get_mef_name() -> *mut c_char {
    debug_assert!(!p_mef.is_null());

    // If 'makeef' is empty, use a temp file.
    if *p_mef == 0 {
        let name = vim_tempname();
        if name.is_null() {
            emsg(c"E483: Can't get temp file name".as_ptr());
        }
        return name;
    }

    // Find "##" in p_mef.
    let mef_str = CStr::from_ptr(p_mef);
    let mef_bytes = mef_str.to_bytes();

    // Find position of "##"
    let hash_pos = mef_bytes.windows(2).position(|w| w == b"##");

    let Some(hash_pos) = hash_pos else {
        // No "##" -- use p_mef directly (xstrdup).
        return nvim_xstrdup(p_mef);
    };

    // Keep trying until the name doesn't exist yet.
    loop {
        let start_val = MEF_START.load(std::sync::atomic::Ordering::Relaxed);
        let start = if start_val == -1 {
            let pid = os_get_pid() as c_int;
            MEF_START.store(pid, std::sync::atomic::Ordering::Relaxed);
            pid
        } else {
            let new_off = MEF_OFF.fetch_add(19, std::sync::atomic::Ordering::Relaxed) + 19;
            MEF_OFF.store(new_off, std::sync::atomic::Ordering::Relaxed);
            start_val
        };
        let off_val = MEF_OFF.load(std::sync::atomic::Ordering::Relaxed);

        let mef_len = mef_bytes.len();
        // Allocate: mef_len + 30 bytes (enough for two integers).
        let name_buf_size = mef_len + 30;
        let name: *mut c_char = nvim_xmalloc(name_buf_size).cast::<c_char>();

        // Copy p_mef up to "##"
        std::ptr::copy_nonoverlapping(p_mef, name, mef_len + 1); // include NUL

        // Overwrite from hash_pos with "startoff\0" + rest of p_mef after "##"
        let write_ptr = name.add(hash_pos);
        // Compute the digits string
        let digits = format!("{start}{off_val}");
        let digits_c = CString::new(digits).unwrap();
        let digits_len = strlen(digits_c.as_ptr());
        // Write digits
        std::ptr::copy_nonoverlapping(digits_c.as_ptr(), write_ptr, digits_len);
        // Append the tail (p_mef after "##")
        let tail_ptr = p_mef.add(hash_pos + 2);
        let tail_len = strlen(tail_ptr);
        std::ptr::copy_nonoverlapping(tail_ptr, write_ptr.add(digits_len), tail_len + 1);

        // Don't accept a symbolic link (security risk).
        if !nvim_os_fileinfo_link_exists(name) {
            return name;
        }
        nvim_xfree(name.cast::<c_void>());
    }
}

// =============================================================================
// Phase 1: make_get_fullcmd — build the make/grep shell command string
// =============================================================================

/// Build the full shell command: `<shq><makecmd><shq>[ <sp> <fname>]`.
/// Displays the command via msg.
/// Returns xmalloc'd string; caller must xfree.
///
/// # Safety
///
/// `makecmd` and `fname` must be valid non-null C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_make_get_fullcmd(
    makecmd: *const c_char,
    fname: *const c_char,
) -> *mut c_char {
    let shq_len = strlen(p_shq);
    let makecmd_len = strlen(makecmd);

    let mut len = shq_len * 2 + makecmd_len + 1;
    if *p_sp != 0 {
        len += strlen(p_sp) + strlen(fname) + 3;
    }

    let cmd: *mut c_char = nvim_xmalloc(len).cast::<c_char>();

    // cmd = p_shq + makecmd + p_shq
    let fmt = c"%s%s%s".as_ptr();
    snprintf(cmd, len, fmt, p_shq, makecmd, p_shq);

    // If 'shellpipe' non-empty, redirect to fname.
    if *p_sp != 0 {
        append_redir(cmd, len, p_sp, fname);
    }

    // Display the command. If cursor is at column 0 reset msg_didout.
    if msg_col == 0 {
        msg_didout = false;
    }
    nvim_msg_start();
    msg_puts(c":!".as_ptr());
    msg_outtrans(cmd, 0, false);

    cmd
}

// =============================================================================
// Phase 1: ex_make
// =============================================================================

/// `:make`, `:lmake`, `:grep`, `:lgrep`, `:grepadd`, `:lgrepadd` handler.
///
/// # Safety
///
/// `eap` must be a valid pointer to a C `exarg_T`.
#[allow(clippy::too_many_lines)]
#[export_name = "ex_make"]
pub unsafe extern "C" fn rs_ex_make(eap: EapHandle) {
    // Redirect ":grep" to ":vimgrep" if 'grepprg' is "internal".
    if rs_grep_internal(nvim_eap_get_cmdidx(eap)) != 0 {
        rs_ex_vimgrep(eap);
        return;
    }

    let cmdidx = nvim_eap_get_cmdidx(eap);
    let au_name = rs_make_get_auname(cmdidx);

    // Inlined nvim_qf_apply_autocmd_pre: fire EVENT_QUICKFIXCMDPRE with curbuf->b_fname
    if !au_name.is_null()
        && apply_autocmds(
            EVENT_QUICKFIXCMDPRE,
            au_name.cast_mut(),
            nvim_qf_buf_get_fname(curbuf).cast_mut(),
            true,
            curbuf,
        )
        && aborting()
    {
        return;
    }

    let wp: WinHandle = if is_loclist_cmd(cmdidx) {
        nvim_get_curwin()
    } else {
        std::ptr::null_mut()
    };

    autowrite_all();

    let fname = rs_get_mef_name();
    if fname.is_null() {
        return;
    }
    nvim_os_remove(fname); // in case it's not unique

    let arg = nvim_eap_get_arg(eap);
    let cmd = rs_make_get_fullcmd(arg, fname);

    do_shell(cmd, 0);

    nvim_incr_quickfix_busy();

    // Determine errorformat: for make/lmake use p_efm, for grep use gefm.
    let errorformat: *mut c_char = if cmdidx != CMD_MAKE && cmdidx != CMD_LMAKE {
        let b_gefm = nvim_curbuf_get_b_p_gefm();
        if !b_gefm.is_null() && *b_gefm != 0 {
            b_gefm.cast_mut()
        } else {
            p_gefm.cast_mut()
        }
    } else {
        p_efm.cast_mut()
    };

    let newlist = cmdidx != CMD_GREPADD && cmdidx != CMD_LGREPADD;

    // Determine encoding: buffer-local b_p_menc if set, else global p_menc.
    let b_menc = nvim_curbuf_get_b_p_menc();
    let enc: *mut c_char = if !b_menc.is_null() && *b_menc != 0 {
        b_menc.cast_mut()
    } else {
        p_menc.cast_mut()
    };

    // Build qf title.
    let mut title_buf = [0u8; 512];
    let cmdlinep = nvim_eap_get_cmdlinep_deref_make(eap);
    let title = make_cmdtitle(cmdlinep, &mut title_buf);

    // Get or allocate the quickfix stack.
    // Use ll_get_or_alloc_list for the init call (allocates if needed).
    let qi_for_init = if wp.is_null() {
        nvim_get_ql_info()
    } else {
        rs_ll_get_or_alloc_list(wp)
    };

    let qi_idx = nvim_qf_get_curlist_idx(qi_for_init);
    let res = rs_qf_init_ext(
        qi_for_init,
        qi_idx,
        fname,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        errorformat,
        newlist,
        0,
        0,
        title,
        enc,
    );

    // After init, re-fetch qi (autocmds might have changed things).
    let qi_post = if wp.is_null() {
        nvim_get_ql_info()
    } else {
        nvim_win_get_loclist(wp).cast_mut()
    };

    if qi_post.is_null() {
        // goto cleanup
        nvim_decr_quickfix_busy();
        nvim_os_remove(fname);
        nvim_xfree(fname.cast::<c_void>());
        nvim_xfree(cmd.cast::<c_void>());
        return;
    }

    if res >= 0 {
        let qfl = nvim_qf_get_curlist_mut(qi_post);
        crate::rs_qf_incr_changedtick(qfl.cast());
    }

    let save_qfid = nvim_qf_get_id(nvim_qf_get_curlist_mut(qi_post).cast_const());

    // Inlined nvim_qf_apply_autocmd_post: fire EVENT_QUICKFIXCMDPOST with curbuf->b_fname
    if !au_name.is_null() {
        apply_autocmds(
            EVENT_QUICKFIXCMDPOST,
            au_name.cast_mut(),
            nvim_qf_buf_get_fname(curbuf).cast_mut(),
            true,
            curbuf,
        );
    }

    if res > 0 && !nvim_eap_get_forceit(eap) && rs_qflist_valid(wp, save_qfid) {
        rs_qf_jump_first(qi_post, save_qfid, 0);
    }

    // cleanup:
    nvim_decr_quickfix_busy();
    nvim_os_remove(fname);
    nvim_xfree(fname.cast::<c_void>());
    nvim_xfree(cmd.cast::<c_void>());
}

// =============================================================================
// Phase 2: ex_cfile
// =============================================================================

/// `:cfile`, `:cgetfile`, `:caddfile`, `:lfile`, `:lgetfile`, `:laddfile` handler.
///
/// # Safety
///
/// `eap` must be a valid pointer to a C `exarg_T`.
#[export_name = "ex_cfile"]
pub unsafe extern "C" fn rs_ex_cfile(eap: EapHandle) {
    let cmdidx = nvim_eap_get_cmdidx(eap);
    let au_name = rs_cfile_get_auname(cmdidx);

    // Inlined nvim_qf_apply_autocmd_pre_null: fire EVENT_QUICKFIXCMDPRE with NULL fname_io
    if !au_name.is_null()
        && apply_autocmds(
            EVENT_QUICKFIXCMDPRE,
            au_name.cast_mut(),
            std::ptr::null_mut(),
            false,
            curbuf,
        )
        && aborting()
    {
        return;
    }

    // If argument given, set 'errorfile' option to it.
    let arg = nvim_eap_get_arg(eap);
    if !arg.is_null() && *arg != 0 {
        nvim_set_option_direct_ef(arg);
    }

    let b_menc = nvim_curbuf_get_b_p_menc();
    let enc: *mut c_char = if !b_menc.is_null() && *b_menc != 0 {
        b_menc.cast_mut()
    } else {
        p_menc.cast_mut()
    };

    let wp: WinHandle = if is_loclist_cmd(cmdidx) {
        nvim_get_curwin()
    } else {
        std::ptr::null_mut()
    };

    nvim_incr_quickfix_busy();

    let mut title_buf = [0u8; 512];
    let cmdlinep = nvim_eap_get_cmdlinep_deref_make(eap);
    let title = make_cmdtitle(cmdlinep, &mut title_buf);

    let qi_for_init = if wp.is_null() {
        nvim_get_ql_info()
    } else {
        rs_ll_get_or_alloc_list(wp)
    };

    let qi_idx = nvim_qf_get_curlist_idx(qi_for_init);
    let newlist = cmdidx != CMD_CADDFILE && cmdidx != CMD_LADDFILE;

    let res = rs_qf_init_ext(
        qi_for_init,
        qi_idx,
        p_ef,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        p_efm.cast_mut(),
        newlist,
        0,
        0,
        title,
        enc,
    );

    // Re-fetch qi after init (autocmds might change things).
    let qi_post = if wp.is_null() {
        nvim_get_ql_info()
    } else {
        nvim_win_get_loclist(wp).cast_mut()
    };

    if qi_post.is_null() {
        nvim_decr_quickfix_busy();
        return;
    }

    if res >= 0 {
        let qfl = nvim_qf_get_curlist_mut(qi_post);
        crate::rs_qf_incr_changedtick(qfl.cast());
    }

    let save_qfid = nvim_qf_get_id(nvim_qf_get_curlist_mut(qi_post).cast_const());

    // Inlined nvim_qf_apply_autocmd_post_null: fire EVENT_QUICKFIXCMDPOST with NULL fname_io
    if !au_name.is_null() {
        apply_autocmds(
            EVENT_QUICKFIXCMDPOST,
            au_name.cast_mut(),
            std::ptr::null_mut(),
            false,
            curbuf,
        );
    }

    // Jump for cfile/lfile only (not cgetfile/caddfile).
    if res > 0 && (cmdidx == CMD_CFILE || cmdidx == CMD_LFILE) && rs_qflist_valid(wp, save_qfid) {
        rs_qf_jump_first(qi_post, save_qfid, c_int::from(nvim_eap_get_forceit(eap)));
    }

    nvim_decr_quickfix_busy();
}

// =============================================================================
// Phase 3: ex_cbuffer + cbuffer_process_args
// =============================================================================

/// Validate and set line1/line2 from eap for cbuffer commands.
/// Returns OK (1) or FAIL (0).
///
/// # Safety
///
/// `eap` and `bufp` must be valid pointers.
unsafe fn cbuffer_process_args(eap: EapHandle, bufp: *mut BufHandle) -> c_int {
    let arg = nvim_eap_get_arg(eap);
    let buf: BufHandle;

    if arg.is_null() || *arg == 0 {
        buf = curbuf;
    } else {
        let tail = skipdigits(arg);
        // if *skipwhite(skipdigits(arg)) == NUL
        let tail_ws = skip_nul_whitespace(tail);
        if !tail_ws.is_null() && *tail_ws == 0 {
            buf = nvim_buflist_findnr_ptr(atoi(arg));
        } else {
            buf = std::ptr::null_mut();
        }
    }

    if buf.is_null() {
        emsg(c"E474: Invalid argument".as_ptr());
        return FAIL;
    }

    if !nvim_buf_has_ml_mfp_void(buf) {
        emsg(c"E681: Buffer is not loaded".as_ptr());
        return FAIL;
    }

    let ml_line_count = nvim_buf_get_ml_line_count_void(buf);

    let addr_count = nvim_eap_get_addr_count(eap);
    if addr_count == 0 {
        nvim_eap_set_line1(eap, 1);
        nvim_eap_set_line2(eap, ml_line_count);
    }

    let line1 = nvim_eap_get_line1(eap);
    let line2 = nvim_eap_get_line2(eap);

    if line1 < 1 || line1 > ml_line_count || line2 < 1 || line2 > ml_line_count {
        emsg(c"E16: Invalid range".as_ptr());
        return FAIL;
    }

    *bufp = buf;
    OK
}

/// Skip NUL whitespace helper (equivalent to skipwhite for the tail check).
const unsafe fn skip_nul_whitespace(p: *const c_char) -> *const c_char {
    if p.is_null() {
        return p;
    }
    let mut ptr = p;
    while *ptr == b' ' as c_char || *ptr == b'\t' as c_char {
        ptr = ptr.add(1);
    }
    ptr
}

/// `:cbuffer`, `:caddbuffer`, `:cgetbuffer`, `:lbuffer`, `:laddbuffer`, `:lgetbuffer` handler.
///
/// # Safety
///
/// `eap` must be a valid pointer to a C `exarg_T`.
#[export_name = "ex_cbuffer"]
pub unsafe extern "C" fn rs_ex_cbuffer(eap: EapHandle) {
    let cmdidx = nvim_eap_get_cmdidx(eap);
    let au_name = rs_cbuffer_get_auname(cmdidx);

    // Inlined nvim_qf_apply_autocmd_pre: fire EVENT_QUICKFIXCMDPRE with curbuf->b_fname
    if !au_name.is_null()
        && apply_autocmds(
            EVENT_QUICKFIXCMDPRE,
            au_name.cast_mut(),
            nvim_qf_buf_get_fname(curbuf).cast_mut(),
            true,
            curbuf,
        )
        && aborting()
    {
        return;
    }

    // Must come after autocommands.
    let mut wp: WinHandle = std::ptr::null_mut();
    let qi = rs_qf_cmd_get_or_alloc_stack(eap, &raw mut wp);

    let mut buf: BufHandle = std::ptr::null_mut();
    if cbuffer_process_args(eap, &raw mut buf) == FAIL {
        return;
    }

    // Build title.
    let mut title_buf = [0u8; 512];
    let cmdlinep = nvim_eap_get_cmdlinep_deref_make(eap);
    let title_base = make_cmdtitle(cmdlinep, &mut title_buf);

    // If buf has sfname, append " (sfname)" to title via IObuff.
    let sfname = nvim_buf_get_sfname_void(buf);
    let title: *const c_char = if sfname.is_null() {
        title_base
    } else {
        nvim_qf_snprintf_iobuff(title_base, sfname);
        IObuff
    };

    nvim_incr_quickfix_busy();

    let qi_idx = nvim_qf_get_curlist_idx(qi);
    let newlist = cmdidx != CMD_CADDBUFFER && cmdidx != CMD_LADDBUFFER;
    let line1 = nvim_eap_get_line1(eap);
    let line2 = nvim_eap_get_line2(eap);

    let res = rs_qf_init_ext(
        qi,
        qi_idx,
        std::ptr::null(),
        buf,
        std::ptr::null_mut(),
        p_efm.cast_mut(),
        newlist,
        line1,
        line2,
        title,
        std::ptr::null_mut(),
    );

    if rs_qf_stack_empty(qi.cast_const()) {
        nvim_decr_quickfix_busy();
        return;
    }

    if res >= 0 {
        let qfl = nvim_qf_get_curlist_mut(qi);
        crate::rs_qf_incr_changedtick(qfl.cast());
    }

    let save_qfid = nvim_qf_get_id(nvim_qf_get_curlist_mut(qi).cast_const());

    // Inlined nvim_qf_apply_autocmd_post_track: fire EVENT_QUICKFIXCMDPOST, return true if curbuf changed
    let curbuf_changed = if au_name.is_null() {
        false
    } else {
        let old_curbuf = curbuf;
        apply_autocmds(
            EVENT_QUICKFIXCMDPOST,
            au_name.cast_mut(),
            nvim_qf_buf_get_fname(curbuf).cast_mut(),
            true,
            curbuf,
        );
        curbuf != old_curbuf
    };
    let res_final = if curbuf_changed { 0 } else { res };

    if res_final > 0
        && (cmdidx == CMD_CBUFFER || cmdidx == CMD_LBUFFER)
        && rs_qflist_valid(wp, save_qfid)
    {
        rs_qf_jump_first(qi, save_qfid, c_int::from(nvim_eap_get_forceit(eap)));
    }

    nvim_decr_quickfix_busy();
}

// =============================================================================
// Phase 4: ex_cexpr
// =============================================================================

/// `:cexpr`, `:cgetexpr`, `:caddexpr`, `:lexpr`, `:lgetexpr`, `:laddexpr` handler.
///
/// # Safety
///
/// `eap` must be a valid pointer to a C `exarg_T`.
#[export_name = "ex_cexpr"]
pub unsafe extern "C" fn rs_ex_cexpr(eap: EapHandle) {
    let cmdidx = nvim_eap_get_cmdidx(eap);
    let au_name = rs_cexpr_get_auname(cmdidx);

    // Inlined nvim_qf_apply_autocmd_pre: fire EVENT_QUICKFIXCMDPRE with curbuf->b_fname
    if !au_name.is_null()
        && apply_autocmds(
            EVENT_QUICKFIXCMDPRE,
            au_name.cast_mut(),
            nvim_qf_buf_get_fname(curbuf).cast_mut(),
            true,
            curbuf,
        )
        && aborting()
    {
        return;
    }

    let mut wp: WinHandle = std::ptr::null_mut();
    let qi = rs_qf_cmd_get_or_alloc_stack(eap, &raw mut wp);

    // Evaluate the expression. When the result is a string or a list we can
    // use it to fill the errorlist.
    let arg = nvim_eap_get_arg(eap);
    let tv = nvim_eval_expr(arg, eap);
    if tv.is_null() {
        return;
    }

    let tv_type = nvim_tv_get_type_void(tv);
    let tv_str = nvim_tv_get_vval_string(tv);
    let is_list = nvim_tv_is_list(tv);

    if (tv_type == VAR_STRING && !tv_str.is_null()) || is_list {
        nvim_incr_quickfix_busy();

        let mut title_buf = [0u8; 512];
        let cmdlinep = nvim_eap_get_cmdlinep_deref_make(eap);
        let title = make_cmdtitle(cmdlinep, &mut title_buf);

        let qi_idx = nvim_qf_get_curlist_idx(qi);
        let newlist = cmdidx != CMD_CADDEXPR && cmdidx != CMD_LADDEXPR;

        let res = rs_qf_init_ext(
            qi,
            qi_idx,
            std::ptr::null(),
            std::ptr::null_mut(),
            tv,
            p_efm.cast_mut(),
            newlist,
            0,
            0,
            title,
            std::ptr::null_mut(),
        );

        if rs_qf_stack_empty(qi.cast_const()) {
            nvim_decr_quickfix_busy();
            nvim_tv_free_void(tv);
            return;
        }

        if res >= 0 {
            let qfl = nvim_qf_get_curlist_mut(qi);
            crate::rs_qf_incr_changedtick(qfl.cast());
        }

        let save_qfid = nvim_qf_get_id(nvim_qf_get_curlist_mut(qi).cast_const());

        // Inlined nvim_qf_apply_autocmd_post: fire EVENT_QUICKFIXCMDPOST with curbuf->b_fname
        if !au_name.is_null() {
            apply_autocmds(
                EVENT_QUICKFIXCMDPOST,
                au_name.cast_mut(),
                nvim_qf_buf_get_fname(curbuf).cast_mut(),
                true,
                curbuf,
            );
        }

        if res > 0 && (cmdidx == CMD_CEXPR || cmdidx == CMD_LEXPR) && rs_qflist_valid(wp, save_qfid)
        {
            rs_qf_jump_first(qi, save_qfid, c_int::from(nvim_eap_get_forceit(eap)));
        }

        nvim_decr_quickfix_busy();
    } else {
        emsg(c"E777: String or List expected".as_ptr());
    }

    nvim_tv_free_void(tv);
}

// =============================================================================
// Phase 5: copy_loclist_stack
// =============================================================================

/// Copy location list stack from `from` window to `to` window.
///
/// # Safety
///
/// `from` and `to` must be valid non-null pointers to `win_T`.
#[export_name = "copy_loclist_stack"]
pub unsafe extern "C" fn rs_copy_loclist_stack(from: *mut c_void, to: *mut c_void) {
    // Get the qi to copy from: for LL windows use w_llist_ref, else w_llist.
    let qi = nvim_win_get_llist_or_ref(from);
    if qi.is_null() {
        return; // no location list to copy
    }

    // Get lhi (p_lhi) from `from` window.
    let lhi = nvim_win_get_p_lhi(from);

    // Allocate a new location list stack with size = from->w_p_lhi.
    let new_qi = rs_qf_alloc_stack(QFLT_LOCATION, lhi);

    // Set to->w_llist to the new stack.
    nvim_win_set_llist(to, new_qi);

    // new_qi->qf_maxcount is set by alloc; copy lhi from it.
    let maxcount = nvim_qf_get_maxcount(new_qi);
    // (The window's w_p_lhi is updated by nvim_win_set_llist in C side via
    //  the existing `nvim_win_set_p_lhi` accessor if needed; the C original
    //  sets to->w_p_lhi = to->w_llist->qf_maxcount. We mirror that via accessor.)
    // Actually the C original does: to->w_p_lhi = to->w_llist->qf_maxcount.
    // We need to set that. Use existing accessor:
    nvim_win_set_p_lhi(to, maxcount);

    let listcount = nvim_qf_get_listcount(qi);
    nvim_qf_set_listcount(new_qi, listcount);

    // Copy each location list.
    for idx in 0..listcount {
        // Set new_qi->qf_curlist = idx
        nvim_qf_set_curlist_idx(new_qi, idx);

        let from_list = nvim_qi_get_list_qi(qi, idx);
        let to_list = nvim_qi_get_list_qi(new_qi, idx);

        if rs_copy_loclist(from_list, to_list) == FAIL {
            nvim_qf_free_all_win(to);
            return;
        }
    }

    // Set new_qi->qf_curlist = qi->qf_curlist (current list).
    let cur = nvim_qf_get_curlist_idx(qi);
    nvim_qf_set_curlist_idx(new_qi, cur);
}
