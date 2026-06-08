//! Ex-command entry points for user functions: `:call` and `:function`.
//!
//! Wave 2 Phase 2: `ex_call` migrated here from `src/nvim/eval/userfunc.c`.
//! Wave 2 Phase 4: `ex_function` migrated here from `src/nvim/eval/userfunc.c`.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::if_not_else)]
#![allow(clippy::ptr_cast_constness)]
#![allow(unused_assignments)]

use std::ffi::{c_char, c_int, c_void};

use nvim_eval_exec::eval::EvalargT;
use nvim_eval_exec::funcexe::FuncExeT;

use super::parsing::GarrayT;

// ============================================================================
// Return value constants (matching C OK/FAIL)
// ============================================================================
const FAIL: c_int = 0;
const OK: c_int = 1;

// ============================================================================
// Function flags (matching C FC_* and TFN_*)
// ============================================================================
const TFN_INT: c_int = 1;
const TFN_NO_AUTOLOAD: c_int = 4;
const FC_RANGE: c_int = 0x02;
const FC_DICT: c_int = 0x04;
const FC_ABORT: c_int = 0x01;
const FC_CLOSURE: c_int = 0x08;
// VAR_FUNC and VAR_DEF_SCOPE
const VAR_FUNC: c_int = 3;
const VAR_DEF_SCOPE: c_int = 1;
// AUTOLOAD_CHAR
const AUTOLOAD_CHAR: c_int = b'#' as c_int;

// ============================================================================
// C extern declarations — raw pointer versions to avoid private Handle types
// ============================================================================

