//! Provider infrastructure for VimL: eval_has_provider, eval_call_provider,
//! and script_host_eval.
//!
//! Migrated from `eval_shim.c` Phase 3 (eval_shim pass 6).

#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]

use std::ffi::{c_char, c_int, c_void, CStr};

use crate::funcexe::FuncExeT;

// =============================================================================
// C Extern Declarations
// =============================================================================

extern "C" {
    // ----- provider infrastructure -----
    fn nvim_eval_nlua_is_deferred_safe() -> bool;
    // nvim_semsg_fast_api_disabled: now in nvim_eval::errors
    fn nvim_eval_variable(
        name: *const c_char,
        len: c_int,
        rettv: *mut c_void,
        verbose: bool,
        import_script: bool,
    ) -> c_int;
    fn nvim_script_autoload(name: *const c_char, name_len: usize, reload: bool) -> bool;
    fn nvim_eval_find_func(name: *const c_char) -> bool;
    fn nvim_eval_get_p_lpl() -> bool;
    // nvim_semsg_provider_*: now in nvim_eval::errors

    // ----- typval accessors -----
    fn nvim_tv_get_type(tv: *mut c_void) -> c_int;
    fn nvim_eval_tv_get_vnumber(tv: *const c_void) -> i64;
    fn nvim_tv_alloc_zero() -> *mut c_void;
    fn nvim_tv_set_number_zero(tv: *mut c_void);
    fn tv_clear(tv: *mut c_void);
    fn nvim_tv_get_vstring(tv: *mut c_void) -> *mut c_char;

    // ----- list operations -----
    fn nvim_eval_list_alloc_n(n: c_int) -> *mut c_void;
    fn nvim_tv_list_append_string(l: *mut c_void, str: *const c_char, len: isize);
    fn nvim_tv_list_unref(l: *mut c_void);
    fn nvim_eval_list_ref(l: *mut c_void);

    // ----- provider caller scope save/restore -----
    fn nvim_save_provider_caller_scope() -> *mut c_void;
    fn nvim_restore_provider_caller_scope(saved: *mut c_void);
    fn nvim_provider_call_nesting_inc();
    fn nvim_provider_call_nesting_dec();

    // ----- funccal save/restore -----
    fn nvim_eval_save_funccal() -> *mut c_void;
    fn nvim_eval_restore_funccal(entry: *mut c_void);

    // ----- direct call_func (replaces nvim_eval_provider_call_func) -----
    fn call_func(
        funcname: *const c_char,
        len: c_int,
        rettv: *mut c_void,
        argcount: c_int,
        argvars: *mut c_void,
        funcexe: *mut FuncExeT,
    ) -> c_int;
    fn nvim_curwin_get_cursor_lnum() -> i32;
    fn nvim_tv_set_type(tv: *mut c_void, vtype: c_int);
    fn nvim_tv_set_vstring_raw(tv: *mut c_void, s: *mut c_char);
    fn nvim_tv_set_list(tv: *mut c_void, list: *mut c_void);

    // ----- string utilities -----
    #[link_name = "strchrsub"]
    fn rs_strchrsub(str: *mut c_char, c: c_char, x: c_char);

    // ----- security check -----
    fn rs_check_secure() -> c_int;

    // ----- error messages -----
    fn emsg(s: *const c_char) -> c_int;
    fn nvim_gettext(s: *const c_char) -> *const c_char;
    fn xfree(ptr: *mut c_void);
}

// =============================================================================
// VarType constants
// =============================================================================

const VAR_NUMBER: c_int = 1;
const VAR_STRING: c_int = 2;
const FAIL: c_int = 0;

// =============================================================================
// Provider name table
// =============================================================================

/// Known provider feature names.
static PROVIDERS: &[&[u8]] = &[
    b"clipboard\0",
    b"python3\0",
    b"python3_compiled\0",
    b"python3_dynamic\0",
    b"perl\0",
    b"ruby\0",
    b"node\0",
];

/// Check if `feat` is one of the known provider feature names.
fn is_known_provider(feat: &[u8]) -> bool {
    PROVIDERS.iter().any(|&p| {
        let p = &p[..p.len() - 1]; // strip NUL
        p == feat
    })
}

// =============================================================================
// Phase 3: rs_eval_has_provider
// =============================================================================

