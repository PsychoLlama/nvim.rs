//! `VimL` function implementations for command-line completion.
//!
//! Provides `f_getcompletion`, `f_getcompletiontype`, and `f_cmdcomplete_info`.

use libc::{c_char, c_int};

// =============================================================================
// Constants
// =============================================================================

/// `EXPAND_NOTHING` — unrecognised context (value 0).
const EXPAND_NOTHING: c_int = 0;

/// `EXPAND_USER_DEFINED` — custom completion function (value 30).
const EXPAND_USER_DEFINED: c_int = 30;

/// `EXPAND_USER_LIST` — custom list completion function (value 31).
const EXPAND_USER_LIST: c_int = 31;

/// `EXPAND_MENUS` — menu completion (value 11).
const EXPAND_MENUS: c_int = 11;

/// `EXPAND_SIGN` — sign completion (value 34).
const EXPAND_SIGN: c_int = 34;

/// `EXPAND_RUNTIME` — runtime completion (value 51).
const EXPAND_RUNTIME: c_int = 51;

/// `EXPAND_SHELLCMDLINE` — shell command line completion (value 57).
const EXPAND_SHELLCMDLINE: c_int = 57;

/// `EXPAND_FILETYPECMD` — filetype command completion (value 59).
const EXPAND_FILETYPECMD: c_int = 59;

/// `EXPAND_LUA` — Lua completion (value 63).
const EXPAND_LUA: c_int = 63;

/// `WILD_SILENT` option flag.
const WILD_SILENT: c_int = 0x40;

/// `WILD_USE_NL` option flag.
const WILD_USE_NL: c_int = 0x04;

/// `WILD_ADD_SLASH` option flag.
const WILD_ADD_SLASH: c_int = 0x10;

/// `WILD_NO_BEEP` option flag.
const WILD_NO_BEEP: c_int = 0x08;

/// `WILD_HOME_REPLACE` option flag.
const WILD_HOME_REPLACE: c_int = 0x02;

/// `WILD_ICASE` option flag.
const WILD_ICASE: c_int = 0x100;

/// `WILD_KEEP_ALL` option flag.
const WILD_KEEP_ALL: c_int = 0x20;

/// `WILD_ALL_KEEP` mode.
const WILD_ALL_KEEP: c_int = 8;

/// `OK` return value.
const OK: c_int = 1;

/// `FAIL` return value.
const FAIL: c_int = 0;

// =============================================================================
// Opaque handles
// =============================================================================

/// Opaque handle for `typval_T *`.
type TypvalHandle = *mut libc::c_void;

