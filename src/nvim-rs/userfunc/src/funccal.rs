//! Funccal management, ex_delfunction, and helper functions for VimL.
//!
//! Migrated from `src/nvim/eval/userfunc.c` Phase 7.
//! Phase 13: Several impl shims inlined directly.
//! Phase 15: callback_call_retnr migrated.

#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, c_void};

// SctxT layout: sc_sid(i32) + sc_seq(i32) + sc_lnum(i32) + _pad(i32) + sc_chan(u64) = 24 bytes.
// Must match C's sctx_T exactly.
#[repr(C)]
#[derive(Copy, Clone)]
struct SctxT {
    sc_sid: c_int,
    sc_seq: c_int,
    sc_lnum: i32,
    _pad: i32,
    sc_chan: u64,
}

#[allow(clashing_extern_declarations)]
extern "C" {
    // Phase 27: for rs_funccal_unref inlining
    fn nvim_get_previous_funccal() -> *mut c_void;
    fn nvim_set_previous_funccal(fc: *mut c_void);
    fn nvim_fc_decrement_refcount(fc: *mut c_void) -> c_int;
    fn nvim_fc_ufuncs_null_matching(fc: *mut c_void, fp: *mut c_void);
    // Phase 14: For inlining nvim_user_func_error_impl:
    fn nvim_semsg_not_callable(name: *const c_char);

    // current_funccal access (for inlining remove_funccal and create_funccal)
    fn get_current_funccal() -> *mut c_void;
    fn set_current_funccal(fc: *mut c_void);
    fn nvim_set_current_funccal(fc: *mut c_void);
    fn nvim_fc_get_caller(fc: *mut c_void) -> *mut c_void;
    fn nvim_fc_get_func(fc: *mut c_void) -> *mut c_void;

    // Phase 13: For inlining nvim_free_funccal_impl:
    fn nvim_fc_ufuncs_len(fc: *const c_void) -> c_int;
    fn nvim_fc_ufuncs_item(fc: *const c_void, i: c_int) -> *mut c_void;
    fn nvim_fc_ufuncs_ga_clear(fc: *mut c_void);
    fn nvim_ufunc_get_scoped(fp: *mut c_void) -> *mut c_void;
    fn nvim_ufunc_set_scoped(fp: *mut c_void, fc: *mut c_void);
    fn func_ptr_unref(fp: *mut c_void);
    fn xfree(ptr: *mut c_void);

    // Phase 13: For inlining nvim_emsg_funcname_impl:
    fn nvim_emsg_funcname_mk_snr(name: *const c_char) -> *mut c_char;
    fn nvim_semsg_with_name(errmsg: *const c_char, name: *const c_char);

    // Phase 13: For inlining nvim_save_funccal_impl and nvim_restore_funccal_impl:
    fn nvim_funccal_stack_head_mut() -> *mut c_void;
    fn nvim_set_funccal_stack(entry: *mut c_void);
    fn nvim_fc_entry_set_top(fce: *mut c_void, fc: *mut c_void);
    fn nvim_fc_entry_set_next(fce: *mut c_void, next: *mut c_void);
    fn nvim_funccal_entry_top(fce: *mut c_void) -> *mut c_void;
    fn nvim_funccal_entry_next(fce: *mut c_void) -> *mut c_void;
    fn nvim_iemsg(msg: *const c_char);

    // Phase 13: For inlining nvim_create_funccal_impl:
    fn xcalloc(count: usize, size: usize) -> *mut c_void;
    fn nvim_sizeof_funccall() -> usize;
    fn func_ptr_ref(fp: *mut c_void);
    fn nvim_fc_set_func(fc: *mut c_void, fp: *mut c_void);
    fn nvim_fc_set_rettv(fc: *mut c_void, rettv: *mut c_void);
    fn nvim_fc_set_caller(fc: *mut c_void, caller: *mut c_void);

    // Phase 15: For callback_call_retnr
    fn callback_call(
        callback: *mut c_void,
        argcount: c_int,
        argvars: *mut c_void,
        rettv: *mut c_void,
    ) -> bool;
    fn tv_get_number_chk(tv: *const c_void, denote: *mut c_int) -> i64;
    fn tv_clear(tv: *mut c_void);

    // Phase 16: For call_simple_luafunc and call_simple_func
    fn nvim_tv_set_number(tv: *mut c_void, n: i64);
    fn nlua_typval_call(
        funcname: *const c_char,
        len: usize,
        argvars: *mut c_void,
        argcount: c_int,
        rettv: *mut c_void,
    );
    fn find_func(name: *const c_char) -> *mut c_void;
    fn nvim_ufunc_get_flags(fp: *mut c_void) -> c_int;
    fn nvim_call_user_func_check_simple(
        fp: *mut c_void,
        argvars: *mut c_void,
        rettv: *mut c_void,
    ) -> c_int;
    fn rs_fname_trans_sid(
        name: *const c_char,
        fname_buf: *mut c_char,
        tofree: *mut *mut c_char,
        error: *mut c_int,
    ) -> *mut c_char;
    fn xstrnsave(s: *const c_char, len: usize) -> *mut c_char;

    // Phase 21: For call_user_func_check migration
    fn nvim_ufunc_get_luaref(fp: *mut c_void) -> c_int;
    fn typval_exec_lua_callable(
        lua_cb: c_int,
        argcount: c_int,
        argvars: *mut c_void,
        rettv: *mut c_void,
    ) -> c_int;
    fn call_user_func(
        fp: *mut c_void,
        argcount: c_int,
        argvars: *mut c_void,
        rettv: *mut c_void,
        firstline: i32,
        lastline: i32,
        selfdict: *mut c_void,
    );
    fn check_user_func_argcount(fp: *mut c_void, argcount: c_int) -> c_int;
    fn nvim_funcexe_get_doesrange(fe: *mut c_void) -> *mut bool;
    fn nvim_funcexe_get_firstline(fe: *mut c_void) -> i32;
    fn nvim_funcexe_get_lastline(fe: *mut c_void) -> i32;

    // Phase 22: For call_func migration
    fn nvim_funcexe_get_selfdict(fe: *const c_void) -> *mut c_void;
    fn nvim_funcexe_get_partial(fe: *const c_void) -> *mut c_void;
    fn nvim_funcexe_get_evaluate(fe: *const c_void) -> bool;
    fn nvim_funcexe_get_basetv(fe: *const c_void) -> *mut c_void;
    fn nvim_funcexe_get_found_var(fe: *const c_void) -> bool;
    fn nvim_funcexe_call_argv_func(
        fe: *mut c_void,
        argcount: c_int,
        argvars: *mut c_void,
        argv_clear: c_int,
        fp: *mut c_void,
    ) -> c_int;
    fn nvim_partial_get_auto(pt: *const c_void) -> bool;
    fn nvim_partial_get_func(pt: *const c_void) -> *mut c_void;
    fn nvim_partial_get_argc(pt: *const c_void) -> c_int;
    fn nvim_partial_get_dict(pt: *const c_void) -> *mut c_void;
    fn nvim_partial_get_argv(pt: *const c_void) -> *mut c_void;

    fn apply_autocmds_for_funcundefined(name: *const c_char) -> c_int;
    fn script_autoload(name: *const c_char, name_len: usize, reload: bool) -> bool;
    fn aborting() -> bool;
    fn update_force_abort();
    fn call_internal_func(
        fname: *const c_char,
        argcount: c_int,
        argvars: *mut c_void,
        rettv: *mut c_void,
    ) -> c_int;
    fn call_internal_method(
        fname: *const c_char,
        argcount: c_int,
        argvars: *mut c_void,
        rettv: *mut c_void,
        basetv: *mut c_void,
    ) -> c_int;
    fn xmemdupz(src: *const c_void, len: usize) -> *mut c_void;
    fn tv_copy(from: *const c_void, to: *mut c_void);
    fn nvim_tv_set_unknown(tv: *mut c_void);
    fn rs_is_luafunc(partial: *mut c_void) -> bool;
    fn rs_builtin_function(name: *const c_char, len: c_int) -> c_int;
    // argv_add_base is Rust (lookup.rs), exported as "argv_add_base"
    fn argv_add_base(
        basetv: *const c_void,
        argvars: *mut *mut c_void,
        argcount: *mut c_int,
        new_argvars: *mut c_void,
        argv_base: *mut c_int,
    );
    // call_user_func_check is Rust (funccal.rs), exported as "call_user_func_check"
    // Already declared above as "call_user_func_check" -- using that one.

    // Phase 23: For get_func_tv migration
    fn nvim_evalarg_should_evaluate(ea: *const c_void) -> bool;
    fn nvim_funcargs_ga_itemsize() -> c_int;
    fn nvim_funcargs_ga_init();
    fn nvim_funcargs_ga_grow();
    fn nvim_funcargs_push_tv_ptr(tv: *mut c_void);
    fn nvim_funcargs_dec_len(n: c_int);
    fn nvim_get_testing_flag() -> c_int;
    fn nvim_emsg_e740_too_many_args(name: *const c_char);
    fn nvim_emsg_e116_invalid_args(name: *const c_char);
    // get_func_arguments is Rust (parsing.rs), exported as "get_func_arguments"
    fn get_func_arguments(
        arg: *mut *mut c_char,
        evalarg: *mut c_void,
        partial_argc: c_int,
        argvars: *mut c_void,
        argcount: *mut c_int,
    ) -> c_int;
    fn skipwhite(p: *const c_char) -> *mut c_char;

    // Phase 25: for ex_call_inner migration
    fn nvim_eap_get_line1(eap: *const c_void) -> i32;
    fn nvim_eap_get_line2(eap: *const c_void) -> i32;
    fn nvim_eap_get_addr_count(eap: *const c_void) -> c_int;
    fn nvim_ex_call_check_advance_cursor(lnum: i32) -> c_int;
    fn nvim_handle_subscript_eval_evaluate(arg: *mut *mut c_char, rettv: *mut c_void) -> c_int;
    // nvim_emsg_invrange deleted (Phase 9): use emsg(gettext(e_invrange)) directly
    fn emsg(s: *const c_char);
    fn gettext(msgid: *const c_char) -> *const c_char;
    #[link_name = "e_invrange"]
    static e_invrange_uf: [u8; 0];
    // get_func_tv is Rust (funccal.rs), linked by name -- already declared above

    // Phase 28: for nvim_ex_return_impl migration
    fn nvim_eap_get_arg(eap: *const c_void) -> *mut c_char;
    fn nvim_eap_get_skip(eap: *const c_void) -> c_int;
    fn nvim_eap_set_nextcmd(eap: *mut c_void, val: *mut c_char);
    fn nvim_eap_get_nextcmd(eap: *const c_void) -> *mut c_char;
    fn nvim_emsg_return_not_in_func();
    fn nvim_syn_emsg_skip_inc();
    fn nvim_syn_emsg_skip_dec();
    fn eval0(arg: *mut c_char, rettv: *mut c_void, eap: *mut c_void, evalarg: *mut c_void)
        -> c_int;
    fn do_return(eap: *mut c_void, reanimate: c_int, is_cmd: c_int, rettv: *mut c_void) -> c_int;
    fn check_nextcmd(p: *const c_char) -> *mut c_char;
    fn clear_evalarg(evalarg: *mut c_void, eap: *mut c_void);

    // Phase 32: for nvim_ex_delfunction_impl migration
    fn nvim_eap_get_forceit_int(eap: *const c_void) -> c_int;
    fn nvim_ufunc_get_refcount(fp: *const c_void) -> c_int;
    fn nvim_ufunc_or_flags_deleted(fp: *mut c_void);
    fn nvim_ufunc_get_calls(fp: *mut c_void) -> c_int;
    fn nvim_ufunc_decrement_refcount(fp: *mut c_void) -> c_int;
    fn nvim_fudi_get_dict(fudi: *const c_void) -> *mut c_void;
    fn nvim_fudi_get_newkey(fudi: *const c_void) -> *mut c_char;
    fn nvim_fudi_get_di(fudi: *const c_void) -> *mut c_void;
    fn nvim_tv_dict_item_remove(dict: *mut c_void, di: *mut c_void);
    fn nvim_emsg_funcref();
    fn nvim_ends_excmd_skipwhite(p: *const c_char) -> c_int;
    fn nvim_semsg_e_invarg2(arg: *const c_char);
    fn nvim_semsg_nofunc(arg: *const c_char);
    fn nvim_semsg_e131_in_use(arg: *const c_char);
    fn nvim_semsg_cannot_delete_internal(arg: *const c_char);
    fn nvim_ufunc_get_name(fp: *mut c_void) -> *const c_char;
    fn trans_function_name(
        pp: *mut *mut c_char,
        skip: c_int,
        flags: c_int,
        fudi: *mut c_void,
        partial: *mut c_void,
    ) -> *mut c_char;
    fn nvim_emsg_trailing_arg(p: *const c_char);

    // Phase 33: for get_func_line migration
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn nvim_fc_get_returned(fc: *mut c_void) -> c_int;
    fn nvim_fc_get_linenr(fc: *mut c_void) -> c_int;
    fn nvim_fc_set_linenr(fc: *mut c_void, v: c_int);
    fn nvim_fc_postincrement_linenr(fc: *mut c_void) -> c_int;
    fn nvim_fc_get_breakpoint_ptr(fc: *mut c_void) -> *mut i32;
    fn nvim_fc_get_dbg_tick_ptr(fc: *mut c_void) -> *mut c_int;
    fn nvim_ufunc_get_lines_len(fp: *mut c_void) -> c_int;
    fn nvim_ufunc_funcline_is_null(fp: *mut c_void, idx: c_int) -> c_int;
    fn nvim_ufunc_get_funcline(fp: *mut c_void, i: c_int) -> *const c_char;
    fn nvim_get_sourcing_lnum_direct() -> i32;
    fn nvim_rt_set_sourcing_lnum(lnum: c_int);
    fn func_line_start(cookie: *mut c_void);
    fn func_line_end(cookie: *mut c_void);
    fn dbg_find_breakpoint(file: bool, fname: *const c_char, after: i32) -> i32;
    fn dbg_breakpoint(name: *const c_char, lnum: i32);
    static did_emsg: c_int;
    static debug_tick: c_int;
    static do_profiling: c_int;

    // Phase 35: for func_call migration
    fn nvim_func_call_iter_args(args: *mut c_void, argv: *mut c_void, max_args: c_int) -> c_int;
    fn nvim_curwin_cursor_lnum() -> i32;

    // Phase 31: for nvim_free_funccal_contents_impl and nvim_cleanup_function_call_impl migration
    fn nvim_fc_l_vars_ht_clear(fc: *mut c_void);
    fn nvim_fc_l_avars_ht_clear(fc: *mut c_void);
    fn nvim_fc_l_varlist_tv_clear_all(fc: *mut c_void);
    fn nvim_fc_pop_current_funccal(fc: *mut c_void);
    fn nvim_fc_l_avars_ht_clear_ext_false(fc: *mut c_void);
    fn nvim_fc_l_avars_tv_copy_all(fc: *mut c_void);
    fn nvim_fc_l_varlist_set_lv_first_null(fc: *mut c_void);
    fn nvim_fc_l_varlist_tv_copy_all(fc: *mut c_void);
    fn nvim_cleanup_function_call_put_in_prev_list(fc: *mut c_void);
    fn nvim_fc_l_vars_dv_refcount(fc: *const c_void) -> c_int;
    fn nvim_fc_l_avars_dv_refcount(fc: *const c_void) -> c_int;
    fn nvim_fc_varlist_lv_refcount(fc: *const c_void) -> c_int;
    fn nvim_get_fc_refcount(fc: *const c_void) -> c_int;

    // === Phase call_user_func (plan 5e037151) ===
    fn nvim_fc_fixvar_item(fc: *mut c_void, idx: c_int) -> *mut c_void; // dictitem_T*
    fn nvim_fc_listitem_tv(fc: *mut c_void, ai: c_int) -> *mut c_void; // typval_T*
    fn nvim_fc_set_level(fc: *mut c_void, v: c_int);
    fn nvim_get_current_sctx() -> SctxT;
    fn nvim_set_current_sctx(s: SctxT);
    fn nvim_redrawing_disabled_incr();
    fn nvim_redrawing_disabled_decr();
    fn nvim_no_wait_return_incr();
    fn nvim_no_wait_return_decr();
    fn nvim_sandbox_incr();
    fn nvim_sandbox_decr();
    fn nvim_ex_nesting_level_incr();
    fn nvim_ex_nesting_level_decr();
    fn nvim_get_trylevel() -> c_int;
    fn nvim_ufunc_incr_calls(fp: *mut c_void);
    fn nvim_sizeof_save_redo() -> usize;
    // save/restore search patterns and redo buffer
    fn save_search_patterns();
    fn restore_search_patterns();
    fn saveRedobuff(save_redo: *mut c_void);
    fn restoreRedobuff(save_redo: *mut c_void);
    fn rs_ins_compl_active() -> c_int;
    fn line_breakcheck();
    // funccal setup helpers
    fn nvim_fc_l_vars_dict(fc: *mut c_void) -> *mut c_void; // dict_T*
    fn nvim_fc_l_vars_var_ptr(fc: *mut c_void) -> *mut c_void; // ScopeDictDictItem*
    fn nvim_fc_l_avars_dict(fc: *mut c_void) -> *mut c_void; // dict_T*
    fn nvim_fc_l_avars_var_ptr(fc: *mut c_void) -> *mut c_void; // ScopeDictDictItem*
    fn nvim_fc_l_varlist(fc: *mut c_void) -> *mut c_void; // list_T*
    fn nvim_fc_l_vars_ht(fc: *mut c_void) -> *mut c_void; // hashtab_T*
    fn nvim_fc_l_avars_ht(fc: *mut c_void) -> *mut c_void; // hashtab_T*
    fn init_var_dict(dict: *mut c_void, dict_var: *mut c_void, scope: c_int);
    fn hash_add(ht: *mut c_void, key: *const c_char) -> c_int;
    fn tv_dict_item_alloc_len(key: *const c_char, len: usize) -> *mut c_void; // dictitem_T*
    fn tv_list_init_static(list: *mut c_void);
    fn nvim_tv_list_set_lock(list: *mut c_void, lock: c_int);
    // estack
    fn estack_push_ufunc(fp: *mut c_void, lnum: i32);
    fn estack_pop();
    // verbose
    fn verbose_enter_scroll();
    fn verbose_leave_scroll();
    fn smsg(hifmt: c_int, fmt: *const c_char, ...) -> c_int;
    fn msg_puts(s: *const c_char);
    fn msg_outnum(n: c_int);
    fn encode_tv2string(tv: *const c_void, len: *mut usize) -> *mut c_char;
    fn vim_strsize(s: *const c_char) -> c_int;
    fn trunc_string(s: *const c_char, buf: *mut c_char, maxlen: c_int, buflen: usize);
    fn nvim_tv_get_type(tv: *const c_void) -> c_int;
    fn nvim_tv_get_number(tv: *const c_void) -> i64;
    // profiling
    fn func_do_profile(fp: *mut c_void);
    fn has_profiling(file: bool, fname: *const c_char, prop: *mut c_void) -> bool;
    fn profile_start() -> u64;
    fn profile_end(tm: u64) -> u64;
    fn profile_sub_wait(wait_start: u64, tm: u64) -> u64;
    fn profile_add(tm1: u64, tm2: u64) -> u64;
    fn profile_self(self_tm: u64, tm: u64, children: u64) -> u64;
    fn profile_zero() -> u64;
    fn script_prof_save(wait_start: *mut u64);
    fn script_prof_restore(wait_start: *const u64);
    fn nvim_ufunc_get_profiling(fp: *const c_void) -> c_int;
    fn nvim_ufunc_set_profiling(fp: *mut c_void, val: c_int);
    fn nvim_ufunc_get_tm_count(fp: *const c_void) -> c_int;
    fn nvim_ufunc_set_tm_count(fp: *mut c_void, val: c_int);
    fn nvim_ufunc_get_tm_total(fp: *const c_void) -> u64;
    fn nvim_ufunc_set_tm_total(fp: *mut c_void, val: u64);
    fn nvim_ufunc_get_tm_self(fp: *const c_void) -> u64;
    fn nvim_ufunc_set_tm_self(fp: *mut c_void, val: u64);
    fn nvim_ufunc_get_tm_children(fp: *const c_void) -> u64;
    fn nvim_ufunc_set_tm_children(fp: *mut c_void, val: u64);
    fn nvim_ufunc_get_tml_children(fp: *const c_void) -> u64;
    fn nvim_ufunc_set_tml_children(fp: *mut c_void, val: u64);
    fn nvim_get_did_emsg() -> bool;
    fn nvim_set_did_emsg(val: bool);
    fn nvim_aborting() -> bool;
    fn nvim_sourcing_name_get() -> *const c_char;
    fn nvim_ufunc_get_args_len(fp: *mut c_void) -> c_int;
    fn nvim_ufunc_get_arg(fp: *mut c_void, i: c_int) -> *const c_char;
    fn nvim_ufunc_get_def_args_len(fp: *mut c_void) -> c_int;
    fn nvim_ufunc_get_def_arg(fp: *mut c_void, i: c_int) -> *const c_char;
    fn nvim_ufunc_get_name_ptr(fp: *mut c_void) -> *mut c_char;
    fn nvim_ufunc_get_script_ctx(fp: *mut c_void) -> SctxT;
    // nvim_ufunc_get_refcount / get_calls already declared in Phase 32 block above
    // dictitem field accessors
    fn nvim_dictitem_set_flags(di: *mut c_void, flags: c_int);
    fn nvim_dictitem_strcpy_key(di: *mut c_void, name: *const c_char);
    fn nvim_dictitem_key_ptr(di: *const c_void) -> *const c_char;
    fn nvim_dictitem_copy_tv(di: *mut c_void, src: *const c_void);
    fn nvim_dictitem_tv_set_lock(di: *mut c_void, lock: c_int);
    fn nvim_dictitem_tv_set_type(di: *mut c_void, vtype: c_int);
    fn nvim_dictitem_tv_set_vval_dict(di: *mut c_void, dict: *mut c_void);
    fn nvim_dictitem_tv_set_vval_list(di: *mut c_void, list: *mut c_void);
    fn nvim_dictitem_tv_ptr(di: *mut c_void) -> *mut c_void; // typval_T*
    fn nvim_dictitem_or_flags_ro_fix(di: *mut c_void);
    fn nvim_dict_incr_refcount(dict: *mut c_void);
    fn nvim_ufunc_lambda_body_ptr(fp: *mut c_void) -> *mut c_char;
    fn nvim_format_vararg_name(buf: *mut c_char, bufsz: usize, ai_plus_1: c_int) -> c_int;
    fn nvim_get_numbuflen() -> c_int;
    fn nvim_var_def_scope() -> c_int;
    fn nvim_var_scope() -> c_int;
    fn nvim_var_fixed() -> c_int;
    fn nvim_var_unlocked() -> c_int;
    fn nvim_var_number() -> c_int;
    fn nvim_var_dict() -> c_int;
    fn nvim_var_list() -> c_int;
    fn nvim_fixvar_cnt() -> c_int;
    fn nvim_var_short_len() -> c_int;
    fn nvim_di_flags_ro_fix() -> c_int;
    fn rs_func_clear_free(fp: *mut c_void, force: c_int);
    fn rs_handle_defer_one(funccal: *mut c_void);
    fn rs_add_nr_var(dp: *mut c_void, v: *mut c_void, name: *const c_char, nr: i64);
    // get_func_line is already exported as extern fn; pass the symbol pointer to do_cmdline
    fn do_cmdline(
        cmdline: *mut c_char,
        fgetline: Option<unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char>,
        cookie: *mut c_void,
        flags: c_int,
    ) -> c_int;
    static mut p_mfd: i64;
    fn nvim_emsg_e132();
    fn eval1(arg: *mut *mut c_char, rettv: *mut c_void, evalarg: *mut c_void) -> c_int;
    fn nvim_get_ex_nesting_level() -> c_int;
    fn nvim_fc_ufuncs_ga_init_wrapper(fc: *mut c_void);
    fn nvim_fc_l_avars_set_lock(fc: *mut c_void, lock: c_int);
    fn nvim_fc_listitem_append(fc: *mut c_void, ai: c_int, list: *mut c_void);
    fn nvim_emsg_off_inc();
    fn nvim_emsg_off_dec();
    fn nvim_get_p_verbose() -> c_int;
    fn nvim_fc_get_rettv(fc: *mut c_void) -> *mut c_void; // typval_T*
    fn nvim_ufunc_set_calls(fp: *mut c_void, v: c_int);
    static mut EVALARG_EVALUATE: c_void;
}

