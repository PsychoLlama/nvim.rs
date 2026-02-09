//! FFI declarations for C accessor functions
//!
//! All opaque handle types and extern "C" declarations for accessing
//! C-side data structures used by the arglist module.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Opaque Handle Types
// =============================================================================

/// Opaque pointer to alist_T
pub type AlistPtr = *mut c_void;

/// Opaque pointer to aentry_T
pub type AentryPtr = *mut c_void;

/// Opaque pointer to win_T
pub type WinPtr = *mut c_void;

/// Opaque pointer to buf_T
pub type BufPtr = *mut c_void;

/// Opaque pointer to tabpage_T
pub type TabpagePtr = *mut c_void;

/// Opaque pointer to exarg_T
pub type ExargPtr = *mut c_void;

/// Opaque pointer to garray_T
pub type GarrayPtr = *mut c_void;

/// Opaque pointer to typval_T
pub type TypvalPtr = *mut c_void;

/// Opaque pointer to expand_T
pub type ExpandPtr = *mut c_void;

/// Opaque pointer to bufref_T
pub type BufrefPtr = *mut c_void;

// =============================================================================
// Phase 1: Global State Accessors
// =============================================================================

extern "C" {
    // -- Globals --
    pub fn nvim_al_get_arglist_locked() -> c_int;
    pub fn nvim_al_set_arglist_locked(val: c_int);
    pub fn nvim_al_get_global_alist() -> AlistPtr;
    pub fn nvim_al_get_arg_had_last() -> c_int;
    pub fn nvim_al_set_arg_had_last(val: c_int);
    pub fn nvim_al_get_max_alist_id() -> c_int;
    pub fn nvim_al_inc_max_alist_id() -> c_int;
    pub fn nvim_al_get_curwin() -> WinPtr;
    pub fn nvim_al_get_curbuf() -> BufPtr;
    pub fn nvim_al_get_curtab() -> TabpagePtr;
    pub fn nvim_al_get_got_int() -> c_int;

    // -- Macros --
    pub fn nvim_al_ARGCOUNT() -> c_int;
    pub fn nvim_al_ARGLIST() -> AentryPtr;
    pub fn nvim_al_GARGCOUNT() -> c_int;
    pub fn nvim_al_GARGLIST() -> AentryPtr;
    pub fn nvim_al_ALIST_curwin() -> AlistPtr;
    pub fn nvim_al_WARGCOUNT(wp: WinPtr) -> c_int;
    pub fn nvim_al_WARGLIST(wp: WinPtr) -> AentryPtr;
    pub fn nvim_al_AARGLIST(al: AlistPtr, i: c_int) -> AentryPtr;

    // -- alist_T fields --
    pub fn nvim_al_ga_ptr(al: AlistPtr) -> GarrayPtr;
    pub fn nvim_al_get_refcount(al: AlistPtr) -> c_int;
    pub fn nvim_al_set_refcount(al: AlistPtr, val: c_int);
    pub fn nvim_al_inc_refcount(al: AlistPtr);
    pub fn nvim_al_dec_refcount(al: AlistPtr) -> c_int;
    pub fn nvim_al_get_id(al: AlistPtr) -> c_int;
    pub fn nvim_al_set_id(al: AlistPtr, val: c_int);

    // -- aentry_T fields --
    pub fn nvim_al_ae_get_fname(ae: AentryPtr) -> *mut c_char;
    pub fn nvim_al_ae_set_fname(ae: AentryPtr, fname: *mut c_char);
    pub fn nvim_al_ae_get_fnum(ae: AentryPtr) -> c_int;
    pub fn nvim_al_ae_set_fnum(ae: AentryPtr, fnum: c_int);

    // -- garray_T ops --
    pub fn nvim_al_ga_get_len(ga: GarrayPtr) -> c_int;
    pub fn nvim_al_ga_set_len(ga: GarrayPtr, len: c_int);
    pub fn nvim_al_ga_get_data(ga: GarrayPtr) -> *mut c_void;
    pub fn nvim_al_ga_init_aentry(ga: GarrayPtr);
    pub fn nvim_al_ga_grow(ga: GarrayPtr, n: c_int);
    pub fn nvim_al_ga_clear(ga: GarrayPtr);

    // -- curwin fields --
    pub fn nvim_al_win_get_arg_idx(wp: WinPtr) -> c_int;
    pub fn nvim_al_win_set_arg_idx(wp: WinPtr, idx: c_int);
    pub fn nvim_al_win_get_alist(wp: WinPtr) -> AlistPtr;
    pub fn nvim_al_win_set_alist(wp: WinPtr, al: AlistPtr);
    pub fn nvim_al_win_set_locked(wp: WinPtr, val: c_int);
}