// trans_function_name: last param is `*mut *mut c_void` here (double pointer for partial_T**)
// but other modules use `*mut c_void`. Same ABI; allow the clash.
// aborting: some modules use `-> bool`, others `-> c_int`; same ABI on this platform.
#[allow(clashing_extern_declarations)]
extern "C" {
    // eap field accessors (existing in userfunc.c)
    fn nvim_eap_get_arg(eap: *const c_void) -> *mut c_char;
    fn nvim_eap_get_skip(eap: *const c_void) -> c_int;
    fn nvim_eap_get_line1(eap: *const c_void) -> i32;
    fn nvim_eap_get_line2(eap: *const c_void) -> i32;
    fn nvim_eap_set_nextcmd(eap: *mut c_void, val: *mut c_char);
    fn nvim_eap_get_cstack(eap: *const c_void) -> *mut c_void;
    fn nvim_eap_get_cmdidx(eap: *const c_void) -> c_int;
    fn nvim_eap_get_forceit_int(eap: *const c_void) -> c_int;
    fn nvim_eap_get_cmd(eap: *const c_void) -> *mut c_char;
    fn nvim_eap_set_skip(eap: *mut c_void, v: c_int);

    // Wave 2 Phase 2 new accessors (added to userfunc.c)
    fn nvim_cstack_get_trylevel(cstack: *const c_void) -> c_int;
    fn nvim_cmd_defer_idx() -> c_int;
    fn nvim_semsg_e_missingparen(name: *const c_char);

    // fudi (funcdict_T) field accessors (existing in userfunc.c)
    fn nvim_fudi_get_dict(fudi: *const c_void) -> *mut c_void;
    fn nvim_fudi_get_newkey(fudi: *const c_void) -> *mut c_char;
    fn nvim_fudi_get_di(fudi: *const c_void) -> *mut c_void;

    // dict refcount + unref
    fn nvim_tv_dict_incr_refcount(dict: *mut c_void);
    fn tv_dict_unref(dict: *mut c_void);

    // message helpers
    fn nvim_semsg_dictkey(key: *const c_char);

    // emsg_skip global (inc/dec)
    fn nvim_syn_emsg_skip_inc();
    fn nvim_syn_emsg_skip_dec();

    // String/memory helpers
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn xfree(ptr: *mut c_void);
    fn strlen(s: *const c_char) -> usize;
    fn xmemdupz(src: *const c_void, len: usize) -> *mut c_char;
    fn vim_strchr(string: *const c_char, c: c_int) -> *mut c_char;

    // eval helpers — raw pointer signatures (these are already Rust symbols
    // via export_name; using raw *mut c_void avoids private handle types)
    fn fill_evalarg_from_eap(evalarg: *mut c_void, eap: *mut c_void, skip: bool);
    fn clear_evalarg(evalarg: *mut c_void, eap: *mut c_void);
    fn eval0(arg: *mut c_char, rettv: *mut c_void, eap: *mut c_void, evalarg: *mut c_void)
        -> c_int;
    fn tv_clear(tv: *mut c_void);

    // trans_function_name (Rust, names.rs, Phase 3)
    fn trans_function_name(
        pp: *mut *mut c_char,
        skip: c_int,
        flags: c_int,
        fdp: *mut c_void,
        partial: *mut *mut c_void,
    ) -> *mut c_char;

    // save_function_name (Rust, names.rs)
    fn save_function_name(
        name: *mut *mut c_char,
        skip: c_int,
        flags: c_int,
        fudi: *mut c_void,
    ) -> *mut c_char;

    // deref_func_name (Rust, lookup.rs)
    fn deref_func_name(
        name: *const c_char,
        lenp: *mut c_int,
        partialp: *mut *mut c_void,
        no_autoload: c_int,
        found_var: *mut bool,
    ) -> *mut c_char;

    // rs_ex_defer_inner (Rust, defer.rs)
    fn rs_ex_defer_inner(
        name: *mut c_char,
        arg: *mut *mut c_char,
        partial: *const c_void,
        evalarg: *mut c_void,
    ) -> c_int;

    // ex_call_inner (Rust, funccal.rs; exported as "ex_call_inner")
    fn ex_call_inner(
        eap: *const c_void,
        name: *const c_char,
        arg: *mut *mut c_char,
        startarg: *mut c_char,
        funcexe_init: *const c_void,
        evalarg: *mut c_void,
    ) -> c_int;

    // check_nextcmd, ends_excmd, aborting (C)
    fn check_nextcmd(p: *const c_char) -> *mut c_char;
    fn ends_excmd(c: c_int) -> c_int;
    fn aborting() -> bool;

    // emsg_severe and did_throw globals
    static mut emsg_severe: bool;
    static did_throw: bool;

    // e_trailing_arg message accessor
    fn nvim_e_trailing_arg() -> *const c_char;
    fn nvim_semsg_with_name(errmsg: *const c_char, name: *const c_char);

    // Wave 2 Phase 4: ex_function helpers
    // listing (Rust, listing.rs)
    fn rs_list_functions();
    fn rs_list_functions_matching_pat(eap: *mut c_void) -> *mut c_char;
    fn rs_list_one_function(eap: *mut c_void, name: *const c_char, p: *mut c_char) -> *mut c_void;
    // function name checks (Rust, eval crate)
    fn rs_eval_isnamec(c: c_int) -> bool;
    fn rs_eval_isnamec1(c: c_int) -> bool;
    // garray helpers (C)
    fn ga_init(gap: *mut GarrayT, itemsize: c_int, growsize: c_int);
    fn ga_clear_strings(gap: *mut c_void);
    // find_var (C)
    fn find_var(
        name: *const c_char,
        len: usize,
        ht: *mut *mut c_void,
        no_autoload: c_int,
    ) -> *mut c_void;
    // value_check_lock (C)
    fn value_check_lock(lock: c_int, name: *const c_char, name_len: usize) -> bool;
    // autoload_name, path_fnamecmp (C)
    fn autoload_name(name: *const c_char, name_len: usize) -> *mut c_char;
    fn path_fnamecmp(fname1: *const c_char, fname2: *const c_char) -> c_int;
    // tv_is_func wrapper (C, typval.c)
    fn nvim_tv_is_func(tv: *const c_void) -> bool;
    // dict entry helpers (C, typval.c)
    fn tv_dict_item_alloc(key: *const c_char) -> *mut c_void;
    fn tv_dict_add(dict: *mut c_void, item: *mut c_void) -> c_int;
    // alloc/clear ufunc (Rust via C shim / Rust direct)
    fn rs_alloc_ufunc(name: *const c_char, namelen: usize) -> *mut c_void;
    fn rs_register_closure(fp: *mut c_void);
    fn rs_func_clear_items(fp: *mut c_void);
    fn prof_def_func() -> bool;
    fn func_do_profile(fp: *mut c_void);
    // did_emsg accessors (C)
    fn nvim_get_did_emsg() -> bool;
    fn nvim_set_did_emsg(val: bool);
    // get_current_funccal (C)
    fn get_current_funccal() -> *mut c_void;
    // ui helpers (C)
    fn ui_ext_cmdline_block_append(indent: usize, s: *const c_char);
    fn ui_ext_cmdline_block_leave();
    fn msg_putchar(c: c_int);
    fn nvim_ui_has_cmdline() -> c_int;
    // get_function_args (Rust, parsing.rs)
    fn get_function_args(
        argp: *mut *mut c_char,
        endchar: c_char,
        newargs: *mut GarrayT,
        varargs: *mut c_int,
        default_args: *mut GarrayT,
        skip: c_int,
    ) -> c_int;
    // get_function_body (C shim → Rust body.rs)
    fn get_function_body(
        eap: *mut c_void,
        newlines: *mut GarrayT,
        line_arg_in: *mut c_char,
        line_to_free: *mut *mut c_char,
        show_block: bool,
    ) -> c_int;
    // emsg helpers (C) - kept for potential use; suppress dead_code warning
    #[allow(dead_code)]
    fn emsg_funcname(errmsg: *const c_char, name: *const c_char);
    // Wave 2 Phase 4 new accessors (added to userfunc.c)
    fn nvim_keytyped() -> c_int;
    fn nvim_get_msg_row() -> c_int;
    fn nvim_set_cmdline_row(v: c_int);
    fn nvim_next_func_nr() -> c_int;
    fn nvim_func_ht_overwrite_fp(name: *const c_char, fp: *mut c_void);
    fn nvim_func_ht_try_add_fp(fp: *mut c_void) -> c_int;
    fn nvim_ufunc_set_garray_fields(
        fp: *mut c_void,
        args: *mut GarrayT,
        def_args: *mut GarrayT,
        lines: *mut GarrayT,
    );
    fn nvim_ufunc_finalize_user_func(
        fp: *mut c_void,
        varargs: c_int,
        flags: c_int,
        sourcing_lnum_top: i32,
    );
    fn nvim_get_sourcing_name() -> *const c_char;
    fn nvim_get_sourcing_lnum() -> i32;
    fn nvim_emsg_e124_missing_paren(arg: *const c_char);
    fn nvim_emsg_e707_func_name_conflict(name: *const c_char);
    fn nvim_emsg_e127_cannot_redefine(name: *const c_char);
    fn nvim_emsg_e122_func_exists(name: *const c_char);
    fn nvim_emsg_e746_autoload_mismatch(name: *const c_char);
    fn nvim_emsg_e932_closure_toplevel(name: *const c_char);
    fn nvim_emsg_e862_no_g_dict();
    fn nvim_emsg_e717_funcdict();
    fn nvim_emsg_trailing_arg(name: *const c_char);
    // dictitem accessors (C)
    fn nvim_dictitem_di_tv(di: *mut c_void) -> *mut c_void;
    fn nvim_tv_get_type(tv: *const c_void) -> c_int;
    fn nvim_tv_get_lock(tv: *const c_void) -> c_int;
    fn nvim_dict_get_scope_impl(dict: *const c_void) -> c_int;
    fn nvim_dict_get_lock(dict: *const c_void) -> c_int;
    // ufunc field accessors
    fn nvim_ufunc_get_refcount(fp: *const c_void) -> c_int;
    fn nvim_ufunc_get_calls(fp: *mut c_void) -> c_int;
    fn nvim_ufunc_get_flags(fp: *mut c_void) -> c_int;
    fn nvim_ufunc_set_flags(fp: *mut c_void, flags: c_int);
    fn nvim_ufunc_dec_refcount(fp: *mut c_void);
    fn nvim_ufunc_set_refcount(fp: *mut c_void, val: c_int);
    fn nvim_ufunc_get_name_exp(fp: *mut c_void) -> *const c_char;
    fn nvim_ufunc_set_name_exp(fp: *mut c_void, val: *mut c_char);
    fn nvim_ufunc_set_profiling(fp: *mut c_void, val: c_int);
    fn nvim_ufunc_set_prof_initialized(fp: *mut c_void, val: c_int);
    fn nvim_ufunc_get_uf_name_ptr(fp: *const c_void) -> *const c_char;
    // ufunc ga field access for re-init at erret (return *mut c_void to match other decls)
    fn nvim_ufunc_get_args_ga(fp: *mut c_void) -> *mut c_void;
    fn nvim_ufunc_get_def_args_ga(fp: *mut c_void) -> *mut c_void;
    // dictitem tv field set
    fn nvim_dictitem_set_tv_func(di: *mut c_void, s: *const c_char, len: usize);
    // XFREE_CLEAR(fp->uf_name_exp)
    fn nvim_ufunc_free_name_exp(fp: *mut c_void);
    // sc_sid / sc_seq accessors
    fn nvim_ufunc_get_script_ctx_sid(fp: *const c_void) -> c_int;
    fn nvim_ufunc_get_script_ctx_seq(fp: *const c_void) -> c_int;
    fn nvim_get_current_sctx_sid() -> c_int;
    fn nvim_current_sctx_get_seq() -> c_int;
    // e_invarg2 semsg
    fn nvim_semsg_e_invarg2(arg: *const c_char);
}