// =============================================================================
// Constants
// =============================================================================

const OK: c_int = 1;
const FAIL: c_int = 0;
const NOTDONE: c_int = 2;
const FCERR_NONE: c_int = 5;
const FCERR_UNKNOWN_OK: c_int = 0; // FCERR_UNKNOWN: "no error" return from check_user_func_argcount
const FC_DELETED_FLAG: c_int = 0x10;
const FC_LUAREF: c_int = 0x800;
const FC_RANGE: c_int = 0x02;
const FC_DICT: c_int = 0x04;
const FLEN_FIXED: usize = 40;
const MAX_FUNC_ARGS: usize = 20;
const SIZEOF_TYPVAL: usize = 16;

// =============================================================================
// free_funccal
// =============================================================================
//
// Phase 13: inlined from nvim_free_funccal_impl.

#[no_mangle]
pub unsafe extern "C" fn rs_free_funccal(fc: *mut c_void) {
    let len = unsafe { nvim_fc_ufuncs_len(fc) };
    for i in 0..len {
        let fp = unsafe { nvim_fc_ufuncs_item(fc, i) };
        if !fp.is_null() && unsafe { nvim_ufunc_get_scoped(fp) } == fc {
            unsafe { nvim_ufunc_set_scoped(fp, std::ptr::null_mut()) };
        }
    }
    unsafe { nvim_fc_ufuncs_ga_clear(fc) };
    let func = unsafe { nvim_fc_get_func(fc) };
    unsafe { func_ptr_unref(func) };
    unsafe { xfree(fc) };
}

