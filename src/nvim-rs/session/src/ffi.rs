//! FFI declarations for C accessor functions used by session migration.
//!
//! Each `nvim_ses_*` function is a thin C accessor defined in `ex_session.c`
//! that provides access to struct fields and global variables.

use std::ffi::{c_char, c_int, c_uint, c_void};

/// Opaque handle types for C structs
pub type WinPtr = *mut c_void;
pub type FramePtr = *mut c_void;
pub type FramePtrConst = *const c_void;
pub type BufPtr = *mut c_void;
pub type TabpagePtr = *mut c_void;
pub type ExargPtr = *mut c_void;
pub type GarrayPtr = *mut c_void;

extern "C" {
    // --- Window accessors (Phase 2) ---
    pub fn nvim_ses_win_get_floating(wp: WinPtr) -> bool;
    pub fn nvim_ses_win_get_buffer(wp: WinPtr) -> BufPtr;
    pub fn nvim_ses_win_get_next(wp: WinPtr) -> WinPtr;

    // --- Buffer query accessors (Phase 2) ---
    pub fn nvim_ses_buf_get_fname(buf: BufPtr) -> *const c_char;
    pub fn nvim_ses_buf_is_terminal(buf: BufPtr) -> bool;
    pub fn nvim_ses_bt_nofilename(buf: BufPtr) -> bool;
    pub fn nvim_ses_bt_help(buf: BufPtr) -> bool;
    pub fn nvim_ses_bt_terminal(buf: BufPtr) -> bool;

    // --- Session flags (Phase 2) ---
    pub fn nvim_ses_get_ssop_flags() -> c_uint;
    pub fn nvim_ses_get_ssop_flags_ptr() -> *const c_uint;

    // --- Frame accessors (Phase 2) ---
    pub fn nvim_ses_frame_get_layout(fr: FramePtr) -> c_int;
    pub fn nvim_ses_frame_get_child(fr: FramePtr) -> FramePtr;
    pub fn nvim_ses_frame_get_next(fr: FramePtr) -> FramePtr;
    pub fn nvim_ses_frame_get_win(fr: FramePtr) -> WinPtr;

    // --- Filename helper accessors (Phase 3) ---
    pub fn nvim_ses_buf_get_sfname(buf: BufPtr) -> *const c_char;
    pub fn nvim_ses_buf_get_ffname(buf: BufPtr) -> *const c_char;
    pub fn nvim_ses_get_vop_flags_ptr() -> *const c_uint;
    pub fn nvim_ses_get_p_acd() -> c_int;
    pub fn nvim_ses_get_did_lcd() -> c_int;
    pub fn nvim_ses_set_did_lcd(val: c_int);
    pub fn nvim_ses_home_replace_save(name: *const c_char) -> *mut c_char;
    pub fn nvim_ses_vim_strsave_fnameescape(name: *const c_char) -> *mut c_char;
    pub fn nvim_ses_xfree(p: *mut c_void);
    pub fn nvim_ses_utfc_ptr2len(p: *const c_char) -> c_int;

    // --- Window struct accessors (Phase 4) ---
    pub fn nvim_ses_win_get_curswant(wp: WinPtr) -> c_int;
    pub fn nvim_ses_win_get_virtcol(wp: WinPtr) -> c_int;
    pub fn nvim_ses_win_get_height(wp: WinPtr) -> c_int;
    pub fn nvim_ses_win_get_hsep_height(wp: WinPtr) -> c_int;
    pub fn nvim_ses_win_get_status_height(wp: WinPtr) -> c_int;
    pub fn nvim_ses_win_get_width(wp: WinPtr) -> c_int;

    // --- Global variables (Phase 4) ---
    pub fn nvim_ses_get_topframe() -> FramePtr;
    pub fn nvim_ses_topframe_get_height() -> c_int;
    pub fn nvim_ses_get_Rows() -> c_int;
    pub fn nvim_ses_get_Columns() -> c_int;

    // --- garray / arglist accessors (Phase 4) ---
    pub fn nvim_ses_ga_get_len(gap: GarrayPtr) -> c_int;
    pub fn nvim_ses_alist_name_at(gap: GarrayPtr, i: c_int) -> *mut c_char;
    pub fn nvim_ses_xmalloc(size: usize) -> *mut c_char;
    pub fn nvim_ses_vim_FullName(
        fname: *const c_char,
        buf: *mut c_char,
        len: usize,
        force: bool,
    ) -> c_int;

    // --- Phase 5: store_session_globals callback ---
    /// Iterate over session-flavoured global variables.
    /// Calls `cb` for each variable. `var_type`: 0=number, 1=string, 2=float.
    /// For type 0/1: `escaped_val` is the escaped string value.
    /// For type 2: `float_val` and `float_sign` (' ' or '-') are set.
    pub fn nvim_ses_foreach_session_global(
        cb: unsafe extern "C" fn(
            key: *const c_char,
            var_type: c_int,
            escaped_val: *const c_char,
            float_val: f64,
            float_sign: c_int,
            ud: *mut c_void,
        ) -> c_int,
        ud: *mut c_void,
    ) -> c_int;

    // --- Phase 5: get_view_file accessors ---
    pub fn nvim_ses_get_curbuf_ffname() -> *const c_char;
    pub fn nvim_ses_emsg_noname();
    pub fn nvim_ses_get_p_vdir() -> *const c_char;
    pub fn nvim_ses_vim_ispathsep(c: c_int) -> bool;
    pub fn nvim_ses_add_pathsep(p: *mut c_char) -> bool;

    // --- Phase 6: put_view accessors ---

    // Window argument list
    pub fn nvim_ses_win_uses_global_alist(wp: WinPtr) -> bool;
    pub fn nvim_ses_win_get_alist_ga(wp: WinPtr) -> GarrayPtr;
    pub fn nvim_ses_win_get_arg_idx(wp: WinPtr) -> c_int;
    pub fn nvim_ses_win_get_arg_idx_invalid(wp: WinPtr) -> bool;
    pub fn nvim_ses_win_wargcount(wp: WinPtr) -> c_int;

    // Window tag stack
    pub fn nvim_ses_win_get_tagstackidx(wp: WinPtr) -> c_int;
    pub fn nvim_ses_win_get_tagstacklen(wp: WinPtr) -> c_int;
    pub fn nvim_ses_win_get_tagname(wp: WinPtr, idx: c_int) -> *const c_char;

    // Window alternate file
    pub fn nvim_ses_win_get_alt_fnum(wp: WinPtr) -> c_int;

    // Window cursor/view
    pub fn nvim_ses_win_get_cursor_lnum(wp: WinPtr) -> i32;
    pub fn nvim_ses_win_get_cursor_col(wp: WinPtr) -> c_int;
    pub fn nvim_ses_win_get_topline(wp: WinPtr) -> i32;
    pub fn nvim_ses_win_get_view_height(wp: WinPtr) -> c_int;
    pub fn nvim_ses_win_get_p_wrap(wp: WinPtr) -> bool;
    pub fn nvim_ses_win_get_leftcol(wp: WinPtr) -> c_int;
    pub fn nvim_ses_win_get_localdir(wp: WinPtr) -> *mut c_char;

    // Buffer query
    pub fn nvim_ses_buf_get_p_bl(buf: BufPtr) -> bool;
    pub fn nvim_ses_bt_normal(buf: BufPtr) -> bool;

    // Tabpage
    pub fn nvim_ses_tp_get_localdir(tp: TabpagePtr) -> *mut c_char;

    // Buffer lookup
    pub fn nvim_ses_buflist_findnr(nr: c_int) -> BufPtr;

    // Global state
    pub fn nvim_ses_get_curwin() -> WinPtr;
    pub fn nvim_ses_set_curwin(wp: WinPtr);

    // C functions wrapped for put_view
    pub fn nvim_ses_makemap(fd: *mut libc::FILE, buf: BufPtr) -> c_int;
    pub fn nvim_ses_makeset(fd: *mut libc::FILE, opt: c_int, local_only: bool) -> c_int;
    pub fn nvim_ses_makefoldset(fd: *mut libc::FILE) -> c_int;
    #[link_name = "rs_put_folds"]
    pub fn nvim_ses_put_folds(fd: *mut libc::FILE, wp: WinPtr) -> c_int;

    // --- Phase 7: makeopens accessors ---

    // Buffer iteration
    pub fn nvim_ses_foreach_buffer(
        cb: unsafe extern "C" fn(buf: BufPtr, only_save_windows: bool, ud: *mut c_void) -> c_int,
        only_save_windows: bool,
        ud: *mut c_void,
    ) -> c_int;

    // Buffer fields
    pub fn nvim_ses_buf_get_nwindows(buf: BufPtr) -> c_int;
    pub fn nvim_ses_buf_is_help(buf: BufPtr) -> bool;
    pub fn nvim_ses_buf_get_wininfo_lnum(buf: BufPtr) -> i64;

    // Global argument list
    pub fn nvim_ses_get_global_alist_ga() -> GarrayPtr;

    // Tabpage iteration and fields
    pub fn nvim_ses_get_first_tabpage() -> TabpagePtr;
    pub fn nvim_ses_get_curtab() -> TabpagePtr;
    pub fn nvim_ses_tp_get_next(tp: TabpagePtr) -> TabpagePtr;
    pub fn nvim_ses_tp_get_firstwin(tp: TabpagePtr) -> WinPtr;
    pub fn nvim_ses_tp_get_topframe(tp: TabpagePtr) -> FramePtr;

    // Window globals
    pub fn nvim_ses_get_firstwin() -> WinPtr;
    #[link_name = "rs_tabpage_index"]
    pub fn nvim_ses_tabpage_index(tp: TabpagePtr) -> c_int;

    // Session option globals
    pub fn nvim_ses_get_globaldir() -> *const c_char;
    pub fn nvim_ses_get_p_wh() -> i64;
    pub fn nvim_ses_get_p_wiw() -> i64;
    pub fn nvim_ses_get_p_shm() -> *const c_char;
    pub fn nvim_ses_get_p_stal() -> i64;

    // --- Phase 8: ex_mkrc, ex_loadview accessors ---

    // exarg_T field accessors
    pub fn nvim_ses_eap_get_cmdidx(eap: ExargPtr) -> c_int;
    pub fn nvim_ses_eap_get_arg(eap: ExargPtr) -> *mut c_char;
    pub fn nvim_ses_eap_get_forceit(eap: ExargPtr) -> bool;
    pub fn nvim_ses_eap_set_forceit(eap: ExargPtr, val: bool);

    // CMD enum accessors
    pub fn nvim_ses_get_CMD_mksession() -> c_int;
    pub fn nvim_ses_get_CMD_mkview() -> c_int;
    pub fn nvim_ses_get_CMD_mkvimrc() -> c_int;
    pub fn nvim_ses_get_CMD_mkexrc() -> c_int;

    // File I/O wrappers
    pub fn nvim_ses_open_exfile(
        fname: *mut c_char,
        forceit: c_int,
        mode: *mut c_char,
    ) -> *mut libc::FILE;
    pub fn nvim_ses_fclose(fd: *mut libc::FILE) -> c_int;
    pub fn nvim_ses_do_source(fname: *mut c_char) -> c_int;

    // OS wrappers
    pub fn nvim_ses_os_isdir(dir: *const c_char) -> bool;
    pub fn nvim_ses_vim_mkdir_emsg(dir: *const c_char, perm: c_int) -> c_int;
    pub fn nvim_ses_os_dirname(buf: *mut c_char, len: usize) -> c_int;
    pub fn nvim_ses_os_chdir(dir: *const c_char) -> c_int;
    pub fn nvim_ses_vim_chdirfile(fname: *mut c_char) -> c_int;
    pub fn nvim_ses_shorten_fnames(force: c_int);

    // Session-related global state
    pub fn nvim_ses_get_p_hls() -> bool;
    pub fn nvim_ses_get_no_hlsearch() -> bool;
    pub fn nvim_ses_set_vim_var_string(val: *const c_char);
    pub fn nvim_ses_apply_autocmds_session();
    pub fn nvim_ses_emsg(s: *const c_char);
    pub fn nvim_ses_semsg(fmt: *const c_char, arg: *const c_char);
    pub fn nvim_ses_get_curbuf() -> BufPtr;

    // Error message string accessors
    pub fn nvim_ses_get_e_prev_dir() -> *const c_char;
    pub fn nvim_ses_get_e_write() -> *const c_char;
    pub fn nvim_ses_get_e_notopen() -> *const c_char;

    // Filename constants
    pub fn nvim_ses_get_VIMRC_FILE() -> *const c_char;
    pub fn nvim_ses_get_SESSION_FILE() -> *const c_char;
    pub fn nvim_ses_get_EXRC_FILE() -> *const c_char;

    // Option flag values
    pub fn nvim_ses_get_OPT_GLOBAL() -> c_int;
    pub fn nvim_ses_get_OPT_SKIPRTP() -> c_int;
}