// ============================================================================
// ex_function  (Wave 2 Phase 4: migrated from userfunc.c)
// ============================================================================

// K_SPECIAL byte value (0x80 = 128)
const K_SPECIAL: u8 = 0x80;
// FC_REMOVED flag
const FC_REMOVED: c_int = 0x20;

/// `:function` ex-command entry point.
/// Defines, lists, or overrides user-defined functions.
///
/// Replaces C `ex_function` — body is now here; C file has `extern` decl only.
///
/// # Safety
/// `eap` must be a valid `exarg_T *`.
#[unsafe(export_name = "ex_function")]
pub unsafe extern "C" fn rs_ex_function(eap: *mut c_void) {
    // SAFETY: All raw pointer accesses go through C accessor functions.
    unsafe { rs_ex_function_inner(eap) }
}

unsafe fn rs_ex_function_inner(eap: *mut c_void) {
    // State tracking: which levels of cleanup are needed
    let mut line_to_free: *mut c_char = std::ptr::null_mut();
    let mut line_arg: *mut c_char = std::ptr::null_mut();
    // GarrayT default (zero-initialized)
    let ga_zero = GarrayT {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: std::ptr::null_mut(),
    };
    let mut newargs = ga_zero;
    let mut default_args = ga_zero;
    let mut newlines = ga_zero;
    let mut varargs: c_int = 0;
    let mut flags: c_int = 0;
    let mut fp: *mut c_void = std::ptr::null_mut();
    let mut free_fp = false;
    let mut overwrite = false;
    // funcdict_T fudi: 3 pointers (dict_T*, char*, dictitem_T*) = 24 bytes, zero-init
    let mut fudi = [0u8; 24usize];
    let fudi_ptr: *mut c_void = fudi.as_mut_ptr().cast();
    let mut show_block = false;
    // Track which arrays were ga_init'd (needed for errret paths)
    let mut garrays_inited = false;

    let eap_arg = unsafe { nvim_eap_get_arg(eap) };
    let skip_orig = unsafe { nvim_eap_get_skip(eap) } != 0;
    let forceit = unsafe { nvim_eap_get_forceit_int(eap) } != 0;

    // ":function" without argument: list functions.
    if unsafe { ends_excmd(c_int::from(*eap_arg as u8)) } != 0 {
        if !skip_orig {
            unsafe { rs_list_functions() };
        }
        let next = unsafe { check_nextcmd(eap_arg) };
        unsafe { nvim_eap_set_nextcmd(eap, next) };
        return;
    }

    // ":function /pat": list functions matching pattern.
    if unsafe { *eap_arg } == b'/' as c_char {
        let p = unsafe { rs_list_functions_matching_pat(eap) };
        let next = unsafe { check_nextcmd(p) };
        unsafe { nvim_eap_set_nextcmd(eap, next) };
        return;
    }

    // Get the function name.
    let mut p = eap_arg;
    let mut name = unsafe {
        save_function_name(
            std::ptr::addr_of_mut!(p),
            c_int::from(skip_orig),
            TFN_NO_AUTOLOAD,
            fudi_ptr,
        )
    };

    let fd_dict = unsafe { nvim_fudi_get_dict(fudi_ptr) };
    let fd_newkey = unsafe { nvim_fudi_get_newkey(fudi_ptr) };
    let fd_di = unsafe { nvim_fudi_get_di(fudi_ptr) };

    let paren = !unsafe { vim_strchr(p, c_int::from(b'(')) }.is_null();

    let mut skip = skip_orig;
    if name.is_null() && (fd_dict.is_null() || !paren) && !skip {
        if !unsafe { aborting() } {
            if !fd_newkey.is_null() {
                unsafe { nvim_semsg_dictkey(fd_newkey) };
            }
            unsafe { xfree(fd_newkey.cast()) };
            return;
        }
        skip = true;
        // Also propagate to eap so get_function_body/rs_list_one_function see it
        unsafe { nvim_eap_set_skip(eap, 1) };
    }

    // An error in a function call during evaluation of an expression in magic
    // braces should not cause the function not to be defined.
    let saved_did_emsg = unsafe { nvim_get_did_emsg() };
    unsafe { nvim_set_did_emsg(false) };

    // Use a closure-style pattern: run the main body, collect an error level,
    // then do the appropriate cleanup.
    // Error levels: 0=success/ret_free, 1=errret_keep, 2=errret_2/erret
    let err = ex_function_body(
        eap,
        eap_arg,
        skip,
        skip_orig,
        forceit,
        paren,
        fudi_ptr,
        fd_dict,
        fd_newkey,
        fd_di,
        std::ptr::addr_of_mut!(p),
        std::ptr::addr_of_mut!(name),
        std::ptr::addr_of_mut!(newargs),
        std::ptr::addr_of_mut!(default_args),
        std::ptr::addr_of_mut!(newlines),
        std::ptr::addr_of_mut!(garrays_inited),
        std::ptr::addr_of_mut!(varargs),
        std::ptr::addr_of_mut!(flags),
        std::ptr::addr_of_mut!(fp),
        std::ptr::addr_of_mut!(free_fp),
        std::ptr::addr_of_mut!(overwrite),
        std::ptr::addr_of_mut!(show_block),
        std::ptr::addr_of_mut!(line_arg),
        std::ptr::addr_of_mut!(line_to_free),
    );

    // Cleanup based on error level
    if err >= 2 {
        // erret: re-init fp garrays if fp was partially set up
        if !fp.is_null() {
            unsafe {
                ga_init(
                    nvim_ufunc_get_args_ga(fp).cast::<GarrayT>(),
                    std::mem::size_of::<*mut c_char>() as c_int,
                    1,
                );
                ga_init(
                    nvim_ufunc_get_def_args_ga(fp).cast::<GarrayT>(),
                    std::mem::size_of::<*mut c_char>() as c_int,
                    1,
                );
                nvim_ufunc_free_name_exp(fp);
            }
        }
        if free_fp {
            unsafe { xfree(fp) };
        }
    }
    if err >= 1 && garrays_inited {
        // errret_keep: clear the garrays
        unsafe {
            ga_clear_strings(std::ptr::addr_of_mut!(newargs).cast());
            ga_clear_strings(std::ptr::addr_of_mut!(default_args).cast());
            ga_clear_strings(std::ptr::addr_of_mut!(newlines).cast());
        }
    }
    // ret_free:
    unsafe {
        xfree(line_to_free.cast());
        xfree(fd_newkey.cast());
        xfree(name.cast());
        nvim_set_did_emsg(nvim_get_did_emsg() | saved_did_emsg);
        if show_block {
            ui_ext_cmdline_block_leave();
        }
    }
}