// =============================================================================
// Phase 2: Extra Accessors
// =============================================================================

extern "C" {
    pub fn nvim_al_emsg_arglist_locked();
    pub fn nvim_al_xfree(ptr: *mut c_void);
    pub fn nvim_al_xmalloc(size: usize) -> *mut c_void;
    pub fn nvim_al_xstrdup(s: *const c_char) -> *mut c_char;
    pub fn nvim_al_deep_clear_aentry(al: AlistPtr);
    pub fn nvim_al_buflist_add(fname: *const c_char, flags: c_int) -> c_int;
    pub fn nvim_al_buf_set_name(fnum: c_int, name: *const c_char);
    pub fn nvim_al_os_breakcheck();
    pub fn nvim_al_alloc_alist() -> AlistPtr;
    pub fn nvim_al_ga_init_charptr(ga: GarrayPtr);
    pub fn nvim_al_ga_append_charptr(ga: GarrayPtr, ptr: *mut c_char);
}

// =============================================================================
// Phase 3: String Parsing Accessors
// =============================================================================

extern "C" {
    pub fn nvim_al_rem_backslash(p: *const c_char) -> c_int;
    pub fn nvim_al_ascii_isspace(c: c_int) -> c_int;
    pub fn nvim_al_skipwhite(p: *const c_char) -> *mut c_char;
    pub fn nvim_al_expand_wildcards(
        num_pat: c_int,
        pat: *mut *mut c_char,
        num_files: *mut c_int,
        files: *mut *mut *mut c_char,
        flags: c_int,
    ) -> c_int;
    pub fn nvim_al_gen_expand_wildcards(
        num_pat: c_int,
        pat: *mut *mut c_char,
        num_files: *mut c_int,
        files: *mut *mut *mut c_char,
        flags: c_int,
    ) -> c_int;
}

// =============================================================================
// Phase 4: Manipulation Accessors
// =============================================================================

extern "C" {
    pub fn nvim_al_foreach_tab_window(
        cb: extern "C" fn(WinPtr, TabpagePtr, *mut c_void),
        ud: *mut c_void,
    );
    pub fn nvim_al_memmove_aentry(dst: AentryPtr, src: AentryPtr, count: c_int);
    pub fn nvim_al_file_pat_to_reg_pat(
        pat: *const c_char,
        pat_end: *const c_char,
        allow_dirs: *mut c_int,
        no_bslash: c_int,
    ) -> *mut c_char;
    pub fn nvim_al_regmatch_compile(pat: *const c_char, ic: c_int) -> *mut c_void;
    pub fn nvim_al_regmatch_exec(rm: *mut c_void, str: *const c_char) -> c_int;
    pub fn nvim_al_regmatch_free(rm: *mut c_void);
    pub fn nvim_al_magic_isset() -> c_int;
    pub fn nvim_al_get_p_fic() -> c_int;
    pub fn nvim_al_semsg_nomatch2(pat: *const c_char);
    pub fn nvim_al_emsg_nomatch();
    pub fn nvim_al_curbuf_b_ffname() -> *mut c_char;
    pub fn nvim_al_curbuf_b_fname() -> *mut c_char;
}

// =============================================================================
// Phase 5: Query Accessors
// =============================================================================

extern "C" {
    pub fn nvim_al_buflist_findnr(fnum: c_int) -> BufPtr;
    pub fn nvim_al_buf_get_fname(buf: BufPtr) -> *mut c_char;
    pub fn nvim_al_buf_get_ffname(buf: BufPtr) -> *mut c_char;
    pub fn nvim_al_buf_get_fnum(buf: BufPtr) -> c_int;
    pub fn nvim_al_path_full_compare(
        s1: *const c_char,
        s2: *const c_char,
        check_name: c_int,
        expand_env: c_int,
    ) -> c_int;
    pub fn nvim_al_win_get_buffer(wp: WinPtr) -> BufPtr;
    pub fn nvim_al_win_set_arg_idx_invalid(wp: WinPtr, val: c_int);
}