// =============================================================================
// free_funccal_contents
// =============================================================================
//
// Phase 31: inlined from nvim_free_funccal_contents_impl.

#[no_mangle]
pub unsafe extern "C" fn rs_free_funccal_contents(fc: *mut c_void) {
    // Free all l: variables.
    unsafe { nvim_fc_l_vars_ht_clear(fc) };
    // Free all a: variables.
    unsafe { nvim_fc_l_avars_ht_clear(fc) };
    // Free the a:000 variables.
    unsafe { nvim_fc_l_varlist_tv_clear_all(fc) };
    unsafe { rs_free_funccal(fc) };
}

// =============================================================================
// cleanup_function_call
// =============================================================================
//
// Phase 31: inlined from nvim_cleanup_function_call_impl.

// DO_NOT_FREE_CNT must match C's DO_NOT_FREE_CNT (INT_MAX / 2).
const DO_NOT_FREE_CNT: c_int = c_int::MAX / 2;

#[no_mangle]
pub unsafe extern "C" fn rs_cleanup_function_call(fc: *mut c_void) {
    let may_free_fc = unsafe { nvim_get_fc_refcount(fc) } <= 0;
    let mut free_fc = true;

    // current_funccal = fc->fc_caller
    unsafe { nvim_fc_pop_current_funccal(fc) };

    // Free all l: variables if not referred.
    if may_free_fc && unsafe { nvim_fc_l_vars_dv_refcount(fc) } == DO_NOT_FREE_CNT {
        unsafe { nvim_fc_l_vars_ht_clear(fc) };
    } else {
        free_fc = false;
    }

    // If the a:000 list and the l: and a: dicts are not referenced and
    // there is no closure using it, we can free the funccall_T and what's in it.
    if may_free_fc && unsafe { nvim_fc_l_avars_dv_refcount(fc) } == DO_NOT_FREE_CNT {
        unsafe { nvim_fc_l_avars_ht_clear_ext_false(fc) };
    } else {
        free_fc = false;
        // Make a copy of the a: variables, since we didn't do that above.
        unsafe { nvim_fc_l_avars_tv_copy_all(fc) };
    }

    if may_free_fc && unsafe { nvim_fc_varlist_lv_refcount(fc) } == DO_NOT_FREE_CNT {
        unsafe { nvim_fc_l_varlist_set_lv_first_null(fc) };
    } else {
        free_fc = false;
        // Make a copy of the a:000 items, since we didn't do that above.
        unsafe { nvim_fc_l_varlist_tv_copy_all(fc) };
    }

    if free_fc {
        unsafe { rs_free_funccal(fc) };
    } else {
        // "fc" is still in use. Link into previous_funccal list for GC.
        unsafe { nvim_cleanup_function_call_put_in_prev_list(fc) };
    }
}

// =============================================================================
// funccal_unref
// =============================================================================
//
// Phase 27: inlined from nvim_funccal_unref_impl.

#[no_mangle]
pub unsafe extern "C" fn rs_funccal_unref(fc: *mut c_void, fp: *mut c_void, force: c_int) {
    if fc.is_null() {
        return;
    }

    let refcount = unsafe { nvim_fc_decrement_refcount(fc) };
    let should_free = if force != 0 {
        refcount <= 0
    } else {
        let referenced = unsafe { crate::gc::rs_fc_referenced(fc.cast_const()) };
        referenced == 0
    };

    if should_free {
        // Search previous_funccal list and remove fc if found.
        let mut prev: *mut c_void = std::ptr::null_mut();
        let mut cur = unsafe { nvim_get_previous_funccal() };
        while !cur.is_null() {
            let next = unsafe { nvim_fc_get_caller(cur) };
            if cur == fc {
                // Remove from linked list.
                if prev.is_null() {
                    unsafe { nvim_set_previous_funccal(next) };
                } else {
                    unsafe { nvim_fc_set_caller(prev, next) };
                }
                unsafe { rs_free_funccal_contents(fc) };
                return;
            }
            prev = cur;
            cur = next;
        }
    }

    // Not freed: null out matching ufuncs entries.
    unsafe { nvim_fc_ufuncs_null_matching(fc, fp) };
}

// =============================================================================
// create_funccal
// =============================================================================
//
// Phase 13: inlined from nvim_create_funccal_impl.

#[unsafe(export_name = "create_funccal")]
pub unsafe extern "C" fn rs_create_funccal(fp: *mut c_void, rettv: *mut c_void) -> *mut c_void {
    let size = unsafe { nvim_sizeof_funccall() };
    let fc = unsafe { xcalloc(1, size) };
    let caller = unsafe { get_current_funccal() };
    unsafe { nvim_fc_set_caller(fc, caller) };
    unsafe { nvim_set_current_funccal(fc) };
    unsafe { nvim_fc_set_func(fc, fp) };
    unsafe { func_ptr_ref(fp) };
    unsafe { nvim_fc_set_rettv(fc, rettv) };
    fc
}

// =============================================================================
// remove_funccal
// =============================================================================

#[unsafe(export_name = "remove_funccal")]
pub unsafe extern "C" fn rs_remove_funccal() {
    let fc = unsafe { get_current_funccal() };
    let caller = unsafe { nvim_fc_get_caller(fc) };
    unsafe { set_current_funccal(caller) };
    unsafe { rs_free_funccal(fc) };
}

// =============================================================================
// save_funccal
// =============================================================================
//
// Phase 13: inlined from nvim_save_funccal_impl.

#[unsafe(export_name = "save_funccal")]
pub unsafe extern "C" fn rs_save_funccal(entry: *mut c_void) {
    let cur = unsafe { get_current_funccal() };
    let stack = unsafe { nvim_funccal_stack_head_mut() };
    unsafe { nvim_fc_entry_set_top(entry, cur) };
    unsafe { nvim_fc_entry_set_next(entry, stack) };
    unsafe { nvim_set_funccal_stack(entry) };
    unsafe { nvim_set_current_funccal(std::ptr::null_mut()) };
}

// =============================================================================
// restore_funccal
// =============================================================================
//
// Phase 13: inlined from nvim_restore_funccal_impl.

#[unsafe(export_name = "restore_funccal")]
pub unsafe extern "C" fn rs_restore_funccal() {
    let stack = unsafe { nvim_funccal_stack_head_mut() };
    if stack.is_null() {
        unsafe { nvim_iemsg(c"INTERNAL: restore_funccal()".as_ptr()) };
    } else {
        let top = unsafe { nvim_funccal_entry_top(stack) };
        let next = unsafe { nvim_funccal_entry_next(stack) };
        unsafe { nvim_set_current_funccal(top) };
        unsafe { nvim_set_funccal_stack(next) };
    }
}

// =============================================================================
// ex_delfunction
// =============================================================================
//
// Phase 32: inlined from nvim_ex_delfunction_impl.
// funcdict_T is 3 pointers = 24 bytes on 64-bit.
const SIZEOF_FUNCDICT: usize = 24;

#[unsafe(export_name = "ex_delfunction")]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_ex_delfunction(eap: *mut c_void) {
    let mut fudi = [0u8; SIZEOF_FUNCDICT];
    let fudi_ptr = fudi.as_mut_ptr().cast::<c_void>();

    // p = eap->arg; then trans_function_name advances *pp
    let mut p = unsafe { nvim_eap_get_arg(eap) };
    let skip = unsafe { nvim_eap_get_skip(eap) };
    let name = unsafe {
        trans_function_name(
            std::ptr::addr_of_mut!(p),
            skip,
            0,
            fudi_ptr,
            std::ptr::null_mut(),
        )
    };

    // xfree(fudi.fd_newkey)
    let newkey = unsafe { nvim_fudi_get_newkey(fudi_ptr) };
    if !newkey.is_null() {
        unsafe { xfree(newkey.cast::<c_void>()) };
    }

    if name.is_null() {
        let fd_dict = unsafe { nvim_fudi_get_dict(fudi_ptr) };
        if !fd_dict.is_null() && skip == 0 {
            unsafe { nvim_emsg_funcref() };
        }
        return;
    }

    if unsafe { nvim_ends_excmd_skipwhite(p) } == 0 {
        unsafe { xfree(name.cast::<c_void>()) };
        unsafe { nvim_emsg_trailing_arg(p) };
        return;
    }

    let nextcmd = unsafe { check_nextcmd(p) };
    unsafe { nvim_eap_set_nextcmd(eap, nextcmd) };
    if !nextcmd.is_null() {
        // *p = NUL
        unsafe { *p.cast::<u8>() = 0 };
    }

    let fd_dict = unsafe { nvim_fudi_get_dict(fudi_ptr) };
    let first_byte = unsafe { *name.cast::<u8>() };
    if first_byte.is_ascii_digit() && fd_dict.is_null() {
        if skip == 0 {
            let eap_arg = unsafe { nvim_eap_get_arg(eap) };
            unsafe { nvim_semsg_e_invarg2(eap_arg) };
        }
        unsafe { xfree(name.cast::<c_void>()) };
        return;
    }

    let fp = if skip == 0 {
        unsafe { find_func(name) }
    } else {
        std::ptr::null_mut()
    };
    unsafe { xfree(name.cast::<c_void>()) };

    if skip == 0 {
        let eap_arg = unsafe { nvim_eap_get_arg(eap) };
        if fp.is_null() {
            if unsafe { nvim_eap_get_forceit_int(eap) } == 0 {
                unsafe { nvim_semsg_nofunc(eap_arg) };
            }
            return;
        }
        if unsafe { nvim_ufunc_get_calls(fp) } > 0 {
            unsafe { nvim_semsg_e131_in_use(eap_arg) };
            return;
        }
        // check `uf_refcount > 2` because deleting a function should also reduce
        // the reference count, and 1 is the initial refcount.
        if unsafe { nvim_ufunc_get_refcount(fp) } > 2 {
            unsafe { nvim_semsg_cannot_delete_internal(eap_arg) };
            return;
        }

        if fd_dict.is_null() {
            let fname = unsafe { nvim_ufunc_get_name(fp) };
            // refcount threshold: 0 if numbered/lambda (name_refcount != 0), else 1
            let refcount_threshold =
                c_int::from(unsafe { crate::names::rs_func_name_refcount(fname) } == 0);
            if unsafe { nvim_ufunc_get_refcount(fp) } > refcount_threshold {
                // Function still referenced. Remove from hashtable but keep.
                if unsafe { crate::refcount::rs_func_remove(fp) } != 0 {
                    unsafe { nvim_ufunc_decrement_refcount(fp) };
                }
                unsafe { nvim_ufunc_or_flags_deleted(fp) };
            } else {
                unsafe { crate::refcount::rs_func_clear_free(fp, 0) };
            }
        } else {
            // Delete the dict item that refers to the function; invokes func_unref().
            let di = unsafe { nvim_fudi_get_di(fudi_ptr) };
            unsafe { nvim_tv_dict_item_remove(fd_dict, di) };
        }
    }
}

// =============================================================================
// get_func_line
// =============================================================================
//
// Phase 33: inlined from C get_func_line.
// PROF_YES = 1 (matches C's PROF_YES enum value).
const PROF_YES: c_int = 1;
// FC_ABORT flag value (matches C's FC_ABORT).
const FC_ABORT: c_int = 0x01;