/// Opaque handle for `list_T *`.
type ListHandle = *mut libc::c_void;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    /// Get `v_type` field of argvars[idx].
    fn nvim_cmdexpand_tv_get_type(argvars: TypvalHandle, idx: c_int) -> c_int;

    /// Check that argvars[idx] is a string. Returns FAIL on error.
    fn nvim_cmdexpand_tv_check_for_string_arg(argvars: TypvalHandle, idx: c_int) -> c_int;

    /// Get string value from argvars[idx].
    fn nvim_cmdexpand_tv_get_string(argvars: TypvalHandle, idx: c_int) -> *const c_char;

    /// Get number from argvars[idx] with error check. Sets `*errorp` to 1 on error.
    fn nvim_cmdexpand_tv_get_number_chk(
        argvars: TypvalHandle,
        idx: c_int,
        errorp: *mut c_int,
    ) -> i64;

    /// Allocate a list and set rettv.
    fn nvim_cmdexpand_tv_list_alloc_ret(rettv: TypvalHandle, count: c_int);

    /// Append string to rettv->vval.v_list.
    fn nvim_cmdexpand_tv_list_append_string(rettv: TypvalHandle, str_: *const c_char, len: i64);

    /// Set rettv to `VAR_STRING` with value `str` (takes ownership).
    fn nvim_cmdexpand_tv_set_string(rettv: TypvalHandle, str_: *mut c_char);

    /// Allocate a dict and set rettv.
    fn nvim_cmdexpand_tv_dict_alloc_ret(rettv: TypvalHandle);

    /// Add string to rettv->vval.v_dict. Returns OK or FAIL.
    fn nvim_cmdexpand_tv_dict_add_str(
        rettv: TypvalHandle,
        key: *const c_char,
        klen: usize,
        val: *const c_char,
    ) -> c_int;

    /// Add number to rettv->vval.v_dict. Returns OK or FAIL.
    fn nvim_cmdexpand_tv_dict_add_nr(
        rettv: TypvalHandle,
        key: *const c_char,
        klen: usize,
        val: i64,
    ) -> c_int;

    /// Allocate a list, add to dict, and return the list handle.
    fn nvim_cmdexpand_tv_dict_add_list(
        rettv: TypvalHandle,
        key: *const c_char,
        klen: usize,
        count: c_int,
    ) -> ListHandle;

    /// Append string to a `list_T` directly.
    fn nvim_cmdexpand_list_append_string(li: ListHandle, str_: *const c_char, len: i64);

    /// Get `pum_visible()` return value.
    fn nvim_cmdexpand_pum_visible() -> c_int;

    /// Get `xp_selected` from ccline->xpc.
    fn nvim_cmdexpand_get_ccline_xp_selected() -> c_int;

    /// Get `xp_numfiles` from ccline->xpc. Returns -1 if no xpc.
    fn nvim_cmdexpand_get_ccline_xp_numfiles() -> c_int;

    /// Get `xp_files[idx]` from ccline->xpc. Returns NULL if out of range.
    fn nvim_cmdexpand_get_ccline_xp_file(idx: c_int) -> *const c_char;

    /// Check ccline->xpc->xp_files is not NULL.
    fn nvim_cmdexpand_ccline_has_xp_files() -> c_int;

    /// Convert completion type string to context int.
    fn nvim_cmdexpand_cmdcomplete_str_to_type(type_: *const c_char) -> c_int;

    /// Convert completion context int + arg to string. Returns xstrdup.
    fn nvim_cmdexpand_cmdcomplete_type_to_str(ctx: c_int, arg: *const c_char) -> *mut c_char;

    /// Get `cmdline_orig` static.
    fn nvim_cmdexpand_get_cmdline_orig() -> *const c_char;

    /// `set_context_in_menu_cmd` wrapper.
    fn nvim_cmdexpand_set_context_in_menu_cmd(
        xp: *mut crate::ExpandT,
        cmd: *const c_char,
        arg: *mut c_char,
        delim_optional: bool,
    );

    /// `set_context_in_sign_cmd` wrapper.
    fn nvim_cmdexpand_set_context_in_sign_cmd(xp: *mut crate::ExpandT, arg: *mut c_char);

    /// `set_context_in_runtime_cmd` wrapper.
    fn nvim_cmdexpand_set_context_in_runtime_cmd(xp: *mut crate::ExpandT, arg: *mut c_char);

    /// Set `filetype_expand_what = EXP_FILETYPECMD_ALL`.
    fn nvim_cmdexpand_set_filetype_expand_all();

    /// `emsg(_(e_invarg))` wrapper.
    fn nvim_cmdexpand_emsg_invarg();

    /// `semsg(_(e_invarg2), type)` wrapper.
    fn nvim_cmdexpand_semsg_invarg2(type_: *const c_char);

    /// Get `p_wic` option value.
    fn nvim_cmdexpand_get_p_wic() -> c_int;

    /// `xmemdupz(s, len)` wrapper.
    fn nvim_cmdexpand_xmemdupz(s: *const c_char, len: usize) -> *mut c_char;

    /// `addstar(pattern, len, context)` wrapper.
    fn nvim_cmdexpand_addstar(fname: *mut c_char, len: usize, context: c_int) -> *mut c_char;

    /// `nlua_expand_pat(xp)` wrapper.
    fn nvim_cmdexpand_nlua_expand_pat(xp: *mut crate::ExpandT);

    /// `ExpandOne(xp, str, orig, options, mode)` — callable via C ABI.
    fn ExpandOne(
        xp: *mut crate::ExpandT,
        str_: *mut c_char,
        orig: *mut c_char,
        options: c_int,
        mode: c_int,
    ) -> *mut c_char;

    /// `ExpandCleanup(xp)` — callable via C ABI.
    fn ExpandCleanup(xp: *mut crate::ExpandT);

    /// `ExpandInit(xp)` — callable via C ABI.
    fn ExpandInit(xp: *mut crate::ExpandT);

    fn xfree(ptr: *mut libc::c_void);

    /// Check if cmdline fuzzy completion is supported for this context.
    fn rs_cmdline_fuzzy_completion_supported(ctx: c_int) -> c_int;

    /// Rust accessor: `set_context_for_wildcard_arg`.
    fn rs_set_context_for_wildcard_arg(
        arg: *const c_char,
        is_shell_cmd: c_int,
        xp: *mut crate::ExpandT,
        context: *mut c_int,
    );

    /// `VAR_UNKNOWN` constant.
    fn nvim_cmdexpand_get_var_unknown() -> c_int;
}

// =============================================================================
// f_getcompletion
// =============================================================================