// =============================================================================
// Phase 6: Simple Ex Command Accessors
// =============================================================================

extern "C" {
    // exarg_T field getters/setters
    pub fn nvim_al_eap_get_cmd(eap: ExargPtr) -> *mut c_char;
    pub fn nvim_al_eap_get_arg(eap: ExargPtr) -> *mut c_char;
    pub fn nvim_al_eap_get_line1(eap: ExargPtr) -> i64;
    pub fn nvim_al_eap_get_line2(eap: ExargPtr) -> i64;
    pub fn nvim_al_eap_get_addr_count(eap: ExargPtr) -> c_int;
    pub fn nvim_al_eap_get_forceit(eap: ExargPtr) -> c_int;
    pub fn nvim_al_eap_get_cmdidx(eap: ExargPtr) -> c_int;
    pub fn nvim_al_eap_set_line1(eap: ExargPtr, val: i64);
    pub fn nvim_al_eap_set_line2(eap: ExargPtr, val: i64);

    // Buffer/window operations
    pub fn nvim_al_check_can_set_curbuf_forceit(forceit: c_int) -> c_int;
    pub fn nvim_al_setpcmark();
    pub fn nvim_al_win_split(size: c_int, flags: c_int) -> c_int;
    pub fn nvim_al_reset_binding(wp: WinPtr);
    pub fn nvim_al_buf_hide(buf: BufPtr) -> c_int;
    pub fn nvim_al_fix_fname(fname: *const c_char) -> *mut c_char;
    pub fn nvim_al_otherfile(fname: *const c_char) -> c_int;
    pub fn nvim_al_check_changed(buf: BufPtr, flags: c_int) -> c_int;
    pub fn nvim_al_do_ecmd(
        fnum: c_int,
        ffname: *const c_char,
        sfname: *const c_char,
        eap: ExargPtr,
        newlnum: i64,
        flags: c_int,
        old_curwin: WinPtr,
    ) -> c_int;
    pub fn nvim_al_setmark(c: c_int);
    pub fn nvim_al_FullName_save(fname: *const c_char, force: c_int) -> *mut c_char;
    pub fn nvim_al_path_fnamecmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn nvim_al_get_cmdmod_cmod_tab() -> c_int;
}

// =============================================================================
// Phase 7: Complex Ex Command Accessors
// =============================================================================

extern "C" {
    pub fn nvim_al_gotocmdline(clr: c_int);
    pub fn nvim_al_list_in_columns(items: *mut *mut c_char, count: c_int, current: c_int);
    pub fn nvim_al_maketitle();
    pub fn nvim_al_curbuf_reusable() -> c_int;
    pub fn nvim_al_curbuf_ml_empty() -> c_int;
    pub fn nvim_al_emsg_invarg();
    pub fn nvim_al_emsg_invrange();
    pub fn nvim_al_emsg_E610();
    pub fn nvim_al_emsg_str(msg: *const c_char);
    pub fn nvim_al_alist_name(ae: AentryPtr) -> *mut c_char;
    pub fn nvim_al_ga_init_charptr_n(ga: GarrayPtr, n: c_int);
}

// =============================================================================
// Phase 8: Window Management Accessors
// =============================================================================