#[unsafe(export_name = "get_func_line")]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_get_func_line(
    _c: c_int,
    cookie: *mut c_void,
    _indent: c_int,
    _do_concat: bool,
) -> *mut c_char {
    let fcp = cookie; // funccall_T *
    let fp = unsafe { nvim_fc_get_func(fcp) };

    // If breakpoints have been added/deleted, need to check for it.
    let dbg_tick = unsafe { debug_tick };
    let dbg_tick_ptr = unsafe { nvim_fc_get_dbg_tick_ptr(fcp) };
    if unsafe { *dbg_tick_ptr } != dbg_tick {
        let sourcing_lnum = unsafe { nvim_get_sourcing_lnum_direct() };
        let fname = unsafe { nvim_ufunc_get_name(fp) };
        let bp = unsafe { dbg_find_breakpoint(false, fname, sourcing_lnum) };
        let bp_ptr = unsafe { nvim_fc_get_breakpoint_ptr(fcp) };
        unsafe { *bp_ptr = bp };
        unsafe { *dbg_tick_ptr = dbg_tick };
    }
    if unsafe { do_profiling } == PROF_YES {
        unsafe { func_line_end(cookie) };
    }

    let lines_len = unsafe { nvim_ufunc_get_lines_len(fp) };
    let flags = unsafe { nvim_ufunc_get_flags(fp) };
    let fc_returned = unsafe { nvim_fc_get_returned(fcp) } != 0;
    let aborted = (flags & FC_ABORT != 0) && unsafe { did_emsg } != 0 && !unsafe { aborting() };

    let retval: *mut c_char;
    if aborted || fc_returned {
        retval = std::ptr::null_mut();
    } else {
        // Skip NULL lines (continuation lines).
        while unsafe { nvim_fc_get_linenr(fcp) } < lines_len
            && unsafe { nvim_ufunc_funcline_is_null(fp, nvim_fc_get_linenr(fcp)) } != 0
        {
            let linenr = unsafe { nvim_fc_get_linenr(fcp) };
            unsafe { nvim_fc_set_linenr(fcp, linenr + 1) };
        }
        if unsafe { nvim_fc_get_linenr(fcp) } >= lines_len {
            retval = std::ptr::null_mut();
        } else {
            let line_idx = unsafe { nvim_fc_postincrement_linenr(fcp) };
            let line = unsafe { nvim_ufunc_get_funcline(fp, line_idx) };
            retval = unsafe { xstrdup(line) };
            // SOURCING_LNUM = fcp->fc_linenr (after increment)
            unsafe { nvim_rt_set_sourcing_lnum(nvim_fc_get_linenr(fcp)) };
            if unsafe { do_profiling } == PROF_YES {
                unsafe { func_line_start(cookie) };
            }
        }
    }

    // Did we encounter a breakpoint?
    let bp_ptr = unsafe { nvim_fc_get_breakpoint_ptr(fcp) };
    let breakpoint = unsafe { *bp_ptr };
    let sourcing_lnum = unsafe { nvim_get_sourcing_lnum_direct() };
    if breakpoint != 0 && breakpoint <= sourcing_lnum {
        let fname = unsafe { nvim_ufunc_get_name(fp) };
        unsafe { dbg_breakpoint(fname, sourcing_lnum) };
        // Find next breakpoint.
        let bp = unsafe { dbg_find_breakpoint(false, fname, sourcing_lnum) };
        unsafe { *bp_ptr = bp };
        unsafe { *nvim_fc_get_dbg_tick_ptr(fcp) = debug_tick };
    }

    retval
}

// =============================================================================
// emsg_funcname
// =============================================================================
//
// Phase 13: inlined from nvim_emsg_funcname_impl.

#[unsafe(export_name = "emsg_funcname")]
pub unsafe extern "C" fn rs_emsg_funcname(errmsg: *const c_char, name: *const c_char) {
    let snr = unsafe { nvim_emsg_funcname_mk_snr(name) };
    let display = if snr.is_null() {
        name
    } else {
        snr.cast_const()
    };
    unsafe { nvim_semsg_with_name(errmsg, display) };
    if !snr.is_null() {
        unsafe { xfree(snr.cast::<c_void>()) };
    }
}

// =============================================================================
// user_func_error
// =============================================================================
//
// Phase 14: inlined from nvim_user_func_error_impl.
// FCERR constants must match userfunc.h

const FCERR_UNKNOWN: c_int = 0;
const FCERR_NOTMETHOD: c_int = 8;
const FCERR_DELETED: c_int = 7;
const FCERR_TOOMANY: c_int = 1;
const FCERR_TOOFEW: c_int = 2;
const FCERR_SCRIPT: c_int = 3;
const FCERR_DICT: c_int = 4;

#[no_mangle]
pub unsafe extern "C" fn rs_user_func_error(error: c_int, name: *const c_char, found_var: c_int) {
    match error {
        FCERR_UNKNOWN => {
            if found_var != 0 {
                unsafe { nvim_semsg_not_callable(name) };
            } else {
                unsafe { rs_emsg_funcname(c"E117: Unknown function: %s".as_ptr(), name) };
            }
        }
        FCERR_NOTMETHOD => unsafe {
            rs_emsg_funcname(c"E276: Cannot use function as a method: %s".as_ptr(), name);
        },
        FCERR_DELETED => unsafe {
            rs_emsg_funcname(c"E933: Function was deleted: %s".as_ptr(), name);
        },
        FCERR_TOOMANY => unsafe {
            rs_emsg_funcname(c"E118: Too many arguments for function: %s".as_ptr(), name);
        },
        FCERR_TOOFEW => unsafe {
            rs_emsg_funcname(
                c"E119: Not enough arguments for function: %s".as_ptr(),
                name,
            );
        },
        FCERR_SCRIPT => unsafe {
            rs_emsg_funcname(
                c"E120: Using <SID> not in a script context: %s".as_ptr(),
                name,
            );
        },
        FCERR_DICT => unsafe {
            rs_emsg_funcname(
                c"E725: Calling dict function without Dictionary: %s".as_ptr(),
                name,
            );
        },
        _ => {}
    }
}

// =============================================================================
// callback_call_retnr
// =============================================================================
//
// Phase 15: Migrated from userfunc.c.

/// Call a callback and return the result as a number.
/// Returns -2 when calling the function fails.
///
/// # Safety
/// `callback` must be a valid `Callback *` pointer.
/// `argvars` must be a valid `typval_T *` array of at least `argcount` + 1 elements.
#[unsafe(export_name = "callback_call_retnr")]
pub unsafe extern "C" fn rs_callback_call_retnr(
    callback: *mut c_void,
    argcount: c_int,
    argvars: *mut c_void,
) -> i64 {
    // typval_T is 16 bytes (i32 v_type, i32 v_lock, 8-byte union vval).
    // Zero-initializing gives VAR_UNKNOWN (v_type = 0), which is safe.
    let mut rettv = [0u8; 16];
    let rettv_ptr = rettv.as_mut_ptr().cast::<c_void>();
    if !unsafe { callback_call(callback, argcount, argvars, rettv_ptr) } {
        return -2;
    }
    let retval = unsafe { tv_get_number_chk(rettv_ptr.cast_const(), std::ptr::null_mut()) };
    unsafe { tv_clear(rettv_ptr) };
    retval
}

// =============================================================================
// call_simple_luafunc
// =============================================================================
//
// Phase 16: Migrated from userfunc.c.

/// Call a Lua function by name without arguments.
///
/// # Safety
/// `funcname` must be a valid string pointer of at least `len` bytes.
/// `rettv` must be a valid `typval_T *`.
#[unsafe(export_name = "call_simple_luafunc")]
pub unsafe extern "C" fn rs_call_simple_luafunc(
    funcname: *const c_char,
    len: usize,
    rettv: *mut c_void,
) -> c_int {
    // Set default rettv to number zero.
    unsafe { nvim_tv_set_number(rettv, 0) };
    // typval_T argvars[1]; argvars[0].v_type = VAR_UNKNOWN (0)
    let mut argvars = [0u8; 16];
    unsafe {
        nlua_typval_call(
            funcname,
            len,
            argvars.as_mut_ptr().cast::<c_void>(),
            0,
            rettv,
        );
    };
    OK
}

// =============================================================================
// call_simple_func
// =============================================================================
//
// Phase 16: Migrated from userfunc.c.

/// Call a VimL function by name without arguments.
/// Returns NOTDONE when the function could not be found.
///
/// # Safety
/// `funcname` must be a valid string pointer of at least `len` bytes.
/// `rettv` must be a valid `typval_T *`.
#[unsafe(export_name = "call_simple_func")]
pub unsafe extern "C" fn rs_call_simple_func(
    funcname: *const c_char,
    len: usize,
    rettv: *mut c_void,
) -> c_int {
    let mut ret = FAIL;

    // Set default rettv to number zero.
    unsafe { nvim_tv_set_number(rettv, 0) };

    // Make a copy of the name, an option can be changed in the function.
    let name = unsafe { xstrnsave(funcname, len) };

    let mut error: c_int = FCERR_NONE;
    let mut tofree: *mut c_char = std::ptr::null_mut();
    let mut fname_buf = [0u8; FLEN_FIXED + 1];
    let fname = unsafe {
        rs_fname_trans_sid(
            name,
            fname_buf.as_mut_ptr().cast::<c_char>(),
            std::ptr::addr_of_mut!(tofree),
            std::ptr::addr_of_mut!(error),
        )
    };

    // Skip "g:" before a function name.
    let is_global = unsafe { *fname.cast::<u8>() == b'g' && *fname.add(1).cast::<u8>() == b':' };
    let rfname = if is_global {
        unsafe { fname.add(2) }
    } else {
        fname
    };

    let fp = unsafe { find_func(rfname) };
    if fp.is_null() {
        ret = NOTDONE;
    } else if unsafe { nvim_ufunc_get_flags(fp) } & FC_DELETED_FLAG != 0 {
        error = FCERR_DELETED;
    } else {
        // typval_T argvars[1]; argvars[0].v_type = VAR_UNKNOWN (0)
        let mut argvars = [0u8; 16];
        let argvars_ptr = argvars.as_mut_ptr().cast::<c_void>();
        error = unsafe { nvim_call_user_func_check_simple(fp, argvars_ptr, rettv) };
        if error == FCERR_NONE {
            ret = OK;
        }
    }

    unsafe { rs_user_func_error(error, name, 0) };
    unsafe { xfree(tofree.cast::<c_void>()) };
    unsafe { xfree(name.cast::<c_void>()) };
    ret
}

// =============================================================================
// call_user_func_check
// =============================================================================
//
// Phase 21: migrated from userfunc.c static function.

/// Call a user function after checking the arguments.
///
/// Returns FCERR_NONE (5) on success, or an FCERR_* error code on failure.
///
/// # Safety
/// `fp`, `argvars`, `rettv`, `funcexe` must be valid non-null pointers.
/// `selfdict` may be null.
#[unsafe(export_name = "call_user_func_check")]
pub unsafe extern "C" fn rs_call_user_func_check(
    fp: *mut c_void,
    argcount: c_int,
    argvars: *mut c_void,
    rettv: *mut c_void,
    funcexe: *mut c_void,
    selfdict: *mut c_void,
) -> c_int {
    let flags = unsafe { nvim_ufunc_get_flags(fp) };

    // Lua function: delegate directly to Lua callable
    if flags & FC_LUAREF != 0 {
        let lua_ref = unsafe { nvim_ufunc_get_luaref(fp) };
        return unsafe { typval_exec_lua_callable(lua_ref, argcount, argvars, rettv) };
    }

    // If function takes a range and caller wants to know, mark it
    if flags & FC_RANGE != 0 {
        let doesrange = unsafe { nvim_funcexe_get_doesrange(funcexe) };
        if !doesrange.is_null() {
            unsafe { *doesrange = true };
        }
    }

    // Validate argument count
    let error = unsafe { check_user_func_argcount(fp, argcount) };
    if error != FCERR_UNKNOWN_OK {
        return error;
    }

    // Dict function requires selfdict
    if flags & FC_DICT != 0 && selfdict.is_null() {
        return FCERR_DICT;
    }

    // Call the user function
    let firstline = unsafe { nvim_funcexe_get_firstline(funcexe) };
    let lastline = unsafe { nvim_funcexe_get_lastline(funcexe) };
    let effective_selfdict = if flags & FC_DICT != 0 {
        selfdict
    } else {
        std::ptr::null_mut()
    };
    unsafe {
        call_user_func(
            fp,
            argcount,
            argvars,
            rettv,
            firstline,
            lastline,
            effective_selfdict,
        );
    };

    FCERR_NONE
}

// =============================================================================
// call_func
// =============================================================================
//
// Phase 22: Migrated from userfunc.c.

/// strlen for a *const c_char (NUL-terminated).
fn call_func_strlen(s: *const c_char) -> usize {
    let mut len = 0usize;
    while unsafe { *s.add(len) } != 0 {
        len += 1;
    }
    len
}

