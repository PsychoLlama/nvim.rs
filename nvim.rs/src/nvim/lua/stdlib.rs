extern "C" {
    pub type lua_State;
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
    fn lua_gettop(L: *mut lua_State) -> ::core::ffi::c_int;
    fn lua_settop(L: *mut lua_State, idx: ::core::ffi::c_int);
    fn lua_pushvalue(L: *mut lua_State, idx: ::core::ffi::c_int);
    fn lua_type(L: *mut lua_State, idx: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn lua_toboolean(L: *mut lua_State, idx: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn lua_tolstring(
        L: *mut lua_State,
        idx: ::core::ffi::c_int,
        len: *mut size_t,
    ) -> *const ::core::ffi::c_char;
    fn lua_pushnil(L: *mut lua_State);
    fn lua_pushnumber(L: *mut lua_State, n: lua_Number);
    fn lua_pushinteger(L: *mut lua_State, n: lua_Integer);
    fn lua_pushlstring(L: *mut lua_State, s: *const ::core::ffi::c_char, l: size_t);
    fn lua_pushstring(L: *mut lua_State, s: *const ::core::ffi::c_char);
    fn lua_pushvfstring(
        L: *mut lua_State,
        fmt: *const ::core::ffi::c_char,
        argp: ::core::ffi::VaList,
    ) -> *const ::core::ffi::c_char;
    fn lua_pushcclosure(L: *mut lua_State, fn_0: lua_CFunction, n: ::core::ffi::c_int);
    fn lua_getfield(
        L: *mut lua_State,
        idx: ::core::ffi::c_int,
        k: *const ::core::ffi::c_char,
    );
    fn lua_createtable(
        L: *mut lua_State,
        narr: ::core::ffi::c_int,
        nrec: ::core::ffi::c_int,
    );
    fn lua_newuserdata(L: *mut lua_State, sz: size_t) -> *mut ::core::ffi::c_void;
    fn lua_setfield(
        L: *mut lua_State,
        idx: ::core::ffi::c_int,
        k: *const ::core::ffi::c_char,
    );
    fn lua_rawseti(L: *mut lua_State, idx: ::core::ffi::c_int, n: ::core::ffi::c_int);
    fn lua_setmetatable(
        L: *mut lua_State,
        objindex: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn lua_pcall(
        L: *mut lua_State,
        nargs: ::core::ffi::c_int,
        nresults: ::core::ffi::c_int,
        errfunc: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn lua_error(L: *mut lua_State) -> ::core::ffi::c_int;
    fn lua_next(L: *mut lua_State, idx: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn lua_concat(L: *mut lua_State, n: ::core::ffi::c_int);
    fn luaL_register(
        L: *mut lua_State,
        libname: *const ::core::ffi::c_char,
        l: *const luaL_Reg,
    );
    fn luaL_argerror(
        L: *mut lua_State,
        numarg: ::core::ffi::c_int,
        extramsg: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn luaL_checklstring(
        L: *mut lua_State,
        numArg: ::core::ffi::c_int,
        l: *mut size_t,
    ) -> *const ::core::ffi::c_char;
    fn luaL_checkinteger(L: *mut lua_State, numArg: ::core::ffi::c_int) -> lua_Integer;
    fn luaL_newmetatable(
        L: *mut lua_State,
        tname: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn luaL_checkudata(
        L: *mut lua_State,
        ud: ::core::ffi::c_int,
        tname: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_void;
    fn luaL_where(L: *mut lua_State, lvl: ::core::ffi::c_int);
    fn luaL_error(
        L: *mut lua_State,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memchr(
        __s: *const ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strcasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    static mut g_min_log_level: ::core::ffi::c_int;
    fn lua_cjson_new(l: *mut lua_State) -> ::core::ffi::c_int;
    fn luaopen_mpack(L: *mut lua_State) -> ::core::ffi::c_int;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn strequal(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char) -> bool;
    fn mh_get_int(set: *mut Set_int, key: ::core::ffi::c_int) -> uint32_t;
    fn try_enter(tstate: *mut TryState);
    fn try_leave(tstate: *const TryState, err: *mut Error);
    fn dict_check_writable(
        dict: *mut dict_T,
        key: String_0,
        del: bool,
        err: *mut Error,
    ) -> *mut dictitem_T;
    fn find_buffer_by_handle(buffer: Buffer, err: *mut Error) -> *mut buf_T;
    fn find_window_by_handle(window: Window, err: *mut Error) -> *mut win_T;
    fn find_tab_by_handle(tabpage: Tabpage, err: *mut Error) -> *mut tabpage_T;
    fn api_clear_error(value: *mut Error);
    fn aucmd_prepbuf(aco: *mut aco_save_T, buf: *mut buf_T);
    fn aucmd_restbuf(aco: *mut aco_save_T);
    static mut buffer_handles: Map_int_ptr_t;
    static mut window_handles: Map_int_ptr_t;
    fn tv_dict_watcher_notify(
        dict: *mut dict_T,
        key: *const ::core::ffi::c_char,
        newtv: *mut typval_T,
        oldtv: *mut typval_T,
    );
    fn tv_dict_item_alloc_len(
        key: *const ::core::ffi::c_char,
        key_len: size_t,
    ) -> *mut dictitem_T;
    fn tv_dict_item_remove(dict: *mut dict_T, item: *mut dictitem_T);
    fn tv_dict_find(
        d: *const dict_T,
        key: *const ::core::ffi::c_char,
        len: ptrdiff_t,
    ) -> *mut dictitem_T;
    fn tv_dict_add(d: *mut dict_T, item: *mut dictitem_T) -> ::core::ffi::c_int;
    fn tv_clear(tv: *mut typval_T);
    fn tv_copy(from: *const typval_T, to: *mut typval_T);
    fn get_globvar_dict() -> *mut dict_T;
    fn get_vimvar_dict() -> *mut dict_T;
    fn before_set_vvar(
        varname: *const ::core::ffi::c_char,
        di: *mut dictitem_T,
        tv: *mut typval_T,
        copy: bool,
        watched: bool,
        type_error: *mut bool,
    ) -> bool;
    fn win_execute_before(
        args: *mut win_execute_T,
        wp: *mut win_T,
        tp: *mut tabpage_T,
    ) -> bool;
    fn win_execute_after(args: *mut win_execute_T);
    fn apply_cmdmod(cmod: *mut cmdmod_T);
    fn undo_cmdmod(cmod: *mut cmdmod_T);
    fn aborting() -> bool;
    fn foldUpdate(wp: *mut win_T, top: linenr_T, bot: linenr_T);
    fn luaopen_base64(L: *mut lua_State) -> ::core::ffi::c_int;
    static mut curbuf: *mut buf_T;
    static mut cmdmod: cmdmod_T;
    fn nlua_pop_typval(lstate: *mut lua_State, ret_tv: *mut typval_T) -> bool;
    fn nlua_push_typval(
        lstate: *mut lua_State,
        tv: *mut typval_T,
        flags: ::core::ffi::c_int,
    ) -> bool;
    fn luaopen_spell(L: *mut lua_State) -> ::core::ffi::c_int;
    fn nlua_xdl_diff(lstate: *mut lua_State) -> ::core::ffi::c_int;
    fn utf_ptr2len_len(
        p: *const ::core::ffi::c_char,
        size: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn mb_utflen(
        s: *const ::core::ffi::c_char,
        len: size_t,
        codepoints: *mut size_t,
        codeunits: *mut size_t,
    );
    fn mb_utf_index_to_bytes(
        s: *const ::core::ffi::c_char,
        len: size_t,
        index: size_t,
        use_utf16_units: bool,
    ) -> ssize_t;
    fn utf_cp_bounds_len(
        base: *const ::core::ffi::c_char,
        p_in: *const ::core::ffi::c_char,
        p_len: ::core::ffi::c_int,
    ) -> CharBoundsOff;
    fn enc_skip(p: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn enc_canonize(enc: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn convert_setup(
        vcp: *mut vimconv_T,
        from: *mut ::core::ffi::c_char,
        to: *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn convert_setup_ext(
        vcp: *mut vimconv_T,
        from: *mut ::core::ffi::c_char,
        from_unicode_is_utf8: bool,
        to: *mut ::core::ffi::c_char,
        to_unicode_is_utf8: bool,
    ) -> ::core::ffi::c_int;
    fn string_convert(
        vcp: *const vimconv_T,
        ptr: *mut ::core::ffi::c_char,
        lenp: *mut size_t,
    ) -> *mut ::core::ffi::c_char;
    fn ml_get_buf(buf: *mut buf_T, lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_buf_len(buf: *mut buf_T, lnum: linenr_T) -> colnr_T;
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec(
        rmp: *mut regmatch_T,
        line: *const ::core::ffi::c_char,
        col: colnr_T,
    ) -> bool;
    fn script_autoload(
        name: *const ::core::ffi::c_char,
        name_len: size_t,
        reload: bool,
    ) -> bool;
    fn win_find_tabpage(win: *mut win_T) -> *mut tabpage_T;
}
pub type __builtin_va_list = [__va_list_tag; 1];
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __va_list_tag {
    pub gp_offset: ::core::ffi::c_uint,
    pub fp_offset: ::core::ffi::c_uint,
    pub overflow_arg_area: *mut ::core::ffi::c_void,
    pub reg_save_area: *mut ::core::ffi::c_void,
}
pub type ptrdiff_t = isize;
pub type size_t = usize;
pub type __gnuc_va_list = __builtin_va_list;
pub type __time_t = ::core::ffi::c_long;
pub type va_list = __gnuc_va_list;
pub type ssize_t = isize;
pub type lua_CFunction = Option<
    unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
>;
pub type lua_Number = ::core::ffi::c_double;
pub type lua_Integer = ptrdiff_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct luaL_Reg {
    pub name: *const ::core::ffi::c_char,
    pub func: lua_CFunction,
}
pub type int8_t = i8;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type intptr_t = isize;
pub type time_t = __time_t;
pub type schar_T = uint32_t;
pub type sattr_T = int32_t;
pub type handle_T = ::core::ffi::c_int;
pub type LuaRef = ::core::ffi::c_int;
pub type float_T = ::core::ffi::c_double;
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
    pub b_wininfo: C2Rust_Unnamed_10,
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
    pub b_signcols: C2Rust_Unnamed_2,
    pub terminal: *mut Terminal,
    pub additional_data: *mut AdditionalData,
    pub b_mapped_ctrl_c: ::core::ffi::c_int,
    pub b_marktree: [MarkTree; 1],
    pub b_extmark_ns: [Map_uint32_t_uint32_t; 1],
    pub b_prev_line_count: ::core::ffi::c_int,
    pub update_channels: C2Rust_Unnamed_0,
    pub update_callbacks: C2Rust_Unnamed,
    pub update_need_codepoints: bool,
    pub deleted_bytes: size_t,
    pub deleted_bytes2: size_t,
    pub deleted_codepoints: size_t,
    pub deleted_codeunits: size_t,
    pub flush_count: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed {
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
pub struct C2Rust_Unnamed_0 {
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
    pub data: C2Rust_Unnamed_1,
    pub next: *mut DecorVirtText,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_1 {
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
pub struct C2Rust_Unnamed_2 {
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
    pub sst_union: C2Rust_Unnamed_3,
    pub sst_next_flags: ::core::ffi::c_int,
    pub sst_stacksize: ::core::ffi::c_int,
    pub sst_next_list: *mut int16_t,
    pub sst_tick: disptick_T,
    pub sst_change_lnum: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_3 {
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
    pub data: C2Rust_Unnamed_4,
    pub type_0: CallbackType,
}
pub type CallbackType = ::core::ffi::c_uint;
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_4 {
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
    pub fc_fixvar: [C2Rust_Unnamed_5; 12],
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
pub struct C2Rust_Unnamed_5 {
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
    pub uh_next: C2Rust_Unnamed_9,
    pub uh_prev: C2Rust_Unnamed_8,
    pub uh_alt_next: C2Rust_Unnamed_7,
    pub uh_alt_prev: C2Rust_Unnamed_6,
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
pub union C2Rust_Unnamed_6 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_7 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
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
pub struct C2Rust_Unnamed_10 {
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
    pub type_0: C2Rust_Unnamed_11,
    pub tabnr: ::core::ffi::c_int,
    pub func: *mut ::core::ffi::c_char,
}
pub type C2Rust_Unnamed_11 = ::core::ffi::c_uint;
pub const kStlClickFuncRun: C2Rust_Unnamed_11 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_11 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_11 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_11 = 0;
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
pub type ErrorType = ::core::ffi::c_int;
pub const kErrorTypeValidation: ErrorType = 1;
pub const kErrorTypeException: ErrorType = 0;
pub const kErrorTypeNone: ErrorType = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Error {
    pub type_0: ErrorType,
    pub msg: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct String_0 {
    pub data: *mut ::core::ffi::c_char,
    pub size: size_t,
}
pub type Buffer = handle_T;
pub type Tabpage = handle_T;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dictitem_T {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 0],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Set_int {
    pub h: MapHash,
    pub keys: *mut ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_int_ptr_t {
    pub set: Set_int,
    pub values: *mut ptr_t,
}
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
pub struct bufref_T {
    pub br_buf: *mut buf_T,
    pub br_fnum: ::core::ffi::c_int,
    pub br_buf_free_count: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct diffblock_S {
    pub df_next: *mut diff_T,
    pub df_lnum: [linenr_T; 8],
    pub df_count: [linenr_T; 8],
    pub is_linematched: bool,
    pub has_changes: bool,
    pub df_changes: garray_T,
}
pub type diff_T = diffblock_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tabpage_S {
    pub handle: handle_T,
    pub tp_next: *mut tabpage_T,
    pub tp_topframe: *mut frame_T,
    pub tp_curwin: *mut win_T,
    pub tp_prevwin: *mut win_T,
    pub tp_firstwin: *mut win_T,
    pub tp_lastwin: *mut win_T,
    pub tp_old_Rows_avail: int64_t,
    pub tp_old_Columns: int64_t,
    pub tp_ch_used: OptInt,
    pub tp_did_tabclosedpre: bool,
    pub tp_first_diff: *mut diff_T,
    pub tp_diffbuf: [*mut buf_T; 8],
    pub tp_diff_invalid: ::core::ffi::c_int,
    pub tp_diff_update: ::core::ffi::c_int,
    pub tp_snapshot: [*mut frame_T; 3],
    pub tp_winvar: ScopeDictDictItem,
    pub tp_vars: *mut dict_T,
    pub tp_localdir: *mut ::core::ffi::c_char,
    pub tp_prevdir: *mut ::core::ffi::c_char,
}
pub type tabpage_T = tabpage_S;
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
pub struct TryState {
    pub current_exception: *mut except_T,
    pub private_msg_list: *mut msglist_T,
    pub msg_list: *const *const msglist_T,
    pub got_int: ::core::ffi::c_int,
    pub did_throw: bool,
    pub need_rethrow: ::core::ffi::c_int,
    pub did_emsg: ::core::ffi::c_int,
}
pub type C2Rust_Unnamed_12 = ::core::ffi::c_uint;
pub const CMOD_NOSWAPFILE: C2Rust_Unnamed_12 = 8192;
pub const CMOD_KEEPPATTERNS: C2Rust_Unnamed_12 = 4096;
pub const CMOD_LOCKMARKS: C2Rust_Unnamed_12 = 2048;
pub const CMOD_KEEPJUMPS: C2Rust_Unnamed_12 = 1024;
pub const CMOD_KEEPMARKS: C2Rust_Unnamed_12 = 512;
pub const CMOD_KEEPALT: C2Rust_Unnamed_12 = 256;
pub const CMOD_CONFIRM: C2Rust_Unnamed_12 = 128;
pub const CMOD_BROWSE: C2Rust_Unnamed_12 = 64;
pub const CMOD_HIDE: C2Rust_Unnamed_12 = 32;
pub const CMOD_NOAUTOCMD: C2Rust_Unnamed_12 = 16;
pub const CMOD_UNSILENT: C2Rust_Unnamed_12 = 8;
pub const CMOD_ERRSILENT: C2Rust_Unnamed_12 = 4;
pub const CMOD_SILENT: C2Rust_Unnamed_12 = 2;
pub const CMOD_SANDBOX: C2Rust_Unnamed_12 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmdmod_T {
    pub cmod_flags: ::core::ffi::c_int,
    pub cmod_split: ::core::ffi::c_int,
    pub cmod_tab: ::core::ffi::c_int,
    pub cmod_filter_pat: *mut ::core::ffi::c_char,
    pub cmod_filter_regmatch: regmatch_T,
    pub cmod_filter_force: bool,
    pub cmod_verbose: ::core::ffi::c_int,
    pub cmod_save_ei: *mut ::core::ffi::c_char,
    pub cmod_did_sandbox: ::core::ffi::c_int,
    pub cmod_verbose_save: OptInt,
    pub cmod_save_msg_silent: ::core::ffi::c_int,
    pub cmod_save_msg_scroll: ::core::ffi::c_int,
    pub cmod_did_esilent: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct aco_save_T {
    pub use_aucmd_win_idx: ::core::ffi::c_int,
    pub save_curwin_handle: handle_T,
    pub new_curwin_handle: handle_T,
    pub save_prevwin_handle: handle_T,
    pub new_curbuf: bufref_T,
    pub tp_localdir: *mut ::core::ffi::c_char,
    pub globaldir: *mut ::core::ffi::c_char,
    pub save_VIsual_active: bool,
    pub save_prompt_insert: ::core::ffi::c_int,
}
pub type iconv_t = *mut ::core::ffi::c_void;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const CONV_ICONV: C2Rust_Unnamed_13 = 5;
pub const CONV_TO_LATIN9: C2Rust_Unnamed_13 = 4;
pub const CONV_TO_LATIN1: C2Rust_Unnamed_13 = 3;
pub const CONV_9_TO_UTF8: C2Rust_Unnamed_13 = 2;
pub const CONV_TO_UTF8: C2Rust_Unnamed_13 = 1;
pub const CONV_NONE: C2Rust_Unnamed_13 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct vimconv_T {
    pub vc_type: ::core::ffi::c_int,
    pub vc_factor: ::core::ffi::c_int,
    pub vc_fd: iconv_t,
    pub vc_fail: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CharBoundsOff {
    pub begin_off: int8_t,
    pub end_off: int8_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct switchwin_T {
    pub sw_curwin: *mut win_T,
    pub sw_curtab: *mut tabpage_T,
    pub sw_same_win: bool,
    pub sw_visual_active: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct win_execute_T {
    pub wp: *mut win_T,
    pub curpos: pos_T,
    pub cwd: [::core::ffi::c_char; 4096],
    pub cwd_status: ::core::ffi::c_int,
    pub apply_acd: bool,
    pub save_sfname: *mut ::core::ffi::c_char,
    pub switchwin: switchwin_T,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<
    ::core::ffi::c_void,
>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<
    ::core::ffi::c_void,
>();
pub const LUA_REGISTRYINDEX: ::core::ffi::c_int = -10000 as ::core::ffi::c_int;
pub const LUA_GLOBALSINDEX: ::core::ffi::c_int = -10002 as ::core::ffi::c_int;
pub const LUA_TNIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const LUA_TSTRING: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
static mut value_init_ptr_t: ptr_t = NULL;
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn map_get_int_ptr_t(
    mut map: *mut Map_int_ptr_t,
    mut key: ::core::ffi::c_int,
) -> ptr_t {
    let mut k: uint32_t = mh_get_int(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_ptr_t
    } else {
        *(*map).values.offset(k as isize)
    };
}
#[inline(always)]
unsafe extern "C" fn QUEUE_EMPTY(q: *const QUEUE) -> ::core::ffi::c_int {
    return (q == (*q).next as *const QUEUE) as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn tv_dict_is_watched(d: *const dict_T) -> bool {
    return !d.is_null() && QUEUE_EMPTY(&raw const (*d).watchers) == 0;
}
unsafe extern "C" fn regex_match(
    mut lstate: *mut lua_State,
    mut prog: *mut *mut regprog_T,
    mut str: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut rm: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    rm.regprog = *prog;
    rm.rm_ic = false_0 != 0;
    let mut match_0: bool = vim_regexec(&raw mut rm, str, 0 as colnr_T);
    *prog = rm.regprog;
    if match_0 {
        lua_pushinteger(
            lstate,
            rm.startp[0 as ::core::ffi::c_int as usize].offset_from(str),
        );
        lua_pushinteger(
            lstate,
            rm.endp[0 as ::core::ffi::c_int as usize].offset_from(str),
        );
        return 2 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn regex_match_str(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut prog: *mut *mut regprog_T = regex_check(lstate);
    let mut str: *const ::core::ffi::c_char = luaL_checklstring(
        lstate,
        2 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<size_t>(),
    );
    let mut nret: ::core::ffi::c_int = regex_match(
        lstate,
        prog,
        str as *mut ::core::ffi::c_char,
    );
    if (*prog).is_null() {
        return luaL_error(
            lstate,
            b"regex: internal error\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    return nret;
}
unsafe extern "C" fn regex_match_line(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut prog: *mut *mut regprog_T = regex_check(lstate);
    let mut narg: ::core::ffi::c_int = lua_gettop(lstate);
    if narg < 3 as ::core::ffi::c_int {
        return luaL_error(
            lstate,
            b"not enough args\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    let mut bufnr: handle_T = luaL_checkinteger(lstate, 2 as ::core::ffi::c_int)
        as handle_T;
    let mut rownr: linenr_T = luaL_checkinteger(lstate, 3 as ::core::ffi::c_int)
        as linenr_T;
    let mut start: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut end: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if narg >= 4 as ::core::ffi::c_int {
        start = luaL_checkinteger(lstate, 4 as ::core::ffi::c_int) as ::core::ffi::c_int;
    }
    if narg >= 5 as ::core::ffi::c_int {
        end = luaL_checkinteger(lstate, 5 as ::core::ffi::c_int) as ::core::ffi::c_int;
        if end < 0 as ::core::ffi::c_int {
            return luaL_error(
                lstate,
                b"invalid end\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    }
    let mut buf: *mut buf_T = (if bufnr != 0 {
        map_get_int_ptr_t(&raw mut buffer_handles, bufnr as ::core::ffi::c_int)
    } else {
        curbuf as *mut ::core::ffi::c_void
    }) as *mut buf_T;
    if buf.is_null() || (*buf).b_ml.ml_mfp.is_null() {
        return luaL_error(
            lstate,
            b"invalid buffer\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    if rownr >= (*buf).b_ml.ml_line_count {
        return luaL_error(
            lstate,
            b"invalid row\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    let mut line: *mut ::core::ffi::c_char = ml_get_buf(buf, rownr + 1 as linenr_T);
    let mut len: colnr_T = ml_get_buf_len(buf, rownr + 1 as linenr_T);
    if start < 0 as ::core::ffi::c_int || start > len {
        return luaL_error(
            lstate,
            b"invalid start\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    let mut save: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
    if end >= 0 as ::core::ffi::c_int {
        if end > len || end < start {
            return luaL_error(
                lstate,
                b"invalid end\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save = *line.offset(end as isize);
        *line.offset(end as isize) = NUL as ::core::ffi::c_char;
    }
    let mut nret: ::core::ffi::c_int = regex_match(
        lstate,
        prog,
        line.offset(start as isize),
    );
    if end >= 0 as ::core::ffi::c_int {
        *line.offset(end as isize) = save;
    }
    if (*prog).is_null() {
        return luaL_error(
            lstate,
            b"regex: internal error\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    return nret;
}
unsafe extern "C" fn regex_check(mut L: *mut lua_State) -> *mut *mut regprog_T {
    return luaL_checkudata(
        L,
        1 as ::core::ffi::c_int,
        b"nvim_regex\0".as_ptr() as *const ::core::ffi::c_char,
    ) as *mut *mut regprog_T;
}
unsafe extern "C" fn regex_gc(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut prog: *mut *mut regprog_T = regex_check(lstate);
    vim_regfree(*prog);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn regex_tostring(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    lua_pushstring(lstate, b"<regex>\0".as_ptr() as *const ::core::ffi::c_char);
    return 1 as ::core::ffi::c_int;
}
static mut regex_meta: [luaL_Reg; 5] = [
    luaL_Reg {
        name: b"__gc\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            regex_gc as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"__tostring\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            regex_tostring as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"match_str\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            regex_match_str as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"match_line\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            regex_match_line
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        func: None,
    },
];
#[no_mangle]
pub unsafe extern "C" fn nlua_str_utfindex(
    lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut s1_len: size_t = 0;
    let mut s1: *const ::core::ffi::c_char = luaL_checklstring(
        lstate,
        1 as ::core::ffi::c_int,
        &raw mut s1_len,
    );
    let mut idx: intptr_t = 0;
    if lua_type(lstate, 2 as ::core::ffi::c_int) <= 0 as ::core::ffi::c_int {
        idx = s1_len as intptr_t;
    } else {
        idx = luaL_checkinteger(lstate, 2 as ::core::ffi::c_int) as intptr_t;
        if idx < 0 as intptr_t || idx > s1_len as intptr_t {
            lua_pushnil(lstate);
            lua_pushnil(lstate);
            return 2 as ::core::ffi::c_int;
        }
    }
    let mut codepoints: size_t = 0 as size_t;
    let mut codeunits: size_t = 0 as size_t;
    mb_utflen(s1, idx as size_t, &raw mut codepoints, &raw mut codeunits);
    lua_pushinteger(lstate, codepoints as lua_Integer);
    lua_pushinteger(lstate, codeunits as lua_Integer);
    return 2 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_str_utf_pos(lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut s1_len: size_t = 0;
    let mut s1: *const ::core::ffi::c_char = luaL_checklstring(
        lstate,
        1 as ::core::ffi::c_int,
        &raw mut s1_len,
    );
    lua_createtable(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    let mut idx: size_t = 1 as size_t;
    let mut clen: size_t = 0;
    let mut i: size_t = 0 as size_t;
    while i < s1_len && *s1.offset(i as isize) as ::core::ffi::c_int != NUL {
        clen = utf_ptr2len_len(
            s1.offset(i as isize),
            s1_len.wrapping_sub(i) as ::core::ffi::c_int,
        ) as size_t;
        lua_pushinteger(lstate, i as lua_Integer + 1 as lua_Integer);
        lua_rawseti(lstate, -2 as ::core::ffi::c_int, idx as ::core::ffi::c_int);
        idx = idx.wrapping_add(1);
        i = i.wrapping_add(clen);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_str_utf_start(lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut s1_len: size_t = 0;
    let mut s1: *const ::core::ffi::c_char = luaL_checklstring(
        lstate,
        1 as ::core::ffi::c_int,
        &raw mut s1_len,
    );
    let mut offset: ptrdiff_t = luaL_checkinteger(lstate, 2 as ::core::ffi::c_int);
    if offset <= 0 as ptrdiff_t || offset > s1_len as ptrdiff_t {
        return luaL_error(
            lstate,
            b"index out of range\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    let off: size_t = (offset - 1 as ptrdiff_t) as size_t;
    let mut head_off: ::core::ffi::c_int = -(utf_cp_bounds_len(
            s1,
            s1.offset(off as isize),
            s1_len.wrapping_sub(off) as ::core::ffi::c_int,
        )
        .begin_off as ::core::ffi::c_int);
    lua_pushinteger(lstate, head_off as lua_Integer);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_str_utf_end(lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut s1_len: size_t = 0;
    let mut s1: *const ::core::ffi::c_char = luaL_checklstring(
        lstate,
        1 as ::core::ffi::c_int,
        &raw mut s1_len,
    );
    let mut offset: ptrdiff_t = luaL_checkinteger(lstate, 2 as ::core::ffi::c_int);
    if offset <= 0 as ptrdiff_t || offset > s1_len as ptrdiff_t {
        return luaL_error(
            lstate,
            b"index out of range\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    let off: size_t = (offset - 1 as ptrdiff_t) as size_t;
    let mut tail_off: ::core::ffi::c_int = utf_cp_bounds_len(
            s1,
            s1.offset(off as isize),
            s1_len.wrapping_sub(off) as ::core::ffi::c_int,
        )
        .end_off as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
    lua_pushinteger(lstate, tail_off as lua_Integer);
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn nlua_str_byteindex(
    lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut s1_len: size_t = 0;
    let mut s1: *const ::core::ffi::c_char = luaL_checklstring(
        lstate,
        1 as ::core::ffi::c_int,
        &raw mut s1_len,
    );
    let mut idx: intptr_t = luaL_checkinteger(lstate, 2 as ::core::ffi::c_int)
        as intptr_t;
    if idx < 0 as intptr_t {
        lua_pushnil(lstate);
        return 1 as ::core::ffi::c_int;
    }
    let mut use_utf16: bool = false_0 != 0;
    if lua_gettop(lstate) >= 3 as ::core::ffi::c_int {
        use_utf16 = lua_toboolean(lstate, 3 as ::core::ffi::c_int) != 0;
    }
    let mut byteidx: ssize_t = mb_utf_index_to_bytes(
        s1,
        s1_len,
        idx as size_t,
        use_utf16,
    );
    if byteidx == -1 as ssize_t {
        lua_pushnil(lstate);
        return 1 as ::core::ffi::c_int;
    }
    lua_pushinteger(lstate, byteidx as lua_Integer);
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn nlua_regex(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut text: *const ::core::ffi::c_char = luaL_checklstring(
        lstate,
        1 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<size_t>(),
    );
    let mut prog: *mut regprog_T = ::core::ptr::null_mut::<regprog_T>();
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    prog = vim_regcomp(
        text,
        8 as ::core::ffi::c_int | 1 as ::core::ffi::c_int | 4 as ::core::ffi::c_int,
    );
    try_leave(&raw mut tstate, &raw mut err);
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        nlua_push_errstr(
            lstate,
            b"couldn't parse regex: %s\0".as_ptr() as *const ::core::ffi::c_char,
            err.msg,
        );
        api_clear_error(&raw mut err);
        return lua_error(lstate);
    } else if prog.is_null() {
        nlua_push_errstr(
            lstate,
            b"couldn't parse regex\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return lua_error(lstate);
    }
    let mut p: *mut *mut regprog_T = lua_newuserdata(
        lstate,
        ::core::mem::size_of::<*mut regprog_T>(),
    ) as *mut *mut regprog_T;
    *p = prog;
    lua_getfield(
        lstate,
        LUA_REGISTRYINDEX,
        b"nvim_regex\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_setmetatable(lstate, -2 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_get_var_scope(mut lstate: *mut lua_State) -> *mut dict_T {
    let mut scope: *const ::core::ffi::c_char = luaL_checklstring(
        lstate,
        1 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<size_t>(),
    );
    let mut handle: handle_T = luaL_checkinteger(lstate, 2 as ::core::ffi::c_int)
        as handle_T;
    let mut dict: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    if strequal(scope, b"g\0".as_ptr() as *const ::core::ffi::c_char) {
        dict = get_globvar_dict();
    } else if strequal(scope, b"v\0".as_ptr() as *const ::core::ffi::c_char) {
        dict = get_vimvar_dict();
    } else if strequal(scope, b"b\0".as_ptr() as *const ::core::ffi::c_char) {
        let mut buf: *mut buf_T = find_buffer_by_handle(handle as Buffer, &raw mut err);
        if !buf.is_null() {
            dict = (*buf).b_vars;
        }
    } else if strequal(scope, b"w\0".as_ptr() as *const ::core::ffi::c_char) {
        let mut win: *mut win_T = find_window_by_handle(handle as Window, &raw mut err);
        if !win.is_null() {
            dict = (*win).w_vars;
        }
    } else if strequal(scope, b"t\0".as_ptr() as *const ::core::ffi::c_char) {
        let mut tabpage: *mut tabpage_T = find_tab_by_handle(
            handle as Tabpage,
            &raw mut err,
        );
        if !tabpage.is_null() {
            dict = (*tabpage).tp_vars;
        }
    } else {
        luaL_error(lstate, b"invalid scope\0".as_ptr() as *const ::core::ffi::c_char);
        return ::core::ptr::null_mut::<dict_T>();
    }
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        nlua_push_errstr(
            lstate,
            b"scoped variable: %s\0".as_ptr() as *const ::core::ffi::c_char,
            err.msg,
        );
        api_clear_error(&raw mut err);
        lua_error(lstate);
        return ::core::ptr::null_mut::<dict_T>();
    }
    return dict;
}
#[no_mangle]
pub unsafe extern "C" fn nlua_setvar(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut dict: *mut dict_T = nlua_get_var_scope(lstate);
    let mut key: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    key.data = luaL_checklstring(lstate, 3 as ::core::ffi::c_int, &raw mut key.size)
        as *mut ::core::ffi::c_char;
    let mut del: bool = lua_gettop(lstate) < 4 as ::core::ffi::c_int
        || lua_type(lstate, 4 as ::core::ffi::c_int) == LUA_TNIL;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut di: *mut dictitem_T = dict_check_writable(dict, key, del, &raw mut err);
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        nlua_push_errstr(
            lstate,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            err.msg,
        );
        api_clear_error(&raw mut err);
        lua_error(lstate);
        return 0 as ::core::ffi::c_int;
    }
    let mut watched: bool = tv_dict_is_watched(dict);
    if del {
        if di.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        if watched {
            tv_dict_watcher_notify(
                dict,
                key.data,
                ::core::ptr::null_mut::<typval_T>(),
                &raw mut (*di).di_tv,
            );
        }
        tv_dict_item_remove(dict, di);
    } else {
        let mut tv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        lua_pushvalue(lstate, 4 as ::core::ffi::c_int);
        if !nlua_pop_typval(lstate, &raw mut tv) {
            return luaL_error(
                lstate,
                b"Couldn't convert lua value\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        let mut oldtv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        if di.is_null() {
            di = tv_dict_item_alloc_len(key.data, key.size);
            tv_dict_add(dict, di);
        } else {
            let mut type_error: bool = false_0 != 0;
            if dict == get_vimvar_dict()
                && !before_set_vvar(
                    key.data,
                    di,
                    &raw mut tv,
                    true_0 != 0,
                    watched,
                    &raw mut type_error,
                )
            {
                tv_clear(&raw mut tv);
                if type_error {
                    return luaL_error(
                        lstate,
                        b"Setting v:%s to value with wrong type\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        key.data,
                    );
                }
                return 0 as ::core::ffi::c_int;
            }
            if watched {
                tv_copy(&raw mut (*di).di_tv, &raw mut oldtv);
            }
            tv_clear(&raw mut (*di).di_tv);
        }
        tv_copy(&raw mut tv, &raw mut (*di).di_tv);
        if watched {
            tv_dict_watcher_notify(dict, key.data, &raw mut tv, &raw mut oldtv);
            tv_clear(&raw mut oldtv);
        }
        tv_clear(&raw mut tv);
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn nlua_getvar(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut dict: *mut dict_T = nlua_get_var_scope(lstate);
    let mut len: size_t = 0;
    let mut name: *const ::core::ffi::c_char = luaL_checklstring(
        lstate,
        3 as ::core::ffi::c_int,
        &raw mut len,
    );
    let mut di: *mut dictitem_T = tv_dict_find(dict, name, len as ptrdiff_t);
    if di.is_null() && dict == get_globvar_dict() {
        if !script_autoload(name, len, false_0 != 0)
            || aborting() as ::core::ffi::c_int != 0
        {
            return 0 as ::core::ffi::c_int;
        }
        di = tv_dict_find(dict, name, len as ptrdiff_t);
    }
    if di.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    nlua_push_typval(lstate, &raw mut (*di).di_tv, 0 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_stricmp(lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut s1_len: size_t = 0;
    let mut s2_len: size_t = 0;
    let mut s1: *const ::core::ffi::c_char = luaL_checklstring(
        lstate,
        1 as ::core::ffi::c_int,
        &raw mut s1_len,
    );
    let mut s2: *const ::core::ffi::c_char = luaL_checklstring(
        lstate,
        2 as ::core::ffi::c_int,
        &raw mut s2_len,
    );
    let mut nul1: *const ::core::ffi::c_char = ::core::ptr::null::<
        ::core::ffi::c_char,
    >();
    let mut nul2: *const ::core::ffi::c_char = ::core::ptr::null::<
        ::core::ffi::c_char,
    >();
    let mut ret: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    '_c2rust_label: {
        if *s1.offset(s1_len as isize) as ::core::ffi::c_int
            == '\0' as ::core::ffi::c_int
        {} else {
            __assert_fail(
                b"s1[s1_len] == NUL\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/lua/stdlib.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                481 as ::core::ffi::c_uint,
                b"int nlua_stricmp(lua_State *const)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    '_c2rust_label_0: {
        if *s2.offset(s2_len as isize) as ::core::ffi::c_int
            == '\0' as ::core::ffi::c_int
        {} else {
            __assert_fail(
                b"s2[s2_len] == NUL\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/lua/stdlib.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                482 as ::core::ffi::c_uint,
                b"int nlua_stricmp(lua_State *const)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    loop {
        nul1 = memchr(s1 as *const ::core::ffi::c_void, NUL, s1_len)
            as *const ::core::ffi::c_char;
        nul2 = memchr(s2 as *const ::core::ffi::c_void, NUL, s2_len)
            as *const ::core::ffi::c_char;
        ret = strcasecmp(s1 as *mut ::core::ffi::c_char, s2 as *mut ::core::ffi::c_char);
        if ret != 0 as ::core::ffi::c_int {
            break;
        }
        if nul1.is_null() as ::core::ffi::c_int != nul2.is_null() as ::core::ffi::c_int {
            ret = !nul1.is_null() as ::core::ffi::c_int
                - !nul2.is_null() as ::core::ffi::c_int;
            break;
        } else {
            if nul1.is_null() {
                break;
            }
            '_c2rust_label_1: {
                if !nul2.is_null() {} else {
                    __assert_fail(
                        b"nul2 != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/lua/stdlib.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        494 as ::core::ffi::c_uint,
                        b"int nlua_stricmp(lua_State *const)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            s1_len = s1_len
                .wrapping_sub(
                    (nul1.offset_from(s1) as size_t).wrapping_add(1 as size_t),
                );
            s2_len = s2_len
                .wrapping_sub(
                    (nul2.offset_from(s2) as size_t).wrapping_add(1 as size_t),
                );
            s1 = nul1.offset(1 as ::core::ffi::c_int as isize);
            s2 = nul2.offset(1 as ::core::ffi::c_int as isize);
        }
    }
    lua_settop(lstate, -2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    lua_pushnumber(
        lstate,
        ((ret > 0 as ::core::ffi::c_int) as ::core::ffi::c_int
            - (ret < 0 as ::core::ffi::c_int) as ::core::ffi::c_int) as lua_Number,
    );
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_iconv(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut narg: ::core::ffi::c_int = lua_gettop(lstate);
    if narg < 3 as ::core::ffi::c_int {
        return luaL_error(
            lstate,
            b"Expected at least 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while i <= 3 as ::core::ffi::c_int {
        if lua_type(lstate, i) != LUA_TSTRING {
            return luaL_argerror(
                lstate,
                i,
                b"expected string\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        i += 1;
    }
    let mut str_len: size_t = 0 as size_t;
    let mut str: *const ::core::ffi::c_char = lua_tolstring(
        lstate,
        1 as ::core::ffi::c_int,
        &raw mut str_len,
    );
    let mut from: *mut ::core::ffi::c_char = enc_canonize(
        enc_skip(
            lua_tolstring(
                lstate,
                2 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<size_t>(),
            ) as *mut ::core::ffi::c_char,
        ),
    );
    let mut to: *mut ::core::ffi::c_char = enc_canonize(
        enc_skip(
            lua_tolstring(
                lstate,
                3 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<size_t>(),
            ) as *mut ::core::ffi::c_char,
        ),
    );
    let mut vimconv: vimconv_T = vimconv_T {
        vc_type: 0,
        vc_factor: 0,
        vc_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        vc_fail: false,
    };
    vimconv.vc_type = CONV_NONE as ::core::ffi::c_int;
    convert_setup_ext(&raw mut vimconv, from, false_0 != 0, to, false_0 != 0);
    let mut ret: *mut ::core::ffi::c_char = string_convert(
        &raw mut vimconv,
        str as *mut ::core::ffi::c_char,
        &raw mut str_len,
    );
    convert_setup(
        &raw mut vimconv,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    );
    xfree(from as *mut ::core::ffi::c_void);
    xfree(to as *mut ::core::ffi::c_void);
    if ret.is_null() {
        lua_pushnil(lstate);
    } else {
        lua_pushlstring(lstate, ret, str_len);
        xfree(ret as *mut ::core::ffi::c_void);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_foldupdate(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut window: handle_T = luaL_checkinteger(lstate, 1 as ::core::ffi::c_int)
        as handle_T;
    let mut win: *mut win_T = map_get_int_ptr_t(
        &raw mut window_handles,
        window as ::core::ffi::c_int,
    ) as *mut win_T;
    if win.is_null() {
        return luaL_error(
            lstate,
            b"invalid window\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    let mut top: linenr_T = luaL_checkinteger(lstate, 2 as ::core::ffi::c_int)
        as linenr_T + 1 as linenr_T;
    if top < 1 as linenr_T {
        return luaL_error(
            lstate,
            b"invalid top\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    let mut bot: linenr_T = luaL_checkinteger(lstate, 3 as ::core::ffi::c_int)
        as linenr_T;
    if top > bot {
        return luaL_error(
            lstate,
            b"invalid bot\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    foldUpdate(win, top, bot);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_with(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut win: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut log_level: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    (lua_type(L, 1 as ::core::ffi::c_int) == 5 as ::core::ffi::c_int
        || luaL_argerror(
            L,
            1 as ::core::ffi::c_int,
            b"table expected\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0) as ::core::ffi::c_int;
    lua_pushnil(L);
    while lua_next(L, 1 as ::core::ffi::c_int) != 0 {
        if lua_type(L, -2 as ::core::ffi::c_int) == LUA_TSTRING {
            let mut k: *const ::core::ffi::c_char = lua_tolstring(
                L,
                -2 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<size_t>(),
            );
            let mut v: bool = lua_toboolean(L, -1 as ::core::ffi::c_int) != 0;
            if strequal(b"buf\0".as_ptr() as *const ::core::ffi::c_char, k) {
                buf = map_get_int_ptr_t(
                    &raw mut buffer_handles,
                    luaL_checkinteger(L, -1 as ::core::ffi::c_int) as ::core::ffi::c_int,
                ) as *mut buf_T;
            } else if strequal(b"win\0".as_ptr() as *const ::core::ffi::c_char, k) {
                win = map_get_int_ptr_t(
                    &raw mut window_handles,
                    luaL_checkinteger(L, -1 as ::core::ffi::c_int) as ::core::ffi::c_int,
                ) as *mut win_T;
            } else if strequal(
                b"log_level\0".as_ptr() as *const ::core::ffi::c_char,
                k,
            ) {
                log_level = luaL_checkinteger(L, -1 as ::core::ffi::c_int)
                    as ::core::ffi::c_int;
            } else {
                if strequal(b"sandbox\0".as_ptr() as *const ::core::ffi::c_char, k)
                    as ::core::ffi::c_int != 0 && v as ::core::ffi::c_int != 0
                {
                    flags |= CMOD_SANDBOX as ::core::ffi::c_int;
                }
                if strequal(b"silent\0".as_ptr() as *const ::core::ffi::c_char, k)
                    as ::core::ffi::c_int != 0 && v as ::core::ffi::c_int != 0
                {
                    flags |= CMOD_SILENT as ::core::ffi::c_int;
                }
                if strequal(b"emsg_silent\0".as_ptr() as *const ::core::ffi::c_char, k)
                    as ::core::ffi::c_int != 0 && v as ::core::ffi::c_int != 0
                {
                    flags |= CMOD_ERRSILENT as ::core::ffi::c_int;
                }
                if strequal(b"unsilent\0".as_ptr() as *const ::core::ffi::c_char, k)
                    as ::core::ffi::c_int != 0 && v as ::core::ffi::c_int != 0
                {
                    flags |= CMOD_UNSILENT as ::core::ffi::c_int;
                }
                if strequal(b"noautocmd\0".as_ptr() as *const ::core::ffi::c_char, k)
                    as ::core::ffi::c_int != 0 && v as ::core::ffi::c_int != 0
                {
                    flags |= CMOD_NOAUTOCMD as ::core::ffi::c_int;
                }
                if strequal(b"hide\0".as_ptr() as *const ::core::ffi::c_char, k)
                    as ::core::ffi::c_int != 0 && v as ::core::ffi::c_int != 0
                {
                    flags |= CMOD_HIDE as ::core::ffi::c_int;
                }
                if strequal(b"keepalt\0".as_ptr() as *const ::core::ffi::c_char, k)
                    as ::core::ffi::c_int != 0 && v as ::core::ffi::c_int != 0
                {
                    flags |= CMOD_KEEPALT as ::core::ffi::c_int;
                }
                if strequal(b"keepmarks\0".as_ptr() as *const ::core::ffi::c_char, k)
                    as ::core::ffi::c_int != 0 && v as ::core::ffi::c_int != 0
                {
                    flags |= CMOD_KEEPMARKS as ::core::ffi::c_int;
                }
                if strequal(b"keepjumps\0".as_ptr() as *const ::core::ffi::c_char, k)
                    as ::core::ffi::c_int != 0 && v as ::core::ffi::c_int != 0
                {
                    flags |= CMOD_KEEPJUMPS as ::core::ffi::c_int;
                }
                if strequal(b"lockmarks\0".as_ptr() as *const ::core::ffi::c_char, k)
                    as ::core::ffi::c_int != 0 && v as ::core::ffi::c_int != 0
                {
                    flags |= CMOD_LOCKMARKS as ::core::ffi::c_int;
                }
                if strequal(b"keeppatterns\0".as_ptr() as *const ::core::ffi::c_char, k)
                    as ::core::ffi::c_int != 0 && v as ::core::ffi::c_int != 0
                {
                    flags |= CMOD_KEEPPATTERNS as ::core::ffi::c_int;
                }
            }
        }
        lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    }
    let mut status: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut rets: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if flags & CMOD_ERRSILENT as ::core::ffi::c_int != 0 {
        flags |= CMOD_SILENT as ::core::ffi::c_int;
    }
    let save_min_log_level: ::core::ffi::c_int = g_min_log_level;
    if log_level >= 0 as ::core::ffi::c_int {
        g_min_log_level = log_level;
    }
    let mut save_cmdmod: cmdmod_T = cmdmod;
    memset(
        &raw mut cmdmod as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cmdmod_T>(),
    );
    cmdmod.cmod_flags = flags;
    apply_cmdmod(&raw mut cmdmod);
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    let mut aco: aco_save_T = aco_save_T {
        use_aucmd_win_idx: 0,
        save_curwin_handle: 0,
        new_curwin_handle: 0,
        save_prevwin_handle: 0,
        new_curbuf: bufref_T {
            br_buf: ::core::ptr::null_mut::<buf_T>(),
            br_fnum: 0,
            br_buf_free_count: 0,
        },
        tp_localdir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        globaldir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        save_VIsual_active: false,
        save_prompt_insert: 0,
    };
    let mut win_execute_args: win_execute_T = win_execute_T {
        wp: ::core::ptr::null_mut::<win_T>(),
        curpos: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        cwd: [0; 4096],
        cwd_status: 0,
        apply_acd: false,
        save_sfname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        switchwin: switchwin_T {
            sw_curwin: ::core::ptr::null_mut::<win_T>(),
            sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
            sw_same_win: false,
            sw_visual_active: false,
        },
    };
    's_376: {
        if !win.is_null() {
            let mut tabpage: *mut tabpage_T = win_find_tabpage(win);
            if !win_execute_before(&raw mut win_execute_args, win, tabpage) {
                break 's_376;
            }
        } else if !buf.is_null() {
            aucmd_prepbuf(&raw mut aco, buf);
        }
        let mut s: ::core::ffi::c_int = lua_gettop(L);
        lua_pushvalue(L, 2 as ::core::ffi::c_int);
        status = lua_pcall(
            L,
            0 as ::core::ffi::c_int,
            -1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        rets = lua_gettop(L) - s;
        if !win.is_null() {
            win_execute_after(&raw mut win_execute_args);
        } else if !buf.is_null() {
            aucmd_restbuf(&raw mut aco);
        }
    }
    try_leave(&raw mut tstate, &raw mut err);
    undo_cmdmod(&raw mut cmdmod);
    cmdmod = save_cmdmod;
    if log_level >= 0 as ::core::ffi::c_int {
        g_min_log_level = save_min_log_level;
    }
    if status != 0 {
        return lua_error(L)
    } else if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        nlua_push_errstr(L, b"%s\0".as_ptr() as *const ::core::ffi::c_char, err.msg);
        api_clear_error(&raw mut err);
        return lua_error(L);
    }
    return rets;
}
unsafe extern "C" fn nlua_state_add_internal(lstate: *mut lua_State) {
    lua_pushcclosure(
        lstate,
        Some(nlua_getvar as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"_getvar\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_setvar as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"_setvar\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_foldupdate as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"_foldupdate\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_with as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"_with_c\0".as_ptr() as *const ::core::ffi::c_char,
    );
}
#[no_mangle]
pub unsafe extern "C" fn nlua_state_add_stdlib(
    lstate: *mut lua_State,
    mut is_thread: bool,
) {
    if !is_thread {
        lua_pushcclosure(
            lstate,
            Some(
                nlua_stricmp
                    as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"stricmp\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushcclosure(
            lstate,
            Some(
                nlua_str_utfindex
                    as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"_str_utfindex\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushcclosure(
            lstate,
            Some(
                nlua_str_byteindex
                    as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"_str_byteindex\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushcclosure(
            lstate,
            Some(
                nlua_str_utf_pos
                    as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"str_utf_pos\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushcclosure(
            lstate,
            Some(
                nlua_str_utf_start
                    as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"str_utf_start\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushcclosure(
            lstate,
            Some(
                nlua_str_utf_end
                    as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"str_utf_end\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushcclosure(
            lstate,
            Some(
                nlua_regex as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"regex\0".as_ptr() as *const ::core::ffi::c_char,
        );
        luaL_newmetatable(
            lstate,
            b"nvim_regex\0".as_ptr() as *const ::core::ffi::c_char,
        );
        luaL_register(
            lstate,
            ::core::ptr::null::<::core::ffi::c_char>(),
            &raw mut regex_meta as *mut luaL_Reg,
        );
        lua_pushvalue(lstate, -1 as ::core::ffi::c_int);
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"__index\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        luaopen_spell(lstate);
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"spell\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushcclosure(
            lstate,
            Some(
                nlua_iconv as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"iconv\0".as_ptr() as *const ::core::ffi::c_char,
        );
        luaopen_base64(lstate);
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"base64\0".as_ptr() as *const ::core::ffi::c_char,
        );
        nlua_state_add_internal(lstate);
    }
    luaopen_mpack(lstate);
    lua_pushvalue(lstate, -1 as ::core::ffi::c_int);
    lua_setfield(
        lstate,
        -3 as ::core::ffi::c_int,
        b"mpack\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_getfield(
        lstate,
        LUA_GLOBALSINDEX,
        b"package\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_getfield(
        lstate,
        -1 as ::core::ffi::c_int,
        b"loaded\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushvalue(lstate, -3 as ::core::ffi::c_int);
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"mpack\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_settop(lstate, -3 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    extern "C" {
        #[link_name = "luaopen_lpeg"]
        fn luaopen_lpeg_0(_: *mut lua_State) -> ::core::ffi::c_int;
    }
    luaopen_lpeg_0(lstate);
    lua_pushvalue(lstate, -1 as ::core::ffi::c_int);
    lua_setfield(
        lstate,
        -4 as ::core::ffi::c_int,
        b"lpeg\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_getfield(
        lstate,
        LUA_GLOBALSINDEX,
        b"package\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_getfield(
        lstate,
        -1 as ::core::ffi::c_int,
        b"loaded\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushvalue(lstate, -3 as ::core::ffi::c_int);
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"lpeg\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_settop(lstate, -4 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    lua_pushcclosure(
        lstate,
        Some(
            nlua_xdl_diff as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"diff\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_cjson_new(lstate);
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"json\0".as_ptr() as *const ::core::ffi::c_char,
    );
}
#[no_mangle]
pub unsafe extern "C" fn nlua_push_errstr(
    mut L: *mut lua_State,
    mut fmt: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) {
    let mut argp: ::core::ffi::VaListImpl;
    argp = c2rust_args.clone();
    luaL_where(L, 1 as ::core::ffi::c_int);
    lua_pushvfstring(L, fmt, argp.as_va_list());
    lua_concat(L, 2 as ::core::ffi::c_int);
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
