extern "C" {
    pub type terminal;
    pub type regprog;
    pub type undo_object;
    pub type qf_info_S;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn abort() -> !;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memmove(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memcmp(
        __s1: *const ::core::ffi::c_void,
        __s2: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn memchr(
        __s: *const ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strcpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strchr(__s: *const ::core::ffi::c_char, __c: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn strstr(
        __haystack: *const ::core::ffi::c_char,
        __needle: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn xmallocz(size: size_t) -> *mut ::core::ffi::c_void;
    fn xmemdupz(data: *const ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
    fn xmemcpyz(
        dst: *mut ::core::ffi::c_void,
        src: *const ::core::ffi::c_void,
        len: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn xstrlcpy(
        dst: *mut ::core::ffi::c_char,
        src: *const ::core::ffi::c_char,
        dsize: size_t,
    ) -> size_t;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xmemrchr(
        src: *const ::core::ffi::c_void,
        c: uint8_t,
        len: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn apply_autocmds(
        event: event_T,
        fname: *mut ::core::ffi::c_char,
        fname_io: *mut ::core::ffi::c_char,
        force: bool,
        buf: *mut buf_T,
    ) -> bool;
    static mut p_ic: ::core::ffi::c_int;
    static mut p_mfd: OptInt;
    static mut p_verbose: OptInt;
    fn xstrnsave(string: *const ::core::ffi::c_char, len: size_t) -> *mut ::core::ffi::c_char;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn concat_str(
        str1: *const ::core::ffi::c_char,
        str2: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn vim_strsize(s: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn skiptowhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn getdigits(pp: *mut *mut ::core::ffi::c_char, strict: bool, def: intmax_t) -> intmax_t;
    fn dbg_find_breakpoint(
        file: bool,
        fname: *mut ::core::ffi::c_char,
        after: linenr_T,
    ) -> linenr_T;
    fn has_profiling(file: bool, fname: *mut ::core::ffi::c_char, fp: *mut bool) -> bool;
    fn dbg_breakpoint(name: *mut ::core::ffi::c_char, lnum: linenr_T);
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static e_invarg2: [::core::ffi::c_char; 0];
    static e_invexpr2: [::core::ffi::c_char; 0];
    static e_invrange: [::core::ffi::c_char; 0];
    static e_toomanyarg: [::core::ffi::c_char; 0];
    static e_toofewarg: [::core::ffi::c_char; 0];
    static e_dictkey: [::core::ffi::c_char; 0];
    static e_trailing_arg: [::core::ffi::c_char; 0];
    static e_usingsid: [::core::ffi::c_char; 0];
    static e_missingparen: [::core::ffi::c_char; 0];
    static e_unknown_function_str: [::core::ffi::c_char; 0];
    static e_str_not_inside_function: [::core::ffi::c_char; 0];
    static e_not_callable_type_str: [::core::ffi::c_char; 0];
    static mut eval_lavars_used: *mut bool;
    static mut EVALARG_EVALUATE: evalarg_T;
    fn fill_evalarg_from_eap(evalarg: *mut evalarg_T, eap: *mut exarg_T, skip: bool);
    fn skip_expr(pp: *mut *mut ::core::ffi::c_char, evalarg: *mut evalarg_T) -> ::core::ffi::c_int;
    fn get_lval(
        name: *mut ::core::ffi::c_char,
        rettv: *mut typval_T,
        lp: *mut lval_T,
        unlet: bool,
        skip: bool,
        flags: ::core::ffi::c_int,
        fne_flags: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn clear_lval(lp: *mut lval_T);
    fn clear_evalarg(evalarg: *mut evalarg_T, eap: *mut exarg_T);
    fn eval0(
        arg: *mut ::core::ffi::c_char,
        rettv: *mut typval_T,
        eap: *mut exarg_T,
        evalarg: *mut evalarg_T,
    ) -> ::core::ffi::c_int;
    fn eval1(
        arg: *mut *mut ::core::ffi::c_char,
        rettv: *mut typval_T,
        evalarg: *mut evalarg_T,
    ) -> ::core::ffi::c_int;
    fn partial_name(pt: *mut partial_T) -> *mut ::core::ffi::c_char;
    fn partial_unref(pt: *mut partial_T);
    fn garbage_collect(testing: bool) -> bool;
    fn set_ref_in_ht(
        ht: *mut hashtab_T,
        copyID: ::core::ffi::c_int,
        list_stack: *mut *mut list_stack_T,
    ) -> bool;
    fn set_ref_in_list_items(
        l: *mut list_T,
        copyID: ::core::ffi::c_int,
        ht_stack: *mut *mut ht_stack_T,
    ) -> bool;
    fn set_ref_in_item(
        tv: *mut typval_T,
        copyID: ::core::ffi::c_int,
        ht_stack: *mut *mut ht_stack_T,
        list_stack: *mut *mut list_stack_T,
    ) -> bool;
    fn callback_call(
        callback: *mut Callback,
        argcount_in: ::core::ffi::c_int,
        argvars_in: *mut typval_T,
        rettv: *mut typval_T,
    ) -> bool;
    fn get_id_len(arg: *mut *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn find_name_end(
        arg: *const ::core::ffi::c_char,
        expr_start: *mut *const ::core::ffi::c_char,
        expr_end: *mut *const ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
    ) -> *const ::core::ffi::c_char;
    fn eval_isnamec(c: ::core::ffi::c_int) -> bool;
    fn eval_isnamec1(c: ::core::ffi::c_int) -> bool;
    fn is_luafunc(partial: *mut partial_T) -> bool;
    fn check_luafunc_name(str: *const ::core::ffi::c_char, paren: bool) -> ::core::ffi::c_int;
    fn handle_subscript(
        arg: *mut *const ::core::ffi::c_char,
        rettv: *mut typval_T,
        evalarg: *mut evalarg_T,
        verbose: bool,
    ) -> ::core::ffi::c_int;
    fn last_set_msg(script_ctx: sctx_T);
    fn encode_tv2string(tv: *mut typval_T, len: *mut size_t) -> *mut ::core::ffi::c_char;
    fn encode_tv2echo(tv: *mut typval_T, len: *mut size_t) -> *mut ::core::ffi::c_char;
    fn find_internal_func(name: *const ::core::ffi::c_char) -> *const EvalFuncDef;
    fn check_internal_func(
        fdef: *const EvalFuncDef,
        argcount: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn call_internal_func(
        fname: *const ::core::ffi::c_char,
        argcount: ::core::ffi::c_int,
        argvars: *mut typval_T,
        rettv: *mut typval_T,
    ) -> ::core::ffi::c_int;
    fn call_internal_method(
        fname: *const ::core::ffi::c_char,
        argcount: ::core::ffi::c_int,
        argvars: *mut typval_T,
        rettv: *mut typval_T,
        basetv: *mut typval_T,
    ) -> ::core::ffi::c_int;
    static mut hash_removed: ::core::ffi::c_char;
    fn hash_init(ht: *mut hashtab_T);
    fn hash_find(ht: *const hashtab_T, key: *const ::core::ffi::c_char) -> *mut hashitem_T;
    fn hash_find_len(
        ht: *const hashtab_T,
        key: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut hashitem_T;
    fn hash_add(ht: *mut hashtab_T, key: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn hash_remove(ht: *mut hashtab_T, hi: *mut hashitem_T);
    fn trunc_string(
        s: *const ::core::ffi::c_char,
        buf: *mut ::core::ffi::c_char,
        room_in: ::core::ffi::c_int,
        buflen: ::core::ffi::c_int,
    );
    fn smsg(hl_id: ::core::ffi::c_int, s: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn iemsg(s: *const ::core::ffi::c_char);
    fn internal_error(where_0: *const ::core::ffi::c_char);
    fn msg_ext_set_kind(msg_kind: *const ::core::ffi::c_char);
    fn msg_start();
    fn msg_putchar(c: ::core::ffi::c_int);
    fn msg_outnum(n: ::core::ffi::c_int);
    fn msg_prt_line(s: *const ::core::ffi::c_char, list: bool);
    fn msg_puts(s: *const ::core::ffi::c_char);
    fn message_filtered(msg: *const ::core::ffi::c_char) -> bool;
    fn msg_clr_eos();
    fn verbose_enter_scroll();
    fn verbose_leave_scroll();
    fn swmsg(hl: bool, fmt: *const ::core::ffi::c_char, ...);
    fn tv_list_init_static(l: *mut list_T);
    fn tv_list_append(l: *mut list_T, item: *mut listitem_T);
    fn tv_dict_item_alloc_len(key: *const ::core::ffi::c_char, key_len: size_t) -> *mut dictitem_T;
    fn tv_dict_item_alloc(key: *const ::core::ffi::c_char) -> *mut dictitem_T;
    fn tv_dict_item_remove(dict: *mut dict_T, item: *mut dictitem_T);
    fn tv_dict_unref(d: *mut dict_T);
    fn tv_dict_add(d: *mut dict_T, item: *mut dictitem_T) -> ::core::ffi::c_int;
    fn tv_clear(tv: *mut typval_T);
    fn tv_copy(from: *const typval_T, to: *mut typval_T);
    fn value_check_lock(
        lock: VarLockStatus,
        name: *const ::core::ffi::c_char,
        name_len: size_t,
    ) -> bool;
    fn tv_get_number_chk(tv: *const typval_T, ret_error: *mut bool) -> varnumber_T;
    fn skip_var_list(
        arg: *const ::core::ffi::c_char,
        var_count: *mut ::core::ffi::c_int,
        semicolon: *mut ::core::ffi::c_int,
        silent: bool,
    ) -> *const ::core::ffi::c_char;
    fn list_hashtable_vars(
        ht: *mut hashtab_T,
        prefix: *const ::core::ffi::c_char,
        empty: ::core::ffi::c_int,
        first: *mut ::core::ffi::c_int,
    );
    fn get_vim_var_nr(idx: VimVarIndex) -> varnumber_T;
    fn find_var(
        name: *const ::core::ffi::c_char,
        name_len: size_t,
        htp: *mut *mut hashtab_T,
        no_autoload: ::core::ffi::c_int,
    ) -> *mut dictitem_T;
    fn find_var_in_ht(
        ht: *mut hashtab_T,
        htname: ::core::ffi::c_int,
        varname: *const ::core::ffi::c_char,
        varname_len: size_t,
        no_autoload: ::core::ffi::c_int,
    ) -> *mut dictitem_T;
    fn find_var_ht(
        name: *const ::core::ffi::c_char,
        name_len: size_t,
        varname: *mut *const ::core::ffi::c_char,
    ) -> *mut hashtab_T;
    fn init_var_dict(dict: *mut dict_T, dict_var: *mut ScopeDictDictItem, scope: ScopeType);
    fn vars_clear(ht: *mut hashtab_T);
    fn vars_clear_ext(ht: *mut hashtab_T, free_val: bool);
    fn aborting() -> bool;
    fn update_force_abort();
    fn aborted_in_try() -> bool;
    fn exception_state_save(estate: *mut exception_state_T);
    fn exception_state_restore(estate: *mut exception_state_T);
    fn exception_state_clear();
    fn report_make_pending(pending: ::core::ffi::c_int, value: *mut ::core::ffi::c_void);
    fn cleanup_conditionals(
        cstack: *mut cstack_T,
        searched_cond: ::core::ffi::c_int,
        inclusive: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn do_cmdline(
        cmdline: *mut ::core::ffi::c_char,
        fgetline: LineGetter,
        cookie: *mut ::core::ffi::c_void,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn checkforcmd(
        pp: *mut *mut ::core::ffi::c_char,
        cmd: *const ::core::ffi::c_char,
        len: ::core::ffi::c_int,
    ) -> bool;
    fn skip_range(
        cmd: *const ::core::ffi::c_char,
        ctx: *mut ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn ends_excmd(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn check_nextcmd(p: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn getcmdline(
        firstc: ::core::ffi::c_int,
        count: ::core::ffi::c_int,
        indent: ::core::ffi::c_int,
        do_concat: bool,
    ) -> *mut ::core::ffi::c_char;
    fn ui_ext_cmdline_block_append(indent: size_t, line: *const ::core::ffi::c_char);
    fn ui_ext_cmdline_block_leave();
    fn ga_clear(gap: *mut garray_T);
    fn ga_clear_strings(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    fn ga_append_via_ptr(gap: *mut garray_T, item_size: size_t) -> *mut ::core::ffi::c_void;
    fn saveRedobuff(save_redo: *mut save_redo_T);
    fn restoreRedobuff(save_redo: *mut save_redo_T);
    fn ins_compl_active() -> bool;
    fn api_free_luaref(ref_0: LuaRef);
    fn nlua_typval_call(
        str: *const ::core::ffi::c_char,
        len: size_t,
        args: *mut typval_T,
        argcount: ::core::ffi::c_int,
        ret_tv: *mut typval_T,
    );
    fn typval_exec_lua_callable(
        lua_cb: LuaRef,
        argcount: ::core::ffi::c_int,
        argvars: *mut typval_T,
        rettv: *mut typval_T,
    ) -> ::core::ffi::c_int;
    fn nlua_set_sctx(current: *mut sctx_T);
    fn mb_strnicmp(
        s1: *const ::core::ffi::c_char,
        s2: *const ::core::ffi::c_char,
        nn: size_t,
    ) -> ::core::ffi::c_int;
    fn line_breakcheck();
    fn path_fnamecmp(
        fname1: *const ::core::ffi::c_char,
        fname2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn profile_start() -> proftime_T;
    fn profile_end(tm: proftime_T) -> proftime_T;
    fn profile_zero() -> proftime_T;
    fn profile_add(tm1: proftime_T, tm2: proftime_T) -> proftime_T;
    fn profile_self(self_0: proftime_T, total: proftime_T, children: proftime_T) -> proftime_T;
    fn profile_sub_wait(tm: proftime_T, tma: proftime_T) -> proftime_T;
    fn prof_def_func() -> bool;
    fn func_do_profile(fp: *mut ufunc_T);
    fn func_line_start(cookie: *mut ::core::ffi::c_void);
    fn func_line_end(cookie: *mut ::core::ffi::c_void);
    fn script_prof_save(tm: *mut proftime_T);
    fn script_prof_restore(tm: *const proftime_T);
    fn skip_regexp(
        startp: *mut ::core::ffi::c_char,
        delim: ::core::ffi::c_int,
        magic: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
    static mut exestack: garray_T;
    static mut script_items: garray_T;
    fn estack_push_ufunc(ufunc: *mut ufunc_T, lnum: linenr_T);
    fn estack_pop();
    fn get_sourced_lnum(fgetline: LineGetter, cookie: *mut ::core::ffi::c_void) -> linenr_T;
    fn autoload_name(
        name: *const ::core::ffi::c_char,
        name_len: size_t,
    ) -> *mut ::core::ffi::c_char;
    fn script_autoload(name: *const ::core::ffi::c_char, name_len: size_t, reload: bool) -> bool;
    fn save_search_patterns();
    fn restore_search_patterns();
    fn ui_has(ext: UIExtension) -> bool;
    static mut Rows: ::core::ffi::c_int;
    static mut cmdline_row: ::core::ffi::c_int;
    static mut msg_row: ::core::ffi::c_int;
    static mut msg_scroll: ::core::ffi::c_int;
    static mut emsg_off: ::core::ffi::c_int;
    static mut emsg_skip: ::core::ffi::c_int;
    static mut emsg_severe: bool;
    static mut did_emsg: ::core::ffi::c_int;
    static mut no_wait_return: ::core::ffi::c_int;
    static mut need_wait_return: bool;
    static mut lines_left: ::core::ffi::c_int;
    static mut ex_nesting_level: ::core::ffi::c_int;
    static mut debug_tick: ::core::ffi::c_int;
    static mut debug_backtrace_level: ::core::ffi::c_int;
    static mut do_profiling: ::core::ffi::c_int;
    static mut did_throw: bool;
    static mut trylevel: ::core::ffi::c_int;
    static mut want_garbage_collect: bool;
    static mut current_sctx: sctx_T;
    static mut curwin: *mut win_T;
    static mut curbuf: *mut buf_T;
    static mut sandbox: ::core::ffi::c_int;
    static mut IObuff: [::core::ffi::c_char; 1025];
    static mut RedrawingDisabled: ::core::ffi::c_int;
    static mut KeyTyped: bool;
    static mut got_int: bool;
}
pub type __time_t = ::core::ffi::c_long;
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const _ISalnum: C2Rust_Unnamed = 8;
pub const _ISpunct: C2Rust_Unnamed = 4;
pub const _IScntrl: C2Rust_Unnamed = 2;
pub const _ISblank: C2Rust_Unnamed = 1;
pub const _ISgraph: C2Rust_Unnamed = 32768;
pub const _ISprint: C2Rust_Unnamed = 16384;
pub const _ISspace: C2Rust_Unnamed = 8192;
pub const _ISxdigit: C2Rust_Unnamed = 4096;
pub const _ISdigit: C2Rust_Unnamed = 2048;
pub const _ISalpha: C2Rust_Unnamed = 1024;
pub const _ISlower: C2Rust_Unnamed = 512;
pub const _ISupper: C2Rust_Unnamed = 256;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type intmax_t = ::libc::intmax_t;
pub type size_t = usize;
pub type time_t = __time_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Arena {
    pub cur_blk: *mut ::core::ffi::c_char,
    pub pos: size_t,
    pub size: size_t,
}
pub type schar_T = uint32_t;
pub type sattr_T = int32_t;
pub type handle_T = ::core::ffi::c_int;
pub type LuaRef = ::core::ffi::c_int;
pub type float_T = ::core::ffi::c_double;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MsgpackRpcRequestHandler {
    pub name: *const ::core::ffi::c_char,
    pub fn_0: ApiDispatchWrapper,
    pub fast: bool,
    pub ret_alloc: bool,
}
pub type ApiDispatchWrapper =
    Option<unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Error {
    pub type_0: ErrorType,
    pub msg: *mut ::core::ffi::c_char,
}
pub type ErrorType = ::core::ffi::c_int;
pub const kErrorTypeValidation: ErrorType = 1;
pub const kErrorTypeException: ErrorType = 0;
pub const kErrorTypeNone: ErrorType = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Array {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut Object,
}
pub type Object = object;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct object {
    pub type_0: ObjectType,
    pub data: C2Rust_Unnamed_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_0 {
    pub boolean: Boolean,
    pub integer: Integer,
    pub floating: Float,
    pub string: String_0,
    pub array: Array,
    pub dict: Dict,
    pub luaref: LuaRef,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Dict {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut KeyValuePair,
}
pub type KeyValuePair = key_value_pair;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct key_value_pair {
    pub key: String_0,
    pub value: Object,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct String_0 {
    pub data: *mut ::core::ffi::c_char,
    pub size: size_t,
}
pub type Float = ::core::ffi::c_double;
pub type Integer = int64_t;
pub type Boolean = bool;
pub type ObjectType = ::core::ffi::c_uint;
pub const kObjectTypeTabpage: ObjectType = 10;
pub const kObjectTypeWindow: ObjectType = 9;
pub const kObjectTypeBuffer: ObjectType = 8;
pub const kObjectTypeLuaRef: ObjectType = 7;
pub const kObjectTypeDict: ObjectType = 6;
pub const kObjectTypeArray: ObjectType = 5;
pub const kObjectTypeString: ObjectType = 4;
pub const kObjectTypeFloat: ObjectType = 3;
pub const kObjectTypeInteger: ObjectType = 2;
pub const kObjectTypeBoolean: ObjectType = 1;
pub const kObjectTypeNil: ObjectType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub union EvalFuncData {
    pub float_func: Option<unsafe extern "C" fn(float_T) -> float_T>,
    pub api_handler: *const MsgpackRpcRequestHandler,
    pub null: *mut ::core::ffi::c_void,
}
pub type proftime_T = uint64_t;
pub type OptInt = int64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct file_buffer {
    pub handle: handle_T,
    pub b_ml: memline_T,
    pub b_next: *mut buf_T,
    pub b_prev: *mut buf_T,
    pub b_nwindows: ::core::ffi::c_int,
    pub b_flags: ::core::ffi::c_int,
    pub b_locked: ::core::ffi::c_int,
    pub b_locked_split: ::core::ffi::c_int,
    pub b_ro_locked: ::core::ffi::c_int,
    pub b_ffname: *mut ::core::ffi::c_char,
    pub b_sfname: *mut ::core::ffi::c_char,
    pub b_fname: *mut ::core::ffi::c_char,
    pub file_id_valid: bool,
    pub file_id: FileID,
    pub b_changed: ::core::ffi::c_int,
    pub b_changed_invalid: bool,
    pub changedtick_di: ChangedtickDictItem,
    pub b_last_changedtick: varnumber_T,
    pub b_last_changedtick_i: varnumber_T,
    pub b_last_changedtick_pum: varnumber_T,
    pub b_saving: bool,
    pub b_mod_set: bool,
    pub b_mod_top: linenr_T,
    pub b_mod_bot: linenr_T,
    pub b_mod_xlines: linenr_T,
    pub b_wininfo: C2Rust_Unnamed_12,
    pub b_mod_tick_syn: disptick_T,
    pub b_mod_tick_decor: disptick_T,
    pub b_mtime: int64_t,
    pub b_mtime_ns: int64_t,
    pub b_mtime_read: int64_t,
    pub b_mtime_read_ns: int64_t,
    pub b_orig_size: uint64_t,
    pub b_orig_mode: ::core::ffi::c_int,
    pub b_last_used: time_t,
    pub b_namedm: [fmark_T; 26],
    pub b_visual: visualinfo_T,
    pub b_visual_mode_eval: ::core::ffi::c_int,
    pub b_last_cursor: fmark_T,
    pub b_last_insert: fmark_T,
    pub b_last_change: fmark_T,
    pub b_changelist: [fmark_T; 100],
    pub b_changelistlen: ::core::ffi::c_int,
    pub b_new_change: bool,
    pub b_chartab: [uint64_t; 4],
    pub b_maphash: [*mut mapblock_T; 256],
    pub b_first_abbr: *mut mapblock_T,
    pub b_ucmds: garray_T,
    pub b_op_start: pos_T,
    pub b_op_start_orig: pos_T,
    pub b_op_end: pos_T,
    pub b_marks_read: bool,
    pub b_modified_was_set: bool,
    pub b_did_filetype: bool,
    pub b_keep_filetype: bool,
    pub b_au_did_filetype: bool,
    pub b_u_oldhead: *mut u_header_T,
    pub b_u_newhead: *mut u_header_T,
    pub b_u_curhead: *mut u_header_T,
    pub b_u_numhead: ::core::ffi::c_int,
    pub b_u_synced: bool,
    pub b_u_seq_last: ::core::ffi::c_int,
    pub b_u_save_nr_last: ::core::ffi::c_int,
    pub b_u_seq_cur: ::core::ffi::c_int,
    pub b_u_time_cur: time_t,
    pub b_u_save_nr_cur: ::core::ffi::c_int,
    pub b_u_line_ptr: *mut ::core::ffi::c_char,
    pub b_u_line_lnum: linenr_T,
    pub b_u_line_colnr: colnr_T,
    pub b_scanned: bool,
    pub b_p_iminsert: OptInt,
    pub b_p_imsearch: OptInt,
    pub b_kmap_state: int16_t,
    pub b_kmap_ga: garray_T,
    pub b_p_initialized: bool,
    pub b_p_script_ctx: [sctx_T; 92],
    pub b_p_ac: ::core::ffi::c_int,
    pub b_p_ai: ::core::ffi::c_int,
    pub b_p_ai_nopaste: ::core::ffi::c_int,
    pub b_p_bkc: *mut ::core::ffi::c_char,
    pub b_bkc_flags: ::core::ffi::c_uint,
    pub b_p_ci: ::core::ffi::c_int,
    pub b_p_bin: ::core::ffi::c_int,
    pub b_p_bomb: ::core::ffi::c_int,
    pub b_p_bh: *mut ::core::ffi::c_char,
    pub b_p_bt: *mut ::core::ffi::c_char,
    pub b_p_busy: OptInt,
    pub b_has_qf_entry: ::core::ffi::c_int,
    pub b_p_bl: ::core::ffi::c_int,
    pub b_p_channel: OptInt,
    pub b_p_cin: ::core::ffi::c_int,
    pub b_p_cino: *mut ::core::ffi::c_char,
    pub b_p_cink: *mut ::core::ffi::c_char,
    pub b_p_cinw: *mut ::core::ffi::c_char,
    pub b_p_cinsd: *mut ::core::ffi::c_char,
    pub b_p_com: *mut ::core::ffi::c_char,
    pub b_p_cms: *mut ::core::ffi::c_char,
    pub b_p_cot: *mut ::core::ffi::c_char,
    pub b_cot_flags: ::core::ffi::c_uint,
    pub b_p_cpt: *mut ::core::ffi::c_char,
    pub b_p_cpt_cb: *mut Callback,
    pub b_p_cpt_count: ::core::ffi::c_int,
    pub b_p_cfu: *mut ::core::ffi::c_char,
    pub b_cfu_cb: Callback,
    pub b_p_ofu: *mut ::core::ffi::c_char,
    pub b_ofu_cb: Callback,
    pub b_p_tfu: *mut ::core::ffi::c_char,
    pub b_tfu_cb: Callback,
    pub b_p_ffu: *mut ::core::ffi::c_char,
    pub b_ffu_cb: Callback,
    pub b_p_eof: ::core::ffi::c_int,
    pub b_p_eol: ::core::ffi::c_int,
    pub b_p_fixeol: ::core::ffi::c_int,
    pub b_p_et: ::core::ffi::c_int,
    pub b_p_et_nobin: ::core::ffi::c_int,
    pub b_p_et_nopaste: ::core::ffi::c_int,
    pub b_p_fenc: *mut ::core::ffi::c_char,
    pub b_p_ff: *mut ::core::ffi::c_char,
    pub b_p_ft: *mut ::core::ffi::c_char,
    pub b_p_fo: *mut ::core::ffi::c_char,
    pub b_p_flp: *mut ::core::ffi::c_char,
    pub b_p_inf: ::core::ffi::c_int,
    pub b_p_isk: *mut ::core::ffi::c_char,
    pub b_p_def: *mut ::core::ffi::c_char,
    pub b_p_inc: *mut ::core::ffi::c_char,
    pub b_p_inex: *mut ::core::ffi::c_char,
    pub b_p_inex_flags: uint32_t,
    pub b_p_inde: *mut ::core::ffi::c_char,
    pub b_p_inde_flags: uint32_t,
    pub b_p_indk: *mut ::core::ffi::c_char,
    pub b_p_fp: *mut ::core::ffi::c_char,
    pub b_p_fex: *mut ::core::ffi::c_char,
    pub b_p_fex_flags: uint32_t,
    pub b_p_fs: ::core::ffi::c_int,
    pub b_p_kp: *mut ::core::ffi::c_char,
    pub b_p_lisp: ::core::ffi::c_int,
    pub b_p_lop: *mut ::core::ffi::c_char,
    pub b_p_menc: *mut ::core::ffi::c_char,
    pub b_p_mps: *mut ::core::ffi::c_char,
    pub b_p_ml: ::core::ffi::c_int,
    pub b_p_ml_nobin: ::core::ffi::c_int,
    pub b_p_ma: ::core::ffi::c_int,
    pub b_p_nf: *mut ::core::ffi::c_char,
    pub b_p_pi: ::core::ffi::c_int,
    pub b_p_qe: *mut ::core::ffi::c_char,
    pub b_p_ro: ::core::ffi::c_int,
    pub b_p_sw: OptInt,
    pub b_p_scbk: OptInt,
    pub b_p_si: ::core::ffi::c_int,
    pub b_p_sts: OptInt,
    pub b_p_sts_nopaste: OptInt,
    pub b_p_sua: *mut ::core::ffi::c_char,
    pub b_p_swf: ::core::ffi::c_int,
    pub b_p_smc: OptInt,
    pub b_p_syn: *mut ::core::ffi::c_char,
    pub b_p_ts: OptInt,
    pub b_p_tw: OptInt,
    pub b_p_tw_nobin: OptInt,
    pub b_p_tw_nopaste: OptInt,
    pub b_p_wm: OptInt,
    pub b_p_wm_nobin: OptInt,
    pub b_p_wm_nopaste: OptInt,
    pub b_p_vsts: *mut ::core::ffi::c_char,
    pub b_p_vsts_array: *mut colnr_T,
    pub b_p_vsts_nopaste: *mut ::core::ffi::c_char,
    pub b_p_vts: *mut ::core::ffi::c_char,
    pub b_p_vts_array: *mut colnr_T,
    pub b_p_keymap: *mut ::core::ffi::c_char,
    pub b_p_gefm: *mut ::core::ffi::c_char,
    pub b_p_gp: *mut ::core::ffi::c_char,
    pub b_p_mp: *mut ::core::ffi::c_char,
    pub b_p_efm: *mut ::core::ffi::c_char,
    pub b_p_ep: *mut ::core::ffi::c_char,
    pub b_p_path: *mut ::core::ffi::c_char,
    pub b_p_ar: ::core::ffi::c_int,
    pub b_p_tags: *mut ::core::ffi::c_char,
    pub b_p_tc: *mut ::core::ffi::c_char,
    pub b_tc_flags: ::core::ffi::c_uint,
    pub b_p_dict: *mut ::core::ffi::c_char,
    pub b_p_dia: *mut ::core::ffi::c_char,
    pub b_p_tsr: *mut ::core::ffi::c_char,
    pub b_p_tsrfu: *mut ::core::ffi::c_char,
    pub b_tsrfu_cb: Callback,
    pub b_p_ul: OptInt,
    pub b_p_udf: ::core::ffi::c_int,
    pub b_p_lw: *mut ::core::ffi::c_char,
    pub b_ind_level: ::core::ffi::c_int,
    pub b_ind_open_imag: ::core::ffi::c_int,
    pub b_ind_no_brace: ::core::ffi::c_int,
    pub b_ind_first_open: ::core::ffi::c_int,
    pub b_ind_open_extra: ::core::ffi::c_int,
    pub b_ind_close_extra: ::core::ffi::c_int,
    pub b_ind_open_left_imag: ::core::ffi::c_int,
    pub b_ind_jump_label: ::core::ffi::c_int,
    pub b_ind_case: ::core::ffi::c_int,
    pub b_ind_case_code: ::core::ffi::c_int,
    pub b_ind_case_break: ::core::ffi::c_int,
    pub b_ind_param: ::core::ffi::c_int,
    pub b_ind_func_type: ::core::ffi::c_int,
    pub b_ind_comment: ::core::ffi::c_int,
    pub b_ind_in_comment: ::core::ffi::c_int,
    pub b_ind_in_comment2: ::core::ffi::c_int,
    pub b_ind_cpp_baseclass: ::core::ffi::c_int,
    pub b_ind_continuation: ::core::ffi::c_int,
    pub b_ind_unclosed: ::core::ffi::c_int,
    pub b_ind_unclosed2: ::core::ffi::c_int,
    pub b_ind_unclosed_noignore: ::core::ffi::c_int,
    pub b_ind_unclosed_wrapped: ::core::ffi::c_int,
    pub b_ind_unclosed_whiteok: ::core::ffi::c_int,
    pub b_ind_matching_paren: ::core::ffi::c_int,
    pub b_ind_paren_prev: ::core::ffi::c_int,
    pub b_ind_maxparen: ::core::ffi::c_int,
    pub b_ind_maxcomment: ::core::ffi::c_int,
    pub b_ind_scopedecl: ::core::ffi::c_int,
    pub b_ind_scopedecl_code: ::core::ffi::c_int,
    pub b_ind_java: ::core::ffi::c_int,
    pub b_ind_js: ::core::ffi::c_int,
    pub b_ind_keep_case_label: ::core::ffi::c_int,
    pub b_ind_hash_comment: ::core::ffi::c_int,
    pub b_ind_cpp_namespace: ::core::ffi::c_int,
    pub b_ind_if_for_while: ::core::ffi::c_int,
    pub b_ind_cpp_extern_c: ::core::ffi::c_int,
    pub b_ind_pragma: ::core::ffi::c_int,
    pub b_no_eol_lnum: linenr_T,
    pub b_start_eof: ::core::ffi::c_int,
    pub b_start_eol: ::core::ffi::c_int,
    pub b_start_ffc: ::core::ffi::c_int,
    pub b_start_fenc: *mut ::core::ffi::c_char,
    pub b_bad_char: ::core::ffi::c_int,
    pub b_start_bomb: ::core::ffi::c_int,
    pub b_bufvar: ScopeDictDictItem,
    pub b_vars: *mut dict_T,
    pub b_may_swap: bool,
    pub b_did_warn: bool,
    pub b_help: bool,
    pub b_spell: bool,
    pub b_prompt_text: *mut ::core::ffi::c_char,
    pub b_prompt_callback: Callback,
    pub b_prompt_interrupt: Callback,
    pub b_prompt_append_new_line: bool,
    pub b_prompt_insert: ::core::ffi::c_int,
    pub b_prompt_start: fmark_T,
    pub b_s: synblock_T,
    pub b_signcols: C2Rust_Unnamed_4,
    pub terminal: *mut Terminal,
    pub additional_data: *mut AdditionalData,
    pub b_mapped_ctrl_c: ::core::ffi::c_int,
    pub b_marktree: [MarkTree; 1],
    pub b_extmark_ns: [Map_uint32_t_uint32_t; 1],
    pub b_prev_line_count: ::core::ffi::c_int,
    pub update_channels: C2Rust_Unnamed_2,
    pub update_callbacks: C2Rust_Unnamed_1,
    pub update_need_codepoints: bool,
    pub deleted_bytes: size_t,
    pub deleted_bytes2: size_t,
    pub deleted_codepoints: size_t,
    pub deleted_codeunits: size_t,
    pub flush_count: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_1 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut BufUpdateCallbacks,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct BufUpdateCallbacks {
    pub on_lines: LuaRef,
    pub on_bytes: LuaRef,
    pub on_changedtick: LuaRef,
    pub on_detach: LuaRef,
    pub on_reload: LuaRef,
    pub utf_sizes: bool,
    pub preview: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_2 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut uint64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_uint32_t_uint32_t {
    pub set: Set_uint32_t,
    pub values: *mut uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Set_uint32_t {
    pub h: MapHash,
    pub keys: *mut uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MapHash {
    pub n_buckets: uint32_t,
    pub size: uint32_t,
    pub n_occupied: uint32_t,
    pub upper_bound: uint32_t,
    pub n_keys: uint32_t,
    pub keys_capacity: uint32_t,
    pub hash: *mut uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MarkTree {
    pub root: *mut MTNode,
    pub meta_root: [uint32_t; 5],
    pub n_keys: size_t,
    pub n_nodes: size_t,
    pub id2node: [Map_uint64_t_ptr_t; 1],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_uint64_t_ptr_t {
    pub set: Set_uint64_t,
    pub values: *mut ptr_t,
}
pub type ptr_t = *mut ::core::ffi::c_void;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Set_uint64_t {
    pub h: MapHash,
    pub keys: *mut uint64_t,
}
pub type MTNode = mtnode_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mtnode_s {
    pub n: int32_t,
    pub level: int16_t,
    pub p_idx: int16_t,
    pub intersect: Intersection,
    pub parent: *mut MTNode,
    pub key: [MTKey; 19],
    pub s: [mtnode_inner_s; 0],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mtnode_inner_s {
    pub i_ptr: [*mut MTNode; 20],
    pub i_meta: [[uint32_t; 5]; 20],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MTKey {
    pub pos: MTPos,
    pub ns: uint32_t,
    pub id: uint32_t,
    pub flags: uint16_t,
    pub decor_data: DecorInlineData,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union DecorInlineData {
    pub hl: DecorHighlightInline,
    pub ext: DecorExt,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorExt {
    pub sh_idx: uint32_t,
    pub vt: *mut DecorVirtText,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorVirtText {
    pub flags: uint8_t,
    pub hl_mode: uint8_t,
    pub priority: DecorPriority,
    pub width: ::core::ffi::c_int,
    pub col: ::core::ffi::c_int,
    pub pos: VirtTextPos,
    pub data: C2Rust_Unnamed_3,
    pub next: *mut DecorVirtText,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_3 {
    pub virt_text: VirtText,
    pub virt_lines: VirtLines,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VirtLines {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut virt_line,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct virt_line {
    pub line: VirtText,
    pub flags: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VirtText {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut VirtTextChunk,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VirtTextChunk {
    pub text: *mut ::core::ffi::c_char,
    pub hl_id: ::core::ffi::c_int,
}
pub type VirtTextPos = ::core::ffi::c_uint;
pub const kVPosWinCol: VirtTextPos = 5;
pub const kVPosRightAlign: VirtTextPos = 4;
pub const kVPosOverlay: VirtTextPos = 3;
pub const kVPosInline: VirtTextPos = 2;
pub const kVPosEndOfLineRightAlign: VirtTextPos = 1;
pub const kVPosEndOfLine: VirtTextPos = 0;
pub type DecorPriority = uint16_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorHighlightInline {
    pub flags: uint16_t,
    pub priority: DecorPriority,
    pub hl_id: ::core::ffi::c_int,
    pub conceal_char: schar_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MTPos {
    pub row: int32_t,
    pub col: int32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Intersection {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut uint64_t,
    pub init_array: [uint64_t; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct AdditionalData {
    pub nitems: uint32_t,
    pub nbytes: uint32_t,
    pub data: [::core::ffi::c_char; 0],
}
pub type Terminal = terminal;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_4 {
    pub max: ::core::ffi::c_int,
    pub last_max: ::core::ffi::c_int,
    pub count: [::core::ffi::c_int; 9],
    pub autom: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct synblock_T {
    pub b_keywtab: hashtab_T,
    pub b_keywtab_ic: hashtab_T,
    pub b_syn_error: bool,
    pub b_syn_slow: bool,
    pub b_syn_ic: ::core::ffi::c_int,
    pub b_syn_foldlevel: ::core::ffi::c_int,
    pub b_syn_spell: ::core::ffi::c_int,
    pub b_syn_patterns: garray_T,
    pub b_syn_clusters: garray_T,
    pub b_spell_cluster_id: ::core::ffi::c_int,
    pub b_nospell_cluster_id: ::core::ffi::c_int,
    pub b_syn_containedin: ::core::ffi::c_int,
    pub b_syn_sync_flags: ::core::ffi::c_int,
    pub b_syn_sync_id: int16_t,
    pub b_syn_sync_minlines: linenr_T,
    pub b_syn_sync_maxlines: linenr_T,
    pub b_syn_sync_linebreaks: linenr_T,
    pub b_syn_linecont_pat: *mut ::core::ffi::c_char,
    pub b_syn_linecont_prog: *mut regprog_T,
    pub b_syn_linecont_time: syn_time_T,
    pub b_syn_linecont_ic: ::core::ffi::c_int,
    pub b_syn_topgrp: ::core::ffi::c_int,
    pub b_syn_conceal: ::core::ffi::c_int,
    pub b_syn_folditems: ::core::ffi::c_int,
    pub b_sst_array: *mut synstate_T,
    pub b_sst_len: ::core::ffi::c_int,
    pub b_sst_first: *mut synstate_T,
    pub b_sst_firstfree: *mut synstate_T,
    pub b_sst_freecount: ::core::ffi::c_int,
    pub b_sst_check_lnum: linenr_T,
    pub b_sst_lasttick: disptick_T,
    pub b_langp: garray_T,
    pub b_spell_ismw: [bool; 256],
    pub b_spell_ismw_mb: *mut ::core::ffi::c_char,
    pub b_p_spc: *mut ::core::ffi::c_char,
    pub b_cap_prog: *mut regprog_T,
    pub b_p_spf: *mut ::core::ffi::c_char,
    pub b_p_spl: *mut ::core::ffi::c_char,
    pub b_p_spo: *mut ::core::ffi::c_char,
    pub b_p_spo_flags: ::core::ffi::c_uint,
    pub b_cjk: ::core::ffi::c_int,
    pub b_syn_chartab: [uint8_t; 32],
    pub b_syn_isk: *mut ::core::ffi::c_char,
}
pub type regprog_T = regprog;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct garray_T {
    pub ga_len: ::core::ffi::c_int,
    pub ga_maxlen: ::core::ffi::c_int,
    pub ga_itemsize: ::core::ffi::c_int,
    pub ga_growsize: ::core::ffi::c_int,
    pub ga_data: *mut ::core::ffi::c_void,
}
pub type disptick_T = uint64_t;
pub type linenr_T = int32_t;
pub type synstate_T = syn_state;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct syn_state {
    pub sst_next: *mut synstate_T,
    pub sst_lnum: linenr_T,
    pub sst_union: C2Rust_Unnamed_5,
    pub sst_next_flags: ::core::ffi::c_int,
    pub sst_stacksize: ::core::ffi::c_int,
    pub sst_next_list: *mut int16_t,
    pub sst_tick: disptick_T,
    pub sst_change_lnum: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_5 {
    pub sst_stack: [bufstate_T; 7],
    pub sst_ga: garray_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bufstate_T {
    pub bs_idx: ::core::ffi::c_int,
    pub bs_flags: ::core::ffi::c_int,
    pub bs_seqnr: ::core::ffi::c_int,
    pub bs_cchar: ::core::ffi::c_int,
    pub bs_extmatch: *mut reg_extmatch_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct reg_extmatch_T {
    pub refcnt: int16_t,
    pub matches: [*mut uint8_t; 10],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct syn_time_T {
    pub total: proftime_T,
    pub slowest: proftime_T,
    pub count: ::core::ffi::c_int,
    pub match_0: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hashtab_T {
    pub ht_mask: hash_T,
    pub ht_used: size_t,
    pub ht_filled: size_t,
    pub ht_changed: ::core::ffi::c_int,
    pub ht_locked: ::core::ffi::c_int,
    pub ht_array: *mut hashitem_T,
    pub ht_smallarray: [hashitem_T; 16],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hashitem_T {
    pub hi_hash: hash_T,
    pub hi_key: *mut ::core::ffi::c_char,
}
pub type hash_T = size_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct fmark_T {
    pub mark: pos_T,
    pub fnum: ::core::ffi::c_int,
    pub timestamp: Timestamp,
    pub view: fmarkv_T,
    pub additional_data: *mut AdditionalData,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct fmarkv_T {
    pub topline_offset: linenr_T,
    pub skipcol: colnr_T,
}
pub type colnr_T = ::core::ffi::c_int;
pub type Timestamp = uint64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct pos_T {
    pub lnum: linenr_T,
    pub col: colnr_T,
    pub coladd: colnr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Callback {
    pub data: C2Rust_Unnamed_6,
    pub type_0: CallbackType,
}
pub type CallbackType = ::core::ffi::c_uint;
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_6 {
    pub funcref: *mut ::core::ffi::c_char,
    pub partial: *mut partial_T,
    pub luaref: LuaRef,
}
pub type partial_T = partial_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct partial_S {
    pub pt_refcount: ::core::ffi::c_int,
    pub pt_copyID: ::core::ffi::c_int,
    pub pt_name: *mut ::core::ffi::c_char,
    pub pt_func: *mut ufunc_T,
    pub pt_auto: bool,
    pub pt_argc: ::core::ffi::c_int,
    pub pt_argv: *mut typval_T,
    pub pt_dict: *mut dict_T,
}
pub type dict_T = dictvar_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dictvar_S {
    pub dv_lock: VarLockStatus,
    pub dv_scope: ScopeType,
    pub dv_refcount: ::core::ffi::c_int,
    pub dv_copyID: ::core::ffi::c_int,
    pub dv_hashtab: hashtab_T,
    pub dv_copydict: *mut dict_T,
    pub dv_used_next: *mut dict_T,
    pub dv_used_prev: *mut dict_T,
    pub watchers: QUEUE,
    pub lua_table_ref: LuaRef,
}
pub type QUEUE = queue;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct queue {
    pub next: *mut queue,
    pub prev: *mut queue,
}
pub type ScopeType = ::core::ffi::c_uint;
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_NO_SCOPE: ScopeType = 0;
pub type VarLockStatus = ::core::ffi::c_uint;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct typval_T {
    pub v_type: VarType,
    pub v_lock: VarLockStatus,
    pub vval: typval_vval_union,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union typval_vval_union {
    pub v_number: varnumber_T,
    pub v_bool: BoolVarValue,
    pub v_special: SpecialVarValue,
    pub v_float: float_T,
    pub v_string: *mut ::core::ffi::c_char,
    pub v_list: *mut list_T,
    pub v_dict: *mut dict_T,
    pub v_partial: *mut partial_T,
    pub v_blob: *mut blob_T,
}
pub type blob_T = blobvar_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct blobvar_S {
    pub bv_ga: garray_T,
    pub bv_refcount: ::core::ffi::c_int,
    pub bv_lock: VarLockStatus,
}
pub type list_T = listvar_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct listvar_S {
    pub lv_first: *mut listitem_T,
    pub lv_last: *mut listitem_T,
    pub lv_watch: *mut listwatch_T,
    pub lv_idx_item: *mut listitem_T,
    pub lv_copylist: *mut list_T,
    pub lv_used_next: *mut list_T,
    pub lv_used_prev: *mut list_T,
    pub lv_refcount: ::core::ffi::c_int,
    pub lv_len: ::core::ffi::c_int,
    pub lv_idx: ::core::ffi::c_int,
    pub lv_copyID: ::core::ffi::c_int,
    pub lv_lock: VarLockStatus,
    pub lua_table_ref: LuaRef,
}
pub type listitem_T = listitem_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct listitem_S {
    pub li_next: *mut listitem_T,
    pub li_prev: *mut listitem_T,
    pub li_tv: typval_T,
}
pub type listwatch_T = listwatch_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct listwatch_S {
    pub lw_item: *mut listitem_T,
    pub lw_next: *mut listwatch_T,
}
pub type SpecialVarValue = ::core::ffi::c_uint;
pub const kSpecialVarNull: SpecialVarValue = 0;
pub type BoolVarValue = ::core::ffi::c_uint;
pub const kBoolVarTrue: BoolVarValue = 1;
pub const kBoolVarFalse: BoolVarValue = 0;
pub type varnumber_T = int64_t;
pub type VarType = ::core::ffi::c_uint;
pub const VAR_BLOB: VarType = 10;
pub const VAR_PARTIAL: VarType = 9;
pub const VAR_SPECIAL: VarType = 8;
pub const VAR_BOOL: VarType = 7;
pub const VAR_FLOAT: VarType = 6;
pub const VAR_DICT: VarType = 5;
pub const VAR_LIST: VarType = 4;
pub const VAR_FUNC: VarType = 3;
pub const VAR_STRING: VarType = 2;
pub const VAR_NUMBER: VarType = 1;
pub const VAR_UNKNOWN: VarType = 0;
pub type ufunc_T = ufunc_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ufunc_S {
    pub uf_varargs: ::core::ffi::c_int,
    pub uf_flags: ::core::ffi::c_int,
    pub uf_calls: ::core::ffi::c_int,
    pub uf_cleared: bool,
    pub uf_args: garray_T,
    pub uf_def_args: garray_T,
    pub uf_lines: garray_T,
    pub uf_profiling: ::core::ffi::c_int,
    pub uf_prof_initialized: ::core::ffi::c_int,
    pub uf_luaref: LuaRef,
    pub uf_tm_count: ::core::ffi::c_int,
    pub uf_tm_total: proftime_T,
    pub uf_tm_self: proftime_T,
    pub uf_tm_children: proftime_T,
    pub uf_tml_count: *mut ::core::ffi::c_int,
    pub uf_tml_total: *mut proftime_T,
    pub uf_tml_self: *mut proftime_T,
    pub uf_tml_start: proftime_T,
    pub uf_tml_children: proftime_T,
    pub uf_tml_wait: proftime_T,
    pub uf_tml_idx: ::core::ffi::c_int,
    pub uf_tml_execed: ::core::ffi::c_int,
    pub uf_script_ctx: sctx_T,
    pub uf_refcount: ::core::ffi::c_int,
    pub uf_scoped: *mut funccall_T,
    pub uf_name_exp: *mut ::core::ffi::c_char,
    pub uf_namelen: size_t,
    pub uf_name: [::core::ffi::c_char; 0],
}
pub type funccall_T = funccall_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct funccall_S {
    pub fc_func: *mut ufunc_T,
    pub fc_linenr: ::core::ffi::c_int,
    pub fc_returned: ::core::ffi::c_int,
    pub fc_fixvar: [C2Rust_Unnamed_7; 12],
    pub fc_l_vars: dict_T,
    pub fc_l_vars_var: ScopeDictDictItem,
    pub fc_l_avars: dict_T,
    pub fc_l_avars_var: ScopeDictDictItem,
    pub fc_l_varlist: list_T,
    pub fc_l_listitems: [listitem_T; 20],
    pub fc_rettv: *mut typval_T,
    pub fc_breakpoint: linenr_T,
    pub fc_dbg_tick: ::core::ffi::c_int,
    pub fc_level: ::core::ffi::c_int,
    pub fc_defer: garray_T,
    pub fc_prof_child: proftime_T,
    pub fc_caller: *mut funccall_T,
    pub fc_refcount: ::core::ffi::c_int,
    pub fc_copyID: ::core::ffi::c_int,
    pub fc_ufuncs: garray_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ScopeDictDictItem {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 1],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_7 {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 21],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sctx_T {
    pub sc_sid: scid_T,
    pub sc_seq: ::core::ffi::c_int,
    pub sc_lnum: linenr_T,
    pub sc_chan: uint64_t,
}
pub type scid_T = ::core::ffi::c_int;
pub type u_header_T = u_header;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct u_header {
    pub uh_next: C2Rust_Unnamed_11,
    pub uh_prev: C2Rust_Unnamed_10,
    pub uh_alt_next: C2Rust_Unnamed_9,
    pub uh_alt_prev: C2Rust_Unnamed_8,
    pub uh_seq: ::core::ffi::c_int,
    pub uh_walk: ::core::ffi::c_int,
    pub uh_entry: *mut u_entry_T,
    pub uh_getbot_entry: *mut u_entry_T,
    pub uh_cursor: pos_T,
    pub uh_cursor_vcol: colnr_T,
    pub uh_flags: ::core::ffi::c_int,
    pub uh_namedm: [fmark_T; 26],
    pub uh_extmark: extmark_undo_vec_t,
    pub uh_visual: visualinfo_T,
    pub uh_time: time_t,
    pub uh_save_nr: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct visualinfo_T {
    pub vi_start: pos_T,
    pub vi_end: pos_T,
    pub vi_mode: ::core::ffi::c_int,
    pub vi_curswant: colnr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct extmark_undo_vec_t {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ExtmarkUndoObject,
}
pub type ExtmarkUndoObject = undo_object;
pub type u_entry_T = u_entry;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct u_entry {
    pub ue_next: *mut u_entry_T,
    pub ue_top: linenr_T,
    pub ue_bot: linenr_T,
    pub ue_lcount: linenr_T,
    pub ue_array: *mut *mut ::core::ffi::c_char,
    pub ue_size: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_8 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_9 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_10 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_11 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
pub type mapblock_T = mapblock;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mapblock {
    pub m_next: *mut mapblock_T,
    pub m_alt: *mut mapblock_T,
    pub m_keys: *mut ::core::ffi::c_char,
    pub m_str: *mut ::core::ffi::c_char,
    pub m_orig_str: *mut ::core::ffi::c_char,
    pub m_luaref: LuaRef,
    pub m_keylen: ::core::ffi::c_int,
    pub m_mode: ::core::ffi::c_int,
    pub m_simplified: ::core::ffi::c_int,
    pub m_noremap: ::core::ffi::c_int,
    pub m_silent: ::core::ffi::c_char,
    pub m_nowait: ::core::ffi::c_char,
    pub m_expr: ::core::ffi::c_char,
    pub m_script_ctx: sctx_T,
    pub m_desc: *mut ::core::ffi::c_char,
    pub m_replace_keycodes: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_12 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut *mut WinInfo,
}
pub type WinInfo = wininfo_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wininfo_S {
    pub wi_win: *mut win_T,
    pub wi_mark: fmark_T,
    pub wi_optset: bool,
    pub wi_opt: winopt_T,
    pub wi_fold_manual: bool,
    pub wi_folds: garray_T,
    pub wi_changelistidx: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct winopt_T {
    pub wo_arab: ::core::ffi::c_int,
    pub wo_bri: ::core::ffi::c_int,
    pub wo_briopt: *mut ::core::ffi::c_char,
    pub wo_diff: ::core::ffi::c_int,
    pub wo_fdc: *mut ::core::ffi::c_char,
    pub wo_eiw: *mut ::core::ffi::c_char,
    pub wo_fdc_save: *mut ::core::ffi::c_char,
    pub wo_fen: ::core::ffi::c_int,
    pub wo_fen_save: ::core::ffi::c_int,
    pub wo_fdi: *mut ::core::ffi::c_char,
    pub wo_fdl: OptInt,
    pub wo_fdl_save: OptInt,
    pub wo_fdm: *mut ::core::ffi::c_char,
    pub wo_fdm_save: *mut ::core::ffi::c_char,
    pub wo_fml: OptInt,
    pub wo_fdn: OptInt,
    pub wo_fde: *mut ::core::ffi::c_char,
    pub wo_fdt: *mut ::core::ffi::c_char,
    pub wo_fmr: *mut ::core::ffi::c_char,
    pub wo_lbr: ::core::ffi::c_int,
    pub wo_list: ::core::ffi::c_int,
    pub wo_nu: ::core::ffi::c_int,
    pub wo_rnu: ::core::ffi::c_int,
    pub wo_ve: *mut ::core::ffi::c_char,
    pub wo_ve_flags: ::core::ffi::c_uint,
    pub wo_nuw: OptInt,
    pub wo_wfb: ::core::ffi::c_int,
    pub wo_wfh: ::core::ffi::c_int,
    pub wo_wfw: ::core::ffi::c_int,
    pub wo_pvw: ::core::ffi::c_int,
    pub wo_lhi: OptInt,
    pub wo_rl: ::core::ffi::c_int,
    pub wo_rlc: *mut ::core::ffi::c_char,
    pub wo_scr: OptInt,
    pub wo_sms: ::core::ffi::c_int,
    pub wo_spell: ::core::ffi::c_int,
    pub wo_cuc: ::core::ffi::c_int,
    pub wo_cul: ::core::ffi::c_int,
    pub wo_culopt: *mut ::core::ffi::c_char,
    pub wo_cc: *mut ::core::ffi::c_char,
    pub wo_sbr: *mut ::core::ffi::c_char,
    pub wo_stc: *mut ::core::ffi::c_char,
    pub wo_stl: *mut ::core::ffi::c_char,
    pub wo_wbr: *mut ::core::ffi::c_char,
    pub wo_scb: ::core::ffi::c_int,
    pub wo_diff_saved: ::core::ffi::c_int,
    pub wo_scb_save: ::core::ffi::c_int,
    pub wo_wrap: ::core::ffi::c_int,
    pub wo_wrap_save: ::core::ffi::c_int,
    pub wo_cocu: *mut ::core::ffi::c_char,
    pub wo_cole: OptInt,
    pub wo_crb: ::core::ffi::c_int,
    pub wo_crb_save: ::core::ffi::c_int,
    pub wo_scl: *mut ::core::ffi::c_char,
    pub wo_siso: OptInt,
    pub wo_so: OptInt,
    pub wo_winhl: *mut ::core::ffi::c_char,
    pub wo_lcs: *mut ::core::ffi::c_char,
    pub wo_fcs: *mut ::core::ffi::c_char,
    pub wo_winbl: OptInt,
    pub wo_wrap_flags: uint32_t,
    pub wo_stl_flags: uint32_t,
    pub wo_wbr_flags: uint32_t,
    pub wo_fde_flags: uint32_t,
    pub wo_fdt_flags: uint32_t,
    pub wo_script_ctx: [sctx_T; 51],
}
pub type win_T = window_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_S {
    pub handle: handle_T,
    pub w_buffer: *mut buf_T,
    pub w_s: *mut synblock_T,
    pub w_ns_hl: ::core::ffi::c_int,
    pub w_ns_hl_winhl: ::core::ffi::c_int,
    pub w_ns_hl_active: ::core::ffi::c_int,
    pub w_ns_hl_attr: *mut ::core::ffi::c_int,
    pub w_ns_set: Set_uint32_t,
    pub w_hl_id_normal: ::core::ffi::c_int,
    pub w_hl_attr_normal: ::core::ffi::c_int,
    pub w_hl_attr_normalnc: ::core::ffi::c_int,
    pub w_hl_needs_update: ::core::ffi::c_int,
    pub w_prev: *mut win_T,
    pub w_next: *mut win_T,
    pub w_locked: bool,
    pub w_frame: *mut frame_T,
    pub w_cursor: pos_T,
    pub w_curswant: colnr_T,
    pub w_set_curswant: ::core::ffi::c_int,
    pub w_cursorline: linenr_T,
    pub w_last_cursorline: linenr_T,
    pub w_old_visual_mode: ::core::ffi::c_char,
    pub w_old_cursor_lnum: linenr_T,
    pub w_old_cursor_fcol: colnr_T,
    pub w_old_cursor_lcol: colnr_T,
    pub w_old_visual_lnum: linenr_T,
    pub w_old_visual_col: colnr_T,
    pub w_old_curswant: colnr_T,
    pub w_last_cursor_lnum_rnu: linenr_T,
    pub w_p_lcs_chars: lcs_chars_T,
    pub w_p_fcs_chars: fcs_chars_T,
    pub w_topline: linenr_T,
    pub w_topline_was_set: ::core::ffi::c_char,
    pub w_topfill: ::core::ffi::c_int,
    pub w_old_topfill: ::core::ffi::c_int,
    pub w_botfill: bool,
    pub w_old_botfill: bool,
    pub w_leftcol: colnr_T,
    pub w_skipcol: colnr_T,
    pub w_last_topline: linenr_T,
    pub w_last_topfill: ::core::ffi::c_int,
    pub w_last_leftcol: colnr_T,
    pub w_last_skipcol: colnr_T,
    pub w_last_width: ::core::ffi::c_int,
    pub w_last_height: ::core::ffi::c_int,
    pub w_winrow: ::core::ffi::c_int,
    pub w_height: ::core::ffi::c_int,
    pub w_prev_winrow: ::core::ffi::c_int,
    pub w_prev_height: ::core::ffi::c_int,
    pub w_status_height: ::core::ffi::c_int,
    pub w_winbar_height: ::core::ffi::c_int,
    pub w_wincol: ::core::ffi::c_int,
    pub w_width: ::core::ffi::c_int,
    pub w_hsep_height: ::core::ffi::c_int,
    pub w_vsep_width: ::core::ffi::c_int,
    pub w_save_cursor: pos_save_T,
    pub w_do_win_fix_cursor: bool,
    pub w_winrow_off: ::core::ffi::c_int,
    pub w_wincol_off: ::core::ffi::c_int,
    pub w_view_height: ::core::ffi::c_int,
    pub w_view_width: ::core::ffi::c_int,
    pub w_height_request: ::core::ffi::c_int,
    pub w_width_request: ::core::ffi::c_int,
    pub w_border_adj: [::core::ffi::c_int; 4],
    pub w_height_outer: ::core::ffi::c_int,
    pub w_width_outer: ::core::ffi::c_int,
    pub w_valid: ::core::ffi::c_int,
    pub w_valid_cursor: pos_T,
    pub w_valid_leftcol: colnr_T,
    pub w_valid_skipcol: colnr_T,
    pub w_viewport_invalid: bool,
    pub w_viewport_last_topline: linenr_T,
    pub w_viewport_last_botline: linenr_T,
    pub w_viewport_last_topfill: linenr_T,
    pub w_viewport_last_skipcol: linenr_T,
    pub w_cline_height: ::core::ffi::c_int,
    pub w_cline_folded: bool,
    pub w_cline_row: ::core::ffi::c_int,
    pub w_virtcol: colnr_T,
    pub w_wrow: ::core::ffi::c_int,
    pub w_wcol: ::core::ffi::c_int,
    pub w_botline: linenr_T,
    pub w_empty_rows: ::core::ffi::c_int,
    pub w_filler_rows: ::core::ffi::c_int,
    pub w_lines_valid: ::core::ffi::c_int,
    pub w_lines: *mut wline_T,
    pub w_lines_size: ::core::ffi::c_int,
    pub w_folds: garray_T,
    pub w_fold_manual: bool,
    pub w_foldinvalid: bool,
    pub w_nrwidth: ::core::ffi::c_int,
    pub w_scwidth: ::core::ffi::c_int,
    pub w_minscwidth: ::core::ffi::c_int,
    pub w_maxscwidth: ::core::ffi::c_int,
    pub w_redr_type: ::core::ffi::c_int,
    pub w_upd_rows: ::core::ffi::c_int,
    pub w_redraw_top: linenr_T,
    pub w_redraw_bot: linenr_T,
    pub w_redr_status: bool,
    pub w_redr_border: bool,
    pub w_redr_statuscol: bool,
    pub w_display_tick: disptick_T,
    pub w_stl_cursor: pos_T,
    pub w_stl_virtcol: colnr_T,
    pub w_stl_topline: linenr_T,
    pub w_stl_line_count: linenr_T,
    pub w_stl_topfill: ::core::ffi::c_int,
    pub w_stl_empty: ::core::ffi::c_char,
    pub w_stl_recording: ::core::ffi::c_int,
    pub w_stl_state: ::core::ffi::c_int,
    pub w_stl_visual_mode: ::core::ffi::c_int,
    pub w_stl_visual_pos: pos_T,
    pub w_alt_fnum: ::core::ffi::c_int,
    pub w_alist: *mut alist_T,
    pub w_arg_idx: ::core::ffi::c_int,
    pub w_arg_idx_invalid: ::core::ffi::c_int,
    pub w_localdir: *mut ::core::ffi::c_char,
    pub w_prevdir: *mut ::core::ffi::c_char,
    pub w_onebuf_opt: winopt_T,
    pub w_allbuf_opt: winopt_T,
    pub w_p_cc_cols: *mut ::core::ffi::c_int,
    pub w_p_culopt_flags: uint8_t,
    pub w_briopt_min: ::core::ffi::c_int,
    pub w_briopt_shift: ::core::ffi::c_int,
    pub w_briopt_sbr: bool,
    pub w_briopt_list: ::core::ffi::c_int,
    pub w_briopt_vcol: ::core::ffi::c_int,
    pub w_scbind_pos: ::core::ffi::c_int,
    pub w_winvar: ScopeDictDictItem,
    pub w_vars: *mut dict_T,
    pub w_pcmark: pos_T,
    pub w_prev_pcmark: pos_T,
    pub w_jumplist: [xfmark_T; 100],
    pub w_jumplistlen: ::core::ffi::c_int,
    pub w_jumplistidx: ::core::ffi::c_int,
    pub w_changelistidx: ::core::ffi::c_int,
    pub w_match_head: *mut matchitem_T,
    pub w_next_match_id: ::core::ffi::c_int,
    pub w_tagstack: [taggy_T; 20],
    pub w_tagstackidx: ::core::ffi::c_int,
    pub w_tagstacklen: ::core::ffi::c_int,
    pub w_grid: GridView,
    pub w_grid_alloc: ScreenGrid,
    pub w_pos_changed: bool,
    pub w_floating: bool,
    pub w_float_is_info: bool,
    pub w_config: WinConfig,
    pub w_fraction: ::core::ffi::c_int,
    pub w_prev_fraction_row: ::core::ffi::c_int,
    pub w_nrwidth_line_count: linenr_T,
    pub w_statuscol_line_count: linenr_T,
    pub w_nrwidth_width: ::core::ffi::c_int,
    pub w_llist: *mut qf_info_T,
    pub w_llist_ref: *mut qf_info_T,
    pub w_status_click_defs: *mut StlClickDefinition,
    pub w_status_click_defs_size: size_t,
    pub w_winbar_click_defs: *mut StlClickDefinition,
    pub w_winbar_click_defs_size: size_t,
    pub w_statuscol_click_defs: *mut StlClickDefinition,
    pub w_statuscol_click_defs_size: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StlClickDefinition {
    pub type_0: C2Rust_Unnamed_13,
    pub tabnr: ::core::ffi::c_int,
    pub func: *mut ::core::ffi::c_char,
}
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const kStlClickFuncRun: C2Rust_Unnamed_13 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_13 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_13 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_13 = 0;
pub type qf_info_T = qf_info_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct WinConfig {
    pub window: Window,
    pub bufpos: lpos_T,
    pub height: ::core::ffi::c_int,
    pub width: ::core::ffi::c_int,
    pub row: ::core::ffi::c_double,
    pub col: ::core::ffi::c_double,
    pub anchor: FloatAnchor,
    pub relative: FloatRelative,
    pub external: bool,
    pub focusable: bool,
    pub mouse: bool,
    pub split: WinSplit,
    pub zindex: ::core::ffi::c_int,
    pub style: WinStyle,
    pub border: bool,
    pub shadow: bool,
    pub border_chars: [[::core::ffi::c_char; 32]; 8],
    pub border_hl_ids: [::core::ffi::c_int; 8],
    pub border_attr: [::core::ffi::c_int; 8],
    pub title: bool,
    pub title_pos: AlignTextPos,
    pub title_chunks: VirtText,
    pub title_width: ::core::ffi::c_int,
    pub footer: bool,
    pub footer_pos: AlignTextPos,
    pub footer_chunks: VirtText,
    pub footer_width: ::core::ffi::c_int,
    pub noautocmd: bool,
    pub fixed: bool,
    pub hide: bool,
    pub _cmdline_offset: ::core::ffi::c_int,
}
pub type AlignTextPos = ::core::ffi::c_uint;
pub const kAlignRight: AlignTextPos = 2;
pub const kAlignCenter: AlignTextPos = 1;
pub const kAlignLeft: AlignTextPos = 0;
pub type WinStyle = ::core::ffi::c_uint;
pub const kWinStyleMinimal: WinStyle = 1;
pub const kWinStyleUnused: WinStyle = 0;
pub type WinSplit = ::core::ffi::c_uint;
pub const kWinSplitBelow: WinSplit = 3;
pub const kWinSplitAbove: WinSplit = 2;
pub const kWinSplitRight: WinSplit = 1;
pub const kWinSplitLeft: WinSplit = 0;
pub type FloatRelative = ::core::ffi::c_uint;
pub const kFloatRelativeLaststatus: FloatRelative = 5;
pub const kFloatRelativeTabline: FloatRelative = 4;
pub const kFloatRelativeMouse: FloatRelative = 3;
pub const kFloatRelativeCursor: FloatRelative = 2;
pub const kFloatRelativeWindow: FloatRelative = 1;
pub const kFloatRelativeEditor: FloatRelative = 0;
pub type FloatAnchor = ::core::ffi::c_int;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lpos_T {
    pub lnum: linenr_T,
    pub col: colnr_T,
}
pub type Window = handle_T;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ScreenGrid {
    pub handle: handle_T,
    pub chars: *mut schar_T,
    pub attrs: *mut sattr_T,
    pub vcols: *mut colnr_T,
    pub line_offset: *mut size_t,
    pub dirty_col: *mut ::core::ffi::c_int,
    pub rows: ::core::ffi::c_int,
    pub cols: ::core::ffi::c_int,
    pub valid: bool,
    pub throttled: bool,
    pub blending: bool,
    pub mouse_enabled: bool,
    pub zindex: ::core::ffi::c_int,
    pub comp_row: ::core::ffi::c_int,
    pub comp_col: ::core::ffi::c_int,
    pub comp_width: ::core::ffi::c_int,
    pub comp_height: ::core::ffi::c_int,
    pub comp_index: size_t,
    pub comp_disabled: bool,
    pub pending_comp_index_update: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct GridView {
    pub target: *mut ScreenGrid,
    pub row_offset: ::core::ffi::c_int,
    pub col_offset: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct taggy_T {
    pub tagname: *mut ::core::ffi::c_char,
    pub fmark: fmark_T,
    pub cur_match: ::core::ffi::c_int,
    pub cur_fnum: ::core::ffi::c_int,
    pub user_data: *mut ::core::ffi::c_char,
}
pub type matchitem_T = matchitem;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct matchitem {
    pub mit_next: *mut matchitem_T,
    pub mit_id: ::core::ffi::c_int,
    pub mit_priority: ::core::ffi::c_int,
    pub mit_pattern: *mut ::core::ffi::c_char,
    pub mit_match: regmmatch_T,
    pub mit_pos_array: *mut llpos_T,
    pub mit_pos_count: ::core::ffi::c_int,
    pub mit_pos_cur: ::core::ffi::c_int,
    pub mit_toplnum: linenr_T,
    pub mit_botlnum: linenr_T,
    pub mit_hl: match_T,
    pub mit_hlg_id: ::core::ffi::c_int,
    pub mit_conceal_char: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct match_T {
    pub rm: regmmatch_T,
    pub buf: *mut buf_T,
    pub lnum: linenr_T,
    pub attr: ::core::ffi::c_int,
    pub attr_cur: ::core::ffi::c_int,
    pub first_lnum: linenr_T,
    pub startcol: colnr_T,
    pub endcol: colnr_T,
    pub is_addpos: bool,
    pub has_cursor: bool,
    pub tm: proftime_T,
}
pub type buf_T = file_buffer;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regmmatch_T {
    pub regprog: *mut regprog_T,
    pub startpos: [lpos_T; 10],
    pub endpos: [lpos_T; 10],
    pub rmm_matchcol: colnr_T,
    pub rmm_ic: ::core::ffi::c_int,
    pub rmm_maxcol: colnr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct llpos_T {
    pub lnum: linenr_T,
    pub col: colnr_T,
    pub len: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct xfmark_T {
    pub fmark: fmark_T,
    pub fname: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct alist_T {
    pub al_ga: garray_T,
    pub al_refcount: ::core::ffi::c_int,
    pub id: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wline_T {
    pub wl_lnum: linenr_T,
    pub wl_size: uint16_t,
    pub wl_valid: bool,
    pub wl_folded: bool,
    pub wl_foldend: linenr_T,
    pub wl_lastlnum: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct pos_save_T {
    pub w_topline_save: ::core::ffi::c_int,
    pub w_topline_corr: ::core::ffi::c_int,
    pub w_cursor_save: pos_T,
    pub w_cursor_corr: pos_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct fcs_chars_T {
    pub stl: schar_T,
    pub stlnc: schar_T,
    pub wbr: schar_T,
    pub horiz: schar_T,
    pub horizup: schar_T,
    pub horizdown: schar_T,
    pub vert: schar_T,
    pub vertleft: schar_T,
    pub vertright: schar_T,
    pub verthoriz: schar_T,
    pub fold: schar_T,
    pub foldopen: schar_T,
    pub foldclosed: schar_T,
    pub foldsep: schar_T,
    pub foldinner: schar_T,
    pub diff: schar_T,
    pub msgsep: schar_T,
    pub eob: schar_T,
    pub lastline: schar_T,
    pub trunc: schar_T,
    pub truncrl: schar_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lcs_chars_T {
    pub eol: schar_T,
    pub ext: schar_T,
    pub prec: schar_T,
    pub nbsp: schar_T,
    pub space: schar_T,
    pub tab1: schar_T,
    pub tab2: schar_T,
    pub tab3: schar_T,
    pub leadtab1: schar_T,
    pub leadtab2: schar_T,
    pub leadtab3: schar_T,
    pub lead: schar_T,
    pub trail: schar_T,
    pub multispace: *mut schar_T,
    pub leadmultispace: *mut schar_T,
    pub conceal: schar_T,
}
pub type frame_T = frame_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct frame_S {
    pub fr_layout: ::core::ffi::c_char,
    pub fr_width: ::core::ffi::c_int,
    pub fr_newwidth: ::core::ffi::c_int,
    pub fr_height: ::core::ffi::c_int,
    pub fr_newheight: ::core::ffi::c_int,
    pub fr_parent: *mut frame_T,
    pub fr_next: *mut frame_T,
    pub fr_prev: *mut frame_T,
    pub fr_child: *mut frame_T,
    pub fr_win: *mut win_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ChangedtickDictItem {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 12],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FileID {
    pub inode: uint64_t,
    pub device_id: uint64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct memline_T {
    pub ml_line_count: linenr_T,
    pub ml_mfp: *mut memfile_T,
    pub ml_stack: *mut infoptr_T,
    pub ml_stack_top: ::core::ffi::c_int,
    pub ml_stack_size: ::core::ffi::c_int,
    pub ml_flags: ::core::ffi::c_int,
    pub ml_line_textlen: colnr_T,
    pub ml_line_lnum: linenr_T,
    pub ml_line_ptr: *mut ::core::ffi::c_char,
    pub ml_line_offset: size_t,
    pub ml_line_offset_ff: ::core::ffi::c_int,
    pub ml_locked: *mut bhdr_T,
    pub ml_locked_low: linenr_T,
    pub ml_locked_high: linenr_T,
    pub ml_locked_lineadd: ::core::ffi::c_int,
    pub ml_chunksize: *mut chunksize_T,
    pub ml_numchunks: ::core::ffi::c_int,
    pub ml_usedchunks: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct chunksize_T {
    pub mlcs_numlines: ::core::ffi::c_int,
    pub mlcs_totalsize: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bhdr_T {
    pub bh_bnum: blocknr_T,
    pub bh_data: *mut ::core::ffi::c_void,
    pub bh_page_count: ::core::ffi::c_uint,
    pub bh_flags: ::core::ffi::c_uint,
}
pub type blocknr_T = int64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct infoptr_T {
    pub ip_bnum: blocknr_T,
    pub ip_low: linenr_T,
    pub ip_high: linenr_T,
    pub ip_index: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct memfile_T {
    pub mf_fname: *mut ::core::ffi::c_char,
    pub mf_ffname: *mut ::core::ffi::c_char,
    pub mf_fd: ::core::ffi::c_int,
    pub mf_flags: ::core::ffi::c_int,
    pub mf_reopen: bool,
    pub mf_free_first: *mut bhdr_T,
    pub mf_hash: Map_int64_t_ptr_t,
    pub mf_trans: Map_int64_t_int64_t,
    pub mf_blocknr_max: blocknr_T,
    pub mf_blocknr_min: blocknr_T,
    pub mf_neg_count: blocknr_T,
    pub mf_infile_count: blocknr_T,
    pub mf_page_size: ::core::ffi::c_uint,
    pub mf_dirty: mfdirty_T,
}
pub type mfdirty_T = ::core::ffi::c_uint;
pub const MF_DIRTY_YES_NOSYNC: mfdirty_T = 2;
pub const MF_DIRTY_YES: mfdirty_T = 1;
pub const MF_DIRTY_NO: mfdirty_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_int64_t_int64_t {
    pub set: Set_int64_t,
    pub values: *mut int64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Set_int64_t {
    pub h: MapHash,
    pub keys: *mut int64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_int64_t_ptr_t {
    pub set: Set_int64_t,
    pub values: *mut ptr_t,
}
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const DO_NOT_FREE_CNT: C2Rust_Unnamed_14 = 1073741823;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dictitem_T {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 0],
}
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const DI_FLAGS_ALLOC: C2Rust_Unnamed_15 = 16;
pub const DI_FLAGS_LOCK: C2Rust_Unnamed_15 = 8;
pub const DI_FLAGS_FIX: C2Rust_Unnamed_15 = 4;
pub const DI_FLAGS_RO_SBX: C2Rust_Unnamed_15 = 2;
pub const DI_FLAGS_RO: C2Rust_Unnamed_15 = 1;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const MAX_FUNC_ARGS: C2Rust_Unnamed_16 = 20;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const VAR_SHORT_LEN: C2Rust_Unnamed_17 = 20;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const FIXVAR_CNT: C2Rust_Unnamed_18 = 12;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ht_stack_S {
    pub ht: *mut hashtab_T,
    pub prev: *mut ht_stack_S,
}
pub type ht_stack_T = ht_stack_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct list_stack_S {
    pub list: *mut list_T,
    pub prev: *mut list_stack_S,
}
pub type list_stack_T = list_stack_S;
pub type Direction = ::core::ffi::c_int;
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub type xp_prefix_T = ::core::ffi::c_uint;
pub const XP_PREFIX_INV: xp_prefix_T = 2;
pub const XP_PREFIX_NO: xp_prefix_T = 1;
pub const XP_PREFIX_NONE: xp_prefix_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct expand_T {
    pub xp_pattern: *mut ::core::ffi::c_char,
    pub xp_context: ::core::ffi::c_int,
    pub xp_pattern_len: size_t,
    pub xp_prefix: xp_prefix_T,
    pub xp_arg: *mut ::core::ffi::c_char,
    pub xp_luaref: LuaRef,
    pub xp_script_ctx: sctx_T,
    pub xp_backslash: ::core::ffi::c_int,
    pub xp_shell: bool,
    pub xp_numfiles: ::core::ffi::c_int,
    pub xp_col: ::core::ffi::c_int,
    pub xp_selected: ::core::ffi::c_int,
    pub xp_orig: *mut ::core::ffi::c_char,
    pub xp_files: *mut *mut ::core::ffi::c_char,
    pub xp_line: *mut ::core::ffi::c_char,
    pub xp_buf: [::core::ffi::c_char; 256],
    pub xp_search_dir: Direction,
    pub xp_pre_incsearch_pos: pos_T,
}
pub type C2Rust_Unnamed_19 = ::core::ffi::c_int;
pub const EXPAND_LSP: C2Rust_Unnamed_19 = 64;
pub const EXPAND_LUA: C2Rust_Unnamed_19 = 63;
pub const EXPAND_CHECKHEALTH: C2Rust_Unnamed_19 = 62;
pub const EXPAND_RETAB: C2Rust_Unnamed_19 = 61;
pub const EXPAND_PATTERN_IN_BUF: C2Rust_Unnamed_19 = 60;
pub const EXPAND_FILETYPECMD: C2Rust_Unnamed_19 = 59;
pub const EXPAND_FINDFUNC: C2Rust_Unnamed_19 = 58;
pub const EXPAND_SHELLCMDLINE: C2Rust_Unnamed_19 = 57;
pub const EXPAND_DIRS_IN_CDPATH: C2Rust_Unnamed_19 = 56;
pub const EXPAND_KEYMAP: C2Rust_Unnamed_19 = 55;
pub const EXPAND_ARGOPT: C2Rust_Unnamed_19 = 54;
pub const EXPAND_SETTING_SUBTRACT: C2Rust_Unnamed_19 = 53;
pub const EXPAND_STRING_SETTING: C2Rust_Unnamed_19 = 52;
pub const EXPAND_RUNTIME: C2Rust_Unnamed_19 = 51;
pub const EXPAND_SCRIPTNAMES: C2Rust_Unnamed_19 = 50;
pub const EXPAND_BREAKPOINT: C2Rust_Unnamed_19 = 49;
pub const EXPAND_DIFF_BUFFERS: C2Rust_Unnamed_19 = 48;
pub const EXPAND_ARGLIST: C2Rust_Unnamed_19 = 47;
pub const EXPAND_MAPCLEAR: C2Rust_Unnamed_19 = 46;
pub const EXPAND_MESSAGES: C2Rust_Unnamed_19 = 45;
pub const EXPAND_PACKADD: C2Rust_Unnamed_19 = 44;
pub const EXPAND_USER_ADDR_TYPE: C2Rust_Unnamed_19 = 43;
pub const EXPAND_SYNTIME: C2Rust_Unnamed_19 = 42;
pub const EXPAND_USER: C2Rust_Unnamed_19 = 41;
pub const EXPAND_HISTORY: C2Rust_Unnamed_19 = 40;
pub const EXPAND_LOCALES: C2Rust_Unnamed_19 = 39;
pub const EXPAND_OWNSYNTAX: C2Rust_Unnamed_19 = 38;
pub const EXPAND_FILES_IN_PATH: C2Rust_Unnamed_19 = 37;
pub const EXPAND_FILETYPE: C2Rust_Unnamed_19 = 36;
pub const EXPAND_PROFILE: C2Rust_Unnamed_19 = 35;
pub const EXPAND_SIGN: C2Rust_Unnamed_19 = 34;
pub const EXPAND_SHELLCMD: C2Rust_Unnamed_19 = 33;
pub const EXPAND_USER_LUA: C2Rust_Unnamed_19 = 32;
pub const EXPAND_USER_LIST: C2Rust_Unnamed_19 = 31;
pub const EXPAND_USER_DEFINED: C2Rust_Unnamed_19 = 30;
pub const EXPAND_COMPILER: C2Rust_Unnamed_19 = 29;
pub const EXPAND_COLORS: C2Rust_Unnamed_19 = 28;
pub const EXPAND_LANGUAGE: C2Rust_Unnamed_19 = 27;
pub const EXPAND_ENV_VARS: C2Rust_Unnamed_19 = 26;
pub const EXPAND_USER_COMPLETE: C2Rust_Unnamed_19 = 25;
pub const EXPAND_USER_NARGS: C2Rust_Unnamed_19 = 24;
pub const EXPAND_USER_CMD_FLAGS: C2Rust_Unnamed_19 = 23;
pub const EXPAND_USER_COMMANDS: C2Rust_Unnamed_19 = 22;
pub const EXPAND_MENUNAMES: C2Rust_Unnamed_19 = 21;
pub const EXPAND_EXPRESSION: C2Rust_Unnamed_19 = 20;
pub const EXPAND_USER_FUNC: C2Rust_Unnamed_19 = 19;
pub const EXPAND_FUNCTIONS: C2Rust_Unnamed_19 = 18;
pub const EXPAND_TAGS_LISTFILES: C2Rust_Unnamed_19 = 17;
pub const EXPAND_MAPPINGS: C2Rust_Unnamed_19 = 16;
pub const EXPAND_USER_VARS: C2Rust_Unnamed_19 = 15;
pub const EXPAND_AUGROUP: C2Rust_Unnamed_19 = 14;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_19 = 13;
pub const EXPAND_SYNTAX: C2Rust_Unnamed_19 = 12;
pub const EXPAND_MENUS: C2Rust_Unnamed_19 = 11;
pub const EXPAND_EVENTS: C2Rust_Unnamed_19 = 10;
pub const EXPAND_BUFFERS: C2Rust_Unnamed_19 = 9;
pub const EXPAND_HELP: C2Rust_Unnamed_19 = 8;
pub const EXPAND_OLD_SETTING: C2Rust_Unnamed_19 = 7;
pub const EXPAND_TAGS: C2Rust_Unnamed_19 = 6;
pub const EXPAND_BOOL_SETTINGS: C2Rust_Unnamed_19 = 5;
pub const EXPAND_SETTINGS: C2Rust_Unnamed_19 = 4;
pub const EXPAND_DIRECTORIES: C2Rust_Unnamed_19 = 3;
pub const EXPAND_FILES: C2Rust_Unnamed_19 = 2;
pub const EXPAND_COMMANDS: C2Rust_Unnamed_19 = 1;
pub const EXPAND_NOTHING: C2Rust_Unnamed_19 = 0;
pub const EXPAND_OK: C2Rust_Unnamed_19 = -1;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_19 = -2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regmatch_T {
    pub regprog: *mut regprog_T,
    pub startp: [*mut ::core::ffi::c_char; 10],
    pub endp: [*mut ::core::ffi::c_char; 10],
    pub rm_matchcol: colnr_T,
    pub rm_ic: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct eslist_elem {
    pub saved_emsg_silent: ::core::ffi::c_int,
    pub next: *mut eslist_T,
}
pub type eslist_T = eslist_elem;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cstack_T {
    pub cs_flags: [::core::ffi::c_int; 50],
    pub cs_pending: [::core::ffi::c_char; 50],
    pub cs_pend: C2Rust_Unnamed_20,
    pub cs_forinfo: [*mut ::core::ffi::c_void; 50],
    pub cs_line: [::core::ffi::c_int; 50],
    pub cs_idx: ::core::ffi::c_int,
    pub cs_looplevel: ::core::ffi::c_int,
    pub cs_trylevel: ::core::ffi::c_int,
    pub cs_emsg_silent_list: *mut eslist_T,
    pub cs_lflags: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_20 {
    pub csp_rv: [*mut ::core::ffi::c_void; 50],
    pub csp_ex: [*mut ::core::ffi::c_void; 50],
}
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const CSTP_FINISH: C2Rust_Unnamed_21 = 32;
pub const CSTP_RETURN: C2Rust_Unnamed_21 = 24;
pub const CSTP_CONTINUE: C2Rust_Unnamed_21 = 16;
pub const CSTP_BREAK: C2Rust_Unnamed_21 = 8;
pub const CSTP_THROW: C2Rust_Unnamed_21 = 4;
pub const CSTP_INTERRUPT: C2Rust_Unnamed_21 = 2;
pub const CSTP_ERROR: C2Rust_Unnamed_21 = 1;
pub const CSTP_NONE: C2Rust_Unnamed_21 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msglist {
    pub next: *mut msglist_T,
    pub msg: *mut ::core::ffi::c_char,
    pub throw_msg: *mut ::core::ffi::c_char,
    pub sfile: *mut ::core::ffi::c_char,
    pub slnum: linenr_T,
    pub multiline: bool,
}
pub type msglist_T = msglist;
pub type except_type_T = ::core::ffi::c_uint;
pub const ET_INTERRUPT: except_type_T = 2;
pub const ET_ERROR: except_type_T = 1;
pub const ET_USER: except_type_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct vim_exception {
    pub type_0: except_type_T,
    pub value: *mut ::core::ffi::c_char,
    pub messages: *mut msglist_T,
    pub throw_name: *mut ::core::ffi::c_char,
    pub throw_lnum: linenr_T,
    pub stacktrace: *mut list_T,
    pub caught: *mut except_T,
}
pub type except_T = vim_exception;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct exception_state_S {
    pub estate_current_exception: *mut except_T,
    pub estate_did_throw: bool,
    pub estate_need_rethrow: bool,
    pub estate_trylevel: ::core::ffi::c_int,
    pub estate_did_emsg: ::core::ffi::c_int,
}
pub type exception_state_T = exception_state_S;
pub type CMD_index = ::core::ffi::c_int;
pub const CMD_USER_BUF: CMD_index = -2;
pub const CMD_USER: CMD_index = -1;
pub const CMD_SIZE: CMD_index = 557;
pub const CMD_Next: CMD_index = 556;
pub const CMD_tilde: CMD_index = 555;
pub const CMD_at: CMD_index = 554;
pub const CMD_rshift: CMD_index = 553;
pub const CMD_equal: CMD_index = 552;
pub const CMD_lshift: CMD_index = 551;
pub const CMD_and: CMD_index = 550;
pub const CMD_pound: CMD_index = 549;
pub const CMD_bang: CMD_index = 548;
pub const CMD_z: CMD_index = 547;
pub const CMD_yank: CMD_index = 546;
pub const CMD_xunmenu: CMD_index = 545;
pub const CMD_xunmap: CMD_index = 544;
pub const CMD_xnoremenu: CMD_index = 543;
pub const CMD_xnoremap: CMD_index = 542;
pub const CMD_xmenu: CMD_index = 541;
pub const CMD_xmapclear: CMD_index = 540;
pub const CMD_xmap: CMD_index = 539;
pub const CMD_xall: CMD_index = 538;
pub const CMD_xit: CMD_index = 537;
pub const CMD_wviminfo: CMD_index = 536;
pub const CMD_wundo: CMD_index = 535;
pub const CMD_wshada: CMD_index = 534;
pub const CMD_wqall: CMD_index = 533;
pub const CMD_wq: CMD_index = 532;
pub const CMD_wprevious: CMD_index = 531;
pub const CMD_wnext: CMD_index = 530;
pub const CMD_winpos: CMD_index = 529;
pub const CMD_windo: CMD_index = 528;
pub const CMD_wincmd: CMD_index = 527;
pub const CMD_winsize: CMD_index = 526;
pub const CMD_while: CMD_index = 525;
pub const CMD_wall: CMD_index = 524;
pub const CMD_wNext: CMD_index = 523;
pub const CMD_write: CMD_index = 522;
pub const CMD_vunmenu: CMD_index = 521;
pub const CMD_vunmap: CMD_index = 520;
pub const CMD_vsplit: CMD_index = 519;
pub const CMD_vnoremenu: CMD_index = 518;
pub const CMD_vnew: CMD_index = 517;
pub const CMD_vnoremap: CMD_index = 516;
pub const CMD_vmenu: CMD_index = 515;
pub const CMD_vmapclear: CMD_index = 514;
pub const CMD_vmap: CMD_index = 513;
pub const CMD_viusage: CMD_index = 512;
pub const CMD_vimgrepadd: CMD_index = 511;
pub const CMD_vimgrep: CMD_index = 510;
pub const CMD_view: CMD_index = 509;
pub const CMD_visual: CMD_index = 508;
pub const CMD_vertical: CMD_index = 507;
pub const CMD_verbose: CMD_index = 506;
pub const CMD_version: CMD_index = 505;
pub const CMD_vglobal: CMD_index = 504;
pub const CMD_update: CMD_index = 503;
pub const CMD_unsilent: CMD_index = 502;
pub const CMD_unmenu: CMD_index = 501;
pub const CMD_unmap: CMD_index = 500;
pub const CMD_unlockvar: CMD_index = 499;
pub const CMD_unlet: CMD_index = 498;
pub const CMD_uniq: CMD_index = 497;
pub const CMD_unhide: CMD_index = 496;
pub const CMD_unabbreviate: CMD_index = 495;
pub const CMD_undolist: CMD_index = 494;
pub const CMD_undojoin: CMD_index = 493;
pub const CMD_undo: CMD_index = 492;
pub const CMD_tunmap: CMD_index = 491;
pub const CMD_tunmenu: CMD_index = 490;
pub const CMD_tselect: CMD_index = 489;
pub const CMD_try: CMD_index = 488;
pub const CMD_trust: CMD_index = 487;
pub const CMD_trewind: CMD_index = 486;
pub const CMD_tprevious: CMD_index = 485;
pub const CMD_topleft: CMD_index = 484;
pub const CMD_tnoremap: CMD_index = 483;
pub const CMD_tnext: CMD_index = 482;
pub const CMD_tmapclear: CMD_index = 481;
pub const CMD_tmap: CMD_index = 480;
pub const CMD_tmenu: CMD_index = 479;
pub const CMD_tlunmenu: CMD_index = 478;
pub const CMD_tlnoremenu: CMD_index = 477;
pub const CMD_tlmenu: CMD_index = 476;
pub const CMD_tlast: CMD_index = 475;
pub const CMD_tjump: CMD_index = 474;
pub const CMD_throw: CMD_index = 473;
pub const CMD_tfirst: CMD_index = 472;
pub const CMD_terminal: CMD_index = 471;
pub const CMD_tclfile: CMD_index = 470;
pub const CMD_tcldo: CMD_index = 469;
pub const CMD_tcl: CMD_index = 468;
pub const CMD_tabs: CMD_index = 467;
pub const CMD_tabrewind: CMD_index = 466;
pub const CMD_tabNext: CMD_index = 465;
pub const CMD_tabprevious: CMD_index = 464;
pub const CMD_tabonly: CMD_index = 463;
pub const CMD_tabnew: CMD_index = 462;
pub const CMD_tabnext: CMD_index = 461;
pub const CMD_tablast: CMD_index = 460;
pub const CMD_tabmove: CMD_index = 459;
pub const CMD_tabfirst: CMD_index = 458;
pub const CMD_tabfind: CMD_index = 457;
pub const CMD_tabedit: CMD_index = 456;
pub const CMD_tabdo: CMD_index = 455;
pub const CMD_tabclose: CMD_index = 454;
pub const CMD_tab: CMD_index = 453;
pub const CMD_tags: CMD_index = 452;
pub const CMD_tag: CMD_index = 451;
pub const CMD_tNext: CMD_index = 450;
pub const CMD_tchdir: CMD_index = 449;
pub const CMD_tcd: CMD_index = 448;
pub const CMD_t: CMD_index = 447;
pub const CMD_syncbind: CMD_index = 446;
pub const CMD_syntime: CMD_index = 445;
pub const CMD_syntax: CMD_index = 444;
pub const CMD_swapname: CMD_index = 443;
pub const CMD_sview: CMD_index = 442;
pub const CMD_suspend: CMD_index = 441;
pub const CMD_sunmenu: CMD_index = 440;
pub const CMD_sunmap: CMD_index = 439;
pub const CMD_sunhide: CMD_index = 438;
pub const CMD_stselect: CMD_index = 437;
pub const CMD_stjump: CMD_index = 436;
pub const CMD_stopinsert: CMD_index = 435;
pub const CMD_startreplace: CMD_index = 434;
pub const CMD_startgreplace: CMD_index = 433;
pub const CMD_startinsert: CMD_index = 432;
pub const CMD_stag: CMD_index = 431;
pub const CMD_stop: CMD_index = 430;
pub const CMD_srewind: CMD_index = 429;
pub const CMD_sprevious: CMD_index = 428;
pub const CMD_spellwrong: CMD_index = 427;
pub const CMD_spellundo: CMD_index = 426;
pub const CMD_spellrare: CMD_index = 425;
pub const CMD_spellrepall: CMD_index = 424;
pub const CMD_spellinfo: CMD_index = 423;
pub const CMD_spelldump: CMD_index = 422;
pub const CMD_spellgood: CMD_index = 421;
pub const CMD_split: CMD_index = 420;
pub const CMD_sort: CMD_index = 419;
pub const CMD_source: CMD_index = 418;
pub const CMD_snoremenu: CMD_index = 417;
pub const CMD_snoremap: CMD_index = 416;
pub const CMD_snomagic: CMD_index = 415;
pub const CMD_snext: CMD_index = 414;
pub const CMD_smenu: CMD_index = 413;
pub const CMD_smapclear: CMD_index = 412;
pub const CMD_smap: CMD_index = 411;
pub const CMD_smagic: CMD_index = 410;
pub const CMD_slast: CMD_index = 409;
pub const CMD_sleep: CMD_index = 408;
pub const CMD_silent: CMD_index = 407;
pub const CMD_sign: CMD_index = 406;
pub const CMD_simalt: CMD_index = 405;
pub const CMD_sfirst: CMD_index = 404;
pub const CMD_sfind: CMD_index = 403;
pub const CMD_setlocal: CMD_index = 402;
pub const CMD_setglobal: CMD_index = 401;
pub const CMD_setfiletype: CMD_index = 400;
pub const CMD_set: CMD_index = 399;
pub const CMD_scriptencoding: CMD_index = 398;
pub const CMD_scriptnames: CMD_index = 397;
pub const CMD_sbrewind: CMD_index = 396;
pub const CMD_sbprevious: CMD_index = 395;
pub const CMD_sbnext: CMD_index = 394;
pub const CMD_sbmodified: CMD_index = 393;
pub const CMD_sblast: CMD_index = 392;
pub const CMD_sbfirst: CMD_index = 391;
pub const CMD_sball: CMD_index = 390;
pub const CMD_sbNext: CMD_index = 389;
pub const CMD_sbuffer: CMD_index = 388;
pub const CMD_saveas: CMD_index = 387;
pub const CMD_sandbox: CMD_index = 386;
pub const CMD_sall: CMD_index = 385;
pub const CMD_sargument: CMD_index = 384;
pub const CMD_sNext: CMD_index = 383;
pub const CMD_substitute: CMD_index = 382;
pub const CMD_rviminfo: CMD_index = 381;
pub const CMD_rubyfile: CMD_index = 380;
pub const CMD_rubydo: CMD_index = 379;
pub const CMD_ruby: CMD_index = 378;
pub const CMD_rundo: CMD_index = 377;
pub const CMD_runtime: CMD_index = 376;
pub const CMD_rshada: CMD_index = 375;
pub const CMD_rightbelow: CMD_index = 374;
pub const CMD_right: CMD_index = 373;
pub const CMD_rewind: CMD_index = 372;
pub const CMD_return: CMD_index = 371;
pub const CMD_retab: CMD_index = 370;
pub const CMD_restart: CMD_index = 369;
pub const CMD_resize: CMD_index = 368;
pub const CMD_registers: CMD_index = 367;
pub const CMD_redrawtabline: CMD_index = 366;
pub const CMD_redrawstatus: CMD_index = 365;
pub const CMD_redraw: CMD_index = 364;
pub const CMD_redir: CMD_index = 363;
pub const CMD_redo: CMD_index = 362;
pub const CMD_recover: CMD_index = 361;
pub const CMD_read: CMD_index = 360;
pub const CMD_qall: CMD_index = 359;
pub const CMD_quitall: CMD_index = 358;
pub const CMD_quit: CMD_index = 357;
pub const CMD_pyxfile: CMD_index = 356;
pub const CMD_pythonx: CMD_index = 355;
pub const CMD_pyxdo: CMD_index = 354;
pub const CMD_pyx: CMD_index = 353;
pub const CMD_py3file: CMD_index = 352;
pub const CMD_python3: CMD_index = 351;
pub const CMD_py3do: CMD_index = 350;
pub const CMD_py3: CMD_index = 349;
pub const CMD_pyfile: CMD_index = 348;
pub const CMD_pydo: CMD_index = 347;
pub const CMD_python: CMD_index = 346;
pub const CMD_pwd: CMD_index = 345;
pub const CMD_put: CMD_index = 344;
pub const CMD_ptselect: CMD_index = 343;
pub const CMD_ptrewind: CMD_index = 342;
pub const CMD_ptprevious: CMD_index = 341;
pub const CMD_ptnext: CMD_index = 340;
pub const CMD_ptlast: CMD_index = 339;
pub const CMD_ptjump: CMD_index = 338;
pub const CMD_ptfirst: CMD_index = 337;
pub const CMD_ptNext: CMD_index = 336;
pub const CMD_ptag: CMD_index = 335;
pub const CMD_psearch: CMD_index = 334;
pub const CMD_profdel: CMD_index = 333;
pub const CMD_profile: CMD_index = 332;
pub const CMD_previous: CMD_index = 331;
pub const CMD_preserve: CMD_index = 330;
pub const CMD_ppop: CMD_index = 329;
pub const CMD_popup: CMD_index = 328;
pub const CMD_pop: CMD_index = 327;
pub const CMD_pedit: CMD_index = 326;
pub const CMD_perlfile: CMD_index = 325;
pub const CMD_perldo: CMD_index = 324;
pub const CMD_perl: CMD_index = 323;
pub const CMD_pclose: CMD_index = 322;
pub const CMD_pbuffer: CMD_index = 321;
pub const CMD_packloadall: CMD_index = 320;
pub const CMD_packadd: CMD_index = 319;
pub const CMD_print: CMD_index = 318;
pub const CMD_ownsyntax: CMD_index = 317;
pub const CMD_ounmenu: CMD_index = 316;
pub const CMD_ounmap: CMD_index = 315;
pub const CMD_options: CMD_index = 314;
pub const CMD_onoremenu: CMD_index = 313;
pub const CMD_onoremap: CMD_index = 312;
pub const CMD_only: CMD_index = 311;
pub const CMD_omenu: CMD_index = 310;
pub const CMD_omapclear: CMD_index = 309;
pub const CMD_omap: CMD_index = 308;
pub const CMD_oldfiles: CMD_index = 307;
pub const CMD_nunmenu: CMD_index = 306;
pub const CMD_nunmap: CMD_index = 305;
pub const CMD_number: CMD_index = 304;
pub const CMD_normal: CMD_index = 303;
pub const CMD_noswapfile: CMD_index = 302;
pub const CMD_noremenu: CMD_index = 301;
pub const CMD_noreabbrev: CMD_index = 300;
pub const CMD_nohlsearch: CMD_index = 299;
pub const CMD_noautocmd: CMD_index = 298;
pub const CMD_noremap: CMD_index = 297;
pub const CMD_nnoremenu: CMD_index = 296;
pub const CMD_nnoremap: CMD_index = 295;
pub const CMD_nmenu: CMD_index = 294;
pub const CMD_nmapclear: CMD_index = 293;
pub const CMD_nmap: CMD_index = 292;
pub const CMD_new: CMD_index = 291;
pub const CMD_next: CMD_index = 290;
pub const CMD_mzfile: CMD_index = 289;
pub const CMD_mzscheme: CMD_index = 288;
pub const CMD_mode: CMD_index = 287;
pub const CMD_mkview: CMD_index = 286;
pub const CMD_mkvimrc: CMD_index = 285;
pub const CMD_mkspell: CMD_index = 284;
pub const CMD_mksession: CMD_index = 283;
pub const CMD_mkexrc: CMD_index = 282;
pub const CMD_messages: CMD_index = 281;
pub const CMD_menutranslate: CMD_index = 280;
pub const CMD_menu: CMD_index = 279;
pub const CMD_match: CMD_index = 278;
pub const CMD_marks: CMD_index = 277;
pub const CMD_mapclear: CMD_index = 276;
pub const CMD_map: CMD_index = 275;
pub const CMD_make: CMD_index = 274;
pub const CMD_mark: CMD_index = 273;
pub const CMD_move: CMD_index = 272;
pub const CMD_lsp: CMD_index = 271;
pub const CMD_ls: CMD_index = 270;
pub const CMD_lwindow: CMD_index = 269;
pub const CMD_lvimgrepadd: CMD_index = 268;
pub const CMD_lvimgrep: CMD_index = 267;
pub const CMD_luafile: CMD_index = 266;
pub const CMD_luado: CMD_index = 265;
pub const CMD_lua: CMD_index = 264;
pub const CMD_lunmap: CMD_index = 263;
pub const CMD_ltag: CMD_index = 262;
pub const CMD_lrewind: CMD_index = 261;
pub const CMD_lpfile: CMD_index = 260;
pub const CMD_lprevious: CMD_index = 259;
pub const CMD_lopen: CMD_index = 258;
pub const CMD_lolder: CMD_index = 257;
pub const CMD_lockvar: CMD_index = 256;
pub const CMD_lockmarks: CMD_index = 255;
pub const CMD_loadkeymap: CMD_index = 254;
pub const CMD_loadview: CMD_index = 253;
pub const CMD_lnfile: CMD_index = 252;
pub const CMD_lnewer: CMD_index = 251;
pub const CMD_lnext: CMD_index = 250;
pub const CMD_lnoremap: CMD_index = 249;
pub const CMD_lmake: CMD_index = 248;
pub const CMD_lmapclear: CMD_index = 247;
pub const CMD_lmap: CMD_index = 246;
pub const CMD_llist: CMD_index = 245;
pub const CMD_llast: CMD_index = 244;
pub const CMD_ll: CMD_index = 243;
pub const CMD_lhistory: CMD_index = 242;
pub const CMD_lhelpgrep: CMD_index = 241;
pub const CMD_lgrepadd: CMD_index = 240;
pub const CMD_lgrep: CMD_index = 239;
pub const CMD_lgetexpr: CMD_index = 238;
pub const CMD_lgetbuffer: CMD_index = 237;
pub const CMD_lgetfile: CMD_index = 236;
pub const CMD_lfirst: CMD_index = 235;
pub const CMD_lfdo: CMD_index = 234;
pub const CMD_lfile: CMD_index = 233;
pub const CMD_lexpr: CMD_index = 232;
pub const CMD_let: CMD_index = 231;
pub const CMD_leftabove: CMD_index = 230;
pub const CMD_left: CMD_index = 229;
pub const CMD_ldo: CMD_index = 228;
pub const CMD_lclose: CMD_index = 227;
pub const CMD_lchdir: CMD_index = 226;
pub const CMD_lcd: CMD_index = 225;
pub const CMD_lbottom: CMD_index = 224;
pub const CMD_lbelow: CMD_index = 223;
pub const CMD_lbefore: CMD_index = 222;
pub const CMD_lbuffer: CMD_index = 221;
pub const CMD_later: CMD_index = 220;
pub const CMD_lafter: CMD_index = 219;
pub const CMD_laddfile: CMD_index = 218;
pub const CMD_laddbuffer: CMD_index = 217;
pub const CMD_laddexpr: CMD_index = 216;
pub const CMD_language: CMD_index = 215;
pub const CMD_labove: CMD_index = 214;
pub const CMD_last: CMD_index = 213;
pub const CMD_lNfile: CMD_index = 212;
pub const CMD_lNext: CMD_index = 211;
pub const CMD_list: CMD_index = 210;
pub const CMD_keepalt: CMD_index = 209;
pub const CMD_keeppatterns: CMD_index = 208;
pub const CMD_keepjumps: CMD_index = 207;
pub const CMD_keepmarks: CMD_index = 206;
pub const CMD_k: CMD_index = 205;
pub const CMD_jumps: CMD_index = 204;
pub const CMD_join: CMD_index = 203;
pub const CMD_iunmenu: CMD_index = 202;
pub const CMD_iunabbrev: CMD_index = 201;
pub const CMD_iunmap: CMD_index = 200;
pub const CMD_isplit: CMD_index = 199;
pub const CMD_isearch: CMD_index = 198;
pub const CMD_iput: CMD_index = 197;
pub const CMD_intro: CMD_index = 196;
pub const CMD_inoremenu: CMD_index = 195;
pub const CMD_inoreabbrev: CMD_index = 194;
pub const CMD_inoremap: CMD_index = 193;
pub const CMD_imenu: CMD_index = 192;
pub const CMD_imapclear: CMD_index = 191;
pub const CMD_imap: CMD_index = 190;
pub const CMD_ilist: CMD_index = 189;
pub const CMD_ijump: CMD_index = 188;
pub const CMD_if: CMD_index = 187;
pub const CMD_iabclear: CMD_index = 186;
pub const CMD_iabbrev: CMD_index = 185;
pub const CMD_insert: CMD_index = 184;
pub const CMD_horizontal: CMD_index = 183;
pub const CMD_history: CMD_index = 182;
pub const CMD_hide: CMD_index = 181;
pub const CMD_highlight: CMD_index = 180;
pub const CMD_helptags: CMD_index = 179;
pub const CMD_helpgrep: CMD_index = 178;
pub const CMD_helpclose: CMD_index = 177;
pub const CMD_help: CMD_index = 176;
pub const CMD_gvim: CMD_index = 175;
pub const CMD_gui: CMD_index = 174;
pub const CMD_grepadd: CMD_index = 173;
pub const CMD_grep: CMD_index = 172;
pub const CMD_goto: CMD_index = 171;
pub const CMD_global: CMD_index = 170;
pub const CMD_fclose: CMD_index = 169;
pub const CMD_function: CMD_index = 168;
pub const CMD_for: CMD_index = 167;
pub const CMD_foldopen: CMD_index = 166;
pub const CMD_folddoclosed: CMD_index = 165;
pub const CMD_folddoopen: CMD_index = 164;
pub const CMD_foldclose: CMD_index = 163;
pub const CMD_fold: CMD_index = 162;
pub const CMD_first: CMD_index = 161;
pub const CMD_finish: CMD_index = 160;
pub const CMD_finally: CMD_index = 159;
pub const CMD_find: CMD_index = 158;
pub const CMD_filter: CMD_index = 157;
pub const CMD_filetype: CMD_index = 156;
pub const CMD_files: CMD_index = 155;
pub const CMD_file: CMD_index = 154;
pub const CMD_exusage: CMD_index = 153;
pub const CMD_exit: CMD_index = 152;
pub const CMD_execute: CMD_index = 151;
pub const CMD_ex: CMD_index = 150;
pub const CMD_eval: CMD_index = 149;
pub const CMD_enew: CMD_index = 148;
pub const CMD_endwhile: CMD_index = 147;
pub const CMD_endtry: CMD_index = 146;
pub const CMD_endfor: CMD_index = 145;
pub const CMD_endfunction: CMD_index = 144;
pub const CMD_endif: CMD_index = 143;
pub const CMD_emenu: CMD_index = 142;
pub const CMD_elseif: CMD_index = 141;
pub const CMD_else: CMD_index = 140;
pub const CMD_echon: CMD_index = 139;
pub const CMD_echomsg: CMD_index = 138;
pub const CMD_echohl: CMD_index = 137;
pub const CMD_echoerr: CMD_index = 136;
pub const CMD_echo: CMD_index = 135;
pub const CMD_earlier: CMD_index = 134;
pub const CMD_edit: CMD_index = 133;
pub const CMD_dsplit: CMD_index = 132;
pub const CMD_dsearch: CMD_index = 131;
pub const CMD_drop: CMD_index = 130;
pub const CMD_doautoall: CMD_index = 129;
pub const CMD_doautocmd: CMD_index = 128;
pub const CMD_dlist: CMD_index = 127;
pub const CMD_djump: CMD_index = 126;
pub const CMD_digraphs: CMD_index = 125;
pub const CMD_diffthis: CMD_index = 124;
pub const CMD_diffsplit: CMD_index = 123;
pub const CMD_diffput: CMD_index = 122;
pub const CMD_diffpatch: CMD_index = 121;
pub const CMD_diffoff: CMD_index = 120;
pub const CMD_diffget: CMD_index = 119;
pub const CMD_diffupdate: CMD_index = 118;
pub const CMD_display: CMD_index = 117;
pub const CMD_detach: CMD_index = 116;
pub const CMD_delfunction: CMD_index = 115;
pub const CMD_delcommand: CMD_index = 114;
pub const CMD_defer: CMD_index = 113;
pub const CMD_debuggreedy: CMD_index = 112;
pub const CMD_debug: CMD_index = 111;
pub const CMD_delmarks: CMD_index = 110;
pub const CMD_delete: CMD_index = 109;
pub const CMD_cwindow: CMD_index = 108;
pub const CMD_cunmenu: CMD_index = 107;
pub const CMD_cunabbrev: CMD_index = 106;
pub const CMD_cunmap: CMD_index = 105;
pub const CMD_crewind: CMD_index = 104;
pub const CMD_cquit: CMD_index = 103;
pub const CMD_cpfile: CMD_index = 102;
pub const CMD_cprevious: CMD_index = 101;
pub const CMD_copen: CMD_index = 100;
pub const CMD_const: CMD_index = 99;
pub const CMD_connect: CMD_index = 98;
pub const CMD_confirm: CMD_index = 97;
pub const CMD_continue: CMD_index = 96;
pub const CMD_compiler: CMD_index = 95;
pub const CMD_comclear: CMD_index = 94;
pub const CMD_command: CMD_index = 93;
pub const CMD_colorscheme: CMD_index = 92;
pub const CMD_colder: CMD_index = 91;
pub const CMD_copy: CMD_index = 90;
pub const CMD_cnoremenu: CMD_index = 89;
pub const CMD_cnoreabbrev: CMD_index = 88;
pub const CMD_cnoremap: CMD_index = 87;
pub const CMD_cnfile: CMD_index = 86;
pub const CMD_cnewer: CMD_index = 85;
pub const CMD_cnext: CMD_index = 84;
pub const CMD_cmenu: CMD_index = 83;
pub const CMD_cmapclear: CMD_index = 82;
pub const CMD_cmap: CMD_index = 81;
pub const CMD_clearjumps: CMD_index = 80;
pub const CMD_close: CMD_index = 79;
pub const CMD_clast: CMD_index = 78;
pub const CMD_clist: CMD_index = 77;
pub const CMD_chistory: CMD_index = 76;
pub const CMD_checktime: CMD_index = 75;
pub const CMD_checkpath: CMD_index = 74;
pub const CMD_checkhealth: CMD_index = 73;
pub const CMD_changes: CMD_index = 72;
pub const CMD_chdir: CMD_index = 71;
pub const CMD_cgetexpr: CMD_index = 70;
pub const CMD_cgetbuffer: CMD_index = 69;
pub const CMD_cgetfile: CMD_index = 68;
pub const CMD_cfirst: CMD_index = 67;
pub const CMD_cfdo: CMD_index = 66;
pub const CMD_cfile: CMD_index = 65;
pub const CMD_cexpr: CMD_index = 64;
pub const CMD_center: CMD_index = 63;
pub const CMD_cdo: CMD_index = 62;
pub const CMD_cd: CMD_index = 61;
pub const CMD_cclose: CMD_index = 60;
pub const CMD_cc: CMD_index = 59;
pub const CMD_cbottom: CMD_index = 58;
pub const CMD_cbelow: CMD_index = 57;
pub const CMD_cbefore: CMD_index = 56;
pub const CMD_cbuffer: CMD_index = 55;
pub const CMD_catch: CMD_index = 54;
pub const CMD_call: CMD_index = 53;
pub const CMD_cafter: CMD_index = 52;
pub const CMD_caddfile: CMD_index = 51;
pub const CMD_caddexpr: CMD_index = 50;
pub const CMD_caddbuffer: CMD_index = 49;
pub const CMD_cabove: CMD_index = 48;
pub const CMD_cabclear: CMD_index = 47;
pub const CMD_cabbrev: CMD_index = 46;
pub const CMD_cNfile: CMD_index = 45;
pub const CMD_cNext: CMD_index = 44;
pub const CMD_change: CMD_index = 43;
pub const CMD_bwipeout: CMD_index = 42;
pub const CMD_bunload: CMD_index = 41;
pub const CMD_bufdo: CMD_index = 40;
pub const CMD_buffers: CMD_index = 39;
pub const CMD_browse: CMD_index = 38;
pub const CMD_breaklist: CMD_index = 37;
pub const CMD_breakdel: CMD_index = 36;
pub const CMD_breakadd: CMD_index = 35;
pub const CMD_break: CMD_index = 34;
pub const CMD_brewind: CMD_index = 33;
pub const CMD_bprevious: CMD_index = 32;
pub const CMD_botright: CMD_index = 31;
pub const CMD_bnext: CMD_index = 30;
pub const CMD_bmodified: CMD_index = 29;
pub const CMD_blast: CMD_index = 28;
pub const CMD_bfirst: CMD_index = 27;
pub const CMD_belowright: CMD_index = 26;
pub const CMD_bdelete: CMD_index = 25;
pub const CMD_balt: CMD_index = 24;
pub const CMD_badd: CMD_index = 23;
pub const CMD_ball: CMD_index = 22;
pub const CMD_bNext: CMD_index = 21;
pub const CMD_buffer: CMD_index = 20;
pub const CMD_aunmenu: CMD_index = 19;
pub const CMD_augroup: CMD_index = 18;
pub const CMD_autocmd: CMD_index = 17;
pub const CMD_ascii: CMD_index = 16;
pub const CMD_argument: CMD_index = 15;
pub const CMD_arglocal: CMD_index = 14;
pub const CMD_argglobal: CMD_index = 13;
pub const CMD_argedit: CMD_index = 12;
pub const CMD_argdedupe: CMD_index = 11;
pub const CMD_argdo: CMD_index = 10;
pub const CMD_argdelete: CMD_index = 9;
pub const CMD_argadd: CMD_index = 8;
pub const CMD_args: CMD_index = 7;
pub const CMD_anoremenu: CMD_index = 6;
pub const CMD_amenu: CMD_index = 5;
pub const CMD_all: CMD_index = 4;
pub const CMD_aboveleft: CMD_index = 3;
pub const CMD_abclear: CMD_index = 2;
pub const CMD_abbreviate: CMD_index = 1;
pub const CMD_append: CMD_index = 0;
pub type cmdidx_T = CMD_index;
pub type cmd_addr_T = ::core::ffi::c_uint;
pub const ADDR_NONE: cmd_addr_T = 11;
pub const ADDR_OTHER: cmd_addr_T = 10;
pub const ADDR_UNSIGNED: cmd_addr_T = 9;
pub const ADDR_QUICKFIX: cmd_addr_T = 8;
pub const ADDR_QUICKFIX_VALID: cmd_addr_T = 7;
pub const ADDR_TABS_RELATIVE: cmd_addr_T = 6;
pub const ADDR_TABS: cmd_addr_T = 5;
pub const ADDR_BUFFERS: cmd_addr_T = 4;
pub const ADDR_LOADED_BUFFERS: cmd_addr_T = 3;
pub const ADDR_ARGUMENTS: cmd_addr_T = 2;
pub const ADDR_WINDOWS: cmd_addr_T = 1;
pub const ADDR_LINES: cmd_addr_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct exarg {
    pub arg: *mut ::core::ffi::c_char,
    pub args: *mut *mut ::core::ffi::c_char,
    pub arglens: *mut size_t,
    pub argc: size_t,
    pub nextcmd: *mut ::core::ffi::c_char,
    pub cmd: *mut ::core::ffi::c_char,
    pub cmdlinep: *mut *mut ::core::ffi::c_char,
    pub cmdline_tofree: *mut ::core::ffi::c_char,
    pub cmdidx: cmdidx_T,
    pub argt: uint32_t,
    pub skip: ::core::ffi::c_int,
    pub forceit: ::core::ffi::c_int,
    pub addr_count: ::core::ffi::c_int,
    pub line1: linenr_T,
    pub line2: linenr_T,
    pub addr_type: cmd_addr_T,
    pub flags: ::core::ffi::c_int,
    pub do_ecmd_cmd: *mut ::core::ffi::c_char,
    pub do_ecmd_lnum: linenr_T,
    pub append: ::core::ffi::c_int,
    pub usefilter: ::core::ffi::c_int,
    pub amount: ::core::ffi::c_int,
    pub regname: ::core::ffi::c_int,
    pub force_bin: ::core::ffi::c_int,
    pub read_edit: ::core::ffi::c_int,
    pub mkdir_p: ::core::ffi::c_int,
    pub force_ff: ::core::ffi::c_int,
    pub force_enc: ::core::ffi::c_int,
    pub bad_char: ::core::ffi::c_int,
    pub useridx: ::core::ffi::c_int,
    pub errmsg: *mut ::core::ffi::c_char,
    pub ea_getline: LineGetter,
    pub cookie: *mut ::core::ffi::c_void,
    pub cstack: *mut cstack_T,
}
pub type LineGetter = Option<
    unsafe extern "C" fn(
        ::core::ffi::c_int,
        *mut ::core::ffi::c_void,
        ::core::ffi::c_int,
        bool,
    ) -> *mut ::core::ffi::c_char,
>;
pub type exarg_T = exarg;
pub type auto_event = ::core::ffi::c_uint;
pub const NUM_EVENTS: auto_event = 145;
pub const EVENT_WINSCROLLED: auto_event = 144;
pub const EVENT_WINRESIZED: auto_event = 143;
pub const EVENT_WINNEWPRE: auto_event = 142;
pub const EVENT_WINNEW: auto_event = 141;
pub const EVENT_WINLEAVE: auto_event = 140;
pub const EVENT_WINENTER: auto_event = 139;
pub const EVENT_WINCLOSED: auto_event = 138;
pub const EVENT_VIMSUSPEND: auto_event = 137;
pub const EVENT_VIMRESUME: auto_event = 136;
pub const EVENT_VIMRESIZED: auto_event = 135;
pub const EVENT_VIMLEAVEPRE: auto_event = 134;
pub const EVENT_VIMLEAVE: auto_event = 133;
pub const EVENT_VIMENTER: auto_event = 132;
pub const EVENT_USER: auto_event = 131;
pub const EVENT_UILEAVE: auto_event = 130;
pub const EVENT_UIENTER: auto_event = 129;
pub const EVENT_TEXTYANKPOST: auto_event = 128;
pub const EVENT_TEXTCHANGEDT: auto_event = 127;
pub const EVENT_TEXTCHANGEDP: auto_event = 126;
pub const EVENT_TEXTCHANGEDI: auto_event = 125;
pub const EVENT_TEXTCHANGED: auto_event = 124;
pub const EVENT_TERMRESPONSE: auto_event = 123;
pub const EVENT_TERMREQUEST: auto_event = 122;
pub const EVENT_TERMOPEN: auto_event = 121;
pub const EVENT_TERMLEAVE: auto_event = 120;
pub const EVENT_TERMENTER: auto_event = 119;
pub const EVENT_TERMCLOSE: auto_event = 118;
pub const EVENT_TERMCHANGED: auto_event = 117;
pub const EVENT_TABNEWENTERED: auto_event = 116;
pub const EVENT_TABNEW: auto_event = 115;
pub const EVENT_TABLEAVE: auto_event = 114;
pub const EVENT_TABENTER: auto_event = 113;
pub const EVENT_TABCLOSEDPRE: auto_event = 112;
pub const EVENT_TABCLOSED: auto_event = 111;
pub const EVENT_SYNTAX: auto_event = 110;
pub const EVENT_SWAPEXISTS: auto_event = 109;
pub const EVENT_STDINREADPRE: auto_event = 108;
pub const EVENT_STDINREADPOST: auto_event = 107;
pub const EVENT_SPELLFILEMISSING: auto_event = 106;
pub const EVENT_SOURCEPRE: auto_event = 105;
pub const EVENT_SOURCEPOST: auto_event = 104;
pub const EVENT_SOURCECMD: auto_event = 103;
pub const EVENT_SIGNAL: auto_event = 102;
pub const EVENT_SHELLFILTERPOST: auto_event = 101;
pub const EVENT_SHELLCMDPOST: auto_event = 100;
pub const EVENT_SESSIONWRITEPOST: auto_event = 99;
pub const EVENT_SESSIONLOADPRE: auto_event = 98;
pub const EVENT_SESSIONLOADPOST: auto_event = 97;
pub const EVENT_SEARCHWRAPPED: auto_event = 96;
pub const EVENT_SAFESTATE: auto_event = 95;
pub const EVENT_REMOTEREPLY: auto_event = 94;
pub const EVENT_RECORDINGLEAVE: auto_event = 93;
pub const EVENT_RECORDINGENTER: auto_event = 92;
pub const EVENT_QUITPRE: auto_event = 91;
pub const EVENT_QUICKFIXCMDPRE: auto_event = 90;
pub const EVENT_QUICKFIXCMDPOST: auto_event = 89;
pub const EVENT_PROGRESS: auto_event = 88;
pub const EVENT_PACKCHANGEDPRE: auto_event = 87;
pub const EVENT_PACKCHANGED: auto_event = 86;
pub const EVENT_OPTIONSET: auto_event = 85;
pub const EVENT_MODECHANGED: auto_event = 84;
pub const EVENT_MENUPOPUP: auto_event = 83;
pub const EVENT_MARKSET: auto_event = 82;
pub const EVENT_LSPTOKENUPDATE: auto_event = 81;
pub const EVENT_LSPREQUEST: auto_event = 80;
pub const EVENT_LSPPROGRESS: auto_event = 79;
pub const EVENT_LSPNOTIFY: auto_event = 78;
pub const EVENT_LSPDETACH: auto_event = 77;
pub const EVENT_LSPATTACH: auto_event = 76;
pub const EVENT_INSERTLEAVEPRE: auto_event = 75;
pub const EVENT_INSERTLEAVE: auto_event = 74;
pub const EVENT_INSERTENTER: auto_event = 73;
pub const EVENT_INSERTCHARPRE: auto_event = 72;
pub const EVENT_INSERTCHANGE: auto_event = 71;
pub const EVENT_GUIFAILED: auto_event = 70;
pub const EVENT_GUIENTER: auto_event = 69;
pub const EVENT_FUNCUNDEFINED: auto_event = 68;
pub const EVENT_FOCUSLOST: auto_event = 67;
pub const EVENT_FOCUSGAINED: auto_event = 66;
pub const EVENT_FILTERWRITEPRE: auto_event = 65;
pub const EVENT_FILTERWRITEPOST: auto_event = 64;
pub const EVENT_FILTERREADPRE: auto_event = 63;
pub const EVENT_FILTERREADPOST: auto_event = 62;
pub const EVENT_FILEWRITEPRE: auto_event = 61;
pub const EVENT_FILEWRITEPOST: auto_event = 60;
pub const EVENT_FILEWRITECMD: auto_event = 59;
pub const EVENT_FILETYPE: auto_event = 58;
pub const EVENT_FILEREADPRE: auto_event = 57;
pub const EVENT_FILEREADPOST: auto_event = 56;
pub const EVENT_FILEREADCMD: auto_event = 55;
pub const EVENT_FILEENCODING: auto_event = 54;
pub const EVENT_FILECHANGEDSHELLPOST: auto_event = 53;
pub const EVENT_FILECHANGEDSHELL: auto_event = 52;
pub const EVENT_FILECHANGEDRO: auto_event = 51;
pub const EVENT_FILEAPPENDPRE: auto_event = 50;
pub const EVENT_FILEAPPENDPOST: auto_event = 49;
pub const EVENT_FILEAPPENDCMD: auto_event = 48;
pub const EVENT_EXITPRE: auto_event = 47;
pub const EVENT_ENCODINGCHANGED: auto_event = 46;
pub const EVENT_DIRCHANGEDPRE: auto_event = 45;
pub const EVENT_DIRCHANGED: auto_event = 44;
pub const EVENT_DIFFUPDATED: auto_event = 43;
pub const EVENT_DIAGNOSTICCHANGED: auto_event = 42;
pub const EVENT_CURSORMOVEDI: auto_event = 41;
pub const EVENT_CURSORMOVEDC: auto_event = 40;
pub const EVENT_CURSORMOVED: auto_event = 39;
pub const EVENT_CURSORHOLDI: auto_event = 38;
pub const EVENT_CURSORHOLD: auto_event = 37;
pub const EVENT_COMPLETEDONEPRE: auto_event = 36;
pub const EVENT_COMPLETEDONE: auto_event = 35;
pub const EVENT_COMPLETECHANGED: auto_event = 34;
pub const EVENT_COLORSCHEMEPRE: auto_event = 33;
pub const EVENT_COLORSCHEME: auto_event = 32;
pub const EVENT_CMDWINLEAVE: auto_event = 31;
pub const EVENT_CMDWINENTER: auto_event = 30;
pub const EVENT_CMDUNDEFINED: auto_event = 29;
pub const EVENT_CMDLINELEAVEPRE: auto_event = 28;
pub const EVENT_CMDLINELEAVE: auto_event = 27;
pub const EVENT_CMDLINEENTER: auto_event = 26;
pub const EVENT_CMDLINECHANGED: auto_event = 25;
pub const EVENT_CHANOPEN: auto_event = 24;
pub const EVENT_CHANINFO: auto_event = 23;
pub const EVENT_BUFWRITEPRE: auto_event = 22;
pub const EVENT_BUFWRITEPOST: auto_event = 21;
pub const EVENT_BUFWRITECMD: auto_event = 20;
pub const EVENT_BUFWRITE: auto_event = 19;
pub const EVENT_BUFWIPEOUT: auto_event = 18;
pub const EVENT_BUFWINLEAVE: auto_event = 17;
pub const EVENT_BUFWINENTER: auto_event = 16;
pub const EVENT_BUFUNLOAD: auto_event = 15;
pub const EVENT_BUFREADPRE: auto_event = 14;
pub const EVENT_BUFREADPOST: auto_event = 13;
pub const EVENT_BUFREADCMD: auto_event = 12;
pub const EVENT_BUFREAD: auto_event = 11;
pub const EVENT_BUFNEWFILE: auto_event = 10;
pub const EVENT_BUFNEW: auto_event = 9;
pub const EVENT_BUFMODIFIEDSET: auto_event = 8;
pub const EVENT_BUFLEAVE: auto_event = 7;
pub const EVENT_BUFHIDDEN: auto_event = 6;
pub const EVENT_BUFFILEPRE: auto_event = 5;
pub const EVENT_BUFFILEPOST: auto_event = 4;
pub const EVENT_BUFENTER: auto_event = 3;
pub const EVENT_BUFDELETE: auto_event = 2;
pub const EVENT_BUFCREATE: auto_event = 1;
pub const EVENT_BUFADD: auto_event = 0;
pub type event_T = auto_event;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct AutoPat {
    pub refcount: size_t,
    pub pat: *mut ::core::ffi::c_char,
    pub reg_prog: *mut regprog_T,
    pub group: ::core::ffi::c_int,
    pub patlen: ::core::ffi::c_int,
    pub buflocal_nr: ::core::ffi::c_int,
    pub allow_dirs: ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct AutoPatCmd_S {
    pub lastpat: *mut AutoPat,
    pub auidx: size_t,
    pub ausize: size_t,
    pub afile_orig: *mut ::core::ffi::c_char,
    pub fname: *mut ::core::ffi::c_char,
    pub sfname: *mut ::core::ffi::c_char,
    pub tail: *mut ::core::ffi::c_char,
    pub group: ::core::ffi::c_int,
    pub event: event_T,
    pub script_ctx: sctx_T,
    pub arg_bufnr: ::core::ffi::c_int,
    pub data: *mut Object,
    pub next: *mut AutoPatCmd,
}
pub type AutoPatCmd = AutoPatCmd_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct evalarg_T {
    pub eval_flags: ::core::ffi::c_int,
    pub eval_getline: LineGetter,
    pub eval_cookie: *mut ::core::ffi::c_void,
    pub eval_tofree: *mut ::core::ffi::c_char,
}
pub type VimVarIndex = ::core::ffi::c_uint;
pub const VV_EXITREASON: VimVarIndex = 105;
pub const VV_STARTTIME: VimVarIndex = 104;
pub const VV_VIRTNUM: VimVarIndex = 103;
pub const VV_RELNUM: VimVarIndex = 102;
pub const VV_LUA: VimVarIndex = 101;
pub const VV__NULL_BLOB: VimVarIndex = 100;
pub const VV__NULL_DICT: VimVarIndex = 99;
pub const VV__NULL_LIST: VimVarIndex = 98;
pub const VV__NULL_STRING: VimVarIndex = 97;
pub const VV_MSGPACK_TYPES: VimVarIndex = 96;
pub const VV_STDERR: VimVarIndex = 95;
pub const VV_VIM_DID_INIT: VimVarIndex = 94;
pub const VV_STACKTRACE: VimVarIndex = 93;
pub const VV_MAXCOL: VimVarIndex = 92;
pub const VV_EXITING: VimVarIndex = 91;
pub const VV_COLLATE: VimVarIndex = 90;
pub const VV_ARGV: VimVarIndex = 89;
pub const VV_ARGF: VimVarIndex = 88;
pub const VV_ECHOSPACE: VimVarIndex = 87;
pub const VV_VERSIONLONG: VimVarIndex = 86;
pub const VV_EVENT: VimVarIndex = 85;
pub const VV_TYPE_BLOB: VimVarIndex = 84;
pub const VV_TYPE_BOOL: VimVarIndex = 83;
pub const VV_TYPE_FLOAT: VimVarIndex = 82;
pub const VV_TYPE_DICT: VimVarIndex = 81;
pub const VV_TYPE_LIST: VimVarIndex = 80;
pub const VV_TYPE_FUNC: VimVarIndex = 79;
pub const VV_TYPE_STRING: VimVarIndex = 78;
pub const VV_TYPE_NUMBER: VimVarIndex = 77;
pub const VV_TESTING: VimVarIndex = 76;
pub const VV_VIM_DID_ENTER: VimVarIndex = 75;
pub const VV_NUMBERSIZE: VimVarIndex = 74;
pub const VV_NUMBERMIN: VimVarIndex = 73;
pub const VV_NUMBERMAX: VimVarIndex = 72;
pub const VV_NULL: VimVarIndex = 71;
pub const VV_TRUE: VimVarIndex = 70;
pub const VV_FALSE: VimVarIndex = 69;
pub const VV_ERRORS: VimVarIndex = 68;
pub const VV_OPTION_TYPE: VimVarIndex = 67;
pub const VV_OPTION_COMMAND: VimVarIndex = 66;
pub const VV_OPTION_OLDGLOBAL: VimVarIndex = 65;
pub const VV_OPTION_OLDLOCAL: VimVarIndex = 64;
pub const VV_OPTION_OLD: VimVarIndex = 63;
pub const VV_OPTION_NEW: VimVarIndex = 62;
pub const VV_COMPLETED_ITEM: VimVarIndex = 61;
pub const VV_PROGPATH: VimVarIndex = 60;
pub const VV_WINDOWID: VimVarIndex = 59;
pub const VV_OLDFILES: VimVarIndex = 58;
pub const VV_HLSEARCH: VimVarIndex = 57;
pub const VV_SEARCHFORWARD: VimVarIndex = 56;
pub const VV_OP: VimVarIndex = 55;
pub const VV_MOUSE_COL: VimVarIndex = 54;
pub const VV_MOUSE_LNUM: VimVarIndex = 53;
pub const VV_MOUSE_WINID: VimVarIndex = 52;
pub const VV_MOUSE_WIN: VimVarIndex = 51;
pub const VV_CHAR: VimVarIndex = 50;
pub const VV_SWAPCOMMAND: VimVarIndex = 49;
pub const VV_SWAPCHOICE: VimVarIndex = 48;
pub const VV_SWAPNAME: VimVarIndex = 47;
pub const VV_SCROLLSTART: VimVarIndex = 46;
pub const VV_BEVAL_TEXT: VimVarIndex = 45;
pub const VV_BEVAL_COL: VimVarIndex = 44;
pub const VV_BEVAL_LNUM: VimVarIndex = 43;
pub const VV_BEVAL_WINID: VimVarIndex = 42;
pub const VV_BEVAL_WINNR: VimVarIndex = 41;
pub const VV_BEVAL_BUFNR: VimVarIndex = 40;
pub const VV_FCS_CHOICE: VimVarIndex = 39;
pub const VV_FCS_REASON: VimVarIndex = 38;
pub const VV_PROFILING: VimVarIndex = 37;
pub const VV_KEY: VimVarIndex = 36;
pub const VV_VAL: VimVarIndex = 35;
pub const VV_INSERTMODE: VimVarIndex = 34;
pub const VV_CMDBANG: VimVarIndex = 33;
pub const VV_REG: VimVarIndex = 32;
pub const VV_THROWPOINT: VimVarIndex = 31;
pub const VV_EXCEPTION: VimVarIndex = 30;
pub const VV_DYING: VimVarIndex = 29;
pub const VV_SEND_SERVER: VimVarIndex = 28;
pub const VV_PROGNAME: VimVarIndex = 27;
pub const VV_FOLDLEVEL: VimVarIndex = 26;
pub const VV_FOLDDASHES: VimVarIndex = 25;
pub const VV_FOLDEND: VimVarIndex = 24;
pub const VV_FOLDSTART: VimVarIndex = 23;
pub const VV_CMDARG: VimVarIndex = 22;
pub const VV_FNAME_DIFF: VimVarIndex = 21;
pub const VV_FNAME_NEW: VimVarIndex = 20;
pub const VV_FNAME_OUT: VimVarIndex = 19;
pub const VV_FNAME_IN: VimVarIndex = 18;
pub const VV_CC_TO: VimVarIndex = 17;
pub const VV_CC_FROM: VimVarIndex = 16;
pub const VV_CTYPE: VimVarIndex = 15;
pub const VV_LC_TIME: VimVarIndex = 14;
pub const VV_LANG: VimVarIndex = 13;
pub const VV_FNAME: VimVarIndex = 12;
pub const VV_TERMRESPONSE: VimVarIndex = 11;
pub const VV_TERMREQUEST: VimVarIndex = 10;
pub const VV_LNUM: VimVarIndex = 9;
pub const VV_VERSION: VimVarIndex = 8;
pub const VV_THIS_SESSION: VimVarIndex = 7;
pub const VV_SHELL_ERROR: VimVarIndex = 6;
pub const VV_STATUSMSG: VimVarIndex = 5;
pub const VV_WARNINGMSG: VimVarIndex = 4;
pub const VV_ERRMSG: VimVarIndex = 3;
pub const VV_PREVCOUNT: VimVarIndex = 2;
pub const VV_COUNT1: VimVarIndex = 1;
pub const VV_COUNT: VimVarIndex = 0;
pub type UIExtension = ::core::ffi::c_uint;
pub const kUIExtCount: UIExtension = 10;
pub const kUIFloatDebug: UIExtension = 9;
pub const kUITermColors: UIExtension = 8;
pub const kUIHlState: UIExtension = 7;
pub const kUIMultigrid: UIExtension = 6;
pub const kUILinegrid: UIExtension = 5;
pub const kUIMessages: UIExtension = 4;
pub const kUIWildmenu: UIExtension = 3;
pub const kUITabline: UIExtension = 2;
pub const kUIPopupmenu: UIExtension = 1;
pub const kUICmdline: UIExtension = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lval_T {
    pub ll_name: *const ::core::ffi::c_char,
    pub ll_name_len: size_t,
    pub ll_exp_name: *mut ::core::ffi::c_char,
    pub ll_tv: *mut typval_T,
    pub ll_li: *mut listitem_T,
    pub ll_list: *mut list_T,
    pub ll_range: bool,
    pub ll_empty2: bool,
    pub ll_n1: ::core::ffi::c_int,
    pub ll_n2: ::core::ffi::c_int,
    pub ll_dict: *mut dict_T,
    pub ll_di: *mut dictitem_T,
    pub ll_newkey: *mut ::core::ffi::c_char,
    pub ll_blob: *mut blob_T,
}
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const TFN_READ_ONLY: C2Rust_Unnamed_22 = 16;
pub const TFN_NO_DEREF: C2Rust_Unnamed_22 = 8;
pub const TFN_NO_AUTOLOAD: C2Rust_Unnamed_22 = 4;
pub const TFN_QUIET: C2Rust_Unnamed_22 = 2;
pub const TFN_INT: C2Rust_Unnamed_22 = 1;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const GLV_READ_ONLY: C2Rust_Unnamed_23 = 16;
pub const GLV_NO_AUTOLOAD: C2Rust_Unnamed_23 = 4;
pub const GLV_QUIET: C2Rust_Unnamed_23 = 2;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const EVAL_EVALUATE: C2Rust_Unnamed_24 = 1;
pub type VimLFunc = Option<unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct EvalFuncDef {
    pub name: *mut ::core::ffi::c_char,
    pub min_argc: uint8_t,
    pub max_argc: uint8_t,
    pub base_arg: uint8_t,
    pub fast: bool,
    pub func: VimLFunc,
    pub data: EvalFuncData,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct funcdict_T {
    pub fd_dict: *mut dict_T,
    pub fd_newkey: *mut ::core::ffi::c_char,
    pub fd_di: *mut dictitem_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct funccal_entry {
    pub top_funccal: *mut ::core::ffi::c_void,
    pub next: *mut funccal_entry_T,
}
pub type funccal_entry_T = funccal_entry;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const FCERR_NOTMETHOD: C2Rust_Unnamed_25 = 8;
pub const FCERR_DELETED: C2Rust_Unnamed_25 = 7;
pub const FCERR_OTHER: C2Rust_Unnamed_25 = 6;
pub const FCERR_NONE: C2Rust_Unnamed_25 = 5;
pub const FCERR_DICT: C2Rust_Unnamed_25 = 4;
pub const FCERR_SCRIPT: C2Rust_Unnamed_25 = 3;
pub const FCERR_TOOFEW: C2Rust_Unnamed_25 = 2;
pub const FCERR_TOOMANY: C2Rust_Unnamed_25 = 1;
pub const FCERR_UNKNOWN: C2Rust_Unnamed_25 = 0;
pub type ArgvFunc = Option<
    unsafe extern "C" fn(
        ::core::ffi::c_int,
        *mut typval_T,
        ::core::ffi::c_int,
        *mut ufunc_T,
    ) -> ::core::ffi::c_int,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct funcexe_T {
    pub fe_argv_func: ArgvFunc,
    pub fe_firstline: linenr_T,
    pub fe_lastline: linenr_T,
    pub fe_doesrange: *mut bool,
    pub fe_evaluate: bool,
    pub fe_partial: *mut partial_T,
    pub fe_selfdict: *mut dict_T,
    pub fe_basetv: *mut typval_T,
    pub fe_found_var: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct estack_T {
    pub es_lnum: linenr_T,
    pub es_name: *mut ::core::ffi::c_char,
    pub es_type: etype_T,
    pub es_info: C2Rust_Unnamed_26,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_26 {
    pub sctx: *mut sctx_T,
    pub ufunc: *mut ufunc_T,
    pub aucmd: *mut AutoPatCmd,
    pub except: *mut except_T,
}
pub type etype_T = ::core::ffi::c_uint;
pub const ETYPE_SPELL: etype_T = 9;
pub const ETYPE_INTERNAL: etype_T = 8;
pub const ETYPE_ENV: etype_T = 7;
pub const ETYPE_ARGS: etype_T = 6;
pub const ETYPE_EXCEPT: etype_T = 5;
pub const ETYPE_MODELINE: etype_T = 4;
pub const ETYPE_AUCMD: etype_T = 3;
pub const ETYPE_UFUNC: etype_T = 2;
pub const ETYPE_SCRIPT: etype_T = 1;
pub const ETYPE_TOP: etype_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct save_redo_T {
    pub sr_redobuff: buffheader_T,
    pub sr_old_redobuff: buffheader_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct buffheader_T {
    pub bh_first: buffblock_T,
    pub bh_curr: *mut buffblock_T,
    pub bh_index: size_t,
    pub bh_space: size_t,
    pub bh_create_newblock: bool,
}
pub type buffblock_T = buffblock;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct buffblock {
    pub b_next: *mut buffblock,
    pub b_strlen: size_t,
    pub b_str: [::core::ffi::c_char; 1],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct defer_T {
    pub dr_name: *mut ::core::ffi::c_char,
    pub dr_argvars: [typval_T; 21],
    pub dr_argcount: ::core::ffi::c_int,
}
pub const DOCMD_REPEAT: C2Rust_Unnamed_27 = 4;
pub const DOCMD_VERBOSE: C2Rust_Unnamed_27 = 1;
pub const DOCMD_NOWAIT: C2Rust_Unnamed_27 = 2;
pub const KE_SNR: key_extra = 82;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const DOCMD_KEEPLINE: C2Rust_Unnamed_27 = 32;
pub const DOCMD_EXCRESET: C2Rust_Unnamed_27 = 16;
pub const DOCMD_KEYTYPED: C2Rust_Unnamed_27 = 8;
pub type key_extra = ::core::ffi::c_uint;
pub const KE_WILD: key_extra = 108;
pub const KE_COMMAND: key_extra = 104;
pub const KE_LUA: key_extra = 103;
pub const KE_EVENT: key_extra = 102;
pub const KE_MOUSEMOVE: key_extra = 100;
pub const KE_NOP: key_extra = 97;
pub const KE_DROP: key_extra = 95;
pub const KE_X2RELEASE: key_extra = 94;
pub const KE_X2DRAG: key_extra = 93;
pub const KE_X2MOUSE: key_extra = 92;
pub const KE_X1RELEASE: key_extra = 91;
pub const KE_X1DRAG: key_extra = 90;
pub const KE_X1MOUSE: key_extra = 89;
pub const KE_C_END: key_extra = 88;
pub const KE_C_HOME: key_extra = 87;
pub const KE_C_RIGHT: key_extra = 86;
pub const KE_C_LEFT: key_extra = 85;
pub const KE_CMDWIN: key_extra = 84;
pub const KE_PLUG: key_extra = 83;
pub const KE_KDEL: key_extra = 80;
pub const KE_KINS: key_extra = 79;
pub const KE_MOUSERIGHT: key_extra = 78;
pub const KE_MOUSELEFT: key_extra = 77;
pub const KE_MOUSEUP: key_extra = 76;
pub const KE_MOUSEDOWN: key_extra = 75;
pub const KE_S_XF4: key_extra = 74;
pub const KE_S_XF3: key_extra = 73;
pub const KE_S_XF2: key_extra = 72;
pub const KE_S_XF1: key_extra = 71;
pub const KE_LEFTRELEASE_NM: key_extra = 70;
pub const KE_LEFTMOUSE_NM: key_extra = 69;
pub const KE_XRIGHT: key_extra = 68;
pub const KE_XLEFT: key_extra = 67;
pub const KE_XDOWN: key_extra = 66;
pub const KE_XUP: key_extra = 65;
pub const KE_ZHOME: key_extra = 64;
pub const KE_XHOME: key_extra = 63;
pub const KE_ZEND: key_extra = 62;
pub const KE_XEND: key_extra = 61;
pub const KE_XF4: key_extra = 60;
pub const KE_XF3: key_extra = 59;
pub const KE_XF2: key_extra = 58;
pub const KE_XF1: key_extra = 57;
pub const KE_S_TAB_OLD: key_extra = 55;
pub const KE_TAB: key_extra = 54;
pub const KE_IGNORE: key_extra = 53;
pub const KE_RIGHTRELEASE: key_extra = 52;
pub const KE_RIGHTDRAG: key_extra = 51;
pub const KE_RIGHTMOUSE: key_extra = 50;
pub const KE_MIDDLERELEASE: key_extra = 49;
pub const KE_MIDDLEDRAG: key_extra = 48;
pub const KE_MIDDLEMOUSE: key_extra = 47;
pub const KE_LEFTRELEASE: key_extra = 46;
pub const KE_LEFTDRAG: key_extra = 45;
pub const KE_LEFTMOUSE: key_extra = 44;
pub const KE_MOUSE: key_extra = 43;
pub const KE_S_F37: key_extra = 42;
pub const KE_S_F36: key_extra = 41;
pub const KE_S_F35: key_extra = 40;
pub const KE_S_F34: key_extra = 39;
pub const KE_S_F33: key_extra = 38;
pub const KE_S_F32: key_extra = 37;
pub const KE_S_F31: key_extra = 36;
pub const KE_S_F30: key_extra = 35;
pub const KE_S_F29: key_extra = 34;
pub const KE_S_F28: key_extra = 33;
pub const KE_S_F27: key_extra = 32;
pub const KE_S_F26: key_extra = 31;
pub const KE_S_F25: key_extra = 30;
pub const KE_S_F24: key_extra = 29;
pub const KE_S_F23: key_extra = 28;
pub const KE_S_F22: key_extra = 27;
pub const KE_S_F21: key_extra = 26;
pub const KE_S_F20: key_extra = 25;
pub const KE_S_F19: key_extra = 24;
pub const KE_S_F18: key_extra = 23;
pub const KE_S_F17: key_extra = 22;
pub const KE_S_F16: key_extra = 21;
pub const KE_S_F15: key_extra = 20;
pub const KE_S_F14: key_extra = 19;
pub const KE_S_F13: key_extra = 18;
pub const KE_S_F12: key_extra = 17;
pub const KE_S_F11: key_extra = 16;
pub const KE_S_F10: key_extra = 15;
pub const KE_S_F9: key_extra = 14;
pub const KE_S_F8: key_extra = 13;
pub const KE_S_F7: key_extra = 12;
pub const KE_S_F6: key_extra = 11;
pub const KE_S_F5: key_extra = 10;
pub const KE_S_F4: key_extra = 9;
pub const KE_S_F3: key_extra = 8;
pub const KE_S_F2: key_extra = 7;
pub const KE_S_F1: key_extra = 6;
pub const KE_S_DOWN: key_extra = 5;
pub const KE_S_UP: key_extra = 4;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_iswhite_nl_or_nul(mut c: ::core::ffi::c_int) -> bool {
    return ascii_iswhite(c) as ::core::ffi::c_int != 0
        || c == '\n' as ::core::ffi::c_int
        || c == NUL;
}
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL,
};
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const FNE_INCL_BR: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FNE_CHECK_START: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const AUTOLOAD_CHAR: ::core::ffi::c_int = '#' as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn tv_list_set_lock(l: *mut list_T, lock: VarLockStatus) {
    if l.is_null() {
        '_c2rust_label: {
            if lock as ::core::ffi::c_uint == VAR_FIXED as ::core::ffi::c_int as ::core::ffi::c_uint
            {
            } else {
                __assert_fail(
                    b"lock == VAR_FIXED\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/eval/typval.h\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    76 as ::core::ffi::c_uint,
                    b"void tv_list_set_lock(list_T *const, const VarLockStatus)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        return;
    }
    (*l).lv_lock = lock;
}
#[inline(always)]
unsafe extern "C" fn tv_is_func(tv: typval_T) -> bool {
    return tv.v_type as ::core::ffi::c_uint
        == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
        || tv.v_type as ::core::ffi::c_uint
            == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint;
}
pub const TV_CSTRING: ::core::ffi::c_ulong = SIZE_MAX.wrapping_sub(1 as ::core::ffi::c_ulong);
static mut func_hashtab: hashtab_T = hashtab_T {
    ht_mask: 0,
    ht_used: 0,
    ht_filled: 0,
    ht_changed: 0,
    ht_locked: 0,
    ht_array: ::core::ptr::null_mut::<hashitem_T>(),
    ht_smallarray: [hashitem_T {
        hi_hash: 0,
        hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    }; 16],
};
static mut funcargs: garray_T = GA_EMPTY_INIT_VALUE;
static mut current_funccal: *mut funccall_T = ::core::ptr::null_mut::<funccall_T>();
static mut previous_funccal: *mut funccall_T = ::core::ptr::null_mut::<funccall_T>();
static mut e_funcexts: *const ::core::ffi::c_char =
    b"E122: Function %s already exists, add ! to replace it\0".as_ptr()
        as *const ::core::ffi::c_char;
static mut e_funcdict: *const ::core::ffi::c_char =
    b"E717: Dictionary entry already exists\0".as_ptr() as *const ::core::ffi::c_char;
static mut e_funcref: *const ::core::ffi::c_char =
    b"E718: Funcref required\0".as_ptr() as *const ::core::ffi::c_char;
static mut e_nofunc: *const ::core::ffi::c_char =
    b"E130: Unknown function: %s\0".as_ptr() as *const ::core::ffi::c_char;
static mut e_function_list_was_modified: [::core::ffi::c_char; 33] = unsafe {
    ::core::mem::transmute::<[u8; 33], [::core::ffi::c_char; 33]>(
        *b"E454: Function list was modified\0",
    )
};
static mut e_function_nesting_too_deep: [::core::ffi::c_char; 33] = unsafe {
    ::core::mem::transmute::<[u8; 33], [::core::ffi::c_char; 33]>(
        *b"E1058: Function nesting too deep\0",
    )
};
static mut e_no_white_space_allowed_before_str_str: [::core::ffi::c_char; 46] = unsafe {
    ::core::mem::transmute::<[u8; 46], [::core::ffi::c_char; 46]>(
        *b"E1068: No white space allowed before '%s': %s\0",
    )
};
static mut e_missing_heredoc_end_marker_str: [::core::ffi::c_char; 38] = unsafe {
    ::core::mem::transmute::<[u8; 38], [::core::ffi::c_char; 38]>(
        *b"E1145: Missing heredoc end marker: %s\0",
    )
};
static mut e_cannot_use_partial_with_dictionary_for_defer: [::core::ffi::c_char; 55] = unsafe {
    ::core::mem::transmute::<[u8; 55], [::core::ffi::c_char; 55]>(
        *b"E1300: Cannot use a partial with dictionary for :defer\0",
    )
};
#[no_mangle]
pub unsafe extern "C" fn func_init() {
    hash_init(&raw mut func_hashtab);
}
#[no_mangle]
pub unsafe extern "C" fn func_tbl_get() -> *mut hashtab_T {
    return &raw mut func_hashtab;
}
unsafe extern "C" fn one_function_arg(
    mut arg: *mut ::core::ffi::c_char,
    mut newargs: *mut garray_T,
    mut skip: bool,
) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = arg;
    while *p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
        || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
            && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
        || ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        || *p as ::core::ffi::c_int == '_' as ::core::ffi::c_int
    {
        p = p.offset(1);
    }
    if arg == p
        || *(*__ctype_b_loc()).offset(*arg as uint8_t as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
        || p.offset_from(arg) == 9 as isize
            && strncmp(
                arg,
                b"firstline\0".as_ptr() as *const ::core::ffi::c_char,
                9 as size_t,
            ) == 0 as ::core::ffi::c_int
        || p.offset_from(arg) == 8 as isize
            && strncmp(
                arg,
                b"lastline\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
    {
        if !skip {
            semsg(
                gettext(b"E125: Illegal argument: %s\0".as_ptr() as *const ::core::ffi::c_char),
                arg,
            );
        }
        return arg;
    }
    if !newargs.is_null() {
        ga_grow(newargs, 1 as ::core::ffi::c_int);
        let mut c: uint8_t = *p as uint8_t;
        *p = NUL as ::core::ffi::c_char;
        let mut arg_copy: *mut ::core::ffi::c_char = xstrdup(arg);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*newargs).ga_len {
            if strcmp(
                *((*newargs).ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize),
                arg_copy,
            ) == 0 as ::core::ffi::c_int
            {
                semsg(
                    gettext(b"E853: Duplicate argument name: %s\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    arg_copy,
                );
                xfree(arg_copy as *mut ::core::ffi::c_void);
                return arg;
            }
            i += 1;
        }
        *((*newargs).ga_data as *mut *mut ::core::ffi::c_char).offset((*newargs).ga_len as isize) =
            arg_copy;
        (*newargs).ga_len += 1;
        *p = c as ::core::ffi::c_char;
    }
    return p;
}
unsafe extern "C" fn get_function_args(
    mut argp: *mut *mut ::core::ffi::c_char,
    mut endchar: ::core::ffi::c_char,
    mut newargs: *mut garray_T,
    mut varargs: *mut ::core::ffi::c_int,
    mut default_args: *mut garray_T,
    mut skip: bool,
) -> ::core::ffi::c_int {
    let mut mustend: bool = false_0 != 0;
    let mut arg: *mut ::core::ffi::c_char = *argp;
    let mut p: *mut ::core::ffi::c_char = arg;
    if !newargs.is_null() {
        ga_init(
            newargs,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
            3 as ::core::ffi::c_int,
        );
    }
    if !default_args.is_null() {
        ga_init(
            default_args,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
            3 as ::core::ffi::c_int,
        );
    }
    if !varargs.is_null() {
        *varargs = false_0;
    }
    let mut any_default: bool = false_0 != 0;
    '_err_ret: {
        while *p as ::core::ffi::c_int != endchar as ::core::ffi::c_int {
            if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '.' as ::core::ffi::c_int
                && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '.' as ::core::ffi::c_int
            {
                if !varargs.is_null() {
                    *varargs = true_0;
                }
                p = p.offset(3 as ::core::ffi::c_int as isize);
                mustend = true_0 != 0;
            } else {
                arg = p;
                p = one_function_arg(p, newargs, skip);
                if p == arg {
                    break;
                }
                if *skipwhite(p) as ::core::ffi::c_int == '=' as ::core::ffi::c_int
                    && !default_args.is_null()
                {
                    let mut rettv: typval_T = typval_T {
                        v_type: VAR_UNKNOWN,
                        v_lock: VAR_UNLOCKED,
                        vval: typval_vval_union { v_number: 0 },
                    };
                    any_default = true_0 != 0;
                    p = skipwhite(p).offset(1 as ::core::ffi::c_int as isize);
                    p = skipwhite(p);
                    let mut expr: *mut ::core::ffi::c_char = p;
                    if eval1(
                        &raw mut p,
                        &raw mut rettv,
                        ::core::ptr::null_mut::<evalarg_T>(),
                    ) != FAIL
                    {
                        ga_grow(default_args, 1 as ::core::ffi::c_int);
                        while p > expr
                            && ascii_iswhite(
                                *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            ) as ::core::ffi::c_int
                                != 0
                        {
                            p = p.offset(-1);
                        }
                        let mut c: uint8_t = *p as uint8_t;
                        *p = NUL as ::core::ffi::c_char;
                        expr = xstrdup(expr);
                        *((*default_args).ga_data as *mut *mut ::core::ffi::c_char)
                            .offset((*default_args).ga_len as isize) = expr;
                        (*default_args).ga_len += 1;
                        *p = c as ::core::ffi::c_char;
                    } else {
                        mustend = true_0 != 0;
                    }
                } else if any_default {
                    emsg(gettext(
                        b"E989: Non-default argument follows default argument\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ));
                    mustend = true_0 != 0;
                }
                if ascii_iswhite(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                    && *skipwhite(p) as ::core::ffi::c_int == ',' as ::core::ffi::c_int
                {
                    if !skip {
                        semsg(
                            gettext(
                                &raw const e_no_white_space_allowed_before_str_str
                                    as *const ::core::ffi::c_char,
                            ),
                            b",\0".as_ptr() as *const ::core::ffi::c_char,
                            p,
                        );
                        break '_err_ret;
                    } else {
                        p = skipwhite(p);
                    }
                }
                if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
                    p = p.offset(1);
                } else {
                    mustend = true_0 != 0;
                }
            }
            p = skipwhite(p);
            if !(mustend as ::core::ffi::c_int != 0
                && *p as ::core::ffi::c_int != endchar as ::core::ffi::c_int)
            {
                continue;
            }
            if !skip {
                semsg(
                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                    *argp,
                );
            }
            break;
        }
        if *p as ::core::ffi::c_int == endchar as ::core::ffi::c_int {
            p = p.offset(1);
            *argp = p;
            return OK;
        }
    }
    if !newargs.is_null() {
        ga_clear_strings(newargs);
    }
    if !default_args.is_null() {
        ga_clear_strings(default_args);
    }
    return FAIL;
}
unsafe extern "C" fn register_closure(mut fp: *mut ufunc_T) {
    if (*fp).uf_scoped == current_funccal {
        return;
    }
    funccal_unref((*fp).uf_scoped, fp, false_0 != 0);
    (*fp).uf_scoped = current_funccal;
    (*current_funccal).fc_refcount += 1;
    ga_grow(
        &raw mut (*current_funccal).fc_ufuncs,
        1 as ::core::ffi::c_int,
    );
    let c2rust_fresh1 = (*current_funccal).fc_ufuncs.ga_len;
    (*current_funccal).fc_ufuncs.ga_len = (*current_funccal).fc_ufuncs.ga_len + 1;
    let c2rust_lvalue_ptr = &raw mut *((*current_funccal).fc_ufuncs.ga_data as *mut *mut ufunc_T)
        .offset(c2rust_fresh1 as isize);
    *c2rust_lvalue_ptr = fp;
}
static mut lambda_name: [::core::ffi::c_char; 73] = [0; 73];
unsafe extern "C" fn get_lambda_name() -> String_0 {
    static mut lambda_no: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    lambda_no += 1;
    let mut n: ::core::ffi::c_int = snprintf(
        &raw mut lambda_name as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 73]>(),
        b"<lambda>%d\0".as_ptr() as *const ::core::ffi::c_char,
        lambda_no,
    );
    return String_0 {
        data: &raw mut lambda_name as *mut ::core::ffi::c_char,
        size: if n < 1 as ::core::ffi::c_int {
            0 as size_t
        } else {
            (if n < ::core::mem::size_of::<[::core::ffi::c_char; 73]>() as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int
            {
                n
            } else {
                ::core::mem::size_of::<[::core::ffi::c_char; 73]>() as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int
            }) as size_t
        },
    };
}
unsafe extern "C" fn alloc_ufunc(
    mut name: *const ::core::ffi::c_char,
    mut namelen: size_t,
) -> *mut ufunc_T {
    let mut len: size_t = (240 as size_t)
        .wrapping_add(namelen)
        .wrapping_add(1 as size_t);
    let mut fp: *mut ufunc_T = xcalloc(1 as size_t, len) as *mut ufunc_T;
    xmemcpyz(
        &raw mut (*fp).uf_name as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        name as *const ::core::ffi::c_void,
        namelen,
    );
    (*fp).uf_namelen = namelen;
    if *name.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int == K_SPECIAL
    {
        len = namelen.wrapping_add(3 as size_t);
        (*fp).uf_name_exp = xmalloc(len) as *mut ::core::ffi::c_char;
        snprintf(
            (*fp).uf_name_exp,
            len,
            b"<SNR>%s\0".as_ptr() as *const ::core::ffi::c_char,
            (&raw mut (*fp).uf_name as *mut ::core::ffi::c_char)
                .offset(3 as ::core::ffi::c_int as isize),
        );
    }
    return fp;
}
#[no_mangle]
pub unsafe extern "C" fn get_lambda_tv(
    mut arg: *mut *mut ::core::ffi::c_char,
    mut rettv: *mut typval_T,
    mut evalarg: *mut evalarg_T,
) -> ::core::ffi::c_int {
    let mut start: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let evaluate: bool =
        !evalarg.is_null() && (*evalarg).eval_flags & EVAL_EVALUATE as ::core::ffi::c_int != 0;
    let mut newargs: garray_T = GA_EMPTY_INIT_VALUE;
    let mut pnewargs: *mut garray_T = ::core::ptr::null_mut::<garray_T>();
    let mut fp: *mut ufunc_T = ::core::ptr::null_mut::<ufunc_T>();
    let mut pt: *mut partial_T = ::core::ptr::null_mut::<partial_T>();
    let mut varargs: ::core::ffi::c_int = 0;
    let mut old_eval_lavars: *mut bool = eval_lavars_used;
    let mut eval_lavars: bool = false_0 != 0;
    let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *mut ::core::ffi::c_char =
        skipwhite((*arg).offset(1 as ::core::ffi::c_int as isize));
    let mut ret: ::core::ffi::c_int = get_function_args(
        &raw mut s,
        '-' as ::core::ffi::c_char,
        ::core::ptr::null_mut::<garray_T>(),
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ::core::ptr::null_mut::<garray_T>(),
        true_0 != 0,
    );
    if ret == FAIL || *s as ::core::ffi::c_int != '>' as ::core::ffi::c_int {
        return NOTDONE;
    }
    if evaluate {
        pnewargs = &raw mut newargs;
    } else {
        pnewargs = ::core::ptr::null_mut::<garray_T>();
    }
    *arg = skipwhite((*arg).offset(1 as ::core::ffi::c_int as isize));
    ret = get_function_args(
        arg,
        '-' as ::core::ffi::c_char,
        pnewargs,
        &raw mut varargs,
        ::core::ptr::null_mut::<garray_T>(),
        false_0 != 0,
    );
    if !(ret == FAIL || **arg as ::core::ffi::c_int != '>' as ::core::ffi::c_int) {
        if evaluate {
            eval_lavars_used = &raw mut eval_lavars;
        }
        *arg = skipwhite((*arg).offset(1 as ::core::ffi::c_int as isize));
        start = *arg;
        ret = skip_expr(arg, evalarg);
        end = *arg;
        if ret != FAIL {
            if !evalarg.is_null() {
                tofree = (*evalarg).eval_tofree;
                (*evalarg).eval_tofree = ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            *arg = skipwhite(*arg);
            if **arg as ::core::ffi::c_int != '}' as ::core::ffi::c_int {
                semsg(
                    gettext(b"E451: Expected }: %s\0".as_ptr() as *const ::core::ffi::c_char),
                    *arg,
                );
            } else {
                *arg = (*arg).offset(1);
                if evaluate {
                    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    let mut newlines: garray_T = garray_T {
                        ga_len: 0,
                        ga_maxlen: 0,
                        ga_itemsize: 0,
                        ga_growsize: 0,
                        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    };
                    let mut name: String_0 = get_lambda_name();
                    fp = alloc_ufunc(name.data, name.size);
                    pt =
                        xcalloc(1 as size_t, ::core::mem::size_of::<partial_T>()) as *mut partial_T;
                    ga_init(
                        &raw mut newlines,
                        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
                        1 as ::core::ffi::c_int,
                    );
                    ga_grow(&raw mut newlines, 1 as ::core::ffi::c_int);
                    let mut len: size_t = (end
                        .offset(7 as ::core::ffi::c_int as isize)
                        .offset_from(start)
                        + 1 as isize) as size_t;
                    let mut p: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
                    let c2rust_fresh0 = newlines.ga_len;
                    newlines.ga_len = newlines.ga_len + 1;
                    let c2rust_lvalue_ptr = &raw mut *(newlines.ga_data
                        as *mut *mut ::core::ffi::c_char)
                        .offset(c2rust_fresh0 as isize);
                    *c2rust_lvalue_ptr = p;
                    strcpy(
                        p,
                        b"return \0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                    );
                    xmemcpyz(
                        p.offset(7 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                        start as *const ::core::ffi::c_void,
                        end.offset_from(start) as size_t,
                    );
                    if strstr(
                        p.offset(7 as ::core::ffi::c_int as isize),
                        b"a:\0".as_ptr() as *const ::core::ffi::c_char,
                    )
                    .is_null()
                    {
                        flags |= FC_NOARGS;
                    }
                    (*fp).uf_refcount = 1 as ::core::ffi::c_int;
                    hash_add(
                        &raw mut func_hashtab,
                        &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
                    );
                    (*fp).uf_args = newargs;
                    ga_init(
                        &raw mut (*fp).uf_def_args,
                        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
                        1 as ::core::ffi::c_int,
                    );
                    (*fp).uf_lines = newlines;
                    if !current_funccal.is_null() && eval_lavars as ::core::ffi::c_int != 0 {
                        flags |= FC_CLOSURE;
                        register_closure(fp);
                    } else {
                        (*fp).uf_scoped = ::core::ptr::null_mut::<funccall_T>();
                    }
                    if prof_def_func() {
                        func_do_profile(fp);
                    }
                    if sandbox != 0 {
                        flags |= FC_SANDBOX;
                    }
                    (*fp).uf_varargs = true_0;
                    (*fp).uf_flags = flags;
                    (*fp).uf_calls = 0 as ::core::ffi::c_int;
                    (*fp).uf_script_ctx = current_sctx;
                    (*fp).uf_script_ctx.sc_lnum =
                        ((*fp).uf_script_ctx.sc_lnum as ::core::ffi::c_int
                            + ((*(exestack.ga_data as *mut estack_T)
                                .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
                            .es_lnum
                                - newlines.ga_len as linenr_T)
                                as ::core::ffi::c_int) as linenr_T;
                    (*pt).pt_func = fp;
                    (*pt).pt_refcount = 1 as ::core::ffi::c_int;
                    (*rettv).vval.v_partial = pt;
                    (*rettv).v_type = VAR_PARTIAL;
                }
                eval_lavars_used = old_eval_lavars;
                if !evalarg.is_null() && (*evalarg).eval_tofree.is_null() {
                    (*evalarg).eval_tofree = tofree;
                } else {
                    xfree(tofree as *mut ::core::ffi::c_void);
                }
                return OK;
            }
        }
    }
    ga_clear_strings(&raw mut newargs);
    '_c2rust_label: {
        if fp.is_null() {
        } else {
            __assert_fail(
                b"fp == NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/eval/userfunc.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                418 as ::core::ffi::c_uint,
                b"int get_lambda_tv(char **, typval_T *, evalarg_T *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    xfree(pt as *mut ::core::ffi::c_void);
    if !evalarg.is_null() && (*evalarg).eval_tofree.is_null() {
        (*evalarg).eval_tofree = tofree;
    } else {
        xfree(tofree as *mut ::core::ffi::c_void);
    }
    eval_lavars_used = old_eval_lavars;
    return FAIL;
}
#[no_mangle]
pub unsafe extern "C" fn deref_func_name(
    mut name: *const ::core::ffi::c_char,
    mut lenp: *mut ::core::ffi::c_int,
    partialp: *mut *mut partial_T,
    mut no_autoload: bool,
    mut found_var: *mut bool,
) -> *mut ::core::ffi::c_char {
    if !partialp.is_null() {
        *partialp = ::core::ptr::null_mut::<partial_T>();
    }
    let v: *mut dictitem_T = find_var(
        name,
        *lenp as size_t,
        ::core::ptr::null_mut::<*mut hashtab_T>(),
        no_autoload as ::core::ffi::c_int,
    );
    if v.is_null() {
        return name as *mut ::core::ffi::c_char;
    }
    let tv: *mut typval_T = &raw mut (*v).di_tv;
    if !found_var.is_null() {
        *found_var = true_0 != 0;
    }
    if (*tv).v_type as ::core::ffi::c_uint == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*tv).vval.v_string.is_null() {
            *lenp = 0 as ::core::ffi::c_int;
            return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        *lenp = strlen((*tv).vval.v_string) as ::core::ffi::c_int;
        return (*tv).vval.v_string;
    }
    if (*tv).v_type as ::core::ffi::c_uint
        == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let pt: *mut partial_T = (*tv).vval.v_partial;
        if pt.is_null() {
            *lenp = 0 as ::core::ffi::c_int;
            return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        if !partialp.is_null() {
            *partialp = pt;
        }
        let mut s: *mut ::core::ffi::c_char = partial_name(pt);
        *lenp = strlen(s) as ::core::ffi::c_int;
        return s;
    }
    return name as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn emsg_funcname(
    mut errmsg: *const ::core::ffi::c_char,
    mut name: *const ::core::ffi::c_char,
) {
    let mut p: *mut ::core::ffi::c_char = name as *mut ::core::ffi::c_char;
    if *name.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int == K_SPECIAL
        && *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        && *name.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
    {
        p = concat_str(
            b"<SNR>\0".as_ptr() as *const ::core::ffi::c_char,
            name.offset(3 as ::core::ffi::c_int as isize),
        );
    }
    semsg(gettext(errmsg), p);
    if p != name as *mut ::core::ffi::c_char {
        xfree(p as *mut ::core::ffi::c_void);
    }
}
unsafe extern "C" fn get_func_arguments(
    mut arg: *mut *mut ::core::ffi::c_char,
    evalarg: *mut evalarg_T,
    mut partial_argc: ::core::ffi::c_int,
    mut argvars: *mut typval_T,
    mut argcount: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut argp: *mut ::core::ffi::c_char = *arg;
    let mut ret: ::core::ffi::c_int = OK;
    while *argcount < MAX_FUNC_ARGS as ::core::ffi::c_int - partial_argc {
        argp = skipwhite(argp.offset(1 as ::core::ffi::c_int as isize));
        if *argp as ::core::ffi::c_int == ')' as ::core::ffi::c_int
            || *argp as ::core::ffi::c_int == ',' as ::core::ffi::c_int
            || *argp as ::core::ffi::c_int == NUL
        {
            break;
        }
        if eval1(&raw mut argp, argvars.offset(*argcount as isize), evalarg) == FAIL {
            ret = FAIL;
            break;
        } else {
            *argcount += 1;
            if *argp as ::core::ffi::c_int != ',' as ::core::ffi::c_int {
                break;
            }
        }
    }
    argp = skipwhite(argp);
    if *argp as ::core::ffi::c_int == ')' as ::core::ffi::c_int {
        argp = argp.offset(1);
    } else {
        ret = FAIL;
    }
    *arg = argp;
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn get_func_tv(
    mut name: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut rettv: *mut typval_T,
    mut arg: *mut *mut ::core::ffi::c_char,
    evalarg: *mut evalarg_T,
    mut funcexe: *mut funcexe_T,
) -> ::core::ffi::c_int {
    let mut argvars: [typval_T; 21] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 21];
    let mut argcount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let evaluate: bool = if evalarg.is_null() {
        false_0
    } else {
        (*evalarg).eval_flags & EVAL_EVALUATE as ::core::ffi::c_int
    } != 0;
    let mut argp: *mut ::core::ffi::c_char = *arg;
    let mut ret: ::core::ffi::c_int = get_func_arguments(
        &raw mut argp,
        evalarg,
        if (*funcexe).fe_partial.is_null() {
            0 as ::core::ffi::c_int
        } else {
            (*(*funcexe).fe_partial).pt_argc
        },
        &raw mut argvars as *mut typval_T,
        &raw mut argcount,
    );
    '_c2rust_label: {
        if ret == 1 as ::core::ffi::c_int || ret == 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"ret == OK || ret == FAIL\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/eval/userfunc.c\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                565 as ::core::ffi::c_uint,
                b"int get_func_tv(const char *, int, typval_T *, char **, evalarg_T *const, funcexe_T *)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if ret == OK {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if get_vim_var_nr(VV_TESTING) != 0 {
            if funcargs.ga_itemsize == 0 as ::core::ffi::c_int {
                ga_init(
                    &raw mut funcargs,
                    ::core::mem::size_of::<*mut typval_T>() as ::core::ffi::c_int,
                    50 as ::core::ffi::c_int,
                );
            }
            i = 0 as ::core::ffi::c_int;
            while i < argcount {
                ga_grow(&raw mut funcargs, 1 as ::core::ffi::c_int);
                let c2rust_fresh2 = funcargs.ga_len;
                funcargs.ga_len = funcargs.ga_len + 1;
                let c2rust_lvalue_ptr = &raw mut *(funcargs.ga_data as *mut *mut typval_T)
                    .offset(c2rust_fresh2 as isize);
                *c2rust_lvalue_ptr = (&raw mut argvars as *mut typval_T).offset(i as isize);
                i += 1;
            }
        }
        ret = call_func(
            name,
            len,
            rettv,
            argcount,
            &raw mut argvars as *mut typval_T,
            funcexe,
        );
        funcargs.ga_len -= i;
    } else if !aborting() && evaluate as ::core::ffi::c_int != 0 {
        if argcount == MAX_FUNC_ARGS as ::core::ffi::c_int {
            emsg_funcname(
                b"E740: Too many arguments for function %s\0".as_ptr()
                    as *const ::core::ffi::c_char,
                name,
            );
        } else {
            emsg_funcname(
                b"E116: Invalid arguments for function %s\0".as_ptr() as *const ::core::ffi::c_char,
                name,
            );
        }
    }
    loop {
        argcount -= 1;
        if argcount < 0 as ::core::ffi::c_int {
            break;
        }
        tv_clear((&raw mut argvars as *mut typval_T).offset(argcount as isize));
    }
    *arg = skipwhite(argp);
    return ret;
}
pub const FLEN_FIXED: ::core::ffi::c_int = 40 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn eval_fname_sid(name: *const ::core::ffi::c_char) -> bool {
    return *name as ::core::ffi::c_int == 's' as ::core::ffi::c_int
        || (if (*name.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            < 'a' as ::core::ffi::c_int
            || *name.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                > 'z' as ::core::ffi::c_int
        {
            *name.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        } else {
            *name.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        }) == 'I' as ::core::ffi::c_int;
}
unsafe extern "C" fn fname_trans_sid(
    name: *const ::core::ffi::c_char,
    fname_buf: *mut ::core::ffi::c_char,
    tofree: *mut *mut ::core::ffi::c_char,
    error: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut script_name: *const ::core::ffi::c_char = name.offset(eval_fname_script(name) as isize);
    if script_name == name {
        return name as *mut ::core::ffi::c_char;
    }
    *fname_buf.offset(0 as ::core::ffi::c_int as isize) = K_SPECIAL as ::core::ffi::c_char;
    *fname_buf.offset(1 as ::core::ffi::c_int as isize) = KS_EXTRA as ::core::ffi::c_char;
    *fname_buf.offset(2 as ::core::ffi::c_int as isize) =
        KE_SNR as ::core::ffi::c_int as ::core::ffi::c_char;
    let mut fname_buflen: size_t = 3 as size_t;
    if !eval_fname_sid(name) {
        *fname_buf.offset(fname_buflen as isize) = NUL as ::core::ffi::c_char;
    } else if current_sctx.sc_sid <= 0 as ::core::ffi::c_int {
        *error = FCERR_SCRIPT as ::core::ffi::c_int;
    } else {
        fname_buflen = fname_buflen.wrapping_add(snprintf(
            fname_buf.offset(fname_buflen as isize),
            ((FLEN_FIXED + 1 as ::core::ffi::c_int) as size_t).wrapping_sub(fname_buflen),
            b"%d_\0".as_ptr() as *const ::core::ffi::c_char,
            current_sctx.sc_sid,
        ) as size_t);
    }
    let mut fnamelen: size_t = fname_buflen.wrapping_add(strlen(script_name));
    let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if fnamelen < FLEN_FIXED as size_t {
        strcpy(
            fname_buf.offset(fname_buflen as isize),
            script_name as *mut ::core::ffi::c_char,
        );
        fname = fname_buf;
    } else {
        fname = xmalloc(fnamelen.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        *tofree = fname;
        snprintf(
            fname,
            fnamelen.wrapping_add(1 as size_t),
            b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
            fname_buf,
            script_name,
        );
    }
    return fname;
}
#[no_mangle]
pub unsafe extern "C" fn get_func_arity(
    mut name: *const ::core::ffi::c_char,
    mut required: *mut ::core::ffi::c_int,
    mut optional: *mut ::core::ffi::c_int,
    mut varargs: *mut bool,
) -> ::core::ffi::c_int {
    let mut argcount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut min_argcount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut fdef: *const EvalFuncDef = find_internal_func(name);
    if !fdef.is_null() {
        argcount = (*fdef).max_argc as ::core::ffi::c_int;
        min_argcount = (*fdef).min_argc as ::core::ffi::c_int;
        *varargs = false_0 != 0;
    } else {
        let mut fname_buf: [::core::ffi::c_char; 41] = [0; 41];
        let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut error: ::core::ffi::c_int = FCERR_NONE as ::core::ffi::c_int;
        let mut fname: *mut ::core::ffi::c_char = fname_trans_sid(
            name,
            &raw mut fname_buf as *mut ::core::ffi::c_char,
            &raw mut tofree,
            &raw mut error,
        );
        let mut ufunc: *mut ufunc_T = ::core::ptr::null_mut::<ufunc_T>();
        if error == FCERR_NONE as ::core::ffi::c_int {
            ufunc = find_func(fname);
        }
        xfree(tofree as *mut ::core::ffi::c_void);
        if ufunc.is_null() {
            return FAIL;
        }
        argcount = (*ufunc).uf_args.ga_len;
        min_argcount = (*ufunc).uf_args.ga_len - (*ufunc).uf_def_args.ga_len;
        *varargs = (*ufunc).uf_varargs != 0;
    }
    *required = min_argcount;
    *optional = argcount - min_argcount;
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn find_func(mut name: *const ::core::ffi::c_char) -> *mut ufunc_T {
    let mut hi: *mut hashitem_T = hash_find(&raw mut func_hashtab, name);
    if !((*hi).hi_key.is_null() || (*hi).hi_key == &raw mut hash_removed) {
        return (*hi).hi_key.offset(-(240 as ::core::ffi::c_ulong as isize)) as *mut ufunc_T;
    }
    return ::core::ptr::null_mut::<ufunc_T>();
}
unsafe extern "C" fn func_is_global(mut ufunc: *const ufunc_T) -> bool {
    return *(&raw const (*ufunc).uf_name as *const ::core::ffi::c_char)
        .offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
        != K_SPECIAL;
}
unsafe extern "C" fn cat_func_name(
    mut buf: *mut ::core::ffi::c_char,
    mut bufsize: size_t,
    mut fp: *const ufunc_T,
) -> ::core::ffi::c_int {
    let mut len: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut uflen: size_t = (*fp).uf_namelen;
    '_c2rust_label: {
        if uflen > 0 as size_t {
        } else {
            __assert_fail(
                b"uflen > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/eval/userfunc.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                736 as ::core::ffi::c_uint,
                b"int cat_func_name(char *, size_t, const ufunc_T *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if !func_is_global(fp) && uflen > 3 as size_t {
        len = snprintf(
            buf,
            bufsize,
            b"<SNR>%s\0".as_ptr() as *const ::core::ffi::c_char,
            (&raw const (*fp).uf_name as *const ::core::ffi::c_char)
                .offset(3 as ::core::ffi::c_int as isize),
        );
    } else {
        len = snprintf(
            buf,
            bufsize,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            &raw const (*fp).uf_name as *const ::core::ffi::c_char,
        );
    }
    '_c2rust_label_0: {
        if len > 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"len > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/eval/userfunc.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                744 as ::core::ffi::c_uint,
                b"int cat_func_name(char *, size_t, const ufunc_T *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    return if len >= bufsize as ::core::ffi::c_int {
        bufsize as ::core::ffi::c_int - 1 as ::core::ffi::c_int
    } else {
        len
    };
}
unsafe extern "C" fn add_nr_var(
    mut dp: *mut dict_T,
    mut v: *mut dictitem_T,
    mut name: *mut ::core::ffi::c_char,
    mut nr: varnumber_T,
) {
    strcpy(&raw mut (*v).di_key as *mut ::core::ffi::c_char, name);
    (*v).di_flags =
        (DI_FLAGS_RO as ::core::ffi::c_int | DI_FLAGS_FIX as ::core::ffi::c_int) as uint8_t;
    hash_add(
        &raw mut (*dp).dv_hashtab,
        &raw mut (*v).di_key as *mut ::core::ffi::c_char,
    );
    (*v).di_tv.v_type = VAR_NUMBER;
    (*v).di_tv.v_lock = VAR_FIXED;
    (*v).di_tv.vval.v_number = nr;
}
unsafe extern "C" fn free_funccal(mut fc: *mut funccall_T) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*fc).fc_ufuncs.ga_len {
        let mut fp: *mut ufunc_T =
            *((*fc).fc_ufuncs.ga_data as *mut *mut ufunc_T).offset(i as isize);
        if !fp.is_null() && (*fp).uf_scoped == fc {
            (*fp).uf_scoped = ::core::ptr::null_mut::<funccall_T>();
        }
        i += 1;
    }
    ga_clear(&raw mut (*fc).fc_ufuncs);
    func_ptr_unref((*fc).fc_func);
    xfree(fc as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn free_funccal_contents(mut fc: *mut funccall_T) {
    vars_clear(&raw mut (*fc).fc_l_vars.dv_hashtab);
    vars_clear(&raw mut (*fc).fc_l_avars.dv_hashtab);
    let l_: *mut list_T = &raw mut (*fc).fc_l_varlist;
    if !l_.is_null() {
        let mut li: *mut listitem_T = (*l_).lv_first;
        while !li.is_null() {
            tv_clear(&raw mut (*li).li_tv);
            li = (*li).li_next;
        }
    }
    free_funccal(fc);
}
unsafe extern "C" fn cleanup_function_call(mut fc: *mut funccall_T) {
    let mut may_free_fc: bool = (*fc).fc_refcount <= 0 as ::core::ffi::c_int;
    let mut free_fc: bool = true_0 != 0;
    current_funccal = (*fc).fc_caller;
    if may_free_fc as ::core::ffi::c_int != 0
        && (*fc).fc_l_vars.dv_refcount == DO_NOT_FREE_CNT as ::core::ffi::c_int
    {
        vars_clear(&raw mut (*fc).fc_l_vars.dv_hashtab);
    } else {
        free_fc = false_0 != 0;
    }
    if may_free_fc as ::core::ffi::c_int != 0
        && (*fc).fc_l_avars.dv_refcount == DO_NOT_FREE_CNT as ::core::ffi::c_int
    {
        vars_clear_ext(&raw mut (*fc).fc_l_avars.dv_hashtab, false_0 != 0);
    } else {
        free_fc = false_0 != 0;
        let dihi_ht_: *mut hashtab_T = &raw mut (*fc).fc_l_avars.dv_hashtab;
        let mut dihi_todo_: size_t = (*dihi_ht_).ht_used;
        let mut dihi_: *mut hashitem_T = (*dihi_ht_).ht_array;
        while dihi_todo_ != 0 {
            if !((*dihi_).hi_key.is_null() || (*dihi_).hi_key == &raw mut hash_removed) {
                dihi_todo_ = dihi_todo_.wrapping_sub(1);
                let di: *mut dictitem_T = (*dihi_)
                    .hi_key
                    .offset(-(17 as ::core::ffi::c_ulong as isize))
                    as *mut dictitem_T;
                tv_copy(&raw mut (*di).di_tv, &raw mut (*di).di_tv);
            }
            dihi_ = dihi_.offset(1);
        }
    }
    if may_free_fc as ::core::ffi::c_int != 0
        && (*fc).fc_l_varlist.lv_refcount == DO_NOT_FREE_CNT as ::core::ffi::c_int
    {
        (*fc).fc_l_varlist.lv_first = ::core::ptr::null_mut::<listitem_T>();
    } else {
        free_fc = false_0 != 0;
        let l_: *mut list_T = &raw mut (*fc).fc_l_varlist;
        if !l_.is_null() {
            let mut li: *mut listitem_T = (*l_).lv_first;
            while !li.is_null() {
                tv_copy(&raw mut (*li).li_tv, &raw mut (*li).li_tv);
                li = (*li).li_next;
            }
        }
    }
    if free_fc {
        free_funccal(fc);
    } else {
        static mut made_copy: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        (*fc).fc_caller = previous_funccal;
        previous_funccal = fc;
        if want_garbage_collect {
            made_copy = 0 as ::core::ffi::c_int;
        } else {
            made_copy += 1;
            if made_copy
                >= ((4096 as ::core::ffi::c_int * 1024 as ::core::ffi::c_int) as usize)
                    .wrapping_div(::core::mem::size_of::<funccall_T>())
                    as ::core::ffi::c_int
            {
                made_copy = 0 as ::core::ffi::c_int;
                want_garbage_collect = true_0 != 0;
            }
        }
    };
}
unsafe extern "C" fn funccal_unref(mut fc: *mut funccall_T, mut fp: *mut ufunc_T, mut force: bool) {
    if fc.is_null() {
        return;
    }
    (*fc).fc_refcount -= 1;
    if if force as ::core::ffi::c_int != 0 {
        ((*fc).fc_refcount <= 0 as ::core::ffi::c_int) as ::core::ffi::c_int
    } else {
        !fc_referenced(fc) as ::core::ffi::c_int
    } != 0
    {
        let mut pfc: *mut *mut funccall_T = &raw mut previous_funccal;
        while !(*pfc).is_null() {
            if fc == *pfc {
                *pfc = (*fc).fc_caller;
                free_funccal_contents(fc);
                return;
            }
            pfc = &raw mut (**pfc).fc_caller;
        }
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*fc).fc_ufuncs.ga_len {
        if *((*fc).fc_ufuncs.ga_data as *mut *mut ufunc_T).offset(i as isize) == fp {
            *((*fc).fc_ufuncs.ga_data as *mut *mut ufunc_T).offset(i as isize) =
                ::core::ptr::null_mut::<ufunc_T>();
        }
        i += 1;
    }
}
unsafe extern "C" fn func_remove(mut fp: *mut ufunc_T) -> bool {
    let mut hi: *mut hashitem_T = hash_find(
        &raw mut func_hashtab,
        &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
    );
    if (*hi).hi_key.is_null() || (*hi).hi_key == &raw mut hash_removed {
        return false_0 != 0;
    }
    hash_remove(&raw mut func_hashtab, hi);
    return true_0 != 0;
}
unsafe extern "C" fn func_clear_items(mut fp: *mut ufunc_T) {
    ga_clear_strings(&raw mut (*fp).uf_args);
    ga_clear_strings(&raw mut (*fp).uf_def_args);
    ga_clear_strings(&raw mut (*fp).uf_lines);
    if (*fp).uf_flags & FC_LUAREF != 0 {
        api_free_luaref((*fp).uf_luaref);
        (*fp).uf_luaref = LUA_NOREF as LuaRef;
    }
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*fp).uf_tml_count as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    *ptr_;
    let mut ptr__0: *mut *mut ::core::ffi::c_void =
        &raw mut (*fp).uf_tml_total as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__0);
    *ptr__0 = NULL;
    *ptr__0;
    let mut ptr__1: *mut *mut ::core::ffi::c_void =
        &raw mut (*fp).uf_tml_self as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__1);
    *ptr__1 = NULL;
    *ptr__1;
}
unsafe extern "C" fn func_clear(mut fp: *mut ufunc_T, mut force: bool) {
    if (*fp).uf_cleared {
        return;
    }
    (*fp).uf_cleared = true_0 != 0;
    func_clear_items(fp);
    funccal_unref((*fp).uf_scoped, fp, force);
}
unsafe extern "C" fn func_free(mut fp: *mut ufunc_T) {
    if (*fp).uf_flags & (FC_DELETED | FC_REMOVED) == 0 as ::core::ffi::c_int {
        func_remove(fp);
    }
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*fp).uf_name_exp as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    *ptr_;
    xfree(fp as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn func_clear_free(mut fp: *mut ufunc_T, mut force: bool) {
    func_clear(fp, force);
    func_free(fp);
}
#[no_mangle]
pub unsafe extern "C" fn create_funccal(
    mut fp: *mut ufunc_T,
    mut rettv: *mut typval_T,
) -> *mut funccall_T {
    let mut fc: *mut funccall_T =
        xcalloc(1 as size_t, ::core::mem::size_of::<funccall_T>()) as *mut funccall_T;
    (*fc).fc_caller = current_funccal;
    current_funccal = fc;
    (*fc).fc_func = fp;
    func_ptr_ref(fp);
    (*fc).fc_rettv = rettv;
    return fc;
}
#[no_mangle]
pub unsafe extern "C" fn remove_funccal() {
    let mut fc: *mut funccall_T = current_funccal;
    current_funccal = (*fc).fc_caller;
    free_funccal(fc);
}
#[no_mangle]
pub unsafe extern "C" fn call_user_func(
    mut fp: *mut ufunc_T,
    mut argcount: ::core::ffi::c_int,
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut firstline: linenr_T,
    mut lastline: linenr_T,
    mut selfdict: *mut dict_T,
) {
    let mut using_sandbox: bool = false_0 != 0;
    static mut depth: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut v: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    let mut fixvar_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut islambda: bool = false_0 != 0;
    let mut numbuf: [::core::ffi::c_char; 65] = [0; 65];
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut namelen: size_t = 0;
    let mut tv_to_free: [*mut typval_T; 20] = [::core::ptr::null_mut::<typval_T>(); 20];
    let mut tv_to_free_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut wait_start: proftime_T = 0;
    let mut call_start: proftime_T = 0;
    let mut started_profiling: bool = false_0 != 0;
    let mut did_save_redo: bool = false_0 != 0;
    let mut save_redo: save_redo_T = save_redo_T {
        sr_redobuff: buffheader_T {
            bh_first: buffblock_T {
                b_next: ::core::ptr::null_mut::<buffblock>(),
                b_strlen: 0,
                b_str: [0; 1],
            },
            bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
            bh_index: 0,
            bh_space: 0,
            bh_create_newblock: false,
        },
        sr_old_redobuff: buffheader_T {
            bh_first: buffblock_T {
                b_next: ::core::ptr::null_mut::<buffblock>(),
                b_strlen: 0,
                b_str: [0; 1],
            },
            bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
            bh_index: 0,
            bh_space: 0,
            bh_create_newblock: false,
        },
    };
    if depth as OptInt >= p_mfd {
        emsg(gettext(
            b"E132: Function call depth is higher than 'maxfuncdepth'\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
        (*rettv).v_type = VAR_NUMBER;
        (*rettv).vval.v_number = -1 as varnumber_T;
        return;
    }
    depth += 1;
    save_search_patterns();
    if !ins_compl_active() {
        saveRedobuff(&raw mut save_redo);
        did_save_redo = true_0 != 0;
    }
    (*fp).uf_calls += 1;
    line_breakcheck();
    let mut fc: *mut funccall_T = create_funccal(fp, rettv);
    (*fc).fc_level = ex_nesting_level;
    (*fc).fc_breakpoint = dbg_find_breakpoint(
        false_0 != 0,
        &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
        0 as linenr_T,
    );
    (*fc).fc_dbg_tick = debug_tick;
    ga_init(
        &raw mut (*fc).fc_ufuncs,
        ::core::mem::size_of::<*mut ufunc_T>() as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    if strncmp(
        &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
        b"<lambda>\0".as_ptr() as *const ::core::ffi::c_char,
        8 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        islambda = true_0 != 0;
    }
    init_var_dict(
        &raw mut (*fc).fc_l_vars,
        &raw mut (*fc).fc_l_vars_var,
        VAR_DEF_SCOPE,
    );
    if !selfdict.is_null() {
        let c2rust_fresh3 = fixvar_idx;
        fixvar_idx = fixvar_idx + 1;
        v = (&raw mut (*fc).fc_fixvar as *mut C2Rust_Unnamed_7).offset(c2rust_fresh3 as isize)
            as *mut dictitem_T;
        name = &raw mut (*v).di_key as *mut ::core::ffi::c_char;
        strcpy(
            name,
            b"self\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        (*v).di_flags =
            (DI_FLAGS_RO as ::core::ffi::c_int | DI_FLAGS_FIX as ::core::ffi::c_int) as uint8_t;
        hash_add(
            &raw mut (*fc).fc_l_vars.dv_hashtab,
            &raw mut (*v).di_key as *mut ::core::ffi::c_char,
        );
        (*v).di_tv.v_type = VAR_DICT;
        (*v).di_tv.v_lock = VAR_UNLOCKED;
        (*v).di_tv.vval.v_dict = selfdict;
        (*selfdict).dv_refcount += 1;
    }
    init_var_dict(
        &raw mut (*fc).fc_l_avars,
        &raw mut (*fc).fc_l_avars_var,
        VAR_SCOPE,
    );
    if (*fp).uf_flags & FC_NOARGS == 0 as ::core::ffi::c_int {
        let c2rust_fresh4 = fixvar_idx;
        fixvar_idx = fixvar_idx + 1;
        add_nr_var(
            &raw mut (*fc).fc_l_avars,
            (&raw mut (*fc).fc_fixvar as *mut C2Rust_Unnamed_7).offset(c2rust_fresh4 as isize)
                as *mut dictitem_T,
            b"0\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            (if argcount >= (*fp).uf_args.ga_len {
                argcount - (*fp).uf_args.ga_len
            } else {
                0 as ::core::ffi::c_int
            }) as varnumber_T,
        );
    }
    (*fc).fc_l_avars.dv_lock = VAR_FIXED;
    if (*fp).uf_flags & FC_NOARGS == 0 as ::core::ffi::c_int {
        let c2rust_fresh5 = fixvar_idx;
        fixvar_idx = fixvar_idx + 1;
        v = (&raw mut (*fc).fc_fixvar as *mut C2Rust_Unnamed_7).offset(c2rust_fresh5 as isize)
            as *mut dictitem_T;
        name = &raw mut (*v).di_key as *mut ::core::ffi::c_char;
        strcpy(
            name,
            b"000\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        (*v).di_flags =
            (DI_FLAGS_RO as ::core::ffi::c_int | DI_FLAGS_FIX as ::core::ffi::c_int) as uint8_t;
        hash_add(
            &raw mut (*fc).fc_l_avars.dv_hashtab,
            &raw mut (*v).di_key as *mut ::core::ffi::c_char,
        );
        (*v).di_tv.v_type = VAR_LIST;
        (*v).di_tv.v_lock = VAR_FIXED;
        (*v).di_tv.vval.v_list = &raw mut (*fc).fc_l_varlist;
    }
    tv_list_init_static(&raw mut (*fc).fc_l_varlist);
    tv_list_set_lock(&raw mut (*fc).fc_l_varlist, VAR_FIXED);
    if (*fp).uf_flags & FC_NOARGS == 0 as ::core::ffi::c_int {
        let c2rust_fresh6 = fixvar_idx;
        fixvar_idx = fixvar_idx + 1;
        add_nr_var(
            &raw mut (*fc).fc_l_avars,
            (&raw mut (*fc).fc_fixvar as *mut C2Rust_Unnamed_7).offset(c2rust_fresh6 as isize)
                as *mut dictitem_T,
            b"firstline\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            firstline as varnumber_T,
        );
        let c2rust_fresh7 = fixvar_idx;
        fixvar_idx = fixvar_idx + 1;
        add_nr_var(
            &raw mut (*fc).fc_l_avars,
            (&raw mut (*fc).fc_fixvar as *mut C2Rust_Unnamed_7).offset(c2rust_fresh7 as isize)
                as *mut dictitem_T,
            b"lastline\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            lastline as varnumber_T,
        );
    }
    let mut default_arg_err: bool = false_0 != 0;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < argcount || i < (*fp).uf_args.ga_len {
        let mut addlocal: bool = false_0 != 0;
        let mut isdefault: bool = false_0 != 0;
        let mut def_rettv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let mut ai: ::core::ffi::c_int = i - (*fp).uf_args.ga_len;
        if ai < 0 as ::core::ffi::c_int {
            name = *((*fp).uf_args.ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize);
            if islambda {
                addlocal = true_0 != 0;
            }
            isdefault = ai + (*fp).uf_def_args.ga_len >= 0 as ::core::ffi::c_int && i >= argcount;
            if isdefault {
                let mut default_expr: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                def_rettv.v_type = VAR_NUMBER;
                def_rettv.vval.v_number = -1 as varnumber_T;
                default_expr = *((*fp).uf_def_args.ga_data as *mut *mut ::core::ffi::c_char)
                    .offset((ai + (*fp).uf_def_args.ga_len) as isize);
                if eval1(
                    &raw mut default_expr,
                    &raw mut def_rettv,
                    &raw mut EVALARG_EVALUATE,
                ) == FAIL
                {
                    default_arg_err = true_0 != 0;
                    break;
                }
            }
            namelen = strlen(name);
        } else {
            if (*fp).uf_flags & FC_NOARGS != 0 as ::core::ffi::c_int {
                break;
            }
            namelen = snprintf(
                &raw mut numbuf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 65]>(),
                b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                ai + 1 as ::core::ffi::c_int,
            ) as size_t;
            name = &raw mut numbuf as *mut ::core::ffi::c_char;
        }
        if fixvar_idx < FIXVAR_CNT as ::core::ffi::c_int
            && namelen <= VAR_SHORT_LEN as ::core::ffi::c_int as size_t
        {
            let c2rust_fresh8 = fixvar_idx;
            fixvar_idx = fixvar_idx + 1;
            v = (&raw mut (*fc).fc_fixvar as *mut C2Rust_Unnamed_7).offset(c2rust_fresh8 as isize)
                as *mut dictitem_T;
            (*v).di_flags =
                (DI_FLAGS_RO as ::core::ffi::c_int | DI_FLAGS_FIX as ::core::ffi::c_int) as uint8_t;
            strcpy(&raw mut (*v).di_key as *mut ::core::ffi::c_char, name);
        } else {
            v = tv_dict_item_alloc_len(name, namelen);
            (*v).di_flags = ((*v).di_flags as ::core::ffi::c_int
                | (DI_FLAGS_RO as ::core::ffi::c_int | DI_FLAGS_FIX as ::core::ffi::c_int))
                as uint8_t;
        }
        (*v).di_tv = if isdefault as ::core::ffi::c_int != 0 {
            def_rettv
        } else {
            *argvars.offset(i as isize)
        };
        (*v).di_tv.v_lock = VAR_FIXED;
        if isdefault {
            let c2rust_fresh9 = tv_to_free_len;
            tv_to_free_len = tv_to_free_len + 1;
            let c2rust_lvalue_ptr = &raw mut tv_to_free[c2rust_fresh9 as usize];
            *c2rust_lvalue_ptr = &raw mut (*v).di_tv;
        }
        if addlocal {
            tv_copy(&raw mut (*v).di_tv, &raw mut (*v).di_tv);
            hash_add(
                &raw mut (*fc).fc_l_vars.dv_hashtab,
                &raw mut (*v).di_key as *mut ::core::ffi::c_char,
            );
        } else {
            hash_add(
                &raw mut (*fc).fc_l_avars.dv_hashtab,
                &raw mut (*v).di_key as *mut ::core::ffi::c_char,
            );
        }
        if ai >= 0 as ::core::ffi::c_int && ai < MAX_FUNC_ARGS as ::core::ffi::c_int {
            let mut li: *mut listitem_T =
                (&raw mut (*fc).fc_l_listitems as *mut listitem_T).offset(ai as isize);
            (*li).li_tv = *argvars.offset(i as isize);
            (*li).li_tv.v_lock = VAR_FIXED;
            tv_list_append(&raw mut (*fc).fc_l_varlist, li);
        }
        i += 1;
    }
    RedrawingDisabled += 1;
    if (*fp).uf_flags & FC_SANDBOX != 0 {
        using_sandbox = true_0 != 0;
        sandbox += 1;
    }
    estack_push_ufunc(fp, 1 as linenr_T);
    if p_verbose >= 12 as OptInt {
        no_wait_return += 1;
        verbose_enter_scroll();
        smsg(
            0 as ::core::ffi::c_int,
            gettext(b"calling %s\0".as_ptr() as *const ::core::ffi::c_char),
            (*(exestack.ga_data as *mut estack_T)
                .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_name,
        );
        if p_verbose >= 14 as OptInt {
            msg_puts(b"(\0".as_ptr() as *const ::core::ffi::c_char);
            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_0 < argcount {
                if i_0 > 0 as ::core::ffi::c_int {
                    msg_puts(b", \0".as_ptr() as *const ::core::ffi::c_char);
                }
                if (*argvars.offset(i_0 as isize)).v_type as ::core::ffi::c_uint
                    == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    msg_outnum((*argvars.offset(i_0 as isize)).vval.v_number as ::core::ffi::c_int);
                } else {
                    emsg_off += 1;
                    let mut tofree: *mut ::core::ffi::c_char = encode_tv2string(
                        argvars.offset(i_0 as isize),
                        ::core::ptr::null_mut::<size_t>(),
                    );
                    emsg_off -= 1;
                    if !tofree.is_null() {
                        let mut s: *mut ::core::ffi::c_char = tofree;
                        let mut buf: [::core::ffi::c_char; 480] = [0; 480];
                        if vim_strsize(s) > MSG_BUF_CLEN {
                            trunc_string(
                                s,
                                &raw mut buf as *mut ::core::ffi::c_char,
                                MSG_BUF_CLEN,
                                ::core::mem::size_of::<[::core::ffi::c_char; 480]>()
                                    as ::core::ffi::c_int,
                            );
                            s = &raw mut buf as *mut ::core::ffi::c_char;
                        }
                        msg_puts(s);
                        xfree(tofree as *mut ::core::ffi::c_void);
                    }
                }
                i_0 += 1;
            }
            msg_puts(b")\0".as_ptr() as *const ::core::ffi::c_char);
        }
        msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
        verbose_leave_scroll();
        no_wait_return -= 1;
    }
    let do_profiling_yes: bool = do_profiling == PROF_YES;
    let mut func_not_yet_profiling_but_should: bool = do_profiling_yes as ::core::ffi::c_int != 0
        && (*fp).uf_profiling == 0
        && has_profiling(
            false_0 != 0,
            &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<bool>(),
        ) as ::core::ffi::c_int
            != 0;
    if func_not_yet_profiling_but_should {
        started_profiling = true_0 != 0;
        func_do_profile(fp);
    }
    let mut func_or_func_caller_profiling: bool = do_profiling_yes as ::core::ffi::c_int != 0
        && ((*fp).uf_profiling != 0
            || !(*fc).fc_caller.is_null() && (*(*(*fc).fc_caller).fc_func).uf_profiling != 0);
    if func_or_func_caller_profiling {
        (*fp).uf_tm_count += 1;
        call_start = profile_start();
        (*fp).uf_tm_children = profile_zero();
    }
    if do_profiling_yes {
        script_prof_save(&raw mut wait_start);
    }
    let save_current_sctx: sctx_T = current_sctx;
    current_sctx = (*fp).uf_script_ctx;
    let mut save_did_emsg: ::core::ffi::c_int = did_emsg;
    did_emsg = false_0;
    if default_arg_err as ::core::ffi::c_int != 0
        && ((*fp).uf_flags & FC_ABORT != 0 || trylevel > 0 as ::core::ffi::c_int)
    {
        did_emsg = true_0;
    } else if islambda {
        let mut p: *mut ::core::ffi::c_char = (*((*fp).uf_lines.ga_data
            as *mut *mut ::core::ffi::c_char))
            .offset(7 as ::core::ffi::c_int as isize);
        ex_nesting_level += 1;
        eval1(&raw mut p, rettv, &raw mut EVALARG_EVALUATE);
        ex_nesting_level -= 1;
    } else {
        do_cmdline(
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            Some(
                get_func_line
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut ::core::ffi::c_void,
                        ::core::ffi::c_int,
                        bool,
                    ) -> *mut ::core::ffi::c_char,
            ),
            fc as *mut ::core::ffi::c_void,
            DOCMD_NOWAIT as ::core::ffi::c_int
                | DOCMD_VERBOSE as ::core::ffi::c_int
                | DOCMD_REPEAT as ::core::ffi::c_int,
        );
    }
    handle_defer_one(current_funccal);
    RedrawingDisabled -= 1;
    if did_emsg != 0 && (*fp).uf_flags & FC_ABORT != 0
        || (*rettv).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_clear(rettv);
        (*rettv).v_type = VAR_NUMBER;
        (*rettv).vval.v_number = -1 as varnumber_T;
    }
    if func_or_func_caller_profiling {
        call_start = profile_end(call_start);
        call_start = profile_sub_wait(wait_start, call_start);
        (*fp).uf_tm_total = profile_add((*fp).uf_tm_total, call_start);
        (*fp).uf_tm_self = profile_self((*fp).uf_tm_self, call_start, (*fp).uf_tm_children);
        if !(*fc).fc_caller.is_null() && (*(*(*fc).fc_caller).fc_func).uf_profiling != 0 {
            (*(*(*fc).fc_caller).fc_func).uf_tm_children =
                profile_add((*(*(*fc).fc_caller).fc_func).uf_tm_children, call_start);
            (*(*(*fc).fc_caller).fc_func).uf_tml_children =
                profile_add((*(*(*fc).fc_caller).fc_func).uf_tml_children, call_start);
        }
        if started_profiling {
            (*fp).uf_profiling = false_0;
        }
    }
    if p_verbose >= 12 as OptInt {
        no_wait_return += 1;
        verbose_enter_scroll();
        if aborting() {
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"%s aborted\0".as_ptr() as *const ::core::ffi::c_char),
                (*(exestack.ga_data as *mut estack_T)
                    .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_name,
            );
        } else if (*(*fc).fc_rettv).v_type as ::core::ffi::c_uint
            == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"%s returning #%ld\0".as_ptr() as *const ::core::ffi::c_char),
                (*(exestack.ga_data as *mut estack_T)
                    .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_name,
                (*(*fc).fc_rettv).vval.v_number,
            );
        } else {
            let mut buf_0: [::core::ffi::c_char; 480] = [0; 480];
            emsg_off += 1;
            let mut s_0: *mut ::core::ffi::c_char =
                encode_tv2string((*fc).fc_rettv, ::core::ptr::null_mut::<size_t>());
            let mut tofree_0: *mut ::core::ffi::c_char = s_0;
            emsg_off -= 1;
            if !s_0.is_null() {
                if vim_strsize(s_0) > MSG_BUF_CLEN {
                    trunc_string(
                        s_0,
                        &raw mut buf_0 as *mut ::core::ffi::c_char,
                        MSG_BUF_CLEN,
                        MSG_BUF_LEN,
                    );
                    s_0 = &raw mut buf_0 as *mut ::core::ffi::c_char;
                }
                smsg(
                    0 as ::core::ffi::c_int,
                    gettext(b"%s returning %s\0".as_ptr() as *const ::core::ffi::c_char),
                    (*(exestack.ga_data as *mut estack_T)
                        .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_name,
                    s_0,
                );
                xfree(tofree_0 as *mut ::core::ffi::c_void);
            }
        }
        msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
        verbose_leave_scroll();
        no_wait_return -= 1;
    }
    estack_pop();
    current_sctx = save_current_sctx;
    if do_profiling_yes {
        script_prof_restore(&raw mut wait_start);
    }
    if using_sandbox {
        sandbox -= 1;
    }
    if p_verbose >= 12 as OptInt
        && !(*(exestack.ga_data as *mut estack_T)
            .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_name
        .is_null()
    {
        no_wait_return += 1;
        verbose_enter_scroll();
        smsg(
            0 as ::core::ffi::c_int,
            gettext(b"continuing in %s\0".as_ptr() as *const ::core::ffi::c_char),
            (*(exestack.ga_data as *mut estack_T)
                .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_name,
        );
        msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
        verbose_leave_scroll();
        no_wait_return -= 1;
    }
    did_emsg |= save_did_emsg;
    depth -= 1;
    let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_1 < tv_to_free_len {
        tv_clear(tv_to_free[i_1 as usize]);
        i_1 += 1;
    }
    cleanup_function_call(fc);
    (*fp).uf_calls -= 1;
    if (*fp).uf_calls <= 0 as ::core::ffi::c_int && (*fp).uf_refcount <= 0 as ::core::ffi::c_int {
        func_clear_free(fp, false_0 != 0);
    }
    if did_save_redo {
        restoreRedobuff(&raw mut save_redo);
    }
    restore_search_patterns();
}
unsafe extern "C" fn func_name_refcount(mut name: *const ::core::ffi::c_char) -> bool {
    return *(*__ctype_b_loc()).offset(*name as uint8_t as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
        != 0
        || *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '<' as ::core::ffi::c_int
            && *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'l' as ::core::ffi::c_int;
}
unsafe extern "C" fn check_user_func_argcount(
    mut fp: *mut ufunc_T,
    mut argcount: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let regular_args: ::core::ffi::c_int = (*fp).uf_args.ga_len;
    if argcount < regular_args - (*fp).uf_def_args.ga_len {
        return FCERR_TOOFEW as ::core::ffi::c_int;
    } else if (*fp).uf_varargs == 0 && argcount > regular_args {
        return FCERR_TOOMANY as ::core::ffi::c_int;
    }
    return FCERR_UNKNOWN as ::core::ffi::c_int;
}
unsafe extern "C" fn call_user_func_check(
    mut fp: *mut ufunc_T,
    mut argcount: ::core::ffi::c_int,
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut funcexe: *mut funcexe_T,
    mut selfdict: *mut dict_T,
) -> ::core::ffi::c_int {
    if (*fp).uf_flags & FC_LUAREF != 0 {
        return typval_exec_lua_callable((*fp).uf_luaref, argcount, argvars, rettv);
    }
    if (*fp).uf_flags & FC_RANGE != 0 && !(*funcexe).fe_doesrange.is_null() {
        *(*funcexe).fe_doesrange = true_0 != 0;
    }
    let mut error: ::core::ffi::c_int = check_user_func_argcount(fp, argcount);
    if error != FCERR_UNKNOWN as ::core::ffi::c_int {
        return error;
    }
    if (*fp).uf_flags & FC_DICT != 0 && selfdict.is_null() {
        error = FCERR_DICT as ::core::ffi::c_int;
    } else {
        call_user_func(
            fp,
            argcount,
            argvars,
            rettv,
            (*funcexe).fe_firstline,
            (*funcexe).fe_lastline,
            if (*fp).uf_flags & FC_DICT != 0 {
                selfdict
            } else {
                ::core::ptr::null_mut::<dict_T>()
            },
        );
        error = FCERR_NONE as ::core::ffi::c_int;
    }
    return error;
}
static mut funccal_stack: *mut funccal_entry_T = ::core::ptr::null_mut::<funccal_entry_T>();
#[no_mangle]
pub unsafe extern "C" fn save_funccal(mut entry: *mut funccal_entry_T) {
    (*entry).top_funccal = current_funccal as *mut ::core::ffi::c_void;
    (*entry).next = funccal_stack;
    funccal_stack = entry;
    current_funccal = ::core::ptr::null_mut::<funccall_T>();
}
#[no_mangle]
pub unsafe extern "C" fn restore_funccal() {
    if funccal_stack.is_null() {
        iemsg(b"INTERNAL: restore_funccal()\0".as_ptr() as *const ::core::ffi::c_char);
    } else {
        current_funccal = (*funccal_stack).top_funccal as *mut funccall_T;
        funccal_stack = (*funccal_stack).next;
    };
}
#[no_mangle]
pub unsafe extern "C" fn get_current_funccal() -> *mut funccall_T {
    return current_funccal;
}
#[no_mangle]
pub unsafe extern "C" fn set_current_funccal(mut fc: *mut funccall_T) {
    current_funccal = fc;
}
unsafe extern "C" fn builtin_function(
    mut name: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) -> bool {
    if !(*name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
        >= 'a' as ::core::ffi::c_uint
        && *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
            <= 'z' as ::core::ffi::c_uint)
        || *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == ':' as ::core::ffi::c_int
    {
        return false_0 != 0;
    }
    let mut p: *const ::core::ffi::c_char = (if len == -1 as ::core::ffi::c_int {
        strchr(name, AUTOLOAD_CHAR) as *mut ::core::ffi::c_void
    } else {
        memchr(
            name as *const ::core::ffi::c_void,
            AUTOLOAD_CHAR,
            len as size_t,
        )
    }) as *const ::core::ffi::c_char;
    return p.is_null();
}
#[no_mangle]
pub unsafe extern "C" fn func_call(
    mut name: *mut ::core::ffi::c_char,
    mut args: *mut typval_T,
    mut partial: *mut partial_T,
    mut selfdict: *mut dict_T,
    mut rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    let mut funcexe: funcexe_T = funcexe_T {
        fe_argv_func: None,
        fe_firstline: 0,
        fe_lastline: 0,
        fe_doesrange: ::core::ptr::null_mut::<bool>(),
        fe_evaluate: false,
        fe_partial: ::core::ptr::null_mut::<partial_T>(),
        fe_selfdict: ::core::ptr::null_mut::<dict_T>(),
        fe_basetv: ::core::ptr::null_mut::<typval_T>(),
        fe_found_var: false,
    };
    let mut argv: [typval_T; 21] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 21];
    let mut argc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut r: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let l_: *mut list_T = (*args).vval.v_list;
    '_func_call_skip_call: {
        's_51: {
            if !l_.is_null() {
                let mut item: *mut listitem_T = (*l_).lv_first;
                loop {
                    if item.is_null() {
                        break 's_51;
                    }
                    if argc
                        == MAX_FUNC_ARGS as ::core::ffi::c_int
                            - (if partial.is_null() {
                                0 as ::core::ffi::c_int
                            } else {
                                (*partial).pt_argc
                            })
                    {
                        emsg(gettext(
                            b"E699: Too many arguments\0".as_ptr() as *const ::core::ffi::c_char
                        ));
                        break '_func_call_skip_call;
                    } else {
                        let c2rust_fresh11 = argc;
                        argc = argc + 1;
                        tv_copy(
                            &raw mut (*item).li_tv,
                            (&raw mut argv as *mut typval_T).offset(c2rust_fresh11 as isize),
                        );
                        item = (*item).li_next;
                    }
                }
            }
        }
        funcexe = FUNCEXE_INIT;
        funcexe.fe_firstline = (*curwin).w_cursor.lnum;
        funcexe.fe_lastline = (*curwin).w_cursor.lnum;
        funcexe.fe_evaluate = true_0 != 0;
        funcexe.fe_partial = partial;
        funcexe.fe_selfdict = selfdict;
        r = call_func(
            name,
            -1 as ::core::ffi::c_int,
            rettv,
            argc,
            &raw mut argv as *mut typval_T,
            &raw mut funcexe,
        );
    }
    while argc > 0 as ::core::ffi::c_int {
        argc -= 1;
        tv_clear((&raw mut argv as *mut typval_T).offset(argc as isize));
    }
    return r;
}
#[no_mangle]
pub unsafe extern "C" fn callback_call_retnr(
    mut callback: *mut Callback,
    mut argcount: ::core::ffi::c_int,
    mut argvars: *mut typval_T,
) -> varnumber_T {
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    if !callback_call(callback, argcount, argvars, &raw mut rettv) {
        return -2 as varnumber_T;
    }
    let mut retval: varnumber_T =
        tv_get_number_chk(&raw mut rettv, ::core::ptr::null_mut::<bool>());
    tv_clear(&raw mut rettv);
    return retval;
}
unsafe extern "C" fn user_func_error(
    mut error: ::core::ffi::c_int,
    mut name: *const ::core::ffi::c_char,
    mut found_var: bool,
) {
    match error {
        0 => {
            if found_var {
                semsg(
                    gettext(&raw const e_not_callable_type_str as *const ::core::ffi::c_char),
                    name,
                );
            } else {
                emsg_funcname(
                    &raw const e_unknown_function_str as *const ::core::ffi::c_char,
                    name,
                );
            }
        }
        8 => {
            emsg_funcname(
                b"E276: Cannot use function as a method: %s\0".as_ptr()
                    as *const ::core::ffi::c_char,
                name,
            );
        }
        7 => {
            emsg_funcname(
                b"E933: Function was deleted: %s\0".as_ptr() as *const ::core::ffi::c_char,
                name,
            );
        }
        1 => {
            emsg_funcname(
                gettext(&raw const e_toomanyarg as *const ::core::ffi::c_char),
                name,
            );
        }
        2 => {
            emsg_funcname(
                gettext(&raw const e_toofewarg as *const ::core::ffi::c_char),
                name,
            );
        }
        3 => {
            emsg_funcname(
                b"E120: Using <SID> not in a script context: %s\0".as_ptr()
                    as *const ::core::ffi::c_char,
                name,
            );
        }
        4 => {
            emsg_funcname(
                b"E725: Calling dict function without Dictionary: %s\0".as_ptr()
                    as *const ::core::ffi::c_char,
                name,
            );
        }
        _ => {}
    };
}
unsafe extern "C" fn argv_add_base(
    basetv: *mut typval_T,
    argvars: *mut *mut typval_T,
    argcount: *mut ::core::ffi::c_int,
    new_argvars: *mut typval_T,
    argv_base: *mut ::core::ffi::c_int,
) {
    if !basetv.is_null() {
        memmove(
            new_argvars.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            *argvars as *const ::core::ffi::c_void,
            ::core::mem::size_of::<typval_T>().wrapping_mul(*argcount as size_t),
        );
        *new_argvars.offset(0 as ::core::ffi::c_int as isize) = *basetv;
        *argcount += 1;
        *argvars = new_argvars;
        *argv_base = 1 as ::core::ffi::c_int;
    }
}
#[no_mangle]
pub unsafe extern "C" fn call_func(
    mut funcname: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut rettv: *mut typval_T,
    mut argcount_in: ::core::ffi::c_int,
    mut argvars_in: *mut typval_T,
    mut funcexe: *mut funcexe_T,
) -> ::core::ffi::c_int {
    let mut ret: ::core::ffi::c_int = FAIL;
    let mut error: ::core::ffi::c_int = FCERR_NONE as ::core::ffi::c_int;
    let mut fp: *mut ufunc_T = ::core::ptr::null_mut::<ufunc_T>();
    let mut fname_buf: [::core::ffi::c_char; 41] = [0; 41];
    let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut argcount: ::core::ffi::c_int = argcount_in;
    let mut argvars: *mut typval_T = argvars_in;
    let mut selfdict: *mut dict_T = (*funcexe).fe_selfdict;
    let mut argv: [typval_T; 21] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 21];
    let mut argv_clear: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut argv_base: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut partial: *mut partial_T = (*funcexe).fe_partial;
    (*rettv).v_type = VAR_UNKNOWN;
    if len <= 0 as ::core::ffi::c_int {
        len = strlen(funcname) as ::core::ffi::c_int;
    }
    if !partial.is_null() {
        fp = (*partial).pt_func;
    }
    if fp.is_null() {
        name = xmemdupz(funcname as *const ::core::ffi::c_void, len as size_t)
            as *mut ::core::ffi::c_char;
        fname = fname_trans_sid(
            name,
            &raw mut fname_buf as *mut ::core::ffi::c_char,
            &raw mut tofree,
            &raw mut error,
        );
    }
    if !(*funcexe).fe_doesrange.is_null() {
        *(*funcexe).fe_doesrange = false_0 != 0;
    }
    '_theend: {
        if !partial.is_null() {
            if !(*partial).pt_dict.is_null() && (selfdict.is_null() || !(*partial).pt_auto) {
                selfdict = (*partial).pt_dict;
            }
            if error == FCERR_NONE as ::core::ffi::c_int
                && (*partial).pt_argc > 0 as ::core::ffi::c_int
            {
                argv_clear = 0 as ::core::ffi::c_int;
                while argv_clear < (*partial).pt_argc {
                    if argv_clear + argcount_in >= MAX_FUNC_ARGS as ::core::ffi::c_int {
                        error = FCERR_TOOMANY as ::core::ffi::c_int;
                        break '_theend;
                    } else {
                        tv_copy(
                            (*partial).pt_argv.offset(argv_clear as isize),
                            (&raw mut argv as *mut typval_T).offset(argv_clear as isize),
                        );
                        argv_clear += 1;
                    }
                }
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < argcount_in {
                    argv[(i + argv_clear) as usize] = *argvars_in.offset(i as isize);
                    i += 1;
                }
                argvars = &raw mut argv as *mut typval_T;
                argcount = (*partial).pt_argc + argcount_in;
            }
        }
        if error == FCERR_NONE as ::core::ffi::c_int
            && (*funcexe).fe_evaluate as ::core::ffi::c_int != 0
        {
            let mut is_global: bool = fp.is_null()
                && *fname.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 'g' as ::core::ffi::c_int
                && *fname.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ':' as ::core::ffi::c_int;
            let mut rfname: *mut ::core::ffi::c_char = if is_global as ::core::ffi::c_int != 0 {
                fname.offset(2 as ::core::ffi::c_int as isize)
            } else {
                fname
            };
            (*rettv).v_type = VAR_NUMBER;
            (*rettv).vval.v_number = 0 as varnumber_T;
            error = FCERR_UNKNOWN as ::core::ffi::c_int;
            if is_luafunc(partial) {
                if len > 0 as ::core::ffi::c_int {
                    error = FCERR_NONE as ::core::ffi::c_int;
                    argv_add_base(
                        (*funcexe).fe_basetv,
                        &raw mut argvars,
                        &raw mut argcount,
                        &raw mut argv as *mut typval_T,
                        &raw mut argv_base,
                    );
                    nlua_typval_call(funcname, len as size_t, argvars, argcount, rettv);
                } else {
                    let mut ptr_: *mut *mut ::core::ffi::c_void =
                        &raw mut name as *mut *mut ::core::ffi::c_void;
                    xfree(*ptr_);
                    *ptr_ = NULL;
                    *ptr_;
                    funcname = b"v:lua\0".as_ptr() as *const ::core::ffi::c_char;
                }
            } else if !fp.is_null() || !builtin_function(rfname, -1 as ::core::ffi::c_int) {
                if fp.is_null() {
                    fp = find_func(rfname);
                }
                if fp.is_null()
                    && apply_autocmds(
                        EVENT_FUNCUNDEFINED,
                        rfname,
                        rfname,
                        true_0 != 0,
                        ::core::ptr::null_mut::<buf_T>(),
                    ) as ::core::ffi::c_int
                        != 0
                    && !aborting()
                {
                    fp = find_func(rfname);
                }
                if fp.is_null()
                    && script_autoload(rfname, strlen(rfname), true_0 != 0) as ::core::ffi::c_int
                        != 0
                    && !aborting()
                {
                    fp = find_func(rfname);
                }
                if !fp.is_null() && (*fp).uf_flags & FC_DELETED != 0 {
                    error = FCERR_DELETED as ::core::ffi::c_int;
                } else if !fp.is_null() {
                    if (*funcexe).fe_argv_func.is_some() {
                        argcount = (*funcexe).fe_argv_func.expect("non-null function pointer")(
                            argcount, argvars, argv_clear, fp,
                        );
                    }
                    argv_add_base(
                        (*funcexe).fe_basetv,
                        &raw mut argvars,
                        &raw mut argcount,
                        &raw mut argv as *mut typval_T,
                        &raw mut argv_base,
                    );
                    error = call_user_func_check(fp, argcount, argvars, rettv, funcexe, selfdict);
                }
            } else if !(*funcexe).fe_basetv.is_null() {
                error = call_internal_method(fname, argcount, argvars, rettv, (*funcexe).fe_basetv);
            } else {
                error = call_internal_func(fname, argcount, argvars, rettv);
            }
            update_force_abort();
        }
        if error == FCERR_NONE as ::core::ffi::c_int {
            ret = OK;
        }
    }
    if !aborting() {
        user_func_error(
            error,
            if !name.is_null() {
                name as *const ::core::ffi::c_char
            } else {
                funcname
            },
            (*funcexe).fe_found_var,
        );
    }
    while argv_clear > 0 as ::core::ffi::c_int {
        argv_clear -= 1;
        tv_clear((&raw mut argv as *mut typval_T).offset((argv_clear + argv_base) as isize));
    }
    xfree(tofree as *mut ::core::ffi::c_void);
    xfree(name as *mut ::core::ffi::c_void);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn call_simple_luafunc(
    mut funcname: *const ::core::ffi::c_char,
    mut len: size_t,
    mut rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = 0 as varnumber_T;
    let mut argvars: [typval_T; 1] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 1];
    argvars[0 as ::core::ffi::c_int as usize].v_type = VAR_UNKNOWN;
    nlua_typval_call(
        funcname,
        len,
        &raw mut argvars as *mut typval_T,
        0 as ::core::ffi::c_int,
        rettv,
    );
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn call_simple_func(
    mut funcname: *const ::core::ffi::c_char,
    mut len: size_t,
    mut rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    let mut ret: ::core::ffi::c_int = FAIL;
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = 0 as varnumber_T;
    let mut name: *mut ::core::ffi::c_char = xstrnsave(funcname, len);
    let mut error: ::core::ffi::c_int = FCERR_NONE as ::core::ffi::c_int;
    let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut fname_buf: [::core::ffi::c_char; 41] = [0; 41];
    let mut fname: *mut ::core::ffi::c_char = fname_trans_sid(
        name,
        &raw mut fname_buf as *mut ::core::ffi::c_char,
        &raw mut tofree,
        &raw mut error,
    );
    let mut is_global: bool = *fname.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 'g' as ::core::ffi::c_int
        && *fname.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == ':' as ::core::ffi::c_int;
    let mut rfname: *mut ::core::ffi::c_char = if is_global as ::core::ffi::c_int != 0 {
        fname.offset(2 as ::core::ffi::c_int as isize)
    } else {
        fname
    };
    let mut fp: *mut ufunc_T = find_func(rfname);
    if fp.is_null() {
        ret = NOTDONE;
    } else if !fp.is_null() && (*fp).uf_flags & FC_DELETED != 0 {
        error = FCERR_DELETED as ::core::ffi::c_int;
    } else if !fp.is_null() {
        let mut argvars: [typval_T; 1] = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 1];
        argvars[0 as ::core::ffi::c_int as usize].v_type = VAR_UNKNOWN;
        let mut funcexe: funcexe_T = FUNCEXE_INIT;
        funcexe.fe_evaluate = true_0 != 0;
        error = call_user_func_check(
            fp,
            0 as ::core::ffi::c_int,
            &raw mut argvars as *mut typval_T,
            rettv,
            &raw mut funcexe,
            ::core::ptr::null_mut::<dict_T>(),
        );
        if error == FCERR_NONE as ::core::ffi::c_int {
            ret = OK;
        }
    }
    user_func_error(error, name, false_0 != 0);
    xfree(tofree as *mut ::core::ffi::c_void);
    xfree(name as *mut ::core::ffi::c_void);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn printable_func_name(mut fp: *mut ufunc_T) -> *mut ::core::ffi::c_char {
    return if !(*fp).uf_name_exp.is_null() {
        (*fp).uf_name_exp
    } else {
        &raw mut (*fp).uf_name as *mut ::core::ffi::c_char
    };
}
unsafe extern "C" fn function_list_modified(
    prev_ht_changed: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if prev_ht_changed != func_hashtab.ht_changed {
        emsg(gettext(
            &raw const e_function_list_was_modified as *const ::core::ffi::c_char,
        ));
        return true_0;
    }
    return false_0;
}
unsafe extern "C" fn list_func_head(
    mut fp: *mut ufunc_T,
    mut indent: bool,
    mut force: bool,
) -> ::core::ffi::c_int {
    let prev_ht_changed: ::core::ffi::c_int = func_hashtab.ht_changed;
    msg_start();
    if function_list_modified(prev_ht_changed) != 0 {
        return FAIL;
    }
    if indent {
        msg_puts(b"   \0".as_ptr() as *const ::core::ffi::c_char);
    }
    msg_puts(if force as ::core::ffi::c_int != 0 {
        b"function! \0".as_ptr() as *const ::core::ffi::c_char
    } else {
        b"function \0".as_ptr() as *const ::core::ffi::c_char
    });
    if !(*fp).uf_name_exp.is_null() {
        msg_puts((*fp).uf_name_exp);
    } else {
        msg_puts(&raw mut (*fp).uf_name as *mut ::core::ffi::c_char);
    }
    msg_putchar('(' as ::core::ffi::c_int);
    let mut j: ::core::ffi::c_int = 0;
    j = 0 as ::core::ffi::c_int;
    while j < (*fp).uf_args.ga_len {
        if j != 0 {
            msg_puts(b", \0".as_ptr() as *const ::core::ffi::c_char);
        }
        msg_puts(*((*fp).uf_args.ga_data as *mut *mut ::core::ffi::c_char).offset(j as isize));
        if j >= (*fp).uf_args.ga_len - (*fp).uf_def_args.ga_len {
            msg_puts(b" = \0".as_ptr() as *const ::core::ffi::c_char);
            msg_puts(
                *((*fp).uf_def_args.ga_data as *mut *mut ::core::ffi::c_char)
                    .offset((j - (*fp).uf_args.ga_len + (*fp).uf_def_args.ga_len) as isize),
            );
        }
        j += 1;
    }
    if (*fp).uf_varargs != 0 {
        if j != 0 {
            msg_puts(b", \0".as_ptr() as *const ::core::ffi::c_char);
        }
        msg_puts(b"...\0".as_ptr() as *const ::core::ffi::c_char);
    }
    msg_putchar(')' as ::core::ffi::c_int);
    if (*fp).uf_flags & FC_ABORT != 0 {
        msg_puts(b" abort\0".as_ptr() as *const ::core::ffi::c_char);
    }
    if (*fp).uf_flags & FC_RANGE != 0 {
        msg_puts(b" range\0".as_ptr() as *const ::core::ffi::c_char);
    }
    if (*fp).uf_flags & FC_DICT != 0 {
        msg_puts(b" dict\0".as_ptr() as *const ::core::ffi::c_char);
    }
    if (*fp).uf_flags & FC_CLOSURE != 0 {
        msg_puts(b" closure\0".as_ptr() as *const ::core::ffi::c_char);
    }
    msg_clr_eos();
    if p_verbose > 0 as OptInt {
        last_set_msg((*fp).uf_script_ctx);
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn trans_function_name(
    mut pp: *mut *mut ::core::ffi::c_char,
    mut skip: bool,
    mut flags: ::core::ffi::c_int,
    mut fdp: *mut funcdict_T,
    mut partial: *mut *mut partial_T,
) -> *mut ::core::ffi::c_char {
    let mut sid_buflen: size_t = 0;
    let mut sid_buf: [::core::ffi::c_char; 20] = [0; 20];
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: ::core::ffi::c_int = 0;
    let mut lv: lval_T = lval_T {
        ll_name: ::core::ptr::null::<::core::ffi::c_char>(),
        ll_name_len: 0,
        ll_exp_name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ll_tv: ::core::ptr::null_mut::<typval_T>(),
        ll_li: ::core::ptr::null_mut::<listitem_T>(),
        ll_list: ::core::ptr::null_mut::<list_T>(),
        ll_range: false,
        ll_empty2: false,
        ll_n1: 0,
        ll_n2: 0,
        ll_dict: ::core::ptr::null_mut::<dict_T>(),
        ll_di: ::core::ptr::null_mut::<dictitem_T>(),
        ll_newkey: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ll_blob: ::core::ptr::null_mut::<blob_T>(),
    };
    if !fdp.is_null() {
        memset(
            fdp as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<funcdict_T>(),
        );
    }
    let mut start: *const ::core::ffi::c_char = *pp;
    if *(*pp).offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int == K_SPECIAL
        && *(*pp).offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            == KS_EXTRA
        && *(*pp).offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == KE_SNR as ::core::ffi::c_int
    {
        *pp = (*pp).offset(3 as ::core::ffi::c_int as isize);
        len = get_id_len(pp as *mut *const ::core::ffi::c_char) + 3 as ::core::ffi::c_int;
        return xmemdupz(start as *const ::core::ffi::c_void, len as size_t)
            as *mut ::core::ffi::c_char;
    }
    let mut lead: ::core::ffi::c_int = eval_fname_script(start);
    if lead > 2 as ::core::ffi::c_int {
        start = start.offset(lead as isize);
    }
    let mut end: *const ::core::ffi::c_char = get_lval(
        start as *mut ::core::ffi::c_char,
        ::core::ptr::null_mut::<typval_T>(),
        &raw mut lv,
        false_0 != 0,
        skip,
        flags | GLV_READ_ONLY as ::core::ffi::c_int,
        if lead > 2 as ::core::ffi::c_int {
            0 as ::core::ffi::c_int
        } else {
            FNE_CHECK_START
        },
    );
    '_theend: {
        if end == start {
            if !skip {
                emsg(gettext(
                    b"E129: Function name required\0".as_ptr() as *const ::core::ffi::c_char
                ));
            }
        } else if end.is_null()
            || !lv.ll_tv.is_null()
                && (lead > 2 as ::core::ffi::c_int || lv.ll_range as ::core::ffi::c_int != 0)
        {
            if !aborting() {
                if !end.is_null() {
                    semsg(
                        gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                        start,
                    );
                }
            } else {
                *pp = find_name_end(
                    start,
                    ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
                    FNE_INCL_BR,
                ) as *mut ::core::ffi::c_char;
            }
        } else if !lv.ll_tv.is_null() {
            if !fdp.is_null() {
                (*fdp).fd_dict = lv.ll_dict;
                (*fdp).fd_newkey = lv.ll_newkey;
                lv.ll_newkey = ::core::ptr::null_mut::<::core::ffi::c_char>();
                (*fdp).fd_di = lv.ll_di;
            }
            if (*lv.ll_tv).v_type as ::core::ffi::c_uint
                == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
                && !(*lv.ll_tv).vval.v_string.is_null()
            {
                name = xstrdup((*lv.ll_tv).vval.v_string);
                *pp = end as *mut ::core::ffi::c_char;
            } else if (*lv.ll_tv).v_type as ::core::ffi::c_uint
                == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
                && !(*lv.ll_tv).vval.v_partial.is_null()
            {
                if is_luafunc((*lv.ll_tv).vval.v_partial) as ::core::ffi::c_int != 0
                    && *end as ::core::ffi::c_int == '.' as ::core::ffi::c_int
                {
                    len = check_luafunc_name(
                        end.offset(1 as ::core::ffi::c_int as isize),
                        true_0 != 0,
                    );
                    if len == 0 as ::core::ffi::c_int {
                        semsg(
                            &raw const e_invexpr2 as *const ::core::ffi::c_char,
                            b"v:lua\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                        break '_theend;
                    } else {
                        name = xmallocz(len as size_t) as *mut ::core::ffi::c_char;
                        memcpy(
                            name as *mut ::core::ffi::c_void,
                            end.offset(1 as ::core::ffi::c_int as isize)
                                as *const ::core::ffi::c_void,
                            len as size_t,
                        );
                        *pp = (end as *mut ::core::ffi::c_char)
                            .offset(1 as ::core::ffi::c_int as isize)
                            .offset(len as isize);
                    }
                } else {
                    name = xstrdup(partial_name((*lv.ll_tv).vval.v_partial));
                    *pp = end as *mut ::core::ffi::c_char;
                }
                if !partial.is_null() {
                    *partial = (*lv.ll_tv).vval.v_partial;
                }
            } else {
                if !skip
                    && flags & TFN_QUIET as ::core::ffi::c_int == 0
                    && (fdp.is_null() || lv.ll_dict.is_null() || (*fdp).fd_newkey.is_null())
                {
                    emsg(gettext(e_funcref));
                } else {
                    *pp = end as *mut ::core::ffi::c_char;
                }
                name = ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
        } else if lv.ll_name.is_null() {
            *pp = end as *mut ::core::ffi::c_char;
        } else {
            if !lv.ll_exp_name.is_null() {
                len = strlen(lv.ll_exp_name) as ::core::ffi::c_int;
                name = deref_func_name(
                    lv.ll_exp_name,
                    &raw mut len,
                    partial,
                    flags & TFN_NO_AUTOLOAD as ::core::ffi::c_int != 0,
                    ::core::ptr::null_mut::<bool>(),
                );
                if name == lv.ll_exp_name {
                    name = ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
            } else if flags & TFN_NO_DEREF as ::core::ffi::c_int == 0 {
                len = end.offset_from(*pp) as ::core::ffi::c_int;
                name = deref_func_name(
                    *pp,
                    &raw mut len,
                    partial,
                    flags & TFN_NO_AUTOLOAD as ::core::ffi::c_int != 0,
                    ::core::ptr::null_mut::<bool>(),
                );
                if name == *pp {
                    name = ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
            }
            if !name.is_null() {
                name = xstrdup(name);
                *pp = end as *mut ::core::ffi::c_char;
                if strncmp(
                    name,
                    b"<SNR>\0".as_ptr() as *const ::core::ffi::c_char,
                    5 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    *name.offset(0 as ::core::ffi::c_int as isize) =
                        K_SPECIAL as ::core::ffi::c_char;
                    *name.offset(1 as ::core::ffi::c_int as isize) =
                        KS_EXTRA as ::core::ffi::c_char;
                    *name.offset(2 as ::core::ffi::c_int as isize) =
                        KE_SNR as ::core::ffi::c_int as ::core::ffi::c_char;
                    memmove(
                        name.offset(3 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                        name.offset(5 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                        strlen(name.offset(5 as ::core::ffi::c_int as isize))
                            .wrapping_add(1 as size_t),
                    );
                }
            } else {
                if !lv.ll_exp_name.is_null() {
                    len = strlen(lv.ll_exp_name) as ::core::ffi::c_int;
                    if lead <= 2 as ::core::ffi::c_int
                        && lv.ll_name == lv.ll_exp_name as *const ::core::ffi::c_char
                        && lv.ll_name_len >= 2 as size_t
                        && memcmp(
                            lv.ll_name as *const ::core::ffi::c_void,
                            b"s:\0".as_ptr() as *const ::core::ffi::c_char
                                as *const ::core::ffi::c_void,
                            2 as size_t,
                        ) == 0 as ::core::ffi::c_int
                    {
                        lv.ll_name = lv.ll_name.offset(2 as ::core::ffi::c_int as isize);
                        lv.ll_name_len = lv.ll_name_len.wrapping_sub(2 as size_t);
                        len -= 2 as ::core::ffi::c_int;
                        lead = 2 as ::core::ffi::c_int;
                    }
                } else {
                    if lead == 2 as ::core::ffi::c_int
                        || *lv.ll_name.offset(0 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            == 'g' as ::core::ffi::c_int
                            && *lv.ll_name.offset(1 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == ':' as ::core::ffi::c_int
                    {
                        lv.ll_name = lv.ll_name.offset(2 as ::core::ffi::c_int as isize);
                        lv.ll_name_len = lv.ll_name_len.wrapping_sub(2 as size_t);
                    }
                    len = end.offset_from(lv.ll_name) as ::core::ffi::c_int;
                }
                sid_buflen = 0 as size_t;
                sid_buf = [0; 20];
                if skip {
                    lead = 0 as ::core::ffi::c_int;
                } else if lead > 0 as ::core::ffi::c_int {
                    lead = 3 as ::core::ffi::c_int;
                    if !lv.ll_exp_name.is_null()
                        && eval_fname_sid(lv.ll_exp_name) as ::core::ffi::c_int != 0
                        || eval_fname_sid(*pp) as ::core::ffi::c_int != 0
                    {
                        if current_sctx.sc_sid <= 0 as ::core::ffi::c_int {
                            emsg(gettext(&raw const e_usingsid as *const ::core::ffi::c_char));
                            break '_theend;
                        } else {
                            sid_buflen = snprintf(
                                &raw mut sid_buf as *mut ::core::ffi::c_char,
                                ::core::mem::size_of::<[::core::ffi::c_char; 20]>(),
                                b"%d_\0".as_ptr() as *const ::core::ffi::c_char,
                                current_sctx.sc_sid,
                            ) as size_t;
                            lead += sid_buflen as ::core::ffi::c_int;
                        }
                    }
                } else if flags & TFN_INT as ::core::ffi::c_int == 0
                    && builtin_function(lv.ll_name, lv.ll_name_len as ::core::ffi::c_int)
                        as ::core::ffi::c_int
                        != 0
                {
                    semsg(
                        gettext(
                            b"E128: Function name must start with a capital or \"s:\": %s\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ),
                        start,
                    );
                    break '_theend;
                }
                if !skip
                    && flags & TFN_QUIET as ::core::ffi::c_int == 0
                    && flags & TFN_NO_DEREF as ::core::ffi::c_int == 0
                {
                    let mut cp: *mut ::core::ffi::c_char = xmemrchr(
                        lv.ll_name as *const ::core::ffi::c_void,
                        ':' as uint8_t,
                        lv.ll_name_len,
                    )
                        as *mut ::core::ffi::c_char;
                    if !cp.is_null() && cp < end as *mut ::core::ffi::c_char {
                        semsg(
                            gettext(b"E884: Function name cannot contain a colon: %s\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            start,
                        );
                        break '_theend;
                    }
                }
                name = xmalloc(
                    (len as size_t)
                        .wrapping_add(lead as size_t)
                        .wrapping_add(1 as size_t),
                ) as *mut ::core::ffi::c_char;
                if !skip && lead > 0 as ::core::ffi::c_int {
                    *name.offset(0 as ::core::ffi::c_int as isize) =
                        K_SPECIAL as ::core::ffi::c_char;
                    *name.offset(1 as ::core::ffi::c_int as isize) =
                        KS_EXTRA as ::core::ffi::c_char;
                    *name.offset(2 as ::core::ffi::c_int as isize) =
                        KE_SNR as ::core::ffi::c_int as ::core::ffi::c_char;
                    if sid_buflen > 0 as size_t {
                        memcpy(
                            name.offset(3 as ::core::ffi::c_int as isize)
                                as *mut ::core::ffi::c_void,
                            &raw mut sid_buf as *mut ::core::ffi::c_char
                                as *const ::core::ffi::c_void,
                            sid_buflen,
                        );
                    }
                }
                memmove(
                    name.offset(lead as isize) as *mut ::core::ffi::c_void,
                    lv.ll_name as *const ::core::ffi::c_void,
                    len as size_t,
                );
                *name.offset((lead + len) as isize) = NUL as ::core::ffi::c_char;
                *pp = end as *mut ::core::ffi::c_char;
            }
        }
    }
    clear_lval(&raw mut lv);
    return name;
}
#[no_mangle]
pub unsafe extern "C" fn get_scriptlocal_funcname(
    mut funcname: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if funcname.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if strncmp(
        funcname,
        b"s:\0".as_ptr() as *const ::core::ffi::c_char,
        2 as size_t,
    ) != 0 as ::core::ffi::c_int
        && strncmp(
            funcname,
            b"<SID>\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) != 0 as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if !(current_sctx.sc_sid > 0 as ::core::ffi::c_int
        && current_sctx.sc_sid <= script_items.ga_len)
    {
        emsg(gettext(&raw const e_usingsid as *const ::core::ffi::c_char));
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut sid_buf: [::core::ffi::c_char; 25] = [0; 25];
    let mut sid_buflen: size_t = snprintf(
        &raw mut sid_buf as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 25]>(),
        b"<SNR>%d_\0".as_ptr() as *const ::core::ffi::c_char,
        current_sctx.sc_sid,
    ) as size_t;
    let off: ::core::ffi::c_int = if *funcname as ::core::ffi::c_int == 's' as ::core::ffi::c_int {
        2 as ::core::ffi::c_int
    } else {
        5 as ::core::ffi::c_int
    };
    let mut newnamesize: size_t = sid_buflen
        .wrapping_add(strlen(funcname.offset(off as isize)))
        .wrapping_add(1 as size_t);
    let mut newname: *mut ::core::ffi::c_char = xmalloc(newnamesize) as *mut ::core::ffi::c_char;
    snprintf(
        newname,
        newnamesize,
        b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
        &raw mut sid_buf as *mut ::core::ffi::c_char,
        funcname.offset(off as isize),
    );
    return newname;
}
#[no_mangle]
pub unsafe extern "C" fn save_function_name(
    mut name: *mut *mut ::core::ffi::c_char,
    mut skip: bool,
    mut flags: ::core::ffi::c_int,
    mut fudi: *mut funcdict_T,
) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = *name;
    let mut saved: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if strncmp(
        p,
        b"<lambda>\0".as_ptr() as *const ::core::ffi::c_char,
        8 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        p = p.offset(8 as ::core::ffi::c_int as isize);
        getdigits(&raw mut p, false_0 != 0, 0 as intmax_t);
        saved = xmemdupz(
            *name as *const ::core::ffi::c_void,
            p.offset_from(*name) as size_t,
        ) as *mut ::core::ffi::c_char;
        if !fudi.is_null() {
            memset(
                fudi as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<funcdict_T>(),
            );
        }
    } else {
        saved = trans_function_name(
            &raw mut p,
            skip,
            flags,
            fudi,
            ::core::ptr::null_mut::<*mut partial_T>(),
        );
    }
    *name = p;
    return saved;
}
unsafe extern "C" fn list_functions(mut regmatch: *mut regmatch_T) {
    let prev_ht_changed: ::core::ffi::c_int = func_hashtab.ht_changed;
    let mut todo: size_t = func_hashtab.ht_used;
    let ht_array: *const hashitem_T = func_hashtab.ht_array;
    msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
    let mut hi: *const hashitem_T = ht_array;
    while todo > 0 as size_t && !got_int {
        if !((*hi).hi_key.is_null() || (*hi).hi_key == &raw mut hash_removed) {
            let mut fp: *mut ufunc_T =
                (*hi).hi_key.offset(-(240 as ::core::ffi::c_ulong as isize)) as *mut ufunc_T;
            todo = todo.wrapping_sub(1);
            if if regmatch.is_null() {
                (!message_filtered(&raw mut (*fp).uf_name as *mut ::core::ffi::c_char)
                    && !func_name_refcount(&raw mut (*fp).uf_name as *mut ::core::ffi::c_char))
                    as ::core::ffi::c_int
            } else {
                (*(*__ctype_b_loc()).offset(
                    *(&raw mut (*fp).uf_name as *mut ::core::ffi::c_char) as uint8_t
                        as ::core::ffi::c_int as isize,
                ) as ::core::ffi::c_int
                    & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    == 0
                    && vim_regexec(
                        regmatch,
                        &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
                        0 as colnr_T,
                    ) as ::core::ffi::c_int
                        != 0) as ::core::ffi::c_int
            } != 0
            {
                if list_func_head(fp, false_0 != 0, false_0 != 0) == FAIL {
                    return;
                }
                if function_list_modified(prev_ht_changed) != 0 {
                    return;
                }
            }
        }
        hi = hi.offset(1);
    }
}
unsafe extern "C" fn list_functions_matching_pat(
    mut eap: *mut exarg_T,
) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = skip_regexp(
        (*eap).arg.offset(1 as ::core::ffi::c_int as isize),
        '/' as ::core::ffi::c_int,
        true_0,
    );
    if (*eap).skip == 0 {
        let mut regmatch: regmatch_T = regmatch_T {
            regprog: ::core::ptr::null_mut::<regprog_T>(),
            startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            rm_matchcol: 0,
            rm_ic: false,
        };
        let mut c: ::core::ffi::c_char = *p;
        *p = NUL as ::core::ffi::c_char;
        regmatch.regprog = vim_regcomp(
            (*eap).arg.offset(1 as ::core::ffi::c_int as isize),
            RE_MAGIC,
        );
        *p = c;
        if !regmatch.regprog.is_null() {
            regmatch.rm_ic = p_ic != 0;
            list_functions(&raw mut regmatch);
            vim_regfree(regmatch.regprog);
        }
    }
    if *p as ::core::ffi::c_int == '/' as ::core::ffi::c_int {
        p = p.offset(1);
    }
    return p;
}
unsafe extern "C" fn list_one_function(
    mut eap: *mut exarg_T,
    mut name: *mut ::core::ffi::c_char,
    mut p: *mut ::core::ffi::c_char,
) -> *mut ufunc_T {
    if ends_excmd(*skipwhite(p) as ::core::ffi::c_int) == 0 {
        semsg(
            gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
            p,
        );
        return ::core::ptr::null_mut::<ufunc_T>();
    }
    (*eap).nextcmd = check_nextcmd(p);
    if !(*eap).nextcmd.is_null() {
        *p = NUL as ::core::ffi::c_char;
    }
    if (*eap).skip != 0 || got_int as ::core::ffi::c_int != 0 {
        return ::core::ptr::null_mut::<ufunc_T>();
    }
    let mut fp: *mut ufunc_T = find_func(name);
    if fp.is_null() {
        emsg_funcname(
            b"E123: Undefined function: %s\0".as_ptr() as *const ::core::ffi::c_char,
            name,
        );
        return ::core::ptr::null_mut::<ufunc_T>();
    }
    let prev_ht_changed: ::core::ffi::c_int = func_hashtab.ht_changed;
    msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
    if list_func_head(fp, (*eap).forceit == 0, (*eap).forceit != 0) != OK {
        return fp;
    }
    let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while j < (*fp).uf_lines.ga_len && !got_int {
        if !(*((*fp).uf_lines.ga_data as *mut *mut ::core::ffi::c_char).offset(j as isize))
            .is_null()
        {
            msg_putchar('\n' as ::core::ffi::c_int);
            if (*eap).forceit == 0 {
                msg_outnum(j + 1 as ::core::ffi::c_int);
                if j < 9 as ::core::ffi::c_int {
                    msg_putchar(' ' as ::core::ffi::c_int);
                }
                if j < 99 as ::core::ffi::c_int {
                    msg_putchar(' ' as ::core::ffi::c_int);
                }
                if function_list_modified(prev_ht_changed) != 0 {
                    break;
                }
            }
            msg_prt_line(
                *((*fp).uf_lines.ga_data as *mut *mut ::core::ffi::c_char).offset(j as isize),
                false_0 != 0,
            );
            line_breakcheck();
        }
        j += 1;
    }
    if !got_int {
        msg_putchar('\n' as ::core::ffi::c_int);
        if function_list_modified(prev_ht_changed) == 0 {
            msg_puts(if (*eap).forceit != 0 {
                b"endfunction\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"   endfunction\0".as_ptr() as *const ::core::ffi::c_char
            });
        }
    }
    return fp;
}
pub const MAX_FUNC_NESTING: ::core::ffi::c_int = 50 as ::core::ffi::c_int;
unsafe extern "C" fn get_function_body(
    mut eap: *mut exarg_T,
    mut newlines: *mut garray_T,
    mut line_arg_in: *mut ::core::ffi::c_char,
    mut line_to_free: *mut *mut ::core::ffi::c_char,
    mut show_block: bool,
) -> ::core::ffi::c_int {
    let mut saved_wait_return: bool = need_wait_return;
    let mut line_arg: *mut ::core::ffi::c_char = line_arg_in;
    let mut indent: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
    let mut nesting: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut skip_until: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ret: ::core::ffi::c_int = FAIL;
    let mut is_heredoc: bool = false_0 != 0;
    let mut heredoc_trimmed: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut heredoc_trimmedlen: size_t = 0 as size_t;
    let mut do_concat: bool = true_0 != 0;
    '_theend: {
        loop {
            if KeyTyped {
                msg_scroll = true_0;
                saved_wait_return = false_0 != 0;
            }
            need_wait_return = false_0 != 0;
            let mut theline: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            if !line_arg.is_null() {
                theline = line_arg;
                p = vim_strchr(theline, '\n' as ::core::ffi::c_int);
                if p.is_null() {
                    line_arg = line_arg.offset(strlen(line_arg) as isize);
                } else {
                    *p = NUL as ::core::ffi::c_char;
                    line_arg = p.offset(1 as ::core::ffi::c_int as isize);
                }
            } else {
                xfree(*line_to_free as *mut ::core::ffi::c_void);
                if (*eap).ea_getline.is_none() {
                    theline = getcmdline(
                        ':' as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        indent,
                        do_concat,
                    );
                } else {
                    theline = (*eap).ea_getline.expect("non-null function pointer")(
                        ':' as ::core::ffi::c_int,
                        (*eap).cookie,
                        indent,
                        do_concat,
                    );
                }
                *line_to_free = theline;
            }
            if KeyTyped {
                lines_left = Rows - 1 as ::core::ffi::c_int;
            }
            if theline.is_null() {
                if !skip_until.is_null() {
                    semsg(
                        gettext(
                            &raw const e_missing_heredoc_end_marker_str
                                as *const ::core::ffi::c_char,
                        ),
                        skip_until,
                    );
                } else {
                    emsg(gettext(
                        b"E126: Missing :endfunction\0".as_ptr() as *const ::core::ffi::c_char
                    ));
                }
                break '_theend;
            } else {
                if show_block {
                    '_c2rust_label: {
                        if indent >= 0 as ::core::ffi::c_int {
                        } else {
                            __assert_fail(
                                b"indent >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                                b"/home/overlord/projects/neovim/neovim/src/nvim/eval/userfunc.c\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                2419 as ::core::ffi::c_uint,
                                b"int get_function_body(exarg_T *, garray_T *, char *, char **, _Bool)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    ui_ext_cmdline_block_append(indent as size_t, theline);
                }
                let mut sourcing_lnum_off: linenr_T =
                    get_sourced_lnum((*eap).ea_getline, (*eap).cookie);
                if (*(exestack.ga_data as *mut estack_T)
                    .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum
                    < sourcing_lnum_off
                {
                    sourcing_lnum_off -= (*(exestack.ga_data as *mut estack_T)
                        .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_lnum;
                } else {
                    sourcing_lnum_off = 0 as ::core::ffi::c_int as linenr_T;
                }
                if !skip_until.is_null() {
                    if heredoc_trimmed.is_null()
                        || is_heredoc as ::core::ffi::c_int != 0 && skipwhite(theline) == theline
                        || strncmp(theline, heredoc_trimmed, heredoc_trimmedlen)
                            == 0 as ::core::ffi::c_int
                    {
                        if heredoc_trimmed.is_null() {
                            p = theline;
                        } else if is_heredoc {
                            p = if skipwhite(theline) == theline {
                                theline
                            } else {
                                theline.offset(heredoc_trimmedlen as isize)
                            };
                        } else {
                            p = theline.offset(heredoc_trimmedlen as isize);
                        }
                        if strcmp(p, skip_until) == 0 as ::core::ffi::c_int {
                            let mut ptr_: *mut *mut ::core::ffi::c_void =
                                &raw mut skip_until as *mut *mut ::core::ffi::c_void;
                            xfree(*ptr_);
                            *ptr_ = NULL;
                            *ptr_;
                            let mut ptr__0: *mut *mut ::core::ffi::c_void =
                                &raw mut heredoc_trimmed as *mut *mut ::core::ffi::c_void;
                            xfree(*ptr__0);
                            *ptr__0 = NULL;
                            *ptr__0;
                            heredoc_trimmedlen = 0 as size_t;
                            do_concat = true_0 != 0;
                            is_heredoc = false_0 != 0;
                        }
                    }
                } else {
                    p = theline;
                    while ascii_iswhite(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                        || *p as ::core::ffi::c_int == ':' as ::core::ffi::c_int
                    {
                        p = p.offset(1);
                    }
                    if checkforcmd(
                        &raw mut p,
                        b"endfunction\0".as_ptr() as *const ::core::ffi::c_char,
                        4 as ::core::ffi::c_int,
                    ) as ::core::ffi::c_int
                        != 0
                        && {
                            let c2rust_fresh12 = nesting;
                            nesting = nesting - 1;
                            c2rust_fresh12 == 0 as ::core::ffi::c_int
                        }
                    {
                        if *p as ::core::ffi::c_int == '!' as ::core::ffi::c_int {
                            p = p.offset(1);
                        }
                        let mut nextcmd: *mut ::core::ffi::c_char =
                            ::core::ptr::null_mut::<::core::ffi::c_char>();
                        if *p as ::core::ffi::c_int == '|' as ::core::ffi::c_int {
                            nextcmd = p.offset(1 as ::core::ffi::c_int as isize);
                        } else if !line_arg.is_null()
                            && *skipwhite(line_arg) as ::core::ffi::c_int != NUL
                        {
                            nextcmd = line_arg;
                        } else if *p as ::core::ffi::c_int != NUL
                            && *p as ::core::ffi::c_int != '"' as ::core::ffi::c_int
                            && p_verbose > 0 as OptInt
                        {
                            swmsg(
                                true_0 != 0,
                                gettext(b"W22: Text found after :endfunction: %s\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                p,
                            );
                        }
                        if !nextcmd.is_null() {
                            (*eap).nextcmd = nextcmd;
                            if !(*line_to_free).is_null() {
                                xfree(*(*eap).cmdlinep as *mut ::core::ffi::c_void);
                                *(*eap).cmdlinep = *line_to_free;
                                *line_to_free = ::core::ptr::null_mut::<::core::ffi::c_char>();
                            }
                        }
                        break;
                    } else {
                        if indent > 2 as ::core::ffi::c_int
                            && strncmp(
                                p,
                                b"end\0".as_ptr() as *const ::core::ffi::c_char,
                                3 as size_t,
                            ) == 0 as ::core::ffi::c_int
                        {
                            indent -= 2 as ::core::ffi::c_int;
                        } else if strncmp(
                            p,
                            b"if\0".as_ptr() as *const ::core::ffi::c_char,
                            2 as size_t,
                        ) == 0 as ::core::ffi::c_int
                            || strncmp(
                                p,
                                b"wh\0".as_ptr() as *const ::core::ffi::c_char,
                                2 as size_t,
                            ) == 0 as ::core::ffi::c_int
                            || strncmp(
                                p,
                                b"for\0".as_ptr() as *const ::core::ffi::c_char,
                                3 as size_t,
                            ) == 0 as ::core::ffi::c_int
                            || strncmp(
                                p,
                                b"try\0".as_ptr() as *const ::core::ffi::c_char,
                                3 as size_t,
                            ) == 0 as ::core::ffi::c_int
                        {
                            indent += 2 as ::core::ffi::c_int;
                        }
                        if checkforcmd(
                            &raw mut p,
                            b"function\0".as_ptr() as *const ::core::ffi::c_char,
                            2 as ::core::ffi::c_int,
                        ) {
                            if *p as ::core::ffi::c_int == '!' as ::core::ffi::c_int {
                                p = skipwhite(p.offset(1 as ::core::ffi::c_int as isize));
                            }
                            p = p.offset(eval_fname_script(p) as isize);
                            xfree(trans_function_name(
                                &raw mut p,
                                true_0 != 0,
                                0 as ::core::ffi::c_int,
                                ::core::ptr::null_mut::<funcdict_T>(),
                                ::core::ptr::null_mut::<*mut partial_T>(),
                            ) as *mut ::core::ffi::c_void);
                            if *skipwhite(p) as ::core::ffi::c_int == '(' as ::core::ffi::c_int {
                                if nesting == MAX_FUNC_NESTING - 1 as ::core::ffi::c_int {
                                    emsg(gettext(
                                        &raw const e_function_nesting_too_deep
                                            as *const ::core::ffi::c_char,
                                    ));
                                } else {
                                    nesting += 1;
                                    indent += 2 as ::core::ffi::c_int;
                                }
                            }
                        }
                        p = skip_range(p, ::core::ptr::null_mut::<::core::ffi::c_int>());
                        let tp: *mut ::core::ffi::c_char = p;
                        if (checkforcmd(
                            &raw mut p,
                            b"append\0".as_ptr() as *const ::core::ffi::c_char,
                            1 as ::core::ffi::c_int,
                        ) as ::core::ffi::c_int
                            != 0
                            || checkforcmd(
                                &raw mut p,
                                b"change\0".as_ptr() as *const ::core::ffi::c_char,
                                1 as ::core::ffi::c_int,
                            ) as ::core::ffi::c_int
                                != 0
                            || checkforcmd(
                                &raw mut p,
                                b"insert\0".as_ptr() as *const ::core::ffi::c_char,
                                1 as ::core::ffi::c_int,
                            ) as ::core::ffi::c_int
                                != 0)
                            && (*p as ::core::ffi::c_int == '!' as ::core::ffi::c_int
                                || *p as ::core::ffi::c_int == '|' as ::core::ffi::c_int
                                || ascii_iswhite_nl_or_nul(*p as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                                    != 0)
                        {
                            skip_until = xmemdupz(
                                b".\0".as_ptr() as *const ::core::ffi::c_char
                                    as *const ::core::ffi::c_void,
                                1 as size_t,
                            ) as *mut ::core::ffi::c_char;
                        } else {
                            p = tp;
                        }
                        arg = skipwhite(skiptowhite(p));
                        if *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '<' as ::core::ffi::c_int
                            && *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == '<' as ::core::ffi::c_int
                            && (*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == 'p' as ::core::ffi::c_int
                                && *p.offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == 'y' as ::core::ffi::c_int
                                && (!(*p.offset(2 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_uint
                                    >= 'A' as ::core::ffi::c_uint
                                    && *p.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_uint
                                        <= 'Z' as ::core::ffi::c_uint
                                    || *p.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_uint
                                        >= 'a' as ::core::ffi::c_uint
                                        && *p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            <= 'z' as ::core::ffi::c_uint
                                    || ascii_isdigit(*p.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int)
                                        as ::core::ffi::c_int
                                        != 0)
                                    || *p.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == 't' as ::core::ffi::c_int
                                    || (*p.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == '3' as ::core::ffi::c_int
                                        || *p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == 'x' as ::core::ffi::c_int)
                                        && !(*p.offset(3 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            >= 'A' as ::core::ffi::c_uint
                                            && *p.offset(3 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_uint
                                                <= 'Z' as ::core::ffi::c_uint
                                            || *p.offset(3 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_uint
                                                >= 'a' as ::core::ffi::c_uint
                                                && *p.offset(3 as ::core::ffi::c_int as isize)
                                                    as ::core::ffi::c_uint
                                                    <= 'z' as ::core::ffi::c_uint))
                                || *p.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == 'p' as ::core::ffi::c_int
                                    && *p.offset(1 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == 'e' as ::core::ffi::c_int
                                    && (!(*p.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_uint
                                        >= 'A' as ::core::ffi::c_uint
                                        && *p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            <= 'Z' as ::core::ffi::c_uint
                                        || *p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            >= 'a' as ::core::ffi::c_uint
                                            && *p.offset(2 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_uint
                                                <= 'z' as ::core::ffi::c_uint)
                                        || *p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == 'r' as ::core::ffi::c_int)
                                || *p.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == 't' as ::core::ffi::c_int
                                    && *p.offset(1 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == 'c' as ::core::ffi::c_int
                                    && (!(*p.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_uint
                                        >= 'A' as ::core::ffi::c_uint
                                        && *p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            <= 'Z' as ::core::ffi::c_uint
                                        || *p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            >= 'a' as ::core::ffi::c_uint
                                            && *p.offset(2 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_uint
                                                <= 'z' as ::core::ffi::c_uint)
                                        || *p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == 'l' as ::core::ffi::c_int)
                                || *p.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == 'l' as ::core::ffi::c_int
                                    && *p.offset(1 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == 'u' as ::core::ffi::c_int
                                    && *p.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == 'a' as ::core::ffi::c_int
                                    && !(*p.offset(3 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_uint
                                        >= 'A' as ::core::ffi::c_uint
                                        && *p.offset(3 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            <= 'Z' as ::core::ffi::c_uint
                                        || *p.offset(3 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            >= 'a' as ::core::ffi::c_uint
                                            && *p.offset(3 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_uint
                                                <= 'z' as ::core::ffi::c_uint)
                                || *p.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == 'r' as ::core::ffi::c_int
                                    && *p.offset(1 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == 'u' as ::core::ffi::c_int
                                    && *p.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == 'b' as ::core::ffi::c_int
                                    && (!(*p.offset(3 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_uint
                                        >= 'A' as ::core::ffi::c_uint
                                        && *p.offset(3 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            <= 'Z' as ::core::ffi::c_uint
                                        || *p.offset(3 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            >= 'a' as ::core::ffi::c_uint
                                            && *p.offset(3 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_uint
                                                <= 'z' as ::core::ffi::c_uint)
                                        || *p.offset(3 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == 'y' as ::core::ffi::c_int)
                                || *p.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == 'm' as ::core::ffi::c_int
                                    && *p.offset(1 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == 'z' as ::core::ffi::c_int
                                    && (!(*p.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_uint
                                        >= 'A' as ::core::ffi::c_uint
                                        && *p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            <= 'Z' as ::core::ffi::c_uint
                                        || *p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            >= 'a' as ::core::ffi::c_uint
                                            && *p.offset(2 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_uint
                                                <= 'z' as ::core::ffi::c_uint)
                                        || *p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == 's' as ::core::ffi::c_int))
                        {
                            p = skipwhite(arg.offset(2 as ::core::ffi::c_int as isize));
                            if strncmp(
                                p,
                                b"trim\0".as_ptr() as *const ::core::ffi::c_char,
                                4 as size_t,
                            ) == 0 as ::core::ffi::c_int
                                && (*p.offset(4 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == NUL
                                    || ascii_iswhite(*p.offset(4 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int)
                                        as ::core::ffi::c_int
                                        != 0)
                            {
                                p = skipwhite(p.offset(4 as ::core::ffi::c_int as isize));
                                heredoc_trimmedlen =
                                    skipwhite(theline).offset_from(theline) as size_t;
                                heredoc_trimmed = xmemdupz(
                                    theline as *const ::core::ffi::c_void,
                                    heredoc_trimmedlen,
                                )
                                    as *mut ::core::ffi::c_char;
                            }
                            if *p as ::core::ffi::c_int == NUL {
                                skip_until = xmemdupz(
                                    b".\0".as_ptr() as *const ::core::ffi::c_char
                                        as *const ::core::ffi::c_void,
                                    1 as size_t,
                                )
                                    as *mut ::core::ffi::c_char;
                            } else {
                                skip_until = xmemdupz(
                                    p as *const ::core::ffi::c_void,
                                    skiptowhite(p).offset_from(p) as size_t,
                                )
                                    as *mut ::core::ffi::c_char;
                            }
                            do_concat = false_0 != 0;
                            is_heredoc = true_0 != 0;
                        }
                        if !is_heredoc {
                            arg = p;
                            if checkforcmd(
                                &raw mut arg,
                                b"let\0".as_ptr() as *const ::core::ffi::c_char,
                                2 as ::core::ffi::c_int,
                            ) as ::core::ffi::c_int
                                != 0
                                || checkforcmd(
                                    &raw mut p,
                                    b"const\0".as_ptr() as *const ::core::ffi::c_char,
                                    5 as ::core::ffi::c_int,
                                ) as ::core::ffi::c_int
                                    != 0
                            {
                                let mut var_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                let mut semicolon: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                arg = skip_var_list(
                                    arg,
                                    &raw mut var_count,
                                    &raw mut semicolon,
                                    true_0 != 0,
                                ) as *mut ::core::ffi::c_char;
                                if !arg.is_null() {
                                    arg = skipwhite(arg);
                                }
                                if !arg.is_null()
                                    && strncmp(
                                        arg,
                                        b"=<<\0".as_ptr() as *const ::core::ffi::c_char,
                                        3 as size_t,
                                    ) == 0 as ::core::ffi::c_int
                                {
                                    p = skipwhite(arg.offset(3 as ::core::ffi::c_int as isize));
                                    let mut has_trim: bool = false_0 != 0;
                                    loop {
                                        if strncmp(
                                            p,
                                            b"trim\0".as_ptr() as *const ::core::ffi::c_char,
                                            4 as size_t,
                                        ) == 0 as ::core::ffi::c_int
                                            && (*p.offset(4 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int
                                                == NUL
                                                || ascii_iswhite(
                                                    *p.offset(4 as ::core::ffi::c_int as isize)
                                                        as ::core::ffi::c_int,
                                                )
                                                    as ::core::ffi::c_int
                                                    != 0)
                                        {
                                            p = skipwhite(
                                                p.offset(4 as ::core::ffi::c_int as isize),
                                            );
                                            has_trim = true_0 != 0;
                                        } else {
                                            if !(strncmp(
                                                p,
                                                b"eval\0".as_ptr() as *const ::core::ffi::c_char,
                                                4 as size_t,
                                            ) == 0 as ::core::ffi::c_int
                                                && (*p.offset(4 as ::core::ffi::c_int as isize)
                                                    as ::core::ffi::c_int
                                                    == NUL
                                                    || ascii_iswhite(
                                                        *p.offset(4 as ::core::ffi::c_int as isize)
                                                            as ::core::ffi::c_int,
                                                    )
                                                        as ::core::ffi::c_int
                                                        != 0))
                                            {
                                                break;
                                            }
                                            p = skipwhite(
                                                p.offset(4 as ::core::ffi::c_int as isize),
                                            );
                                        }
                                    }
                                    if has_trim {
                                        heredoc_trimmedlen =
                                            skipwhite(theline).offset_from(theline) as size_t;
                                        heredoc_trimmed = xmemdupz(
                                            theline as *const ::core::ffi::c_void,
                                            heredoc_trimmedlen,
                                        )
                                            as *mut ::core::ffi::c_char;
                                    }
                                    let mut ptr__1: *mut *mut ::core::ffi::c_void =
                                        &raw mut skip_until as *mut *mut ::core::ffi::c_void;
                                    xfree(*ptr__1);
                                    *ptr__1 = NULL;
                                    *ptr__1;
                                    skip_until = xmemdupz(
                                        p as *const ::core::ffi::c_void,
                                        skiptowhite(p).offset_from(p) as size_t,
                                    )
                                        as *mut ::core::ffi::c_char;
                                    do_concat = false_0 != 0;
                                    is_heredoc = true_0 != 0;
                                }
                            }
                        }
                    }
                }
                ga_grow(
                    newlines,
                    1 as ::core::ffi::c_int + sourcing_lnum_off as ::core::ffi::c_int,
                );
                p = xstrdup(theline);
                let c2rust_fresh13 = (*newlines).ga_len;
                (*newlines).ga_len = (*newlines).ga_len + 1;
                let c2rust_lvalue_ptr = &raw mut *((*newlines).ga_data
                    as *mut *mut ::core::ffi::c_char)
                    .offset(c2rust_fresh13 as isize);
                *c2rust_lvalue_ptr = p;
                loop {
                    let c2rust_fresh14 = sourcing_lnum_off;
                    sourcing_lnum_off = sourcing_lnum_off - 1;
                    if c2rust_fresh14 <= 0 as linenr_T {
                        break;
                    }
                    let c2rust_fresh15 = (*newlines).ga_len;
                    (*newlines).ga_len = (*newlines).ga_len + 1;
                    let c2rust_lvalue_ptr_0 = &raw mut *((*newlines).ga_data
                        as *mut *mut ::core::ffi::c_char)
                        .offset(c2rust_fresh15 as isize);
                    *c2rust_lvalue_ptr_0 = ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                if !line_arg.is_null() && *line_arg as ::core::ffi::c_int == NUL {
                    line_arg = ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
            }
        }
        if did_emsg == 0 {
            ret = OK;
        }
    }
    xfree(skip_until as *mut ::core::ffi::c_void);
    xfree(heredoc_trimmed as *mut ::core::ffi::c_void);
    need_wait_return =
        need_wait_return as ::core::ffi::c_int | saved_wait_return as ::core::ffi::c_int != 0;
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn ex_function(mut eap: *mut exarg_T) {
    let mut sourcing_lnum_top: linenr_T = 0;
    let mut namelen: size_t = 0;
    let mut line_to_free: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut line_arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut newargs: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut default_args: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut newlines: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut varargs: ::core::ffi::c_int = false_0;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut fp: *mut ufunc_T = ::core::ptr::null_mut::<ufunc_T>();
    let mut free_fp: bool = false_0 != 0;
    let mut overwrite: bool = false_0 != 0;
    let mut fudi: funcdict_T = funcdict_T {
        fd_dict: ::core::ptr::null_mut::<dict_T>(),
        fd_newkey: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fd_di: ::core::ptr::null_mut::<dictitem_T>(),
    };
    static mut func_nr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut ht: *mut hashtab_T = ::core::ptr::null_mut::<hashtab_T>();
    let mut show_block: bool = false_0 != 0;
    if ends_excmd(*(*eap).arg as ::core::ffi::c_int) != 0 {
        if (*eap).skip == 0 {
            list_functions(::core::ptr::null_mut::<regmatch_T>());
        }
        (*eap).nextcmd = check_nextcmd((*eap).arg);
        return;
    }
    if *(*eap).arg as ::core::ffi::c_int == '/' as ::core::ffi::c_int {
        let mut p: *mut ::core::ffi::c_char = list_functions_matching_pat(eap);
        (*eap).nextcmd = check_nextcmd(p);
        return;
    }
    let mut p_0: *mut ::core::ffi::c_char = (*eap).arg;
    let mut name: *mut ::core::ffi::c_char = save_function_name(
        &raw mut p_0,
        (*eap).skip != 0,
        TFN_NO_AUTOLOAD as ::core::ffi::c_int,
        &raw mut fudi,
    );
    let mut paren: ::core::ffi::c_int =
        !vim_strchr(p_0, '(' as ::core::ffi::c_int).is_null() as ::core::ffi::c_int;
    if name.is_null() && (fudi.fd_dict.is_null() || paren == 0) && (*eap).skip == 0 {
        if !aborting() {
            if !fudi.fd_newkey.is_null() {
                semsg(
                    gettext(&raw const e_dictkey as *const ::core::ffi::c_char),
                    fudi.fd_newkey,
                );
            }
            xfree(fudi.fd_newkey as *mut ::core::ffi::c_void);
            return;
        }
        (*eap).skip = true_0;
    }
    let saved_did_emsg: ::core::ffi::c_int = did_emsg;
    did_emsg = false_0;
    '_ret_free: {
        if paren == 0 {
            fp = list_one_function(eap, name, p_0);
        } else {
            p_0 = skipwhite(p_0);
            if *p_0 as ::core::ffi::c_int != '(' as ::core::ffi::c_int {
                if (*eap).skip == 0 {
                    semsg(
                        gettext(b"E124: Missing '(': %s\0".as_ptr() as *const ::core::ffi::c_char),
                        (*eap).arg,
                    );
                    break '_ret_free;
                } else if !vim_strchr(p_0, '(' as ::core::ffi::c_int).is_null() {
                    p_0 = vim_strchr(p_0, '(' as ::core::ffi::c_int);
                }
            }
            p_0 = skipwhite(p_0.offset(1 as ::core::ffi::c_int as isize));
            ga_init(
                &raw mut newargs,
                ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
                3 as ::core::ffi::c_int,
            );
            ga_init(
                &raw mut newlines,
                ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
                3 as ::core::ffi::c_int,
            );
            if (*eap).skip == 0 {
                if !name.is_null() {
                    arg = name;
                } else {
                    arg = fudi.fd_newkey;
                }
                if !arg.is_null() && (fudi.fd_di.is_null() || !tv_is_func((*fudi.fd_di).di_tv)) {
                    let mut name_base: *mut ::core::ffi::c_char = arg;
                    if arg != fudi.fd_newkey {
                        if *arg as uint8_t as ::core::ffi::c_int == K_SPECIAL {
                            name_base = vim_strchr(arg, '_' as ::core::ffi::c_int);
                            if name_base.is_null() {
                                name_base = arg.offset(3 as ::core::ffi::c_int as isize);
                            } else {
                                name_base = name_base.offset(1);
                            }
                        }
                        let mut i: ::core::ffi::c_int = 0;
                        i = 0 as ::core::ffi::c_int;
                        while *name_base.offset(i as isize) as ::core::ffi::c_int != NUL
                            && (if i == 0 as ::core::ffi::c_int {
                                eval_isnamec1(*name_base.offset(i as isize) as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                            } else {
                                eval_isnamec(*name_base.offset(i as isize) as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                            }) != 0
                        {
                            i += 1;
                        }
                        if *name_base.offset(i as isize) as ::core::ffi::c_int != NUL {
                            emsg_funcname(&raw const e_invarg2 as *const ::core::ffi::c_char, arg);
                            break '_ret_free;
                        }
                    }
                }
                if !fudi.fd_dict.is_null()
                    && (*fudi.fd_dict).dv_scope as ::core::ffi::c_uint
                        == VAR_DEF_SCOPE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    emsg(gettext(
                        b"E862: Cannot use g: here\0".as_ptr() as *const ::core::ffi::c_char
                    ));
                    break '_ret_free;
                }
            }
            '_errret_keep: {
                if get_function_args(
                    &raw mut p_0,
                    ')' as ::core::ffi::c_char,
                    &raw mut newargs,
                    &raw mut varargs,
                    &raw mut default_args,
                    (*eap).skip != 0,
                ) != FAIL
                {
                    if KeyTyped as ::core::ffi::c_int != 0
                        && ui_has(kUICmdline) as ::core::ffi::c_int != 0
                    {
                        show_block = true_0 != 0;
                        ui_ext_cmdline_block_append(0 as size_t, (*eap).cmd);
                    }
                    '_erret: {
                        loop {
                            p_0 = skipwhite(p_0);
                            if strncmp(
                                p_0,
                                b"range\0".as_ptr() as *const ::core::ffi::c_char,
                                5 as size_t,
                            ) == 0 as ::core::ffi::c_int
                            {
                                flags |= FC_RANGE;
                                p_0 = p_0.offset(5 as ::core::ffi::c_int as isize);
                            } else if strncmp(
                                p_0,
                                b"dict\0".as_ptr() as *const ::core::ffi::c_char,
                                4 as size_t,
                            ) == 0 as ::core::ffi::c_int
                            {
                                flags |= FC_DICT;
                                p_0 = p_0.offset(4 as ::core::ffi::c_int as isize);
                            } else if strncmp(
                                p_0,
                                b"abort\0".as_ptr() as *const ::core::ffi::c_char,
                                5 as size_t,
                            ) == 0 as ::core::ffi::c_int
                            {
                                flags |= FC_ABORT;
                                p_0 = p_0.offset(5 as ::core::ffi::c_int as isize);
                            } else {
                                if strncmp(
                                    p_0,
                                    b"closure\0".as_ptr() as *const ::core::ffi::c_char,
                                    7 as size_t,
                                ) != 0 as ::core::ffi::c_int
                                {
                                    break;
                                }
                                flags |= FC_CLOSURE;
                                p_0 = p_0.offset(7 as ::core::ffi::c_int as isize);
                                if !current_funccal.is_null() {
                                    continue;
                                }
                                emsg_funcname(
                                    b"E932: Closure function should not be at top level: %s\0"
                                        .as_ptr()
                                        as *const ::core::ffi::c_char,
                                    if name.is_null() {
                                        b"\0".as_ptr() as *const ::core::ffi::c_char
                                    } else {
                                        name as *const ::core::ffi::c_char
                                    },
                                );
                                break '_erret;
                            }
                        }
                        if *p_0 as ::core::ffi::c_int == '\n' as ::core::ffi::c_int {
                            line_arg = p_0.offset(1 as ::core::ffi::c_int as isize);
                        } else if *p_0 as ::core::ffi::c_int != NUL
                            && *p_0 as ::core::ffi::c_int != '"' as ::core::ffi::c_int
                            && (*eap).skip == 0
                            && did_emsg == 0
                        {
                            semsg(
                                gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                                p_0,
                            );
                        }
                        if KeyTyped {
                            if (*eap).skip == 0 && (*eap).forceit == 0 {
                                if !fudi.fd_dict.is_null() && fudi.fd_newkey.is_null() {
                                    emsg(gettext(e_funcdict));
                                } else if !name.is_null() && !find_func(name).is_null() {
                                    emsg_funcname(e_funcexts, name);
                                }
                            }
                            if (*eap).skip == 0 && did_emsg != 0 {
                                break '_erret;
                            } else {
                                if !ui_has(kUICmdline) {
                                    msg_putchar('\n' as ::core::ffi::c_int);
                                }
                                cmdline_row = msg_row;
                            }
                        }
                        sourcing_lnum_top = (*(exestack.ga_data as *mut estack_T)
                            .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
                        .es_lnum;
                        if !(get_function_body(
                            eap,
                            &raw mut newlines,
                            line_arg,
                            &raw mut line_to_free,
                            show_block,
                        ) == FAIL
                            || (*eap).skip != 0)
                        {
                            namelen = 0 as size_t;
                            if fudi.fd_dict.is_null() {
                                let mut v: *mut dictitem_T =
                                    find_var(name, strlen(name), &raw mut ht, false_0);
                                if !v.is_null()
                                    && (*v).di_tv.v_type as ::core::ffi::c_uint
                                        == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
                                {
                                    emsg_funcname(
                                        b"E707: Function name conflicts with variable: %s\0"
                                            .as_ptr()
                                            as *const ::core::ffi::c_char,
                                        name,
                                    );
                                    break '_erret;
                                } else {
                                    fp = find_func(name);
                                    if !fp.is_null() {
                                        if (*eap).forceit == 0
                                            && ((*fp).uf_script_ctx.sc_sid != current_sctx.sc_sid
                                                || (*fp).uf_script_ctx.sc_seq
                                                    == current_sctx.sc_seq)
                                        {
                                            emsg_funcname(e_funcexts, name);
                                            break '_errret_keep;
                                        } else if (*fp).uf_calls > 0 as ::core::ffi::c_int {
                                            emsg_funcname(
                                                b"E127: Cannot redefine function %s: It is in use\0"
                                                    .as_ptr()
                                                    as *const ::core::ffi::c_char,
                                                name,
                                            );
                                            break '_errret_keep;
                                        } else if (*fp).uf_refcount > 1 as ::core::ffi::c_int {
                                            (*fp).uf_refcount -= 1;
                                            (*fp).uf_flags |= FC_REMOVED;
                                            fp = ::core::ptr::null_mut::<ufunc_T>();
                                            overwrite = true_0 != 0;
                                        } else {
                                            let mut exp_name: *mut ::core::ffi::c_char =
                                                (*fp).uf_name_exp;
                                            let mut ptr_: *mut *mut ::core::ffi::c_void =
                                                &raw mut name as *mut *mut ::core::ffi::c_void;
                                            xfree(*ptr_);
                                            *ptr_ = NULL;
                                            *ptr_;
                                            (*fp).uf_name_exp =
                                                ::core::ptr::null_mut::<::core::ffi::c_char>();
                                            func_clear_items(fp);
                                            (*fp).uf_name_exp = exp_name;
                                            (*fp).uf_profiling = false_0;
                                            (*fp).uf_prof_initialized = false_0;
                                        }
                                    }
                                }
                            } else {
                                let mut numbuf: [::core::ffi::c_char; 65] = [0; 65];
                                fp = ::core::ptr::null_mut::<ufunc_T>();
                                if fudi.fd_newkey.is_null() && (*eap).forceit == 0 {
                                    emsg(gettext(e_funcdict));
                                    break '_erret;
                                } else {
                                    if fudi.fd_di.is_null() {
                                        if value_check_lock(
                                            (*fudi.fd_dict).dv_lock,
                                            (*eap).arg,
                                            TV_CSTRING as size_t,
                                        ) {
                                            break '_erret;
                                        }
                                    } else if value_check_lock(
                                        (*fudi.fd_di).di_tv.v_lock,
                                        (*eap).arg,
                                        TV_CSTRING as size_t,
                                    ) {
                                        break '_erret;
                                    }
                                    xfree(name as *mut ::core::ffi::c_void);
                                    func_nr += 1;
                                    namelen = snprintf(
                                        &raw mut numbuf as *mut ::core::ffi::c_char,
                                        ::core::mem::size_of::<[::core::ffi::c_char; 65]>(),
                                        b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                                        func_nr,
                                    ) as size_t;
                                    name = xmemdupz(
                                        &raw mut numbuf as *mut ::core::ffi::c_char
                                            as *const ::core::ffi::c_void,
                                        namelen,
                                    )
                                        as *mut ::core::ffi::c_char;
                                }
                            }
                            if fp.is_null() {
                                if fudi.fd_dict.is_null()
                                    && !vim_strchr(name, AUTOLOAD_CHAR).is_null()
                                {
                                    let mut j: ::core::ffi::c_int = FAIL;
                                    if !(*(exestack.ga_data as *mut estack_T).offset(
                                        (exestack.ga_len - 1 as ::core::ffi::c_int) as isize,
                                    ))
                                    .es_name
                                    .is_null()
                                    {
                                        let mut scriptname: *mut ::core::ffi::c_char =
                                            autoload_name(name, strlen(name));
                                        p_0 = vim_strchr(scriptname, '/' as ::core::ffi::c_int);
                                        let mut plen: ::core::ffi::c_int =
                                            strlen(p_0) as ::core::ffi::c_int;
                                        let mut slen: ::core::ffi::c_int = strlen(
                                            (*(exestack.ga_data as *mut estack_T).offset(
                                                (exestack.ga_len - 1 as ::core::ffi::c_int)
                                                    as isize,
                                            ))
                                            .es_name,
                                        )
                                            as ::core::ffi::c_int;
                                        if slen > plen
                                            && path_fnamecmp(
                                                p_0,
                                                (*(exestack.ga_data as *mut estack_T).offset(
                                                    (exestack.ga_len - 1 as ::core::ffi::c_int)
                                                        as isize,
                                                ))
                                                .es_name
                                                .offset(slen as isize)
                                                .offset(-(plen as isize)),
                                            ) == 0 as ::core::ffi::c_int
                                        {
                                            j = OK;
                                        }
                                        xfree(scriptname as *mut ::core::ffi::c_void);
                                    }
                                    if j == FAIL {
                                        semsg(
                                            gettext(
                                                b"E746: Function name does not match script file name: %s\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            ),
                                            name,
                                        );
                                        break '_erret;
                                    }
                                }
                                if namelen == 0 as size_t {
                                    namelen = strlen(name);
                                }
                                fp = alloc_ufunc(name, namelen);
                                if !fudi.fd_dict.is_null() {
                                    if fudi.fd_di.is_null() {
                                        fudi.fd_di = tv_dict_item_alloc(fudi.fd_newkey);
                                        if tv_dict_add(fudi.fd_dict, fudi.fd_di) == FAIL {
                                            xfree(fudi.fd_di as *mut ::core::ffi::c_void);
                                            let mut ptr__0: *mut *mut ::core::ffi::c_void =
                                                &raw mut fp as *mut *mut ::core::ffi::c_void;
                                            xfree(*ptr__0);
                                            *ptr__0 = NULL;
                                            *ptr__0;
                                            break '_erret;
                                        }
                                    } else {
                                        tv_clear(&raw mut (*fudi.fd_di).di_tv);
                                    }
                                    (*fudi.fd_di).di_tv.v_type = VAR_FUNC;
                                    (*fudi.fd_di).di_tv.vval.v_string =
                                        xmemdupz(name as *const ::core::ffi::c_void, namelen)
                                            as *mut ::core::ffi::c_char;
                                    flags |= FC_DICT;
                                }
                                if overwrite {
                                    let mut hi: *mut hashitem_T =
                                        hash_find(&raw mut func_hashtab, name);
                                    (*hi).hi_key =
                                        &raw mut (*fp).uf_name as *mut ::core::ffi::c_char;
                                } else if hash_add(
                                    &raw mut func_hashtab,
                                    &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
                                ) == FAIL
                                {
                                    free_fp = true_0 != 0;
                                    break '_erret;
                                }
                                (*fp).uf_refcount = 1 as ::core::ffi::c_int;
                            }
                            (*fp).uf_args = newargs;
                            (*fp).uf_def_args = default_args;
                            (*fp).uf_lines = newlines;
                            if flags & FC_CLOSURE != 0 as ::core::ffi::c_int {
                                register_closure(fp);
                            } else {
                                (*fp).uf_scoped = ::core::ptr::null_mut::<funccall_T>();
                            }
                            if prof_def_func() {
                                func_do_profile(fp);
                            }
                            (*fp).uf_varargs = varargs;
                            if sandbox != 0 {
                                flags |= FC_SANDBOX;
                            }
                            (*fp).uf_flags = flags;
                            (*fp).uf_calls = 0 as ::core::ffi::c_int;
                            (*fp).uf_script_ctx = current_sctx;
                            (*fp).uf_script_ctx.sc_lnum += sourcing_lnum_top;
                            nlua_set_sctx(&raw mut (*fp).uf_script_ctx);
                            break '_ret_free;
                        }
                    }
                    if !fp.is_null() {
                        ga_init(
                            &raw mut (*fp).uf_args,
                            ::core::mem::size_of::<*mut ::core::ffi::c_char>()
                                as ::core::ffi::c_int,
                            1 as ::core::ffi::c_int,
                        );
                        ga_init(
                            &raw mut (*fp).uf_def_args,
                            ::core::mem::size_of::<*mut ::core::ffi::c_char>()
                                as ::core::ffi::c_int,
                            1 as ::core::ffi::c_int,
                        );
                    }
                }
                if !fp.is_null() {
                    let mut ptr__1: *mut *mut ::core::ffi::c_void =
                        &raw mut (*fp).uf_name_exp as *mut *mut ::core::ffi::c_void;
                    xfree(*ptr__1);
                    *ptr__1 = NULL;
                    *ptr__1;
                }
                if free_fp {
                    let mut ptr__2: *mut *mut ::core::ffi::c_void =
                        &raw mut fp as *mut *mut ::core::ffi::c_void;
                    xfree(*ptr__2);
                    *ptr__2 = NULL;
                    *ptr__2;
                }
            }
            ga_clear_strings(&raw mut newargs);
            ga_clear_strings(&raw mut default_args);
            ga_clear_strings(&raw mut newlines);
        }
    }
    xfree(line_to_free as *mut ::core::ffi::c_void);
    xfree(fudi.fd_newkey as *mut ::core::ffi::c_void);
    xfree(name as *mut ::core::ffi::c_void);
    did_emsg |= saved_did_emsg;
    if show_block {
        ui_ext_cmdline_block_leave();
    }
}
#[no_mangle]
pub unsafe extern "C" fn eval_fname_script(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '<' as ::core::ffi::c_int
        && (mb_strnicmp(
            p.offset(1 as ::core::ffi::c_int as isize),
            b"SID>\0".as_ptr() as *const ::core::ffi::c_char,
            4 as size_t,
        ) == 0 as ::core::ffi::c_int
            || mb_strnicmp(
                p.offset(1 as ::core::ffi::c_int as isize),
                b"SNR>\0".as_ptr() as *const ::core::ffi::c_char,
                4 as size_t,
            ) == 0 as ::core::ffi::c_int)
    {
        return 5 as ::core::ffi::c_int;
    }
    if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 's' as ::core::ffi::c_int
        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == ':' as ::core::ffi::c_int
    {
        return 2 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn translated_function_exists(mut name: *const ::core::ffi::c_char) -> bool {
    if builtin_function(name, -1 as ::core::ffi::c_int) {
        return !find_internal_func(name).is_null();
    }
    return !find_func(name).is_null();
}
#[no_mangle]
pub unsafe extern "C" fn function_exists(
    name: *const ::core::ffi::c_char,
    mut no_deref: bool,
) -> bool {
    let mut nm: *const ::core::ffi::c_char = name;
    let mut n: bool = false_0 != 0;
    let mut flag: ::core::ffi::c_int = TFN_INT as ::core::ffi::c_int
        | TFN_QUIET as ::core::ffi::c_int
        | TFN_NO_AUTOLOAD as ::core::ffi::c_int;
    if no_deref {
        flag |= TFN_NO_DEREF as ::core::ffi::c_int;
    }
    let p: *mut ::core::ffi::c_char = trans_function_name(
        &raw mut nm as *mut *mut ::core::ffi::c_char,
        false_0 != 0,
        flag,
        ::core::ptr::null_mut::<funcdict_T>(),
        ::core::ptr::null_mut::<*mut partial_T>(),
    );
    nm = skipwhite(nm);
    if !p.is_null()
        && (*nm as ::core::ffi::c_int == NUL
            || *nm as ::core::ffi::c_int == '(' as ::core::ffi::c_int)
    {
        n = translated_function_exists(p);
    }
    xfree(p as *mut ::core::ffi::c_void);
    return n;
}
#[no_mangle]
pub unsafe extern "C" fn get_user_func_name(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    static mut done: size_t = 0;
    static mut changed: ::core::ffi::c_int = 0;
    static mut hi: *mut hashitem_T = ::core::ptr::null_mut::<hashitem_T>();
    if idx == 0 as ::core::ffi::c_int {
        done = 0 as size_t;
        hi = func_hashtab.ht_array;
        changed = func_hashtab.ht_changed;
    }
    '_c2rust_label: {
        if !hi.is_null() {
        } else {
            __assert_fail(
                b"hi\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/eval/userfunc.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                3083 as ::core::ffi::c_uint,
                b"char *get_user_func_name(expand_T *, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if changed == func_hashtab.ht_changed && done < func_hashtab.ht_used {
        let c2rust_fresh16 = done;
        done = done.wrapping_add(1);
        if c2rust_fresh16 > 0 as size_t {
            hi = hi.offset(1);
        }
        while (*hi).hi_key.is_null() || (*hi).hi_key == &raw mut hash_removed {
            hi = hi.offset(1);
        }
        let mut fp: *mut ufunc_T =
            (*hi).hi_key.offset(-(240 as ::core::ffi::c_ulong as isize)) as *mut ufunc_T;
        if (*fp).uf_flags & FC_DICT != 0
            || strncmp(
                &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
                b"<lambda>\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        if (*fp).uf_namelen.wrapping_add(4 as size_t) >= IOSIZE as size_t {
            return &raw mut (*fp).uf_name as *mut ::core::ffi::c_char;
        }
        let mut len: ::core::ffi::c_int = cat_func_name(
            &raw mut IObuff as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            fp,
        );
        if (*xp).xp_context != EXPAND_USER_FUNC as ::core::ffi::c_int {
            xstrlcpy(
                (&raw mut IObuff as *mut ::core::ffi::c_char).offset(len as isize),
                b"(\0".as_ptr() as *const ::core::ffi::c_char,
                (IOSIZE as size_t).wrapping_sub(len as size_t),
            );
            if (*fp).uf_varargs == 0 && (*fp).uf_args.ga_len <= 0 as ::core::ffi::c_int {
                len += 1;
                xstrlcpy(
                    (&raw mut IObuff as *mut ::core::ffi::c_char).offset(len as isize),
                    b")\0".as_ptr() as *const ::core::ffi::c_char,
                    (IOSIZE as size_t).wrapping_sub(len as size_t),
                );
            }
        }
        return &raw mut IObuff as *mut ::core::ffi::c_char;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn ex_delfunction(mut eap: *mut exarg_T) {
    let mut fp: *mut ufunc_T = ::core::ptr::null_mut::<ufunc_T>();
    let mut fudi: funcdict_T = funcdict_T {
        fd_dict: ::core::ptr::null_mut::<dict_T>(),
        fd_newkey: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fd_di: ::core::ptr::null_mut::<dictitem_T>(),
    };
    let mut p: *mut ::core::ffi::c_char = (*eap).arg;
    let mut name: *mut ::core::ffi::c_char = trans_function_name(
        &raw mut p,
        (*eap).skip != 0,
        0 as ::core::ffi::c_int,
        &raw mut fudi,
        ::core::ptr::null_mut::<*mut partial_T>(),
    );
    xfree(fudi.fd_newkey as *mut ::core::ffi::c_void);
    if name.is_null() {
        if !fudi.fd_dict.is_null() && (*eap).skip == 0 {
            emsg(gettext(e_funcref));
        }
        return;
    }
    if ends_excmd(*skipwhite(p) as ::core::ffi::c_int) == 0 {
        xfree(name as *mut ::core::ffi::c_void);
        semsg(
            gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
            p,
        );
        return;
    }
    (*eap).nextcmd = check_nextcmd(p);
    if !(*eap).nextcmd.is_null() {
        *p = NUL as ::core::ffi::c_char;
    }
    if *(*__ctype_b_loc()).offset(*name as uint8_t as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
        != 0
        && fudi.fd_dict.is_null()
    {
        if (*eap).skip == 0 {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                (*eap).arg,
            );
        }
        xfree(name as *mut ::core::ffi::c_void);
        return;
    }
    if (*eap).skip == 0 {
        fp = find_func(name);
    }
    xfree(name as *mut ::core::ffi::c_void);
    if (*eap).skip == 0 {
        if fp.is_null() {
            if (*eap).forceit == 0 {
                semsg(gettext(e_nofunc), (*eap).arg);
            }
            return;
        }
        if (*fp).uf_calls > 0 as ::core::ffi::c_int {
            semsg(
                gettext(b"E131: Cannot delete function %s: It is in use\0".as_ptr()
                    as *const ::core::ffi::c_char),
                (*eap).arg,
            );
            return;
        }
        if (*fp).uf_refcount > 2 as ::core::ffi::c_int {
            semsg(
                gettext(
                    b"Cannot delete function %s: It is being used internally\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                (*eap).arg,
            );
            return;
        }
        if !fudi.fd_dict.is_null() {
            tv_dict_item_remove(fudi.fd_dict, fudi.fd_di);
        } else if (*fp).uf_refcount
            > (if func_name_refcount(&raw mut (*fp).uf_name as *mut ::core::ffi::c_char)
                as ::core::ffi::c_int
                != 0
            {
                0 as ::core::ffi::c_int
            } else {
                1 as ::core::ffi::c_int
            })
        {
            if func_remove(fp) {
                (*fp).uf_refcount -= 1;
            }
            (*fp).uf_flags |= FC_DELETED;
        } else {
            func_clear_free(fp, false_0 != 0);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn func_unref(mut name: *mut ::core::ffi::c_char) {
    if name.is_null() || !func_name_refcount(name) {
        return;
    }
    let mut fp: *mut ufunc_T = find_func(name);
    if fp.is_null()
        && *(*__ctype_b_loc()).offset(*name as uint8_t as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
    {
        internal_error(b"func_unref()\0".as_ptr() as *const ::core::ffi::c_char);
        abort();
    }
    func_ptr_unref(fp);
}
#[no_mangle]
pub unsafe extern "C" fn func_ptr_unref(mut fp: *mut ufunc_T) {
    if !fp.is_null() && {
        (*fp).uf_refcount -= 1;
        (*fp).uf_refcount <= 0 as ::core::ffi::c_int
    } {
        if (*fp).uf_calls == 0 as ::core::ffi::c_int {
            func_clear_free(fp, false_0 != 0);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn func_ref(mut name: *mut ::core::ffi::c_char) {
    if name.is_null() || !func_name_refcount(name) {
        return;
    }
    let mut fp: *mut ufunc_T = find_func(name);
    if !fp.is_null() {
        (*fp).uf_refcount += 1;
    } else if *(*__ctype_b_loc()).offset(*name as uint8_t as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
        != 0
    {
        internal_error(b"func_ref()\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
#[no_mangle]
pub unsafe extern "C" fn func_ptr_ref(mut fp: *mut ufunc_T) {
    if !fp.is_null() {
        (*fp).uf_refcount += 1;
    }
}
#[inline(always)]
unsafe extern "C" fn fc_referenced(fc: *const funccall_T) -> bool {
    return (*fc).fc_l_varlist.lv_refcount != DO_NOT_FREE_CNT as ::core::ffi::c_int
        || (*fc).fc_l_vars.dv_refcount != DO_NOT_FREE_CNT as ::core::ffi::c_int
        || (*fc).fc_l_avars.dv_refcount != DO_NOT_FREE_CNT as ::core::ffi::c_int
        || (*fc).fc_refcount > 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn can_free_funccal(
    mut fc: *mut funccall_T,
    mut copyID: ::core::ffi::c_int,
) -> bool {
    return (*fc).fc_l_varlist.lv_copyID != copyID
        && (*fc).fc_l_vars.dv_copyID != copyID
        && (*fc).fc_l_avars.dv_copyID != copyID
        && (*fc).fc_copyID != copyID;
}
#[no_mangle]
pub unsafe extern "C" fn ex_return(mut eap: *mut exarg_T) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut returning: bool = false_0 != 0;
    if current_funccal.is_null() {
        emsg(gettext(
            b"E133: :return not inside a function\0".as_ptr() as *const ::core::ffi::c_char
        ));
        return;
    }
    let mut evalarg: evalarg_T = evalarg_T {
        eval_flags: if (*eap).skip != 0 {
            0 as ::core::ffi::c_int
        } else {
            EVAL_EVALUATE as ::core::ffi::c_int
        },
        eval_getline: None,
        eval_cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        eval_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    if (*eap).skip != 0 {
        emsg_skip += 1;
    }
    (*eap).nextcmd = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if *arg as ::core::ffi::c_int != NUL
        && *arg as ::core::ffi::c_int != '|' as ::core::ffi::c_int
        && *arg as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
        && eval0(arg, &raw mut rettv, eap, &raw mut evalarg) != FAIL
    {
        if (*eap).skip == 0 {
            returning = do_return(
                eap,
                false_0 != 0,
                true_0 != 0,
                &raw mut rettv as *mut ::core::ffi::c_void,
            );
        } else {
            tv_clear(&raw mut rettv);
        }
    } else if (*eap).skip == 0 {
        update_force_abort();
        if !aborting() {
            returning = do_return(eap, false_0 != 0, true_0 != 0, NULL);
        }
    }
    if returning {
        (*eap).nextcmd = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else if (*eap).nextcmd.is_null() {
        (*eap).nextcmd = check_nextcmd(arg);
    }
    if (*eap).skip != 0 {
        emsg_skip -= 1;
    }
    clear_evalarg(&raw mut evalarg, eap);
}
unsafe extern "C" fn ex_call_inner(
    mut eap: *mut exarg_T,
    mut name: *mut ::core::ffi::c_char,
    mut arg: *mut *mut ::core::ffi::c_char,
    mut startarg: *mut ::core::ffi::c_char,
    funcexe_init: *const funcexe_T,
    evalarg: *mut evalarg_T,
) -> ::core::ffi::c_int {
    let mut doesrange: bool = false;
    let mut failed: bool = false_0 != 0;
    let mut lnum: linenr_T = (*eap).line1;
    while lnum <= (*eap).line2 {
        if (*eap).addr_count > 0 as ::core::ffi::c_int {
            if lnum > (*curbuf).b_ml.ml_line_count {
                emsg(gettext(&raw const e_invrange as *const ::core::ffi::c_char));
                break;
            } else {
                (*curwin).w_cursor.lnum = lnum;
                (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                (*curwin).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
            }
        }
        *arg = startarg;
        let mut funcexe: funcexe_T = *funcexe_init;
        funcexe.fe_doesrange = &raw mut doesrange;
        let mut rettv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        rettv.v_type = VAR_UNKNOWN;
        if get_func_tv(
            name,
            -1 as ::core::ffi::c_int,
            &raw mut rettv,
            arg,
            evalarg,
            &raw mut funcexe,
        ) == FAIL
        {
            failed = true_0 != 0;
            break;
        } else if handle_subscript(
            arg as *mut *const ::core::ffi::c_char,
            &raw mut rettv,
            &raw mut EVALARG_EVALUATE,
            true_0 != 0,
        ) == FAIL
        {
            failed = true_0 != 0;
            break;
        } else {
            tv_clear(&raw mut rettv);
            if doesrange {
                break;
            }
            if aborting() {
                break;
            }
            lnum += 1;
        }
    }
    return failed as ::core::ffi::c_int;
}
unsafe extern "C" fn ex_defer_inner(
    mut name: *mut ::core::ffi::c_char,
    mut arg: *mut *mut ::core::ffi::c_char,
    partial: *const partial_T,
    evalarg: *mut evalarg_T,
) -> ::core::ffi::c_int {
    let mut argvars: [typval_T; 21] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 21];
    let mut partial_argc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut argcount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if current_funccal.is_null() {
        semsg(
            gettext(&raw const e_str_not_inside_function as *const ::core::ffi::c_char),
            b"defer\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return FAIL;
    }
    if !partial.is_null() {
        if !(*partial).pt_dict.is_null() {
            emsg(gettext(
                &raw const e_cannot_use_partial_with_dictionary_for_defer
                    as *const ::core::ffi::c_char,
            ));
            return FAIL;
        }
        if (*partial).pt_argc > 0 as ::core::ffi::c_int {
            partial_argc = (*partial).pt_argc;
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < partial_argc {
                tv_copy(
                    (*partial).pt_argv.offset(i as isize),
                    (&raw mut argvars as *mut typval_T).offset(i as isize),
                );
                i += 1;
            }
        }
    }
    let mut r: ::core::ffi::c_int = get_func_arguments(
        arg,
        evalarg,
        false_0,
        (&raw mut argvars as *mut typval_T).offset(partial_argc as isize),
        &raw mut argcount,
    );
    argcount += partial_argc;
    if r == OK {
        if builtin_function(name, -1 as ::core::ffi::c_int) {
            let fdef: *const EvalFuncDef = find_internal_func(name);
            if fdef.is_null() {
                emsg_funcname(
                    &raw const e_unknown_function_str as *const ::core::ffi::c_char,
                    name,
                );
                r = FAIL;
            } else if check_internal_func(fdef, argcount) == -1 as ::core::ffi::c_int {
                r = FAIL;
            }
        } else {
            let mut ufunc: *mut ufunc_T = find_func(name);
            if !ufunc.is_null() {
                let mut error: ::core::ffi::c_int = check_user_func_argcount(ufunc, argcount);
                if error != FCERR_UNKNOWN as ::core::ffi::c_int {
                    user_func_error(error, name, false_0 != 0);
                    r = FAIL;
                }
            }
        }
    }
    if r == FAIL {
        loop {
            argcount -= 1;
            if argcount < 0 as ::core::ffi::c_int {
                break;
            }
            tv_clear((&raw mut argvars as *mut typval_T).offset(argcount as isize));
        }
        return FAIL;
    }
    add_defer(name, argcount, &raw mut argvars as *mut typval_T);
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn can_add_defer() -> bool {
    if get_current_funccal().is_null() {
        semsg(
            gettext(&raw const e_str_not_inside_function as *const ::core::ffi::c_char),
            b"defer\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return false_0 != 0;
    }
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn add_defer(
    mut name: *mut ::core::ffi::c_char,
    mut argcount_arg: ::core::ffi::c_int,
    mut argvars: *mut typval_T,
) {
    let mut saved_name: *mut ::core::ffi::c_char = xstrdup(name);
    let mut argcount: ::core::ffi::c_int = argcount_arg;
    if (*current_funccal).fc_defer.ga_itemsize == 0 as ::core::ffi::c_int {
        ga_init(
            &raw mut (*current_funccal).fc_defer,
            ::core::mem::size_of::<defer_T>() as ::core::ffi::c_int,
            10 as ::core::ffi::c_int,
        );
    }
    let mut dr: *mut defer_T = ga_append_via_ptr(
        &raw mut (*current_funccal).fc_defer,
        ::core::mem::size_of::<defer_T>(),
    ) as *mut defer_T;
    (*dr).dr_name = saved_name;
    (*dr).dr_argcount = argcount;
    while argcount > 0 as ::core::ffi::c_int {
        argcount -= 1;
        (*dr).dr_argvars[argcount as usize] = *argvars.offset(argcount as isize);
    }
}
unsafe extern "C" fn handle_defer_one(mut funccal: *mut funccall_T) {
    let mut idx: ::core::ffi::c_int = (*funccal).fc_defer.ga_len - 1 as ::core::ffi::c_int;
    while idx >= 0 as ::core::ffi::c_int {
        let mut dr: *mut defer_T =
            ((*funccal).fc_defer.ga_data as *mut defer_T).offset(idx as isize);
        if !(*dr).dr_name.is_null() {
            let mut funcexe: funcexe_T = funcexe_T {
                fe_argv_func: None,
                fe_firstline: 0,
                fe_lastline: 0,
                fe_doesrange: ::core::ptr::null_mut::<bool>(),
                fe_evaluate: true_0 != 0,
                fe_partial: ::core::ptr::null_mut::<partial_T>(),
                fe_selfdict: ::core::ptr::null_mut::<dict_T>(),
                fe_basetv: ::core::ptr::null_mut::<typval_T>(),
                fe_found_var: false,
            };
            let mut rettv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            rettv.v_type = VAR_UNKNOWN;
            let mut name: *mut ::core::ffi::c_char = (*dr).dr_name;
            (*dr).dr_name = ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut estate: exception_state_T = exception_state_T {
                estate_current_exception: ::core::ptr::null_mut::<except_T>(),
                estate_did_throw: false,
                estate_need_rethrow: false,
                estate_trylevel: 0,
                estate_did_emsg: 0,
            };
            exception_state_save(&raw mut estate);
            exception_state_clear();
            call_func(
                name,
                -1 as ::core::ffi::c_int,
                &raw mut rettv,
                (*dr).dr_argcount,
                &raw mut (*dr).dr_argvars as *mut typval_T,
                &raw mut funcexe,
            );
            exception_state_restore(&raw mut estate);
            tv_clear(&raw mut rettv);
            xfree(name as *mut ::core::ffi::c_void);
            let mut i: ::core::ffi::c_int = (*dr).dr_argcount - 1 as ::core::ffi::c_int;
            while i >= 0 as ::core::ffi::c_int {
                tv_clear((&raw mut (*dr).dr_argvars as *mut typval_T).offset(i as isize));
                i -= 1;
            }
        }
        idx -= 1;
    }
    ga_clear(&raw mut (*funccal).fc_defer);
}
#[no_mangle]
pub unsafe extern "C" fn invoke_all_defer() {
    let mut fc: *mut funccall_T = current_funccal;
    while !fc.is_null() {
        handle_defer_one(fc);
        fc = (*fc).fc_caller;
    }
    let mut fce: *mut funccal_entry_T = funccal_stack;
    while !fce.is_null() {
        let mut fc_0: *mut funccall_T = (*fce).top_funccal as *mut funccall_T;
        while !fc_0.is_null() {
            handle_defer_one(fc_0);
            fc_0 = (*fc_0).fc_caller;
        }
        fce = (*fce).next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ex_call(mut eap: *mut exarg_T) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut failed: bool = false_0 != 0;
    let mut fudi: funcdict_T = funcdict_T {
        fd_dict: ::core::ptr::null_mut::<dict_T>(),
        fd_newkey: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fd_di: ::core::ptr::null_mut::<dictitem_T>(),
    };
    let mut partial: *mut partial_T = ::core::ptr::null_mut::<partial_T>();
    let mut evalarg: evalarg_T = evalarg_T {
        eval_flags: 0,
        eval_getline: None,
        eval_cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        eval_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    fill_evalarg_from_eap(&raw mut evalarg, eap, (*eap).skip != 0);
    if (*eap).skip != 0 {
        let mut rettv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        emsg_skip += 1;
        if eval0((*eap).arg, &raw mut rettv, eap, &raw mut evalarg) != FAIL {
            tv_clear(&raw mut rettv);
        }
        emsg_skip -= 1;
        clear_evalarg(&raw mut evalarg, eap);
        return;
    }
    let mut tofree: *mut ::core::ffi::c_char = trans_function_name(
        &raw mut arg,
        false_0 != 0,
        TFN_INT as ::core::ffi::c_int,
        &raw mut fudi,
        &raw mut partial,
    );
    if !fudi.fd_newkey.is_null() {
        semsg(
            gettext(&raw const e_dictkey as *const ::core::ffi::c_char),
            fudi.fd_newkey,
        );
        xfree(fudi.fd_newkey as *mut ::core::ffi::c_void);
    }
    if tofree.is_null() {
        return;
    }
    if !fudi.fd_dict.is_null() {
        (*fudi.fd_dict).dv_refcount += 1;
    }
    let mut len: ::core::ffi::c_int = strlen(tofree) as ::core::ffi::c_int;
    let mut found_var: bool = false_0 != 0;
    let mut name: *mut ::core::ffi::c_char = deref_func_name(
        tofree,
        &raw mut len,
        if !partial.is_null() {
            ::core::ptr::null_mut::<*mut partial_T>()
        } else {
            &raw mut partial
        },
        false_0 != 0,
        &raw mut found_var,
    );
    let mut startarg: *mut ::core::ffi::c_char = skipwhite(arg);
    if *startarg as ::core::ffi::c_int != '(' as ::core::ffi::c_int {
        semsg(
            gettext(&raw const e_missingparen as *const ::core::ffi::c_char),
            (*eap).arg,
        );
    } else {
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_defer as ::core::ffi::c_int {
            arg = startarg;
            failed = ex_defer_inner(name, &raw mut arg, partial, &raw mut evalarg) == FAIL;
        } else {
            let mut funcexe: funcexe_T = FUNCEXE_INIT;
            funcexe.fe_partial = partial;
            funcexe.fe_selfdict = fudi.fd_dict;
            funcexe.fe_firstline = (*eap).line1;
            funcexe.fe_lastline = (*eap).line2;
            funcexe.fe_found_var = found_var;
            funcexe.fe_evaluate = true_0 != 0;
            failed = ex_call_inner(
                eap,
                name,
                &raw mut arg,
                startarg,
                &raw mut funcexe,
                &raw mut evalarg,
            ) != 0;
        }
        if (!aborting() || did_throw as ::core::ffi::c_int != 0)
            && (!failed || (*(*eap).cstack).cs_trylevel > 0 as ::core::ffi::c_int)
        {
            if ends_excmd(*arg as ::core::ffi::c_int) == 0 {
                if !failed && !aborting() {
                    emsg_severe = true_0 != 0;
                    semsg(
                        gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                        arg,
                    );
                }
            } else {
                (*eap).nextcmd = check_nextcmd(arg);
            }
        }
        clear_evalarg(&raw mut evalarg, eap);
    }
    tv_dict_unref(fudi.fd_dict);
    xfree(tofree as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn do_return(
    mut eap: *mut exarg_T,
    mut reanimate: bool,
    mut is_cmd: bool,
    mut rettv: *mut ::core::ffi::c_void,
) -> bool {
    let cstack: *mut cstack_T = (*eap).cstack;
    if reanimate {
        (*current_funccal).fc_returned = false_0;
    }
    let mut idx: ::core::ffi::c_int =
        cleanup_conditionals((*eap).cstack, 0 as ::core::ffi::c_int, true_0);
    if idx >= 0 as ::core::ffi::c_int {
        (*cstack).cs_pending[idx as usize] =
            CSTP_RETURN as ::core::ffi::c_int as ::core::ffi::c_char;
        if !is_cmd && !reanimate {
            (*cstack).cs_pend.csp_rv[idx as usize] = rettv;
        } else {
            if reanimate {
                '_c2rust_label: {
                    if !(*current_funccal).fc_rettv.is_null() {
                    } else {
                        __assert_fail(
                            b"current_funccal->fc_rettv\0".as_ptr() as *const ::core::ffi::c_char,
                            b"/home/overlord/projects/neovim/neovim/src/nvim/eval/userfunc.c\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            3664 as ::core::ffi::c_uint,
                            b"_Bool do_return(exarg_T *, _Bool, _Bool, void *)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                rettv = (*current_funccal).fc_rettv as *mut ::core::ffi::c_void;
            }
            if !rettv.is_null() {
                (*cstack).cs_pend.csp_rv[idx as usize] =
                    xcalloc(1 as size_t, ::core::mem::size_of::<typval_T>());
                *((*cstack).cs_pend.csp_rv[idx as usize] as *mut typval_T) =
                    *(rettv as *mut typval_T);
            } else {
                (*cstack).cs_pend.csp_rv[idx as usize] = NULL;
            }
            if reanimate {
                (*(*current_funccal).fc_rettv).v_type = VAR_NUMBER;
                (*(*current_funccal).fc_rettv).vval.v_number = 0 as varnumber_T;
            }
        }
        report_make_pending(CSTP_RETURN as ::core::ffi::c_int, rettv);
    } else {
        (*current_funccal).fc_returned = true_0;
        if !reanimate && !rettv.is_null() {
            tv_clear((*current_funccal).fc_rettv);
            *(*current_funccal).fc_rettv = *(rettv as *mut typval_T);
            if !is_cmd {
                xfree(rettv);
            }
        }
    }
    return idx < 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn get_return_cmd(
    mut rettv: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_char {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut slen: size_t = 0 as size_t;
    if !rettv.is_null() {
        s = encode_tv2echo(rettv as *mut typval_T, ::core::ptr::null_mut::<size_t>());
        tofree = s;
    }
    if s.is_null() {
        s = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    } else {
        slen = strlen(s);
    }
    xstrlcpy(
        &raw mut IObuff as *mut ::core::ffi::c_char,
        b":return \0".as_ptr() as *const ::core::ffi::c_char,
        IOSIZE as size_t,
    );
    xstrlcpy(
        (&raw mut IObuff as *mut ::core::ffi::c_char).offset(8 as ::core::ffi::c_int as isize),
        s,
        (IOSIZE - 8 as ::core::ffi::c_int) as size_t,
    );
    let mut IObufflen: size_t = (8 as size_t).wrapping_add(slen);
    if IObufflen >= IOSIZE as size_t {
        strcpy(
            (&raw mut IObuff as *mut ::core::ffi::c_char)
                .offset((1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
                .offset(-(4 as ::core::ffi::c_int as isize)),
            b"...\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        IObufflen = (IOSIZE - 1 as ::core::ffi::c_int) as size_t;
    }
    xfree(tofree as *mut ::core::ffi::c_void);
    return xstrnsave(&raw mut IObuff as *mut ::core::ffi::c_char, IObufflen);
}
#[no_mangle]
pub unsafe extern "C" fn get_func_line(
    mut c: ::core::ffi::c_int,
    mut cookie: *mut ::core::ffi::c_void,
    mut indent: ::core::ffi::c_int,
    mut do_concat: bool,
) -> *mut ::core::ffi::c_char {
    let mut fcp: *mut funccall_T = cookie as *mut funccall_T;
    let mut fp: *mut ufunc_T = (*fcp).fc_func;
    let mut retval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*fcp).fc_dbg_tick != debug_tick {
        (*fcp).fc_breakpoint = dbg_find_breakpoint(
            false_0 != 0,
            &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
            (*(exestack.ga_data as *mut estack_T)
                .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum,
        );
        (*fcp).fc_dbg_tick = debug_tick;
    }
    if do_profiling == PROF_YES {
        func_line_end(cookie);
    }
    let mut gap: *mut garray_T = &raw mut (*fp).uf_lines;
    if (*fp).uf_flags & FC_ABORT != 0 && did_emsg != 0 && !aborted_in_try()
        || (*fcp).fc_returned != 0
    {
        retval = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else {
        while (*fcp).fc_linenr < (*gap).ga_len
            && (*((*gap).ga_data as *mut *mut ::core::ffi::c_char)
                .offset((*fcp).fc_linenr as isize))
            .is_null()
        {
            (*fcp).fc_linenr += 1;
        }
        if (*fcp).fc_linenr >= (*gap).ga_len {
            retval = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else {
            let c2rust_fresh10 = (*fcp).fc_linenr;
            (*fcp).fc_linenr = (*fcp).fc_linenr + 1;
            retval = xstrdup(
                *((*gap).ga_data as *mut *mut ::core::ffi::c_char).offset(c2rust_fresh10 as isize),
            );
            (*(exestack.ga_data as *mut estack_T)
                .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum = (*fcp).fc_linenr as linenr_T;
            if do_profiling == PROF_YES {
                func_line_start(cookie);
            }
        }
    }
    if (*fcp).fc_breakpoint != 0 as linenr_T
        && (*fcp).fc_breakpoint
            <= (*(exestack.ga_data as *mut estack_T)
                .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum
    {
        dbg_breakpoint(
            &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
            (*(exestack.ga_data as *mut estack_T)
                .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum,
        );
        (*fcp).fc_breakpoint = dbg_find_breakpoint(
            false_0 != 0,
            &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
            (*(exestack.ga_data as *mut estack_T)
                .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum,
        );
        (*fcp).fc_dbg_tick = debug_tick;
    }
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn func_has_ended(
    mut cookie: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut fcp: *mut funccall_T = cookie as *mut funccall_T;
    return ((*(*fcp).fc_func).uf_flags & FC_ABORT != 0 && did_emsg != 0 && !aborted_in_try()
        || (*fcp).fc_returned != 0) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn func_has_abort(
    mut cookie: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    return (*(*(cookie as *mut funccall_T)).fc_func).uf_flags & FC_ABORT;
}
#[no_mangle]
pub unsafe extern "C" fn make_partial(selfdict: *mut dict_T, rettv: *mut typval_T) {
    let mut fp: *mut ufunc_T = ::core::ptr::null_mut::<ufunc_T>();
    let mut fname_buf: [::core::ffi::c_char; 41] = [0; 41];
    let mut error: ::core::ffi::c_int = 0;
    if (*rettv).v_type as ::core::ffi::c_uint
        == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
        && !(*rettv).vval.v_partial.is_null()
        && !(*(*rettv).vval.v_partial).pt_func.is_null()
    {
        fp = (*(*rettv).vval.v_partial).pt_func;
    } else {
        let mut fname: *mut ::core::ffi::c_char = if (*rettv).v_type as ::core::ffi::c_uint
            == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*rettv).v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*rettv).vval.v_string
        } else if (*rettv).vval.v_partial.is_null() {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        } else {
            (*(*rettv).vval.v_partial).pt_name
        };
        if fname.is_null() {
            (*rettv).v_type = VAR_FUNC;
            (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else {
            let mut tofree: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            fname = fname_trans_sid(
                fname,
                &raw mut fname_buf as *mut ::core::ffi::c_char,
                &raw mut tofree,
                &raw mut error,
            );
            fp = find_func(fname);
            xfree(tofree as *mut ::core::ffi::c_void);
        }
    }
    if !fp.is_null() && (*fp).uf_flags & FC_DICT != 0 {
        let mut pt: *mut partial_T =
            xcalloc(1 as size_t, ::core::mem::size_of::<partial_T>()) as *mut partial_T;
        (*pt).pt_refcount = 1 as ::core::ffi::c_int;
        (*pt).pt_dict = selfdict;
        (*selfdict).dv_refcount += 1;
        (*pt).pt_auto = true_0 != 0;
        if (*rettv).v_type as ::core::ffi::c_uint
            == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*rettv).v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*pt).pt_name = (*rettv).vval.v_string;
        } else {
            let mut ret_pt: *mut partial_T = (*rettv).vval.v_partial;
            if !(*ret_pt).pt_name.is_null() {
                (*pt).pt_name = xstrdup((*ret_pt).pt_name);
                func_ref((*pt).pt_name);
            } else {
                (*pt).pt_func = (*ret_pt).pt_func;
                func_ptr_ref((*pt).pt_func);
            }
            if (*ret_pt).pt_argc > 0 as ::core::ffi::c_int {
                let mut arg_size: size_t =
                    ::core::mem::size_of::<typval_T>().wrapping_mul((*ret_pt).pt_argc as size_t);
                (*pt).pt_argv = xmalloc(arg_size) as *mut typval_T;
                (*pt).pt_argc = (*ret_pt).pt_argc;
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < (*pt).pt_argc {
                    tv_copy(
                        (*ret_pt).pt_argv.offset(i as isize),
                        (*pt).pt_argv.offset(i as isize),
                    );
                    i += 1;
                }
            }
            partial_unref(ret_pt);
        }
        (*rettv).v_type = VAR_PARTIAL;
        (*rettv).vval.v_partial = pt;
    }
}
#[no_mangle]
pub unsafe extern "C" fn func_name(
    mut cookie: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_char {
    return &raw mut (*(*(cookie as *mut funccall_T)).fc_func).uf_name as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn func_breakpoint(mut cookie: *mut ::core::ffi::c_void) -> *mut linenr_T {
    return &raw mut (*(cookie as *mut funccall_T)).fc_breakpoint;
}
#[no_mangle]
pub unsafe extern "C" fn func_dbg_tick(
    mut cookie: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_int {
    return &raw mut (*(cookie as *mut funccall_T)).fc_dbg_tick;
}
#[no_mangle]
pub unsafe extern "C" fn func_level(mut cookie: *mut ::core::ffi::c_void) -> ::core::ffi::c_int {
    return (*(cookie as *mut funccall_T)).fc_level;
}
#[no_mangle]
pub unsafe extern "C" fn current_func_returned() -> ::core::ffi::c_int {
    return (*current_funccal).fc_returned;
}
#[no_mangle]
pub unsafe extern "C" fn free_unref_funccal(
    mut copyID: ::core::ffi::c_int,
    mut testing: ::core::ffi::c_int,
) -> bool {
    let mut did_free: bool = false_0 != 0;
    let mut did_free_funccal: bool = false_0 != 0;
    let mut pfc: *mut *mut funccall_T = &raw mut previous_funccal;
    while !(*pfc).is_null() {
        if can_free_funccal(*pfc, copyID) {
            let mut fc: *mut funccall_T = *pfc;
            *pfc = (*fc).fc_caller;
            free_funccal_contents(fc);
            did_free = true_0 != 0;
            did_free_funccal = true_0 != 0;
        } else {
            pfc = &raw mut (**pfc).fc_caller;
        }
    }
    if did_free_funccal {
        garbage_collect(testing != 0);
    }
    return did_free;
}
#[no_mangle]
pub unsafe extern "C" fn get_funccal() -> *mut funccall_T {
    let mut funccal: *mut funccall_T = current_funccal;
    if debug_backtrace_level > 0 as ::core::ffi::c_int {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < debug_backtrace_level {
            let mut temp_funccal: *mut funccall_T = (*funccal).fc_caller;
            if !temp_funccal.is_null() {
                funccal = temp_funccal;
            } else {
                debug_backtrace_level = i;
            }
            i += 1;
        }
    }
    return funccal;
}
#[no_mangle]
pub unsafe extern "C" fn get_funccal_local_dict() -> *mut dict_T {
    if current_funccal.is_null()
        || (*current_funccal).fc_l_vars.dv_refcount == 0 as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<dict_T>();
    }
    return &raw mut (*(get_funccal as unsafe extern "C" fn() -> *mut funccall_T)()).fc_l_vars;
}
#[no_mangle]
pub unsafe extern "C" fn get_funccal_local_ht() -> *mut hashtab_T {
    let mut d: *mut dict_T = get_funccal_local_dict();
    return if !d.is_null() {
        &raw mut (*d).dv_hashtab
    } else {
        ::core::ptr::null_mut::<hashtab_T>()
    };
}
#[no_mangle]
pub unsafe extern "C" fn get_funccal_local_var() -> *mut dictitem_T {
    if current_funccal.is_null()
        || (*current_funccal).fc_l_vars.dv_refcount == 0 as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<dictitem_T>();
    }
    return &raw mut (*(get_funccal as unsafe extern "C" fn() -> *mut funccall_T)()).fc_l_vars_var
        as *mut dictitem_T;
}
#[no_mangle]
pub unsafe extern "C" fn get_funccal_args_dict() -> *mut dict_T {
    if current_funccal.is_null()
        || (*current_funccal).fc_l_vars.dv_refcount == 0 as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<dict_T>();
    }
    return &raw mut (*(get_funccal as unsafe extern "C" fn() -> *mut funccall_T)()).fc_l_avars;
}
#[no_mangle]
pub unsafe extern "C" fn get_funccal_args_ht() -> *mut hashtab_T {
    let mut d: *mut dict_T = get_funccal_args_dict();
    return if !d.is_null() {
        &raw mut (*d).dv_hashtab
    } else {
        ::core::ptr::null_mut::<hashtab_T>()
    };
}
#[no_mangle]
pub unsafe extern "C" fn get_funccal_args_var() -> *mut dictitem_T {
    if current_funccal.is_null()
        || (*current_funccal).fc_l_vars.dv_refcount == 0 as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<dictitem_T>();
    }
    return &raw mut (*(get_funccal as unsafe extern "C" fn() -> *mut funccall_T)()).fc_l_avars_var
        as *mut dictitem_T;
}
#[no_mangle]
pub unsafe extern "C" fn list_func_vars(mut first: *mut ::core::ffi::c_int) {
    if !current_funccal.is_null()
        && (*current_funccal).fc_l_vars.dv_refcount > 0 as ::core::ffi::c_int
    {
        list_hashtable_vars(
            &raw mut (*current_funccal).fc_l_vars.dv_hashtab,
            b"l:\0".as_ptr() as *const ::core::ffi::c_char,
            false_0,
            first,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn get_current_funccal_dict(mut ht: *mut hashtab_T) -> *mut dict_T {
    if !current_funccal.is_null() && ht == &raw mut (*current_funccal).fc_l_vars.dv_hashtab {
        return &raw mut (*current_funccal).fc_l_vars;
    }
    return ::core::ptr::null_mut::<dict_T>();
}
#[no_mangle]
pub unsafe extern "C" fn find_hi_in_scoped_ht(
    mut name: *const ::core::ffi::c_char,
    mut pht: *mut *mut hashtab_T,
) -> *mut hashitem_T {
    if current_funccal.is_null() || (*(*current_funccal).fc_func).uf_scoped.is_null() {
        return ::core::ptr::null_mut::<hashitem_T>();
    }
    let mut old_current_funccal: *mut funccall_T = current_funccal;
    let mut hi: *mut hashitem_T = ::core::ptr::null_mut::<hashitem_T>();
    let namelen: size_t = strlen(name);
    let mut varname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    current_funccal = (*(*current_funccal).fc_func).uf_scoped;
    while !current_funccal.is_null() {
        let mut ht: *mut hashtab_T = find_var_ht(name, namelen, &raw mut varname);
        if !ht.is_null() && *varname as ::core::ffi::c_int != NUL {
            hi = hash_find_len(
                ht,
                varname,
                namelen.wrapping_sub(varname.offset_from(name) as size_t),
            );
            if !((*hi).hi_key.is_null() || (*hi).hi_key == &raw mut hash_removed) {
                *pht = ht;
                break;
            }
        }
        if current_funccal == (*(*current_funccal).fc_func).uf_scoped {
            break;
        }
        current_funccal = (*(*current_funccal).fc_func).uf_scoped;
    }
    current_funccal = old_current_funccal;
    return hi;
}
#[no_mangle]
pub unsafe extern "C" fn find_var_in_scoped_ht(
    mut name: *const ::core::ffi::c_char,
    namelen: size_t,
    mut no_autoload: ::core::ffi::c_int,
) -> *mut dictitem_T {
    if current_funccal.is_null() || (*(*current_funccal).fc_func).uf_scoped.is_null() {
        return ::core::ptr::null_mut::<dictitem_T>();
    }
    let mut v: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    let mut old_current_funccal: *mut funccall_T = current_funccal;
    let mut varname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    current_funccal = (*(*current_funccal).fc_func).uf_scoped;
    while !current_funccal.is_null() {
        let mut ht: *mut hashtab_T = find_var_ht(name, namelen, &raw mut varname);
        if !ht.is_null() && *varname as ::core::ffi::c_int != NUL {
            v = find_var_in_ht(
                ht,
                *name as ::core::ffi::c_int,
                varname,
                namelen.wrapping_sub(varname.offset_from(name) as size_t),
                no_autoload,
            );
            if !v.is_null() {
                break;
            }
        }
        if current_funccal == (*(*current_funccal).fc_func).uf_scoped {
            break;
        }
        current_funccal = (*(*current_funccal).fc_func).uf_scoped;
    }
    current_funccal = old_current_funccal;
    return v;
}
#[no_mangle]
pub unsafe extern "C" fn set_ref_in_previous_funccal(mut copyID: ::core::ffi::c_int) -> bool {
    let mut fc: *mut funccall_T = previous_funccal;
    while !fc.is_null() {
        (*fc).fc_copyID = copyID + 1 as ::core::ffi::c_int;
        if set_ref_in_ht(
            &raw mut (*fc).fc_l_vars.dv_hashtab,
            copyID + 1 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<*mut list_stack_T>(),
        ) as ::core::ffi::c_int
            != 0
            || set_ref_in_ht(
                &raw mut (*fc).fc_l_avars.dv_hashtab,
                copyID + 1 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as ::core::ffi::c_int
                != 0
            || set_ref_in_list_items(
                &raw mut (*fc).fc_l_varlist,
                copyID + 1 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
            ) as ::core::ffi::c_int
                != 0
        {
            return true_0 != 0;
        }
        fc = (*fc).fc_caller;
    }
    return false_0 != 0;
}
unsafe extern "C" fn set_ref_in_funccal(
    mut fc: *mut funccall_T,
    mut copyID: ::core::ffi::c_int,
) -> bool {
    if (*fc).fc_copyID != copyID {
        (*fc).fc_copyID = copyID;
        if set_ref_in_ht(
            &raw mut (*fc).fc_l_vars.dv_hashtab,
            copyID,
            ::core::ptr::null_mut::<*mut list_stack_T>(),
        ) as ::core::ffi::c_int
            != 0
            || set_ref_in_ht(
                &raw mut (*fc).fc_l_avars.dv_hashtab,
                copyID,
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as ::core::ffi::c_int
                != 0
            || set_ref_in_list_items(
                &raw mut (*fc).fc_l_varlist,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
            ) as ::core::ffi::c_int
                != 0
            || set_ref_in_func(
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                (*fc).fc_func,
                copyID,
            ) as ::core::ffi::c_int
                != 0
        {
            return true_0 != 0;
        }
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn set_ref_in_call_stack(mut copyID: ::core::ffi::c_int) -> bool {
    let mut fc: *mut funccall_T = current_funccal;
    while !fc.is_null() {
        if set_ref_in_funccal(fc, copyID) {
            return true_0 != 0;
        }
        fc = (*fc).fc_caller;
    }
    let mut entry: *mut funccal_entry_T = funccal_stack;
    while !entry.is_null() {
        let mut fc_0: *mut funccall_T = (*entry).top_funccal as *mut funccall_T;
        while !fc_0.is_null() {
            if set_ref_in_funccal(fc_0, copyID) {
                return true_0 != 0;
            }
            fc_0 = (*fc_0).fc_caller;
        }
        entry = (*entry).next;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn set_ref_in_functions(mut copyID: ::core::ffi::c_int) -> bool {
    let mut todo: ::core::ffi::c_int = func_hashtab.ht_used as ::core::ffi::c_int;
    let mut hi: *mut hashitem_T = func_hashtab.ht_array;
    while todo > 0 as ::core::ffi::c_int && !got_int {
        if !((*hi).hi_key.is_null() || (*hi).hi_key == &raw mut hash_removed) {
            todo -= 1;
            let mut fp: *mut ufunc_T =
                (*hi).hi_key.offset(-(240 as ::core::ffi::c_ulong as isize)) as *mut ufunc_T;
            if !func_name_refcount(&raw mut (*fp).uf_name as *mut ::core::ffi::c_char)
                && set_ref_in_func(::core::ptr::null_mut::<::core::ffi::c_char>(), fp, copyID)
                    as ::core::ffi::c_int
                    != 0
            {
                return true_0 != 0;
            }
        }
        hi = hi.offset(1);
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn set_ref_in_func_args(mut copyID: ::core::ffi::c_int) -> bool {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < funcargs.ga_len {
        if set_ref_in_item(
            *(funcargs.ga_data as *mut *mut typval_T).offset(i as isize),
            copyID,
            ::core::ptr::null_mut::<*mut ht_stack_T>(),
            ::core::ptr::null_mut::<*mut list_stack_T>(),
        ) {
            return true_0 != 0;
        }
        i += 1;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn set_ref_in_func(
    mut name: *mut ::core::ffi::c_char,
    mut fp_in: *mut ufunc_T,
    mut copyID: ::core::ffi::c_int,
) -> bool {
    let mut fp: *mut ufunc_T = fp_in;
    let mut error: ::core::ffi::c_int = FCERR_NONE as ::core::ffi::c_int;
    let mut fname_buf: [::core::ffi::c_char; 41] = [0; 41];
    let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut abort_0: bool = false_0 != 0;
    if name.is_null() && fp_in.is_null() {
        return false_0 != 0;
    }
    if fp_in.is_null() {
        let mut fname: *mut ::core::ffi::c_char = fname_trans_sid(
            name,
            &raw mut fname_buf as *mut ::core::ffi::c_char,
            &raw mut tofree,
            &raw mut error,
        );
        fp = find_func(fname);
    }
    if !fp.is_null() {
        let mut fc: *mut funccall_T = (*fp).uf_scoped;
        while !fc.is_null() {
            abort_0 = abort_0 as ::core::ffi::c_int != 0
                || set_ref_in_funccal(fc, copyID) as ::core::ffi::c_int != 0;
            fc = (*(*fc).fc_func).uf_scoped;
        }
    }
    xfree(tofree as *mut ::core::ffi::c_void);
    return abort_0;
}
#[no_mangle]
pub unsafe extern "C" fn register_luafunc(mut ref_0: LuaRef) -> *mut ::core::ffi::c_char {
    let mut name: String_0 = get_lambda_name();
    let mut fp: *mut ufunc_T = alloc_ufunc(name.data, name.size);
    (*fp).uf_refcount = 1 as ::core::ffi::c_int;
    (*fp).uf_varargs = true_0;
    (*fp).uf_flags = FC_LUAREF;
    (*fp).uf_calls = 0 as ::core::ffi::c_int;
    (*fp).uf_script_ctx = current_sctx;
    (*fp).uf_luaref = ref_0;
    hash_add(
        &raw mut func_hashtab,
        &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
    );
    return &raw mut (*fp).uf_name as *mut ::core::ffi::c_char;
}
pub const FC_ABORT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const FC_RANGE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const FC_DICT: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const FC_CLOSURE: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const FC_DELETED: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const FC_REMOVED: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const FC_SANDBOX: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const FC_NOARGS: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const FC_LUAREF: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const FUNCEXE_INIT: funcexe_T = funcexe_T {
    fe_argv_func: None,
    fe_firstline: 0 as linenr_T,
    fe_lastline: 0 as linenr_T,
    fe_doesrange: ::core::ptr::null_mut::<bool>(),
    fe_evaluate: false_0 != 0,
    fe_partial: ::core::ptr::null_mut::<partial_T>(),
    fe_selfdict: ::core::ptr::null_mut::<dict_T>(),
    fe_basetv: ::core::ptr::null_mut::<typval_T>(),
    fe_found_var: false_0 != 0,
};
pub const K_SPECIAL: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const KS_EXTRA: ::core::ffi::c_int = 253 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const MSG_BUF_LEN: ::core::ffi::c_int = 480 as ::core::ffi::c_int;
pub const MSG_BUF_CLEN: ::core::ffi::c_int = MSG_BUF_LEN / 6 as ::core::ffi::c_int;
pub const PROF_YES: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