/// Central function call dispatcher for VimL.
///
/// Calls user-defined functions, built-in functions, Lua functions, or
/// method functions depending on `funcname` and `funcexe`.
///
/// # Safety
/// All pointer arguments must be valid. `funcname` must be a valid C string.
/// `rettv`, `argvars_in`, `funcexe` must be non-null.
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::too_many_lines)]
#[allow(clippy::nonminimal_bool)]
#[unsafe(export_name = "call_func")]
pub unsafe extern "C" fn rs_call_func(
    funcname: *const c_char,
    len: c_int,
    rettv: *mut c_void,
    argcount_in: c_int,
    argvars_in: *mut c_void,
    funcexe: *mut c_void,
) -> c_int {
    let mut ret = FAIL;
    let mut error = FCERR_NONE;
    let mut fp: *mut c_void = std::ptr::null_mut();
    let mut fname_buf = [0u8; FLEN_FIXED + 1];
    let mut tofree: *mut c_char = std::ptr::null_mut();
    let mut fname: *mut c_char = std::ptr::null_mut();
    let mut name: *mut c_char = std::ptr::null_mut();
    let mut argcount = argcount_in;
    let mut argvars = argvars_in;

    // argv stack buffer: (MAX_FUNC_ARGS + 1) * SIZEOF_TYPVAL bytes
    let mut argv_buf = [0u8; (MAX_FUNC_ARGS + 1) * SIZEOF_TYPVAL];
    let argv = argv_buf.as_mut_ptr().cast::<c_void>();
    let mut argv_clear: c_int = 0;
    let mut argv_base: c_int = 0;

    let selfdict = unsafe { nvim_funcexe_get_selfdict(funcexe.cast_const()) };
    let partial = unsafe { nvim_funcexe_get_partial(funcexe.cast_const()) };

    // Initialize rettv so caller can safely tv_clear(rettv) even on FAIL.
    unsafe { nvim_tv_set_unknown(rettv) };

    let len = if len <= 0 {
        call_func_strlen(funcname) as c_int
    } else {
        len
    };

    if !partial.is_null() {
        fp = unsafe { nvim_partial_get_func(partial.cast_const()) };
    }

    if fp.is_null() {
        // Copy the name so it won't be changed by the called function.
        name = unsafe { xmemdupz(funcname.cast::<c_void>(), len as usize) }.cast::<c_char>();
        fname = unsafe {
            rs_fname_trans_sid(
                name,
                fname_buf.as_mut_ptr().cast::<c_char>(),
                std::ptr::addr_of_mut!(tofree),
                std::ptr::addr_of_mut!(error),
            )
        };
    }

    // Clear doesrange flag
    let doesrange_ptr = unsafe { nvim_funcexe_get_doesrange(funcexe) };
    if !doesrange_ptr.is_null() {
        unsafe { *doesrange_ptr = false };
    }

    // Compute effective selfdict from partial
    let selfdict = if partial.is_null() {
        selfdict
    } else {
        let pt_dict = unsafe { nvim_partial_get_dict(partial.cast_const()) };
        let pt_auto = unsafe { nvim_partial_get_auto(partial.cast_const()) };
        if !pt_dict.is_null() && (selfdict.is_null() || !pt_auto) {
            pt_dict
        } else {
            selfdict
        }
    };

    // Prepend partial args to argv, then append caller args
    let mut toomany = false;
    if !partial.is_null() && error == FCERR_NONE {
        let pt_argc = unsafe { nvim_partial_get_argc(partial.cast_const()) };
        if pt_argc > 0 {
            while argv_clear < pt_argc {
                if argv_clear + argcount_in >= MAX_FUNC_ARGS as c_int {
                    toomany = true;
                    break;
                }
                let src = unsafe {
                    nvim_partial_get_argv(partial.cast_const())
                        .cast::<u8>()
                        .add(argv_clear as usize * SIZEOF_TYPVAL)
                        .cast::<c_void>()
                };
                let dst = unsafe {
                    argv.cast::<u8>()
                        .add(argv_clear as usize * SIZEOF_TYPVAL)
                        .cast::<c_void>()
                };
                unsafe { tv_copy(src.cast_const(), dst) };
                argv_clear += 1;
            }
            if !toomany {
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        argvars_in.cast::<u8>(),
                        argv.cast::<u8>().add(argv_clear as usize * SIZEOF_TYPVAL),
                        argcount_in as usize * SIZEOF_TYPVAL,
                    );
                }
                argvars = argv;
                argcount = pt_argc + argcount_in;
            }
        }
    }

    // If partial args overflowed, skip evaluation (matches C `goto theend`)
    if !toomany && error == FCERR_NONE && unsafe { nvim_funcexe_get_evaluate(funcexe.cast_const()) }
    {
        // Skip "g:" prefix
        let is_global = fp.is_null()
            && !fname.is_null()
            && unsafe { *fname.cast::<u8>() == b'g' && *fname.add(1).cast::<u8>() == b':' };
        let rfname = if is_global {
            unsafe { fname.add(2) }
        } else {
            fname
        };

        // Set rettv default: number 0
        unsafe { nvim_tv_set_number(rettv, 0) };
        error = FCERR_UNKNOWN;

        if unsafe { rs_is_luafunc(partial) } {
            if len > 0 {
                error = FCERR_NONE;
                unsafe {
                    argv_add_base(
                        nvim_funcexe_get_basetv(funcexe.cast_const()).cast_const(),
                        std::ptr::addr_of_mut!(argvars),
                        std::ptr::addr_of_mut!(argcount),
                        argv,
                        std::ptr::addr_of_mut!(argv_base),
                    );
                }
                unsafe { nlua_typval_call(funcname, len as usize, argvars, argcount, rettv) };
            } else {
                // v:lua called directly; funcname is already "v:lua" for error
                unsafe { xfree(name.cast::<c_void>()) };
                name = std::ptr::null_mut();
            }
        } else if fp.is_null() && unsafe { rs_builtin_function(rfname, -1) } != 0 {
            // Built-in or method function
            let basetv = unsafe { nvim_funcexe_get_basetv(funcexe.cast_const()) };
            if basetv.is_null() {
                error = unsafe { call_internal_func(fname, argcount, argvars, rettv) };
            } else {
                error = unsafe { call_internal_method(fname, argcount, argvars, rettv, basetv) };
            }
        } else {
            // User defined function
            if fp.is_null() {
                fp = unsafe { find_func(rfname) };
            }

            // Trigger FuncUndefined autocommand
            if fp.is_null()
                && unsafe { apply_autocmds_for_funcundefined(rfname) } != 0
                && !unsafe { aborting() }
            {
                fp = unsafe { find_func(rfname) };
            }

            // Try loading a package
            if fp.is_null() {
                let rlen = call_func_strlen(rfname);
                if unsafe { script_autoload(rfname, rlen, true) && !aborting() } {
                    fp = unsafe { find_func(rfname) };
                }
            }

            if !fp.is_null() && unsafe { nvim_ufunc_get_flags(fp) } & FC_DELETED_FLAG != 0 {
                error = FCERR_DELETED;
            } else if !fp.is_null() {
                argcount = unsafe {
                    nvim_funcexe_call_argv_func(funcexe, argcount, argvars, argv_clear, fp)
                };
                unsafe {
                    argv_add_base(
                        nvim_funcexe_get_basetv(funcexe.cast_const()).cast_const(),
                        std::ptr::addr_of_mut!(argvars),
                        std::ptr::addr_of_mut!(argcount),
                        argv,
                        std::ptr::addr_of_mut!(argv_base),
                    );
                }
                error = unsafe {
                    rs_call_user_func_check(fp, argcount, argvars, rettv, funcexe, selfdict)
                };
            }
        }
        // Update force_abort flag for reliable aborting() detection
        unsafe { update_force_abort() };
    }

    if error == FCERR_NONE {
        ret = OK;
    }

    // Report error unless call was aborted
    if !unsafe { aborting() } {
        let err_name = if name.is_null() {
            funcname
        } else {
            name.cast_const()
        };
        let found_var = unsafe { nvim_funcexe_get_found_var(funcexe.cast_const()) };
        unsafe { rs_user_func_error(error, err_name, c_int::from(found_var)) };
    }

    // Clear partial arg copies
    while argv_clear > 0 {
        argv_clear -= 1;
        let slot = unsafe {
            argv.cast::<u8>()
                .add((argv_clear + argv_base) as usize * SIZEOF_TYPVAL)
                .cast::<c_void>()
        };
        unsafe { tv_clear(slot) };
    }

    unsafe { xfree(tofree.cast::<c_void>()) };
    unsafe { xfree(name.cast::<c_void>()) };

    ret
}

// =============================================================================
// get_func_tv
// =============================================================================
//
// Phase 23: Migrated from userfunc.c.

/// Parse function arguments and call the function.
///
/// # Safety
/// All pointers must be valid. `name` must be a NUL-terminated C string.
/// `arg` must point to a valid C string pointer. `evalarg` may be null.
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_sign_loss)]
#[unsafe(export_name = "get_func_tv")]
pub unsafe extern "C" fn rs_get_func_tv(
    name: *const c_char,
    len: c_int,
    rettv: *mut c_void,
    arg: *mut *mut c_char,
    evalarg: *mut c_void,
    funcexe: *mut c_void,
) -> c_int {
    // argvars: (MAX_FUNC_ARGS + 1) * SIZEOF_TYPVAL bytes, zero-initialized
    let mut argvars_buf = [0u8; (MAX_FUNC_ARGS + 1) * SIZEOF_TYPVAL];
    let argvars = argvars_buf.as_mut_ptr().cast::<c_void>();
    let mut argcount: c_int = 0;

    let evaluate = unsafe { nvim_evalarg_should_evaluate(evalarg) };

    let mut argp = unsafe { *arg };

    // Get partial_argc from funcexe->fe_partial->pt_argc
    let partial = unsafe { nvim_funcexe_get_partial(funcexe.cast_const()) };
    let partial_argc = if partial.is_null() {
        0
    } else {
        unsafe { nvim_partial_get_argc(partial.cast_const()) }
    };

    let ret = unsafe {
        get_func_arguments(
            std::ptr::addr_of_mut!(argp),
            evalarg,
            partial_argc,
            argvars,
            std::ptr::addr_of_mut!(argcount),
        )
    };

    #[allow(clippy::cast_possible_truncation)]
    let max_func_args: c_int = MAX_FUNC_ARGS as c_int;
    let ret = if ret == OK {
        let mut i: c_int = 0;

        if unsafe { nvim_get_testing_flag() } != 0 {
            // Register argvars for test_garbagecollect_now()
            if unsafe { nvim_funcargs_ga_itemsize() } == 0 {
                unsafe { nvim_funcargs_ga_init() };
            }
            while i < argcount {
                unsafe { nvim_funcargs_ga_grow() };
                let tv_slot = unsafe {
                    argvars
                        .cast::<u8>()
                        .add(i as usize * SIZEOF_TYPVAL)
                        .cast::<c_void>()
                };
                unsafe { nvim_funcargs_push_tv_ptr(tv_slot) };
                i += 1;
            }
        }

        let r = unsafe { rs_call_func(name, len, rettv, argcount, argvars, funcexe) };
        unsafe { nvim_funcargs_dec_len(i) };
        r
    } else {
        if evaluate && !unsafe { aborting() } {
            if argcount == max_func_args {
                unsafe { nvim_emsg_e740_too_many_args(name) };
            } else {
                unsafe { nvim_emsg_e116_invalid_args(name) };
            }
        }
        FAIL
    };

    // Clear argument typvals
    let mut ac = argcount;
    while ac > 0 {
        ac -= 1;
        let slot = unsafe {
            argvars
                .cast::<u8>()
                .add(ac as usize * SIZEOF_TYPVAL)
                .cast::<c_void>()
        };
        unsafe { tv_clear(slot) };
    }

    unsafe { *arg = skipwhite(argp) };
    ret
}

// =============================================================================
// ex_call_inner  (Phase 25)
// =============================================================================

/// Size of funcexe_T in bytes (must match C's sizeof(funcexe_T)).
const SIZEOF_FUNCEXE: usize = 64;
/// Byte offset of fe_doesrange in funcexe_T.
const FUNCEXE_DOESRANGE_OFFSET: usize = 16;

/// Inner loop for `:call func(args)` with optional range.
///
/// Phase 25: migrated from static C `ex_call_inner`.
///
/// # Safety
/// All pointers must be valid.
#[unsafe(export_name = "ex_call_inner")]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_ex_call_inner(
    eap: *const c_void,
    name: *const c_char,
    arg: *mut *mut c_char,
    startarg: *mut c_char,
    funcexe_init: *const c_void,
    evalarg: *mut c_void,
) -> c_int {
    let mut doesrange: bool = false;
    let mut failed = false;

    let line1 = unsafe { nvim_eap_get_line1(eap) };
    let line2 = unsafe { nvim_eap_get_line2(eap) };
    let addr_count = unsafe { nvim_eap_get_addr_count(eap) };

    let mut lnum = line1;
    while lnum <= line2 {
        if addr_count > 0 {
            // Check line count; advance cursor if valid.
            if unsafe { nvim_ex_call_check_advance_cursor(lnum) } != 0 {
                // lnum > line count: function deleted lines or switched buffer
                unsafe { emsg(gettext(e_invrange_uf.as_ptr().cast())) };
                failed = true;
                break;
            }
        }
        unsafe { *arg = startarg };

        // Build a local copy of funcexe with fe_doesrange = &doesrange.
        let mut funcexe = [0u8; SIZEOF_FUNCEXE];
        unsafe {
            std::ptr::copy_nonoverlapping(
                funcexe_init.cast::<u8>(),
                funcexe.as_mut_ptr(),
                SIZEOF_FUNCEXE,
            );
        };
        let doesrange_ptr: *mut bool = &raw mut doesrange;
        // Write the pointer into the byte array at the correct offset.
        // Use write_unaligned to avoid alignment UB (byte array is u8-aligned).
        #[allow(clippy::cast_ptr_alignment)]
        unsafe {
            let slot = funcexe
                .as_mut_ptr()
                .add(FUNCEXE_DOESRANGE_OFFSET)
                .cast::<*mut bool>();
            std::ptr::write_unaligned(slot, doesrange_ptr);
        };
        let funcexe_ptr = funcexe.as_mut_ptr().cast::<c_void>();

        // Allocate rettv on the stack (typval_T = 16 bytes).
        let mut rettv = [0u8; SIZEOF_TYPVAL];
        let rettv_ptr = rettv.as_mut_ptr().cast::<c_void>();
        unsafe { nvim_tv_set_unknown(rettv_ptr) }; // v_type = VAR_UNKNOWN

        if unsafe { rs_get_func_tv(name, -1, rettv_ptr, arg, evalarg, funcexe_ptr) } == FAIL {
            failed = true;
            break;
        }

        // Handle function returning a Funcref, Dict, or List.
        if unsafe { nvim_handle_subscript_eval_evaluate(arg, rettv_ptr) } == FAIL {
            failed = true;
            break;
        }

        unsafe { tv_clear(rettv_ptr) };
        if doesrange {
            break;
        }

        // Stop on abort/interrupt/exception.
        if unsafe { aborting() } {
            break;
        }

        lnum += 1;
    }

    c_int::from(failed)
}