/// `getcompletion()` `VimL` function.
///
/// # Safety
///
/// `argvars` and `rettv` must be valid `typval_T *` pointers.
/// `fptr` is unused and may be null.
///
/// # Panics
///
/// Does not panic under normal operation.
#[unsafe(export_name = "f_getcompletion")]
pub unsafe extern "C" fn rs_f_getcompletion(
    argvars: TypvalHandle,
    rettv: TypvalHandle,
    _fptr: *mut libc::c_void,
) {
    let mut xpc = crate::ExpandT::zeroed();
    let mut filtered = false;
    let mut options = WILD_SILENT | WILD_USE_NL | WILD_ADD_SLASH | WILD_NO_BEEP | WILD_HOME_REPLACE;

    if nvim_cmdexpand_tv_check_for_string_arg(argvars, 1) == FAIL {
        return;
    }
    let type_ = nvim_cmdexpand_tv_get_string(argvars, 1);

    let var_unknown = nvim_cmdexpand_get_var_unknown();
    if nvim_cmdexpand_tv_get_type(argvars, 2) != var_unknown {
        filtered = nvim_cmdexpand_tv_get_number_chk(argvars, 2, std::ptr::null_mut()) != 0;
    }

    if nvim_cmdexpand_get_p_wic() != 0 {
        options |= WILD_ICASE;
    }

    // For filtered results, 'wildignore' is used; otherwise keep all.
    if !filtered {
        options |= WILD_KEEP_ALL;
    }

    if nvim_cmdexpand_tv_get_type(argvars, 0) != crate::VAR_STRING {
        nvim_cmdexpand_emsg_invarg();
        return;
    }
    let pattern = nvim_cmdexpand_tv_get_string(argvars, 0);
    let pattern_start = pattern;

    if libc::strcmp(type_, c"cmdline".as_ptr()) == 0 {
        let cmdline_len = libc::strlen(pattern) as c_int;
        crate::rs_set_cmd_context(
            &raw mut xpc,
            pattern.cast_mut(),
            cmdline_len,
            cmdline_len,
            0,
        );
        // Jump to theend
        complete_getcompletion(&raw mut xpc, rettv, options, pattern_start, pattern_start);
        ExpandCleanup(&raw mut xpc);
        return;
    }

    ExpandInit(&raw mut xpc);
    xpc.xp_pattern = pattern.cast_mut();
    xpc.xp_pattern_len = libc::strlen(pattern);
    xpc.xp_line = pattern.cast_mut();

    xpc.xp_context = nvim_cmdexpand_cmdcomplete_str_to_type(type_);

    if xpc.xp_context == EXPAND_NOTHING {
        nvim_cmdexpand_semsg_invarg2(type_);
        return;
    }

    if xpc.xp_context == EXPAND_USER_DEFINED {
        if libc::strncmp(type_, c"custom,".as_ptr(), 7) != 0 {
            nvim_cmdexpand_semsg_invarg2(type_);
            return;
        }
        xpc.xp_arg = type_.add(7).cast_mut();
    } else if xpc.xp_context == EXPAND_USER_LIST {
        if libc::strncmp(type_, c"customlist,".as_ptr(), 11) != 0 {
            nvim_cmdexpand_semsg_invarg2(type_);
            return;
        }
        xpc.xp_arg = type_.add(11).cast_mut();
    } else if xpc.xp_context == EXPAND_MENUS {
        nvim_cmdexpand_set_context_in_menu_cmd(
            &raw mut xpc,
            c"menu".as_ptr(),
            xpc.xp_pattern,
            false,
        );
        xpc.xp_pattern_len -= (xpc.xp_pattern as usize) - (pattern_start as usize);
    } else if xpc.xp_context == EXPAND_SIGN {
        nvim_cmdexpand_set_context_in_sign_cmd(&raw mut xpc, xpc.xp_pattern);
        xpc.xp_pattern_len -= (xpc.xp_pattern as usize) - (pattern_start as usize);
    } else if xpc.xp_context == EXPAND_RUNTIME {
        nvim_cmdexpand_set_context_in_runtime_cmd(&raw mut xpc, xpc.xp_pattern);
        xpc.xp_pattern_len -= (xpc.xp_pattern as usize) - (pattern_start as usize);
    } else if xpc.xp_context == EXPAND_SHELLCMDLINE {
        let mut context = EXPAND_SHELLCMDLINE;
        rs_set_context_for_wildcard_arg(xpc.xp_pattern, 0, &raw mut xpc, &raw mut context);
        xpc.xp_pattern_len -= (xpc.xp_pattern as usize) - (pattern_start as usize);
    } else if xpc.xp_context == EXPAND_FILETYPECMD {
        nvim_cmdexpand_set_filetype_expand_all();
    }

    complete_getcompletion(&raw mut xpc, rettv, options, pattern_start, pattern_start);
    ExpandCleanup(&raw mut xpc);
}

