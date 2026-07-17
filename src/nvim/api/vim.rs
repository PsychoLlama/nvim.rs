extern "C" {
    pub type terminal;
    pub type regprog;
    pub type undo_object;
    pub type qf_info_S;
    pub type multiqueue;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn labs(__x: ::core::ffi::c_long) -> ::core::ffi::c_long;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memcmp(
        __s1: *const ::core::ffi::c_void,
        __s2: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    static mut arena_alloc_count: size_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn memchrsub(
        data: *mut ::core::ffi::c_void,
        c: ::core::ffi::c_char,
        x: ::core::ffi::c_char,
        len: size_t,
    );
    fn strequal(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char) -> bool;
    fn arena_alloc(arena: *mut Arena, size: size_t, align: bool) -> *mut ::core::ffi::c_void;
    fn arena_strdup(arena: *mut Arena, str: *const ::core::ffi::c_char)
        -> *mut ::core::ffi::c_char;
    fn api_buf_ensure_loaded(buf: Buffer, err: *mut Error) -> *mut buf_T;
    fn nvim_buf_del_keymap(
        channel_id: uint64_t,
        buf: Buffer,
        mode: String_0,
        lhs: String_0,
        err: *mut Error,
    );
    fn buffer_get_line(
        buffer: Buffer,
        index: Integer,
        arena: *mut Arena,
        err: *mut Error,
    ) -> String_0;
    fn buffer_set_line(
        buffer: Buffer,
        index: Integer,
        line: String_0,
        arena: *mut Arena,
        err: *mut Error,
    );
    fn buffer_del_line(buffer: Buffer, index: Integer, arena: *mut Arena, err: *mut Error);
    fn vim_to_object(obj: *mut typval_T, arena: *mut Arena, reuse_strdata: bool) -> Object;
    fn logmsg(
        log_level: ::core::ffi::c_int,
        context: *const ::core::ffi::c_char,
        func_name: *const ::core::ffi::c_char,
        line_num: ::core::ffi::c_int,
        eol: bool,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> bool;
    fn mh_get_uint64_t(set: *mut Set_uint64_t, key: uint64_t) -> uint32_t;
    fn api_err_invalid(
        err: *mut Error,
        name: *const ::core::ffi::c_char,
        val_s: *const ::core::ffi::c_char,
        val_n: int64_t,
        quote_val: bool,
    );
    fn api_err_exp(
        err: *mut Error,
        name: *const ::core::ffi::c_char,
        expected: *const ::core::ffi::c_char,
        actual: *const ::core::ffi::c_char,
    );
    fn api_err_required(err: *mut Error, name: *const ::core::ffi::c_char);
    fn try_enter(tstate: *mut TryState);
    fn try_leave(tstate: *const TryState, err: *mut Error);
    fn dict_get_value(
        dict: *mut dict_T,
        key: String_0,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
    fn dict_set_var(
        dict: *mut dict_T,
        key: String_0,
        value: Object,
        del: bool,
        retval: bool,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
    fn find_buffer_by_handle(buffer: Buffer, err: *mut Error) -> *mut buf_T;
    fn find_window_by_handle(window: Window, err: *mut Error) -> *mut win_T;
    fn find_tab_by_handle(tabpage: Tabpage, err: *mut Error) -> *mut tabpage_T;
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    fn string_to_array(input: String_0, crlf: bool, arena: *mut Arena) -> Array;
    fn arena_array(arena: *mut Arena, max_size: size_t) -> Array;
    fn arena_dict(arena: *mut Arena, max_size: size_t) -> Dict;
    fn arena_string(arena: *mut Arena, str: String_0) -> String_0;
    fn arena_take_arraybuilder(arena: *mut Arena, arr: *mut ArrayBuilder) -> Array;
    fn api_metadata() -> Object;
    fn copy_string(str: String_0, arena: *mut Arena) -> String_0;
    fn copy_array(array: Array, arena: *mut Arena) -> Array;
    fn copy_dict(dict: Dict, arena: *mut Arena) -> Dict;
    fn copy_object(obj: Object, arena: *mut Arena) -> Object;
    fn api_set_error(err: *mut Error, errType: ErrorType, format: *const ::core::ffi::c_char, ...);
    fn api_typename(t: ObjectType) -> *mut ::core::ffi::c_char;
    fn parse_hl_msg(chunks: Array, is_err: bool, err: *mut Error) -> HlMessage;
    fn set_mark(
        buf: *mut buf_T,
        name: String_0,
        line: Integer,
        col: Integer,
        err: *mut Error,
    ) -> bool;
    fn get_default_stl_hl(
        wp: *mut win_T,
        use_winbar: bool,
        stc_hl_id: ::core::ffi::c_int,
    ) -> *const ::core::ffi::c_char;
    fn api_set_sctx(channel_id: uint64_t) -> sctx_T;
    fn apply_autocmds(
        event: event_T,
        fname: *mut ::core::ffi::c_char,
        fname_io: *mut ::core::ffi::c_char,
        force: bool,
        buf: *mut buf_T,
    ) -> bool;
    fn block_autocmds();
    fn unblock_autocmds();
    fn may_trigger_vim_suspend_resume(suspend: bool);
    fn set_bufref(bufref: *mut bufref_T, buf: *mut buf_T);
    fn bufref_valid(bufref: *mut bufref_T) -> bool;
    fn buf_close_terminal(buf: *mut buf_T);
    fn do_buffer(
        action: ::core::ffi::c_int,
        start: ::core::ffi::c_int,
        dir: ::core::ffi::c_int,
        count: ::core::ffi::c_int,
        forceit: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn buflist_new(
        ffname_arg: *mut ::core::ffi::c_char,
        sfname_arg: *mut ::core::ffi::c_char,
        lnum: linenr_T,
        flags: ::core::ffi::c_int,
    ) -> *mut buf_T;
    fn buflist_nr2name(
        n: ::core::ffi::c_int,
        fullname: ::core::ffi::c_int,
        helptail: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn read_buffer_into(buf: *mut buf_T, start: linenr_T, end: linenr_T, sb: *mut StringBuilder);
    static mut channels: Map_uint64_t_ptr_t;
    fn channel_alloc(type_0: ChannelStreamType) -> *mut Channel;
    fn channel_incref(chan: *mut Channel);
    fn channel_decref(chan: *mut Channel);
    fn channel_send(
        id: uint64_t,
        data: *mut ::core::ffi::c_char,
        len: size_t,
        data_owned: bool,
        error: *mut *const ::core::ffi::c_char,
    ) -> size_t;
    fn channel_info(id: uint64_t, arena: *mut Arena) -> Dict;
    fn channel_all_info(arena: *mut Arena) -> Array;
    static mut kCtxAll: ::core::ffi::c_int;
    fn get_cursor_rel_lnum(wp: *mut win_T, lnum: linenr_T) -> linenr_T;
    fn ctx_free(ctx: *mut Context);
    fn ctx_save(ctx: *mut Context, flags: ::core::ffi::c_int);
    fn ctx_restore(ctx: *mut Context, flags: ::core::ffi::c_int) -> bool;
    fn ctx_to_dict(ctx: *mut Context, arena: *mut Arena) -> Dict;
    fn ctx_from_dict(dict: Dict, ctx: *mut Context, err: *mut Error) -> ::core::ffi::c_int;
    fn decor_redraw_signs(
        wp: *mut win_T,
        buf: *mut buf_T,
        row: ::core::ffi::c_int,
        sattrs: *mut SignTextAttrs,
        line_id: *mut ::core::ffi::c_int,
        cul_id: *mut ::core::ffi::c_int,
        num_id: *mut ::core::ffi::c_int,
    );
    fn use_cursor_line_highlight(wp: *mut win_T, lnum: linenr_T) -> bool;
    fn msg_id_exists(id: int64_t) -> bool;
    fn msg_multihl(
        id: Object,
        hl_msg: HlMessage,
        kind: *const ::core::ffi::c_char,
        history: bool,
        err: bool,
        msg_data: *mut MessageData,
        needs_msg_clear: *mut bool,
    ) -> Object;
    fn hl_msg_free(hl_msg: HlMessage);
    fn do_autocmd_progress(msg_id: Object, msg: HlMessage, msg_data: *mut MessageData);
    fn verbose_enter();
    fn verbose_leave();
    fn verbose_stop();
    fn tv_dict_find(
        d: *const dict_T,
        key: *const ::core::ffi::c_char,
        len: ptrdiff_t,
    ) -> *mut dictitem_T;
    fn get_globvar_dict() -> *mut dict_T;
    fn get_vimvar_dict() -> *mut dict_T;
    fn set_vim_var_nr(idx: VimVarIndex, val: varnumber_T);
    fn changedir_func(new_dir: *mut ::core::ffi::c_char, scope: CdScope) -> bool;
    fn exec_normal(was_typed: bool, use_vpeekc: bool);
    fn aborting() -> bool;
    fn fold_info(win: *mut win_T, lnum: linenr_T) -> foldinfo_T;
    fn ins_typebuf(
        str: *mut ::core::ffi::c_char,
        noremap: ::core::ffi::c_int,
        offset: ::core::ffi::c_int,
        nottyped: bool,
        silent: bool,
    ) -> ::core::ffi::c_int;
    fn paste_store(channel_id: uint64_t, state: TriState, str: String_0, crlf: bool);
    static mut default_grid: ScreenGrid;
    fn schar_cache_clear();
    fn schar_get(buf_out: *mut ::core::ffi::c_char, sc: schar_T) -> size_t;
    fn win_grid_alloc(wp: *mut win_T);
    fn get_win_by_grid_handle(handle: handle_T) -> *mut win_T;
    static mut g_stats: nvim_stats_s;
    static mut Columns: ::core::ffi::c_int;
    static mut msg_scroll: ::core::ffi::c_int;
    static mut msg_didany: bool;
    static mut did_emsg: ::core::ffi::c_int;
    static mut no_wait_return: ::core::ffi::c_int;
    static mut need_wait_return: bool;
    static mut vgetc_busy: ::core::ffi::c_int;
    static mut lines_left: ::core::ffi::c_int;
    static mut msg_no_more: bool;
    static mut current_sctx: sctx_T;
    static mut firstwin: *mut win_T;
    static mut curwin: *mut win_T;
    static mut first_tabpage: *mut tabpage_T;
    static mut curtab: *mut tabpage_T;
    static mut redraw_tabline: bool;
    static mut firstbuf: *mut buf_T;
    static mut curbuf: *mut buf_T;
    static mut textlock: ::core::ffi::c_int;
    static mut VIsual_active: bool;
    static mut cmdpreview: bool;
    static mut msg_silent: ::core::ffi::c_int;
    static mut RedrawingDisabled: ::core::ffi::c_int;
    static mut typebuf: typebuf_T;
    static mut ex_normal_busy: ::core::ffi::c_int;
    static mut must_redraw: ::core::ffi::c_int;
    static mut cmdwin_buf: *mut buf_T;
    static mut typebuf_was_filled: bool;
    static mut ns_hl_global: NS;
    static mut ns_hl_fast: NS;
    static mut p_cpo: *mut ::core::ffi::c_char;
    static mut p_lz: ::core::ffi::c_int;
    fn highlight_use_hlstate() -> bool;
    fn ns_hl_def(
        ns_id: NS,
        hl_id: ::core::ffi::c_int,
        attrs: HlAttrs,
        link_id: ::core::ffi::c_int,
        dict: *mut KeyDict_highlight,
    );
    fn hl_check_ns() -> bool;
    fn win_check_ns_hl(wp: *mut win_T) -> bool;
    fn hl_ns_get_attrs(
        ns_id: ::core::ffi::c_int,
        hl_id: ::core::ffi::c_int,
        optional: *mut bool,
        attrs: *mut HlAttrs,
    ) -> bool;
    fn hl_get_attr_by_id(
        attr_id: Integer,
        rgb: Boolean,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Dict;
    fn dict2hlattrs(
        dict: *mut KeyDict_highlight,
        use_rgb: bool,
        link_id: *mut ::core::ffi::c_int,
        base: *mut HlAttrs,
        err: *mut Error,
    ) -> HlAttrs;
    fn hl_inspect(attr: ::core::ffi::c_int, arena: *mut Arena) -> Array;
    fn ns_get_hl_defs(
        ns_id: NS,
        opts: *mut KeyDict_get_highlight,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Dict;
    fn syn_id2name(id: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn syn_check_group(name: *const ::core::ffi::c_char, len: size_t) -> ::core::ffi::c_int;
    fn name_to_color(name: *const ::core::ffi::c_char, idx: *mut ::core::ffi::c_int) -> RgbValue;
    static mut color_name_table: [color_name_table_T; 708];
    fn get_cot_flags() -> ::core::ffi::c_uint;
    fn name_to_mod_mask(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn replace_termcodes(
        from: *const ::core::ffi::c_char,
        from_len: size_t,
        bufp: *mut *mut ::core::ffi::c_char,
        sid_arg: scid_T,
        flags: ::core::ffi::c_int,
        did_simplify: *mut bool,
        cpo_val: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn vim_strsave_escape_ks(p: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn nlua_get_global_ref_count() -> ::core::ffi::c_int;
    fn api_free_luaref(ref_0: LuaRef);
    fn nlua_exec(
        str: String_0,
        chunkname: *const ::core::ffi::c_char,
        args: Array,
        mode: LuaRetMode,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
    fn nlua_call_ref(
        ref_0: LuaRef,
        name: *const ::core::ffi::c_char,
        args: Array,
        mode: LuaRetMode,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
    fn nlua_is_deferred_safe() -> bool;
    static mut tslua_query_parse_count: uint64_t;
    fn modify_keymap(
        channel_id: uint64_t,
        buffer: Buffer,
        is_unmap: bool,
        mode: String_0,
        lhs: String_0,
        rhs: String_0,
        opts: *mut KeyDict_keymap,
        err: *mut Error,
    );
    fn keymap_array(mode: String_0, buf: *mut buf_T, arena: *mut Arena) -> Array;
    fn mark_get_global(resolve: bool, name: ::core::ffi::c_int) -> *mut xfmark_T;
    fn xpopcount(x: uint64_t) -> ::core::ffi::c_uint;
    fn mb_string2cells(str: *const ::core::ffi::c_char) -> size_t;
    fn utfc_ptr2schar(p: *const ::core::ffi::c_char, firstc: *mut ::core::ffi::c_int) -> schar_T;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn ml_open(buf: *mut buf_T) -> ::core::ffi::c_int;
    fn update_topline(wp: *mut win_T);
    fn changed_window_setting(wp: *mut win_T);
    fn validate_cursor(wp: *mut win_T);
    fn win_col_off(wp: *mut win_T) -> ::core::ffi::c_int;
    fn rpc_set_client_info(id: uint64_t, info: Dict);
    fn unpack(
        data: *const ::core::ffi::c_char,
        size: size_t,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
    fn reset_VIsual_and_resel();
    fn set_option_direct_for(
        opt_idx: OptIndex,
        value: OptVal,
        opt_flags: ::core::ffi::c_int,
        set_sid: scid_T,
        scope: OptScope,
        from: *mut ::core::ffi::c_void,
    );
    fn buf_copy_options(buf: *mut buf_T, flags: ::core::ffi::c_int);
    fn check_stl_option(s: *mut ::core::ffi::c_char) -> *const ::core::ffi::c_char;
    fn input_enqueue_raw(data: *const ::core::ffi::c_char, size: size_t);
    fn input_enqueue(chan_id: uint64_t, keys: String_0) -> size_t;
    fn input_enqueue_mouse(
        code: ::core::ffi::c_int,
        modifier: uint8_t,
        grid: ::core::ffi::c_int,
        row: ::core::ffi::c_int,
        col: ::core::ffi::c_int,
    );
    fn input_blocking() -> bool;
    fn os_proc_children(
        ppid: ::core::ffi::c_int,
        proc_list: *mut *mut ::core::ffi::c_int,
        proc_count: *mut size_t,
    ) -> ::core::ffi::c_int;
    fn pum_set_info(selected: ::core::ffi::c_int, info: *mut ::core::ffi::c_char) -> *mut win_T;
    fn pum_ext_select_item(item: ::core::ffi::c_int, insert: bool, finish: bool);
    static mut pum_grid: ScreenGrid;
    fn do_put(
        regname: ::core::ffi::c_int,
        reg: *mut yankreg_T,
        dir: ::core::ffi::c_int,
        count: ::core::ffi::c_int,
        flags: ::core::ffi::c_int,
    );
    fn prepare_yankreg_from_object(reg: *mut yankreg_T, regtype: String_0, lines: size_t) -> bool;
    fn finish_yankreg_from_object(reg: *mut yankreg_T, clipboard_adjust: bool);
    fn runtime_inspect(arena: *mut Arena) -> Array;
    fn runtime_get_named(lua: bool, pat: Array, all: bool, arena: *mut Arena) -> Array;
    fn do_in_runtimepath(
        name: *mut ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
        callback: DoInRuntimepathCB,
        cookie: *mut ::core::ffi::c_void,
    ) -> ::core::ffi::c_int;
    fn get_lib_dir() -> *mut ::core::ffi::c_char;
    fn do_source(
        fname: *mut ::core::ffi::c_char,
        check_other: bool,
        is_vimrc: ::core::ffi::c_int,
        ret_sid: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn script_autoload(name: *const ::core::ffi::c_char, name_len: size_t, reload: bool) -> bool;
    static e_cmdwin: [::core::ffi::c_char; 0];
    static e_invchan: [::core::ffi::c_char; 0];
    fn get_mode(buf: *mut ::core::ffi::c_char);
    fn update_screen() -> ::core::ffi::c_int;
    fn setcursor_mayforce(wp: *mut win_T, force: bool);
    fn redraw_later(wp: *mut win_T, type_0: ::core::ffi::c_int);
    fn redraw_all_later(type_0: ::core::ffi::c_int);
    fn redraw_buf_later(buf: *mut buf_T, type_0: ::core::ffi::c_int);
    fn redraw_buf_range_later(buf: *mut buf_T, first: linenr_T, last: linenr_T);
    fn win_update_cursorline(wp: *mut win_T, foldinfo: *mut foldinfo_T);
    fn win_redr_status(wp: *mut win_T);
    fn win_redr_winbar(wp: *mut win_T);
    fn fillchar_status(group: *mut hlf_T, wp: *mut win_T) -> schar_T;
    fn draw_tabline();
    fn build_stl_str_hl(
        wp: *mut win_T,
        out: *mut ::core::ffi::c_char,
        outlen: size_t,
        fmt: *mut ::core::ffi::c_char,
        opt_idx: OptIndex,
        opt_scope: ::core::ffi::c_int,
        fillchar: schar_T,
        maxwidth: ::core::ffi::c_int,
        hltab: *mut *mut stl_hlrec_t,
        hltab_len: *mut size_t,
        tabtab: *mut *mut StlClickRecord,
        stcp: *mut statuscol_T,
    ) -> ::core::ffi::c_int;
    fn ui_flush();
    fn ui_array(arena: *mut Arena) -> Array;
    fn terminal_alloc(buf: *mut buf_T, opts: TerminalOptions) -> *mut Terminal;
    fn terminal_open(termpp: *mut *mut Terminal, buf: *mut buf_T);
    fn terminal_check_size(term: *mut Terminal);
    fn terminal_destroy(termpp: *mut *mut Terminal);
    fn terminal_set_streamed_paste(term: *mut Terminal, streamed: bool);
    fn terminal_buf(term: *const Terminal) -> Buffer;
    fn terminal_running(term: *const Terminal) -> bool;
    fn ui_call_screenshot(path: String_0);
    fn goto_tabpage_tp(
        tp: *mut tabpage_T,
        trigger_enter_autocmds: bool,
        trigger_leave_autocmds: bool,
    );
    fn goto_tabpage_win(tp: *mut tabpage_T, wp: *mut win_T);
    fn win_find_tabpage(win: *mut win_T) -> *mut tabpage_T;
    fn global_stl_height() -> ::core::ffi::c_int;
}
pub type __uid_t = ::core::ffi::c_uint;
pub type __gid_t = ::core::ffi::c_uint;
pub type __time_t = ::core::ffi::c_long;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type ptrdiff_t = isize;
pub type size_t = usize;
pub type ssize_t = isize;
pub type gid_t = __gid_t;
pub type uid_t = __uid_t;
pub type time_t = __time_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __pthread_internal_list {
    pub __prev: *mut __pthread_internal_list,
    pub __next: *mut __pthread_internal_list,
}
pub type __pthread_list_t = __pthread_internal_list;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __pthread_mutex_s {
    pub __lock: ::core::ffi::c_int,
    pub __count: ::core::ffi::c_uint,
    pub __owner: ::core::ffi::c_int,
    pub __nusers: ::core::ffi::c_uint,
    pub __kind: ::core::ffi::c_int,
    pub __spins: ::core::ffi::c_short,
    pub __elision: ::core::ffi::c_short,
    pub __list: __pthread_list_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __pthread_rwlock_arch_t {
    pub __readers: ::core::ffi::c_uint,
    pub __writers: ::core::ffi::c_uint,
    pub __wrphase_futex: ::core::ffi::c_uint,
    pub __writers_futex: ::core::ffi::c_uint,
    pub __pad3: ::core::ffi::c_uint,
    pub __pad4: ::core::ffi::c_uint,
    pub __cur_writer: ::core::ffi::c_int,
    pub __shared: ::core::ffi::c_int,
    pub __rwelision: ::core::ffi::c_schar,
    pub __pad1: [::core::ffi::c_uchar; 7],
    pub __pad2: ::core::ffi::c_ulong,
    pub __flags: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union pthread_mutex_t {
    pub __data: __pthread_mutex_s,
    pub __size: [::core::ffi::c_char; 40],
    pub __align: ::core::ffi::c_long,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union pthread_rwlock_t {
    pub __data: __pthread_rwlock_arch_t,
    pub __size: [::core::ffi::c_char; 56],
    pub __align: ::core::ffi::c_long,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct consumed_blk {
    pub prev: *mut consumed_blk,
}
pub type ArenaMem = *mut consumed_blk;
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
    pub data: C2Rust_Unnamed,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed {
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
pub type NS = handle_T;
pub type proftime_T = uint64_t;
pub type TriState = ::core::ffi::c_int;
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
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
    pub b_wininfo: C2Rust_Unnamed_11,
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
    pub b_signcols: C2Rust_Unnamed_3,
    pub terminal: *mut Terminal,
    pub additional_data: *mut AdditionalData,
    pub b_mapped_ctrl_c: ::core::ffi::c_int,
    pub b_marktree: [MarkTree; 1],
    pub b_extmark_ns: [Map_uint32_t_uint32_t; 1],
    pub b_prev_line_count: ::core::ffi::c_int,
    pub update_channels: C2Rust_Unnamed_1,
    pub update_callbacks: C2Rust_Unnamed_0,
    pub update_need_codepoints: bool,
    pub deleted_bytes: size_t,
    pub deleted_bytes2: size_t,
    pub deleted_codepoints: size_t,
    pub deleted_codeunits: size_t,
    pub flush_count: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_0 {
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
pub struct C2Rust_Unnamed_1 {
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
    pub data: C2Rust_Unnamed_2,
    pub next: *mut DecorVirtText,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_2 {
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
pub struct C2Rust_Unnamed_3 {
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
    pub sst_union: C2Rust_Unnamed_4,
    pub sst_next_flags: ::core::ffi::c_int,
    pub sst_stacksize: ::core::ffi::c_int,
    pub sst_next_list: *mut int16_t,
    pub sst_tick: disptick_T,
    pub sst_change_lnum: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_4 {
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
    pub data: C2Rust_Unnamed_5,
    pub type_0: CallbackType,
}
pub type CallbackType = ::core::ffi::c_uint;
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_5 {
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
    pub fc_fixvar: [C2Rust_Unnamed_6; 12],
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
pub struct C2Rust_Unnamed_6 {
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
    pub uh_next: C2Rust_Unnamed_10,
    pub uh_prev: C2Rust_Unnamed_9,
    pub uh_alt_next: C2Rust_Unnamed_8,
    pub uh_alt_prev: C2Rust_Unnamed_7,
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
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_10 {
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
pub struct C2Rust_Unnamed_11 {
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
    pub type_0: C2Rust_Unnamed_12,
    pub tabnr: ::core::ffi::c_int,
    pub func: *mut ::core::ffi::c_char,
}
pub type C2Rust_Unnamed_12 = ::core::ffi::c_uint;
pub const kStlClickFuncRun: C2Rust_Unnamed_12 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_12 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_12 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_12 = 0;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct loop_0 {
    pub uv: uv_loop_t,
    pub events: *mut MultiQueue,
    pub thread_events: *mut MultiQueue,
    pub fast_events: *mut MultiQueue,
    pub children: C2Rust_Unnamed_21,
    pub children_watcher: uv_signal_t,
    pub children_kill_timer: uv_timer_t,
    pub poll_timer: uv_timer_t,
    pub exit_delay_timer: uv_timer_t,
    pub async_0: uv_async_t,
    pub mutex: uv_mutex_t,
    pub recursive: ::core::ffi::c_int,
    pub closing: bool,
}
pub type uv_mutex_t = pthread_mutex_t;
pub type uv_async_t = uv_async_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_async_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_18,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub async_cb: uv_async_cb,
    pub queue: uv__queue,
    pub pending: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv__queue {
    pub next: *mut uv__queue,
    pub prev: *mut uv__queue,
}
pub type uv_async_cb = Option<unsafe extern "C" fn(*mut uv_async_t) -> ()>;
pub type uv_handle_t = uv_handle_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_handle_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_13,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_13 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_close_cb = Option<unsafe extern "C" fn(*mut uv_handle_t) -> ()>;
pub type uv_handle_type = ::core::ffi::c_uint;
pub const UV_HANDLE_TYPE_MAX: uv_handle_type = 18;
pub const UV_FILE: uv_handle_type = 17;
pub const UV_SIGNAL: uv_handle_type = 16;
pub const UV_UDP: uv_handle_type = 15;
pub const UV_TTY: uv_handle_type = 14;
pub const UV_TIMER: uv_handle_type = 13;
pub const UV_TCP: uv_handle_type = 12;
pub const UV_STREAM: uv_handle_type = 11;
pub const UV_PROCESS: uv_handle_type = 10;
pub const UV_PREPARE: uv_handle_type = 9;
pub const UV_POLL: uv_handle_type = 8;
pub const UV_NAMED_PIPE: uv_handle_type = 7;
pub const UV_IDLE: uv_handle_type = 6;
pub const UV_HANDLE: uv_handle_type = 5;
pub const UV_FS_POLL: uv_handle_type = 4;
pub const UV_FS_EVENT: uv_handle_type = 3;
pub const UV_CHECK: uv_handle_type = 2;
pub const UV_ASYNC: uv_handle_type = 1;
pub const UV_UNKNOWN_HANDLE: uv_handle_type = 0;
pub type uv_loop_t = uv_loop_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_loop_s {
    pub data: *mut ::core::ffi::c_void,
    pub active_handles: ::core::ffi::c_uint,
    pub handle_queue: uv__queue,
    pub active_reqs: C2Rust_Unnamed_17,
    pub internal_fields: *mut ::core::ffi::c_void,
    pub stop_flag: ::core::ffi::c_uint,
    pub flags: ::core::ffi::c_ulong,
    pub backend_fd: ::core::ffi::c_int,
    pub pending_queue: uv__queue,
    pub watcher_queue: uv__queue,
    pub watchers: *mut *mut uv__io_t,
    pub nwatchers: ::core::ffi::c_uint,
    pub nfds: ::core::ffi::c_uint,
    pub wq: uv__queue,
    pub wq_mutex: uv_mutex_t,
    pub wq_async: uv_async_t,
    pub cloexec_lock: uv_rwlock_t,
    pub closing_handles: *mut uv_handle_t,
    pub process_handles: uv__queue,
    pub prepare_handles: uv__queue,
    pub check_handles: uv__queue,
    pub idle_handles: uv__queue,
    pub async_handles: uv__queue,
    pub async_unused: Option<unsafe extern "C" fn() -> ()>,
    pub async_io_watcher: uv__io_t,
    pub async_wfd: ::core::ffi::c_int,
    pub timer_heap: C2Rust_Unnamed_16,
    pub timer_counter: uint64_t,
    pub time: uint64_t,
    pub signal_pipefd: [::core::ffi::c_int; 2],
    pub signal_io_watcher: uv__io_t,
    pub child_watcher: uv_signal_t,
    pub emfile_fd: ::core::ffi::c_int,
    pub inotify_read_watcher: uv__io_t,
    pub inotify_watchers: *mut ::core::ffi::c_void,
    pub inotify_fd: ::core::ffi::c_int,
}
pub type uv__io_t = uv__io_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv__io_s {
    pub cb: uv__io_cb,
    pub pending_queue: uv__queue,
    pub watcher_queue: uv__queue,
    pub pevents: ::core::ffi::c_uint,
    pub events: ::core::ffi::c_uint,
    pub fd: ::core::ffi::c_int,
}
pub type uv__io_cb =
    Option<unsafe extern "C" fn(*mut uv_loop_s, *mut uv__io_s, ::core::ffi::c_uint) -> ()>;
pub type uv_signal_t = uv_signal_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_signal_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_15,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub signal_cb: uv_signal_cb,
    pub signum: ::core::ffi::c_int,
    pub tree_entry: C2Rust_Unnamed_14,
    pub caught_signals: ::core::ffi::c_uint,
    pub dispatched_signals: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_14 {
    pub rbe_left: *mut uv_signal_s,
    pub rbe_right: *mut uv_signal_s,
    pub rbe_parent: *mut uv_signal_s,
    pub rbe_color: ::core::ffi::c_int,
}
pub type uv_signal_cb = Option<unsafe extern "C" fn(*mut uv_signal_t, ::core::ffi::c_int) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_15 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_16 {
    pub min: *mut ::core::ffi::c_void,
    pub nelts: ::core::ffi::c_uint,
}
pub type uv_rwlock_t = pthread_rwlock_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_17 {
    pub unused: *mut ::core::ffi::c_void,
    pub count: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_18 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_timer_t = uv_timer_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_timer_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_20,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub timer_cb: uv_timer_cb,
    pub node: C2Rust_Unnamed_19,
    pub timeout: uint64_t,
    pub repeat: uint64_t,
    pub start_id: uint64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_19 {
    pub heap: [*mut ::core::ffi::c_void; 3],
    pub queue: uv__queue,
}
pub type uv_timer_cb = Option<unsafe extern "C" fn(*mut uv_timer_t) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_20 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_21 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut *mut Proc,
}
pub type Proc = proc;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct proc {
    pub type_0: ProcType,
    pub loop_0: *mut Loop,
    pub data: *mut ::core::ffi::c_void,
    pub pid: ::core::ffi::c_int,
    pub status: ::core::ffi::c_int,
    pub refcount: ::core::ffi::c_int,
    pub exit_signal: uint8_t,
    pub stopped_time: uint64_t,
    pub cwd: *const ::core::ffi::c_char,
    pub argv: *mut *mut ::core::ffi::c_char,
    pub exepath: *const ::core::ffi::c_char,
    pub env: *mut dict_T,
    pub in_0: Stream,
    pub out: RStream,
    pub err: RStream,
    pub cb: proc_exit_cb,
    pub state_cb: proc_state_cb,
    pub internal_exit_cb: internal_proc_cb,
    pub internal_close_cb: internal_proc_cb,
    pub closed: bool,
    pub detach: bool,
    pub overlapped: bool,
    pub fwd_err: bool,
    pub stdio_noinherit: bool,
    pub events: *mut MultiQueue,
}
pub type MultiQueue = multiqueue;
pub type internal_proc_cb = Option<unsafe extern "C" fn(*mut Proc) -> ()>;
pub type proc_state_cb =
    Option<unsafe extern "C" fn(*mut Proc, bool, *mut ::core::ffi::c_void) -> ()>;
pub type proc_exit_cb =
    Option<unsafe extern "C" fn(*mut Proc, ::core::ffi::c_int, *mut ::core::ffi::c_void) -> ()>;
pub type RStream = rstream;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct rstream {
    pub s: Stream,
    pub did_eof: bool,
    pub want_read: bool,
    pub pending_read: bool,
    pub paused_full: bool,
    pub buffer: *mut ::core::ffi::c_char,
    pub read_pos: *mut ::core::ffi::c_char,
    pub write_pos: *mut ::core::ffi::c_char,
    pub uvbuf: uv_buf_t,
    pub read_cb: stream_read_cb,
    pub num_bytes: size_t,
}
pub type stream_read_cb = Option<
    unsafe extern "C" fn(
        *mut RStream,
        *const ::core::ffi::c_char,
        size_t,
        *mut ::core::ffi::c_void,
        bool,
    ) -> size_t,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_buf_t {
    pub base: *mut ::core::ffi::c_char,
    pub len: size_t,
}
pub type Stream = stream;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stream {
    pub closed: bool,
    pub uv: C2Rust_Unnamed_23,
    pub uvstream: *mut uv_stream_t,
    pub fd: uv_file,
    pub fpos: int64_t,
    pub cb_data: *mut ::core::ffi::c_void,
    pub before_close_cb: stream_close_cb,
    pub close_cb: stream_close_cb,
    pub internal_close_cb: stream_close_cb,
    pub close_cb_data: *mut ::core::ffi::c_void,
    pub internal_data: *mut ::core::ffi::c_void,
    pub pending_reqs: size_t,
    pub events: *mut MultiQueue,
    pub write_cb: stream_write_cb,
    pub curmem: size_t,
    pub maxmem: size_t,
}
pub type stream_write_cb =
    Option<unsafe extern "C" fn(*mut Stream, *mut ::core::ffi::c_void, ::core::ffi::c_int) -> ()>;
pub type stream_close_cb =
    Option<unsafe extern "C" fn(*mut Stream, *mut ::core::ffi::c_void) -> ()>;
pub type uv_file = ::core::ffi::c_int;
pub type uv_stream_t = uv_stream_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_stream_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_22,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub write_queue_size: size_t,
    pub alloc_cb: uv_alloc_cb,
    pub read_cb: uv_read_cb,
    pub connect_req: *mut uv_connect_t,
    pub shutdown_req: *mut uv_shutdown_t,
    pub io_watcher: uv__io_t,
    pub write_queue: uv__queue,
    pub write_completed_queue: uv__queue,
    pub connection_cb: uv_connection_cb,
    pub delayed_error: ::core::ffi::c_int,
    pub accepted_fd: ::core::ffi::c_int,
    pub queued_fds: *mut ::core::ffi::c_void,
}
pub type uv_connection_cb =
    Option<unsafe extern "C" fn(*mut uv_stream_t, ::core::ffi::c_int) -> ()>;
pub type uv_shutdown_t = uv_shutdown_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_shutdown_s {
    pub data: *mut ::core::ffi::c_void,
    pub type_0: uv_req_type,
    pub reserved: [*mut ::core::ffi::c_void; 6],
    pub handle: *mut uv_stream_t,
    pub cb: uv_shutdown_cb,
}
pub type uv_shutdown_cb =
    Option<unsafe extern "C" fn(*mut uv_shutdown_t, ::core::ffi::c_int) -> ()>;
pub type uv_req_type = ::core::ffi::c_uint;
pub const UV_REQ_TYPE_MAX: uv_req_type = 11;
pub const UV_RANDOM: uv_req_type = 10;
pub const UV_GETNAMEINFO: uv_req_type = 9;
pub const UV_GETADDRINFO: uv_req_type = 8;
pub const UV_WORK: uv_req_type = 7;
pub const UV_FS: uv_req_type = 6;
pub const UV_UDP_SEND: uv_req_type = 5;
pub const UV_SHUTDOWN: uv_req_type = 4;
pub const UV_WRITE: uv_req_type = 3;
pub const UV_CONNECT: uv_req_type = 2;
pub const UV_REQ: uv_req_type = 1;
pub const UV_UNKNOWN_REQ: uv_req_type = 0;
pub type uv_connect_t = uv_connect_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_connect_s {
    pub data: *mut ::core::ffi::c_void,
    pub type_0: uv_req_type,
    pub reserved: [*mut ::core::ffi::c_void; 6],
    pub cb: uv_connect_cb,
    pub handle: *mut uv_stream_t,
    pub queue: uv__queue,
}
pub type uv_connect_cb = Option<unsafe extern "C" fn(*mut uv_connect_t, ::core::ffi::c_int) -> ()>;
pub type uv_read_cb =
    Option<unsafe extern "C" fn(*mut uv_stream_t, ssize_t, *const uv_buf_t) -> ()>;
pub type uv_alloc_cb = Option<unsafe extern "C" fn(*mut uv_handle_t, size_t, *mut uv_buf_t) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_22 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_23 {
    pub pipe: uv_pipe_t,
    pub tcp: uv_tcp_t,
    pub idle: uv_idle_t,
}
pub type uv_idle_t = uv_idle_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_idle_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_24,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub idle_cb: uv_idle_cb,
    pub queue: uv__queue,
}
pub type uv_idle_cb = Option<unsafe extern "C" fn(*mut uv_idle_t) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_24 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_tcp_t = uv_tcp_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_tcp_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_25,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub write_queue_size: size_t,
    pub alloc_cb: uv_alloc_cb,
    pub read_cb: uv_read_cb,
    pub connect_req: *mut uv_connect_t,
    pub shutdown_req: *mut uv_shutdown_t,
    pub io_watcher: uv__io_t,
    pub write_queue: uv__queue,
    pub write_completed_queue: uv__queue,
    pub connection_cb: uv_connection_cb,
    pub delayed_error: ::core::ffi::c_int,
    pub accepted_fd: ::core::ffi::c_int,
    pub queued_fds: *mut ::core::ffi::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_25 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_pipe_t = uv_pipe_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_pipe_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_26,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub write_queue_size: size_t,
    pub alloc_cb: uv_alloc_cb,
    pub read_cb: uv_read_cb,
    pub connect_req: *mut uv_connect_t,
    pub shutdown_req: *mut uv_shutdown_t,
    pub io_watcher: uv__io_t,
    pub write_queue: uv__queue,
    pub write_completed_queue: uv__queue,
    pub connection_cb: uv_connection_cb,
    pub delayed_error: ::core::ffi::c_int,
    pub accepted_fd: ::core::ffi::c_int,
    pub queued_fds: *mut ::core::ffi::c_void,
    pub ipc: ::core::ffi::c_int,
    pub pipe_fname: *const ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_26 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type Loop = loop_0;
pub type ProcType = ::core::ffi::c_uint;
pub const kProcTypePty: ProcType = 1;
pub const kProcTypeUv: ProcType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StringBuilder {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ::core::ffi::c_char,
}
pub type MessageType = ::core::ffi::c_int;
pub const kMessageTypeRedrawEvent: MessageType = 3;
pub const kMessageTypeNotification: MessageType = 2;
pub const kMessageTypeResponse: MessageType = 1;
pub const kMessageTypeRequest: MessageType = 0;
pub const kMessageTypeUnknown: MessageType = -1;
pub type Buffer = handle_T;
pub type Tabpage = handle_T;
pub type OptionalKeys = uint64_t;
pub type HLGroupID = Integer;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_empty {
    pub is_set__empty_: OptionalKeys,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_context {
    pub is_set__context_: OptionalKeys,
    pub types: Array,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_keymap {
    pub is_set__keymap_: OptionalKeys,
    pub noremap: Boolean,
    pub nowait: Boolean,
    pub silent: Boolean,
    pub script: Boolean,
    pub expr: Boolean,
    pub unique: Boolean,
    pub callback: LuaRef,
    pub desc: String_0,
    pub replace_keycodes: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_runtime {
    pub is_lua: Boolean,
    pub do_source: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_eval_statusline {
    pub is_set__eval_statusline_: OptionalKeys,
    pub winid: Window,
    pub maxwidth: Integer,
    pub fillchar: String_0,
    pub highlights: Boolean,
    pub use_winbar: Boolean,
    pub use_tabline: Boolean,
    pub use_statuscol_lnum: Integer,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_highlight {
    pub is_set__highlight_: OptionalKeys,
    pub altfont: Boolean,
    pub blink: Boolean,
    pub bold: Boolean,
    pub conceal: Boolean,
    pub dim: Boolean,
    pub italic: Boolean,
    pub nocombine: Boolean,
    pub overline: Boolean,
    pub reverse: Boolean,
    pub standout: Boolean,
    pub strikethrough: Boolean,
    pub undercurl: Boolean,
    pub underdashed: Boolean,
    pub underdotted: Boolean,
    pub underdouble: Boolean,
    pub underline: Boolean,
    pub default_: Boolean,
    pub cterm: Dict,
    pub foreground: Object,
    pub fg: Object,
    pub background: Object,
    pub bg: Object,
    pub ctermfg: Object,
    pub ctermbg: Object,
    pub special: Object,
    pub sp: Object,
    pub link: HLGroupID,
    pub link_global: HLGroupID,
    pub fallback: Boolean,
    pub blend: Integer,
    pub fg_indexed: Boolean,
    pub bg_indexed: Boolean,
    pub force: Boolean,
    pub update: Boolean,
    pub url: String_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_get_highlight {
    pub is_set__get_highlight_: OptionalKeys,
    pub id: Integer,
    pub name: String_0,
    pub link: Boolean,
    pub create: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_get_ns {
    pub is_set__get_ns_: OptionalKeys,
    pub winid: Window,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_echo_opts {
    pub is_set__echo_opts_: OptionalKeys,
    pub err: Boolean,
    pub verbose: Boolean,
    pub _truncate: Boolean,
    pub kind: String_0,
    pub id: Object,
    pub title: String_0,
    pub status: String_0,
    pub percent: Integer,
    pub source: String_0,
    pub data: Dict,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_open_term {
    pub is_set__open_term_: OptionalKeys,
    pub on_input: LuaRef,
    pub force_crlf: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_complete_set {
    pub is_set__complete_set_: OptionalKeys,
    pub info: String_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_redraw {
    pub is_set__redraw_: OptionalKeys,
    pub flush: Boolean,
    pub cursor: Boolean,
    pub valid: Boolean,
    pub statuscolumn: Boolean,
    pub statusline: Boolean,
    pub tabline: Boolean,
    pub winbar: Boolean,
    pub range: Array,
    pub win: Window,
    pub buf: Buffer,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dictitem_T {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 0],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct GridLineEvent {
    pub args: [::core::ffi::c_int; 3],
    pub icell: ::core::ffi::c_int,
    pub ncells: ::core::ffi::c_int,
    pub coloff: ::core::ffi::c_int,
    pub cur_attr: ::core::ffi::c_int,
    pub clear_width: ::core::ffi::c_int,
    pub wrap: bool,
}
pub type RgbValue = int32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HlAttrs {
    pub rgb_ae_attr: int32_t,
    pub cterm_ae_attr: int32_t,
    pub rgb_fg_color: RgbValue,
    pub rgb_bg_color: RgbValue,
    pub rgb_sp_color: RgbValue,
    pub cterm_fg_color: int16_t,
    pub cterm_bg_color: int16_t,
    pub hl_blend: int32_t,
    pub url: int32_t,
}
pub type hlf_T = ::core::ffi::c_uint;
pub const HLF_COUNT: hlf_T = 76;
pub const HLF_PRE: hlf_T = 75;
pub const HLF_OK: hlf_T = 74;
pub const HLF_SO: hlf_T = 73;
pub const HLF_SE: hlf_T = 72;
pub const HLF_TSNC: hlf_T = 71;
pub const HLF_TS: hlf_T = 70;
pub const HLF_BFOOTER: hlf_T = 69;
pub const HLF_BTITLE: hlf_T = 68;
pub const HLF_CU: hlf_T = 67;
pub const HLF_WBRNC: hlf_T = 66;
pub const HLF_WBR: hlf_T = 65;
pub const HLF_BORDER: hlf_T = 64;
pub const HLF_MSG: hlf_T = 63;
pub const HLF_NFLOAT: hlf_T = 62;
pub const HLF_MSGSEP: hlf_T = 61;
pub const HLF_INACTIVE: hlf_T = 60;
pub const HLF_0: hlf_T = 59;
pub const HLF_QFL: hlf_T = 58;
pub const HLF_MC: hlf_T = 57;
pub const HLF_CUL: hlf_T = 56;
pub const HLF_CUC: hlf_T = 55;
pub const HLF_TPF: hlf_T = 54;
pub const HLF_TPS: hlf_T = 53;
pub const HLF_TP: hlf_T = 52;
pub const HLF_PBR: hlf_T = 51;
pub const HLF_PST: hlf_T = 50;
pub const HLF_PSB: hlf_T = 49;
pub const HLF_PSX: hlf_T = 48;
pub const HLF_PNX: hlf_T = 47;
pub const HLF_PSK: hlf_T = 46;
pub const HLF_PNK: hlf_T = 45;
pub const HLF_PMSI: hlf_T = 44;
pub const HLF_PMNI: hlf_T = 43;
pub const HLF_PSI: hlf_T = 42;
pub const HLF_PNI: hlf_T = 41;
pub const HLF_SPL: hlf_T = 40;
pub const HLF_SPR: hlf_T = 39;
pub const HLF_SPC: hlf_T = 38;
pub const HLF_SPB: hlf_T = 37;
pub const HLF_CONCEAL: hlf_T = 36;
pub const HLF_SC: hlf_T = 35;
pub const HLF_TXA: hlf_T = 34;
pub const HLF_TXD: hlf_T = 33;
pub const HLF_DED: hlf_T = 32;
pub const HLF_CHD: hlf_T = 31;
pub const HLF_ADD: hlf_T = 30;
pub const HLF_FC: hlf_T = 29;
pub const HLF_FL: hlf_T = 28;
pub const HLF_WM: hlf_T = 27;
pub const HLF_W: hlf_T = 26;
pub const HLF_VNC: hlf_T = 25;
pub const HLF_V: hlf_T = 24;
pub const HLF_T: hlf_T = 23;
pub const HLF_VSP: hlf_T = 22;
pub const HLF_C: hlf_T = 21;
pub const HLF_SNC: hlf_T = 20;
pub const HLF_S: hlf_T = 19;
pub const HLF_R: hlf_T = 18;
pub const HLF_CLF: hlf_T = 17;
pub const HLF_CLS: hlf_T = 16;
pub const HLF_CLN: hlf_T = 15;
pub const HLF_LNB: hlf_T = 14;
pub const HLF_LNA: hlf_T = 13;
pub const HLF_N: hlf_T = 12;
pub const HLF_CM: hlf_T = 11;
pub const HLF_M: hlf_T = 10;
pub const HLF_LC: hlf_T = 9;
pub const HLF_L: hlf_T = 8;
pub const HLF_I: hlf_T = 7;
pub const HLF_E: hlf_T = 6;
pub const HLF_D: hlf_T = 5;
pub const HLF_AT: hlf_T = 4;
pub const HLF_TERM: hlf_T = 3;
pub const HLF_EOB: hlf_T = 2;
pub const HLF_8: hlf_T = 1;
pub const HLF_NONE: hlf_T = 0;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_int;
pub const BACKWARD_FILE: C2Rust_Unnamed_27 = -3;
pub const FORWARD_FILE: C2Rust_Unnamed_27 = 3;
pub const BACKWARD: C2Rust_Unnamed_27 = -1;
pub const FORWARD: C2Rust_Unnamed_27 = 1;
pub const kDirectionNotSet: C2Rust_Unnamed_27 = 0;
pub type CdScope = ::core::ffi::c_int;
pub const kCdScopeGlobal: CdScope = 2;
pub const kCdScopeTabpage: CdScope = 1;
pub const kCdScopeWindow: CdScope = 0;
pub const kCdScopeInvalid: CdScope = -1;
pub type OptIndex = ::core::ffi::c_int;
pub const kOptWritedelay: OptIndex = 373;
pub const kOptWritebackup: OptIndex = 372;
pub const kOptWriteany: OptIndex = 371;
pub const kOptWrite: OptIndex = 370;
pub const kOptWrapscan: OptIndex = 369;
pub const kOptWrapmargin: OptIndex = 368;
pub const kOptWrap: OptIndex = 367;
pub const kOptWinwidth: OptIndex = 366;
pub const kOptWinminwidth: OptIndex = 365;
pub const kOptWinminheight: OptIndex = 364;
pub const kOptWinhighlight: OptIndex = 363;
pub const kOptWinheight: OptIndex = 362;
pub const kOptWinfixwidth: OptIndex = 361;
pub const kOptWinfixheight: OptIndex = 360;
pub const kOptWinfixbuf: OptIndex = 359;
pub const kOptWindow: OptIndex = 358;
pub const kOptWinborder: OptIndex = 357;
pub const kOptWinblend: OptIndex = 356;
pub const kOptWinbar: OptIndex = 355;
pub const kOptWinaltkeys: OptIndex = 354;
pub const kOptWildoptions: OptIndex = 353;
pub const kOptWildmode: OptIndex = 352;
pub const kOptWildmenu: OptIndex = 351;
pub const kOptWildignorecase: OptIndex = 350;
pub const kOptWildignore: OptIndex = 349;
pub const kOptWildcharm: OptIndex = 348;
pub const kOptWildchar: OptIndex = 347;
pub const kOptWhichwrap: OptIndex = 346;
pub const kOptWarn: OptIndex = 345;
pub const kOptVisualbell: OptIndex = 344;
pub const kOptVirtualedit: OptIndex = 343;
pub const kOptViewoptions: OptIndex = 342;
pub const kOptViewdir: OptIndex = 341;
pub const kOptVerbosefile: OptIndex = 340;
pub const kOptVerbose: OptIndex = 339;
pub const kOptVartabstop: OptIndex = 338;
pub const kOptVarsofttabstop: OptIndex = 337;
pub const kOptUpdatetime: OptIndex = 336;
pub const kOptUpdatecount: OptIndex = 335;
pub const kOptUndoreload: OptIndex = 334;
pub const kOptUndolevels: OptIndex = 333;
pub const kOptUndofile: OptIndex = 332;
pub const kOptUndodir: OptIndex = 331;
pub const kOptTtyfast: OptIndex = 330;
pub const kOptTtimeoutlen: OptIndex = 329;
pub const kOptTtimeout: OptIndex = 328;
pub const kOptTitlestring: OptIndex = 327;
pub const kOptTitleold: OptIndex = 326;
pub const kOptTitlelen: OptIndex = 325;
pub const kOptTitle: OptIndex = 324;
pub const kOptTimeoutlen: OptIndex = 323;
pub const kOptTimeout: OptIndex = 322;
pub const kOptTildeop: OptIndex = 321;
pub const kOptThesaurusfunc: OptIndex = 320;
pub const kOptThesaurus: OptIndex = 319;
pub const kOptTextwidth: OptIndex = 318;
pub const kOptTerse: OptIndex = 317;
pub const kOptTermsync: OptIndex = 316;
pub const kOptTermpastefilter: OptIndex = 315;
pub const kOptTermguicolors: OptIndex = 314;
pub const kOptTermencoding: OptIndex = 313;
pub const kOptTermbidi: OptIndex = 312;
pub const kOptTagstack: OptIndex = 311;
pub const kOptTags: OptIndex = 310;
pub const kOptTagrelative: OptIndex = 309;
pub const kOptTaglength: OptIndex = 308;
pub const kOptTagfunc: OptIndex = 307;
pub const kOptTagcase: OptIndex = 306;
pub const kOptTagbsearch: OptIndex = 305;
pub const kOptTabstop: OptIndex = 304;
pub const kOptTabpagemax: OptIndex = 303;
pub const kOptTabline: OptIndex = 302;
pub const kOptTabclose: OptIndex = 301;
pub const kOptSyntax: OptIndex = 300;
pub const kOptSynmaxcol: OptIndex = 299;
pub const kOptSwitchbuf: OptIndex = 298;
pub const kOptSwapfile: OptIndex = 297;
pub const kOptSuffixesadd: OptIndex = 296;
pub const kOptSuffixes: OptIndex = 295;
pub const kOptStatusline: OptIndex = 294;
pub const kOptStatuscolumn: OptIndex = 293;
pub const kOptStartofline: OptIndex = 292;
pub const kOptSplitright: OptIndex = 291;
pub const kOptSplitkeep: OptIndex = 290;
pub const kOptSplitbelow: OptIndex = 289;
pub const kOptSpellsuggest: OptIndex = 288;
pub const kOptSpelloptions: OptIndex = 287;
pub const kOptSpelllang: OptIndex = 286;
pub const kOptSpellfile: OptIndex = 285;
pub const kOptSpellcapcheck: OptIndex = 284;
pub const kOptSpell: OptIndex = 283;
pub const kOptSofttabstop: OptIndex = 282;
pub const kOptSmoothscroll: OptIndex = 281;
pub const kOptSmarttab: OptIndex = 280;
pub const kOptSmartindent: OptIndex = 279;
pub const kOptSmartcase: OptIndex = 278;
pub const kOptSigncolumn: OptIndex = 277;
pub const kOptSidescrolloff: OptIndex = 276;
pub const kOptSidescroll: OptIndex = 275;
pub const kOptShowtabline: OptIndex = 274;
pub const kOptShowmode: OptIndex = 273;
pub const kOptShowmatch: OptIndex = 272;
pub const kOptShowfulltag: OptIndex = 271;
pub const kOptShowcmdloc: OptIndex = 270;
pub const kOptShowcmd: OptIndex = 269;
pub const kOptShowbreak: OptIndex = 268;
pub const kOptShortmess: OptIndex = 267;
pub const kOptShiftwidth: OptIndex = 266;
pub const kOptShiftround: OptIndex = 265;
pub const kOptShellxquote: OptIndex = 264;
pub const kOptShellxescape: OptIndex = 263;
pub const kOptShelltemp: OptIndex = 262;
pub const kOptShellslash: OptIndex = 261;
pub const kOptShellredir: OptIndex = 260;
pub const kOptShellquote: OptIndex = 259;
pub const kOptShellpipe: OptIndex = 258;
pub const kOptShellcmdflag: OptIndex = 257;
pub const kOptShell: OptIndex = 256;
pub const kOptShadafile: OptIndex = 255;
pub const kOptShada: OptIndex = 254;
pub const kOptSessionoptions: OptIndex = 253;
pub const kOptSelectmode: OptIndex = 252;
pub const kOptSelection: OptIndex = 251;
pub const kOptSecure: OptIndex = 250;
pub const kOptSections: OptIndex = 249;
pub const kOptScrollopt: OptIndex = 248;
pub const kOptScrolloff: OptIndex = 247;
pub const kOptScrolljump: OptIndex = 246;
pub const kOptScrollbind: OptIndex = 245;
pub const kOptScrollback: OptIndex = 244;
pub const kOptScroll: OptIndex = 243;
pub const kOptRuntimepath: OptIndex = 242;
pub const kOptRulerformat: OptIndex = 241;
pub const kOptRuler: OptIndex = 240;
pub const kOptRightleftcmd: OptIndex = 239;
pub const kOptRightleft: OptIndex = 238;
pub const kOptRevins: OptIndex = 237;
pub const kOptReport: OptIndex = 236;
pub const kOptRemap: OptIndex = 235;
pub const kOptRelativenumber: OptIndex = 234;
pub const kOptRegexpengine: OptIndex = 233;
pub const kOptRedrawtime: OptIndex = 232;
pub const kOptRedrawdebug: OptIndex = 231;
pub const kOptReadonly: OptIndex = 230;
pub const kOptQuoteescape: OptIndex = 229;
pub const kOptQuickfixtextfunc: OptIndex = 228;
pub const kOptPyxversion: OptIndex = 227;
pub const kOptPumwidth: OptIndex = 226;
pub const kOptPummaxwidth: OptIndex = 225;
pub const kOptPumheight: OptIndex = 224;
pub const kOptPumborder: OptIndex = 223;
pub const kOptPumblend: OptIndex = 222;
pub const kOptPrompt: OptIndex = 221;
pub const kOptPreviewwindow: OptIndex = 220;
pub const kOptPreviewheight: OptIndex = 219;
pub const kOptPreserveindent: OptIndex = 218;
pub const kOptPath: OptIndex = 217;
pub const kOptPatchmode: OptIndex = 216;
pub const kOptPatchexpr: OptIndex = 215;
pub const kOptPastetoggle: OptIndex = 214;
pub const kOptPaste: OptIndex = 213;
pub const kOptParagraphs: OptIndex = 212;
pub const kOptPackpath: OptIndex = 211;
pub const kOptOperatorfunc: OptIndex = 210;
pub const kOptOpendevice: OptIndex = 209;
pub const kOptOmnifunc: OptIndex = 208;
pub const kOptNumberwidth: OptIndex = 207;
pub const kOptNumber: OptIndex = 206;
pub const kOptNrformats: OptIndex = 205;
pub const kOptMousetime: OptIndex = 204;
pub const kOptMouseshape: OptIndex = 203;
pub const kOptMousescroll: OptIndex = 202;
pub const kOptMousemoveevent: OptIndex = 201;
pub const kOptMousemodel: OptIndex = 200;
pub const kOptMousehide: OptIndex = 199;
pub const kOptMousefocus: OptIndex = 198;
pub const kOptMouse: OptIndex = 197;
pub const kOptMore: OptIndex = 196;
pub const kOptModified: OptIndex = 195;
pub const kOptModifiable: OptIndex = 194;
pub const kOptModelines: OptIndex = 193;
pub const kOptModelineexpr: OptIndex = 192;
pub const kOptModeline: OptIndex = 191;
pub const kOptMkspellmem: OptIndex = 190;
pub const kOptMessagesopt: OptIndex = 189;
pub const kOptMenuitems: OptIndex = 188;
pub const kOptMaxsearchcount: OptIndex = 187;
pub const kOptMaxmempattern: OptIndex = 186;
pub const kOptMaxmapdepth: OptIndex = 185;
pub const kOptMaxfuncdepth: OptIndex = 184;
pub const kOptMaxcombine: OptIndex = 183;
pub const kOptMatchtime: OptIndex = 182;
pub const kOptMatchpairs: OptIndex = 181;
pub const kOptMakeprg: OptIndex = 180;
pub const kOptMakeencoding: OptIndex = 179;
pub const kOptMakeef: OptIndex = 178;
pub const kOptMagic: OptIndex = 177;
pub const kOptLoadplugins: OptIndex = 176;
pub const kOptListchars: OptIndex = 175;
pub const kOptList: OptIndex = 174;
pub const kOptLispwords: OptIndex = 173;
pub const kOptLispoptions: OptIndex = 172;
pub const kOptLisp: OptIndex = 171;
pub const kOptLinespace: OptIndex = 170;
pub const kOptLines: OptIndex = 169;
pub const kOptLinebreak: OptIndex = 168;
pub const kOptLhistory: OptIndex = 167;
pub const kOptLazyredraw: OptIndex = 166;
pub const kOptLaststatus: OptIndex = 165;
pub const kOptLangremap: OptIndex = 164;
pub const kOptLangnoremap: OptIndex = 163;
pub const kOptLangmenu: OptIndex = 162;
pub const kOptLangmap: OptIndex = 161;
pub const kOptKeywordprg: OptIndex = 160;
pub const kOptKeymodel: OptIndex = 159;
pub const kOptKeymap: OptIndex = 158;
pub const kOptJumpoptions: OptIndex = 157;
pub const kOptJoinspaces: OptIndex = 156;
pub const kOptIsprint: OptIndex = 155;
pub const kOptIskeyword: OptIndex = 154;
pub const kOptIsident: OptIndex = 153;
pub const kOptIsfname: OptIndex = 152;
pub const kOptInsertmode: OptIndex = 151;
pub const kOptInfercase: OptIndex = 150;
pub const kOptIndentkeys: OptIndex = 149;
pub const kOptIndentexpr: OptIndex = 148;
pub const kOptIncsearch: OptIndex = 147;
pub const kOptIncludeexpr: OptIndex = 146;
pub const kOptInclude: OptIndex = 145;
pub const kOptInccommand: OptIndex = 144;
pub const kOptImsearch: OptIndex = 143;
pub const kOptIminsert: OptIndex = 142;
pub const kOptImdisable: OptIndex = 141;
pub const kOptImcmdline: OptIndex = 140;
pub const kOptIgnorecase: OptIndex = 139;
pub const kOptIconstring: OptIndex = 138;
pub const kOptIcon: OptIndex = 137;
pub const kOptHlsearch: OptIndex = 136;
pub const kOptHkmapp: OptIndex = 135;
pub const kOptHkmap: OptIndex = 134;
pub const kOptHistory: OptIndex = 133;
pub const kOptHighlight: OptIndex = 132;
pub const kOptHidden: OptIndex = 131;
pub const kOptHelplang: OptIndex = 130;
pub const kOptHelpheight: OptIndex = 129;
pub const kOptHelpfile: OptIndex = 128;
pub const kOptGuitabtooltip: OptIndex = 127;
pub const kOptGuitablabel: OptIndex = 126;
pub const kOptGuioptions: OptIndex = 125;
pub const kOptGuifontwide: OptIndex = 124;
pub const kOptGuifont: OptIndex = 123;
pub const kOptGuicursor: OptIndex = 122;
pub const kOptGrepprg: OptIndex = 121;
pub const kOptGrepformat: OptIndex = 120;
pub const kOptGdefault: OptIndex = 119;
pub const kOptFsync: OptIndex = 118;
pub const kOptFormatprg: OptIndex = 117;
pub const kOptFormatoptions: OptIndex = 116;
pub const kOptFormatlistpat: OptIndex = 115;
pub const kOptFormatexpr: OptIndex = 114;
pub const kOptFoldtext: OptIndex = 113;
pub const kOptFoldopen: OptIndex = 112;
pub const kOptFoldnestmax: OptIndex = 111;
pub const kOptFoldminlines: OptIndex = 110;
pub const kOptFoldmethod: OptIndex = 109;
pub const kOptFoldmarker: OptIndex = 108;
pub const kOptFoldlevelstart: OptIndex = 107;
pub const kOptFoldlevel: OptIndex = 106;
pub const kOptFoldignore: OptIndex = 105;
pub const kOptFoldexpr: OptIndex = 104;
pub const kOptFoldenable: OptIndex = 103;
pub const kOptFoldcolumn: OptIndex = 102;
pub const kOptFoldclose: OptIndex = 101;
pub const kOptFixendofline: OptIndex = 100;
pub const kOptFindfunc: OptIndex = 99;
pub const kOptFillchars: OptIndex = 98;
pub const kOptFiletype: OptIndex = 97;
pub const kOptFileignorecase: OptIndex = 96;
pub const kOptFileformats: OptIndex = 95;
pub const kOptFileformat: OptIndex = 94;
pub const kOptFileencodings: OptIndex = 93;
pub const kOptFileencoding: OptIndex = 92;
pub const kOptExrc: OptIndex = 91;
pub const kOptExpandtab: OptIndex = 90;
pub const kOptEventignorewin: OptIndex = 89;
pub const kOptEventignore: OptIndex = 88;
pub const kOptErrorformat: OptIndex = 87;
pub const kOptErrorfile: OptIndex = 86;
pub const kOptErrorbells: OptIndex = 85;
pub const kOptEqualprg: OptIndex = 84;
pub const kOptEqualalways: OptIndex = 83;
pub const kOptEndofline: OptIndex = 82;
pub const kOptEndoffile: OptIndex = 81;
pub const kOptEncoding: OptIndex = 80;
pub const kOptEmoji: OptIndex = 79;
pub const kOptEdcompatible: OptIndex = 78;
pub const kOptEadirection: OptIndex = 77;
pub const kOptDisplay: OptIndex = 76;
pub const kOptDirectory: OptIndex = 75;
pub const kOptDigraph: OptIndex = 74;
pub const kOptDiffopt: OptIndex = 73;
pub const kOptDiffexpr: OptIndex = 72;
pub const kOptDiffanchors: OptIndex = 71;
pub const kOptDiff: OptIndex = 70;
pub const kOptDictionary: OptIndex = 69;
pub const kOptDelcombine: OptIndex = 68;
pub const kOptDefine: OptIndex = 67;
pub const kOptDebug: OptIndex = 66;
pub const kOptCursorlineopt: OptIndex = 65;
pub const kOptCursorline: OptIndex = 64;
pub const kOptCursorcolumn: OptIndex = 63;
pub const kOptCursorbind: OptIndex = 62;
pub const kOptCpoptions: OptIndex = 61;
pub const kOptCopyindent: OptIndex = 60;
pub const kOptConfirm: OptIndex = 59;
pub const kOptConceallevel: OptIndex = 58;
pub const kOptConcealcursor: OptIndex = 57;
pub const kOptCompletetimeout: OptIndex = 56;
pub const kOptCompleteslash: OptIndex = 55;
pub const kOptCompleteopt: OptIndex = 54;
pub const kOptCompleteitemalign: OptIndex = 53;
pub const kOptCompletefunc: OptIndex = 52;
pub const kOptComplete: OptIndex = 51;
pub const kOptCompatible: OptIndex = 50;
pub const kOptCommentstring: OptIndex = 49;
pub const kOptComments: OptIndex = 48;
pub const kOptColumns: OptIndex = 47;
pub const kOptColorcolumn: OptIndex = 46;
pub const kOptCmdwinheight: OptIndex = 45;
pub const kOptCmdheight: OptIndex = 44;
pub const kOptClipboard: OptIndex = 43;
pub const kOptCinwords: OptIndex = 42;
pub const kOptCinscopedecls: OptIndex = 41;
pub const kOptCinoptions: OptIndex = 40;
pub const kOptCinkeys: OptIndex = 39;
pub const kOptCindent: OptIndex = 38;
pub const kOptChistory: OptIndex = 37;
pub const kOptCharconvert: OptIndex = 36;
pub const kOptChannel: OptIndex = 35;
pub const kOptCedit: OptIndex = 34;
pub const kOptCdpath: OptIndex = 33;
pub const kOptCdhome: OptIndex = 32;
pub const kOptCasemap: OptIndex = 31;
pub const kOptBusy: OptIndex = 30;
pub const kOptBuftype: OptIndex = 29;
pub const kOptBuflisted: OptIndex = 28;
pub const kOptBufhidden: OptIndex = 27;
pub const kOptBrowsedir: OptIndex = 26;
pub const kOptBreakindentopt: OptIndex = 25;
pub const kOptBreakindent: OptIndex = 24;
pub const kOptBreakat: OptIndex = 23;
pub const kOptBomb: OptIndex = 22;
pub const kOptBinary: OptIndex = 21;
pub const kOptBelloff: OptIndex = 20;
pub const kOptBackupskip: OptIndex = 19;
pub const kOptBackupext: OptIndex = 18;
pub const kOptBackupdir: OptIndex = 17;
pub const kOptBackupcopy: OptIndex = 16;
pub const kOptBackup: OptIndex = 15;
pub const kOptBackspace: OptIndex = 14;
pub const kOptBackground: OptIndex = 13;
pub const kOptAutowriteall: OptIndex = 12;
pub const kOptAutowrite: OptIndex = 11;
pub const kOptAutoread: OptIndex = 10;
pub const kOptAutoindent: OptIndex = 9;
pub const kOptAutocompletetimeout: OptIndex = 8;
pub const kOptAutocompletedelay: OptIndex = 7;
pub const kOptAutocomplete: OptIndex = 6;
pub const kOptAutochdir: OptIndex = 5;
pub const kOptArabicshape: OptIndex = 4;
pub const kOptArabic: OptIndex = 3;
pub const kOptAmbiwidth: OptIndex = 2;
pub const kOptAllowrevins: OptIndex = 1;
pub const kOptAleph: OptIndex = 0;
pub const kOptInvalid: OptIndex = -1;
pub type OptValType = ::core::ffi::c_int;
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeNumber: OptValType = 1;
pub const kOptValTypeBoolean: OptValType = 0;
pub const kOptValTypeNil: OptValType = -1;
pub type OptScope = ::core::ffi::c_uint;
pub const kOptScopeBuf: OptScope = 2;
pub const kOptScopeWin: OptScope = 1;
pub const kOptScopeGlobal: OptScope = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub union OptValData {
    pub boolean: TriState,
    pub number: OptInt,
    pub string: String_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct OptVal {
    pub type_0: OptValType,
    pub data: OptValData,
}
pub type uv_gid_t = gid_t;
pub type uv_uid_t = uid_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_process_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_28,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub exit_cb: uv_exit_cb,
    pub pid: ::core::ffi::c_int,
    pub queue: uv__queue,
    pub status: ::core::ffi::c_int,
}
pub type uv_exit_cb =
    Option<unsafe extern "C" fn(*mut uv_process_t, int64_t, ::core::ffi::c_int) -> ()>;
pub type uv_process_t = uv_process_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_28 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_stdio_flags = ::core::ffi::c_uint;
pub const UV_OVERLAPPED_PIPE: uv_stdio_flags = 64;
pub const UV_NONBLOCK_PIPE: uv_stdio_flags = 64;
pub const UV_WRITABLE_PIPE: uv_stdio_flags = 32;
pub const UV_READABLE_PIPE: uv_stdio_flags = 16;
pub const UV_INHERIT_STREAM: uv_stdio_flags = 4;
pub const UV_INHERIT_FD: uv_stdio_flags = 2;
pub const UV_CREATE_PIPE: uv_stdio_flags = 1;
pub const UV_IGNORE: uv_stdio_flags = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_stdio_container_s {
    pub flags: uv_stdio_flags,
    pub data: C2Rust_Unnamed_29,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_29 {
    pub stream: *mut uv_stream_t,
    pub fd: ::core::ffi::c_int,
}
pub type uv_stdio_container_t = uv_stdio_container_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_process_options_s {
    pub exit_cb: uv_exit_cb,
    pub file: *const ::core::ffi::c_char,
    pub args: *mut *mut ::core::ffi::c_char,
    pub env: *mut *mut ::core::ffi::c_char,
    pub cwd: *const ::core::ffi::c_char,
    pub flags: ::core::ffi::c_uint,
    pub stdio_count: ::core::ffi::c_int,
    pub stdio: *mut uv_stdio_container_t,
    pub uid: uv_uid_t,
    pub gid: uv_gid_t,
}
pub type uv_process_options_t = uv_process_options_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct foldinfo_T {
    pub fi_lnum: linenr_T,
    pub fi_level: ::core::ffi::c_int,
    pub fi_low_level: ::core::ffi::c_int,
    pub fi_lines: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SignTextAttrs {
    pub text: [schar_T; 2],
    pub hl_id: ::core::ffi::c_int,
}
pub type StlFlag = ::core::ffi::c_uint;
pub const STL_CLICK_FUNC: StlFlag = 64;
pub const STL_TABCLOSENR: StlFlag = 88;
pub const STL_TABPAGENR: StlFlag = 84;
pub const STL_HIGHLIGHT_COMB: StlFlag = 36;
pub const STL_HIGHLIGHT: StlFlag = 35;
pub const STL_USER_HL: StlFlag = 42;
pub const STL_TRUNCMARK: StlFlag = 60;
pub const STL_SEPARATE: StlFlag = 61;
pub const STL_VIM_EXPR: StlFlag = 123;
pub const STL_SIGNCOL: StlFlag = 115;
pub const STL_FOLDCOL: StlFlag = 67;
pub const STL_SHOWCMD: StlFlag = 83;
pub const STL_PAGENUM: StlFlag = 78;
pub const STL_ARGLISTSTAT: StlFlag = 97;
pub const STL_ALTPERCENT: StlFlag = 80;
pub const STL_PERCENTAGE: StlFlag = 112;
pub const STL_QUICKFIX: StlFlag = 113;
pub const STL_MODIFIED_ALT: StlFlag = 77;
pub const STL_MODIFIED: StlFlag = 109;
pub const STL_PREVIEWFLAG_ALT: StlFlag = 87;
pub const STL_PREVIEWFLAG: StlFlag = 119;
pub const STL_FILETYPE_ALT: StlFlag = 89;
pub const STL_FILETYPE: StlFlag = 121;
pub const STL_HELPFLAG_ALT: StlFlag = 72;
pub const STL_HELPFLAG: StlFlag = 104;
pub const STL_ROFLAG_ALT: StlFlag = 82;
pub const STL_ROFLAG: StlFlag = 114;
pub const STL_BYTEVAL_X: StlFlag = 66;
pub const STL_BYTEVAL: StlFlag = 98;
pub const STL_OFFSET_X: StlFlag = 79;
pub const STL_OFFSET: StlFlag = 111;
pub const STL_KEYMAP: StlFlag = 107;
pub const STL_BUFNO: StlFlag = 110;
pub const STL_NUMLINES: StlFlag = 76;
pub const STL_LINE: StlFlag = 108;
pub const STL_VIRTCOL_ALT: StlFlag = 86;
pub const STL_VIRTCOL: StlFlag = 118;
pub const STL_COLUMN: StlFlag = 99;
pub const STL_FILENAME: StlFlag = 116;
pub const STL_FULLPATH: StlFlag = 70;
pub const STL_FILEPATH: StlFlag = 102;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StlClickRecord {
    pub def: StlClickDefinition,
    pub start: *const ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stl_hlrec {
    pub start: *mut ::core::ffi::c_char,
    pub userhl: ::core::ffi::c_int,
    pub item: StlFlag,
}
pub type stl_hlrec_t = stl_hlrec;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct statuscol_T {
    pub width: ::core::ffi::c_int,
    pub lnum: linenr_T,
    pub sign_cul_id: ::core::ffi::c_int,
    pub draw: bool,
    pub hlrec: *mut stl_hlrec_t,
    pub foldinfo: foldinfo_T,
    pub fold_vcol: [colnr_T; 9],
    pub sattrs: *mut SignTextAttrs,
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
pub struct HlMessageChunk {
    pub text: String_0,
    pub hl_id: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HlMessage {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut HlMessageChunk,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_data {
    pub source: String_0,
    pub percent: Integer,
    pub title: String_0,
    pub status: String_0,
    pub data: Dict,
}
pub type MessageData = msg_data;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ArrayBuilder {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut Object,
    pub init_array: [Object; 16],
}
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
pub const UPD_NOT_VALID: C2Rust_Unnamed_34 = 40;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct typebuf_T {
    pub tb_buf: *mut uint8_t,
    pub tb_noremap: *mut uint8_t,
    pub tb_buflen: ::core::ffi::c_int,
    pub tb_off: ::core::ffi::c_int,
    pub tb_len: ::core::ffi::c_int,
    pub tb_maplen: ::core::ffi::c_int,
    pub tb_silent: ::core::ffi::c_int,
    pub tb_no_abbr_cnt: ::core::ffi::c_int,
    pub tb_change_cnt: ::core::ffi::c_int,
}
pub const REMAP_NONE: RemapValues = -1;
pub const REMAP_YES: RemapValues = 0;
pub const KE_LEFTMOUSE: key_extra = 44;
pub const KE_LEFTRELEASE: key_extra = 46;
pub const KE_LEFTDRAG: key_extra = 45;
pub const KE_MOUSEMOVE: key_extra = 100;
pub const KE_MOUSELEFT: key_extra = 77;
pub const KE_MOUSERIGHT: key_extra = 78;
pub const KE_MOUSEUP: key_extra = 76;
pub const KE_MOUSEDOWN: key_extra = 75;
pub const KE_X2MOUSE: key_extra = 92;
pub const KE_X1MOUSE: key_extra = 89;
pub const KE_RIGHTMOUSE: key_extra = 50;
pub const KE_MIDDLEMOUSE: key_extra = 47;
pub const REPTERM_NO_SPECIAL: C2Rust_Unnamed_36 = 4;
pub const REPTERM_DO_LT: C2Rust_Unnamed_36 = 2;
pub const REPTERM_FROM_PART: C2Rust_Unnamed_36 = 1;
pub type LuaRetMode = ::core::ffi::c_uint;
pub const kRetMulti: LuaRetMode = 3;
pub const kRetLuaref: LuaRetMode = 2;
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetObject: LuaRetMode = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct RuntimeCookie {
    pub rv: ArrayBuilder,
    pub arena: *mut Arena,
}
pub const DIP_ALL: C2Rust_Unnamed_41 = 1;
pub const DIP_DIRFILE: C2Rust_Unnamed_41 = 512;
pub type DoInRuntimepathCB = Option<
    unsafe extern "C" fn(
        ::core::ffi::c_int,
        *mut *mut ::core::ffi::c_char,
        bool,
        *mut ::core::ffi::c_void,
    ) -> bool,
>;
pub const DOSO_NONE: C2Rust_Unnamed_40 = 0;
pub const DOBUF_FIRST: dobuf_start_values = 1;
pub const DOBUF_GOTO: dobuf_action_values = 0;
pub const BLN_LISTED: bln_values = 2;
pub const BLN_NEW: bln_values = 8;
pub const BLN_NOOPT: bln_values = 16;
pub type event_T = auto_event;
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
pub const OPT_LOCAL: C2Rust_Unnamed_38 = 2;
pub const BCO_NOHELP: C2Rust_Unnamed_37 = 4;
pub const BCO_ENTER: C2Rust_Unnamed_37 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Channel {
    pub id: uint64_t,
    pub refcount: size_t,
    pub events: *mut MultiQueue,
    pub streamtype: ChannelStreamType,
    pub stream: C2Rust_Unnamed_32,
    pub is_rpc: bool,
    pub detach: bool,
    pub rpc: RpcState,
    pub term: *mut Terminal,
    pub on_data: CallbackReader,
    pub on_stderr: CallbackReader,
    pub on_exit: Callback,
    pub exit_status: ::core::ffi::c_int,
    pub callback_busy: bool,
    pub callback_scheduled: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CallbackReader {
    pub cb: Callback,
    pub self_0: *mut dict_T,
    pub buffer: garray_T,
    pub eof: bool,
    pub buffered: bool,
    pub fwd_err: bool,
    pub type_0: *const ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct RpcState {
    pub closed: bool,
    pub unpacker: *mut Unpacker,
    pub ui: *mut RemoteUI,
    pub next_request_id: uint32_t,
    pub call_stack: C2Rust_Unnamed_30,
    pub info: Dict,
    pub client_type: ClientType,
}
pub type ClientType = ::core::ffi::c_int;
pub const kClientTypePlugin: ClientType = 4;
pub const kClientTypeHost: ClientType = 3;
pub const kClientTypeEmbedder: ClientType = 2;
pub const kClientTypeUi: ClientType = 1;
pub const kClientTypeMsgpackRpc: ClientType = 5;
pub const kClientTypeRemote: ClientType = 0;
pub const kClientTypeUnknown: ClientType = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_30 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut *mut ChannelCallFrame,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ChannelCallFrame {
    pub request_id: uint32_t,
    pub returned: bool,
    pub errored: bool,
    pub result: Object,
    pub result_mem: ArenaMem,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct RemoteUI {
    pub rgb: bool,
    pub override_0: bool,
    pub composed: bool,
    pub ui_ext: [bool; 10],
    pub width: ::core::ffi::c_int,
    pub height: ::core::ffi::c_int,
    pub pum_nlines: ::core::ffi::c_int,
    pub pum_pos: bool,
    pub pum_row: ::core::ffi::c_double,
    pub pum_col: ::core::ffi::c_double,
    pub pum_height: ::core::ffi::c_double,
    pub pum_width: ::core::ffi::c_double,
    pub term_name: *mut ::core::ffi::c_char,
    pub term_background: *mut ::core::ffi::c_char,
    pub term_colors: ::core::ffi::c_int,
    pub stdin_tty: bool,
    pub stdout_tty: bool,
    pub channel_id: uint64_t,
    pub packer: PackerBuffer,
    pub cur_event: *const ::core::ffi::c_char,
    pub nevents_pos: *mut ::core::ffi::c_char,
    pub ncalls_pos: *mut ::core::ffi::c_char,
    pub nevents: uint32_t,
    pub ncalls: uint32_t,
    pub flushed_events: bool,
    pub incomplete_event: bool,
    pub ncells_pending: size_t,
    pub hl_id: ::core::ffi::c_int,
    pub cursor_row: Integer,
    pub cursor_col: Integer,
    pub client_row: Integer,
    pub client_col: Integer,
    pub wildmenu_active: bool,
}
pub type PackerBuffer = packer_buffer_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct packer_buffer_t {
    pub startptr: *mut ::core::ffi::c_char,
    pub ptr: *mut ::core::ffi::c_char,
    pub endptr: *mut ::core::ffi::c_char,
    pub anydata: *mut ::core::ffi::c_void,
    pub anyint: int64_t,
    pub packer_flush: PackerBufferFlush,
}
pub type PackerBufferFlush = Option<unsafe extern "C" fn(*mut PackerBuffer) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Unpacker {
    pub parser: mpack_parser_t,
    pub reader: mpack_tokbuf_t,
    pub read_ptr: *const ::core::ffi::c_char,
    pub read_size: size_t,
    pub ext_buf: [::core::ffi::c_char; 9],
    pub state: ::core::ffi::c_int,
    pub type_0: MessageType,
    pub request_id: uint32_t,
    pub method_name_len: size_t,
    pub handler: MsgpackRpcRequestHandler,
    pub error: Object,
    pub result: Object,
    pub unpack_error: Error,
    pub arena: Arena,
    pub nevents: ::core::ffi::c_int,
    pub ncalls: ::core::ffi::c_int,
    pub ui_handler: UIClientHandler,
    pub grid_line_event: GridLineEvent,
    pub has_grid_line_event: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct UIClientHandler {
    pub name: *const ::core::ffi::c_char,
    pub fn_0: Option<unsafe extern "C" fn(Array) -> ()>,
}
pub type mpack_tokbuf_t = mpack_tokbuf_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mpack_tokbuf_s {
    pub pending: [::core::ffi::c_char; 9],
    pub pending_tok: mpack_token_t,
    pub ppos: size_t,
    pub plen: size_t,
    pub passthrough: mpack_uint32_t,
}
pub type mpack_uint32_t = ::core::ffi::c_uint;
pub type mpack_token_t = mpack_token_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mpack_token_s {
    pub type_0: mpack_token_type_t,
    pub length: mpack_uint32_t,
    pub data: C2Rust_Unnamed_31,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_31 {
    pub value: mpack_value_t,
    pub chunk_ptr: *const ::core::ffi::c_char,
    pub ext_type: ::core::ffi::c_int,
}
pub type mpack_value_t = mpack_value_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mpack_value_s {
    pub lo: mpack_uint32_t,
    pub hi: mpack_uint32_t,
}
pub type mpack_token_type_t = ::core::ffi::c_uint;
pub const MPACK_TOKEN_EXT: mpack_token_type_t = 11;
pub const MPACK_TOKEN_STR: mpack_token_type_t = 10;
pub const MPACK_TOKEN_BIN: mpack_token_type_t = 9;
pub const MPACK_TOKEN_MAP: mpack_token_type_t = 8;
pub const MPACK_TOKEN_ARRAY: mpack_token_type_t = 7;
pub const MPACK_TOKEN_CHUNK: mpack_token_type_t = 6;
pub const MPACK_TOKEN_FLOAT: mpack_token_type_t = 5;
pub const MPACK_TOKEN_SINT: mpack_token_type_t = 4;
pub const MPACK_TOKEN_UINT: mpack_token_type_t = 3;
pub const MPACK_TOKEN_BOOLEAN: mpack_token_type_t = 2;
pub const MPACK_TOKEN_NIL: mpack_token_type_t = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mpack_parser_t {
    pub data: mpack_data_t,
    pub size: mpack_uint32_t,
    pub capacity: mpack_uint32_t,
    pub status: ::core::ffi::c_int,
    pub exiting: ::core::ffi::c_int,
    pub tokbuf: mpack_tokbuf_t,
    pub items: [mpack_node_t; 33],
}
pub type mpack_node_t = mpack_node_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mpack_node_s {
    pub tok: mpack_token_t,
    pub pos: size_t,
    pub key_visited: ::core::ffi::c_int,
    pub data: [mpack_data_t; 2],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union mpack_data_t {
    pub p: *mut ::core::ffi::c_void,
    pub u: mpack_uintmax_t,
    pub i: mpack_sintmax_t,
    pub d: ::core::ffi::c_double,
}
pub type mpack_sintmax_t = ::core::ffi::c_longlong;
pub type mpack_uintmax_t = ::core::ffi::c_ulonglong;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_32 {
    pub proc: Proc,
    pub uv: LibuvProc,
    pub pty: PtyProc,
    pub socket: RStream,
    pub stdio: StdioPair,
    pub err: StderrState,
    pub internal: InternalState,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct InternalState {
    pub cb: LuaRef,
    pub closed: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StderrState {
    pub closed: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StdioPair {
    pub in_0: RStream,
    pub out: Stream,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PtyProc {
    pub proc: Proc,
    pub width: uint16_t,
    pub height: uint16_t,
    pub winsize: winsize,
    pub tty_fd: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct winsize {
    pub ws_row: ::core::ffi::c_ushort,
    pub ws_col: ::core::ffi::c_ushort,
    pub ws_xpixel: ::core::ffi::c_ushort,
    pub ws_ypixel: ::core::ffi::c_ushort,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LibuvProc {
    pub proc: Proc,
    pub uv: uv_process_t,
    pub uvopts: uv_process_options_t,
    pub uvstdio: [uv_stdio_container_t; 4],
}
pub type ChannelStreamType = ::core::ffi::c_uint;
pub const kChannelStreamInternal: ChannelStreamType = 4;
pub const kChannelStreamStderr: ChannelStreamType = 3;
pub const kChannelStreamStdio: ChannelStreamType = 2;
pub const kChannelStreamSocket: ChannelStreamType = 1;
pub const kChannelStreamProc: ChannelStreamType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TerminalOptions {
    pub data: *mut ::core::ffi::c_void,
    pub width: uint16_t,
    pub height: uint16_t,
    pub read_pause_cb: terminal_read_pause_cb,
    pub write_cb: terminal_write_cb,
    pub resize_cb: terminal_resize_cb,
    pub resume_cb: terminal_resume_cb,
    pub close_cb: terminal_close_cb,
    pub force_crlf: bool,
}
pub type terminal_close_cb = Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>;
pub type terminal_resume_cb = Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>;
pub type terminal_resize_cb =
    Option<unsafe extern "C" fn(uint16_t, uint16_t, *mut ::core::ffi::c_void) -> ()>;
pub type terminal_write_cb = Option<
    unsafe extern "C" fn(*const ::core::ffi::c_char, size_t, *mut ::core::ffi::c_void) -> (),
>;
pub type terminal_read_pause_cb =
    Option<unsafe extern "C" fn(bool, *mut ::core::ffi::c_void) -> ()>;
pub const PUT_CURSEND: C2Rust_Unnamed_39 = 2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct yankreg_T {
    pub y_array: *mut String_0,
    pub y_size: size_t,
    pub y_type: MotionType,
    pub y_width: colnr_T,
    pub timestamp: Timestamp,
    pub additional_data: *mut AdditionalData,
}
pub type MotionType = ::core::ffi::c_int;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct color_name_table_T {
    pub name: *mut ::core::ffi::c_char,
    pub color: RgbValue,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Context {
    pub regs: String_0,
    pub jumps: String_0,
    pub bufs: String_0,
    pub gvars: String_0,
    pub funcs: Array,
}
pub const kCtxFuncs: C2Rust_Unnamed_33 = 32;
pub const kCtxSFuncs: C2Rust_Unnamed_33 = 16;
pub const kCtxGVars: C2Rust_Unnamed_33 = 8;
pub const kCtxBufs: C2Rust_Unnamed_33 = 4;
pub const kCtxJumps: C2Rust_Unnamed_33 = 2;
pub const kCtxRegs: C2Rust_Unnamed_33 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct nvim_stats_s {
    pub fsync: int64_t,
    pub redraw: int64_t,
    pub log_skip: int16_t,
}
pub const UPD_CLEAR: C2Rust_Unnamed_34 = 50;
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
pub const kOptCotFlagPopup: C2Rust_Unnamed_35 = 16;
pub const UPD_VALID: C2Rust_Unnamed_34 = 10;
pub type bln_values = ::core::ffi::c_uint;
pub const BLN_NOCURWIN: bln_values = 128;
pub const BLN_DUMMY: bln_values = 4;
pub const BLN_CURBUF: bln_values = 1;
pub type dobuf_action_values = ::core::ffi::c_uint;
pub const DOBUF_WIPE: dobuf_action_values = 4;
pub const DOBUF_DEL: dobuf_action_values = 3;
pub const DOBUF_UNLOAD: dobuf_action_values = 2;
pub const DOBUF_SPLIT: dobuf_action_values = 1;
pub type dobuf_start_values = ::core::ffi::c_uint;
pub const DOBUF_MOD: dobuf_start_values = 3;
pub const DOBUF_LAST: dobuf_start_values = 2;
pub const DOBUF_CURRENT: dobuf_start_values = 0;
pub type C2Rust_Unnamed_33 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_uint;
pub const UPD_SOME_VALID: C2Rust_Unnamed_34 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_34 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_34 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_34 = 20;
pub type RemapValues = ::core::ffi::c_int;
pub const REMAP_SKIP: RemapValues = -3;
pub const REMAP_SCRIPT: RemapValues = -2;
pub type C2Rust_Unnamed_35 = ::core::ffi::c_uint;
pub const kOptCotFlagNearest: C2Rust_Unnamed_35 = 1024;
pub const kOptCotFlagPreinsert: C2Rust_Unnamed_35 = 512;
pub const kOptCotFlagNosort: C2Rust_Unnamed_35 = 256;
pub const kOptCotFlagFuzzy: C2Rust_Unnamed_35 = 128;
pub const kOptCotFlagNoselect: C2Rust_Unnamed_35 = 64;
pub const kOptCotFlagNoinsert: C2Rust_Unnamed_35 = 32;
pub const kOptCotFlagPreview: C2Rust_Unnamed_35 = 8;
pub const kOptCotFlagLongest: C2Rust_Unnamed_35 = 4;
pub const kOptCotFlagMenuone: C2Rust_Unnamed_35 = 2;
pub const kOptCotFlagMenu: C2Rust_Unnamed_35 = 1;
pub type key_extra = ::core::ffi::c_uint;
pub const KE_WILD: key_extra = 108;
pub const KE_COMMAND: key_extra = 104;
pub const KE_LUA: key_extra = 103;
pub const KE_EVENT: key_extra = 102;
pub const KE_NOP: key_extra = 97;
pub const KE_DROP: key_extra = 95;
pub const KE_X2RELEASE: key_extra = 94;
pub const KE_X2DRAG: key_extra = 93;
pub const KE_X1RELEASE: key_extra = 91;
pub const KE_X1DRAG: key_extra = 90;
pub const KE_C_END: key_extra = 88;
pub const KE_C_HOME: key_extra = 87;
pub const KE_C_RIGHT: key_extra = 86;
pub const KE_C_LEFT: key_extra = 85;
pub const KE_CMDWIN: key_extra = 84;
pub const KE_PLUG: key_extra = 83;
pub const KE_SNR: key_extra = 82;
pub const KE_KDEL: key_extra = 80;
pub const KE_KINS: key_extra = 79;
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
pub const KE_MIDDLERELEASE: key_extra = 49;
pub const KE_MIDDLEDRAG: key_extra = 48;
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
pub type C2Rust_Unnamed_36 = ::core::ffi::c_uint;
pub const REPTERM_NO_SIMPLIFY: C2Rust_Unnamed_36 = 8;
pub type C2Rust_Unnamed_37 = ::core::ffi::c_uint;
pub const BCO_ALWAYS: C2Rust_Unnamed_37 = 2;
pub type C2Rust_Unnamed_38 = ::core::ffi::c_uint;
pub const OPT_SKIPRTP: C2Rust_Unnamed_38 = 128;
pub const OPT_NO_REDRAW: C2Rust_Unnamed_38 = 64;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_38 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_38 = 16;
pub const OPT_WINONLY: C2Rust_Unnamed_38 = 8;
pub const OPT_MODELINE: C2Rust_Unnamed_38 = 4;
pub const OPT_GLOBAL: C2Rust_Unnamed_38 = 1;
pub type C2Rust_Unnamed_39 = ::core::ffi::c_uint;
pub const PUT_BLOCK_INNER: C2Rust_Unnamed_39 = 64;
pub const PUT_LINE_FORWARD: C2Rust_Unnamed_39 = 32;
pub const PUT_LINE_SPLIT: C2Rust_Unnamed_39 = 16;
pub const PUT_LINE: C2Rust_Unnamed_39 = 8;
pub const PUT_CURSLINE: C2Rust_Unnamed_39 = 4;
pub const PUT_FIXINDENT: C2Rust_Unnamed_39 = 1;
pub type C2Rust_Unnamed_40 = ::core::ffi::c_uint;
pub const DOSO_VIMRC: C2Rust_Unnamed_40 = 1;
pub type C2Rust_Unnamed_41 = ::core::ffi::c_uint;
pub const DIP_AFTER: C2Rust_Unnamed_41 = 128;
pub const DIP_NOAFTER: C2Rust_Unnamed_41 = 64;
pub const DIP_NORTP: C2Rust_Unnamed_41 = 32;
pub const DIP_OPT: C2Rust_Unnamed_41 = 16;
pub const DIP_START: C2Rust_Unnamed_41 = 8;
pub const DIP_ERR: C2Rust_Unnamed_41 = 4;
pub const DIP_DIR: C2Rust_Unnamed_41 = 2;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
#[inline(always)]
unsafe extern "C" fn _memcpy_free(
    dest: *mut ::core::ffi::c_void,
    src: *mut ::core::ffi::c_void,
    size: size_t,
) -> *mut ::core::ffi::c_void {
    memcpy(dest, src, size);
    let mut ptr_: *mut *mut ::core::ffi::c_void = &raw const src as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    *ptr_;
    return dest;
}
pub const MAX_SCHAR_SIZE: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const STRING_INIT: String_0 = String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
};
pub const INTERNAL_CALL_MASK: uint64_t = (1 as ::core::ffi::c_int as uint64_t)
    << ::core::mem::size_of::<uint64_t>()
        .wrapping_mul(8 as usize)
        .wrapping_sub(1 as usize);
#[inline(always)]
unsafe extern "C" fn is_internal_call(channel_id: uint64_t) -> bool {
    return channel_id & INTERNAL_CALL_MASK != 0;
}
pub const KEYSET_OPTIDX_context__types: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_eval_statusline__fillchar: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_eval_statusline__maxwidth: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_eval_statusline__use_statuscol_lnum: ::core::ffi::c_int =
    7 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__url: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__update: ::core::ffi::c_int = 13 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_ns__winid: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_open_term__on_input: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_open_term__force_crlf: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_complete_set__info: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_redraw__buf: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_redraw__win: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_redraw__flush: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_redraw__range: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_redraw__valid: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const LOGLVL_DBG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
static mut value_init_ptr_t: ptr_t = NULL;
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn map_get_uint64_t_ptr_t(
    mut map: *mut Map_uint64_t_ptr_t,
    mut key: uint64_t,
) -> ptr_t {
    let mut k: uint32_t = mh_get_uint64_t(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_ptr_t
    } else {
        *(*map).values.offset(k as isize)
    };
}
pub const NULL_STRING: String_0 = STRING_INIT;
#[no_mangle]
pub unsafe extern "C" fn nvim_get_hl_id_by_name(mut name: String_0) -> Integer {
    return syn_check_group(name.data, name.size) as Integer;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_get_hl(
    mut ns_id: Integer,
    mut opts: *mut KeyDict_get_highlight,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    return ns_get_hl_defs(ns_id as NS, opts, arena, err);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_set_hl(
    mut channel_id: uint64_t,
    mut ns_id: Integer,
    mut name: String_0,
    mut val: *mut KeyDict_highlight,
    mut err: *mut Error,
) {
    let mut hl_id: ::core::ffi::c_int = syn_check_group(name.data, name.size);
    if !(hl_id != 0 as ::core::ffi::c_int) {
        api_err_invalid(
            err,
            b"highlight name\0".as_ptr() as *const ::core::ffi::c_char,
            name.data,
            0 as int64_t,
            true_0 != 0,
        );
        return;
    }
    let mut link_id: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if (*val).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__url
        != 0 as ::core::ffi::c_ulonglong
    {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"Invalid key: 'url'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut update: bool = (*val).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__update
        != 0 as ::core::ffi::c_ulonglong
        && (*val).update as ::core::ffi::c_int != 0;
    let mut base: *mut HlAttrs = ::core::ptr::null_mut::<HlAttrs>();
    let mut base_attrs: HlAttrs = HlAttrs {
        rgb_ae_attr: 0,
        cterm_ae_attr: 0,
        rgb_fg_color: 0,
        rgb_bg_color: 0,
        rgb_sp_color: 0,
        cterm_fg_color: 0,
        cterm_bg_color: 0,
        hl_blend: 0,
        url: 0,
    };
    if update as ::core::ffi::c_int != 0
        && hl_ns_get_attrs(
            ns_id as ::core::ffi::c_int,
            hl_id,
            ::core::ptr::null_mut::<bool>(),
            &raw mut base_attrs,
        ) as ::core::ffi::c_int
            != 0
    {
        base = &raw mut base_attrs;
    }
    let mut attrs: HlAttrs = dict2hlattrs(val, true_0 != 0, &raw mut link_id, base, err);
    if !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
        let save_current_sctx: sctx_T = api_set_sctx(channel_id);
        ns_hl_def(ns_id as NS, hl_id, attrs, link_id, val);
        current_sctx = save_current_sctx;
    }
}
#[no_mangle]
pub unsafe extern "C" fn nvim_get_hl_ns(
    mut opts: *mut KeyDict_get_ns,
    mut err: *mut Error,
) -> Integer {
    if (*opts).is_set__get_ns_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_ns__winid
        != 0 as ::core::ffi::c_ulonglong
    {
        let mut win: *mut win_T = find_window_by_handle((*opts).winid, err);
        if win.is_null() {
            return 0 as Integer;
        }
        return (*win).w_ns_hl as Integer;
    } else {
        return ns_hl_global as Integer;
    };
}
#[no_mangle]
pub unsafe extern "C" fn nvim_set_hl_ns(mut ns_id: Integer, mut err: *mut Error) {
    if !(ns_id >= 0 as Integer) {
        api_err_invalid(
            err,
            b"namespace\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
            ns_id as int64_t,
            false_0 != 0,
        );
        return;
    }
    ns_hl_global = ns_id as NS;
    hl_check_ns();
    redraw_all_later(UPD_NOT_VALID as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_set_hl_ns_fast(mut ns_id: Integer, mut _err: *mut Error) {
    ns_hl_fast = ns_id as NS;
    hl_check_ns();
}
#[no_mangle]
pub unsafe extern "C" fn nvim_feedkeys(
    mut keys: String_0,
    mut mode: String_0,
    mut escape_ks: Boolean,
) {
    let mut remap: bool = true_0 != 0;
    let mut insert: bool = false_0 != 0;
    let mut typed: bool = false_0 != 0;
    let mut execute: bool = false_0 != 0;
    let mut dangerous: bool = false_0 != 0;
    let mut lowlevel: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < mode.size {
        match *mode.data.offset(i as isize) as ::core::ffi::c_int {
            110 => {
                remap = false_0 != 0;
            }
            109 => {
                remap = true_0 != 0;
            }
            116 => {
                typed = true_0 != 0;
            }
            105 => {
                insert = true_0 != 0;
            }
            120 => {
                execute = true_0 != 0;
            }
            33 => {
                dangerous = true_0 != 0;
            }
            76 => {
                lowlevel = true_0 != 0;
            }
            _ => {}
        }
        i = i.wrapping_add(1);
    }
    if keys.size == 0 as size_t && !execute {
        return;
    }
    let mut keys_esc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if escape_ks {
        keys_esc = vim_strsave_escape_ks(keys.data);
    } else {
        keys_esc = keys.data;
    }
    if lowlevel {
        input_enqueue_raw(keys_esc, strlen(keys_esc));
    } else {
        ins_typebuf(
            keys_esc,
            if remap as ::core::ffi::c_int != 0 {
                REMAP_YES as ::core::ffi::c_int
            } else {
                REMAP_NONE as ::core::ffi::c_int
            },
            if insert as ::core::ffi::c_int != 0 {
                0 as ::core::ffi::c_int
            } else {
                typebuf.tb_len
            },
            !typed,
            false_0 != 0,
        );
        if vgetc_busy != 0 {
            typebuf_was_filled = true_0 != 0;
        }
    }
    if escape_ks {
        xfree(keys_esc as *mut ::core::ffi::c_void);
    }
    if execute {
        let mut save_msg_scroll: ::core::ffi::c_int = msg_scroll;
        msg_scroll = false_0;
        if !dangerous {
            ex_normal_busy += 1;
        }
        exec_normal(true_0 != 0, lowlevel);
        if !dangerous {
            ex_normal_busy -= 1;
        }
        msg_scroll |= save_msg_scroll;
    }
}
#[no_mangle]
pub unsafe extern "C" fn nvim_input(mut channel_id: uint64_t, mut keys: String_0) -> Integer {
    may_trigger_vim_suspend_resume(false_0 != 0);
    return input_enqueue(channel_id, keys) as Integer;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_input_mouse(
    mut button: String_0,
    mut action: String_0,
    mut modifier: String_0,
    mut grid: Integer,
    mut row: Integer,
    mut col: Integer,
    mut err: *mut Error,
) {
    let mut code: ::core::ffi::c_int = 0;
    let mut modmask: ::core::ffi::c_int = 0;
    may_trigger_vim_suspend_resume(false_0 != 0);
    '_error: {
        if !(button.data.is_null() || action.data.is_null()) {
            code = 0 as ::core::ffi::c_int;
            if strequal(
                button.data,
                b"left\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                code = KE_LEFTMOUSE as ::core::ffi::c_int;
            } else if strequal(
                button.data,
                b"middle\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                code = KE_MIDDLEMOUSE as ::core::ffi::c_int;
            } else if strequal(
                button.data,
                b"right\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                code = KE_RIGHTMOUSE as ::core::ffi::c_int;
            } else if strequal(
                button.data,
                b"wheel\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                code = KE_MOUSEDOWN as ::core::ffi::c_int;
            } else if strequal(button.data, b"x1\0".as_ptr() as *const ::core::ffi::c_char) {
                code = KE_X1MOUSE as ::core::ffi::c_int;
            } else if strequal(button.data, b"x2\0".as_ptr() as *const ::core::ffi::c_char) {
                code = KE_X2MOUSE as ::core::ffi::c_int;
            } else if strequal(
                button.data,
                b"move\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                code = KE_MOUSEMOVE as ::core::ffi::c_int;
            } else {
                break '_error;
            }
            if code == KE_MOUSEDOWN as ::core::ffi::c_int {
                if strequal(
                    action.data,
                    b"down\0".as_ptr() as *const ::core::ffi::c_char,
                ) {
                    code = KE_MOUSEUP as ::core::ffi::c_int;
                } else if !strequal(action.data, b"up\0".as_ptr() as *const ::core::ffi::c_char) {
                    if strequal(
                        action.data,
                        b"left\0".as_ptr() as *const ::core::ffi::c_char,
                    ) {
                        code = KE_MOUSERIGHT as ::core::ffi::c_int;
                    } else if strequal(
                        action.data,
                        b"right\0".as_ptr() as *const ::core::ffi::c_char,
                    ) {
                        code = KE_MOUSELEFT as ::core::ffi::c_int;
                    } else {
                        break '_error;
                    }
                }
            } else if code != KE_MOUSEMOVE as ::core::ffi::c_int {
                if !strequal(
                    action.data,
                    b"press\0".as_ptr() as *const ::core::ffi::c_char,
                ) {
                    if strequal(
                        action.data,
                        b"drag\0".as_ptr() as *const ::core::ffi::c_char,
                    ) {
                        code +=
                            KE_LEFTDRAG as ::core::ffi::c_int - KE_LEFTMOUSE as ::core::ffi::c_int;
                    } else if strequal(
                        action.data,
                        b"release\0".as_ptr() as *const ::core::ffi::c_char,
                    ) {
                        code += KE_LEFTRELEASE as ::core::ffi::c_int
                            - KE_LEFTMOUSE as ::core::ffi::c_int;
                    } else {
                        break '_error;
                    }
                }
            }
            modmask = 0 as ::core::ffi::c_int;
            let mut i: size_t = 0 as size_t;
            while i < modifier.size {
                let mut byte: ::core::ffi::c_char = *modifier.data.offset(i as isize);
                if byte as ::core::ffi::c_int != '-' as ::core::ffi::c_int {
                    let mut mod_0: ::core::ffi::c_int =
                        name_to_mod_mask(byte as ::core::ffi::c_int);
                    if !(mod_0 != 0 as ::core::ffi::c_int) {
                        api_set_error(
                            err,
                            kErrorTypeValidation,
                            b"Invalid modifier: %c\0".as_ptr() as *const ::core::ffi::c_char,
                            byte as ::core::ffi::c_int,
                        );
                        return;
                    }
                    modmask |= mod_0;
                }
                i = i.wrapping_add(1);
            }
            input_enqueue_mouse(
                code,
                modmask as uint8_t,
                grid as ::core::ffi::c_int,
                row as ::core::ffi::c_int,
                col as ::core::ffi::c_int,
            );
            return;
        }
    }
    api_set_error(
        err,
        kErrorTypeValidation,
        b"invalid button or action\0".as_ptr() as *const ::core::ffi::c_char,
    );
}
#[no_mangle]
pub unsafe extern "C" fn nvim_replace_termcodes(
    mut str: String_0,
    mut from_part: Boolean,
    mut do_lt: Boolean,
    mut special: Boolean,
) -> String_0 {
    if str.size == 0 as size_t {
        return String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0 as size_t,
        };
    }
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if from_part {
        flags |= REPTERM_FROM_PART as ::core::ffi::c_int;
    }
    if do_lt {
        flags |= REPTERM_DO_LT as ::core::ffi::c_int;
    }
    if !special {
        flags |= REPTERM_NO_SPECIAL as ::core::ffi::c_int;
    }
    let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    replace_termcodes(
        str.data,
        str.size,
        &raw mut ptr,
        0 as scid_T,
        flags,
        ::core::ptr::null_mut::<bool>(),
        p_cpo,
    );
    return cstr_as_string(ptr);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_exec_lua(
    mut code: String_0,
    mut args: Array,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    return nlua_exec(
        code,
        ::core::ptr::null::<::core::ffi::c_char>(),
        args,
        kRetObject,
        arena,
        err,
    );
}
#[no_mangle]
pub unsafe extern "C" fn nvim__exec_lua_fast(
    mut code: String_0,
    mut args: Array,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    return nvim_exec_lua(code, args, arena, err);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_strwidth(mut text: String_0, mut err: *mut Error) -> Integer {
    if !(text.size <= 2147483647 as ::core::ffi::c_int as size_t) {
        api_err_invalid(
            err,
            b"text length\0".as_ptr() as *const ::core::ffi::c_char,
            b"(too long)\0".as_ptr() as *const ::core::ffi::c_char,
            0 as int64_t,
            true_0 != 0,
        );
        return 0 as Integer;
    }
    return mb_string2cells(text.data) as Integer;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_list_runtime_paths(
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    return nvim_get_runtime_file(NULL_STRING, true_0 != 0, arena, err);
}
#[no_mangle]
pub unsafe extern "C" fn nvim__runtime_inspect(mut arena: *mut Arena) -> Array {
    return runtime_inspect(arena);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_get_runtime_file(
    mut name: String_0,
    mut all: Boolean,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    let mut cookie: RuntimeCookie = RuntimeCookie {
        rv: ArrayBuilder {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
            init_array: [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            }; 16],
        },
        arena: arena,
    };
    cookie.rv.capacity = ::core::mem::size_of::<[Object; 16]>()
        .wrapping_div(::core::mem::size_of::<Object>())
        .wrapping_div(
            (::core::mem::size_of::<[Object; 16]>().wrapping_rem(::core::mem::size_of::<Object>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    cookie.rv.size = 0 as size_t;
    cookie.rv.items = &raw mut cookie.rv.init_array as *mut Object;
    let mut flags: ::core::ffi::c_int = DIP_DIRFILE as ::core::ffi::c_int
        | (if all as ::core::ffi::c_int != 0 {
            DIP_ALL as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        });
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
    do_in_runtimepath(
        (if name.size != 0 {
            name.data as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        }) as *mut ::core::ffi::c_char,
        flags,
        Some(
            find_runtime_cb
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *mut ::core::ffi::c_char,
                    bool,
                    *mut ::core::ffi::c_void,
                ) -> bool,
        ),
        &raw mut cookie as *mut ::core::ffi::c_void,
    );
    try_leave(&raw mut tstate, err);
    return arena_take_arraybuilder(arena, &raw mut cookie.rv);
}
unsafe extern "C" fn find_runtime_cb(
    mut num_fnames: ::core::ffi::c_int,
    mut fnames: *mut *mut ::core::ffi::c_char,
    mut all: bool,
    mut c: *mut ::core::ffi::c_void,
) -> bool {
    let mut cookie: *mut RuntimeCookie = c as *mut RuntimeCookie;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num_fnames {
        if (*cookie).rv.size == (*cookie).rv.capacity {
            (*cookie).rv.capacity = if (*cookie).rv.capacity << 1 as ::core::ffi::c_int
                > ::core::mem::size_of::<[Object; 16]>()
                    .wrapping_div(::core::mem::size_of::<Object>())
                    .wrapping_div(
                        (::core::mem::size_of::<[Object; 16]>()
                            .wrapping_rem(::core::mem::size_of::<Object>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                (*cookie).rv.capacity << 1 as ::core::ffi::c_int
            } else {
                ::core::mem::size_of::<[Object; 16]>()
                    .wrapping_div(::core::mem::size_of::<Object>())
                    .wrapping_div(
                        (::core::mem::size_of::<[Object; 16]>()
                            .wrapping_rem(::core::mem::size_of::<Object>())
                            == 0) as ::core::ffi::c_int as size_t,
                    )
            };
            (*cookie).rv.items = (if (*cookie).rv.capacity
                == ::core::mem::size_of::<[Object; 16]>()
                    .wrapping_div(::core::mem::size_of::<Object>())
                    .wrapping_div(
                        (::core::mem::size_of::<[Object; 16]>()
                            .wrapping_rem(::core::mem::size_of::<Object>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                if (*cookie).rv.items == &raw mut (*cookie).rv.init_array as *mut Object {
                    (*cookie).rv.items as *mut ::core::ffi::c_void
                } else {
                    _memcpy_free(
                        &raw mut (*cookie).rv.init_array as *mut Object as *mut ::core::ffi::c_void,
                        (*cookie).rv.items as *mut ::core::ffi::c_void,
                        (*cookie)
                            .rv
                            .size
                            .wrapping_mul(::core::mem::size_of::<Object>()),
                    )
                }
            } else {
                if (*cookie).rv.items == &raw mut (*cookie).rv.init_array as *mut Object {
                    memcpy(
                        xmalloc(
                            (*cookie)
                                .rv
                                .capacity
                                .wrapping_mul(::core::mem::size_of::<Object>()),
                        ),
                        (*cookie).rv.items as *const ::core::ffi::c_void,
                        (*cookie)
                            .rv
                            .size
                            .wrapping_mul(::core::mem::size_of::<Object>()),
                    )
                } else {
                    xrealloc(
                        (*cookie).rv.items as *mut ::core::ffi::c_void,
                        (*cookie)
                            .rv
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<Object>()),
                    )
                }
            }) as *mut Object;
        } else {
        };
        let c2rust_fresh0 = (*cookie).rv.size;
        (*cookie).rv.size = (*cookie).rv.size.wrapping_add(1);
        *(*cookie).rv.items.offset(c2rust_fresh0 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: arena_string((*cookie).arena, cstr_as_string(*fnames.offset(i as isize))),
            },
        };
        if !all {
            return true_0 != 0;
        }
        i += 1;
    }
    return num_fnames > 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn nvim__get_lib_dir() -> String_0 {
    return cstr_as_string(get_lib_dir());
}
#[no_mangle]
pub unsafe extern "C" fn nvim__get_runtime(
    mut pat: Array,
    mut all: Boolean,
    mut opts: *mut KeyDict_runtime,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    if !(!(*opts).do_source || nlua_is_deferred_safe() as ::core::ffi::c_int != 0) {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            b"'do_source' used in fast callback\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
    }
    let mut res: Array = runtime_get_named((*opts).is_lua as bool, pat, all as bool, arena);
    if (*opts).do_source {
        let mut i: size_t = 0 as size_t;
        while i < res.size {
            let mut name: String_0 = (*res.items.offset(i as isize)).data.string;
            do_source(
                name.data,
                false_0 != 0,
                DOSO_NONE as ::core::ffi::c_int,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
            );
            i = i.wrapping_add(1);
        }
    }
    return res;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_set_current_dir(mut dir: String_0, mut err: *mut Error) {
    if !(dir.size < 4096 as size_t) {
        api_err_invalid(
            err,
            b"directory name\0".as_ptr() as *const ::core::ffi::c_char,
            b"(too long)\0".as_ptr() as *const ::core::ffi::c_char,
            0 as int64_t,
            true_0 != 0,
        );
        return;
    }
    let mut string: [::core::ffi::c_char; 4096] = [0; 4096];
    memcpy(
        &raw mut string as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        dir.data as *const ::core::ffi::c_void,
        dir.size,
    );
    string[dir.size as usize] = NUL as ::core::ffi::c_char;
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
    changedir_func(&raw mut string as *mut ::core::ffi::c_char, kCdScopeGlobal);
    try_leave(&raw mut tstate, err);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_get_current_line(
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> String_0 {
    return buffer_get_line(
        (*curbuf).handle as Buffer,
        ((*curwin).w_cursor.lnum - 1 as linenr_T) as Integer,
        arena,
        err,
    );
}
#[no_mangle]
pub unsafe extern "C" fn nvim_set_current_line(
    mut line: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) {
    buffer_set_line(
        (*curbuf).handle as Buffer,
        ((*curwin).w_cursor.lnum - 1 as linenr_T) as Integer,
        line,
        arena,
        err,
    );
}
#[no_mangle]
pub unsafe extern "C" fn nvim_del_current_line(mut arena: *mut Arena, mut err: *mut Error) {
    buffer_del_line(
        (*curbuf).handle as Buffer,
        ((*curwin).w_cursor.lnum - 1 as linenr_T) as Integer,
        arena,
        err,
    );
}
#[no_mangle]
pub unsafe extern "C" fn nvim_get_var(
    mut name: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    let mut di: *mut dictitem_T =
        tv_dict_find(get_globvar_dict(), name.data, name.size as ptrdiff_t);
    if di.is_null() {
        let mut found: bool =
            script_autoload(name.data, name.size, false_0 != 0) as ::core::ffi::c_int != 0
                && !aborting();
        if !found {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"Key not found: %s\0".as_ptr() as *const ::core::ffi::c_char,
                name.data,
            );
            return object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            };
        }
        di = tv_dict_find(get_globvar_dict(), name.data, name.size as ptrdiff_t);
    }
    if di.is_null() {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"Key not found: %s\0".as_ptr() as *const ::core::ffi::c_char,
            name.data,
        );
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    return vim_to_object(&raw mut (*di).di_tv, arena, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_set_var(mut name: String_0, mut value: Object, mut err: *mut Error) {
    dict_set_var(
        get_globvar_dict(),
        name,
        value,
        false_0 != 0,
        false_0 != 0,
        ::core::ptr::null_mut::<Arena>(),
        err,
    );
}
#[no_mangle]
pub unsafe extern "C" fn nvim_del_var(mut name: String_0, mut err: *mut Error) {
    dict_set_var(
        get_globvar_dict(),
        name,
        object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        true_0 != 0,
        false_0 != 0,
        ::core::ptr::null_mut::<Arena>(),
        err,
    );
}
#[no_mangle]
pub unsafe extern "C" fn nvim_get_vvar(
    mut name: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    return dict_get_value(get_vimvar_dict(), name, arena, err);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_set_vvar(mut name: String_0, mut value: Object, mut err: *mut Error) {
    dict_set_var(
        get_vimvar_dict(),
        name,
        value,
        false_0 != 0,
        false_0 != 0,
        ::core::ptr::null_mut::<Arena>(),
        err,
    );
}
#[no_mangle]
pub unsafe extern "C" fn nvim_echo(
    mut chunks: Array,
    mut history: Boolean,
    mut opts: *mut KeyDict_echo_opts,
    mut err: *mut Error,
) -> Object {
    let mut kind: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut is_progress: bool = false;
    let mut needs_clear: bool = false;
    let mut msg_data: MessageData = MessageData {
        source: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        percent: 0,
        title: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        status: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        data: Dict {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        },
    };
    let mut save_nwr: bool = false;
    let mut save_lines_left: ::core::ffi::c_int = 0;
    let mut save_msg_didany: bool = false;
    let mut id: Object = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: -1 as Integer,
        },
    };
    let mut hl_msg: HlMessage = parse_hl_msg(chunks, (*opts).err as bool, err);
    if (*err).type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
        kind = (*opts).kind.data;
        if (*opts).verbose {
            verbose_enter();
        } else if kind.is_null() {
            kind = (if (*opts).err as ::core::ffi::c_int != 0 {
                b"echoerr\0".as_ptr() as *const ::core::ffi::c_char
            } else if history as ::core::ffi::c_int != 0 {
                b"echomsg\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"echo\0".as_ptr() as *const ::core::ffi::c_char
            }) as *mut ::core::ffi::c_char;
        }
        is_progress = strequal(kind, b"progress\0".as_ptr() as *const ::core::ffi::c_char);
        needs_clear = !history;
        if !(is_progress as ::core::ffi::c_int != 0
            || (*opts).status.size == 0 as size_t
                && (*opts).title.size == 0 as size_t
                && (*opts).percent == 0 as Integer
                && (*opts).data.size == 0 as size_t
                && (*opts).source.size == 0 as size_t)
        {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"Conflict: title/source/status/percent/data not allowed with kind='%s'\0".as_ptr()
                    as *const ::core::ffi::c_char,
                kind,
            );
        } else if !(!is_progress
            || strequal(
                (*opts).status.data,
                b"success\0".as_ptr() as *const ::core::ffi::c_char,
            ) as ::core::ffi::c_int
                != 0
            || strequal(
                (*opts).status.data,
                b"failed\0".as_ptr() as *const ::core::ffi::c_char,
            ) as ::core::ffi::c_int
                != 0
            || strequal(
                (*opts).status.data,
                b"running\0".as_ptr() as *const ::core::ffi::c_char,
            ) as ::core::ffi::c_int
                != 0
            || strequal(
                (*opts).status.data,
                b"cancel\0".as_ptr() as *const ::core::ffi::c_char,
            ) as ::core::ffi::c_int
                != 0)
        {
            api_err_exp(
                err,
                b"status\0".as_ptr() as *const ::core::ffi::c_char,
                b"success|failed|running|cancel\0".as_ptr() as *const ::core::ffi::c_char,
                (*opts).status.data,
            );
        } else if !(!is_progress
            || (*opts).percent >= 0 as Integer && (*opts).percent <= 100 as Integer)
        {
            api_err_invalid(
                err,
                b"percent\0".as_ptr() as *const ::core::ffi::c_char,
                b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                0 as int64_t,
                false_0 != 0,
            );
        } else if !(!is_progress || (*opts).source.size != 0 as size_t) {
            api_err_required(err, b"opts.source\0".as_ptr() as *const ::core::ffi::c_char);
        } else if !((*opts).id.type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
            || msg_id_exists((*opts).id.data.integer as int64_t) as ::core::ffi::c_int != 0)
        {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"Invalid 'id': %ld\0".as_ptr() as *const ::core::ffi::c_char,
                (*opts).id.data.integer,
            );
        } else {
            msg_data = msg_data {
                source: (*opts).source,
                percent: (*opts).percent,
                title: (*opts).title,
                status: (*opts).status,
                data: (*opts).data,
            };
            save_nwr = need_wait_return;
            save_lines_left = lines_left;
            save_msg_didany = msg_didany;
            if (*opts)._truncate {
                no_wait_return += 1;
                lines_left = 0 as ::core::ffi::c_int;
                msg_didany = true_0 != 0;
                msg_no_more = true_0 != 0;
            }
            id = msg_multihl(
                (*opts).id,
                hl_msg,
                kind,
                history as bool,
                (*opts).err as bool,
                &raw mut msg_data,
                &raw mut needs_clear,
            );
            if (*opts)._truncate {
                msg_no_more = false_0 != 0;
                msg_didany = save_msg_didany;
                lines_left = save_lines_left;
                no_wait_return -= 1;
                need_wait_return = save_nwr;
            }
            if (*opts).verbose {
                verbose_leave();
                verbose_stop();
            }
            if is_progress {
                do_autocmd_progress(id, hl_msg, &raw mut msg_data);
            }
            if !needs_clear {
                return id;
            }
        }
    }
    hl_msg_free(hl_msg);
    return id;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_list_bufs(mut arena: *mut Arena) -> Array {
    let mut n: size_t = 0 as size_t;
    let mut b: *mut buf_T = firstbuf;
    while !b.is_null() {
        n = n.wrapping_add(1);
        b = (*b).b_next;
    }
    let mut rv: Array = arena_array(arena, n);
    let mut b_0: *mut buf_T = firstbuf;
    while !b_0.is_null() {
        let c2rust_fresh1 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh1 as isize) = object {
            type_0: kObjectTypeBuffer,
            data: C2Rust_Unnamed {
                integer: (*b_0).handle as Integer,
            },
        };
        b_0 = (*b_0).b_next;
    }
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_get_current_buf() -> Buffer {
    return (*curbuf).handle as Buffer;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_set_current_buf(mut buf: Buffer, mut err: *mut Error) {
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return;
    }
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
    do_buffer(
        DOBUF_GOTO as ::core::ffi::c_int,
        DOBUF_FIRST as ::core::ffi::c_int,
        FORWARD as ::core::ffi::c_int,
        (*b).handle as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    try_leave(&raw mut tstate, err);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_list_wins(mut arena: *mut Arena) -> Array {
    let mut n: size_t = 0 as size_t;
    let mut tp: *mut tabpage_T = first_tabpage as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab {
            firstwin
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            n = n.wrapping_add(1);
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    let mut rv: Array = arena_array(arena, n);
    let mut tp_0: *mut tabpage_T = first_tabpage as *mut tabpage_T;
    while !tp_0.is_null() {
        let mut wp_0: *mut win_T = if tp_0 == curtab {
            firstwin
        } else {
            (*tp_0).tp_firstwin
        };
        while !wp_0.is_null() {
            let c2rust_fresh2 = rv.size;
            rv.size = rv.size.wrapping_add(1);
            *rv.items.offset(c2rust_fresh2 as isize) = object {
                type_0: kObjectTypeWindow,
                data: C2Rust_Unnamed {
                    integer: (*wp_0).handle as Integer,
                },
            };
            wp_0 = (*wp_0).w_next;
        }
        tp_0 = (*tp_0).tp_next as *mut tabpage_T;
    }
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_get_current_win() -> Window {
    return (*curwin).handle as Window;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_set_current_win(mut win: Window, mut err: *mut Error) {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return;
    }
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
    if (*w).w_buffer != curbuf {
        reset_VIsual_and_resel();
    }
    goto_tabpage_win(win_find_tabpage(w), w);
    try_leave(&raw mut tstate, err);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_create_buf(
    mut listed: Boolean,
    mut scratch: Boolean,
    mut err: *mut Error,
) -> Buffer {
    let mut ret: Buffer = 0 as Buffer;
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
    block_autocmds();
    let mut buf: *mut buf_T = buflist_new(
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        0 as linenr_T,
        BLN_NOOPT as ::core::ffi::c_int
            | BLN_NEW as ::core::ffi::c_int
            | (if listed as ::core::ffi::c_int != 0 {
                BLN_LISTED as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }),
    );
    if buf.is_null() {
        unblock_autocmds();
    } else if ml_open(buf) == 0 as ::core::ffi::c_int {
        unblock_autocmds();
    } else {
        (*buf).b_last_changedtick = buf_get_changedtick(buf);
        (*buf).b_last_changedtick_i = buf_get_changedtick(buf);
        (*buf).b_last_changedtick_pum = buf_get_changedtick(buf);
        buf_copy_options(
            buf,
            BCO_ENTER as ::core::ffi::c_int | BCO_NOHELP as ::core::ffi::c_int,
        );
        if scratch {
            set_option_direct_for(
                kOptBufhidden,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: String_0 {
                            data: b"hide\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                                .wrapping_sub(1 as size_t),
                        },
                    },
                },
                OPT_LOCAL as ::core::ffi::c_int,
                0 as scid_T,
                kOptScopeBuf,
                buf as *mut ::core::ffi::c_void,
            );
            set_option_direct_for(
                kOptBuftype,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: String_0 {
                            data: b"nofile\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                                .wrapping_sub(1 as size_t),
                        },
                    },
                },
                OPT_LOCAL as ::core::ffi::c_int,
                0 as scid_T,
                kOptScopeBuf,
                buf as *mut ::core::ffi::c_void,
            );
            '_c2rust_label: {
                if (*(*buf).b_ml.ml_mfp).mf_fd < 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"buf->b_ml.ml_mfp->mf_fd < 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/api/vim.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        1077 as ::core::ffi::c_uint,
                        b"Buffer nvim_create_buf(Boolean, Boolean, Error *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            (*buf).b_p_swf = 0 as ::core::ffi::c_int;
            (*buf).b_p_ml = 0 as ::core::ffi::c_int;
        }
        unblock_autocmds();
        let mut bufref: bufref_T = bufref_T {
            br_buf: ::core::ptr::null_mut::<buf_T>(),
            br_fnum: 0,
            br_buf_free_count: 0,
        };
        set_bufref(&raw mut bufref, buf);
        if !(apply_autocmds(
            EVENT_BUFNEW,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false,
            buf,
        ) as ::core::ffi::c_int
            != 0
            && !bufref_valid(&raw mut bufref))
        {
            if !(listed as ::core::ffi::c_int != 0
                && apply_autocmds(
                    EVENT_BUFADD,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    false,
                    buf,
                ) as ::core::ffi::c_int
                    != 0
                && !bufref_valid(&raw mut bufref))
            {
                ret = (*buf).handle as Buffer;
            }
        }
    }
    try_leave(&raw mut tstate, err);
    if ret == 0 as ::core::ffi::c_int
        && !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int)
    {
        api_set_error(
            err,
            kErrorTypeException,
            b"Failed to create buffer\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_open_term(
    mut buf: Buffer,
    mut opts: *mut KeyDict_open_term,
    mut err: *mut Error,
) -> Integer {
    let mut b: *mut buf_T = api_buf_ensure_loaded(buf, err);
    if b.is_null() {
        return 0 as Integer;
    }
    if b == cmdwin_buf {
        api_set_error(
            err,
            kErrorTypeException,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            &raw const e_cmdwin as *const ::core::ffi::c_char,
        );
        return 0 as Integer;
    }
    let mut may_read_buffer: bool = true_0 != 0;
    if !(*b).terminal.is_null() {
        if terminal_running((*b).terminal) {
            api_set_error(
                err,
                kErrorTypeException,
                b"Terminal already connected to buffer %d\0".as_ptr() as *const ::core::ffi::c_char,
                (*b).handle,
            );
            return 0 as Integer;
        }
        buf_close_terminal(b);
        may_read_buffer = false_0 != 0;
    }
    let mut cb: LuaRef = LUA_NOREF;
    if (*opts).is_set__open_term_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_open_term__on_input
        != 0 as ::core::ffi::c_ulonglong
    {
        cb = (*opts).on_input;
        (*opts).on_input = LUA_NOREF as LuaRef;
    }
    let mut chan: *mut Channel = channel_alloc(kChannelStreamInternal);
    (*chan).stream.internal.cb = cb;
    (*chan).stream.internal.closed = false_0 != 0;
    let mut topts: TerminalOptions = TerminalOptions {
        data: chan as *mut ::core::ffi::c_void,
        width: (if (*curwin).w_view_width - win_col_off(curwin) > 0 as ::core::ffi::c_int {
            (*curwin).w_view_width - win_col_off(curwin)
        } else {
            0 as ::core::ffi::c_int
        }) as uint16_t,
        height: (*curwin).w_view_height as uint16_t,
        read_pause_cb: Some(
            term_read_pause as unsafe extern "C" fn(bool, *mut ::core::ffi::c_void) -> (),
        ),
        write_cb: Some(
            term_write
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_char,
                    size_t,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        resize_cb: Some(
            term_resize as unsafe extern "C" fn(uint16_t, uint16_t, *mut ::core::ffi::c_void) -> (),
        ),
        resume_cb: Some(term_resume as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
        close_cb: Some(term_close as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
        force_crlf: if (*opts).is_set__open_term_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_open_term__force_crlf
            != 0 as ::core::ffi::c_ulonglong
        {
            (*opts).force_crlf as ::core::ffi::c_int
        } else {
            true_0
        } != 0,
    };
    let mut contents: StringBuilder = StringBuilder {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    if may_read_buffer {
        read_buffer_into(b, 1 as linenr_T, (*b).b_ml.ml_line_count, &raw mut contents);
    }
    channel_incref(chan);
    (*chan).term = terminal_alloc(b, topts);
    terminal_open(&raw mut (*chan).term, b);
    if !(*chan).term.is_null() {
        terminal_check_size((*chan).term);
    }
    channel_decref(chan);
    if contents.size > 0 as size_t {
        let mut error: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        channel_send(
            (*chan).id,
            contents.items,
            contents.size,
            true_0 != 0,
            &raw mut error,
        );
        if !error.is_null() {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                error,
            );
        }
    }
    return (*chan).id as Integer;
}
unsafe extern "C" fn term_read_pause(mut _pause: bool, mut _data: *mut ::core::ffi::c_void) {}
unsafe extern "C" fn term_write(
    mut buf: *const ::core::ffi::c_char,
    mut size: size_t,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut chan: *mut Channel = data as *mut Channel;
    let mut cb: LuaRef = (*chan).stream.internal.cb;
    if cb == LUA_NOREF {
        return;
    }
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 3] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 3];
    args.capacity = 3 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh3 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh3 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: (*chan).id as Integer,
        },
    };
    let c2rust_fresh4 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh4 as isize) = object {
        type_0: kObjectTypeBuffer,
        data: C2Rust_Unnamed {
            integer: terminal_buf((*chan).term) as Integer,
        },
    };
    let c2rust_fresh5 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh5 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed {
            string: String_0 {
                data: buf as *mut ::core::ffi::c_char,
                size: size,
            },
        },
    };
    textlock += 1;
    nlua_call_ref(
        cb,
        b"input\0".as_ptr() as *const ::core::ffi::c_char,
        args,
        kRetNilBool,
        ::core::ptr::null_mut::<Arena>(),
        ::core::ptr::null_mut::<Error>(),
    );
    textlock -= 1;
}
unsafe extern "C" fn term_resize(
    mut _width: uint16_t,
    mut _height: uint16_t,
    mut _data: *mut ::core::ffi::c_void,
) {
}
unsafe extern "C" fn term_resume(mut _data: *mut ::core::ffi::c_void) {}
unsafe extern "C" fn term_close(mut data: *mut ::core::ffi::c_void) {
    let mut chan: *mut Channel = data as *mut Channel;
    terminal_destroy(&raw mut (*chan).term);
    api_free_luaref((*chan).stream.internal.cb);
    (*chan).stream.internal.cb = LUA_NOREF as LuaRef;
    channel_decref(chan);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_chan_send(
    mut chan: Integer,
    mut data: String_0,
    mut err: *mut Error,
) {
    let mut error: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if data.size == 0 {
        return;
    }
    channel_send(
        chan as uint64_t,
        data.data,
        data.size,
        false_0 != 0,
        &raw mut error,
    );
    if !error.is_null() {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            error,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn nvim_list_tabpages(mut arena: *mut Arena) -> Array {
    let mut n: size_t = 0 as size_t;
    let mut tp: *mut tabpage_T = first_tabpage as *mut tabpage_T;
    while !tp.is_null() {
        n = n.wrapping_add(1);
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    let mut rv: Array = arena_array(arena, n);
    let mut tp_0: *mut tabpage_T = first_tabpage as *mut tabpage_T;
    while !tp_0.is_null() {
        let c2rust_fresh6 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh6 as isize) = object {
            type_0: kObjectTypeTabpage,
            data: C2Rust_Unnamed {
                integer: (*tp_0).handle as Integer,
            },
        };
        tp_0 = (*tp_0).tp_next as *mut tabpage_T;
    }
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_get_current_tabpage() -> Tabpage {
    return (*curtab).handle as Tabpage;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_set_current_tabpage(mut tabpage: Tabpage, mut err: *mut Error) {
    let mut tp: *mut tabpage_T = find_tab_by_handle(tabpage, err);
    if tp.is_null() {
        return;
    }
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
    goto_tabpage_tp(tp, true, true);
    try_leave(&raw mut tstate, err);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_paste(
    mut channel_id: uint64_t,
    mut data: String_0,
    mut crlf: Boolean,
    mut phase: Integer,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Boolean {
    let mut lines: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 2] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 2];
    let mut rv: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    static mut cancelled: bool = false_0 != 0;
    if !(phase >= -1 as Integer && phase <= 3 as Integer) {
        api_err_invalid(
            err,
            b"phase\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
            phase as int64_t,
            false_0 != 0,
        );
        return false;
    }
    's_151: {
        if phase == -1 as Integer || phase == 1 as Integer {
            cancelled = false_0 != 0;
            if !(*curbuf).terminal.is_null() {
                terminal_set_streamed_paste((*curbuf).terminal, true_0 != 0);
            }
        } else if cancelled {
            break 's_151;
        }
        lines = string_to_array(data, crlf as bool, arena);
        args = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        args__items = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 2];
        args.capacity = 2 as size_t;
        args.items = &raw mut args__items as *mut Object;
        let c2rust_fresh7 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh7 as isize) = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed { array: lines },
        };
        let c2rust_fresh8 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh8 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: phase },
        };
        rv = nlua_exec(
            String_0 {
                data: b"return vim.paste(...)\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 22]>().wrapping_sub(1 as size_t),
            },
            ::core::ptr::null::<::core::ffi::c_char>(),
            args,
            kRetNilBool,
            arena,
            err,
        );
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
            || rv.type_0 as ::core::ffi::c_uint
                == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
                && !rv.data.boolean
        {
            cancelled = true_0 != 0;
        }
        if (phase == -1 as Integer || phase == 3 as Integer || cancelled as ::core::ffi::c_int != 0)
            && !(*curbuf).terminal.is_null()
        {
            terminal_set_streamed_paste((*curbuf).terminal, false_0 != 0);
        }
        if !cancelled && (phase == -1 as Integer || phase == 1 as Integer) {
            paste_store(channel_id, kFalse, NULL_STRING, crlf as bool);
        }
        if !cancelled {
            paste_store(channel_id, kNone, data, crlf as bool);
        }
        if phase == 3 as Integer
            || phase
                == (if cancelled as ::core::ffi::c_int != 0 {
                    2 as ::core::ffi::c_int
                } else {
                    -1 as ::core::ffi::c_int
                }) as Integer
        {
            paste_store(channel_id, kTrue, NULL_STRING, crlf as bool);
        }
    }
    let mut retval: bool = !cancelled;
    if phase == -1 as Integer || phase == 3 as Integer {
        cancelled = false_0 != 0;
    }
    return retval as Boolean;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_put(
    mut lines: Array,
    mut type_0: String_0,
    mut after: Boolean,
    mut follow: Boolean,
    mut arena: *mut Arena,
    mut err: *mut Error,
) {
    let mut reg: [yankreg_T; 1] = [yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    }];
    if !prepare_yankreg_from_object(&raw mut reg as *mut yankreg_T, type_0, lines.size) {
        api_err_invalid(
            err,
            b"type\0".as_ptr() as *const ::core::ffi::c_char,
            type_0.data,
            0 as int64_t,
            true_0 != 0,
        );
        return;
    }
    if lines.size == 0 as size_t {
        return;
    }
    (*(&raw mut reg as *mut yankreg_T)).y_array = arena_alloc(
        arena,
        lines.size.wrapping_mul(::core::mem::size_of::<String_0>()),
        true_0 != 0,
    ) as *mut String_0;
    (*(&raw mut reg as *mut yankreg_T)).y_size = lines.size;
    let mut i: size_t = 0 as size_t;
    while i < lines.size {
        if kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
            != (*lines.items.offset(i as isize)).type_0 as ::core::ffi::c_uint
        {
            api_err_exp(
                err,
                b"line\0".as_ptr() as *const ::core::ffi::c_char,
                api_typename(kObjectTypeString),
                api_typename((*lines.items.offset(i as isize)).type_0),
            );
            return;
        }
        let mut line: String_0 = (*lines.items.offset(i as isize)).data.string;
        *(*(&raw mut reg as *mut yankreg_T))
            .y_array
            .offset(i as isize) = copy_string(line, arena);
        memchrsub(
            (*(*(&raw mut reg as *mut yankreg_T))
                .y_array
                .offset(i as isize))
            .data as *mut ::core::ffi::c_void,
            NUL as ::core::ffi::c_char,
            NL as ::core::ffi::c_char,
            line.size,
        );
        i = i.wrapping_add(1);
    }
    finish_yankreg_from_object(&raw mut reg as *mut yankreg_T, false_0 != 0);
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
    let mut VIsual_was_active: bool = VIsual_active;
    msg_silent += 1;
    do_put(
        0 as ::core::ffi::c_int,
        &raw mut reg as *mut yankreg_T,
        if after as ::core::ffi::c_int != 0 {
            FORWARD as ::core::ffi::c_int
        } else {
            BACKWARD as ::core::ffi::c_int
        },
        1 as ::core::ffi::c_int,
        if follow as ::core::ffi::c_int != 0 {
            PUT_CURSEND as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        },
    );
    msg_silent -= 1;
    VIsual_active = VIsual_was_active;
    try_leave(&raw mut tstate, err);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_get_color_by_name(mut name: String_0) -> Integer {
    let mut dummy: ::core::ffi::c_int = 0;
    return name_to_color(name.data, &raw mut dummy) as Integer;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_get_color_map(mut arena: *mut Arena) -> Dict {
    let mut colors: Dict = arena_dict(
        arena,
        ::core::mem::size_of::<[color_name_table_T; 708]>()
            .wrapping_div(::core::mem::size_of::<color_name_table_T>())
            .wrapping_div(
                (::core::mem::size_of::<[color_name_table_T; 708]>()
                    .wrapping_rem(::core::mem::size_of::<color_name_table_T>())
                    == 0) as ::core::ffi::c_int as size_t,
            ),
    );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while !color_name_table[i as usize].name.is_null() {
        let c2rust_fresh9 = colors.size;
        colors.size = colors.size.wrapping_add(1);
        *colors.items.offset(c2rust_fresh9 as isize) = key_value_pair {
            key: cstr_as_string(color_name_table[i as usize].name),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: color_name_table[i as usize].color as Integer,
                },
            },
        };
        i += 1;
    }
    return colors;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_get_context(
    mut opts: *mut KeyDict_context,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    let mut types: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    if (*opts).is_set__context_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_context__types
        != 0 as ::core::ffi::c_ulonglong
    {
        types = (*opts).types;
    }
    let mut int_types: ::core::ffi::c_int = if types.size > 0 as size_t {
        0 as ::core::ffi::c_int
    } else {
        kCtxAll
    };
    if types.size > 0 as size_t {
        let mut i: size_t = 0 as size_t;
        while i < types.size {
            if (*types.items.offset(i as isize)).type_0 as ::core::ffi::c_uint
                == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let s: *const ::core::ffi::c_char =
                    (*types.items.offset(i as isize)).data.string.data;
                if strequal(s, b"regs\0".as_ptr() as *const ::core::ffi::c_char) {
                    int_types |= kCtxRegs as ::core::ffi::c_int;
                } else if strequal(s, b"jumps\0".as_ptr() as *const ::core::ffi::c_char) {
                    int_types |= kCtxJumps as ::core::ffi::c_int;
                } else if strequal(s, b"bufs\0".as_ptr() as *const ::core::ffi::c_char) {
                    int_types |= kCtxBufs as ::core::ffi::c_int;
                } else if strequal(s, b"gvars\0".as_ptr() as *const ::core::ffi::c_char) {
                    int_types |= kCtxGVars as ::core::ffi::c_int;
                } else if strequal(s, b"sfuncs\0".as_ptr() as *const ::core::ffi::c_char) {
                    int_types |= kCtxSFuncs as ::core::ffi::c_int;
                } else if strequal(s, b"funcs\0".as_ptr() as *const ::core::ffi::c_char) {
                    int_types |= kCtxFuncs as ::core::ffi::c_int;
                } else if true {
                    api_err_invalid(
                        err,
                        b"type\0".as_ptr() as *const ::core::ffi::c_char,
                        s,
                        0 as int64_t,
                        true_0 != 0,
                    );
                    return Dict {
                        size: 0 as size_t,
                        capacity: 0 as size_t,
                        items: ::core::ptr::null_mut::<KeyValuePair>(),
                    };
                }
            }
            i = i.wrapping_add(1);
        }
    }
    let mut ctx: Context = CONTEXT_INIT;
    ctx_save(&raw mut ctx, int_types);
    let mut dict: Dict = ctx_to_dict(&raw mut ctx, arena);
    ctx_free(&raw mut ctx);
    return dict;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_load_context(mut dict: Dict, mut err: *mut Error) -> Object {
    let mut ctx: Context = CONTEXT_INIT;
    let mut save_did_emsg: ::core::ffi::c_int = did_emsg;
    did_emsg = false_0;
    ctx_from_dict(dict, &raw mut ctx, err);
    if !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
        ctx_restore(&raw mut ctx, kCtxAll);
    }
    ctx_free(&raw mut ctx);
    did_emsg = save_did_emsg;
    return object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
}
#[no_mangle]
pub unsafe extern "C" fn nvim_get_mode(mut arena: *mut Arena) -> Dict {
    let mut rv: Dict = arena_dict(arena, 2 as size_t);
    let mut modestr: *mut ::core::ffi::c_char =
        arena_alloc(arena, MODE_MAX_LENGTH as size_t, false_0 != 0) as *mut ::core::ffi::c_char;
    get_mode(modestr);
    let mut blocked: bool = input_blocking();
    let c2rust_fresh10 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh10 as isize) = key_value_pair {
        key: cstr_as_string(b"mode\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string(modestr),
            },
        },
    };
    let c2rust_fresh11 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh11 as isize) = key_value_pair {
        key: cstr_as_string(b"blocking\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeBoolean,
            data: C2Rust_Unnamed { boolean: blocked },
        },
    };
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_get_keymap(mut mode: String_0, mut arena: *mut Arena) -> Array {
    return keymap_array(mode, ::core::ptr::null_mut::<buf_T>(), arena);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_set_keymap(
    mut channel_id: uint64_t,
    mut mode: String_0,
    mut lhs: String_0,
    mut rhs: String_0,
    mut opts: *mut KeyDict_keymap,
    mut err: *mut Error,
) {
    modify_keymap(
        channel_id,
        -1 as Buffer,
        false_0 != 0,
        mode,
        lhs,
        rhs,
        opts,
        err,
    );
}
#[no_mangle]
pub unsafe extern "C" fn nvim_del_keymap(
    mut channel_id: uint64_t,
    mut mode: String_0,
    mut lhs: String_0,
    mut err: *mut Error,
) {
    nvim_buf_del_keymap(channel_id, -1 as Buffer, mode, lhs, err);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_get_api_info(
    mut channel_id: uint64_t,
    mut arena: *mut Arena,
) -> Array {
    let mut rv: Array = arena_array(arena, 2 as size_t);
    '_c2rust_label: {
        if channel_id <= 9223372036854775807 as uint64_t {
        } else {
            __assert_fail(
                b"channel_id <= INT64_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/api/vim.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1658 as ::core::ffi::c_uint,
                b"Array nvim_get_api_info(uint64_t, Arena *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let c2rust_fresh12 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh12 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: channel_id as int64_t,
        },
    };
    let c2rust_fresh13 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh13 as isize) = api_metadata();
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_set_client_info(
    mut channel_id: uint64_t,
    mut name: String_0,
    mut version: Dict,
    mut type_0: String_0,
    mut methods: Dict,
    mut attributes: Dict,
    mut arena: *mut Arena,
    mut _err: *mut Error,
) {
    let mut info: Dict = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut info__items: [KeyValuePair; 5] = [KeyValuePair {
        key: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        value: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
    }; 5];
    info.capacity = 5 as size_t;
    info.items = &raw mut info__items as *mut KeyValuePair;
    let c2rust_fresh14 = info.size;
    info.size = info.size.wrapping_add(1);
    *info.items.offset(c2rust_fresh14 as isize) = key_value_pair {
        key: cstr_as_string(b"name\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed { string: name },
        },
    };
    let mut has_major: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < version.size {
        if strequal(
            (*version.items.offset(i as isize)).key.data,
            b"major\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            has_major = true_0 != 0;
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }
    if !has_major {
        let mut v: Dict = arena_dict(arena, version.size.wrapping_add(1 as size_t));
        if version.size != 0 {
            memcpy(
                v.items as *mut ::core::ffi::c_void,
                version.items as *const ::core::ffi::c_void,
                version
                    .size
                    .wrapping_mul(::core::mem::size_of::<KeyValuePair>()),
            );
            v.size = version.size;
        }
        let c2rust_fresh15 = v.size;
        v.size = v.size.wrapping_add(1);
        *v.items.offset(c2rust_fresh15 as isize) = key_value_pair {
            key: cstr_as_string(b"major\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: 0 as Integer,
                },
            },
        };
        version = v;
    }
    let c2rust_fresh16 = info.size;
    info.size = info.size.wrapping_add(1);
    *info.items.offset(c2rust_fresh16 as isize) = key_value_pair {
        key: cstr_as_string(b"version\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeDict,
            data: C2Rust_Unnamed { dict: version },
        },
    };
    let c2rust_fresh17 = info.size;
    info.size = info.size.wrapping_add(1);
    *info.items.offset(c2rust_fresh17 as isize) = key_value_pair {
        key: cstr_as_string(b"type\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed { string: type_0 },
        },
    };
    let c2rust_fresh18 = info.size;
    info.size = info.size.wrapping_add(1);
    *info.items.offset(c2rust_fresh18 as isize) = key_value_pair {
        key: cstr_as_string(b"methods\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeDict,
            data: C2Rust_Unnamed { dict: methods },
        },
    };
    let c2rust_fresh19 = info.size;
    info.size = info.size.wrapping_add(1);
    *info.items.offset(c2rust_fresh19 as isize) = key_value_pair {
        key: cstr_as_string(b"attributes\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeDict,
            data: C2Rust_Unnamed { dict: attributes },
        },
    };
    rpc_set_client_info(
        channel_id,
        copy_dict(info, ::core::ptr::null_mut::<Arena>()),
    );
}
#[no_mangle]
pub unsafe extern "C" fn nvim__chan_set_detach(
    mut channel_id: uint64_t,
    mut detach: Boolean,
    mut err: *mut Error,
) {
    let mut chan: *mut Channel = find_channel(channel_id);
    if chan.is_null() {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            &raw const e_invchan as *const ::core::ffi::c_char,
        );
        return;
    }
    (*chan).detach = detach;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_get_chan_info(
    mut channel_id: uint64_t,
    mut chan: Integer,
    mut arena: *mut Arena,
    mut _err: *mut Error,
) -> Dict {
    if chan < 0 as Integer {
        return Dict {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        };
    }
    if chan == 0 as Integer && !is_internal_call(channel_id) {
        '_c2rust_label: {
            if channel_id <= 9223372036854775807 as uint64_t {
            } else {
                __assert_fail(
                    b"channel_id <= INT64_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/api/vim.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1800 as ::core::ffi::c_uint,
                    b"Dict nvim_get_chan_info(uint64_t, Integer, Arena *, Error *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        chan = channel_id as Integer;
    }
    return channel_info(chan as uint64_t, arena);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_list_chans(mut arena: *mut Arena) -> Array {
    return channel_all_info(arena);
}
#[no_mangle]
pub unsafe extern "C" fn nvim__id(mut obj: Object, mut arena: *mut Arena) -> Object {
    return copy_object(obj, arena);
}
#[no_mangle]
pub unsafe extern "C" fn nvim__id_array(mut arr: Array, mut arena: *mut Arena) -> Array {
    return copy_array(arr, arena);
}
#[no_mangle]
pub unsafe extern "C" fn nvim__id_dict(mut dct: Dict, mut arena: *mut Arena) -> Dict {
    return copy_dict(dct, arena);
}
#[no_mangle]
pub unsafe extern "C" fn nvim__id_float(mut flt: Float) -> Float {
    return flt;
}
#[no_mangle]
pub unsafe extern "C" fn nvim__stats(mut arena: *mut Arena) -> Dict {
    let mut rv: Dict = arena_dict(arena, 6 as size_t);
    let c2rust_fresh20 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh20 as isize) = key_value_pair {
        key: cstr_as_string(b"fsync\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: g_stats.fsync,
            },
        },
    };
    let c2rust_fresh21 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh21 as isize) = key_value_pair {
        key: cstr_as_string(b"log_skip\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: g_stats.log_skip as Integer,
            },
        },
    };
    let c2rust_fresh22 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh22 as isize) = key_value_pair {
        key: cstr_as_string(b"lua_refcount\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: nlua_get_global_ref_count() as Integer,
            },
        },
    };
    let c2rust_fresh23 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh23 as isize) = key_value_pair {
        key: cstr_as_string(b"redraw\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: g_stats.redraw,
            },
        },
    };
    let c2rust_fresh24 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh24 as isize) = key_value_pair {
        key: cstr_as_string(b"arena_alloc_count\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: arena_alloc_count as Integer,
            },
        },
    };
    let c2rust_fresh25 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh25 as isize) = key_value_pair {
        key: cstr_as_string(b"ts_query_parse_count\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: tslua_query_parse_count as Integer,
            },
        },
    };
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_list_uis(mut arena: *mut Arena) -> Array {
    return ui_array(arena);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_get_proc_children(
    mut pid: Integer,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    let mut proc_count: size_t = 0;
    let mut rv: ::core::ffi::c_int = 0;
    let mut rvobj: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut proc_list: *mut ::core::ffi::c_int = ::core::ptr::null_mut::<::core::ffi::c_int>();
    if !(pid > 0 as Integer && pid <= 2147483647 as Integer) {
        api_err_invalid(
            err,
            b"pid\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
            pid as int64_t,
            false_0 != 0,
        );
    } else {
        proc_count = 0;
        rv = os_proc_children(
            pid as ::core::ffi::c_int,
            &raw mut proc_list,
            &raw mut proc_count,
        );
        if rv == 2 as ::core::ffi::c_int {
            logmsg(
                LOGLVL_DBG,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"nvim_get_proc_children\0".as_ptr() as *const ::core::ffi::c_char,
                1924 as ::core::ffi::c_int,
                true_0 != 0,
                b"fallback to vim._os_proc_children()\0".as_ptr() as *const ::core::ffi::c_char,
            );
            let mut a: Array = Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            };
            let mut a__items: [Object; 1] = [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            }; 1];
            a.capacity = 1 as size_t;
            a.items = &raw mut a__items as *mut Object;
            let c2rust_fresh26 = a.size;
            a.size = a.size.wrapping_add(1);
            *a.items.offset(c2rust_fresh26 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed { integer: pid },
            };
            let mut o: Object = nlua_exec(
                String_0 {
                    data: b"return vim._os_proc_children(...)\0".as_ptr()
                        as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 34]>()
                        .wrapping_sub(1 as size_t),
                },
                ::core::ptr::null::<::core::ffi::c_char>(),
                a,
                kRetObject,
                arena,
                err,
            );
            if o.type_0 as ::core::ffi::c_uint
                == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                rvobj = o.data.array;
            } else if !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int)
            {
                api_set_error(
                    err,
                    kErrorTypeException,
                    b"Failed to get process children. pid=%ld error=%d\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    pid,
                    rv,
                );
            }
        } else {
            rvobj = arena_array(arena, proc_count);
            let mut i: size_t = 0 as size_t;
            while i < proc_count {
                let c2rust_fresh27 = rvobj.size;
                rvobj.size = rvobj.size.wrapping_add(1);
                *rvobj.items.offset(c2rust_fresh27 as isize) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: *proc_list.offset(i as isize) as Integer,
                    },
                };
                i = i.wrapping_add(1);
            }
        }
    }
    xfree(proc_list as *mut ::core::ffi::c_void);
    return rvobj;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_get_proc(
    mut pid: Integer,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    let mut rvobj: Object = object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    if !(pid > 0 as Integer && pid <= 2147483647 as Integer) {
        api_err_invalid(
            err,
            b"pid\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
            pid as int64_t,
            false_0 != 0,
        );
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    let mut a: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut a__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 1];
    a.capacity = 1 as size_t;
    a.items = &raw mut a__items as *mut Object;
    if a.size == a.capacity {
        a.capacity = if a.capacity != 0 {
            a.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        a.items = xrealloc(
            a.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<Object>().wrapping_mul(a.capacity),
        ) as *mut Object;
    } else {
    };
    let c2rust_fresh28 = a.size;
    a.size = a.size.wrapping_add(1);
    *a.items.offset(c2rust_fresh28 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: pid },
    };
    let mut o: Object = nlua_exec(
        String_0 {
            data: b"return vim._os_proc_info(...)\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            size: ::core::mem::size_of::<[::core::ffi::c_char; 30]>().wrapping_sub(1 as size_t),
        },
        ::core::ptr::null::<::core::ffi::c_char>(),
        a,
        kRetObject,
        arena,
        err,
    );
    if o.type_0 as ::core::ffi::c_uint
        == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
        && o.data.array.size == 0 as size_t
    {
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    } else if o.type_0 as ::core::ffi::c_uint
        == kObjectTypeDict as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        rvobj = o;
    } else if !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
        api_set_error(
            err,
            kErrorTypeException,
            b"Failed to get process info. pid=%ld\0".as_ptr() as *const ::core::ffi::c_char,
            pid,
        );
    }
    return rvobj;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_select_popupmenu_item(
    mut item: Integer,
    mut insert: Boolean,
    mut finish: Boolean,
    mut _opts: *mut KeyDict_empty,
    mut _err: *mut Error,
) {
    if finish {
        insert = true_0 != 0;
    }
    pum_ext_select_item(item as ::core::ffi::c_int, insert as bool, finish as bool);
}
#[no_mangle]
pub unsafe extern "C" fn nvim__inspect_cell(
    mut grid: Integer,
    mut row: Integer,
    mut col: Integer,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    let mut ret: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut g: *mut ScreenGrid = &raw mut default_grid;
    if grid == pum_grid.handle as Integer {
        g = &raw mut pum_grid;
    } else if grid > 1 as Integer {
        let mut wp: *mut win_T = get_win_by_grid_handle(grid as handle_T);
        if !(!wp.is_null() && !(*wp).w_grid_alloc.chars.is_null()) {
            api_err_invalid(
                err,
                b"grid handle\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<::core::ffi::c_char>(),
                grid as int64_t,
                false_0 != 0,
            );
            return ret;
        }
        g = &raw mut (*wp).w_grid_alloc;
    }
    if row < 0 as Integer
        || row >= (*g).rows as Integer
        || col < 0 as Integer
        || col >= (*g).cols as Integer
    {
        return ret;
    }
    ret = arena_array(arena, 3 as size_t);
    let mut off: size_t =
        (*(*g).line_offset.offset(row as size_t as isize)).wrapping_add(col as size_t);
    let mut sc_buf: *mut ::core::ffi::c_char =
        arena_alloc(arena, MAX_SCHAR_SIZE as size_t, false_0 != 0) as *mut ::core::ffi::c_char;
    schar_get(sc_buf, *(*g).chars.offset(off as isize));
    let c2rust_fresh29 = ret.size;
    ret.size = ret.size.wrapping_add(1);
    *ret.items.offset(c2rust_fresh29 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed {
            string: cstr_as_string(sc_buf),
        },
    };
    let mut attr: ::core::ffi::c_int = *(*g).attrs.offset(off as isize) as ::core::ffi::c_int;
    let c2rust_fresh30 = ret.size;
    ret.size = ret.size.wrapping_add(1);
    *ret.items.offset(c2rust_fresh30 as isize) = object {
        type_0: kObjectTypeDict,
        data: C2Rust_Unnamed {
            dict: hl_get_attr_by_id(attr as Integer, true, arena, err),
        },
    };
    if !highlight_use_hlstate() {
        let c2rust_fresh31 = ret.size;
        ret.size = ret.size.wrapping_add(1);
        *ret.items.offset(c2rust_fresh31 as isize) = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed {
                array: hl_inspect(attr, arena),
            },
        };
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn nvim__screenshot(mut path: String_0) {
    ui_call_screenshot(path);
}
#[no_mangle]
pub unsafe extern "C" fn nvim__invalidate_glyph_cache() {
    schar_cache_clear();
    must_redraw = UPD_CLEAR as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn nvim__unpack(
    mut str: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    return unpack(str.data, str.size, arena, err);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_del_mark(mut name: String_0, mut err: *mut Error) -> Boolean {
    let mut res: bool = false_0 != 0;
    if !(name.size == 1 as size_t) {
        api_err_invalid(
            err,
            b"mark name (must be a single char)\0".as_ptr() as *const ::core::ffi::c_char,
            name.data,
            0 as int64_t,
            true_0 != 0,
        );
        return res as Boolean;
    }
    if !(*name.data as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && *name.data as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
        || ascii_isdigit(*name.data as ::core::ffi::c_int) as ::core::ffi::c_int != 0)
    {
        api_err_invalid(
            err,
            b"mark name (must be file/uppercase)\0".as_ptr() as *const ::core::ffi::c_char,
            name.data,
            0 as int64_t,
            true_0 != 0,
        );
        return res as Boolean;
    }
    res = set_mark(
        ::core::ptr::null_mut::<buf_T>(),
        name,
        0 as Integer,
        0 as Integer,
        err,
    );
    return res as Boolean;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_get_mark(
    mut name: String_0,
    mut _opts: *mut KeyDict_empty,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    let mut rv: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    if !(name.size == 1 as size_t) {
        api_err_invalid(
            err,
            b"mark name (must be a single char)\0".as_ptr() as *const ::core::ffi::c_char,
            name.data,
            0 as int64_t,
            true_0 != 0,
        );
        return rv;
    }
    if !(*name.data as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && *name.data as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
        || ascii_isdigit(*name.data as ::core::ffi::c_int) as ::core::ffi::c_int != 0)
    {
        api_err_invalid(
            err,
            b"mark name (must be file/uppercase)\0".as_ptr() as *const ::core::ffi::c_char,
            name.data,
            0 as int64_t,
            true_0 != 0,
        );
        return rv;
    }
    let mut mark: *mut xfmark_T = mark_get_global(false_0 != 0, *name.data as ::core::ffi::c_int);
    let mut pos: pos_T = (*mark).fmark.mark;
    let mut allocated: bool = false_0 != 0;
    let mut bufnr: ::core::ffi::c_int = 0;
    let mut filename: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*mark).fmark.fnum != 0 as ::core::ffi::c_int {
        bufnr = (*mark).fmark.fnum;
        filename = buflist_nr2name(bufnr, true_0, true_0);
        allocated = true_0 != 0;
    } else {
        filename = (*mark).fname;
        bufnr = 0 as ::core::ffi::c_int;
    }
    let mut exists: bool = !filename.is_null();
    let mut row: Integer = 0;
    let mut col: Integer = 0;
    if !exists || pos.lnum <= 0 as linenr_T {
        if allocated {
            xfree(filename as *mut ::core::ffi::c_void);
            allocated = false_0 != 0;
        }
        filename = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        bufnr = 0 as ::core::ffi::c_int;
        row = 0 as Integer;
        col = 0 as Integer;
    } else {
        row = pos.lnum as Integer;
        col = pos.col as Integer;
    }
    rv = arena_array(arena, 4 as size_t);
    let c2rust_fresh32 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh32 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: row },
    };
    let c2rust_fresh33 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh33 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: col },
    };
    let c2rust_fresh34 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh34 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: bufnr as Integer,
        },
    };
    let c2rust_fresh35 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh35 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed {
            string: arena_string(arena, cstr_as_string(filename)),
        },
    };
    if allocated {
        xfree(filename as *mut ::core::ffi::c_void);
    }
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_eval_statusline(
    mut str: String_0,
    mut opts: *mut KeyDict_eval_statusline,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    let mut result: Dict = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut maxwidth: ::core::ffi::c_int = 0;
    let mut fillchar: schar_T = 0 as schar_T;
    let mut statuscol_lnum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if str.size < 2 as size_t
        || memcmp(
            str.data as *const ::core::ffi::c_void,
            b"%!\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            2 as size_t,
        ) != 0 as ::core::ffi::c_int
    {
        let errmsg: *const ::core::ffi::c_char = check_stl_option(str.data);
        if !errmsg.is_null() {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                errmsg,
            );
            return result;
        }
    }
    let mut window: Window = (*opts).winid;
    if (*opts).is_set__eval_statusline_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_eval_statusline__fillchar
        != 0 as ::core::ffi::c_ulonglong
    {
        if !(*(*opts).fillchar.data as ::core::ffi::c_int != 0 as ::core::ffi::c_int
            && utfc_ptr2len((*opts).fillchar.data) as size_t == (*opts).fillchar.size)
        {
            api_err_exp(
                err,
                b"fillchar\0".as_ptr() as *const ::core::ffi::c_char,
                b"single character\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<::core::ffi::c_char>(),
            );
            return result;
        }
        let mut c: ::core::ffi::c_int = 0;
        fillchar = utfc_ptr2schar((*opts).fillchar.data, &raw mut c);
    }
    let mut use_bools: ::core::ffi::c_int =
        (*opts).use_winbar as ::core::ffi::c_int + (*opts).use_tabline as ::core::ffi::c_int;
    let mut wp: *mut win_T = if (*opts).use_tabline as ::core::ffi::c_int != 0 {
        curwin
    } else {
        find_window_by_handle(window, err)
    };
    if wp.is_null() {
        api_set_error(
            err,
            kErrorTypeException,
            b"unknown winid %d\0".as_ptr() as *const ::core::ffi::c_char,
            window,
        );
        return result;
    }
    if (*opts).is_set__eval_statusline_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_eval_statusline__use_statuscol_lnum
        != 0 as ::core::ffi::c_ulonglong
    {
        statuscol_lnum = (*opts).use_statuscol_lnum as ::core::ffi::c_int;
        if !(statuscol_lnum > 0 as ::core::ffi::c_int
            && statuscol_lnum as linenr_T <= (*(*wp).w_buffer).b_ml.ml_line_count)
        {
            api_err_invalid(
                err,
                b"use_statuscol_lnum\0".as_ptr() as *const ::core::ffi::c_char,
                b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                0 as int64_t,
                false_0 != 0,
            );
            return result;
        }
        use_bools += 1;
    }
    if !(use_bools <= 1 as ::core::ffi::c_int) {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            b"Can only use one of 'use_winbar', 'use_tabline' and 'use_statuscol_lnum'\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
        return result;
    }
    let mut stc_hl_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut scl_hl_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut statuscol: statuscol_T = statuscol_T {
        width: 0 as ::core::ffi::c_int,
        lnum: 0,
        sign_cul_id: 0,
        draw: false,
        hlrec: ::core::ptr::null_mut::<stl_hlrec_t>(),
        foldinfo: foldinfo_T {
            fi_lnum: 0,
            fi_level: 0,
            fi_low_level: 0,
            fi_lines: 0,
        },
        fold_vcol: [0; 9],
        sattrs: ::core::ptr::null_mut::<SignTextAttrs>(),
    };
    let mut sattrs: [SignTextAttrs; 9] = [
        SignTextAttrs {
            text: [0 as schar_T, 0],
            hl_id: 0,
        },
        SignTextAttrs {
            text: [0; 2],
            hl_id: 0,
        },
        SignTextAttrs {
            text: [0; 2],
            hl_id: 0,
        },
        SignTextAttrs {
            text: [0; 2],
            hl_id: 0,
        },
        SignTextAttrs {
            text: [0; 2],
            hl_id: 0,
        },
        SignTextAttrs {
            text: [0; 2],
            hl_id: 0,
        },
        SignTextAttrs {
            text: [0; 2],
            hl_id: 0,
        },
        SignTextAttrs {
            text: [0; 2],
            hl_id: 0,
        },
        SignTextAttrs {
            text: [0; 2],
            hl_id: 0,
        },
    ];
    if statuscol_lnum != 0 {
        let mut line_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut cul_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut num_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut lnum: linenr_T = statuscol_lnum as linenr_T;
        let mut cursorline_fi: foldinfo_T = foldinfo_T {
            fi_lnum: 0 as linenr_T,
            fi_level: 0,
            fi_low_level: 0,
            fi_lines: 0,
        };
        decor_redraw_signs(
            wp,
            (*wp).w_buffer,
            lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            &raw mut sattrs as *mut SignTextAttrs,
            &raw mut line_id,
            &raw mut cul_id,
            &raw mut num_id,
        );
        statuscol.sattrs = &raw mut sattrs as *mut SignTextAttrs;
        statuscol.foldinfo = fold_info(wp, lnum);
        win_update_cursorline(wp, &raw mut cursorline_fi);
        statuscol.sign_cul_id = if use_cursor_line_highlight(wp, lnum) as ::core::ffi::c_int != 0 {
            cul_id
        } else {
            0 as ::core::ffi::c_int
        };
        scl_hl_id = if use_cursor_line_highlight(wp, lnum) as ::core::ffi::c_int != 0 {
            HLF_CLS as ::core::ffi::c_int
        } else {
            HLF_SC as ::core::ffi::c_int
        };
        if num_id != 0 {
            stc_hl_id = num_id;
        } else if use_cursor_line_highlight(wp, lnum) {
            stc_hl_id = HLF_CLN as ::core::ffi::c_int;
        } else if (*wp).w_onebuf_opt.wo_rnu != 0 {
            stc_hl_id = if lnum < (*wp).w_cursor.lnum {
                HLF_LNA as ::core::ffi::c_int
            } else {
                HLF_LNB as ::core::ffi::c_int
            };
        } else {
            stc_hl_id = HLF_N as ::core::ffi::c_int;
        }
        set_vim_var_nr(VV_LNUM, lnum as varnumber_T);
        set_vim_var_nr(
            VV_RELNUM,
            labs(get_cursor_rel_lnum(wp, lnum) as ::core::ffi::c_long) as varnumber_T,
        );
        set_vim_var_nr(VV_VIRTNUM, 0 as varnumber_T);
    } else if fillchar == 0 as schar_T && !(*opts).use_tabline {
        if (*opts).use_winbar {
            fillchar = (*wp).w_p_fcs_chars.wbr;
        } else {
            let mut group: hlf_T = HLF_NONE;
            fillchar = fillchar_status(&raw mut group, wp);
        }
    }
    if (*opts).is_set__eval_statusline_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_eval_statusline__maxwidth
        != 0 as ::core::ffi::c_ulonglong
    {
        maxwidth = (*opts).maxwidth as ::core::ffi::c_int;
    } else {
        maxwidth = if statuscol_lnum != 0 {
            win_col_off(wp)
        } else if (*opts).use_tabline as ::core::ffi::c_int != 0
            || !(*opts).use_winbar && global_stl_height() > 0 as ::core::ffi::c_int
        {
            Columns
        } else {
            (*wp).w_width
        };
    }
    result = arena_dict(arena, 3 as size_t);
    let mut buf: *mut ::core::ffi::c_char =
        arena_alloc(arena, MAXPATHL as size_t, false_0 != 0) as *mut ::core::ffi::c_char;
    let mut hltab: *mut stl_hlrec_t = ::core::ptr::null_mut::<stl_hlrec_t>();
    let mut hltab_len: size_t = 0 as size_t;
    let mut p_crb_save: ::core::ffi::c_int = (*wp).w_onebuf_opt.wo_crb;
    (*wp).w_onebuf_opt.wo_crb = false_0;
    let mut width: ::core::ffi::c_int = build_stl_str_hl(
        wp,
        buf,
        MAXPATHL as size_t,
        str.data,
        kOptInvalid,
        0 as ::core::ffi::c_int,
        fillchar,
        maxwidth,
        if (*opts).highlights as ::core::ffi::c_int != 0 {
            &raw mut hltab
        } else {
            ::core::ptr::null_mut::<*mut stl_hlrec_t>()
        },
        &raw mut hltab_len,
        ::core::ptr::null_mut::<*mut StlClickRecord>(),
        if statuscol_lnum != 0 {
            &raw mut statuscol
        } else {
            ::core::ptr::null_mut::<statuscol_T>()
        },
    );
    let c2rust_fresh36 = result.size;
    result.size = result.size.wrapping_add(1);
    *result.items.offset(c2rust_fresh36 as isize) = key_value_pair {
        key: cstr_as_string(b"width\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: width as Integer,
            },
        },
    };
    (*wp).w_onebuf_opt.wo_crb = p_crb_save;
    if (*opts).highlights {
        let mut hl_values: Array = arena_array(arena, hltab_len.wrapping_add(1 as size_t));
        let mut user_group: [::core::ffi::c_char; 15] = [0; 15];
        let mut dfltname: *const ::core::ffi::c_char = get_default_stl_hl(
            if (*opts).use_tabline as ::core::ffi::c_int != 0 {
                ::core::ptr::null_mut::<win_T>()
            } else {
                wp
            },
            (*opts).use_winbar as bool,
            stc_hl_id,
        );
        if (*hltab).start.is_null() || (*hltab).start.offset_from(buf) != 0 as isize {
            let mut hl_info: Dict = arena_dict(arena, 3 as size_t);
            let c2rust_fresh37 = hl_info.size;
            hl_info.size = hl_info.size.wrapping_add(1);
            *hl_info.items.offset(c2rust_fresh37 as isize) = key_value_pair {
                key: cstr_as_string(b"start\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: 0 as Integer,
                    },
                },
            };
            let c2rust_fresh38 = hl_info.size;
            hl_info.size = hl_info.size.wrapping_add(1);
            *hl_info.items.offset(c2rust_fresh38 as isize) = key_value_pair {
                key: cstr_as_string(b"group\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string(dfltname),
                    },
                },
            };
            let mut groups: Array = arena_array(arena, 1 as size_t);
            let c2rust_fresh39 = groups.size;
            groups.size = groups.size.wrapping_add(1);
            *groups.items.offset(c2rust_fresh39 as isize) = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string(dfltname),
                },
            };
            let c2rust_fresh40 = hl_info.size;
            hl_info.size = hl_info.size.wrapping_add(1);
            *hl_info.items.offset(c2rust_fresh40 as isize) = key_value_pair {
                key: cstr_as_string(b"groups\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeArray,
                    data: C2Rust_Unnamed { array: groups },
                },
            };
            let c2rust_fresh41 = hl_values.size;
            hl_values.size = hl_values.size.wrapping_add(1);
            *hl_values.items.offset(c2rust_fresh41 as isize) = object {
                type_0: kObjectTypeDict,
                data: C2Rust_Unnamed { dict: hl_info },
            };
        }
        let mut sp: *mut stl_hlrec_t = hltab;
        while !(*sp).start.is_null() {
            let mut grpname: *const ::core::ffi::c_char =
                ::core::ptr::null::<::core::ffi::c_char>();
            if (*sp).userhl == 0 as ::core::ffi::c_int {
                grpname = get_default_stl_hl(
                    if (*opts).use_tabline as ::core::ffi::c_int != 0 {
                        ::core::ptr::null_mut::<win_T>()
                    } else {
                        wp
                    },
                    (*opts).use_winbar as bool,
                    stc_hl_id,
                );
            } else if (*sp).userhl < 0 as ::core::ffi::c_int {
                grpname = syn_id2name(-(*sp).userhl);
            } else {
                snprintf(
                    &raw mut user_group as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 15]>(),
                    b"User%d\0".as_ptr() as *const ::core::ffi::c_char,
                    (*sp).userhl,
                );
                grpname = arena_strdup(arena, &raw mut user_group as *mut ::core::ffi::c_char);
            }
            let mut combine: *const ::core::ffi::c_char = if (*sp).item as ::core::ffi::c_uint
                == STL_SIGNCOL as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                syn_id2name(scl_hl_id) as *const ::core::ffi::c_char
            } else if (*sp).item as ::core::ffi::c_uint
                == STL_FOLDCOL as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                grpname
            } else {
                dfltname
            };
            let mut hl_info_0: Dict = arena_dict(arena, 3 as size_t);
            let c2rust_fresh42 = hl_info_0.size;
            hl_info_0.size = hl_info_0.size.wrapping_add(1);
            *hl_info_0.items.offset(c2rust_fresh42 as isize) = key_value_pair {
                key: cstr_as_string(b"start\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: (*sp).start.offset_from(buf) as i64,
                    },
                },
            };
            let c2rust_fresh43 = hl_info_0.size;
            hl_info_0.size = hl_info_0.size.wrapping_add(1);
            *hl_info_0.items.offset(c2rust_fresh43 as isize) = key_value_pair {
                key: cstr_as_string(b"group\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string(grpname),
                    },
                },
            };
            let mut groups_0: Array = arena_array(
                arena,
                (1 as ::core::ffi::c_int + (combine != grpname) as ::core::ffi::c_int) as size_t,
            );
            if combine != grpname {
                let c2rust_fresh44 = groups_0.size;
                groups_0.size = groups_0.size.wrapping_add(1);
                *groups_0.items.offset(c2rust_fresh44 as isize) = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string(combine),
                    },
                };
            }
            let c2rust_fresh45 = groups_0.size;
            groups_0.size = groups_0.size.wrapping_add(1);
            *groups_0.items.offset(c2rust_fresh45 as isize) = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string(grpname),
                },
            };
            let c2rust_fresh46 = hl_info_0.size;
            hl_info_0.size = hl_info_0.size.wrapping_add(1);
            *hl_info_0.items.offset(c2rust_fresh46 as isize) = key_value_pair {
                key: cstr_as_string(b"groups\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeArray,
                    data: C2Rust_Unnamed { array: groups_0 },
                },
            };
            let c2rust_fresh47 = hl_values.size;
            hl_values.size = hl_values.size.wrapping_add(1);
            *hl_values.items.offset(c2rust_fresh47 as isize) = object {
                type_0: kObjectTypeDict,
                data: C2Rust_Unnamed { dict: hl_info_0 },
            };
            sp = sp.offset(1);
        }
        let c2rust_fresh48 = result.size;
        result.size = result.size.wrapping_add(1);
        *result.items.offset(c2rust_fresh48 as isize) = key_value_pair {
            key: cstr_as_string(b"highlights\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed { array: hl_values },
            },
        };
    }
    let c2rust_fresh49 = result.size;
    result.size = result.size.wrapping_add(1);
    *result.items.offset(c2rust_fresh49 as isize) = key_value_pair {
        key: cstr_as_string(b"str\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string(buf),
            },
        },
    };
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn nvim__complete_set(
    mut index: Integer,
    mut opts: *mut KeyDict_complete_set,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    let mut rv: Dict = arena_dict(arena, 2 as size_t);
    if get_cot_flags() & kOptCotFlagPopup as ::core::ffi::c_int as ::core::ffi::c_uint
        == 0 as ::core::ffi::c_uint
    {
        api_set_error(
            err,
            kErrorTypeException,
            b"completeopt option does not include popup\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return rv;
    }
    if (*opts).is_set__complete_set_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_complete_set__info
        != 0 as ::core::ffi::c_ulonglong
    {
        let mut wp: *mut win_T = pum_set_info(index as ::core::ffi::c_int, (*opts).info.data);
        if !wp.is_null() {
            let c2rust_fresh50 = rv.size;
            rv.size = rv.size.wrapping_add(1);
            *rv.items.offset(c2rust_fresh50 as isize) = key_value_pair {
                key: cstr_as_string(b"winid\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeWindow,
                    data: C2Rust_Unnamed {
                        integer: (*wp).handle as Integer,
                    },
                },
            };
            let c2rust_fresh51 = rv.size;
            rv.size = rv.size.wrapping_add(1);
            *rv.items.offset(c2rust_fresh51 as isize) = key_value_pair {
                key: cstr_as_string(b"bufnr\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeBuffer,
                    data: C2Rust_Unnamed {
                        integer: (*(*wp).w_buffer).handle as Integer,
                    },
                },
            };
        }
    }
    return rv;
}
unsafe extern "C" fn redraw_status(
    mut wp: *mut win_T,
    mut opts: *mut KeyDict_redraw,
    mut flush: *mut bool,
) {
    if (*opts).statuscolumn as ::core::ffi::c_int != 0
        && *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int != NUL
    {
        (*wp).w_nrwidth_line_count = 0 as ::core::ffi::c_int as linenr_T;
        changed_window_setting(wp);
    }
    let mut old_row_offset: ::core::ffi::c_int = (*wp).w_grid.row_offset;
    win_grid_alloc(wp);
    if (*wp).w_lines_valid == 0 as ::core::ffi::c_int || (*wp).w_grid.row_offset != old_row_offset {
        *flush = true_0 != 0;
    }
    if *flush as ::core::ffi::c_int != 0
        && ((*opts).statusline as ::core::ffi::c_int != 0
            || (*opts).winbar as ::core::ffi::c_int != 0)
    {
        (*wp).w_redr_status = true_0 != 0;
    } else if (*opts).statusline as ::core::ffi::c_int != 0
        || (*opts).winbar as ::core::ffi::c_int != 0
    {
        win_check_ns_hl(wp);
        if (*opts).winbar {
            win_redr_winbar(wp);
        }
        if (*opts).statusline {
            win_redr_status(wp);
        }
        win_check_ns_hl(::core::ptr::null_mut::<win_T>());
    }
}
#[no_mangle]
pub unsafe extern "C" fn nvim__redraw(mut opts: *mut KeyDict_redraw, mut err: *mut Error) {
    let mut win: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    if (*opts).is_set__redraw_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_redraw__win
        != 0 as ::core::ffi::c_ulonglong
    {
        win = find_window_by_handle((*opts).win, err);
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return;
        }
    }
    if (*opts).is_set__redraw_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_redraw__buf
        != 0 as ::core::ffi::c_ulonglong
    {
        if !win.is_null() {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                b"cannot use both 'buf' and 'win'\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return;
        }
        buf = find_buffer_by_handle((*opts).buf, err);
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return;
        }
    }
    let mut count: ::core::ffi::c_uint = (!win.is_null() as ::core::ffi::c_int
        + !buf.is_null() as ::core::ffi::c_int)
        as ::core::ffi::c_uint;
    if !(xpopcount((*opts).is_set__redraw_ as uint64_t) > count) {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            b"at least one action required\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    if (*opts).is_set__redraw_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_redraw__valid
        != 0 as ::core::ffi::c_ulonglong
    {
        let mut type_0: ::core::ffi::c_int = if (*opts).valid as ::core::ffi::c_int != 0 {
            UPD_VALID as ::core::ffi::c_int
        } else {
            UPD_NOT_VALID as ::core::ffi::c_int
        };
        if !win.is_null() {
            redraw_later(win, type_0);
        } else if !buf.is_null() {
            redraw_buf_later(buf, type_0);
        } else {
            redraw_all_later(type_0);
        }
    }
    if (*opts).is_set__redraw_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_redraw__range
        != 0 as ::core::ffi::c_ulonglong
    {
        if !((*opts).range.size == 2 as size_t
            && (*(*opts).range.items.offset(0 as ::core::ffi::c_int as isize)).type_0
                as ::core::ffi::c_uint
                == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*(*opts).range.items.offset(1 as ::core::ffi::c_int as isize)).type_0
                as ::core::ffi::c_uint
                == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*(*opts).range.items.offset(0 as ::core::ffi::c_int as isize))
                .data
                .integer
                >= 0 as Integer
            && (*(*opts).range.items.offset(1 as ::core::ffi::c_int as isize))
                .data
                .integer
                >= -1 as Integer)
        {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                b"Invalid 'range': Expected 2-tuple of Integers\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
            return;
        }
        let mut begin_raw: int64_t = (*(*opts).range.items.offset(0 as ::core::ffi::c_int as isize))
            .data
            .integer as int64_t;
        let mut end_raw: int64_t = (*(*opts).range.items.offset(1 as ::core::ffi::c_int as isize))
            .data
            .integer as int64_t;
        let mut rbuf: *mut buf_T = if !win.is_null() {
            (*win).w_buffer
        } else if !buf.is_null() {
            buf
        } else {
            curbuf
        };
        let mut line_count: linenr_T = (*rbuf).b_ml.ml_line_count;
        let mut begin: ::core::ffi::c_int = (if begin_raw < line_count as int64_t {
            begin_raw
        } else {
            line_count as int64_t
        }) as ::core::ffi::c_int;
        let mut end: ::core::ffi::c_int = 0;
        if end_raw == -1 as int64_t {
            end = line_count as ::core::ffi::c_int;
        } else {
            end = (if (if begin as int64_t > end_raw {
                begin as int64_t
            } else {
                end_raw
            }) < line_count as int64_t
            {
                if begin as int64_t > end_raw {
                    begin as int64_t
                } else {
                    end_raw
                }
            } else {
                line_count as int64_t
            }) as ::core::ffi::c_int;
        }
        if begin < end {
            redraw_buf_range_later(rbuf, 1 as linenr_T + begin as linenr_T, end as linenr_T);
        }
    }
    if (*opts).is_set__redraw_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_redraw__valid
        != 0 as ::core::ffi::c_ulonglong
        || (*opts).is_set__redraw_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_redraw__range
            != 0 as ::core::ffi::c_ulonglong
    {
        (*opts).flush = if (*opts).is_set__redraw_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_redraw__flush
            != 0 as ::core::ffi::c_ulonglong
        {
            (*opts).flush as ::core::ffi::c_int
        } else {
            true_0
        } != 0;
    }
    let mut flush_ui: bool = (*opts).flush as bool;
    if (*opts).tabline {
        if redraw_tabline as ::core::ffi::c_int != 0
            && (*firstwin).w_lines_valid == 0 as ::core::ffi::c_int
        {
            (*opts).flush = true_0 != 0;
        } else {
            draw_tabline();
        }
        flush_ui = true_0 != 0;
    }
    let mut save_lz: bool = p_lz != 0;
    let mut save_rd: ::core::ffi::c_int = RedrawingDisabled;
    RedrawingDisabled = 0 as ::core::ffi::c_int;
    p_lz = false_0;
    if (*opts).statuscolumn as ::core::ffi::c_int != 0
        || (*opts).statusline as ::core::ffi::c_int != 0
        || (*opts).winbar as ::core::ffi::c_int != 0
    {
        if win.is_null() {
            let mut wp: *mut win_T = if curtab == curtab {
                firstwin
            } else {
                (*curtab).tp_firstwin
            };
            while !wp.is_null() {
                if buf.is_null() || (*wp).w_buffer == buf {
                    redraw_status(wp, opts, &raw mut (*opts).flush);
                }
                wp = (*wp).w_next;
            }
        } else {
            redraw_status(win, opts, &raw mut (*opts).flush);
        }
        flush_ui = true_0 != 0;
    }
    let mut cwin: *mut win_T = if !win.is_null() { win } else { curwin };
    if (*opts).cursor as ::core::ffi::c_int != 0
        && ((*cwin).w_grid.target.is_null() || !(*(*cwin).w_grid.target).valid)
    {
        (*opts).flush = true_0 != 0;
    }
    if (*opts).flush as ::core::ffi::c_int != 0 && !cmdpreview {
        validate_cursor(curwin);
        update_topline(curwin);
        update_screen();
    }
    if (*opts).cursor {
        setcursor_mayforce(cwin, true_0 != 0);
        flush_ui = true_0 != 0;
    }
    if flush_ui {
        ui_flush();
    }
    RedrawingDisabled = save_rd;
    p_lz = save_lz as ::core::ffi::c_int;
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn buf_get_changedtick(buf: *const buf_T) -> varnumber_T {
    return (*buf).changedtick_di.di_tv.vval.v_number;
}
#[inline]
unsafe extern "C" fn find_channel(mut id: uint64_t) -> *mut Channel {
    return map_get_uint64_t_ptr_t(&raw mut channels, id) as *mut Channel;
}
pub const CONTEXT_INIT: Context = Context {
    regs: STRING_INIT,
    jumps: STRING_INIT,
    bufs: STRING_INIT,
    gvars: STRING_INIT,
    funcs: Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    },
};
pub const MODE_MAX_LENGTH: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