/// Check if the provider for feature `feat` is enabled.
///
/// # Safety
/// - `feat` must be a valid null-terminated C string.
/// - `throw_if_fast` controls whether a "fast API disabled" error is thrown.
#[export_name = "eval_has_provider"]
pub unsafe extern "C" fn rs_eval_has_provider(feat: *const c_char, throw_if_fast: bool) -> bool {
    let feat_bytes = unsafe { CStr::from_ptr(feat) }.to_bytes();

    // Only handle known provider names; other has() features skip autoload.
    if !is_known_provider(feat_bytes) {
        return false;
    }

    if throw_if_fast && !unsafe { nvim_eval_nlua_is_deferred_safe() } {
        unsafe { nvim_eval::errors::semsg_fast_api_disabled() };
        return false;
    }

    // Normalize name: "python3_compiled" => "python3" (chop at '_')
    let mut name_buf = [0u8; 32];
    let copy_len = feat_bytes.len().min(31);
    name_buf[..copy_len].copy_from_slice(&feat_bytes[..copy_len]);
    // name_buf is NUL-terminated by zero-initialization

    // Chop at '_' using rs_strchrsub(name, '_', NUL)
    unsafe { rs_strchrsub(name_buf.as_mut_ptr() as *mut c_char, b'_' as c_char, 0) };

    let name_ptr = name_buf.as_ptr() as *const c_char;
    let name_bytes = unsafe { CStr::from_ptr(name_ptr) }.to_bytes();
    let name_len = name_bytes.len();

    // Build "g:loaded_<name>_provider"
    let mut varname_buf = [0u8; 256];
    let prefix = b"g:loaded_";
    let suffix = b"_provider";
    let mut pos = 0;
    varname_buf[pos..pos + prefix.len()].copy_from_slice(prefix);
    pos += prefix.len();
    varname_buf[pos..pos + name_len].copy_from_slice(name_bytes);
    pos += name_len;
    varname_buf[pos..pos + suffix.len()].copy_from_slice(suffix);
    pos += suffix.len();
    let varname_len = pos as c_int;

    let rettv = unsafe { nvim_tv_alloc_zero() };

    // Try to get g:loaded_<name>_provider
    if unsafe {
        nvim_eval_variable(
            varname_buf.as_ptr() as *const c_char,
            varname_len,
            rettv,
            false,
            true,
        )
    } == FAIL
    {
        // Not found: trigger autoload once with "provider#<name>#bogus"
        let mut autoload_buf = [0u8; 256];
        let autoload_prefix = b"provider#";
        let autoload_suffix = b"#bogus";
        let mut ap = 0;
        autoload_buf[ap..ap + autoload_prefix.len()].copy_from_slice(autoload_prefix);
        ap += autoload_prefix.len();
        autoload_buf[ap..ap + name_len].copy_from_slice(name_bytes);
        ap += name_len;
        autoload_buf[ap..ap + autoload_suffix.len()].copy_from_slice(autoload_suffix);
        ap += autoload_suffix.len();

        unsafe { nvim_script_autoload(autoload_buf.as_ptr() as *const c_char, ap, false) };

        // Retry the variable lookup
        if unsafe {
            nvim_eval_variable(
                varname_buf.as_ptr() as *const c_char,
                varname_len,
                rettv,
                false,
                true,
            )
        } == FAIL
        {
            // Show a hint if Call() is defined but g:loaded_<name>_provider is missing
            let mut call_buf = [0u8; 256];
            let call_prefix = b"provider#";
            let call_suffix = b"#Call";
            let mut cp = 0;
            call_buf[cp..cp + call_prefix.len()].copy_from_slice(call_prefix);
            cp += call_prefix.len();
            call_buf[cp..cp + name_len].copy_from_slice(name_bytes);
            cp += name_len;
            call_buf[cp..cp + call_suffix.len()].copy_from_slice(call_suffix);
            let _ = cp;

            if unsafe { nvim_eval_find_func(call_buf.as_ptr() as *const c_char) }
                && unsafe { nvim_eval_get_p_lpl() }
            {
                unsafe { nvim_eval::errors::semsg_provider_missing_var(name_ptr) };
            }

            unsafe { xfree(rettv) };
            return false;
        }
    }

    // Check: v_type == VAR_NUMBER && v_number == 2 means "loaded and working"
    let tv_type = unsafe { nvim_tv_get_type(rettv) };
    let ok = if tv_type == VAR_NUMBER {
        let vnum = unsafe { nvim_eval_tv_get_vnumber(rettv as *const c_void) };
        vnum == 2
    } else {
        false
    };

    unsafe { tv_clear(rettv) };
    unsafe { xfree(rettv) };

    if ok {
        // Call() must be defined if provider claims to be working
        let mut call_buf = [0u8; 256];
        let call_prefix = b"provider#";
        let call_suffix = b"#Call";
        let mut cp = 0;
        call_buf[cp..cp + call_prefix.len()].copy_from_slice(call_prefix);
        cp += call_prefix.len();
        call_buf[cp..cp + name_len].copy_from_slice(name_bytes);
        cp += name_len;
        call_buf[cp..cp + call_suffix.len()].copy_from_slice(call_suffix);
        let _ = cp;

        if !unsafe { nvim_eval_find_func(call_buf.as_ptr() as *const c_char) } {
            unsafe {
                nvim_eval::errors::semsg_provider_no_call(
                    name_ptr,
                    call_buf.as_ptr() as *const c_char,
                )
            };
            return false;
        }
    }

    ok
}

// =============================================================================
// Phase 3: rs_eval_call_provider
// =============================================================================