extern "C" {
    pub fn nvim_al_get_firstwin() -> WinPtr;
    pub fn nvim_al_get_lastwin() -> WinPtr;
    pub fn nvim_al_get_first_tabpage() -> TabpagePtr;
    pub fn nvim_al_goto_tabpage_tp(tp: TabpagePtr, trigger_enter: c_int, trigger_leave: c_int);
    pub fn nvim_al_valid_tabpage(tp: TabpagePtr) -> c_int;
    pub fn nvim_al_win_valid(wp: WinPtr) -> c_int;
    pub fn nvim_al_win_close(wp: WinPtr, free_buf: c_int, force: c_int);
    pub fn nvim_al_win_enter(wp: WinPtr, undo_sync: c_int);
    pub fn nvim_al_win_move_after(wp: WinPtr, after: WinPtr);
    pub fn nvim_al_lastwin_nofloating() -> WinPtr;
    pub fn nvim_al_win_is_floating(wp: WinPtr) -> c_int;
    pub fn nvim_al_win_get_prev(wp: WinPtr) -> WinPtr;
    pub fn nvim_al_win_get_next(wp: WinPtr) -> WinPtr;
    pub fn nvim_al_win_get_width(wp: WinPtr) -> c_int;
    pub fn nvim_al_win_get_frame_parent(wp: WinPtr) -> *mut c_void;
    pub fn nvim_al_get_Columns() -> c_int;
    pub fn nvim_al_buf_get_nwindows(buf: BufPtr) -> c_int;
    pub fn nvim_al_bufIsChanged(buf: BufPtr) -> c_int;
    pub fn nvim_al_buf_is_empty(buf: BufPtr) -> c_int;
    pub fn nvim_al_autowrite(buf: BufPtr, eap_forceit: c_int) -> c_int;
    pub fn nvim_al_bufref_create(buf: BufPtr) -> BufrefPtr;
    pub fn nvim_al_bufref_valid(br: BufrefPtr) -> c_int;
    pub fn nvim_al_bufref_destroy(br: BufrefPtr);
    pub fn nvim_al_set_bufref(br: BufrefPtr, buf: BufPtr);
    pub fn nvim_al_ONE_WINDOW() -> c_int;
    pub fn nvim_al_is_aucmd_win(wp: WinPtr) -> c_int;
    pub fn nvim_al_reset_VIsual_and_resel();
    pub fn nvim_al_xcalloc(count: usize, size: usize) -> *mut c_void;
    pub fn nvim_al_tabpage_index(tp: TabpagePtr) -> c_int;
    pub fn nvim_al_get_p_tpm() -> c_int;
    pub fn nvim_al_get_p_ea() -> c_int;
    pub fn nvim_al_set_p_ea(val: c_int);
    pub fn nvim_al_set_cmdmod_cmod_tab(val: c_int);
    pub fn nvim_al_get_cmdwin_type() -> c_int;
    pub fn nvim_al_get_autocmd_no_enter() -> c_int;
    pub fn nvim_al_set_autocmd_no_enter(val: c_int);
    pub fn nvim_al_get_autocmd_no_leave() -> c_int;
    pub fn nvim_al_set_autocmd_no_leave(val: c_int);
    pub fn nvim_al_get_tabpage_move_disallowed() -> c_int;
    pub fn nvim_al_set_tabpage_move_disallowed(val: c_int);
    pub fn nvim_al_tp_get_next(tp: TabpagePtr) -> TabpagePtr;
    pub fn nvim_al_foreach_windows_in_tab(
        cb: extern "C" fn(WinPtr, *mut c_void),
        tp: TabpagePtr,
        ud: *mut c_void,
    );
    pub fn nvim_al_buf_get_changed(buf: BufPtr) -> c_int;
    pub fn nvim_al_set_lastused_tabpage(tp: TabpagePtr);
}

// =============================================================================
// Phase 9: VimL Function Accessors
// =============================================================================

extern "C" {
    pub fn nvim_al_tv_get_type(tv: TypvalPtr) -> c_int;
    pub fn nvim_al_tv_get_number(tv: TypvalPtr) -> i64;
    pub fn nvim_al_tv_get_number_chk(tv: TypvalPtr, error: *mut c_int) -> i64;
    pub fn nvim_al_rettv_set_number(rettv: TypvalPtr, val: i64);
    pub fn nvim_al_rettv_set_string(rettv: TypvalPtr, s: *mut c_char);
    pub fn nvim_al_rettv_set_type(rettv: TypvalPtr, typ: c_int);
    pub fn nvim_al_tv_list_alloc_ret(rettv: TypvalPtr, len: c_int);
    pub fn nvim_al_tv_list_append_string(rettv: TypvalPtr, s: *const c_char, len: i64);
    pub fn nvim_al_find_win_by_nr_or_id(tv: TypvalPtr) -> WinPtr;
    pub fn nvim_al_find_tabwin(tv_tab: TypvalPtr, tv_win: TypvalPtr) -> WinPtr;
    pub fn nvim_al_win_get_alist_id(wp: WinPtr) -> c_int;
    pub fn nvim_al_tv_idx(tv: TypvalPtr, idx: c_int) -> TypvalPtr;
}

// =============================================================================
// Additional utility accessors (for arglist_del_files, do_arglist etc.)
// =============================================================================

extern "C" {
    pub fn nvim_al_emsg_e_cmdwin();
}