// =============================================================================
// nvim_ex_return_impl
// =============================================================================
//
// Phase 28: inlined from nvim_ex_return_impl.
// Called via rs_ex_return in scope.rs.

// evalarg_T layout:
//   eval_flags   (int)        offset 0,  size 4
//   [padding]                 offset 4,  size 4
//   eval_getline (fn ptr)     offset 8,  size 8
//   eval_cookie  (void*)      offset 16, size 8
//   eval_tofree  (char*)      offset 24, size 8
// Total: 32 bytes (verified by eval_struct_check.c static assert)
const SIZEOF_EVALARG: usize = 32;
const EVAL_EVALUATE: u32 = 1; // matches C's EVAL_EVALUATE enum value

#[no_mangle]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn nvim_ex_return_impl(eap: *mut c_void) {
    let arg = unsafe { nvim_eap_get_arg(eap) };

    if unsafe { get_current_funccal() }.is_null() {
        unsafe { nvim_emsg_return_not_in_func() };
        return;
    }

    let skip = unsafe { nvim_eap_get_skip(eap) };

    // evalarg_T evalarg = { .eval_flags = eap->skip ? 0 : EVAL_EVALUATE };
    let mut evalarg = [0u8; SIZEOF_EVALARG];
    if skip == 0 {
        // write EVAL_EVALUATE (1) as u32 at offset 0 (eval_flags field)
        unsafe {
            std::ptr::write_unaligned(evalarg.as_mut_ptr().cast::<u32>(), EVAL_EVALUATE);
        }
    }
    let evalarg_ptr = evalarg.as_mut_ptr().cast::<c_void>();

    if skip != 0 {
        unsafe { nvim_syn_emsg_skip_inc() };
    }

    unsafe { nvim_eap_set_nextcmd(eap, std::ptr::null_mut()) };

    // Build a local rettv buffer (typval_T, 16 bytes)
    let mut rettv = [0u8; SIZEOF_TYPVAL];
    let rettv_ptr = rettv.as_mut_ptr().cast::<c_void>();
    unsafe { nvim_tv_set_unknown(rettv_ptr) };

    // (*arg != NUL && *arg != '|' && *arg != '\n')
    let first_char = unsafe { *arg.cast::<u8>() };
    let mut returning = false;

    if first_char != 0
        && first_char != b'|'
        && first_char != b'\n'
        && unsafe { eval0(arg, rettv_ptr, eap, evalarg_ptr) } != FAIL
    {
        if skip == 0 {
            returning = unsafe { do_return(eap, 0, 1, rettv_ptr) } != 0;
        } else {
            unsafe { tv_clear(rettv_ptr) };
        }
    } else if skip == 0 {
        // It's safer to return also on error.
        // In return statement, cause_abort should be force_abort.
        unsafe { update_force_abort() };

        // Return unless the expression evaluation has been cancelled due to an
        // aborting error, an interrupt, or an exception.
        if !unsafe { aborting() } {
            returning = unsafe { do_return(eap, 0, 1, std::ptr::null_mut()) } != 0;
        }
    }

    // When skipping or the return gets pending, advance to the next command
    // in this line (!returning).  Otherwise, ignore the rest of the line.
    // Following lines will be ignored by get_func_line().
    if returning {
        unsafe { nvim_eap_set_nextcmd(eap, std::ptr::null_mut()) };
    } else if unsafe { nvim_eap_get_nextcmd(eap) }.is_null() {
        // no argument: check for nextcmd
        let next = unsafe { check_nextcmd(arg) };
        unsafe { nvim_eap_set_nextcmd(eap, next) };
    }

    if skip != 0 {
        unsafe { nvim_syn_emsg_skip_dec() };
    }
    unsafe { clear_evalarg(evalarg_ptr, eap) };
}

// =============================================================================
// func_call
// =============================================================================
//
// Phase 35: inlined from C func_call.

// funcexe_T field offsets (see userfunc.h; matches 64-byte C struct on 64-bit).
const FUNCEXE_FIRSTLINE_OFFSET: usize = 8;
const FUNCEXE_LASTLINE_OFFSET: usize = 12;
const FUNCEXE_EVALUATE_OFFSET: usize = 24;
const FUNCEXE_PARTIAL_OFFSET: usize = 32;
const FUNCEXE_SELFDICT_OFFSET: usize = 40;
const SIZEOF_FUNCEXE_FC: usize = 64;

/// Call a function with its typval list arguments.
///
/// # Safety
/// All pointers must be valid.
#[unsafe(export_name = "func_call")]
#[allow(clippy::cast_possible_wrap, clippy::similar_names)]
pub unsafe extern "C" fn rs_func_call(
    name: *mut c_char,
    args: *mut c_void,
    partial: *const c_void,
    selfdict: *mut c_void,
    rettv: *mut c_void,
) -> c_int {
    let mut argv_buf = [0u8; (MAX_FUNC_ARGS + 1) * SIZEOF_TYPVAL];
    let argv_ptr = argv_buf.as_mut_ptr().cast::<c_void>();

    let partial_argc = if partial.is_null() {
        0
    } else {
        unsafe { nvim_partial_get_argc(partial) }
    };
    #[allow(clippy::cast_possible_truncation)]
    let max_args = (MAX_FUNC_ARGS as c_int) - partial_argc;

    let arg_count = unsafe { nvim_func_call_iter_args(args, argv_ptr, max_args) };
    if arg_count < 0 {
        // Error already emitted; copies already freed by the C shim.
        return FAIL;
    }

    // Build funcexe_T on stack (zero-initialized = FUNCEXE_INIT).
    let mut funcexe = [0u8; SIZEOF_FUNCEXE_FC];
    let cursor_lnum: i32 = unsafe { nvim_curwin_cursor_lnum() };
    // fe_firstline (i32 at offset 8)
    #[allow(clippy::cast_ptr_alignment)]
    unsafe {
        std::ptr::write_unaligned(
            funcexe
                .as_mut_ptr()
                .add(FUNCEXE_FIRSTLINE_OFFSET)
                .cast::<i32>(),
            cursor_lnum,
        );
    }
    // fe_lastline (i32 at offset 12)
    #[allow(clippy::cast_ptr_alignment)]
    unsafe {
        std::ptr::write_unaligned(
            funcexe
                .as_mut_ptr()
                .add(FUNCEXE_LASTLINE_OFFSET)
                .cast::<i32>(),
            cursor_lnum,
        );
    }
    // fe_evaluate (bool at offset 24) = true
    funcexe[FUNCEXE_EVALUATE_OFFSET] = 1;
    // fe_partial (ptr at offset 32)
    #[allow(clippy::cast_ptr_alignment)]
    unsafe {
        std::ptr::write_unaligned(
            funcexe
                .as_mut_ptr()
                .add(FUNCEXE_PARTIAL_OFFSET)
                .cast::<*const c_void>(),
            partial,
        );
    }
    // fe_selfdict (ptr at offset 40)
    #[allow(clippy::cast_ptr_alignment)]
    unsafe {
        std::ptr::write_unaligned(
            funcexe
                .as_mut_ptr()
                .add(FUNCEXE_SELFDICT_OFFSET)
                .cast::<*mut c_void>(),
            selfdict,
        );
    }

    let r = unsafe {
        rs_call_func(
            name,
            -1,
            rettv,
            arg_count,
            argv_ptr,
            funcexe.as_mut_ptr().cast::<c_void>(),
        )
    };

    // Free the arguments.
    let mut i = arg_count - 1;
    while i >= 0 {
        let slot = unsafe {
            argv_ptr
                .cast::<u8>()
                .add(i as usize * SIZEOF_TYPVAL)
                .cast::<c_void>()
        };
        unsafe { tv_clear(slot) };
        i -= 1;
    }

    r
}

// =============================================================================
// call_user_func (Phase A / plan 5e037151)
// =============================================================================
//
// Phase A: Rust scaffold for call_user_func. Exported as `rs_call_user_func`
// (distinct symbol to avoid duplicate-symbol link error with C `call_user_func`).
// Phase B will flip the export name to `call_user_func` and delete the C body.

// Recursion depth guard (mirrors C function-static `static int depth`).
static CUF_DEPTH: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

// DOCMD flags
const DOCMD_NOWAIT: c_int = 0x02;
const DOCMD_VERBOSE: c_int = 0x01;
const DOCMD_REPEAT: c_int = 0x04;

// MSG_BUF constants
const MSG_BUF_LEN: usize = 480;
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
const MSG_BUF_CLEN: c_int = (MSG_BUF_LEN / 6) as c_int; // 80