/// Call a provider's function and return the result typval.
///
/// The result is written to `out_rettv` (a typval_T pointer passed by the C wrapper).
/// If `discard` is true, the result is cleared before returning.
///
/// # Safety
/// - `provider` and `method` must be valid null-terminated C strings.
/// - `arguments` must be a valid list_T pointer.
/// - `out_rettv` must be a valid typval_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_call_provider(
    provider: *const c_char,
    method: *const c_char,
    arguments: *mut c_void,
    discard: bool,
    out_rettv: *mut c_void,
) {
    if !unsafe { rs_eval_has_provider(provider, false) } {
        unsafe { nvim_eval::errors::semsg_no_provider(provider) };
        unsafe { nvim_tv_set_number_zero(out_rettv) };
        return;
    }

    // Build function name: "provider#<name>#Call"
    let mut func_buf = [0u8; 256];
    let prefix = b"provider#";
    let suffix = b"#Call";

    let provider_bytes = unsafe { CStr::from_ptr(provider) }.to_bytes();
    let provider_len = provider_bytes.len();

    let mut pos = 0;
    func_buf[pos..pos + prefix.len()].copy_from_slice(prefix);
    pos += prefix.len();
    func_buf[pos..pos + provider_len].copy_from_slice(provider_bytes);
    pos += provider_len;
    func_buf[pos..pos + suffix.len()].copy_from_slice(suffix);
    pos += suffix.len();
    let name_len = pos as c_int;

    // Save caller scope
    let saved_scope = unsafe { nvim_save_provider_caller_scope() };
    let funccal_entry = unsafe { nvim_eval_save_funccal() };
    unsafe { nvim_provider_call_nesting_inc() };

    // Ref the arguments list for the call
    unsafe { nvim_eval_list_ref(arguments) };

    // Build argvars[3]: [VAR_STRING method, VAR_LIST arguments, VAR_UNKNOWN]
    // typval_T is 16 bytes: v_type(4) + v_lock(4) + vval(8)
    // VAR_STRING=2, VAR_LIST=4, VAR_UNKNOWN=0, VAR_UNLOCKED=0
    let mut argvars = [0u8; 48usize]; // 3 * 16 bytes, zeroed
    let av0 = argvars.as_mut_ptr() as *mut c_void;
    // av2 stays all-zero (VAR_UNKNOWN=0, VAR_UNLOCKED=0, vval=null)
    let lnum = unsafe { nvim_curwin_get_cursor_lnum() };
    let mut funcexe = FuncExeT::new();
    funcexe.fe_firstline = lnum;
    funcexe.fe_lastline = lnum;
    funcexe.fe_evaluate = true;
    unsafe {
        let av1 = argvars.as_mut_ptr().add(16) as *mut c_void;
        nvim_tv_set_type(av0, 2); // VAR_STRING
        nvim_tv_set_vstring_raw(av0, method as *mut c_char);
        nvim_tv_set_type(av1, 4); // VAR_LIST
        nvim_tv_set_list(av1, arguments);
        // Set out_rettv to VAR_UNKNOWN/VAR_UNLOCKED before call
        nvim_tv_set_type(out_rettv, 0); // VAR_UNKNOWN
        call_func(
            func_buf.as_ptr() as *const c_char,
            name_len,
            out_rettv,
            2,
            av0,
            &mut funcexe,
        );
    };

    // Unref arguments
    unsafe { nvim_tv_list_unref(arguments) };

    // Restore
    unsafe { nvim_eval_restore_funccal(funccal_entry) };
    unsafe { nvim_restore_provider_caller_scope(saved_scope) };
    unsafe { nvim_provider_call_nesting_dec() };

    if discard {
        unsafe { tv_clear(out_rettv) };
    }
}

// =============================================================================
// Phase 3: rs_script_host_eval
// =============================================================================

/// Evaluate an expression via a script host provider.
///
/// # Safety
/// - `name` must be a valid null-terminated C string.
/// - `argvars` must point to at least 1 typval_T.
/// - `rettv` must be a valid typval_T pointer.
#[export_name = "script_host_eval"]
pub unsafe extern "C" fn rs_script_host_eval(
    name: *const c_char,
    argvars: *mut c_void,
    rettv: *mut c_void,
) {
    if unsafe { rs_check_secure() } != 0 {
        return;
    }

    // argvars[0] must be a string
    if unsafe { nvim_tv_get_type(argvars) } != VAR_STRING {
        let msg = unsafe { nvim_gettext(c"E474: Invalid argument".as_ptr()) };
        unsafe { emsg(msg) };
        return;
    }

    // Build args list with one string item: argvars[0].v_string
    let args = unsafe { nvim_eval_list_alloc_n(1) };
    // argvars[0] is a VAR_STRING typval; get its v_string field via accessor
    let s = unsafe { nvim_tv_get_vstring(argvars) };
    unsafe { nvim_tv_list_append_string(args, s, -1) };

    // Call the provider and write result into rettv
    unsafe { rs_eval_call_provider(name, c"eval".as_ptr(), args, false, rettv) };
}