/// Body of ex_function, factored out for cleaner cleanup.
/// Returns error level: 0 = success (ret_free), 1 = errret_keep, 2 = erret.
#[allow(clippy::too_many_arguments)]
unsafe fn ex_function_body(
    eap: *mut c_void,
    eap_arg: *mut c_char,
    skip: bool,
    _skip_orig: bool,
    forceit: bool,
    paren: bool,
    _fudi_ptr: *mut c_void,
    fd_dict: *mut c_void,
    fd_newkey: *mut c_char,
    fd_di: *mut c_void,
    p_out: *mut *mut c_char,
    name_out: *mut *mut c_char,
    newargs_out: *mut GarrayT,
    default_args_out: *mut GarrayT,
    newlines_out: *mut GarrayT,
    garrays_inited_out: *mut bool,
    varargs_out: *mut c_int,
    flags_out: *mut c_int,
    fp_out: *mut *mut c_void,
    free_fp_out: *mut bool,
    overwrite_out: *mut bool,
    show_block_out: *mut bool,
    line_arg_out: *mut *mut c_char,
    line_to_free_out: *mut *mut c_char,
) -> c_int {
    let mut p = unsafe { *p_out };
    let mut name = unsafe { *name_out };
    let mut varargs: c_int = 0;
    let mut flags: c_int = 0;
    let mut fp: *mut c_void = std::ptr::null_mut();
    let mut free_fp = false;
    let mut overwrite = false;
    let mut show_block = false;
    let mut line_arg: *mut c_char = std::ptr::null_mut();
    let mut line_to_free: *mut c_char = std::ptr::null_mut();

    macro_rules! flush_out {
        () => {
            unsafe {
                *p_out = p;
                *name_out = name;
                *varargs_out = varargs;
                *flags_out = flags;
                *fp_out = fp;
                *free_fp_out = free_fp;
                *overwrite_out = overwrite;
                *show_block_out = show_block;
                *line_arg_out = line_arg;
                *line_to_free_out = line_to_free;
            }
        };
    }

    // ":function func" with only function name: list function.
    if !paren {
        unsafe { rs_list_one_function(eap, name, p) };
        flush_out!();
        return 0;
    }

    // ":function name(arg1, arg2)" Define function.
    p = unsafe { skipwhite(p) };
    if unsafe { *p } != b'(' as c_char {
        if !skip {
            unsafe { nvim_emsg_e124_missing_paren(eap_arg) };
            flush_out!();
            return 0; // goto ret_free
        }
        // attempt to continue by skipping some text
        let paren_pos = unsafe { vim_strchr(p, b'(' as c_int) };
        if !paren_pos.is_null() {
            p = paren_pos;
        }
    }
    p = unsafe { skipwhite(p.add(1)) };

    let item_size = std::mem::size_of::<*mut c_char>() as c_int;
    unsafe {
        ga_init(newargs_out, item_size, 3);
        ga_init(newlines_out, item_size, 3);
        *garrays_inited_out = true;
    }

    if !skip {
        // Check the name of the function. Unless it's a dictionary function
        // (that we are overwriting).
        let arg = if !name.is_null() { name } else { fd_newkey };
        if !arg.is_null()
            && (fd_di.is_null() || !unsafe { nvim_tv_is_func(nvim_dictitem_di_tv(fd_di)) })
        {
            let mut name_base = arg;
            if unsafe { *arg as u8 } == K_SPECIAL {
                let underscore = unsafe { vim_strchr(arg, b'_' as c_int) };
                name_base = if underscore.is_null() {
                    unsafe { arg.add(3) }
                } else {
                    unsafe { underscore.add(1) }
                };
            }
            let mut i: usize = 0;
            loop {
                let c = unsafe { *name_base.add(i) };
                if c == 0 {
                    break;
                }
                let ok = if i == 0 {
                    unsafe { rs_eval_isnamec1(c_int::from(c as u8)) }
                } else {
                    unsafe { rs_eval_isnamec(c_int::from(c as u8)) }
                };
                if !ok {
                    break;
                }
                i += 1;
            }
            if unsafe { *name_base.add(i) } != 0 {
                unsafe { nvim_semsg_e_invarg2(arg) };
                flush_out!();
                return 0; // goto ret_free
            }
        }
        // Disallow using the g: dict.
        if !fd_dict.is_null() && unsafe { nvim_dict_get_scope_impl(fd_dict) } == VAR_DEF_SCOPE {
            unsafe { nvim_emsg_e862_no_g_dict() };
            flush_out!();
            return 0; // goto ret_free
        }
    }

    if unsafe {
        get_function_args(
            std::ptr::addr_of_mut!(p),
            b')' as c_char,
            newargs_out,
            std::ptr::addr_of_mut!(varargs),
            default_args_out,
            skip as c_int,
        )
    } == FAIL
    {
        flush_out!();
        return 2; // goto errret_2
    }

    if unsafe { nvim_keytyped() } != 0 && unsafe { nvim_ui_has_cmdline() } != 0 {
        show_block = true;
        unsafe { ui_ext_cmdline_block_append(0, nvim_eap_get_cmd(eap)) };
    }

    // find extra arguments "range", "dict", "abort" and "closure"
    loop {
        p = unsafe { skipwhite(p) };
        // SAFETY: p points into a NUL-terminated C string; we check byte-by-byte
        let bytes = unsafe { std::slice::from_raw_parts(p as *const u8, 7) };
        if bytes.starts_with(b"range") {
            flags |= FC_RANGE;
            p = unsafe { p.add(5) };
        } else if bytes.starts_with(b"dict") {
            flags |= FC_DICT;
            p = unsafe { p.add(4) };
        } else if bytes.starts_with(b"abort") {
            flags |= FC_ABORT;
            p = unsafe { p.add(5) };
        } else if bytes.starts_with(b"closure") {
            flags |= FC_CLOSURE;
            p = unsafe { p.add(7) };
            if unsafe { get_current_funccal() }.is_null() {
                unsafe { nvim_emsg_e932_closure_toplevel(name) };
                flush_out!();
                return 2; // goto erret
            }
        } else {
            break;
        }
    }

    // When there is a line break use what follows for the function body.
    if unsafe { *p } == b'\n' as c_char {
        line_arg = unsafe { p.add(1) };
    } else if unsafe { *p } != 0
        && unsafe { *p } != b'"' as c_char
        && !skip
        && !unsafe { nvim_get_did_emsg() }
    {
        unsafe { nvim_emsg_trailing_arg(p) };
    }

    // Read the body of the function, until ":endfunction" is found.
    if unsafe { nvim_keytyped() } != 0 {
        if !skip && !forceit {
            if !fd_dict.is_null() && fd_newkey.is_null() {
                unsafe { nvim_emsg_e717_funcdict() };
            } else if !name.is_null() {
                let existing = unsafe { ex_func_find(name) };
                if !existing.is_null() {
                    unsafe { nvim_emsg_e122_func_exists(name) };
                }
            }
        }

        if !skip && unsafe { nvim_get_did_emsg() } {
            flush_out!();
            return 2; // goto erret
        }

        if unsafe { nvim_ui_has_cmdline() } == 0 {
            unsafe { msg_putchar(b'\n' as c_int) };
        }
        unsafe { nvim_set_cmdline_row(nvim_get_msg_row()) };
    }

    // Save the starting line number.
    let sourcing_lnum_top = unsafe { nvim_get_sourcing_lnum() };

    // Do not define the function when getting the body fails and when skipping.
    if unsafe {
        get_function_body(
            eap,
            newlines_out,
            line_arg,
            std::ptr::addr_of_mut!(line_to_free),
            show_block,
        )
    } == FAIL
        || skip
    {
        flush_out!();
        return 2; // goto erret
    }

    // If there are no errors, add the function
    let mut namelen: usize = 0;
    if fd_dict.is_null() {
        let mut ht: *mut c_void = std::ptr::null_mut();
        let v = unsafe { find_var(name, strlen(name), std::ptr::addr_of_mut!(ht), 0) };
        if !v.is_null() {
            let tv = unsafe { nvim_dictitem_di_tv(v) };
            if unsafe { nvim_tv_get_type(tv) } == VAR_FUNC {
                unsafe { nvim_emsg_e707_func_name_conflict(name) };
                flush_out!();
                return 2; // goto erret
            }
        }

        fp = unsafe { ex_func_find(name) };
        if !fp.is_null() {
            let fp_sc_sid = unsafe { nvim_ufunc_get_script_ctx_sid(fp) };
            let fp_sc_seq = unsafe { nvim_ufunc_get_script_ctx_seq(fp) };
            let cur_sc_sid = unsafe { nvim_get_current_sctx_sid() };
            let cur_sc_seq = unsafe { nvim_current_sctx_get_seq() };
            if !forceit && (fp_sc_sid != cur_sc_sid || fp_sc_seq == cur_sc_seq) {
                unsafe { nvim_emsg_e122_func_exists(name) };
                flush_out!();
                return 1; // goto errret_keep
            }
            if unsafe { nvim_ufunc_get_calls(fp) } > 0 {
                unsafe { nvim_emsg_e127_cannot_redefine(name) };
                flush_out!();
                return 1; // goto errret_keep
            }
            if unsafe { nvim_ufunc_get_refcount(fp) } > 1 {
                unsafe { nvim_ufunc_dec_refcount(fp) };
                let cur_flags = unsafe { nvim_ufunc_get_flags(fp) };
                unsafe { nvim_ufunc_set_flags(fp, cur_flags | FC_REMOVED) };
                fp = std::ptr::null_mut();
                overwrite = true;
            } else {
                let exp_name = unsafe { nvim_ufunc_get_name_exp(fp) } as *mut c_char;
                unsafe { xfree(name.cast()) };
                name = std::ptr::null_mut();
                unsafe { nvim_ufunc_set_name_exp(fp, std::ptr::null_mut()) };
                unsafe { rs_func_clear_items(fp) };
                unsafe { nvim_ufunc_set_name_exp(fp, exp_name) };
                unsafe { nvim_ufunc_set_profiling(fp, 0) };
                unsafe { nvim_ufunc_set_prof_initialized(fp, 0) };
            }
        }
    } else {
        // dict function path
        fp = std::ptr::null_mut();
        if fd_newkey.is_null() && !forceit {
            unsafe { nvim_emsg_e717_funcdict() };
            flush_out!();
            return 2; // goto erret
        }
        if fd_di.is_null() {
            let lock = unsafe { nvim_dict_get_lock(fd_dict) };
            if unsafe { value_check_lock(lock, eap_arg, strlen(eap_arg)) } {
                flush_out!();
                return 2; // goto erret
            }
        } else {
            let di_tv = unsafe { nvim_dictitem_di_tv(fd_di) };
            let lock = unsafe { nvim_tv_get_lock(di_tv) };
            if unsafe { value_check_lock(lock, eap_arg, strlen(eap_arg)) } {
                flush_out!();
                return 2; // goto erret
            }
        }

        // Give the function a sequential number.
        unsafe { xfree(name.cast()) };
        name = std::ptr::null_mut();
        let nr = unsafe { nvim_next_func_nr() };
        let s = format!("{nr}\0");
        namelen = s.len() - 1;
        name = unsafe { xmemdupz(s.as_ptr().cast(), namelen) };
    }

    if fp.is_null() {
        if fd_dict.is_null() && !unsafe { vim_strchr(name, AUTOLOAD_CHAR) }.is_null() {
            let mut j = FAIL;
            let sourcing_name = unsafe { nvim_get_sourcing_name() };
            if !sourcing_name.is_null() {
                let namelen_auto = unsafe { strlen(name) };
                let scriptname = unsafe { autoload_name(name, namelen_auto) };
                let pslash = unsafe { vim_strchr(scriptname, b'/' as c_int) };
                if !pslash.is_null() {
                    let plen = unsafe { strlen(pslash) };
                    let slen = unsafe { strlen(sourcing_name) };
                    if slen > plen
                        && unsafe { path_fnamecmp(pslash, sourcing_name.add(slen - plen)) } == 0
                    {
                        j = OK;
                    }
                }
                unsafe { xfree(scriptname.cast()) };
            }
            if j == FAIL {
                unsafe { nvim_emsg_e746_autoload_mismatch(name) };
                flush_out!();
                return 2; // goto erret
            }
        }

        if namelen == 0 {
            namelen = unsafe { strlen(name) };
        }
        fp = unsafe { rs_alloc_ufunc(name, namelen) };

        if !fd_dict.is_null() {
            let di = if fd_di.is_null() {
                let new_di = unsafe { tv_dict_item_alloc(fd_newkey) };
                if unsafe { tv_dict_add(fd_dict, new_di) } == FAIL {
                    unsafe { xfree(new_di) };
                    unsafe { xfree(fp) };
                    fp = std::ptr::null_mut();
                    flush_out!();
                    return 2; // goto erret
                }
                new_di
            } else {
                unsafe { tv_clear(nvim_dictitem_di_tv(fd_di)) };
                fd_di
            };
            unsafe { nvim_dictitem_set_tv_func(di, name, namelen) };
            flags |= FC_DICT;
        }

        if overwrite {
            let fp_name = unsafe { nvim_ufunc_get_uf_name_ptr(fp) };
            unsafe { nvim_func_ht_overwrite_fp(fp_name, fp) };
        } else if unsafe { nvim_func_ht_try_add_fp(fp) } == FAIL {
            free_fp = true;
            flush_out!();
            return 2; // goto erret (with free_fp=true)
        }
        unsafe { nvim_ufunc_set_refcount(fp, 1) };
    }

    // Set the function body fields
    unsafe {
        nvim_ufunc_set_garray_fields(fp, newargs_out, default_args_out, newlines_out);
    }
    if flags & FC_CLOSURE != 0 {
        unsafe { rs_register_closure(fp) };
        // else fp->uf_scoped = NULL, handled by rs_alloc_ufunc zeroing
    }
    if unsafe { prof_def_func() } {
        unsafe { func_do_profile(fp) };
    }
    unsafe { nvim_ufunc_finalize_user_func(fp, varargs, flags, sourcing_lnum_top) };

    flush_out!();
    0 // ret_free
}