/// Common tail of `f_getcompletion` after context setup.
///
/// # Safety
///
/// `xpc`, `rettv`, `pattern_start` must be valid pointers.
unsafe fn complete_getcompletion(
    xpc: *mut crate::ExpandT,
    rettv: TypvalHandle,
    options: c_int,
    pattern_start: *const c_char,
    _orig_pattern: *const c_char,
) {
    if (*xpc).xp_context == EXPAND_LUA {
        (*xpc).xp_col = libc::strlen((*xpc).xp_line) as c_int;
        nvim_cmdexpand_nlua_expand_pat(xpc);
        (*xpc).xp_pattern_len -= ((*xpc).xp_pattern as usize) - (pattern_start as usize);
    }

    let pat = if rs_cmdline_fuzzy_completion_supported((*xpc).xp_context) != 0 {
        // Don't modify the search string for fuzzy matching
        nvim_cmdexpand_xmemdupz((*xpc).xp_pattern, (*xpc).xp_pattern_len)
    } else {
        nvim_cmdexpand_addstar((*xpc).xp_pattern, (*xpc).xp_pattern_len, (*xpc).xp_context)
    };

    ExpandOne(xpc, pat, std::ptr::null_mut(), options, WILD_ALL_KEEP);
    nvim_cmdexpand_tv_list_alloc_ret(rettv, (*xpc).xp_numfiles);

    for i in 0..(*xpc).xp_numfiles {
        let s = *(*xpc).xp_files.add(i as usize);
        nvim_cmdexpand_tv_list_append_string(rettv, s, -1);
    }
    xfree(pat.cast::<libc::c_void>());
}

// =============================================================================
// f_getcompletiontype
// =============================================================================

/// `getcompletiontype()` `VimL` function.
///
/// # Safety
///
/// `argvars` and `rettv` must be valid `typval_T *` pointers.
/// `fptr` is unused and may be null.
///
/// # Panics
///
/// Does not panic under normal operation.
#[unsafe(export_name = "f_getcompletiontype")]
pub unsafe extern "C" fn rs_f_getcompletiontype(
    argvars: TypvalHandle,
    rettv: TypvalHandle,
    _fptr: *mut libc::c_void,
) {
    // Set rettv to VAR_STRING with NULL (default)
    nvim_cmdexpand_tv_set_string(rettv, std::ptr::null_mut());

    if nvim_cmdexpand_tv_check_for_string_arg(argvars, 0) == FAIL {
        return;
    }

    let pat = nvim_cmdexpand_tv_get_string(argvars, 0);
    let mut xpc = crate::ExpandT::zeroed();
    ExpandInit(&raw mut xpc);

    let cmdline_len = libc::strlen(pat) as c_int;
    crate::rs_set_cmd_context(&raw mut xpc, pat.cast_mut(), cmdline_len, cmdline_len, 0);
    let result_str = nvim_cmdexpand_cmdcomplete_type_to_str(xpc.xp_context, xpc.xp_arg);
    nvim_cmdexpand_tv_set_string(rettv, result_str);

    ExpandCleanup(&raw mut xpc);
}

// =============================================================================
// f_cmdcomplete_info
// =============================================================================

/// `cmdcomplete_info()` `VimL` function.
///
/// # Safety
///
/// `argvars` and `rettv` must be valid `typval_T *` pointers.
/// `fptr` is unused and may be null.
///
/// # Panics
///
/// Does not panic under normal operation.
#[unsafe(export_name = "f_cmdcomplete_info")]
pub unsafe extern "C" fn rs_f_cmdcomplete_info(
    _argvars: TypvalHandle,
    rettv: TypvalHandle,
    _fptr: *mut libc::c_void,
) {
    nvim_cmdexpand_tv_dict_alloc_ret(rettv);

    if nvim_cmdexpand_ccline_has_xp_files() == 0 {
        return;
    }

    let cmdline_orig = nvim_cmdexpand_get_cmdline_orig();
    let mut ret = nvim_cmdexpand_tv_dict_add_str(
        rettv,
        c"cmdline_orig".as_ptr(),
        12,
        if cmdline_orig.is_null() {
            c"".as_ptr()
        } else {
            cmdline_orig
        },
    );
    if ret == OK {
        ret = nvim_cmdexpand_tv_dict_add_nr(
            rettv,
            c"pum_visible".as_ptr(),
            11,
            i64::from(nvim_cmdexpand_pum_visible()),
        );
    }
    if ret == OK {
        ret = nvim_cmdexpand_tv_dict_add_nr(
            rettv,
            c"selected".as_ptr(),
            8,
            i64::from(nvim_cmdexpand_get_ccline_xp_selected()),
        );
    }
    if ret == OK {
        let num_files = nvim_cmdexpand_get_ccline_xp_numfiles();
        let li = nvim_cmdexpand_tv_dict_add_list(rettv, c"matches".as_ptr(), 7, num_files.max(0));
        let mut idx = 0;
        while ret == OK && idx < num_files {
            let s = nvim_cmdexpand_get_ccline_xp_file(idx);
            if !s.is_null() {
                nvim_cmdexpand_list_append_string(li, s, -1);
            }
            idx += 1;
        }
    }
}