/// Rust port of C `call_user_func` (Phase A: exported under distinct name).
///
/// This is the VimL user-function execution engine. It matches the C body
/// branch-for-branch per the 23-point semantics checklist in the migration plan.
///
/// # Safety
/// All pointer arguments must be valid. `fp`, `argvars`, `rettv` are non-null.
/// `selfdict` may be null.
#[no_mangle]
#[allow(clippy::too_many_lines)]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::similar_names)]
#[allow(clippy::cast_lossless)]
#[allow(clippy::ptr_cast_constness)]
#[allow(clippy::semicolon_if_nothing_returned)]
#[allow(clippy::cast_ptr_alignment)]
#[allow(clippy::ptr_as_ptr)]
#[allow(clippy::needless_range_loop)]
#[allow(clippy::unnecessary_operation)]
pub unsafe extern "C" fn rs_call_user_func(
    fp: *mut c_void,
    argcount: c_int,
    argvars: *mut c_void,
    rettv: *mut c_void,
    firstline: i32,
    lastline: i32,
    selfdict: *mut c_void,
) {
    use std::sync::atomic::Ordering;

    // --- Depth guard (checklist #1) ---
    // Early return BEFORE any save/alloc. depth++ after the guard.
    let depth = CUF_DEPTH.load(Ordering::Relaxed);
    let p_mfd_val = unsafe { p_mfd } as c_int;
    if depth >= p_mfd_val {
        unsafe { nvim_emsg_e132() };
        unsafe { nvim_tv_set_number(rettv, -1) };
        return;
    }
    CUF_DEPTH.store(depth + 1, Ordering::Relaxed);

    // --- Save search patterns + redo buffer (checklist #2) ---
    unsafe { save_search_patterns() };
    let redo_size = unsafe { nvim_sizeof_save_redo() };
    let save_redo = unsafe { xcalloc(1, redo_size) };
    let did_save_redo = if unsafe { rs_ins_compl_active() } == 0 {
        unsafe { saveRedobuff(save_redo) };
        true
    } else {
        false
    };

    // fp->uf_calls++; line_breakcheck()
    unsafe { nvim_ufunc_incr_calls(fp) };
    unsafe { line_breakcheck() };

    // --- Create funccal (checklist #3) ---
    let fc = unsafe { rs_create_funccal(fp, rettv) };
    // fc->fc_level = ex_nesting_level (we read via ex_nesting_level_incr trick:
    // we need the current value, not to increment. We'll save it by calling
    // nvim_fc_set_level with the result of nvim_get_ex_nesting_level).
    // We don't have a getter, but we can: increment then decrement to get it.
    // Actually, we have no getter. Let's add one... but the plan says we can
    // use nvim_ex_nesting_level_incr/decr. A cleaner approach: since ex_nesting_level
    // is only written from a few places, we can use a C accessor that just reads it.
    // Per plan, we add a getter when needed. Use nvim_get_ex_nesting_level.
    // WAIT: the plan says incr/decr only. But we need the VALUE here.
    // The plan lists: nvim_ex_nesting_level_incr/decr. We need fc_level = ex_nesting_level.
    // We cannot get the value without a getter. We must add one.
    // Since the plan says "add inc/dec only if a writable static is not already viable",
    // and since we genuinely need the current value, we add a getter.
    // However, to stay within plan scope: we can use a C shim
    // nvim_get_ex_nesting_level() - we'll add it below. For now, use a local workaround:
    // We temporarily call nvim_ex_nesting_level_incr, read a C global, then decr.
    // That's wrong. We need to call a getter. Let's just add a getter accessor to userfunc.c
    // as a 1-liner (it's a read-only global access, justified by plan assumption 2).
    // For this initial compile, we call the C function that was already declared above.
    // Since we don't have the getter yet, we'll use a C helper below.
    // For now: use a separate extern declared below.
    unsafe { nvim_fc_set_level(fc, nvim_get_ex_nesting_level()) };

    // fc_breakpoint, fc_dbg_tick
    let fp_name = unsafe { nvim_ufunc_get_name_ptr(fp) };
    let bp = unsafe { dbg_find_breakpoint(false, fp_name.cast_const(), 0) };
    let bp_ptr = unsafe { nvim_fc_get_breakpoint_ptr(fc) };
    unsafe { *bp_ptr = bp };
    let dbg_tick_val = unsafe { debug_tick };
    let dbg_tick_ptr = unsafe { nvim_fc_get_dbg_tick_ptr(fc) };
    unsafe { *dbg_tick_ptr = dbg_tick_val };

    // ga_init(&fc->fc_ufuncs, sizeof(ufunc_T*), 1)
    // create_funccal already does ga_init in the current Rust impl (check):
    // Actually create_funccal just allocs and sets func/rettv/caller. ga_init for
    // fc_ufuncs is done here per the C code. We need nvim_fc_ufuncs_ga_init accessor.
    // But wait: create_funccal in funccal.rs is already Rust (rs_create_funccal).
    // Looking at rs_create_funccal: it does xcalloc (zeroes memory), so ga_len=0,
    // ga_maxlen=0 initially. Then C's ga_init sets ga_itemsize and ga_growsize.
    // We need nvim_fc_ga_init_ufuncs or just call ga_init directly on fc_ufuncs ptr.
    // The plan says: ga_init already used in crate. Let's use it.
    unsafe { nvim_fc_ufuncs_ga_init_wrapper(fc) };

    // --- islambda (checklist #4) ---
    // strncmp(fp->uf_name, "<lambda>", 8) == 0
    let islambda = !fp_name.is_null()
        && unsafe {
            let name_ptr = fp_name as *const u8;
            let prefix = b"<lambda>";
            let mut eq = true;
            for k in 0..8usize {
                if *name_ptr.add(k) != prefix[k] {
                    eq = false;
                    break;
                }
            }
            eq
        };

    // --- l: init + l:self (checklist #5) ---
    let l_vars = unsafe { nvim_fc_l_vars_dict(fc) };
    let l_vars_var = unsafe { nvim_fc_l_vars_var_ptr(fc) };
    let var_def_scope = unsafe { nvim_var_def_scope() };
    unsafe { init_var_dict(l_vars, l_vars_var, var_def_scope) };

    let mut fixvar_idx: c_int = 0;
    if !selfdict.is_null() {
        let v = unsafe { nvim_fc_fixvar_item(fc, fixvar_idx) };
        fixvar_idx += 1;
        let flags = unsafe { nvim_di_flags_ro_fix() };
        unsafe { nvim_dictitem_set_flags(v, flags) };
        unsafe { nvim_dictitem_strcpy_key(v, c"self".as_ptr()) };
        let l_vars_ht = unsafe { nvim_fc_l_vars_ht(fc) };
        let key = unsafe { nvim_dictitem_key_ptr(v) };
        unsafe { hash_add(l_vars_ht, key) };
        let var_dict = unsafe { nvim_var_dict() };
        let var_unlocked = unsafe { nvim_var_unlocked() };
        unsafe { nvim_dictitem_tv_set_type(v, var_dict) };
        unsafe { nvim_dictitem_tv_set_lock(v, var_unlocked) };
        unsafe { nvim_dictitem_tv_set_vval_dict(v, selfdict) };
        unsafe { nvim_dict_incr_refcount(selfdict) };
    }

    // --- a: init (checklist #6) ---
    let l_avars = unsafe { nvim_fc_l_avars_dict(fc) };
    let l_avars_var = unsafe { nvim_fc_l_avars_var_ptr(fc) };
    let var_scope = unsafe { nvim_var_scope() };
    unsafe { init_var_dict(l_avars, l_avars_var, var_scope) };

    let fc_noargs_flag: c_int = 0x200; // FC_NOARGS
    let uf_flags = unsafe { nvim_ufunc_get_flags(fp) };

    if (uf_flags & fc_noargs_flag) == 0 {
        // add_nr_var for a:0
        let v0 = unsafe { nvim_fc_fixvar_item(fc, fixvar_idx) };
        fixvar_idx += 1;
        let args_len = unsafe { nvim_ufunc_get_args_len(fp) };
        let a0_val = if argcount >= args_len {
            (argcount - args_len) as i64
        } else {
            0i64
        };
        unsafe { rs_add_nr_var(l_avars, v0, c"0".as_ptr(), a0_val) };
    }

    // l_avars.dv_lock = VAR_FIXED
    let var_fixed = unsafe { nvim_var_fixed() };
    unsafe { nvim_fc_l_avars_set_lock(fc, var_fixed) };

    if (uf_flags & fc_noargs_flag) == 0 {
        // a:000 variable
        let v000 = unsafe { nvim_fc_fixvar_item(fc, fixvar_idx) };
        fixvar_idx += 1;
        let flags = unsafe { nvim_di_flags_ro_fix() };
        unsafe { nvim_dictitem_set_flags(v000, flags) };
        unsafe { nvim_dictitem_strcpy_key(v000, c"000".as_ptr()) };
        let l_avars_ht = unsafe { nvim_fc_l_avars_ht(fc) };
        let key = unsafe { nvim_dictitem_key_ptr(v000) };
        unsafe { hash_add(l_avars_ht, key) };
        let var_list = unsafe { nvim_var_list() };
        unsafe { nvim_dictitem_tv_set_type(v000, var_list) };
        unsafe { nvim_dictitem_tv_set_lock(v000, var_fixed) };
        let l_varlist = unsafe { nvim_fc_l_varlist(fc) };
        unsafe { nvim_dictitem_tv_set_vval_list(v000, l_varlist) };
    }
    // tv_list_init_static(&fc->fc_l_varlist); tv_list_set_lock(VAR_FIXED)
    let l_varlist = unsafe { nvim_fc_l_varlist(fc) };
    unsafe { tv_list_init_static(l_varlist) };
    unsafe { nvim_tv_list_set_lock(l_varlist, var_fixed) };

    // --- a:firstline, a:lastline (checklist #7) ---
    if (uf_flags & fc_noargs_flag) == 0 {
        let v_fl = unsafe { nvim_fc_fixvar_item(fc, fixvar_idx) };
        fixvar_idx += 1;
        unsafe { rs_add_nr_var(l_avars, v_fl, c"firstline".as_ptr(), firstline as i64) };
        let v_ll = unsafe { nvim_fc_fixvar_item(fc, fixvar_idx) };
        fixvar_idx += 1;
        unsafe { rs_add_nr_var(l_avars, v_ll, c"lastline".as_ptr(), lastline as i64) };
    }

    // --- Arg loop (checklist #8) ---
    // NUMBUFLEN = 65, VAR_SHORT_LEN = 20, FIXVAR_CNT = 12
    let numbuflen = unsafe { nvim_get_numbuflen() } as usize;
    let var_short_len = unsafe { nvim_var_short_len() };
    let fixvar_cnt = unsafe { nvim_fixvar_cnt() };
    let args_len = unsafe { nvim_ufunc_get_args_len(fp) };
    let max_i = argcount.max(args_len);
    let mut default_arg_err = false;
    // tv_to_free: up to MAX_FUNC_ARGS (20) pointers to typval_T* needing tv_clear
    let mut tv_to_free: [*mut c_void; 20] = [std::ptr::null_mut(); 20];
    let mut tv_to_free_len: usize = 0;
    // numbuf for vararg names ("%d")
    let mut numbuf = vec![0u8; numbuflen];
    let l_vars_ht = unsafe { nvim_fc_l_vars_ht(fc) };
    let l_avars_ht = unsafe { nvim_fc_l_avars_ht(fc) };

    let mut i = 0i32;
    while i < max_i {
        let mut addlocal = false;
        let mut isdefault = false;
        // def_rettv: typval_T on stack (16 bytes, zero = VAR_UNKNOWN)
        let mut def_rettv = [0u8; 16];
        let def_rettv_ptr = def_rettv.as_mut_ptr().cast::<c_void>();

        let ai = i - args_len;
        let name: *const c_char;
        let namelen: usize;

        if ai < 0 {
            // named argument
            name = unsafe { nvim_ufunc_get_arg(fp, i) };
            if islambda {
                addlocal = true;
            }
            // isdefault: ai + def_args_len >= 0 && i >= argcount
            let def_args_len = unsafe { nvim_ufunc_get_def_args_len(fp) };
            isdefault = (ai + def_args_len >= 0) && (i >= argcount);
            if isdefault {
                // default expression eval
                unsafe { nvim_tv_set_number(def_rettv_ptr, -1) };
                let default_expr_orig = unsafe { nvim_ufunc_get_def_arg(fp, ai + def_args_len) };
                // eval1 takes *mut *mut c_char; we need a mutable copy of the pointer
                let mut default_expr = default_expr_orig as *mut c_char;
                if unsafe {
                    eval1(
                        std::ptr::addr_of_mut!(default_expr),
                        def_rettv_ptr,
                        std::ptr::addr_of_mut!(EVALARG_EVALUATE),
                    )
                } == FAIL
                {
                    default_arg_err = true;
                    break;
                }
            }
            // namelen = strlen(name)
            namelen = unsafe {
                let mut n = 0usize;
                while *name.add(n) != 0 {
                    n += 1;
                }
                n
            };
        } else {
            // vararg: FC_NOARGS → break
            if (uf_flags & fc_noargs_flag) != 0 {
                break;
            }
            // name = "%d" % (ai+1) into numbuf
            let written = unsafe {
                nvim_format_vararg_name(numbuf.as_mut_ptr().cast::<c_char>(), numbuflen, ai + 1)
            };
            name = numbuf.as_ptr().cast::<c_char>();
            namelen = written as usize;
        }

        // Allocate/select dictitem slot
        let v: *mut c_void;
        let flags = unsafe { nvim_di_flags_ro_fix() };
        if fixvar_idx < fixvar_cnt && (namelen as c_int) <= var_short_len {
            v = unsafe { nvim_fc_fixvar_item(fc, fixvar_idx) };
            fixvar_idx += 1;
            unsafe { nvim_dictitem_set_flags(v, flags) };
            unsafe { nvim_dictitem_strcpy_key(v, name) };
        } else {
            v = unsafe { tv_dict_item_alloc_len(name, namelen) };
            unsafe { nvim_dictitem_or_flags_ro_fix(v) };
        }

        // v->di_tv = isdefault ? def_rettv : argvars[i]
        if isdefault {
            unsafe { nvim_dictitem_copy_tv(v, def_rettv_ptr.cast_const()) };
        } else {
            let argvars_i = unsafe { argvars.cast::<u8>().add(i as usize * 16).cast::<c_void>() };
            unsafe { nvim_dictitem_copy_tv(v, argvars_i.cast_const()) };
        }
        unsafe { nvim_dictitem_tv_set_lock(v, var_fixed) };

        // Track for tv_clear in teardown
        if isdefault {
            let di_tv = unsafe { nvim_dictitem_tv_ptr(v) };
            if tv_to_free_len < tv_to_free.len() {
                tv_to_free[tv_to_free_len] = di_tv;
                tv_to_free_len += 1;
            }
        }

        // Add to l: or a: hashtab
        let key = unsafe { nvim_dictitem_key_ptr(v) };
        if addlocal {
            // lambda: tv_copy(&v->di_tv, &v->di_tv) (self-copy for closure)
            let di_tv = unsafe { nvim_dictitem_tv_ptr(v) };
            unsafe { tv_copy(di_tv.cast_const(), di_tv) };
            unsafe { hash_add(l_vars_ht, key) };
        } else {
            unsafe { hash_add(l_avars_ht, key) };
        }

        // If vararg (ai>=0) and within MAX_FUNC_ARGS, append to a:000 list
        if ai >= 0 && ai < MAX_FUNC_ARGS as c_int {
            let li_tv = unsafe { nvim_fc_listitem_tv(fc, ai) };
            let argvars_i = unsafe { argvars.cast::<u8>().add(i as usize * 16).cast::<c_void>() };
            // *TV_LIST_ITEM_TV(li) = argvars[i]
            unsafe {
                std::ptr::copy_nonoverlapping(argvars_i.cast::<u8>(), li_tv.cast::<u8>(), 16)
            };
            // v_lock = VAR_FIXED (at offset 4 in typval_T: i32 v_type, i32 v_lock)
            unsafe { *(li_tv.cast::<u8>().add(4) as *mut i32) = var_fixed };
            // tv_list_append(&fc->fc_l_varlist, li)
            // We need the listitem_T* not just the tv_ptr.
            // nvim_fc_listitem_tv gives us &li->li_tv; we need &li (the listitem).
            // li_tv IS &li->li_tv where li_tv == &fc->fc_l_listitems[ai].li_tv.
            // li_tv is the first field of listitem_T (or not?). Let me check.
            // Actually li_tv points INSIDE the listitem. We need the listitem pointer.
            // tv_list_append takes listitem_T*. We need nvim_fc_listitem(fc, ai).
            // We only have nvim_fc_listitem_tv. We need to add nvim_fc_listitem accessor.
            // Use the raw pointer arithmetic: listitem_T has {listitem_T* li_next,
            // listitem_T* li_prev, typval_T li_tv}. li_tv is at offset 2*sizeof(ptr).
            // Actually the layout might have li_tv first. Let's add a C accessor.
            // For now use nvim_fc_get_listitem which we'll add, or use a shim.
            // We'll call nvim_fc_listitem_append helper (see below).
            unsafe { nvim_fc_listitem_append(fc, ai, l_varlist) };
        }

        i += 1;
    }

    // --- RedrawingDisabled++ (checklist #9) ---
    unsafe { nvim_redrawing_disabled_incr() };

    // --- Sandbox (checklist #10) ---
    let fc_sandbox_flag: c_int = 0x40; // FC_SANDBOX
    let using_sandbox = (uf_flags & fc_sandbox_flag) != 0;
    if using_sandbox {
        unsafe { nvim_sandbox_incr() };
    }

    // --- estack_push_ufunc (checklist #11) ---
    unsafe { estack_push_ufunc(fp, 1) };

    // --- Verbose entry (checklist #12) ---
    let p_verbose = unsafe { nvim_get_p_verbose() };
    if p_verbose >= 12 {
        unsafe { nvim_no_wait_return_incr() };
        unsafe { verbose_enter_scroll() };
        let sourcing_name = unsafe { nvim_sourcing_name_get() };
        unsafe { smsg(0, c"calling %s".as_ptr(), sourcing_name) };
        if p_verbose >= 14 {
            unsafe { msg_puts(c"(".as_ptr()) };
            let mut vi = 0i32;
            while vi < argcount {
                if vi > 0 {
                    unsafe { msg_puts(c", ".as_ptr()) };
                }
                let argv_i = unsafe { argvars.cast::<u8>().add(vi as usize * 16).cast::<c_void>() };
                let var_number = unsafe { nvim_var_number() };
                if unsafe { nvim_tv_get_type(argv_i.cast_const()) } == var_number {
                    #[allow(clippy::cast_possible_truncation)]
                    let n = unsafe { nvim_tv_get_number(argv_i.cast_const()) } as c_int;
                    unsafe { msg_outnum(n) };
                } else {
                    // emsg_off++; encode_tv2string; trunc if needed; msg_puts; xfree; emsg_off--
                    unsafe { nvim_emsg_off_inc() };
                    let tofree =
                        unsafe { encode_tv2string(argv_i.cast_const(), std::ptr::null_mut()) };
                    unsafe { nvim_emsg_off_dec() };
                    if !tofree.is_null() {
                        let mut s = tofree;
                        let mut buf = [0u8; MSG_BUF_LEN];
                        if unsafe { vim_strsize(s.cast_const()) } > MSG_BUF_CLEN {
                            unsafe {
                                trunc_string(
                                    s.cast_const(),
                                    buf.as_mut_ptr().cast::<c_char>(),
                                    MSG_BUF_CLEN,
                                    MSG_BUF_LEN,
                                )
                            };
                            s = buf.as_mut_ptr().cast::<c_char>();
                        }
                        unsafe { msg_puts(s.cast_const()) };
                        unsafe { xfree(tofree.cast::<c_void>()) };
                    }
                }
                vi += 1;
            }
            unsafe { msg_puts(c")".as_ptr()) };
        }
        unsafe { msg_puts(c"\n".as_ptr()) };
        unsafe { verbose_leave_scroll() };
        unsafe { nvim_no_wait_return_decr() };
    }

    // --- Profiling setup (checklist #13) ---
    let do_profiling_yes = unsafe { do_profiling } == PROF_YES;
    let uf_profiling = unsafe { nvim_ufunc_get_profiling(fp) } != 0;
    let fp_name_cstr = unsafe { nvim_ufunc_get_name_ptr(fp) }.cast_const();

    let func_not_yet_profiling_but_should = do_profiling_yes
        && !uf_profiling
        && unsafe { has_profiling(false, fp_name_cstr, std::ptr::null_mut()) };

    let mut started_profiling = false;
    if func_not_yet_profiling_but_should {
        started_profiling = true;
        unsafe { func_do_profile(fp) };
    }

    let caller = unsafe { nvim_fc_get_caller(fc) };
    let caller_profiling =
        !caller.is_null() && unsafe { nvim_ufunc_get_profiling(nvim_fc_get_func(caller)) } != 0;
    let uf_profiling2 = unsafe { nvim_ufunc_get_profiling(fp) } != 0; // re-read after func_do_profile

    let func_or_func_caller_profiling = do_profiling_yes && (uf_profiling2 || caller_profiling);

    let mut call_start: u64 = 0;
    let mut wait_start: u64 = 0;
    if func_or_func_caller_profiling {
        let count = unsafe { nvim_ufunc_get_tm_count(fp) };
        unsafe { nvim_ufunc_set_tm_count(fp, count + 1) };
        call_start = unsafe { profile_start() };
        unsafe { nvim_ufunc_set_tm_children(fp, profile_zero()) };
    }
    if do_profiling_yes {
        unsafe { script_prof_save(std::ptr::addr_of_mut!(wait_start)) };
    }

    // --- sctx + did_emsg save (checklist #14) ---
    let save_current_sctx = unsafe { nvim_get_current_sctx() };
    let script_ctx = unsafe { nvim_ufunc_get_script_ctx(fp) };
    unsafe { nvim_set_current_sctx(script_ctx) };
    let save_did_emsg = unsafe { nvim_get_did_emsg() };
    unsafe { nvim_set_did_emsg(false) };

    // --- Body dispatch (checklist #15) ---
    if default_arg_err && ((uf_flags & FC_ABORT != 0) || unsafe { nvim_get_trylevel() } > 0) {
        unsafe { nvim_set_did_emsg(true) };
    } else if islambda {
        let p_orig = unsafe { nvim_ufunc_lambda_body_ptr(fp) };
        let mut p = p_orig;
        unsafe { nvim_ex_nesting_level_incr() };
        unsafe {
            eval1(
                std::ptr::addr_of_mut!(p),
                rettv,
                std::ptr::addr_of_mut!(EVALARG_EVALUATE),
            )
        };
        unsafe { nvim_ex_nesting_level_decr() };
    } else {
        unsafe {
            do_cmdline(
                std::ptr::null_mut(),
                Some(rs_get_func_line),
                fc,
                DOCMD_NOWAIT | DOCMD_VERBOSE | DOCMD_REPEAT,
            )
        };
    }

    // --- Defer (checklist #16) ---
    // Note: current_funccal, NOT fc
    let cur_funccal = unsafe { get_current_funccal() };
    unsafe { rs_handle_defer_one(cur_funccal) };

    // --- RedrawingDisabled-- (checklist #17) ---
    unsafe { nvim_redrawing_disabled_decr() };

    // --- Abort rettv (checklist #18) ---
    let fc_abort_flag: c_int = FC_ABORT;
    let did_emsg_now = unsafe { nvim_get_did_emsg() };
    let rettv_type = unsafe { nvim_tv_get_type(rettv.cast_const()) };
    let var_unknown: c_int = 0;
    if (did_emsg_now && (uf_flags & fc_abort_flag != 0)) || rettv_type == var_unknown {
        unsafe { tv_clear(rettv) };
        unsafe { nvim_tv_set_number(rettv, -1) };
    }

    // --- Profiling aggregation (checklist #19) ---
    if func_or_func_caller_profiling {
        call_start = unsafe { profile_end(call_start) };
        call_start = unsafe { profile_sub_wait(wait_start, call_start) };
        let total = unsafe { nvim_ufunc_get_tm_total(fp) };
        unsafe { nvim_ufunc_set_tm_total(fp, profile_add(total, call_start)) };
        let self_tm = unsafe { nvim_ufunc_get_tm_self(fp) };
        let children = unsafe { nvim_ufunc_get_tm_children(fp) };
        unsafe { nvim_ufunc_set_tm_self(fp, profile_self(self_tm, call_start, children)) };
        if !caller.is_null() && caller_profiling {
            let caller_func = unsafe { nvim_fc_get_func(caller) };
            let c_children = unsafe { nvim_ufunc_get_tm_children(caller_func) };
            unsafe { nvim_ufunc_set_tm_children(caller_func, profile_add(c_children, call_start)) };
            let c_tml_children = unsafe { nvim_ufunc_get_tml_children(caller_func) };
            unsafe {
                nvim_ufunc_set_tml_children(caller_func, profile_add(c_tml_children, call_start))
            };
        }
        if started_profiling {
            unsafe { nvim_ufunc_set_profiling(fp, 0) };
        }
    }

    // --- Verbose exit (checklist #20) ---
    let p_verbose2 = unsafe { nvim_get_p_verbose() };
    if p_verbose2 >= 12 {
        unsafe { nvim_no_wait_return_incr() };
        unsafe { verbose_enter_scroll() };
        let sourcing_name = unsafe { nvim_sourcing_name_get() };
        let fc_rettv = unsafe { nvim_fc_get_rettv(fc) };
        if unsafe { nvim_aborting() } {
            unsafe { smsg(0, c"%s aborted".as_ptr(), sourcing_name) };
        } else if !fc_rettv.is_null()
            && unsafe { nvim_tv_get_type(fc_rettv.cast_const()) } == unsafe { nvim_var_number() }
        {
            let n = unsafe { nvim_tv_get_number(fc_rettv.cast_const()) };
            unsafe { smsg(0, c"%s returning #%ld".as_ptr(), sourcing_name, n as i64) };
        } else {
            let mut buf = [0u8; MSG_BUF_LEN];
            unsafe { nvim_emsg_off_inc() };
            let tofree = if fc_rettv.is_null() {
                std::ptr::null_mut()
            } else {
                unsafe { encode_tv2string(fc_rettv.cast_const(), std::ptr::null_mut()) }
            };
            unsafe { nvim_emsg_off_dec() };
            if !tofree.is_null() {
                let mut s = tofree;
                if unsafe { vim_strsize(s.cast_const()) } > MSG_BUF_CLEN {
                    unsafe {
                        trunc_string(
                            s.cast_const(),
                            buf.as_mut_ptr().cast::<c_char>(),
                            MSG_BUF_CLEN,
                            MSG_BUF_LEN,
                        )
                    };
                    s = buf.as_mut_ptr().cast::<c_char>();
                }
                unsafe { smsg(0, c"%s returning %s".as_ptr(), sourcing_name, s) };
                unsafe { xfree(tofree.cast::<c_void>()) };
            }
        }
        unsafe { msg_puts(c"\n".as_ptr()) };
        unsafe { verbose_leave_scroll() };
        unsafe { nvim_no_wait_return_decr() };
    }

    // --- estack_pop; sctx restore; script_prof_restore; sandbox-- (checklist #21) ---
    unsafe { estack_pop() };
    unsafe { nvim_set_current_sctx(save_current_sctx) };
    if do_profiling_yes {
        unsafe { script_prof_restore(std::ptr::addr_of!(wait_start)) };
    }
    if using_sandbox {
        unsafe { nvim_sandbox_decr() };
    }

    // --- Verbose continuing (checklist #22) ---
    let p_verbose3 = unsafe { nvim_get_p_verbose() };
    if p_verbose3 >= 12 {
        let sourcing_name3 = unsafe { nvim_sourcing_name_get() };
        if !sourcing_name3.is_null() {
            unsafe { nvim_no_wait_return_incr() };
            unsafe { verbose_enter_scroll() };
            unsafe { smsg(0, c"continuing in %s".as_ptr(), sourcing_name3) };
            unsafe { msg_puts(c"\n".as_ptr()) };
            unsafe { verbose_leave_scroll() };
            unsafe { nvim_no_wait_return_decr() };
        }
    }

    // --- Teardown tail (checklist #23) ---
    let did_emsg_final = unsafe { nvim_get_did_emsg() };
    unsafe { nvim_set_did_emsg(did_emsg_final | save_did_emsg) };
    CUF_DEPTH.store(CUF_DEPTH.load(Ordering::Relaxed) - 1, Ordering::Relaxed);

    // Free default-arg tvs
    for k in 0..tv_to_free_len {
        unsafe { tv_clear(tv_to_free[k]) };
    }

    unsafe { rs_cleanup_function_call(fc) };

    // If fp->uf_calls goes to 0 and uf_refcount <= 0, free the function
    // Matches C: if (--fp->uf_calls <= 0 && fp->uf_refcount <= 0)
    let calls = unsafe { nvim_ufunc_get_calls(fp) } - 1;
    unsafe { nvim_ufunc_set_calls(fp, calls) };
    if calls <= 0 && unsafe { nvim_ufunc_get_refcount(fp) } <= 0 {
        unsafe { rs_func_clear_free(fp, 0) };
    }

    // Restore redo buffer and search patterns
    if did_save_redo {
        unsafe { restoreRedobuff(save_redo) };
    }
    unsafe { xfree(save_redo) };
    unsafe { restore_search_patterns() };
}