/// Internal helper: find_func (Rust, hashtab.rs, exported as "find_func").
/// Declared separately to avoid clashing_extern_declarations issues.
unsafe fn ex_func_find(name: *const c_char) -> *mut c_void {
    extern "C" {
        fn find_func(name: *const c_char) -> *mut c_void;
    }
    unsafe { find_func(name) }
}

// ============================================================================
// ex_call  (Wave 2 Phase 2: migrated from userfunc.c)
// ============================================================================

/// `:1,25call func(arg1, arg2)` function call.
/// `:defer func(arg1, arg2)`    deferred function call.
///
/// Replaces C `ex_call` — body is now here; C file has `extern` decl only.
///
/// # Safety
/// `eap` must be a valid `exarg_T *`.
#[unsafe(export_name = "ex_call")]
pub unsafe extern "C" fn rs_ex_call(eap: *mut c_void) {
    // funcdict_T fudi — 3 pointer fields (dict_T*, char*, dictitem_T*) = 24 bytes, zero-init
    let mut fudi = [0u8; 24usize];
    let mut partial: *mut c_void = std::ptr::null_mut();
    // evalarg_T on the stack, zero-initialized via EvalargT::new_skip()
    let mut evalarg = EvalargT::new_skip();

    let mut arg = unsafe { nvim_eap_get_arg(eap) };
    let skip = unsafe { nvim_eap_get_skip(eap) } != 0;

    unsafe { fill_evalarg_from_eap(std::ptr::addr_of_mut!(evalarg).cast::<c_void>(), eap, skip) };

    if skip {
        // trans_function_name() doesn't work well when skipping; use eval0()
        // to skip to any following command (e.g. `:if 0 | call dict.foo().bar() | endif`).
        // typval_T on stack: v_type (4 bytes int) + 4 pad + vval (8 bytes) + v_lock (4 bytes) + 4 pad = 24 bytes
        let mut rettv = [0u8; 24usize];
        unsafe { nvim_syn_emsg_skip_inc() };
        let eap_arg = unsafe { nvim_eap_get_arg(eap) };
        if unsafe {
            eval0(
                eap_arg,
                rettv.as_mut_ptr().cast::<c_void>(),
                eap,
                std::ptr::addr_of_mut!(evalarg).cast::<c_void>(),
            )
        } != FAIL
        {
            unsafe { tv_clear(rettv.as_mut_ptr().cast::<c_void>()) };
        }
        unsafe { nvim_syn_emsg_skip_dec() };
        unsafe { clear_evalarg(std::ptr::addr_of_mut!(evalarg).cast::<c_void>(), eap) };
        return;
    }

    let tofree = unsafe {
        trans_function_name(
            std::ptr::addr_of_mut!(arg),
            0, // skip = false
            TFN_INT,
            fudi.as_mut_ptr().cast::<c_void>(),
            std::ptr::addr_of_mut!(partial),
        )
    };

    let fudi_ptr: *mut c_void = fudi.as_mut_ptr().cast();
    let newkey = unsafe { nvim_fudi_get_newkey(fudi_ptr) };
    if !newkey.is_null() {
        // Still need to give an error message for missing key.
        unsafe { nvim_semsg_dictkey(newkey) };
        unsafe { xfree(newkey.cast()) };
    }

    let fd_dict = unsafe { nvim_fudi_get_dict(fudi_ptr) };

    if tofree.is_null() {
        unsafe { clear_evalarg(std::ptr::addr_of_mut!(evalarg).cast::<c_void>(), eap) };
        unsafe { tv_dict_unref(fd_dict) };
        return;
    }

    // Increase refcount on dictionary; it could get deleted when evaluating args.
    if !fd_dict.is_null() {
        unsafe { nvim_tv_dict_incr_refcount(fd_dict) };
    }

    // If it is a VAR_FUNC or VAR_PARTIAL variable, use its contents.
    // For VAR_PARTIAL get its partial, unless we already have one from trans_function_name().
    #[allow(clippy::cast_possible_truncation)]
    let mut len = unsafe { strlen(tofree) } as c_int;
    let mut found_var = false;
    let name = unsafe {
        deref_func_name(
            tofree,
            std::ptr::addr_of_mut!(len),
            if partial.is_null() {
                std::ptr::addr_of_mut!(partial)
            } else {
                std::ptr::null_mut()
            },
            0, // no_autoload = false
            std::ptr::addr_of_mut!(found_var),
        )
    };

    // Skip white space to allow ":call func ()".
    let startarg = unsafe { skipwhite(arg) };

    if unsafe { *startarg } != b'(' as c_char {
        let eap_arg = unsafe { nvim_eap_get_arg(eap) };
        unsafe { nvim_semsg_e_missingparen(eap_arg) };
        // goto end
        unsafe { tv_dict_unref(fd_dict) };
        unsafe { xfree(tofree.cast()) };
        unsafe { clear_evalarg(std::ptr::addr_of_mut!(evalarg).cast::<c_void>(), eap) };
        return;
    }

    let cmd_defer_idx = unsafe { nvim_cmd_defer_idx() };
    let cmdidx = unsafe { nvim_eap_get_cmdidx(eap) };
    let failed: c_int;

    if cmdidx == cmd_defer_idx {
        let mut arg_mut = startarg;
        // FAIL = 0, OK = 1; c_int::from(result == FAIL) = 1 if failed
        let result = unsafe {
            rs_ex_defer_inner(
                name,
                std::ptr::addr_of_mut!(arg_mut),
                partial,
                std::ptr::addr_of_mut!(evalarg).cast::<c_void>(),
            )
        };
        failed = c_int::from(result == FAIL);
        arg = arg_mut;
    } else {
        let mut funcexe = FuncExeT::new();
        funcexe.fe_partial = partial;
        funcexe.fe_selfdict = fd_dict;
        funcexe.fe_firstline = unsafe { nvim_eap_get_line1(eap) };
        funcexe.fe_lastline = unsafe { nvim_eap_get_line2(eap) };
        funcexe.fe_found_var = found_var;
        funcexe.fe_evaluate = true;

        let mut arg_mut = startarg;
        failed = unsafe {
            ex_call_inner(
                eap,
                name,
                std::ptr::addr_of_mut!(arg_mut),
                startarg,
                std::ptr::addr_of!(funcexe).cast::<c_void>(),
                std::ptr::addr_of_mut!(evalarg).cast::<c_void>(),
            )
        };
        arg = arg_mut;
    }

    // When inside :try we need to check for following "| catch" or "| endtry".
    // Not when there was an error, but do check if an exception was thrown.
    let cstack = unsafe { nvim_eap_get_cstack(eap) };
    let trylevel = unsafe { nvim_cstack_get_trylevel(cstack) };
    if (!unsafe { aborting() } || unsafe { did_throw }) && (failed == 0 || trylevel > 0) {
        // Check for trailing illegal characters and a following command.
        if unsafe { ends_excmd(c_int::from(*arg as u8)) } == 0 {
            if failed == 0 && !unsafe { aborting() } {
                unsafe { emsg_severe = true };
                unsafe { nvim_semsg_with_name(nvim_e_trailing_arg(), arg) };
            }
        } else {
            let next = unsafe { check_nextcmd(arg) };
            unsafe { nvim_eap_set_nextcmd(eap, next) };
        }
    }
    unsafe { clear_evalarg(std::ptr::addr_of_mut!(evalarg).cast::<c_void>(), eap) };

    // end:
    unsafe { tv_dict_unref(fd_dict) };
    unsafe { xfree(tofree.cast()) };
}
